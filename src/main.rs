//! The input method executable. Without arguments it runs the input method.
//! The installer and uninstaller (§8) use the other modes.

use std::process::ExitCode;

use skons_keyboard::mac;

fn main() -> ExitCode {
    let result = match std::env::args().nth(1).as_deref() {
        Some("--register") => mac::register(),
        Some("--unregister") => mac::unregister(),
        Some("--request-accessibility") => {
            mac::request_accessibility();
            Ok(())
        }
        _ => {
            mac::run();
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("skonsKeyboard: {message}");
            ExitCode::FAILURE
        }
    }
}
