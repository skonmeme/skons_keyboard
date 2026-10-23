//! Checks that the tables in the code match `spec/STANDARD.md`, which is the
//! only normative document (§9.1).

use crate::composer::COMBINATIONS;
use crate::layout::{KEYS, Output, Slot};

fn spec() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/spec/STANDARD.md");
    std::fs::read_to_string(path).expect("read spec/STANDARD.md")
}

/// The text between two headings of the specification.
fn section<'a>(spec: &'a str, start: &str, end: &str) -> &'a str {
    let from = spec
        .find(start)
        .unwrap_or_else(|| panic!("missing {start}"));
    let to = spec[from..]
        .find(end)
        .unwrap_or_else(|| panic!("missing {end}"));
    &spec[from..from + to]
}

fn code_span(cell: &str) -> Option<&str> {
    cell.strip_prefix("`` ")
        .and_then(|s| s.strip_suffix(" ``"))
        .or_else(|| cell.strip_prefix('`').and_then(|s| s.strip_suffix('`')))
}

fn parse_output(cell: &str) -> Output {
    if let Some(text) = code_span(cell) {
        let mut chars = text.chars();
        let ch = chars.next().expect("character");
        assert!(chars.next().is_none(), "one character expected in {cell}");
        return Output::Char(ch);
    }
    let (role, jamo) = cell
        .split_once(' ')
        .unwrap_or_else(|| panic!("bad cell {cell}"));
    let slot = match role {
        "initial" => Slot::Initial,
        "medial" => Slot::Medial,
        "final" => Slot::Final,
        _ => panic!("bad role in {cell}"),
    };
    Output::Jamo(slot, jamo.chars().next().expect("jamo"))
}

#[test]
fn layout_matches_section_4_1() {
    let spec = spec();
    let rows: Vec<Vec<&str>> = section(&spec, "### 4.1 Layout", "### 4.2")
        .lines()
        .filter(|line| line.starts_with("| `"))
        .map(|line| line.trim_matches('|').split('|').map(str::trim).collect())
        .collect();
    assert_eq!(rows.len(), KEYS.len(), "number of keys");
    for row in rows {
        let legend = code_span(row[0])
            .expect("key legend")
            .chars()
            .next()
            .unwrap();
        let key = KEYS
            .iter()
            .find(|k| k.legend == legend)
            .unwrap_or_else(|| panic!("key {legend}"));
        assert_eq!(key.base, parse_output(row[1]), "key {legend} without Shift");
        assert_eq!(key.shifted, parse_output(row[2]), "key {legend} with Shift");
    }
}

#[test]
fn combinations_match_section_4_2() {
    let spec = spec();
    let mut expected = Vec::new();
    for line in section(&spec, "Combination table:", "A medial vowel typed").lines() {
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        let slot = match cells[0] {
            "Initial" => Slot::Initial,
            "Medial" => Slot::Medial,
            "Final" => Slot::Final,
            _ => continue,
        };
        for entry in cells[1].split(", ") {
            let chars: Vec<char> = entry.chars().collect();
            assert_eq!(
                (chars.len(), chars[1], chars[3]),
                (5, '+', '='),
                "entry {entry}"
            );
            expected.push((slot, chars[0], chars[2], chars[4]));
        }
    }
    assert_eq!(expected, COMBINATIONS.to_vec());
}
