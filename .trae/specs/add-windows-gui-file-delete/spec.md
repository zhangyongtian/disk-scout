# disk-scout Windows 图形界面与文件删除 Spec

## Why
Windows 用户更习惯通过图形界面选择扫描参数并浏览结果；在定位到占用空间最大的文件后，希望可以在同一界面完成安全删除，以降低清理成本。

## What Changes
- 新增仅面向 Windows 的可选 GUI 程序 `disk-scout-gui`：提供参数选择、启动扫描、展示 Top 文件/目录列表
- GUI 支持对“Top 文件”列表条目执行删除操作（带二次确认），用于快速清理空间
- CLI（Linux/Windows 均可用）保持现有行为与参数，仍为只读扫描，不提供删除能力
- **BREAKING**：项目对外定位从“只读扫描器”扩展为“可选清理能力（仅 GUI）”，需在文档中明确安全边界与默认行为

## Impact
- Affected specs: 扫描参数呈现、结果展示交互、删除安全策略、Windows 发布产物形态
- Affected code: `disk-scout/` crate 结构（抽取可复用 core）、新增 GUI bin、Release 工作流（Windows 产物追加 GUI exe）、README 使用说明

## ADDED Requirements

### Requirement: Windows GUI 启动与参数选择
系统 SHALL 在 Windows 平台提供 `disk-scout-gui` 图形界面程序，支持用户选择扫描参数并启动扫描。

#### Scenario: 选择扫描根路径并启动扫描
- **WHEN** 用户在 GUI 中选择一个目录作为扫描根路径，并点击“开始扫描”
- **THEN** GUI SHALL 按所选参数启动一次扫描
- **AND** 扫描完成后 SHALL 展示 Top 文件与 Top 目录列表

#### Scenario: 参数覆盖
- **WHEN** 用户在 GUI 中设置 top-files / top-dirs / min-size / ignore / ignore-file 任意参数
- **THEN** GUI SHALL 使用这些参数发起扫描，并与 CLI 参数语义保持一致

### Requirement: 扫描结果可视化
系统 SHALL 在 GUI 中以可浏览形式展示扫描结果，并可复制路径信息。

#### Scenario: 展示 Top 文件列表
- **WHEN** 扫描完成
- **THEN** GUI SHALL 以表格/列表形式展示 Top 文件条目
- **AND** 每条条目至少包含：路径、大小（人类可读）、（可选）占比/排序序号

#### Scenario: 展示 Top 目录列表
- **WHEN** 扫描完成
- **THEN** GUI SHALL 展示 Top 目录条目
- **AND** 每条条目至少包含：路径、大小（人类可读）、（可选）占比/排序序号

### Requirement: 文件删除（GUI）
系统 SHALL 允许用户在 GUI 中对 Top 文件条目执行删除操作，并提供安全确认与错误反馈。

#### Scenario: 删除确认
- **WHEN** 用户点击某条 Top 文件的“删除”按钮
- **THEN** GUI SHALL 弹出二次确认对话框，明确展示将被删除的文件路径
- **AND** 用户取消时 SHALL 不做任何文件系统变更

#### Scenario: 删除成功
- **WHEN** 用户确认删除，且文件删除成功
- **THEN** GUI SHALL 将该条目从列表中移除或标记为“已删除”
- **AND** GUI SHALL 在状态区提示删除成功

#### Scenario: 删除失败
- **WHEN** 用户确认删除，但删除失败（权限不足、文件被占用、路径不存在等）
- **THEN** GUI SHALL 保持条目仍可见（或标记失败）
- **AND** GUI SHALL 给出可理解的错误提示（不泄露敏感信息）

#### Scenario: 删除安全边界
- **WHEN** 用户尝试删除不在本次扫描根路径下的文件（例如外部路径或路径穿越）
- **THEN** GUI SHALL 拒绝删除并提示原因

#### Scenario: 删除默认策略（安全优先）
- **WHEN** 运行在 Windows 平台且系统回收站能力可用
- **THEN** GUI SHOULD 优先将文件移入回收站（而非直接永久删除）

## MODIFIED Requirements

### Requirement: 只读扫描承诺
系统 SHALL 保持 `disk-scout` CLI 的只读行为，不因新增 GUI 而引入默认的删除/修改能力。

#### Scenario: CLI 保持只读
- **WHEN** 用户使用 CLI `disk-scout scan <path>`
- **THEN** 工具 SHALL 不执行任何删除/修改操作

## REMOVED Requirements
无。
