use crate::app::{Message, NavigationTarget, MATERIAL_ICONS};
use crate::icons;
use crate::theme::{self as t};
use crate::widgets::sidebar::create_sidebar;
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Alignment, Border, Color, Element, Length, Shadow, Vector,
};

/// 新建条目页状态
#[derive(Debug, Clone)]
pub struct NewEntryScreen {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    #[allow(dead_code)]
    pub notes: String,
    pub totp_secret: String,
    pub totp_issuer: String,
    pub include_totp: bool,
    pub show_password: bool,
    pub error_message: Option<String>,
    // Section 折叠状态
    pub password_section_expanded: bool,
    pub totp_section_expanded: bool,
    // 切换动画进度 (0.0 = 关/左, 1.0 = 开/右)
    pub password_toggle_anim: f32,
    pub totp_toggle_anim: f32,
    // 分类选择
    pub category: Option<String>,
}

impl Default for NewEntryScreen {
    fn default() -> Self {
        Self {
            title: String::new(),
            username: String::new(),
            password: String::new(),
            url: String::new(),
            notes: String::new(),
            totp_secret: String::new(),
            totp_issuer: String::new(),
            include_totp: false,
            show_password: false,
            error_message: None,
            password_section_expanded: true, // 密码section默认展开
            totp_section_expanded: false,    // TOTP section默认折叠
            password_toggle_anim: 1.0,       // 对应 expanded = true
            totp_toggle_anim: 0.0,           // 对应 expanded = false
            category: None,
        }
    }
}

impl NewEntryScreen {
    pub fn view<'a>(&'a self) -> Element<'a, Message> {
        // ── 顶栏 ──────────────────────────────────────────────────────────
        let topbar = container(
            row![
                button(text(icons::CLOSE).font(MATERIAL_ICONS).size(22).color(Color::WHITE))
                    .on_press(Message::NavigateTo(NavigationTarget::List))
                    .padding(8)
                    .style(|_, status| button::Style {
                        background: Some(iced::Background::Color(match status {
                            button::Status::Hovered => Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                            _ => Color::TRANSPARENT,
                        })),
                        text_color: Color::WHITE,
                        border: Border {
                            color: Color::TRANSPARENT,
                            width: 0.0,
                            radius: 100.0.into(),
                        },
                        shadow: Shadow::default(),
                    }),
                Space::with_width(8),
                text(icons::ADD_CIRCLE)
                    .font(MATERIAL_ICONS)
                    .size(20)
                    .color(Color::WHITE),
                text("新建条目").size(18).color(Color::WHITE),
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
                .padding([8, 16])
                .style(|_, status| button::Style {
                    background: Some(iced::Background::Color(match status {
                        button::Status::Hovered => Color::from_rgba(1.0, 1.0, 1.0, 0.15),
                        _ => Color::TRANSPARENT,
                    })),
                    text_color: Color::WHITE,
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 8.0.into(),
                    },
                    shadow: Shadow::default(),
                }),
            ]
            .align_y(Alignment::Center)
            .spacing(6)
            .padding([0, 12]),
        )
        .height(56)
        .width(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(t::PRIMARY)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        });

        let vis_icon = if self.show_password {
            icons::VISIBILITY_OFF
        } else {
            icons::VISIBILITY
        };

        // ── 表单内容 ──────────────────────────────────────────────────────
        
        // 构建密码区块内容
        let password_section_body = if self.password_section_expanded {
            Some(
                column![
                    label_field("网址 (URL)"),
                    text_input("https://example.com", &self.url)
                        .on_input(Message::NewEntryUrlChanged)
                        .size(14)
                        .padding([8, 10]),
                    Space::with_height(12),
                    label_field("用户名"),
                    text_input("用户名或邮箱", &self.username)
                        .on_input(Message::NewEntryUsernameChanged)
                        .size(14)
                        .padding([8, 10]),
                    Space::with_height(12),
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
                ]
                .spacing(4),
            )
        } else {
            None
        };

        // 构建 TOTP 区块内容
        let totp_section_body = if self.totp_section_expanded {
            Some(
                column![
                    label_field("密钥 (Secret) *"),
                    text_input("JBSWY3DPEHPK3PXP", &self.totp_secret)
                        .on_input(Message::NewEntryTotpSecretChanged)
                        .size(14)
                        .padding([8, 10]),
                    Space::with_height(12),
                    label_field("发行者 (Issuer)"),
                    text_input("GitHub / Google / ...", &self.totp_issuer)
                        .on_input(Message::NewEntryTotpIssuerChanged)
                        .size(14)
                        .padding([8, 10]),
                ]
                .spacing(4),
            )
        } else {
            None
        };

        // 密码区块：可折叠区块容器
        let password_section = {
            let toggle_widget =
                toggle_switch(self.password_toggle_anim, Message::NewEntryTogglePasswordSection);

            // 区块标题栏
            let section_header = button(
                container(
                    row![
                        row![
                            text(icons::KEY)
                                .font(MATERIAL_ICONS)
                                .size(16)
                                .color(t::ON_SURFACE),
                            Space::with_width(8),
                            text("密码").size(14),
                        ]
                        .align_y(Alignment::Center),
                        Space::with_width(Length::Fill),
                        toggle_widget,
                    ]
                    .align_y(Alignment::Center),
                )
                .padding([12, 16])
                .width(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(t::SURFACE_VARIANT)),
                    border: Border {
                        radius: 0.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            )
            .on_press(Message::NewEntryTogglePasswordSection)
            .style(|_, _| button::Style {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                border: Border::default(),
                shadow: Shadow::default(),
                text_color: Color::BLACK,
            });

            // 区块主体
            let mut section_col = column![section_header];
            if let Some(body) = password_section_body {
                section_col = section_col.push(
                    container(body)
                        .padding(16)
                        .width(Length::Fill)
                        .style(|_: &iced::Theme| iced::widget::container::Style {
                            background: Some(iced::Background::Color(Color::WHITE)),
                            ..Default::default()
                        }),
                );
            }

            container(section_col)
                .width(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::WHITE)),
                    border: Border {
                        color: t::OUTLINE_VARIANT,
                        width: 1.5,
                        radius: 12.0.into(),
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
                        offset: Vector::new(0.0, 2.0),
                        blur_radius: 8.0,
                    },
                    ..Default::default()
                })
        };

        // TOTP 区块：可折叠区块容器
        let totp_section = {
            let toggle_widget =
                toggle_switch(self.totp_toggle_anim, Message::NewEntryToggleTotpSection);

            // 区块标题栏
            let section_header = button(
                container(
                    row![
                        row![
                            text(icons::PHONELINK_LOCK)
                                .font(MATERIAL_ICONS)
                                .size(16)
                                .color(t::ON_SURFACE),
                            Space::with_width(8),
                            text("TOTP 双因素验证")
                                .size(14),
                        ]
                        .align_y(Alignment::Center),
                        Space::with_width(Length::Fill),
                        toggle_widget,
                    ]
                    .align_y(Alignment::Center),
                )
                .padding([12, 16])
                .width(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(t::SURFACE_VARIANT)),
                    border: Border {
                        radius: 0.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            )
            .on_press(Message::NewEntryToggleTotpSection)
            .style(|_, _| button::Style {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                border: Border::default(),
                shadow: Shadow::default(),
                text_color: Color::BLACK,
            });

            // 区块主体
            let mut section_col = column![section_header];
            if let Some(body) = totp_section_body {
                section_col = section_col.push(
                    container(body)
                        .padding(16)
                        .width(Length::Fill)
                        .style(|_: &iced::Theme| iced::widget::container::Style {
                            background: Some(iced::Background::Color(Color::WHITE)),
                            ..Default::default()
                        }),
                );
            }

            container(section_col)
                .width(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::WHITE)),
                    border: Border {
                        color: t::OUTLINE_VARIANT,
                        width: 1.5,
                        radius: 12.0.into(),
                    },
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
                        offset: Vector::new(0.0, 2.0),
                        blur_radius: 8.0,
                    },
                    ..Default::default()
                })
        };

        let mut form = column![
            // 标题（必填）
            label_field("标题 *"),
            text_input("条目名称（必填）", &self.title)
                .on_input(Message::NewEntryTitleChanged)
                .size(14)
                .padding([8, 10]),
            Space::with_height(12),
            // 密码区块
            password_section,
            Space::with_height(12),
            // TOTP 区块
            totp_section,
        ]
        .spacing(4);

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

        // ── 侧边栏：使用可复用的侧边栏组件 ────────────────────────────────
        let sidebar_container = create_sidebar(NavigationTarget::NewEntry, 0);

        column![topbar, row![sidebar_container, content]].into()
    }
}

