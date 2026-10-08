//! Plain-text settings. No serialization runtime or database.
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
  Flamingo,
  Mint,
  Mandarin,
  Monochrome,
  Pastel,
}
impl Theme {
  pub const ALL: [Self; 5] = [
    Self::Flamingo,
    Self::Mint,
    Self::Mandarin,
    Self::Monochrome,
    Self::Pastel,
  ];
  pub fn label(self) -> &'static str {
    match self {
      Self::Flamingo => "Flamingo",
      Self::Mint => "Mint",
      Self::Mandarin => "Mandarin",
      Self::Monochrome => "Monochrome",
      Self::Pastel => "Pastel",
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sound {
  Ping,
  Glass,
  Pop,
  Tink,
  Basso,
  Hero,
  Funk,
  Frog,
  Bottle,
  Purr,
  System,
  None,
}
impl Sound {
  pub const ALL: [Self; 12] = [
    Self::Ping,
    Self::Glass,
    Self::Pop,
    Self::Tink,
    Self::Basso,
    Self::Hero,
    Self::Funk,
    Self::Frog,
    Self::Bottle,
    Self::Purr,
    Self::System,
    Self::None,
  ];
  pub fn label(self) -> &'static str {
    match self {
      Self::Ping => "Ping",
      Self::Glass => "Glass",
      Self::Pop => "Pop",
      Self::Tink => "Tink",
      Self::Basso => "Basso",
      Self::Hero => "Hero",
      Self::Funk => "Funk",
      Self::Frog => "Frog",
      Self::Bottle => "Bottle",
      Self::Purr => "Purr",
      Self::System => "System",
      Self::None => "None",
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Settings {
  pub session_minutes: u32,
  pub short_break_minutes: u32,
  pub long_break_minutes: u32,
  pub long_break_cycle: u32,
  pub auto_start_session: bool,
  pub auto_start_break: bool,
  pub disable_breaks: bool,
  pub disable_long_breaks: bool,
  pub show_completed_only: bool,
  pub hide_on_start: bool,
  pub keep_in_front: bool,
  pub hide_on_launch: bool,
  pub show_on_finish: bool,
  pub overtime: bool,
  pub pause_on_sleep: bool,
  pub resume_on_wake: bool,
  pub ticking_sound: bool,
  pub mute_ticking_on_break: bool,
  pub theme: Theme,
  pub notification_sound: Sound,
}
impl Default for Settings {
  fn default() -> Self {
    Self {
      session_minutes: 25,
      short_break_minutes: 5,
      long_break_minutes: 15,
      long_break_cycle: 4,
      auto_start_session: true,
      auto_start_break: true,
      disable_breaks: false,
      disable_long_breaks: false,
      show_completed_only: true,
      hide_on_start: false,
      keep_in_front: false,
      hide_on_launch: false,
      show_on_finish: true,
      overtime: false,
      pause_on_sleep: true,
      resume_on_wake: true,
      ticking_sound: false,
      mute_ticking_on_break: true,
      theme: Theme::Flamingo,
      notification_sound: Sound::Ping,
    }
  }
}
impl Settings {
  pub fn serialize(&self) -> String {
    use std::fmt::Write;
    let mut text = String::with_capacity(640);
    macro_rules! fields { ($($field:ident),*) => { $(writeln!(text, "{}={}", stringify!($field), self.$field).expect("string write");)* }; }
    fields!(
      session_minutes,
      short_break_minutes,
      long_break_minutes,
      long_break_cycle,
      auto_start_session,
      auto_start_break,
      disable_breaks,
      disable_long_breaks,
      show_completed_only,
      hide_on_start,
      keep_in_front,
      hide_on_launch,
      show_on_finish,
      overtime,
      pause_on_sleep,
      resume_on_wake,
      ticking_sound,
      mute_ticking_on_break
    );
    writeln!(text, "theme={}", self.theme.label()).expect("string write");
    writeln!(
      text,
      "notification_sound={}",
      self.notification_sound.label()
    )
    .expect("string write");
    text
  }
  pub fn parse(text: &str) -> Self {
    let mut settings = Self::default();
    for line in text.lines() {
      let Some((key, value)) = line.split_once('=') else {
        continue;
      };
      let (key, value) = (key.trim(), value.trim());
      macro_rules! boolean {
        ($field:ident) => {
          settings.$field = match value {
            "true" | "1" => true,
            "false" | "0" => false,
            _ => settings.$field,
          }
        };
      }
      match key {
        "session_minutes" => {
          settings.session_minutes = clamp(value, 1, 180, settings.session_minutes)
        }
        "short_break_minutes" => {
          settings.short_break_minutes = clamp(value, 1, 60, settings.short_break_minutes)
        }
        "long_break_minutes" => {
          settings.long_break_minutes = clamp(value, 1, 120, settings.long_break_minutes)
        }
        "long_break_cycle" => {
          settings.long_break_cycle = clamp(value, 2, 10, settings.long_break_cycle)
        }
        "auto_start_session" => boolean!(auto_start_session),
        "auto_start_break" => boolean!(auto_start_break),
        "disable_breaks" => boolean!(disable_breaks),
        "disable_long_breaks" => boolean!(disable_long_breaks),
        "show_completed_only" => boolean!(show_completed_only),
        "hide_on_start" => boolean!(hide_on_start),
        "keep_in_front" => boolean!(keep_in_front),
        "hide_on_launch" => boolean!(hide_on_launch),
        "show_on_finish" => boolean!(show_on_finish),
        "overtime" => boolean!(overtime),
        "pause_on_sleep" => boolean!(pause_on_sleep),
        "resume_on_wake" => boolean!(resume_on_wake),
        "ticking_sound" => boolean!(ticking_sound),
        "mute_ticking_on_break" => boolean!(mute_ticking_on_break),
        "theme" => {
          if let Some(theme) = Theme::ALL.into_iter().find(|t| {
            t.label().eq_ignore_ascii_case(value) || (*t == Theme::Monochrome && value == "mono")
          }) {
            settings.theme = theme;
          }
        }
        "notification_sound" => {
          if let Some(sound) = Sound::ALL
            .into_iter()
            .find(|s| s.label().eq_ignore_ascii_case(value))
          {
            settings.notification_sound = sound;
          }
        }
        _ => {}
      }
    }
    settings
  }
  pub fn load(path: &Path) -> Self {
    fs::read_to_string(path)
      .map(|text| Self::parse(&text))
      .unwrap_or_default()
  }
  pub fn save(&self, path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("conf.tmp");
    fs::write(&temporary, self.serialize())?;
    fs::rename(temporary, path)
  }
}
fn clamp(text: &str, min: u32, max: u32, default: u32) -> u32 {
  text
    .parse::<u32>()
    .map(|v| v.clamp(min, max))
    .unwrap_or(default)
}
pub fn config_dir(app_name: &str) -> PathBuf {
  #[cfg(target_os = "macos")]
  if let Some(home) = std::env::var_os("HOME") {
    return Path::new(&home)
      .join("Library/Application Support")
      .join(app_name);
  }
  #[cfg(target_os = "windows")]
  if let Some(appdata) = std::env::var_os("APPDATA") {
    return Path::new(&appdata).join(app_name);
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
      return Path::new(&xdg).join(app_name);
    }
    if let Some(home) = std::env::var_os("HOME") {
      return Path::new(&home).join(".config").join(app_name);
    }
  }
  PathBuf::from(app_name)
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn round_trip() {
    let settings = Settings {
      session_minutes: 45,
      theme: Theme::Mint,
      notification_sound: Sound::Glass,
      hide_on_launch: true,
      overtime: true,
      ..Default::default()
    };
    assert_eq!(Settings::parse(&settings.serialize()), settings);
  }
  #[test]
  fn invalid_input_and_ranges() {
    let s = Settings::parse(
      "session_minutes=9999\nlong_break_cycle=0\ntheme=unknown\nshort_break_minutes=abc\n",
    );
    assert_eq!(s.session_minutes, 180);
    assert_eq!(s.long_break_cycle, 2);
    assert_eq!(s.short_break_minutes, 5);
    assert_eq!(s.theme, Theme::Flamingo);
    assert_eq!(Settings::parse("garbage"), Settings::default());
  }
  #[test]
  fn old_theme_keys_still_load() {
    assert_eq!(Settings::parse("theme=mint").theme, Theme::Mint);
    assert_eq!(Settings::parse("theme=mono").theme, Theme::Monochrome);
  }
}
