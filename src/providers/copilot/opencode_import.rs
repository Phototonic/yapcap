// SPDX-License-Identifier: MPL-2.0

use super::device_flow::{DEFAULT_IDENTITY_URL, fetch_identity};
use super::login::{
    CopilotLoginEvent, CopilotLoginState, CopilotLoginStatus, commit_validated_login,
};
use crate::config::Config;
use crate::providers::opencode_auth::{OpenCodeCredential, discover};
use cosmic::iced::Task;

const PROVIDER_ID: &str = "github-copilot";

pub fn is_available() -> bool {
    discover(PROVIDER_ID).is_some_and(|credential| convert(credential).is_ok())
}

pub fn prepare_opencode_import(
    config: Config,
    target_account_id: Option<String>,
) -> Result<(CopilotLoginState, Task<CopilotLoginEvent>), String> {
    let expected_github_user_id = target_account_id
        .as_deref()
        .map(|target_account_id| {
            config
                .copilot_managed_accounts
                .iter()
                .find(|account| account.id == target_account_id)
                .map(|account| account.github_user_id)
                .ok_or_else(|| "Copilot account no longer exists".to_string())
        })
        .transpose()?;
    let flow_id = format!(
        "copilot-{}-{}",
        chrono::Utc::now().timestamp_millis(),
        std::process::id()
    );
    let state = CopilotLoginState {
        flow_id: flow_id.clone(),
        status: CopilotLoginStatus::Running,
        user_code: None,
        verification_uri: None,
        error: None,
        code_copied: false,
        importing_from_opencode: true,
    };
    let task = Task::perform(
        run_import(
            flow_id.clone(),
            config,
            target_account_id,
            expected_github_user_id,
        ),
        move |result| CopilotLoginEvent::Finished {
            flow_id,
            result: Box::new(result),
        },
    );
    Ok((state, task))
}

async fn run_import(
    _flow_id: String,
    config: Config,
    target_account_id: Option<String>,
    expected_github_user_id: Option<u64>,
) -> Result<super::login::CopilotLoginSuccess, String> {
    run_import_with(
        config,
        target_account_id,
        expected_github_user_id,
        &crate::runtime::http_client(),
        DEFAULT_IDENTITY_URL,
    )
    .await
}

async fn run_import_with(
    config: Config,
    target_account_id: Option<String>,
    expected_github_user_id: Option<u64>,
    client: &reqwest::Client,
    identity_endpoint: &str,
) -> Result<super::login::CopilotLoginSuccess, String> {
    let access_token = discover(PROVIDER_ID)
        .ok_or_else(|| "No compatible Copilot credential was found in OpenCode.".to_string())
        .and_then(convert)?;
    let identity = fetch_identity(client, identity_endpoint, &access_token)
        .await
        .map_err(|_| "OpenCode Copilot credentials could not be validated.".to_string())?;
    commit_validated_login(
        &config,
        expected_github_user_id,
        target_account_id.as_deref(),
        identity,
        access_token,
    )
    .map_err(|_| "OpenCode Copilot credentials could not be imported.".to_string())
}

