---
name: smart-claim-verified
description: smart-claim 的“带测试”模式（verify=on），按 Issue 编号自动选择对应的验证脚本。
---

这是 `/smart-claim` 的一个“带测试”变体，用于你本地已经具备项目运行环境（例如 Rust 的 cargo）时：
- 在开发每个 Issue 之后，先执行对应的验证脚本，验证通过才会继续提交/PR/合并
- 验证命令保持可注入、可替换，避免把某语言/框架写死在通用流程里

默认规则（disk-scout 项目约定）：
- `verify=on`
- `verify-cmd` 使用：`bash disk-scout/scripts/verify_issue_<number>.sh`

用法示例：
- 只跑 1 个任务（推荐用于验证流程）：`/smart-claim-verified limit=1 automerge`
- 领取并开发 Issue #3（过滤能力）：`/smart-claim-verified limit=1 automerge`

执行逻辑：
1) 先按 `/smart-claim` 的规则挑选可领取 Issue
2) 进入开发循环后，验证阶段强制执行：
   - `bash disk-scout/scripts/verify_issue_<number>.sh`
3) 验证失败则停止（不提交、不建 PR、不合并），把失败输出贴到本次总结中

注意：
- 若脚本不存在，视为验证缺失，必须停止并提示先补齐验证脚本
- 若你希望覆盖默认验证命令，可直接回退使用 `/smart-claim verify=on verify-cmd="<cmd>" ...`
