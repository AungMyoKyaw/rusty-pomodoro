//! Pure Pomodoro state machine with a monotonic clock.
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
struct Clock {
  #[cfg(target_os = "macos")]
  ticks: u64,
  #[cfg(not(target_os = "macos"))]
  instant: std::time::Instant,
}
impl Clock {
  fn now() -> Self {
    #[cfg(target_os = "macos")]
    // SAFETY: mach_continuous_time has no arguments or preconditions and includes suspend.
    {
      Self {
        ticks: unsafe { mach_continuous_time() },
      }
    }
    #[cfg(not(target_os = "macos"))]
    {
      Self {
        instant: std::time::Instant::now(),
      }
    }
  }
  fn elapsed(self) -> Duration {
    #[cfg(target_os = "macos")]
    {
      static SCALE: std::sync::OnceLock<(u32, u32)> = std::sync::OnceLock::new();
      let &(numer, denom) = SCALE.get_or_init(|| {
        let mut info = TimebaseInfo { numer: 0, denom: 0 };
        // SAFETY: info is writable stack storage of the declared type.
        let code = unsafe { mach_timebase_info(&mut info) };
        assert_eq!(code, 0, "mach_timebase_info");
        (info.numer, info.denom)
      });
      let nanos = u128::from(Clock::now().ticks.saturating_sub(self.ticks)) * u128::from(numer)
        / u128::from(denom);
      Duration::from_nanos(nanos.min(u128::from(u64::MAX)) as u64)
    }
    #[cfg(not(target_os = "macos"))]
    {
      self.instant.elapsed()
    }
  }
}
#[cfg(target_os = "macos")]
#[repr(C)]
struct TimebaseInfo {
  numer: u32,
  denom: u32,
}
#[cfg(target_os = "macos")]
unsafe extern "C" {
  fn mach_continuous_time() -> u64;
  fn mach_timebase_info(info: *mut TimebaseInfo) -> i32;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Activity {
  Session,
  ShortBreak,
  LongBreak,
}

impl Activity {
  pub fn is_break(self) -> bool {
    matches!(self, Activity::ShortBreak | Activity::LongBreak)
  }

  pub fn label(self) -> &'static str {
    match self {
      Activity::Session => "Session",
      Activity::ShortBreak => "Short Break",
      Activity::LongBreak => "Long Break",
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RunState {
  Idle,
  Running,
  Paused,
}

/// Result of advancing the state machine after an activity finished.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Completion {
  /// The activity ran to the end.
  Finished,
  /// The user skipped the activity.
  Skipped,
}

#[derive(Debug)]
pub struct Timer {
  activity: Activity,
  state: RunState,
  duration: Duration,
  elapsed: Duration,
  started_at: Option<Clock>,
  completed_sessions: u32,
  settings: Durations,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Durations {
  pub session: Duration,
  pub short_break: Duration,
  pub long_break: Duration,
  pub long_break_cycle: u32,
  pub disable_breaks: bool,
  pub disable_long_breaks: bool,
}

impl Default for Durations {
  fn default() -> Self {
    Self {
      session: Duration::from_secs(25 * 60),
      short_break: Duration::from_secs(5 * 60),
      long_break: Duration::from_secs(15 * 60),
      long_break_cycle: 4,
      disable_breaks: false,
      disable_long_breaks: false,
    }
  }
}

impl Timer {
  pub fn new(settings: Durations) -> Self {
    let duration = duration_for(Activity::Session, settings);
    Self {
      activity: Activity::Session,
      state: RunState::Idle,
      duration,
      elapsed: Duration::ZERO,
      started_at: None,
      completed_sessions: 0,
      settings,
    }
  }

  pub fn activity(&self) -> Activity {
    self.activity
  }

  pub fn state(&self) -> RunState {
    self.state
  }

  pub fn duration(&self) -> Duration {
    self.duration
  }

  pub fn completed_sessions(&self) -> u32 {
    self.completed_sessions
  }

  /// Preserve elapsed time, including overtime, when settings change.
  pub fn apply_settings(&mut self, settings: Durations) {
    self.elapsed = self.elapsed();
    self.started_at = (self.state == RunState::Running).then(Clock::now);
    self.settings = settings;
    self.duration = duration_for(self.activity, settings);
  }

  pub fn elapsed(&self) -> Duration {
    match self.started_at {
      Some(start) => self.elapsed + start.elapsed(),
      None => self.elapsed,
    }
  }

  pub fn remaining(&self) -> Duration {
    self.duration.saturating_sub(self.elapsed())
  }

  /// 0.0 at the start, 1.0 when the activity is done.
  pub fn progress(&self) -> f32 {
    if self.duration.is_zero() {
      return 1.0;
    }
    (self.elapsed().as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0)
  }

  /// Remaining time rounded up to whole seconds, so a fresh timer shows 25:00 not 24:59.
  pub fn remaining_display(&self) -> Duration {
    let remaining = self.remaining();
    if remaining.is_zero() {
      return Duration::ZERO;
    }
    Duration::from_secs(remaining.as_secs() + u64::from(remaining.subsec_nanos() != 0))
  }

  pub fn reset_counter(&mut self) {
    self.completed_sessions = 0;
  }

  /// Delay until the next displayed second changes, not its complement.
  pub fn next_display_change(&self) -> Duration {
    let remaining = self.remaining();
    if remaining.is_zero() {
      return Duration::from_secs(1);
    }
    let nanos = remaining.subsec_nanos();
    if nanos == 0 {
      Duration::from_secs(1)
    } else {
      Duration::from_nanos(u64::from(nanos))
    }
  }

  pub fn start(&mut self) {
    if self.state == RunState::Idle && self.elapsed >= self.duration {
      self.elapsed = Duration::ZERO;
    }
    if self.state != RunState::Running {
      self.state = RunState::Running;
      self.started_at = Some(Clock::now());
    }
  }

  pub fn pause(&mut self) {
    if self.state == RunState::Running {
      if let Some(start) = self.started_at.take() {
        self.elapsed += start.elapsed();
      }
      self.state = RunState::Paused;
    }
  }

  pub fn toggle(&mut self) {
    match self.state {
      RunState::Running => self.pause(),
      RunState::Idle | RunState::Paused => self.start(),
    }
  }

  /// Stop the current activity and return to the start of the same activity.
  pub fn stop(&mut self) {
    self.started_at = None;
    self.elapsed = Duration::ZERO;
    self.state = RunState::Idle;
  }

  /// Restart the current activity from zero, staying in the same run state.
  pub fn restart(&mut self) {
    self.elapsed = Duration::ZERO;
    self.started_at = if self.state == RunState::Running {
      Some(Clock::now())
    } else {
      None
    };
  }

  /// Move to the next activity without running anything.
  pub fn skip(&mut self) -> Completion {
    self.advance(Completion::Skipped)
  }

  /// Move to the next activity after a completed run.
  pub fn finish(&mut self) -> Completion {
    self.advance(Completion::Finished)
  }

  fn advance(&mut self, completion: Completion) -> Completion {
    let finished = self.activity;
    if completion == Completion::Finished && finished == Activity::Session {
      self.completed_sessions = self.completed_sessions.saturating_add(1);
    }

    let next = match finished {
      Activity::Session => {
        let cycle = self.settings.long_break_cycle.max(1);
        let due_long = completion == Completion::Finished
          && self.completed_sessions > 0
          && self.completed_sessions.is_multiple_of(cycle);
        if self.settings.disable_breaks {
          Activity::Session
        } else if due_long && !self.settings.disable_long_breaks {
          Activity::LongBreak
        } else {
          Activity::ShortBreak
        }
      }
      Activity::ShortBreak | Activity::LongBreak => Activity::Session,
    };

    self.activity = next;
    self.duration = duration_for(next, self.settings);
    self.elapsed = Duration::ZERO;
    self.started_at = None;
    self.state = RunState::Idle;
    completion
  }
}

fn duration_for(activity: Activity, settings: Durations) -> Duration {
  match activity {
    Activity::Session => settings.session,
    Activity::ShortBreak => settings.short_break,
    Activity::LongBreak => settings.long_break,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_settings() -> Durations {
    Durations {
      session: Duration::from_secs(10),
      short_break: Duration::from_secs(4),
      long_break: Duration::from_secs(20),
      long_break_cycle: 4,
      disable_breaks: false,
      disable_long_breaks: false,
    }
  }

  #[test]
  fn starts_idle_on_a_session() {
    let timer = Timer::new(test_settings());
    assert_eq!(timer.activity(), Activity::Session);
    assert_eq!(timer.state(), RunState::Idle);
    assert_eq!(timer.remaining(), Duration::from_secs(10));
  }

  #[test]
  fn session_cycles_into_short_then_long_breaks() {
    let mut timer = Timer::new(test_settings());

    for _ in 0..3 {
      assert_eq!(timer.finish(), Completion::Finished);
      assert_eq!(timer.activity(), Activity::ShortBreak);
      assert_eq!(timer.duration(), Duration::from_secs(4));
      timer.finish();
      assert_eq!(timer.activity(), Activity::Session);
    }

    timer.finish();
    assert_eq!(timer.activity(), Activity::LongBreak);
    assert_eq!(timer.duration(), Duration::from_secs(20));
    assert_eq!(timer.completed_sessions(), 4);
  }

  #[test]
  fn skip_moves_on_without_counting_a_session() {
    let mut timer = Timer::new(test_settings());
    timer.skip();
    assert_eq!(timer.activity(), Activity::ShortBreak);
    assert_eq!(timer.completed_sessions(), 0);
  }

  #[test]
  fn disable_breaks_jumps_straight_back_to_a_session() {
    let mut settings = test_settings();
    settings.disable_breaks = true;
    let mut timer = Timer::new(settings);
    timer.finish();
    assert_eq!(timer.activity(), Activity::Session);
    assert_eq!(timer.completed_sessions(), 1);
  }

  #[test]
  fn disable_long_breaks_keeps_only_short_breaks() {
    let mut settings = test_settings();
    settings.disable_long_breaks = true;
    let mut timer = Timer::new(settings);
    for _ in 0..4 {
      timer.finish();
      assert_eq!(timer.activity(), Activity::ShortBreak);
      timer.finish();
    }
  }

  #[test]
  fn pause_stops_elapsed_time() {
    let mut timer = Timer::new(test_settings());
    timer.start();
    std::thread::sleep(Duration::from_millis(30));
    timer.pause();
    let after_pause = timer.elapsed();
    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(timer.elapsed(), after_pause);
    assert_eq!(timer.state(), RunState::Paused);
  }

  #[test]
  fn stop_resets_the_current_activity() {
    let mut timer = Timer::new(test_settings());
    timer.start();
    std::thread::sleep(Duration::from_millis(20));
    timer.stop();
    assert_eq!(timer.elapsed(), Duration::ZERO);
    assert_eq!(timer.state(), RunState::Idle);
    assert_eq!(timer.activity(), Activity::Session);
  }

  #[test]
  fn restart_clears_progress_but_keeps_running() {
    let mut timer = Timer::new(test_settings());
    timer.start();
    std::thread::sleep(Duration::from_millis(20));
    timer.restart();
    assert!(timer.elapsed() < Duration::from_millis(20));
    assert_eq!(timer.state(), RunState::Running);
  }

  #[test]
  fn progress_and_remaining_track_the_activity() {
    let mut timer = Timer::new(test_settings());
    timer.start();
    std::thread::sleep(Duration::from_millis(20));
    assert!(timer.progress() > 0.0);
    assert!(timer.progress() < 1.0);
    assert!(timer.remaining() <= Duration::from_secs(10));
  }

  #[test]
  fn remaining_display_rounds_up_whole_seconds() {
    let timer = Timer::new(test_settings());
    assert_eq!(timer.remaining_display(), Duration::from_secs(10));
  }

  #[test]
  fn exact_and_fractional_rounding_and_deadlines() {
    let mut timer = Timer::new(test_settings());
    timer.elapsed = Duration::from_millis(250);
    assert_eq!(timer.remaining_display(), Duration::from_secs(10));
    assert_eq!(timer.next_display_change(), Duration::from_millis(750));
    timer.elapsed = Duration::from_secs(1);
    assert_eq!(timer.remaining_display(), Duration::from_secs(9));
    assert_eq!(timer.next_display_change(), Duration::from_secs(1));
    timer.elapsed = Duration::from_secs(10);
    assert_eq!(timer.remaining_display(), Duration::ZERO);
  }

  #[test]
  fn skipped_session_at_cycle_boundary_does_not_schedule_long_break() {
    let mut timer = Timer::new(test_settings());
    for _ in 0..4 {
      timer.finish();
      timer.finish();
    }
    timer.skip();
    assert_eq!(timer.activity(), Activity::ShortBreak);
    timer.reset_counter();
    assert_eq!(timer.completed_sessions(), 0);
  }

  #[test]
  fn changing_settings_preserves_overtime() {
    let mut timer = Timer::new(test_settings());
    timer.elapsed = Duration::from_secs(12);
    timer.apply_settings(test_settings());
    assert_eq!(timer.elapsed(), Duration::from_secs(12));
    assert_eq!(timer.remaining(), Duration::ZERO);
  }

  #[test]
  fn running_duration_change_clamps_actual_progress() {
    let mut timer = Timer::new(test_settings());
    timer.elapsed = Duration::from_secs(8);
    timer.start();
    timer.apply_settings(Durations {
      session: Duration::from_secs(5),
      ..test_settings()
    });
    assert_eq!(timer.remaining(), Duration::ZERO);
    assert_eq!(timer.state(), RunState::Running);
  }

  #[test]
  fn new_settings_reshape_the_current_activity() {
    let mut timer = Timer::new(test_settings());
    timer.skip();
    assert_eq!(timer.activity(), Activity::ShortBreak);
    let mut settings = test_settings();
    settings.short_break = Duration::from_secs(7);
    timer.apply_settings(settings);
    assert_eq!(timer.duration(), Duration::from_secs(7));
  }
}
