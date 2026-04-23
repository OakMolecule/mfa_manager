use crate::app::{Message, NavigationTarget, MATERIAL_ICONS};
use crate::icons;
use crate::screens::generator::GeneratorScreen;
use crate::theme::{self as t};
use chrono::Utc;
use iced::{
    widget::{button, column, container, row, stack, text, Space},
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
        entry: Option<&'a Entry>,
        generator_open: bool,
        generator: &'a GeneratorScreen,
    ) -> Element<'a, Message> {
        // 计算相对时间（例如“2小时前”）
        let update_text = if let Some(e) = entry {
            let now = Utc::now();
            let delta = now.signed_duration_since(e.updated_at);
            let s = if delta.num_seconds() < 60 {
                "刚刚".to_string()
            } else if delta.num_minutes() < 60 {
                format!("{}分钟前", delta.num_minutes())
            } else if delta.num_hours() < 24 {
                format!("{}小时前", delta.num_hours())
            } else if delta.num_days() < 30 {
                format!("{}天前", delta.num_days())
            } else if delta.num_days() < 365 {
                format!("{}月前", delta.num_days() / 30)
            } else {
                format!("{}年前", delta.num_days() / 365)
            };
            s
        } else {
            String::new()
        };

        let topbar = container(
            row![
                button(text(icons::CLOSE).font(MATERIAL_ICONS).size(22))
                    .on_press(Message::NavigateTo(NavigationTarget::List))
                    .padding(8),
                Space::with_width(8),
                text(icons::KEY)
                    .font(MATERIAL_ICONS)
                    .size(20)
                    .color(t::PRIMARY),
                text("条目详情").size(18).color(t::ON_SURFACE),
                Space::with_width(Length::Fill),
                text(update_text).size(12).color(t::ON_SURFACE_VARIANT),
                Space::with_width(8),
                text(icons::ARROW_FORWARD)
                    .font(MATERIAL_ICONS)
                    .size(18)
                    .color(t::ON_SURFACE_VARIANT),
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

        let body: Element<Message> = if let Some(entry) = entry {
            let mut content_col = column![
                // 标题
                row![
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
                        .size(20)
                        .color(Color::WHITE)
                    )
                    .width(44)
                    .height(44)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(t::PRIMARY)),
                        border: Border {
                            radius: 22.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                    Space::with_width(12),
                    column![
                        text(&entry.title).size(18).color(t::ON_SURFACE),
                        text(entry.category.to_string())
                            .size(12)
                            .color(t::ON_SURFACE_VARIANT),
                    ]
                    .spacing(2),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(16);

            // TOTP 区块
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

                let totp_block = container(
                    column![
                        row![
                            text(icons::PHONELINK_LOCK)
                                .font(MATERIAL_ICONS)
                                .size(18)
                                .color(code_color),
                            Space::with_width(6),
                            text("验证码 (TOTP)").size(12).color(t::ON_SURFACE_VARIANT),
                        ]
                        .align_y(Alignment::Center),
                        row![
                            text(code_display.clone())
                                .font(iced::Font::with_name("Roboto Mono"))
                                .size(32)
                                .color(code_color),
                            Space::with_width(Length::Fill),
                            text(format!("{}s", remaining))
                                .size(13)
                                .color(t::ON_SURFACE_VARIANT),
                            Space::with_width(8),
                            button(
                                text(icons::CONTENT_COPY)
                                    .font(MATERIAL_ICONS)
                                    .size(18)
                                    .color(t::PRIMARY)
                            )
                            .on_press(Message::CopyToClipboard(code))
                            .padding(6),
                        ]
                        .align_y(Alignment::Center),
                        iced::widget::progress_bar(0.0..=1.0, remaining as f32 / period as f32)
                            .height(4)
                            .style(move |_: &iced::Theme| iced::widget::progress_bar::Style {
                                background: iced::Background::Color(t::SURFACE_VARIANT),
                                bar: iced::Background::Color(code_color),
                                border: Border::default(),
                            }),
                        row![
                            text(icons::PERSON)
                                .font(MATERIAL_ICONS)
                                .size(14)
                                .color(t::ON_SURFACE_VARIANT),
                            Space::with_width(4),
                            text(format!("{} / {}", totp_data.issuer, totp_data.account))
                                .size(12)
                                .color(t::ON_SURFACE_VARIANT),
                        ]
                        .align_y(Alignment::Center),
                    ]
                    .spacing(8)
                    .padding([12, 16]),
                )
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgba(
                        0.067, 0.463, 0.824, 0.06,
                    ))),
                    border: Border {
                        radius: 10.0.into(),
                        color: t::SURFACE_VARIANT,
                        width: 1.0,
                    },
                    ..Default::default()
                });

                content_col = content_col.push(totp_block);
            }

            // 密码区块
            if let Some(pw) = &entry.password {
                let pw_display = if self.show_password {
                    pw.password.clone()
                } else {
                    "•".repeat(pw.password.len().min(20))
                };
                let pw_copy = pw.password.clone();
                let username_copy = pw.username.clone();
                let vis_icon = if self.show_password {
                    icons::VISIBILITY_OFF
                } else {
                    icons::VISIBILITY
                };

                let mut pw_col: iced::widget::Column<'_, Message> = column![
                    // 用户名行
                    row![
                        text(icons::PERSON)
                            .font(MATERIAL_ICONS)
                            .size(16)
                            .color(t::ON_SURFACE_VARIANT),
                        Space::with_width(6),
                        column![
                            text("用户名").size(11).color(t::ON_SURFACE_VARIANT),
                            text(&pw.username).size(14).color(t::ON_SURFACE),
                        ]
                        .spacing(1)
                        .width(Length::Fill),
                        button(
                            text(icons::CONTENT_COPY)
                                .font(MATERIAL_ICONS)
                                .size(16)
                                .color(t::PRIMARY)
                        )
                        .on_press(Message::CopyToClipboard(username_copy))
                        .padding(6),
                    ]
                    .align_y(Alignment::Center),
                    // 分割线
                    container(Space::with_height(1)).width(Length::Fill).style(
                        |_: &iced::Theme| iced::widget::container::Style {
                            background: Some(iced::Background::Color(t::SURFACE_VARIANT)),
                            ..Default::default()
                        }
                    ),
                    // 密码行
                    row![
                        text(icons::KEY)
                            .font(MATERIAL_ICONS)
                            .size(16)
                            .color(t::ON_SURFACE_VARIANT),
                        Space::with_width(6),
                        column![
                            text("密码").size(11).color(t::ON_SURFACE_VARIANT),
                            text(pw_display)
                                .font(iced::Font::with_name("Roboto Mono"))
                                .size(14)
                                .color(t::ON_SURFACE),
                        ]
                        .spacing(1)
                        .width(Length::Fill),
                        button(
                            text(vis_icon)
                                .font(MATERIAL_ICONS)
                                .size(16)
                                .color(t::ON_SURFACE_VARIANT)
                        )
                        .on_press(Message::TogglePasswordVisible(entry.id))
                        .padding(6),
                        button(
                            text(icons::CONTENT_COPY)
                                .font(MATERIAL_ICONS)
                                .size(16)
                                .color(t::PRIMARY)
                        )
                        .on_press(Message::CopyToClipboard(pw_copy))
                        .padding(6),
                    ]
                    .align_y(Alignment::Center),
                ]
                .spacing(8)
                .padding([12u16, 16]);

                if !pw.url.is_empty() {
                    pw_col = pw_col.push(
                        row![
                            text(icons::SHIELD)
                                .font(MATERIAL_ICONS)
                                .size(14)
                                .color(t::ON_SURFACE_VARIANT),
                            Space::with_width(6),
                            text(pw.url.clone()).size(13).color(t::PRIMARY),
                        ]
                        .align_y(Alignment::Center)
                        .padding([4u16, 0]),
                    );
                }

                let pw_block =
                    container(pw_col).style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color::WHITE)),
                        border: Border {
                            radius: 10.0.into(),
                            color: t::SURFACE_VARIANT,
                            width: 1.0,
                        },
                        ..Default::default()
                    });

                content_col = content_col.push(pw_block);
            }

            // 备注行（如有）
            if let Some(pw) = &entry.password {
                if !pw.notes.is_empty() {
                    content_col = content_col.push(
                        container(
                            column![
                                text("备注").size(11).color(t::ON_SURFACE_VARIANT),
                                text(&pw.notes).size(13).color(t::ON_SURFACE),
                            ]
                            .spacing(4)
                            .padding([12, 16]),
                        )
                        .style(|_: &iced::Theme| {
                            iced::widget::container::Style {
                                background: Some(iced::Background::Color(Color::WHITE)),
                                border: Border {
                                    radius: 10.0.into(),
                                    color: t::SURFACE_VARIANT,
                                    width: 1.0,
                                },
                                ..Default::default()
                            }
                        }),
                    );
                }
            }

            // 删除按钮
            content_col = content_col.push(Space::with_height(8)).push(
                button(
                    row![
                        text(icons::DELETE)
                            .font(MATERIAL_ICONS)
                            .size(18)
                            .color(t::ERROR),
                        text("删除此条目").size(14).color(t::ERROR),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::DeleteEntry(entry.id))
                .padding([8, 16]),
            );

            container(content_col.padding(20))
                .width(560)
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
                })
                .into()
        } else {
            container(text("找不到该条目").size(16).color(t::ON_SURFACE_VARIANT))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        };

        let content = container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(t::SURFACE)),
                ..Default::default()
            });

        let main_view: Element<Message> = column![topbar, content].into();

        // 如果生成器打开，叠加抽屉层
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
                row![Space::with_width(Length::Fill), generator.view_drawer(),]
                    .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill);

            stack![main_view, backdrop, drawer_container].into()
        } else {
            main_view
        }
    }
}
