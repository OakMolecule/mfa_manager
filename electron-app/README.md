# VaultX — Electron 版

本地优先、安全可靠的跨平台密码管理工具（Electron 实现）。

## 功能

- AES-256-GCM 加密 + Argon2id 密钥派生（m=64MiB, t=3, p=4）
- 密码管理：创建/编辑/删除条目，用户名、密码、网址、备注
- TOTP 管理：实时 6 位验证码 + 进度条，即将过期橙色高亮
- 密码强度实时评估 + 内置密码生成器
- 一键复制（30 秒后自动清除剪贴板）
- 自动锁定（可配置，默认 5 分钟）
- 连续错误指数退避锁定（5 次后）
- 亮色/暗色/跟随系统主题
- 单文件 `.vaultx` 加密存储，兼容 Rust 版格式

## 项目结构

```
electron-app/
├── src/
│   ├── main/
│   │   ├── index.js      # 主进程：窗口管理、IPC 处理
│   │   ├── vault.js      # 金库加密/解密核心逻辑
│   │   └── generator.js  # 密码生成器
│   ├── preload/
│   │   └── index.js      # 预加载脚本（安全桥接）
│   └── renderer/
│       ├── index.html    # 入口 HTML
│       ├── app.js        # 渲染进程主逻辑（路由 + 页面）
│       ├── totp.js       # TOTP 计算（Web Crypto API）
│       └── styles/
│           ├── main.css              # Material Design 3 主题样式
│           └── material-icons.css    # 图标字体
├── .npmrc               # npm 镜像配置（华为云）
└── package.json
```

## 快速开始

```bash
cd electron-app

# 安装依赖（使用华为云镜像加速 Electron 下载）
npm install

# 启动开发模式
npm start

# 打包构建
npm run build
```

## 安全设计

- **contextIsolation + sandbox**：渲染进程无法直接访问 Node.js API
- **IPC 通信**：所有敏感操作（加解密、文件读写）在主进程执行
- **内存安全**：密钥在锁定时通过 `buffer.fill(0)` 归零
- **原子写入**：先写 `.tmp` 再 `rename`，防止写入中断导致数据损坏
- **无网络请求**：完全本地运行，无遥测

## 与 Rust 版格式兼容

`.vaultx` 文件格式完全兼容 Rust（`vaultx-core`）版本：

- 相同的 JSON 结构（`version`, `argon2_params`, `nonce`, `ciphertext`）
- 相同的 Argon2id 参数（m=65536, t=3, p=4）
- 相同的 AES-256-GCM 加密方式（nonce 12 字节，GCM tag 16 字节附在密文尾部）
- Base64 编码（标准编码）
