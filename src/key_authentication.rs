// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct KeyAuthenticationState {
    pub account_id: String,
    pub label: String,
    pub status: KeyAuthenticationStatus,
    pub api_key: String,
    pub api_key_from_opencode: bool,
    pub api_key_visible: bool,
    pub error: Option<String>,
    reauth_target: Option<KeyAuthenticationTarget>,
}

#[derive(Debug, Clone)]
pub(crate) struct KeyAuthenticationTarget {
    pub id: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAuthenticationStatus {
    Editing,
    Failed,
}

#[derive(Debug, Clone)]
pub enum KeyAuthenticationEvent {
    ApiKeyChanged(String),
    ApiKeyVisibilityToggled,
    LabelChanged(String),
    Saved,
}

pub(crate) trait KeyAuthenticationAdapter {
    type Account;

    fn new_account_id() -> String;
    fn find_account(config: &Config, account_id: &str) -> Result<KeyAuthenticationTarget, String>;
    fn discover_api_key() -> Option<String>;
    fn empty_key_error() -> String;
    fn build_account(
        account_id: String,
        label: String,
        created_at: DateTime<Utc>,
        authenticated_at: DateTime<Utc>,
    ) -> Self::Account;
    fn persist(account: &Self::Account, api_key: &str) -> Result<(), String>;
}

impl KeyAuthenticationState {
    pub fn new(account_id: String) -> Self {
        Self {
            account_id,
            label: String::new(),
            status: KeyAuthenticationStatus::Editing,
            api_key: String::new(),
            api_key_from_opencode: false,
            api_key_visible: false,
            error: None,
            reauth_target: None,
        }
    }

    pub fn failed(error: String) -> Self {
        let mut state = Self::new("failed".to_string());
        state.status = KeyAuthenticationStatus::Failed;
        state.error = Some(error);
        state
    }

    pub fn update_label(&mut self, label: String) {
        self.label = label;
    }

    pub fn update_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
        self.api_key_from_opencode = false;
        self.error = None;
    }

    pub fn toggle_api_key_visibility(&mut self) {
        self.api_key_visible = !self.api_key_visible;
    }

    pub(crate) fn save<A: KeyAuthenticationAdapter>(&mut self) -> Result<A::Account, String> {
        self.error = None;
        if self.api_key.trim().is_empty() {
            return self.fail_save(A::empty_key_error());
        }

        let authenticated_at = Utc::now();
        let (account_id, label, created_at) = self.reauth_target.as_ref().map_or_else(
            || {
                (
                    self.account_id.clone(),
                    self.label.clone(),
                    authenticated_at,
                )
            },
            |target| (target.id.clone(), target.label.clone(), target.created_at),
        );
        let account = A::build_account(account_id, label, created_at, authenticated_at);
        if let Err(error) = A::persist(&account, &self.api_key) {
            return self.fail_save(error);
        }
        Ok(account)
    }

    fn fail_save<T>(&mut self, error: String) -> Result<T, String> {
        self.status = KeyAuthenticationStatus::Editing;
        self.error = Some(error.clone());
        Err(error)
    }
}

pub(crate) fn prepare<A: KeyAuthenticationAdapter>() -> KeyAuthenticationState {
    let mut state = KeyAuthenticationState::new(A::new_account_id());
    prefill::<A>(&mut state);
    state
}

pub(crate) fn prepare_for_reauth<A: KeyAuthenticationAdapter>(
    config: &Config,
    account_id: &str,
) -> Result<KeyAuthenticationState, String> {
    let target = A::find_account(config, account_id)?;
    let mut state = KeyAuthenticationState::new(target.id.clone());
    state.label = target.label.clone();
    state.reauth_target = Some(target);
    prefill::<A>(&mut state);
    Ok(state)
}

fn prefill<A: KeyAuthenticationAdapter>(state: &mut KeyAuthenticationState) {
    let discovered_api_key = A::discover_api_key();
    state.api_key_from_opencode = discovered_api_key.is_some();
    state.api_key = discovered_api_key.unwrap_or_default();
}
