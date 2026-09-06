//! The login item on disk, and whether it still points at this build.
//!
//! `auto-launch` calls a login item enabled when its plist exists, whatever
//! the plist points at. Enable it once from a build under `target/` and the
//! file names that binary until something rewrites it, so every login
//! starts a stale dev build beside the installed app. The startup reconcile
//! therefore reads the program the plist names and compares it with the
//! running executable, and only an installed build is allowed to write.

use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Enable,
    Disable,
}

/// What the startup reconcile should do to the login item. `registered`
/// is the program the plist on disk names, if any; `current` is the
/// running executable.
pub fn action(wanted: bool, registered: Option<&Path>, current: &Path) -> Option<Action> {
    if !wanted {
        return registered.map(|_| Action::Disable);
    }
    if registered == Some(current) {
        return None;
    }
    // A build under target/ never writes: the last one that did is how
    // the login item came to point at a dev binary in the first place.
    is_installed(current).then_some(Action::Enable)
}

/// An `.app` bundle under an `Applications` folder, system or per-user.
/// A bundle under `target/release/bundle` is not installed, and a login
/// item pointing at it would break on the next clean build.
pub fn is_installed(exe: &Path) -> bool {
    let mut seen_bundle = false;
    for component in exe.components().rev() {
        let name = component.as_os_str().to_string_lossy();
        if name.ends_with(".app") {
            seen_bundle = true;
        } else if name == "Applications" {
            return seen_bundle;
        }
    }
    false
}

/// `~/Library/LaunchAgents/{app_name}.plist`, the file `auto-launch` writes.
pub fn plist_path(app_name: &str) -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join("Library/LaunchAgents")
            .join(format!("{app_name}.plist")),
    )
}

/// The program the login item starts: `ProgramArguments[0]`. None for a
/// missing or unreadable file, which the caller treats as not registered.
pub fn registered_program(plist: &Path) -> Option<PathBuf> {
    let value = plist::Value::from_file(plist).ok()?;
    let first = value
        .as_dictionary()?
        .get("ProgramArguments")?
        .as_array()?
        .first()?
        .as_string()?;
    Some(PathBuf::from(first))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const INSTALLED: &str = "/Applications/Ambient Context.app/Contents/MacOS/ambient-context";
    const DEV: &str = "/Users/me/Sites/ambient-context/src-tauri/target/debug/ambient-context";

    #[test]
    fn a_wanted_login_item_that_is_missing_is_enabled() {
        assert_eq!(
            action(true, None, Path::new(INSTALLED)),
            Some(Action::Enable)
        );
    }

    #[test]
    fn a_wanted_login_item_that_points_at_this_build_is_left_alone() {
        assert_eq!(
            action(true, Some(Path::new(INSTALLED)), Path::new(INSTALLED)),
            None
        );
    }

    #[test]
    fn a_wanted_login_item_that_points_elsewhere_is_rewritten_by_an_installed_build() {
        assert_eq!(
            action(true, Some(Path::new(DEV)), Path::new(INSTALLED)),
            Some(Action::Enable)
        );
    }

    #[test]
    fn a_build_that_is_not_installed_never_writes_a_login_item() {
        assert_eq!(action(true, None, Path::new(DEV)), None);
        assert_eq!(
            action(true, Some(Path::new(INSTALLED)), Path::new(DEV)),
            None
        );
    }

    #[test]
    fn an_unwanted_login_item_is_disabled_whichever_build_it_points_at() {
        assert_eq!(
            action(false, Some(Path::new(DEV)), Path::new(INSTALLED)),
            Some(Action::Disable)
        );
        assert_eq!(action(false, None, Path::new(INSTALLED)), None);
    }

    #[test]
    fn installed_means_an_app_bundle_under_an_applications_folder() {
        assert!(is_installed(Path::new(INSTALLED)));
        assert!(is_installed(Path::new(
            "/Users/me/Applications/Ambient Context.app/Contents/MacOS/ambient-context"
        )));
        assert!(!is_installed(Path::new(DEV)));
        assert!(!is_installed(Path::new(
            "/Users/me/Sites/x/target/release/bundle/macos/Ambient Context.app/Contents/MacOS/ambient-context"
        )));
    }

    #[test]
    fn the_registered_program_is_the_first_program_argument() {
        let dir = tempfile::tempdir().unwrap();
        let plist = dir.path().join("Ambient Context.plist");
        std::fs::write(
            &plist,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
  <key>Label</key>
  <string>Ambient Context</string>
  <key>ProgramArguments</key>
  <array><string>/Users/me/Sites/ambient-context/src-tauri/target/debug/ambient-context</string></array>
  <key>RunAtLoad</key>
  <true/>
  </dict>
</plist>"#,
        )
        .unwrap();
        assert_eq!(registered_program(&plist).as_deref(), Some(Path::new(DEV)));
    }

    #[test]
    fn a_missing_or_malformed_plist_has_no_registered_program() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(registered_program(&dir.path().join("none.plist")), None);
        let bad = dir.path().join("bad.plist");
        std::fs::write(&bad, "not a plist").unwrap();
        assert_eq!(registered_program(&bad), None);
    }
}
