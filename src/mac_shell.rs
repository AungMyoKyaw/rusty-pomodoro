//! Renderer-free macOS UI. Rust owns timer/storage; AppKit owns controls and drawing.
//! One one-shot NSTimer while running; no wakeups while idle and no GPU context.
use crate::app::{App, Panel};
use crate::config::{self, Settings, Sound, Theme};
use crate::stats::{self, StatsStore};
use crate::timer::RunState;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadOnly};
use objc2_app_kit::*;
use objc2_foundation::{
  MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString,
  NSTimer,
};
use std::cell::{OnceCell, RefCell};

struct SettingsUi {
  window: Retained<NSWindow>,
  numbers: [Retained<NSTextField>; 4],
  checks: Vec<Retained<NSButton>>,
  theme: Retained<NSPopUpButton>,
  sound: Retained<NSPopUpButton>,
}
struct StatisticsUi {
  window: Retained<NSWindow>,
  summary: Retained<NSTextField>,
  days: Vec<(Retained<NSTextField>, Retained<NSProgressIndicator>)>,
}
struct State {
  app: App,
  alarm: Option<Retained<NSTimer>>,
  settings: Option<SettingsUi>,
  statistics: Option<StatisticsUi>,
}
struct Views {
  window: Retained<NSWindow>,
  time: Retained<NSTextField>,
  activity: Retained<NSTextField>,
  counter: Retained<NSTextField>,
  status: Retained<NSTextField>,
  toggle: Retained<NSButton>,
  finish: Retained<NSButton>,
  item: Retained<NSStatusItem>,
  menu_toggle: Retained<NSMenuItem>,
}
pub struct Ivars {
  state: RefCell<State>,
  views: OnceCell<Views>,
  keys: OnceCell<crate::hotkeys::Hotkeys>,
}

