# Claude Code Rust - PowerShell 配置脚本
# 这个脚本配置 PowerShell 别名，用于区分 Rust 版本和 TypeScript 版本的 Claude Code

# 颜色定义
$Green = "`e[32m"
$Yellow = "`e[33m"
$Blue = "`e[34m"
$Reset = "`e[0m"

Write-Host "${Blue}=============================================${Reset}"
Write-Host "${Blue}  Claude Code Rust - PowerShell 配置工具${Reset}"
Write-Host "${Blue}=============================================${Reset}"
Write-Host ""

# 检查 PowerShell 配置文件
if (!(Test-Path -Path $PROFILE)) {
    Write-Host "${Yellow}正在创建 PowerShell 配置文件...${Reset}"
    New-Item -ItemType File -Path $PROFILE -Force | Out-Null
    Write-Host "${Green}✓ 配置文件已创建: $PROFILE${Reset}"
} else {
    Write-Host "${Green}✓ PowerShell 配置文件已存在: $PROFILE${Reset}"
}

Write-Host ""

# 定义路径
$RustClaudePath = "C:\迅雷下载\claude-code-rev-main\claude-code-rust\bin\claude.exe"
$NpmClaudePath = "C:\Users\user\AppData\Roaming\npm\claude"

# 检查 Rust 版本是否存在
if (Test-Path $RustClaudePath) {
    Write-Host "${Green}✓ 找到 Rust 版本: $RustClaudePath${Reset}"
} else {
    Write-Host "${Yellow}⚠ 未找到 Rust 版本，请确认路径: $RustClaudePath${Reset}"
    $RustClaudePath = Read-Host "请输入 Rust 版本的完整路径"
}

# 检查 TypeScript 版本是否存在
if (Test-Path $NpmClaudePath) {
    Write-Host "${Green}✓ 找到 TypeScript 版本: $NpmClaudePath${Reset}"
} else {
    Write-Host "${Yellow}⚠ 未找到 TypeScript 版本 (npm)${Reset}"
    $NpmClaudePath = Read-Host "请输入 npm claude 的完整路径 (或按 Enter 跳过)"
}

Write-Host ""
Write-Host "${Blue}正在配置别名...${Reset}"

# 读取现有配置文件内容
$profileContent = Get-Content -Path $PROFILE -Raw -ErrorAction SilentlyContinue

# 检查是否已存在配置
$existingConfig = $profileContent -match "# Claude Code Rust Configuration"

if ($existingConfig) {
    Write-Host "${Yellow}⚠ 检测到已有配置，是否覆盖? (Y/N)${Reset}"
    $response = Read-Host
    if ($response -ne 'Y' -and $response -ne 'y') {
        Write-Host "${Yellow}已取消配置${Reset}"
        exit 0
    }
    # 移除旧配置
    $profileContent = $profileContent -replace "(?s)# Claude Code Rust Configuration.*?# End Claude Code Rust Configuration", ""
}

# 创建新的配置块
$newConfig = @"

# Claude Code Rust Configuration
# 配置日期: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")

# Claude Code Rust 版本 (高性能实现)
function claude-rust {
    & "$RustClaudePath" @args
}
Set-Alias -Name claude-rust -Value claude-rust

# Claude Code TypeScript 版本 (原版)
function claude-npm {
    & "$NpmClaudePath" @args
}
Set-Alias -Name claude-npm -Value claude-npm

# 环境变量
`$env:CLAUDE_RUST_PATH = "$RustClaudePath"
`$env:CLAUDE_NPM_PATH = "$NpmClaudePath"

# 快速切换函数
function Switch-ClaudeVersion {
    param(
        [Parameter(Mandatory=`$true)]
        [ValidateSet("rust", "npm")]
        [string]`$Version
    )

    switch (`$Version) {
        "rust" {
            Write-Host "正在启动 Claude Code Rust (高性能版本)..." -ForegroundColor Green
            & `$env:CLAUDE_RUST_PATH
        }
        "npm" {
            Write-Host "正在启动 Claude Code (TypeScript 原版)..." -ForegroundColor Blue
            & `$env:CLAUDE_NPM_PATH
        }
    }
}
Set-Alias -Name claude-switch -Value Switch-ClaudeVersion

# End Claude Code Rust Configuration
"@

# 追加到配置文件
Add-Content -Path $PROFILE -Value $newConfig

Write-Host "${Green}✓ 配置已添加到 PowerShell 配置文件${Reset}"
Write-Host ""
Write-Host "${Blue}=============================================${Reset}"
Write-Host "${Blue}  配置完成！${Reset}"
Write-Host "${Blue}=============================================${Reset}"
Write-Host ""
Write-Host "可用的命令:"
Write-Host "  ${Green}claude-rust${Reset}     - 启动 Rust 版本 (高性能)"
Write-Host "  ${Green}claude-npm${Reset}      - 启动 TypeScript 版本 (原版)"
Write-Host "  ${Green}claude-switch rust${Reset}  - 切换到 Rust 版本"
Write-Host "  ${Green}claude-switch npm${Reset}   - 切换到 TypeScript 版本"
Write-Host ""
Write-Host "${Yellow}请重新打开 PowerShell 或运行以下命令使配置生效:${Reset}"
Write-Host "  . `$PROFILE"
Write-Host ""
Write-Host "${Blue}验证安装:${Reset}"
Write-Host "  claude-rust --version"
Write-Host "  claude-npm --version"
Write-Host ""
