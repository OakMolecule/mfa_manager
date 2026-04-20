use crate::entry::{TotpAlgorithm, TotpData};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use totp_rs::{Algorithm, Secret, TOTP};

#[derive(Debug, Error)]
pub enum TotpError {
    #[error("Invalid TOTP secret: {0}")]
    InvalidSecret(String),
    #[error("Failed to generate TOTP code: {0}")]
    GenerationFailed(String),
    #[error("System time error")]
    SystemTime,
}

/// TOTP 计算结果
#[derive(Debug, Clone)]
pub struct TotpResult {
    /// 6 位验证码字符串（前置补零）
    pub code: String,
    /// 当前周期已过去的秒数
    pub elapsed: u64,
    /// 总周期秒数（默认 30）
    pub period: u64,
    /// 剩余有效秒数
    pub remaining: u64,
    /// 是否即将过期（≤5s）
    pub expiring: bool,
}

pub struct TotpEngine;

impl TotpEngine {
    /// 根据 TotpData 计算当前验证码
    pub fn compute(data: &TotpData) -> Result<TotpResult, TotpError> {
        let algorithm = match data.algorithm {
            TotpAlgorithm::SHA1 => Algorithm::SHA1,
            TotpAlgorithm::SHA256 => Algorithm::SHA256,
            TotpAlgorithm::SHA512 => Algorithm::SHA512,
        };

        let secret = Secret::Encoded(data.secret.clone())
            .to_bytes()
            .map_err(|e| TotpError::InvalidSecret(e.to_string()))?;

        let totp = TOTP::new(
            algorithm,
            data.digits as usize,
            1, // skew
            data.period,
            secret,
        )
        .map_err(|e| TotpError::InvalidSecret(e.to_string()))?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TotpError::SystemTime)?
            .as_secs();

        let code = totp.generate(now).to_string();

        // 格式化为前置补零的 N 位字符串（例如 "006183"）
        let formatted = format!("{:0>width$}", code, width = data.digits as usize);

        let elapsed = now % data.period;
        let remaining = data.period - elapsed;

        Ok(TotpResult {
            code: formatted,
            elapsed,
            period: data.period,
            remaining,
            expiring: remaining <= 5,
        })
    }

    /// 将 6 位验证码格式化为 "XXX XXX" 显示格式
    pub fn format_display(code: &str) -> String {
        if code.len() == 6 {
            format!("{} {}", &code[..3], &code[3..])
        } else {
            code.to_string()
        }
    }
}
