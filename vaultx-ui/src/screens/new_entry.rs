use crate::app::{Message, NavigationTarget, MATERIAL_ICONS};
use crate::icons;
use crate::theme::{self as t};
use iced::{
    widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space},
    Alignment, Border, Color, Element, Length, Shadow, Vector,
};

/// 新建条目页状态
#[derive(Debug, Clone, Default)]
pub struct NewEntryScreen {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
    pub totp_secret: String,
    pub totp_issuer: String,
    pub include_totp: bool,
    pub show_password: bool,
    pub error_message: Option<String>,
}

impl NewEntryScreen {
    pub fn view(&self) -> Element<'_, Message> {
        // ── 顶栏 ──────────────────────────────────────────────────────────
        let topbar = container(
            row![
                button(text(icons::CLOSE).font(MATERIAL_ICONS).size(22))
                    .on_press(Message::NavigateTo(NavigationTarget::List))
                    .padding(8),
                Space::with_width(8),
                text(icons::ADD_CIRCLE)
                    .font(MATERIAL_ICONS)
                    .size(20)
                    .color(t::PRIMARY),
                text("新建条目").size(18).color(t::ON_SURFACE),
                Space::with_width(Length::Fill),
                button(
                    row![
                        text(icons::CHECK_CIRCLE)
                            .font(MATERIAL_ICONS)
                            .size(18)
                            .color(Color::WHITE),
                        text("保存").size(14).color(Color::WHITE),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::SaveEntry)
                .padding([8, 16]),
            ]
            .align_y(Alignment::Center)
            .spacing(6)
            .padding([0, 12]),
        )
        .height(56)
        .width(Length::Fill)
        .center_y(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::WHITE)),
            border: Border {
                color: t::SURFACE_VARIANT,
                width: 1.0,
                radius: 0.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        });

        let vis_icon = if self.show_password {
            icons::VISIBILITY_OFF
        } else {
            icons::VISIBILITY
        };

        // ── 表单内容 ──────────────────────────────────────────────────────
        let mut form = column![
            // 标题（必填）
            label_field("标题 *"),
            text_input("条目名称（必填）", &self.title)
                .on_input(Message::NewEntryTitleChanged)
                .size(14)
                .padding([8, 10]),
            Space::with_height(12),
            // 用户名
            label_field("用户名"),
            text_input("用户名或邮箱", &self.username)
                .on_input(Message::NewEntryUsernameChanged)
                .size(14)
                .padding([8, 10]),
            Space::with_height(12),
            // 密码
            label_field("密码"),
            row![
                text_input("密码", &self.password)
                    .secure(!self.show_password)
                    .on_input(Message::NewEntryPasswordChanged)
                    .size(14)
                    .padding([8, 10])
                    .width(Length::Fill),
                button(
                    text(vis_icon)
                        .font(MATERIAL_ICONS)
                        .size(18)
                        .color(t::ON_SURFACE_VARIANT)
                )
                .on_press(Message::NewEntryToggleShowPassword)
                .padding([8, 10]),
                button(
                    row![
                        text(icons::REFRESH).font(MATERIAL_ICONS).size(16),
                        text("生成").size(12),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::NavigateTo(NavigationTarget::Generator))
                .padding([8, 10]),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
            Space::with_height(12),
            // URL
            label_field("网址 (URL)"),
            text_input("https://example.com", &self.url)
                .on_input(Message::NewEntryUrlChanged)
                .size(14)
                .padding([8, 10]),
            Space::with_height(12),
            // TOTP 开关
            checkbox("启用双因素验证 (TOTP)", self.include_totp)
                .on_toggle(|_| Message::NewEntryToggleTotp)
                .size(16)
                .text_size(14),
        ]
        .spacing(4);

        // TOTP 字段（可选）
        if self.include_totp {
            form = form
                .push(Space::with_height(8u16))
                .push(
                    text("TOTP 密钥 (Base32)")
                        .size(12)
                        .color(t::ON_SURFACE_VARIANT),
                )
                .push(
                    text_input("JBSWY3DPEHPK3PXP", &self.totp_secret)
                        .on_input(Message::NewEntryTotpSecretChanged)
                        .size(14)
                        .padding([8u16, 10]),
                )
                .push(Space::with_height(8u16))
                .push(
                    text("发行者 (Issuer)")
                        .size(12)
                        .color(t::ON_SURFACE_VARIANT),
                )
                .push(
                    text_input("GitHub / Google / ...", &self.totp_issuer)
                        .on_input(Message::NewEntryTotpIssuerChanged)
                        .size(14)
                        .padding([8u16, 10]),
                );
        }

        // 错误提示
        if let Some(err) = &self.error_message {
            form = form.push(Space::with_height(8)).push(
                container(
                    row![
                        text(icons::WARNING)
                            .font(MATERIAL_ICONS)
                            .size(16)
                            .color(t::ERROR),
                        Space::with_width(6),
                        text(err.as_str()).size(13).color(t::ERROR),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding([8, 12])
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.83, 0.18, 0.18, 0.08,
                    ))),
                    border: Border {
                        color: t::ERROR,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }),
            );
        }

        let form_col = form.padding(iced::Padding {
            top: 24.0,
            right: 24.0,
            bottom: 24.0,
            left: 24.0,
        });

        let card = container(form_col).width(520).style(|_: &iced::Theme| {
            iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::WHITE)),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 24.0,
                },
                ..Default::default()
            }
        });

        // card 列直接传给 scrollable，不用 container 包一层
        let card_col = column![card]
            .align_x(Alignment::Center)
            .padding(iced::Padding {
                top: 20.0,
                right: 0.0,
                bottom: 20.0,
                left: 0.0,
            });

        let content = container(
            scrollable(card_col)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(t::SURFACE)),
            ..Default::default()
        });

        column![topbar, content].into()
    }
}

fn label_field(label: &str) -> iced::widget::Text<'static> {
    iced::widget::text(label.to_string())
        .size(12)
        .color(t::ON_SURFACE_VARIANT)
}