fn convert(credential: OpenCodeCredential) -> Result<String, String> {
    let OpenCodeCredential::OAuth {
        refresh,
        enterprise_url,
        ..
    } = credential
    else {
        return Err("OpenCode does not contain a compatible Copilot OAuth credential.".to_string());
    };
    if enterprise_url.is_some_and(|url| !url.trim().is_empty()) {
        return Err("GitHub Enterprise credentials from OpenCode are not supported.".to_string());
    }
    if refresh.is_empty() {
        return Err("OpenCode does not contain a compatible Copilot OAuth credential.".to_string());
    }
    Ok(refresh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ManagedCopilotAccountConfig, paths};
    use crate::providers::copilot::storage::{CopilotMetadata, CopilotTokens, write_account};
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
        (format!("http://{address}/user"), handle)
    }

    #[test]
    fn converts_refresh_token_only() {
        assert_eq!(
            convert(OpenCodeCredential::OAuth {
                access: "not-the-github-token".to_string(),
                refresh: "github-token".to_string(),
                expires: 0,
                account_id: None,
                enterprise_url: None,
            })
            .unwrap(),
            "github-token"
        );
    }

    #[test]
    fn rejects_enterprise_and_empty_refresh() {
        assert!(
            convert(OpenCodeCredential::OAuth {
                access: "access".to_string(),
                refresh: "".to_string(),
                expires: 0,
                account_id: None,
                enterprise_url: None,
            })
            .is_err()
        );
        assert!(
            convert(OpenCodeCredential::OAuth {
                access: "access".to_string(),
                refresh: "token".to_string(),
                expires: 0,
                account_id: None,
                enterprise_url: Some("https://github.example".to_string()),
            })
            .is_err()
        );
    }

    #[test]
    fn availability_accepts_zero_expiry_and_rejects_enterprise_credentials() {
        assert!(availability_from(
            r#"{"github-copilot":{"type":"oauth","access":"ignored","refresh":"token","expires":0}}"#,
        ));
        assert!(!availability_from(
            r#"{"github-copilot":{"type":"oauth","access":"ignored","refresh":"token","expires":0,"enterpriseUrl":"https://github.example"}}"#,
        ));
    }

    #[tokio::test]
    async fn imports_validated_github_credentials_and_rereads_the_auth_file_at_click_time() {
        let temp = tempdir().unwrap();
        let auth_path = temp.path().join("auth.json");
        let state_root = temp.path().join("state");
        fs::write(
            &auth_path,
            r#"{"github-copilot":{"type":"oauth","access":"ignored","refresh":"github-first","expires":0}}"#,
        )
        .unwrap();
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &auth_path);
        env.set("XDG_STATE_HOME", &state_root);
        let client = reqwest::Client::new();
        let (endpoint, request) = server(200, r#"{"id":42,"login":"octocat"}"#).await;

        let first = run_import_with(Config::default(), None, None, &client, &endpoint)
            .await
            .unwrap();
        assert_eq!(first.account.github_user_id, 42);
        assert!(
            request
                .await
                .unwrap()
                .contains("authorization: token github-first")
        );

        fs::write(
            &auth_path,
            r#"{"github-copilot":{"type":"oauth","access":"ignored","refresh":"github-second","expires":0}}"#,
        )
        .unwrap();
        let (endpoint, request) = server(200, r#"{"id":42,"login":"octocat"}"#).await;
        let config = Config {
            copilot_managed_accounts: vec![first.account.clone()],
            ..Config::default()
        };
        run_import_with(config, None, None, &client, &endpoint)
            .await
            .unwrap();
        assert!(
            request
                .await
                .unwrap()
                .contains("authorization: token github-second")
        );
        let tokens = crate::providers::copilot::storage::load_tokens("copilot-42").unwrap();
        assert_eq!(tokens.access_token, "github-second");
    }

    #[tokio::test]
    async fn enterprise_and_identity_validation_failures_do_not_write_copilot_storage() {
        let temp = tempdir().unwrap();
        let auth_path = temp.path().join("auth.json");
        let state_root = temp.path().join("state");
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &auth_path);
        env.set("XDG_STATE_HOME", &state_root);
        let client = reqwest::Client::new();
        fs::write(
            &auth_path,
            r#"{"github-copilot":{"type":"oauth","access":"ignored","refresh":"token","expires":0,"enterpriseUrl":"https://github.example"}}"#,
        )
        .unwrap();
        assert!(
            run_import_with(
                Config::default(),
                None,
                None,
                &client,
                "http://127.0.0.1:9/user",
            )
            .await
            .is_err()
        );
        assert!(!crate::config::paths().copilot_accounts_dir.exists());

        fs::write(
            &auth_path,
            r#"{"github-copilot":{"type":"oauth","access":"ignored","refresh":"token","expires":0}}"#,
        )
        .unwrap();
        let (endpoint, _) = server(401, "{}").await;
        assert!(
            run_import_with(Config::default(), None, None, &client, &endpoint)
                .await
                .is_err()
        );
        assert!(!crate::config::paths().copilot_accounts_dir.exists());
    }

    #[tokio::test]
    async fn targeted_import_with_different_identity_preserves_storage_and_config() {
        let temp = tempdir().unwrap();
        let auth_path = temp.path().join("auth.json");
        let state_root = temp.path().join("state");
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &auth_path);
        env.set("XDG_STATE_HOME", &state_root);
        fs::write(
            &auth_path,
            r#"{"github-copilot":{"type":"oauth","access":"ignored","refresh":"new-token","expires":0}}"#,
        )
        .unwrap();
        let now = chrono::Utc::now();
        write_account(
            "copilot-42",
            &CopilotTokens {
                access_token: "original-token".to_string(),
            },
            &CopilotMetadata {
                github_user_id: 42,
                login: "octocat".to_string(),
                created_at: now,
                updated_at: now,
                last_authenticated_at: Some(now),
            },
        )
        .unwrap();
        let config = Config {
            copilot_managed_accounts: vec![ManagedCopilotAccountConfig {
                id: "copilot-42".to_string(),
                label: "octocat".to_string(),
                github_user_id: 42,
                login: "octocat".to_string(),
                created_at: now,
                updated_at: now,
                last_authenticated_at: Some(now),
            }],
            selected_copilot_account_ids: vec!["copilot-42".to_string()],
            ..Config::default()
        };
        let original_config = config.clone();
        let tokens_path = paths().copilot_accounts_dir.join("copilot-42/tokens.json");
        let original_tokens = fs::read(&tokens_path).unwrap();
        let (endpoint, _) = server(200, r#"{"id":7,"login":"other"}"#).await;

        assert!(
            run_import_with(
                config.clone(),
                Some("copilot-42".to_string()),
                Some(42),
                &reqwest::Client::new(),
                &endpoint,
            )
            .await
            .is_err()
        );
        assert_eq!(config, original_config);
        assert_eq!(fs::read(tokens_path).unwrap(), original_tokens);
    }
}
