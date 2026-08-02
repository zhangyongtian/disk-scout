# disk-scout 只读磁盘占用扫描器 Spec

## Why
当磁盘/目录快满时，用户往往不清楚哪些文件与目录占用最多空间。该工具提供只读扫描与可导出的报告，帮助快速定位空间占用热点，并可将结果贴到 GitHub Issue/Wiki 便于协作排查。

## What Changes
- 新增 Rust CLI 工具 disk-scout：对指定目录/挂载点进行只读扫描
- 输出 Top N 最大文件与 Top N 最大目录（按总大小排序）
- 支持生成两类报告：人类可读文本与 JSON
- 默认不跟随符号链接；遇到权限/IO 错误不中断扫描，仅记录统计
- 提供基础过滤能力：最小大小阈值与忽略规则（pattern/ignore-file）

## Impact
- Affected specs: CLI 参数设计、扫描策略、报告格式、错误处理策略
- Affected code: Rust crate（Cargo.toml、src/**）、GitHub Actions（后续）、打包脚本（后续）

## ADDED Requirements

### Requirement: 扫描命令
系统 SHALL 提供 `scan` 子命令，对用户指定的 `path` 进行只读扫描并输出结果。

#### Scenario: 成功扫描并输出文本报告
- **WHEN** 用户执行 `disk-scout scan <path>`
- **THEN** 工具 SHALL 在标准输出打印文本报告
- **AND** 报告 SHALL 包含 Top N 最大文件与 Top N 最大目录
- **AND** 报告中的大小 SHALL 以人类可读格式展示（如 KiB/MiB/GiB）

#### Scenario: 输出 JSON 报告
- **WHEN** 用户执行 `disk-scout scan <path> --format json`
- **THEN** 工具 SHALL 输出结构化 JSON
- **AND** JSON SHALL 至少包含：扫描根路径、扫描统计信息、top_files、top_dirs

#### Scenario: 路径不可访问
- **WHEN** 扫描过程中遇到权限不足/路径不可读/IO 错误
- **THEN** 工具 SHALL 继续扫描其他路径
- **AND** 工具 SHALL 在最终报告中给出错误数量统计（可选附带部分样例）

### Requirement: Top N 排行与阈值
系统 SHALL 支持用户配置 Top N 与最小大小阈值，以减少噪声并控制输出规模。

#### Scenario: 自定义 Top N
- **WHEN** 用户执行 `disk-scout scan <path> --top-files 100 --top-dirs 50`
- **THEN** 工具 SHALL 分别输出最多 100 个文件与 50 个目录条目

#### Scenario: 最小大小阈值
- **WHEN** 用户执行 `disk-scout scan <path> --min-size 10MB`
- **THEN** 工具 SHALL 忽略小于阈值的文件条目（目录大小仍基于其包含文件累计）

### Requirement: 忽略规则
系统 SHALL 支持忽略规则以跳过无关路径。

#### Scenario: 命令行忽略 pattern
- **WHEN** 用户执行 `disk-scout scan <path> --ignore '**/.git/**' --ignore '**/node_modules/**'`
- **THEN** 工具 SHALL 在扫描时跳过匹配的路径

#### Scenario: ignore file
- **WHEN** 用户执行 `disk-scout scan <path> --ignore-file <file>`
- **THEN** 工具 SHALL 从文件中加载忽略规则并应用

### Requirement: 符号链接处理
系统 SHALL 默认不跟随符号链接，以避免循环与重复统计。

#### Scenario: 默认不跟随符号链接
- **WHEN** 目录树中存在符号链接
- **THEN** 工具 SHALL 不进入该链接目标路径进行递归扫描

## MODIFIED Requirements
无。

## REMOVED Requirements
无。

## Non-Goals（明确不做）
- 不提供自动删除/清理建议/生成清理脚本能力
- 不提供交互式 TUI 界面
- 不强制对齐系统 `du` 的每一种细节行为（例如稀疏文件、硬链接重复计数等）；如涉及差异，优先保证一致性与可解释性

