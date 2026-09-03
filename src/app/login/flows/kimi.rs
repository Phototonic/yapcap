use super::super::{LoginEventKind, LoginFlow, apply_login_success};
use crate::account_selection::select_account_after_login;
use crate::app::{AppModel, Config, Handle, Message, ProviderId, Task, kimi};
use crate::providers::kimi::login::{KimiLoginEvent, KimiLoginState, KimiLoginStatus};

pub(crate) struct KimiLoginFlow;

impl LoginFlow for KimiLoginFlow {
    type State = KimiLoginState;
    type Event = KimiLoginEvent;
    const PROVIDER: ProviderId = ProviderId::Kimi;

    fn state(app: &AppModel) -> &Option<Self::State> {
        &app.kimi_login
    }

    fn state_mut(app: &mut AppModel) -> &mut Option<Self::State> {
        &mut app.kimi_login
    }

    fn handle_mut(app: &mut AppModel) -> &mut Option<Handle> {
        &mut app.kimi_login_handle
    }

    fn is_running(state: &Self::State) -> bool {
        state.status == KimiLoginStatus::Editing
    }

    fn log_id(state: &Self::State) -> &str {
        &state.account_id
    }

    fn status_debug(state: &Self::State) -> String {
        format!("{:?}", state.status)
    }

    fn account_exists(config: &Config, account_id: &str) -> bool {
        config
            .kimi_managed_accounts
            .iter()
            .any(|account| account.id == account_id)
    }

    fn failed_state(error: String) -> Self::State {
        let mut state = KimiLoginState::new("failed".to_string());
        state.status = KimiLoginStatus::Failed;
        state.error = Some(error);
        state
    }

    fn prepare(config: Config) -> Result<(Self::State, cosmic::iced::Task<Self::Event>), String> {
        let _ = config;
        Ok((kimi::login::prepare(), cosmic::iced::Task::none()))
    }

    fn prepare_for_reauth(
        config: Config,
        account_id: &str,
    ) -> Result<(Self::State, cosmic::iced::Task<Self::Event>), String> {
        Ok((
            kimi::login::prepare_for_reauth(config, account_id)?,
            cosmic::iced::Task::none(),
        ))
    }

    fn wrap_event(event: Self::Event) -> Message {
        Message::LoginEvent(ProviderId::Kimi, Box::new(LoginEventKind::Kimi(event)))
    }

    fn on_event(app: &mut AppModel, event: Self::Event) -> Task<Message> {
        match event {
            KimiLoginEvent::ApiKeyChanged(api_key) => {
                if let Some(login) = app.kimi_login.as_mut() {
                    login.update_api_key(api_key);
                }
                Task::none()
            }
            KimiLoginEvent::ApiKeyVisibilityToggled => {
                if let Some(login) = app.kimi_login.as_mut() {
                    login.toggle_api_key_visibility();
                }
                Task::none()
            }
            KimiLoginEvent::LabelChanged(label) => {
                if let Some(login) = app.kimi_login.as_mut() {
                    login.update_label(label);
                }
                Task::none()
            }
            KimiLoginEvent::Saved => {
                let Some(login) = app.kimi_login.as_ref() else {
                    return Task::none();
                };
                let flow_id = login.account_id.clone();
                match login.save(&mut app.config) {
                    Ok(managed_account) => {
                        let account_id = managed_account.id.clone();
                        let selected_account_id = account_id.clone();
                        let task = apply_login_success(
                            app,
                            ProviderId::Kimi,
                            &flow_id,
                            account_id,
                            move |config| {
                                kimi::account::apply_login_account(config, managed_account);
                                select_account_after_login(
                                    config,
                                    ProviderId::Kimi,
                                    selected_account_id,
                                );
                            },
                        );
                        app.kimi_login_handle = None;
                        app.kimi_login = None;
                        task
                    }
                    Err(error) => {
                        if let Some(login) = app.kimi_login.as_mut() {
                            login.error = Some(error);
                            login.status = KimiLoginStatus::Failed;
                        }
                        Task::none()
                    }
                }
            }
        }
    }
}
