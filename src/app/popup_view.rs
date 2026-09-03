// SPDX-License-Identifier: MPL-2.0

mod badges;
mod detail;
mod measure;
mod settings;

use self::badges::{
    account_label_text, apply_alpha, badge_accent, badge_destructive, badge_destructive_soft,
    badge_neutral, badge_neutral_soft, badge_success, badge_success_soft, badge_warning,
    badge_warning_soft, badge_with_tooltip, disabled_account_label_text, plan_badge,
};
#[cfg(test)]
use self::detail::active_snapshot;
use self::detail::{empty_state_view, provider_body_height_for_page, selected_provider_view};
use self::measure::Measure;
use self::settings::{
    about_view, general_settings_view, manage_providers_view, provider_settings_view,
    settings_body_height,
};
use super::provider_assets::{provider_icon_handle, provider_icon_variant};
use crate::app::{Message, PopupRoute};
use crate::config::{Config, PanelIconStyle, ResetTimeFormat, UsageAmountFormat};
use crate::detection::DetectionSnapshot;
use crate::fl;
use crate::model::{
    AppState, ProviderAccountRuntimeState, ProviderId, ProviderRuntimeState, UsageWindow,
};
use crate::providers::antigravity::{AntigravityLoginState, AntigravityLoginStatus};
use crate::providers::claude::{ClaudeLoginState, ClaudeLoginStatus};
use crate::providers::codex::{CodexLoginState, CodexLoginStatus};
use crate::providers::copilot::{CopilotLoginState, CopilotLoginStatus};
use crate::providers::cursor::CursorScanState;
use crate::providers::gemini::{GeminiLoginState, GeminiLoginStatus};
use crate::providers::interface::ProviderAccountActionSupport;
use crate::providers::kimi::login::KimiLoginState;
use crate::providers::minimax::MinimaxLoginState;
use crate::providers::registry;
use crate::updates::UpdateStatus;
use crate::usage_display;
use cosmic::Element;
use cosmic::iced::widget::{column, container, progress_bar, row, scrollable};
use cosmic::iced::{Alignment, Background, Color, ContentFit, Length, Size};
use cosmic::widget;

pub const POPUP_COLUMN_WIDTH: f32 = 420.0;
const POPUP_WIDTH: f32 = POPUP_COLUMN_WIDTH;
const POPUP_MAX_HEIGHT: f32 = 1080.0;
const POPUP_PADDING: f32 = 32.0;
const POPUP_HEADER_HEIGHT: f32 = 36.0;
const POPUP_TAB_HEIGHT: f32 = 48.0;
pub(crate) const PROVIDER_VIEWPORT_SIZE: usize = 6;
const POPUP_BODY_PANEL_PADDING: f32 = 24.0;
const POPUP_BODY_BOTTOM_SLACK: f32 = 8.0;
const EMPTY_STATE_BODY_HEIGHT: f32 = 240.0;
const PROVIDER_CARD_SPACING: f32 = 8.0;
const ACCOUNT_PAGER_HEIGHT: f32 = 40.0;
const PROVIDER_SUMMARY_HEIGHT: f32 = 58.0;
const PROVIDER_ACCOUNT_HEADER_HEIGHT: f32 = 96.0;
const PROVIDER_SECTION_HEIGHT: f32 = 84.0;
const PROVIDER_SECTION_WITH_ACTION_HEIGHT: f32 = 120.0;
const PROVIDER_GROUP_HEADER_HEIGHT: f32 = 28.0;
const PROVIDER_GROUP_PADDING: f32 = 24.0;
const PROVIDER_GROUP_SPACING: f32 = 12.0;
const PROVIDER_CARD_PADDING: f32 = 16.0;
const SETTINGS_SECTION_HEIGHT: f32 = 104.0;
const SETTINGS_PROVIDER_ROW_HEIGHT: f32 = 44.0;
const PROVIDER_PICKER_TILE_HEIGHT: f32 = 104.0;
const PROVIDER_PICKER_COMPACT_TILE_HEIGHT: f32 = 48.0;
const PROVIDER_PICKER_HEIGHT: f32 = 680.0;
const UPDATE_NOTIFICATION_DOT_COLOR: Color = Color::from_rgb(0.93, 0.11, 0.15);
const ACCENT_SOFT_FILL_ALPHA: f32 = 0.14;

