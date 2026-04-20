use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{password_hash::SaltString, Argon2, Params, PasswordHasher, Version};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use secrecy::{ExposeSecret, SecretString, SecretVec};
use thiserror::Error;
use zeroize::Zeroize;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Argon2 key derivation failed: {0}")]
    KeyDerivation(String),
    #[error("Encryption failed")]
    Encryption,
    #[error("Decryption failed — wrong password or corrupted data")]
    Decryption,
    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("Invalid nonce length")]
    InvalidNonce,
}

/// Argon2id 参数（存储在文件头中以便验证）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Argon2Params {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
    pub salt: String, // base64 编码的 16 字节盐值
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            m_cost: 65536, // 64 MiB
            t_cost: 3,
            p_cost: 4,
            salt: String::new(), // 首次创建时随机生成
        }
    }
}

pub struct VaultCrypto;

impl VaultCrypto {
    /// 生成随机盐值并更新 params，返回派生的 256-bit AES key
    pub fn derive_key(
        password: &SecretString,
        params: &mut Argon2Params,
    ) -> Result<SecretVec<u8>, CryptoError> {
        // 首次调用时生成盐值
        if params.salt.is_empty() {
            let salt = SaltString::generate(&mut rand::thread_rng());
            params.salt = salt.to_string();
        }

        let argon2_params = Params::new(params.m_cost, params.t_cost, params.p_cost, Some(32))
            .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, argon2_params);

        let salt = SaltString::from_b64(&params.salt)
            .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

        let hash = argon2
            .hash_password(password.expose_secret().as_bytes(), &salt)
            .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

        // 提取 PHC 哈希中的原始 key bytes（32 字节）
        let raw_hash = hash
            .hash
            .ok_or_else(|| CryptoError::KeyDerivation("no hash output".into()))?;

        let key_bytes = raw_hash.as_bytes().to_vec();
        Ok(SecretVec::new(key_bytes))
    }

    /// 使用 AES-256-GCM 加密明文，返回 (nonce_base64, ciphertext_base64)
    pub fn encrypt(key: &SecretVec<u8>, plaintext: &[u8]) -> Result<(String, String), CryptoError> {
        let aes_key = Key::<Aes256Gcm>::from_slice(key.expose_secret());
        let cipher = Aes256Gcm::new(aes_key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| CryptoError::Encryption)?;

        Ok((BASE64.encode(nonce.as_slice()), BASE64.encode(&ciphertext)))
    }

    /// 使用 AES-256-GCM 解密，返回明文字节
    pub fn decrypt(
        key: &SecretVec<u8>,
        nonce_b64: &str,
        ciphertext_b64: &str,
    ) -> Result<Vec<u8>, CryptoError> {
        let nonce_bytes = BASE64.decode(nonce_b64)?;
        if nonce_bytes.len() != 12 {
            return Err(CryptoError::InvalidNonce);
        }
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = BASE64.decode(ciphertext_b64)?;

        let aes_key = Key::<Aes256Gcm>::from_slice(key.expose_secret());
        let cipher = Aes256Gcm::new(aes_key);

        cipher
            .decrypt(nonce, ciphertext.as_slice())
            .map_err(|_| CryptoError::Decryption)
    }
}

/// 确保临时 key bytes 在离开作用域后被归零
pub struct ZeroizingKey(pub Vec<u8>);

impl Drop for ZeroizingKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
