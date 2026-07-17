
# spec-discovery

## Objective
Systematically scan all available sources to surface Evidence before any design or implementation work begins. This is the **first step** in the V3 lifecycle — run before any other skill.

---

## Flow
```
Scan sources (requirements, designs, code, tests, existing decisions)
    ↓
Extract facts (what is known, documented, committed)
    ↓
Identify gaps (what is missing, ambiguous, contradictory)
    ↓
For each gap → Generate Evidence record E{NNN}.md
    ↓
Report: Known facts + Evidence gap list
```

---

## Inputs
| Source | Path | What to extract |
|--------|------|-----------------|
| Business Rules | `.spec/business-rules/` | Existing BR{NNN} records, status, test coverage |
| Requirements | `.spec/requirements/` | Existing business rules, acceptance criteria |
| Designs | `.spec/designs/db_schema.sql` | Tables, columns, relationships |
| Designs | `.spec/designs/api_swagger.yaml` | Endpoints, request/response schemas |
| Designs | `.spec/designs/domain_model.puml` | Domain entities, relationships |
| Code | `src/main/java/` | Existing implementations, service interfaces — **use `codegraph_explore` MCP tool as primary scanner** |
| Code | `src/main/resources/` | Mapper XML, configuration |
| Tests | `src/test/java/` | Existing test cases, coverage gaps |
| Decisions | `.spec/decisions/` | Previously closed decisions |
| Traceability | `.spec/traceability/requirements_matrix.md` | Current linkage state |

---

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Evidence records | `.spec/evidence/E{NNN}.md` | Per the evidence template |
| Evidence gap report | (reported to user) | Summary of gaps found |

---

## Rules

### Discovery rules
1. Scan **all** input sources before making any judgment.
2. **Code scanning**: Use `codegraph_explore` MCP tool as the primary scanner for code sources — one call returns symbols, call paths, and source grouped by file. Fall back to grep/glob/Read only when codegraph has no index or the query is unsupported.
3. Separate **known facts** from **gaps** — never conflate them.
4. A gap is: missing documentation, ambiguous wording, contradictory statements across sources, or absence of a required artifact.
5. Each gap gets its own Evidence record (E{NNN}.md).
6. Do not try to resolve gaps in discovery — that is spec-grill's job.
7. Code is the **lowest priority** source — it may be stale.

### Evidence record structure
Each E{NNN}.md must contain:
- **Source**: which file(s) / conversation(s) the evidence was drawn from
- **Type**: `known_fact` | `gap` | `contradiction` | `ambiguity`
- **Description**: what was found
- **Location**: exact file path + line or section

### Exit criteria
- All inputs scanned.
- At least one Evidence record exists, or a clear statement that no gaps exist.
- Report delivered to user.

---

## Evidence record template

```markdown
# E{NNN} — {Short Title}

**Source**: {file path(s)}
**Type**: {known_fact | gap | contradiction | ambiguity}
**Location**: {file:line or section}

**Description**:
{What was found — be precise and cite the source}

**Relevance**:
{Why this matters — what decision or requirement depends on it}
```
