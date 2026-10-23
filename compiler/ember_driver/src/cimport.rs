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
    /// `[FFI-39]` — the classes the import's overlays declare `@ffi(trampoline, …)`.
    pub trampolines: &'a [TrampolineDecl],
    /// `[FFI-17a]` (ODR-198) — each alias of a `std::variant` the overlays name
    /// `@ffi(variant=[…])`, with its alternatives' names.
    pub variants: &'a [(String, Vec<String>)],
}

/// `[FFI-17a]` (ODR-198) — the overlays' `@ffi(variant=["A", "B"]) type Shape` entries, read before
/// the header is: each alias and the names of its alternatives, in order.
pub fn variant_decls(overlays: &str) -> Vec<(String, Vec<String>)> {
    let quoted = |text: &str| -> Vec<String> { text.split('"').skip(1).step_by(2).map(str::to_string).collect() };
    let mut found = Vec::new();
    let mut attributes: Vec<String> = Vec::new();
    for line in overlays.lines() {
        let line = line.trim();
        if line.starts_with('@') {
            attributes.push(line.to_string());
            continue;
        }
        let entry = line.trim_start_matches("override ").trim();
        if let Some(name) = entry.strip_prefix("type ").map(str::trim)
            && let Some(names) = attributes.iter().find_map(|attribute| {
                let (_, rest) = attribute.strip_prefix("@ffi(")?.split_once("variant")?;
                let (_, list) = rest.split_once('[')?;
                Some(quoted(list.split_once(']')?.0))
            })
        {
            found.push((name.to_string(), names));
        }
        if !line.is_empty() && !line.starts_with('#') {
            attributes.clear();
        }
    }
    found
}

/// `[FFI-17a]` (ODR-198) — a `std::variant` an overlay names: the thunks' stem, the Ember enum
/// (as the module that uses it names it) and the bytes that hold the C++ object.
#[derive(Clone, Debug)]
struct Variant {
    ops: String,
    ember: String,
    storage: String,
}

/// `[FFI-39]`, `[FFI-17d]` — an overlay's `@ffi(trampoline, virtuals=[…], owner=…)` entry for a
/// C++ class, read before the header is: the class (its Ember name), the virtuals Ember code
/// overrides, and whether a foreign `delete` destroys the object (`owner="foreign"`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrampolineDecl {
    pub class: String,
    pub virtuals: Vec<String>,
    pub foreign: bool,
    /// ODR-221 — the program's classes ask for it (or for some of its virtuals): a name its
    /// classes override that is not a virtual of the base is theirs to report, not the import's.
    pub from_program: bool,
    /// ODR-221 — the first class of the program deriving from it, and where its base is written:
    /// a base that cannot be derived from is reported there.
    pub requested: Option<(String, Span)>,
}

/// `[FFI-39]` — the `@ffi(trampoline, …)` entries of overlay sources, as written: an attribute line
/// naming `trampoline`, then `type Name`. The overlay is applied (and checked, `E5010`) later; this
/// only tells the importer which classes get a trampoline.
pub fn trampoline_decls(overlays: &str) -> Vec<TrampolineDecl> {
    let quoted = |text: &str| -> Vec<String> { text.split('"').skip(1).step_by(2).map(str::to_string).collect() };
    let mut found = Vec::new();
    let mut attributes: Vec<String> = Vec::new();
    for line in overlays.lines() {
        let line = line.trim();
        if line.starts_with('@') {
            attributes.push(line.to_string());
            continue;
        }
        let entry = line.trim_start_matches("override ").trim();
        if let Some(name) = entry.strip_prefix("type ").map(str::trim)
            && let Some(attribute) = attributes.iter().find(|attribute| attribute.starts_with("@ffi(") && attribute.contains("trampoline"))
        {
            let virtuals = attribute
                .split_once("virtuals")
                .and_then(|(_, rest)| rest.split_once('[').and_then(|(_, list)| list.split_once(']')).map(|(list, _)| quoted(list)))
                .unwrap_or_default();
            let foreign = attribute.split_once("owner").is_some_and(|(_, rest)| quoted(rest).first().is_some_and(|owner| owner == "foreign"));
            found.push(TrampolineDecl { class: name.to_string(), virtuals, foreign, from_program: false, requested: None });
        }
        if !line.is_empty() && !line.starts_with('#') {
            attributes.clear();
        }
    }
    found
}

/// `[FFI-39]` (ODR-221) — a module of the program, as the importer reads it for the classes that
/// derive from an imported C++ class: its folder and path, each `import cpp` it writes (the header
/// as written, and the name it binds), the names its other imports bind, its type aliases and its
/// classes.
#[derive(Clone, Debug, Default)]
pub struct ProgramModule {
    pub dir: PathBuf,
    pub path: Vec<String>,
    pub foreign: Vec<(String, String)>,
    /// `import a.b [as x]`: the name bound (`x`, or `b`) and the module's path.
    pub modules: Vec<(String, Vec<String>)>,
    /// `from a.b import X [as Y]`: the name bound, the module's path and the item; and each
    /// `from a.b import *`'s module path.
    pub items: Vec<(String, Vec<String>, String)>,
    pub globs: Vec<Vec<String>>,
    /// `type X = …` with no generic parameter: the name and the type as written.
    pub aliases: Vec<(String, Vec<String>)>,
    pub classes: Vec<ProgramClass>,
}

/// A class of the program: its name, its base as written (`cpp.Layer` is `["cpp", "Layer"]`), and
/// the methods it declares `override`.
#[derive(Clone, Debug)]
pub struct ProgramClass {
    pub name: String,
    pub base: Vec<String>,
    pub base_span: Span,
    pub overrides: Vec<String>,
}

/// `[FFI-39]` (ODR-221) — the program's modules, read for `load`: what each imports, and the
/// classes it declares.
pub fn program_module(module: &ast::Module, dir: &Path, path: &[String]) -> ProgramModule {
    let names = |path: &[ast::Ident]| path.iter().map(|segment| segment.name.to_string()).collect::<Vec<_>>();
    let mut found = ProgramModule { dir: dir.to_path_buf(), path: path.to_vec(), ..ProgramModule::default() };
    for import in &module.imports {
        match &import.kind {
            ast::ImportKind::Foreign { language: ast::ForeignLanguage::Cpp, header, alias, .. } => {
                found.foreign.push((header.clone(), alias.map_or_else(|| "cpp".to_string(), |alias| alias.name.to_string())));
            }
            ast::ImportKind::Foreign { .. } => {}
            ast::ImportKind::Module { path, alias } => {
                let bound = alias.map(|alias| alias.name.to_string()).or_else(|| path.last().map(|last| last.name.to_string()));
                found.modules.extend(bound.map(|bound| (bound, names(path))));
            }
            ast::ImportKind::Items { path, items, glob } => {
                if *glob {
                    found.globs.push(names(path));
                }
                for item in items {
                    let bound = item.alias.unwrap_or(item.name).name.to_string();
                    found.items.push((bound, names(path), item.name.name.to_string()));
                }
            }
        }
    }
    let written = |ty: &ast::TypeExpr| match &ty.kind {
        ast::TypeKind::Path { segments, .. } => segments.iter().map(|segment| segment.name.to_string()).collect(),
        _ => Vec::new(),
    };
    found.aliases = module
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            ast::ItemKind::TypeAlias(alias) if alias.generics.is_empty() => alias.value.as_ref().map(|value| (alias.name.name.to_string(), written(value))),
            _ => None,
        })
        .collect();
    let classes = module
        .items
        .iter()
        .filter_map(|item| match &item.kind {
            ast::ItemKind::Class(decl) => {
                let base = decl.base.as_ref().map(&written).unwrap_or_default();
                let overrides = decl
                    .members
                    .iter()
                    .filter_map(|member| match &member.kind {
                        ast::MemberKind::Fn(function) if function.dispatch == ast::Dispatch::Override => Some(function.name.name.to_string()),
                        _ => None,
                    })
                    .collect();
                Some(ProgramClass { name: decl.name.name.to_string(), base, base_span: decl.base.as_ref().map_or(decl.name.span, |base| base.span), overrides })
            }
            _ => None,
        })
        .collect();
    found.classes = classes;
    found
}

/// `[FFI-39]` (ODR-221) — the C++ class of a header (bound in a module by one of `bindings(m)`) a
/// class of the program derives from: directly (`class Mine(cpp.Layer)`) or through other classes
/// of the program, each named as its module names it (its own class, an item it imports, a module's
/// item, or a type alias of one).
fn header_base(program: &[ProgramModule], bindings: &dyn Fn(usize) -> Vec<String>, module: usize, written: &[String]) -> Option<String> {
    let (mut module, mut written) = (module, written.to_vec());
    let module_at = |path: &[String]| program.iter().position(|candidate| candidate.path == path);
    // Each step names a class of the program, an alias or a header's class; a chain longer than
    // every class and alias of the program is a cycle.
    let limit = program.iter().map(|module| module.classes.len() + module.aliases.len()).sum::<usize>() + 1;
    // A name the header's module is bound to here: an import of it, an item imported from a module
    // that re-exports it (`pub import cpp "…" as x`).
    let is_header = |module: usize, bound: &str| {
        bindings(module).iter().any(|binding| binding == bound)
            || program[module].items.iter().any(|(written, path, item)| written == bound && module_at(path).is_some_and(|other| bindings(other).contains(item)))
    };
    for _ in 0..limit {
        let here = &program[module];
        let (owner, name) = match written.as_slice() {
            [first, .., last] if is_header(module, first) => return Some(last.clone()),
            // `m.x.Layer`: a module's re-export of the header.
            [first, second, .., last]
                if here.modules.iter().find(|(bound, _)| bound == first).and_then(|(_, path)| module_at(path)).is_some_and(|other| is_header(other, second)) =>
            {
                return Some(last.clone());
            }
            [name] => {
                if here.classes.iter().any(|class| class.name == *name) || here.aliases.iter().any(|(alias, _)| alias == name) {
                    (module, name.clone())
                } else if let Some((_, path, item)) = here.items.iter().find(|(bound, ..)| bound == name) {
                    (module_at(path)?, item.clone())
                } else {
                    let glob = here.globs.iter().filter_map(|path| module_at(path)).find(|&other| {
                        program[other].classes.iter().any(|class| class.name == *name) || program[other].aliases.iter().any(|(alias, _)| alias == name)
                    })?;
                    (glob, name.clone())
                }
            }
            [prefix @ .., name] => {
                let path = match prefix {
                    [bound] => here.modules.iter().find(|(written, _)| written == bound).map(|(_, path)| path.clone()).unwrap_or_else(|| prefix.to_vec()),
                    _ => prefix.to_vec(),
                };
                (module_at(&path)?, name.clone())
            }
            [] => return None,
        };
        let there = &program[owner];
        written = match there.classes.iter().find(|class| class.name == name) {
            Some(class) => class.base.clone(),
            None => there.aliases.iter().find(|(alias, _)| *alias == name)?.1.clone(),
        };
        module = owner;
    }
    None
}

/// `[FFI-39]` (ODR-221) — the trampolines the program asks of the C++ header at `header` (as
/// `from_dir` and the written name find it): for each class of the header an Ember class derives
/// from, directly (`class Mine(cpp.Layer)`) or through other classes of the program, in any of its
/// modules, the virtuals those classes override, which the trampoline passes on. No overlay entry
/// is needed.
fn program_trampolines(program: &[ProgramModule], from_dir: &Path, header: &str) -> Vec<TrampolineDecl> {
    let same = |dir: &Path, written: &str| {
        written == header && dir == from_dir
            || std::fs::canonicalize(dir.join(written)).ok().is_some_and(|found| std::fs::canonicalize(from_dir.join(header)).ok() == Some(found))
    };
    let bindings = |module: usize| -> Vec<String> {
        let module = &program[module];
        module.foreign.iter().filter(|(written, _)| same(&module.dir, written)).map(|(_, bound)| bound.clone()).collect()
    };
    let mut found: Vec<TrampolineDecl> = Vec::new();
    for (at_module, module) in program.iter().enumerate() {
        for class in &module.classes {
            let Some(base) = header_base(program, &bindings, at_module, &class.base) else { continue };
            let at = match found.iter().position(|decl| decl.class == base) {
                Some(at) => at,
                None => {
                    found.push(TrampolineDecl { class: base, virtuals: Vec::new(), foreign: false, from_program: true, requested: Some((class.name.clone(), class.base_span)) });
                    found.len() - 1
                }
            };
            for name in &class.overrides {
                if !found[at].virtuals.contains(name) {
                    found[at].virtuals.push(name.clone());
                }
            }
        }
    }
    found
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
    /// `[CXX-1]` — whether a C++ import wrote it, and the configuration it parsed with (its
    /// project's defines and flags as libclang read them).
    pub cpp: bool,
    pub config: String,
}

/// A declaration not imported, with the construct that defeated it and a workaround.
#[derive(Clone, Debug, Default)]
pub struct Skipped {
    pub name: String,
    pub reason: String,
    /// `file:line` in the header.
    pub at: String,
    /// `[FFI-41]` — the C++ namespace it is in, whose module says it was skipped (`[FFI-20a]`).
    pub namespace: Vec<String>,
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
    /// `[FFI-1]` — the effects its declaration gives it: `__attribute__((const))` none, and the
    /// same result for the same arguments; `__attribute__((pure))` none (it only reads memory).
    pub derived: Derived,
    /// `[FFI-8]` — its parameters end `...`: `@ffi(variadic)`, called only in `unsafe`.
    pub variadic: bool,
}

/// `[FFI-1]` — what a C declaration says of a function's effects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Derived {
    #[default]
    Nothing,
    Pure,
    Const,
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
    /// With a `pointee`, the parameter is an opaque class's, which the wrapper takes as a reference
    /// (`ref `, `ref mut `) or as the `ForeignBox` it moves from (`owned `), and passes on as the
    /// raw pointer `ember` names ([`wrapped`]).
    pub mode: String,
    /// `[FFI-8]` — a data pointer to an incomplete type (an `extern type`): `one` makes it a
    /// reference, never a parameter mode (`[TYP-22]`).
    pub opaque: bool,
    /// `[FFI-21]` — for a function pointer, through typedefs: the function it points to as
    /// `extern "C" fn(…) -> R`, which a `callback(…)` contract's parameter is declared with.
    pub function: Option<String>,
}

/// `[FFI-39]` — a virtual a trampoline overrides: its name, `const`-ness, purity, parameters and
/// result.
struct Slot {
    name: String,
    constant: bool,
    pure: bool,
    params: Vec<SlotParam>,
    result: Option<CType>,
    /// The result's C++ type (`void` for none).
    result_spelled: String,
}

/// A parameter of a trampoline's virtual: its Ember name and type, the C type the entry function
/// receives, the C++ expression that passes it there, its declaration in the override, and how
/// the thunk of the base implementation passes it on.
struct SlotParam {
    name: String,
    ty: CType,
    c: String,
    to_ember: String,
    declaration: String,
    to_base: String,
    local: String,
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
    /// `[FFI-41]` — a C++ variable: an `extern static` reached through the thunk giving its address;
    /// a static data member's by the C name `Outer::name`.
    Variable,
    /// `[FFI-39]` — a trampoline's Ember class, with what it needs, written at the module's level.
    Class,
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
    pub conflicts: Vec<(String, String, String)>,
    /// `[FFI-30]` (D-879) — each record an earlier import of the build wrote, which this module
    /// names by an alias: its members (a C++ class's methods, constructors, operators) are that
    /// import's, written once. `namespace|Name`: the C++ namespace it is in (`::`-joined) and its
    /// Ember name.
    pub adopted: Vec<String>,
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
    /// `[FFI-39]` — each class with a trampoline.
    pub trampolines: Vec<Trampoline>,
    /// `[FFI-42]` — each member operator imported: its class's Ember name, its method's, and the
    /// Ember operator interface it implements when its call is asserted safe.
    pub operators: Vec<(String, String, String)>,
    /// `[FFI-24d]` — each function whose header added or removed `noexcept` since the binding was
    /// last made (and has not changed it back): its namespace, its Ember name (`Owner.name` for a
    /// member), and whether it is `noexcept` now.
    pub policy_changes: Vec<(String, String, bool)>,
    /// `[DIA-5]` — each declaration of the header as the header writes it, for an error about it to
    /// name: its namespace (`::`-joined), its C name, its text on one line, and `file:line`.
    pub declared: Vec<(String, String, String, String)>,
}

/// `[FFI-39]` — a C++ class an Ember class may derive from: its Ember class, the opaque type of
/// the C++ object it owns, whether the C++ class has a default constructor (`E5058` for a
/// subclass that declares no `init` otherwise), and the virtuals the trampoline does not override
/// (none a class of the program overrides, ODR-221).
#[derive(Clone, Debug, Default)]
pub struct Trampoline {
    pub class: String,
    pub opaque: String,
    pub default_init: bool,
    pub unnamed_virtuals: Vec<String>,
}

