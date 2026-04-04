@echo off
setlocal enabledelayedexpansion

echo ===============================
echo Claude Code 性能测试脚本
echo ===============================
echo.

rem 测试Rust版本
echo 测试 Rust 版本...
echo -------------------------------

rem 测试启动速度
echo 1. 测试启动速度：
for /L %%i in (1,1,5) do (
    set "starttime=!time!"
    target\release\claude.exe --version > nul
    set "endtime=!time!"
    call :calc_time "!starttime!" "!endtime!" rust_startup%%i
)

rem 测试命令执行速度
echo 2. 测试命令执行速度：
for /L %%i in (1,1,5) do (
    set "starttime=!time!"
    target\release\claude.exe config > nul
    set "endtime=!time!"
    call :calc_time "!starttime!" "!endtime!" rust_command%%i
)

rem 测试TypeScript版本
echo.
echo 测试 TypeScript 版本...
echo -------------------------------

rem 测试启动速度
echo 1. 测试启动速度：
for /L %%i in (1,1,5) do (
    set "starttime=!time!"
    node_modules\.bin\claude --version > nul
    set "endtime=!time!"
    call :calc_time "!starttime!" "!endtime!" ts_startup%%i
)

rem 测试命令执行速度
echo 2. 测试命令执行速度：
for /L %%i in (1,1,5) do (
    set "starttime=!time!"
    node_modules\.bin\claude config > nul
    set "endtime=!time!"
    call :calc_time "!starttime!" "!endtime!" ts_command%%i
)

rem 计算平均值
echo.
echo 性能测试结果：
echo ===============================

rem 计算Rust启动速度平均值
set "sum=0"
for /L %%i in (1,1,5) do (
    set /a "sum+=!rust_startup%%i!"
)
set /a "avg_rust_startup=sum/5"
echo Rust 版本启动速度：!avg_rust_startup! ms

rem 计算Rust命令执行速度平均值
set "sum=0"
for /L %%i in (1,1,5) do (
    set /a "sum+=!rust_command%%i!"
)
set /a "avg_rust_command=sum/5"
echo Rust 版本命令执行速度：!avg_rust_command! ms

rem 计算TypeScript启动速度平均值
set "sum=0"
for /L %%i in (1,1,5) do (
    set /a "sum+=!ts_startup%%i!"
)
set /a "avg_ts_startup=sum/5"
echo TypeScript 版本启动速度：!avg_ts_startup! ms

rem 计算TypeScript命令执行速度平均值
set "sum=0"
for /L %%i in (1,1,5) do (
    set /a "sum+=!ts_command%%i!"
)
set /a "avg_ts_command=sum/5"
echo TypeScript 版本命令执行速度：!avg_ts_command! ms

echo.
echo 性能提升：
echo -------------------------------
set /a "startup_improvement=avg_ts_startup*100/avg_rust_startup"
echo 启动速度提升：!startup_improvement!%%

set /a "command_improvement=avg_ts_command*100/avg_rust_command"
echo 命令执行速度提升：!command_improvement!%%

goto :eof

:calc_time
set "start=%~1"
set "end=%~2"
set "var=%~3"

rem 解析开始时间
for /f "tokens=1-4 delims=:.," %%a in ("%start%") do (
    set "sh=%%a"
    set "sm=%%b"
    set "ss=%%c"
    set "sms=%%d"
)

rem 解析结束时间
for /f "tokens=1-4 delims=:.," %%a in ("%end%") do (
    set "eh=%%a"
    set "em=%%b"
    set "es=%%c"
    set "ems=%%d"
)

rem 计算时间差（毫秒）
set /a "start_total=sh*3600000+sm*60000+ss*1000+sms"
set /a "end_total=eh*3600000+em*60000+es*1000+ems"
set /a "diff=end_total-start_total"

rem 处理负值（跨小时）
if !diff! lss 0 set /a "diff+=24*3600000"

set "%var%=%diff%"
goto :eof