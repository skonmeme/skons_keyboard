//! The InputMethodKit controller. IMK creates one controller per client.

use std::cell::{Cell, RefCell};
use std::ffi::c_long;

use objc2::rc::{Allocated, Retained};
use objc2::runtime::{AnyObject, NSObject, NSObjectProtocol};
use objc2::{DefinedClass, define_class, msg_send};
use objc2_app_kit::{NSEvent, NSEventMask, NSEventModifierFlags, NSEventType};
use objc2_foundation::{NSString, NSUInteger};
use objc2_input_method_kit::{IMKInputController, IMKServer};

use super::client::Client;
use super::{
    KEYBOARD_LAYOUT, TERMINALS, current_mode, mode_from_id, mode_id, set_current_mode, system,
};
use crate::engine::{ClientCall, Engine, Environment, KeyDown, Mode, Modifiers, keycode};

/// `kTextServiceInputModePropertyTag` ('imim').
const INPUT_MODE_TAG: c_long = 0x696D_696D;

pub struct Ivars {
    engine: RefCell<Engine>,
    /// True while a key event is being handled. Commits and mode changes
    /// that the system requests meanwhile wait until it ends (§7.2 rule 4).
    in_key_event: Cell<bool>,
    pending_commit: Cell<bool>,
    pending_mode: Cell<Option<Mode>>,
}

define_class!(
    // SAFETY: IMKInputController has no subclassing requirements beyond
    // calling its designated initializer, and the class does not implement
    // Drop.
    #[unsafe(super(IMKInputController, NSObject))]
    #[name = "SKInputController"]
    #[ivars = Ivars]
    pub struct InputController;

    impl InputController {
        #[unsafe(method_id(initWithServer:delegate:client:))]
        fn init_with_server(
            this: Allocated<Self>,
            server: Option<&IMKServer>,
            delegate: Option<&AnyObject>,
            client: Option<&AnyObject>,
        ) -> Option<Retained<Self>> {
            let this = this.set_ivars(Ivars {
                engine: RefCell::new(Engine::new(current_mode())),
                in_key_event: Cell::new(false),
                pending_commit: Cell::new(false),
                pending_mode: Cell::new(None),
            });
            unsafe { msg_send![super(this), initWithServer: server, delegate: delegate, client: client] }
        }

        #[unsafe(method(handleEvent:client:))]
        fn handle_event(&self, event: Option<&NSEvent>, sender: Option<&AnyObject>) -> bool {
            match (event, sender) {
                (Some(event), Some(sender)) => self.key_event(event, Client(sender)),
                _ => false,
            }
        }

        #[unsafe(method(recognizedEvents:))]
        fn recognized_events(&self, _sender: Option<&AnyObject>) -> NSUInteger {
            // Key down only, so that IMK commits on mouse clicks (§7.2 rule 6).
            NSEventMask::KeyDown.0 as NSUInteger
        }

        #[unsafe(method(activateServer:))]
        fn activate_server(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                let client = Client(sender);
                client.override_keyboard(KEYBOARD_LAYOUT);
                self.change_mode(current_mode(), client);
            }
        }

        #[unsafe(method(deactivateServer:))]
        fn deactivate_server(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                self.commit(Client(sender));
            }
        }

        #[unsafe(method(commitComposition:))]
        fn commit_composition(&self, sender: Option<&AnyObject>) {
            if let Some(sender) = sender {
                self.commit(Client(sender));
            }
        }

        #[unsafe(method(setValue:forTag:client:))]
        fn set_value(&self, value: Option<&AnyObject>, tag: c_long, sender: Option<&AnyObject>) {
            if tag != INPUT_MODE_TAG {
                return;
            }
            let mode = value
                .and_then(|value| value.downcast_ref::<NSString>())
                .and_then(|id| mode_from_id(&id.to_string()));
            if let (Some(mode), Some(sender)) = (mode, sender) {
                // §3: follow the system, never request a change in response.
                set_current_mode(mode);
                let client = Client(sender);
                client.override_keyboard(KEYBOARD_LAYOUT);
                self.change_mode(mode, client);
            }
        }
    }

    unsafe impl NSObjectProtocol for InputController {}
);

/// Facts for §4.5, gathered before the engine runs so that no client call
/// happens while the engine is borrowed.
struct Facts {
    terminal: bool,
    can_post_events: bool,
}

impl Environment for Facts {
    fn is_terminal(&self) -> bool {
        self.terminal
    }
    fn can_post_events(&self) -> bool {
        self.can_post_events
    }
}

impl InputController {
    fn key_event(&self, event: &NSEvent, client: Client<'_>) -> bool {
        if event.r#type() != NSEventType::KeyDown {
            return false;
        }
        // A key posted by §4.5 passes through without other handling.
        if system::is_reposted(event) {
            return false;
        }
        let key = key_down(event);
        let ivars = self.ivars();
        let facts = if (key.code == keycode::RETURN || key.code == keycode::ESCAPE)
            && ivars.engine.borrow().is_composing()
        {
            Facts {
                terminal: client
                    .bundle_identifier()
                    .is_some_and(|id| TERMINALS.contains(&id.as_str())),
                can_post_events: system::can_post_events(),
            }
        } else {
            Facts {
                terminal: false,
                can_post_events: false,
            }
        };

        ivars.in_key_event.set(true);
        let response = ivars.engine.borrow_mut().key_down(key, &facts);
        apply(client, &response.calls);
        if let Some(mode) = response.switch_to {
            set_current_mode(mode);
            client.select_mode(mode_id(mode));
        }
        if let Some(code) = response.repost {
            system::post_key(code);
        }
        ivars.in_key_event.set(false);

        // Run what the system asked for while the key was being handled.
        if let Some(mode) = ivars.pending_mode.take() {
            self.change_mode(mode, client);
        }
        if ivars.pending_commit.take() {
            self.commit(client);
        }
        response.consumed
    }

    fn commit(&self, client: Client<'_>) {
        let ivars = self.ivars();
        if ivars.in_key_event.get() {
            ivars.pending_commit.set(true);
            return;
        }
        let calls = ivars.engine.borrow_mut().commit();
        apply(client, &calls);
    }

    fn change_mode(&self, mode: Mode, client: Client<'_>) {
        let ivars = self.ivars();
        if ivars.in_key_event.get() {
            ivars.pending_mode.set(Some(mode));
            return;
        }
        let calls = ivars.engine.borrow_mut().set_mode(mode);
        apply(client, &calls);
    }
}

fn apply(client: Client<'_>, calls: &[ClientCall]) {
    for call in calls {
        client.apply(call);
    }
}

fn key_down(event: &NSEvent) -> KeyDown {
    let flags = event.modifierFlags();
    KeyDown {
        code: event.keyCode(),
        modifiers: Modifiers {
            shift: flags.contains(NSEventModifierFlags::Shift),
            control: flags.contains(NSEventModifierFlags::Control),
            option: flags.contains(NSEventModifierFlags::Option),
            command: flags.contains(NSEventModifierFlags::Command),
            function: flags.contains(NSEventModifierFlags::Function),
        },
        repeat: event.isARepeat(),
    }
}
