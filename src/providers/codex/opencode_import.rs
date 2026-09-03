// SPDX-License-Identifier: MPL-2.0

use super::login::{CodexLoginEvent, CodexLoginState, CodexLoginStatus, commit_login};
use crate::auth::CodexAuth;
use crate::config::Config;
use crate::providers::codex::{fetch_oauth, fetch_oauth_at};
use crate::providers::opencode_auth::{OpenCodeCredential, discover};
use chrono::{DateTime, Utc};
use cosmic::iced::Task;

const PROVIDER_ID: &str = "openai";

pub fn is_available() -> bool {
    discover(PROVIDER_ID).is_some_and(|credential| convert(credential).is_ok())
}

pub fn prepare_opencode_import(
    config: Config,
    target_account_id: Option<String>,
) -> Result<(CodexLoginState, Task<CodexLoginEvent>), String> {
    if let Some(target_account_id) = target_account_id.as_deref()
        && !config
            .codex_managed_accounts
            .iter()
            .any(|account| account.id == target_account_id)
    {
        return Err("Codex account no longer exists".to_string());
    }
    let flow_id = super::account::new_account_id();
    let state = CodexLoginState {
        flow_id: flow_id.clone(),
        status: CodexLoginStatus::Running,
        login_url: None,
        error: None,
        importing_from_opencode: true,
    };
    let task = Task::perform(
        run_import(flow_id.clone(), config, target_account_id),
        move |result| CodexLoginEvent::Finished {
            flow_id,
            result: Box::new(result),
        },
    );
    Ok((state, task))
}

async fn run_import(
    flow_id: String,
    config: Config,
    target_account_id: Option<String>,
) -> Result<super::login::CodexLoginSuccess, String> {
    run_import_with(
        flow_id,
        config,
        target_account_id,
        &crate::runtime::http_client(),
        None,
    )
    .await
}

async fn run_import_with(
    flow_id: String,
    config: Config,
    target_account_id: Option<String>,
    client: &reqwest::Client,
    endpoint: Option<&str>,
) -> Result<super::login::CodexLoginSuccess, String> {
    let auth = discover(PROVIDER_ID)
        .ok_or_else(|| "No compatible Codex credential was found in OpenCode.".to_string())
        .and_then(convert)?;
    let snapshot = match endpoint {
        Some(endpoint) => fetch_oauth_at(client, &auth, endpoint).await,
        None => fetch_oauth(client, &auth).await,
    }
    .map_err(|_| "OpenCode Codex credentials could not be validated.".to_string())?;
    commit_login(
        &flow_id,
        &config,
        auth,
        Some(snapshot),
        target_account_id.as_deref(),
    )
    .map_err(|_| "OpenCode Codex credentials could not be imported.".to_string())
}

