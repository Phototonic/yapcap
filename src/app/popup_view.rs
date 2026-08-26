// SPDX-License-Identifier: MPL-2.0

mod badges;
mod detail;
mod measure;
mod settings;

use self::badges::{
    account_label_text, apply_alpha, badge_destructive, badge_destructive_soft, badge_neutral,
    badge_neutral_soft, badge_success, badge_success_soft, badge_warning, badge_warning_soft,
    badge_with_tooltip, plan_badge,
};
use self::detail::{active_snapshot, provider_body_height_multi, selected_provider_view};
use self::measure::Measure;
use self::settings::{general_settings_view, provider_settings_view, settings_body_height};
use super::provider_assets::{provider_icon_handle, provider_icon_variant};
use crate::app::{Message, PagerDirection, PopupRoute, SettingsRoute};
use crate::config::{Config, PanelIconStyle, ResetTimeFormat, UsageAmountFormat};
use crate::fl;
use crate::model::{
    AppState, ProviderAccountRuntimeState, ProviderId, ProviderRuntimeState, UsageWindow,
};
use crate::providers::claude::{ClaudeLoginState, ClaudeLoginStatus};
use crate::providers::codex::{CodexLoginState, CodexLoginStatus};
use crate::providers::copilot::{CopilotLoginState, CopilotLoginStatus};
use crate::providers::cursor::CursorScanState;
use crate::providers::gemini::{GeminiLoginState, GeminiLoginStatus};
use crate::providers::interface::ProviderAccountActionSupport;
use crate::providers::kimi::login::KimiLoginState;
use crate::providers::minimax::MinimaxLoginState;
use crate::providers::opencode_go::login::OpenCodeGoLoginState;
use crate::providers::registry;
use crate::updates::UpdateStatus;
use crate::usage_display;
use cosmic::Element;
use cosmic::iced::widget::{column, container, progress_bar, row, scrollable};
use cosmic::iced::{Alignment, Background, Color, Length, Size};
use cosmic::widget;

pub const POPUP_COLUMN_WIDTH: f32 = 420.0;
const POPUP_WIDTH: f32 = POPUP_COLUMN_WIDTH;
const POPUP_MAX_HEIGHT: f32 = 1080.0;
const POPUP_PADDING: f32 = 32.0;
const POPUP_CHROME_SPACING: f32 = 42.0;
const POPUP_HEADER_HEIGHT: f32 = 36.0;
pub(crate) const PROVIDER_TAB_ROW_HEIGHT: f32 = 72.0;
const PROVIDER_TAB_ROW_SPACING: f32 = 8.0;
const PROVIDER_TAB_MAX_COLUMNS: usize = 4;
const PROVIDER_TAB_MAX_BARS: usize = 2;
const PROVIDER_TAB_VERTICAL_PADDING: u16 = 5;
const PROVIDER_TAB_HORIZONTAL_PADDING: u16 = 5;
const PROVIDER_TAB_ITEM_SPACING: f32 = 3.0;
const PROVIDER_NAV_LABEL_SIZE: u16 = 10;
const PROVIDER_TAB_LABEL_LINE_HEIGHT: f32 = 13.0;
const PROVIDER_TAB_BAR_GIRTH: f32 = 4.0;
const PROVIDER_TAB_BAR_AREA_HEIGHT: f32 = 2.0 * PROVIDER_TAB_BAR_GIRTH + PROVIDER_TAB_ITEM_SPACING;
const SETTINGS_CATEGORY_MAX_COLUMNS: usize = 5;
const SETTINGS_CATEGORY_ROW_HEIGHT: f32 = 64.0;
const ACCOUNT_PAGER_HEIGHT: f32 = 40.0;
const POPUP_FOOTER_HEIGHT: f32 = 28.0;
const POPUP_BODY_PANEL_PADDING: f32 = 24.0;
const POPUP_BODY_BOTTOM_SLACK: f32 = 8.0;
const PROVIDER_CARD_SPACING: f32 = 8.0;
const PROVIDER_SUMMARY_HEIGHT: f32 = 58.0;
const PROVIDER_ACCOUNT_HEADER_HEIGHT: f32 = 96.0;
const PROVIDER_SECTION_HEIGHT: f32 = 84.0;
const PROVIDER_SECTION_WITH_ACTION_HEIGHT: f32 = 120.0;
const SETTINGS_SECTION_HEIGHT: f32 = 104.0;
const SETTINGS_PROVIDER_ROW_HEIGHT: f32 = 44.0;
const PROVIDER_TAB_ICON_SIZE: u16 = 16;
const PROVIDER_TAB_ICON_LENGTH: f32 = 16.0;
const PROVIDER_TAB_LABEL_SIZE: u16 = 11;
const UPDATE_NOTIFICATION_DOT_COLOR: Color = Color::from_rgb(0.93, 0.11, 0.15);
const ACCENT_SOFT_FILL_ALPHA: f32 = 0.14;

