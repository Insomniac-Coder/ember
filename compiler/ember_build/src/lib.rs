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

pub mod cdylib;
pub mod interface;
pub mod reload;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Profile {
    #[default]
    Debug,
    /// Optimised and still debuggable: line information, symbols, thin LTO (`[PRF-3]`).
    Development,
    /// The build users get: optimisation 3, stripped, full LTO (`[PRF-3]`, ODR-117).
    Release,
}

impl Profile {
    pub fn name(self) -> &'static str {
        match self {
            Profile::Debug => "debug",
            Profile::Development => "development",
            Profile::Release => "release",
        }
    }

    pub fn from_name(name: &str) -> Option<Profile> {
        Some(match name {
            "debug" => Profile::Debug,
            "development" => Profile::Development,
            "release" => Profile::Release,
            _ => return None,
        })
    }

    /// `[PRF-3]` — the profile's link-time optimisation, unless the manifest's
    /// `[profiles.<name>] lto` gives another ([`set_lto`]).
    pub fn lto(self) -> Lto {
        match self {
            Profile::Debug => Lto::Off,
            Profile::Development => Lto::Thin,
            Profile::Release => Lto::On,
        }
    }
}

/// `[BLD-6]` — link-time optimisation: none, ThinLTO's (each unit optimised with summaries of the
/// others, in parallel), or the whole program's as one. No correctness property depends on it: it
/// changes only how far the C compiler inlines and optimises across translation units.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Lto {
    Off,
    Thin,
    On,
}

impl Lto {
    /// In order of how much the link optimises: [`Toolchain::lto`]'s "nearest" is along it.
    pub const ALL: [Lto; 3] = [Lto::Off, Lto::Thin, Lto::On];

    /// The manifest's word for it.
    pub fn name(self) -> &'static str {
        match self {
            Lto::Off => "off",
            Lto::Thin => "thin",
            Lto::On => "on",
        }
    }

    pub fn from_name(name: &str) -> Option<Lto> {
        Lto::ALL.into_iter().find(|lto| lto.name() == name)
    }

    fn rank(self) -> isize {
        Lto::ALL.iter().position(|lto| *lto == self).unwrap_or(0) as isize
    }
}

/// `[BLD-6]` — who links a build's objects: this build (a program, a shared library), or a link it
/// does not run (a static library's, `--emit obj`'s), which needs objects the toolchain's own
/// linker reads whatever it is asked.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Link {
    Here,
    Elsewhere,
}

/// `[BLD-6]` — the manifest's `lto` for this build's profile; `None` is the profile's own.
static LTO: std::sync::Mutex<Option<Lto>> = std::sync::Mutex::new(None);

/// `[BLD-6]` — set the link-time optimisation this build asks for: the manifest's
/// `[profiles.<name>] lto`, or `None` for the profile's own ([`Profile::lto`]).
pub fn set_lto(lto: Option<Lto>) {
    if let Ok(mut set) = LTO.lock() {
        *set = lto;
    }
}

/// `[BLD-6]` — the link-time optimisation a build of `profile` asks for.
pub fn asked_lto(profile: Profile) -> Lto {
    LTO.lock().ok().and_then(|set| *set).unwrap_or(profile.lto())
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

/// `[TCB-5]` — a content hash, for an evidence record's identities.
pub fn content_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

impl Toolchain {
    pub fn name(&self) -> &'static str {
        match self {
            Toolchain::Msvc { .. } => "msvc",
            Toolchain::Clang(_) => "clang",
            Toolchain::Gcc(_) => "gcc",
        }
    }

    /// `[TCB-5]` — the C compiler's identity: its kind, where it is, and its executable's hash
    /// (a bare name is looked up on `PATH`).
    pub fn identity(&self) -> String {
        let path = match self {
            Toolchain::Msvc { cl, .. } => cl.clone(),
            Toolchain::Clang(path) | Toolchain::Gcc(path) => path.clone(),
        };
        let found = if path.is_file() {
            Some(path.clone())
        } else {
            std::env::var_os("PATH").and_then(|paths| {
                std::env::split_paths(&paths).map(|dir| dir.join(&path)).find(|candidate| candidate.is_file())
            })
        };
        let hash = found.as_ref().and_then(|file| std::fs::read(file).ok()).map(|bytes| content_hash(&bytes));
        format!("{} {} {}", self.name(), found.unwrap_or(path).display(), hash.unwrap_or_else(|| "unhashed".to_string()))
    }

    /// `[TOOL-4]` — the C compiler's file, as found (a bare name looked up on `PATH`).
    pub fn compiler_path(&self) -> PathBuf {
        let path = match self {
            Toolchain::Msvc { cl, .. } => cl.clone(),
            Toolchain::Clang(path) | Toolchain::Gcc(path) => path.clone(),
        };
        compiler_file(self).unwrap_or(path)
    }

    /// `[TOOL-4]` — the linker the C compiler runs: MSVC's `link.exe` beside `cl.exe`; for GCC and
    /// Clang, the one the driver names.
    pub fn linker(&self) -> String {
        match self {
            Toolchain::Msvc { .. } => self.compiler_path().parent().map(|dir| dir.join("link.exe"))
                .filter(|path| path.is_file())
                .map_or_else(|| "link.exe".to_string(), |path| path.display().to_string()),
            Toolchain::Clang(_) | Toolchain::Gcc(_) => {
                let named = output_started(compiler_command(self).arg("-print-prog-name=ld")).ok()
                    .filter(|output| output.status.success())
                    .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
                    .filter(|name| !name.is_empty());
                named.unwrap_or_else(|| "ld".to_string())
            }
        }
    }

    /// `[BLD-6]` — whether this toolchain does `lto` for objects linked `link`. An object another
    /// link reads must serve that link as it is. cl does the whole program's (`/GL`, `/LTCG`) at
    /// its own link; a `/GL` object holds no code, binds an archive to that compiler's version and
    /// turns its reader's link into link-time code generation, so not for a link elsewhere. gcc
    /// does the whole program's (its partitions are not ThinLTO's summaries); its fat objects (code
    /// beside the intermediate form) serve any link, optimised across only by one that asks. Clang
    /// does both kinds, linked by LLVM's `lld` (every clang-cl link already is); its objects hold
    /// only LLVM's intermediate form, so not for a link elsewhere either.
    pub fn supports_lto(&self, lto: Lto, link: Link) -> bool {
        match (self, lto, link) {
            (_, Lto::Off, _) => true,
            (Toolchain::Msvc { cl, .. }, lto, link) if !is_clang_cl(cl) => lto == Lto::On && link == Link::Here,
            (Toolchain::Gcc(_), lto, _) => lto == Lto::On,
            (Toolchain::Msvc { .. } | Toolchain::Clang(_), _, Link::Elsewhere) => false,
            (Toolchain::Msvc { .. }, _, Link::Here) => true,
            (Toolchain::Clang(_), _, Link::Here) => cfg!(target_os = "macos") || self.lld().is_some(),
        }
    }

    /// `[BLD-6]` — `asked`, or the nearest kind this toolchain does when it does not; of two as
    /// near, the one that optimises more (`thin` with cl or gcc is `on`): it is what `thin` is for,
    /// and nothing a program means depends on it.
    pub fn lto(&self, asked: Lto, link: Link) -> Lto {
        Lto::ALL
            .into_iter()
            .filter(|lto| self.supports_lto(*lto, link))
            .min_by_key(|lto| ((lto.rank() - asked.rank()).abs(), -lto.rank()))
            .unwrap_or(Lto::Off)
    }

    /// LLVM's linker beside Clang or on `PATH`: `lld-link` on Windows, `ld.lld` elsewhere.
    fn lld(&self) -> Option<PathBuf> {
        let name = if cfg!(windows) { "lld-link.exe" } else { "ld.lld" };
        let beside = self.compiler_path().parent().map(|dir| dir.join(name)).filter(|path| path.is_file());
        beside.or_else(|| program_file(Path::new(name), std::env::var_os("PATH")))
    }

    /// Find a usable C compiler, preferring `requested` when it is given.
    /// `[DET-7]` — the C compiler's own statement of its version, the first line it prints, which
    /// names it on any machine where its path does not: `cl` prints it unasked (on standard error),
    /// gcc and clang for `--version`.
    pub fn version(&self) -> String {
        let mut command = compiler_command(self);
        if !matches!(self, Toolchain::Msvc { cl, .. } if !is_clang_cl(cl)) {
            command.arg("--version");
        }
        let Ok(output) = command.output() else { return self.name().to_string() };
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or(self.name()).to_string()
    }

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
    /// front end. It ignores `/GL`: its link-time optimisation is clang's
    /// (`[BLD-6]`, [`lto_compile_flags`]).
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
    write_atomically(&file, &text);
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
    /// `[TYP-9c]` — the C compiler ignored or refused a flag that relaxes floating point for one
    /// unit (`/fp:contract` on an MSVC that predates it): `@fastmath` (`fast`) or `@fp(contract)`
    /// cannot be honoured as written, and an attribute is never ignored (`E9041`).
    FloatModeRefused {
        toolchain: String,
        flag: String,
        fast: bool,
    },
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // `[TOOL-3]` — `E9002` says no C compiler was found, and its help names the host's remedy;
            // a bare missing path is never the message.
            BuildError::NoToolchain(which) => {
                let looked = match *which {
                    "any" => "MSVC, clang or gcc".to_string(),
                    one => format!("`{one}`"),
                };
                write!(f, "error[E9002]: no C compiler was found (looked for {looked})
  = help: {}", toolchain_remedy())
            }
            BuildError::UnknownToolchain(name) => write!(f, "unknown C compiler `{name}`"),
            BuildError::Io(e) => write!(f, "{e}"),
            BuildError::CompilerFailed { command, output } => {
                write!(f, "the C compiler failed\n  command: {command}\n{output}")
            }
            BuildError::ArchiverFailed { command, output } => {
                write!(f, "the archive tool failed\n  command: {command}\n{output}")
            }
            BuildError::FloatModeRefused { toolchain, flag, fast } => {
                let attribute = if *fast { "@fastmath" } else { "@fp(contract)" };
                write!(f, "`{toolchain}` does not take `{flag}`, which `{attribute}` needs")
            }
        }
    }
}

