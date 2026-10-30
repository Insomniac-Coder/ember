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
mod arenas;
mod arrays;
mod builtins;
mod closures;
mod cmp;
mod debug;
mod dispatch;
mod ffi;
mod fmt;
mod fold;
mod func;
mod glue;
mod leaks;
mod ranges;
mod reload;
mod runtime;
mod simd;
mod soa;
mod statics;
mod sync;
mod types;
mod views;
mod visualisers;

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
    /// `[TST-3]` — a test binary's tests, by symbol, in the runner's order: its entry runs the one
    /// its argument numbers.
    pub tests: Option<Vec<String>>,
    /// Annex B — a reloadable image's functions, statics and mode, as the driver decides them.
    pub reload: Option<&'a ember_codegen_c::ReloadImage>,
    /// `[FFI-28]` — a shared library: the module's entry points, and its exports' `creator` thread
    /// its own.
    pub module: bool,
    /// `[BLD-FFI-5a]` — a library's `@sync` classes' functions its header declares.
    pub classes: &'a [ember_codegen_c::ClassExport],
    /// `[FFI-28]` — a library package's name, under whose namespace its shims are named.
    pub package: Option<&'a str>,
    /// `[FFI-37]` — an instrumented test (`ember test --instrument-ffi`): each call to a foreign
    /// function counted at its site.
    pub instrument_ffi: bool,
    /// `[BLD-4]` (ODR-244) — a `debug` program as a unit per module, as the C
    /// backend writes it: each unit's name and the bodies it defines (`Output::units`); this module
    /// defines the rest, and the entry.
    pub units: Option<&'a [(String, BTreeSet<String>)]>,
}

pub struct Output {
    /// The module's IR.
    pub ir: String,
    /// The C unit through which the IR calls the runtime's inline functions, when it calls any:
    /// compiled and linked beside the IR (`runtime::shim_source`).
    pub shim: Option<String>,
    /// `[DSP-5]` — `--emit-optimization-report`'s lines: each virtual call made directly, by place,
    /// as the C backend writes them.
    pub optimization_report: Vec<String>,
    /// `[CG-C-9]` — the program's debugger visualisers, which the build writes beside it.
    pub visualisers: ember_codegen_c::Visualisers,
    /// `[FFI-37]` — what an instrumented test counts, and the C unit that counts it.
    pub ffi_instrument: Option<ember_codegen_c::FfiInstrumentOutput>,
    /// `[BLD-4]` — each unit `Options::units` names, by its name: its IR.
    pub units: Vec<(String, String)>,
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
    /// The glue written so far, each once however often asked for.
    pub pending_done: BTreeSet<glue::Pending>,
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
    /// `[CG-LL-3]` — the debug information.
    pub debug: debug::Debug,
    /// `[CG-C-3]` — the bodies the C backend makes `static inline` (`inlinehint` here), with
    /// whether it forces them (`alwaysinline`): the same bodies inline whichever backend builds.
    pub inline_bodies: BTreeMap<String, bool>,
    /// `[FFI-21]` — the lambdas C calls back through a trampoline, with what C passes.
    pub callback_closures: BTreeSet<String>,
    /// D-569 — on Windows x64, the functions whose views go as two words (`fold::split_view_functions`).
    pub split_views: BTreeSet<String>,
    /// Annex B — a reloadable image's functions, statics and mode (`reload.rs`), and what the image
    /// says of itself (`ember_codegen_c::image`'s planner, taken while it writes).
    pub reload: Option<ember_codegen_c::ReloadImage>,
    pub reload_plan: Option<ember_codegen_c::image::Planner<'a>>,
    /// The interface identities the image defines.
    pub reload_ids: BTreeSet<String>,
    /// A function the planner's writer could not write (`reload::LlvmWriter`).
    pub reload_error: Option<Unsupported>,
    /// `[FFI-28]` — a shared library's module (`Options::module`).
    pub module: bool,
    /// The namespace of the shims' names: a library's package's (`Options::package`), else none.
    pub shim_namespace: String,
    /// `[DSP-5]` — each virtual call made directly to the one body it can run, by place
    /// (`Output::optimization_report`).
    pub direct_calls: BTreeSet<(ember_span::FileId, u32, String)>,
    /// `[FFI-37]` — an instrumented test's foreign functions and call sites.
    pub ffi: Option<ffi::FfiInstrument>,
    /// `[BLD-4]` — the program is units (`Options::units`): each one's own definitions are one
    /// program-wide (`one_definition`), and what it names of another's is declared.
    pub split: bool,
    /// The IR name of each body's function, to the body's symbol, where the program is units.
    body_names: std::sync::Arc<HashMap<String, String>>,
}

