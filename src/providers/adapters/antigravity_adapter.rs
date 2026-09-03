// SPDX-License-Identifier: MPL-2.0

use super::reconcile_provider_account_descriptors;
use crate::account_storage::ProviderAccountStorage;
use crate::config::{Config, managed_antigravity_account_dir, paths};
use crate::error::AppError;
use crate::model::{AppState, ProviderId, UsageSnapshot};
use crate::providers::antigravity;
use crate::providers::interface::{
    BoxFuture, ProviderAccountAction, ProviderAccountDescriptor, ProviderAccountHandle,
    ProviderAdapter, ProviderCapabilities, ProviderLoginKind,
};

pub(super) struct AntigravityAdapter;

impl ProviderAdapter for AntigravityAdapter {
    fn id(&self) -> ProviderId {
        ProviderId::Antigravity
    }

    fn login_kind(&self) -> ProviderLoginKind {
        ProviderLoginKind::Antigravity
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_background_status_refresh: false,
            requires_auth_prompt_on_auth_failure: false,
        }
    }

    fn discover_accounts(&self, config: &Config) -> Vec<ProviderAccountDescriptor> {
        config
            .antigravity_managed_accounts
            .iter()
            .cloned()
            .map(|managed| ProviderAccountDescriptor {
                provider: self.id(),
                account_id: managed.id.clone(),
                label: managed.label.clone(),
                actions: vec![
                    ProviderAccountAction::Delete,
                    ProviderAccountAction::Reauthenticate,
                ],
                handle: ProviderAccountHandle::Antigravity(managed),
            })
            .collect()
    }

    fn sync_managed_accounts(&self, config: &mut Config) -> bool {
        antigravity::sync_managed_accounts(config)
    }

    fn delete_account(&self, account_id: &str, config: &mut Config) -> bool {
        if !config
            .antigravity_managed_accounts
            .iter()
            .any(|a| a.id == account_id)
        {
            return false;
        }
        let storage = ProviderAccountStorage::new(paths().antigravity_accounts_dir);
        if let Err(error) = storage.delete_account(account_id) {
            tracing::warn!(account_id, error = %error, "failed to delete antigravity account");
        }
        config
            .antigravity_managed_accounts
            .retain(|a| a.id != account_id);
        config
            .selected_antigravity_account_ids
            .retain(|id| id != account_id);
        true
    }

    fn reconcile_provider_accounts(&self, config: &Config, state: &mut AppState) {
        let accounts = self.discover_accounts(config);
        reconcile_provider_account_descriptors(self.id(), config, state, &accounts);
    }

    fn fetch_account<'a>(
        &self,
        handle: &'a ProviderAccountHandle,
        client: &'a reqwest::Client,
    ) -> BoxFuture<'a, crate::error::Result<UsageSnapshot, AppError>> {
        let provider = self.id();
        Box::pin(async move {
            match handle {
                ProviderAccountHandle::Antigravity(managed) => antigravity::fetch(
                    client,
                    &managed.id,
                    managed_antigravity_account_dir(&managed.id),
                )
                .await
                .map_err(AppError::from),
                _ => Err(AppError::InvalidAccountHandle { provider }),
            }
        })
    }
}
