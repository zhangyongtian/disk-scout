# Tasks
- [x] Task 1: 初始化 Rust CLI 工程与基本命令结构
  - [x] 创建 crate（bin），确定项目名为 disk-scout
  - [x] 设计 CLI：`scan <path>` 与必要参数（top-files/top-dirs/min-size/format/ignore/ignore-file）
  - [x] 提供 `--version` / `--help` 输出（由 CLI 框架生成即可）

- [ ] Task 2: 实现只读扫描与统计模型
  - [ ] 实现目录遍历：默认不跟随符号链接，遇到权限/IO 错误不中断
  - [ ] 实现文件大小统计与目录大小汇总（按目录聚合）
  - [ ] 维护 Top N 最大文件与 Top N 最大目录（避免全量排序）

- [ ] Task 3: 实现报告输出（text/json）
  - [ ] 文本报告：包含扫描元信息、统计信息、Top 文件/目录列表
  - [ ] JSON 报告：meta + top_files + top_dirs（字段与 spec 对齐）
  - [ ] 大小格式化：text 使用人类可读单位，json 使用 bytes

- [ ] Task 4: 实现过滤能力（min-size 与 ignore）
  - [ ] 解析 `--min-size`（支持 KB/MB/GB 或 KiB/MiB/GiB）
  - [ ] 支持 `--ignore` 多次指定
  - [ ] 支持 `--ignore-file` 读取忽略规则

- [ ] Task 5: 发布与多平台产物（CI / Release）
  - [ ] GitHub Actions：PR/Push 进行 build/test
  - [ ] Tag/Release 时产出多平台可运行包：Linux tar.gz + Windows zip（含 .exe）
  - [ ] 生成 `sha256.txt` 并作为 Release asset 上传
  - [ ] 后续扩展：rpm/deb（可作为下一里程碑或后续 Task）

- [ ] Task 6: 最小验证脚本（打印式验证）
  - [ ] 添加一个硬编码路径/测试目录的脚本或最小示例，验证 Top N 输出与 JSON 结构
  - [ ] 在 CI 中运行最小验证（不依赖复杂测试框架）

# Task Dependencies
- Task 2 depends on Task 1
- Task 3 depends on Task 2
- Task 4 depends on Task 2
- Task 5 depends on Task 1
- Task 6 depends on Task 3 and Task 4
