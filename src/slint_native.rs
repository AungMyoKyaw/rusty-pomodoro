//! Optional macOS integration for the shared UI. Other platforms keep the same UI.
#[cfg(target_os = "macos")]
mod mac {
    use crate::{app::App, config::Sound, hotkeys::Hotkeys};
    use objc2::rc::Retained;
    use objc2::{define_class, msg_send, sel, AnyThread, DefinedClass, MainThreadOnly};
    use objc2_app_kit::{
        NSApplication, NSApplicationActivationPolicy, NSImage, NSMenu, NSMenuItem, NSSound,
        NSStatusBar, NSStatusItem, NSWorkspace, NSWorkspaceDidWakeNotification,
        NSWorkspaceWillSleepNotification,
    };
    use objc2_foundation::{
        MainThreadMarker, NSAppleEventDescriptor, NSAppleEventManager, NSData, NSNotification,
        NSObject, NSObjectProtocol, NSString,
    };
    use std::{cell::Cell, rc::Rc, time::Duration};
    const CORE_EVENT: u32 = u32::from_be_bytes(*b"aevt");
    const REOPEN_EVENT: u32 = u32::from_be_bytes(*b"rapp");
    pub struct Ivars {
        callback: Rc<dyn Fn(i32)>,
    }
    define_class!(
        // SAFETY: NSObject imposes no subclass requirements; callbacks are main-thread-only.
        #[unsafe(super=NSObject)]
        #[thread_kind=MainThreadOnly]
        #[ivars=Ivars]
        struct Target;
        unsafe impl NSObjectProtocol for Target {}
        impl Target {
            #[unsafe(method(perform:))]
            fn perform(&self,item:&NSMenuItem){
                let id=item.tag() as i32;
                if id==6 || id==7 {(self.ivars().callback)(20);}
                (self.ivars().callback)(id);
            }
            // Handle only Dock/LaunchServices reopen, leaving Winit's delegate intact.
            #[unsafe(method(reopen:replyEvent:))]
            fn reopen(&self,_:&NSAppleEventDescriptor,_:&NSAppleEventDescriptor){
                let callback = self.ivars().callback.clone();
                slint::Timer::single_shot(Duration::ZERO, move || callback(20));
            }
            #[unsafe(method(willSleep:))]
            fn sleep(&self,_:&NSNotification){(self.ivars().callback)(21);}
            #[unsafe(method(didWake:))]
            fn wake(&self,_:&NSNotification){(self.ivars().callback)(22);}
        }
    );
    impl Target {
        fn new(mtm: MainThreadMarker, callback: Rc<dyn Fn(i32)>) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(Ivars { callback });
            // SAFETY: NSObject init signature is correct.
            unsafe { msg_send![super(this), init] }
        }
    }
    unsafe extern "C" fn hotkey(
        _: *mut std::ffi::c_void,
        event: *mut std::ffi::c_void,
        data: *mut std::ffi::c_void,
    ) -> i32 {
        // SAFETY: callback installed for this retained Target on application's main thread.
        let target = unsafe { &*data.cast::<Target>() };
        let Some(id) = (unsafe { crate::hotkeys::key_id(event) }) else {
            return -9874;
        };
        (target.ivars().callback)([0, 1, 2, 3, 20][id as usize]);
        0
    }
    pub struct Native {
        item: Retained<NSStatusItem>,
        target: Retained<Target>,
        toggle: Retained<NSMenuItem>,
        keys: Hotkeys,
        reopen_installed: Cell<bool>,
    }
    impl Native {
        pub fn new(callback: impl Fn(i32) + 'static) -> Self {
            let mtm = MainThreadMarker::new().expect("main thread");
            let target = Target::new(mtm, Rc::new(callback));
            let item = NSStatusBar::systemStatusBar().statusItemWithLength(-1.0);
            let menu = NSMenu::new(mtm);
            menu.setAutoenablesItems(false);
            let mut toggle = None;
            for (title, tag) in [
                ("Start / Pause", 0),
                ("Restart", 1),
                ("Skip", 2),
                ("Stop", 3),
                ("Finish overtime", 4),
                ("Show timer", 20),
                ("Settings...", 6),
                ("Statistics...", 7),
                ("Reset cycle", 8),
                ("Quit", 9),
            ] {
                // SAFETY: perform: signature is implemented; target retained until menu teardown.
                let action = unsafe {
                    NSMenuItem::initWithTitle_action_keyEquivalent(
                        NSMenuItem::alloc(mtm),
                        &NSString::from_str(title),
                        Some(sel!(perform:)),
                        &NSString::from_str(""),
                    )
                };
                action.setTag(tag);
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
            // SAFETY: Target is retained by Native until keys are removed in Drop.
            let keys =
                unsafe { Hotkeys::new(hotkey, (&*target as *const Target).cast_mut().cast()) };
            if keys.failed {
                eprintln!(
                    "Some global shortcuts are already in use; menu controls remain available"
                );
            }
            Self {
                item,
                target,
                toggle: toggle.unwrap(),
                keys,
                reopen_installed: Cell::new(false),
            }
        }
        pub fn install_dock_handler(&self) {
            let application =
                NSApplication::sharedApplication(MainThreadMarker::new().expect("main thread"));
            // Changing policy before finishLaunching can fail; apply it on the startup event-loop callback.
            application.setActivationPolicy(NSApplicationActivationPolicy::Regular);
            assert_eq!(
                application.activationPolicy(),
                NSApplicationActivationPolicy::Regular,
                "Dock activation policy"
            );
            let image = NSImage::initWithData(
                NSImage::alloc(),
                &NSData::with_bytes(include_bytes!("../assets/tomito-icon-256.png")),
            )
            .expect("valid bundled app icon");
            // SAFETY: provide a valid retained image, never nil; NSApplication retains it.
            unsafe {
                application.setApplicationIconImage(Some(&image));
            }
            // Install after AppKit finishes launching, so its default registration cannot replace ours.
            // SAFETY: selector accepts the documented descriptors; Native retains its Target until removal.
            unsafe {
                NSAppleEventManager::sharedAppleEventManager()
                    .setEventHandler_andSelector_forEventClass_andEventID(
                        &self.target,
                        sel!(reopen:replyEvent:),
                        CORE_EVENT,
                        REOPEN_EVENT,
                    );
            }
            self.reopen_installed.set(true);
        }
        pub fn update(&self, app: &App) {
            let seconds = app.display_seconds();
            let title = NSString::from_str(&format!(
                "{}{:02}:{:02}",
                if app.is_overtime() { "+" } else { "" },
                seconds / 60,
                seconds % 60
            ));
            if let Some(button) = self.item.button(MainThreadMarker::new().unwrap()) {
                if button.title() != title {
                    button.setTitle(&title);
                }
            }
            self.toggle.setTitle(&NSString::from_str(
                if app.timer.state() == crate::timer::RunState::Running {
                    "Pause"
                } else {
                    "Start / Resume"
                },
            ));
        }
    }
    impl Drop for Native {
        fn drop(&mut self) {
            self.keys.cleanup();
            if self.reopen_installed.get() {
                NSAppleEventManager::sharedAppleEventManager()
                    .removeEventHandlerForEventClass_andEventID(CORE_EVENT, REOPEN_EVENT);
            }
            self.item.setMenu(None);
            unsafe {
                NSWorkspace::sharedWorkspace()
                    .notificationCenter()
                    .removeObserver(&self.target);
            }
            NSStatusBar::systemStatusBar().removeStatusItem(&self.item);
        }
    }
    pub fn play(sound: Sound) {
        if sound == Sound::System {
            objc2_app_kit::NSBeep();
        } else if sound != Sound::None {
            if let Some(audio) = NSSound::soundNamed(&NSString::from_str(sound.label())) {
                audio.play();
            }
        }
    }
    pub fn activate() {
        #[allow(deprecated)]
        NSApplication::sharedApplication(MainThreadMarker::new().unwrap())
            .activateIgnoringOtherApps(true);
    }
}
#[cfg(target_os = "macos")]
pub use mac::{activate, play, Native};
#[cfg(not(target_os = "macos"))]
pub struct Native;
#[cfg(not(target_os = "macos"))]
impl Native {
    pub fn new(_: impl Fn(i32) + 'static) -> Self {
        Self
    }
    pub fn install_dock_handler(&self) {}
    pub fn update(&self, _: &crate::app::App) {}
}
#[cfg(not(target_os = "macos"))]
pub fn play(_: crate::config::Sound) {}
#[cfg(not(target_os = "macos"))]
pub fn activate() {}
