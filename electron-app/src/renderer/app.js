'use strict';

/* ══════════════════════════════════════
   STATE
══════════════════════════════════════ */
const S = {
  page: 'accounts',     // accounts | groups | security | settings
  entries: [],
  filterCat: 'all',     // all | work | finance | personal
  searchQ: '',
  viewMode: 'grid',     // grid | list
  vaultPath: localStorage.getItem('vaultPath') || null,
  vaultOpen: false,
  settings: JSON.parse(localStorage.getItem('settings') || '{"autoLockTimeout":300,"clipboardClearSeconds":30,"maxErrorCount":5}'),
  editingEntry: null,
};

/* ══════════════════════════════════════
   THEME
══════════════════════════════════════ */
function getAppEl() { return document.getElementById('app'); }

function setTheme(t) {
  getAppEl().dataset.theme = t;
  localStorage.setItem('theme', t);
  document.querySelectorAll('.th-btn').forEach(b => b.classList.toggle('active', b.dataset.t === t));
}

(function initTheme() {
  const saved = localStorage.getItem('theme') || 'c';
  getAppEl().dataset.theme = saved;
})();

/* ══════════════════════════════════════
   ACTIVITY & LOCK LISTENER
══════════════════════════════════════ */
['mousemove', 'keydown', 'click', 'scroll'].forEach(ev =>
  document.addEventListener(ev, () => window.vaultxAPI.pingActivity(), { passive: true }));

window.vaultxAPI.onVaultLocked(() => {
  S.vaultOpen = false;
  S.entries = [];
  renderLock();
  showSnack('金库已自动锁定');
});

/* ══════════════════════════════════════
   SNACKBAR
══════════════════════════════════════ */
let snackTimer = null;
function showSnack(msg, action, onAction) {
  const snack = document.getElementById('snack');
  if (!snack) return;
  snack.querySelector('.snack-msg').textContent = msg;
  const act = snack.querySelector('.snack-act');
  if (action && onAction) { act.textContent = action; act.style.display = ''; act.onclick = onAction; }
  else { act.style.display = 'none'; }
  snack.classList.add('show');
  if (snackTimer) clearTimeout(snackTimer);
  snackTimer = setTimeout(() => snack.classList.remove('show'), 2800);
}

/* ══════════════════════════════════════
   CLIPBOARD
══════════════════════════════════════ */
async function copyText(text, feedbackEl) {
  await window.vaultxAPI.clipboard.write(text);
  showSnack('已复制，将在 30 秒后自动清除剪贴板');
  if (feedbackEl) {
    feedbackEl.classList.add('copied');
    setTimeout(() => feedbackEl.classList.remove('copied'), 1500);
  }
}

