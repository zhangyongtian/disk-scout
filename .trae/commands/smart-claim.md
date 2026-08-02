---
name: smart-claim
description: 一键领取并完成一个 Issue（从 main 开分支→实现→提交→PR→合并/开启 auto-merge→同步 main），并按需循环直到无可领取任务。
---

你将作为“任务领取管家”，按主流程串行处理 Issue：从 GitHub 领取一个可执行 Issue → 从 main 开分支 → 实现 →（可选）本地验证 → 提交 → 建 PR → 合并或开启 auto-merge → 同步 main。一次只推进一个 Issue，避免上下文混乱。

约束（必须）：
- 尊重依赖：Issue 声明 `Depends on: #X,#Y` 时，只有依赖全部 closed 才可领取
- 默认不抢占：Issue 已带 `status/in-progress` 或已分配给其他人则跳过
- 写 GitHub（加 label/assign/建 PR/合并）必须使用 `gh`；否则输出可复制命令并停止
- 只从 main 开新分支；开始前工作区必须干净

参数（主干）：
- `repo=<owner/name>`：默认从 origin 推断
- `milestone=<title>`：只从某里程碑领取
- `assignee=@me|<login>|none`：默认 `@me`
- `automerge`：创建 PR 后尝试开启 auto-merge；若无法开启且不能直接合并则停止
- `limit=<N>`：最多连续处理 N 个 Issue（默认 999）
- `verify=on|off`：是否执行本地最小验证（默认 off）
- `verify-cmd=<cmd>`：verify=on 时必填，例如 `cargo test` / `mvn test`

主流程（对每个 Issue 重复）：

1) 前置检查（必须）
- `gh auth status -h github.com` 通过
- `git status --porcelain=v1` 为空
- 同步 main：`git fetch origin main --prune && git switch main && git pull --ff-only`

2) 选择可领取 Issue（必须）
- 候选：open 且带 `task`，且不包含 `status/in-progress` / `status/done`
- milestone 参数存在则按 milestone 过滤
- 依赖检查：`Depends on` 指向的 issues 必须全部 closed
- 排序：优先 `Order: NNN` 升序，其次 issue number 升序

3) 领取并标识（必须）
- 给 Issue 加 label：`status/in-progress`
- 设置 assignee（除非 `assignee=none`）
- 可选留言：`Claimed by <user>, branch: agent/issue-<number>-<yyyymmdd>`

4) 从 main 开分支（必须）
- `git fetch origin main --prune`
- `git switch main && git pull --ff-only`
- `git switch -c agent/issue-<number>-<yyyymmdd>`

5) 实现与验证（必须）
- 按 Issue 的 Summary/DoD 实现
- 若 `verify=on`：执行 `verify-cmd`；失败则停止（不提交、不建 PR），并输出失败日志摘要
- 若 `verify=off`：在 PR 中写清验证方式（或说明依赖 CI）

6) 提交（必须）
- 按 `.trae/rules/git-commit-message.md` 生成原子提交
- trailers 至少包含：
  - `Agent-Task: Issue #<number> - <title>`
  - `Agent-Decision: <关键决策与原因>`
- 仅当确实完成该 Issue 时才加：`Closes: #<number>`

7) 建 PR（必须）
- `git push -u origin HEAD`
- PR body 至少包含：
  - Summary（做了什么/为什么）
  - How to verify（verify=off 时必须）
  - `Closes: #<number>`（仅当该 PR 完成 Issue）

8) 合并或开启 auto-merge（必须）
- 若可直接合并：`gh pr merge --merge`（支持时附带 `--delete-branch`）
- 否则若传了 `automerge`：尝试 `gh pr merge --auto --merge`
- 若不可直接合并且未传 `automerge`：停止并输出阻塞原因（等待 CI/Review）

9) 同步 main（合并后必须）
- `git switch main && git pull --ff-only`

无可领取任务时：
- 输出原因并退出：全部 in-progress / 依赖未完成 / milestone 无候选
