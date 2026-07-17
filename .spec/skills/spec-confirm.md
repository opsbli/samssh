
# spec-confirm

## Objective
Persist user answers as permanent Decision records, update requirements / business rules / use cases, and close the traceability loop. This runs after spec-grill receives a user answer.

---

## Flow
```
Receive: User Answer to Q{NNN}
    ↓
Load: Related Evidence E{NNN}
    ↓
Create Decision Record D{NNN}.md
    ↓
Update Requirements (if answer affects business rules)
    ↓
Update Business Rules (business-rules file)
    ↓
Update Use Cases (if answer affects workflow)
    ↓
Update Traceability (link E→Q→D→R)
    ↓
Report: Decision persisted and artifacts updated
```

---

## Inputs
| Source | Path | Purpose |
|--------|------|---------|
| Question | (from spec-grill) | Q{NNN} — what was asked |
| Evidence | `.spec/evidence/E{NNN}.md` | What triggered the question |
| User answer | (from spec-grill) | The raw answer text |
| Existing requirements | `.spec/requirements/` | To be updated if needed |
| Existing business rules | `.spec/business-rules/` | To be updated if needed |
| Use cases | `.spec/use-cases/` | To be updated if needed |
| Traceability | `.spec/traceability/requirements_matrix.md` | To be updated |

---

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Decision record | `.spec/decisions/D{NNN}.md` | Full decision record |
| Updated requirements | `.spec/requirements/R{NNN}.md` | Updated with decision context |
| Updated business rules | `.spec/business-rules/BR{NNN}.md` | New/changed rules |
| Updated use cases | `.spec/use-cases/UC{NNN}.md` | Updated flows |
| Updated traceability | `.spec/traceability/requirements_matrix.md` | E→Q→D→R links |

---

## Rules

### Decision record rules
1. Every user answer that closes a question produces exactly one Decision record.
2. Decision records are **never deleted** — only superseded by a later Decision.
3. When a later Decision supersedes an earlier one, the earlier record gets a `superseded_by: D{NNN}` marker.
4. Before writing, check if a similar Decision already exists — if so, either skip or supersede.

### Update rules
1. Only update artifacts that are directly impacted by the answer.
2. Every update must cite the Decision ID that drove the change.
3. Do not make changes beyond the scope of the answer.

### Traceability rules
1. Every new link (E→Q→D→R) must be added to the traceability matrix.
2. If a Requirement was created or modified, add a backward link from Requirement → Decision as well.

---

## Decision Record Template

```markdown
# D{NNN} — {Short Title}

**Date**: {YYYY-MM-DD}

**Context**:
{What was the situation — what feature, what ambiguity}

**Evidence**: E{NNN}
{What triggered the question}

**Question**: Q{NNN}
{The exact question asked}

**Answer**:
{The user's exact answer or choice}

**Impact**:
- Requirements affected: {R{NNN}, R{NNN}...}
- Business rules affected: {BR{NNN}...}
- Designs affected: {if any}
- Code affected: {if any}

**Supersedes**: {D{NNN} if this decision replaces an earlier one, else N/A}
**Superseded by**: {D{NNN} if this decision was later replaced, else N/A}
```

---

## Exit Criteria
- Decision record D{NNN}.md exists in `.spec/decisions/`.
- All impacted requirements, business rules, use cases are updated.
- Traceability matrix updated with E→Q→D→R links.
- Summary report delivered.
