//! 画质预设与可调参数。配置文件可由 CLI 覆盖。

use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Low,
    Medium,
    High,
    Ultra,
}

impl Quality {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "low" | "l" | "低" => Some(Self::Low),
            "medium" | "med" | "m" | "中" => Some(Self::Medium),
            "high" | "h" | "高" => Some(Self::High),
            "ultra" | "u" | "最高" => Some(Self::Ultra),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Ultra => "ultra",
        }
    }

    pub fn apply(self, cfg: &mut Config) {
        match self {
            Self::Low => {
                cfg.step = 4;
                cfg.fps = 24.0;
                cfg.legend_stride = 4;
                cfg.count = 12;
            }
            Self::Medium => {
                cfg.step = 2;
                cfg.fps = 30.0;
                cfg.legend_stride = 2;
                cfg.count = 17;
            }
            Self::High => {
                cfg.step = 1;
                cfg.fps = 30.0;
                cfg.legend_stride = 1;
                cfg.count = 17;
            }
            Self::Ultra => {
                cfg.step = 1;
                cfg.fps = 60.0;
                cfg.legend_stride = 1;
                cfg.count = 17;
            }
        }
        cfg.quality = self;
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub wallpaper: bool,
    pub windowed: bool,
    pub seed: u32,
    pub quality: Quality,
    pub step: usize,
    pub fps: f64,
    pub count: usize,
    pub legend: bool,
    pub formula: bool,
    pub legend_stride: usize,
    pub point_size: f32,
    pub vsync: bool,
    pub setup: bool,
    pub lang: String,
}

impl Config {
    pub fn default_high() -> Self {
        let mut cfg = Self {
            wallpaper: false,
            windowed: false,
            seed: now_seed(),
            quality: Quality::High,
            step: 1,
            fps: 30.0,
            count: 17,
            legend: true,
            formula: true,
            legend_stride: 1,
            point_size: 1.0,
            vsync: true,
            setup: false,
            lang: detect_lang(),
        };
        Quality::High.apply(&mut cfg);
        cfg
    }

    pub fn clamp(&mut self) {
        self.step = self.step.clamp(1, 12);
        self.fps = self.fps.clamp(10.0, 120.0);
        self.count = self.count.clamp(1, crate::formulas::SPECIES.len());
        self.legend_stride = self.legend_stride.clamp(1, 16);
        self.point_size = self.point_size.clamp(0.4, 3.0);
        self.lang = normalize_lang(&self.lang);
    }
}

fn detect_lang() -> String {
    for key in ["CYBER_OCEAN_LANG", "LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(v) = std::env::var(key) {
            if v.is_empty() {
                continue;
            }
            return normalize_lang(&v);
        }
    }
    "zh".into()
}

fn normalize_lang(s: &str) -> String {
    let low = s.trim().to_ascii_lowercase();
    if low == "en" || low.starts_with("en_") || low.starts_with("en-") || low.starts_with("en.") {
        "en".into()
    } else {
        "zh".into()
    }
}

fn now_seed() -> u32 {
    (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u32)
        | 1
}

pub fn config_path() -> PathBuf {
    if let Ok(raw) = std::env::var("CYBER_OCEAN_CONFIG") {
        let p = PathBuf::from(raw);
        if p.extension().is_some() {
            return p;
        }
        return p.join("settings.json");
    }
    if cfg!(windows) {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| dirs_home().join("AppData").join("Roaming"));
        return base.join("cyber-ocean").join("settings.json");
    }
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs_home().join(".config"));
    base.join("cyber-ocean").join("settings.json")
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Default)]
struct FileOverlay {
    quality: Option<Quality>,
    step: Option<usize>,
    fps: Option<f64>,
    count: Option<usize>,
    legend: Option<bool>,
    formula: Option<bool>,
    legend_stride: Option<usize>,
    point_size: Option<f32>,
    vsync: Option<bool>,
    seed: Option<u32>,
    lang: Option<String>,
}

fn parse_json_overlay(src: &str) -> FileOverlay {
    let mut o = FileOverlay::default();
    o.quality = json_str(src, "quality").and_then(|s| Quality::parse(&s));
    o.step = json_u(src, "step");
    o.fps = json_f(src, "fps");
    o.count = json_u(src, "count");
    o.legend = json_bool(src, "legend");
    o.formula = json_bool(src, "formula");
    o.legend_stride = json_u(src, "legend_stride");
    o.point_size = json_f(src, "point_size").map(|v| v as f32);
    o.vsync = json_bool(src, "vsync");
    o.seed = json_u(src, "seed").map(|v| v as u32);
    o.lang = json_str(src, "lang");
    o
}

