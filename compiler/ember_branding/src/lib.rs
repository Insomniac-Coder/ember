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

/// `[MAN-2]` — the resolved dependencies with content hashes, beside the manifest.
pub const LOCK_FILE: &str = "ember.lock";

/// `[LEX-3]` — the source extension, without the dot. No other extension is a
/// compilation unit.
pub const SOURCE_EXT: &str = "em";

/// `[FFI-*]` — the content-addressed binding cache, without the dot.
pub const BINDING_EXT: &str = "embind";

/// `[ALC-3]` — the Ember names of the two functions a binary package's global allocator is
/// reached through, which the driver writes into the module declaring it and the entry point gives
/// `ember_rt_init` as its allocation functions.
pub fn global_allocator_glue() -> (String, String) {
    (format!("__{SYMBOL_PREFIX}_global_allocate"), format!("__{SYMBOL_PREFIX}_global_deallocate"))
}

/// The header the emitted C includes.
pub fn runtime_header() -> String {
    format!("{SYMBOL_PREFIX}_rt.h")
}

/// `[MNG-5]` — "the mangled prefix `em_` … is derived from
/// `EMBER_SYMBOL_PREFIX` and constructed in exactly one function". This is
/// that function: the first two characters of the prefix, then an underscore.
/// Short, because it is on every symbol in the emitted C, and a reader of that
/// C should be able to tell a generated name from a runtime entry point at a
/// glance — `em_3game6update` against `ember_alloc`.
pub fn mangle_prefix() -> String {
    prefix().to_string()
}

/// [`mangle_prefix`], made once: every symbol is built and read with it.
fn prefix() -> &'static str {
    static PREFIX: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        let short: String = SYMBOL_PREFIX.chars().take(2).collect();
        format!("{short}_")
    });
    &PREFIX
}

/// `[MNG-1]` — how many hex digits of the BLAKE3 hash of a generic instance's arguments its
/// symbol keeps.
pub const INSTANCE_HASH_DIGITS: usize = 16;

/// `[MNG-1]` — the letter between an instance's path and its hash: `em_3maxG…`. A component
/// begins with a digit or [`TRANSLITERATED_MARK`] and one, so the letter ends the path.
pub const INSTANCE_MARK: char = 'G';

/// `[MNG-3]` (ODR-202) — the letter before the length of a component that was transliterated:
/// `café` is `u10caf_u00E9_`, the ASCII name `caf_u00E9_` is `10caf_u00E9_`, so the two never meet.
pub const TRANSLITERATED_MARK: char = 'u';

/// `[MNG-1]` — an Ember path (`lib.m_x.f`) as a symbol writes it after `em_`: each component as its
/// length in decimal and then its text (`3lib3m_x1f`), a character beyond ASCII transliterated
/// first and the length marked so (`[MNG-3]`, `u10caf_u00E9_`). The compiler spells a path with `.` between its components, which no
/// identifier holds, so `.` is exactly where one component ends; the length prefixes then make the
/// symbol name each component exactly too, where `_` between them did not (`m_x.f` and `m.x_f`).
pub fn mangled_path(path: &str) -> String {
    let mut out = String::with_capacity(path.len() + 8);
    push_mangled_path(&mut out, path);
    out
}

/// [`mangled_path`], written onto `out`.
pub fn push_mangled_path(out: &mut String, path: &str) {
    for component in path.split('.') {
        push_component(out, component);
    }
}

/// One path component: its length, then its text, transliterated (`[MNG-3]`) before the length
/// is taken; a transliterated one has [`TRANSLITERATED_MARK`] before its length (ODR-202).
fn push_component(out: &mut String, text: &str) {
    use std::fmt::Write;
    debug_assert!(!text.is_empty(), "an empty path component");
    let written = transliterated(text);
    // `[MNG-3]` (ODR-213) — marked whenever transliteration wrote anything: a character beyond
    // ASCII, or an ASCII `_` that would read as an escape.
    if matches!(written, std::borrow::Cow::Owned(_)) {
        out.push(TRANSLITERATED_MARK);
    }
    let _ = write!(out, "{}{written}", written.len());
}

