//! Hangul composition (§4.2) and its text representation (§4.3).

use crate::layout::{FINALS, INITIALS, MEDIALS, Slot};

/// Combination table of §4.2: (slot, current, typed, combined).
pub(crate) const COMBINATIONS: [(Slot, char, char, char); 25] = [
    (Slot::Initial, 'ㄱ', 'ㄱ', 'ㄲ'),
    (Slot::Initial, 'ㄷ', 'ㄷ', 'ㄸ'),
    (Slot::Initial, 'ㅂ', 'ㅂ', 'ㅃ'),
    (Slot::Initial, 'ㅅ', 'ㅅ', 'ㅆ'),
    (Slot::Initial, 'ㅈ', 'ㅈ', 'ㅉ'),
    (Slot::Medial, 'ㅗ', 'ㅏ', 'ㅘ'),
    (Slot::Medial, 'ㅗ', 'ㅐ', 'ㅙ'),
    (Slot::Medial, 'ㅗ', 'ㅣ', 'ㅚ'),
    (Slot::Medial, 'ㅜ', 'ㅓ', 'ㅝ'),
    (Slot::Medial, 'ㅜ', 'ㅔ', 'ㅞ'),
    (Slot::Medial, 'ㅜ', 'ㅣ', 'ㅟ'),
    (Slot::Medial, 'ㅡ', 'ㅣ', 'ㅢ'),
    (Slot::Final, 'ㄱ', 'ㄱ', 'ㄲ'),
    (Slot::Final, 'ㄱ', 'ㅅ', 'ㄳ'),
    (Slot::Final, 'ㄴ', 'ㅈ', 'ㄵ'),
    (Slot::Final, 'ㄴ', 'ㅎ', 'ㄶ'),
    (Slot::Final, 'ㄹ', 'ㄱ', 'ㄺ'),
    (Slot::Final, 'ㄹ', 'ㅁ', 'ㄻ'),
    (Slot::Final, 'ㄹ', 'ㅂ', 'ㄼ'),
    (Slot::Final, 'ㄹ', 'ㅅ', 'ㄽ'),
    (Slot::Final, 'ㄹ', 'ㅌ', 'ㄾ'),
    (Slot::Final, 'ㄹ', 'ㅍ', 'ㄿ'),
    (Slot::Final, 'ㄹ', 'ㅎ', 'ㅀ'),
    (Slot::Final, 'ㅂ', 'ㅅ', 'ㅄ'),
    (Slot::Final, 'ㅅ', 'ㅅ', 'ㅆ'),
];

fn combine(slot: Slot, current: char, typed: char) -> Option<char> {
    COMBINATIONS
        .iter()
        .find(|&&(s, a, b, _)| s == slot && a == current && b == typed)
        .map(|&(_, _, _, combined)| combined)
}

/// A syllable with three slots in the fixed order initial < medial < final.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Syllable {
    initial: Option<char>,
    medial: Option<char>,
    final_: Option<char>,
}

impl Syllable {
    fn get(&self, slot: Slot) -> Option<char> {
        match slot {
            Slot::Initial => self.initial,
            Slot::Medial => self.medial,
            Slot::Final => self.final_,
        }
    }

    fn set(&mut self, slot: Slot, jamo: char) {
        match slot {
            Slot::Initial => self.initial = Some(jamo),
            Slot::Medial => self.medial = Some(jamo),
            Slot::Final => self.final_ = Some(jamo),
        }
    }

    /// The filled slot that comes latest in slot order.
    fn last_slot(&self) -> Option<Slot> {
        [Slot::Final, Slot::Medial, Slot::Initial]
            .into_iter()
            .find(|&slot| self.get(slot).is_some())
    }

    fn is_empty(&self) -> bool {
        self.last_slot().is_none()
    }

    /// §4.3: the precomposed syllable, or the compatibility jamo of each
    /// filled slot in slot order.
    fn text(&self) -> String {
        let index = |table: &[char], ch: char| table.iter().position(|&t| t == ch);
        if let (Some(l), Some(v)) = (self.initial, self.medial) {
            let l = index(&INITIALS, l).expect("initial jamo") as u32;
            let v = index(&MEDIALS, v).expect("medial jamo") as u32;
            let t = self
                .final_
                .map_or(0, |t| index(&FINALS, t).expect("final jamo") as u32 + 1);
            let syllable = char::from_u32(0xAC00 + (l * 21 + v) * 28 + t).expect("Hangul syllable");
            return syllable.to_string();
        }
        [self.initial, self.medial, self.final_]
            .into_iter()
            .flatten()
            .collect()
    }
}

/// The composition of Korean mode.
#[derive(Debug, Default)]
pub struct Composer {
    current: Syllable,
    /// The states before each keystroke of the current composition, for
    /// Backspace.
    history: Vec<Syllable>,
}

impl Composer {
    pub fn is_empty(&self) -> bool {
        self.current.is_empty()
    }

    /// The composition as shown in marked text (§4.3).
    pub fn text(&self) -> String {
        self.current.text()
    }

