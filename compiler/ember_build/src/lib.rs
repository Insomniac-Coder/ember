//! The build driver: finding a C toolchain and running it (Part XIX §3).
//!
//! `[BLD-5]` — output goes to `target/<profile>/{bin,lib,c,obj,inspect}`.
//!
//! The driver emits one C translation unit for a package's reachable modules.
//! Binaries link it with the runtime; static libraries archive package and
//! runtime objects separately. A general build graph and Ninja-driven
//! incremental C build (`[BLD-1]`..`[BLD-4]`) remain future work.

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
        // Already a Visual Studio developer environment (a Developer Prompt,
        // or CI's msvc-dev-cmd): used as it is, as the one the user chose.
        // Only an x64 one: the plain Developer Prompt targets x86, and the
        // runtime and the emitted C are 64-bit, which is what vcvars64.bat
        // gives otherwise.
        let here = Toolchain::Msvc { cl: PathBuf::from("cl.exe"), env: BTreeMap::new() };
        let x64 = std::env::var("VSCMD_ARG_TGT_ARCH")
            .is_ok_and(|arch| arch.eq_ignore_ascii_case("x64") || arch.eq_ignore_ascii_case("amd64"));
        if x64
            && std::env::var_os("VCINSTALLDIR").is_some()
            && std::env::var_os("INCLUDE").is_some()
            && std::env::var_os("LIB").is_some()
            && compiler_file(&here).is_some()
        {
            return Some(here);
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
/// The cache holds only what the batch file put on PATH; this process's PATH
/// follows it (D-255), so a later change to PATH is never masked.
fn msvc_environment(vcvars: &Path) -> Option<BTreeMap<String, String>> {
    let mut env = cached_msvc_environment(vcvars)?;
    if let (Some((_, path)), Some(current)) =
        (env.iter_mut().find(|(name, _)| name.eq_ignore_ascii_case("PATH")), std::env::var_os("PATH"))
    {
        path.push(';');
        path.push_str(&current.to_string_lossy());
    }
    Some(env)
}

fn cached_msvc_environment(vcvars: &Path) -> Option<BTreeMap<String, String>> {
    let mut key = blake3::Hasher::new();
    // v3: captured without the calling shell's Visual Studio variables, and
    // PATH holds only the batch file's own directories (D-255).
    key.update(b"v3\0");
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
    // The batch files extend what they inherit (INCLUDE, LIB, an existing
    // VSINSTALLDIR), so a shell's own Visual Studio variables would end up in
    // the machine-wide cache. The child gets none of them, and what the batch
    // files set is measured against the child's input, not this process.
    let visual_studio = |name: &str| {
        let upper = name.to_ascii_uppercase();
        ["INCLUDE", "LIB", "LIBPATH", "EXTERNAL_INCLUDE", "VSINSTALLDIR", "VCINSTALLDIR", "DEVENVDIR", "PLATFORM"]
            .contains(&upper.as_str())
            || upper.starts_with("VSCMD_")
            || upper.starts_with("__VSCMD_")
            || (upper.starts_with("VS") && upper.ends_with("COMNTOOLS"))
    };
    let mut input: BTreeMap<String, String> = BTreeMap::new();
    for (name, value) in std::env::vars_os() {
        let name = name.to_string_lossy().into_owned();
        if visual_studio(&name) {
            command.env_remove(&name);
        } else {
            input.insert(name.to_ascii_uppercase(), value.to_string_lossy().into_owned());
        }
    }
    // D-255 — the batch files build PATH on single `cmd` lines, which stop at
    // 8,191 characters, so a long PATH made them fail. They run on Windows'
    // own short PATH instead, and `msvc_environment` appends this process's
    // PATH to what they add. Telemetry is skipped: it ends with `START
    // powershell.exe`, which opens a "cannot find" window when PowerShell is
    // not on PATH.
    let system = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string());
    let short = format!("{system}\\System32;{system};{system}\\System32\\Wbem;{system}\\System32\\WindowsPowerShell\\v1.0");
    command.env("PATH", &short);
    input.insert("PATH".to_string(), short);
    command.env("VSCMD_SKIP_SENDTELEMETRY", "1");
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
        let upper = name.to_ascii_uppercase();
        needed.contains(&upper.as_str())
            || (upper != "VSCMD_SKIP_SENDTELEMETRY" && input.get(&upper) != Some(value))
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
    ArchiverFailed {
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
            BuildError::ArchiverFailed { command, output } => {
                write!(f, "the archive tool failed\n  command: {command}\n{output}")
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
    pub lib: PathBuf,
    pub inspect: PathBuf,
}

impl Layout {
    pub fn new(target_dir: &Path, profile: Profile) -> Result<Layout, BuildError> {
        let root = target_dir.join(profile.name());
        let layout = Layout {
            c: root.join("c"),
            obj: root.join("obj"),
            bin: root.join("bin"),
            lib: root.join("lib"),
            inspect: root.join("inspect"),
            root,
        };
        std::fs::create_dir_all(&layout.c)?;
        std::fs::create_dir_all(&layout.obj)?;
        std::fs::create_dir_all(&layout.bin)?;
        std::fs::create_dir_all(&layout.lib)?;
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
    compile_and_link_impl(toolchain, request, None)
}

/// Link a driver-owned executable whose externally callable C definitions are
/// known. Linux GCC with a capable linker can remove unreachable sections;
/// each retained symbol is required to have a definition. Other linkers retain
/// the ordinary complete program. Ordinary host linking and archives do not
/// opt into this policy. Names must be the emitted C symbols, not source names.
pub fn compile_and_link_executable_with_roots(
    toolchain: &Toolchain,
    request: &LinkRequest,
    retained_symbols: &[String],
) -> Result<(), BuildError> {
    compile_and_link_impl(toolchain, request, Some(retained_symbols))
}

fn executable_section_flags(toolchain: &Toolchain, managed_executable: bool) -> &'static [&'static str] {
    if managed_executable && cfg!(target_os = "linux") && matches!(toolchain, Toolchain::Gcc(_)) {
        &["-ffunction-sections", "-fdata-sections"]
    } else {
        &[]
    }
}

/// Whether GCC's selected Linux linker supports both section removal and
/// required definition roots. GCC may use a linker other than GNU ld; an
/// unavailable or unrecognized capability preserves ordinary linking.
pub fn executable_section_gc_supported(toolchain: &Toolchain) -> bool {
    if executable_section_flags(toolchain, true).is_empty() { return false; }
    let Ok(selected) = compiler_command(toolchain).arg("-print-prog-name=ld").output() else { return false; };
    if !selected.status.success() { return false; }
    let Ok(selected) = std::str::from_utf8(&selected.stdout) else { return false; };
    let selected = selected.trim();
    if selected.is_empty() || selected.contains('\n') { return false; }
    let Ok(help) = Command::new(selected).arg("--help").output() else { return false; };
    if !help.status.success() { return false; }
    let help = String::from_utf8_lossy(&help.stdout);
    help.contains("--gc-sections") && help.contains("--require-defined")
}

fn compile_and_link_impl(
    toolchain: &Toolchain,
    request: &LinkRequest,
    retained_symbols: Option<&[String]>,
) -> Result<(), BuildError> {
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
            command.args(gnu_flags(request.profile)).args(gcc_flags(toolchain, request.profile));
            let section_flags = executable_section_flags(toolchain, retained_symbols.is_some());
            command.args(section_flags);
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
            if retained_symbols.is_some() && executable_section_gc_supported(toolchain) {
                command.arg("-Xlinker").arg("--gc-sections");
                for symbol in retained_symbols.unwrap_or_default() {
                    command.arg("-Xlinker").arg(format!("--require-defined={symbol}"));
                }
            }
        }
    }
    run(command)
}

