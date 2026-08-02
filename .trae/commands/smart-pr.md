---
name: smart-pr
description: 将当前分支的提交推送到远端并创建 PR（支持可选自动合并）。
---

你将作为“PR 管家”，把当前分支的提交整理成一个可审查的 PR：创建/切换分支、推送远端、生成 PR 标题与描述，并在条件满足时可开启自动合并。

运行前置：
- 工作区必须干净（无未提交变更）
- 必须存在远端 `origin`

可选参数（用户在命令后追加即可）：
- `task=<id-or-desc>`：用于生成分支名与 PR 标题，例如 `task=kb-import`
- `order=<NNN>`：合并顺序号（3 位数字推荐，如 001/010）；建议总是提供；若指定 `automerge` 必须同时提供
- `issue=<number>`：关联的任务编号（可选，常见为 Issue 编号）；提供后 PR body 追加 `Closes: #<issue>`
- `base=<branch>`：目标基线分支，默认 `main`
- `automerge`：在 PR 创建后尝试开启 Auto-merge（需要仓库允许且 CI/规则通过）

严格遵循：`.trae/rules/git-commit-message.md`（本命令不会改写 commit message，但会用其风格生成 PR 标题/描述）。

## 1) 采集状态（必须）

1. 确认工作区干净：
   - `git status --porcelain=v1`
2. 确认远端：
   - `git remote -v`
3. 解析参数：
   - `base` 缺省为 `main`
   - `task` 缺省为 `misc`
   - `order` 缺省为空
   - `issue` 缺省为空

若指定了 `automerge` 但未提供 `order`，停止并提示补齐 `order`（用于合并队列排序）。

若工作区不干净，停止并提示先运行 `/smart-commit` 或手动提交。

## 2) 分支策略（必须）

1. 获取当前分支：
   - `git rev-parse --abbrev-ref HEAD`
2. 若当前分支是 `<base>`（默认 main），则从当前 HEAD 创建新分支并切换：
   - 分支名：`agent/<task>-<yyyymmdd>`（task 需做 slug 化：小写、非字母数字替换为 `-`、连续 `-` 合并）
   - `git switch -c <branch>`
3. 若当前分支不是 `<base>`，保持不变，视为已在 feature/agent 分支上。

## 3) 生成 PR 内容（必须）

1. 列出相对 base 的提交与变更范围：
   - `git fetch origin <base> --prune`（失败则继续，但要提示可能不准）
   - `git log --oneline origin/<base>..HEAD`
   - 为了让 PR 里的 commit 可点击跳转，生成一份“可跳转 commits 列表”（建议）：
     - 取 repo url：优先 `gh repo view --json url -q .url`，否则从 `git remote get-url origin` 推断
     - 取 commits：`git log --pretty=format:%H::%s origin/<base>..HEAD`
     - 逐行渲染为 markdown：
       - `- [<sha7>](<repo_url>/commit/<sha40>) <subject>`
   - `git diff --name-only origin/<base>..HEAD`
   - 生成 PR body 时必须把上述两条命令的输出“写入文本”，禁止把命令替换表达式（如 `$(git log ...)`）原样写入 body
   - 注意：若使用 heredoc 写文件，禁止使用 `cat <<'EOF'`（单引号 heredoc 会阻止 `$(...)` 与 `$VAR` 展开）；应先执行命令得到输出再拼接文本，或使用不带引号的 `<<EOF`
2. PR 标题规则：
   - 优先使用最新一条提交的 header 作为 PR 标题
   - 若最新提交不适合作为标题，使用：`feat(infra): <subject>` 或 `docs(config): <subject>`（按实际变更归类）
3. PR 描述（body）必须包含：
   - Task：来自 `task` 参数
   - Order：来自 `order` 参数（若提供）
   - Summary：用 3–6 条要点概述本次变更意图（不要只罗列文件名）
   - Commits（Linkable）：以 markdown link 列表形式贴出（让 reviewer 可点击跳转到每个 commit）
   - Commits（Raw，可选）：如需要保留“命令原始输出”，再追加 `git log --oneline origin/<base>..HEAD` 的逐行文本
   - Files Changed：贴 `git diff --name-only origin/<base>..HEAD`
   - Known Limitations / Follow-ups：如有则写
   - 若提供 `issue`：
     - 仅当本次变更与该任务一致时，追加 `Closes: #<issue>`
     - 若一致性不足（无法从 diff/提交/描述判断），改为 `Refs: #<issue>` 或不关联

4. PR body 模板（必须按此结构生成）

```
Task: <task>
Order: <NNN>
Issue: #<issue>（可选）

Summary:
- ...

Commits (Linkable):
- [<sha7>](<repo_url>/commit/<sha40>) <subject>

Commits (Raw, optional):
<把 git log --oneline origin/<base>..HEAD 的实际输出逐行贴在这里>

Files Changed:
<把 git diff --name-only origin/<base>..HEAD 的实际输出逐行贴在这里>

Known Limitations / Follow-ups:
- （可选）

Closes: #<issue>（可选）
```

一致性判断建议：
- 变更文件路径与 issue scope 匹配（例如 api/db/workflow/infra/docs 等）
- PR summary 与 issue title/summary 关键词高度相关
- 不要把无关代码变更硬关联到 issue（会污染任务追踪与队列顺序）

## 4) 推送与创建 PR（必须）

1. 推送分支到 origin：
   - `git push -u origin HEAD`
2. 创建 PR（优先使用 GitHub CLI `gh`）：
   - 先检测：`command -v gh`
   - 若可用且已登录：
     - 创建 PR：`gh pr create --base <base> --head <branch> --title <title> --body-file <file>`
     - 若指定 `automerge`：给 PR 加标签 `automerge`
     - 若需要与 GitHub 上的队列工具/脚本配合：可额外加标签 `queue/<order>`（可选）
   - 若 `gh` 不可用或未登录：
     - 输出远端分支地址
     - 输出 PR 标题与 PR body 内容
     - 提示用户在 GitHub Web 上手动创建 PR（停止自动化）

## 5) 可选：开启 Auto-merge（仅当用户指定 automerge）

1. 仅当 PR 已成功创建且 `gh` 可用时执行：
   - `gh pr merge --auto --merge`
   - 说明：本命令只负责创建 PR/可选开启 auto-merge，不保证“合并后删除分支”。删除分支由仓库设置或后续合并流程负责（见下方说明）。
2. 若仓库策略要求审批或检查未通过，不要强行合并；仅输出当前阻塞条件（例如缺少 review、CI 未通过）。

## 分支删除说明（合并后）

合并后“远程分支是否自动删除”取决于仓库设置与合并方式，本命令本身不会主动删除分支：

- 推荐：在 GitHub 仓库设置开启 Automatically delete head branches（PR 合并后自动删除 head 分支）。
- 若使用 `gh pr merge` 执行合并：可在合并命令中追加 `--delete-branch`（若当前 gh 版本支持）以在合并后删除远程分支。
- 若分支已合并但仍残留：可手动删除远程分支 `git push origin --delete <branch>`；本地分支可再 `git branch -d <branch>` 清理（可选）。

## 6) 输出总结（必须）

- 当前分支名、远端分支名
- PR 链接（若已创建）
- 是否已启用 auto-merge（若用户指定）
- Commits（必须）：贴 `git log --oneline origin/<base>..HEAD` 的输出
- Files Changed（必须）：贴 `git diff --name-only origin/<base>..HEAD` 的输出
- 下一步建议（例如：等待 CI、请求 review、补充说明）
