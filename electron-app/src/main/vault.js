'use strict';

const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const argon2 = require('argon2');

const VAULT_FORMAT_VERSION = 1;

// Argon2id 参数（与 Rust 实现相同）
const ARGON2_PARAMS = {
  memoryCost: 65536, // 64 MiB
  timeCost: 3,
  parallelism: 4,
  hashLength: 32,
  type: argon2.argon2id,
};

/**
 * 原子写入：先写临时文件再重命名，防止写入中断导致数据损坏
 */
function atomicWrite(filePath, data) {
  const tmp = filePath + '.tmp';
  fs.writeFileSync(tmp, data, 'utf-8');
  fs.renameSync(tmp, filePath);
}

class VaultManager {
  constructor() {
    this.filePath = null;
    this.params = null;   // { salt: Buffer, memoryCost, timeCost, parallelism }
    this.key = null;      // Buffer (32 bytes)
    this.data = null;     // { entries: [] }
    this.failCount = 0;   // 连续错误次数
    this.lockoutUntil = 0; // 锁定截止时间戳（ms）
  }

  isUnlocked() {
    return this.key !== null;
  }

  /**
   * 加载金库文件元数据（不解密）
   */
  async open(filePath) {
    if (!fs.existsSync(filePath)) {
      throw new Error(`金库文件不存在: ${filePath}`);
    }
    const raw = fs.readFileSync(filePath, 'utf-8');
    const vaultFile = JSON.parse(raw);
    if (vaultFile.version !== VAULT_FORMAT_VERSION) {
      throw new Error(`不支持的金库格式版本: ${vaultFile.version}`);
    }
    this.filePath = filePath;
    this.params = {
      salt: Buffer.from(vaultFile.argon2_params.salt, 'base64'),
      memoryCost: vaultFile.argon2_params.m_cost,
      timeCost: vaultFile.argon2_params.t_cost,
      parallelism: vaultFile.argon2_params.p_cost,
    };
    this.key = null;
    this.data = null;
  }

  /**
   * 创建新金库
   */
  async create(filePath, password) {
    const salt = crypto.randomBytes(16);
    const saltB64 = salt.toString('base64');

    const key = await argon2.hash(password, {
      ...ARGON2_PARAMS,
      salt,
      raw: true,
    });

    const emptyData = { entries: [] };
    const plaintext = Buffer.from(JSON.stringify(emptyData), 'utf-8');
    const { nonce, ciphertext } = encrypt(key, plaintext);

    const vaultFile = {
      version: VAULT_FORMAT_VERSION,
      argon2_params: {
        m_cost: ARGON2_PARAMS.memoryCost,
        t_cost: ARGON2_PARAMS.timeCost,
        p_cost: ARGON2_PARAMS.parallelism,
        salt: saltB64,
      },
      nonce: nonce.toString('base64'),
      ciphertext: ciphertext.toString('base64'),
    };

    atomicWrite(filePath, JSON.stringify(vaultFile, null, 2));

    this.filePath = filePath;
    this.params = { salt, memoryCost: ARGON2_PARAMS.memoryCost, timeCost: ARGON2_PARAMS.timeCost, parallelism: ARGON2_PARAMS.parallelism };
    this.key = key;
    this.data = emptyData;
    this.failCount = 0;
    this.lockoutUntil = 0;
  }

