//! `[FFI-6]` — `import c "header" with (…) [as name]`: the header parsed with libclang (loaded at
//! run time through `clang-sys`, so the compiler builds and runs without it), and its declarations
//! written out as an Ember module of the forms a hand declaration uses (`[FFI-8]`): `@layout(c)`
//! structs, `@repr(…) @non_exhaustive` enums, type aliases, constants, and an `unsafe extern` block
//! of functions, opaque types and statics. Everything after the parser then treats them as it treats
//! hand declarations. `[FFI-38]` — a fact the header does not state is never guessed: a function is
//! uncontracted (its calls need `unsafe`) unless an overlay states its contract (`[FFI-2]`).
//! `[FFI-20a]` — a declaration that cannot be represented is recorded with the construct that
//! defeated it (`W5002`), an object-like macro that is no constant with Clang's reason (`W5001`).
//! `[FFI-29]` — the shim: layout assertions (`[FFI-5a]`) and an external wrapper for each function
//! with internal linkage (`[FFI-29a]`).
#![allow(unsafe_code)] // libclang's C interface
#![allow(non_upper_case_globals)] // clang-sys names its constants as libclang does

use clang_sys::*;
use ember_ast::{self as ast, AttrArg, Expr, ExprKind, Literal};
use ember_diag::{Diagnostic, Sink, codes};
use ember_span::{SourceMap, Span, Symbol};
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_ulong};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// What an import is parsed with: `[BLD-FFI-1a]` the package's C flags plus the import's own
/// `defines` and `include_paths`, and no others.
pub struct Config<'a> {
    /// The header as written: `"mylib.h"`, `"vulkan/vulkan.h"`.
    pub header: &'a str,
    /// The importing file's directory, searched first as C searches a quoted include's.
    pub from_dir: &'a Path,
    pub include_paths: &'a [PathBuf],
    pub defines: &'a [String],
    pub flags: &'a [String],
    /// The module's path joined with `_`: the shim's symbols are named with it.
    pub symbol_stem: &'a str,
    /// `[FFI-30]` — the declarations earlier imports of the build wrote, by Clang's USR.
    pub shared: &'a HashMap<String, Shared>,
    /// The name the import binds (`c`, or its `as` name).
    pub binding: &'a str,
    /// `[FFI-17]` — for `import cpp`: the C++ standard the header is parsed with (`c++17`).
    pub cpp: Option<&'a str>,
    /// `[FFI-17b]` — the explicit template instantiations to import: `"Vec<int>"`, `"max<double>"`.
    pub instantiate: &'a [String],
}

/// `[FFI-30]` — a C declaration (by Clang's USR) an import of the build wrote: one Ember entity
/// for every import of it with the same layout.
#[derive(Clone, Debug)]
pub struct Shared {
    /// The module that wrote it, and its name there.
    pub module: Vec<String>,
    pub name: String,
    /// Its layout: size, alignment and offsets of a struct, representation and values of an enum.
    pub layout: String,
    pub binding: String,
    pub header: String,
}

/// A declaration not imported, with the construct that defeated it and a workaround.
#[derive(Clone, Debug)]
pub struct Skipped {
    pub name: String,
    pub reason: String,
    /// `file:line` in the header.
    pub at: String,
}

/// `[FFI-5]` — Clang's layout of an imported struct, in bytes.
#[derive(Clone, Debug)]
pub struct RecordLayout {
    /// The Ember name of the struct in its module.
    pub name: String,
    pub size: u64,
    pub align: u64,
    pub fields: Vec<(String, u64)>,
    /// `[FFI-41]` — the C++ namespace it is in, its module's path under the import's.
    pub namespace: Vec<String>,
}

/// A foreign function as the header declares it, for an overlay to contract (`[FFI-2]`).
#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    /// Each parameter's Ember name and C type.
    pub params: Vec<(String, CType)>,
    pub result: Option<CType>,
    /// The symbol it binds when that is not its name: a shim wrapper (`[FFI-29a]`), or a C++
    /// function's thunk (`[FFI-3]`).
    pub link_name: Option<String>,
    pub abi: &'static str,
    /// `[FFI-24]` — a C++ function's exception policy; `Direct` for every C function.
    pub policy: Policy,
    /// `[FFI-40]`, `[FFI-41]` — for a C++ member function: its class's Ember name, and what it is.
    pub owner: Option<(String, Member)>,
    /// A C++ class result, which the thunk writes through a pointer the caller gives.
    pub out: bool,
    /// `[FFI-41]` — the C++ namespace it is in.
    pub namespace: Vec<String>,
}

/// `[FFI-40]`, `[FFI-41]`, `[FFI-32]` — what a C++ member function is to its class's Ember type:
/// a `const` method (`self`), any other (`mut self`), a static one (an associated function), a
/// constructor of a class by value (`new(…) -> T`) or of an opaque one (`new(…) ->
/// ForeignBox[T]`), or the destructor a `ForeignBox` calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Member {
    Method,
    MutMethod,
    Static,
    Constructor,
    BoxedConstructor,
    Destructor,
}

const MEMBERS: [Member; 6] = [Member::Method, Member::MutMethod, Member::Static, Member::Constructor, Member::BoxedConstructor, Member::Destructor];

/// `[FFI-24]` — how a C++ function is called: directly when its declaration is non-throwing,
/// else through a thunk that catches (its Ember function returns `Result[T, CppError]`); an
/// overlay's `@ffi(noexcept)` keeps the catching thunk and panics on a throw (`[FFI-24b]`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    Direct,
    Catching,
    Asserted,
    /// Neither derived nor declared: `E5061` where it is used.
    Unknown,
}

const POLICIES: [Policy; 4] = [Policy::Direct, Policy::Catching, Policy::Asserted, Policy::Unknown];

/// `[FFI-6b]` — a function-like macro: its name and parameters, for an overlay's
/// `@ffi(macro_fn)` entry to state its signature.
#[derive(Clone, Debug)]
pub struct MacroFunction {
    pub name: String,
    pub params: Vec<String>,
}

/// A C type as the importer maps it (`[FFI-8]`).
#[derive(Clone, Debug)]
pub struct CType {
    /// The Ember type with no contract: `*mut T`, `*T`, a scalar, a struct.
    pub ember: String,
    /// For a data pointer: what it points to, as an Ember type, and whether it is `const`.
    pub pointee: Option<(String, bool)>,
    /// `const char*` or `char*`: a `cstr` under `nul_terminated`.
    pub char_pointer: bool,
    /// The C type as the header writes it.
    pub c: String,
    /// `[FFI-40]` — the parameter's mode: `mut ` for a C++ `T&`, `owned ` for a `T&&`, else none.
    pub mode: String,
}

/// What a declaration of the header became: an overlay finds it by its C name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A struct, an enum or a type alias.
    Type,
    Const,
    /// An `extern type` of the `unsafe extern` block.
    Opaque,
    /// An `extern static`.
    Static,
}

/// One declaration of the module, without `pub` (an overlay may hide it).
#[derive(Clone, Debug)]
pub struct Item {
    /// Its C name.
    pub name: String,
    pub kind: Kind,
    /// Its text at column 0; attributes first.
    pub text: String,
    /// `[FFI-41]` — the C++ namespace it is in.
    pub namespace: Vec<String>,
}

pub struct Imported {
    /// The module's declarations but its functions, in the order written.
    pub items: Vec<Item>,
    /// The header file found.
    pub header: PathBuf,
    /// `W5002`.
    pub skipped: Vec<Skipped>,
    /// `W5002` too: a struct imported only as an opaque type, with why.
    pub opaque: Vec<Skipped>,
    /// `W5001`.
    pub macros: Vec<Skipped>,
    pub layouts: Vec<RecordLayout>,
    /// `[FFI-29]` — the shim's C, which includes the header by its path.
    pub shim: String,
    pub functions: Vec<Function>,
    /// `[FFI-30]` — the modules whose entities this one names, each declaration it wrote (its USR,
    /// name and layout), and each one an earlier import wrote with another layout.
    pub uses: Vec<Vec<String>>,
    pub entities: Vec<(String, String, String)>,
    pub conflicts: Vec<(String, String)>,
    /// `[FFI-6b]` — the function-like macros, for an overlay entry to declare.
    pub macro_functions: Vec<MacroFunction>,
    /// `[FFI-6b]` — each type the module declares, by its Ember name, as C spells it: a macro's
    /// wrapper is written with it.
    pub c_types: Vec<(String, String)>,
    /// `[FFI-41]` — `W5034`: each entity of an anonymous namespace.
    pub anonymous_namespace: Vec<Skipped>,
    /// `[FFI-40]` — `W5033`: each `&&`-qualified member function.
    pub rvalue_members: Vec<Skipped>,
    /// `[FFI-41]` — each C++ namespace with declarations (a module of its own), and which
    /// namespace's module names which other's types.
    pub namespaces: Vec<Vec<String>>,
    pub ns_uses: Vec<(Vec<String>, Vec<String>)>,
}

fn text(string: CXString) -> String {
    // SAFETY(ffi): a CXString from libclang, read once and disposed.
    unsafe {
        let chars = clang_getCString(string);
        let out = if chars.is_null() { String::new() } else { CStr::from_ptr(chars).to_string_lossy().into_owned() };
        clang_disposeString(string);
        out
    }
}

extern "C" fn gather(cursor: CXCursor, _: CXCursor, data: CXClientData) -> CXChildVisitResult {
    // SAFETY(ffi): `data` is the `Vec` `children` passes, alive for the visit.
    unsafe { (*(data as *mut Vec<CXCursor>)).push(cursor) };
    CXChildVisit_Continue
}

extern "C" fn gather_field(cursor: CXCursor, data: CXClientData) -> CXVisitorResult {
    // SAFETY(ffi): `data` is the `Vec` `fields` passes, alive for the visit.
    unsafe { (*(data as *mut Vec<CXCursor>)).push(cursor) };
    CXVisit_Continue
}

/// `[FFI-17b]` — a record's fields as its type has them: a template specialization's, which its
/// cursor's children do not show.
fn record_fields(ty: CXType) -> Vec<CXCursor> {
    let mut out: Vec<CXCursor> = Vec::new();
    // SAFETY(ffi): the visitor only pushes into `out`, which outlives the call.
    unsafe { clang_Type_visitFields(ty, gather_field, &mut out as *mut Vec<CXCursor> as CXClientData) };
    out
}

fn children(cursor: CXCursor) -> Vec<CXCursor> {
    let mut out: Vec<CXCursor> = Vec::new();
    // SAFETY(ffi): the visitor only pushes into `out`.
    unsafe { clang_visitChildren(cursor, gather, &mut out as *mut Vec<CXCursor> as CXClientData) };
    out
}

extern "C" fn inclusion(file: CXFile, _: *mut CXSourceLocation, depth: c_uint, data: CXClientData) {
    // SAFETY(ffi): `data` is the `Vec` `included_directly` passes.
    unsafe { (*(data as *mut Vec<(String, u32)>)).push((text(clang_getFileName(file)), depth)) };
}

fn spelling(cursor: CXCursor) -> String {
    // SAFETY(ffi): a cursor of a live translation unit.
    text(unsafe { clang_getCursorSpelling(cursor) })
}

fn type_spelling(ty: CXType) -> String {
    // SAFETY(ffi): a type of a live translation unit.
    text(unsafe { clang_getTypeSpelling(ty) })
}

fn kind(cursor: CXCursor) -> CXCursorKind {
    // SAFETY(ffi): as above.
    unsafe { clang_getCursorKind(cursor) }
}

fn usr(cursor: CXCursor) -> String {
    // SAFETY(ffi): as above; the canonical cursor names every redeclaration alike.
    unsafe { text(clang_getCursorUSR(clang_getCanonicalCursor(cursor))) }
}

/// The file and line a cursor's declaration is written at, where a macro expanded it.
fn location(cursor: CXCursor) -> (String, u32, bool) {
    // SAFETY(ffi): out-parameters of a live location.
    unsafe {
        let at = clang_getCursorLocation(cursor);
        let mut file: CXFile = std::ptr::null_mut();
        let (mut line, mut column, mut offset) = (0, 0, 0);
        clang_getExpansionLocation(at, &mut file, &mut line, &mut column, &mut offset);
        let name = if file.is_null() { String::new() } else { text(clang_getFileName(file)) };
        (name, line, clang_Location_isInSystemHeader(at) != 0)
    }
}

/// An Ember name for a C identifier: a raw identifier where it is an Ember keyword (`[FFI-8]`,
/// `[LEX-14]`).
fn ident(name: &str) -> String {
    if ember_lexer::Kw::from_str(name).is_some() || ember_lexer::Reserved::from_str(name).is_some() {
        format!("r#{name}")
    } else {
        name.to_string()
    }
}

/// `[FFI-8]` — the C names whose Ember type is fixed whatever they are defined as.
fn standard_typedef(name: &str) -> Option<&'static str> {
    Some(match name {
        "size_t" | "__size_t" | "uintptr_t" => "usize",
        "ptrdiff_t" | "__ptrdiff_t" | "intptr_t" | "ssize_t" => "isize",
        _ => return None,
    })
}

/// A translation unit and its index, disposed together.
struct Unit {
    index: CXIndex,
    tu: CXTranslationUnit,
}

impl Drop for Unit {
    fn drop(&mut self) {
        // SAFETY(ffi): each made once by `parse`, disposed once here.
        unsafe {
            clang_disposeTranslationUnit(self.tu);
            clang_disposeIndex(self.index);
        }
    }
}

/// One Clang diagnostic: its severity, its file and line, and its text.
struct Note {
    error: bool,
    file: String,
    line: u32,
    text: String,
}

impl Unit {
    fn parse(main: &Path, contents: &str, args: &[String]) -> Result<Unit, String> {
        let main_c = CString::new(main.to_string_lossy().into_owned()).map_err(|e| e.to_string())?;
        let contents_c = CString::new(contents).map_err(|e| e.to_string())?;
        let args_c: Vec<CString> = args.iter().map(|arg| CString::new(arg.as_str())).collect::<Result<_, _>>().map_err(|e| e.to_string())?;
        let argv: Vec<*const c_char> = args_c.iter().map(|arg| arg.as_ptr()).collect();
        let mut unsaved = CXUnsavedFile {
            Filename: main_c.as_ptr(),
            Contents: contents_c.as_ptr(),
            Length: contents.len() as c_ulong,
        };
        // SAFETY(ffi): every pointer outlives the call; the unit is disposed by `Drop`.
        unsafe {
            let index = clang_createIndex(0, 0);
            let tu = clang_parseTranslationUnit(
                index,
                main_c.as_ptr(),
                argv.as_ptr(),
                argv.len() as c_int,
                &mut unsaved,
                1,
                CXTranslationUnit_DetailedPreprocessingRecord | CXTranslationUnit_SkipFunctionBodies,
            );
            if tu.is_null() {
                clang_disposeIndex(index);
                return Err("libclang could not parse it".to_string());
            }
            Ok(Unit { index, tu })
        }
    }

    fn cursor(&self) -> CXCursor {
        // SAFETY(ffi): a live unit.
        unsafe { clang_getTranslationUnitCursor(self.tu) }
    }

    fn notes(&self) -> Vec<Note> {
        let mut out = Vec::new();
        // SAFETY(ffi): each diagnostic read, then disposed.
        unsafe {
            for index in 0..clang_getNumDiagnostics(self.tu) {
                let diagnostic = clang_getDiagnostic(self.tu, index);
                let error = clang_getDiagnosticSeverity(diagnostic) >= CXDiagnostic_Error;
                let mut file: CXFile = std::ptr::null_mut();
                let (mut line, mut column, mut offset) = (0, 0, 0);
                clang_getExpansionLocation(clang_getDiagnosticLocation(diagnostic), &mut file, &mut line, &mut column, &mut offset);
                let file = if file.is_null() { String::new() } else { text(clang_getFileName(file)) };
                out.push(Note { error, file, line, text: text(clang_getDiagnosticSpelling(diagnostic)) });
                clang_disposeDiagnostic(diagnostic);
            }
        }
        out
    }

    /// Every file the main file includes, with how deep: 1 for the ones it includes itself.
    fn inclusions(&self) -> Vec<(String, u32)> {
        let mut out: Vec<(String, u32)> = Vec::new();
        // SAFETY(ffi): the visitor only pushes into `out`.
        unsafe { clang_getInclusions(self.tu, inclusion, &mut out as *mut Vec<(String, u32)> as CXClientData) };
        out
    }
}

/// libclang, loaded once per thread; its absence names what to install.
fn load_libclang() -> Result<(), String> {
    if clang_sys::is_loaded() {
        return Ok(());
    }
    clang_sys::load().map_err(|error| {
        format!(
            "{error}\n  = help: install LLVM (it provides libclang: `winget install LLVM.LLVM` on Windows, \
             `apt install libclang-dev` on Debian and Ubuntu), or set LIBCLANG_PATH to the folder that holds it"
        )
    })?;
    // D-751 — never unloaded: clang-sys keeps the library in a thread-local, and freeing libclang
    // as that is destroyed at exit crashed the compiler (an access violation after its work was
    // done) with the LLVM of the Windows CI image. One reference is kept for the process's life.
    std::mem::forget(clang_sys::get_library());
    Ok(())
}

/// The arguments libclang parses with: C11, as the generated C is compiled, for the host target.
fn arguments(config: &Config) -> Vec<String> {
    let mut args = match config.cpp {
        Some(standard) => vec!["-x".to_string(), "c++".to_string(), format!("-std={standard}")],
        None => vec!["-x".to_string(), "c".to_string(), "-std=c11".to_string()],
    };
    args.extend(config.include_paths.iter().map(|dir| format!("-I{}", dir.display())));
    args.extend(config.defines.iter().map(|define| format!("-D{define}")));
    args.extend(config.flags.iter().cloned());
    args
}

/// `[FFI-6]` — parse the header and write its module.
/// `[FFI-6]` — parse the header and write its module, with every file the header includes.
pub fn import_header(config: &Config) -> Result<(Imported, Vec<String>), String> {
    load_libclang()?;
    // An absolute folder: Clang then names every file it finds by its absolute path.
    let from_dir = std::path::absolute(config.from_dir).map_err(|e| e.to_string())?;
    let main = from_dir.join(format!("{}_import_c.{}", ember_branding::SYMBOL_PREFIX, if config.cpp.is_some() { "cpp" } else { "c" }));
    let args = arguments(config);
    let include = format!("#include \"{}\"\n", config.header.replace('\\', "/"));
    let unit = Unit::parse(&main, &include, &args)?;
    let errors: Vec<String> = unit
        .notes()
        .into_iter()
        .filter(|note| note.error)
        // Where the header is included from is this importer's own file: only the header's are shown.
        .map(|note| if note.file.is_empty() || Path::new(&note.file) == main { note.text } else { format!("{}:{}: {}", note.file, note.line, note.text) })
        .collect();
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let inclusions = unit.inclusions();
    // `[FFI-17b]` — each listed instantiation, declared after the header: a class template's
    // explicitly instantiated and named, a function template's address taken.
    let unit = if config.cpp.is_some() && !config.instantiate.is_empty() {
        let prefix = ember_branding::SYMBOL_PREFIX;
        let mut source = include.clone();
        for (index, written) in config.instantiate.iter().enumerate() {
            let id = written.trim().trim_start_matches("::");
            let template = id.split('<').next().unwrap_or(id).trim();
            match find_template(unit.cursor(), template) {
                Some(CXCursor_ClassTemplate) => {
                    let _ = writeln!(source, "template class ::{id};\nusing {prefix}_inst_{index} = ::{id};");
                }
                Some(_) => {
                    let _ = writeln!(source, "constexpr auto {prefix}_inst_{index} = &::{id};");
                }
                None => return Err(format!("`instantiate` names `{written}`, and `{}` declares no template `{template}` ([FFI-17b])", config.header)),
            }
        }
        let instantiated = Unit::parse(&main, &source, &args)?;
        let errors: Vec<String> = instantiated.notes().into_iter().filter(|note| note.error).map(|note| note.text).collect();
        if !errors.is_empty() {
            return Err(format!("the instantiations `instantiate` lists do not compile:\n{}", errors.join("\n")));
        }
        instantiated
    } else {
        unit
    };
    let header = inclusions.iter().find(|(_, depth)| *depth == 1).map(|(file, _)| file.clone()).ok_or_else(|| format!("`{}` was not found", config.header))?;
    let files: Vec<String> = inclusions.into_iter().filter(|(_, depth)| *depth > 0).map(|(file, _)| file).collect();
    let header_path = std::path::absolute(from_dir.join(&header)).unwrap_or_else(|_| PathBuf::from(&header));
    let mut writer = Writer {
        header: header.clone(),
        stem: config.symbol_stem.to_string(),
        shared: config.shared.clone(),
        binding: config.binding.to_string(),
        cpp: config.cpp.is_some(),
        ..Writer::default()
    };
    let in_header = |cursor: CXCursor| {
        let (file, _, system) = location(cursor);
        file == header || !(system || file.is_empty() || Path::new(&file) == main)
    };
    if writer.cpp {
        // `[FFI-32]`, `[FFI-24]` — what only C++'s own rules answer, asked of Clang in one more parse.
        let probe_args = {
            let mut probe_args = args.clone();
            probe_args.push("-ferror-limit=0".to_string());
            probe_args
        };
        (writer.by_value, writer.nothrow) = cpp_facts(&unit, &main, &include, &probe_args, &in_header)?;
    }
    for cursor in children(unit.cursor()) {
        if in_header(cursor) {
            writer.declaration(cursor);
        }
    }
    // `[FFI-17b]` — the instantiations, each under a name made of its template-id (`Vec<int>` is
    // `Vec_int`), in its template's namespace.
    let prefix = format!("{}_inst_", ember_branding::SYMBOL_PREFIX);
    for cursor in children(unit.cursor()) {
        let Some(index) = spelling(cursor).strip_prefix(&prefix).and_then(|index| index.parse::<usize>().ok()) else { continue };
        let written = config.instantiate[index].trim().trim_start_matches("::").to_string();
        writer.instantiation(cursor, &written);
    }
    // `[FFI-6a]` — the object-like macros, evaluated by Clang in a second parse.
    let macros = writer.macro_names.clone();
    if !macros.is_empty() {
        let mut probe = include.clone();
        for (index, (name, _)) in macros.iter().enumerate() {
            // Unparenthesised: Clang evaluates a string only as the literal itself.
            let deduced = if writer.cpp { "auto" } else { "__auto_type" };
            let _ = writeln!(probe, "static const {deduced} {}_macro_{index} = {name};", ember_branding::SYMBOL_PREFIX);
        }
        // Every macro's error reported, however many: each names its own line.
        let mut probe_args = args.clone();
        probe_args.push("-ferror-limit=0".to_string());
        let unit = Unit::parse(&main, &probe, &probe_args)?;
        let main_name = main.to_string_lossy().replace('\\', "/");
        let mut reasons: HashMap<usize, String> = HashMap::new();
        for note in unit.notes().into_iter().filter(|note| note.error) {
            if note.file.replace('\\', "/") == main_name && note.line >= 2 {
                reasons.entry(note.line as usize - 2).or_insert(note.text);
            }
        }
        let prefix = format!("{}_macro_", ember_branding::SYMBOL_PREFIX);
        let mut values: HashMap<usize, CXCursor> = HashMap::new();
        for cursor in children(unit.cursor()) {
            if kind(cursor) == CXCursor_VarDecl
                && let Some(index) = spelling(cursor).strip_prefix(&prefix).and_then(|index| index.parse::<usize>().ok())
            {
                values.insert(index, cursor);
            }
        }
        // `[FFI-6a]` (ODR-145) — a null-pointer or pointer-cast macro: its address, from a third parse.
        let mut addresses: Vec<(usize, String)> = Vec::new();
        for (index, (name, at)) in macros.iter().enumerate() {
            let outcome = match (reasons.get(&index), values.get(&index)) {
                (Some(reason), _) => Err(format!("Clang: {reason}")),
                (None, Some(&cursor)) => writer.constant_value(cursor),
                (None, None) => Err("Clang made no declaration of its value".to_string()),
            };
            if outcome.is_err()
                && reasons.get(&index).is_none()
                && let Some(pointer) = values.get(&index).and_then(|&cursor| writer.pointer_constant(cursor))
            {
                addresses.push((index, pointer));
                continue;
            }
            match outcome {
                Ok((ty, value)) if writer.claim(name) => {
                    writer.push(name, Kind::Const, format!("const {}: {ty} = {value}", ident(name)));
                }
                Ok(_) => writer.macro_skipped.push(Skipped {
                    name: name.clone(),
                    reason: "its name is taken by a declaration of the header".to_string(),
                    at: at.clone(),
                }),
                Err(reason) => writer.macro_skipped.push(Skipped { name: name.clone(), reason, at: at.clone() }),
            }
        }
        if !addresses.is_empty() {
            let mut probe = include.clone();
            for (line, (index, _)) in addresses.iter().enumerate() {
                let _ = writeln!(probe, "static const unsigned long long {prefix}{line} = (unsigned long long)({});", macros[*index].0);
            }
            let unit = Unit::parse(&main, &probe, &probe_args)?;
            let mut reasons: HashMap<usize, String> = HashMap::new();
            for note in unit.notes().into_iter().filter(|note| note.error) {
                if note.file.replace('\\', "/") == main_name && note.line >= 2 {
                    reasons.entry(note.line as usize - 2).or_insert(note.text);
                }
            }
            let mut values: HashMap<usize, u64> = HashMap::new();
            for cursor in children(unit.cursor()) {
                if kind(cursor) == CXCursor_VarDecl
                    && let Some(line) = spelling(cursor).strip_prefix(&prefix).and_then(|line| line.parse::<usize>().ok())
                    && let Some(address) = evaluate_unsigned(cursor)
                {
                    values.insert(line, address);
                }
            }
            for (line, (index, pointer)) in addresses.iter().enumerate() {
                let (name, at) = &macros[*index];
                match (values.get(&line), reasons.get(&line)) {
                    (Some(address), _) if writer.claim(name) => {
                        writer.push(name, Kind::Const, format!("const {}: {pointer} = {address}usize as {pointer}", ident(name)));
                    }
                    (Some(_), _) => writer.macro_skipped.push(Skipped {
                        name: name.clone(),
                        reason: "its name is taken by a declaration of the header".to_string(),
                        at: at.clone(),
                    }),
                    (None, reason) => writer.macro_skipped.push(Skipped {
                        name: name.clone(),
                        reason: match reason {
                            Some(reason) => format!("a pointer whose address Clang does not evaluate: {reason}"),
                            None => "a pointer whose address Clang does not evaluate to an integer".to_string(),
                        },
                        at: at.clone(),
                    }),
                }
            }
        }
    }
    Ok((writer.finish(header_path), files))
}

/// What a C record became.
#[derive(Clone)]
enum Record {
    /// Being written: a pointer to it may name it.
    Pending(String),
    Full(String),
    /// An `extern type`, used only behind pointers; why, when it has a definition.
    Opaque(String, Option<String>),
}

