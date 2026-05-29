'use strict';

interface ImportedEntry {
  id: string;
  icon?: string;
  issuer?: string;
  title?: string;
  label?: string;
  username?: string;
  password?: string;
  category?: string;
  type?: 'totp' | 'password';
  totp?: { secret: string; algorithm?: string; digits?: number; period?: number };
  secret?: string;
  createdAt?: number;
  updatedAt?: number;
}

interface CategoryOption {
  key: string;
  label: string;
}

function getText(parent: Element, tag: string): string {
  const el = parent.getElementsByTagName(tag)[0];
  return el?.textContent?.trim() || '';
}

function parseISODate(s: string): number | undefined {
  if (!s) return undefined;
  const t = Date.parse(s);
  return isNaN(t) ? undefined : t;
}

function getFieldStrings(entryEl: Element): Record<string, string> {
  const fields: Record<string, string> = {};
  const strings = entryEl.getElementsByTagName('String');
  for (let i = 0; i < strings.length; i++) {
    const key = getText(strings[i], 'Key');
    const val = getText(strings[i], 'Value');
    if (key) fields[key] = val;
  }
  return fields;
}

function parseTimes(entryEl: Element): { createdAt?: number; updatedAt?: number; expired: boolean } {
  const times = entryEl.getElementsByTagName('Times')[0];
  if (!times) return { expired: false };
  const expires = getText(times, 'Expires').toLowerCase() === 'true';
  const expiryTime = parseISODate(getText(times, 'ExpiryTime'));
  return {
    createdAt: parseISODate(getText(times, 'CreationTime')),
    updatedAt: parseISODate(getText(times, 'LastModificationTime')),
    expired: expires && !!expiryTime && expiryTime < Date.now(),
  };
}

function detectTotp(fields: Record<string, string>): { type: 'totp'; totp: ImportedEntry['totp']; secret?: string } | null {
  // KeePass built-in: TimeOtp-Secret-Base32
  if (fields['TimeOtp-Secret-Base32']) {
    return {
      type: 'totp',
      totp: {
        secret: fields['TimeOtp-Secret-Base32'],
        period: parseInt(fields['TimeOtp-Period']) || 30,
        digits: parseInt(fields['TimeOtp-Length']) || 6,
        algorithm: fields['TimeOtp-Algorithm'] || 'SHA1',
      },
    };
  }
  // KeePassXC: TOTP Seed + TOTP Settings
  if (fields['TOTP Seed']) {
    const settings = (fields['TOTP Settings'] || '').split(';');
    return {
      type: 'totp',
      totp: {
        secret: fields['TOTP Seed'],
        period: parseInt(settings[0]) || 30,
        digits: parseInt(settings[1]) || 6,
      },
    };
  }
  // KeeOtp plugin: otp field
  if (fields['otp']) {
    const otpVal = fields['otp'];
    if (otpVal.startsWith('otpauth://')) {
      try {
        const url = new URL(otpVal);
        const secret = url.searchParams.get('secret') || '';
        const period = parseInt(url.searchParams.get('period') || '30');
        const digits = parseInt(url.searchParams.get('digits') || '6');
        const algo = (url.searchParams.get('algorithm') || 'SHA1').toUpperCase();
        return { type: 'totp', totp: { secret, period, digits, algorithm: algo } };
      } catch { /* ignore parse error */ }
    }
    // Raw secret
    if (otpVal && !otpVal.includes('://')) {
      return { type: 'totp', totp: { secret: otpVal, period: 30, digits: 6 } };
    }
  }
  return null;
}

function matchCategory(groupName: string, categories: CategoryOption[]): string {
  const lower = groupName.toLowerCase();
  const found = categories.find(c => c.label.toLowerCase() === lower || c.key.toLowerCase() === lower);
  return found?.key || categories[0]?.key || 'personal';
}

function parseGroup(groupEl: Element, categories: CategoryOption[]): ImportedEntry[] {
  const entries: ImportedEntry[] = [];
  const groupName = getText(groupEl, 'Name');
  const catKey = matchCategory(groupName, categories);

  // Parse direct child entries only (skip History)
  const entryEls = Array.from(groupEl.children).filter(c => c.tagName === 'Entry');
  for (const entryEl of entryEls) {
    const fields = getFieldStrings(entryEl);
    const { expired, ...times } = parseTimes(entryEl);
    if (expired) continue;
    const totpInfo = detectTotp(fields);

    const entry: ImportedEntry = {
      id: crypto.randomUUID(),
      icon: '🔐',
      issuer: fields['Title'] || '',
      title: fields['Title'] || '',
      label: fields['Title'] || '',
      username: fields['UserName'] || '',
      password: fields['Password'] || '',
      category: catKey,
      type: totpInfo?.type || 'password',
      ...times,
    };

    if (totpInfo?.totp) {
      entry.totp = totpInfo.totp;
      entry.secret = totpInfo.totp.secret;
    }

    if (entry.issuer || entry.username || entry.password) {
      entries.push(entry);
    }
  }

  // Recurse into sub-groups
  const subGroups = Array.from(groupEl.children).filter(c => c.tagName === 'Group');
  for (const sub of subGroups) {
    entries.push(...parseGroup(sub, categories));
  }

  return entries;
}

function collectGroupNames(groupEl: Element, names: Set<string>, categories: CategoryOption[]): void {
  const groupName = getText(groupEl, 'Name');
  if (groupName && !categories.find(c => c.label.toLowerCase() === groupName.toLowerCase() || c.key.toLowerCase() === groupName.toLowerCase())) {
    names.add(groupName);
  }
  const subGroups = Array.from(groupEl.children).filter(c => c.tagName === 'Group');
  for (const sub of subGroups) collectGroupNames(sub, names, categories);
}

export function collectUnmatchedGroups(xml: string, categories: CategoryOption[]): string[] {
  const parser = new DOMParser();
  const doc = parser.parseFromString(xml, 'text/xml');
  const names = new Set<string>();
  const rootGroups = doc.querySelectorAll('Root > Group');
  rootGroups.forEach(group => collectGroupNames(group, names, categories));
  return [...names];
}

export function parseKeePassXml(xml: string, categories: CategoryOption[]): ImportedEntry[] {
  const parser = new DOMParser();
  const doc = parser.parseFromString(xml, 'text/xml');

  const errorNode = doc.querySelector('parsererror');
  if (errorNode) throw new Error('XML 解析失败');

  const entries: ImportedEntry[] = [];
  const rootGroups = doc.querySelectorAll('Root > Group');
  rootGroups.forEach(group => entries.push(...parseGroup(group, categories)));

  return entries;
}
