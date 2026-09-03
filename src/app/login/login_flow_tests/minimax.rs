use super::support::{isolated_xdg, test_app};
use crate::app::login::{LoginFlow, MinimaxLoginFlow, start_login};
use crate::model::ProviderId;
use crate::providers::minimax::{MinimaxLoginEvent, MinimaxLoginState, MinimaxLoginStatus};
use std::fs;

#[test]
fn minimax_on_event_api_key_and_label_changes_update_state() {
    let mut app = test_app();
    app.minimax_login = Some(MinimaxLoginState::new("draft".to_string()));

    let _ = MinimaxLoginFlow::on_event(
        &mut app,
        MinimaxLoginEvent::ApiKeyChanged("sk-test".to_string()),
    );
    let _ = MinimaxLoginFlow::on_event(
        &mut app,
        MinimaxLoginEvent::LabelChanged("Work".to_string()),
    );

    let login = app.minimax_login.as_ref().unwrap();
    assert_eq!(login.api_key, "sk-test");
    assert_eq!(login.label, "Work");
}

#[test]
fn minimax_login_prefills_api_key_from_opencode_auth_file() {
    let (mut env, root) = isolated_xdg("minimax-opencode-prefill");
    fs::create_dir_all(&root).unwrap();
    let auth_path = root.join("auth.json");
    fs::write(
        &auth_path,
        r#"{"minimax":{"type":"api","key":"fake-minimax-key"}}"#,
    )
    .unwrap();
    env.set("YAPCAP_OPENCODE_AUTH_PATH", &auth_path);
    let mut app = test_app();

    let _ = start_login::<MinimaxLoginFlow>(&mut app);

    let login = app.minimax_login.as_ref().unwrap();
    assert_eq!(login.api_key, "fake-minimax-key");
    assert!(login.api_key_from_opencode);
}

#[test]
fn minimax_on_event_saved_persists_account_and_seeds_runtime_state() {
    let (_env, _root) = isolated_xdg("minimax-saved");
    let mut app = test_app();
    app.minimax_login = Some(MinimaxLoginState::new("minimax-1".to_string()));
    let _ = MinimaxLoginFlow::on_event(
        &mut app,
        MinimaxLoginEvent::ApiKeyChanged("sk-test".to_string()),
    );

    let _ = MinimaxLoginFlow::on_event(&mut app, MinimaxLoginEvent::Saved);

    assert!(app.minimax_login.is_none());
    assert!(
        app.config
            .minimax_managed_accounts
            .iter()
            .any(|account| account.id == "minimax-1")
    );
    assert!(
        app.state
            .accounts_for(ProviderId::Minimax)
            .into_iter()
            .any(|account| account.account_id == "minimax-1")
    );
}

#[test]
fn minimax_on_event_saved_replaces_existing_selection() {
    let (_env, _root) = isolated_xdg("minimax-exclusive");
    let mut app = test_app();
    app.config.selected_minimax_account_ids = vec!["minimax-existing".to_string()];
    app.minimax_login = Some(MinimaxLoginState::new("minimax-new".to_string()));
    let _ = MinimaxLoginFlow::on_event(
        &mut app,
        MinimaxLoginEvent::ApiKeyChanged("sk-test".to_string()),
    );

    let _ = MinimaxLoginFlow::on_event(&mut app, MinimaxLoginEvent::Saved);

    assert_eq!(app.config.selected_minimax_account_ids, ["minimax-new"]);
    assert_eq!(
        app.state
            .provider(ProviderId::Minimax)
            .unwrap()
            .selected_account_ids,
        ["minimax-new"]
    );
}

#[test]
fn minimax_on_event_saved_without_api_key_fails() {
    let mut app = test_app();
    app.minimax_login = Some(MinimaxLoginState::new("minimax-2".to_string()));

    let _ = MinimaxLoginFlow::on_event(&mut app, MinimaxLoginEvent::Saved);

    let login = app.minimax_login.as_ref().unwrap();
    assert_eq!(login.status, MinimaxLoginStatus::Failed);
    assert!(login.error.is_some());
}
