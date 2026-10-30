//! Annex B — what a reloadable build links: the one shared runtime of the process (`[HR-29]`), the
//! package's images (shared libraries the runtime loads at run time, `[HR-4]`), and the toolchain's
//! own host (`[HR-33]`), a program of one call that starts the first image.
//!
//! The runtime and every image are compiled with `EMBER_RELOAD`, and with `EMBER_RELOAD_INSTANCES`
//! when the build migrates instances (`[HR-12]`'s 40-byte header): the two must agree, which the
//! image's link against the runtime checks (`[HR-12a]`).
//!
//! On Windows the runtime's exports are every external symbol of its object with the runtime's
//! prefix, read from the object's COFF symbol table into a module-definition file: the runtime's
//! C has no export annotations, and a C program that links it statically must not get any.

use std::path::{Path, PathBuf};

use crate::{
    BuildError, Key, Link, Lto, Profile, Toolchain, clang_windows_runtime, compiler_command, gcc_flags, gnu_flags, lto_compile_flags,
    msvc_flags, run,
};

/// The runtime a reloadable program shares, as the build links it.
pub struct SharedRuntime {
    /// The library itself, beside the program (`ember_rt.dll`, `libember_rt.so`).
    pub library: PathBuf,
    /// What an image or the host links against: the import library on Windows, the library
    /// itself elsewhere.
    pub link_with: PathBuf,
}

/// The preprocessor definitions of a reloadable build.
/// The C definitions a reloadable build's units are compiled with.
pub fn reload_defines(instances: bool) -> Vec<String> {
    let upper = ember_branding::SYMBOL_PREFIX.to_uppercase();
    let mut defines = vec![format!("{upper}_RELOAD")];
    if instances {
        defines.push(format!("{upper}_RELOAD_INSTANCES"));
    }
    defines
}

/// The file name of a shared library: `stem.dll` on Windows, `libstem.so` elsewhere.
pub fn shared_library_name(stem: &str) -> String {
    if cfg!(windows) { format!("{stem}.dll") } else { format!("lib{stem}.so") }
}

/// The extension an image has on this platform.
pub fn image_extension() -> &'static str {
    if cfg!(windows) { "dll" } else { "so" }
}

/// Compile one C file of a reloadable build to an object: the profile's flags, the reload
/// definitions, and position-independent code where shared libraries need it.
fn compile_reload_object(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    output: &Path,
    profile: Profile,
    instances: bool,
    hidden: bool,
) -> Result<(), BuildError> {
    // `[BLD-6]` — no link-time optimisation for an image: its functions are reached through thunks,
    // and a reloadable one is never inlined (`[HR-9a]`), so there is nothing for it to do.
    compile_library_object(toolchain, source, include_dirs, output, profile, &reload_defines(instances), hidden, Lto::Off)
}

/// Compile one C file of a shared library to an object: the profile's flags, `defines`, and
/// position-independent code where shared libraries need it; `hidden` gives its symbols hidden
/// visibility off Windows; `lto` is the link-time optimisation of the link it goes into
/// (`[BLD-6]`, one the toolchain does).
#[allow(clippy::too_many_arguments)]
pub(crate) fn compile_library_object(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    output: &Path,
    profile: Profile,
    defines: &[String],
    hidden: bool,
    lto: Lto,
) -> Result<(), BuildError> {
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut command = compiler_command(toolchain);
    match toolchain {
        Toolchain::Msvc { .. } => {
            // `/Z7` keeps the debug information in the object: no PDB shared between builds.
            command.args(msvc_flags(profile).into_iter().map(|flag| if flag == "/Zi" { "/Z7" } else { flag }));
            command.args(lto_compile_flags(toolchain, lto, Link::Here));
            for define in defines {
                command.arg(format!("/D{define}"));
            }
            command.arg("/c");
            for dir in include_dirs {
                command.arg(format!("/I{}", dir.display()));
            }
            command.arg(source).arg(format!("/Fo{}", output.display()));
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.args(gnu_flags(profile)).args(gcc_flags(toolchain, profile)).args(lto_compile_flags(toolchain, lto, Link::Here));
            command.args(clang_windows_runtime(toolchain));
            command.args(crate::ir_flags(&[source]));
            if !cfg!(windows) {
                command.arg("-fPIC");
            }
            // An image exports nothing: the runtime finds its manifest by section ([HR-8]).
            if hidden && !cfg!(windows) {
                command.arg("-fvisibility=hidden");
            }
            for define in defines {
                command.arg(format!("-D{define}"));
            }
            command.arg("-c");
            for dir in include_dirs {
                command.arg("-I").arg(dir);
            }
            command.arg(source).arg("-o").arg(output);
        }
    }
    if let Err(error) = run(command) {
        let _ = std::fs::remove_file(output);
        return Err(error);
    }
    Ok(())
}

