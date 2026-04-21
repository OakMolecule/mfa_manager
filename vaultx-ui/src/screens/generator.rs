use crate::app::{Message, NavigationTarget, MATERIAL_ICONS};
use crate::icons;
use crate::theme::{self as t};
use iced::{
    widget::{button, checkbox, column, container, row, slider, text, Space},
    Alignment, Border, Color, Element, Length, Shadow, Vector,
};
use vaultx_core::GeneratorConfig;

/// 密码生成器页状态
#[derive(Debug, Clone)]
pub struct GeneratorScreen {
    pub config: GeneratorConfig,
    pub generated: String,
}

impl Default for GeneratorScreen {
    fn default() -> Self {
        let config = GeneratorConfig::default();
        let generated = vaultx_core::PasswordGenerator::generate(&config);
        Self { config, generated }
    }
}

impl GeneratorScreen {
    pub fn view(&self) -> Element<'_, Message> {
        // ── 顶栏 ──────────────────────────────────────────────────────────
        let topbar = container(
            row![
                button(text(icons::CLOSE).font(MATERIAL_ICONS).size(22))
                    .on_press(Message::NavigateTo(NavigationTarget::List))
                    .padding(8),
                Space::with_width(8),
                text(icons::VPN_KEY)
                    .font(MATERIAL_ICONS)
                    .size(20)
                    .color(t::PRIMARY),
                text("密码生成器").size(18).color(t::ON_SURFACE),
                Space::with_width(Length::Fill),
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

        // ── 生成的密码展示框 ──────────────────────────────────────────────
        let pw_display = container(
            row![
                text(icons::KEY)
                    .font(MATERIAL_ICONS)
                    .size(18)
                    .color(t::PRIMARY),
                Space::with_width(8),
                text(&self.generated)
                    .font(iced::Font::with_name("Roboto Mono"))
                    .size(16)
                    .color(t::ON_SURFACE)
                    .width(Length::Fill),
                button(
                    row![
                        text(icons::CONTENT_COPY).font(MATERIAL_ICONS).size(16),
                        text("复制").size(13),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::CopyToClipboard(self.generated.clone()))
                .padding([6, 12]),
                Space::with_width(4),
                button(
                    row![
                        text(icons::REFRESH).font(MATERIAL_ICONS).size(16),
                        text("重新生成").size(13),
                    ]
                    .spacing(4)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::GeneratePassword)
                .padding([6, 12]),
            ]
            .align_y(Alignment::Center)
            .spacing(4),
        )
        .padding([14, 16])
        .width(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(t::SURFACE_VARIANT)),
            border: Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        // ── 长度滑块 ──────────────────────────────────────────────────────
        let len_val = self.config.length as u8;
        let length_row = column![
            row![
                text("密码长度").size(14).color(t::ON_SURFACE),
                Space::with_width(Length::Fill),
                container(text(format!("{}", len_val)).size(14).color(t::PRIMARY))
                    .padding([2, 8])
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(t::SURFACE_VARIANT)),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            ]
            .align_y(Alignment::Center),
            slider(8..=64, len_val, Message::GeneratorLengthChanged),
        ]
        .spacing(6);

        // ── 字符集选项 ────────────────────────────────────────────────────
        let options = column![
            row![
                checkbox("大写字母 (A-Z)", self.config.uppercase)
                    .on_toggle(|_| Message::GeneratorToggleUppercase)
                    .size(16)
                    .text_size(14)
                    .width(Length::FillPortion(1)),
                checkbox("小写字母 (a-z)", self.config.lowercase)
                    .on_toggle(|_| Message::GeneratorToggleLowercase)
                    .size(16)
                    .text_size(14)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(16),
            row![
                checkbox("数字 (0-9)", self.config.digits)
                    .on_toggle(|_| Message::GeneratorToggleDigits)
                    .size(16)
                    .text_size(14)
                    .width(Length::FillPortion(1)),
                checkbox("符号 (!@#...)", self.config.symbols)
                    .on_toggle(|_| Message::GeneratorToggleSymbols)
                    .size(16)
                    .text_size(14)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(16),
            checkbox("排除易混淆字符 (0O, 1lI)", self.config.exclude_ambiguous)
                .on_toggle(|_| Message::GeneratorToggleAmbiguous)
                .size(16)
                .text_size(14),
        ]
        .spacing(12);

        // ── 卡片容器 ──────────────────────────────────────────────────────
        let card = container(
            column![
                text("生成的密码").size(13).color(t::ON_SURFACE_VARIANT),
                Space::with_height(4),
                pw_display,
                Space::with_height(20),
                text("配置选项").size(13).color(t::ON_SURFACE_VARIANT),
                Space::with_height(8),
                length_row,
                Space::with_height(12),
                options,
            ]
            .spacing(0)
            .padding(24),
        )
        .width(520)
        .style(|_: &iced::Theme| iced::widget::container::Style {
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
        });

        let content = container(card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(t::SURFACE)),
                ..Default::default()
            });

        column![topbar, content].into()
    }

    /// 渲染抽屉式生成器（在 Detail 页面上叠加显示）
    pub fn view_drawer(&self) -> Element<'_, Message> {
        // ── 生成的密码展示框 ──────────────────────────────────────────────
        let pw_display = container(
            row![
                text(icons::KEY)
                    .font(MATERIAL_ICONS)
                    .size(18)
                    .color(t::PRIMARY),
                Space::with_width(8),
                text(&self.generated)
                    .font(iced::Font::with_name("Roboto Mono"))
                    .size(14)
                    .color(t::ON_SURFACE)
                    .width(Length::Fill),
            ]
            .align_y(Alignment::Center)
            .spacing(4),
        )
        .padding([12, 16])
        .width(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(t::SURFACE_VARIANT)),
            border: Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        // ── 操作按钮 ──────────────────────────────────────────────────────
        let action_buttons = row![
            button(
                row![
                    text(icons::CONTENT_COPY).font(MATERIAL_ICONS).size(16),
                    text("复制").size(13),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .on_press(Message::CopyToClipboard(self.generated.clone()))
            .padding([8, 14])
            .style(|_: &iced::Theme, _| iced::widget::button::Style {
                background: Some(iced::Background::Color(t::PRIMARY)),
                text_color: Color::WHITE,
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .width(Length::Fill),
            button(
                row![
                    text(icons::REFRESH).font(MATERIAL_ICONS).size(16),
                    text("重新生成").size(13),
                ]
                .spacing(4)
                .align_y(Alignment::Center),
            )
            .on_press(Message::GeneratePassword)
            .padding([8, 14])
            .style(|_: &iced::Theme, _| iced::widget::button::Style {
                background: Some(iced::Background::Color(Color::from_rgb(
                    0.6, 0.6, 0.6
                ))),
                text_color: Color::WHITE,
                border: Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .width(Length::Fill),
        ]
        .spacing(8);

        // ── 长度滑块 ──────────────────────────────────────────────────────
        let len_val = self.config.length as u8;
        let length_row = column![
            row![
                text("密码长度").size(13).color(t::ON_SURFACE),
                Space::with_width(Length::Fill),
                container(text(format!("{}", len_val)).size(13).color(t::PRIMARY))
                    .padding([2, 8])
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(t::SURFACE_VARIANT)),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            ]
            .align_y(Alignment::Center),
            slider(8..=64, len_val, Message::GeneratorLengthChanged),
        ]
        .spacing(6);

        // ── 字符集选项 ────────────────────────────────────────────────────
        let options = column![
            checkbox("大写字母 (A-Z)", self.config.uppercase)
                .on_toggle(|_| Message::GeneratorToggleUppercase)
                .size(16)
                .text_size(13),
            checkbox("小写字母 (a-z)", self.config.lowercase)
                .on_toggle(|_| Message::GeneratorToggleLowercase)
                .size(16)
                .text_size(13),
            checkbox("数字 (0-9)", self.config.digits)
                .on_toggle(|_| Message::GeneratorToggleDigits)
                .size(16)
                .text_size(13),
            checkbox("符号 (!@#...)", self.config.symbols)
                .on_toggle(|_| Message::GeneratorToggleSymbols)
                .size(16)
                .text_size(13),
            checkbox("排除易混淆字符", self.config.exclude_ambiguous)
                .on_toggle(|_| Message::GeneratorToggleAmbiguous)
                .size(16)
                .text_size(13),
        ]
        .spacing(10);

        // ── 抽屉卡片 ──────────────────────────────────────────────────────
        let drawer_card = container(
            column![
                // 标题行
                row![
                    text(icons::VPN_KEY)
                        .font(MATERIAL_ICONS)
                        .size(18)
                        .color(t::PRIMARY),
                    text("密码生成器").size(16).color(t::ON_SURFACE),
                    Space::with_width(Length::Fill),
                    button(text(icons::CLOSE).font(MATERIAL_ICONS).size(20))
                        .on_press(Message::CloseGenerator)
                        .padding(4)
                        .style(|_: &iced::Theme, _| iced::widget::button::Style {
                            background: None,
                            text_color: t::ON_SURFACE,
                            ..Default::default()
                        }),
                ]
                .align_y(Alignment::Center)
                .spacing(6),
                Space::with_height(16),
                pw_display,
                Space::with_height(12),
                action_buttons,
                Space::with_height(20),
                text("配置选项")
                    .size(12)
                    .color(t::ON_SURFACE_VARIANT),
                Space::with_height(10),
                length_row,
                Space::with_height(14),
                options,
            ]
            .spacing(0)
            .padding(20),
        )
        .width(340)
        .height(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::WHITE)),
            border: Border {
                radius: 0.0.into(),
                ..Default::default()
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: Vector::new(-4.0, 0.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        });

        drawer_card.into()
    }
}

