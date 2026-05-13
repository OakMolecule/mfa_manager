'use strict';

const crypto = require('crypto');

/**
 * 密码生成器（对应 Rust vaultx-core::generator）
 */
function generatePassword(config) {
  const {
    length = 16,
    uppercase = true,
    lowercase = true,
    digits = true,
    symbols = true,
    excludeAmbiguous = false,
  } = config;

  let charset = '';
  if (uppercase) charset += excludeAmbiguous ? 'ABCDEFGHJKLMNPQRSTUVWXYZ' : 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
  if (lowercase) charset += excludeAmbiguous ? 'abcdefghjkmnpqrstuvwxyz' : 'abcdefghijklmnopqrstuvwxyz';
  if (digits)    charset += excludeAmbiguous ? '23456789' : '0123456789';
  if (symbols)   charset += '!@#$%^&*-_=+';
  if (!charset)  charset = 'abcdefghijklmnopqrstuvwxyz';

  const chars = Array.from(charset);
  let result = '';
  for (let i = 0; i < length; i++) {
    const idx = crypto.randomInt(0, chars.length);
    result += chars[idx];
  }
  return result;
}

/**
 * 密码强度评估（对应 Rust vaultx-core::generator::evaluate_strength）
 * 返回 'weak' | 'medium' | 'strong'
 */
function evaluatePasswordStrength(password) {
  if (!password || password.length === 0) return 'weak';
  const len = password.length;
  const hasUpper = /[A-Z]/.test(password);
  const hasLower = /[a-z]/.test(password);
  const hasDigit = /[0-9]/.test(password);
  const hasSymbol = /[^A-Za-z0-9]/.test(password);
  const variety = [hasUpper, hasLower, hasDigit, hasSymbol].filter(Boolean).length;

  if (len >= 16 && variety >= 3) return 'strong';
  if (len >= 12 && variety >= 2) return 'medium';
  return 'weak';
}

module.exports = { generatePassword, evaluatePasswordStrength };
