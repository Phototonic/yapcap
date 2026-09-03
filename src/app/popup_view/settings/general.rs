use super::super::{
    Alignment, Background, ButtonInteraction, Element, Length, Message, PanelIconStyle, ProviderId,
    ResetTimeFormat, UsageAmountFormat, UsageWindow, apply_alpha, container, fl, progress_bar,
    provider_icon_handle, provider_icon_variant, row, settings_block, usage_display, widget,
};

pub(super) fn general_settings_view<'a>(config: &'a crate::config::Config) -> Element<'a, Message> {
    let refresh_section = refresh_section(config.refresh_interval_seconds);
    let panel_icon_section = panel_icon_section(config.panel_icon_style);
    let reset_time_section = reset_time_section(config.reset_time_format);
    let usage_amount_section = usage_amount_section(config.usage_amount_format);
    Element::from(
        cosmic::iced::widget::column![
            refresh_section,
            panel_icon_section,
            reset_time_section,
            usage_amount_section
        ]
        .spacing(14)
        .width(Length::Fill),
    )
}

fn refresh_section(current_seconds: u64) -> Element<'static, Message> {
    let options: &[(u64, &str)] = &[(60, "1"), (300, "5"), (900, "15"), (1800, "30")];

    let buttons = options.iter().enumerate().fold(
        row![].width(Length::Fill),
        |row, (index, (secs, text))| {
            let is_selected = *secs == current_seconds;
            let content = segmented_option_content(
                index,
                is_selected,
                segmented_option_text((*text).to_string(), 12, is_selected),
                [9, 8],
                34.0,
            );
            row.push(
                widget::button::custom(content)
                    .class(segmented_option_class(
                        is_selected,
                        index == 0,
                        index + 1 == options.len(),
                    ))
                    .padding(0)
                    .on_press(Message::SetRefreshInterval(*secs))
                    .width(Length::FillPortion(1)),
            )
        },
    );

    settings_block(
        widget::text(fl!("refresh-section-title")).size(16).into(),
        segmented_options(buttons.into()),
    )
}

fn panel_icon_section(current_style: PanelIconStyle) -> Element<'static, Message> {
    let options = [
        PanelIconStyle::LogoAndBars,
        PanelIconStyle::BarsOnly,
        PanelIconStyle::LogoAndPercent,
        PanelIconStyle::PercentOnly,
    ];

    let buttons =
        options
            .iter()
            .enumerate()
            .fold(row![].width(Length::Fill), |row, (index, style)| {
                let is_selected = *style == current_style;
                let content = segmented_option_content(
                    index,
                    is_selected,
                    panel_icon_preview(*style),
                    [9, 6],
                    36.0,
                );
                let button = widget::button::custom(content)
                    .class(segmented_option_class(
                        is_selected,
                        index == 0,
                        index + 1 == options.len(),
                    ))
                    .padding(0)
                    .on_press(Message::SetPanelIconStyle(*style))
                    .width(Length::FillPortion(1));
                let content: Element<'static, Message> = if *style == PanelIconStyle::PercentOnly {
                    widget::tooltip::tooltip(
                        button,
                        widget::text(fl!("panel-icon-percent-only-tooltip")).size(12),
                        widget::tooltip::Position::Top,
                    )
                    .into()
                } else {
                    button.into()
                };

                row.push(content)
            });

    settings_block(
        widget::text(fl!("panel-icon-section-title"))
            .size(16)
            .into(),
        segmented_options(buttons.into()),
    )
}

fn panel_icon_preview(style: PanelIconStyle) -> Element<'static, Message> {
    let logo = widget::icon::icon(provider_icon_handle(
        ProviderId::Codex,
        provider_icon_variant(),
    ))
    .size(16)
    .width(Length::Fixed(16.0))
    .height(Length::Fixed(16.0));
    let bars = cosmic::iced::widget::column![
        progress_bar(0.0..=100.0, 86.5)
            .length(Length::Fixed(38.0))
            .girth(Length::Fixed(5.0)),
        progress_bar(0.0..=100.0, 42.0)
            .length(Length::Fixed(38.0))
            .girth(Length::Fixed(3.0)),
    ]
    .spacing(3)
    .width(Length::Fixed(38.0));

    let preview: Element<'static, Message> = match style {
        PanelIconStyle::LogoAndBars => row![logo, bars]
            .spacing(5)
            .align_y(Alignment::Center)
            .into(),
        PanelIconStyle::BarsOnly => bars.into(),
        PanelIconStyle::LogoAndPercent => row![logo, widget::text("86.5%").size(12)]
            .spacing(5)
            .align_y(Alignment::Center)
            .into(),
        PanelIconStyle::PercentOnly => widget::text("86.5%").size(12).into(),
    };

    container(preview)
        .height(Length::Fixed(22.0))
        .align_y(Alignment::Center)
        .into()
}

