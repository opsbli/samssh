
# AGENTS.md — Spec Constitution

---

## 架构
Rust (edition 2021), gpui (GUI 框架), gpui-component (UI 组件库), wgpu (渲染引擎), russh (SSH 协议), wezterm-term (终端模拟), Windows DPAPI (加密), Windows 10/11 (DX12/Vulkan)

---

## 入口

收到任何非平凡需求后：

```
1. 读 AGENTS.md（本文件）—— 遵守所有规则
2. 判断任务类型 → 查下方 Skill 调用矩阵，选定生命周期路径
3. 进入 plan mode —— 只读探索，不写文件
4. 输出分阶段实施计划 —— 等待用户审批
   — 计划中必须显式声明所选路径名称（完整/快速/维护/入门/追溯）
     和完整的 phase 序列，锁定后不可变更
5. 审批通过后按生命周期逐步执行
```

### ⚡ Skill 调用矩阵 — 按任务类型自动触发

| 任务类型 | 调用链 | 说明 |
|---------|--------|------|
| **新功能开发** | `spec-discovery → spec-grill → spec-confirm → spec-requirement → spec-design → spec-plan → spec-implement → spec-test → spec-trace → spec-review → spec-release` | 全生命周期，10 个 phase |
| **文档补齐** | `spec-discovery → spec-grill → spec-confirm → spec-sync → spec-trace → spec-review` | 快速路径，跳过编码/测试 |
| **代码维护/修复** | `spec-discovery → spec-grill → spec-confirm → spec-implement → spec-test → spec-sync → spec-trace → spec-review` | 跳过设计和计划 phase |
| **微修复（≤5行核心逻辑）** | `spec-discovery → spec-confirm → spec-implement → spec-test → spec-review` | 跳过 grill/sync/trace，仅适用于 ≤5行核心逻辑变更、无新功能 |
| **项目入门** | `spec-bootstrap → spec-discovery → spec-grill → spec-confirm → spec-requirement → spec-design → spec-trace` | 一次性初始化 |
| **追溯检查** | `spec-trace → spec-review` | 只检查，不修改 |

> **规则**: 每个 skill 完成后自动调用链中下一个 skill，直到链结束或遇到 blocker。

### 🔒 路径锁定规则
- **路径选定后锁定**：agent 必须在 plan 中显式输出所选路径名称和完整 phase 序列，锁定后不可变更
- **路径变更须重新审批**：如需变更路径，必须暂停当前执行，输出新路径并等待用户审批
- **门禁驱动执行**：每个 phase 完成后必须验证通往下一 phase 的门禁（见下方门禁规则），门禁未通过即 blocker，不得调用链中下一个 skill

### ⚡ 风险路由 — Plan vs TDD

在 spec-grill 阶段完成后，须根据风险信号选择 implementation 的执行强度：

| 条件 | 路由 | 额外要求 |
|------|------|---------|
| 文档/配置/简单功能/已有测试覆盖充分 | **plan 模式** | 按现有路径执行 |
| bug fix / 权限 / 安全 / 资金 / 状态机 / 并发 / 幂等 / 迁移 / 历史回归 | **tdd 模式** | implement 前必须先写红灯测试（RED test→GREEN test→refactor） |
| 不确定 | auto（由 agent 判断，允许 plan→tdd 升级） | 发现风险信号时自动升级，无需审批 |

**规则**：
- 允许 `plan → tdd` 升级（发现风险时自动升级）
- 禁止静默 `tdd → plan` 降级（需用户显式 override 并记录风险到 Decision）
- 风险路由结果须写入对应 Requirement 的 `execution_mode` 字段和 `执行策略` 章节

---

## Grill Rules — HIGHEST PRIORITY
When evidence is insufficient, these rules bind all agents absolutely:

- **禁止推断业务规则** — Never infer a business rule from code or convention.
- **禁止推断状态机** — Never infer state transitions or workflow logic.
- **禁止推断数据库含义** — Never infer the meaning of a column, table, or relationship from names alone.
- **禁止推断UI行为** — Never infer what a button or page should do from design alone.
- **必须调用 spec-grill** — When any of the above would be needed, call `spec-grill` immediately.
- **必须等待问题关闭** — Do not proceed until all open questions from spec-grill are closed with a Decision record.
- **复用已有 Decision** — Before asking a new question, search `.spec/decisions/` for existing Decisions covering the same pattern. If a parallel or symmetric scenario exists, reuse the established principle instead of re-grilling. The reused Decision must be cited in the new Requirement.
- **提供推荐答案** — For each question, provide your recommended answer with reasoning before asking. This gives the user an anchor to agree with or correct.
- **事实优先** — Before asking a question, exhaust all available sources (code, database schema, existing docs) to answer it yourself. If the answer can be found, record it as Evidence instead of asking.
- **决策树走查** — Walk the decision tree branch by branch, resolving dependencies between decisions one-by-one. A decision may expose sub-branches that must also be resolved before moving to the next top-level branch.
- **类别覆盖检查** — The grilling session must cover at least **3 of the 8 categories** in the spec-grill skill checklist. If fewer than 3 are covered, actively expand into uncovered categories. Categories irrelevant to the task must be noted as "not applicable" in a Decision record.
- **共享理解确认** — After all questions are resolved, present a summary of all Decisions and ask the user to confirm shared understanding before proceeding. If the user disagrees, reopen the relevant branch.

