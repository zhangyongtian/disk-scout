---
name: smart-project
description: 创建/同步 GitHub Project 看板，并把指定 Issues/PRs 加入（可选自动设置 Status）。
---

你将作为“看板管家”，在 GitHub Projects（v2）中创建或复用一个 Project，并把仓库的任务 Issue/PR 加入到看板中，便于多 agent 并行协作与进度追踪。

注意：`gh project` 需要 token scope `project`。若缺失，先提示用户执行：
- `gh auth refresh -s project`

可选参数（用户在命令后追加即可）：
- `owner=<owner>`：Project owner，默认从仓库 owner 推断
- `repo=<owner/name>`：默认从 git remote 推断
- `title=<text>`：Project 标题，默认 `disk-scout Roadmap`
- `issues=1,2,3`：要加入的 issue 编号列表；省略则自动列出 open issues 并加入带 `task` label 的
- `dry-run`：只输出计划与命令，不实际修改

## 1) 前置检查（必须）

1. 确认 `gh` 登录且具备 `project` scope：
   - `gh auth status -h github.com`
2. 若 scope 缺失，停止并提示用户 refresh

## 2) 创建或复用 Project（必须）

1. 列出 owner 下现有 projects：
   - `gh project list --owner <owner>`
2. 若存在标题一致的 project，复用；否则创建：
   - `gh project create --owner <owner> --title "<title>"`

## 3) 字段与工作流（可选）

1. 确保存在 `Status` 字段（单选）并包含：
   - `Backlog`、`In Progress`、`In Review`、`Done`
2. 若已存在则不改动；若不存在则创建并写入选项

## 4) 添加 items（必须）

1. 若提供 `issues=...`：
   - 逐个执行：`gh project item-add <project-number> --owner <owner> --url <issue-url>`
2. 若未提供：
   - 自动选择带 `task` label 的 open issues 加入

## 5) 输出总结（必须）

输出：
- Project 链接
- 加入的 issues 列表
- 推荐的用法：与 `/smart-bootstrap` 配合，在创建 issues 后自动加入 project

