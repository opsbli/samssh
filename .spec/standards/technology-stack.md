# SamSSH 技术栈标准

**Source**: docs/需求功能文档.md 第 1.1 节

---

## 核心依赖

| 层次 | 技术选型 | 用途 |
|------|---------|------|
| GUI 框架 | gpui | 原生 GUI 框架，响应式声明式 UI |
| UI 组件库 | gpui-component | Tab, Tree, Panel, Dialog, Split 等组件 |
| 渲染引擎 | wgpu（DX12 / Vulkan） | GPU 渲染后端 |
| SSH 协议 | russh | SSH 客户端协议实现 |
| SFTP 协议 | russh-sftp | SFTP 文件传输协议 |
| 终端模拟 | wezterm-term | VT100/xterm 终端模拟 |
| 终端配色 | WezTerm TOML 格式 | 终端颜色方案 |
| 配置加密 | Windows DPAPI | 凭据加密存储 |
| 异步运行时 | gpui::BackgroundExecutor | gpui 内置异步 |
| 系统托盘 | tray-icon | Windows 托盘图标 |
| 全局剪贴板 | arboard | 剪贴板读写 |
| 文件对话框 | rfd | 原生 Windows 文件选择 |
| 日志 | tracing | 结构化日志 |

## Rust 版本要求
- Edition: 2021
- Minimum Rust: 1.75+
- Target: x86_64-pc-windows-msvc

## Cargo Features 约定
- 默认禁用不必要的 feature
- 使用 `default-features = false` + 按需启用
