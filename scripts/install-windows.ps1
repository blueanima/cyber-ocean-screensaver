# 赛博海洋馆 · Windows 安装
# 用法：powershell -ExecutionPolicy Bypass -File scripts\install-windows.ps1

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Python = Get-Command python -ErrorAction SilentlyContinue
if (-not $Python) { $Python = Get-Command python3 -ErrorAction SilentlyContinue }
if (-not $Python) { throw "未找到 python / python3，请先安装 Python 3.10+" }

$StartMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$Desktop = [Environment]::GetFolderPath("Desktop")
New-Item -ItemType Directory -Force -Path $StartMenu | Out-Null

function New-Shortcut($Path, $Arguments, $Name) {
    $w = New-Object -ComObject WScript.Shell
    $lnk = $w.CreateShortcut((Join-Path $Path $Name))
    $lnk.TargetPath = $Python.Source
    $lnk.Arguments = "`"$Root\main.py`" $Arguments"
    $lnk.WorkingDirectory = $Root
    $lnk.WindowStyle = 7
    $lnk.Description = "赛博海洋馆"
    $lnk.Save()
}

New-Shortcut $StartMenu "--screensaver" "赛博海洋馆屏幕保护.lnk"
New-Shortcut $StartMenu "" "赛博海洋馆集合馆.lnk"
New-Shortcut $Desktop "--screensaver" "赛博海洋馆屏幕保护.lnk"

Write-Host "已创建开始菜单和桌面快捷方式。"
Write-Host ""
Write-Host "立即预览："
Write-Host "  python `"$Root\main.py`" --screensaver"
Write-Host ""
Write-Host "作为系统屏保（.scr）："
Write-Host "  1. 从 GitHub Releases 下载 CyberOcean.scr"
Write-Host "  2. 右键 → 安装"
Write-Host "  3. 设置 → 个性化 → 锁屏界面 → 屏幕保护程序"
Write-Host ""
Write-Host "或用 Lively Wallpaper 加载本页（壁纸模式，鼠标不会退出）："
Write-Host "  python `"$Root\main.py`" --wallpaper --no-browser"
Write-Host "  然后在 Lively 中添加网址 http://127.0.0.1:8765/screensaver?wallpaper=1"
