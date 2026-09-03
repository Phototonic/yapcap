use super::super::super::{
    Alignment, Background, Config, Element, Length, Message, ProviderAccountActionSupport,
    ProviderAccountRuntimeState, ProviderId, accent_selection_fill, account_label_text,
    apply_alpha, badge_destructive, badge_destructive_soft, badge_neutral, badge_neutral_soft,
    badge_success, badge_success_soft, badge_warning, badge_warning_soft, badge_with_tooltip,
    container, disabled_account_label_text, fl, registry, row, widget,
};
use crate::model::{AuthState, ProviderHealth, STALE_THRESHOLD};

#[derive(Clone, Copy, PartialEq, Eq)]
enum RowBadgeKind {
    Warning,
    Neutral,
    Destructive,
}

struct RowStatus {
    kind: RowBadgeKind,
    badge_text: String,
    tooltip_text: String,
    reauth_eligible: bool,
    style_as_action_required: bool,
}

#[derive(Clone, Copy)]
pub(super) struct AccountRowPosition {
    pub(super) first: bool,
    pub(super) last: bool,
}

fn row_status(provider: ProviderId, account: &ProviderAccountRuntimeState) -> Option<RowStatus> {
    match provider {
        ProviderId::Cursor => {
            (account.auth_state == AuthState::ActionRequired).then(|| RowStatus {
                kind: RowBadgeKind::Neutral,
                badge_text: fl!("cursor-account-reauth-badge"),
                tooltip_text: fl!("badge-reauth-tooltip"),
                reauth_eligible: true,
                style_as_action_required: true,
            })
        }
        ProviderId::Claude => claude_account_row_status(account).map(|status| match status {
            ClaudeAccountRowStatus::ReauthRequired => RowStatus {
                kind: RowBadgeKind::Warning,
                badge_text: fl!("badge-login-required"),
                tooltip_text: fl!("badge-login-required-tooltip"),
                reauth_eligible: true,
                style_as_action_required: true,
            },
            ClaudeAccountRowStatus::Error => RowStatus {
                kind: RowBadgeKind::Destructive,
                badge_text: fl!("badge-error"),
                tooltip_text: fl!("badge-error-tooltip"),
                reauth_eligible: false,
                style_as_action_required: true,
            },
            ClaudeAccountRowStatus::Stale => RowStatus {
                kind: RowBadgeKind::Warning,
                badge_text: fl!("badge-stale"),
                tooltip_text: fl!("badge-stale-tooltip"),
                reauth_eligible: false,
                style_as_action_required: false,
            },
        }),
        ProviderId::Codex
        | ProviderId::Gemini
        | ProviderId::Copilot
        | ProviderId::Minimax
        | ProviderId::Kimi
        | ProviderId::Antigravity
        | ProviderId::OpenCodeGo => {
            (account.auth_state == AuthState::ActionRequired).then(|| RowStatus {
                kind: RowBadgeKind::Warning,
                badge_text: fl!("badge-login-required"),
                tooltip_text: fl!("badge-login-required-tooltip"),
                reauth_eligible: true,
                style_as_action_required: true,
            })
        }
    }
}

fn status_badge(status: &RowStatus, enabled: bool) -> Element<'static, Message> {
    let badge = match (status.kind, enabled) {
        (RowBadgeKind::Warning, true) => badge_warning(status.badge_text.clone()),
        (RowBadgeKind::Warning, false) => badge_warning_soft(status.badge_text.clone()),
        (RowBadgeKind::Neutral, true) => badge_neutral(status.badge_text.clone()),
        (RowBadgeKind::Neutral, false) => badge_neutral_soft(status.badge_text.clone()),
        (RowBadgeKind::Destructive, true) => badge_destructive(status.badge_text.clone()),
        (RowBadgeKind::Destructive, false) => badge_destructive_soft(status.badge_text.clone()),
    };
    badge_with_tooltip(badge, status.tooltip_text.clone())
}

fn row_label(
    provider: ProviderId,
    account: &ProviderAccountRuntimeState,
    config: &Config,
) -> String {
    if provider == ProviderId::Claude {
        claude_account_row_label(account, config)
    } else {
        account.label.clone()
    }
}

