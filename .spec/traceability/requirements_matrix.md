# SamSSH 全链路追溯矩阵

**Date**: 2026-07-17

> Evidence → Question → Decision → Requirement → Design → Code → Test

---

## 全链路条目

### E001 — 项目状态（零源码）

| 阶段 | 引用 | 说明 |
|------|------|------|
| E | E001 | 新项目，零源码，零测试 |
| Q | Q003 | 如何初始化项目？ |
| D | D003 | 自动搭建 Cargo 骨架 + git init |
| R | (无) | 基础设施决策，无功能需求 |
| Design | (无) | 骨架创建后即可项目初始化 |

### E002 — 需求功能文档

| 阶段 | 引用 | 说明 |
|------|------|------|
| E | E002 | 存在完整需求功能文档（13 域 130+ 功能点） |
| Q | Q002 | MVP 范围如何界定？ |
| D | D002 | MVP = 全部 51 个 P0 功能点 |
| R | R001~R009 | 9 个 Requirement 覆盖全部 P0 功能域 |
| Design | architecture.puml, app_state.puml, module_deps.md | 设计三件套 |

### E003 — 技术栈声明

| 阶段 | 引用 | 说明 |
|------|------|------|
| E | E003 | 技术栈声明：gpui + russh + wezterm-term + DPAPI |
| Q | Q001 | gpui 如何引入？ |
| D | D001 | 从 GitHub (zed-industries/gpui) git 依赖 |
| R | (无) | 基础设施决策，影响 Cargo.toml 配置 |
| Design | module_deps.md | 模块依赖中注明外部 crate |

### E004 — 架构设计描述

| 阶段 | 引用 | 说明 |
|------|------|------|
| E | E004 | 需求文档第 2 节提供完整架构描述 |
| Q | (无) | 文档直接可用，无需盘问 |
| D | (无) | 架构直接转换 |
| R | R001~R009 | 所有 P0 需求 |
| Design | architecture.puml, app_state.puml, module_deps.md | 三件套从架构描述转换而来 |

### E005 — 功能域与优先级清单

| 阶段 | 引用 | 说明 |
|------|------|------|
| E | E005 | 51 个 P0 功能点清单 |
| Q | Q002, Q004 | MVP 范围 + 测试策略 |
| D | D002, D004 | 全部 P0 为 MVP + 标准+集成测试 |
| R | R001~R009 | 9 个 Requirement 分域覆盖 |
| Design | 三件套 | 所有设计覆盖 P0 需求 |

---

## 矩阵统计

| 指标 | 值 |
|------|-----|
| Evidence 记录 | 5 (E001~E005) |
| Decision 记录 | 4 (D001~D004) |
| Requirement 记录 | 9 (R001~R009) |
| Design 三件套 | 3 (architecture.puml, app_state.puml, module_deps.md) |
| 完整 6 层链路 (E→Q→D→R→Design) | 3 条 (E002, E003, E005) |
| 缺失层 | Code（待实现）、Test（待实现） |
