// SPDX-License-Identifier: MPL-2.0

use super::reconcile_provider_account_descriptors;
use crate::account_storage::ProviderAccountStorage;
use crate::config::{Config, paths};
use crate::error::AppError;
use crate::model::{AppState, ProviderId, UsageSnapshot};
use crate::providers::interface::{
    BoxFuture, ProviderAccountAction, ProviderAccountDescriptor, ProviderAccountHandle,
    ProviderAdapter, ProviderCapabilities, ProviderLoginKind,
};
use crate::providers::zai;

pub(super) struct ZaiAdapter;

impl ProviderAdapter for ZaiAdapter {
    fn id(&self) -> ProviderId {
        ProviderId::Zai
    }

    fn login_kind(&self) -> ProviderLoginKind {
        ProviderLoginKind::Zai
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_background_status_refresh: false,
            requires_auth_prompt_on_auth_failure: false,
        }
    }

    fn selection_required_message(&self) -> Option<String> {
        Some(crate::fl!("badge-select-required"))
    }

    fn discover_accounts(&self, config: &Config) -> Vec<ProviderAccountDescriptor> {
        zai::discover_accounts(config)
            .into_iter()
            .filter_map(|account| {
                config
                    .zai_managed_accounts
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
                        handle: ProviderAccountHandle::Zai(managed),
                    })
            })
            .collect()
    }

    fn sync_managed_accounts(&self, config: &mut Config) -> bool {
        zai::sync_managed_accounts(config)
    }

    fn delete_account(&self, account_id: &str, config: &mut Config) -> bool {
        if !config
            .zai_managed_accounts
            .iter()
            .any(|account| account.id == account_id)
        {
            return false;
        }
        if ProviderAccountStorage::new(paths().zai_accounts_dir)
            .delete_account(account_id)
            .is_err()
        {
            return false;
        }
        config
            .zai_managed_accounts
            .retain(|account| account.id != account_id);
        config
            .selected_zai_account_ids
            .retain(|id| id != account_id);
        true
    }

    fn reconcile_provider_accounts(&self, config: &Config, state: &mut AppState) {
        let accounts = self.discover_accounts(config);
        reconcile_provider_account_descriptors(self.id(), config, state, &accounts);
        if let Some(provider_state) = state.provider_mut(self.id()) {
            provider_state.system_active_account_id = None;
        }
    }

    fn fetch_account<'a>(
        &self,
        handle: &'a ProviderAccountHandle,
        client: &'a reqwest::Client,
    ) -> BoxFuture<'a, crate::error::Result<UsageSnapshot, AppError>> {
        let provider = self.id();
        Box::pin(async move {
            match handle {
                ProviderAccountHandle::Zai(account) => {
                    zai::fetch(client, account).await.map_err(AppError::from)
                }
                _ => Err(AppError::InvalidAccountHandle { provider }),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ManagedZaiAccountConfig, paths};
    use chrono::Utc;
    use tempfile::tempdir;

    fn account(id: &str) -> ManagedZaiAccountConfig {
        let now = Utc::now();
        ManagedZaiAccountConfig {
            id: id.to_string(),
            label: "Z.AI account".to_string(),
            api_key_source: "stored".to_string(),
            created_at: now,
            updated_at: now,
            last_authenticated_at: None,
        }
    }

    #[test]
    fn exposes_managed_key_account_contract() {
        let adapter = ZaiAdapter;
        assert_eq!(adapter.id(), ProviderId::Zai);
        assert_eq!(adapter.login_kind(), ProviderLoginKind::Zai);
        assert_eq!(
            adapter.capabilities(),
            ProviderCapabilities {
                supports_background_status_refresh: false,
                requires_auth_prompt_on_auth_failure: false,
            }
        );
        assert!(!adapter.supports_opencode_import());
        assert_eq!(adapter.system_active_account_id(&Config::default()), None);
    }

    #[test]
    fn discovers_zai_accounts_with_only_delete_and_reauthenticate() {
        let adapter = ZaiAdapter;
        let config = Config {
            zai_managed_accounts: vec![account("zai-1")],
            ..Config::default()
        };

        let descriptors = adapter.discover_accounts(&config);

        assert_eq!(descriptors.len(), 1);
        assert_eq!(descriptors[0].provider, ProviderId::Zai);
        assert_eq!(
            descriptors[0].actions,
            vec![
                ProviderAccountAction::Delete,
                ProviderAccountAction::Reauthenticate
            ]
        );
        assert!(matches!(
            descriptors[0].handle,
            ProviderAccountHandle::Zai(_)
        ));
    }

    #[test]
    fn deleting_account_removes_storage_config_and_selection() {
        let mut env = crate::test_support::test_env();
        let root = tempdir().unwrap();
        env.set("XDG_STATE_HOME", root.path());
        let id = "zai-delete";
        let storage = ProviderAccountStorage::new(paths().zai_accounts_dir.clone());
        storage.write_text_file(id, "api_key.txt", "key").unwrap();
        let mut config = Config {
            zai_managed_accounts: vec![account(id)],
            selected_zai_account_ids: vec![id.to_string()],
            ..Config::default()
        };

        assert!(ZaiAdapter.delete_account(id, &mut config));
        assert!(config.zai_managed_accounts.is_empty());
        assert!(config.selected_zai_account_ids.is_empty());
        assert!(!paths().zai_accounts_dir.join(id).exists());
    }
}
