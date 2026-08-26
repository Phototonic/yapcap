// SPDX-License-Identifier: MPL-2.0

use crate::account_storage::validated_account_dir;
use crate::config::{Config, ManagedMinimaxAccountConfig, paths};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimaxAccount {
    pub id: String,
    pub label: String,
    pub config_dir: PathBuf,
}

pub fn discover_accounts(config: &Config) -> Vec<MinimaxAccount> {
    let mut accounts = Vec::new();
    for managed in &config.minimax_managed_accounts {
        let Ok(config_dir) = validated_account_dir(&paths().minimax_accounts_dir, &managed.id)
        else {
            continue;
        };
        let discovered = MinimaxAccount {
            id: managed.id.clone(),
            label: managed.label.clone(),
            config_dir,
        };
        accounts.push(discovered);
    }
    accounts
}

pub fn apply_login_account(config: &mut Config, account: ManagedMinimaxAccountConfig) {
    let account_id = account.id.clone();
    config
        .minimax_managed_accounts
        .retain(|existing| existing.id != account_id);
    config.minimax_managed_accounts.push(account);
}
