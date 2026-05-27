'use strict';

/* ══════════════════════════════════════════════════════════════
   CONSTANTS
══════════════════════════════════════════════════════════════ */
const CIRC = 2 * Math.PI * 16;
const ICONS = ['🔐', '🏦', '📧', '🐙', '🍎', '🤖', '🎮', '🛒', '💼', '🏠', '💳', '🌐'];
const CATS = ['all', 'work', 'finance', 'personal'];
const CAT_COLORS = { work: 'var(--cat-work)', finance: 'var(--cat-finance)', personal: 'var(--cat-personal)' };
const CAT_LABELS = { all: '全部', work: '工作', finance: '财务', personal: '个人' };
const CAT_MAP = { work: 'Work', finance: 'Finance', personal: 'Personal' };
const THEME_META = [
  { t: 'a', name: '紫罗兰', c1: '#FFFBFE', c2: '#6750A4' },
  { t: 'b', name: '深蓝',   c1: '#0D1220', c2: '#4F8EF7' },
  { t: 'c', name: '碳灰',   c1: '#1E2128', c2: '#8AB4F8' },
  { t: 'd', name: '翠绿',   c1: '#0D1B12', c2: '#00C58E' },
  { t: 'e', name: '玫红',   c1: '#1A0D14', c2: '#F06292' },
  { t: 'f', name: '琥珀',   c1: '#1A1408', c2: '#F5A623' },
  { t: 'g', name: '日光',   c1: '#FFFFFF', c2: '#0B57D0' },
];

/* ══════════════════════════════════════════════════════════════
   STATE
══════════════════════════════════════════════════════════════ */
const S = {
  page: 'accounts',
  entries: [],
  filterCat: 'all',
  searchQ: '',
  viewMode: 'grid',
  vaultPath: localStorage.getItem('vaultPath') || null,
  vaultOpen: false,
  settings: JSON.parse(localStorage.getItem('settings') || '{"autoLockTimeout":300,"clipboardClearSeconds":30,"maxErrorCount":5}'),
  editingEntry: null,
};

const cardTimers = new Map();

/* ══════════════════════════════════════════════════════════════
   UTILITIES
══════════════════════════════════════════════════════════════ */
function $(sel) { return document.querySelector(sel); }
function $$(sel) { return document.querySelectorAll(sel); }
function getAppEl() { return document.getElementById('app'); }

