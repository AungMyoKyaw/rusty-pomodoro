//! Five Carbon hotkeys, without a keyboard tap, polling, or background thread.
use std::cell::Cell;
use std::ffi::c_void;
type Handle = *mut c_void;
#[repr(C)]
struct EventType {
    class: u32,
    kind: u32,
}
#[repr(C)]
#[derive(Default)]
struct KeyId {
    signature: u32,
    id: u32,
}
pub type Callback = unsafe extern "C" fn(Handle, Handle, Handle) -> i32;
#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn GetApplicationEventTarget() -> Handle;
    fn InstallEventHandler(
        target: Handle,
        handler: Callback,
        count: u32,
        events: *const EventType,
        data: Handle,
        out: *mut Handle,
    ) -> i32;
    fn RemoveEventHandler(handler: Handle) -> i32;
    fn RegisterEventHotKey(
        code: u32,
        modifiers: u32,
        id: KeyId,
        target: Handle,
        options: u32,
        out: *mut Handle,
    ) -> i32;
    fn UnregisterEventHotKey(key: Handle) -> i32;
    fn GetEventParameter(
        event: Handle,
        name: u32,
        kind: u32,
        actual_kind: *mut u32,
        size: u32,
        actual_size: *mut u32,
        data: Handle,
    ) -> i32;
}
pub struct Hotkeys {
    handler: Handle,
    keys: [Handle; 5],
    active: Cell<bool>,
    pub failed: bool,
}
impl Hotkeys {
    /// Caller keeps callback data alive until cleanup and stays on AppKit's main thread.
    pub unsafe fn new(callback: Callback, data: Handle) -> Self {
        let mut this = Self {
            handler: std::ptr::null_mut(),
            keys: [std::ptr::null_mut(); 5],
            active: Cell::new(true),
            failed: false,
        };
        let event = EventType {
            class: u32::from_be_bytes(*b"keyb"),
            kind: 5,
        };
        // SAFETY: event/handles are writable local storage; callback/data lifetime is caller's contract.
        let target = unsafe { GetApplicationEventTarget() };
        let code =
            unsafe { InstallEventHandler(target, callback, 1, &event, data, &mut this.handler) };
        if code != 0 {
            this.failed = true;
            return this;
        }
        for (i, key) in [49, 15, 1, 7, 17].into_iter().enumerate() {
            // Control (4096) + Option (2048): Space, R, S, X, T.
            let id = KeyId {
                signature: u32::from_be_bytes(*b"TMRs"),
                id: i as u32,
            };
            let code =
                unsafe { RegisterEventHotKey(key, 4096 | 2048, id, target, 0, &mut this.keys[i]) };
            if code != 0 {
                this.failed = true;
            }
        }
        this
    }
    pub fn cleanup(&self) {
        if !self.active.replace(false) {
            return;
        }
        for &key in &self.keys {
            if !key.is_null() {
                unsafe {
                    UnregisterEventHotKey(key);
                }
            }
        }
        if !self.handler.is_null() {
            unsafe {
                RemoveEventHandler(self.handler);
            }
        }
    }
}
impl Drop for Hotkeys {
    fn drop(&mut self) {
        self.cleanup();
    }
}
/// Extract only our registered hotkey ID from the Carbon event.
pub unsafe fn key_id(event: Handle) -> Option<u32> {
    let mut id = KeyId::default();
    // SAFETY: 8-byte KeyId matches Carbon's EventHotKeyID ABI; output points to initialized storage.
    let code = unsafe {
        GetEventParameter(
            event,
            u32::from_be_bytes(*b"----"),
            u32::from_be_bytes(*b"hkid"),
            std::ptr::null_mut(),
            8,
            std::ptr::null_mut(),
            (&mut id as *mut KeyId).cast(),
        )
    };
    (code == 0 && id.signature == u32::from_be_bytes(*b"TMRs") && id.id < 5).then_some(id.id)
}
