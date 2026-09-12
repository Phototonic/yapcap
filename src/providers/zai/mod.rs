// SPDX-License-Identifier: MPL-2.0

pub mod account;
pub mod login;
pub mod opencode;
mod quota;
pub mod storage;

use crate::config::{Config, ManagedZaiAccountConfig};
use crate::error::ZaiError;
use crate::model::UsageSnapshot;
use crate::providers::zai::storage::normalize_api_key;

pub use account::discover_accounts;
pub use login::{ZaiLoginEvent, ZaiLoginState, ZaiLoginStatus};
pub use quota::parse;

const ZAI_API_URL: &str = "https://api.z.ai/api/monitor/usage/quota/limit";

pub fn sync_managed_accounts(config: &mut Config) -> bool {
    let original_len = config.zai_managed_accounts.len();
    config
        .zai_managed_accounts
        .retain(|account| matches!(account.api_key_source.as_str(), "stored" | "demo"));
    config.zai_managed_accounts.len() != original_len
}

pub async fn fetch(
    client: &reqwest::Client,
    account: &ManagedZaiAccountConfig,
) -> Result<UsageSnapshot, ZaiError> {
    fetch_with_endpoint(client, account, ZAI_API_URL).await
}

pub(crate) async fn fetch_with_endpoint(
    _client: &reqwest::Client,
    account: &ManagedZaiAccountConfig,
    endpoint: &str,
) -> Result<UsageSnapshot, ZaiError> {
    let client = crate::runtime::http_client_without_redirects();
    let stored_key = storage::load_api_key(&account.id).map_err(|_| ZaiError::LoginRequired)?;
    let api_key = normalize_api_key(&stored_key).map_err(|error| {
        if error == "API key is required" {
            ZaiError::LoginRequired
        } else {
            ZaiError::InvalidApiKey
        }
    })?;

    let mut response = send_request(&client, endpoint, &api_key, true).await?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        response = send_request(&client, endpoint, &api_key, false).await?;
    }

    classify_response(response).await
}

async fn send_request(
    client: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    bearer: bool,
) -> Result<reqwest::Response, ZaiError> {
    let authorization = if bearer {
        format!("Bearer {api_key}")
    } else {
        api_key.to_string()
    };
    let authorization = reqwest::header::HeaderValue::from_str(&authorization)
        .map_err(|_| ZaiError::InvalidApiKey)?;

    client
        .get(endpoint)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::AUTHORIZATION, authorization)
        .send()
        .await
        .map_err(ZaiError::UsageRequest)
}

