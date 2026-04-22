use crate::screens::{
    detail::DetailScreen, generator::GeneratorScreen, list::ListScreen, new_entry::NewEntryScreen,
    settings::SettingsScreen, totp_view::TotpViewScreen, unlock::UnlockScreen,
};
use iced::{Element, Subscription, Task, Theme};
use secrecy::SecretString;
use std::path::PathBuf;
use vaultx_core::{vault::VaultError, Vault};

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 应用程序主题偏好
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreference {
    #[default]
    Light,
    Dark,
    #[allow(dead_code)]
    System,
}

/// 应用程序全局状态
pub struct VaultApp {
    /// 当前显示的屏幕
    screen: Screen,
    /// 金库（解锁后持有）
    vault: Option<Vault>,
    /// 解锁异步任务完成后暂存 Vault 的共享槽
    pending_vault: std::sync::Arc<std::sync::Mutex<Option<Vault>>>,
    /// 当前金库文件路径
    vault_path: Option<PathBuf>,
    /// 主题偏好
    theme_pref: ThemePreference,
    /// 密码错误次数（用于退避锁定）
    error_count: u32,
    /// 退避锁定解除时间（Unix 秒）
    lockout_until: Option<u64>,
    /// 最后活动时间（Unix 秒）
    last_activity: u64,
    /// 自动锁定超时（秒），0 = 禁用
    auto_lock_timeout: u64,
    /// 上次复制内容到剪贴板的时间（Unix 秒），用于自动清除
    clipboard_copied_at: Option<u64>,
    /// 剪贴板清除延迟（秒）
    clipboard_clear_seconds: u64,
    /// 密码错误最大次数（触发退避锁定）
    max_error_count: u32,
    /// 密码生成器（抽屉式）
    generator: GeneratorScreen,
    /// 密码生成器是否打开
    generator_open: bool,
}

/// 屏幕枚举
#[derive(Debug, Clone)]
pub enum Screen {
    Unlock(UnlockScreen),
    List(ListScreen),
    Detail(DetailScreen),
    TotpView(TotpViewScreen),
    Generator(GeneratorScreen),
    Settings(SettingsScreen),
    NewEntry(NewEntryScreen),
}

/// 全局消息枚举
#[derive(Debug, Clone)]
pub enum Message {
    // ── 解锁 ──────────────────────────────────────────────────
    PasswordChanged(String),
    UnlockPressed,
    CreateVaultPressed,
    UnlockSuccess,
    UnlockFailed(String),

    // ── 导航 ──────────────────────────────────────────────────
    NavigateTo(NavigationTarget),
    #[allow(dead_code)]
    GoBack,

    // ── 列表 ──────────────────────────────────────────────────
    SearchChanged(String),
    CopyToClipboard(String),
    TogglePasswordVisible(uuid::Uuid),
    TotpTick,

    // ── 条目操作 ───────────────────────────────────────────────
    SelectEntry(uuid::Uuid),
    DeleteEntry(uuid::Uuid),
    SaveEntry,

    // ── 主题 ──────────────────────────────────────────────────
    SetTheme(ThemePreference),
    SetAutoLockTimeout(u64),
    SetClipboardClearSeconds(u64),
    SetMaxErrorCount(u32),

    // ── 新建条目 ───────────────────────────────────────────────
    NewEntryTitleChanged(String),
    NewEntryUsernameChanged(String),
    NewEntryPasswordChanged(String),
    NewEntryUrlChanged(String),
    NewEntryToggleShowPassword,
    NewEntryToggleTotp,
    NewEntryTotpSecretChanged(String),
    NewEntryTotpIssuerChanged(String),
    NewEntryTogglePasswordSection,
    NewEntryToggleTotpSection,
    NewEntrySetCategory(Option<String>),
    ToggleAnimTick,

    // ── 生成器 ────────────────────────────────────────────────
    OpenGenerator,
    CloseGenerator,
    GeneratePassword,
    GeneratorLengthChanged(u8),
    GeneratorToggleUppercase,
    GeneratorToggleLowercase,
    GeneratorToggleDigits,
    GeneratorToggleSymbols,
    GeneratorToggleAmbiguous,

    // ── 系统 ──────────────────────────────────────────────────
    LockVault,
    #[allow(dead_code)]
    Noop,
}

#[derive(Debug, Clone)]
pub enum NavigationTarget {
    List,
    Detail(uuid::Uuid),
    TotpView,
    Generator,
    Settings,
    NewEntry,
}

