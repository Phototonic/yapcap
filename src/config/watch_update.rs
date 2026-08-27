// SPDX-License-Identifier: MPL-2.0

use super::Config;

impl Config {
    pub fn apply_watcher_update(&mut self, update: Self, keys: &[&str]) {
        if keys.is_empty() {
            *self = update;
            return;
        }
        for key in keys {
            if !self.apply_general_watcher_key(&update, key) {
                self.apply_account_watcher_key(&update, key);
            }
        }
    }

    fn apply_general_watcher_key(&mut self, update: &Self, key: &str) -> bool {
        match key {
            "refresh_interval_seconds" => {
                self.refresh_interval_seconds = update.refresh_interval_seconds;
            }
            "reset_time_format" => self.reset_time_format = update.reset_time_format,
            "usage_amount_format" => self.usage_amount_format = update.usage_amount_format,
            "panel_icon_style" => self.panel_icon_style = update.panel_icon_style,
            "selected_provider" => self.selected_provider = update.selected_provider,
            "provider_visibility_mode" => {
                self.provider_visibility_mode = update.provider_visibility_mode;
            }
            "codex_enabled" => self.codex_enabled = update.codex_enabled,
            "claude_enabled" => self.claude_enabled = update.claude_enabled,
            "cursor_enabled" => self.cursor_enabled = update.cursor_enabled,
            "gemini_enabled" => self.gemini_enabled = update.gemini_enabled,
            "copilot_enabled" => self.copilot_enabled = update.copilot_enabled,
            "minimax_enabled" => self.minimax_enabled = update.minimax_enabled,
            "kimi_enabled" => self.kimi_enabled = update.kimi_enabled,
            "opencode_go_enabled" => self.opencode_go_enabled = update.opencode_go_enabled,
            "show_all_accounts" => self.show_all_accounts = update.show_all_accounts.clone(),
            "log_level" => self.log_level.clone_from(&update.log_level),
            _ => return false,
        }
        true
    }

