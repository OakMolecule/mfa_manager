use crate::app::{Message, NavigationTarget, MATERIAL_ICONS};
use crate::icons;
use crate::screens::generator::GeneratorScreen;
use crate::theme::{self as t};
use crate::widgets::buttons::topbar_icon_button;
use crate::widgets::sidebar::create_sidebar;
use iced::{
    widget::{button, column, container, progress_bar, row, scrollable, stack, text, Space},
    Alignment, Border, Color, Element, Length, Shadow, Vector,
};
use uuid::Uuid;
use vaultx_core::{entry::Entry, totp::TotpEngine};

/// 条目详情页状态
#[derive(Debug, Clone)]
pub struct DetailScreen {
    pub entry_id: Uuid,
    pub show_password: bool,
}

impl DetailScreen {
    pub fn new(entry_id: Uuid) -> Self {
        Self {
            entry_id,
            show_password: false,
        }
    }

    pub fn view<'a>(
        &'a self,
        entries: &'a [Entry],
        generator_open: bool,
        generator: &'a GeneratorScreen,
    ) -> Element<'a, Message> {
        let entry = entries.iter().find(|e| e.id == self.entry_id);

        // ── 顶栏：蓝色背景，高 56px ──────────────────────────────────────
        // 右侧：编辑按钮 + 删除按钮
        let right_btns: Element<Message> = row![
            topbar_icon_button(icons::EDIT, Message::NavigateTo(NavigationTarget::List)),
            // 删除按钮：ERROR 弱化色图标
            button(
                container(
                    text(icons::DELETE)
                        .font(MATERIAL_ICONS)
                        .size(22)
                        .color(Color::from_rgba(1.0, 0.42, 0.42, 0.9)),
                )
                .width(32)
                .height(32)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            )
            .on_press(
                entry
                    .map(|e| Message::DeleteEntry(e.id))
                    .unwrap_or(Message::NavigateTo(NavigationTarget::List)),
            )
            .padding(0)
            .style(|_: &iced::Theme, status| iced::widget::button::Style {
                background: Some(iced::Background::Color(match status {
                    iced::widget::button::Status::Hovered
                    | iced::widget::button::Status::Pressed => {
                        t::TOPBAR_SEARCH_BG
                    }
                    _ => Color::TRANSPARENT,
                })),
                border: Border {
                    radius: 16.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ]
        .spacing(4)
        .align_y(Alignment::Center)
        .into();

        let topbar = container(
            row![
                topbar_icon_button(
                    icons::ARROW_BACK,
                    Message::NavigateTo(NavigationTarget::List),
                ),
                Space::with_width(8),
                text("条目详情")
                    .size(18)
                    .font(iced::Font {
                        weight: iced::font::Weight::Semibold,
                        ..Default::default()
                    })
                    .color(Color::WHITE),
                Space::with_width(Length::Fill),
                right_btns,
            ]
            .spacing(6)
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

        // ── 侧边栏 ────────────────────────────────────────────────────────
        let totp_count = entries.iter().filter(|e| e.totp.is_some()).count();
        let sidebar = create_sidebar(NavigationTarget::Detail(self.entry_id), totp_count);

        // ── 主体内容区 ────────────────────────────────────────────────────
        let body: Element<Message> = if let Some(entry) = entry {
            let mut cards = column![].spacing(12).padding(iced::Padding {
                top: 20.0,
                right: 24.0,
                bottom: 24.0,
                left: 24.0,
            });

            // ── 详情头卡：头像 + 标题 + 分类标签 ─────────────────────────
            let first_char = entry
                .title
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();

            let avatar = container(
                text(first_char)
                    .size(24)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..Default::default()
                    })
                    .color(Color::WHITE),
            )
            .width(56)
            .height(56)
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Center)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Gradient(iced::Gradient::Linear(
                    iced::gradient::Linear::new(std::f32::consts::FRAC_PI_4)
                        .add_stop(0.0, t::PRIMARY)
                        .add_stop(1.0, t::PRIMARY_LIGHT),
                ))),
                border: Border {
                    radius: 28.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

            let category_chip = container(
                row![
                    text(icons::LABEL)
                        .font(MATERIAL_ICONS)
                        .size(12)
                        .color(t::PRIMARY),
                    Space::with_width(4),
                    text(entry.category.to_string())
                        .size(12)
                        .font(iced::Font {
                            weight: iced::font::Weight::Medium,
                            ..Default::default()
                        })
                        .color(t::PRIMARY),
                ]
                .align_y(Alignment::Center),
            )
            .padding(iced::Padding {
                top: 4.0,
                right: 10.0,
                bottom: 4.0,
                left: 8.0,
            })
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(t::PRIMARY_CONTAINER)),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

            let header_card = card_container(
                row![
                    avatar,
                    Space::with_width(16),
                    column![
                        text(&entry.title)
                            .size(20)
                            .font(iced::Font {
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .color(t::ON_SURFACE),
                        Space::with_height(6),
                        category_chip,
                    ],
                ]
                .align_y(Alignment::Center)
                .padding(iced::Padding {
                    top: 16.0,
                    right: 16.0,
                    bottom: 16.0,
                    left: 16.0,
                })
                .into(),
            );

            cards = cards.push(header_card);

            // ── TOTP 卡 ───────────────────────────────────────────────────
            if let Some(totp_data) = &entry.totp {
                let tr = TotpEngine::compute(totp_data);
                let (code, remaining, period, expiring) = match &tr {
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

                let totp_card = card_container(
                    column![
                        // 标题行
                        row![
                            text(icons::PHONELINK_LOCK)
                                .font(MATERIAL_ICONS)
                                .size(16)
                                .color(t::PRIMARY),
                            Space::with_width(6),
                            text("TOTP 验证码")
                                .size(13)
                                .font(iced::Font {
                                    weight: iced::font::Weight::Medium,
                                    ..Default::default()
                                })
                                .color(t::ON_SURFACE_VARIANT),
                            Space::with_width(Length::Fill),
                            // 复制按钮
                            copy_btn(code.clone()),
                        ]
                        .align_y(Alignment::Center),
                        // TOTP 数字：32px Bold Roboto Mono
                        text(code_display)
                            .font(iced::Font {
                                family: iced::font::Family::Name("Roboto Mono"),
                                weight: iced::font::Weight::Bold,
                                ..Default::default()
                            })
                            .size(32)
                            .color(code_color),
                        // 进度条：OUTLINE_VARIANT 轨道，code_color 填充
                        progress_bar(0.0..=1.0, progress).height(4).style(
                            move |_: &iced::Theme| iced::widget::progress_bar::Style {
                                background: iced::Background::Color(t::OUTLINE_VARIANT),
                                bar: iced::Background::Color(code_color),
                                border: Border::default(),
                            }
                        ),
                        // 剩余秒数
                        text(format!("{}秒后刷新", remaining))
                            .size(12)
                            .color(t::ON_SURFACE_VARIANT),
                    ]
                    .spacing(10)
                    .padding(iced::Padding {
                        top: 14.0,
                        right: 16.0,
                        bottom: 14.0,
                        left: 16.0,
                    })
                    .into(),
                );

                cards = cards.push(totp_card);
            }

            // ── 密码和账号字段卡 ──────────────────────────────────────────
            if let Some(pw) = &entry.password {
                // URL 卡（非空才显示）
                if !pw.url.is_empty() {
                    let url_card = card_container(
                        column![
                            field_label(icons::SHIELD, "网址"),
                            Space::with_height(4),
                            row![
                                text(
                                    pw.url
                                        .trim_start_matches("https://")
                                        .trim_start_matches("http://")
                                        .to_string()
                                )
                                .size(14)
                                .color(t::PRIMARY),
                                Space::with_width(Length::Fill),
                                copy_btn(pw.url.clone()),
                            ]
                            .align_y(Alignment::Center),
                        ]
                        .spacing(0)
                        .padding(iced::Padding {
                            top: 14.0,
                            right: 16.0,
                            bottom: 14.0,
                            left: 16.0,
                        })
                        .into(),
                    );
                    cards = cards.push(url_card);
                }

                // 用户名卡
                let username_card = card_container(
                    column![
                        field_label(icons::PERSON, "用户名"),
                        Space::with_height(4),
                        row![
                            text(&pw.username).size(14).color(t::ON_SURFACE),
                            Space::with_width(Length::Fill),
                            copy_btn(pw.username.clone()),
                        ]
                        .align_y(Alignment::Center),
                    ]
                    .spacing(0)
                    .padding(iced::Padding {
                        top: 14.0,
                        right: 16.0,
                        bottom: 14.0,
                        left: 16.0,
                    })
                    .into(),
                );
                cards = cards.push(username_card);

                // 密码卡（含强度条）
                let pw_display = if self.show_password {
                    pw.password.clone()
                } else {
                    "•".repeat(pw.password.len().min(20))
                };
                let pw_copy = pw.password.clone();
                let vis_icon = if self.show_password {
                    icons::VISIBILITY_OFF
                } else {
                    icons::VISIBILITY
                };
                let (strength_score, strength_label, strength_color) =
                    password_strength(&pw.password);
                let strength_fraction = strength_score as f32 / 4.0;

                let password_card =
                    card_container(
                        column![
                            field_label(icons::KEY, "密码"),
                            Space::with_height(6),
                            row![
                                // 密码值：Roboto Mono 14px
                                container(
                                    text(pw_display)
                                        .font(iced::Font::with_name("Roboto Mono"))
                                        .size(14)
                                        .color(t::ON_SURFACE),
                                )
                                .padding(iced::Padding {
                                    top: 8.0,
                                    right: 10.0,
                                    bottom: 8.0,
                                    left: 10.0,
                                })
                                .width(Length::Fill)
                                .style(|_: &iced::Theme| {
                                    iced::widget::container::Style {
                                        background: Some(iced::Background::Color(t::CARD_BG)),
                                        border: Border {
                                            radius: 8.0.into(),
                                            width: 1.0,
                                            color: t::OUTLINE,
                                        },
                                        ..Default::default()
                                    }
                                }),
                                Space::with_width(8),
                                // 眼睛按钮：透明背景，不抢眼
                                button(
                                    container(
                                        text(vis_icon)
                                            .font(MATERIAL_ICONS)
                                            .size(18)
                                            .color(t::ON_SURFACE_VARIANT),
                                    )
                                    .width(32)
                                    .height(32)
                                    .align_x(iced::alignment::Horizontal::Center)
                                    .align_y(iced::alignment::Vertical::Center),
                                )
                                .on_press(Message::TogglePasswordVisible(entry.id))
                                .padding(0)
                                .style(|_: &iced::Theme, _| iced::widget::button::Style {
                                    background: None,
                                    ..Default::default()
                                }),
                                Space::with_width(4),
                                // 复制按钮：PRIMARY_CONTAINER 底
                                button(
                                    container(
                                        text(icons::CONTENT_COPY)
                                            .font(MATERIAL_ICONS)
                                            .size(16)
                                            .color(t::PRIMARY),
                                    )
                                    .width(32)
                                    .height(32)
                                    .align_x(iced::alignment::Horizontal::Center)
                                    .align_y(iced::alignment::Vertical::Center),
                                )
                                .on_press(Message::CopyToClipboard(pw_copy))
                                .padding(0)
                                .style(|_: &iced::Theme, _| iced::widget::button::Style {
                                    background: Some(iced::Background::Color(t::PRIMARY_CONTAINER)),
                                    border: Border {
                                        radius: 8.0.into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }),
                            ]
                            .align_y(Alignment::Center),
                            Space::with_height(8),
                            // 密码强度条
                            progress_bar(0.0..=1.0, strength_fraction).height(4).style(
                                move |_: &iced::Theme| iced::widget::progress_bar::Style {
                                    background: iced::Background::Color(t::OUTLINE_VARIANT),
                                    bar: iced::Background::Color(strength_color),
                                    border: Border::default(),
                                }
                            ),
                            Space::with_height(4),
                            text(format!("密码强度：{}", strength_label))
                                .size(12)
                                .font(iced::Font {
                                    weight: iced::font::Weight::Semibold,
                                    ..Default::default()
                                })
                                .color(strength_color),
                        ]
                        .spacing(0)
                        .padding(iced::Padding {
                            top: 14.0,
                            right: 16.0,
                            bottom: 14.0,
                            left: 16.0,
                        })
                        .into(),
                    );
                cards = cards.push(password_card);

                // 备注卡（非空才显示）
                let notes_text = if pw.notes.is_empty() {
                    "（无备注）".to_string()
                } else {
                    pw.notes.clone()
                };
                let notes_color = if pw.notes.is_empty() {
                    t::ON_SURFACE_VARIANT
                } else {
                    t::ON_SURFACE
                };
                let notes_card = card_container(
                    column![
                        field_label(icons::EDIT, "备注"),
                        Space::with_height(6),
                        text(notes_text).size(14).color(notes_color),
                    ]
                    .spacing(0)
                    .padding(iced::Padding {
                        top: 14.0,
                        right: 16.0,
                        bottom: 14.0,
                        left: 16.0,
                    })
                    .into(),
                );
                cards = cards.push(notes_card);
            }

            scrollable(cards)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            container(text("找不到该条目").size(16).color(t::ON_SURFACE_VARIANT))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        };

        let content_area = container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(t::SURFACE)),
                ..Default::default()
            });

        let main_view: Element<Message> =
            column![topbar, row![sidebar, content_area].height(Length::Fill),].into();

        // 生成器抽屉叠层
        if generator_open {
            let backdrop = container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.0, 0.0, 0.0, 0.3,
                    ))),
                    ..Default::default()
                });
            let drawer_container = container(
                row![Space::with_width(Length::Fill), generator.view_drawer()].height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill);
            stack![main_view, backdrop, drawer_container].into()
        } else {
            main_view
        }
    }
}