/// `[TOOL-3]` — what installs a C compiler on this host.
pub fn toolchain_remedy() -> &'static str {
    if cfg!(windows) {
        "install the Visual Studio Build Tools with the \"Desktop development with C++\" workload (it provides MSVC), or clang"
    } else if cfg!(target_os = "macos") {
        "install the Xcode Command Line Tools (`xcode-select --install`), which provide clang"
    } else {
        "install gcc or clang with the system's package manager (on Debian or Ubuntu, `sudo apt install build-essential`)"
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
/// Ninja-driven incremental build arrives with the module system. A program
/// linked before from the same inputs is copied from the global cache instead
/// (ADR-136, [`CachedProgram`]).
pub fn compile_and_link(toolchain: &Toolchain, request: &LinkRequest) -> Result<(), BuildError> {
    compile_and_link_impl(toolchain, request, None, &[], program_cache().as_deref(), env_layout())
}

/// D-755, `[BEN-1]` — how many code layouts `ember bench` builds each side at.
pub const LAYOUTS: usize = 5;

/// D-755, `[BEN-1]` — as [`compile_and_link`], with the program's code at layout `layout` (below
/// [`LAYOUTS`]): `0` is the toolchain's own; each other moves every function of the program to
/// another address and changes nothing else. A small loop's speed depends on where it lies in
/// the instruction cache's lines, so the same C runs up to twice as fast at one address as at
/// another; `ember bench` times an Ember program and its C twin each at every layout and compares
/// their fastest.
pub fn compile_and_link_at_layout(toolchain: &Toolchain, request: &LinkRequest, layout: usize) -> Result<(), BuildError> {
    compile_and_link_impl(toolchain, request, None, &[], program_cache().as_deref(), layout)
}

/// The layout a build of `ember bench`'s asks for (`ember_branding::bench_layout_var`); `0` otherwise.
fn env_layout() -> usize {
    std::env::var(ember_branding::bench_layout_var()).ok().and_then(|value| value.parse().ok()).filter(|layout| *layout < LAYOUTS).unwrap_or(0)
}

/// D-755 — the flags that put a program's code at `layout`: MSVC's linker pads before each function
/// (`/FUNCTIONPADMIN`, which pads to the next 16 bytes), so a function's address moves by a multiple
/// of 16 with the functions before it; gcc and clang align every loop to 16, 32, 64 or 128 bytes
/// (`-falign-loops`), padding run once before the loop is entered. clang-cl's linker has no such
/// option: one layout.
fn layout_flags(toolchain: &Toolchain, layout: usize) -> (Option<String>, Option<String>) {
    if layout == 0 {
        return (None, None);
    }
    match toolchain {
        Toolchain::Msvc { cl, .. } if !is_clang_cl(cl) => (None, Some(format!("/FUNCTIONPADMIN:{}", 16 * (layout - 1)))),
        Toolchain::Msvc { .. } => (None, None),
        Toolchain::Clang(_) | Toolchain::Gcc(_) => (Some(format!("-falign-loops={}", 8 << layout)), None),
    }
}

/// Link a driver-owned executable whose externally callable C definitions are
/// known. Linux GCC with a capable linker can remove unreachable sections;
/// each retained symbol is required to have a definition. Other linkers retain
/// the ordinary program, and every linker keeps the retained symbols, which
/// link-time optimisation would otherwise drop when nothing calls them. Ordinary host linking and archives do not
/// opt into this policy. Names must be the emitted C symbols, not source names.
pub fn compile_and_link_executable_with_roots(
    toolchain: &Toolchain,
    request: &LinkRequest,
    retained_symbols: &[String],
) -> Result<(), BuildError> {
    compile_and_link_impl(toolchain, request, Some(retained_symbols), &[], program_cache().as_deref(), env_layout())
}

/// `[BLD-FFI-3]` — as [`compile_and_link_executable_with_roots`], linking `libraries` after the
/// program's own objects, in order: each a name the linker searches for (`m`, `vulkan-1`) or a
/// path to a library file.
pub fn compile_and_link_program(
    toolchain: &Toolchain,
    request: &LinkRequest,
    retained_symbols: &[String],
    libraries: &[String],
) -> Result<(), BuildError> {
    // ponytail: a program linking libraries is not kept in the program cache, whose key would
    // need each library's contents; key them by their files if the relink cost shows.
    let cache = if libraries.is_empty() { program_cache() } else { None };
    compile_and_link_impl(toolchain, request, Some(retained_symbols), libraries, cache.as_deref(), env_layout())
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
///
/// Asking takes two processes (GCC names its linker, the linker lists its
/// options), on every build: the answer is kept in the global cache, keyed by
/// the compiler file and what steers GCC's choice of linker, and it holds
/// while the linker GCC named is still the same file at the same place.
pub fn executable_section_gc_supported(toolchain: &Toolchain) -> bool {
    if executable_section_flags(toolchain, true).is_empty() { return false; }
    let cache = linker_probe_file(toolchain);
    if let Some(answer) = cache.as_deref().and_then(read_linker_probe) {
        return answer;
    }
    let (named, answer) = probe_linker(toolchain);
    // D-510 — only an answer the probe finished is kept: one whose program
    // could not be started says nothing about the linker.
    if let (Some(file), Some(named), Some(answer)) = (cache, named, answer)
        && let Some(found) = program_file(Path::new(&named), std::env::var_os("PATH"))
        && let Some(stamp) = file_stamp(&found)
    {
        let text = format!("{}\n{named}\n{}\n{stamp}\n", if answer { "yes" } else { "no" }, found.display());
        write_atomically(&file, &text);
    }
    answer.unwrap_or(false)
}

/// GCC's linker, as GCC names it, and whether it has both options; `None`
/// for the answer when a program could not be started.
fn probe_linker(toolchain: &Toolchain) -> (Option<String>, Option<bool>) {
    let Ok(selected) = output_started(compiler_command(toolchain).arg("-print-prog-name=ld")) else { return (None, None) };
    if !selected.status.success() { return (None, Some(false)); }
    let Ok(selected) = std::str::from_utf8(&selected.stdout) else { return (None, Some(false)) };
    let selected = selected.trim();
    if selected.is_empty() || selected.contains('\n') { return (None, Some(false)); }
    let named = Some(selected.to_string());
    let Ok(help) = output_started(Command::new(selected).arg("--help")) else { return (named, None) };
    if !help.status.success() { return (named, Some(false)); }
    let help = String::from_utf8_lossy(&help.stdout);
    (named, Some(help.contains("--gc-sections") && help.contains("--require-defined")))
}

/// D-510 — a command's output, tried again when it could not be started at
/// all: on Linux a file just written cannot be executed ("text file busy")
/// while another thread's new child still holds it open, for a moment. The
/// tests start the programs they link through it too: the program cache
/// copies one into place while other test threads start processes.
pub fn output_started(command: &mut Command) -> std::io::Result<std::process::Output> {
    let mut tries = 0;
    loop {
        match command.output() {
            Err(_) if tries < 4 => {
                tries += 1;
                std::thread::sleep(std::time::Duration::from_millis(5 << tries));
            }
            result => return result,
        }
    }
}

/// Where the probe's answer for this compiler is kept: keyed by the compiler
/// file (path, size, modification time) and the variables GCC searches for
/// its linker by. `None` when the compiler file cannot be found.
fn linker_probe_file(toolchain: &Toolchain) -> Option<PathBuf> {
    let compiler = compiler_file(toolchain)?;
    let mut key = blake3::Hasher::new();
    key.update(b"linker-gc v1\0");
    key.update(compiler.to_string_lossy().as_bytes());
    key.update(file_stamp(&compiler)?.as_bytes());
    for name in ["COMPILER_PATH", "GCC_EXEC_PREFIX"] {
        key.update(b"\0");
        key.update(name.as_bytes());
        key.update(b"=");
        if let Some(value) = std::env::var_os(name) {
            key.update(value.to_string_lossy().as_bytes());
        }
    }
    Some(cache_root().join("linker").join(format!("{}.txt", key.finalize().to_hex())))
}

/// A cached answer, if the linker GCC named is still found at the same file,
/// unchanged: a bare name is looked up on this process's `PATH` again.
fn read_linker_probe(file: &Path) -> Option<bool> {
    let text = std::fs::read_to_string(file).ok()?;
    let mut lines = text.lines();
    let answer = match lines.next()? {
        "yes" => true,
        "no" => false,
        _ => return None,
    };
    let (named, found, stamp) = (lines.next()?, lines.next()?, lines.next()?);
    let now = program_file(Path::new(named), std::env::var_os("PATH"))?;
    (now == Path::new(found) && file_stamp(&now)? == stamp).then_some(answer)
}

/// A file's size and modification time, as text.
fn file_stamp(file: &Path) -> Option<String> {
    let meta = file.metadata().ok()?;
    Some(format!("{} {}", meta.len(), modified_nanos(&meta)))
}

/// Write a cache file whole or not at all: another build may be reading it.
fn write_atomically(file: &Path, text: &str) {
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
        let temp = file.with_extension(format!("tmp{}", std::process::id()));
        if std::fs::write(&temp, text).is_ok() && std::fs::rename(&temp, file).is_err() {
            let _ = std::fs::remove_file(&temp);
        }
    }
}

/// `[TST-27]` — whether the program's own C is compiled with warnings as errors.
fn warnings_as_errors() -> bool {
    std::env::var(ember_branding::c_warnings_as_errors_var()).is_ok_and(|value| value == "1")
}

fn compile_and_link_impl(
    toolchain: &Toolchain,
    request: &LinkRequest,
    retained_symbols: Option<&[String]>,
    libraries: &[String],
    cache: Option<&Path>,
    layout: usize,
) -> Result<(), BuildError> {
    let mut command = compiler_command(toolchain);
    let (layout_compile, layout_link) = layout_flags(toolchain, layout);
    let lto = build_lto(toolchain, request.profile, Link::Here);
    match toolchain {
        Toolchain::Msvc { cl, .. } => {
            let clang_cl = is_clang_cl(cl);
            command.args(msvc_flags(request.profile)).args(lto_compile_flags(toolchain, lto, Link::Here));
            if warnings_as_errors() {
                command.arg("/WX");
            }
            // `[BLD-13]` — clang-cl's debug information names folders as gcc's does, and its objects
            // carry no time stamp: the linker's `/Brepro` hashes them into the program.
            if clang_cl {
                command.arg("/Brepro");
                // Only where there is debug information: a `release` build has none, and clang-cl
                // warns of a flag it does not use (an error under the C gate, `[TST-27]`).
                if request.profile != Profile::Release {
                    command.arg("/clang:-gno-codeview-command-line").arg("/clang:-fdebug-compilation-dir=.");
                }
                // The object's own folder too: CodeView names the object file.
                let sources: Vec<PathBuf> = request.sources.iter().cloned().chain([request.obj_dir.join("object")]).collect();
                command.args(folder_name_flags(&sources, request.include_dirs).into_iter().map(|flag| format!("/clang:{flag}")));
            }
            for dir in request.include_dirs {
                command.arg(format!("/I{}", dir.display()));
            }
            for source in request.sources {
                command.arg(source);
            }
            for library in libraries {
                command.arg(cdylib::windows_library(library));
            }
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.args(gnu_flags(request.profile)).args(gcc_flags(toolchain, request.profile));
            match inherited_gnu_lto() {
                // `[BLD-FFI-4]` — the C++ project's link-time optimisation, for the program's C and
                // its link; an empty one is the project's `-fno-lto`.
                Some(inherited) => {
                    if !inherited.is_empty() {
                        command.arg(inherited);
                        // Clang on Windows links link-time-optimised code only with LLVM's `lld-link`.
                        if cfg!(windows) && matches!(toolchain, Toolchain::Clang(_)) {
                            command.arg("-fuse-ld=lld");
                        }
                    }
                }
                // `[BLD-6]` — compiled and linked in one: the link's flags hold the compile's.
                None => {
                    command.args(lto_link_flags(toolchain, lto));
                }
            }
            command.args(layout_compile.iter());
            if warnings_as_errors() {
                command.arg("-Werror");
            }
            folder_names(&mut command, request.sources, request.include_dirs);
            let section_flags = executable_section_flags(toolchain, retained_symbols.is_some());
            command.args(section_flags);
            for dir in request.include_dirs {
                command.arg("-I").arg(dir);
            }
            for source in request.sources {
                command.arg(source);
            }
            for library in libraries {
                command.arg(cdylib::gnu_library(library));
            }
            if !cfg!(windows) {
                // `[THR-10]` — the runtime starts threads with pthreads.
                command.arg("-lm").arg("-pthread");
            }
            // The roots stay though nothing in the program calls them: section removal and
            // link-time optimisation each drop what is unreferenced.
            let section_gc = retained_symbols.is_some() && executable_section_gc_supported(toolchain);
            if section_gc {
                command.arg("-Xlinker").arg("--gc-sections");
            }
            for symbol in retained_symbols.unwrap_or_default() {
                match (section_gc, cfg!(windows)) {
                    (true, _) => command.arg("-Xlinker").arg(format!("--require-defined={symbol}")),
                    (false, true) => command.arg("-Xlinker").arg(format!("/INCLUDE:{symbol}")),
                    (false, false) => command.args(["-Xlinker", "-u", "-Xlinker", symbol]),
                };
            }
        }
    }
    // Everything but where the outputs go: the cache's key.
    let cached = cache.and_then(|dir| CachedProgram::find(dir, toolchain, &command, request));
    if let Some(cached) = &cached {
        if cached.serve(&request.output) {
            return Ok(());
        }
        // The files the sources include, wherever they are found.
        command.arg(if matches!(toolchain, Toolchain::Msvc { .. }) { "/showIncludes" } else { "-H" });
    }
    match toolchain {
        Toolchain::Msvc { cl, .. } => {
            // Object files land in obj/, keeping the C directory readable.
            command.arg(format!("/Fo{}\\", request.obj_dir.display()));
            command.arg(format!("/Fe{}", request.output.display()));
            if request.profile != Profile::Release {
                command.arg(format!("/Fd{}\\", request.obj_dir.display()));
            }
            // `[BLD-13]` — the same inputs link to the same bytes: no time stamp (`/Brepro`), no
            // incremental link (whose output depends on the one before it; `[BLD-FFI-3]` too: it
            // keeps what it resolved before, so libraries listed in another order would not change
            // which one defines a symbol), and the program database named without its folder.
            // `[BLD-13]` — clang-cl links with LLVM's `lld-link`: Microsoft's linker writes each
            // relative source path of a clang-cl object into the program database as an absolute
            // one, from its own working folder, and hashes that into `/Brepro`'s time stamp;
            // `lld-link` writes them relative to a stated folder (`/pdbsourcepath`).
            if is_clang_cl(cl) {
                command.arg("-fuse-ld=lld");
            }
            command.args(["/link", "/Brepro", "/INCREMENTAL:NO"]).args(lto_link_flags(toolchain, lto));
            // The roots stay though nothing in the program calls them (`/OPT:REF`, `/LTCG`).
            command.args(retained_symbols.unwrap_or_default().iter().map(|symbol| format!("/INCLUDE:{symbol}")));
            if request.profile != Profile::Release {
                command.arg("/PDBALTPATH:%_PDB%");
                if is_clang_cl(cl) {
                    command.arg("/pdbsourcepath:.");
                }
            }
            // `[RT-4]` — symbols for a development build's backtrace, and its link as it was: `/Zi`
            // makes `cl` link with `/DEBUG`, which would turn these off.
            if request.profile == Profile::Development {
                command.args(["/OPT:REF", "/OPT:ICF"]);
            }
            // `[CG-C-9]` — the program's visualisers in its program database, where the debugger
            // finds them. After the cache's key: the file is the C's, and its place is the build's.
            if let Some(natvis) = natvis_beside(request) {
                command.arg(format!("/NATVIS:{}", natvis.display()));
            }
            command.args(layout_link.iter());
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.arg("-o").arg(&request.output);
            // `[BLD-13]` — clang on Windows links with Microsoft's linker: as for MSVC.
            if cfg!(windows) {
                command.args(["-Wl,/Brepro", "-Wl,/INCREMENTAL:NO"]);
                if request.profile != Profile::Release {
                    command.arg("-Wl,/PDBALTPATH:%_PDB%");
                }
            }
            // `[RT-4]` — as for MSVC: `-g` links with `/DEBUG`, which turns these off.
            if cfg!(windows) && request.profile == Profile::Development {
                command.args(["-Wl,/OPT:REF", "-Wl,/OPT:ICF"]);
            }
            // `[CG-C-9]` — as for MSVC.
            if cfg!(windows) && let Some(natvis) = natvis_beside(request) {
                command.arg(format!("-Wl,/NATVIS:{}", natvis.display()));
            }
        }
    }
    let started = std::time::SystemTime::now();
    let compiled = run_output(command)?;
    if let Some(cached) = cached {
        cached.store(request, &compiled, started);
    }
    Ok(())
}

/// `[CG-C-9]` — the visualiser file the build wrote beside the program (`<program>.natvis`), for a
/// build with a program database (every profile but `release`).
fn natvis_beside(request: &LinkRequest) -> Option<PathBuf> {
    let natvis = request.output.with_extension("natvis");
    (request.profile != Profile::Release && natvis.is_file()).then_some(natvis)
}

/// The global cache's linked programs, unless the variable
/// `no_compile_cache_var()` names is set.
fn program_cache() -> Option<PathBuf> {
    let off = std::env::var_os(ember_branding::no_compile_cache_var()).is_some_and(|value| !value.is_empty());
    (!off).then(|| cache_root().join("programs"))
}

/// What the C compilers and linkers read from the environment besides their
/// arguments: MSVC's search paths and extra options, then GCC's and clang's.
const PROGRAM_VARIABLES: [&str; 12] = [
    "INCLUDE", "LIB", "LIBPATH", "CL", "_CL_", "LINK", "_LINK_",
    "CPATH", "C_INCLUDE_PATH", "LIBRARY_PATH", "COMPILER_PATH", "GCC_EXEC_PREFIX",
];

/// ADR-136 — a program linked before from the same inputs, kept in the global
/// cache to be copied instead of built again.
///
/// Its key is the compile command without its outputs, each source in it by
/// what it holds (the same program built in another folder is the same
/// executable), the variables the compiler and linker read, what each include
/// directory lists, and the compiler's file. The files the sources include,
/// found wherever the compiler finds them (beside the including file, in an
/// include directory or in the system's), only the compile can name
/// (`/showIncludes`, `-H`): the manifest `<key>.txt` names the program those
/// files built and lists each file with its hash, and a build is served only
/// while every one of them still has it. The ceiling: the system's libraries
/// are known by the compiler's file and `LIB` alone, and a header newly put
/// ahead of an included one, beside the including file, is not seen.
struct CachedProgram {
    dir: PathBuf,
    key: String,
}

impl CachedProgram {
    /// `None` when a source cannot be read: the compiler reports it.
    fn find(dir: &Path, toolchain: &Toolchain, command: &Command, request: &LinkRequest) -> Option<CachedProgram> {
        let mut key = Key::new();
        // v2: a program is kept with its PDB (`[RT-4]`); one kept before has none.
        key.part(b"program v2");
        key.part(command.get_program().as_encoded_bytes());
        for arg in command.get_args() {
            match request.sources.iter().find(|source| source.as_os_str() == arg) {
                Some(source) => {
                    key.part(source.extension().unwrap_or_default().as_encoded_bytes());
                    key.part(&std::fs::read(source).ok()?);
                }
                None => key.part(arg.as_encoded_bytes()),
            }
        }
        for name in PROGRAM_VARIABLES {
            key.part(variable(command, name).unwrap_or_default().as_encoded_bytes());
        }
        // `[CG-C-9]` — the visualisers the link puts in the program database, by their content.
        if let Some(natvis) = natvis_beside(request) {
            key.part(b"natvis");
            key.part(&std::fs::read(natvis).ok()?);
        }
        for include in request.include_dirs {
            let mut names: Vec<_> = std::fs::read_dir(include).ok()?
                .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
                .collect();
            names.sort();
            for name in names {
                key.part(name.as_encoded_bytes());
            }
        }
        key.compiler_file(toolchain);
        Some(CachedProgram { dir: dir.to_path_buf(), key: key.finish() })
    }

    fn manifest(&self) -> PathBuf {
        self.dir.join(format!("{}.txt", self.key))
    }

    /// Copy the kept program to `output`, if every file it was built from
    /// still holds what it held.
    fn serve(&self, output: &Path) -> bool {
        let Ok(text) = std::fs::read_to_string(self.manifest()) else { return false };
        let mut lines = text.lines();
        let Some(program) = lines.next().map(|name| self.dir.join(name)) else { return false };
        for line in lines {
            let Some((hash, file)) = line.split_once(' ') else { return false };
            if !std::fs::read(file).is_ok_and(|bytes| blake3::hash(&bytes).to_hex().as_str() == hash) {
                return false;
            }
        }
        if std::fs::copy(&program, output).is_err() {
            return false;
        }
        // `[RT-4]` — its symbols, under the name the program looks for them by (`dbghelp` looks
        // beside the program for the PDB file it was linked with).
        let symbols = program.with_extension("pdb");
        if let (true, Ok(name), Some(dir)) =
            (symbols.is_file(), std::fs::read_to_string(program.with_extension("pdbname")), output.parent())
        {
            let _ = std::fs::copy(&symbols, dir.join(name.trim()));
        }
        // Used now: the trim removes the least recently used first.
        for file in [program.with_extension("pdb"), program.with_extension("pdbname"), program, self.manifest()] {
            if let Ok(open) = std::fs::File::options().write(true).open(file) {
                let _ = open.set_modified(std::time::SystemTime::now());
            }
        }
        true
    }

    /// Keep the program just built, with the files its compile named; not when
    /// it named none (notes this does not read), or when one of them changed
    /// since `started`, or just before it (a file's time can be coarse): what
    /// would be kept might not be what was built.
    fn store(&self, request: &LinkRequest, compiled: &std::process::Output, started: std::time::SystemTime) {
        let mut files = Vec::new();
        for stream in [&compiled.stdout, &compiled.stderr] {
            for line in String::from_utf8_lossy(stream).lines() {
                if let Some(file) = included_file(line) && let Ok(file) = std::path::absolute(file) {
                    files.push(file);
                }
            }
        }
        files.sort();
        files.dedup();
        let compiles_c = request.sources.iter().any(|source| source.extension().is_some_and(|ext| ext == "c"));
        if files.is_empty() && compiles_c {
            return;
        }
        let settled = started.checked_sub(std::time::Duration::from_secs(2)).unwrap_or(started);
        let mut manifest = String::new();
        let mut built = Key::new();
        built.part(self.key.as_bytes());
        for file in &files {
            let Ok(meta) = file.metadata() else { return };
            if meta.modified().map_or(true, |time| time > settled) {
                return;
            }
            let Ok(bytes) = std::fs::read(file) else { return };
            let hash = blake3::hash(&bytes).to_hex();
            built.part(hash.as_bytes());
            manifest.push_str(&format!("{hash} {}\n", file.display()));
        }
        let extension = if cfg!(windows) { "exe" } else { "bin" };
        let name = format!("{}-{}.{extension}", self.key, &built.finish()[..16]);
        let program = self.dir.join(&name);
        if std::fs::create_dir_all(&self.dir).is_err() {
            return;
        }
        // `[RT-4]` — a program's PDB, kept beside it with the file name it was linked with, before
        // the program itself: `serve` gives it back under that name.
        let symbols = request.output.with_extension("pdb");
        if !program.is_file()
            && symbols.is_file()
            && let Some(file_name) = symbols.file_name()
            && std::fs::copy(&symbols, program.with_extension("pdb")).is_ok()
        {
            write_atomically(&program.with_extension("pdbname"), &file_name.to_string_lossy());
        }
        if !program.is_file() {
            // Copied in under a temporary name: another build sees the whole
            // program or none.
            let temp = self.dir.join(format!("{name}.tmp{}", std::process::id()));
            if std::fs::copy(&request.output, &temp).is_err() || std::fs::rename(&temp, &program).is_err() {
                let _ = std::fs::remove_file(&temp);
                if !program.is_file() {
                    return;
                }
            }
        }
        write_atomically(&self.manifest(), &format!("{name}\n{manifest}"));
        trim_programs(&self.dir, PROGRAM_CACHE_BYTES);
    }
}

/// A variable as `command` will see it: set on it, else inherited.
fn variable(command: &Command, name: &str) -> Option<std::ffi::OsString> {
    match command.get_envs().find(|(set, _)| set.eq_ignore_ascii_case(name)) {
        Some((_, value)) => value.map(std::ffi::OsStr::to_os_string),
        None => std::env::var_os(name),
    }
}

/// The file an include note names: `-H`'s dots, a space and the path; or the
/// note `/showIncludes` writes in the compiler's own language ("Note:
/// including file:" in English), whose path is absolute and follows a colon.
/// `None` for the compiler's other output: a diagnostic starts with its file.
fn included_file(line: &str) -> Option<&str> {
    let undotted = line.trim_start_matches('.');
    if undotted.len() < line.len() {
        return undotted.strip_prefix(' ').filter(|path| !path.is_empty());
    }
    let mut at = 0;
    while let Some(found) = line[at..].find(": ") {
        let rest = line[at + found + 1..].trim_start_matches(' ').trim_end();
        let bytes = rest.as_bytes();
        let drive = bytes.len() > 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'\\';
        if drive || rest.starts_with("\\\\") {
            return Some(rest);
        }
        at += found + 2;
    }
    None
}

/// The program cache's size, kept by `trim_programs`. ponytail: fixed; a
/// setting when a machine needs another.
const PROGRAM_CACHE_BYTES: u64 = 2 << 30;

/// Keep the program cache under `limit` bytes (trimmed to three quarters of
/// it), the least recently used first; at most once an hour, as listing the
/// folder on every build would cost more than the cache saves.
fn trim_programs(dir: &Path, limit: u64) {
    let stamp = dir.join("trimmed");
    let recent = stamp.metadata().and_then(|meta| meta.modified())
        .is_ok_and(|time| time.elapsed().is_ok_and(|age| age.as_secs() < 3600));
    if recent || std::fs::write(&stamp, b"").is_err() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut files = Vec::new();
    for entry in entries.flatten() {
        if let Ok(meta) = entry.metadata() && meta.is_file() && let Ok(time) = meta.modified() {
            files.push((time, meta.len(), entry.path()));
        }
    }
    let mut total: u64 = files.iter().map(|(_, len, _)| len).sum();
    if total <= limit {
        return;
    }
    files.sort();
    for (_, len, file) in files {
        if total <= limit / 4 * 3 {
            break;
        }
        if file != stamp && std::fs::remove_file(&file).is_ok() {
            total -= len;
        }
    }
}

/// A cache key: each part hashed with its length, so no two lists of parts
/// hash the same bytes.
struct Key(blake3::Hasher);

impl Key {
    fn new() -> Key {
        Key(blake3::Hasher::new())
    }

    fn part(&mut self, bytes: &[u8]) {
        self.0.update(&(bytes.len() as u64).to_le_bytes());
        self.0.update(bytes);
    }

    /// An upgraded compiler, or another one first on PATH, is a new key.
    fn compiler_file(&mut self, toolchain: &Toolchain) {
        if let Some(file) = compiler_file(toolchain) {
            self.part(file.to_string_lossy().as_bytes());
            if let Ok(meta) = file.metadata() {
                self.part(&meta.len().to_le_bytes());
                self.part(&modified_nanos(&meta).to_le_bytes());
            }
        }
    }

    fn finish(&self) -> String {
        self.0.finalize().to_hex().to_string()
    }
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
    compile_relaxed_object_impl(toolchain, source, include_dirs, output, profile, fast, false, false)
}

/// A relaxed-FP unit of a shared library: position-independent off Windows.
pub fn compile_relaxed_library_object(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    output: &Path,
    profile: Profile,
    fast: bool,
) -> Result<(), BuildError> {
    compile_relaxed_object_impl(toolchain, source, include_dirs, output, profile, fast, false, true)
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
    compile_relaxed_object_impl(toolchain, source, include_dirs, output, profile, fast, true, false)
}

fn compile_relaxed_object_impl(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    output: &Path,
    profile: Profile,
    fast: bool,
    managed_executable: bool,
    shared_library: bool,
) -> Result<(), BuildError> {
    if let Some(parent) = output.parent() { std::fs::create_dir_all(parent)?; }
    let mut command = compiler_command(toolchain);
    // `[TYP-9c]` — the flags that relax this unit, beyond the profile's own.
    let (relaxed, strict): (Vec<&str>, Vec<&str>) = match toolchain {
        Toolchain::Msvc { cl, .. } if is_clang_cl(cl) => (relaxed_clang_cl_flags(profile, fast), msvc_flags(profile)),
        Toolchain::Msvc { .. } => (relaxed_msvc_flags(profile, fast), msvc_flags(profile)),
        Toolchain::Clang(_) | Toolchain::Gcc(_) => (relaxed_gnu_flags(profile, fast), gnu_flags(profile)),
    };
    let relaxing: Vec<&str> = relaxed.iter().copied().filter(|flag| !strict.contains(flag)).collect();
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
            if shared_library && !cfg!(windows) {
                command.arg("-fPIC");
            }
            folder_names(&mut command, &[source.to_path_buf()], include_dirs);
            for dir in include_dirs { command.arg("-I").arg(dir); }
            command.arg(source).arg("-o").arg(output);
        }
    }
    let refused = |text: &str| {
        refused_relaxation(text, &relaxing).map(|flag| BuildError::FloatModeRefused {
            toolchain: toolchain_name(toolchain),
            flag: flag.to_string(),
            fast,
        })
    };
    match run_output(command) {
        Ok(done) => {
            let text = format!("{}{}", String::from_utf8_lossy(&done.stdout), String::from_utf8_lossy(&done.stderr));
            refused(&text).map_or(Ok(()), Err)
        }
        Err(BuildError::CompilerFailed { command, output }) => {
            Err(refused(&output).unwrap_or(BuildError::CompilerFailed { command, output }))
        }
        Err(other) => Err(other),
    }
}