#[derive(Clone, Copy)]
pub struct ProviderLoginStates<'a> {
    pub provider_picker_open: bool,
    pub codex: Option<&'a CodexLoginState>,
    pub claude: Option<&'a ClaudeLoginState>,
    pub cursor_scan: &'a CursorScanState,
    pub gemini: Option<&'a GeminiLoginState>,
    pub copilot: Option<&'a CopilotLoginState>,
    pub minimax: Option<&'a MinimaxLoginState>,
    pub kimi: Option<&'a KimiLoginState>,
    pub antigravity: Option<&'a AntigravityLoginState>,
    pub opencode_go: Option<&'a crate::providers::opencode_go::login::OpenCodeGoLoginState>,
}

#[derive(Clone, Copy)]
pub struct DetailSelection {
    pub provider: ProviderId,
    pub account_page: usize,
    pub provider_viewport_offset: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopupBodyMeasureTarget {
    Provider(ProviderId),
    EmptyState,
    Route(PopupRoute),
}

pub fn popup_content<'a>(
    state: &'a AppState,
    config: &'a Config,
    detection: &'a DetectionSnapshot,
    logins: ProviderLoginStates<'a>,
    selection: DetailSelection,
    route: &'a PopupRoute,
    update_status: &'a UpdateStatus,
) -> Element<'a, Message> {
    let empty_state = popup_empty_state_active(state);

    let picker_open =
        logins.provider_picker_open && matches!(route, PopupRoute::ProviderDetail) && !empty_state;
    let header = popup_header(route, empty_state, update_status);

    let nav_row: Option<Element<'_, Message>> = match route {
        PopupRoute::ProviderDetail if empty_state => None,
        PopupRoute::ProviderDetail if !picker_open => {
            (enabled_provider_count(state) > 1).then(|| {
                provider_tab_rows(
                    state,
                    selection.provider,
                    selection.provider_viewport_offset,
                )
            })
        }
        PopupRoute::ProviderDetail => None,
        PopupRoute::Settings
        | PopupRoute::ManageProviders
        | PopupRoute::ManageAccounts(_)
        | PopupRoute::About => None,
    };

    let body = if picker_open {
        provider_picker_view(state, detection)
    } else {
        popup_body_view(
            state,
            config,
            detection,
            logins,
            selection,
            route,
            update_status,
        )
    };

    let body = if matches!(route, PopupRoute::Settings | PopupRoute::About) {
        body
    } else {
        scrollable(body).width(Length::Fill).into()
    };
    let body_panel: Element<'_, Message> = container(panel(body))
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    let body_stack = popup_body_stack(
        state,
        config,
        detection,
        logins,
        update_status,
        selection,
        body_panel,
    );

    let mut chrome = column![narrow_chrome(header)].spacing(14);
    if let Some(nav_row) = nav_row {
        chrome = chrome.push(narrow_chrome(nav_row));
    }
    let body_spacing = if matches!(route, PopupRoute::ProviderDetail) {
        6
    } else {
        14
    };
    let content = column![chrome, body_stack]
        .spacing(body_spacing)
        .padding(16)
        .width(Length::Fill)
        .height(Length::Fill);

    Element::from(content)
}

fn popup_body_view<'a>(
    state: &'a AppState,
    config: &'a Config,
    detection: &'a DetectionSnapshot,
    logins: ProviderLoginStates<'a>,
    selection: DetailSelection,
    route: &'a PopupRoute,
    update_status: &'a UpdateStatus,
) -> Element<'a, Message> {
    match route {
        PopupRoute::ProviderDetail if popup_empty_state_active(state) => empty_state_view(),
        PopupRoute::ProviderDetail => selected_provider_view(
            selected_state(state, selection.provider),
            state,
            config,
            detection,
            selection.account_page,
        ),
        PopupRoute::Settings => general_settings_view(config),
        PopupRoute::ManageProviders => manage_providers_view(state),
        PopupRoute::ManageAccounts(id) => {
            provider_settings_view(state, config, detection, logins, *id)
        }
        PopupRoute::About => about_view(update_status),
    }
}

