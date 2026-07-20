# Design → Requirement 追溯

**Date**: 2026-07-20

---

| Design Artifact | 覆盖的 Requirement |
|----------------|-------------------|
| `architecture.puml` | R001, R002, R003, R004, R005, R006, R007, R008, R009, R010 |
| `app_state.puml` | R001, R002, R003, R004, R005, R008, R009, R010 |
| `module_deps.md` | R001, R002, R003, R004, R005, R006, R007, R008, R009, R010 |

### Requirement → Design 覆盖

| Requirement | Design Artifact |
|------------|----------------|
| R001 (SSH 连接管理) | `architecture.puml` (SSHService 组件), `module_deps.md` (ssh/ 模块) |
| R002 (终端功能) | `architecture.puml` (TerminalView), `module_deps.md` (terminal/ 模块) |
| R003 (SFTP 文件管理) | `architecture.puml` (FileManagerView + SFTPClient), `module_deps.md` (sftp/ 模块) |
| R004 (标签页与侧边栏) | `architecture.puml` (SidebarTree + TabBar), `app_state.puml` (SidebarState + TabState) |
| R005 (配置与设置) | `architecture.puml` (ConfigStore + SettingsDialog), `module_deps.md` (config/ 模块) |
| R006 (主题系统) | `architecture.puml` (ThemeSelector), `app_state.puml` (—), `module_deps.md` (theme/ 模块) |
| R007 (下载管理器) | `architecture.puml` (—), `app_state.puml` (—), `module_deps.md` (sftp/transfer) |
| R008 (安全与加密) | `architecture.puml` (CryptoService + KeyVerifier), `module_deps.md` (crypto/ 模块) |
| R009 (窗口与 UI 体验) | `architecture.puml` (TitleBar), `app_state.puml` (—) |
| R010 (新建/编辑连接) | `architecture.puml` (ProfileDialog + Workspace), `app_state.puml` (ProfileDialog + Workspace), `module_deps.md` (dialog.rs + workspace.rs) |