impl<'a> Module<'a> {
    /// `[BLD-4]` — a module for one unit of the program: this one's view of the program, written
    /// before anything of it is (no reloadable image, no instrumented test: either is one unit).
    fn fork(&self, options: &Options<'_>) -> Module<'a> {
        Module {
            ty: self.ty.clone(),
            map: self.map,
            profile: self.profile,
            bodies: self.bodies.clone(),
            strings: BTreeMap::new(),
            decls: BTreeMap::new(),
            globals: Vec::new(),
            fns: String::new(),
            pending: Vec::new(),
            pending_done: BTreeSet::new(),
            helpers: BTreeSet::new(),
            dispatch: self.dispatch.clone(),
            shims: BTreeSet::new(),
            callables: self.callables.clone(),
            shim_functions: BTreeMap::new(),
            clone_fns: self.clone_fns.clone(),
            debug: debug::Debug::new(self.map, options.module_name, options.profile != Profile::Debug, options.profile == Profile::Debug),
            inline_bodies: self.inline_bodies.clone(),
            callback_closures: self.callback_closures.clone(),
            split_views: self.split_views.clone(),
            reload: None,
            reload_plan: None,
            reload_ids: BTreeSet::new(),
            reload_error: None,
            module: self.module,
            shim_namespace: self.shim_namespace.clone(),
            direct_calls: BTreeSet::new(),
            ffi: None,
            split: true,
            body_names: self.body_names.clone(),
        }
    }

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

/// An IR identifier: the symbol as written when LLVM reads it so, else quoted. `[FFI-49]`,
/// `[MNG-2]` — a foreign or exported function's alias (`em_0x616273_3abs`) is the symbol it
/// stands for (`abs`), which IR spells whatever it is.
pub(crate) fn global_name(symbol: &str) -> String {
    let symbol = &link_symbol(symbol);
    let plain = !symbol.is_empty()
        && !symbol.starts_with(|c: char| c.is_ascii_digit())
        && symbol.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$' | '-'));
    if plain { format!("@{symbol}") } else { format!("@\"{}\"", escape(symbol.as_bytes())) }
}

/// `[FFI-9]` — the symbol the linker sees for the function `body` of a module built for `target`:
/// its own, or a `__vectorcall` function's MSVC decoration (`scale@@16`), for a link that keeps
/// the program's exports (`/INCLUDE:`).
pub fn function_link_symbol(types: &ember_types::TypeTable, target: Target, body: &Body) -> String {
    if body.abi.as_deref() == Some("vectorcall") && target == Target::Win64 {
        return func::vectorcall_symbol(&Types::new(types, target), body);
    }
    link_symbol(&body.symbol)
}

/// The symbol the linker sees for `symbol`: an alias's exact symbol, else itself.
pub(crate) fn link_symbol(symbol: &str) -> String {
    ember_branding::export_alias_symbol(symbol).unwrap_or_else(|| symbol.to_string())
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
        pending_done: BTreeSet::new(),
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
        debug: debug::Debug::new(map, options.module_name, options.profile != Profile::Debug, options.profile == Profile::Debug),
        // `[HR-9a]` — a reloadable function is never inlined: its callers see only its thunk.
        inline_bodies: ember_codegen_c::inline_header_symbols(bodies, mir.types())
            .into_iter()
            .filter(|(symbol, _)| !options.reload.is_some_and(|reload| reload.reloadable.contains(symbol)))
            .collect(),
        callback_closures: bodies
            .iter()
            .filter_map(|body| body.ffi_counted.as_ref())
            .flat_map(|counted| counted.callbacks.iter().filter_map(|callback| callback.closure.clone()))
            .collect(),
        // A reloadable function's body is named by the reload manifest, not only called.
        split_views: if options.target == Target::Win64 {
            fold::split_view_functions(bodies)
                .into_iter()
                .filter(|symbol| !options.reload.is_some_and(|reload| reload.reloadable.contains(symbol)))
                .collect()
        } else {
            BTreeSet::new()
        },
        reload: options.reload.cloned(),
        reload_plan: options.reload.map(|reload| ember_codegen_c::image::Planner::new(mir.types(), reload, bodies, &ember_codegen_c::image::lazy_statics(bodies))),
        reload_ids: BTreeSet::new(),
        reload_error: None,
        module: options.module,
        shim_namespace: options.package.map(ember_codegen_c::package_type_namespace).unwrap_or_default(),
        direct_calls: BTreeSet::new(),
        ffi: options.instrument_ffi.then(|| ffi::FfiInstrument::new(bodies)),
        split: options.units.is_some(),
        body_names: Default::default(),
    };
    if let Some(reload) = options.reload {
        module.ty.header = reload.header_bytes();
        reload::tables(&mut module);
    }
    module.dispatch = dispatch::Dispatch::collect(&module, bodies);
    module.callables = closures::Callables::collect(&module, bodies);
    // `[BLD-4]` (ODR-244), `[BLD-7]` — each module's unit first: its own bodies, and what they ask
    // for, each unit a function of the program and its bodies alone, so the units are shared out in
    // runs among the `-j` threads. What a unit made for itself the main module may make too; each
    // is one program-wide.
    let mut units = Vec::new();
    let plan = options.units.unwrap_or(&[]);
    let in_units: BTreeSet<&str> = plan.iter().flat_map(|(_, symbols)| symbols.iter().map(String::as_str)).collect();
    if module.split {
        module.body_names = std::sync::Arc::new(bodies.iter().map(|body| (func::body_name(&module, body), body.symbol.clone())).collect());
        let run = plan.len().div_ceil(ember_mir::parallel::jobs().max(1)).max(1);
        let template = &module;
        let runs: Vec<Result<Vec<Unit>>> = std::thread::scope(|scope| {
            let writers: Vec<_> = plan
                .chunks(run)
                .map(|run| ember_span::thread::spawn_scoped(scope, move || run.iter().map(|(name, symbols)| write_unit(template, name, symbols, bodies, options)).collect()))
                .collect();
            writers.into_iter().map(|writer| writer.join().expect("a unit's writer does not panic")).collect()
        });
        for run in runs {
            for unit in run? {
                module.shims.extend(unit.shims);
                module.shim_functions.extend(unit.shim_functions);
                module.direct_calls.extend(unit.direct_calls);
                module.debug.shown.extend(unit.shown);
                units.push((unit.name, unit.ir));
            }
        }
    }
    let defined = define_bodies(&mut module, bodies, |body| !in_units.contains(body.symbol.as_str()))?;
    if let Some(tests) = &options.tests {
        func::emit_entry(&mut module, bodies, options, Some(tests))?;
    } else if options.has_main {
        func::emit_entry(&mut module, bodies, options, None)?;
    }
    if options.module {
        func::emit_module_entry(&mut module, options.leak_check)?;
    }
    ffi::emit_class_exports(&mut module, options.classes)?;
    glue::emit_pending(&mut module)?;
    // Annex B — the image's manifest and what it names (`[HR-8]`).
    if module.reload.is_some() {
        reload::emit_manifest(&mut module, bodies, options.has_main)?;
    }
    // `[FFI-26]` — each export table: a constant of its struct holding each field's function, under
    // a symbol carrying its protocol version (ODR-129).
    for table in module.ty.table.export_tables().to_vec() {
        let (size, align) = module.ty.size_align(table.ty);
        let offsets = module.ty.table.layout(table.ty).field_offsets;
        let (mut types, mut values, mut end) = (Vec::new(), Vec::new(), 0u64);
        for (entry, offset) in table.entries.iter().zip(offsets) {
            if offset > end {
                types.push(format!("[{} x i8]", offset - end));
                values.push(format!("[{} x i8] zeroinitializer", offset - end));
            }
            if !defined.contains(entry.as_str()) && module.bodies.contains_key(entry.as_str()) {
                func::declare_body_symbol(&mut module, entry)?;
            }
            types.push("ptr".to_string());
            values.push(format!("ptr {}", global_name(entry)));
            end = offset + 8;
        }
        if size > end {
            types.push(format!("[{} x i8]", size - end));
            values.push(format!("[{} x i8] zeroinitializer", size - end));
        }
        module.global(format!(
            "{} = constant <{{ {} }}> <{{ {} }}>, align {}",
            global_name(&table.symbol()),
            types.join(", "),
            values.join(", "),
            align.max(1)
        ));
        // The program may read it back as a foreign static: defined here, not declared.
        module.decls.remove(&link_symbol(&table.symbol()));
    }
    finish_definitions(&mut module, &defined)?;
    // `[FFI-37]` — the foreign functions an instrumented test counts, as the instrumentation unit
    // declares them: each one's name and, where a site calls it, the address of what it calls,
    // which names its library.
    let ffi_instrument = module.ffi.take().map(|ffi| {
        let called: BTreeSet<u32> = ffi.sites.iter().map(|(function, _)| *function).collect();
        let mut entries = Vec::with_capacity(ffi.functions.len());
        for (index, (name, address)) in ffi.functions.iter().enumerate() {
            let name = module.bytes(name.as_bytes());
            let address = match ffi.declared.get(address) {
                Some(declared) => declared.clone(),
                None if module.decls.contains_key(&link_symbol(address)) => global_name(address),
                None => "null".to_string(),
            };
            let address = if called.contains(&(index as u32)) { address } else { "null".to_string() };
            entries.push(format!("{{ ptr, ptr }} {{ ptr {name}, ptr {address} }}"));
        }
        let rt = ember_branding::RUNTIME_PREFIX;
        let table = if entries.is_empty() {
            "[1 x { ptr, ptr }] zeroinitializer".to_string()
        } else {
            format!("[{} x {{ ptr, ptr }}] [{}]", entries.len(), entries.join(", "))
        };
        module.global(format!("@{rt}ffi_functions = constant {table}"));
        module.global(format!("@{rt}ffi_function_count = constant i32 {}", ffi.functions.len()));
        module.global(format!("@{rt}ffi_site_count = constant i32 {}", ffi.sites.len()));
        ember_codegen_c::FfiInstrumentOutput {
            source: ember_codegen_c::ffi_instrument_source(),
            functions: ffi.functions.into_iter().map(|(name, _)| name).collect(),
            sites: ffi.sites,
        }
    });
    let ir = assemble(&mut module, options);
    let shim = (!module.shims.is_empty() || !module.shim_functions.is_empty()).then(|| {
        let mut source = runtime::shim_source(&module.shims, &module.shim_namespace);
        for function in module.shim_functions.values() {
            source.push_str(function);
        }
        source
    });
    let optimization_report = std::mem::take(&mut module.direct_calls).into_iter().map(|(_, _, line)| line).collect();
    let visualisers = visualisers::render(&module.debug.shown);
    Ok(Output { ir, shim, optimization_report, visualisers, ffi_instrument, units })
}

/// `[BLD-4]` — one module's unit: its IR, and what the main module gathers of it (the runtime's
/// inline functions it calls through the shim, its report lines, its visualised types).
struct Unit {
    name: String,
    ir: String,
    shims: BTreeSet<&'static str>,
    shim_functions: BTreeMap<String, String>,
    direct_calls: BTreeSet<(ember_span::FileId, u32, String)>,
    shown: Vec<debug::Shown>,
}

/// The unit `name`, defining the bodies `symbols` names, written from `template`'s view of the
/// program.
fn write_unit<'a>(template: &Module<'a>, name: &str, symbols: &BTreeSet<String>, bodies: &'a [Body], options: &Options<'_>) -> Result<Unit> {
    let mut unit = template.fork(options);
    let defined = define_bodies(&mut unit, bodies, |body| symbols.contains(&body.symbol))?;
    glue::emit_pending(&mut unit)?;
    finish_definitions(&mut unit, &defined)?;
    let ir = assemble(&mut unit, options);
    Ok(Unit {
        name: name.to_string(),
        ir,
        shims: unit.shims,
        shim_functions: unit.shim_functions,
        direct_calls: unit.direct_calls,
        shown: unit.debug.shown,
    })
}

