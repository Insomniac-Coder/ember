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

pub mod interface;

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
        /// What `vcvars64.bat` sets, applied over the inherited environment.
        /// `cl.exe` cannot find its own headers or libraries without `INCLUDE`
        /// and `LIB`, and there is no flag that substitutes for them.
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
            Some("clang-cl") => Self::find_clang_cl().ok_or(BuildError::NoToolchain("clang-cl")),
            Some("clang") => Self::find_on_path("clang")
                .map(Toolchain::Clang)
                .ok_or(BuildError::NoToolchain("clang")),
            Some("gcc") => Self::find_on_path("gcc")
                .map(Toolchain::Gcc)
                .ok_or(BuildError::NoToolchain("gcc")),
            Some(other) if other != "auto" => Err(BuildError::UnknownToolchain(other.to_string())),
            _ => Self::find_msvc()
                .or_else(|| Self::find_on_path("clang").map(Toolchain::Clang))
                .or_else(|| Self::find_on_path("gcc").map(Toolchain::Gcc))
                .ok_or(BuildError::NoToolchain("any")),
        }
    }

    fn find_on_path(name: &str) -> Option<PathBuf> {
        let exe = if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_string()
        };
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
        if cfg!(windows) && name.starts_with("clang") {
            for root in ["C:\\Program Files\\LLVM", "C:\\Program Files (x86)\\LLVM"] {
                let candidate = Path::new(root).join("bin").join(&exe);
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
        let env = msvc_environment(&vcvars)?;
        // With the captured environment, `cl` resolves through its own PATH.
        Some(Toolchain::Msvc {
            cl: PathBuf::from("cl.exe"),
            env,
        })
    }

    /// clang's MSVC-compatible driver: MSVC's environment and flags, clang's
    /// front end. It ignores `/GL`, so `shipping` gets no link-time code
    /// generation from it.
    fn find_clang_cl() -> Option<Toolchain> {
        let Toolchain::Msvc { env, .. } = Self::find_msvc()? else {
            return None;
        };
        Some(Toolchain::Msvc {
            cl: Self::find_on_path("clang-cl")?,
            env,
        })
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

/// What `vcvars64.bat` sets, read from the global cache when the batch file
/// has been run before: running it costs over a second, on every build. The
/// key covers the batch file and the toolset version it selects, so an update
/// to either runs it again, and so does a cached `INCLUDE` folder that is gone.
fn msvc_environment(vcvars: &Path) -> Option<BTreeMap<String, String>> {
    let mut key = blake3::Hasher::new();
    key.update(vcvars.to_string_lossy().as_bytes());
    for file in [vcvars.to_path_buf(), vcvars.with_file_name("Microsoft.VCToolsVersion.default.txt")] {
        if let Ok(meta) = file.metadata() {
            key.update(&meta.len().to_le_bytes());
            key.update(&modified_nanos(&meta).to_le_bytes());
        }
    }
    let file = cache_root().join("msvc").join(format!("{}.env", key.finalize().to_hex()));
    if let Some(env) = read_environment(&file) {
        return Some(env);
    }
    let env = capture_environment(vcvars)?;
    let text: String = env.iter().map(|(name, value)| format!("{name}={value}\n")).collect();
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
        let temp = file.with_extension(format!("tmp{}", std::process::id()));
        if std::fs::write(&temp, text).is_ok() && std::fs::rename(&temp, &file).is_err() {
            let _ = std::fs::remove_file(&temp);
        }
    }
    Some(env)
}

/// A cached environment, if every `INCLUDE` folder it names still exists.
fn read_environment(file: &Path) -> Option<BTreeMap<String, String>> {
    let env: BTreeMap<String, String> = std::fs::read_to_string(file)
        .ok()?
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect();
    let include = env.iter().find(|(name, _)| name.eq_ignore_ascii_case("INCLUDE"))?.1;
    include
        .split(';')
        .filter(|dir| !dir.is_empty())
        .all(|dir| Path::new(dir).is_dir())
        .then_some(env)
}

fn modified_nanos(meta: &std::fs::Metadata) -> u128 {
    meta.modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |since| since.as_nanos())
}

