use crate::app::{Message, NavigationTarget, MATERIAL_ICONS};
use crate::icons;
use crate::theme as t;
use iced::{
    widget::{button, column, container, row, text, Space},
    Alignment, Border, Color, Element, Length,
};

/// 创建标准侧边栏
///
/// # 参数
/// - `current_target`: 当前激活的导航目标
/// - `totp_count`: TOTP 条目数量（用于显示徽章）
pub fn create_sidebar<'a>(
    current_target: NavigationTarget,
    totp_count: usize,
) -> Element<'a, Message> {
    let nav_item = |icon_cp: &'a str,
                    label: &'a str,
                    badge: Option<usize>,
                    active: bool,
                    target: NavigationTarget|
     -> Element<'a, Message> {
        let icon_color = if active { t::PRIMARY } else { t::ON_SURFACE };
        let label_color = if active { t::PRIMARY } else { t::ON_SURFACE };
        let bg = if active {
            Some(iced::Background::Color(t::PRIMARY_CONTAINER)) // #BBDEFB
        } else {
            None
        };
        let pad_left = if active { 7.0_f32 } else { 10.0_f32 };

        let mut content_row = row![
            text(icon_cp)
                .font(MATERIAL_ICONS)
                .size(16)
                .color(icon_color),
            text(label).size(13).color(label_color),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        // 添加徽章
        if let Some(count) = badge {
            content_row = content_row.push(Space::with_width(Length::Fill));
            content_row = content_row.push(
                container(text(count.to_string()).size(11).color(t::ON_SURFACE))
                    .padding([2, 6])
                    .style(|_: &iced::Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            0.5, 0.5, 0.5, 0.2,
                        ))),
                        border: Border {
                            radius: 10.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            );
        }

        let inner = container(content_row.width(Length::Fill))
            .padding(iced::Padding {
                top: 9.0,
                right: 10.0,
                bottom: 9.0,
                left: pad_left,
            })
            .width(Length::Fill);

        let styled_btn = button(inner)
            .on_press(Message::NavigateTo(target))
            .width(Length::Fill)
            .padding(0)
            .style(move |_: &iced::Theme, _| iced::widget::button::Style {
                background: bg,
                border: Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

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

    // 判断当前激活的导航项
    let is_list_active = matches!(current_target, NavigationTarget::List);
    let is_totp_active = matches!(current_target, NavigationTarget::TotpView);
    let is_generator_active = matches!(current_target, NavigationTarget::Generator);
    let is_settings_active = matches!(current_target, NavigationTarget::Settings);

    let sidebar_col = column![
        // 全部条目
        nav_item(
            icons::FOFDER_OPEN,
            "全部条目",
            None,
            is_list_active,
            NavigationTarget::List
        ),
        // 分隔线
        container(Space::with_height(1))
            .width(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(
                    0.773, 0.859, 0.941
                ))),
                ..Default::default()
            }),
        // 类型分组标题
        container(text("类型").size(11).color(t::ON_SURFACE_VARIANT)).padding(iced::Padding {
            top: 8.0,
            right: 4.0,
            bottom: 4.0,
            left: 14.0
        }),
        nav_item(icons::KEY, "密码", None, false, NavigationTarget::List),
        // TOTP 项（使用 badge 显示计数）
        nav_item(
            icons::PHONELINK_LOCK,
            "TOTP",
            Some(totp_count),
            is_totp_active,
            NavigationTarget::TotpView
        ),
        // 分隔线
        container(Space::with_height(1))
            .width(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(
                    0.773, 0.859, 0.941
                ))),
                ..Default::default()
            }),
        // 工具分组标题
        container(text("工具").size(11).color(t::ON_SURFACE_VARIANT)).padding(iced::Padding {
            top: 8.0,
            right: 4.0,
            bottom: 4.0,
            left: 14.0
        }),
        nav_item(
            icons::VPN_KEY,
            "密码生成器",
            None,
            is_generator_active,
            NavigationTarget::Generator
        ),
        // 底部空白
        Space::with_height(Length::Fill),
        // 设置
        nav_item(
            icons::SETTINGS,
            "设置",
            None,
            is_settings_active,
            NavigationTarget::Settings
        ),
    ]
    .spacing(2)
    .padding([8, 0]);

    container(sidebar_col)
        .width(200)
        .height(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(t::SURFACE_VARIANT)),
            ..Default::default()
        })
        .into()
}
