// SPDX-License-Identifier: MPL-2.0

use crate::account_storage::validated_account_dir;
use crate::config::{Config, ManagedKimiAccountConfig, paths};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KimiAccount {
    pub id: String,
    pub label: String,
    pub config_dir: PathBuf,
}

pub fn discover_accounts(config: &Config) -> Vec<KimiAccount> {
    config
        .kimi_managed_accounts
        .iter()
        .filter_map(|managed| {
            validated_account_dir(&paths().kimi_accounts_dir, &managed.id)
                .ok()
                .map(|config_dir| KimiAccount {
                    id: managed.id.clone(),
                    label: managed.label.clone(),
                    config_dir,
                })
        })
        .collect()
}

pub fn apply_login_account(config: &mut Config, account: ManagedKimiAccountConfig) {
    let account_id = account.id.clone();
    config
        .kimi_managed_accounts
        .retain(|existing| existing.id != account_id);
    config.kimi_managed_accounts.push(account);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn apply_login_account_replaces_matching_account() {
        let mut config = Config {
            kimi_managed_accounts: vec![account("kimi-1", "First")],
            ..Config::default()
        };

        apply_login_account(&mut config, account("kimi-1", "Updated"));

        assert_eq!(config.kimi_managed_accounts.len(), 1);
        assert_eq!(config.kimi_managed_accounts[0].label, "Updated");
    }

    fn account(id: &str, label: &str) -> ManagedKimiAccountConfig {
        let now = Utc::now();
        ManagedKimiAccountConfig {
            id: id.to_string(),
            label: label.to_string(),
            api_key_source: "stored".to_string(),
            created_at: now,
            updated_at: now,
            last_authenticated_at: Some(now),
        }
    }
}
