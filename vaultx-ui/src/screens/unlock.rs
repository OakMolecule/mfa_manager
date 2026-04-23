use crate::app::{Message, MATERIAL_ICONS};
use crate::icons;
use crate::theme::{self as t};
use iced::{
    widget::{button, column, container, row, text, text_input, Space},
    Alignment, Border, Color, Element, Length,
};

pub const PASSWORD_INPUT_ID: &str = "unlock-pw";

/// 解锁页状态
#[derive(Debug, Clone, Default)]
pub struct UnlockScreen {
    pub password: String,
    pub is_new_vault: bool,
    pub error_message: Option<String>,
    pub show_password: bool,
    pub is_loading: bool,
}

impl UnlockScreen {
    pub fn new_vault_mode() -> Self {
        Self {
            is_new_vault: true,
            ..Default::default()
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let logo = column![
            container(
                text(icons::LOCK)
                    .font(MATERIAL_ICONS)
                    .size(34)
                    .color(iced::Color::WHITE)
            )
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(t::PRIMARY)),
                border: iced::Border {
                    radius: 20.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .width(68)
            .height(68)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center),
            text("VaultX").size(26).color(t::PRIMARY),
            text("本地优先 · 安全可靠的密码管理器")
                .size(13)
                .color(t::ON_SURFACE_VARIANT),
        ]
        .spacing(8)
        .align_x(Alignment::Center);

        let pw_label = row![
            text(icons::KEY)
                .font(MATERIAL_ICONS)
                .size(16)
                .color(t::ON_SURFACE_VARIANT),
            text("主密码").size(13).color(t::ON_SURFACE_VARIANT),
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        let pw_input = text_input("输入主密码...", &self.password)
            .id(text_input::Id::new(PASSWORD_INPUT_ID))
            .secure(!self.show_password)
            .on_input(Message::PasswordChanged)
            .on_submit(Message::UnlockPressed)
            .size(14)
            .padding(12);

        let unlock_icon = if self.is_new_vault {
            icons::ADD_CIRCLE
        } else {
            icons::LOCK_OPEN
        };
        let unlock_label = if self.is_new_vault {
            "创建新金库"
        } else {
            "解　锁"
        };

        let unlock_btn = button(
            row![
                text(unlock_icon)
                    .font(MATERIAL_ICONS)
                    .size(18)
                    .color(Color::WHITE),
                text(unlock_label).size(15).color(Color::WHITE),
            ]
            .align_y(Alignment::Center)
            .spacing(6),
        )
        .on_press_maybe(if self.is_loading {
            None
        } else if self.is_new_vault {
            Some(Message::CreateVaultPressed)
        } else {
            Some(Message::UnlockPressed)
        })
        .width(Length::Fill)
        .padding([10, 16])
        .style(|_: &iced::Theme, _| iced::widget::button::Style {
            background: Some(iced::Background::Color(t::PRIMARY)),
            text_color: Color::WHITE,
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        let mut card_content = column![logo, Space::with_height(12), pw_label, pw_input,]
            .spacing(8)
            .align_x(Alignment::Center);

        if let Some(err) = &self.error_message {
            card_content = card_content.push(
                container(
                    row![
                        text(icons::WARNING)
                            .font(MATERIAL_ICONS)
                            .size(16)
                            .color(t::ERROR),
                        text(err.as_str()).size(13).color(t::ERROR),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding([8, 12])
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                        0.83, 0.18, 0.18, 0.08,
                    ))),
                    border: iced::Border {
                        color: t::ERROR,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }),
            );
        }

        card_content = card_content.push(Space::with_height(4)).push(unlock_btn);

        if !self.is_new_vault {
            card_content = card_content.push(
                button(
                    row![
                        text(icons::ADD_CIRCLE)
                            .font(MATERIAL_ICONS)
                            .size(18)
                            .color(t::PRIMARY),
                        text("创建新金库").size(14).color(t::PRIMARY),
                    ]
                    .align_y(Alignment::Center)
                    .spacing(6),
                )
                .on_press(Message::CreateVaultPressed)
                .width(Length::Fill)
                .padding([8, 16])
                .style(|_: &iced::Theme, _| iced::widget::button::Style {
                    background: Some(iced::Background::Color(Color::WHITE)),
                    text_color: t::PRIMARY,
                    border: Border {
                        color: t::PRIMARY,
                        width: 1.5,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }),
            );
        }

        let card = container(card_content.padding(40))
            .width(380)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(t::CARD_BG)),
                border: iced::Border {
                    radius: 16.0.into(),
                    ..Default::default()
                },
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.1, 0.46, 0.82, 0.18),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 40.0,
                },
                ..Default::default()
            });

        container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Gradient(iced::Gradient::Linear(
                    iced::gradient::Linear::new(std::f32::consts::FRAC_PI_4)
                        .add_stop(0.0, t::SURFACE_VARIANT)
                        .add_stop(0.6, t::SURFACE)
                        .add_stop(1.0, iced::color!(0xE8F5E9)),
                ))),
                ..Default::default()
            })
            .into()
    }
}
