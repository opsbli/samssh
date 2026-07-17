
# spec-test

## Objective
Generate and maintain tests that verify every business rule and requirement. Every test must trace back to a specific Requirement and its underlying Decision.

---

## Flow
```
Load Requirement R{NNN}
    ↓
Load Business Rules BR{NNN}...
    ↓
Load related Decision D{NNN} (for context)
    ↓
For each business rule → create test case
    ↓
Write test in Given / When / Then format
    ↓
Update traceability (Code → Test links + Test → Requirement back-link)
```

---

## Inputs
| Source | Path | Purpose |
|--------|------|---------|
| Requirements | `.spec/requirements/` | Acceptance criteria to test |
| Business rules | `.spec/business-rules/` | Rules to verify |
| Decisions | `.spec/decisions/` | Behavioral context |
| Existing tests | `src/test/java/` | Current test coverage |

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Test cases | `src/test/java/` | JUnit / Spring Boot Test |
| Traceability | `.spec/traceability/requirements_matrix.md` | Code→Test + Test→Requirement links |
| Test case document | `.spec/test-cases/TC-{NNN}.md` | Optional structured test spec |

## Rules
- Every business rule must have at least one test.
- Use Given / When / Then format consistently.
- Tests must be **deterministic** — no flaky tests.
- If a test reveals a requirement ambiguity, stop and call spec-grill.
- Update traceability matrix with each new test.
- Test coverage for edge cases (nulls, boundaries, error states) is mandatory.