/// `[CG-C-11]` — compile a relaxed floating-point unit: the profile's flags
/// with strict IEEE arithmetic replaced by `@fp(contract)`'s fused
/// multiply-add (`-ffp-contract=fast`; MSVC `/fp:contract`) or, with `fast`,
/// by every relaxation `@fastmath` allows (`-ffast-math`; MSVC `/fp:fast`).
/// The object is linked with the program's.
pub fn compile_relaxed_object(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    output: &Path,
    profile: Profile,
    fast: bool,
) -> Result<(), BuildError> {
    compile_relaxed_object_impl(toolchain, source, include_dirs, output, profile, fast, false)
}

/// A relaxed-FP unit for a managed executable. Sections permit the same
/// reachability policy as its ordinary units without relaxing the FP boundary.
pub fn compile_relaxed_object_for_executable(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    output: &Path,
    profile: Profile,
    fast: bool,
) -> Result<(), BuildError> {
    compile_relaxed_object_impl(toolchain, source, include_dirs, output, profile, fast, true)
}

fn compile_relaxed_object_impl(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    output: &Path,
    profile: Profile,
    fast: bool,
    managed_executable: bool,
) -> Result<(), BuildError> {
    if let Some(parent) = output.parent() { std::fs::create_dir_all(parent)?; }
    let mut command = compiler_command(toolchain);
    match toolchain {
        Toolchain::Msvc { cl, .. } => {
            if is_clang_cl(cl) {
                command.args(relaxed_clang_cl_flags(profile, fast));
            } else {
                command.args(relaxed_msvc_flags(profile, fast));
            }
            command.arg("/c");
            for dir in include_dirs { command.arg(format!("/I{}", dir.display())); }
            command.arg(source).arg(format!("/Fo{}", output.display()));
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.args(relaxed_gnu_flags(profile, fast)).args(gcc_flags(toolchain, profile))
                .args(executable_section_flags(toolchain, managed_executable)).arg("-c");
            for dir in include_dirs { command.arg("-I").arg(dir); }
            command.arg(source).arg("-o").arg(output);
        }
    }
    run(command)
}

/// `[CG-C-11]` — the profile's MSVC flags with `/fp:precise` relaxed:
/// `/fp:contract` added, or `/fp:fast` in its place. `/Z7` keeps the debug
/// information in the object, as no program database is named for it. No
/// `/GL`: link-time code generation could inline across the two modes, so
/// the relaxed object is finished code the linker only places.
fn relaxed_msvc_flags(profile: Profile, fast: bool) -> Vec<&'static str> {
    msvc_flags(profile)
        .into_iter()
        .flat_map(|flag| match flag {
            "/fp:precise" if fast => vec!["/fp:fast"],
            "/fp:precise" => vec!["/fp:precise", "/fp:contract"],
            "/Zi" => vec!["/Z7"],
            "/GL" => vec![],
            other => vec![other],
        })
        .collect()
}