/// `[TYP-9c]` — which of the flags relaxing a unit's floating point the C compiler's output says
/// it ignored (MSVC's `D9002`, "ignoring unknown option") or refused (clang's "unknown argument",
/// gcc's "unrecognized command-line option"), if any.
fn refused_relaxation<'a>(output: &str, relaxing: &[&'a str]) -> Option<&'a str> {
    const REFUSALS: [&str; 5] = ["D9002", "unknown argument", "unrecognized command-line option", "unknown option", "unsupported option"];
    output.lines().find_map(|line| {
        if !REFUSALS.iter().any(|refusal| line.contains(refusal)) {
            return None;
        }
        relaxing.iter().copied().find(|flag| line.contains(flag))
    })
}

/// The C compiler as a diagnostic or a build record names it.
pub fn toolchain_name(toolchain: &Toolchain) -> String {
    let path = match toolchain {
        Toolchain::Msvc { cl, .. } => cl,
        Toolchain::Clang(path) | Toolchain::Gcc(path) => path,
    };
    path.file_stem().map_or_else(|| path.display().to_string(), |stem| stem.to_string_lossy().into_owned())
}

/// `[CG-C-11]` — the profile's MSVC flags with `/fp:precise` relaxed:
/// `/fp:contract` added, or `/fp:fast` in its place. `/Z7` keeps the debug
/// information in the object, as no program database is named for it. No
/// link-time optimisation (`[BLD-6]`): it could inline across the two modes,
/// so the relaxed object is finished code the linker only places.
fn relaxed_msvc_flags(profile: Profile, fast: bool) -> Vec<&'static str> {
    msvc_flags(profile)
        .into_iter()
        .flat_map(|flag| match flag {
            "/fp:precise" if fast => vec!["/fp:fast"],
            "/fp:precise" => vec!["/fp:precise", "/fp:contract"],
            "/Zi" => vec!["/Z7"],
            other => vec![other],
        })
        .collect()
}