// ── 辅助：白卡容器 ─────────────────────────────────────────────────────────
fn card_container(content: Element<'_, Message>) -> Element<'_, Message> {
    container(content)
        .width(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(t::CARD_BG)),
            border: Border {
                radius: 12.0.into(),
                ..Default::default()
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.05),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 10.0,
            },
            ..Default::default()
        })
        .into()
}

// ── 辅助：字段标签（图标 + 文字，13px Medium，ON_SURFACE_VARIANT）─────────
fn field_label(icon_cp: &'static str, label: &'static str) -> Element<'static, Message> {
    row![
        text(icon_cp)
            .font(MATERIAL_ICONS)
            .size(14)
            .color(t::ON_SURFACE_VARIANT),
        Space::with_width(6),
        text(label)
            .size(13)
            .font(iced::Font {
                weight: iced::font::Weight::Medium,
                ..Default::default()
            })
            .color(t::ON_SURFACE_VARIANT),
    ]
    .align_y(Alignment::Center)
    .into()
}

// ── 辅助：复制按钮 32×32，PRIMARY_CONTAINER 底 ────────────────────────────
fn copy_btn(value: String) -> Element<'static, Message> {
    button(
        container(
            text(icons::CONTENT_COPY)
                .font(MATERIAL_ICONS)
                .size(16)
                .color(t::PRIMARY),
        )
        .width(32)
        .height(32)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(Message::CopyToClipboard(value))
    .padding(0)
    .style(|_: &iced::Theme, _| iced::widget::button::Style {
        background: Some(iced::Background::Color(t::PRIMARY_CONTAINER)),
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

// ── 辅助：密码强度计算 ────────────────────────────────────────────────────
// 返回 (分数 0-4, 标签, 颜色)
fn password_strength(pw: &str) -> (u32, &'static str, Color) {
    if pw.len() < 8 {
        return (1, "弱", t::ERROR);
    }
    let mut score = 0u32;
    if pw.chars().any(|c| c.is_uppercase()) {
        score += 1;
    }
    if pw.chars().any(|c| c.is_lowercase()) {
        score += 1;
    }
    if pw.chars().any(|c| c.is_ascii_digit()) {
        score += 1;
    }
    if pw.chars().any(|c| !c.is_alphanumeric()) {
        score += 1;
    }
    if pw.len() >= 16 && score == 4 {
        return (4, "非常强", t::SUCCESS);
    }
    match score {
        0 | 1 => (1, "弱", t::ERROR),
        2 => (2, "中等", t::WARNING),
        3 => (3, "强", t::SUCCESS),
        _ => (4, "非常强", t::SUCCESS),
    }
}