    /// Applies a jamo key (§4.2). Returns the text committed by rule 3.
    pub fn input(&mut self, slot: Slot, jamo: char) -> Option<String> {
        let last = self.current.last_slot();
        // Rule 1: empty, or the slot comes after the last slot.
        if last.is_none_or(|last| slot > last) {
            self.history.push(self.current);
            self.current.set(slot, jamo);
            return None;
        }
        // Rule 2: combine in the last slot.
        if last == Some(slot) {
            let current = self.current.get(slot).expect("last slot is filled");
            if let Some(combined) = combine(slot, current, jamo) {
                self.history.push(self.current);
                self.current.set(slot, combined);
                return None;
            }
        }
        // Rule 3: commit and start a new composition.
        let committed = self.commit();
        self.history.push(self.current);
        self.current.set(slot, jamo);
        committed
    }

    /// Undoes the most recent keystroke. Returns false if there was no
    /// composition to change.
    pub fn backspace(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }
        self.current = self.history.pop().unwrap_or_default();
        true
    }

    /// Takes the composition as committed text, leaving it empty. Committing
    /// an empty composition returns `None`.
    pub fn commit(&mut self) -> Option<String> {
        if self.is_empty() {
            return None;
        }
        let text = self.current.text();
        self.current = Syllable::default();
        self.history.clear();
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{Output, code_of, lookup};

    /// Types QWERTY legends; `⌫` is Backspace. Returns the composition after
    /// each key and all committed text so far plus the final composition.
    fn type_keys(keys: &str) -> (Vec<String>, String) {
        let mut composer = Composer::default();
        let mut committed = String::new();
        let mut steps = Vec::new();
        for legend in keys.split_whitespace() {
            if legend == "⌫" {
                composer.backspace();
            } else {
                let ch = legend.chars().next().unwrap();
                let code = code_of(ch).unwrap_or_else(|| panic!("no key {ch}"));
                match lookup(code, false) {
                    Some(Output::Jamo(slot, jamo)) => {
                        committed.extend(composer.input(slot, jamo));
                    }
                    other => panic!("{ch} is not a jamo key: {other:?}"),
                }
            }
            steps.push(composer.text());
        }
        committed.push_str(&composer.text());
        (steps, committed)
    }

    /// The examples of §4.2.
    #[test]
    fn specification_examples() {
        let examples = [
            ("m f s k g w", "ㅎ 하 한 ㄱ 그 글", "한글"),
            ("n c ; t w n d x", "ㅅ 세 ㅂ 버 벌 ㅅ 시 식", "세벌식"),
            ("k k f", "ㄱ ㄲ 까", "까"),
            ("k v f", "ㄱ 고 과", "과"),
            ("k f w x", "ㄱ 가 갈 갉", "갉"),
            ("k f x f", "ㄱ 가 각 ㅏ", "각ㅏ"),
            ("k f x ⌫ ⌫", "ㄱ 가 각 가 ㄱ", "ㄱ"),
        ];
        for (keys, steps, result) in examples {
            let (got_steps, got_result) = type_keys(keys);
            assert_eq!(got_steps.join(" "), steps, "steps of {keys}");
            assert_eq!(got_result, result, "result of {keys}");
        }
    }

    #[test]
    fn incomplete_syllables_use_compatibility_jamo() {
        let mut composer = Composer::default();
        composer.input(Slot::Medial, 'ㅏ');
        composer.input(Slot::Final, 'ㄱ');
        assert_eq!(composer.text(), "\u{314F}\u{3131}");

        let mut composer = Composer::default();
        composer.input(Slot::Initial, 'ㄱ');
        composer.input(Slot::Final, 'ㄴ');
        assert_eq!(composer.text(), "ㄱㄴ");
    }

    #[test]
    fn precomposed_syllable_range() {
        let mut composer = Composer::default();
        composer.input(Slot::Initial, 'ㄱ');
        composer.input(Slot::Medial, 'ㅏ');
        assert_eq!(composer.text(), "\u{AC00}");
        let mut composer = Composer::default();
        composer.input(Slot::Initial, 'ㅎ');
        composer.input(Slot::Medial, 'ㅣ');
        composer.input(Slot::Final, 'ㅎ');
        assert_eq!(composer.text(), "\u{D7A3}");
    }

    #[test]
    fn combinations_apply_only_in_the_last_slot() {
        // ㄱ ㅗ then ㄱ: the initial is not the last slot, so no ㄲ.
        let mut composer = Composer::default();
        composer.input(Slot::Initial, 'ㄱ');
        composer.input(Slot::Medial, 'ㅗ');
        assert_eq!(composer.input(Slot::Initial, 'ㄱ'), Some("고".into()));
        assert_eq!(composer.text(), "ㄱ");
    }

    #[test]
    fn uncombinable_jamo_in_the_last_slot_commits() {
        let mut composer = Composer::default();
        composer.input(Slot::Initial, 'ㄱ');
        composer.input(Slot::Initial, 'ㄱ');
        assert_eq!(composer.input(Slot::Initial, 'ㄱ'), Some("ㄲ".into()));
        assert_eq!(composer.text(), "ㄱ");
    }

    #[test]
    fn backspace_after_rule_three_ends_the_new_composition() {
        let mut composer = Composer::default();
        composer.input(Slot::Initial, 'ㄱ');
        composer.input(Slot::Medial, 'ㅏ');
        composer.input(Slot::Initial, 'ㄴ');
        assert!(composer.backspace());
        assert!(composer.is_empty());
        assert!(!composer.backspace());
    }

    #[test]
    fn commit_is_idempotent() {
        let mut composer = Composer::default();
        assert_eq!(composer.commit(), None);
        composer.input(Slot::Initial, 'ㅎ');
        assert_eq!(composer.commit(), Some("ㅎ".into()));
        assert_eq!(composer.commit(), None);
    }
}