/* ══════════════════════════════════════
   FULL SHELL (only built once)
══════════════════════════════════════ */
function buildShell() {
  const app = getAppEl();
  app.innerHTML = `
  <!-- titlebar -->
  <div class="titlebar">
    <div class="traffic"><span class="t-r"></span><span class="t-y"></span><span class="t-g"></span></div>
    <span class="titlebar-title">VaultX</span>
    <div class="titlebar-right">
      <button class="icon-btn round" id="tb-lock" title="锁定"><span class="material-icons-round">lock</span></button>
    </div>
  </div>

  <!-- app body -->
  <div class="app-body">
    <!-- sidebar -->
    <aside class="sidebar">
      <div class="sidebar-nav">
        <button class="snav-item active" data-page="accounts">
          <span class="material-icons-round">shield</span>账户
        </button>
        <button class="snav-item" data-page="groups">
          <span class="material-icons-round">folder</span>分组
        </button>
        <button class="snav-item" data-page="security">
          <span class="material-icons-round">security</span>安全
        </button>
        <button class="snav-item" data-page="settings">
          <span class="material-icons-round">settings</span>设置
        </button>
      </div>
      <div class="sidebar-divider"></div>
      <div class="sidebar-sec-label">分类</div>
      <div class="sidebar-cats" id="sidebar-cats"></div>
      <div class="sidebar-spacer"></div>
      <div class="sidebar-stats" id="sidebar-stats"></div>
      <div class="sidebar-bottom">
        <button class="sidebar-add-btn" id="btn-add-entry">
          <span class="material-icons-round">add</span>添加账户
        </button>
      </div>
    </aside>

    <!-- main content -->
    <div class="main-content">
      <div class="content-toolbar" id="content-toolbar">
        <div class="content-title-group">
          <span class="content-title" id="ct-title">账户</span>
          <span class="content-sub" id="ct-sub"></span>
        </div>
        <div class="toolbar-search" id="toolbar-search">
          <span class="material-icons-round">search</span>
          <input type="text" id="search-input" placeholder="搜索账户…">
        </div>
        <button class="icon-btn" id="btn-view-toggle" title="切换视图"><span class="material-icons-round">grid_view</span></button>
        <button class="icon-btn" id="btn-import" title="导入"><span class="material-icons-round">upload</span></button>
        <button class="icon-btn" id="btn-export" title="导出"><span class="material-icons-round">download</span></button>
      </div>

      <!-- pages -->
      <div class="d-page active" id="d-page-accounts"></div>
      <div class="d-page" id="d-page-groups"></div>
      <div class="d-page" id="d-page-security"></div>
      <div class="d-page" id="d-page-settings"></div>
    </div>
  </div>

  <!-- modal sheet (add/edit) -->
  <div class="scrim" id="scrim">
    <div class="sheet" id="sheet"></div>
  </div>

  <!-- confirm dialog -->
  <div id="confirm-overlay" style="display:none" class="modal-overlay">
    <div class="modal">
      <div class="modal-title" id="confirm-title">确认</div>
      <div class="modal-msg" id="confirm-msg"></div>
      <div class="modal-actions">
        <button class="modal-btn modal-btn-cancel" id="confirm-cancel">取消</button>
        <button class="modal-btn modal-btn-danger" id="confirm-ok">确认</button>
      </div>
    </div>
  </div>

  <!-- lock overlay -->
  <div class="lock-ov" id="lock-ov"></div>

  <!-- snackbar -->
  <div class="snack" id="snack">
    <span class="snack-msg"></span>
    <button class="snack-act" style="display:none"></button>
  </div>`;

  document.getElementById('tb-lock').onclick = () => lockVault();

  document.querySelectorAll('.snav-item[data-page]').forEach(btn => {
    btn.onclick = () => navigatePage(btn.dataset.page);
  });

  document.getElementById('btn-add-entry').onclick = () => openAddSheet();
  document.getElementById('btn-view-toggle').onclick = () => toggleViewMode();
  document.getElementById('btn-import').onclick = () => importVault();
  document.getElementById('btn-export').onclick = () => exportVault();
  document.getElementById('search-input').oninput = (e) => {
    S.searchQ = e.target.value.toLowerCase();
    renderAccountsPage();
  };
  document.getElementById('scrim').onclick = (e) => {
    if (e.target.id === 'scrim') closeSheet();
  };
}

/* ══════════════════════════════════════
   NAVIGATION
══════════════════════════════════════ */
function navigatePage(page) {
  S.page = page;
  document.querySelectorAll('.snav-item[data-page]').forEach(b =>
    b.classList.toggle('active', b.dataset.page === page));
  ['accounts','groups','security','settings'].forEach(p => {
    const el = document.getElementById('d-page-' + p);
    if (el) el.classList.toggle('active', p === page);
  });

  const toolbarSearch = document.getElementById('toolbar-search');
  const btnViewToggle = document.getElementById('btn-view-toggle');
  const ctTitle = document.getElementById('ct-title');
  const ctSub = document.getElementById('ct-sub');
  const btnImport = document.getElementById('btn-import');
  const btnExport = document.getElementById('btn-export');

  const pageNames = { accounts: '账户', groups: '分组', security: '安全', settings: '设置' };
  ctTitle.textContent = pageNames[page] || page;
  ctSub.textContent = '';

  const isAccounts = page === 'accounts';
  toolbarSearch.style.display = isAccounts ? '' : 'none';
  btnViewToggle.style.display = isAccounts ? '' : 'none';
  btnImport.style.display = isAccounts ? '' : 'none';
  btnExport.style.display = isAccounts ? '' : 'none';

  if (isAccounts) renderAccountsPage();
  else if (page === 'groups') renderGroupsPage();
  else if (page === 'security') renderSecurityPage();
  else if (page === 'settings') renderSettingsPage();

  renderSidebarCats();
  renderSidebarStats();
}

