---
name: smart-pr
description: 将当前分支的提交推送到远端并创建 PR（支持可选自动合并）。
---

你将作为“PR 管家”：检查状态 → 生成 PR 标题/描述 → 推送分支 → 创建 PR（可选开启 auto-merge）。

参数：
- `task=<id-or-desc>`：用于分支名与 PR 描述中的任务标识（默认 `misc`）
- `order=<NNN>`：合并顺序号（建议提供；若指定 `automerge` 则必须提供）
- `issue=<number>`：关联 Issue（仅在确定一致时写 `Closes`，否则写 `Refs` 或不写）
- `base=<branch>`：目标分支（默认 `main`）
- `automerge`：创建 PR 后尝试开启 GitHub Auto-merge
- `commits-raw=on|off`：是否在 PR 描述附纯文本 commits 列表（默认 `off`）

安全检查（必须）：
- 工作区必须干净：`git status --porcelain=v1`
- 必须存在远端 `origin`
- 若 `automerge` 且未提供 `order`：停止并提示补齐

主流程：

1) 分支策略
- 若当前在 `<base>`：创建并切到新分支（建议：`agent/<task>-<yyyymmdd>`）
- 否则：保持当前分支

2) 生成 PR 标题与描述
- 标题：优先用最新提交的 header；不合适时按变更类型生成（type/scope + 中文 subject）
- 描述必须包含：
  - Task / Order（如有）/ Issue（如有）
  - Summary：3–6 条说明“意图与影响”
  - Commits：列出 `origin/<base>..HEAD` 的提交（可链接优先；至少要可读）
  - Files Changed：列出 `git diff --name-only origin/<base>..HEAD`
  - Known Limitations / Follow-ups（如有）
- 禁止把命令文本（如 `$(git log ...)`）原样写进 PR body；必须写入实际输出内容

3) 推送与创建 PR
- `git push -u origin HEAD`
- 优先使用 `gh pr create --base <base> --head <branch> --title <title> --body-file <file>`
- 若 `gh` 不可用/未登录：输出分支与 PR 文本，提示用户手动创建

4) 可选：开启 auto-merge（仅当指定 `automerge`）
- `gh pr merge --auto --merge`
- 不绕过审批/检查；只输出当前阻塞条件

输出（必须）：
- 分支名、PR 链接（若创建成功）、是否开启 auto-merge
- Commits 与 Files Changed 的实际输出
