use crate::{
    crypto::{Argon2Params, VaultCrypto},
    entry::VaultData,
    VAULT_FORMAT_VERSION,
};
use secrecy::{ExposeSecret, SecretString, SecretVec};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use zeroize::Zeroize;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Crypto error: {0}")]
    Crypto(#[from] crate::crypto::CryptoError),
    #[error("Unsupported vault format version: {0}")]
    UnsupportedVersion(u32),
    #[error("Wrong password")]
    WrongPassword,
    #[error("Vault is locked")]
    Locked,
}

/// .vaultx 文件的磁盘格式
#[derive(Debug, Serialize, Deserialize)]
struct VaultFile {
    version: u32,
    argon2_params: Argon2Params,
    nonce: String,
    ciphertext: String,
}

/// 运行时金库状态
pub struct Vault {
    /// 金库文件路径
    pub path: PathBuf,
    /// Argon2 参数（含盐值，从文件加载或首次创建时生成）
    pub params: Argon2Params,
    /// 解锁后的派生密钥（锁定时清零并设为 None）
    key: Option<SecretVec<u8>>,
    /// 解锁后的明文数据
    data: Option<VaultData>,
}

impl Vault {
    /// 从文件路径创建 Vault 句柄（尚未解锁）
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, VaultError> {
        let path = path.into();
        if !path.exists() {
            return Err(VaultError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Vault file not found: {}", path.display()),
            )));
        }

        let raw = std::fs::read_to_string(&path)?;
        let file: VaultFile = serde_json::from_str(&raw)?;

        if file.version != VAULT_FORMAT_VERSION {
            return Err(VaultError::UnsupportedVersion(file.version));
        }

        Ok(Self {
            path,
            params: file.argon2_params,
            key: None,
            data: None,
        })
    }

    /// 创建新金库（文件不存在时使用）
    pub fn create(path: impl Into<PathBuf>, password: &SecretString) -> Result<Self, VaultError> {
        let path = path.into();
        let mut params = Argon2Params::default();
        let key = VaultCrypto::derive_key(password, &mut params)?;

        let empty = VaultData::default();
        let plaintext = serde_json::to_vec(&empty)?;
        let (nonce, ciphertext) = VaultCrypto::encrypt(&key, &plaintext)?;

        let file = VaultFile {
            version: VAULT_FORMAT_VERSION,
            argon2_params: params.clone(),
            nonce,
            ciphertext,
        };
        let json = serde_json::to_string_pretty(&file)?;
        atomic_write(&path, json.as_bytes())?;

        Ok(Self {
            path,
            params,
            key: Some(key),
            data: Some(empty),
        })
    }

    /// 解锁金库（验证密码并解密数据）
    pub fn unlock(&mut self, password: &SecretString) -> Result<(), VaultError> {
        let raw = std::fs::read_to_string(&self.path)?;
        let file: VaultFile = serde_json::from_str(&raw)?;

        let mut params = file.argon2_params.clone();
        let key = VaultCrypto::derive_key(password, &mut params)?;

        let plaintext = VaultCrypto::decrypt(&key, &file.nonce, &file.ciphertext)
            .map_err(|_| VaultError::WrongPassword)?;

        let data: VaultData = serde_json::from_slice(&plaintext)?;

        self.params = params;
        self.key = Some(key);
        self.data = Some(data);
        Ok(())
    }

    /// 锁定金库，清零内存中的敏感数据
    pub fn lock(&mut self) {
        if let Some(key) = self.key.take() {
            drop(key); // SecretVec 在 Drop 时通过 zeroize 归零
        }
        self.data = None;
    }

    pub fn is_unlocked(&self) -> bool {
        self.key.is_some()
    }

    /// 获取条目列表（需已解锁）
    pub fn entries(&self) -> Result<&[crate::entry::Entry], VaultError> {
        self.data
            .as_ref()
            .map(|d| d.entries.as_slice())
            .ok_or(VaultError::Locked)
    }

    /// 获取可变条目列表（需已解锁）
    pub fn entries_mut(&mut self) -> Result<&mut Vec<crate::entry::Entry>, VaultError> {
        self.data
            .as_mut()
            .map(|d| &mut d.entries)
            .ok_or(VaultError::Locked)
    }

    /// 将当前状态加密并原子写入磁盘
    pub fn save(&self) -> Result<(), VaultError> {
        let key = self.key.as_ref().ok_or(VaultError::Locked)?;
        let data = self.data.as_ref().ok_or(VaultError::Locked)?;

        let mut plaintext = serde_json::to_vec(data)?;
        let (nonce, ciphertext) = VaultCrypto::encrypt(key, &plaintext)?;
        plaintext.zeroize();

        let file = VaultFile {
            version: VAULT_FORMAT_VERSION,
            argon2_params: self.params.clone(),
            nonce,
            ciphertext,
        };
        let json = serde_json::to_string_pretty(&file)?;
        atomic_write(&self.path, json.as_bytes())?;
        Ok(())
    }

    /// 修改主密码（重新派生 key 并保存）
    pub fn change_password(&mut self, new_password: &SecretString) -> Result<(), VaultError> {
        if !self.is_unlocked() {
            return Err(VaultError::Locked);
        }
        // 重置盐值，触发新盐生成
        self.params.salt.clear();
        let new_key = VaultCrypto::derive_key(new_password, &mut self.params)?;
        self.key = Some(new_key);
        self.save()
    }
}

/// 原子写入：先写临时文件，成功后替换原文件
fn atomic_write(path: &Path, data: &[u8]) -> Result<(), std::io::Error> {
    let tmp_path = path.with_extension("vaultx.tmp");
    std::fs::write(&tmp_path, data)?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}
