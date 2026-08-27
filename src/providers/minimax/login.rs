// SPDX-License-Identifier: MPL-2.0

use crate::config::{Config, ManagedMinimaxAccountConfig};
use crate::providers::minimax::opencode;
use crate::providers::minimax::storage::write_api_key;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct MinimaxLoginState {
    pub account_id: String,
    pub label: String,
    pub status: MinimaxLoginStatus,
    pub api_key: String,
    pub api_key_from_opencode: bool,
    pub api_key_visible: bool,
    pub error: Option<String>,
    reauth_target: Option<MinimaxReauthTarget>,
}

#[derive(Debug, Clone)]
struct MinimaxReauthTarget {
    id: String,
    label: String,
    created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinimaxLoginStatus {
    Editing,
    Saved,
    Failed,
}

impl MinimaxLoginState {
    pub fn new(account_id: String) -> Self {
        Self {
            account_id,
            label: String::new(),
            status: MinimaxLoginStatus::Editing,
            api_key: String::new(),
            api_key_from_opencode: false,
            api_key_visible: false,
            error: None,
            reauth_target: None,
        }
    }

    pub fn update_label(&mut self, label: String) {
        self.label = label;
    }

    pub fn update_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
        self.api_key_from_opencode = false;
    }

    pub fn toggle_api_key_visibility(&mut self) {
        self.api_key_visible = !self.api_key_visible;
    }

    pub fn save(&self, _config: &mut Config) -> Result<ManagedMinimaxAccountConfig, String> {
        if self.api_key.is_empty() {
            return Err("API key is required".to_string());
        }

        let now = Utc::now();
        let (account_id, label, created_at) = self.reauth_target.as_ref().map_or_else(
            || (self.account_id.clone(), self.label.clone(), now),
            |target| (target.id.clone(), target.label.clone(), target.created_at),
        );
        let account = ManagedMinimaxAccountConfig {
            id: account_id,
            label,
            api_key_source: "stored".to_string(),
            created_at,
            updated_at: now,
            last_authenticated_at: Some(now),
        };

        write_api_key(&account.id, &self.api_key)?;

        Ok(account)
    }
}

#[derive(Debug, Clone)]
pub enum MinimaxLoginEvent {
    #[allow(dead_code)]
    Started,
    ApiKeyChanged(String),
    ApiKeyVisibilityToggled,
    LabelChanged(String),
    Saved,
    #[allow(dead_code)]
    Cancelled,
    #[allow(dead_code)]
    Failed(String),
}

pub fn prepare() -> MinimaxLoginState {
    let account_id = format!(
        "minimax-{}",
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let mut state = MinimaxLoginState::new(account_id);
    let discovered_api_key = opencode::discover_api_key();
    state.api_key_from_opencode = discovered_api_key.is_some();
    state.api_key = discovered_api_key.unwrap_or_default();
    state
}

pub fn prepare_for_reauth(config: Config, account_id: &str) -> Result<MinimaxLoginState, String> {
    let account = config
        .minimax_managed_accounts
        .iter()
        .find(|account| account.id == account_id)
        .ok_or_else(|| "Minimax account not found".to_string())?;
    let discovered_api_key = opencode::discover_api_key();
    let mut state = MinimaxLoginState::new(account.id.clone());
    state.label = account.label.clone();
    state.api_key_from_opencode = discovered_api_key.is_some();
    state.api_key = discovered_api_key.unwrap_or_default();
    state.reauth_target = Some(MinimaxReauthTarget {
        id: account.id.clone(),
        label: account.label.clone(),
        created_at: account.created_at,
    });
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn save_requires_an_api_key() {
        let state = MinimaxLoginState::new("minimax-test".to_string());

        assert_eq!(
            state.save(&mut Config::default()),
            Err("API key is required".to_string())
        );
    }

    #[test]
    fn prepare_marks_discovered_api_key_as_opencode_prefill() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("auth.json");
        fs::write(&path, r#"{"minimax":{"type":"api","key":"test-key"}}"#).unwrap();
        let mut env = crate::test_support::test_env();
        env.set("YAPCAP_OPENCODE_AUTH_PATH", &path);

        let state = prepare();

        assert_eq!(state.api_key, "test-key");
        assert!(state.api_key_from_opencode);
        assert!(!state.api_key_visible);
    }

    #[test]
    fn prepare_without_discovered_api_key_is_masked_without_prefill() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("missing-auth.json");
        let mut env = crate::test_support::test_env();
        env.set("YAPCAP_OPENCODE_AUTH_PATH", &path);

        let state = prepare();

        assert!(state.api_key.is_empty());
        assert!(!state.api_key_from_opencode);
        assert!(!state.api_key_visible);
    }

    #[test]
    fn update_api_key_clears_opencode_provenance() {
        let mut state = MinimaxLoginState::new("minimax-test".to_string());
        state.api_key = "test-key".to_string();
        state.api_key_from_opencode = true;

        state.update_api_key("typed-key".to_string());

        assert_eq!(state.api_key, "typed-key");
        assert!(!state.api_key_from_opencode);
    }

    #[test]
    fn api_key_visibility_defaults_masked_and_toggles() {
        let mut state = MinimaxLoginState::new("minimax-test".to_string());

        assert!(!state.api_key_visible);
        state.toggle_api_key_visibility();
        assert!(state.api_key_visible);
        state.toggle_api_key_visibility();
        assert!(!state.api_key_visible);
    }

    #[test]
    fn prepare_for_reauth_preserves_target_identity() {
        let created_at = Utc::now() - chrono::Duration::days(1);
        let config = Config {
            minimax_managed_accounts: vec![ManagedMinimaxAccountConfig {
                id: "minimax-existing".to_string(),
                label: "Existing".to_string(),
                api_key_source: "stored".to_string(),
                created_at,
                updated_at: created_at,
                last_authenticated_at: Some(created_at),
            }],
            ..Config::default()
        };

        let state = prepare_for_reauth(config, "minimax-existing").unwrap();

        assert_eq!(state.account_id, "minimax-existing");
        assert_eq!(state.label, "Existing");
    }
}
