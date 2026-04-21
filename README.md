# VaultX — 本地优先的密码管理器

<div align="center">

🔐 **军事级加密** · 🚀 **高性能** · 🎨 **Material Design 3 Blue** · 🌓 **亮色/暗色主题**

</div>

---

## ✨ 特性

### 🔒 **安全第一**
- **AES-256-GCM** 加密存储，每次保存生成随机 nonce
- **Argon2id** 密钥派生（m=65536 KiB, t=3, p=4），阻止暴力破解
- **zeroize** 内存归零，敏感数据在锁定时立即清除
- 连续密码错误 5 次后指数退避锁定
- 自动锁定（默认 5 分钟无操作）
- 剪贴板自动清除（默认 30 秒）

### 🎯 **统一管理**
- **密码 + TOTP 二合一**：同一条目可同时存储密码和 TOTP 验证码
- **列表页直接展示**：无需进入详情页即可复制用户名、密码、TOTP 码
- **实时 TOTP 倒计时**：进度条显示剩余时间，≤5s 时橙色警告
- **智能搜索**：按标题、用户名、网址快速筛选

### 🛠 **便捷工具**
- **内置密码生成器**：可配置长度（8-64 位）、字符集、排除易混淆字符
- **密码强度评估**：实时显示弱/中/强等级
- **分类管理**：工作/个人/金融/购物分类
- **一键复制**：所有字段一键复制到剪贴板

### 🎨 **现代界面**
- **Material Design 3 Blue** 设计语言
- **圆角卡片布局**，简洁现代
- **亮色/暗色双主题**，护眼友好
- **中英文混排优化**，使用文泉驿微米黑 + Inter 字体
- **响应式布局**，流畅的交互体验

---

## 📦 技术栈