/// Run `vcvars64.bat` and read back what it set: the variables it changed,
/// and always the four a compile needs. The rest is inherited at each build,
/// so the cache keeps no copy of this process's other variables.
fn capture_environment(vcvars: &Path) -> Option<BTreeMap<String, String>> {
    let line = format!("/c call \"{}\" >nul 2>&1 && set", vcvars.display());
    let mut command = Command::new("cmd");
    // D-251 — passed as written. `arg` would escape the quotes around the
    // path as `\"`, which `cmd` does not read, so the batch file never ran and
    // MSVC was never found.
    #[cfg(windows)]
    std::os::windows::process::CommandExt::raw_arg(&mut command, &line);
    #[cfg(not(windows))]
    command.arg(&line);
    let output = command.output().ok()?;
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
    if !env.contains_key("INCLUDE") {
        return None;
    }
    let needed = ["INCLUDE", "LIB", "LIBPATH", "PATH"];
    env.retain(|name, value| {
        needed.iter().any(|n| n.eq_ignore_ascii_case(name))
            || std::env::var(name.as_str()).ok().as_deref() != Some(value.as_str())
    });
    Some(env)
}

#[derive(Debug)]
pub enum BuildError {
    NoToolchain(&'static str),
    UnknownToolchain(String),
    Io(std::io::Error),
    /// The C compiler ran and failed. Its own output is carried through, since
    /// a failure here is a compiler defect (`[CG-C-1]`) and the message is the
    /// evidence.
    CompilerFailed {
        command: String,
        output: String,
    },
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
    pub inspect: PathBuf,
}

impl Layout {
    pub fn new(target_dir: &Path, profile: Profile) -> Result<Layout, BuildError> {
        let root = target_dir.join(profile.name());
        let layout = Layout {
            c: root.join("c"),
            obj: root.join("obj"),
            bin: root.join("bin"),
            inspect: root.join("inspect"),
            root,
        };
        std::fs::create_dir_all(&layout.c)?;
        std::fs::create_dir_all(&layout.obj)?;
        std::fs::create_dir_all(&layout.bin)?;
        std::fs::create_dir_all(&layout.inspect)?;
        Ok(layout)
    }
}

pub struct LinkRequest<'a> {
    /// C sources, and objects to link with them (see [`runtime_object`]).
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
    let mut command = compiler_command(toolchain);
    match toolchain {
        Toolchain::Msvc { .. } => {
            command.args(msvc_flags(request.profile));
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
            command.args(gnu_flags(request.profile));
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
    run(command)
}

/// The C compiler, with MSVC's environment over the inherited one.
fn compiler_command(toolchain: &Toolchain) -> Command {
    match toolchain {
        Toolchain::Msvc { cl, env } => {
            let mut command = Command::new(cl);
            command.envs(env);
            command
        }
        Toolchain::Clang(path) | Toolchain::Gcc(path) => Command::new(path),
    }
}

/// cl's flags for a profile. The release C runtime in every profile, as clang
/// builds and Rust's use: the debug one (`/MDd`) reports a failed check or an
/// abort() in a window that waits for a click, which stops any unattended run,
/// and it needs Visual Studio's own DLLs to start at all (D-252).
fn msvc_flags(profile: Profile) -> Vec<&'static str> {
    let optimisation: &[&str] = match profile {
        Profile::Debug => &["/Od", "/Zi", "/MD"],
        Profile::Release => &["/O2", "/MD"],
        Profile::Shipping => &["/O2", "/GL", "/MD"],
    };
    ["/nologo", "/std:c11", "/W3"].iter().chain(optimisation).copied().collect()
}

/// clang's and gcc's flags for a profile, shared by the program's compile and
/// the runtime object's so the two always agree.
fn gnu_flags(profile: Profile) -> Vec<&'static str> {
    let optimisation: &[&str] = match profile {
        Profile::Debug => &["-O0", "-g"],
        Profile::Release => &["-O2"],
        Profile::Shipping => &["-O3"],
    };
    ["-std=c11", "-Wall", "-Wextra"].iter().chain(optimisation).copied().collect()
}

