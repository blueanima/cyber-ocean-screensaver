# 赛博海洋馆屏幕保护 / Cyber Ocean Screensaver

公式生物在深海里游动：白点勾勒的浮蚕、水母、栉水母……互相躲开，也会被指针轻轻推开。

Parametric sea-life drawn as translucent scatter points. They steer clear of one another and drift away from the pointer. Runs on **macOS / Linux / Windows** with Python 3.10+ and Chrome, Edge, or Firefox.

## 截图 / Screenshots

![赛博海洋与动态图例 / Cyber ocean with live legend](docs/screenshots/ocean.png)

*屏保 / 壁纸模式：左上角图例为中英对照，剪影随生物一起游动。*  
*Screensaver / wallpaper: bilingual legend, live silhouettes, lock-on line to the highlighted creature.*

![集合馆 / Gallery](docs/screenshots/gallery.png)

*集合馆：顶栏、底栏与图例均为中英双语。点击图例可点亮对应生物。*  
*Gallery HUD is bilingual. Click a legend row to pulse that creature.*

## 快速开始 / Quick start

```bash
git clone https://github.com/blueanima/cyber-ocean-screensaver.git
cd cyber-ocean-screensaver
python3 main.py --screensaver
```

移动鼠标或按任意键退出。Move the mouse or press any key to exit.

| 命令 Command | 中文 | English |
| --- | --- | --- |
| `python3 main.py --screensaver` | 屏幕保护（键鼠退出） | Screensaver (input exits) |
| `python3 main.py --wallpaper` | 壁纸 / 展览（键鼠不退出） | Wallpaper / exhibit (input stays) |
| `python3 main.py` | 集合馆、赛博海洋、分步讲解 | Gallery, ocean, step-by-step lesson |

需要 Chrome / Edge / Chromium / Firefox 才能全屏 kiosk；否则打开普通标签页，可按 `F11`。  
A kiosk browser is required for true fullscreen; otherwise open a tab and press `F11`.

## 图例怎么读 / How to read the legend

左上角 **图例 / Legend** 列出当前海里的每只公式生物：

- 左侧小剪影与海里的个体同步摆动  
- 中文名 + 英文名  
- 指针靠近，或点选图例，会高亮并用虚线连过去  
- 屏保模式下图例会自动轮询点名  

The **Legend** (top-left) lists every creature in the tank: a live mini silhouette, Chinese + English names, a dashed lock-on when highlighted, and an auto-scan in screensaver mode.

## 生物名录 / Species

| 中文 | English | 备注 Notes |
| --- | --- | --- |
| 北斗浮蚕 | Beidou Fucan | 海报公式 / poster formula |
| 蚰蜒 | House Centipede | yuruyurau · life 1 |
| 脊虫 | Spine Worm | life 2 |
| 小水母 | Jellyfish | life 3 |
| 星云水母 | Nebula Jelly | life 4 |
| 花水母 | Lantern Jelly | life 5 |
| 羽鳃 | Feather Gill | life 6 |
| 触须虫 | Tentacle Worm | life 7 |
| 六瓣花 | Six-petal | life 8 |
| 轮虫花 | Rotifer Wheel | life 9 |
| 螺灯 | Spiral Lamp | 原创 / original |
| 栉水母 | Comb Jelly | 原创 / original |
| 锯鳗 | Saw Eel | 原创 / original |
| 八腕星 | Octo Star | 原创 / original |
| 磷虾 | Krill | 原创 / original |
| 涡虫 | Vortex Worm | 原创 / original |
| 海天使 | Sea Angel | 原创 / original |

## 安装 / Install

### Linux

```bash
chmod +x scripts/install-linux.sh
./scripts/install-linux.sh
cyber-ocean-screensaver
```

空闲启动 Idle hook：`swayidle -w timeout 180 'cyber-ocean-screensaver'`

### macOS

```bash
chmod +x scripts/install-macos.sh
./scripts/install-macos.sh
open "$HOME/Applications/CyberOcean Screensaver.app"
```

请先安装 Chrome 或 Edge。可将该应用放到触发角。  
Install Chrome or Edge first. Optional: use as a Hot Corner action.

### Windows

```powershell
powershell -ExecutionPolicy Bypass -File scripts\install-windows.ps1
python main.py --screensaver
```

系统屏保：从 [Releases](../../releases) 下载 `CyberOcean.scr`，右键「安装」。  
OS screensaver: download `CyberOcean.scr` from Releases, right-click **Install**.

动态壁纸 Lively Wallpaper：`python main.py --wallpaper --no-browser`，再添加

`http://127.0.0.1:8765/screensaver?wallpaper=1`

## 离线 HTML / Offline HTML

```bash
python3 main.py --write-screensaver screensaver.html
python3 main.py --write-screensaver wallpaper.html --wallpaper
```

固定种子截图 Fixed-seed screenshot URL：

`http://127.0.0.1:8765/screensaver?wallpaper=1&seed=42&shot=1`

重新截图 Regenerate shots：`./scripts/capture-screenshots.sh`

## 依赖 / Dependencies

仅 Python 标准库，无 pip 包。Only the Python standard library. A graphical browser is needed for fullscreen.

## 致谢 / Credits

- 部分公式生物来自 [@yuruyurau](https://x.com/yuruyurau)，Matlab 复现见 [slandarer digital life](https://ww2.mathworks.cn/matlabcentral/fileexchange/179115)  
  Some formulas follow @yuruyurau, as reproduced in slandarer’s Matlab digital life series.
- 北斗浮蚕与若干原创物种为本仓库实现。  
  Beidou Fucan and several extra species are original to this repo.

## 许可 / License

MIT。公式造型的视觉风格归原作者；本仓库是独立的展示与屏保程序。  
MIT. Visual style of the original formulas belongs to their authors; this repo is a separate viewer and screensaver.
