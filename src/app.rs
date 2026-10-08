//! Application state. Native effects are consumed by the shell, not the timer.
use crate::config::{Settings, Sound, Theme};
use crate::stats::StatsStore;
#[cfg(test)]
use crate::timer::Activity;
use crate::timer::{Durations, RunState, Timer};
use std::path::PathBuf;

pub fn benchmark_locked() -> bool {
    std::env::var_os("TOMITO_BENCHMARK").is_some()
        && matches!(
            std::env::var("TOMITO_BENCHMARK_SCENARIO").as_deref(),
            Ok("idle" | "running" | "statistics")
        )
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    None,
    Settings,
    Statistics,
}
pub struct App {
    pub settings: Settings,
    pub settings_path: PathBuf,
    pub stats: StatsStore,
    pub timer: Timer,
    #[cfg(feature = "egui-ui")]
    pub palette: crate::theme::Palette,
    pub panel: Panel,
    pub stats_scope_day: bool,
    pub status: String,
    pub exit_requested: bool,
    pub hide_requested: bool,
    pub show_requested: bool,
    pub notify_requested: bool,
    pub tick_requested: bool,
    #[cfg(feature = "egui-ui")]
    pub confirm_reset: bool,
    settings_dirty: bool,
    last_second: u64,
    notified: bool,
    paused_for_sleep: bool,
}
impl App {
    pub fn new(settings: Settings, settings_path: PathBuf, stats: StatsStore) -> Self {
        Self {
            #[cfg(feature = "egui-ui")]
            palette: crate::theme::palette(settings.theme),
            timer: Timer::new(durations_from(&settings)),
            hide_requested: settings.hide_on_launch,
            settings,
            settings_path,
            stats,
            panel: Panel::None,
            stats_scope_day: true,
            status: String::new(),
            exit_requested: false,
            show_requested: false,
            notify_requested: false,
            tick_requested: false,
            #[cfg(feature = "egui-ui")]
            confirm_reset: false,
            settings_dirty: false,
            last_second: u64::MAX,
            notified: false,
            paused_for_sleep: false,
        }
    }
    /// Abort an isolated measurement rather than silently benchmarking a state changed by input.
    pub fn verify_benchmark_scenario(&self) {
        if std::env::var_os("TOMITO_BENCHMARK").is_none() {
            return;
        }
        let Ok(scenario) = std::env::var("TOMITO_BENCHMARK_SCENARIO") else {
            return;
        };
        let (expected, panel) = match scenario.as_str() {
            "running" => (RunState::Running, Panel::None),
            "idle" => (RunState::Idle, Panel::None),
            "statistics" => (RunState::Idle, Panel::Statistics),
            _ => return,
        };
        assert_eq!(
            self.timer.state(),
            expected,
            "benchmark timer state changed by external input"
        );
        assert!(
            self.panel == panel,
            "benchmark panel changed by external input"
        );
    }
    pub fn apply_settings(&mut self) {
        self.timer.apply_settings(durations_from(&self.settings));
        #[cfg(feature = "egui-ui")]
        {
            self.palette = crate::theme::palette(self.settings.theme);
        }
        self.settings_dirty = true;
    }
    pub fn set_theme(&mut self, theme: Theme) {
        self.settings.theme = theme;
        self.apply_settings();
    }
    pub fn set_sound(&mut self, sound: Sound) {
        self.settings.notification_sound = sound;
        self.settings_dirty = true;
    }
    pub fn save_settings(&mut self) {
        match self.settings.save(&self.settings_path) {
            Ok(()) => self.settings_dirty = false,
            Err(error) => self.status = format!("Settings save failed: {error}"),
        }
    }
    pub fn toggle(&mut self) {
        self.timer.toggle();
        self.paused_for_sleep = false;
        if self.settings.hide_on_start && self.timer.state() == RunState::Running {
            self.hide_requested = true;
        }
    }
    pub fn stop(&mut self) {
        self.timer.stop();
        self.notified = false;
        self.paused_for_sleep = false;
    }
    pub fn restart(&mut self) {
        self.timer.restart();
        self.notified = false;
    }
    fn record(&mut self, completed: bool) {
        let elapsed = self.timer.elapsed().as_secs();
        let seconds = if completed {
            elapsed.max(self.timer.duration().as_secs())
        } else {
            elapsed
        };
        if let Err(error) = self.stats.record(self.timer.activity(), seconds, completed) {
            self.status = format!("Activity save failed: {error}");
        }
    }
    pub fn skip(&mut self) {
        self.record(false);
        self.timer.skip();
        self.notified = false;
        self.paused_for_sleep = false;
    }
    pub fn complete_current(&mut self) {
        let finished = self.timer.activity();
        self.status = format!("{} finished", finished.label());
        self.record(true);
        self.timer.finish();
        self.notified = false;
        self.paused_for_sleep = false;
        let auto = if self.timer.activity().is_break() {
            self.settings.auto_start_break
        } else {
            self.settings.auto_start_session
        };
        if auto {
            self.timer.start();
        }
    }
    pub fn tick(&mut self) {
        if self.timer.state() != RunState::Running {
            return;
        }
        if self.timer.remaining().is_zero() {
            if !self.notified {
                self.notified = true;
                self.notify_requested = true;
                if self.settings.show_on_finish {
                    self.show_requested = true;
                }
            }
            if !self.settings.overtime {
                self.complete_current();
            }
        }
        let second = self.display_seconds();
        if second != self.last_second {
            self.last_second = second;
            self.tick_requested = self.settings.ticking_sound
                && !(self.settings.mute_ticking_on_break && self.timer.activity().is_break());
        }
    }
    pub fn display_seconds(&self) -> u64 {
        if self.is_overtime() {
            self.timer
                .elapsed()
                .saturating_sub(self.timer.duration())
                .as_secs()
        } else {
            self.timer.remaining_display().as_secs()
        }
    }
    pub fn is_overtime(&self) -> bool {
        self.settings.overtime
            && self.timer.remaining().is_zero()
            && self.timer.state() != RunState::Idle
    }
    #[cfg(feature = "egui-ui")]
    pub fn schedule_repaint(&self, ctx: &egui::Context) {
        if self.timer.state() == RunState::Running {
            ctx.request_repaint_after(self.timer.next_display_change());
        }
    }
    pub fn sleep(&mut self) {
        if self.settings.pause_on_sleep && self.timer.state() == RunState::Running {
            self.timer.pause();
            self.paused_for_sleep = true;
        }
    }
    pub fn wake(&mut self) {
        if self.paused_for_sleep && self.settings.resume_on_wake {
            self.timer.start();
        }
        self.paused_for_sleep = false;
    }
    #[cfg(feature = "egui-ui")]
    pub fn handle_keys(&mut self, ctx: &egui::Context) {
        let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        if escape {
            self.panel = Panel::None;
            self.confirm_reset = false;
        }
        if ctx.egui_wants_keyboard_input() || self.panel != Panel::None {
            return;
        }
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.toggle();
            }
            if i.key_pressed(egui::Key::R) {
                self.restart();
            }
            if i.key_pressed(egui::Key::S) {
                self.skip();
            }
            if i.key_pressed(egui::Key::X) {
                self.stop();
            }
            if i.key_pressed(egui::Key::F) && self.is_overtime() {
                self.complete_current();
            }
        });
    }
    pub fn should_save(&self) -> bool {
        self.settings_dirty
    }
}
pub fn durations_from(settings: &Settings) -> Durations {
    Durations {
        session: std::time::Duration::from_secs(u64::from(settings.session_minutes) * 60),
        short_break: std::time::Duration::from_secs(u64::from(settings.short_break_minutes) * 60),
        long_break: std::time::Duration::from_secs(u64::from(settings.long_break_minutes) * 60),
        long_break_cycle: settings.long_break_cycle,
        disable_breaks: settings.disable_breaks,
        disable_long_breaks: settings.disable_long_breaks,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    fn app() -> App {
        let dir = std::env::temp_dir().join(format!(
            "tomito-app-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        App::new(
            Settings::default(),
            dir.join("settings.conf"),
            StatsStore::new(dir.join("stats.csv")),
        )
    }
    #[test]
    fn completion_records_and_auto_starts_break() {
        let mut app = app();
        app.complete_current();
        assert_eq!(app.timer.activity(), Activity::ShortBreak);
        assert_eq!(app.timer.state(), RunState::Running);
        assert_eq!(
            app.stats
                .totals_for_day(crate::stats::today_index(), true)
                .unwrap()
                .sessions,
            1
        );
        std::fs::remove_dir_all(app.settings_path.parent().unwrap()).unwrap();
    }
    #[test]
    fn skip_does_not_count_completion_or_long_break() {
        let mut app = app();
        app.skip();
        assert_eq!(app.timer.activity(), Activity::ShortBreak);
        assert_eq!(
            app.stats
                .totals_for_day(crate::stats::today_index(), false)
                .unwrap()
                .skipped,
            1
        );
        assert_eq!(
            app.stats
                .totals_for_day(crate::stats::today_index(), true)
                .unwrap()
                .skipped,
            0
        );
        std::fs::remove_dir_all(app.settings_path.parent().unwrap()).unwrap();
    }
    #[test]
    fn sleep_only_resumes_previously_running_timer() {
        let mut app = app();
        app.sleep();
        app.wake();
        assert_eq!(app.timer.state(), RunState::Idle);
        app.toggle();
        app.sleep();
        assert_eq!(app.timer.state(), RunState::Paused);
        app.wake();
        assert_eq!(app.timer.state(), RunState::Running);
        app.toggle();
        app.sleep();
        app.wake();
        assert_eq!(app.timer.state(), RunState::Paused);
    }
    #[test]
    fn settings_dirty_and_hide_request() {
        let mut app = app();
        app.set_theme(Theme::Mint);
        assert!(app.should_save());
        app.settings.hide_on_start = true;
        app.toggle();
        assert!(app.hide_requested);
        app.settings.auto_start_break = false;
        app.complete_current();
        assert_eq!(app.timer.state(), RunState::Idle);
        std::fs::remove_dir_all(app.settings_path.parent().unwrap()).unwrap();
    }
    #[test]
    fn expiration_and_overtime_notify_once() {
        let mut app = app();
        app.timer = Timer::new(Durations {
            session: std::time::Duration::ZERO,
            ..Default::default()
        });
        app.settings.overtime = true;
        app.timer.start();
        app.tick();
        assert!(app.notify_requested && app.show_requested && app.is_overtime());
        app.notify_requested = false;
        app.tick();
        assert!(!app.notify_requested);
        assert_eq!(app.timer.activity(), Activity::Session);
        app.complete_current();
        assert_eq!(app.timer.activity(), Activity::ShortBreak);
        std::fs::remove_dir_all(app.settings_path.parent().unwrap()).unwrap();
    }
}
