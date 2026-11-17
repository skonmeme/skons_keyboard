//! Number and password fields (§3.1), independent of macOS: which focused
//! element is an English field, and which mode to select when focus moves.

use crate::engine::Mode;

/// What has focus in the browser.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    /// A number field or a password field (§3.1).
    EnglishField,
    /// Anything else, including another application.
    Other,
}

/// Classifies a focused element by what the Accessibility API reports
/// (§3.1). `child_roles` are needed only for a text field.
pub fn classify(role: &str, subrole: &str, child_roles: &[String]) -> Focus {
    let number = role == "AXIncrementor"
        || (role == "AXTextField" && child_roles.iter().any(|r| r == "AXIncrementor"));
    if number || subrole == "AXSecureTextField" {
        Focus::EnglishField
    } else {
        Focus::Other
    }
}

/// Whether the children of an element with this role and subrole decide
/// what it is: a text field in Safari is a number field if it has an
/// incrementor child.
pub fn needs_child_roles(role: &str, subrole: &str) -> bool {
    role == "AXTextField" && subrole != "AXSecureTextField"
}

/// Remembers the mode to select again when focus leaves English fields.
#[derive(Debug, Default)]
pub struct Fields {
    in_field: bool,
    /// The mode to select when focus leaves English fields.
    restore: Option<Mode>,
    /// The mode before a system change to English mode made while focus was
    /// not in an English field. A browser can make it for a password field
    /// before the focus notification arrives.
    unconfirmed: Option<Mode>,
}

impl Fields {
    pub fn in_english_field(&self) -> bool {
        self.in_field
    }

    /// Focus moved to `focus` while `current` is selected. Returns the mode
    /// to select, if any.
    pub fn focus(&mut self, focus: Focus, current: Mode) -> Option<Mode> {
        let unconfirmed = self.unconfirmed.take();
        match (focus, self.in_field) {
            (Focus::EnglishField, false) => {
                self.in_field = true;
                self.restore = Some(unconfirmed.unwrap_or(current));
                (current == Mode::Korean).then_some(Mode::English)
            }
            (Focus::Other, true) => {
                self.in_field = false;
                self.restore.take().filter(|&mode| mode != current)
            }
            _ => None,
        }
    }

    /// The system selected `new` while `previous` was selected (§3).
    pub fn system_changed(&mut self, previous: Mode, new: Mode) {
        if previous == new {
            return;
        }
        if self.in_field {
            // The user chose Korean mode in the field: keep it after leaving.
            if new == Mode::Korean {
                self.restore = None;
            }
        } else if new == Mode::English {
            self.unconfirmed.get_or_insert(previous);
        } else {
            self.unconfirmed = None;
        }
    }

    /// Shift+Space switched modes: the user chose the mode.
    pub fn shift_space(&mut self) {
        self.unconfirmed = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Focus::{EnglishField, Other};
    use Mode::{English, Korean};

    fn roles(roles: &[&str]) -> Vec<String> {
        roles.iter().map(|r| r.to_string()).collect()
    }

    #[test]
    fn classifies_the_fields_of_section_3_1() {
        // Safari number field, Firefox number field, password fields.
        assert_eq!(
            classify("AXTextField", "-", &roles(&["AXIncrementor"])),
            EnglishField
        );
        assert_eq!(classify("AXIncrementor", "-", &[]), EnglishField);
        assert_eq!(
            classify("AXTextField", "AXSecureTextField", &[]),
            EnglishField
        );
        assert_eq!(
            classify("AXTextField", "AXSecureTextField", &roles(&["AXButton"])),
            EnglishField
        );
        // Text fields (also type="tel"), the page, windows.
        assert_eq!(classify("AXTextField", "-", &[]), Other);
        assert_eq!(classify("AXTextArea", "-", &[]), Other);
        assert_eq!(classify("AXWebArea", "-", &roles(&["AXGroup"])), Other);
        assert_eq!(classify("AXWindow", "AXStandardWindow", &[]), Other);
    }

    #[test]
    fn only_plain_text_fields_need_their_children() {
        assert!(needs_child_roles("AXTextField", "-"));
        assert!(!needs_child_roles("AXTextField", "AXSecureTextField"));
        assert!(!needs_child_roles("AXIncrementor", "-"));
        assert!(!needs_child_roles("AXWebArea", "-"));
    }

    #[test]
    fn korean_mode_returns_after_leaving() {
        let mut fields = Fields::default();
        assert_eq!(fields.focus(EnglishField, Korean), Some(English));
        assert!(fields.in_english_field());
        // Moving to another English field changes nothing.
        assert_eq!(fields.focus(EnglishField, English), None);
        assert_eq!(fields.focus(Other, English), Some(Korean));
        assert!(!fields.in_english_field());
        // Nothing to restore any more.
        assert_eq!(fields.focus(Other, English), None);
    }

    #[test]
    fn english_mode_stays_after_leaving() {
        let mut fields = Fields::default();
        assert_eq!(fields.focus(EnglishField, English), None);
        assert_eq!(fields.focus(Other, English), None);
    }

    #[test]
    fn korean_mode_chosen_in_the_field_is_kept() {
        let mut fields = Fields::default();
        fields.focus(EnglishField, Korean);
        // The echo of our own request is not a change.
        fields.system_changed(English, English);
        fields.system_changed(English, Korean);
        assert!(fields.in_english_field());
        assert_eq!(fields.focus(Other, Korean), None);
    }

    #[test]
    fn english_mode_forced_in_the_field_keeps_the_remembered_mode() {
        // Safari: the focus notification comes before macOS selects English
        // mode for the password field.
        let mut fields = Fields::default();
        assert_eq!(fields.focus(EnglishField, Korean), Some(English));
        fields.system_changed(Korean, English);
        assert_eq!(fields.focus(Other, English), Some(Korean));
    }

    #[test]
    fn english_mode_forced_before_the_notification_is_not_remembered() {
        // Firefox: macOS selects English mode for the password field before
        // the focus notification arrives.
        let mut fields = Fields::default();
        fields.system_changed(Korean, English);
        assert_eq!(fields.focus(EnglishField, English), None);
        assert_eq!(fields.focus(Other, English), Some(Korean));
    }

    #[test]
    fn a_change_followed_by_focus_or_shift_space_is_the_users() {
        let mut fields = Fields::default();
        fields.system_changed(Korean, English);
        fields.focus(Other, English);
        assert_eq!(fields.focus(EnglishField, English), None);
        assert_eq!(fields.focus(Other, English), None);

        let mut fields = Fields::default();
        fields.system_changed(Korean, English);
        fields.shift_space();
        fields.focus(EnglishField, English);
        assert_eq!(fields.focus(Other, English), None);

        // A change to Korean mode is never made for a password field.
        let mut fields = Fields::default();
        fields.system_changed(Korean, English);
        fields.system_changed(English, Korean);
        assert_eq!(fields.focus(EnglishField, Korean), Some(English));
        assert_eq!(fields.focus(Other, English), Some(Korean));
    }
}
