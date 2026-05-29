'use strict';

import { computeTotp, type TotpConfig, type TotpResult } from './totp';

/* ══════════════════════════════════════════════════════════════
   INTERFACES
══════════════════════════════════════════════════════════════ */
interface Entry {
  id: string;
  icon?: string;
  issuer?: string;
  title?: string;
  label?: string;
  username?: string;
  password?: string;
  category?: string;
  type?: 'totp' | 'password';
  totp?: TotpConfig;
  secret?: string;
  createdAt?: number;
  updatedAt?: number;
}

interface AppSettings {
  autoLockTimeout: number;
  clipboardClearSeconds: number;
  maxErrorCount: number;
}

interface AppState {
  page: string;
  entries: Entry[];
  categories: Category[];
  filterCat: string;
  searchQ: string;
  viewMode: 'grid' | 'list';
  vaultPath: string | null;
  vaultOpen: boolean;
  settings: AppSettings;
  editingEntry: Entry | null;
}

interface ThemeMeta {
  t: string;
  name: string;
  c1: string;
  c2: string;
}

/* ══════════════════════════════════════════════════════════════
   CONSTANTS
══════════════════════════════════════════════════════════════ */
const CIRC = 2 * Math.PI * 16;
const ICONS = ['🔐', '🏦', '📧', '🐙', '🍎', '🤖', '🎮', '🛒', '💼', '🏠', '💳', '🌐'];
const DEFAULT_CATEGORIES: Category[] = [
  { key: 'work', label: '工作', icon: 'work', color: '#4CAF50' },
  { key: 'finance', label: '财务', icon: 'account_balance', color: '#FF9800' },
  { key: 'personal', label: '个人', icon: 'person', color: '#2196F3' },
];
const THEME_META: ThemeMeta[] = [
  { t: 'light', name: '白色', c1: '#FFFFFF', c2: '#1976D2' },
  { t: 'dark',  name: '黑色', c1: '#1E1E1E', c2: '#90CAF9' },
];

/* ══════════════════════════════════════════════════════════════
   STATE
══════════════════════════════════════════════════════════════ */
const S: AppState = {
  page: 'accounts',
  entries: [],
  categories: [],
  filterCat: 'all',
  searchQ: '',
  viewMode: (localStorage.getItem('viewMode') as 'grid' | 'list') || 'grid',
  vaultPath: localStorage.getItem('vaultPath') || null,
  vaultOpen: false,
  settings: JSON.parse(localStorage.getItem('settings') || '{"autoLockTimeout":300,"clipboardClearSeconds":30,"maxErrorCount":5}'),
  editingEntry: null,
};

const cardTimers = new Map<string, ReturnType<typeof setInterval>>();
let entryFormHTML = '';

/* ══════════════════════════════════════════════════════════════
   UTILITIES
══════════════════════════════════════════════════════════════ */
function $(sel: string): HTMLElement | null { return document.querySelector(sel); }
function $$(sel: string): NodeListOf<HTMLElement> { return document.querySelectorAll(sel); }
function getAppEl(): HTMLElement { return document.getElementById('app')!; }