/// Whether an MSVC-style compiler is clang's driver for it, which reads
/// `/fp:fast` as all of `-ffast-math`.
pub(crate) fn is_clang_cl(cl: &Path) -> bool {
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
/// ordinary executable compilation, including release's `/GL` on MSVC.
pub fn compile_object(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    output: &Path,
    profile: Profile,
) -> Result<(), BuildError> {
    compile_object_with_flags(toolchain, source, include_dirs, &[], output, profile)
}

/// [`compile_object`] with flags of the program's own after the build's: `[BLD-FFI-1a]` an
/// `import c`'s shim and C sources, with the package's `[c] flags` and the import's `-D`/`-I`
/// (every toolchain takes those two spellings).
pub fn compile_object_with_flags(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    flags: &[String],
    output: &Path,
    profile: Profile,
) -> Result<(), BuildError> {
    if let Some(parent) = output.parent() { std::fs::create_dir_all(parent)?; }
    let mut command = compiler_command(toolchain);
    // `[BLD-6]` — for whichever link reads the object.
    let lto = lto_compile_flags(toolchain, build_lto(toolchain, profile, Link::Elsewhere), Link::Elsewhere);
    match toolchain {
        Toolchain::Msvc { .. } => {
            command.args(msvc_flags(profile).into_iter().map(|flag|
                if flag == "/Zi" { "/Z7" } else { flag })).args(lto);
            command.arg("/c").args(flags);
            for dir in include_dirs { command.arg(format!("/I{}", dir.display())); }
            command.arg(source).arg(format!("/Fo{}", output.display()));
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.args(gnu_flags(profile)).args(gcc_flags(toolchain, profile)).args(lto).arg("-c").args(flags);
            folder_names(&mut command, &[source.to_path_buf()], include_dirs);
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

/// `[FFI-3]` — an `import cpp`'s thunk file, compiled as C++ by the build's own compiler
/// (`[BLD-FFI-4]`: cl or clang-cl with `/TP /EHsc`, g++ for gcc, clang++ for clang) with the
/// profile's flags but the C standard, the given C++ standard, and the project's flags.
pub fn compile_cpp_object(
    toolchain: &Toolchain,
    source: &Path,
    flags: &[String],
    output: &Path,
    profile: Profile,
    standard: &str,
) -> Result<(), BuildError> {
    if let Some(parent) = output.parent() { std::fs::create_dir_all(parent)?; }
    let mut command = match toolchain {
        Toolchain::Msvc { .. } => compiler_command(toolchain),
        Toolchain::Clang(path) | Toolchain::Gcc(path) => {
            let name = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
            let cpp = if name.contains("clang") { name.replacen("clang", "clang++", 1) } else { name.replacen("gcc", "g++", 1) };
            let mut command = Command::new(path.with_file_name(cpp));
            // `[THR-14]`, as for the C.
            if cfg!(target_arch = "x86_64") { command.arg("-mcx16"); }
            command
        }
    };
    // `[BLD-6]` — for whichever link reads the object, as the program's C.
    let lto = lto_compile_flags(toolchain, build_lto(toolchain, profile, Link::Elsewhere), Link::Elsewhere);
    match toolchain {
        Toolchain::Msvc { .. } => {
            // cl's oldest C++ standard switch is C++14.
            let standard = if standard == "c++11" { "c++14" } else { standard };
            command.args(msvc_flags(profile).into_iter().filter(|flag| *flag != "/std:c11").map(|flag| if flag == "/Zi" { "/Z7" } else { flag }));
            command.args(lto).args(["/TP", "/EHsc", &format!("/std:{standard}")]).arg("/c").args(flags);
            command.arg(source).arg(format!("/Fo{}", output.display()));
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.args(gnu_flags(profile).into_iter().filter(|flag| *flag != "-std=c11" && *flag != "-mcx16")).args(lto).arg(format!("-std={standard}")).arg("-c");
            command.args(inherited_gnu_runtime()).args(flags);
            // D-790 — position-independent, for a reloadable image's shared library too (Annex B):
            // the thunks' `thread_local` record of a caught exception is otherwise reached as an
            // executable's, which a shared object cannot be linked with.
            if !cfg!(windows) { command.arg("-fPIC"); }
            command.arg(source).arg("-o").arg(output);
        }
    }
    if let Err(error) = run(command) {
        let _ = std::fs::remove_file(output);
        return Err(error);
    }
    Ok(())
}

/// `[SIMD-6]` — the host C compiler's own vectorisation remarks on one
/// unit, as corroborating evidence only: the unit is compiled again, with the
/// build's flags, to an object nothing uses, adding the compiler's report
/// flags (clang `-Rpass{,-missed,-analysis}=loop-vectorize`, gcc
/// `-fopt-info-vec-all`, MSVC `/Qvec-report:2`). The remarks kept are those
/// placed in Ember source (the C carries `#line`s, `[CG-C-8]`) or in the unit
/// itself, not the runtime's header, in the order given, each once.
pub fn vectorisation_report(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    scratch: &Path,
    profile: Profile,
) -> Result<Vec<String>, BuildError> {
    std::fs::create_dir_all(scratch)?;
    let object = scratch.join(match toolchain {
        Toolchain::Msvc { .. } => "vectorisation_report.obj",
        _ => "vectorisation_report.o",
    });
    let mut command = compiler_command(toolchain);
    match toolchain {
        Toolchain::Msvc { .. } => {
            command.args(msvc_flags(profile).into_iter().filter(|flag| *flag != "/Zi"));
            command.arg("/Qvec-report:2").arg("/c");
            for dir in include_dirs { command.arg(format!("/I{}", dir.display())); }
            command.arg(source).arg(format!("/Fo{}", object.display()));
        }
        Toolchain::Clang(_) => {
            command.args(gnu_flags(profile)).args(gcc_flags(toolchain, profile));
            command.args(["-Rpass=loop-vectorize", "-Rpass-missed=loop-vectorize", "-Rpass-analysis=loop-vectorize", "-c"]);
            for dir in include_dirs { command.arg("-I").arg(dir); }
            command.arg(source).arg("-o").arg(&object);
        }
        Toolchain::Gcc(_) => {
            command.args(gnu_flags(profile)).args(gcc_flags(toolchain, profile));
            command.args(["-fopt-info-vec-all", "-c"]);
            for dir in include_dirs { command.arg("-I").arg(dir); }
            command.arg(source).arg("-o").arg(&object);
        }
    }
    let output = run_output(command);
    let _ = std::fs::remove_file(&object);
    let output = output?;
    let ext = ember_branding::SOURCE_EXT;
    let unit = source.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let mut seen = std::collections::BTreeSet::new();
    let mut remarks = Vec::new();
    for stream in [&output.stdout, &output.stderr] {
        for line in String::from_utf8_lossy(stream).lines() {
            let line = line.trim();
            // clang places a loop's remark at its function in the C when the
            // loop has no line of its own; MSVC and gcc follow the `#line`s.
            let in_source = line.contains(&format!(".{ext}:")) || line.contains(&format!(".{ext}(")) || line.contains(&unit);
            let remark = ["remark:", "optimized:", "missed:", "C5001", "C5002"].iter().any(|mark| line.contains(mark));
            if in_source && remark && seen.insert(line.to_string()) {
                remarks.push(line.to_string());
            }
        }
    }
    Ok(remarks)
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

/// `[BLD-13]` — gcc's and clang's debug information names each file by its path and the folder the
/// compiler ran in: the working folder and each source's folder are written `.`, and each include
/// folder by its own name, so a program holds no absolute path of the machine that built it.
fn folder_names(command: &mut Command, sources: &[PathBuf], include_dirs: &[PathBuf]) {
    command.args(folder_name_flags(sources, include_dirs));
}

fn folder_name_flags(sources: &[PathBuf], include_dirs: &[PathBuf]) -> Vec<String> {
    let mut folders: Vec<(PathBuf, String)> = Vec::new();
    if let Ok(current) = std::env::current_dir() {
        folders.push((current, ".".to_string()));
    }
    for source in sources {
        if let Some(parent) = source.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            folders.push((parent.to_path_buf(), ".".to_string()));
        }
    }
    for dir in include_dirs {
        let name = dir.file_name().map_or_else(|| ".".to_string(), |name| name.to_string_lossy().into_owned());
        folders.push((dir.clone(), name));
    }
    // Each folder as given and as an absolute path: the compiler records the one or the other.
    let mut flags: Vec<String> = Vec::new();
    for (folder, name) in folders {
        let absolute = std::path::absolute(&folder).unwrap_or_else(|_| folder.clone());
        for spelling in [folder, absolute] {
            let flag = format!("-ffile-prefix-map={}={name}", spelling.display());
            if !flags.contains(&flag) {
                flags.push(flag);
            }
        }
    }
    flags
}

/// The C compiler, with MSVC's environment over the inherited one.
fn compiler_command(toolchain: &Toolchain) -> Command {
    match toolchain {
        Toolchain::Msvc { cl, env } => {
            let mut command = Command::new(cl);
            command.envs(env);
            // `[THR-14]` — clang-cl turns `_InterlockedCompareExchange128` into a call to
            // compiler-rt's `__atomic_compare_exchange_16`, which an MSVC link does not have, unless
            // x64's 16-byte compare-exchange is enabled; cl itself has it always.
            if cfg!(target_arch = "x86_64") && is_clang_cl(cl) {
                command.arg("-mcx16");
            }
            // `[FFI-8]` — a C header's `_Atomic` (`<stdatomic.h>`) compiles with cl only with C11
            // atomics switched on (Visual Studio 17.5 on); clang-cl has them always.
            if !is_clang_cl(cl) {
                command.arg("/experimental:c11atomics");
            }
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
        // `[RT-4]` — development keeps its symbols in a PDB beside the program, for a panic's
        // backtrace; the code is the same (the link keeps `/OPT:REF,ICF`).
        Profile::Development => &["/O2", "/Zi", "/MD"],
        Profile::Release => &["/O2", "/MD", "/DEMBER_RELEASE"],
    };
    // `[BLD-FFI-1b]` — a C++ project's runtime switch and layout defines, where the build inherits them.
    let inherited = inherited_msvc_runtime();
    let switch = msvc_runtime_switch();
    ["/nologo", "/std:c11", "/W3", "/fp:precise"]
        .iter()
        .chain(optimisation)
        .copied()
        .map(|flag| if flag == "/MD" { switch } else { flag })
        .chain(inherited.into_iter().filter(|flag| !flag.starts_with("/M")))
        .collect()
}

/// `[BLD-FFI-1b]` — the MSVC runtime-library switch and the defines that change the C++ standard
/// library's layout (`_DEBUG`, `_ITERATOR_DEBUG_LEVEL`) that the build inherits from the C++ project
/// it imports, which every translation unit of the target is then compiled with. Empty when there
/// is none to inherit: the release runtime, `/MD` (D-252), and no define.
static INHERITED_MSVC_RUNTIME: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());

/// `[BLD-FFI-1b]` — inherit a C++ project's runtime switch and layout defines for this build.
pub fn inherit_msvc_runtime(flags: Vec<&'static str>) {
    if let Ok(mut inherited) = INHERITED_MSVC_RUNTIME.lock() {
        *inherited = flags;
    }
}

fn inherited_msvc_runtime() -> Vec<&'static str> {
    INHERITED_MSVC_RUNTIME.lock().map(|inherited| inherited.clone()).unwrap_or_default()
}

/// `[BLD-FFI-4]` — the link-time optimisation a C++ project's GCC or Clang flags ask for (`-flto`,
/// `-flto=thin`, …), which the program's own C is then compiled and linked with too, so the link
/// inlines calls across the boundary as the project's own. `None` for none.
static INHERITED_GNU_LTO: std::sync::Mutex<Option<&'static str>> = std::sync::Mutex::new(None);

/// `[BLD-FFI-4]` — inherit a C++ project's link-time optimisation flag for this build.
pub fn inherit_gnu_lto(flag: Option<&'static str>) {
    if let Ok(mut inherited) = INHERITED_GNU_LTO.lock() {
        *inherited = flag;
    }
}

fn inherited_gnu_lto() -> Option<&'static str> {
    INHERITED_GNU_LTO.lock().ok().and_then(|inherited| *inherited)
}

/// `[BLD-FFI-1b]` — the defines that change the C++ standard library's layout under GCC and Clang
/// (libstdc++'s debug mode, `-D_GLIBCXX_DEBUG`) that the build inherits from the C++ project it
/// imports, which every C++ translation unit it compiles (each import's thunks) is then compiled
/// with. Empty for none.
static INHERITED_GNU_RUNTIME: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());

/// `[BLD-FFI-1b]` — inherit a C++ project's layout defines for this build's C++ translation units.
pub fn inherit_gnu_runtime(flags: Vec<&'static str>) {
    if let Ok(mut inherited) = INHERITED_GNU_RUNTIME.lock() {
        *inherited = flags;
    }
}

fn inherited_gnu_runtime() -> Vec<&'static str> {
    INHERITED_GNU_RUNTIME.lock().map(|inherited| inherited.clone()).unwrap_or_default()
}

