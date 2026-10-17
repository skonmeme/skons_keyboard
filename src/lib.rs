//! skon's Keyboard: a Korean (Sebeolsik Final) / English input method for macOS.
//!
//! The behavior is specified in `spec/STANDARD.md`. Section numbers in
//! comments (for example §4.2) refer to that document.

pub mod composer;
pub mod engine;
pub mod layout;
pub mod mac;

#[cfg(test)]
mod spec_tests;
