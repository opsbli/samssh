
# spec-bootstrap — 代码感知版

## Objective
Initialize or refresh project documentation from existing source code and project context. This version is **code-generation-aware**: it scans code generator templates to extract enforced patterns, then validates them against real code.

Use when entering an existing project with no (or incomplete) `.spec/` documentation.

---

## Flow
```
1. Scan project structure (src/, .spec/, configs/)
    ↓
2. Scan code generator templates (ruoyi-generator/src/main/resources/vm/)
   ├── Extract enforced coding patterns (entity/BO/VO/Mapper/Controller/Service)
   ├── Extract DDL patterns (table schema, audit fields)
   └── Extract permission patterns (menu SQL, permission prefix)
    ↓
3. Scan real code to verify patterns match
   ├── Controller layer (verify REST conventions)
   ├── Service layer (verify @Transactional, constructor injection)
   ├── Domain layer (verify entity inheritance, audit fields)
   └── DDL files (verify compliance with generator patterns)
    ↓
4. Generate standards documentation (.spec/standards/)
   ├── api_design.md       — from controller.vm + real controllers
   ├── architecture.md     — from module structure + pom.xml
   ├── coding_style.md     — from domain.vm + serviceImpl.vm + bo.vm
   ├── database_design.md  — from domain.vm + sql.vm + real DDL
   ├── logging.md          — from @Log annotation usage
   ├── security.md         — from @SaCheckPermission patterns
   ├── testing.md          — from project structure (or lack thereof)
   └── database-conventions.md — from DDL naming conventions
    ↓
### 5. Design Artifacts Handling

After generating standards, handle `.spec/designs/`:

| Artifact | Source | Bootstrap Action |
|----------|--------|-----------------|
| `db_schema.sql` | `script/sql/` (DDL files) | **Copy** existing DDL files directly |
| `api_swagger.yaml` | Controller annotations | **Skip** — requires code analysis tooling; flag as evidence gap |
| `domain_model.puml` | Entity classes | **Skip** — requires relationship extraction; flag as evidence gap |

```text
5. Scan script/sql/ for DDL files → copy to .spec/designs/
   ↓
6. Flag missing swagger/plantuml as evidence gap
   ↓
7. Run spec-discovery to surface evidence gaps
```
    ↓
6. For each evidence gap → run spec-grill
    ↓
7. For each decision → run spec-confirm → spec-requirement
    ↓
8. Generate initial traceability matrix
```

---

## Inputs
| Source | Path | Purpose |
|--------|------|---------|
| Code generator templates | `{module}/ruoyi-generator/src/main/resources/vm/` | Extract enforced patterns |
| Source code | `src/main/java/` (controllers, services, entities) | Verify patterns in practice |
| Database schema | `script/sql/` or `.spec/designs/db_schema.sql` | Extract DDL conventions |
| Build config | `pom.xml` | Extract tech stack, dependencies |
| Existing docs | `.spec/` | Any prior documentation |

---

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Code pattern report | `.spec/evidence/E{NNN}.md` | Structured pattern extraction |
| Standards documentation | `.spec/standards/` | 8+ standard files derived from code |
| Evidence records | `.spec/evidence/E{NNN}.md` | What exists vs what is missing |
| Initial requirements | `.spec/requirements/R{NNN}.md` | Extracted or created |
| Traceability matrix | `.spec/traceability/requirements_matrix.md` | Initial linkage state |

---

## Code Generator Scanning Rules

### 1. Controller Pattern Extraction
Scan `vm/java/controller.java.vm` for:
- Base class: `extends BaseController`
- Injection: `@RequiredArgsConstructor` (constructor injection)
- Auth: `@SaCheckPermission("${permissionPrefix}:{action}")`
- Endpoint naming: `GET /list`, `GET /{id}`, `POST /`, `PUT /`, `DELETE /{ids}`, `POST /export`
- Response types: `TableDataInfo<Vo>` (list), `R<Vo>` (detail), `R` (write)
- Validation: `@Validated`, `AddGroup.class`, `EditGroup.class`
- Logging: `@Log(title = "...", businessType = BusinessType.{ACTION})`
- Idempotent: `@RepeatSubmit`

### 2. Domain Pattern Extraction
Scan `vm/java/domain.java.vm` for:
- Entity inheritance: `extends TenantEntity` (if has tenantId) or `extends BaseEntity`
- Annotations: `@Data`, `@EqualsAndHashCode(callSuper = true)`, `@TableName`
- Primary key: `@TableId(value = "id")`
- Logic delete: `@TableLogic` on `delFlag`
- Optimistic lock: `@Version` on `version`
- Serializable: `@Serial` + `serialVersionUID`

### 3. Service Pattern Extraction
Scan `vm/java/serviceImpl.java.vm` for:
- Annotations: `@Slf4j`, `@RequiredArgsConstructor`, `@Service`
- Mapper injection: `private final XxxMapper baseMapper`
- Standard methods: `queryById`, `queryPageList`, `queryList`, `insertByBo`, `updateByBo`, `deleteWithValidByIds`
- Query builder: `LambdaQueryWrapper` + `buildQueryWrapper(bo)`
- BO→Entity conversion: `MapstructUtils.convert(bo, Entity.class)`

### 4. DDL Pattern Extraction
Scan `vm/sql/sql.vm` and real DDL for:
- Menu SQL: `insert into sys_menu (menu_id, menu_name, parent_id, order_num, path, component, ...)`
- Permission naming: `{prefix}:list`, `{prefix}:query`, `{prefix}:add`, `{prefix}:edit`, `{prefix}:remove`, `{prefix}:export`

### 5. Real Code Validation
After extracting patterns from templates, scan real code to:
- Verify real controllers follow the template patterns
- Flag deviations as evidence gaps (e.g., missing `@Log`, missing `@SaCheckPermission`)
- Note any custom patterns not covered by templates (e.g., `/generate/{planId}` actions)

---

## Standard Generation Rules

After scanning, generate each standards file by:
1. Reading the relevant template (vm file)
2. Reading 2-3 real code examples
3. Extracting the consensus pattern
4. Writing to `.spec/standards/{name}.md`

### File-to-Source Mapping
| Standards File | Primary Source | Verification Source |
|---------------|---------------|-------------------|
| `api_design.md` | controller.java.vm | Real controllers |
| `architecture.md` | pom.xml + directory structure | Module structure |
| `coding_style.md` | domain.vm + bo.vm + serviceImpl.vm | Real entities + services |
| `database_design.md` | domain.vm + sql.vm + DDL | Real DDL files |
| `logging.md` | controller.vm (@Log) | Real controllers |
| `security.md` | controller.vm (@SaCheckPermission) | Real controllers |
| `testing.md` | Project structure (test/ existence) | Default TDD patterns |
| `database-conventions.md` | DDL conventions across all SQL files | Real DDL files |

---

## Rules
- **Code generator templates take precedence** over individual code examples (they define the intended pattern).
- **Real code deviations** from templates must be flagged as evidence gaps, not silently accepted.
- Do not create requirements from code alone — always run spec-grill for confirmation.
- Bootstrap is a **one-time** setup; subsequent work uses the normal V3 lifecycle.
- When in doubt about any fact extracted from code, flag it as evidence (gap type) and let spec-grill resolve.
