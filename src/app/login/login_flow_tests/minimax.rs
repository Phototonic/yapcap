use super::support::{isolated_xdg, test_app};
use crate::app::login::{MinimaxLoginFlow, start_login};
use crate::config::Config;
use crate::providers::minimax::login::prepare_for_reauth;
use std::fs;

#[test]
fn minimax_login_uses_the_minimax_opencode_credential_for_prefill() {
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
fn minimax_reauth_reports_a_minimax_specific_missing_account_error() {
    let (_env, _root) = isolated_xdg("minimax-missing-account");

    assert_eq!(
        prepare_for_reauth(Config::default(), "missing").unwrap_err(),
        "Minimax account not found"
    );
}
