'use strict';

const { app, BrowserWindow, ipcMain, dialog, clipboard, nativeTheme } = require('electron');
const path = require('path');
const VaultManager = require('./vault');
const { generatePassword, evaluatePasswordStrength } = require('./generator');

// ── 安全设置 ──────────────────────────────────────────────────────────────
process.env.ELECTRON_DISABLE_SECURITY_WARNINGS = 'true';

let mainWindow = null;
let vaultManager = new VaultManager();
let clipboardTimer = null;
let autoLockTimer = null;
let autoLockTimeout = 300; // 默认 5 分钟（秒）
let clipboardClearSeconds = 30; // 默认 30 秒

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1100,
    height: 720,
    minWidth: 800,
    minHeight: 550,
    backgroundColor: '#FAFAFA',
    titleBarStyle: 'default',
    webPreferences: {
      preload: path.join(__dirname, '../preload/index.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });

  mainWindow.loadFile(path.join(__dirname, '../renderer/index.html'));
  mainWindow.webContents.openDevTools();

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});

// ── 重置自动锁定计时器 ────────────────────────────────────────────────────
function resetAutoLockTimer() {
  if (autoLockTimer) clearTimeout(autoLockTimer);
  if (autoLockTimeout > 0 && vaultManager.isUnlocked()) {
    autoLockTimer = setTimeout(() => {
      vaultManager.lock();
      mainWindow?.webContents.send('vault:locked');
    }, autoLockTimeout * 1000);
  }
}

// ── IPC: 金库操作 ─────────────────────────────────────────────────────────

ipcMain.handle('vault:check', async (_event, filePath) => {
  try {
    const exists = require('fs').existsSync(filePath);
    return { ok: true, exists };
  } catch (e) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:open', async (_event, filePath) => {
  try {
    await vaultManager.open(filePath);
    return { ok: true };
  } catch (e) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:create', async (_event, filePath, password) => {
  try {
    await vaultManager.create(filePath, password);
    resetAutoLockTimer();
    return { ok: true };
  } catch (e) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:unlock', async (_event, password) => {
  try {
    await vaultManager.unlock(password);
    resetAutoLockTimer();
    return { ok: true };
  } catch (e) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:lock', async () => {
  vaultManager.lock();
  if (autoLockTimer) { clearTimeout(autoLockTimer); autoLockTimer = null; }
  return { ok: true };
});

ipcMain.handle('vault:getEntries', async () => {
  try {
    const entries = vaultManager.getEntries();
    resetAutoLockTimer();
    return { ok: true, entries };
  } catch (e) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:addEntry', async (_event, entry) => {
  try {
    vaultManager.addEntry(entry);
    await vaultManager.save();
    resetAutoLockTimer();
    return { ok: true };
  } catch (e) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:updateEntry', async (_event, entry) => {
  try {
    vaultManager.updateEntry(entry);
    await vaultManager.save();
    resetAutoLockTimer();
    return { ok: true };
  } catch (e) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:deleteEntry', async (_event, id) => {
  try {
    vaultManager.deleteEntry(id);
    await vaultManager.save();
    resetAutoLockTimer();
    return { ok: true };
  } catch (e) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:changePassword', async (_event, oldPassword, newPassword) => {
  try {
    await vaultManager.changePassword(oldPassword, newPassword);
    return { ok: true };
  } catch (e) {
    return { ok: false, error: e.message };
  }
});

// ── IPC: 文件对话框 ───────────────────────────────────────────────────────

ipcMain.handle('dialog:openFile', async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    filters: [{ name: 'VaultX Files', extensions: ['vaultx'] }],
    properties: ['openFile'],
  });
  return result.canceled ? null : result.filePaths[0];
});

ipcMain.handle('dialog:saveFile', async () => {
  const result = await dialog.showSaveDialog(mainWindow, {
    defaultPath: 'vault.vaultx',
    filters: [{ name: 'VaultX Files', extensions: ['vaultx'] }],
  });
  return result.canceled ? null : result.filePath;
});

ipcMain.handle('dialog:exportJson', async () => {
  const result = await dialog.showSaveDialog(mainWindow, {
    defaultPath: 'vaultx-export.json',
    filters: [{ name: 'JSON Files', extensions: ['json'] }],
  });
  return result.canceled ? null : result.filePath;
});

// ── IPC: 剪贴板 ───────────────────────────────────────────────────────────

ipcMain.handle('clipboard:write', async (_event, text) => {
  clipboard.writeText(text);
  resetAutoLockTimer();
  // 自动清除
  if (clipboardClearSeconds > 0) {
    if (clipboardTimer) clearTimeout(clipboardTimer);
    clipboardTimer = setTimeout(() => {
      // 仅当剪贴板内容未被修改时清除
      if (clipboard.readText() === text) {
        clipboard.writeText('');
      }
    }, clipboardClearSeconds * 1000);
  }
  return { ok: true };
});

// ── IPC: 密码生成器 ───────────────────────────────────────────────────────

ipcMain.handle('generator:generate', async (_event, config) => {
  const password = generatePassword(config);
  return { ok: true, password };
});

ipcMain.handle('generator:evaluate', async (_event, password) => {
  const strength = evaluatePasswordStrength(password);
  return { ok: true, strength };
});

// ── IPC: 主题 ─────────────────────────────────────────────────────────────

ipcMain.handle('theme:getSystem', async () => {
  return { dark: nativeTheme.shouldUseDarkColors };
});

nativeTheme.on('updated', () => {
  mainWindow?.webContents.send('theme:systemChanged', nativeTheme.shouldUseDarkColors);
});

// ── IPC: 设置 ─────────────────────────────────────────────────────────────

ipcMain.handle('settings:update', async (_event, settings) => {
  if (typeof settings.autoLockTimeout === 'number') {
    autoLockTimeout = settings.autoLockTimeout;
    resetAutoLockTimer();
  }
  if (typeof settings.clipboardClearSeconds === 'number') {
    clipboardClearSeconds = settings.clipboardClearSeconds;
  }
  return { ok: true };
});

// ── IPC: 导出 JSON ────────────────────────────────────────────────────────

ipcMain.handle('vault:exportJson', async (_event, filePath) => {
  try {
    const fs = require('fs');
    const entries = vaultManager.getEntries();
    fs.writeFileSync(filePath, JSON.stringify({ entries }, null, 2), 'utf-8');
    return { ok: true };
  } catch (e) {
    return { ok: false, error: e.message };
  }
});

// ── IPC: 活动心跳（重置自动锁定计时器）──────────────────────────────────

ipcMain.on('activity:ping', () => {
  resetAutoLockTimer();
});
