//! Key handling of both modes (§3, §4.4, §4.5, §5), independent of macOS.

use crate::composer::Composer;
use crate::layout::{self, Output};

/// macOS virtual key codes used outside the layout table.
pub mod keycode {
    pub const RETURN: u16 = 0x24;
    pub const SPACE: u16 = 0x31;
    pub const BACKSPACE: u16 = 0x33;
    pub const ESCAPE: u16 = 0x35;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Korean,
    English,
}

impl Mode {
    pub fn other(self) -> Mode {
        match self {
            Mode::Korean => Mode::English,
            Mode::English => Mode::Korean,
        }
    }
}

/// Modifier keys held during a key press. Caps Lock is not listed because it
/// is ignored (§3, §4.4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub option: bool,
    pub command: bool,
    pub function: bool,
}

impl Modifiers {
    fn any_but_shift(self) -> bool {
        self.control || self.option || self.command || self.function
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyDown {
    pub code: u16,
    pub modifiers: Modifiers,
    /// True for an auto-repeated key down.
    pub repeat: bool,
}

/// Facts about the client and the system that §4.5 needs. They are asked
/// only when §4.5 could apply.
pub trait Environment {
    /// The client is Terminal or Ghostty.
    fn is_terminal(&self) -> bool;
    /// The input method has the Accessibility permission.
    fn can_post_events(&self) -> bool;
}

/// A call the input method makes to the client, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientCall {
    /// Insert final text at the selection (replacing the marked text).
    Insert(String),
    /// Show the composition as marked text.
    SetMarked(String),
    /// Remove the marked text without inserting anything.
    ClearMarked,
}

/// What to do with a key down event.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Response {
    pub calls: Vec<ClientCall>,
    /// True if the event is consumed; false if it is passed through.
    pub consumed: bool,
    /// The mode to select in the system (Shift+Space).
    pub switch_to: Option<Mode>,
    /// A key to post again after the event (§4.5).
    pub repost: Option<u16>,
}

#[derive(Debug)]
pub struct Engine {
    mode: Mode,
    composer: Composer,
}

