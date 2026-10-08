use crate::{
  app::{App, Panel},
  config::{self, Settings, Sound, Theme},
  stats::{self, StatsStore},
  timer::RunState,
};
use slint::winit_030::WinitWindowAccessor;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc, time::Duration};

slint::slint! { export { PortableWindow, SwitchRow, DayRow } from "src/slint_ui.slint"; }

pub struct Shell {
  pub app: App,
  pub ui: slint::Weak<PortableWindow>,
  alarm: slint::Timer,
  switches: Rc<VecModel<SwitchRow>>,
  pub native: Option<crate::slint_native::Native>,
}
type State = Rc<RefCell<Shell>>;
const OPTIONS: [&str; 14] = [
  "Start sessions automatically",
  "Start breaks automatically",
  "Disable all breaks",
  "Disable long breaks",
  "Show completed activities only",
  "Hide when timer starts",
  "Keep timer in front",
  "Hide on launch",
  "Show when timer ends",
  "Continue past zero until Finish",
  "Pause when Mac sleeps",
  "Resume when Mac wakes",
  "Tick once per second",
  "Mute ticking during breaks",
];
fn flags(s: Settings) -> [bool; 14] {
  [
    s.auto_start_session,
    s.auto_start_break,
    s.disable_breaks,
    s.disable_long_breaks,
    s.show_completed_only,
    s.hide_on_start,
    s.keep_in_front,
    s.hide_on_launch,
    s.show_on_finish,
    s.overtime,
    s.pause_on_sleep,
    s.resume_on_wake,
    s.ticking_sound,
    s.mute_ticking_on_break,
  ]
}
fn apply_flags(s: &mut Settings, values: [bool; 14]) {
  [
    s.auto_start_session,
    s.auto_start_break,
    s.disable_breaks,
    s.disable_long_breaks,
    s.show_completed_only,
    s.hide_on_start,
    s.keep_in_front,
    s.hide_on_launch,
    s.show_on_finish,
    s.overtime,
    s.pause_on_sleep,
    s.resume_on_wake,
    s.ticking_sound,
    s.mute_ticking_on_break,
  ] = values;
}
fn accent(theme: Theme) -> slint::Color {
  let rgb = match theme {
    Theme::Flamingo => (232, 74, 95),
    Theme::Mint => (63, 193, 160),
    Theme::Mandarin => (242, 140, 40),
    Theme::Monochrome => (216, 216, 216),
    Theme::Pastel => (168, 198, 232),
  };
  slint::Color::from_rgb_u8(rgb.0, rgb.1, rgb.2)
}
fn number(text: &str, min: u32, max: u32) -> Result<u32, &'static str> {
  text
    .trim()
    .parse::<u32>()
    .map(|n| n.clamp(min, max))
    .map_err(|_| "Enter whole numbers for durations and cycle")
}
fn fill_settings(shell: &Shell, ui: &PortableWindow) {
  let s = shell.app.settings;
  ui.set_session_input(s.session_minutes.to_string().into());
  ui.set_short_input(s.short_break_minutes.to_string().into());
  ui.set_long_input(s.long_break_minutes.to_string().into());
  ui.set_cycle_input(s.long_break_cycle.to_string().into());
  for (i, checked) in flags(s).into_iter().enumerate() {
    shell.switches.set_row_data(
      i,
      SwitchRow {
        label: OPTIONS[i].into(),
        checked,
      },
    );
  }
  ui.set_theme_index(Theme::ALL.iter().position(|t| *t == s.theme).unwrap() as i32);
  ui.set_theme_label(s.theme.label().into());
  ui.set_sound_index(
    Sound::ALL
      .iter()
      .position(|t| *t == s.notification_sound)
      .unwrap() as i32,
  );
  ui.set_sound_label(s.notification_sound.label().into());
}
fn statistics(shell: &Shell, ui: &PortableWindow) {
  let day = stats::today_index();
  let only = shell.app.settings.show_completed_only;
  let total = if shell.app.stats_scope_day {
    shell.app.stats.totals_for_day(day, only)
  } else {
    shell.app.stats.totals_for_week(day, only)
  };
  match total {
    Ok(t) => ui.set_summary_text(
      format!(
        "{}: {} sessions / {} breaks\nFocus {} / Breaks {}\nLong breaks {} / Skipped {}",
        if shell.app.stats_scope_day {
          "Today"
        } else {
          "Last 7 days"
        },
        t.sessions,
        t.breaks,
        stats::format_duration(t.session_seconds),
        stats::format_duration(t.break_seconds),
        t.long_breaks,
        t.skipped
      )
      .into(),
    ),
    Err(e) => ui.set_summary_text(format!("Statistics read failed: {e}").into()),
  }
  match shell.app.stats.daily_totals(day, only) {
    Ok(series) => {
      let max = series
        .iter()
        .map(|(_, t)| t.session_seconds)
        .max()
        .unwrap_or(1)
        .max(1);
      let data = series
        .into_iter()
        .map(|(date, t)| DayRow {
          label: format!(
            "{} / {}",
            stats::format_date(date),
            stats::format_duration(t.session_seconds)
          )
          .into(),
          fraction: t.session_seconds as f32 / max as f32,
        })
        .collect::<Vec<_>>();
      ui.set_days(ModelRc::new(VecModel::from(data)));
    }
    Err(e) => ui.set_status_text(format!("Statistics read failed: {e}").into()),
  }
}
pub fn refresh(state: &State) {
  let weak = Rc::downgrade(state);
  let mut shell = state.borrow_mut();
  let Some(ui) = shell.ui.upgrade() else { return };
  shell.alarm.stop();
  let app = &mut shell.app;
  app.verify_benchmark_scenario();
  let seconds = app.display_seconds();
  ui.set_time_text(
    format!(
      "{}{:02}:{:02}",
      if app.is_overtime() { "+" } else { "" },
      seconds / 60,
      seconds % 60
    )
    .into(),
  );
  ui.set_activity_text(app.timer.activity().label().into());
  ui.set_counter_text(
    format!(
      "Session {} of {} in cycle",
      app.timer.completed_sessions() % app.settings.long_break_cycle + 1,
      app.settings.long_break_cycle
    )
    .into(),
  );
  ui.set_state_text(
    match app.timer.state() {
      RunState::Idle => "Ready",
      RunState::Running => "Running",
      RunState::Paused => "Paused",
    }
    .into(),
  );
  ui.set_action_label(
    match app.timer.state() {
      RunState::Idle => "Start",
      RunState::Running => "Pause",
      RunState::Paused => "Resume",
    }
    .into(),
  );
  ui.set_running(app.timer.state() == RunState::Running);
  ui.set_overtime(app.is_overtime());
  ui.set_progress(app.timer.progress());
  ui.set_accent(accent(app.settings.theme));
  ui.set_status_text(app.status.clone().into());
  let hide = std::mem::take(&mut app.hide_requested);
  let show = std::mem::take(&mut app.show_requested);
  let notify = std::mem::take(&mut app.notify_requested);
  let tick = std::mem::take(&mut app.tick_requested);
  let sound = app.settings.notification_sound;
  let front = app.settings.keep_in_front;
  let running = app.timer.state() == RunState::Running;
  let delay = app.timer.next_display_change();
  if app.should_save() {
    app.save_settings();
    ui.set_status_text(app.status.clone().into());
  }
  if hide && cfg!(target_os = "macos") {
    let _ = ui.hide();
  }
  if show {
    let _ = ui.show();
    ui.window().set_minimized(false);
    if std::env::var_os("TOMITO_BENCHMARK").is_none() {
      crate::slint_native::activate();
      ui.window().with_winit_window(|w| w.focus_window());
    }
  }
  ui.window().with_winit_window(|w| {
    if crate::app::benchmark_locked() {
      w.set_cursor_hittest(false)
        .expect("benchmark mouse input isolation");
    }
    w.set_window_level(if front {
      slint::winit_030::winit::window::WindowLevel::AlwaysOnTop
    } else {
      slint::winit_030::winit::window::WindowLevel::Normal
    })
  });
  if notify {
    crate::slint_native::play(sound);
  }
  if tick {
    crate::slint_native::play(Sound::Tink);
  }
  if let Some(native) = &shell.native {
    native.update(&shell.app);
  }
  if ui.get_page() == 2 {
    statistics(&shell, &ui);
  }
  if running {
    shell.alarm.start(
      slint::TimerMode::SingleShot,
      delay.max(Duration::from_millis(10)),
      move || {
        if let Some(state) = weak.upgrade() {
          state.borrow_mut().app.tick();
          refresh(&state);
        }
      },
    );
  }
}
pub fn dispatch(state: &State, action: i32) {
  if crate::app::benchmark_locked() && !matches!(action, 21 | 22) {
    return;
  }
  {
    let mut shell = state.borrow_mut();
    let Some(ui) = shell.ui.upgrade() else { return };
    match action {
      0 => shell.app.toggle(),
      1 => shell.app.restart(),
      2 => shell.app.skip(),
      3 => shell.app.stop(),
      4 => {
        if shell.app.is_overtime() {
          shell.app.complete_current();
        }
      }
      5 => {
        shell.app.panel = Panel::None;
        ui.set_page(0);
        ui.set_confirm_reset(false);
      }
      6 => {
        shell.app.panel = Panel::Settings;
        fill_settings(&shell, &ui);
        ui.set_page(1);
      }
      7 => {
        shell.app.panel = Panel::Statistics;
        ui.set_page(2);
      }
      8 => shell.app.timer.reset_counter(),
      9 => {
        shell.app.exit_requested = true;
        shell.app.save_settings();
        let _ = slint::quit_event_loop();
      }
      15 => crate::slint_native::play(Sound::ALL[ui.get_sound_index() as usize]),
      20 => {
        shell.app.show_requested = true;
      }
      21 => shell.app.sleep(),
      22 => shell.app.wake(),
      _ => {}
    }
  }
  refresh(state);
}
fn save(state: &State) {
  {
    let mut shell = state.borrow_mut();
    let Some(ui) = shell.ui.upgrade() else { return };
    let result = (|| {
      let mut s = shell.app.settings;
      s.session_minutes = number(&ui.get_session_input(), 1, 180)?;
      s.short_break_minutes = number(&ui.get_short_input(), 1, 60)?;
      s.long_break_minutes = number(&ui.get_long_input(), 1, 120)?;
      s.long_break_cycle = number(&ui.get_cycle_input(), 2, 10)?;
      apply_flags(
        &mut s,
        std::array::from_fn(|i| shell.switches.row_data(i).unwrap().checked),
      );
      s.theme = Theme::ALL[ui.get_theme_index() as usize];
      s.notification_sound = Sound::ALL[ui.get_sound_index() as usize];
      Ok::<_, &str>(s)
    })();
    match result {
      Ok(s) => {
        shell.app.settings = s;
        shell.app.set_theme(s.theme);
        shell.app.set_sound(s.notification_sound);
        shell.app.status = "Settings saved".into();
        shell.app.save_settings();
        fill_settings(&shell, &ui);
      }
      Err(e) => shell.app.status = e.into(),
    }
  }
  refresh(state);
}
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
  // Both values specified: SLINT_BACKEND cannot silently substitute a GPU renderer.
  let benchmark = std::env::var_os("TOMITO_BENCHMARK").is_some();
  let smoke = std::env::args().any(|a| a == "--smoke" || a == "--smoke-expiry");
  slint::BackendSelector::new()
    .backend_name("winit".into())
    .renderer_name("software".into())
    .with_winit_window_attributes_hook(move |attributes| attributes.with_active(!benchmark))
    .select()?;
  let ui = PortableWindow::new()?;
  ui.set_native_features(cfg!(target_os = "macos"));
  let dir = std::env::var_os("TOMITO_CONFIG_DIR")
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|| config::config_dir("tomito-rs"));
  if smoke
    && (std::env::var_os("TOMITO_CONFIG_DIR").is_none()
      || (dir.exists() && std::fs::read_dir(&dir)?.next().is_some()))
  {
    return Err("Smoke tests require an explicitly set, empty TOMITO_CONFIG_DIR".into());
  }
  let app = App::new(
    Settings::load(&dir.join("settings.conf")),
    dir.join("settings.conf"),
    StatsStore::new(dir.join("activities.csv")),
  );
  let values = flags(app.settings);
  let switches = Rc::new(VecModel::from(
    OPTIONS
      .iter()
      .enumerate()
      .map(|(i, label)| SwitchRow {
        label: (*label).into(),
        checked: values[i],
      })
      .collect::<Vec<_>>(),
  ));
  ui.set_switches(switches.clone().into());
  let state = Rc::new(RefCell::new(Shell {
    app,
    ui: ui.as_weak(),
    alarm: slint::Timer::default(),
    switches,
    native: None,
  }));
  {
    let shell = state.borrow();
    fill_settings(&shell, &ui);
  }
  let weak = Rc::downgrade(&state);
  ui.on_action(move |id| {
    if let Some(s) = weak.upgrade() {
      dispatch(&s, id);
    }
  });
  let weak = Rc::downgrade(&state);
  ui.on_save_settings(move || {
    if let Some(s) = weak.upgrade() {
      save(&s);
    }
  });
  let weak = Rc::downgrade(&state);
  ui.on_option_toggled(move |index| {
    if let Some(s) = weak.upgrade() {
      let shell = s.borrow();
      if let Some(mut row) = shell.switches.row_data(index as usize) {
        row.checked = !row.checked;
        shell.switches.set_row_data(index as usize, row);
      }
    }
  });
  let weak = ui.as_weak();
  ui.on_next_theme(move || {
    if let Some(u) = weak.upgrade() {
      let i = (u.get_theme_index() as usize + 1) % Theme::ALL.len();
      u.set_theme_index(i as i32);
      u.set_theme_label(Theme::ALL[i].label().into());
    }
  });
  let weak = ui.as_weak();
  ui.on_next_sound(move || {
    if let Some(u) = weak.upgrade() {
      let i = (u.get_sound_index() as usize + 1) % Sound::ALL.len();
      u.set_sound_index(i as i32);
      u.set_sound_label(Sound::ALL[i].label().into());
    }
  });
  let weak = Rc::downgrade(&state);
  ui.on_select_scope(move |day| {
    if let Some(s) = weak.upgrade() {
      s.borrow_mut().app.stats_scope_day = day;
      refresh(&s);
    }
  });
  let weak = Rc::downgrade(&state);
  ui.on_export_history(move || {
    if let Some(s) = weak.upgrade() {
      {
        let mut shell = s.borrow_mut();
        let target = shell.app.stats.path().with_file_name("tomito-stats.csv");
        shell.app.status = match shell.app.stats.export_csv(&target) {
          Ok(()) => format!("Exported to {}", target.display()),
          Err(e) => format!("Export failed: {e}"),
        };
      }
      refresh(&s);
    }
  });
  let weak = Rc::downgrade(&state);
  ui.on_reset_history(move || {
    if let Some(s) = weak.upgrade() {
      {
        let mut shell = s.borrow_mut();
        shell.app.status = match shell.app.stats.reset() {
          Ok(()) => "Statistics cleared".into(),
          Err(e) => format!("Reset failed: {e}"),
        };
        if let Some(u) = shell.ui.upgrade() {
          u.set_confirm_reset(false);
        }
      }
      refresh(&s);
    }
  });
  let weak = Rc::downgrade(&state);
  state.borrow_mut().native = Some(crate::slint_native::Native::new(move |command| {
    if let Some(s) = weak.upgrade() {
      dispatch(&s, command);
    }
  }));
  let weak = Rc::downgrade(&state);
  ui.window().on_close_requested(move || {
    if crate::app::benchmark_locked() {
      return slint::CloseRequestResponse::KeepWindowShown;
    }
    #[cfg(target_os = "macos")]
    {
      if let Some(s) = weak.upgrade() {
        if let Some(u) = s.borrow().ui.upgrade() {
          let _ = u.hide();
        }
      }
      slint::CloseRequestResponse::KeepWindowShown
    }
    #[cfg(not(target_os = "macos"))]
    {
      if let Some(s) = weak.upgrade() {
        s.borrow_mut().app.save_settings();
      }
      slint::CloseRequestResponse::HideWindow
    }
  });
  ui.show()?;
  match std::env::var("TOMITO_BENCHMARK_SCENARIO").as_deref() {
    Ok("running") => state.borrow_mut().app.toggle(),
    Ok("statistics") => {
      state.borrow_mut().app.panel = Panel::Statistics;
      ui.set_page(2);
    }
    Ok("settings") => {
      state.borrow_mut().app.panel = Panel::Settings;
      ui.set_page(1);
    }
    _ => {}
  }
  refresh(&state);
  // Winit's native window is created when the event loop starts; apply window level/input policy then.
  let weak = Rc::downgrade(&state);
  slint::Timer::single_shot(Duration::from_millis(100), move || {
    if let Some(s) = weak.upgrade() {
      {
        if let Some(native) = &s.borrow().native {
          native.install_dock_handler();
        }
      }
      refresh(&s);
    }
  });
  if std::env::args().any(|a| a == "--smoke") {
    start_smoke(state.clone());
  }
  if std::env::args().any(|a| a == "--smoke-expiry") {
    start_expiry_smoke(state.clone());
  }
  #[cfg(target_os = "macos")]
  slint::run_event_loop_until_quit()?;
  #[cfg(not(target_os = "macos"))]
  slint::run_event_loop()?;
  state.borrow_mut().app.save_settings();
  Ok(())
}
fn start_smoke(state: State) {
  slint::Timer::single_shot(Duration::from_millis(400), move || {
    let ui = state.borrow().ui.upgrade().unwrap();
    ui.window()
      .dispatch_event(slint::platform::WindowEvent::KeyPressed { text: " ".into() });
    ui.window()
      .dispatch_event(slint::platform::WindowEvent::KeyReleased { text: " ".into() });
    assert_eq!(
      state.borrow().app.timer.state(),
      RunState::Running,
      "keyboard start"
    );
    dispatch(&state, 0);
    assert_eq!(state.borrow().app.timer.state(), RunState::Paused);
    dispatch(&state, 1);
    assert_eq!(ui.get_time_text(), "25:00");
    dispatch(&state, 2);
    assert_eq!(ui.get_activity_text(), "Short Break");
    dispatch(&state, 3);
    assert_eq!(ui.get_time_text(), "05:00");
    ui.window()
      .dispatch_event(slint::platform::WindowEvent::PointerPressed {
        position: slint::LogicalPosition::new(190.0, 34.0),
        button: slint::platform::PointerEventButton::Left,
      });
    ui.window()
      .dispatch_event(slint::platform::WindowEvent::PointerReleased {
        position: slint::LogicalPosition::new(190.0, 34.0),
        button: slint::platform::PointerEventButton::Left,
      });
    assert_eq!(ui.get_page(), 1, "pointer opens Settings");
    ui.set_session_input("9999".into());
    ui.invoke_save_settings();
    assert_eq!(state.borrow().app.settings.session_minutes, 180);
    assert_eq!(ui.get_session_input(), "180");
    ui.invoke_next_theme();
    ui.invoke_next_sound();
    ui.invoke_option_toggled(6);
    ui.invoke_option_toggled(4); // Include the incomplete activity recorded by Skip.
    ui.invoke_save_settings();
    assert_eq!(state.borrow().app.settings.theme, Theme::Mint);
    assert!(state.borrow().app.settings.keep_in_front);
    let persisted = Settings::load(&state.borrow().app.settings_path);
    assert_eq!(persisted, state.borrow().app.settings);
    dispatch(&state, 7);
    assert!(ui.get_summary_text().contains("Skipped 1"));
    ui.invoke_export_history();
    assert!(state
      .borrow()
      .app
      .stats
      .path()
      .with_file_name("tomito-stats.csv")
      .exists());
    ui.set_confirm_reset(true);
    ui.invoke_reset_history();
    assert!(!ui.get_confirm_reset());
    assert!(ui.get_summary_text().contains("Skipped 0"));
    dispatch(&state, 5);
    #[cfg(target_os = "macos")]
    {
      ui.window()
        .dispatch_event(slint::platform::WindowEvent::CloseRequested);
      assert!(!ui.window().is_visible());
      dispatch(&state, 20);
      assert!(ui.window().is_visible());
    }
    println!("PASS Slint real window keyboard, timer actions, settings clamp/save/reload, theme/sound/toggle, statistics/export/reset, close/restore");
    dispatch(&state, 9);
  });
}
fn start_expiry_smoke(state: State) {
  slint::Timer::single_shot(Duration::from_millis(400), move || {
    {
      let mut shell = state.borrow_mut();
      shell.app.settings.session_minutes = 1;
      shell.app.settings.short_break_minutes = 1;
      shell.app.settings.auto_start_break = false;
      shell.app.settings.auto_start_session = false;
      shell.app.settings.hide_on_start = true;
      shell.app.settings.show_on_finish = true;
      shell.app.settings.notification_sound = Sound::None;
      shell.app.apply_settings();
    }
    dispatch(&state, 0);
    #[cfg(target_os = "macos")]
    assert!(!state.borrow().ui.upgrade().unwrap().window().is_visible());
    slint::Timer::single_shot(Duration::from_secs(62), move || {
      {
        let shell = state.borrow();
        let ui = shell.ui.upgrade().unwrap();
        assert_eq!(
          shell.app.timer.activity(),
          crate::timer::Activity::ShortBreak
        );
        assert_eq!(shell.app.timer.state(), RunState::Idle);
        assert_eq!(ui.get_time_text(), "01:00");
        assert!(
          ui.window().is_visible(),
          "timer completion restores hidden window"
        );
        let totals = shell
          .app
          .stats
          .totals_for_day(stats::today_index(), true)
          .unwrap();
        assert_eq!(totals.sessions, 1);
        assert_eq!(totals.session_seconds, 60);
      }
      println!("PASS Slint real 60-second hidden timer expiry, restored window, short break, activity recorded");
      dispatch(&state, 9);
    });
  });
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn settings_flags_round_trip() {
    let s = Settings::default();
    let mut copy = Settings {
      hide_on_start: true,
      ..s
    };
    apply_flags(&mut copy, flags(s));
    assert_eq!(copy, s);
  }
  #[test]
  fn numbers_validate_and_clamp() {
    assert_eq!(number("9999", 1, 180), Ok(180));
    assert_eq!(number("0", 2, 10), Ok(2));
    assert!(number("wrong", 1, 60).is_err());
  }
}
