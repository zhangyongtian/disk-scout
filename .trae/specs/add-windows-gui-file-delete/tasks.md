# Tasks
- [x] [001] Task 1: 抽取可复用的扫描核心（core）（Issue #22）
  - [x] 将扫描/过滤/统计/输出所需的核心数据结构抽为库接口，供 CLI 与 GUI 复用
  - [x] 保持现有 CLI 行为与输出不变（回归对齐已有测试/脚本）

- [x] [002] Task 2: 新增 Windows GUI 程序入口（Issue #23）
  - [x] 新增 `disk-scout-gui`（仅 Windows 构建），提供基础窗口与状态区
  - [x] 提供目录选择与参数输入控件：root path、top-files、top-dirs、min-size、ignore、ignore-file

- [x] [003] Task 3: GUI 扫描执行与结果展示（Issue #24）
  - [x] 点击“开始扫描”触发后台扫描（避免阻塞 UI），扫描中展示进度/忙碌状态
  - [x] 扫描完成后展示 Top 文件与 Top 目录列表（路径 + 人类可读大小）
  - [x] 提供复制路径能力（至少支持复制 Top 文件路径）

- [x] [004] Task 4: GUI 文件删除（安全优先）（Issue #25）
  - [x] 为 Top 文件条目提供“删除”按钮与二次确认对话框
  - [x] 实现删除安全边界：仅允许删除扫描根路径下的常规文件
  - [x] 默认优先移入回收站（不可用时给出明确提示或拒绝执行）
  - [x] 删除后更新 UI（移除/标记）并展示结果提示

- [x] [005] Task 5: 发布产物与文档对齐（Issue #26）
  - [x] Windows release 包内同时包含 CLI 与 GUI（命名清晰，避免双击“闪退”困惑）
  - [x] README 增加 Windows GUI 使用说明与安全提示（回收站/永久删除差异、权限/占用常见问题）

# Task Dependencies
- Task 2 depends on Task 1
- Task 3 depends on Task 1 and Task 2
- Task 4 depends on Task 3
- Task 5 depends on Task 2