#[derive(Default)]
struct Writer {
    header: String,
    stem: String,
    shared: HashMap<String, Shared>,
    binding: String,
    uses: Vec<Vec<String>>,
    entities: Vec<(String, String, String)>,
    conflicts: Vec<(String, String)>,
    items: Vec<Item>,
    shim: String,
    skipped: Vec<Skipped>,
    opaque: Vec<Skipped>,
    macro_skipped: Vec<Skipped>,
    layouts: Vec<RecordLayout>,
    functions: Vec<Function>,
    /// Object-like macros to evaluate, with where each is defined.
    macro_names: Vec<(String, String)>,
    macro_functions: Vec<MacroFunction>,
    c_types: Vec<(String, String)>,
    /// Ember names taken in the module.
    names: HashSet<String>,
    records: HashMap<String, Record>,
    enums: HashMap<String, Result<String, String>>,
    typedefs: HashMap<String, Result<String, String>>,
    seen: HashSet<String>,
    /// `[FFI-8]` — the generated `<Parent>_anon<N>` types of anonymous members: C has no name for
    /// them, so the shim checks their bytes through the parent.
    anonymous: HashSet<String>,
    /// `[FFI-17]` — the header is C++: its functions are reached through thunks.
    cpp: bool,
    /// `[FFI-32]` — each C++ class, by USR, that is standard-layout and trivially copyable.
    by_value: HashMap<String, bool>,
    /// `[FFI-24]` — each C++ function, by USR, whose `noexcept(expr)` holds.
    nothrow: HashMap<String, bool>,
    /// The thunks written, for their names.
    thunks: usize,
    /// `[FFI-17b]` — the function template specialization being imported, as C++ names it.
    callee: Option<String>,
    anonymous_namespace: Vec<Skipped>,
    rvalue_members: Vec<Skipped>,
    /// `[FFI-41]` — the namespace being written, each declaration's (by USR), every namespace with
    /// declarations, and which namespace names which other's types.
    writing: Vec<String>,
    homes: HashMap<String, Vec<String>>,
    namespaces: Vec<Vec<String>>,
    ns_uses: Vec<(Vec<String>, Vec<String>)>,
}

impl Writer {
    fn at(&self, cursor: CXCursor) -> String {
        let (file, line, _) = location(cursor);
        let file = Path::new(&file).file_name().map_or(file.clone(), |name| name.to_string_lossy().into_owned());
        format!("{file}:{line}")
    }

    fn skip(&mut self, cursor: CXCursor, name: &str, reason: impl Into<String>) {
        let at = self.at(cursor);
        self.skipped.push(Skipped { name: name.to_string(), reason: reason.into(), at });
    }

    fn push(&mut self, name: &str, kind: Kind, text: String) {
        self.items.push(Item { name: name.to_string(), kind, text: text.trim_end().to_string(), namespace: self.writing.clone() });
    }

    /// `[FFI-6b]` — the C spelling of a type just declared, by its Ember name: Clang's (`struct
    /// Pair`, `Point`), or the typedef's name of a struct with no tag of its own.
    fn spelled(&mut self, cursor: CXCursor, named: &str) {
        if self.anonymous.contains(named) {
            return;
        }
        // SAFETY(ffi): a live cursor.
        let spelled = type_spelling(unsafe { clang_getCursorType(cursor) });
        let c = if spelled.is_empty() || spelled.contains('(') { named.to_string() } else { spelled };
        self.c_types.push((ident(named), c));
    }

    /// Take `name` in the module's one namespace; C's tags and ordinary names are two.
    fn claim(&mut self, name: &str) -> bool {
        // `[FFI-41]` — each C++ namespace is a module, with a namespace of names of its own.
        self.names.insert(format!("{}::{name}", self.writing.join("::")))
    }

    fn declaration(&mut self, cursor: CXCursor) {
        match kind(cursor) {
            CXCursor_FunctionDecl => self.function(cursor),
            // `extern "C" { … }` in a C++ header.
            CXCursor_LinkageSpec => {
                for child in children(cursor) {
                    self.declaration(child);
                }
            }
            // `[FFI-41]` — a namespace is a module path: its declarations a module under the import's
            // (`engine::math::dot` is `cpp.math.dot`); an anonymous namespace's entities have
            // internal linkage, `W5034`.
            CXCursor_Namespace => {
                // SAFETY(ffi): a live cursor.
                let anonymous = unsafe { clang_Cursor_isAnonymous(cursor) } != 0 || spelling(cursor).is_empty();
                if anonymous {
                    for child in children(cursor) {
                        let name = spelling(child);
                        if !name.is_empty() {
                            let at = self.at(child);
                            self.anonymous_namespace.push(Skipped { name, reason: "it is in an anonymous namespace, so each C++ file has its own".to_string(), at });
                        }
                    }
                    return;
                }
                let mut inner = self.writing.clone();
                inner.push(spelling(cursor));
                if !self.namespaces.contains(&inner) {
                    self.namespaces.push(inner.clone());
                }
                let saved = std::mem::replace(&mut self.writing, inner);
                for child in children(cursor) {
                    self.declaration(child);
                }
                self.writing = saved;
            }
            CXCursor_ClassTemplate | CXCursor_FunctionTemplate | CXCursor_ClassTemplatePartialSpecialization => {
                let name = spelling(cursor);
                self.skip(cursor, &name, "[E5055] a C++ template: only the explicit instantiations `instantiate=[…]` lists are imported, and no Ember code, generic or not, instantiates one ([FFI-17b])");
            }
            CXCursor_ClassDecl => {
                // SAFETY(ffi): a live cursor.
                if unsafe { clang_Cursor_isAnonymous(cursor) } == 0 {
                    let record = self.record(cursor, None);
                    self.cpp_members(cursor, record);
                }
            }
            CXCursor_TypeAliasDecl => {
                let _ = self.typedef(cursor);
            }
            CXCursor_StructDecl | CXCursor_UnionDecl => {
                // SAFETY(ffi): a live cursor.
                if unsafe { clang_Cursor_isAnonymous(cursor) } == 0 {
                    let record = self.record(cursor, None);
                    if self.cpp && kind(cursor) == CXCursor_StructDecl {
                        self.cpp_members(cursor, record);
                    }
                }
            }
            CXCursor_EnumDecl => {
                // SAFETY(ffi): as above.
                if unsafe { clang_Cursor_isAnonymous(cursor) } != 0 {
                    self.anonymous_enum(cursor);
                } else {
                    let _ = self.enumeration(cursor, None);
                }
            }
            CXCursor_TypedefDecl => {
                let _ = self.typedef(cursor);
            }
            CXCursor_VarDecl => self.global(cursor),
            CXCursor_MacroDefinition => self.macro_definition(cursor),
            _ => {}
        }
    }

    // -- types --------------------------------------------------------------

    /// `[FFI-8]` — the Ember type of a C type; `Err` names what cannot be represented.
    /// `behind_pointer`: an opaque struct may be named.
    fn ty(&mut self, ty: CXType, behind_pointer: bool) -> Result<String, String> {
        // SAFETY(ffi): types and declarations of a live translation unit.
        unsafe {
            match ty.kind {
                CXType_Void => Ok("void".to_string()),
                CXType_Bool => Ok("bool".to_string()),
                // `std.ffi`'s C aliases, whose types are the target's (D-742).
                CXType_Char_S | CXType_Char_U => Ok("c_char".to_string()),
                CXType_Int => Ok("c_int".to_string()),
                CXType_UInt => Ok("c_uint".to_string()),
                CXType_Long => Ok("c_long".to_string()),
                CXType_ULong => Ok("c_ulong".to_string()),
                CXType_WChar => Ok("c_wchar".to_string()),
                CXType_SChar | CXType_UChar | CXType_Short | CXType_UShort | CXType_LongLong | CXType_ULongLong
                | CXType_Char16 | CXType_Char32 => fixed_integer(ty),
                CXType_Float => Ok("f32".to_string()),
                CXType_Double => Ok("f64".to_string()),
                CXType_LongDouble => Err("`long double` (Ember's `c_longdouble` is not built yet)".to_string()),
                CXType_Int128 | CXType_UInt128 => {
                    Err("a 128-bit integer (`__int128`: its argument passing differs between C compilers)".to_string())
                }
                CXType_Half | CXType_Float16 => Err("`_Float16` (Ember's `f16` crosses as a 16-bit integer)".to_string()),
                CXType_Elaborated => self.ty(clang_Type_getNamedType(ty), behind_pointer),
                CXType_Typedef => {
                    let name = text(clang_getTypedefName(ty));
                    if let Some(fixed) = standard_typedef(&name) {
                        return Ok(fixed.to_string());
                    }
                    let declaration = clang_getTypeDeclaration(ty);
                    let (file, _, system) = location(declaration);
                    if file == self.header || !system {
                        let alias = self.typedef(declaration)?;
                        if !behind_pointer && let Some(Record::Opaque(_, why)) = self.typedef_record(declaration) {
                            return Err(opaque_by_value(&alias, why.as_deref()));
                        }
                        return Ok(self.qualify(alias, declaration));
                    }
                    self.ty(clang_getTypedefDeclUnderlyingType(declaration), behind_pointer)
                }
                CXType_Record => {
                    let declaration = clang_getTypeDeclaration(ty);
                    // `[FFI-17a]` — the C++ standard library's classes cross only as its bridge types.
                    if self.cpp && location(declaration).2 {
                        return Err(format!("`{}`: the C++ standard library's types cross as `[FFI-17a]`'s bridge types, which are not built yet", cpp_spelling(ty)));
                    }
                    // A struct with no tag is known by its typedef's name once that is written
                    // (`typedef struct {…} Rect`, seen through a canonical type).
                    if clang_Cursor_isAnonymous(declaration) != 0 && !self.records.contains_key(&usr(declaration)) {
                        return Err("an anonymous struct or union member (its generated `<Parent>_anon<N>` type is not built yet)".to_string());
                    }
                    match self.record(declaration, None) {
                        Record::Full(name) | Record::Pending(name) => Ok(self.qualify(name, declaration)),
                        Record::Opaque(name, _) if behind_pointer => Ok(self.qualify(name, declaration)),
                        Record::Opaque(name, why) => Err(opaque_by_value(&name, why.as_deref())),
                    }
                }
                CXType_Enum => {
                    let declaration = clang_getTypeDeclaration(ty);
                    if clang_Cursor_isAnonymous(declaration) != 0 {
                        return self.ty(clang_getEnumDeclIntegerType(declaration), false);
                    }
                    let name = self.enumeration(declaration, None)?;
                    Ok(self.qualify(name, declaration))
                }
                CXType_Pointer => {
                    let pointee = clang_getPointeeType(ty);
                    if let Some(function) = self.function_type(pointee)? {
                        return Ok(format!("Option[{function}]"));
                    }
                    let inner = self.ty(pointee, true)?;
                    let constant = clang_isConstQualifiedType(pointee) != 0;
                    Ok(format!("*{}{inner}", if constant { "" } else { "mut " }))
                }
                CXType_ConstantArray => {
                    let element = self.ty(clang_getArrayElementType(ty), false)?;
                    Ok(format!("[{element}; {}]", clang_getArraySize(ty)))
                }
                CXType_IncompleteArray => Err("an array of unknown size (a flexible array member's unsized struct is not built yet)".to_string()),
                CXType_FunctionProto | CXType_FunctionNoProto => Err("a function type that is not a pointer".to_string()),
                // Sugar libclang does not expose, as Clang's own `__size_t` in a builtin's type
                // (`size_t strlen(const char*)`): by its name, else as what it stands for.
                _ if let Some(fixed) = standard_typedef(&type_spelling(ty)) => Ok(fixed.to_string()),
                _ if clang_getCanonicalType(ty).kind != ty.kind => self.ty(clang_getCanonicalType(ty), behind_pointer),
                _ => Err(format!("the C type `{}`", type_spelling(ty))),
            }
        }
    }

    /// The function a C function pointer points to as `extern "C" fn(…) -> R`, if `ty` is a
    /// function type (through typedefs and parentheses).
    fn function_type(&mut self, ty: CXType) -> Result<Option<String>, String> {
        // SAFETY(ffi): types of a live translation unit.
        unsafe {
            let canonical = clang_getCanonicalType(ty);
            match canonical.kind {
                CXType_FunctionProto => {}
                CXType_FunctionNoProto => return Err("a function pointer without a prototype".to_string()),
                _ => return Ok(None),
            }
            if clang_isFunctionTypeVariadic(canonical) != 0 {
                return Err("a variadic function pointer".to_string());
            }
            let abi = convention(clang_getFunctionTypeCallingConv(canonical))
                .ok_or("a function pointer of a calling convention Ember does not have (`[FFI-9]`)")?;
            // The written type keeps its typedef names; a parameter's own is its canonical one.
            let mut params = Vec::new();
            for index in 0..clang_getNumArgTypes(canonical).max(0) as u32 {
                params.push(self.parameter_type(clang_getArgType(canonical, index))?);
            }
            let result = clang_getResultType(canonical);
            let arrow = if result.kind == CXType_Void { String::new() } else { format!(" -> {}", self.ty(result, false)?) };
            Ok(Some(format!("extern \"{abi}\" fn({}){arrow}", params.join(", "))))
        }
    }

    /// A parameter's type: an array decays to a pointer to its element, as in C.
    fn parameter_type(&mut self, ty: CXType) -> Result<String, String> {
        // SAFETY(ffi): types of a live translation unit.
        unsafe {
            let canonical = clang_getCanonicalType(ty);
            if matches!(canonical.kind, CXType_ConstantArray | CXType_IncompleteArray) {
                let element = clang_getArrayElementType(canonical);
                let constant = clang_isConstQualifiedType(element) != 0;
                return Ok(format!("*{}{}", if constant { "" } else { "mut " }, self.ty(element, true)?));
            }
        }
        self.ty(ty, false)
    }

    /// A parameter's or result's C type, with what an overlay's contract words need.
    fn c_type(&mut self, ty: CXType, parameter: bool) -> Result<CType, String> {
        let ember = if parameter { self.parameter_type(ty)? } else { self.ty(ty, false)? };
        // SAFETY(ffi): types of a live translation unit.
        let pointee = unsafe {
            // Through typedefs to the pointer, keeping the names the pointee is written with.
            let mut written = ty;
            while matches!(written.kind, CXType_Typedef | CXType_Elaborated) {
                written = if written.kind == CXType_Typedef {
                    clang_getTypedefDeclUnderlyingType(clang_getTypeDeclaration(written))
                } else {
                    clang_Type_getNamedType(written)
                };
            }
            match written.kind {
                CXType_Pointer if clang_getCanonicalType(clang_getPointeeType(written)).kind != CXType_FunctionProto => {
                    Some(clang_getPointeeType(written))
                }
                CXType_ConstantArray | CXType_IncompleteArray if parameter => Some(clang_getArrayElementType(written)),
                _ => None,
            }
        };
        let c = type_spelling(ty);
        let Some(pointee) = pointee else { return Ok(CType { ember, pointee: None, char_pointer: false, c, mode: String::new() }) };
        // SAFETY(ffi): as above.
        let (constant, character) = unsafe {
            let kind = clang_getCanonicalType(pointee).kind;
            (clang_isConstQualifiedType(pointee) != 0, matches!(kind, CXType_Char_S | CXType_Char_U))
        };
        let inner = self.ty(pointee, true)?;
        Ok(CType { ember, pointee: Some((inner, constant)), char_pointer: character, c, mode: String::new() })
    }

    /// `[FFI-41]` — write a declaration in the module of its own namespace, and remember where.
    fn at_home<T>(&mut self, cursor: CXCursor, write: impl FnOnce(&mut Self) -> T) -> T {
        if !self.cpp {
            return write(self);
        }
        let home = namespace_of(cursor);
        self.homes.insert(usr(cursor), home.clone());
        let saved = std::mem::replace(&mut self.writing, home);
        let out = write(self);
        self.writing = saved;
        out
    }

    /// `[FFI-41]` — a type's name as the namespace being written names it: through the module of
    /// the namespace it is in, when that is another.
    fn qualify(&mut self, name: String, declaration: CXCursor) -> String {
        if !self.cpp || name.contains('.') {
            return name;
        }
        let Some(home) = self.homes.get(&usr(declaration)).cloned() else { return name };
        if home == self.writing {
            return name;
        }
        let used = (self.writing.clone(), home.clone());
        if !self.ns_uses.contains(&used) {
            self.ns_uses.push(used);
        }
        format!("{}.{name}", namespace_alias(&home))
    }

    fn typedef(&mut self, cursor: CXCursor) -> Result<String, String> {
        self.at_home(cursor, |writer| writer.typedef_here(cursor))
    }

    fn typedef_here(&mut self, cursor: CXCursor) -> Result<String, String> {
        let key = usr(cursor);
        if let Some(done) = self.typedefs.get(&key) {
            return done.clone();
        }
        let name = spelling(cursor);
        if let Some(fixed) = standard_typedef(&name) {
            return Ok(fixed.to_string());
        }
        // A pointer to the alias inside its own definition names it.
        self.typedefs.insert(key.clone(), Ok(ident(&name)));
        let outcome = self.typedef_target(cursor, &name);
        let outcome = match outcome {
            Ok(None) => Ok(ident(&name)),
            Ok(Some(target)) if self.claim(&name) => {
                self.push(&name, Kind::Type, format!("type {} = {target}", ident(&name)));
                self.spelled(cursor, &name);
                Ok(ident(&name))
            }
            Ok(Some(_)) => {
                let reason = "its name is taken by another declaration of the header";
                self.skip(cursor, &name, reason);
                Err(format!("`{name}`, not imported: {reason}"))
            }
            Err(reason) => {
                self.skip(cursor, &name, reason.clone());
                Err(format!("`{name}`, not imported: {reason}"))
            }
        };
        self.typedefs.insert(key, outcome.clone());
        outcome
    }

    /// `None` when the typedef names a struct or enum that takes its name; else the alias's target.
    fn typedef_target(&mut self, cursor: CXCursor, name: &str) -> Result<Option<String>, String> {
        // SAFETY(ffi): a live typedef cursor.
        unsafe {
            let underlying = clang_getTypedefDeclUnderlyingType(cursor);
            let named = if underlying.kind == CXType_Elaborated { clang_Type_getNamedType(underlying) } else { underlying };
            if matches!(named.kind, CXType_Record | CXType_Enum) {
                let declaration = clang_getTypeDeclaration(named);
                let anonymous = clang_Cursor_isAnonymous(declaration) != 0;
                let same = spelling(declaration) == name;
                if named.kind == CXType_Record {
                    if anonymous || same {
                        self.record(declaration, Some(name));
                        return Ok(None);
                    }
                    let target = match self.record(declaration, None) {
                        Record::Full(target) | Record::Pending(target) | Record::Opaque(target, _) => target,
                    };
                    return Ok(Some(target));
                }
                if anonymous || same {
                    self.enumeration(declaration, Some(name))?;
                    return Ok(None);
                }
            }
            if let Some(function) = self.function_type(underlying)? {
                return Err(format!("a function type `{function}` that is not a pointer"));
            }
            if underlying.kind == CXType_Pointer
                && let Some(function) = self.function_type(clang_getPointeeType(underlying))?
            {
                return Ok(Some(format!("Option[{function}]")));
            }
            self.ty(underlying, true).map(Some)
        }
    }

    /// The record a typedef names, if it names one.
    fn typedef_record(&self, cursor: CXCursor) -> Option<Record> {
        // SAFETY(ffi): a live typedef cursor.
        unsafe {
            let underlying = clang_getCanonicalType(clang_getTypedefDeclUnderlyingType(cursor));
            (underlying.kind == CXType_Record).then(|| self.records.get(&usr(clang_getTypeDeclaration(underlying))).cloned()).flatten()
        }
    }

    /// `[FFI-8]` — a complete struct is `@layout(c) struct`, `Copy` (every field is plain data); an
    /// incomplete one an `extern type`. `[FFI-5]` — Clang's layout is recorded.
    fn record(&mut self, cursor: CXCursor, typedef_name: Option<&str>) -> Record {
        self.at_home(cursor, |writer| writer.record_here(cursor, typedef_name))
    }

    fn record_here(&mut self, cursor: CXCursor, typedef_name: Option<&str>) -> Record {
        let key = usr(cursor);
        if let Some(done) = self.records.get(&key) {
            return done.clone();
        }
        // SAFETY(ffi): cursors and types of a live translation unit.
        unsafe {
            let tag = spelling(cursor);
            let named = typedef_name.map_or(tag.clone(), str::to_string);
            let name = ident(&named);
            let definition = clang_getCursorDefinition(cursor);
            if clang_Cursor_isNull(definition) != 0 {
                return self.opaque(cursor, &key, &named, None);
            }
            if kind(definition) == CXCursor_UnionDecl {
                return self.opaque(cursor, &key, &named, Some("a C union (Ember's `union` is reserved for a later version)".to_string()));
            }
            // `[FFI-48]` — multiple inheritance and virtual bases: `E5034` where the class is named.
            if self.cpp {
                let bases: Vec<CXCursor> = children(definition).into_iter().filter(|child| kind(*child) == CXCursor_CXXBaseSpecifier).collect();
                if bases.len() > 1 || bases.iter().any(|base| clang_isVirtualBase(*base) != 0) {
                    let why = "multiple inheritance, or a virtual base: the pointer adjustment is ABI-private and differs between MSVC and Itanium (write a single-inheritance facade in C++)";
                    self.skip(cursor, &named, format!("[E5034] {why}"));
                    let record = Record::Opaque(name, Some(why.to_string()));
                    self.records.insert(key, record.clone());
                    return record;
                }
            }
            // `[FFI-32]` — a C++ class is a struct only when it is standard-layout and trivially
            // copyable; any other is opaque behind a pointer, made only by its constructors.
            if self.cpp && !self.by_value.get(&key).copied().unwrap_or(false) {
                return self.opaque(cursor, &key, &named, None);
            }
            self.records.insert(key.clone(), Record::Pending(name.clone()));
            let ty = clang_getCursorType(definition);
            let mut fields = Vec::new();
            let mut layout = Vec::new();
            let mut natural = 1u64;
            let mut packed = false;
            let mut plain = Vec::new();
            let mut bits = Vec::new();
            let mut anonymous = 0;
            let mut members_seen = HashSet::new();
            // A class template's specialization shows its fields only through its type.
            let specialized = self.cpp && clang_Cursor_isNull(clang_getSpecializedCursorTemplate(definition)) == 0;
            let members = if specialized { record_fields(ty) } else { children(definition) };
            for child in members {
                match kind(child) {
                    CXCursor_PackedAttr => packed = true,
                    // A record declared inside is a member only when it is an anonymous one.
                    CXCursor_StructDecl | CXCursor_UnionDecl if clang_Cursor_isAnonymousRecordDecl(child) == 0 => {}
                    CXCursor_FieldDecl | CXCursor_StructDecl | CXCursor_UnionDecl => {
                        let field = if kind(child) == CXCursor_FieldDecl { spelling(child) } else { String::new() };
                        if clang_Cursor_isBitField(child) != 0 {
                            // `[FFI-8]` — read and written by `get_<f>`/`set_<f>`; an unnamed one is padding.
                            let width = clang_getFieldDeclBitWidth(child).max(0) as u64;
                            if field.is_empty() || width == 0 {
                                continue;
                            }
                            let field_ty = clang_getCursorType(child);
                            let canonical = clang_getCanonicalType(field_ty).kind;
                            let boolean = canonical == CXType_Bool;
                            let integer = matches!(canonical, CXType_Char_S | CXType_SChar | CXType_Short | CXType_Int | CXType_Long | CXType_LongLong
                                | CXType_Char_U | CXType_UChar | CXType_UShort | CXType_UInt | CXType_ULong | CXType_ULongLong);
                            let mapped = match self.ty(field_ty, false) {
                                Ok(mapped) if boolean || integer => mapped,
                                _ => return self.opaque(cursor, &key, &named, Some(format!("bit-field `{field}` of type `{}` (only integer and `bool` bit-fields have accessors)", type_spelling(field_ty)))),
                            };
                            let signed = matches!(canonical, CXType_Char_S | CXType_SChar | CXType_Short | CXType_Int | CXType_Long | CXType_LongLong);
                            let offset = clang_Cursor_getOffsetOfField(child).max(0) as u64;
                            bits.push(BitField { name: ident(&field), offset, width, ty: mapped, signed, boolean });
                            continue;
                        }
                        let field_ty = clang_getCursorType(child);
                        let member = if kind(child) == CXCursor_FieldDecl { clang_getTypeDeclaration(field_ty) } else { child };
                        // `[FFI-8]` — an anonymous member is a field `_anon<N>` of a generated type
                        // `<Parent>_anon<N>`, whose fields read through it (`p.x` is `p._anon0.x`).
                        // libclang shows it as the record, or as an unnamed field too: once.
                        let anonymous_member = field.is_empty() || clang_Cursor_isAnonymousRecordDecl(member) != 0;
                        if anonymous_member && !members_seen.insert(usr(member)) {
                            continue;
                        }
                        let offset = if anonymous_member {
                            match anonymous_offset(ty, member) {
                                Some(offset) => offset,
                                None => return self.opaque(cursor, &key, &named, Some("an anonymous member with no named field".to_string())),
                            }
                        } else {
                            (clang_Cursor_getOffsetOfField(child).max(0) as u64) / 8
                        };
                        let (field, mapped) = if anonymous_member {
                            if kind(member) == CXCursor_UnionDecl {
                                return self.opaque(cursor, &key, &named, Some("an anonymous union member (Ember's `union` is reserved for a later version)".to_string()));
                            }
                            let generated = format!("{named}_anon{anonymous}");
                            self.anonymous.insert(generated.clone());
                            match self.record(member, Some(&generated)) {
                                Record::Full(_) => {}
                                Record::Pending(_) | Record::Opaque(..) => {
                                    return self.opaque(cursor, &key, &named, Some(format!("its anonymous member `{generated}` cannot be represented")));
                                }
                            }
                            anonymous += 1;
                            (format!("_anon{}", anonymous - 1), ident(&generated))
                        } else {
                            match self.ty(field_ty, false) {
                                Ok(mapped) => (field, mapped),
                                Err(reason) => return self.opaque(cursor, &key, &named, Some(format!("field `{field}`: {reason}"))),
                            }
                        };
                        let field_align = clang_Type_getAlignOf(field_ty).max(1) as u64;
                        natural = natural.max(field_align);
                        let line = format!("    pub {}: {mapped}\n", ident(&field));
                        layout.push((ident(&field), offset));
                        fields.push(line.clone());
                        let size = clang_Type_getSizeOf(field_ty).max(0) as u64;
                        plain.push(Plain { name: ident(&field), offset, size, align: field_align, line });
                    }
                    _ => {}
                }
            }
            let size = clang_Type_getSizeOf(ty).max(0) as u64;
            let align = clang_Type_getAlignOf(ty).max(1) as u64;
            // The fields C has, whose offsets the shim checks; with bit-fields, the members Ember
            // lays out beside them and the accessors.
            let c_fields = layout.clone();
            let mut accessors = String::new();
            if !bits.is_empty() {
                match bit_field_members(&name, &plain, &bits, size, align, packed) {
                    Ok((lines, members, methods)) => (fields, layout, accessors) = (lines, members, methods),
                    Err(why) => return self.opaque(cursor, &key, &named, Some(why)),
                }
            }
            if fields.is_empty() {
                // A C++ class with no data is an opaque one: its members are what it has.
                let why = (!self.cpp).then(|| "a struct with no fields (a GNU C extension)".to_string());
                return self.opaque(cursor, &key, &named, why);
            }
            let fingerprint = format!("struct {size} {align} {:?}", layout.iter().map(|(_, offset)| offset).collect::<Vec<_>>());
            if let Some(adopted) = self.adopt(cursor, &key, &named, &fingerprint, false) {
                return adopted;
            }
            if !self.claim(&named) {
                return self.opaque_unclaimed(cursor, &key, &named);
            }
            packed |= align == 1 && natural > 1;
            let mut attributes = String::from("@layout(c)\n@derive(Copy)\n");
            if packed {
                attributes.push_str("@packed\n");
            }
            if align > if packed { 1 } else { natural } {
                let _ = writeln!(attributes, "@align({align})");
            }
            self.push(&named, Kind::Type, format!("{attributes}struct {name}:\n{}{accessors}", fields.concat()));
            self.spelled(cursor, &named);
            // `[FFI-5a]` — the project's C compiler checks the layout on every build.
            // `struct Point`, or the typedef name of a struct with no tag of its own.
            let spelled = type_spelling(ty);
            let c_name = if spelled.contains('(') { named.clone() } else { spelled };
            // An anonymous member's type has no C name: its parent's checks cover its bytes.
            if !self.anonymous.contains(&named) {
                let _ = writeln!(self.shim, "_Static_assert(sizeof({c_name}) == {size}, \"{named}: size\");");
                let _ = writeln!(self.shim, "_Static_assert(_Alignof({c_name}) == {align}, \"{named}: alignment\");");
                for (field, offset) in c_fields.iter().filter(|(field, _)| !field.starts_with("_anon")) {
                    let c_field = field.trim_start_matches("r#");
                    let _ = writeln!(self.shim, "_Static_assert(offsetof({c_name}, {c_field}) == {offset}, \"{named}.{c_field}: offset\");");
                }
            }
            self.layouts.push(RecordLayout { name: name.clone(), size, align, fields: layout, namespace: self.writing.clone() });
            self.entities.push((key.clone(), name.clone(), fingerprint));
            self.records.insert(key, Record::Full(name.clone()));
            Record::Full(name)
        }
    }

