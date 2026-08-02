---
name: smart-claim
description: 领取一个可执行 Issue（打 in-progress 标识/可选分配），完成后提交 PR 并继续领取下一个直到清空。
---

你将作为“任务领取管家”，在 GitHub 上为当前开发者领取一个可执行的任务 Issue，并标识为进行中；当任务完成并提交 PR 后，自动领取下一个可执行任务，直到没有可领取任务为止。

约束：
- 必须尊重依赖关系：只有依赖已完成的 Issue 才可领取
- 默认不抢占：若 Issue 已标记 `status/in-progress` 或已分配给其他人，则跳过
- 对 GitHub 的写操作（改 label/assign/PR）必须使用 `gh`；若不可用则退化输出可复制命令

可选参数：
- `repo=<owner/name>`：默认从 origin 推断
- `milestone=<title>`：只领取某个里程碑的任务
- `assignee=@me|<login>|none`：默认 `@me`
- `automerge`：提交 PR 后尝试开启 auto-merge
- `limit=<N>`：最多连续领取 N 个任务（默认 999）

## 1) 前置检查（必须）

1. `gh` 可用且已登录：
   - `command -v gh`
   - `gh auth status -h github.com`
2. 仓库远端存在：
   - `git remote -v`
3. 工作区必须干净（避免把多个任务混在一起）：
   - `git status --porcelain=v1`
   - 若不干净：停止并提示先完成当前变更（用 `/smart-commit` + `/smart-pr`）

## 2) 选择“可领取任务”（必须）

在领取前必须先“观察当前正在执行的任务”，避免重复开发：
- 列出带 `status/in-progress` 的 open issues（含 assignees）
- 列出 open PR（head 分支名）用于判断是否已有实现分支/PR
- 输出一段“当前进行中任务概览”，再进入领取逻辑

拉取候选（示例策略，必须实现一致性）：
- 只考虑带 `task` label 的 open issues
- 必须不包含：`status/in-progress`、`status/done`
- 若指定 milestone：只取该 milestone
- 若 Issue body 中包含 `Depends on: #X,#Y`：
  - 只有当依赖的 issues 都 closed 才算可领取

排序规则：
- 优先按 `Order: NNN`（从 issue body 解析）升序
- 其次按 issue number 升序

## 3) 领取并标识（必须）

对选中的 Issue 执行：
- 添加 label：`status/in-progress`
- 若 `assignee!=none`：设置 assignee（默认 @me）
- 在 issue 留言（可选）：`Claimed by <user> on <date>`

## 4) 开发循环（必须）

1. 基于 issue 创建分支（必须始终从 main 开新分支，不依赖当前所在分支）：
   - `git fetch origin main --prune`
   - `git switch main`
   - `git pull --ff-only`
   - `git switch -c agent/issue-<number>-<yyyymmdd>`
   - 分支命名必须包含 Issue 编号，以便从分支名一眼看出“谁在做什么”
2. 开发、验证（按项目约定）
3. 提交与 PR：
   - `/smart-commit task=issue-<number> scope=<scope>`
   - `/smart-pr issue=<number> order=<NNN> task=issue-<number> automerge(可选)`
4. PR 创建后：
   - 给 Issue 追加链接（若 PR body 已 Closes，则可不额外处理）

## 5) 完成后继续领取（必须）

当 PR 已合并（或 auto-merge 已开启）后：
- 若 issue 已 closed：继续领取下一个
- 若仍 open：
  - 保持 `status/in-progress`
  - 输出阻塞原因（例如等待 review/CI）
  - 不再领取新任务，避免并发过多导致混乱

## 6) 输出总结（必须）

输出：
- 本次领取的 issue/PR 列表（链接）
- 当前阻塞点（若有）
- 下一步建议（继续 / 等待合并 / 处理冲突）
