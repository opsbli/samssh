# SamSSH UI 组件约定

**Source**: docs/需求功能文档.md

---

## 组件命名约定

- 组件以 PascalCase 命名：`SidebarTree`, `TerminalView`, `FileManager`
- Entity 状态以 `AppState`, `SessionState`, `TransferState` 形式命名
- 文件名使用 snake_case：`sidebar_tree.rs`, `terminal_view.rs`

## 核心组件结构

```rust
struct MyComponent {
    // 状态字段
}

impl Render for MyComponent {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        // 声明式 UI
    }
}
```

## 事件通信模式

| 模式 | 方法 | 场景 |
|------|------|------|
| 冒泡事件 | `cx.emit()` / `cx.subscribe()` | 子→父通信 |
| 全局状态 | `Entity<AppState>` 共享 | 跨组件状态共享 |
| 后台消息 | 消息队列 + 事件订阅 | UI↔后台服务通信 |

## 布局组件（gpui-component）

| 组件 | 用途 |
|------|------|
| TabBar | 标签页管理 |
| Tree | 侧边栏连接树 |
| Panel | 面板容器 |
| Dialog | 对话框 |
| Split | 分割面板 |
| ContextMenu | 右键菜单 |