    /// `[FFI-30]` — a declaration an earlier import of the build wrote with this layout is that
    /// import's entity: here an alias of it. With another layout it is a distinct entity, `E5011`
    /// unless this import binds a name of its own. `any`: an incomplete struct takes any layout.
    fn adopt(&mut self, cursor: CXCursor, key: &str, named: &str, layout: &str, any: bool) -> Option<Record> {
        let shared = self.shared.get(key)?.clone();
        if !any && shared.layout != layout {
            if shared.binding == self.binding && shared.layout != "opaque" {
                self.conflicts.push((named.to_string(), shared.header.clone()));
            }
            return None;
        }
        if !self.claim(named) {
            return None;
        }
        if !self.uses.contains(&shared.module) {
            self.uses.push(shared.module.clone());
        }
        let alias = format!("{}_{}.{}", ember_branding::SYMBOL_PREFIX, shared.module.join("_"), shared.name);
        let name = ident(named);
        self.push(named, Kind::Type, format!("type {name} = {alias}"));
        self.spelled(cursor, named);
        let record = if shared.layout == "opaque" { Record::Opaque(name, None) } else { Record::Full(name) };
        self.records.insert(key.to_string(), record.clone());
        Some(record)
    }

    /// An `extern type`: used only behind pointers. A definition it could not represent is
    /// `W5002`; an incomplete struct is represented exactly.
    fn opaque(&mut self, cursor: CXCursor, key: &str, named: &str, why: Option<String>) -> Record {
        // An incomplete struct is the entity another import wrote, whatever its layout there.
        if why.is_none()
            && let Some(adopted) = self.adopt(cursor, key, named, "", true)
        {
            return adopted;
        }
        if !self.claim(named) {
            return self.opaque_unclaimed(cursor, key, named);
        }
        let name = ident(named);
        self.push(named, Kind::Opaque, format!("type {name}"));
        self.spelled(cursor, named);
        if why.is_none() {
            self.entities.push((key.to_string(), name.clone(), "opaque".to_string()));
        }
        if let Some(why) = &why {
            let at = self.at(cursor);
            self.opaque.push(Skipped { name: named.to_string(), reason: why.clone(), at });
        }
        let record = Record::Opaque(name, why);
        self.records.insert(key.to_string(), record.clone());
        record
    }

    fn opaque_unclaimed(&mut self, cursor: CXCursor, key: &str, named: &str) -> Record {
        let why = "its name is taken by another declaration of the header (C keeps struct tags apart; Ember has one namespace)";
        self.skip(cursor, named, why);
        let record = Record::Opaque(ident(named), Some(why.to_string()));
        self.records.insert(key.to_string(), record.clone());
        record
    }

    /// `[FFI-8]` — `@repr(<underlying>) enum`, `@non_exhaustive`: any value may arrive from C. A
    /// second name for a value already named is a constant of the enum.
    fn enumeration(&mut self, cursor: CXCursor, typedef_name: Option<&str>) -> Result<String, String> {
        self.at_home(cursor, |writer| writer.enumeration_here(cursor, typedef_name))
    }

    fn enumeration_here(&mut self, cursor: CXCursor, typedef_name: Option<&str>) -> Result<String, String> {
        let key = usr(cursor);
        if let Some(done) = self.enums.get(&key) {
            return done.clone();
        }
        let named = typedef_name.map_or_else(|| spelling(cursor), str::to_string);
        let name = ident(&named);
        // SAFETY(ffi): a live enum cursor. `@repr` takes a fixed-width type.
        let repr = fixed_integer(unsafe { clang_getEnumDeclIntegerType(cursor) });
        let layout = |repr: &str| {
            let values: Vec<i64> = children(cursor)
                .into_iter()
                .filter(|child| kind(*child) == CXCursor_EnumConstantDecl)
                // SAFETY(ffi): a live enum constant.
                .map(|child| unsafe { clang_getEnumConstantDeclValue(child) })
                .collect();
            format!("enum {repr} {values:?}")
        };
        // `[FFI-30]` — the enum an earlier import wrote, with these values, is that one.
        if let Ok(repr) = &repr
            && self.adopt(cursor, &key, &named, &layout(repr), false).is_some()
        {
            self.enums.insert(key, Ok(name.clone()));
            return Ok(name);
        }
        let outcome = match repr {
            Err(reason) => Err(reason),
            Ok(_) if !self.claim(&named) => Err("its name is taken by another declaration of the header".to_string()),
            Ok(repr) => {
                let unsigned = repr.starts_with('u');
                let mut variants = String::new();
                let mut aliases = Vec::new();
                let mut values: HashMap<i128, String> = HashMap::new();
                for child in children(cursor).into_iter().filter(|child| kind(*child) == CXCursor_EnumConstantDecl) {
                    let constant = ident(&spelling(child));
                    // SAFETY(ffi): a live enum constant.
                    let value = unsafe {
                        if unsigned { clang_getEnumConstantDeclUnsignedValue(child) as i128 } else { clang_getEnumConstantDeclValue(child) as i128 }
                    };
                    match values.get(&value) {
                        Some(first) => aliases.push((spelling(child), format!("const {constant}: {name} = {name}.{first}"))),
                        None => {
                            values.insert(value, constant.clone());
                            let _ = writeln!(variants, "    {constant} = {value}");
                        }
                    }
                }
                if variants.is_empty() {
                    variants.push_str("    pass\n");
                }
                self.push(&named, Kind::Type, format!("@repr({repr})\n@non_exhaustive\nenum {name}:\n{variants}"));
                self.spelled(cursor, &named);
                self.entities.push((key.clone(), name.clone(), layout(&repr)));
                for (alias, text) in aliases {
                    if self.claim(&alias) {
                        self.push(&alias, Kind::Const, text);
                    }
                }
                Ok(name)
            }
        };
        if let Err(reason) = &outcome {
            self.skip(cursor, &named, reason.clone());
        }
        let outcome = outcome.map_err(|reason| format!("`{named}`, not imported: {reason}"));
        self.enums.insert(key, outcome.clone());
        outcome
    }

    /// An enum with no name: its constants, each of the enum's integer type.
    fn anonymous_enum(&mut self, cursor: CXCursor) {
        if !self.seen.insert(usr(cursor) + &self.at(cursor)) {
            return;
        }
        // SAFETY(ffi): a live enum cursor.
        let repr = match self.ty(unsafe { clang_getEnumDeclIntegerType(cursor) }, false) {
            Ok(repr) => repr,
            Err(reason) => return self.skip(cursor, "enum", reason),
        };
        for child in children(cursor).into_iter().filter(|child| kind(*child) == CXCursor_EnumConstantDecl) {
            let constant = spelling(child);
            if !self.claim(&constant) {
                self.skip(child, &constant, "its name is taken by another declaration of the header");
                continue;
            }
            // SAFETY(ffi): a live enum constant.
            let value = unsafe {
                if repr.starts_with('u') { clang_getEnumConstantDeclUnsignedValue(child) as i128 } else { clang_getEnumConstantDeclValue(child) as i128 }
            };
            self.push(&constant, Kind::Const, format!("const {}: {repr} = {value}", ident(&constant)));
        }
    }

    // -- functions and globals ----------------------------------------------

    fn function(&mut self, cursor: CXCursor) {
        let name = spelling(cursor);
        // A C++ function is known by its USR: an overload is another function of one name.
        let key = if self.cpp { usr(cursor) } else { name.clone() };
        if !self.seen.insert(format!("fn {key}")) {
            return;
        }
        if self.cpp && self.functions.iter().any(|function| function.name == name && function.owner.is_none() && function.namespace == self.writing) {
            self.skip(cursor, &name, "an overload: Ember has no overloading, and the first declaration of the name is the one imported");
            return;
        }
        // `[FFI-3]` — a C++ function, never its mangled symbol: an `extern "C"` thunk, unless the
        // header declares it `extern "C"` itself (`[FFI-44]`: Part XVI's).
        // SAFETY(ffi): a live function cursor.
        let c_linkage = unsafe { text(clang_Cursor_getMangling(cursor)) } == name;
        let parts = if self.cpp && !c_linkage { self.cpp_function(cursor, &name) } else { self.function_parts(cursor, &name) };
        match parts {
            Ok(function) => {
                if !self.claim(&name) {
                    self.skip(cursor, &name, "its name is taken by another declaration of the header");
                    return;
                }
                self.functions.push(Function { namespace: self.writing.clone(), ..function });
            }
            Err(reason) => self.skip(cursor, &name, reason),
        }
    }

    fn function_parts(&mut self, cursor: CXCursor, name: &str) -> Result<Function, String> {
        // SAFETY(ffi): a live function cursor.
        unsafe {
            let ty = clang_getCursorType(cursor);
            if ty.kind == CXType_FunctionNoProto {
                return Err("declared without a prototype (`f()` in C before C23 takes any arguments)".to_string());
            }
            if clang_isFunctionTypeVariadic(ty) != 0 {
                return Err("variadic (`...`): Ember's variadic foreign calls are not built yet".to_string());
            }
            let abi = convention(clang_getFunctionTypeCallingConv(ty))
                .ok_or("its calling convention is not one Ember has (`[FFI-9]`)")?;
            let mut params = Vec::new();
            for index in 0..clang_Cursor_getNumArguments(cursor).max(0) as u32 {
                let argument = clang_Cursor_getArgument(cursor, index);
                let written = spelling(argument);
                let param = if written.is_empty() { format!("_{index}") } else { ident(&written) };
                let c_type = self.c_type(clang_getCursorType(argument), true).map_err(|reason| format!("parameter `{param}`: {reason}"))?;
                params.push((param, c_type));
            }
            let result = clang_getResultType(ty);
            let result = if result.kind == CXType_Void { None } else { Some(self.c_type(result, false).map_err(|reason| format!("its result: {reason}"))?) };
            // `[FFI-29a]` — a function with internal linkage is reached through the shim's wrapper, as
            // is a C++ header's `extern "C"` inline one, whose symbol only a C++ file that uses it has.
            let inline_cpp = self.cpp && clang_Cursor_isFunctionInlined(cursor) != 0;
            let link_name = (clang_getCursorLinkage(cursor) == CXLinkage_Internal || inline_cpp).then(|| {
                let wrapper = format!("{}_cshim_{}_{name}", ember_branding::SYMBOL_PREFIX, self.stem);
                self.wrapper(cursor, &wrapper, name);
                wrapper
            });
            Ok(Function { name: name.to_string(), params, result, link_name, abi, policy: Policy::Direct, owner: None, out: false, namespace: Vec::new() })
        }
    }

    /// `[FFI-17]`, `[FFI-3]`, `[FFI-24]`, `[FFI-40]` — a C++ function: an `extern "C"` thunk in the
    /// shim calls it, by its qualified name. Its exception policy is derived (`policy`); a thunk of a
    /// function that may throw catches every exception, sets its last argument, and keeps the
    /// exception's type name and `what()` for the Ember side's `CppError`. A `T&` parameter is
    /// passed by address (`mut T`, or `ref mut T` for an opaque class); a `const T&` as a borrowed
    /// value (`T`, or `ref T`); a `T&&` as an owned value, moved in; a `T&` result as a raw pointer.
    fn cpp_function(&mut self, cursor: CXCursor, name: &str) -> Result<Function, String> {
        self.cpp_thunk(cursor, name, None)
    }

    /// `[FFI-40]`, `[FFI-41]`, `[FFI-32]` — a C++ class's public members, each through a thunk: its
    /// methods (`const` ones as `self`, unless the class has a `mutable` member, `[FFI-40a]`), its
    /// static member functions, its first constructor (not a copy or move one, nor an abstract
    /// class's), and, for an opaque class, its destructor, which the type's `ForeignBox` calls
    /// (`@ffi(destroys(…))`, `[FFI-36a]`). An `&&`-qualified member is `W5033`; an operator and an
    /// overload are skipped (`W5002`).
    fn cpp_members(&mut self, class: CXCursor, record: Record) {
        let (owner, by_value) = match record {
            Record::Full(name) => (name, true),
            Record::Opaque(name, None) => (name, false),
            _ => return,
        };
        // SAFETY(ffi): cursors and types of a live translation unit.
        unsafe {
            let definition = clang_getCursorDefinition(class);
            if clang_Cursor_isNull(definition) != 0 || !self.seen.insert(format!("members {}", usr(definition))) {
                return;
            }
            let class_spelled = cpp_spelling(clang_getCursorType(definition));
            let abstract_class = clang_CXXRecord_isAbstract(definition) != 0;
            let mutable_state = children(definition).into_iter().any(|child| kind(child) == CXCursor_FieldDecl && clang_CXXField_isMutable(child) != 0);
            // An opaque class's object is freed by its destructor: the one it declares, when that
            // is public, else the implicit one, which is.
            let declared = children(definition).into_iter().find(|child| kind(*child) == CXCursor_Destructor);
            let destructible = by_value
                || declared.is_none_or(|destructor| clang_getCXXAccessSpecifier(destructor) == CX_CXXPublic && clang_getCursorAvailability(destructor) != CXAvailability_NotAvailable);
            if !by_value && destructible {
                let destroys = format!("{}_cpp_delete_{owner}", ember_branding::SYMBOL_PREFIX);
                let thunk = format!("{}_cpp_{}_{}", ember_branding::SYMBOL_PREFIX, self.stem, self.thunks);
                self.thunks += 1;
                let _ = writeln!(self.shim, "extern \"C\" void {thunk}({class_spelled}* self) noexcept {{ delete self; }}");
                if let Some(item) = self.items.iter_mut().find(|item| ident(&item.name) == owner && item.kind == Kind::Opaque) {
                    item.text = format!("@ffi(destroys({destroys}))\n{}", item.text);
                }
                let owner = Some((owner.clone(), Member::Destructor));
                self.functions.push(Function { name: "drop".to_string(), params: Vec::new(), result: None, link_name: Some(thunk), abi: "C", policy: Policy::Direct, owner, out: false, namespace: self.writing.clone() });
            }
            let mut taken: HashSet<String> = HashSet::new();
            for child in children(definition) {
                if clang_getCXXAccessSpecifier(child) != CX_CXXPublic || clang_getCursorAvailability(child) == CXAvailability_NotAvailable {
                    continue;
                }
                let name = spelling(child);
                let member = match kind(child) {
                    CXCursor_Constructor => {
                        if abstract_class || clang_CXXConstructor_isCopyConstructor(child) != 0 || clang_CXXConstructor_isMoveConstructor(child) != 0 {
                            continue;
                        }
                        if !destructible {
                            self.skip(child, &format!("{owner}.new"), "a constructor of a class whose destructor is not public: no `ForeignBox` could free what it makes");
                            continue;
                        }
                        if by_value { Member::Constructor } else { Member::BoxedConstructor }
                    }
                    CXCursor_CXXMethod => {
                        if name.starts_with("operator") {
                            self.skip(child, &format!("{owner}.{name}"), "an operator (`[FFI-42]`'s operator interfaces are not built yet)");
                            continue;
                        }
                        if clang_Type_getCXXRefQualifier(clang_getCursorType(child)) == CXRefQualifier_RValue {
                            let at = self.at(child);
                            self.rvalue_members.push(Skipped { name: format!("{owner}.{name}"), reason: "an `&&`-qualified member function, which only a temporary may call".to_string(), at });
                            continue;
                        }
                        if clang_CXXMethod_isStatic(child) != 0 {
                            Member::Static
                        } else if clang_CXXMethod_isConst(child) != 0 && !mutable_state {
                            Member::Method
                        } else {
                            Member::MutMethod
                        }
                    }
                    _ => continue,
                };
                let key = match member {
                    Member::Constructor | Member::BoxedConstructor => "new".to_string(),
                    Member::Destructor => "drop".to_string(),
                    _ => name.clone(),
                };
                if !taken.insert(key.clone()) {
                    self.skip(child, &format!("{owner}.{key}"), "an overload: Ember has no overloading, and the first declaration of the name is the one imported");
                    continue;
                }
                match self.cpp_thunk(child, &key, Some((&owner, member, &class_spelled))) {
                    Ok(function) => self.functions.push(Function { namespace: self.writing.clone(), ..function }),
                    Err(reason) => self.skip(child, &format!("{owner}.{key}"), reason),
                }
            }
        }
    }

    /// A thunk for a C++ function or member (`member`: its class's Ember name, what it is, and the
    /// class as C++ spells it).
    fn cpp_thunk(&mut self, cursor: CXCursor, name: &str, member: Option<(&str, Member, &str)>) -> Result<Function, String> {
        // SAFETY(ffi): a live function cursor and its types.
        unsafe {
            let ty = clang_getCursorType(cursor);
            if clang_isFunctionTypeVariadic(ty) != 0 {
                return Err("variadic (`...`): Ember's variadic foreign calls are not built yet".to_string());
            }
            let mut params = Vec::new();
            let mut declared = Vec::new();
            let mut passed = Vec::new();
            for index in 0..clang_Cursor_getNumArguments(cursor).max(0) as u32 {
                let argument = clang_Cursor_getArgument(cursor, index);
                let written = spelling(argument);
                let param = if written.is_empty() { format!("_{index}") } else { ident(&written) };
                let (c_type, declaration, pass) = self
                    .cpp_parameter(clang_getCursorType(argument), &format!("a{index}"))
                    .map_err(|reason| format!("[E5031] parameter `{param}`: {reason}"))?;
                params.push((param, c_type));
                declared.push(declaration);
                passed.push(pass);
            }
            let policy = self.policy(cursor);
            let result_ty = clang_getResultType(ty);
            let arguments = passed.join(", ");
            let call = match member {
                None => format!("{}({arguments})", self.callee.clone().unwrap_or_else(|| qualified_name(cursor))),
                Some((_, Member::Static, _)) => format!("{}({arguments})", qualified_name(cursor)),
                Some((_, Member::Method | Member::MutMethod, _)) => format!("self->{}({arguments})", spelling(cursor)),
                Some((_, Member::Constructor, class)) => format!("{class}({arguments})"),
                Some((_, Member::BoxedConstructor, class)) => format!("new {class}({arguments})"),
                Some((_, Member::Destructor, _)) => "delete self".to_string(),
            };
            match member {
                Some((_, Member::Method, class)) => declared.insert(0, format!("const {class}* self")),
                Some((_, Member::MutMethod | Member::Destructor, class)) => declared.insert(0, format!("{class}* self")),
                _ => {}
            }
            let (result, returned, call) = match (member, result_ty.kind) {
                (Some((owner, Member::Constructor, class)), _) => {
                    let ember = CType { ember: owner.to_string(), pointee: None, char_pointer: false, c: class.to_string(), mode: String::new() };
                    (Some(ember), Some(class.to_string()), call)
                }
                (Some((owner, Member::BoxedConstructor, class)), _) => {
                    let ember = CType { ember: format!("*mut {owner}"), pointee: Some((owner.to_string(), false)), char_pointer: false, c: format!("{class}*"), mode: String::new() };
                    (Some(ember), Some(format!("{class}*")), call)
                }
                (_, CXType_Void) => (None, None, call),
                (_, CXType_LValueReference) => {
                    let pointee = clang_getPointeeType(result_ty);
                    let constant = clang_isConstQualifiedType(pointee) != 0;
                    let inner = self.ty(pointee, true).map_err(|reason| format!("its result: {reason}"))?;
                    let pointer = format!("*{}{inner}", if constant { "" } else { "mut " });
                    let c = type_spelling(result_ty);
                    let ember = CType { ember: pointer, pointee: Some((inner, constant)), char_pointer: false, c, mode: String::new() };
                    (Some(ember), Some(format!("{}*", cpp_spelling(pointee))), format!("&{call}"))
                }
                _ => {
                    let mapped = self.c_type(result_ty, false).map_err(|reason| format!("its result: {reason}"))?;
                    (Some(mapped), Some(cpp_spelling(result_ty)), call)
                }
            };
            // A special member the class defaults is non-throwing.
            let policy = if member.is_some() && clang_CXXMethod_isDefaulted(cursor) != 0 { Policy::Direct } else { policy };
            let thunk = format!("{}_cpp_{}_{}", ember_branding::SYMBOL_PREFIX, self.stem, self.thunks);
            self.thunks += 1;
            let catching = policy != Policy::Direct;
            // A class by value is written through a pointer the caller gives: MSVC returns a class
            // with a constructor in memory where C returns a struct of its size in registers.
            let class_result = matches!(member, Some((_, Member::Constructor, _)))
                || (result.is_some() && clang_getCanonicalType(result_ty).kind == CXType_Record && !matches!(member, Some((_, Member::BoxedConstructor, _))));
            let (returned, call) = match (class_result, returned) {
                (true, Some(class)) => {
                    declared.push(format!("{class}* out"));
                    let made = if matches!(member, Some((_, Member::Constructor, _))) { call } else { format!("{class}({call})") };
                    (None, format!("::new (static_cast<void*>(out)) {made}"))
                }
                (_, returned) => (returned, call),
            };
            if catching {
                declared.push("int* failed".to_string());
            }
            let declared = if declared.is_empty() { "void".to_string() } else { declared.join(", ") };
            let cleared = if class_result { " std::memset(static_cast<void*>(out), 0, sizeof *out);" } else { "" };
            let body = match (&returned, catching) {
                (None, false) => format!("{call};"),
                (Some(_), false) => format!("return {call};"),
                (None, true) => format!("try {{ *failed = 0; {call}; }} catch (...) {{ *failed = 1; caught();{cleared} }}"),
                (Some(returned), true) => format!("try {{ *failed = 0; return {call}; }} catch (...) {{ *failed = 1; caught(); return zero<{returned}>(); }}"),
            };
            let returned = returned.unwrap_or_else(|| "void".to_string());
            let _ = writeln!(self.shim, "extern \"C\" {returned} {thunk}({declared}) noexcept {{ {body} }}");
            let owner = member.map(|(owner, member, _)| (owner.to_string(), member));
            Ok(Function { name: name.to_string(), params, result, link_name: Some(thunk), abi: "C", policy, owner, out: class_result, namespace: Vec::new() })
        }
    }

    /// `[FFI-40]` — a C++ parameter: its Ember type and mode, the thunk's declaration of it, and the
    /// expression the thunk passes on.
    fn cpp_parameter(&mut self, ty: CXType, name: &str) -> Result<(CType, String, String), String> {
        // SAFETY(ffi): types of a live translation unit.
        unsafe {
            let c = type_spelling(ty);
            let plain = |ember: String, mode: &str| CType { ember, pointee: None, char_pointer: false, c: c.clone(), mode: mode.to_string() };
            match ty.kind {
                CXType_LValueReference => {
                    let pointee = clang_getPointeeType(ty);
                    let constant = clang_isConstQualifiedType(pointee) != 0;
                    let by_value = self.ty(pointee, false);
                    let spelled = cpp_spelling(pointee);
                    Ok(match (constant, by_value) {
                        (true, Ok(mapped)) => (plain(mapped, ""), format!("{spelled} {name}"), name.to_string()),
                        (true, Err(_)) => (plain(format!("*{}", self.ty(pointee, true)?), ""), format!("{spelled}* {name}"), format!("*{name}")),
                        (false, Ok(mapped)) => (plain(mapped, "mut "), format!("{spelled}* {name}"), format!("*{name}")),
                        (false, Err(_)) => (plain(format!("*mut {}", self.ty(pointee, true)?), ""), format!("{spelled}* {name}"), format!("*{name}")),
                    })
                }
                CXType_RValueReference => {
                    let pointee = clang_getPointeeType(ty);
                    let mapped = self.ty(pointee, false).map_err(|reason| format!("an rvalue reference to a type that does not map by value: {reason}"))?;
                    Ok((plain(mapped, "owned "), format!("{} {name}", cpp_spelling(pointee)), format!("std::move({name})")))
                }
                _ => {
                    let mapped = self.c_type(ty, true)?;
                    Ok((mapped, c_declaration(clang_getCanonicalType(ty), name), name.to_string()))
                }
            }
        }
    }