fn json_after<'a>(src: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{key}\"");
    let i = src.find(&pat)?;
    let rest = src[i + pat.len()..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    Some(rest)
}

fn json_str(src: &str, key: &str) -> Option<String> {
    let rest = json_after(src, key)?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn json_u(src: &str, key: &str) -> Option<usize> {
    json_num(src, key)?.parse().ok()
}

fn json_f(src: &str, key: &str) -> Option<f64> {
    json_num(src, key)?.parse().ok()
}

fn json_num<'a>(src: &'a str, key: &str) -> Option<&'a str> {
    let rest = json_after(src, key)?;
    let n = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+'))
        .unwrap_or(rest.len());
    let s = rest[..n].trim();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn json_bool(src: &str, key: &str) -> Option<bool> {
    let rest = json_after(src, key)?;
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn apply_overlay(cfg: &mut Config, o: &FileOverlay) {
    if let Some(q) = o.quality {
        q.apply(cfg);
    }
    if let Some(v) = o.step {
        cfg.step = v;
    }
    if let Some(v) = o.fps {
        cfg.fps = v;
    }
    if let Some(v) = o.count {
        cfg.count = v;
    }
    if let Some(v) = o.legend {
        cfg.legend = v;
    }
    if let Some(v) = o.formula {
        cfg.formula = v;
    }
    if let Some(v) = o.legend_stride {
        cfg.legend_stride = v;
    }
    if let Some(v) = o.point_size {
        cfg.point_size = v;
    }
    if let Some(v) = o.vsync {
        cfg.vsync = v;
    }
    if let Some(v) = o.seed {
        cfg.seed = v;
    }
    if let Some(v) = &o.lang {
        cfg.lang = normalize_lang(v);
    }
}

fn load_file(path: &Path, cfg: &mut Config) {
    let Ok(src) = std::fs::read_to_string(path) else {
        return;
    };
    apply_overlay(cfg, &parse_json_overlay(&src));
}

pub fn reload_file_into(cfg: &mut Config) {
    let wallpaper = cfg.wallpaper;
    let windowed = cfg.windowed;
    let seed = cfg.seed;
    let setup = cfg.setup;
    let mut next = Config::default_high();
    load_file(&config_path(), &mut next);
    next.wallpaper = wallpaper;
    next.windowed = windowed;
    next.seed = seed;
    next.setup = setup;
    next.clamp();
    *cfg = next;
}

pub fn config_mtime() -> Option<std::time::SystemTime> {
    std::fs::metadata(config_path())
        .and_then(|m| m.modified())
        .ok()
}

fn find_python() -> Option<PathBuf> {
    if let Ok(raw) = std::env::var("CYBER_OCEAN_PYTHON") {
        let p = PathBuf::from(raw);
        if p.is_file() {
            return Some(p);
        }
    }
    if let Ok(appdir) = std::env::var("APPDIR") {
        let p = PathBuf::from(appdir).join("usr").join("python").join("bin").join("python3");
        if p.is_file() {
            return Some(p);
        }
    }
    let names: &[&str] = if cfg!(windows) {
        &["pythonw.exe", "python.exe", "py.exe", "python3"]
    } else {
        &["python3", "python"]
    };
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            for name in names {
                let p = dir.join(name);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn find_project_root() -> Option<PathBuf> {
    if let Ok(raw) = std::env::var("CYBER_OCEAN_ROOT") {
        let p = PathBuf::from(raw);
        if p.join("fucan").is_dir() {
            return Some(p);
        }
    }
    if let Ok(appdir) = std::env::var("APPDIR") {
        let p = PathBuf::from(appdir).join("usr").join("share").join("cyber-ocean");
        if p.join("fucan").is_dir() {
            return p.canonicalize().ok().or(Some(p));
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin) = exe.parent() {
            let share = bin.join("..").join("share").join("cyber-ocean");
            if share.join("fucan").is_dir() {
                return share.canonicalize().ok();
            }
            let repo = bin.join("..").join("..").join("..");
            if repo.join("fucan").is_dir() {
                return repo.canonicalize().ok();
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("fucan").is_dir() {
            return Some(cwd);
        }
    }
    None
}

pub fn spawn_settings_dialog() -> Option<std::process::Child> {
    let py = find_python()?;
    let root = find_project_root()?;
    let mut cmd = std::process::Command::new(&py);
    cmd.current_dir(&root)
        .args(["-m", "fucan.settings"])
        .stdin(std::process::Stdio::null());
    let mut pp = root.display().to_string();
    if let Ok(old) = std::env::var("PYTHONPATH") {
        pp = format!("{pp}:{old}");
    }
    cmd.env("PYTHONPATH", pp);
    match cmd.spawn() {
        Ok(child) => Some(child),
        Err(err) => {
            eprintln!("无法打开设置窗口（{}）：{err}", py.display());
            None
        }
    }
}

pub fn run_settings_dialog_blocking() -> bool {
    let Some(mut child) = spawn_settings_dialog() else {
        return false;
    };
    child.wait().map(|s| s.success()).unwrap_or(false)
}

pub fn parse_args() -> Config {
    let mut cfg = Config::default_high();
    load_file(&config_path(), &mut cfg);

    let mut wallpaper = cfg.wallpaper;
    let mut windowed = cfg.windowed;
    let mut dump = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--wallpaper" => wallpaper = true,
            "--screensaver" => wallpaper = false,
            "--windowed" => windowed = true,
            "--setup" => cfg.setup = true,
            "--no-setup" => cfg.setup = false,
            "--no-legend" => cfg.legend = false,
            "--legend" => cfg.legend = true,
            "--no-formula" => cfg.formula = false,
            "--formula" => cfg.formula = true,
            "--vsync" => cfg.vsync = true,
            "--no-vsync" => cfg.vsync = false,
            "--quality" => {
                if let Some(v) = args.next() {
                    if let Some(q) = Quality::parse(&v) {
                        q.apply(&mut cfg);
                    }
                }
            }
            "--step" | "--density" => {
                if let Some(v) = args.next() {
                    cfg.step = v.parse().unwrap_or(cfg.step);
                }
            }
            "--fps" => {
                if let Some(v) = args.next() {
                    cfg.fps = v.parse().unwrap_or(cfg.fps);
                }
            }
            "--count" => {
                if let Some(v) = args.next() {
                    cfg.count = v.parse().unwrap_or(cfg.count);
                }
            }
            "--legend-stride" => {
                if let Some(v) = args.next() {
                    cfg.legend_stride = v.parse().unwrap_or(cfg.legend_stride);
                }
            }
            "--point-size" => {
                if let Some(v) = args.next() {
                    cfg.point_size = v.parse().unwrap_or(cfg.point_size);
                }
            }
            "--seed" => {
                if let Some(v) = args.next() {
                    cfg.seed = v.parse().unwrap_or(cfg.seed);
                }
            }
            "--lang" => {
                if let Some(v) = args.next() {
                    cfg.lang = normalize_lang(&v);
                }
            }
            "--print-config" => dump = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
    }
    cfg.wallpaper = wallpaper;
    cfg.windowed = windowed;
    cfg.clamp();
    if dump {
        print_config(&cfg);
        std::process::exit(0);
    }
    cfg
}

pub fn print_config(cfg: &Config) {
    eprintln!(
        "quality={} step={} fps={} count={} legend={} formula={} legend_stride={} point_size={} vsync={} seed={} lang={} config={}",
        cfg.quality.as_str(),
        cfg.step,
        cfg.fps,
        cfg.count,
        cfg.legend,
        cfg.formula,
        cfg.legend_stride,
        cfg.point_size,
        cfg.vsync,
        cfg.seed,
        cfg.lang,
        config_path().display()
    );
}

fn print_help() {
    eprintln!(
        "\
cyber-ocean-native — 赛博海洋馆原生屏保

  --screensaver          全屏屏保（键鼠退出）
  --wallpaper            全屏壁纸（键鼠不退出）
  --windowed             窗口模式
  --setup                启动前弹出设置窗口
  --quality low|medium|high|ultra
                         画质预设（默认 high）
  --step N               粒子密度，1 最密，越大越疏
  --fps N                帧率上限（默认随预设，high=30）
  --count N              生物数量 1–17
  --legend / --no-legend 左上图例
  --formula / --no-formula 高亮生物公式
  --legend-stride N      图例抽样，1 全点
  --point-size F         点大小倍率，默认 1.0
  --vsync / --no-vsync
  --seed N
  --lang zh|en           图例标题语言（设置窗口可切换）
  --print-config         打印当前生效参数后退出

运行中在画面上点右键可再次打开设置。空闲自动启动请加 --no-setup。

配置文件（CLI 可覆盖）：
  {}
也可用环境变量 CYBER_OCEAN_CONFIG 指定路径。

预设：
  low     step=4  fps=24  12 只  图例 1/4
  medium  step=2  fps=30  17 只  图例 1/2
  high    step=1  fps=30  17 只  图例全点
  ultra   step=1  fps=60  17 只  图例全点",
        config_path().display()
    );
}