/// Whether an MSVC-style compiler is clang's driver for it, which reads
/// `/fp:fast` as all of `-ffast-math`.
fn is_clang_cl(cl: &Path) -> bool {
    cl.file_stem().is_some_and(|stem| stem.eq_ignore_ascii_case("clang-cl"))
}

/// `[CG-C-11]` — clang-cl's relaxed flags: MSVC's for the profile with
/// clang's relaxations passed through `/clang:`, as `relaxed_gnu_flags`
/// gives them, and never `/fp:fast`, which is clang's `-ffast-math` with the
/// finite-only assumption that makes a NaN argument undefined behaviour.
fn relaxed_clang_cl_flags(profile: Profile, fast: bool) -> Vec<&'static str> {
    msvc_flags(profile)
        .into_iter()
        .flat_map(|flag| match flag {
            "/fp:precise" if fast => vec![
                "/fp:precise",
                "/clang:-ffp-contract=fast",
                "/clang:-funsafe-math-optimizations",
                "/clang:-fno-math-errno",
            ],
            "/fp:precise" => vec!["/fp:precise", "/clang:-ffp-contract=fast"],
            "/Zi" => vec!["/Z7"],
            "/GL" => vec![],
            other => vec![other],
        })
        .collect()
}

/// `[CG-C-11]` — the profile's clang and gcc flags with contraction on
/// (`-ffp-contract=fast`) and, for `@fastmath`, every relaxation of
/// `-ffast-math` but one: reassociation, reciprocals, signed zeros, traps and
/// `errno` (`-funsafe-math-optimizations -fno-math-errno`), not the
/// assumption that no value is NaN or infinite (`-ffinite-math-only`). Under
/// it clang makes a NaN argument undefined behaviour, and a `@fastmath`
/// function called with one could then skip a bounds check, which a Safe
/// program never may (`[PHIL-10]`, ODR-090).
fn relaxed_gnu_flags(profile: Profile, fast: bool) -> Vec<&'static str> {
    gnu_flags(profile)
        .into_iter()
        .flat_map(|flag| match flag {
            "-ffp-contract=off" => vec!["-ffp-contract=fast"],
            "-fno-fast-math" if fast => vec!["-funsafe-math-optimizations", "-fno-math-errno"],
            other => vec![other],
        })
        .collect()
}

/// Compile one C translation unit for a distributable library. MSVC keeps
/// debug data inside the object: an archive must not depend on a build-local
/// PDB beside it (`[FFI-28]`). Profile optimisation and safety flags match
/// ordinary executable compilation, including shipping's `/GL` on MSVC.
pub fn compile_object(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    output: &Path,
    profile: Profile,
) -> Result<(), BuildError> {
    if let Some(parent) = output.parent() { std::fs::create_dir_all(parent)?; }
    let mut command = compiler_command(toolchain);
    match toolchain {
        Toolchain::Msvc { .. } => {
            command.args(msvc_flags(profile).into_iter().map(|flag|
                if flag == "/Zi" { "/Z7" } else { flag }));
            command.arg("/c");
            for dir in include_dirs { command.arg(format!("/I{}", dir.display())); }
            command.arg(source).arg(format!("/Fo{}", output.display()));
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.args(gnu_flags(profile)).args(gcc_flags(toolchain, profile)).arg("-c");
            for dir in include_dirs { command.arg("-I").arg(dir); }
            command.arg(source).arg("-o").arg(output);
        }
    }
    if let Err(error) = run(command) {
        let _ = std::fs::remove_file(output);
        return Err(error);
    }
    Ok(())
}

/// The archive extension follows the C ABI toolchain, not just the host OS.
pub fn archive_name(toolchain: &Toolchain, stem: &str) -> String {
    match toolchain {
        Toolchain::Msvc { .. } => format!("{stem}.lib"),
        Toolchain::Clang(_) | Toolchain::Gcc(_) => format!("lib{stem}.a"),
    }
}

/// Publish a fresh archive. The temporary archive starts empty, so removed
/// objects can never survive a rebuild; a failed command leaves no final
/// archive that a host could mistake for this build's output.
pub fn archive_objects(
    toolchain: &Toolchain,
    objects: &[PathBuf],
    output: &Path,
) -> Result<(), BuildError> {
    if let Some(parent) = output.parent() { std::fs::create_dir_all(parent)?; }
    if output.is_file() { std::fs::remove_file(output)?; }
    let file_name = output.file_name().and_then(|name| name.to_str()).unwrap_or("archive");
    let temp = output.with_file_name(format!("{file_name}.tmp-{}", std::process::id()));
    if temp.is_file() { std::fs::remove_file(&temp)?; }
    let mut command = match toolchain {
        Toolchain::Msvc { env, .. } => {
            let mut command = Command::new("lib.exe");
            command.envs(env);
            command.arg("/nologo").arg("/BREPRO").arg(format!("/OUT:{}", temp.display()));
            command
        }
        Toolchain::Clang(path) => {
            let beside_clang = path.parent().map(|dir| dir.join(if cfg!(windows) { "llvm-ar.exe" } else { "llvm-ar" }));
            let archiver = beside_clang.filter(|path| path.is_file()).unwrap_or_else(|| {
                if tool_on_path("llvm-ar") { PathBuf::from("llvm-ar") } else { PathBuf::from("ar") }
            });
            let mut command = Command::new(archiver);
            command.arg("crsD").arg(&temp);
            command
        }
        Toolchain::Gcc(_) => {
            let mut command = Command::new("ar");
            command.arg("crsD").arg(&temp);
            command
        }
    };
    for object in objects { command.arg(object); }
    let result = run(command).map_err(|error| match error {
        BuildError::CompilerFailed { command, output } => BuildError::ArchiverFailed { command, output },
        other => other,
    });
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temp);
        return Err(error);
    }
    std::fs::rename(&temp, output).map_err(|error| {
        let _ = std::fs::remove_file(&temp);
        BuildError::Io(error)
    })
}

