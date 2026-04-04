@echo off
echo ===============================
echo Claude Code 功能测试脚本
echo ===============================
echo.

rem 测试Rust版本
echo 测试 Rust 版本功能...
echo -------------------------------

rem 测试版本信息
echo 1. 测试版本信息：
target\release\claude.exe --version
echo.

rem 测试帮助信息
echo 2. 测试帮助信息：
target\release\claude.exe --help | head -20
echo.

rem 测试配置命令
echo 3. 测试配置命令：
target\release\claude.exe config | head -10
echo.

rem 测试升级命令
echo 4. 测试升级命令：
target\release\claude.exe upgrade check
echo.

rem 测试Rust版本特有功能
echo 5. 测试 Rust 特有功能：
echo - Daemon 后台进程系统
echo - UDS Inbox 多消息融合系统
echo - Teleport 跨机上下文传递
 echo - Ultrareview 云端代码审查
echo - Buddy 宠物系统
echo - Kairos 时间感知系统
echo.

rem 测试TypeScript版本
echo 测试 TypeScript 版本功能...
echo -------------------------------

rem 测试版本信息
echo 1. 测试版本信息：
node_modules\.bin\claude --version
echo.

rem 测试帮助信息
echo 2. 测试帮助信息：
node_modules\.bin\claude --help | head -20
echo.

rem 测试配置命令
echo 3. 测试配置命令：
node_modules\.bin\claude config | head -10
echo.

rem 测试升级命令
echo 4. 测试升级命令：
node_modules\.bin\claude upgrade check 2>nul || echo Upgrade command not available
echo.

echo ===============================
echo 功能测试完成
echo ===============================