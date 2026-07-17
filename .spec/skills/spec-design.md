
# spec-design

## Objective
Update design artifacts (database schema, API spec, domain model) to reflect approved Requirements and Decisions. Design must be complete BEFORE any coding begins.

---

## Flow
```
Load Requirement R{NNN}
    ↓
Load related Decision D{NNN}
    ↓
Load templates from .spec/templates/
    ↓
For each design artifact:
  → Read existing file (if as-is update)
  → Or create from template (if new module)
  → Apply design changes
  → Validate format
    ↓
Update traceability (Requirement → Design links)
```

---

## Directory Structure

每个业务模块一个子目录，三份标准格式文件：

```
.spec/designs/{module}/
├── db_schema.sql          # 全量 DDL — 标准 SQL
├── api_swagger.yaml       # 全量 API 规范 — OpenAPI 3.0 YAML
└── domain_model.puml      # 全量领域模型 — PlantUML 类图
```

{module} 命名规则见 `.spec/templates/design-template.md`。

---

## Templates
| Template | Path | Format | Purpose |
|----------|------|--------|---------|
| 设计文档主模板 | `.spec/templates/design-template.md` | Markdown | 三文件关系、生成规则、AI使用方式 |
| API 骨架 | `.spec/templates/api-swagger-template.yaml` | OpenAPI 3.0 YAML | 端点/schema 模板，含 RuoYi 标准响应 |
| 领域模型骨架 | `.spec/templates/domain-model-template.puml` | PlantUML | 类图骨架，含基础实体和关系符号 |

## Inputs
| Source | Path | Purpose |
|--------|------|---------|
| Requirements | `.spec/requirements/` | What to design for |
| Decisions | `.spec/decisions/` | Design decisions and constraints |
| Existing designs | `.spec/designs/` | Current state to modify |
| Templates | `.spec/templates/` | Standard format skeletons |

## Outputs
| Artifact | Location | Format |
|----------|----------|--------|
| Database schema | `.spec/designs/{module}/db_schema.sql` | Standard SQL DDL |
| API spec | `.spec/designs/{module}/api_swagger.yaml` | OpenAPI 3.0 YAML |
| Domain model | `.spec/designs/{module}/domain_model.puml` | PlantUML class diagram |
| Traceability | `.spec/traceability/requirements_matrix.md` | R→Design links |

## Generation Rules

### db_schema.sql（从代码反推 / as-is）
1. 扫描 `script/sql/` 目录找到对应模块的 DDL 文件
2. 复制到 `.spec/designs/{module}/db_schema.sql`
3. 清理无关的跨模块引用（只保留本模块表）
4. 遵循 `.spec/standards/database_design.md` 完整规范

### db_schema.sql（从需求设计 / to-be）
1. 从 Requirements 提取实体定义 → 每实体一张表
2. 自动追加 BaseEntity/TenantEntity 审计字段
3. 生成索引、字典数据、菜单权限 DDL
4. 遵循 `.spec/standards/database_design.md` 完整规范

### api_swagger.yaml（从代码反推 / as-is）
1. 从 Controller 注解提取所有端点（`@RequestMapping`, HTTP 方法注解）
2. 从方法参数提取请求/响应 schema
3. 从 `@SaCheckPermission` 提取权限标识
4. 填充到 `api-swagger-template.yaml` 骨架中
5. 验证 YAML 语法正确

### api_swagger.yaml（从需求设计 / to-be）
1. 从 Requirements 提取 API 描述 → 定义端点路径和方法
2. 从 Business Rules 提取权限约束 → 定义 security
3. 从字段定义提取 VO/BO → 定义 schemas
4. 填充到 `api-swagger-template.yaml` 骨架中
5. 验证 YAML 语法正确

### domain_model.puml（从代码反推 / as-is）
1. 从 Entity 类提取实体名和字段
2. 从继承关系推导父类（BaseEntity/TenantEntity）
3. 从数据库外键关系推导实体关联
4. 填充到 `domain-model-template.puml` 骨架中

### domain_model.puml（从需求设计 / to-be）
1. 从 Requirements 提取实体和关系定义
2. 标注聚合边界和关系多重性
3. 填充到 `domain-model-template.puml` 骨架中

## Rules
- **No coding before design update** — this is a hard gate.
- Every design change must cite the Requirement that drives it.
- Database changes must follow the Database Rules in AGENTS.md.
- API changes must follow the API Rules in AGENTS.md.
- **Use the templates.** Always reference `.spec/templates/` for the correct skeleton.
- **Module name must be consistent** across all three files and file paths.
- **All three files must exist** before a module is considered "designed" (no 2/3).
- **YAML must be syntactically valid** — verify after every change.
- If a design change introduces ambiguity, call spec-grill before proceeding.

## Completion Checklist
Each design session must verify:
- [ ] All three files exist at `.spec/designs/{module}/`
- [ ] {module} naming consistent across files, paths, and references
- [ ] Every endpoint/table/entity traces to a Requirement
- [ ] Format syntax: SQL parseable / YAML valid / PUML renderable
- [ ] Standards compliance checked (database_design.md / api_design.md)
- [ ] Traceability matrix updated (R→Design links)
