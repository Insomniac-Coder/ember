//! The LLVM backend: MIR to textual LLVM IR (Part XVIII §7, `[CG-LL-1]`–`[CG-LL-3]`).
//!
//! It takes the verified MIR the C backend takes (`ember_mir::verify::for_codegen`), the type
//! table and its layouts, and writes one `.ll` module, which the Clang the build finds compiles
//! (and so verifies). No LLVM library is linked into the compiler. The runtime stays C: the IR
//! calls it through the entry points the C backend's C calls, by the platform's C ABI, which this
//! backend implements itself (`abi`).
//!
//! `[CG-LL-2]` — the IR has no undefined behaviour: no `nsw`, `nuw`, `inbounds` or `noundef`
//! that is not proved, checked arithmetic through `llvm.*.with.overflow`, shifts masked to their
//! width, and no fast-math flag but where `@fastmath` asks (`[TYP-9]`, `[DET-5]`).
//!
//! The IR is written for LLVM 21 and later (opaque pointers; nothing newer).

pub mod abi;
mod arrays;
mod builtins;
mod closures;
mod cmp;
mod dispatch;
mod fmt;
mod func;
mod glue;
mod ranges;
mod runtime;
mod types;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;

use ember_mir::Body;
use ember_mir::verify::VerifiedMir;
use ember_span::SourceMap;

pub use abi::Target;
use types::Types;

/// `[PRF-3]` — the build's profile, which the C backend reads from the C compiler's macros
/// (`EMBER_RELEASE`, `EMBER_RT_DEBUG_ACCESS`) and this backend from here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Profile {
    Debug,
    Development,
    Release,
}

pub struct Options<'a> {
    pub module_name: &'a str,
    /// The program's `main` is its entry: the module defines the C `main`.
    pub has_main: bool,
    /// `[WK-15]` — the debug leak check.
    pub leak_check: bool,
    pub profile: Profile,
    pub target: Target,
    /// The triple Clang compiles the module for (`clang -print-effective-triple`), so it has none
    /// to override; `None` leaves it out.
    pub triple: Option<String>,
}

pub struct Output {
    /// The module's IR.
    pub ir: String,
    /// The C unit through which the IR calls the runtime's inline functions, when it calls any:
    /// compiled and linked beside the IR (`runtime::shim_source`).
    pub shim: Option<String>,
}

/// What this backend cannot build yet: an error naming the construct, never a crash.
#[derive(Debug)]
pub struct Unsupported(pub String);

impl std::fmt::Display for Unsupported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the LLVM backend does not build {} yet", self.0)
    }
}

pub(crate) type Result<T> = std::result::Result<T, Unsupported>;

pub(crate) fn unsupported<T>(what: impl Into<String>) -> Result<T> {
    Err(Unsupported(what.into()))
}

/// Everything the functions of one module share: its constants, the declarations its calls need,
/// and the bodies it defines.
pub(crate) struct Module<'a> {
    pub ty: Types<'a>,
    pub map: &'a SourceMap,
    pub profile: Profile,
    pub bodies: HashMap<&'a str, &'a Body>,
    /// Each byte string's constant, by its index (`@.str.N`, NUL-terminated so a C function can
    /// read it as a `const char*`).
    strings: BTreeMap<Vec<u8>, usize>,
    /// The functions the module calls without defining, by symbol: their `declare` lines.
    decls: BTreeMap<String, String>,
    /// Other global definitions, in the order made.
    globals: Vec<String>,
    /// The function definitions.
    pub fns: String,
    /// The glue the functions asked for, written after them (`glue::emit_pending`).
    pub pending: Vec<glue::Pending>,
    /// The helper functions written so far, by symbol.
    helpers: BTreeSet<String>,
    /// What the program's MIR says about dynamic dispatch.
    pub dispatch: dispatch::Dispatch,
    /// The runtime's inline functions the module calls through the shim (`runtime::shim_source`).
    pub shims: BTreeSet<&'static str>,
    /// The program's callable values.
    pub callables: closures::Callables,
    /// Functions the module's C shim defines beyond the wrappers (a sort), by symbol.
    pub shim_functions: BTreeMap<String, String>,
    /// Each type with a `clone` method, its symbol and whether it takes its receiver by address.
    pub clone_fns: HashMap<ember_types::Ty, (String, bool)>,
}

impl<'a> Module<'a> {
    pub fn request(&mut self, work: glue::Pending) {
        self.pending.push(work);
    }

    /// Whether the helper `symbol` is written; from now on it counts as written.
    pub fn helper_defined(&mut self, symbol: &str) -> bool {
        !self.helpers.insert(symbol.to_string())
    }

    /// A NUL-terminated constant holding `bytes`, as a `ptr` operand.
    pub fn bytes(&mut self, bytes: &[u8]) -> String {
        let next = self.strings.len();
        let index = *self.strings.entry(bytes.to_vec()).or_insert(next);
        format!("@.str.{index}")
    }