impl Engine {
    pub fn new(mode: Mode) -> Engine {
        Engine {
            mode,
            composer: Composer::default(),
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn is_composing(&self) -> bool {
        !self.composer.is_empty()
    }

    /// Handles a key down event.
    pub fn key_down(&mut self, key: KeyDown, env: &impl Environment) -> Response {
        let mods = key.modifiers;

        // §3: Shift+Space switches modes.
        if key.code == keycode::SPACE && mods.shift && !mods.any_but_shift() {
            if key.repeat {
                return consume(Vec::new());
            }
            let calls = self.commit();
            self.mode = self.mode.other();
            return Response {
                switch_to: Some(self.mode),
                ..consume(calls)
            };
        }

        // §5: English mode passes everything else through.
        if self.mode == Mode::English {
            return Response::default();
        }

        // §4.4, first matching row.
        if mods.any_but_shift() {
            return pass(self.commit());
        }
        if key.code == keycode::BACKSPACE {
            if !self.composer.backspace() {
                return pass(Vec::new());
            }
            let call = if self.composer.is_empty() {
                ClientCall::ClearMarked
            } else {
                ClientCall::SetMarked(self.composer.text())
            };
            return consume(vec![call]);
        }
        if let Some(output) = layout::lookup(key.code, mods.shift) {
            return match output {
                Output::Jamo(slot, jamo) => {
                    let mut calls: Vec<ClientCall> = self
                        .composer
                        .input(slot, jamo)
                        .map(ClientCall::Insert)
                        .into_iter()
                        .collect();
                    calls.push(ClientCall::SetMarked(self.composer.text()));
                    consume(calls)
                }
                Output::Char(ch) => {
                    let mut calls = self.commit();
                    calls.push(ClientCall::Insert(ch.to_string()));
                    consume(calls)
                }
            };
        }
        if key.code == keycode::SPACE && !self.composer.is_empty() {
            let mut calls = self.commit();
            calls.push(ClientCall::Insert(" ".into()));
            return consume(calls);
        }

        // §4.5: Return and Escape in terminals.
        if (key.code == keycode::RETURN || key.code == keycode::ESCAPE)
            && !mods.shift
            && !self.composer.is_empty()
            && env.is_terminal()
            && env.can_post_events()
        {
            return Response {
                repost: Some(key.code),
                ..consume(self.commit())
            };
        }

        pass(self.commit())
    }

    /// The system selected a mode (§3). Commits the composition when the mode
    /// changes. Never requests a mode change in response.
    pub fn set_mode(&mut self, mode: Mode) -> Vec<ClientCall> {
        if mode == self.mode {
            return Vec::new();
        }
        self.mode = mode;
        self.commit()
    }

    /// Commits the composition, for example when the system asks for it.
    /// Returns no calls when there is nothing to commit.
    pub fn commit(&mut self) -> Vec<ClientCall> {
        self.composer
            .commit()
            .map(ClientCall::Insert)
            .into_iter()
            .collect()
    }
}

fn consume(calls: Vec<ClientCall>) -> Response {
    Response {
        calls,
        consumed: true,
        ..Response::default()
    }
}

fn pass(calls: Vec<ClientCall>) -> Response {
    Response {
        calls,
        consumed: false,
        ..Response::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::code_of;

    struct Env {
        terminal: bool,
        permission: bool,
    }

    impl Environment for Env {
        fn is_terminal(&self) -> bool {
            self.terminal
        }
        fn can_post_events(&self) -> bool {
            self.permission
        }
    }

    const APP: Env = Env {
        terminal: false,
        permission: false,
    };

    fn press(code: u16) -> KeyDown {
        KeyDown {
            code,
            modifiers: Modifiers::default(),
            repeat: false,
        }
    }

    fn shifted(code: u16) -> KeyDown {
        KeyDown {
            modifiers: Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            ..press(code)
        }
    }

    fn legend(ch: char) -> KeyDown {
        press(code_of(ch).unwrap())
    }

    fn insert(text: &str) -> ClientCall {
        ClientCall::Insert(text.into())
    }

    fn marked(text: &str) -> ClientCall {
        ClientCall::SetMarked(text.into())
    }

    /// An engine in Korean mode with 한 composed.
    fn composing_han() -> Engine {
        let mut engine = Engine::new(Mode::Korean);
        for ch in ['m', 'f', 's'] {
            engine.key_down(legend(ch), &APP);
        }
        engine
    }

    #[test]
    fn jamo_keys_compose_in_marked_text() {
        let mut engine = Engine::new(Mode::Korean);
        assert_eq!(
            engine.key_down(legend('m'), &APP),
            consume(vec![marked("ㅎ")])
        );
        assert_eq!(
            engine.key_down(legend('f'), &APP),
            consume(vec![marked("하")])
        );
        assert_eq!(
            engine.key_down(legend('s'), &APP),
            consume(vec![marked("한")])
        );
        assert_eq!(
            engine.key_down(legend('k'), &APP),
            consume(vec![insert("한"), marked("ㄱ")])
        );
    }

    #[test]
    fn shift_space_commits_and_switches() {
        let mut engine = composing_han();
        let response = engine.key_down(shifted(keycode::SPACE), &APP);
        assert_eq!(
            response,
            Response {
                switch_to: Some(Mode::English),
                ..consume(vec![insert("한")])
            }
        );
        assert_eq!(engine.mode(), Mode::English);
        // English mode passes keys through.
        assert_eq!(engine.key_down(legend('m'), &APP), pass(Vec::new()));
        let back = engine.key_down(shifted(keycode::SPACE), &APP);
        assert_eq!(back.switch_to, Some(Mode::Korean));
    }

    #[test]
    fn repeated_shift_space_does_not_switch() {
        let mut engine = Engine::new(Mode::Korean);
        let repeat = KeyDown {
            repeat: true,
            ..shifted(keycode::SPACE)
        };
        assert_eq!(engine.key_down(repeat, &APP), consume(Vec::new()));
        assert_eq!(engine.mode(), Mode::Korean);
    }

    #[test]
    fn shift_space_with_other_modifiers_is_not_a_switch() {
        let mut engine = Engine::new(Mode::English);
        let key = KeyDown {
            modifiers: Modifiers {
                shift: true,
                command: true,
                ..Modifiers::default()
            },
            ..press(keycode::SPACE)
        };
        assert_eq!(engine.key_down(key, &APP), pass(Vec::new()));
        assert_eq!(engine.mode(), Mode::English);
    }

    #[test]
    fn modifier_keys_commit_and_pass_through() {
        for modifiers in [
            Modifiers {
                command: true,
                ..Modifiers::default()
            },
            Modifiers {
                control: true,
                ..Modifiers::default()
            },
            Modifiers {
                option: true,
                ..Modifiers::default()
            },
            Modifiers {
                function: true,
                ..Modifiers::default()
            },
        ] {
            let mut engine = composing_han();
            let key = KeyDown {
                modifiers,
                ..legend('c')
            };
            assert_eq!(engine.key_down(key, &APP), pass(vec![insert("한")]));
        }
    }

    #[test]
    fn backspace_undoes_keystrokes_then_passes_through() {
        let mut engine = Engine::new(Mode::Korean);
        for ch in ['k', 'f', 's'] {
            engine.key_down(legend(ch), &APP);
        }
        let backspace = press(keycode::BACKSPACE);
        assert_eq!(
            engine.key_down(backspace, &APP),
            consume(vec![marked("가")])
        );
        assert_eq!(
            engine.key_down(backspace, &APP),
            consume(vec![marked("ㄱ")])
        );
        assert_eq!(
            engine.key_down(backspace, &APP),
            consume(vec![ClientCall::ClearMarked])
        );
        assert_eq!(engine.key_down(backspace, &APP), pass(Vec::new()));
    }

    #[test]
    fn character_keys_commit_and_insert() {
        let mut engine = composing_han();
        assert_eq!(
            engine.key_down(shifted(code_of('j').unwrap()), &APP),
            consume(vec![insert("한"), insert("1")])
        );
        assert_eq!(
            engine.key_down(legend('['), &APP),
            consume(vec![insert("(")])
        );
    }

    #[test]
    fn caps_lock_is_ignored_because_only_shift_selects_the_column() {
        let mut engine = Engine::new(Mode::Korean);
        // A key event with Caps Lock on has no Shift in `Modifiers`.
        assert_eq!(
            engine.key_down(legend('k'), &APP),
            consume(vec![marked("ㄱ")])
        );
    }

    #[test]
    fn space_commits_and_inserts_a_space_only_while_composing() {
        let mut engine = composing_han();
        assert_eq!(
            engine.key_down(press(keycode::SPACE), &APP),
            consume(vec![insert("한"), insert(" ")])
        );
        assert_eq!(
            engine.key_down(press(keycode::SPACE), &APP),
            pass(Vec::new())
        );
    }

    #[test]
    fn other_keys_commit_and_pass_through() {
        for code in [
            keycode::RETURN,
            keycode::ESCAPE,
            0x30, /* Tab */
            0x7B, /* Left */
        ] {
            let mut engine = composing_han();
            assert_eq!(engine.key_down(press(code), &APP), pass(vec![insert("한")]));
        }
    }

    #[test]
    fn return_and_escape_are_posted_again_in_terminals_with_permission() {
        let terminal = Env {
            terminal: true,
            permission: true,
        };
        for code in [keycode::RETURN, keycode::ESCAPE] {
            let mut engine = composing_han();
            assert_eq!(
                engine.key_down(press(code), &terminal),
                Response {
                    repost: Some(code),
                    ..consume(vec![insert("한")])
                }
            );
        }
    }

    #[test]
    fn return_is_passed_through_without_the_conditions_of_section_4_5() {
        let no_permission = Env {
            terminal: true,
            permission: false,
        };
        let not_terminal = Env {
            terminal: false,
            permission: true,
        };
        for env in [&no_permission, &not_terminal] {
            let mut engine = composing_han();
            assert_eq!(
                engine.key_down(press(keycode::RETURN), env),
                pass(vec![insert("한")])
            );
        }
        // Not composing.
        let terminal = Env {
            terminal: true,
            permission: true,
        };
        let mut engine = Engine::new(Mode::Korean);
        assert_eq!(
            engine.key_down(press(keycode::RETURN), &terminal),
            pass(Vec::new())
        );
        // Shift+Return.
        let mut engine = composing_han();
        assert_eq!(
            engine.key_down(shifted(keycode::RETURN), &terminal),
            pass(vec![insert("한")])
        );
    }

    #[test]
    fn system_mode_change_commits_without_requesting_a_switch() {
        let mut engine = composing_han();
        assert_eq!(engine.set_mode(Mode::Korean), Vec::new());
        assert_eq!(engine.set_mode(Mode::English), vec![insert("한")]);
        assert_eq!(engine.mode(), Mode::English);
    }

    #[test]
    fn commit_without_composition_makes_no_calls() {
        let mut engine = Engine::new(Mode::Korean);
        assert_eq!(engine.commit(), Vec::new());
    }
}
