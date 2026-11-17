//! Number and password fields in Safari and Firefox (§3.1): focus
//! notifications of the Accessibility API, and the mode changes they cause.
//!
//! The state lives on the main thread. Requests to a browser run on a serial
//! dispatch queue, because a browser may be waiting for the input method on
//! its main thread (for example in `handleEvent`) and could not answer.

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::ptr;

use dispatch2::{DispatchQueue, DispatchRetained};
use objc2::rc::{Retained, autoreleasepool};
use objc2::runtime::AnyObject;
use objc2_app_kit::NSWorkspace;
use objc2_core_foundation::{CFRunLoop, CFRunLoopSource, kCFRunLoopDefaultMode};
use objc2_foundation::{NSArray, NSString};

use super::{controller, current_mode, system};
use crate::engine::Mode;
use crate::field::{self, Fields, Focus};

/// The browsers where §3.1 applies.
const BROWSERS: [&str; 2] = ["com.apple.Safari", "org.mozilla.firefox"];

/// How long a request to a browser may take before it fails, in seconds.
const MESSAGING_TIMEOUT: f32 = 1.0;

const AX_SUCCESS: i32 = 0;
/// `kAXErrorNoValue`: the attribute has no value, for example nothing has
/// focus.
const AX_NO_VALUE: i32 = -25212;

type ObserverCallback = unsafe extern "C" fn(
    observer: *mut c_void,
    element: *mut c_void,
    notification: *const c_void,
    refcon: *mut c_void,
);

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> *mut c_void;
    fn AXUIElementCreateSystemWide() -> *mut c_void;
    fn AXUIElementCopyAttributeValue(
        element: *const c_void,
        attribute: *const c_void,
        value: *mut *mut c_void,
    ) -> i32;
    fn AXUIElementSetMessagingTimeout(element: *const c_void, timeout: f32) -> i32;
    fn AXObserverCreate(pid: i32, callback: ObserverCallback, observer: *mut *mut c_void) -> i32;
    fn AXObserverAddNotification(
        observer: *const c_void,
        element: *const c_void,
        notification: *const c_void,
        refcon: *mut c_void,
    ) -> i32;
    fn AXObserverGetRunLoopSource(observer: *const c_void) -> *mut c_void;
}

/// An `AXUIElement` or `AXObserver`. Core Foundation objects work as
/// Objective-C objects for retain and release.
struct Ax(Retained<AnyObject>);

// SAFETY: these objects only refer to another process's interface, and the
// Accessibility API may use them from any thread.
unsafe impl Send for Ax {}

impl Ax {
    /// Takes ownership of an object returned with +1 retain count.
    fn from_owned(object: *mut c_void) -> Option<Ax> {
        unsafe { Retained::from_raw(object.cast()) }.map(Ax)
    }

    fn as_ptr(&self) -> *const c_void {
        Retained::as_ptr(&self.0).cast()
    }
}

/// The browser whose focus changes are observed.
struct Watched {
    pid: i32,
    observer: Ax,
}

impl Watched {
    fn source(&self) -> Option<&CFRunLoopSource> {
        unsafe {
            AXObserverGetRunLoopSource(self.observer.as_ptr())
                .cast::<CFRunLoopSource>()
                .as_ref()
        }
    }
}

thread_local! {
    static FIELDS: RefCell<Fields> = RefCell::new(Fields::default());
    static WATCHED: RefCell<Option<Watched>> = const { RefCell::new(None) };
    /// The mode to select when a client is next activated.
    static PENDING: Cell<Option<Mode>> = const { Cell::new(None) };
    /// The number of the latest focus check. Older results are dropped.
    static CHECK: Cell<u64> = const { Cell::new(0) };
    static QUEUE: DispatchRetained<DispatchQueue> =
        DispatchQueue::new("com.skon.inputmethod.skonsKeyboard.fields", None);
}

/// Limits how long the input method waits for a browser (§3.1).
pub fn init() {
    if let Some(system_wide) = Ax::from_owned(unsafe { AXUIElementCreateSystemWide() }) {
        unsafe { AXUIElementSetMessagingTimeout(system_wide.as_ptr(), MESSAGING_TIMEOUT) };
    }
}

/// Focus is in a number or password field of a browser.
pub fn in_english_field() -> bool {
    FIELDS.with_borrow(Fields::in_english_field)
}

/// The system selected `new` while `previous` was selected (§3).
pub fn system_changed(previous: Mode, new: Mode) {
    if previous != new {
        PENDING.set(None);
    }
    FIELDS.with_borrow_mut(|fields| fields.system_changed(previous, new));
}

/// Shift+Space switched modes.
pub fn shift_space() {
    FIELDS.with_borrow_mut(Fields::shift_space);
}

/// A client of the input method was activated. Returns the mode to select
/// through it. Makes no request to the browser: the client may be waiting.
pub fn activated(bundle_id: Option<&str>) -> Option<Mode> {
    let pending = PENDING.take();
    if bundle_id.is_some_and(|id| BROWSERS.contains(&id)) {
        DispatchQueue::main().exec_async(watch_frontmost);
        return pending;
    }
    // Another application: focus left the browser.
    FIELDS
        .with_borrow_mut(|fields| fields.focus(Focus::Other, current_mode()))
        .or(pending)
}