/// Writes each body `wanted` picks that has a definition, and returns their symbols.
fn define_bodies<'a>(m: &mut Module<'a>, bodies: &'a [Body], wanted: impl Fn(&Body) -> bool) -> Result<BTreeSet<&'a str>> {
    let mut defined = BTreeSet::new();
    for body in bodies {
        if body.is_abstract || body.is_extern_declaration || !wanted(body) {
            continue;
        }
        if body.ffi_counted.is_some() {
            ffi::emit_counted_wrapper(m, body)?;
        } else {
            func::emit_body(m, body)?;
        }
        defined.insert(body.symbol.as_str());
    }
    Ok(defined)
}

/// No declaration of a body the module defines; `[BLD-4]`, where the program is units, one of each
/// body its definitions name that another unit defines (a table's entry is named, never called).
fn finish_definitions(m: &mut Module<'_>, defined: &BTreeSet<&str>) -> Result<()> {
    if m.split {
        let names = m.body_names.clone();
        let mut named = BTreeSet::new();
        for text in std::iter::once(m.fns.as_str()).chain(m.globals.iter().map(String::as_str)) {
            let mut rest = text;
            while let Some(at) = rest.find('@') {
                let token = ir_name(&rest[at..]);
                if let Some(symbol) = names.get(token) {
                    named.insert(symbol.clone());
                }
                rest = &rest[at + token.len().max(1)..];
            }
        }
        for symbol in named {
            if !defined.contains(symbol.as_str()) && m.bodies.get(symbol.as_str()).is_some_and(|body| !body.is_abstract) {
                func::declare_body_symbol(m, &symbol)?;
            }
        }
    }
    for symbol in defined {
        m.decls.remove(&link_symbol(symbol));
    }
    Ok(())
}

