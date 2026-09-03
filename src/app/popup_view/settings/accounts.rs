mod login_controls;
mod rows;

use self::login_controls::{
    antigravity_login_controls, claude_login_controls, codex_login_controls,
    copilot_login_controls, cursor_scan_controls, gemini_login_controls, kimi_login_controls,
    minimax_login_controls, opencode_go_login_controls,
};
use self::rows::{account_selector_list, account_settings_row};
use super::super::{
    Alignment, AppState, Config, DetectionSnapshot, Element, Length, Message, ProviderId,
    ProviderLoginStates, detected_without_accounts, fl, provider_icon_handle,
    provider_icon_variant, row, settings_block_enabled, widget,
};
use crate::providers::antigravity::AntigravityLoginState;
use crate::providers::claude::ClaudeLoginState;
use crate::providers::codex::CodexLoginState;
use crate::providers::copilot::CopilotLoginState;
use crate::providers::cursor::CursorScanState;
use crate::providers::gemini::GeminiLoginState;
use crate::providers::kimi::login::KimiLoginState;
use crate::providers::minimax::MinimaxLoginState;
use crate::providers::opencode_go::login::OpenCodeGoLoginState;

pub(super) fn provider_settings_view<'a>(
    state: &'a AppState,
    config: &'a Config,
    detection: &'a DetectionSnapshot,
    logins: ProviderLoginStates<'a>,
    provider_id: ProviderId,
) -> Element<'a, Message> {
    let enabled = state
        .provider(provider_id)
        .is_some_and(|provider| provider.enabled);

    let provider_header = row![
        widget::icon::icon(provider_icon_handle(provider_id, provider_icon_variant())).size(22),
        widget::text(provider_id.label()).size(18),
        cosmic::iced::widget::Space::new().width(Length::Fill),
        widget::toggler(enabled)
            .on_toggle(move |enabled| Message::SetProviderEnabled(provider_id, enabled)),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    let accounts_section = match provider_id {
        ProviderId::Codex => codex_accounts_section(state, config, logins.codex, enabled),
        ProviderId::Claude => claude_accounts_section(state, config, logins.claude, enabled),
        ProviderId::Cursor => cursor_accounts_section(state, config, logins.cursor_scan, enabled),
        ProviderId::Gemini => gemini_accounts_section(state, config, logins.gemini, enabled),
        ProviderId::Copilot => copilot_accounts_section(state, config, logins.copilot, enabled),
        ProviderId::Minimax => minimax_accounts_section(state, config, logins.minimax, enabled),
        ProviderId::Kimi => kimi_accounts_section(state, config, logins.kimi, enabled),
        ProviderId::Antigravity => {
            antigravity_accounts_section(state, config, logins.antigravity, enabled)
        }
        ProviderId::OpenCodeGo => {
            opencode_go_accounts_section(state, config, logins.opencode_go, enabled)
        }
    };

    let mut sections = cosmic::iced::widget::column![provider_header].spacing(14);
    if detected_without_accounts(state, detection, provider_id) {
        sections = sections.push(widget::text(fl!("provider-detected-caption")).size(13));
    }
    Element::from(sections.push(accounts_section).width(Length::Fill))
}

