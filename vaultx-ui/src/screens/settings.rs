use crate::app::{Message, NavigationTarget, ThemePreference, MATERIAL_ICONS};
use crate::icons;
use crate::theme::{self as t};
use iced::{
    widget::{button, column, container, radio, row, text, Space},
    Alignment, Border, Color, Element, Length, Shadow, Vector,
};

/// 自动锁定超时选项（秒）
const AUTO_LOCK_OPTIONS: &[(&str, u64)] = &[
    ("从不", 0),
    ("1 分钟", 60),
    ("5 分钟", 300),
    ("15 分钟", 900),
    ("30 分钟", 1800),
    ("1 小时", 3600),
];

/// 设置页状态
#[derive(Debug, Clone, Default)]
pub struct SettingsScreen {
    pub theme_pref: ThemePreference,
    pub auto_lock_timeout: u64,
}

impl SettingsScreen {
    pub fn view(&self) -> Element<'_, Message> {
        let topbar = container(
            row![
                button(text(icons::CLOSE).font(MATERIAL_ICONS).size(22))
                    .on_press(Message::NavigateTo(NavigationTarget::List))
                    .padding(8),
                Space::with_width(8),
                text(icons::SETTINGS)
                    .font(MATERIAL_ICONS)
                    .size(20)
                    .color(t::PRIMARY),
                text("设置").size(18).color(t::ON_SURFACE),
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

        let theme_options = column![
            radio(
                "浅色模式",
                ThemePreference::Light,
                Some(self.theme_pref),
                Message::SetTheme
            )
            .size(16)
            .text_size(14),
            radio(
                "深色模式",
                ThemePreference::Dark,
                Some(self.theme_pref),
                Message::SetTheme
            )
            .size(16)
            .text_size(14),
        ]
        .spacing(10);

        let auto_lock_options =
            AUTO_LOCK_OPTIONS
                .iter()
                .fold(column![].spacing(8), |col, &(label, secs)| {
                    col.push(
                        radio(
                            label,
                            secs,
                            Some(self.auto_lock_timeout),
                            Message::SetAutoLockTimeout,
                        )
                        .size(16)
                        .text_size(14),
                    )
                });

        let danger_zone = column![
            text("自动锁定超时").size(13).color(t::ON_SURFACE),
            Space::with_height(6),
            auto_lock_options,
            Space::with_height(16),
            button(
                row![
                    text(icons::LOCK)
                        .font(MATERIAL_ICONS)
                        .size(18)
                        .color(Color::WHITE),
                    text("立即锁定金库").size(14).color(Color::WHITE),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .on_press(Message::LockVault)
            .padding([10, 16]),
        ]
        .spacing(4);

        let version_info = column![
            text("VaultX v0.1.0").size(12).color(t::ON_SURFACE_VARIANT),
            text("本地优先 · AES-256-GCM 加密 · Argon2id 密钥派生")
                .size(11)
                .color(t::ON_SURFACE_VARIANT),
        ]
        .spacing(4)
        .align_x(Alignment::Center);

        let card = container(
            column![
                text("外观").size(12).color(t::ON_SURFACE_VARIANT),
                Space::with_height(8),
                theme_options,
                Space::with_height(24),
                text("安全").size(12).color(t::ON_SURFACE_VARIANT),
                Space::with_height(8),
                danger_zone,
                Space::with_height(32),
                version_info,
            ]
            .spacing(0)
            .padding(24),
        )
        .width(480)
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
}
