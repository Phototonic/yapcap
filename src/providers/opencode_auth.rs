// SPDX-License-Identifier: MPL-2.0

use crate::config::host_user_home_dir;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub const OPENCODE_AUTH_PATH_ENV: &str = "YAPCAP_OPENCODE_AUTH_PATH";
pub const OPENCODE_AUTH_CONTENT_ENV: &str = "OPENCODE_AUTH_CONTENT";

pub enum OpenCodeCredential {
    Api {
        key: String,
    },
    OAuth {
        access: String,
        refresh: String,
        expires: i64,
        account_id: Option<String>,
        enterprise_url: Option<String>,
    },
    WellKnown {
        key: String,
        token: String,
    },
}

impl OpenCodeCredential {
    fn is_valid(&self) -> bool {
        match self {
            Self::Api { key } => !key.is_empty(),
            Self::OAuth {
                access,
                refresh,
                expires,
                account_id,
                enterprise_url,
            } => {
                let _ = (expires, account_id, enterprise_url);
                !access.is_empty() && !refresh.is_empty()
            }
            Self::WellKnown { key, token } => !key.is_empty() && !token.is_empty(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum RawCredential {
    #[serde(rename = "api")]
    Api { key: String },
    #[serde(rename = "oauth")]
    OAuth {
        access: String,
        refresh: String,
        expires: i64,
        #[serde(rename = "accountId")]
        account_id: Option<String>,
        #[serde(rename = "enterpriseUrl")]
        enterprise_url: Option<String>,
    },
    #[serde(rename = "wellknown")]
    WellKnown { key: String, token: String },
}

pub fn auth_path() -> Option<PathBuf> {
    std::env::var_os(OPENCODE_AUTH_PATH_ENV)
        .map(PathBuf::from)
        .or_else(|| host_user_home_dir().map(|home| home.join(".local/share/opencode/auth.json")))
}

pub fn discover(provider_id: &str) -> Option<OpenCodeCredential> {
    let auth = auth_json()?;
    discover_from_auth(&auth, provider_id)
}

pub fn discover_api_key(provider_ids: &[&str]) -> Option<String> {
    let auth = auth_json()?;
    discover_api_key_from_auth(&auth, provider_ids)
}

pub fn has_api_key_at_path(path: &Path, provider_ids: &[&str]) -> bool {
    let auth = fs::read_to_string(path)
        .ok()
        .and_then(|body| serde_json::from_str(&body).ok());
    auth.as_ref()
        .and_then(|auth| discover_api_key_from_auth(auth, provider_ids))
        .is_some()
}

fn discover_from_auth(auth: &Value, provider_id: &str) -> Option<OpenCodeCredential> {
    let entry = auth.get(provider_id)?.clone();
    let credential: RawCredential = serde_json::from_value(entry).ok()?;
    let credential = into_credential(credential);

    credential.is_valid().then_some(credential)
}

fn discover_api_key_from_auth(auth: &Value, provider_ids: &[&str]) -> Option<String> {
    provider_ids.iter().find_map(|provider_id| {
        let credential = discover_from_auth(auth, provider_id)?;
        match credential {
            OpenCodeCredential::Api { key } => usable_api_key(&key),
            OpenCodeCredential::OAuth { .. } | OpenCodeCredential::WellKnown { .. } => None,
        }
    })
}

fn into_credential(credential: RawCredential) -> OpenCodeCredential {
    match credential {
        RawCredential::Api { key } => OpenCodeCredential::Api { key },
        RawCredential::OAuth {
            access,
            refresh,
            expires,
            account_id,
            enterprise_url,
        } => OpenCodeCredential::OAuth {
            access,
            refresh,
            expires,
            account_id,
            enterprise_url,
        },
        RawCredential::WellKnown { key, token } => OpenCodeCredential::WellKnown { key, token },
    }
}

fn usable_api_key(key: &str) -> Option<String> {
    let key = key.trim();
    if key.is_empty() || key.chars().any(char::is_control) {
        None
    } else {
        Some(key.to_string())
    }
}

fn auth_json() -> Option<Value> {
    let from_content = std::env::var(OPENCODE_AUTH_CONTENT_ENV)
        .ok()
        .filter(|content| !content.is_empty())
        .and_then(|content| serde_json::from_str(&content).ok());
    from_content.or_else(|| {
        let path = auth_path()?;
        let body = fs::read_to_string(path).ok()?;
        serde_json::from_str(&body).ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn discover_from(contents: &str, provider_id: &str) -> Option<OpenCodeCredential> {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(&path, contents).unwrap();
        let mut env = crate::test_support::test_env();
        env.remove(OPENCODE_AUTH_CONTENT_ENV);
        env.set(OPENCODE_AUTH_PATH_ENV, &path);
        discover(provider_id)
    }

    #[test]
    fn discovers_api_credentials() {
        let credential = discover_from(
            r#"{"provider":{"type":"api","key":"test-key"}}"#,
            "provider",
        );

        match credential {
            Some(OpenCodeCredential::Api { key }) => assert_eq!(key, "test-key"),
            _ => panic!("expected API credential"),
        }
    }

    #[test]
    fn discovers_oauth_credentials() {
        let credential = discover_from(
            r#"{"provider":{"type":"oauth","refresh":"r","access":"a","expires":123}}"#,
            "provider",
        );

        match credential {
            Some(OpenCodeCredential::OAuth {
                access,
                refresh,
                expires,
                account_id,
                enterprise_url,
            }) => {
                assert_eq!(access, "a");
                assert_eq!(refresh, "r");
                assert_eq!(expires, 123);
                assert_eq!(account_id, None);
                assert_eq!(enterprise_url, None);
            }
            _ => panic!("expected OAuth credential"),
        }
    }

    #[test]
    fn discovers_oauth_optional_fields() {
        let credential = discover_from(
            r#"{"provider":{"type":"oauth","refresh":"r","access":"a","expires":123,"accountId":"account","enterpriseUrl":"example.com"}}"#,
            "provider",
        );

        match credential {
            Some(OpenCodeCredential::OAuth {
                account_id,
                enterprise_url,
                ..
            }) => {
                assert_eq!(account_id.as_deref(), Some("account"));
                assert_eq!(enterprise_url.as_deref(), Some("example.com"));
            }
            _ => panic!("expected OAuth credential"),
        }
    }

    #[test]
    fn discovers_oauth_with_zero_expiry() {
        let credential = discover_from(
            r#"{"provider":{"type":"oauth","refresh":"r","access":"a","expires":0}}"#,
            "provider",
        );

        assert!(matches!(
            credential,
            Some(OpenCodeCredential::OAuth { expires: 0, .. })
        ));
    }

    #[test]
    fn discovers_wellknown_credentials() {
        let credential = discover_from(
            r#"{"provider":{"type":"wellknown","key":"k","token":"t"}}"#,
            "provider",
        );

        match credential {
            Some(OpenCodeCredential::WellKnown { key, token }) => {
                assert_eq!(key, "k");
                assert_eq!(token, "t");
            }
            _ => panic!("expected well-known credential"),
        }
    }

    #[test]
    fn finds_the_requested_provider_among_multiple_entries() {
        let credential = discover_from(
            r#"{"first":{"type":"api","key":"one"},"second":{"type":"api","key":"two"}}"#,
            "second",
        );

        match credential {
            Some(OpenCodeCredential::Api { key }) => assert_eq!(key, "two"),
            _ => panic!("expected API credential"),
        }
    }

    #[test]
    fn different_provider_ids_select_different_credentials() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(
            &path,
            r#"{"first":{"type":"api","key":"one"},"second":{"type":"wellknown","key":"two","token":"token"}}"#,
        )
        .unwrap();
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &path);

        assert!(matches!(
            discover("first"),
            Some(OpenCodeCredential::Api { key }) if key == "one"
        ));
        assert!(matches!(
            discover("second"),
            Some(OpenCodeCredential::WellKnown { key, token }) if key == "two" && token == "token"
        ));
    }

    #[test]
    fn missing_auth_file_returns_none() {
        let temp = tempdir().unwrap();
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, temp.path().join("auth.json"));

        assert!(discover("provider").is_none());
    }

    #[test]
    fn malformed_auth_file_returns_none() {
        assert!(discover_from("not-json", "provider").is_none());
    }

    #[test]
    fn unknown_credential_type_returns_none() {
        assert!(
            discover_from(
                r#"{"provider":{"type":"future","data":"value"}}"#,
                "provider"
            )
            .is_none()
        );
    }

    #[test]
    fn missing_required_field_returns_none() {
        assert!(discover_from(r#"{"provider":{"type":"api"}}"#, "provider").is_none());
    }

    #[test]
    fn empty_api_key_returns_none() {
        assert!(discover_from(r#"{"provider":{"type":"api","key":""}}"#, "provider").is_none());
    }

    #[test]
    fn empty_oauth_access_returns_none() {
        assert!(
            discover_from(
                r#"{"provider":{"type":"oauth","refresh":"r","access":"","expires":123}}"#,
                "provider"
            )
            .is_none()
        );
    }

    #[test]
    fn empty_wellknown_token_returns_none() {
        assert!(
            discover_from(
                r#"{"provider":{"type":"wellknown","key":"k","token":""}}"#,
                "provider"
            )
            .is_none()
        );
    }

    #[test]
    fn missing_provider_returns_none() {
        assert!(discover_from(r#"{"other":{"type":"api","key":"key"}}"#, "provider").is_none());
    }

    #[test]
    fn environment_override_selects_auth_file() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(&path, r#"{"provider":{"type":"api","key":"override-key"}}"#).unwrap();
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &path);

        assert!(matches!(
            discover("provider"),
            Some(OpenCodeCredential::Api { key }) if key == "override-key"
        ));
    }

    #[test]
    fn auth_content_overrides_auth_file() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(&path, r#"{"provider":{"type":"api","key":"file-key"}}"#).unwrap();
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &path);
        env.set(
            OPENCODE_AUTH_CONTENT_ENV,
            r#"{"provider":{"type":"api","key":"content-key"}}"#,
        );

        assert!(matches!(
            discover("provider"),
            Some(OpenCodeCredential::Api { key }) if key == "content-key"
        ));
    }

    #[test]
    fn malformed_auth_content_falls_back_to_auth_file() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(&path, r#"{"provider":{"type":"api","key":"file-key"}}"#).unwrap();
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &path);
        env.set(OPENCODE_AUTH_CONTENT_ENV, "not-json");

        assert!(matches!(
            discover("provider"),
            Some(OpenCodeCredential::Api { key }) if key == "file-key"
        ));
    }

    #[test]
    fn discovers_api_key_in_requested_order() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(
            &path,
            r#"{"zai-coding-plan":{"type":"api","key":"primary"},"zai":{"type":"api","key":"alias"}}"#,
        )
        .unwrap();
        let mut env = crate::test_support::test_env();
        env.remove(OPENCODE_AUTH_CONTENT_ENV);
        env.set(OPENCODE_AUTH_PATH_ENV, &path);

        assert_eq!(
            discover_api_key(&["zai-coding-plan", "zai"]).as_deref(),
            Some("primary")
        );
    }

    #[test]
    fn api_key_discovery_uses_alias_when_primary_is_not_usable() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(
            &path,
            r#"{"zai-coding-plan":{"type":"oauth","refresh":"r","access":"a","expires":1},"zai":{"type":"api","key":" alias "}}"#,
        )
        .unwrap();
        let mut env = crate::test_support::test_env();
        env.remove(OPENCODE_AUTH_CONTENT_ENV);
        env.set(OPENCODE_AUTH_PATH_ENV, &path);

        assert_eq!(
            discover_api_key(&["zai-coding-plan", "zai"]).as_deref(),
            Some("alias")
        );
    }

    #[test]
    fn api_key_presence_at_path_returns_only_usable_api_credential_fact() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(&path, r#"{"zai":{"type":"api","key":"key"}}"#).unwrap();

        assert!(has_api_key_at_path(&path, &["zai-coding-plan", "zai"]));
        assert!(!has_api_key_at_path(&path, &["other"]));
    }

    #[test]
    fn api_key_presence_at_path_rejects_empty_control_and_non_api_credentials() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");

        for body in [
            r#"{"zai":{"type":"api","key":" "}}"#,
            r#"{"zai":{"type":"api","key":"bad\nkey"}}"#,
            r#"{"zai":{"type":"oauth","refresh":"r","access":"a","expires":1}}"#,
            "not-json",
        ] {
            fs::write(&path, body).unwrap();
            assert!(!has_api_key_at_path(&path, &["zai"]));
        }
    }
}