#[derive(Clone, Copy)]
pub struct ProviderLoginStates<'a> {
    pub codex: Option<&'a CodexLoginState>,
    pub claude: Option<&'a ClaudeLoginState>,
    pub cursor_scan: &'a CursorScanState,
    pub gemini: Option<&'a GeminiLoginState>,
    pub copilot: Option<&'a CopilotLoginState>,
    pub minimax: Option<&'a MinimaxLoginState>,
    pub kimi: Option<&'a KimiLoginState>,
    pub opencode_go: Option<&'a OpenCodeGoLoginState>,
    pub opencode_import_availability: super::OpenCodeImportAvailability,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopupBodyMeasureTarget {
    Provider(ProviderId),
    Settings(SettingsRoute),
}

pub fn popup_content<'a>(
    state: &'a AppState,
    config: &'a Config,
    logins: ProviderLoginStates<'a>,
    selected_provider: ProviderId,
    account_page: usize,
    route: &'a PopupRoute,
    update_status: &'a UpdateStatus,
) -> Element<'a, Message> {
    let selected = selected_state(state, selected_provider);

    let header = popup_header(route);

    let nav_row: Element<'_, Message> = match route {
        PopupRoute::ProviderDetail => provider_nav(state, selected_provider),
        PopupRoute::Settings(settings_route) => {
            settings_category_nav(settings_route, update_status)
        }
    };

    let body = popup_body_view(
        state,
        config,
        logins,
        selected,
        account_page,
        route,
        update_status,
    );

    let footer_action: Element<'_, Message> = match route {
        PopupRoute::ProviderDetail => settings_footer_action(update_status),
        PopupRoute::Settings(_) => widget::button::text(fl!("done"))
            .on_press(Message::NavigateTo(PopupRoute::ProviderDetail))
            .into(),
    };

    let footer = row![
        widget::button::text(fl!("quit")).on_press(Message::Quit),
        cosmic::iced::widget::Space::new().width(Length::Fill),
        footer_action,
    ]
    .align_y(Alignment::Center);

    let body_panel: Element<'_, Message> = container(panel(scrollable(body).width(Length::Fill)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    let body_stack = popup_body_stack(
        state,
        config,
        logins,
        update_status,
        account_page,
        body_panel,
    );

    let content = column![
        narrow_chrome(header),
        narrow_chrome(nav_row),
        body_stack,
        narrow_chrome(footer),
    ]
    .spacing(14)
    .padding(16)
    .width(Length::Fill)
    .height(Length::Fill);

    Element::from(content)
}

fn popup_body_view<'a>(
    state: &'a AppState,
    config: &'a Config,
    logins: ProviderLoginStates<'a>,
    selected: Option<&'a ProviderRuntimeState>,
    account_page: usize,
    route: &'a PopupRoute,
    update_status: &'a UpdateStatus,
) -> Element<'a, Message> {
    match route {
        PopupRoute::ProviderDetail => selected_provider_view(selected, state, config, account_page),
        PopupRoute::Settings(SettingsRoute::General) => {
            general_settings_view(config, update_status)
        }
        PopupRoute::Settings(SettingsRoute::Provider(id)) => {
            provider_settings_view(state, config, logins, *id)
        }
    }
}

