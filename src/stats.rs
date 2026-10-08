//! Streaming CSV storage and a fixed seven-day cache: memory does not grow with history.
use crate::timer::Activity;
use std::cell::RefCell;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Totals {
  pub sessions: u32,
  pub breaks: u32,
  pub long_breaks: u32,
  pub session_seconds: u64,
  pub break_seconds: u64,
  pub skipped: u32,
}
impl Totals {
  fn add(&mut self, entry: Entry) {
    if !entry.completed {
      self.skipped = self.skipped.saturating_add(1);
    }
    match entry.activity {
      Activity::Session => {
        if entry.completed {
          self.sessions = self.sessions.saturating_add(1);
        }
        self.session_seconds = self.session_seconds.saturating_add(entry.seconds);
      }
      Activity::ShortBreak | Activity::LongBreak => {
        if entry.completed {
          self.breaks = self.breaks.saturating_add(1);
          if entry.activity == Activity::LongBreak {
            self.long_breaks = self.long_breaks.saturating_add(1);
          }
        }
        self.break_seconds = self.break_seconds.saturating_add(entry.seconds);
      }
    }
  }
  fn merge(&mut self, other: Self) {
    self.sessions = self.sessions.saturating_add(other.sessions);
    self.breaks = self.breaks.saturating_add(other.breaks);
    self.long_breaks = self.long_breaks.saturating_add(other.long_breaks);
    self.skipped = self.skipped.saturating_add(other.skipped);
    self.session_seconds = self.session_seconds.saturating_add(other.session_seconds);
    self.break_seconds = self.break_seconds.saturating_add(other.break_seconds);
  }
}
fn epoch_now() -> i64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_secs() as i64)
    .unwrap_or(0)
}
pub fn today_index() -> i64 {
  local_day(epoch_now())
}
fn local_day(epoch: i64) -> i64 {
  #[cfg(unix)]
  {
    let timestamp = epoch as libc::time_t;
    let mut date = std::mem::MaybeUninit::<libc::tm>::uninit();
    // SAFETY: pointers refer to initialized timestamp and writable tm storage.
    if !unsafe { libc::localtime_r(&timestamp, date.as_mut_ptr()) }.is_null() {
      // SAFETY: successful localtime_r initialized tm.
      let date = unsafe { date.assume_init() };
      return day_from_civil(
        i64::from(date.tm_year) + 1900,
        i64::from(date.tm_mon) + 1,
        i64::from(date.tm_mday),
      );
    }
  }
  epoch.div_euclid(86_400)
}
#[cfg(unix)]
fn day_from_civil(mut year: i64, month: i64, day: i64) -> i64 {
  year -= i64::from(month <= 2);
  let era = year.div_euclid(400);
  let yoe = year - era * 400;
  let mp = month + if month > 2 { -3 } else { 9 };
  let doy = (153 * mp + 2) / 5 + day - 1;
  era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468
}
pub fn format_date(day: i64) -> String {
  let z = day + 719468;
  let era = z.div_euclid(146097);
  let doe = z - era * 146097;
  let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
  let mut year = yoe + era * 400;
  let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
  let mp = (5 * doy + 2) / 153;
  let date = doy - (153 * mp + 2) / 5 + 1;
  let month = mp + if mp < 10 { 3 } else { -9 };
  year += i64::from(month <= 2);
  format!("{year:04}-{month:02}-{date:02}")
}
pub fn format_duration(seconds: u64) -> String {
  if seconds >= 3600 {
    format!("{}h {}m", seconds / 3600, seconds % 3600 / 60)
  } else {
    format!("{}m", seconds / 60)
  }
}

