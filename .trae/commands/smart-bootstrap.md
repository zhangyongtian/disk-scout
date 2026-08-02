---
name: smart-bootstrap
description: 将 spec/tasks 同步为 GitHub 的 Milestone/Labels/Issues（为多 agent 并行开发做准备）。
---

你将作为“项目启动管家”，把本地 `.trae/specs/<change-id>/` 中的 spec 与 tasks，同步为 GitHub 仓库内可协作的里程碑与任务 Issue，并输出可用于并行开发的任务分配建议。

本命令优先面向 GitHub（使用 `gh`），但必须在任何一步失败时提供可复制的手动操作文本。

可选参数（用户在命令后追加即可）：
- `repo=<owner/name>`：默认从 `git remote get-url origin` 推断
- `change=<change-id>`：默认取 `.trae/specs/` 下最新一个目录
- `milestones=v0.1.0,v0.2.0,v1.0.0`：里程碑标题列表；可省略
- `dry-run`：只输出将要执行的动作与 gh 命令，不实际创建

## 1) 前置检查（必须）

1. 确认 `gh` 可用且已登录：
   - `command -v gh`
   - `gh auth status -h github.com`
2. 确认仓库存在且可访问：
   - `gh repo view <repo> --json name,url,defaultBranchRef`
3. 解析 change-id：
   - 若用户提供 `change=...`，使用该目录
   - 否则，从 `.trae/specs/` 选择最新的一个目录作为 change-id
4. 读取并理解：
   - `.trae/specs/<change>/spec.md`
   - `.trae/specs/<change>/tasks.md`

## 2) 标签策略（必须）

1. 确保存在以下 labels（若无则创建；若存在则保持不变）：
   - `task`
   - `scope/cli`、`scope/scan`、`scope/report`、`scope/filter`、`scope/ci`、`scope/docs`（按实际 tasks 调整）
   - `priority/high`、`priority/medium`、`priority/low`
2. 创建方式（示例）：
   - `gh label create "task" -R <repo> --color 1d76db --description "可执行任务" --force`

## 3) 里程碑同步（必须）

1. 若用户提供 `milestones=...`：
   - 确保这些 milestones 存在（不存在则创建）
2. 若未提供：
   - 从 spec/任务拆分中推导最少两个 milestones（例如 v0.1.0/v0.2.0）

## 4) 从 tasks.md 生成 Issues（必须）

1. 从 tasks.md 提取每个顶层 Task，生成一个 Issue：
   - 标题：`[<Order>] <任务名>`（Order 若无则用 999）
   - Body 必须包含：
     - Order、Scope
     - Summary（为什么/目标/边界）
     - Definition of Done（可验证的 checklist）
2. 自动推断：
   - Scope：根据任务内容映射到 `scope/*` label
   - Priority：根据依赖与用户价值设定 high/medium/low
   - Milestone：按任务所属版本分配（例如 v0.1.0：核心功能；v0.2.0：CI/Release）
3. 去重策略（必须）：
   - 在创建前执行：`gh issue list --state open --limit 50 --json number,title,url`
   - 若已存在高度相似标题的 issue，优先复用并停止重复创建

## 5) 结果回填与输出（必须）

1. 在本地 `.trae/specs/<change>/tasks.md` 中把每个 Task 标题追加 `（Issue #N）`
2. 输出：
   - 创建/复用的 Issue 列表（编号、链接、milestone、labels）
   - 推荐的并行开发分配（哪些 issue 可并行，哪些需顺序）
   - 下一步建议命令：
     - `/smart-branch task=<...> scope=<...>`
     - `/smart-commit`
     - `/smart-pr issue=<N> order=<NNN> task=<...>`