fn frontmost_browser() -> Option<i32> {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let id = app.bundleIdentifier()?.to_string();
    BROWSERS
        .contains(&id.as_str())
        .then(|| app.processIdentifier())
}

/// Checks the focused element of the frontmost browser, and starts
/// observing it if needed.
fn watch_frontmost() {
    if !system::can_read_focus() {
        return;
    }
    let Some(pid) = frontmost_browser() else {
        return;
    };
    let watched = WATCHED.with_borrow(|watched| watched.as_ref().is_some_and(|w| w.pid == pid));
    check_focus(pid, !watched);
}

/// Called on the main run loop for `AXFocusedUIElementChanged`. The element
/// of the notification may be a window or a group, so the focused element is
/// read again (§3.1).
unsafe extern "C" fn focus_changed(
    _observer: *mut c_void,
    _element: *mut c_void,
    _notification: *const c_void,
    refcon: *mut c_void,
) {
    check_focus(refcon as isize as i32, false);
}

fn check_focus(pid: i32, observe: bool) {
    let check = CHECK.get() + 1;
    CHECK.set(check);
    QUEUE.with(|queue| {
        queue.exec_async(move || {
            let (observer, focus) = autoreleasepool(|_| {
                let observer = if observe { observe_browser(pid) } else { None };
                (observer, focused(pid))
            });
            DispatchQueue::main().exec_async(move || finish_check(pid, check, observer, focus));
        });
    });
}

fn finish_check(pid: i32, check: u64, observer: Option<Ax>, focus: Option<Focus>) {
    if let Some(observer) = observer {
        watch(pid, observer);
    }
    // Only the latest check of the active browser counts, and only while one
    // of the input method's modes is selected.
    if check != CHECK.get() || frontmost_browser() != Some(pid) || !system::mode_selected() {
        return;
    }
    let Some(focus) = focus else {
        return;
    };
    if let Some(mode) = FIELDS.with_borrow_mut(|fields| fields.focus(focus, current_mode()))
        && !controller::select_through_active(mode)
    {
        PENDING.set(Some(mode));
    }
}

/// Puts the observer's source on the main run loop, replacing the observer
/// of another browser.
fn watch(pid: i32, observer: Ax) {
    let (Some(run_loop), Some(mode)) = (CFRunLoop::main(), unsafe { kCFRunLoopDefaultMode }) else {
        return;
    };
    let new = Watched { pid, observer };
    WATCHED.with_borrow_mut(|watched| {
        if let Some(old) = watched.take() {
            run_loop.remove_source(old.source(), Some(mode));
        }
        run_loop.add_source(new.source(), Some(mode));
        *watched = Some(new);
    });
}

// Requests to the browser, on the dispatch queue.

fn application(pid: i32) -> Option<Ax> {
    Ax::from_owned(unsafe { AXUIElementCreateApplication(pid) })
}

fn observe_browser(pid: i32) -> Option<Ax> {
    let mut observer = ptr::null_mut();
    if unsafe { AXObserverCreate(pid, focus_changed, &mut observer) } != AX_SUCCESS {
        return None;
    }
    let observer = Ax::from_owned(observer)?;
    let app = application(pid)?;
    let name = NSString::from_str("AXFocusedUIElementChanged");
    let refcon = pid as isize as *mut c_void;
    let status = unsafe {
        AXObserverAddNotification(observer.as_ptr(), app.as_ptr(), ns_ptr(&name), refcon)
    };
    (status == AX_SUCCESS).then_some(observer)
}

/// What has focus in the browser, or `None` if it did not answer.
fn focused(pid: i32) -> Option<Focus> {
    let app = application(pid)?;
    let element = match attribute(&app, "AXFocusedUIElement") {
        Ok(Some(element)) => Ax(element),
        Ok(None) => return Some(Focus::Other),
        Err(()) => return None,
    };
    let role = string_attribute(&element, "AXRole");
    let subrole = string_attribute(&element, "AXSubrole");
    let mut child_roles = Vec::new();
    if field::needs_child_roles(&role, &subrole)
        && let Ok(Some(children)) = attribute(&element, "AXChildren")
        && let Ok(children) = children.downcast::<NSArray>()
    {
        for child in children.to_vec() {
            child_roles.push(string_attribute(&Ax(child), "AXRole"));
        }
    }
    Some(field::classify(&role, &subrole, &child_roles))
}

/// The value of an attribute: `Ok(None)` if it has no value.
fn attribute(element: &Ax, name: &str) -> Result<Option<Retained<AnyObject>>, ()> {
    let name = NSString::from_str(name);
    let mut value = ptr::null_mut();
    match unsafe { AXUIElementCopyAttributeValue(element.as_ptr(), ns_ptr(&name), &mut value) } {
        AX_SUCCESS => Ok(unsafe { Retained::from_raw(value.cast()) }),
        AX_NO_VALUE => Ok(None),
        _ => Err(()),
    }
}

/// A string attribute such as the role, or an empty string.
fn string_attribute(element: &Ax, name: &str) -> String {
    match attribute(element, name) {
        Ok(Some(value)) => value
            .downcast::<NSString>()
            .map(|s| s.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// An `NSString` as a `CFStringRef` (toll-free bridged).
fn ns_ptr(string: &NSString) -> *const c_void {
    ptr::from_ref(string).cast()
}