#[derive(Clone, Copy, Debug)]
struct Entry {
  epoch: i64,
  seconds: u64,
  activity: Activity,
  completed: bool,
}
impl Entry {
  fn parse(line: &[u8]) -> Option<Self> {
    let mut parts = std::str::from_utf8(line).ok()?.trim().split(',');
    let epoch = parts.next()?.parse().ok()?;
    let seconds = parts.next()?.parse().ok()?;
    let activity = match parts.next()? {
      "0" => Activity::Session,
      "1" => Activity::ShortBreak,
      "2" => Activity::LongBreak,
      _ => return None,
    };
    let completed = match parts.next()? {
      "1" | "true" => true,
      "0" | "false" => false,
      _ => return None,
    };
    if parts.next().is_some() {
      return None;
    }
    Some(Self {
      epoch,
      seconds,
      activity,
      completed,
    })
  }
}
#[derive(Clone, Copy)]
struct Cache {
  day: i64,
  completed_only: bool,
  series: [(i64, Totals); 7],
}
pub struct StatsStore {
  path: PathBuf,
  cache: RefCell<Option<Cache>>,
}
impl StatsStore {
  pub fn new(path: PathBuf) -> Self {
    Self {
      path,
      cache: RefCell::new(None),
    }
  }
  pub fn path(&self) -> &Path {
    &self.path
  }
  pub fn record(&self, activity: Activity, seconds: u64, completed: bool) -> io::Result<()> {
    if let Some(parent) = self.path.parent() {
      fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
      .create(true)
      .append(true)
      .open(&self.path)?;
    let kind = match activity {
      Activity::Session => 0,
      Activity::ShortBreak => 1,
      Activity::LongBreak => 2,
    };
    writeln!(
      file,
      "{},{seconds},{kind},{}",
      epoch_now(),
      u8::from(completed)
    )?;
    *self.cache.borrow_mut() = None;
    Ok(())
  }
  /// Fixed input buffers, even for corrupt multi-megabyte lines.
  fn scan(&self, mut visit: impl FnMut(Entry) -> io::Result<()>) -> io::Result<()> {
    let mut file = match File::open(&self.path) {
      Ok(file) => file,
      Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
      Err(e) => return Err(e),
    };
    let mut chunk = [0; 8192];
    let mut line = [0; 256];
    let (mut len, mut overflow) = (0, false);
    loop {
      let count = file.read(&mut chunk)?;
      if count == 0 {
        break;
      }
      for &byte in &chunk[..count] {
        if byte == b'\n' {
          if !overflow {
            if let Some(entry) = Entry::parse(&line[..len]) {
              visit(entry)?;
            }
          }
          len = 0;
          overflow = false;
        } else if len < line.len() {
          line[len] = byte;
          len += 1;
        } else {
          overflow = true;
        }
      }
    }
    if !overflow {
      if let Some(entry) = Entry::parse(&line[..len]) {
        visit(entry)?;
      }
    }
    Ok(())
  }
  pub fn daily_totals(&self, day: i64, completed_only: bool) -> io::Result<[(i64, Totals); 7]> {
    if let Some(cache) = *self.cache.borrow() {
      if cache.day == day && cache.completed_only == completed_only {
        return Ok(cache.series);
      }
    }
    let mut series = std::array::from_fn(|i| (day - 6 + i as i64, Totals::default()));
    self.scan(|entry| {
      let date = local_day(entry.epoch);
      if (day - 6..=day).contains(&date) && (!completed_only || entry.completed) {
        series[(date - day + 6) as usize].1.add(entry);
      }
      Ok(())
    })?;
    *self.cache.borrow_mut() = Some(Cache {
      day,
      completed_only,
      series,
    });
    Ok(series)
  }
  pub fn totals_for_day(&self, day: i64, completed_only: bool) -> io::Result<Totals> {
    Ok(self.daily_totals(day, completed_only)?[6].1)
  }
  pub fn totals_for_week(&self, day: i64, completed_only: bool) -> io::Result<Totals> {
    let mut totals = Totals::default();
    for (_, value) in self.daily_totals(day, completed_only)? {
      totals.merge(value);
    }
    Ok(totals)
  }
  pub fn export_csv(&self, path: &Path) -> io::Result<()> {
    if path == self.path {
      return Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Export must not overwrite the activity log",
      ));
    }
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("csv.tmp");
    let mut out = BufWriter::new(File::create(&temporary)?);
    writeln!(
      out,
      "date,epoch_seconds,duration_seconds,activity,completed"
    )?;
    self.scan(|entry| {
      let kind = match entry.activity {
        Activity::Session => "session",
        Activity::ShortBreak => "short_break",
        Activity::LongBreak => "long_break",
      };
      writeln!(
        out,
        "{},{},{},{},{}",
        format_date(local_day(entry.epoch)),
        entry.epoch,
        entry.seconds,
        kind,
        if entry.completed { "yes" } else { "no" }
      )
    })?;
    out.flush()?;
    drop(out);
    fs::rename(temporary, path)
  }
  pub fn reset(&self) -> io::Result<()> {
    match fs::remove_file(&self.path) {
      Ok(()) => {}
      Err(e) if e.kind() == io::ErrorKind::NotFound => {}
      Err(e) => return Err(e),
    }
    *self.cache.borrow_mut() = None;
    Ok(())
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicUsize, Ordering};
  static NEXT: AtomicUsize = AtomicUsize::new(0);
  fn store(text: &str) -> StatsStore {
    let path = std::env::temp_dir().join(format!(
      "tomito-stats-{}-{}.csv",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&path, text).unwrap();
    StatsStore::new(path)
  }
  #[test]
  fn totals_filter_incomplete_and_cover_seven_days() {
    let now = epoch_now();
    let old = now - 8 * 86400;
    let store = store(&format!(
      "{now},1500,0,1\n{now},300,1,1\n{now},120,0,0\n{now},60,2,0\n{old},999,0,1\n"
    ));
    let totals = store.totals_for_week(today_index(), true).unwrap();
    assert_eq!(
      (
        totals.sessions,
        totals.breaks,
        totals.session_seconds,
        totals.skipped
      ),
      (1, 1, 1500, 0)
    );
    let totals = store.totals_for_day(today_index(), false).unwrap();
    assert_eq!(
      (totals.session_seconds, totals.break_seconds, totals.skipped),
      (1620, 360, 2)
    );
    assert_eq!(
      store.daily_totals(today_index(), true).unwrap()[0].0,
      today_index() - 6
    );
    store.reset().unwrap();
  }
  #[test]
  fn export_has_five_columns_and_original_timestamp() {
    let store = store("1600000000,1500,0,1\nbroken\n1600000000,300,2,0\n");
    let target = store.path.with_extension("export.csv");
    store.export_csv(&target).unwrap();
    let text = fs::read_to_string(&target).unwrap();
    assert_eq!(text.lines().count(), 3);
    assert!(text.lines().all(|line| line.split(',').count() == 5));
    assert!(text.contains(",1600000000,1500,session,yes"));
    assert!(store.export_csv(&store.path).is_err());
    fs::remove_file(target).unwrap();
    store.reset().unwrap();
  }
  #[test]
  fn oversized_and_malformed_lines_are_ignored() {
    let store = store(&format!("{}\n1600000000,1500,0,1", "x".repeat(100_000)));
    let mut count = 0;
    store
      .scan(|_| {
        count += 1;
        Ok(())
      })
      .unwrap();
    assert_eq!(count, 1);
    assert!(Entry::parse(b"0,1,0,nonsense").is_none());
    assert!(Entry::parse(b"0,1,0,1,extra").is_none());
    store.reset().unwrap();
  }
  #[test]
  fn record_and_reset_invalidate_cache() {
    let store = store("");
    assert_eq!(
      store.totals_for_day(today_index(), true).unwrap().sessions,
      0
    );
    store.record(Activity::Session, 1500, true).unwrap();
    assert_eq!(
      store.totals_for_day(today_index(), true).unwrap().sessions,
      1
    );
    store.reset().unwrap();
    assert_eq!(
      store.totals_for_day(today_index(), true).unwrap().sessions,
      0
    );
  }
  #[test]
  fn calendar_and_duration_format() {
    assert_eq!(format_date(0), "1970-01-01");
    assert_eq!(format_date(19723), "2024-01-01");
    assert_eq!(format_duration(3900), "1h 5m");
    #[cfg(unix)]
    assert_eq!(day_from_civil(2024, 1, 1), 19723);
  }
}