/// `[DIA-5]` — C source text on one line: each `{…}` body elided, whitespace collapsed, no trailing
/// `;`, and at most 160 characters.
fn one_line(written: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in written.chars() {
        match c {
            '{' => {
                if depth == 0 {
                    out.push_str("{…}");
                }
                depth += 1;
            }
            '}' => depth = depth.saturating_sub(1),
            _ if depth > 0 => {}
            c if c.is_whitespace() => {
                if !out.ends_with(' ') {
                    out.push(' ');
                }
            }
            c => out.push(c),
        }
    }
    let out = out.trim().trim_end_matches(';').trim_end();
    if out.chars().count() > 160 {
        format!("{}…", out.chars().take(159).collect::<String>())
    } else {
        out.to_string()
    }
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

/// `[FFI-10]` — whether a value of `ty` holds a pointer (a reference, a `char*`, a record or array
/// with one), which is not `Sync`: a C++ `const` variable of such a type is read in `unsafe`, never
/// through `@ffi(immutable)`, whose safe read needs `Sync` (`[STA-1]`). A function pointer is a
/// function value, which is `Sync`.
fn holds_pointer(ty: CXType) -> bool {
    // SAFETY(ffi): types of a live translation unit.
    unsafe {
        let ty = clang_getCanonicalType(ty);
        match ty.kind {
            CXType_Pointer => !matches!(clang_getCanonicalType(clang_getPointeeType(ty)).kind, CXType_FunctionProto | CXType_FunctionNoProto),
            CXType_LValueReference | CXType_RValueReference | CXType_MemberPointer | CXType_BlockPointer => true,
            CXType_ConstantArray | CXType_IncompleteArray => holds_pointer(clang_getArrayElementType(ty)),
            CXType_Record => record_fields(ty).into_iter().any(|field| holds_pointer(clang_getCursorType(field))),
            _ => false,
        }
    }
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

/// A C++ declaration's skip reason, `context: reason`, tagged with the code the reason carries
/// (`[FFI-48]`'s `E5034` for a type the importer refuses), else with `default` (a parameter's
/// `E5031`, `[FFI-32]`) if there is one.
fn tagged(default: Option<&str>, context: &str, reason: &str) -> String {
    match (reason.strip_prefix("[E").and_then(|rest| rest.split_once("] ")), default) {
        (Some((code, rest)), _) => format!("[E{code}] {context}: {rest}"),
        (None, Some(code)) => format!("[{code}] {context}: {reason}"),
        (None, None) => format!("{context}: {reason}"),
    }
}

/// `[FFI-42]` — the method a C++ member operator imports as, and the Ember operator interface that
/// method is (`[TYP-21]`): `operator+` with one operand is `Add.add`, `operator-` with none
/// `Neg.neg`, `operator+=` `AddAssign.add_assign`, `operator==` `Eq.eq`, a `const` `operator[]`
/// `Index.index` and another `IndexMut.index_mut`; an operator with no such interface is skipped,
/// saying why. ODR-252 — `operator*` and `operator->` of a smart-pointer-like class read through it:
/// `get()`, and `get_mut()` where the operator is not `const`, which implement no interface.
fn operator_method(name: &str, operands: c_int, member: Member) -> Result<(&'static str, Option<&'static str>), String> {
    let operator = name.trim_start_matches("operator").trim();
    let binary = [
        ("+", "add", "Add"), ("-", "sub", "Sub"), ("*", "mul", "Mul"), ("/", "div", "Div"), ("%", "rem", "Rem"),
        ("&", "bitand", "BitAnd"), ("|", "bitor", "BitOr"), ("^", "bitxor", "BitXor"), ("<<", "shl", "Shl"), (">>", "shr", "Shr"),
        ("+=", "add_assign", "AddAssign"), ("-=", "sub_assign", "SubAssign"), ("*=", "mul_assign", "MulAssign"),
        ("/=", "div_assign", "DivAssign"), ("%=", "rem_assign", "RemAssign"), ("&=", "bitand_assign", "BitAndAssign"),
        ("|=", "bitor_assign", "BitOrAssign"), ("^=", "bitxor_assign", "BitXorAssign"), ("<<=", "shl_assign", "ShlAssign"),
        (">>=", "shr_assign", "ShrAssign"), ("==", "eq", "Eq"),
    ];
    let found = match (operator, operands) {
        _ if member == Member::Static => None,
        ("[]", 1) if member == Member::Method => Some(("index", "Index")),
        ("[]", 1) => Some(("index_mut", "IndexMut")),
        ("-", 0) => Some(("neg", "Neg")),
        ("~", 0) => Some(("not", "Not")),
        (_, 1) => binary.iter().find(|(spelled, _, _)| *spelled == operator).map(|(_, method, interface)| (*method, *interface)),
        ("*" | "->", 0) if member == Member::Method => return Ok(("get", None)),
        ("*" | "->", 0) => return Ok(("get_mut", None)),
        _ => None,
    };
    found.map(|(method, interface)| (method, Some(interface))).ok_or_else(|| match operator {
        "!=" => "`operator!=`: Ember's `!=` is `Eq`'s, from `operator==` ([FFI-42])".to_string(),
        "<" | "<=" | ">" | ">=" | "<=>" => format!("`{name}`: Ember orders through `Ord.cmp`, which no one C++ operator is ([FFI-42])"),
        _ => format!("`{name}`: Ember has no operator interface for it ([FFI-42])"),
    })
}

/// `[FFI-42]` — each class the header defines (in a namespace or another class too) with public
/// `begin()` and `end()` members taking nothing, as C++ spells it.
fn iterable_classes(cursor: CXCursor, in_header: &dyn Fn(CXCursor) -> bool, out: &mut Vec<String>) {
    for child in children(cursor) {
        if !in_header(child) {
            continue;
        }
        // SAFETY(ffi): cursors and types of a live translation unit.
        unsafe {
            match kind(child) {
                CXCursor_Namespace | CXCursor_LinkageSpec => iterable_classes(child, in_header, out),
                CXCursor_StructDecl | CXCursor_ClassDecl if clang_isCursorDefinition(child) != 0 => {
                    let members = children(child);
                    let has = |name: &str| {
                        members.iter().any(|member| {
                            kind(*member) == CXCursor_CXXMethod
                                && spelling(*member) == name
                                && clang_getCXXAccessSpecifier(*member) == CX_CXXPublic
                                && clang_CXXMethod_isStatic(*member) == 0
                                && clang_Cursor_getNumArguments(*member) == 0
                        })
                    };
                    let spelled = cpp_spelling(clang_getCursorType(child));
                    if has("begin") && has("end") && !spelled.contains('(') {
                        out.push(spelled);
                    }
                    iterable_classes(child, in_header, out);
                }
                _ => {}
            }
        }
    }
}

/// `[FFI-11f]` — whether a parameter is declared `[[clang::lifetimebound]]`: an attribute of it
/// whose text, or that of the macro it expands (`#define LIFETIMEBOUND [[clang::lifetimebound]]`),
/// names `lifetimebound`.
fn lifetimebound(param: CXCursor) -> bool {
    // SAFETY(ffi): cursors of a live translation unit; the tokens are disposed of before returning.
    unsafe {
        let unit = clang_Cursor_getTranslationUnit(param);
        let tokens = |range: CXSourceRange| -> Vec<(String, CXSourceLocation)> {
            let (mut tokens, mut count) = (std::ptr::null_mut(), 0);
            clang_tokenize(unit, range, &mut tokens, &mut count);
            let out = (0..count as usize).map(|at| (text(clang_getTokenSpelling(unit, *tokens.add(at))), clang_getTokenLocation(unit, *tokens.add(at)))).collect();
            if !tokens.is_null() {
                clang_disposeTokens(unit, tokens, count);
            }
            out
        };
        let names = |range: CXSourceRange| {
            tokens(range).into_iter().any(|(spelled, at)| {
                if spelled == "lifetimebound" {
                    return true;
                }
                let expansion = clang_getCursor(unit, at);
                if kind(expansion) != CXCursor_MacroExpansion {
                    return false;
                }
                let definition = clang_getCursorReferenced(expansion);
                clang_Cursor_isNull(definition) == 0 && tokens(clang_getCursorExtent(definition)).iter().any(|(spelled, _)| spelled == "lifetimebound")
            })
        };
        // Clang shows the attribute as an unexposed one of the parameter.
        children(param).into_iter().any(|child| clang_isAttribute(kind(child)) != 0 && names(clang_getCursorExtent(child)))
    }
}

/// `[FFI-40a]` — whether a class has a `mutable` member, its own or a base's.
fn has_mutable_member(class: CXCursor) -> bool {
    children(class).into_iter().any(|child| match kind(child) {
        // SAFETY(ffi): cursors of a live translation unit.
        CXCursor_FieldDecl => (unsafe { clang_CXXField_isMutable(child) }) != 0,
        CXCursor_CXXBaseSpecifier => {
            // SAFETY(ffi): as above.
            let base = unsafe { clang_getCursorDefinition(clang_getTypeDeclaration(clang_getCanonicalType(clang_getCursorType(child)))) };
            // SAFETY(ffi): as above.
            (unsafe { clang_Cursor_isNull(base) }) == 0 && has_mutable_member(base)
        }
        _ => false,
    })
}

/// `[FFI-40a]` — whether the header declares a class with a `mutable` member.
fn mutable_class(cursor: CXCursor, in_header: &dyn Fn(CXCursor) -> bool) -> bool {
    children(cursor).into_iter().any(|child| match kind(child) {
        CXCursor_Namespace => mutable_class(child, in_header),
        CXCursor_StructDecl | CXCursor_ClassDecl if in_header(child) => {
            has_mutable_member(child) || mutable_class(child, in_header)
        }
        _ => false,
    })
}

/// The spellings of a cursor's tokens.
fn tokens_of(cursor: CXCursor) -> Vec<String> {
    // SAFETY(ffi): a cursor of a live translation unit; the tokens are disposed of before returning.
    unsafe {
        let unit = clang_Cursor_getTranslationUnit(cursor);
        let (mut tokens, mut count) = (std::ptr::null_mut(), 0);
        clang_tokenize(unit, clang_getCursorExtent(cursor), &mut tokens, &mut count);
        let out = (0..count as usize).map(|at| text(clang_getTokenSpelling(unit, *tokens.add(at)))).collect();
        if !tokens.is_null() {
            clang_disposeTokens(unit, tokens, count);
        }
        out
    }
}

/// An expression with what only wraps it (an implicit conversion, parentheses) taken away.
fn unwrapped(mut expr: CXCursor) -> CXCursor {
    while matches!(kind(expr), CXCursor_UnexposedExpr | CXCursor_ParenExpr) {
        match children(expr).as_slice() {
            [inner] => expr = *inner,
            _ => break,
        }
    }
    expr
}

/// `[FFI-40a]` — whether an expression names the object a method runs on, or a part of it: `this`,
/// `*this`, a field (`count`, `this->count`, `stats.hits`, `cells[i]`) or what a field points to.
fn names_object(expr: CXCursor) -> bool {
    let expr = unwrapped(expr);
    match kind(expr) {
        CXCursor_CXXThisExpr => true,
        // SAFETY(ffi): a cursor of a live translation unit.
        CXCursor_MemberRefExpr if kind(unsafe { clang_getCursorReferenced(expr) }) == CXCursor_FieldDecl => {
            children(expr).first().is_none_or(|base| names_object(*base))
        }
        CXCursor_ArraySubscriptExpr | CXCursor_UnaryOperator => children(expr).first().is_some_and(|base| names_object(*base)),
        _ => false,
    }
}

/// `[FFI-40a]` — whether a parameter of type `ty` lets a function change what it is given: a
/// reference or pointer to something not `const`.
fn writable_parameter(ty: CXType) -> bool {
    // SAFETY(ffi): a type of a live translation unit.
    unsafe {
        matches!(ty.kind, CXType_LValueReference | CXType_RValueReference | CXType_Pointer)
            && clang_isConstQualifiedType(clang_getPointeeType(ty)) == 0
    }
}

/// `[FFI-40a]` (ODR-160) — whether a `const` method of a class with a `mutable` member may change
/// its object, read from its body. One the header does not have may. One may when it writes a
/// field (an assignment, a compound assignment, `++` or `--` of one, its address taken, a
/// non-`const` reference or pointer bound to it), calls a non-`const` method of a field, hands the
/// object or a field to a function by non-`const` reference or pointer, or calls a method of the
/// object (or a `const` one of a field whose class has `mutable` state) that may; and a call the
/// importer cannot follow may.
fn changes_object(method: CXCursor, seen: &mut HashSet<String>) -> bool {
    // SAFETY(ffi): cursors of a live translation unit.
    let definition = unsafe { clang_getCursorDefinition(method) };
    // SAFETY(ffi): as above.
    if unsafe { clang_Cursor_isNull(definition) } != 0 {
        return true;
    }
    // A method being read already: what it writes is found there.
    if !seen.insert(usr(definition)) {
        return false;
    }
    children(definition).into_iter().filter(|child| kind(*child) == CXCursor_CompoundStmt).any(|body| writes_object(body, seen))
}

fn writes_object(cursor: CXCursor, seen: &mut HashSet<String>) -> bool {
    let parts = children(cursor);
    let found = match kind(cursor) {
        CXCursor_CompoundAssignOperator => parts.first().is_some_and(|target| names_object(*target)),
        CXCursor_BinaryOperator => match parts.as_slice() {
            [target, _] => tokens_of(cursor).get(tokens_of(*target).len()).is_some_and(|operator| operator == "=") && names_object(*target),
            _ => true,
        },
        CXCursor_UnaryOperator => {
            let spelled = tokens_of(cursor);
            let changes = |token: Option<&String>| token.is_some_and(|token| matches!(token.as_str(), "++" | "--"));
            (changes(spelled.first()) || changes(spelled.last()) || spelled.first().is_some_and(|token| token == "&"))
                && parts.first().is_some_and(|operand| names_object(*operand))
        }
        // SAFETY(ffi): a cursor of a live translation unit.
        CXCursor_VarDecl => writable_parameter(unsafe { clang_getCursorType(cursor) }) && parts.iter().any(|init| names_object(*init)),
        CXCursor_CallExpr => call_writes_object(cursor, seen),
        _ => false,
    };
    found || parts.into_iter().any(|part| writes_object(part, seen))
}

fn call_writes_object(call: CXCursor, seen: &mut HashSet<String>) -> bool {
    // SAFETY(ffi): cursors and types of a live translation unit.
    unsafe {
        let callee = clang_getCursorReferenced(call);
        let arguments: Vec<CXCursor> = (0..clang_Cursor_getNumArguments(call).max(0) as c_uint).map(|at| clang_Cursor_getArgument(call, at)).collect();
        if clang_Cursor_isNull(callee) != 0 {
            return children(call).into_iter().any(|part| names_object(part) || kind(unwrapped(part)) == CXCursor_CXXThisExpr);
        }
        let mut skip = 0;
        if kind(callee) == CXCursor_CXXMethod && clang_CXXMethod_isStatic(callee) == 0 {
            // The object it is called on: `f()` and `this->f()` the method's own, `count.f()` a field.
            let member = children(call)
                .into_iter()
                .map(unwrapped)
                .find(|part| kind(*part) == CXCursor_MemberRefExpr && clang_equalCursors(clang_getCursorReferenced(*part), callee) != 0);
            let object = match member {
                Some(member) => children(member).first().copied(),
                // An operator's object is its first argument (`cells[i]`).
                None => {
                    skip = 1;
                    arguments.first().copied()
                }
            };
            let own = object.is_none_or(|object| kind(unwrapped(object)) == CXCursor_CXXThisExpr);
            if own || object.is_some_and(names_object) {
                if clang_CXXMethod_isConst(callee) == 0 {
                    return true;
                }
                let class = clang_getCursorSemanticParent(callee);
                if (own || has_mutable_member(class)) && changes_object(callee, seen) {
                    return true;
                }
            }
        }
        let signature = clang_getCursorType(callee);
        let declared = clang_getNumArgTypes(signature).max(0) as usize;
        arguments.iter().enumerate().skip(skip).any(|(at, argument)| {
            let mut given = unwrapped(*argument);
            let address = kind(given) == CXCursor_UnaryOperator && tokens_of(given).first().is_some_and(|token| token == "&");
            if address {
                given = children(given).first().copied().unwrap_or(given);
            }
            if !names_object(given) && kind(unwrapped(given)) != CXCursor_CXXThisExpr {
                return false;
            }
            let index = at - skip;
            // Past the declared parameters, a variadic argument may be anything.
            index >= declared || writable_parameter(clang_getArgType(signature, index as c_uint))
        })
    }
}

/// `[FFI-44]` — the definition of a class's base, when it has exactly one, public and not virtual,
/// that the header (not the system's library) defines.
fn single_public_base(class: CXCursor) -> Option<CXCursor> {
    // SAFETY(ffi): cursors and types of a live translation unit.
    unsafe {
        let bases: Vec<CXCursor> = children(class).into_iter().filter(|child| kind(*child) == CXCursor_CXXBaseSpecifier).collect();
        let [base] = bases.as_slice() else { return None };
        if clang_getCXXAccessSpecifier(*base) != CX_CXXPublic || clang_isVirtualBase(*base) != 0 {
            return None;
        }
        let definition = clang_getCursorDefinition(clang_getTypeDeclaration(clang_getCanonicalType(clang_getCursorType(*base))));
        (clang_Cursor_isNull(definition) == 0 && !location(definition).2).then_some(definition)
    }
}

/// The name of the class template a canonical record type `ty` instantiates, when the template's
/// outermost namespace is `std` (inline ones, `__1`, `__cxx11`, inside it).
fn std_template(ty: CXType) -> Option<String> {
    // SAFETY(ffi): a type of a live translation unit.
    unsafe {
        if ty.kind != CXType_Record {
            return None;
        }
        let template = clang_getSpecializedCursorTemplate(clang_getTypeDeclaration(ty));
        if clang_Cursor_isNull(template) != 0 {
            return None;
        }
        let mut outermost = String::new();
        let mut parent = clang_getCursorSemanticParent(template);
        while kind(parent) == CXCursor_Namespace {
            outermost = spelling(parent);
            parent = clang_getCursorSemanticParent(parent);
        }
        (outermost == "std").then(|| spelling(template))
    }
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
/// `[FFI-8]` — whether a type is written `va_list` (through its typedefs): on Windows it is a
/// `char*`, which only the name tells apart.
fn written_va_list(mut ty: CXType) -> bool {
    // SAFETY(ffi): types of a live translation unit.
    unsafe {
        loop {
            match ty.kind {
                CXType_Typedef if standard_typedef(&text(clang_getTypedefName(ty))) == Some("VaList") => return true,
                CXType_Typedef => ty = clang_getTypedefDeclUnderlyingType(clang_getTypeDeclaration(ty)),
                CXType_Elaborated => ty = clang_Type_getNamedType(ty),
                _ => return false,
            }
        }
    }
}

fn standard_typedef(name: &str) -> Option<&'static str> {
    Some(match name {
        "size_t" | "__size_t" | "uintptr_t" => "usize",
        "ptrdiff_t" | "__ptrdiff_t" | "intptr_t" | "ssize_t" => "isize",
        // `[FFI-8]` — `<stdarg.h>`'s, however the target spells it.
        "va_list" | "__builtin_va_list" | "__gnuc_va_list" | "__va_list" => "VaList",
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
    /// `bodies`: keep the function bodies, which only `[FFI-40a]`'s reading of `const` methods
    /// needs; any other parse skips them.
    fn parse(main: &Path, contents: &str, args: &[String], bodies: bool) -> Result<Unit, String> {
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
                CXTranslationUnit_DetailedPreprocessingRecord | if bodies { 0 } else { CXTranslationUnit_SkipFunctionBodies },
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
    args.extend(clang_flags(config.flags));
    args
}

/// A project's compile flags as libclang parses with them: GCC-style ones as written; MSVC's `/D`,
/// `/U` and `/I` as their GCC forms; its runtime switch as the macros cl defines for it (`/MDd` is
/// `_MT`, `_DLL` and `_DEBUG`, which set the C++ standard library's layout, `[BLD-FFI-1b]`); and
/// each other switch that changes what a header declares or how it lays it out, as Clang's own
/// (`[CXX-1]`: the header binds as cl compiles it): `/GR-` (no RTTI data: no `_CPPRTTI`), `/EHs-c-` (no
/// exceptions: no `_CPPUNWIND`), `/J` (`char` unsigned) and `/Zp<n>` (packing). Any other MSVC
/// switch, and a response file, left out, since they change no declaration.
fn clang_flags(flags: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = flags.iter();
    while let Some(flag) = rest.next() {
        let Some(switch) = flag.strip_prefix('/') else {
            if !flag.starts_with('@') {
                out.push(flag.clone());
            }
            continue;
        };
        match switch {
            "MD" => out.extend(["-D_MT".to_string(), "-D_DLL".to_string()]),
            "MDd" => out.extend(["-D_MT".to_string(), "-D_DLL".to_string(), "-D_DEBUG".to_string()]),
            "MT" => out.push("-D_MT".to_string()),
            "MTd" => out.extend(["-D_MT".to_string(), "-D_DEBUG".to_string()]),
            "J" => out.push("-funsigned-char".to_string()),
            // As clang-cl reads it: `/Zp` alone packs to 1.
            "Zp" => out.push("-fpack-struct=1".to_string()),
            _ if switch.starts_with("Zp") && switch[2..].parse::<u32>().is_ok() => out.push(format!("-fpack-struct={}", &switch[2..])),
            // `/EHsc`, `/EHa`: C++ exceptions; `/EHs-c-`: none (the last one written wins, as for cl).
            _ if switch.starts_with("EH") => {
                let options = &switch[2..];
                let on = ['s', 'a'].iter().any(|letter| options.find(*letter).is_some_and(|at| !options[at + 1..].starts_with('-')));
                out.push(if on { "-fcxx-exceptions" } else { "-fno-cxx-exceptions" }.to_string());
            }
            "D" | "U" | "I" => out.extend(rest.next().map(|value| format!("-{switch}{value}"))),
            _ if switch.len() > 1 && matches!(&switch[..1], "D" | "U" | "I") => out.push(format!("-{switch}")),
            _ => {}
        }
    }
    // As clang-cl reads `/GR-` (the last of `/GR` and `/GR-` winning): no RTTI data, so no
    // `_CPPRTTI`, while `typeid` still compiles. Only Clang's own front end takes that switch.
    if flags.iter().rev().find(|flag| matches!(flag.as_str(), "/GR" | "/GR-")).is_some_and(|flag| flag == "/GR-") {
        out.extend(["-Xclang".to_string(), "-fno-rtti-data".to_string()]);
    }
    out
}

/// `[CXX-1]` — what of an import's configuration can change a declaration or its layout: its
/// defines and its flags as libclang parses with them (`clang_flags`), in order.
fn configuration(config: &Config) -> String {
    let mut parts: Vec<String> = config.defines.iter().map(|define| format!("-D{define}")).collect();
    parts.extend(clang_flags(config.flags));
    parts.join(" ")
}

/// `configuration` as a diagnostic shows it.
fn shown_configuration(configuration: &str) -> String {
    if configuration.is_empty() { "no defines or layout flags".to_string() } else { format!("`{configuration}`") }
}

/// `[FFI-6]` — parse the header and write its module.
/// `[FFI-6]` — parse the header and write its module, with every file the header includes.
pub fn import_header(config: &Config) -> Result<(Imported, Vec<String>), String> {
    load_libclang()?;
    // An absolute folder: Clang then names every file it finds by its absolute path.
    let from_dir = std::path::absolute(config.from_dir).map_err(|e| e.to_string())?;
    let main = from_dir.join(format!("{}_import_c.{}", ember_branding::SYMBOL_PREFIX, if config.cpp.is_some() { "cpp" } else { "c" }));
    let args = arguments(config);
    // `[FFI-48]` — a C++20 module the header declares or imports.
    if config.cpp.is_some() && let Some(line) = module_line(&from_dir.join(config.header)) {
        return Err(format!(
            "[E5034] `{line}`: a C++20 module: a compiled module interface has no stable AST to read; parse the headers instead ([FFI-48])"
        ));
    }
    let include = format!("#include \"{}\"\n", config.header.replace('\\', "/"));
    let unit = Unit::parse(&main, &include, &args, false)?;
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
    let in_header = |cursor: CXCursor| {
        let (file, _, system) = location(cursor);
        file == header || !(system || file.is_empty() || Path::new(&file) == main)
    };
    // `[FFI-40a]` (ODR-160) — a class with a `mutable` member: its `const` methods' bodies are read,
    // so the header is parsed again with them.
    let bodies = config.cpp.is_some() && mutable_class(unit.cursor(), &in_header);
    let unit = if bodies { Unit::parse(&main, &include, &args, true)? } else { unit };
    // `[FFI-42]` — each class with `begin()` and `end()`, whose element type the parse below asks.
    let mut iterables = Vec::new();
    if config.cpp.is_some() {
        iterable_classes(unit.cursor(), &in_header, &mut iterables);
    }
    // `[FFI-17b]` — each listed instantiation, declared after the header: a class template's
    // explicitly instantiated and named, a function template's address taken.
    let unit = if config.cpp.is_some() && (!config.instantiate.is_empty() || !iterables.is_empty()) {
        let prefix = ember_branding::SYMBOL_PREFIX;
        let mut source = include.clone();
        // `[FFI-42]` — `*c.begin()`'s type, or `void` where it or `begin() != end()` does not compile
        // (the parse with the header's function bodies skipped has not made the iterator's class).
        if !iterables.is_empty() {
            let _ = writeln!(
                source,
                "#include <utility>\n\
                 template <class...> struct {prefix}_void {{ using type = void; }};\n\
                 template <class C, class = void> struct {prefix}_element {{ using type = void; }};\n\
                 template <class C> struct {prefix}_element<C, typename {prefix}_void<decltype(*std::declval<C&>().begin()), decltype(std::declval<C&>().begin() != std::declval<C&>().end())>::type> {{ using type = decltype(*std::declval<C&>().begin()); }};"
            );
            for (index, spelled) in iterables.iter().enumerate() {
                let _ = writeln!(source, "using {prefix}_element_{index} = {prefix}_element<{spelled}>::type;");
            }
        }
        for (index, written) in config.instantiate.iter().enumerate() {
            let id = written.trim().trim_start_matches("::");
            let template = id.split('<').next().unwrap_or(id).trim();
            match find_template(unit.cursor(), template) {
                // `[FFI-48]` — a template with a non-type parameter of class type.
                Some(found) if let Some(parameter) = class_type_parameter(found) => {
                    return Err(format!("[E5034] `instantiate` names `{written}`: {}", class_parameter_reason(template, &parameter)));
                }
                Some(found) if kind(found) == CXCursor_ClassTemplate => {
                    let _ = writeln!(source, "template class ::{id};\nusing {prefix}_inst_{index} = ::{id};");
                    // Its methods, which Clang shows no visitor of an instantiation: each public one
                    // the template declares once, by its address in the instantiation.
                    let members = children(found);
                    let methods: Vec<String> = members
                        .iter()
                        // SAFETY(ffi): cursors of a live translation unit.
                        .filter(|child| kind(**child) == CXCursor_CXXMethod && unsafe { clang_getCXXAccessSpecifier(**child) } == CX_CXXPublic)
                        .map(|child| spelling(*child))
                        .collect();
                    // A name a member template shares is an overload set too (`parse`), which no
                    // address names.
                    let overloaded = |name: &String| {
                        methods.iter().filter(|other| *other == name).count() > 1
                            || members.iter().any(|child| kind(*child) == CXCursor_FunctionTemplate && spelling(*child) == *name)
                    };
                    for name in methods.iter().filter(|name| !name.starts_with("operator") && !overloaded(name)) {
                        let _ = writeln!(source, "constexpr auto {prefix}_inst_{index}_{name} = &::{id}::{name};");
                    }
                    // `[FFI-42]` — each operator the template declares once, by its address too.
                    for (at, name) in methods.iter().filter(|name| name.starts_with("operator") && !overloaded(name)).enumerate() {
                        let _ = writeln!(source, "constexpr auto {prefix}_inst_{index}_operator{at} = &::{id}::{name};");
                    }
                }
                Some(_) => {
                    let _ = writeln!(source, "constexpr auto {prefix}_inst_{index} = &::{id};");
                }
                None => return Err(format!("`instantiate` names `{written}`, and `{}` declares no template `{template}` ([FFI-17b])", config.header)),
            }
        }
        let instantiated = Unit::parse(&main, &source, &args, bodies)?;
        let errors: Vec<String> = instantiated.notes().into_iter().filter(|note| note.error).map(|note| note.text).collect();
        if !errors.is_empty() {
            return Err(format!("the instantiations `instantiate` lists do not compile:\n{}", errors.join("\n")));
        }
        instantiated
    } else {
        unit
    };
    let files: Vec<String> = inclusions.into_iter().filter(|(_, depth)| *depth > 0).map(|(file, _)| file).collect();
    let header_path = std::path::absolute(from_dir.join(&header)).unwrap_or_else(|_| PathBuf::from(&header));
    let mut writer = Writer {
        header: header.clone(),
        stem: config.symbol_stem.to_string(),
        shared: config.shared.clone(),
        binding: config.binding.to_string(),
        cpp: config.cpp.is_some(),
        trampoline_decls: config.trampolines.to_vec(),
        variant_decls: config.variants.to_vec(),
        ..Writer::default()
    };
    let element_prefix = format!("{}_element_", ember_branding::SYMBOL_PREFIX);
    for cursor in children(unit.cursor()) {
        if kind(cursor) == CXCursor_TypeAliasDecl
            && let Some(index) = spelling(cursor).strip_prefix(&element_prefix).and_then(|index| index.parse::<usize>().ok())
            && let Some(spelled) = iterables.get(index)
        {
            // SAFETY(ffi): a type of a live translation unit.
            writer.elements.insert(spelled.clone(), unsafe { clang_getCanonicalType(clang_getTypedefDeclUnderlyingType(cursor)) });
        }
    }
    // `[FFI-17b]` — a class instantiation's name, wherever the header names the class: any other
    // specialization of a class template is the template's, which no import names.
    let instance_prefix = format!("{}_inst_", ember_branding::SYMBOL_PREFIX);
    for cursor in children(unit.cursor()) {
        if kind(cursor) == CXCursor_TypeAliasDecl
            && let Some(index) = spelling(cursor).strip_prefix(&instance_prefix).and_then(|index| index.parse::<usize>().ok())
        {
            // SAFETY(ffi): a type of a live translation unit.
            let class = unsafe { clang_getTypeDeclaration(clang_getCanonicalType(clang_getTypedefDeclUnderlyingType(cursor))) };
            writer.instances.insert(usr(class), instance_name(&config.instantiate[index]));
        }
    }
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
    let mut methods: HashMap<usize, Vec<CXCursor>> = HashMap::new();
    for cursor in children(unit.cursor()) {
        if let Some((index, _)) = spelling(cursor).strip_prefix(&prefix).and_then(|rest| rest.split_once('_'))
            && let Ok(index) = index.parse::<usize>()
        {
            methods.entry(index).or_default().push(cursor);
        }
    }
    for cursor in children(unit.cursor()) {
        let Some(index) = spelling(cursor).strip_prefix(&prefix).and_then(|index| index.parse::<usize>().ok()) else { continue };
        let written = config.instantiate[index].trim().trim_start_matches("::").to_string();
        writer.instantiation(cursor, &written, methods.get(&index).map_or(&[], Vec::as_slice));
    }
    // `[FFI-42]` — each iterable class's iterator (`Bag_iter`), after the header's own declarations,
    // so one of them named so keeps its name and the iteration is the one skipped, saying why.
    let writing = std::mem::take(&mut writer.writing);
    for (class, owner, spelled, namespace) in std::mem::take(&mut writer.iterations) {
        writer.writing = namespace;
        writer.cpp_iteration(class, &owner, &spelled);
    }
    writer.writing = writing;
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
        let unit = Unit::parse(&main, &probe, &probe_args, false)?;
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
                    ..Skipped::default()
                }),
                Err(reason) => writer.macro_skipped.push(Skipped { name: name.clone(), reason, at: at.clone(), ..Skipped::default() }),
            }
        }
        if !addresses.is_empty() {
            let mut probe = include.clone();
            for (line, (index, _)) in addresses.iter().enumerate() {
                let _ = writeln!(probe, "static const unsigned long long {prefix}{line} = (unsigned long long)({});", macros[*index].0);
            }
            let unit = Unit::parse(&main, &probe, &probe_args, false)?;
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
                        ..Skipped::default()
                    }),
                    (None, reason) => writer.macro_skipped.push(Skipped {
                        name: name.clone(),
                        reason: match reason {
                            Some(reason) => format!("a pointer whose address Clang does not evaluate: {reason}"),
                            None => "a pointer whose address Clang does not evaluate to an integer".to_string(),
                        },
                        at: at.clone(),
                        ..Skipped::default()
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
    /// Not imported, since another declaration of the header took its name (C keeps struct tags
    /// apart; a C++ class template's specializations share one): no type is named for it, so no
    /// use of it can reach the declaration that took the name, and each says why.
    Unnamed(String),
}

#[derive(Default)]
struct Writer {
    header: String,
    stem: String,
    shared: HashMap<String, Shared>,
    binding: String,
    uses: Vec<Vec<String>>,
    entities: Vec<(String, String, String)>,
    conflicts: Vec<(String, String, String)>,
    adopted: Vec<String>,
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
    /// `[FFI-8]` — the records written without `@derive(Copy)`: one holding an `Atomic[T]`
    /// (`_Atomic`), or a record that holds one of these.
    move_only: HashSet<String>,
    /// `[FFI-17]` — the header is C++: its functions are reached through thunks.
    cpp: bool,
    /// `[FFI-32]` — each C++ class, by USR, that is standard-layout and trivially copyable; and, by
    /// `movable <USR>`, each that can be moved (`[FFI-40]`'s `T&&`).
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
    /// `[FFI-39]` — the trampolines the overlays declare, and those made.
    trampoline_decls: Vec<TrampolineDecl>,
    /// `[FFI-17a]` (ODR-198) — the overlays' variant entries, and each variant made of one, by its
    /// canonical C++ spelling.
    variant_decls: Vec<(String, Vec<String>)>,
    variants: HashMap<String, Variant>,
    trampolines: Vec<Trampoline>,
    /// `[FFI-17a]` — each bridge type the module uses (by its C++ type and the namespace it is
    /// used in), with the thunks written for it.
    bridges: HashMap<(String, Vec<String>), Bridge>,
    /// `[FFI-42]` — the member operators imported (`Imported::operators`).
    operators: Vec<(String, String, String)>,
    /// `[FFI-42]` — for each class with `begin()` and `end()`, as C++ spells it, the type of
    /// `*c.begin()` (`void` where that or `begin() != end()` does not compile).
    elements: HashMap<String, CXType>,
    /// `[FFI-42]` — each class whose iterator is written once every declaration of the header has
    /// taken its name (`cpp_iteration`): its definition, Ember name, C++ spelling and namespace.
    iterations: Vec<(CXCursor, String, String, Vec<String>)>,
    /// `[FFI-17b]` — each class instantiation `instantiate=[…]` lists, by its USR: its Ember name.
    instances: HashMap<String, String>,
    /// `[DIA-5]` — `Imported::declared`, and each header file's text, read once for it.
    declared: Vec<(String, String, String, String)>,
    sources: HashMap<String, String>,
}

/// The C++ type a thunk gives a new object's address as, when the caller comes to own the object
/// (an opaque class by value or from its constructor, a bridge value's object, a released
/// `std::unique_ptr`, an iteration's range): Ember's `usize`, which the wrapper adopts
/// (`[FFI-36a]`) or wraps (`[FFI-17a]`). An integer, so that a declaration whose contract is
/// complete may return it (`[FFI-2]`: a scalar can be an address), and the C and C++ sides declare
/// one type.
const ADDRESS: &str = "std::uintptr_t";

/// `[FFI-17a]` — a C++ standard-library type as its bridge type: the Ember type, the C++ type, and
/// how a value of it is made from the C++ object's address (`@`): `CppVector.from_thunks(@, …)`
/// with its element type's thunks, or `adopt[T](@)` for a `std::unique_ptr<T>`.
#[derive(Clone, Debug)]
struct Bridge {
    ember: String,
    spelled: String,
    make: String,
    /// `std::unique_ptr<T>`: the thunk gives up the object (`release()`) rather than copying the
    /// smart pointer to the heap; its raw result is a `*mut T`, not a `*mut u8`.
    unique: Option<String>,
}

impl Writer {
    fn at(&self, cursor: CXCursor) -> String {
        let (file, line, _) = location(cursor);
        let file = Path::new(&file).file_name().map_or(file.clone(), |name| name.to_string_lossy().into_owned());
        format!("{file}:{line}")
    }

    /// `[DIA-5]` — a declaration as the header writes it, on one line: its source text with each
    /// `{…}` body elided, its whitespace collapsed and no `;`. Its name where the text cannot be
    /// read.
    fn c_text(&mut self, cursor: CXCursor) -> String {
        // SAFETY(ffi): a live cursor's extent and the locations of its ends.
        let (file, start, end) = unsafe {
            let extent = clang_getCursorExtent(cursor);
            let (mut file, mut end_file): (CXFile, CXFile) = (std::ptr::null_mut(), std::ptr::null_mut());
            let (mut line, mut column, mut start, mut end): (c_uint, c_uint, c_uint, c_uint) = (0, 0, 0, 0);
            clang_getExpansionLocation(clang_getRangeStart(extent), &mut file, &mut line, &mut column, &mut start);
            clang_getExpansionLocation(clang_getRangeEnd(extent), &mut end_file, &mut line, &mut column, &mut end);
            let (name, end_name) = if file.is_null() || end_file.is_null() {
                (String::new(), None)
            } else {
                (text(clang_getFileName(file)), Some(text(clang_getFileName(end_file))))
            };
            (if end_name.as_deref() == Some(name.as_str()) { name } else { String::new() }, start as usize, end as usize)
        };
        let written = if file.is_empty() {
            None
        } else {
            let source = self.sources.entry(file.clone()).or_insert_with(|| std::fs::read_to_string(&file).unwrap_or_default());
            source.get(start..end).map(one_line)
        };
        written.filter(|written| !written.is_empty()).unwrap_or_else(|| spelling(cursor))
    }

    fn skip(&mut self, cursor: CXCursor, name: &str, reason: impl Into<String>) {
        let at = self.at(cursor);
        let namespace = if self.cpp { namespace_of(cursor) } else { Vec::new() };
        self.skipped.push(Skipped { name: name.to_string(), reason: reason.into(), at, namespace });
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
        // `[DIA-5]` — what an error about the declaration names.
        if matches!(
            kind(cursor),
            CXCursor_FunctionDecl
                | CXCursor_StructDecl
                | CXCursor_UnionDecl
                | CXCursor_ClassDecl
                | CXCursor_EnumDecl
                | CXCursor_TypedefDecl
                | CXCursor_TypeAliasDecl
                | CXCursor_VarDecl
        ) {
            let name = spelling(cursor);
            if !name.is_empty() {
                let (text, at) = (self.c_text(cursor), self.at(cursor));
                self.declared.push((self.writing.join("::"), name, text, at));
            }
        }
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
                            self.anonymous_namespace.push(Skipped {
                                name,
                                reason: "it is in an anonymous namespace, so each C++ file has its own".to_string(),
                                at,
                                namespace: self.writing.clone(),
                            });
                        }
                    }
                    return;
                }
                let mut inner = self.writing.clone();
                if !inline_namespace(cursor) {
                    inner.push(spelling(cursor));
                }
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
                CXType_LongDouble => Ok("c_longdouble".to_string()),
                CXType_Int128 | CXType_UInt128 => {
                    Err("a 128-bit integer (`__int128`: its argument passing differs between C compilers)".to_string())
                }
                CXType_Half | CXType_Float16 => Err("`_Float16` (Ember's `f16` crosses as a 16-bit integer)".to_string()),
                CXType_Elaborated => self.ty(clang_Type_getNamedType(ty), behind_pointer),
                // `[FFI-8]` — `_Atomic T` is `std.sync.Atomic[T]` when C lays it out as `T` (an
                // integer, a `bool` or a pointer, `[THR-14]`); otherwise `E5016`.
                CXType_Atomic => {
                    if !clang_Type_getValueType::is_loaded() {
                        return Err("`_Atomic` (this libclang is older than 11, which reads its value type)".to_string());
                    }
                    let value = clang_Type_getValueType(ty);
                    let canonical = clang_getCanonicalType(value).kind;
                    let atomic_kind = matches!(canonical, CXType_Bool | CXType_Char_S | CXType_SChar | CXType_Char_U | CXType_UChar
                        | CXType_Short | CXType_UShort | CXType_Int | CXType_UInt | CXType_Long | CXType_ULong
                        | CXType_LongLong | CXType_ULongLong | CXType_Pointer);
                    if !atomic_kind {
                        return Err(format!("[E5016] `{}`: `Atomic[T]` holds an integer, a `bool` or a pointer ([THR-14])", type_spelling(ty)));
                    }
                    if clang_Type_getSizeOf(ty) != clang_Type_getSizeOf(value) || clang_Type_getAlignOf(ty) != clang_Type_getAlignOf(value) {
                        return Err(format!(
                            "[E5016] `{}` is laid out as {} bytes aligned to {}, and `Atomic[T]` as `T`: {} bytes aligned to {}",
                            type_spelling(ty), clang_Type_getSizeOf(ty), clang_Type_getAlignOf(ty), clang_Type_getSizeOf(value), clang_Type_getAlignOf(value)
                        ));
                    }
                    Ok(format!("Atomic[{}]", self.ty(value, behind_pointer)?))
                }
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
                        return Err(match self.bridge(ty) {
                            Some(Ok(bridge)) => format!("`{}` by value or behind a pointer: it crosses as `{}`, by reference or as a result ([FFI-17a])", cpp_spelling(ty), bridge.ember),
                            Some(Err(reason)) => reason,
                            None if matches!(std_template(clang_getCanonicalType(ty)).as_deref(), Some("basic_string_view" | "span" | "optional")) => {
                                format!("`{}` here: a view or an optional value crosses as a function's parameter or result ([FFI-17a])", cpp_spelling(ty))
                            }
                            None => format!(
                                "`{}`: a C++ standard library type with no bridge type ([FFI-17a] bridges `std::vector`, `std::string`, `std::string_view`, `std::span`, `std::optional`, `std::shared_ptr`, `std::weak_ptr` and `std::unique_ptr`)",
                                cpp_spelling(ty)
                            ),
                        });
                    }
                    // A struct with no tag is known by its typedef's name once that is written
                    // (`typedef struct {…} Rect`, seen through a canonical type).
                    if clang_Cursor_isAnonymous(declaration) != 0 && !self.records.contains_key(&usr(declaration)) {
                        return Err("an anonymous struct or union member (its generated `<Parent>_anon<N>` type is not built yet)".to_string());
                    }
                    match self.record(declaration, None) {
                        Record::Full(name) | Record::Pending(name) => Ok(self.qualify(name, declaration)),
                        Record::Unnamed(why) => Err(why),
                        Record::Opaque(name, _) if behind_pointer => Ok(self.qualify(name, declaration)),
                        // `[FFI-32]` — a C++ class held behind a pointer by design, not for want of
                        // a definition.
                        Record::Opaque(name, None) if self.cpp && clang_Cursor_isNull(clang_getCursorDefinition(declaration)) == 0 => Err(format!(
                            "`{name}` by value: a C++ class that is not standard-layout and trivially copyable is held only behind a pointer ([FFI-32]); a C++ function taking a `const {}&` takes it",
                            cpp_spelling(ty)
                        )),
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
                    // `[FFI-8]` — `volatile T*` is `Volatile[*mut T]`: its reads and writes are
                    // volatile (`[UNS-5]`).
                    if clang_isVolatileQualifiedType(pointee) != 0 {
                        return Ok(format!("Volatile[*{}{inner}]", if constant { "" } else { "mut " }));
                    }
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
            // Each parameter and the result as the prototype writes them, typedef names kept: a
            // standard one (`intptr_t`, `size_t`) maps as `[FFI-8]` maps it anywhere else.
            let mut params = Vec::new();
            for index in 0..clang_getNumArgTypes(canonical).max(0) as u32 {
                let written = clang_getArgType(ty, index);
                // `[FFI-8]` — a `va_list`, which only its written name tells from what it is.
                if written_va_list(written) {
                    params.push("owned VaList".to_string());
                    continue;
                }
                params.push(self.parameter_type(written)?);
            }
            let result = clang_getResultType(ty);
            let arrow = if result.kind == CXType_Void { String::new() } else { format!(" -> {}", self.ty(result, false)?) };
            Ok(Some(format!("extern \"{abi}\" fn({}){arrow}", params.join(", "))))
        }
    }

    /// A parameter's type: an array decays to a pointer to its element, as in C.
    fn parameter_type(&mut self, ty: CXType) -> Result<String, String> {
        // `[FFI-8]` — a `va_list`, which is an array where System V's x86-64 makes it one.
        if written_va_list(ty) {
            return Ok("VaList".to_string());
        }
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
                // A `volatile T*` is a `Volatile[*mut T]`, which no contract word makes a reference.
                CXType_Pointer if clang_isVolatileQualifiedType(clang_getPointeeType(written)) != 0 => None,
                CXType_Pointer if clang_getCanonicalType(clang_getPointeeType(written)).kind != CXType_FunctionProto => {
                    Some(clang_getPointeeType(written))
                }
                CXType_ConstantArray | CXType_IncompleteArray if parameter => Some(clang_getArrayElementType(written)),
                _ => None,
            }
        };
        let c = type_spelling(ty);
        // `[FFI-21]` — a function pointer's function, for a callback's contract.
        // SAFETY(ffi): types of a live translation unit.
        // Through typedefs to the pointer, so the function's parameters keep their written names
        // (`va_list`, `[FFI-8]`).
        let function = unsafe {
            let mut written = ty;
            while matches!(written.kind, CXType_Typedef | CXType_Elaborated) {
                written = if written.kind == CXType_Typedef {
                    clang_getTypedefDeclUnderlyingType(clang_getTypeDeclaration(written))
                } else {
                    clang_Type_getNamedType(written)
                };
            }
            let pointer = if written.kind == CXType_Pointer { written } else { clang_getCanonicalType(ty) };
            if pointer.kind == CXType_Pointer { self.function_type(clang_getPointeeType(pointer)).ok().flatten() } else { None }
        };
        // `[FFI-8]` — whoever is passed a `va_list` reads it: it is passed on once, moved. (On
        // Windows it is a `char*`, which no contract word reads through.)
        let va_list = ember == "VaList";
        let mode = if va_list { "owned ".to_string() } else { String::new() };
        let Some(pointee) = pointee.filter(|_| !va_list) else { return Ok(CType { ember, pointee: None, char_pointer: false, c, mode, opaque: false, function }) };
        // SAFETY(ffi): as above.
        let (constant, character) = unsafe {
            let kind = clang_getCanonicalType(pointee).kind;
            (clang_isConstQualifiedType(pointee) != 0, matches!(kind, CXType_Char_S | CXType_Char_U))
        };
        let inner = self.ty(pointee, true)?;
        // SAFETY(ffi): as above.
        let opaque = unsafe {
            let canonical = clang_getCanonicalType(pointee);
            canonical.kind == CXType_Record && matches!(self.records.get(&usr(clang_getTypeDeclaration(canonical))), Some(Record::Opaque(..)))
        };
        Ok(CType { ember, pointee: Some((inner, constant)), char_pointer: character, c, mode: String::new(), opaque, function: None })
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
        let name = if self.cpp { nested_name(cursor) } else { spelling(cursor) };
        if let Some(fixed) = standard_typedef(&name) {
            return Ok(fixed.to_string());
        }
        // `[FFI-17a]` (ODR-198) — an alias of a `std::variant` an overlay names: its enum, which
        // crosses only as a function's parameter or result (its C++ object is not its Ember value).
        if self.cpp && let Some((_, names)) = self.variant_decls.iter().find(|(alias, _)| *alias == name || ident(&name) == *alias).cloned() {
            let outcome = match self.variant(cursor, &name, &names) {
                Ok(()) => Err(format!("`{name}`: a `std::variant` crosses as a function's parameter or result, not here ([FFI-17a])")),
                Err(reason) => {
                    self.skip(cursor, &name, reason.clone());
                    Err(format!("`{name}`, not imported: {reason}"))
                }
            };
            self.typedefs.insert(key, outcome.clone());
            return outcome;
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

    /// `[FFI-17a]` (ODR-198) — the enum of a `std::variant` alias the overlay names, `names` its
    /// alternatives': each alternative maps by value (so the variant is trivially copyable, and its
    /// bytes move as any value's). Written: the enum; the storage, the variant's bytes; the thunks
    /// that read its index and an alternative and make one; and the module's helpers that turn the
    /// storage into the enum (`<ops>_take`) and back (`<ops>_store`).
    fn variant(&mut self, cursor: CXCursor, name: &str, names: &[String]) -> Result<(), String> {
        // SAFETY(ffi): cursors and types of a live translation unit.
        unsafe {
            let canonical = clang_getCanonicalType(clang_getTypedefDeclUnderlyingType(cursor));
            if std_template(canonical).as_deref() != Some("variant") {
                return Err(format!("`@ffi(variant=[…])` names `{name}`, which is no alias of a `std::variant` ([FFI-17a])"));
            }
            let count = clang_Type_getNumTemplateArguments(canonical).max(0) as usize;
            if names.len() != count {
                return Err(format!("`{name}` has {count} alternatives, and `@ffi(variant=[…])` names {} ([FFI-17a])", names.len()));
            }
            let spelled = cpp_spelling(canonical);
            let mut alternatives = Vec::new();
            for (index, variant_name) in names.iter().enumerate() {
                let alternative = clang_Type_getTemplateArgumentAsType(canonical, index as c_uint);
                let mapped = self.ty(alternative, false).map_err(|reason| format!("its alternative `{}` does not map by value ({reason}) [FFI-17a]", cpp_spelling(alternative)))?;
                alternatives.push((ident(variant_name), mapped, cpp_spelling(alternative)));
            }
            if !self.claim(name) {
                return Err("its name is taken by another declaration of the header".to_string());
            }
            let enum_name = ident(name);
            let ops = format!("{}_cpp_{}_variant_{}", ember_branding::SYMBOL_PREFIX, self.stem, self.variants.len());
            let storage = format!("{ops}_storage");
            let (size, align) = (clang_Type_getSizeOf(canonical).max(1), clang_Type_getAlignOf(canonical).max(1));
            let copy = alternatives.iter().all(|(_, mapped, _)| !mapped.split(|c: char| !(c.is_alphanumeric() || c == '_')).any(|word| word == "Atomic" || self.move_only.contains(word)));
            let mut text = format!(
                "## `[FFI-17a]` (ODR-198) — `{spelled}`, as the header's `{name}` names it: one value of an alternative,\n## crossing to and from C++ as a parameter or a result.\n{}enum {enum_name}:\n",
                if copy { "@derive(Copy)\n" } else { "" }
            );
            for (variant_name, mapped, _) in &alternatives {
                let _ = writeln!(text, "    {variant_name}({mapped})");
            }
            self.push(name, Kind::Type, text);
            self.spelled(cursor, name);
            // The C++ object's bytes, and the thunks that read and write it.
            let mut shim = format!(
                "static_assert(std::is_trivially_copyable_v<{spelled}> && sizeof({spelled}) == {size} && alignof({spelled}) == {align}, \"{name}: a variant Ember holds as bytes\");\n\
                 extern \"C\" std::size_t {ops}_index(const void* v) noexcept {{ return static_cast<const {spelled}*>(v)->index(); }}\n"
            );
            let mut decls = format!("    @ffi(link_name=\"{ops}_index\")\n    fn {ops}_index(object: *u8) -> usize\n");
            let (mut take, mut store, mut slots) = (String::new(), String::new(), String::new());
            for (index, (variant_name, mapped, alternative)) in alternatives.iter().enumerate() {
                let _ = writeln!(
                    shim,
                    "extern \"C\" void {ops}_get_{index}(const void* v, {alternative}* out) noexcept {{ ::new (static_cast<void*>(out)) {alternative}(std::get<{index}>(*static_cast<const {spelled}*>(v))); }}\n\
                     extern \"C\" void {ops}_make_{index}(void* at, const {alternative}* value) noexcept {{ ::new (at) {spelled}(std::in_place_index<{index}>, *value); }}"
                );
                let _ = write!(
                    decls,
                    "    @ffi(link_name=\"{ops}_get_{index}\")\n    fn {ops}_get_{index}(object: *u8, value: *mut {mapped})\n    @ffi(link_name=\"{ops}_make_{index}\")\n    fn {ops}_make_{index}(object: *mut u8, value: *{mapped})\n"
                );
                // Each alternative's slot, before the block that reads one (`[UNS-3]`).
                let _ = writeln!(slots, "    slot{index}: MaybeUninit[{mapped}] = MaybeUninit[{mapped}].uninit()");
                let made = format!(
                    "{ops}_get_{index}(at, ((ref mut slot{index}) as *mut MaybeUninit[{mapped}]) as *mut {mapped})\n{{indent}}return {enum_name}.{variant_name}(slot{index}.assume_init())\n"
                );
                if index + 1 < alternatives.len() {
                    let _ = write!(take, "        if index == {index}:\n            {}", made.replace("{indent}", "            "));
                } else {
                    let _ = write!(take, "        {}", made.replace("{indent}", "        "));
                }
                let _ = write!(store, "            {enum_name}.{variant_name}(ref payload):\n                {ops}_make_{index}(at, (ref payload) as *{mapped})\n");
            }
            let _ = write!(self.shim, "{shim}");
            self.push(
                &storage,
                Kind::Class,
                format!(
                    "## `[FFI-17a]` — the bytes of a `{spelled}` ({size}, aligned to {align}), which the thunks read and write.\n@layout(c)\n@derive(Copy)\n@align({align})\nstruct {storage}:\n    bytes: [u8; {size}]\n\n\
                     ## `[FFI-17a]` — the thunks of `{spelled}`.\nunsafe extern \"C\":\n{decls}\n\
                     ## `[FFI-17a]` — the `{enum_name}` the bytes of a `{spelled}` hold, its active alternative copied out.\nfn {ops}_take(storage: {storage}) -> {enum_name}:\n{slots}    unsafe:  # SAFETY(ffi): `storage` holds a `{spelled}` a thunk made, whose active alternative is read\n        at = ((ref storage) as *{storage}) as *u8\n        index = {ops}_index(at)\n{take}\n\
                     ## `[FFI-17a]` — the bytes of the `{spelled}` holding `value`'s alternative.\nfn {ops}_store(value: {enum_name}) -> {storage}:\n    storage = {storage}([0; {size}])\n    unsafe:  # SAFETY(ffi): `storage` has the variant's size and alignment, and the payload is read for the call\n        at = ((ref mut storage) as *mut {storage}) as *mut u8\n        match value:\n{store}    return storage\n"
                ),
            );
            let key = variant_key(canonical);
            let ember = self.qualify(enum_name, cursor);
            let storage = self.qualify(storage, cursor);
            self.variants.insert(key, Variant { ops, ember, storage });
            Ok(())
        }
    }

    /// `[FFI-17a]` (ODR-198) — the variant an overlay names that `ty` (or what it refers to) is.
    fn variant_of(&self, ty: CXType) -> Option<Variant> {
        self.variants.get(&variant_key(ty)).cloned()
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
                // C's `typedef struct Pair Pair`: one name, one scope. A C++ alias of the same name
                // in another scope (`using value_t = detail::value_t;` in a class) is an alias.
                let same = spelling(declaration) == name && clang_equalCursors(clang_getCursorSemanticParent(cursor), clang_getCursorSemanticParent(declaration)) != 0;
                if named.kind == CXType_Record {
                    if anonymous || same {
                        self.record(declaration, Some(name));
                        return Ok(None);
                    }
                    let target = match self.record(declaration, None) {
                        Record::Full(target) | Record::Pending(target) | Record::Opaque(target, _) => target,
                        Record::Unnamed(why) => return Err(why),
                    };
                    return Ok(Some(self.qualify(target, declaration)));
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
            let tag = if self.cpp { nested_name(cursor) } else { spelling(cursor) };
            let mut named = typedef_name.map_or(tag.clone(), str::to_string);
            // `[FFI-17b]` — a specialization of a class template is imported only as an instantiation
            // `instantiate=[…]` lists, under its name; any other, written in the header or not, is
            // the template's (`E5055`), never the template's name for another specialization.
            if self.cpp && clang_Cursor_isNull(clang_getSpecializedCursorTemplate(cursor)) == 0 {
                match self.instances.get(&key) {
                    Some(instance) => named = instance.clone(),
                    None if let Some(parameter) = class_type_parameter(clang_getSpecializedCursorTemplate(cursor)) => {
                        let why = format!("[E5034] `{}`: {}", type_spelling(clang_getCursorType(cursor)), class_parameter_reason(&spelling(clang_getSpecializedCursorTemplate(cursor)), &parameter));
                        return self.opaque(cursor, &key, &named, Some(why));
                    }
                    None => {
                        let why = format!(
                            "[E5055] `{}`: a specialization of a C++ template that `instantiate=[…]` does not list, and no Ember code, generic or not, instantiates one ([FFI-17b])",
                            cpp_spelling(clang_getCursorType(cursor))
                        );
                        self.skip(cursor, &tag, why.clone());
                        let record = Record::Unnamed(why);
                        self.records.insert(key, record.clone());
                        return record;
                    }
                }
            }
            let name = ident(&named);
            let definition = clang_getCursorDefinition(cursor);
            if clang_Cursor_isNull(definition) != 0 {
                return self.opaque(cursor, &key, &named, None);
            }
            // `[FFI-8]` — a C union is a `@layout(c, union)` struct: its fields share its storage.
            let union = kind(definition) == CXCursor_UnionDecl;
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
            // `[FFI-39]` — the C++ object of a class with a trampoline: an opaque type of its own
            // name, which the Ember class of the class's name owns.
            if self.cpp && self.trampoline_decls.iter().any(|decl| decl.class == named) {
                return self.opaque(cursor, &key, &format!("{}_cpp_{named}", ember_branding::SYMBOL_PREFIX), None);
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
            // `[FFI-8]` — a field that is never copied (`Atomic[T]`, `[THR-14]`) makes the record so.
            let mut move_only = false;
            // `[FFI-8]` — a flexible array member: its name, its element type and its offset.
            let mut flexible: Option<(String, String, u64)> = None;
            // A class template's specialization shows its fields only through its type.
            let specialized = self.cpp && clang_Cursor_isNull(clang_getSpecializedCursorTemplate(definition)) == 0;
            let members = if specialized { record_fields(ty) } else { children(definition) };
            // `[FFI-32]` — a standard-layout class's fields share one access: private or protected
            // ones are the module's own, as C++ keeps them its class's.
            let hidden = self.cpp
                && members.iter().any(|child| kind(*child) == CXCursor_FieldDecl && clang_getCXXAccessSpecifier(*child) != CX_CXXPublic);
            let visibility = if hidden { "" } else { "pub " };
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
                            let canonical_ty = clang_getCanonicalType(field_ty);
                            // `[FFI-8]` — a C enum's bit-field reads and writes the enum, through
                            // its integer type.
                            let enumeration = (canonical_ty.kind == CXType_Enum).then(|| clang_getTypeDeclaration(canonical_ty));
                            let integer_ty = enumeration.map_or(canonical_ty, |declaration| clang_getCanonicalType(clang_getEnumDeclIntegerType(declaration)));
                            let canonical = integer_ty.kind;
                            let boolean = canonical == CXType_Bool;
                            let integer = matches!(canonical, CXType_Char_S | CXType_SChar | CXType_Short | CXType_Int | CXType_Long | CXType_LongLong
                                | CXType_Char_U | CXType_UChar | CXType_UShort | CXType_UInt | CXType_ULong | CXType_ULongLong);
                            let first_variant = enumeration.and_then(|declaration| children(declaration).into_iter().find(|child| kind(*child) == CXCursor_EnumConstantDecl).map(|child| ident(&spelling(child))));
                            let mapped = match self.ty(field_ty, false) {
                                Ok(mapped) if boolean || (integer && (enumeration.is_none() || first_variant.is_some())) => mapped,
                                _ => return self.opaque(cursor, &key, &named, Some(format!("bit-field `{field}` of type `{}` (only integer, `bool` and C enum bit-fields have accessors)", type_spelling(field_ty)))),
                            };
                            let enumerated = match (enumeration, first_variant) {
                                (Some(_), Some(first)) => match fixed_integer(integer_ty) {
                                    Ok(repr) => Some((repr, first)),
                                    Err(reason) => return self.opaque(cursor, &key, &named, Some(format!("bit-field `{field}`: {reason}"))),
                                },
                                _ => None,
                            };
                            let signed = matches!(canonical, CXType_Char_S | CXType_SChar | CXType_Short | CXType_Int | CXType_Long | CXType_LongLong);
                            let offset = clang_Cursor_getOffsetOfField(child).max(0) as u64;
                            bits.push(BitField { name: ident(&field), offset, width, ty: mapped, signed, boolean, enumerated });
                            continue;
                        }
                        let field_ty = clang_getCursorType(child);
                        // `[FFI-8]` — a flexible array member ends the struct: its elements follow
                        // it, reached by `tail(n)`; the struct is unsized.
                        if kind(child) == CXCursor_FieldDecl && clang_getCanonicalType(field_ty).kind == CXType_IncompleteArray {
                            let element = clang_getArrayElementType(clang_getCanonicalType(field_ty));
                            match self.ty(element, false) {
                                Ok(mapped) => flexible = Some((ident(&field), mapped, (clang_Cursor_getOffsetOfField(child).max(0) as u64) / 8)),
                                Err(reason) => return self.opaque(cursor, &key, &named, Some(format!("its flexible array member `{field}`: {reason}"))),
                            }
                            continue;
                        }
                        if flexible.is_some() {
                            return self.opaque(cursor, &key, &named, Some("a member after a flexible array member".to_string()));
                        }
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
                            let generated = format!("{named}_anon{anonymous}");
                            self.anonymous.insert(generated.clone());
                            match self.record(member, Some(&generated)) {
                                Record::Full(_) => {}
                                Record::Pending(_) | Record::Opaque(..) | Record::Unnamed(_) => {
                                    return self.opaque(cursor, &key, &named, Some(format!("its anonymous member `{generated}` cannot be represented")));
                                }
                            }
                            anonymous += 1;
                            (format!("_anon{}", anonymous - 1), ident(&generated))
                        } else {
                            match self.ty(field_ty, false) {
                                Ok(mapped) => (field, mapped),
                                Err(reason) => return self.opaque(cursor, &key, &named, Some(tagged(None, &format!("field `{field}`"), &reason))),
                            }
                        };
                        let field_align = clang_Type_getAlignOf(field_ty).max(1) as u64;
                        natural = natural.max(field_align);
                        move_only |= mapped.split(|c: char| !(c.is_alphanumeric() || c == '_')).any(|word| word == "Atomic" || self.move_only.contains(word));
                        let line = format!("    {visibility}{}: {mapped}\n", ident(&field));
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
            if union && !bits.is_empty() {
                return self.opaque(cursor, &key, &named, Some("a union with bit-fields".to_string()));
            }
            if !bits.is_empty() {
                match bit_field_members(&name, &plain, &bits, size, align, packed, visibility) {
                    Ok((lines, members, methods)) => (fields, layout, accessors) = (lines, members, methods),
                    Err(why) => return self.opaque(cursor, &key, &named, Some(why)),
                }
            }
            // `[FFI-8]` — the elements a flexible array member holds, as a view the caller vouches for.
            if let Some((field, element, offset)) = &flexible {
                if union {
                    return self.opaque(cursor, &key, &named, Some("a union with a flexible array member".to_string()));
                }
                if accessors.is_empty() {
                    accessors = format!("\nextend {name}:\n");
                }
                let _ = write!(
                    accessors,
                    "    ## `[FFI-8]` — the first `n` elements of the flexible array member `{field}`, which follow the struct.\n    @safety(\"the allocation holding this struct holds at least `n` elements of `{field}` after it\")\n    {visibility}unsafe fn tail(self, n: usize) -> Span[{element}]:  # SAFETY: the caller keeps the obligation\n        address = ref_to_ptr(ref self) as usize + {offset}\n        return {}_cpp_span(ref self, address as *{element}, n)\n",
                    ember_branding::SYMBOL_PREFIX
                );
            }
            if fields.is_empty() {
                // A C++ class with no data is an opaque one: its members are what it has.
                let why = (!self.cpp).then(|| "a struct with no fields (a GNU C extension)".to_string());
                return self.opaque(cursor, &key, &named, why);
            }
            let fingerprint = format!("{} {size} {align} {:?}", if union { "union" } else { "struct" }, layout.iter().map(|(_, offset)| offset).collect::<Vec<_>>());
            if let Some(adopted) = self.adopt(cursor, &key, &named, &fingerprint, false) {
                return adopted;
            }
            if !self.claim(&named) {
                return self.opaque_unclaimed(cursor, &key, &named);
            }
            packed |= align == 1 && natural > 1;
            if move_only {
                self.move_only.insert(name.clone());
            }
            let mut attributes = String::from(match (union, flexible.is_some(), move_only) {
                (true, _, false) => "@layout(c, union)\n@derive(Copy)\n",
                (true, _, true) => "@layout(c, union)\n",
                // Unsized: never copied, never by value.
                (false, true, _) => "@layout(c, flexible)\n",
                (false, false, false) => "@layout(c)\n@derive(Copy)\n",
                (false, false, true) => "@layout(c)\n",
            });
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
                // A private field's offset no code outside its class may name: its size and
                // alignment above check it, as a standard-layout class lays out its fields as C does.
                for (field, offset) in c_fields.iter().filter(|(field, _)| !hidden && !field.starts_with("_anon")) {
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
    /// `[CXX-1]` — a C++ class two C++ imports lay out two ways is `E9020` whatever binds it: their
    /// configurations disagree, and one C++ program would hold the class in both layouts.
    fn adopt(&mut self, cursor: CXCursor, key: &str, named: &str, layout: &str, any: bool) -> Option<Record> {
        let shared = self.shared.get(key)?.clone();
        if !any && shared.layout != layout {
            if (self.cpp && shared.cpp) || (shared.binding == self.binding && shared.layout != "opaque") {
                self.conflicts.push((named.to_string(), shared.header.clone(), shared.config.clone()));
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
        // By the namespace it is written in and its Ember name, as a member's owner is named.
        self.adopted.push(format!("{}|{}", self.writing.join("::"), ident(named.rsplit("::").next().unwrap_or(named))));
        self.push(named, Kind::Type, format!("type {name} = {alias}"));
        self.spelled(cursor, named);
        let record = if shared.layout.starts_with("opaque") { Record::Opaque(name, None) } else { Record::Full(name) };
        self.records.insert(key.to_string(), record.clone());
        Some(record)
    }

    /// An `extern type`: used only behind pointers. A definition it could not represent is
    /// `W5002`; an incomplete struct is represented exactly.
    fn opaque(&mut self, cursor: CXCursor, key: &str, named: &str, why: Option<String>) -> Record {
        // An incomplete struct is the entity another import wrote, whatever its layout there. A
        // complete C++ class Ember holds behind a pointer has one size and alignment in a target
        // (`[CXX-1]`): those are its fingerprint.
        // SAFETY(ffi): a cursor of a live translation unit.
        let (size, align) = unsafe {
            let ty = clang_getCursorType(cursor);
            (clang_Type_getSizeOf(ty), clang_Type_getAlignOf(ty))
        };
        let complete = self.cpp && size > 0;
        let fingerprint = if complete { format!("opaque {size} {align}") } else { "opaque".to_string() };
        if why.is_none()
            && let Some(adopted) = self.adopt(cursor, key, named, &fingerprint, !complete)
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
            self.entities.push((key.to_string(), name.clone(), fingerprint));
        }
        if let Some(why) = &why {
            let at = self.at(cursor);
            self.opaque.push(Skipped { name: named.to_string(), reason: why.clone(), at, namespace: self.writing.clone() });
        }
        let record = Record::Opaque(name, why);
        self.records.insert(key.to_string(), record.clone());
        record
    }

    fn opaque_unclaimed(&mut self, cursor: CXCursor, key: &str, named: &str) -> Record {
        let why = "its name is taken by another declaration of the header (C keeps struct tags apart; Ember has one namespace)";
        self.skip(cursor, named, why);
        let record = Record::Unnamed(format!("`{named}`: {why}"));
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
        let named = typedef_name.map_or_else(|| if self.cpp { nested_name(cursor) } else { spelling(cursor) }, str::to_string);
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
        if self.cpp && name.starts_with("operator") {
            self.free_operator(cursor, &name);
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
            // `[FFI-8]` — `...`: called in `unsafe`, each further argument as the program types it.
            let variadic = clang_isFunctionTypeVariadic(ty) != 0;
            if variadic && clang_Cursor_getNumArguments(cursor) < 1 {
                return Err("variadic with no declared parameter (C23's `f(...)`): C has no portable call of it".to_string());
            }
            let abi = convention(clang_getFunctionTypeCallingConv(ty))
                .ok_or("its calling convention is not one Ember has (`[FFI-9]`)")?;
            let mut params = Vec::new();
            for index in 0..clang_Cursor_getNumArguments(cursor).max(0) as u32 {
                let argument = clang_Cursor_getArgument(cursor, index);
                let written = spelling(argument);
                let param = if written.is_empty() { format!("_{index}") } else { ident(&written) };
                let c_type = self.c_type(clang_getCursorType(argument), true).map_err(|reason| tagged(None, &format!("parameter `{param}`"), &reason))?;
                params.push((param, c_type));
            }
            let result = clang_getResultType(ty);
            let result = if result.kind == CXType_Void { None } else { Some(self.c_type(result, false).map_err(|reason| tagged(None, "its result", &reason))?) };
            // `[FFI-29a]` — a function with internal linkage is reached through the shim's wrapper, as
            // is a C++ header's `extern "C"` inline one, whose symbol only a C++ file that uses it has.
            let inline_cpp = self.cpp && clang_Cursor_isFunctionInlined(cursor) != 0;
            if variadic && (clang_getCursorLinkage(cursor) == CXLinkage_Internal || inline_cpp) {
                return Err("a variadic `static inline` function: a wrapper cannot pass `...` on ([FFI-29c])".to_string());
            }
            let link_name = (clang_getCursorLinkage(cursor) == CXLinkage_Internal || inline_cpp).then(|| {
                let wrapper = format!("{}_cshim_{}_{name}", ember_branding::SYMBOL_PREFIX, self.stem);
                self.wrapper(cursor, &wrapper, name);
                wrapper
            });
            // `[FFI-1]` — the function's effects, where its declaration states them.
            let attributes: Vec<CXCursorKind> = children(cursor).into_iter().map(kind).collect();
            let derived = if attributes.contains(&CXCursor_ConstAttr) {
                Derived::Const
            } else if attributes.contains(&CXCursor_PureAttr) {
                Derived::Pure
            } else {
                Derived::Nothing
            };
            Ok(Function { name: name.to_string(), params, result, link_name, abi, policy: Policy::Direct, owner: None, out: false, namespace: Vec::new(), derived, variadic })
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

    /// `[FFI-42]` — an operator declared outside its class: the method of its Ember operator
    /// interface on the class of its first operand (`P operator+(const P&, const P&)` is `P.add`),
    /// a `const` or by-value operand its `self` and any other its `mut self`, as a member's would be
    /// (`[FFI-40a]`'s `mutable` members too). One whose first operand is no class the header
    /// imports, or that maps to no interface, is skipped, saying why.
    fn free_operator(&mut self, cursor: CXCursor, name: &str) {
        // SAFETY(ffi): a live function cursor and its types.
        unsafe {
            let operands = clang_Cursor_getNumArguments(cursor);
            if operands < 1 {
                return self.skip(cursor, name, "an operator with no operand of a class ([FFI-42])");
            }
            let first = clang_getCursorType(clang_Cursor_getArgument(cursor, 0));
            let (class, constant) = match first.kind {
                CXType_LValueReference => (clang_getPointeeType(first), clang_isConstQualifiedType(clang_getPointeeType(first)) != 0),
                CXType_RValueReference => return self.skip(cursor, name, "an operator whose first operand is a `T&&` ([FFI-42])"),
                _ => (first, true),
            };
            let declaration = clang_getTypeDeclaration(clang_getCanonicalType(class));
            let trampoline = |owner: &str| owner.starts_with(&format!("{}_cpp_", ember_branding::SYMBOL_PREFIX));
            let record = if clang_getCanonicalType(class).kind == CXType_Record && !location(declaration).2 { Some(self.record(declaration, None)) } else { None };
            let owner = match record {
                Some(Record::Full(owner) | Record::Opaque(owner, None)) if !trampoline(&owner) => owner,
                _ => return self.skip(cursor, name, "an operator whose first operand is no class the header imports ([FFI-42])"),
            };
            let mutable_state = children(clang_getCursorDefinition(declaration)).into_iter().any(|child| kind(child) == CXCursor_FieldDecl && clang_CXXField_isMutable(child) != 0);
            let member = if constant && !mutable_state { Member::Method } else { Member::MutMethod };
            let shown = format!("{owner}.{name}");
            let (method, interface) = match operator_method(name, operands - 1, member) {
                Ok(found) => found,
                Err(why) => return self.skip(cursor, &shown, why),
            };
            if self.functions.iter().any(|function| function.name == method && function.owner.as_ref().is_some_and(|(of, _)| *of == owner)) {
                return self.skip(cursor, &shown, "an overload: Ember has no overloading, and the first declaration of the name is the one imported");
            }
            let spelled = cpp_spelling(clang_getCursorType(declaration));
            // Written in its class's namespace, with the class's other methods.
            let saved = std::mem::replace(&mut self.writing, namespace_of(declaration));
            match self.cpp_thunk(cursor, method, Some((&owner, member, &spelled))) {
                Ok(function) => {
                    if let Some(interface) = interface {
                        self.operators.push((owner.clone(), method.to_string(), interface.to_string()));
                    }
                    self.functions.push(Function { namespace: self.writing.clone(), ..function });
                }
                Err(reason) => self.skip(cursor, &shown, reason),
            }
            self.writing = saved;
        }
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
            let mutable_state = has_mutable_member(definition);
            // `[FFI-39]` — a class with a trampoline: its methods are its Ember class's, its
            // constructors and destructor the trampoline's, its named virtuals the Ember class's
            // virtual methods. One the trampoline cannot be made for is imported as any class.
            let opaque = owner.clone();
            let trampoline = match self.trampoline_decls.iter().find(|decl| format!("{}_cpp_{}", ember_branding::SYMBOL_PREFIX, decl.class) == owner).cloned() {
                Some(decl) => match self.trampoline_check(definition, &decl) {
                    Ok(named) => Some((decl, named)),
                    Err((code, why)) => {
                        self.skip(definition, &decl.class, format!("[{code}] {why}"));
                        None
                    }
                },
                None => None,
            };
            let owner = trampoline.as_ref().map_or(owner, |(decl, _)| ident(&decl.class));
            let mut unnamed_virtuals = Vec::new();
            // An opaque class's object is freed by its destructor: the one it declares, when that
            // is public, else the implicit one, which is.
            let declared = children(definition).into_iter().find(|child| kind(*child) == CXCursor_Destructor);
            let destructible = by_value
                || declared.is_none_or(|destructor| clang_getCXXAccessSpecifier(destructor) == CX_CXXPublic && clang_getCursorAvailability(destructor) != CXAvailability_NotAvailable);
            if !by_value && destructible && trampoline.is_none() {
                let destroys = format!("{}_cpp_delete_{owner}", ember_branding::SYMBOL_PREFIX);
                let thunk = format!("{}_cpp_{}_{}", ember_branding::SYMBOL_PREFIX, self.stem, self.thunks);
                self.thunks += 1;
                let _ = writeln!(self.shim, "extern \"C\" void {thunk}({class_spelled}* self) noexcept {{ delete self; }}");
                if let Some(item) = self.items.iter_mut().find(|item| ident(&item.name) == owner && item.kind == Kind::Opaque) {
                    item.text = format!("@ffi(destroys({destroys}))\n{}", item.text);
                }
                let owner = Some((owner.clone(), Member::Destructor));
                self.functions.push(Function { name: "drop".to_string(), params: Vec::new(), result: None, link_name: Some(thunk), abi: "C", policy: Policy::Direct, owner, out: false, namespace: self.writing.clone(), derived: Derived::Nothing, variadic: false });
            }
            let mut taken: HashSet<String> = HashSet::new();
            // `[FFI-48]` — the first method of each name, which another is compared with.
            let mut first_of: HashMap<String, CXCursor> = HashMap::new();
            let mut friends = Vec::new();
            // `[FFI-44]` — a class with one public, non-virtual C++ base the header defines has the
            // base's public methods too (a name of its own hides the base's, as in C++): the thunk
            // calls them through the derived object.
            let mut members = children(definition);
            let own = members.len();
            if trampoline.is_none() {
                let mut base = single_public_base(definition);
                for _ in 0..64 {
                    let Some(found) = base else { break };
                    members.extend(children(found).into_iter().filter(|child| kind(*child) == CXCursor_CXXMethod));
                    base = single_public_base(found);
                }
            }
            // ODR-252 — a class with `operator*` is read through `get()`, which `->` is reached
            // through too: its `operator->` is not imported again.
            let dereferences = members.iter().any(|member| kind(*member) == CXCursor_CXXMethod && spelling(*member) == "operator*" && clang_Cursor_getNumArguments(*member) == 0);
            for (index, child) in members.into_iter().enumerate() {
                let inherited = index >= own;
                // `[FFI-42]` — an operator the class declares its friend (often defined there, found
                // only by argument-dependent lookup), as one declared outside it, after the members.
                if kind(child) == CXCursor_FriendDecl {
                    friends.extend(children(child).into_iter().filter(|friend| kind(*friend) == CXCursor_FunctionDecl && spelling(*friend).starts_with("operator")));
                    continue;
                }
                if clang_getCXXAccessSpecifier(child) != CX_CXXPublic || clang_getCursorAvailability(child) == CXAvailability_NotAvailable {
                    continue;
                }
                let name = spelling(child);
                if dereferences && name == "operator->" {
                    continue;
                }
                match kind(child) {
                    // `[FFI-41]` — a nested type, under its outer type's name.
                    CXCursor_StructDecl | CXCursor_ClassDecl | CXCursor_UnionDecl | CXCursor_EnumDecl | CXCursor_TypedefDecl | CXCursor_TypeAliasDecl => {
                        self.declaration(child);
                        continue;
                    }
                    // `[FFI-41]` — a static data member, under its class's name too.
                    CXCursor_VarDecl => {
                        if taken.insert(name.clone()) {
                            self.cpp_variable(child, &name, Some(&owner));
                        } else {
                            self.skip(child, &format!("{owner}.{name}"), "its name is taken by another member of the class");
                        }
                        continue;
                    }
                    _ => {}
                }
                if let Some((_, slots)) = &trampoline {
                    // The virtuals the trampoline passes on (each pure one among them) are the
                    // Ember class's virtual methods.
                    if kind(child) == CXCursor_Constructor || slots.iter().any(|slot| slot.name == name) {
                        continue;
                    }
                    if kind(child) == CXCursor_CXXMethod && clang_CXXMethod_isVirtual(child) != 0 {
                        unnamed_virtuals.push(name.clone());
                    }
                }
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
                    CXCursor_CXXMethod => match self.method_member(child, &owner, mutable_state) {
                        Some(member) => member,
                        None => continue,
                    },
                    _ => continue,
                };
                // `[FFI-42]` — an operator: the method of its Ember operator interface.
                let operator = if name.starts_with("operator") {
                    match operator_method(&name, clang_Cursor_getNumArguments(child), member) {
                        Ok(found) => Some(found),
                        Err(why) => {
                            self.skip(child, &format!("{owner}.{name}"), why);
                            continue;
                        }
                    }
                } else {
                    None
                };
                let key = match member {
                    Member::Constructor | Member::BoxedConstructor => "new".to_string(),
                    Member::Destructor => "drop".to_string(),
                    _ => operator.map_or(name.clone(), |(method, _)| method.to_string()),
                };
                if !taken.insert(key.clone()) {
                    if !inherited {
                        // `[FFI-48]` — the same parameters, another result (`T& get() &`, `T&& get() &&`).
                        let why = match first_of.get(&key) {
                            Some(first) if same_parameters(*first, child) && result_spelling(*first) != result_spelling(child) => {
                                "[E5034] overloads differing only in their return type: Ember has no return-type overloading; rename one in the overlay ([FFI-48])"
                            }
                            _ => "an overload: Ember has no overloading, and the first declaration of the name is the one imported",
                        };
                        self.skip(child, &format!("{owner}.{key}"), why);
                    }
                    continue;
                }
                first_of.insert(key.clone(), child);
                match self.cpp_thunk(child, &key, Some((&owner, member, &class_spelled))) {
                    Ok(function) => {
                        if let Some((method, Some(interface))) = operator {
                            self.operators.push((owner.clone(), method.to_string(), interface.to_string()));
                        }
                        self.functions.push(Function { namespace: self.writing.clone(), ..function });
                    }
                    Err(reason) => self.skip(child, &format!("{owner}.{key}"), reason),
                }
            }
            for friend in friends {
                if self.seen.insert(format!("fn {}", usr(friend))) {
                    self.free_operator(friend, &spelling(friend));
                }
            }
            if let Some((decl, slots)) = trampoline {
                self.trampoline(definition, &opaque, &decl, slots, unnamed_virtuals);
            } else if taken.insert("iter".to_string()) {
                self.iterations.push((definition, owner.clone(), class_spelled.clone(), self.writing.clone()));
            }
        }
    }

    /// `[FFI-42]` — a class with `begin()` and `end()` is iterable: its type gets `iter()`, whose
    /// iterator gives a `ref` to each element, through thunks that walk the C++ iterators (a throw
    /// from one aborts at the boundary, `[FFI-39e]`). `iter()` is a call into C++ like any method,
    /// safe when an overlay entry `fn iter()` asserts it (`[FFI-2]`). `[FFI-42a]` — it takes `mut
    /// self`, so a `for` loop borrows the container mutably, and every method that could invalidate
    /// its iterators — every `mut self` one — is refused inside the loop by the borrow rules.
    fn cpp_iteration(&mut self, class: CXCursor, owner: &str, spelled: &str) {
        // SAFETY(ffi): cursors and types of a live translation unit.
        unsafe {
            let method = |name: &str| {
                children(class)
                    .into_iter()
                    .filter(|child| {
                        kind(*child) == CXCursor_CXXMethod
                            && spelling(*child) == name
                            && clang_getCXXAccessSpecifier(*child) == CX_CXXPublic
                            && clang_CXXMethod_isStatic(*child) == 0
                            && clang_Cursor_getNumArguments(*child) == 0
                    })
                    // The non-`const` one, which a loop that borrows mutably calls.
                    .min_by_key(|child| clang_CXXMethod_isConst(*child))
            };
            let (Some(begin), Some(_)) = (method("begin"), method("end")) else { return };
            let name = format!("{owner}.iter");
            // What `*c.begin()` is, as the parse that asked it of Clang found.
            let element = self.elements.get(spelled).copied().filter(|reference| reference.kind == CXType_LValueReference).map(|reference| clang_getPointeeType(reference));
            let Some(element) = element else {
                self.skip(begin, &name, "`*begin()` gives no `T&` to an element, or `begin() != end()` does not compile ([FFI-42])");
                return;
            };
            let mapped = match self.ty(element, true) {
                Ok(mapped) => mapped,
                Err(reason) => {
                    self.skip(begin, &name, format!("its elements: {reason}"));
                    return;
                }
            };
            // The iterator is the class's, named as a member's static is (`Outer_name`): `Bag_iter`,
            // whose symbol `[MNG-1]` keeps apart from the method `Bag.iter`'s.
            let walker = format!("{owner}_iter");
            if !self.claim(&walker) {
                self.skip(begin, &name, format!("its iterator's name `{walker}` is taken by another declaration of the header ([FFI-42])"));
                return;
            }
            let prefix = ember_branding::SYMBOL_PREFIX;
            let ops = format!("{prefix}_cpp_{}_iter_{}", self.stem, self.thunks);
            self.thunks += 1;
            let _ = writeln!(
                self.shim,
                "struct {ops}_range {{ decltype(std::declval<{spelled}&>().begin()) at; decltype(std::declval<{spelled}&>().end()) end; }};\n\
                 extern \"C\" {ADDRESS} {ops}_begin({spelled}* self) noexcept {{ try {{ return reinterpret_cast<{ADDRESS}>(new {ops}_range{{self->begin(), self->end()}}); }} catch (...) {{ crossed(\"{spelled}::begin\"); }} }}\n\
                 extern \"C\" void* {ops}_next(void* range) noexcept {{ try {{ auto* walk = static_cast<{ops}_range*>(range); if (walk->at == walk->end) return nullptr; auto* item = std::addressof(*walk->at); ++walk->at; return const_cast<void*>(static_cast<const void*>(item)); }} catch (...) {{ crossed(\"the iterator of {spelled}\"); }} }}\n\
                 extern \"C\" void {ops}_end(void* range) noexcept {{ delete static_cast<{ops}_range*>(range); }}"
            );
            self.push(
                &walker,
                Kind::Class,
                format!(
                    "## `[FFI-42]` — a `for` loop over a `{spelled}`, through its `begin()` and `end()`: each element a\n\
                     ## `ref` to it. `[FFI-42a]` — the loop borrows the `{spelled}` mutably.\n\
                     pub struct {walker}:\n    range: *mut u8\n    owner: ref mut {owner}\n\n\
                     \x20   pub fn next(mut self) -> Option[ref {mapped}]:\n\
                     \x20       item = null[*mut {mapped}]()\n\
                     \x20       unsafe:  # SAFETY(ffi): the range's own thunk, while the iterator holds its container\n\
                     \x20           item = {ops}_next(self.range)\n\
                     \x20       if item.is_null():\n            return None\n\
                     \x20       unsafe:  # SAFETY(ffi): an element of the container the iterator holds\n\
                     \x20           return Some({prefix}_cpp_ref(item))\n\n\
                     \x20   fn drop(mut self):\n\
                     \x20       unsafe:  # SAFETY(ffi): the range's end, once\n\
                     \x20           {ops}_end(self.range)\n\n\
                     unsafe extern \"C\":\n\
                     \x20   @ffi(link_name=\"{ops}_next\")\n    fn {ops}_next(range: *mut u8) -> *mut {mapped}\n\
                     \x20   @ffi(link_name=\"{ops}_end\")\n    fn {ops}_end(range: *mut u8)\n"
                ),
            );
            let result = CType {
                ember: "usize".to_string(),
                pointee: Some((walker.clone(), false)),
                char_pointer: false,
                c: ADDRESS.to_string(),
                mode: format!("bridge {walker}(range = @ as *mut u8, owner = ref mut self)"),
                opaque: false,
                function: None,
            };
            let owner = Some((owner.to_string(), Member::MutMethod));
            self.functions.push(Function {
                name: "iter".to_string(),
                params: Vec::new(),
                result: Some(result),
                link_name: Some(format!("{ops}_begin")),
                abi: "C",
                policy: Policy::Direct,
                owner,
                out: false,
                namespace: self.writing.clone(),
                derived: Derived::Nothing,
                variadic: false,
            });
        }
    }

    /// `[FFI-17a]` — the bridge type of a C++ standard-library type, its thunks written the first
    /// time the module's namespace uses it; `None` for a type that is not one of the bridged
    /// templates, `Err` for one whose argument does not map.
    fn bridge(&mut self, ty: CXType) -> Option<Result<Bridge, String>> {
        // SAFETY(ffi): types and cursors of a live translation unit.
        unsafe {
            let ty = clang_getCanonicalType(ty);
            let name = std_template(ty)?;
            if !matches!(name.as_str(), "vector" | "basic_string" | "shared_ptr" | "weak_ptr" | "unique_ptr" | "function") {
                return None;
            }
            let declaration = clang_getTypeDeclaration(ty);
            let spelled = cpp_spelling(clang_getCursorType(declaration));
            let key = (spelled.clone(), self.writing.clone());
            if let Some(found) = self.bridges.get(&key) {
                return Some(Ok(found.clone()));
            }
            // `[FFI-48]` — a container whose allocator (or a string whose traits) is not the standard
            // one: the allocator is part of the type and opaque (`E5034`).
            let defaulted = |index: u32, expected: &str| {
                index as c_int >= clang_Type_getNumTemplateArguments(ty)
                    || std_template(clang_getCanonicalType(clang_Type_getTemplateArgumentAsType(ty, index as c_uint))).as_deref() == Some(expected)
            };
            let standard = match name.as_str() {
                "vector" => defaulted(1, "allocator"),
                "basic_string" => defaulted(1, "char_traits") && defaulted(2, "allocator"),
                _ => true,
            };
            if !standard {
                return Some(Err(format!(
                    "[E5034] `{spelled}`: an allocator-parameterised container: the allocator is part of the type and opaque; expose a span, or keep it native ([FFI-48])"
                )));
            }
            // `[FFI-17a]` — a `std::unique_ptr` with a deleter of its own: its `ForeignBox` frees the
            // object through the deleter, which a stateless one is wherever it is made. One that
            // holds state (a function pointer, a class with members) would need its state kept
            // beside the object, which a `ForeignBox` does not hold.
            let deleter = if name == "unique_ptr" && !defaulted(1, "default_delete") {
                let deleter = clang_getCanonicalType(clang_Type_getTemplateArgumentAsType(ty, 1));
                let definition = clang_getCursorDefinition(clang_getTypeDeclaration(deleter));
                let stateless = deleter.kind == CXType_Record
                    && clang_Cursor_isNull(definition) == 0
                    && clang_Type_getSizeOf(deleter) == 1
                    && !children(definition).into_iter().any(|member| kind(member) == CXCursor_FieldDecl);
                if !stateless {
                    return Some(Err(format!(
                        "[E5034] `{spelled}`: a deleter that holds state (`{}`): a `ForeignBox` keeps only its object's address; free it in C++, or give it a stateless deleter [FFI-17a]",
                        cpp_spelling(deleter)
                    )));
                }
                true
            } else {
                false
            };
            let argument = clang_Type_getTemplateArgumentAsType(ty, 0);
            let prefix = ember_branding::SYMBOL_PREFIX;
            let ops = format!("{prefix}_cpp_{}_bridge_{}", self.stem, self.bridges.len());
            let element = cpp_spelling(argument);
            let mut decls = String::new();
            let declare = |decls: &mut String, suffix: &str, signature: &str| {
                let _ = writeln!(decls, "    @ffi(link_name=\"{ops}_{suffix}\")\n    fn {ops}_{suffix}{signature}");
            };
            let bridge = match name.as_str() {
                // `[FFI-17a]` (ODR-199) — `std::function<R(A…)>`: an owned `CppFunction`, or, with
                // its function's entry's `@ffi(std_function)`, an `owned fn(A…) -> R` capturing it.
                "function" => {
                    let signature = clang_getCanonicalType(argument);
                    if signature.kind != CXType_FunctionProto || clang_isFunctionTypeVariadic(signature) != 0 {
                        return Some(Err(format!("`{spelled}`: a `std::function` of a signature with no Ember function type ([FFI-17a])")));
                    }
                    let (mut params, mut declared, mut passed, mut forwarded) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
                    for index in 0..clang_getNumArgTypes(signature).max(0) as u32 {
                        let param = clang_getArgType(argument, index);
                        if matches!(param.kind, CXType_LValueReference | CXType_RValueReference) {
                            return Some(Err(format!("`{spelled}`: a parameter by reference (`{}`): a `std::function` crosses when its parameters cross by value ([FFI-17a])", cpp_spelling(param))));
                        }
                        let mapped = match self.ty(param, false) {
                            Ok(mapped) => mapped,
                            Err(reason) => return Some(Err(format!("`{spelled}`: its parameter `{}` does not map by value ({reason}) [FFI-17a]", cpp_spelling(param)))),
                        };
                        params.push(format!("a{index}: {mapped}"));
                        declared.push(mapped);
                        passed.push(format!("{} a{index}", cpp_spelling(param)));
                        forwarded.push(format!("a{index}"));
                    }
                    let result = clang_getResultType(argument);
                    let mapped = if clang_getCanonicalType(result).kind == CXType_Void {
                        None
                    } else {
                        match self.ty(result, false) {
                            Ok(mapped) => Some(mapped),
                            Err(reason) => return Some(Err(format!("`{spelled}`: its result does not map by value ({reason}) [FFI-17a]"))),
                        }
                    };
                    let returned = mapped.as_ref().map_or(String::new(), |mapped| format!(" -> {mapped}"));
                    let c_result = if mapped.is_some() { cpp_spelling(result) } else { "void".to_string() };
                    let comma = |items: &[String]| items.iter().map(|item| format!(", {item}")).collect::<String>();
                    // An exception its target throws has nowhere to go in an Ember call: it is reported
                    // and the program stops, naming it ([FFI-39e]).
                    let _ = writeln!(
                        self.shim,
                        "extern \"C\" {c_result} {ops}_call(void* f{}) noexcept {{ try {{ return (*static_cast<{spelled}*>(f))({}); }} catch (...) {{ crossed(\"std::function\"); }} }}\n\
                         extern \"C\" void {ops}_destroy(void* f) noexcept {{ delete static_cast<{spelled}*>(f); }}",
                        comma(&passed),
                        forwarded.join(", ")
                    );
                    declare(&mut decls, "call", &format!("(object: *mut u8{}){returned}", comma(&params)));
                    declare(&mut decls, "destroy", "(object: *mut u8)");
                    let callable = format!("fn({}){returned}", declared.join(", "));
                    self.push(
                        &format!("{ops}_callable"),
                        Kind::Class,
                        format!(
                            "## `[FFI-17a]` (ODR-199) — `{spelled}` as an Ember callable: its call is the thunk's, its capture the `std::function`, deleted with it.\n\
                             fn {ops}_callable(owned holder: CppFunction) -> {callable}:\n    return owned fn({}){returned} => {ops}_invoke(holder{})\n\n\
                             fn {ops}_invoke(holder: ref CppFunction{}){returned}:\n    unsafe:  # SAFETY(ffi): the holder owns a live `{spelled}` of this signature\n        {}{ops}_call(holder.as_ptr(){})\n",
                            params.join(", "),
                            comma(&forwarded),
                            comma(&params),
                            if mapped.is_some() { "return " } else { "" },
                            comma(&forwarded)
                        ),
                    );
                    Bridge {
                        ember: "CppFunction".to_string(),
                        spelled: spelled.clone(),
                        make: format!("function {ops} {callable}"),
                        unique: None,
                    }
                }
                "vector" => {
                    if argument.kind == CXType_Bool {
                        return Some(Err(format!("`{spelled}`: a `std::vector<bool>` stores bits, which no view of its elements can show ([FFI-17a])")));
                    }
                    let mapped = match self.ty(argument, false) {
                        Ok(mapped) => mapped,
                        Err(reason) => return Some(Err(format!("`{spelled}`: its elements do not map by value ({reason}) [FFI-17a]"))),
                    };
                    let _ = writeln!(
                        self.shim,
                        "extern \"C\" {element}* {ops}_data(void* v) noexcept {{ return static_cast<{spelled}*>(v)->data(); }}\n\
                         extern \"C\" std::size_t {ops}_size(void* v) noexcept {{ return static_cast<{spelled}*>(v)->size(); }}\n\
                         extern \"C\" void {ops}_push(void* v, const {element}* value) noexcept {{ try {{ static_cast<{spelled}*>(v)->push_back(*value); }} catch (...) {{ crossed(\"std::vector::push_back\"); }} }}\n\
                         extern \"C\" void {ops}_destroy(void* v) noexcept {{ delete static_cast<{spelled}*>(v); }}"
                    );
                    declare(&mut decls, "data", &format!("(object: *mut u8) -> *mut {mapped}"));
                    declare(&mut decls, "size", "(object: *mut u8) -> usize");
                    declare(&mut decls, "push", &format!("(object: *mut u8, value: *{mapped})"));
                    declare(&mut decls, "destroy", "(object: *mut u8)");
                    Bridge {
                        ember: format!("CppVector[{mapped}]"),
                        spelled: spelled.clone(),
                        make: format!("CppVector.from_thunks(@ as *mut u8, {ops}_data, {ops}_size, {ops}_push, {ops}_destroy)"),
                        unique: None,
                    }
                }
                "basic_string" => {
                    if !matches!(argument.kind, CXType_Char_S | CXType_Char_U) {
                        return Some(Err(format!("`{spelled}`: only a string of `char` crosses, as `CppString` ([FFI-17a])")));
                    }
                    let _ = writeln!(
                        self.shim,
                        "extern \"C\" const char* {ops}_data(void* s) noexcept {{ return static_cast<{spelled}*>(s)->data(); }}\n\
                         extern \"C\" std::size_t {ops}_size(void* s) noexcept {{ return static_cast<{spelled}*>(s)->size(); }}\n\
                         extern \"C\" void {ops}_destroy(void* s) noexcept {{ delete static_cast<{spelled}*>(s); }}"
                    );
                    declare(&mut decls, "data", "(object: *mut u8) -> *u8");
                    declare(&mut decls, "size", "(object: *mut u8) -> usize");
                    declare(&mut decls, "destroy", "(object: *mut u8)");
                    Bridge { ember: "CppString".to_string(), spelled: spelled.clone(), make: format!("CppString.from_thunks(@ as *mut u8, {ops}_data, {ops}_size, {ops}_destroy)"), unique: None }
                }
                "shared_ptr" => {
                    let mapped = match self.ty(argument, true) {
                        Ok(mapped) => mapped,
                        Err(reason) => return Some(Err(format!("`{spelled}`: the type it shares does not map ({reason}) [FFI-17a]"))),
                    };
                    let _ = writeln!(
                        self.shim,
                        "extern \"C\" {element}* {ops}_pointee(void* p) noexcept {{ return static_cast<{spelled}*>(p)->get(); }}\n\
                         extern \"C\" void* {ops}_copy(void* p) noexcept {{ return new {spelled}(*static_cast<{spelled}*>(p)); }}\n\
                         extern \"C\" void {ops}_destroy(void* p) noexcept {{ delete static_cast<{spelled}*>(p); }}"
                    );
                    declare(&mut decls, "pointee", &format!("(object: *mut u8) -> *mut {mapped}"));
                    declare(&mut decls, "copy", "(object: *mut u8) -> *mut u8");
                    declare(&mut decls, "destroy", "(object: *mut u8)");
                    Bridge {
                        ember: format!("CppShared[{mapped}]"),
                        spelled: spelled.clone(),
                        make: format!("CppShared.from_thunks(@ as *mut u8, {ops}_pointee, {ops}_copy, {ops}_destroy)"),
                        unique: None,
                    }
                }
                // `[WK-14]` — a `std::weak_ptr<T>` is a `CppWeak[T]`, never a `Weak`: its `lock()` gives
                // the `CppShared[T]` C++ counts, so it carries that bridge's thunks too.
                "weak_ptr" => {
                    let mapped = match self.ty(argument, true) {
                        Ok(mapped) => mapped,
                        Err(reason) => return Some(Err(format!("`{spelled}`: the type it observes does not map ({reason}) [FFI-17a]"))),
                    };
                    let shared = format!("std::shared_ptr<{element}>");
                    let _ = writeln!(
                        self.shim,
                        "extern \"C\" void* {ops}_lock(void* w) noexcept {{ auto locked = static_cast<{spelled}*>(w)->lock(); return locked ? new {shared}(std::move(locked)) : nullptr; }}\n\
                         extern \"C\" bool {ops}_expired(void* w) noexcept {{ return static_cast<{spelled}*>(w)->expired(); }}\n\
                         extern \"C\" void* {ops}_copy(void* w) noexcept {{ return new {spelled}(*static_cast<{spelled}*>(w)); }}\n\
                         extern \"C\" void {ops}_destroy(void* w) noexcept {{ delete static_cast<{spelled}*>(w); }}\n\
                         extern \"C\" {element}* {ops}_pointee(void* p) noexcept {{ return static_cast<{shared}*>(p)->get(); }}\n\
                         extern \"C\" void* {ops}_shared_copy(void* p) noexcept {{ return new {shared}(*static_cast<{shared}*>(p)); }}\n\
                         extern \"C\" void {ops}_shared_destroy(void* p) noexcept {{ delete static_cast<{shared}*>(p); }}"
                    );
                    declare(&mut decls, "lock", "(object: *mut u8) -> *mut u8");
                    declare(&mut decls, "expired", "(object: *mut u8) -> bool");
                    declare(&mut decls, "copy", "(object: *mut u8) -> *mut u8");
                    declare(&mut decls, "destroy", "(object: *mut u8)");
                    declare(&mut decls, "pointee", &format!("(object: *mut u8) -> *mut {mapped}"));
                    declare(&mut decls, "shared_copy", "(object: *mut u8) -> *mut u8");
                    declare(&mut decls, "shared_destroy", "(object: *mut u8)");
                    Bridge {
                        ember: format!("CppWeak[{mapped}]"),
                        spelled: spelled.clone(),
                        make: format!(
                            "CppWeak.from_thunks(@ as *mut u8, {ops}_lock, {ops}_expired, {ops}_copy, {ops}_destroy, {ops}_pointee, {ops}_shared_copy, {ops}_shared_destroy)"
                        ),
                        unique: None,
                    }
                }
                _ => {
                    // `std::unique_ptr<T>`: the object it gives up, owned by a `ForeignBox` that frees
                    // it with the class's destructor thunk (an opaque class's, `[FFI-36a]`).
                    let declaration = clang_getTypeDeclaration(clang_getCanonicalType(argument));
                    let opaque = matches!(self.record(declaration, None), Record::Opaque(_, None));
                    let mapped = match self.ty(argument, true) {
                        Ok(mapped) if opaque => mapped,
                        Ok(_) => return Some(Err(format!("`{spelled}`: a `std::unique_ptr` crosses as a `ForeignBox` of a class Ember holds behind a pointer ([FFI-17a])"))),
                        Err(reason) => return Some(Err(format!("`{spelled}`: the type it owns does not map ({reason}) [FFI-17a]"))),
                    };
                    if !deleter {
                        let bridge = Bridge { ember: format!("ForeignBox[{mapped}]"), spelled: spelled.clone(), make: format!("adopt[{mapped}](@ as *mut {mapped})"), unique: Some(mapped) };
                        self.bridges.insert(key, bridge.clone());
                        return Some(Ok(bridge));
                    }
                    // The deleter's thunk: a `std::unique_ptr<T, D>` made of the object ends, so
                    // `D{}` frees it.
                    let _ = writeln!(self.shim, "extern \"C\" void {ops}_delete({element}* object) noexcept {{ {spelled} owner(object); }}");
                    declare(&mut decls, "delete", &format!("(object: *mut {mapped})"));
                    Bridge {
                        ember: format!("ForeignBox[{mapped}]"),
                        spelled: spelled.clone(),
                        make: format!("{prefix}_cpp_adopt_deleted[{mapped}](@ as *mut {mapped}, {ops}_delete)"),
                        unique: Some(mapped),
                    }
                }
            };
            self.push(&ops, Kind::Class, format!("## `[FFI-17a]` — the thunks of `{spelled}`.\nunsafe extern \"C\":\n{decls}"));
            self.bridges.insert(key, bridge.clone());
            Some(Ok(bridge))
        }
    }

    /// `[FFI-39]`, `[FFI-17d]` — whether a trampoline can be made for a class: it has a virtual
    /// destructor (`E5059`), is not `final`, each virtual `virtuals` names is one virtual method of
    /// it (`E5057`), every pure virtual it declares is passed on, and each passed-on one passes
    /// values a trampoline can pass (`E5031`). Its named virtuals, as the trampoline calls them.
    fn trampoline_check(&mut self, definition: CXCursor, decl: &TrampolineDecl) -> Result<Vec<Slot>, (&'static str, String)> {
        // SAFETY(ffi): cursors and types of a live translation unit.
        unsafe {
            let spelled = cpp_spelling(clang_getCursorType(definition));
            let key = usr(definition);
            if !self.by_value.get(&format!("virtual_dtor {key}")).copied().unwrap_or(false) {
                return Err(("E5059", format!("`{spelled}` has no virtual destructor, so its trampoline subclass cannot be destroyed through it ([FFI-17d])")));
            }
            if self.by_value.get(&format!("final {key}")).copied().unwrap_or(false) {
                return Err(("E5034", format!("`{spelled}` is `final`: no class derives from it")));
            }
            let methods: Vec<CXCursor> = children(definition).into_iter().filter(|child| kind(*child) == CXCursor_CXXMethod).collect();
            // ODR-221 — what the program's classes override: a name that is no virtual of the base
            // is theirs to report (the Ember class has no such virtual to override). Every pure
            // virtual is passed on: each Ember class deriving from it implements it, as from an
            // abstract class.
            let virtual_named = |name: &str| {
                let found: Vec<&CXCursor> = methods.iter().filter(|method| spelling(**method) == name).collect();
                matches!(found.as_slice(), [method] if clang_CXXMethod_isVirtual(**method) != 0)
            };
            let mut names: Vec<String> = decl.virtuals.iter().filter(|name| !decl.from_program || virtual_named(name)).cloned().collect();
            for pure in methods.iter().filter(|method| clang_CXXMethod_isPureVirtual(**method) != 0) {
                let name = spelling(*pure);
                if !names.contains(&name) {
                    names.push(name);
                }
            }
            let mut slots = Vec::new();
            for name in &names {
                let found: Vec<CXCursor> = methods.iter().copied().filter(|method| spelling(*method) == *name).collect();
                let method = match found.as_slice() {
                    [method] if clang_CXXMethod_isVirtual(*method) != 0 => *method,
                    [] => return Err(("E5057", format!("`virtuals` names `{name}`, and `{spelled}` declares no method `{name}`"))),
                    [_] => return Err(("E5057", format!("`virtuals` names `{name}`, which is not virtual in `{spelled}`"))),
                    _ => return Err(("E5057", format!("`{name}` is a virtual `{spelled}` overloads: a trampoline overrides one function of a name"))),
                };
                let ty = clang_getCursorType(method);
                let mut params = Vec::new();
                for index in 0..clang_Cursor_getNumArguments(method).max(0) as u32 {
                    let argument = clang_Cursor_getArgument(method, index);
                    let written = spelling(argument);
                    let param = if written.is_empty() { format!("_{index}") } else { ident(&written) };
                    let local = format!("a{index}");
                    let argument_ty = clang_getArgType(ty, index);
                    let (c_type, declaration, pass) = match self.cpp_parameter(argument_ty, &local) {
                        Ok(found) if matches!(found.0.mode.as_str(), "" | "ref " | "ref mut ") => found,
                        Ok(_) => return Err(("E5031", format!("parameter `{param}` of `{spelled}::{name}`: a trampoline passes values, and references to classes Ember holds behind a pointer"))),
                        Err(reason) => return Err(("E5031", format!("parameter `{param}` of `{spelled}::{name}`: {reason}"))),
                    };
                    let Some(c) = declaration.strip_suffix(&local).map(str::trim_end).filter(|c| !c.is_empty() && !c.contains('(')) else {
                        return Err(("E5031", format!("parameter `{param}` of `{spelled}::{name}`: a type a function pointer passes in another form")));
                    };
                    let to_ember = if pass.starts_with('*') { format!("&{local}") } else { local.clone() };
                    params.push(SlotParam { name: param, ty: c_type, c: c.to_string(), to_ember, declaration: format!("{} {local}", cpp_spelling(argument_ty)), to_base: pass, local });
                }
                let result_ty = clang_getResultType(ty);
                let result = if result_ty.kind == CXType_Void {
                    None
                } else {
                    if matches!(clang_getCanonicalType(result_ty).kind, CXType_Record | CXType_LValueReference | CXType_RValueReference) {
                        return Err(("E5031", format!("`{spelled}::{name}` returns `{}`: a trampoline returns a value Ember passes as C does", cpp_spelling(result_ty))));
                    }
                    Some(self.c_type(result_ty, false).map_err(|reason| ("E5031", format!("the result of `{spelled}::{name}`: {reason}")))?)
                };
                let result_spelled = if result.is_some() { cpp_spelling(result_ty) } else { "void".to_string() };
                slots.push(Slot {
                    name: name.clone(),
                    constant: clang_CXXMethod_isConst(method) != 0,
                    pure: clang_CXXMethod_isPureVirtual(method) != 0,
                    params,
                    result,
                    result_spelled,
                });
            }
            Ok(slots)
        }
    }

    /// `[FFI-39]`, `[FFI-39b]`, `[FFI-39e]`, `[FFI-17c]`, `[FFI-17d]` — the trampoline of a class: a
    /// C++ subclass whose named virtuals call Ember through entry functions it is given when made,
    /// and an Ember `open class` of the class's name, which owns one. Its `init` (and an
    /// `ember_cpp_init_<k>` for each other constructor, which `super.init(…)` selects among by
    /// arity and types) makes the C++ object; its named virtuals are virtual methods whose own body
    /// is the C++ base class's implementation; the C++ object is deleted when the Ember fields have
    /// been dropped, through a base field of a struct whose `drop` deletes it. With
    /// `owner="foreign"`, a foreign `delete` destroys the Ember object (which then holds no handle
    /// but the one given to C++); with `owner="ember"` it panics. An exception that would leave a
    /// constructor or a base implementation for Ember aborts, naming it.
    fn trampoline(&mut self, definition: CXCursor, opaque: &str, decl: &TrampolineDecl, slots: Vec<Slot>, unnamed_virtuals: Vec<String>) {
        let prefix = ember_branding::SYMBOL_PREFIX;
        let class = ident(&decl.class);
        let stem = self.stem.clone();
        // SAFETY(ffi): cursors and types of a live translation unit.
        unsafe {
            let spelled = cpp_spelling(clang_getCursorType(definition));
            let tramp = format!("{prefix}_tramp_{stem}_{}", decl.class);
            let entry = |k: &dyn std::fmt::Display| format!("{prefix}_cpp_tramp_{stem}_{}_{k}", decl.class);
            // The entry functions' C types, as the trampoline holds them.
            let pointer_types: Vec<String> = slots
                .iter()
                .enumerate()
                .map(|(k, slot)| format!("{} (*v{k})(void*{})", slot.result_spelled, slot.params.iter().map(|param| format!(", {}", param.c)).collect::<String>()))
                .collect();
            // The trampoline's own members, under the prefix no C++ class uses.
            let p = ember_branding::SYMBOL_PREFIX;
            let mut text = String::new();
            let _ = writeln!(text, "\n/* `[FFI-39]` — the trampoline of `{spelled}`: its named virtuals call the Ember object it was made for. */");
            let _ = writeln!(text, "struct {tramp} final : {spelled} {{");
            let _ = writeln!(text, "    void* {p}_self;\n    void (*{p}_deleted)(void*);");
            for pointer in &pointer_types {
                let _ = writeln!(text, "    {};", pointer.replacen("(*v", &format!("(*{p}_v"), 1));
            }
            let _ = writeln!(text, "    bool {p}_cpp_deleting = false;");
            let mut parameters = vec!["void* self".to_string(), "void (*deleted)(void*)".to_string()];
            parameters.extend(pointer_types.iter().cloned());
            let mut initialisers = vec![format!("{spelled}(static_cast<A&&>(a)...)"), format!("{p}_self(self)"), format!("{p}_deleted(deleted)")];
            initialisers.extend((0..slots.len()).map(|k| format!("{p}_v{k}(v{k})")));
            let _ = writeln!(text, "    template <class... A>\n    explicit {tramp}({}, A&&... a) : {} {{}}", parameters.join(", "), initialisers.join(", "));
            for (k, slot) in slots.iter().enumerate() {
                let declared: Vec<&str> = slot.params.iter().map(|param| param.declaration.as_str()).collect();
                let to_ember: String = slot.params.iter().map(|param| format!(", {}", param.to_ember)).collect();
                let locals: Vec<&str> = slot.params.iter().map(|param| param.local.as_str()).collect();
                let fallback = if slot.pure { "std::abort();".to_string() } else { format!("return {spelled}::{}({});", slot.name, locals.join(", ")) };
                // ODR-221 — an object whose Ember class does not override it was given no entry: the
                // C++ implementation, directly.
                let _ = writeln!(
                    text,
                    "    {} {}({}){} noexcept override {{ if ({p}_self != nullptr && {p}_v{k} != nullptr) return {p}_v{k}({p}_self{to_ember}); {fallback} }}",
                    slot.result_spelled,
                    slot.name,
                    declared.join(", "),
                    if slot.constant { " const" } else { "" }
                );
            }
            let _ = writeln!(
                text,
                "    ~{tramp}() override {{ if ({p}_self != nullptr) {{ void* self = {p}_self; {p}_self = nullptr; {p}_cpp_deleting = true; {p}_deleted(self); }} }}\n}};"
            );
            self.shim.push_str(&text);

            // The Ember side's raw declarations, entries and class.
            let mut raw = String::new();
            let mut body = String::new();
            let fn_types: Vec<String> = slots
                .iter()
                .map(|slot| {
                    let params: String = slot.params.iter().map(|param| format!(", {}", param.ty.ember)).collect();
                    let result = slot.result.as_ref().map(|ty| format!(" -> {}", ty.ember)).unwrap_or_default();
                    format!("extern \"C\" fn(*mut u8{params}){result}")
                })
                .collect();
            let entries: Vec<String> = (0..slots.len()).map(|k| entry(&k)).collect();
            let deleted = entry(&"deleted");
            // Each constructor: the default one is `init`, any other an `ember_cpp_init_<k>`;
            // a class that declares none has the implicit default one.
            let mut ctors: Vec<Option<CXCursor>> = children(definition)
                .into_iter()
                .filter(|child| {
                    kind(*child) == CXCursor_Constructor
                        && clang_getCXXAccessSpecifier(*child) != CX_CXXPrivate
                        && clang_getCursorAvailability(*child) != CXAvailability_NotAvailable
                        && clang_CXXConstructor_isCopyConstructor(*child) == 0
                        && clang_CXXConstructor_isMoveConstructor(*child) == 0
                })
                .map(Some)
                .collect();
            let declares_any = children(definition).into_iter().any(|child| kind(child) == CXCursor_Constructor);
            if !declares_any {
                ctors.push(None);
            }
            let default_at = ctors.iter().position(|ctor| ctor.is_none_or(|ctor| clang_CXXConstructor_isDefaultConstructor(ctor) != 0));
            let init_at = default_at.unwrap_or(0);
            for (j, ctor) in ctors.iter().enumerate() {
                // A default constructor is called with no arguments, its own defaults applying.
                let mut params: Vec<(String, CType, String, String)> = Vec::new();
                if let Some(ctor) = ctor.filter(|_| Some(j) != default_at) {
                    let ty = clang_getCursorType(ctor);
                    for index in 0..clang_Cursor_getNumArguments(ctor).max(0) as u32 {
                        let argument = clang_Cursor_getArgument(ctor, index);
                        let written = spelling(argument);
                        let param = if written.is_empty() { format!("_{index}") } else { ident(&written) };
                        match self.cpp_parameter(clang_getArgType(ty, index), &format!("a{index}")) {
                            Ok((c_type, declaration, pass)) => params.push((param, c_type, declaration, pass)),
                            Err(reason) => {
                                self.skip(ctor, &format!("{class}.init"), tagged(Some("E5031"), &format!("parameter `{param}`"), &reason));
                                params.clear();
                                break;
                            }
                        }
                    }
                    if params.len() != clang_Cursor_getNumArguments(ctor).max(0) as usize {
                        continue;
                    }
                }
                let thunk = format!("{prefix}_cpp_{stem}_{}", self.thunks);
                self.thunks += 1;
                let declared: String = params.iter().map(|(_, _, declaration, _)| format!(", {declaration}")).collect();
                let passed: String = params.iter().map(|(_, _, _, pass)| format!(", {pass}")).collect();
                let entry_args: String = (0..slots.len()).map(|k| format!(", v{k}")).collect();
                let _ = writeln!(
                    self.shim,
                    "extern \"C\" {spelled}* {thunk}({}{declared}) noexcept {{ try {{ return new {tramp}(self, deleted{entry_args}{passed}); }} catch (...) {{ crossed(\"the constructor of {spelled}\"); }} }}",
                    parameters.join(", ")
                );
                let construct = format!("{prefix}_raw_{class}_make_{j}");
                let mut raw_params = vec!["object: *mut u8".to_string(), "deleted: extern \"C\" fn(*mut u8)".to_string()];
                raw_params.extend(fn_types.iter().enumerate().map(|(k, ty)| format!("v{k}: Option[{ty}]")));
                raw_params.extend(params.iter().map(|(param, ty, _, _)| if wrapped(ty) { format!("{param}: {}", ty.ember) } else { format!("{}{param}: {}", ty.mode, ty.ember) }));
                let _ = writeln!(raw, "    @ffi(link_name=\"{thunk}\")\n    fn {construct}({}) -> *mut {opaque}", raw_params.join(", "));
                let init = if j == init_at { "init".to_string() } else { format!("{prefix}_cpp_init_{j}") };
                let wrapper_params: Vec<String> = params.iter().map(|(param, ty, _, _)| wrapper_param(param, ty)).collect();
                let arguments: String = params.iter().map(|(param, ty, _, _)| format!(", {}", wrapper_argument(param, ty))).collect();
                let receiver = if wrapper_params.is_empty() { "mut self".to_string() } else { format!("mut self, {}", wrapper_params.join(", ")) };
                // ODR-221 — the entries of the virtuals the object's own class overrides, exactly: a
                // pure virtual every class implements.
                let given: String = entries
                    .iter()
                    .zip(&slots)
                    .map(|(entry, slot)| if slot.pure {
                        format!(", Some({entry})")
                    } else {
                        format!(", Some({entry}) if {prefix}_cpp_overrides[{class}](self, \"{}\") else None", ident(&slot.name))
                    })
                    .collect();
                let _ = write!(
                    body,
                    "\n    pub fn {init}({receiver}):\n        unsafe:  # SAFETY: the C++ constructor takes this function's own arguments, and the object's address, which the trampoline keeps without a count\n            self.{prefix}_cpp_base = {prefix}_cpp_base_{class}(object = {construct}({prefix}_cpp_address(self), {deleted}{given}{arguments}))\n"
                );
            }
            // The named virtuals: Ember's virtual methods, and the entries the trampoline calls.
            let mut functions = String::new();
            for (k, slot) in slots.iter().enumerate() {
                let wrapper_params: Vec<String> = slot.params.iter().map(|param| wrapper_param(&param.name, &param.ty)).collect();
                let names: Vec<String> = slot.params.iter().map(|param| param.name.clone()).collect();
                let result = slot.result.as_ref().map(|ty| format!(" -> {}", ty.ember)).unwrap_or_default();
                let receiver = if wrapper_params.is_empty() { "self".to_string() } else { format!("self, {}", wrapper_params.join(", ")) };
                let name = ident(&slot.name);
                if slot.pure {
                    // A pure virtual: each Ember class deriving from the class implements it.
                    let _ = write!(body, "\n    pub virtual fn {name}({receiver}){result}\n");
                } else {
                    let helper = format!("{prefix}_cpp_base_{}", slot.name);
                    let _ = write!(
                        body,
                        "\n    pub virtual fn {name}({receiver}){result}:\n        unsafe:  # SAFETY: the C++ class's own implementation; the overlay's trampoline entry asserts its virtuals\n            {}self.{helper}({})\n",
                        if slot.result.is_some() { "return " } else { "" },
                        names.join(", ")
                    );
                    // The base implementation, reached past the trampoline: an exception that would
                    // leave it for Ember aborts ([FFI-39e]).
                    let thunk = format!("{prefix}_cpp_{stem}_{}", self.thunks);
                    self.thunks += 1;
                    let declared: String = slot.params.iter().map(|param| format!(", {} {}", param.c, param.local)).collect();
                    let passed: Vec<&str> = slot.params.iter().map(|param| param.to_base.as_str()).collect();
                    let _ = writeln!(
                        self.shim,
                        "extern \"C\" {} {thunk}({}{spelled}* self{declared}) noexcept {{ try {{ return self->{spelled}::{}({}); }} catch (...) {{ crossed(\"{spelled}::{}\"); }} }}",
                        slot.result_spelled,
                        if slot.constant { "const " } else { "" },
                        slot.name,
                        passed.join(", "),
                        slot.name
                    );
                    let member = if slot.constant { Member::Method } else { Member::MutMethod };
                    self.functions.push(Function {
                        name: helper,
                        params: slot.params.iter().map(|param| (param.name.clone(), param.ty.clone())).collect(),
                        result: slot.result.clone(),
                        link_name: Some(thunk),
                        abi: "C",
                        policy: Policy::Direct,
                        owner: Some((class.clone(), member)),
                        out: false,
                        namespace: self.writing.clone(),
                        derived: Derived::Nothing,
                        variadic: false,
                    });
                }
                let entry_params: String = slot.params.iter().map(|param| format!(", {}: {}", param.local, param.ty.ember)).collect();
                let arguments: Vec<String> = slot
                    .params
                    .iter()
                    .map(|param| match param.ty.mode.as_str() {
                        "ref " => format!("{prefix}_cpp_ref({})", param.local),
                        "ref mut " => format!("{prefix}_cpp_ref_mut({})", param.local),
                        _ => param.local.clone(),
                    })
                    .collect();
                let _ = write!(
                    functions,
                    "## `[FFI-39]` — `{spelled}::{}` as its trampoline calls it: the Ember object's own method, virtually.\nextern \"C\" fn {}(object: *mut u8{entry_params}){result}:\n    unsafe:  # SAFETY: the trampoline passes the address of the object it was made for, which lives while C++ calls it\n        {}{prefix}_cpp_handle[{class}](object).{name}({})\n\n",
                    slot.name,
                    entries[k],
                    if slot.result.is_some() { "return " } else { "" },
                    arguments.join(", ")
                );
            }
            if decl.foreign {
                let _ = write!(
                    functions,
                    "## `[FFI-17d]` — a foreign `delete` of the C++ object (`owner=\"foreign\"`): the Ember object ends with the handle C++ was given.\nextern \"C\" fn {deleted}(object: *mut u8):\n    unsafe:  # SAFETY: C++ deletes the object it was given, whose count is the one it holds\n        _ = {prefix}_cpp_adopt[{class}](object)\n\n"
                );
            } else {
                let _ = write!(
                    functions,
                    "## `[FFI-17d]` — a foreign `delete` of an object Ember owns (`owner=\"ember\"`).\nextern \"C\" fn {deleted}(_object: *mut u8):\n    panic(\"a foreign `delete` of a `{class}`, which Ember owns: its trampoline declares `owner=\\\"ember\\\"` [FFI-17d]\")\n\n"
                );
            }
            // The C++ object's end: after the Ember fields, by the base field's `drop` ([FFI-17c]).
            let destroy_thunk = format!("{prefix}_cpp_{stem}_{}", self.thunks);
            self.thunks += 1;
            let _ = writeln!(
                self.shim,
                "extern \"C\" void {destroy_thunk}({spelled}* object) noexcept {{ auto* made = static_cast<{tramp}*>(object); if (made->{prefix}_cpp_deleting) return; made->{prefix}_self = nullptr; delete made; }}"
            );
            let destroy = format!("{prefix}_raw_{class}_destroy");
            let _ = writeln!(raw, "    @ffi(link_name=\"{destroy_thunk}\")\n    fn {destroy}(object: *mut {opaque})");
            let mut item = String::new();
            let _ = write!(
                item,
                "## `[FFI-39]`, `[FFI-17c]` — the C++ object a `{class}` owns, deleted when its Ember fields have been dropped.\nstruct {prefix}_cpp_base_{class}:\n    object: *mut {opaque}\n\n    fn drop(mut self):\n        if not self.object.is_null():\n            unsafe:  # SAFETY: the C++ object this Ember object made, deleted once\n                {destroy}(self.object)\n\n"
            );
            // A class with a pure virtual is abstract (`[CLS-4]`): what derives from it implements it.
            let openness = if slots.iter().any(|slot| slot.pure) { "abstract" } else { "open" };
            let _ = write!(
                item,
                "## `[FFI-39]` — `{spelled}`, which an Ember class may derive from: the virtuals Ember classes override are virtual methods (ODR-221).\npub {openness} class {class}:\n    {prefix}_cpp_base: {prefix}_cpp_base_{class} = {prefix}_cpp_base_{class}(object = null[*mut {opaque}]())\n{body}\n"
            );
            item.push_str(&functions);
            let _ = write!(item, "unsafe extern \"C\":\n{raw}");
            self.push(&decl.class, Kind::Class, item);
            self.trampolines.push(Trampoline { class: decl.class.clone(), opaque: opaque.to_string(), default_init: default_at.is_some(), unnamed_virtuals });
        }
    }

    /// `[FFI-40]`, `[FFI-41]` — what a method of the class `owner` is: a static one an associated
    /// function, a `const` one `self` (unless the class has a `mutable` member, `[FFI-40a]`), any
    /// other `mut self`; `None` for an `&&`-qualified one (`W5033`).
    /// `[FFI-32]` (ODR-166) — the Ember name of the opaque class a by-value C++ value of `ty` is,
    /// when a `ForeignBox` can own one: a complete class of the header, not the standard library's
    /// (`[FFI-17a]` bridges those), that is not abstract and whose destructor is public.
    fn boxed_class(&mut self, ty: CXType) -> Option<String> {
        if !self.cpp {
            return None;
        }
        // SAFETY(ffi): types and cursors of a live translation unit.
        unsafe {
            let canonical = clang_getCanonicalType(ty);
            if canonical.kind != CXType_Record {
                return None;
            }
            let declaration = clang_getTypeDeclaration(canonical);
            let definition = clang_getCursorDefinition(declaration);
            if location(declaration).2 || clang_Cursor_isNull(definition) != 0 || clang_CXXRecord_isAbstract(definition) != 0 {
                return None;
            }
            let destructor = children(definition).into_iter().find(|child| kind(*child) == CXCursor_Destructor);
            if destructor.is_some_and(|destructor| clang_getCXXAccessSpecifier(destructor) != CX_CXXPublic || clang_getCursorAvailability(destructor) == CXAvailability_NotAvailable) {
                return None;
            }
            match self.record(declaration, None) {
                Record::Opaque(name, None) => Some(self.qualify(name, declaration)),
                _ => None,
            }
        }
    }

    fn method_member(&mut self, method: CXCursor, owner: &str, mutable_state: bool) -> Option<Member> {
        let name = spelling(method);
        // SAFETY(ffi): a live method cursor.
        unsafe {
            if clang_Type_getCXXRefQualifier(clang_getCursorType(method)) == CXRefQualifier_RValue {
                let at = self.at(method);
                self.rvalue_members.push(Skipped {
                    name: format!("{owner}.{name}"),
                    reason: "an `&&`-qualified member function, which only a temporary may call".to_string(),
                    at,
                    namespace: self.writing.clone(),
                });
                return None;
            }
            // `[FFI-40a]` (ODR-160) — a `const` method of a class with a `mutable` member is `self`
            // only when its body, which the header has, changes no field.
            Some(if clang_CXXMethod_isStatic(method) != 0 {
                Member::Static
            } else if clang_CXXMethod_isConst(method) != 0 && (!mutable_state || !changes_object(method, &mut HashSet::new())) {
                Member::Method
            } else {
                Member::MutMethod
            })
        }
    }

    /// A thunk for a C++ function or member (`member`: its class's Ember name, what it is, and the
    /// class as C++ spells it).
    fn cpp_thunk(&mut self, cursor: CXCursor, name: &str, member: Option<(&str, Member, &str)>) -> Result<Function, String> {
        // SAFETY(ffi): a live function cursor and its types.
        unsafe {
            let ty = clang_getCursorType(cursor);
            if clang_isFunctionTypeVariadic(ty) != 0 {
                return Err("a variadic C++ function: a thunk cannot pass `...` on ([FFI-8])".to_string());
            }
            if let Some(why) = coroutine(cursor) {
                return Err(why);
            }
            let mut params = Vec::new();
            let mut declared = Vec::new();
            let mut passed = Vec::new();
            // `[FFI-42]` — a free operator as a method: its first operand is the receiver.
            let free_operator = member.is_some() && kind(cursor) == CXCursor_FunctionDecl;
            // `[FFI-11f]` — the parameters declared `[[clang::lifetimebound]]`, with their modes.
            let mut lifetime_bound: Vec<(String, String)> = Vec::new();
            for index in u32::from(free_operator)..clang_Cursor_getNumArguments(cursor).max(0) as u32 {
                let argument = clang_Cursor_getArgument(cursor, index);
                let written = spelling(argument);
                let param = if written.is_empty() { format!("_{index}") } else { ident(&written) };
                let (c_type, declaration, pass) = self
                    .cpp_parameter(clang_getCursorType(argument), &format!("a{index}"))
                    .map_err(|reason| tagged(Some("E5031"), &format!("parameter `{param}`"), &reason))?;
                if lifetimebound(argument) {
                    lifetime_bound.push((param.clone(), c_type.mode.clone()));
                }
                params.push((param, c_type));
                declared.push(declaration);
                passed.push(pass);
            }
            let policy = self.policy(cursor);
            let result_ty = clang_getResultType(ty);
            // `[FFI-42]` — a compound assignment's `T&` result is dropped: `AddAssign.add_assign`
            // and the others give nothing back.
            let discarded = name.ends_with("_assign") && spelling(cursor).starts_with("operator");
            let arguments = passed.join(", ");
            let call = match member {
                None => format!("{}({arguments})", self.callee.clone().unwrap_or_else(|| qualified_name(cursor))),
                Some((_, Member::Static, class)) => format!("::{class}::{}({arguments})", spelling(cursor)),
                // `[FFI-42]` — a free operator, used as C++ writes it, which finds it as C++ would.
                Some((_, Member::Method | Member::MutMethod, _)) if free_operator => {
                    let operator = spelling(cursor).trim_start_matches("operator").trim().to_string();
                    if arguments.is_empty() { format!("{operator}(*self)") } else { format!("(*self) {operator} ({arguments})") }
                }
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
            // `[FFI-17a]` — a standard-library result's bridge type, if it is one.
            let bridged_result = if discarded || result_ty.kind == CXType_Void || member.is_some_and(|(_, member, _)| matches!(member, Member::Constructor | Member::BoxedConstructor)) {
                None
            } else {
                self.bridge(clang_getCanonicalType(result_ty))
            };
            // `[FFI-17a]` — a view or an optional value: the thunk gives the view's address and writes
            // its length, or says whether the optional has a value and writes it.
            let viewed = if discarded || result_ty.kind == CXType_Void || bridged_result.is_some() || member.is_some_and(|(_, member, _)| matches!(member, Member::Constructor | Member::BoxedConstructor)) {
                None
            } else {
                self.std_result(clang_getCanonicalType(result_ty))
            };
            // `[FFI-32]` (ODR-166) — an opaque class by value is held as a constructor's object is.
            let boxed = if member.is_some_and(|(_, member, _)| matches!(member, Member::Constructor | Member::BoxedConstructor))
                || bridged_result.is_some()
                || viewed.is_some()
                || self.variant_of(result_ty).is_some()
            {
                None
            } else {
                self.boxed_class(result_ty)
            };
            // `[FFI-17a]` (ODR-198) — a variant the overlay names: written through the pointer the
            // caller gives, as a class by value is, into the bytes its enum is made from.
            let variant_result = if discarded || result_ty.kind == CXType_Void { None } else { self.variant_of(result_ty) };
            let (result, returned, call) = match (member, result_ty.kind) {
                _ if let Some(variant) = &variant_result => {
                    let ember = CType {
                        ember: variant.storage.clone(),
                        pointee: None,
                        char_pointer: false,
                        c: cpp_spelling(clang_getCanonicalType(result_ty)),
                        mode: format!("variant {} {}", variant.ops, variant.ember),
                        opaque: false,
                        function: None,
                    };
                    (Some(ember), Some(cpp_spelling(clang_getCanonicalType(result_ty))), call)
                }
                (Some((owner, Member::Constructor, class)), _) => {
                    let ember = CType { ember: owner.to_string(), pointee: None, char_pointer: false, c: class.to_string(), mode: String::new(), opaque: false, function: None };
                    (Some(ember), Some(class.to_string()), call)
                }
                // An object the caller comes to own is given as its address (`ADDRESS`, `usize`): a
                // contracted declaration may return that (`[FFI-2]`), and the wrapper adopts it.
                (Some((owner, Member::BoxedConstructor, _)), _) => {
                    let ember = CType { ember: "usize".to_string(), pointee: Some((owner.to_string(), false)), char_pointer: false, c: ADDRESS.to_string(), mode: String::new(), opaque: false, function: None };
                    (Some(ember), Some(ADDRESS.to_string()), format!("reinterpret_cast<{ADDRESS}>({call})"))
                }
                (_, CXType_Void) => (None, None, call),
                _ if discarded => (None, None, call),
                // `[FFI-32]` (ODR-166) — moved into an object of its own, which the `ForeignBox` the
                // wrapper adopts it into frees with the class's destructor.
                _ if let Some(owner) = &boxed => {
                    let class = cpp_spelling(result_ty);
                    let ember = CType { ember: "usize".to_string(), pointee: Some((owner.clone(), false)), char_pointer: false, c: ADDRESS.to_string(), mode: "boxed".to_string(), opaque: false, function: None };
                    (Some(ember), Some(ADDRESS.to_string()), format!("reinterpret_cast<{ADDRESS}>(new {class}({call}))"))
                }
                _ if let Some(found) = &viewed => {
                    let (ember, element) = found.clone().map_err(|reason| tagged(None, "its result", &reason))?;
                    if ember.mode.starts_with("fixed ") {
                        let made = format!("[&] {{ auto view = {call}; return view.data(); }}()");
                        (Some(ember), Some(format!("{element}*")), made)
                    } else if ember.mode == "optional" {
                        let made = format!(
                            "[&] {{ auto optional = {call}; if (optional) {{ ::new (static_cast<void*>(value)) {element}(*optional); return true; }} std::memset(static_cast<void*>(value), 0, sizeof *value); return false; }}()"
                        );
                        (Some(ember), Some("bool".to_string()), made)
                    } else {
                        let made = format!("[&] {{ auto view = {call}; *size = view.size(); return reinterpret_cast<{ADDRESS}>(view.data()); }}()");
                        (Some(ember), Some(ADDRESS.to_string()), made)
                    }
                }
                // `[FFI-17a]` — a standard-library value: moved to the heap (or, for a
                // `std::unique_ptr`, given up) for its bridge type to own.
                _ if bridged_result.is_some() => {
                    let bridge = bridged_result.clone().into_iter().next().expect("a bridge").map_err(|reason| tagged(None, "its result", &reason))?;
                    match &bridge.unique {
                        Some(_) => {
                            let ember = CType { ember: "usize".to_string(), pointee: Some((bridge.ember.clone(), false)), char_pointer: false, c: ADDRESS.to_string(), mode: format!("bridge {}", bridge.make), opaque: false, function: None };
                            (Some(ember), Some(ADDRESS.to_string()), format!("reinterpret_cast<{ADDRESS}>(({call}).release())"))
                        }
                        None => {
                            // ODR-199 — a `std::function`'s mode is its own (`function <ops> <type>`).
                            let mode = if bridge.make.starts_with("function ") { bridge.make.clone() } else { format!("bridge {}", bridge.make) };
                            let ember = CType { ember: "usize".to_string(), pointee: Some((bridge.ember.clone(), false)), char_pointer: false, c: ADDRESS.to_string(), mode, opaque: false, function: None };
                            (Some(ember), Some(ADDRESS.to_string()), format!("reinterpret_cast<{ADDRESS}>(new {}({call}))", bridge.spelled))
                        }
                    }
                }
                (_, CXType_LValueReference) => {
                    let pointee = clang_getPointeeType(result_ty);
                    let constant = clang_isConstQualifiedType(pointee) != 0;
                    let inner = self.ty(pointee, true).map_err(|reason| tagged(None, "its result", &reason))?;
                    let pointer = format!("*{}{inner}", if constant { "" } else { "mut " });
                    let c = type_spelling(result_ty);
                    // `[FFI-40]` — a method's `T&` result is a reference borrowing its receiver:
                    // `ref mut` from a `mut self` method unless the `T` is `const`, else `ref` (a
                    // `const` method's receiver is a shared borrow). A free or static function's
                    // is a raw pointer: it has no receiver to borrow.
                    let mode = match member {
                        Some((_, Member::MutMethod, _)) if !constant => "ref mut",
                        Some((_, Member::Method | Member::MutMethod, _)) => "ref",
                        _ => "",
                    }
                    .to_string();
                    // `[FFI-11f]` — a result `[[clang::lifetimebound]]` ties to a parameter the thunk
                    // takes by address (not one it copies): a `ref` borrowing it too, an asserted fact.
                    let by_address = |mode: &str| matches!(mode, "ref " | "ref mut " | "mut " | "bridged " | "bridged mut " | "tramp ");
                    let bound: Vec<&(String, String)> = lifetime_bound.iter().filter(|(_, mode)| by_address(mode)).collect();
                    let method = matches!(member, Some((_, Member::Method | Member::MutMethod, _)));
                    let mode = if bound.is_empty() {
                        mode
                    } else {
                        let mutable = !constant && !matches!(member, Some((_, Member::Method, _))) && bound.iter().all(|(_, mode)| mode.contains("mut"));
                        let from: Vec<&str> = method.then_some("self").into_iter().chain(bound.iter().map(|(param, _)| param.as_str())).collect();
                        format!("ref{} from {}", if mutable { " mut" } else { "" }, from.join(", "))
                    };
                    let ember = CType { ember: pointer, pointee: Some((inner, constant)), char_pointer: false, c, mode, opaque: false, function: None };
                    (Some(ember), Some(format!("{}*", cpp_spelling(pointee))), format!("&{call}"))
                }
                _ => {
                    let mapped = self.c_type(result_ty, false).map_err(|reason| tagged(None, "its result", &reason))?;
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
                || (result.is_some()
                    && clang_getCanonicalType(result_ty).kind == CXType_Record
                    && !matches!(member, Some((_, Member::BoxedConstructor, _)))
                    && boxed.is_none()
                    && viewed.is_none()
                    && !result.as_ref().is_some_and(|ty| ty.mode.starts_with("bridge ") || ty.mode.starts_with("function ")));
            let (returned, call) = match (class_result, returned) {
                (true, Some(class)) => {
                    declared.push(format!("{class}* out"));
                    let made = if matches!(member, Some((_, Member::Constructor, _))) { call } else { format!("{class}({call})") };
                    (None, format!("::new (static_cast<void*>(out)) {made}"))
                }
                (_, returned) => (returned, call),
            };
            // What the thunk writes beside its result, cleared when it caught.
            let cleared = match result.as_ref().map(|ty| ty.mode.as_str()) {
                _ if class_result => " std::memset(static_cast<void*>(out), 0, sizeof *out);",
                Some("view" | "view mut") => {
                    declared.push("std::size_t* size".to_string());
                    " *size = 0;"
                }
                Some("optional") => {
                    let element = viewed.clone().and_then(Result::ok).map(|(_, element)| element).unwrap_or_default();
                    declared.push(format!("{element}* value"));
                    " std::memset(static_cast<void*>(value), 0, sizeof *value);"
                }
                _ => "",
            };
            if catching {
                declared.push("int* failed".to_string());
            }
            let declared = if declared.is_empty() { "void".to_string() } else { declared.join(", ") };
            let body = match (&returned, catching) {
                (None, false) => format!("{call};"),
                (Some(_), false) => format!("return {call};"),
                (None, true) => format!("try {{ *failed = 0; {call}; }} catch (...) {{ *failed = 1; caught();{cleared} }}"),
                (Some(returned), true) => format!("try {{ *failed = 0; return {call}; }} catch (...) {{ *failed = 1; caught();{cleared} return zero<{returned}>(); }}"),
            };
            let returned = returned.unwrap_or_else(|| "void".to_string());
            let _ = writeln!(self.shim, "extern \"C\" {returned} {thunk}({declared}) noexcept {{ {body} }}");
            let owner = member.map(|(owner, member, _)| (owner.to_string(), member));
            Ok(Function { name: name.to_string(), params, result, link_name: Some(thunk), abi: "C", policy, owner, out: class_result, namespace: Vec::new(), derived: Derived::Nothing, variadic: false })
        }
    }

    /// `[FFI-17a]` — a standard-library result the wrapper makes in its own way: a view
    /// (`std::string_view` as `Span[u8]`, `std::span<const T>` as `Span[T]`, `std::span<T>` as
    /// `MutSpan[T]`) from the address and length its thunk gives, borrowing what the overlay's
    /// lifetime word names (`[FFI-11]`: a result's lifetime is never derived); a
    /// `std::optional<T>` of a `T` that maps by value as `Option[T]`. Its Ember type and mode, and
    /// the C++ type of what the thunk writes; `None` for any other type.
    fn std_result(&mut self, canonical: CXType) -> Option<Result<(CType, String), String>> {
        let template = std_template(canonical)?;
        // SAFETY(ffi): types of a live translation unit.
        unsafe {
            let spelled = cpp_spelling(canonical);
            let argument = clang_Type_getTemplateArgumentAsType(canonical, 0);
            let made = |ember: &str, pointee: (String, bool), c: &str, mode: &str| CType { ember: ember.to_string(), pointee: Some(pointee), char_pointer: false, c: c.to_string(), mode: mode.to_string(), opaque: false, function: None };
            match template.as_str() {
                "basic_string_view" => Some(if matches!(clang_getCanonicalType(argument).kind, CXType_Char_S | CXType_Char_U) {
                    Ok((made("usize", ("u8".to_string(), true), ADDRESS, "view"), "char".to_string()))
                } else {
                    Err(format!("`{spelled}`: only a view of `char` crosses, as `Span[u8]` ([FFI-17a])"))
                }),
                "span" => Some(match self.ty(argument, false) {
                    // ODR-200 — `std::span<T, N>`: a `ref [T; N]`, from the elements' address alone.
                    Ok(mapped) if let Some(extent) = span_extent(canonical) => {
                        let constant = clang_isConstQualifiedType(argument) != 0;
                        let pointer = format!("*{}{mapped}", if constant { "" } else { "mut " });
                        let mode = format!("fixed {}{extent}", if constant { "" } else { "mut " });
                        Ok((made(&pointer, (mapped, constant), &format!("{}*", cpp_spelling(argument)), &mode), cpp_spelling(argument)))
                    }
                    Ok(mapped) => {
                        let constant = clang_isConstQualifiedType(argument) != 0;
                        Ok((made("usize", (mapped, constant), ADDRESS, if constant { "view" } else { "view mut" }), cpp_spelling(argument)))
                    }
                    Err(reason) => Err(format!("`{spelled}`: its element does not map by value ({reason}) [FFI-17a]")),
                }),
                "optional" => Some(match self.ty(argument, false) {
                    Ok(mapped) => Ok((made("bool", (mapped, false), "bool", "optional"), cpp_spelling(argument))),
                    Err(reason) => Err(format!("`{spelled}`: its value does not map by value ({reason}) [FFI-17a]")),
                }),
                _ => None,
            }
        }
    }

    /// `[FFI-17a]` — a standard-library parameter the wrapper passes in its own way, zero-copy: a view
    /// (`std::string_view` as `Span[u8]`, `std::span<const T>` as `Span[T]`, `std::span<T>` as
    /// `MutSpan[T]`) as its pointer and its length, from which the thunk makes the view the callee
    /// takes; a `std::optional<T>` of a `T` that maps by value (`Option[T]`) as a pointer to its
    /// value, null for none; a `std::unique_ptr<T>` taken by value or `&&` as the object the
    /// caller's `ForeignBox[T]` gives up. `value` is the parameter's type without its reference,
    /// `ty` its type as written. `None` for any other type.
    fn std_parameter(&mut self, value: CXType, ty: CXType, name: &str) -> Result<Option<(CType, String, String)>, String> {
        // SAFETY(ffi): types and cursors of a live translation unit.
        unsafe {
            let canonical = clang_getCanonicalType(value);
            let Some(template) = std_template(canonical) else { return Ok(None) };
            // The type as its declaration names it: without the parameter's `const`.
            let spelled = cpp_spelling(clang_getCursorType(clang_getTypeDeclaration(canonical)));
            let lent = ty.kind == CXType_LValueReference && clang_isConstQualifiedType(value) == 0;
            let c = type_spelling(ty);
            let argument = clang_Type_getTemplateArgumentAsType(canonical, 0);
            let by_value = |writer: &mut Self, what: &str| {
                writer.ty(argument, false).map_err(|reason| format!("`{spelled}`: {what} does not map by value ({reason}) [FFI-17a]"))
            };
            let made = |ember: String, pointee: (String, bool), mode: &str| CType { ember, pointee: Some(pointee), char_pointer: false, c: c.clone(), mode: mode.to_string(), opaque: false, function: None };
            match template.as_str() {
                "basic_string_view" | "span" => {
                    if lent {
                        return Err(format!("`{spelled}` lent mutably (`&`): a view crosses by value or by `const&` ([FFI-17a])"));
                    }
                    let (element, mapped, constant) = if template == "basic_string_view" {
                        if !matches!(clang_getCanonicalType(argument).kind, CXType_Char_S | CXType_Char_U) {
                            return Err(format!("`{spelled}`: only a view of `char` crosses, as `Span[u8]` ([FFI-17a])"));
                        }
                        ("const char".to_string(), "u8".to_string(), true)
                    } else {
                        // ODR-200 — `std::span<T, N>`: its length is part of its type, so it is a
                        // `ref [T; N]`, passed as its elements' address.
                        if let Some(extent) = span_extent(canonical) {
                            let mapped = by_value(self, "its element")?;
                            let constant = clang_isConstQualifiedType(argument) != 0;
                            let mode = format!("fixed {}{extent} ", if constant { "" } else { "mut " });
                            let ember = made(format!("*{}{mapped}", if constant { "" } else { "mut " }), (mapped, constant), &mode);
                            return Ok(Some((ember, format!("{}* {name}", cpp_spelling(argument)), format!("{spelled}({name}, {extent})"))));
                        }
                        let mapped = by_value(self, "its element")?;
                        (cpp_spelling(argument), mapped, clang_isConstQualifiedType(argument) != 0)
                    };
                    let mode = if constant { "view " } else { "view mut " };
                    let ember = made(format!("*{}{mapped}", if constant { "" } else { "mut " }), (mapped, constant), mode);
                    Ok(Some((ember, format!("{element}* {name}, std::size_t {name}_size"), format!("{spelled}({name}, {name}_size)"))))
                }
                "optional" => {
                    if lent {
                        return Err(format!("`{spelled}` lent mutably (`&`): an optional value crosses by value or by `const&` ([FFI-17a])"));
                    }
                    let mapped = by_value(self, "its value")?;
                    let element = cpp_spelling(argument);
                    let ember = made(format!("*{mapped}"), (mapped, true), "optional ");
                    Ok(Some((ember, format!("const {element}* {name}"), format!("({name} ? {spelled}(*{name}) : {spelled}())"))))
                }
                "unique_ptr" => {
                    if ty.kind == CXType_LValueReference {
                        return Err(format!("`{spelled}` by reference: the callee takes no ownership a `ForeignBox` could give; take it by value or by `&&` ([FFI-17a])"));
                    }
                    let bridge = match self.bridge(canonical) {
                        Some(Ok(bridge)) => bridge,
                        Some(Err(reason)) => return Err(reason),
                        None => return Ok(None),
                    };
                    let inner = bridge.unique.clone().unwrap_or_default();
                    let element = cpp_spelling(argument);
                    let ember = made(format!("*mut {inner}"), (inner, false), "unique ");
                    Ok(Some((ember, format!("{element}* {name}"), format!("{spelled}({name})"))))
                }
                _ => Ok(None),
            }
        }
    }

    /// `[FFI-40]` — a C++ parameter: its Ember type and mode, the thunk's declaration of it, and the
    /// expression the thunk passes on.
    fn cpp_parameter(&mut self, ty: CXType, name: &str) -> Result<(CType, String, String), String> {
        // SAFETY(ffi): types of a live translation unit.
        unsafe {
            let c = type_spelling(ty);
            // `[FFI-17a]` — a standard-library value by reference or by value: the bridge value,
            // whose C++ object the wrapper passes on (`bridged`): a `const T&` or a `T` borrowed
            // (the callee copies it), a `T&` lent mutably, a `T&&` given up.
            let referenced = matches!(ty.kind, CXType_LValueReference | CXType_RValueReference);
            if referenced || clang_getCanonicalType(ty).kind == CXType_Record {
                let value = if referenced { clang_getPointeeType(ty) } else { ty };
                // `[FFI-17a]` (ODR-198) — a variant the overlay names: its bytes, from the enum.
                if let Some(variant) = self.variant_of(value) {
                    let constant = clang_isConstQualifiedType(value) != 0;
                    if ty.kind == CXType_LValueReference && !constant {
                        return Err(format!("`{}` lent mutably (`&`): a variant crosses by value, by `const&` or by `&&` ([FFI-17a])", cpp_spelling(value)));
                    }
                    let spelled = cpp_spelling(clang_getCanonicalType(value));
                    // A `const&`'s pointee is spelled `const …` already: g++ refuses `const const`.
                    let read_only = if constant { spelled.clone() } else { format!("const {spelled}") };
                    let pass = if ty.kind == CXType_RValueReference {
                        format!("std::move(*static_cast<{spelled}*>(const_cast<void*>({name})))")
                    } else {
                        format!("*static_cast<{read_only}*>({name})")
                    };
                    let ember = CType {
                        ember: format!("*{}", variant.storage),
                        pointee: Some((variant.storage.clone(), true)),
                        char_pointer: false,
                        c: c.clone(),
                        mode: format!("variant {} {} ", variant.ops, variant.ember),
                        opaque: false,
                        function: None,
                    };
                    return Ok((ember, format!("const void* {name}"), pass));
                }
                // `[FFI-17a]` — `E5030`: a `std::function` is never a parameter type.
                if std_template(clang_getCanonicalType(value)).as_deref() == Some("function") {
                    return Err(format!(
                        "[E5030] `{}`: a `std::function` is never a parameter type: take a function pointer and a context, or wrap the call in C++ ([FFI-17a])",
                        cpp_spelling(clang_getCanonicalType(value))
                    ));
                }
                if let Some(found) = self.std_parameter(value, ty, name)? {
                    return Ok(found);
                }
                if let Some(found) = self.bridge(clang_getCanonicalType(value)) {
                    let bridge = found?;
                    let constant = clang_isConstQualifiedType(value) != 0;
                    let (mode, pass) = match ty.kind {
                        CXType_LValueReference if !constant => ("bridged mut ", format!("*static_cast<{}*>({name})", bridge.spelled)),
                        CXType_RValueReference => ("bridged owned ", format!("std::move(*static_cast<{}*>({name}))", bridge.spelled)),
                        _ => ("bridged ", format!("*static_cast<const {}*>({name})", bridge.spelled)),
                    };
                    let ember = CType { ember: "*mut u8".to_string(), pointee: Some((bridge.ember.clone(), mode == "bridged ")), char_pointer: false, c: c.clone(), mode: mode.to_string(), opaque: false, function: None };
                    return Ok((ember, format!("void* {name}"), pass));
                }
            }
            // `[FFI-39]` — a `T*` or `T&` of a class with a trampoline: the Ember object, whose C++
            // object the wrapper passes on (`tramp`).
            if matches!(ty.kind, CXType_LValueReference | CXType_Pointer) {
                let pointee = clang_getPointeeType(ty);
                let declaration = clang_getTypeDeclaration(clang_getCanonicalType(pointee));
                let named = nested_name(declaration);
                if clang_getCanonicalType(pointee).kind == CXType_Record && self.trampoline_decls.iter().any(|decl| decl.class == named) {
                    let opaque = self.ty(pointee, true)?;
                    let constant = clang_isConstQualifiedType(pointee) != 0;
                    let pass = if ty.kind == CXType_LValueReference { format!("*{name}") } else { name.to_string() };
                    let ember = CType { ember: format!("*{}{opaque}", if constant { "" } else { "mut " }), pointee: Some((ident(&named), constant)), char_pointer: false, c: c.clone(), mode: "tramp ".to_string(), opaque: false, function: None };
                    return Ok((ember, format!("{}* {name}", cpp_spelling(pointee)), pass));
                }
            }
            let plain = |ember: String, mode: &str| CType { ember, pointee: None, char_pointer: false, c: c.clone(), mode: mode.to_string(), opaque: false, function: None };
            match ty.kind {
                CXType_LValueReference => {
                    let pointee = clang_getPointeeType(ty);
                    let constant = clang_isConstQualifiedType(pointee) != 0;
                    let by_value = self.ty(pointee, false);
                    let spelled = cpp_spelling(pointee);
                    // A class that is opaque takes no parameter mode (it is incomplete, `E5015`): its
                    // `const T&` is a `ref T`, its `T&` a `ref mut T`, which the wrapper passes on
                    // as the thunk's pointer.
                    let referenced = |inner: String, constant: bool| CType {
                        ember: format!("*{}{inner}", if constant { "" } else { "mut " }),
                        pointee: Some((inner, constant)),
                        char_pointer: false,
                        c: c.clone(),
                        mode: if constant { "ref " } else { "ref mut " }.to_string(),
                        opaque: true,
                        function: None,
                    };
                    Ok(match (constant, by_value) {
                        (true, Ok(mapped)) => (plain(mapped, ""), format!("{spelled} {name}"), name.to_string()),
                        (true, Err(_)) => (referenced(self.ty(pointee, true)?, true), format!("{spelled}* {name}"), format!("*{name}")),
                        (false, Ok(mapped)) => (plain(mapped, "mut "), format!("{spelled}* {name}"), format!("*{name}")),
                        (false, Err(_)) => (referenced(self.ty(pointee, true)?, false), format!("{spelled}* {name}"), format!("*{name}")),
                    })
                }
                CXType_RValueReference => {
                    let pointee = clang_getPointeeType(ty);
                    let spelled = cpp_spelling(pointee);
                    match self.ty(pointee, false) {
                        Ok(mapped) => Ok((plain(mapped, "owned "), format!("{spelled} {name}"), format!("std::move({name})"))),
                        // An opaque class that can be moved: the `ForeignBox` the caller gives up,
                        // whose object the callee moves from and whose drop frees what is left.
                        Err(reason) => {
                            let class = usr(clang_getTypeDeclaration(clang_getCanonicalType(pointee)));
                            if !self.by_value.get(&format!("movable {class}")).copied().unwrap_or(false) {
                                return Err(format!("an rvalue reference to a type that is neither mapped by value ({reason}) nor a class that can be moved"));
                            }
                            let inner = self.ty(pointee, true)?;
                            let ember = CType { ember: format!("*mut {inner}"), pointee: Some((inner, false)), char_pointer: false, c: c.clone(), mode: "owned ".to_string(), opaque: false, function: None };
                            Ok((ember, format!("{spelled}* {name}"), format!("std::move(*{name})")))
                        }
                    }
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
    fn instantiation(&mut self, cursor: CXCursor, written: &str, methods: &[CXCursor]) {
        let name = instance_name(written);
        // SAFETY(ffi): cursors and types of a live translation unit.
        unsafe {
            if kind(cursor) == CXCursor_TypeAliasDecl {
                let class = clang_getTypeDeclaration(clang_getCanonicalType(clang_getTypedefDeclUnderlyingType(cursor)));
                let saved = std::mem::replace(&mut self.writing, namespace_of(class));
                let record = self.record(class, Some(&name));
                self.cpp_members(class, record.clone());
                // Its methods, each named by the address the instantiation's import file takes.
                let owner = match record {
                    Record::Full(owner) | Record::Opaque(owner, None) => Some(owner),
                    _ => None,
                };
                if let Some(owner) = owner {
                    let class_spelled = cpp_spelling(clang_getCursorType(class));
                    let mutable_state = record_fields(clang_getCursorType(class)).into_iter().any(|field| clang_CXXField_isMutable(field) != 0);
                    // What the import file names no address of: the template's constructors, its
                    // operators and its overloaded methods.
                    // The template's definition, whose members are listed (`[FFI-20a]`: each one not
                    // imported is reported), not a forward declaration of it.
                    let pattern = clang_getSpecializedCursorTemplate(class);
                    let defined = clang_getCursorDefinition(pattern);
                    let pattern = if clang_Cursor_isNull(defined) == 0 { defined } else { pattern };
                    let declared: Vec<CXCursor> = children(pattern).into_iter().filter(|child| clang_getCXXAccessSpecifier(*child) == CX_CXXPublic).collect();
                    let template_name = spelling(pattern);
                    for child in &declared {
                        let member = spelling(*child);
                        // A constructor template is spelled with the class's parameters (`basic_json<ObjectType, …>`).
                        let constructor = kind(*child) == CXCursor_Constructor
                            || (kind(*child) == CXCursor_FunctionTemplate && member.split('<').next() == Some(template_name.as_str()));
                        let reason = match kind(*child) {
                            // `[FFI-17b]` — a member template is a template: only an instantiation names it.
                            CXCursor_FunctionTemplate => {
                                "[E5055] a member template of a class template's instantiation: only the explicit instantiations `instantiate=[…]` lists are imported, and no Ember code, generic or not, instantiates one ([FFI-17b])"
                            }
                            CXCursor_Constructor if clang_CXXConstructor_isCopyConstructor(*child) == 0 && clang_CXXConstructor_isMoveConstructor(*child) == 0 => {
                                "a constructor of a class template's instantiation: no C++ expression names one to take its types from"
                            }
                            CXCursor_CXXMethod
                                if declared.iter().filter(|other| matches!(kind(**other), CXCursor_CXXMethod | CXCursor_FunctionTemplate) && spelling(**other) == member).count() > 1 =>
                            {
                                "an overload of a class template's instantiation: its address names no one of them"
                            }
                            _ => continue,
                        };
                        let shown = if constructor { format!("{owner}.new") } else { format!("{owner}.{member}") };
                        self.skip(*child, &shown, reason);
                    }
                    for method in methods.iter().filter_map(|var| referenced_function(*var)) {
                        let key = spelling(method);
                        let Some(member) = self.method_member(method, &owner, mutable_state) else { continue };
                        // `[FFI-42]` — an operator: the method of its Ember operator interface.
                        let operator = if key.starts_with("operator") {
                            match operator_method(&key, clang_Cursor_getNumArguments(method), member) {
                                Ok(found) => Some(found),
                                Err(why) => {
                                    self.skip(method, &format!("{owner}.{key}"), why);
                                    continue;
                                }
                            }
                        } else {
                            None
                        };
                        let name = operator.map_or(key.clone(), |(method, _)| method.to_string());
                        match self.cpp_thunk(method, &name, Some((&owner, member, &class_spelled))) {
                            Ok(function) => {
                                if let Some((method, Some(interface))) = operator {
                                    self.operators.push((owner.clone(), method.to_string(), interface.to_string()));
                                }
                                self.functions.push(Function { namespace: self.writing.clone(), ..function });
                            }
                            Err(reason) => self.skip(method, &format!("{owner}.{key}"), reason),
                        }
                    }
                }
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
        // A static data member's definition outside its class: the class's members import it.
        // SAFETY(ffi): a live variable cursor.
        if self.cpp && matches!(kind(unsafe { clang_getCursorSemanticParent(cursor) }), CXCursor_StructDecl | CXCursor_ClassDecl | CXCursor_UnionDecl) {
            return;
        }
        if !self.seen.insert(format!("var {name}")) {
            return;
        }
        // `[FFI-3]` — a variable of a C++ header is reached through its thunk: one of C++ linkage
        // has a mangled symbol, which Ember never links, and an `inline` one has a symbol only where
        // C++ code uses it.
        // SAFETY(ffi): a live variable cursor.
        if self.cpp && unsafe { clang_getCursorLinkage(cursor) } != CXLinkage_Internal {
            self.cpp_variable(cursor, &name, None);
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

    /// `[FFI-41]`, `[FFI-3]` — a C++ variable of C++ linkage: a static data member (of the class
    /// `owner` names) or a namespace's. A `const` one with a constant value is a constant (a
    /// class's, `Outer.limit`); any other is a foreign static, read and written as a global
    /// (`Outer.count += 1`, `count`; ODR-161), whose every use goes through a thunk that gives its
    /// address, since its symbol is mangled. A member's static is the module's private
    /// `Outer_count`, which the checker gives `Outer.count` to (`ForeignImport::variables`).
    fn cpp_variable(&mut self, cursor: CXCursor, name: &str, owner: Option<&str>) {
        let shown = owner.map_or_else(|| name.to_string(), |owner| format!("{owner}.{name}"));
        // SAFETY(ffi): a live variable cursor.
        unsafe {
            let ty = clang_getCursorType(cursor);
            let constant = clang_isConstQualifiedType(ty) != 0;
            if constant && let Ok((ty, value)) = self.constant_value(cursor) {
                let key = owner.map_or_else(|| name.to_string(), |owner| format!("{owner}::{name}"));
                if self.claim(&key) {
                    self.push(&key, Kind::Const, format!("const {}: {ty} = {value}", ident(name)));
                } else {
                    self.skip(cursor, &shown, "its name is taken by another declaration of the header");
                }
                return;
            }
            if clang_getCursorTLSKind(cursor) != CXTLS_None {
                return self.skip(cursor, &shown, "a `thread_local` variable: each thread has its own");
            }
            let mapped = match self.ty(ty, false) {
                Ok(mapped) => mapped,
                Err(reason) => return self.skip(cursor, &shown, reason),
            };
            let static_name = variable_static(owner, name);
            if !self.claim(&static_name) {
                return self.skip(cursor, &shown, "its name is taken by another declaration of the header");
            }
            let thunk = format!("{}_cpp_{}_{}", ember_branding::SYMBOL_PREFIX, self.stem, self.thunks);
            self.thunks += 1;
            let _ = writeln!(self.shim, "extern \"C\" {}* {thunk}(void) noexcept {{ return &{}; }}", cpp_spelling(ty), qualified_name(cursor));
            // A `const` one is read outside `unsafe` (`@ffi(immutable)`) only when its type is `Sync`;
            // one holding a pointer is read in `unsafe`, as every other C++ variable is (the owner's
            // ruling of 2026-10-21, ODR-163).
            let (contract, mutable) = match (constant, holds_pointer(ty)) {
                (true, false) => (", immutable", ""),
                (true, true) => ("", ""),
                (false, _) => ("", "mut "),
            };
            let key = owner.map_or_else(|| name.to_string(), |owner| format!("{owner}::{name}"));
            self.push(&key, Kind::Variable, format!("@ffi(link_name=\"{thunk}\"{contract})\nstatic {mutable}{static_name}: {mapped}"));
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
                    ..Skipped::default()
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
            adopted: self.adopted,
            macro_functions: self.macro_functions,
            c_types: self.c_types,
            anonymous_namespace: self.anonymous_namespace,
            rvalue_members: self.rvalue_members,
            namespaces: self.namespaces,
            ns_uses: self.ns_uses,
            trampolines: self.trampolines,
            operators: self.operators,
            policy_changes: Vec::new(),
            declared: self.declared,
        }
    }
}

/// `[FFI-17b]` — the template a template-id names, by its qualified name (`engine::Vec`), if the
/// header declares one: a class template's cursor kind or a function template's.
/// `[FFI-17a]` (ODR-198) — a variant's key: its type as its declaration names it, without the
/// `const` a parameter's may carry.
fn variant_key(ty: CXType) -> String {
    // SAFETY(ffi): a type of a live translation unit.
    cpp_spelling(unsafe { clang_getCursorType(clang_getTypeDeclaration(clang_getCanonicalType(ty))) })
}

/// ODR-200 — the extent of a `std::span<T, N>` of fixed extent; `None` for a span of any length.
fn span_extent(canonical: CXType) -> Option<u64> {
    // SAFETY(ffi): a type of a live translation unit.
    unsafe {
        let declaration = clang_getTypeDeclaration(canonical);
        let extent = clang_Cursor_getTemplateArgumentUnsignedValue(declaration, 1);
        (clang_Cursor_getNumTemplateArguments(declaration) > 1 && extent != u64::MAX && extent != u64::from(u32::MAX)).then_some(extent)
    }
}

/// ODR-200 — a fixed span's mode (`fixed [mut ]N`) as its extent and the array a reference to it
/// names (`[T; N]`, `mut [T; N]`).
fn fixed_array(mode: &str, inner: &str) -> (String, String) {
    let rest = mode.trim().trim_start_matches("fixed ");
    let (mutable, extent) = match rest.strip_prefix("mut ") {
        Some(extent) => ("mut ", extent),
        None => ("", rest),
    };
    (extent.to_string(), format!("{mutable}[{inner}; {extent}]"))
}

/// `[FFI-48]` — the line of a C++ header that declares or imports a C++20 module (`export module
/// m;`, `module;`, `import m;`, `import <h>;`), if one does.
fn module_line(header: &Path) -> Option<String> {
    let text = std::fs::read_to_string(header).ok()?;
    text.lines().map(str::trim).find(|line| {
        let line = line.strip_prefix("export ").map_or(*line, str::trim_start);
        let declares = |word: &str| line.strip_prefix(word).is_some_and(|rest| rest.starts_with([' ', '\t', ';', '<', '"']));
        (declares("module") || declares("import")) && line.ends_with(';')
    }).map(str::to_string)
}

/// `[FFI-48]` — a template's non-type parameter of class type (C++20), by name.
fn class_type_parameter(template: CXCursor) -> Option<String> {
    // SAFETY(ffi): cursors and types of a live translation unit.
    unsafe {
        if clang_Cursor_isNull(template) != 0 {
            return None;
        }
        children(template)
            .into_iter()
            .find(|child| kind(*child) == CXCursor_NonTypeTemplateParameter && clang_getCanonicalType(clang_getCursorType(*child)).kind == CXType_Record)
            .map(|parameter| format!("{} {}", type_spelling(clang_getCursorType(parameter)), spelling(parameter)))
    }
}

fn class_parameter_reason(template: &str, parameter: &str) -> String {
    format!(
        "`{template}` has a non-type template parameter of class type (`{parameter}`): its mangling differs between compilers; instantiate it in C++ and export a typedef ([FFI-48])"
    )
}

/// `[FFI-48]` — why a C++ function is a coroutine, if it is one: its result's class declares
/// `promise_type`, or its body awaits, yields or returns as one.
fn coroutine(cursor: CXCursor) -> Option<String> {
    // SAFETY(ffi): cursors and types of a live translation unit.
    unsafe {
        let result = clang_getCanonicalType(clang_getCursorResultType(cursor));
        let declaration = clang_getTypeDeclaration(result);
        let promise = result.kind == CXType_Record
            && [clang_getCursorDefinition(declaration), clang_getSpecializedCursorTemplate(declaration)]
                .into_iter()
                .filter(|cursor| clang_Cursor_isNull(*cursor) == 0)
                .any(|class| children(class).into_iter().any(|member| spelling(member) == "promise_type"));
        let body = tokens_of(cursor).iter().any(|token| matches!(token.as_str(), "co_await" | "co_yield" | "co_return"));
        (promise || body).then(|| {
            let shown = if promise { format!("`{}` declares `promise_type`", type_spelling(clang_getCursorResultType(cursor))) } else { "its body awaits".to_string() };
            format!("[E5034] a C++ coroutine ({shown}): its promise and frame layout are implementation-defined; take a callback or a completion handle ([FFI-48])")
        })
    }
}

/// `[FFI-48]` — whether two functions take the same parameters, as C++ spells their types.
fn same_parameters(first: CXCursor, second: CXCursor) -> bool {
    // SAFETY(ffi): cursors of a live translation unit.
    unsafe {
        let count = clang_Cursor_getNumArguments(first);
        count == clang_Cursor_getNumArguments(second)
            && (0..count.max(0) as c_uint).all(|index| {
                let of = |cursor: CXCursor| type_spelling(clang_getCanonicalType(clang_getCursorType(clang_Cursor_getArgument(cursor, index))));
                of(first) == of(second)
            })
    }
}

fn result_spelling(cursor: CXCursor) -> String {
    // SAFETY(ffi): a cursor of a live translation unit.
    type_spelling(unsafe { clang_getCanonicalType(clang_getCursorResultType(cursor)) })
}

fn find_template(cursor: CXCursor, wanted: &str) -> Option<CXCursor> {
    for child in children(cursor) {
        match kind(child) {
            CXCursor_Namespace | CXCursor_LinkageSpec => {
                if let Some(found) = find_template(child, wanted) {
                    return Some(found);
                }
            }
            // A class template's definition, whose members are listed, not a forward declaration of
            // it (`nlohmann::basic_json`'s, before its definition).
            CXCursor_ClassTemplate | CXCursor_FunctionTemplate if qualified_name(child).trim_start_matches("::") == wanted => {
                // SAFETY(ffi): a cursor of a live translation unit.
                let definition = unsafe { clang_getCursorDefinition(child) };
                // SAFETY(ffi): as above.
                return Some(if unsafe { clang_Cursor_isNull(definition) } == 0 { definition } else { child });
            }
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
        if kind(child) == CXCursor_DeclRefExpr && matches!(kind(referenced), CXCursor_FunctionDecl | CXCursor_CXXMethod) {
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
    // The template's name without its namespace, and its arguments whole (`Pair<std::size_t>` is
    // `Pair_std_size_t`).
    let (template, arguments) = written.split_once('<').map_or((written, ""), |(template, arguments)| (template, arguments));
    let template = format!("{}<{arguments}", template.rsplit("::").next().unwrap_or(template));
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
/// `[FFI-41]` — the module's static for a C++ variable: its own name, or a member's `Outer_name`.
fn variable_static(owner: Option<&str>, name: &str) -> String {
    ident(&owner.map_or_else(|| name.to_string(), |owner| format!("{owner}_{name}")))
}

/// `[FFI-41]` — the C++ variables of one namespace's module, for the checker: each static, and the
/// class and member a static data member's is reached by (`Grid.made`).
fn variables(imported: &Imported, namespace: &[String]) -> Vec<(String, Option<(String, String)>)> {
    imported
        .items
        .iter()
        .filter(|item| item.kind == Kind::Variable && item.namespace == namespace)
        .map(|item| match item.name.split_once("::") {
            Some((owner, name)) => (variable_static(Some(owner), name), Some((ident(owner), ident(name)))),
            None => (variable_static(None, &item.name), None),
        })
        .collect()
}

fn namespace_of(cursor: CXCursor) -> Vec<String> {
    let mut path = Vec::new();
    // SAFETY(ffi): cursors of a live translation unit.
    let mut parent = unsafe { clang_getCursorSemanticParent(cursor) };
    while !matches!(kind(parent), CXCursor_TranslationUnit | CXCursor_InvalidFile | CXCursor_NoDeclFound | CXCursor_NotImplemented) {
        if kind(parent) == CXCursor_Namespace && !inline_namespace(parent) {
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

/// `[FFI-41]` — a C++ declaration's name, under its outer classes' names when it is nested in
/// them: `Outer::Inner` is `Outer_Inner` (Ember has no nested types).
fn nested_name(cursor: CXCursor) -> String {
    // SAFETY(ffi): cursors of a live translation unit.
    let parent = unsafe { clang_getCursorSemanticParent(cursor) };
    match kind(parent) {
        CXCursor_StructDecl | CXCursor_ClassDecl | CXCursor_UnionDecl if !spelling(parent).is_empty() => format!("{}_{}", nested_name(parent), spelling(cursor)),
        _ => spelling(cursor),
    }
}

/// `[FFI-41]` — the name a namespace's module is imported under by another namespace's.
fn namespace_alias(home: &[String]) -> String {
    format!("{}_ns_{}", ember_branding::SYMBOL_PREFIX, if home.is_empty() { "root".to_string() } else { home.join("_") })
}

/// `[FFI-3]` — a C++ type as a thunk spells it: its canonical spelling, every name qualified.
/// Clang spells an enumerator argument of a template by its enumeration alone
/// (`external_constructor<value_t::boolean>`), which names nothing at the thunks' file scope: each
/// specialization with one is spelled again, its enumerator arguments as
/// `static_cast<::ns::E>(value)`.
fn cpp_spelling(ty: CXType) -> String {
    // SAFETY(ffi): a type of a live translation unit.
    let canonical = unsafe { clang_getCanonicalType(ty) };
    let mut spelled = type_spelling(canonical);
    respell_enumerator_arguments(canonical, &mut spelled);
    spelled
}

/// `cpp_spelling`'s respelling, of `ty` (canonical) and of the types it is made of.
fn respell_enumerator_arguments(ty: CXType, spelled: &mut String) {
    // SAFETY(ffi): types and cursors of a live translation unit.
    unsafe {
        match ty.kind {
            CXType_Pointer | CXType_LValueReference | CXType_RValueReference => {
                respell_enumerator_arguments(clang_getCanonicalType(clang_getPointeeType(ty)), spelled);
            }
            CXType_ConstantArray | CXType_IncompleteArray => respell_enumerator_arguments(clang_getCanonicalType(clang_getArrayElementType(ty)), spelled),
            CXType_Record => {
                let count = clang_Type_getNumTemplateArguments(ty);
                if count <= 0 {
                    return;
                }
                let declaration = clang_getTypeDeclaration(ty);
                let template = clang_getSpecializedCursorTemplate(declaration);
                // A value argument's type is its parameter's, which libclang gives only there.
                let parameters: Vec<CXCursor> =
                    children(template).into_iter().filter(|child| matches!(kind(*child), CXCursor_TemplateTypeParameter | CXCursor_NonTypeTemplateParameter | CXCursor_TemplateTemplateParameter)).collect();
                let (mut enumerator, mut types, mut arguments) = (false, Vec::new(), Vec::new());
                for index in 0..count as c_uint {
                    let argument = clang_Type_getTemplateArgumentAsType(ty, index);
                    if argument.kind != CXType_Invalid {
                        types.push(clang_getCanonicalType(argument));
                        arguments.push(cpp_spelling(argument));
                        continue;
                    }
                    let of = parameters.get(index as usize).map_or(clang_getCursorType(clang_getNullCursor()), |parameter| clang_getCanonicalType(clang_getCursorType(*parameter)));
                    if clang_Cursor_getTemplateArgumentKind(declaration, index) != CXTemplateArgumentKind_Integral || !matches!(of.kind, CXType_Enum | CXType_Bool | CXType_Char_S | CXType_Char_U | CXType_SChar | CXType_UChar | CXType_Short | CXType_UShort | CXType_Int | CXType_UInt | CXType_Long | CXType_ULong | CXType_LongLong | CXType_ULongLong) {
                        // A declaration, a template, an expression, or a value of a type the template
                        // leaves to another argument: left as Clang spells it.
                        arguments.push(String::new());
                        continue;
                    }
                    enumerator |= of.kind == CXType_Enum;
                    let underlying = if of.kind == CXType_Enum { clang_getCanonicalType(clang_getEnumDeclIntegerType(clang_getTypeDeclaration(of))) } else { of };
                    let unsigned = matches!(underlying.kind, CXType_Bool | CXType_Char_U | CXType_UChar | CXType_UShort | CXType_UInt | CXType_ULong | CXType_ULongLong);
                    let value = if unsigned {
                        clang_Cursor_getTemplateArgumentUnsignedValue(declaration, index).to_string()
                    } else {
                        clang_Cursor_getTemplateArgumentValue(declaration, index).to_string()
                    };
                    arguments.push(format!("static_cast<::{}>({value})", type_spelling(of)));
                }
                if !enumerator || arguments.iter().any(String::is_empty) {
                    // Only its type arguments can hold a specialization to spell again.
                    for argument in types {
                        respell_enumerator_arguments(argument, spelled);
                    }
                    return;
                }
                // The specialization alone, without the `const` the canonical type may carry.
                let written = type_spelling(ty);
                let written = written.trim_start_matches("const ").trim_start_matches("volatile ");
                let again = format!("{}<{}>", qualified_name(template), arguments.join(", "));
                *spelled = spelled.replace(written, &again);
            }
            _ => {}
        }
    }
}

/// `[FFI-41]` (ODR-165) — an inline namespace (`nlohmann::json_abi_v3_11_3`, `std::__1`) adds nothing
/// to a name: C++ names its members as the enclosing namespace's, and so does the import.
fn inline_namespace(cursor: CXCursor) -> bool {
    // SAFETY(ffi): a live cursor.
    kind(cursor) == CXCursor_Namespace && unsafe { clang_Cursor_isInlineNamespace(cursor) } != 0
}

/// `[FFI-17]` — a C++ entity's fully qualified name: `::engine::Mesh::size`.
fn qualified_name(cursor: CXCursor) -> String {
    let mut parts = vec![spelling(cursor)];
    // SAFETY(ffi): cursors of a live translation unit.
    let mut parent = unsafe { clang_getCursorSemanticParent(cursor) };
    while matches!(kind(parent), CXCursor_Namespace | CXCursor_StructDecl | CXCursor_ClassDecl | CXCursor_ClassTemplate | CXCursor_UnionDecl) {
        if !inline_namespace(parent) {
            parts.push(spelling(parent));
        }
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
                            out.push((true, format!("movable {}", usr(child)), format!("__is_constructible({spelled}, {spelled}&&)")));
                            out.push((true, format!("virtual_dtor {}", usr(child)), format!("__has_virtual_destructor({spelled})")));
                            out.push((true, format!("final {}", usr(child)), format!("__is_final({spelled})")));
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
    let probed = Unit::parse(main, &probe, args, false)?;
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
/// library says how, without MSVC's `class `) and its `what()`, per thread. `[CXX-1]` — a project
/// built without RTTI (`/GR-`, `-fno-rtti`) cannot name an exception's own type: there the name is
/// the most derived standard exception class it is. With MSVC's switches only `_CPPRTTI` says so:
/// clang-cl's `/GR-` keeps `typeid` (and `__cpp_rtti`) and drops the data it reads.
fn cpp_prelude(stem: &str) -> String {
    let prefix = ember_branding::SYMBOL_PREFIX;
    format!(
        r#"#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <exception>
#include <memory>
#include <new>
#include <stdexcept>
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
#if (defined(_MSC_VER) && defined(_CPPRTTI)) || (!defined(_MSC_VER) && (defined(__cpp_rtti) || defined(__GXX_RTTI)))
    catch (const std::exception& error) {{ caught_type = type_name(typeid(error).name()); caught_what = error.what(); }}
#else
#define {prefix}_CAUGHT(T) catch (const T& error) {{ caught_type = #T; caught_what = error.what(); }}
    {prefix}_CAUGHT(std::bad_array_new_length) {prefix}_CAUGHT(std::bad_alloc)
    {prefix}_CAUGHT(std::invalid_argument) {prefix}_CAUGHT(std::domain_error) {prefix}_CAUGHT(std::length_error) {prefix}_CAUGHT(std::out_of_range) {prefix}_CAUGHT(std::logic_error)
    {prefix}_CAUGHT(std::range_error) {prefix}_CAUGHT(std::overflow_error) {prefix}_CAUGHT(std::underflow_error) {prefix}_CAUGHT(std::runtime_error)
    {prefix}_CAUGHT(std::exception)
#undef {prefix}_CAUGHT
#endif
    catch (...) {{ caught_type = "an exception that is no std::exception"; caught_what.clear(); }}
}}
template <class T> [[maybe_unused]] T zero() noexcept {{ union {{ unsigned char bytes[sizeof(T)]; T value; }} none = {{}}; return none.value; }}
[[noreturn]] [[maybe_unused]] void crossed(const char* where) noexcept {{
    caught();
    std::fprintf(stderr, "a C++ exception would cross into Ember from %s: %s: %s [FFI-39e]\n", where, caught_type.c_str(), caught_what.c_str());
    std::fflush(stderr);
    std::abort();
}}
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
    /// `[FFI-8]` — a C enum's bit-field: the enum's integer type, and a variant (the first) that
    /// `from_repr` falls back to, which it never does (a C enum holds every value of its type).
    enumerated: Option<(String, String)>,
}

/// `[FFI-8]` — a struct with bit-fields, laid out as C lays it out: its plain fields, each run of
/// bytes its bit-fields share as a byte array `_bits<k>` (and `_pad<k>` where C leaves a gap Ember
/// would not), each public with a zero default so the struct can be made; and `get_<f>`/`set_<f>`
/// for each bit-field, reading and writing its bits there (little-endian, as C's bit numbering is
/// on the targets Ember builds for). Returns the field lines, every member's offset and the
/// `extend` block.
fn bit_field_members(name: &str, plain: &[Plain], bits: &[BitField], size: u64, align: u64, packed: bool, visibility: &str) -> Result<(Vec<String>, Vec<(String, u64)>, String), String> {
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
        lines.push(format!("    {visibility}_pad{pads}: [u8; {length}] = [{}]\n", vec!["0"; length as usize].join(", ")));
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
                lines.push(format!("    {visibility}_bits{run}: [u8; {length}] = [{}]\n", vec!["0"; length as usize].join(", ")));
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
        // A C enum's: its integer, as the enum (every value is one: `from_repr` is always `Some`).
        let integer = field.enumerated.as_ref().map_or(ty.as_str(), |(repr, _)| repr.as_str());
        let value = if field.boolean {
            "value != 0".to_string()
        } else if field.signed {
            format!("(((value << {0}) as i64) >> {0}) as {integer}", 64 - width)
        } else {
            format!("value as {integer}")
        };
        let value = match &field.enumerated {
            Some((_, first)) => format!("{ty}.from_repr({value}).unwrap_or({ty}.{first})"),
            None => value,
        };
        let _ = write!(
            methods,
            "    ## The {width}-bit field `{f}`.\n    {visibility}fn get_{f}(self) -> {ty}:\n        bits: u64 = {}\n        value = (bits >> {shift}) & {mask:#x}\n        return {value}\n\n",
            read.join(" | ")
        );
        let input = if field.boolean {
            "bits: u64 = 0\n        if value:\n            bits = 1".to_string()
        } else if field.enumerated.is_some() {
            format!("bits: u64 = (value as {integer}) as u64")
        } else {
            "bits: u64 = value as u64".to_string()
        };
        let _ = write!(methods, "    ## Writes the {width}-bit field `{f}`: its low {width} bits.\n    {visibility}fn set_{f}(mut self, value: {ty}):\n        {input}\n        bits = (bits & {mask:#x}) << {shift}\n");
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
    /// `[BLD-FFI-1b]` — the first import whose flags name the runtime: its header, those flags, and
    /// what they amount to, which every other import's must agree with.
    runtime: Option<(String, Vec<&'static str>, Runtime)>,
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
    program: &[ProgramModule],
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
            match cmake_target(&base.join(&build_dir), &target, "CXX") {
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
    // `[FFI-30]` (D-879) — one header imported the same way is one module, however each importer
    // spells its path (`"stages.hpp"` beside it, `"../stages.hpp"` from a folder below): the files
    // as found, not as written, make the key. A header only an include path finds stays keyed by
    // the folder it is looked up from.
    let found = |path: PathBuf| std::fs::canonicalize(&path).ok().filter(|found| found.is_file());
    let header_key = found(from_dir.join(&header)).map_or_else(|| format!("{}|{header}", from_dir.display()), |found| found.display().to_string());
    let resolved = |written: &str| found(base.join(written)).map_or_else(|| written.to_string(), |found| found.display().to_string());
    let key = format!(
        "{header_key}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
        options.include_paths.iter().map(|dir| std::fs::canonicalize(base.join(dir)).map_or_else(|_| dir.clone(), |found| found.display().to_string())).collect::<Vec<_>>(),
        options.defines,
        options.overlays.iter().map(|(written, _)| resolved(written)).collect::<Vec<_>>(),
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
        trampolines: &[],
        variants: &[],
    };
    // `[FFI-7]` — the overlays are part of what the record is for.
    let overlay_text = options
        .overlays
        .iter()
        .map(|(written, _)| std::fs::read_to_string(base.join(written)).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n\0\n");
    // `[FFI-39]` — the classes that get a trampoline: those the program's classes derive from
    // (ODR-221), passing on the virtuals they override, and those an overlay entry still names.
    let mut trampolines = if cpp { program_trampolines(program, &from_dir, &header) } else { Vec::new() };
    for decl in if cpp { trampoline_decls(&overlay_text) } else { Vec::new() } {
        match trampolines.iter_mut().find(|found| found.class == decl.class) {
            Some(found) => {
                found.foreign |= decl.foreign;
                found.requested = None;
                for name in decl.virtuals {
                    if !found.virtuals.contains(&name) {
                        found.virtuals.push(name);
                    }
                }
                found.from_program = false;
            }
            None => trampolines.push(decl),
        }
    }
    let variants = if cpp { variant_decls(&overlay_text) } else { Vec::new() };
    let config = Config { trampolines: &trampolines, variants: &variants, ..config };
    let mut cached = false;
    let (mut imported, header_hash) = match import_cached(&config, &overlay_text, &mut cached) {
        Ok((imported, _, header_hash)) => {
            let stage = if cached { format!("import c, from its .{}", ember_branding::BINDING_EXT) } else { "import c, parsed".to_string() };
            crate::mark(&stage, &header);
            (imported, header_hash)
        }
        // `[FFI-48]` — a construct of the header the import refuses, naming its row.
        Err(message) if message.starts_with("[E5034] ") => {
            sink.emit(Diagnostic::error(codes::E5034, span, format!("cannot import `{header}`: {}", &message["[E5034] ".len()..])));
            return None;
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
    // `[CXX-1]` — one C++ class, two layouts: the two configurations disagree, and never bind it
    // twice.
    let configured = configuration(&config);
    for (name, other, its) in &imported.conflicts {
        let diagnostic = if cpp {
            Diagnostic::error(codes::E9020, span, format!("`{name}` is laid out one way by the import of `{other}` and another by this import of `{shown}`: their C++ configurations disagree"))
                .note(format!("`{other}` was parsed with {}, and `{shown}` with {}", shown_configuration(its), shown_configuration(&configured)))
                .note("one target's C++ is compiled in one configuration: two layouts of one class would be one C++ program holding both [CXX-1], [BLD-FFI-1b]")
                .help("build the C++ projects with the same defines and layout flags")
        } else {
            Diagnostic::error(codes::E5011, span, format!("`{name}` is imported from `{shown}` with a layout other than the one `{other}` gave it"))
                .help("give one import its own alias (`as name`), making its entities distinct")
                .note("two imports of one C declaration with one layout are one Ember entity [FFI-30]")
        };
        sink.emit(diagnostic);
    }
    for (usr, name, layout) in &imported.entities {
        known.shared.entry(usr.clone()).or_insert_with(|| Shared {
            module: path.clone(),
            name: name.clone(),
            layout: layout.clone(),
            binding: bound.to_string(),
            header: shown.clone(),
            cpp,
            config: configured.clone(),
        });
    }
    // `[FFI-17b]` — a template is not reported: it is imported only as its instantiations, and
    // naming it is `E5055`. The code a reason carries for the type checker is not shown here.
    // `[FFI-39]` — a class an overlay gives a trampoline that cannot have one.
    for skipped in imported.skipped.iter().filter(|skipped| trampolines.iter().any(|decl| decl.class == skipped.name)) {
        let (code, why) = match skipped.reason.split_once("] ") {
            Some(("[E5057", why)) => (codes::E5057, why),
            Some(("[E5059", why)) => (codes::E5059, why),
            Some(("[E5034", why)) => (codes::E5034, why),
            Some((_, why)) => (codes::E5031, why),
            None => (codes::E5031, skipped.reason.as_str()),
        };
        // ODR-221 — asked for by a class of the program: said where it derives.
        let requested = trampolines.iter().find(|decl| decl.class == skipped.name).and_then(|decl| decl.requested.clone());
        sink.emit(match requested {
            Some((class, at)) => Diagnostic::error(code, at, format!("`{class}` cannot derive from `{}` of `{shown}`: {why}", skipped.name))
                .note(format!("declared at {}; a C++ class an Ember class derives from gets a trampoline, a C++ subclass that passes the virtuals Ember overrides on to it [FFI-39]", skipped.at)),
            None => Diagnostic::error(code, span, format!("`{}` from `{shown}` cannot have a trampoline: {why}", skipped.name))
                .note(format!("declared at {}; its overlay entry declares `@ffi(trampoline, …)` [FFI-39]", skipped.at)),
        });
    }
    for skipped in imported.skipped.iter().filter(|skipped| !skipped.reason.starts_with("[E5055]") && !trampolines.iter().any(|decl| decl.class == skipped.name)) {
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
        let reason = opaque.reason.split_once("] ").filter(|(tag, _)| tag.starts_with("[E")).map_or(opaque.reason.as_str(), |(_, rest)| rest);
        sink.emit(
            Diagnostic::warning(codes::W5002, span, format!("`{}` is imported from `{shown}` only as an opaque type, usable behind a pointer: {reason}", opaque.name))
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
    // `[FFI-30]` (D-879) — an entry for a member of a record an earlier import wrote: that
    // import's entity, whose members it wrote, so the entry states nothing here.
    for (name, _) in applied.functions.iter() {
        let named: Vec<&Function> = imported.functions.iter().filter(|function| function.name == *name).collect();
        if let Some(member) = named.first().filter(|_| named.iter().all(|function| adopted_member(&imported, function)))
            && let Some((owner, _)) = &member.owner
            && let Some(shared) = known.shared.values().find(|shared| ident(&shared.name) == *owner || ident(shared.name.rsplit("::").next().unwrap_or(&shared.name)) == *owner)
        {
            sink.emit(
                Diagnostic::error(codes::E5010, span, format!("the overlay's entry for `{owner}.{name}` states nothing: `{owner}` is the entity the import of `{}` as `{}` wrote, members and all", shared.header, shared.module.join(".")))
                    .help(format!("state the contract in that import's overlay, or import `{shown}` the same way in both places, which makes the two one module"))
                    .note("two imports of one C++ class are one Ember entity, whose members are written once [FFI-30]"),
            );
        }
    }
    // `[FFI-40a]` (ODR-160) — a method line's receiver wins over the reading of the body.
    for function in &mut imported.functions {
        if let Some((owner, member)) = &mut function.owner
            && let Some(&mutable) = applied.receivers.get(&(owner.clone(), function.name.clone()))
        {
            *member = if mutable { Member::MutMethod } else { Member::Method };
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
    let undecided: Vec<Skipped> = imported
        .functions
        .iter()
        .filter(|function| function.policy == Policy::Unknown && applied.functions.get(&function.name).and_then(|entry| entry.policy).is_none())
        .map(|function| Skipped {
            name: function.name.clone(),
            reason: "[E5061] its exception policy cannot be derived from the header (Clang could not evaluate its `noexcept(…)`), and no overlay declares one".to_string(),
            namespace: function.namespace.clone(),
            ..Skipped::default()
        })
        .collect();
    // `[FFI-20a]` — what a module of the import (its root's, or a namespace's) did not import, by name.
    let skipped_in = |namespace: &[String]| -> Vec<(String, String)> {
        imported
            .skipped
            .iter()
            .chain(&imported.anonymous_namespace)
            .chain(&undecided)
            .filter(|skipped| skipped.namespace == namespace)
            .map(|skipped| (skipped.name.clone(), skipped.reason.clone()))
            .chain(
                imported
                    .macros
                    .iter()
                    .filter(|skipped| namespace.is_empty() && !declared(&skipped.name))
                    .map(|skipped| (skipped.name.clone(), format!("a macro, {}", skipped.reason))),
            )
            .collect()
    };
    // `[FFI-39]` — the classes of a module refused a trampoline above, with an error there.
    let refused_in = |namespace: &[String]| -> Vec<String> {
        imported
            .skipped
            .iter()
            .filter(|skipped| skipped.namespace == namespace && trampolines.iter().any(|decl| decl.class == skipped.name))
            .map(|skipped| skipped.name.clone())
            .collect()
    };
    let mut c_flags: Vec<String> = flags.to_vec();
    c_flags.extend(include_paths.iter().map(|dir| format!("-I{}", dir.display())));
    c_flags.extend(options.defines.iter().map(|define| format!("-D{define}")));
    // `[BLD-FFI-1b]` — the runtime switch, `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` the import's flags
    // name, which the target inherits and every import must agree on.
    if let Err(diagnostic) = runtime_agrees(&mut known.runtime, &c_flags, cpp, &shown, span) {
        sink.emit(diagnostic);
        return None;
    }
    let mut identities = vec![("header".to_string(), imported.header.clone())];
    identities.extend(options.overlays.iter().map(|(written, _)| ("overlay".to_string(), base.join(written))));
    let foreign = ember_typeck::ForeignImport {
        backed: Vec::new(),
        header: shown.clone(),
        identities,
        skipped: skipped_in(&[]),
        refused: refused_in(&[]),
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
        trampolines: imported
            .trampolines
            .iter()
            .filter(|found| imported.items.iter().any(|item| item.kind == Kind::Class && item.name == found.class && item.namespace.is_empty()))
            .map(|found| (ident(&found.class), found.default_init, found.unnamed_virtuals.clone()))
            .collect(),
        policy_changes: changed_signatures(&imported, &applied, "", &shown),
        variables: variables(&imported, &[]),
        declarations: declarations(&imported, &applied, ""),
        header_hash: cpp.then_some(header_hash),
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
                trampolines: imported
                    .trampolines
                    .iter()
                    .filter(|found| imported.items.iter().any(|item| item.kind == Kind::Class && item.name == found.class && item.namespace == *namespace))
                    .map(|found| (ident(&found.class), found.default_init, found.unnamed_virtuals.clone()))
                    .collect(),
                policy_changes: changed_signatures(&imported, &applied, &namespace.join("::"), &foreign.header),
                variables: variables(&imported, namespace),
                skipped: skipped_in(namespace),
                refused: refused_in(namespace),
                declarations: declarations(&imported, &applied, &namespace.join("::")),
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
/// compiled with that import's flags into objects of their own, and its libraries in order: by
/// `toolchain`, the `[c]` section's compiler. `[BLD-FFI-4]` — each `import cpp`'s thunks by
/// `backend`, the compiler of Ember's own C. `shared_library`: they go into one (a reloadable
/// image, a `cdylib`), which takes only position-independent code.
pub fn build_objects(
    modules: &[ember_typeck::LoadedModule],
    toolchain: &ember_build::Toolchain,
    backend: &ember_build::Toolchain,
    c_dir: &Path,
    obj_dir: &Path,
    profile: ember_build::Profile,
    shared_library: bool,
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
            let named = toolchain_name(backend);
            if let Some(compiler) = compiler.as_deref().filter(|compiler| *compiler != named) {
                return Err(format!("the C++ project of `{}` is compiled with `{compiler}`, and this build's C with `{named}`: one compiler builds both ([BLD-FFI-4])", foreign.header));
            }
            let thunks = c_dir.join(format!("{stem}.thunks.cpp"));
            std::fs::write(&thunks, &foreign.shim).map_err(|e| e.to_string())?;
            let object = obj_dir.join(format!("{stem}.thunks.{}", if matches!(backend, ember_build::Toolchain::Msvc { .. }) { "obj" } else { "o" }));
            ember_build::compile_cpp_object(backend, &thunks, &foreign.flags, &object, profile, standard, shared_library).map_err(|e| e.to_string())?;
            objects.push(object);
            libraries.extend(foreign.link.iter().cloned());
            if !matches!(backend, ember_build::Toolchain::Msvc { .. }) {
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
            ember_build::compile_foreign_object(toolchain, &source, &foreign.flags, &object, profile, shared_library).map_err(|e| e.to_string())?;
            objects.push(object);
        }
        libraries.extend(foreign.link.iter().cloned());
    }
    Ok((objects, libraries))
}

/// `[BLD-FFI-4]` — the link-time optimisation flag a C++ project's GCC or Clang flags give, the last
/// one written (`-fno-lto` turning it off), which the program's own C inherits.
pub fn inherited_lto(modules: &[ember_typeck::LoadedModule]) -> Option<&'static str> {
    const KNOWN: [&str; 4] = ["-flto", "-flto=thin", "-flto=full", "-flto=auto"];
    modules
        .iter()
        .filter_map(|module| module.foreign.as_ref().filter(|foreign| foreign.cpp.is_some()))
        .find_map(|foreign| {
            let last = foreign.flags.iter().rev().find(|flag| flag.starts_with("-flto") || *flag == "-fno-lto")?;
            Some(KNOWN.into_iter().find(|known| known == last).unwrap_or(if last == "-fno-lto" { "" } else { "-flto" }))
        })
        .filter(|flag| !flag.is_empty())
}

/// `[BLD-FFI-1b]` — the MSVC runtime-library switch, `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` (and
/// libstdc++'s debug mode, `_GLIBCXX_DEBUG`, its debug runtime under GCC and Clang) the build
/// inherits, byte for byte, from the projects of its imports (a C++ project's flags and defines, or
/// the `[c]` flags a C import is compiled with); each import agreed with the others when it was
/// loaded (`runtime_agrees`).
pub fn inherited_runtime(modules: &[ember_typeck::LoadedModule], target: Option<(&str, &[String])>) -> Result<Vec<&'static str>, Diagnostic> {
    let mut first = None;
    // `[BLD-FFI-2]` — the CMake target whose flags the build takes, first.
    if let Some((name, flags)) = target {
        runtime_agrees(&mut first, flags, false, &format!("CMake target `{name}`"), Span::DUMMY)?;
    }
    for module in modules {
        let Some(foreign) = module.foreign.as_ref().filter(|foreign| !foreign.shim.is_empty()) else { continue };
        runtime_agrees(&mut first, &foreign.flags, foreign.cpp.is_some(), &foreign.header, Span::DUMMY)?;
    }
    Ok(first.map(|(_, flags, _)| flags).unwrap_or_default())
}

/// `[BLD-FFI-1b]` — whether an import's flags agree on the runtime with the first import's that
/// named it (`first`, which the first to name it becomes): every import that names any of the
/// runtime switch, `_DEBUG`, `_ITERATOR_DEBUG_LEVEL` and `_GLIBCXX_DEBUG` must name the same, the
/// implied ones included (`/MDd` defines `_DEBUG`, which makes the iterator debug level 2), or the
/// target's translation units would disagree (`E9020`, `[CXX-1]`: two configurations never bind
/// one standard library two ways). A C++ project whose flags pass a response file, which may name
/// them, leaves its runtime undetermined (`E9021`).
fn runtime_agrees(first: &mut Option<(String, Vec<&'static str>, Runtime)>, flags: &[String], cpp: bool, header: &str, span: Span) -> Result<(), Diagnostic> {
    if cpp && let Some(file) = flags.iter().find(|flag| flag.starts_with('@')) {
        return Err(Diagnostic::error(codes::E9021, span, format!("the C++ runtime of `{header}` could not be determined: its project's flags pass the response file `{}`", &file[1..]))
            .note("the MSVC runtime switch, `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` change the C++ standard library's layout and heap, and the build inherits them byte for byte [BLD-FFI-1b]")
            .help("write the project's runtime switch (`/MD`, `/MDd`, `/MT` or `/MTd`) and those defines in its `[cpp.<project>]` flags"));
    }
    let (named, meaning) = runtime_flags(flags);
    if named.is_empty() {
        return Ok(());
    }
    match first {
        None => *first = Some((header.to_string(), named, meaning)),
        Some((before, written, was)) if *was != meaning => {
            return Err(Diagnostic::error(codes::E9020, span, format!("the imports of `{before}` and `{header}` disagree on the C++ runtime"))
                .note(format!("`{before}` is compiled with `{}`, and `{header}` with `{}`", written.join(" "), named.join(" ")))
                .note("one target's translation units are compiled with one runtime switch, `_DEBUG`, `_ITERATOR_DEBUG_LEVEL` and `_GLIBCXX_DEBUG`, which change the C++ standard library's layout [BLD-FFI-1b]")
                .help("build the C++ projects with one runtime, and name it in each one's flags"));
        }
        Some(_) => {}
    }
    Ok(())
}

/// `[BLD-FFI-1b]` — the runtime a project's flags amount to: the MSVC switch, whether `_DEBUG` is
/// defined, the iterator debug level, and whether libstdc++'s debug mode is on.
type Runtime = (&'static str, bool, u8, bool);

/// `[BLD-FFI-1b]` — what of the runtime a list of flags names: the flags as the build passes them on
/// (`/MDd`, `/D_DEBUG`, `/D_ITERATOR_DEBUG_LEVEL=1`, `-D_GLIBCXX_DEBUG`; `-` forms read too, the
/// last switch winning as it does for cl), and what they amount to (`Runtime`).
fn runtime_flags(flags: &[String]) -> (Vec<&'static str>, Runtime) {
    const SWITCHES: [&str; 4] = ["/MD", "/MDd", "/MT", "/MTd"];
    const LEVELS: [&str; 3] = ["/D_ITERATOR_DEBUG_LEVEL=0", "/D_ITERATOR_DEBUG_LEVEL=1", "/D_ITERATOR_DEBUG_LEVEL=2"];
    let plain = |flag: &String| flag.replacen('-', "/", 1);
    let switch = flags.iter().rev().map(plain).find_map(|flag| SWITCHES.into_iter().find(|known| *known == flag));
    let debug = flags.iter().map(plain).any(|flag| flag == "/D_DEBUG" || flag == "/D_DEBUG=1");
    let level = flags.iter().rev().map(plain).find_map(|flag| LEVELS.into_iter().find(|known| *known == flag));
    let mut named: Vec<&'static str> = switch.into_iter().collect();
    if debug {
        named.push("/D_DEBUG");
    }
    named.extend(level);
    let shown = switch.unwrap_or("/MD");
    let debugging = debug || shown.ends_with('d');
    let iterators = level.and_then(|flag| flag.rsplit('=').next()?.parse().ok()).unwrap_or(if debugging { 2 } else { 0 });
    // libstdc++'s debug mode: its containers are other classes, laid out otherwise. `-U` names the
    // release one; the last of the two written wins, as for the compiler.
    let gnu = flags.iter().rev().map(plain).find_map(|flag| match flag.as_str() {
        "/D_GLIBCXX_DEBUG" | "/D_GLIBCXX_DEBUG=1" => Some(true),
        "/U_GLIBCXX_DEBUG" => Some(false),
        _ => None,
    });
    named.extend(gnu.map(|debug| if debug { "-D_GLIBCXX_DEBUG" } else { "-U_GLIBCXX_DEBUG" }));
    (named, (shown, debugging, iterators, gnu == Some(true)))
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
fn cmake_target(build_dir: &Path, target: &str, language: &str) -> Result<CmakeTarget, String> {
    let (reply, index) = cmake_reply(build_dir)?;
    let codemodel = index["reply"]["codemodel-v2"]["jsonFile"].as_str().ok_or("the File API reply has no `codemodel-v2`")?;
    let codemodel = read_json(&reply.join(codemodel))?;
    let target_file = codemodel["configurations"][0]["targets"]
        .as_array()
        .and_then(|targets| targets.iter().find(|entry| entry["name"].as_str() == Some(target)))
        .and_then(|entry| entry["jsonFile"].as_str())
        .ok_or_else(|| format!("the CMake build has no target `{target}`"))?;
    let target_reply = read_json(&reply.join(target_file))?;
    let group = target_reply["compileGroups"]
        .as_array()
        .and_then(|groups| groups.iter().find(|group| group["language"].as_str() == Some(language)))
        .ok_or_else(|| format!("CMake compiles no {} for target `{target}`", if language == "CXX" { "C++" } else { language }))?;
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

/// A JSON file of the File API's reply.
fn read_json(path: &Path) -> Result<serde_json::Value, String> {
    let text = std::fs::read_to_string(path).map_err(|error| format!("cannot read `{}`: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("`{}` is not JSON: {error}", path.display()))
}

/// `[FFI-50]`, `[BLD-FFI-2]` — the File API reply folder of a CMake build, and its newest index.
/// With no reply yet, the queries that ask for one (the code model and the toolchains) are written,
/// and the error says to configure again.
fn cmake_reply(build_dir: &Path) -> Result<(PathBuf, serde_json::Value), String> {
    let api = build_dir.join(".cmake").join("api").join("v1");
    let reply = api.join("reply");
    let mut indexes: Vec<PathBuf> = std::fs::read_dir(&reply)
        .map(|entries| entries.filter_map(Result::ok).map(|entry| entry.path()).filter(|path| path.file_name().is_some_and(|name| name.to_string_lossy().starts_with("index-"))).collect())
        .unwrap_or_default();
    indexes.sort();
    let Some(index) = indexes.last() else {
        let query = api.join("query");
        let _ = std::fs::create_dir_all(&query).and_then(|()| std::fs::write(query.join("codemodel-v2"), "")).and_then(|()| std::fs::write(query.join("toolchains-v1"), ""));
        return Err(format!("`{}` has no CMake File API reply: the query for one is written there now, so configure it with CMake again", build_dir.display()));
    };
    let index = read_json(index)?;
    Ok((reply, index))
}

/// `[BLD-FFI-2]` — `--cc-flags-from-target <target>`: the C compile of a CMake target, as the File
/// API reply in the CMake build folder `build_dir` gives it: its defines, include folders (made
/// absolute) and other flags, in the forms cl, clang and gcc all read (`-D`, `-I`).
pub fn cmake_c_flags(build_dir: &Path, target: &str) -> Result<Vec<String>, String> {
    let found = cmake_target(build_dir, target, "C")?;
    Ok(found.defines.iter().map(|define| format!("-D{define}")).chain(found.includes.iter().map(|dir| format!("-I{dir}"))).chain(found.flags).collect())
}

/// `[BLD-FFI-2]` — the C compiler of a CMake build, by the name `--cc` gives it, from the File API's
/// toolchains reply: MSVC's `cl` is `msvc`, Clang with MSVC's driver `clang-cl`, other Clang
/// `clang`, GCC `gcc`. `None` when the reply does not say.
pub fn cmake_c_compiler(build_dir: &Path) -> Option<&'static str> {
    let (reply, index) = cmake_reply(build_dir).ok()?;
    let toolchains = read_json(&reply.join(index["reply"]["toolchains-v1"]["jsonFile"].as_str()?)).ok()?;
    let compiler = &toolchains["toolchains"].as_array()?.iter().find(|toolchain| toolchain["language"].as_str() == Some("C"))?["compiler"];
    let stem = Path::new(compiler["path"].as_str().unwrap_or_default()).file_stem().map(|stem| stem.to_string_lossy().to_lowercase()).unwrap_or_default();
    match compiler["id"].as_str()? {
        "MSVC" => Some("msvc"),
        "Clang" | "AppleClang" if stem.starts_with("clang-cl") => Some("clang-cl"),
        "Clang" | "AppleClang" => Some("clang"),
        "GNU" => Some("gcc"),
        _ => None,
    }
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

/// `[DIA-5]` — the declarations of one namespace's module (`""` for the import's own), each by the
/// names the module binds it to (its C name, and the name an overlay gives it), with its text as
/// the header writes it and where.
fn declarations(imported: &Imported, applied: &Applied, namespace: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for (_, name, text, at) in imported.declared.iter().filter(|(written_in, ..)| written_in == namespace) {
        out.push((name.clone(), text.clone(), at.clone()));
        if let Some(renamed) = applied.renamed.get(name) {
            out.push((renamed.clone(), text.clone(), at.clone()));
        }
    }
    out
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
    /// `[FFI-40a]` (ODR-160) — each method a `class` section restates, by class and name: whether
    /// its line takes `mut self`.
    receivers: HashMap<(String, String), bool>,
}

/// An overlay's function entry: each parameter's contract words, the result's, and the entry's own
/// `@ffi(…)` arguments, each as written.
struct Entry {
    params: Vec<Vec<String>>,
    result: Vec<String>,
    arguments: Vec<String>,
    /// `[FFI-17a]` (ODR-199) — `@ffi(std_function)`: the function's `std::function` result is an
    /// Ember `owned fn(A…) -> R`.
    std_function: bool,
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
const C_ALIASES: [(&str, &str); 15] = [
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
    ("c_longdouble", "long double"),
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
    // A `class` section's method lines come last: a line restates the method as the import gives
    // it, which an entry for the method changes (`[FFI-11]`).
    let class = |entry: &&ast::OverlayItem| matches!(entry.kind, ast::OverlayItemKind::Class { .. });
    for entry in overlay.items.iter().filter(|entry| !class(entry)).chain(overlay.items.iter().filter(class)) {
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
                if !matched {
                    continue;
                }
                // `[FFI-24b]`, `[FFI-43]` — the exception policy an entry declares, never one it
                // declares twice over, nor one an earlier overlay's entry contradicts (`E5062`).
                let (declared, arguments): (Vec<String>, Vec<String>) = arguments().into_iter().partition(|written| {
                    let written: String = written.chars().filter(|c| !c.is_whitespace()).collect();
                    matches!(written.as_str(), "noexcept" | "noexcept=true" | "noexcept=false")
                });
                // ODR-199 — `@ffi(std_function)`, on a function whose result is a `std::function`.
                let (std_function, arguments): (Vec<String>, Vec<String>) = arguments.into_iter().partition(|written| written.trim() == "std_function");
                let std_function = !std_function.is_empty();
                if std_function && !function.result.as_ref().is_some_and(|ty| ty.mode.starts_with("function ")) {
                    mismatch(sink, name.span, format!("`{}` returns no `std::function` (`@ffi(std_function)` makes one an Ember callable)", name.name));
                    continue;
                }
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
                if !entry.is_override
                    && let Some(earlier) = applied.functions.get(&function.name).and_then(|earlier| earlier.policy)
                    && policy.is_some_and(|policy| policy != earlier)
                {
                    let said = |policy: Policy| if policy == Policy::Asserted { "`noexcept`" } else { "throwing" };
                    sink.emit(
                        Diagnostic::error(codes::E5062, name.span, format!("the overlays disagree on `{}`: an earlier one declares it {}, this one {}", name.name, said(earlier), said(policy.unwrap_or(earlier))))
                            .help("begin this entry with `override` to replace the earlier one")
                            .note("a C++ function has one exception policy, whichever overlays compose [FFI-43]"),
                    );
                    continue;
                }
                // `[FFI-17a]`, `[FFI-11]` — a view's lifetime is never derived: the entry names what it
                // borrows.
                // An entry that leaves a fact unknown (`TODO(…)`, `unsafe`) asserts nothing yet.
                let unknown = params.iter().flat_map(|param| &param.contract).chain(result).any(|written| matches!(word(&snippet(written.span)), "TODO" | "unsafe"));
                let lifetime = unknown || result.iter().any(|written| word(&snippet(written.span)) == "from");
                if function.result.as_ref().is_some_and(|ty| ty.mode.starts_with("view") || ty.mode.starts_with("fixed ")) && !lifetime {
                    sink.emit(
                        Diagnostic::error(codes::E5002, name.span, format!("`{}` returns a view (a `std::string_view` or a `std::span`), whose lifetime only its entry states", name.name))
                            .help(format!("write what the view borrows: `fn {}(…) -> from(self)`, `from(<parameter>)` or `from(static)`", name.name))
                            .note("a result's lifetime is never derived from a header [FFI-11], [FFI-17a]"),
                    );
                    continue;
                }
                if !take(applied, sink, layer, entry.is_override, "fn", name) {
                    continue;
                }
                applied.functions.insert(
                    function.name.clone(),
                    Entry {
                        params: params.iter().map(|param| param.contract.iter().map(|word| snippet(word.span)).collect()).collect(),
                        result: result.iter().map(|word| snippet(word.span)).collect(),
                        arguments,
                        std_function,
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
                let c_type = |ty: &ast::TypeExpr, c: &str| CType { ember: snippet(ty.span), pointee: None, char_pointer: false, c: c.to_string(), mode: String::new(), opaque: false, function: None };
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
                    derived: Derived::Nothing,
                    variadic: false,
                };
                // A raw pointer states nothing a call can rely on: the call stays unsafe (`[FFI-2]`).
                let raw = params.iter().map(|(_, ty)| ty).chain(result).any(|ty| matches!(ty.kind, ast::TypeKind::Ptr { .. }));
                let made = Entry {
                    params: vec![Vec::new(); params.len()],
                    result: if raw { vec!["unsafe".to_string()] } else { Vec::new() },
                    arguments: arguments(),
                    std_function: false,
                    policy: None,
                };
                match applied.macros.iter_mut().find(|(earlier, ..)| earlier.name == found.name) {
                    Some(earlier) => *earlier = (function, made, c_text),
                    None => applied.macros.push((function, made, c_text)),
                }
            }
            // `[FFI-39]` — `@ffi(trampoline, virtuals=[…], owner=…) type Name`: the importer made
            // the class's trampoline (it read the entry before the header); a class it could not
            // make one for was reported there.
            ast::OverlayItemKind::Type(name) if arguments().iter().any(|written| written == "trampoline") => {
                let made = imported.trampolines.iter().any(|found| found.class == name.name.as_str());
                let refused = imported.skipped.iter().any(|skipped| skipped.name == name.name.as_str() && skipped.reason.starts_with("[E50"));
                if !made && !refused {
                    mismatch(sink, name.span, format!("`{header}` declares no class `{}` that an Ember class could derive from", name.name));
                    continue;
                }
                for written in arguments() {
                    let key = written.split('=').next().unwrap_or("").trim();
                    if !matches!(key, "trampoline" | "virtuals" | "owner") {
                        mismatch(sink, name.span, format!("`@ffi({written})` is no argument of a trampoline entry: it takes `virtuals=[…]` and `owner=\"ember\" | \"foreign\"`"));
                    }
                    if key == "owner" && !matches!(written.split('=').nth(1).map(|owner| owner.trim().trim_matches('"')), Some("ember" | "foreign")) {
                        mismatch(sink, name.span, format!("`{written}`: a trampoline's object is owned by `\"ember\"` (the default) or `\"foreign\"` [FFI-17d]"));
                    }
                }
                let _ = take(applied, sink, layer, entry.is_override, "type", name);
            }
            // `[FFI-17a]` (ODR-198) — `@ffi(variant=[…]) type Name`: the importer made the alias's
            // enum (it read the entry before the header), or said why it could not.
            ast::OverlayItemKind::Type(name) if arguments().iter().any(|written| written.trim_start().starts_with("variant")) => {
                let made = imported.items.iter().any(|item| item.name == name.name.as_str() && item.kind == Kind::Type && item.text.contains(&format!("enum {}:", ident(name.name.as_str()))));
                if !made {
                    let why = imported.skipped.iter().find(|skipped| skipped.name == name.name.as_str()).map(|skipped| {
                        skipped.reason.split_once("] ").filter(|(tag, _)| tag.starts_with("[E")).map_or(skipped.reason.clone(), |(_, rest)| rest.to_string())
                    });
                    mismatch(sink, name.span, why.unwrap_or_else(|| format!("`{header}` declares no alias `{}` of a `std::variant`", name.name)));
                    continue;
                }
                let _ = take(applied, sink, layer, entry.is_override, "type", name);
            }
            // `[FFI-8]` (ODR-249) — `@ffi(newtype) type Name`: the header's `typedef` made a nominal
            // type, its alias written with the attribute.
            ast::OverlayItemKind::Type(name) if arguments().iter().any(|written| written.trim() == "newtype") => {
                let alias = format!("type {} = ", ident(name.name.as_str()));
                if !imported.items.iter().any(|item| item.name == name.name.as_str() && item.kind == Kind::Type && item.text.starts_with(&alias)) {
                    mismatch(sink, name.span, format!("`{header}` declares no `typedef` `{}` that `@ffi(newtype)` could make nominal", name.name));
                    continue;
                }
                if let Some(other) = arguments().iter().find(|written| written.trim() != "newtype") {
                    mismatch(sink, name.span, format!("`@ffi({other})` is no argument of a newtype entry: it takes `newtype` alone"));
                    continue;
                }
                if take(applied, sink, layer, entry.is_override, "type", name) {
                    applied.attributes.insert(name.name.to_string(), attributes());
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
            // `[FFI-40a]` (ODR-160) — a method line: the method's signature as the import gives it,
            // with the receiver, `self` or `mut self`, asserted over the importer's reading of its
            // body.
            ast::OverlayItemKind::Class { name, methods } => {
                for method in methods {
                    let shown = format!("{}.{}", name.name, method.name.name);
                    let found = imported.functions.iter().find(|function| {
                        function.name == method.name.name.as_str()
                            && function.owner.as_ref().is_some_and(|(owner, member)| owner == name.name.as_str() && matches!(member, Member::Method | Member::MutMethod))
                    });
                    let Some(function) = found else {
                        mismatch(sink, method.name.span, format!("`{header}` declares no method `{shown}`"));
                        continue;
                    };
                    let (mutable, params) = match method.params.split_first() {
                        Some((first, rest)) if matches!(first.kind, ast::ParamKind::Receiver { ty: None }) && matches!(first.mode, ast::Mode::Borrow | ast::Mode::Mut) => {
                            (first.mode == ast::Mode::Mut, rest)
                        }
                        _ => {
                            mismatch(sink, method.name.span, format!("the line for `{shown}` takes `self` or `mut self` first"));
                            continue;
                        }
                    };
                    let tight = |written: &str| written.split_whitespace().collect::<String>();
                    // The method as its wrapper gives it: contracted when an entry states every fact.
                    let stated = applied.functions.get(&function.name);
                    let policy = match (function.policy, stated.and_then(|stated| stated.policy)) {
                        (Policy::Direct, _) => Policy::Direct,
                        (_, Some(declared)) => declared,
                        (derived, None) => derived,
                    };
                    let (declared, value) = signature_of(&render_wrapper(function, stated, None, false, policy, None, &|_| None).1).unwrap_or_default();
                    let declared: Vec<String> = declared.into_iter().skip(1).collect();
                    let written: Vec<String> = params.iter().map(|param| tight(&snippet(param.span))).collect();
                    let result = method.ret.as_ref().map(|ty| tight(&snippet(ty.span)));
                    let result_matches = match (&value, &result) {
                        (None, None) => true,
                        (Some(value), Some(result)) => tight(value) == *result,
                        _ => false,
                    };
                    if written != declared.iter().map(|param| tight(param)).collect::<Vec<_>>() || !result_matches || method.is_unsafe || !method.generics.is_empty() {
                        let arrow = value.map_or(String::new(), |value| format!(" -> {value}"));
                        let receiver = if mutable { "mut self" } else { "self" };
                        let line = std::iter::once(receiver.to_string()).chain(declared).collect::<Vec<_>>().join(", ");
                        mismatch(sink, method.name.span, format!("`{shown}` is `fn {}({line}){arrow}` in `{header}`", method.name.name));
                        continue;
                    }
                    let key = ast::Ident { name: shown.as_str().into(), span: method.name.span };
                    if take(applied, sink, layer, entry.is_override, "method", &key) {
                        applied.receivers.insert((name.name.to_string(), method.name.name.to_string()), mutable);
                    }
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

/// `[FFI-30]` (D-879) — whether `function` is a member (a method, a constructor, an operator) of a
/// record an earlier import of the build wrote, whose members that import wrote once: written again
/// here, each would be the entity's a second time.
fn adopted_member(imported: &Imported, function: &Function) -> bool {
    function.owner.as_ref().is_some_and(|(owner, _)| {
        let key = format!("{}|{owner}", function.namespace.join("::"));
        imported.adopted.iter().any(|adopted| *adopted == key)
    })
}

/// `[GRM-35]` — an overlay writes a slot's words in any order; a hand declaration's `param(…)` and
/// `result(…)` read the ownership word first, then the count word, then the rest.
fn canonical(mut words: Vec<String>) -> Vec<String> {
    words.sort_by_key(|written| match word(written) {
        "borrowed" | "owned" | "retained" | "returns_owned" | "out" | "inout_count" => 0,
        "one" | "count" | "fixed" | "nul_terminated" => 1,
        _ => 2,
    });
    words
}

/// `[FFI-11]` — the words a declaration's `param(p, …)` gives a parameter: the entry's (`unsafe`
/// states no fact), in a hand declaration's order. `[FFI-40]` — a C++ `T&` of a type by value
/// (`mut p: T`) is what the header says when the entry writes nothing: one `T`, borrowed
/// exclusively for the call. `[FFI-2a]` — a `T*` is a mutable borrow, which Ember's is when
/// exclusive.
fn slot_words(ty: &CType, written: &[String]) -> Vec<String> {
    let mut words: Vec<String> = written.iter().filter(|written| word(written) != "unsafe").cloned().collect();
    if words.is_empty() && ty.mode == "mut " && ty.pointee.is_none() {
        words = ["borrowed", "one", "exclusive"].map(str::to_string).to_vec();
    }
    let has = |wanted: &str| words.iter().any(|written| word(written) == wanted);
    if ty.pointee.as_ref().is_some_and(|(_, constant)| !constant) && has("borrowed") && !has("exclusive") && !has("aliased") {
        words.push("exclusive".to_string());
    }
    canonical(words)
}

/// `[FFI-11]` — the Ember type a contracted pointer parameter is, as a hand declaration writes it:
/// `T*` is a mutable borrow and `const T*` a shared one (`[FFI-2a]`, derived, never written).
fn safe_parameter(name: &str, ty: &CType, words: &[String]) -> String {
    let has = |wanted: &str| words.iter().any(|written| word(written) == wanted);
    let raw = format!("{name}: {}", ty.ember);
    // `[FFI-21]` — a callback is declared as its function pointer, given (`Option` of one with
    // `nullable`); the contract makes it the callable a caller passes.
    if has("callback") && let Some(function) = &ty.function {
        return if has("nullable") { format!("{name}: Option[{function}]") } else { format!("{name}: {function}") };
    }
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
        // `[TYP-22]` — an incomplete type takes no parameter mode: its exclusive borrow is a
        // `ref mut`.
        if mutable && !nullable && !ty.opaque {
            return format!("mut {name}: {inner}");
        }
        format!("ref {}{inner}", if mutable { "mut " } else { "" })
    } else {
        return raw;
    };
    format!("{name}: {}", optional(base))
}

/// `[FFI-21]` — the user data a `callback(…, user_data=q)` word names.
fn callback_user_data(written: &str) -> Option<String> {
    let inner = written.trim().strip_prefix("callback(")?;
    let (_, rest) = inner.split_once("user_data")?;
    let name = rest.trim_start().strip_prefix('=')?.trim_start();
    Some(name.split(|c: char| !(c.is_alphanumeric() || c == '_')).next()?.to_string())
}

/// `[FFI-21]` (ODR-245) — with `user_data=q@N`, the callback's parameter `N` (1-based) that
/// receives the user data.
fn callback_user_data_place(written: &str) -> Option<usize> {
    let inner = written.trim().strip_prefix("callback(")?;
    let (_, rest) = inner.split_once("user_data")?;
    let (_, place) = rest.split_once('@')?;
    place.trim_start().split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()
}

/// `[FFI-21]` — the callable a callback with user data is to its caller: the C function pointer's
/// type (`extern "C" fn(*mut void, c_int) -> R`) without the parameter that receives the user data
/// (its `place`, 1-based, or else its one parameter of the user data's type), a `once fn` for a
/// `once` callback.
fn callable_of(function: &str, data: &str, place: Option<usize>, once: bool) -> Option<String> {
    let (_, rest) = function.split_once("fn(")?;
    let (inside, after) = rest.rsplit_once(')')?;
    let (mut parts, mut depth, mut start) = (Vec::new(), 0, 0);
    for (at, c) in inside.char_indices() {
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(inside[start..at].trim().to_string());
                start = at + 1;
            }
            _ => {}
        }
    }
    if !inside.trim().is_empty() {
        parts.push(inside[start..].trim().to_string());
    }
    let slot = match place {
        Some(place) => place.checked_sub(1).filter(|&slot| parts.get(slot).is_some_and(|part| part == data))?,
        None => parts.iter().position(|part| part == data)?,
    };
    parts.remove(slot);
    Some(format!("{}fn({}){after}", if once { "once " } else { "" }, parts.join(", ")))
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

const VARIADIC: &str = "C reads each further argument as the type its format or count names: the caller matches them ([FFI-8])";

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
        // `[FFI-11c]` — a `TODO(…)` or `unsafe` word leaves the call unsafe; `unsafe` says so on
        // purpose (`[FFI-11]`), which the declaration keeps.
        let left_unsafe = entry.params.iter().flatten().chain(&entry.result).any(|written| word(written) == "unsafe");
        safe = !words.into_iter().any(|written| matches!(word(written), "TODO" | "unsafe"));
        if left_unsafe {
            arguments.push("unsafe".to_string());
        }
        for ((param, ty), written) in function.params.iter().zip(&entry.params) {
            let words = slot_words(ty, written);
            if !words.is_empty() {
                arguments.push(format!("param({param}, {})", words.join(", ")));
            }
        }
        let result_words = canonical(entry.result.iter().filter(|written| word(written) != "unsafe").cloned().collect());
        if !result_words.is_empty() {
            arguments.push(format!("result({})", result_words.join(", ")));
        }
        arguments.extend(entry.arguments.iter().cloned());
        if safe {
            params = function.params.iter().zip(&entry.params).map(|((param, ty), words)| safe_parameter(&format!("{}{param}", ty.mode), ty, words)).collect();
            result = function.result.as_ref().map(|ty| safe_result(ty, &entry.result));
        }
    }
    // `[FFI-1]` — the effects the header's declaration states, unless the entry states its own.
    let stated = |key: &str| entry.is_some_and(|entry| entry.arguments.iter().any(|argument| argument.trim_start().starts_with(key)));
    if function.derived != Derived::Nothing && !stated("effects") {
        arguments.push("effects=[]".to_string());
    }
    if function.derived == Derived::Const && !stated("deterministic") {
        arguments.push("deterministic".to_string());
    }
    if let Some(link) = link {
        arguments.push(format!("link_name=\"{link}\""));
    }
    // `[FFI-8]` — never safe: C reads its further arguments by what its format or count names. Its
    // entry's contract still types the parameters C is passed unchanged (`cstr`, `ref T`).
    let safety = if function.variadic {
        arguments.insert(0, "variadic".to_string());
        safe = false;
        VARIADIC
    } else {
        UNCONTRACTED
    };
    let mut text = String::new();
    match function.derived {
        Derived::Const => text.push_str("    # derived: `__attribute__((const))`, so no effects and the same result for the same arguments [FFI-1]
"),
        Derived::Pure => text.push_str("    # derived: `__attribute__((pure))`, so no effects: it only reads memory [FFI-1]
"),
        Derived::Nothing => {}
    }
    if !arguments.is_empty() {
        let _ = writeln!(text, "    @ffi({})", arguments.join(", "));
    }
    if !safe {
        let _ = writeln!(text, "    @safety(\"{safety}\")");
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
            // `pub` goes on the declaration, after its comments and attributes.
            if first && !line.starts_with('@') && !line.starts_with('#') {
                let _ = writeln!(out, "{indent}{public}{line}");
                first = false;
            } else {
                let _ = writeln!(out, "{indent}{line}");
            }
        }
        out
    };
    let alias = |item: &Item| applied.renamed.get(&item.name).map(|to| format!("pub type {} = {}\n", ident(to), ident(&item.name)));
    let in_class = |item: &Item| item.kind == Kind::Const && item.name.contains("::");
    for item in imported.items.iter().filter(|item| matches!(item.kind, Kind::Type | Kind::Const | Kind::Class) && !in_class(item) && item.namespace == namespace) {
        // `[FFI-39]` — a trampoline's class is written whole: its parts say what is public.
        if item.kind == Kind::Class {
            body.push_str(&item.text);
            body.push_str("\n\n");
            continue;
        }
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
    for item in imported.items.iter().filter(|item| matches!(item.kind, Kind::Opaque | Kind::Static | Kind::Variable) && item.namespace == namespace) {
        // `[FFI-41]` — a static data member's static is the module's own: `Outer.name` reaches it.
        if item.kind == Kind::Variable && item.name.contains("::") {
            member("C", item.text.lines().map(|line| format!("    {line}
")).collect());
            continue;
        }
        member("C", declare(item, "    "));
        if let Some(alias) = alias(item) {
            body.push_str(&alias);
        }
    }
    // A hidden declaration stays, private to the module: another may name it (`destructor=f`).
    let mut wrappers = String::new();
    let mut extensions: Vec<(String, String)> = Vec::new();
    // `[FFI-41]` — a static data member that is a constant: its class's (`Outer.limit`).
    for item in imported.items.iter().filter(|item| in_class(item) && item.namespace == namespace) {
        let owner = item.name.split("::").next().unwrap_or_default().to_string();
        let text = format!("{}\n", declare(item, "").trim_end());
        match extensions.iter_mut().find(|(of, _)| *of == owner) {
            Some((_, members)) => members.push_str(&text),
            None => extensions.push((owner, text)),
        }
    }
    let mut catching = false;
    let mut operators: Vec<(String, String, String)> = Vec::new();
    // `[FFI-16]` — the error type a status enum's `@ffi(status=E, ok=X, error=N)` names.
    let status_error = |status: &str| -> Option<String> {
        let attributes = applied.attributes.get(status.rsplit('.').next().unwrap_or(status))?;
        attributes.iter().flat_map(|attribute| attribute.split(['(', ',', ')'])).find_map(|argument| {
            let (key, value) = argument.split_once('=')?;
            (key.trim() == "error").then(|| value.trim().to_string())
        })
    };
    for function in imported.functions.iter().filter(|function| function.namespace == namespace && !adopted_member(imported, function)) {
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
            // A result with a mode of its own (a bridge value, `[FFI-17a]`; a lifetime-bound `ref`,
            // `[FFI-11f]`) is made by the wrapper too.
            (Policy::Direct, None) if !function.out && !function.params.iter().any(|(_, ty)| wrapped(ty)) && function.result.as_ref().is_none_or(|ty| ty.mode.is_empty()) => {
                member(function.abi, render_function(function, entry, renamed, hidden))
            }
            // `E5061` where it is used.
            (Policy::Unknown, _) => {}
            (policy, owner) => {
                // `[FFI-39]` — a method of a class with a trampoline is its Ember class's; the base
                // implementations of its named virtuals are the class's own.
                let trampoline = owner.as_ref().and_then(|(owner, _)| imported.trampolines.iter().find(|found| ident(&found.class) == *owner)).map(|found| found.opaque.as_str());
                let hidden = hidden || (trampoline.is_some() && function.name.starts_with(&format!("{}_cpp_base_", ember_branding::SYMBOL_PREFIX)));
                let (raw, wrapper) = render_wrapper(function, entry, renamed, hidden, policy, trampoline, &status_error);
                member(function.abi, raw);
                catching |= policy != Policy::Direct;
                // `[FFI-42]` — a member operator, for its interface below.
                let operator = owner.as_ref().and_then(|(owner, _)| {
                    imported.operators.iter().find(|(of, method, _)| of == owner && *method == function.name && renamed.is_none())
                });
                match owner {
                    Some(_) if wrapper.is_empty() => {}
                    Some((owner, _)) if let Some((_, _, interface)) = operator => operators.push((owner.clone(), interface.clone(), wrapper)),
                    Some((owner, _)) => match extensions.iter_mut().find(|(of, _)| of == owner) {
                        Some((_, methods)) => methods.push_str(&wrapper),
                        None => extensions.push((owner.clone(), wrapper)),
                    },
                    None => wrappers.push_str(&wrapper),
                }
            }
        }
    }
    // `[FFI-42]` — each member operator whose wrapper is its interface's method implements the
    // interface, in a block of its own (`IndexMut` in `Index`'s); any other is a method of its class.
    // An `IndexMut` comes after the `Index` it extends.
    operators.sort_by_key(|(_, interface, _)| interface == "IndexMut");
    for (owner, interface, wrapper) in operators {
        let block = match operator_interface(&interface, &owner, &wrapper) {
            Some((implemented, output)) if interface == "IndexMut" => {
                let index = implemented.replacen("IndexMut[", "Index[", 1);
                let output = output.replacen("type Output = ", "", 1);
                extensions
                    .iter()
                    .position(|(of, methods)| *of == format!("{owner} implements {index}") && methods.starts_with(&format!("type Output = {output}\n")))
                    .map(|at| {
                        extensions[at].0.push_str(&format!(", {implemented}"));
                        at
                    })
            }
            Some((implemented, output)) => {
                extensions.push((format!("{owner} implements {implemented}"), if output.is_empty() { String::new() } else { format!("{output}\n") }));
                Some(extensions.len() - 1)
            }
            None => None,
        };
        let at = match block {
            Some(at) => at,
            None => match extensions.iter().position(|(of, _)| *of == owner) {
                Some(at) => at,
                None => {
                    extensions.push((owner.clone(), String::new()));
                    extensions.len() - 1
                }
            },
        };
        extensions[at].1.push_str(&wrapper);
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
    let mut used: Vec<&str> = C_ALIASES.iter().map(|(ember, _)| *ember).chain(["ForeignBox", "CppError", "adopt", "Retained", "CppVector", "CppString", "CppShared", "CppWeak", "CppObject", "CppFunction", "VaList"]).filter(|name| words.contains(name)).collect();
    // `[FFI-21]` — a `retained` callback's call returns a `Retained[ForeignCallback]`.
    if body.contains("callback(retained") {
        used.push("ForeignCallback");
    }
    // `[FFI-8]` — `volatile T*`, and `_Atomic T`.
    if words.contains("Volatile") {
        let _ = writeln!(imports, "from std.mem import Volatile");
    }
    if words.contains("Atomic") {
        let _ = writeln!(imports, "from std.sync import Atomic");
    }
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

/// `[FFI-42]` — the interface a member operator's wrapper implements, as its `extend` block names
/// it, and the block's `type Output` line: only when the wrapper is safe to call (an overlay entry
/// asserts it, `[FFI-2]`; an interface's method is never `unsafe`) and its signature is the
/// interface method's. `None` otherwise: the operator is then an ordinary method of its class.
fn operator_interface(interface: &str, owner: &str, wrapper: &str) -> Option<(String, String)> {
    let line = wrapper.lines().find(|line| !line.starts_with('@') && !line.starts_with('#'))?;
    let line = line.strip_prefix("pub ").unwrap_or(line).strip_prefix("fn ")?;
    let (params, output) = line.split_once('(')?.1.rsplit_once(')')?;
    let output = output.trim_end_matches(':').trim().strip_prefix("-> ").unwrap_or("");
    let params: Vec<&str> = if params.is_empty() { Vec::new() } else { params.split(", ").collect() };
    // The operand: a plain value parameter (an opaque class's `ref T` is no interface's `Rhs`).
    let operand = match params.get(1) {
        Some(param) => match param.split_once(": ") {
            Some((name, ty)) if !name.contains(' ') && !ty.starts_with("ref ") => Some(ty),
            _ => return None,
        },
        None => None,
    };
    let assign = interface.ends_with("Assign");
    match (interface, params.first().copied(), operand, params.len()) {
        ("Neg" | "Not", Some("self"), None, 1) if !output.is_empty() => Some((interface.to_string(), format!("type Output = {output}"))),
        ("Eq", Some("self"), Some(ty), 2) if ty == owner && output == "bool" => Some(("Eq".to_string(), String::new())),
        ("Index", Some("self"), Some(ty), 2) => {
            output.strip_prefix("ref ").filter(|inner| !inner.starts_with("mut ")).map(|inner| (format!("Index[{ty}]"), format!("type Output = {inner}")))
        }
        ("IndexMut", Some("mut self"), Some(ty), 2) => output.strip_prefix("ref mut ").map(|inner| (format!("IndexMut[{ty}]"), format!("type Output = {inner}"))),
        ("Neg" | "Not" | "Eq" | "Index" | "IndexMut", ..) => None,
        (_, Some("mut self"), Some(ty), 2) if assign && output.is_empty() => Some((format!("{interface}[{ty}]"), String::new())),
        (_, Some("self"), Some(ty), 2) if !assign && !output.is_empty() => Some((format!("{interface}[{ty}]"), format!("type Output = {output}"))),
        _ => None,
    }
}

/// `[FFI-40]` — a parameter of an opaque class, which the wrapper takes as a reference or a
/// `ForeignBox` and passes on as a raw pointer, or one the importer passes in its own way (a bridge
/// value's object, a trampoline's, a view, an optional value).
fn wrapped(ty: &CType) -> bool {
    ty.pointee.is_some() && !ty.mode.is_empty()
}

/// `[FFI-40]`, `[FFI-39]` — a wrapper's parameter as Ember code writes it.
fn wrapper_param(param: &str, ty: &CType) -> String {
    match &ty.pointee {
        Some((inner, _)) if ty.mode == "owned " => format!("owned {param}: ForeignBox[{inner}]"),
        Some((inner, _)) if ty.mode == "tramp " => format!("{param}: {inner}"),
        // `[FFI-17a]` — a bridge value, as the C++ parameter takes it.
        Some((inner, _)) if ty.mode == "bridged " => format!("{param}: {inner}"),
        Some((inner, _)) if ty.mode == "bridged mut " => format!("mut {param}: {inner}"),
        Some((inner, _)) if ty.mode == "bridged owned " => format!("owned {param}: {inner}"),
        // `[FFI-39c]` (ODR-156) — a `retained` one: the token the program pinned and keeps.
        Some((inner, _)) if ty.mode == "token " => format!("{param}: Retained[{inner}]"),
        Some((inner, _)) if wrapped(ty) => format!("{param}: {}{inner}", ty.mode),
        _ => format!("{}{param}: {}", ty.mode, ty.ember),
    }
}

/// `[FFI-40]`, `[FFI-39]` — what a wrapper passes its thunk for a parameter.
fn wrapper_argument(param: &str, ty: &CType) -> String {
    let prefix = ember_branding::SYMBOL_PREFIX;
    match &ty.pointee {
        _ if ty.mode == "owned " && wrapped(ty) => format!("{param}.as_ptr()"),
        _ if ty.mode.starts_with("bridged ") => format!("{param}.as_ptr()"),
        // `[FFI-39]` — the C++ object a trampoline class's object owns.
        Some((_, true)) if ty.mode == "tramp " => format!("{param}.{prefix}_cpp_base.object as {}", ty.ember),
        Some((_, false)) if ty.mode == "tramp " => format!("{param}.{prefix}_cpp_base.object"),
        Some((_, true)) if ty.mode == "token " => format!("{prefix}_cpp_token_object({param}).{prefix}_cpp_base.object as {}", ty.ember),
        Some((_, false)) if ty.mode == "token " => format!("{prefix}_cpp_token_object({param}).{prefix}_cpp_base.object"),
        Some((inner, true)) if wrapped(ty) => format!("(ref {param}) as *{inner}"),
        // The caller lent it mutably; the borrow the wrapper passes on is its own.
        Some((inner, false)) if wrapped(ty) => format!("((ref {param}) as *{inner}) as *mut {inner}"),
        _ => param.to_string(),
    }
}

/// `[FFI-24]`, `[FFI-24b]`, `[FFI-40]` — a C++ function reached through an Ember function of the
/// module: the declaration of its thunk, private (the receiver first for a method; then each
/// parameter, a view as its pointer and its length; then what the thunk writes — a class by value,
/// a view's length, an optional's value — and, when the thunk catches, where it says it caught),
/// and the function the module exports — a method, associated function or constructor of the
/// class's type in an `extend` block, or a function returning `Result[T, CppError]` (or `T`,
/// panicking on a throw, when an overlay asserts `@ffi(noexcept)`). A destructor has only its
/// declaration, which the type's `ForeignBox` calls.
///
/// `[FFI-2]`, `[FFI-11]` (D-808) — when an overlay entry states every fact, the declaration is
/// contracted as a C function's is (`safe fn`, `@ffi(param(…), result(…))`), so the checker lowers
/// each safe type to what the thunk takes, as for any hand declaration (`[FFI-10]`): a `count(n)`
/// witness and an `out` leave its callable signature, a `Span` gives the pointer and its length.
/// What the importer passes itself is a `borrowed one` reference, a `Span` or an `out` of the
/// declaration too, and the wrapper takes the safe types, needing `unsafe` only to name an object
/// by its address or to own one the thunk gives (`ADDRESS`). Otherwise the declaration takes raw
/// pointers, and the wrapper is `unsafe`: its calls are the caller's to keep (`[FFI-38]`).
fn render_wrapper(function: &Function, entry: Option<&Entry>, renamed: Option<&String>, hidden: bool, policy: Policy, trampoline: Option<&str>, status_error: &dyn Fn(&str) -> Option<String>) -> (String, String) {
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
    let contract = entry.filter(|entry| member != Some(Member::Destructor) && contracts_everything(entry));
    let contracted = contract.is_some();
    let words_of = |index: usize| contract.and_then(|entry| entry.params.get(index)).map_or(&[][..], Vec::as_slice);
    // The words the importer itself acts on, whatever else the entry leaves unknown.
    let said = |index: usize, wanted: &str| entry.and_then(|entry| entry.params.get(index)).is_some_and(|words| words.iter().any(|written| word(written) == wanted));
    let has = |words: &[String], wanted: &str| words.iter().any(|written| word(written) == wanted);
    let one = |exclusive: bool| -> Vec<String> { ["borrowed", "one"].into_iter().chain(exclusive.then_some("exclusive")).map(str::to_string).collect() };
    let deref = |pointer: String, constant: bool| format!("{prefix}_cpp_ref{}({pointer})", if constant { "" } else { "_mut" });
    let self_name = format!("{prefix}_self");
    // Each slot of the declaration: its name, its text, its contract words, and what the wrapper
    // passes (`None`: it leaves the callable signature).
    let mut slots: Vec<(String, String, Vec<String>, Option<String>)> = Vec::new();
    let mut params: Vec<String> = Vec::new();
    let mut locals: Vec<String> = Vec::new();
    // Lines before the call that need no `unsafe`, each indented as written.
    let mut prelude: Vec<String> = Vec::new();
    // Whether passing the arguments needs `unsafe` (to name an object by its address).
    let mut call_unsafe = !contracted;
    // The receiver: the object `self` is, or the C++ object a trampoline class's object owns.
    let receiver = match (member, trampoline) {
        (Some(Member::Method), None) => Some((owner, false, "ref self".to_string(), format!("(ref self) as *{owner}"))),
        (Some(Member::MutMethod), None) => Some((owner, true, "ref mut self".to_string(), format!("(ref mut self) as *mut {owner}"))),
        (Some(Member::Destructor), _) => Some((owner, true, String::new(), String::new())),
        (Some(Member::Method), Some(opaque)) => Some((opaque, false, deref(format!("self.{prefix}_cpp_base.object"), true), format!("self.{prefix}_cpp_base.object as *{opaque}"))),
        (Some(Member::MutMethod), Some(opaque)) => Some((opaque, true, deref(format!("self.{prefix}_cpp_base.object"), false), format!("self.{prefix}_cpp_base.object"))),
        _ => None,
    };
    if let Some((object, mutable, reference, address)) = receiver {
        let mutable_word = if mutable { "mut " } else { "" };
        if contracted {
            call_unsafe |= trampoline.is_some();
            slots.push((self_name.clone(), format!("{self_name}: ref {mutable_word}{object}"), one(mutable), Some(reference)));
        } else {
            slots.push((self_name.clone(), format!("{self_name}: *{mutable_word}{object}"), Vec::new(), Some(address)));
        }
    }
    match member {
        Some(Member::Method) => params.push("self".to_string()),
        // A class method's receiver is its handle, which any method may write through ([CLS-7]).
        Some(Member::MutMethod) if trampoline.is_some() => params.push("self".to_string()),
        Some(Member::MutMethod) => params.push("mut self".to_string()),
        _ => {}
    }
    // A `count(n)` witness, an `out` and an `inout_count`'s count leave the callable signature.
    let words: Vec<Vec<String>> = function.params.iter().enumerate().map(|(index, (_, ty))| slot_words(ty, words_of(index))).collect();
    let witnesses: HashSet<&str> = words.iter().flatten().filter_map(|written| written.strip_prefix("count(").and_then(|rest| rest.strip_suffix(')'))).map(str::trim).collect();
    // `[FFI-21]` — a callback's user data leaves the callable signature too: the call gives C the
    // callable's address there.
    let user_data: HashSet<String> = words.iter().flatten().filter_map(|written| callback_user_data(written)).collect();
    let (mut outs, mut inout) = (Vec::new(), None);
    let mut given = Vec::new();
    for (index, (param, ty)) in function.params.iter().enumerate() {
        let constant = ty.pointee.as_ref().is_some_and(|(_, constant)| *constant);
        let inner = ty.pointee.as_ref().map_or(String::new(), |(inner, _)| inner.clone());
        let opaque = ty.ember.trim_start_matches('*').trim_start_matches("mut ").to_string();
        let mutable_word = if constant { "" } else { "mut " };
        let mut ty = ty.clone();
        // `[FFI-39c]` (ODR-156), `[FFI-17d]` — an object of a class with a trampoline that C++ keeps
        // is passed as the token the program holds; one given up gets a count of its own.
        if ty.mode == "tramp " && said(index, "retained") {
            ty.mode = "token ".to_string();
        }
        if ty.mode == "tramp " && said(index, "owned") {
            given.push(format!("{prefix}_cpp_give({param})"));
        }
        if !wrapped(&ty) {
            // A pointer the entry's words contract, a scalar, a value, and a C++ reference to a
            // type by value (`[FFI-40]`: `mut p: T`, `owned p: T`).
            let (text, contract_words) = if contracted {
                (safe_parameter(&format!("{}{param}", ty.mode), &ty, &words[index]), words[index].clone())
            } else {
                (format!("{}{param}: {}", ty.mode, ty.ember), Vec::new())
            };
            if has(&contract_words, "out") {
                outs.push(inner.clone());
            } else if has(&contract_words, "inout_count") {
                inout = Some(inner.clone());
            }
            let leaves = contracted
                && (witnesses.contains(param.trim_start_matches("r#"))
                    || user_data.contains(param.trim_start_matches("r#"))
                    || has(&contract_words, "out")
                    || has(&contract_words, "inout_count"));
            if !leaves {
                // `[FFI-21]` — a callback with user data: the callable the caller passes, of the
                // callback's other parameters (the declaration's is the C function pointer).
                let callable = contract_words.iter().find(|written| callback_user_data(written).is_some()).and_then(|written| {
                    let data = callback_user_data(written)?;
                    let (_, data_ty) = function.params.iter().find(|(name, _)| name.trim_start_matches("r#") == data)?;
                    let kept = written.contains("once") || written.contains("retained");
                    let callable = callable_of(ty.function.as_deref()?, &data_ty.ember, callback_user_data_place(written), written.contains("once"))?;
                    // ODR-253 — a `nullable` one is an `Option` of a callable value, taken over
                    // where C keeps it.
                    Some(if has(&contract_words, "nullable") {
                        format!("{}{param}: Option[{callable}]", if kept { "owned " } else { "" })
                    } else {
                        format!("{param}: {callable}")
                    })
                });
                params.push(callable.unwrap_or_else(|| text.clone()));
            }
            slots.push((param.clone(), text, contract_words, (!leaves).then(|| param.clone())));
            continue;
        }
        params.push(match ty.mode.as_str() {
            // `[FFI-17a]` — a view, an optional value, a `std::unique_ptr` given up.
            "view " => format!("{param}: Span[{inner}]"),
            "view mut " => format!("{param}: MutSpan[{inner}]"),
            "optional " => format!("{param}: Option[{inner}]"),
            "unique " => format!("owned {param}: ForeignBox[{inner}]"),
            fixed if fixed.starts_with("fixed ") => format!("{param}: ref {}", fixed_array(fixed, &inner).1),
            // ODR-198 — a variant: its enum.
            variant if variant.starts_with("variant ") => format!("{param}: {}", variant.trim().rsplit(' ').next().unwrap_or_default()),
            _ => wrapper_param(param, &ty),
        });
        // `[FFI-17a]` — a view is its pointer and its length: a `Span` the declaration counts, or a
        // raw pointer and the length beside it.
        if ty.mode.starts_with("view ") {
            let length = format!("{prefix}_{}_size", param.trim_start_matches("r#"));
            if contracted {
                let mut contract_words = vec!["borrowed".to_string(), format!("count({length})")];
                contract_words.extend((!constant).then(|| "exclusive".to_string()));
                slots.push((param.clone(), format!("{param}: {}[{inner}]", if constant { "Span" } else { "MutSpan" }), contract_words, Some(param.clone())));
                slots.push((length.clone(), format!("{length}: usize"), Vec::new(), None));
            } else {
                // A `MutSpan` the caller lent: its elements are the callee's to write.
                let pointer = if constant { format!("{param}.as_ptr()") } else { format!("{param}.as_ptr() as *mut {inner}") };
                slots.push((param.clone(), format!("{param}: {}", ty.ember), Vec::new(), Some(pointer)));
                slots.push((length.clone(), format!("{length}: usize"), Vec::new(), Some(format!("{param}.len()"))));
            }
            continue;
        }
        let (text, contract_words, argument) = match (ty.mode.as_str(), contracted) {
            ("ref ", true) => (format!("{param}: ref {inner}"), one(false), param.clone()),
            ("ref mut ", true) => (format!("{param}: ref mut {inner}"), one(true), param.clone()),
            // The `ForeignBox` the callee moves from, whose drop frees what is left.
            ("owned ", true) => (format!("{param}: ref mut {inner}"), one(true), format!("{param}.get_mut()")),
            ("bridged ", true) => (format!("{param}: ref CppObject"), one(false), deref(format!("{param}.as_ptr() as *CppObject"), true)),
            ("bridged mut " | "bridged owned ", true) => (format!("{param}: ref mut CppObject"), one(true), deref(format!("{param}.as_ptr() as *mut CppObject"), false)),
            ("tramp ", true) => (format!("{param}: ref {mutable_word}{opaque}"), one(!constant), deref(format!("{param}.{prefix}_cpp_base.object"), constant)),
            ("token ", true) => (format!("{param}: ref {mutable_word}{opaque}"), one(!constant), deref(format!("{prefix}_cpp_token_object({param}).{prefix}_cpp_base.object"), constant)),
            // `[FFI-17a]` — `std::optional<T>`: a pointer to the value, null for none.
            ("optional ", true) => (format!("{param}: Option[ref {inner}]"), ["borrowed", "one", "nullable"].map(str::to_string).to_vec(), format!("{param}.as_ref()")),
            ("optional ", false) => {
                let at = format!("{prefix}_{}_at", param.trim_start_matches("r#"));
                prelude.extend([
                    format!("{at} = null[*{inner}]()"),
                    format!("match {param}:"),
                    "    Some(ref inner):".to_string(),
                    format!("        {at} = (ref inner) as *{inner}"),
                    "    None:".to_string(),
                    "        pass".to_string(),
                ]);
                (format!("{param}: {}", ty.ember), Vec::new(), at)
            }
            // ODR-198 — a variant: the bytes the enum makes, which live for the call.
            (variant, contracted) if variant.starts_with("variant ") => {
                let ops = variant.trim().split(' ').nth(1).unwrap_or_default();
                let held = format!("{prefix}_{}_held", param.trim_start_matches("r#"));
                prelude.push(format!("{held} = {ops}_store({param})"));
                if contracted {
                    (format!("{param}: ref {inner}"), one(false), held)
                } else {
                    (format!("{param}: *{inner}"), Vec::new(), format!("(ref {held}) as *{inner}"))
                }
            }
            // ODR-200 — a `std::span<T, N>`: the array's address.
            (fixed, true) if fixed.starts_with("fixed ") => {
                let (extent, array) = fixed_array(fixed, &inner);
                let words = ["borrowed".to_string(), format!("fixed({extent})")].into_iter().chain(array.starts_with("mut ").then(|| "exclusive".to_string())).collect();
                (format!("{param}: ref {array}"), words, param.clone())
            }
            (fixed, false) if fixed.starts_with("fixed ") => {
                let mutable = fixed_array(fixed, &inner).1.starts_with("mut ");
                let pointer = format!("*{}{inner}", if mutable { "mut " } else { "" });
                (format!("{param}: {pointer}"), Vec::new(), format!("ref_to_ptr({}{param}) as {pointer}", if mutable { "ref mut " } else { "ref " }))
            }
            // `[FFI-17a]` — a `std::unique_ptr<T>` parameter: the box is given up (`owned`).
            ("unique ", true) => (format!("owned {param}: ForeignBox[{inner}]"), vec!["owned".to_string(), "one".to_string()], param.clone()),
            ("unique ", false) => (format!("{param}: {}", ty.ember), Vec::new(), format!("{param}.into_raw()")),
            (_, _) => (format!("{param}: {}", ty.ember), Vec::new(), wrapper_argument(param, &ty)),
        };
        call_unsafe |= ty.mode.starts_with("bridged ") || matches!(ty.mode.as_str(), "tramp " | "token ");
        slots.push((param.clone(), text, contract_words, Some(argument)));
    }
    // What the thunk writes where the caller says: a class by value (`[FFI-32]`), a view's length,
    // an optional's value (`[FFI-17a]`) — an `out`, which the call returns, or a slot the wrapper
    // gives.
    let result_ty = function.result.as_ref().map(|ty| ty.ember.clone()).unwrap_or_default();
    let result_mode = function.result.as_ref().map_or("", |ty| ty.mode.as_str());
    let pointee = function.result.as_ref().and_then(|ty| ty.pointee.clone()).map(|(inner, _)| inner).unwrap_or_default();
    let written: Option<(String, String)> = if function.out {
        Some(("out".to_string(), result_ty.clone()))
    } else if result_mode.starts_with("view") {
        Some(("size".to_string(), "usize".to_string()))
    } else if result_mode == "optional" {
        Some(("value".to_string(), pointee.clone()))
    } else {
        None
    };
    if let Some((what, ty)) = &written {
        let name = format!("{prefix}_{what}");
        if contracted {
            slots.push((name.clone(), format!("mut {name}: {ty}"), vec!["out".to_string(), "one".to_string()], None));
        } else if what == "size" {
            locals.push(format!("{name}: usize = 0"));
            slots.push((name.clone(), format!("{name}: *mut usize"), Vec::new(), Some(format!("(ref mut {name}) as *mut usize"))));
        } else {
            locals.push(format!("{prefix}_slot: MaybeUninit[{ty}] = MaybeUninit[{ty}].uninit()"));
            slots.push((name.clone(), format!("{name}: *mut {ty}"), Vec::new(), Some(format!("((ref mut {prefix}_slot) as *mut MaybeUninit[{ty}]) as *mut {ty}"))));
        }
    }
    if catching {
        let failed = format!("{prefix}_failed");
        locals.push(format!("{failed}: c_int = 0"));
        if contracted {
            slots.push((failed.clone(), format!("mut {failed}: c_int"), one(true), Some(failed.clone())));
        } else {
            slots.push((failed.clone(), format!("{failed}: *mut c_int"), Vec::new(), Some(format!("(ref mut {failed}) as *mut c_int"))));
        }
    }
    // The result the declaration gives: the entry's words for a pointer, a `T&`'s reference, the
    // address of an object the caller comes to own, a view's data, whether an optional has a value.
    let from_self = |written: &String| if written.replace(' ', "") == "from(self)" { format!("from({self_name})") } else { written.clone() };
    let user_words: Vec<String> = contract.map_or(Vec::new(), |entry| canonical(entry.result.iter().filter(|written| word(written) != "unsafe").cloned().collect()));
    // ODR-199 — the entry makes a `std::function` result an Ember callable.
    let std_function = contract.is_some_and(|entry| entry.std_function);
    // `[FFI-11]` — the region a result borrows, as the entry's `from(…)` names it.
    let lifetime = user_words.iter().find_map(|written| written.strip_prefix("from(").and_then(|rest| rest.strip_suffix(')'))).map(|from| from.trim().to_string());
    let (result, result_words, borrows) = match &function.result {
        None => (None, Vec::new(), None),
        Some(_) if function.out => (None, Vec::new(), None),
        // `[FFI-40]` — a method's `T&` borrows its receiver; `[FFI-11f]` — and each parameter
        // `[[clang::lifetimebound]]` names, of which a contracted declaration names the first.
        Some(ty) if ty.mode.starts_with("ref") => {
            let mutable = ty.mode.starts_with("ref mut");
            let bound = ty.mode.split_once(" from ").map(|(_, from)| from.to_string());
            if contracted {
                let first = bound.as_deref().and_then(|from| from.split(", ").next()).unwrap_or("self");
                let source = if first == "self" { self_name.clone() } else { first.to_string() };
                let mut words = one(mutable);
                words.push(format!("from({source})"));
                (Some(format!("ref {}{pointee}", if mutable { "mut " } else { "" })), words, bound)
            } else {
                (Some(ty.ember.clone()), Vec::new(), bound)
            }
        }
        Some(ty) if ty.mode.is_empty() && contracted => {
            let borrows = lifetime.clone().filter(|from| from != "static");
            (Some(safe_result(ty, &user_words)), user_words.iter().map(from_self).collect(), borrows)
        }
        // A view's lifetime is the entry's word (`[FFI-17a]`, `[FFI-11]`).
        Some(ty) if ty.mode.starts_with("view") => (Some(ty.ember.clone()), Vec::new(), lifetime.clone().filter(|from| from != "static")),
        // ODR-200 — a `std::span<T, N>`: a `ref [T; N]` borrowing what the entry names; with no
        // entry, its elements' address.
        Some(ty) if ty.mode.starts_with("fixed ") && contracted => {
            let (extent, array) = fixed_array(&ty.mode, &pointee);
            let mut words = vec!["borrowed".to_string(), format!("fixed({extent})")];
            words.extend(array.starts_with("mut ").then(|| "exclusive".to_string()));
            words.extend(user_words.iter().filter(|written| word(written) == "from").map(from_self));
            (Some(format!("ref {array}")), words, lifetime.clone().filter(|from| from != "static"))
        }
        Some(ty) => (Some(ty.ember.clone()), Vec::new(), None),
    };
    let declared: Vec<&str> = slots.iter().map(|(_, text, _, _)| text.as_str()).collect();
    let arrow = result.as_ref().map_or(String::new(), |result| format!(" -> {result}"));
    let link = function.link_name.as_deref().map(|link| format!("link_name=\"{link}\""));
    let raw_text = if contracted {
        let mut arguments: Vec<String> = slots.iter().filter(|(_, _, words, _)| !words.is_empty()).map(|(name, _, words, _)| format!("param({name}, {})", words.join(", "))).collect();
        if !result_words.is_empty() {
            arguments.push(format!("result({})", result_words.join(", ")));
        }
        // `[FFI-21]` — `destroys(f)` names the C++ function's declaration here.
        arguments.extend(contract.iter().flat_map(|entry| entry.arguments.iter().map(|argument| match argument.trim().strip_prefix("destroys(").and_then(|rest| rest.strip_suffix(')')) {
            Some(function) => format!("destroys({prefix}_raw_{})", function.trim()),
            None => argument.clone(),
        })));
        arguments.extend(link);
        format!("    @ffi({})\n    safe fn {raw_name}({}){arrow}\n", arguments.join(", "), declared.join(", "))
    } else {
        let attribute = link.map_or(String::new(), |link| format!("    @ffi({link})\n"));
        format!("{attribute}    @safety(\"{UNCONTRACTED}\")\n    fn {raw_name}({}){arrow}\n", declared.join(", "))
    };
    if member == Some(Member::Destructor) {
        return (raw_text, String::new());
    }
    let name = match member {
        Some(Member::Constructor | Member::BoxedConstructor) => "new".to_string(),
        _ => ident(renamed.unwrap_or(&function.name)),
    };
    // What the call gives (`[FFI-11b]`, `[FFI-16]`, ODR-137): a status's `Result` of the outputs,
    // else the C++ function's own result first, then each output in its order.
    let call = format!("{raw_name}({})", slots.iter().filter_map(|(_, _, _, argument)| argument.clone()).collect::<Vec<_>>().join(", "));
    let tuple = |parts: &[String]| if parts.len() == 1 { parts[0].clone() } else { format!("({})", parts.join(", ")) };
    // The value of the C++ function's own result, from what the declaration returns (`given`) and
    // what the thunk wrote (`slot`).
    let own = |given: Option<String>, slot: Option<String>| -> (Option<String>, Option<String>, bool) {
        let given_or = |default: &str| given.clone().unwrap_or_else(|| default.to_string());
        match (member, result_mode) {
            // ODR-198 — a variant's bytes, made its enum.
            (_, mode) if mode.starts_with("variant ") => {
                let mut words = mode.split(' ').skip(1);
                let (ops, ember) = (words.next().unwrap_or_default(), words.next().unwrap_or_default());
                (slot.map(|slot| format!("{ops}_take({slot})")), Some(ember.to_string()), false)
            }
            _ if function.out => (slot, Some(result_ty.clone()), false),
            (Some(Member::BoxedConstructor), _) => (Some(format!("adopt[{owner}]({} as *mut {owner})", given_or("0"))), Some(format!("ForeignBox[{owner}]")), true),
            (_, "boxed") => (Some(format!("adopt[{pointee}]({} as *mut {pointee})", given_or("0"))), Some(format!("ForeignBox[{pointee}]")), true),
            (_, made) if made.starts_with("bridge ") => (Some(made.trim_start_matches("bridge ").replace('@', &given_or("0"))), Some(pointee.clone()), true),
            // ODR-199 — a `std::function`: the object, owned; with `@ffi(std_function)`, the callable
            // capturing it.
            (_, made) if made.starts_with("function ") => {
                let (ops, callable) = made.trim_start_matches("function ").split_once(' ').unwrap_or_default();
                let object = format!("CppFunction.from_thunks({} as *mut u8, {ops}_destroy)", given_or("0"));
                if std_function {
                    (Some(format!("{ops}_callable({object})")), Some(callable.to_string()), true)
                } else {
                    (Some(object), Some("CppFunction".to_string()), true)
                }
            }
            (_, made) if made.starts_with("view") => {
                let (span, mutable) = if made == "view mut" { ("MutSpan", true) } else { ("Span", false) };
                let constant = if mutable { "mut " } else { "" };
                let data = format!("{} as *{constant}{pointee}", given_or("0"));
                let size = slot.unwrap_or_default();
                if !contracted {
                    // No lifetime word: the pointer and the length, the caller's to keep (`[FFI-38]`).
                    return (Some(format!("({data}, {size})")), Some(format!("(*{constant}{pointee}, usize)")), false);
                }
                let span_of = format!("{prefix}_cpp_span{}", if mutable { "_mut" } else { "" });
                let made = match lifetime.as_deref() {
                    Some("static") | None => format!("{span_of}({data}, {size})"),
                    Some("self") => format!("{span_of}(ref {}self, {data}, {size})", if mutable { "mut " } else { "" }),
                    Some(source) => format!("{span_of}({source}, {data}, {size})"),
                };
                (Some(made), Some(format!("{span}[{pointee}]")), true)
            }
            (_, "optional") => (Some(format!("Some({}) if {} else None", slot.unwrap_or_default(), given_or("false"))), Some(format!("Option[{pointee}]")), false),
            (_, mode) if mode.starts_with("ref") && !contracted => {
                let mutable = mode.starts_with("ref mut");
                (Some(format!("{prefix}_cpp_ref{}({})", if mutable { "_mut" } else { "" }, given_or("0"))), Some(format!("ref {}{pointee}", if mutable { "mut " } else { "" })), true)
            }
            _ => (given, result.clone(), false),
        }
    };
    let mut statements = Vec::new();
    // Whether making the value needs `unsafe` (to own an object the thunk gives, or view it).
    let mut value_unsafe = !contracted;
    let status = has(&result_words, "status");
    let (value_expr, value) = if contracted && status {
        let error = result.as_deref().and_then(status_error).unwrap_or_default();
        (Some(call.clone()), Some(format!("Result[{}, {error}]", if outs.is_empty() { "void".to_string() } else { tuple(&outs) })))
    } else if contracted && let Some(count) = &inout {
        (Some(call.clone()), Some(result.as_ref().map_or(count.clone(), |result| format!("({result}, {count})"))))
    } else if contracted {
        // The parts the call returns: the declaration's own result, each `out` of the entry, and
        // what the thunk wrote for the importer.
        let parts = usize::from(result.is_some()) + outs.len() + usize::from(written.is_some());
        let names: Vec<String> = (0..parts).map(|index| format!("{prefix}_part{index}")).collect();
        let (given, user, slot) = if parts == 0 {
            statements.push(call.clone());
            (None, Vec::new(), None)
        } else if parts == 1 {
            let whole = Some(call.clone());
            match (result.is_some(), written.is_some()) {
                (true, _) => (whole, Vec::new(), None),
                (false, true) => (None, Vec::new(), whole),
                (false, false) => (None, outs.iter().map(|_| call.clone()).collect(), None),
            }
        } else {
            statements.push(format!("{} = {call}", names.join(", ")));
            let mut rest = names.iter().cloned();
            let given = result.is_some().then(|| rest.next()).flatten();
            let user: Vec<String> = rest.by_ref().take(outs.len()).collect();
            (given, user, rest.next())
        };
        let (own_value, own_ty, unsafe_made) = own(given, slot);
        value_unsafe |= unsafe_made;
        let value_expr = std::iter::once(own_value.clone()).flatten().chain(user.iter().cloned()).collect::<Vec<_>>();
        let value_ty = std::iter::once(own_ty.clone()).flatten().chain(outs.iter().cloned()).collect::<Vec<_>>();
        if value_expr.is_empty() {
            (None, None)
        } else {
            (Some(tuple(&value_expr)), Some(tuple(&value_ty)))
        }
    } else {
        // The thunk's slot is read once the call has written it.
        let slot = written.as_ref().map(|(what, _)| if what == "size" { format!("{prefix}_size") } else { format!("{prefix}_slot.assume_init()") });
        let given = if result_mode == "optional" {
            // Whether there is a value, before the slot is read.
            statements.push(format!("{prefix}_has = {call}"));
            Some(format!("{prefix}_has"))
        } else if result.is_some() {
            Some(call.clone())
        } else {
            statements.push(call.clone());
            None
        };
        let (own_value, own_ty, _) = own(given, slot);
        (own_value, own_ty)
    };
    let returned = match (policy, &value) {
        (Policy::Direct | Policy::Asserted, Some(value)) => format!(" -> {value}"),
        (Policy::Direct | Policy::Asserted, None) => String::new(),
        (_, value) => format!(" -> Result[{}, CppError]", value.as_deref().unwrap_or("void")),
    };
    // An entry that states every fact contracts the declaration; one that leaves a fact unknown
    // leaves the call unsafe (`[FFI-11c]`).
    let safe = contracted;
    let quoted = function.name.replace('"', "");
    let failed = format!("{prefix}_failed");
    let mut text = String::new();
    if !safe {
        let _ = writeln!(text, "@safety(\"{UNCONTRACTED}\")");
    }
    if let Some(from) = &borrows {
        let _ = writeln!(text, "@borrows({from})");
    }
    let public = if hidden { "" } else { "pub " };
    let note = if safe { String::new() } else { "  # SAFETY: the caller keeps what the header does not state".to_string() };
    let _ = writeln!(text, "{public}{}fn {name}({}){returned}:{note}", if safe { "" } else { "unsafe " }, params.join(", "));
    for local in locals.iter().chain(&given).chain(&prelude) {
        let _ = writeln!(text, "    {local}");
    }
    // The body's lines, each with whether it needs `unsafe`: the statements (which make the call),
    // the line that returns the value (which makes it, and the call when no statement does), and
    // what follows. Consecutive lines that need it share one `unsafe:` block, and no other line is
    // in one (`L3010`).
    let mut lines: Vec<(String, bool)> = statements.iter().map(|statement| (statement.clone(), call_unsafe)).collect();
    let returned_unsafe = value_unsafe || (statements.is_empty() && call_unsafe);
    match (value_expr, policy) {
        (Some(value), Policy::Direct) => lines.push((format!("return {value}"), returned_unsafe)),
        (Some(value), Policy::Asserted) => lines.push((format!("return {prefix}_cpp_value({value}, {failed}, \"{quoted}\")"), returned_unsafe)),
        (Some(value), _) => lines.push((format!("return {prefix}_cpp_result({value}, {failed})"), returned_unsafe)),
        (None, Policy::Direct) => {}
        (None, Policy::Asserted) => lines.push((format!("{prefix}_cpp_value((), {failed}, \"{quoted}\")"), false)),
        (None, _) => lines.push((format!("return {prefix}_cpp_result((), {failed})"), false)),
    }
    let lifetime_bound = function.result.as_ref().and_then(|ty| ty.mode.split_once(" from ")).map(|(_, from)| from.to_string()).filter(|_| !contracted);
    let safety = match lifetime_bound {
        Some(from) => format!(
            "# SAFETY: the thunk takes this function's own arguments; its result lives as long as `{from}`, as the header's `[[clang::lifetimebound]]` states: asserted, never checked, since C++ does not enforce it [FFI-11f]"
        ),
        None if contracted => {
            let mut reasons = Vec::new();
            if call_unsafe {
                reasons.push("each object named by its address is the one this function's argument holds, borrowed for the call".to_string());
            }
            if result_mode.starts_with("view") {
                let lives = lifetime.as_deref().filter(|from| *from != "static").map_or("for the program's whole run".to_string(), |from| format!("as long as `{from}`"));
                reasons.push(format!("the view the thunk gives lives {lives}, as the overlay's entry asserts [FFI-17a]"));
            } else if result_mode == "boxed" || result_mode.starts_with("bridge ") || result_mode.starts_with("function ") || member == Some(Member::BoxedConstructor) {
                reasons.push("the object the thunk gives is a new one, which nothing else owns".to_string());
            }
            format!("# SAFETY: {}", reasons.join("; "))
        }
        None => "# SAFETY: the thunk takes this function's own arguments".to_string(),
    };
    let mut open = false;
    for (line, needs) in lines {
        if needs && !open {
            let _ = writeln!(text, "    unsafe:  {safety}");
        }
        open = needs;
        let _ = writeln!(text, "{}{line}", if needs { "        " } else { "    " });
    }
    text.push('\n');
    (raw_text, text)
}

/// `[FFI-2]`, `[FFI-11c]` — whether an entry states every fact: no `TODO(…)` word, and no `unsafe`.
fn contracts_everything(entry: &Entry) -> bool {
    !entry.params.iter().flatten().chain(&entry.result).any(|written| matches!(word(written), "TODO" | "unsafe"))
}

/// A wrapper's parameters, its receiver first, and its result, as `render_wrapper` writes its
/// line.
fn signature_of(wrapper: &str) -> Option<(Vec<String>, Option<String>)> {
    let line = wrapper.lines().find(|line| !line.starts_with('@') && !line.starts_with('#'))?;
    let (_, rest) = line.split_once('(')?;
    let (params, after) = rest.rsplit_once(')')?;
    let mut split = Vec::new();
    let (mut depth, mut start) = (0, 0);
    for (at, c) in params.char_indices() {
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => depth -= 1,
            ',' if depth == 0 => {
                split.push(params[start..at].trim().to_string());
                start = at + 1;
            }
            _ => {}
        }
    }
    if !params.trim().is_empty() {
        split.push(params[start..].trim().to_string());
    }
    // What follows the parameters, up to the line's `:` and its comment.
    let after = after.split('#').next().unwrap_or("").trim().trim_end_matches(':');
    let returned = after.trim().strip_prefix("->").map(|returned| returned.trim().to_string());
    Some((split, returned))
}

/// The overlay entry that states nothing yet for a function: each unknown fact a `TODO(…)` word
/// (`[FFI-11c]`), and `-> unsafe` where every parameter and the result are scalars, since being safe
/// to call is the one fact no header states (`[FFI-2]`).
fn starter_entry(function: &Function) -> String {
    // A pointer the header declares; what the importer passes in its own way (a C++ reference, a
    // bridge value, a view, an optional value) states its facts.
    let todo = |ty: &CType| (ty.pointee.is_some() && ty.mode.is_empty()) || ty.ember.starts_with("Option[");
    let mut text = String::new();
    for (name, ty) in function.params.iter().filter(|(_, ty)| ty.mode.is_empty()) {
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
        // `[FFI-17a]` — a view's lifetime, which no header states.
        Some(ty) if ty.mode.starts_with("view") => " -> TODO(lifetime)",
        Some(ty) if todo(ty) => " -> TODO(ownership) TODO(count) TODO(nullable) TODO(lifetime)",
        _ if function.params.iter().any(|(_, ty)| todo(ty)) => "",
        _ => " -> unsafe",
    };
    let _ = writeln!(text, "    fn {}({}){result}", function.name, params.join(", "));
    text
}

/// `[CLI-5]`, `[CLI-6]`, `[CLI-7]` — `ember bind <header> [-I dir] [-D name[=value]]
/// (--init [--merge] | --report [--overlay <file>] [--baseline <file> | --write-baseline <file>]
/// | --check <overlay>)`. `[CXX-1]` — with `--flag <flag>` (a project flag, as `[cpp.<project>]`
/// `flags` gives it) and `--instantiate <template-id>`, `--outcomes [--overlay <file>]` prints each
/// declaration's outcome, and `--emit-cpp [--out <file>]` writes the thunk file.
pub fn bind(args: &[String]) -> Result<std::process::ExitCode, String> {
    use std::process::ExitCode;
    let mut header = None;
    let (mut include_paths, mut defines) = (Vec::new(), Vec::new());
    let (mut init, mut merge, mut report) = (false, false, false);
    let (mut check, mut overlay, mut baseline, mut write_baseline) = (None, None, None, None);
    let (mut explain, mut emit) = (None, false);
    // `[FFI-50]` — `--emit-cpp`: the header is C++, and its thunk file is written for inspection.
    let (mut emit_cpp, mut standard) = (false, "c++17".to_string());
    // `[FFI-17b]` — `--instantiate <template-id>`, as the import's `instantiate=[…]`.
    let mut instantiate = Vec::new();
    // `[CXX-1]` — the project's flags, its corpus's outcomes, and where its thunk file goes.
    let (mut flags, mut outcomes, mut out) = (Vec::new(), false, None);
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
            "--instantiate" => instantiate.push(value("--instantiate")?),
            "--flag" => flags.push(value("--flag")?),
            "--outcomes" => outcomes = true,
            "--out" => out = Some(PathBuf::from(value("--out")?)),
            flag if flag.starts_with('-') => return Err(format!("unknown `ember bind` option `{flag}`")),
            file if header.is_none() => header = Some(PathBuf::from(file)),
            extra => return Err(format!("unexpected `ember bind` argument `{extra}`")),
        }
    }
    let header = header.ok_or("`ember bind` needs a header")?;
    let file = header.file_name().ok_or("`ember bind` needs a header file")?.to_string_lossy().into_owned();
    // A C++ header by its extension, or for `--emit-cpp`.
    let cpp = emit_cpp || header.extension().is_some_and(|extension| matches!(extension.to_string_lossy().as_ref(), "hpp" | "hh" | "hxx" | "h++" | "ipp"));
    let dir = header.parent().filter(|dir| !dir.as_os_str().is_empty()).unwrap_or(Path::new(".")).to_path_buf();
    let shared = HashMap::new();
    let config = Config {
        header: &file,
        from_dir: &dir,
        include_paths: &include_paths,
        defines: &defines,
        flags: &flags,
        symbol_stem: "bind",
        shared: &shared,
        binding: if cpp { "cpp" } else { "c" },
        cpp: cpp.then_some(standard.as_str()),
        instantiate: &instantiate,
        trampolines: &[],
        variants: &[],
    };
    let (imported, files) = import_header(&config)?;
    if emit_cpp {
        // The header by its name, as the file beside it includes it: the same file on every machine.
        let absolute = std::path::absolute(&dir).unwrap_or_else(|_| dir.clone());
        let mut shim = imported.shim.clone();
        for written in [absolute.display().to_string(), absolute.display().to_string().replace('\\', "/")] {
            shim = shim.replace(&format!("{written}{}", if written.contains('\\') { "\\" } else { "/" }), "");
        }
        let path = out.unwrap_or_else(|| dir.join(format!("{}.thunks.cpp", header.file_stem().unwrap_or_default().to_string_lossy())));
        std::fs::write(&path, shim).map_err(|error| format!("cannot write `{}`: {error}", path.display()))?;
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
    if outcomes {
        // `[CXX-1]` — each declaration's outcome: by value, opaque, behind an overlay, or refused
        // with the diagnostic the import reports it with.
        let applied = match &overlay {
            Some(path) => load_overlay(path, &mut sink)?,
            None => Applied::default(),
        };
        eprint!("{}", sink.render(&map));
        print!("{}", outcome_lines(&imported, &applied));
        return Ok(if sink.has_errors() { ExitCode::FAILURE } else { ExitCode::SUCCESS });
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
        // `[FFI-24d]` — a C++ function's exception policy is part of its Ember signature: the
        // baseline keeps each one's (`api <name> noexcept|throwing`).
        let policy = |function: &Function| if function.policy == Policy::Direct { "noexcept" } else { "throwing" };
        let key = |function: &Function| match &function.owner {
            Some((owner, _)) => format!("{owner}.{}", function.name),
            None => function.name.clone(),
        };
        let api: Vec<String> = if cpp { imported.functions.iter().map(|function| format!("api {} {}", key(function), policy(function))).collect() } else { Vec::new() };
        if let Some(path) = write_baseline {
            let lines: Vec<&str> = names.iter().copied().chain(api.iter().map(String::as_str)).collect();
            std::fs::write(&path, lines.join("\n") + "\n").map_err(|error| format!("cannot write `{path}`: {error}"))?;
        }
        let known: Vec<String> = match &baseline {
            Some(path) => std::fs::read_to_string(path).map_err(|error| format!("cannot read `{path}`: {error}"))?.lines().map(str::to_string).collect(),
            None => Vec::new(),
        };
        let new = names.iter().filter(|name| !known.iter().any(|known| known == *name)).count();
        if new > 0 {
            println!("{new} skipped declaration(s){}", if baseline.is_some() { " not in the baseline" } else { "" });
        }
        // `[FFI-24d]` — API changes: each function whose header added or removed `noexcept` since
        // the baseline, which changes its Ember signature.
        let mut changes = 0;
        for function in imported.functions.iter().filter(|_| cpp) {
            let name = key(function);
            let before = known.iter().find_map(|line| line.strip_prefix("api ").and_then(|rest| rest.rsplit_once(' ')).filter(|(found, _)| *found == name).map(|(_, policy)| policy));
            if let Some(before) = before.filter(|before| *before != policy(function)) {
                changes += 1;
                let what = if policy(function) == "noexcept" {
                    "`noexcept` added: it returns its value, no longer a `Result[…, CppError]`"
                } else {
                    "`noexcept` removed: it returns a `Result[…, CppError]`"
                };
                println!("API change: `{name}` was {before}: {what} [FFI-24d]");
            }
        }
        return Ok(if new > 0 || changes > 0 || sink.has_errors() { ExitCode::FAILURE } else { ExitCode::SUCCESS });
    }
    Err("`ember bind` needs `--init`, `--report`, `--outcomes`, `--check <overlay>`, `--explain <fn>`, `--emit-embind` or `--emit-cpp`".to_string())
}

/// `[CXX-1]` — an import's outcome for each declaration, one line each, in the order the header
/// declares them: `value` (it crosses by value), `opaque` (a type held behind a pointer), `overlay`
/// (a function whose pointers' facts only an overlay states: `stated` when the overlay given does)
/// or `refused <code>` (not imported, with the diagnostic the import reports: `E5055` for a template,
/// `W5001` for a macro, `W5033`, `W5034`, else `W5002` or the code its reason names).
fn outcome_lines(imported: &Imported, applied: &Applied) -> String {
    let mut text = String::new();
    let path = |namespace: &[String], name: &str| namespace.iter().map(String::as_str).chain([name]).collect::<Vec<_>>().join(".");
    for item in &imported.items {
        let (outcome, what) = match item.kind {
            Kind::Type => ("value", "type"),
            Kind::Opaque => ("opaque", "type"),
            Kind::Const => ("value", "const"),
            Kind::Static | Kind::Variable => ("value", "static"),
            Kind::Class => ("overlay", "class"),
        };
        let _ = writeln!(text, "{outcome:<8} {what:<6} {}", path(&item.namespace, &item.name));
    }
    for function in &imported.functions {
        let name = match &function.owner {
            Some((owner, _)) => format!("{owner}.{}", function.name),
            None => function.name.clone(),
        };
        let policy = if function.policy == Policy::Direct { "noexcept" } else { "throwing" };
        let pointers: Vec<&str> = function
            .params
            .iter()
            // `[FFI-17a]` — a view, an optional value and a `std::unique_ptr` state their facts; a view
            // result its lifetime only in an overlay.
            .filter(|(_, ty)| ty.pointee.is_some() && !matches!(ty.mode.as_str(), "view " | "view mut " | "optional " | "unique ") && !ty.mode.starts_with("fixed "))
            .map(|(param, _)| param.as_str())
            .chain(function.result.iter().filter(|ty| ty.pointee.is_some() && !matches!(ty.mode.as_str(), "boxed" | "optional")).map(|_| "the result"))
            .collect();
        let line = if applied.functions.contains_key(&function.name) {
            format!("overlay  fn     {} ({policy}; stated)", path(&function.namespace, &name))
        } else if pointers.is_empty() {
            format!("value    fn     {} ({policy})", path(&function.namespace, &name))
        } else {
            format!("overlay  fn     {} ({policy}; unstated: {})", path(&function.namespace, &name), pointers.join(", "))
        };
        let _ = writeln!(text, "{line}");
    }
    let refused = |code: &str, skipped: &Skipped| format!("refused  {code:<6} {} ({})\n", skipped.name, skipped.at);
    for skipped in &imported.skipped {
        let code = skipped.reason.strip_prefix('[').and_then(|rest| rest.split_once(']')).map_or("W5002", |(code, _)| code);
        text.push_str(&refused(code, skipped));
    }
    for skipped in &imported.rvalue_members {
        text.push_str(&refused("W5033", skipped));
    }
    for skipped in &imported.anonymous_namespace {
        text.push_str(&refused("W5034", skipped));
    }
    for skipped in imported.macros.iter().filter(|skipped| !applied.macros.iter().any(|(function, ..)| function.name == skipped.name)) {
        text.push_str(&refused("W5001", skipped));
    }
    text
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

/// 15: a function pointer's parameters and result keep their written typedef names (`intptr_t` is
/// `isize` there too, `[FFI-8]`); the records adopted from an earlier import (D-879).
const EMBIND_VERSION: u64 = 15;

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
    Value::Array(list.iter().map(|skip| texts(&[&skip.name, &skip.reason, &skip.at, &skip.namespace.join("::")])).collect())
}

fn read_skips(value: &Value) -> Option<Vec<Skipped>> {
    array_of(value)?
        .iter()
        .map(|entry| {
            let entry = array_of(entry)?;
            Some(Skipped { name: text_of(entry.first()?)?, reason: text_of(entry.get(1)?)?, at: text_of(entry.get(2)?)?, namespace: split_namespace(&text_of(entry.get(3)?)?) })
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
        Value::Uint(u64::from(ty.opaque)),
        Value::Text(ty.function.clone().unwrap_or_default()),
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
        opaque: uint_of(entry.get(6)?)? == 1,
        function: Some(text_of(entry.get(7)?)?).filter(|function| !function.is_empty()),
    })
}

const KINDS: [Kind; 6] = [Kind::Type, Kind::Const, Kind::Opaque, Kind::Static, Kind::Class, Kind::Variable];
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
                            ("derived".to_string(), Value::Uint(function.derived as u64)),
                            ("variadic".to_string(), Value::Uint(u64::from(function.variadic))),
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
        ("conflicts".to_string(), Value::Array(imported.conflicts.iter().map(|(name, header, config)| texts(&[name, header, config])).collect())),
        ("adopted".to_string(), Value::Array(imported.adopted.iter().map(|name| Value::Text(name.clone())).collect())),
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
        (
            "trampolines".to_string(),
            Value::Array(
                imported
                    .trampolines
                    .iter()
                    .map(|found| texts(&[&found.class, &found.opaque, if found.default_init { "1" } else { "0" }, &found.unnamed_virtuals.join(",")]))
                    .collect(),
            ),
        ),
        ("operators".to_string(), Value::Array(imported.operators.iter().map(|(owner, method, interface)| texts(&[owner, method, interface])).collect())),
        (
            "policy_changes".to_string(),
            Value::Array(imported.policy_changes.iter().map(|(namespace, name, now)| texts(&[namespace, name, if *now { "1" } else { "0" }])).collect()),
        ),
        (
            "declared".to_string(),
            Value::Array(imported.declared.iter().map(|(namespace, name, text, at)| texts(&[namespace, name, text, at])).collect()),
        ),
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
                derived: [Derived::Nothing, Derived::Pure, Derived::Const].get(uint_of(field(entry, "derived")?)? as usize).copied()?,
                variadic: uint_of(field(entry, "variadic")?)? == 1,
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
        conflicts: triples(field(&value, "conflicts")?)?,
        adopted: array_of(field(&value, "adopted")?)?.iter().map(text_of).collect::<Option<_>>()?,
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
        trampolines: array_of(field(&value, "trampolines")?)?
            .iter()
            .map(|entry| {
                let entry = array_of(entry)?;
                let unnamed = text_of(entry.get(3)?)?;
                Some(Trampoline {
                    class: text_of(entry.first()?)?,
                    opaque: text_of(entry.get(1)?)?,
                    default_init: text_of(entry.get(2)?)? == "1",
                    unnamed_virtuals: unnamed.split(',').filter(|name| !name.is_empty()).map(str::to_string).collect(),
                })
            })
            .collect::<Option<_>>()?,
        operators: triples(field(&value, "operators")?)?,
        policy_changes: triples(field(&value, "policy_changes")?)?.into_iter().map(|(namespace, name, now)| (namespace, name, now == "1")).collect(),
        declared: array_of(field(&value, "declared")?)?
            .iter()
            .map(|entry| {
                let entry = array_of(entry)?;
                Some((text_of(entry.first()?)?, text_of(entry.get(1)?)?, text_of(entry.get(2)?)?, text_of(entry.get(3)?)?))
            })
            .collect::<Option<_>>()?,
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
/// Returned with the record's path and the hash of the files the header included, each with its
/// content's hash (`[HR-23]`).
fn import_cached(config: &Config, overlay: &str, cached: &mut bool) -> Result<(Imported, PathBuf, String), String> {
    let included = |files: &[(String, String)]| {
        let mut lines: Vec<String> = files.iter().map(|(file, hash)| format!("{file}={hash}")).collect();
        lines.sort();
        ember_build::content_hash(lines.join("\n").as_bytes())
    };
    load_libclang()?;
    let overlay_hash = if overlay.is_empty() { String::new() } else { ember_build::content_hash(overlay.as_bytes()) };
    let library = clang_sys::get_library().map(|library| library.path().display().to_string()).unwrap_or_default();
    let mut shared: Vec<String> = config
        .shared
        .iter()
        .map(|(usr, shared)| format!("{usr}={}:{}:{}:{}:{}:{}", shared.module.join("."), shared.name, shared.layout, shared.binding, shared.cpp, shared.config))
        .collect();
    shared.sort();
    // `[FFI-17b]` — the instantiations listed are part of what is imported.
    // `[FFI-39]` (ODR-221) — the trampolines the program's classes ask for are part of it too.
    let trampolines: Vec<String> = config.trampolines.iter().map(|decl| format!("{}:{}:{}:{}", decl.class, decl.virtuals.join(","), decl.foreign, decl.from_program)).collect();
    let configuration = format!(
        "{library}\n{}\n{}\n{}\n{}\n{}",
        arguments(config).join(" "),
        config.header,
        std::path::absolute(config.from_dir).unwrap_or_default().display(),
        config.instantiate.join("\n"),
        trampolines.join(";")
    );
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
    let previous = match std::fs::read(&path).ok().and_then(|bytes| read_embind(&bytes)) {
        Some((files, imported)) if files.iter().all(|(file, hash)| file_hash(file).as_deref() == Some(hash.as_str())) => {
            *cached = true;
            return Ok((imported, path, included(&files)));
        }
        // `[FFI-24d]` — the binding the header's last parse made, to compare with.
        previous => previous.map(|(_, imported)| imported),
    };
    let (mut imported, files) = import_header(config)?;
    if let Some(previous) = &previous {
        imported.policy_changes = policy_changes(previous, &imported);
    }
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
    Ok((imported, path, included(&files)))
}

/// `[FFI-24d]` — for the module of `namespace`, each function whose Ember signature a `noexcept`
/// the header added or removed changed (an overlay's `@ffi(noexcept)` keeps it), by its Ember name,
/// with the note a diagnostic in dependent code gives as the cause.
fn changed_signatures(imported: &Imported, applied: &Applied, namespace: &str, header: &str) -> Vec<(String, String)> {
    imported
        .policy_changes
        .iter()
        .filter(|(at, name, _)| at == namespace && applied.functions.get(name.rsplit('.').next().unwrap_or(name)).is_none_or(|entry| entry.policy.is_none()))
        .map(|(_, name, now)| {
            let change = if *now {
                format!("`{header}` now declares `{name}` `noexcept` (it did not when this import was last bound), so it returns its value, no longer a `Result[…, CppError]`")
            } else {
                format!("`{header}` no longer declares `{name}` `noexcept` (it did when this import was last bound), so it returns a `Result[…, CppError]`, as a function that may throw does")
            };
            (name.clone(), format!("the cause: {change} [FFI-24d]"))
        })
        .collect()
}

/// `[FFI-24d]` — each function of `current` whose header added or removed `noexcept` against
/// `previous` (a member by `Owner.name`), and each change `previous` recorded that still holds.
fn policy_changes(previous: &Imported, current: &Imported) -> Vec<(String, String, bool)> {
    let key = |function: &Function| match &function.owner {
        Some((owner, _)) => format!("{owner}.{}", function.name),
        None => function.name.clone(),
    };
    let direct = |imported: &Imported, namespace: &str, name: &str| {
        imported.functions.iter().find(|function| function.namespace.join("::") == namespace && key(function) == name).map(|function| function.policy == Policy::Direct)
    };
    let mut changes: Vec<(String, String, bool)> = Vec::new();
    for function in &current.functions {
        let (namespace, name) = (function.namespace.join("::"), key(function));
        let now = function.policy == Policy::Direct;
        if direct(previous, &namespace, &name).is_some_and(|before| before != now) {
            changes.push((namespace, name, now));
        }
    }
    for (namespace, name, now) in &previous.policy_changes {
        if direct(current, namespace, name) == Some(*now) && !changes.iter().any(|(at, found, _)| at == namespace && found == name) {
            changes.push((namespace.clone(), name.clone(), *now));
        }
    }
    changes
}

fn opaque_by_value(name: &str, why: Option<&str>) -> String {
    match why {
        // The code a reason carries leads, for the type checker (`[FFI-48]`'s `E5034`).
        Some(why) if let Some((code, rest)) = why.strip_prefix("[E").and_then(|rest| rest.split_once("] ")) => {
            format!("[E{code}] `{name}` by value, which is opaque: {rest}")
        }
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
