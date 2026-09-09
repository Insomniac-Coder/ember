//! The one place the project's names live (`[RT-5]`, `[MAN-4]`, `[MNG-5]`).
//!
//! `[RT-5]`: "No file in the compiler, runtime, CMake module, examples or test
//! corpus may hard-code the symbol prefix, the CLI name, the manifest file
//! name, or the source and binding-cache extensions; each is read from a
//! single `branding` module." This is that module, and
//! `tools/check_branding.py` fails CI on any hard-coded occurrence elsewhere.
//!
//! The point is not that the names are expected to change. It is that a name
//! spelled in ninety places cannot be *audited*: a rename becomes a
//! search-and-replace across generated C, CMake, the test harness and the
//! diagnostics, and the one occurrence that is missed produces a link error
//! months later with nothing pointing at its cause.
//!
//! The runtime keeps the same constant in exactly one place of its own
//! (`EMBER_SYMBOL_PREFIX` in `ember_rt.h`), because the runtime is C and
//! cannot read this crate. Those two definitions must agree; the C header is
//! the interface document embedders read, so it carries literal identifiers
//! rather than macro concatenations, exactly as `[RT-5]` requires.

/// The project's name, written exactly once in the whole workspace.
///
/// A macro rather than a `const` so that `RUNTIME_PREFIX` can be built from it
/// at compile time with `concat!`, which keeps it usable in a format string's
/// implicit arguments.
#[macro_export]
macro_rules! symbol_prefix {
    () => {
        "ember"
    };
}

/// `EMBER_SYMBOL_PREFIX`. Every runtime entry point begins with this and an
/// underscore: `ember_alloc`, `ember_panic_bounds`, `ember_retain`.
pub const SYMBOL_PREFIX: &str = symbol_prefix!();

/// `SYMBOL_PREFIX` with its separator, for the emitted C. A `const` so the
/// backend can write `format!("{RT}vec_push(…)")` and keep the generated call
/// legible next to the C it sits in.
pub const RUNTIME_PREFIX: &str = concat!(symbol_prefix!(), "_");

/// The CLI's name, used in diagnostics and in what the driver prints.
pub const CLI_NAME: &str = "ember";

/// `[MAN-1]` — a package is a directory tree with this at its root.
pub const MANIFEST: &str = "ember.toml";

/// `[LEX-3]` — the source extension, without the dot. No other extension is a
/// compilation unit.
pub const SOURCE_EXT: &str = "em";

/// `[FFI-*]` — the content-addressed binding cache, without the dot.
pub const BINDING_EXT: &str = "embind";

/// The header the emitted C includes.
pub fn runtime_header() -> String {
    format!("{SYMBOL_PREFIX}_rt.h")
}

/// `[MNG-5]` — "the mangled prefix `em_` … is derived from
/// `EMBER_SYMBOL_PREFIX` and constructed in exactly one function". This is
/// that function: the first two characters of the prefix, then an underscore.
/// Short, because it is on every symbol in the emitted C, and a reader of that
/// C should be able to tell a generated name from a runtime entry point at a
/// glance — `em_game_update` against `ember_alloc`.
pub fn mangle_prefix() -> String {
    let short: String = SYMBOL_PREFIX.chars().take(2).collect();
    format!("{short}_")
}

/// A mangled symbol for an Ember item: `em_std_math_Vec3_length`.
/// `[MNG-3]` — the caller has already transliterated the name to ASCII.
pub fn mangled(name: &str) -> String {
    format!("{}{}", mangle_prefix(), name.replace('.', "_"))
}

/// A runtime entry point: `runtime("alloc")` is `ember_alloc`.
pub fn runtime(symbol: &str) -> String {
    format!("{SYMBOL_PREFIX}_{symbol}")
}

/// `[MNG-4]` — class object structs, vtables and type infos.
pub fn object_struct(class: &str) -> String {
    format!("{}obj_{}", mangle_prefix(), class.replace('.', "_"))
}

pub fn vtable(name: &str) -> String {
    format!("{}vt_{}", mangle_prefix(), name.replace('.', "_"))
}

pub fn type_info(name: &str) -> String {
    format!("{}ti_{}", mangle_prefix(), name.replace('.', "_"))
}

/// `hello.em`, for a message that names a file the user should create.
pub fn source_file(stem: &str) -> String {
    format!("{stem}.{SOURCE_EXT}")
}

/// `[MOD-5]`, `[STD-1]` — the standard library's package name.
pub const STD_PACKAGE: &str = "std";

/// The environment variable that overrides where `std`'s sources are found.
/// `[TOOL-1]`'s toolchain archive ships them, so an installed compiler finds
/// them beside itself; this is for a compiler run out of its build tree.
pub fn std_path_var() -> String {
    format!("{}_STD", SYMBOL_PREFIX.to_uppercase())
}

/// Where `std`'s sources live, or `None` where they cannot be found.
///
/// The order is: the environment variable; then `std/src` beside the running
/// executable, which is where `[TOOL-1]`'s archive puts it; then `std/src`
/// found by walking up from the executable and from the working directory,
/// which is what makes a compiler run out of `target/debug/` work.
pub fn std_root() -> Option<std::path::PathBuf> {
    if let Ok(path) = std::env::var(std_path_var()) {
        let path = std::path::PathBuf::from(path);
        if path.is_dir() {
            return Some(path);
        }
    }
    let mut starts: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            starts.push(dir.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        starts.push(cwd);
    }
    for start in starts {
        let mut dir = start.as_path();
        loop {
            let candidate = dir.join(STD_PACKAGE).join("src");
            if candidate.is_dir() {
                return Some(candidate);
            }
            match dir.parent() {
                Some(parent) => dir = parent,
                None => break,
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_std_path_variable_is_derived_too() {
        assert_eq!(std_path_var(), "EMBER_STD");
    }

    #[test]
    fn the_mangled_prefix_is_derived_not_written() {
        // `[MNG-5]`. If `SYMBOL_PREFIX` ever changes, this changes with it,
        // which is the whole point of deriving it.
        assert_eq!(mangle_prefix(), "em_");
        assert_eq!(mangled("std.math.Vec3_length"), "em_std_math_Vec3_length");
        assert_eq!(runtime("alloc"), "ember_alloc");
        assert_eq!(runtime_header(), "ember_rt.h");
    }

    #[test]
    fn a_runtime_symbol_is_distinguishable_from_a_generated_one() {
        // The reason the mangled prefix is short: a reader of the emitted C
        // tells the two apart without a lookup.
        assert!(runtime("retain").starts_with(SYMBOL_PREFIX));
        assert!(!mangled("retain").starts_with(&format!("{SYMBOL_PREFIX}_")));
    }
}
