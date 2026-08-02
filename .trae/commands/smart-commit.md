---
name: smart-commit
description: 基于项目提交规范自动分析 diff、给出拆分方案，并按 atomic commit 自动创建提交（可 dry-run）。
---

你将作为“提交管家”：采集状态 → 生成 atomic commit 拆分方案 →（可选）自动落地提交 → 复核结果。

严格遵循：`.trae/rules/git-commit-message.md`。

运行模式：
- 默认：执行提交
- `dry-run`：只输出拆分方案与 commit message，不执行 `git add/commit`

参数：
- `task=<id-or-desc>`：写入 trailers 的任务标识（默认 `misc`）
- `scope=<scope>`：影响范围（可选；缺省可根据文件路径推断）

安全检查（必须）：
- 禁止在 `main` 分支提交
- 未跟踪文件必须列出；生成物/临时文件/敏感文件一律不提交
- 每个 commit 前必须复核 `git diff --staged` 只包含该提交的语义变更
- 本命令只做本地提交，禁止 `git push`（需要 PR 用 `/smart-pr`）

主流程：

1) 采集状态
- `git rev-parse --abbrev-ref HEAD`（若为 `main`：停止）
- `git status --porcelain=v1`
- `git diff --staged`（优先）与 `git diff`（补充）

2) 生成 commit plan（atomic）
- 按“语义边界优先、目录次之”拆分
- 功能/修复/重构/格式化/依赖/文档不得混在同一提交
- 对每个 planned commit 输出：C1/C2、包含文件、目的说明、commit message（含必要 trailers）

3) 落地提交（非 dry-run）
- 对每个 commit：
  - `git reset` 确保暂存区干净
  - `git add <files...>` 仅暂存本次提交文件
  - `git diff --staged` 复核
  - 提交（完整多行 message，含 trailers）
  - `git show -1 --name-only --stat --oneline` 复核
- 若混入无关变更：`git reset --soft HEAD~1` 回退并重新拆分

4) 输出（必须）
- `git log --oneline -n <N>`
- `git status --porcelain=v1`
- 无法自动归类的变更与下一步建议