function escHtml(str: string): string {
  return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function normalizeCategory(cat?: string): string {
  if (!cat) return S.categories[0]?.key || 'personal';
  const c = cat.toLowerCase();
  const found = S.categories.find(g => g.key === c || g.label === cat);
  return found ? found.key : (S.categories[0]?.key || 'personal');
}

/* ══════════════════════════════════════════════════════════════
   SNACKBAR & CLIPBOARD
══════════════════════════════════════════════════════════════ */
let snackTimer: ReturnType<typeof setTimeout> | null = null;

function showSnack(msg: string, action?: string, onAction?: () => void): void {
  const snack = document.getElementById('snack');
  if (!snack) return;
  snack.querySelector('.snack-msg')!.textContent = msg;
  const act = snack.querySelector('.snack-act') as HTMLElement;
  if (action && onAction) {
    act.textContent = action;
    act.style.display = '';
    act.onclick = onAction;
  } else {
    act.style.display = 'none';
  }
  snack.classList.add('show');
  if (snackTimer) clearTimeout(snackTimer);
  snackTimer = setTimeout(() => snack.classList.remove('show'), 2800);
}

async function copyText(text: string, feedbackEl?: HTMLElement | null): Promise<void> {
  await window.vaultxAPI.clipboard.write(text);
  showSnack(`已复制，将在 ${S.settings.clipboardClearSeconds} 秒后自动清除剪贴板`);
  if (feedbackEl) {
    feedbackEl.classList.add('copied');
    setTimeout(() => feedbackEl.classList.remove('copied'), 1500);
  }
}

/* ══════════════════════════════════════════════════════════════
   THEME
══════════════════════════════════════════════════════════════ */
function setTheme(t: string): void {
  getAppEl().dataset.theme = t;
  localStorage.setItem('theme', t);
  $$('.th-btn').forEach(b => b.classList.toggle('active', b.dataset.t === t));
}

(function initTheme() {
  getAppEl().dataset.theme = localStorage.getItem('theme') || 'dark';
})();

/* ══════════════════════════════════════════════════════════════
   ACTIVITY & AUTO-LOCK LISTENER
══════════════════════════════════════════════════════════════ */
(['mousemove', 'keydown', 'click', 'scroll'] as const).forEach(ev =>
  document.addEventListener(ev, () => window.vaultxAPI.pingActivity(), { passive: true }));

window.vaultxAPI.onVaultLocked(() => {
  S.vaultOpen = false;
  S.entries = [];
  renderLock();
  showSnack('金库已自动锁定');
});

/* ══════════════════════════════════════════════════════════════
   WIRE SHELL
══════════════════════════════════════════════════════════════ */
function wireShell(): void {
  $$('.snav-item[data-page]').forEach(btn => {
    btn.onclick = () => navigatePage(btn.dataset.page!);
  });

  document.getElementById('tb-lock')!.onclick = () => lockVault();
  document.getElementById('btn-add-entry')!.onclick = () => openAddSheet();
  document.getElementById('btn-view-toggle')!.onclick = () => toggleViewMode();
  document.getElementById('btn-import')!.onclick = () => importVault();
  document.getElementById('btn-export')!.onclick = () => exportVault();
  (document.getElementById('search-input') as HTMLInputElement).oninput = (e) => {
    S.searchQ = (e.target as HTMLInputElement).value.toLowerCase();
    filterCardsBySearch();
  };

  document.getElementById('sheet-close')!.onclick = closeSheet;
  document.getElementById('scrim')!.onclick = (e) => {
    if ((e.target as HTMLElement).id === 'scrim') closeSheet();
  };

  wireEntryForm();
  document.addEventListener('click', closeAllMenus);
}

function wireEntryForm(): void {
  const picker = document.getElementById('icon-picker')!;
  picker.innerHTML = '';
  ICONS.forEach(ic => {
    const opt = document.createElement('div');
    opt.className = 'icon-opt' + (ic === '🔐' ? ' picked' : '');
    opt.dataset.icon = ic;
    opt.textContent = ic;
    opt.onclick = () => {
      $$('.icon-opt').forEach(o => o.classList.remove('picked'));
      opt.classList.add('picked');
      (document.getElementById('f-icon') as HTMLInputElement).value = ic;
    };
    picker.appendChild(opt);
  });

  $$('.type-btn').forEach(btn => {
    btn.onclick = () => {
      $$('.type-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      (document.getElementById('f-type') as HTMLInputElement).value = btn.dataset.t!;
      document.getElementById('form-totp-only')!.classList.toggle('hidden', btn.dataset.t === 'password');
    };
  });

  const pwInput = document.getElementById('f-password') as HTMLInputElement;
  const pwEye = document.getElementById('pw-eye')!;
  pwEye.onclick = () => {
    const show = pwInput.type === 'password';
    pwInput.type = show ? 'text' : 'password';
    pwEye.querySelector('.material-icons-round')!.textContent = show ? 'visibility_off' : 'visibility';
  };

  document.getElementById('pw-gen')!.onclick = async () => {
    const r = await window.vaultxAPI.generator.generate({ length: 20, uppercase: true, lowercase: true, digits: true, symbols: true });
    if (r.ok && r.password) {
      pwInput.value = r.password;
      pwInput.type = 'text';
    }
  };

  document.getElementById('btn-save-entry')!.onclick = () => saveEntry();
}

/* ══════════════════════════════════════════════════════════════
   NAVIGATION
══════════════════════════════════════════════════════════════ */
function navigatePage(page: string): void {
  S.page = page;

  $$('.snav-item[data-page]').forEach(b =>
    b.classList.toggle('active', b.dataset.page === page));
  ['accounts', 'groups', 'security', 'settings'].forEach(p => {
    const el = document.getElementById('d-page-' + p);
    if (el) el.classList.toggle('active', p === page);
  });

  const pageNames: Record<string, string> = { accounts: '账户', groups: '分组', security: '安全', settings: '设置' };
  document.getElementById('ct-title')!.textContent = pageNames[page] || page;
  document.getElementById('ct-sub')!.textContent = '';

  const isAccounts = page === 'accounts';
  document.getElementById('toolbar-search')!.style.display = isAccounts ? '' : 'none';
  document.getElementById('btn-view-toggle')!.style.display = isAccounts ? '' : 'none';
  document.getElementById('btn-import')!.style.display = isAccounts ? '' : 'none';
  document.getElementById('btn-export')!.style.display = isAccounts ? '' : 'none';

  if (isAccounts) renderAccountsPage();
  else if (page === 'groups') renderGroupsPage();
  else if (page === 'security') renderSecurityPage();
  else if (page === 'settings') renderSettingsPage();

  renderSidebarCats();
  renderSidebarStats();
}

/* ══════════════════════════════════════════════════════════════
   SIDEBAR
══════════════════════════════════════════════════════════════ */
function renderSidebarCats(): void {
  const el = document.getElementById('sidebar-cats');
  if (!el) return;
  const allItem = `<button class="scat-item${S.filterCat === 'all' ? ' active' : ''}" data-cat="all">
    <span class="scat-dot" style="background:var(--on-surface-v)"></span>
    <span class="scat-name">全部</span>
    <span class="scat-count">${S.entries.length}</span>
  </button>`;
  el.innerHTML = allItem + S.categories.map(cat => {
    const count = S.entries.filter(e => normalizeCategory(e.category) === cat.key).length;
    return `<button class="scat-item${S.filterCat === cat.key ? ' active' : ''}" data-cat="${cat.key}">
      <span class="scat-dot" style="background:${cat.color}"></span>
      <span class="scat-name">${escHtml(cat.label)}</span>
      <span class="scat-count">${count}</span>
    </button>`;
  }).join('');
  el.querySelectorAll<HTMLElement>('.scat-item').forEach(btn => {
    btn.onclick = () => { S.filterCat = btn.dataset.cat!; renderSidebarCats(); renderAccountsPage(); };
  });
}

function renderSidebarStats(): void {
  const el = document.getElementById('sidebar-stats');
  if (!el) return;
  const totp = S.entries.filter(e => (e.type || 'totp') === 'totp').length;
  const pw = S.entries.filter(e => e.type === 'password').length;
  el.textContent = `${S.entries.length} 个账户 · ${totp} TOTP · ${pw} 密码`;
}

/* ══════════════════════════════════════════════════════════════
   ACCOUNTS PAGE
══════════════════════════════════════════════════════════════ */
function getFilteredEntries(): Entry[] {
  let list = S.entries;
  if (S.filterCat !== 'all') list = list.filter(e => normalizeCategory(e.category) === S.filterCat);
  if (S.searchQ) list = list.filter(e =>
    (e.issuer || '').toLowerCase().includes(S.searchQ) ||
    (e.label || '').toLowerCase().includes(S.searchQ) ||
    (e.username || '').toLowerCase().includes(S.searchQ));
  return list;
}

function renderAccountsPage(): void {
  const page = document.getElementById('d-page-accounts');
  if (!page) return;
  const list = getFilteredEntries();

  if (list.length === 0) {
    page.innerHTML = `<div class="empty-state">
      <span class="material-icons-round">lock_open</span>
      <p>${S.searchQ ? '没有匹配的账户' : '还没有账户，点击"添加账户"开始'}</p>
    </div>`;
    return;
  }

  const groups: Record<string, Entry[]> = {};
  S.categories.forEach(c => groups[c.key] = []);
  list.forEach(e => {
    const key = normalizeCategory(e.category);
    if (!groups[key]) groups[key] = [];
    groups[key].push(e);
  });

  const container = document.createElement('div');
  container.className = 'account-list';

  S.categories.forEach(cat => {
    const items = groups[cat.key] || [];
    if (!items.length) return;

    const sec = document.createElement('div');
    sec.className = 'sec-hd';
    sec.innerHTML = `<span class="sec-dot" style="background:${cat.color}"></span>
      <span class="sec-label">${escHtml(cat.label)}</span>
      <span class="sec-line"></span>
      <span class="sec-count">${items.length}</span>`;
    container.appendChild(sec);

    const grid = document.createElement('div');
    grid.className = 'cards-grid' + (S.viewMode === 'list' ? ' list-view' : '');
    items.forEach(entry => grid.appendChild(buildCard(entry)));
    container.appendChild(grid);
  });

  page.innerHTML = '';
  page.appendChild(container);

  if (S.searchQ) filterCardsBySearch();
  else document.getElementById('ct-sub')!.textContent = `${list.length} 个账户`;
}

function filterCardsBySearch(): void {
  const q = S.searchQ;
  const page = document.getElementById('d-page-accounts');
  if (!page) return;
  let visibleCount = 0;

  page.querySelectorAll<HTMLElement>('.card').forEach(card => {
    const entry = S.entries.find(e => String(e.id) === card.dataset.id);
    if (!entry) return;
    const match = !q ||
      (entry.issuer || '').toLowerCase().includes(q) ||
      (entry.label || '').toLowerCase().includes(q) ||
      (entry.username || '').toLowerCase().includes(q);
    card.style.display = match ? '' : 'none';
    if (match) visibleCount++;
  });

  page.querySelectorAll<HTMLElement>('.sec-hd').forEach(sec => {
    const grid = sec.nextElementSibling as HTMLElement | null;
    if (!grid) return;
    const hasVisible = Array.from(grid.querySelectorAll<HTMLElement>('.card')).some(c => c.style.display !== 'none');
    sec.style.display = hasVisible ? '' : 'none';
    grid.style.display = hasVisible ? '' : 'none';
  });

  document.getElementById('ct-sub')!.textContent = q ? `${visibleCount} 个匹配` : `${S.entries.length} 个账户`;
}

function toggleViewMode(): void {
  S.viewMode = S.viewMode === 'grid' ? 'list' : 'grid';
  localStorage.setItem('viewMode', S.viewMode);
  const icon = document.querySelector('#btn-view-toggle .material-icons-round');
  if (icon) icon.textContent = S.viewMode === 'grid' ? 'grid_view' : 'view_list';
  if (S.page === 'accounts') renderAccountsPage();
}

/* ══════════════════════════════════════════════════════════════
   CARD BUILDING
══════════════════════════════════════════════════════════════ */
function buildCard(entry: Entry): HTMLElement {
  const type = entry.type || 'totp';
  const tplId = type === 'totp' ? 'tpl-card-totp' : 'tpl-card-pw';
  const frag = (document.getElementById(tplId) as HTMLTemplateElement).content.cloneNode(true) as DocumentFragment;
  const card = frag.querySelector('.card') as HTMLElement;

  card.dataset.id = entry.id;
  card.dataset.cat = normalizeCategory(entry.category);
  card.querySelector('.avatar')!.textContent = entry.icon || '🔐';
  card.querySelector('.card-issuer')!.textContent = entry.issuer || entry.label || '未知';
  const label = (entry.label && entry.label !== entry.issuer) ? entry.label : (entry.username || '');
  card.querySelector('.card-label')!.textContent = label;

  const relTime = (ts?: number) => {
    if (!ts) return '—';
    const diff = Date.now() - ts;
    if (diff < 60_000) return '刚刚';
    if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} 分钟前`;
    if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)} 小时前`;
    if (diff < 2_592_000_000) return `${Math.floor(diff / 86_400_000)} 天前`;
    return `${Math.floor(diff / 2_592_000_000)} 个月前`;
  };
  const fullTime = (ts?: number) => ts ? new Date(ts).toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit' }) : '—';
  const menuBtn = card.querySelector('.menu-btn')!;
  const ts = document.createElement('div');
  ts.className = 'card-ts';
  ts.innerHTML = `<span class="ts-item" data-tip="创建于 ${fullTime(entry.createdAt)}">创建于 ${relTime(entry.createdAt)}</span><span class="ts-sep">·</span><span class="ts-item" data-tip="修改于 ${fullTime(entry.updatedAt)}">修改于 ${relTime(entry.updatedAt)}</span>`;
  menuBtn.before(ts);

  buildCardR3(card, entry);
  wireCard(card, entry);

  return card;
}

function buildCardR3(card: HTMLElement, entry: Entry): void {
  let r3 = card.querySelector('.card-r3') as HTMLElement | null;
  if (!r3) {
    r3 = document.createElement('div');
    r3.className = 'card-r3';
    card.querySelector('.card-in')!.appendChild(r3);
  }

  let html = '';
  if (entry.username) {
    html += `<span class="r3-section"><span class="cred-label">用户</span><span class="cred-val">${escHtml(entry.username)}</span></span>`;
  }
  if (entry.password) {
    html += `<span class="r3-divider"></span><span class="r3-section"><span class="cred-label">密码</span>
      <span class="cred-val masked-pw r3-pw">••••••••</span>
      <button class="pw-toggle r3-pw-toggle"><span class="material-icons-round">visibility</span></button>
      <button class="copy-chip r3-pw-copy" style="opacity:1;transform:scale(1)"><span class="material-icons-round">content_copy</span></button></span>`;
  }
  if ((entry.type || 'totp') === 'totp' && entry.totp?.secret) {
    html += `<span class="r3-divider"></span><span class="r3-section">
      <span class="r3-timer-wrap">
        <svg class="r3-timer-svg" width="22" height="22" viewBox="0 0 38 38">
          <circle class="t-bg" cx="19" cy="19" r="16"/>
          <circle class="t-ring r3-ring" cx="19" cy="19" r="16"
            stroke-dasharray="100.53" stroke-dashoffset="100.53" stroke="var(--primary)"/>
        </svg>
        <span class="r3-t-num">30</span>
      </span>
      <span class="otp r3-otp masked" id="r3-otp-${entry.id}">••• •••</span>
      <button class="copy-chip r3-otp-copy" style="opacity:1;transform:scale(1)"><span class="material-icons-round">content_copy</span></button></span>`;
  }
  if (html) {
    r3.innerHTML = html;
  } else {
    r3.remove();
  }
}

/* ══════════════════════════════════════════════════════════════
   CARD WIRING
══════════════════════════════════════════════════════════════ */
function wireCard(card: HTMLElement, entry: Entry): void {
  const type = entry.type || 'totp';

  const menuBtn = card.querySelector('.menu-btn') as HTMLElement;
  const ctxMenu = card.querySelector('.ctx-menu') as HTMLElement;
  menuBtn.onclick = (e) => {
    e.stopPropagation();
    const wasOpen = ctxMenu.classList.contains('open');
    closeAllMenus();
    if (!wasOpen) ctxMenu.classList.add('open');
  };
  (card.querySelector('[data-action="edit"]') as HTMLElement).onclick = (e) => { e.stopPropagation(); closeAllMenus(); openEditSheet(entry); };
  (card.querySelector('[data-action="delete"]') as HTMLElement).onclick = (e) => { e.stopPropagation(); closeAllMenus(); confirmDelete(entry); };

  if (type === 'totp') {
    wireTotpCard(card, entry);
  } else {
    wirePasswordCard(card, entry);
  }

  // Double-click to copy in list view sections
  wireR3DblCopy(card, entry);
}

function wireTotpCard(card: HTMLElement, entry: Entry): void {
  card.onclick = (e) => {
    if ((e.target as HTMLElement).closest('.menu-btn,.ctx-menu,.copy-chip,.r3-pw-toggle,.r3-pw-copy')) return;
    if (!card.classList.contains('revealed')) revealTOTP(card, entry);
  };

  (card.querySelector('.copy-chip') as HTMLElement).onclick = (e) => {
    e.stopPropagation();
    const code = card.querySelector('.otp')!.textContent!.replace(/\s/g, '');
    copyText(code, card.querySelector('.copy-chip') as HTMLElement | null);
  };

  wireR3Password(card, entry);
}

function wirePasswordCard(card: HTMLElement, entry: Entry): void {
  const pwDisplay = card.querySelector('.pw-display') as HTMLElement;
  const copyChip = card.querySelector('.card-r2-pw .copy-chip') as HTMLElement;
  let pwVisible = false;

  card.onclick = (e) => {
    if ((e.target as HTMLElement).closest('.menu-btn,.ctx-menu')) return;

    if ((e.target as HTMLElement).closest('.card-r2-pw .pw-toggle')) {
      pwVisible = !pwVisible;
      pwDisplay.textContent = pwVisible ? (entry.password || '') : '••••••••';
      pwDisplay.classList.toggle('revealed-pw', pwVisible);
      (card.querySelector('.card-r2-pw .pw-toggle .material-icons-round') as HTMLElement).textContent = pwVisible ? 'visibility_off' : 'visibility';
      return;
    }

    if ((e.target as HTMLElement).closest('.card-r2-pw .copy-chip')) {
      copyText(entry.password || '', copyChip);
      return;
    }

    if ((e.target as HTMLElement).closest('.r3-pw-toggle')) {
      const r3Pw = card.querySelector('.r3-pw') as HTMLElement;
      if (r3Pw) {
        const vis = r3Pw.classList.contains('revealed-pw');
        r3Pw.textContent = vis ? '••••••••' : (entry.password || '');
        r3Pw.classList.toggle('masked-pw', vis);
        r3Pw.classList.toggle('revealed-pw', !vis);
        const icon = card.querySelector('.r3-pw-toggle .material-icons-round') as HTMLElement;
        if (icon) icon.textContent = vis ? 'visibility' : 'visibility_off';
      }
      return;
    }
    if ((e.target as HTMLElement).closest('.r3-pw-copy')) {
      copyText(entry.password || '');
      return;
    }

    card.classList.toggle('revealed');
  };
}

function wireR3DblCopy(card: HTMLElement, entry: Entry): void {
  const sections = card.querySelectorAll('.r3-section');
  sections.forEach(sec => {
    const el = sec as HTMLElement;
    // Determine what this section contains
    if (el.querySelector('.cred-val') && !el.querySelector('.r3-pw')) {
      // Username section
      el.ondblclick = (e) => {
        e.stopPropagation();
        if (entry.username) copyText(entry.username, el);
      };
    } else if (el.querySelector('.r3-pw')) {
      // Password section
      el.ondblclick = (e) => {
        e.stopPropagation();
        if (entry.password) copyText(entry.password, el);
      };
    } else if (el.querySelector('.r3-otp')) {
      // TOTP section
      el.ondblclick = (e) => {
        e.stopPropagation();
        const otpText = el.querySelector('.r3-otp')?.textContent?.replace(/\s/g, '');
        if (otpText && !otpText.includes('•')) copyText(otpText, el);
      };
    }
  });
}

function wireR3Password(card: HTMLElement, entry: Entry): void {
  const r3PwEl = card.querySelector('.r3-pw') as HTMLElement;
  const r3PwToggle = card.querySelector('.r3-pw-toggle') as HTMLElement;
  const r3PwCopy = card.querySelector('.r3-pw-copy') as HTMLElement;
  if (!r3PwEl || !entry.password) return;

  r3PwToggle.onclick = (e) => {
    e.stopPropagation();
    const visible = r3PwEl.classList.contains('revealed-pw');
    r3PwEl.textContent = visible ? '••••••••' : entry.password!;
    r3PwEl.classList.toggle('masked-pw', visible);
    r3PwEl.classList.toggle('revealed-pw', !visible);
    (r3PwToggle.querySelector('.material-icons-round') as HTMLElement).textContent = visible ? 'visibility' : 'visibility_off';
  };

  r3PwCopy.onclick = (e) => { e.stopPropagation(); copyText(entry.password!, r3PwCopy); };
}

function closeAllMenus(): void {
  $$('.ctx-menu.open').forEach(m => m.classList.remove('open'));
}

/* ══════════════════════════════════════════════════════════════
   TOTP REVEAL & TIMER
══════════════════════════════════════════════════════════════ */
function revealTOTP(card: HTMLElement, entry: Entry): void {
  card.classList.add('revealed');
  const otpEl = card.querySelector('.card-r2 .otp') as HTMLElement;
  const ringEl = card.querySelector('.card-r2 .t-ring') as SVGCircleElement;
  const numEl = card.querySelector('.card-r2 .t-num') as HTMLElement;
  const timerWrap = card.querySelector('.timer-wrap') as HTMLElement;
  const r3OtpEl = card.querySelector('.r3-otp') as HTMLElement | null;
  const r3RingEl = card.querySelector('.r3-ring') as SVGCircleElement | null;
  const r3NumEl = card.querySelector('.r3-t-num') as HTMLElement | null;
  if (timerWrap) timerWrap.style.display = '';

  async function update(): Promise<void> {
    const conf: TotpConfig = entry.totp || { secret: entry.secret || '', algorithm: 'SHA1', digits: 6, period: 30 };
    try {
      const r: TotpResult = await computeTotp(conf);
      const d = conf.digits || 6;
      const codeText = d === 6 ? r.code.slice(0, 3) + ' ' + r.code.slice(3) : r.code;
      if (otpEl) {
        otpEl.textContent = codeText;
        otpEl.classList.remove('masked', 'warn', 'danger');
        if (r.remaining <= 5) otpEl.classList.add('danger');
        else if (r.remaining <= 10) otpEl.classList.add('warn');
      }
      if (r3OtpEl) {
        r3OtpEl.textContent = codeText;
        r3OtpEl.classList.remove('masked', 'warn', 'danger');
        if (r.remaining <= 5) r3OtpEl.classList.add('danger');
        else if (r.remaining <= 10) r3OtpEl.classList.add('warn');
      }
      const p = conf.period || 30;
      const col = r.remaining <= 5 ? 'var(--danger)' : r.remaining <= 10 ? 'var(--warn)' : 'var(--primary)';
      if (ringEl) {
        ringEl.style.strokeDashoffset = (CIRC * (1 - r.remaining / p)).toFixed(2);
        ringEl.style.stroke = col;
        numEl.style.color = col;
        numEl.textContent = String(r.remaining);
      }
      if (r3RingEl) {
        const r3Circ = 2 * Math.PI * 16;
        r3RingEl.style.strokeDashoffset = (r3Circ * (1 - r.remaining / p)).toFixed(2);
        r3RingEl.style.stroke = col;
      }
      if (r3NumEl) {
        r3NumEl.style.color = col;
        r3NumEl.textContent = String(r.remaining);
      }
    } catch {
      if (otpEl) otpEl.textContent = '错误';
      if (r3OtpEl) r3OtpEl.textContent = '错误';
    }
  }

  update();
  const id = entry.id;
  if (cardTimers.has(id)) clearInterval(cardTimers.get(id));
  cardTimers.set(id, setInterval(update, 1000));
}

/* ══════════════════════════════════════════════════════════════
   ADD / EDIT SHEET
══════════════════════════════════════════════════════════════ */
function openAddSheet(): void {
  S.editingEntry = null;
  document.getElementById('sheet-form')!.innerHTML = entryFormHTML;
  wireEntryForm();
  buildCatSelect();
  resetSheet();
  document.getElementById('scrim')!.classList.add('open');
}

function openEditSheet(entry: Entry): void {
  S.editingEntry = entry;
  document.getElementById('sheet-form')!.innerHTML = entryFormHTML;
  wireEntryForm();
  buildCatSelect();
  populateSheet(entry);
  document.getElementById('scrim')!.classList.add('open');
}

function closeSheet(): void {
  document.getElementById('scrim')!.classList.remove('open');
}

function buildCatSelect(): void {
  const sel = document.getElementById('f-cat') as HTMLSelectElement | null;
  if (!sel) return;
  sel.innerHTML = S.categories.map(c => `<option value="${c.key}">${escHtml(c.label)}</option>`).join('');
}

function resetSheet(): void {
  document.getElementById('sheet-title')!.textContent = '添加账户';
  document.getElementById('btn-save-entry')!.innerHTML = '<span class="material-icons-round">add</span>添加账户';
  (document.getElementById('f-icon') as HTMLInputElement).value = '🔐';
  $$('.icon-opt').forEach((o, i) => o.classList.toggle('picked', i === 0));
  (document.getElementById('f-type') as HTMLInputElement).value = 'totp';
  $$('.type-btn').forEach(b => b.classList.toggle('active', b.dataset.t === 'totp'));
  document.getElementById('form-totp-only')!.classList.remove('hidden');
  (document.getElementById('f-issuer') as HTMLInputElement).value = '';
  (document.getElementById('f-label') as HTMLInputElement).value = '';
  (document.getElementById('f-username') as HTMLInputElement).value = '';
  (document.getElementById('f-secret') as HTMLInputElement).value = '';
  (document.getElementById('f-algo') as HTMLSelectElement).value = 'SHA1';
  (document.getElementById('f-period') as HTMLSelectElement).value = '30';
  (document.getElementById('f-password') as HTMLInputElement).value = '';
  (document.getElementById('f-password') as HTMLInputElement).type = 'password';
  document.getElementById('pw-eye')!.querySelector('.material-icons-round')!.textContent = 'visibility';
  (document.getElementById('f-cat') as HTMLSelectElement).value = S.categories[0]?.key || 'personal';
}

function populateSheet(entry: Entry): void {
  resetSheet();
  const type = entry.type || 'totp';
  document.getElementById('sheet-title')!.textContent = '编辑账户';
  document.getElementById('btn-save-entry')!.innerHTML = '<span class="material-icons-round">save</span>保存更改';
  (document.getElementById('f-icon') as HTMLInputElement).value = entry.icon || '🔐';
  $$('.icon-opt').forEach(o => o.classList.toggle('picked', o.dataset.icon === entry.icon));
  (document.getElementById('f-type') as HTMLInputElement).value = type;
  $$('.type-btn').forEach(b => b.classList.toggle('active', b.dataset.t === type));
  document.getElementById('form-totp-only')!.classList.toggle('hidden', type === 'password');
  (document.getElementById('f-issuer') as HTMLInputElement).value = entry.issuer || '';
  (document.getElementById('f-label') as HTMLInputElement).value = entry.label || '';
  (document.getElementById('f-username') as HTMLInputElement).value = entry.username || '';
  (document.getElementById('f-password') as HTMLInputElement).value = entry.password || '';
  if (type === 'totp') {
    const totp = entry.totp || {} as TotpConfig;
    (document.getElementById('f-secret') as HTMLInputElement).value = totp.secret || entry.secret || '';
    (document.getElementById('f-algo') as HTMLSelectElement).value = totp.algorithm || 'SHA1';
    (document.getElementById('f-period') as HTMLSelectElement).value = String(totp.period || 30);
  }
  (document.getElementById('f-cat') as HTMLSelectElement).value = normalizeCategory(entry.category);
}

async function saveEntry(): Promise<void> {
  const isEdit = !!S.editingEntry;
  const existing = S.editingEntry;
  const type = (document.getElementById('f-type') as HTMLInputElement).value;
  const issuer = (document.getElementById('f-issuer') as HTMLInputElement).value.trim();
  const secret = (document.getElementById('f-secret') as HTMLInputElement).value.trim();
  if (!issuer) { showSnack('请填写服务名称'); return; }
  if (type === 'totp' && !secret) { showSnack('请填写 TOTP 密钥'); return; }

  const pwValue = (document.getElementById('f-password') as HTMLInputElement).value;
  const now = Date.now();
  const entryData: Entry = {
    id: isEdit && existing ? existing.id : crypto.randomUUID(),
    icon: (document.getElementById('f-icon') as HTMLInputElement).value,
    issuer,
    title: issuer,
    label: (document.getElementById('f-label') as HTMLInputElement).value.trim(),
    username: (document.getElementById('f-username') as HTMLInputElement).value.trim(),
    category: (document.getElementById('f-cat') as HTMLSelectElement).value,
    type: type as 'totp' | 'password',
    createdAt: isEdit && existing ? existing.createdAt : now,
    updatedAt: now,
  };

  if (type === 'totp') {
    entryData.totp = {
      secret,
      algorithm: (document.getElementById('f-algo') as HTMLSelectElement).value,
      digits: 6,
      period: parseInt((document.getElementById('f-period') as HTMLSelectElement).value),
    };
    entryData.password = pwValue || (existing ? existing.password : '');
  } else {
    entryData.password = pwValue;
  }

  try {
    if (isEdit && existing) {
      await window.vaultxAPI.vault.updateEntry(entryData);
      showSnack('账户已更新');
    } else {
      await window.vaultxAPI.vault.addEntry(entryData);
      showSnack('账户已添加');
    }
    closeSheet();
    await refreshEntries();
  } catch (e: any) {
    showSnack('保存失败: ' + (e.message || e));
  }
}

/* ══════════════════════════════════════════════════════════════
   DELETE
══════════════════════════════════════════════════════════════ */
function confirmDelete(entry: Entry): void {
  const overlay = document.getElementById('confirm-overlay') as HTMLElement;
  document.getElementById('confirm-title')!.textContent = '删除账户';
  document.getElementById('confirm-msg')!.textContent = `确定要删除"${entry.issuer || entry.label}"吗？`;
  overlay.style.display = 'flex';
  document.getElementById('confirm-cancel')!.onclick = () => { overlay.style.display = 'none'; };
  document.getElementById('confirm-ok')!.onclick = async () => {
    overlay.style.display = 'none';
    await window.vaultxAPI.vault.deleteEntry(entry.id);
    showSnack('账户已删除');
    await refreshEntries();
  };
}

async function refreshEntries(): Promise<void> {
  const ge = await window.vaultxAPI.vault.getEntries();
  S.entries = ge.ok ? (ge.entries as Entry[]) : [];
  renderSidebarCats();
  renderSidebarStats();
  if (S.page === 'accounts') renderAccountsPage();
}

/* ══════════════════════════════════════════════════════════════
   GROUPS PAGE
══════════════════════════════════════════════════════════════ */
function renderGroupsPage(): void {
  const page = document.getElementById('d-page-groups');
  if (!page) return;
  const relTime = (ts?: number) => {
    if (!ts) return '—';
    const diff = Date.now() - ts;
    if (diff < 60_000) return '刚刚';
    if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} 分钟前`;
    if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)} 小时前`;
    if (diff < 2_592_000_000) return `${Math.floor(diff / 86_400_000)} 天前`;
    return `${Math.floor(diff / 2_592_000_000)} 个月前`;
  };
  const fullTime = (ts?: number) => ts ? new Date(ts).toLocaleString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit' }) : '—';
  const cats = S.categories;
  page.innerHTML = '<div class="d-group-grid">' + cats.map(g => {
    const count = S.entries.filter(e => normalizeCategory(e.category) === g.key).length;
    return `<div class="d-group-card">
      <div class="d-group-card-top">
        <div class="d-group-icon" style="background:${g.color}"><span class="material-icons-round">${g.icon}</span></div>
        <div style="flex:1;min-width:0"><div class="d-group-name">${escHtml(g.label)}</div><div class="d-group-key">${escHtml(g.key)}</div></div>
        <div class="card-ts"><span class="ts-item" data-tip="创建于 ${fullTime(g.createdAt)}">创建于 ${relTime(g.createdAt)}</span><span class="ts-sep">·</span><span class="ts-item" data-tip="修改于 ${fullTime(g.updatedAt)}">修改于 ${relTime(g.updatedAt)}</span></div>
      </div>
      <div class="d-group-stats"><span class="d-group-stat">${count} 个账户</span></div>
      <div class="d-group-actions">
        <button class="d-group-btn" data-g="${g.key}"><span class="material-icons-round">visibility</span>查看</button>
        <button class="d-group-btn d-group-edit" data-g="${g.key}"><span class="material-icons-round">edit</span>编辑</button>
        <button class="d-group-btn d-group-del" data-g="${g.key}"><span class="material-icons-round">delete</span>删除</button>
      </div>
    </div>`;
  }).join('') + `<div class="d-group-card d-group-add" id="btn-add-cat">
      <div class="d-group-card-top">
        <div class="d-group-icon" style="background:var(--on-surface-v)"><span class="material-icons-round">add</span></div>
        <div><div class="d-group-name">添加分组</div></div>
      </div>
    </div></div>`;

  page.querySelectorAll<HTMLElement>('.d-group-btn:not(.d-group-edit):not(.d-group-del)').forEach(b => {
    b.onclick = () => { S.filterCat = b.dataset.g!; navigatePage('accounts'); };
  });
  page.querySelectorAll<HTMLElement>('.d-group-edit').forEach(b => {
    b.onclick = () => openCatSheet(b.dataset.g!);
  });
  page.querySelectorAll<HTMLElement>('.d-group-del').forEach(b => {
    b.onclick = () => deleteCategory(b.dataset.g!);
  });
  document.getElementById('btn-add-cat')?.addEventListener('click', () => openCatSheet());
}

