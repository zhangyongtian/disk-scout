---
name: smart-release
description: 将已领取但未完成的任务放回队列（回滚 status/in-progress 到 status/todo，可选取消 assignee）。
---

你将作为“任务放回管家”，把当前开发者已领取但暂时不继续执行的任务放回队列，供其他人领取。

约束：
- 仅允许对 open issue 执行放回
- 若 issue 已有关联 PR（open PR 且 title/body 显示关联），默认不放回以避免丢上下文；除非用户显式指定 `force`

可选参数：
- `repo=<owner/name>`：默认从 origin 推断
- `issue=<number>`：要放回的 issue 编号；若省略则自动选择“由当前用户领取的 in-progress issue”
- `force`：即使已有 PR 也强制放回（慎用）
- `unassign=on|off`：是否取消 assignee；默认 on

## 1) 前置检查（必须）

1. `gh` 可用且已登录：
   - `command -v gh`
   - `gh auth status -h github.com`
2. 确认仓库远端：
   - `git remote -v`

## 2) 选择要放回的任务（必须）

1. 若提供 `issue=<number>`：选择该 issue
2. 否则：
   - 从 open issues 中筛选带 `status/in-progress` 且 assignee 包含当前用户的 issue
   - 若多条命中，按更新时间最新的一条

## 3) 放回动作（必须）

对目标 issue 执行：
- 移除 `status/in-progress`，添加 `status/todo`
- 若 `unassign=on`：取消 assignee（仅对当前用户）
- 留言（可选）：`Released by <user> on <date>`

若检测到该 issue 已有关联 PR：
- 默认停止并提示使用 `force`（或建议保持 in-progress）

## 4) 输出总结（必须）

- issue 编号与链接
- 是否取消 assignee
- 是否检测到关联 PR

