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
    pub at: String,
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
}

pub struct Imported {
    /// The module's Ember source.
    pub text: String,
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
        "size_t" | "uintptr_t" => "usize",
        "ptrdiff_t" | "intptr_t" | "ssize_t" => "isize",
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

    /// The files the main file includes itself.
    fn included_directly(&self) -> Vec<String> {
        let mut out: Vec<(String, u32)> = Vec::new();
        // SAFETY(ffi): the visitor only pushes into `out`.
        unsafe { clang_getInclusions(self.tu, inclusion, &mut out as *mut Vec<(String, u32)> as CXClientData) };
        out.into_iter().filter(|(_, depth)| *depth == 1).map(|(file, _)| file).collect()
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
pub fn import_header(config: &Config) -> Result<Imported, String> {
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
        .map(|note| if note.file.is_empty() { note.text } else { format!("{}:{}: {}", note.file, note.line, note.text) })
        .collect();
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let header = unit.included_directly().into_iter().next().ok_or_else(|| format!("`{}` was not found", config.header))?;
    let header_path = std::path::absolute(from_dir.join(&header)).unwrap_or_else(|_| PathBuf::from(&header));
    let mut writer = Writer { header: header.clone(), stem: config.symbol_stem.to_string(), ..Writer::default() };
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
                    let _ = writeln!(writer.items, "pub const {}: {ty} = {value}\n", ident(name));
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
    Ok(writer.finish(header_path))
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
    /// Types, constants and aliases, in the order written.
    items: String,
    /// The `unsafe extern "C"` block's members, by calling convention.
    externs: Vec<(&'static str, String)>,
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
                    if clang_Cursor_isAnonymous(declaration) != 0 {
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
        let Some(pointee) = pointee else { return Ok(CType { ember, pointee: None, char_pointer: false }) };
        // SAFETY(ffi): as above.
        let (constant, character) = unsafe {
            let kind = clang_getCanonicalType(pointee).kind;
            (clang_isConstQualifiedType(pointee) != 0, matches!(kind, CXType_Char_S | CXType_Char_U))
        };
        let inner = self.ty(pointee, true)?;
        Ok(CType { ember, pointee: Some((inner, constant)), char_pointer: character })
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
                let _ = writeln!(self.items, "pub type {} = {target}\n", ident(&name));
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
            if !self.claim(&named) {
                return self.opaque_unclaimed(cursor, &key, &named);
            }
            let size = clang_Type_getSizeOf(ty).max(0) as u64;
            let align = clang_Type_getAlignOf(ty).max(1) as u64;
            packed |= align == 1 && natural > 1;
            let mut attributes = String::from("@layout(c)\n@derive(Copy)\n");
            if packed {
                attributes.push_str("@packed\n");
            }
            if align > if packed { 1 } else { natural } {
                let _ = writeln!(attributes, "@align({align})");
            }
            let _ = write!(self.items, "{attributes}pub struct {name}:\n{}\n", fields.concat());
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
            self.records.insert(key, Record::Full(name.clone()));
            Record::Full(name)
        }
    }

    /// An `extern type`: used only behind pointers. A definition it could not represent is
    /// `W5002`; an incomplete struct is represented exactly.
    fn opaque(&mut self, cursor: CXCursor, key: &str, named: &str, why: Option<String>) -> Record {
        if !self.claim(named) {
            return self.opaque_unclaimed(cursor, key, named);
        }
        let name = ident(named);
        self.externs.push(("C", format!("    pub type {name}\n")));
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
        let outcome = match repr {
            Err(reason) => Err(reason),
            Ok(_) if !self.claim(&named) => Err("its name is taken by another declaration of the header".to_string()),
            Ok(repr) => {
                let unsigned = repr.starts_with('u');
                let mut variants = String::new();
                let mut aliases = String::new();
                let mut values: HashMap<i128, String> = HashMap::new();
                for child in children(cursor).into_iter().filter(|child| kind(*child) == CXCursor_EnumConstantDecl) {
                    let constant = ident(&spelling(child));
                    // SAFETY(ffi): a live enum constant.
                    let value = unsafe {
                        if unsigned { clang_getEnumConstantDeclUnsignedValue(child) as i128 } else { clang_getEnumConstantDeclValue(child) as i128 }
                    };
                    match values.get(&value) {
                        Some(first) => {
                            let _ = writeln!(aliases, "pub const {constant}: {name} = {name}.{first}");
                        }
                        None => {
                            values.insert(value, constant.clone());
                            let _ = writeln!(variants, "    {constant} = {value}");
                        }
                    }
                }
                if variants.is_empty() {
                    variants.push_str("    pass\n");
                }
                let _ = write!(self.items, "@repr({repr})\n@non_exhaustive\npub enum {name}:\n{variants}\n{aliases}");
                if !aliases.is_empty() {
                    self.items.push('\n');
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
            let _ = writeln!(self.items, "pub const {}: {repr} = {value}\n", ident(&constant));
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
            Ok(Function { name: name.to_string(), params, result, link_name, abi, at: self.at(cursor) })
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
                        let _ = writeln!(self.items, "pub const {}: {ty} = {value}\n", ident(&name));
                    }
                    _ => self.skip(cursor, &name, "a global with internal linkage (`static`): each C file has its own, which no binding may name ([FFI-29a])"),
                }
                return;
            }
            match self.ty(ty, false) {
                Ok(mapped) if self.claim(&name) => {
                    let mutable = if constant { "" } else { "mut " };
                    self.externs.push(("C", format!("    pub static {mutable}{}: {mapped}\n", ident(&name))));
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
                    reason: "a function-like macro: Ember has no macros; an overlay can declare its signature ([FFI-6b])".to_string(),
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
            let ty = clang_getCanonicalType(clang_getCursorType(cursor));
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
                CXEval_Int => self.ty(ty, false).map(|mapped| {
                    let value = if clang_EvalResult_isUnsignedInt(result) != 0 {
                        clang_EvalResult_getAsUnsigned(result).to_string()
                    } else {
                        clang_EvalResult_getAsLongLong(result).to_string()
                    };
                    if mapped == "bool" { (mapped, (value != "0").to_string()) } else { (mapped, value) }
                }),
                CXEval_Float => self.ty(ty, false).and_then(|mapped| {
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

    fn finish(mut self, header: PathBuf) -> Imported {
        let mut text = format!(
            "# The declarations of `{}`, written by `import c` ([FFI-6]).\n\n",
            header.file_name().map_or(String::new(), |name| name.to_string_lossy().into_owned())
        );
        text.push_str(&self.items);
        for function in &self.functions {
            let mut line = String::from("    @safety(\"the header states no contract: each pointer's ownership, count, nullability and lifetime are the caller's to keep ([FFI-38])\")\n");
            if let Some(link) = &function.link_name {
                let _ = writeln!(line, "    @ffi(link_name=\"{link}\")");
            }
            let params: Vec<String> = function.params.iter().map(|(name, ty)| format!("{name}: {}", ty.ember)).collect();
            let result = function.result.as_ref().map_or(String::new(), |ty| format!(" -> {}", ty.ember));
            let _ = writeln!(line, "    pub fn {}({}){result}", ident(&function.name), params.join(", "));
            self.externs.push((function.abi, line));
        }
        for abi in ["C", "system", "fastcall", "vectorcall"] {
            let members: Vec<&str> = self.externs.iter().filter(|(of, _)| *of == abi).map(|(_, member)| member.as_str()).collect();
            if !members.is_empty() {
                let _ = write!(text, "unsafe extern \"{abi}\":\n{}\n", members.concat());
            }
        }
        let shim = format!(
            "/* The shim of `import c \"{}\"` ([FFI-29]): the layout checks and wrappers Ember needs. */\n#include <stddef.h>\n#include \"{}\"\n\n{}",
            header.display(),
            header.to_string_lossy().replace('\\', "/"),
            self.shim
        );
        Imported {
            text,
            header,
            skipped: self.skipped,
            opaque: self.opaque,
            macros: self.macro_skipped,
            layouts: self.layouts,
            shim,
            functions: self.functions,
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
                        .help("write `overlay = \"overlays/name.em\"`"));
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

/// A module `import c` wrote: its path, its declarations, and what else the import found.
pub struct Loaded {
    pub path: Vec<String>,
    pub module: ast::Module,
    pub foreign: ember_typeck::ForeignImport,
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
    known: &mut HashMap<String, Vec<String>>,
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
    let key = format!("{}|{header}|{:?}|{:?}|{:?}", from_dir.display(), options.include_paths, options.defines, options.overlay.as_ref().map(|o| &o.0));
    let rewrite = |import: &mut ast::Import, path: &[String]| {
        let segments = path.iter().map(|segment| ast::Ident { name: Symbol::intern(segment), span }).collect();
        import.kind = ast::ImportKind::Module { path: segments, alias: Some(ast::Ident { name: bound, span }) };
    };
    if let Some(path) = known.get(&key) {
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
    while known.values().any(|taken| *taken == path) {
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
    };
    let imported = match import_header(&config) {
        Ok(imported) => imported,
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
    let file = map.add(PathBuf::from(format!("<import c \"{header}\">")), imported.text.clone());
    let text = map.file(file).text.clone();
    let lexed = ember_lexer::lex(file, &text, sink);
    let module = ember_parser::parse(file, &text, lexed.tokens, sink);
    let mut c_flags: Vec<String> = flags.to_vec();
    c_flags.extend(include_paths.iter().map(|dir| format!("-I{}", dir.display())));
    c_flags.extend(options.defines.iter().map(|define| format!("-D{define}")));
    let foreign = ember_typeck::ForeignImport {
        header: shown,
        skipped: imported
            .skipped
            .iter()
            .map(|skipped| (skipped.name.clone(), skipped.reason.clone()))
            .chain(imported.macros.iter().map(|skipped| (skipped.name.clone(), format!("a macro, {}", skipped.reason))))
            .collect(),
        layouts: imported.layouts.iter().map(|layout| (layout.name.clone(), layout.size, layout.align, layout.fields.clone())).collect(),
        shim: imported.shim,
        sources: source.into_iter().collect(),
        flags: c_flags,
        link: options
            .link
            .iter()
            .map(|library| if ember_build::cdylib::is_library_path(library) { base.join(library).to_string_lossy().into_owned() } else { library.clone() })
            .collect(),
    };
    rewrite(import, &path);
    known.insert(key, path.clone());
    Some(Loaded { path, module, foreign })
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