  /**
   * 解锁金库（验证密码并解密）
   * 实现指数退避：连续错误 5 次后锁定 2^(n-5) 秒，最大 64 秒
   */
  async unlock(password) {
    // 检查是否在锁定期
    if (this.lockoutUntil > Date.now()) {
      const remaining = Math.ceil((this.lockoutUntil - Date.now()) / 1000);
      throw new Error(`密码错误次数过多，请等待 ${remaining} 秒后再试`);
    }

    const raw = fs.readFileSync(this.filePath, 'utf-8');
    const vaultFile = JSON.parse(raw);

    const salt = Buffer.from(vaultFile.argon2_params.salt, 'base64');
    let key;
    try {
      key = await argon2.hash(password, {
        memoryCost: vaultFile.argon2_params.m_cost,
        timeCost: vaultFile.argon2_params.t_cost,
        parallelism: vaultFile.argon2_params.p_cost,
        hashLength: 32,
        type: argon2.argon2id,
        salt,
        raw: true,
      });
    } catch (e) {
      throw new Error('密钥派生失败');
    }

    try {
      const nonce = Buffer.from(vaultFile.nonce, 'base64');
      const ciphertext = Buffer.from(vaultFile.ciphertext, 'base64');
      const plaintext = decrypt(key, nonce, ciphertext);
      this.data = JSON.parse(plaintext.toString('utf-8'));
    } catch (e) {
      // 解密失败 = 密码错误
      this.failCount++;
      if (this.failCount >= 5) {
        const delay = Math.min(Math.pow(2, this.failCount - 5), 64);
        this.lockoutUntil = Date.now() + delay * 1000;
        throw new Error(`密码错误（已连续失败 ${this.failCount} 次），锁定 ${delay} 秒`);
      }
      throw new Error(`密码错误（已连续失败 ${this.failCount} 次）`);
    }

    this.key = key;
    this.params = {
      salt,
      memoryCost: vaultFile.argon2_params.m_cost,
      timeCost: vaultFile.argon2_params.t_cost,
      parallelism: vaultFile.argon2_params.p_cost,
    };
    this.failCount = 0;
    this.lockoutUntil = 0;
  }

  /**
   * 锁定金库，清零内存中的密钥
   */
  lock() {
    if (this.key) {
      this.key.fill(0); // zeroize
      this.key = null;
    }
    this.data = null;
  }

  getEntries() {
    if (!this.data) throw new Error('金库已锁定');
    return this.data.entries;
  }

  addEntry(entry) {
    if (!this.data) throw new Error('金库已锁定');
    this.data.entries.push(entry);
  }

  updateEntry(updated) {
    if (!this.data) throw new Error('金库已锁定');
    const idx = this.data.entries.findIndex(e => e.id === updated.id);
    if (idx === -1) throw new Error(`条目不存在: ${updated.id}`);
    this.data.entries[idx] = updated;
  }

  deleteEntry(id) {
    if (!this.data) throw new Error('金库已锁定');
    this.data.entries = this.data.entries.filter(e => e.id !== id);
  }

  async save() {
    if (!this.key || !this.data) throw new Error('金库已锁定');
    const plaintext = Buffer.from(JSON.stringify(this.data), 'utf-8');
    const { nonce, ciphertext } = encrypt(this.key, plaintext);

    const saltB64 = this.params.salt.toString('base64');
    const vaultFile = {
      version: VAULT_FORMAT_VERSION,
      argon2_params: {
        m_cost: this.params.memoryCost,
        t_cost: this.params.timeCost,
        p_cost: this.params.parallelism,
        salt: saltB64,
      },
      nonce: nonce.toString('base64'),
      ciphertext: ciphertext.toString('base64'),
    };
    atomicWrite(this.filePath, JSON.stringify(vaultFile, null, 2));
  }

  async changePassword(oldPassword, newPassword) {
    // 先用旧密码验证
    await this.unlock(oldPassword);
    // 生成新盐值
    const newSalt = crypto.randomBytes(16);
    const newKey = await argon2.hash(newPassword, {
      ...ARGON2_PARAMS,
      salt: newSalt,
      raw: true,
    });
    if (this.key) this.key.fill(0);
    this.key = newKey;
    this.params.salt = newSalt;
    await this.save();
  }
}

// ── AES-256-GCM 加密/解密 ────────────────────────────────────────────────

function encrypt(key, plaintext) {
  const nonce = crypto.randomBytes(12);
  const cipher = crypto.createCipheriv('aes-256-gcm', key, nonce);
  const encrypted = Buffer.concat([cipher.update(plaintext), cipher.final()]);
  const tag = cipher.getAuthTag();
  // 将 tag 附加到密文尾部（与 aes-gcm crate 行为一致）
  const ciphertext = Buffer.concat([encrypted, tag]);
  return { nonce, ciphertext };
}

function decrypt(key, nonce, ciphertext) {
  // 最后 16 字节为 GCM tag
  const tag = ciphertext.slice(ciphertext.length - 16);
  const encrypted = ciphertext.slice(0, ciphertext.length - 16);
  const decipher = crypto.createDecipheriv('aes-256-gcm', key, nonce);
  decipher.setAuthTag(tag);
  return Buffer.concat([decipher.update(encrypted), decipher.final()]);
}

module.exports = VaultManager;
