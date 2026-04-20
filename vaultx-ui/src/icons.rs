/// Material Icons — Unicode 码点常量（使用 MaterialIcons-Regular.ttf）
use crate::app::MATERIAL_ICONS;
use iced::widget::text;

pub const LOCK: &str = "\u{E897}";
pub const LOCK_OPEN: &str = "\u{E898}";
pub const WARNING: &str = "\u{E002}";
pub const ADD_CIRCLE: &str = "\u{E147}";
pub const CONTENT_COPY: &str = "\u{E14D}";
pub const VISIBILITY: &str = "\u{E8F4}";
pub const VISIBILITY_OFF: &str = "\u{E8F5}";
pub const PERSON: &str = "\u{E7FD}";
pub const KEY: &str = "\u{E73C}";
pub const SEARCH: &str = "\u{E8B6}";
pub const SETTINGS: &str = "\u{E8B8}";
pub const DELETE: &str = "\u{E872}";
pub const EDIT: &str = "\u{E3C9}";
pub const CLOSE: &str = "\u{E5CD}";
pub const HOME: &str = "\u{E88A}";
pub const ADD: &str = "\u{E145}";
pub const PHONELINK_LOCK: &str = "\u{E0DC}";
pub const SHIELD: &str = "\u{E9E0}";
pub const CHECK_CIRCLE: &str = "\u{E86C}";
pub const MORE_VERT: &str = "\u{E5D4}";
pub const TIMER: &str = "\u{E425}";
pub const STAR: &str = "\u{E838}";
pub const LOGOUT: &str = "\u{E9BA}";
pub const VPN_KEY: &str = "\u{E0DA}";
pub const REFRESH: &str = "\u{E5D5}";

/// 便捷函数：创建图标文本 widget
pub fn icon(codepoint: &'static str) -> iced::widget::Text<'static> {
    text(codepoint).font(MATERIAL_ICONS)
}

/// 带颜色的图标
pub fn icon_color(codepoint: &'static str, color: iced::Color) -> iced::widget::Text<'static> {
    text(codepoint).font(MATERIAL_ICONS).color(color)
}
