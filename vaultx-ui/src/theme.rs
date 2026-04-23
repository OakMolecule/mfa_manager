use iced::color;

// ─── Material Design Blue - 亮色 ───────────────────────────────────────────
pub const PRIMARY: iced::Color = color!(0x1976D2);
#[allow(dead_code)]
pub const PRIMARY_LIGHT: iced::Color = color!(0x42A5F5);
#[allow(dead_code)]
pub const PRIMARY_DARK: iced::Color = color!(0x0D47A1);
#[allow(dead_code)]
pub const PRIMARY_CONTAINER: iced::Color = color!(0xBBDEFB);
#[allow(dead_code)]
pub const ON_PRIMARY: iced::Color = iced::Color::WHITE;

pub const SURFACE: iced::Color = color!(0xFAFAFA);
pub const SURFACE_VARIANT: iced::Color = color!(0xE3F2FD);
pub const CARD_BG: iced::Color = iced::Color::WHITE;
pub const ON_SURFACE: iced::Color = color!(0x1A1A1A);
pub const ON_SURFACE_VARIANT: iced::Color = color!(0x455A64);

pub const OUTLINE: iced::Color = color!(0x90A4AE);
pub const OUTLINE_VARIANT: iced::Color = color!(0xC5CAE9);

pub const TOPBAR_BG: iced::Color = color!(0x1976D2);
pub const SIDEBAR_BG: iced::Color = color!(0xE3F2FD);

pub const ERROR: iced::Color = color!(0xD32F2F);
pub const WARNING: iced::Color = color!(0xFF6D00);
#[allow(dead_code)]
pub const SUCCESS: iced::Color = color!(0x2E7D32);

// ─── Material Design Blue - 暗色 ───────────────────────────────────────────
#[allow(dead_code)]
pub mod dark {
    use iced::color;

    pub const PRIMARY: iced::Color = color!(0x90CAF9);
    pub const PRIMARY_CONTAINER: iced::Color = color!(0x0D47A1);

    pub const SURFACE: iced::Color = color!(0x121212);
    pub const SURFACE_VARIANT: iced::Color = color!(0x1E2A3A);
    pub const CARD_BG: iced::Color = color!(0x1E1E1E);
    pub const ON_SURFACE: iced::Color = color!(0xE8EAED);
    pub const ON_SURFACE_VARIANT: iced::Color = color!(0x90A4AE);

    pub const OUTLINE: iced::Color = color!(0x455A64);
    pub const OUTLINE_VARIANT: iced::Color = color!(0x2A3A4A);

    pub const TOPBAR_BG: iced::Color = color!(0x0D1B2A);
    pub const SIDEBAR_BG: iced::Color = color!(0x1A2433);

    pub const ERROR: iced::Color = color!(0xEF9A9A);
}

// ─── 圆角半径 ────────────────────────────────────────────────────────────────
#[allow(dead_code)]
pub mod radius {
    pub const CARD: f32 = 12.0;
    pub const BUTTON: f32 = 8.0;
    pub const INPUT: f32 = 8.0;
    pub const DIALOG: f32 = 16.0;
}

// ─── 字体大小 ────────────────────────────────────────────────────────────────
#[allow(dead_code)]
pub mod font_size {
    pub const TOPBAR_TITLE: f32 = 18.0;
    pub const CARD_TITLE: f32 = 15.0;
    pub const TOTP_CODE: f32 = 22.0;
    pub const BODY: f32 = 14.0;
    pub const SECONDARY: f32 = 13.0;
    pub const LABEL: f32 = 11.0;
    pub const MONO: f32 = 13.0;
}

// ─── 布局尺寸 ────────────────────────────────────────────────────────────────
#[allow(dead_code)]
pub mod layout {
    pub const TOPBAR_HEIGHT: f32 = 56.0;
    pub const SIDEBAR_WIDTH: f32 = 200.0;
    pub const APP_WIDTH: f32 = 920.0;
    pub const APP_HEIGHT: f32 = 640.0;
}