fn tool_on_path(name: &str) -> bool {
    Command::new(name).arg("--version").output().is_ok_and(|output| output.status.success())
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
/// `/fp:precise` with the source's `#pragma fp_contract(off)` is strict IEEE
/// arithmetic (`[CG-C-11]`).
fn msvc_flags(profile: Profile) -> Vec<&'static str> {
    let optimisation: &[&str] = match profile {
        Profile::Debug => &["/Od", "/Zi", "/MD", "/DEMBER_RT_DEBUG_ACCESS"],
        Profile::Release => &["/O2", "/MD"],
        Profile::Shipping => &["/O2", "/GL", "/MD"],
    };
    ["/nologo", "/std:c11", "/W3", "/fp:precise"].iter().chain(optimisation).copied().collect()
}

/// clang's and gcc's flags for a profile, shared by the program's compile and
/// the runtime object's so the two always agree. `-ffp-contract=off` and
/// `-fno-fast-math` are strict IEEE arithmetic (`[CG-C-11]`, D-325): no
/// `a * b + c` fused where the target has the instruction.
fn gnu_flags(profile: Profile) -> Vec<&'static str> {
    let optimisation: &[&str] = match profile {
        Profile::Debug => &["-O0", "-g", "-DEMBER_RT_DEBUG_ACCESS"],
        Profile::Release => &["-O2"],
        Profile::Shipping => &["-O3"],
    };
    ["-std=c11", "-Wall", "-Wextra", "-ffp-contract=off", "-fno-fast-math"].iter().chain(optimisation).copied().collect()
}

/// ADR-098 — gcc's own flags for a profile, after `gnu_flags`. At `-O2` gcc's
/// vectoriser takes only a loop whose vector code leaves no tail, so a loop
/// over a list's elements (its count known only at run time) stays scalar,
/// where clang's and MSVC's `-O2` vectorise it with the tail as a scalar
/// loop: `-fvect-cost-model=cheap` asks gcc for the same (measured: walking a
/// list with `enumerate`, 1.45x the hand-written C built with the same flags,
/// and 0.99x with it). `-O3` already uses a model at least as permissive.
/// ADR-099 — and `-funroll-loops`: gcc unrolls no loop at `-O2` (clang does),
/// and a loop whose turn adds to an object's field with its overflow check,
/// the `jo` on each add waiting for the last add's value, runs at half speed
/// rolled (`mut self` calls: 1.98x the C++ built alike, 0.98x unrolled).
fn gcc_flags(toolchain: &Toolchain, profile: Profile) -> &'static [&'static str] {
    match (toolchain, profile) {
        (Toolchain::Gcc(_), Profile::Release) => &["-fvect-cost-model=cheap", "-funroll-loops"],
        _ => &[],
    }
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
/// shapes it: the compile command, MSVC's `INCLUDE` and `LIB`, the source,
/// the headers in `include_dirs` and the compiler's file on disk. It is compiled
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
    runtime_object_impl(toolchain, source, include_dirs, profile, cache, false)
}

/// Cache the runtime for a managed executable. Section flags are part of the
/// hashed compile command, so an ordinary cached object cannot mask this policy.
pub fn runtime_object_for_executable(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    profile: Profile,
    cache: &Path,
) -> Result<Option<PathBuf>, BuildError> {
    runtime_object_impl(toolchain, source, include_dirs, profile, cache, true)
}