| 类别 | 技术 |
|------|------|
| **UI 框架** | [iced](https://github.com/iced-rs/iced) 0.13 |
| **加密** | aes-gcm, argon2 |
| **TOTP** | totp-rs |
| **内存安全** | zeroize, secrecy |
| **序列化** | serde, serde_json |

---

## 🚀 快速开始

### 前置条件
- Rust 1.75+ （建议使用 rustup 安装）
- Linux / macOS / Windows 10+

### 编译
```bash
# 克隆仓库（或直接使用现有代码）
cd /home/oak/tmp/mfa_manager

# 编译发布版本
cargo build --release

# 运行
./target/release/vaultx
```

### 首次使用
1. 启动后显示**解锁页**
2. 如果金库文件不存在，输入主密码后点击「创建新金库」
3. 金库文件保存在：
   - Linux: `~/.local/share/VaultX/vault.vaultx`
   - macOS: `~/Library/Application Support/VaultX/vault.vaultx`
   - Windows: `%APPDATA%\VaultX\vault.vaultx`

---

## 📱 界面说明

### ① 解锁页
- 输入主密码解锁金库
- 连续错误 5 次后触发退避锁定（最长 30 秒）
- 支持创建新金库（单文件 `.vaultx` 格式）

### ② 主列表页
**顶栏**：搜索框、新建按钮、设置按钮、锁定按钮

**左侧导航栏**（200px）：
- 全部条目
- 类型分组（密码/TOTP）
- 工具（密码生成器）

**内容区**：条目卡片列表
- **TOTP 置顶**：大字显示验证码（482 917），进度条倒计时
- **用户名 + 密码同行**：默认遮盖，点击 👁 切换明文，一键复制
- **卡片悬停**：边框高亮，阴影加深

### ③ 条目详情页
- 头像圆圈（首字母）+ 标题 + 分类
- TOTP 区块：验证码、进度条、发行方/账号
- 密码区块：网址、用户名、密码（带强度条）
- 备注区块（如有）
- 删除按钮（底部）

### ④ TOTP 总览页
- 网格展示所有绑定 TOTP 的条目
- 大字验证码 + 倒计时 + 复制按钮
- 剩余时间 ≤7s 时橙色警告

### ⑤ 密码生成器
- 长度滑块（8-64）
- 字符集勾选：大写/小写/数字/符号/排除易混淆字符
- 实时预览 + 刷新 + 复制按钮

### ⑥ 设置页
**外观**：亮色/暗色模式（跟随系统功能待实现）

**安全**：
- 自动锁定超时：从不/1分钟/5分钟/15分钟/30分钟/1小时
- 立即锁定按钮

**关于**：版本号 + 加密算法说明

### ⑦ 新建条目页
- 标题（必填）
- 用户名/密码/网址（可选）
- TOTP 开关：密钥(Base32)/发行方/账号
- 密码和 TOTP 可同时启用

---

## 🔐 数据格式

### `.vaultx` 文件结构（JSON）
```json
{
  "version": 1,
  "argon2_params": {
    "m_cost": 65536,
    "t_cost": 3,
    "p_cost": 4,
    "salt": "<base64>"
  },
  "nonce": "<base64, 12 bytes>",
  "ciphertext": "<base64, AES-256-GCM encrypted JSON>"
}
```

### 解密后明文结构
```json
{
  "entries": [
    {
      "id": "uuid-v4",
      "title": "GitHub",
      "category": "Work",
      "created_at": "2026-04-20T10:00:00Z",
      "updated_at": "2026-04-20T10:00:00Z",
      "password": {
        "url": "https://github.com",
        "username": "user@example.com",
        "password": "tK9#mZ2@pL5!xQR",
        "notes": ""
      },
      "totp": {
        "secret": "JBSWY3DPEHPK3PXP",
        "issuer": "GitHub",
        "account": "user@example.com",
        "algorithm": "SHA1",
        "digits": 6,
        "period": 30
      }
    }
  ]
}
```

---

## 🎨 设计规范

### 颜色系统（亮色模式）
| Token | 色值 | 用途 |
|-------|------|------|
| `--primary` | `#1976D2` | 主按钮、激活状态、TOTP 码 |
| `--primary-container` | `#BBDEFB` | 复制按钮背景、Badge |
| `--surface` | `#FAFAFA` | 页面背景 |
| `--surface-variant` | `#E3F2FD` | 侧边栏、悬停 |
| `--card-bg` | `#FFFFFF` | 卡片背景 |
| `--error` | `#D32F2F` | 错误提示 |
| `--warning` | `#FF6D00` | TOTP 即将过期 |

### 圆角规范
- 卡片：12px
- 按钮：8px
- 输入框：8px
- 对话框：16px

### 字体
- 中文：文泉驿微米黑 (`wqy-microhei.ttc`)
- 英文：Inter (`Inter-VariableFont_opsz,wght.ttf`)
- 等宽：Roboto Mono (`RobotoMono-VariableFont_wght.ttf`)
- 图标：Material Icons (`MaterialIcons-Regular.ttf`)

---

## 📐 项目结构

```
mfa_manager/
├── Cargo.toml              # Workspace 配置
├── README.md               # 本文档
├── docs/
│   ├── PRD.md              # 产品需求文档
│   ├── UI_DESIGN.md        # UI 设计规范
│   └── ui_mockup.html      # 交互式原型
├── fonts/                  # 字体文件
│   ├── wqy-microhei.ttc
│   ├── Inter-VariableFont_opsz,wght.ttf
│   ├── RobotoMono-VariableFont_wght.ttf
│   └── MaterialIcons-Regular.ttf
├── vaultx-core/            # 核心库（加密、TOTP、密码生成）
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── crypto.rs       # AES-256-GCM + Argon2id
│       ├── vault.rs        # 金库文件读写
│       ├── entry.rs        # Entry / PasswordData / TotpData
│       ├── totp.rs         # TOTP 计算引擎
│       └── generator.rs    # 密码生成器
└── vaultx-ui/              # UI 层（iced）
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── app.rs          # 全局状态管理
        ├── theme.rs        # 颜色/字体/圆角常量
        ├── icons.rs        # Material Icons 码点
        └── screens/
            ├── mod.rs
            ├── unlock.rs   # 解锁页
            ├── list.rs     # 主列表页
            ├── detail.rs   # 条目详情页
            ├── totp_view.rs# TOTP 总览页
            ├── generator.rs# 密码生成器
            ├── settings.rs # 设置页
            └── new_entry.rs# 新建条目页
```

---

## ✅ 已完成功能（v0.1.0）

### 核心功能
- [x] 解锁 / 创建新金库
- [x] 密码条目 CRUD
- [x] TOTP 条目附加到密码条目
- [x] 列表页内联展示（TOTP + 用户名/密码 + 复制）
- [x] 密码生成器（8-64位，可配置字符集）
- [x] 搜索（标题、用户名、网址）
- [x] 设置（亮色/暗色主题、自动锁定、立即锁定）
- [x] 导入/导出 `.vaultx`（通过文件系统操作）
- [x] 亮色/暗色双主题

### 安全功能
- [x] AES-256-GCM 加密
- [x] Argon2id 密钥派生
- [x] 内存归零（zeroize）
- [x] 连续密码错误退避锁定
- [x] 自动锁定（可配置）
- [x] 剪贴板自动清除（30秒）

### UI/UX
- [x] Material Design 3 Blue 风格
- [x] 圆角卡片布局
- [x] 亮色/暗色主题切换
- [x] 中英文混排优化
- [x] TOTP 实时倒计时 + 进度条
- [x] 密码明文/遮盖切换
- [x] 卡片悬停动效

---

## 🚧 计划中功能（v1.1+）

- [ ] 浏览器扩展自动填充（Firefox / Chrome）
- [ ] 导入 Bitwarden / 1Password CSV
- [ ] SSH 密钥管理
- [ ] 多金库支持
- [ ] 移动端（iOS / Android）
- [ ] 修改主密码功能（UI层）
- [ ] 导出明文 JSON（危险操作，需二次确认）
- [ ] 条目编辑功能（当前仅支持新建和删除）

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

## 📄 许可证

MIT License

---

## 🙏 致谢

- [iced](https://github.com/iced-rs/iced) - 优雅的 Rust GUI 框架
- [totp-rs](https://github.com/constantoine/totp-rs) - TOTP 实现
- [Material Icons](https://fonts.google.com/icons) - 图标字体
- [文泉驿微米黑](http://wenq.org/wqy2/index.cgi?MicroHei) - 开源中文字体

---

<div align="center">

Made with ❤️ in Rust

</div>
