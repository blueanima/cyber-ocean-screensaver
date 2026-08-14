# 赛博海洋馆屏幕保护

公式生物在深海里游动：白点勾勒的浮蚕、水母、栉水母……互相躲开，也会被指针轻轻推开。支持 **macOS / Linux / Windows**，只需 Python 3.10+ 和 Chrome / Edge / Firefox。

## 快速开始

```bash
git clone https://github.com/blueanima/cyber-ocean-screensaver.git
cd cyber-ocean-screensaver
python3 main.py --screensaver
```

移动鼠标或按任意键退出。需要 Chrome、Edge、Chromium 或 Firefox（全屏 kiosk）。若没装这些浏览器，会打开普通标签页，可按 `F11` 全屏。

| 命令 | 作用 |
| --- | --- |
| `python3 main.py --screensaver` | 屏幕保护（键鼠退出） |
| `python3 main.py --wallpaper` | 壁纸 / 展览模式（键鼠不退出） |
| `python3 main.py` | 集合馆：点选单只生物、赛博海洋、分步讲解 |

## 各系统安装

### Linux

```bash
chmod +x scripts/install-linux.sh
./scripts/install-linux.sh
cyber-ocean-screensaver
```

空闲自动启动示例：

```bash
# swayidle
swayidle -w timeout 180 'cyber-ocean-screensaver'

# hypridle（hyprland）
# listener { timeout = 180; on-timeout = cyber-ocean-screensaver; }
```

### macOS

```bash
chmod +x scripts/install-macos.sh
./scripts/install-macos.sh
open "$HOME/Applications/CyberOcean Screensaver.app"
```

请先安装 [Chrome](https://www.google.com/chrome/) 或 Edge，以便进入真正的全屏 kiosk。也可把该应用放到触发角（快捷角）里，当作屏保入口。

### Windows

```powershell
powershell -ExecutionPolicy Bypass -File scripts\install-windows.ps1
python main.py --screensaver
```

系统屏保（`.scr`）：

1. 从 [Releases](../../releases) 下载 `CyberOcean.scr`（由 GitHub Actions 打包）
2. 右键 → **安装**
3. 设置 → 个性化 → 锁屏界面 → 屏幕保护程序 → 选「CyberOcean」

也可用 [Lively Wallpaper](https://github.com/rocksdanister/lively) 当动态壁纸：先 `python main.py --wallpaper --no-browser`，再在 Lively 里添加

`http://127.0.0.1:8765/screensaver?wallpaper=1`

## 写出离线 HTML

不启动服务器、给 Lively 或浏览器直接打开：

```bash
python3 main.py --write-screensaver screensaver.html
python3 main.py --write-screensaver wallpaper.html --wallpaper
```

## 依赖

仅 Python 标准库，无 pip 包。需要本机已安装图形浏览器才能全屏。

## 致谢

- 部分公式生物来自 [@yuruyurau](https://x.com/yuruyurau) 的参数方程，Matlab 复现见 [slandarer digital life](https://ww2.mathworks.cn/matlabcentral/fileexchange/179115)
- 北斗浮蚕与若干原创物种为本仓库实现

## 许可

MIT。公式造型的视觉风格归原作者；本仓库提供的是独立的展示与屏保程序。
