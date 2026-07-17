
# spec-requirement

## Objective
Transform verified Decisions into structured, traceable Requirements. This runs after spec-confirm has persisted a Decision — it codifies the business truth.

---

## Flow
```
Load Decision D{NNN}
    ↓
Identify impacted requirement area(s)
    ↓
Assess risk → 确定 execution_mode (plan|tdd)
    ↓
Create or update Requirement R{NNN}.md
  - 根据任务类型选择模板版本：
    · 新功能开发 / 文档补齐 → 使用 full 模板（`requirement-template.md`，16节）
    · 代码维护/修复 / 微修复 → 使用 lite 模板（`requirement-template-lite.md`，6节）
  - 在 frontmatter 中写入 id/title/template/status/execution_mode/mode_source/approval
    ↓
Link Decision → Requirement in traceability
    ↓
Update business rules if applicable
```

---

## Inputs
| Source | Path | Purpose |
|--------|------|---------|
| Decision | `.spec/decisions/D{NNN}.md` | The business truth to codify |
| Existing requirements | `.spec/requirements/` | Area to update or extend |
| Business rules | `.spec/business-rules/` | Rules affected by the decision |

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Requirement | `.spec/requirements/R{NNN}.md` | full 模板（16节）或 lite 模板（6节），根据任务类型选择 |
| Updated business rules | `.spec/business-rules/BR{NNN}.md` | New or modified rules |
| Traceability link | `.spec/traceability/requirements_matrix.md` | D→R link added |

## Rules
- Every Requirement must trace to at least one Decision (D{NNN}).
- Every Requirement must use the appropriate template:
  - **新功能开发 / 文档补齐** → 使用 full 模板（`.spec/templates/requirement-template.md`，16节，含 Background/Goals/Users/BusinessRules/Scenarios/AC）
  - **代码维护/修复 / 微修复** → 使用 lite 模板（`.spec/templates/requirement-template-lite.md`，6节，仅 Problem/BusinessRules/AC/Invariants/Strategy）
- Business rules extracted from the Decision must be stored in `.spec/business-rules/`.
- Requirements are **never** inferred — always derived from a user-confirmed Decision.
- When a Decision supersedes a previous one, update the affected Requirement (set `supersedes: D{OLD}` in the linked Decision context).
- `execution_mode` 字段根据 AGENTS.md 的风险路由规则填写：低风险=plan，高风险=tdd，不确定=auto。