fn reauth_capability_satisfied(
    provider: ProviderId,
    action_support: Option<&ProviderAccountActionSupport>,
) -> bool {
    match provider {
        ProviderId::Codex | ProviderId::Minimax | ProviderId::Kimi | ProviderId::OpenCodeGo => true,
        ProviderId::Cursor => action_support.is_some_and(|support| {
            support.can_reauthenticate && support.supports_background_status_refresh
        }),
        ProviderId::Claude | ProviderId::Gemini | ProviderId::Copilot | ProviderId::Antigravity => {
            action_support.is_some_and(|support| support.can_reauthenticate)
        }
    }
}

fn reauth_tooltip(provider: ProviderId) -> String {
    match provider {
        ProviderId::Codex => fl!("codex-account-reauth-tooltip"),
        ProviderId::Claude => fl!("claude-account-reauth-tooltip"),
        ProviderId::Cursor => fl!("cursor-account-reauth-tooltip"),
        ProviderId::Gemini => fl!("gemini-account-reauth-tooltip"),
        ProviderId::Copilot => fl!("copilot-account-reauth-tooltip"),
        ProviderId::Minimax => fl!("minimax-account-reauth-tooltip"),
        ProviderId::Kimi => fl!("kimi-account-reauth-tooltip"),
        ProviderId::Antigravity => fl!("antigravity-account-reauth-tooltip"),
        ProviderId::OpenCodeGo => "Re-authenticate this OpenCode Go account".to_string(),
    }
}

pub(super) fn account_settings_row<'a>(
    provider: ProviderId,
    account: &'a ProviderAccountRuntimeState,
    selected_ids: &[&str],
    active_id: Option<&str>,
    config: &'a Config,
    enabled: bool,
    position: AccountRowPosition,
) -> Element<'a, Message> {
    let is_selected = selected_ids.contains(&account.account_id.as_str());
    let is_active = active_id == Some(account.account_id.as_str());
    let status = row_status(provider, account);
    let action_support = account_action_support(config, provider, account.account_id.as_str());
    let can_reauthenticate = enabled
        && status.as_ref().is_some_and(|status| status.reauth_eligible)
        && reauth_capability_satisfied(provider, action_support.as_ref());
    let account_id = account.account_id.clone();
    let label = row_label(provider, account, config);

    let account_label = if enabled {
        account_label_text(&label, 14)
    } else {
        disabled_account_label_text(&label, 14)
    };
    let mut title_row = row![account_label]
        .spacing(8)
        .align_y(Alignment::Center)
        .height(Length::Fixed(22.0))
        .width(Length::Fill);
    if is_active {
        title_row = title_row.push(badge_with_tooltip(
            active_badge(enabled),
            fl!("badge-active-tooltip"),
        ));
    }
    if let Some(status) = &status {
        title_row = title_row.push(status_badge(status, enabled));
    }

    let selector_content = container(title_row).padding([8, 12]).width(Length::Fill);

    let selector = widget::button::custom(selector_content)
        .class(account_row_button_class(is_selected))
        .width(Length::Fill)
        .on_press_maybe(enabled.then_some(Message::ToggleAccountSelection(
            provider,
            account_id.clone(),
        )));

    let can_delete = action_support.is_some_and(|support| support.can_delete);
    let delete_press =
        (enabled && can_delete).then_some(Message::DeleteAccount(provider, account_id.clone()));
    let mut actions = row![account_selected_marker(is_selected, enabled)]
        .spacing(0)
        .align_y(Alignment::Center);
    if can_reauthenticate {
        let message = if matches!(provider, ProviderId::Codex | ProviderId::Copilot)
            && account.auth_state == AuthState::ActionRequired
        {
            Message::RestoreFromOpenCode(provider, account_id.clone())
        } else {
            Message::ReauthenticateAccount(provider, account_id.clone())
        };
        actions = actions.push(account_action_icon_button(
            "view-refresh-symbolic",
            reauth_tooltip(provider),
            Some(message),
        ));
    }
    actions = actions.push(account_action_icon_button(
        "edit-delete-symbolic",
        fl!("account-delete-tooltip"),
        delete_press,
    ));

    Element::from(account_row_container(
        selector.into(),
        actions.into(),
        is_selected,
        enabled,
        status.is_some_and(|status| status.style_as_action_required),
        position.first,
        position.last,
    ))
}