fn reset_time_section(current_format: ResetTimeFormat) -> Element<'static, Message> {
    let options = [
        (ResetTimeFormat::Relative, fl!("reset-time-relative")),
        (ResetTimeFormat::Absolute, fl!("reset-time-absolute")),
    ];
    let buttons = options.iter().enumerate().fold(
        row![].width(Length::Fill),
        |row, (index, (format, text))| {
            let is_selected = *format == current_format;
            let now = chrono::Utc::now();
            let example_window = UsageWindow {
                label: "Session".to_string(),
                used_percent: 50.0,
                reset_at: Some(now + chrono::Duration::hours(4)),
                window_seconds: None,
                reset_description: None,
                group: None,
            };
            let example = usage_display::reset_label(&example_window, now, *format)
                .unwrap_or_else(|| fl!("reset-now"));
            let content = segmented_option_content(
                index,
                is_selected,
                cosmic::iced::widget::column![
                    segmented_option_text(text.clone(), 12, is_selected),
                    segmented_option_text(example, 9, is_selected),
                ]
                .spacing(2)
                .align_x(Alignment::Center)
                .into(),
                [9, 8],
                48.0,
            );
            row.push(
                widget::button::custom(content)
                    .class(segmented_option_class(
                        is_selected,
                        index == 0,
                        index + 1 == options.len(),
                    ))
                    .padding(0)
                    .on_press(Message::SetResetTimeFormat(*format))
                    .width(Length::FillPortion(1)),
            )
        },
    );

    settings_block(
        widget::text(fl!("reset-time-section-title"))
            .size(16)
            .into(),
        segmented_options(buttons.into()),
    )
}

fn usage_amount_section(current_format: UsageAmountFormat) -> Element<'static, Message> {
    let options = [
        (UsageAmountFormat::Used, fl!("usage-amount-used")),
        (UsageAmountFormat::Left, fl!("usage-amount-left")),
    ];

    let buttons = options.iter().enumerate().fold(
        row![].width(Length::Fill),
        |row, (index, (format, text))| {
            let is_selected = *format == current_format;
            let content = segmented_option_content(
                index,
                is_selected,
                segmented_option_text(text.clone(), 12, is_selected),
                [9, 8],
                34.0,
            );
            row.push(
                widget::button::custom(content)
                    .class(segmented_option_class(
                        is_selected,
                        index == 0,
                        index + 1 == options.len(),
                    ))
                    .padding(0)
                    .on_press(Message::SetUsageAmountFormat(*format))
                    .width(Length::FillPortion(1)),
            )
        },
    );

    settings_block(
        widget::text(fl!("usage-amount-section-title"))
            .size(16)
            .into(),
        segmented_options(buttons.into()),
    )
}

fn segmented_option_text(text: String, size: u16, selected: bool) -> Element<'static, Message> {
    let text = widget::text(text).size(size);
    if selected {
        text.class(cosmic::theme::Text::Accent).into()
    } else {
        text.into()
    }
}

fn segmented_option_content(
    index: usize,
    selected: bool,
    content: Element<'static, Message>,
    padding: [u16; 2],
    divider_height: f32,
) -> Element<'static, Message> {
    let content: Element<'static, Message> = if selected {
        let checkmark: Element<'static, Message> = container(
            widget::icon::icon(widget::icon::from_name("object-select-symbolic").into()).size(12),
        )
        .style(|theme| segmented_option_content_style(theme, true))
        .into();
        container(
            row![checkmark, content]
                .spacing(5)
                .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
    } else {
        content
    };
    let content = container(content)
        .width(Length::Fill)
        .padding(padding)
        .align_x(Alignment::Center);
    let content: Element<'static, Message> = if index == 0 {
        content.into()
    } else {
        row![segmented_divider(divider_height), content]
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    };

    container(content)
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .style(move |theme| segmented_option_content_style(theme, selected))
        .into()
}