/// The IR name at the start of `text` (`@name`, or `@"…"` quoted), `@` included.
fn ir_name(text: &str) -> &str {
    let bytes = text.as_bytes();
    if bytes.get(1) == Some(&b'"') {
        return match text[2..].find('"') {
            Some(end) => &text[..end + 3],
            None => text,
        };
    }
    let end = text[1..].find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$' | '-'))).map_or(text.len(), |end| end + 1);
    &text[..end]
}

/// `[BLD-4]` — a unit's own definitions (`internal`: its bodies, and the glue, tables and statics
/// it made for itself, as every unit that needs one makes them) one program-wide each: `weak_odr`,
/// in a COMDAT of its own name where the object format has them, so the link keeps one, whose
/// address every unit names, and drops none another unit calls (`linkonce_odr` may).
fn one_definition(ir: &str) -> String {
    let comdats = !cfg!(target_os = "macos");
    let mut out = String::with_capacity(ir.len() + ir.len() / 8);
    let mut groups = Vec::new();
    for line in ir.lines() {
        if let Some(rest) = line.strip_prefix("define internal ") {
            let mut header = format!("define weak_odr {rest}");
            if comdats && let Some(at) = rest.find('@') {
                groups.push(ir_name(&rest[at..]).to_string());
                let place = header.rfind(" !dbg ").or_else(|| header.rfind(" {")).unwrap_or(header.len());
                header.insert_str(place, " comdat");
            }
            out.push_str(&header);
        } else if line.starts_with('@') && line.contains(" = internal ") {
            let name = ir_name(line);
            let mut global = line.replacen(" = internal ", " = weak_odr ", 1);
            if comdats {
                groups.push(name.to_string());
                match global.rfind(", align ") {
                    Some(place) => global.insert_str(place, ", comdat"),
                    None => global.push_str(", comdat"),
                }
            }
            out.push_str(&global);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    if groups.is_empty() {
        return out;
    }
    let mut comdat = String::new();
    for name in groups {
        let _ = writeln!(comdat, "${} = comdat any", &name[1..]);
    }
    // After the header's lines (comment, source file, triple), before the first definition.
    let at = out.find("\n\n").map_or(0, |at| at + 2);
    out.insert_str(at, &comdat);
    out
}

/// The module's IR text.
fn assemble(module: &mut Module<'_>, options: &Options<'_>) -> String {
    let mut ir = String::new();
    let _ = writeln!(ir, "; Generated by the Ember compiler. Do not edit.");
    let _ = writeln!(ir, "; module: {}", options.module_name);
    let _ = writeln!(ir, "source_filename = \"{}\"", escape(options.module_name.as_bytes()));
    if let Some(triple) = &options.triple {
        let _ = writeln!(ir, "target triple = \"{}\"", escape(triple.as_bytes()));
    }
    ir.push('\n');
    if module.reload.is_some() {
        ir.push_str(&reload::type_declarations());
        ir.push('\n');
    }
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
    module.debug.write(options.target, &mut ir);
    if module.split { one_definition(&ir) } else { ir }
}