pub(super) fn account_selector_list<'a>(
    rows: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(rows)
        .width(Length::Fill)
        .style(|theme: &cosmic::Theme| {
            let cosmic = theme.cosmic();
            let surface = &cosmic.background(theme.transparent).component;
            widget::container::Style {
                text_color: Some(surface.on.into()),
                background: Some(Background::Color(surface.base.into())),
                border: cosmic::iced::Border {
                    radius: cosmic.corner_radii.radius_s.into(),
                    width: 1.0,
                    color: surface.divider.into(),
                },
                shadow: cosmic::iced::Shadow::default(),
                icon_color: Some(surface.on.into()),
                snap: true,
            }
        })
        .into()
}

pub(super) fn account_action_container<'a>(
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(content)
        .padding([16, 0, 0, 0])
        .width(Length::Fill)
        .into()
}

fn claude_account_row_label(account: &ProviderAccountRuntimeState, config: &Config) -> String {
    let id = account.account_id.as_str();
    let managed = config
        .claude_managed_accounts
        .iter()
        .find(|managed| managed.id == id);
    let config_email = managed
        .and_then(|managed| managed.email.as_deref())
        .filter(|email| !email.is_empty());
    let snapshot_email = account
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.identity.email.as_deref())
        .filter(|email| !email.is_empty());
    snapshot_email
        .or(config_email)
        .unwrap_or(account.label.as_str())
        .to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClaudeAccountRowStatus {
    ReauthRequired,
    Error,
    Stale,
}

fn claude_account_row_status(
    account: &ProviderAccountRuntimeState,
) -> Option<ClaudeAccountRowStatus> {
    if account.auth_state == AuthState::ActionRequired {
        return Some(ClaudeAccountRowStatus::ReauthRequired);
    }
    if account.health == ProviderHealth::Error {
        if account.snapshot.is_some() {
            return Some(ClaudeAccountRowStatus::Stale);
        }
        return Some(ClaudeAccountRowStatus::Error);
    }
    if account.snapshot.is_some()
        && account
            .last_success_at
            .is_none_or(|updated| chrono::Utc::now() - updated >= STALE_THRESHOLD)
    {
        return Some(ClaudeAccountRowStatus::Stale);
    }
    None
}

fn active_badge(enabled: bool) -> Element<'static, Message> {
    if enabled {
        badge_success(fl!("badge-active"))
    } else {
        badge_success_soft(fl!("badge-active"))
    }
}

fn account_selected_marker(selected: bool, enabled: bool) -> Element<'static, Message> {
    if !selected {
        return cosmic::iced::widget::Space::new()
            .width(Length::Fixed(18.0))
            .into();
    }

    container(
        widget::icon::icon(widget::icon::from_name("object-select-symbolic").into())
            .size(18)
            .width(Length::Fixed(18.0))
            .height(Length::Fixed(18.0)),
    )
    .style(move |theme| {
        let cosmic = theme.cosmic();
        let color = if enabled {
            cosmic.accent.base.into()
        } else {
            apply_alpha(
                cosmic.background(theme.transparent).component.on.into(),
                0.45,
            )
        };
        widget::container::Style {
            text_color: Some(color),
            background: None,
            border: cosmic::iced::Border::default(),
            shadow: cosmic::iced::Shadow::default(),
            icon_color: Some(color),
            snap: true,
        }
    })
    .into()
}

fn account_action_support(
    config: &Config,
    provider: ProviderId,
    account_id: &str,
) -> Option<ProviderAccountActionSupport> {
    registry::discover_accounts(provider, config)
        .into_iter()
        .find(|account| account.provider == provider && account.account_id == account_id)
        .map(|account| account.action_support())
}

