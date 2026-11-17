//! System services: posting keys (§4.5), the Accessibility permission, and
//! input source registration used by the installer (§8.1).

use std::ffi::c_void;
use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::NSEvent;
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventSource, CGEventSourceStateID, CGEventTapLocation,
    CGPreflightPostEventAccess, CGRequestPostEventAccess,
};
use objc2_foundation::{NSArray, NSBundle, NSDictionary, NSNumber, NSString};

use super::{ENGLISH_MODE_ID, KOREAN_MODE_ID};

/// Marks the key events this input method posts ('SKRP').
const REPOST_MARKER: i64 = 0x534B_5250;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    static kAXTrustedCheckOptionPrompt: *const c_void;
    fn AXIsProcessTrusted() -> u8;
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> u8;
}

/// Whether the input method has the Accessibility permission needed to post
/// events. Never asks for it (§4.5).
pub fn can_post_events() -> bool {
    CGPreflightPostEventAccess()
}

/// Whether the input method has the Accessibility permission needed to read
/// the focused element of other applications. Never asks for it (§3.1).
pub fn can_read_focus() -> bool {
    unsafe { AXIsProcessTrusted() != 0 }
}

/// Posts a marked key down and key up for the key (§4.5).
pub fn post_key(code: u16) {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState);
    for key_down in [true, false] {
        if let Some(event) = CGEvent::new_keyboard_event(source.as_deref(), code, key_down) {
            CGEvent::set_integer_value_field(
                Some(&event),
                CGEventField::EventSourceUserData,
                REPOST_MARKER,
            );
            CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
        }
    }
}

/// Whether the event is one that `post_key` posted.
pub fn is_reposted(event: &NSEvent) -> bool {
    event.CGEvent().is_some_and(|event| {
        CGEvent::integer_value_field(Some(&event), CGEventField::EventSourceUserData)
            == REPOST_MARKER
    })
}

/// Permission request mode (§8.2): makes macOS ask the user to allow this
/// input method to use Accessibility, for reading the focused element (§3.1)
/// and for posting key events (§4.5).
pub fn request_accessibility() {
    let mut asked = false;
    if !can_read_focus() {
        let key: &NSString = unsafe { &*kAXTrustedCheckOptionPrompt.cast() };
        let prompt = NSNumber::new_bool(true);
        let options = NSDictionary::from_slices(&[key], &[&*prompt]);
        unsafe { AXIsProcessTrustedWithOptions(Retained::as_ptr(&options).cast()) };
        asked = true;
    }
    if !CGPreflightPostEventAccess() {
        CGRequestPostEventAccess();
        asked = true;
    }
    if asked {
        // Give the system time to show its prompt before the process exits.
        std::thread::sleep(Duration::from_secs(3));
    }
}

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    static kTISPropertyInputSourceID: *const c_void;
    fn TISRegisterInputSource(location: *const c_void) -> i32;
    fn TISCreateInputSourceList(
        properties: *const c_void,
        include_all_installed: u8,
    ) -> *mut c_void;
    fn TISEnableInputSource(source: *const c_void) -> i32;
    fn TISDisableInputSource(source: *const c_void) -> i32;
    fn TISCopyCurrentKeyboardInputSource() -> *mut c_void;
    fn TISGetInputSourceProperty(source: *const c_void, key: *const c_void) -> *const c_void;
}

/// Whether one of this input method's modes is selected in the system
/// (§3.1).
pub fn mode_selected() -> bool {
    // The current source is returned with +1 ownership.
    let source: Option<Retained<AnyObject>> =
        unsafe { Retained::from_raw(TISCopyCurrentKeyboardInputSource().cast()) };
    let Some(source) = source else {
        return false;
    };
    let id = unsafe {
        TISGetInputSourceProperty(Retained::as_ptr(&source).cast(), kTISPropertyInputSourceID)
    };
    let id: Option<&NSString> = unsafe { id.cast::<NSString>().as_ref() };
    id.is_some_and(|id| [KOREAN_MODE_ID, ENGLISH_MODE_ID].contains(&id.to_string().as_str()))
}

/// The input sources with this ID. Core Foundation types are toll-free
/// bridged to their Foundation counterparts.
fn input_sources(id: &str) -> Vec<Retained<AnyObject>> {
    let key: &NSString = unsafe { &*kTISPropertyInputSourceID.cast() };
    let value = NSString::from_str(id);
    let properties = NSDictionary::from_slices(&[key], &[&*value]);
    let list = unsafe { TISCreateInputSourceList(Retained::as_ptr(&properties).cast(), 1) };
    // The list is returned with +1 ownership.
    let list: Option<Retained<NSArray<AnyObject>>> = unsafe { Retained::from_raw(list.cast()) };
    list.map(|list| list.to_vec()).unwrap_or_default()
}

fn for_each_mode(
    action: unsafe extern "C" fn(*const c_void) -> i32,
    name: &str,
) -> Result<(), String> {
    for id in [KOREAN_MODE_ID, ENGLISH_MODE_ID] {
        let sources = input_sources(id);
        if sources.is_empty() {
            return Err(format!("input source {id} not found"));
        }
        for source in sources {
            let status = unsafe { action(Retained::as_ptr(&source).cast()) };
            if status != 0 {
                return Err(format!("{name} {id}: error {status}"));
            }
        }
    }
    Ok(())
}

/// Registers this bundle and enables both modes for the current user (§8.1).
pub fn register() -> Result<(), String> {
    let url = NSBundle::mainBundle().bundleURL();
    let status = unsafe { TISRegisterInputSource(Retained::as_ptr(&url).cast()) };
    if status != 0 {
        return Err(format!("TISRegisterInputSource: error {status}"));
    }
    for_each_mode(TISEnableInputSource, "TISEnableInputSource")
}

/// Disables both modes for the current user (§8.3).
pub fn unregister() -> Result<(), String> {
    for_each_mode(TISDisableInputSource, "TISDisableInputSource")
}
