use super::support::{isolated_xdg, test_app};
use crate::app::login::{KimiLoginFlow, LoginFlow, reauthenticate};
use crate::config::{Config, ManagedKimiAccountConfig, managed_kimi_account_dir};
use crate::model::ProviderId;
use crate::providers::kimi::login::{KimiLoginEvent, KimiLoginState, KimiLoginStatus};
use crate::providers::kimi::storage::{load_api_key, write_api_key};
use crate::shared_state::RefreshRequestReason;
use chrono::{Duration, Utc};

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
fn kimi_saved_login_adds_new_account_to_show_all_selection_under_cap() {
    let (_env, _root) = isolated_xdg("kimi-show-all-under-cap");
    let mut app = test_app();
    app.config.kimi_managed_accounts = vec![kimi_account("kimi-existing", "Existing")];
    app.config.selected_kimi_account_ids = vec!["kimi-existing".to_string()];
    app.config.set_provider_show_all(ProviderId::Kimi, true);
    app.kimi_login = Some(KimiLoginState::new("kimi-new".to_string()));
    let _ = KimiLoginFlow::on_event(
        &mut app,
        KimiLoginEvent::ApiKeyChanged("new-key".to_string()),
    );

    let _ = KimiLoginFlow::on_event(&mut app, KimiLoginEvent::Saved);

    assert_eq!(
        app.config.selected_kimi_account_ids,
        ["kimi-existing", "kimi-new"]
    );
    assert!(
        app.config
            .kimi_managed_accounts
            .iter()
            .any(|account| account.id == "kimi-new")
    );
}

#[test]
fn kimi_saved_login_keeps_new_account_unselected_at_show_all_cap() {
    let (_env, _root) = isolated_xdg("kimi-show-all-at-cap");
    let mut app = test_app();
    app.config.kimi_managed_accounts = (1..=4)
        .map(|index| kimi_account(&format!("kimi-{index}"), &format!("Account {index}")))
        .collect();
    app.config.selected_kimi_account_ids = (1..=4).map(|index| format!("kimi-{index}")).collect();
    app.config.set_provider_show_all(ProviderId::Kimi, true);
    app.kimi_login = Some(KimiLoginState::new("kimi-new".to_string()));
    let _ = KimiLoginFlow::on_event(
        &mut app,
        KimiLoginEvent::ApiKeyChanged("new-key".to_string()),
    );

    let _ = KimiLoginFlow::on_event(&mut app, KimiLoginEvent::Saved);

    assert_eq!(app.config.selected_kimi_account_ids.len(), 4);
    assert!(
        !app.config
            .selected_kimi_account_ids
            .contains(&"kimi-new".to_string())
    );
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
    write_api_key(&managed_kimi_account_dir("kimi-existing"), "old-key").unwrap();

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
    assert_eq!(
        load_api_key(&managed_kimi_account_dir("kimi-existing")).unwrap(),
        "new-key"
    );
    assert_eq!(app.config.kimi_managed_accounts.len(), 1);
    assert_eq!(
        app.kimi_login.as_ref().unwrap().status,
        KimiLoginStatus::Saved
    );
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
