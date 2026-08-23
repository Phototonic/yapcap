// SPDX-License-Identifier: MPL-2.0

use crate::config::{Config, ManagedKimiAccountConfig, managed_kimi_account_dir};
use std::fs;
use std::path::{Path, PathBuf};

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
        .map(|managed| KimiAccount {
            id: managed.id.clone(),
            label: managed.label.clone(),
            config_dir: managed_kimi_account_dir(&managed.id),
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

pub fn remove_managed_config_dir(config_dir: &Path) {
    let root = managed_kimi_account_dir("");
    let Some(root) = root.parent() else {
        return;
    };
    let Ok(root) = root.canonicalize() else {
        return;
    };
    let Ok(metadata) = fs::symlink_metadata(config_dir) else {
        return;
    };
    if metadata.file_type().is_symlink() {
        tracing::warn!(path = %config_dir.display(), "refusing to delete symlinked Kimi account config dir");
        return;
    }
    let Ok(config_dir) = config_dir.canonicalize() else {
        return;
    };
    if !config_dir.starts_with(&root) {
        tracing::warn!(path = %config_dir.display(), root = %root.display(), "refusing to delete Kimi account outside managed root");
        return;
    }
    if let Err(error) = fs::remove_dir_all(&config_dir) {
        tracing::warn!(path = %config_dir.display(), error = %error, "failed to delete Kimi account config dir");
    }
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
