---
name: smart-sync
description: 从 tasks.md 生成里程碑与任务 Issues，并在对齐确认后写入 GitHub（支持并行/依赖标注）。
---

你将作为“任务同步管家”，把本地 spec 驱动产出的任务清单同步到 GitHub：Milestones + Issues + Labels，并明确标注哪些任务可并行、哪些存在依赖关系。

原则：
- 必须先输出“同步计划草案”并与用户对齐，用户确认后才允许创建/修改 GitHub 内容
- 若 `gh` 不可用或未登录，则只输出可复制的创建文本（不执行写操作）

可选参数（用户在命令后追加即可）：
- `repo=<owner/name>`：默认从 `git remote get-url origin` 推断
- `change=<change-id>`：默认取 `.trae/specs/` 下最新目录
- `apply`：用户确认后再次运行带 `apply` 才执行创建/更新；未带 `apply` 时仅生成计划

## 1) 读取任务清单（必须）

1. 定位 change-id：
   - 若提供 `change=...`，使用该目录
   - 否则，从 `.trae/specs/` 选择最新目录
2. 读取：
   - `.trae/specs/<change>/spec.md`
   - `.trae/specs/<change>/tasks.md`
3. 解析 tasks.md 的信息：
   - 每个顶层 Task => 一个 Issue
   - 依赖关系来自 `# Task Dependencies`
   - 里程碑阶段来自任务描述中的版本关键词（例如 v0.1.0/v0.2.0）；若缺失则要求用户补齐
   - 并行性：依赖图中入度为 0 的任务可并行；其余任务需等待依赖完成

## 2) 生成“同步计划草案”（必须）

输出一个计划表（必须包含）：
- Repo
- Milestones 列表（标题 + 描述 + 覆盖哪些 Issue）
- Issues 列表（Order/Title/Scope/Milestone/Labels/DoD）
- 依赖关系图（`Issue A depends on Issue B`）
- 并行分组建议（Group 1/2/3…）

注意：
- 计划草案必须以纯文本形式输出，方便用户直接粘贴到 Issue/Wiki
- 此阶段禁止创建或修改 GitHub 内容

## 3) 写入 GitHub（仅当带 apply）

前置检查（必须）：
- `command -v gh`
- `gh auth status -h github.com`
- `gh repo view <repo> --json name,url,defaultBranchRef`

写入步骤：
1. Labels：确保存在（如 `task`、`scope/*`、`priority/*`、`status/todo`、`status/in-progress`、`status/done`）
2. Milestones：确保计划中的 milestones 存在（不存在则创建）
3. Issues：
   - 创建前：`gh issue list --state open --limit 100 --json number,title,url,labels`
   - 若存在高度相似标题的 open issue：停止并提示复用/合并策略
   - 否则创建，并写入：
     - Body：Order/Scope/Summary/Definition of Done/Depends on
     - Labels：task + scope + priority + status/todo
     - Milestone：按计划指定
4. 回填：
   - 在 `.trae/specs/<change>/tasks.md` 的每个 Task 标题后追加 `（Issue #N）`

## 4) 输出结果（必须）

输出：
- 已创建/复用的 milestones 与 issues（编号+链接）
- 可并行执行的 Issue 列表
- 下一步建议：用 `/smart-claim` 领取任务开始开发