    /// `[FFI-17b]` — an explicit instantiation `instantiate=[…]` lists: a class template's
    /// specialization as a class of the module, or a function template's as a function.
    fn instantiation(&mut self, cursor: CXCursor, written: &str) {
        let name = instance_name(written);
        // SAFETY(ffi): cursors and types of a live translation unit.
        unsafe {
            if kind(cursor) == CXCursor_TypeAliasDecl {
                let class = clang_getTypeDeclaration(clang_getCanonicalType(clang_getTypedefDeclUnderlyingType(cursor)));
                let saved = std::mem::replace(&mut self.writing, namespace_of(class));
                let record = self.record(class, Some(&name));
                self.cpp_members(class, record);
                self.writing = saved;
                return;
            }
            let Some(function) = referenced_function(cursor) else {
                self.skip(cursor, &name, "an instantiation Clang names no function for");
                return;
            };
            let saved = std::mem::replace(&mut self.writing, namespace_of(function));
            self.callee = Some(format!("::{written}"));
            match self.cpp_thunk(function, &name, None) {
                Ok(made) if self.claim(&name) => self.functions.push(Function { namespace: self.writing.clone(), ..made }),
                Ok(_) => self.skip(cursor, &name, "its name is taken by another declaration of the header"),
                Err(reason) => self.skip(cursor, &name, reason),
            }
            self.callee = None;
            self.writing = saved;
        }
    }

    /// `[FFI-24]` — a C++ function's exception policy, derived from its declaration: a
    /// `noexcept`, a `throw()`, a `noexcept(expr)` whose `expr` holds, and a destructor are
    /// non-throwing; a declaration with no exception specification may throw; a specification
    /// Clang has not evaluated (a template's) derives nothing.
    fn policy(&self, cursor: CXCursor) -> Policy {
        // SAFETY(ffi): a live cursor.
        let specification = unsafe { clang_getCursorExceptionSpecificationType(cursor) };
        match specification {
            _ if kind(cursor) == CXCursor_Destructor => Policy::Direct,
            CXCursor_ExceptionSpecificationKind_BasicNoexcept | CXCursor_ExceptionSpecificationKind_DynamicNone | CXCursor_ExceptionSpecificationKind_NoThrow => Policy::Direct,
            CXCursor_ExceptionSpecificationKind_ComputedNoexcept | CXCursor_ExceptionSpecificationKind_Uninstantiated | CXCursor_ExceptionSpecificationKind_Unevaluated => match self.nothrow.get(&usr(cursor)) {
                Some(true) => Policy::Direct,
                Some(false) => Policy::Catching,
                None => Policy::Unknown,
            },
            CXCursor_ExceptionSpecificationKind_None | CXCursor_ExceptionSpecificationKind_Dynamic | CXCursor_ExceptionSpecificationKind_MSAny => Policy::Catching,
            _ => Policy::Unknown,
        }
    }

    /// `[FFI-29]` — an external wrapper for a function the header defines with internal linkage
    /// (`static inline`): one more call, unless the shim takes part in link-time optimisation.
    fn wrapper(&mut self, cursor: CXCursor, wrapper: &str, name: &str) {
        // SAFETY(ffi): a live function cursor.
        unsafe {
            let ty = clang_getCursorType(cursor);
            let count = clang_Cursor_getNumArguments(cursor).max(0) as u32;
            let params: Vec<String> = (0..count).map(|index| c_declaration(clang_getArgType(ty, index), &format!("a{index}"))).collect();
            let arguments: Vec<String> = (0..count).map(|index| format!("a{index}")).collect();
            let params = if params.is_empty() { "void".to_string() } else { params.join(", ") };
            let result = clang_getResultType(ty);
            let call = format!("{name}({})", arguments.join(", "));
            let body = if result.kind == CXType_Void { format!("{call};") } else { format!("return {call};") };
            let linkage = if self.cpp { "extern \"C\" " } else { "" };
            let _ = writeln!(self.shim, "{linkage}{} {{ {body} }}", c_declaration(result, &format!("{wrapper}({params})")));
        }
    }

    /// `[FFI-8]` — an `extern` global is an `extern static` (`mut` unless `const`); a `static const`
    /// with a constant value is a constant; any other internal global is `W5002`.
    fn global(&mut self, cursor: CXCursor) {
        let name = spelling(cursor);
        if !self.seen.insert(format!("var {name}")) {
            return;
        }
        // SAFETY(ffi): a live variable cursor.
        unsafe {
            let ty = clang_getCursorType(cursor);
            let constant = clang_isConstQualifiedType(ty) != 0;
            if clang_getCursorLinkage(cursor) == CXLinkage_Internal {
                let outcome = if constant { self.constant_value(cursor) } else { Err(String::new()) };
                match outcome {
                    Ok((ty, value)) if self.claim(&name) => {
                        self.push(&name, Kind::Const, format!("const {}: {ty} = {value}", ident(&name)));
                    }
                    _ => self.skip(cursor, &name, "a global with internal linkage (`static`): each C file has its own, which no binding may name ([FFI-29a])"),
                }
                return;
            }
            match self.ty(ty, false) {
                Ok(mapped) if self.claim(&name) => {
                    let mutable = if constant { "" } else { "mut " };
                    self.push(&name, Kind::Static, format!("static {mutable}{}: {mapped}", ident(&name)));
                }
                Ok(_) => self.skip(cursor, &name, "its name is taken by another declaration of the header"),
                Err(reason) => self.skip(cursor, &name, reason),
            }
        }
    }

    /// `[FFI-6a]` — an object-like macro to evaluate; any other is `W5001`. A macro that expands
    /// to nothing (an include guard, a feature flag) has no value to import. `[FFI-6b]` — a
    /// function-like macro is recorded with its parameters, for an overlay entry to declare.
    fn macro_definition(&mut self, cursor: CXCursor) {
        let name = spelling(cursor);
        // SAFETY(ffi): a live macro cursor and its tokens.
        unsafe {
            if clang_Cursor_isMacroBuiltin(cursor) != 0 || !self.seen.insert(format!("macro {name}")) {
                return;
            }
            let at = self.at(cursor);
            let tu = clang_Cursor_getTranslationUnit(cursor);
            let mut tokens: *mut CXToken = std::ptr::null_mut();
            let mut count: c_uint = 0;
            clang_tokenize(tu, clang_getCursorExtent(cursor), &mut tokens, &mut count);
            let spelled: Vec<String> = (0..count as usize).map(|index| text(clang_getTokenSpelling(tu, *tokens.add(index)))).collect();
            clang_disposeTokens(tu, tokens, count);
            if clang_Cursor_isMacroFunctionLike(cursor) != 0 {
                // `NAME ( a , b ) …`: its parameters, `...` for a variadic one.
                let params = spelled.iter().skip(2).take_while(|token| *token != ")").filter(|token| *token != ",").cloned().collect();
                self.macro_functions.push(MacroFunction { name: name.clone(), params });
                self.macro_skipped.push(Skipped {
                    name,
                    reason: "a function-like macro: an overlay's `@ffi(macro_fn)` entry declares its signature, and the import's shim wraps it ([FFI-6b])".to_string(),
                    at,
                });
                return;
            }
            if count > 1 {
                self.macro_names.push((name, at));
            }
        }
    }

    /// `[FFI-6a]` (ODR-145) — the Ember pointer type of a macro whose value is a data pointer that
    /// is no string: `*mut void` for `((void*)0)`. A function pointer is not one.
    fn pointer_constant(&mut self, cursor: CXCursor) -> Option<String> {
        // SAFETY(ffi): a live variable cursor.
        unsafe {
            let ty = clang_getCanonicalType(clang_getCursorType(cursor));
            if ty.kind != CXType_Pointer {
                return None;
            }
            let pointee = clang_getPointeeType(ty);
            if matches!(clang_getCanonicalType(pointee).kind, CXType_FunctionProto | CXType_FunctionNoProto) {
                return None;
            }
            let inner = self.ty(pointee, true).ok()?;
            Some(format!("*{}{inner}", if clang_isConstQualifiedType(pointee) != 0 { "" } else { "mut " }))
        }
    }

    /// A constant's Ember type and literal, as Clang evaluates it (`[FFI-6a]`).
    fn constant_value(&mut self, cursor: CXCursor) -> Result<(String, String), String> {
        // SAFETY(ffi): a live variable cursor; the result is disposed.
        unsafe {
            // A macro's probe is `__auto_type`, whose deduced type keeps its typedef only in its
            // spelling: `sizeof` is `size_t` (Clang's `__size_t`), `usize` ([FFI-8]).
            let written = clang_getCursorType(cursor);
            let ty = clang_getCanonicalType(written);
            let spelled = type_spelling(written);
            let fixed = standard_typedef(spelled.trim_start_matches("const ").trim());
            let written = if written.kind == CXType_Auto { ty } else { written };
            if ty.kind == CXType_Enum {
                return Err("a constant of an enum type: Ember's enum constants are its variants".to_string());
            }
            if ty.kind == CXType_Pointer {
                let pointee = clang_getCanonicalType(clang_getPointeeType(ty)).kind;
                if !matches!(pointee, CXType_Char_S | CXType_Char_U) {
                    return Err("a function pointer: a `const` holds only an address of data ([CT-2])".to_string());
                }
            }
            let result = clang_Cursor_Evaluate(cursor);
            if result.is_null() {
                return Err("Clang does not evaluate it to a constant".to_string());
            }
            let value = match clang_EvalResult_getKind(result) {
                CXEval_Int => fixed.map(|fixed| Ok(fixed.to_string())).unwrap_or_else(|| self.ty(written, false)).map(|mapped| {
                    let value = if clang_EvalResult_isUnsignedInt(result) != 0 {
                        clang_EvalResult_getAsUnsigned(result).to_string()
                    } else {
                        clang_EvalResult_getAsLongLong(result).to_string()
                    };
                    if mapped == "bool" { (mapped, (value != "0").to_string()) } else { (mapped, value) }
                }),
                CXEval_Float => self.ty(written, false).and_then(|mapped| {
                    let value = clang_EvalResult_getAsDouble(result);
                    if value.is_finite() { Ok((mapped, float_literal(value))) } else { Err("an infinite or NaN value".to_string()) }
                }),
                CXEval_StrLiteral => {
                    let bytes = CStr::from_ptr(clang_EvalResult_getAsStr(result)).to_bytes();
                    match std::str::from_utf8(bytes) {
                        Ok(string) => Ok(("cstr".to_string(), format!("c\"{}\"", escape(string)))),
                        Err(_) => Err("a string that is not UTF-8".to_string()),
                    }
                }
                _ => Err(format!(
                    "its value, of type `{}`, is not an integer, floating or string constant",
                    type_spelling(ty)
                )),
            };
            clang_EvalResult_dispose(result);
            value
        }
    }

    fn finish(self, header: PathBuf) -> Imported {
        let shim = if self.cpp {
            format!(
                "/* The thunks of `import cpp \"{}\"` ([FFI-3], [FFI-29]): the layout checks and the `extern \"C\"` functions Ember calls. */\n{}#include \"{}\"\n\n{}",
                header.display(),
                cpp_prelude(&self.stem),
                header.to_string_lossy().replace('\\', "/"),
                self.shim
            )
        } else {
            format!(
                "/* The shim of `import c \"{}\"` ([FFI-29]): the layout checks and wrappers Ember needs. */\n#include <stddef.h>\n#include \"{}\"\n\n{}",
                header.display(),
                header.to_string_lossy().replace('\\', "/"),
                self.shim
            )
        };
        Imported {
            items: self.items,
            header,
            skipped: self.skipped,
            opaque: self.opaque,
            macros: self.macro_skipped,
            layouts: self.layouts,
            shim,
            functions: self.functions,
            uses: self.uses,
            entities: self.entities,
            conflicts: self.conflicts,
            macro_functions: self.macro_functions,
            c_types: self.c_types,
            anonymous_namespace: self.anonymous_namespace,
            rvalue_members: self.rvalue_members,
            namespaces: self.namespaces,
            ns_uses: self.ns_uses,
        }
    }
}

/// `[FFI-17b]` — the template a template-id names, by its qualified name (`engine::Vec`), if the
/// header declares one: a class template's cursor kind or a function template's.
fn find_template(cursor: CXCursor, wanted: &str) -> Option<CXCursorKind> {
    for child in children(cursor) {
        match kind(child) {
            CXCursor_Namespace | CXCursor_LinkageSpec => {
                if let Some(found) = find_template(child, wanted) {
                    return Some(found);
                }
            }
            CXCursor_ClassTemplate | CXCursor_FunctionTemplate if qualified_name(child).trim_start_matches("::") == wanted => return Some(kind(child)),
            _ => {}
        }
    }
    None
}

/// `[FFI-17b]` — the function a `&::max<double>` names.
fn referenced_function(cursor: CXCursor) -> Option<CXCursor> {
    for child in children(cursor) {
        // SAFETY(ffi): cursors of a live translation unit.
        let referenced = unsafe { clang_getCursorReferenced(child) };
        if kind(child) == CXCursor_DeclRefExpr && kind(referenced) == CXCursor_FunctionDecl {
            return Some(referenced);
        }
        if let Some(found) = referenced_function(child) {
            return Some(found);
        }
    }
    None
}

/// `[FFI-17b]` — an instantiation's Ember name, made of its template-id: `Vec<int>` is `Vec_int`,
/// `Pair<int, float>` is `Pair_int_float`.
fn instance_name(written: &str) -> String {
    let template = written.rsplit("::").next().unwrap_or(written);
    let mut name = String::new();
    for c in template.chars() {
        if c.is_ascii_alphanumeric() {
            name.push(c);
        } else if !name.ends_with('_') {
            name.push('_');
        }
    }
    name.trim_end_matches('_').to_string()
}

/// `[FFI-41]` — the namespaces a declaration is in, outermost first.
fn namespace_of(cursor: CXCursor) -> Vec<String> {
    let mut path = Vec::new();
    // SAFETY(ffi): cursors of a live translation unit.
    let mut parent = unsafe { clang_getCursorSemanticParent(cursor) };
    while !matches!(kind(parent), CXCursor_TranslationUnit | CXCursor_InvalidFile | CXCursor_NoDeclFound | CXCursor_NotImplemented) {
        if kind(parent) == CXCursor_Namespace {
            path.push(spelling(parent));
        }
        // SAFETY(ffi): as above.
        parent = unsafe { clang_getCursorSemanticParent(parent) };
        if unsafe { clang_Cursor_isNull(parent) } != 0 {
            break;
        }
    }
    path.reverse();
    path
}

/// `[FFI-41]` — the name a namespace's module is imported under by another namespace's.
fn namespace_alias(home: &[String]) -> String {
    format!("{}_ns_{}", ember_branding::SYMBOL_PREFIX, if home.is_empty() { "root".to_string() } else { home.join("_") })
}

/// `[FFI-3]` — a C++ type as a thunk spells it: its canonical spelling, every name qualified.
fn cpp_spelling(ty: CXType) -> String {
    // SAFETY(ffi): a type of a live translation unit.
    type_spelling(unsafe { clang_getCanonicalType(ty) })
}

/// `[FFI-17]` — a C++ entity's fully qualified name: `::engine::Mesh::size`.
fn qualified_name(cursor: CXCursor) -> String {
    let mut parts = vec![spelling(cursor)];
    // SAFETY(ffi): cursors of a live translation unit.
    let mut parent = unsafe { clang_getCursorSemanticParent(cursor) };
    while matches!(kind(parent), CXCursor_Namespace | CXCursor_StructDecl | CXCursor_ClassDecl | CXCursor_ClassTemplate | CXCursor_UnionDecl) {
        parts.push(spelling(parent));
        // SAFETY(ffi): as above.
        parent = unsafe { clang_getCursorSemanticParent(parent) };
    }
    parts.reverse();
    format!("::{}", parts.join("::"))
}

/// `[FFI-24]` — whether a function's exception specification is one only Clang can evaluate:
/// `noexcept(expr)`, or one a template's instantiation has not evaluated yet.
fn undecided(cursor: CXCursor) -> bool {
    // SAFETY(ffi): a live cursor.
    matches!(
        unsafe { clang_getCursorExceptionSpecificationType(cursor) },
        CXCursor_ExceptionSpecificationKind_ComputedNoexcept | CXCursor_ExceptionSpecificationKind_Uninstantiated | CXCursor_ExceptionSpecificationKind_Unevaluated
    )
}

/// `[FFI-24]` — a call of a function with values of its parameters' types, for `noexcept(…)` to
/// ask of: `::f(std::declval<int>())`, `std::declval<C&>().m(…)`, `C(…)`.
fn probe_call(cursor: CXCursor) -> String {
    // SAFETY(ffi): cursors and types of a live translation unit.
    unsafe {
        let ty = clang_getCursorType(cursor);
        let arguments: Vec<String> = (0..clang_getNumArgTypes(ty).max(0) as u32)
            .map(|index| format!("std::declval<{}>()", type_spelling(clang_getCanonicalType(clang_getArgType(ty, index)))))
            .collect();
        let arguments = arguments.join(", ");
        let class = || cpp_spelling(clang_getCursorType(clang_getCursorSemanticParent(cursor)));
        match kind(cursor) {
            CXCursor_Constructor => format!("{}({arguments})", class()),
            CXCursor_CXXMethod if clang_CXXMethod_isStatic(cursor) == 0 => format!("std::declval<{}&>().{}({arguments})", class(), spelling(cursor)),
            _ => format!("{}({arguments})", qualified_name(cursor)),
        }
    }
}

/// `[FFI-32]`, `[FFI-24]` — what only C++'s own rules answer, asked of Clang in one parse of the
/// header: for each class it defines, whether it is standard-layout and trivially copyable; for
/// each function declared `noexcept(expr)`, whether `expr` holds. Each by USR.
fn cpp_facts(unit: &Unit, main: &Path, include: &str, args: &[String], in_header: &dyn Fn(CXCursor) -> bool) -> Result<(HashMap<String, bool>, HashMap<String, bool>), String> {
    // (a class's question, the USR, the question)
    fn gather(cursor: CXCursor, in_header: &dyn Fn(CXCursor) -> bool, out: &mut Vec<(bool, String, String)>) {
        for child in children(cursor) {
            // `[FFI-17b]` — a function template's instantiation `instantiate=[…]` lists, whose
            // `noexcept(expr)` is evaluated only for it.
            if kind(child) == CXCursor_VarDecl
                && spelling(child).starts_with(&format!("{}_inst_", ember_branding::SYMBOL_PREFIX))
                && let Some(function) = referenced_function(child)
                && undecided(function)
            {
                out.push((false, usr(function), format!("noexcept({})", probe_call(function))));
                continue;
            }
            // `[FFI-17b]` — a class template's instantiation `instantiate=[…]` lists.
            if kind(child) == CXCursor_TypeAliasDecl && spelling(child).starts_with(&format!("{}_inst_", ember_branding::SYMBOL_PREFIX)) {
                // SAFETY(ffi): types of a live translation unit.
                let ty = unsafe { clang_getCanonicalType(clang_getTypedefDeclUnderlyingType(child)) };
                let spelled = cpp_spelling(ty);
                // SAFETY(ffi): as above.
                out.push((true, usr(unsafe { clang_getTypeDeclaration(ty) }), format!("__is_trivially_copyable({spelled}) && __is_standard_layout({spelled})")));
                continue;
            }
            if !in_header(child) {
                continue;
            }
            // SAFETY(ffi): cursors and types of a live translation unit.
            unsafe {
                match kind(child) {
                    CXCursor_Namespace | CXCursor_LinkageSpec => gather(child, in_header, out),
                    CXCursor_StructDecl | CXCursor_ClassDecl if clang_isCursorDefinition(child) != 0 => {
                        let spelled = cpp_spelling(clang_getCursorType(child));
                        if !spelled.contains('(') {
                            out.push((true, usr(child), format!("__is_trivially_copyable({spelled}) && __is_standard_layout({spelled})")));
                        }
                        gather(child, in_header, out);
                    }
                    CXCursor_FunctionDecl | CXCursor_CXXMethod | CXCursor_Constructor if undecided(child) => {
                        out.push((false, usr(child), format!("noexcept({})", probe_call(child))));
                    }
                    _ => {}
                }
            }
        }
    }
    let mut questions = Vec::new();
    gather(unit.cursor(), in_header, &mut questions);
    let (mut by_value, mut nothrow) = (HashMap::new(), HashMap::new());
    if questions.is_empty() {
        return Ok((by_value, nothrow));
    }
    let prefix = format!("{}_cpp_fact_", ember_branding::SYMBOL_PREFIX);
    let mut probe = format!("{include}#include <utility>\n");
    for (index, (_, _, question)) in questions.iter().enumerate() {
        let _ = writeln!(probe, "constexpr bool {prefix}{index} = ({question});");
    }
    let probed = Unit::parse(main, &probe, args)?;
    for cursor in children(probed.cursor()) {
        if kind(cursor) == CXCursor_VarDecl
            && let Some(index) = spelling(cursor).strip_prefix(&prefix).and_then(|index| index.parse::<usize>().ok())
            && let Some(answer) = evaluate_unsigned(cursor)
        {
            let (class, usr, _) = &questions[index];
            if *class { &mut by_value } else { &mut nothrow }.insert(usr.clone(), answer != 0);
        }
    }
    Ok((by_value, nothrow))
}

/// `[FFI-3]`, `[FFI-24]` — the start of an `import cpp`'s thunk file: what the thunks use, and
/// where a caught exception is kept for the Ember side — its type's name (demangled where the C++
/// library says how, without MSVC's `class `) and its `what()`, per thread.
fn cpp_prelude(stem: &str) -> String {
    let prefix = ember_branding::SYMBOL_PREFIX;
    format!(
        r#"#include <cstddef>
#include <cstdlib>
#include <cstring>
#include <exception>
#include <new>
#include <string>
#include <typeinfo>
#include <utility>
#if __has_include(<cxxabi.h>)
#include <cxxabi.h>
#endif
#define _Static_assert static_assert
#define _Alignof alignof

namespace {{
thread_local std::string caught_type, caught_what;
[[maybe_unused]] std::string type_name(const char* name) {{
#if __has_include(<cxxabi.h>)
    int status = 0;
    char* demangled = abi::__cxa_demangle(name, nullptr, nullptr, &status);
    if (status == 0 && demangled) {{ std::string out(demangled); std::free(demangled); return out; }}
#endif
    std::string out(name);
    for (const char* tag : {{"class ", "struct "}}) {{ if (out.rfind(tag, 0) == 0) out.erase(0, std::strlen(tag)); }}
    return out;
}}
[[maybe_unused]] void caught() noexcept {{
    try {{ throw; }}
    catch (const std::exception& error) {{ caught_type = type_name(typeid(error).name()); caught_what = error.what(); }}
    catch (...) {{ caught_type = "an exception that is no std::exception"; caught_what.clear(); }}
}}
template <class T> [[maybe_unused]] T zero() noexcept {{ union {{ unsigned char bytes[sizeof(T)]; T value; }} none = {{}}; return none.value; }}
}}

extern "C" const char* {prefix}_cpp_{stem}_error_type(void) noexcept {{ return caught_type.c_str(); }}
extern "C" const char* {prefix}_cpp_{stem}_error_what(void) noexcept {{ return caught_what.c_str(); }}
"#
    )
}

/// `[FFI-8]` — where in `parent` its anonymous member `member` starts, in bytes: Clang gives a field's
/// offset by name through anonymous members, so the member's is its first named field's offset in
/// the parent less that field's offset in the member.
fn anonymous_offset(parent: CXType, member: CXCursor) -> Option<u64> {
    // SAFETY(ffi): types and cursors of a live translation unit; the names live for the calls.
    unsafe {
        let member_ty = clang_getCursorType(member);
        let named = children(member).into_iter().find(|child| kind(*child) == CXCursor_FieldDecl && !spelling(*child).is_empty())?;
        let name = CString::new(spelling(named)).ok()?;
        let in_parent = clang_Type_getOffsetOf(parent, name.as_ptr());
        let in_member = clang_Type_getOffsetOf(member_ty, name.as_ptr());
        (in_parent >= 0 && in_member >= 0 && in_parent >= in_member).then(|| ((in_parent - in_member) / 8) as u64)
    }
}

/// A field of a struct with bit-fields that is not one: where C puts it, and its line.
struct Plain {
    name: String,
    offset: u64,
    size: u64,
    align: u64,
    line: String,
}

/// `[FFI-8]` — a bit-field: its Ember name, its offset and width in bits, and its type.
struct BitField {
    name: String,
    offset: u64,
    width: u64,
    ty: String,
    signed: bool,
    boolean: bool,
}

/// `[FFI-8]` — a struct with bit-fields, laid out as C lays it out: its plain fields, each run of
/// bytes its bit-fields share as a byte array `_bits<k>` (and `_pad<k>` where C leaves a gap Ember
/// would not), each public with a zero default so the struct can be made; and `get_<f>`/`set_<f>`
/// for each bit-field, reading and writing its bits there (little-endian, as C's bit numbering is
/// on the targets Ember builds for). Returns the field lines, every member's offset and the
/// `extend` block.
fn bit_field_members(name: &str, plain: &[Plain], bits: &[BitField], size: u64, align: u64, packed: bool) -> Result<(Vec<String>, Vec<(String, u64)>, String), String> {
    if cfg!(target_endian = "big") {
        return Err("bit-fields on a big-endian target (their accessors are written for little-endian bit numbering)".to_string());
    }
    // Each bit-field's bytes; bit-fields that share a byte share one array.
    let mut sorted: Vec<&BitField> = bits.iter().collect();
    sorted.sort_by_key(|field| field.offset);
    let mut runs: Vec<(u64, u64)> = Vec::new();
    for field in &sorted {
        let (first, end) = (field.offset / 8, (field.offset + field.width).div_ceil(8));
        match runs.last_mut() {
            Some((_, last)) if first < *last => *last = (*last).max(end),
            _ => runs.push((first, end)),
        }
    }
    let mut members: Vec<(u64, Option<&Plain>, usize)> = plain.iter().map(|field| (field.offset, Some(field), 0)).collect();
    members.extend(runs.iter().enumerate().map(|(run, (start, _))| (*start, None, run)));
    members.sort_by_key(|(offset, ..)| *offset);
    let (mut lines, mut layout, mut cursor, mut pads) = (Vec::new(), Vec::new(), 0u64, 0);
    let mut pad = |lines: &mut Vec<String>, layout: &mut Vec<(String, u64)>, from: u64, to: u64| {
        let length = to - from;
        lines.push(format!("    pub _pad{pads}: [u8; {length}] = [{}]\n", vec!["0"; length as usize].join(", ")));
        layout.push((format!("_pad{pads}"), from));
        pads += 1;
    };
    for (offset, field, run) in members {
        if offset < cursor {
            return Err("bit-fields laid out over another field".to_string());
        }
        let natural = match field {
            Some(field) if !packed => cursor.next_multiple_of(field.align),
            _ => cursor,
        };
        if natural != offset {
            pad(&mut lines, &mut layout, cursor, offset);
        }
        match field {
            Some(field) => {
                lines.push(field.line.clone());
                layout.push((field.name.clone(), offset));
                cursor = offset + field.size;
            }
            None => {
                let (start, end) = runs[run];
                let length = end - start;
                lines.push(format!("    pub _bits{run}: [u8; {length}] = [{}]\n", vec!["0"; length as usize].join(", ")));
                layout.push((format!("_bits{run}"), start));
                cursor = end;
            }
        }
    }
    // Ember rounds the size up to the alignment, as C does; padding only where C's goes further.
    if cursor.next_multiple_of(align) != size && cursor < size {
        pad(&mut lines, &mut layout, cursor, size);
    }
    let mut methods = format!("\nextend {name}:\n");
    for field in bits {
        let (first, last) = (field.offset / 8, (field.offset + field.width - 1) / 8);
        if last - first >= 8 {
            return Err(format!("bit-field `{}` spans more than 8 bytes", field.name));
        }
        let run = runs.iter().position(|(start, end)| first >= *start && first < *end).expect("a bit-field is in its run");
        let start = runs[run].0;
        let shift = field.offset - first * 8;
        let mask: u64 = if field.width >= 64 { u64::MAX } else { (1 << field.width) - 1 };
        let read: Vec<String> = (first..=last)
            .map(|byte| match byte - first {
                0 => format!("(self._bits{run}[{}] as u64)", byte - start),
                j => format!("((self._bits{run}[{}] as u64) << {})", byte - start, 8 * j),
            })
            .collect();
        let (f, ty, width) = (&field.name, &field.ty, field.width);
        let value = if field.boolean {
            "value != 0".to_string()
        } else if field.signed {
            format!("(((value << {0}) as i64) >> {0}) as {ty}", 64 - width)
        } else {
            format!("value as {ty}")
        };
        let _ = write!(
            methods,
            "    ## The {width}-bit field `{f}`.\n    pub fn get_{f}(self) -> {ty}:\n        bits: u64 = {}\n        value = (bits >> {shift}) & {mask:#x}\n        return {value}\n\n",
            read.join(" | ")
        );
        let input = if field.boolean { "bits: u64 = 0\n        if value:\n            bits = 1".to_string() } else { "bits: u64 = value as u64".to_string() };
        let _ = write!(methods, "    ## Writes the {width}-bit field `{f}`: its low {width} bits.\n    pub fn set_{f}(mut self, value: {ty}):\n        {input}\n        bits = (bits & {mask:#x}) << {shift}\n");
        for byte in first..=last {
            let j = byte - first;
            let part = ((mask << shift) >> (8 * j)) & 0xff;
            let keep = !part & 0xff;
            let _ = writeln!(
                methods,
                "        self._bits{run}[{0}] = (((self._bits{run}[{0}] as u64) & {keep:#x}) | ((bits >> {1}) & {part:#x})) as u8",
                byte - start,
                8 * j
            );
        }
        methods.push('\n');
    }
    Ok((lines, layout, methods.trim_end().to_string() + "\n"))
}