const CAT_ICONS = ['work', 'account_balance', 'person', 'school', 'shopping_cart', 'local_hospital', 'flight', 'code', 'sports_esports', 'music_note', 'restaurant', 'pets'];
const CAT_COLORS_PRESET = ['#4CAF50', '#FF9800', '#2196F3', '#9C27B0', '#F44336', '#00BCD4', '#795548', '#607D8B', '#E91E63', '#3F51B5', '#009688', '#FFC107'];

function openCatSheet(editKey?: string): void {
  const existing = editKey ? S.categories.find(c => c.key === editKey) : null;
  document.getElementById('sheet-title')!.textContent = existing ? '编辑分组' : '添加分组';
  const form = document.getElementById('sheet-form')!;
  form.innerHTML = `
    <div class="field"><input type="text" id="cat-label" placeholder=" " value="${existing ? escHtml(existing.label) : ''}"><label>名称</label></div>
    <div class="field"><input type="text" id="cat-key" placeholder=" " value="${existing ? escHtml(existing.key) : ''}" ${existing ? 'readonly' : ''}><label>标识</label></div>
    <div class="form-label-sm">图标</div>
    <div class="icon-picker-row">${CAT_ICONS.map(ic =>
      `<button class="icon-opt${existing?.icon === ic ? ' picked' : ''}" data-icon="${ic}"><span class="material-icons-round">${ic}</span></button>`
    ).join('')}</div>
    <div class="form-label-sm">颜色</div>
    <div class="icon-picker-row">${CAT_COLORS_PRESET.map(c =>
      `<button class="color-opt${existing?.color === c ? ' picked' : ''}" data-color="${c}" style="background:${c}"></button>`
    ).join('')}</div>
    <button class="btn-fill" id="btn-save-cat"><span class="material-icons-round">${existing ? 'save' : 'add'}</span>${existing ? '保存' : '添加'}</button>`;

  form.querySelectorAll<HTMLElement>('.icon-opt').forEach(btn => {
    btn.onclick = () => { form.querySelectorAll('.icon-opt').forEach(b => b.classList.remove('picked')); btn.classList.add('picked'); };
  });
  form.querySelectorAll<HTMLElement>('.color-opt').forEach(btn => {
    btn.onclick = () => { form.querySelectorAll('.color-opt').forEach(b => b.classList.remove('picked')); btn.classList.add('picked'); };
  });

  document.getElementById('btn-save-cat')!.onclick = async () => {
    const label = (document.getElementById('cat-label') as HTMLInputElement).value.trim();
    const key = (document.getElementById('cat-key') as HTMLInputElement).value.trim().toLowerCase();
    const icon = form.querySelector('.icon-opt.picked')?.getAttribute('data-icon') || 'label';
    const color = form.querySelector('.color-opt.picked')?.getAttribute('data-color') || '#607D8B';
    if (!label || !key) { showSnack('请填写名称和标识'); return; }
    if (!/^[a-z][a-z0-9_]*$/.test(key)) { showSnack('标识只能包含小写字母、数字和下划线'); return; }
    const now = Date.now();
    const cat: Category = { key, label, icon, color, createdAt: existing?.createdAt ?? now, updatedAt: now };
    try {
      if (existing) {
        await window.vaultxAPI.vault.updateCategory(cat);
      } else {
        if (S.categories.some(c => c.key === key)) { showSnack('标识已存在'); return; }
        await window.vaultxAPI.vault.addCategory(cat);
      }
      const res = await window.vaultxAPI.vault.getCategories();
      if (res.ok && res.categories) S.categories = res.categories;
      closeSheet();
      renderGroupsPage();
      renderSidebarCats();
      showSnack(existing ? '分组已更新' : '分组已添加');
    } catch (e: any) {
      showSnack('保存失败: ' + (e.message || e));
    }
  };
  document.getElementById('scrim')!.classList.add('open');
}

