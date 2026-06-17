'use strict';

import { contextBridge, ipcRenderer } from 'electron';

contextBridge.exposeInMainWorld('vaultxAPI', {
  vault: {
    check:          (filePath: string)              => ipcRenderer.invoke('vault:check', filePath),
    open:           (filePath: string)              => ipcRenderer.invoke('vault:open', filePath),
    create:         (filePath: string, password: string) => ipcRenderer.invoke('vault:create', filePath, password),
    unlock:         (password: string)              => ipcRenderer.invoke('vault:unlock', password),
    lock:           ()                              => ipcRenderer.invoke('vault:lock'),
    getEntries:     ()                              => ipcRenderer.invoke('vault:getEntries'),
    addEntry:       (entry: unknown)                => ipcRenderer.invoke('vault:addEntry', entry),
    importEntries:  (entries: unknown[])             => ipcRenderer.invoke('vault:importEntries', entries),
    updateEntry:    (entry: unknown)                => ipcRenderer.invoke('vault:updateEntry', entry),
    deleteEntry:    (id: string)                    => ipcRenderer.invoke('vault:deleteEntry', id),
    deleteEntries:  (ids: string[])                 => ipcRenderer.invoke('vault:deleteEntries', ids),
    changePassword: (oldPwd: string, newPwd: string) => ipcRenderer.invoke('vault:changePassword', oldPwd, newPwd),
    exportJson:     (filePath: string)              => ipcRenderer.invoke('vault:exportJson', filePath),
    getCategories:  ()                              => ipcRenderer.invoke('vault:getCategories'),
    addCategory:    (cat: unknown)                  => ipcRenderer.invoke('vault:addCategory', cat),
    updateCategory: (cat: unknown)                  => ipcRenderer.invoke('vault:updateCategory', cat),
    deleteCategory: (key: string)                   => ipcRenderer.invoke('vault:deleteCategory', key),
  },

  dialog: {
    openFile:      () => ipcRenderer.invoke('dialog:openFile'),
    openImportXml: () => ipcRenderer.invoke('dialog:openImportXml'),
    saveFile:      () => ipcRenderer.invoke('dialog:saveFile'),
    exportJson:    () => ipcRenderer.invoke('dialog:exportJson'),
  },

  clipboard: {
    write: (text: string) => ipcRenderer.invoke('clipboard:write', text),
  },

  generator: {
    generate:  (config: unknown)    => ipcRenderer.invoke('generator:generate', config),
    evaluate:  (password: string)   => ipcRenderer.invoke('generator:evaluate', password),
  },

  theme: {
    getSystem: () => ipcRenderer.invoke('theme:getSystem'),
    onSystemChanged: (cb: (dark: boolean) => void) => ipcRenderer.on('theme:systemChanged', (_e: unknown, dark: boolean) => cb(dark)),
  },

  settings: {
    update: (settings: unknown) => ipcRenderer.invoke('settings:update', settings),
  },

  onVaultLocked: (cb: () => void) => ipcRenderer.on('vault:locked', cb),

  readFile: (filePath: string) => ipcRenderer.invoke('util:readFile', filePath),

  openExternal: (url: string) => ipcRenderer.invoke('shell:openExternal', url),

  window: {
    close: () => ipcRenderer.send('window:close'),
    minimize: () => ipcRenderer.send('window:minimize'),
    maximize: () => ipcRenderer.send('window:maximize'),
  },

  devtools: {
    toggle: () => ipcRenderer.invoke('devtools:toggle'),
  },

  pingActivity: () => ipcRenderer.send('activity:ping'),
});
