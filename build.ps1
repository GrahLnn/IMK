Write-Host "正在构建 IMK 输入法管理器..." -ForegroundColor Green

# 发布应用程序
dotnet publish -c Release

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "构建成功！" -ForegroundColor Green
    Write-Host "可执行文件位置: bin\Release\net8.0-windows\publish\IMK.exe" -ForegroundColor Yellow
    Write-Host ""
    
    # 显示文件大小
    $exePath = "bin\Release\net8.0-windows\publish\IMK.exe"
    if (Test-Path $exePath) {
        $fileSize = (Get-Item $exePath).Length / 1KB
        Write-Host "文件大小: $([math]::Round($fileSize, 0)) KB" -ForegroundColor Cyan
        
        # 显示整个发布文件夹的大小
        $publishDir = "bin\Release\net8.0-windows\publish\"
        if (Test-Path $publishDir) {
            $totalSize = (Get-ChildItem $publishDir -Recurse | Measure-Object -Property Length -Sum).Sum / 1MB
            Write-Host "发布文件夹总大小: $([math]::Round($totalSize, 2)) MB" -ForegroundColor Cyan
        }
    }
    
    Write-Host ""
    Write-Host "注意: 运行此程序需要目标系统安装 .NET 8.0 运行时" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "按任意键继续..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
} else {
    Write-Host ""
    Write-Host "构建失败！" -ForegroundColor Red
    Write-Host ""
    Write-Host "按任意键继续..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
} 