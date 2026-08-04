# disk-scout Windows GUI 交互与中文显示优化 Spec

## Why
当前 GUI 在结果列表较长时只能通过拉长窗口查看，缺少每个列表独立的滚动条；`min-size` 仅支持字符串输入，缺少直观的单位选择；同时 Windows 下中文路径/文本显示存在乱码或方块，影响可用性。

## What Changes
- Windows GUI：为 Top files / Top dirs 各自提供独立的纵向滚动区域（独立滚动条）
- Windows GUI：`min-size` 输入改为“数值 + 单位下拉选择”（K/M/G 等），并换算为 bytes 传入扫描 core
- Windows GUI：修复中文显示问题（优先加载 Windows 系统中文字体，失败则降级）

## Impact
- Affected specs: GUI 交互与可用性、参数输入体验、Windows 字体与国际化支持
- Affected code: `disk-scout/src/bin/disk-scout-gui.rs`、（可选）新增字体加载/单位换算小模块、CI 无需新增步骤（已有 windows gui build 校验）

## ADDED Requirements

### Requirement: Top 列表独立滚动
系统 SHALL 在 Windows GUI 中为 Top files 与 Top dirs 列表分别提供独立的纵向滚动条，避免必须拉伸窗口才能浏览全部结果。

#### Scenario: Top files 独立滚动
- **WHEN** Top files 列表内容超过可视区域高度
- **THEN** Top files 区域 SHALL 显示独立滚动条
- **AND** 滚动 Top files 不影响 Top dirs 区域的滚动位置

#### Scenario: Top dirs 独立滚动
- **WHEN** Top dirs 列表内容超过可视区域高度
- **THEN** Top dirs 区域 SHALL 显示独立滚动条
- **AND** 滚动 Top dirs 不影响 Top files 区域的滚动位置

### Requirement: min-size 单位选择
系统 SHALL 在 Windows GUI 中提供 `min-size` 的单位选择控件，使用户无需手写 `KiB/MiB/GiB` 字符串即可设置阈值。

#### Scenario: 选择单位并换算
- **WHEN** 用户输入 `min-size` 数值并选择单位（B/K/M/G 或 KiB/MiB/GiB）
- **THEN** GUI SHALL 将其换算为 bytes，并作为扫描计划的 `min_size` 参数传入 core

#### Scenario: 输入校验
- **WHEN** 用户输入非法数值（空、负数、非数字）
- **THEN** GUI SHALL 阻止开始扫描并给出可理解的提示

### Requirement: 中文显示
系统 SHALL 在 Windows GUI 中正确显示中文（UI 文本与扫描结果路径），避免出现方块/乱码。

#### Scenario: 优先加载系统字体
- **WHEN** 运行在 Windows 平台
- **THEN** GUI SHALL 尝试加载系统中文字体（例如微软雅黑/黑体等常见字体文件）
- **AND** 若加载失败 SHALL 继续可运行并使用默认字体（但需给出可理解的状态提示或日志提示）

## MODIFIED Requirements
无。

## REMOVED Requirements
无。
