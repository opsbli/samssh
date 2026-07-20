# Design ↔ Requirement 映射

**Date**: 2026-07-21

---

## architecture.puml 组件 → Requirement 映射

| 组件 | 需求 | 状态 | 说明 |
|------|------|------|------|
| TitleBar | R009 (窗口与 UI) | ✅ 已实现 | 自定义标题栏 |
| SidebarTree | R004 (侧边栏) + R010 | ✅ 已实现 | 连接树 + 右键菜单 |
| TabBar | R004 (标签页) | ✅ 已实现 | 标签栏（使用 gpui-component TabBar） |
| ProfileDialog | R010 (新建连接) | ✅ 已实现 | 新建/编辑连接对话框 |
| TerminalView | R002 (终端) | ✅ 已实现 | 终端模拟视图渲染 |
| **FileManagerView** | **R003 (SFTP 文件管理)** | **✅ 已实现** | **双面板文件浏览器（新增 2026-07-21）** |
| **FileBrowser** | **R003 (SFTP 文件管理)** | **✅ 已实现** | **通用文件浏览器组件（新增 2026-07-21）** |
| **LocalFileSystem** | **R003 (本地面板)** | **✅ 已实现** | **本地面板异步目录读取** |
| SettingsDialog | R005 (配置设置) | ❌ TODO | 设置对话框 |
| ThemeSelector | R006 (主题) | ❌ TODO | 主题选择器 |
| SSHService | R001 + R008 | ✅ 已实现 | SSH 连接管理 + 安全 |
| SFTPClient | R003 | ✅ 已实现 | SFTP 文件操作 |
| CryptoService | R008 | ✅ 已实现 | DPAPI 加密 |
| KeyVerifier | R001 | ✅ 已实现 | 主机密钥验证 |

---

## app_state.puml 实体 → Requirement 映射

| 实体 | 需求 | 状态 |
|------|------|------|
| SidebarState | R004 | ✅ 已实现 |
| TabState | R004 | ✅ 已实现 |
| SessionState | R001 | ✅ 已实现 |
| Config / Profile | R005 | ✅ 已实现 |
| Workspace | R010 | ✅ 已实现 |
| SidebarTree | R004 | ✅ 已实现 |
| ProfileDialog | R010 | ✅ 已实现 |
| **FileManagerState** | **R003** | **✅ 已实现（新增 2026-07-21）** |
| **FileBrowserState** | **R003** | **✅ 已实现（新增 2026-07-21）** |
| **FileEntry** | **R003** | **✅ 已实现（新增 2026-07-21）** |

---

## module_deps.md 模块 → Requirement 映射

| 模块 | 需求 | 状态 |
|------|------|------|
| `app/state.rs` | 全部 | ✅ 已更新 |
| `app/components/title_bar.rs` | R009 | ✅ 已实现 |
| `app/components/sidebar_tree.rs` | R004 + R010 | ✅ 已实现 |
| `app/components/tab_bar.rs` | R004 | ✅ 已实现 |
| `app/components/dialog.rs` | R010 | ✅ 已实现 |
| **`app/components/file_browser.rs`** | **R003** | **✅ 已实现（新增 2026-07-21）** |
| **`app/components/file_manager_view.rs`** | **R003** | **✅ 已实现（新增 2026-07-21）** |
| `app/views/workspace.rs` | R010 + R003 | ✅ 已更新（Tab kind 切换） |
| `app/views/terminal_view.rs` | R002 | ✅ 已实现 |
| `session.rs` | R001 + R003 | ✅ 已更新（SFTP 标签创建） |
| `ssh/` | R001 + R008 | ✅ 已实现 |
| `sftp/` | R003 | ✅ 已实现 |
| `terminal/` | R002 | ✅ 已实现 |
| `config/` | R005 | ✅ 已实现 |
| `crypto/` | R008 | ✅ 已实现 |
| `theme/` | R006 | ❌ TODO |
| `tray/` | R009 | ❌ TODO |
