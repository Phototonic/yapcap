use super::super::{LoginEventKind, LoginFlow, apply_login_success};
use crate::account_selection::select_account_after_login;
use crate::app::{AppModel, Config, Handle, Message, ProviderId, Task};
use crate::providers::opencode_go;
use crate::providers::opencode_go::login::{
    OpenCodeGoLoginEvent, OpenCodeGoLoginState, OpenCodeGoLoginStatus,
};

pub(crate) struct OpenCodeGoLoginFlow;

impl LoginFlow for OpenCodeGoLoginFlow {
    type State = OpenCodeGoLoginState;
    type Event = OpenCodeGoLoginEvent;
    const PROVIDER: ProviderId = ProviderId::OpenCodeGo;

    fn state(app: &AppModel) -> &Option<Self::State> {
        &app.opencode_go_login
    }

    fn state_mut(app: &mut AppModel) -> &mut Option<Self::State> {
        &mut app.opencode_go_login
    }

    fn handle_mut(app: &mut AppModel) -> &mut Option<Handle> {
        &mut app.opencode_go_login_handle
    }

    fn is_running(state: &Self::State) -> bool {
        state.status == OpenCodeGoLoginStatus::Editing
    }

    fn log_id(state: &Self::State) -> &str {
        &state.account_id
    }

    fn status_debug(state: &Self::State) -> String {
        format!("{:?}", state.status)
    }

    fn account_exists(config: &Config, account_id: &str) -> bool {
        config
            .opencode_go_managed_accounts
            .iter()
            .any(|account| account.id == account_id)
    }

    fn failed_state(error: String) -> Self::State {
        OpenCodeGoLoginState::failed(error)
    }

    fn prepare(config: Config) -> Result<(Self::State, cosmic::iced::Task<Self::Event>), String> {
        let _ = config;
        Ok((opencode_go::login::prepare(), cosmic::iced::Task::none()))
    }

    fn prepare_for_reauth(
        config: Config,
        account_id: &str,
    ) -> Result<(Self::State, cosmic::iced::Task<Self::Event>), String> {
        Ok((
            opencode_go::login::prepare_for_reauth(config, account_id)?,
            cosmic::iced::Task::none(),
        ))
    }

    fn wrap_event(event: Self::Event) -> Message {
        Message::LoginEvent(
            ProviderId::OpenCodeGo,
            Box::new(LoginEventKind::OpenCodeGo(event)),
        )
    }

    fn on_event(app: &mut AppModel, event: Self::Event) -> Task<Message> {
        match event {
            OpenCodeGoLoginEvent::ApiKeyChanged(api_key) => {
                if let Some(login) = app.opencode_go_login.as_mut() {
                    login.update_api_key(api_key);
                }
                Task::none()
            }
            OpenCodeGoLoginEvent::ApiKeyVisibilityToggled => {
                if let Some(login) = app.opencode_go_login.as_mut() {
                    login.toggle_api_key_visibility();
                }
                Task::none()
            }
            OpenCodeGoLoginEvent::LabelChanged(label) => {
                if let Some(login) = app.opencode_go_login.as_mut() {
                    login.update_label(label);
                }
                Task::none()
            }
            OpenCodeGoLoginEvent::Saved => {
                let Some(login) = app.opencode_go_login.as_mut() else {
                    return Task::none();
                };
                let flow_id = login.account_id.clone();
                match opencode_go::login::save(login) {
                    Ok(managed_account) => {
                        let account_id = managed_account.id.clone();
                        let selected_account_id = account_id.clone();
                        let task = apply_login_success(
                            app,
                            ProviderId::OpenCodeGo,
                            &flow_id,
                            account_id,
                            move |config| {
                                opencode_go::account::apply_login_account(config, managed_account);
                                select_account_after_login(
                                    config,
                                    ProviderId::OpenCodeGo,
                                    selected_account_id,
                                );
                            },
                        );
                        app.opencode_go_login_handle = None;
                        app.opencode_go_login = None;
                        task
                    }
                    Err(_) => Task::none(),
                }
            }
        }
    }
}