/// The options an `import c` takes (`[FFI-6]`).
#[derive(Default)]
struct Options {
    include_paths: Vec<String>,
    defines: Vec<String>,
    link: Vec<String>,
    /// `[FFI-30c]` (ODR-149) — applied first to last.
    overlays: Vec<(String, Span)>,
    implementation: Vec<String>,
    /// `[FFI-17]` — `import cpp`'s: the `[cpp.<project>]` section, and the explicit template
    /// instantiations.
    project: Option<(String, Span)>,
    instantiate: Vec<String>,
}

const OPTIONS: &str = "`include_paths`, `defines`, `link`, `overlay` and `implementation`";
const CPP_OPTIONS: &str = "`project`, `overlay` and `instantiate`";

/// `[FFI-6]` — the options, each a list of strings but `overlay`, a string or a list of them
/// (`[FFI-30c]`). `[FFI-17]` — `import cpp` takes `project` (a string), `overlay` and
/// `instantiate` (a list of strings). An unknown option, or one written otherwise, is `E0104`.
fn options(written: &[AttrArg], cpp: bool, sink: &mut Sink) -> Option<Options> {
    let mut options = Options::default();
    let mut ok = true;
    let (form, accepted, rule) = if cpp { ("import cpp", CPP_OPTIONS, "FFI-17") } else { ("import c", OPTIONS, "FFI-6") };
    for arg in written {
        let AttrArg::Named { name, value } = arg else {
            let AttrArg::Expr(expr) = arg else { unreachable!() };
            sink.emit(
                Diagnostic::error(codes::E0104, expr.span, format!("an option of `{form}` is written `name = value`"))
                    .help(format!("the options are {accepted} [{rule}]")),
            );
            ok = false;
            continue;
        };
        let strings = |value: &Expr| -> Option<Vec<String>> {
            let ExprKind::ArrayLit(items) = &value.kind else { return None };
            items.iter().map(|item| match &item.kind { ExprKind::Lit(Literal::Str(text)) => Some(text.clone()), _ => None }).collect()
        };
        let list = match name.name.as_str() {
            "include_paths" if !cpp => &mut options.include_paths,
            "defines" if !cpp => &mut options.defines,
            "link" if !cpp => &mut options.link,
            "implementation" if !cpp => &mut options.implementation,
            "instantiate" if cpp => &mut options.instantiate,
            "project" if cpp => {
                if let ExprKind::Lit(Literal::Str(project)) = &value.kind {
                    options.project = Some((project.clone(), value.span));
                } else {
                    sink.emit(Diagnostic::error(codes::E0104, value.span, "`project` names a `[cpp.<project>]` section of the manifest: a string")
                        .help("write `project = \"engine\"` [FFI-17]"));
                    ok = false;
                }
                continue;
            }
            "overlay" => {
                let one = |expr: &Expr| match &expr.kind {
                    ExprKind::Lit(Literal::Str(path)) => Some((path.clone(), expr.span)),
                    _ => None,
                };
                let written = match &value.kind {
                    ExprKind::ArrayLit(items) => items.iter().map(one).collect::<Option<Vec<_>>>(),
                    _ => one(value).map(|written| vec![written]),
                };
                match written {
                    Some(written) => options.overlays.extend(written),
                    None => {
                        let ext = ember_branding::SOURCE_EXT;
                        sink.emit(
                            Diagnostic::error(codes::E0104, value.span, "`overlay` names an overlay file, or a list of them applied first to last")
                                .help(format!("write `overlay = \"overlays/name.{ext}\"` or `overlay = [\"theirs.{ext}\", \"ours.{ext}\"]` [FFI-30c]")),
                        );
                        ok = false;
                    }
                }
                continue;
            }
            other => {
                sink.emit(
                    Diagnostic::error(codes::E0104, name.span, format!("`{other}` is not an option of `{form}`"))
                        .help(format!("the options are {accepted} [{rule}]")),
                );
                ok = false;
                continue;
            }
        };
        match strings(value) {
            Some(items) => list.extend(items),
            None => {
                sink.emit(Diagnostic::error(codes::E0104, value.span, format!("`{}` is a list of strings", name.name))
                    .help(format!("write `{} = [\"…\"]`", name.name)));
                ok = false;
            }
        }
    }
    ok.then_some(options)
}

/// What a build's `import c`s have loaded so far.
#[derive(Default)]
pub struct Known {
    /// Each import (its header, folder and options), with its module's path.
    modules: HashMap<String, Vec<String>>,
    /// `[FFI-29b]` — each header's implementation macros compiled, with the import that asked.
    implementations: HashMap<(PathBuf, String), Span>,
    /// `[FFI-30]` — each C declaration an import wrote, by Clang's USR.
    shared: HashMap<String, Shared>,
}

/// A module `import c` wrote: its path, its declarations, and what else the import found.
pub struct Loaded {
    pub path: Vec<String>,
    pub module: ast::Module,
    pub foreign: ember_typeck::ForeignImport,
    /// The import's overlay files, each a module of its own whose entries are checked as any
    /// overlay's (`[GRM-35]`), with the function entries whose `instrumented` grade evidence backs
    /// (`[TCB-1]`).
    pub overlays: Vec<(Vec<String>, ast::Module, Vec<String>)>,
    /// `[FFI-41]` — a module for each C++ namespace of the header, under the import's.
    pub children: Vec<(Vec<String>, ast::Module, ember_typeck::ForeignImport)>,
}

/// `[FFI-6]` — the module an `import c` names, written from its header, and the import rewritten
/// to bind it (to `name`, or `c`). `known` maps each import already loaded (its header, folder and
/// options) to its module's path: a second import of it binds the same module and returns `None`.
/// `base`: where an option's relative path is found (the package's folder, else the importing
/// file's); `flags`: the package's `[c] flags` (`[BLD-FFI-1a]`).
#[allow(clippy::too_many_arguments)]
pub fn load(
    import: &mut ast::Import,
    importer: &Path,
    base: &Path,
    flags: &[String],
    manifest: Option<&str>,
    known: &mut Known,
    map: &mut SourceMap,
    sink: &mut Sink,
) -> Option<Loaded> {
    let span = import.span;
    let ast::ImportKind::Foreign { language, header, options: written, alias } = &import.kind else { return None };
    let language = *language;
    let cpp = language == ast::ForeignLanguage::Cpp;
    let mut options = options(written, cpp, sink)?;
    let bound = alias.map_or_else(|| Symbol::intern(if cpp { "cpp" } else { "c" }), |alias| alias.name);
    // `[FFI-17]`, `[FFI-50]` — the C++ project's configuration: its standard, defines, include
    // paths and flags, from the manifest's `[cpp.<project>]`.
    let project = if cpp {
        let section = options.project.as_ref().map(|(name, _)| format!("cpp.{name}"));
        let value = |key: &str| section.as_deref().and_then(|section| manifest.and_then(|text| manifest_value(text, section, key)));
        if let (Some((name, at)), None) = (&options.project, section.as_deref().and_then(|section| manifest.filter(|text| has_section(text, section)))) {
            sink.emit(
                Diagnostic::error(codes::E0104, *at, format!("the manifest has no `[cpp.{name}]` section"))
                    .help(format!("add `[cpp.{name}]` to the package's manifest, with the project's `standard`, `defines`, `include_paths` and `flags` [FFI-50]")),
            );
            return None;
        }
        options.defines = value("defines").map(|written| manifest_strings(&written)).unwrap_or_default();
        options.include_paths = value("include_paths").map(|written| manifest_strings(&written)).unwrap_or_default();
        let mut standard = value("standard").map(|written| written.trim_matches('"').to_string());
        let mut project_flags = value("flags").map(|written| manifest_strings(&written)).unwrap_or_default();
        let compiler = value("compiler").map(|written| written.trim_matches('"').to_string());
        // `[FFI-50]` — `cmake = { build_dir, target }`: the target's C++ flags, as CMake's File API
        // replies them for the configured build.
        if let Some(written) = value("cmake") {
            let at = options.project.as_ref().map_or(span, |(_, at)| *at);
            let entry = |key: &str| manifest_inline(&written, key);
            let (Some(build_dir), Some(target)) = (entry("build_dir"), entry("target")) else {
                sink.emit(Diagnostic::error(codes::E0104, at, "`cmake` names the CMake build: `cmake = { build_dir = \"build\", target = \"engine\" }`")
                    .note("the C++ project's flags are read from CMake's File API [FFI-50]"));
                return None;
            };
            match cmake_target(&base.join(&build_dir), &target) {
                Ok(found) => {
                    options.defines.extend(found.defines);
                    options.include_paths.extend(found.includes);
                    project_flags.extend(found.flags);
                    standard = standard.or(found.standard);
                }
                Err(message) => {
                    sink.emit(Diagnostic::error(codes::E9002, at, format!("cannot read the flags of CMake target `{target}`")).note(message).note("the C++ project's flags are read from CMake's File API [FFI-50]"));
                    return None;
                }
            }
        }
        let standard = standard.unwrap_or_else(|| "c++17".to_string());
        Some((standard, project_flags, compiler))
    } else {
        None
    };
    let flags: &[String] = match &project {
        Some((_, project_flags, _)) => project_flags,
        None => flags,
    };
    // A bare file name's folder is the working one.
    let from_dir = importer.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or(Path::new(".")).to_path_buf();
    // `[BLD-FFI-1]` — `import c "./bridge.c"` compiles `bridge.c` and imports `bridge.h`.
    let (header, source) = match header.strip_suffix(".c") {
        Some(stem) => (format!("{stem}.h"), Some(from_dir.join(header))),
        None => (header.clone(), None),
    };
    let key = format!(
        "{}|{header}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
        from_dir.display(),
        options.include_paths,
        options.defines,
        options.overlays.iter().map(|(written, _)| written).collect::<Vec<_>>(),
        options.implementation,
        project,
        options.instantiate
    );
    let rewrite = |import: &mut ast::Import, path: &[String]| {
        let segments = path.iter().map(|segment| ast::Ident { name: Symbol::intern(segment), span }).collect();
        import.kind = ast::ImportKind::Module { path: segments, alias: Some(ast::Ident { name: bound, span }) };
    };
    if let Some(path) = known.modules.get(&key) {
        let path = path.clone();
        rewrite(import, &path);
        return None;
    }
    let stem: String = Path::new(&header)
        .file_stem()
        .map(|stem| stem.to_string_lossy().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect())
        .unwrap_or_default();
    let mut path = vec![if cpp { "cpp" } else { "c" }.to_string(), format!("h_{stem}")];
    let mut suffix = 1;
    while known.modules.values().any(|taken| *taken == path) {
        suffix += 1;
        path[1] = format!("h_{stem}_{suffix}");
    }
    let include_paths: Vec<PathBuf> = options.include_paths.iter().map(|dir| base.join(dir)).collect();
    let config = Config {
        header: &header,
        from_dir: &from_dir,
        include_paths: &include_paths,
        defines: &options.defines,
        flags,
        symbol_stem: &path.join("_"),
        shared: &known.shared,
        binding: bound.as_str(),
        cpp: project.as_ref().map(|(standard, ..)| standard.as_str()),
        instantiate: &options.instantiate,
    };
    // `[FFI-7]` — the overlays are part of what the record is for.
    let overlay_text = options
        .overlays
        .iter()
        .map(|(written, _)| std::fs::read_to_string(base.join(written)).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n\0\n");
    let mut cached = false;
    let imported = match import_cached(&config, &overlay_text, &mut cached) {
        Ok((imported, _)) => {
            let stage = if cached { format!("import c, from its .{}", ember_branding::BINDING_EXT) } else { "import c, parsed".to_string() };
            crate::mark(&stage, &header);
            imported
        }
        Err(message) => {
            sink.emit(
                Diagnostic::error(codes::E9002, span, format!("cannot import `{header}`"))
                    .note(message)
                    .note("`import c` parses the header with libclang [FFI-6]"),
            );
            return None;
        }
    };
    let shown = imported.header.file_name().map_or(header.clone(), |name| name.to_string_lossy().into_owned());
    // `[FFI-30]` — one C declaration, two layouts, one name: which is meant cannot be told.
    for (name, other) in &imported.conflicts {
        sink.emit(
            Diagnostic::error(codes::E5011, span, format!("`{name}` is imported from `{shown}` with a layout other than the one `{other}` gave it"))
                .help("give one import its own alias (`as name`), making its entities distinct")
                .note("two imports of one C declaration with one layout are one Ember entity [FFI-30]"),
        );
    }
    for (usr, name, layout) in &imported.entities {
        known.shared.entry(usr.clone()).or_insert_with(|| Shared {
            module: path.clone(),
            name: name.clone(),
            layout: layout.clone(),
            binding: bound.to_string(),
            header: shown.clone(),
        });
    }
    // `[FFI-17b]` — a template is not reported: it is imported only as its instantiations, and
    // naming it is `E5055`. The code a reason carries for the type checker is not shown here.
    for skipped in imported.skipped.iter().filter(|skipped| !skipped.reason.starts_with("[E5055]")) {
        let reason = skipped.reason.split_once("] ").filter(|(tag, _)| tag.starts_with("[E")).map_or(skipped.reason.as_str(), |(_, rest)| rest);
        let help = if cpp {
            "to use it, wrap it in a C++ function of a header of yours whose types Ember has"
        } else {
            "to use it, declare it by hand in an `unsafe extern \"C\":` block with types Ember has, or wrap it in a C function the header declares"
        };
        sink.emit(
            Diagnostic::warning(codes::W5002, span, format!("`{}` is not imported from `{shown}`: {reason}", skipped.name))
                .note(format!("declared at {} [FFI-20a]", skipped.at))
                .help(help),
        );
    }
    // `[FFI-40]` — `&&`-qualified members.
    for skipped in &imported.rvalue_members {
        sink.emit(
            Diagnostic::warning(codes::W5033, span, format!("`{}` is not imported from `{shown}`: {}", skipped.name, skipped.reason))
                .note(format!("declared at {} [FFI-40]", skipped.at)),
        );
    }
    // `[FFI-41]` — an anonymous namespace's entities.
    for skipped in &imported.anonymous_namespace {
        sink.emit(
            Diagnostic::warning(codes::W5034, span, format!("`{}` is not imported from `{shown}`: {}", skipped.name, skipped.reason))
                .note(format!("declared at {} [FFI-41]", skipped.at)),
        );
    }
    for opaque in &imported.opaque {
        sink.emit(
            Diagnostic::warning(codes::W5002, span, format!("`{}` is imported from `{shown}` only as an opaque type, usable behind a pointer: {}", opaque.name, opaque.reason))
                .note(format!("declared at {} [FFI-20a]", opaque.at))
                .help("to reach its fields, wrap them in C functions the header declares"),
        );
    }
    // `[FFI-2]` — the overlays, applied first to last before the module is written (`[FFI-30c]`).
    let symbol_stem = path.join("_");
    let mut applied = Applied::default();
    let mut overlay_modules = Vec::new();
    for (layer, (written, at)) in options.overlays.iter().enumerate() {
        let overlay_file = base.join(written);
        match map.load(&overlay_file) {
            Err(error) => sink.emit(crate::load_diagnostic(&error, *at)),
            Ok(file) => {
                let text = map.file(file).text.clone();
                let lexed = ember_lexer::lex(file, &text, sink);
                let parsed = ember_parser::parse(file, &text, lexed.tokens, sink);
                let overlays: Vec<&ast::OverlayDecl> = parsed
                    .items
                    .iter()
                    .filter_map(|item| match &item.kind {
                        ast::ItemKind::Overlay(overlay) if overlay.language == language => Some(overlay),
                        _ => None,
                    })
                    .collect();
                match overlays.as_slice() {
                    [overlay] => apply(&mut applied, layer, overlay, &text, &shown, &imported, &symbol_stem, sink),
                    _ => sink.emit(
                        Diagnostic::error(codes::E5010, *at, format!("`{written}` holds {} `overlay c` declarations: an import's overlay file holds one", overlays.len()))
                            .note("an overlay states the contracts of one header's declarations [FFI-12]"),
                    ),
                }
                let mut overlay_path = path.clone();
                overlay_path[1].push_str("_overlay");
                if layer > 0 {
                    let _ = write!(overlay_path[1], "_{}", layer + 1);
                }
                overlay_modules.push((overlay_path, parsed, Vec::new()));
            }
        }
    }
    // `[FFI-6b]` — a macro an overlay declares is imported.
    let declared = |name: &str| applied.macros.iter().any(|(function, ..)| function.name == name);
    for skipped in imported.macros.iter().filter(|skipped| !declared(&skipped.name)) {
        sink.emit(
            Diagnostic::warning(codes::W5001, span, format!("macro `{}` is not imported from `{shown}`: {}", skipped.name, skipped.reason))
                .note(format!("defined at {} [FFI-6a]", skipped.at)),
        );
    }
    // Named for its module: the source map keeps one file per name, and one header imported twice
    // is two modules.
    // No quotes: the name reaches the C as `#line`'s file name ([CG-C-8]).
    let rendered = render(&imported, &applied, &symbol_stem, &path, &[]);
    // `[TCB-1]` — each `@grade(instrumented)` entry an evidence record of this package backs.
    let evidence = if overlay_modules.is_empty() { None } else { crate::foreign::load_evidence(importer) };
    for (_, overlay, backed) in &mut overlay_modules {
        let Some(evidence) = &evidence else { break };
        let source_hash = ember_build::content_hash(rendered.as_bytes());
        for item in overlay.items.iter().filter_map(|item| match &item.kind {
            ast::ItemKind::Overlay(overlay) => Some(overlay),
            _ => None,
        }).flat_map(|overlay| &overlay.items) {
            let ast::OverlayItemKind::Fn { name, .. } = &item.kind else { continue };
            let graded = item.attrs.iter().any(|attr| {
                attr.path.len() == 1 && attr.path[0].name.is("grade")
                    && matches!(attr.args.as_slice(), [AttrArg::Expr(Expr { kind: ExprKind::Path { segments }, .. })]
                        if segments.len() == 1 && segments[0].name.is("instrumented"))
            });
            let bound_name = applied.renamed.get(name.name.as_str()).map_or(name.name.as_str(), String::as_str);
            let qualified = format!("{}.{bound_name}", path.join("."));
            if graded && crate::foreign::backs_instrumented(evidence, &qualified, &source_hash) {
                backed.push(name.name.to_string());
            }
        }
    }
    let file = map.add(PathBuf::from(format!("<import {} {header} as {}>", if cpp { "cpp" } else { "c" }, path.join("."))), rendered);
    let text = map.file(file).text.clone();
    let lexed = ember_lexer::lex(file, &text, sink);
    let module = ember_parser::parse(file, &text, lexed.tokens, sink);
    // `[FFI-29b]` — a single-header library's implementation, compiled once per build: in this
    // import's shim, with its macros defined; a second request for one header is `E5014`.
    let mut shim = String::new();
    for macro_name in &options.implementation {
        match known.implementations.get(&(imported.header.clone(), macro_name.clone())) {
            Some(&first) => sink.emit(
                Diagnostic::error(codes::E5014, span, format!("`{shown}`'s implementation (`{macro_name}`) is already compiled into this build"))
                    .secondary(first, "requested here first")
                    .note("a single-header library's implementation is compiled exactly once per build [FFI-29b]")
                    .help("request it in one `import c` only, or import the header the same way everywhere"),
            ),
            None => {
                known.implementations.insert((imported.header.clone(), macro_name.clone()), span);
                let _ = writeln!(shim, "#define {macro_name}");
            }
        }
    }
    shim.push_str(&imported.shim);
    if !applied.macros.is_empty() {
        shim.push_str("\n/* The wrappers of the macros the overlays declare ([FFI-6b]). */\n#include <stdint.h>\n");
        for (_, _, wrapper) in &applied.macros {
            shim.push_str(wrapper);
        }
    }
    // `[FFI-24]` — a C++ function whose policy is neither derived nor declared is `E5061` where
    // it is used.
    let undecided: Vec<(String, String)> = imported
        .functions
        .iter()
        .filter(|function| function.policy == Policy::Unknown && applied.functions.get(&function.name).and_then(|entry| entry.policy).is_none())
        .map(|function| (function.name.clone(), "[E5061] its exception policy cannot be derived from the header (its specification is not evaluated), and no overlay declares one".to_string()))
        .collect();
    let mut c_flags: Vec<String> = flags.to_vec();
    c_flags.extend(include_paths.iter().map(|dir| format!("-I{}", dir.display())));
    c_flags.extend(options.defines.iter().map(|define| format!("-D{define}")));
    let mut identities = vec![("header".to_string(), imported.header.clone())];
    identities.extend(options.overlays.iter().map(|(written, _)| ("overlay".to_string(), base.join(written))));
    let foreign = ember_typeck::ForeignImport {
        backed: Vec::new(),
        header: shown,
        identities,
        skipped: imported
            .skipped
            .iter()
            .map(|skipped| (skipped.name.clone(), skipped.reason.clone()))
            .chain(imported.macros.iter().filter(|skipped| !declared(&skipped.name)).map(|skipped| (skipped.name.clone(), format!("a macro, {}", skipped.reason))))
            .chain(imported.anonymous_namespace.iter().map(|skipped| (skipped.name.clone(), skipped.reason.clone())))
            .chain(undecided)
            .collect(),
        layouts: imported.layouts.iter().filter(|layout| layout.namespace.is_empty()).map(|layout| (layout.name.clone(), layout.size, layout.align, layout.fields.clone())).collect(),
        shim,
        sources: source.into_iter().collect(),
        flags: c_flags,
        link: options
            .link
            .iter()
            .map(|library| if ember_build::cdylib::is_library_path(library) { base.join(library).to_string_lossy().into_owned() } else { library.clone() })
            .collect(),
        cpp: project.map(|(standard, _, compiler)| (standard, compiler)),
    };
    // `[FFI-41]` — each C++ namespace a module under the import's.
    let children = imported
        .namespaces
        .iter()
        .map(|namespace| {
            let text = render(&imported, &applied, &symbol_stem, &path, namespace);
            let child: Vec<String> = path.iter().chain(namespace).cloned().collect();
            let file = map.add(PathBuf::from(format!("<import cpp {header} as {}>", child.join("."))), text);
            let text = map.file(file).text.clone();
            let lexed = ember_lexer::lex(file, &text, sink);
            let module = ember_parser::parse(file, &text, lexed.tokens, sink);
            let foreign = ember_typeck::ForeignImport {
                header: foreign.header.clone(),
                identities: foreign.identities.clone(),
                layouts: imported
                    .layouts
                    .iter()
                    .filter(|layout| layout.namespace == *namespace)
                    .map(|layout| (layout.name.clone(), layout.size, layout.align, layout.fields.clone()))
                    .collect(),
                cpp: foreign.cpp.clone(),
                ..Default::default()
            };
            (child, module, foreign)
        })
        .collect();
    rewrite(import, &path);
    known.modules.insert(key, path.clone());
    Some(Loaded { path, module, foreign, overlays: overlay_modules, children })
}

/// `[FFI-29]`, `[BLD-FFI-1]`, `[BLD-FFI-1a]`, `[BLD-FFI-3]` — each `import c`'s shim and C sources,
/// compiled with that import's flags into objects of their own, and its libraries in order.
pub fn build_objects(
    modules: &[ember_typeck::LoadedModule],
    toolchain: &ember_build::Toolchain,
    c_dir: &Path,
    obj_dir: &Path,
    profile: ember_build::Profile,
) -> Result<(Vec<PathBuf>, Vec<String>), String> {
    let ext = if matches!(toolchain, ember_build::Toolchain::Msvc { .. }) { "obj" } else { "o" };
    let mut objects = Vec::new();
    let mut libraries = Vec::new();
    for module in modules {
        let Some(foreign) = &module.foreign else { continue };
        // An overlay's module has no C of its own.
        if foreign.shim.is_empty() {
            continue;
        }
        let stem = module.path.join("_");
        // `[FFI-3]` — an `import cpp`'s thunks are C++, compiled as C++ by the build's compiler
        // (`[BLD-FFI-4]`), with the project's flags, and the C++ library linked.
        if let Some((standard, compiler)) = &foreign.cpp {
            let named = toolchain_name(toolchain);
            if let Some(compiler) = compiler.as_deref().filter(|compiler| *compiler != named) {
                return Err(format!("the C++ project of `{}` is compiled with `{compiler}`, and this build's C with `{named}`: one compiler builds both ([BLD-FFI-4])", foreign.header));
            }
            let thunks = c_dir.join(format!("{stem}.thunks.cpp"));
            std::fs::write(&thunks, &foreign.shim).map_err(|e| e.to_string())?;
            let object = obj_dir.join(format!("{stem}.thunks.{ext}"));
            ember_build::compile_cpp_object(toolchain, &thunks, &foreign.flags, &object, profile, standard).map_err(|e| e.to_string())?;
            objects.push(object);
            libraries.extend(foreign.link.iter().cloned());
            if !matches!(toolchain, ember_build::Toolchain::Msvc { .. }) {
                let library = if cfg!(target_os = "macos") { "c++" } else { "stdc++" };
                if !libraries.iter().any(|known| known == library) {
                    libraries.push(library.to_string());
                }
            }
            continue;
        }
        let shim = c_dir.join(format!("{stem}.shim.c"));
        std::fs::write(&shim, &foreign.shim).map_err(|e| e.to_string())?;
        let mut units = vec![(shim, obj_dir.join(format!("{stem}.shim.{ext}")))];
        for source in &foreign.sources {
            let name = source.file_stem().map_or(String::new(), |name| name.to_string_lossy().into_owned());
            units.push((source.clone(), obj_dir.join(format!("{stem}.{name}.{ext}"))));
        }
        for (source, object) in units {
            ember_build::compile_object_with_flags(toolchain, &source, &[], &foreign.flags, &object, profile).map_err(|e| e.to_string())?;
            objects.push(object);
        }
        libraries.extend(foreign.link.iter().cloned());
    }
    Ok((objects, libraries))
}

/// `[FFI-50]` — one key of a manifest inline table as written: `build_dir` of
/// `{ build_dir = "build", target = "engine" }`.
fn manifest_inline(written: &str, key: &str) -> Option<String> {
    written.trim().trim_start_matches('{').trim_end_matches('}').split(',').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        (name.trim() == key).then(|| value.trim().trim_matches('"').to_string())
    })
}