async fn classify_response(response: reqwest::Response) -> Result<UsageSnapshot, ZaiError> {
    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(ZaiError::LoginRequired);
    }
    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let retry_after_secs = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok());
        return Err(ZaiError::RateLimited { retry_after_secs });
    }
    if status.is_server_error() || !status.is_success() {
        return Err(ZaiError::UsageHttp {
            status: status.as_u16(),
        });
    }

    let body = response.text().await.map_err(ZaiError::UsageRequest)?;
    parse(&body, chrono::Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::zai::storage::write_api_key;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::time::timeout;

    const SERVER_TIMEOUT: Duration = Duration::from_secs(5);

    fn account_id(suffix: &str) -> String {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("zai-test-{millis}-{suffix}")
    }

    fn account(id: &str) -> ManagedZaiAccountConfig {
        let now = chrono::Utc::now();
        ManagedZaiAccountConfig {
            id: id.to_string(),
            label: "Z.AI".to_string(),
            api_key_source: "stored".to_string(),
            created_at: now,
            updated_at: now,
            last_authenticated_at: Some(now),
        }
    }

    async fn server(responses: Vec<String>) -> (String, tokio::task::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let mut headers = Vec::new();
            for response in responses {
                let (mut stream, _) = timeout(SERVER_TIMEOUT, listener.accept())
                    .await
                    .expect("test server accept timed out")
                    .expect("test server accept failed");
                let mut request = Vec::new();
                let mut buffer = [0_u8; 1024];
                loop {
                    let read = timeout(SERVER_TIMEOUT, stream.read(&mut buffer))
                        .await
                        .expect("test server read timed out")
                        .expect("test server read failed");
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..read]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                let request = String::from_utf8_lossy(&request);
                headers.push(
                    request
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("authorization")
                                .then(|| value.trim().to_string())
                        })
                        .unwrap_or_default()
                        .to_string(),
                );
                timeout(SERVER_TIMEOUT, stream.write_all(response.as_bytes()))
                    .await
                    .expect("test server write timed out")
                    .expect("test server write failed");
                timeout(SERVER_TIMEOUT, stream.shutdown())
                    .await
                    .expect("test server shutdown timed out")
                    .expect("test server shutdown failed");
            }
            headers
        });
        (endpoint, task)
    }

    async fn redirect_server() -> (String, tokio::task::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let redirect_endpoint = format!("{endpoint}/redirected");
        let task = tokio::spawn(async move {
            let mut headers = Vec::new();
            let (mut stream, _) = timeout(SERVER_TIMEOUT, listener.accept())
                .await
                .expect("redirect test server accept timed out")
                .expect("redirect test server accept failed");
            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];
            loop {
                let read = timeout(SERVER_TIMEOUT, stream.read(&mut buffer))
                    .await
                    .expect("redirect test server read timed out")
                    .expect("redirect test server read failed");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let request = String::from_utf8_lossy(&request);
            headers.push(
                request
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("authorization")
                            .then(|| value.trim().to_string())
                    })
                    .unwrap_or_default()
                    .to_string(),
            );
            let redirect = response(302, &format!("Location: {redirect_endpoint}\r\n"), "");
            timeout(SERVER_TIMEOUT, stream.write_all(redirect.as_bytes()))
                .await
                .expect("redirect test server write timed out")
                .expect("redirect test server write failed");
            timeout(SERVER_TIMEOUT, stream.shutdown())
                .await
                .expect("redirect test server shutdown timed out")
                .expect("redirect test server shutdown failed");

            if let Ok(Ok((mut redirected, _))) =
                timeout(Duration::from_millis(250), listener.accept()).await
            {
                let mut redirected_request = Vec::new();
                loop {
                    let read = timeout(SERVER_TIMEOUT, redirected.read(&mut buffer))
                        .await
                        .expect("redirected request read timed out")
                        .expect("redirected request read failed");
                    if read == 0 {
                        break;
                    }
                    redirected_request.extend_from_slice(&buffer[..read]);
                    if redirected_request
                        .windows(4)
                        .any(|window| window == b"\r\n\r\n")
                    {
                        break;
                    }
                }
                let redirected_request = String::from_utf8_lossy(&redirected_request);
                headers.push(
                    redirected_request
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("authorization")
                                .then(|| value.trim().to_string())
                        })
                        .unwrap_or_default()
                        .to_string(),
                );
                let body = r#"{"code":200,"success":true,"data":{"limits":[]}}"#;
                let success = response(200, "Content-Type: application/json\r\n", body);
                let _ = redirected.write_all(success.as_bytes()).await;
                let _ = redirected.shutdown().await;
            }
            headers
        });
        (endpoint, task)
    }

    fn response(status: u16, headers: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status} Test\r\nConnection: close\r\nContent-Length: {}\r\n{headers}\r\n{body}",
            body.len(),
            headers = headers,
            body = body,
        )
    }

    #[test]
    fn sync_keeps_only_stored_and_demo_sources() {
        let now = chrono::Utc::now();
        let mut config = Config {
            zai_managed_accounts: vec![
                ManagedZaiAccountConfig {
                    id: "stored".to_string(),
                    label: "Stored".to_string(),
                    api_key_source: "stored".to_string(),
                    created_at: now,
                    updated_at: now,
                    last_authenticated_at: None,
                },
                ManagedZaiAccountConfig {
                    id: "demo".to_string(),
                    label: "Demo".to_string(),
                    api_key_source: "demo".to_string(),
                    created_at: now,
                    updated_at: now,
                    last_authenticated_at: None,
                },
                ManagedZaiAccountConfig {
                    id: "other".to_string(),
                    label: "Other".to_string(),
                    api_key_source: "env:ZAI_API_KEY".to_string(),
                    created_at: now,
                    updated_at: now,
                    last_authenticated_at: None,
                },
            ],
            ..Config::default()
        };

        assert!(sync_managed_accounts(&mut config));
        assert_eq!(config.zai_managed_accounts.len(), 2);
    }

    #[tokio::test]
    async fn bearer_success_sends_fixed_auth_shape() {
        let _env = crate::test_support::test_env();
        let id = account_id("bearer");
        write_api_key(&id, "key-value").unwrap();
        let body = r#"{"code":200,"success":true,"data":{"limits":[{"type":"TIME_LIMIT","unit":5,"number":1,"percentage":1}]}}"#;
        let (endpoint, task) = server(vec![response(
            200,
            "Content-Type: application/json\r\n",
            body,
        )])
        .await;

        let result = fetch_with_endpoint(&reqwest::Client::new(), &account(&id), &endpoint).await;
        let headers = task.await.unwrap();

        assert!(result.is_ok());
        assert_eq!(headers, ["Bearer key-value"]);
        let _ = crate::account_storage::ProviderAccountStorage::new(
            &crate::config::paths().zai_accounts_dir,
        )
        .delete_account(&id);
    }

    #[tokio::test]
    async fn retries_once_with_raw_authorization_after_bearer_401() {
        let _env = crate::test_support::test_env();
        let id = account_id("raw-retry");
        write_api_key(&id, "key-value").unwrap();
        let body = r#"{"code":200,"success":true,"data":{"limits":[{"type":"TIME_LIMIT","unit":5,"number":1,"percentage":1}]}}"#;
        let (endpoint, task) = server(vec![
            response(401, "", ""),
            response(200, "Content-Type: application/json\r\n", body),
        ])
        .await;

        let result = fetch_with_endpoint(&reqwest::Client::new(), &account(&id), &endpoint).await;
        let headers = task.await.unwrap();

        assert!(result.is_ok());
        assert_eq!(headers, ["Bearer key-value", "key-value"]);
        let _ = crate::account_storage::ProviderAccountStorage::new(
            &crate::config::paths().zai_accounts_dir,
        )
        .delete_account(&id);
    }

    #[tokio::test]
    async fn final_401_is_login_required_without_a_third_request() {
        let _env = crate::test_support::test_env();
        let id = account_id("auth-failure");
        write_api_key(&id, "key-value").unwrap();
        let (endpoint, task) = server(vec![response(401, "", ""), response(401, "", "")]).await;

        let result = fetch_with_endpoint(&reqwest::Client::new(), &account(&id), &endpoint).await;
        let headers = task.await.unwrap();

        assert!(matches!(result, Err(ZaiError::LoginRequired)));
        assert_eq!(headers, ["Bearer key-value", "key-value"]);
        let _ = crate::account_storage::ProviderAccountStorage::new(
            &crate::config::paths().zai_accounts_dir,
        )
        .delete_account(&id);
    }

    #[tokio::test]
    async fn rate_limit_preserves_numeric_retry_after() {
        let _env = crate::test_support::test_env();
        let id = account_id("rate-limit");
        write_api_key(&id, "key-value").unwrap();
        let (endpoint, task) = server(vec![response(429, "Retry-After: 17\r\n", "")]).await;

        let result = fetch_with_endpoint(&reqwest::Client::new(), &account(&id), &endpoint).await;
        let headers = task.await.unwrap();

        assert!(matches!(
            result,
            Err(ZaiError::RateLimited {
                retry_after_secs: Some(17)
            })
        ));
        assert_eq!(headers, ["Bearer key-value"]);
        let _ = crate::account_storage::ProviderAccountStorage::new(
            &crate::config::paths().zai_accounts_dir,
        )
        .delete_account(&id);
    }

    #[tokio::test]
    async fn server_errors_are_transient_without_retrying_raw_auth() {
        let _env = crate::test_support::test_env();
        let id = account_id("server-error");
        write_api_key(&id, "key-value").unwrap();
        let (endpoint, task) = server(vec![response(503, "", "")]).await;

        let result = fetch_with_endpoint(&reqwest::Client::new(), &account(&id), &endpoint).await;
        let headers = task.await.unwrap();

        assert!(matches!(&result, Err(ZaiError::UsageHttp { status: 503 })));
        assert_eq!(headers, ["Bearer key-value"]);
        assert!(
            result
                .as_ref()
                .err()
                .is_some_and(|error| error.is_transient())
        );
        let _ = crate::account_storage::ProviderAccountStorage::new(
            &crate::config::paths().zai_accounts_dir,
        )
        .delete_account(&id);
    }

    #[tokio::test]
    async fn redirects_are_rejected_without_forwarding_credentials() {
        let _env = crate::test_support::test_env();
        let id = account_id("redirect");
        write_api_key(&id, "key-value").unwrap();
        let (endpoint, task) = redirect_server().await;

        let result = fetch_with_endpoint(&reqwest::Client::new(), &account(&id), &endpoint).await;
        let headers = task.await.unwrap();

        assert!(matches!(result, Err(ZaiError::UsageHttp { status: 302 })));
        assert_eq!(headers, ["Bearer key-value"]);
        let _ = crate::account_storage::ProviderAccountStorage::new(
            &crate::config::paths().zai_accounts_dir,
        )
        .delete_account(&id);
    }
}
