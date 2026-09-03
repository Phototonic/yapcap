use super::super::super::{
    Alignment, Background, Element, Length, Message, ProviderId, container, fl, row, widget,
};

fn account_add_message(provider: ProviderId) -> Message {
    match provider {
        ProviderId::Cursor => Message::StartCursorScan,
        _ => Message::StartLogin(provider),
    }
}

pub(super) fn opencode_import_available(provider: ProviderId) -> bool {
    match provider {
        ProviderId::Codex => crate::providers::codex::opencode_import_available(),
        ProviderId::Copilot => crate::providers::copilot::opencode_import_available(),
        _ => false,
    }
}

fn empty_account_button(
    label: String,
    icon: Element<'static, Message>,
    class: cosmic::theme::Button,
    press: Option<Message>,
) -> Element<'static, Message> {
    let content = row![
        cosmic::iced::widget::Space::new().width(Length::Fill),
        icon,
        widget::text(label).size(14),
        cosmic::iced::widget::Space::new().width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .height(Length::Fill)
    .width(Length::Fill);

    widget::button::custom(content)
        .width(Length::Fill)
        .height(Length::Fixed(42.0))
        .class(class)
        .on_press_maybe(press)
        .into()
}

fn empty_account_button_class(accent: bool, enabled: bool) -> cosmic::theme::Button {
    cosmic::theme::Button::Custom {
        active: Box::new(move |_focused, theme| {
            empty_account_button_style(theme, accent, enabled, false, false)
        }),
        disabled: Box::new(move |theme| {
            empty_account_button_style(theme, accent, false, false, false)
        }),
        hovered: Box::new(move |_focused, theme| {
            empty_account_button_style(theme, accent, enabled, true, false)
        }),
        pressed: Box::new(move |_focused, theme| {
            empty_account_button_style(theme, accent, enabled, false, true)
        }),
    }
}

fn empty_account_button_style(
    theme: &cosmic::Theme,
    accent: bool,
    enabled: bool,
    hovered: bool,
    pressed: bool,
) -> widget::button::Style {
    let cosmic = theme.cosmic();
    let component = if accent {
        &cosmic.accent_button
    } else {
        &cosmic.button
    };
    let mut style = widget::button::Style::new();
    style.background = Some(Background::Color(if !enabled {
        component.disabled.into()
    } else if pressed {
        component.pressed.into()
    } else if hovered {
        component.hover.into()
    } else {
        component.base.into()
    }));
    style.border_radius = cosmic.corner_radii.radius_xl.into();
    style.border_width = 1.0;
    style.border_color = if enabled {
        component.border.into()
    } else {
        component.disabled_border.into()
    };
    style.text_color = Some(if enabled {
        component.on.into()
    } else {
        component.on_disabled.into()
    });
    style.icon_color = style.text_color;
    style
}

pub(super) fn empty_accounts_state(
    provider: ProviderId,
    enabled: bool,
) -> Element<'static, Message> {
    let add_button = empty_account_button(
        fl!("account-add"),
        widget::text("+").size(20).into(),
        empty_account_button_class(true, enabled),
        enabled.then(|| account_add_message(provider)),
    );
    let mut actions = cosmic::iced::widget::column![add_button]
        .spacing(8)
        .width(Length::Fixed(250.0));
    if opencode_import_available(provider) {
        actions = actions.push(empty_account_button(
            fl!("import-from-opencode"),
            widget::icon::from_name("go-up-symbolic")
                .icon()
                .size(18)
                .into(),
            empty_account_button_class(false, enabled),
            enabled.then_some(Message::ImportFromOpenCode(provider, None)),
        ));
    }

    let content = cosmic::iced::widget::column![
        widget::text(fl!("accounts-empty-title")).size(17),
        widget::text(fl!("accounts-empty-description")).size(12),
        container(actions)
            .padding([20, 0, 0, 0])
            .width(Length::Fixed(250.0)),
    ]
    .spacing(4)
    .align_x(Alignment::Center)
    .width(Length::Fill);

    container(content)
        .width(Length::Fill)
        .padding([30, 16, 20, 16])
        .align_x(Alignment::Center)
        .style(empty_accounts_state_style)
        .into()
}

fn empty_accounts_state_style(theme: &cosmic::Theme) -> widget::container::Style {
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
