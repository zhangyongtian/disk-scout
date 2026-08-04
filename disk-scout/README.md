# disk-scout

disk-scout 是一个专注“可靠、可验证”的磁盘空间定位工具：对指定目录进行只读扫描，输出占用空间最大的文件与目录列表，并提供 Windows GUI 便捷操作与安全删除能力。

- Windows：CLI + GUI（可选构建 GUI）
- Linux/macOS：CLI

## 目录

- [特性](#特性)
- [安装](#安装)
- [快速开始](#快速开始)
- [Windows GUI 使用说明](#windows-gui-使用说明)
- [CLI 命令与参数](#cli-命令与参数)
- [SIZE 语法（--min-size）](#size-语法---min-size)
- [Ignore 规则](#ignore-规则)
- [输出格式](#输出格式)
- [安全模型（GUI 删除）](#安全模型gui-删除)
- [构建与开发](#构建与开发)
- [CI / Release](#ci--release)
- [常见问题](#常见问题)

## 特性

- 只读扫描：默认不修改文件系统（GUI 删除为显式操作，且有安全边界）
- Top 列表：分别输出 Top files 与 Top dirs（按大小降序）
- 过滤能力：`--min-size` 阈值过滤 + ignore 规则（命令行 + ignore 文件）
- 双输出格式：`text`（可读）与 `json`（便于脚本处理）
- Windows GUI：目录选择、参数配置、独立滚动列表、复制路径、安全删除（回收站 + 二次确认）

## 安装

### 方式 A：下载 Release（推荐）

从 GitHub Releases 下载对应平台的压缩包，解压后即可运行：

- Windows：
  - `disk-scout.exe`（CLI）
  - `disk-scout-gui.exe`（GUI）
- Linux/macOS：
  - `disk-scout`（CLI）

### 方式 B：从源码构建

需要 Rust 工具链：

```bash
cd disk-scout
cargo build --release
```

产物位于：

- Linux/macOS：`disk-scout/target/release/disk-scout`
- Windows：`disk-scout/target/release/disk-scout.exe`

可选构建 GUI（仅 Windows，且需要启用 `gui` feature）：

```bash
cd disk-scout
cargo build --release --locked --features gui --bin disk-scout-gui
```

产物位于：

- Windows：`disk-scout/target/release/disk-scout-gui.exe`

## 快速开始

### Windows CLI（PowerShell）

推荐在 PowerShell/CMD 中运行（而非双击）：

```powershell
.\disk-scout.exe --help
.\disk-scout.exe scan "C:\Users\%USERNAME%\Downloads"
```

### Windows GUI（PowerShell）

```powershell
.\disk-scout-gui.exe
```

### Linux/macOS（bash）

```bash
./disk-scout --help
./disk-scout scan ~/Downloads
```

### 常用用法（CLI）

自定义 Top N 与最小大小阈值：

```bash
disk-scout scan . --top-files 50 --top-dirs 50 --min-size 10MiB
```

输出 JSON（重定向到文件）：

```bash
disk-scout scan . --format json > report.json
```

## Windows GUI 使用说明

GUI 目标是把“定位 + 处置”变成一个低摩擦的闭环：

- Root path：选择扫描根目录
- Top files / Top dirs：控制列表长度
- Min size：用“数值 + 单位（B/K/M/G）”输入阈值；内部换算为 bytes（与 CLI 的 `--min-size` 语义一致）
- Ignore patterns：逐行输入 ignore（每行一个 pattern）
- Ignore file：选择 ignore 文件（与 CLI 的 `--ignore-file` 行为一致）
- Start scan：后台线程执行扫描，不阻塞 UI

结果区：

- Top files：每条目提供 Copy path 与 Delete
- Top dirs：独立列表展示（与 Top files 分离滚动）
- 独立滚动：Top files 与 Top dirs 各自拥有滚动区域，避免通过拉伸窗口才能浏览
- 字体状态：窗口顶部会显示字体加载状态；若中文路径显示方块，优先确认状态为“已加载 Windows 中文字体”

## CLI 命令与参数

### scan

基本用法：

```bash
disk-scout scan <path> [options]
```

参数：

- `--top-files <N>`：输出最大的文件 Top N（默认 20）
- `--top-dirs <N>`：输出最大的目录 Top N（默认 20）
- `--min-size <SIZE>`：过滤小于阈值的条目（默认 0）
- `--format <FORMAT>`：输出格式，`text`（默认）或 `json`
- `--ignore <PATTERN>`：忽略规则（可重复传入多个）
- `--ignore-file <PATH>`：从文件读取忽略规则

## SIZE 语法（--min-size）

`--min-size` 支持以下写法：

- 纯数字：`4096`（字节）
- 数字 + 单位（可紧贴或用空格）：`1KiB`、`1 MiB`、`2 mb`

单位（大小写不敏感）：

- 二进制单位（基数 1024）：`KiB` `MiB` `GiB` `TiB`
- 十进制单位（基数 1000）：`KB` `MB` `GB` `TB`（也支持 `K/M/G/T`）
- `B` 或空单位表示字节

## Ignore 规则

### 命令行 ignore（--ignore）

支持 `*` 与 `?` 的简单 glob 匹配；可以多次传入：

```bash
disk-scout scan . --ignore "target/*" --ignore "*.log" --ignore "*.tmp"
```

### ignore 文件（--ignore-file）

文件按行解析：

- 空行会忽略
- 以 `#` 开头的行视为注释，会忽略

示例 `.disk-scout-ignore`：

```text
# build outputs
target/*
*.log
*.tmp
```

运行：

```bash
disk-scout scan . --ignore-file .disk-scout-ignore
```

## 输出格式

### text（默认）

输出包含统计信息与两个列表：`top_files`、`top_dirs`。

示例：

```text
root: /tmp/root
bytes_total: 2.00 KiB (2048)
files_seen: 3
dirs_seen: 2
errors_total: 1
duration_ms: 12

top_files:
  2.00 KiB (2048) /tmp/root/a.bin

top_dirs:
  2.00 KiB (2048) /tmp/root
```

### json

JSON 输出是“可读格式”的单个 JSON 对象（不是 jsonl/ndjson），结构大致如下：

```json
{
  "meta": {
    "scan_root": "/tmp/root",
    "stats": {
      "bytes_total": 2048,
      "files_seen": 3,
      "dirs_seen": 2,
      "errors_total": 1,
      "duration_ms": 12
    }
  },
  "top_files": [
    { "size_bytes": 2048, "path": "/tmp/root/a.bin" }
  ],
  "top_dirs": [
    { "size_bytes": 2048, "path": "/tmp/root" }
  ],
  "errors": {
    "total": 1,
    "samples": [
      { "path": "/tmp/root/x", "message": "denied" }
    ]
  }
}
```

## 安全模型（GUI 删除）

GUI 删除能力以“安全优先”为目标：

- 二次确认：点击 Delete 后会弹出确认对话框
- 扫描根边界：拒绝删除扫描根目录之外的路径
- 文件类型限制：拒绝删除非常规文件类型（只允许常规文件）
- 默认回收站：优先将文件移入回收站（而非永久删除）；失败会给出明确错误提示

## 构建与开发

运行测试：

```bash
cd disk-scout
cargo test --locked
```

本项目采用 `lib + bin` 结构：

- `disk-scout`：CLI 入口
- `disk-scout-gui`：Windows GUI 入口（`--features gui`，且仅在 Windows 上可用）

## CI / Release

- CI：在 Windows 上会额外验证 GUI 的编译（`--features gui --bin disk-scout-gui`）
- Release：基于 tag 构建（必须先推送 tag，例如 `v0.3.0`），产物包含：
  - Linux：`disk-scout-<tag>-x86_64-unknown-linux-gnu.tar.gz`
  - Windows：`disk-scout-<tag>-x86_64-pc-windows-msvc.zip`（同时包含 CLI 与 GUI）

## 常见问题

### Windows 双击“闪退”

CLI 程序双击运行会立即退出，看起来像闪退。请在 PowerShell/CMD 中运行：

```powershell
.\disk-scout.exe --help
.\disk-scout.exe scan "C:\"
```

如需图形界面，请运行 `disk-scout-gui.exe`。

### Windows 中文路径显示方块

GUI 启动时会尝试加载 Windows 系统中文字体；窗口顶部会显示字体状态。若仍显示方块，通常是系统字体文件不可用或被裁剪，可尝试安装/启用常见中文字体后重试。

### Windows 提示缺少 DLL（如 VCRUNTIME140_1.dll）

若下载的是 `windows-msvc` 产物，可能需要安装 Microsoft Visual C++ Redistributable（2015–2022，x64）。安装后重新运行即可。
