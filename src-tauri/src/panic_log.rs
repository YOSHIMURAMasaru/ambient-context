//! A panic hook that writes the message to a file before the process dies.
//!
//! A panic inside Tauri's `setup` runs within AppKit's `did_finish_launching`
//! callback, which cannot unwind, so the process aborts with a second panic
//! (`panic_cannot_unwind`) and the crash report names only that one. The
//! original message went to stderr, which a Dock or launchd launch sends
//! nowhere. The hook is the last code that runs while the message exists.

use std::io::Write;
use std::path::PathBuf;

/// One log line. `location` is `file:line:column` when the panic has one.
pub fn entry(now: &str, thread: &str, message: &str, location: Option<&str>) -> String {
    match location {
        Some(at) => format!("{now} thread '{thread}' panicked at {at}: {message}\n"),
        None => format!("{now} thread '{thread}' panicked: {message}\n"),
    }
}

/// Appends every panic to `path`, then hands over to the hook that was in
/// place before, so stderr output is unchanged. A file that cannot be
/// written is ignored: a panic hook must never itself fail.
pub fn install(path: PathBuf) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-string panic payload".to_string());
        let thread = std::thread::current();
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
        let line = entry(
            &chrono::Local::now().to_rfc3339(),
            thread.name().unwrap_or("unnamed"),
            &message,
            location.as_deref(),
        );
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entry_carries_the_time_thread_message_and_location() {
        assert_eq!(
            entry(
                "2026-09-06T19:42:33+10:00",
                "main",
                "Failed to setup app: tray",
                Some("src/lib.rs:1894:13"),
            ),
            "2026-09-06T19:42:33+10:00 thread 'main' panicked at src/lib.rs:1894:13: Failed to setup app: tray\n"
        );
    }

    #[test]
    fn an_entry_with_no_location_still_reads() {
        assert_eq!(
            entry("t", "main", "boom", None),
            "t thread 'main' panicked: boom\n"
        );
    }

    #[test]
    fn a_panic_is_written_to_the_file_with_its_message_and_location() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("panic.log");
        install(path.clone());
        let joined = std::thread::Builder::new()
            .name("panicky".into())
            .spawn(|| panic!("boom from a test"))
            .unwrap()
            .join();
        assert!(joined.is_err());
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(
            written.contains("thread 'panicky' panicked at "),
            "{written}"
        );
        assert!(written.contains("panic_log.rs"), "{written}");
        assert!(written.contains(": boom from a test\n"), "{written}");
    }
}