    pub fn declare(&mut self, symbol: &str, line: impl FnOnce() -> String) {
        if !self.decls.contains_key(symbol) {
            let line = line();
            self.decls.insert(symbol.to_string(), line);
        }
    }

    pub fn global(&mut self, line: String) {
        self.globals.push(line);
    }
}

/// An IR identifier: the symbol as written when LLVM reads it so, else quoted.
pub(crate) fn global_name(symbol: &str) -> String {
    let plain = !symbol.is_empty()
        && !symbol.starts_with(|c: char| c.is_ascii_digit())
        && symbol.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$' | '-'));
    if plain { format!("@{symbol}") } else { format!("@\"{}\"", escape(symbol.as_bytes())) }
}

/// Bytes as an IR string body: printable ASCII as itself, everything else `\XX`.
pub(crate) fn escape(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    for &byte in bytes {
        if (0x20..0x7f).contains(&byte) && byte != b'"' && byte != b'\\' {
            out.push(byte as char);
        } else {
            let _ = write!(out, "\\{byte:02X}");
        }
    }
    out
}

/// The module for `mir`.
pub fn emit(mir: VerifiedMir<'_>, map: &SourceMap, options: &Options<'_>) -> std::result::Result<Output, Unsupported> {
    let bodies = mir.bodies();
    let mut module = Module {
        ty: Types::new(mir.types(), options.target),
        map,
        profile: options.profile,
        bodies: bodies.iter().map(|body| (body.symbol.as_str(), body)).collect(),
        strings: BTreeMap::new(),
        decls: BTreeMap::new(),
        globals: Vec::new(),
        fns: String::new(),
        pending: Vec::new(),
        helpers: BTreeSet::new(),
        dispatch: dispatch::Dispatch::default(),
        shims: BTreeSet::new(),
        callables: closures::Callables::default(),
        shim_functions: BTreeMap::new(),
        clone_fns: bodies
            .iter()
            .filter(|body| {
                body.arg_count == 1 && body.name == "clone" && body.locals.get(1).is_some_and(|local| local.name.as_deref() == Some("self"))
            })
            .filter_map(|body| {
                let receiver = body.locals[1].ty;
                let (owner, by_address) = match mir.types().kind(receiver) {
                    ember_types::TyKind::Ref { inner, .. } => (*inner, true),
                    _ => (receiver, false),
                };
                (body.return_ty() == owner).then(|| (owner, (body.symbol.clone(), by_address)))
            })
            .collect(),
    };
    module.dispatch = dispatch::Dispatch::collect(&module, bodies);
    module.callables = closures::Callables::collect(&module, bodies);
    let defined: BTreeSet<&str> = bodies
        .iter()
        .filter(|body| !body.is_abstract && !body.is_extern_declaration)
        .map(|body| body.symbol.as_str())
        .collect();
    for body in bodies {
        if body.is_abstract || body.is_extern_declaration {
            continue;
        }
        if body.ffi_counted.is_some() {
            return unsupported("a foreign function's counted wrapper ([FFI-11])");
        }
        func::emit_body(&mut module, body)?;
    }
    if options.has_main {
        func::emit_entry(&mut module, bodies, options)?;
    }
    glue::emit_pending(&mut module)?;
    for symbol in &defined {
        module.decls.remove(*symbol);
    }

    let mut ir = String::new();
    let _ = writeln!(ir, "; Generated by the Ember compiler. Do not edit.");
    let _ = writeln!(ir, "; module: {}", options.module_name);
    let _ = writeln!(ir, "source_filename = \"{}\"", escape(options.module_name.as_bytes()));
    if let Some(triple) = &options.triple {
        let _ = writeln!(ir, "target triple = \"{}\"", escape(triple.as_bytes()));
    }
    ir.push('\n');
    let mut strings: Vec<(&Vec<u8>, &usize)> = module.strings.iter().collect();
    strings.sort_by_key(|(_, index)| **index);
    for (bytes, index) in strings {
        let _ = writeln!(
            ir,
            "@.str.{index} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1",
            bytes.len() + 1,
            escape(bytes)
        );
    }
    for global in &module.globals {
        ir.push_str(global);
        ir.push('\n');
    }
    ir.push('\n');
    for line in module.decls.values() {
        ir.push_str(line);
        ir.push('\n');
    }
    ir.push('\n');
    ir.push_str(&module.fns);
    let shim = (!module.shims.is_empty() || !module.shim_functions.is_empty()).then(|| {
        let mut source = runtime::shim_source(&module.shims);
        for function in module.shim_functions.values() {
            source.push_str(function);
        }
        source
    });
    Ok(Output { ir, shim })
}
