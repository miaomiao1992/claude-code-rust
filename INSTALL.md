# Claude Code Rust 安装指南

本文档提供详细的安装和配置说明，帮助您在 Windows 系统上安装和配置 Claude Code Rust 版本。

## 📋 前置要求

- Windows 10/11 操作系统
- PowerShell 5.1 或 PowerShell 7+
- (可选) Rust 工具链（如果需要从源码编译）

## 🚀 快速安装（推荐）

### 方式一：一键配置 PowerShell 别名

1. **运行 PowerShell 配置脚本**
   ```powershell
   # 在项目目录中运行
   Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
   .\scripts\setup-powershell.ps1
   ```

2. **重新加载 PowerShell 配置**
   ```powershell
   . $PROFILE
   ```

3. **验证安装**
   ```powershell
   claude-rust --version
   claude-npm --version
   ```

### 方式二：手动配置 PowerShell

1. **编辑 PowerShell 配置文件**
   ```powershell
   notepad $PROFILE
   ```

2. **添加以下内容到配置文件末尾**
   ```powershell
   # Claude Code Rust Configuration
   function claude-rust {
       & "C:\迅雷下载\claude-code-rev-main\claude-code-rust\bin\claude.exe" @args
   }
   
   function claude-npm {
       & "C:\Users\user\AppData\Roaming\npm\claude" @args
   }
   
   $env:CLAUDE_RUST_PATH = "C:\迅雷下载\claude-code-rev-main\claude-code-rust\bin\claude.exe"
   $env:CLAUDE_NPM_PATH = "C:\Users\user\AppData\Roaming\npm\claude"
   ```

3. **保存并重新加载配置**
   ```powershell
   . $PROFILE
   ```

## 🔧 命令行使用方法

### 启动 Rust 版本（高性能）

```powershell
# 方式一：使用配置的别名（推荐）
claude-rust

# 方式二：使用完整路径
& "C:\迅雷下载\claude-code-rev-main\claude-code-rust\bin\claude.exe"

# 方式三：如果在项目目录
.\bin\claude.exe
```

### 启动 TypeScript 版本（原版）

```powershell
# 方式一：使用配置的别名（推荐）
claude-npm

# 方式二：使用完整路径
& "C:\Users\user\AppData\Roaming\npm\claude"

# 方式三：直接使用（如果 PATH 配置正确）
claude
```

### 常用命令对比

| 操作 | Rust 版本 | TypeScript 版本 |
|------|-----------|-----------------|
| 查看版本 | `claude-rust --version` | `claude-npm --version` |
| 进入交互模式 | `claude-rust` | `claude-npm` |
| 执行单次查询 | `claude-rust query "问题"` | `claude-npm "问题"` |
| 查看帮助 | `claude-rust --help` | `claude-npm --help` |
| 配置管理 | `claude-rust config` | `claude-npm config` |

### 版本验证

```powershell
# Rust 版本（显示版本号和 "High-performance implementation" 标识）
claude-rust --version
# 输出: 0.1.1 (Claude Code Rust - High-performance implementation)

# TypeScript 版本（显示原版标识）
claude-npm --version
# 输出: 2.1.92 (Claude Code)
```

## 🛠️ 从源码编译（可选）

如果您需要从源码编译 Rust 版本：

### 1. 安装 Rust 工具链

```powershell
# 使用 rustup 安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 或在 Windows 上使用 winget
winget install Rustlang.Rustup
```

### 2. 克隆仓库

```powershell
git clone https://github.com/lorryjovens-hub/claude-code-rust.git
cd claude-code-rust
```

### 3. 编译项目

```powershell
# 开发模式编译
cargo build

# 发布模式编译（推荐）
cargo build --release
```

### 4. 复制二进制文件

```powershell
# 创建 bin 目录
New-Item -ItemType Directory -Force -Path .\bin

# 复制编译后的二进制文件
Copy-Item .\target\release\claude.exe .\bin\claude.exe
```

## 📝 PowerShell 高级配置

### 自定义快捷函数

在 PowerShell 配置文件中添加以下函数：

