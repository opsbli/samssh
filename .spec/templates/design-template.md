# 设计文档模板

> 本文档定义 `.spec/designs/` 的通用结构和生成规则。
> 每个业务模块使用三份标准格式文件，不依赖自创的 Markdown 包装格式。

---

## 目录结构

每个业务模块一个子目录：

```
.spec/designs/{module}/
├── db_schema.sql          # 全量 DDL — 标准 SQL
├── api_swagger.yaml       # 全量 API 规范 — OpenAPI 3.0 YAML
└── domain_model.puml      # 全量领域模型 — PlantUML 类图
```

### {module} 命名规则

| 来源 | 规则 | 示例 |
|------|------|------|
| 模块目录名 | 连字符小写 | `inventory-check` |
| Java package | 替换连字符为点 | `inventory-check` → `inventory.check` |
| URL 路径前缀 | 直接使用 | `/asset/inventory/...` |
| 数据库表前缀 | 连字符换下划线 | `inventory-check` → `asset_inventory_` |

---

## 文件一：db_schema.sql

### 角色
数据库设计的事实标准。AI 从该文件理解所有表结构、字段、索引、字典数据。

### 格式
标准 SQL DDL，多条 `CREATE TABLE` + `INSERT INTO` 语句。

### 生成规则（从代码反推 / as-is）
1. 从 Entity 类的 `@TableName` 和字段注解获取表名和列定义
2. 继承 `BaseEntity` / `TenantEntity` 的字段（`create_by`, `create_time`, `update_by`, `update_time`, `del_flag`）自动追加
3. 补充索引（`KEY` / `UNIQUE KEY`）：从 Mapper XML 或代码逻辑推断
4. 补充字典数据 `sys_dict_type` + `sys_dict_data`
5. 补充菜单权限 `sys_menu`
6. 遵循 `.spec/standards/database_design.md` 的完整规范

### 生成规则（从需求设计 / to-be）
1. 从 `Requirements` 中的实体定义推导表结构
2. 从业务规则推导校验约束和枚举值
3. 先写设计，审核通过后再编码
4. 遵循 `.spec/standards/database_design.md` 的完整规范

### AI 使用方式
- **读**：直接解析 SQL DDL 获取字段和约束
- **写**：先写完整 DDL，再对应的 Entity 代码
- **校验**：字段命名、类型、索引、逻辑删除字段 4 项必查

---

## 文件二：api_swagger.yaml

### 角色
API 设计的事实标准。AI 从该文件理解所有端点、参数、权限、响应格式。

### 格式
[OpenAPI 3.0](https://spec.openapis.org/oas/v3.0.3) YAML 格式。

### 生成规则（从代码反推 / as-is）
1. 从 Controller 类 `@RequestMapping` 获取基础路径
2. 从每个方法获取：HTTP 方法、路径、参数、`@SaCheckPermission`
3. 从 `@Validated` / `@NotBlank` / `@NotNull` 等获取校验规则
4. 从返回类型 `R<T>` / `TableDataInfo<T>` 构建响应 schema
5. 按端点分组，路径参数优先
6. 参考 `.spec/templates/api-swagger-template.yaml` 骨架

### 生成规则（从需求设计 / to-be）
1. 从 `Requirements` 中的 API 描述定义端点
2. 从 `Business Rules` 中的权限约束定义 `@SaCheckPermission`
3. 从 `Design` 中的数据结构定义请求/响应 schema
4. 先写设计，审核通过后再编码
5. 参考 `.spec/templates/api-swagger-template.yaml` 骨架

### AI 使用方式
- **读**：解析 YAML 字段，提取 `paths` 生成路由，提取 `schemas` 生成 VO/BO
- **写**：先写完整 OpenAPI spec，再生成对应的 Controller 代码
- **校验**：每个端点有 method + path + permission + request schema + response schema，5 项必查

### RuoYi 标准响应格式

```yaml
components:
  schemas:
    R:
      type: object
      properties:
        code:
          type: integer
          description: 状态码 (200=成功)
        msg:
          type: string
          description: 提示信息
        data:
          type: object
          description: 响应数据（泛型，实际类型在具体端点中定义）
    TableDataInfo:
      type: object
      properties:
        code:
          type: integer
          example: 200
        msg:
          type: string
          example: "查询成功"
        total:
          type: integer
          description: 总记录数
        rows:
          type: array
          items:
            $ref: '#/components/schemas/{具体VO}'
```

---

## 文件三：domain_model.puml

### 角色
领域模型的事实标准。AI 从该文件理解实体关系、聚合边界、继承层次。

### 格式
[PlantUML 类图](https://plantuml.com/class-diagram) 标准语法。

### 生成规则（从代码反推 / as-is）
1. 从 Entity 类提取：类名、字段、类型、`@TableName`
2. 从 `@TableLogic` / `@TableField` 等注解获取额外信息
3. 从数据库外键关系推断实体关联（一对一、一对多、多对多）
4. 从继承关系（`extends BaseEntity` / `extends TenantEntity`）标注父类
5. 只标注核心实体和值对象，不包含纯 Mapper/Service/Controller 类
6. 参考 `.spec/templates/domain-model-template.puml` 骨架

### 生成规则（从需求设计 / to-be）
1. 从 `Requirements` 中的实体关系定义领域模型
2. 从 `Business Rules` 中的约束标注关系多重性
3. 先写设计，审核通过后再编码
4. 参考 `.spec/templates/domain-model-template.puml` 骨架

### AI 使用方式
- **读**：解析 PUML 语法提取实体和关系
- **写**：先画领域模型，明确实体边界和聚合，再生成 Entity 代码
- **校验**：核心实体全覆盖、关系方向正确、多重性标注、继承关系明确，4 项必查

---

## 三文件关系

```
domain_model.puml        ← 实体和关系（高层抽象）
      ↓ 实例化
db_schema.sql              ← 表结构和字段（物理存储）
      ↓ 暴露
api_swagger.yaml          ← API 端点和数据交换（外部契约）
```

- `domain_model.puml` 中的每个实体 → `db_schema.sql` 中至少一张表
- `db_schema.sql` 中的表 → `api_swagger.yaml` 中至少一个响应 schema
- `api_swagger.yaml` 中的每个端点 → 追溯到一条 Requirement

---

## 完成检查清单

每个设计文件生成后，必须逐项检查：

- [ ] `{module}` 命名在所有三文件中一致
- [ ] 三文件均存在于 `.spec/designs/{module}/` 目录下
- [ ] 无缺失文件（不允许 2/3）
- [ ] 每个实体/端点/表都可追溯到一条 Requirement
- [ ] 遵循对应 standards 文件中的规范（database_design.md / api_design.md）
- [ ] 格式语法正确（SQL 可执行 / YAML 可解析 / PUML 可渲染）
