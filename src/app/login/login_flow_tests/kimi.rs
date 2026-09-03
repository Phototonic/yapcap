use super::support::{isolated_xdg, test_app};
use crate::app::login::{KimiLoginFlow, LoginFlow, reauthenticate, start_login};
use crate::config::{Config, ManagedKimiAccountConfig};
use crate::model::ProviderId;
use crate::providers::kimi::login::{KimiLoginEvent, KimiLoginState};
use crate::providers::kimi::storage::{load_api_key, write_api_key};
use crate::shared_state::RefreshRequestReason;
use chrono::{Duration, Utc};
use std::fs;

#[test]
fn kimi_login_prefills_api_key_from_opencode_auth_file() {
    let (mut env, root) = isolated_xdg("kimi-opencode-prefill");
    fs::create_dir_all(&root).unwrap();
    let auth_path = root.join("auth.json");
    fs::write(
        &auth_path,
        r#"{"kimi-for-coding":{"type":"api","key":"fake-kimi-key"}}"#,
    )
    .unwrap();
    env.set("YAPCAP_OPENCODE_AUTH_PATH", &auth_path);
    let mut app = test_app();

    let _ = start_login::<KimiLoginFlow>(&mut app);

    let login = app.kimi_login.as_ref().unwrap();
    assert_eq!(login.api_key, "fake-kimi-key");
    assert!(login.api_key_from_opencode);
}

#[test]
fn kimi_saved_login_persists_selection_and_reconciles_runtime() {
    let (_env, _root) = isolated_xdg("kimi-exclusive");
    let mut app = test_app();
    app.kimi_login = Some(KimiLoginState::new("kimi-new".to_string()));
    let _ = KimiLoginFlow::on_event(
        &mut app,
        KimiLoginEvent::ApiKeyChanged("new-key".to_string()),
    );

    let task = KimiLoginFlow::on_event(&mut app, KimiLoginEvent::Saved);

    assert_eq!(task.units(), 0);
    assert_eq!(app.config.selected_kimi_account_ids, ["kimi-new"]);
    assert!(app.kimi_login.is_none());
    assert!(
        app.state
            .accounts_for(ProviderId::Kimi)
            .iter()
            .any(|account| { account.account_id == "kimi-new" })
    );
    assert_eq!(app.shared_control.requests.len(), 1);
    assert_eq!(
        app.shared_control.requests[0].reason,
        RefreshRequestReason::AccountAction
    );
    assert!(!app.state.provider(ProviderId::Kimi).unwrap().is_refreshing);
}

#[test]
fn kimi_saved_login_selects_new_account() {
    let (_env, _root) = isolated_xdg("kimi-exclusive-replacement");
    let mut app = test_app();
    app.config.kimi_managed_accounts = vec![kimi_account("kimi-existing", "Existing")];
    app.config.selected_kimi_account_ids = vec!["kimi-existing".to_string()];
    app.kimi_login = Some(KimiLoginState::new("kimi-new".to_string()));
    let _ = KimiLoginFlow::on_event(
        &mut app,
        KimiLoginEvent::ApiKeyChanged("new-key".to_string()),
    );

    let _ = KimiLoginFlow::on_event(&mut app, KimiLoginEvent::Saved);

    assert_eq!(app.config.selected_kimi_account_ids, ["kimi-new"]);
    assert!(app.kimi_login.is_none());
    assert!(
        app.config
            .kimi_managed_accounts
            .iter()
            .any(|account| account.id == "kimi-new")
    );
}

#[test]
fn kimi_saved_login_replaces_multiple_existing_selections() {
    let (_env, _root) = isolated_xdg("kimi-exclusive-multiple");
    let mut app = test_app();
    app.config.kimi_managed_accounts = (1..=4)
        .map(|index| kimi_account(&format!("kimi-{index}"), &format!("Account {index}")))
        .collect();
    app.config.selected_kimi_account_ids = (1..=4).map(|index| format!("kimi-{index}")).collect();
    app.kimi_login = Some(KimiLoginState::new("kimi-new".to_string()));
    let _ = KimiLoginFlow::on_event(
        &mut app,
        KimiLoginEvent::ApiKeyChanged("new-key".to_string()),
    );

    let _ = KimiLoginFlow::on_event(&mut app, KimiLoginEvent::Saved);

    assert_eq!(app.config.selected_kimi_account_ids, ["kimi-new"]);
    assert!(
        app.config
            .kimi_managed_accounts
            .iter()
            .any(|account| account.id == "kimi-new")
    );
}

#[test]
fn kimi_reauth_preserves_target_identity_and_replaces_key_in_place() {
    let (_env, _root) = isolated_xdg("kimi-reauth");
    let mut app = test_app();
    let created_at = Utc::now() - Duration::days(2);
    let updated_at = Utc::now() - Duration::days(1);
    app.config = Config {
        kimi_managed_accounts: vec![ManagedKimiAccountConfig {
            id: "kimi-existing".to_string(),
            label: "Existing".to_string(),
            api_key_source: "stored".to_string(),
            created_at,
            updated_at,
            last_authenticated_at: Some(updated_at),
        }],
        ..Config::default()
    };
    write_api_key("kimi-existing", "old-key").unwrap();

    let _ = reauthenticate::<KimiLoginFlow>(&mut app, "kimi-existing");
    let login = app.kimi_login.as_ref().unwrap();
    assert_eq!(login.account_id, "kimi-existing");
    assert_eq!(login.label, "Existing");
    let _ = KimiLoginFlow::on_event(
        &mut app,
        KimiLoginEvent::LabelChanged("Edited label".to_string()),
    );
    let _ = KimiLoginFlow::on_event(
        &mut app,
        KimiLoginEvent::ApiKeyChanged("new-key".to_string()),
    );
    let _ = KimiLoginFlow::on_event(&mut app, KimiLoginEvent::Saved);

    assert_eq!(app.config.kimi_managed_accounts.len(), 1);
    let account = &app.config.kimi_managed_accounts[0];
    assert_eq!(account.id, "kimi-existing");
    assert_eq!(account.label, "Existing");
    assert_eq!(account.created_at, created_at);
    assert_eq!(account.api_key_source, "stored");
    assert!(account.updated_at > updated_at);
    assert!(
        account
            .last_authenticated_at
            .is_some_and(|time| time > updated_at)
    );
    assert_eq!(load_api_key("kimi-existing").unwrap(), "new-key");
    assert_eq!(app.config.kimi_managed_accounts.len(), 1);
    assert!(app.kimi_login.is_none());
}

fn kimi_account(id: &str, label: &str) -> ManagedKimiAccountConfig {
    let now = Utc::now();
    ManagedKimiAccountConfig {
        id: id.to_string(),
        label: label.to_string(),
        api_key_source: "stored".to_string(),
        created_at: now,
        updated_at: now,
        last_authenticated_at: Some(now),
    }
}