fn popup_body_stack<'a>(
    state: &'a AppState,
    config: &'a Config,
    detection: &'a DetectionSnapshot,
    logins: ProviderLoginStates<'a>,
    update_status: &'a UpdateStatus,
    selection: DetailSelection,
    body_panel: Element<'a, Message>,
) -> Element<'a, Message> {
    let mut stack = cosmic::iced::widget::Stack::new()
        .push(body_panel)
        .width(Length::Fill)
        .height(Length::Fill);

    if popup_empty_state_active(state) {
        stack = stack.push(Measure::new(
            empty_state_view(),
            body_measure_width(1.0),
            |size| Message::PopupBodyMeasured(PopupBodyMeasureTarget::EmptyState, size),
        ));
    }

    if let Some(provider) = selected_state(state, selection.provider) {
        let provider_id = provider.provider;
        let width = body_measure_width(1.0);
        let body = selected_provider_view(
            Some(provider),
            state,
            config,
            detection,
            selection.account_page,
        );
        stack = stack.push(Measure::new(body, width, move |size| {
            Message::PopupBodyMeasured(PopupBodyMeasureTarget::Provider(provider_id), size)
        }));
    }

    let general = general_settings_view(config);
    stack = stack.push(Measure::new(general, body_measure_width(1.0), |size| {
        Message::PopupBodyMeasured(PopupBodyMeasureTarget::Route(PopupRoute::Settings), size)
    }));

    for provider in ProviderId::ALL {
        let body = provider_settings_view(state, config, detection, logins, provider);
        stack = stack.push(Measure::new(body, body_measure_width(1.0), move |size| {
            Message::PopupBodyMeasured(
                PopupBodyMeasureTarget::Route(PopupRoute::ManageAccounts(provider)),
                size,
            )
        }));
    }

    let providers = manage_providers_view(state);
    stack = stack.push(Measure::new(providers, body_measure_width(1.0), |size| {
        Message::PopupBodyMeasured(
            PopupBodyMeasureTarget::Route(PopupRoute::ManageProviders),
            size,
        )
    }));

    let about = about_view(update_status);
    stack = stack.push(Measure::new(about, body_measure_width(1.0), |size| {
        Message::PopupBodyMeasured(PopupBodyMeasureTarget::Route(PopupRoute::About), size)
    }));

    stack.into()
}

pub fn popup_max_width(_state: &AppState) -> f32 {
    POPUP_WIDTH
}

pub fn popup_session_size(state: &AppState, selected_provider: ProviderId) -> Size {
    popup_session_size_for_page(state, selected_provider, 0)
}

pub fn popup_session_size_for_page(
    state: &AppState,
    selected_provider: ProviderId,
    account_page: usize,
) -> Size {
    if popup_empty_state_active(state) {
        return popup_empty_state_size(EMPTY_STATE_BODY_HEIGHT);
    }
    let provider_height = provider_body_height_for_page(
        state,
        selected_state(state, selected_provider),
        account_page,
    );
    Size::new(
        POPUP_WIDTH,
        popup_total_height(provider_nav_height(state), provider_height),
    )
}

pub const fn popup_provider_picker_size() -> Size {
    Size::new(POPUP_WIDTH, PROVIDER_PICKER_HEIGHT)
}

pub fn popup_session_size_with_body_height(
    state: &AppState,
    _selected_provider: ProviderId,
    body_height: f32,
) -> Size {
    if popup_empty_state_active(state) {
        return popup_empty_state_size(body_height);
    }
    Size::new(
        POPUP_WIDTH,
        popup_total_height(provider_nav_height(state), body_height),
    )
}

pub fn popup_settings_size(state: &AppState) -> Size {
    Size::new(
        POPUP_WIDTH,
        popup_total_height(None, settings_body_height(state)),
    )
}

pub fn popup_settings_size_with_body_height(body_height: f32) -> Size {
    Size::new(POPUP_WIDTH, popup_total_height(None, body_height))
}

fn popup_total_height(nav_height: Option<f32>, body_height: f32) -> f32 {
    let chrome_spacing = if nav_height.is_some() { 20.0 } else { 6.0 };
    let height = POPUP_PADDING
        + chrome_spacing
        + POPUP_HEADER_HEIGHT
        + nav_height.unwrap_or(0.0)
        + POPUP_BODY_PANEL_PADDING
        + POPUP_BODY_BOTTOM_SLACK
        + body_height;
    height.clamp(1.0, POPUP_MAX_HEIGHT)
}