fn popup_body_stack<'a>(
    state: &'a AppState,
    config: &'a Config,
    logins: ProviderLoginStates<'a>,
    update_status: &'a UpdateStatus,
    account_page: usize,
    body_panel: Element<'a, Message>,
) -> Element<'a, Message> {
    let mut stack = cosmic::iced::widget::Stack::new()
        .push(body_panel)
        .width(Length::Fill)
        .height(Length::Fill);

    for provider in state.providers.iter().filter(|provider| provider.enabled) {
        let provider_id = provider.provider;
        let body = selected_provider_view(Some(provider), state, config, account_page);
        stack = stack.push(Measure::new(body, POPUP_WIDTH, move |size| {
            Message::PopupBodyMeasured(PopupBodyMeasureTarget::Provider(provider_id), size)
        }));
    }

    let general = general_settings_view(config, update_status);
    stack = stack.push(Measure::new(general, POPUP_WIDTH, |size| {
        Message::PopupBodyMeasured(
            PopupBodyMeasureTarget::Settings(SettingsRoute::General),
            size,
        )
    }));

    for provider in ProviderId::ALL {
        let body = provider_settings_view(state, config, logins, provider);
        stack = stack.push(Measure::new(body, POPUP_WIDTH, move |size| {
            Message::PopupBodyMeasured(
                PopupBodyMeasureTarget::Settings(SettingsRoute::Provider(provider)),
                size,
            )
        }));
    }

    stack.into()
}

pub fn popup_session_size(state: &AppState, _selected_provider: ProviderId) -> Size {
    let provider_height = state
        .providers
        .iter()
        .filter(|provider| provider.enabled)
        .map(|provider| provider_body_height_multi(state, Some(provider)))
        .fold(PROVIDER_SUMMARY_HEIGHT, f32::max);
    let height = POPUP_PADDING
        + POPUP_CHROME_SPACING
        + POPUP_HEADER_HEIGHT
        + provider_nav_height(state)
        + POPUP_FOOTER_HEIGHT
        + POPUP_BODY_PANEL_PADDING
        + POPUP_BODY_BOTTOM_SLACK
        + provider_height;

    Size::new(POPUP_WIDTH, height.clamp(1.0, POPUP_MAX_HEIGHT))
}

pub fn popup_session_size_with_body_height(
    state: &AppState,
    _selected_provider: ProviderId,
    body_height: f32,
) -> Size {
    Size::new(
        POPUP_WIDTH,
        popup_total_height(provider_nav_height(state), body_height),
    )
}

pub fn popup_settings_size(state: &AppState) -> Size {
    let height = POPUP_PADDING
        + POPUP_CHROME_SPACING
        + POPUP_HEADER_HEIGHT
        + settings_nav_height(settings_category_count())
        + POPUP_FOOTER_HEIGHT
        + POPUP_BODY_PANEL_PADDING
        + POPUP_BODY_BOTTOM_SLACK
        + settings_body_height(state);
    Size::new(POPUP_WIDTH, height.clamp(1.0, POPUP_MAX_HEIGHT))
}

pub fn popup_settings_size_with_body_height(body_height: f32) -> Size {
    Size::new(
        POPUP_WIDTH,
        popup_total_height(settings_nav_height(settings_category_count()), body_height),
    )
}

pub(crate) fn settings_category_count() -> usize {
    ProviderId::ALL.len() + 1
}

pub(crate) fn settings_category_row_sizes(category_count: usize) -> Vec<usize> {
    balanced_row_sizes(category_count, SETTINGS_CATEGORY_MAX_COLUMNS)
}

pub(crate) fn settings_nav_height(category_count: usize) -> f32 {
    let rows = settings_category_row_sizes(category_count).len().max(1) as f32;
    rows * SETTINGS_CATEGORY_ROW_HEIGHT + (rows - 1.0) * PROVIDER_TAB_ROW_SPACING
}

fn popup_total_height(nav_height: f32, body_height: f32) -> f32 {
    let height = POPUP_PADDING
        + POPUP_CHROME_SPACING
        + POPUP_HEADER_HEIGHT
        + nav_height
        + POPUP_FOOTER_HEIGHT
        + POPUP_BODY_PANEL_PADDING
        + POPUP_BODY_BOTTOM_SLACK
        + body_height;
    height.clamp(1.0, POPUP_MAX_HEIGHT)
}

fn narrow_chrome<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(container(content.into()).width(Length::Fixed(POPUP_WIDTH)))
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
}

fn panel<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    Element::from(container(content).width(Length::Fill).padding(12))
}

fn popup_header(route: &PopupRoute) -> Element<'static, Message> {
    let mut header = row![
        widget::text(fl!("app-title")).size(22),
        cosmic::iced::widget::Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .spacing(12);

    if matches!(route, PopupRoute::ProviderDetail) {
        header =
            header.push(widget::button::standard(fl!("refresh-now")).on_press(Message::RefreshNow));
    }

    header.into()
}

fn card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    Element::from(container(content).width(Length::Fill).padding(8))
}

