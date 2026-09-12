use super::super::{LoginEventKind, LoginFlow, apply_login_success_checked};
use crate::account_selection::select_account_after_login;
use crate::app::{AppModel, Config, Handle, Message, ProviderId, Task};
use crate::providers::zai;
use crate::providers::zai::login::{ZaiLoginEvent, ZaiLoginState, ZaiLoginStatus};

pub(crate) struct ZaiLoginFlow;

impl LoginFlow for ZaiLoginFlow {
    type State = ZaiLoginState;
    type Event = ZaiLoginEvent;
    const PROVIDER: ProviderId = ProviderId::Zai;

    fn state(app: &AppModel) -> &Option<Self::State> {
        &app.zai_login
    }

    fn state_mut(app: &mut AppModel) -> &mut Option<Self::State> {
        &mut app.zai_login
    }

    fn handle_mut(app: &mut AppModel) -> &mut Option<Handle> {
        &mut app.zai_login_handle
    }

    fn is_running(state: &Self::State) -> bool {
        state.status == ZaiLoginStatus::Editing
    }

    fn log_id(state: &Self::State) -> &str {
        &state.account_id
    }

    fn status_debug(state: &Self::State) -> String {
        format!("{:?}", state.status)
    }

    fn account_exists(config: &Config, account_id: &str) -> bool {
        config
            .zai_managed_accounts
            .iter()
            .any(|account| account.id == account_id)
    }

    fn failed_state(error: String) -> Self::State {
        ZaiLoginState::failed(error)
    }

    fn prepare(config: Config) -> Result<(Self::State, cosmic::iced::Task<Self::Event>), String> {
        let _ = config;
        Ok((zai::login::prepare(), cosmic::iced::Task::none()))
    }

    fn prepare_for_reauth(
        config: Config,
        account_id: &str,
    ) -> Result<(Self::State, cosmic::iced::Task<Self::Event>), String> {
        Ok((
            zai::login::prepare_for_reauth(config, account_id)?,
            cosmic::iced::Task::none(),
        ))
    }

    fn wrap_event(event: Self::Event) -> Message {
        Message::LoginEvent(ProviderId::Zai, Box::new(LoginEventKind::Zai(event)))
    }

    fn on_event(app: &mut AppModel, event: Self::Event) -> Task<Message> {
        match event {
            ZaiLoginEvent::ApiKeyChanged(api_key) => {
                if let Some(login) = app.zai_login.as_mut() {
                    login.update_api_key(api_key);
                }
                Task::none()
            }
            ZaiLoginEvent::ApiKeyVisibilityToggled => {
                if let Some(login) = app.zai_login.as_mut() {
                    login.toggle_api_key_visibility();
                }
                Task::none()
            }
            ZaiLoginEvent::LabelChanged(label) => {
                if let Some(login) = app.zai_login.as_mut() {
                    login.update_label(label);
                }
                Task::none()
            }
            ZaiLoginEvent::Saved => {
                let Some(login) = app.zai_login.as_mut() else {
                    return Task::none();
                };
                let flow_id = login.account_id.clone();
                let is_reauthentication = app
                    .config
                    .zai_managed_accounts
                    .iter()
                    .any(|account| account.id == flow_id);
                match zai::login::save(&app.config, login) {
                    Ok(managed_account) => {
                        let account_id = managed_account.id.clone();
                        let selected_account_id = account_id.clone();
                        let result = apply_login_success_checked(
                            app,
                            ProviderId::Zai,
                            &flow_id,
                            account_id.clone(),
                            move |config| {
                                zai::account::apply_login_account(config, managed_account);
                                select_account_after_login(
                                    config,
                                    ProviderId::Zai,
                                    selected_account_id,
                                );
                            },
                        );
                        match result {
                            Ok(task) => {
                                app.zai_login_handle = None;
                                app.zai_login = None;
                                task
                            }
                            Err(()) => {
                                cleanup_new_account_storage(&account_id, is_reauthentication);
                                if let Some(login) = app.zai_login.as_mut() {
                                    login.status = ZaiLoginStatus::Editing;
                                    login.error = Some(
                                        "Failed to save Z.AI account configuration".to_string(),
                                    );
                                }
                                Task::none()
                            }
                        }
                    }
                    Err(_) => Task::none(),
                }
            }
        }
    }
}

fn cleanup_new_account_storage(account_id: &str, is_reauthentication: bool) {
    if is_reauthentication {
        return;
    }
    if let Err(error) = zai::storage::delete_account(account_id) {
        tracing::error!(
            account_id,
            error = %error,
            "failed to clean up new Z.AI account storage"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::cleanup_new_account_storage;
    use crate::providers::zai::storage;

    #[test]
    fn config_failure_removes_new_account_storage() {
        let _env = crate::test_support::test_env();
        let account_id = "zai-config-failure-new";
        storage::write_api_key(account_id, "new-key").unwrap();

        cleanup_new_account_storage(account_id, false);

        assert!(storage::load_api_key(account_id).is_err());
    }

    #[test]
    fn config_failure_does_not_replace_reauthentication_candidate() {
        let _env = crate::test_support::test_env();
        let account_id = "zai-config-failure-existing";
        storage::write_api_key(account_id, "candidate-key").unwrap();

        cleanup_new_account_storage(account_id, true);

        assert_eq!(storage::load_api_key(account_id).unwrap(), "candidate-key");
    }
}