fn run(mut command: Command) -> Result<(), BuildError> {
    let rendered = format!("{command:?}");
    let output = command.output()?;
    if !output.status.success() {
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        return Err(BuildError::CompilerFailed {
            command: rendered,
            output: text,
        });
    }
    Ok(())
}

/// The global build cache, shared by every build on the machine: the variable
/// named by `cache_dir_var()`, else the platform's cache directory, else the
/// temporary directory. Zig and Go keep their build caches the same way.
pub fn cache_root() -> PathBuf {
    let name = ember_branding::CLI_NAME;
    let env_dir = |var: &str| std::env::var_os(var).filter(|v| !v.is_empty()).map(PathBuf::from);
    if let Some(dir) = env_dir(&ember_branding::cache_dir_var()) {
        return dir;
    }
    let platform = if cfg!(windows) {
        env_dir("LOCALAPPDATA").map(|dir| dir.join(name).join("cache"))
    } else if cfg!(target_os = "macos") {
        env_dir("HOME").map(|dir| dir.join("Library").join("Caches").join(name))
    } else {
        env_dir("XDG_CACHE_HOME")
            .or_else(|| env_dir("HOME").map(|dir| dir.join(".cache")))
            .map(|dir| dir.join(name))
    };
    platform.unwrap_or_else(|| std::env::temp_dir().join(format!("{name}-cache")))
}

/// The runtime compiled once per toolchain and profile into `cache`, to be
/// linked into every program instead of recompiled with each (about 145 ms
/// of every clang build). The object's name is a hash of everything that
/// shapes it: the compile command (with MSVC's environment), the source, the
/// headers in `include_dirs` and the compiler's file on disk. It is compiled
/// under a temporary name and renamed into place, so a build running
/// alongside sees the whole object or none.
///
/// `None` means: compile `source` with the program. That is MSVC's
/// `shipping` path (`/GL` compiles for link-time code generation, which wants
/// the program and the runtime compiled together), and the fallback when
/// `cache` cannot be created.
pub fn runtime_object(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    profile: Profile,
    cache: &Path,
) -> Result<Option<PathBuf>, BuildError> {
    let mut command = compiler_command(toolchain);
    let extension = match toolchain {
        Toolchain::Msvc { .. } if profile == Profile::Shipping => return Ok(None),
        Toolchain::Msvc { .. } => {
            // `/Z7` keeps the debug information inside the object; `/Zi`
            // would tie a shared object to a PDB file beside it.
            let flags = msvc_flags(profile).into_iter().map(|f| if f == "/Zi" { "/Z7" } else { f });
            command.args(flags).arg("/c");
            for dir in include_dirs {
                command.arg(format!("/I{}", dir.display()));
            }
            "obj"
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.args(gnu_flags(profile)).arg("-c");
            for dir in include_dirs {
                command.arg("-I").arg(dir);
            }
            "o"
        }
    };
    command.arg(source);

    let mut key = blake3::Hasher::new();
    let mut part = |bytes: &[u8]| {
        key.update(&(bytes.len() as u64).to_le_bytes());
        key.update(bytes);
    };
    part(format!("{command:?}").as_bytes());
    part(&std::fs::read(source)?);
    // ponytail: the include directories' own files, not their subdirectories
    // (the runtime's headers are flat); walk deeper if that changes.
    for dir in include_dirs {
        let mut headers: Vec<PathBuf> = std::fs::read_dir(dir)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_file())
            .collect();
        headers.sort();
        for header in headers {
            part(header.to_string_lossy().as_bytes());
            part(&std::fs::read(&header)?);
        }
    }
    // An upgraded compiler, or another one first on PATH, is a new key.
    if let Some(file) = compiler_file(toolchain) {
        part(file.to_string_lossy().as_bytes());
        if let Ok(meta) = file.metadata() {
            part(&meta.len().to_le_bytes());
            part(&modified_nanos(&meta).to_le_bytes());
        }
    }

    let dir = cache.join("runtime");
    if std::fs::create_dir_all(&dir).is_err() {
        return Ok(None);
    }
    let name = key.finalize().to_hex();
    let object = dir.join(format!("{name}.{extension}"));
    if object.is_file() {
        return Ok(Some(object));
    }
    let temp = dir.join(format!("{name}.tmp{}.{extension}", std::process::id()));
    match toolchain {
        Toolchain::Msvc { .. } => command.arg(format!("/Fo{}", temp.display())),
        Toolchain::Clang(_) | Toolchain::Gcc(_) => command.arg("-o").arg(&temp),
    };
    if let Err(error) = run(command) {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }
    if let Err(error) = std::fs::rename(&temp, &object) {
        let _ = std::fs::remove_file(&temp);
        // Another build renamed its identical object into place first (and
        // Windows refuses to replace a file that a linker has open).
        if !object.is_file() {
            return Err(error.into());
        }
    }
    Ok(Some(object))
}

