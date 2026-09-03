use super::*;
use crate::config::Config;

#[test]
fn providers_expose_expected_capabilities() {
    assert_eq!(
        capabilities(ProviderId::Codex),
        ProviderCapabilities {
            supports_delete: true,
            supports_reauthentication: false,
            supports_background_status_refresh: false,
            requires_auth_prompt_on_auth_failure: false,
        }
    );
    assert_eq!(
        capabilities(ProviderId::Claude),
        ProviderCapabilities {
            supports_delete: true,
            supports_reauthentication: true,
            supports_background_status_refresh: false,
            requires_auth_prompt_on_auth_failure: false,
        }
    );
    assert_eq!(
        capabilities(ProviderId::Cursor),
        ProviderCapabilities {
            supports_delete: true,
            supports_reauthentication: true,
            supports_background_status_refresh: true,
            requires_auth_prompt_on_auth_failure: true,
        }
    );
    assert_eq!(
        capabilities(ProviderId::Kimi),
        ProviderCapabilities {
            supports_delete: true,
            supports_reauthentication: true,
            supports_background_status_refresh: false,
            requires_auth_prompt_on_auth_failure: false,
        }
    );
    assert_eq!(
        capabilities(ProviderId::OpenCodeGo),
        ProviderCapabilities {
            supports_delete: true,
            supports_reauthentication: true,
            supports_background_status_refresh: false,
            requires_auth_prompt_on_auth_failure: false,
        }
    );
}

#[test]
fn cursor_supports_background_status_refresh() {
    assert!(supports_background_status_refresh(ProviderId::Cursor));
    assert!(!supports_background_status_refresh(ProviderId::Codex));
    assert!(!supports_background_status_refresh(ProviderId::Claude));
}

#[test]
fn cursor_requires_reauth_prompt_on_auth_error() {
    assert!(auth_error_requires_reauth_prompt(ProviderId::Cursor));
    assert!(!auth_error_requires_reauth_prompt(ProviderId::Codex));
    assert!(!auth_error_requires_reauth_prompt(ProviderId::Claude));
}

#[test]
fn each_provider_resolves_accounts() {
    let config = Config::default();
    for provider in ProviderId::ALL {
        let accounts = discover_accounts(provider, &config);
        assert!(
            accounts.is_empty(),
            "default config should have no accounts for {provider:?}"
        );
    }
}

#[test]
fn action_support_matches_capabilities() {
    let support = capabilities(ProviderId::Cursor).action_support();
    assert!(support.can_delete);
    assert!(support.can_reauthenticate);
    assert!(support.supports_background_status_refresh);
}