fn popup_empty_state_size(body_height: f32) -> Size {
    let height = POPUP_PADDING
        + 6.0
        + POPUP_HEADER_HEIGHT
        + POPUP_BODY_PANEL_PADDING
        + POPUP_BODY_BOTTOM_SLACK
        + body_height;
    Size::new(POPUP_WIDTH, height.clamp(1.0, POPUP_MAX_HEIGHT))
}

pub(super) fn popup_empty_state_active(state: &AppState) -> bool {
    state.providers.iter().all(|provider| !provider.enabled)
}

pub(super) fn detected_without_accounts(
    state: &AppState,
    detection: &DetectionSnapshot,
    provider: ProviderId,
) -> bool {
    detection.detected(provider) && state.accounts_for(provider).is_empty()
}

fn body_measure_width(_columns: f32) -> f32 {
    POPUP_WIDTH - POPUP_PADDING - POPUP_BODY_PANEL_PADDING
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

fn narrow_chrome<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(container(content.into()).width(Length::Fixed(POPUP_WIDTH)))
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
}

fn panel<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    Element::from(container(content).width(Length::Fill).padding(12))
}

fn popup_header(
    route: &PopupRoute,
    empty_state: bool,
    update_status: &UpdateStatus,
) -> Element<'static, Message> {
    if !matches!(route, PopupRoute::ProviderDetail) {
        return widget::button::text(fl!("back"))
            .leading_icon(widget::icon::from_name("go-previous-symbolic"))
            .class(back_button_class())
            .on_press(Message::NavigateTo(PopupRoute::ProviderDetail))
            .into();
    }

    let mut header = row![
        widget::button::custom(widget::text(fl!("app-title")).size(22))
            .class(header_title_button_class())
            .padding(0)
            .on_press(Message::NavigateTo(PopupRoute::ProviderDetail)),
        cosmic::iced::widget::Space::new().width(Length::Fill),
    ]
    .align_y(Alignment::Center)
    .spacing(12);

    let about = header_icon_button(
        "dialog-information-symbolic",
        Message::NavigateTo(PopupRoute::About),
    );
    let about: Element<'static, Message> = if update_available(update_status) {
        container(
            row![about, notification_dot(8.0)]
                .spacing(2)
                .align_y(Alignment::Center),
        )
        .into()
    } else {
        about
    };
    let mut actions = row![
        about,
        header_icon_button(
            "view-list-symbolic",
            Message::NavigateTo(PopupRoute::ManageProviders),
        ),
        header_icon_button(
            "preferences-system-symbolic",
            Message::NavigateTo(PopupRoute::Settings),
        ),
    ]
    .spacing(7)
    .align_y(Alignment::Center);
    if !empty_state {
        actions = actions.push(widget::tooltip::tooltip(
            header_icon_button("view-refresh-symbolic", Message::RefreshNow),
            widget::text(fl!("refresh-now")).size(12),
            widget::tooltip::Position::Top,
        ));
    }
    header = header.push(actions);

    header.into()
}

fn header_icon_button(icon_name: &'static str, message: Message) -> Element<'static, Message> {
    widget::button::icon(widget::icon::from_name(icon_name))
        .extra_small()
        .padding(3)
        .class(header_icon_button_class())
        .on_press(message)
        .into()
}

fn header_icon_button_class() -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(|_focused, theme| header_icon_button_style(theme, false)),
        disabled: Box::new(|theme| header_icon_button_style(theme, false)),
        hovered: Box::new(|_focused, theme| header_icon_button_style(theme, true)),
        pressed: Box::new(|_focused, theme| header_icon_button_style(theme, true)),
    }
}

fn header_icon_button_style(theme: &cosmic::Theme, hovered: bool) -> widget::button::Style {
    let cosmic = theme.cosmic();
    let mut style = widget::button::Style::new();
    let surface = &cosmic.background(theme.transparent).component;
    style.background = hovered.then(|| Background::Color(surface.hover.into()));
    style.border_radius = cosmic.corner_radii.radius_s.into();
    style.icon_color = Some(apply_alpha(
        surface.on.into(),
        if hovered { 0.68 } else { 0.40 },
    ));
    style.text_color = style.icon_color;
    style
}