impl VaultApp {
    pub fn new() -> (Self, Task<Message>) {
        let vault_path = default_vault_path();
        let screen = if vault_path.as_ref().map_or(false, |p| p.exists()) {
            Screen::Unlock(UnlockScreen::default())
        } else {
            Screen::Unlock(UnlockScreen::new_vault_mode())
        };

        (
            Self {
                screen,
                vault: None,
                pending_vault: Default::default(),
                vault_path,
                theme_pref: ThemePreference::Light,
                error_count: 0,
                lockout_until: None,
                last_activity: now_secs(),
                auto_lock_timeout: 300, // 默认 5 分钟
                clipboard_copied_at: None,
                clipboard_clear_seconds: 30, // 默认 30 秒
                max_error_count: 5,          // 默认最多 5 次
                generator: GeneratorScreen::default(),
                generator_open: false,
            },
            Task::none(),
        )
    }

    pub fn title(&self) -> String {
        "VaultX — 密码管理器".to_string()
    }

    pub fn theme(&self) -> Theme {
        match self.theme_pref {
            ThemePreference::Dark => Theme::Dark,
            _ => Theme::Light,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PasswordChanged(pw) => {
                if let Screen::Unlock(s) = &mut self.screen {
                    s.password = pw;
                }
                Task::none()
            }
            Message::UnlockPressed => {
                let Screen::Unlock(s) = &mut self.screen else {
                    return Task::none();
                };
                if s.password.is_empty() {
                    s.error_message = Some("请输入主密码".to_string());
                    return Task::none();
                }
                if let Some(until) = self.lockout_until {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if now < until {
                        s.error_message = Some(format!("多次错误，请 {} 秒后重试", until - now));
                        return Task::none();
                    }
                }
                s.is_loading = true;
                s.error_message = None;
                let password = SecretString::new(s.password.clone().into());
                let vault_path = self.vault_path.clone();
                let slot = self.pending_vault.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || match &vault_path {
                            Some(path) => {
                                let mut v = Vault::open(path)?;
                                v.unlock(&password)?;
                                *slot.lock().unwrap() = Some(v);
                                Ok::<(), VaultError>(())
                            }
                            None => Err(VaultError::Io(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "未找到金库路径",
                            ))),
                        })
                        .await
                        .unwrap_or_else(|e| {
                            Err(VaultError::Io(std::io::Error::other(e.to_string())))
                        })
                    },
                    |result| match result {
                        Ok(()) => Message::UnlockSuccess,
                        Err(e) => Message::UnlockFailed(e.to_string()),
                    },
                )
            }
            Message::CreateVaultPressed => {
                let Screen::Unlock(s) = &mut self.screen else {
                    return Task::none();
                };
                if s.password.is_empty() {
                    s.error_message = Some("请输入主密码".to_string());
                    return Task::none();
                }
                s.is_loading = true;
                s.error_message = None;
                let password = SecretString::new(s.password.clone().into());
                let vault_path = self.vault_path.clone();
                let slot = self.pending_vault.clone();
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || match &vault_path {
                            Some(path) => {
                                if let Some(parent) = path.parent() {
                                    std::fs::create_dir_all(parent)?;
                                }
                                let v = Vault::create(path, &password)?;
                                *slot.lock().unwrap() = Some(v);
                                Ok::<(), VaultError>(())
                            }
                            None => Err(VaultError::Io(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "未找到金库路径",
                            ))),
                        })
                        .await
                        .unwrap_or_else(|e| {
                            Err(VaultError::Io(std::io::Error::other(e.to_string())))
                        })
                    },
                    |result| match result {
                        Ok(()) => Message::UnlockSuccess,
                        Err(e) => Message::UnlockFailed(e.to_string()),
                    },
                )
            }
            Message::UnlockSuccess => {
                let vault = self.pending_vault.lock().unwrap().take();
                self.vault = vault;
                self.error_count = 0;
                self.lockout_until = None;
                self.screen = Screen::List(ListScreen::default());
                Task::none()
            }
            Message::UnlockFailed(err) => {
                self.error_count += 1;
                let backoff = if self.error_count >= self.max_error_count {
                    30
                } else if self.error_count >= (self.max_error_count / 2).max(2) {
                    10
                } else {
                    0
                };
                if backoff > 0 {
                    let until = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        + backoff;
                    self.lockout_until = Some(until);
                }
                if let Screen::Unlock(s) = &mut self.screen {
                    s.is_loading = false;
                    s.password.clear();
                    s.error_message = Some(err);
                }
                Task::none()
            }
            Message::LockVault => {
                if let Some(vault) = self.vault.as_mut() {
                    vault.lock();
                }
                self.screen = Screen::Unlock(UnlockScreen::default());
                Task::none()
            }
            Message::SetTheme(pref) => {
                self.theme_pref = pref;
                Task::none()
            }
            Message::SetAutoLockTimeout(secs) => {
                self.auto_lock_timeout = secs;
                self.last_activity = now_secs();
                Task::none()
            }
            Message::SetClipboardClearSeconds(secs) => {
                self.clipboard_clear_seconds = secs;
                Task::none()
            }
            Message::SetMaxErrorCount(count) => {
                self.max_error_count = count;
                Task::none()
            }
            Message::CopyToClipboard(text) => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(text);
                }
                self.clipboard_copied_at = Some(now_secs());
                self.last_activity = now_secs();
                Task::none()
            }
            Message::NavigateTo(target) => {
                self.last_activity = now_secs();
                self.navigate(target);
                Task::none()
            }
            Message::GoBack => {
                self.screen = Screen::List(ListScreen::default());
                Task::none()
            }
            Message::TotpTick => {
                let now = now_secs();
                // 自动锁定检查
                if self.vault.is_some()
                    && self.auto_lock_timeout > 0
                    && now.saturating_sub(self.last_activity) >= self.auto_lock_timeout
                {
                    if let Some(vault) = self.vault.as_mut() {
                        let _ = vault.lock();
                    }
                    self.vault = None;
                    self.screen = Screen::Unlock(UnlockScreen::default());
                    self.clipboard_copied_at = None;
                    return Task::none();
                }
                // 剪贴板自动清除（使用配置的延迟）
                if let Some(copied_at) = self.clipboard_copied_at {
                    if now.saturating_sub(copied_at) >= self.clipboard_clear_seconds {
                        if let Ok(mut cb) = arboard::Clipboard::new() {
                            let _ = cb.set_text("");
                        }
                        self.clipboard_copied_at = None;
                    }
                }
                Task::none()
            }
            Message::TogglePasswordVisible(id) => {
                match &mut self.screen {
                    Screen::List(s) => {
                        if s.visible_passwords.contains(&id) {
                            s.visible_passwords.remove(&id);
                        } else {
                            s.visible_passwords.insert(id);
                        }
                    }
                    Screen::Detail(s) => {
                        s.show_password = !s.show_password;
                    }
                    _ => {}
                }
                Task::none()
            }
            Message::SearchChanged(q) => {
                if let Screen::List(s) = &mut self.screen {
                    s.search_query = q;
                }
                Task::none()
            }
            Message::SelectEntry(id) => {
                self.navigate(NavigationTarget::Detail(id));
                Task::none()
            }
            Message::DeleteEntry(id) => {
                if let Some(vault) = self.vault.as_mut() {
                    if let Ok(entries) = vault.entries_mut() {
                        entries.retain(|e| e.id != id);
                        let _ = vault.save();
                    }
                }
                Task::none()
            }
            Message::NewEntryTitleChanged(v) => {
                if let Screen::NewEntry(s) = &mut self.screen {
                    s.title = v;
                }
                Task::none()
            }
            Message::NewEntryUsernameChanged(v) => {
                if let Screen::NewEntry(s) = &mut self.screen {
                    s.username = v;
                }
                Task::none()
            }
            Message::NewEntryPasswordChanged(v) => {
                if let Screen::NewEntry(s) = &mut self.screen {
                    s.password = v;
                }
                Task::none()
            }
            Message::NewEntryUrlChanged(v) => {
                if let Screen::NewEntry(s) = &mut self.screen {
                    s.url = v;
                }
                Task::none()
            }
            Message::NewEntryToggleShowPassword => {
                if let Screen::NewEntry(s) = &mut self.screen {
                    s.show_password = !s.show_password;
                }
                Task::none()
            }
            Message::NewEntryToggleTotp => {
                if let Screen::NewEntry(s) = &mut self.screen {
                    s.include_totp = !s.include_totp;
                }
                Task::none()
            }
            Message::NewEntryTotpSecretChanged(v) => {
                if let Screen::NewEntry(s) = &mut self.screen {
                    s.totp_secret = v;
                }
                Task::none()
            }
            Message::NewEntryTotpIssuerChanged(v) => {
                if let Screen::NewEntry(s) = &mut self.screen {
                    s.totp_issuer = v;
                }
                Task::none()
            }
            Message::NewEntryTogglePasswordSection => {
                if let Screen::NewEntry(s) = &mut self.screen {
                    s.password_section_expanded = !s.password_section_expanded;
                }
                Task::none()
            }
            Message::NewEntryToggleTotpSection => {
                if let Screen::NewEntry(s) = &mut self.screen {
                    s.totp_section_expanded = !s.totp_section_expanded;
                }
                Task::none()
            }
            Message::ToggleAnimTick => {
                const SPEED: f32 = 0.12;
                if let Screen::NewEntry(s) = &mut self.screen {
                    let pw_target = if s.password_section_expanded {
                        1.0_f32
                    } else {
                        0.0
                    };
                    s.password_toggle_anim = step_towards(s.password_toggle_anim, pw_target, SPEED);
                    let totp_target = if s.totp_section_expanded {
                        1.0_f32
                    } else {
                        0.0
                    };
                    s.totp_toggle_anim = step_towards(s.totp_toggle_anim, totp_target, SPEED);
                }
                Task::none()
            }
            Message::NewEntrySetCategory(cat) => {
                if let Screen::NewEntry(s) = &mut self.screen {
                    s.category = cat;
                }
                Task::none()
            }
            Message::SaveEntry => {
                let Screen::NewEntry(s) = &self.screen else {
                    return Task::none();
                };
                if s.title.is_empty() {
                    if let Screen::NewEntry(s) = &mut self.screen {
                        s.error_message = Some("标题不能为空".to_string());
                    }
                    return Task::none();
                }
                // 构建 Entry
                let pw_data =
                    if !s.username.is_empty() || !s.password.is_empty() || !s.url.is_empty() {
                        Some(vaultx_core::entry::PasswordData {
                            username: s.username.clone(),
                            password: s.password.clone(),
                            url: s.url.clone(),
                            notes: String::new(),
                        })
                    } else {
                        None
                    };
                let totp_data = if s.include_totp && !s.totp_secret.is_empty() {
                    Some(vaultx_core::entry::TotpData {
                        secret: s.totp_secret.clone(),
                        issuer: s.totp_issuer.clone(),
                        account: s.username.clone(),
                        algorithm: vaultx_core::entry::TotpAlgorithm::SHA1,
                        digits: 6,
                        period: 30,
                    })
                } else {
                    None
                };
                let title = s.title.clone();
                if pw_data.is_none() && totp_data.is_none() {
                    if let Screen::NewEntry(s) = &mut self.screen {
                        s.error_message = Some("请至少填写用户名/密码或启用 TOTP".to_string());
                    }
                    return Task::none();
                }
                let mut entry = vaultx_core::entry::Entry::new_password(
                    title,
                    pw_data.unwrap_or(vaultx_core::entry::PasswordData {
                        username: String::new(),
                        password: String::new(),
                        url: String::new(),
                        notes: String::new(),
                    }),
                );
                entry.totp = totp_data;
                if let Some(vault) = self.vault.as_mut() {
                    if let Ok(entries) = vault.entries_mut() {
                        entries.push(entry);
                        let _ = vault.save();
                    }
                }
                self.screen = Screen::List(ListScreen::default());
                Task::none()
            }
            Message::OpenGenerator => {
                self.generator_open = true;
                Task::none()
            }
            Message::CloseGenerator => {
                self.generator_open = false;
                Task::none()
            }
            Message::GeneratePassword => {
                self.generator.generated =
                    vaultx_core::PasswordGenerator::generate(&self.generator.config);
                Task::none()
            }
            Message::GeneratorLengthChanged(len) => {
                self.generator.config.length = len as usize;
                self.generator.generated =
                    vaultx_core::PasswordGenerator::generate(&self.generator.config);
                Task::none()
            }
            Message::GeneratorToggleUppercase => {
                self.generator.config.uppercase = !self.generator.config.uppercase;
                self.generator.generated =
                    vaultx_core::PasswordGenerator::generate(&self.generator.config);
                Task::none()
            }
            Message::GeneratorToggleLowercase => {
                self.generator.config.lowercase = !self.generator.config.lowercase;
                self.generator.generated =
                    vaultx_core::PasswordGenerator::generate(&self.generator.config);
                Task::none()
            }
            Message::GeneratorToggleDigits => {
                self.generator.config.digits = !self.generator.config.digits;
                self.generator.generated =
                    vaultx_core::PasswordGenerator::generate(&self.generator.config);
                Task::none()
            }
            Message::GeneratorToggleSymbols => {
                self.generator.config.symbols = !self.generator.config.symbols;
                self.generator.generated =
                    vaultx_core::PasswordGenerator::generate(&self.generator.config);
                Task::none()
            }
            Message::GeneratorToggleAmbiguous => {
                self.generator.config.exclude_ambiguous = !self.generator.config.exclude_ambiguous;
                self.generator.generated =
                    vaultx_core::PasswordGenerator::generate(&self.generator.config);
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Unlock(s) => s.view(),
            Screen::List(s) => {
                let entries = self
                    .vault
                    .as_ref()
                    .and_then(|v| v.entries().ok())
                    .unwrap_or(&[]);
                s.view(entries)
            }
            Screen::Detail(s) => {
                let entry = self
                    .vault
                    .as_ref()
                    .and_then(|v| v.entries().ok())
                    .and_then(|entries| entries.iter().find(|e| e.id == s.entry_id));
                s.view(entry, self.generator_open, &self.generator)
            }
            Screen::TotpView(s) => {
                let empty: &[_] = &[];
                let entries = self
                    .vault
                    .as_ref()
                    .and_then(|v| v.entries().ok())
                    .unwrap_or(empty);
                s.view(entries)
            }
            Screen::Generator(s) => s.view(),
            Screen::Settings(s) => s.view(),
            Screen::NewEntry(s) => s.view(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let totp_tick =
            iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::TotpTick);

        let needs_anim = if let Screen::NewEntry(s) = &self.screen {
            let pw_target = if s.password_section_expanded {
                1.0_f32
            } else {
                0.0
            };
            let totp_target = if s.totp_section_expanded {
                1.0_f32
            } else {
                0.0
            };
            (s.password_toggle_anim - pw_target).abs() > 0.005
                || (s.totp_toggle_anim - totp_target).abs() > 0.005
        } else {
            false
        };

        if needs_anim {
            Subscription::batch([
                totp_tick,
                iced::time::every(std::time::Duration::from_millis(16))
                    .map(|_| Message::ToggleAnimTick),
            ])
        } else {
            totp_tick
        }
    }

    fn navigate(&mut self, target: NavigationTarget) {
        self.screen = match target {
            NavigationTarget::List => Screen::List(ListScreen::default()),
            NavigationTarget::Detail(id) => Screen::Detail(DetailScreen::new(id)),
            NavigationTarget::TotpView => Screen::TotpView(TotpViewScreen::default()),
            NavigationTarget::Generator => {
                self.generator_open = true;
                self.screen.clone()
            }
            NavigationTarget::Settings => Screen::Settings(SettingsScreen {
                theme_pref: self.theme_pref,
                auto_lock_timeout: self.auto_lock_timeout,
                clipboard_clear_seconds: self.clipboard_clear_seconds,
                max_error_count: self.max_error_count,
            }),
            NavigationTarget::NewEntry => Screen::NewEntry(NewEntryScreen::default()),
        };
    }
}

