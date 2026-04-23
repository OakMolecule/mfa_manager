use crate::app::{Message, NavigationTarget, MATERIAL_ICONS};
use crate::icons;
use crate::theme::{self as t};
use crate::widgets::sidebar::create_sidebar;
use chrono::Utc;
use iced::{
    widget::{button, column, container, progress_bar, row, scrollable, text, text_input, Space},
    Alignment, Border, Color, Element, Length, Shadow, Vector,
};
use std::collections::HashSet;
use uuid::Uuid;
use vaultx_core::{entry::Entry, totp::TotpEngine};

/// 主列表页状态
#[derive(Debug, Clone, Default)]
pub struct ListScreen {
    pub search_query: String,
    pub visible_passwords: HashSet<Uuid>,
}

// ── 顶栏圆形图标按钮（36×36，半透明白底，hover 变亮）───────────────────────
fn topbar_icon_btn(icon_cp: &'static str, msg: Message) -> Element<'static, Message> {
    button(
        container(
            text(icon_cp)
                .font(MATERIAL_ICONS)
                .size(20)
                .color(Color::WHITE),
        )
        .width(36)
        .height(36)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(msg)
    .padding(0)
    .style(|_: &iced::Theme, status| {
        let alpha = match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => 0.28,
            _ => 0.15,
        };
        iced::widget::button::Style {
            background: Some(iced::Background::Color(Color::from_rgba(
                1.0, 1.0, 1.0, alpha,
            ))),
            border: Border {
                radius: 18.0.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    })
    .into()
}

impl ListScreen {
    pub fn view<'a>(&'a self, entries: &'a [Entry]) -> Element<'a, Message> {
        // ── 顶栏：蓝色背景，高 56px，左右内边距 16px ─────────────────────
        let topbar = container(
            row![
                text(icons::LOCK)
                    .font(MATERIAL_ICONS)
                    .size(26)
                    .color(Color::WHITE),
                Space::with_width(12),
                // 搜索框：Fill 占满剩余空间，高 36px，圆角 18px
                container(
                    row![
                        text(icons::SEARCH)
                            .font(MATERIAL_ICONS)
                            .size(16)
                            .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                        Space::with_width(6),
                        text_input("搜索条目...", &self.search_query)
                            .on_input(Message::SearchChanged)
                            .size(14)
                            .padding([0, 0])
                            .width(Length::Fill)
                            .style(|_: &iced::Theme, _| iced::widget::text_input::Style {
                                background: iced::Background::Color(Color::TRANSPARENT),
                                border: Border {
                                    width: 0.0,
                                    ..Default::default()
                                },
                                icon: Color::from_rgba(1.0, 1.0, 1.0, 0.7),
                                placeholder: Color::from_rgba(1.0, 1.0, 1.0, 0.7),
                                value: Color::WHITE,
                                selection: Color::from_rgba(1.0, 1.0, 1.0, 0.3),
                            }),
                    ]
                    .spacing(0)
                    .align_y(Alignment::Center)
                    .padding(iced::Padding {
                        top: 0.0,
                        right: 12.0,
                        bottom: 0.0,
                        left: 16.0,
                    }),
                )
                .width(Length::Fill)
                .height(36)
                .align_y(iced::alignment::Vertical::Center)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        1.0, 1.0, 1.0, 0.15,
                    ))),
                    border: Border {
                        radius: 18.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Default::default()
                }),
                Space::with_width(8),
                topbar_icon_btn(icons::ADD, Message::NavigateTo(NavigationTarget::NewEntry)),
                topbar_icon_btn(
                    icons::SETTINGS,
                    Message::NavigateTo(NavigationTarget::Settings),
                ),
                topbar_icon_btn(icons::LOCK, Message::LockVault),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .padding([0, 16]),
        )
        .height(56)
        .width(Length::Fill)
        .align_y(iced::alignment::Vertical::Center)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(t::PRIMARY)),
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        });

        // ── 侧边栏（200px 固定，浅蓝灰背景）──────────────────────────────
        let totp_count = entries.iter().filter(|e| e.totp.is_some()).count();
        let sidebar_container = create_sidebar(NavigationTarget::List, totp_count);

        // ── 过滤条目 ──────────────────────────────────────────────────────
        let query = self.search_query.to_lowercase();
        let filtered: Vec<&Entry> = entries
            .iter()
            .filter(|e| {
                query.is_empty()
                    || e.title.to_lowercase().contains(&query)
                    || e.password
                        .as_ref()
                        .map(|p| p.username.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .collect();

        // ── 条目卡片列表 ───────────────────────────────────────────────────
        let cards: Vec<Element<Message>> = filtered
            .iter()
            .map(|entry| self.entry_card(entry))
            .collect();

        let mut content_col = column![].spacing(10).padding(iced::Padding {
            top: 16.0,
            right: 20.0,
            bottom: 20.0,
            left: 20.0,
        });

        if cards.is_empty() {
            content_col = content_col.push(Space::with_height(60)).push(
                container(
                    column![
                        text(icons::SHIELD)
                            .font(MATERIAL_ICONS)
                            .size(64)
                            .color(t::SURFACE_VARIANT),
                        text(if entries.is_empty() {
                            "金库为空，点击「新建」添加条目"
                        } else {
                            "没有匹配的条目"
                        })
                        .size(15)
                        .color(t::ON_SURFACE_VARIANT),
                    ]
                    .spacing(12)
                    .align_x(Alignment::Center),
                )
                .width(Length::Fill)
                .center_x(Length::Fill),
            );
        } else {
            for card in cards {
                content_col = content_col.push(card);
            }
        }

        let scroll = scrollable(content_col)
            .width(Length::Fill)
            .height(Length::Fill);

        let main_area = container(scroll)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(t::SURFACE)),
                ..Default::default()
            });

        // ── 页面主体（顶栏 + 侧边栏 + 内容区）─────────────────────────────
        let page_body: Element<Message> = column![
            topbar,
            row![sidebar_container, main_area].height(Length::Fill),
        ]
        .height(Length::Fill)
        .into();

        // ── 悬浮 FAB：右下角 56×56 圆形新建按钮 ──────────────────────────
        let fab: Element<Message> = container(
            button(
                container(
                    text(icons::ADD)
                        .font(MATERIAL_ICONS)
                        .size(24)
                        .color(Color::WHITE),
                )
                .width(56)
                .height(56)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            )
            .on_press(Message::NavigateTo(NavigationTarget::NewEntry))
            .padding(0)
            .style(|_: &iced::Theme, status| {
                let bg = match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => t::PRIMARY_LIGHT,
                    _ => t::PRIMARY,
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(bg)),
                    border: Border {
                        radius: 28.0.into(),
                        ..Default::default()
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
                        offset: Vector::new(0.0, 4.0),
                        blur_radius: 12.0,
                    },
                    ..Default::default()
                }
            }),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Right)
        .align_y(iced::alignment::Vertical::Bottom)
        .padding(iced::Padding {
            top: 0.0,
            right: 24.0,
            bottom: 24.0,
            left: 0.0,
        })
        .into();

        iced::widget::Stack::new().push(page_body).push(fab).into()
    }

    fn entry_card<'a>(&'a self, entry: &'a Entry) -> Element<'a, Message> {
        let show_pw = self.visible_passwords.contains(&entry.id);
        let id = entry.id;
        let has_pw = entry.password.is_some();
        let has_totp = entry.totp.is_some();

        // ── 相对更新时间 ──────────────────────────────────────────────────
        let update_text = {
            let now = Utc::now();
            let delta = now.signed_duration_since(entry.updated_at);
            if delta.num_seconds() < 60 {
                "刚刚".to_string()
            } else if delta.num_minutes() < 60 {
                format!("{} 分钟前", delta.num_minutes())
            } else if delta.num_hours() < 24 {
                format!("{} 小时前", delta.num_hours())
            } else if delta.num_days() < 30 {
                format!("{} 天前", delta.num_days())
            } else if delta.num_days() < 365 {
                format!("{} 月前", delta.num_days() / 30)
            } else {
                format!("{} 年前", delta.num_days() / 365)
            }
        };

        // ── 副标题：URL 域名或条目分类 ────────────────────────────────────
        let subtitle = entry
            .password
            .as_ref()
            .and_then(|p| {
                if p.url.is_empty() {
                    None
                } else {
                    Some(
                        p.url
                            .trim_start_matches("https://")
                            .trim_start_matches("http://")
                            .to_string(),
                    )
                }
            })
            .unwrap_or_else(|| entry.category.to_string());

        // ── 类型指示器（11px 小图标）─────────────────────────────────────
        let mut type_indicators = row![].spacing(3).align_y(Alignment::Center);
        if has_pw {
            type_indicators = type_indicators.push(
                text(icons::KEY)
                    .font(MATERIAL_ICONS)
                    .size(11)
                    .color(t::ON_SURFACE_VARIANT),
            );
        }
        if has_totp {
            type_indicators = type_indicators.push(
                text(icons::PHONELINK_LOCK)
                    .font(MATERIAL_ICONS)
                    .size(11)
                    .color(t::ON_SURFACE_VARIANT),
            );
        }

        // ── 卡片头部（可点击，hover 变浅蓝）──────────────────────────────
        let header_inner = row![
            // 站点头像：40×40 圆形，首字母 17px Bold
            container(
                text(
                    entry
                        .title
                        .chars()
                        .next()
                        .unwrap_or('?')
                        .to_uppercase()
                        .to_string()
                )
                .size(17)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..Default::default()
                })
                .color(Color::WHITE),
            )
            .width(40)
            .height(40)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(t::PRIMARY)),
                border: Border {
                    radius: 20.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::with_width(12),
            // 标题 15px Semibold + 类型标记 + 副标题 13px
            column![
                row![
                    text(&entry.title)
                        .size(15)
                        .font(iced::Font {
                            weight: iced::font::Weight::Semibold,
                            ..Default::default()
                        })
                        .color(t::ON_SURFACE),
                    Space::with_width(6),
                    type_indicators,
                ]
                .align_y(Alignment::Center),
                text(subtitle).size(13).color(t::ON_SURFACE_VARIANT),
            ]
            .spacing(2),
            Space::with_width(Length::Fill),
            text(update_text).size(12).color(t::ON_SURFACE_VARIANT),
            Space::with_width(6),
            text(icons::ARROW_FORWARD)
                .font(MATERIAL_ICONS)
                .size(16)
                .color(t::ON_SURFACE_VARIANT),
            Space::with_width(4),
        ]
        .align_y(Alignment::Center);

        let header_btn = button(container(header_inner).padding(iced::Padding {
            top: 12.0,
            right: 8.0,
            bottom: 12.0,
            left: 14.0,
        }))
        .on_press(Message::SelectEntry(id))
        .width(Length::Fill)
        .padding(0)
        .style(|_: &iced::Theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                    Some(iced::Background::Color(Color::from_rgba(
                        0.067, 0.463, 0.824, 0.05,
                    )))
                }
                _ => None,
            };
            iced::widget::button::Style {
                background: bg,
                border: Border {
                    radius: iced::border::Radius {
                        top_left: 12.0,
                        top_right: 12.0,
                        bottom_left: 0.0,
                        bottom_right: 0.0,
                    },
                    ..Default::default()
                },
                ..Default::default()
            }
        });

        let mut card_col = column![header_btn].spacing(0);

        // 头部下方 1px 分隔线
        if has_pw || has_totp {
            card_col = card_col.push(container(Space::with_height(1)).width(Length::Fill).style(
                |_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(t::OUTLINE_VARIANT)),
                    ..Default::default()
                },
            ));
        }

        // ── TOTP 整行 ─────────────────────────────────────────────────────
        if let Some(totp_data) = &entry.totp {
            let totp_result = TotpEngine::compute(totp_data);
            let (code, remaining, period, expiring) = match &totp_result {
                Ok(r) => (r.code.clone(), r.remaining, r.period, r.expiring),
                Err(_) => ("------".to_string(), 30, 30, false),
            };

            let code_display = if code.len() == 6 {
                format!("{} {}", &code[..3], &code[3..])
            } else {
                code.clone()
            };

            let code_color = if expiring { t::WARNING } else { t::PRIMARY };
            let progress = remaining as f32 / period as f32;

            let totp_row = container(
                row![
                    text(icons::PHONELINK_LOCK)
                        .font(MATERIAL_ICONS)
                        .size(15)
                        .color(code_color),
                    Space::with_width(8),
                    // TOTP 码：22px Bold Roboto Mono
                    text(code_display.clone())
                        .font(iced::Font {
                            family: iced::font::Family::Name("Roboto Mono"),
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .size(22)
                        .color(code_color),
                    Space::with_width(Length::Fill),
                    // 进度条 44×4 + 倒计时 12px
                    row![
                        progress_bar(0.0..=1.0, progress).width(44).height(4).style(
                            move |_: &iced::Theme| iced::widget::progress_bar::Style {
                                background: iced::Background::Color(t::SURFACE_VARIANT),
                                bar: iced::Background::Color(code_color),
                                border: Border::default(),
                            }
                        ),
                        text(format!("{}s", remaining))
                            .size(12)
                            .color(t::ON_SURFACE_VARIANT),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                    Space::with_width(8),
                    button(
                        container(
                            text(icons::CONTENT_COPY)
                                .font(MATERIAL_ICONS)
                                .size(14)
                                .color(t::PRIMARY),
                        )
                        .width(28)
                        .height(28)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                    )
                    .on_press(Message::CopyToClipboard(code.clone()))
                    .padding(0)
                    .style(|_: &iced::Theme, _| iced::widget::button::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.067, 0.463, 0.824, 0.12,
                        ))),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ]
                .align_y(Alignment::Center),
            )
            .padding(iced::Padding {
                top: 8.0,
                right: 14.0,
                bottom: 8.0,
                left: 14.0,
            })
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: None,
                ..Default::default()
            });

            card_col = card_col.push(totp_row);

            if has_pw {
                card_col =
                    card_col.push(container(Space::with_height(1)).width(Length::Fill).style(
                        |_: &iced::Theme| iced::widget::container::Style {
                            background: Some(iced::Background::Color(t::OUTLINE_VARIANT)),
                            ..Default::default()
                        },
                    ));
            }
        }

        // ── 用户名 + 密码（左右各 50%，1px 竖线分割）────────────────────
        if let Some(pw_data) = &entry.password {
            let pw_display = if show_pw {
                pw_data.password.clone()
            } else {
                "•".repeat(pw_data.password.len().min(12))
            };
            let pw_copy_value = pw_data.password.clone();
            let username_copy = pw_data.username.clone();
            let vis_icon = if show_pw {
                icons::VISIBILITY_OFF
            } else {
                icons::VISIBILITY
            };

            let credentials_row = row![
                // 左半区：用户名
                row![
                    text(icons::PERSON)
                        .font(MATERIAL_ICONS)
                        .size(15)
                        .color(t::ON_SURFACE_VARIANT),
                    Space::with_width(6),
                    column![
                        text("用户名").size(11).color(t::ON_SURFACE_VARIANT),
                        text(&pw_data.username).size(13).color(t::ON_SURFACE),
                    ]
                    .spacing(1),
                    Space::with_width(Length::Fill),
                    button(
                        container(
                            text(icons::CONTENT_COPY)
                                .font(MATERIAL_ICONS)
                                .size(14)
                                .color(t::PRIMARY),
                        )
                        .width(28)
                        .height(28)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                    )
                    .on_press(Message::CopyToClipboard(username_copy))
                    .padding(0)
                    .style(|_: &iced::Theme, _| iced::widget::button::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.067, 0.463, 0.824, 0.12,
                        ))),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ]
                .align_y(Alignment::Center)
                .width(Length::FillPortion(1)),
                // 竖向分隔线
                container(Space::with_width(1))
                    .height(36)
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(t::OUTLINE_VARIANT)),
                        ..Default::default()
                    }),
                Space::with_width(8),
                // 右半区：密码
                row![
                    text(icons::KEY)
                        .font(MATERIAL_ICONS)
                        .size(15)
                        .color(t::ON_SURFACE_VARIANT),
                    Space::with_width(6),
                    column![
                        text("密码").size(11).color(t::ON_SURFACE_VARIANT),
                        text(pw_display)
                            .font(iced::Font::with_name("Roboto Mono"))
                            .size(13)
                            .color(t::ON_SURFACE),
                    ]
                    .spacing(1),
                    Space::with_width(Length::Fill),
                    // 眼睛按钮：24×24，透明背景
                    button(
                        container(
                            text(vis_icon)
                                .font(MATERIAL_ICONS)
                                .size(14)
                                .color(t::ON_SURFACE_VARIANT),
                        )
                        .width(24)
                        .height(24)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                    )
                    .on_press(Message::TogglePasswordVisible(id))
                    .padding(0)
                    .style(|_: &iced::Theme, _| iced::widget::button::Style {
                        background: None,
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    Space::with_width(4),
                    // 复制按钮：28×28，浅蓝底
                    button(
                        container(
                            text(icons::CONTENT_COPY)
                                .font(MATERIAL_ICONS)
                                .size(14)
                                .color(t::PRIMARY),
                        )
                        .width(28)
                        .height(28)
                        .align_x(iced::alignment::Horizontal::Center)
                        .align_y(iced::alignment::Vertical::Center),
                    )
                    .on_press(Message::CopyToClipboard(pw_copy_value))
                    .padding(0)
                    .style(|_: &iced::Theme, _| iced::widget::button::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.067, 0.463, 0.824, 0.12,
                        ))),
                        border: Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ]
                .align_y(Alignment::Center)
                .width(Length::FillPortion(1)),
            ]
            .align_y(Alignment::Center)
            .padding(iced::Padding {
                top: 8.0,
                right: 14.0,
                bottom: 12.0,
                left: 14.0,
            })
            .spacing(8);

            card_col = card_col.push(credentials_row);
        }

        // ── 卡片容器：hover 时边框变蓝、阴影增强，用 button 实现 hover 感知 ─
        button(card_col)
            .on_press(Message::SelectEntry(id))
            .width(Length::Fill)
            .padding(0)
            .style(|_: &iced::Theme, status| {
                let (border_width, border_color, shadow_color, shadow_offset, blur) = match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => (
                        1.5_f32,
                        Color::from_rgba(0.067, 0.463, 0.824, 0.5),
                        Color::from_rgba(0.067, 0.463, 0.824, 0.15),
                        Vector::new(0.0, 4.0),
                        18.0_f32,
                    ),
                    _ => (
                        0.0_f32,
                        Color::TRANSPARENT,
                        Color::from_rgba(0.0, 0.0, 0.0, 0.06),
                        Vector::new(0.0, 2.0),
                        12.0_f32,
                    ),
                };
                iced::widget::button::Style {
                    background: Some(iced::Background::Color(Color::WHITE)),
                    border: Border {
                        radius: 12.0.into(),
                        width: border_width,
                        color: border_color,
                    },
                    shadow: Shadow {
                        color: shadow_color,
                        offset: shadow_offset,
                        blur_radius: blur,
                    },
                    text_color: t::ON_SURFACE,
                }
            })
            .into()
    }
}