fn header_title_button_class() -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(|_focused, theme| header_title_button_style(theme)),
        disabled: Box::new(header_title_button_style),
        hovered: Box::new(|_focused, theme| header_title_button_style(theme)),
        pressed: Box::new(|_focused, theme| header_title_button_style(theme)),
    }
}

fn header_title_button_style(theme: &cosmic::Theme) -> widget::button::Style {
    let cosmic = theme.cosmic();
    let mut style = widget::button::Style::new();
    let foreground = cosmic.background(theme.transparent).on.into();
    style.text_color = Some(foreground);
    style.icon_color = Some(foreground);
    style
}

fn back_button_class() -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(|_focused, _theme| back_button_style()),
        disabled: Box::new(|_theme| back_button_style()),
        hovered: Box::new(|_focused, _theme| back_button_style()),
        pressed: Box::new(|_focused, _theme| back_button_style()),
    }
}

fn back_button_style() -> widget::button::Style {
    let mut style = widget::button::Style::new();
    style.text_color = Some(Color::WHITE);
    style.icon_color = Some(Color::WHITE);
    style
}

fn provider_picker_providers(state: &AppState, detection: &DetectionSnapshot) -> Vec<ProviderId> {
    let mut providers = ProviderId::ALL.to_vec();
    providers.sort_by_key(|provider| !detected_without_accounts(state, detection, *provider));
    providers
}

fn provider_picker_view(
    state: &AppState,
    detection: &DetectionSnapshot,
) -> Element<'static, Message> {
    let providers = provider_picker_providers(state, detection);
    let detected_count = providers
        .iter()
        .take_while(|provider| detected_without_accounts(state, detection, **provider))
        .count();
    let (detected, remaining) = providers.split_at(detected_count);
    let mut content = column![].spacing(5).width(Length::Fill);

    if !detected.is_empty() {
        content = content
            .push(widget::text(fl!("provider-picker-detected-section")).size(12))
            .push(provider_picker_tile_rows(detected, true));
    }

    if !remaining.is_empty() {
        content = content
            .push(widget::text(fl!("provider-picker-all-section")).size(12))
            .push(provider_picker_tile_rows(remaining, false));
    }

    container(content).padding(4).into()
}

fn provider_picker_tile_rows(
    providers: &[ProviderId],
    detected: bool,
) -> Element<'static, Message> {
    let mut rows = column![].spacing(8);
    for pair in providers.chunks(2) {
        let mut row = row![].spacing(8);
        for provider in pair {
            row = row.push(provider_picker_tile(*provider, detected));
        }
        if pair.len() == 1 {
            row = row.push(cosmic::iced::widget::Space::new().width(Length::FillPortion(1)));
        }
        rows = rows.push(row);
    }
    rows.into()
}

fn provider_picker_tile(provider: ProviderId, detected: bool) -> Element<'static, Message> {
    let content: Element<'static, Message> = if detected {
        column![
            provider_picker_icon(provider),
            widget::text(provider.label()).size(14),
            widget::text(fl!("provider-picker-connect-account")).size(12),
        ]
        .spacing(8)
        .width(Length::Fill)
        .into()
    } else {
        row![
            provider_picker_icon(provider),
            widget::text(provider.label()).size(14),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
    };
    let height = if detected {
        PROVIDER_PICKER_TILE_HEIGHT
    } else {
        PROVIDER_PICKER_COMPACT_TILE_HEIGHT
    };
    let padding = if detected { [12, 12] } else { [4, 12] };
    let content = container(content).padding(padding);
    let content = if detected {
        content
    } else {
        content.height(Length::Fill).align_y(Alignment::Center)
    };
    widget::button::custom(content)
        .class(provider_tab_class(false))
        .width(Length::FillPortion(1))
        .height(Length::Fixed(height))
        .on_press(Message::OpenProviderPickerProvider(provider))
        .into()
}

