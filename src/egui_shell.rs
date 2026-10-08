//! Small Rust Pomodoro shell: event-driven UI and native menu bar.
use crate::app::{App, Panel};
use crate::config::{Settings, Sound};
use crate::native::{Command, Native};
use crate::stats::StatsStore;
use crate::{config, native, theme, ui};
use std::time::{Duration, Instant};
const APP_NAME: &str = "tomito-rs";

pub fn run() -> eframe::Result {
  let dir = config::var_os("RUSTY_POMODORO_CONFIG_DIR")
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|| config::config_dir(APP_NAME));
  let settings = Settings::load(&dir.join("settings.conf"));
  let store = StatsStore::new(dir.join("activities.csv"));
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
      .with_title("Rusty Pomodoro")
      .with_active(config::var_os("RUSTY_POMODORO_BENCHMARK").is_none())
      .with_inner_size([400.0, 460.0])
      .with_min_inner_size([360.0, 440.0]),
    multisampling: 0,
    depth_buffer: 0,
    stencil_buffer: 0,
    renderer: eframe::Renderer::Glow,
    centered: true,
    persist_window: false,
    dithering: false,
    ..Default::default()
  };
  eframe::run_native(
    APP_NAME,
    options,
    Box::new(move |cc| {
      let ctx = cc.egui_ctx.clone();
      ctx.options_mut(|options| {
        options.max_passes = std::num::NonZeroUsize::new(1).expect("one pass");
        options.reduce_texture_memory = true;
        options.screen_reader = false;
        options.warn_on_id_clash = false;
        options.zoom_with_keyboard = false;
      });
      // A single 302 KiB licensed font replaces four default fonts and emoji fallbacks.
      let mut fonts = egui::FontDefinitions::empty();
      fonts.font_data.insert(
        "Hack".into(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
          "../assets/Hack-Regular.ttf"
        ))),
      );
      for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts.families.insert(family, vec!["Hack".into()]);
      }
      ctx.set_fonts(fonts);
      theme::apply(&ctx, theme::palette(settings.theme));
      let mut app = App::new(settings, dir.join("settings.conf"), store);
      match config::var("RUSTY_POMODORO_BENCHMARK_SCENARIO").as_deref() {
        Ok("running") => app.toggle(),
        Ok("statistics") => app.panel = Panel::Statistics,
        _ => {}
      }
      Ok(Box::new(RustyPomodoroApp {
        app,
        native: Native::new(ctx),
        last_save: Instant::now(),
        applied_theme: settings.theme,
        applied_front: None,
      }))
    }),
  )
}
struct RustyPomodoroApp {
  app: App,
  native: Native,
  last_save: Instant,
  applied_theme: config::Theme,
  applied_front: Option<bool>,
}
impl RustyPomodoroApp {
  fn effects(&mut self, ctx: &egui::Context) {
    if self.applied_theme != self.app.settings.theme {
      theme::apply(ctx, self.app.palette);
      self.applied_theme = self.app.settings.theme;
    }
    if self.applied_front != Some(self.app.settings.keep_in_front) {
      ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
        if self.app.settings.keep_in_front {
          egui::WindowLevel::AlwaysOnTop
        } else {
          egui::WindowLevel::Normal
        },
      ));
      self.applied_front = Some(self.app.settings.keep_in_front);
    }
    #[cfg(target_os = "macos")]
    {
      if std::mem::take(&mut self.app.hide_requested) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
      }
      if std::mem::take(&mut self.app.show_requested) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        self.native.activate();
      }
    }
    if std::mem::take(&mut self.app.notify_requested) {
      native::play(self.app.settings.notification_sound);
    }
    if std::mem::take(&mut self.app.tick_requested) {
      native::play(Sound::Tink);
    }
    self.native.update(&self.app);
    self.app.schedule_repaint(ctx);
    if self.app.should_save() {
      let wait = Duration::from_secs(2).saturating_sub(self.last_save.elapsed());
      if wait.is_zero() {
        self.app.save_settings();
        self.last_save = Instant::now();
      } else {
        ctx.request_repaint_after(wait);
      }
    }
    if self.app.exit_requested {
      ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
  }
}
impl eframe::App for RustyPomodoroApp {
  fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
    for command in self.native.drain() {
      if crate::app::benchmark_locked() && !matches!(command, Command::Sleep | Command::Wake) {
        continue;
      }
      match command {
        Command::Toggle => self.app.toggle(),
        Command::Restart => self.app.restart(),
        Command::Skip => self.app.skip(),
        Command::Stop => self.app.stop(),
        Command::Finish => {
          if self.app.is_overtime() {
            self.app.complete_current();
          }
        }
        Command::Show => self.app.show_requested = true,
        Command::Settings => {
          self.app.show_requested = true;
          self.app.panel = Panel::Settings;
        }
        Command::Statistics => {
          self.app.show_requested = true;
          self.app.panel = Panel::Statistics;
        }
        Command::ResetCounter => self.app.timer.reset_counter(),
        Command::Quit => self.app.exit_requested = true,
        Command::Sleep => self.app.sleep(),
        Command::Wake => self.app.wake(),
      }
    }
    #[cfg(target_os = "macos")]
    if ctx.input(|i| i.viewport().close_requested()) && !self.app.exit_requested {
      ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
      self.app.hide_requested = true;
    }
    self.app.tick();
    self.effects(ctx);
  }
  fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
    let ctx = root.ctx().clone();
    if !crate::app::benchmark_locked() {
      self.app.handle_keys(&ctx);
    }
    ui::main::draw(&mut self.app, root);
    ui::settings::draw(&mut self.app, &ctx);
    ui::statistics::draw(&mut self.app, &ctx);
    // Schedule after input too, so starting an idle timer always gets its next tick.
    self.app.verify_benchmark_scenario();
    self.effects(&ctx);
  }
  fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
    self.app.save_settings();
  }
}
