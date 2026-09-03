use super::support::{isolated_xdg, test_app};
use crate::app::login::{KimiLoginFlow, reauthenticate, start_login};
use crate::config::Config;
use crate::providers::kimi::login::prepare_for_reauth;
use std::fs;

#[test]
fn kimi_login_uses_the_kimi_opencode_credential_for_prefill() {
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
fn kimi_reauth_reports_a_kimi_specific_missing_account_error() {
    let (_env, _root) = isolated_xdg("kimi-missing-account");

    assert_eq!(
        prepare_for_reauth(Config::default(), "missing").unwrap_err(),
        "Kimi account not found"
    );
}

#[test]
fn kimi_reauth_is_not_started_for_an_unknown_account() {
    let (_env, _root) = isolated_xdg("kimi-unknown-reauth");
    let mut app = test_app();

    let _ = reauthenticate::<KimiLoginFlow>(&mut app, "missing");

    assert!(app.kimi_login.is_none());
}