fn provider_picker_icon(provider: ProviderId) -> Element<'static, Message> {
    widget::icon::icon(provider_icon_handle(provider, provider_icon_variant()))
        .size(22)
        .width(Length::Fixed(22.0))
        .height(Length::Fixed(22.0))
        .into()
}

fn card<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    Element::from(container(content).width(Length::Fill).padding(8))
}

fn accent_selection_fill(theme: &cosmic::Theme) -> Color {
    let cosmic = theme.cosmic();
    apply_alpha(cosmic.accent.base.into(), ACCENT_SOFT_FILL_ALPHA)
}

fn provider_tab_selection_fill(theme: &cosmic::Theme) -> Color {
    let cosmic = theme.cosmic();
    apply_alpha(
        cosmic.background(theme.transparent).component.on.into(),
        0.12,
    )
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
            text_color: Some(apply_alpha(
                cosmic.background(theme.transparent).on.into(),
                0.45,
            )),
            background: Some(Background::Color(apply_alpha(
                cosmic.background(theme.transparent).component.base.into(),
                0.45,
            ))),
            border: cosmic::iced::Border {
                radius: cosmic.corner_radii.radius_s.into(),
                width: 1.0,
                color: apply_alpha(cosmic.background(theme.transparent).divider.into(), 0.45),
            },
            shadow: cosmic::iced::Shadow::default(),
            icon_color: Some(apply_alpha(
                cosmic.background(theme.transparent).on.into(),
                0.45,
            )),
            snap: true,
        }
    }))
}

fn update_available(update_status: &UpdateStatus) -> bool {
    matches!(update_status, UpdateStatus::UpdateAvailable { .. })
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
                width: 1.0,
                color: Color::WHITE,
            },
            shadow: cosmic::iced::Shadow {
                color: apply_alpha(UPDATE_NOTIFICATION_DOT_COLOR, 0.72),
                offset: cosmic::iced::Vector::new(0.0, 0.0),
                blur_radius: 4.0,
            },
            icon_color: None,
            snap: true,
        }),
    )
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

fn provider_tab_rows(
    state: &AppState,
    selected_provider: ProviderId,
    offset: usize,
) -> Element<'static, Message> {
    let providers: Vec<_> = state
        .providers
        .iter()
        .filter(|provider| provider.enabled)
        .map(|provider| provider.provider)
        .collect();
    let offset = offset.min(provider_viewport_max_offset(providers.len()));
    let mut tabs = row![].width(Length::Fill).align_y(Alignment::Center);
    for provider in provider_viewport(&providers, offset) {
        tabs = tabs.push(provider_viewport_tab(
            *provider,
            *provider == selected_provider,
        ));
    }
    for _ in provider_viewport(&providers, offset).len()..PROVIDER_VIEWPORT_SIZE {
        tabs = tabs.push(provider_viewport_spacer());
    }

    if provider_viewport_navigation_visible(providers.len()) {
        let previous =
            widget::button::icon(widget::icon::from_name("go-previous-symbolic")).extra_small();
        let previous = if offset == 0 {
            previous
        } else {
            previous.on_press(Message::PageProviderViewport(
                crate::app::PagerDirection::Previous,
            ))
        };
        let next = widget::button::icon(widget::icon::from_name("go-next-symbolic")).extra_small();
        let next = if offset >= provider_viewport_max_offset(providers.len()) {
            next
        } else {
            next.on_press(Message::PageProviderViewport(
                crate::app::PagerDirection::Next,
            ))
        };

        row![previous, tabs, next]
            .spacing(6)
            .width(Length::Fill)
            .align_y(Alignment::Center)
            .into()
    } else {
        tabs.into()
    }
}

pub(crate) fn provider_viewport(providers: &[ProviderId], offset: usize) -> &[ProviderId] {
    let start = offset.min(provider_viewport_max_offset(providers.len()));
    let end = (start + PROVIDER_VIEWPORT_SIZE).min(providers.len());
    &providers[start..end]
}

fn provider_viewport_max_offset(provider_count: usize) -> usize {
    provider_count.saturating_sub(PROVIDER_VIEWPORT_SIZE)
}

pub(crate) fn provider_viewport_navigation_visible(provider_count: usize) -> bool {
    provider_count >= PROVIDER_VIEWPORT_SIZE
}

