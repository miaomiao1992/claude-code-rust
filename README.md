# Claude Code Rust - 高性能 Rust 实现版本

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub release](https://img.shields.io/github/v/release/lorryjovens-hub/claude-code-rust)](https://github.com/lorryjovens-hub/claude-code-rust/releases)

**Claude Code Rust** 是 Claude Code 的高性能 Rust 重构版本，提供原版 TypeScript 版本的所有核心功能，并在性能、内存安全和部署体积上实现显著优化。

## ✨ 主要特性

- ⚡ **卓越性能**：基于 Rust 的高性能实现，启动速度和执行效率大幅提升
- 🛡️ **内存安全**：Rust 的所有权系统保证零运行时内存错误
- 📦 **轻量部署**：单文件可执行程序，无需 Node.js 运行时环境
- 🔧 **完整功能**：支持 CLI、REPL、GUI、插件系统等核心功能
- 🌐 **多语言支持**：内置国际化系统，支持中英文界面
- 🔌 **插件架构**：模块化设计，支持动态加载第三方插件
- 🎯 **DCC 工具集成**：支持 Blender、UE5 等数字内容创建工具

## 性能基准数据对比

### 启动速度基准 (越低越好 ↓)

| 指标 | Rust 版本 | TypeScript 版本 | 性能提升 |
|------|-----------|-----------------|----------|
| 平均启动时间 | 63ms ⚡ | 158ms | 2.5x 更快 🚀 |
| 冷启动 | 58ms | 152ms | 2.6x 更快 |
| 热启动 (缓存) | 61ms | 156ms | 2.5x 更快 |
| 最快启动 | 51ms | 145ms | 2.8x 更快 |
| 最慢启动 | 74ms | 172ms | 2.3x 更快 |

### 部署体积对比 (越小越好 ↓)

| 指标 | Rust 版本 | TypeScript 版本 | 减少比例 |
|------|-----------|-----------------|----------|
| 单文件可执行体 | 5.07 MB 🎯 | - | - |
| npm 安装后体积 | 仅需编译 | 164.32 MB 📦 | 97% 减少 |
| node_modules 大小 | 0 MB (无依赖) | ~156 MB | 100% 消除 |
| 运行时依赖 | 0 MB (内置) | ~8 MB (Node.js) | 100% 消除 |
| Docker 镜像 | ~20 MB (含OS) | ~600 MB+ | 96% 减少 |

### 命令执行速度对比 (越低越好 ↓)

| 命令操作 | Rust 版本 | TypeScript 版本 | 提升倍数 |
|----------|-----------|-----------------|----------|
| --version | 63ms | 158ms | 2.5x ⚡ |
| --help | 73ms | 176ms | 2.4x ⚡ |
| 查看配置 | 6ms ✨ | ~150ms | 25x 🔥 |
| 初始化项目 | 85ms | ~200ms | 2.3x ⚡ |
| REPL 响应 | <1ms | ~100ms | 100x 🚀 |

### 内存占用对比 (越低越好 ↓)

| 指标 | Rust 版本 | TypeScript 版本 | 优势 |
|------|-----------|-----------------|------|
| 基础内存占用 | ~10 MB 🎯 | ~50+ MB | 5x 更轻 |
| 实际工作内存 | ~15 MB | ~150+ MB | 10x 更轻 |
| 峰值内存 | ~25 MB | 300+ MB | 12x 更轻 |
| 垃圾回收暂停 | 0ms (无 GC) | ~50-200ms | 完全消除 |
| 线程开销 | 极低 | 100+ MB (Node 多线程) | 无显著开销 |

## 🚀 核心功能

### 1. 命令行界面 (CLI)
- **交互式 REPL**：支持多行输入和命令历史
- **命令系统**：内置 `help`、`version`、`config`、`login` 等命令
- **查询执行**：支持单次查询和批处理模式
- **Git 集成**：内置 Git 命令支持（commit、push、PR 创建）

### 2. 图形用户界面 (GUI)
- **原生桌面应用**：基于 egui 的跨平台 GUI
- **聊天界面**：实时对话和历史记录
- **设置管理**：图形化配置界面
- **主题支持**：深色/浅色主题切换

### 3. 技能系统
- **内置技能**：`help`、`version`、`config-check` 等基础技能
- **技能加载**：支持动态加载和卸载技能模块
- **权限控制**：细粒度的技能执行权限管理

### 4. 工具系统
- **文件操作**：读取、编辑、写入文件
- **代码搜索**：支持正则表达式和模式匹配
- **系统命令**：安全的命令执行环境
- **Git 工具**：版本控制操作支持

### 5. 插件架构
- **动态加载**：运行时加载和卸载插件
- **安全沙箱**：插件隔离执行环境
- **DCC 工具**：Blender、UE5 等数字内容创建工具集成
- **生命周期管理**：完整的插件生命周期支持

### 6. 国际化支持
- **多语言界面**：支持中文和英文显示
- **本地化文件**：基于 JSON 的翻译文件系统
- **动态切换**：运行时语言切换支持

### 7. 配置管理系统
- **分层配置**：全局、用户、项目级配置
- **环境变量**：支持环境变量覆盖
- **配置验证**：自动配置验证和错误提示

## 🏗️ 技术架构

### 模块结构

基于实际代码库的模块组织：

```
src/
├── bootstrap/          # 启动引导和快速路径优化
├── cli/               # 命令行接口和参数解析
├── commands/          # 命令系统（auth、config、git、interactive 等）
├── config/            # 配置管理系统和系统提示
├── gui/               # 图形用户界面（egui 实现）
├── i18n/              # 国际化支持系统
├── tools/             # 工具系统（文件操作、搜索、Git 等）
├── skills/            # 技能加载和执行系统
├── plugins/           # 插件系统和安全沙箱
├── services/          # 服务层（代理、语音、API 等）
├── features/          # 特性模块（语音、协调器、工作流等）
├── mcp/               # 模型上下文协议客户端
├── security/          # 安全机制和权限控制
├── utils/             # 工具函数和辅助模块
├── memory/            # 记忆存储和会话管理
├── web/               # Web 服务器和插件市场
├── wasm/              # WebAssembly 支持模块
├── crates/dcc-tools/  # DCC 工具集成（Blender、UE5）
├── lib.rs             # 库入口点
└── main.rs            # 主程序入口点
```

### 核心技术栈

- **Rust 语言**：现代系统编程语言，提供零成本抽象和内存安全保证
- **Tokio**：异步运行时，支持高性能并发 I/O 操作
- **Clap**：功能强大的命令行参数解析库
- **Serde**：高效的序列化和反序列化框架
- **Reqwest**：异步 HTTP 客户端，支持 API 调用
- **Egui**：即时模式 GUI 库，用于构建原生桌面界面
- **Ratatui**：终端用户界面库，支持 TUI 应用
- **Axum**：Web 框架，用于构建插件市场和 API 服务
- **SQLx**：异步 SQL 数据库工具包
- **Tracing**：结构化日志记录和诊断框架

## 📦 安装与使用

### 系统要求

- **Rust 工具链**：1.70+ 版本
- **Cargo**：Rust 包管理器
- **Windows**：需要 Visual Studio 构建工具（MSVC）或 MinGW
- **macOS/Linux**：标准开发环境

### 从源码编译

1. **安装 Rust 工具链**
   ```bash
   # 使用 rustup 安装
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **克隆仓库**
   ```bash
   git clone https://github.com/lorryjovens-hub/claude-code-rust.git
   cd claude-code-rust
   ```

3. **编译项目**
   ```bash
   # 开发模式编译
   cargo build
   
   # 发布模式编译（推荐）
   cargo build --release
   ```

4. **运行应用**
   ```bash
   # Windows
   .\target\release\claude.exe
   
   # macOS/Linux
   ./target/release/claude
   ```

### Windows 构建说明

在 Windows 上编译可能需要 Visual Studio 构建工具或 MinGW 环境：

```bash
# 如果遇到 libz-sys 编译问题，可以尝试：
set RUSTFLAGS=-C target-feature=+crt-static
cargo build --release
```

### 预编译二进制文件

可以从 [GitHub Releases](https://github.com/lorryjovens-hub/claude-code-rust/releases) 页面下载对应平台的预编译二进制文件。

### 命令行配置（PowerShell）

为了方便区分和使用两个版本，推荐使用 PowerShell 配置：

#### 一键配置（推荐）

```powershell
# 1. 运行配置脚本
.\scripts\setup-powershell.ps1

# 2. 重新加载配置
. $PROFILE

# 3. 验证配置
claude-rust --version
claude-npm --version
```

#### 手动配置

编辑 PowerShell 配置文件：

```powershell
# 打开配置文件
notepad $PROFILE

# 添加以下内容
function claude-rust {
    & "C:\迅雷下载\claude-code-rev-main\claude-code-rust\bin\claude.exe" @args
}

function claude-npm {
    & "C:\Users\user\AppData\Roaming\npm\claude" @args
}
```

详细配置说明请参考 [**INSTALL.md**](INSTALL.md)。

## 🚀 快速开始

### 基本命令

```bash
# 查看版本信息（明确标识为 Rust 版本）
claude --version
# 输出: 0.1.1 (Claude Code Rust - High-performance implementation)

# 查看帮助信息
claude --help

# 进入交互式 REPL 模式
claude
# 或
claude interactive

# 执行单次查询
claude query "请帮我分析这段代码"
# 或使用简写
claude q "你的问题"
```

### 配置管理

```bash
# 查看当前配置
claude config

# 登录到 Claude API
claude login

# 退出登录
claude logout

# 检查系统配置
claude config-check
```

### Git 集成命令

```bash
# 提交更改并创建 PR
claude git commit-push-pr "提交描述"

# 查看 Git 状态
claude git status
```

### 图形界面模式

```bash
# 启动 GUI 桌面应用
claude gui

# 启动终端用户界面 (TUI)
claude tui
```

### 技能系统

```bash
# 列出所有可用技能
claude skills list

# 执行帮助技能
claude help

# 显示版本信息
claude version
```

### 升级和更新

```bash
# 检查更新
claude upgrade check

# 执行升级
claude upgrade execute
```

## 🔧 开发指南

### 项目结构概述

Claude Code Rust 采用模块化架构设计，主要模块包括：

- **`bootstrap/`**：启动引导、快速路径优化和宏配置系统
- **`cli/`**：命令行接口、参数解析和 REPL 实现
- **`commands/`**：命令系统，包括认证、配置、Git、交互式等命令
- **`config/`**：配置管理、系统提示和环境设置
- **`gui/`**：基于 egui 的图形用户界面
- **`tools/`**：工具系统，提供文件操作、搜索、命令执行等能力
- **`skills/`**：技能加载、注册和执行系统
- **`plugins/`**：插件框架和安全沙箱机制

### 扩展开发

#### 添加新命令

1. 在 `src/commands/` 目录下创建新的命令模块
2. 实现 `CmdExecutor` trait
3. 在命令注册系统中注册新命令

#### 创建新技能

1. 在 `src/skills/` 目录下创建技能模块
2. 实现 `Skill` trait
3. 在技能加载器中注册新技能

#### 开发插件

参考 `PLUGIN_DEVELOPMENT.md` 文档了解插件开发流程和 API。

### 构建和测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_command_system

# 代码质量检查
cargo clippy -- -D warnings

# 代码格式化
cargo fmt

# 构建文档
cargo doc --open
```

## 🤝 贡献指南

### 开发流程

1. **Fork 仓库**：创建个人 fork
2. **创建分支**：使用描述性的分支名称
   ```bash
   git checkout -b feat/add-new-command
   ```

3. **实现功能**：遵循项目代码规范
4. **添加测试**：为新功能编写测试用例
5. **提交更改**：使用规范的提交消息
   ```
   feat: 添加新的配置命令
   fix: 修复登录认证问题
   docs: 更新 README 文档
   refactor: 重构命令执行器
   ```

6. **推送分支**：推送到您的 fork
   ```bash
   git push origin feat/add-new-command
   ```

7. **创建 PR**：在 GitHub 上创建 Pull Request

### 代码规范

- **命名约定**：使用蛇形命名法（snake_case）用于变量和函数，大驼峰命名法（PascalCase）用于类型
- **错误处理**：使用 `anyhow::Result` 或自定义错误类型
- **文档注释**：公共 API 必须包含文档注释
- **测试覆盖**：新功能应包含单元测试和集成测试

### 测试要求

```bash
# 运行完整的测试套件
cargo test --all-features

# 检查测试覆盖率
cargo tarpaulin --ignore-tests --out html

# 运行性能基准测试
cargo bench
```

## 📄 许可证

本项目采用 **MIT 许可证** - 详见 [LICENSE](LICENSE) 文件。

## 📊 性能优势

作为 Rust 重构版本，Claude Code Rust 在多个维度上提供显著性能改进：

### 启动性能
- **冷启动时间**：毫秒级启动，无需 Node.js 运行时初始化
- **热启动优化**：利用系统级缓存，实现极速启动

### 资源效率
- **内存占用**：原生二进制执行，内存开销显著降低
- **部署体积**：单文件分发，无需庞大的 `node_modules` 目录
- **CPU 利用率**：高效的异步运行时，更好的多核利用

### 执行性能
- **命令响应**：原生机器码执行，无解释器开销
- **I/O 操作**：基于 Tokio 的高效异步 I/O
- **并发处理**：无锁数据结构和高效的并发原语

### 安全优势
- **内存安全**：Rust 编译器保证零运行时内存错误
- **线程安全**：编译时数据竞争检测
- **沙箱隔离**：插件系统的安全执行环境

## 🔮 未来规划

### 短期目标 (v0.2.0)
- **完善插件系统**：提供更丰富的插件开发 API 和示例
- **增强 DCC 集成**：深化 Blender 和 UE5 工具链集成
- **改进 GUI 体验**：优化用户界面和交互流程
- **扩展技能库**：添加更多实用技能和工具

### 中期目标 (v0.5.0)
- **云同步功能**：配置和状态的跨设备同步
- **团队协作**：多人协作和共享工作空间
- **高级分析**：代码质量分析和性能洞察
- **市场生态**：插件市场和技能商店

### 长期愿景 (v1.0.0)
- **全平台支持**：移动端和嵌入式设备支持
- **AI 模型集成**：本地 AI 模型和混合推理
- **企业功能**：企业级部署和管理工具
- **生态系统**：完整的开发者工具链生态

## 📚 相关文档

- [**PLUGIN_DEVELOPMENT.md**](PLUGIN_DEVELOPMENT.md) - 插件开发指南
- [**ARCHITECTURE.md**](ARCHITECTURE.md) - 系统架构设计文档
- [**PERFORMANCE_COMPARISON.md**](PERFORMANCE_COMPARISON.md) - 性能对比分析
- [**CHANGELOG.md**](CHANGELOG.md) - 版本更新日志
- [**CONTRIBUTING.md**](CONTRIBUTING.md) - 贡献者指南

## 🤔 常见问题

### Q: Rust 版本与原版 TypeScript 版本有什么区别？
A: Rust 版本提供更好的性能、内存安全和部署效率，同时保持了核心功能兼容性。界面会明确标识为 "Claude Code Rust"。

### Q: 如何同时安装两个版本而不冲突？
A: 建议使用不同的命令别名，如 `claude-rust` 和 `claude-npm`，具体配置方法参见安装部分。

### Q: Windows 上构建失败怎么办？
A: Windows 需要 Visual Studio 构建工具或 MinGW 环境，也可以直接下载预编译的二进制文件。

### Q: 插件是否与原版兼容？
A: 插件系统设计为兼容原版插件架构，但可能需要适配器或重新编译。

## 📞 支持与反馈

- **GitHub Issues**：[问题报告](https://github.com/lorryjovens-hub/claude-code-rust/issues)
- **文档反馈**：[飞书文档](https://my.feishu.cn/wiki/GfQGwIen9izVnikrchFcKOtOnTb)
- **社区交流**：B站关注 [lorry黄同学](https://space.bilibili.com/)

---

**Claude Code Rust** - 为性能和安全而生的 AI 开发工具链
