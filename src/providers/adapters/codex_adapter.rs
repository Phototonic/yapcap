// SPDX-License-Identifier: MPL-2.0

use super::{codex_system_active_account_id, reconcile_provider_account_descriptors};
use crate::config::{Config, managed_codex_account_dir};
use crate::error::AppError;
use crate::model::{AppState, AuthState, ProviderHealth, ProviderId, UsageSnapshot};
use crate::providers::adapters::remove_managed_codex_account;
use crate::providers::codex;
use crate::providers::interface::{
    BoxFuture, ProviderAccountAction, ProviderAccountDescriptor, ProviderAccountHandle,
    ProviderAdapter, ProviderCapabilities, ProviderLoginKind,
};

pub(super) struct CodexAdapter;

impl ProviderAdapter for CodexAdapter {
    fn id(&self) -> ProviderId {
        ProviderId::Codex
    }

    fn login_kind(&self) -> ProviderLoginKind {
        ProviderLoginKind::Codex
    }

    fn supports_opencode_import(&self) -> bool {
        codex::opencode_import_available()
    }

    fn selection_required_message(&self) -> Option<String> {
        Some(crate::fl!("codex-account-select-required"))
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_background_status_refresh: false,
            requires_auth_prompt_on_auth_failure: false,
        }
    }

    fn discover_accounts(&self, config: &Config) -> Vec<ProviderAccountDescriptor> {
        let restore_from_opencode_available = codex::opencode_import_available();
        let discovered_accounts = codex::discover_accounts(config);
        codex_account_descriptors(
            config,
            &discovered_accounts,
            restore_from_opencode_available,
        )
    }

    fn sync_managed_accounts(&self, config: &mut Config) -> bool {
        codex::sync_managed_accounts(config)
    }

    fn delete_account(&self, account_id: &str, config: &mut Config) -> bool {
        let account = config
            .codex_managed_accounts
            .iter()
            .find(|a| a.id == account_id)
            .cloned();
        let Some(account) = account else {
            return false;
        };
        remove_managed_codex_account(&account.id);
        config.codex_managed_accounts.retain(|a| a.id != account_id);
        config
            .selected_codex_account_ids
            .retain(|id| id != account_id);
        true
    }

    fn reconcile_provider_accounts(&self, config: &Config, state: &mut AppState) {
        let discovered_accounts = codex::discover_accounts(config);
        let accounts = codex_account_descriptors(
            config,
            &discovered_accounts,
            codex::opencode_import_available(),
        );
        reconcile_provider_account_descriptors(self.id(), config, state, &accounts);
        for discovered in discovered_accounts {
            if discovered.credentials_available {
                continue;
            }
            let Some(account) = state.provider_accounts.iter_mut().find(|account| {
                account.provider == ProviderId::Codex && account.account_id == discovered.id
            }) else {
                continue;
            };
            account.health = ProviderHealth::Error;
            account.retry_after = None;
            if let Some(error) = discovered.credentials_error {
                account.auth_state = AuthState::Error;
                account.error = Some(error);
            } else {
                account.auth_state = AuthState::ActionRequired;
                account.error = Some(
                    "Codex credentials are missing; restore from OpenCode or sign in again"
                        .to_string(),
                );
            }
        }
        if let Some(provider_state) = state.provider_mut(ProviderId::Codex) {
            provider_state.system_active_account_id = self.system_active_account_id(config);
        }
    }

    fn system_active_account_id(&self, config: &Config) -> Option<String> {
        codex_system_active_account_id(&config.codex_managed_accounts)
    }

    fn fetch_account<'a>(
        &self,
        handle: &'a ProviderAccountHandle,
        client: &'a reqwest::Client,
    ) -> BoxFuture<'a, crate::error::Result<UsageSnapshot, AppError>> {
        let provider = self.id();
        Box::pin(async move {
            match handle {
                ProviderAccountHandle::Codex(account) => {
                    codex::fetch(client, &account.id, managed_codex_account_dir(&account.id))
                        .await
                        .map_err(AppError::from)
                }
                _ => Err(AppError::InvalidAccountHandle { provider }),
            }
        })
    }
}

fn codex_account_descriptors(
    config: &Config,
    discovered_accounts: &[codex::CodexAccount],
    restore_from_opencode_available: bool,
) -> Vec<ProviderAccountDescriptor> {
    discovered_accounts
        .iter()
        .filter_map(|account| {
            config
                .codex_managed_accounts
                .iter()
                .find(|managed| managed.id == account.id)
                .cloned()
                .map(|managed| {
                    let mut actions = vec![
                        ProviderAccountAction::Delete,
                        ProviderAccountAction::Reauthenticate,
                    ];
                    if !account.credentials_available
                        && account.credentials_error.is_none()
                        && restore_from_opencode_available
                    {
                        actions.push(ProviderAccountAction::RestoreFromOpenCode);
                    }
                    ProviderAccountDescriptor {
                        provider: ProviderId::Codex,
                        account_id: account.id.clone(),
                        label: account.label.clone(),
                        actions,
                        handle: ProviderAccountHandle::Codex(managed),
                    }
                })
        })
        .collect()
}
