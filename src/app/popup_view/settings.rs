mod about;
mod accounts;
mod general;

use super::{
    Alignment, AppState, Background, Config, DetectionSnapshot, Element, Length, Message,
    ProviderId, ProviderLoginStates, SETTINGS_PROVIDER_ROW_HEIGHT, SETTINGS_SECTION_HEIGHT,
    UpdateStatus, container, fl, provider_icon_handle, provider_icon_variant, widget,
};

pub(super) fn general_settings_view<'a>(config: &'a Config) -> Element<'a, Message> {
    general::general_settings_view(config)
}

pub(super) fn manage_providers_view(state: &AppState) -> Element<'static, Message> {
    let providers = ProviderId::ALL;
    let mut rows = cosmic::iced::widget::column![].width(Length::Fill);
    for (index, provider_id) in providers.into_iter().enumerate() {
        let enabled = state
            .provider(provider_id)
            .is_some_and(|provider| provider.enabled);
        rows = rows.push(manage_provider_row(provider_id, enabled));
        if index + 1 < providers.len() {
            rows = rows.push(manage_provider_divider());
        }
    }
    Element::from(
        cosmic::iced::widget::column![
            cosmic::widget::text(fl!("manage-providers")).size(20),
            container(rows)
                .width(Length::Fill)
                .style(manage_provider_list_style),
        ]
        .spacing(16)
        .width(Length::Fill),
    )
}

fn manage_provider_row(provider: ProviderId, enabled: bool) -> Element<'static, Message> {
    container(
        cosmic::iced::widget::row![
            widget::icon::icon(provider_icon_handle(provider, provider_icon_variant())).size(20),
            cosmic::widget::text(provider.label()).size(16),
            cosmic::iced::widget::Space::new().width(Length::Fill),
            cosmic::widget::toggler(enabled)
                .on_toggle(move |enabled| Message::SetProviderEnabled(provider, enabled)),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .padding([12, 12])
    .width(Length::Fill)
    .into()
}

fn manage_provider_divider() -> Element<'static, Message> {
    container(cosmic::iced::widget::Space::new().height(Length::Fixed(1.0)))
        .width(Length::Fill)
        .style(|theme: &cosmic::Theme| {
            let cosmic = theme.cosmic();
            widget::container::Style {
                text_color: None,
                background: Some(Background::Color(
                    cosmic
                        .background(theme.transparent)
                        .component
                        .divider
                        .into(),
                )),
                border: cosmic::iced::Border::default(),
                shadow: cosmic::iced::Shadow::default(),
                icon_color: None,
                snap: true,
            }
        })
        .into()
}

fn manage_provider_list_style(theme: &cosmic::Theme) -> widget::container::Style {
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
}

pub(super) fn about_view(update_status: &UpdateStatus) -> Element<'static, Message> {
    about::about_view(update_status)
}

pub(super) fn provider_settings_view<'a>(
    state: &'a AppState,
    config: &'a Config,
    detection: &'a DetectionSnapshot,
    logins: ProviderLoginStates<'a>,
    provider_id: ProviderId,
) -> Element<'a, Message> {
    accounts::provider_settings_view(state, config, detection, logins, provider_id)
}

pub(super) fn settings_body_height(state: &AppState) -> f32 {
    let account_counts: Vec<usize> = ProviderId::ALL
        .into_iter()
        .map(|provider| state.accounts_for(provider).len())
        .collect();
    let max_accounts = account_counts.iter().copied().max().unwrap_or(0).max(1);
    let account_rows = f32::from(u16::try_from(max_accounts).unwrap_or(u16::MAX));
    let show_all_row = if account_counts.iter().copied().any(|count| count > 1) {
        36.0
    } else {
        0.0
    };
    let general_height = {
        let refresh = SETTINGS_SECTION_HEIGHT;
        let panel_icon = 128.0;
        let reset_time = SETTINGS_SECTION_HEIGHT;
        let usage_amount = SETTINGS_SECTION_HEIGHT;
        refresh + panel_icon + reset_time + usage_amount + 14.0
    };
    let provider_settings_height = {
        let provider_header = 40.0;
        let accounts_section =
            40.0 + account_rows * SETTINGS_PROVIDER_ROW_HEIGHT + show_all_row + 40.0;
        provider_header + accounts_section + 28.0 + 8.0
    };
    let placeholder_height = SETTINGS_SECTION_HEIGHT * 2.0 + 28.0;
    general_height
        .max(provider_settings_height)
        .max(placeholder_height)
}
