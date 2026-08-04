# Tasks
- [ ] [001] Task 1: 为 Top files / Top dirs 添加独立滚动区域（Issue #32）
  - [ ] 调整 GUI 布局：将 Top files 与 Top dirs 放入各自的 ScrollArea（独立 id）
  - [ ] 保持现有按钮能力不变（Copy path / Delete / Confirm delete）

- [ ] [002] Task 2: min-size 增加单位选择并完成换算（Issue #33）
  - [ ] 增加 GUI 控件：min-size 数值输入 + 单位下拉（B/K/M/G 或 KiB/MiB/GiB）
  - [ ] 将“数值 + 单位”换算为 bytes，填充到 `ScanPlan.min_size`
  - [ ] 保持现有 `disk_scout::size::parse_size_bytes` 仍可用（CLI 语义不变）

- [ ] [003] Task 3: 修复 Windows 中文显示（Issue #34）
  - [ ] 在 GUI 启动时配置 egui 字体：优先加载 Windows 系统中文字体文件
  - [ ] 字体加载失败时降级为默认字体，并在 GUI 状态区提示（不影响运行）

- [ ] [004] Task 4: 验证与回归（Issue #35）
  - [ ] 确认 Windows CI 的 GUI 编译步骤仍通过（`--features gui --bin disk-scout-gui`）
  - [ ] 更新/补充最小人工验证说明（GUI 滚动、min-size 单位、中文显示）

# Task Dependencies
- Task 4 depends on Task 1 and Task 2 and Task 3
