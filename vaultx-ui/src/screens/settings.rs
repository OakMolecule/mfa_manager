use crate::app::{Message, NavigationTarget, ThemePreference, MATERIAL_ICONS};
use crate::icons;
use crate::theme::{self as t};
use crate::widgets::buttons::primary_topbar;
use iced::{
    widget::{button, column, container, pick_list, radio, row, scrollable, text, Space},
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

/// 剪贴板清除延迟选项（秒）
const CLIPBOARD_CLEAR_OPTIONS: &[(&str, u64)] = &[
    ("从不", 0),
    ("10 秒", 10),
    ("30 秒", 30),
    ("60 秒", 60),
    ("120 秒", 120),
];

/// 最大错误次数选项
const MAX_ERROR_COUNT_OPTIONS: &[(&str, u32)] =
    &[("3 次", 3), ("5 次", 5), ("10 次", 10), ("不限制", 999)];

/// 设置页状态
#[derive(Debug, Clone, Default)]
pub struct SettingsScreen {
    pub theme_pref: ThemePreference,
    pub auto_lock_timeout: u64,
    pub clipboard_clear_seconds: u64,
    pub max_error_count: u32,
}

impl SettingsScreen {
    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        let topbar = primary_topbar(
            icons::ARROW_BACK,
            Message::NavigateTo(NavigationTarget::List),
            icons::SETTINGS,
            "设置",
            None,
        );

        // ── 左侧导航栏 ──────────────────────────────────────────────────────
        let nav_item = |icon_cp: &'a str,
                        label: String,
                        active: bool,
                        msg: Option<Message>|
         -> Element<'a, Message> {
            let icon_color = if active { t::PRIMARY } else { t::ON_SURFACE };
            let label_color = if active { t::PRIMARY } else { t::ON_SURFACE };
            let bg = if active {
                Some(iced::Background::Color(Color::from_rgb(
                    0.733, 0.871, 0.984,
                )))
            } else {
                None
            };
            let pad_left = if active { 7.0_f32 } else { 10.0_f32 };

            let content = row![
                text(icon_cp)
                    .font(MATERIAL_ICONS)
                    .size(16)
                    .color(icon_color),
                text(label).size(13).color(label_color),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .width(Length::Fill);

            let inner = container(content)
                .padding(iced::Padding {
                    top: 9.0,
                    right: 10.0,
                    bottom: 9.0,
                    left: pad_left,
                })
                .width(Length::Fill);

            let styled_btn = if let Some(m) = msg {
                button(inner)
                    .on_press(m)
                    .width(Length::Fill)
                    .padding(0)
                    .style(move |_: &iced::Theme, _| iced::widget::button::Style {
                        background: bg,
                        border: Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
            } else {
                button(inner)
                    .width(Length::Fill)
                    .padding(0)
                    .style(move |_: &iced::Theme, _| iced::widget::button::Style {
                        background: bg,
                        border: Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
            };

            if active {
                row![
                    container(Space::with_width(3))
                        .height(Length::Shrink)
                        .padding([4, 0])
                        .style(|_: &iced::Theme| iced::widget::container::Style {
                            background: Some(iced::Background::Color(t::PRIMARY)),
                            border: Border {
                                radius: iced::border::Radius {
                                    top_left: 0.0,
                                    top_right: 3.0,
                                    bottom_right: 3.0,
                                    bottom_left: 0.0
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    styled_btn,
                ]
                .width(Length::Fill)
                .into()
            } else {
                styled_btn.into()
            }
        };

        let sidebar_col = column![
            nav_item(
                icons::FOFDER_OPEN,
                "全部条目".to_string(),
                false,
                Some(Message::NavigateTo(NavigationTarget::List))
            ),
            container(Space::with_height(1))
                .width(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(
                        0.773, 0.859, 0.941
                    ))),
                    ..Default::default()
                }),
            container(text("类型").size(11).color(t::ON_SURFACE_VARIANT),).padding(iced::Padding {
                top: 8.0,
                right: 4.0,
                bottom: 4.0,
                left: 14.0
            }),
            nav_item(icons::KEY, "密码".to_string(), false, None),
            nav_item(icons::PHONELINK_LOCK, "TOTP".to_string(), false, None),
            container(Space::with_height(1))
                .width(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(
                        0.773, 0.859, 0.941
                    ))),
                    ..Default::default()
                }),
            container(text("设置").size(11).color(t::ON_SURFACE_VARIANT),).padding(iced::Padding {
                top: 8.0,
                right: 4.0,
                bottom: 4.0,
                left: 14.0
            }),
            nav_item(icons::SETTINGS, "设置".to_string(), true, None),
        ]
        .spacing(2)
        .padding([8, 0]);

        let sidebar_container = container(sidebar_col)
            .width(200)
            .height(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(t::SURFACE_VARIANT)),
                ..Default::default()
            });

        // ── 设置内容区 ──────────────────────────────────────────────────────
        let settings_content = column![
            // 安全设置组
            container(
                column![
                    text("安全设置").size(14).color(t::ON_SURFACE_VARIANT),
                    // 自动锁定（下拉选择）
                    container(
                        row![
                            text("自动锁定")
                                .size(14)
                                .color(t::ON_SURFACE)
                                .width(Length::Fill),
                            pick_list(
                                AUTO_LOCK_OPTIONS
                                    .iter()
                                    .map(|(label, _)| *label)
                                    .collect::<Vec<_>>(),
                                AUTO_LOCK_OPTIONS
                                    .iter()
                                    .find(|(_, v)| *v == self.auto_lock_timeout)
                                    .map(|(l, _)| *l),
                                |selected| {
                                    let timeout = AUTO_LOCK_OPTIONS
                                        .iter()
                                        .find(|(l, _)| *l == selected)
                                        .map(|(_, v)| *v)
                                        .unwrap_or(300);
                                    Message::SetAutoLockTimeout(timeout)
                                }
                            )
                            .text_size(14)
                            .padding([6, 12])
                            .style(|_: &iced::Theme, _| {
                                iced::widget::pick_list::Style {
                                    text_color: t::ON_SURFACE_VARIANT,
                                    placeholder_color: t::ON_SURFACE_VARIANT,
                                    handle_color: t::ON_SURFACE_VARIANT,
                                    background: iced::Background::Color(Color::TRANSPARENT),
                                    border: Border {
                                        radius: 6.0.into(),
                                        color: t::SURFACE_VARIANT,
                                        width: 1.0,
                                    },
                                }
                            }),
                        ]
                        .align_y(Alignment::Center)
                    )
                    .padding(iced::Padding {
                        top: 12.0,
                        right: 16.0,
                        bottom: 12.0,
                        left: 16.0
                    })
                    .width(Length::Fill)
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color::WHITE)),
                        border: Border {
                            radius: 0.0.into(),
                            color: t::SURFACE_VARIANT,
                            width: 1.0
                        },
                        ..Default::default()
                    }),
                    // 剪贴板清除延迟（下拉选择）
                    container(
                        row![
                            text("剪贴板自动清除")
                                .size(14)
                                .color(t::ON_SURFACE)
                                .width(Length::Fill),
                            pick_list(
                                CLIPBOARD_CLEAR_OPTIONS
                                    .iter()
                                    .map(|(label, _)| *label)
                                    .collect::<Vec<_>>(),
                                CLIPBOARD_CLEAR_OPTIONS
                                    .iter()
                                    .find(|(_, v)| *v == self.clipboard_clear_seconds)
                                    .map(|(l, _)| *l),
                                |selected| {
                                    let seconds = CLIPBOARD_CLEAR_OPTIONS
                                        .iter()
                                        .find(|(l, _)| *l == selected)
                                        .map(|(_, v)| *v)
                                        .unwrap_or(30);
                                    Message::SetClipboardClearSeconds(seconds)
                                }
                            )
                            .text_size(14)
                            .padding([6, 12])
                            .style(|_: &iced::Theme, _| {
                                iced::widget::pick_list::Style {
                                    text_color: t::ON_SURFACE_VARIANT,
                                    placeholder_color: t::ON_SURFACE_VARIANT,
                                    handle_color: t::ON_SURFACE_VARIANT,
                                    background: iced::Background::Color(Color::TRANSPARENT),
                                    border: Border {
                                        radius: 6.0.into(),
                                        color: t::SURFACE_VARIANT,
                                        width: 1.0,
                                    },
                                }
                            }),
                        ]
                        .align_y(Alignment::Center)
                    )
                    .padding(iced::Padding {
                        top: 12.0,
                        right: 16.0,
                        bottom: 12.0,
                        left: 16.0
                    })
                    .width(Length::Fill)
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color::WHITE)),
                        border: Border {
                            radius: 0.0.into(),
                            color: t::SURFACE_VARIANT,
                            width: 1.0
                        },
                        ..Default::default()
                    }),
                    // 密码错误最大次数（下拉选择）
                    container(
                        row![
                            text("密码错误最大次数")
                                .size(14)
                                .color(t::ON_SURFACE)
                                .width(Length::Fill),
                            pick_list(
                                MAX_ERROR_COUNT_OPTIONS
                                    .iter()
                                    .map(|(label, _)| *label)
                                    .collect::<Vec<_>>(),
                                MAX_ERROR_COUNT_OPTIONS
                                    .iter()
                                    .find(|(_, v)| *v == self.max_error_count)
                                    .map(|(l, _)| *l),
                                |selected| {
                                    let count = MAX_ERROR_COUNT_OPTIONS
                                        .iter()
                                        .find(|(l, _)| *l == selected)
                                        .map(|(_, v)| *v)
                                        .unwrap_or(5);
                                    Message::SetMaxErrorCount(count)
                                }
                            )
                            .text_size(14)
                            .padding([6, 12])
                            .style(|_: &iced::Theme, _| {
                                iced::widget::pick_list::Style {
                                    text_color: t::ON_SURFACE_VARIANT,
                                    placeholder_color: t::ON_SURFACE_VARIANT,
                                    handle_color: t::ON_SURFACE_VARIANT,
                                    background: iced::Background::Color(Color::TRANSPARENT),
                                    border: Border {
                                        radius: 6.0.into(),
                                        color: t::SURFACE_VARIANT,
                                        width: 1.0,
                                    },
                                }
                            }),
                        ]
                        .align_y(Alignment::Center)
                    )
                    .padding(iced::Padding {
                        top: 12.0,
                        right: 16.0,
                        bottom: 12.0,
                        left: 16.0
                    })
                    .width(Length::Fill)
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color::WHITE)),
                        border: Border {
                            radius: 0.0.into(),
                            color: t::SURFACE_VARIANT,
                            width: 1.0
                        },
                        ..Default::default()
                    }),
                    // 立即锁定按钮
                    container(
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
                        .padding([10, 16])
                        .style(|_: &iced::Theme, _| {
                            iced::widget::button::Style {
                                background: Some(iced::Background::Color(t::PRIMARY)),
                                text_color: Color::WHITE,
                                border: Border {
                                    radius: 6.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        }),
                    )
                    .padding(iced::Padding {
                        top: 12.0,
                        right: 16.0,
                        bottom: 12.0,
                        left: 16.0
                    })
                    .width(Length::Fill)
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color::WHITE)),
                        border: Border {
                            radius: 0.0.into(),
                            color: t::SURFACE_VARIANT,
                            width: 1.0
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(0)
            )
            .padding(iced::Padding {
                top: 16.0,
                right: 16.0,
                bottom: 16.0,
                left: 16.0
            })
            .width(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::WHITE)),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
                    offset: Vector::new(0.0, 1.0),
                    blur_radius: 3.0
                },
                ..Default::default()
            }),
            Space::with_height(20),
            // 外观设置组
            container(
                column![
                    text("外观").size(14).color(t::ON_SURFACE_VARIANT),
                    Space::with_height(12),
                    // 主题选择
                    container(
                        column![
                            text("主题").size(14).color(t::ON_SURFACE),
                            Space::with_height(8),
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
                        .spacing(6)
                    )
                    .padding(iced::Padding {
                        top: 12.0,
                        right: 16.0,
                        bottom: 12.0,
                        left: 16.0
                    })
                    .width(Length::Fill)
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color::WHITE)),
                        border: Border {
                            radius: 0.0.into(),
                            color: t::SURFACE_VARIANT,
                            width: 1.0
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(0)
            )
            .padding(iced::Padding {
                top: 16.0,
                right: 16.0,
                bottom: 16.0,
                left: 16.0
            })
            .width(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::WHITE)),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
                    offset: Vector::new(0.0, 1.0),
                    blur_radius: 3.0
                },
                ..Default::default()
            }),
            Space::with_height(20),
            // 关于组
            container(
                column![
                    text("关于").size(14).color(t::ON_SURFACE_VARIANT),
                    Space::with_height(12),
                    // 版本信息
                    container(
                        column![
                            row![
                                text("版本")
                                    .size(14)
                                    .color(t::ON_SURFACE)
                                    .width(Length::Fill),
                                text("v0.1.0").size(14).color(t::ON_SURFACE_VARIANT),
                            ]
                            .align_y(Alignment::Center),
                            Space::with_height(8),
                            text("本地优先 · AES-256-GCM 加密 · Argon2id 密钥派生")
                                .size(11)
                                .color(t::ON_SURFACE_VARIANT),
                        ]
                        .spacing(0)
                    )
                    .padding(iced::Padding {
                        top: 12.0,
                        right: 16.0,
                        bottom: 12.0,
                        left: 16.0
                    })
                    .width(Length::Fill)
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color::WHITE)),
                        border: Border {
                            radius: 0.0.into(),
                            color: t::SURFACE_VARIANT,
                            width: 1.0
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(0)
            )
            .padding(iced::Padding {
                top: 16.0,
                right: 16.0,
                bottom: 16.0,
                left: 16.0
            })
            .width(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::WHITE)),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
                    offset: Vector::new(0.0, 1.0),
                    blur_radius: 3.0
                },
                ..Default::default()
            }),
        ]
        .spacing(0)
        .padding(iced::Padding {
            top: 20.0,
            right: 20.0,
            bottom: 20.0,
            left: 20.0,
        })
        .width(Length::Fill);

        let content_container = container(
            iced::widget::scrollable(settings_content)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(t::SURFACE)),
            ..Default::default()
        });

        column![
            topbar,
            row![sidebar_container, content_container,].height(Length::Fill),
        ]
        .into()
    }
}