pub(crate) fn provider_viewport_max_offset_for(state: &AppState) -> usize {
    provider_viewport_max_offset(enabled_provider_count(state))
}

fn enabled_provider_count(state: &AppState) -> usize {
    state
        .providers
        .iter()
        .filter(|provider| provider.enabled)
        .count()
}

fn provider_nav_height(state: &AppState) -> Option<f32> {
    let count = enabled_provider_count(state);
    (count > 1).then_some(POPUP_TAB_HEIGHT)
}

fn provider_viewport_tab(provider: ProviderId, selected: bool) -> Element<'static, Message> {
    let icon_variant = provider_icon_variant();
    let icon = widget::icon::icon(provider_icon_handle(provider, icon_variant))
        .size(22)
        .width(Length::Fixed(22.0))
        .height(Length::Fixed(22.0))
        .content_fit(ContentFit::Contain);
    let icon = container(icon)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);
    let baseline_height = provider_viewport_baseline_height(selected);

    column![
        widget::button::custom(icon)
            .class(provider_tab_class(selected))
            .width(Length::FillPortion(1))
            .height(Length::Fixed(POPUP_TAB_HEIGHT - baseline_height))
            .on_press(Message::SelectProvider(provider)),
        provider_viewport_baseline(selected),
    ]
    .width(Length::FillPortion(1))
    .into()
}

fn provider_viewport_spacer() -> Element<'static, Message> {
    let baseline_height = provider_viewport_baseline_height(false);
    column![
        cosmic::iced::widget::Space::new()
            .height(Length::Fixed(POPUP_TAB_HEIGHT - baseline_height)),
        provider_viewport_baseline(false),
    ]
    .width(Length::FillPortion(1))
    .into()
}

fn provider_viewport_baseline_height(selected: bool) -> f32 {
    if selected { 4.0 } else { 2.0 }
}

fn provider_viewport_baseline(selected: bool) -> Element<'static, Message> {
    container(
        cosmic::iced::widget::Space::new()
            .height(Length::Fixed(provider_viewport_baseline_height(selected))),
    )
    .width(Length::Fill)
    .style(move |theme: &cosmic::Theme| {
        let cosmic = theme.cosmic();
        let color = if selected {
            cosmic.background(theme.transparent).component.on.into()
        } else {
            cosmic
                .background(theme.transparent)
                .component
                .divider
                .into()
        };
        widget::container::Style {
            text_color: None,
            background: Some(Background::Color(color)),
            border: cosmic::iced::Border::default(),
            shadow: cosmic::iced::Shadow::default(),
            icon_color: None,
            snap: true,
        }
    })
    .into()
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
    let surface = &cosmic.background(theme.transparent).component;

    let background = if selected {
        if interaction.pressed {
            Some(surface.divider.into())
        } else {
            Some(provider_tab_selection_fill(theme))
        }
    } else if interaction.pressed {
        Some(surface.divider.into())
    } else if interaction.hovered {
        Some(cosmic.background(theme.transparent).component.hover.into())
    } else {
        None
    };

    style.background = background.map(|color| Background::Color(apply_alpha(color, opacity)));
    style.border_radius = cosmic::iced::border::top(8.0);
    style.border_width = 0.0;
    style.border_color = Color::TRANSPARENT;
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

fn provider_summary(
    provider: &ProviderRuntimeState,
    detected_without_accounts: bool,
) -> Element<'static, Message> {
    let mut title = row![
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

    if detected_without_accounts {
        title = title.push(badge_accent(fl!("provider-detected-chip")));
    }

    card(title)
}

fn info_block(
    title: String,
    primary: String,
    secondary: Option<String>,
    action: Option<Element<'static, Message>>,
) -> Element<'static, Message> {
    let mut col = column![widget::text(title).size(15), widget::text(primary).size(14)].spacing(6);

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