async function deleteCategory(key: string): Promise<void> {
  const count = S.entries.filter(e => normalizeCategory(e.category) === key).length;
  if (count > 0) { showSnack(`该分组下有 ${count} 个账户，无法删除`); return; }
  try {
    await window.vaultxAPI.vault.deleteCategory(key);
    const res = await window.vaultxAPI.vault.getCategories();
    if (res.ok && res.categories) S.categories = res.categories;
    renderGroupsPage();
    renderSidebarCats();
    showSnack('分组已删除');
  } catch (e: any) {
    showSnack('删除失败: ' + (e.message || e));
  }
}

/* ══════════════════════════════════════════════════════════════
   SECURITY PAGE
══════════════════════════════════════════════════════════════ */
function renderSecurityPage(): void {
  const page = document.getElementById('d-page-security');
  if (!page) return;
  page.innerHTML = `<div class="settings-wrap">
    <div class="settings-card">
      <div class="settings-card-hd">金库状态</div>
      <div class="s-item no-tap">
        <div class="s-icon" style="background:var(--success)"><span class="material-icons-round">verified_user</span></div>
        <div class="s-body"><div class="s-label">加密算法</div><div class="s-sub">AES-256-GCM + Argon2id</div></div>
      </div>
      <div class="s-item no-tap">
        <div class="s-icon" style="background:var(--primary)"><span class="material-icons-round">storage</span></div>
        <div class="s-body"><div class="s-label">金库路径</div><div class="s-sub" style="word-break:break-all">${escHtml(S.vaultPath || '未知')}</div></div>
      </div>
      <div class="s-item no-tap">
        <div class="s-icon" style="background:var(--warn)"><span class="material-icons-round">numbers</span></div>
        <div class="s-body"><div class="s-label">账户数量</div><div class="s-sub">${S.entries.length} 个账户</div></div>
      </div>
    </div>
    <div class="settings-card">
      <div class="settings-card-hd">操作</div>
      <div class="s-item danger" id="sec-lock">
        <div class="s-icon" style="background:var(--danger)"><span class="material-icons-round">lock</span></div>
        <div class="s-body"><div class="s-label">立即锁定</div></div>
      </div>
    </div>
  </div>`;
  (page.querySelector('#sec-lock') as HTMLElement).onclick = () => lockVault();
}

