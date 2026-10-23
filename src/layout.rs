//! Sebeolsik Final (3-91) key map (§4.1).
//!
//! Jamo are written as Hangul Compatibility Jamo together with their slot, so
//! the table reads like the one in the specification. `Slot` plus the glyph
//! identifies the conjoining jamo of §4.1 exactly.

/// The role of a jamo in a syllable (§4.2). The order is the slot order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Slot {
    Initial,
    Medial,
    Final,
}

/// What a layout key produces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    /// A jamo for the given slot.
    Jamo(Slot, char),
    /// A character that is inserted as is.
    Char(char),
}

/// Initial consonants in Unicode order (U+1100–U+1112).
pub const INITIALS: [char; 19] = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ',
    'ㅌ', 'ㅍ', 'ㅎ',
];

/// Medial vowels in Unicode order (U+1161–U+1175).
pub const MEDIALS: [char; 21] = [
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ', 'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ',
    'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
];

/// Final consonants in Unicode order (U+11A8–U+11C2).
pub const FINALS: [char; 27] = [
    'ㄱ', 'ㄲ', 'ㄳ', 'ㄴ', 'ㄵ', 'ㄶ', 'ㄷ', 'ㄹ', 'ㄺ', 'ㄻ', 'ㄼ', 'ㄽ', 'ㄾ', 'ㄿ', 'ㅀ', 'ㅁ',
    'ㅂ', 'ㅄ', 'ㅅ', 'ㅆ', 'ㅇ', 'ㅈ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
];

/// The jamo of a slot, in Unicode order.
pub fn jamo_of(slot: Slot) -> &'static [char] {
    match slot {
        Slot::Initial => &INITIALS,
        Slot::Medial => &MEDIALS,
        Slot::Final => &FINALS,
    }
}

/// One physical key of an ANSI keyboard.
#[derive(Clone, Copy, Debug)]
pub struct Key {
    /// macOS virtual key code (`kVK_ANSI_*`).
    pub code: u16,
    /// US QWERTY legend without Shift.
    pub legend: char,
    pub base: Output,
    pub shifted: Output,
}

const fn i(c: char) -> Output {
    Output::Jamo(Slot::Initial, c)
}
const fn m(c: char) -> Output {
    Output::Jamo(Slot::Medial, c)
}
const fn f(c: char) -> Output {
    Output::Jamo(Slot::Final, c)
}
const fn c(ch: char) -> Output {
    Output::Char(ch)
}
const fn key(code: u16, legend: char, base: Output, shifted: Output) -> Key {
    Key {
        code,
        legend,
        base,
        shifted,
    }
}