/// Whether a component begins `rest`: a length (a digit other than `0`), marked or not.
fn starts_component(rest: &str) -> bool {
    rest.strip_prefix(TRANSLITERATED_MARK).unwrap_or(rest).starts_with(|c: char| matches!(c, '1'..='9'))
}

/// The component that begins `rest` ([`starts_component`]): whether it was transliterated, its text
/// as written, and what follows it; `None` when its length runs past the end.
fn split_component(rest: &str) -> Option<(bool, &str, &str)> {
    let body = rest.strip_prefix(TRANSLITERATED_MARK);
    let marked = body.is_some();
    let body = body.unwrap_or(rest);
    let digits = body.find(|c: char| !c.is_ascii_digit()).unwrap_or(body.len());
    let length: usize = body[..digits].parse().ok()?;
    let text = body.get(digits..digits.checked_add(length)?)?;
    Some((marked, text, &body[digits + length..]))
}

/// `[MNG-1]` — the end of a generic instance's symbol: `G` and the first 16 hex digits of the
/// BLAKE3 hash of the canonical spelling of its arguments.
pub fn instance_suffix(arguments: &str) -> String {
    let hash = blake3::hash(arguments.as_bytes());
    let mut out = String::with_capacity(1 + INSTANCE_HASH_DIGITS);
    out.push(INSTANCE_MARK);
    out.push_str(&hash.to_hex()[..INSTANCE_HASH_DIGITS]);
    out
}

/// `[MNG-1]` — an item's symbol: `em_` and its path, mangled (`em_3lib3m_x1f`).
pub fn item_symbol(path: &str) -> String {
    let mut out = prefix().to_string();
    push_mangled_path(&mut out, path);
    out
}

/// `[FN-8]` — the root module's `main`, which the C entry point calls: `em_4main`.
pub fn entry_symbol() -> String {
    item_symbol("main")
}

/// `[MNG-1]` — a generic instance's symbol: its item's, then [`instance_suffix`] of `arguments`
/// (the canonical spelling of its arguments); with none, the item's own.
pub fn instance_symbol(path: &str, arguments: Option<&str>) -> String {
    let mut out = item_symbol(path);
    if let Some(arguments) = arguments {
        out.push_str(&instance_suffix(arguments));
    }
    out
}

/// `[MNG-1]`, `[CLO-3]` — a further instance of the item `symbol` names, over more `arguments`
/// (their canonical spelling): its path, then the hash of its own instance's hash (when it is one)
/// and `arguments`. `[CLO-3]`'s copy of a function for a function passed to it is such an
/// instance. A name the compiler made for itself (a lambda's, `em_closure3`) is no item's: it
/// keeps itself, then `_` and the hash.
pub fn reinstance(symbol: &str, arguments: &str) -> String {
    let Some((path, instance)) = item_parts(symbol) else {
        return format!("{symbol}_{}", instance_suffix(arguments));
    };
    let spelling = match instance {
        Some(hash) => format!("{INSTANCE_MARK}{hash}{arguments}"),
        None => arguments.to_string(),
    };
    format!("{path}{}", instance_suffix(&spelling))
}

/// An item symbol's prefix and path (`em_5apply`) and its instance hash, when that is all it is.
fn item_parts(symbol: &str) -> Option<(&str, Option<&str>)> {
    let mut rest = symbol.strip_prefix(prefix())?;
    let mut components = 0;
    while starts_component(rest) {
        (_, _, rest) = split_component(rest)?;
        components += 1;
    }
    let path = &symbol[..symbol.len() - rest.len()];
    match rest.strip_prefix(INSTANCE_MARK) {
        _ if components == 0 => None,
        None if rest.is_empty() => Some((path, None)),
        Some(hash) if hash.len() == INSTANCE_HASH_DIGITS && hash.bytes().all(|b| b.is_ascii_hexdigit()) => Some((path, Some(hash))),
        _ => None,
    }
}

