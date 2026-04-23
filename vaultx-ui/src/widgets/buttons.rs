use crate::app::{Message, MATERIAL_ICONS};
use crate::theme as t;
use iced::{
    widget::{button, container, row, text, Space},
    Alignment, Border, Color, Element, Length, Shadow, Vector,
};

/// 蓝色顶栏圆形图标按钮（导航类：关闭 / 返回）
///
/// - 默认透明背景，鼠标悬停或按下时显示半透明白色圆圈
/// - 尺寸：32 × 32 px，圆角半径 16
///
/// # 参数
/// - `icon`：图标字符（来自 `crate::icons`）
/// - `on_press`：点击时发送的消息
pub fn topbar_icon_button(icon: &'static str, on_press: Message) -> Element<'static, Message> {
    button(
        container(
            text(icon)
                .font(MATERIAL_ICONS)
                .size(22)
                .line_height(1.0)
                .color(Color::WHITE),
        )
        .width(32)
        .height(32)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(on_press)
    .padding(0)
    .style(|_: &iced::Theme, status| iced::widget::button::Style {
        background: Some(iced::Background::Color(match status {
            iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                Color::from_rgba(1.0, 1.0, 1.0, 0.25)
            }
            _ => Color::TRANSPARENT,
        })),
        border: Border {
            radius: 16.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .into()
}

/// 蓝色主色调顶栏（Primary Topbar）
///
/// 结构：`[nav_btn] [8px] [title_icon] [title] [Fill] [可选右侧内容]`
///
/// # 参数
/// - `nav_icon`：导航按钮图标（通常是 CLOSE 或 ARROW_BACK）
/// - `nav_msg`：导航按钮点击消息
/// - `title_icon`：标题区图标
/// - `title`：标题文字
/// - `right`：可选的右侧内容（如「保存」按钮）
pub fn primary_topbar<'a>(
    nav_icon: &'static str,
    nav_msg: Message,
    title_icon: &'static str,
    title: &'a str,
    right: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let row_base = row![
        topbar_icon_button(nav_icon, nav_msg),
        Space::with_width(8),
        text(title_icon)
            .font(MATERIAL_ICONS)
            .size(20)
            .color(Color::WHITE),
        text(title).size(18).color(Color::WHITE),
        Space::with_width(Length::Fill),
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .padding([0, 16]);

    let row_final = if let Some(right_content) = right {
        row_base.push(right_content)
    } else {
        row_base
    };

    container(row_final)
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
        })
        .into()
}