/* ══════════════════════════════════════════════════════════════
   SETTINGS PAGE
══════════════════════════════════════════════════════════════ */
function renderSettingsPage(): void {
  const page = document.getElementById('d-page-settings');
  if (!page) return;
  const cur = getAppEl().dataset.theme || 'c';
  const swatches = THEME_META.map(tm =>
    `<button class="th-btn${tm.t === cur ? ' active' : ''}" data-t="${tm.t}">
      <div class="th-swatch"><div class="th-half" style="background:${tm.c1}"></div><div class="th-half" style="background:${tm.c2}"></div></div>
      <span class="th-name">${tm.name}</span>
    </button>`).join('');

  page.innerHTML = `<div class="settings-wrap">
    <div class="settings-card">
      <div class="settings-card-hd">主题</div>
      <div class="s-item no-tap" style="flex-wrap:wrap;gap:8px;padding:14px 18px;">${swatches}</div>
    </div>
    <div class="settings-row-grid">
      <div class="settings-card">
        <div class="settings-card-hd">安全</div>
        <div class="s-item no-tap">
          <div class="s-body"><div class="s-label">自动锁定</div><div class="s-sub">闲置后自动锁定</div></div>
          <div class="s-ctrl"><select class="s-select" id="st-lock">${[1, 5, 10, 30, 60, 0].map(m => `<option value="${m * 60}"${S.settings.autoLockTimeout === (m * 60) ? ' selected' : ''}>${m === 0 ? '从不' : m < 60 ? m + '分钟' : '1小时'}</option>`).join('')}</select></div>
        </div>
        <div class="s-item no-tap">
          <div class="s-body"><div class="s-label">剪贴板清除</div></div>
          <div class="s-ctrl"><select class="s-select" id="st-clip">${[15, 30, 60, 120].map(s => `<option value="${s}"${S.settings.clipboardClearSeconds === s ? ' selected' : ''}>${s}秒</option>`).join('')}</select></div>
        </div>
      </div>
      <div class="settings-card">
        <div class="settings-card-hd">数据</div>
        <div class="s-item" id="st-import">
          <div class="s-icon" style="background:#4CAF50"><span class="material-icons-round">upload</span></div>
          <div class="s-body"><div class="s-label">导入金库</div></div>
          <span class="material-icons-round s-chevron">chevron_right</span>
        </div>
        <div class="s-item" id="st-export">
          <div class="s-icon" style="background:#2196F3"><span class="material-icons-round">download</span></div>
          <div class="s-body"><div class="s-label">导出金库</div></div>
          <span class="material-icons-round s-chevron">chevron_right</span>
        </div>
      </div>
    </div>
    <div class="settings-card">
      <div class="settings-card-hd">关于</div>
      <div class="s-item no-tap">
        <div class="s-icon" style="background:var(--primary)"><span class="material-icons-round">shield</span></div>
        <div class="s-body"><div class="s-label">VaultX</div><div class="s-sub">版本 1.0.0 · Electron</div></div>
      </div>
    </div>
  </div>`;

  page.querySelectorAll<HTMLElement>('.th-btn').forEach(b => { b.onclick = () => setTheme(b.dataset.t!); });
  (page.querySelector('#st-lock') as HTMLSelectElement).onchange = (e) => { S.settings.autoLockTimeout = parseInt((e.target as HTMLSelectElement).value); saveSettings(); };
  (page.querySelector('#st-clip') as HTMLSelectElement).onchange = (e) => { S.settings.clipboardClearSeconds = parseInt((e.target as HTMLSelectElement).value); saveSettings(); };
  (page.querySelector('#st-import') as HTMLElement).onclick = () => importVault();
  (page.querySelector('#st-export') as HTMLElement).onclick = () => exportVault();
}