function escHtml(str) {
  return String(str).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function normalizeCategory(cat) {
  if (!cat) return 'personal';
  const c = cat.toLowerCase();
  if (c === 'work' || c === '工作') return 'work';
  if (c === 'finance' || c === '财务' || c === 'financial') return 'finance';
  return 'personal';
}

/* ══════════════════════════════════════════════════════════════
   SNACKBAR & CLIPBOARD
══════════════════════════════════════════════════════════════ */
let snackTimer = null;

function showSnack(msg, action, onAction) {
  const snack = document.getElementById('snack');
  if (!snack) return;
  snack.querySelector('.snack-msg').textContent = msg;
  const act = snack.querySelector('.snack-act');
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

async function copyText(text, feedbackEl) {
  await window.vaultxAPI.clipboard.write(text);
  showSnack('已复制，将在 30 秒后自动清除剪贴板');
  if (feedbackEl) {
    feedbackEl.classList.add('copied');
    setTimeout(() => feedbackEl.classList.remove('copied'), 1500);
  }
}

/* ══════════════════════════════════════════════════════════════
   THEME
══════════════════════════════════════════════════════════════ */
function setTheme(t) {
  getAppEl().dataset.theme = t;
  localStorage.setItem('theme', t);
  $$('.th-btn').forEach(b => b.classList.toggle('active', b.dataset.t === t));
}

(function initTheme() {
  getAppEl().dataset.theme = localStorage.getItem('theme') || 'c';
})();

/* ══════════════════════════════════════════════════════════════
   ACTIVITY & AUTO-LOCK LISTENER
══════════════════════════════════════════════════════════════ */
['mousemove', 'keydown', 'click', 'scroll'].forEach(ev =>
  document.addEventListener(ev, () => window.vaultxAPI.pingActivity(), { passive: true }));

window.vaultxAPI.onVaultLocked(() => {
  S.vaultOpen = false;
  S.entries = [];
  renderLock();
  showSnack('金库已自动锁定');
});

/* ══════════════════════════════════════════════════════════════
   WIRE SHELL (bind events to existing DOM)
══════════════════════════════════════════════════════════════ */
function wireShell() {
  // Navigation
  $$('.snav-item[data-page]').forEach(btn => {
    btn.onclick = () => navigatePage(btn.dataset.page);
  });

  // Toolbar
  document.getElementById('tb-lock').onclick = () => lockVault();
  document.getElementById('btn-add-entry').onclick = () => openAddSheet();
  document.getElementById('btn-view-toggle').onclick = () => toggleViewMode();
  document.getElementById('btn-import').onclick = () => importVault();
  document.getElementById('btn-export').onclick = () => exportVault();
  document.getElementById('search-input').oninput = (e) => {
    S.searchQ = e.target.value.toLowerCase();
    filterCardsBySearch();
  };

  // Sheet
  document.getElementById('sheet-close').onclick = closeSheet;
  document.getElementById('scrim').onclick = (e) => {
    if (e.target.id === 'scrim') closeSheet();
  };

  // Icon picker
  const picker = document.getElementById('icon-picker');
  ICONS.forEach(ic => {
    const opt = document.createElement('div');
    opt.className = 'icon-opt' + (ic === '🔐' ? ' picked' : '');
    opt.dataset.icon = ic;
    opt.textContent = ic;
    opt.onclick = () => {
      $$('.icon-opt').forEach(o => o.classList.remove('picked'));
      opt.classList.add('picked');
      document.getElementById('f-icon').value = ic;
    };
    picker.appendChild(opt);
  });

  // Type toggle
  $$('.type-btn').forEach(btn => {
    btn.onclick = () => {
      $$('.type-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      document.getElementById('f-type').value = btn.dataset.t;
      document.getElementById('form-totp-only').classList.toggle('hidden', btn.dataset.t === 'password');
    };
  });

  // Password eye
  const pwInput = document.getElementById('f-password');
  const pwEye = document.getElementById('pw-eye');
  pwEye.onclick = () => {
    const show = pwInput.type === 'password';
    pwInput.type = show ? 'text' : 'password';
    pwEye.querySelector('.material-icons-round').textContent = show ? 'visibility_off' : 'visibility';
  };

  // Password generator
  document.getElementById('pw-gen').onclick = async () => {
    const pw = await window.vaultxAPI.generator.generate({ length: 20, upper: true, lower: true, numbers: true, symbols: true });
    pwInput.value = pw;
    pwInput.type = 'text';
  };

  // Save
  document.getElementById('btn-save-entry').onclick = () => saveEntry();

  // Close all context menus on background click
  document.addEventListener('click', closeAllMenus);
}

/* ══════════════════════════════════════════════════════════════
   NAVIGATION
══════════════════════════════════════════════════════════════ */
function navigatePage(page) {
  S.page = page;

  $$('.snav-item[data-page]').forEach(b =>
    b.classList.toggle('active', b.dataset.page === page));
  ['accounts', 'groups', 'security', 'settings'].forEach(p => {
    const el = document.getElementById('d-page-' + p);
    if (el) el.classList.toggle('active', p === page);
  });

  const pageNames = { accounts: '账户', groups: '分组', security: '安全', settings: '设置' };
  document.getElementById('ct-title').textContent = pageNames[page] || page;
  document.getElementById('ct-sub').textContent = '';

  const isAccounts = page === 'accounts';
  document.getElementById('toolbar-search').style.display = isAccounts ? '' : 'none';
  document.getElementById('btn-view-toggle').style.display = isAccounts ? '' : 'none';
  document.getElementById('btn-import').style.display = isAccounts ? '' : 'none';
  document.getElementById('btn-export').style.display = isAccounts ? '' : 'none';

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
function renderSidebarCats() {
  const el = document.getElementById('sidebar-cats');
  if (!el) return;
  el.innerHTML = CATS.map(cat => {
    const count = cat === 'all' ? S.entries.length : S.entries.filter(e => normalizeCategory(e.category) === cat).length;
    const dotStyle = cat === 'all' ? 'background:var(--on-surface-v)' : `background:${CAT_COLORS[cat]}`;
    return `<button class="scat-item${S.filterCat === cat ? ' active' : ''}" data-cat="${cat}">
      <span class="scat-dot" style="${dotStyle}"></span>
      <span class="scat-name">${CAT_LABELS[cat]}</span>
      <span class="scat-count">${count}</span>
    </button>`;
  }).join('');
  el.querySelectorAll('.scat-item').forEach(btn => {
    btn.onclick = () => { S.filterCat = btn.dataset.cat; renderSidebarCats(); renderAccountsPage(); };
  });
}

function renderSidebarStats() {
  const el = document.getElementById('sidebar-stats');
  if (!el) return;
  const totp = S.entries.filter(e => (e.type || 'totp') === 'totp').length;
  const pw = S.entries.filter(e => e.type === 'password').length;
  el.textContent = `${S.entries.length} 个账户 · ${totp} TOTP · ${pw} 密码`;
}

/* ══════════════════════════════════════════════════════════════
   ACCOUNTS PAGE
══════════════════════════════════════════════════════════════ */
function getFilteredEntries() {
  let list = S.entries;
  if (S.filterCat !== 'all') list = list.filter(e => normalizeCategory(e.category) === S.filterCat);
  if (S.searchQ) list = list.filter(e =>
    (e.issuer || '').toLowerCase().includes(S.searchQ) ||
    (e.label || '').toLowerCase().includes(S.searchQ) ||
    (e.username || '').toLowerCase().includes(S.searchQ));
  return list;
}

function renderAccountsPage() {
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

  const groups = { work: [], finance: [], personal: [] };
  list.forEach(e => groups[normalizeCategory(e.category)].push(e));

  const container = document.createElement('div');
  container.className = 'account-list';

  ['work', 'finance', 'personal'].forEach(cat => {
    const items = groups[cat];
    if (!items.length) return;

    // Section header
    const sec = document.createElement('div');
    sec.className = 'sec-hd';
    sec.innerHTML = `<span class="sec-dot" style="background:${CAT_COLORS[cat]}"></span>
      <span class="sec-label">${CAT_LABELS[cat]}</span>
      <span class="sec-line"></span>
      <span class="sec-count">${items.length}</span>`;
    container.appendChild(sec);

    // Cards grid
    const grid = document.createElement('div');
    grid.className = 'cards-grid' + (S.viewMode === 'list' ? ' list-view' : '');
    items.forEach(entry => grid.appendChild(buildCard(entry)));
    container.appendChild(grid);
  });

  page.innerHTML = '';
  page.appendChild(container);

  if (S.searchQ) filterCardsBySearch();
  else document.getElementById('ct-sub').textContent = `${list.length} 个账户`;
}

function filterCardsBySearch() {
  const q = S.searchQ;
  const page = document.getElementById('d-page-accounts');
  if (!page) return;
  let visibleCount = 0;

  page.querySelectorAll('.card').forEach(card => {
    const entry = S.entries.find(e => String(e.id) === card.dataset.id);
    if (!entry) return;
    const match = !q ||
      (entry.issuer || '').toLowerCase().includes(q) ||
      (entry.label || '').toLowerCase().includes(q) ||
      (entry.username || '').toLowerCase().includes(q);
    card.style.display = match ? '' : 'none';
    if (match) visibleCount++;
  });

  page.querySelectorAll('.sec-hd').forEach(sec => {
    const grid = sec.nextElementSibling;
    if (!grid) return;
    const hasVisible = Array.from(grid.querySelectorAll('.card')).some(c => c.style.display !== 'none');
    sec.style.display = hasVisible ? '' : 'none';
    grid.style.display = hasVisible ? '' : 'none';
  });

  document.getElementById('ct-sub').textContent = q ? `${visibleCount} 个匹配` : `${S.entries.length} 个账户`;
}

function toggleViewMode() {
  S.viewMode = S.viewMode === 'grid' ? 'list' : 'grid';
  const icon = document.querySelector('#btn-view-toggle .material-icons-round');
  if (icon) icon.textContent = S.viewMode === 'grid' ? 'grid_view' : 'view_list';
  if (S.page === 'accounts') renderAccountsPage();
}

/* ══════════════════════════════════════════════════════════════
   CARD BUILDING (from <template>)
══════════════════════════════════════════════════════════════ */
function buildCard(entry) {
  const type = entry.type || 'totp';
  const tplId = type === 'totp' ? 'tpl-card-totp' : 'tpl-card-pw';
  const frag = document.getElementById(tplId).content.cloneNode(true);
  const card = frag.querySelector('.card');

  // Common fields
  card.dataset.id = entry.id;
  card.dataset.cat = normalizeCategory(entry.category);
  card.querySelector('.avatar').textContent = entry.icon || '🔐';
  card.querySelector('.card-issuer').textContent = entry.issuer || entry.label || '未知';
  const label = (entry.label && entry.label !== entry.issuer) ? entry.label : (entry.username || '');
  card.querySelector('.card-label').textContent = label;

  // Row 3: username & password
  buildCardR3(card, entry);

  // Wire interactions
  wireCard(card, entry);

  return card;
}

function buildCardR3(card, entry) {
  const r3 = card.querySelector('.card-r3');
  if (!entry.username && !entry.password) {
    r3.remove();
    return;
  }

  let html = '';
  if (entry.username) {
    html += `<span class="cred-label">用户</span><span class="cred-val">${escHtml(entry.username)}</span>`;
  }
  if (entry.password) {
    html += `<span class="cred-label" style="${entry.username ? 'margin-left:8px' : ''}">密码</span>
      <span class="cred-val masked-pw r3-pw">••••••••</span>
      <button class="pw-toggle r3-pw-toggle"><span class="material-icons-round">visibility</span></button>
      <button class="copy-chip r3-pw-copy" style="opacity:1;transform:scale(1)"><span class="material-icons-round">content_copy</span>复制</button>`;
  }
  r3.innerHTML = html;
}

/* ══════════════════════════════════════════════════════════════
   CARD WIRING
══════════════════════════════════════════════════════════════ */
function wireCard(card, entry) {
  const type = entry.type || 'totp';

  // Context menu
  const menuBtn = card.querySelector('.menu-btn');
  const ctxMenu = card.querySelector('.ctx-menu');
  menuBtn.onclick = (e) => {
    e.stopPropagation();
    const wasOpen = ctxMenu.classList.contains('open');
    closeAllMenus();
    if (!wasOpen) ctxMenu.classList.add('open');
  };
  card.querySelector('[data-action="edit"]').onclick = (e) => { e.stopPropagation(); closeAllMenus(); openEditSheet(entry); };
  card.querySelector('[data-action="delete"]').onclick = (e) => { e.stopPropagation(); closeAllMenus(); confirmDelete(entry); };

  if (type === 'totp') {
    wireTotpCard(card, entry);
  } else {
    wirePasswordCard(card, entry);
  }
}

function wireTotpCard(card, entry) {
  card.onclick = (e) => {
    if (e.target.closest('.menu-btn,.ctx-menu,.copy-chip,.r3-pw-toggle,.r3-pw-copy')) return;
    if (!card.classList.contains('revealed')) revealTOTP(card, entry);
  };

  card.querySelector('.copy-chip').onclick = (e) => {
    e.stopPropagation();
    const code = card.querySelector('.otp').textContent.replace(/\s/g, '');
    copyText(code, card.querySelector('.copy-chip'));
  };

  wireR3Password(card, entry);
}

function wirePasswordCard(card, entry) {
  const pwDisplay = card.querySelector('.pw-display');
  const copyChip = card.querySelector('.copy-chip');
  let pwVisible = false;

  card.onclick = (e) => {
    if (e.target.closest('.menu-btn,.ctx-menu')) return;

    // Password toggle
    if (e.target.closest('.pw-toggle')) {
      pwVisible = !pwVisible;
      pwDisplay.textContent = pwVisible ? (entry.password || '') : '••••••••';
      pwDisplay.classList.toggle('revealed-pw', pwVisible);
      card.querySelector('.pw-toggle .material-icons-round').textContent = pwVisible ? 'visibility_off' : 'visibility';
      return;
    }

    // Copy
    if (e.target.closest('.copy-chip')) {
      copyText(entry.password || '', copyChip);
      return;
    }

    card.classList.toggle('revealed');
  };
}

function wireR3Password(card, entry) {
  const r3PwEl = card.querySelector('.r3-pw');
  const r3PwToggle = card.querySelector('.r3-pw-toggle');
  const r3PwCopy = card.querySelector('.r3-pw-copy');
  if (!r3PwEl || !entry.password) return;

  r3PwToggle.onclick = (e) => {
    e.stopPropagation();
    const visible = r3PwEl.classList.contains('revealed-pw');
    r3PwEl.textContent = visible ? '••••••••' : entry.password;
    r3PwEl.classList.toggle('masked-pw', visible);
    r3PwEl.classList.toggle('revealed-pw', !visible);
    r3PwToggle.querySelector('.material-icons-round').textContent = visible ? 'visibility' : 'visibility_off';
  };

  r3PwCopy.onclick = (e) => { e.stopPropagation(); copyText(entry.password, r3PwCopy); };
}

function closeAllMenus() {
  $$('.ctx-menu.open').forEach(m => m.classList.remove('open'));
}

/* ══════════════════════════════════════════════════════════════
   TOTP REVEAL & TIMER
══════════════════════════════════════════════════════════════ */
function revealTOTP(card, entry) {
  card.classList.add('revealed');
  const otpEl = card.querySelector('.otp');
  const ringEl = card.querySelector('.t-ring');
  const numEl = card.querySelector('.t-num');
  const timerWrap = card.querySelector('.timer-wrap');
  timerWrap.style.display = '';

  async function update() {
    const conf = entry.totp || { secret: entry.secret, algorithm: 'SHA1', digits: 6, period: 30 };
    try {
      const r = await window.TotpUtil.computeTotp(conf);
      const d = conf.digits || 6;
      otpEl.textContent = d === 6 ? r.code.slice(0, 3) + ' ' + r.code.slice(3) : r.code;
      otpEl.classList.remove('masked', 'warn', 'danger');
      if (r.remaining <= 5) otpEl.classList.add('danger');
      else if (r.remaining <= 10) otpEl.classList.add('warn');
      const p = conf.period || 30;
      ringEl.style.strokeDashoffset = (CIRC * (1 - r.remaining / p)).toFixed(2);
      const col = r.remaining <= 5 ? 'var(--danger)' : r.remaining <= 10 ? 'var(--warn)' : 'var(--primary)';
      ringEl.style.stroke = col;
      numEl.style.color = col;
      numEl.textContent = r.remaining;
    } catch (e) {
      otpEl.textContent = '错误';
    }
  }

  update();
  const id = entry.id;
  if (cardTimers.has(id)) clearInterval(cardTimers.get(id));
  cardTimers.set(id, setInterval(update, 1000));
}

/* ══════════════════════════════════════════════════════════════
   ADD / EDIT SHEET (populate existing DOM)
══════════════════════════════════════════════════════════════ */
function openAddSheet() {
  S.editingEntry = null;
  resetSheet();
  document.getElementById('scrim').classList.add('open');
}

function openEditSheet(entry) {
  S.editingEntry = entry;
  populateSheet(entry);
  document.getElementById('scrim').classList.add('open');
}

function closeSheet() {
  document.getElementById('scrim').classList.remove('open');
}

function resetSheet() {
  document.getElementById('sheet-title').textContent = '添加账户';
  document.getElementById('btn-save-entry').innerHTML = '<span class="material-icons-round">add</span>添加账户';
  document.getElementById('f-icon').value = '🔐';
  $$('.icon-opt').forEach((o, i) => o.classList.toggle('picked', i === 0));
  document.getElementById('f-type').value = 'totp';
  $$('.type-btn').forEach(b => b.classList.toggle('active', b.dataset.t === 'totp'));
  document.getElementById('form-totp-only').classList.remove('hidden');
  document.getElementById('f-issuer').value = '';
  document.getElementById('f-label').value = '';
  document.getElementById('f-username').value = '';
  document.getElementById('f-secret').value = '';
  document.getElementById('f-algo').value = 'SHA1';
  document.getElementById('f-period').value = '30';
  document.getElementById('f-password').value = '';
  document.getElementById('f-password').type = 'password';
  document.getElementById('pw-eye').querySelector('.material-icons-round').textContent = 'visibility';
  document.getElementById('f-cat').value = 'personal';
}

function populateSheet(entry) {
  resetSheet();
  const type = entry.type || 'totp';
  document.getElementById('sheet-title').textContent = '编辑账户';
  document.getElementById('btn-save-entry').innerHTML = '<span class="material-icons-round">save</span>保存更改';
  document.getElementById('f-icon').value = entry.icon || '🔐';
  $$('.icon-opt').forEach(o => o.classList.toggle('picked', o.dataset.icon === entry.icon));
  document.getElementById('f-type').value = type;
  $$('.type-btn').forEach(b => b.classList.toggle('active', b.dataset.t === type));
  document.getElementById('form-totp-only').classList.toggle('hidden', type === 'password');
  document.getElementById('f-issuer').value = entry.issuer || '';
  document.getElementById('f-label').value = entry.label || '';
  document.getElementById('f-username').value = entry.username || '';
  document.getElementById('f-password').value = entry.password || '';
  if (type === 'totp') {
    const totp = entry.totp || {};
    document.getElementById('f-secret').value = totp.secret || entry.secret || '';
    document.getElementById('f-algo').value = totp.algorithm || 'SHA1';
    document.getElementById('f-period').value = totp.period || 30;
  }
  document.getElementById('f-cat').value = normalizeCategory(entry.category);
}

async function saveEntry() {
  const isEdit = !!S.editingEntry;
  const existing = S.editingEntry;
  const type = document.getElementById('f-type').value;
  const issuer = document.getElementById('f-issuer').value.trim();
  const secret = document.getElementById('f-secret').value.trim();
  if (!issuer) { showSnack('请填写服务名称'); return; }
  if (type === 'totp' && !secret) { showSnack('请填写 TOTP 密钥'); return; }

  const pwValue = document.getElementById('f-password').value;
  const entryData = {
    id: isEdit && existing ? existing.id : crypto.randomUUID(),
    icon: document.getElementById('f-icon').value,
    issuer,
    title: issuer,
    label: document.getElementById('f-label').value.trim(),
    username: document.getElementById('f-username').value.trim(),
    category: CAT_MAP[document.getElementById('f-cat').value] || 'Personal',
    type,
  };

  if (type === 'totp') {
    entryData.totp = { secret, algorithm: document.getElementById('f-algo').value, digits: 6, period: parseInt(document.getElementById('f-period').value) };
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
  } catch (e) {
    showSnack('保存失败: ' + (e.message || e));
  }
}

/* ══════════════════════════════════════════════════════════════
   DELETE
══════════════════════════════════════════════════════════════ */
function confirmDelete(entry) {
  const overlay = document.getElementById('confirm-overlay');
  document.getElementById('confirm-title').textContent = '删除账户';
  document.getElementById('confirm-msg').textContent = `确定要删除"${entry.issuer || entry.label}"吗？`;
  overlay.style.display = 'flex';
  document.getElementById('confirm-cancel').onclick = () => { overlay.style.display = 'none'; };
  document.getElementById('confirm-ok').onclick = async () => {
    overlay.style.display = 'none';
    await window.vaultxAPI.vault.deleteEntry(entry.id);
    showSnack('账户已删除');
    await refreshEntries();
  };
}

async function refreshEntries() {
  const ge = await window.vaultxAPI.vault.getEntries();
  S.entries = ge.ok ? ge.entries : [];
  renderSidebarCats();
  renderSidebarStats();
  if (S.page === 'accounts') renderAccountsPage();
}

/* ══════════════════════════════════════════════════════════════
   GROUPS PAGE
══════════════════════════════════════════════════════════════ */
function renderGroupsPage() {
  const page = document.getElementById('d-page-groups');
  if (!page) return;
  const groups = [
    { key: 'work', label: '工作', icon: 'work', color: 'var(--cat-work)', desc: '工作相关账户' },
    { key: 'finance', label: '财务', icon: 'account_balance', color: 'var(--cat-finance)', desc: '银行、支付、理财' },
    { key: 'personal', label: '个人', icon: 'person', color: 'var(--cat-personal)', desc: '个人与社交账户' },
  ];
  page.innerHTML = '<div class="d-group-grid">' + groups.map(g => {
    const count = S.entries.filter(e => normalizeCategory(e.category) === g.key).length;
    return `<div class="d-group-card">
      <div class="d-group-card-top">
        <div class="d-group-icon" style="background:${g.color}"><span class="material-icons-round">${g.icon}</span></div>
        <div><div class="d-group-name">${g.label}</div><div class="d-group-desc">${g.desc}</div></div>
      </div>
      <div class="d-group-stats"><span class="d-group-stat">${count} 个账户</span></div>
      <div class="d-group-actions">
        <button class="d-group-btn" data-g="${g.key}"><span class="material-icons-round">visibility</span>查看</button>
      </div>
    </div>`;
  }).join('') + '</div>';
  page.querySelectorAll('.d-group-btn').forEach(b => {
    b.onclick = () => { S.filterCat = b.dataset.g; navigatePage('accounts'); };
  });
}

/* ══════════════════════════════════════════════════════════════
   SECURITY PAGE
══════════════════════════════════════════════════════════════ */
function renderSecurityPage() {
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
  page.querySelector('#sec-lock').onclick = () => lockVault();
}

/* ══════════════════════════════════════════════════════════════
   SETTINGS PAGE
══════════════════════════════════════════════════════════════ */
function renderSettingsPage() {
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

  page.querySelectorAll('.th-btn').forEach(b => { b.onclick = () => setTheme(b.dataset.t); });
  page.querySelector('#st-lock').onchange = (e) => { S.settings.autoLockTimeout = parseInt(e.target.value); saveSettings(); };
  page.querySelector('#st-clip').onchange = (e) => { S.settings.clipboardClearSeconds = parseInt(e.target.value); saveSettings(); };
  page.querySelector('#st-import').onclick = () => importVault();
  page.querySelector('#st-export').onclick = () => exportVault();
}

async function saveSettings() {
  localStorage.setItem('settings', JSON.stringify(S.settings));
  await window.vaultxAPI.settings.update(S.settings);
}

/* ══════════════════════════════════════════════════════════════
   IMPORT / EXPORT
══════════════════════════════════════════════════════════════ */
async function importVault() {
  const path = await window.vaultxAPI.dialog.openFile({ filters: [{ name: 'VaultX', extensions: ['vaultx', 'json'] }] });
  if (path) showSnack('导入功能待实现');
}

async function exportVault() {
  const path = await window.vaultxAPI.dialog.saveFile({ defaultPath: 'vaultx-backup.json', filters: [{ name: 'JSON', extensions: ['json'] }] });
  if (path) showSnack('导出功能待实现');
}

/* ══════════════════════════════════════════════════════════════
   VAULT: LOCK / UNLOCK / INIT / CREATE
══════════════════════════════════════════════════════════════ */
async function lockVault() {
  await window.vaultxAPI.vault.lock();
  S.vaultOpen = false;
  S.entries = [];
  renderLock();
}

async function initVault() {
  if (!S.vaultPath) { renderCreateVault(); return; }
  const status = await window.vaultxAPI.vault.check(S.vaultPath);
  if (!status.ok || !status.exists) { renderCreateVault(); return; }
  renderLock();
}

async function changeVault() {
  const path = await window.vaultxAPI.dialog.openFile({ filters: [{ name: 'VaultX', extensions: ['vaultx'] }] });
  if (path) {
    S.vaultPath = path;
    localStorage.setItem('vaultPath', path);
    initVault();
  }
}

function renderLock() {
  const lockOv = document.getElementById('lock-ov');
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

  const pwInput = lockOv.querySelector('#lock-pw');
  const eyeBtn = lockOv.querySelector('#lock-eye');
  const errEl = lockOv.querySelector('#lock-err');
  const submitBtn = lockOv.querySelector('#lock-submit');

  eyeBtn.onclick = () => {
    const show = pwInput.type === 'password';
    pwInput.type = show ? 'text' : 'password';
    eyeBtn.querySelector('.material-icons-round').textContent = show ? 'visibility_off' : 'visibility';
  };

  pwInput.onkeydown = (e) => { if (e.key === 'Enter') submitBtn.click(); };

  submitBtn.onclick = async () => {
    const pw = pwInput.value;
    if (!pw) { errEl.textContent = '请输入密码'; errEl.classList.add('show'); return; }
    submitBtn.disabled = true;
    submitBtn.textContent = '解锁中…';
    try {
      const openR = await window.vaultxAPI.vault.open(S.vaultPath);
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
        const ge = await window.vaultxAPI.vault.getEntries();
        S.entries = ge.ok ? ge.entries : [];
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
    } catch (e) {
      errEl.textContent = '解锁失败: ' + (e.message || e);
      errEl.classList.add('show');
      submitBtn.disabled = false;
      submitBtn.textContent = '解锁';
    }
  };

  lockOv.querySelector('#lock-change').onclick = () => changeVault();
  setTimeout(() => pwInput.focus(), 100);
}

function renderCreateVault() {
  const lockOv = document.getElementById('lock-ov');
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

  const pw1 = lockOv.querySelector('#create-pw');
  const pw2 = lockOv.querySelector('#create-pw2');
  const errEl = lockOv.querySelector('#create-err');
  const eye = lockOv.querySelector('#create-eye');

  eye.onclick = () => {
    const s = pw1.type === 'password';
    pw1.type = s ? 'text' : 'password';
    eye.querySelector('.material-icons-round').textContent = s ? 'visibility_off' : 'visibility';
  };

  lockOv.querySelector('#create-choose').onclick = async () => {
    const path = await window.vaultxAPI.dialog.saveFile({ defaultPath: 'vault.vaultx', filters: [{ name: 'VaultX', extensions: ['vaultx'] }] });
    if (path) {
      S.vaultPath = path;
      localStorage.setItem('vaultPath', path);
      showSnack('位置: ' + path);
    }
  };

  lockOv.querySelector('#create-submit').onclick = async () => {
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
  await initVault();
});
