# SamSSH 模块依赖关系

**Date**: 2026-07-20

---

## 模块层级

```
src/
├── main.rs                  # 入口点：gpui::App::new()
├── app/                     # UI 组件树
│   ├── mod.rs               # AppState, 模块导出
│   ├── components/          # 可复用 UI 组件
│   │   ├── sidebar_tree.rs   # 侧边栏连接树 (R004) + 右键菜单 (R010)
│   │   ├── tab_bar.rs        # 标签页 (R004) — 未集成到 workspace
│   │   ├── title_bar.rs      # 自定义标题栏 (R009)
│   │   └── dialog.rs         # 新建/编辑连接对话框 (R010)
│   ├── views/               # 视图层
│   │   ├── workspace.rs      # 主工作区 (R010) — 唯一已实现视图
│   │   ├── terminal_view.rs  # 终端视图 (R002) — TODO
│   │   ├── file_manager.rs   # SFTP 文件浏览器 (R003) — TODO
│   │   ├── settings.rs       # 设置对话框 (R005) — TODO
│   │   ├── transfer_mgr.rs   # 下载管理器窗口 (R007) — TODO
│   │   └── theme_selector.rs # 主题选择器 (R006) — TODO
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
│   ├── ui_theme.rs           # .thm 文件解析 — TODO
│   └── term_theme.rs         # TOML 配色加载 — TODO
├── crypto/                  # 加密工具 (R008)
│   ├── mod.rs
│   └── dpapi.rs              # Windows DPAPI 封装
├── tray/                    # 系统托盘 — TODO
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
        ├── theme/ (TODO)
        ├── crypto/
        ├── tray/ (TODO)
        └── logger/
```

## 关键依赖规则

1. **UI 层（app/）不直接依赖外部 crate** — 通过 service 模块中介
2. **ssh/ 和 sftp/ 是纯异步层** — 通过消息队列与 UI 通信
3. **config/ 是同步层** — 读写 JSON，DPAPI 加密通过 crypto/
4. **终端渲染由 wezterm-term 输出位图** — 由 terminal_view.rs 渲染到 gpui 画布

---

## SSH 模块内部设计

### 核心类型

```rust
/// 连接状态机
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Authenticating,
    Connected,
    Error(String),
}

/// 认证方式
pub enum AuthMethod {
    Password { password: String },
    Key { key_path: PathBuf, passphrase: Option<String> },
    KeyboardInteractive,
}
```

### 模块内部依赖

```
ssh/
├── mod.rs              # 模块导出 + 公共类型定义
│   ├── client.rs       # SshClient + russh::client::Handler 实现
│   │                   #   connect → tcp + ssh 握手
│   │                   #   authenticate → password/key/kbi
│   │                   #   disconnect → 优雅断开
│   │                   #   事件发送 → mpsc::Sender 到 UI
│   ├── auth.rs         # 认证逻辑
│   │                   #   perform_password_auth()
│   │                   #   perform_key_auth() → russh-keys
│   │                   #   perform_kbi_auth() → 键盘交互
│   │                   #   auth_priority: Key > Password > KBI
│   └── known_hosts.rs  # 主机密钥验证
│                       #   load_known_hosts() → ~/.ssh/known_hosts
│                       #   save_host_key()
│                       #   verify_host_key() → check_server_key
```

### SSH → UI 事件通道

```rust
pub enum SshEvent {
    Connected { session_id: SessionId },
    Disconnected { session_id: SessionId, reason: String },
    AuthBanner { message: String },
    KeyVerificationRequired { host: String, key_fingerprint: String, key_type: String },
    KeyVerificationChanged { host: String, old_fingerprint: String, new_fingerprint: String },
    Error { session_id: SessionId, message: String },
    StatusChanged { session_id: SessionId, status: ConnectionStatus },
}
```

### 认证流程

```
connect() → TCP → 密钥交换 → 主机密钥验证 → 认证阶段
                                                 │
              ┌────────────────────────────────────┤
              ▼                                    ▼
   ┌──────────────────┐              ┌──────────────────────┐
   │  私钥认证 (P1)    │              │  密码/键盘交互 (P2)  │
   │  → russh-keys     │              │                      │
   │  → 签名认证       │              │                      │
   └────────┬─────────┘              └──────────┬───────────┘
            │ 失败                               │ 失败
            ▼                                    ▼
       ┌──────────────────────────────────────────────┐
       │  键盘交互认证 (P3) → UI 弹 challange 框      │
       └──────────────────────────────────────────────┘
                        │ 成功
                        ▼
                ┌──────────────────┐
                │  认证完成 / Ready │
                └──────────────────┘
```

### 主机密钥验证流程

```
check_server_key(key)
  ├─ 查找 known_hosts → 不存在 → KeyVerificationRequired
  │   ├─ 接受 → save_host_key() → true
  │   └─ 拒绝 → false
  ├─ 存在 + 匹配 → true (静默)
  └─ 存在 + 不匹配 → KeyVerificationChanged
      ├─ 确认 → update_key() → true
      └─ 拒绝 → false
```

---

## Config 模块内部设计

### 核心类型

```rust
/// SSH 连接配置 Profile
pub struct Profile {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,             // 默认 22
    pub username: String,
    pub auth_method: AuthMethod,  // 密码/密钥/KBI
    pub group: Option<String>,
    pub sort_order: u32,
}

/// 认证方式（配置存储用，不含明文密码）
pub enum StoredAuthMethod {
    Password,                    // 密码（DPAPI 加密存储）
    Key { key_path: String },    // 私钥路径
    KeyboardInteractive,
}

/// 应用全局设置
pub struct AppSettings {
    pub font_family: String,
    pub font_size: f64,
    pub color_scheme: Option<String>,
    pub minimize_to_tray: bool,
    pub save_layout: bool,
    pub scrollback_lines: u32,
    pub auto_update_check: bool,
}

/// 顶层配置
pub struct Config {
    pub profiles: Vec<Profile>,
    pub settings: AppSettings,
    pub window_position: Option<(i32, i32)>,
    pub window_size: Option<(u32, u32)>,
    pub window_maximized: bool,
}
```

### 模块内部依赖

```
config/
├── mod.rs           # 模块导出
│   ├── profile.rs   # Profile 定义 + 序列化
│   ├── settings.rs  # AppSettings 定义 + 序列化
│   └── store.rs     # ConfigStore: 加载/保存/加密集成
│                     #   load() → 读 JSON → DPAPI 解密
│                     #   save() → DPAPI 加密 → 原子写 JSON
│                     #   路径: %APPDATA%/SamSSH/config.json
```

### 配置持久化流程

```
save():
  Config → serde_json → JSON string → DPAPI encrypt → Base64 encode
  → 写入 %APPDATA%/SamSSH/config.json（原子写入）

load():
  读取文件 → Base64 decode → DPAPI decrypt → serde_json → Config
  → 解密失败时加载不包含凭据的纯文本配置
```

---

## Crypto 模块设计

### DPAPI 加密 (crypto/dpapi.rs)

```rust
/// 使用当前用户 DPAPI 加密数据
pub fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>, CryptoError>

/// 使用当前用户 DPAPI 解密数据
pub fn decrypt(ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError>
```

### 加密安全规则

1. **作用域**: `CRYPTPROTECT_CURRENT_SERCURITY_DESCRIPTOR`（仅当前用户可解密）
2. **存储**: Base64 编码字符串，仅加密敏感字段（密码/passphrase）
3. **降级**: 加密失败时保存不包含凭据的纯文本配置，不阻止程序启动
4. **清除**: 内存中明文凭据使用 `zeroize` 模式及时清零
