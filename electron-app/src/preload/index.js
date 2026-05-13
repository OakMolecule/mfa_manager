'use strict';

const { contextBridge, ipcRenderer } = require('electron');

// 暴露安全 API 到渲染进程（contextIsolation + sandbox）
contextBridge.exposeInMainWorld('vaultxAPI', {
  // 金库操作
  vault: {
    check:          (filePath)              => ipcRenderer.invoke('vault:check', filePath),
    open:           (filePath)              => ipcRenderer.invoke('vault:open', filePath),
    create:         (filePath, password)    => ipcRenderer.invoke('vault:create', filePath, password),
    unlock:         (password)              => ipcRenderer.invoke('vault:unlock', password),
    lock:           ()                      => ipcRenderer.invoke('vault:lock'),
    getEntries:     ()                      => ipcRenderer.invoke('vault:getEntries'),
    addEntry:       (entry)                 => ipcRenderer.invoke('vault:addEntry', entry),
    updateEntry:    (entry)                 => ipcRenderer.invoke('vault:updateEntry', entry),
    deleteEntry:    (id)                    => ipcRenderer.invoke('vault:deleteEntry', id),
    changePassword: (oldPwd, newPwd)        => ipcRenderer.invoke('vault:changePassword', oldPwd, newPwd),
    exportJson:     (filePath)              => ipcRenderer.invoke('vault:exportJson', filePath),
  },

  // 对话框
  dialog: {
    openFile:   () => ipcRenderer.invoke('dialog:openFile'),
    saveFile:   () => ipcRenderer.invoke('dialog:saveFile'),
    exportJson: () => ipcRenderer.invoke('dialog:exportJson'),
  },

  // 剪贴板
  clipboard: {
    write: (text) => ipcRenderer.invoke('clipboard:write', text),
  },

  // 密码生成器
  generator: {
    generate:  (config)    => ipcRenderer.invoke('generator:generate', config),
    evaluate:  (password)  => ipcRenderer.invoke('generator:evaluate', password),
  },

  // 主题
  theme: {
    getSystem: () => ipcRenderer.invoke('theme:getSystem'),
    onSystemChanged: (cb) => ipcRenderer.on('theme:systemChanged', (_e, dark) => cb(dark)),
  },

  // 设置
  settings: {
    update: (settings) => ipcRenderer.invoke('settings:update', settings),
  },

  // 金库被自动锁定时的通知
  onVaultLocked: (cb) => ipcRenderer.on('vault:locked', cb),

  // 用户活动心跳（重置自动锁定计时器）
  pingActivity: () => ipcRenderer.send('activity:ping'),
});
