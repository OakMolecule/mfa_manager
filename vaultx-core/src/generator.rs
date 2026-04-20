use rand::{distributions::Uniform, Rng};

/// 密码生成器配置
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub digits: bool,
    pub symbols: bool,
    pub exclude_ambiguous: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            length: 16,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: true,
            exclude_ambiguous: false,
        }
    }
}

pub struct PasswordGenerator;

impl PasswordGenerator {
    /// 根据配置生成随机密码
    pub fn generate(config: &GeneratorConfig) -> String {
        let mut charset = String::new();

        if config.uppercase {
            if config.exclude_ambiguous {
                charset.push_str("ABCDEFGHJKLMNPQRSTUVWXYZ");
            } else {
                charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
            }
        }
        if config.lowercase {
            if config.exclude_ambiguous {
                charset.push_str("abcdefghjkmnpqrstuvwxyz");
            } else {
                charset.push_str("abcdefghijklmnopqrstuvwxyz");
            }
        }
        if config.digits {
            if config.exclude_ambiguous {
                charset.push_str("23456789");
            } else {
                charset.push_str("0123456789");
            }
        }
        if config.symbols {
            charset.push_str("!@#$%^&*-_=+");
        }

        // 如果没有选择任何字符集，回退到小写字母
        if charset.is_empty() {
            charset.push_str("abcdefghijklmnopqrstuvwxyz");
        }

        let chars: Vec<char> = charset.chars().collect();
        let dist = Uniform::from(0..chars.len());
        let mut rng = rand::thread_rng();

        (0..config.length)
            .map(|_| chars[rng.sample(dist)])
            .collect()
    }

    /// 评估密码强度，返回 0-100 分
    pub fn evaluate_strength(password: &str) -> PasswordStrength {
        let len = password.len();
        if len == 0 {
            return PasswordStrength::Weak;
        }

        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

        let variety = [has_upper, has_lower, has_digit, has_symbol]
            .iter()
            .filter(|&&x| x)
            .count();

        if len >= 16 && variety >= 3 {
            PasswordStrength::Strong
        } else if len >= 12 && variety >= 2 {
            PasswordStrength::Medium
        } else {
            PasswordStrength::Weak
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    Weak,
    Medium,
    Strong,
}

impl std::fmt::Display for PasswordStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Weak => write!(f, "弱"),
            Self::Medium => write!(f, "中"),
            Self::Strong => write!(f, "强"),
        }
    }
}
