---
name: smart-claim-verified
description: smart-claim 的“带测试”模式（verify=on），通过 verify-cmd 注入本地验证命令。
---

这是 `/smart-claim` 的一个“带测试”变体，用于你本地已经具备项目运行环境（例如 Rust 的 cargo）时：
- 在开发每个 Issue 之后，先执行对应的验证脚本，验证通过才会继续提交/PR/合并
- 验证命令保持可注入、可替换，避免把某语言/框架写死在通用流程里

默认规则（通用）：
- `verify=on`
- 必须显式提供 `verify-cmd=<cmd>`，否则视为配置缺失并停止（避免在不同语言项目里做错误假设）

用法示例：
- Rust：`/smart-claim-verified verify-cmd="cargo test" automerge`
- Java（Maven）：`/smart-claim-verified verify-cmd="mvn test" automerge`
- Java（Gradle）：`/smart-claim-verified verify-cmd="./gradlew test" automerge`
- 只跑 1 个任务（用于验证流程）：`/smart-claim-verified verify-cmd="<cmd>" limit=1 automerge`

执行逻辑：
1) 先按 `/smart-claim` 的规则挑选可领取 Issue
2) 进入开发循环后，验证阶段强制执行：
   - 执行 `verify-cmd`
3) 验证失败则停止（不提交、不建 PR、不合并），把失败输出贴到本次总结中

注意：
- 若你希望更细粒度控制，可直接使用 `/smart-claim verify=on verify-cmd="<cmd>" ...`
