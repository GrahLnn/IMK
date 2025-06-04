@echo off
echo 正在构建 IMK 输入法管理器...

REM 发布应用程序
dotnet publish -c Release

if %ERRORLEVEL% EQU 0 (
    echo.
    echo 构建成功！
    echo 可执行文件位置: bin\Release\net8.0-windows\publish\IMK.exe
    echo.
    echo 注意: 运行此程序需要目标系统安装 .NET 8.0 运行时
    echo.
    pause
) else (
    echo.
    echo 构建失败！
    echo.
    pause
) 