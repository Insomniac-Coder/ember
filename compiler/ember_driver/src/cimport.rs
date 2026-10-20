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
}

/// A foreign function as the header declares it, for an overlay to contract (`[FFI-2]`).
#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    /// Each parameter's Ember name and C type.
    pub params: Vec<(String, CType)>,
    pub result: Option<CType>,
    /// The symbol it binds when that is not its name: a shim wrapper (`[FFI-29a]`).
    pub link_name: Option<String>,
    pub abi: &'static str,
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
    })
}

/// The arguments libclang parses with: C11, as the generated C is compiled, for the host target.
fn arguments(config: &Config) -> Vec<String> {
    let mut args = vec!["-x".to_string(), "c".to_string(), "-std=c11".to_string()];
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
    let main = from_dir.join(format!("{}_import_c.c", ember_branding::SYMBOL_PREFIX));
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
    let header = inclusions.iter().find(|(_, depth)| *depth == 1).map(|(file, _)| file.clone()).ok_or_else(|| format!("`{}` was not found", config.header))?;
    let files: Vec<String> = inclusions.into_iter().filter(|(_, depth)| *depth > 0).map(|(file, _)| file).collect();
    let header_path = std::path::absolute(from_dir.join(&header)).unwrap_or_else(|_| PathBuf::from(&header));
    let mut writer = Writer {
        header: header.clone(),
        stem: config.symbol_stem.to_string(),
        shared: config.shared.clone(),
        binding: config.binding.to_string(),
        ..Writer::default()
    };
    for cursor in children(unit.cursor()) {
        let (file, _, system) = location(cursor);
        if file != header && (system || file.is_empty() || Path::new(&file) == main) {
            continue;
        }
        writer.declaration(cursor);
    }
    // `[FFI-6a]` — the object-like macros, evaluated by Clang in a second parse.
    let macros = writer.macro_names.clone();
    if !macros.is_empty() {
        let mut probe = include.clone();
        for (index, (name, _)) in macros.iter().enumerate() {
            // Unparenthesised: Clang evaluates a string only as the literal itself.
            let _ = writeln!(probe, "static const __auto_type {}_macro_{index} = {name};", ember_branding::SYMBOL_PREFIX);
        }
        let unit = Unit::parse(&main, &probe, &args)?;
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
        for (index, (name, at)) in macros.iter().enumerate() {
            let outcome = match (reasons.get(&index), values.get(&index)) {
                (Some(reason), _) => Err(format!("Clang: {reason}")),
                (None, Some(&cursor)) => writer.constant_value(cursor),
                (None, None) => Err("Clang made no declaration of its value".to_string()),
            };
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
    /// Ember names taken in the module.
    names: HashSet<String>,
    records: HashMap<String, Record>,
    enums: HashMap<String, Result<String, String>>,
    typedefs: HashMap<String, Result<String, String>>,
    seen: HashSet<String>,
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
        self.items.push(Item { name: name.to_string(), kind, text: text.trim_end().to_string() });
    }

    /// Take `name` in the module's one namespace; C's tags and ordinary names are two.
    fn claim(&mut self, name: &str) -> bool {
        self.names.insert(name.to_string())
    }

    fn declaration(&mut self, cursor: CXCursor) {
        match kind(cursor) {
            CXCursor_FunctionDecl => self.function(cursor),
            CXCursor_StructDecl | CXCursor_UnionDecl => {
                // SAFETY(ffi): a live cursor.
                if unsafe { clang_Cursor_isAnonymous(cursor) } == 0 {
                    let _ = self.record(cursor, None);
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
            let size = clang_Type_getSizeOf(ty);
            let int = |signed: bool| -> Result<String, String> {
                let bits = match size {
                    1 => "8",
                    2 => "16",
                    4 => "32",
                    8 => "64",
                    _ => return Err(format!("a {size}-byte integer")),
                };
                Ok(format!("{}{bits}", if signed { "i" } else { "u" }))
            };
            match ty.kind {
                CXType_Void => Ok("void".to_string()),
                CXType_Bool => Ok("bool".to_string()),
                CXType_Char_S | CXType_SChar | CXType_Short | CXType_Int | CXType_Long | CXType_LongLong => int(true),
                CXType_Char_U | CXType_UChar | CXType_UShort | CXType_UInt | CXType_ULong | CXType_ULongLong
                | CXType_Char16 | CXType_Char32 => int(false),
                // `c_wchar`: `u16` on Windows, `i32` elsewhere.
                CXType_WChar => int(!cfg!(windows)),
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
                        return Ok(alias);
                    }
                    self.ty(clang_getTypedefDeclUnderlyingType(declaration), behind_pointer)
                }
                CXType_Record => {
                    let declaration = clang_getTypeDeclaration(ty);
                    // A struct with no tag is known by its typedef's name once that is written
                    // (`typedef struct {…} Rect`, seen through a canonical type).
                    if clang_Cursor_isAnonymous(declaration) != 0 && !self.records.contains_key(&usr(declaration)) {
                        return Err("an anonymous struct or union member (its generated `<Parent>_anon<N>` type is not built yet)".to_string());
                    }
                    match self.record(declaration, None) {
                        Record::Full(name) | Record::Pending(name) => Ok(name),
                        Record::Opaque(name, _) if behind_pointer => Ok(name),
                        Record::Opaque(name, why) => Err(opaque_by_value(&name, why.as_deref())),
                    }
                }
                CXType_Enum => {
                    let declaration = clang_getTypeDeclaration(ty);
                    if clang_Cursor_isAnonymous(declaration) != 0 {
                        return self.ty(clang_getEnumDeclIntegerType(declaration), false);
                    }
                    self.enumeration(declaration, None)
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
        let Some(pointee) = pointee else { return Ok(CType { ember, pointee: None, char_pointer: false, c }) };
        // SAFETY(ffi): as above.
        let (constant, character) = unsafe {
            let kind = clang_getCanonicalType(pointee).kind;
            (clang_isConstQualifiedType(pointee) != 0, matches!(kind, CXType_Char_S | CXType_Char_U))
        };
        let inner = self.ty(pointee, true)?;
        Ok(CType { ember, pointee: Some((inner, constant)), char_pointer: character, c })
    }

    fn typedef(&mut self, cursor: CXCursor) -> Result<String, String> {
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
            self.records.insert(key.clone(), Record::Pending(name.clone()));
            let ty = clang_getCursorType(definition);
            let mut fields = Vec::new();
            let mut layout = Vec::new();
            let mut natural = 1u64;
            let mut packed = false;
            for child in children(definition) {
                match kind(child) {
                    CXCursor_PackedAttr => packed = true,
                    CXCursor_FieldDecl => {
                        let field = spelling(child);
                        if clang_Cursor_isBitField(child) != 0 {
                            return self.opaque(cursor, &key, &named, Some(format!("bit-field `{field}` (its `get_`/`set_` methods are not built yet)")));
                        }
                        if field.is_empty() || clang_Cursor_isAnonymousRecordDecl(clang_getTypeDeclaration(clang_getCursorType(child))) != 0 {
                            return self.opaque(cursor, &key, &named, Some("an anonymous member (its generated `<Parent>_anon<N>` type is not built yet)".to_string()));
                        }
                        let field_ty = clang_getCursorType(child);
                        let mapped = match self.ty(field_ty, false) {
                            Ok(mapped) => mapped,
                            Err(reason) => return self.opaque(cursor, &key, &named, Some(format!("field `{field}`: {reason}"))),
                        };
                        natural = natural.max(clang_Type_getAlignOf(field_ty).max(1) as u64);
                        layout.push((ident(&field), (clang_Cursor_getOffsetOfField(child).max(0) as u64) / 8));
                        fields.push(format!("    pub {}: {mapped}\n", ident(&field)));
                    }
                    _ => {}
                }
            }
            if fields.is_empty() {
                return self.opaque(cursor, &key, &named, Some("a struct with no fields (a GNU C extension)".to_string()));
            }
            let size = clang_Type_getSizeOf(ty).max(0) as u64;
            let align = clang_Type_getAlignOf(ty).max(1) as u64;
            let fingerprint = format!("struct {size} {align} {:?}", layout.iter().map(|(_, offset)| offset).collect::<Vec<_>>());
            if let Some(adopted) = self.adopt(&key, &named, &fingerprint, false) {
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
            self.push(&named, Kind::Type, format!("{attributes}struct {name}:\n{}", fields.concat()));
            // `[FFI-5a]` — the project's C compiler checks the layout on every build.
            // `struct Point`, or the typedef name of a struct with no tag of its own.
            let spelled = type_spelling(ty);
            let c_name = if spelled.contains('(') { named.clone() } else { spelled };
            let _ = writeln!(self.shim, "_Static_assert(sizeof({c_name}) == {size}, \"{named}: size\");");
            let _ = writeln!(self.shim, "_Static_assert(_Alignof({c_name}) == {align}, \"{named}: alignment\");");
            for (field, offset) in &layout {
                let c_field = field.trim_start_matches("r#");
                let _ = writeln!(self.shim, "_Static_assert(offsetof({c_name}, {c_field}) == {offset}, \"{named}.{c_field}: offset\");");
            }
            self.layouts.push(RecordLayout { name: name.clone(), size, align, fields: layout });
            self.entities.push((key.clone(), name.clone(), fingerprint));
            self.records.insert(key, Record::Full(name.clone()));
            Record::Full(name)
        }
    }

    /// `[FFI-30]` — a declaration an earlier import of the build wrote with this layout is that
    /// import's entity: here an alias of it. With another layout it is a distinct entity, `E5011`
    /// unless this import binds a name of its own. `any`: an incomplete struct takes any layout.
    fn adopt(&mut self, key: &str, named: &str, layout: &str, any: bool) -> Option<Record> {
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
        let record = if shared.layout == "opaque" { Record::Opaque(name, None) } else { Record::Full(name) };
        self.records.insert(key.to_string(), record.clone());
        Some(record)
    }

    /// An `extern type`: used only behind pointers. A definition it could not represent is
    /// `W5002`; an incomplete struct is represented exactly.
    fn opaque(&mut self, cursor: CXCursor, key: &str, named: &str, why: Option<String>) -> Record {
        // An incomplete struct is the entity another import wrote, whatever its layout there.
        if why.is_none()
            && let Some(adopted) = self.adopt(key, named, "", true)
        {
            return adopted;
        }
        if !self.claim(named) {
            return self.opaque_unclaimed(cursor, key, named);
        }
        let name = ident(named);
        self.push(named, Kind::Opaque, format!("type {name}"));
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
        let key = usr(cursor);
        if let Some(done) = self.enums.get(&key) {
            return done.clone();
        }
        let named = typedef_name.map_or_else(|| spelling(cursor), str::to_string);
        let name = ident(&named);
        // SAFETY(ffi): a live enum cursor.
        let repr = self.ty(unsafe { clang_getEnumDeclIntegerType(cursor) }, false);
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
            && self.adopt(&key, &named, &layout(repr), false).is_some()
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
        if !self.seen.insert(format!("fn {name}")) {
            return;
        }
        match self.function_parts(cursor, &name) {
            Ok(function) => {
                if !self.claim(&name) {
                    self.skip(cursor, &name, "its name is taken by another declaration of the header");
                    return;
                }
                self.functions.push(function);
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
            // `[FFI-29a]` — a function with internal linkage is reached through the shim's wrapper.
            let link_name = (clang_getCursorLinkage(cursor) == CXLinkage_Internal).then(|| {
                let wrapper = format!("{}_cshim_{}_{name}", ember_branding::SYMBOL_PREFIX, self.stem);
                self.wrapper(cursor, &wrapper, name);
                wrapper
            });
            Ok(Function { name: name.to_string(), params, result, link_name, abi })
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
            let _ = writeln!(self.shim, "{} {{ {body} }}", c_declaration(result, &format!("{wrapper}({params})")));
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
    /// to nothing (an include guard, a feature flag) has no value to import.
    fn macro_definition(&mut self, cursor: CXCursor) {
        let name = spelling(cursor);
        // SAFETY(ffi): a live macro cursor and its tokens.
        unsafe {
            if clang_Cursor_isMacroBuiltin(cursor) != 0 || !self.seen.insert(format!("macro {name}")) {
                return;
            }
            let at = self.at(cursor);
            if clang_Cursor_isMacroFunctionLike(cursor) != 0 {
                self.macro_skipped.push(Skipped {
                    name,
                    reason: "a function-like macro: Ember has no macros (a `static inline` C function in a header of your own can call it)".to_string(),
                    at,
                });
                return;
            }
            let tu = clang_Cursor_getTranslationUnit(cursor);
            let mut tokens: *mut CXToken = std::ptr::null_mut();
            let mut count: c_uint = 0;
            clang_tokenize(tu, clang_getCursorExtent(cursor), &mut tokens, &mut count);
            clang_disposeTokens(tu, tokens, count);
            if count > 1 {
                self.macro_names.push((name, at));
            }
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
                    return Err("a pointer constant: Ember's compile-time values cannot be pointers ([CT-5])".to_string());
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
        let shim = format!(
            "/* The shim of `import c \"{}\"` ([FFI-29]): the layout checks and wrappers Ember needs. */\n#include <stddef.h>\n#include \"{}\"\n\n{}",
            header.display(),
            header.to_string_lossy().replace('\\', "/"),
            self.shim
        );
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
        }
    }
}

/// The options an `import c` takes (`[FFI-6]`).
#[derive(Default)]
struct Options {
    include_paths: Vec<String>,
    defines: Vec<String>,
    link: Vec<String>,
    overlay: Option<(String, Span)>,
    implementation: Vec<String>,
}

const OPTIONS: &str = "`include_paths`, `defines`, `link`, `overlay` and `implementation`";

/// `[FFI-6]` — the options, each a list of strings but `overlay`, a string. An unknown option, or
/// one written otherwise, is `E0104`.
fn options(written: &[AttrArg], sink: &mut Sink) -> Option<Options> {
    let mut options = Options::default();
    let mut ok = true;
    for arg in written {
        let AttrArg::Named { name, value } = arg else {
            let AttrArg::Expr(expr) = arg else { unreachable!() };
            sink.emit(
                Diagnostic::error(codes::E0104, expr.span, "an option of `import c` is written `name = value`")
                    .help(format!("the options are {OPTIONS} [FFI-6]")),
            );
            ok = false;
            continue;
        };
        let strings = |value: &Expr| -> Option<Vec<String>> {
            let ExprKind::ArrayLit(items) = &value.kind else { return None };
            items.iter().map(|item| match &item.kind { ExprKind::Lit(Literal::Str(text)) => Some(text.clone()), _ => None }).collect()
        };
        let list = match name.name.as_str() {
            "include_paths" => &mut options.include_paths,
            "defines" => &mut options.defines,
            "link" => &mut options.link,
            "implementation" => &mut options.implementation,
            "overlay" => {
                if let ExprKind::Lit(Literal::Str(path)) = &value.kind {
                    options.overlay = Some((path.clone(), value.span));
                } else {
                    sink.emit(Diagnostic::error(codes::E0104, value.span, "`overlay` names one overlay file: a string")
                        .help(format!("write `overlay = \"overlays/name.{}\"`", ember_branding::SOURCE_EXT)));
                    ok = false;
                }
                continue;
            }
            other => {
                sink.emit(
                    Diagnostic::error(codes::E0104, name.span, format!("`{other}` is not an option of `import c`"))
                        .help(format!("the options are {OPTIONS} [FFI-6]")),
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
    /// The import's overlay file, a module of its own whose entries are checked as any overlay's
    /// (`[GRM-35]`), with the function entries whose `instrumented` grade evidence backs (`[TCB-1]`).
    pub overlay: Option<(Vec<String>, ast::Module, Vec<String>)>,
}

/// `[FFI-6]` — the module an `import c` names, written from its header, and the import rewritten
/// to bind it (to `name`, or `c`). `known` maps each import already loaded (its header, folder and
/// options) to its module's path: a second import of it binds the same module and returns `None`.
/// `base`: where an option's relative path is found (the package's folder, else the importing
/// file's); `flags`: the package's `[c] flags` (`[BLD-FFI-1a]`).
pub fn load(
    import: &mut ast::Import,
    importer: &Path,
    base: &Path,
    flags: &[String],
    known: &mut Known,
    map: &mut SourceMap,
    sink: &mut Sink,
) -> Option<Loaded> {
    let span = import.span;
    let ast::ImportKind::Foreign { language, header, options: written, alias } = &import.kind else { return None };
    if *language == ast::ForeignLanguage::Cpp {
        sink.emit(Diagnostic::error(codes::E0900, span, "`import cpp` is not implemented yet (Annex C)"));
        return None;
    }
    let options = options(written, sink)?;
    let bound = alias.map_or_else(|| Symbol::intern("c"), |alias| alias.name);
    // A bare file name's folder is the working one.
    let from_dir = importer.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or(Path::new(".")).to_path_buf();
    // `[BLD-FFI-1]` — `import c "./bridge.c"` compiles `bridge.c` and imports `bridge.h`.
    let (header, source) = match header.strip_suffix(".c") {
        Some(stem) => (format!("{stem}.h"), Some(from_dir.join(header))),
        None => (header.clone(), None),
    };
    let key = format!(
        "{}|{header}|{:?}|{:?}|{:?}|{:?}",
        from_dir.display(),
        options.include_paths,
        options.defines,
        options.overlay.as_ref().map(|o| &o.0),
        options.implementation
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
    let mut path = vec!["c".to_string(), format!("h_{stem}")];
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
    };
    // `[FFI-7]` — the overlay is part of what the record is for.
    let overlay_text = options.overlay.as_ref().and_then(|(written, _)| std::fs::read_to_string(base.join(written)).ok()).unwrap_or_default();
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
    for skipped in &imported.skipped {
        sink.emit(
            Diagnostic::warning(codes::W5002, span, format!("`{}` is not imported from `{shown}`: {}", skipped.name, skipped.reason))
                .note(format!("declared at {} [FFI-20a]", skipped.at))
                .help("to use it, declare it by hand in an `unsafe extern \"C\":` block with types Ember has, or wrap it in a C function the header declares"),
        );
    }
    for opaque in &imported.opaque {
        sink.emit(
            Diagnostic::warning(codes::W5002, span, format!("`{}` is imported from `{shown}` only as an opaque type, usable behind a pointer: {}", opaque.name, opaque.reason))
                .note(format!("declared at {} [FFI-20a]", opaque.at))
                .help("to reach its fields, wrap them in C functions the header declares"),
        );
    }
    for skipped in &imported.macros {
        sink.emit(
            Diagnostic::warning(codes::W5001, span, format!("macro `{}` is not imported from `{shown}`: {}", skipped.name, skipped.reason))
                .note(format!("defined at {} [FFI-6a]", skipped.at)),
        );
    }
    // `[FFI-2]` — the overlay, applied before the module is written.
    let mut applied = Applied::default();
    let mut overlay_module = None;
    if let Some((written, at)) = &options.overlay {
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
                        ast::ItemKind::Overlay(overlay) if overlay.language == ast::ForeignLanguage::C => Some(overlay),
                        _ => None,
                    })
                    .collect();
                match overlays.as_slice() {
                    [overlay] => applied = apply(overlay, &text, &shown, &imported, sink),
                    _ => sink.emit(
                        Diagnostic::error(codes::E5010, *at, format!("`{written}` holds {} `overlay c` declarations: an import's overlay file holds one", overlays.len()))
                            .note("an overlay states the contracts of one header's declarations [FFI-12]"),
                    ),
                }
                let mut overlay_path = path.clone();
                overlay_path[1].push_str("_overlay");
                overlay_module = Some((overlay_path, parsed, Vec::new()));
            }
        }
    }
    // Named for its module: the source map keeps one file per name, and one header imported twice
    // is two modules.
    // No quotes: the name reaches the C as `#line`'s file name ([CG-C-8]).
    let rendered = render(&imported, &applied);
    // `[TCB-1]` — each `@grade(instrumented)` entry an evidence record of this package backs.
    if let Some((_, overlay, backed)) = &mut overlay_module
        && let Some(evidence) = crate::foreign::load_evidence(importer)
    {
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
            if graded && crate::foreign::backs_instrumented(&evidence, &qualified, &source_hash) {
                backed.push(name.name.to_string());
            }
        }
    }
    let file = map.add(PathBuf::from(format!("<import c {header} as {}>", path.join("."))), rendered);
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
    let mut c_flags: Vec<String> = flags.to_vec();
    c_flags.extend(include_paths.iter().map(|dir| format!("-I{}", dir.display())));
    c_flags.extend(options.defines.iter().map(|define| format!("-D{define}")));
    let mut identities = vec![("header".to_string(), imported.header.clone())];
    identities.extend(options.overlay.as_ref().map(|(written, _)| ("overlay".to_string(), base.join(written))));
    let foreign = ember_typeck::ForeignImport {
        backed: Vec::new(),
        header: shown,
        identities,
        skipped: imported
            .skipped
            .iter()
            .map(|skipped| (skipped.name.clone(), skipped.reason.clone()))
            .chain(imported.macros.iter().map(|skipped| (skipped.name.clone(), format!("a macro, {}", skipped.reason))))
            .collect(),
        layouts: imported.layouts.iter().map(|layout| (layout.name.clone(), layout.size, layout.align, layout.fields.clone())).collect(),
        shim,
        sources: source.into_iter().collect(),
        flags: c_flags,
        link: options
            .link
            .iter()
            .map(|library| if ember_build::cdylib::is_library_path(library) { base.join(library).to_string_lossy().into_owned() } else { library.clone() })
            .collect(),
    };
    rewrite(import, &path);
    known.modules.insert(key, path.clone());
    Some(Loaded { path, module, foreign, overlay: overlay_module })
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

/// `[FFI-2]`, `[FFI-12]`, `[FFI-13]`, `[FFI-30a]` — what an overlay changes in an imported module:
/// the contracts, safety and names of functions, facts on opaque types and enums, what is hidden,
/// and the wrappers its `extend` blocks add. It never changes a type's identity, layout or fields.
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
}

/// An overlay's function entry: each parameter's contract words, the result's, and the entry's own
/// `@ffi(…)` arguments, each as written.
struct Entry {
    params: Vec<Vec<String>>,
    result: Vec<String>,
    arguments: Vec<String>,
}

/// The word of a contract word as written: `count` of `count(n)`.
fn word(written: &str) -> &str {
    written.split('(').next().unwrap_or(written).trim()
}

/// `[FFI-12]` — apply an overlay to the module the header made; an entry whose name, parameter
/// count or parameter names do not match the header is `E5010`.
fn apply(overlay: &ast::OverlayDecl, text: &str, header: &str, imported: &Imported, sink: &mut Sink) -> Applied {
    let snippet = |span: Span| text.get(span.start as usize..span.end as usize).unwrap_or("").to_string();
    let mut applied = Applied::default();
    let written = Path::new(&overlay.header).file_name().map(|name| name.to_string_lossy().into_owned());
    if written.as_deref() != Some(header) {
        sink.emit(
            Diagnostic::error(codes::E5010, overlay.span, format!("this overlay is for `{}`, and the import is of `{header}`", overlay.header))
                .note("an overlay states the contracts of one header's declarations [FFI-12]"),
        );
        return applied;
    }
    let item = |name: &str, kind: Kind| imported.items.iter().any(|item| item.name == name && item.kind == kind);
    let mismatch = |sink: &mut Sink, span: Span, message: String| {
        sink.emit(Diagnostic::error(codes::E5010, span, message).note("an overlay entry matches the header's declaration, so overlays cannot drift [FFI-12]"));
    };
    for entry in &overlay.items {
        let attributes = || entry.attrs.iter().map(|attr| snippet(attr.span)).collect::<Vec<_>>();
        match &entry.kind {
            ast::OverlayItemKind::Fn { name, params, result } => {
                let Some(function) = imported.functions.iter().find(|function| function.name == name.name.as_str()) else {
                    mismatch(sink, name.span, format!("`{header}` declares no function `{}`", name.name));
                    continue;
                };
                if applied.functions.contains_key(&function.name) {
                    mismatch(sink, name.span, format!("a second entry for `{}`", name.name));
                    continue;
                }
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
                if !matched {
                    continue;
                }
                let arguments = entry
                    .attrs
                    .iter()
                    .filter(|attr| attr.path.len() == 1 && attr.path[0].name.is("ffi"))
                    .flat_map(|attr| &attr.args)
                    .map(|arg| match arg {
                        AttrArg::Expr(expr) => snippet(expr.span),
                        AttrArg::Named { name, value } => snippet(name.span.to(value.span)),
                    })
                    .collect();
                applied.functions.insert(
                    function.name.clone(),
                    Entry {
                        params: params.iter().map(|param| param.contract.iter().map(|word| snippet(word.span)).collect()).collect(),
                        result: result.iter().map(|word| snippet(word.span)).collect(),
                        arguments,
                    },
                );
            }
            ast::OverlayItemKind::Type(name) | ast::OverlayItemKind::Enum(name) => {
                let kind = if matches!(entry.kind, ast::OverlayItemKind::Type(_)) { Kind::Opaque } else { Kind::Type };
                let found = imported.items.iter().any(|item| item.name == name.name.as_str() && item.kind == kind && (kind == Kind::Opaque || item.text.starts_with("@repr")));
                if !found {
                    let what = if kind == Kind::Opaque { "opaque type" } else { "enum" };
                    mismatch(sink, name.span, format!("`{header}` declares no {what} `{}`", name.name));
                    continue;
                }
                applied.attributes.entry(name.name.to_string()).or_default().extend(attributes());
            }
            ast::OverlayItemKind::Rename { from, to } => {
                let function = imported.functions.iter().any(|function| function.name == from.name.as_str());
                if !function && !item(from.name.as_str(), Kind::Type) && !item(from.name.as_str(), Kind::Opaque) {
                    mismatch(sink, from.span, format!("`{header}` declares no function or type `{}` to rename", from.name));
                    continue;
                }
                applied.renamed.insert(from.name.to_string(), to.name.to_string());
            }
            ast::OverlayItemKind::Hide(name) => {
                let function = imported.functions.iter().any(|function| function.name == name.name.as_str());
                if !function && !imported.items.iter().any(|item| item.name == name.name.as_str()) {
                    mismatch(sink, name.span, format!("`{header}` declares no `{}` to hide", name.name));
                    continue;
                }
                applied.hidden.insert(name.name.to_string());
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
    applied
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
    let mut params: Vec<String> = function.params.iter().map(|(name, ty)| format!("{name}: {}", ty.ember)).collect();
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
            params = function.params.iter().zip(&entry.params).map(|((param, ty), words)| safe_parameter(param, ty, words)).collect();
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
fn render(imported: &Imported, applied: &Applied) -> String {
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
    for item in imported.items.iter().filter(|item| matches!(item.kind, Kind::Type | Kind::Const)) {
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
    for item in imported.items.iter().filter(|item| matches!(item.kind, Kind::Opaque | Kind::Static)) {
        member("C", declare(item, "    "));
        if let Some(alias) = alias(item) {
            body.push_str(&alias);
        }
    }
    // A hidden declaration stays, private to the module: another may name it (`destructor=f`).
    for function in &imported.functions {
        let hidden = applied.hidden.contains(&function.name);
        member(function.abi, render_function(function, applied.functions.get(&function.name), applied.renamed.get(&function.name), hidden));
    }
    for (abi, members) in blocks {
        let _ = write!(body, "unsafe extern \"{abi}\":\n{members}\n");
    }
    for extend in &applied.extends {
        let _ = write!(body, "{extend}\n\n");
    }
    let mut imports = String::new();
    for module in &imported.uses {
        let _ = writeln!(imports, "import {} as {}_{}", module.join("."), ember_branding::SYMBOL_PREFIX, module.join("_"));
    }
    if body.contains("ForeignBox[") {
        imports.push_str("from std.ffi import ForeignBox\n");
    }
    if !imports.is_empty() {
        imports.push('\n');
    }
    format!("# The declarations of `{header}`, written by `import c` ([FFI-6]).\n\n{imports}{body}")
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
        binding: "c",
    };
    let (imported, files) = import_header(&config)?;
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
            [overlay] => Ok(apply(overlay, &text, &file, &imported, sink)),
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

const EMBIND_VERSION: u64 = 1;

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
                        Value::Array(vec![Value::Text(item.name.clone()), Value::Uint(kind), Value::Text(item.text.clone())])
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
            Some(Item { name: text_of(entry.first()?)?, kind: *KINDS.get(uint_of(entry.get(1)?)? as usize)?, text: text_of(entry.get(2)?)? })
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