/// The 47 layout keys, row by row as in §4.1.
pub const KEYS: [Key; 47] = [
    // Number row
    key(0x32, '`', c('*'), c('※')),
    key(0x12, '1', f('ㅎ'), f('ㄲ')),
    key(0x13, '2', f('ㅆ'), f('ㄺ')),
    key(0x14, '3', f('ㅂ'), f('ㅈ')),
    key(0x15, '4', m('ㅛ'), f('ㄿ')),
    key(0x17, '5', m('ㅠ'), f('ㄾ')),
    key(0x16, '6', m('ㅑ'), c('=')),
    key(0x1A, '7', m('ㅖ'), c('“')),
    key(0x1C, '8', m('ㅢ'), c('”')),
    key(0x19, '9', m('ㅜ'), c('\'')),
    key(0x1D, '0', i('ㅋ'), c('~')),
    key(0x1B, '-', c(')'), c(';')),
    key(0x18, '=', c('>'), c('+')),
    // Top row
    key(0x0C, 'q', f('ㅅ'), f('ㅍ')),
    key(0x0D, 'w', f('ㄹ'), f('ㅌ')),
    key(0x0E, 'e', m('ㅕ'), f('ㄵ')),
    key(0x0F, 'r', m('ㅐ'), f('ㅀ')),
    key(0x11, 't', m('ㅓ'), f('ㄽ')),
    key(0x10, 'y', i('ㄹ'), c('5')),
    key(0x20, 'u', i('ㄷ'), c('6')),
    key(0x22, 'i', i('ㅁ'), c('7')),
    key(0x1F, 'o', i('ㅊ'), c('8')),
    key(0x23, 'p', i('ㅍ'), c('9')),
    key(0x21, '[', c('('), c('%')),
    key(0x1E, ']', c('<'), c('/')),
    key(0x2A, '\\', c(':'), c('\\')),
    // Home row
    key(0x00, 'a', f('ㅇ'), f('ㄷ')),
    key(0x01, 's', f('ㄴ'), f('ㄶ')),
    key(0x02, 'd', m('ㅣ'), f('ㄼ')),
    key(0x03, 'f', m('ㅏ'), f('ㄻ')),
    key(0x05, 'g', m('ㅡ'), m('ㅒ')),
    key(0x04, 'h', i('ㄴ'), c('0')),
    key(0x26, 'j', i('ㅇ'), c('1')),
    key(0x28, 'k', i('ㄱ'), c('2')),
    key(0x25, 'l', i('ㅈ'), c('3')),
    key(0x29, ';', i('ㅂ'), c('4')),
    key(0x27, '\'', i('ㅌ'), c('·')),
    // Bottom row
    key(0x06, 'z', f('ㅁ'), f('ㅊ')),
    key(0x07, 'x', f('ㄱ'), f('ㅄ')),
    key(0x08, 'c', m('ㅔ'), f('ㅋ')),
    key(0x09, 'v', m('ㅗ'), f('ㄳ')),
    key(0x0B, 'b', m('ㅜ'), c('?')),
    key(0x2D, 'n', i('ㅅ'), c('-')),
    key(0x2E, 'm', i('ㅎ'), c('"')),
    key(0x2B, ',', c(','), c(',')),
    key(0x2F, '.', c('.'), c('.')),
    key(0x2C, '/', m('ㅗ'), c('!')),
];

/// What the layout key with this virtual key code produces, or `None` if the
/// key is not a layout key.
pub fn lookup(code: u16, shift: bool) -> Option<Output> {
    KEYS.iter()
        .find(|k| k.code == code)
        .map(|k| if shift { k.shifted } else { k.base })
}

/// The virtual key code of the layout key with this QWERTY legend.
pub fn code_of(legend: char) -> Option<u16> {
    KEYS.iter().find(|k| k.legend == legend).map(|k| k.code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_jamo_belongs_to_its_slot() {
        for k in KEYS {
            for out in [k.base, k.shifted] {
                if let Output::Jamo(slot, ch) = out {
                    assert!(jamo_of(slot).contains(&ch), "{ch} is not a {slot:?} jamo");
                }
            }
        }
    }

    #[test]
    fn codes_and_legends_are_unique() {
        for (n, a) in KEYS.iter().enumerate() {
            for b in &KEYS[n + 1..] {
                assert_ne!(a.code, b.code);
                assert_ne!(a.legend, b.legend);
            }
        }
    }

    #[test]
    fn every_basic_jamo_has_a_key() {
        let produced: Vec<Output> = KEYS.iter().flat_map(|k| [k.base, k.shifted]).collect();
        // All 14 basic initials, all 27 finals, and the 15 medials that have keys.
        for ch in "ㄱㄴㄷㄹㅁㅂㅅㅇㅈㅊㅋㅌㅍㅎ".chars() {
            assert!(
                produced.contains(&Output::Jamo(Slot::Initial, ch)),
                "initial {ch}"
            );
        }
        for ch in FINALS {
            assert!(
                produced.contains(&Output::Jamo(Slot::Final, ch)),
                "final {ch}"
            );
        }
        for ch in "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅛㅜㅠㅡㅢㅣ".chars() {
            assert!(
                produced.contains(&Output::Jamo(Slot::Medial, ch)),
                "medial {ch}"
            );
        }
    }

    #[test]
    fn excluded_ascii_symbols_are_not_produced() {
        // §4.1: Korean mode cannot produce these.
        for ch in "`@#$^&_[]{}|".chars() {
            for k in KEYS {
                assert_ne!(k.base, Output::Char(ch));
                assert_ne!(k.shifted, Output::Char(ch));
            }
        }
    }
}
