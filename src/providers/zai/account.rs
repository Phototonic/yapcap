// SPDX-License-Identifier: MPL-2.0

use crate::account_storage::validated_account_dir;
use crate::config::{Config, ManagedZaiAccountConfig, paths};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZaiAccount {
    pub id: String,
    pub label: String,
    pub config_dir: PathBuf,
}

pub fn discover_accounts(config: &Config) -> Vec<ZaiAccount> {
    config
        .zai_managed_accounts
        .iter()
        .filter_map(|managed| {
            let config_dir = validated_account_dir(&paths().zai_accounts_dir, &managed.id).ok()?;
            Some(ZaiAccount {
                id: managed.id.clone(),
                label: managed.label.clone(),
                config_dir,
            })
        })
        .collect()
}

pub fn apply_login_account(config: &mut Config, account: ManagedZaiAccountConfig) {
    let account_id = account.id.clone();
    config
        .zai_managed_accounts
        .retain(|existing| existing.id != account_id);
    config.zai_managed_accounts.push(account);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn account(id: &str) -> ManagedZaiAccountConfig {
        let now = Utc::now();
        ManagedZaiAccountConfig {
            id: id.to_string(),
            label: id.to_string(),
            api_key_source: "stored".to_string(),
            created_at: now,
            updated_at: now,
            last_authenticated_at: Some(now),
        }
    }

    #[test]
    fn applies_login_by_replacing_the_same_account_id() {
        let mut config = Config {
            zai_managed_accounts: vec![account("zai-1")],
            ..Config::default()
        };
        let mut replacement = account("zai-1");
        replacement.label = "Replacement".to_string();

        apply_login_account(&mut config, replacement);

        assert_eq!(config.zai_managed_accounts.len(), 1);
        assert_eq!(config.zai_managed_accounts[0].label, "Replacement");
    }

    #[test]
    fn discovery_rejects_path_escaping_account_ids() {
        let config = Config {
            zai_managed_accounts: vec![account("zai-1"), account("../outside")],
            ..Config::default()
        };

        let discovered = discover_accounts(&config);

        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].id, "zai-1");
    }
}