async function saveSettings(): Promise<void> {
  localStorage.setItem('settings', JSON.stringify(S.settings));
  await window.vaultxAPI.settings.update(S.settings);
}

/* ══════════════════════════════════════════════════════════════
   IMPORT / EXPORT
══════════════════════════════════════════════════════════════ */
async function importVault(): Promise<void> {
  const path = await window.vaultxAPI.dialog.openFile();
  if (path) showSnack('导入功能待实现');
}

async function exportVault(): Promise<void> {
  const path = await window.vaultxAPI.dialog.saveFile();
  if (path) showSnack('导出功能待实现');
}

/* ══════════════════════════════════════════════════════════════
   VAULT: LOCK / UNLOCK / INIT / CREATE
══════════════════════════════════════════════════════════════ */
async function lockVault(): Promise<void> {
  await window.vaultxAPI.vault.lock();
  S.vaultOpen = false;
  S.entries = [];
  S.categories = [];
  renderLock();
}

async function initVault(): Promise<void> {
  if (!S.vaultPath) { renderCreateVault(); return; }
  const status = await window.vaultxAPI.vault.check(S.vaultPath);
  if (!status.ok || !status.exists) { renderCreateVault(); return; }
  renderLock();
}

async function changeVault(): Promise<void> {
  const path = await window.vaultxAPI.dialog.openFile();
  if (path) {
    S.vaultPath = path;
    localStorage.setItem('vaultPath', path);
    initVault();
  }
}

