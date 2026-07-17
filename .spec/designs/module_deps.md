# SamSSH 模块依赖关系

**Date**: 2026-07-17

---

## 模块层级

```
src/
├── main.rs                  # 入口点：eframe::run_native()
├── app/                     # UI 组件树
│   ├── mod.rs               # AppState, 模块导出
│   ├── components/          # 可复用 UI 组件
│   │   ├── sidebar_tree.rs   # 侧边栏连接树 (R004)
│   │   ├── tab_bar.rs        # 标签页 (R004)
│   │   ├── title_bar.rs      # 自定义标题栏 (R009)
│   │   └── dialog.rs         # 通用对话框
│   ├── views/               # 视图层
│   │   ├── terminal_view.rs  # 终端视图 (R002)
│   │   ├── file_manager.rs   # SFTP 文件浏览器 (R003)
│   │   ├── settings.rs       # 设置对话框 (R005)
│   │   ├── transfer_mgr.rs   # 下载管理器窗口 (R007)
│   │   └── theme_selector.rs # 主题选择器 (R006)
│   └── state.rs             # AppState Entity 定义
├── ssh/                     # SSH 连接管理 (R001)
│   ├── mod.rs
│   ├── client.rs             # russh 客户端封装
│   ├── auth.rs               # 认证处理
│   └── known_hosts.rs        # known_hosts 管理
├── sftp/                    # SFTP 文件传输 (R003)
│   ├── mod.rs
│   ├── client.rs             # russh-sftp 客户端
│   └── transfer.rs           # 传输任务管理
├── terminal/                # 终端模拟 (R002)
│   ├── mod.rs
│   └── terminal.rs           # wezterm-term 封装
├── config/                  # 配置管理 (R005)
│   ├── mod.rs
│   ├── profile.rs            # Profile 定义
│   ├── settings.rs           # 应用设置
│   └── store.rs              # 持久化 (JSON + DPAPI)
├── theme/                   # 主题系统 (R006)
│   ├── mod.rs
│   ├── ui_theme.rs           # .thm 文件解析
│   └── term_theme.rs         # TOML 配色加载
├── crypto/                  # 加密工具 (R008)
│   ├── mod.rs
│   └── dpapi.rs              # Windows DPAPI 封装
├── tray/                    # 系统托盘 (P1)
├── logger/                  # 日志
└── model.rs                 # 核心数据模型
```

## 依赖方向

```
main.rs
  └── app/ (UI 层)
        ├── ssh/ ─────→ russh crate
        ├── sftp/ ────→ russh-sftp crate
        ├── terminal/ ─→ wezterm-term crate
        ├── config/ ───→ crypto/ ─→ windows-sys (DPAPI)
        ├── theme/
        ├── crypto/
        ├── tray/
        └── logger/
```

## 关键依赖规则

1. **UI 层（app/）不直接依赖外部 crate** — 通过 service 模块中介
2. **ssh/ 和 sftp/ 是纯异步层** — 通过消息队列与 UI 通信
3. **config/ 是同步层** — 读写 JSON，DPAPI 加密通过 crypto/
4. **终端渲染由 wezterm-term 输出位图** — 由 terminal_view.rs 渲染到 gpui 画布
