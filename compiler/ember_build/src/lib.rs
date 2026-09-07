//! The build driver: finding a C toolchain and running it (Part XIX §3).
//!
//! `[BLD-5]` — output goes to `target/<profile>/{bin,c,obj}`.
//!
//! Phase 0 compiles one `.em` file at a time and links it with `ember_rt`.
//! The manifest, the build graph, content-addressed caching and the
//! Ninja-driven incremental C build (`[BLD-1]`..`[BLD-4]`) arrive with the
//! module system in Phase 1, which is the first phase that has more than one
//! translation unit to order.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Profile {
    #[default]
    Debug,
    Release,
    Shipping,
}

impl Profile {
    pub fn name(self) -> &'static str {
        match self {
            Profile::Debug => "debug",
            Profile::Release => "release",
            Profile::Shipping => "shipping",
        }
    }

    pub fn from_name(name: &str) -> Option<Profile> {
        Some(match name {
            "debug" => Profile::Debug,
            "release" => Profile::Release,
            "shipping" => Profile::Shipping,
            _ => return None,
        })
    }
}

/// Which C compiler drives the backend. `[MAN-1]`'s `c_compiler = "auto"`
/// resolves to MSVC on Windows when it can be found, then clang, then gcc.
#[derive(Clone, Debug)]
pub enum Toolchain {
    Msvc {
        cl: PathBuf,
        /// The environment `vcvars64.bat` sets. `cl.exe` cannot find its own
        /// headers or libraries without `INCLUDE` and `LIB`, and there is no
        /// flag that substitutes for them.
        env: BTreeMap<String, String>,
    },
    Clang(PathBuf),
    Gcc(PathBuf),
}

impl Toolchain {
    pub fn name(&self) -> &'static str {
        match self {
            Toolchain::Msvc { .. } => "msvc",
            Toolchain::Clang(_) => "clang",
            Toolchain::Gcc(_) => "gcc",
        }
    }

    /// Find a usable C compiler, preferring `requested` when it is given.
    pub fn detect(requested: Option<&str>) -> Result<Toolchain, BuildError> {
        match requested {
            Some("msvc") => Self::find_msvc().ok_or(BuildError::NoToolchain("msvc")),
            Some("clang") => Self::find_on_path("clang").map(Toolchain::Clang).ok_or(BuildError::NoToolchain("clang")),
            Some("gcc") => Self::find_on_path("gcc").map(Toolchain::Gcc).ok_or(BuildError::NoToolchain("gcc")),
            Some(other) => Err(BuildError::UnknownToolchain(other.to_string())),
            None => Self::find_msvc()
                .or_else(|| Self::find_on_path("clang").map(Toolchain::Clang))
                .or_else(|| Self::find_on_path("gcc").map(Toolchain::Gcc))
                .ok_or(BuildError::NoToolchain("any")),
        }
    }

    fn find_on_path(name: &str) -> Option<PathBuf> {
        let exe = if cfg!(windows) { format!("{name}.exe") } else { name.to_string() };
        // A bare name works when the program is on PATH; test it by asking for
        // its version rather than by scanning PATH ourselves.
        let ok = Command::new(&exe)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            return Some(PathBuf::from(exe));
        }
        // The LLVM installer does not add itself to PATH on Windows.
        if cfg!(windows) && name == "clang" {
            for root in ["C:\\Program Files\\LLVM", "C:\\Program Files (x86)\\LLVM"] {
                let candidate = Path::new(root).join("bin").join("clang.exe");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    fn find_msvc() -> Option<Toolchain> {
        if !cfg!(windows) {
            return None;
        }
        let vcvars = Self::find_vcvars()?;
        let env = capture_environment(&vcvars)?;
        // With the captured environment, `cl` resolves through its own PATH.
        Some(Toolchain::Msvc { cl: PathBuf::from("cl.exe"), env })
    }

    fn find_vcvars() -> Option<PathBuf> {
        let roots = [
            "C:\\Program Files\\Microsoft Visual Studio\\2022",
            "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022",
            "C:\\Program Files\\Microsoft Visual Studio\\2019",
            "C:\\Program Files (x86)\\Microsoft Visual Studio\\2019",
        ];
        let editions = ["BuildTools", "Community", "Professional", "Enterprise"];
        for root in roots {
            for edition in editions {
                let candidate = Path::new(root)
                    .join(edition)
                    .join("VC\\Auxiliary\\Build\\vcvars64.bat");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }
}

/// Run `vcvars64.bat` and read back the environment it set.
fn capture_environment(vcvars: &Path) -> Option<BTreeMap<String, String>> {
    let output = Command::new("cmd")
        .arg("/c")
        .arg(format!("call \"{}\" >nul 2>&1 && set", vcvars.display()))
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut env = BTreeMap::new();
    for line in text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            env.insert(key.to_string(), value.to_string());
        }
    }
    // If INCLUDE is missing the batch file did not really run, and cl.exe
    // would fail later with a confusing "cannot open stdio.h".
    env.contains_key("INCLUDE").then_some(env)
}

#[derive(Debug)]
pub enum BuildError {
    NoToolchain(&'static str),
    UnknownToolchain(String),
    Io(std::io::Error),
    /// The C compiler ran and failed. Its own output is carried through, since
    /// a failure here is a compiler defect (`[CG-C-1]`) and the message is the
    /// evidence.
    CompilerFailed { command: String, output: String },
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::NoToolchain(which) => {
                write!(f, "no C compiler found (looked for: {which})")
            }
            BuildError::UnknownToolchain(name) => write!(f, "unknown C compiler `{name}`"),
            BuildError::Io(e) => write!(f, "{e}"),
            BuildError::CompilerFailed { command, output } => {
                write!(f, "the C compiler failed\n  command: {command}\n{output}")
            }
        }
    }
}

