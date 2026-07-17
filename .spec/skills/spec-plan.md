
# spec-plan

## Objective
Generate an implementation plan from approved Requirements. The plan must be traceable back to Requirements and Decisions, with clear task breakdown, risk assessment, and ordering.

---

## Flow
```
Load Requirements R{NNN}...
    ↓
Load related Decisions D{NNN}...
    ↓
Break down into implementation tasks
    ↓
Order tasks by dependency and risk
    ↓
Add traceability links (task → requirement)
    ↓
Report plan
```

---

## Inputs
| Source | Path | Purpose |
|--------|------|---------|
| Requirements | `.spec/requirements/` | What to implement |
| Decisions | `.spec/decisions/` | Design choices affecting implementation order |
| Designs | `.spec/designs/` | Architecture to implement |
| Plan template | `.spec/templates/plan-template.md` | Structure to follow |

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Implementation plan | 临时产物，通过 `todo_write` 输出并等待用户审批，执行完后不持久化到 `.spec/` | Todo / Doing / Done / Risks per plan template |
| Traceability links | `.spec/traceability/requirements_matrix.md` | Plan → Requirement links |

## Rules
- Every task must trace to at least one Requirement.
- Tasks that depend on unresolved spec-grill questions must be marked `blocked`.
- Risk assessment is mandatory for each task.
- Use the plan template structure: Todo, Doing, Done, Risks.
- **Plans are ephemeral**: output via `todo_write` for user approval, execute, then discard. Do not persist plan files to disk — they are not tracked in `.spec/`.