fn account_row_container<'a>(
    selector: Element<'a, Message>,
    delete_button: Element<'a, Message>,
    selected: bool,
    enabled: bool,
    action_required: bool,
    first: bool,
    last: bool,
) -> Element<'a, Message> {
    container(
        row![selector, delete_button]
            .spacing(0)
            .align_y(Alignment::Center)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .style(move |theme: &cosmic::Theme| {
        let cosmic = theme.cosmic();
        let surface = &cosmic.background(theme.transparent).component;
        let warning = cosmic.warning.base;
        widget::container::Style {
            text_color: Some(surface.on.into()),
            background: Some(Background::Color(if action_required {
                apply_alpha(warning.into(), 0.08)
            } else if selected && enabled {
                accent_selection_fill(theme)
            } else {
                surface.base.into()
            })),
            border: cosmic::iced::Border {
                radius: account_row_radius(cosmic.corner_radii.radius_s, first, last),
                width: if selected { 2.0 } else { 1.0 },
                color: if selected {
                    if enabled {
                        cosmic.accent.base.into()
                    } else {
                        apply_alpha(surface.on.into(), 0.45)
                    }
                } else if action_required {
                    apply_alpha(warning.into(), 0.72)
                } else {
                    surface.divider.into()
                },
            },
            shadow: cosmic::iced::Shadow::default(),
            icon_color: Some(if enabled {
                surface.on.into()
            } else {
                apply_alpha(surface.on.into(), 0.45)
            }),
            snap: true,
        }
    })
    .into()
}

fn account_row_radius(radius: [f32; 4], first: bool, last: bool) -> cosmic::iced::border::Radius {
    cosmic::iced::border::Radius {
        top_left: if first { radius[0] } else { 0.0 },
        top_right: if first { radius[1] } else { 0.0 },
        bottom_right: if last { radius[2] } else { 0.0 },
        bottom_left: if last { radius[3] } else { 0.0 },
    }
}

fn account_row_button_class(selected: bool) -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(move |focused, theme| {
            account_row_button_style(theme, selected, focused, 1.0)
        }),
        disabled: Box::new(move |theme| account_row_button_style(theme, selected, false, 0.45)),
        hovered: Box::new(move |focused, theme| {
            account_row_button_style(theme, selected, focused, 1.0)
        }),
        pressed: Box::new(move |focused, theme| {
            account_row_button_style(theme, selected, focused, 0.92)
        }),
    }
}

fn account_row_button_style(
    theme: &cosmic::Theme,
    selected: bool,
    focused: bool,
    opacity: f32,
) -> widget::button::Style {
    let cosmic = theme.cosmic();
    let mut style = widget::button::Style::new();
    let foreground = cosmic.background(theme.transparent).component.on.into();

    style.icon_color = Some(apply_alpha(foreground, opacity));
    style.text_color = Some(apply_alpha(foreground, opacity));
    style.border_radius = cosmic.corner_radii.radius_s.into();
    style.border_width = if focused && selected { 1.0 } else { 0.0 };
    style.border_color = cosmic.accent.base.into();

    style
}

fn account_row_icon_button_class(available: bool) -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(move |_focused, theme| {
            account_row_icon_button_style(theme, if available { 1.0 } else { 0.45 })
        }),
        disabled: Box::new(move |theme| account_row_icon_button_style(theme, 0.45)),
        hovered: Box::new(move |_focused, theme| {
            account_row_icon_button_style(theme, if available { 1.0 } else { 0.45 })
        }),
        pressed: Box::new(move |_focused, theme| {
            account_row_icon_button_style(theme, if available { 0.85 } else { 0.45 })
        }),
    }
}

fn account_action_icon_button(
    icon_name: &'static str,
    tooltip: String,
    press: Option<Message>,
) -> Element<'static, Message> {
    let available = press.is_some();
    let handle = widget::icon::from_name(icon_name)
        .icon()
        .into_svg_handle()
        .unwrap_or_else(|| widget::svg::Handle::from_memory(Vec::new()));
    let icon = widget::Svg::new(handle)
        .symbolic(true)
        .class(cosmic::theme::Svg::custom(|theme| widget::svg::Style {
            color: Some(
                theme
                    .cosmic()
                    .background(theme.transparent)
                    .component
                    .on
                    .into(),
            ),
        }))
        .opacity(if available { 1.0_f32 } else { 0.45_f32 })
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0));

    widget::tooltip::tooltip(
        widget::button::custom(icon)
            .class(account_row_icon_button_class(available))
            .padding(4)
            .on_press_maybe(press),
        widget::text(tooltip).size(12),
        widget::tooltip::Position::Top,
    )
    .into()
}

