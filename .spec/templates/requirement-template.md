---
id: R{NNN}
title: "{Title}"
template: full
status: drafted|reviewing|approved
execution_mode: plan|tdd
mode_source: model-selected|user-override
approval: explicit|inferred
owner: "{Owner}"
created_at: "{YYYY-MM-DD}"
updated_at: "{YYYY-MM-DD}"
---

# R{NNN} — {Title}

**Source Decision**: D{NNN}

---

## 1. Background

### Current Problem
- {当前问题 1：用户反馈 / 系统缺陷 / 业务瓶颈}
- {当前问题 2}

### Business Context / User Value
- {业务价值 1：降低流失 / 提升效率 / 合规要求}
- {业务价值 2}

### Why Now
- {驱动因素：用户投诉增多 / 上游系统就绪 / 合规截止期}

---

## 2. Goals & Non-Goals

### Goals
1. {目标 1：明确、可验证、可衡量}
2. {目标 2}

### Non-Goals
1. {明确不做的事项，如：本期不改数据库 schema}
2. {不做国际化}
3. {不做后台运营功能}

---

## 3. Users & Scenarios

### Target Users
- {用户角色 1：如普通用户、管理员、财务人员}
- {用户角色 2}

### Core Scenarios
1. 作为 **{角色}**，我希望 **{行为}**，从而 **{价值}**
2. 作为 **{角色}**，我希望 **{行为}**，从而 **{价值}**

### Edge / Exceptional Scenarios
- {边界场景 1：输入为空 / 重复提交 / 资源不存在}
- {边界场景 2：权限不足 / 网络异常 / 数据过大}

---

## 4. Business Rules

### Rules
1. {业务规则 1：必须遵守的业务逻辑}
2. {业务规则 2}
3. {业务规则 3}

### Assumptions
- {假设 1：默认成立的前提，如：已有邮件服务}
- {假设 2}

### Constraints
- {约束 1：不可违反的限制，如：不得修改老接口协议}
- {约束 2：必须兼容现有数据结构}

---

## 5. 技术目标

| 项 | 内容 |
|---|------|
| 目标能力 | {系统完成后具备什么能力} |
| 关键约束 | {一致性、性能、权限、兼容性等硬约束} |
| 成功信号 | {可观测、可验证的成功结果} |
| 风险等级 | low / medium / high |
| 执行模式 | plan / tdd |

### 本期范围
{模块 / 边界} | {本期交付}
---|---

### 非本期范围
- {明确不做的能力}

---

## 6. Functional Requirements

1. {功能需求 1：系统必须做什么}
2. {功能需求 2}
3. {功能需求 3}

---

## 7. Behavior Spec

### Happy Path
1. {步骤 1：用户动作 → 系统处理 → 系统结果}
2. {步骤 2}
3. {步骤 3}

### Alternative Flows
#### Flow A: {分支名称}
1. {步骤 1}
2. {步骤 2}

### Failure Flows
#### Failure 1: {失败场景}
- **Trigger**: {触发条件}
- **System Behavior**: {系统行为}
- **User Visible Result**: {用户可见结果}

---

## 8. Context Basis

| 来源 | 已采用事实或约束 | 对方案的影响 |
|------|-----------------|-------------|
| 用户输入 | {事实} | {影响} |
| 代码 / 测试 | {事实} | {影响} |
| 现有决策 | D{NNN} | {影响} |
| 待确认 | {问题或 None} | {影响} |

---

## 9. Core Invariants

| 编号 | 不变量 | 验证方式 |
|------|--------|---------|
| INV-1 | {任何时候都必须成立的系统约束} | {测试 / gate / 审计} |

### Data Consistency

| 场景 | 策略 |
|------|------|
| 并发写 | {锁 / CAS / 事务 / N/A} |
| 幂等 | {幂等键和重复请求行为} |
| 失败补偿 | {回滚 / 补偿任务 / N/A} |

---

## 10. API / Contract Spec

### Endpoint 1
- **Method**: GET / POST / PUT / DELETE
- **Path**: {接口路径}
- **Auth**: {是否需要认证}
- **Idempotency**: {幂等说明}

### Request Fields
- `field1`: {类型}，{是否必填}，{说明}

### Response Fields
- `field1`: {类型}，{说明}

### Error Codes
- `ERROR_CODE_1`: {说明}

---

## 11. 安全与权限

### Permission Rules
- {谁可以访问}
- {谁不能访问}

### Security Requirements
- {安全要求 1}
- {安全要求 2}

---

## 12. Acceptance Criteria

### Checklist
- [ ] {验收项 1}
- [ ] {验收项 2}
- [ ] {验收项 3}

### Scenario-Based

#### Scenario 1: {场景名称}
**Given** {前置条件}
**When** {用户动作}
**Then** {系统结果}

#### Scenario 2: {场景名称}
**Given** {前置条件}
**When** {用户动作}
**Then** {系统结果}

---

## 13. Verification Plan

| 验证项 | 必需 | 证据 |
|--------|------|------|
| 构建 | yes | {命令} |
| 单元测试 | yes | {范围} |
| 集成测试 | conditional | {环境} |
| 验收覆盖 | yes | AC 覆盖率 |
| Spec vs Code | yes | spec-review |

---

## 14. Execution Strategy

- **Mode**: plan / tdd
- **Reason**: {选择原因}
- **Upgrade**: plan → tdd 允许（发现风险时自动升级）
- **Downgrade**: tdd → plan 需用户显式 override

---

## 15. Risks / Open Questions

### Risks
- {风险 1：性能 / 上线 / 依赖不稳定}

### Open Questions
- {待确认问题 1}

---

## 16. Traceability

```
E{NNN} → Q{NNN} → D{NNN} → R{NNN}
```
