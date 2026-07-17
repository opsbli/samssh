# Lifecycle 门禁检查清单

> 每次阶段转换前运行此检查清单。所有检查项必须通过才能进入下一阶段。
> 由 spec-trace 和 spec-review 技能自动验证。

## Phase 1→2: 发现 → 盘问

### 门禁：至少有一个 Evidence 记录
- [ ] `.spec/evidence/` 目录存在
- [ ] 至少一个 E{NNN}.md 文件存在
- [ ] 每个 E 记录含有 Type（known_fact / gap / contradiction / ambiguity）
- [ ] 所有 gap-type E 记录有对应的 Q 记录

## Phase 2→3: 盘问 → 确认

### 门禁：所有 Question 有 Answer
- [ ] 每个 gap-type E 有 Q 记录
- [ ] 每个 Q 标注了 status（open / closed）
- [ ] 每个 closed Q 有对应的 D 记录
- [ ] 没有 open 状态的 Q 记录

## Phase 3→4: 确认 → 需求

### 门禁：所有 Decision 有对应 Requirement；关键规则有 Business Rule 记录
- [ ] `.spec/decisions/` 目录存在
- [ ] `.spec/requirements/` 目录存在
- [ ] `.spec/business-rules/` 目录存在
- [ ] 每个 D 记录可追溯到至少一个 R 记录
- [ ] 每个 R 记录包含 Background, Goals, Business Rules, Acceptance Criteria
- [ ] 每个 R 中引用的 BR{NNN} 有对应的文件在 `.spec/business-rules/`

## Phase 4→5: 需求 → 设计

### 门禁：设计文件与 Requirement 可追溯
- [ ] `.spec/designs/` 存在且非空
- [ ] 每个 R 至少链接到一个 Design artifact
- [ ] `.spec/traceability/design-to-requirement.md` 存在
- [ ] 核心设计三件套检查：
  - [ ] `architecture.puml` 存在（架构组件图，至少覆盖目标功能域）
  - [ ] `app_state.puml` 存在或标记为 gap 有计划补齐（gpui Entity<T> 状态树）
  - [ ] `module_deps.md` 存在或标记为 gap 有计划补齐（模块依赖关系）

## Phase 5→6: 设计 → 计划

### 门禁：Plan 已审批
- [ ] Plan 已通过用户审批（审批时使用 todo_write 跟踪任务列表）
- [ ] 每个 Task 可追溯到 R{NNN}

## Phase 6→7: 计划 → 编码

### 门禁：代码与设计一致
- [ ] 编码使用 spec-implement 流程
- [ ] 代码跟随 designs（architecture.puml, app_state.puml, module_deps.md）
- [ ] 遵循 AGENTS.md 编码规则：
  - [ ] `Entity<T>` / `impl Render` gpui 组件模式
  - [ ] SSH/SFTP 网络 I/O 通过后台异步任务 + 消息队列通信
  - [ ] 凭据使用 Windows DPAPI 加密存储

## Phase 7→8: 编码 → 测试

### 门禁：测试通过
- [ ] `src/test/` 目录存在
- [ ] 所有代码变更覆盖模块有测试文件
- [ ] 测试用 Given / When / Then 格式
- [ ] 测试全部通过（运行 `cargo test` 验证）

## Phase 8→9: 测试 → 追溯

### 门禁：全链路无缺失链接
- [ ] `.spec/traceability/requirements_matrix.md` 存在
- [ ] 对每个 R：
  - [ ] E→Q 链路完整
  - [ ] Q→D 链路完整
  - [ ] D→BR 链路完整（如有 BR）
  - [ ] BR→R 链路完整（如有 BR）
  - [ ] R→Design 链路完整
  - [ ] Design→Code 链路完整
  - [ ] Code→Test 链路完整
  - [ ] Test→R 反向链路完整

## Phase 9→10: 追溯 → 审查

### 门禁：无 blocker 问题
- [ ] spec-review 运行且无 blocker
- [ ] 所有缺失链接被记录为 issue
- [ ] 无 open Questions

## Phase 10→11: 审查 → 发布

### 门禁：发布就绪
- [ ] spec-review pass
- [ ] 所有测试通过
- [ ] Traceability 已更新
- [ ] 无 open Questions
- [ ] 所有 Decisions 已持久化
- [ ] Release 报告已生成

---

## 自动化脚本参考

```bash
# 检查 gateway 状态
# Phase 1→2
ls .spec/evidence/ 2>/dev/null || echo "FAIL: No evidence"

# Phase 2→3
echo "SKIP: Q records are embedded in E files, no separate questions directory"

# Phase 3→4
for d in .spec/decisions/*.md; do
  grep -q "R\d{3}" "$d" || echo "WARN: No R link in $d"
done

# Phase 4→5
ls .spec/designs/ 2>/dev/null || echo "FAIL: No designs"
ls .spec/traceability/design-to-requirement.md 2>/dev/null || echo "FAIL: No D→R trace"

# Phase 5→6
echo "SKIP: Plans are ephemeral (todo_write), not persisted to disk"

# Phase 8→9 (full trace check)
ls .spec/traceability/requirements_matrix.md 2>/dev/null || echo "FAIL: No matrix"
```
