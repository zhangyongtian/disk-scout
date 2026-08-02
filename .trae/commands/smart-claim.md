---
name: smart-claim
description: 一键领取并完成一个 Issue（从 main 开分支→实现→提交→PR→合并/开启 auto-merge→同步 main），并按需循环直到无可领取任务。
---

你将作为“任务领取管家”，执行 `/smart-claim` 后进入端到端自动流程：领取一个可执行的任务 Issue → 标识进行中 → 始终从 main 创建带 issue 编号的分支 → 完成实现与验证 → 自动提交 → 自动创建 PR（包含 commits/files changed）→ 尝试合并或开启 auto-merge → 同步本地 main。若成功完成则继续领取下一个，直到没有可领取任务为止。

约束：
- 必须尊重依赖关系：只有依赖已完成的 Issue 才可领取
- 默认不抢占：若 Issue 已标记 `status/in-progress` 或已分配给其他人，则跳过
- 对 GitHub 的写操作（改 label/assign/PR）必须使用 `gh`；若不可用则退化输出可复制命令

可选参数：
- `repo=<owner/name>`：默认从 origin 推断
- `milestone=<title>`：只领取某个里程碑的任务
- `assignee=@me|<login>|none`：默认 `@me`
- `automerge`：提交 PR 后尝试开启 auto-merge（建议默认开启）
- `mode=auto|manual`：默认 `auto`；manual 只“领取+建分支”，后续由用户手动 `/smart-commit` `/smart-pr` `/smart-merge`
- `limit=<N>`：最多连续领取 N 个任务（默认 999）
- `watch=on|off`：当没有可领取任务时是否持续等待依赖完成并重试；默认 `on`
- `watch-interval=<seconds>`：watch 轮询间隔；默认 60
- `release-on-stop=on|off`：人为停止时是否释放当前已领取但未产出 PR 的任务；默认 `on`

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

若 `mode=manual`：
1. 仅完成领取并切分支，然后停止（后续由用户手动执行提交/PR/合并）：
   - 分支创建规则同下
2. 输出下一步建议：
   - `/smart-commit task=issue-<number>`
   - `/smart-pr issue=<number> order=<NNN> task=issue-<number> automerge(可选)`
   - `/smart-merge`

若 `mode=auto`（默认）：
1. 基于 issue 创建分支（必须始终从 main 开新分支，不依赖当前所在分支）：
   - `git fetch origin main --prune`
   - `git switch main`
   - `git pull --ff-only`
   - `git switch -c agent/issue-<number>-<yyyymmdd>`
   - 分支命名必须包含 Issue 编号，以便从分支名一眼看出“谁在做什么”
2. 实现与验证（必须）
   - 读取该 Issue 的 Summary/DoD，按 DoD 实现
   - 执行最小验证（项目约定的 print-based / 脚本验证）
3. 自动提交（必须）
   - 按 `.trae/rules/git-commit-message.md` 生成原子提交
   - commit message trailers 必须包含：
     - `Agent-Task: Issue #<number> - <title>`
     - `Agent-Decision: ...`
     - `Closes: #<number>`（仅当本提交确实完成该 issue）
4. 自动创建 PR（必须）
   - push：`git push -u origin HEAD`
   - PR body 必须包含：
     - Summary
     - Commits（`git log --oneline origin/main..HEAD` 实际输出）
     - Files Changed（`git diff --name-only origin/main..HEAD` 实际输出）
     - `Closes: #<number>`
5. 合并策略（必须）
   - 若指定 `automerge`：优先尝试开启 auto-merge（失败则输出阻塞原因并停止）
   - 否则若可直接合并：执行 merge 并删除分支
   - 合并成功后同步本地 main（fast-forward）

## 5) 完成后继续领取（必须）

当 PR 已合并（或 auto-merge 已开启）后：
- 若 issue 已 closed：继续领取下一个
- 若仍 open：
  - 保持 `status/in-progress`
  - 输出阻塞原因（例如等待 review/CI）
  - 不再领取新任务，避免并发过多导致混乱

## 6) 无可领取任务时的行为（必须）

当出现以下情况之一：
- 所有 open issues 都已 `status/in-progress`（有人在做）
- 只有存在依赖未完成的 issues（不可领取）
- 或者按 milestone 过滤后没有候选

若 `watch=off`：
- 输出“无可领取任务”的原因摘要并退出

若 `watch=on`（默认）：
- 输出“等待中”的原因摘要（例如依赖未完成的 issue 列表）
- 每 `watch-interval` 秒重新评估一次是否出现可领取任务
- 一旦出现可领取任务，立即继续领取并进入自动开发循环

## 7) 人为停止与任务释放（必须）

当用户人为停止（例如 Ctrl+C）时：
- 若 `release-on-stop=on`（默认），并且“当前已领取的 issue 尚未创建 PR”：
  - 将该 issue 的 `status/in-progress` 回滚为 `status/todo`
  - 若设置过 assignee，则取消 assignee（仅当 assignee 为 @me 且是本次领取设置的）
  - 留言（可选）：`Released by <user> on <date>`
- 若当前 issue 已创建 PR（已进入 review/merge 阶段）：
  - 不回滚状态，保持 `status/in-progress`，避免丢失上下文
- 输出停止时的最后状态摘要（当前 issue、是否释放、是否存在 PR）

## 8) 输出总结（必须）

输出：
- 本次领取的 issue/PR 列表（链接）
- 当前阻塞点（若有）
- 下一步建议（继续 / 等待合并 / 处理冲突）