fn accent_selection_fill(theme: &cosmic::Theme) -> Color {
    let cosmic = theme.cosmic();
    apply_alpha(cosmic.accent.base.into(), ACCENT_SOFT_FILL_ALPHA)
}

fn settings_block<'a>(
    title: Element<'a, Message>,
    body: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    settings_block_enabled(title, body, true)
}

fn settings_block_enabled<'a>(
    title: Element<'a, Message>,
    body: impl Into<Element<'a, Message>>,
    enabled: bool,
) -> Element<'a, Message> {
    let content = column![title, body.into()].spacing(10).width(Length::Fill);

    let outer = container(content).width(Length::Fill).padding(12);
    if enabled {
        return Element::from(outer);
    }

    Element::from(outer.style(|theme| {
        let cosmic = theme.cosmic();
        widget::container::Style {
            text_color: Some(apply_alpha(cosmic.background.on.into(), 0.45)),
            background: Some(Background::Color(apply_alpha(
                cosmic.background.component.base.into(),
                0.45,
            ))),
            border: cosmic::iced::Border {
                radius: cosmic.corner_radii.radius_s.into(),
                width: 1.0,
                color: apply_alpha(cosmic.background.divider.into(), 0.45),
            },
            shadow: cosmic::iced::Shadow::default(),
            icon_color: Some(apply_alpha(cosmic.background.on.into(), 0.45)),
            snap: true,
        }
    }))
}

fn settings_category_nav(
    route: &SettingsRoute,
    update_status: &UpdateStatus,
) -> Element<'static, Message> {
    let mut start = 0;
    settings_category_row_sizes(settings_category_count())
        .into_iter()
        .fold(column![].spacing(PROVIDER_TAB_ROW_SPACING), |nav, size| {
            let tab_row = (start..start + size).fold(
                row![].spacing(PROVIDER_TAB_ROW_SPACING).width(Length::Fill),
                |tab_row, index| {
                    tab_row.push(settings_category_tab_at(index, route, update_status))
                },
            );
            start += size;
            nav.push(tab_row)
        })
        .into()
}

fn settings_category_tab_at(
    index: usize,
    route: &SettingsRoute,
    update_status: &UpdateStatus,
) -> Element<'static, Message> {
    if index == 0 {
        return settings_category_tab(
            fl!("settings-general-title"),
            settings_category_icon(&SettingsRoute::General),
            matches!(route, SettingsRoute::General),
            SettingsRoute::General,
            update_available(update_status),
        );
    }
    let provider = ProviderId::ALL[index - 1];
    let target_route = SettingsRoute::Provider(provider);
    settings_category_tab(
        provider.label().to_string(),
        settings_category_icon(&target_route),
        matches!(route, SettingsRoute::Provider(id) if *id == provider),
        target_route,
        false,
    )
}

fn settings_category_tab(
    label: String,
    icon: widget::icon::Handle,
    selected: bool,
    route: SettingsRoute,
    notify: bool,
) -> Element<'static, Message> {
    let icon = widget::icon::icon(icon)
        .size(PROVIDER_TAB_ICON_SIZE)
        .width(Length::Fixed(PROVIDER_TAB_ICON_LENGTH))
        .height(Length::Fixed(PROVIDER_TAB_ICON_LENGTH));
    let label: Element<'static, Message> = if notify {
        container(
            row![
                widget::text(label)
                    .size(PROVIDER_TAB_LABEL_SIZE)
                    .width(Length::Shrink)
                    .align_x(Alignment::Center),
                update_notification_dot(6.0)
            ]
            .spacing(5)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
    } else {
        widget::text(label)
            .size(PROVIDER_TAB_LABEL_SIZE)
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .into()
    };
    let content = container(
        column![icon, label]
            .spacing(3)
            .align_x(Alignment::Center)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([5, 9])
    .align_x(Alignment::Center)
    .align_y(Alignment::Center);

    Element::from(
        widget::button::custom(content)
            .class(settings_category_tab_class(selected))
            .width(Length::FillPortion(1))
            .height(Length::Fixed(SETTINGS_CATEGORY_ROW_HEIGHT))
            .on_press(Message::NavigateTo(PopupRoute::Settings(route))),
    )
}

fn update_available(update_status: &UpdateStatus) -> bool {
    matches!(update_status, UpdateStatus::UpdateAvailable { .. })
}

