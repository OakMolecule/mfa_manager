'use strict';

import fs from 'fs';
import crypto from 'crypto';
import argon2 from 'argon2';

const VAULT_FORMAT_VERSION = 1;

const ARGON2_PARAMS = {
  memoryCost: 65536, // 64 MiB
  timeCost: 3,
  parallelism: 4,
  hashLength: 32,
  type: argon2.argon2id,
};

export interface VaultEntry {
  id: string;
  icon?: string;
  issuer?: string;
  title?: string;
  label?: string;
  username?: string;
  password?: string;
  category?: string;
  type?: 'totp' | 'password';
  totp?: {
    secret: string;
    algorithm?: string;
    digits?: number;
    period?: number;
  };
  secret?: string;
}

interface VaultParams {
  salt: Buffer;
  memoryCost: number;
  timeCost: number;
  parallelism: number;
}

interface VaultData {
  entries: VaultEntry[];
}

interface VaultFile {
  version: number;
  argon2_params: {
    m_cost: number;
    t_cost: number;
    p_cost: number;
    salt: string;
  };
  nonce: string;
  ciphertext: string;
}

function atomicWrite(filePath: string, data: string): void {
  const tmp = filePath + '.tmp';
  fs.writeFileSync(tmp, data, 'utf-8');
  fs.renameSync(tmp, filePath);
}

function encrypt(key: Buffer, plaintext: Buffer): { nonce: Buffer; ciphertext: Buffer } {
  const nonce = crypto.randomBytes(12);
  const cipher = crypto.createCipheriv('aes-256-gcm', key, nonce);
  const encrypted = Buffer.concat([cipher.update(plaintext), cipher.final()]);
  const tag = cipher.getAuthTag();
  const ciphertext = Buffer.concat([encrypted, tag]);
  return { nonce, ciphertext };
}

function decrypt(key: Buffer, nonce: Buffer, ciphertext: Buffer): Buffer {
  const tag = ciphertext.slice(ciphertext.length - 16);
  const encrypted = ciphertext.slice(0, ciphertext.length - 16);
  const decipher = crypto.createDecipheriv('aes-256-gcm', key, nonce);
  decipher.setAuthTag(tag);
  return Buffer.concat([decipher.update(encrypted), decipher.final()]);
}

export class VaultManager {
  private filePath: string | null = null;
  private params: VaultParams | null = null;
  private key: Buffer | null = null;
  private data: VaultData | null = null;
  private failCount = 0;
  private lockoutUntil = 0;

  isUnlocked(): boolean {
    return this.key !== null;
  }

  async open(filePath: string): Promise<void> {
    if (!fs.existsSync(filePath)) {
      throw new Error(`金库文件不存在: ${filePath}`);
    }
    const raw = fs.readFileSync(filePath, 'utf-8');
    const vaultFile: VaultFile = JSON.parse(raw);
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

  async create(filePath: string, password: string): Promise<void> {
    const salt = crypto.randomBytes(16);
    const saltB64 = salt.toString('base64');

    const key = (await argon2.hash(password, {
      ...ARGON2_PARAMS,
      salt,
      raw: true,
    })) as unknown as Buffer;

    const emptyData: VaultData = { entries: [] };
    const plaintext = Buffer.from(JSON.stringify(emptyData), 'utf-8');
    const { nonce, ciphertext } = encrypt(key, plaintext);

    const vaultFile: VaultFile = {
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
    this.params = {
      salt,
      memoryCost: ARGON2_PARAMS.memoryCost,
      timeCost: ARGON2_PARAMS.timeCost,
      parallelism: ARGON2_PARAMS.parallelism,
    };
    this.key = key;
    this.data = emptyData;
    this.failCount = 0;
    this.lockoutUntil = 0;
  }

  async unlock(password: string): Promise<void> {
    if (this.lockoutUntil > Date.now()) {
      const remaining = Math.ceil((this.lockoutUntil - Date.now()) / 1000);
      throw new Error(`密码错误次数过多，请等待 ${remaining} 秒后再试`);
    }

    const raw = fs.readFileSync(this.filePath!, 'utf-8');
    const vaultFile: VaultFile = JSON.parse(raw);

    const salt = Buffer.from(vaultFile.argon2_params.salt, 'base64');
    let key: Buffer;
    try {
      key = (await argon2.hash(password, {
        memoryCost: vaultFile.argon2_params.m_cost,
        timeCost: vaultFile.argon2_params.t_cost,
        parallelism: vaultFile.argon2_params.p_cost,
        hashLength: 32,
        type: argon2.argon2id,
        salt,
        raw: true,
      })) as unknown as Buffer;
    } catch {
      throw new Error('密钥派生失败');
    }

    try {
      const nonce = Buffer.from(vaultFile.nonce, 'base64');
      const ciphertext = Buffer.from(vaultFile.ciphertext, 'base64');
      const plaintext = decrypt(key, nonce, ciphertext);
      this.data = JSON.parse(plaintext.toString('utf-8'));
    } catch {
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

  lock(): void {
    if (this.key) {
      this.key.fill(0);
      this.key = null;
    }
    this.data = null;
  }

  getEntries(): VaultEntry[] {
    if (!this.data) throw new Error('金库已锁定');
    return this.data.entries;
  }

  addEntry(entry: VaultEntry): void {
    if (!this.data) throw new Error('金库已锁定');
    this.data.entries.push(entry);
  }

  updateEntry(updated: VaultEntry): void {
    if (!this.data) throw new Error('金库已锁定');
    const idx = this.data.entries.findIndex(e => e.id === updated.id);
    if (idx === -1) throw new Error(`条目不存在: ${updated.id}`);
    this.data.entries[idx] = updated;
  }

  deleteEntry(id: string): void {
    if (!this.data) throw new Error('金库已锁定');
    this.data.entries = this.data.entries.filter(e => e.id !== id);
  }

  async save(): Promise<void> {
    if (!this.key || !this.data || !this.params) throw new Error('金库已锁定');
    const plaintext = Buffer.from(JSON.stringify(this.data), 'utf-8');
    const { nonce, ciphertext } = encrypt(this.key, plaintext);

    const saltB64 = this.params.salt.toString('base64');
    const vaultFile: VaultFile = {
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
    atomicWrite(this.filePath!, JSON.stringify(vaultFile, null, 2));
  }

  async changePassword(oldPassword: string, newPassword: string): Promise<void> {
    await this.unlock(oldPassword);
    const newSalt = crypto.randomBytes(16);
    const newKey = (await argon2.hash(newPassword, {
      ...ARGON2_PARAMS,
      salt: newSalt,
      raw: true,
    })) as unknown as Buffer;
    if (this.key) this.key.fill(0);
    this.key = newKey;
    this.params!.salt = newSalt;
    await this.save();
  }
}

export default VaultManager;