/// `[BLD-6]` — the link-time optimisation a build of `profile` does for objects linked `link`: what
/// it asks for ([`asked_lto`]), or the nearest the toolchain does.
pub fn build_lto(toolchain: &Toolchain, profile: Profile, link: Link) -> Lto {
    toolchain.lto(asked_lto(profile), link)
}

/// `[BLD-6]` — what a build record says of link-time optimisation: what the build asked for, what
/// it did, and why the two differ when they do. A C++ project's own flag, which the program's C
/// inherits with GCC or Clang (`[BLD-FFI-4]`), is what it did then.
pub fn lto_record(toolchain: &Toolchain, profile: Profile, link: Link) -> (Lto, Lto, Option<String>) {
    let asked = asked_lto(profile);
    if let (Toolchain::Clang(_) | Toolchain::Gcc(_), Some(flag)) = (toolchain, inherited_gnu_lto()) {
        let used = match flag {
            "" => Lto::Off,
            "-flto=thin" => Lto::Thin,
            _ => Lto::On,
        };
        let why = (used != asked).then(|| "the imported C++ project's own flags choose it [BLD-FFI-4]".to_string());
        return (asked, used, why);
    }
    let used = toolchain.lto(asked, link);
    let whose = if link == Link::Elsewhere { " for objects another link reads" } else { "" };
    let why = (used != asked).then(|| {
        format!("{} does not do `{}` link-time optimisation{whose}; `{}` is the nearest it does", toolchain_name(toolchain), asked.name(), used.name())
    });
    (asked, used, why)
}

/// `[BLD-6]` — the C compiler's flags for `lto`, one the toolchain does ([`Toolchain::lto`]): an
/// object a link elsewhere reads is gcc's fat one, which holds the code too.
fn lto_compile_flags(toolchain: &Toolchain, lto: Lto, link: Link) -> &'static [&'static str] {
    match (toolchain, lto) {
        (_, Lto::Off) => &[],
        (Toolchain::Msvc { cl, .. }, _) if !is_clang_cl(cl) => &["/GL"],
        (Toolchain::Msvc { .. } | Toolchain::Clang(_), Lto::Thin) => &["-flto=thin"],
        (Toolchain::Msvc { .. } | Toolchain::Clang(_), _) => &["-flto"],
        (Toolchain::Gcc(_), _) if link == Link::Elsewhere => &["-flto=auto", "-ffat-lto-objects"],
        (Toolchain::Gcc(_), _) => &["-flto=auto"],
    }
}

/// `[BLD-6]` — the link's flags for `lto`: cl's linker's, after `/link`; clang-cl links with
/// `lld-link` already (`-fuse-ld=lld`), which reads its objects as they are; gcc and Clang
/// optimise at the link with the compile's flag, Clang's linked by `lld` (Apple's linker reads
/// them itself).
fn lto_link_flags(toolchain: &Toolchain, lto: Lto) -> &'static [&'static str] {
    let apple = cfg!(target_os = "macos");
    match (toolchain, lto) {
        (_, Lto::Off) => &[],
        (Toolchain::Msvc { cl, .. }, _) if !is_clang_cl(cl) => &["/LTCG"],
        (Toolchain::Msvc { .. }, _) => &[],
        (Toolchain::Clang(_), Lto::Thin) if apple => &["-flto=thin"],
        (Toolchain::Clang(_), _) if apple => &["-flto"],
        (Toolchain::Clang(_), Lto::Thin) => &["-flto=thin", "-fuse-ld=lld"],
        (Toolchain::Clang(_), _) => &["-flto", "-fuse-ld=lld"],
        (Toolchain::Gcc(_), _) => &["-flto=auto"],
    }
}