fn label_field(label: &str) -> iced::widget::Text<'static> {
    iced::widget::text(label.to_string())
        .size(12)
        .color(t::ON_SURFACE_VARIANT)
}

/// Material Design 风格切换开关（Toggle Switch）
///
/// 尺寸规格：
/// - 轨道：44 × 24 px，两端圆角（radius 12）
/// - 滑块（圆形）：20 × 20 px（radius 10），带 0.5px 描边和轻微阴影
/// - 水平内边距：左右各 2 px，圆形滑动行程 = 44 - 20 - 4 = 20 px
///
/// 参数：
/// - `anim_progress`：动画进度，`0.0` = 完全关闭（圆在左），`1.0` = 完全开启（圆在右）；
///   支持中间值以实现平滑滑动动画。
/// - `on_press`：用户点击开关时发送的 [`Message`]。
///
/// 颜色插值：
/// - 关闭状态轨道色 `rgb(0.78, 0.78, 0.78)`（浅灰）→ 开启状态 `t::PRIMARY`（蓝色）
fn toggle_switch(anim_progress: f32, on_press: crate::app::Message) -> iced::Element<'static, crate::app::Message> {
    use iced::widget::{button, container, row, Space};
    use iced::{Alignment, Border, Color, Shadow, Vector};

    let toggle_bg = lerp_color(
        Color::from_rgb(0.78, 0.78, 0.78),
        t::PRIMARY,
        anim_progress,
    );
    let offset = anim_progress * 20.0;

    let toggle_circle = container(Space::with_width(0))
        .width(20)
        .height(20)
        .style(move |_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(Color::WHITE)),
            border: Border {
                radius: 10.0.into(),
                width: 0.5,
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 3.0,
            },
            ..Default::default()
        });

    let toggle_content = row![Space::with_width(offset), toggle_circle]
        .align_y(Alignment::Center);

    button(
        container(toggle_content)
            .width(44)
            .center_y(24)
            .padding([0, 2])
            .style(move |_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(toggle_bg)),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
    )
    .on_press(on_press)
    .style(|_, _| button::Style {
        background: Some(iced::Background::Color(Color::TRANSPARENT)),
        border: Border::default(),
        shadow: Shadow::default(),
        text_color: Color::BLACK,
    })
    .into()
}

fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    Color {
        r: from.r + (to.r - from.r) * t,
        g: from.g + (to.g - from.g) * t,
        b: from.b + (to.b - from.b) * t,
        a: 1.0,
    }
}
