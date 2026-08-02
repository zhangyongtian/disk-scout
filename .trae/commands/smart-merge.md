---
name: smart-merge
description: 自动合并 PR 并同步本地 main（可按标题关键词或编号选择 PR）。
---

你将作为“合并管家”：前置检查 → 选择 PR → 合并（或开启 auto-merge）→ 可选同步本地。

参数：
- `pr=<number>`：指定 PR（最高优先级）
- `q=<keyword>`：按标题关键词匹配（不区分大小写；多条命中取 updatedAt 最新）
- `base=<branch>`：目标分支（默认 `main`）
- `method=merge|squash|rebase`：合并方式（默认 `merge`）
- `pull=on|off`：合并后是否同步本地 `<base>`（默认 `on`）

安全检查（必须）：
- `gh` 可用且已登录：`gh auth status -h github.com`
- 工作区必须干净：`git status --porcelain=v1`
- 必须存在远端 `origin`
- 跳过 draft PR

选择规则：
- 提供 `pr`：选该 PR
- 否则提供 `q`：标题匹配，取最新
- 否则：只处理带 `automerge` 标签的 PR，取最新；找不到则停止并输出候选列表

合并策略：
- `mergeStateStatus=CLEAN`：按 `method` 直接合并，并删除远端分支（必须）
- 否则：开启 GitHub Auto-merge（不绕过 review/CI 规则，只提示阻塞原因）

分支清理（必须）：

1) 远端分支（必须尝试删除）

- 所有合并/auto-merge 命令必须携带 `--delete-branch`，让 GitHub 在合并后删除 PR 的 head 分支（仅对同仓库分支有效；来自 fork 时可能无权限删除）。

2) 本地分支（尽量自动清理）

- 合并后同步本地 `<base>` 之后，若本地存在 PR 的 head 分支且不是 `<base>`，删除本地分支以保持本地分支列表干净：
  - 若当前不在该分支：`git branch -d <head-branch>`
  - 若正在该分支：先切换到 `<base>` 再删
- 若 `-d` 失败（仍有未合并提交），停止并输出原因，让用户决定是否手动 `-D` 强删。

同步本地（pull=on）：
- `git fetch origin <base> --prune`
- `git switch <base> && git pull --ff-only`

输出（必须）：
- 选择的 PR（编号/标题/链接）、合并方式、结果（已合并/已开启 auto-merge/被阻塞原因）
- 当前分支与 `git log -1 --oneline`