#[test]
fn system_active_account_id_only_supported_by_codex_claude_gemini_minimax_kimi_opencode_go() {
    use crate::account_storage::{
        NewProviderAccount, ProviderAccountStorage, ProviderAccountTokens,
    };
    use crate::config::{
        ManagedClaudeAccountConfig, ManagedCodexAccountConfig, ManagedGeminiAccountConfig,
        ManagedKimiAccountConfig, ManagedMinimaxAccountConfig, ManagedOpenCodeGoAccountConfig,
        paths,
    };
    use chrono::Utc;
    use std::fs;
    use std::path::PathBuf;

    let mut env = crate::test_support::test_env();
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("home");
    let state = temp.path().join("state");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&state).unwrap();
    env.set("HOME", &home);
    env.set("XDG_STATE_HOME", &state);
    env.set("MINIMAX_API_KEY", "test-minimax-key");
    env.set("KIMI_API_KEY", "test-kimi-key");
    env.set("OPENCODE_GO_API_KEY", "test-opencode-go-key");
    env.remove("FLATPAK_ID");

    let codex_dir = home.join(".codex");
    fs::create_dir_all(&codex_dir).unwrap();
    let id_token = "eyJhbGciOiJSUzI1NiJ9.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOiB7ImNoYXRncHRfYWNjb3VudF9pZCI6ICJhY2N0LWFiYy0xMjMifX0.fakesig";
    fs::write(
        codex_dir.join("auth.json"),
        format!(r#"{{"tokens":{{"id_token":"{id_token}"}}}}"#),
    )
    .unwrap();

    let gemini_dir = home.join(".gemini");
    fs::create_dir_all(&gemini_dir).unwrap();
    fs::write(
        gemini_dir.join("google_accounts.json"),
        r#"{"active":"alice@example.com"}"#,
    )
    .unwrap();

    let storage = ProviderAccountStorage::new(paths().claude_accounts_dir.clone());
    let stored = storage
        .create_account(NewProviderAccount {
            provider: ProviderId::Claude,
            email: "claude@example.com".to_string(),
            provider_account_id: Some("acct-uuid".to_string()),
            organization_id: None,
            organization_name: None,
            tokens: ProviderAccountTokens {
                access_token: "a".to_string(),
                refresh_token: "r".to_string(),
                expires_at: Utc::now(),
                scope: vec![],
                token_id: None,
            },
            snapshot: None,
        })
        .unwrap();
    fs::write(
        home.join(".claude.json"),
        r#"{"oauthAccount":{"accountUuid":"acct-uuid"}}"#,
    )
    .unwrap();

    let config = Config {
        codex_managed_accounts: vec![ManagedCodexAccountConfig {
            id: "codex-1".to_string(),
            label: "Codex".to_string(),
            codex_home: PathBuf::from("/tmp"),
            email: None,
            provider_account_id: Some("acct-abc-123".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_authenticated_at: None,
        }],
        claude_managed_accounts: vec![ManagedClaudeAccountConfig {
            id: stored.metadata.account_id.clone(),
            label: "Claude".to_string(),
            config_dir: paths()
                .claude_accounts_dir
                .join(&stored.metadata.account_id),
            email: Some("claude@example.com".to_string()),
            organization: None,
            subscription_type: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_authenticated_at: None,
        }],
        gemini_managed_accounts: vec![ManagedGeminiAccountConfig {
            id: "gemini-1".to_string(),
            label: "Gemini".to_string(),
            account_root: PathBuf::from("/tmp/gemini-1"),
            email: "alice@example.com".to_string(),
            sub: "sub".to_string(),
            hd: None,
            last_tier_id: None,
            last_cloudaicompanion_project: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_authenticated_at: None,
        }],
        minimax_managed_accounts: vec![ManagedMinimaxAccountConfig {
            id: "minimax-1".to_string(),
            label: "Minimax".to_string(),
            api_key_source: "env:MINIMAX_API_KEY".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_authenticated_at: None,
        }],
        kimi_managed_accounts: vec![ManagedKimiAccountConfig {
            id: "kimi-1".to_string(),
            label: "Kimi".to_string(),
            api_key_source: "env:KIMI_API_KEY".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_authenticated_at: None,
        }],
        opencode_go_managed_accounts: vec![ManagedOpenCodeGoAccountConfig {
            id: "opencode-go-1".to_string(),
            label: "OpenCode Go".to_string(),
            api_key_source: "env:OPENCODE_GO_API_KEY".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_authenticated_at: None,
        }],
        ..Config::default()
    };

    let expectations = [
        (ProviderId::Codex, true),
        (ProviderId::Claude, true),
        (ProviderId::Gemini, true),
        (ProviderId::Cursor, false),
        (ProviderId::Copilot, false),
        (ProviderId::Minimax, true),
        (ProviderId::Kimi, true),
        (ProviderId::OpenCodeGo, true),
    ];
    for (provider, expect_some) in expectations {
        let result = system_active_account_id(provider, &config);
        assert_eq!(
            result.is_some(),
            expect_some,
            "provider {provider:?} expected has_active={expect_some}, got {result:?}"
        );
    }
}

#[test]
fn opencode_go_system_active_account_id_matches_opencode_auth_file() {
    use crate::config::{ManagedOpenCodeGoAccountConfig, paths};
    use crate::providers::opencode_auth::OPENCODE_AUTH_PATH_ENV;
    use crate::providers::opencode_go::storage::write_api_key_at;
    use chrono::Utc;
    use std::fs;

    let temp = tempfile::tempdir().unwrap();
    let state = temp.path().join("state");
    let auth_path = temp.path().join("auth.json");
    let mut env = crate::test_support::test_env();
    env.set("XDG_STATE_HOME", &state);
    env.set(OPENCODE_AUTH_PATH_ENV, &auth_path);
    env.remove("OPENCODE_API_KEY");
    env.remove("OPENCODE_GO_API_KEY");

    fs::write(
        &auth_path,
        r#"{"opencode-go":{"type":"api","key":"test-key"}}"#,
    )
    .unwrap();
    write_api_key_at(&paths().opencode_go_accounts_dir, "go-1", "test-key").unwrap();
    let config = Config {
        opencode_go_managed_accounts: vec![ManagedOpenCodeGoAccountConfig {
            id: "go-1".to_string(),
            label: "OpenCode Go".to_string(),
            api_key_source: "stored".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_authenticated_at: None,
        }],
        ..Config::default()
    };

    assert_eq!(
        system_active_account_id(ProviderId::OpenCodeGo, &config),
        Some("go-1".to_string())
    );
}
