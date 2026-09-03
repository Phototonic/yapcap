use super::super::{LoginEventKind, LoginFlow, apply_login_success};
use crate::account_selection::select_account_after_login;
use crate::app::{AppModel, Config, Handle, Message, ProviderId, Task, minimax};
use crate::providers::minimax::login::{MinimaxLoginEvent, MinimaxLoginState, MinimaxLoginStatus};

pub(crate) struct MinimaxLoginFlow;

impl LoginFlow for MinimaxLoginFlow {
    type State = MinimaxLoginState;
    type Event = MinimaxLoginEvent;
    const PROVIDER: ProviderId = ProviderId::Minimax;

    fn state(app: &AppModel) -> &Option<Self::State> {
        &app.minimax_login
    }
    fn state_mut(app: &mut AppModel) -> &mut Option<Self::State> {
        &mut app.minimax_login
    }
    fn handle_mut(app: &mut AppModel) -> &mut Option<Handle> {
        &mut app.minimax_login_handle
    }
    fn is_running(state: &Self::State) -> bool {
        state.status == MinimaxLoginStatus::Editing
    }
    fn log_id(state: &Self::State) -> &str {
        &state.account_id
    }
    fn status_debug(state: &Self::State) -> String {
        format!("{:?}", state.status)
    }
    fn account_exists(config: &Config, account_id: &str) -> bool {
        config
            .minimax_managed_accounts
            .iter()
            .any(|a| a.id == account_id)
    }
    fn failed_state(error: String) -> Self::State {
        MinimaxLoginState::failed(error)
    }
    fn prepare(config: Config) -> Result<(Self::State, cosmic::iced::Task<Self::Event>), String> {
        let _ = config;
        Ok((minimax::login::prepare(), cosmic::iced::Task::none()))
    }
    fn prepare_for_reauth(
        config: Config,
        account_id: &str,
    ) -> Result<(Self::State, cosmic::iced::Task<Self::Event>), String> {
        Ok((
            minimax::login::prepare_for_reauth(config, account_id)?,
            cosmic::iced::Task::none(),
        ))
    }
    fn wrap_event(event: Self::Event) -> Message {
        Message::LoginEvent(
            ProviderId::Minimax,
            Box::new(LoginEventKind::Minimax(event)),
        )
    }
    fn on_event(app: &mut AppModel, event: Self::Event) -> Task<Message> {
        match event {
            MinimaxLoginEvent::ApiKeyChanged(api_key) => {
                if let Some(login) = app.minimax_login.as_mut() {
                    login.update_api_key(api_key);
                }
                Task::none()
            }
            MinimaxLoginEvent::ApiKeyVisibilityToggled => {
                if let Some(login) = app.minimax_login.as_mut() {
                    login.toggle_api_key_visibility();
                }
                Task::none()
            }
            MinimaxLoginEvent::LabelChanged(label) => {
                if let Some(login) = app.minimax_login.as_mut() {
                    login.update_label(label);
                }
                Task::none()
            }
            MinimaxLoginEvent::Saved => {
                let Some(login) = app.minimax_login.as_mut() else {
                    return Task::none();
                };
                let flow_id = login.account_id.clone();
                match minimax::login::save(&app.config, login) {
                    Ok(managed_account) => {
                        let account_id = managed_account.id.clone();
                        let selected_account_id = account_id.clone();
                        let task = apply_login_success(
                            app,
                            ProviderId::Minimax,
                            &flow_id,
                            account_id,
                            move |config| {
                                minimax::account::apply_login_account(config, managed_account);
                                select_account_after_login(
                                    config,
                                    ProviderId::Minimax,
                                    selected_account_id,
                                );
                            },
                        );
                        app.minimax_login_handle = None;
                        app.minimax_login = None;
                        task
                    }
                    Err(_) => Task::none(),
                }
            }
        }
    }
}
