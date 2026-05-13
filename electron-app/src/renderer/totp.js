'use strict';

/**
 * 纯前端 TOTP 计算（RFC 6238 / TOTP）
 * 支持 SHA1 / SHA256 / SHA512（Web Crypto API）
 */

const BASE32_CHARS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';

function base32Decode(input) {
  if (typeof input !== 'string') throw new TypeError('base32 input must be a string');
  const str = input.toUpperCase().replace(/=+$/, '').replace(/\s/g, '');
  const bytes = [];
  let buffer = 0;
  let bitsLeft = 0;

  for (const char of str) {
    const val = BASE32_CHARS.indexOf(char);
    if (val < 0) continue;
    buffer = (buffer << 5) | val;
    bitsLeft += 5;
    if (bitsLeft >= 8) {
      bitsLeft -= 8;
      bytes.push((buffer >> bitsLeft) & 0xFF);
    }
  }
  return new Uint8Array(bytes);
}

function uint64ToBytes(n) {
  const buf = new Uint8Array(8);
  let val = BigInt(n);
  for (let i = 7; i >= 0; i--) {
    buf[i] = Number(val & 0xFFn);
    val >>= 8n;
  }
  return buf;
}

async function computeHotp(key, counter, digits, algorithm) {
  const algoMap = { SHA1: 'SHA-1', SHA256: 'SHA-256', SHA512: 'SHA-512' };
  const algoName = algoMap[algorithm] || 'SHA-1';

  const cryptoKey = await crypto.subtle.importKey(
    'raw', key,
    { name: 'HMAC', hash: algoName },
    false, ['sign']
  );

  const counterBytes = uint64ToBytes(counter);
  const sig = await crypto.subtle.sign('HMAC', cryptoKey, counterBytes);
  const hmac = new Uint8Array(sig);

  const offset = hmac[hmac.length - 1] & 0x0F;
  const code = (
    ((hmac[offset]     & 0x7F) << 24) |
    ((hmac[offset + 1] & 0xFF) << 16) |
    ((hmac[offset + 2] & 0xFF) << 8)  |
     (hmac[offset + 3] & 0xFF)
  ) % Math.pow(10, digits);

  return String(code).padStart(digits, '0');
}

/**
 * 计算 TOTP 验证码
 * @param {object} totpData - { secret, algorithm, digits, period }
 * @returns {Promise<{code, elapsed, period, remaining, expiring}>}
 */
async function computeTotp(totpData) {
  const { secret, algorithm = 'SHA1', digits = 6, period = 30 } = totpData;
  const keyBytes = base32Decode(secret);
  const now = Math.floor(Date.now() / 1000);
  const counter = Math.floor(now / period);
  const code = await computeHotp(keyBytes, counter, digits, algorithm);
  const elapsed = now % period;
  const remaining = period - elapsed;
  return {
    code,
    elapsed,
    period,
    remaining,
    expiring: remaining <= 5,
  };
}

// 导出到全局（在渲染进程中以脚本形式加载）
if (typeof window !== 'undefined') {
  window.TotpUtil = { computeTotp };
}
