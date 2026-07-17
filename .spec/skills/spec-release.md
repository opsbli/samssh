
# spec-release

## Objective
Verify release readiness. All gates must pass: review, tests, traceability, and zero open spec-grill questions.

---

## Flow
```
Run spec-review → must pass
    ↓
Run tests → must all pass
    ↓
Check traceability → all links complete
    ↓
Check spec-grill → no open questions
    ↓
Check Decisions → all evidence gaps closed
    ↓
Report release readiness
```

---

## Inputs
| Source | Path | Purpose |
|--------|------|---------|
| Review result | (from spec-review) | Pass/fail status |
| Test result | (from test runner) | Pass/fail status |
| Traceability | `.spec/traceability/requirements_matrix.md` | Complete linkage check |
| Decisions | `.spec/decisions/` | All gaps closed check |
| Release checklist | `.spec/operations/release_checklist.md` | Operational readiness |

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Release verdict | (reported to user) | READY / NOT READY with reasons |
| Release notes | (optional) | Summary of what changed |

## Rules
- All gates must pass — no exceptions.
- If review fails, do not release; report blocker issues.
- If tests fail, do not release; report failing tests.
- If open questions exist in spec-grill, do not release; list unanswered questions.
- If any Decision record lacks a corresponding Requirement, do not release.