#[cfg(test)]
fn tab_percents(
    state: &AppState,
    provider: &ProviderRuntimeState,
    usage_amount_format: UsageAmountFormat,
) -> Vec<f32> {
    let now = chrono::Utc::now();
    let accounts = state.display_selected_accounts(provider.provider);
    if accounts.is_empty() {
        let pct = active_snapshot(state, provider)
            .and_then(|s| s.headline_window())
            .map_or(0.0, |w| {
                usage_display::displayed_amount_percent(w, now, usage_amount_format)
            });
        return vec![pct];
    }
    accounts
        .into_iter()
        .map(|account| {
            account
                .snapshot
                .as_ref()
                .and_then(|s| s.headline_window())
                .map_or(0.0, |w| {
                    usage_display::displayed_amount_percent(w, now, usage_amount_format)
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_tab_percents_respect_usage_amount_format() {
        let mut state = AppState::empty();
        let account_id = "codex-test";
        state
            .provider_mut(ProviderId::Codex)
            .unwrap()
            .selected_account_ids = vec![account_id.to_string()];
        let mut account =
            ProviderAccountRuntimeState::empty(ProviderId::Codex, account_id, "test@example.com");
        account.snapshot = Some(crate::model::UsageSnapshot {
            provider: ProviderId::Codex,
            source: "test".to_string(),
            updated_at: chrono::Utc::now(),
            headline: crate::model::UsageHeadline(0),
            windows: vec![UsageWindow {
                label: "Session".to_string(),
                used_percent: 25.0,
                reset_at: None,
                window_seconds: None,
                reset_description: None,
                group: None,
            }],
            provider_cost: None,
            extra_usage: None,
            identity: crate::model::ProviderIdentity::default(),
        });
        state.upsert_account(account);
        let provider = state.provider(ProviderId::Codex).unwrap();

        assert_eq!(
            tab_percents(&state, provider, UsageAmountFormat::Used),
            vec![25.0]
        );
        assert_eq!(
            tab_percents(&state, provider, UsageAmountFormat::Left),
            vec![75.0]
        );
    }

    #[test]
    fn header_title_uses_normal_foreground_color() {
        let theme = cosmic::Theme::default();
        let expected = theme.cosmic().background(theme.transparent).on.into();

        assert_eq!(header_title_button_style(&theme).text_color, Some(expected));
    }

    #[test]
    fn empty_state_is_active_without_enabled_provider_tabs() {
        let mut state = AppState::empty();
        for provider in ProviderId::ALL {
            state.provider_mut(provider).unwrap().enabled = false;
        }

        assert!(popup_empty_state_active(&state));
        assert_eq!(
            popup_session_size(&state, ProviderId::Codex).width,
            POPUP_WIDTH
        );

        state.provider_mut(ProviderId::Codex).unwrap().enabled = true;

        assert!(!popup_empty_state_active(&state));
    }

    #[test]
    fn detected_settings_hint_ignores_explicit_disablement_but_hides_after_account_added() {
        let home = tempfile::tempdir().expect("create temporary home");
        std::fs::create_dir(home.path().join(".codex")).expect("create Codex marker");
        let detection = crate::detection::detect(home.path());
        let mut state = AppState::empty();
        state.provider_mut(ProviderId::Codex).unwrap().enabled = false;

        assert!(detected_without_accounts(
            &state,
            &detection,
            ProviderId::Codex
        ));

        state
            .provider_accounts
            .push(ProviderAccountRuntimeState::empty(
                ProviderId::Codex,
                "codex-test",
                "test@example.com",
            ));
        assert!(!detected_without_accounts(
            &state,
            &detection,
            ProviderId::Codex
        ));
    }

    #[test]
    fn provider_picker_lists_detected_unconfigured_providers_first() {
        let home = tempfile::tempdir().expect("create temporary home");
        std::fs::create_dir(home.path().join(".codex")).expect("create Codex marker");
        let detection = crate::detection::detect(home.path());
        let state = AppState::empty();

        assert_eq!(
            provider_picker_providers(&state, &detection),
            vec![
                ProviderId::Codex,
                ProviderId::Claude,
                ProviderId::Cursor,
                ProviderId::Antigravity,
                ProviderId::Gemini,
                ProviderId::Copilot,
                ProviderId::Minimax,
                ProviderId::Kimi,
                ProviderId::OpenCodeGo,
            ]
        );
    }

    #[test]
    fn provider_picker_has_room_for_the_complete_chooser() {
        assert_eq!(popup_provider_picker_size(), Size::new(POPUP_WIDTH, 680.0));
    }
}
