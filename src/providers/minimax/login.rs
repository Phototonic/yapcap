// SPDX-License-Identifier: MPL-2.0

use crate::config::{Config, ManagedMinimaxAccountConfig};
use crate::key_authentication::{
    KeyAuthenticationAdapter, KeyAuthenticationTarget, prepare as prepare_key_authentication,
    prepare_for_reauth as prepare_key_authentication_for_reauth,
};
use crate::providers::minimax::{opencode, storage};
use chrono::{DateTime, Utc};

pub use crate::key_authentication::{
    KeyAuthenticationEvent as MinimaxLoginEvent, KeyAuthenticationState as MinimaxLoginState,
    KeyAuthenticationStatus as MinimaxLoginStatus,
};

pub(crate) struct MinimaxKeyAuthentication;

impl KeyAuthenticationAdapter for MinimaxKeyAuthentication {
    type Account = ManagedMinimaxAccountConfig;

    fn new_account_id() -> String {
        format!(
            "minimax-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        )
    }

    fn find_account(config: &Config, account_id: &str) -> Result<KeyAuthenticationTarget, String> {
        config
            .minimax_managed_accounts
            .iter()
            .find(|account| account.id == account_id)
            .map(|account| KeyAuthenticationTarget {
                id: account.id.clone(),
                label: account.label.clone(),
                created_at: account.created_at,
            })
            .ok_or_else(|| "Minimax account not found".to_string())
    }

    fn discover_api_key() -> Option<String> {
        opencode::discover_api_key()
    }

    fn empty_key_error() -> String {
        "API key is required".to_string()
    }

    fn build_account(
        account_id: String,
        label: String,
        created_at: DateTime<Utc>,
        authenticated_at: DateTime<Utc>,
    ) -> Self::Account {
        ManagedMinimaxAccountConfig {
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
            .map_err(|error| format!("Failed to save Minimax API key: {error}"))
    }
}

pub fn prepare() -> MinimaxLoginState {
    prepare_key_authentication::<MinimaxKeyAuthentication>()
}

pub fn prepare_for_reauth(config: Config, account_id: &str) -> Result<MinimaxLoginState, String> {
    prepare_key_authentication_for_reauth::<MinimaxKeyAuthentication>(&config, account_id)
}

pub(crate) fn save(state: &mut MinimaxLoginState) -> Result<ManagedMinimaxAccountConfig, String> {
    state.save::<MinimaxKeyAuthentication>()
}