fn settings_footer_action(update_status: &UpdateStatus) -> Element<'static, Message> {
    let target = Message::NavigateTo(PopupRoute::Settings(SettingsRoute::General));

    if !update_available(update_status) {
        return widget::button::text(fl!("settings"))
            .leading_icon(widget::icon::from_name("preferences-system-symbolic"))
            .on_press(target)
            .into();
    }

    let icon = row![
        notification_dot(6.0),
        widget::icon::icon(widget::icon::from_name("preferences-system-symbolic").into())
            .size(16)
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0)),
    ]
    .spacing(5)
    .align_y(Alignment::Center);
    let content = row![icon, widget::text(fl!("settings")).size(14)]
        .spacing(4)
        .align_y(Alignment::Center);

    widget::button::custom(content)
        .class(cosmic::theme::Button::Text)
        .padding([0, 8])
        .on_press(target)
        .into()
}

fn notification_dot(size: f32) -> Element<'static, Message> {
    Element::from(
        container(
            cosmic::iced::widget::Space::new()
                .width(Length::Fixed(size))
                .height(Length::Fixed(size)),
        )
        .style(move |_theme: &cosmic::Theme| widget::container::Style {
            text_color: None,
            background: Some(Background::Color(UPDATE_NOTIFICATION_DOT_COLOR)),
            border: cosmic::iced::Border {
                radius: (size / 2.0).into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: cosmic::iced::Shadow::default(),
            icon_color: None,
            snap: true,
        }),
    )
}

fn update_notification_dot(size: f32) -> Element<'static, Message> {
    widget::tooltip::tooltip(
        notification_dot(size),
        widget::text(fl!("update-dot-tooltip")).size(12),
        widget::tooltip::Position::Top,
    )
    .into()
}

fn settings_category_icon(route: &SettingsRoute) -> widget::icon::Handle {
    match route {
        SettingsRoute::General => widget::icon::from_name("preferences-system-symbolic").into(),
        SettingsRoute::Provider(provider) => {
            provider_icon_handle(*provider, provider_icon_variant())
        }
    }
}

fn settings_category_tab_class(selected: bool) -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(move |_focused, theme| {
            tab_button_style(theme, selected, ButtonInteraction::idle(false), 1.0)
        }),
        disabled: Box::new(move |theme| {
            tab_button_style(theme, selected, ButtonInteraction::idle(false), 0.45)
        }),
        hovered: Box::new(move |_focused, theme| {
            tab_button_style(theme, selected, ButtonInteraction::hover(false), 1.0)
        }),
        pressed: Box::new(move |_focused, theme| {
            tab_button_style(theme, selected, ButtonInteraction::press(false), 0.92)
        }),
    }
}

#[derive(Clone, Copy)]
struct ButtonInteraction {
    focused: bool,
    hovered: bool,
    pressed: bool,
}

impl ButtonInteraction {
    const fn idle(focused: bool) -> Self {
        Self {
            focused,
            hovered: false,
            pressed: false,
        }
    }

    const fn hover(focused: bool) -> Self {
        Self {
            focused,
            hovered: true,
            pressed: false,
        }
    }

    const fn press(focused: bool) -> Self {
        Self {
            focused,
            hovered: true,
            pressed: true,
        }
    }
}

pub(crate) fn provider_tab_row_sizes(tab_count: usize) -> Vec<usize> {
    balanced_row_sizes(tab_count, PROVIDER_TAB_MAX_COLUMNS)
}

fn balanced_row_sizes(count: usize, max_columns: usize) -> Vec<usize> {
    if count == 0 {
        return Vec::new();
    }
    let rows = count.div_ceil(max_columns);
    let base = count / rows;
    let remainder = count % rows;
    (0..rows)
        .map(|row| base + usize::from(row < remainder))
        .collect()
}

#[cfg(test)]
pub(crate) fn provider_tab_bar_area_height() -> f32 {
    let bars = f32::from(u8::try_from(PROVIDER_TAB_MAX_BARS).unwrap_or(u8::MAX));
    bars * PROVIDER_TAB_BAR_GIRTH + (bars - 1.0) * PROVIDER_TAB_ITEM_SPACING
}

#[cfg(test)]
pub(crate) fn provider_tab_content_height() -> f32 {
    2.0 * f32::from(PROVIDER_TAB_VERTICAL_PADDING)
        + PROVIDER_TAB_ICON_LENGTH
        + 2.0 * PROVIDER_TAB_ITEM_SPACING
        + PROVIDER_TAB_LABEL_LINE_HEIGHT
        + provider_tab_bar_area_height()
}

