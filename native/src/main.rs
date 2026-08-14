mod formulas;
mod gait;
mod gpu;
mod hud;
mod ocean;
mod settings;

use std::process::Child;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowId};

use settings::Config;

struct App {
    cfg: Config,
    window: Option<Arc<Window>>,
    gpu: Option<gpu::Gpu>,
    ocean: Vec<ocean::Creature>,
    scratches: Vec<Vec<[f32; 2]>>,
    points: Vec<gpu::Instance>,
    text: hud::TextEngine,
    hud: Vec<hud::HudInstance>,
    last: Instant,
    start: Instant,
    next_frame: Instant,
    ocean_t: f64,
    pointer: (f64, f64),
    highlight: usize,
    armed: bool,
    last_mouse: Option<(f64, f64)>,
    dialog: Option<Child>,
    cfg_mtime: Option<SystemTime>,
}

impl App {
    fn new(cfg: Config) -> Self {
        let now = Instant::now();
        let ocean = ocean::spawn(cfg.seed, cfg.count);
        let scratches = (0..ocean.len())
            .map(|_| Vec::with_capacity(24_000))
            .collect();
        Self {
            ocean,
            cfg,
            window: None,
            gpu: None,
            scratches,
            points: Vec::with_capacity(180_000),
            text: hud::TextEngine::new(),
            hud: Vec::with_capacity(2048),
            last: now,
            start: now,
            next_frame: now,
            ocean_t: 0.0,
            pointer: (0.5, 0.5),
            highlight: 0,
            armed: false,
            last_mouse: None,
            dialog: None,
            cfg_mtime: settings::config_mtime(),
        }
    }