/* ══════════════════════════════════════
   SIDEBAR
══════════════════════════════════════ */
const CAT_COLORS = { work: 'var(--cat-work)', finance: 'var(--cat-finance)', personal: 'var(--cat-personal)' };
const CAT_LABELS = { all: '全部', work: '工作', finance: '财务', personal: '个人' };

function renderSidebarCats() {
  const el = document.getElementById('sidebar-cats');
  if (!el) return;
  const cats = ['all', 'work', 'finance', 'personal'];
  el.innerHTML = cats.map(cat => {
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

function normalizeCategory(cat) {
  if (!cat) return 'personal';
  const c = cat.toLowerCase();
  if (c === 'work' || c === '工作') return 'work';
  if (c === 'finance' || c === '财务' || c === 'financial') return 'finance';
  return 'personal';
}

/* ══════════════════════════════════════
   ACCOUNTS PAGE
══════════════════════════════════════ */
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
  const groups = { work: [], finance: [], personal: [] };
  list.forEach(e => { const c = normalizeCategory(e.category); (groups[c] = groups[c] || []).push(e); });

  if (list.length === 0) {
    page.innerHTML = `<div class="empty-state">
      <span class="material-icons-round">lock_open</span>
      <p>${S.searchQ ? '没有匹配的账户' : '还没有账户，点击"添加账户"开始'}</p>
    </div>`;
    return;
  }

  let html = '<div class="account-list">';
  ['work','finance','personal'].forEach(cat => {
    const items = groups[cat];
    if (!items || items.length === 0) return;
    html += `<div class="sec-hd">
      <span class="sec-dot" style="background:${CAT_COLORS[cat]}"></span>
      <span class="sec-label">${CAT_LABELS[cat]}</span>
      <span class="sec-line"></span>
      <span class="sec-count">${items.length}</span>
    </div>
    <div class="cards-grid${S.viewMode === 'list' ? ' list-view' : ''}">
      ${items.map(e => buildCardHTML(e)).join('')}
    </div>`;
  });
  html += '</div>';
  page.innerHTML = html;

  page.querySelectorAll('.card').forEach(card => {
    const entry = S.entries.find(e => String(e.id) === card.dataset.id);
    if (entry) wireCard(card, entry);
  });

  const ctSub = document.getElementById('ct-sub');
  if (ctSub) ctSub.textContent = `${list.length} 个账户`;
}

function buildCardHTML(entry) {
  const cat = normalizeCategory(entry.category);
  const type = entry.type || 'totp';
  const icon = entry.icon || '🔐';
  const issuer = entry.issuer || entry.label || '未知';
  const label = (entry.label && entry.label !== entry.issuer) ? entry.label : (entry.username || '');
  const CIRC = 2 * Math.PI * 16;

  let r2Html = '';
  if (type === 'totp') {
    r2Html = `
      <span class="otp masked">• • • • • •</span>
      <span class="reveal-hint"><span class="material-icons-round">visibility</span>点击显示</span>
      <button class="copy-chip"><span class="material-icons-round">content_copy</span>复制</button>
      <div class="timer-wrap" style="display:none">
        <svg class="timer-svg" width="38" height="38" viewBox="0 0 38 38">
          <circle class="t-bg" cx="19" cy="19" r="16"/>
          <circle class="t-ring" cx="19" cy="19" r="16"
            stroke-dasharray="${CIRC.toFixed(2)}" stroke-dashoffset="${CIRC.toFixed(2)}" stroke="var(--primary)"/>
        </svg>
        <div class="t-num" style="color:var(--primary)">30</div>
      </div>`;
  } else {
    r2Html = `
      <div class="card-r2-pw">
        <span class="pw-display">••••••••</span>
        <button class="pw-toggle"><span class="material-icons-round">visibility</span></button>
        <button class="copy-chip"><span class="material-icons-round">content_copy</span>复制</button>
      </div>`;
  }

  return `<div class="card" data-id="${entry.id}" data-cat="${cat}">
    <div class="card-in">
      <div class="card-r1">
        <div class="avatar">${icon}</div>
        <div class="card-txt">
          <div class="card-issuer">${escHtml(issuer)}</div>
          <div class="card-label">${escHtml(label)}</div>
        </div>
        <button class="menu-btn"><span class="material-icons-round">more_vert</span></button>
        <div class="ctx-menu">
          <button class="ctx-item" data-action="edit"><span class="material-icons-round">edit</span>编辑</button>
          <div class="ctx-divider"></div>
          <button class="ctx-item danger" data-action="delete"><span class="material-icons-round">delete</span>删除</button>
        </div>
      </div>
      <div class="card-r2">${r2Html}</div>
      <div class="card-r3">
        <span class="cred-label">用户</span>
        <span class="cred-val">${escHtml(entry.username || '')}</span>
      </div>
    </div>
  </div>`;
}

/* ══════════════════════════════════════
   CARD WIRING
══════════════════════════════════════ */
const cardTimers = new Map();

function wireCard(card, entry) {
  const type = entry.type || 'totp';
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
    card.onclick = (e) => {
      if (e.target.closest('.menu-btn,.ctx-menu,.copy-chip')) return;
      if (!card.classList.contains('revealed')) revealTOTP(card, entry);
    };
    card.querySelector('.copy-chip').onclick = (e) => {
      e.stopPropagation();
      const code = card.querySelector('.otp').textContent.replace(/\s/g,'');
      copyText(code, card.querySelector('.copy-chip'));
    };
  } else {
    const pwDisplay = card.querySelector('.pw-display');
    const pwToggle = card.querySelector('.pw-toggle');
    const copyChip = card.querySelector('.copy-chip');
    let pwVisible = false;
    card.onclick = (e) => {
      if (e.target.closest('.menu-btn,.ctx-menu,.copy-chip,.pw-toggle')) return;
      card.classList.toggle('revealed');
    };
    pwToggle.onclick = (e) => {
      e.stopPropagation();
      pwVisible = !pwVisible;
      pwDisplay.textContent = pwVisible ? (entry.password || '') : '••••••••';
      pwDisplay.classList.toggle('revealed-pw', pwVisible);
      pwToggle.querySelector('.material-icons-round').textContent = pwVisible ? 'visibility_off' : 'visibility';
    };
    copyChip.onclick = (e) => { e.stopPropagation(); copyText(entry.password || '', copyChip); };
  }
}

function closeAllMenus() {
  document.querySelectorAll('.ctx-menu.open').forEach(m => m.classList.remove('open'));
}
document.addEventListener('click', () => closeAllMenus());

const CIRC_VAL = 2 * Math.PI * 16;

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
      otpEl.textContent = d === 6 ? r.code.slice(0,3) + ' ' + r.code.slice(3) : r.code;
      otpEl.classList.remove('masked','warn','danger');
      if (r.remaining <= 5) otpEl.classList.add('danger');
      else if (r.remaining <= 10) otpEl.classList.add('warn');
      const p = conf.period || 30;
      ringEl.style.strokeDashoffset = (CIRC_VAL * (1 - r.remaining / p)).toFixed(2);
      const col = r.remaining <= 5 ? 'var(--danger)' : r.remaining <= 10 ? 'var(--warn)' : 'var(--primary)';
      ringEl.style.stroke = col; numEl.style.color = col;
      numEl.textContent = r.remaining;
    } catch(e) { otpEl.textContent = '错误'; }
  }

  update();
  const id = entry.id;
  if (cardTimers.has(id)) clearInterval(cardTimers.get(id));
  cardTimers.set(id, setInterval(update, 1000));
}

