# VaultX 构建与运行指南

## 📦 系统要求

### 操作系统
- **Linux**: Ubuntu 20.04+ / Fedora 35+ / Arch Linux
- **macOS**: 10.15 Catalina 或更高版本
- **Windows**: Windows 10 (1903+) / Windows 11

### 开发工具
- **Rust**: 1.75.0 或更高版本
  ```bash
  # 安装 Rust（如果尚未安装）
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  
  # 验证安装
  rustc --version
  cargo --version
  ```

### 依赖库（Linux）
```bash
# Ubuntu / Debian
sudo apt-get install -y \
    libxkbcommon-dev \
    libwayland-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libvulkan-dev

# Fedora
sudo dnf install -y \
    wayland-devel \
    libxkbcommon-devel \
    libxcb-devel \
    vulkan-loader-devel

# Arch Linux
sudo pacman -S \
    wayland \
    libxkbcommon \
    libxcb \
    vulkan-icd-loader
```

### 依赖库（macOS）
```bash
# 无需额外依赖，系统自带
```

### 依赖库（Windows）
```bash
# 安装 Visual Studio 2019 或更高版本（含 C++ 工具链）
# 或者安装 Visual Studio Build Tools
```

---

## 🔨 编译

### 1. 进入项目目录
```bash
cd /home/oak/tmp/mfa_manager
```

### 2. 开发模式编译（快速）
```bash
cargo build
```
- 输出：`target/debug/vaultx`
- 编译时间：~2-5 分钟（首次编译，后续增量编译 <30秒）
- 包体积：~150 MB（包含调试符号）

### 3. 发布模式编译（优化）
```bash
cargo build --release
```
- 输出：`target/release/vaultx`
- 编译时间：~5-10 分钟（首次编译）
- 包体积：~12 MB（去除调试符号后）
- 优化级别：`opt-level = 3`

### 4. 检查编译警告
```bash
cargo check
```

### 5. 运行测试（如果有）
```bash
cargo test
```

---

## 🚀 运行

### 直接运行
```bash
# 开发模式
cargo run

# 发布模式
cargo run --release
```

### 独立运行（编译后）
```bash
# 开发模式
./target/debug/vaultx

# 发布模式
./target/release/vaultx
```

---

## 📁 金库文件位置

首次运行时，VaultX 会在以下位置创建金库文件：

### Linux
```bash
~/.local/share/VaultX/vault.vaultx
```

### macOS
```bash
~/Library/Application Support/VaultX/vault.vaultx
```

### Windows
```bash
%APPDATA%\VaultX\vault.vaultx
```

如果目录不存在，会自动创建。

---

## 🧪 功能测试

### 1. 创建新金库
1. 启动 VaultX
2. 在解锁页输入主密码（如 `test1234`）
3. 点击「创建新金库」
4. 等待 Argon2id 密钥派生完成（约 0.5-1.0 秒）
5. 成功后自动跳转到主列表页

### 2. 添加密码条目
1. 点击右上角「＋」按钮
2. 填写标题（必填）：`GitHub`
3. 填写用户名：`user@example.com`
4. 填写密码：`Test@123456`（或点击「生成」按钮）
5. 填写网址：`https://github.com`
6. 点击「保存」
7. 返回列表页，看到新条目

### 3. 添加 TOTP 条目
1. 点击「＋」新建条目
2. 填写标题：`GitHub 2FA`
3. 勾选「启用双因素验证 (TOTP)」
4. 填写 TOTP 密钥（Base32）：`JBSWY3DPEHPK3PXP`
5. 填写发行者：`GitHub`
6. 点击「保存」
7. 列表页顶部看到大字 TOTP 验证码（每 30 秒刷新）

### 4. 测试复制功能
1. 在列表页点击用户名旁的「📋」按钮
2. 打开文本编辑器，粘贴（Ctrl+V / Cmd+V）
3. 验证复制成功
4. 等待 30 秒，再次粘贴，剪贴板应已清空

### 5. 测试密码生成器
1. 点击左侧导航栏「密码生成器」
2. 调整长度滑块（如 24 位）
3. 勾选/取消字符集选项
4. 点击「重新生成」
5. 点击「复制」
6. 返回列表页

### 6. 测试自动锁定
1. 在设置页选择「自动锁定超时：1 分钟」
2. 返回列表页
3. 等待 1 分钟不操作
4. 自动跳转到解锁页
5. 重新输入主密码解锁

### 7. 测试主题切换
1. 点击「设置」
2. 在「外观」区域选择「深色模式」
3. 整个界面切换为暗色主题
4. 再切回「浅色模式」

### 8. 测试 TOTP 总览
1. 点击左侧导航栏「TOTP (N)」
2. 查看所有带 TOTP 的条目
3. 观察倒计时和进度条
4. 剩余 <7 秒时验证码和倒计时变为橙色

