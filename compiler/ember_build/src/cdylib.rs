//! `[FFI-28]`, `[FFI-31]`, `[FFI-31c]` — a package built as `kind = "cdylib"`: a shared library
//! exporting only its `@export` symbols and `ember_module_init`/`ember_module_shutdown`.
//!
//! With `[build] runtime = "static"` the library carries its own copy of the runtime; with
//! `"shared"` it imports the one shared runtime of the process (the plain build of
//! [`crate::reload::shared_runtime`]), so several Ember modules share one runtime.
//!
//! What is exported is decided by a list, never by annotations in the C: a module-definition file
//! on Windows, a version script elsewhere, which also keeps the private runtime's own symbols
//! local. `-Bsymbolic` binds the library's calls of its own exports to itself, so a host that loads
//! two Ember libraries never sends one's call into the other.

use std::path::{Path, PathBuf};

use crate::reload::{SharedRuntime, compile_library_object, shared_library_name};
use crate::{BuildError, Profile, Toolchain, compiler_command, run};

/// What a `cdylib` build writes.
pub struct Cdylib {
    /// The library itself (`name.dll`, `libname.so`).
    pub library: PathBuf,
    /// What a C host links against at build time: the import library on Windows, the library
    /// itself elsewhere.
    pub link_with: PathBuf,
}

/// Compile `source` (the package's C) and, without `shared`, the runtime into one shared library
/// in `lib`, exporting exactly `exports`; `libraries` are linked after them, in order
/// (`[BLD-FFI-3]`).
#[allow(clippy::too_many_arguments)]
pub fn link_cdylib(
    toolchain: &Toolchain,
    name: &str,
    source: &Path,
    runtime_source: &Path,
    include_dirs: &[PathBuf],
    profile: Profile,
    shared: Option<&SharedRuntime>,
    exports: &[String],
    libraries: &[String],
    obj_dir: &Path,
    lib: &Path,
) -> Result<Cdylib, BuildError> {
    if cfg!(windows) && matches!(toolchain, Toolchain::Gcc(_)) {
        return Err(BuildError::CompilerFailed {
            command: "gcc".to_string(),
            output: "a `cdylib` on Windows needs MSVC, clang-cl or clang\n".to_string(),
        });
    }
    std::fs::create_dir_all(obj_dir)?;
    std::fs::create_dir_all(lib)?;
    let ext = if matches!(toolchain, Toolchain::Msvc { .. }) { "obj" } else { "o" };
    let mut objects = vec![obj_dir.join(format!("{name}.{ext}"))];
    compile_library_object(toolchain, source, include_dirs, &objects[0], profile, &[], false)?;
    if shared.is_none() {
        let runtime = obj_dir.join(format!("{}_rt.module.{ext}", ember_branding::SYMBOL_PREFIX));
        compile_library_object(toolchain, runtime_source, include_dirs, &runtime, profile, &[], false)?;
        objects.push(runtime);
    }
    let library = lib.join(shared_library_name(name));
    let import = lib.join(format!("{name}.lib"));
    for stale in [&library, &import] {
        if stale.is_file() {
            std::fs::remove_file(stale)?;
        }
    }
    let mut command = compiler_command(toolchain);
    match toolchain {
        Toolchain::Msvc { .. } => {
            let definitions = obj_dir.join(format!("{name}.def"));
            write_definitions(&definitions, name, exports)?;
            command.args(["/nologo", "/LD", "/MD"]).args(&objects);
            if let Some(shared) = shared {
                command.arg(&shared.link_with);
            }
            command.arg(format!("/Fe{}", library.display())).arg("/link")
                .arg(format!("/DEF:{}", definitions.display()))
                .arg(format!("/IMPLIB:{}", import.display()));
            if profile != Profile::Shipping {
                command.arg("/DEBUG").arg(format!("/PDB:{}", library.with_extension("pdb").display()));
            }
            for library in libraries {
                command.arg(windows_library(library));
            }
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) if cfg!(windows) => {
            let definitions = obj_dir.join(format!("{name}.def"));
            write_definitions(&definitions, name, exports)?;
            command.arg("-shared").args(&objects);
            if let Some(shared) = shared {
                command.arg(&shared.link_with);
            }
            command.arg("-o").arg(&library)
                .arg(format!("-Wl,/DEF:{}", definitions.display()))
                .arg(format!("-Wl,/IMPLIB:{}", import.display()));
            for library in libraries {
                command.arg(gnu_library(library));
            }
        }
        Toolchain::Clang(_) | Toolchain::Gcc(_) => {
            let script = obj_dir.join(format!("{name}.map"));
            let mut text = String::from("{\n  global:\n");
            for symbol in exports {
                text.push_str(&format!("    {symbol};\n"));
            }
            text.push_str("  local: *;\n};\n");
            std::fs::write(&script, text)?;
            command.arg("-shared").args(&objects);
            if let Some(shared) = shared {
                command.arg(&shared.link_with).arg("-Wl,-rpath,$ORIGIN");
            }
            command.arg("-o").arg(&library)
                .arg(format!("-Wl,--version-script={}", script.display()))
                .arg("-Wl,-Bsymbolic")
                .arg(format!("-Wl,-soname,{}", shared_library_name(name)));
            for library in libraries {
                command.arg(gnu_library(library));
            }
            command.args(["-lm", "-pthread"]);
        }
    }
    run(command)?;
    let link_with = if cfg!(windows) { import } else { library.clone() };
    Ok(Cdylib { library, link_with })
}

/// The module-definition file: the library's name and each export, by name.
fn write_definitions(file: &Path, name: &str, exports: &[String]) -> Result<(), BuildError> {
    let mut text = format!("LIBRARY {name}\nEXPORTS\n");
    for symbol in exports {
        text.push_str(&format!("    {symbol}\n"));
    }
    std::fs::write(file, text)?;
    Ok(())
}

/// `[BLD-FFI-3]` — a library as MSVC's linker takes it: a path as written, a name as `name.lib`.
pub fn windows_library(library: &str) -> String {
    if is_library_path(library) { library.to_string() } else { format!("{library}.lib") }
}

/// `[BLD-FFI-3]` — a library as a GNU-style driver takes it: a path as written, a name as `-lname`.
pub fn gnu_library(library: &str) -> String {
    if is_library_path(library) { library.to_string() } else { format!("-l{library}") }
}

/// Whether `[link] libs` names a file rather than a library to search for: it has a directory or
/// a library file's extension.
pub fn is_library_path(library: &str) -> bool {
    library.contains('/') || library.contains('\\')
        || [".a", ".so", ".lib", ".dylib", ".dll"].iter().any(|ext| library.ends_with(ext))
}