/* ══════════════════════════════════════
   VIEW TOGGLE
══════════════════════════════════════ */
function toggleViewMode() {
  S.viewMode = S.viewMode === 'grid' ? 'list' : 'grid';
  const icon = document.querySelector('#btn-view-toggle .material-icons-round');
  if (icon) icon.textContent = S.viewMode === 'grid' ? 'grid_view' : 'view_list';
  if (S.page === 'accounts') renderAccountsPage();
}

/* ══════════════════════════════════════
   ADD / EDIT SHEET
══════════════════════════════════════ */
const ICONS = ['🔐','🏦','📧','🐙','🍎','🤖','🎮','🛒','💼','🏠','💳','🌐'];
const CATS_LIST = [{ v:'work', l:'工作' },{ v:'finance', l:'财务' },{ v:'personal', l:'个人' }];

function openAddSheet() { S.editingEntry = null; openSheet(null); }
function openEditSheet(entry) { S.editingEntry = entry; openSheet(entry); }

function openSheet(entry) {
  const isEdit = !!entry;
  const type = isEdit ? (entry.type || 'totp') : 'totp';
  document.getElementById('sheet').innerHTML = buildSheetHTML(isEdit, entry, type);
  document.getElementById('scrim').classList.add('open');
  wireSheet(isEdit, entry);
}

