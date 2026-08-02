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
- `watch=on|off`：当没有可领取任务时是否持续等待并重试；默认 `off`（推荐关闭，避免空转）
- `watch-interval=<seconds>`：watch 轮询间隔；默认 60（仅当 watch=on 时生效）
- `release-on-stop=on|off`：人为停止时是否释放当前已领取但未产出 PR 的任务；默认 `on`
- `local-claim=on|off`：是否在本地 main 的 tasks.md 先做“占位标记”再同步远端；默认 `on`
- `local-finish=on|off`：当某个 Issue 已完成并合并后，是否回填本地 tasks.md 的完成状态；默认 `on`
- `verify=on|off`：是否在本地执行“最小验证”命令；默认 `off`（保持命令通用，由 CI 或用户手动验证）
- `verify-cmd=<cmd>`：本地最小验证命令（仅当 verify=on 时生效），例如 `cargo test` / `bash scripts/verify.sh`

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

领取采用两阶段占位（默认启用 `local-claim=on`）：

### 3.1 本地占位（main）

若 `local-claim=on`：
- 必须先同步本地 main 到最新：
  - `git fetch origin main --prune`
  - `git switch main`
  - `git pull --ff-only`
- 在 `.trae/specs/<change-id>/tasks.md` 对应任务行末追加占位标记（不新增文件）：
  - `@claimed-by:<login>`
  - `@claim-at:<yyyy-mm-dd HH:MM>`
  - `@issue:#<number>`
  - `@branch:agent/issue-<number>-<yyyymmdd>`
- 将上述回填提交到 main（只允许提交 tasks.md 的变更）：
  - `chore(claim): 标记任务已领取（Issue #<number>）`
  - trailers：`Agent-Task` / `Agent-Decision`
- 不允许 push；push 仍由后续 PR 流程完成

若本地占位提交失败（例如冲突/工作区不干净），停止领取并提示用户先对齐 main。

### 3.2 远端占位（GitHub）

对选中的 Issue 执行：
- 添加 label：`status/in-progress`
- 若 `assignee!=none`：设置 assignee（默认 @me）
- 在 issue 留言（可选）：`Claimed by <user> on <date>, branch: agent/issue-<number>-<yyyymmdd>`

若远端占位失败：
- 必须提示用户执行 `/smart-release issue=<number>` 放回
- 若启用了本地占位：提示用户回滚本地占位提交（或后续用 smart-sync 对齐回滚）

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
   - 若 `verify=on`：执行最小验证（项目约定的 print-based / 脚本验证），命令由 `verify-cmd` 指定
   - 若 `verify=off`：跳过本地验证，仅确保已生成可复现的验证脚本/用例，并在 PR 中说明如何验证
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
     - Commits（Linkable）：以 markdown link 列表形式贴出（让 reviewer 可点击跳转到每个 commit）
       - 取 repo url：优先 `gh repo view --json url -q .url`，否则从 `git remote get-url origin` 推断
       - 取 commits：`git log --pretty=format:%H::%s origin/main..HEAD`
       - 逐行渲染为 markdown：
         - `- [<sha7>](<repo_url>/commit/<sha40>) <subject>`
     - Commits（Raw，可选）：如需要保留“命令原始输出”，再追加 `git log --oneline origin/main..HEAD` 的逐行文本
     - Files Changed（`git diff --name-only origin/main..HEAD` 实际输出）
     - `Closes: #<number>`
5. 合并策略（必须）
   - 若可直接合并：执行 merge，并在合并完成后删除分支（见“分支删除说明”）
   - 否则若指定 `automerge`：尝试开启 auto-merge（失败则输出阻塞原因并停止）
   - 若不可直接合并且未指定 `automerge`：停止并输出阻塞原因（等待 CI/Review），避免继续领取导致 in-progress 过多
   - 合并成功后同步本地 main（fast-forward）

### 分支删除说明（合并后）

合并后“远程分支是否自动删除”不是必然行为，依赖于仓库设置与合并方式：

- 推荐：在 GitHub 仓库设置开启 Automatically delete head branches（PR 合并后自动删除 head 分支）。
- 若使用 `gh pr merge` 执行合并：合并命令可追加 `--delete-branch`（若当前 gh 版本支持）以在合并后删除远程分支。
- 若分支已合并但仍残留：手动删除远程分支 `git push origin --delete <branch>`；本地分支可再 `git branch -d <branch>` 清理（可选）。

## 5) 完成后继续领取（必须）

当 PR 已合并（或 auto-merge 已开启）后：
- 若 issue 已 closed：继续领取下一个
- 若仍 open：
  - 保持 `status/in-progress`
  - 输出阻塞原因（例如等待 review/CI）
  - 不再领取新任务，避免并发过多导致混乱

若 `local-finish=on` 且 issue 已 closed：
- 切回并同步本地 main：
  - `git fetch origin main --prune`
  - `git switch main`
  - `git pull --ff-only`
- 在 `.trae/specs/<change-id>/tasks.md` 中找到对应 Task（通过 `（Issue #<number>）` 匹配）并回填：
  - 将顶层 Task 标记为 `[x]`
  - 将子项标记为 `[x]`
  - 移除该行的 `@claimed-by/@claim-at/@issue/@branch` 标记（避免“已完成但仍显示领取中”）
- 将回填提交到 main（仅允许提交 tasks.md）：
  - `chore(claim): 标记任务已完成（Issue #<number>）`
  - trailers：`Agent-Task` / `Agent-Decision`

## 6) 无可领取任务时的行为（必须）

当出现以下情况之一：
- 所有 open issues 都已 `status/in-progress`（有人在做）
- 只有存在依赖未完成的 issues（不可领取）
- 或者按 milestone 过滤后没有候选

默认行为（推荐）：
- 输出“无可领取任务”的原因摘要并退出
- 若原因是“依赖未完成”：
  - 必须列出阻塞的 issue 及其 `Depends on` 指向的未完成上游
  - 必须说明当前有哪些任务正在执行中（`status/in-progress` 列表）
- 不进入循环等待，不进行轮询空转

若 `watch=on`（非默认）：
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