function renderLock(): void {
  const lockOv = document.getElementById('lock-ov') as HTMLElement;
  if (!lockOv) return;
  lockOv.innerHTML = `
    <div class="lock-logo"><span class="material-icons-round">shield</span></div>
    <div class="lock-h">VaultX</div>
    <div class="lock-sub">请输入主密码解锁金库</div>
    <div class="lock-form">
      <div class="lock-input-wrap">
        <input class="lock-input" id="lock-pw" type="password" placeholder="主密码" autocomplete="current-password">
        <button class="lock-eye" id="lock-eye"><span class="material-icons-round">visibility</span></button>
      </div>
      <div class="lock-err" id="lock-err"></div>
      <button class="lock-btn" id="lock-submit">解锁</button>
      <div class="lock-vault-row">
        <span class="material-icons-round">storage</span>
        <span style="flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${escHtml(S.vaultPath || '未知路径')}</span>
        <button class="lock-link" id="lock-change">切换</button>
      </div>
    </div>`;
  lockOv.classList.add('show');

  const pwInput = lockOv.querySelector('#lock-pw') as HTMLInputElement;
  const eyeBtn = lockOv.querySelector('#lock-eye') as HTMLElement;
  const errEl = lockOv.querySelector('#lock-err') as HTMLElement;
  const submitBtn = lockOv.querySelector('#lock-submit') as HTMLButtonElement;

  eyeBtn.onclick = () => {
    const show = pwInput.type === 'password';
    pwInput.type = show ? 'text' : 'password';
    eyeBtn.querySelector('.material-icons-round')!.textContent = show ? 'visibility_off' : 'visibility';
  };

  pwInput.onkeydown = (e) => { if (e.key === 'Enter') submitBtn.click(); };

  submitBtn.onclick = async () => {
    const pw = pwInput.value;
    if (!pw) { errEl.textContent = '请输入密码'; errEl.classList.add('show'); return; }
    submitBtn.disabled = true;
    submitBtn.textContent = '解锁中…';
    try {
      const openR = await window.vaultxAPI.vault.open(S.vaultPath!);
      if (!openR.ok) {
        errEl.textContent = '无法打开金库: ' + openR.error;
        errEl.classList.add('show');
        submitBtn.disabled = false;
        submitBtn.textContent = '解锁';
        return;
      }
      const unlockR = await window.vaultxAPI.vault.unlock(pw);
      if (unlockR.ok) {
        lockOv.classList.remove('show');
        S.vaultOpen = true;
        const [ge, gc] = await Promise.all([
          window.vaultxAPI.vault.getEntries(),
          window.vaultxAPI.vault.getCategories(),
        ]);
        S.entries = ge.ok ? (ge.entries as Entry[]) : [];
        S.categories = gc.ok && gc.categories?.length ? gc.categories : [...DEFAULT_CATEGORIES];
        buildCatSelect();
        renderSidebarCats();
        renderSidebarStats();
        navigatePage('accounts');
      } else {
        errEl.textContent = '密码错误，请重试';
        errEl.classList.add('show');
        pwInput.value = '';
        pwInput.focus();
        submitBtn.disabled = false;
        submitBtn.textContent = '解锁';
      }
    } catch (e: any) {
      errEl.textContent = '解锁失败: ' + (e.message || e);
      errEl.classList.add('show');
      submitBtn.disabled = false;
      submitBtn.textContent = '解锁';
    }
  };

  (lockOv.querySelector('#lock-change') as HTMLElement).onclick = () => changeVault();
  setTimeout(() => pwInput.focus(), 100);
}