/// `[BLD-FFI-1b]` — `/MD`, or the runtime switch the build inherits.
pub fn msvc_runtime_switch() -> &'static str {
    inherited_msvc_runtime().into_iter().find(|flag| flag.starts_with("/M")).unwrap_or("/MD")
}

/// clang's and gcc's flags for a profile, shared by the program's compile and
/// the runtime object's so the two always agree. `-ffp-contract=off` and
/// `-fno-fast-math` are strict IEEE arithmetic (`[CG-C-11]`, D-325): no
/// `a * b + c` fused where the target has the instruction.
fn gnu_flags(profile: Profile) -> Vec<&'static str> {
    let optimisation: &[&str] = match profile {
        Profile::Debug => &["-O0", "-g", "-DEMBER_RT_DEBUG_ACCESS"],
        // `[RT-4]` — on Windows a development build keeps its symbols in a PDB, for a panic's backtrace;
        // elsewhere the program's own symbol table names its functions.
        Profile::Development if cfg!(windows) => &["-O2", "-g"],
        Profile::Development => &["-O2"],
        Profile::Release => &["-O3", "-DEMBER_RELEASE"],
    };
    // `[THR-14]` — x64's 16-byte compare-exchange inline, for `Atomic[i128]`: without it GCC calls
    // libatomic, which a program does not link.
    let cx16: &[&str] = if cfg!(target_arch = "x86_64") { &["-mcx16"] } else { &[] };
    // `[RT-12]` — a frame larger than a page touches each page in order, so no write lands beyond
    // the guard page (MSVC and clang-cl probe by default).
    let probes: &[&str] = if cfg!(target_os = "linux") { &["-fstack-clash-protection"] } else { &[] };
    ["-std=c11", "-Wall", "-Wextra", "-ffp-contract=off", "-fno-fast-math"].iter().chain(optimisation).chain(cx16).chain(probes).copied().collect()
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
        (Toolchain::Gcc(_), Profile::Development) => &["-fvect-cost-model=cheap", "-funroll-loops"],
        _ => &[],
    }
}

fn run(command: Command) -> Result<(), BuildError> {
    run_output(command).map(drop)
}

/// `run`, keeping what the compiler wrote.
fn run_output(mut command: Command) -> Result<std::process::Output, BuildError> {
    let rendered = format!("{command:?}");
    let output = command.output()?;
    if !output.status.success() {
        let mut text = String::new();
        for stream in [&output.stdout, &output.stderr] {
            // The include notes the program cache asks for are not the failure.
            for line in String::from_utf8_lossy(stream).lines().filter(|line| included_file(line).is_none()) {
                text.push_str(line);
                text.push('\n');
            }
        }
        return Err(BuildError::CompilerFailed {
            command: rendered,
            output: text,
        });
    }
    Ok(output)
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
/// `release` path (`/GL` compiles for link-time code generation, which wants
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
    // `[BLD-6]` — the program's link-time optimisation reaches into the runtime too.
    let lto = build_lto(toolchain, profile, Link::Here);
    let extension = match toolchain {
        Toolchain::Msvc { cl, .. } if lto != Lto::Off && !is_clang_cl(cl) => return Ok(None),
        Toolchain::Msvc { .. } => {
            // `/Z7` keeps the debug information inside the object; `/Zi`
            // would tie a shared object to a PDB file beside it.
            let flags = msvc_flags(profile).into_iter().map(|f| if f == "/Zi" { "/Z7" } else { f });
            command.args(flags).args(lto_compile_flags(toolchain, lto, Link::Here)).arg("/c");
            for dir in include_dirs {
                command.arg(format!("/I{}", dir.display()));
            }
            "obj"
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.args(gnu_flags(profile)).args(gcc_flags(toolchain, profile))
                .args(executable_section_flags(toolchain, managed_executable)).arg("-c");
            // A C++ project's own link-time optimisation leaves the runtime as it was (`[BLD-FFI-4]`).
            if inherited_gnu_lto().is_none() {
                command.args(lto_compile_flags(toolchain, lto, Link::Here));
            }
            folder_names(&mut command, &[source.to_path_buf()], include_dirs);
            for dir in include_dirs {
                command.arg("-I").arg(dir);
            }
            "o"
        }
    };
    command.arg(source);

    let mut key = Key::new();
    key.part(format!("{command:?}").as_bytes());
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
            key.part(value.as_bytes());
        }
    }
    key.part(&std::fs::read(source)?);
    // ponytail: the include directories' own files, not their subdirectories
    // (the runtime's headers are flat); walk deeper if that changes.
    for dir in include_dirs {
        let mut headers: Vec<PathBuf> = std::fs::read_dir(dir)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_file())
            .collect();
        headers.sort();
        for header in headers {
            key.part(header.to_string_lossy().as_bytes());
            key.part(&std::fs::read(&header)?);
        }
    }
    key.compiler_file(toolchain);

    let dir = cache.join("runtime");
    if std::fs::create_dir_all(&dir).is_err() {
        return Ok(None);
    }
    let name = key.finish();
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
    program_file(program, search.or_else(|| std::env::var_os("PATH")))
}

