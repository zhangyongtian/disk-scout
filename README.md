# disk-scout

只读磁盘扫描器：扫描指定目录，按大小输出 Top N 的文件与目录，并给出统计信息。

本仓库采用“仓库根目录 + 子目录工程”的结构，Rust 项目位于 `disk-scout/` 目录下。

## 快速开始

Windows（PowerShell）：

```powershell
cd disk-scout
.\disk-scout.exe --help
.\disk-scout.exe scan "C:\Users\%USERNAME%\Downloads"
```

Linux/macOS：

```bash
cd disk-scout
./disk-scout --help
./disk-scout scan ~/Downloads
```

## 使用文档

完整参数说明、ignore/min-size 规则与输出格式示例见：

- [disk-scout/README.md](file:///home/roott/github_learn/disk-scout/README.md)