function buildSheetHTML(isEdit, entry, type) {
  const curIcon = entry ? (entry.icon || '🔐') : '🔐';
  const curCat = entry ? normalizeCategory(entry.category) : 'personal';
  const CIRC = 2 * Math.PI * 16;
  return `
  <div class="sheet-top">
    <span class="sheet-title">${isEdit ? '编辑账户' : '添加账户'}</span>
    <button class="sheet-close" id="sheet-close"><span class="material-icons-round">close</span></button>
  </div>
  <div class="form">
    <div class="form-label-sm">图标</div>
    <div class="icon-picker-row" id="icon-picker">
      ${ICONS.map(ic => `<div class="icon-opt${ic===curIcon?' picked':''}" data-icon="${ic}">${ic}</div>`).join('')}
    </div>
    <input type="hidden" id="f-icon" value="${curIcon}">
    <div class="type-toggle">
      <button class="type-btn${type==='totp'?' active':''}" data-t="totp"><span class="material-icons-round">lock_clock</span>TOTP</button>
      <button class="type-btn${type==='password'?' active':''}" data-t="password"><span class="material-icons-round">key</span>密码</button>
    </div>
    <input type="hidden" id="f-type" value="${type}">
    <div class="row2">
      <div class="field"><input type="text" id="f-issuer" placeholder=" " value="${escHtml(entry?entry.issuer||'':'')}"><label>服务名称 *</label></div>
      <div class="field"><input type="text" id="f-label" placeholder=" " value="${escHtml(entry?entry.label||'':'')}"><label>标签</label></div>
    </div>
    <div class="field"><input type="text" id="f-username" placeholder=" " value="${escHtml(entry ? entry.username || '' : '')}"><label>用户名 / 邮箱</label></div>
    <div class="form-totp-only${type==='password'?' hidden':''}">
      <div class="field"><input type="text" id="f-secret" placeholder=" " value="${escHtml(entry && entry.totp ? entry.totp.secret || '' : entry ? entry.secret || '' : '')}"><label>密钥 (Base32) *</label></div>
      <div class="row2">
        <div class="field"><select id="f-algo">${['SHA1','SHA256','SHA512'].map(a=>`<option${entry&&entry.totp&&entry.totp.algorithm===a?' selected':''}>${a}</option>`).join('')}</select></div>
        <div class="field"><select id="f-period">${[30,60].map(p=>`<option value="${p}"${entry&&entry.totp&&entry.totp.period===p?' selected':''}>${p}s</option>`).join('')}</select></div>
      </div>
    </div>
    <div class="field-pw" id="pw-field" style="${type==='totp'?'display:none':''}">
      <input type="password" id="f-password" placeholder=" " value="${escHtml(entry&&entry.password?entry.password:'')}">
      <label>密码</label>
      <div class="pw-actions">
        <button type="button" class="pw-act" id="pw-eye"><span class="material-icons-round">visibility</span></button>
        <button type="button" class="pw-act" id="pw-gen" title="生成密码"><span class="material-icons-round">autorenew</span></button>
      </div>
    </div>
    <div class="field">
      <select id="f-cat">${CATS_LIST.map(c=>`<option value="${c.v}"${curCat===c.v?' selected':''}>${c.l}</option>`).join('')}</select>
    </div>
    <button class="btn-fill" id="btn-save-entry">
      <span class="material-icons-round">${isEdit?'save':'add'}</span>${isEdit?'保存更改':'添加账户'}
    </button>
  </div>`;
}