function renderCreateVault(): void {
  const lockOv = document.getElementById('lock-ov') as HTMLElement;
  lockOv.innerHTML = `
    <div class="lock-logo"><span class="material-icons-round">add_circle</span></div>
    <div class="lock-h">创建金库</div>
    <div class="lock-sub">首次使用，请设置主密码</div>
    <div class="lock-form">
      <div class="lock-input-wrap">
        <input class="lock-input" id="create-pw" type="password" placeholder="主密码">
        <button class="lock-eye" id="create-eye"><span class="material-icons-round">visibility</span></button>
      </div>
      <div class="lock-input-wrap">
        <input class="lock-input" id="create-pw2" type="password" placeholder="确认密码">
      </div>
      <div class="lock-err" id="create-err"></div>
      <button class="lock-btn" id="create-submit">创建金库</button>
      <div class="lock-vault-row">
        <span class="material-icons-round">storage</span>
        <button class="lock-link" id="create-choose">选择保存位置</button>
      </div>
    </div>`;
  lockOv.classList.add('show');

  const pw1 = lockOv.querySelector('#create-pw') as HTMLInputElement;
  const pw2 = lockOv.querySelector('#create-pw2') as HTMLInputElement;
  const errEl = lockOv.querySelector('#create-err') as HTMLElement;
  const eye = lockOv.querySelector('#create-eye') as HTMLElement;

  eye.onclick = () => {
    const s = pw1.type === 'password';
    pw1.type = s ? 'text' : 'password';
    eye.querySelector('.material-icons-round')!.textContent = s ? 'visibility_off' : 'visibility';
  };

  (lockOv.querySelector('#create-choose') as HTMLElement).onclick = async () => {
    const path = await window.vaultxAPI.dialog.saveFile();
    if (path) {
      S.vaultPath = path;
      localStorage.setItem('vaultPath', path);
      showSnack('位置: ' + path);
    }
  };

  (lockOv.querySelector('#create-submit') as HTMLElement).onclick = async () => {
    const p1 = pw1.value, p2 = pw2.value;
    if (!p1) { errEl.textContent = '请输入密码'; errEl.classList.add('show'); return; }
    if (p1 !== p2) { errEl.textContent = '两次密码不一致'; errEl.classList.add('show'); return; }
    if (p1.length < 8) { errEl.textContent = '密码至少 8 位'; errEl.classList.add('show'); return; }
    const path = S.vaultPath || (Date.now() + '.vaultx');
    const r = await window.vaultxAPI.vault.create(path, p1);
    if (!r.ok) { errEl.textContent = '创建失败: ' + (r.error || '未知错误'); errEl.classList.add('show'); return; }
    S.vaultPath = path;
    localStorage.setItem('vaultPath', path);
    lockOv.classList.remove('show');
    S.vaultOpen = true;
    S.entries = [];
    S.categories = [...DEFAULT_CATEGORIES];
    buildCatSelect();
    renderSidebarCats();
    renderSidebarStats();
    navigatePage('accounts');
    showSnack('金库已创建！');
  };

  setTimeout(() => pw1.focus(), 100);
}

/* ══════════════════════════════════════════════════════════════
   BOOT
══════════════════════════════════════════════════════════════ */
document.addEventListener('DOMContentLoaded', async () => {
  wireShell();
  entryFormHTML = document.getElementById('sheet-form')!.innerHTML;
  await window.vaultxAPI.settings.update(S.settings);
  // Restore view toggle icon
  const viewIcon = document.querySelector('#btn-view-toggle .material-icons-round');
  if (viewIcon) viewIcon.textContent = S.viewMode === 'grid' ? 'grid_view' : 'view_list';
  await initVault();
});
