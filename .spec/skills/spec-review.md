
# spec-review

## Objective
Perform a consistency review of the entire V3 chain: Evidence → Question → Decision → Requirement → Design → Code → Test. All links must be complete and correct.

---

## Flow
```
For each Requirement R{NNN}:
    ↓
  Check: Has Decision D{NNN}? (back-link)
    ↓
  Check: Has Design link? (forward-link)
    ↓
  Check: Has Code implementation? (forward-link)
    ↓
  Check: Has Test coverage? (forward-link)
    ↓
  Check: Spec doc vs Code consistency
    ↓
  Check: Evidence file exists for each Decision D{NNN} (E{NNN}.md in `.spec/evidence/`)
    ↓
  Check: If micro-fix or skipped-phase path, does D contain "Skipped phases" section with justification?
    ↓
For each gap found → report as issue
    ↓
If gaps exist → call spec-grill to resolve
    ↓
If review passes → generate 验证摘要:
  ├─ AC 覆盖: {N/M pass}
  ├─ 构建: {pass/fail}
  ├─ 测试: {pass/fail}
  ├─ Spec vs Code: {consistent/inconsistent}
  ├─ 风险残留: {none / list}
  └─ 知识候选: {K{NNN} extracted / none}
  ├─ E 文件完整性: {all created / missing: E{NNN}}
  └─ D 跳过理由: {all documented / missing in D{NNN}}
    ↓
    若有知识候选 → 写入 `.spec/knowledge/K{NNN}.md` + 更新索引
```

---

## Inputs
| Source | Path | Purpose |
|--------|------|---------|
| Requirements | `.spec/requirements/` | Check forward/back links |
| Decisions | `.spec/decisions/` | Verify every D has a corresponding R |
| Evidence | `.spec/evidence/` | Verify every E has a corresponding Q and D |
| Designs | `.spec/designs/` | Verify design coverage |
| Code | `src/main/java/` | Verify code coverage |
| Tests | `src/test/java/` | Verify test coverage |
| Traceability | `.spec/traceability/requirements_matrix.md` | Verify matrix accuracy |

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Review report | (reported to user) | List of passed checks + gaps found |
| Issues list | (reported to user) | Each issue with severity (blocker / warning / info) |
| 验证摘要 | (reported to user) | AC覆盖 / 构建 / 测试 / Spec一致性 / 风险残留 |
| 知识候选 | `.spec/knowledge/K{NNN}.md` | 可复用经验（按 review 判断提取） |

## Rules
- Check **every** link in the V3 traceability chain.
- Missing links are **blockers** — do not pass review.
- Ambiguous links are **warnings** — flag but may pass with note.
- **Spec doc vs Code 一致性检查** — For each Decision D{NNN} that modifies existing business rules:
  - Identify the corresponding main requirement specification document in `.spec/requirements/{module}.md`
  - Check whether the document's business rule descriptions still match the implemented behavior
  - If the document describes the **old** behavior (not updated for D{NNN}), this is a **blocker** — must update the spec doc before passing review
  - This check also extends to API design documents in `.spec/designs/{module}/spec/` — field descriptions that are now inaccurate are blockers
- If the review passes, update release readiness in traceability.
- If the review fails, call spec-grill for each blocker gap.
- **E 文件完整性检查** — For each Decision D{NNN}, verify that a corresponding E{NNN}.md exists in `.spec/evidence/`. Missing E file is a **blocker** — the evidence must be captured before review passes.
- **D 跳过理由检查** — If the current task uses the 微修复 path or any path that skips phases (grill/sync/trace), each D{NNN} must contain a `Skipped phases` section listing each skipped phase and its 豁免理由. Missing justifications are a **blocker**.
