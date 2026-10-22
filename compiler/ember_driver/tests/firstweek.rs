//! The first-week corpus, `tests/firstweek/`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const EMBER: &str = env!("CARGO_BIN_EXE_ember");

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is compiler/ember_driver.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root is two levels above this package")
        .to_path_buf()
}

/// §XVII.6's shapes and the codes their first error may carry (§XVII.6.1's rows; the code each
/// §XVII.6.2 shape is reported with, N3 also under `E1010`, which an unknown method gets today). An
/// empty list: the row names no code. `DIA-21` is the Python-habit table of `[DIA-21]`.
const SHAPES: &[(&str, &[&str])] = &[
    ("O1", &["E3040"]), ("O2", &["E3010", "E3011", "E3012", "E3013"]), ("O3", &["E3041"]),
    ("O4", &["E3042"]), ("O5", &["E3030"]), ("O6", &["E3050"]), ("O7", &["E3070"]),
    ("O8", &["E3014", "E3015"]), ("O9", &["E3016"]),
    ("B1", &["E3022"]), ("B2", &["E3020"]), ("B3", &["E3021"]), ("B4", &["E3023"]), ("B5", &["E3024"]),
    ("B6", &["E3062"]), ("B7", &["E3060"]), ("B8", &["E3025"]), ("B9", &["E3026"]), ("B10", &["E3027"]),
    ("B11", &["E3095"]), ("B12", &["E3063"]), ("B14", &["E3065"]), ("B15", &["E2228"]),
    ("X1", &["E3080"]), ("S1", &["E4030"]), ("A1", &["E3061", "E3090", "E3096"]), ("U1", &["E3100", "E3105"]),
    ("R1", &[]),
    ("N1", &["E1010"]), ("N2", &["E1010", "E1061"]), ("N3", &["E2072", "E1010"]), ("N4", &["E2020"]),
    ("N5", &["E2010"]), ("N6", &["E2020"]), ("N7", &[]), ("N8", &[]), ("N9", &["E2040"]),
    ("N10", &["E2050"]), ("N11", &["E2070"]), ("N12", &["E0002", "E0003", "E0004"]), ("N13", &["E2072"]),
    ("DIA-21", &[]),
];

/// Whether `shape` may be the shape of a first error with `code`: a catalogued shape whose row
/// allows the code, or `none:<code>` / `gap:<code>` naming it.
fn shape_fits(shape: &str, code: &str) -> bool {
    if let Some(named) = shape.strip_prefix("none:").or_else(|| shape.strip_prefix("gap:")) {
        return named == code;
    }
    SHAPES.iter().any(|(name, codes)| *name == shape && (codes.is_empty() || codes.contains(&code)))
}

/// The error codes `ember check --json` reports for `file`, in its order.
fn error_codes(root: &Path, file: &Path) -> Vec<String> {
    let output = ember_build::output_started(Command::new(EMBER).arg("check").arg("--json").arg(file).current_dir(root))
        .expect("ember starts");
    let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    let codes: Vec<String> = text
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line.trim()).ok())
        .filter(|d| d["severity"] == "error")
        .map(|d| d["code"].as_str().unwrap_or("none").to_string())
        .collect();
    assert_eq!(output.status.success(), codes.is_empty(), "{}: exit status and errors disagree:\n{text}", file.display());
    codes
}

/// `ember check` on every file, on as many threads as the machine has.
fn check_all(root: &Path, files: &[PathBuf]) -> Vec<Vec<String>> {
    let next = AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).min(files.len().max(1));
    let mut results: Vec<(usize, Vec<String>)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| scope.spawn(|| {
                let mut done = Vec::new();
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(file) = files.get(i) else { break done };
                    done.push((i, error_codes(root, file)));
                }
            }))
            .collect();
        workers.into_iter().flat_map(|w| w.join().expect("a check thread")).collect()
    });
    results.sort_by_key(|(i, _)| *i);
    results.into_iter().map(|(_, codes)| codes).collect()
}

/// `Acceptance rate: A of N as written, B of N after the helps.` in `README.md`, and the history
/// table's last row.
fn published_rate(readme: &str) -> ((usize, usize, usize), (usize, usize, usize)) {
    let numbers = |text: &str| -> Vec<usize> {
        text.split(|c: char| !c.is_ascii_digit()).filter(|s| !s.is_empty()).filter_map(|s| s.parse().ok()).collect()
    };
    let line = readme
        .lines()
        .find(|l| l.starts_with("Acceptance rate: "))
        .expect("README.md publishes `Acceptance rate: A of N as written, B of N after the helps.`");
    let [a, n, b, m] = numbers(line)[..] else { panic!("cannot read the published rate: {line}") };
    assert_eq!(n, m, "{line}");
    let last = readme.lines().filter(|l| l.starts_with("| 20")).last().expect("README.md keeps the rate's history");
    let cells: Vec<&str> = last.split('|').map(str::trim).collect();
    let (written, helped) = (numbers(cells[3]), numbers(cells[4]));
    let [hw, hn] = written[..] else { panic!("cannot read the history row: {last}") };
    let [hb, _] = helped[..] else { panic!("cannot read the history row: {last}") };
    ((a, b, n), (hw, hb, hn))
}