/// The file a program name runs: a path as it is, a bare name found on `search`.
fn program_file(program: &Path, search: Option<std::ffi::OsString>) -> Option<PathBuf> {
    if program.components().count() > 1 {
        return Some(program.to_path_buf());
    }
    std::env::split_paths(&search?)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    /// `[TOOL-3]` — with no C compiler, the message is `E9002` saying so, and the help names this
    /// host's remedy; the path the search tried is never the primary message.
    #[test]
    fn no_c_compiler_is_e9002_with_the_hosts_remedy() {
        let any = super::BuildError::NoToolchain("any").to_string();
        assert!(any.starts_with("error[E9002]: no C compiler was found"), "{any}");
        assert!(any.contains("= help: install"), "{any}");
        if cfg!(windows) {
            assert!(any.contains("Visual Studio Build Tools"), "{any}");
        }
        let one = super::BuildError::NoToolchain("clang-cl").to_string();
        assert!(one.contains("looked for `clang-cl`"), "{one}");
    }

    /// `[TYP-9c]` — the C compiler's own words for a relaxing flag it ignored or refused are
    /// `E9041`'s evidence: MSVC's `D9002`, clang's "unknown argument", gcc's "unrecognized
    /// command-line option"; any other output, or another flag, is not.
    #[test]
    fn a_refused_float_flag_is_found_in_the_compilers_output() {
        let relaxing = ["/fp:contract", "-ffp-contract=fast"];
        let cl = "unit.c\ncl : Command line warning D9002 : ignoring unknown option '/fp:contract'\n";
        assert_eq!(super::refused_relaxation(cl, &relaxing), Some("/fp:contract"));
        let clang = "clang: error: unknown argument: '-ffp-contract=fast'\n";
        assert_eq!(super::refused_relaxation(clang, &relaxing), Some("-ffp-contract=fast"));
        let gcc = "gcc: error: unrecognized command-line option '-ffp-contract=fast'\n";
        assert_eq!(super::refused_relaxation(gcc, &relaxing), Some("-ffp-contract=fast"));
        assert_eq!(super::refused_relaxation("unit.c\n", &relaxing), None);
        let other = "cl : Command line warning D9002 : ignoring unknown option '/arch:AVX9'\n";
        assert_eq!(super::refused_relaxation(other, &relaxing), None);
    }

    use super::*;

    /// Runs a program a test has just written. On Linux another test thread
    /// may fork while this one still holds the new file open for writing;
    /// until that child execs, its copy of the descriptor keeps the file
    /// busy, and starting it fails with "Text file busy" (CI, D-573). That
    /// is over within moments, so the start is retried for up to a second.
    fn run_fresh(command: &mut Command) -> std::io::Result<std::process::Output> {
        let mut tries = 0;
        loop {
            match command.output() {
                Err(error) if error.kind() == std::io::ErrorKind::ExecutableFileBusy && tries < 100 => {
                    tries += 1;
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                result => return result,
            }
        }
    }

    /// `[BLD-6]` — each toolchain's kinds of link-time optimisation, the nearest of them to what a
    /// build asks for (of two as near, the one that optimises more), and their flags: cl does the
    /// whole program's (`/GL`, `/LTCG`) at its own link; gcc the whole program's, as fat objects
    /// for a link elsewhere; clang-cl both kinds at its own link. Neither cl's objects nor LLVM's
    /// serve a link elsewhere as they are, so a library another link reads gets none from them.
    /// `[PRF-3]`: `debug` off, `development` thin, `release` on.
    #[test]
    fn link_time_optimisation_is_the_nearest_the_toolchain_does() {
        let cl = Toolchain::Msvc { cl: PathBuf::from("cl.exe"), env: BTreeMap::new() };
        let clang_cl = Toolchain::Msvc { cl: PathBuf::from("clang-cl.exe"), env: BTreeMap::new() };
        let gcc = Toolchain::Gcc(PathBuf::from("gcc"));
        assert_eq!([Profile::Debug.lto(), Profile::Development.lto(), Profile::Release.lto()], [Lto::Off, Lto::Thin, Lto::On]);
        assert_eq!(Lto::ALL.map(|lto| Lto::from_name(lto.name())), [Some(Lto::Off), Some(Lto::Thin), Some(Lto::On)]);
        assert_eq!(Lto::from_name("full"), None);
        for link in [Link::Here, Link::Elsewhere] {
            assert_eq!(Lto::ALL.map(|asked| gcc.lto(asked, link)), [Lto::Off, Lto::On, Lto::On], "{link:?}");
        }
        assert_eq!(Lto::ALL.map(|asked| cl.lto(asked, Link::Here)), [Lto::Off, Lto::On, Lto::On]);
        assert_eq!(Lto::ALL.map(|asked| clang_cl.lto(asked, Link::Here)), Lto::ALL);
        for toolchain in [&cl, &clang_cl] {
            assert_eq!(Lto::ALL.map(|asked| toolchain.lto(asked, Link::Elsewhere)), [Lto::Off; 3], "{toolchain:?}");
        }
        assert_eq!(lto_compile_flags(&cl, Lto::On, Link::Here), ["/GL"]);
        assert_eq!(lto_link_flags(&cl, Lto::On), ["/LTCG"]);
        assert_eq!(lto_compile_flags(&clang_cl, Lto::Thin, Link::Here), ["-flto=thin"]);
        assert!(lto_link_flags(&clang_cl, Lto::On).is_empty(), "clang-cl links with lld-link already");
        assert_eq!(lto_compile_flags(&gcc, Lto::On, Link::Here), ["-flto=auto"]);
        assert_eq!(lto_compile_flags(&gcc, Lto::On, Link::Elsewhere), ["-flto=auto", "-ffat-lto-objects"]);
        assert_eq!(lto_link_flags(&gcc, Lto::On), ["-flto=auto"]);
        for toolchain in [&cl, &clang_cl, &gcc] {
            assert!(lto_compile_flags(toolchain, Lto::Off, Link::Here).is_empty() && lto_link_flags(toolchain, Lto::Off).is_empty());
        }
        // The profile's flags carry none of it: a relaxed unit (`[CG-C-11]`) is never optimised
        // across, and every other unit takes the build's own.
        assert!(!msvc_flags(Profile::Release).contains(&"/GL"));
        assert!(!gnu_flags(Profile::Release).iter().any(|flag| flag.starts_with("-flto")));
    }

    /// ADR-098 — gcc's development build asks for the vectoriser cost model that
    /// takes a loop with a scalar tail; clang (which does by default), the
    /// other profiles and MSVC get nothing extra.
    #[test]
    fn gcc_release_asks_for_the_cheap_vectoriser_model_and_unrolling() {
        let gcc = Toolchain::Gcc(PathBuf::from("gcc"));
        let clang = Toolchain::Clang(PathBuf::from("clang"));
        assert_eq!(gcc_flags(&gcc, Profile::Development), ["-fvect-cost-model=cheap", "-funroll-loops"]);
        assert!(gcc_flags(&gcc, Profile::Debug).is_empty());
        assert!(gcc_flags(&gcc, Profile::Release).is_empty());
        assert!(gcc_flags(&clang, Profile::Development).is_empty());
    }

    /// ADR-136 — a second identical build is the kept program, copied; a
    /// changed source builds again, and so does a changed file it includes from
    /// beside itself, which no include directory names.
    #[test]
    fn program_cache_serves_identical_builds_and_sees_every_included_file() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let dir = std::env::temp_dir().join(format!("program-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let cache = dir.join("cache");
        let (source, header) = (dir.join("main.c"), dir.join("value.h"));
        let write_header = |value: u32| {
            std::fs::write(&header, format!("#define VALUE {value}\n")).expect("the header is writable");
            // A file written just before its build is not kept with it.
            let past = std::time::SystemTime::now() - std::time::Duration::from_secs(60);
            std::fs::File::options().write(true).open(&header).and_then(|file| file.set_modified(past))
                .expect("the header's time is settable");
        };
        let write_source = |extra: u32| {
            let text = format!("#include <stdio.h>\n#include \"value.h\"\nint main(void) {{ printf(\"%d\\n\", VALUE + {extra}); return 0; }}\n");
            std::fs::write(&source, text).expect("the source is writable");
        };
        let program = |name: &str| dir.join(if cfg!(windows) { format!("{name}.exe") } else { name.to_string() });
        let build = |name: &str| {
            compile_and_link_impl(&toolchain, &LinkRequest {
                sources: &[source.clone()], include_dirs: &[], output: program(name),
                profile: Profile::Debug, obj_dir: dir.join("obj"),
            }, None, &[], Some(&cache), 0).expect("the program builds");
            let output = run_fresh(&mut Command::new(program(name))).expect("the program runs");
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };
        let kept = || {
            let mut found: Vec<PathBuf> = std::fs::read_dir(&cache).expect("the cache exists").flatten()
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == if cfg!(windows) { "exe" } else { "bin" }))
                .collect();
            found.sort();
            found
        };
        write_header(1);
        write_source(0);
        assert_eq!(build("first"), "1");
        let first = kept();
        assert_eq!(first.len(), 1, "one program kept");
        // Marked, to tell the kept program from one built again.
        let mut marked = std::fs::read(&first[0]).expect("the kept program reads");
        marked.extend_from_slice(b"kept");
        std::fs::write(&first[0], &marked).expect("the kept program is writable");
        assert_eq!(build("second"), "1");
        assert_eq!(std::fs::read(program("second")).expect("the second program reads"), marked,
            "the second build is the kept program");
        write_header(2);
        assert_eq!(build("third"), "2", "a changed include builds again");
        write_source(5);
        assert_eq!(build("fourth"), "7", "a changed source builds again");
        assert_eq!(kept().len(), 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The trim removes the least recently used programs first, down to three
    /// quarters of the limit, and keeps its own stamp.
    #[test]
    fn program_cache_trims_the_least_recently_used() {
        let dir = std::env::temp_dir().join(format!("program-trim-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("the test directory is creatable");
        let now = std::time::SystemTime::now();
        for age in 1..=4u64 {
            let file = dir.join(format!("{age}.exe"));
            std::fs::write(&file, [0u8; 100]).expect("a program is writable");
            std::fs::File::options().write(true).open(&file)
                .and_then(|open| open.set_modified(now - std::time::Duration::from_secs(age * 3600)))
                .expect("a program's time is settable");
        }
        trim_programs(&dir, 250);
        let mut left: Vec<String> = std::fs::read_dir(&dir).expect("the folder lists").flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(left, ["1.exe", "trimmed"]);
        // Within the hour, no second trim.
        std::fs::write(dir.join("5.exe"), [0u8; 300]).expect("a program is writable");
        trim_programs(&dir, 250);
        assert!(dir.join("5.exe").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Include notes from `-H` and from `/showIncludes` in two languages; a
    /// diagnostic, which starts with its file, is none.
    #[test]
    fn include_notes_name_their_files() {
        assert_eq!(included_file(". /usr/include/stdio.h"), Some("/usr/include/stdio.h"));
        assert_eq!(included_file(".. C:\\llvm\\stdint.h"), Some("C:\\llvm\\stdint.h"));
        assert_eq!(included_file("Note: including file:  C:\\VC\\include\\stdio.h"), Some("C:\\VC\\include\\stdio.h"));
        assert_eq!(included_file("Remarque : inclusion du fichier :   C:\\a b\\x.h"), Some("C:\\a b\\x.h"));
        assert_eq!(included_file("Note: including file: \\\\server\\share\\x.h"), Some("\\\\server\\share\\x.h"));
        assert_eq!(included_file("C:\\x\\main.c(3): error C2143: syntax error: missing ';'"), None);
        assert_eq!(included_file("C:\\x\\main.c(1): fatal error C1083: Cannot open include file: 'C:\\y.h': No such file"), None);
        assert_eq!(included_file("main.c"), None);
        assert_eq!(included_file("main.c:3:5: error: expected ';'"), None);
    }

    #[test]
    fn profile_names_round_trip() {
        for profile in [Profile::Debug, Profile::Development, Profile::Release] {
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
        for profile in [Profile::Debug, Profile::Development, Profile::Release] {
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
                    let output = run_fresh(Command::new(&executable).arg(mode)).unwrap();
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
                // A root stays; what nothing calls goes with section removal or link-time optimisation.
                let lto = build_lto(&toolchain, profile, Link::Here) != Lto::Off;
                assert_eq!(names.contains(&"retained_c_root"), uses_gc || !lto, "{} {lto}", profile.name());
                assert_eq!(names.contains(&"dead_helper"), !uses_gc && !lto, "{} {lto}", profile.name());
                let elf = Command::new("readelf").args(["-lW"]).arg(&executable).output().unwrap();
                assert!(elf.status.success());
                let headers = String::from_utf8_lossy(&elf.stdout);
                assert!(headers.contains("GNU_RELRO"));
                let stack = headers.lines().find(|line| line.contains("GNU_STACK")).expect("stack permissions");
                assert!(stack.contains("RW ") && !stack.contains("RWE"), "{stack}");
                // D-502: the runtime calls nothing in libm, so a program that
                // does not either never loads it.
                let dynamic = Command::new("readelf").args(["-dW"]).arg(&executable).output().unwrap();
                let dynamic = String::from_utf8_lossy(&dynamic.stdout);
                assert!(!dynamic.contains("libm.so"), "the runtime made libm needed (section removal {uses_gc}):
{dynamic}");
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
            profile: Profile::Development, obj_dir: dir.join("obj"),
        }, &["retained_c_root".to_string()]).expect("ordinary fallback links");
        assert!(Command::new(&executable).status().unwrap().success());
        let listed = Command::new("nm").arg("--defined-only").arg(executable).output().unwrap();
        assert!(listed.status.success());
        let listing = String::from_utf8_lossy(&listed.stdout);
        let names: Vec<_> = listing.lines().filter_map(|line| line.split_whitespace().last()).collect();
        // The root stays without section removal too; the helper goes only with link-time optimisation.
        let lto = build_lto(&simulated, Profile::Development, Link::Here) != Lto::Off;
        assert!(names.contains(&"retained_c_root"), "{listing}");
        assert_eq!(names.contains(&"dead_helper"), !lto, "{listing}");
    }

    /// ADR-118's probe runs GCC and its linker once; the answer is reused
    /// until the linker GCC names is a different file. D-510: the scripts are
    /// run just after they are written, which another test's new child can
    /// make "text file busy" for a moment; the probe tries again, and keeps
    /// no answer it could not finish.
    #[cfg(target_os = "linux")]
    #[test]
    fn the_linker_probe_is_cached_until_the_linker_changes() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("linker-probe-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let linker = dir.join("linker");
        let log = dir.join("calls");
        let shell_quote = |path: &Path| format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"));
        let wrapper = dir.join("gcc-wrapper");
        std::fs::write(&wrapper, format!(
            "#!/bin/sh\necho \"$1\" >> {}\nif [ \"$1\" = '-print-prog-name=ld' ]; then printf '%s\\n' {}; exit 0; fi\nexit 1\n",
            shell_quote(&log), shell_quote(&linker))).unwrap();
        let both = "#!/bin/sh\nprintf '%s\\n' '--gc-sections' '--require-defined=SYMBOL'\n";
        std::fs::write(&linker, both).unwrap();
        for script in [&linker, &wrapper] {
            std::fs::set_permissions(script, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let simulated = Toolchain::Gcc(wrapper);
        let calls = || std::fs::read_to_string(&log).unwrap_or_default().lines().count();
        assert!(executable_section_gc_supported(&simulated));
        assert_eq!(calls(), 1);
        assert!(executable_section_gc_supported(&simulated), "the cached answer");
        assert_eq!(calls(), 1, "a cached answer runs nothing");
        // A different linker at the same path: asked again.
        std::fs::write(&linker, "#!/bin/sh\nprintf '%s\\n' '--gc-sections'\n").unwrap();
        assert!(!executable_section_gc_supported(&simulated));
        assert_eq!(calls(), 2);
        // D-510 — a linker that cannot be started gives no answer to keep:
        // once it can be, it is asked.
        std::fs::write(&linker, both).unwrap();
        std::fs::set_permissions(&linker, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(!executable_section_gc_supported(&simulated));
        assert_eq!(calls(), 3);
        std::fs::set_permissions(&linker, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(executable_section_gc_supported(&simulated), "the failed start was not kept");
        assert_eq!(calls(), 4);
        let _ = std::fs::remove_dir_all(&dir);
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
            profile: Profile::Development, obj_dir: dir.join("obj"),
        }).expect("the guarded byte-sum oracle compiles");
        let output = run_fresh(&mut Command::new(program)).expect("the guarded oracle runs");
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
            profile: Profile::Development, obj_dir: dir.join("obj"),
        }).expect("the guarded scalar-decoder oracle compiles");
        let output = run_fresh(&mut Command::new(program)).expect("the guarded oracle runs");
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
            command.args(gnu_flags(Profile::Development))
                .arg("-I").arg(runtime.join("include"))
                .arg(&fixture).arg("-o").arg(&program)
                .arg("-rtlib=compiler-rt");
            run(command).expect("the allocator-rounding oracle compiles");
        } else {
            compile_and_link(&toolchain, &LinkRequest {
                sources: &[fixture], include_dirs: &[runtime.join("include")],
                output: program.clone(), profile: Profile::Development, obj_dir: dir.join("obj"),
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

    /// `[RC-4]`, `[WK-12]` — a Sync object's counts under four threads at
    /// once: their retains and releases balance exactly, and weak handles
    /// upgraded while the last strong one is released never bring the object
    /// back (an upgrade holds it alive or finds it gone), each object
    /// deinitialised once. Ember has no threads of its own yet (Phase 6), so
    /// the runtime is driven from C (`templates/tests/sync_counts.c.in`).
    #[test]
    fn sync_counts_hold_under_threads() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("sync-counts-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "sync_counts.exe" } else { "sync_counts" });
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[runtime.join("tests/sync_counts.c"), runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))],
            include_dirs: &[runtime.join("include")], output: program.clone(),
            profile: Profile::Development, obj_dir: dir.join("obj"),
        }).expect("the Sync-count oracle compiles");
        let output = run_fresh(&mut Command::new(program)).expect("the Sync-count oracle runs");
        assert!(output.status.success(), "Sync counts failed: {}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `[TXT-6]` — across the C ABI a `str` is `{const uint8_t *ptr; size_t len}`: the runtime header
    /// every program and every exported header includes defines it so.
    #[test]
    fn a_str_is_a_byte_pointer_and_a_length() {
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let header = std::fs::read_to_string(runtime.join("include").join(ember_branding::runtime_header()))
            .expect("the runtime header is readable");
        let name = format!("{}_str", ember_branding::SYMBOL_PREFIX);
        let definition = format!("typedef struct {name} {{
    const uint8_t* ptr;
    size_t len;
}} {name};");
        assert!(header.contains(&definition), "[TXT-6] `{name}` is not {{const uint8_t *ptr; size_t len}}");
    }

    /// `[CG-C-5]` — every panic function the runtime declares is `noreturn` and cold (gcc and
    /// clang's `cold`; MSVC has none), so a check's call is a forward branch predicted not taken.
    #[test]
    fn panic_functions_are_noreturn_and_cold() {
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let header = std::fs::read_to_string(runtime.join("include").join(ember_branding::runtime_header()))
            .expect("the runtime header is readable");
        let upper = ember_branding::SYMBOL_PREFIX.to_uppercase();
        assert!(header.contains(&format!("#define {upper}_COLD __attribute__((cold))")), "[CG-C-5] no cold attribute");
        assert!(header.contains(&format!("#define {upper}_PANIC_FN {upper}_NORETURN {upper}_COLD")), "[CG-C-5] no panic attribute");
        let noreturn = format!("{upper}_NORETURN void ");
        let undecorated: Vec<&str> = header.lines().filter(|line| line.trim_start().starts_with(&noreturn)).collect();
        assert!(undecorated.is_empty(), "[CG-C-5] a noreturn function that is not a cold panic function: {undecorated:?}");
        let panic = format!("{upper}_PANIC_FN void {}panic", ember_branding::RUNTIME_PREFIX);
        assert!(header.lines().filter(|line| line.trim_start().starts_with(&panic)).count() >= 9, "[CG-C-5] the panic family is not cold");
    }

    /// `[RT-7]` — a reference count never wraps: a retain at the count's maximum panics with
    /// `reference count overflow on <Class>`, for a plain and a Sync object; a retain of an
    /// object at zero is still the invalid-object panic (`templates/tests/count_overflow.c.in`).
    #[test]
    fn a_reference_count_never_wraps() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("count-overflow-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "count_overflow.exe" } else { "count_overflow" });
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[runtime.join("tests/count_overflow.c"), runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))],
            include_dirs: &[runtime.join("include")], output: program.clone(),
            profile: Profile::Development, obj_dir: dir.join("obj"),
        }).expect("the count-overflow oracle compiles");
        for (mode, message) in [
            ("plain", "reference count overflow on Node"),
            ("sync", "reference count overflow on Shared Node"),
            ("zero", "retain of deinitialising or invalid object"),
        ] {
            let output = run_fresh(Command::new(&program).arg(mode)).expect("the count-overflow oracle runs");
            assert!(output.status.success(), "[RT-7] {mode}: exit {:?}: {}", output.status.code(), String::from_utf8_lossy(&output.stderr));
            assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), message, "[RT-7] {mode}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `[FFI-33a]` — in the debug profile an object that is not Sync is its maker's: a foreign
    /// thread that attaches and then retains, releases or weakly retains one panics naming the
    /// class and both threads; a Sync object is any thread's, and the maker counts freely
    /// (`templates/tests/object_creator_thread.c.in`). A development or release build does not check.
    #[test]
    fn an_object_that_is_not_sync_is_its_makers() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("creator-thread-{}", std::process::id()));
        for profile in [Profile::Debug, Profile::Development] {
            let program = dir.join(format!("creator_thread_{profile:?}{}", if cfg!(windows) { ".exe" } else { "" }));
            std::fs::create_dir_all(dir.join(format!("obj_{profile:?}"))).expect("the test directory is creatable");
            compile_and_link(&toolchain, &LinkRequest {
                sources: &[runtime.join("tests/object_creator_thread.c"), runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))],
                include_dirs: &[runtime.join("include")], output: program.clone(),
                profile, obj_dir: dir.join(format!("obj_{profile:?}")),
            }).expect("the creator-thread oracle compiles");
            for mode in ["retain", "release", "weak", "sync", "same"] {
                let output = run_fresh(Command::new(&program).arg(mode)).expect("the creator-thread oracle runs");
                assert!(output.status.success(), "[FFI-33a] {profile:?} {mode}: exit {:?}: {}", output.status.code(), String::from_utf8_lossy(&output.stderr));
                let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if profile == Profile::Debug && matches!(mode, "retain" | "release" | "weak") {
                    assert!(out.starts_with("a `Node` object made on thread 1 (main) is retained or released on thread 2:"), "[FFI-33a] {mode}: {out}");
                } else {
                    assert_eq!(out, "ok", "[FFI-33a] {profile:?} {mode}");
                }
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `[THR-3]` — a mutex word serialises four threads' critical sections: no addition under
    /// the lock is lost, contended locking sleeps on the word, and the word ends unlocked.
    #[test]
    fn a_mutex_word_serialises_threads() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("mutex-threads-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "mutex_threads.exe" } else { "mutex_threads" });
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[runtime.join("tests/mutex_threads.c"), runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))],
            include_dirs: &[runtime.join("include")], output: program.clone(),
            profile: Profile::Development, obj_dir: dir.join("obj"),
        }).expect("the mutex oracle compiles");
        let output = run_fresh(&mut Command::new(program)).expect("the mutex oracle runs");
        assert!(output.status.success(), "the mutex oracle failed ({:?}): {}", output.status.code(), String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `[JOB-1]`, `[JOB-2]`, `[JOB-3]` — the job pool: a scope's end waits for every job of it, a
    /// state larger than a slot runs from its own block, a job given another starts after it, and
    /// more jobs than slots are waited out.
    #[test]
    fn the_job_pool_runs_every_job_once_in_order() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("job-pool-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "job_pool.exe" } else { "job_pool" });
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[runtime.join("tests/job_pool.c"), runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))],
            include_dirs: &[runtime.join("include")], output: program.clone(),
            profile: Profile::Development, obj_dir: dir.join("obj"),
        }).expect("the job-pool oracle compiles");
        let output = run_fresh(&mut Command::new(program)).expect("the job-pool oracle runs");
        assert!(output.status.success(), "the job-pool oracle failed ({:?}): {}{}", output.status.code(), String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `[THR-3]` — an `RwLock` word lets readers share it and a writer have it alone, under four
    /// threads: no reader sees a write half done, and no write is lost.
    #[test]
    fn an_rwlock_word_shares_reads_and_serialises_writes() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("rwlock-threads-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "rwlock_threads.exe" } else { "rwlock_threads" });
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[runtime.join("tests/rwlock_threads.c"), runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))],
            include_dirs: &[runtime.join("include")], output: program.clone(),
            profile: Profile::Development, obj_dir: dir.join("obj"),
        }).expect("the rwlock oracle compiles");
        let output = run_fresh(&mut Command::new(program)).expect("the rwlock oracle runs");
        assert!(output.status.success(), "the rwlock oracle failed ({:?}): {}", output.status.code(), String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// D-586 — `[STA-3]`: a lazily initialised static's initialiser runs once
    /// when four threads ask for it first together; the rest wait for it.
    #[test]
    fn a_lazy_static_initialises_once_across_threads() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("lazy-static-threads-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "lazy_static_threads.exe" } else { "lazy_static_threads" });
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[runtime.join("tests/lazy_static_threads.c"), runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))],
            include_dirs: &[runtime.join("include")], output: program.clone(),
            profile: Profile::Development, obj_dir: dir.join("obj"),
        }).expect("the lazy-static oracle compiles");
        let output = run_fresh(&mut Command::new(program)).expect("the lazy-static oracle runs");
        assert!(output.status.success(), "the lazy-static oracle failed ({:?}): {}", output.status.code(), String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// D-583 — `[VER-4]`: `ember_rt_init` refuses a caller compiled against
    /// another runtime ABI, naming both versions, and reads no
    /// `ember_rt_config` field past the caller's `size`.
    #[test]
    fn the_runtime_checks_its_callers_abi_version() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("abi-version-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "abi_version.exe" } else { "abi_version" });
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[runtime.join("tests/abi_version.c"), runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))],
            include_dirs: &[runtime.join("include")], output: program.clone(),
            profile: Profile::Development, obj_dir: dir.join("obj"),
        }).expect("the ABI-version oracle compiles");
        let output = run_fresh(&mut Command::new(program)).expect("the ABI-version oracle runs");
        assert!(output.status.success(), "the ABI-version oracle failed: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("ABI version 3, and its caller was compiled against version 4"), "{stderr}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `[FFI-27]` — a C host routes allocation and logging to its own functions; shutdown runs the
    /// at-exit hooks, newest first, before the debug leak report, which reaches the host's log as
    /// one warning and nothing of it standard error. `[FFI-31]`, `[FFI-31c]`: the modules of one
    /// runtime are counted, and the last one's shutdown ends a runtime the host did not start.
    /// `[ABI-3]`: a module of another runtime ABI is refused with its own code
    /// (`templates/tests/embedding.c.in` says each step).
    #[test]
    fn a_host_routes_logging_and_shutdown_runs_at_exit_hooks() {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let dir = std::env::temp_dir().join(format!("embedding-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("obj")).expect("the test directory is creatable");
        let program = dir.join(if cfg!(windows) { "embedding.exe" } else { "embedding" });
        compile_and_link(&toolchain, &LinkRequest {
            sources: &[runtime.join("tests/embedding.c"), runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX))],
            include_dirs: &[runtime.join("include")], output: program.clone(),
            profile: Profile::Debug, obj_dir: dir.join("obj"),
        }).expect("the embedding oracle compiles");
        let output = run_fresh(&mut Command::new(program)).expect("the embedding oracle runs");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "the embedding oracle failed ({:?}): {stderr}", output.status.code());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok");
        assert!(!stderr.contains("reference cycle"), "the leak report reached standard error: {stderr}");
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
            profile: Profile::Development, obj_dir: dir.join("obj"),
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
            .args(gnu_flags(Profile::Development))
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
            command.args(gnu_flags(Profile::Release)).args(extra).arg("-mfma").arg("-S").arg(&source).arg("-o").arg(&output);
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
            let gnu = relaxed_gnu_flags(Profile::Release, fast);
            assert!(gnu.contains(&"-ffp-contract=fast") && !gnu.contains(&"-ffp-contract=off"), "{gnu:?}");
            assert_eq!(gnu.contains(&"-funsafe-math-optimizations"), fast, "{gnu:?}");
            assert_eq!(gnu.contains(&"-fno-fast-math"), !fast, "{gnu:?}");
            // Never the finite-only assumption: a NaN must stay a value.
            assert!(!gnu.iter().any(|flag| *flag == "-ffast-math" || flag.contains("finite-math")), "{gnu:?}");
            assert_eq!(gnu.len(), gnu_flags(Profile::Release).len() + usize::from(fast), "{gnu:?}");
            let msvc = relaxed_msvc_flags(Profile::Release, fast);
            assert_eq!(msvc.contains(&"/fp:fast"), fast, "{msvc:?}");
            assert_eq!(msvc.contains(&"/fp:contract"), !fast, "{msvc:?}");
            assert_eq!(msvc.contains(&"/fp:precise"), !fast, "{msvc:?}");
            assert!(!msvc.contains(&"/GL"), "{msvc:?}");
            let clang_cl = relaxed_clang_cl_flags(Profile::Release, fast);
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
        command.args(relaxed_gnu_flags(Profile::Release, false)).arg("-mfma").arg("-S").arg(&source).arg("-o").arg(&output);
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
