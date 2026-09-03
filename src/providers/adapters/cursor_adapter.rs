// SPDX-License-Identifier: MPL-2.0

use super::{
    cursor_system_active_account_id, reconcile_provider_account_descriptors,
    remove_managed_cursor_account,
};
use crate::config::Config;
use crate::error::AppError;
use crate::model::{
    AppState, AuthState, ProviderAccountRuntimeState, ProviderHealth, ProviderId, UsageSnapshot,
};
use crate::providers::cursor;
use crate::providers::interface::{
    BoxFuture, ProviderAccountAction, ProviderAccountAddAction, ProviderAccountDescriptor,
    ProviderAccountHandle, ProviderAccountStatus, ProviderAccountStatusKind, ProviderAdapter,
    ProviderCapabilities, ProviderLoginKind,
};
use chrono::Utc;
use std::collections::HashMap;
use tokio::task::JoinSet;

pub(super) struct CursorAdapter;

impl ProviderAdapter for CursorAdapter {
    fn id(&self) -> ProviderId {
        ProviderId::Cursor
    }

    fn login_kind(&self) -> ProviderLoginKind {
        ProviderLoginKind::Cursor
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_background_status_refresh: true,
            requires_auth_prompt_on_auth_failure: true,
        }
    }

    fn account_add_action(&self) -> ProviderAccountAddAction {
        ProviderAccountAddAction::Scan
    }

    fn reauthenticate_tooltip(&self) -> String {
        crate::fl!("cursor-account-reauth-tooltip")
    }

    fn account_status(
        &self,
        account: &ProviderAccountRuntimeState,
    ) -> Option<ProviderAccountStatus> {
        (account.auth_state == AuthState::ActionRequired).then(|| ProviderAccountStatus {
            kind: ProviderAccountStatusKind::Neutral,
            badge_text: crate::fl!("cursor-account-reauth-badge"),
            tooltip_text: crate::fl!("badge-reauth-tooltip"),
            reauth_eligible: true,
            style_as_action_required: true,
        })
    }

    fn discover_accounts(&self, config: &Config) -> Vec<ProviderAccountDescriptor> {
        cursor::discover_accounts(config)
            .into_iter()
            .map(cursor_account_descriptor)
            .collect()
    }

    fn sync_managed_accounts(&self, config: &mut Config) -> bool {
        cursor::sync_managed_accounts(config)
    }

    fn delete_account(&self, account_id: &str, config: &mut Config) -> bool {
        let Some(account) =
            cursor::find_managed_account(&config.cursor_managed_accounts, account_id).cloned()
        else {
            return false;
        };
        let email = account.email.clone();
        remove_managed_cursor_account(&account.id);
        config.cursor_managed_accounts.retain(|a| a.email != email);
        config
            .selected_cursor_account_ids
            .retain(|id| id != account_id);
        true
    }

    fn reconcile_provider_accounts(&self, config: &Config, state: &mut AppState) {
        let accounts = self.discover_accounts(config);
        reconcile_provider_account_descriptors(self.id(), config, state, &accounts);
        if let Some(provider_state) = state.provider_mut(ProviderId::Cursor) {
            provider_state.system_active_account_id = self.system_active_account_id(config);
        }
    }

    fn system_active_account_id(&self, config: &Config) -> Option<String> {
        cursor_system_active_account_id(&config.cursor_managed_accounts)
    }

    fn fetch_account<'a>(
        &self,
        handle: &'a ProviderAccountHandle,
        client: &'a reqwest::Client,
    ) -> BoxFuture<'a, crate::error::Result<UsageSnapshot, AppError>> {
        let provider = self.id();
        Box::pin(async move {
            match handle {
                ProviderAccountHandle::Cursor(account) => {
                    cursor::fetch(client, account).await.map_err(AppError::from)
                }
                _ => Err(AppError::InvalidAccountHandle { provider }),
            }
        })
    }

    fn refresh_account_statuses(
        &self,
        config: Config,
        previous_accounts: Vec<ProviderAccountRuntimeState>,
    ) -> BoxFuture<'static, Vec<ProviderAccountRuntimeState>> {
        Box::pin(refresh_cursor_account_statuses(config, previous_accounts))
    }
}

fn cursor_account_descriptor(
    account: crate::config::ManagedCursorAccountConfig,
) -> ProviderAccountDescriptor {
    let account_id = cursor::managed_account_id(&account.id);
    let label = account.email.clone();
    ProviderAccountDescriptor {
        provider: ProviderId::Cursor,
        account_id,
        label,
        actions: vec![ProviderAccountAction::Delete, ProviderAccountAction::Rescan],
        handle: ProviderAccountHandle::Cursor(account),
    }
}

async fn refresh_cursor_account_statuses(
    config: Config,
    previous_accounts: Vec<ProviderAccountRuntimeState>,
) -> Vec<ProviderAccountRuntimeState> {
    let client = crate::runtime::http_client();
    let accounts = cursor::discover_accounts(&config);
    let mut previous_by_id = previous_accounts
        .into_iter()
        .filter(|entry| entry.provider == ProviderId::Cursor)
        .map(|entry| (entry.account_id.clone(), entry))
        .collect::<HashMap<_, _>>();
    let mut tasks = JoinSet::new();

    for managed in accounts {
        let client = client.clone();
        let account_id = cursor::managed_account_id(&managed.id);
        let previous = previous_by_id.remove(&account_id);
        tasks.spawn(async move { refresh_cursor_account_status(client, managed, previous).await });
    }

    let mut refreshed = Vec::with_capacity(tasks.len());
    while let Some(result) = tasks.join_next().await {
        if let Ok(account) = result {
            refreshed.push(account);
        }
    }
    refreshed.sort_by(|left, right| left.account_id.cmp(&right.account_id));
    refreshed
}

async fn refresh_cursor_account_status(
    client: reqwest::Client,
    managed: crate::config::ManagedCursorAccountConfig,
    previous: Option<ProviderAccountRuntimeState>,
) -> ProviderAccountRuntimeState {
    let account_id = cursor::managed_account_id(&managed.id);
    let label = managed.email.clone();
    let mut account = previous.unwrap_or_else(|| {
        ProviderAccountRuntimeState::empty(ProviderId::Cursor, account_id, label)
    });

    if account.auth_state == AuthState::ActionRequired {
        return account;
    }

    match cursor::fetch(&client, &managed).await {
        Ok(snapshot) => {
            account.health = ProviderHealth::Ok;
            account.auth_state = AuthState::Ready;
            account.source_label = Some(snapshot.source.clone());
            account.last_success_at = Some(Utc::now());
            account.snapshot = Some(snapshot);
            account.error = None;
        }
        Err(error) => {
            let error = AppError::from(error);
            account.health = ProviderHealth::Error;
            account.auth_state = crate::runtime::classify_auth_state(&error);
            account.error = Some(error.user_message());
        }
    }

    account
}