function wireSheet(isEdit, entry) {
  document.getElementById('sheet-close').onclick = closeSheet;
  document.querySelectorAll('.icon-opt').forEach(opt => {
    opt.onclick = () => {
      document.querySelectorAll('.icon-opt').forEach(o => o.classList.remove('picked'));
      opt.classList.add('picked');
      document.getElementById('f-icon').value = opt.dataset.icon;
    };
  });
  document.querySelectorAll('.type-btn').forEach(btn => {
    btn.onclick = () => {
      document.querySelectorAll('.type-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      document.getElementById('f-type').value = btn.dataset.t;
      const isTOTP = btn.dataset.t === 'totp';
      document.querySelector('.form-totp-only').classList.toggle('hidden', !isTOTP);
      document.getElementById('pw-field').style.display = isTOTP ? 'none' : '';
    };
  });
  const pwInput = document.getElementById('f-password');
  const pwEye = document.getElementById('pw-eye');
  if (pwEye) pwEye.onclick = () => {
    const show = pwInput.type === 'password';
    pwInput.type = show ? 'text' : 'password';
    pwEye.querySelector('.material-icons-round').textContent = show ? 'visibility_off' : 'visibility';
  };
  const pwGen = document.getElementById('pw-gen');
  if (pwGen) pwGen.onclick = async () => {
    const pw = await window.vaultxAPI.generator.generate({ length:20, upper:true, lower:true, numbers:true, symbols:true });
    if (pwInput) { pwInput.value = pw; pwInput.type = 'text'; }
  };
  document.getElementById('btn-save-entry').onclick = () => saveEntry(isEdit, entry);
}

async function saveEntry(isEdit, existing) {
  const type = document.getElementById('f-type').value;
  const issuer = document.getElementById('f-issuer').value.trim();
  const secret = document.getElementById('f-secret') ? document.getElementById('f-secret').value.trim() : '';
  if (!issuer) { showSnack('请填写服务名称'); return; }
  if (type === 'totp' && !secret) { showSnack('请填写 TOTP 密钥'); return; }
  const catMap = { work:'Work', finance:'Finance', personal:'Personal' };
  const entryData = {
    icon: document.getElementById('f-icon').value,
    issuer,
    label: document.getElementById('f-label').value.trim(),
    username: document.getElementById('f-username').value.trim(),
    category: catMap[document.getElementById('f-cat').value] || 'Personal',
    type,
  };
  if (type === 'totp') {
    entryData.totp = { secret, algorithm: document.getElementById('f-algo').value, digits: 6, period: parseInt(document.getElementById('f-period').value) };
  } else {
    entryData.password = document.getElementById('f-password').value;
  }
  try {
    if (isEdit && existing) { await window.vaultxAPI.vault.updateEntry(existing.id, entryData); showSnack('账户已更新'); }
    else { await window.vaultxAPI.vault.addEntry(entryData); showSnack('账户已添加'); }
    closeSheet();
    await refreshEntries();
  } catch(e) { showSnack('保存失败: ' + (e.message || e)); }
}

function closeSheet() { document.getElementById('scrim').classList.remove('open'); }

/* ══════════════════════════════════════
   DELETE
══════════════════════════════════════ */
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
  S.entries = await window.vaultxAPI.vault.getEntries();
  renderSidebarCats();
  renderSidebarStats();
  if (S.page === 'accounts') renderAccountsPage();
}

