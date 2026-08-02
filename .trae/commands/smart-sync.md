---
name: smart-sync
description: 对齐 specs 的任务与验收清单到 GitHub（plan/apply；支持幂等写入与进度回填）。
---

你将作为“任务同步管家”，把本地 `.trae/specs/<change-id>/tasks.md` 与 GitHub 的 Milestones/Issues/Labels 做双向对齐，形成一套可反复执行、不会重复创建的同步机制：
- 新规划：从 tasks.md 生成 milestones + issues，并回填本地 `（Issue #N）`
- 持续对齐：把远端 issue 状态（open/closed、labels、milestone、assignee、PR）对齐回本地 tasks.md 的勾选与信息；并补齐远端缺失的管理标签

同时，本命令会维护 `.trae/specs/<change-id>/checklist.md` 的“待验收”状态：
- checklist 每条可通过 `@issues:#N,#M` 声明关联的 GitHub Issue
- `apply` 时若关联 issues 全部 closed，则把 checklist 从 `[ ]` 标记为 `[-]`（待验收）
- 本命令不会把 checklist 自动改为 `[x]`

原则：
- 必须先输出“同步计划草案”并与用户对齐，用户确认后才允许创建/修改 GitHub 内容
- 若 `gh` 不可用或未登录，则只输出可复制的创建文本（不执行写操作）
- `apply` 模式必须幂等：重复执行不会重复创建 issues；优先复用与更新

可选参数（用户在命令后追加即可）：
- `repo=<owner/name>`：默认从 `git remote get-url origin` 推断
- `change=<change-id>`：默认取 `.trae/specs/` 下最新目录
- `apply`：用户确认后再次运行带 `apply` 才执行创建/更新；未带 `apply` 时仅生成计划（plan）
- `commit=on|off`：apply 后是否将本地回填的变更提交到 main；默认 `on`
- `base=<branch>`：用于同步 main 的基线分支；默认 `main`
- `verbose=on|off`：是否展开打印“已完成/无变化”的条目；默认 `off`（仅输出增量与摘要）

## 1) 读取任务清单（必须）

1. 定位 change-id：
   - 若提供 `change=...`，使用该目录
   - 否则，从 `.trae/specs/` 选择最新目录
2. 读取：
   - `.trae/specs/<change>/spec.md`
   - `.trae/specs/<change>/tasks.md`
   - `.trae/specs/<change>/checklist.md`（若存在）
3. 解析 tasks.md 的信息：
   - 每个顶层 Task => 一个 Issue
   - Issue 主键匹配优先级（必须按此顺序）：
     - 任务标题中已有 `（Issue #N）` => 直接匹配 #N
     - 否则匹配 `Order`（从任务标题的 `[NNN]` 提取；没有则要求用户补齐）
     - 最后才允许用标题相似度兜底（必须输出风险提示）
   - 依赖关系来自 `# Task Dependencies`，并在 issue body 中写为 `Depends on: #X,#Y`（或无）
   - 里程碑：从任务标题/正文中的 `v0.1.0/v0.2.0/...` 识别；若无法识别必须阻断并提示用户补齐
   - 并行性：按依赖图拓扑分层生成 Group 1/2/3…（入度为 0 为 Group 1）

## 2) 生成“同步计划草案”（必须）

在 plan 阶段必须同时生成“本地计划”和“远端现状”，并输出“对齐差异”：

1) 本地计划（来自 tasks.md）
- Repo
- Change-id
- Milestones 列表（标题 + 描述 + 覆盖的 Order/Task）
- Issues 列表（Order/Title/Scope/Milestone/Priority/Depends on/DoD）
- 并行分组建议（Group 1/2/3…）

2) 远端现状（来自 GitHub）
- Open issues（带 task label）
- Closed issues（最近 N 条，或按 Order 匹配）
- Labels/milestones 是否齐全

3) checklist 待验收预测（若存在 checklist.md 且包含 `@issues:`）
- 哪些条目因关联 issues 全部 closed 将进入 `[-]`

注意：
- 计划草案必须以纯文本形式输出，方便用户直接粘贴到 Issue/Wiki
- 此阶段禁止创建或修改 GitHub 内容
- 若发现以下任一问题，必须阻断并提示修正后再 plan：
  - 某个顶层 Task 缺少 Order（无法生成 `[NNN]`）
  - 无法识别 milestone
  - 依赖关系引用了不存在的任务

输出规则（默认增量，避免任务多时刷屏）：
- 默认（`verbose=off`）：
  - 仅输出：open issues、存在差异的条目、本次会发生的写入动作摘要、以及 checklist 将进入 `[-]` 的条目
  - 已 closed 且已对齐（labels/milestone 完整）的 issues 仅输出计数，不逐条打印
