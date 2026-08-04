# disk-scout

只读磁盘扫描器：扫描指定目录，按大小输出 Top N 的文件与目录，并给出统计信息。适合快速定位“磁盘空间被谁占了”。

## 特性

- 只读扫描：不修改文件系统
- 输出 Top 文件/目录：分别输出最大文件与最大目录列表
- 过滤能力：支持 `--min-size` 与 ignore 规则（命令行与 ignore 文件）
- 双输出格式：默认人类可读的 `text`，以及便于脚本处理的 `json`

## 安装

### 方式 A：下载 Release

从 GitHub Releases 下载对应平台的压缩包，解压后即可运行：

- Windows：解压后得到
  - `disk-scout.exe`（CLI）
  - `disk-scout-gui.exe`（GUI）
- Linux：解压后得到 `disk-scout`

### 方式 B：从源码构建

需要 Rust 工具链：

```bash
cargo build --release
```

产物位于：

- Linux/macOS：`target/release/disk-scout`
- Windows：`target/release/disk-scout.exe`

可选构建 GUI（仅 Windows）：

```bash
cargo build --release --features gui --bin disk-scout-gui
```

产物位于：

- Windows：`target/release/disk-scout-gui.exe`

## 快速开始

Windows（PowerShell，推荐用终端运行而非双击）：

```powershell
.\disk-scout.exe --help
.\disk-scout.exe scan "C:\Users\%USERNAME%\Downloads"
```

Windows GUI：

```powershell
.\disk-scout-gui.exe
```

GUI 交互要点：

- Top files 与 Top dirs 各自独立滚动（不需要拉长窗口）
- Min size 使用“数值 + 单位（B/K/M/G）”选择，等价于字节阈值过滤
- 顶部会显示字体状态；若中文路径显示方块，可优先确认是否已加载 Windows 中文字体

Linux/macOS：

```bash
./disk-scout --help
./disk-scout scan ~/Downloads
```

自定义 Top N 与最小大小阈值：

```bash
disk-scout scan . --top-files 50 --top-dirs 50 --min-size 10MiB
```

输出 JSON（重定向到文件）：

```bash
disk-scout scan . --format json > report.json
```

## 命令与参数

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

## SIZE 格式（--min-size）

`--min-size` 支持以下写法：

- 纯数字：`4096`（字节）
- 数字+单位（可紧贴或用空格）：`1KiB`、`1 MiB`、`2 mb`

支持单位（大小写不敏感）：

- 二进制单位（基数 1024）：`KiB` `MiB` `GiB` `TiB`
- 十进制单位（基数 1000）：`KB` `MB` `GB` `TB`（也支持 `K/M/G/T`）
- `B` 或空单位表示字节

## Ignore 规则

### 命令行 ignore

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

## 输出

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

## 退出码

- `0`：扫描成功
- `2`：扫描失败（例如无法访问根目录）

## 常见问题

### Windows 双击“闪退”

这是命令行程序，双击运行窗口会很快关闭，看起来像闪退。推荐在 PowerShell/CMD 中运行，例如：

```powershell
.\disk-scout.exe --help
.\disk-scout.exe scan "C:\"
```

如果你希望使用图形界面，请运行 `disk-scout-gui.exe`。

### GUI 删除文件是否安全

GUI 的删除功能默认会尝试将文件移入回收站，并在执行前弹出二次确认。同时会拒绝删除扫描根路径之外的文件与非常规文件类型。

### Windows 提示缺少 DLL（如 VCRUNTIME140_1.dll）

你下载的是 `windows-msvc` 产物时，可能需要安装 Microsoft Visual C++ Redistributable（2015–2022，x64）。安装后重新运行即可。

## 开发

运行测试：

```bash
cargo test
```