fn codex_accounts_section<'a>(
    state: &'a AppState,
    config: &'a Config,
    codex_login: Option<&'a CodexLoginState>,
    enabled: bool,
) -> Element<'a, Message> {
    let codex = state.provider(ProviderId::Codex);
    let selected_ids: Vec<&str> = codex
        .map(|provider| {
            provider
                .selected_account_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let accounts = state.accounts_for(ProviderId::Codex);
    let active_id = codex.and_then(|provider| provider.system_active_account_id.as_deref());
    let mut rows = cosmic::iced::widget::column![]
        .spacing(8)
        .width(Length::Fill);

    if accounts.is_empty() {
        rows = rows.push(widget::text(fl!("codex-accounts-empty")).size(13));
    } else {
        let mut account_rows = cosmic::iced::widget::column![]
            .spacing(0)
            .width(Length::Fill);
        for account in &accounts {
            account_rows = account_rows.push(account_settings_row(
                ProviderId::Codex,
                account,
                &selected_ids,
                active_id,
                config,
                enabled,
            ));
        }
        rows = rows.push(account_selector_list(account_rows));
    }

    if let Some(provider) = codex
        && provider.account_status == crate::model::AccountSelectionStatus::SelectionRequired
    {
        rows = rows.push(widget::text(fl!("codex-account-select-required")).size(13));
    }

    rows = rows.push(codex_login_controls(
        codex_login,
        crate::providers::codex::opencode_import_available(),
        enabled,
    ));

    settings_block_enabled(
        widget::text(fl!("codex-accounts-title")).size(16).into(),
        rows,
        enabled,
    )
}

fn claude_accounts_section<'a>(
    state: &'a AppState,
    config: &'a Config,
    claude_login: Option<&'a ClaudeLoginState>,
    enabled: bool,
) -> Element<'a, Message> {
    let selected_ids: Vec<&str> = state
        .provider(ProviderId::Claude)
        .map(|provider| {
            provider
                .selected_account_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let accounts = state.accounts_for(ProviderId::Claude);
    let active_id = state
        .provider(ProviderId::Claude)
        .and_then(|provider| provider.system_active_account_id.as_deref());
    let mut rows = cosmic::iced::widget::column![]
        .spacing(8)
        .width(Length::Fill);

    if accounts.is_empty() {
        rows = rows.push(widget::text(fl!("claude-accounts-empty")).size(13));
    } else {
        let mut account_rows = cosmic::iced::widget::column![]
            .spacing(0)
            .width(Length::Fill);
        for account in &accounts {
            account_rows = account_rows.push(account_settings_row(
                ProviderId::Claude,
                account,
                &selected_ids,
                active_id,
                config,
                enabled,
            ));
        }
        rows = rows.push(account_selector_list(account_rows));
    }

    if let Some(login) = claude_login
        && login.status == crate::providers::claude::ClaudeLoginStatus::Running
    {
        rows = rows.push(widget::text(fl!("account-browser-login-hint")).size(12));
    }
    rows = rows.push(claude_login_controls(claude_login, enabled));

    settings_block_enabled(
        widget::text(fl!("claude-accounts-title")).size(16).into(),
        rows,
        enabled,
    )
}

fn gemini_accounts_section<'a>(
    state: &'a AppState,
    config: &'a Config,
    gemini_login: Option<&'a GeminiLoginState>,
    enabled: bool,
) -> Element<'a, Message> {
    let selected_ids: Vec<&str> = state
        .provider(ProviderId::Gemini)
        .map(|provider| {
            provider
                .selected_account_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let accounts = state.accounts_for(ProviderId::Gemini);
    let active_id = state
        .provider(ProviderId::Gemini)
        .and_then(|provider| provider.system_active_account_id.as_deref());
    let mut rows = cosmic::iced::widget::column![]
        .spacing(8)
        .width(Length::Fill);

    if accounts.is_empty() {
        rows = rows.push(widget::text(fl!("gemini-accounts-empty")).size(13));
    } else {
        let mut account_rows = cosmic::iced::widget::column![]
            .spacing(0)
            .width(Length::Fill);
        for account in &accounts {
            account_rows = account_rows.push(account_settings_row(
                ProviderId::Gemini,
                account,
                &selected_ids,
                active_id,
                config,
                enabled,
            ));
        }
        rows = rows.push(account_selector_list(account_rows));
    }

    rows = rows.push(gemini_login_controls(gemini_login, enabled));

    settings_block_enabled(
        widget::text(fl!("gemini-accounts-title")).size(16).into(),
        rows,
        enabled,
    )
}

fn antigravity_accounts_section<'a>(
    state: &'a AppState,
    config: &'a Config,
    antigravity_login: Option<&'a AntigravityLoginState>,
    enabled: bool,
) -> Element<'a, Message> {
    let selected_ids: Vec<&str> = state
        .provider(ProviderId::Antigravity)
        .map(|provider| {
            provider
                .selected_account_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let accounts = state.accounts_for(ProviderId::Antigravity);
    let active_id = state
        .provider(ProviderId::Antigravity)
        .and_then(|provider| provider.system_active_account_id.as_deref());
    let mut rows = cosmic::iced::widget::column![]
        .spacing(8)
        .width(Length::Fill);

    if accounts.is_empty() {
        rows = rows.push(widget::text(fl!("antigravity-accounts-empty")).size(13));
    } else {
        let mut account_rows = cosmic::iced::widget::column![]
            .spacing(0)
            .width(Length::Fill);
        for account in &accounts {
            account_rows = account_rows.push(account_settings_row(
                ProviderId::Antigravity,
                account,
                &selected_ids,
                active_id,
                config,
                enabled,
            ));
        }
        rows = rows.push(account_selector_list(account_rows));
    }

    rows = rows.push(antigravity_login_controls(antigravity_login, enabled));

    settings_block_enabled(
        widget::text(fl!("antigravity-accounts-title"))
            .size(16)
            .into(),
        rows,
        enabled,
    )
}

fn cursor_accounts_section<'a>(
    state: &'a AppState,
    config: &'a Config,
    cursor_scan: &'a CursorScanState,
    enabled: bool,
) -> Element<'a, Message> {
    let selected_ids: Vec<&str> = state
        .provider(ProviderId::Cursor)
        .map(|provider| {
            provider
                .selected_account_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let accounts = state.accounts_for(ProviderId::Cursor);
    let active_id = state
        .provider(ProviderId::Cursor)
        .and_then(|provider| provider.system_active_account_id.as_deref());
    let mut rows = cosmic::iced::widget::column![]
        .spacing(8)
        .width(Length::Fill);

    if accounts.is_empty() {
        rows = rows.push(widget::text(fl!("cursor-accounts-empty")).size(13));
    } else {
        let mut account_rows = cosmic::iced::widget::column![]
            .spacing(0)
            .width(Length::Fill);
        for account in &accounts {
            account_rows = account_rows.push(account_settings_row(
                ProviderId::Cursor,
                account,
                &selected_ids,
                active_id,
                config,
                enabled,
            ));
        }
        rows = rows.push(account_selector_list(account_rows));
    }

    rows = rows.push(cursor_scan_controls(cursor_scan, enabled));

    settings_block_enabled(
        widget::text(fl!("cursor-accounts-title")).size(16).into(),
        rows,
        enabled,
    )
}

fn copilot_accounts_section<'a>(
    state: &'a AppState,
    config: &'a Config,
    copilot_login: Option<&'a CopilotLoginState>,
    enabled: bool,
) -> Element<'a, Message> {
    let selected_ids: Vec<&str> = state
        .provider(ProviderId::Copilot)
        .map(|provider| {
            provider
                .selected_account_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let accounts = state.accounts_for(ProviderId::Copilot);
    let active_id = state
        .provider(ProviderId::Copilot)
        .and_then(|provider| provider.system_active_account_id.as_deref());
    let mut rows = cosmic::iced::widget::column![]
        .spacing(8)
        .width(Length::Fill);

    if accounts.is_empty() {
        rows = rows.push(widget::text(fl!("copilot-accounts-empty")).size(13));
    } else {
        let mut account_rows = cosmic::iced::widget::column![]
            .spacing(0)
            .width(Length::Fill);
        for account in &accounts {
            account_rows = account_rows.push(account_settings_row(
                ProviderId::Copilot,
                account,
                &selected_ids,
                active_id,
                config,
                enabled,
            ));
        }
        rows = rows.push(account_selector_list(account_rows));
    }

    rows = rows.push(copilot_login_controls(
        copilot_login,
        crate::providers::copilot::opencode_import_available(),
        enabled,
    ));

    settings_block_enabled(
        widget::text(fl!("copilot-accounts-title")).size(16).into(),
        rows,
        enabled,
    )
}

fn minimax_accounts_section<'a>(
    state: &'a AppState,
    config: &'a Config,
    minimax_login: Option<&'a MinimaxLoginState>,
    enabled: bool,
) -> Element<'a, Message> {
    let minimax = state.provider(ProviderId::Minimax);
    let selected_ids: Vec<&str> = minimax
        .map(|provider| {
            provider
                .selected_account_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let accounts = state.accounts_for(ProviderId::Minimax);
    let active_id = minimax.and_then(|provider| provider.system_active_account_id.as_deref());
    let mut rows = cosmic::iced::widget::column![]
        .spacing(8)
        .width(Length::Fill);

    if accounts.is_empty() {
        rows = rows.push(widget::text(fl!("minimax-accounts-empty")).size(13));
    } else {
        let mut account_rows = cosmic::iced::widget::column![]
            .spacing(0)
            .width(Length::Fill);
        for account in &accounts {
            account_rows = account_rows.push(account_settings_row(
                ProviderId::Minimax,
                account,
                &selected_ids,
                active_id,
                config,
                enabled,
            ));
        }
        rows = rows.push(account_selector_list(account_rows));
    }

    if let Some(provider) = minimax
        && provider.account_status == crate::model::AccountSelectionStatus::SelectionRequired
    {
        rows = rows.push(widget::text(fl!("minimax-account-select-required")).size(13));
    }

    rows = rows.push(minimax_login_controls(minimax_login, enabled));

    settings_block_enabled(
        widget::text("Minimax Accounts").size(16).into(),
        rows,
        enabled,
    )
}

fn kimi_accounts_section<'a>(
    state: &'a AppState,
    config: &'a Config,
    kimi_login: Option<&'a KimiLoginState>,
    enabled: bool,
) -> Element<'a, Message> {
    let kimi = state.provider(ProviderId::Kimi);
    let selected_ids: Vec<&str> = kimi
        .map(|provider| {
            provider
                .selected_account_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let accounts = state.accounts_for(ProviderId::Kimi);
    let active_id = kimi.and_then(|provider| provider.system_active_account_id.as_deref());
    let mut rows = cosmic::iced::widget::column![]
        .spacing(8)
        .width(Length::Fill);

    if accounts.is_empty() {
        rows = rows.push(widget::text(fl!("kimi-accounts-empty")).size(13));
    } else {
        let mut account_rows = cosmic::iced::widget::column![]
            .spacing(0)
            .width(Length::Fill);
        for account in &accounts {
            account_rows = account_rows.push(account_settings_row(
                ProviderId::Kimi,
                account,
                &selected_ids,
                active_id,
                config,
                enabled,
            ));
        }
        rows = rows.push(account_selector_list(account_rows));
    }

    if let Some(provider) = kimi
        && provider.account_status == crate::model::AccountSelectionStatus::SelectionRequired
    {
        rows = rows.push(widget::text(fl!("kimi-account-select-required")).size(13));
    }

    rows = rows.push(kimi_login_controls(kimi_login, enabled));

    settings_block_enabled(
        widget::text(fl!("kimi-accounts-title")).size(16).into(),
        rows,
        enabled,
    )
}

fn opencode_go_accounts_section<'a>(
    state: &'a AppState,
    config: &'a Config,
    login: Option<&'a OpenCodeGoLoginState>,
    enabled: bool,
) -> Element<'a, Message> {
    let provider = state.provider(ProviderId::OpenCodeGo);
    let selected_ids: Vec<&str> = provider
        .map(|value| {
            value
                .selected_account_ids
                .iter()
                .map(String::as_str)
                .collect()
        })
        .unwrap_or_default();
    let accounts = state.accounts_for(ProviderId::OpenCodeGo);
    let active_id = provider.and_then(|value| value.system_active_account_id.as_deref());
    let mut rows = cosmic::iced::widget::column![]
        .spacing(8)
        .width(Length::Fill);
    if accounts.is_empty() {
        rows = rows.push(widget::text("No OpenCode Go accounts").size(13));
    } else {
        let mut account_rows = cosmic::iced::widget::column![]
            .spacing(0)
            .width(Length::Fill);
        for account in &accounts {
            account_rows = account_rows.push(account_settings_row(
                ProviderId::OpenCodeGo,
                account,
                &selected_ids,
                active_id,
                config,
                enabled,
            ));
        }
        rows = rows.push(account_selector_list(account_rows));
    }
    rows = rows.push(opencode_go_login_controls(login, enabled));
    settings_block_enabled(
        widget::text("OpenCode Go Accounts").size(16).into(),
        rows,
        enabled,
    )
}