/// The compiler's file on disk, found as `Command` finds a bare name: on
/// MSVC's `PATH` for `cl.exe`, else on this process's.
fn compiler_file(toolchain: &Toolchain) -> Option<PathBuf> {
    let (program, search) = match toolchain {
        Toolchain::Msvc { cl, env } => (
            cl,
            env.iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("PATH"))
                .map(|(_, value)| std::ffi::OsString::from(value)),
        ),
        Toolchain::Clang(path) | Toolchain::Gcc(path) => (path, None),
    };
    if program.components().count() > 1 {
        return Some(program.clone());
    }
    std::env::split_paths(&search.or_else(|| std::env::var_os("PATH"))?)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
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

    /// Compiled once, reused while nothing changes, and rebuilt when a header
    /// changes.
    #[test]
    fn the_runtime_object_is_reused_until_an_input_changes() {
        let toolchain = Toolchain::detect(None).expect("a C toolchain");
        let dir = std::env::temp_dir().join(format!("runtime-object-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("include")).expect("the test directory is creatable");
        let source = dir.join("rt.c");
        std::fs::write(&source, "int rt_answer(void) { return 42; }\n").expect("the source is writable");
        let includes = [dir.join("include")];
        let build = || {
            runtime_object(&toolchain, &source, &includes, Profile::Debug, &dir.join("cache"))
                .expect("the runtime compiles")
        };
        // MSVC compiles the runtime with each program instead.
        let Some(first) = build() else { return };
        let stamp = || std::fs::metadata(&first).and_then(|m| m.modified()).expect("the object exists");
        let built = stamp();
        assert_eq!(build().as_ref(), Some(&first));
        assert_eq!(stamp(), built, "an unchanged runtime was compiled again");
        std::fs::write(dir.join("include").join("rt.h"), "#define RT 1\n").expect("the header is writable");
        let second = build().expect("the object is built");
        assert_ne!(second, first, "a changed header kept the old object");
        assert!(second.is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A cached MSVC environment is used only while its `INCLUDE` folders
    /// exist; otherwise `vcvars64.bat` runs again.
    #[test]
    fn a_cached_msvc_environment_needs_its_include_folders() {
        let dir = std::env::temp_dir().join(format!("msvc-environment-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("include")).expect("the test directory is creatable");
        let file = dir.join("environment");
        let include = dir.join("include").display().to_string();
        std::fs::write(&file, format!("INCLUDE={include};\nLIB=libs\n")).expect("the file is writable");
        let env = read_environment(&file).expect("a cached environment whose folders exist");
        assert_eq!(env.get("LIB").map(String::as_str), Some("libs"));
        let gone = dir.join("gone").display().to_string();
        std::fs::write(&file, format!("INCLUDE={include};{gone}\n")).expect("the file is writable");
        assert!(read_environment(&file).is_none(), "a missing INCLUDE folder must mean running vcvars64.bat again");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_toolchain_is_found_on_this_machine() {
        // Phase 0's exit criterion needs one; if this fails, the environment
        // is the problem, not the compiler.
        let found = Toolchain::detect(None);
        assert!(found.is_ok(), "no C toolchain: {found:?}");
    }
}