/// `[FFI-50]` — what CMake's File API says of a target's C++ compile: its defines, include
/// folders (made absolute against the build folder), other flags, and standard.
struct CmakeTarget {
    defines: Vec<String>,
    includes: Vec<String>,
    flags: Vec<String>,
    standard: Option<String>,
}

/// `[FFI-50]` — read `target`'s C++ compile from the File API reply in `build_dir` (its newest
/// index, the codemodel it names, the target's own reply). With no reply yet, the query that asks
/// for one is written, and configuring with CMake again makes it.
fn cmake_target(build_dir: &Path, target: &str) -> Result<CmakeTarget, String> {
    let api = build_dir.join(".cmake").join("api").join("v1");
    let reply = api.join("reply");
    let read = |path: &Path| -> Result<serde_json::Value, String> {
        let text = std::fs::read_to_string(path).map_err(|error| format!("cannot read `{}`: {error}", path.display()))?;
        serde_json::from_str(&text).map_err(|error| format!("`{}` is not JSON: {error}", path.display()))
    };
    let mut indexes: Vec<PathBuf> = std::fs::read_dir(&reply)
        .map(|entries| entries.filter_map(Result::ok).map(|entry| entry.path()).filter(|path| path.file_name().is_some_and(|name| name.to_string_lossy().starts_with("index-"))).collect())
        .unwrap_or_default();
    indexes.sort();
    let Some(index) = indexes.last() else {
        let query = api.join("query");
        let _ = std::fs::create_dir_all(&query).and_then(|()| std::fs::write(query.join("codemodel-v2"), ""));
        return Err(format!("`{}` has no CMake File API reply: the query for one is written there now, so configure it with CMake again", build_dir.display()));
    };
    let index = read(index)?;
    let codemodel = index["reply"]["codemodel-v2"]["jsonFile"].as_str().ok_or("the File API reply has no `codemodel-v2`")?;
    let codemodel = read(&reply.join(codemodel))?;
    let target_file = codemodel["configurations"][0]["targets"]
        .as_array()
        .and_then(|targets| targets.iter().find(|entry| entry["name"].as_str() == Some(target)))
        .and_then(|entry| entry["jsonFile"].as_str())
        .ok_or_else(|| format!("the CMake build has no target `{target}`"))?;
    let target_reply = read(&reply.join(target_file))?;
    let group = target_reply["compileGroups"]
        .as_array()
        .and_then(|groups| groups.iter().find(|group| group["language"].as_str() == Some("CXX")))
        .ok_or_else(|| format!("CMake compiles no C++ for target `{target}`"))?;
    let texts = |key: &str, field: &str| -> Vec<String> {
        group[key].as_array().into_iter().flatten().filter_map(|entry| entry[field].as_str().map(str::to_string)).collect()
    };
    Ok(CmakeTarget {
        defines: texts("defines", "define"),
        includes: texts("includes", "path")
            .into_iter()
            .map(|path| std::path::absolute(build_dir.join(&path)).unwrap_or_else(|_| build_dir.join(&path)).to_string_lossy().into_owned())
            .collect(),
        flags: texts("compileCommandFragments", "fragment").iter().flat_map(|fragment| fragment.split_whitespace().map(str::to_string)).collect(),
        standard: group["languageStandard"]["standard"].as_str().map(|standard| format!("c++{standard}")),
    })
}

/// `[FFI-50]` — a toolchain by the name a `[cpp.<project>]` section's `compiler` gives it.
fn toolchain_name(toolchain: &ember_build::Toolchain) -> &'static str {
    match toolchain {
        ember_build::Toolchain::Msvc { cl, .. } if cl.file_stem().is_some_and(|stem| stem.to_string_lossy().contains("clang")) => "clang-cl",
        ember_build::Toolchain::Msvc { .. } => "msvc",
        ember_build::Toolchain::Clang(_) => "clang",
        ember_build::Toolchain::Gcc(_) => "gcc",
    }
}

/// `[FFI-50]` — the manifest's value of `key` in `[section]`, as written.
fn manifest_value(manifest: &str, wanted: &str, key: &str) -> Option<String> {
    let mut section = "";
    for line in manifest.lines() {
        let line = line.split_once('#').map_or(line, |(before, _)| before).trim();
        if let Some(name) = line.strip_prefix('[').and_then(|line| line.strip_suffix(']')) {
            section = name.trim();
        } else if let Some((name, value)) = line.split_once('=')
            && section == wanted
            && name.trim() == key
        {
            return Some(value.trim().to_string());
        }
    }
    None
}

fn has_section(manifest: &str, wanted: &str) -> bool {
    manifest.lines().any(|line| line.trim().strip_prefix('[').and_then(|line| line.strip_suffix(']')).is_some_and(|name| name.trim() == wanted))
}

/// A manifest list of strings as written: `["A", "B=1"]`.
fn manifest_strings(written: &str) -> Vec<String> {
    written.trim().trim_start_matches('[').trim_end_matches(']').split(',').map(|item| item.trim().trim_matches('"').to_string()).filter(|item| !item.is_empty()).collect()
}

/// `[FFI-2]`, `[FFI-12]`, `[FFI-13]`, `[FFI-30a]` — what an import's overlays change in its module:
/// the contracts, safety and names of functions, facts on opaque types and enums, what is hidden,
/// the wrappers its `extend` blocks add, and the macros it declares (`[FFI-6b]`). It never changes a
/// type's identity, layout or fields.
#[derive(Default)]
struct Applied {
    hidden: HashSet<String>,
    /// A function's new name; a type's second name (an alias, so its identity stays).
    renamed: HashMap<String, String>,
    /// The attributes a `type` or `enum` entry states, as written.
    attributes: HashMap<String, Vec<String>>,
    functions: HashMap<String, Entry>,
    /// Each `extend` block, as written, at column 0.
    extends: Vec<String>,
    /// `[FFI-6b]` — each macro an entry declares: its binding, as a function of the header's would
    /// be, the entry that makes it safe or not, and its wrapper in the shim.
    macros: Vec<(Function, Entry, String)>,
    /// `[FFI-30c]` — which overlay, by its place in the import's list, wrote each entry.
    origins: HashMap<(&'static str, String), usize>,
}

/// An overlay's function entry: each parameter's contract words, the result's, and the entry's own
/// `@ffi(…)` arguments, each as written.
struct Entry {
    params: Vec<Vec<String>>,
    result: Vec<String>,
    arguments: Vec<String>,
    /// `[FFI-24b]`, `[FFI-43]` — a C++ function's exception policy, as the entry declares it:
    /// `@ffi(noexcept)` asserts it non-throwing, `@ffi(noexcept=false)` declares it throwing.
    policy: Option<Policy>,
}

/// The word of a contract word as written: `count` of `count(n)`.
fn word(written: &str) -> &str {
    written.split('(').next().unwrap_or(written).trim()
}

/// `[FFI-30c]` (ODR-149) — take the entry of `category` for `name` for the overlay at `layer`: a later
/// overlay's `override` entry replaces an earlier one's; any other second entry is `E5010`.
fn take(applied: &mut Applied, sink: &mut Sink, layer: usize, is_override: bool, category: &'static str, name: &ast::Ident) -> bool {
    let key = (category, name.name.to_string());
    let note = "an import's overlays compose in the order it lists them, and a later one replaces an entry only with `override` [FFI-30c]";
    let refused = match applied.origins.get(&key) {
        None if is_override => format!("`override` replaces an earlier overlay's entry, and none has one for `{}`", name.name),
        Some(&earlier) if earlier == layer => format!("a second entry for `{}` in one overlay", name.name),
        Some(_) if !is_override => {
            sink.emit(
                Diagnostic::error(codes::E5010, name.span, format!("an earlier overlay has an entry for `{}`", name.name))
                    .help("begin this entry with `override` to replace it")
                    .note(note),
            );
            return false;
        }
        _ => {
            applied.origins.insert(key, layer);
            return true;
        }
    };
    sink.emit(Diagnostic::error(codes::E5010, name.span, refused).note(note));
    false
}

/// `std.ffi`'s C aliases, each with the C type it names (`[FFI-8]`).
const C_ALIASES: [(&str, &str); 14] = [
    ("c_char", "char"),
    ("c_schar", "signed char"),
    ("c_uchar", "unsigned char"),
    ("c_short", "short"),
    ("c_ushort", "unsigned short"),
    ("c_int", "int"),
    ("c_uint", "unsigned int"),
    ("c_long", "long"),
    ("c_ulong", "unsigned long"),
    ("c_longlong", "long long"),
    ("c_ulonglong", "unsigned long long"),
    ("c_float", "float"),
    ("c_double", "double"),
    ("c_wchar", "wchar_t"),
];

/// `[FFI-6b]` — the C type a macro's wrapper declares for an Ember type its entry writes: a scalar,
/// `cstr`, a type of the module (by value only when it is not opaque), or a raw pointer to one.
fn c_spelling(ty: &ast::TypeExpr, imported: &Imported, behind_pointer: bool) -> Option<String> {
    match &ty.kind {
        ast::TypeKind::Ptr { mutable, inner } => {
            let pointee = if matches!(inner.kind, ast::TypeKind::Void) { "void".to_string() } else { c_spelling(inner, imported, true)? };
            Some(format!("{pointee}{} *", if *mutable { "" } else { " const" }))
        }
        ast::TypeKind::Path { segments, args } if segments.len() == 1 && args.is_empty() => {
            let name = segments[0].name.as_str();
            if let Some((_, c)) = C_ALIASES.iter().find(|(ember, _)| *ember == name) {
                return Some(c.to_string());
            }
            let scalar = match name {
                "i8" => "int8_t",
                "i16" => "int16_t",
                "i32" => "int32_t",
                "i64" => "int64_t",
                "u8" => "uint8_t",
                "u16" => "uint16_t",
                "u32" => "uint32_t",
                "u64" => "uint64_t",
                "isize" => "ptrdiff_t",
                "usize" => "size_t",
                "f32" => "float",
                "f64" => "double",
                "bool" => "_Bool",
                "cstr" => "char const *",
                _ => {
                    let opaque = imported.items.iter().any(|item| ident(&item.name) == name && item.kind == Kind::Opaque);
                    if opaque && !behind_pointer {
                        return None;
                    }
                    return imported.c_types.iter().find(|(ember, _)| ember == name).map(|(_, c)| c.clone());
                }
            };
            Some(scalar.to_string())
        }
        _ => None,
    }
}

/// `[FFI-12]` — apply an overlay, the `layer`th of the import's list, to the module the header
/// made; an entry whose name, parameter count or parameter names do not match the header is
/// `E5010`. `stem`: the module's path joined with `_`, which names the shim's wrappers.
#[allow(clippy::too_many_arguments)]
fn apply(applied: &mut Applied, layer: usize, overlay: &ast::OverlayDecl, text: &str, header: &str, imported: &Imported, stem: &str, sink: &mut Sink) {
    let snippet = |span: Span| text.get(span.start as usize..span.end as usize).unwrap_or("").to_string();
    let written = Path::new(&overlay.header).file_name().map(|name| name.to_string_lossy().into_owned());
    if written.as_deref() != Some(header) {
        sink.emit(
            Diagnostic::error(codes::E5010, overlay.span, format!("this overlay is for `{}`, and the import is of `{header}`", overlay.header))
                .note("an overlay states the contracts of one header's declarations [FFI-12]"),
        );
        return;
    }
    let item = |name: &str, kind: Kind| imported.items.iter().any(|item| item.name == name && item.kind == kind);
    let mismatch = |sink: &mut Sink, span: Span, message: String| {
        sink.emit(Diagnostic::error(codes::E5010, span, message).note("an overlay entry matches the header's declaration, so overlays cannot drift [FFI-12]"));
    };
    for entry in &overlay.items {
        let attributes = || entry.attrs.iter().map(|attr| snippet(attr.span)).collect::<Vec<_>>();
        // The entry's own `@ffi(…)` arguments, as written; `macro_fn` is what the entry is.
        let arguments = || -> Vec<String> {
            entry
                .attrs
                .iter()
                .filter(|attr| attr.path.len() == 1 && attr.path[0].name.is("ffi"))
                .flat_map(|attr| &attr.args)
                .map(|arg| match arg {
                    AttrArg::Expr(expr) => snippet(expr.span),
                    AttrArg::Named { name, value } => snippet(name.span.to(value.span)),
                })
                .filter(|written| written != "macro_fn")
                .collect()
        };
        match &entry.kind {
            ast::OverlayItemKind::Fn { name, params, result } => {
                let Some(function) = imported.functions.iter().find(|function| function.name == name.name.as_str()) else {
                    mismatch(sink, name.span, format!("`{header}` declares no function `{}`", name.name));
                    continue;
                };
                if params.len() != function.params.len() {
                    mismatch(sink, name.span, format!("`{}` takes {} parameters in `{header}`, and the entry names {}", name.name, function.params.len(), params.len()));
                    continue;
                }
                let mut matched = true;
                for (param, (c_name, _)) in params.iter().zip(&function.params) {
                    if param.name.name.as_str() != c_name.trim_start_matches("r#") {
                        mismatch(sink, param.name.span, format!("parameter `{}` of `{}` is `{c_name}` in `{header}`", param.name.name, name.name));
                        matched = false;
                    }
                }
                if !matched || !take(applied, sink, layer, entry.is_override, "fn", name) {
                    continue;
                }
                // `[FFI-24b]`, `[FFI-43]` — the exception policy an entry declares, never one it
                // declares twice over (`E5062`).
                let (declared, arguments): (Vec<String>, Vec<String>) = arguments().into_iter().partition(|written| {
                    let written: String = written.chars().filter(|c| !c.is_whitespace()).collect();
                    matches!(written.as_str(), "noexcept" | "noexcept=true" | "noexcept=false")
                });
                let asserts = declared.iter().any(|written| !written.ends_with("false"));
                let throws = declared.iter().any(|written| written.ends_with("false"));
                if asserts && throws {
                    sink.emit(
                        Diagnostic::error(codes::E5062, name.span, format!("the entry for `{}` declares it both `noexcept` and throwing", name.name))
                            .note("a C++ function has one exception policy [FFI-43]"),
                    );
                    continue;
                }
                let policy = if asserts { Some(Policy::Asserted) } else if throws { Some(Policy::Catching) } else { None };
                applied.functions.insert(
                    function.name.clone(),
                    Entry {
                        params: params.iter().map(|param| param.contract.iter().map(|word| snippet(word.span)).collect()).collect(),
                        result: result.iter().map(|word| snippet(word.span)).collect(),
                        arguments,
                        policy,
                    },
                );
            }
            // `[FFI-6b]` (ODR-147) — a macro's signature in Ember types, and the shim's wrapper.
            ast::OverlayItemKind::MacroFn { name, params, result } => {
                let Some(found) = imported.macro_functions.iter().find(|found| found.name == name.name.as_str()) else {
                    mismatch(sink, name.span, format!("`{header}` defines no function-like macro `{}`", name.name));
                    continue;
                };
                if imported.functions.iter().any(|function| function.name == found.name) {
                    mismatch(sink, name.span, format!("`{}` is a function of `{header}` as well as a macro: its entry is one without `@ffi(macro_fn)`", name.name));
                    continue;
                }
                if params.len() != found.params.len() {
                    mismatch(sink, name.span, format!("`{}` takes {} parameters in `{header}`, and the entry names {}", name.name, found.params.len(), params.len()));
                    continue;
                }
                let mut matched = true;
                for ((param, _), c_name) in params.iter().zip(&found.params) {
                    if param.name.as_str() != c_name {
                        mismatch(sink, param.span, format!("parameter `{}` of `{}` is `{c_name}` in `{header}`", param.name, name.name));
                        matched = false;
                    }
                }
                let mut spell = |ty: &ast::TypeExpr| {
                    let spelled = c_spelling(ty, imported, false);
                    if spelled.is_none() {
                        sink.emit(
                            Diagnostic::error(codes::E5015, ty.span, format!("`{}` has no C type for the wrapper of macro `{}`", snippet(ty.span), name.name))
                                .note("a macro's wrapper takes and returns scalars, `cstr`, the header's types and raw pointers [FFI-6b]"),
                        );
                        matched = false;
                    }
                    spelled.unwrap_or_default()
                };
                let c_params: Vec<String> = params.iter().map(|(_, ty)| spell(ty)).collect();
                let c_result = result.as_ref().map(&mut spell);
                if !matched || !take(applied, sink, layer, entry.is_override, "fn", name) {
                    continue;
                }
                let wrapper = format!("{}_cshim_{stem}_{}", ember_branding::SYMBOL_PREFIX, found.name);
                let declared: Vec<String> = c_params.iter().enumerate().map(|(index, c)| format!("{c} a{index}")).collect();
                let call = format!("{}({})", found.name, (0..params.len()).map(|index| format!("a{index}")).collect::<Vec<_>>().join(", "));
                let body = if result.is_some() { format!("return {call};") } else { format!("{call};") };
                let declared = if declared.is_empty() { "void".to_string() } else { declared.join(", ") };
                let c_text = format!("{} {wrapper}({declared}) {{ {body} }}\n", c_result.as_deref().unwrap_or("void"));
                let c_type = |ty: &ast::TypeExpr, c: &str| CType { ember: snippet(ty.span), pointee: None, char_pointer: false, c: c.to_string(), mode: String::new() };
                let function = Function {
                    name: found.name.clone(),
                    params: params.iter().zip(&c_params).map(|((param, ty), c)| (param.name.to_string(), c_type(ty, c))).collect(),
                    result: result.as_ref().zip(c_result.as_ref()).map(|(ty, c)| c_type(ty, c)),
                    link_name: Some(wrapper),
                    abi: "C",
                    policy: Policy::Direct,
                    owner: None,
                    out: false,
                    namespace: Vec::new(),
                };
                // A raw pointer states nothing a call can rely on: the call stays unsafe (`[FFI-2]`).
                let raw = params.iter().map(|(_, ty)| ty).chain(result).any(|ty| matches!(ty.kind, ast::TypeKind::Ptr { .. }));
                let made = Entry {
                    params: vec![Vec::new(); params.len()],
                    result: if raw { vec!["unsafe".to_string()] } else { Vec::new() },
                    arguments: arguments(),
                    policy: None,
                };
                match applied.macros.iter_mut().find(|(earlier, ..)| earlier.name == found.name) {
                    Some(earlier) => *earlier = (function, made, c_text),
                    None => applied.macros.push((function, made, c_text)),
                }
            }
            ast::OverlayItemKind::Type(name) | ast::OverlayItemKind::Enum(name) => {
                let kind = if matches!(entry.kind, ast::OverlayItemKind::Type(_)) { Kind::Opaque } else { Kind::Type };
                let found = imported.items.iter().any(|item| item.name == name.name.as_str() && item.kind == kind && (kind == Kind::Opaque || item.text.starts_with("@repr")));
                if !found {
                    let what = if kind == Kind::Opaque { "opaque type" } else { "enum" };
                    mismatch(sink, name.span, format!("`{header}` declares no {what} `{}`", name.name));
                    continue;
                }
                if take(applied, sink, layer, entry.is_override, "type", name) {
                    applied.attributes.insert(name.name.to_string(), attributes());
                }
            }
            ast::OverlayItemKind::Rename { from, to } => {
                let function = imported.functions.iter().any(|function| function.name == from.name.as_str())
                    || imported.macro_functions.iter().any(|found| found.name == from.name.as_str());
                if !function && !item(from.name.as_str(), Kind::Type) && !item(from.name.as_str(), Kind::Opaque) {
                    mismatch(sink, from.span, format!("`{header}` declares no function or type `{}` to rename", from.name));
                    continue;
                }
                if take(applied, sink, layer, entry.is_override, "rename", from) {
                    applied.renamed.insert(from.name.to_string(), to.name.to_string());
                }
            }
            ast::OverlayItemKind::Hide(name) => {
                let function = imported.functions.iter().any(|function| function.name == name.name.as_str())
                    || imported.macro_functions.iter().any(|found| found.name == name.name.as_str());
                if !function && !imported.items.iter().any(|item| item.name == name.name.as_str()) {
                    mismatch(sink, name.span, format!("`{header}` declares no `{}` to hide", name.name));
                    continue;
                }
                if take(applied, sink, layer, entry.is_override, "hide", name) {
                    applied.hidden.insert(name.name.to_string());
                }
            }
            // `[FFI-13]` — wrappers written in Ember, over the module's own declarations.
            ast::OverlayItemKind::Extend(_) => {
                let start = entry.attrs.first().map_or(entry.span, |attr| attr.span).start as usize;
                let line_start = text[..start].rfind('\n').map_or(0, |at| at + 1);
                let indent = start - line_start;
                let block: Vec<&str> = text[line_start..entry.span.end as usize].lines().collect();
                let dedented: Vec<&str> = block.iter().map(|line| line.get(indent.min(line.len() - line.trim_start().len())..).unwrap_or("")).collect();
                applied.extends.push(dedented.join("\n"));
            }
        }
    }
}

/// `[FFI-11]` — the Ember type a contracted pointer parameter is, as a hand declaration writes it:
/// `T*` is a mutable borrow and `const T*` a shared one (`[FFI-2a]`, derived, never written).
fn safe_parameter(name: &str, ty: &CType, words: &[String]) -> String {
    let has = |wanted: &str| words.iter().any(|written| word(written) == wanted);
    let raw = format!("{name}: {}", ty.ember);
    let Some((inner, constant)) = &ty.pointee else { return raw };
    if has("out") || has("inout_count") {
        return format!("mut {name}: {inner}");
    }
    let nullable = has("nullable");
    let optional = |base: String| if nullable { format!("Option[{base}]") } else { base };
    if has("owned") {
        return format!("owned {name}: {}", optional(format!("ForeignBox[{inner}]")));
    }
    if !has("borrowed") {
        return raw;
    }
    let mutable = !constant && !has("aliased");
    let fixed = words.iter().find_map(|written| written.strip_prefix("fixed(").and_then(|rest| rest.strip_suffix(')')));
    let base = if has("nul_terminated") && ty.char_pointer {
        "cstr".to_string()
    } else if has("count") {
        format!("{}[{inner}]", if mutable { "MutSpan" } else { "Span" })
    } else if let Some(length) = fixed {
        format!("ref {}[{inner}; {length}]", if mutable { "mut " } else { "" })
    } else if has("one") {
        if mutable && !nullable {
            return format!("mut {name}: {inner}");
        }
        format!("ref {}{inner}", if mutable { "mut " } else { "" })
    } else {
        return raw;
    };
    format!("{name}: {}", optional(base))
}

/// `[FFI-11]` — the Ember type a contracted pointer result is: shared unless `exclusive`.
fn safe_result(ty: &CType, words: &[String]) -> String {
    let has = |wanted: &str| words.iter().any(|written| word(written) == wanted);
    let Some((inner, _)) = &ty.pointee else { return ty.ember.clone() };
    let optional = |base: String| if has("nullable") { format!("Option[{base}]") } else { base };
    let exclusive = has("exclusive");
    let fixed = words.iter().find_map(|written| written.strip_prefix("fixed(").and_then(|rest| rest.strip_suffix(')')));
    let base = if has("returns_owned") {
        format!("ForeignBox[{inner}]")
    } else if !has("borrowed") {
        return ty.ember.clone();
    } else if has("nul_terminated") && ty.char_pointer {
        "cstr".to_string()
    } else if has("count") {
        format!("{}[{inner}]", if exclusive { "MutSpan" } else { "Span" })
    } else if let Some(length) = fixed {
        format!("ref {}[{inner}; {length}]", if exclusive { "mut " } else { "" })
    } else if has("one") {
        format!("ref {}{inner}", if exclusive { "mut " } else { "" })
    } else {
        return ty.ember.clone();
    };
    optional(base)
}

const UNCONTRACTED: &str = "the header states no contract: each pointer's ownership, count, nullability and lifetime are the caller's to keep ([FFI-38])";

/// A function of the `unsafe extern` block: as the header declares it, uncontracted (`[FFI-38]`),
/// or as its overlay entry contracts it (`[FFI-2]`).
fn render_function(function: &Function, entry: Option<&Entry>, renamed: Option<&String>, hidden: bool) -> String {
    let name = ident(renamed.unwrap_or(&function.name));
    let link = function.link_name.clone().or_else(|| renamed.map(|_| function.name.clone()));
    let mut arguments = Vec::new();
    let mut params: Vec<String> = function.params.iter().map(|(name, ty)| format!("{}{name}: {}", ty.mode, ty.ember)).collect();
    let mut result = function.result.as_ref().map(|ty| ty.ember.clone());
    let mut safe = false;
    if let Some(entry) = entry {
        let words = entry.params.iter().flatten().chain(&entry.result);
        // `[FFI-11c]` — a `TODO(…)` or `unsafe` word leaves the call unsafe.
        safe = !words.into_iter().any(|written| matches!(word(written), "TODO" | "unsafe"));
        for ((param, ty), written) in function.params.iter().zip(&entry.params) {
            // `[FFI-11]` — `unsafe` states no fact: it leaves the call unsafe.
            let mut words: Vec<String> = written.iter().filter(|written| word(written) != "unsafe").cloned().collect();
            if words.is_empty() {
                continue;
            }
            // `[FFI-2a]` — `T*` is a mutable borrow: Ember's is exclusive.
            if ty.pointee.as_ref().is_some_and(|(_, constant)| !constant)
                && words.iter().any(|written| word(written) == "borrowed")
                && !words.iter().any(|written| matches!(word(written), "exclusive" | "aliased"))
            {
                words.push("exclusive".to_string());
            }
            arguments.push(format!("param({param}, {})", words.join(", ")));
        }
        let result_words: Vec<&str> = entry.result.iter().map(String::as_str).filter(|written| word(written) != "unsafe").collect();
        if !result_words.is_empty() {
            arguments.push(format!("result({})", result_words.join(", ")));
        }
        arguments.extend(entry.arguments.iter().cloned());
        if safe {
            params = function.params.iter().zip(&entry.params).map(|((param, ty), words)| safe_parameter(&format!("{}{param}", ty.mode), ty, words)).collect();
            result = function.result.as_ref().map(|ty| safe_result(ty, &entry.result));
        }
    }
    if let Some(link) = link {
        arguments.push(format!("link_name=\"{link}\""));
    }
    let mut text = String::new();
    if !arguments.is_empty() {
        let _ = writeln!(text, "    @ffi({})", arguments.join(", "));
    }
    if !safe {
        let _ = writeln!(text, "    @safety(\"{UNCONTRACTED}\")");
    }
    let arrow = result.map_or(String::new(), |result| format!(" -> {result}"));
    let public = if hidden { "" } else { "pub " };
    let _ = writeln!(text, "    {public}{}fn {name}({}){arrow}", if safe { "safe " } else { "" }, params.join(", "));
    text
}

/// The module's text: its declarations with an overlay applied.
/// `[FFI-41]` — `namespace`: the C++ namespace whose module this is (`[]` for the import's own),
/// `root`: the import's module path.
fn render(imported: &Imported, applied: &Applied, stem: &str, root: &[String], namespace: &[String]) -> String {
    let header = imported.header.file_name().map_or(String::new(), |name| name.to_string_lossy().into_owned());
    let mut body = String::new();
    let declare = |item: &Item, indent: &str| -> String {
        let mut out = String::new();
        for attribute in applied.attributes.get(&item.name).into_iter().flatten() {
            let _ = writeln!(out, "{indent}{attribute}");
        }
        let public = if applied.hidden.contains(&item.name) { "" } else { "pub " };
        let mut first = true;
        for line in item.text.lines() {
            if first && !line.starts_with('@') {
                let _ = writeln!(out, "{indent}{public}{line}");
                first = false;
            } else {
                let _ = writeln!(out, "{indent}{line}");
            }
        }
        out
    };
    let alias = |item: &Item| applied.renamed.get(&item.name).map(|to| format!("pub type {} = {}\n", ident(to), ident(&item.name)));
    for item in imported.items.iter().filter(|item| matches!(item.kind, Kind::Type | Kind::Const) && item.namespace == namespace) {
        body.push_str(&declare(item, ""));
        if let Some(alias) = alias(item) {
            body.push_str(&alias);
        }
        body.push('\n');
    }
    let mut blocks: Vec<(&str, String)> = Vec::new();
    let mut member = |abi: &'static str, text: String| match blocks.iter_mut().find(|(of, _)| *of == abi) {
        Some((_, members)) => members.push_str(&text),
        None => blocks.push((abi, text)),
    };
    for item in imported.items.iter().filter(|item| matches!(item.kind, Kind::Opaque | Kind::Static) && item.namespace == namespace) {
        member("C", declare(item, "    "));
        if let Some(alias) = alias(item) {
            body.push_str(&alias);
        }
    }
    // A hidden declaration stays, private to the module: another may name it (`destructor=f`).
    let mut wrappers = String::new();
    let mut extensions: Vec<(String, String)> = Vec::new();
    let mut catching = false;
    for function in imported.functions.iter().filter(|function| function.namespace == namespace) {
        let hidden = applied.hidden.contains(&function.name);
        let (entry, renamed) = (applied.functions.get(&function.name), applied.renamed.get(&function.name));
        // `[FFI-24b]` — an asserted `noexcept` keeps the catching thunk, and a throw panics; a
        // declared throwing function is caught; a non-throwing header declaration stays direct.
        let policy = match (function.policy, entry.and_then(|entry| entry.policy)) {
            (Policy::Direct, _) => Policy::Direct,
            (_, Some(declared)) => declared,
            (derived, None) => derived,
        };
        match (policy, &function.owner) {
            (Policy::Direct, None) if !function.out => member(function.abi, render_function(function, entry, renamed, hidden)),
            // `E5061` where it is used.
            (Policy::Unknown, _) => {}
            (policy, owner) => {
                let (raw, wrapper) = render_wrapper(function, entry, renamed, hidden, policy);
                member(function.abi, raw);
                catching |= policy != Policy::Direct;
                match owner {
                    Some(_) if wrapper.is_empty() => {}
                    Some((owner, _)) => match extensions.iter_mut().find(|(of, _)| of == owner) {
                        Some((_, methods)) => methods.push_str(&wrapper),
                        None => extensions.push((owner.clone(), wrapper)),
                    },
                    None => wrappers.push_str(&wrapper),
                }
            }
        }
    }
    for (owner, methods) in &extensions {
        let indented: String = methods.lines().map(|line| if line.is_empty() { "\n".to_string() } else { format!("    {line}\n") }).collect();
        let _ = write!(wrappers, "extend {owner}:\n{}\n", indented.trim_end());
        wrappers.push_str("\n\n");
    }
    if catching {
        let prefix = ember_branding::SYMBOL_PREFIX;
        for part in ["type", "what"] {
            member("C", format!("    @ffi(link_name=\"{prefix}_cpp_{stem}_error_{part}\", result(borrowed, nul_terminated, from(static)))\n    safe fn {prefix}_cpp_error_{part}() -> cstr\n"));
        }
        let _ = write!(
            wrappers,
            "fn {prefix}_cpp_text(text: cstr) -> String:\n    match text.to_str():\n        Ok(read):\n            return read.to_string()\n        Err(_):\n            return \"(not UTF-8)\".to_string()\n\n\
             ## `[FFI-24]` — the exception the last catching thunk caught on this thread.\n\
             fn {prefix}_cpp_error() -> CppError:\n    return CppError(type_name = {prefix}_cpp_text({prefix}_cpp_error_type()), what = {prefix}_cpp_text({prefix}_cpp_error_what()))\n\n\
             ## `[FFI-24]` — a catching call's value, or the exception its thunk caught.\n\
             fn {prefix}_cpp_result[T](owned value: T, failed: c_int) -> Result[T, CppError]:\n    if failed != 0:\n        return Err({prefix}_cpp_error())\n    return Ok(value)\n\n\
             ## `[FFI-24b]` — a call an overlay asserts `noexcept`: a throw panics, naming the declaration.\n\
             fn {prefix}_cpp_value[T](owned value: T, failed: c_int, name: str) -> T:\n    if failed != 0:\n        error = {prefix}_cpp_error()\n        panic(f\"`{{name}}` threw `{{error.type_name}}` ({{error.what}}), though its overlay asserts it `noexcept` [FFI-24b]\")\n    return value\n\n"
        );
    }
    // `[FFI-6b]` — each declared macro, bound to its wrapper in the shim.
    for (function, entry, _) in applied.macros.iter().filter(|_| namespace.is_empty()) {
        let hidden = applied.hidden.contains(&function.name);
        member(function.abi, render_function(function, Some(entry), applied.renamed.get(&function.name), hidden));
    }
    for (abi, members) in blocks {
        let _ = write!(body, "unsafe extern \"{abi}\":\n{members}\n");
    }
    body.push_str(&wrappers);
    for extend in applied.extends.iter().filter(|_| namespace.is_empty()) {
        let _ = write!(body, "{extend}\n\n");
    }
    let mut imports = String::new();
    for module in &imported.uses {
        let _ = writeln!(imports, "import {} as {}_{}", module.join("."), ember_branding::SYMBOL_PREFIX, module.join("_"));
    }
    // `[FFI-41]` — the other namespaces' modules whose types this one names.
    for (_, home) in imported.ns_uses.iter().filter(|(from, _)| from == namespace) {
        let path: Vec<&str> = root.iter().chain(home).map(String::as_str).collect();
        let _ = writeln!(imports, "import {} as {}", path.join("."), namespace_alias(home));
    }
    // `std.ffi`'s names the module's text uses: its C aliases (`[FFI-8]`) and `ForeignBox`.
    let words: HashSet<&str> = body.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).collect();
    let used: Vec<&str> = C_ALIASES.iter().map(|(ember, _)| *ember).chain(["ForeignBox", "CppError", "adopt"]).filter(|name| words.contains(name)).collect();
    if !used.is_empty() {
        let _ = writeln!(imports, "from std.ffi import {}", used.join(", "));
    }
    if !imports.is_empty() {
        imports.push('\n');
    }
    if !namespace.is_empty() {
        return format!("# The declarations of `{header}`'s namespace `{}`, written by `import cpp` ([FFI-41]).\n\n{imports}{body}", namespace.join("::"));
    }
    format!("# The declarations of `{header}`, written by `import c` ([FFI-6]).\n\n{imports}{body}")
}