    fn apply_account_watcher_key(&mut self, update: &Self, key: &str) {
        match key {
            "selected_codex_account_ids" => {
                self.selected_codex_account_ids = update.selected_codex_account_ids.clone();
            }
            "codex_managed_accounts" => {
                self.codex_managed_accounts = update.codex_managed_accounts.clone();
            }
            "selected_claude_account_ids" => {
                self.selected_claude_account_ids = update.selected_claude_account_ids.clone();
            }
            "claude_managed_accounts" => {
                self.claude_managed_accounts = update.claude_managed_accounts.clone();
            }
            "selected_cursor_account_ids" => {
                self.selected_cursor_account_ids = update.selected_cursor_account_ids.clone();
            }
            "cursor_managed_accounts" => {
                self.cursor_managed_accounts = update.cursor_managed_accounts.clone();
            }
            "selected_gemini_account_ids" => {
                self.selected_gemini_account_ids = update.selected_gemini_account_ids.clone();
            }
            "gemini_managed_accounts" => {
                self.gemini_managed_accounts = update.gemini_managed_accounts.clone();
            }
            "selected_copilot_account_ids" => {
                self.selected_copilot_account_ids = update.selected_copilot_account_ids.clone();
            }
            "copilot_managed_accounts" => {
                self.copilot_managed_accounts = update.copilot_managed_accounts.clone();
            }
            "selected_minimax_account_ids" => {
                self.selected_minimax_account_ids = update.selected_minimax_account_ids.clone();
            }
            "minimax_managed_accounts" => {
                self.minimax_managed_accounts = update.minimax_managed_accounts.clone();
            }
            "selected_kimi_account_ids" => {
                self.selected_kimi_account_ids = update.selected_kimi_account_ids.clone();
            }
            "kimi_managed_accounts" => {
                self.kimi_managed_accounts = update.kimi_managed_accounts.clone();
            }
            "selected_opencode_go_account_ids" => {
                self.selected_opencode_go_account_ids =
                    update.selected_opencode_go_account_ids.clone();
            }
            "opencode_go_managed_accounts" => {
                self.opencode_go_managed_accounts = update.opencode_go_managed_accounts.clone();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ManagedKimiAccountConfig, ManagedMinimaxAccountConfig};
    use chrono::Utc;

    #[test]
    fn applies_minimax_watcher_keys_without_replacing_unrelated_configuration() {
        let mut config = Config::default();
        let mut update = Config {
            codex_enabled: false,
            minimax_enabled: false,
            selected_minimax_account_ids: vec!["minimax-2".to_string()],
            minimax_managed_accounts: vec![minimax_account("minimax-2")],
            ..Config::default()
        };
        update.minimax_managed_accounts[0].label = "Second Minimax".to_string();

        config.apply_watcher_update(
            update,
            &[
                "minimax_enabled",
                "selected_minimax_account_ids",
                "minimax_managed_accounts",
            ],
        );

        assert!(!config.minimax_enabled);
        assert_eq!(config.selected_minimax_account_ids, ["minimax-2"]);
        assert_eq!(config.minimax_managed_accounts[0].label, "Second Minimax");
        assert!(config.codex_enabled);
    }

    #[test]
    fn applies_kimi_watcher_keys_without_replacing_unrelated_configuration() {
        let mut config = Config::default();
        let mut update = Config {
            codex_enabled: false,
            kimi_enabled: false,
            selected_kimi_account_ids: vec!["kimi-2".to_string()],
            kimi_managed_accounts: vec![kimi_account("kimi-2")],
            ..Config::default()
        };
        update.kimi_managed_accounts[0].label = "Second Kimi".to_string();

        config.apply_watcher_update(
            update,
            &[
                "kimi_enabled",
                "selected_kimi_account_ids",
                "kimi_managed_accounts",
            ],
        );

        assert!(!config.kimi_enabled);
        assert_eq!(config.selected_kimi_account_ids, ["kimi-2"]);
        assert_eq!(config.kimi_managed_accounts[0].label, "Second Kimi");
        assert!(config.codex_enabled);
    }

    #[test]
    fn applies_opencode_go_watcher_keys_without_replacing_unrelated_configuration() {
        let mut config = Config::default();
        let mut update = Config {
            codex_enabled: false,
            opencode_go_enabled: false,
            selected_opencode_go_account_ids: vec!["go-2".to_string()],
            opencode_go_managed_accounts: vec![opencode_go_account("go-2")],
            ..Config::default()
        };
        update.opencode_go_managed_accounts[0].label = "Second OpenCode Go".to_string();

        config.apply_watcher_update(
            update,
            &[
                "opencode_go_enabled",
                "selected_opencode_go_account_ids",
                "opencode_go_managed_accounts",
            ],
        );

        assert!(!config.opencode_go_enabled);
        assert_eq!(config.selected_opencode_go_account_ids, ["go-2"]);
        assert_eq!(
            config.opencode_go_managed_accounts[0].label,
            "Second OpenCode Go"
        );
        assert!(config.codex_enabled);
    }

    #[test]
    fn ignores_unknown_keys() {
        let mut config = Config::default();
        let update = Config {
            kimi_enabled: false,
            ..Config::default()
        };

        config.apply_watcher_update(update, &["unknown"]);

        assert!(config.kimi_enabled);
    }

    #[test]
    fn empty_keys_replace_the_entire_configuration() {
        let mut config = Config::default();
        let update = Config {
            kimi_enabled: false,
            ..Config::default()
        };

        config.apply_watcher_update(update, &[]);

        assert!(!config.kimi_enabled);
    }

    fn minimax_account(id: &str) -> ManagedMinimaxAccountConfig {
        let now = Utc::now();
        ManagedMinimaxAccountConfig {
            id: id.to_string(),
            label: id.to_string(),
            api_key_source: "stored".to_string(),
            created_at: now,
            updated_at: now,
            last_authenticated_at: None,
        }
    }

    fn kimi_account(id: &str) -> ManagedKimiAccountConfig {
        let now = Utc::now();
        ManagedKimiAccountConfig {
            id: id.to_string(),
            label: id.to_string(),
            api_key_source: "stored".to_string(),
            created_at: now,
            updated_at: now,
            last_authenticated_at: None,
        }
    }

    fn opencode_go_account(id: &str) -> crate::config::ManagedOpenCodeGoAccountConfig {
        let now = Utc::now();
        crate::config::ManagedOpenCodeGoAccountConfig {
            id: id.to_string(),
            label: id.to_string(),
            api_key_source: "stored".to_string(),
            created_at: now,
            updated_at: now,
            last_authenticated_at: None,
        }
    }
}