/// A symbol whose mangled form (path and instance hash) was made elsewhere: `em_` and it. A type's
/// is its type table's (`ember_types::TypeTable::mangled`).
pub fn symbol(mangled: &str) -> String {
    format!("{}{mangled}", prefix())
}

/// A name the compiler makes for itself, not for an item: `em_` and `tag`, which begins with a
/// letter and is not the start of a component (`em_closure3`, `em_drop_glue_2`), so it is never an
/// item's (whose path begins with a digit, or `u` and one). A name keyed by types or items writes
/// their mangled forms after the tag, each after `_` (`em_vt_dyn_7Observe_4Bump`): a mangled form
/// begins with a component and its components' lengths say where it ends, so a `_` followed by a
/// component starts the next one, and one followed by any other letter starts a tag of the name's
/// own (`_slot0`).
pub fn generated(tag: &str) -> String {
    debug_assert!(
        tag.starts_with(|c: char| c.is_ascii_alphabetic()) && !starts_component(tag),
        "a generated name's tag begins with a letter and no component: `{tag}`"
    );
    format!("{}{tag}", prefix())
}

/// `[MNG-3]` — whether `rest`, which begins with an ASCII `_`, reads as an escape: `_u`, four or
/// more hex digits, and `_`.
fn reads_as_escape(rest: &str) -> bool {
    let Some(after) = rest.strip_prefix("_u") else { return false };
    let digits = after.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(after.len());
    digits >= 4 && after[digits..].starts_with('_')
}

/// `[MNG-3]` — a name with each character beyond ASCII written `_uXXXX_`, its code point in hex
/// (four digits or more): `café` is `caf_u00E9_`. ODR-213 — and an ASCII `_` that begins what would
/// otherwise read as an escape written `_u005F_`, so no two names share a spelling: the ASCII name
/// `caf_u00E9_` is `caf_u005F_u00E9_`. A name with neither is itself.
pub fn transliterated(name: &str) -> std::borrow::Cow<'_, str> {
    let escaped_underscore = |at: usize| name.as_bytes()[at] == b'_' && reads_as_escape(&name[at..]);
    if name.is_ascii() && !(0..name.len()).any(escaped_underscore) {
        return std::borrow::Cow::Borrowed(name);
    }
    let mut out = String::with_capacity(name.len() + 8);
    for (at, c) in name.char_indices() {
        if !c.is_ascii() {
            out.push_str(&format!("_u{:04X}_", c as u32));
        } else if c == '_' && reads_as_escape(&name[at..]) {
            out.push_str("_u005F_");
        } else {
            out.push(c);
        }
    }
    std::borrow::Cow::Owned(out)
}

/// `[MNG-3]` — an Ember name as C spells it where it is not a symbol (a field, a local): each
/// character beyond ASCII transliterated, and each `.` an `_`.
pub fn c_identifier(name: &str) -> String {
    transliterated(name).replace('.', "_")
}

/// `[CG-C-10]` — an item symbol read back: its path's components (each transliteration undone)
/// and, for a generic instance, its hash; `rest` is whatever follows (`__rl` on a reloadable
/// function's body). `None` for a name that is not an item's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Demangled<'a> {
    pub components: Vec<String>,
    pub instance: Option<&'a str>,
    pub rest: &'a str,
}

impl Demangled<'_> {
    /// The Ember path: `lib.m_x.f`.
    pub fn path(&self) -> String {
        self.components.join(".")
    }
}