```powershell
# 在项目目录快速启动
function cc-here {
    $currentPath = Get-Location
    Set-Location "C:\迅雷下载\claude-code-rev-main\claude-code-rust"
    .\bin\claude.exe @args
    Set-Location $currentPath
}

# 在任何目录启动 Rust 版本
function ccr {
    & $env:CLAUDE_RUST_PATH @args
}

# 在任何目录启动 npm 版本
function ccnpm {
    & $env:CLAUDE_NPM_PATH @args
}

# 版本对比函数
function Compare-ClaudeVersions {
    Write-Host "=== Claude Code 版本对比 ===" -ForegroundColor Cyan
    Write-Host ""
    
    Write-Host "Rust 版本:" -ForegroundColor Green
    claude-rust --version
    
    Write-Host ""
    Write-Host "TypeScript 版本:" -ForegroundColor Blue
    claude-npm --version
}
Set-Alias -Name cc-compare -Value Compare-ClaudeVersions
```

### 环境变量配置

```powershell
# 编辑配置文件
notepad $PROFILE

# 添加环境变量
$env:CLAUDE_RUST_PATH = "C:\迅雷下载\claude-code-rev-main\claude-code-rust\bin\claude.exe"
$env:CLAUDE_NPM_PATH = "C:\Users\user\AppData\Roaming\npm\claude"

# 可选：设置默认编辑器
$env:EDITOR = "code"
```

## 🔍 故障排除

### 问题 1：找不到 claude-rust 命令

**解决方案**：
```powershell
# 检查配置文件是否正确加载
Test-Path $PROFILE

# 重新加载配置文件
. $PROFILE

# 检查函数是否定义
Get-Command claude-rust
Get-Command claude-npm
```

### 问题 2：执行策略限制

**解决方案**：
```powershell
# 以管理员身份运行 PowerShell
# 然后执行
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# 验证更改
Get-ExecutionPolicy -List
```

### 问题 3：Windows 上构建失败（libz-sys 错误）

**解决方案**：
```powershell
# 方案一：安装 Visual Studio 构建工具
# 方案二：使用静态链接
$env:RUSTFLAGS = "-C target-feature=+crt-static"
cargo build --release

# 方案三：直接下载预编译版本（如果可用）
```

### 问题 4：路径中包含中文导致问题

**解决方案**：
```powershell
# 将项目移动到英文路径
cd C:\Projects
# 然后重新配置路径
```

## ✅ 验证安装

运行以下命令验证安装是否成功：

```powershell
# 检查 Rust 版本
claude-rust --version
# 预期输出: 0.1.1 (Claude Code Rust - High-performance implementation)

# 检查 TypeScript 版本
claude-npm --version
# 预期输出: 2.1.92 (Claude Code)

# 测试交互模式
claude-rust help
```

## 🔄 卸载

### 移除 PowerShell 配置

```powershell
# 编辑 PowerShell 配置文件
notepad $PROFILE

# 删除 "# Claude Code Rust Configuration" 注释块之间的所有内容

# 重新加载配置
. $PROFILE
```

### 删除项目文件

```powershell
# 删除项目目录
Remove-Item -Path "C:\迅雷下载\claude-code-rev-main\claude-code-rust" -Recurse -Force

# 删除配置文件（如果需要）
Remove-Item -Path "$env:USERPROFILE\.config\claude-code-rust" -Recurse -Force
```

## 📚 相关文档

- [README.md](README.md) - 项目介绍和主要功能
- [PLUGIN_DEVELOPMENT.md](PLUGIN_DEVELOPMENT.md) - 插件开发指南
- [ARCHITECTURE.md](ARCHITECTURE.md) - 系统架构文档
- [CONTRIBUTING.md](CONTRIBUTING.md) - 贡献指南

## 🤝 获取帮助

- **GitHub Issues**: https://github.com/lorryjovens-hub/claude-code-rust/issues
- **文档反馈**: https://my.feishu.cn/wiki/GfQGwIen9izVnikrchFcKOtOnTb

---

**提示**: 使用 `claude-rust` 命令启动高性能 Rust 版本，使用 `claude-npm` 命令启动原版 TypeScript 版本。两个版本在界面和功能上有所区别，Rust 版本会在欢迎界面显示 "🟢 Claude Code Rust - 重构高性能版本"。
