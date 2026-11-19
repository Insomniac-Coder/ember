//! A kept run is given only while the program and its input are the same.

use std::path::Path;
use std::process::{Command, Stdio};

fn run(file: &Path, kept: &Path, out_dir: &Path, stdin: &str) -> (String, bool) {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_ember"))
        .args(["run", "--out-dir"])
        .arg(out_dir)
        .arg(file)
        .env(ember_branding::run_cache_var(), kept)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the compiler starts");
    child.stdin.take().expect("its input").write_all(stdin.as_bytes()).expect("the input is written");
    let output = child.wait_with_output().expect("the compiler ends");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let built = out_dir.join("debug").join("bin").join(format!("kept{}", std::env::consts::EXE_SUFFIX)).is_file();
    (String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"), built)
}

#[test]
fn a_kept_run_is_given_only_for_the_same_program_and_input() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("run-cache");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the folder is made");
    let (file, kept) = (dir.join("kept.em"), dir.join("kept"));
    let source = |n: i32| format!("fn main():\n    println({n}, input())\n");
    std::fs::write(&file, source(1)).expect("the program is written");
    let fresh = |name: &str| dir.join(name);

    assert_eq!(run(&file, &kept, &fresh("a"), "x\n"), ("1 x\n".to_string(), true), "the first run builds");
    assert_eq!(run(&file, &kept, &fresh("b"), "x\n"), ("1 x\n".to_string(), false), "the same run is not built again");
    assert_eq!(run(&file, &kept, &fresh("c"), "y\n"), ("1 y\n".to_string(), true), "other input runs");
    std::fs::write(&file, source(2)).expect("the program changes");
    assert_eq!(run(&file, &kept, &fresh("d"), "x\n"), ("2 x\n".to_string(), true), "another program runs");
}