---

## Evidence Rules
- Every decision must be grounded in one or more pieces of **Evidence**.
- Evidence sources (ordered by priority):
  1. **Requirements** — written business truth (`.spec/requirements/`)
  2. **Designs** — architecture, API, DB, domain models (`.spec/designs/`)
  3. **User statements** — direct answers from user interactions
  4. **Code** — existing implementation (lowest priority, may be stale)
- Evidence must be captured in a structured record before asking a question.
- Evidence ID format: `E001`, `E002`...
- **下游影响扫描** — spec-discovery must trace 1-2 levels downstream for each evidence gap to identify cascading fix points before closing the discovery phase. Findings must be recorded in the Evidence or noted as potential follow-up items.

---

## Question Rules
- Questions must be single, atomic, and answerable with a short answer.
- Never ask compound questions (split into multiple rounds).
- Every question must cite the Evidence that triggered it.
- Question ID format: `Q001`, `Q002`...

---

## Decision Rules
- Every user answer that closes a question becomes a **Decision**.
- Decisions are persisted — they never expire.
- Decision ID format: `D001`, `D002`...
- Decision records include:
  - **Context** — what was the situation
  - **Evidence** — what triggered the question
  - **Question** — what was asked
  - **Answer** — what the user said
  - **Impact** — which requirements/rules/designs change as a result
- Before asking the same or similar question, check existing Decision records.
- Decision records live in `.spec/decisions/`

---

## Requirement Rules
- Requirements are business truth derived from Decisions.
- Every requirement must trace to at least one Decision.
- Every requirement must have ID: `R001`, `R002`...
- No requirement may be inferred — must come from a user-confirmed Decision.

---

## Design Rules
- `architecture.puml` 是架构真理（PlantUML 组件/架构图）
- `app_state.puml` 是状态树真理（gpui Entity<T> 状态结构）
- `module_deps.md` 是模块依赖真理
- No coding before design update.
- Every design element must trace to a Requirement.

---

## Code Rules
- Rust 命名规范：snake_case 函数/变量，PascalCase 类型
- `Entity<T>` / `Model<T>` + `impl Render` 是 gpui 组件标准模式
- `cx.spawn()` 用于后台异步任务
- `cx.subscribe()` / `cx.emit()` 用于事件通信
- SSH/SFTP 网络 I/O 通过后台异步任务 + 消息队列与 UI 层通信
- 凭据使用 Windows DPAPI 加密存储
- `tracing` crate 用于日志

---

## Review Rules
- **对称性检查** — When modifying one side of a symmetric pair (upload/download, connect/disconnect, lock/unlock, show/hide), the review must audit the counterpart for the same class of issue. If found, either fix it in the same change or record a follow-up Decision.

---

## Testing Rules
- Every business rule has a test
- Given / When / Then format in test comments
- Every test must trace to a Requirement
- **核心方法测试覆盖** — When modifying a method that has zero direct test coverage, the implement phase MUST include adding or updating unit tests covering its boundary conditions.

---

## Traceability Rules — V3
- **Full chain**: Evidence → Question → Decision → Requirement → Design → Code → Test
- Every artifact in the chain must be linkable to its predecessor.
- Missing links are blockers.

| Link | Direction | Required |
|------|-----------|----------|
| Evidence → Question | forward | yes |
| Question → Decision | forward | yes |
| Decision → Requirement | forward | yes |
| Requirement → Design | forward | yes |
| Design → Code | forward | yes |
| Code → Test | forward | yes |
| Test → Requirement | backward | yes |

---

## Lifecycle Orchestration — 自动调用顺序

收到需求后，Agent 必须按以下生命周期自动调用 Skills。**链内所有 phase 强制按序执行，不可跳过，不可重排。**

### 🔄 完整生命周期（按顺序调用）

```
入口：读 AGENTS.md + plan mode 等待审批
  │
  ▼
[Phase 1 — 发现]  spec-discovery
  │  扫描所有来源，创建 Evidence 记录
  ▼
[Phase 2 — 盘问]  spec-grill
  │  每次一问，直到无未决问题
  ▼
[Phase 3 — 确认]  spec-confirm + spec-requirement
  │  将回答转为 Decision → Requirement
  ▼
[Phase 4 — 设计]  spec-design
  │  Update architecture.puml / app_state.puml / module_deps.md
  ▼
[Phase 5 — 计划]  spec-plan
  │  生成实施计划，等待用户审批
  ▼
[Phase 6 — 编码]  spec-implement
  │  按设计编码，不得超出 Requirement 范围
  ▼
[Phase 7 — 测试]  spec-test
  │  每条业务规则一个 Given/When/Then 测试
  ▼
[Phase 8 — 追溯]  spec-trace
  │  验证全链条 E→Q→D→R→Design→Code→Test 完整
  ▼
[Phase 9 — 审查]  spec-review
  │  一致性审查，无 blocker 方可继续
  ▼
[Phase 10 — 发布] spec-release
  │  所有门禁通过后报告发布就绪
```