impl From<std::io::Error> for BuildError {
    fn from(e: std::io::Error) -> BuildError {
        BuildError::Io(e)
    }
}

/// Where a build writes its artefacts (`[BLD-5]`).
pub struct Layout {
    pub root: PathBuf,
    pub c: PathBuf,
    pub obj: PathBuf,
    pub bin: PathBuf,
}

impl Layout {
    pub fn new(target_dir: &Path, profile: Profile) -> Result<Layout, BuildError> {
        let root = target_dir.join(profile.name());
        let layout = Layout {
            c: root.join("c"),
            obj: root.join("obj"),
            bin: root.join("bin"),
            root,
        };
        std::fs::create_dir_all(&layout.c)?;
        std::fs::create_dir_all(&layout.obj)?;
        std::fs::create_dir_all(&layout.bin)?;
        Ok(layout)
    }
}

pub struct LinkRequest<'a> {
    pub sources: &'a [PathBuf],
    pub include_dirs: &'a [PathBuf],
    pub output: PathBuf,
    pub profile: Profile,
    pub obj_dir: PathBuf,
}

/// Compile and link in one invocation. Phase 0 has one translation unit plus
/// the runtime, so separate compilation buys nothing yet; `[BLD-4]`'s
/// Ninja-driven incremental build arrives with the module system.
pub fn compile_and_link(toolchain: &Toolchain, request: &LinkRequest) -> Result<(), BuildError> {
    let mut command = match toolchain {
        Toolchain::Msvc { cl, env } => {
            let mut c = Command::new(cl);
            c.env_clear();
            for (key, value) in env {
                c.env(key, value);
            }
            c
        }
        Toolchain::Clang(path) => Command::new(path),
        Toolchain::Gcc(path) => Command::new(path),
    };

    match toolchain {
        Toolchain::Msvc { .. } => {
            command.arg("/nologo").arg("/std:c11").arg("/W3");
            match request.profile {
                Profile::Debug => {
                    command.arg("/Od").arg("/Zi").arg("/MDd");
                }
                Profile::Release => {
                    command.arg("/O2").arg("/MD");
                }
                Profile::Shipping => {
                    command.arg("/O2").arg("/GL").arg("/MD");
                }
            }
            for dir in request.include_dirs {
                command.arg(format!("/I{}", dir.display()));
            }
            for source in request.sources {
                command.arg(source);
            }
            // Object files land in obj/, keeping the C directory readable.
            command.arg(format!("/Fo{}\\", request.obj_dir.display()));
            command.arg(format!("/Fe{}", request.output.display()));
            if request.profile == Profile::Debug {
                command.arg(format!("/Fd{}\\", request.obj_dir.display()));
            }
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.arg("-std=c11").arg("-Wall").arg("-Wextra");
            match request.profile {
                Profile::Debug => {
                    command.arg("-O0").arg("-g");
                }
                Profile::Release => {
                    command.arg("-O2");
                }
                Profile::Shipping => {
                    command.arg("-O3");
                }
            }
            for dir in request.include_dirs {
                command.arg("-I").arg(dir);
            }
            for source in request.sources {
                command.arg(source);
            }
            command.arg("-o").arg(&request.output);
            if !cfg!(windows) {
                command.arg("-lm");
            }
        }
    }

    let rendered = format!("{command:?}");
    let output = command.output()?;
    if !output.status.success() {
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        return Err(BuildError::CompilerFailed { command: rendered, output: text });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_names_round_trip() {
        for profile in [Profile::Debug, Profile::Release, Profile::Shipping] {
            assert_eq!(Profile::from_name(profile.name()), Some(profile));
        }
        assert_eq!(Profile::from_name("fast"), None);
    }

    #[test]
    fn a_toolchain_is_found_on_this_machine() {
        // Phase 0's exit criterion needs one; if this fails, the environment
        // is the problem, not the compiler.
        let found = Toolchain::detect(None);
        assert!(found.is_ok(), "no C toolchain: {found:?}");
    }
}
