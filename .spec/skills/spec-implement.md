
# spec-implement

## Objective
Implement code according to approved designs, requirements, and decisions. Follow the architecture and coding standards defined in AGENTS.md.

---

## Flow
```
Load Requirement R{NNN}
    ↓
Load related Designs (db, api, domain)
    ↓
Load related Decisions D{NNN}
    ↓
Implement code following the design
    ↓
Update traceability (Design → Code links)
```

---

## Inputs
| Source | Path | Purpose |
|--------|------|---------|
| Requirements | `.spec/requirements/` | Business rules to implement |
| Designs | `.spec/designs/db_schema.sql`, `.spec/designs/api_swagger.yaml`, `.spec/designs/domain_model.puml` | Implementation spec |
| Decisions | `.spec/decisions/` | Technical constraints |
| Coding standards | `.spec/standards/` | Style and architecture guides |

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Source code | `src/main/java/` | Per project structure |
| Mapper XML | `src/main/resources/mapper/` | MyBatis XML (if used) |
| Traceability | `.spec/traceability/requirements_matrix.md` | Design→Code links |

## Rules
- Follow designs exactly — if implementation reveals a design gap, stop and call spec-grill.
- Constructor injection, `@Transactional` on service boundary, no business logic in controller.
- Every implemented method should be traceable to a design element.
- Do not implement requirements that lack an approved design.
- Do not add features not in the requirements.
