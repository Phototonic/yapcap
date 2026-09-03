// SPDX-License-Identifier: MPL-2.0

use crate::config::{Config, ManagedOpenCodeGoAccountConfig};
use crate::key_authentication::{
    KeyAuthenticationAdapter, KeyAuthenticationTarget, prepare as prepare_key_authentication,
    prepare_for_reauth as prepare_key_authentication_for_reauth,
};
use crate::providers::opencode_go::{opencode, storage};
use chrono::{DateTime, Utc};

pub use crate::key_authentication::{
    KeyAuthenticationEvent as OpenCodeGoLoginEvent, KeyAuthenticationState as OpenCodeGoLoginState,
    KeyAuthenticationStatus as OpenCodeGoLoginStatus,
};

pub(crate) struct OpenCodeGoKeyAuthentication;

impl KeyAuthenticationAdapter for OpenCodeGoKeyAuthentication {
    type Account = ManagedOpenCodeGoAccountConfig;

    fn new_account_id() -> String {
        format!(
            "opencode-go-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        )
    }

    fn find_account(config: &Config, account_id: &str) -> Result<KeyAuthenticationTarget, String> {
        config
            .opencode_go_managed_accounts
            .iter()
            .find(|account| account.id == account_id)
            .map(|account| KeyAuthenticationTarget {
                id: account.id.clone(),
                label: account.label.clone(),
                created_at: account.created_at,
            })
            .ok_or_else(|| "OpenCode Go account not found".to_string())
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
        ManagedOpenCodeGoAccountConfig {
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
            .map_err(|error| format!("Failed to save OpenCode Go API key: {error}"))
    }
}

pub fn prepare() -> OpenCodeGoLoginState {
    prepare_key_authentication::<OpenCodeGoKeyAuthentication>()
}

pub fn prepare_for_reauth(
    config: Config,
    account_id: &str,
) -> Result<OpenCodeGoLoginState, String> {
    prepare_key_authentication_for_reauth::<OpenCodeGoKeyAuthentication>(&config, account_id)
}

pub(crate) fn save(
    state: &mut OpenCodeGoLoginState,
) -> Result<ManagedOpenCodeGoAccountConfig, String> {
    state.save::<OpenCodeGoKeyAuthentication>()
}