> ⚠️ **跳过申请**：若因任务特性某 phase 确定不适用，必须：① 引用 Evidence 证明该 phase 不适用；② 生成 Decision 记录（D{NNN}）说明理由和影响范围；③ 等待用户审批该跳过申请；④ 审批通过后方可跳过。**未经用户审批的跳过视为违规，追溯时标记为 blocker。**

### ⚡ 快速路径（仅文档/非代码变更）
```
1. spec-discovery → 2. spec-grill → 3. spec-confirm → 4. spec-sync → 5. spec-review
```

### 🛠 维护路径（代码已存在，需同步文档）
```
1. spec-bootstrap (一次性) → 2. spec-discovery → 3. spec-grill → 4. spec-sync → 5. spec-trace
```

### 🔧 微修复路径（≤5行核心逻辑，无新功能）
```
1. spec-discovery → 2. spec-confirm → 3. spec-implement → 4. spec-test → 5. spec-review
```

### 门禁规则
- **入口门禁（Phase 0→1）**：必须声明所选路径并锁定，否则不得开始执行
- **Phase 1→2**: 必须有至少一个 Evidence 记录
- **Phase 2→3**: 所有 Question 有 Answer
- **Phase 3→4**: 所有 Decision 有对应 Requirement
- **Phase 4→5**: 设计文件与 Requirement 可追溯
- **Phase 5→6**: Plan 已审批；实际执行的 phase 序列与锁定路径一致
- **Phase 6→7**: 代码与设计一致
- **Phase 6→7 (增强门禁)**: 若被修改的方法零测试覆盖，testing phase 必须附带新增边界条件测试
- **Phase 7→8**: 测试通过
- **Phase 8→9**: 全链路无缺失链接
- **Phase 9→10**: 无 blocker 问题；密钥泄露扫描无高/中风险
- **跳过门禁**：任何跳过的 phase 必须有用户审批的 D{NNN} 记录且状态为 approved，否则 blocker
- **Phase 转换一致性**：每次 phase 转换时校验当前 phase 是否在锁定路径中，偏离即 blocker

---

## Approval Gate — 审批门禁

Agent 每次启动工作前：

```
1. 读 AGENTS.md — 遵守本宪法
2. 评估任务 → 查 Skill 调用矩阵，确定走哪个生命周期路径（完整/快速/维护/入门/追溯）
3. 进入 plan mode — 只读探索，不写任何文件
4. 输出分阶段实施计划（PHASES + 子步骤）
   — 必须显式声明所选路径名称和完整 phase 序列，锁定后不可变更
5. 等待用户审批 → 审批通过后方可开始执行
```

### 禁止行为（无审批时）
- ❌ 写入或修改任何文件
- ❌ 运行非读取性 shell 命令
- ❌ 调用 spec-grill 提问（应在 plan mode 中提出）
- ✅ 允许：读取文件、grep、glob 探索——只读操作

---

## Agent 入口激活语

每次进入新会话或收到新需求时，Agent 必须按以下顺序激活：

```
步骤 1 → 读 AGENTS.md（本文件）—— 遵守所有规则
步骤 2 → 判断任务类型（新功能 / 修复 / 文档 / 维护）
步骤 3 → 确定生命周期路径（完整 / 快速 / 维护 / 入门 / 追溯）
          — 输出所选路径名称 + 完整 phase 序列到计划中，锁定后不可变更
步骤 4 → 运行 spec-discovery —— 扫描现状、发现证据缺口
步骤 5 → 运行 spec-grill —— 逐个提问关闭所有缺口
          — 若 agent 认为无缺口，须输出 NoGapReport 经用户确认后方可跳过
步骤 6 → 输出计划 → 等待用户审批
步骤 7 → 审批通过后按生命周期逐步执行
步骤 7a → 每完成一个 phase，执行对应的门禁检查清单
步骤 7b → 门禁通过 → 报告结果并进入下一 phase
步骤 7c → 门禁未通过 → 输出 blocker 报告，暂停，等待用户指令
步骤 7d → 每次 phase 转换时校验当前 phase 是否在锁定路径中，偏离即 blocker
```

> **每次需求会话都以 `spec-discovery` 开始，以 `spec-release` 结束。**

---

## Release Rules
- `spec-review` pass
- Tests pass
- Traceability updated
- No open questions in spec-grill
- All Decisions persisted