fn account_row_icon_button_style(theme: &cosmic::Theme, opacity: f32) -> widget::button::Style {
    let cosmic = theme.cosmic();
    let mut style = widget::button::Style::new();
    let foreground = cosmic.background(theme.transparent).component.on.into();

    style.icon_color = Some(apply_alpha(foreground, opacity));
    style.text_color = Some(apply_alpha(foreground, opacity));
    style.border_radius = cosmic.corner_radii.radius_m.into();

    style
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ManagedClaudeAccountConfig;
    use crate::model::{AuthState, ProviderHealth};
    use chrono::Utc;
    use std::path::PathBuf;

    fn claude_config(id: &str, email: Option<&str>) -> Config {
        Config {
            claude_managed_accounts: vec![ManagedClaudeAccountConfig {
                id: id.to_string(),
                label: "Claude account".to_string(),
                config_dir: PathBuf::from("/tmp/claude-test"),
                email: email.map(str::to_string),
                organization: None,
                subscription_type: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_authenticated_at: Some(Utc::now()),
            }],
            ..Config::default()
        }
    }

    #[test]
    fn claude_row_label_prefers_email_from_config() {
        let config = claude_config("claude-1", Some("user@example.com"));
        let account =
            ProviderAccountRuntimeState::empty(ProviderId::Claude, "claude-1", "Claude account");

        assert_eq!(
            claude_account_row_label(&account, &config),
            "user@example.com"
        );
    }

    #[test]
    fn claude_row_status_marks_action_required_before_error() {
        let mut account =
            ProviderAccountRuntimeState::empty(ProviderId::Claude, "claude-1", "Claude account");
        account.health = ProviderHealth::Error;
        account.auth_state = AuthState::ActionRequired;

        assert_eq!(
            claude_account_row_status(&account),
            Some(ClaudeAccountRowStatus::ReauthRequired)
        );
    }

    #[test]
    fn codex_and_gemini_row_status_requires_action_when_auth_state_demands_it() {
        for provider in [ProviderId::Codex, ProviderId::Gemini] {
            let mut account = ProviderAccountRuntimeState::empty(provider, "acct-1", "Account");
            account.auth_state = AuthState::ActionRequired;
            assert!(
                row_status(provider, &account).is_some(),
                "{provider:?} should require action"
            );
        }
    }

    #[test]
    fn codex_row_status_is_none_when_auth_state_is_ready() {
        let mut account =
            ProviderAccountRuntimeState::empty(ProviderId::Codex, "codex-1", "Codex account");
        account.auth_state = AuthState::Ready;
        assert!(row_status(ProviderId::Codex, &account).is_none());
    }

    #[test]
    fn cursor_reauth_copy_does_not_use_inactive() {
        assert_eq!(fl!("cursor-account-reauth-badge"), "Re-auth needed");
        assert_eq!(
            fl!("cursor-account-reauth-tooltip"),
            "Rescan Cursor account"
        );
        assert!(!fl!("cursor-account-reauth-detail").contains("inactive"));
        assert!(!fl!("cursor-accounts-reauth-summary").contains("inactive"));
    }

    #[test]
    fn claude_row_status_marks_stale_snapshot() {
        let mut account =
            ProviderAccountRuntimeState::empty(ProviderId::Claude, "claude-1", "Claude account");
        account.health = ProviderHealth::Error;
        account.auth_state = AuthState::Ready;
        account.snapshot = Some(crate::model::UsageSnapshot {
            provider: ProviderId::Claude,
            source: "test".to_string(),
            updated_at: Utc::now(),
            headline: crate::model::UsageHeadline(0),
            windows: Vec::new(),
            provider_cost: None,
            extra_usage: None,
            identity: crate::model::ProviderIdentity::default(),
        });

        assert_eq!(
            claude_account_row_status(&account),
            Some(ClaudeAccountRowStatus::Stale)
        );
    }
}
