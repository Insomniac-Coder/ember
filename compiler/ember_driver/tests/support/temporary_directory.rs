//! Directories owned by one test, including its generated C and executables.

use std::ffi::OsStr;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct TemporaryDirectory(PathBuf);

impl Deref for TemporaryDirectory {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsRef<Path> for TemporaryDirectory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<OsStr> for TemporaryDirectory {
    fn as_ref(&self) -> &OsStr {
        self.0.as_os_str()
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            if error.kind() != std::io::ErrorKind::NotFound {
                eprintln!("could not remove test directory {}: {error}", self.0.display());
            }
        }
    }
}

pub fn temporary_directory(label: &str) -> TemporaryDirectory {
    loop {
        // Keep the prefix short: some fixture names approach MSVC's path limit.
        // create_dir also handles stale directories after a reused process ID.
        let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "{}-t-{:x}-{sequence:x}",
            ember_branding::CLI_NAME,
            std::process::id()
        ));
        match std::fs::create_dir(&path) {
            Ok(()) => return TemporaryDirectory(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("cannot create temporary directory for {label}: {error}"),
        }
    }
}
