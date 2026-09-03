use super::support::{isolated_xdg, test_app};
use crate::app::login::{LoginFlow, OpenCodeGoLoginFlow, reauthenticate, start_login};
use crate::config::{Config, ManagedOpenCodeGoAccountConfig};
use crate::model::ProviderId;
use crate::providers::opencode_go::login::prepare_for_reauth;
use crate::providers::opencode_go::login::{OpenCodeGoLoginEvent, OpenCodeGoLoginState};
use crate::providers::opencode_go::storage::{load_api_key, write_api_key};
use crate::shared_state::RefreshRequestReason;
use chrono::{Duration, Utc};
use std::fs;

#[test]
fn opencode_go_login_prefills_api_key_from_opencode_auth_file() {
    let (mut env, root) = isolated_xdg("opencode-go-opencode-prefill");
    fs::create_dir_all(&root).unwrap();
    let auth_path = root.join("auth.json");
    fs::write(
        &auth_path,
        r#"{"opencode-go":{"type":"api","key":"fake-opencode-go-key"}}"#,
    )
    .unwrap();
    env.set("YAPCAP_OPENCODE_AUTH_PATH", &auth_path);
    let mut app = test_app();

    let _ = start_login::<OpenCodeGoLoginFlow>(&mut app);

    let login = app.opencode_go_login.as_ref().unwrap();
    assert_eq!(login.api_key, "fake-opencode-go-key");
    assert!(login.api_key_from_opencode);
}

#[test]
fn opencode_go_reauth_reports_an_opencode_go_specific_missing_account_error() {
    let (_env, _root) = isolated_xdg("opencode-go-missing-account");

    assert_eq!(
        prepare_for_reauth(Config::default(), "missing").unwrap_err(),
        "OpenCode Go account not found"
    );
}

#[test]
fn opencode_go_saved_login_persists_selection_and_reconciles_runtime() {
    let (_env, _root) = isolated_xdg("opencode-go-exclusive");
    let mut app = test_app();
    app.opencode_go_login = Some(OpenCodeGoLoginState::new("go-new".to_string()));
    let _ = OpenCodeGoLoginFlow::on_event(
        &mut app,
        OpenCodeGoLoginEvent::ApiKeyChanged("new-key".to_string()),
    );
    let task = OpenCodeGoLoginFlow::on_event(&mut app, OpenCodeGoLoginEvent::Saved);
    assert_eq!(task.units(), 0);
    assert_eq!(app.config.selected_opencode_go_account_ids, ["go-new"]);
    assert!(app.opencode_go_login.is_none());
    assert!(
        app.state
            .accounts_for(ProviderId::OpenCodeGo)
            .iter()
            .any(|account| { account.account_id == "go-new" })
    );
    assert_eq!(app.shared_control.requests.len(), 1);
    assert_eq!(
        app.shared_control.requests[0].reason,
        RefreshRequestReason::AccountAction
    );
    assert!(
        !app.state
            .provider(ProviderId::OpenCodeGo)
            .unwrap()
            .is_refreshing
    );
}

#[test]
fn opencode_go_saved_login_selects_new_account() {
    let (_env, _root) = isolated_xdg("opencode-go-exclusive-replacement");
    let mut app = test_app();
    app.config.opencode_go_managed_accounts = vec![opencode_go_account("go-existing", "Existing")];
    app.config.selected_opencode_go_account_ids = vec!["go-existing".to_string()];
    app.opencode_go_login = Some(OpenCodeGoLoginState::new("go-new".to_string()));
    let _ = OpenCodeGoLoginFlow::on_event(
        &mut app,
        OpenCodeGoLoginEvent::ApiKeyChanged("new-key".to_string()),
    );
    let _ = OpenCodeGoLoginFlow::on_event(&mut app, OpenCodeGoLoginEvent::Saved);
    assert_eq!(app.config.selected_opencode_go_account_ids, ["go-new"]);
    assert!(app.opencode_go_login.is_none());
    assert!(
        app.config
            .opencode_go_managed_accounts
            .iter()
            .any(|account| { account.id == "go-new" })
    );
}

#[test]
fn opencode_go_saved_login_replaces_multiple_existing_selections() {
    let (_env, _root) = isolated_xdg("opencode-go-exclusive-multiple");
    let mut app = test_app();
    app.config.opencode_go_managed_accounts = (1..=4)
        .map(|index| opencode_go_account(&format!("go-{index}"), &format!("Account {index}")))
        .collect();
    app.config.selected_opencode_go_account_ids =
        (1..=4).map(|index| format!("go-{index}")).collect();
    app.opencode_go_login = Some(OpenCodeGoLoginState::new("go-new".to_string()));
    let _ = OpenCodeGoLoginFlow::on_event(
        &mut app,
        OpenCodeGoLoginEvent::ApiKeyChanged("new-key".to_string()),
    );
    let _ = OpenCodeGoLoginFlow::on_event(&mut app, OpenCodeGoLoginEvent::Saved);
    assert_eq!(app.config.selected_opencode_go_account_ids, ["go-new"]);
    assert!(app.opencode_go_login.is_none());
    assert!(
        app.config
            .opencode_go_managed_accounts
            .iter()
            .any(|account| { account.id == "go-new" })
    );
}

#[test]
fn opencode_go_reauth_preserves_target_identity_and_replaces_key_in_place() {
    let (_env, _root) = isolated_xdg("opencode-go-reauth");
    let mut app = test_app();
    let created_at = Utc::now() - Duration::days(2);
    let updated_at = Utc::now() - Duration::days(1);
    app.config = Config {
        opencode_go_managed_accounts: vec![ManagedOpenCodeGoAccountConfig {
            id: "go-existing".to_string(),
            label: "Existing".to_string(),
            api_key_source: "stored".to_string(),
            created_at,
            updated_at,
            last_authenticated_at: Some(updated_at),
        }],
        ..Config::default()
    };
    write_api_key("go-existing", "old-key").unwrap();
    let _ = reauthenticate::<OpenCodeGoLoginFlow>(&mut app, "go-existing");
    let login = app.opencode_go_login.as_ref().unwrap();
    assert_eq!(login.account_id, "go-existing");
    assert_eq!(login.label, "Existing");
    let _ = OpenCodeGoLoginFlow::on_event(
        &mut app,
        OpenCodeGoLoginEvent::LabelChanged("Edited label".to_string()),
    );
    let _ = OpenCodeGoLoginFlow::on_event(
        &mut app,
        OpenCodeGoLoginEvent::ApiKeyChanged("new-key".to_string()),
    );
    let _ = OpenCodeGoLoginFlow::on_event(&mut app, OpenCodeGoLoginEvent::Saved);
    assert_eq!(app.config.opencode_go_managed_accounts.len(), 1);
    let account = &app.config.opencode_go_managed_accounts[0];
    assert_eq!(account.id, "go-existing");
    assert_eq!(account.label, "Existing");
    assert_eq!(account.created_at, created_at);
    assert_eq!(account.api_key_source, "stored");
    assert!(account.updated_at > updated_at);
    assert!(
        account
            .last_authenticated_at
            .is_some_and(|time| time > updated_at)
    );
    assert_eq!(load_api_key("go-existing").unwrap(), "new-key");
    assert!(app.opencode_go_login.is_none());
}

fn opencode_go_account(id: &str, label: &str) -> ManagedOpenCodeGoAccountConfig {
    let now = Utc::now();
    ManagedOpenCodeGoAccountConfig {
        id: id.to_string(),
        label: label.to_string(),
        api_key_source: "stored".to_string(),
        created_at: now,
        updated_at: now,
        last_authenticated_at: Some(now),
    }
}