- `verbose=on`：
  - 允许展开输出所有匹配到的 closed issues 与所有 checklist 条目状态

## 3) 写入 GitHub（仅当带 apply）

前置检查（必须）：
- `command -v gh`
- `gh auth status -h github.com`
- `gh repo view <repo> --json name,url,defaultBranchRef`

apply 步骤（必须幂等）：

### 3.1 同步 main（必须）

对齐与回填必须基于最新 main，确保后续 `/smart-claim` 从 main 切分支拿到最新清单：
- 若当前不在 `<base>`：
  - 若工作区不干净：先停止并提示用户处理（避免把别的任务混进回填提交）
  - `git fetch origin <base> --prune`
  - `git switch <base>`
  - `git pull --ff-only`

### 3.2 Labels（必须）

确保存在以下 labels（不存在则创建，存在则不改动）：
- `task`
- `status/todo`、`status/in-progress`、`status/done`
- `scope/*`（从 tasks 推导最小集合，例如 cli/scan/report/filter/ci/docs）
- `priority/high`、`priority/medium`、`priority/low`

### 3.3 Milestones（必须）

确保 plan 中的 milestones 存在（不存在则创建）。

### 3.4 Issues（创建/更新/补齐）

1) 拉取远端 issues（open + closed）用于匹配：
- open：`gh issue list --state open --limit 200 --json number,title,url,labels,milestone,assignees`
- closed：`gh issue list --state closed --limit 200 --json number,title,url,labels,milestone,assignees`

2) 对每个本地 Task 执行：
- 若 tasks.md 已有 `（Issue #N）`：复用 #N，并补齐：
  - labels：task + scope + priority + status/todo（若未 in-progress/done）
  - milestone：按 plan 修正
  - body：补齐 `Order:`、`Scope:`、`Summary:`、`Definition of Done:`、`Depends on:`（仅追加缺失字段，避免覆盖用户手写内容）
- 否则按 Order 匹配已有 issue：
  - 优先匹配标题前缀 `[NNN]`
  - 命中则复用并回填 `（Issue #N）`
  - 否则创建新 issue 并回填

3) 状态对齐（apply 内必须执行）
- issue 已 closed：确保带 `status/done`
- issue open 且无 `status/in-progress`：确保带 `status/todo`

### 3.5 回填到本地 tasks.md（必须）

对每个 Task 回填：
- `（Issue #N）`
- 若 issue 已 closed：将该 Task 勾为 `[x]`（仅对顶层 Task；子任务保持原样）

### 3.6 回填到本地 checklist.md（若存在则必须）

若 `.trae/specs/<change>/checklist.md` 存在且包含 `@issues:`：
- 解析每条 checklist 的 `@issues:#N,#M`
- 若关联 issues 全部 closed：
  - 当该条为 `[ ]` 时，将其改为 `[-]`
  - 当该条为 `[-]` 或 `[x]` 时保持不变
- 若存在任意未 closed：
  - 不降级（保持现状），仅在输出中提示“仍被阻塞的 issues”

### 3.7 将回填结果提交到 main（默认开启）

若 `commit=on`：
- 确认仅包含 `.trae/specs/<change>/tasks.md` 与 `.trae/specs/<change>/checklist.md` 的变更
- 在 `<base>`（默认 main）上提交：
  - `chore(sync): 对齐 specs 与 GitHub 任务状态`
  - body 必须包含：change-id、同步到的 milestones/issues 数量、是否有新建 issue
- 不允许 push（push/PR 仍然走 `/smart-pr`）

## 4) 输出结果（必须）

输出：
- 已创建/复用的 milestones 与 issues（编号+链接）
- 并行分组建议（Group 1/2/3…）
- checklist 待验收清单（`[-]` 条目列表，若存在）
- 本地 tasks/checklist 是否已回填并提交到 main（commit=on/off）
- 下一步建议：
  - `git status` 应为干净
  - 用 `/smart-claim` 从最新 main 开始领取任务

输出规则（默认增量，避免任务多时刷屏）：
- 默认（`verbose=off`）：
  - 仅逐条打印：open issues 的变更、以及本次创建/更新/补齐的 issues
  - closed issues 若无变化仅输出计数（例如：`closed issues aligned: 12 (hidden)`）
  - checklist 仅输出“本次从 `[ ] -> [-]` 的新增条目”与“仍被阻塞的条目摘要”
- `verbose=on`：
  - 允许展开输出所有对齐对象（open + closed）与所有 checklist 条目状态
