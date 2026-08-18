// SPDX-License-Identifier: MPL-2.0

use crate::config::host_user_home_dir;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const OPENCODE_AUTH_PATH_ENV: &str = "YAPCAP_OPENCODE_AUTH_PATH";

pub fn discover_api_key() -> Option<String> {
    let path = std::env::var_os(OPENCODE_AUTH_PATH_ENV)
        .map(PathBuf::from)
        .or_else(|| {
            host_user_home_dir().map(|home| home.join(".local/share/opencode/auth.json"))
        })?;
    discover_api_key_at(&path)
}

fn discover_api_key_at(path: &Path) -> Option<String> {
    let body = fs::read_to_string(path).ok()?;
    let auth: Value = serde_json::from_str(&body).ok()?;
    auth.get("kimi-for-coding")?
        .get("key")?
        .as_str()
        .filter(|key| !key.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn discovers_kimi_for_coding_api_key() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(
            &path,
            r#"{"kimi-for-coding":{"type":"api","key":"test-key"}}"#,
        )
        .unwrap();

        assert_eq!(discover_api_key_at(&path).as_deref(), Some("test-key"));
    }

    #[test]
    fn missing_auth_file_has_no_api_key() {
        let temp = tempdir().unwrap();

        assert_eq!(discover_api_key_at(&temp.path().join("auth.json")), None);
    }

    #[test]
    fn malformed_auth_file_has_no_api_key() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(&path, "not-json").unwrap();

        assert_eq!(discover_api_key_at(&path), None);
    }

    #[test]
    fn wrong_auth_shape_has_no_api_key() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(&path, r#"{"kimi-for-coding":{"token":"test-key"}}"#).unwrap();

        assert_eq!(discover_api_key_at(&path), None);
    }

    #[test]
    fn environment_override_selects_auth_file() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(&path, r#"{"kimi-for-coding":{"key":"test-key"}}"#).unwrap();
        let mut env = crate::test_support::test_env();
        env.set(OPENCODE_AUTH_PATH_ENV, &path);

        assert_eq!(discover_api_key().as_deref(), Some("test-key"));
    }
}
