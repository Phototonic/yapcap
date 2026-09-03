// SPDX-License-Identifier: MPL-2.0

use super::{claude_system_active_account_id, reconcile_provider_account_descriptors};
use crate::account_storage::ProviderAccountStorage;
use crate::config::{Config, managed_claude_account_dir, paths};
use crate::error::AppError;
use crate::model::{
    AppState, AuthState, ProviderAccountRuntimeState, ProviderHealth, ProviderId, STALE_THRESHOLD,
    UsageSnapshot,
};
use crate::providers::claude;
use crate::providers::interface::{
    BoxFuture, ProviderAccountAction, ProviderAccountDescriptor, ProviderAccountHandle,
    ProviderAccountStatus, ProviderAdapter, ProviderCapabilities, ProviderLoginKind,
};

pub(super) struct ClaudeAdapter;

impl ProviderAdapter for ClaudeAdapter {
    fn id(&self) -> ProviderId {
        ProviderId::Claude
    }

    fn login_kind(&self) -> ProviderLoginKind {
        ProviderLoginKind::Claude
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_background_status_refresh: false,
            requires_auth_prompt_on_auth_failure: false,
        }
    }

    fn account_label(
        &self,
        descriptor: &ProviderAccountDescriptor,
        account: &ProviderAccountRuntimeState,
    ) -> String {
        account
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.identity.email.as_deref())
            .filter(|email| !email.is_empty())
            .map_or_else(|| descriptor.label.clone(), str::to_string)
    }

    fn account_status(
        &self,
        account: &ProviderAccountRuntimeState,
    ) -> Option<ProviderAccountStatus> {
        claude_account_status(account)
    }

    fn discover_accounts(&self, config: &Config) -> Vec<ProviderAccountDescriptor> {
        claude::discover_accounts(config)
            .into_iter()
            .filter_map(|account| {
                config
                    .claude_managed_accounts
                    .iter()
                    .find(|managed| managed.id == account.id)
                    .cloned()
                    .map(|managed| ProviderAccountDescriptor {
                        provider: self.id(),
                        account_id: account.id,
                        label: account.label,
                        actions: vec![
                            ProviderAccountAction::Delete,
                            ProviderAccountAction::Reauthenticate,
                        ],
                        handle: ProviderAccountHandle::Claude(managed),
                    })
            })
            .collect()
    }

    fn sync_managed_accounts(&self, config: &mut Config) -> bool {
        claude::sync_managed_account_dirs(config)
    }

    fn delete_account(&self, account_id: &str, config: &mut Config) -> bool {
        if !config
            .claude_managed_accounts
            .iter()
            .any(|a| a.id == account_id)
        {
            return false;
        }
        if ProviderAccountStorage::new(paths().claude_accounts_dir)
            .delete_account(account_id)
            .is_err()
        {
            return false;
        }
        config
            .claude_managed_accounts
            .retain(|a| a.id != account_id);
        config
            .selected_claude_account_ids
            .retain(|id| id != account_id);
        true
    }

    fn reconcile_provider_accounts(&self, config: &Config, state: &mut AppState) {
        let accounts = self.discover_accounts(config);
        reconcile_provider_account_descriptors(self.id(), config, state, &accounts);
        if let Some(provider_state) = state.provider_mut(ProviderId::Claude) {
            provider_state.system_active_account_id = self.system_active_account_id(config);
        }
    }

    fn system_active_account_id(&self, config: &Config) -> Option<String> {
        claude_system_active_account_id(&config.claude_managed_accounts)
    }

    fn fetch_account<'a>(
        &self,
        handle: &'a ProviderAccountHandle,
        client: &'a reqwest::Client,
    ) -> BoxFuture<'a, crate::error::Result<UsageSnapshot, AppError>> {
        let provider = self.id();
        Box::pin(async move {
            match handle {
                ProviderAccountHandle::Claude(account) => {
                    claude::fetch(client, &account.id, managed_claude_account_dir(&account.id))
                        .await
                        .map_err(AppError::from)
                }
                _ => Err(AppError::InvalidAccountHandle { provider }),
            }
        })
    }
}

fn claude_account_status(account: &ProviderAccountRuntimeState) -> Option<ProviderAccountStatus> {
    use crate::providers::interface::ProviderAccountStatusKind;

    if account.auth_state == AuthState::ActionRequired {
        return Some(ProviderAccountStatus {
            kind: ProviderAccountStatusKind::Warning,
            badge_text: crate::fl!("badge-login-required"),
            tooltip_text: crate::fl!("badge-login-required-tooltip"),
            reauth_eligible: true,
            style_as_action_required: true,
        });
    }
    if account.health == ProviderHealth::Error {
        if account.snapshot.is_some() {
            return Some(claude_stale_status());
        }
        return Some(ProviderAccountStatus {
            kind: ProviderAccountStatusKind::Destructive,
            badge_text: crate::fl!("badge-error"),
            tooltip_text: crate::fl!("badge-error-tooltip"),
            reauth_eligible: false,
            style_as_action_required: true,
        });
    }
    if account.snapshot.is_some()
        && account
            .last_success_at
            .is_none_or(|updated| chrono::Utc::now() - updated >= STALE_THRESHOLD)
    {
        return Some(claude_stale_status());
    }
    None
}

fn claude_stale_status() -> ProviderAccountStatus {
    use crate::providers::interface::ProviderAccountStatusKind;

    ProviderAccountStatus {
        kind: ProviderAccountStatusKind::Warning,
        badge_text: crate::fl!("badge-stale"),
        tooltip_text: crate::fl!("badge-stale-tooltip"),
        reauth_eligible: false,
        style_as_action_required: false,
    }
}