/// 获取默认金库路径（跨平台）
pub fn default_vault_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("VaultX").join("vault.vaultx"))
}

/// 字体字节（编译期嵌入）
static INTER_FONT: &[u8] = include_bytes!("../../fonts/Inter-VariableFont_opsz,wght.ttf");
static ROBOTO_MONO_FONT: &[u8] = include_bytes!("../../fonts/RobotoMono-VariableFont_wght.ttf");
static WQY_MICROHEI_FONT: &[u8] = include_bytes!("../../fonts/wqy-microhei.ttc");
static MATERIAL_ICONS_FONT: &[u8] = include_bytes!("../../fonts/MaterialIcons-Regular.ttf");

/// Material Icons Round 字体句柄
pub const MATERIAL_ICONS: iced::Font = iced::Font::with_name("Material Icons");

/// 将 current 向 target 步进 speed，不超过 target
fn step_towards(current: f32, target: f32, speed: f32) -> f32 {
    if (current - target).abs() <= speed {
        target
    } else if current < target {
        current + speed
    } else {
        current - speed
    }
}

/// 应用程序入口
pub fn run() -> iced::Result {
    iced::application(VaultApp::title, VaultApp::update, VaultApp::view)
        .subscription(VaultApp::subscription)
        .theme(VaultApp::theme)
        .font(WQY_MICROHEI_FONT)
        .font(INTER_FONT)
        .font(ROBOTO_MONO_FONT)
        .font(MATERIAL_ICONS_FONT)
        .default_font(iced::Font::with_name("WenQuanYi Micro Hei"))
        .window(iced::window::Settings {
            size: iced::Size::new(
                crate::theme::layout::APP_WIDTH,
                crate::theme::layout::APP_HEIGHT,
            ),
            min_size: Some(iced::Size::new(760.0, 520.0)),
            resizable: true,
            ..Default::default()
        })
        .run_with(VaultApp::new)
}
