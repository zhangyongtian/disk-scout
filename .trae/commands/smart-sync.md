---
name: smart-sync
description: 对齐 specs 的任务与验收清单到 GitHub（plan/apply；支持幂等写入与进度回填）。
---

你将作为“任务同步管家”，将本地清单对齐到 GitHub（plan/apply 两阶段），并在 apply 时回填本地文件。命令可重复执行且不会重复创建（幂等）。

使用方式：
- 默认：plan（只输出计划，不写 GitHub、不改本地）
- 带 `apply`：执行写入与回填

参数（主干）：
- `repo=<owner/name>`：默认从 origin 推断
- `change=<change-id>`：默认取 `.trae/specs/` 最新目录
- `apply`：执行写入/回填（未带则仅 plan）
- `base=<branch>`：回填提交的基线分支（默认 `main`）
- `commit=on|off`：apply 后是否提交本地回填到 `<base>`（默认 `on`）
- `commit-scope=sync|trae|repo`：限制允许提交的文件范围（默认 `sync`）
- `push=on|off`：apply+commit 后是否推送 `<base>` 到 `origin/<base>`（默认 `off`）

安全原则（必须）：
- 必须先 plan 并得到用户确认，才允许 apply 写 GitHub
- apply 必须幂等：重复运行只会复用/更新，不会重复创建
- 任何 git 同步只允许 `--ff-only`；失败即停止；禁止强推
- checklist 只会从 `[ ] -> [-]`（待验收），不会自动改为 `[x]`

## 1) 输入与解析（必须）
读取：
- `.trae/specs/<change>/tasks.md`（必须）
- `.trae/specs/<change>/checklist.md`（可选）

tasks.md 顶层任务 => GitHub Issue（一对一）。
Issue 匹配主键（按优先级）：
1. 标题已包含 `（Issue #N）`：直接绑定 #N
2. 否则用 Order：标题前缀 `[NNN]`（推荐且应唯一）
3. 仅在用户明确允许时才可用标题相似度兜底（需输出风险提示）

## 2) plan（默认，必须先执行）
输出一份“将要发生的变化”清单：
- 将创建/复用/更新的 issues
- 将回填到 tasks.md 的 `（Issue #N）` 与勾选变更（若 issue 已 closed）
- checklist.md 预测：哪些条目会从 `[ ] -> [-]`（@issues 全部 closed）

本阶段禁止写 GitHub、禁止改本地文件。

## 3) apply（仅当带 apply）
### 3.1 预检（必须）
- `gh` 可用且已登录；否则停止并输出可复制文本

### 3.2 同步基线分支（必须）
- 工作区必须干净，否则停止
- 切到 `<base>` 并快进同步：
  - `git fetch origin <base> --prune`
  - `git pull --ff-only`

### 3.3 写入 GitHub（幂等）
- 拉取远端 issues（open/closed）用于匹配与复用
- 对每个本地任务：
  - 若绑定了 `（Issue #N）`：更新 #N（只补齐缺失字段/标签/里程碑，不覆盖用户正文）
  - 否则按 `[NNN]` 查找可复用 issue；命中则回填 `（Issue #N）`
  - 否则创建新 issue，并回填 `（Issue #N）`

### 3.4 回填本地文件（必须）
- tasks.md：回填 `（Issue #N）`
- tasks.md：若对应 issue 已 closed，则将该顶层任务标为 `[x]`
- checklist.md（若存在且含 `@issues:`）：
  - 关联 issues 全部 closed 且当前为 `[ ]`：改为 `[-]`
  - 其它情况保持不变（不会自动改为 `[x]`）

### 3.5 提交与推送（安全阀）
若 `commit=on`：
- 按 `commit-scope` 校验允许提交的文件范围：
  - `sync`：仅允许
    - `.trae/specs/<change>/tasks.md`
    - `.trae/specs/<change>/checklist.md`（若存在）
  - `trae`：允许 `.trae/` 下变更
  - `repo`：允许全仓库变更（需用户明确要求）
- 发现超范围变更：停止并输出文件清单
- 在 `<base>` 创建提交：`chore(sync): 对齐 specs 与 GitHub 任务状态`

若 `push=on`：
- `git push origin <base>`
- 推送失败：停止并提示重新同步后再试（禁止强推）

## 4) 输出结果（必须）
- 新建/复用/更新的 issues 列表（#号与链接）
- 本地是否已回填、是否已提交、是否已推送
- 下一步：保持 `git status` 干净；从最新 `<base>` 开始后续分支操作
