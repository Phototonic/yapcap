// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::error::{AppError, KimiError};
use crate::model::{AppState, ProviderId, UsageSnapshot};
use crate::providers::interface::{
    BoxFuture, ProviderAccountDescriptor, ProviderAccountHandle, ProviderAdapter,
    ProviderCapabilities,
};

pub(super) struct KimiAdapter;

impl ProviderAdapter for KimiAdapter {
    fn id(&self) -> ProviderId {
        ProviderId::Kimi
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_delete: true,
            supports_reauthentication: false,
            supports_background_status_refresh: false,
            requires_auth_prompt_on_auth_failure: false,
        }
    }

    fn discover_accounts(&self, _config: &Config) -> Vec<ProviderAccountDescriptor> {
        Vec::new()
    }

    fn delete_account(&self, _account_id: &str, _config: &mut Config) -> bool {
        false
    }

    fn reconcile_provider_accounts(&self, _config: &Config, _state: &mut AppState) {}

    fn system_active_account_id(&self, _config: &Config) -> Option<String> {
        None
    }

    fn fetch_account<'a>(
        &self,
        _handle: &'a ProviderAccountHandle,
        _client: &'a reqwest::Client,
    ) -> BoxFuture<'a, crate::error::Result<UsageSnapshot, AppError>> {
        Box::pin(async { Err(AppError::from(KimiError::NoUsageData)) })
    }
}
