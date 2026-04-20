pub mod crypto;
pub mod entry;
pub mod generator;
pub mod totp;
pub mod vault;

pub use crypto::VaultCrypto;
pub use entry::{Entry, PasswordData, TotpData};
pub use generator::{GeneratorConfig, PasswordGenerator};
pub use totp::TotpEngine;
pub use vault::Vault;

/// 当前 .vaultx 文件格式版本
pub const VAULT_FORMAT_VERSION: u32 = 1;
