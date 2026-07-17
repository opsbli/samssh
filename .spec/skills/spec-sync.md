
# spec-sync

## Objective
Synchronize documentation and traceability with the current state of code after implementation. This is a **reverse sync** — code changes may have drifted from design docs during implementation, and this skill reconciles them.

---

## Flow
```
Scan current code state
    ↓
Compare against designs (db_schema, api, domain)
    ↓
For each discrepancy:
    ↓
  → If code matches a Decision → update design to match
    ↓
  → If code has undocumented change → flag as evidence gap
    ↓
Update designs to match confirmed decisions
    ↓
审查主需求规格说明书
  → 遍历本次变更涉及的新增 Decision D{NNN}
  → 若 D 修改了既有的业务规则（非全新功能），检查对应的主需求规格说明书 `.spec/requirements/{module}.md` 是否需同步更新
  → 若需更新，在本次 sync 中一并修改
    ↓
Update traceability matrix
    ↓
Report all changes made
```

---

## Inputs
| Source | Path | Purpose |
|--------|------|---------|
| Source code | `src/main/java/` | Current implementation |
| Database schema | `.spec/designs/db_schema.sql` | To reconcile |
| API spec | `.spec/designs/api_swagger.yaml` | To reconcile |
| Domain model | `.spec/designs/domain_model.puml` | To reconcile |
| Decisions | `.spec/decisions/` | Authoritative decisions |
| Requirements | `.spec/requirements/` | What should be true |

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Updated designs | `.spec/designs/` | Reconciled with code |
| Evidence gaps | `.spec/evidence/E{NNN}.md` | For undocumented changes |
| Updated traceability | `.spec/traceability/requirements_matrix.md` | Synced links |

## Rules
- Code is **not** automatically truth — if code differs from a Decision, the Decision takes precedence (file a gap).
- If code differs from designs and there is **no** Decision authorizing it, flag as evidence gap (do NOT update designs).
- Never silently overwrite a design to match code — always check Decisions first.
- Report every sync action taken (which files changed and why).