fn convert(credential: OpenCodeCredential) -> Result<CodexAuth, String> {
    let OpenCodeCredential::OAuth {
        access,
        refresh,
        expires,
        account_id,
        ..
    } = credential
    else {
        return Err("OpenCode does not contain a compatible Codex OAuth credential.".to_string());
    };
    let expires_at = DateTime::<Utc>::from_timestamp_millis(expires)
        .filter(|expires_at| *expires_at > Utc::now())
        .ok_or_else(|| "OpenCode Codex credentials have expired or are invalid.".to_string())?;
    if access.is_empty() || refresh.is_empty() {
        return Err("OpenCode does not contain a compatible Codex OAuth credential.".to_string());
    }
    Ok(CodexAuth {
        access_token: access,
        refresh_token: Some(refresh),
        expires_at: Some(expires_at),
        account_id,
        id_token: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_storage::{
        NewProviderAccount, ProviderAccountStorage, ProviderAccountTokens,
    };
    use crate::config::{ManagedCodexAccountConfig, paths};
    use crate::model::ProviderId;
    use crate::providers::opencode_auth::OPENCODE_AUTH_PATH_ENV;
    use std::fs;
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn availability_from(contents: &str) -> bool {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(&path, contents).unwrap();
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &path);
        is_available()
    }

    async fn server(status: u16, body: &'static str) -> (String, tokio::task::JoinHandle<String>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let address = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buffer = vec![0; 8192];
            let count = stream.read(&mut buffer).await.unwrap();
            let request = String::from_utf8_lossy(&buffer[..count]).to_string();
            let response = format!(
                "HTTP/1.1 {status} OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            request
        });
        (format!("http://{address}/usage"), handle)
    }

    fn seed_target() -> (Config, std::path::PathBuf) {
        let now = Utc::now();
        let storage = ProviderAccountStorage::new(paths().codex_accounts_dir);
        let stored = storage
            .replace_account(
                "codex-target".to_string(),
                NewProviderAccount {
                    provider: ProviderId::Codex,
                    email: "user@example.com".to_string(),
                    provider_account_id: Some("expected-account".to_string()),
                    organization_id: None,
                    organization_name: None,
                    tokens: ProviderAccountTokens {
                        access_token: "original-access".to_string(),
                        refresh_token: "original-refresh".to_string(),
                        expires_at: now + chrono::Duration::hours(1),
                        scope: Vec::new(),
                        token_id: None,
                    },
                    snapshot: None,
                },
            )
            .unwrap();
        (
            Config {
                codex_managed_accounts: vec![ManagedCodexAccountConfig {
                    id: "codex-target".to_string(),
                    label: "user@example.com".to_string(),
                    codex_home: stored.account_dir.clone(),
                    email: Some("user@example.com".to_string()),
                    provider_account_id: Some("expected-account".to_string()),
                    created_at: now,
                    updated_at: now,
                    last_authenticated_at: Some(now),
                }],
                selected_codex_account_ids: vec!["codex-target".to_string()],
                ..Config::default()
            },
            stored.account_dir.join("tokens.json"),
        )
    }

    fn write_auth_fixture(path: &std::path::Path) {
        let expires = Utc::now().timestamp_millis() + 60_000;
        fs::write(
            path,
            format!(r#"{{"openai":{{"type":"oauth","access":"new-access","refresh":"new-refresh","expires":{expires}}}}}"#),
        )
        .unwrap();
    }

    #[test]
    fn converts_future_oauth_credentials() {
        let expires = Utc::now().timestamp_millis() + 60_000;
        let auth = convert(OpenCodeCredential::OAuth {
            access: "access".to_string(),
            refresh: "refresh".to_string(),
            expires,
            account_id: Some("account".to_string()),
            enterprise_url: None,
        })
        .unwrap();
        assert_eq!(auth.access_token, "access");
        assert_eq!(auth.refresh_token.as_deref(), Some("refresh"));
        assert_eq!(auth.account_id.as_deref(), Some("account"));
        assert_eq!(auth.expires_at.unwrap().timestamp_millis(), expires);
    }

    #[test]
    fn rejects_non_oauth_or_invalid_expiry_credentials() {
        assert!(
            convert(OpenCodeCredential::Api {
                key: "key".to_string()
            })
            .is_err()
        );
        assert!(
            convert(OpenCodeCredential::WellKnown {
                key: "key".to_string(),
                token: "token".to_string(),
            })
            .is_err()
        );
        for expires in [0, i64::MAX] {
            assert!(
                convert(OpenCodeCredential::OAuth {
                    access: "access".to_string(),
                    refresh: "refresh".to_string(),
                    expires,
                    account_id: None,
                    enterprise_url: None,
                })
                .is_err()
            );
        }
    }

    #[test]
    fn availability_requires_compatible_future_oauth_credentials() {
        let expires = Utc::now().timestamp_millis() + 60_000;
        assert!(availability_from(&format!(
            r#"{{"openai":{{"type":"oauth","access":"access","refresh":"refresh","expires":{expires}}}}}"#
        )));
        assert!(!availability_from(
            r#"{"openai":{"type":"api","key":"key"}}"#,
        ));
        assert!(!availability_from(
            r#"{"openai":{"type":"wellknown","key":"key","token":"token"}}"#,
        ));
    }

    #[tokio::test]
    async fn imports_validated_credentials_and_rereads_the_auth_file_at_click_time() {
        let temp = tempdir().unwrap();
        let auth_path = temp.path().join("auth.json");
        let state_root = temp.path().join("state");
        let expires = Utc::now().timestamp_millis() + 60_000;
        fs::write(
            &auth_path,
            format!(r#"{{"openai":{{"type":"oauth","access":"first","refresh":"refresh-first","expires":{expires}}}}}"#),
        )
        .unwrap();
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &auth_path);
        env.set("XDG_STATE_HOME", &state_root);
        let body = r#"{"account_id":"acct-1","email":"user@example.com","rate_limit":{"primary_window":{"used_percent":1.0,"reset_at":2000000000}}}"#;
        let client = reqwest::Client::new();
        let (endpoint, request) = server(200, body).await;

        let first = run_import_with(
            "codex-1".to_string(),
            Config::default(),
            None,
            &client,
            Some(&endpoint),
        )
        .await
        .unwrap();
        assert_eq!(first.account.provider_account_id.as_deref(), Some("acct-1"));
        assert!(request.await.unwrap().contains("Bearer first"));

        fs::write(
            &auth_path,
            format!(r#"{{"openai":{{"type":"oauth","access":"second","refresh":"refresh-second","expires":{expires}}}}}"#),
        )
        .unwrap();
        let (endpoint, request) = server(200, body).await;
        let config = Config {
            codex_managed_accounts: vec![first.account.clone()],
            ..Config::default()
        };
        let second = run_import_with(
            "codex-2".to_string(),
            config,
            None,
            &client,
            Some(&endpoint),
        )
        .await
        .unwrap();
        assert_eq!(second.account.id, "codex-1");
        assert!(request.await.unwrap().contains("Bearer second"));
        let tokens = crate::account_storage::ProviderAccountStorage::new(
            crate::config::paths().codex_accounts_dir,
        )
        .load_tokens("codex-1")
        .unwrap();
        assert_eq!(tokens.access_token, "second");
        assert_eq!(tokens.refresh_token, "refresh-second");
    }

    #[tokio::test]
    async fn targeted_restore_preserves_codex_account_identity_and_creation_time() {
        let temp = tempdir().unwrap();
        let auth_path = temp.path().join("auth.json");
        let state_root = temp.path().join("state");
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &auth_path);
        env.set("XDG_STATE_HOME", &state_root);
        write_auth_fixture(&auth_path);
        let (config, _) = seed_target();
        let target = config.codex_managed_accounts[0].clone();
        fs::remove_file(target.codex_home.join("metadata.json")).unwrap();
        let body = r#"{"account_id":"expected-account","email":"user@example.com","rate_limit":{"primary_window":{"used_percent":1.0,"reset_at":2000000000}}}"#;
        let (endpoint, _) = server(200, body).await;

        let restored = run_import_with(
            "codex-restore".to_string(),
            config,
            Some(target.id.clone()),
            &reqwest::Client::new(),
            Some(&endpoint),
        )
        .await
        .unwrap();

        assert_eq!(restored.account.id, target.id);
        assert_eq!(restored.account.created_at, target.created_at);
        assert!(restored.account.updated_at >= target.updated_at);
        assert!(restored.account.last_authenticated_at.is_some());
        let metadata = ProviderAccountStorage::new(paths().codex_accounts_dir)
            .load_metadata(&target.id)
            .unwrap();
        assert_eq!(metadata.created_at, target.created_at);
    }

    #[tokio::test]
    async fn validation_failure_does_not_create_codex_storage() {
        let temp = tempdir().unwrap();
        let auth_path = temp.path().join("auth.json");
        let state_root = temp.path().join("state");
        let expires = Utc::now().timestamp_millis() + 60_000;
        fs::write(
            &auth_path,
            format!(r#"{{"openai":{{"type":"oauth","access":"access","refresh":"refresh","expires":{expires}}}}}"#),
        )
        .unwrap();
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &auth_path);
        env.set("XDG_STATE_HOME", &state_root);
        let (endpoint, _) = server(401, "{}").await;

        assert!(
            run_import_with(
                "codex-failed".to_string(),
                Config::default(),
                None,
                &reqwest::Client::new(),
                Some(&endpoint),
            )
            .await
            .is_err()
        );
        assert!(!crate::config::paths().codex_accounts_dir.exists());
    }

    #[tokio::test]
    async fn targeted_import_with_different_provider_identity_preserves_storage_and_config() {
        let temp = tempdir().unwrap();
        let auth_path = temp.path().join("auth.json");
        let state_root = temp.path().join("state");
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &auth_path);
        env.set("XDG_STATE_HOME", &state_root);
        write_auth_fixture(&auth_path);
        let (config, tokens_path) = seed_target();
        let original_config = config.clone();
        let original_tokens = fs::read(&tokens_path).unwrap();
        let body = r#"{"account_id":"different-account","email":"user@example.com","rate_limit":{"primary_window":{"used_percent":1.0,"reset_at":2000000000}}}"#;
        let (endpoint, _) = server(200, body).await;

        assert!(
            run_import_with(
                "codex-import".to_string(),
                config.clone(),
                Some("codex-target".to_string()),
                &reqwest::Client::new(),
                Some(&endpoint),
            )
            .await
            .is_err()
        );
        assert_eq!(config, original_config);
        assert_eq!(fs::read(tokens_path).unwrap(), original_tokens);
    }

    #[tokio::test]
    async fn targeted_import_without_provider_identity_preserves_storage_and_config() {
        let temp = tempdir().unwrap();
        let auth_path = temp.path().join("auth.json");
        let state_root = temp.path().join("state");
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &auth_path);
        env.set("XDG_STATE_HOME", &state_root);
        write_auth_fixture(&auth_path);
        let (config, tokens_path) = seed_target();
        let original_config = config.clone();
        let original_tokens = fs::read(&tokens_path).unwrap();
        let body = r#"{"email":"user@example.com","rate_limit":{"primary_window":{"used_percent":1.0,"reset_at":2000000000}}}"#;
        let (endpoint, _) = server(200, body).await;

        assert!(
            run_import_with(
                "codex-import".to_string(),
                config.clone(),
                Some("codex-target".to_string()),
                &reqwest::Client::new(),
                Some(&endpoint),
            )
            .await
            .is_err()
        );
        assert_eq!(config, original_config);
        assert_eq!(fs::read(tokens_path).unwrap(), original_tokens);
    }
}