    fn frame_interval(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.cfg.fps.max(1.0))
    }

    fn settings_open(&mut self) -> bool {
        if let Some(child) = self.dialog.as_mut() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    self.dialog = None;
                    false
                }
                Ok(None) => true,
                Err(_) => {
                    self.dialog = None;
                    false
                }
            }
        } else {
            false
        }
    }

    fn open_settings(&mut self) {
        if self.settings_open() {
            return;
        }
        if let Some(child) = settings::spawn_settings_dialog() {
            self.dialog = Some(child);
            if let Some(w) = self.window.as_ref() {
                w.set_cursor_visible(true);
            }
        }
    }

    fn sync_cursor(&self) {
        if let Some(w) = self.window.as_ref() {
            w.set_cursor_visible(
                self.cfg.wallpaper || self.cfg.windowed || self.dialog.is_some(),
            );
        }
    }

    fn apply_reloaded(&mut self) {
        let old_count = self.cfg.count;
        settings::reload_file_into(&mut self.cfg);
        if self.cfg.count != old_count || self.ocean.len() != self.cfg.count {
            self.ocean = ocean::spawn(self.cfg.seed, self.cfg.count);
            self.scratches = (0..self.ocean.len())
                .map(|_| Vec::with_capacity(24_000))
                .collect();
        }
        self.cfg_mtime = settings::config_mtime();
        settings::print_config(&self.cfg);
    }

    fn poll_config_file(&mut self) {
        let now = settings::config_mtime();
        if now != self.cfg_mtime && now.is_some() {
            self.apply_reloaded();
        }
        let was_open = self.dialog.is_some();
        let open = self.settings_open();
        if was_open && !open {
            self.last_mouse = None;
            self.start = Instant::now();
            self.armed = false;
            self.sync_cursor();
        }
        let _ = open;
    }

    fn quit_saver(&mut self, event_loop: &ActiveEventLoop) {
        if self.cfg.wallpaper || self.settings_open() {
            return;
        }
        if self.armed {
            event_loop.exit();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let mut attrs = Window::default_attributes()
            .with_title("赛博海洋馆 / Cyber Ocean")
            .with_decorations(self.cfg.windowed)
            .with_visible(true);
        if !self.cfg.windowed {
            let monitor = event_loop
                .primary_monitor()
                .or_else(|| event_loop.available_monitors().next());
            if let Some(m) = monitor.clone() {
                attrs = attrs.with_inner_size(m.size());
            }
            attrs = attrs.with_fullscreen(Some(Fullscreen::Borderless(monitor)));
        } else {
            attrs = attrs.with_inner_size(winit::dpi::PhysicalSize::new(1280u32, 720));
        }
        let window = Arc::new(event_loop.create_window(attrs).expect("window"));
        window.set_cursor_visible(self.cfg.wallpaper || self.cfg.windowed);
        let gpu = pollster::block_on(gpu::Gpu::new(
            window.clone(),
            self.cfg.vsync,
            self.text.atlas_info(),
        ));
        self.window = Some(window.clone());
        self.gpu = Some(gpu);
        self.next_frame = Instant::now();
        window.request_redraw();
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(
            cause,
            StartCause::ResumeTimeReached { .. } | StartCause::Init
        ) {
            if let Some(w) = self.window.as_ref() {
                w.request_redraw();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key,
                        ..
                    },
                ..
            } => {
                if matches!(physical_key, PhysicalKey::Code(KeyCode::Escape | KeyCode::KeyQ)) {
                    if self.settings_open() {
                        if let Some(child) = self.dialog.as_mut() {
                            let _ = child.kill();
                        }
                        self.dialog = None;
                        self.sync_cursor();
                        return;
                    }
                    event_loop.exit();
                    return;
                }
                self.quit_saver(event_loop);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => self.open_settings(),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            }
            | WindowEvent::MouseWheel { .. } => self.quit_saver(event_loop),
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(gpu) = self.gpu.as_ref() {
                    let (w, h) = gpu.size();
                    if w > 1.0 && h > 1.0 {
                        self.pointer = (position.x / w as f64, position.y / h as f64);
                    }
                }
                if !self.cfg.wallpaper {
                    if let Some((x, y)) = self.last_mouse {
                        if (position.x - x).abs() + (position.y - y).abs() > 12.0 {
                            self.quit_saver(event_loop);
                        }
                    }
                    self.last_mouse = Some((position.x, position.y));
                }
            }
            WindowEvent::RedrawRequested => {
                self.poll_config_file();
                if !self.armed && self.start.elapsed().as_secs_f32() > 1.6 {
                    self.armed = true;
                }
                let now = Instant::now();
                let due = now >= self.next_frame;
                if due {
                    let dt = (now - self.last).as_secs_f64().min(0.05);
                    self.last = now;
                    self.next_frame = now + self.frame_interval();
                    self.ocean_t += dt;
                    let n = self.ocean.len().max(1);
                    self.highlight = ((self.ocean_t / 10.0) as usize) % n;
                    let size = self.gpu.as_ref().map(|g| g.size()).unwrap_or((1280.0, 720.0));
                    ocean::advance_morph(&mut self.ocean, dt);
                    ocean::fill_creatures(&self.ocean, self.cfg.step, &mut self.scratches);
                    ocean::integrate(
                        &mut self.ocean,
                        &self.scratches,
                        self.ocean_t,
                        dt,
                        self.pointer,
                        size,
                        !self.cfg.windowed,
                    );
                    let mul = self.cfg.point_size;
                    let min_side = size.0.min(size.1);
                    let px = (min_side / 760.0).clamp(0.85, 1.50) * mul;
                    let ocean_ps = (px * 0.58).clamp(0.78, 1.28);
                    let legend_ps = (px * 0.42).clamp(0.70, 1.05);
                    ocean::build_points(
                        &self.ocean,
                        &self.scratches,
                        self.ocean_t,
                        size,
                        ocean::PointParams {
                            highlight: self.highlight,
                            ocean_ps,
                            legend_ps,
                            legend: self.cfg.legend,
                            legend_stride: self.cfg.legend_stride,
                        },
                        &mut self.points,
                    );
                    hud::build_chrome(size, self.ocean_t, &mut self.hud);
                    if self.cfg.legend || self.cfg.formula {
                        let geom = ocean::layout_legend(size.0, size.1, self.ocean.len());
                        hud::build_hud(
                            &self.ocean,
                            geom,
                            &self.text,
                            self.highlight,
                            self.cfg.seed,
                            self.ocean_t,
                            &self.cfg.lang,
                            size,
                            self.cfg.legend,
                            self.cfg.formula,
                            &mut self.hud,
                        );
                    }
                }
                let ocean_ps = {
                    let size = self.gpu.as_ref().map(|g| g.size()).unwrap_or((1280.0, 720.0));
                    let px = (size.0.min(size.1) / 760.0).clamp(0.85, 1.50) * self.cfg.point_size;
                    (px * 0.58).clamp(0.78, 1.28)
                };
                if let Some(gpu) = self.gpu.as_mut() {
                    match gpu.render(&self.points, &self.hud, ocean_ps, self.ocean_t as f32) {
                        Ok(()) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            let s = gpu.size();
                            gpu.resize(s.0 as u32, s.1 as u32);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(_) => {}
                    }
                }
                if self.cfg.vsync {
                    event_loop.set_control_flow(ControlFlow::Poll);
                } else {
                    event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.cfg.vsync {
            event_loop.set_control_flow(ControlFlow::Poll);
            if let Some(w) = self.window.as_ref() {
                w.request_redraw();
            }
            return;
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));
        if Instant::now() >= self.next_frame {
            if let Some(w) = self.window.as_ref() {
                w.request_redraw();
            }
        }
    }
}

fn main() {
    let mut cfg = settings::parse_args();
    if cfg.setup {
        let _ = settings::run_settings_dialog_blocking();
        settings::reload_file_into(&mut cfg);
    }
    settings::print_config(&cfg);
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(cfg);
    event_loop.run_app(&mut app).expect("run");
}