#[cfg(test)]
pub(crate) fn provider_tab_card_inner_width() -> f32 {
    let columns = f32::from(u8::try_from(PROVIDER_TAB_MAX_COLUMNS).unwrap_or(u8::MAX));
    (POPUP_WIDTH - (columns - 1.0) * PROVIDER_TAB_ROW_SPACING) / columns
        - 2.0 * f32::from(PROVIDER_TAB_HORIZONTAL_PADDING)
}

#[cfg(test)]
pub(crate) fn longest_provider_label_glyphs() -> f32 {
    let glyphs = ProviderId::ALL
        .iter()
        .map(|provider| provider.label().chars().count())
        .max()
        .unwrap_or(0);
    f32::from(u8::try_from(glyphs).unwrap_or(u8::MAX))
}

pub(crate) fn provider_nav_height(state: &AppState) -> f32 {
    let enabled = state
        .providers
        .iter()
        .filter(|provider| provider.enabled)
        .count();
    let rows = provider_tab_row_sizes(enabled).len().max(1) as f32;
    rows * PROVIDER_TAB_ROW_HEIGHT + (rows - 1.0) * PROVIDER_TAB_ROW_SPACING
}

fn provider_nav(state: &AppState, selected_provider: ProviderId) -> Element<'static, Message> {
    let enabled: Vec<&ProviderRuntimeState> = state
        .providers
        .iter()
        .filter(|provider| provider.enabled)
        .collect();
    let mut start = 0;
    provider_tab_row_sizes(enabled.len())
        .into_iter()
        .fold(column![].spacing(PROVIDER_TAB_ROW_SPACING), |tabs, size| {
            let tab_row = enabled[start..start + size].iter().fold(
                row![].spacing(PROVIDER_TAB_ROW_SPACING),
                |tab_row, provider| {
                    tab_row.push(provider_tab(
                        state,
                        provider,
                        provider.provider == selected_provider,
                    ))
                },
            );
            start += size;
            tabs.push(tab_row)
        })
        .into()
}

fn provider_tab(
    state: &AppState,
    provider: &ProviderRuntimeState,
    selected: bool,
) -> Element<'static, Message> {
    let percents = tab_percents(state, provider);
    let icon_variant = provider_icon_variant();
    let badge = widget::icon::icon(provider_icon_handle(provider.provider, icon_variant))
        .size(PROVIDER_TAB_ICON_SIZE)
        .width(Length::Fixed(PROVIDER_TAB_ICON_LENGTH))
        .height(Length::Fixed(PROVIDER_TAB_ICON_LENGTH));
    let label = widget::text(provider.provider.label())
        .size(PROVIDER_NAV_LABEL_SIZE)
        .line_height(cosmic::iced::core::text::LineHeight::Absolute(
            PROVIDER_TAB_LABEL_LINE_HEIGHT.into(),
        ))
        .width(Length::Fill)
        .align_x(Alignment::Center);
    let bars = container(percents.into_iter().fold(
        column![].spacing(PROVIDER_TAB_ITEM_SPACING),
        |col, pct| {
            col.push(
                progress_bar(0.0..=100.0, pct)
                    .length(Length::Fill)
                    .girth(Length::Fixed(PROVIDER_TAB_BAR_GIRTH)),
            )
        },
    ))
    .width(Length::Fill)
    .height(Length::Fixed(PROVIDER_TAB_BAR_AREA_HEIGHT));

    let content = container(
        column![badge, label, bars]
            .spacing(PROVIDER_TAB_ITEM_SPACING)
            .align_x(Alignment::Center)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_y(Alignment::Center)
    .padding([
        PROVIDER_TAB_VERTICAL_PADDING,
        PROVIDER_TAB_HORIZONTAL_PADDING,
    ]);

    Element::from(
        widget::button::custom(content)
            .class(provider_tab_class(selected))
            .width(Length::FillPortion(1))
            .height(Length::Fixed(PROVIDER_TAB_ROW_HEIGHT))
            .on_press(Message::SelectProvider(provider.provider)),
    )
}

fn provider_tab_class(selected: bool) -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(move |focused, theme| {
            tab_button_style(theme, selected, ButtonInteraction::idle(focused), 1.0)
        }),
        disabled: Box::new(move |theme| {
            tab_button_style(theme, selected, ButtonInteraction::idle(false), 0.45)
        }),
        hovered: Box::new(move |focused, theme| {
            tab_button_style(theme, selected, ButtonInteraction::hover(focused), 1.0)
        }),
        pressed: Box::new(move |focused, theme| {
            tab_button_style(theme, selected, ButtonInteraction::press(focused), 0.92)
        }),
    }
}