/// `[FFI-24]`, `[FFI-24b]`, `[FFI-40]` — a C++ function reached through an Ember function of the
/// module: the raw declaration of its thunk, private (taking the receiver first for a method, and
/// last, when the thunk catches, where it says it caught), and the function the module exports —
/// a method, associated function or constructor of the class's type in an `extend` block, or a
/// function returning `Result[T, CppError]` (or `T`, panicking on a throw, when an overlay asserts
/// `@ffi(noexcept)`). It is safe to call when an overlay entry says so and no parameter is a raw
/// pointer; a destructor has only its raw declaration, which the type's `ForeignBox` calls.
fn render_wrapper(function: &Function, entry: Option<&Entry>, renamed: Option<&String>, hidden: bool, policy: Policy) -> (String, String) {
    let prefix = ember_branding::SYMBOL_PREFIX;
    let (owner, member) = match &function.owner {
        Some((owner, member)) => (owner.as_str(), Some(*member)),
        None => ("", None),
    };
    let raw_name = match member {
        Some(Member::Destructor) => format!("{prefix}_cpp_delete_{owner}"),
        Some(_) => format!("{prefix}_raw_{owner}_{}", function.name),
        None => format!("{prefix}_raw_{}", function.name),
    };
    let catching = policy != Policy::Direct;
    let mut raw = function.clone();
    raw.name = raw_name.clone();
    raw.owner = None;
    let receiver = match member {
        Some(Member::Method) => Some((format!("*{owner}"), "(ref self) as *", true)),
        Some(Member::MutMethod | Member::Destructor) => Some((format!("*mut {owner}"), "(ref mut self) as *mut ", false)),
        _ => None,
    };
    if let Some((pointer, _, constant)) = &receiver {
        let this = CType { ember: pointer.clone(), pointee: Some((owner.to_string(), *constant)), char_pointer: false, c: String::new(), mode: String::new() };
        raw.params.insert(0, ("this".to_string(), this));
    }
    let result_ty = function.result.as_ref().map(|ty| ty.ember.clone()).unwrap_or_default();
    if function.out {
        let out = CType { ember: format!("*mut {result_ty}"), pointee: Some((result_ty.clone(), false)), char_pointer: false, c: String::new(), mode: String::new() };
        raw.params.push(("out".to_string(), out));
        raw.result = None;
    }
    if catching {
        let failed = CType { ember: "*mut c_int".to_string(), pointee: Some(("c_int".to_string(), false)), char_pointer: false, c: "int*".to_string(), mode: String::new() };
        raw.params.push(("failed".to_string(), failed));
    }
    let raw_text = render_function(&raw, None, None, true);
    if member == Some(Member::Destructor) {
        return (raw_text, String::new());
    }
    let name = match member {
        Some(Member::Constructor | Member::BoxedConstructor) => "new".to_string(),
        _ => ident(renamed.unwrap_or(&function.name)),
    };
    let words_safe = entry.is_some_and(|entry| !entry.params.iter().flatten().chain(&entry.result).any(|written| matches!(word(written), "TODO" | "unsafe")));
    let safe = words_safe && !function.params.iter().any(|(_, ty)| ty.ember.starts_with('*'));
    let mut params: Vec<String> = function.params.iter().map(|(param, ty)| format!("{}{param}: {}", ty.mode, ty.ember)).collect();
    match member {
        Some(Member::Method) => params.insert(0, "self".to_string()),
        Some(Member::MutMethod) => params.insert(0, "mut self".to_string()),
        _ => {}
    }
    let mut arguments: Vec<String> = function.params.iter().map(|(param, _)| param.clone()).collect();
    if let Some((_, conversion, _)) = &receiver {
        arguments.insert(0, format!("{conversion}{owner}"));
    }
    if function.out {
        arguments.push(format!("((ref mut slot) as *mut MaybeUninit[{result_ty}]) as *mut {result_ty}"));
    }
    if catching {
        arguments.push("(ref mut failed) as *mut c_int".to_string());
    }
    let mut call = format!("{raw_name}({})", arguments.join(", "));
    let value = match member {
        // `[FFI-36a]` — the object a constructor made, owned through its destructor.
        Some(Member::BoxedConstructor) => {
            call = format!("adopt[{owner}]({call})");
            Some(format!("ForeignBox[{owner}]"))
        }
        _ => function.result.as_ref().map(|ty| ty.ember.clone()),
    };
    let returned = match (policy, &value) {
        (Policy::Direct | Policy::Asserted, Some(value)) => format!(" -> {value}"),
        (Policy::Direct | Policy::Asserted, None) => String::new(),
        (_, value) => format!(" -> Result[{}, CppError]", value.as_deref().unwrap_or("void")),
    };
    let quoted = function.name.replace('"', "");
    let mut text = String::new();
    if !safe {
        let _ = writeln!(text, "@safety(\"{UNCONTRACTED}\")");
    }
    let public = if hidden { "" } else { "pub " };
    let note = if safe { String::new() } else { "  # SAFETY: the caller keeps what the header does not state".to_string() };
    let _ = writeln!(text, "{public}{}fn {name}({}){returned}:{note}", if safe { "" } else { "unsafe " }, params.join(", "));
    let safety = "# SAFETY: the thunk takes this function's own arguments";
    if catching {
        let _ = writeln!(text, "    failed: c_int = 0");
    }
    // A class result: the thunk writes it into an uninitialised slot (zeroed when it caught).
    if function.out {
        let _ = writeln!(text, "    slot: MaybeUninit[{result_ty}] = MaybeUninit[{result_ty}].uninit()");
        let _ = writeln!(text, "    unsafe:  {}\n        {call}\n        result = slot.assume_init()", "# SAFETY: the thunk writes the slot, and zeroes it when it caught");
        let _ = writeln!(text, "    {}", match policy {
            Policy::Direct => "return result".to_string(),
            Policy::Asserted => format!("return {prefix}_cpp_value(result, failed, \"{}\")", function.name.replace('"', "")),
            _ => format!("return {prefix}_cpp_result(result, failed)"),
        });
        text.push('\n');
        let declared = format!("    result: {result_ty}\n");
        let at = text.find("    slot: ").unwrap_or(text.len());
        text.insert_str(at, &declared);
        return (raw_text, text);
    }
    match (value.is_some(), policy) {
        (true, Policy::Direct) => {
            let _ = writeln!(text, "    unsafe:  {safety}\n        return {call}");
        }
        (false, Policy::Direct) => {
            let _ = writeln!(text, "    unsafe:  {safety}\n        {call}");
        }
        (true, Policy::Asserted) => {
            let _ = writeln!(text, "    unsafe:  {safety}\n        return {prefix}_cpp_value({call}, failed, \"{quoted}\")");
        }
        (true, _) => {
            let _ = writeln!(text, "    unsafe:  {safety}\n        return {prefix}_cpp_result({call}, failed)");
        }
        (false, Policy::Asserted) => {
            let _ = writeln!(text, "    unsafe:  {safety}\n        {call}\n    {prefix}_cpp_value((), failed, \"{quoted}\")");
        }
        (false, _) => {
            let _ = writeln!(text, "    unsafe:  {safety}\n        {call}\n    return {prefix}_cpp_result((), failed)");
        }
    }
    text.push('\n');
    (raw_text, text)
}

/// The overlay entry that states nothing yet for a function: each unknown fact a `TODO(…)` word
/// (`[FFI-11c]`), and `-> unsafe` where every parameter and the result are scalars, since being safe
/// to call is the one fact no header states (`[FFI-2]`).
fn starter_entry(function: &Function) -> String {
    let todo = |ty: &CType| ty.pointee.is_some() || ty.ember.starts_with("Option[");
    let mut text = String::new();
    for (name, ty) in &function.params {
        if let Some((_, constant)) = &ty.pointee {
            let borrow = if *constant { "a read-only borrow (`const`)" } else { "a mutable borrow (no `const`)" };
            let _ = writeln!(text, "    # derived: `{name}` is {borrow}");
        }
    }
    let params: Vec<String> = function
        .params
        .iter()
        .map(|(name, ty)| if todo(ty) { format!("{name}: TODO(ownership) TODO(count) TODO(nullable)") } else { name.trim_start_matches("r#").to_string() })
        .collect();
    let result = match &function.result {
        Some(ty) if todo(ty) => " -> TODO(ownership) TODO(count) TODO(nullable) TODO(lifetime)",
        _ if function.params.iter().any(|(_, ty)| todo(ty)) => "",
        _ => " -> unsafe",
    };
    let _ = writeln!(text, "    fn {}({}){result}", function.name, params.join(", "));
    text
}

/// `[CLI-5]`, `[CLI-6]`, `[CLI-7]` — `ember bind <header> [-I dir] [-D name[=value]]
/// (--init [--merge] | --report [--overlay <file>] [--baseline <file> | --write-baseline <file>]
/// | --check <overlay>)`.
pub fn bind(args: &[String]) -> Result<std::process::ExitCode, String> {
    use std::process::ExitCode;
    let mut header = None;
    let (mut include_paths, mut defines) = (Vec::new(), Vec::new());
    let (mut init, mut merge, mut report) = (false, false, false);
    let (mut check, mut overlay, mut baseline, mut write_baseline) = (None, None, None, None);
    let (mut explain, mut emit) = (None, false);
    // `[FFI-50]` — `--emit-cpp`: the header is C++, and its thunk file is written for inspection.
    let (mut emit_cpp, mut standard) = (false, "c++17".to_string());
    let mut rest = args.iter();
    while let Some(argument) = rest.next() {
        let mut value = |flag: &str| rest.next().cloned().ok_or(format!("`{flag}` needs a value"));
        match argument.as_str() {
            "--init" => init = true,
            "--merge" => merge = true,
            "--report" => report = true,
            "--check" => check = Some(value("--check")?),
            "--overlay" => overlay = Some(value("--overlay")?),
            "--baseline" => baseline = Some(value("--baseline")?),
            "--write-baseline" => write_baseline = Some(value("--write-baseline")?),
            "-I" => include_paths.push(PathBuf::from(value("-I")?)),
            "-D" => defines.push(value("-D")?),
            "--explain" => explain = Some(value("--explain")?),
            "--emit-embind" => emit = true,
            "--emit-cpp" => emit_cpp = true,
            "--std" => standard = value("--std")?,
            flag if flag.starts_with('-') => return Err(format!("unknown `ember bind` option `{flag}`")),
            file if header.is_none() => header = Some(PathBuf::from(file)),
            extra => return Err(format!("unexpected `ember bind` argument `{extra}`")),
        }
    }
    let header = header.ok_or("`ember bind` needs a header")?;
    let file = header.file_name().ok_or("`ember bind` needs a header file")?.to_string_lossy().into_owned();
    let dir = header.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or(Path::new(".")).to_path_buf();
    let shared = HashMap::new();
    let config = Config {
        header: &file,
        from_dir: &dir,
        include_paths: &include_paths,
        defines: &defines,
        flags: &[],
        symbol_stem: "bind",
        shared: &shared,
        binding: if emit_cpp { "cpp" } else { "c" },
        cpp: emit_cpp.then_some(standard.as_str()),
        instantiate: &[],
    };
    let (imported, files) = import_header(&config)?;
    if emit_cpp {
        let path = dir.join(format!("{}.thunks.cpp", header.file_stem().unwrap_or_default().to_string_lossy()));
        std::fs::write(&path, &imported.shim).map_err(|error| format!("cannot write `{}`: {error}", path.display()))?;
        println!("wrote {}", path.display());
        return Ok(ExitCode::SUCCESS);
    }
    if emit {
        // `[FFI-14]` — the record a build keeps for this header, written beside it.
        let files: Vec<(String, String)> = files.into_iter().filter_map(|file| file_hash(&file).map(|hash| (file, hash))).collect();
        let path = dir.join(format!("{}.{}", header.file_stem().unwrap_or_default().to_string_lossy(), ember_branding::BINDING_EXT));
        std::fs::write(&path, embind(&imported, &files, &arguments(&config).join(" "), "", "")).map_err(|error| format!("cannot write `{}`: {error}", path.display()))?;
        println!("wrote {}", path.display());
        return Ok(ExitCode::SUCCESS);
    }
    let skipped: Vec<&Skipped> = imported.skipped.iter().chain(&imported.macros).collect();
    let mut map = SourceMap::new();
    let mut sink = Sink::new();
    let mut load_overlay = |path: &str, sink: &mut Sink| -> Result<Applied, String> {
        let id = map.load(path).map_err(|error| format!("cannot read `{path}`: {error}"))?;
        let text = map.file(id).text.clone();
        let lexed = ember_lexer::lex(id, &text, sink);
        let parsed = ember_parser::parse(id, &text, lexed.tokens, sink);
        let overlays: Vec<&ast::OverlayDecl> = parsed
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                ast::ItemKind::Overlay(overlay) => Some(overlay),
                _ => None,
            })
            .collect();
        match overlays.as_slice() {
            [overlay] => {
                let mut applied = Applied::default();
                apply(&mut applied, 0, overlay, &text, &file, &imported, "bind", sink);
                Ok(applied)
            }
            _ => Err(format!("`{path}` holds {} overlay declarations: it should hold one", overlays.len())),
        }
    };
    if init {
        // `[CLI-5]` — a starter overlay beside the header, never written over.
        let path = dir.join(format!("{}.overlay.{}", header.file_stem().unwrap_or_default().to_string_lossy(), ember_branding::SOURCE_EXT));
        let existing = std::fs::read_to_string(&path).ok();
        if existing.is_some() && !merge {
            return Err(format!("`{}` exists: `--merge` adds only the declarations it lacks", path.display()));
        }
        let have = match &existing {
            Some(_) => load_overlay(&path.to_string_lossy(), &mut sink)?,
            None => Applied::default(),
        };
        let mut entries = String::new();
        for function in imported.functions.iter().filter(|function| !have.functions.contains_key(&function.name)) {
            entries.push_str(&starter_entry(function));
        }
        let mentioned = existing.as_deref().unwrap_or("");
        for skip in skipped.iter().filter(|skip| !mentioned.contains(&format!("# skipped: `{}`", skip.name))) {
            let _ = writeln!(entries, "    # skipped: `{}` ({}): {}", skip.name, skip.at, skip.reason);
        }
        let text = match existing {
            Some(mut text) => {
                if !text.ends_with('\n') {
                    text.push('\n');
                }
                text + &entries
            }
            None => format!(
                "# A starter overlay for `{file}`, written by `ember bind --init` ([CLI-5]): replace each `TODO(…)` with\n\
                 # its fact, and remove `-> unsafe` from a function you assert safe to call ([FFI-2]).\n\n\
                 unsafe overlay c \"{file}\":\n{entries}"
            ),
        };
        std::fs::write(&path, text).map_err(|error| format!("cannot write `{}`: {error}", path.display()))?;
        println!("wrote {}", path.display());
        return Ok(ExitCode::SUCCESS);
    }
    if let Some(name) = explain {
        // `[FFI-14]` — a function's contract, each fact with where it comes from.
        let applied = match &overlay {
            Some(path) => load_overlay(path, &mut sink)?,
            None => Applied::default(),
        };
        eprint!("{}", sink.render(&map));
        let Some(function) = imported.functions.iter().find(|function| function.name == name) else {
            let why = skipped.iter().find(|skip| skip.name == name).map_or("it declares no such function".to_string(), |skip| format!("it was skipped: {}", skip.reason));
            return Err(format!("`{name}` is not imported from `{file}`: {why}"));
        };
        let entry = applied.functions.get(&function.name);
        print!("{}", render_function(function, entry, applied.renamed.get(&function.name), false));
        let words = |index: Option<usize>| -> Vec<String> {
            entry.map_or(Vec::new(), |entry| match index {
                Some(index) => entry.params.get(index).cloned().unwrap_or_default(),
                None => entry.result.clone(),
            })
        };
        let explain_slot = |slot: &str, ty: &CType, written: Vec<String>, result: bool| {
            let mut facts = Vec::new();
            match &ty.pointee {
                Some((_, constant)) => {
                    facts.push(if *constant { "a read-only borrow (derived: `const`)".to_string() } else { "a mutable borrow (derived: no `const`)".to_string() });
                    if written.is_empty() {
                        let unknown = if result { "ownership, count, nullability and lifetime (a result's lifetime is never derived)" } else { "ownership, count and nullability" };
                        facts.push(format!("unknown: its {unknown}"));
                    } else {
                        facts.push(format!("stated by the overlay: {}", written.join(" ")));
                    }
                }
                None if ty.ember.starts_with("Option[") => facts.push("a function pointer, which may be null".to_string()),
                None => facts.push("a value: nothing to state".to_string()),
            }
            println!("  {slot}: `{}` — {}", ty.c, facts.join("; "));
        };
        for (index, (param, ty)) in function.params.iter().enumerate() {
            explain_slot(&format!("`{param}`"), ty, words(Some(index)), false);
        }
        if let Some(ty) = &function.result {
            explain_slot("result", ty, words(None), true);
        }
        match entry {
            Some(_) => println!("  safe to call: asserted by the overlay's entry for it ([FFI-2])"),
            None => println!("  safe to call: unknown — never derived, since a scalar can be an address; an overlay entry asserts it ([FFI-2])"),
        }
        if function.link_name.is_some() {
            println!("  called through the import's shim: it has internal linkage (`static inline`) ([FFI-29a])");
        }
        return Ok(if sink.has_errors() { ExitCode::FAILURE } else { ExitCode::SUCCESS });
    }
    if let Some(path) = check {
        // `[CLI-7]` — `[FFI-12]`, then how many declarations are safe, still unsafe, and skipped.
        let applied = load_overlay(&path, &mut sink)?;
        eprint!("{}", sink.render(&map));
        let safe = imported
            .functions
            .iter()
            .filter(|function| applied.functions.get(&function.name).is_some_and(|entry| {
                !entry.params.iter().flatten().chain(&entry.result).any(|written| matches!(word(written), "TODO" | "unsafe"))
            }))
            .count();
        println!("{safe} safe, {} still unsafe, {} skipped", imported.functions.len() - safe, skipped.len());
        return Ok(if sink.has_errors() { ExitCode::FAILURE } else { ExitCode::SUCCESS });
    }
    if report {
        // `[CLI-6]` — every skipped declaration, every one still unsafe with its unknown facts.
        let applied = match &overlay {
            Some(path) => load_overlay(path, &mut sink)?,
            None => Applied::default(),
        };
        eprint!("{}", sink.render(&map));
        for skip in &skipped {
            println!("skipped: `{}` ({}): {}", skip.name, skip.at, skip.reason);
        }
        for opaque in &imported.opaque {
            println!("opaque: `{}` ({}): usable only behind a pointer: {}", opaque.name, opaque.at, opaque.reason);
        }
        for function in &imported.functions {
            let entry = applied.functions.get(&function.name);
            let mut unknown = Vec::new();
            match entry {
                None => unknown.push("safe to call".to_string()),
                Some(entry) => {
                    for ((name, _), words) in function.params.iter().zip(&entry.params) {
                        let todo: Vec<&str> = words.iter().filter(|written| word(written) == "TODO").map(String::as_str).collect();
                        if !todo.is_empty() {
                            unknown.push(format!("`{name}`: {}", todo.join(" ")));
                        }
                    }
                    if entry.result.iter().any(|written| matches!(word(written), "TODO" | "unsafe")) {
                        unknown.push(format!("the result: {}", entry.result.join(" ")));
                    }
                }
            }
            if entry.is_none() {
                for (name, ty) in &function.params {
                    if ty.pointee.is_some() {
                        unknown.push(format!("`{name}`: its ownership, count and nullability"));
                    }
                }
                if function.result.as_ref().is_some_and(|ty| ty.pointee.is_some()) {
                    unknown.push("the result: its ownership, count, nullability and lifetime".to_string());
                }
            }
            if !unknown.is_empty() {
                println!("unsafe: `{}`: {}", function.name, unknown.join("; "));
            }
        }
        // `[FFI-35a]` — what a contract leaves unsaid is assumed: a pointer parameter with no
        // `retained` word does not retain it, and a pointer result with no lifetime word is listed.
        for function in &imported.functions {
            let Some(entry) = applied.functions.get(&function.name) else { continue };
            for ((name, ty), words) in function.params.iter().zip(&entry.params) {
                if ty.pointee.is_some() && !words.iter().any(|written| word(written) == "retained") {
                    println!("assumes: `{}`: `{name}` does not retain the pointer", function.name);
                }
            }
            if function.result.as_ref().is_some_and(|ty| ty.pointee.is_some()) && !entry.result.iter().any(|written| word(written) == "from") {
                println!("assumes: `{}`: its pointer result states no lifetime", function.name);
            }
        }
        let names: Vec<&str> = skipped.iter().map(|skip| skip.name.as_str()).collect();
        if let Some(path) = write_baseline {
            std::fs::write(&path, names.join("\n") + "\n").map_err(|error| format!("cannot write `{path}`: {error}"))?;
        }
        let known: Vec<String> = match &baseline {
            Some(path) => std::fs::read_to_string(path).map_err(|error| format!("cannot read `{path}`: {error}"))?.lines().map(str::to_string).collect(),
            None => Vec::new(),
        };
        let new = names.iter().filter(|name| !known.iter().any(|known| known == *name)).count();
        if new > 0 {
            println!("{new} skipped declaration(s){}", if baseline.is_some() { " not in the baseline" } else { "" });
        }
        return Ok(if new > 0 || sink.has_errors() { ExitCode::FAILURE } else { ExitCode::SUCCESS });
    }
    Err("`ember bind` needs `--init`, `--report`, `--check <overlay>`, `--explain <fn>` or `--emit-embind`".to_string())
}