fn runtime_object_impl(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    profile: Profile,
    cache: &Path,
    managed_executable: bool,
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
            command.args(gnu_flags(profile)).args(gcc_flags(toolchain, profile))
                .args(executable_section_flags(toolchain, managed_executable)).arg("-c");
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
    // MSVC reads its headers and libraries from these: another Windows SDK is
    // another object. From the toolchain's variables, else this process's
    // (a developer environment used as it is).
    if let Toolchain::Msvc { env, .. } = toolchain {
        for name in ["INCLUDE", "LIB"] {
            let value = env
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.clone())
                .or_else(|| std::env::var(name).ok())
                .unwrap_or_default();
            part(value.as_bytes());
        }
    }
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

    /// ADR-098 — gcc's release build asks for the vectoriser cost model that
    /// takes a loop with a scalar tail; clang (which does by default), the
    /// other profiles and MSVC get nothing extra.
    #[test]
    fn gcc_release_asks_for_the_cheap_vectoriser_model_and_unrolling() {
        let gcc = Toolchain::Gcc(PathBuf::from("gcc"));
        let clang = Toolchain::Clang(PathBuf::from("clang"));
        assert_eq!(gcc_flags(&gcc, Profile::Release), ["-fvect-cost-model=cheap", "-funroll-loops"]);
        assert!(gcc_flags(&gcc, Profile::Debug).is_empty());
        assert!(gcc_flags(&gcc, Profile::Shipping).is_empty());
        assert!(gcc_flags(&clang, Profile::Release).is_empty());
    }

    #[test]
    fn profile_names_round_trip() {
        for profile in [Profile::Debug, Profile::Release, Profile::Shipping] {
            assert_eq!(Profile::from_name(profile.name()), Some(profile));
        }
        assert_eq!(Profile::from_name("fast"), None);
    }

    /// Managed section removal preserves native initialization/finalization,
    /// indirect hooks, allocation accounting and panic dispatch. Host links
    /// keep their ordinary symbols; cache entries distinguish both policies.
    #[cfg(target_os = "linux")]
    #[test]
    fn gcc_managed_linking_preserves_runtime_roots_and_host_linking() {
        use std::os::unix::process::ExitStatusExt;
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        if !executable_section_gc_supported(&toolchain) { return; }
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("managed-runtime-roots-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).unwrap();
        let source = dir.join("host.c");
        let fixture = r#"
#include "@HEADER@"
#include <stdlib.h>
#include <string.h>
static int started, allocations, frees;
__attribute__((constructor)) static void startup(void) { started = 1; }
__attribute__((destructor)) static void finish(void) { puts("fini"); }
static void* host_alloc(size_t size, size_t align) {
    (void)align; ++allocations; return malloc(size);
}
static void host_free(void* p, size_t size, size_t align) {
    (void)size; (void)align; ++frees; free(p);
}
static void host_panic(const char* message, size_t length) {
    (void)message; (void)length; fputs("host panic hook\n", stderr);
}
__attribute__((noinline)) int retained_c_root(void) { return 43; }
__attribute__((noinline)) int dead_helper(void) { return 99; }
static int callback(int value) { return value + 1; }
static int (*volatile live_callback)(int) = callback;
int main(int argc, char** argv) {
    if (!started || argc != 2) return 1;
    @RT@rt_config cfg = @RT@rt_config_default();
    cfg.alloc = host_alloc; cfg.free = host_free; cfg.on_panic = host_panic;
    if (@RT@rt_init(strcmp(argv[1], "default") == 0 ? NULL : &cfg)) return 2;
    @RT@alloc_stats stats;
    @RT@debug_alloc_stats(&stats);
    if (stats.live_bytes || stats.total_allocations || stats.total_frees) return 3;
    void* p = @RT@alloc(7, 1);
    @RT@debug_alloc_stats(&stats);
    if (!p || stats.live_bytes != 7 || stats.total_allocations != 1) return 4;
    @RT@free(p, 7, 1);
    @RT@debug_alloc_stats(&stats);
    if (stats.live_bytes || stats.total_allocations != 1 || stats.total_frees != 1) return 5;
    if (strcmp(argv[1], "default") != 0 && (allocations != 1 || frees != 1)) return 6;
    if (strcmp(argv[1], "panic") == 0)
        @RT@panic("managed panic", 13, @RT@loc_at("host", 1, 1));
    @RT@rt_shutdown();
    if (@RT@rt_init(NULL)) return 7;
    @RT@debug_alloc_stats(&stats);
    if (stats.live_bytes || stats.total_allocations != 1 || stats.total_frees != 1) return 8;
    p = @RT@alloc(3, 1);
    @RT@free(p, 3, 1);
    @RT@debug_alloc_stats(&stats);
    if (stats.live_bytes || stats.total_allocations != 2 || stats.total_frees != 2) return 9;
    if (strcmp(argv[1], "default") != 0 && (allocations != 1 || frees != 1)) return 10;
    @RT@rt_shutdown();
    printf("%d\n", live_callback(41));
    return 0;
}
"#.replace("@HEADER@", &ember_branding::runtime_header())
            .replace("@RT@", ember_branding::RUNTIME_PREFIX);
        std::fs::write(&source, fixture).unwrap();
        let includes = [runtime.join("include")];
        let runtime_source = runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX));
        let roots = ["retained_c_root".to_string()];
        for profile in [Profile::Debug, Profile::Release, Profile::Shipping] {
            let ordinary = runtime_object(&toolchain, &runtime_source, &includes, profile, &dir.join("cache"))
                .unwrap().expect("ordinary runtime object");
            let managed = runtime_object_for_executable(&toolchain, &runtime_source, &includes, profile, &dir.join("cache"))
                .unwrap().expect("managed runtime object");
            assert_ne!(ordinary, managed, "section policy did not enter the cache key");
            for (uses_gc, object) in [(false, ordinary), (true, managed)] {
                let executable = dir.join(format!("{}-{uses_gc}", profile.name()));
                let request = LinkRequest {
                    sources: &[source.clone(), object], include_dirs: &includes,
                    output: executable.clone(), profile, obj_dir: dir.join("obj"),
                };
                if uses_gc {
                    compile_and_link_executable_with_roots(&toolchain, &request, &roots).unwrap();
                } else {
                    compile_and_link(&toolchain, &request).unwrap();
                }
                for mode in ["default", "configured", "panic"] {
                    let output = Command::new(&executable).arg(mode).output().unwrap();
                    if mode == "panic" {
                        assert_eq!(output.status.signal(), Some(6));
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        assert!(stderr.contains("managed panic") && stderr.contains("host panic hook"), "{stderr}");
                    } else {
                        assert!(output.status.success(), "{} {mode}: {:?} {}", executable.display(), output.status,
                            String::from_utf8_lossy(&output.stderr));
                        assert_eq!(output.stdout, b"42\nfini\n");
                        assert!(output.stderr.is_empty());
                    }
                }
                let symbols = Command::new("nm").arg("--defined-only").arg(&executable).output().unwrap();
                assert!(symbols.status.success());
                let symbols = String::from_utf8_lossy(&symbols.stdout);
                let names: Vec<_> = symbols.lines().filter_map(|line| line.split_whitespace().last()).collect();
                assert!(names.contains(&"retained_c_root"));
                assert_eq!(names.contains(&"dead_helper"), !uses_gc);
                let elf = Command::new("readelf").args(["-lW"]).arg(&executable).output().unwrap();
                assert!(elf.status.success());
                let headers = String::from_utf8_lossy(&elf.stdout);
                assert!(headers.contains("GNU_RELRO"));
                let stack = headers.lines().find(|line| line.contains("GNU_STACK")).expect("stack permissions");
                assert!(stack.contains("RW ") && !stack.contains("RWE"), "{stack}");
                if uses_gc {
                    assert!(compile_and_link_executable_with_roots(&toolchain, &request,
                        &["missing_definition".to_string()]).is_err(), "missing C root silently linked");
                }
            }
        }
    }

    /// A GCC front end alone does not imply GNU ld. Missing linker capability
    /// must keep the ordinary complete image rather than attempt unsafe GC.
    #[cfg(target_os = "linux")]
    #[test]
    fn gcc_without_required_root_support_keeps_ordinary_linking() {
        use std::os::unix::fs::PermissionsExt;
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        if !matches!(toolchain, Toolchain::Gcc(_)) { return; }
        let compiler = compiler_file(&toolchain).expect("gcc path");
        let dir = std::env::temp_dir().join(format!("managed-linker-fallback-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).unwrap();
        let linker = dir.join("linker");
        std::fs::write(&linker, "#!/bin/sh\nprintf '%s\\n' '--gc-sections'\n").unwrap();
        let wrapper = dir.join("gcc-wrapper");
        let shell_quote = |path: &Path| format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"));
        std::fs::write(&wrapper, format!(
            "#!/bin/sh\nif [ \"$1\" = '-print-prog-name=ld' ]; then printf '%s\\n' {}; exit 0; fi\nexec {} \"$@\"\n",
            shell_quote(&linker), shell_quote(&compiler))).unwrap();
        for script in [&linker, &wrapper] {
            std::fs::set_permissions(script, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let simulated = Toolchain::Gcc(wrapper);
        assert!(!executable_section_gc_supported(&simulated));
        let source = dir.join("host.c");
        std::fs::write(&source,
            "int retained_c_root(void) { return 42; }\nint dead_helper(void) { return 99; }\nint main(void) { return 0; }\n").unwrap();
        let executable = dir.join("host");
        compile_and_link_executable_with_roots(&simulated, &LinkRequest {
            sources: &[source], include_dirs: &[], output: executable.clone(),
            profile: Profile::Release, obj_dir: dir.join("obj"),
        }, &["retained_c_root".to_string()]).expect("ordinary fallback links");
        assert!(Command::new(&executable).status().unwrap().success());
        let listed = Command::new("nm").arg("--defined-only").arg(executable).output().unwrap();
        assert!(listed.status.success());
        let listing = String::from_utf8_lossy(&listed.stdout);
        let names: Vec<_> = listing.lines().filter_map(|line| line.split_whitespace().last()).collect();
        assert!(names.contains(&"retained_c_root") && names.contains(&"dead_helper"));
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

    #[test]
    fn rebuilding_an_archive_drops_removed_objects() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let dir = std::env::temp_dir().join(format!("fresh-archive-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let extension = if matches!(&toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
        let a_source = dir.join("a.c");
        let b_source = dir.join("b.c");
        let host_source = dir.join("host.c");
        std::fs::write(&a_source, "int first(void) { return 1; }\n").unwrap();
        std::fs::write(&b_source, "int removed(void) { return 2; }\n").unwrap();
        std::fs::write(&host_source, "int removed(void); int main(void) { return removed() == 2 ? 0 : 1; }\n").unwrap();
        let a = dir.join(format!("a.{extension}"));
        let b = dir.join(format!("b.{extension}"));
        compile_object(&toolchain, &a_source, &[], &a, Profile::Debug).unwrap();
        compile_object(&toolchain, &b_source, &[], &b, Profile::Debug).unwrap();
        let archive = dir.join(archive_name(&toolchain, "fresh"));
        archive_objects(&toolchain, &[a.clone(), b], &archive).unwrap();
        let first_exe = dir.join(if cfg!(windows) { "before.exe" } else { "before" });
        let first_objects = dir.join("before-obj");
        std::fs::create_dir_all(&first_objects).unwrap();
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[host_source.clone(), archive.clone()], include_dirs: &[],
            output: first_exe.clone(), profile: Profile::Debug, obj_dir: first_objects,
        }).expect("the first archive includes both objects");
        assert!(Command::new(first_exe).status().unwrap().success());

        archive_objects(&toolchain, &[a], &archive).unwrap();
        let second_exe = dir.join(if cfg!(windows) { "after.exe" } else { "after" });
        let second_objects = dir.join("after-obj");
        std::fs::create_dir_all(&second_objects).unwrap();
        assert!(compile_and_link(&toolchain, &LinkRequest {
            sources: &[host_source, archive], include_dirs: &[],
            output: second_exe, profile: Profile::Debug, obj_dir: second_objects,
        }).is_err(), "a removed archive member still linked after rebuild");
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

    /// Byte reductions must widen signed values correctly and never read
    /// beyond a short or unaligned view, even beside an inaccessible page.
    #[test]
    fn byte_sums_agree_with_a_scalar_oracle_at_guard_pages() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("byte-sums-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "byte_sums.exe" } else { "byte_sums" });
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[runtime.join("tests/byte_sums.c")],
            include_dirs: &[runtime.join("include")], output: program.clone(),
            profile: Profile::Release, obj_dir: dir.join("obj"),
        }).expect("the guarded byte-sum oracle compiles");
        let output = Command::new(program).output().expect("the guarded oracle runs");
        assert!(output.status.success(), "guarded oracle failed: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `[TXT-2, TXT-10]` — both cursor types decode every Unicode scalar,
    /// advance exactly once, and stop reading at the end of a guarded view.
    #[test]
    fn text_char_steps_agree_with_a_scalar_oracle_at_guard_pages() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("text-char-steps-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "text_char_steps.exe" } else { "text_char_steps" });
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[runtime.join("tests/text_char_steps.c")],
            include_dirs: &[runtime.join("include")], output: program.clone(),
            profile: Profile::Release, obj_dir: dir.join("obj"),
        }).expect("the guarded scalar-decoder oracle compiles");
        let output = Command::new(program).output().expect("the guarded oracle runs");
        assert!(output.status.success(), "guarded decoder oracle failed: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Over-aligned size rounding must reject overflow before the default
    /// allocator call while preserving zero-size and custom-hook forwarding.
    #[test]
    fn allocator_rounding_rejects_overflow_and_preserves_forwarding() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("allocator-rounding-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "allocator_rounding.exe" } else { "allocator_rounding" });
        let fixture = runtime.join("tests/allocator_rounding.c");
        // The fixture includes the generated runtime after NULL-only allocator
        // interception. A separately compiled runtime would bypass the oracle.
        if cfg!(windows) && matches!(toolchain, Toolchain::Clang(_)) {
            // D-377: the included runtime's native int128 helpers need clang's
            // compiler runtime on Windows, just as the int128 oracle below.
            let mut command = compiler_command(&toolchain);
            command.args(gnu_flags(Profile::Release))
                .arg("-I").arg(runtime.join("include"))
                .arg(&fixture).arg("-o").arg(&program)
                .arg("-rtlib=compiler-rt");
            run(command).expect("the allocator-rounding oracle compiles");
        } else {
            compile_and_link(&toolchain, &LinkRequest {
                sources: &[fixture], include_dirs: &[runtime.join("include")],
                output: program.clone(), profile: Profile::Release, obj_dir: dir.join("obj"),
            }).expect("the allocator-rounding oracle compiles");
        }
        for mode in ["default-controls", "custom", "overflow-default"] {
            let mut command = Command::new(&program);
            command.arg(mode);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                command.creation_flags(0x08000000); // CREATE_NO_WINDOW
            }
            let output = command.output().expect("the allocator-rounding oracle runs");
            assert!(output.status.success(), "{mode} failed: {}", String::from_utf8_lossy(&output.stderr));
            assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
            assert!(output.stderr.is_empty(), "{mode}: {}", String::from_utf8_lossy(&output.stderr));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `[RT-11]` — even sequential host-thread operations must not share
    /// allocation counters. Cross-thread frees belong to the freeing thread.
    #[test]
    fn allocation_statistics_are_independent_per_thread() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("thread-alloc-stats-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "thread_alloc_stats.exe" } else { "thread_alloc_stats" });
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[runtime.join("tests/thread_alloc_stats.c"), runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))],
            include_dirs: &[runtime.join("include")], output: program.clone(),
            profile: Profile::Release, obj_dir: dir.join("obj"),
        }).expect("the allocation-statistics oracle compiles");
        let output = Command::new(program).output().expect("the allocation-statistics oracle runs");
        assert!(output.status.success(), "thread-local statistics failed: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// D-272 — the runtime's 128-bit integers as two 64-bit halves (MSVC's
    /// form) agree bit for bit with the C compiler's `__int128`: the test
    /// program, rendered from `templates/tests/int128_halves.c.in`, compares
    /// every helper over edge values and two million random ones. MSVC has
    /// no `__int128` to compare with, so there it does not run.
    #[test]
    fn the_128_bit_halves_agree_with_int128() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let (Toolchain::Clang(cc) | Toolchain::Gcc(cc)) = &toolchain else { return };
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime")
            .join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("int128-halves-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the test directory is creatable");
        let program = dir.join("int128_halves");
        let mut command = Command::new(cc);
        command
            .args(gnu_flags(Profile::Release))
            .arg(format!("-D{}_SOFT_INT128", ember_branding::SYMBOL_PREFIX.to_uppercase()))
            .arg("-I")
            .arg(runtime.join("include"))
            .arg(runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX)))
            .arg(runtime.join("tests").join("int128_halves.c"))
            .arg("-o")
            .arg(&program);
        if !cfg!(windows) {
            command.arg("-lm");
        }
        // clang for Windows compiles `__int128` division and conversions to
        // calls (`__divti3` and others) into its own runtime library, which
        // it links only when asked (D-377).
        if cfg!(windows) && matches!(toolchain, Toolchain::Clang(_)) {
            command.arg("-rtlib=compiler-rt");
        }
        run(command).expect("the test program compiles");
        let output = Command::new(&program).output().expect("the test program runs");
        let _ = std::fs::remove_dir_all(&dir);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success() && stdout.trim() == "ok", "{stdout}");
    }

    /// `[CG-C-11]` (D-325) — with the build's flags, `a * b + c` stays two
    /// roundings even where the target has a fused multiply-add: the C,
    /// compiled for a processor with FMA, has no FMA instruction. A control
    /// compile that allows contraction shows the check can see one.
    #[test]
    fn the_c_compiler_never_fuses_a_multiply_and_an_add() {
        if !cfg!(target_arch = "x86_64") {
            return;
        }
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let (Toolchain::Clang(cc) | Toolchain::Gcc(cc)) = &toolchain else { return };
        let dir = std::env::temp_dir().join(format!("fp-contract-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the test directory is creatable");
        let source = dir.join("fused.c");
        std::fs::write(&source, "double fused(double a, double b, double c) { return a * b + c; }\n")
            .expect("the source is writable");
        let assembly = |extra: &[&str]| {
            let output = dir.join("fused.s");
            let mut command = Command::new(cc);
            command.args(gnu_flags(Profile::Shipping)).args(extra).arg("-mfma").arg("-S").arg(&source).arg("-o").arg(&output);
            run(command).expect("the source compiles");
            std::fs::read_to_string(&output).expect("the assembly is readable")
        };
        let control = assembly(&["-ffp-contract=fast"]);
        let strict = assembly(&[]);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(control.contains("vfmadd"), "the control compile did not fuse, so the check sees nothing:\n{control}");
        assert!(!strict.contains("vfmadd"), "the build's flags let the C compiler fuse:\n{strict}");
    }

    /// `[CG-C-11]`, `[TYP-9b]` — a relaxed unit's flags let the C compiler
    /// fuse, for `@fp(contract)` and `@fastmath` alike, and change nothing
    /// else a strict unit is compiled with.
    #[test]
    fn a_relaxed_unit_is_compiled_to_fuse() {
        for fast in [false, true] {
            let gnu = relaxed_gnu_flags(Profile::Shipping, fast);
            assert!(gnu.contains(&"-ffp-contract=fast") && !gnu.contains(&"-ffp-contract=off"), "{gnu:?}");
            assert_eq!(gnu.contains(&"-funsafe-math-optimizations"), fast, "{gnu:?}");
            assert_eq!(gnu.contains(&"-fno-fast-math"), !fast, "{gnu:?}");
            // Never the finite-only assumption: a NaN must stay a value.
            assert!(!gnu.iter().any(|flag| *flag == "-ffast-math" || flag.contains("finite-math")), "{gnu:?}");
            assert_eq!(gnu.len(), gnu_flags(Profile::Shipping).len() + usize::from(fast), "{gnu:?}");
            let msvc = relaxed_msvc_flags(Profile::Shipping, fast);
            assert_eq!(msvc.contains(&"/fp:fast"), fast, "{msvc:?}");
            assert_eq!(msvc.contains(&"/fp:contract"), !fast, "{msvc:?}");
            assert_eq!(msvc.contains(&"/fp:precise"), !fast, "{msvc:?}");
            assert!(!msvc.contains(&"/GL"), "{msvc:?}");
            let clang_cl = relaxed_clang_cl_flags(Profile::Shipping, fast);
            assert!(!clang_cl.contains(&"/fp:fast") && clang_cl.contains(&"/clang:-ffp-contract=fast"), "{clang_cl:?}");
            assert_eq!(clang_cl.contains(&"/clang:-funsafe-math-optimizations"), fast, "{clang_cl:?}");
        }
        if !cfg!(target_arch = "x86_64") {
            return;
        }
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let (Toolchain::Clang(cc) | Toolchain::Gcc(cc)) = &toolchain else { return };
        let dir = std::env::temp_dir().join(format!("fp-relaxed-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("the test directory is creatable");
        let source = dir.join("fused.c");
        std::fs::write(&source, "double fused(double a, double b, double c) { return a * b + c; }\n")
            .expect("the source is writable");
        let output = dir.join("fused.s");
        let mut command = Command::new(cc);
        command.args(relaxed_gnu_flags(Profile::Shipping, false)).arg("-mfma").arg("-S").arg(&source).arg("-o").arg(&output);
        run(command).expect("the source compiles");
        let assembly = std::fs::read_to_string(&output).expect("the assembly is readable");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(assembly.contains("vfmadd"), "a relaxed unit's flags did not let the C compiler fuse:\n{assembly}");
    }

    #[test]
    fn a_toolchain_is_found_on_this_machine() {
        // Phase 0's exit criterion needs one; if this fails, the environment
        // is the problem, not the compiler.
        let found = Toolchain::detect(None);
        assert!(found.is_ok(), "no C toolchain: {found:?}");
    }
}