fn segmented_option_content_style(
    theme: &cosmic::Theme,
    selected: bool,
) -> widget::container::Style {
    let color = selected.then(|| theme.cosmic().accent_text_color().into());
    widget::container::Style {
        text_color: color,
        background: None,
        border: cosmic::iced::Border::default(),
        shadow: cosmic::iced::Shadow::default(),
        icon_color: color,
        snap: true,
    }
}

fn segmented_options(content: Element<'static, Message>) -> Element<'static, Message> {
    container(content)
        .width(Length::Fill)
        .style(segmented_options_style)
        .into()
}

fn segmented_divider(height: f32) -> Element<'static, Message> {
    container(cosmic::iced::widget::Space::new().width(Length::Fixed(1.0)))
        .height(Length::Fixed(height))
        .style(|theme: &cosmic::Theme| {
            let cosmic = theme.cosmic();
            widget::container::Style {
                text_color: None,
                background: Some(Background::Color(apply_alpha(
                    cosmic.background(theme.transparent).component.on.into(),
                    0.28,
                ))),
                border: cosmic::iced::Border::default(),
                shadow: cosmic::iced::Shadow::default(),
                icon_color: None,
                snap: true,
            }
        })
        .into()
}

fn segmented_options_style(theme: &cosmic::Theme) -> widget::container::Style {
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

fn segmented_option_class(selected: bool, first: bool, last: bool) -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(move |focused, theme| {
            segmented_option_style(theme, selected, focused, 1.0, first, last)
        }),
        disabled: Box::new(move |theme| {
            segmented_option_style(theme, selected, false, 0.45, first, last)
        }),
        hovered: Box::new(move |focused, theme| {
            segmented_option_interaction_style(
                theme,
                selected,
                ButtonInteraction::hover(focused),
                1.0,
                first,
                last,
            )
        }),
        pressed: Box::new(move |focused, theme| {
            segmented_option_interaction_style(
                theme,
                selected,
                ButtonInteraction::press(focused),
                0.92,
                first,
                last,
            )
        }),
    }
}

fn segmented_option_style(
    theme: &cosmic::Theme,
    selected: bool,
    focused: bool,
    opacity: f32,
    first: bool,
    last: bool,
) -> widget::button::Style {
    segmented_option_interaction_style(
        theme,
        selected,
        ButtonInteraction::idle(focused),
        opacity,
        first,
        last,
    )
}

fn segmented_option_interaction_style(
    theme: &cosmic::Theme,
    selected: bool,
    interaction: ButtonInteraction,
    opacity: f32,
    first: bool,
    last: bool,
) -> widget::button::Style {
    let cosmic = theme.cosmic();
    let mut style = widget::button::Style::new();
    let surface = &cosmic.background(theme.transparent).component;

    let (background, foreground) = if selected {
        (surface.divider.into(), cosmic.accent_text_color().into())
    } else if interaction.pressed {
        (surface.divider.into(), surface.on.into())
    } else if interaction.hovered {
        (surface.hover.into(), surface.on.into())
    } else {
        (surface.base.into(), surface.on.into())
    };

    style.background = Some(Background::Color(apply_alpha(background, opacity)));
    let radius = cosmic.corner_radii.radius_s;
    style.border_radius = cosmic::iced::border::Radius {
        top_left: if first { radius[0] } else { 0.0 },
        top_right: if last { radius[1] } else { 0.0 },
        bottom_right: if last { radius[2] } else { 0.0 },
        bottom_left: if first { radius[3] } else { 0.0 },
    };
    style.border_width = 0.0;
    style.border_color = apply_alpha(surface.divider.into(), opacity);
    style.outline_width = if interaction.focused { 1.0 } else { 0.0 };
    style.outline_color = cosmic.accent.base.into();
    style.text_color = Some(apply_alpha(foreground, opacity));
    style.icon_color = Some(apply_alpha(foreground, opacity));

    style
}