fn tab_button_style(
    theme: &cosmic::Theme,
    selected: bool,
    interaction: ButtonInteraction,
    opacity: f32,
) -> widget::button::Style {
    let cosmic = theme.cosmic();
    let mut style = widget::button::Style::new();
    let surface = &cosmic.background.component;

    let background = if selected {
        if interaction.pressed {
            surface.divider.into()
        } else {
            accent_selection_fill(theme)
        }
    } else if interaction.pressed {
        surface.divider.into()
    } else if interaction.hovered {
        cosmic.background.component.hover.into()
    } else {
        surface.base.into()
    };

    style.background = Some(Background::Color(apply_alpha(background, opacity)));
    style.border_radius = cosmic.corner_radii.radius_s.into();
    style.border_width = if selected { 2.0 } else { 1.0 };
    style.border_color = if selected {
        apply_alpha(cosmic.accent.base.into(), opacity)
    } else {
        apply_alpha(surface.divider.into(), opacity)
    };
    style.outline_width = if interaction.focused && selected {
        1.0
    } else {
        0.0
    };
    style.outline_color = cosmic.accent.base.into();
    style.text_color = Some(apply_alpha(surface.on.into(), opacity));
    style.icon_color = Some(apply_alpha(surface.on.into(), opacity));

    style
}

fn provider_summary(provider: &ProviderRuntimeState) -> Element<'static, Message> {
    let title = row![
        widget::icon::icon(provider_icon_handle(
            provider.provider,
            provider_icon_variant(),
        ))
        .size(24)
        .width(Length::Fixed(24.0))
        .height(Length::Fixed(24.0)),
        widget::text(provider.provider.label()).size(28),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    card(title)
}

fn info_block(
    title: String,
    primary: String,
    secondary: Option<String>,
    action: Option<Element<'static, Message>>,
) -> Element<'static, Message> {
    let mut col = column![widget::text(title).size(18), widget::text(primary).size(14)].spacing(6);

    if let Some(secondary) = secondary {
        col = col.push(widget::text(secondary).size(13));
    }

    if let Some(action) = action {
        col = col.push(action);
    }

    card(col)
}

fn selected_state(
    state: &AppState,
    selected_provider: ProviderId,
) -> Option<&ProviderRuntimeState> {
    state
        .providers
        .iter()
        .find(|p| p.provider == selected_provider && p.enabled)
        .or_else(|| state.providers.iter().find(|p| p.enabled))
}

pub(crate) fn account_page_next(page: usize, account_count: usize) -> usize {
    if account_count == 0 {
        0
    } else {
        (page + 1) % account_count
    }
}

pub(crate) fn account_page_previous(page: usize, account_count: usize) -> usize {
    if account_count == 0 {
        0
    } else if page == 0 {
        account_count - 1
    } else {
        page - 1
    }
}

pub(crate) fn clamp_account_page(page: usize, account_count: usize) -> usize {
    if account_count == 0 {
        0
    } else {
        page.min(account_count - 1)
    }
}

pub(crate) fn pager_account_label(page: usize, label: &str) -> String {
    if label.trim().is_empty() {
        fl!(
            "account-pager-fallback",
            n = i64::try_from(page + 1).unwrap_or(i64::MAX)
        )
    } else {
        label.to_string()
    }
}

pub(crate) fn tab_percents(state: &AppState, provider: &ProviderRuntimeState) -> Vec<f32> {
    let now = chrono::Utc::now();
    let accounts = state.display_selected_accounts(provider.provider);
    if accounts.len() > 1 {
        return accounts
            .iter()
            .take(PROVIDER_TAB_MAX_BARS)
            .map(|account| {
                account
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.headline_window())
                    .map_or(0.0, |window| usage_display::displayed_percent(window, now))
            })
            .collect();
    }
    let Some(windows) = active_snapshot(state, provider).and_then(|s| s.applet_windows()) else {
        return vec![0.0];
    };
    let mut percents = vec![usage_display::displayed_percent(windows.primary, now)];
    if let Some(secondary) = windows.secondary {
        percents.push(usage_display::displayed_percent(secondary, now));
    }
    percents
}