define_class!(
    // SAFETY: NSObject has no subclass requirements; ivars stay on AppKit's main thread.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = Ivars]
    struct Controller;
    unsafe impl NSObjectProtocol for Controller {}
    unsafe impl NSApplicationDelegate for Controller {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn launch(&self, _: &NSNotification) { self.build(); }
        #[unsafe(method(applicationWillTerminate:))]
        fn terminating(&self, _: &NSNotification) {
            let mut state = self.ivars().state.borrow_mut();
            state.app.save_settings();
            if let Some(keys) = self.ivars().keys.get() { keys.cleanup(); }
            if let Some(timer) = state.alarm.take() { timer.invalidate(); }
        }
        #[unsafe(method(applicationShouldTerminateAfterLastWindowClosed:))]
        fn quit_after_close(&self, _: &NSApplication) -> bool { false }
    }
    unsafe impl NSWindowDelegate for Controller {
        #[unsafe(method(windowWillClose:))]
        fn window_closed(&self, notification: &NSNotification) {
            // SAFETY: AppKit's close notification object is the closing NSWindow.
            let Some(window) = notification.object().and_then(|o| o.downcast::<NSWindow>().ok()) else { return };
            let mut state = self.ivars().state.borrow_mut();
            if state.settings.as_ref().is_some_and(|ui| ui.window == window) {
                if let Some(ui) = state.settings.take() {
                    // Defer final window release until AppKit finishes this event.
                    let _ = Retained::autorelease_ptr(ui.window);
                }
            }
            if state.statistics.as_ref().is_some_and(|ui| ui.window == window) {
                if let Some(ui) = state.statistics.take() { let _ = Retained::autorelease_ptr(ui.window); }
            }
        }
        #[unsafe(method(windowShouldClose:))]
        fn should_close(&self, window: &NSWindow) -> bool {
            if crate::app::benchmark_locked() { return false.into(); }
            if &*self.views().window == window {
                window.orderOut(None);
                false
            } else {
                let is_settings = self.ivars().state.borrow().settings.as_ref().is_some_and(|s| &*s.window == window);
                if is_settings { self.apply_controls(); }
                self.ivars().state.borrow_mut().app.panel = Panel::None;
                true
            }
        }
    }
    impl Controller {
        // SAFETY: selectors below have AppKit's action, timer, and notification signatures.
        #[unsafe(method(act:))]
        fn act(&self, sender: &AnyObject) {
            if crate::app::benchmark_locked() { return; }
            // SAFETY: action targets are exclusively our NSButton/NSMenuItem instances.
            let tag: isize = unsafe { msg_send![sender, tag] };
            match tag {
                5 => { self.show(); return; }
                6 => { self.open_settings(); return; }
                7 => { self.open_statistics(); return; }
                9 => {
                    self.ivars().state.borrow_mut().app.exit_requested = true;
                    // SAFETY: termination invoked on application's main thread.
                    NSApplication::sharedApplication(self.mtm()).terminate(None);
                    return;
                }
                10 => { self.apply_controls(); return; }
                11 => { self.export(); return; }
                12 => { self.reset_statistics(); return; }
                13 => {
                    self.ivars().state.borrow_mut().app.stats_scope_day = true;
                    self.update_statistics(); return;
                }
                14 => {
                    self.ivars().state.borrow_mut().app.stats_scope_day = false;
                    self.update_statistics(); return;
                }
                15 => {
                    self.apply_controls();
                    play(self.ivars().state.borrow().app.settings.notification_sound);
                    return;
                }
                _ => {}
            }
            {
                let mut state = self.ivars().state.borrow_mut();
                match tag {
                    0 => state.app.toggle(), 1 => state.app.restart(), 2 => state.app.skip(),
                    3 => state.app.stop(), 4 => { if state.app.is_overtime() { state.app.complete_current(); } }
                    8 => state.app.timer.reset_counter(), _ => {}
                }
            }
            self.refresh();
        }
        #[unsafe(method(tick:))]
        fn tick(&self, _: &NSTimer) {
            self.ivars().state.borrow_mut().app.tick();
            self.refresh();
        }
        #[unsafe(method(willSleep:))]
        fn sleep(&self, _: &NSNotification) { self.ivars().state.borrow_mut().app.sleep(); self.refresh(); }
        #[unsafe(method(didWake:))]
        fn wake(&self, _: &NSNotification) { self.ivars().state.borrow_mut().app.wake(); self.refresh(); }
    }
);
fn rect(x: f64, y: f64, w: f64, h: f64) -> NSRect {
  NSRect::new(NSPoint::new(x, y), NSSize::new(w, h))
}
fn text(
  view: &NSView,
  mtm: MainThreadMarker,
  value: &str,
  frame: NSRect,
  size: f64,
) -> Retained<NSTextField> {
  let label = NSTextField::labelWithString(&NSString::from_str(value), mtm);
  label.setFrame(frame);
  label.setFont(Some(&NSFont::systemFontOfSize(size)));
  // SAFETY: both views are main-thread AppKit objects; parent retains subview.
  view.addSubview(&label);
  label
}
fn play(sound: Sound) {
  if sound == Sound::System {
    NSBeep();
  } else if sound != Sound::None {
    if let Some(audio) = NSSound::soundNamed(&NSString::from_str(sound.label())) {
      audio.play();
    }
  }
}
impl Controller {
  fn new(mtm: MainThreadMarker, app: App) -> Retained<Self> {
    let this = Self::alloc(mtm).set_ivars(Ivars {
      state: RefCell::new(State {
        app,
        alarm: None,
        settings: None,
        statistics: None,
      }),
      views: OnceCell::new(),
      keys: OnceCell::new(),
    });
    // SAFETY: NSObject init signature.
    unsafe { msg_send![super(this), init] }
  }
  fn views(&self) -> &Views {
    self.ivars().views.get().expect("views initialized")
  }
  fn window(&self, title: &str, width: f64, height: f64) -> Retained<NSWindow> {
    // SAFETY: creation on main thread; automatic release on close is disabled.
    let window = unsafe {
      NSWindow::initWithContentRect_styleMask_backing_defer(
        NSWindow::alloc(self.mtm()),
        rect(0.0, 0.0, width, height),
        NSWindowStyleMask::Titled | NSWindowStyleMask::Closable | NSWindowStyleMask::Miniaturizable,
        NSBackingStoreType::Buffered,
        false,
      )
    };
    unsafe {
      window.setReleasedWhenClosed(false);
    }
    window.setTitle(&NSString::from_str(title));
    window.setDelegate(Some(ProtocolObject::from_ref(self)));
    if let Some(appearance) = NSAppearance::appearanceNamed(unsafe { NSAppearanceNameDarkAqua }) {
      window.setAppearance(Some(&appearance));
    }
    window.center();
    window
  }
  fn button(&self, view: &NSView, title: &str, tag: isize, frame: NSRect) -> Retained<NSButton> {
    // SAFETY: act: matches action signature; controller outlives its views.
    let button = unsafe {
      NSButton::buttonWithTitle_target_action(
        &NSString::from_str(title),
        Some(self),
        Some(sel!(act:)),
        self.mtm(),
      )
    };
    button.setTag(tag);
    button.setFrame(frame);
    view.addSubview(&button);
    button
  }
  fn menu_item(&self, menu: &NSMenu, title: &str, tag: isize, key: &str) -> Retained<NSMenuItem> {
    // SAFETY: act: is implemented; controller retained by main until termination.
    let item = unsafe {
      NSMenuItem::initWithTitle_action_keyEquivalent(
        NSMenuItem::alloc(self.mtm()),
        &NSString::from_str(title),
        Some(sel!(act:)),
        &NSString::from_str(key),
      )
    };
    item.setTag(tag);
    unsafe {
      item.setTarget(Some(self));
    }
    menu.addItem(&item);
    item
  }
  fn build(&self) {
    let window = self.window("Rusty Pomodoro", 380.0, 300.0);
    let view = window.contentView().expect("content view");
    let activity = text(
      &view,
      self.mtm(),
      "Session",
      rect(24.0, 238.0, 332.0, 26.0),
      16.0,
    );
    activity.setAlignment(NSTextAlignment::Center);
    let time = text(
      &view,
      self.mtm(),
      "25:00",
      rect(24.0, 155.0, 332.0, 76.0),
      58.0,
    );
    time.setAlignment(NSTextAlignment::Center);
    time.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
      58.0, 0.0,
    )));
    let counter = text(&view, self.mtm(), "", rect(24.0, 126.0, 332.0, 22.0), 12.0);
    counter.setAlignment(NSTextAlignment::Center);
    let toggle = self.button(&view, "Start", 0, rect(70.0, 82.0, 100.0, 32.0));
    toggle.setKeyEquivalent(&NSString::from_str(" "));
    let finish = self.button(&view, "Finish", 4, rect(210.0, 82.0, 100.0, 32.0));
    finish.setHidden(true);
    self.button(&view, "Restart", 1, rect(174.0, 82.0, 96.0, 32.0));
    self.button(&view, "Skip", 2, rect(282.0, 82.0, 72.0, 32.0));
    self.button(&view, "Stop", 3, rect(24.0, 43.0, 75.0, 28.0));
    self.button(&view, "Settings", 6, rect(110.0, 43.0, 116.0, 28.0));
    self.button(&view, "Statistics", 7, rect(238.0, 43.0, 116.0, 28.0));
    let status = text(&view, self.mtm(), "", rect(24.0, 10.0, 332.0, 26.0), 11.0);
    status.setAlignment(NSTextAlignment::Center);
    let item = NSStatusBar::systemStatusBar().statusItemWithLength(-1.0);
    let menu = NSMenu::new(self.mtm());
    menu.setAutoenablesItems(false);
    let menu_toggle = self.menu_item(&menu, "Start", 0, "");
    for (title, tag) in [
      ("Restart", 1),
      ("Skip", 2),
      ("Stop", 3),
      ("Finish overtime", 4),
      ("Show timer", 5),
      ("Settings...", 6),
      ("Statistics...", 7),
      ("Reset session counter", 8),
      ("Quit", 9),
    ] {
      self.menu_item(&menu, title, tag, "");
    }
    item.setMenu(Some(&menu));
    // App-local shortcuts; no global keyboard hook or permission requirement.
    let main_menu = NSMenu::new(self.mtm());
    let application = NSMenuItem::new(self.mtm());
    application.setTitle(&NSString::from_str("Rusty Pomodoro"));
    let commands = NSMenu::new(self.mtm());
    commands.setAutoenablesItems(false);
    for (title, tag, key) in [
      ("Start / Pause", 0, "t"),
      ("Restart", 1, "r"),
      ("Skip", 2, "s"),
      ("Stop", 3, "x"),
      ("Show timer", 5, "1"),
      ("Settings...", 6, ","),
      ("Statistics...", 7, "2"),
      ("Quit", 9, "q"),
    ] {
      self.menu_item(&commands, title, tag, key);
    }
    application.setSubmenu(Some(&commands));
    main_menu.addItem(&application);
    NSApplication::sharedApplication(self.mtm()).setMainMenu(Some(&main_menu));
    self
      .ivars()
      .views
      .set(Views {
        window,
        time,
        activity,
        counter,
        status,
        toggle,
        finish,
        item,
        menu_toggle,
      })
      .ok()
      .expect("one initialization");
    // SAFETY: controller remains retained by run() until after event-loop termination.
    let keys =
      unsafe { crate::hotkeys::Hotkeys::new(hotkey, (self as *const Self).cast_mut().cast()) };
    if keys.failed {
      self.ivars().state.borrow_mut().app.status =
        "Some global shortcuts are already in use".into();
    }
    assert!(self.ivars().keys.set(keys).is_ok());
    let center = NSWorkspace::sharedWorkspace().notificationCenter();
    // SAFETY: observers implemented above and removed after NSApp.run returns.
    unsafe {
      center.addObserver_selector_name_object(
        self,
        sel!(willSleep:),
        Some(NSWorkspaceWillSleepNotification),
        None,
      );
      center.addObserver_selector_name_object(
        self,
        sel!(didWake:),
        Some(NSWorkspaceDidWakeNotification),
        None,
      );
    }
    if !self.ivars().state.borrow().app.settings.hide_on_launch {
      self.show();
    }
    match config::var("RUSTY_POMODORO_BENCHMARK_SCENARIO").as_deref() {
      Ok("running") => self.ivars().state.borrow_mut().app.toggle(),
      Ok("statistics") => self.open_statistics(),
      _ => {}
    }
    self.refresh();
  }
  fn present(&self, window: &NSWindow) {
    if config::var_os("RUSTY_POMODORO_BENCHMARK").is_some() {
      window.setIgnoresMouseEvents(crate::app::benchmark_locked());
      window.orderFront(None);
    } else {
      window.makeKeyAndOrderFront(None);
      #[allow(deprecated)]
      NSApplication::sharedApplication(self.mtm()).activateIgnoringOtherApps(true);
    }
  }
  fn show(&self) {
    self.present(&self.views().window);
  }
  fn refresh(&self) {
    let views = self.views();
    let mut state = self.ivars().state.borrow_mut();
    if let Some(timer) = state.alarm.take() {
      timer.invalidate();
    }
    let app = &mut state.app;
    app.verify_benchmark_scenario();
    let seconds = app.display_seconds();
    let display = format!(
      "{}{:02}:{:02}",
      if app.is_overtime() { "+" } else { "" },
      seconds / 60,
      seconds % 60
    );
    let value = NSString::from_str(&display);
    if views.time.stringValue() != value {
      views.time.setStringValue(&value);
    }
    if let Some(button) = views.item.button(self.mtm()) {
      if button.title() != value {
        button.setTitle(&value);
      }
    }
    let action = match app.timer.state() {
      RunState::Idle => "Start",
      RunState::Running => "Pause",
      RunState::Paused => "Resume",
    };
    views.toggle.setTitle(&NSString::from_str(action));
    views.menu_toggle.setTitle(&NSString::from_str(action));
    views.finish.setHidden(!app.is_overtime());
    // Finish gets its own row, never overlaps Restart/Skip.
    views.finish.setFrame(rect(24.0, 266.0, 76.0, 26.0));
    views
      .activity
      .setStringValue(&NSString::from_str(app.timer.activity().label()));
    views.counter.setStringValue(&NSString::from_str(&format!(
      "Session {} of {} in cycle",
      app.timer.completed_sessions() % app.settings.long_break_cycle + 1,
      app.settings.long_break_cycle
    )));
    views
      .status
      .setStringValue(&NSString::from_str(&app.status));
    views
      .window
      .setLevel(if app.settings.keep_in_front { 3 } else { 0 });
    let rgb = match app.settings.theme {
      Theme::Flamingo => (0.91, 0.29, 0.37),
      Theme::Mint => (0.25, 0.76, 0.63),
      Theme::Mandarin => (0.95, 0.55, 0.16),
      Theme::Monochrome => (0.85, 0.85, 0.85),
      Theme::Pastel => (0.66, 0.78, 0.91),
    };
    views
      .activity
      .setTextColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
        rgb.0, rgb.1, rgb.2, 1.0,
      )));
    if std::mem::take(&mut app.hide_requested) {
      views.window.orderOut(None);
    }
    let show = std::mem::take(&mut app.show_requested);
    if std::mem::take(&mut app.notify_requested) {
      play(app.settings.notification_sound);
    }
    if std::mem::take(&mut app.tick_requested) {
      play(Sound::Tink);
    }
    if app.should_save() {
      app.save_settings();
    }
    if app.timer.state() == RunState::Running {
      let delay = app.timer.next_display_change().as_secs_f64().max(0.01);
      // SAFETY: tick: accepts NSTimer; retained target outlives timer. One shot only.
      let timer = unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
          delay,
          self,
          sel!(tick:),
          None,
          false,
        )
      };
      timer.setTolerance(0.02);
      state.alarm = Some(timer);
    }
    drop(state);
    if show {
      self.show();
    }
    self.update_statistics();
  }
  fn open_settings(&self) {
    self.ivars().state.borrow_mut().app.panel = Panel::Settings;
    if self.ivars().state.borrow().settings.is_none() {
      let settings = self.ivars().state.borrow().app.settings;
      let window = self.window("Settings", 440.0, 580.0);
      let root = window.contentView().expect("content view");
      let scroll = NSScrollView::initWithFrame(
        NSScrollView::alloc(self.mtm()),
        rect(0.0, 58.0, 440.0, 522.0),
      );
      scroll.setHasVerticalScroller(true);
      let document = NSView::initWithFrame(NSView::alloc(self.mtm()), rect(0.0, 0.0, 420.0, 810.0));
      scroll.setDocumentView(Some(&document));
      root.addSubview(&scroll);
      let mut y = 770.0;
      text(
        &document,
        self.mtm(),
        "Durations",
        rect(20.0, y, 370.0, 24.0),
        16.0,
      );
      y -= 32.0;
      let mut numbers = Vec::new();
      for (label, value) in [
        ("Session (1-180 min)", settings.session_minutes),
        ("Short break (1-60 min)", settings.short_break_minutes),
        ("Long break (1-120 min)", settings.long_break_minutes),
        ("Long break cycle (2-10)", settings.long_break_cycle),
      ] {
        text(
          &document,
          self.mtm(),
          label,
          rect(20.0, y, 285.0, 24.0),
          13.0,
        );
        let field =
          NSTextField::initWithFrame(NSTextField::alloc(self.mtm()), rect(310.0, y, 80.0, 24.0));
        field.setStringValue(&NSString::from_str(&value.to_string()));
        document.addSubview(&field);
        numbers.push(field);
        y -= 32.0;
      }
      text(
        &document,
        self.mtm(),
        "Timer and window",
        rect(20.0, y, 370.0, 24.0),
        16.0,
      );
      y -= 30.0;
      let values = [
        ("Start sessions automatically", settings.auto_start_session),
        ("Start breaks automatically", settings.auto_start_break),
        ("Disable all breaks", settings.disable_breaks),
        ("Disable long breaks", settings.disable_long_breaks),
        (
          "Show completed activities only",
          settings.show_completed_only,
        ),
        ("Hide when timer starts", settings.hide_on_start),
        ("Keep timer in front", settings.keep_in_front),
        ("Hide on launch", settings.hide_on_launch),
        ("Show when timer ends", settings.show_on_finish),
        ("Continue past zero until Finish", settings.overtime),
        ("Pause when Mac sleeps", settings.pause_on_sleep),
        ("Resume when Mac wakes", settings.resume_on_wake),
        ("Tick once per second", settings.ticking_sound),
        ("Mute ticking during breaks", settings.mute_ticking_on_break),
      ];
      let mut checks = Vec::new();
      for (title, value) in values {
        // Checkbox edits are applied together, not on every click.
        let check = unsafe {
          NSButton::checkboxWithTitle_target_action(
            &NSString::from_str(title),
            None,
            None,
            self.mtm(),
          )
        };
        check.setFrame(rect(20.0, y, 380.0, 24.0));
        check.setState(isize::from(value));
        document.addSubview(&check);
        checks.push(check);
        y -= 28.0;
      }
      text(
        &document,
        self.mtm(),
        "Theme",
        rect(20.0, y, 180.0, 24.0),
        13.0,
      );
      let theme = NSPopUpButton::initWithFrame_pullsDown(
        NSPopUpButton::alloc(self.mtm()),
        rect(200.0, y, 190.0, 26.0),
        false,
      );
      for choice in Theme::ALL {
        theme.addItemWithTitle(&NSString::from_str(choice.label()));
      }
      theme.selectItemAtIndex(
        Theme::ALL
          .iter()
          .position(|t| *t == settings.theme)
          .unwrap_or(0) as isize,
      );
      document.addSubview(&theme);
      y -= 34.0;
      text(
        &document,
        self.mtm(),
        "Notification sound",
        rect(20.0, y, 180.0, 24.0),
        13.0,
      );
      let sound = NSPopUpButton::initWithFrame_pullsDown(
        NSPopUpButton::alloc(self.mtm()),
        rect(200.0, y, 190.0, 26.0),
        false,
      );
      for choice in Sound::ALL {
        sound.addItemWithTitle(&NSString::from_str(choice.label()));
      }
      sound.selectItemAtIndex(
        Sound::ALL
          .iter()
          .position(|s| *s == settings.notification_sound)
          .unwrap_or(0) as isize,
      );
      document.addSubview(&sound);
      self.button(&root, "Apply and save", 10, rect(20.0, 16.0, 150.0, 30.0));
      self.button(&root, "Preview sound", 15, rect(180.0, 16.0, 130.0, 30.0));
      // Scroll to top because NSView uses bottom-left coordinates.
      document.scrollPoint(NSPoint::new(0.0, 810.0));
      let numbers = numbers.try_into().expect("four duration fields");
      self.ivars().state.borrow_mut().settings = Some(SettingsUi {
        window,
        numbers,
        checks,
        theme,
        sound,
      });
    }
    self.present(
      &self
        .ivars()
        .state
        .borrow()
        .settings
        .as_ref()
        .unwrap()
        .window,
    );
  }
  fn apply_controls(&self) {
    {
      let mut state = self.ivars().state.borrow_mut();
      let Some(ui) = &state.settings else { return };
      let mut settings = state.app.settings;
      let old_theme = settings.theme;
      let old_sound = settings.notification_sound;
      let bounds = [(1, 180), (1, 60), (1, 120), (2, 10)];
      let mut values = [
        settings.session_minutes,
        settings.short_break_minutes,
        settings.long_break_minutes,
        settings.long_break_cycle,
      ];
      for i in 0..4 {
        if let Ok(value) = ui.numbers[i].stringValue().to_string().parse::<u32>() {
          values[i] = value.clamp(bounds[i].0, bounds[i].1);
        }
        ui.numbers[i].setStringValue(&NSString::from_str(&values[i].to_string()));
      }
      [
        settings.session_minutes,
        settings.short_break_minutes,
        settings.long_break_minutes,
        settings.long_break_cycle,
      ] = values;
      macro_rules! checks { ($($field:ident),*) => { let mut index=0; $(settings.$field=ui.checks[index].state()==1; index+=1;)* let _=index; }; }
      checks!(
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
      settings.theme = Theme::ALL[ui.theme.indexOfSelectedItem().max(0) as usize];
      settings.notification_sound = Sound::ALL[ui.sound.indexOfSelectedItem().max(0) as usize];
      state.app.settings = settings;
      if settings.theme != old_theme {
        state.app.set_theme(settings.theme);
      }
      if settings.notification_sound != old_sound {
        state.app.set_sound(settings.notification_sound);
      }
      state.app.apply_settings();
      state.app.status = "Settings saved".into();
      state.app.save_settings();
    }
    self.refresh();
  }
  fn open_statistics(&self) {
    self.ivars().state.borrow_mut().app.panel = Panel::Statistics;
    if self.ivars().state.borrow().statistics.is_none() {
      let window = self.window("Statistics", 440.0, 440.0);
      let root = window.contentView().expect("content view");
      self.button(&root, "Today", 13, rect(20.0, 390.0, 100.0, 30.0));
      self.button(&root, "Last 7 days", 14, rect(126.0, 390.0, 130.0, 30.0));
      let summary = text(&root, self.mtm(), "", rect(20.0, 282.0, 400.0, 94.0), 14.0);
      let mut days = Vec::new();
      for i in 0..7 {
        let y = 245.0 - i as f64 * 27.0;
        let label = text(&root, self.mtm(), "", rect(20.0, y, 220.0, 22.0), 12.0);
        let bar = NSProgressIndicator::initWithFrame(
          NSProgressIndicator::alloc(self.mtm()),
          rect(245.0, y + 4.0, 175.0, 12.0),
        );
        bar.setIndeterminate(false);
        bar.setMinValue(0.0);
        root.addSubview(&bar);
        days.push((label, bar));
      }
      self.button(&root, "Export CSV", 11, rect(20.0, 20.0, 120.0, 30.0));
      self.button(
        &root,
        "Reset statistics",
        12,
        rect(245.0, 20.0, 175.0, 30.0),
      );
      self.ivars().state.borrow_mut().statistics = Some(StatisticsUi {
        window,
        summary,
        days,
      });
    }
    self.present(
      &self
        .ivars()
        .state
        .borrow()
        .statistics
        .as_ref()
        .unwrap()
        .window,
    );
    self.update_statistics();
  }
  fn update_statistics(&self) {
    let state = self.ivars().state.borrow();
    let Some(ui) = &state.statistics else { return };
    if !ui.window.isVisible() {
      return;
    }
    let app = &state.app;
    let today = stats::today_index();
    let totals = if app.stats_scope_day {
      app
        .stats
        .totals_for_day(today, app.settings.show_completed_only)
    } else {
      app
        .stats
        .totals_for_week(today, app.settings.show_completed_only)
    };
    match totals {
      Ok(t) => ui.summary.setStringValue(&NSString::from_str(&format!(
        "{}: {} sessions / {} breaks\nFocus: {} / Breaks: {}\nLong breaks: {} / Skipped: {}",
        if app.stats_scope_day {
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
      ))),
      Err(e) => ui
        .summary
        .setStringValue(&NSString::from_str(&format!("Statistics read failed: {e}"))),
    }
    if let Ok(series) = app
      .stats
      .daily_totals(today, app.settings.show_completed_only)
    {
      let maximum = series
        .iter()
        .map(|(_, t)| t.session_seconds)
        .max()
        .unwrap_or(1)
        .max(1);
      for ((label, bar), (day, totals)) in ui.days.iter().zip(series) {
        label.setStringValue(&NSString::from_str(&format!(
          "{} / {}",
          stats::format_date(day),
          stats::format_duration(totals.session_seconds)
        )));
        bar.setMaxValue(maximum as f64);
        bar.setDoubleValue(totals.session_seconds as f64);
      }
    }
  }
  fn export(&self) {
    {
      let mut state = self.ivars().state.borrow_mut();
      let target = state
        .app
        .stats
        .path()
        .with_file_name("rusty-pomodoro-stats.csv");
      state.app.status = match state.app.stats.export_csv(&target) {
        Ok(()) => format!("Exported to {}", target.display()),
        Err(e) => format!("Export failed: {e}"),
      };
    }
    self.show();
    self.refresh();
  }
  fn reset_statistics(&self) {
    let alert = NSAlert::new(self.mtm());
    alert.setMessageText(&NSString::from_str("Delete all activity history?"));
    alert.setInformativeText(&NSString::from_str(
      "This cannot be undone. Settings and the current timer are not changed.",
    ));
    alert.addButtonWithTitle(&NSString::from_str("Cancel"));
    alert.addButtonWithTitle(&NSString::from_str("Delete history"));
    if alert.runModal() == 1001 {
      let mut state = self.ivars().state.borrow_mut();
      state.app.status = match state.app.stats.reset() {
        Ok(()) => "Statistics cleared".into(),
        Err(e) => format!("Reset failed: {e}"),
      };
    }
    self.refresh();
  }
}
unsafe extern "C" fn hotkey(
  _: *mut std::ffi::c_void,
  event: *mut std::ffi::c_void,
  data: *mut std::ffi::c_void,
) -> i32 {
  // SAFETY: Carbon calls on the application event thread; data is our retained Controller.
  let controller = unsafe { &*data.cast::<Controller>() };
  if crate::app::benchmark_locked() {
    return 0;
  }
  let Some(id) = (unsafe { crate::hotkeys::key_id(event) }) else {
    return -9874;
  };
  if id == 4 {
    controller.show();
    return 0;
  }
  {
    let mut state = controller.ivars().state.borrow_mut();
    match id {
      0 => state.app.toggle(),
      1 => state.app.restart(),
      2 => state.app.skip(),
      3 => state.app.stop(),
      _ => {}
    }
  }
  controller.refresh();
  0
}
pub fn run() {
  let mtm = MainThreadMarker::new().expect("main thread");
  let dir = config::var_os("RUSTY_POMODORO_CONFIG_DIR")
    .map(std::path::PathBuf::from)
    .unwrap_or_else(|| config::config_dir("tomito-rs"));
  let settings = Settings::load(&dir.join("settings.conf"));
  let controller = Controller::new(
    mtm,
    App::new(
      settings,
      dir.join("settings.conf"),
      StatsStore::new(dir.join("activities.csv")),
    ),
  );
  let app = NSApplication::sharedApplication(mtm);
  app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
  app.setDelegate(Some(ProtocolObject::from_ref(&*controller)));
  app.run();
  // SAFETY: registered observer remains alive until after removal.
  unsafe {
    NSWorkspace::sharedWorkspace()
      .notificationCenter()
      .removeObserver(&controller);
  }
  if let Some(views) = controller.ivars().views.get() {
    NSStatusBar::systemStatusBar().removeStatusItem(&views.item);
  }
}