/// [`Demangled`] — the length prefixes say exactly where each component ends, and the mark before a
/// length whether its text is transliterated (`u10caf_u00E9_` is `café`, `10caf_u00E9_` the ASCII
/// name `caf_u00E9_`).
pub fn demangle(symbol: &str) -> Option<Demangled<'_>> {
    let mut rest = symbol.strip_prefix(prefix())?;
    let mut components = Vec::new();
    while starts_component(rest) {
        let (marked, text, after) = split_component(rest)?;
        components.push(if marked { untransliterated(text)? } else { text.to_string() });
        rest = after;
    }
    if components.is_empty() {
        return None;
    }
    let mut instance = None;
    if let Some(hash) = rest.strip_prefix(INSTANCE_MARK)
        && hash.len() >= INSTANCE_HASH_DIGITS
        && hash.as_bytes()[..INSTANCE_HASH_DIGITS].iter().all(u8::is_ascii_hexdigit)
    {
        instance = Some(&hash[..INSTANCE_HASH_DIGITS]);
        rest = &hash[INSTANCE_HASH_DIGITS..];
    }
    Some(Demangled { components, instance, rest })
}

/// Whether `symbol` is an item's whose path begins with `components` (`["std"]` for
/// the standard library's items), and goes on past them. ASCII components only; nothing is
/// allocated, as the passes ask it of every body.
pub fn symbol_path_starts_with(symbol: &str, components: &[&str]) -> bool {
    let Some(mut rest) = symbol.strip_prefix(prefix()) else { return false };
    for expected in components {
        if !starts_component(rest) {
            return false;
        }
        match split_component(rest) {
            Some((false, text, after)) if text == *expected => rest = after,
            _ => return false,
        }
    }
    starts_component(rest)
}

/// A transliterated component's text written back: each `_uXXXX_` as the character it stands for.
/// `None` unless it is exactly what [`transliterated`] writes for a name it changes (ODR-202's mark
/// is on no other component): a name beyond ASCII, or one with an ASCII `_` reading as an escape,
/// which it writes `_u005F_` (ODR-213).
fn untransliterated(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("_u") {
        out.push_str(&rest[..at]);
        let after = &rest[at + 2..];
        let digits = after.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(after.len());
        let decoded = (digits >= 4 && after[digits..].starts_with('_'))
            .then(|| u32::from_str_radix(&after[..digits], 16).ok().and_then(char::from_u32))
            .flatten()
            .filter(|&c| !c.is_ascii() || c == '_');
        match decoded {
            Some(c) => {
                out.push(c);
                rest = &after[digits + 1..];
            }
            None => {
                out.push_str("_u");
                rest = after;
            }
        }
    }
    out.push_str(rest);
    (out != text && transliterated(&out) == text).then_some(out)
}

/// `[MNG-2]` — the C name of a function whose `@export` symbol is not a C identifier
/// (`@export("game-update")`): `em_0x` and the symbol's bytes in hex (an item's symbol has the
/// length of its first component after the prefix, never a `0`). The C names the function so, and
/// gives its object-file symbol exactly as written (an assembler label, or a linker alias with
/// MSVC).
pub fn export_alias(symbol: &str) -> String {
    let hex: String = symbol.bytes().map(|byte| format!("{byte:02x}")).collect();
    format!("{}0x{hex}", prefix())
}

/// `[FFI-6]` (D-743, D-744) — the C name of one foreign declaration of a function: the
/// [`export_alias`] of the symbol it binds, then the declaration's path, mangled
/// (`[MNG-1]`), so each declaration of one symbol has a C name of its own, which the C binds to
/// the symbol.
pub fn foreign_alias(symbol: &str, owner: &str) -> String {
    format!("{}_{}", export_alias(symbol), mangled_path(owner))
}

/// The symbol an [`export_alias`] or a [`foreign_alias`] stands for, if `name` is one.
pub fn export_alias_symbol(name: &str) -> Option<String> {
    let hex = name.strip_prefix(prefix())?.strip_prefix("0x")?;
    // A foreign alias's qualified name follows the hex, after `_`.
    let hex = hex.split_once('_').map_or(hex, |(hex, _)| hex);
    if hex.is_empty() || hex.len() % 2 != 0 {
        return None;
    }
    let bytes: Option<Vec<u8>> = (0..hex.len()).step_by(2).map(|at| u8::from_str_radix(&hex[at..at + 2], 16).ok()).collect();
    String::from_utf8(bytes?).ok()
}

