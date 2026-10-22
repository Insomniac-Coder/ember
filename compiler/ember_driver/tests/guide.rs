//! `[DOC-2]`, `[DOC-4]` — the user guide (`docs/book/`): every sample builds, runs and prints what
//! the guide says; and a release publishes the guide, the error pages and the specification
//! together (`tools/doc_bundle.py`, run by `.github/workflows/release.yml`).

use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "support/temporary_directory.rs"]
mod test_directories;
use test_directories::temporary_directory;

const EMBER: &str = env!("CARGO_BIN_EXE_ember");

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).expect("the workspace root").to_path_buf()
}

/// `[DOC-2]` — `ember test --doc docs/book/`: each ```` ```ember ```` sample of the guide built
/// and run, exiting with status 0 and printing what the ```` ```output ```` block after it says; each
/// ```` ```ember,fragment ```` parsed. The guide has a "coming from Python" chapter, built on
/// Appendix E.
#[test]
fn every_sample_of_the_guide_runs_and_prints_what_the_guide_says() {
    let root = workspace_root();
    let book = root.join("docs").join("book");
    let python = std::fs::read_to_string(book.join("12-coming-from-python.md")).expect("the Python chapter");
    assert!(python.contains("Appendix E"), "the Python chapter is built on Appendix E");
    let out = temporary_directory("guide");
    let run = Command::new(EMBER)
        .args(["test", "--doc", "docs/book/", "--out-dir", &out.to_string_lossy()])
        .current_dir(&root)
        .output()
        .expect("ember runs");
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(run.status.success(), "a sample of the guide fails:\n{stdout}\n{}", String::from_utf8_lossy(&run.stderr));
    let passed = stdout
        .lines()
        .find_map(|line| line.strip_prefix("example result: ok. ")?.split(' ').next()?.parse::<usize>().ok())
        .unwrap_or_else(|| panic!("no summary:\n{stdout}"));
    assert!(passed >= 50, "the guide's samples were not all found: {passed} ran\n{stdout}");
}

fn python() -> &'static str {
    if Command::new("python3").arg("--version").output().is_ok_and(|out| out.status.success()) { "python3" } else { "python" }
}

/// `[DOC-4]` — the release's documentation archive holds the guide, every error page and the
/// specification the compiler implements, under one index naming the release; the release
/// workflow builds it after every example has run, and attaches it to the release.
#[test]
fn a_release_publishes_the_guide_the_error_pages_and_the_specification_together() {
    let root = workspace_root();
    let out = temporary_directory("docs-bundle");
    let made = Command::new(python())
        .args(["tools/doc_bundle.py", "v9.9.9-test"])
        .arg(out.as_os_str())
        .current_dir(&root)
        .output()
        .expect("python runs");
    assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));

    // The archive's names, read from its central directory (no zip library is a dependency).
    let bytes = std::fs::read(out.join("ember-docs-v9.9.9-test.zip")).expect("the archive is written");
    let mut names = Vec::new();
    let mut at = 0;
    while let Some(found) = bytes[at..].windows(4).position(|window| window == b"PK\x01\x02") {
        let entry = at + found;
        let length = u16::from_le_bytes([bytes[entry + 28], bytes[entry + 29]]) as usize;
        names.push(String::from_utf8_lossy(&bytes[entry + 46..entry + 46 + length]).into_owned());
        at = entry + 46 + length;
    }
    let target = std::fs::read_to_string(root.join("docs/spec-source/development-target.json")).expect("the target");
    let spec = target.split("\"path\": \"").nth(1).and_then(|rest| rest.split('"').next()).expect("its path");
    let spec_name = Path::new(spec).file_name().expect("a file").to_string_lossy();
    assert!(names.contains(&"index.md".to_string()), "{names:?}");
    assert!(names.contains(&format!("specification/{spec_name}")), "the specification: {names:?}");
    for page in std::fs::read_dir(root.join("docs/book")).expect("the guide") {
        let name = page.expect("a page").file_name().to_string_lossy().into_owned();
        assert!(names.contains(&format!("book/{name}")), "the guide's {name}: {names:?}");
    }
    let pages = std::fs::read_dir(root.join("docs/errors")).expect("the error pages").count();
    assert_eq!(names.iter().filter(|name| name.starts_with("errors/")).count(), pages, "every error page");

    // The release builds it once every example has run, and attaches it to the release.
    let release = std::fs::read_to_string(root.join(".github/workflows/release.yml")).expect("the release workflow");
    let job = release.split("\n  documentation:").nth(1).expect("the release has a documentation job");
    let examples = job.find("ember test --doc").expect("the examples run");
    let bundle = job.find("tools/doc_bundle.py").expect("the archive is made");
    let attached = job.find("gh release").expect("the archive is attached to the release");
    assert!(examples < bundle && bundle < attached, "examples, then the archive, then the release:\n{job}");
}
