// SPDX-License-Identifier: MPL-2.0

use crate::config::{Config, ManagedZaiAccountConfig};
use crate::key_authentication::{
    KeyAuthenticationAdapter, KeyAuthenticationTarget, prepare as prepare_key_authentication,
    prepare_for_reauth as prepare_key_authentication_for_reauth,
};
use crate::providers::zai::{opencode, storage};
use chrono::{DateTime, Utc};

pub use crate::key_authentication::{
    KeyAuthenticationEvent as ZaiLoginEvent, KeyAuthenticationState as ZaiLoginState,
    KeyAuthenticationStatus as ZaiLoginStatus,
};

pub(crate) struct ZaiKeyAuthentication;

impl KeyAuthenticationAdapter for ZaiKeyAuthentication {
    type Account = ManagedZaiAccountConfig;

    fn new_account_id() -> String {
        format!(
            "zai-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        )
    }

    fn find_account(config: &Config, account_id: &str) -> Result<KeyAuthenticationTarget, String> {
        config
            .zai_managed_accounts
            .iter()
            .find(|account| account.id == account_id)
            .map(|account| KeyAuthenticationTarget {
                id: account.id.clone(),
                label: account.label.clone(),
                created_at: account.created_at,
            })
            .ok_or_else(|| "Z.AI account not found".to_string())
    }

    fn discover_api_key() -> Option<String> {
        opencode::discover_api_key()
    }

    fn empty_key_error() -> String {
        "API key is required".to_string()
    }

    fn validate(
        config: &Config,
        account_id: &str,
        label: &str,
        api_key: &str,
    ) -> Result<(), String> {
        let api_key = storage::normalize_api_key(api_key)?;
        if label.trim().is_empty() {
            return Err("Account name is required".to_string());
        }
        if config
            .zai_managed_accounts
            .iter()
            .any(|account| account.id != account_id && account.label == label)
        {
            return Err("An account with this name already exists".to_string());
        }
        if config.zai_managed_accounts.iter().any(|account| {
            account.id != account_id
                && storage::load_api_key(&account.id)
                    .ok()
                    .and_then(|stored_key| storage::normalize_api_key(&stored_key).ok())
                    .is_some_and(|stored_key| stored_key == api_key)
        }) {
            return Err("An account with this API key already exists".to_string());
        }
        Ok(())
    }

    fn build_account(
        account_id: String,
        label: String,
        created_at: DateTime<Utc>,
        authenticated_at: DateTime<Utc>,
    ) -> Self::Account {
        ManagedZaiAccountConfig {
            id: account_id,
            label,
            api_key_source: "stored".to_string(),
            created_at,
            updated_at: authenticated_at,
            last_authenticated_at: Some(authenticated_at),
        }
    }

    fn persist(account: &Self::Account, api_key: &str) -> Result<(), String> {
        storage::write_api_key(&account.id, api_key)
            .map_err(|error| format!("Failed to save Z.AI API key: {error}"))
    }
}

pub fn prepare() -> ZaiLoginState {
    prepare_key_authentication::<ZaiKeyAuthentication>()
}

pub fn prepare_for_reauth(config: Config, account_id: &str) -> Result<ZaiLoginState, String> {
    prepare_key_authentication_for_reauth::<ZaiKeyAuthentication>(&config, account_id)
}

pub(crate) fn save(
    config: &Config,
    state: &mut ZaiLoginState,
) -> Result<ManagedZaiAccountConfig, String> {
    state.save::<ZaiKeyAuthentication>(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn prepares_with_ordered_opencode_key() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(
            &path,
            r#"{"zai-coding-plan":{"type":"api","key":"primary"},"zai":{"type":"api","key":"alias"}}"#,
        )
        .unwrap();
        let mut env = crate::test_support::test_env();
        env.set(
            crate::providers::opencode_auth::OPENCODE_AUTH_PATH_ENV,
            &path,
        );
        env.remove(crate::providers::opencode_auth::OPENCODE_AUTH_CONTENT_ENV);

        let state = prepare();

        assert_eq!(state.api_key, "primary");
        assert!(state.api_key_from_opencode);
    }

    #[test]
    fn save_normalizes_key_and_keeps_metadata_secret_free() {
        let _env = crate::test_support::test_env();
        let mut state = ZaiLoginState::new("zai-test".to_string());
        state.update_label("Test account".to_string());
        state.update_api_key("  key-value  ".to_string());
        let config = Config::default();

        let account = state.save::<ZaiKeyAuthentication>(&config).unwrap();

        assert_eq!(storage::load_api_key(&account.id).unwrap(), "key-value");
        assert_eq!(account.api_key_source, "stored");
        assert!(
            serde_json::to_string(&account)
                .unwrap()
                .find("key-value")
                .is_none()
        );
        let _ = fs::remove_dir_all(paths().zai_accounts_dir.join(account.id));
    }

    #[test]
    fn save_rejects_control_characters_without_persisting() {
        let _env = crate::test_support::test_env();
        let mut state = ZaiLoginState::new("zai-test-control".to_string());
        state.update_label("Test account".to_string());
        state.update_api_key("valid\nkey".to_string());

        let result = state.save::<ZaiKeyAuthentication>(&Config::default());

        assert!(result.is_err());
        assert!(state.error.is_some());
        assert!(storage::load_api_key("zai-test-control").is_err());
    }
}