### 9. 测试搜索
1. 在列表页顶部搜索框输入「GitHub」
2. 列表过滤只显示匹配条目
3. 清空搜索框，恢复全部条目

### 10. 测试删除
1. 点击条目卡片右上角「编辑」按钮（跳转详情页）
2. 滚动到底部，点击「删除此条目」按钮
3. 条目从列表消失
4. 金库文件自动保存

---

## 🐛 常见问题

### Q1: 编译时报错 `failed to resolve: use of undeclared crate or module`
**A**: 运行 `cargo clean && cargo build`，清理缓存后重新编译。

### Q2: Linux 上运行时黑屏或崩溃
**A**: 确保安装了 Wayland / X11 开发库：
```bash
sudo apt-get install libxkbcommon-dev libwayland-dev libxcb-shape0-dev
```

### Q3: Windows 上编译报错 `link.exe not found`
**A**: 安装 Visual Studio Build Tools：
```bash
# 下载并运行
https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
```

### Q4: macOS 上报错 `xcrun: error: invalid active developer path`
**A**: 安装 Xcode Command Line Tools：
```bash
xcode-select --install
```

### Q5: 运行时窗口尺寸异常
**A**: 删除金库文件（会清空数据）后重新运行：
```bash
rm ~/.local/share/VaultX/vault.vaultx
cargo run --release
```

### Q6: 字体显示异常（中文乱码）
**A**: VaultX 已内嵌文泉驿微米黑字体，如果仍有问题，检查 `fonts/` 目录是否存在所有字体文件。

### Q7: 解锁时一直转圈（卡住）
**A**: Argon2id 密钥派生需要 0.5-1.0 秒，请耐心等待。如果超过 5 秒仍未响应，检查终端错误日志。

---

## 📦 打包发布

### Linux AppImage
```bash
# 安装 linuxdeploy
wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage

# 编译发布版本
cargo build --release

# 打包
./linuxdeploy-x86_64.AppImage \
  --executable target/release/vaultx \
  --appdir AppDir \
  --output appimage
```

### macOS .dmg
```bash
# 编译 universal binary（支持 Intel + Apple Silicon）
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# 合并
lipo -create \
  target/x86_64-apple-darwin/release/vaultx \
  target/aarch64-apple-darwin/release/vaultx \
  -output target/release/vaultx-universal

# 创建 .app 包
mkdir -p VaultX.app/Contents/MacOS
cp target/release/vaultx-universal VaultX.app/Contents/MacOS/VaultX

# 创建 .dmg（需要 create-dmg 工具）
create-dmg VaultX.app
```

### Windows .exe
```bash
# 编译
cargo build --release --target x86_64-pc-windows-msvc

# 添加图标（可选）
# 使用 winres crate 在 build.rs 中设置

# 打包为单文件（已默认静态链接）
# 输出：target/x86_64-pc-windows-msvc/release/vaultx.exe
```

---

## 🔧 性能优化

### 减小包体积
```bash
# 在 Cargo.toml 中添加
[profile.release]
opt-level = "z"  # 优化体积
lto = true       # 链接时优化
codegen-units = 1
strip = true     # 去除调试符号
```

### 加快编译速度
```bash
# 使用 mold 链接器（Linux）
sudo apt-get install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
cargo build --release
```

### 并行编译
```bash
# 设置并行编译任务数
export CARGO_BUILD_JOBS=8
cargo build --release
```

---

## 📊 编译输出示例

### 成功输出
```
   Compiling vaultx-core v0.1.0 (/home/oak/tmp/mfa_manager/vaultx-core)
   Compiling vaultx-ui v0.1.0 (/home/oak/tmp/mfa_manager/vaultx-ui)
    Finished `release` profile [optimized] target(s) in 4m 23s
```

### 包体积
```bash
$ ls -lh target/release/vaultx
-rwxr-xr-x 1 oak oak 12M Apr 21 10:00 target/release/vaultx
```

### 运行时内存占用
- 启动时：~50 MB
- 解锁后：~80 MB
- 1000 条目：~120 MB

---

## 🎯 下一步

1. ✅ 编译成功
2. ✅ 功能测试通过
3. 📝 编写用户手册
4. 📦 打包发布（AppImage / .dmg / .exe）
5. 🚀 发布到 GitHub Releases
6. 📢 宣传推广

---

## 📞 支持

如有问题，请查看：
1. [README.md](../README.md) - 项目概述
2. [IMPLEMENTATION.md](./IMPLEMENTATION.md) - 实施细节
3. [PRD.md](./PRD.md) - 产品需求文档
4. [UI_DESIGN.md](./UI_DESIGN.md) - UI 设计规范

---

<div align="center">

**祝编译顺利！** 🎉

</div>
