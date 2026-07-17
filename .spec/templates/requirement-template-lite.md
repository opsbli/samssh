---
id: R{NNN}
title: "{Title}"
template: lite
status: approved
execution_mode: plan|tdd
mode_source: model-selected
approval: explicit
---

# R{NNN} — {Title}

**Source Decision**: D{NNN}

---

## 1. Problem

{一句话描述要修的问题或要做的功能}

---

## 2. Business Rules

- {业务规则 1}
- {业务规则 2}

### Constraints
- {约束 1}
- {约束 2}

---

## 3. Acceptance Criteria

| 编号 | 前置条件 | 操作 | 期望结果 | 验证方式 |
|------|---------|------|---------|---------|
| AC-1 | {Given} | {When} | {Then} | {Test / gate} |

---

## 4. Core Invariants

| 编号 | 不变量 | 验证方式 |
|------|--------|---------|
| INV-1 | {约束} | {测试} |

---

## 5. Execution Strategy

- **Mode**: plan / tdd
- **Reason**: {低风险 / bug fix / 简单过滤}

---

## 6. Traceability

```
E{NNN} → D{NNN} → R{NNN}
```
