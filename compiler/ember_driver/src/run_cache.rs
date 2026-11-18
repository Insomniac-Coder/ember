//! The test runs between merges (`tasks/impl-0.9.9/annotations.py`): a program whose build would take
//! the same inputs, run with the same arguments and standard input, is neither built nor run again,
//! and its recorded output and status are given. Only with `run_cache_var()` set; `ember run` is
//! otherwise untouched, and the merge suite and the push gate never set it.

use super::{ExitCode, Options, Path, PathBuf, abnormal_end, exit_code_of, run_program};
use std::io::{Read, Write};
use std::process::{Command, Stdio};

/// A larger result is not kept: the folder stays small.
const LARGEST_KEPT: usize = 1 << 20;

pub(crate) struct KeptRun {
    file: PathBuf,
    input: Vec<u8>,
    result: Option<(i32, Vec<u8>, Vec<u8>)>,
}

/// Whether the run of the program `key` names has its result kept, so that it need not be built.
pub(crate) fn consider(key: &str, options: &Options, command: &str, kept: &mut Option<KeptRun>) -> bool {
    if command != "run" {
        return false;
    }
    *kept = open(key, options);
    kept.as_ref().is_some_and(|run| run.result.is_some())
}

fn open(key: &str, options: &Options) -> Option<KeptRun> {
    let dir = std::env::var_os(ember_branding::run_cache_var()).filter(|dir| !dir.is_empty())?;
    let mut input = Vec::new();
    std::io::stdin().read_to_end(&mut input).ok()?;
    let mut hash = blake3::Hasher::new();
    let mut part = |bytes: &[u8]| {
        hash.update(&(bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    part(key.as_bytes());
    for argument in &options.program_args {
        part(argument.as_bytes());
    }
    part(&input);
    let file = PathBuf::from(dir).join(format!("{}.run", hash.finalize().to_hex()));
    let result = std::fs::read(&file).ok().as_deref().and_then(parse);
    Some(KeptRun { file, input, result })
}

/// The status, then the lengths of the output and the error output, then the two.
fn parse(bytes: &[u8]) -> Option<(i32, Vec<u8>, Vec<u8>)> {
    let number = |at: usize| Some(u64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?));
    let (code, out, err) = (number(0)? as i64 as i32, number(8)? as usize, number(16)? as usize);
    let rest = bytes.get(24..)?;
    (rest.len() == out + err).then(|| (code, rest[..out].to_vec(), rest[out..].to_vec()))
}

fn store(file: &Path, code: i32, out: &[u8], err: &[u8]) {
    if out.len() + err.len() > LARGEST_KEPT {
        return;
    }
    let mut bytes = Vec::with_capacity(24 + out.len() + err.len());
    bytes.extend_from_slice(&(i64::from(code) as u64).to_le_bytes());
    bytes.extend_from_slice(&(out.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&(err.len() as u64).to_le_bytes());
    bytes.extend_from_slice(out);
    bytes.extend_from_slice(err);
    // Whole or not at all: several runs write here at once.
    let partial = file.with_extension(format!("{}.part", std::process::id()));
    if file.parent().is_some_and(|dir| std::fs::create_dir_all(dir).is_ok()) && std::fs::write(&partial, bytes).is_ok() && std::fs::rename(&partial, file).is_err() {
        let _ = std::fs::remove_file(&partial);
    }
}

/// `ember run`'s last step: the kept result, or the program, whose result is kept if it ended normally.
pub(crate) fn finish(kept: Option<KeptRun>, exe: &Path, options: &Options) -> Result<ExitCode, String> {
    let Some(run) = kept else { return run_program(exe, options) };
    let (code, out, err) = match run.result {
        Some((code, out, err)) => (Some(code), out, err),
        None => {
            let mut child = Command::new(exe)
                .args(&options.program_args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| e.to_string())?;
            let mut stdin = child.stdin.take().ok_or("the program has no standard input")?;
            let input = run.input;
            let feeding = std::thread::spawn(move || {
                let _ = stdin.write_all(&input);
            });
            let output = child.wait_with_output().map_err(|e| e.to_string())?;
            let _ = feeding.join();
            let how = abnormal_end(&output.status);
            if let (None, Some(code)) = (&how, output.status.code()) {
                store(&run.file, code, &output.stdout, &output.stderr);
            }
            let mut err = output.stderr;
            if let Some(how) = how {
                err.extend_from_slice(format!("error: {how}\n").as_bytes());
            }
            (output.status.code(), output.stdout, err)
        }
    };
    let _ = std::io::stdout().write_all(&out);
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().write_all(&err);
    Ok(exit_code_of(code))
}
