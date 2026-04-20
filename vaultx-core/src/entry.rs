use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::Zeroize;

/// 密码数据（可选，与 TOTP 并存）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordData {
    pub url: String,
    pub username: String,
    pub password: String,
    pub notes: String,
}

impl Drop for PasswordData {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}

/// TOTP 数据（可选，与密码并存）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpData {
    pub secret: String,
    pub issuer: String,
    pub account: String,
    pub algorithm: TotpAlgorithm,
    pub digits: u32,
    pub period: u64,
}

impl Drop for TotpData {
    fn drop(&mut self) {
        self.secret.zeroize();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum TotpAlgorithm {
    #[default]
    SHA1,
    SHA256,
    SHA512,
}

/// 条目分类
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Category {
    #[default]
    Personal,
    Work,
    Finance,
    Shopping,
    Custom(String),
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Personal => write!(f, "个人"),
            Self::Work => write!(f, "工作"),
            Self::Finance => write!(f, "金融"),
            Self::Shopping => write!(f, "购物"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

/// 密码库中的一个条目
/// `password` 和 `totp` 至少一个不为 None
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: Uuid,
    pub title: String,
    pub category: Category,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<PasswordData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub totp: Option<TotpData>,
}

impl Entry {
    pub fn new_password(title: impl Into<String>, data: PasswordData) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            category: Category::default(),
            created_at: now,
            updated_at: now,
            password: Some(data),
            totp: None,
        }
    }

    pub fn new_totp(title: impl Into<String>, data: TotpData) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            category: Category::default(),
            created_at: now,
            updated_at: now,
            password: None,
            totp: Some(data),
        }
    }

    pub fn has_password(&self) -> bool {
        self.password.is_some()
    }

    pub fn has_totp(&self) -> bool {
        self.totp.is_some()
    }

    /// 返回用于头像显示的首字母（大写）
    pub fn avatar_char(&self) -> char {
        self.title
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .next()
            .unwrap_or('?')
    }

    /// 更新时间戳
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now();
    }
}

/// 解密后的明文金库内容
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VaultData {
    pub entries: Vec<Entry>,
}
