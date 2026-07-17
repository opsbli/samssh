
# spec-trace

## Objective
Maintain and validate the V3 full-chain traceability: Evidence → Question → Decision → Requirement → Design → Code → Test. This skill runs periodically and after every lifecycle milestone to ensure no links are missing.

---

## Flow
```
Load all Evidence records E{NNN}...
    ↓
For each E → check Question Q{NNN} exists (forward)
    ↓
For each Q → check Decision D{NNN} exists (forward)
    ↓
For each D → check Requirement R{NNN} exists (forward)
    ↓
For each R → check Design link exists (forward)
    ↓
For each Design → check Code link exists (forward)
    ↓
For each Code → check Test link exists (forward)
    ↓
For each Test → check back-link to Requirement exists (backward)
    ↓
Report: Complete / Missing links report
```

---

## Inputs
| Source | Path | Purpose |
|--------|------|---------|
| Evidence | `.spec/evidence/` | Chain start |
| Questions | (from spec-grill history) | E→Q link |
| Decisions | `.spec/decisions/` | Q→D link |
| Requirements | `.spec/requirements/` | D→R link |
| Designs | `.spec/designs/` | R→Design link |
| Code | `src/main/java/` | Design→Code link |
| Tests | `src/test/java/` | Code→Test + Test→R back-link |
| Current matrix | `.spec/traceability/requirements_matrix.md` | Reference state |

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Updated matrix | `.spec/traceability/requirements_matrix.md` | Full V3 columns |
| Traceability report | (reported to user) | Link status per chain |

## Rules
- V3 traceability has **7 layers**: E → Q → D → R → Design → Code → Test.
- Every layer must have a forward link to the next.
- Tests must also have a backward link to Requirements (bidirectional).
- Missing links are reported with their exact location.
- This skill does **not** create missing artifacts — it only reports them (call the appropriate skill to fill gaps).
- The traceability matrix must be updated at every lifecycle milestone.
