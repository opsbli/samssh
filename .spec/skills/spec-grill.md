
# spec-grill

## Objective
When evidence is insufficient, systematically identify gaps by **walking the decision tree** — one branch at a time, with recommended answers, until every open question is resolved and shared understanding is confirmed.

This skill is the **highest priority** in the V3 lifecycle. It must be invoked before any inference about business rules, state machines, database semantics, or UI behavior.

---

## Flow (4-Step Process)

```
spec-grill 开始
  │
  ├── 步骤 1: 事实查找
  │     遍历上下文，查询所有可查证的事实
  │     标记无需盘问的问题
  │
  ├── 步骤 2: 决策树展开 + 分类覆盖检查
  │     列出所有未解决决策点，标注依赖关系
  │     按盘问类别清单核对覆盖完整性
  │
  ├── 步骤 3: 单问 + 推荐答案
  │     每个决策点:
  │       a) 展示证据
  │       b) 提供推荐答案 + 理由
  │       c) 等待用户同意或修正
  │       d) 记录 Decision
  │       e) 展开子分支
  │
  └── 步骤 4: 共享理解确认
        展示所有 Decision 摘要
        等待用户确认理解一致
```

---

## Inputs

| Source | Path | Purpose |
|--------|------|---------|
| Requirements | `.spec/requirements/` | Check what is already decided |
| Designs | `.spec/designs/` (db_schema.sql, api_swagger.yaml, domain_model.puml) | Check what is designed |
| Code | `src/` | Check existing implementation (lowest priority) |
| Tests | `src/test/` | Check existing test coverage |
| Decisions | `.spec/decisions/` | Avoid re-asking closed questions |
| Traceability | `.spec/traceability/requirements_matrix.md` | Identify missing links |

---

## Outputs

| Artifact | Location | Format |
|----------|----------|--------|
| Evidence record | `.spec/evidence/E{NNN}.md` | Evidence template |
| Question | (prompted to user) | Atomic, single, cites evidence, **includes recommendation** |
| Decision record | `.spec/decisions/D{NNN}.md` | Decision-record template |
| Updated requirements | `.spec/requirements/` | Per requirement template |
| Updated traceability | `.spec/traceability/requirements_matrix.md` | V3 matrix |

---

## Rules

### Before questioning (步骤 1 + 2)

1. **Read ALL input sources first** — do not ask about something already documented.
2. **Check existing Decision records** — if a closed decision covers the question, do not re-ask. 复用原则（D500）: 若当前场景是前序 Decision 的对称/类比场景，自动沿用该原则。
3. **事实优先** — 凡是能从代码、数据库、文档中查证的事实，**必须查证而非盘问**。例如：数据库字段含义查 `mysql_schema`，API 行为查代码，SQL 约束查 DDL。
4. **Generate an Evidence record** (E{NNN}.md) for each gap or contradiction found.
5. **Build the decision tree** — list all unresolved decision points and their dependency graph. A decision's answer may open sub-branches that must also be resolved.
6. **类别覆盖检查** — 对比盘问类别清单，确保已覆盖至少 **3 个不同类别**。若不足，主动补充该类别的问题。
7. If no gaps exist after all checks, report "No open questions" and exit with a NoGapReport.

### 盘问类别清单

每次 spec-grill 必须至少覆盖以下 8 个类别中的 **3 个**，每个类别至少 1 个问题：

| # | 类别 | 示例问题 | 必查场景 |
|---|------|---------|---------|
| 1 | **业务规则** | 取消订单后库存是否回滚？ | 涉及数据变更 |
| 2 | **状态迁移** | 从"待审核"可以到哪些状态？ | 涉及状态机 |
| 3 | **错误处理** | 库存不足时返回什么？ | 涉及外部依赖/资源检查 |
| 4 | **边界条件** | 打卡时间等于 shiftStart 算迟到吗 | 涉及时间/数值比较 |
| 5 | **非功能性** | 响应时间有要求吗？ | 涉及性能/并发 |
| 6 | **权限模型** | 哪些角色可以操作？ | 涉及菜单/按钮权限 |
| 7 | **数据一致性** | 并发写入会重复吗？ | 涉及数据库写入 |
| 8 | **外部依赖** | 是否需要同步到第三方？ | 涉及外部系统 |

> 注：如果任务类型确实不涉及某些类别（例如纯文档变更），可在 Decision 记录中注明"不适用"并说明理由。

### During questioning (步骤 3)

1. **Ask ONE question at a time** — never compound. Asking multiple questions at once is bewildering.
2. Every question must cite its triggering Evidence ID.
3. **Provide your recommended answer** with reasoning. This gives the user an anchor to agree with or correct — much faster than asking them to think from scratch.
4. Questions must be answerable with a short answer or a choice from 2-4 options.
5. **Walk the decision tree** — resolve dependencies between decisions one-by-one. A decision may expose sub-branches; explore those before moving to the next top-level branch.
6. Do not proceed to the next question until the current one is answered.

### After answering

1. Create a Decision record (D{NNN}.md) with full context.
2. Update requirements, business rules, use cases, or designs as directed by the answer.
3. Update traceability matrix — link Evidence → Question → Decision → Requirement.
4. If more questions remain (including sub-branches from the current decision), loop back to "Ask ONE question".
5. If no more questions, proceed to Step 4.

### Step 4: 共享理解确认

1. **Summarize** all Decisions made in a concise list:
   ```
   D501: 取消订单库存不回滚
   D502: 待审核状态可迁移至"已通过"和"已驳回"
   D503: 库存不足时返回 400 + "库存不足"
   ```
2. **Ask the user to confirm**: "以上 Decision 是否与您的理解一致？"
3. Only when the user confirms "一致" or equivalent, proceed to the next lifecycle phase.
4. If the user disagrees, reopen the relevant branch and continue grilling.

---

## Question Format

```
## Q{NNN} — {Short Title}

**Evidence**: E{NNN} — {brief description of evidence/gap}

**Recommended**: {your recommended answer with reasoning}

**Question**: {single atomic question}

**Options** (if applicable):
- {Option A (recommended)}
- {Option B}
- {Option C}

**Context**: {brief situation — what are we building, what's the ambiguity}
```

---

## Exit Criteria
- ✅ 步骤 4 用户已确认"理解一致"
- ✅ All identified gaps have been resolved with Decision records.
- ✅ 至少覆盖 3 个盘问类别（或不适用项已注明理由）
- ✅ Requirements / designs / traceability are updated.
- ✅ Summary of all Decisions is reported and confirmed.
