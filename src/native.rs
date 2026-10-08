//! AppKit bridge. All objects and callbacks stay on the main event-loop thread.
//! No tray daemon, polling loop, process spawning, or bundled sound assets.
#[derive(Clone, Copy)]
pub enum Command {
  Toggle,
  Restart,
  Skip,
  Stop,
  Finish,
  Show,
  Settings,
  Statistics,
  ResetCounter,
  Quit,
  Sleep,
  Wake,
}

#[cfg(target_os = "macos")]
mod mac {
  use super::Command;
  use crate::config::Sound;
  use objc2::rc::Retained;
  use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadOnly};
  use objc2_app_kit::{
    NSApplication, NSMenu, NSMenuItem, NSSound, NSStatusBar, NSStatusItem, NSWorkspace,
    NSWorkspaceDidWakeNotification, NSWorkspaceWillSleepNotification,
  };
  use objc2_foundation::{MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString};
  use std::cell::RefCell;
  use std::collections::VecDeque;

  pub struct Ivars {
    context: egui::Context,
    commands: RefCell<VecDeque<Command>>,
  }
  define_class!(
      // SAFETY: NSObject has no subclassing requirements. Main-thread-only ivars.
      #[unsafe(super = NSObject)]
      #[thread_kind = MainThreadOnly]
      #[ivars = Ivars]
      struct Target;
      unsafe impl NSObjectProtocol for Target {}
      impl Target {
          // SAFETY: these selector signatures match NSMenu target/action and NSNotification.
          #[unsafe(method(perform:))]
          fn perform(&self, item: &NSMenuItem) {
              let command = match item.tag() {
                  0 => Command::Toggle, 1 => Command::Restart, 2 => Command::Skip,
                  3 => Command::Stop, 4 => Command::Finish, 5 => Command::Show,
                  6 => Command::Settings, 7 => Command::Statistics, 8 => Command::ResetCounter,
                  _ => Command::Quit,
              };
              self.push(command);
          }
          #[unsafe(method(willSleep:))]
          fn will_sleep(&self, _notification: &NSNotification) { self.push(Command::Sleep); }
          #[unsafe(method(didWake:))]
          fn did_wake(&self, _notification: &NSNotification) { self.push(Command::Wake); }
      }
  );
  impl Target {
    fn new(mtm: MainThreadMarker, context: egui::Context) -> Retained<Self> {
      let this = Self::alloc(mtm).set_ivars(Ivars {
        context,
        commands: RefCell::new(VecDeque::new()),
      });
      // SAFETY: NSObject init has the declared signature.
      unsafe { msg_send![super(this), init] }
    }
    fn push(&self, command: Command) {
      self.ivars().commands.borrow_mut().push_back(command);
      self.ivars().context.request_repaint();
    }
  }
  pub struct Native {
    item: Retained<NSStatusItem>,
    target: Retained<Target>,
    toggle: Retained<NSMenuItem>,
    title: String,
  }
  impl Native {
    pub fn new(context: egui::Context) -> Self {
      let mtm = MainThreadMarker::new().expect("AppKit main thread");
      if crate::app::benchmark_locked() {
        for window in NSApplication::sharedApplication(mtm).windows().iter() {
          window.setIgnoresMouseEvents(true);
        }
      }
      let target = Target::new(mtm, context);
      let item = NSStatusBar::systemStatusBar().statusItemWithLength(-1.0);
      let menu = NSMenu::new(mtm);
      menu.setAutoenablesItems(false);
      let labels = [
        "Start / Pause",
        "Restart",
        "Skip",
        "Stop",
        "Finish overtime",
        "Show timer",
        "Settings...",
        "Statistics...",
        "Reset session counter",
        "Quit",
      ];
      let mut toggle = None;
      for (tag, title) in labels.into_iter().enumerate() {
        // SAFETY: perform: is implemented above, and target is retained until menu removal.
        let action = unsafe {
          NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            &NSString::from_str(title),
            Some(sel!(perform:)),
            &NSString::from_str(""),
          )
        };
        action.setTag(tag as isize);
        unsafe {
          action.setTarget(Some(&target));
        }
        menu.addItem(&action);
        if tag == 0 {
          toggle = Some(action);
        }
      }
      item.setMenu(Some(&menu));
      let center = NSWorkspace::sharedWorkspace().notificationCenter();
      // SAFETY: observer signatures match; observer stays alive and is removed in Drop.
      unsafe {
        center.addObserver_selector_name_object(
          &target,
          sel!(willSleep:),
          Some(NSWorkspaceWillSleepNotification),
          None,
        );
        center.addObserver_selector_name_object(
          &target,
          sel!(didWake:),
          Some(NSWorkspaceDidWakeNotification),
          None,
        );
      }
      Self {
        item,
        target,
        toggle: toggle.expect("toggle item"),
        title: String::new(),
      }
    }
    pub fn drain(&self) -> Vec<Command> {
      self
        .target
        .ivars()
        .commands
        .borrow_mut()
        .drain(..)
        .collect()
    }
    pub fn update(&mut self, app: &crate::app::App) {
      let seconds = app.display_seconds();
      let prefix = if app.is_overtime() { "+" } else { "" };
      let title = format!("{prefix}{:02}:{:02}", seconds / 60, seconds % 60);
      if title != self.title {
        let mtm = MainThreadMarker::new().expect("main thread");
        if let Some(button) = self.item.button(mtm) {
          button.setTitle(&NSString::from_str(&title));
        }
        self.title = title;
      }
      self
        .toggle
        .setTitle(&NSString::from_str(match app.timer.state() {
          crate::timer::RunState::Running => "Pause",
          crate::timer::RunState::Paused => "Resume",
          crate::timer::RunState::Idle => "Start",
        }));
    }
    pub fn activate(&self) {
      #[allow(deprecated)]
      NSApplication::sharedApplication(MainThreadMarker::new().expect("main thread"))
        .activateIgnoringOtherApps(true);
    }
  }
  impl Drop for Native {
    fn drop(&mut self) {
      self.item.setMenu(None);
      // SAFETY: our target is the registered observer and still alive.
      unsafe {
        NSWorkspace::sharedWorkspace()
          .notificationCenter()
          .removeObserver(&self.target);
      }
      NSStatusBar::systemStatusBar().removeStatusItem(&self.item);
    }
  }
  pub fn play(sound: Sound) {
    if sound == Sound::None {
      return;
    }
    if sound == Sound::System {
      objc2_app_kit::NSBeep();
    } else if let Some(audio) = NSSound::soundNamed(&NSString::from_str(sound.label())) {
      audio.play();
    }
  }
}
#[cfg(target_os = "macos")]
pub use mac::{play, Native};

#[cfg(not(target_os = "macos"))]
pub struct Native;
#[cfg(not(target_os = "macos"))]
impl Native {
  pub fn new(_: egui::Context) -> Self {
    Self
  }
  pub fn drain(&self) -> Vec<Command> {
    Vec::new()
  }
  pub fn update(&mut self, _: &crate::app::App) {}
  pub fn activate(&self) {}
}
#[cfg(not(target_os = "macos"))]
pub fn play(_: crate::config::Sound) {}
