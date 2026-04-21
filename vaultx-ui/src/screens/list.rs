use crate::app::{Message, NavigationTarget, MATERIAL_ICONS};
use crate::icons;
use crate::theme::{self as t};
use crate::widgets::sidebar::create_sidebar;
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

impl ListScreen {
    pub fn view<'a>(&'a self, entries: &'a [Entry]) -> Element<'a, Message> {
        // ── 顶栏：蓝色背景，含搜索框和操作按钮 ──────────────────────────
        let topbar = container(
            row![
                // Logo
                text(icons::LOCK)
                    .font(MATERIAL_ICONS)
                    .size(26)
                    .color(Color::WHITE),
                Space::with_width(12),
                // 搜索框（圆角，半透明白色）
                container(
                    row![
                        text(icons::SEARCH)
                            .font(MATERIAL_ICONS)
                            .size(16)
                            .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                        Space::with_width(4),
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
                .width(280)
                .height(36)
                .align_y(iced::alignment::Vertical::Center)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        1.0, 1.0, 1.0, 0.15
                    ))),
                    border: Border {
                        radius: 18.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT
                    },
                    ..Default::default()
                }),
                Space::with_width(Length::Fill),
                // 新建
                button(
                    text(icons::ADD)
                        .font(MATERIAL_ICONS)
                        .size(20)
                        .color(Color::WHITE),
                )
                .on_press(Message::NavigateTo(NavigationTarget::NewEntry))
                .padding(8)
                .style(|_: &iced::Theme, _| iced::widget::button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        1.0, 1.0, 1.0, 0.15
                    ))),
                    border: Border {
                        radius: 18.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                // 设置
                button(
                    text(icons::SETTINGS)
                        .font(MATERIAL_ICONS)
                        .size(20)
                        .color(Color::WHITE),
                )
                .on_press(Message::NavigateTo(NavigationTarget::Settings))
                .padding(8)
                .style(|_: &iced::Theme, _| iced::widget::button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        1.0, 1.0, 1.0, 0.15
                    ))),
                    border: Border {
                        radius: 18.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                // 锁定
                button(
                    text(icons::LOCK)
                        .font(MATERIAL_ICONS)
                        .size(20)
                        .color(Color::WHITE),
                )
                .on_press(Message::LockVault)
                .padding(8)
                .style(|_: &iced::Theme, _| iced::widget::button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        1.0, 1.0, 1.0, 0.15
                    ))),
                    border: Border {
                        radius: 18.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
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

        // ── 左侧导航栏：使用可复用的侧边栏组件 ───────────────────────────
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

        // ── 背景 + 卡片内容区 ──────────────────────────────────────────
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

        // Column 直接传给 scrollable（不用 container 包装），scrollable 自身可以 Fill
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

        // 最终布局：顶栏全宽，侧边栏 + 内容区在下方
        column![
            topbar,
            row![sidebar_container, main_area,].height(Length::Fill),
        ]
        .into()
    }

    fn entry_card<'a>(&'a self, entry: &'a Entry) -> Element<'a, Message> {
        let show_pw = self.visible_passwords.contains(&entry.id);
        let id = entry.id;
        let mut card_col = column![].spacing(0);

        // ── 卡片标题行 ─────────────────────────────────────────────────────
        let header = row![
            // 头像圆圈（首字）
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
                .size(16)
                .color(Color::WHITE)
            )
            .width(36)
            .height(36)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(t::PRIMARY)),
                border: Border {
                    radius: 18.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            Space::with_width(10),
            column![
                text(&entry.title).size(15).color(t::ON_SURFACE),
                text(entry.category.to_string())
                    .size(11)
                    .color(t::ON_SURFACE_VARIANT),
            ]
            .spacing(1),
            Space::with_width(Length::Fill),
            // 删除按钮
            button(
                text(icons::DELETE)
                    .font(MATERIAL_ICONS)
                    .size(18)
                    .color(t::ON_SURFACE_VARIANT)
            )
            .on_press(Message::DeleteEntry(id))
            .padding(4),
            // 编辑按钮
            button(
                text(icons::EDIT)
                    .font(MATERIAL_ICONS)
                    .size(18)
                    .color(t::PRIMARY)
            )
            .on_press(Message::SelectEntry(id))
            .padding(4),
        ]
        .align_y(Alignment::Center)
        .padding(iced::Padding {
            top: 12.0,
            right: 14.0,
            bottom: 8.0,
            left: 14.0,
        });

        card_col = card_col.push(header);

        // ── TOTP 行 ────────────────────────────────────────────────────────
        if let Some(totp_data) = &entry.totp {
            let totp_result = TotpEngine::compute(totp_data);
            let (code, remaining, period, expiring) = match &totp_result {
                Ok(r) => (r.code.clone(), r.remaining, r.period, r.expiring),
                Err(_) => ("------".to_string(), 30, 30, false),
            };

            // 格式化 TOTP 码：中间加空格 "482 917"
            let code_display = if code.len() == 6 {
                format!("{} {}", &code[..3], &code[3..])
            } else {
                code.clone()
            };

            let code_color = if expiring { t::WARNING } else { t::PRIMARY };
            let progress = remaining as f32 / period as f32;

            let totp_row = container(
                column![
                    row![
                        text(icons::PHONELINK_LOCK)
                            .font(MATERIAL_ICONS)
                            .size(18)
                            .color(code_color),
                        Space::with_width(6),
                        text(code_display.clone())
                            .font(iced::Font::with_name("Roboto Mono"))
                            .size(24)
                            .color(code_color),
                        Space::with_width(Length::Fill),
                        text(format!("{}s", remaining))
                            .size(12)
                            .color(t::ON_SURFACE_VARIANT),
                        Space::with_width(6),
                        button(
                            text(icons::CONTENT_COPY)
                                .font(MATERIAL_ICONS)
                                .size(16)
                                .color(t::PRIMARY)
                        )
                        .on_press(Message::CopyToClipboard(code.clone()))
                        .padding([4, 6]),
                    ]
                    .align_y(Alignment::Center),
                    progress_bar(0.0..=1.0, progress)
                        .height(3)
                        .style(move |_: &iced::Theme| iced::widget::progress_bar::Style {
                            background: iced::Background::Color(t::SURFACE_VARIANT),
                            bar: iced::Background::Color(code_color),
                            border: Border::default(),
                        }),
                ]
                .spacing(6),
            )
            .padding(iced::Padding {
                top: 6.0,
                right: 14.0,
                bottom: 8.0,
                left: 14.0,
            })
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgba(
                    0.067, 0.463, 0.824, 0.05,
                ))),
                ..Default::default()
            });

            card_col = card_col.push(totp_row);
        }

        // ── 用户名 + 密码行（如有密码数据）──────────────────────────────
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
                // 用户名
                row![
                    text(icons::PERSON)
                        .font(MATERIAL_ICONS)
                        .size(15)
                        .color(t::ON_SURFACE_VARIANT),
                    Space::with_width(4),
                    text(&pw_data.username).size(13).color(t::ON_SURFACE),
                    Space::with_width(4),
                    button(
                        text(icons::CONTENT_COPY)
                            .font(MATERIAL_ICONS)
                            .size(14)
                            .color(t::PRIMARY)
                    )
                    .on_press(Message::CopyToClipboard(username_copy))
                    .padding([2, 4]),
                ]
                .align_y(Alignment::Center)
                .width(Length::FillPortion(1)),
                // 分割线
                container(Space::with_width(1))
                    .height(20)
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(t::SURFACE_VARIANT)),
                        ..Default::default()
                    }),
                Space::with_width(8),
                // 密码
                row![
                    text(icons::KEY)
                        .font(MATERIAL_ICONS)
                        .size(15)
                        .color(t::ON_SURFACE_VARIANT),
                    Space::with_width(4),
                    text(pw_display)
                        .font(iced::Font::with_name("Roboto Mono"))
                        .size(13)
                        .color(t::ON_SURFACE),
                    Space::with_width(4),
                    button(
                        text(vis_icon)
                            .font(MATERIAL_ICONS)
                            .size(14)
                            .color(t::ON_SURFACE_VARIANT)
                    )
                    .on_press(Message::TogglePasswordVisible(id))
                    .padding([2, 4]),
                    button(
                        text(icons::CONTENT_COPY)
                            .font(MATERIAL_ICONS)
                            .size(14)
                            .color(t::PRIMARY)
                    )
                    .on_press(Message::CopyToClipboard(pw_copy_value))
                    .padding([2, 4]),
                ]
                .align_y(Alignment::Center)
                .width(Length::FillPortion(1)),
            ]
            .align_y(Alignment::Center)
            .padding(iced::Padding {
                top: 4.0,
                right: 14.0,
                bottom: 12.0,
                left: 14.0,
            })
            .spacing(8);

            card_col = card_col.push(credentials_row);
        }

        container(card_col)
            .width(Length::Fill)
            .height(Length::Shrink)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::WHITE)),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                shadow: Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.06),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 12.0,
                },
                ..Default::default()
            })
            .into()
    }
}
