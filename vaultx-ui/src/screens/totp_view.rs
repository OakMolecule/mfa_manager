use crate::app::{Message, NavigationTarget, MATERIAL_ICONS};
use crate::icons;
use crate::theme::{self as t};
use crate::widgets::sidebar::create_sidebar;
use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Alignment, Border, Color, Element, Length, Shadow, Vector,
};
use vaultx_core::entry::Entry;

/// TOTP 总览页状态
#[derive(Debug, Clone, Default)]
pub struct TotpViewScreen;

impl TotpViewScreen {
    pub fn view<'a>(&'a self, entries: &'a [Entry]) -> Element<'a, Message> {
        // ── 顶栏 ──────────────────────────────────────────────────────────
        let topbar = container(
            row![
                button(
                    text(icons::ARROW_BACK)
                        .font(MATERIAL_ICONS)
                        .size(22)
                        .color(Color::WHITE)
                )
                .on_press(Message::NavigateTo(NavigationTarget::List))
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
                Space::with_width(8),
                text(icons::TIMER)
                    .font(MATERIAL_ICONS)
                    .size(20)
                    .color(Color::WHITE),
                text("TOTP 验证码").size(18).color(Color::WHITE),
                Space::with_width(Length::Fill),
            ]
            .align_y(Alignment::Center)
            .spacing(6)
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
        let sidebar_container = create_sidebar(NavigationTarget::TotpView, totp_count);

        // ── 筛选有 TOTP 的条目 ──────────────────────────────────────────
        let totp_entries: Vec<&Entry> = entries.iter().filter(|e| e.totp.is_some()).collect();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let period = 30u64;
        let remaining = period - (now % period);
        let progress = remaining as f32 / period as f32;

        // 进度条颜色：剩余 < 7 秒显示橙色警告
        let bar_color = if remaining < 7 {
            t::WARNING
        } else {
            t::PRIMARY
        };

        // ── 条目卡片列表 ────────────────────────────────────────────────
        let mut cards = column![].spacing(12);

        if totp_entries.is_empty() {
            cards = cards.push(
                container(
                    column![
                        text(icons::TIMER)
                            .font(MATERIAL_ICONS)
                            .size(48)
                            .color(t::ON_SURFACE_VARIANT),
                        Space::with_height(8),
                        text("暂无启用 TOTP 的条目")
                            .size(14)
                            .color(t::ON_SURFACE_VARIANT),
                    ]
                    .align_x(Alignment::Center)
                    .spacing(4),
                )
                .width(Length::Fill)
                .center_x(Length::Fill)
                .padding(40),
            );
        } else {
            for entry in &totp_entries {
                let code = if let Some(totp_data) = &entry.totp {
                    vaultx_core::totp::TotpEngine::compute(totp_data)
                        .map(|r| r.code.clone())
                        .unwrap_or_else(|_| "------".to_string())
                } else {
                    "------".to_string()
                };

                let issuer = entry
                    .totp
                    .as_ref()
                    .map(|td| td.issuer.clone())
                    .unwrap_or_default();

                // 进度条（手动绘制为薄色块行）
                let bar_filled = container(Space::with_width(Length::FillPortion(
                    (progress * 100.0) as u16,
                )))
                .height(4)
                .style(move |_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(bar_color)),
                    border: Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

                let bar_empty = container(Space::with_width(Length::FillPortion(
                    100u16.saturating_sub((progress * 100.0) as u16),
                )))
                .height(4)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(t::SURFACE_VARIANT)),
                    border: Border {
                        radius: 2.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

                let progress_bar = row![bar_filled, bar_empty].width(Length::Fill);

                let card = container(
                    column![
                        // 条目标题 + 发行者
                        row![
                            text(entry.title.clone()).size(14).color(t::ON_SURFACE),
                            Space::with_width(Length::Fill),
                            text(issuer.to_string())
                                .size(12)
                                .color(t::ON_SURFACE_VARIANT),
                        ]
                        .align_y(Alignment::Center),
                        Space::with_height(4),
                        // 验证码大字 + 倒计时 + 复制按钮
                        row![
                            text(code.clone()).size(32).color(if remaining < 7 {
                                t::WARNING
                            } else {
                                t::PRIMARY
                            }),
                            Space::with_width(Length::Fill),
                            text(format!("{}s", remaining))
                                .size(13)
                                .color(if remaining < 7 {
                                    t::WARNING
                                } else {
                                    t::ON_SURFACE_VARIANT
                                }),
                            Space::with_width(6),
                            button(
                                text(icons::CONTENT_COPY)
                                    .font(MATERIAL_ICONS)
                                    .size(18)
                                    .color(t::PRIMARY),
                            )
                            .on_press(Message::CopyToClipboard(code.clone()))
                            .padding(6),
                        ]
                        .align_y(Alignment::Center),
                        Space::with_height(8),
                        progress_bar,
                    ]
                    .spacing(0)
                    .padding([12, 16]),
                )
                .width(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::WHITE)),
                    border: Border {
                        radius: 10.0.into(),
                        ..Default::default()
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.06),
                        offset: Vector::new(0.0, 2.0),
                        blur_radius: 12.0,
                    },
                    ..Default::default()
                });

                cards = cards.push(card);
            }
        }

        // cards 列直接传给 scrollable（不包 container）
        let cards_padded = cards.padding(iced::Padding {
            top: 16.0,
            right: 20.0,
            bottom: 16.0,
            left: 20.0,
        });

        let content_container = container(
            scrollable(cards_padded)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(t::SURFACE)),
            ..Default::default()
        });

        // 最终布局：顶栏全宽，侧边栏 + 内容区在下方
        column![
            topbar,
            row![sidebar_container, content_container,].height(Length::Fill),
        ]
        .into()
    }
}