/// `[TST-8]` `[TST-9]` `[TST-10]` — `tests/firstweek/` holds at least 24 first drafts a newcomer
/// writes in week one, never edited. Each one's `ember check` outcome is its mark in `marks.json`:
/// no error for `accepted`; for `rejected`, exactly the recorded codes, the first fitting the recorded
/// shape. Each `helped/` copy (the program with its chain of helps applied) checks clean. The
/// acceptance rate the marks give — accepted of all, and accepted after the catalogue's helps — is
/// the one `README.md` publishes, and its history's last row; when it moves, this test says which
/// way, and the README gets a line saying why.
#[test]
fn the_first_week_corpus_keeps_its_marks() {
    let root = workspace_root();
    let dir = root.join("tests").join("firstweek");
    let marks: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(dir.join("marks.json")).expect("tests/firstweek/marks.json"))
            .expect("marks.json is a JSON object");
    let source_ext = format!(".{}", ember_branding::SOURCE_EXT);
    let mut programs: Vec<String> = std::fs::read_dir(&dir)
        .expect("tests/firstweek is readable")
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
        .filter(|name| name.ends_with(&source_ext))
        .collect();
    programs.sort();
    assert!(programs.len() >= 24, "[TST-8] needs at least 24 programs, found {}", programs.len());
    assert_eq!(programs, marks.keys().cloned().collect::<Vec<_>>(), "every program has a mark and every mark a program");

    for name in &programs {
        let text = std::fs::read_to_string(dir.join(name)).expect("readable");
        assert!(text.starts_with("# "), "{name}: the first line says what the program does");
        assert!(!text.lines().any(|l| l.starts_with("#$")), "{name}: a first draft has no `#$` lines");
    }

    let rejected: Vec<&String> = programs.iter().filter(|p| marks[*p]["mark"] == "rejected").collect();
    let mut helped: Vec<String> = std::fs::read_dir(dir.join("helped"))
        .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned())).collect())
        .unwrap_or_default();
    helped.sort();
    assert_eq!(helped.iter().collect::<Vec<_>>(), rejected, "helped/ holds a copy of each rejected program and nothing else");

    let mut files: Vec<PathBuf> = programs.iter().map(|p| dir.join(p)).collect();
    files.extend(helped.iter().map(|p| dir.join("helped").join(p)));
    let outcomes = check_all(&root, &files);
    let mut failures = Vec::new();
    for (name, codes) in programs.iter().zip(&outcomes) {
        let mark = &marks[name];
        match mark["mark"].as_str() {
            Some("accepted") if codes.is_empty() => {}
            Some("accepted") => failures.push(format!("{name}: marked accepted, now rejected with {codes:?}")),
            Some("rejected") => {
                let recorded: BTreeSet<&str> = mark["codes"].as_array().map(|c| c.iter().filter_map(|c| c.as_str()).collect()).unwrap_or_default();
                let got: BTreeSet<&str> = codes.iter().map(String::as_str).collect();
                let shape = mark["shape"].as_str().unwrap_or("");
                if codes.is_empty() {
                    failures.push(format!("{name}: marked rejected ({shape}), now accepted"));
                } else if got != recorded {
                    failures.push(format!("{name}: marked rejected with {recorded:?}, now {got:?}"));
                } else if !shape_fits(shape, &codes[0]) {
                    failures.push(format!("{name}: shape `{shape}` does not fit the first error, {}", codes[0]));
                }
            }
            other => failures.push(format!("{name}: mark {other:?} is neither accepted nor rejected")),
        }
    }
    for (name, codes) in helped.iter().zip(&outcomes[programs.len()..]) {
        if !codes.is_empty() {
            failures.push(format!("helped/{name}: the helps no longer make it compile: {codes:?}"));
        }
    }
    assert!(failures.is_empty(), "[TST-9] the marks no longer hold (update marks.json, helped/ and the README):\n{}", failures.join("\n"));

    let total = programs.len();
    let accepted = programs.iter().filter(|p| marks[*p]["mark"] == "accepted").count();
    let after = accepted + programs.iter().filter(|p| marks[*p]["after_helps"] == "accepted").count();
    let blocking = programs
        .iter()
        .filter(|p| marks[*p]["helps"].as_array().is_some_and(|h| h.iter().filter_map(|s| s.as_str()).any(|s| s.contains("none:") || s.contains("gap:"))))
        .count();
    println!("first week: {accepted} of {total} accepted as written, {after} after the helps; {blocking} meet a rejection in no catalogue or a compiler gap (release blockers, [TST-9])");

    let readme = std::fs::read_to_string(dir.join("README.md")).expect("tests/firstweek/README.md");
    let (published, history) = published_rate(&readme);
    let moved = |what: &str, was: usize, now: usize| match now.cmp(&was) {
        std::cmp::Ordering::Equal => None,
        std::cmp::Ordering::Greater => Some(format!("{what} rose from {was} to {now} of {total}: publish it in README.md with a history line")),
        std::cmp::Ordering::Less => Some(format!("{what} fell from {was} to {now} of {total}: publish it in README.md with a history line saying why")),
    };
    let changes: Vec<String> = [moved("the rate as written", published.0, accepted), moved("the rate after the helps", published.1, after)]
        .into_iter()
        .flatten()
        .collect();
    assert!(changes.is_empty() && published.2 == total, "[TST-10] {}{}", changes.join("; "),
        if published.2 == total { String::new() } else { format!(" (the README counts {} programs, the corpus {total})", published.2) });
    assert_eq!(history, published, "[TST-10] the history's last row is the published rate");
}