/* ══════════════════════════════════════
   GROUPS PAGE
══════════════════════════════════════ */
function renderGroupsPage() {
  const page = document.getElementById('d-page-groups');
  if (!page) return;
  const groups = [
    { key:'work', label:'工作', icon:'work', color:'var(--cat-work)', desc:'工作相关账户' },
    { key:'finance', label:'财务', icon:'account_balance', color:'var(--cat-finance)', desc:'银行、支付、理财' },
    { key:'personal', label:'个人', icon:'person', color:'var(--cat-personal)', desc:'个人与社交账户' },
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

/* ══════════════════════════════════════
   SECURITY PAGE
══════════════════════════════════════ */
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

/* ══════════════════════════════════════
   SETTINGS PAGE
══════════════════════════════════════ */
const THEME_META = [
  { t:'a', name:'紫罗兰', c1:'#FFFBFE', c2:'#6750A4' },
  { t:'b', name:'深蓝', c1:'#0D1220', c2:'#4F8EF7' },
  { t:'c', name:'碳灰', c1:'#1E2128', c2:'#8AB4F8' },
  { t:'d', name:'翠绿', c1:'#0D1B12', c2:'#00C58E' },
  { t:'e', name:'玫红', c1:'#1A0D14', c2:'#F06292' },
  { t:'f', name:'琥珀', c1:'#1A1408', c2:'#F5A623' },
  { t:'g', name:'日光', c1:'#FFFFFF', c2:'#0B57D0' },
];

function renderSettingsPage() {
  const page = document.getElementById('d-page-settings');
  if (!page) return;
  const cur = getAppEl().dataset.theme || 'c';
  const swatches = THEME_META.map(tm =>
    `<button class="th-btn${tm.t===cur?' active':''}" data-t="${tm.t}">
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
          <div class="s-ctrl"><select class="s-select" id="st-lock">${[1,5,10,30,60,0].map(m=>`<option value="${m*60}"${S.settings.autoLockTimeout===(m*60)?' selected':''}>${m===0?'从不':m<60?m+'分钟':'1小时'}</option>`).join('')}</select></div>
        </div>
        <div class="s-item no-tap">
          <div class="s-body"><div class="s-label">剪贴板清除</div></div>
          <div class="s-ctrl"><select class="s-select" id="st-clip">${[15,30,60,120].map(s=>`<option value="${s}"${S.settings.clipboardClearSeconds===s?' selected':''}>${s}秒</option>`).join('')}</select></div>
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

/* ══════════════════════════════════════
   IMPORT / EXPORT
══════════════════════════════════════ */
async function importVault() {
  const path = await window.vaultxAPI.dialog.openFile({ filters:[{name:'VaultX',extensions:['vaultx','json']}] });
  if (path) showSnack('导入功能待实现');
}
async function exportVault() {
  const path = await window.vaultxAPI.dialog.saveFile({ defaultPath:'vaultx-backup.json', filters:[{name:'JSON',extensions:['json']}] });
  if (path) showSnack('导出功能待实现');
}

/* ══════════════════════════════════════
   LOCK / UNLOCK
══════════════════════════════════════ */
async function lockVault() {
  await window.vaultxAPI.vault.lock();
  S.vaultOpen = false; S.entries = [];
  renderLock();
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
        <span style="flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${escHtml(S.vaultPath||'未知路径')}</span>
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
    if (!pw) { errEl.textContent='请输入密码'; errEl.classList.add('show'); return; }
    submitBtn.disabled = true; submitBtn.textContent = '解锁中…';
    try {
      const openR = await window.vaultxAPI.vault.open(S.vaultPath);
      if (!openR.ok) { errEl.textContent = '无法打开金库: ' + openR.error; errEl.classList.add('show'); submitBtn.disabled=false; submitBtn.textContent='解锁'; return; }
      const unlockR = await window.vaultxAPI.vault.unlock(pw);
      if (unlockR.ok) {
        lockOv.classList.remove('show');
        S.vaultOpen = true;
        const ge = await window.vaultxAPI.vault.getEntries();
        S.entries = ge.ok ? ge.entries : [];
        renderSidebarCats(); renderSidebarStats();
        navigatePage('accounts');
      } else {
        errEl.textContent = '密码错误，请重试'; errEl.classList.add('show');
        pwInput.value = ''; pwInput.focus();
        submitBtn.disabled = false; submitBtn.textContent = '解锁';
      }
    } catch(e) {
      errEl.textContent = '解锁失败: ' + (e.message || e); errEl.classList.add('show');
      submitBtn.disabled = false; submitBtn.textContent = '解锁';
    }
  };
  lockOv.querySelector('#lock-change').onclick = () => changeVault();
  setTimeout(() => pwInput.focus(), 100);
}

/* ══════════════════════════════════════
   VAULT INIT
══════════════════════════════════════ */
async function initVault() {
  if (!S.vaultPath) { renderCreateVault(); return; }
  const status = await window.vaultxAPI.vault.check(S.vaultPath);
  if (!status.ok || !status.exists) { renderCreateVault(); return; }
  renderLock();
}

async function changeVault() {
  const path = await window.vaultxAPI.dialog.openFile({ filters:[{name:'VaultX',extensions:['vaultx']}] });
  if (path) { S.vaultPath = path; localStorage.setItem('vaultPath', path); initVault(); }
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
  const pw1 = lockOv.querySelector('#create-pw'), pw2 = lockOv.querySelector('#create-pw2');
  const errEl = lockOv.querySelector('#create-err'), eye = lockOv.querySelector('#create-eye');
  eye.onclick = () => { const s = pw1.type==='password'; pw1.type = s?'text':'password'; eye.querySelector('.material-icons-round').textContent = s?'visibility_off':'visibility'; };
  lockOv.querySelector('#create-choose').onclick = async () => {
    const path = await window.vaultxAPI.dialog.saveFile({ defaultPath:'vault.vaultx', filters:[{name:'VaultX',extensions:['vaultx']}] });
    if (path) { S.vaultPath = path; localStorage.setItem('vaultPath', path); showSnack('位置: '+path); }
  };
  lockOv.querySelector('#create-submit').onclick = async () => {
    const p1=pw1.value, p2=pw2.value;
    if (!p1){errEl.textContent='请输入密码';errEl.classList.add('show');return;}
    if (p1!==p2){errEl.textContent='两次密码不一致';errEl.classList.add('show');return;}
    if (p1.length<8){errEl.textContent='密码至少 8 位';errEl.classList.add('show');return;}
    const path = S.vaultPath || (Date.now()+'.vaultx');
    const r = await window.vaultxAPI.vault.create(path, p1);
    if (!r.ok) { errEl.textContent='创建失败: '+(r.error||'未知错误'); errEl.classList.add('show'); return; }
    S.vaultPath = path; localStorage.setItem('vaultPath', path);
    lockOv.classList.remove('show');
    S.vaultOpen = true; S.entries = [];
    renderSidebarCats(); renderSidebarStats();
    navigatePage('accounts');
    showSnack('金库已创建！');
  };
  setTimeout(() => pw1.focus(), 100);
}

/* ══════════════════════════════════════
   UTILS
══════════════════════════════════════ */
function escHtml(str) {
  return String(str).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

/* ══════════════════════════════════════
   BOOT
══════════════════════════════════════ */
document.addEventListener('DOMContentLoaded', async () => {
  buildShell();
  await initVault();
});
