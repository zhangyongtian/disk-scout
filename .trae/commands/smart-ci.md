---
name: smart-ci
description: 手动触发/管理 CI 与 Release 构建（避免每次提交自动跑 Actions）。
---

你将作为“CI 管家”，把仓库的 GitHub Actions 从“每次提交自动跑”调整为“按需手动触发”，并提供一条命令式入口来触发 CI 或 Release 构建。

目标：
- 日常开发默认不跑 Actions
- 需要验证时手动触发 CI
- 发布时手动触发 Release（产出 Windows exe zip / Linux tar.gz / sha256.txt）

可选参数：
- `repo=<owner/name>`：默认从 origin 推断
- `mode=manual|auto`：默认 manual
- `run=ci|release`：触发哪种工作流
- `tag=vX.Y.Z`：release 使用；若不存在则提示先创建并 push tag

## 1) 切换 Actions 触发模式（必须支持）

### mode=manual（推荐默认）
将工作流触发修改为仅 `workflow_dispatch`（不在 push/PR 上自动跑）。

### mode=auto（可选）
将工作流触发改回 `push/pull_request`（仅在用户明确要求时做）。

执行约束：
- 只有当用户明确指定 `mode=...` 时才允许改动工作流文件
- 修改后必须输出变更点（哪些 workflow 的 on 事件变了）

## 2) 手动触发 CI（run=ci）

前置：
- `gh` 可用且已登录

执行：
- `gh workflow run ci.yml -R <repo>`

输出：
- 触发结果与 run 链接（如可获取）

## 3) 手动触发 Release（run=release）

前置：
- `gh` 可用且已登录
- 必须提供 `tag=vX.Y.Z`

执行策略：
1. 检查 tag 是否存在于远端：
   - `git fetch --tags`
   - `git tag -l <tag>`
2. 若 tag 不存在：停止并提示用户按发布流程创建并 push tag
3. 触发 release workflow：
   - `gh workflow run release.yml -R <repo> -f tag=<tag>`

输出：
- Release 工作流 run 链接（如可获取）
- 期望产物清单：Windows zip（含 exe）、Linux tar.gz、sha256.txt