/// The shared runtime for `profile`, built once per toolchain, profile and header and kept in the
/// global cache, then copied beside the program when it differs from what is there. `reload` is
/// `Some(instances)` for a reloadable program's runtime, `None` for the plain one shared libraries
/// share (`[FFI-31c]`).
pub fn shared_runtime(
    toolchain: &Toolchain,
    runtime_source: &Path,
    include_dirs: &[PathBuf],
    profile: Profile,
    reload: Option<bool>,
    cache: &Path,
    bin: &Path,
) -> Result<SharedRuntime, BuildError> {
    if cfg!(windows) && matches!(toolchain, Toolchain::Gcc(_)) {
        return Err(BuildError::CompilerFailed {
            command: "gcc".to_string(),
            output: "a shared runtime on Windows needs MSVC, clang-cl or clang\n".to_string(),
        });
    }
    let mut key = Key::new();
    // v2: compiled without link-time optimisation in every profile (`[BLD-6]`); v3: under Clang on
    // Windows, with the DLL C runtime (`clang_windows_runtime`); v4: linked without the static one
    // (`clang_windows_link_runtime`).
    key.part(b"reload runtime v4");
    key.part(toolchain.name().as_bytes());
    key.part(profile.name().as_bytes());
    key.part(&[match reload { None => 2, Some(instances) => u8::from(instances) }]);
    key.part(&std::fs::read(runtime_source)?);
    for dir in include_dirs {
        let mut headers: Vec<PathBuf> = std::fs::read_dir(dir)?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_file())
            .collect();
        headers.sort();
        for header in headers {
            key.part(&std::fs::read(&header)?);
        }
    }
    key.compiler_file(toolchain);
    let stem = format!("{}_rt", ember_branding::SYMBOL_PREFIX);
    let built = cache.join("reload-runtime").join(key.finish());
    let library_in_cache = built.join(shared_library_name(&stem));
    let import_in_cache = built.join(format!("{stem}.lib"));
    if !library_in_cache.is_file() {
        let staging = built.with_extension(format!("tmp{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&staging);
        std::fs::create_dir_all(&staging)?;
        let object = staging.join(if matches!(toolchain, Toolchain::Msvc { .. }) { "rt.obj" } else { "rt.o" });
        let defines = reload.map(reload_defines).unwrap_or_default();
        // `[BLD-6]` — its exports are read from the object's symbols, which a link-time-optimised
        // object does not have (`coff_external_symbols`).
        compile_library_object(toolchain, runtime_source, include_dirs, &object, profile, &defines, false, Lto::Off)?;
        let library = staging.join(shared_library_name(&stem));
        let mut command = compiler_command(toolchain);
        match toolchain {
            Toolchain::Msvc { .. } => {
                let definitions = staging.join(format!("{stem}.def"));
                write_definitions(&object, &definitions, &stem)?;
                command.args(["/nologo", "/LD", crate::msvc_runtime_switch()]).arg(&object).arg(format!("/Fe{}", library.display()));
                command.arg("/link").arg(format!("/DEF:{}", definitions.display()))
                    .arg(format!("/IMPLIB:{}", staging.join(format!("{stem}.lib")).display()));
            }
            Toolchain::Clang(_) | Toolchain::Gcc(_) if cfg!(windows) => {
                let definitions = staging.join(format!("{stem}.def"));
                write_definitions(&object, &definitions, &stem)?;
                command.args(crate::clang_windows_link_runtime(toolchain));
                command.arg("-shared").arg(&object).arg("-o").arg(&library)
                    .arg(format!("-Wl,/DEF:{}", definitions.display()))
                    .arg(format!("-Wl,/IMPLIB:{}", staging.join(format!("{stem}.lib")).display()));
            }
            Toolchain::Clang(_) | Toolchain::Gcc(_) => {
                command.arg("-shared").arg(&object).arg("-o").arg(&library)
                    .arg(format!("-Wl,-soname,{}", shared_library_name(&stem)))
                    .args(["-lm", "-pthread", "-ldl"]);
            }
        }
        run(command)?;
        if std::fs::rename(&staging, &built).is_err() && !library_in_cache.is_file() {
            return Err(BuildError::Io(std::io::Error::other(format!(
                "cannot place the shared runtime in {}", built.display()
            ))));
        }
        let _ = std::fs::remove_dir_all(&staging);
    }
    std::fs::create_dir_all(bin)?;
    let library = bin.join(shared_library_name(&stem));
    let same = std::fs::read(&library).ok() == Some(std::fs::read(&library_in_cache)?);
    if !same {
        std::fs::copy(&library_in_cache, &library)?;
    }
    let link_with = if cfg!(windows) { import_in_cache } else { library.clone() };
    Ok(SharedRuntime { library, link_with })
}

/// Compile and link one image: the package's units (its C, or the LLVM backend's IR and the C
/// beside it) as a shared library that imports the runtime and exports nothing, keeping its
/// manifest (`manifest_symbol`) however the linker trims.
#[allow(clippy::too_many_arguments)]
pub fn link_image(
    toolchain: &Toolchain,
    sources: &[PathBuf],
    include_dirs: &[PathBuf],
    profile: Profile,
    instances: bool,
    runtime: &SharedRuntime,
    manifest_symbol: &str,
    relaxed: &[PathBuf],
    libraries: &[String],
    obj_dir: &Path,
    output: &Path,
) -> Result<(), BuildError> {
    let stem = output.file_stem().and_then(|stem| stem.to_str()).unwrap_or("image").to_string();
    let extension = if matches!(toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
    let mut objects = Vec::with_capacity(sources.len());
    for (index, source) in sources.iter().enumerate() {
        let object = crate::unit_object(obj_dir, &stem, index, extension);
        compile_reload_object(toolchain, source, include_dirs, &object, profile, instances, true)?;
        objects.push(object);
    }
    let mut command = compiler_command(toolchain);
    match toolchain {
        Toolchain::Msvc { .. } => {
            command.args(["/nologo", "/LD", crate::msvc_runtime_switch()]).args(&objects).args(relaxed).arg(&runtime.link_with)
                .args(libraries.iter().map(|library| crate::cdylib::windows_library(library)))
                .arg(format!("/Fe{}", output.display()))
                .arg("/link").arg(format!("/INCLUDE:{manifest_symbol}"));
            if profile != Profile::Release {
                command.arg("/DEBUG").arg(format!("/PDB:{}", output.with_extension("pdb").display()));
            }
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) if cfg!(windows) => {
            command.args(crate::clang_windows_link_runtime(toolchain));
            command.arg("-shared").args(&objects).args(relaxed).arg(&runtime.link_with).arg("-o").arg(output)
                .args(libraries.iter().map(|library| crate::cdylib::gnu_library(library)))
                .arg(format!("-Wl,/INCLUDE:{manifest_symbol}"));
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.arg("-shared").args(&objects).args(relaxed).arg(&runtime.link_with).arg("-o").arg(output)
                .args(libraries.iter().map(|library| crate::cdylib::gnu_library(library)))
                .args(["-lm", "-pthread"]);
        }
    }
    run(command)
}

/// The toolchain's host ([HR-33]): `main` hands the process to the runtime, which starts the
/// package's first image beside it.
pub fn link_host(
    toolchain: &Toolchain,
    package: &str,
    include_dirs: &[PathBuf],
    profile: Profile,
    runtime: &SharedRuntime,
    obj_dir: &Path,
    output: &Path,
) -> Result<(), BuildError> {
    let source = obj_dir.join(format!("{package}.host.c"));
    let header = ember_branding::runtime_header();
    let start = ember_branding::runtime("reload_host_main");
    let package_literal = package.replace('\\', "\\\\").replace('"', "\\\"");
    std::fs::write(
        &source,
        format!("/* Generated by the Ember compiler: the host of a reloadable program ([HR-33]). */\n#include \"{header}\"\n\nint main(int argc, char** argv) {{\n    return {start}(argc, argv, \"{package_literal}\");\n}}\n"),
    )?;
    link_host_source(toolchain, &source, include_dirs, profile, runtime, obj_dir, output)
}

/// A host of one's own: a C program linked against the shared runtime, which loads images with
/// `ember_reload_init` and calls their exports ([HR-30], [HR-21]).
pub fn link_host_source(
    toolchain: &Toolchain,
    source: &Path,
    include_dirs: &[PathBuf],
    profile: Profile,
    runtime: &SharedRuntime,
    obj_dir: &Path,
    output: &Path,
) -> Result<(), BuildError> {
    let stem = source.file_stem().and_then(|stem| stem.to_str()).unwrap_or("host");
    let object = obj_dir.join(format!("{stem}.{}", if matches!(toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" }));
    let mut command = compiler_command(toolchain);
    match toolchain {
        Toolchain::Msvc { .. } => {
            command.args(msvc_flags(profile).into_iter().map(|flag| if flag == "/Zi" { "/Z7" } else { flag }));
            for define in reload_defines(false) {
                command.arg(format!("/D{define}"));
            }
            for dir in include_dirs {
                command.arg(format!("/I{}", dir.display()));
            }
            // `/Fo` as a directory: clang-cl refuses an object file name when the command also
            // has the runtime's library to link (it counts two inputs); the object is `<stem>.obj`
            // there either way.
            let _ = &object;
            command.arg(source).arg(format!("/Fo{}{}", obj_dir.display(), std::path::MAIN_SEPARATOR))
                .arg(format!("/Fe{}", output.display()))
                .arg(&runtime.link_with);
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            command.args(gnu_flags(profile)).args(gcc_flags(toolchain, profile)).args(crate::clang_windows_link_runtime(toolchain));
            for define in reload_defines(false) {
                command.arg(format!("-D{define}"));
            }
            for dir in include_dirs {
                command.arg("-I").arg(dir);
            }
            command.arg(source).arg(&runtime.link_with).arg("-o").arg(output);
            if !cfg!(windows) {
                command.arg("-Wl,-rpath,$ORIGIN").args(["-ldl", "-pthread"]);
            }
        }
    }
    run(command)
}

/// The module-definition file exporting every external symbol of `object` that carries the
/// runtime's prefix: functions by name, data with `DATA`.
fn write_definitions(object: &Path, output: &Path, library: &str) -> Result<(), BuildError> {
    let bytes = std::fs::read(object)?;
    let symbols = coff_external_symbols(&bytes).ok_or_else(|| {
        BuildError::Io(std::io::Error::other(format!("cannot read the symbols of {}", object.display())))
    })?;
    let mut text = format!("LIBRARY {library}\nEXPORTS\n");
    for (name, function) in symbols {
        if name.starts_with(ember_branding::RUNTIME_PREFIX) {
            text.push_str(&format!("    {name}{}\n", if function { "" } else { " DATA" }));
        }
    }
    std::fs::write(output, text)?;
    Ok(())
}

/// The symbols a COFF object defines with external linkage, each with whether it is a function:
/// the plain format and `/bigobj`'s. `None` for anything else (an LTCG object, say).
pub fn coff_external_symbols(bytes: &[u8]) -> Option<Vec<(String, bool)>> {
    let u16_at = |at: usize| bytes.get(at..at + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let u32_at = |at: usize| bytes.get(at..at + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    // `/bigobj`: sig1 0, sig2 0xFFFF, version 2; 32-bit section numbers in 20-byte records.
    let big = u16_at(0)? == 0 && u16_at(2)? == 0xFFFF && u16_at(4)? >= 2;
    let (table, count, record) =
        if big { (u32_at(48)? as usize, u32_at(52)? as usize, 20) } else { (u32_at(8)? as usize, u32_at(12)? as usize, 18) };
    let strings = table.checked_add(count.checked_mul(record)?)?;
    let mut out = Vec::new();
    let mut index = 0;
    while index < count {
        let at = table + index * record;
        let raw = bytes.get(at..at + 8)?;
        let (section, kind, class, aux) = if big {
            (u32_at(at + 12)? as i32, u16_at(at + 16)?, *bytes.get(at + 18)?, *bytes.get(at + 19)?)
        } else {
            (i32::from(u16_at(at + 12)? as i16), u16_at(at + 14)?, *bytes.get(at + 16)?, *bytes.get(at + 17)?)
        };
        // IMAGE_SYM_CLASS_EXTERNAL, defined in a section of this object.
        if class == 2 && section > 0 {
            let name = if raw[..4] == [0, 0, 0, 0] {
                let offset = strings + u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]) as usize;
                let end = bytes.get(offset..)?.iter().position(|&b| b == 0)?;
                String::from_utf8_lossy(&bytes[offset..offset + end]).into_owned()
            } else {
                let end = raw.iter().position(|&b| b == 0).unwrap_or(8);
                String::from_utf8_lossy(&raw[..end]).into_owned()
            };
            // IMAGE_SYM_DTYPE_FUNCTION in the type's high nibble.
            out.push((name, kind & 0x30 == 0x20));
        }
        index += 1 + aux as usize;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The runtime's hand-written test images (`templates/tests/reload_image_*.c.in`) built as a
    /// reloadable program in `dir`: the shared runtime, `t.image.0` to `t.image.2` and the host.
    fn build_test_program(dir: &Path) -> PathBuf {
        let requested = std::env::var(ember_branding::cc_var()).ok();
        let toolchain = Toolchain::detect(requested.as_deref()).expect("a C toolchain");
        let runtime = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime")
            .join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
        let includes = vec![runtime.join("include")];
        let source = runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX));
        let obj = dir.join("obj");
        std::fs::create_dir_all(&obj).expect("the test directory is creatable");
        let shared = shared_runtime(&toolchain, &source, &includes, Profile::Debug, Some(true), &crate::cache_root(), dir)
            .expect("the shared runtime builds");
        let manifest = format!("{}test_manifest", ember_branding::RUNTIME_PREFIX);
        for (index, image) in ["a", "b", "c"].iter().enumerate() {
            link_image(
                &toolchain,
                &[runtime.join("tests").join(format!("reload_image_{image}.c"))],
                &includes,
                Profile::Debug,
                true,
                &shared,
                &manifest,
                &[],
                &[],
                &obj,
                &dir.join(format!("t.image.{index}.{}", image_extension())),
            )
            .expect("a test image builds");
        }
        let host = dir.join(if cfg!(windows) { "t.exe" } else { "t" });
        link_host(&toolchain, "t", &includes, Profile::Debug, &shared, &obj, &host).expect("the host builds");
        host
    }

    /// Windows: no error box for a program this test starts. SEM_FAILCRITICALERRORS,
    /// SEM_NOGPFAULTERRORBOX and SEM_NOOPENFILEERRORBOX, which every child inherits: a host that
    /// crashes, or cannot start (a missing library, 0xc0000142), ends with its status instead of
    /// opening a window that waits for a click (D-252).
    #[cfg(windows)]
    #[allow(unsafe_code)]
    fn quiet_errors() {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn SetErrorMode(mode: u32) -> u32;
        }
        // SAFETY: SetErrorMode only sets this process's error mode flags.
        unsafe {
            SetErrorMode(0x0001 | 0x0002 | 0x8000);
        }
    }

    #[cfg(not(windows))]
    fn quiet_errors() {}

    fn run(host: &Path, mode: &str) -> std::process::Output {
        quiet_errors();
        let mut command = std::process::Command::new(host);
        command.arg(mode);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }
        crate::output_started(&mut command).expect("the reload host runs")
    }

    /// `[HR-42a]` — the depth counter is not a lock: entering Ember code on a registered thread is
    /// one relaxed increment of the thread's own counter and one acquire fence, and leaving is one
    /// release decrement; nothing in either waits, takes a lock or looks at another thread.
    #[test]
    fn entering_ember_code_is_one_increment_and_one_fence() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../runtime")
            .join(format!("{}_rt", ember_branding::SYMBOL_PREFIX))
            .join("src")
            .join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX));
        let text = std::fs::read_to_string(source).expect("the runtime source is readable").replace("\r\n", "\n");
        let body = |name: &str| -> String {
            let start = text.find(&format!("void {}{name}(void) {{\n", ember_branding::RUNTIME_PREFIX)).expect("the function is defined");
            let end = start + text[start..].find("\n}\n").expect("the function ends");
            text[start..end].to_string()
        };
        let enter = body("reload_enter");
        // The registration branch runs once per thread, on its first entry ([HR-3a]).
        let fast = &enter[enter.find("    RL_ADD_RELAXED").expect("one relaxed increment")..];
        assert_eq!(fast.trim(), "RL_ADD_RELAXED(&self->depth, 1u);\n    RL_FENCE_ACQUIRE();", "{enter}");
        let leave = body("reload_leave");
        assert!(leave.ends_with("    RL_SUB_RELEASE(&g_rl_self->depth, 1u);"), "{leave}");
        for word in ["lock", "wait", "Sleep", "mutex", "for ("] {
            assert!(!enter.contains(word) && !leave.contains(word), "`{word}` in:\n{enter}\n{leave}");
        }
    }

    /// The runtime's reload protocol, driven from C (`reload_image_a.c.in` says each step):
    /// `[ABI-2]` an image of another protocol is refused on load with `E9037`; `[HR-3]` a poll while
    /// a registered thread is inside Ember code is `EMBER_RELOAD_UNSAFE_POINT`, read from that
    /// thread's depth counter (`[HR-42]`); `[HR-2]`, `[HR-6]` the next poll applies the new image and
    /// the same thunk runs its body; `[HR-4a]`, `[HR-17]` the static keeps its value in the
    /// runtime's storage and the new body reads it; `[HR-4]` the old image's code still runs;
    /// `[HR-30]`, `[HR-32]` the host contract's stats count the reload and carry the build's time;
    /// `[HR-7]` a removed function's thunk reaches a stub that panics naming it.
    #[test]
    fn the_reload_protocol_from_a_c_host() {
        let dir = std::env::temp_dir().join(format!("{}-reload-protocol-{}", ember_branding::CLI_NAME, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("the test directory is creatable");
        let host = build_test_program(&dir);
        let output = run(&host, "protocol");
        let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            stdout,
            "before 2 after 143 own 2 counter 42\nrefused 1 named 1 unsafe 1 applied 1 idle 1\nreloads 1 compile 1000 load 1\n",
            "{stderr}"
        );
        assert!(!output.status.success(), "the removed function returned: {stderr}");
        assert!(stderr.contains("`gone` was removed by a hot reload and is called"), "{stderr}");

        // `[HR-3a]` — a debug build asserts on entry to Ember code from an unregistered thread.
        let output = run(&host, "unregistered");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "an unregistered thread entered Ember code: {stderr}");
        assert!(stderr.contains("a thread the runtime has not registered"), "{stderr}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `[HR-29]` — the shared runtime's exports are read from its object: a COFF symbol table with
    /// one external function, one external datum and one static, each in the 18-byte record.
    #[test]
    fn coff_symbols_name_external_functions_and_data() {
        let mut object = vec![0u8; 20];
        let table = 20u32;
        object[8..12].copy_from_slice(&table.to_le_bytes());
        object[12..16].copy_from_slice(&3u32.to_le_bytes());
        let go = format!("{}go", ember_branding::RUNTIME_PREFIX);
        let datum = format!("{}n", ember_branding::RUNTIME_PREFIX);
        let mut record = |name: &str, section: i16, kind: u16, class: u8| {
            let mut entry = [0u8; 18];
            entry[..name.len()].copy_from_slice(name.as_bytes());
            entry[12..14].copy_from_slice(&section.to_le_bytes());
            entry[14..16].copy_from_slice(&kind.to_le_bytes());
            entry[16] = class;
            object.extend_from_slice(&entry);
        };
        record(&go, 1, 0x20, 2);
        record(&datum, 2, 0, 2);
        record("helper", 1, 0x20, 3);
        object.extend_from_slice(&4u32.to_le_bytes());
        let symbols = coff_external_symbols(&object).expect("a COFF object");
        assert_eq!(symbols, vec![(go, true), (datum, false)]);
    }
}
