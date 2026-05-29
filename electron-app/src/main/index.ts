'use strict';

import { app, BrowserWindow, ipcMain, dialog, clipboard, nativeTheme } from 'electron';
import path from 'path';
import fs from 'fs';
import { VaultManager } from './vault';
import { generatePassword, evaluatePasswordStrength } from './generator';

process.env.ELECTRON_DISABLE_SECURITY_WARNINGS = 'true';

let mainWindow: BrowserWindow | null = null;
const vaultManager = new VaultManager();
let clipboardTimer: ReturnType<typeof setTimeout> | null = null;
let autoLockTimer: ReturnType<typeof setTimeout> | null = null;
let autoLockTimeout = 300;
let clipboardClearSeconds = 30;

function createWindow(): void {
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

  mainWindow.loadFile(path.join(__dirname, '../../src/renderer/index.html'));
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

function resetAutoLockTimer(): void {
  if (autoLockTimer) clearTimeout(autoLockTimer);
  if (autoLockTimeout > 0 && vaultManager.isUnlocked()) {
    autoLockTimer = setTimeout(() => {
      vaultManager.lock();
      mainWindow?.webContents.send('vault:locked');
    }, autoLockTimeout * 1000);
  }
}

// ── IPC: 金库操作 ─────────────────────────────────────────────────────────

ipcMain.handle('vault:check', async (_event, filePath: string) => {
  try {
    const exists = fs.existsSync(filePath);
    return { ok: true, exists };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:open', async (_event, filePath: string) => {
  try {
    await vaultManager.open(filePath);
    return { ok: true };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:create', async (_event, filePath: string, password: string) => {
  try {
    await vaultManager.create(filePath, password);
    resetAutoLockTimer();
    return { ok: true };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:unlock', async (_event, password: string) => {
  try {
    await vaultManager.unlock(password);
    resetAutoLockTimer();
    return { ok: true };
  } catch (e: any) {
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
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:addEntry', async (_event, entry) => {
  try {
    vaultManager.addEntry(entry);
    await vaultManager.save();
    resetAutoLockTimer();
    return { ok: true };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:importEntries', async (_event, entries) => {
  try {
    for (const entry of entries) vaultManager.addEntry(entry);
    await vaultManager.save();
    resetAutoLockTimer();
    return { ok: true, count: entries.length };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:updateEntry', async (_event, entry) => {
  try {
    vaultManager.updateEntry(entry);
    await vaultManager.save();
    resetAutoLockTimer();
    return { ok: true };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:deleteEntry', async (_event, id: string) => {
  try {
    vaultManager.deleteEntry(id);
    await vaultManager.save();
    resetAutoLockTimer();
    return { ok: true };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:deleteEntries', async (_event, ids: string[]) => {
  try {
    for (const id of ids) vaultManager.deleteEntry(id);
    await vaultManager.save();
    resetAutoLockTimer();
    return { ok: true, count: ids.length };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:getCategories', async () => {
  try {
    const categories = vaultManager.getCategories();
    resetAutoLockTimer();
    return { ok: true, categories };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:addCategory', async (_event, cat) => {
  try {
    vaultManager.addCategory(cat);
    await vaultManager.save();
    resetAutoLockTimer();
    return { ok: true };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:updateCategory', async (_event, cat) => {
  try {
    vaultManager.updateCategory(cat);
    await vaultManager.save();
    resetAutoLockTimer();
    return { ok: true };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:deleteCategory', async (_event, key: string) => {
  try {
    vaultManager.deleteCategory(key);
    await vaultManager.save();
    resetAutoLockTimer();
    return { ok: true };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

ipcMain.handle('vault:changePassword', async (_event, oldPassword: string, newPassword: string) => {
  try {
    await vaultManager.changePassword(oldPassword, newPassword);
    return { ok: true };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

// ── IPC: 文件对话框 ───────────────────────────────────────────────────────

ipcMain.handle('dialog:openFile', async () => {
  const result = await dialog.showOpenDialog(mainWindow!, {
    filters: [{ name: 'VaultX Files', extensions: ['vaultx'] }],
    properties: ['openFile'],
  });
  return result.canceled ? null : result.filePaths[0];
});

ipcMain.handle('dialog:openImportXml', async () => {
  const result = await dialog.showOpenDialog(mainWindow!, {
    filters: [{ name: 'KeePass XML', extensions: ['xml'] }],
    properties: ['openFile'],
  });
  return result.canceled ? null : result.filePaths[0];
});

ipcMain.handle('dialog:saveFile', async () => {
  const result = await dialog.showSaveDialog(mainWindow!, {
    defaultPath: 'vault.vaultx',
    filters: [{ name: 'VaultX Files', extensions: ['vaultx'] }],
  });
  return result.canceled ? null : result.filePath;
});

ipcMain.handle('dialog:exportJson', async () => {
  const result = await dialog.showSaveDialog(mainWindow!, {
    defaultPath: 'vaultx-export.json',
    filters: [{ name: 'JSON Files', extensions: ['json'] }],
  });
  return result.canceled ? null : result.filePath;
});

ipcMain.handle('util:readFile', async (_event, filePath: string) => {
  try {
    return { ok: true, content: fs.readFileSync(filePath, 'utf-8') };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

// ── IPC: 剪贴板 ───────────────────────────────────────────────────────────

ipcMain.handle('clipboard:write', async (_event, text: string) => {
  clipboard.writeText(text);
  resetAutoLockTimer();
  if (clipboardClearSeconds > 0) {
    if (clipboardTimer) clearTimeout(clipboardTimer);
    clipboardTimer = setTimeout(() => {
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

ipcMain.handle('generator:evaluate', async (_event, password: string) => {
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

ipcMain.handle('settings:update', async (_event, settings: Record<string, unknown>) => {
  if (typeof settings.autoLockTimeout === 'number') {
    autoLockTimeout = settings.autoLockTimeout as number;
    resetAutoLockTimer();
  }
  if (typeof settings.clipboardClearSeconds === 'number') {
    clipboardClearSeconds = settings.clipboardClearSeconds as number;
  }
  return { ok: true };
});

// ── IPC: 导出 JSON ────────────────────────────────────────────────────────

ipcMain.handle('vault:exportJson', async (_event, filePath: string) => {
  try {
    const entries = vaultManager.getEntries();
    fs.writeFileSync(filePath, JSON.stringify({ entries }, null, 2), 'utf-8');
    return { ok: true };
  } catch (e: any) {
    return { ok: false, error: e.message };
  }
});

// ── IPC: 活动心跳 ─────────────────────────────────────────────────────────

ipcMain.on('activity:ping', () => {
  resetAutoLockTimer();
});