/// `[FFI-14]` — the `.embind` file: an import's result as versioned CBOR. Only the forms it needs:
/// unsigned integers, text, arrays and maps with text keys.
mod cbor {
    #[derive(Clone, Debug, PartialEq)]
    pub enum Value {
        Uint(u64),
        Text(String),
        Array(Vec<Value>),
        Map(Vec<(String, Value)>),
    }

    fn head(out: &mut Vec<u8>, major: u8, n: u64) {
        let major = major << 5;
        match n {
            0..=23 => out.push(major | n as u8),
            24..=0xff => out.extend([major | 24, n as u8]),
            0x100..=0xffff => {
                out.push(major | 25);
                out.extend((n as u16).to_be_bytes());
            }
            0x1_0000..=0xffff_ffff => {
                out.push(major | 26);
                out.extend((n as u32).to_be_bytes());
            }
            _ => {
                out.push(major | 27);
                out.extend(n.to_be_bytes());
            }
        }
    }

    pub fn encode(value: &Value, out: &mut Vec<u8>) {
        match value {
            Value::Uint(n) => head(out, 0, *n),
            Value::Text(text) => {
                head(out, 3, text.len() as u64);
                out.extend(text.as_bytes());
            }
            Value::Array(items) => {
                head(out, 4, items.len() as u64);
                items.iter().for_each(|item| encode(item, out));
            }
            Value::Map(entries) => {
                head(out, 5, entries.len() as u64);
                for (key, item) in entries {
                    encode(&Value::Text(key.clone()), out);
                    encode(item, out);
                }
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<Value> {
        let mut at = 0;
        let value = item(bytes, &mut at)?;
        (at == bytes.len()).then_some(value)
    }

    fn item(bytes: &[u8], at: &mut usize) -> Option<Value> {
        let first = *bytes.get(*at)?;
        *at += 1;
        let extra = |at: &mut usize, size: usize| -> Option<u64> {
            let slice = bytes.get(*at..*at + size)?;
            *at += size;
            Some(slice.iter().fold(0u64, |n, &byte| (n << 8) | u64::from(byte)))
        };
        let n = match first & 0x1f {
            small @ 0..=23 => u64::from(small),
            24 => extra(at, 1)?,
            25 => extra(at, 2)?,
            26 => extra(at, 4)?,
            27 => extra(at, 8)?,
            _ => return None,
        };
        Some(match first >> 5 {
            0 => Value::Uint(n),
            3 => {
                let text = bytes.get(*at..*at + usize::try_from(n).ok()?)?;
                *at += text.len();
                Value::Text(String::from_utf8(text.to_vec()).ok()?)
            }
            4 => Value::Array((0..n).map(|_| item(bytes, at)).collect::<Option<_>>()?),
            5 => Value::Map(
                (0..n)
                    .map(|_| match item(bytes, at)? {
                        Value::Text(key) => Some((key, item(bytes, at)?)),
                        _ => None,
                    })
                    .collect::<Option<_>>()?,
            ),
            _ => return None,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// `[FFI-14]` — what is written reads back.
        #[test]
        fn values_round_trip() {
            let value = Value::Map(vec![
                ("version".to_string(), Value::Uint(1)),
                ("big".to_string(), Value::Uint(1 << 40)),
                ("text".to_string(), Value::Text("é".repeat(300))),
                ("list".to_string(), Value::Array(vec![Value::Uint(24), Value::Uint(256), Value::Array(Vec::new())])),
            ]);
            let mut bytes = Vec::new();
            encode(&value, &mut bytes);
            assert_eq!(decode(&bytes), Some(value));
            assert_eq!(decode(&bytes[..bytes.len() - 1]), None);
        }
    }
}

use cbor::Value;

const EMBIND_VERSION: u64 = 4;

fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::Text(text) => Some(text.clone()),
        _ => None,
    }
}

fn uint_of(value: &Value) -> Option<u64> {
    match value {
        Value::Uint(n) => Some(*n),
        _ => None,
    }
}

fn array_of(value: &Value) -> Option<&[Value]> {
    match value {
        Value::Array(items) => Some(items),
        _ => None,
    }
}

fn field<'v>(value: &'v Value, name: &str) -> Option<&'v Value> {
    match value {
        Value::Map(entries) => entries.iter().find(|(key, _)| key == name).map(|(_, item)| item),
        _ => None,
    }
}

fn split_namespace(written: &str) -> Vec<String> {
    if written.is_empty() { Vec::new() } else { written.split("::").map(str::to_string).collect() }
}

fn read_namespace(value: &Value) -> Option<Vec<String>> {
    text_of(value).map(|written| split_namespace(&written))
}

fn texts(items: &[&str]) -> Value {
    Value::Array(items.iter().map(|item| Value::Text(item.to_string())).collect())
}

fn skips(list: &[Skipped]) -> Value {
    Value::Array(list.iter().map(|skip| texts(&[&skip.name, &skip.reason, &skip.at])).collect())
}

fn read_skips(value: &Value) -> Option<Vec<Skipped>> {
    array_of(value)?
        .iter()
        .map(|entry| {
            let entry = array_of(entry)?;
            Some(Skipped { name: text_of(entry.first()?)?, reason: text_of(entry.get(1)?)?, at: text_of(entry.get(2)?)? })
        })
        .collect()
}

fn c_type_value(ty: &CType) -> Value {
    let (inner, constant) = ty.pointee.clone().map_or((String::new(), 2), |(inner, constant)| (inner, u64::from(constant)));
    Value::Array(vec![
        Value::Text(ty.ember.clone()),
        Value::Text(inner),
        Value::Uint(constant),
        Value::Uint(u64::from(ty.char_pointer)),
        Value::Text(ty.c.clone()),
        Value::Text(ty.mode.clone()),
    ])
}

fn read_c_type(value: &Value) -> Option<CType> {
    let entry = array_of(value)?;
    let constant = uint_of(entry.get(2)?)?;
    Some(CType {
        ember: text_of(entry.first()?)?,
        pointee: (constant != 2).then(|| Some((text_of(&entry[1])?, constant == 1))).flatten(),
        char_pointer: uint_of(entry.get(3)?)? == 1,
        c: text_of(entry.get(4)?)?,
        mode: text_of(entry.get(5)?)?,
    })
}

const KINDS: [Kind; 4] = [Kind::Type, Kind::Const, Kind::Opaque, Kind::Static];
const ABIS: [&str; 4] = ["C", "system", "fastcall", "vectorcall"];

/// `[FFI-14]` — the `.embind` record: the header's files and their hashes (`[FFI-7]`'s "content"),
/// the Clang configuration, the overlay's hash and the contracts it gave, the declarations with
/// their layouts, and what was skipped.
fn embind(imported: &Imported, files: &[(String, String)], configuration: &str, overlay: &str, contracts: &str) -> Vec<u8> {
    let value = Value::Map(vec![
        ("version".to_string(), Value::Uint(EMBIND_VERSION)),
        ("files".to_string(), Value::Array(files.iter().map(|(path, hash)| texts(&[path, hash])).collect())),
        ("clang".to_string(), Value::Text(configuration.to_string())),
        ("overlay".to_string(), Value::Text(overlay.to_string())),
        ("contracts".to_string(), Value::Text(contracts.to_string())),
        ("header".to_string(), Value::Text(imported.header.to_string_lossy().into_owned())),
        (
            "items".to_string(),
            Value::Array(
                imported
                    .items
                    .iter()
                    .map(|item| {
                        let kind = KINDS.iter().position(|kind| *kind == item.kind).unwrap_or(0) as u64;
                        Value::Array(vec![Value::Text(item.name.clone()), Value::Uint(kind), Value::Text(item.text.clone()), Value::Text(item.namespace.join("::"))])
                    })
                    .collect(),
            ),
        ),
        (
            "functions".to_string(),
            Value::Array(
                imported
                    .functions
                    .iter()
                    .map(|function| {
                        Value::Map(vec![
                            ("name".to_string(), Value::Text(function.name.clone())),
                            ("abi".to_string(), Value::Text(function.abi.to_string())),
                            ("link".to_string(), Value::Text(function.link_name.clone().unwrap_or_default())),
                            (
                                "params".to_string(),
                                Value::Array(function.params.iter().map(|(name, ty)| Value::Array(vec![Value::Text(name.clone()), c_type_value(ty)])).collect()),
                            ),
                            ("result".to_string(), Value::Array(function.result.iter().map(c_type_value).collect())),
                            ("policy".to_string(), Value::Uint(POLICIES.iter().position(|policy| *policy == function.policy).unwrap_or(0) as u64)),
                            ("out".to_string(), Value::Uint(u64::from(function.out))),
                            ("namespace".to_string(), Value::Text(function.namespace.join("::"))),
                            ("owner".to_string(), Value::Array(function.owner.iter().map(|(owner, member)| {
                                Value::Array(vec![Value::Text(owner.clone()), Value::Uint(MEMBERS.iter().position(|known| known == member).unwrap_or(0) as u64)])
                            }).collect())),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "layouts".to_string(),
            Value::Array(
                imported
                    .layouts
                    .iter()
                    .map(|layout| {
                        Value::Array(vec![
                            Value::Text(layout.name.clone()),
                            Value::Uint(layout.size),
                            Value::Uint(layout.align),
                            Value::Array(layout.fields.iter().map(|(name, offset)| Value::Array(vec![Value::Text(name.clone()), Value::Uint(*offset)])).collect()),
                            Value::Text(layout.namespace.join("::")),
                        ])
                    })
                    .collect(),
            ),
        ),
        ("skipped".to_string(), skips(&imported.skipped)),
        ("opaque".to_string(), skips(&imported.opaque)),
        ("macros".to_string(), skips(&imported.macros)),
        ("shim".to_string(), Value::Text(imported.shim.clone())),
        ("uses".to_string(), Value::Array(imported.uses.iter().map(|path| Value::Array(path.iter().map(|segment| Value::Text(segment.clone())).collect())).collect())),
        ("entities".to_string(), Value::Array(imported.entities.iter().map(|(usr, name, layout)| texts(&[usr, name, layout])).collect())),
        ("conflicts".to_string(), Value::Array(imported.conflicts.iter().map(|(name, header)| texts(&[name, header])).collect())),
        (
            "macro_functions".to_string(),
            Value::Array(
                imported
                    .macro_functions
                    .iter()
                    .map(|found| Value::Array(std::iter::once(&found.name).chain(&found.params).map(|text| Value::Text(text.clone())).collect()))
                    .collect(),
            ),
        ),
        ("c_types".to_string(), Value::Array(imported.c_types.iter().map(|(ember, c)| texts(&[ember, c])).collect())),
        ("anonymous_namespace".to_string(), skips(&imported.anonymous_namespace)),
        ("rvalue_members".to_string(), skips(&imported.rvalue_members)),
        ("namespaces".to_string(), Value::Array(imported.namespaces.iter().map(|namespace| Value::Text(namespace.join("::"))).collect())),
        ("ns_uses".to_string(), Value::Array(imported.ns_uses.iter().map(|(from, to)| texts(&[&from.join("::"), &to.join("::")])).collect())),
    ]);
    let mut bytes = Vec::new();
    cbor::encode(&value, &mut bytes);
    bytes
}

/// An `.embind` record read back: its files and hashes, and the import.
fn read_embind(bytes: &[u8]) -> Option<(Vec<(String, String)>, Imported)> {
    let value = cbor::decode(bytes)?;
    if uint_of(field(&value, "version")?)? != EMBIND_VERSION {
        return None;
    }
    let pairs = |value: &Value| -> Option<Vec<(String, String)>> {
        array_of(value)?.iter().map(|entry| Some((text_of(array_of(entry)?.first()?)?, text_of(array_of(entry)?.get(1)?)?))).collect()
    };
    let files = pairs(field(&value, "files")?)?;
    let items = array_of(field(&value, "items")?)?
        .iter()
        .map(|entry| {
            let entry = array_of(entry)?;
            Some(Item { name: text_of(entry.first()?)?, kind: *KINDS.get(uint_of(entry.get(1)?)? as usize)?, text: text_of(entry.get(2)?)?, namespace: read_namespace(entry.get(3)?)? })
        })
        .collect::<Option<Vec<_>>>()?;
    let functions = array_of(field(&value, "functions")?)?
        .iter()
        .map(|entry| {
            let link = text_of(field(entry, "link")?)?;
            let abi = text_of(field(entry, "abi")?)?;
            Some(Function {
                name: text_of(field(entry, "name")?)?,
                abi: ABIS.iter().find(|known| **known == abi)?,
                link_name: (!link.is_empty()).then_some(link),
                params: array_of(field(entry, "params")?)?
                    .iter()
                    .map(|param| Some((text_of(array_of(param)?.first()?)?, read_c_type(array_of(param)?.get(1)?)?)))
                    .collect::<Option<_>>()?,
                result: match array_of(field(entry, "result")?)? {
                    [] => None,
                    [ty] => Some(read_c_type(ty)?),
                    _ => return None,
                },
                policy: *POLICIES.get(uint_of(field(entry, "policy")?)? as usize)?,
                out: uint_of(field(entry, "out")?)? == 1,
                namespace: read_namespace(field(entry, "namespace")?)?,
                owner: match array_of(field(entry, "owner")?)? {
                    [] => None,
                    [owner] => Some((text_of(array_of(owner)?.first()?)?, *MEMBERS.get(uint_of(array_of(owner)?.get(1)?)? as usize)?)),
                    _ => return None,
                },
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let layouts = array_of(field(&value, "layouts")?)?
        .iter()
        .map(|entry| {
            let entry = array_of(entry)?;
            Some(RecordLayout {
                name: text_of(entry.first()?)?,
                size: uint_of(entry.get(1)?)?,
                align: uint_of(entry.get(2)?)?,
                fields: array_of(entry.get(3)?)?
                    .iter()
                    .map(|field| Some((text_of(array_of(field)?.first()?)?, uint_of(array_of(field)?.get(1)?)?)))
                    .collect::<Option<_>>()?,
                namespace: read_namespace(entry.get(4)?)?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let triples = |value: &Value| -> Option<Vec<(String, String, String)>> {
        array_of(value)?
            .iter()
            .map(|entry| {
                let entry = array_of(entry)?;
                Some((text_of(entry.first()?)?, text_of(entry.get(1)?)?, text_of(entry.get(2)?)?))
            })
            .collect()
    };
    let imported = Imported {
        items,
        header: PathBuf::from(text_of(field(&value, "header")?)?),
        skipped: read_skips(field(&value, "skipped")?)?,
        opaque: read_skips(field(&value, "opaque")?)?,
        macros: read_skips(field(&value, "macros")?)?,
        layouts,
        shim: text_of(field(&value, "shim")?)?,
        functions,
        uses: array_of(field(&value, "uses")?)?.iter().map(|path| array_of(path)?.iter().map(text_of).collect::<Option<Vec<_>>>()).collect::<Option<_>>()?,
        entities: triples(field(&value, "entities")?)?,
        conflicts: pairs(field(&value, "conflicts")?)?,
        macro_functions: array_of(field(&value, "macro_functions")?)?
            .iter()
            .map(|entry| {
                let entry = array_of(entry)?;
                Some(MacroFunction { name: text_of(entry.first()?)?, params: entry[1..].iter().map(text_of).collect::<Option<_>>()? })
            })
            .collect::<Option<_>>()?,
        c_types: pairs(field(&value, "c_types")?)?,
        anonymous_namespace: read_skips(field(&value, "anonymous_namespace")?)?,
        rvalue_members: read_skips(field(&value, "rvalue_members")?)?,
        namespaces: array_of(field(&value, "namespaces")?)?.iter().map(read_namespace).collect::<Option<_>>()?,
        ns_uses: pairs(field(&value, "ns_uses")?)?.into_iter().map(|(from, to)| (split_namespace(&from), split_namespace(&to))).collect(),
    };
    Some((files, imported))
}

/// The hash of a file's bytes, or nothing when it cannot be read.
fn file_hash(path: &str) -> Option<String> {
    std::fs::read(path).ok().map(|bytes| ember_build::content_hash(&bytes))
}

/// `[FFI-7]` — the import from its `.embind` record when nothing that can change it has: the key is
/// the Clang configuration (libclang, standard, target, flags, defines, include paths), the header as
/// written and where from, the overlay's hash and the declarations other imports of the build wrote;
/// the record holds every file the header included with its hash, each read again here. Otherwise
/// the header is parsed and the record written.
fn import_cached(config: &Config, overlay: &str, cached: &mut bool) -> Result<(Imported, PathBuf), String> {
    load_libclang()?;
    let overlay_hash = if overlay.is_empty() { String::new() } else { ember_build::content_hash(overlay.as_bytes()) };
    let library = clang_sys::get_library().map(|library| library.path().display().to_string()).unwrap_or_default();
    let mut shared: Vec<String> = config
        .shared
        .iter()
        .map(|(usr, shared)| format!("{usr}={}:{}:{}:{}", shared.module.join("."), shared.name, shared.layout, shared.binding))
        .collect();
    shared.sort();
    let configuration = format!("{library}\n{}\n{}\n{}", arguments(config).join(" "), config.header, std::path::absolute(config.from_dir).unwrap_or_default().display());
    // The compiler itself is part of the key: another build of the importer may write another module.
    let key = ember_build::content_hash(
        format!(
            "{EMBIND_VERSION} {}\n{configuration}\n{}\n{overlay_hash}\n{}\n{}",
            ember_build::interface::compiler_identity(),
            config.symbol_stem,
            config.binding,
            shared.join("\n")
        )
        .as_bytes(),
    );
    let path = ember_build::cache_root().join(ember_branding::BINDING_EXT).join(format!("{key}.{}", ember_branding::BINDING_EXT));
    if let Some((files, imported)) = std::fs::read(&path).ok().and_then(|bytes| read_embind(&bytes))
        && files.iter().all(|(file, hash)| file_hash(file).as_deref() == Some(hash.as_str()))
    {
        *cached = true;
        return Ok((imported, path));
    }
    let (imported, files) = import_header(config)?;
    let files: Vec<(String, String)> = files.into_iter().filter_map(|file| file_hash(&file).map(|hash| (file, hash))).collect();
    let bytes = embind(&imported, &files, &configuration, &overlay_hash, overlay);
    // Written aside then renamed: builds side by side read whole records only.
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
        let partial = dir.join(format!("{key}.{}.partial", std::process::id()));
        if std::fs::write(&partial, &bytes).is_ok() && std::fs::rename(&partial, &path).is_err() {
            let _ = std::fs::remove_file(&partial);
        }
    }
    Ok((imported, path))
}

fn opaque_by_value(name: &str, why: Option<&str>) -> String {
    match why {
        Some(why) => format!("`{name}` by value, which is opaque: {why}"),
        None => format!("`{name}` by value, which the header declares but never defines"),
    }
}

/// `[FFI-9]` — the block a calling convention's functions go in.
fn convention(convention: CXCallingConv) -> Option<&'static str> {
    match convention {
        CXCallingConv_C | CXCallingConv_X86_64Win64 | CXCallingConv_X86_64SysV => Some("C"),
        CXCallingConv_X86StdCall => Some("system"),
        CXCallingConv_X86FastCall => Some("fastcall"),
        CXCallingConv_X86VectorCall => Some("vectorcall"),
        _ => None,
    }
}

/// A C declaration of `name` with type `ty`: `int (*name)(int)`, `char name[4]`.
fn c_declaration(ty: CXType, name: &str) -> String {
    // SAFETY(ffi): types of a live translation unit.
    unsafe {
        match ty.kind {
            CXType_Pointer => {
                let pointee = clang_getPointeeType(ty);
                let qualifier = if clang_isConstQualifiedType(ty) != 0 { "const " } else { "" };
                let inner = format!("*{qualifier}{name}");
                match pointee.kind {
                    CXType_FunctionProto | CXType_FunctionNoProto | CXType_ConstantArray | CXType_IncompleteArray => {
                        c_declaration(pointee, &format!("({inner})"))
                    }
                    _ => c_declaration(pointee, &inner),
                }
            }
            CXType_ConstantArray => c_declaration(clang_getArrayElementType(ty), &format!("{name}[{}]", clang_getArraySize(ty))),
            CXType_IncompleteArray => c_declaration(clang_getArrayElementType(ty), &format!("{name}[]")),
            CXType_FunctionProto => {
                let count = clang_getNumArgTypes(ty).max(0) as u32;
                let mut params: Vec<String> = (0..count).map(|index| c_declaration(clang_getArgType(ty, index), "")).collect();
                if params.is_empty() {
                    params.push("void".to_string());
                }
                c_declaration(clang_getResultType(ty), &format!("{name}({})", params.join(", ")))
            }
            _ => format!("{} {name}", type_spelling(ty)).trim_end().to_string(),
        }
    }
}

/// `[FFI-8]` — a C integer type as the fixed-width Ember one of its size and signedness.
fn fixed_integer(ty: CXType) -> Result<String, String> {
    // SAFETY(ffi): a type of a live translation unit.
    let (size, kind) = unsafe { (clang_Type_getSizeOf(ty), clang_getCanonicalType(ty).kind) };
    let signed = matches!(kind, CXType_Char_S | CXType_SChar | CXType_Short | CXType_Int | CXType_Long | CXType_LongLong)
        || (kind == CXType_WChar && !cfg!(windows));
    let bits = match size {
        1 => "8",
        2 => "16",
        4 => "32",
        8 => "64",
        _ => return Err(format!("a {size}-byte integer")),
    };
    Ok(format!("{}{bits}", if signed { "i" } else { "u" }))
}

/// The unsigned integer Clang evaluates a variable's initialiser to.
fn evaluate_unsigned(cursor: CXCursor) -> Option<u64> {
    // SAFETY(ffi): a live variable cursor; the result is disposed.
    unsafe {
        let result = clang_Cursor_Evaluate(cursor);
        if result.is_null() {
            return None;
        }
        let value = (clang_EvalResult_getKind(result) == CXEval_Int).then(|| clang_EvalResult_getAsUnsigned(result));
        clang_EvalResult_dispose(result);
        value
    }
}

/// An Ember float literal that reads back as `value`.
fn float_literal(value: f64) -> String {
    let written = format!("{value:?}");
    match written.split_once('e') {
        Some((mantissa, exponent)) if !mantissa.contains('.') => format!("{mantissa}.0e{exponent}"),
        _ => written,
    }
}

/// The text of a `c"…"` literal.
fn escape(string: &str) -> String {
    let mut out = String::new();
    for character in string.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            character if (character as u32) < 0x20 || character as u32 == 0x7f => {
                let _ = write!(out, "\\x{:02x}", character as u32);
            }
            character => out.push(character),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `[FFI-6a]` — a constant's literal reads back as the value Clang gave.
    #[test]
    fn constant_literals_read_back() {
        assert_eq!(float_literal(1.5), "1.5");
        assert_eq!(float_literal(1e300), "1.0e300");
        assert_eq!(escape("a\"b\\c\t\u{1}"), "a\\\"b\\\\c\\t\\x01");
        assert_eq!(ident("type"), "r#type");
        assert_eq!(ident("width"), "width");
    }
}