/// A runtime entry point: `runtime("alloc")` is `ember_alloc`.
pub fn runtime(symbol: &str) -> String {
    format!("{SYMBOL_PREFIX}_{symbol}")
}

/// `[MNG-4]` — a class's object struct: `em_obj_` and the class's mangled form
/// (`ember_types::TypeTable::mangled`), `em_obj_6Animal`.
pub fn object_struct(mangled_class: &str) -> String {
    format!("{}obj_{mangled_class}", prefix())
}

/// `[MNG-4]` — a method table: `em_vt_` and the class's mangled form (`em_vt_6Animal`), or a
/// tag and what the table is keyed by (`em_vt_dyn_…`, [`generated`]).
pub fn vtable(mangled: &str) -> String {
    format!("{}vt_{mangled}", prefix())
}

/// `[MNG-4]` — a class's type information: `em_ti_` and its mangled form (`em_ti_6Animal`).
pub fn type_info(mangled_class: &str) -> String {
    format!("{}ti_{mangled_class}", prefix())
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

/// The environment variable that moves the global build cache (which holds
/// the compiled runtime) away from the platform's cache directory.
pub fn cache_dir_var() -> String {
    format!("{}_CACHE", SYMBOL_PREFIX.to_uppercase())
}

/// The environment variable that turns off the global cache of linked
/// programs (ADR-136), for timing the C compiler itself.
pub fn no_compile_cache_var() -> String {
    format!("{}_NO_COMPILE_CACHE", SYMBOL_PREFIX.to_uppercase())
}

/// `[BLD-1]` — the environment variable that makes a build check every module, using and keeping
/// no module's checked form (`target/<profile>/incremental/`): a clean build, to compare with.
pub fn no_incremental_var() -> String {
    format!("{}_NO_INCREMENTAL", SYMBOL_PREFIX.to_uppercase())
}

/// The environment variable that chooses the C compiler when `--cc` does not:
/// `msvc`, `clang-cl`, `clang`, `gcc` or `auto`. CI's compiler matrix sets it.
/// `[TST-27]` — set to `1`, the program's own C is compiled with warnings as errors (`/WX`,
/// `-Werror`): the C gate the conformance suite runs every accepted program through.
pub fn c_warnings_as_errors_var() -> String {
    format!("{}_C_WARNINGS_AS_ERRORS", SYMBOL_PREFIX.to_uppercase())
}

/// `[TST-31]` — set to `8`, a build compiles the program and the runtime object with 8-bit counts,
/// and the evaluator counts the same way (`#$ count_bits: 8`): the two count-limit tests reach a
/// count's maximum in 255 steps.
pub fn count_bits_var() -> String {
    format!("{}_COUNT_BITS", SYMBOL_PREFIX.to_uppercase())
}

pub fn cc_var() -> String {
    format!("{}_CC", SYMBOL_PREFIX.to_uppercase())
}

/// D-755 — the code layout `ember bench` builds a program at (`ember_build::LAYOUTS`).
pub fn bench_layout_var() -> String {
    format!("{}_BENCH_LAYOUT", SYMBOL_PREFIX.to_uppercase())
}

/// `[BEN-6]` — set to `1`, a program reports at its end how many allocations it made, which an
/// `@noalloc` path's benchmark asserts exactly (`ember bench`).
pub fn allocations_var() -> String {
    format!("{}_ALLOCATIONS", SYMBOL_PREFIX.to_uppercase())
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
        // D-404 — the variable may name the package (`std/`) as well as its
        // sources (`std/src/`): a package directory taken as the sources
        // held no module, and every `std` import was one not written yet.
        let sources = path.join("src");
        if sources.join(source_file("core")).is_file() {
            return Some(sources);
        }
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
        assert_eq!(cache_dir_var(), "EMBER_CACHE");
        assert_eq!(cc_var(), "EMBER_CC");
    }

    #[test]
    fn the_mangled_prefix_is_derived_not_written() {
        // `[MNG-5]`. If `SYMBOL_PREFIX` ever changes, this changes with it,
        // which is the whole point of deriving it.
        assert_eq!(mangle_prefix(), "em_");
        assert_eq!(item_symbol("std.math.Vec3.length"), "em_3std4math4Vec36length");
        assert_eq!(runtime("alloc"), "ember_alloc");
        assert_eq!(runtime_header(), "ember_rt.h");
    }

    #[test]
    fn a_runtime_symbol_is_distinguishable_from_a_generated_one() {
        // The reason the mangled prefix is short: a reader of the emitted C
        // tells the two apart without a lookup.
        assert!(runtime("retain").starts_with(SYMBOL_PREFIX));
        assert!(!item_symbol("retain").starts_with(&format!("{SYMBOL_PREFIX}_")));
    }

    /// `[MNG-1]` — the specification's own pair, which `_` between components made one symbol.
    #[test]
    fn mangling_is_injective_by_length_prefixes() {
        assert_eq!(item_symbol("lib.m_x.f"), "em_3lib3m_x1f");
        assert_eq!(item_symbol("lib.m.x_f"), "em_3lib1m3x_f");
        assert_ne!(item_symbol("m_x.f"), item_symbol("m.x_f"));
        // A component of ten characters or more has a length of two digits.
        assert_eq!(item_symbol("geometry.probe_cells"), "em_8geometry11probe_cells");
    }

    /// `[MNG-3]` — a character beyond ASCII is transliterated before the length is taken, and the
    /// length is marked so (ODR-202): `café` and the ASCII name `caf_u00E9_` are two symbols.
    #[test]
    fn a_name_beyond_ascii_is_transliterated_before_its_length() {
        assert_eq!(item_symbol("café"), "em_u10caf_u00E9_");
        assert_eq!(item_symbol("größe.maß.f"), "em_u17gr_u00F6__u00DF_eu9ma_u00DF_1f");
        assert_eq!(demangle("em_u10caf_u00E9_").map(|d| d.path()), Some("café".to_string()));
        // `[MNG-3]` (ODR-213) — an ASCII `_` that would read as an escape is one, `_u005F_`, and the
        // component is marked: the ASCII name `caf_u00E9_` and `café` are two symbols, and so are
        // `é_u00E9_` and `éé`; a `_u` that reads as no escape is itself.
        assert_eq!(item_symbol("caf_u00E9_"), "em_u16caf_u005F_u00E9_");
        assert_eq!(demangle("em_u16caf_u005F_u00E9_").map(|d| d.path()), Some("caf_u00E9_".to_string()));
        assert_ne!(item_symbol("é_u00E9_"), item_symbol("éé"));
        assert_eq!(item_symbol("é_u00E9_"), "em_u20_u00E9__u005F_u00E9_");
        assert_eq!(demangle(&item_symbol("é_u00E9_")).map(|d| d.path()), Some("é_u00E9_".to_string()));
        assert_eq!(item_symbol("a_u0041__u0042_"), "em_u27a_u005F_u0041__u005F_u0042_");
        assert_eq!(item_symbol("my_url_x_u12_"), "em_13my_url_x_u12_");
        assert_ne!(c_identifier("caf_u00E9_"), c_identifier("café"));
        assert_eq!(c_identifier("caf_u00E9_"), "caf_u005F_u00E9_");
        // A mark on what the compiler never marks is no item's: an ASCII text, a lower-case code
        // point, an escape of an ASCII character.
        for near in ["em_u3abc", "em_u10caf_u00e9_", "em_u10caf_u0041_"] {
            assert_eq!(demangle(near), None, "{near}");
        }
        // A marked component is a component: an instance after it, and std's prefix test.
        let instance = instance_symbol("geometry.größe", Some("[i64]"));
        assert_eq!(demangle(&instance).map(|d| (d.path(), d.instance.is_some())), Some(("geometry.größe".to_string(), true)));
        assert_eq!(reinstance(&item_symbol("größe"), "[0]"), instance_symbol("größe", Some("[0]")));
        assert!(symbol_path_starts_with(&item_symbol("std.größe"), &["std"]));
        assert!(!symbol_path_starts_with(&item_symbol("größe.f"), &["gr_u00F6__u00DF_e"]));
    }

    /// `[MNG-1]` — an instance's hash: `G` and 16 hex digits of BLAKE3 of its arguments' spelling.
    #[test]
    fn an_instance_ends_with_its_arguments_hash() {
        let symbol = instance_symbol("max", Some("[i64]"));
        let expected = format!("em_3maxG{}", &blake3::hash(b"[i64]").to_hex()[..16]);
        assert_eq!(symbol, expected);
        assert_ne!(symbol, instance_symbol("max", Some("[f64]")));
        assert_eq!(instance_symbol("max", None), "em_3max");
    }

    /// `[CG-C-10]` — `ember demangle` reads a symbol back exactly: every component, the instance's
    /// hash and what follows.
    #[test]
    fn demangling_round_trips() {
        for path in ["lib.m_x.f", "lib.m.x_f", "geometry.Grid.at", "a.b2.c10", "std.core.Option.unwrap", "größe.maß", "caf_u00E9_.café", "u1.u2x"] {
            let symbol = item_symbol(path);
            let read = demangle(&symbol).expect("an item symbol");
            assert_eq!(read.path(), path);
            assert_eq!((read.instance, read.rest), (None, ""));
        }
        let symbol = instance_symbol("geometry.Grid.at", Some("[i64]"));
        let rl = format!("{symbol}__rl");
        let read = demangle(&rl).expect("an instance symbol");
        assert_eq!(read.path(), "geometry.Grid.at");
        assert_eq!(read.instance.map(str::len), Some(INSTANCE_HASH_DIGITS));
        assert_eq!(read.rest, "__rl");
        // Not an item's: a generated name, an export alias, the runtime's.
        assert_eq!(demangle(&generated("closure3")), None);
        assert_eq!(demangle(&export_alias("game-update")), None);
        assert_eq!(demangle(&runtime("alloc")), None);
        assert!(symbol_path_starts_with(&item_symbol("std.fmt.write"), &["std"]));
        assert!(!symbol_path_starts_with(&item_symbol("std"), &["std"]));
        assert!(!symbol_path_starts_with(&item_symbol("stdx.f"), &["std"]));
    }

    /// `[CLO-3]` — a copy of a function for a function passed to it is an instance of that
    /// function; of an instance, an instance over its hash and the new arguments.
    #[test]
    fn a_further_instance_keeps_the_path() {
        let plain = item_symbol("apply");
        let copy = reinstance(&plain, "[0 = em_6triple]");
        assert_eq!(copy, instance_symbol("apply", Some("[0 = em_6triple]")));
        let instance = instance_symbol("apply", Some("[i64]"));
        let again = reinstance(&instance, "[0 = em_6triple]");
        assert_eq!(demangle(&again).map(|found| found.path()), Some("apply".to_string()));
        assert_ne!(again, copy);
        assert_ne!(again, instance);
        assert_eq!(reinstance(&generated("closure3"), "[0]"), format!("{}_{}", generated("closure3"), instance_suffix("[0]")));
    }

    #[test]
    fn a_foreign_alias_names_its_declaration_exactly() {
        assert_ne!(foreign_alias("puts", "m_x.f"), foreign_alias("puts", "m.x_f"));
        assert_eq!(export_alias_symbol(&foreign_alias("puts", "m_x.f")).as_deref(), Some("puts"));
    }
}
