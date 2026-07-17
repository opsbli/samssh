# SamSSH 架构标准

**Source**: docs/需求功能文档.md 第 2 节

---

## 1. 三层架构

```
┌──────────────────────────────────────────────────────────────┐
│                       界面层                                 │
│  [gpui-component](https://github.com/longbridge/gpui-component) 组件树                                       │
│  TabBar │ Panel │ TerminalView │ Tree │ FileManager │ Menu   │
│                       ▲ 状态订阅                              │
│                       ▼                                      │
│  Entity<AppState> — 响应式状态管理                            │
│  • impl Render 声明式构建组件树                               │
│  • 状态变更 → 响应式引擎 → 局部差异化重绘                    │
│  • cx.subscribe() / cx.emit() 事件通信                       │
│  • cx.spawn() 后台异步任务                                   │
├──────────────────────────────────────────────────────────────┤
│                     后台服务层                                │
│  • SSH 连接管理：建立/维护/断开 SSH 会话                     │
│  • SFTP 文件传输：列表/上传/下载/重命名/删除                 │
│  • 主机密钥验证：known_hosts 管理                            │
│  • 键盘交互认证：密码 / 密钥 / OTP                          │
│  • 异步消息通道：与 UI 层通过事件队列通信                    │
└──────────────────────────────────────────────────────────────┘
```

## 2. 核心数据流

```
用户输入 → gpui 事件派发 → 回调修改 Entity<AppState>
    → gpui 响应式引擎触发差异化重绘
    → 后台异步任务(SSH/SFTP)通过消息队列发回结果
    → UI 事件订阅更新状态 → 重绘
    → wgpu 渲染到窗口帧缓冲区
```

## 3. 配置持久化流程

```
用户设置 → 序列化(JSON) → DPAPI 加密 → 原子写入 %APPDATA%\SamSSH\config.json
```

## 4. 模块目录约定

```
src/
├── main.rs              # 入口点
├── app/                 # UI 组件树（gpui Entity + impl Render）
│   ├── components/      # 可复用 UI 组件
│   ├── views/           # 视图层（终端、SFTP、设置）
│   └── state.rs         # AppState Entity 定义
├── ssh/                 # SSH 连接管理
├── sftp/                # SFTP 文件传输
├── config/              # 配置加载/保存/加密
├── terminal/            # 终端模拟封装
├── theme/               # UI/终端主题管理
├── crypto/              # DPAPI 加密
├── tray/                # 系统托盘
└── logger/              # 日志
```
