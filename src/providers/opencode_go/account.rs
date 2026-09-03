// SPDX-License-Identifier: MPL-2.0

use crate::account_storage::{ProviderAccountStorage, validated_account_dir};
use crate::config::{Config, ManagedOpenCodeGoAccountConfig, paths};
use crate::providers::opencode_go::storage::API_KEY_FILE;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenCodeGoAccount {
    pub id: String,
    pub label: String,
    pub config_dir: PathBuf,
}

pub fn discover_accounts(config: &Config) -> Vec<OpenCodeGoAccount> {
    config
        .opencode_go_managed_accounts
        .iter()
        .filter_map(|managed| {
            validated_account_dir(&paths().opencode_go_accounts_dir, &managed.id)
                .ok()
                .map(|config_dir| OpenCodeGoAccount {
                    id: managed.id.clone(),
                    label: managed.label.clone(),
                    config_dir,
                })
        })
        .collect()
}

pub fn apply_login_account(config: &mut Config, account: ManagedOpenCodeGoAccountConfig) {
    let account_id = account.id.clone();
    config
        .opencode_go_managed_accounts
        .retain(|existing| existing.id != account_id);
    config.opencode_go_managed_accounts.push(account);
}

pub(crate) fn system_active_account_id(
    managed_accounts: &[ManagedOpenCodeGoAccountConfig],
    storage: &ProviderAccountStorage,
    api_key: &str,
) -> Option<String> {
    managed_accounts.iter().find_map(|account| {
        storage
            .read_text_file(&account.id, API_KEY_FILE)
            .ok()
            .filter(|stored_key| stored_key == api_key)
            .map(|_| account.id.clone())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::opencode_go::storage::write_api_key_at;
    use chrono::Utc;
    use tempfile::tempdir;

    #[test]
    fn apply_login_account_replaces_matching_account() {
        let mut config = Config {
            opencode_go_managed_accounts: vec![account("go-1", "First")],
            ..Config::default()
        };
        apply_login_account(&mut config, account("go-1", "Updated"));
        assert_eq!(config.opencode_go_managed_accounts.len(), 1);
        assert_eq!(config.opencode_go_managed_accounts[0].label, "Updated");
    }

    #[test]
    fn system_active_account_id_matches_stored_api_key() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("accounts");
        let storage = ProviderAccountStorage::new(&root);
        write_api_key_at(&root, "go-1", "test-key").unwrap();
        let managed = vec![account("go-1", "First")];

        assert_eq!(
            system_active_account_id(&managed, &storage, "test-key"),
            Some("go-1".to_string())
        );
    }

    #[test]
    fn system_active_account_id_ignores_non_matching_api_key() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("accounts");
        let storage = ProviderAccountStorage::new(&root);
        write_api_key_at(&root, "go-1", "stored-key").unwrap();
        let managed = vec![account("go-1", "First")];

        assert_eq!(
            system_active_account_id(&managed, &storage, "other-key"),
            None
        );
    }

    fn account(id: &str, label: &str) -> ManagedOpenCodeGoAccountConfig {
        let now = Utc::now();
        ManagedOpenCodeGoAccountConfig {
            id: id.to_string(),
            label: label.to_string(),
            api_key_source: "stored".to_string(),
            created_at: now,
            updated_at: now,
            last_authenticated_at: Some(now),
        }
    }
}
