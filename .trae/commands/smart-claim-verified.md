---
name: smart-claim-verified
description: smart-claim 的“带测试”模式（verify=on），通过 verify-cmd 注入本地验证命令。
---

这是 `/smart-claim` 的参数预设版本：
- 强制 `verify=on`
- 必须提供 `verify-cmd="<cmd>"`，否则停止（避免对不同语言项目做错误假设）
- 其余参数与 `/smart-claim` 一致并透传（repo/milestone/assignee/automerge/limit 等）

示例：
- `/smart-claim-verified verify-cmd="cargo test" automerge`
- `/smart-claim-verified verify-cmd="mvn test" limit=1`

等价写法：
- `/smart-claim verify=on verify-cmd="<cmd>" ...`
