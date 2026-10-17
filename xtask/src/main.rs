//! Builds the signed and notarized installer and uninstaller packages (§8.5):
//!
//! ```text
//! cargo xtask package
//! ```
//!
//! The packages are written to `target/package/`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const APP_IDENTITY: &str = "Developer ID Application";
const INSTALLER_IDENTITY: &str = "Developer ID Installer";
const NOTARY_PROFILE: &str = "skon's Universe";
const BUNDLE_ID: &str = "com.skon.inputmethod.skonsKeyboard";
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let result = match std::env::args().nth(1).as_deref() {
        Some("package") => package(),
        _ => Err("usage: cargo xtask package".into()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

type Result<T = ()> = std::result::Result<T, String>;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

/// Runs a command and returns its standard output and error together.
fn run(command: &mut Command) -> Result<String> {
    let shown = format!("{command:?}");
    println!("$ {shown}");
    let output = command.output().map_err(|e| format!("{shown}: {e}"))?;
    let text = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        return Err(format!("{shown} failed:\n{text}"));
    }
    Ok(text)
}

fn copy(from: &Path, to: &Path) -> Result {
    fs::copy(from, to)
        .map(drop)
        .map_err(|e| format!("copy {} to {}: {e}", from.display(), to.display()))
}

/// Copies a text file, replacing `@VERSION@`.
fn fill(from: &Path, to: &Path) -> Result {
    let text = fs::read_to_string(from).map_err(|e| format!("read {}: {e}", from.display()))?;
    fs::write(to, text.replace("@VERSION@", VERSION))
        .map_err(|e| format!("write {}: {e}", to.display()))
}

fn package() -> Result {
    let root = root();
    let out = root.join("target/package");
    if out.exists() {
        fs::remove_dir_all(&out).map_err(|e| format!("clean {}: {e}", out.display()))?;
    }
    let parts = out.join("parts");
    fs::create_dir_all(&parts).map_err(|e| e.to_string())?;

    // 1. Build the input method.
    run(Command::new(env!("CARGO"))
        .args(["build", "--release", "--package", "skons-keyboard"])
        .current_dir(&root))?;

    // 2. Assemble the app bundle in the payload (§2.3, §8.1).
    let payload = out.join("payload");
    let app = payload.join("Library/Input Methods/skonsKeyboard.app");
    let contents = app.join("Contents");
    let resources = contents.join("Resources");
    for dir in [contents.join("MacOS"), resources.join("en.lproj")] {
        fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }
    copy(
        &root.join("target/release/skons-keyboard"),
        &contents.join("MacOS/skonsKeyboard"),
    )?;
    fill(
        &root.join("resources/Info.plist"),
        &contents.join("Info.plist"),
    )?;
    for file in ["korean.tiff", "english.tiff", "en.lproj/InfoPlist.strings"] {
        copy(&root.join("resources").join(file), &resources.join(file))?;
    }

    // 3. Sign the app with the hardened runtime and a secure timestamp (§8.4).
    run(Command::new("codesign")
        .args([
            "--force",
            "--options",
            "runtime",
            "--timestamp",
            "--sign",
            APP_IDENTITY,
        ])
        .arg(&app))?;
    run(Command::new("codesign")
        .args(["--verify", "--strict", "--verbose=2"])
        .arg(&app))?;

    // 4. Component packages. The app must not be relocated to another copy
    //    of the bundle elsewhere on the disk.
    let components = out.join("components.plist");
    run(Command::new("pkgbuild")
        .arg("--analyze")
        .arg("--root")
        .arg(&payload)
        .arg(&components))?;
    run(Command::new("plutil")
        .args(["-replace", "0.BundleIsRelocatable", "-bool", "NO"])
        .arg(&components))?;
    let scripts = root.join("packaging/scripts");
    run(Command::new("pkgbuild")
        .arg("--root")
        .arg(&payload)
        .arg("--component-plist")
        .arg(&components)
        .args([
            "--identifier",
            &format!("{BUNDLE_ID}.pkg"),
            "--version",
            VERSION,
        ])
        .args(["--install-location", "/"])
        .arg("--scripts")
        .arg(scripts.join("install"))
        .arg(parts.join("app.pkg")))?;
    for (part, identifier) in [
        ("accessibility", "accessibility"),
        ("uninstall", "uninstall"),
    ] {
        run(Command::new("pkgbuild")
            .arg("--nopayload")
            .args([
                "--identifier",
                &format!("{BUNDLE_ID}.{identifier}"),
                "--version",
                VERSION,
            ])
            .arg("--scripts")
            .arg(scripts.join(part))
            .arg(parts.join(format!("{part}.pkg"))))?;
    }

    // 5. Signed product packages.
    let packages = [
        (
            "installer.xml",
            out.join(format!("skonsKeyboard-{VERSION}.pkg")),
        ),
        (
            "uninstaller.xml",
            out.join(format!("skonsKeyboard-Uninstall-{VERSION}.pkg")),
        ),
    ];
    for (distribution, package) in &packages {
        let filled = out.join(distribution);
        fill(&root.join("packaging").join(distribution), &filled)?;
        run(Command::new("productbuild")
            .arg("--distribution")
            .arg(&filled)
            .arg("--package-path")
            .arg(&parts)
            .arg("--resources")
            .arg(root.join("packaging/resources"))
            .args(["--sign", INSTALLER_IDENTITY, "--timestamp"])
            .arg(package))?;
    }

    // 6. Notarize, staple, and verify (§8.4, §8.6).
    for (_, package) in &packages {
        let submitted = run(Command::new("xcrun")
            .args(["notarytool", "submit"])
            .arg(package)
            .args(["--keychain-profile", NOTARY_PROFILE, "--wait"]))?;
        if !submitted.contains("status: Accepted") {
            return Err(format!(
                "notarization of {} was not accepted:\n{submitted}",
                package.display()
            ));
        }
        run(Command::new("xcrun")
            .args(["stapler", "staple"])
            .arg(package))?;
        let assessed = run(Command::new("spctl")
            .args(["--assess", "--type", "install", "-vv"])
            .arg(package))?;
        if !(assessed.contains("accepted") && assessed.contains("Notarized Developer ID")) {
            return Err(format!(
                "Gatekeeper does not accept {}:\n{assessed}",
                package.display()
            ));
        }
        run(Command::new("xcrun")
            .args(["stapler", "validate"])
            .arg(package))?;
    }

    for (_, package) in &packages {
        println!("built {}", package.display());
    }
    Ok(())
}
