//! The C11 backend: MIR to C (Part XVIII §6).
//!
//! `[CG-C-1]` — the emitted C must compile warning-free under
//! `-std=c11 -Wall -Wextra` and `/W3`, be free of undefined behaviour by
//! construction, and depend on no compiler extension except through
//! `ember_rt.h` macros that have portable fallbacks.
//!
//! `[CG-C-2]` — the output is deterministic for identical input: ordering is
//! stable and nothing is keyed on a pointer, so the build cache and
//! `diff`-based review both work.

use std::fmt::Write as _;

// `[RT-5]` — the emitted C's runtime calls carry this prefix, written in
// one place for the whole workspace. `{RT}` in a format string below
// resolves to it through Rust's implicit format arguments.
use ember_branding::RUNTIME_PREFIX as RT;
use ember_mir::{
    AggregateKind, AssertKind, Body, Builtin, CastKind, Const, FuncRef, LocalKind, Operand, Place,
    ParameterMode, Projection, RETURN_LOCAL, Rvalue, Stmt, StmtKind, Terminator,
};
use ember_mir::verify::VerifiedMir;
use std::path::MAIN_SEPARATOR;

use ember_span::{SourceMap, Symbol};
use ember_types::{ClassId, EnumId, FloatTy, FnParam, FnParamMode, IntTy, Niche, StructId, Ty, TyKind, TypeTable, UintTy};
use std::collections::{BTreeMap, BTreeSet};

pub struct Output {
    /// The single translation unit for this module.
    pub c_source: String,
    /// `[EFF-10]` — per-site safety metadata for the checks this translation
    /// unit emits. The driver writes this to the profile's inspect directory;
    /// elided-check entries are added when an elision pass provides the
    /// required proof and reason; unsupported cases remain emitted checks.
    pub safety_json: String,
}

/// `[MNG-1]` — the C symbol of a type's `drop` method.
///
/// Must agree with `ember_typeck::method_symbol` exactly; the two are separated
/// by a crate boundary and nothing checks that they match, so a change to one
/// silently stops the destructor being found by the other. The
/// `tests/conformance/OWN-2/` cases are what would notice.
fn drop_symbol(owner: &str) -> String {
    let owner: String = owner
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    ember_branding::mangled(&format!("{}_drop", owner.trim_matches('_')))
}

fn class_drop_fields_symbol(owner: &str) -> String {
    let owner: String = owner
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    ember_branding::mangled(&format!("{}_drop_fields", owner.trim_matches('_')))
}

fn class_drop_adapter_symbol(owner: &str) -> String {
    let owner: String = owner
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    ember_branding::mangled(&format!("{}_drop_adapter", owner.trim_matches('_')))
}

fn class_debug_edges_symbol(owner: &str) -> String {
    let owner: String = owner
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    ember_branding::mangled(&format!("{}_debug_edges", owner.trim_matches('_')))
}

/// A C-table identity for one ordered `dyn` bound list. Single-interface
/// identities retain their existing spelling; a composed carrier gets its own
/// table rather than pretending its first bound describes every slot.
fn dyn_table_identity(interfaces: &[Symbol]) -> String {
    match interfaces {
        [interface] => interface.to_string(),
        _ => format!(
            "multi__{}",
            interfaces.iter().map(ToString::to_string).collect::<Vec<_>>().join("__")
        ),
    }
}

/// `[DSP-3]` — each compiler-known interface is identified in the runtime
/// table by the address of a translation-unit static marker. The address is
/// opaque to Ember code; it only needs stable identity within this emitted
/// program, not a source-visible numeric representation.
fn interface_id_symbol(interface: &str) -> String {
    ember_branding::mangled(&format!("interface_id_{interface}"))
}

fn class_itable_symbol(class: &str) -> String {
    ember_branding::mangled(&format!("itables_{class}"))
}

pub fn emit(
    mir: VerifiedMir<'_>,
    map: &SourceMap,
    module_name: &str,
    has_main: bool,
    leak_check: bool,
) -> Output {
    let bodies = mir.bodies();
    let types = mir.types();
    let (order, structural) = plan_types(types);
    let usize_ty = types
        .all()
        .find(|(_, k)| matches!(k, TyKind::Uint(UintTy::Usize)))
        .map(|(ty, _)| ty)
        .or_else(|| types.all().next().map(|(ty, _)| ty))
        .expect("the interner always holds the common types");
    let mut emitter = Emitter {
        types,
        map,
        out: String::new(),
        line_directives: true,
        order,
        structural,
        usize_ty,
        virtual_tables: BTreeMap::new(),
        virtual_signatures: BTreeMap::new(),
        interface_layouts: BTreeMap::new(),
        interface_adapters: BTreeMap::new(),
        class_interface_tables: BTreeMap::new(),
        class_interface_call_interfaces: BTreeSet::new(),
        interface_caches: BTreeMap::new(),
        direct_param_modes: bodies
            .iter()
            .map(|body| (body.symbol.clone(), body.param_modes.clone()))
            .collect(),
        drop_glue: std::cell::RefCell::new(Vec::new()),
        eq_fns: std::cell::RefCell::new(Vec::new()),
        clone_parts_fns: std::cell::RefCell::new(Vec::new()),
        fmt_fns: std::cell::RefCell::new(Vec::new()),
        fmt_prints: std::cell::RefCell::new(Vec::new()),
        fmt_specs: std::cell::RefCell::new(Vec::new()),
        array_helpers: std::cell::RefCell::new(Vec::new()),
        clone_fns: bodies
            .iter()
            .filter(|body| {
                body.arg_count == 1
                    && body.symbol.ends_with("_clone")
                    && body.locals.get(1).is_some_and(|local| local.name.as_deref() == Some("self"))
            })
            .filter_map(|body| {
                let receiver = body.locals[1].ty;
                let (owner, by_address) = match types.kind(receiver) {
                    TyKind::Ref { inner, .. } => (*inner, true),
                    _ => (receiver, false),
                };
                (body.return_ty() == owner).then(|| (owner, (body.symbol.clone(), by_address)))
            })
            .collect(),
    };
    emitter.emit_module(bodies, module_name, has_main, leak_check);
    Output {
        c_source: emitter.out,
        safety_json: safety_json(bodies, map),
    }
}

/// Render the `[EFF-10]` side table. Dynamic class exclusivity checks are
/// explicit `BeginAccess` MIR statements, while proven elisions are retained
/// as compiler metadata on the body. The JSON is deliberately written without
/// a serialization dependency: this artifact is a compiler output with a
/// fixed, small schema rather than a user-facing data model.
fn safety_json(bodies: &[Body], map: &SourceMap) -> String {
    let mut entries = Vec::new();
    for body in bodies {
        for (block_index, block) in body.blocks.iter().enumerate() {
            for (statement_index, stmt) in block.stmts.iter().enumerate() {
                if !matches!(stmt.kind, StmtKind::BeginAccess { .. } | StmtKind::BeginAccessTransfer { .. }) {
                    continue;
                }
                if body.hoisted_accesses.iter().any(|record| {
                    record.preheader.0 as usize == block_index
                        && record.preheader_statement == statement_index
                }) {
                    continue;
                }
                let location = map.location(stmt.span);
                entries.push(format!(
                    "{{\"kind\":\"Aliasing\",\"source\":{},\"function\":{},\"mechanism\":\"dynamic exclusivity\",\"classification\":\"DYNAMIC_PER_ACCESS\",\"reason\":\"not_proven_by_analysis\",\"status\":\"emitted\"}}",
                    json_string(&location),
                    json_string(&body.name),
                ));
            }
        }
        for hoisted in &body.hoisted_accesses {
            entries.push(format!(
                "{{\"kind\":\"Aliasing\",\"source\":{},\"function\":{},\"mechanism\":\"dynamic exclusivity\",\"classification\":\"DYNAMIC_HOISTED_LOOP\",\"reason\":\"inherent_to_mechanism\",\"proof\":{},\"loop\":{},\"check_site\":\"preheader\",\"protected_interval\":\"loop\",\"status\":\"emitted\"}}",
                json_string(&map.location(hoisted.span)),
                json_string(&body.name),
                json_string(hoisted.proof.as_str()),
                json_string(&map.location(hoisted.loop_span)),
            ));
        }
        for elided in &body.elided_accesses {
            entries.push(format!(
                "{{\"kind\":\"Aliasing\",\"source\":{},\"function\":{},\"mechanism\":\"static exclusivity\",\"classification\":\"STATIC_ELIDED\",\"reason\":{},\"status\":\"elided\"}}",
                json_string(&map.location(elided.span)),
                json_string(&body.name),
                json_string(elided.reason.as_str()),
            ));
        }
    }
    format!("{{\"schema\":1,\"checks\":[{}]}}\n", entries.join(","))
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

struct Emitter<'a> {
    types: &'a TypeTable,
    map: &'a SourceMap,
    out: String,
    line_directives: bool,
    /// Every type definition to emit, in an order where a type is defined
    /// after everything it contains by value.
    order: Vec<TypeNode>,
    /// The generated C name of each tuple and fixed-array type.
    structural: BTreeMap<Ty, String>,
    /// `usize`, for the lengths inside an `Array[T]`. The emitter has no
    /// `CommonTypes`, so it finds the one the interner already holds.
    usize_ty: Ty,
    /// `[DSP-2]` — effective implementation for each class vtable slot.
    virtual_tables: BTreeMap<ClassId, Vec<Option<VirtualMethod>>>,
    /// The signature established by the first declaration in each hierarchy.
    virtual_signatures: BTreeMap<(ClassId, usize), VirtualMethod>,
    /// `[TYP-22]` — full declaration-order layouts observed at dynamic
    /// interface call sites. Interface declarations are type-checker data,
    /// so MIR carries this exact erased ABI fact across the backend boundary.
    interface_layouts: BTreeMap<String, Vec<Option<InterfaceMethod>>>,
    /// Concrete implementations observed at a checked borrowed coercion.
    /// These are compiler-owned adapter tables, not source-level virtual tables.
    interface_adapters: BTreeMap<(String, String), InterfaceAdapter>,
    /// `[DSP-3]` tables that a concrete class exposes through its TypeInfo.
    /// The key is an interface identity and the value retains the checked
    /// concrete type used to name the corresponding adapter table.
    class_interface_tables: BTreeMap<ClassId, BTreeMap<String, Ty>>,
    /// Interfaces read through a one-word class handle. A marker is needed
    /// even when the concrete adapter table came from another module.
    class_interface_call_interfaces: BTreeSet<String>,
    /// Per-body hidden caches for repeated interface calls through an
    /// unchanged class-interface parameter.
    interface_caches: BTreeMap<InterfaceCacheKey, String>,
    /// Direct-call ownership modes. A `Copy` operand passed to an `owned`
    /// parameter creates another class-handle owner, so its retain must be
    /// emitted before the call transfers that new owner to the callee.
    direct_param_modes: BTreeMap<String, Vec<ParameterMode>>,
    /// D-182 — aggregate types whose drop is emitted as an out-of-line
    /// `static` function, in first-request order. An `Array` element or a
    /// `Box` payload drops through one, so a type that owns itself through
    /// an indirection (`struct Tree: kids: Array[Tree]`) is a runtime
    /// recursion instead of an infinite inline expansion.
    drop_glue: std::cell::RefCell<Vec<Ty>>,
    /// D-187 — the aggregates `==` has been asked of, one generated
    /// field-wise equality function each, requested and emitted like
    /// `drop_glue`.
    eq_fns: std::cell::RefCell<Vec<Ty>>,
    /// `[STR-5]` — structural clone helpers that write through a destination
    /// pointer instead of returning a chain of by-value temporaries (D-360).
    clone_parts_fns: std::cell::RefCell<Vec<Ty>>,
    /// `[STD-15]` — the per-type helpers the Array methods call (a sort's
    /// comparison, `pop`, `remove`, `clear`, `sorted`), requested and emitted
    /// like `eq_fns`.
    array_helpers: std::cell::RefCell<Vec<(ArrayHelper, Ty)>>,
    /// `[OWN-8]` — each type's `clone` function (derived or written), found
    /// among the bodies, and whether it takes its receiver by address.
    clone_fns: BTreeMap<Ty, (String, bool)>,
    /// `[TYP-39]` — the aggregates printed or formatted, one generated
    /// `Display` function each; and those printed, which also get a wrapper
    /// that formats into a buffer and writes it.
    fmt_fns: std::cell::RefCell<Vec<Ty>>,
    fmt_prints: std::cell::RefCell<Vec<Ty>>,
    /// And those formatted with a spec after `!r`/`!s`, whose wrapper pads
    /// the text the conversion made.
    fmt_specs: std::cell::RefCell<Vec<Ty>>,
}

/// Replaced, once every body has been emitted, by the drop-glue prototypes.
const DROP_GLUE_PROTOTYPES: &str = "/* @@drop-glue-prototypes@@ */";

/// Replaced the same way by the D-187 equality-function prototypes.
const EQ_FN_PROTOTYPES: &str = "/* @@eq-fn-prototypes@@ */";
const CLONE_PARTS_PROTOTYPES: &str = "/* @@clone-parts-prototypes@@ */";

/// And by the `[STD-15]` Array helpers' prototypes.
const ARRAY_HELPER_PROTOTYPES: &str = "/* @@array-helper-prototypes@@ */";

/// And by the `[TYP-39]` formatting functions' prototypes.
const FMT_FN_PROTOTYPES: &str = "/* @@fmt-fn-prototypes@@ */";

/// One per-type helper an Array method calls. `Pop` is keyed by the
/// `Option[T]` it returns, the rest by the element type.
#[derive(Copy, Clone, PartialEq, Eq)]
enum ArrayHelper {
    Less,
    Pop,
    Remove,
    /// `[STD-15]` (ODR-031) — `drain`: move `lo..hi` out into a new buffer
    /// and close the gap.
    Drain,
    /// `[STD-15]` — `swap_remove`: the last element fills the gap.
    SwapRemove,
    /// `[STD-15]` — `truncate`: drop the elements from `n` on.
    Truncate,
    /// `[STD-15]` — `extend`: append a clone of each element of a span.
    Extend,
    Clear,
    Sorted,
    /// `[OWN-8]` — a new buffer holding a clone of each element.
    Clone,
}

#[derive(Clone)]
struct VirtualMethod {
    owner: ClassId,
    symbol: String,
    params: Vec<Ty>,
    ret: Ty,
    is_abstract: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InterfaceMethod {
    params: Vec<Ty>,
    ret: Ty,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InterfaceAdapterMethod {
    symbol: String,
    receiver: ParameterMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InterfaceAdapter {
    concrete: Ty,
    interface: String,
    layout: Vec<Option<InterfaceMethod>>,
    implementations: Vec<Option<InterfaceAdapterMethod>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct InterfaceCacheKey {
    local: usize,
    interface: String,
}

/// Where a projection walk has reached: a type, plus the variant a
/// `Downcast` selected, which the `Field` after it needs.
#[derive(Clone, Copy)]
struct Cursor {
    ty: Ty,
    variant: Option<usize>,
}

/// How one type is written in C: a `struct` with members, or a name for
/// another type.
enum Definition {
    Struct(Vec<String>),
    Alias(String),
    /// `typedef R (*name)(A, B);` — a function pointer, whose declarator wraps
    /// the name rather than preceding it.
    FnPointer { ret: String, args: String },
}

/// A C type definition: either a named Ember struct or a generated struct
/// standing in for a tuple or a fixed array.
///
/// C has no tuple, and a bare C array cannot be assigned, passed or returned
/// by value — but an Ember tuple and an Ember `[T; N]` are ordinary values
/// (Part IV.3). Wrapping both in a generated struct gives them C's value
/// semantics for free, with the same layout, and leaves every copy, argument
/// and return path in this backend unchanged.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TypeNode {
    Struct(StructId),
    Class(ClassId),
    Enum(EnumId),
    Structural(Ty),
}

/// `[CG-C-11]` (D-325) — every translation unit turns off floating-point
/// contraction (`[TYP-9]`): `a * b + c` is two roundings, never a fused
/// multiply-add the C compiler chose. clang takes the standard pragma, MSVC
/// its own spelling; gcc does not implement the pragma and warns about it
/// under `-Wall`, so it has `-ffp-contract=off` alone (ODR-044). The runtime's
/// own source begins the same way.
pub const FP_STRICT: [&str; 8] = [
    "/* [CG-C-11] No floating-point contraction ([TYP-9]). */",
    "#if defined(__clang__)",
    "#pragma STDC FP_CONTRACT OFF",
    "#elif defined(_MSC_VER)",
    "#pragma fp_contract(off)",
    "#endif",
    "/* gcc: -ffp-contract=off, which the build passes; it ignores the pragma",
    " * with a warning. */",
];

impl Emitter<'_> {
    fn line(&mut self, text: &str) {
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn emit_module(
        &mut self,
        bodies: &[Body],
        module_name: &str,
        has_main: bool,
        leak_check: bool,
    ) {
        self.line("/* Generated by the Ember compiler. Do not edit. */");
        self.line(&format!("/* module: {module_name} */"));
        self.line("");
        for line in FP_STRICT {
            self.line(line);
        }
        self.line("");
        let header = ember_branding::runtime_header();
        self.line(&format!("#include \"{header}\""));
        self.line("");

        self.emit_type_declarations();
        self.collect_virtual_methods(bodies);
        self.collect_interface_methods(bodies);
        self.collect_interface_adapters(bodies);
        self.emit_interface_vtable_types();
        self.emit_prototypes(bodies);
        self.line(DROP_GLUE_PROTOTYPES);
        self.line(EQ_FN_PROTOTYPES);
        self.line(CLONE_PARTS_PROTOTYPES);
        self.line(ARRAY_HELPER_PROTOTYPES);
        self.line(FMT_FN_PROTOTYPES);
        self.emit_interface_adapters();
        self.emit_virtual_tables();
        self.emit_class_drop_adapters();
        self.emit_class_field_drop_glue();
        // `[WK-8]` is an opt-in diagnostic mode.  Do not grow ordinary
        // generated programs with unused debug callbacks or registrations.
        if leak_check {
            self.emit_debug_edge_enumerators();
        }
        self.emit_class_type_infos();
        self.emit_shared_type_infos();

        for body in bodies {
            if body.is_abstract {
                continue;
            }
            self.emit_body(body);
        }

        if has_main {
            self.emit_entry_point(leak_check);
        }
        self.emit_fmt_fns();
        self.collect_clone_dependencies();
        // The Array helpers first: `clear` drops through glue it may request.
        self.emit_array_helpers();
        self.emit_clone_parts_fns();
        self.emit_drop_glue();
        self.emit_eq_fns();
    }

    /// The symbol of a structural clone helper, requesting it.
    fn clone_parts_helper(&self, ty: Ty) -> String {
        let mut helpers = self.clone_parts_fns.borrow_mut();
        let index = helpers.iter().position(|&known| known == ty).unwrap_or_else(|| {
            helpers.push(ty);
            helpers.len() - 1
        });
        ember_branding::mangled(&format!("clone_parts_{index}"))
    }

    /// Discover the helpers before defining either family: an Array helper
    /// may clone an Option, and an Option helper may clone an Array.
    fn collect_clone_dependencies(&self) {
        let mut scanned_arrays = 0;
        let mut scanned_parts = 0;
        loop {
            let arrays: Vec<(ArrayHelper, Ty)> = self.array_helpers.borrow()[scanned_arrays..].to_vec();
            let parts: Vec<Ty> = self.clone_parts_fns.borrow()[scanned_parts..].to_vec();
            if arrays.is_empty() && parts.is_empty() { break; }
            scanned_arrays += arrays.len();
            scanned_parts += parts.len();
            for (helper, ty) in arrays {
                if matches!(helper, ArrayHelper::Clone | ArrayHelper::Extend) {
                    self.request_clone_dependency(ty);
                }
            }
            for ty in parts {
                match self.types.kind(ty) {
                    TyKind::Tuple(items) => {
                        for &item in items { self.request_clone_dependency(item); }
                    }
                    TyKind::Enum(id) => {
                        for variant in &self.types.enum_def(*id).variants {
                            for field in &variant.fields { self.request_clone_dependency(field.ty); }
                        }
                    }
                    _ => unreachable!("only structural types request a clone-parts helper"),
                }
            }
        }
    }

    fn request_clone_dependency(&self, ty: Ty) {
        if self.types.is_copy(ty) || self.clone_fns.contains_key(&ty) { return; }
        match self.types.kind(ty) {
            TyKind::Vec { elem, .. } => { self.array_helper(ArrayHelper::Clone, *elem); }
            TyKind::Tuple(_) | TyKind::Enum(_) => { self.clone_parts_helper(ty); }
            _ => {}
        }
    }

    /// Clone one field from stable source storage into its final destination.
    /// Structural children get their own pointer-taking helper, so the C
    /// stack never holds a by-value copy of every enclosing aggregate.
    fn clone_part_lines(&self, dest: &str, source: &str, ty: Ty, out: &mut Vec<String>) {
        if self.types.is_copy(ty) {
            out.push(format!("{dest} = {source};"));
            self.retain_lines_for_value(dest, ty, out);
        } else if let TyKind::Vec { elem, .. } = self.types.kind(ty) {
            let helper = self.array_helper(ArrayHelper::Clone, *elem);
            out.push(format!("{dest} = {helper}(({source}).ptr, ({source}).len);"));
        } else if let Some((symbol, by_address)) = self.clone_fns.get(&ty) {
            let argument = if *by_address { format!("&({source})") } else { source.to_string() };
            out.push(format!("{dest} = {symbol}({argument});"));
        } else if matches!(self.types.kind(ty), TyKind::Tuple(_) | TyKind::Enum(_)) {
            let helper = self.clone_parts_helper(ty);
            out.push(format!("{helper}(&({source}), &({dest}));"));
        } else {
            unreachable!("the type checker proved this field cloneable")
        }
    }

    fn emit_clone_parts_fns(&mut self) {
        let mut emitted = 0;
        let mut prototypes = Vec::new();
        loop {
            let pending: Vec<Ty> = self.clone_parts_fns.borrow()[emitted..].to_vec();
            if pending.is_empty() { break; }
            for ty in pending {
                let symbol = ember_branding::mangled(&format!("clone_parts_{emitted}"));
                let c_ty = self.c_type(ty);
                let signature = format!("static void {symbol}(const {c_ty}* src, {c_ty}* dst)");
                prototypes.push(format!("{signature};"));
                self.line(&format!("{signature} {{"));
                match self.types.kind(ty).clone() {
                    TyKind::Tuple(items) => {
                        for (index, item) in items.into_iter().enumerate() {
                            let mut lines = Vec::new();
                            self.clone_part_lines(&format!("dst->_{index}"), &format!("src->_{index}"), item, &mut lines);
                            for line in lines { self.line(&format!("    {line}")); }
                        }
                    }
                    TyKind::Enum(id) => {
                        let def = self.types.enum_def(id).clone();
                        self.line(&format!("    switch ({}) {{", self.enum_tag(id, "(*src)")));
                        for (variant_index, variant) in def.variants.iter().enumerate() {
                            self.line(&format!("    case {}:", variant.discriminant));
                            self.line("        memset(dst, 0, sizeof *dst);");
                            if self.types.option_niche(id).is_none() {
                                self.line(&format!("        dst->tag = {};", variant.discriminant));
                            }
                            for (field_index, field) in variant.fields.iter().enumerate() {
                                let source = self.enum_member(id, "(*src)", variant_index, field_index);
                                let dest = self.enum_member(id, "(*dst)", variant_index, field_index);
                                let mut lines = Vec::new();
                                self.clone_part_lines(&dest, &source, field.ty, &mut lines);
                                for line in lines { self.line(&format!("        {line}")); }
                            }
                            self.line("        return;");
                        }
                        self.line("    default: EMBER_UNREACHABLE();");
                        self.line("    }");
                    }
                    _ => unreachable!("CloneParts is requested only for tuples and payload enums"),
                }
                self.line("}");
                self.line("");
                emitted += 1;
            }
        }
        self.out = self.out.replacen(CLONE_PARTS_PROTOTYPES, &prototypes.join("\n"), 1);
    }

    fn array_helper(&self, helper: ArrayHelper, ty: Ty) -> String {
        let mut helpers = self.array_helpers.borrow_mut();
        let index = helpers.iter().position(|&known| known == (helper, ty)).unwrap_or_else(|| {
            helpers.push((helper, ty));
            helpers.len() - 1
        });
        ember_branding::mangled(&format!("array_helper_{index}"))
    }

    /// `a < b` for two elements behind `const void*`, in `Ord`'s order:
    /// totalOrder for floats (`[TYP-37]`), bytes for text.
    fn less_behind_pointers(&self, ty: Ty) -> String {
        if self.is_void(ty) {
            return "((void)a, (void)b, 0)".to_string();
        }
        if let Some(suffix) = self.wide_int(ty) {
            return format!("{RT}{suffix}_lt(*(const {RT}{suffix}*)a, *(const {RT}{suffix}*)b)");
        }
        match self.types.kind(ty) {
            TyKind::Float(FloatTy::F64) => format!("{RT}total_lt_f64(*(const double*)a, *(const double*)b)"),
            TyKind::Float(FloatTy::F16) => format!("{RT}total_lt_f16(*(const uint16_t*)a, *(const uint16_t*)b)"),
            TyKind::Float(_) => format!("{RT}total_lt_f32(*(const float*)a, *(const float*)b)"),
            TyKind::Str => format!("{RT}str_cmp(*(const {RT}str*)a, *(const {RT}str*)b) < 0"),
            TyKind::Vec { .. } => format!(
                "{RT}str_cmp({RT}vec_as_str((const {RT}vec*)a), {RT}vec_as_str((const {RT}vec*)b)) < 0"
            ),
            _ => {
                let c = self.c_type(ty);
                format!("*(const {c}*)a < *(const {c}*)b")
            }
        }
    }

    /// Define every requested Array helper, to a fixpoint (`sorted` asks for
    /// a comparison), and put their prototypes where the marker stands.
    fn emit_array_helpers(&mut self) {
        let mut emitted = 0;
        let mut prototypes = Vec::new();
        loop {
            let pending: Vec<(ArrayHelper, Ty)> = self.array_helpers.borrow()[emitted..].to_vec();
            if pending.is_empty() {
                break;
            }
            for (helper, ty) in pending {
                let symbol = ember_branding::mangled(&format!("array_helper_{emitted}"));
                let (signature, body) = match helper {
                    ArrayHelper::Less => (
                        format!("static bool {symbol}(const void* a, const void* b)"),
                        vec![format!("return {};", self.less_behind_pointers(ty))],
                    ),
                    ArrayHelper::Pop => {
                        let TyKind::Enum(id) = *self.types.kind(ty) else {
                            unreachable!("`pop` returns an Option")
                        };
                        let def = self.types.enum_def(id).clone();
                        let (none, some) = (&def.variants[0], &def.variants[1]);
                        let elem = some.fields[0].ty;
                        let (option, elem_c) = (self.c_type(ty), self.c_type(elem));
                        let last = format!("(unsigned char*)v->ptr + v->len * {elem_c_size}, {elem_c_size}", elem_c_size = c_size(&elem_c));
                        // `[TYP-13]` — a niche `Option` is its payload, and
                        // its `None` is zero bytes.
                        let body = if self.types.option_niche(id).is_some() {
                            vec![
                                format!("{option} r;"),
                                "memset(&r, 0, sizeof r);".to_string(),
                                "if (v->len == 0) return r;".to_string(),
                                "v->len -= 1;".to_string(),
                                format!("memcpy(&r, {last});"),
                                "return r;".to_string(),
                            ]
                        } else {
                            vec![
                                format!("{option} r;"),
                                "memset(&r, 0, sizeof r);".to_string(),
                                format!("if (v->len == 0) {{ r.tag = {}; return r; }}", none.discriminant),
                                "v->len -= 1;".to_string(),
                                format!("memcpy(&r.payload.{}.{}, {last});", some.name, some.fields[0].name),
                                format!("r.tag = {};", some.discriminant),
                                "return r;".to_string(),
                            ]
                        };
                        (format!("static {option} {symbol}({RT}vec* v)"), body)
                    }
                    // A zero-sized element is nothing to take out (D-355).
                    ArrayHelper::Remove | ArrayHelper::SwapRemove if self.is_void(ty) => (
                        format!("static void {symbol}({RT}vec* v, size_t i)"),
                        vec!["(void)i;".to_string(), "v->len -= 1;".to_string()],
                    ),
                    ArrayHelper::Remove => {
                        let c = self.c_type(ty);
                        (
                            format!("static {c} {symbol}({RT}vec* v, size_t i)"),
                            vec![
                                format!("{c} r;"),
                                format!("unsigned char* at = (unsigned char*)v->ptr + i * {c_size};", c_size = c_size(&c)),
                                format!("memcpy(&r, at, {c_size});", c_size = c_size(&c)),
                                format!("memmove(at, at + {c_size}, (v->len - i - 1) * {c_size});", c_size = c_size(&c)),
                                "v->len -= 1;".to_string(),
                                "return r;".to_string(),
                            ],
                        )
                    }
                    // The elements are moved out bitwise into the new buffer,
                    // so neither array drops them. An empty range returns
                    // before any pointer arithmetic: an empty array's buffer
                    // is null.
                    ArrayHelper::Drain => {
                        let c = self.c_type(ty);
                        (
                            format!("static {RT}vec {symbol}({RT}vec* v, size_t lo, size_t hi)"),
                            vec![
                                format!("if (lo == hi) {{ return {RT}vec_empty(); }}"),
                                format!("unsigned char* at = (unsigned char*)v->ptr + lo * {c_size};", c_size = c_size(&c)),
                                format!("{RT}vec r = {RT}vec_from_elems({c_size}, at, hi - lo);", c_size = c_size(&c)),
                                format!("memmove(at, at + (hi - lo) * {c_size}, (v->len - hi) * {c_size});", c_size = c_size(&c)),
                                "v->len -= hi - lo;".to_string(),
                                "return r;".to_string(),
                            ],
                        )
                    }
                    ArrayHelper::SwapRemove => {
                        let c = self.c_type(ty);
                        (
                            format!("static {c} {symbol}({RT}vec* v, size_t i)"),
                            vec![
                                format!("{c} r;"),
                                format!("unsigned char* at = (unsigned char*)v->ptr + i * {c_size};", c_size = c_size(&c)),
                                format!("memcpy(&r, at, {c_size});", c_size = c_size(&c)),
                                "v->len -= 1;".to_string(),
                                format!("memmove(at, (unsigned char*)v->ptr + v->len * {c_size}, {c_size});", c_size = c_size(&c)),
                                "return r;".to_string(),
                            ],
                        )
                    }
                    ArrayHelper::Truncate => {
                        let c = self.c_type(ty);
                        let mut drops = Vec::new();
                        self.drop_lines(&format!("(({c}*)v->ptr)[_ci]"), ty, &mut drops);
                        let mut body = vec!["if (n >= v->len) { return; }".to_string()];
                        if !drops.is_empty() {
                            body.push(format!("for (size_t _ci = n; _ci < v->len; ++_ci) {{ {} }}", drops.join(" ")));
                        }
                        body.push("v->len = n;".to_string());
                        (format!("static void {symbol}({RT}vec* v, size_t n)"), body)
                    }
                    ArrayHelper::Extend if self.is_void(ty) => (
                        format!("static void {symbol}({RT}vec* v, const void* elems, size_t count)"),
                        vec![
                            "(void)elems;".to_string(),
                            format!("{RT}vec_reserve(v, 0, v->len + count);"),
                            "v->len += count;".to_string(),
                        ],
                    ),
                    ArrayHelper::Extend => {
                        let c = self.c_type(ty);
                        let mut body = vec![
                            format!("{RT}vec_reserve(v, {c_size}, v->len + count);", c_size = c_size(&c)),
                            format!("{c}* to = ({c}*)v->ptr + v->len;"),
                            format!("memcpy(to, elems, count * {c_size});", c_size = c_size(&c)),
                        ];
                        let copy = "to[_ci]".to_string();
                        let source = format!("(({c}*)elems)[_ci]");
                        let per_element = if self.types.is_copy(ty) {
                            // A class handle (or a value holding one) is `Copy`
                            // but its copy is a retain.
                            let mut retains = Vec::new();
                            self.retain_lines_for_value(&copy, ty, &mut retains);
                            retains.join(" ")
                        } else {
                            let mut lines = Vec::new();
                            self.clone_part_lines(&copy, &source, ty, &mut lines);
                            lines.join(" ")
                        };
                        if !per_element.is_empty() {
                            body.push(format!("for (size_t _ci = 0; _ci < count; ++_ci) {{ {per_element} }}"));
                        }
                        body.push("v->len += count;".to_string());
                        (format!("static void {symbol}({RT}vec* v, const void* elems, size_t count)"), body)
                    }
                    ArrayHelper::Clear => {
                        let c = self.c_type(ty);
                        let mut drops = Vec::new();
                        self.drop_lines(&format!("(({c}*)v->ptr)[_ci]"), ty, &mut drops);
                        let mut body = Vec::new();
                        if !drops.is_empty() {
                            body.push(format!("for (size_t _ci = 0; _ci < v->len; ++_ci) {{ {} }}", drops.join(" ")));
                        }
                        body.push("v->len = 0;".to_string());
                        (format!("static void {symbol}({RT}vec* v)"), body)
                    }
                    ArrayHelper::Clone => {
                        let c = self.c_type(ty);
                        let mut body = vec![format!("{RT}vec r = {RT}vec_from_elems({c_size}, elems, count);", c_size = c_size(&c))];
                        let copy = format!("(({c}*)r.ptr)[_ci]");
                        let source = format!("(({c}*)elems)[_ci]");
                        let per_element = if self.types.is_copy(ty) {
                            // A class handle (or a value holding one) is `Copy`
                            // but its copy is a retain.
                            let mut retains = Vec::new();
                            self.retain_lines_for_value(&copy, ty, &mut retains);
                            retains.join(" ")
                        } else {
                            let mut lines = Vec::new();
                            self.clone_part_lines(&copy, &source, ty, &mut lines);
                            lines.join(" ")
                        };
                        if !per_element.is_empty() {
                            body.push(format!("for (size_t _ci = 0; _ci < count; ++_ci) {{ {per_element} }}"));
                        }
                        body.push("return r;".to_string());
                        (format!("static {RT}vec {symbol}(const void* elems, size_t count)"), body)
                    }
                    ArrayHelper::Sorted => {
                        let c = self.c_type(ty);
                        let less = self.array_helper(ArrayHelper::Less, ty);
                        (
                            format!("static {RT}vec {symbol}(const void* elems, size_t count)"),
                            vec![
                                format!("{RT}vec r = {RT}vec_from_elems({c_size}, elems, count);", c_size = c_size(&c)),
                                format!("{RT}vec_sort(&r, {c_size}, {less});", c_size = c_size(&c)),
                                "return r;".to_string(),
                            ],
                        )
                    }
                };
                prototypes.push(format!("{signature};"));
                self.line(&format!("{signature} {{"));
                for line in body {
                    self.line(&format!("    {line}"));
                }
                self.line("}");
                self.line("");
                emitted += 1;
            }
        }
        self.out = self.out.replacen(ARRAY_HELPER_PROTOTYPES, &prototypes.join("\n"), 1);
    }

    /// D-187 — a C expression, true when `a` and `b` (values of `ty`) are
    /// equal: IEEE `==` on floats, identity on class handles, bytes on text
    /// and `[STR-5]`'s field-wise equality on aggregates.
    fn eq_expr(&self, a: &str, b: &str, ty: Ty) -> String {
        // `void` has one value (`[TYP-36]`); a zero-sized element has no
        // storage to read (D-355).
        if self.is_void(ty) {
            return "1".to_string();
        }
        if let Some((a, b)) = self.text_pair(a, b, ty) {
            return format!("({RT}str_cmp({a}, {b}) == 0)");
        }
        if let Some(suffix) = self.wide_int(ty) {
            return format!("{RT}{suffix}_eq({a}, {b})");
        }
        // D-316 — IEEE equality: `-0.0 == 0.0`, and NaN equals nothing.
        if self.half(ty) {
            return format!("({RT}f16_to_f64({a}) == {RT}f16_to_f64({b}))");
        }
        match self.types.kind(ty) {
            TyKind::Struct(_) | TyKind::Tuple(_) | TyKind::Array { .. } | TyKind::Vec { .. } => {}
            TyKind::Enum(id) if !self.types.enum_def(*id).is_unit_only() => {}
            _ => return format!("(({a}) == ({b}))"),
        }
        let mut fns = self.eq_fns.borrow_mut();
        let index = fns.iter().position(|&known| known == ty).unwrap_or_else(|| {
            fns.push(ty);
            fns.len() - 1
        });
        // The generated function only reads the values. Passing a deeply
        // nested aggregate by value copies its whole outer value at every
        // level and can exhaust a small C thread stack (D-360).
        format!("{}(&({a}), &({b}))", eq_fn_symbol(index))
    }

    /// `[TYP-39]` — a value whose `Display` is a generated function: an
    /// `Array` that is not a `String`, a view, a fixed array, a tuple, and
    /// the payload-carrying enums the checker lets through (`Option`,
    /// `Result`).
    fn is_display_aggregate(&self, ty: Ty) -> bool {
        match self.types.kind(ty) {
            TyKind::Vec { text, .. } => !text,
            TyKind::Span { .. } | TyKind::Array { .. } | TyKind::Tuple(_) => true,
            // `[STR-5]` — structs and enums have `Debug` field-wise; a
            // unit-only enum's `Display` is its variant name (`unit_display`).
            TyKind::Enum(_) | TyKind::Struct(_) => true,
            // `[TYP-36]` — a class handle has `Debug`: its class and address.
            TyKind::Class(_) | TyKind::ClassInterface(_) => true,
            _ => false,
        }
    }

    /// `[TYP-36]` — a unit-only enum's `Display`, its bare variant name, as
    /// a C `str` expression over the value `v`; `None` for any other type
    /// (whose `Display` is its `Debug` text).
    fn unit_display(&self, v: &str, ty: Ty) -> Option<String> {
        // `[TYP-36]` — `void` has no `Display`; printing shows its `Debug`,
        // `()` (D-355).
        if matches!(self.types.kind(ty), TyKind::Void) {
            return Some(format!("{RT}str_lit(\"()\", 2)"));
        }
        let TyKind::Enum(id) = self.types.kind(ty) else { return None };
        let def = self.types.enum_def(*id);
        if !def.is_unit_only() {
            return None;
        }
        let mut expr = format!("{RT}str_lit(\"\", 0)");
        for variant in def.variants.iter().rev() {
            let name = variant.name.as_str();
            expr = format!("(({v}) == {} ? {RT}str_lit(\"{name}\", {}) : {expr})", variant.discriminant, name.len());
        }
        Some(expr)
    }

    /// The name `Debug` shows for a user struct or enum: its own, without a
    /// module path or generic arguments (`Point`, `Pair`).
    fn debug_type_name(&self, name: &str, origin: Option<&(Symbol, Vec<Ty>)>) -> String {
        let name = origin.map(|(name, _)| name.as_str()).unwrap_or(name);
        name.rsplit('.').next().unwrap_or(name).to_string()
    }

    /// The generated `Display` function of `ty`, requesting it.
    fn fmt_fn(&self, ty: Ty) -> String {
        let mut fns = self.fmt_fns.borrow_mut();
        let index = fns.iter().position(|&known| known == ty).unwrap_or_else(|| {
            fns.push(ty);
            fns.len() - 1
        });
        fmt_fn_symbol(index)
    }

    /// The wrapper that formats `ty` with a spec after a conversion,
    /// requesting it.
    fn fmt_spec_fn(&self, ty: Ty) -> String {
        let name = format!("{}_spec", self.fmt_fn(ty));
        let mut specs = self.fmt_specs.borrow_mut();
        if !specs.contains(&ty) {
            specs.push(ty);
        }
        name
    }

    /// The wrapper that prints `ty`, requesting it.
    fn fmt_print_fn(&self, ty: Ty) -> String {
        let name = format!("{}_print", self.fmt_fn(ty));
        let mut prints = self.fmt_prints.borrow_mut();
        if !prints.contains(&ty) {
            prints.push(ty);
        }
        name
    }

    /// `[TYP-39]` — a C statement appending `v`'s `Debug` to the buffer
    /// `out`: text quoted, numbers and `bool` as they display, aggregates
    /// through their function.
    fn debug_stmt(&self, out: &str, v: &str, ty: Ty) -> String {
        // D-227 — a reference inside an aggregate prints what it points to.
        if let TyKind::Ref { inner, .. } = self.types.kind(ty) {
            return self.debug_stmt(out, &format!("(*({v}))"), *inner);
        }
        if let Some((text, _)) = self.text_pair(v, v, ty) {
            return format!("{RT}fmt_repr_str({out}, {text});");
        }
        // `[TYP-36]` — `void` shows as `()`.
        if matches!(self.types.kind(ty), TyKind::Void) {
            return format!("{RT}vec_extend({out}, \"()\", 2);");
        }
        match self.types.kind(ty) {
            TyKind::Char => format!("{RT}fmt_repr_char({out}, {v});"),
            _ if self.is_display_aggregate(ty) => format!("{}({out}, &({v}));", self.fmt_fn(ty)),
            _ => format!("{RT}fmt_{}({out}, {v});", self.builtin_suffix(ty)),
        }
    }

    /// `[TYP-39]` (ODR-034) — a `Map` prints `{k: v, …}` and a `Set` `{a, …}`,
    /// in insertion order, each part by its `Debug`; an empty `Set` prints
    /// `set()`, since `{}` is an empty `Map`. The layout is read from the
    /// declarations in `std/src/collections.em`: a `Set`'s first field is its
    /// `Map`, a `Map`'s first field its entries, `Array[Option[MapSlot]]`,
    /// and a slot has `key` and `value`.
    fn fmt_map_body(&self, ty: Ty) -> String {
        let text = |s: &str| format!("{RT}vec_extend(out, \"{s}\", {});", s.len());
        let TyKind::Struct(id) = *self.types.kind(ty) else { unreachable!("a Map or Set is a struct") };
        let def = self.types.struct_def(id);
        let is_set = def.origin.as_ref().is_some_and(|(name, _)| name.as_str() == "std.collections.Set");
        let (map_ty, access) = if is_set { (def.fields[0].ty, "v->map.") } else { (ty, "v->") };
        let TyKind::Struct(map_id) = *self.types.kind(map_ty) else { unreachable!("a Set holds a Map") };
        let map_def = self.types.struct_def(map_id);
        let entries = &map_def.fields[0];
        let TyKind::Vec { elem: option, .. } = *self.types.kind(entries.ty) else { unreachable!("a Map's entries are an Array") };
        let TyKind::Enum(option_id) = *self.types.kind(option) else { unreachable!("an entry is an Option") };
        let some = self
            .types
            .enum_def(option_id)
            .variants
            .iter()
            .find(|variant| variant.name.as_str() == "Some")
            .expect("an Option has Some");
        let payload = &some.fields[0];
        let TyKind::Struct(slot_id) = *self.types.kind(payload.ty) else { unreachable!("an entry holds a MapSlot") };
        let slot = self.types.struct_def(slot_id);
        let field = |name: &str| slot.fields.iter().find(|f| f.name.as_str() == name).expect("a MapSlot field");
        let (key, value) = (field("key"), field("value"));
        let at = format!("e->payload.{}.{}", some.name, payload.name);
        let mut item = vec![self.debug_stmt("out", &format!("{at}.{}", key.name), key.ty)];
        if !is_set {
            item.push(text(": "));
            item.push(self.debug_stmt("out", &format!("{at}.{}", value.name), value.ty));
        }
        let option_c = self.c_type(option);
        let walk = format!(
            "{open} {{ int first = 1; for (size_t i = 0; i < {access}{entries}.len; ++i) {{              const {option_c}* e = &((const {option_c}*){access}{entries}.ptr)[i];              if (e->tag != {tag}) continue; if (!first) {{ {sep} }} first = 0; {item} }} }} {close}",
            open = text("{"),
            close = text("}"),
            sep = text(", "),
            entries = entries.name,
            tag = some.discriminant,
            item = item.join(" "),
        );
        if is_set {
            format!("if ({access}live == 0) {{ {} }} else {{ {walk} }}", text("set()"))
        } else {
            walk
        }
    }

    /// Define every requested `Display` function, to a fixpoint, then the
    /// print wrappers, and put their prototypes where the marker stands.
    fn emit_fmt_fns(&mut self) {
        let text = |s: &str| format!("{RT}vec_extend(out, \"{s}\", {});", s.len());
        let mut emitted = 0;
        let mut prototypes = Vec::new();
        loop {
            let pending: Vec<Ty> = self.fmt_fns.borrow()[emitted..].to_vec();
            if pending.is_empty() {
                break;
            }
            for ty in pending {
                let symbol = fmt_fn_symbol(emitted);
                let c_ty = self.c_type(ty);
                let each = |this: &Self, elem: Ty, at: &str| {
                    let elem_c = this.c_type(elem);
                    let item = this.debug_stmt("out", &format!("((const {elem_c}*){at})[i]"), elem);
                    format!("for (size_t i = 0; i < {{len}}; ++i) {{ if (i) {} {item} }}", text(", "))
                };
                let body = match self.types.kind(ty).clone() {
                    TyKind::Vec { elem, .. } | TyKind::Span { elem, .. } => {
                        format!("{} {} {}", text("["), each(self, elem, "v->ptr").replace("{len}", "v->len"), text("]"))
                    }
                    TyKind::Array { elem, len } => {
                        format!("{} {} {}", text("["), each(self, elem, "v->_0").replace("{len}", &len.to_string()), text("]"))
                    }
                    TyKind::Class(_) | TyKind::ClassInterface(_) => {
                        format!("{RT}fmt_handle(out, (const void*)(*v));")
                    }
                    TyKind::Tuple(items) => {
                        let mut parts = vec![text("(")];
                        for (i, &item) in items.iter().enumerate() {
                            if i > 0 {
                                parts.push(text(", "));
                            }
                            parts.push(self.debug_stmt("out", &format!("v->_{i}"), item));
                        }
                        // Python writes a one-element tuple `(1,)`.
                        if items.len() == 1 {
                            parts.push(text(","));
                        }
                        parts.push(text(")"));
                        parts.join(" ")
                    }
                    // `[TYP-39]` (ODR-034) — `{'a': 1}`, `{}`, `{1, 2}`, `set()`.
                    TyKind::Struct(id)
                        if matches!(
                            self.types.struct_def(id).origin.as_ref().map(|(name, _)| name.as_str()),
                            Some("std.collections.Map" | "std.collections.Set")
                        ) =>
                    {
                        self.fmt_map_body(ty)
                    }
                    // `[STR-5]`, `[TYP-36]` — `Point(x=1, y=2)`.
                    TyKind::Struct(id) => {
                        let def = self.types.struct_def(id).clone();
                        let name = self.debug_type_name(def.name.as_str(), def.origin.as_ref());
                        let mut parts = vec![text(&format!("{name}("))];
                        for (i, field) in def.fields.iter().enumerate() {
                            if i > 0 {
                                parts.push(text(", "));
                            }
                            parts.push(text(&format!("{}=", field.name)));
                            parts.push(self.debug_stmt("out", &format!("v->{}", field.name), field.ty));
                        }
                        parts.push(text(")"));
                        parts.join(" ")
                    }
                    TyKind::Enum(id) => {
                        let def = self.types.enum_def(id).clone();
                        // `Option` and `Result` show `Some(1)`, `Err('x')`;
                        // a program's enum shows its name, `Shape.Circle(1)`.
                        let compiler_known = def.name.as_str().starts_with("Option_")
                            || def.name.as_str().starts_with("Result_");
                        let prefix = if compiler_known {
                            String::new()
                        } else {
                            format!("{}.", self.debug_type_name(def.name.as_str(), def.origin.as_ref()))
                        };
                        let mut arms = Vec::new();
                        for (variant_index, variant) in def.variants.iter().enumerate() {
                            let mut parts = vec![text(&format!("{prefix}{}", variant.name))];
                            if !variant.fields.is_empty() {
                                parts.push(text("("));
                                for (i, field) in variant.fields.iter().enumerate() {
                                    if i > 0 {
                                        parts.push(text(", "));
                                    }
                                    // A named payload field shows its name;
                                    // a positional one (`_0`) does not.
                                    if !field.name.as_str().starts_with('_') {
                                        parts.push(text(&format!("{}=", field.name)));
                                    }
                                    let member = self.enum_member(id, "*v", variant_index, i);
                                    parts.push(self.debug_stmt("out", &member, field.ty));
                                }
                                parts.push(text(")"));
                            }
                            arms.push(format!("case {}: {} break;", variant.discriminant, parts.join(" ")));
                        }
                        // A unit-only enum is its tag.
                        let tag = if def.is_unit_only() { "*v".to_string() } else { self.enum_tag(id, "*v") };
                        format!("switch ({tag}) {{ {} default: break; }}", arms.join(" "))
                    }
                    _ => unreachable!("[TYP-39] Display functions are requested only for aggregates"),
                };
                // By pointer: a fixed array can be large, and a copy per
                // call (and per nesting level) could overflow the stack.
                prototypes.push(format!("static void {symbol}({RT}vec* out, const {c_ty}* v);"));
                self.line(&format!("static void {symbol}({RT}vec* out, const {c_ty}* v) {{"));
                self.line("    (void)v;");
                self.line(&format!("    {body}"));
                self.line("}");
                self.line("");
                emitted += 1;
            }
        }
        let specs: Vec<Ty> = self.fmt_specs.borrow().clone();
        for ty in specs {
            let symbol = self.fmt_fn(ty);
            let c_ty = self.c_type(ty);
            prototypes.push(format!("static void {symbol}_spec({RT}vec* out, const {c_ty}* v, {RT}fmt_spec spec);"));
            self.line(&format!("static void {symbol}_spec({RT}vec* out, const {c_ty}* v, {RT}fmt_spec spec) {{"));
            self.line(&format!("    {RT}vec text = {RT}vec_empty();"));
            self.line(&format!("    {symbol}(&text, v);"));
            self.line("    spec.kind = 0;");
            self.line(&format!("    {RT}fmt_spec_str(out, {RT}vec_as_str(&text), spec);"));
            self.line(&format!("    {RT}vec_free(&text, 1);"));
            self.line("}");
            self.line("");
        }
        let prints: Vec<Ty> = self.fmt_prints.borrow().clone();
        for ty in prints {
            let symbol = self.fmt_fn(ty);
            let c_ty = self.c_type(ty);
            prototypes.push(format!("static void {symbol}_print(const {c_ty}* v, int to_stderr);"));
            self.line(&format!("static void {symbol}_print(const {c_ty}* v, int to_stderr) {{"));
            self.line(&format!("    {RT}vec buffer = {RT}vec_empty();"));
            self.line(&format!("    {symbol}(&buffer, v);"));
            self.line(&format!("    {RT}str text = {RT}vec_as_str(&buffer);"));
            self.line(&format!("    if (to_stderr) {RT}eprint_str(text); else {RT}print_str(text);"));
            self.line(&format!("    {RT}vec_free(&buffer, 1);"));
            self.line("}");
            self.line("");
        }
        self.out = self.out.replacen(FMT_FN_PROTOTYPES, &prototypes.join("\n"), 1);
    }

    /// `str` and `String` operands as two `str` values, for `{RT}str_cmp`.
    fn text_pair(&self, a: &str, b: &str, ty: Ty) -> Option<(String, String)> {
        match self.types.kind(ty) {
            TyKind::Str => Some((a.to_string(), b.to_string())),
            TyKind::Vec { text: true, .. } => {
                let view = |v: &str| format!("(({RT}str){{ (const unsigned char*)({v}).ptr, ({v}).len }})");
                Some((view(a), view(b)))
            }
            _ => None,
        }
    }

    /// Define every requested equality function, to a fixpoint, and put
    /// their prototypes where the marker stands. The comparison borrows each
    /// value through a pointer, so nesting does not multiply stack use.
    fn emit_eq_fns(&mut self) {
        let mut emitted = 0;
        let mut prototypes = Vec::new();
        loop {
            let pending: Vec<Ty> = self.eq_fns.borrow()[emitted..].to_vec();
            if pending.is_empty() {
                break;
            }
            for ty in pending {
                let symbol = eq_fn_symbol(emitted);
                let c_ty = self.c_type(ty);
                let body = match self.types.kind(ty).clone() {
                    TyKind::Struct(id) => {
                        let fields = self.types.struct_def(id).fields.clone();
                        let parts: Vec<String> = fields
                            .iter()
                            .map(|f| self.eq_expr(&format!("a->{}", f.name), &format!("b->{}", f.name), f.ty))
                            .collect();
                        format!("return {};", conjunction(parts))
                    }
                    TyKind::Tuple(items) => {
                        let parts: Vec<String> = items
                            .iter()
                            .enumerate()
                            .map(|(i, &item)| self.eq_expr(&format!("a->_{i}"), &format!("b->_{i}"), item))
                            .collect();
                        format!("return {};", conjunction(parts))
                    }
                    TyKind::Array { elem, len } => {
                        let each = self.eq_expr("a->_0[i]", "b->_0[i]", elem);
                        format!("for (size_t i = 0; i < {len}; ++i) {{ if (!{each}) return 0; }} return 1;")
                    }
                    TyKind::Vec { elem, .. } => {
                        let elem_c = self.c_type(elem);
                        let each = self.eq_expr(
                            &format!("(({elem_c}*)a->ptr)[i]"),
                            &format!("(({elem_c}*)b->ptr)[i]"),
                            elem,
                        );
                        format!(
                            "if (a->len != b->len) return 0; for (size_t i = 0; i < a->len; ++i) {{ if (!{each}) return 0; }} return 1;"
                        )
                    }
                    TyKind::Enum(id) => {
                        let def = self.types.enum_def(id).clone();
                        let mut arms = Vec::new();
                        for (variant_index, variant) in def.variants.iter().enumerate() {
                            let parts: Vec<String> = variant
                                .fields
                                .iter()
                                .enumerate()
                                .map(|(field_index, f)| {
                                    let a = self.enum_member(id, "(*a)", variant_index, field_index);
                                    let b = self.enum_member(id, "(*b)", variant_index, field_index);
                                    self.eq_expr(&a, &b, f.ty)
                                })
                                .collect();
                            if !parts.is_empty() {
                                arms.push(format!("case {}: return {};", variant.discriminant, conjunction(parts)));
                            }
                        }
                        let (tag_a, tag_b) = (self.enum_tag(id, "(*a)"), self.enum_tag(id, "(*b)"));
                        format!(
                            "if ({tag_a} != {tag_b}) return 0; switch ({tag_a}) {{ {} default: return 1; }}",
                            arms.join(" ")
                        )
                    }
                    _ => unreachable!("D-187 equality functions are requested only for aggregates"),
                };
                prototypes.push(format!("static bool {symbol}(const {c_ty}* a, const {c_ty}* b);"));
                self.line(&format!("static bool {symbol}(const {c_ty}* a, const {c_ty}* b) {{"));
                self.line("    (void)a; (void)b;");
                self.line(&format!("    {body}"));
                self.line("}");
                self.line("");
                emitted += 1;
            }
        }
        self.out = self.out.replacen(EQ_FN_PROTOTYPES, &prototypes.join("\n"), 1);
    }

    /// The name of `ty`'s out-of-line drop glue, requesting it, for a part
    /// reached through an `Array` element, a `Box` payload or an enum
    /// payload. Only aggregates are worth a function; everything else drops
    /// in one line. A part goes out of line when it owns itself, or when its
    /// own drop would open another level of C nesting (D-358): then no glue
    /// function nests more than one level, however deep the type is, and a
    /// type nested 256 deep (`[GRM-39]`) stays within every C compiler's
    /// limits.
    fn drop_glue_call(&self, access: &str, ty: Ty) -> Option<String> {
        if !matches!(
            self.types.kind(ty),
            TyKind::Struct(_)
                | TyKind::Enum(_)
                | TyKind::Tuple(_)
                | TyKind::Array { .. }
                | TyKind::Vec { .. }
        ) || !(self.owns_itself(ty) || self.drop_nests(ty, &mut BTreeSet::new()))
        {
            return None;
        }
        let mut glue = self.drop_glue.borrow_mut();
        let index = match glue.iter().position(|&known| known == ty) {
            Some(index) => index,
            None => {
                glue.push(ty);
                glue.len() - 1
            }
        };
        Some(format!("{}(&{access});", drop_glue_symbol(index)))
    }

    /// Whether dropping `ty` can reach another `ty` it owns — through an
    /// `Array` element or a `Box` payload, the only owning indirections whose
    /// drop the backend expands inline. Only such a type needs out-of-line
    /// glue; every other drop stays inline, as it always was.
    fn owns_itself(&self, ty: Ty) -> bool {
        let mut seen = BTreeSet::new();
        let mut pending = self.owned_parts(ty);
        while let Some(part) = pending.pop() {
            if part == ty {
                return true;
            }
            if seen.insert(part) {
                pending.extend(self.owned_parts(part));
            }
        }
        false
    }

    /// Whether dropping `ty` in line opens a level of C nesting: a loop over
    /// an `Array`'s elements, a `switch` on an enum's tag, or a dereference
    /// of a `Box`'s payload, each of which also wraps the access expression
    /// once more. `seen` stops a recursive type, which has glue anyway.
    fn drop_nests(&self, ty: Ty, seen: &mut BTreeSet<Ty>) -> bool {
        if !self.types.needs_drop(ty) {
            return false;
        }
        if !seen.insert(ty) {
            return true;
        }
        match self.types.kind(ty) {
            TyKind::Vec { elem, .. } => self.types.needs_drop(*elem),
            TyKind::Array { elem, .. } => self.drop_nests(*elem, seen),
            TyKind::Tuple(items) => items.clone().into_iter().any(|item| self.drop_nests(item, seen)),
            TyKind::Enum(id) => {
                let def = self.types.enum_def(*id);
                !def.is_unit_only()
                    && def
                        .variants
                        .iter()
                        .any(|variant| variant.fields.iter().any(|field| self.types.needs_drop(field.ty)))
            }
            TyKind::Struct(id) => {
                if self.shared_inner_id(*id).is_some() || self.weak_inner_id(*id).is_some() {
                    return false;
                }
                if let Some(payload) = self.box_inner_id(*id) {
                    return !matches!(self.types.kind(payload), TyKind::Dyn { .. })
                        && self.types.needs_drop(payload);
                }
                let def = self.types.struct_def(*id);
                def.drops_fields
                    && def.fields.clone().into_iter().any(|field| self.drop_nests(field.ty, seen))
            }
            _ => false,
        }
    }

    /// The values a value of `ty` owns and drops inline.
    fn owned_parts(&self, ty: Ty) -> Vec<Ty> {
        match self.types.kind(ty) {
            TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } => vec![*elem],
            TyKind::Tuple(items) => items.clone(),
            TyKind::Struct(id) => match self.box_inner_id(*id) {
                Some(payload) => vec![payload],
                None => self.types.struct_def(*id).fields.iter().map(|field| field.ty).collect(),
            },
            TyKind::Enum(id) => self
                .types
                .enum_def(*id)
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| field.ty))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Define every requested glue function, to a fixpoint (a glue body can
    /// request more), and put their prototypes where the marker stands.
    fn emit_drop_glue(&mut self) {
        let mut emitted = 0;
        let mut prototypes = Vec::new();
        loop {
            let pending: Vec<Ty> = self.drop_glue.borrow()[emitted..].to_vec();
            if pending.is_empty() {
                break;
            }
            for ty in pending {
                let symbol = drop_glue_symbol(emitted);
                let c_ty = self.c_type(ty);
                let mut lines = Vec::new();
                self.drop_lines("(*value)", ty, &mut lines);
                prototypes.push(format!("static void {symbol}({c_ty}* value);"));
                self.line(&format!("static void {symbol}({c_ty}* value) {{"));
                for line in lines {
                    self.line(&format!("    {line}"));
                }
                self.line("}");
                self.line("");
                emitted += 1;
            }
        }
        self.out = self.out.replacen(DROP_GLUE_PROTOTYPES, &prototypes.join("\n"), 1);
    }

    /// `ember_types.h`'s content, inlined into the single translation unit
    /// Phase 0 emits. Structs are declared in the order they were interned,
    /// which is declaration order, so a struct's fields are always already
    /// complete types.
    fn emit_type_declarations(&mut self) {
        let plan: Vec<(String, Definition)> = self
            .order
            .clone()
            .into_iter()
            .map(|node| (self.node_name(node), self.definition(node)))
            .collect();

        if plan.is_empty() {
            return;
        }
        self.line("/* types */");
        // Every struct is forward-declared, so one may hold a pointer to
        // another defined later. An alias is complete where it stands.
        for (name, definition) in &plan {
            if matches!(definition, Definition::Struct(_)) {
                self.line(&format!("typedef struct {name} {name};"));
            }
        }
        for (name, definition) in &plan {
            match definition {
                Definition::Alias(underlying) => {
                    self.line(&format!("typedef {underlying} {name};"));
                }
                Definition::FnPointer { ret, args } => {
                    self.line(&format!("typedef {ret} (*{name})({args});"));
                }
                Definition::Struct(members) => {
                    self.line(&format!("struct {name} {{"));
                    if members.is_empty() {
                        // `[STR-4]` — a zero-sized struct is legal in Ember but
                        // not in C, so it gets a padding byte. Its size is
                        // never observed through the C type.
                        self.line("    char _empty;");
                    }
                    for member in members {
                        self.line(&format!("    {member};"));
                    }
                    self.line("};");
                }
            }
        }
        self.line("");
    }

    /// Adapt a source class's `drop(mut self)` method to the runtime's
    /// `void(*)(void*)` metadata slot. The ordinary function prototype is
    /// emitted first, so this wrapper remains valid C11 without relying on an
    /// implicit declaration or an incompatible function-pointer conversion.
    fn emit_class_drop_adapters(&mut self) {
        let classes: Vec<ClassId> = self.types.runtime_classes().map(|(id, _)| id).collect();
        for id in classes {
            let def = self.types.class_def(id);
            if !self.class_has_user_drop(id) {
                continue;
            }
            let owner = def.name.to_string();
            let object = ember_branding::object_struct(&owner);
            let adapter = class_drop_adapter_symbol(&owner);
            self.line(&format!("static void {adapter}(void* raw) {{"));
            self.line(&format!(
                "    {object}* handle = ({object}*)raw;",
            ));
            if def.has_drop {
                self.line(&format!("    {}(&handle);", drop_symbol(&owner)));
            }
            // `[CLS-6]` — a derived destructor runs before each base
            // destructor. All handles point at the same allocation, and the
            // base fields occupy its prefix, so a typed pointer adjustment is
            // sufficient for the base method's ordinary `mut self` ABI.
            let mut base = def.base;
            let mut depth = 0;
            while let Some(base_id) = base {
                let base_def = self.types.class_def(base_id);
                if base_def.has_drop {
                    let base_object = ember_branding::object_struct(&base_def.name.to_string());
                    let base_handle = format!("base_handle_{depth}");
                    self.line(&format!(
                        "    {base_object}* {base_handle} = ({base_object}*)raw;"
                    ));
                    self.line(&format!(
                        "    {}(&{base_handle});",
                        drop_symbol(&base_def.name.to_string())
                    ));
                }
                base = base_def.base;
                depth += 1;
            }
            self.line("}");
            self.line("");
        }
    }

    /// Whether releasing this concrete class must invoke at least one source
    /// destructor. Derived classes inherit the obligation even when they do
    /// not declare their own `drop`, because `[CLS-6]` still requires the base
    /// destructor to run.
    fn class_has_user_drop(&self, id: ClassId) -> bool {
        let mut current = Some(id);
        while let Some(class) = current {
            let def = self.types.class_def(class);
            if def.has_drop {
                return true;
            }
            current = def.base;
        }
        false
    }

    /// Generate the field half of `[CLS-6]`'s destruction contract. Class
    /// objects are released through the runtime rather than by the ordinary
    /// local drop walk, so the runtime needs a compiler-owned callback that
    /// can reach the generated object fields. The callback walks the most
    /// derived fields first and each declaration list in reverse order.
    fn emit_class_field_drop_glue(&mut self) {
        let classes: Vec<ClassId> = self.types.runtime_classes().map(|(id, _)| id).collect();
        for id in classes {
            if !self.class_has_dropping_fields(id) {
                continue;
            }
            let def = self.types.class_def(id);
            let object = ember_branding::object_struct(&def.name.to_string());
            let symbol = class_drop_fields_symbol(&def.name.to_string());
            self.line(&format!("static void {symbol}(void* raw) {{"));
            self.line(&format!("    struct {object}* object = (struct {object}*)raw;"));

            let mut chain = Vec::new();
            let mut current = Some(id);
            while let Some(class) = current {
                chain.push(class);
                current = self.types.class_def(class).base;
            }
            for class in chain {
                let fields = self.types.class_def(class).fields.clone();
                for field in fields.iter().rev() {
                    if !self.types.needs_drop(field.ty) {
                        continue;
                    }
                    let mut lines = Vec::new();
                    self.drop_lines(&format!("object->{}", field.name), field.ty, &mut lines);
                    for line in lines {
                        self.line(&format!("    {line}"));
                    }
                }
            }
            self.line("}");
            self.line("");
        }
    }

    fn class_has_dropping_fields(&self, id: ClassId) -> bool {
        let mut current = Some(id);
        while let Some(class) = current {
            let def = self.types.class_def(class);
            if def.fields.iter().any(|field| self.types.needs_drop(field.ty)) {
                return true;
            }
            current = def.base;
        }
        false
    }

    /// Collect the effective class vtables from the already checked method
    /// bodies.  The type checker assigned slots in declaration order; this
    /// pass only materializes the inherited-prefix/override layout required
    /// by `[DSP-2]`.
    fn collect_virtual_methods(&mut self, bodies: &[Body]) {
        let declared: BTreeMap<(ClassId, usize), VirtualMethod> = bodies
            .iter()
            .filter_map(|body| {
                Some((
                    (body.class_owner?, body.class_virtual_slot?),
                    VirtualMethod {
                        owner: body.class_owner?,
                        symbol: body.symbol.clone(),
                        params: body.args().map(|(_, decl)| decl.ty).collect(),
                        ret: body.return_ty(),
                        is_abstract: body.is_abstract,
                    },
                ))
            })
            .collect();
        let mut layouts = BTreeMap::new();
        let ids: Vec<ClassId> = self.types.runtime_classes().map(|(id, _)| id).collect();
        for id in ids {
            let layout = self.build_virtual_layout(id, &declared, &mut layouts);
            for slot in 0..layout.len() {
                if let Some(signature) = self.virtual_slot_signature(id, slot, &declared) {
                    self.virtual_signatures.insert((id, slot), signature);
                }
            }
        }
        self.virtual_tables = layouts;
    }

    /// Collect the dynamic-interface call signatures carried by MIR.
    fn collect_interface_methods(&mut self, bodies: &[Body]) {
        for body in bodies {
            for block in &body.blocks {
                let Terminator::Call {
                    func:
                        FuncRef::Interface {
                            interfaces,
                            interface,
                            layout,
                            class_handle,
                            ..
                        },
                    ..
                } = &block.terminator
                else {
                    continue;
                };
                if *class_handle {
                    self.class_interface_call_interfaces.insert(interface.to_string());
                }
                let layout = layout
                    .iter()
                    .map(|slot| slot.as_ref().map(|slot| InterfaceMethod {
                        params: slot.params.clone(),
                        ret: slot.ret,
                    }))
                    .collect();
                self.register_interface_layout(dyn_table_identity(interfaces), layout);
            }
        }
    }

    fn register_interface_layout(
        &mut self,
        interface: String,
        layout: Vec<Option<InterfaceMethod>>,
    ) {
        match self.interface_layouts.entry(interface) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(layout);
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                assert_eq!(
                    entry.get(),
                    &layout,
                    "one interface identity must carry one canonical vtable layout"
                );
            }
        }
    }

    /// Gather the concrete adapter tables from the explicit MIR cast rather
    /// than rediscovering type/interface conformance in the backend. The
    /// type checker is the authority for both facts, and MIR preserves them
    /// across the code-generation boundary.
    fn collect_interface_adapters(&mut self, bodies: &[Body]) {
        for body in bodies {
            for block in &body.blocks {
                for statement in &block.stmts {
                    match &statement.kind {
                        StmtKind::Assign {
                            rvalue:
                                Rvalue::Cast {
                                    kind:
                                        CastKind::InterfaceUpcast {
                                            concrete,
                                            interfaces,
                                            layout,
                                            implementations,
                                        },
                                    ..
                                },
                            ..
                        } => {
                            self.register_interface_adapter(
                                *concrete,
                                dyn_table_identity(interfaces),
                                layout
                                    .iter()
                                    .map(|slot| slot.as_ref().map(|slot| InterfaceMethod {
                                        params: slot.params.clone(),
                                        ret: slot.ret,
                                    }))
                                    .collect(),
                                implementations
                                    .iter()
                                    .map(|implementation| implementation.as_ref().map(|implementation| {
                                        InterfaceAdapterMethod {
                                            symbol: implementation.symbol.clone(),
                                            receiver: implementation.receiver,
                                        }
                                    }))
                                    .collect(),
                            );
                        }
                        StmtKind::Assign {
                            rvalue:
                                Rvalue::Cast {
                                    kind:
                                        CastKind::ClassInterfaceUpcast {
                                            concrete,
                                            interface,
                                            layout,
                                            implementations,
                                        },
                                    ..
                                },
                            ..
                        } => {
                            let identity = interface.to_string();
                            self.register_interface_adapter(
                                *concrete,
                                identity.clone(),
                                layout
                                    .iter()
                                    .map(|slot| slot.as_ref().map(|slot| InterfaceMethod {
                                        params: slot.params.clone(),
                                        ret: slot.ret,
                                    }))
                                    .collect(),
                                implementations
                                    .iter()
                                    .map(|implementation| implementation.as_ref().map(|implementation| {
                                        InterfaceAdapterMethod {
                                            symbol: implementation.symbol.clone(),
                                            receiver: implementation.receiver,
                                        }
                                    }))
                                    .collect(),
                            );
                            let TyKind::Class(class) = self.types.kind(*concrete) else {
                                unreachable!("verified class-interface cast has a class source");
                            };
                            self.class_interface_tables
                                .entry(*class)
                                .or_default()
                                .insert(identity, *concrete);
                        }
                        _ => {}
                    }
                }
                if let Terminator::Call {
                    func:
                        FuncRef::DynBoxNew {
                            concrete,
                            interfaces,
                            layout,
                            implementations,
                            ..
                        },
                    ..
                } = &block.terminator
                {
                    self.register_interface_adapter(
                        *concrete,
                        dyn_table_identity(interfaces),
                        layout
                            .iter()
                            .map(|slot| {
                                slot.as_ref().map(|slot| InterfaceMethod {
                                    params: slot.params.clone(),
                                    ret: slot.ret,
                                })
                            })
                            .collect(),
                        implementations
                            .iter()
                            .map(|implementation| {
                                implementation.as_ref().map(|implementation| {
                                    InterfaceAdapterMethod {
                                        symbol: implementation.symbol.clone(),
                                        receiver: implementation.receiver,
                                    }
                                })
                            })
                            .collect(),
                    );
                }
            }
        }
    }

    fn register_interface_adapter(
        &mut self,
        concrete: Ty,
        interface: String,
        layout: Vec<Option<InterfaceMethod>>,
        implementations: Vec<Option<InterfaceAdapterMethod>>,
    ) {
        self.register_interface_layout(interface.clone(), layout.clone());
        let key = (interface.clone(), self.interface_adapter_concrete_name(concrete));
        let adapter = InterfaceAdapter { concrete, interface, layout, implementations };
        match self.interface_adapters.entry(key) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(adapter);
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                assert_eq!(
                    entry.get(),
                    &adapter,
                    "one concrete/interface coercion must carry one canonical adapter"
                );
            }
        }
    }

    /// Emit one compiler-owned C struct per interface whose vtable is read by
    /// this translation unit. The first three fields follow `[TYP-22]`'s
    /// `{drop, size, align, method0, ...}` shape; method slots use an erased
    /// `void*` receiver and the statically known interface argument types.
    fn emit_interface_vtable_types(&mut self) {
        let has_dyn_box = self.types.structs().any(|(id, _)| {
            self.box_inner_id(id)
                .is_some_and(|inner| matches!(self.types.kind(inner), TyKind::Dyn { .. }))
        });
        if self.interface_layouts.is_empty() && !has_dyn_box {
            return;
        }
        self.line("/* dynamic interface vtable types */");
        if has_dyn_box {
            let header = ember_branding::mangled("dyn_vtable_header");
            self.line(&format!("struct {header} {{"));
            self.line("    void (*drop)(void*);");
            self.line("    size_t size;");
            self.line("    size_t align;");
            self.line("};");
        }
        let class_handle_interfaces = self
            .class_interface_tables
            .values()
            .flat_map(|tables| tables.keys())
            .cloned()
            .chain(self.class_interface_call_interfaces.iter().cloned())
            .collect::<BTreeSet<_>>();
        for (interface, methods) in self.interface_layouts.clone() {
            if class_handle_interfaces.contains(&interface) {
                self.line(&format!(
                    "static const uint8_t {} = UINT8_C(0);",
                    interface_id_symbol(&interface)
                ));
            }
            let table = ember_branding::vtable(&format!("dyn_{interface}"));
            self.line(&format!("struct {table} {{"));
            self.line("    void (*drop)(void*);");
            self.line("    size_t size;");
            self.line("    size_t align;");
            for (slot, method) in methods.into_iter().enumerate() {
                let Some(method) = method else {
                    self.line(&format!("    void (*slot{slot})(void*);"));
                    continue;
                };
                let params = method
                    .params
                    .iter()
                    .map(|ty| self.c_type(*ty))
                    .collect::<Vec<_>>();
                let params = if params.is_empty() {
                    "void*".to_string()
                } else {
                    format!("void*, {}", params.join(", "))
                };
                self.line(&format!("    {} (*slot{slot})({params});", self.c_type(method.ret)));
            }
            self.line("};");
        }
        self.line("");
    }

    fn interface_adapter_table(&self, concrete: Ty, interface: &str) -> String {
        ember_branding::vtable(&format!(
            "dyn_{interface}_{}",
            self.interface_adapter_concrete_name(concrete)
        ))
    }

    fn interface_adapter_concrete_name(&self, concrete: Ty) -> String {
        if self.types.is_primitive_scalar(concrete) {
            return self.types.symbol_name(concrete);
        }
        match self.types.kind(concrete) {
            TyKind::Class(id) => self.types.class_def(*id).name.to_string(),
            TyKind::Struct(id) => self.types.struct_def(*id).name.to_string(),
            TyKind::Enum(id) => self.types.enum_def(*id).name.to_string(),
            _ => unreachable!("only concrete class, struct, enum, and scalar adapters reach C emission"),
        }
    }

    fn interface_vtable_type(interface: &str) -> String {
        ember_branding::vtable(&format!("dyn_{interface}"))
    }

    /// Emit one C adapter per concrete/interface slot. Class receivers need
    /// their handle ABI; struct receivers use their ordinary value/inout ABI.
    fn emit_interface_adapters(&mut self) {
        let adapters = self.interface_adapters.values().cloned().collect::<Vec<_>>();
        if adapters.is_empty() {
            return;
        }
        self.line("/* concrete dynamic interface adapters */");
        for adapter in &adapters {
            assert_eq!(
                adapter.layout.len(),
                adapter.implementations.len(),
                "every dynamic adapter has one entry per table slot"
            );
            let table = self.interface_adapter_table(adapter.concrete, &adapter.interface);
            let table_type = Self::interface_vtable_type(&adapter.interface);
            let (payload_ty, receiver_ty, class_handle) = match self.types.kind(adapter.concrete) {
                TyKind::Class(class) => {
                    let object = ember_branding::object_struct(
                        &self.types.class_def(*class).name.to_string(),
                    );
                    let object = format!("struct {object}");
                    (object.clone(), format!("{object}*"), true)
                }
                _ => {
                    let concrete = self.c_type(adapter.concrete);
                    (concrete.clone(), concrete, false)
                }
            };
            let value_payload = self.types.is_primitive_scalar(adapter.concrete)
                || matches!(
                    self.types.kind(adapter.concrete),
                    TyKind::Struct(_) | TyKind::Enum(_)
                );
            let (drop, size, align) = if value_payload {
                let name = format!("{table}_drop");
                let mut lines = Vec::new();
                self.drop_lines(
                    &format!("(*({}*)_0)", self.c_type(adapter.concrete)),
                    adapter.concrete,
                    &mut lines,
                );
                self.line(&format!("static void {name}(void* _0) {{"));
                for line in lines {
                    self.line(&format!("    {line}"));
                }
                self.line("}");
                (name, format!("{payload_ty_size}", payload_ty_size = c_size(&payload_ty)), format!("{payload_ty_align}", payload_ty_align = c_align(&payload_ty)))
            } else if class_handle {
                let name = format!("{table}_drop");
                self.line(&format!("static void {name}(void* _0) {{"));
                self.line(&format!(
                    "    {}(({}*)_0);",
                    ember_branding::runtime("release"),
                    ember_branding::runtime("obj_header")
                ));
                self.line("}");
                // A class handle is already an owning allocation. The dyn
                // box carries it directly, so its release has no separate
                // payload allocation to free.
                (name, "0".to_string(), "0".to_string())
            } else {
                ("NULL".to_string(), "0".to_string(), "0".to_string())
            };
            for (slot, (signature, implementation)) in adapter
                .layout
                .iter()
                .zip(&adapter.implementations)
                .enumerate()
            {
                let (Some(signature), Some(implementation)) = (signature, implementation) else {
                    continue;
                };
                let name = format!("{table}_slot{slot}");
                let params = signature
                    .params
                    .iter()
                    .enumerate()
                    .map(|(index, ty)| format!("{} _{}", self.c_type(*ty), index + 1))
                    .collect::<Vec<_>>();
                let params = if params.is_empty() {
                    "void* _0".to_string()
                } else {
                    format!("void* _0, {}", params.join(", "))
                };
                self.line(&format!("static {} {name}({params}) {{", self.c_type(signature.ret)));
                let mut args = Vec::with_capacity(signature.params.len() + 1);
                match implementation.receiver {
                    ParameterMode::Borrow if class_handle => {
                        args.push(format!("({receiver_ty})_0"));
                    }
                    // `[BRW-8]` (ODR-024) — a receiver passed by address gets
                    // the payload pointer itself.
                    ParameterMode::Borrow
                        if self.types.passed_by_address(adapter.concrete)
                            || (self.types.is_view(signature.ret) && !self.types.is_view(adapter.concrete)) =>
                    {
                        args.push(format!("({receiver_ty}*)_0"));
                    }
                    ParameterMode::Borrow => {
                        args.push(format!("*({receiver_ty}*)_0"));
                    }
                    ParameterMode::Mut if class_handle => {
                        self.line(&format!("    {receiver_ty} receiver = ({receiver_ty})_0;"));
                        args.push("&receiver".to_string());
                    }
                    ParameterMode::Mut => args.push(format!("({receiver_ty}*)_0")),
                    ParameterMode::Owned => unreachable!("owned receivers cannot form a dyn adapter"),
                }
                args.extend((1..=signature.params.len()).map(|index| format!("_{index}")));
                let args = args.join(", ");
                // `[EXC-15]` — a `mut self` call on a class object writes every
                // field for the call. Its caller holds only the erased `dyn`
                // value, so the adapter, which knows the class, takes it.
                let object_write = matches!(implementation.receiver, ParameterMode::Mut) && class_handle;
                let object = format!("(({}*)receiver)", ember_branding::runtime("obj_header"));
                let nowhere = format!("({}loc){{ NULL, 0, 0 }}", RT);
                if object_write {
                    self.line(&format!("    {RT}object_begin_write({object}, {nowhere});"));
                }
                if self.is_void(signature.ret) {
                    self.line(&format!("    {}({args});", implementation.symbol));
                    if object_write {
                        self.line(&format!("    {RT}object_end_write({object}, {nowhere});"));
                    }
                } else if object_write {
                    self.line(&format!("    {} result = {}({args});", self.c_type(signature.ret), implementation.symbol));
                    self.line(&format!("    {RT}object_end_write({object}, {nowhere});"));
                    self.line("    return result;");
                } else {
                    self.line(&format!("    return {}({args});", implementation.symbol));
                }
                self.line("}");
            }
            self.line(&format!("static const struct {table_type} {table} = {{"));
            self.line(&format!("    {drop},"));
            self.line(&format!("    {size},"));
            self.line(&format!("    {align},"));
            for (slot, implementation) in adapter.implementations.iter().enumerate() {
                let value = if implementation.is_some() {
                    format!("{table}_slot{slot}")
                } else {
                    "NULL".to_string()
                };
                self.line(&format!("    {value},"));
            }
            self.line("};");
        }
        self.line("");
    }

    fn build_virtual_layout(
        &self,
        id: ClassId,
        declared: &BTreeMap<(ClassId, usize), VirtualMethod>,
        layouts: &mut BTreeMap<ClassId, Vec<Option<VirtualMethod>>>,
    ) -> Vec<Option<VirtualMethod>> {
        if let Some(layout) = layouts.get(&id) {
            return layout.clone();
        }
        let mut layout = self
            .types
            .class_def(id)
            .base
            .map(|base| self.build_virtual_layout(base, declared, layouts))
            .unwrap_or_default();
        for ((owner, slot), method) in declared {
            if *owner != id {
                continue;
            }
            if layout.len() <= *slot {
                layout.resize(*slot + 1, None);
            }
            layout[*slot] = Some(method.clone());
        }
        layouts.insert(id, layout.clone());
        layout
    }

    /// The first declaration in the inheritance chain establishes a slot's
    /// C function-pointer signature. Overrides are adapted to that signature
    /// below rather than relying on incompatible function-pointer casts.
    fn virtual_slot_signature(
        &self,
        id: ClassId,
        slot: usize,
        declared: &BTreeMap<(ClassId, usize), VirtualMethod>,
    ) -> Option<VirtualMethod> {
        if let Some(base) = self.types.class_def(id).base
            && let Some(signature) = self.virtual_slot_signature(base, slot, declared)
        {
            return Some(signature);
        }
        declared.get(&(id, slot)).cloned()
    }

    fn virtual_field_signature(&self, method: &VirtualMethod, field: &str) -> String {
        let params = method
            .params
            .iter()
            .map(|ty| self.c_member_type(*ty))
            .collect::<Vec<_>>();
        let params = if params.is_empty() { "void".to_string() } else { params.join(", ") };
        format!("{} (*{field})({params})", self.c_type(method.ret))
    }

    /// Emit compiler-owned vtable structs, override adapters and concrete
    /// tables. Each derived table begins with the exact base-slot prefix, so a
    /// base-typed call can inspect a derived object's `type_info` safely.
    fn emit_virtual_tables(&mut self) {
        let tables: Vec<(ClassId, Vec<Option<VirtualMethod>>)> = self
            .virtual_tables
            .iter()
            .filter(|(_, slots)| !slots.is_empty())
            .map(|(id, slots)| (*id, slots.clone()))
            .collect();
        if tables.is_empty() {
            return;
        }
        self.line("/* class virtual tables */");
        for (id, slots) in &tables {
            let name = ember_branding::vtable(&self.types.class_def(*id).name.to_string());
            self.line(&format!("struct {name} {{"));
            for (slot, signature) in slots.iter().enumerate() {
                let Some(signature) = signature else { continue };
                let signature = self
                    .virtual_signatures
                    .get(&(*id, slot))
                    .unwrap_or(signature);
                self.line(&format!("    {};", self.virtual_field_signature(signature, &format!("slot{slot}"))));
            }
            self.line("};");
        }
        self.line("");

        for (id, slots) in &tables {
            let class_name = self.types.class_def(*id).name.to_string();
            let table_name = ember_branding::vtable(&class_name);
            for (slot, implementation) in slots.iter().enumerate() {
                let Some(implementation) = implementation else { continue };
                if implementation.is_abstract {
                    continue;
                }
                let Some(signature) = self.virtual_signatures.get(&(*id, slot)).cloned() else {
                    continue;
                };
                if implementation.owner == signature.owner {
                    continue;
                }
                let adapter = format!("{table_name}_slot{slot}");
                let params = signature
                    .params
                    .iter()
                    .enumerate()
                    .map(|(index, ty)| format!("{} _{index}", self.c_member_type(*ty)))
                    .collect::<Vec<_>>();
                let params = if params.is_empty() { "void".to_string() } else { params.join(", ") };
                self.line(&format!("static {} {adapter}({params}) {{", self.c_type(signature.ret)));
                // A cast only where the C types differ (the receiver's class):
                // C has no cast to a struct type, even its own, and MSVC
                // refuses one (D-253).
                let args = implementation
                    .params
                    .iter()
                    .enumerate()
                    .map(|(index, ty)| {
                        let c_type = self.c_type(*ty);
                        let declared = signature.params.get(index).map(|declared| self.c_type(*declared));
                        if declared.as_deref() == Some(c_type.as_str()) {
                            format!("_{index}")
                        } else {
                            format!("({c_type})_{index}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                if self.is_void(signature.ret) {
                    self.line(&format!("    {}({args});", implementation.symbol));
                } else {
                    self.line(&format!("    return {}({args});", implementation.symbol));
                }
                self.line("}");
            }
        }
        self.line("");

        for (id, slots) in &tables {
            if !slots.iter().flatten().any(|method| !method.is_abstract) {
                continue;
            }
            let table_name = ember_branding::vtable(&self.types.class_def(*id).name.to_string());
            self.line(&format!("static const struct {table_name} {table_name} = {{"));
            for (slot, implementation) in slots.iter().enumerate() {
                let Some(implementation) = implementation else { continue };
                if implementation.is_abstract {
                    continue;
                }
                let Some(signature) = self.virtual_signatures.get(&(*id, slot)) else { continue };
                let value = if implementation.owner == signature.owner {
                    implementation.symbol.clone()
                } else {
                    format!("{table_name}_slot{slot}")
                };
                self.line(&format!("    {value},"));
            }
            self.line("};");
        }
        self.line("");
    }

    /// Emit the compiler-owned metadata consumed by the Phase 3 object
    /// runtime. The narrow memberwise construction slice installs the
    /// generated user-drop adapter, field-drop glue and class vtable when
    /// those values are source-reachable.
    ///
    /// The declarations are external-linkage `const` objects rather than
    /// `static` objects.  That avoids a compiler-warning for an as-yet-unused
    /// record in a translation unit containing only class field reads, while
    /// keeping the generated names compiler-owned and deterministic.
    fn emit_class_type_infos(&mut self) {
        let classes: Vec<ClassId> = self.types.runtime_classes().map(|(id, _)| id).collect();
        if classes.is_empty() {
            return;
        }
        self.line("/* class type information */");
        for id in &classes {
            let name = ember_branding::type_info(&self.types.class_def(*id).name.to_string());
            self.line(&format!(
                "extern const {} {name};",
                ember_branding::runtime("type_info")
            ));
        }
        for id in &classes {
            let Some(tables) = self.class_interface_tables.get(id).cloned() else { continue };
            if tables.is_empty() {
                continue;
            }
            let class = self.types.class_def(*id).name.to_string();
            let entries = class_itable_symbol(&class);
            self.line(&format!(
                "static const {} {entries}[] = {{",
                ember_branding::runtime("itable_entry")
            ));
            for (interface, concrete) in tables {
                let table = self.interface_adapter_table(concrete, &interface);
                self.line(&format!(
                    "    {{ &{}, &{table} }},",
                    interface_id_symbol(&interface)
                ));
            }
            self.line("};");
        }
        for id in classes {
            let def = self.types.class_def(id);
            let object = ember_branding::object_struct(&def.name.to_string());
            let info = ember_branding::type_info(&def.name.to_string());
            // `[EXC-19]` — every access word of the class, its bases' included,
            // for a whole-object access to find through the dynamic class.
            let words: Vec<String> = (0..self.types.class_field_count(id))
                .filter(|&index| self.types.class_field_has_access_word(id, index))
                .filter_map(|index| self.types.class_field_at(id, index))
                .map(|field| {
                    let name = field.name.to_string();
                    format!(
                        "    {{ {}, (uint32_t)offsetof(struct {object}, {}) }},",
                        c_string_literal(&name),
                        access_word_member(&name)
                    )
                })
                .collect();
            let fields = if words.is_empty() {
                None
            } else {
                let symbol = format!("{info}_access_words");
                self.line(&format!("static const {} {symbol}[] = {{", ember_branding::runtime("field_desc")));
                for word in &words {
                    self.line(word);
                }
                self.line("};");
                Some(symbol)
            };
            let base = def
                .base
                .map(|base| {
                    let name = self.types.class_def(base).name.to_string();
                    format!("&{}", ember_branding::type_info(&name))
                })
                .unwrap_or_else(|| "NULL".to_string());
            self.line(&format!(
                "const {} {info} = {{",
                ember_branding::runtime("type_info")
            ));
            self.line(&format!("    (uint32_t)sizeof(struct {object}),"));
            self.line(&format!("    (uint32_t)_Alignof(struct {object}),"));
            // `[CLS-8]`/`[THR-*]` synchronization derivation is a later
            // compiler phase.  Until it exists, all collected classes use
            // the plain-counter runtime path and carry no Sync flag.
            self.line("    UINT32_C(0),");
            self.line(&format!("    {},", c_string_literal(&def.name.to_string())));
            self.line(&format!("    {base},"));
            // Field-drop glue is installed below when the object has fields
            // that need destruction. Virtual tables are compiler-owned and
            // use the stable runtime type-info pointer boundary.
            if self.class_has_user_drop(id) {
                self.line(&format!(
                    "    &{},",
                    class_drop_adapter_symbol(&def.name.to_string())
                ));
            } else {
                self.line("    NULL,");
            }
            if self.class_has_dropping_fields(id) {
                self.line(&format!(
                    "    &{},",
                    class_drop_fields_symbol(&def.name.to_string())
                ));
            } else {
                self.line("    NULL,");
            }
            if self
                .virtual_tables
                .get(&id)
                .is_some_and(|slots| slots.iter().flatten().any(|method| !method.is_abstract))
            {
                let table = ember_branding::vtable(&def.name.to_string());
                self.line(&format!("    (const {}*)&{},", ember_branding::runtime("vtable"), table));
            } else {
                self.line("    NULL,");
            }
            if let Some(table_count) = self
                .class_interface_tables
                .get(&id)
                .map(|tables| tables.len())
                .filter(|count| *count != 0)
            {
                self.line(&format!(
                    "    {},",
                    class_itable_symbol(&def.name.to_string())
                ));
                self.line(&format!("    UINT32_C({table_count}),"));
            } else {
                self.line("    NULL,");
                self.line("    UINT32_C(0),");
            }
            match fields {
                Some(symbol) => {
                    self.line(&format!("    {symbol},"));
                    self.line(&format!("    UINT32_C({})", words.len()));
                }
                None => {
                    self.line("    NULL,");
                    self.line("    UINT32_C(0)");
                }
            }
            self.line("};");
        }
    }

    /// `[WK-8]` — generated code is the only layer that knows the concrete C
    /// layout of an Ember field. These callbacks expose ownership edges to the
    /// runtime's live-object SCC pass without giving the runtime permission to
    /// mutate the graph or to infer an edge from an untyped pointer.
    fn emit_debug_edge_enumerators(&mut self) {
        let classes: Vec<ClassId> = self.types.runtime_classes().map(|(id, _)| id).collect();
        let shareds: Vec<(StructId, Ty)> = self
            .types
            .structs()
            .filter_map(|(id, _)| self.shared_inner_id(id).map(|inner| (id, inner)))
            .collect();
        if classes.is_empty() && shareds.is_empty() {
            return;
        }
        let header = ember_branding::runtime("obj_header");
        let visitor = ember_branding::runtime("debug_edge_visitor");
        self.line("/* runtime ownership edge enumerators */");
        for id in classes {
            let def = self.types.class_def(id);
            let object = ember_branding::object_struct(&def.name.to_string());
            let symbol = class_debug_edges_symbol(&def.name.to_string());
            self.line(&format!(
                "static void {symbol}({header}* raw, {visitor} visitor, void* context) {{"
            ));
            self.line(&format!("    struct {object}* object = (struct {object}*)raw;"));
            self.line("    (void)object; (void)visitor; (void)context;");
            let mut loop_counter = 0;
            for field_index in 0..self.types.class_field_count(id) {
                let Some((field_owner, field)) = self.types.class_field_at_info(id, field_index) else {
                    continue;
                };
                let label = format!(
                    "{}.{}",
                    self.types.class_def(field_owner).name,
                    field.name
                );
                let weak_replacement = if self.types.class_def(field_owner).origin.is_none() {
                    match self.types.kind(field.ty) {
                        TyKind::Class(target) => Some(format!(
                            "Weak[{}]",
                            self.types.class_def(*target).name
                        )),
                        _ => None,
                    }
                } else {
                    None
                };
                let mut lines = Vec::new();
                self.debug_edge_lines(
                    &format!("object->{}", field.name),
                    field.ty,
                    &label,
                    weak_replacement.as_deref(),
                    true,
                    &mut loop_counter,
                    &mut lines,
                );
                for line in lines {
                    self.line(&format!("    {line}"));
                }
            }
            self.line("}");
        }
        for (id, inner) in shareds {
            let symbol = self.shared_debug_edges_symbol(id);
            let inner_c = self.c_type(inner);
            let display = self.types.display(inner);
            let label = format!("Shared[{display}].value");
            self.line(&format!(
                "static void {symbol}({header}* raw, {visitor} visitor, void* context) {{"
            ));
            self.line("    (void)raw; (void)visitor; (void)context;");
            let payload = format!(
                "(*(({inner_c}*){}(raw, {inner_c_align})))",
                ember_branding::runtime("obj_payload"), inner_c_align = c_align(&inner_c)
            );
            let mut lines = Vec::new();
            let mut loop_counter = 0;
            self.debug_edge_lines(
                &payload,
                inner,
                &label,
                None,
                false,
                &mut loop_counter,
                &mut lines,
            );
            for line in lines {
                self.line(&format!("    {line}"));
            }
            self.line("}");
        }
    }

    fn debug_edge_lines(
        &self,
        access: &str,
        ty: Ty,
        field: &str,
        weak_replacement: Option<&str>,
        statically_predicted: bool,
        loop_counter: &mut usize,
        out: &mut Vec<String>,
    ) {
        let header = ember_branding::runtime("obj_header");
        let field_c = c_string_literal(field);
        let replacement = weak_replacement
            .map(c_string_literal)
            .unwrap_or_else(|| "NULL".to_string());
        match self.types.kind(ty) {
            TyKind::Class(_) => out.push(format!(
                "visitor(context, ({header}*)({access}), {field_c}, EMBER_OWNERSHIP_EDGE_STRONG, {replacement}, {statically_predicted});"
            )),
            // An erased class-interface field is still a strong handle, but
            // its concrete target is deliberately unknown to static analysis.
            // The debug runtime receives the actual object header without
            // pretending that its class was predicted from the declaration.
            TyKind::ClassInterface(_) => out.push(format!(
                "visitor(context, ({header}*)({access}), {field_c}, EMBER_OWNERSHIP_EDGE_STRONG, NULL, false);"
            )),
            TyKind::Vec { elem, .. } => {
                let index = *loop_counter;
                *loop_counter += 1;
                out.push(format!("if (({access}).ptr != NULL) {{"));
                out.push(format!(
                    "    for (size_t _oe{index} = 0; _oe{index} < ({access}).len; ++_oe{index}) {{"
                ));
                self.debug_edge_lines(
                    &format!("(({}*)({access}).ptr)[_oe{index}]", self.c_type(*elem)),
                    *elem,
                    field,
                    None,
                    statically_predicted,
                    loop_counter,
                    out,
                );
                out.push("    }".to_string());
                out.push("}".to_string());
            }
            TyKind::Struct(id) => {
                if self.shared_inner_id(*id).is_some() {
                    out.push(format!(
                        "visitor(context, ({header}*)({access}), {field_c}, EMBER_OWNERSHIP_EDGE_STRONG, NULL, {statically_predicted});"
                    ));
                    return;
                }
                if self.weak_inner_id(*id).is_some() {
                    out.push(format!(
                        "visitor(context, ({header}*)(({access}).value), {field_c}, EMBER_OWNERSHIP_EDGE_WEAK, NULL, true);"
                    ));
                    return;
                }
                if let Some(inner) = self.box_inner_id(*id) {
                    out.push(format!("if (({access}) != NULL) {{"));
                    self.debug_edge_lines(
                        &format!("(*({access}))"),
                        inner,
                        field,
                        None,
                        statically_predicted,
                        loop_counter,
                        out,
                    );
                    out.push("}".to_string());
                    return;
                }
                for nested in &self.types.struct_def(*id).fields {
                    self.debug_edge_lines(
                        &format!("({access}).{}", nested.name),
                        nested.ty,
                        field,
                        None,
                        statically_predicted,
                        loop_counter,
                        out,
                    );
                }
            }
            TyKind::Enum(id) => {
                let mut arms = Vec::new();
                for (variant_index, variant) in self.types.enum_def(*id).variants.iter().enumerate() {
                    let mut lines = Vec::new();
                    for (field_index, nested) in variant.fields.iter().enumerate() {
                        self.debug_edge_lines(
                            &self.enum_member(*id, access, variant_index, field_index),
                            nested.ty,
                            field,
                            None,
                            statically_predicted,
                            loop_counter,
                            &mut lines,
                        );
                    }
                    if !lines.is_empty() {
                        arms.push(format!(
                            "case {}: {{ {} }} break;",
                            variant.discriminant,
                            lines.join(" ")
                        ));
                    }
                }
                if !arms.is_empty() {
                    out.push(format!("switch ({}) {{ {} default: break; }}", self.enum_tag(*id, access), arms.join(" ")));
                }
            }
            TyKind::Tuple(items) => {
                for (index, item) in items.iter().enumerate() {
                    self.debug_edge_lines(
                        &format!("({access})._{index}"),
                        *item,
                        field,
                        None,
                        statically_predicted,
                        loop_counter,
                        out,
                    );
                }
            }
            TyKind::Array { elem, len } => {
                let index = *loop_counter;
                *loop_counter += 1;
                out.push(format!("for (size_t _oe{index} = 0; _oe{index} < {len}; ++_oe{index}) {{"));
                self.debug_edge_lines(
                    &format!("({access})._0[_oe{index}]"),
                    *elem,
                    field,
                    None,
                    statically_predicted,
                    loop_counter,
                    out,
                );
                out.push("}".to_string());
            }
            TyKind::Dyn { .. } | TyKind::Param { .. } | TyKind::Assoc { .. } => out.push(format!(
                "visitor(context, NULL, {field_c}, EMBER_OWNERSHIP_EDGE_UNKNOWN, NULL, false);"
            )),
            TyKind::Bool
            | TyKind::Char
            | TyKind::Int(_)
            | TyKind::Uint(_)
            | TyKind::Float(_)
            | TyKind::Void
            | TyKind::Never
            | TyKind::Str
            | TyKind::Span { .. }
            | TyKind::Range(_)
            | TyKind::Ref { .. }
            | TyKind::Ptr { .. }
            | TyKind::Fn { .. }
            | TyKind::Infer(_)
            | TyKind::IntLit
            | TyKind::FloatLit
            | TyKind::Error => {}
        }
    }

    /// `[HEAP-3]`, `[HEAP-6]` — every `Shared[T]` uses the class-object
    /// control block, with a compiler-owned `TypeInfo` that describes the
    /// aligned payload and runs ordinary `T` destruction at final release.
    fn emit_shared_type_infos(&mut self) {
        let shareds: Vec<(StructId, Ty)> = self
            .types
            .structs()
            .filter_map(|(id, _)| self.shared_inner_id(id).map(|inner| (id, inner)))
            .collect();
        if shareds.is_empty() {
            return;
        }
        self.line("/* Shared value type information */");
        for (id, inner) in &shareds {
            if !self.types.needs_drop(*inner) {
                continue;
            }
            let drop = self.shared_drop_symbol(*id);
            let inner_c = self.c_type(*inner);
            let payload = format!(
                "(*(({inner_c}*){}(({}*)raw, {inner_c_align})))",
                ember_branding::runtime("obj_payload"),
                ember_branding::runtime("obj_header"), inner_c_align = c_align(&inner_c),
            );
            let mut lines = Vec::new();
            self.drop_lines(&payload, *inner, &mut lines);
            self.line(&format!("static void {drop}(void* raw) {{"));
            for line in lines {
                self.line(&format!("    {line}"));
            }
            self.line("}");
        }
        for (id, inner) in shareds {
            let info = self.shared_type_info_symbol(id);
            let inner_c = self.c_type(inner);
            let display = format!("Shared[{}]", self.types.display(inner));
            self.line(&format!("static const {} {info} = {{", ember_branding::runtime("type_info")));
            self.line(&format!(
                "    (uint32_t)(EMBER_OBJ_PAYLOAD_OFFSET({inner_c_align}) + {inner_c_size}),", inner_c_align = c_align(&inner_c), inner_c_size = c_size(&inner_c)
            ));
            self.line(&format!(
                "    (uint32_t)(({inner_c_align} > _Alignof({RT}obj_header)) ? {inner_c_align} : _Alignof({RT}obj_header)),", inner_c_align = c_align(&inner_c)
            ));
            self.line("    UINT32_C(0),");
            self.line(&format!("    {},", c_string_literal(&display)));
            self.line("    NULL,");
            if self.types.needs_drop(inner) {
                self.line(&format!("    &{},", self.shared_drop_symbol(id)));
            } else {
                self.line("    NULL,");
            }
            self.line("    NULL,");
            self.line("    NULL,");
            self.line("    NULL,");
            self.line("    UINT32_C(0),");
            self.line("    NULL,");
            self.line("    UINT32_C(0)");
            self.line("};");
        }
    }

    fn node_name(&self, node: TypeNode) -> String {
        match node {
            TypeNode::Struct(id) => c_name(&self.types.struct_def(id).name.to_string()),
            TypeNode::Class(id) => {
                ember_branding::object_struct(&self.types.class_def(id).name.to_string())
            }
            TypeNode::Enum(id) => c_name(&self.types.enum_def(id).name.to_string()),
            TypeNode::Structural(ty) => self.structural_name(ty),
        }
    }

    /// The payload of the compiler-known `Box[T]` structural wrapper, by its
    /// origin and the compiler's own module: a root module may declare a
    /// `Box[T]` of its own (D-305).
    fn box_inner_id(&self, id: StructId) -> Option<Ty> {
        self.types.compiler_box_inner(id)
    }

    /// The payload of a compiler-known `Shared[T]` counted handle. The C
    /// representation is the common runtime header pointer; its `T` payload
    /// lives immediately after that header at the alignment-aware offset.
    fn shared_inner_id(&self, id: StructId) -> Option<Ty> {
        let def = self.types.struct_def(id);
        match &def.origin {
            Some((name, args)) if name.is("Shared") && args.len() == 1 => Some(args[0]),
            _ => None,
        }
    }

    /// The payload of a compiler-known `Cell[T]` wrapper. Unlike ordinary
    /// source structs, its private generated definition belongs to no source
    /// module; that makes this check a compiler-internal capability boundary,
    /// not a spelling convention a program can forge.
    fn cell_inner_id(&self, id: StructId) -> Option<Ty> {
        let def = self.types.struct_def(id);
        (def.declaring_module == usize::MAX
            && def.name.as_str().starts_with("Cell_")
            && def.fields.len() == 1)
            .then_some(def.fields[0].ty)
    }

    fn shared_type_info_symbol(&self, id: StructId) -> String {
        format!("{}_type_info", c_name(&self.types.struct_def(id).name.to_string()))
    }

    fn shared_debug_edges_symbol(&self, id: StructId) -> String {
        format!("{}_debug_edges", c_name(&self.types.struct_def(id).name.to_string()))
    }

    fn shared_drop_symbol(&self, id: StructId) -> String {
        format!("{}_drop_payload", c_name(&self.types.struct_def(id).name.to_string()))
    }

    /// The counted-owner payload of a compiler-known `Weak[O]` wrapper. Unlike
    /// Box, Weak stays a one-field C struct so copies and drops can route
    /// through the runtime's separate weak-count operations.
    fn weak_inner_id(&self, id: StructId) -> Option<Ty> {
        let def = self.types.struct_def(id);
        match &def.origin {
            Some((name, args)) if name.is("Weak") && args.len() == 1 => Some(args[0]),
            _ => None,
        }
    }

    /// What one type definition looks like in C. Members are already written
    /// as declarators — an array member interleaves its type and its name, so
    /// this cannot be a `(type, name)` pair.
    fn definition(&self, node: TypeNode) -> Definition {
        match node {
            TypeNode::Struct(id) => {
                // Part XIX.6 maps `Box[T]` directly to `T*`. The checker uses
                // one private logical pointer field so ordinary projection
                // and ownership machinery can follow auto-deref; the backend
                // erases that wrapper rather than emitting a second struct.
                if let Some(inner) = self.box_inner_id(id) {
                    if matches!(self.types.kind(inner), TyKind::Dyn { .. }) {
                        return Definition::Alias(self.c_type(inner));
                    }
                    return Definition::Alias(format!("{}*", self.c_type(inner)));
                }
                if let Some(inner) = self.weak_inner_id(id) {
                    // `Shared[T]` itself is an alias for this same header
                    // pointer. Spelling the wrapper member with the runtime
                    // ABI type avoids a by-value dependency on an alias that
                    // may be emitted after its `Weak[Shared[T]]` wrapper.
                    let value = match self.types.kind(inner) {
                        TyKind::Struct(inner) if self.shared_inner_id(*inner).is_some() => {
                            format!("{RT}obj_header*")
                        }
                        _ => self.c_type(inner),
                    };
                    return Definition::Struct(vec![format!("{value} value")]);
                }
                if self.shared_inner_id(id).is_some() {
                    return Definition::Alias(format!("{}obj_header*", RT));
                }
                Definition::Struct(
                    self.types
                        .struct_def(id)
                        .fields
                        .iter()
                        .map(|f| format!("{} {}", self.c_member_type(f.ty), f.name))
                        .collect(),
                )
            }
            TypeNode::Class(id) => {
                // Class handles point at an object whose fixed runtime header
                // is followed by base fields and then derived fields. The
                // flattened member list matches the single-inheritance
                // layout while keeping field projections deterministic.
                let mut members = vec![format!("{RT}obj_header _header")];
                for index in 0..self.types.class_field_count(id) {
                    if let Some(field) = self.types.class_field_at(id, index) {
                        // `[EXC-19]` — a non-`Copy` field's access word stands
                        // in front of it.
                        if self.types.class_field_has_access_word(id, index) {
                            members.push(format!("uint32_t {}", access_word_member(&field.name.to_string())));
                        }
                        members.push(format!("{} {}", self.c_member_type(field.ty), field.name));
                    }
                }
                Definition::Struct(members)
            }
            // `[ENM-3]` — a unit-only enum *is* its discriminant, so it is a
            // name for the repr integer and nothing more. `[TYP-12]` — a
            // payload enum is `{tag, union of variants}`.
            TypeNode::Enum(id) => {
                let def = self.types.enum_def(id);
                if def.is_unit_only() {
                    return Definition::Alias(self.c_type(def.repr));
                }
                // `[TYP-13]` — an `Option` with a niche is its payload.
                if let Some(niche) = self.types.option_niche(id) {
                    return Definition::Alias(self.c_type(niche.payload));
                }
                let mut members = vec![format!("{} tag", self.c_type(def.repr))];
                let mut union = vec!["union {".to_string()];
                for variant in def.variants.iter().filter(|v| !v.fields.is_empty()) {
                    let fields: Vec<String> = variant
                        .fields
                        .iter()
                        // Ember's `void` is a unit value. A payload such as
                        // `Result[void, E].Ok` still has a semantic field for
                        // generic substitution, but C cannot declare a field
                        // of type `void`; one private byte is its portable,
                        // zero-information carrier.
                        .map(|f| {
                            let ty = if self.is_void(f.ty) {
                                "uint8_t".to_string()
                            } else {
                                self.c_type(f.ty)
                            };
                            format!("{ty} {}; ", f.name)
                        })
                        .collect();
                    union.push(format!("        struct {{ {}}} {};", fields.concat(), variant.name));
                }
                union.push("    } payload".to_string());
                members.push(union.join("\n"));
                Definition::Struct(members)
            }
            // `[CG-C-*]`'s "function pointer typedefs". A function-pointer
            // *type name* in C is `R (*)(A)`, which cannot be written where a
            // simple `T name` is wanted, so each one gets a typedef and every
            // use names it.
            TypeNode::Structural(ty) if matches!(self.types.kind(ty), TyKind::Fn { .. }) => {
                let TyKind::Fn { params, ret, .. } = self.types.kind(ty) else { unreachable!() };
                let rendered: Vec<String> = params
                    .iter()
                    .map(|param| self.callable_param_c_type(*param))
                    .collect();
                let args = if rendered.is_empty() { "void".to_string() } else { rendered.join(", ") };
                Definition::FnPointer { ret: self.c_type(*ret), args }
            }
            TypeNode::Structural(ty) => Definition::Struct(match self.types.kind(ty) {
                TyKind::Tuple(items) => items
                    .iter()
                    .enumerate()
                    .map(|(i, &item)| format!("{} _{i}", self.c_member_type(item)))
                    .collect(),
                // `[T; 0]` is legal in Ember; a zero-length array is not legal
                // C, so the member is padded to one element the same way an
                // empty struct is.
                TyKind::Array { elem, len } => {
                    vec![format!("{} _0[{}]", self.c_member_type(*elem), (*len).max(1))]
                }
                _ => Vec::new(),
            }),
        }
    }

    /// Part XVIII §4.9's drop glue, written straight into the C rather than
    /// through a synthesised function: what has to run for the value at
    /// `access` to release everything it owns.
    ///
    /// `[CELL-7]` — find the `RefCell[T]` for a `Ref[T]`/`RefMut[T]` guard's
    /// `T`, for the release in `drop_lines`. Guards and cells are ordinary
    /// structs here (ADR-019), so the link is structural: the cell is the
    /// `RefCell_…` struct whose field 0 holds this guard's `T`. Name-checked
    /// first (like `is_option`'s `Option_` prefix) so a user struct with the
    /// same shape does not capture the release.
    fn refcell_for_guard(&self, guard_ty: Ty) -> Option<Ty> {
        let TyKind::Struct(guard_id) = *self.types.kind(guard_ty) else {
            return None;
        };
        let guard_def = self.types.struct_def(guard_id);
        if guard_def.fields.len() != 1 {
            return None;
        }
        let inner = match self.types.kind(guard_def.fields[0].ty) {
            TyKind::Ref { inner, .. } => *inner,
            _ => return None,
        };
        for (ty, kind) in self.types.all() {
            let TyKind::Struct(cell_id) = kind else {
                continue;
            };
            let def = self.types.struct_def(*cell_id);
            if !def.name.as_str().starts_with("RefCell_") {
                continue;
            }
            if def.fields.len() != 4 {
                continue;
            }
            if def.fields[0].ty == inner {
                return Some(ty);
            }
        }
        None
    }

    /// `[DRP-2]` — a struct's fields drop after the struct's own `drop`, in
    /// reverse declaration order; an enum drops the active variant's payload;
    /// a tuple drops in reverse; an array drops its elements in index order.
    fn drop_lines(&self, access: &str, ty: Ty, out: &mut Vec<String>) {
        match self.types.kind(ty) {
            // `[OWN-7]` — class handles are language-level `Copy` values, but
            // the copy is a strong-reference copy rather than a raw pointer
            // duplicate.  The matching retain is emitted at the assignment
            // boundary; every ordinary class lifetime end releases one
            // strong reference through the runtime ABI.
            TyKind::Class(_) | TyKind::ClassInterface(_) => {
                out.push(format!(
                    "{}(({}*){access});",
                    ember_branding::runtime("release"),
                    ember_branding::runtime("obj_header")
                ));
            }
            TyKind::Vec { elem, .. } => {
                // An `Array[T]` owns its elements as well as its buffer.
                if self.types.needs_drop(*elem) {
                    // D-231 — one index per nesting level: an element that
                    // itself owns an `Array` (`Array[Bag]`, `Bag` holding an
                    // `Array[String]`) loops inside this loop, and a shared
                    // name would shadow the outer index.
                    let index = format!("_di{}", access.matches("[_di").count());
                    let mut inner = Vec::new();
                    let element = format!("(({}*){access}.ptr)[{index}]", self.c_type(*elem));
                    match self.drop_glue_call(&element, *elem) {
                        Some(call) => inner.push(call),
                        None => self.drop_lines(&element, *elem, &mut inner),
                    }
                    if !inner.is_empty() {
                        out.push(format!(
                            "for (size_t {index} = 0; {index} < {access}.len; ++{index}) {{ {} }}",
                            inner.join(" ")
                        ));
                    }
                }
                out.push(format!("{RT}vec_free(&{access}, {});", c_size(&self.c_type(*elem))));
            }
            TyKind::Struct(id) => {
                let def = self.types.struct_def(*id);
                if self.shared_inner_id(*id).is_some() {
                    out.push(format!(
                        "{}(({}*){access});",
                        ember_branding::runtime("release"),
                        ember_branding::runtime("obj_header")
                    ));
                    return;
                }
                if self.weak_inner_id(*id).is_some() {
                    out.push(format!(
                        "{}(({}*){access}.value);",
                        ember_branding::runtime("weak_release"),
                        ember_branding::runtime("obj_header")
                    ));
                    return;
                }
                if let Some(inner) = self.box_inner_id(*id)
                    && matches!(self.types.kind(inner), TyKind::Dyn { .. })
                {
                    let table = ember_branding::mangled("dyn_vtable_header");
                    out.push(format!(
                        "((const struct {table}*){access}.vtable)->drop({access}.data);"
                    ));
                    out.push(format!(
                        "if (((const struct {table}*){access}.vtable)->size != 0) {{ {RT}free({access}.data, ((const struct {table}*){access}.vtable)->size, ((const struct {table}*){access}.vtable)->align); }}"
                    ));
                    return;
                }
                // IX.1 / `[DRP-6]` — the compiler-known Box owns the value
                // behind its private pointer. Its drop order is observable:
                // destroy `T` first, then release exactly that allocation.
                // `drops_fields` is false, so this is the sole ownership walk.
                let compiler_box = self.types.compiler_box_inner(*id).is_some();
                if compiler_box && def.fields.len() == 1 {
                    if let TyKind::Ptr { inner, .. } = self.types.kind(def.fields[0].ty) {
                        if self.types.needs_drop(*inner) {
                            let payload = format!("(*{access})");
                            match self.drop_glue_call(&payload, *inner) {
                                Some(call) => out.push(call),
                                None => self.drop_lines(&payload, *inner, out),
                            }
                        }
                        let inner_c = self.c_type(*inner);
                        out.push(format!(
                            "{RT}free({access}, {inner_c_size}, {inner_c_align});", inner_c_size = c_size(&inner_c), inner_c_align = c_align(&inner_c)
                        ));
                        return;
                    }
                }
                // `[CELL-7]` — a `Ref`/`RefMut` guard's `drop` releases the
                // borrow state. The guard holds a pointer to the cell's `value`
                // (at offset 0, so the value pointer is also the cell pointer);
                // a shared guard decrements the counter, a mutable one resets
                // it to `0`. This is the only `has_drop` struct with no
                // `drop` method: the release is the drop glue, and there is no
                // `drop_symbol` to call.
                let guard_name = def.name.as_str();
                // `[ARN-1]`/`[ARN-2]` — Arena owns its chunks, but never
                // drops the values inside them individually. It has no
                // source-level destructor; this compiler-provided glue
                // releases only the opaque arena state.
                if guard_name == "Arena" {
                    out.push(format!("{RT}arena_free({access}.state);"));
                    return;
                }
                // `[ARN-6]` — a scope guard owns no values and no chunks. Its
                // destructor rewinds the shared arena to the captured mark;
                // the held parent borrow is compile-time state only.
                if guard_name == "ScopedArena" {
                    out.push(format!("{RT}arena_rewind({access}.state, {access}.mark);"));
                    return;
                }
                if guard_name.starts_with("RefMut_") || guard_name.starts_with("Ref_") {
                    if let Some(cell_ty) = self.refcell_for_guard(ty) {
                        let cell_c = self.c_type(cell_ty);
                        if guard_name.starts_with("RefMut_") {
                            out.push(format!(
                                "((({cell_c}*){access}.value)->borrow = 0);"
                            ));
                        } else {
                            out.push(format!(
                                "((({cell_c}*){access}.value)->borrow--);"
                            ));
                        }
                        return;
                    }
                    // If the cell cannot be found (user struct coincidentally
                    // named `Ref_…`), fall through to ordinary glue rather
                    // than emitting nothing: failing closed means rejecting a
                    // correct program is preferable to leaking a borrow.
                }
                // `[OWN-2]` — "its `drop` method (if any) runs, **then** its
                // fields are dropped in reverse declaration order". Both
                // halves, and in that order: the destructor still sees a whole
                // value, which is the only reason it can read its own fields.
                //
                // The call was missing entirely. A struct whose only claim on
                // `needs_drop` was its own `drop` method produced *no lines at
                // all* here, so the `Drop` statement lowered to nothing and a
                // declared destructor never ran.
                if def.has_drop {
                    let owner = self.types.symbol_name(ty);
                    out.push(format!("{}(&{access});", drop_symbol(&owner)));
                }
                if def.drops_fields {
                    let fields = def.fields.clone();
                    for field in fields.iter().rev() {
                        if self.types.needs_drop(field.ty) {
                            self.drop_lines(&format!("{access}.{}", field.name), field.ty, out);
                        }
                    }
                }
            }
            TyKind::Tuple(items) => {
                for (index, item) in items.iter().enumerate().rev() {
                    if self.types.needs_drop(*item) {
                        self.drop_lines(&format!("{access}._{index}"), *item, out);
                    }
                }
            }
            TyKind::Array { elem, len } => {
                if !self.types.needs_drop(*elem) {
                    return;
                }
                for index in 0..*len {
                    self.drop_lines(&format!("{access}._0[{index}]"), *elem, out);
                }
            }
            // `[DRP-2]` — only the active variant's payload is dropped, which
            // is a switch on the tag.
            TyKind::Enum(id) => {
                let def = self.types.enum_def(*id);
                // `[OWN-2]` again: the enum's own `drop` runs before its
                // payload is dropped, and a unit-only enum with a `drop` still
                // has one to run.
                if def.has_drop {
                    let owner = self.types.symbol_name(ty);
                    out.push(format!("{}(&{access});", drop_symbol(&owner)));
                }
                if def.is_unit_only() {
                    return;
                }
                let mut arms = Vec::new();
                for (variant_index, variant) in def.variants.iter().enumerate() {
                    let mut inner = Vec::new();
                    for (field_index, field) in variant.fields.iter().enumerate().rev() {
                        if self.types.needs_drop(field.ty) {
                            let member = self.enum_member(*id, access, variant_index, field_index);
                            match self.drop_glue_call(&member, field.ty) {
                                Some(call) => inner.push(call),
                                None => self.drop_lines(&member, field.ty, &mut inner),
                            }
                        }
                    }
                    if !inner.is_empty() {
                        arms.push(format!(
                            "case {}: {{ {} }} break;",
                            variant.discriminant,
                            inner.join(" ")
                        ));
                    }
                }
                if !arms.is_empty() {
                    out.push(format!(
                        "switch ({}) {{ {} default: break; }}",
                        self.enum_tag(*id, access),
                        arms.join(" ")
                    ));
                }
            }
            _ => {}
        }
    }

    /// `[RNG-3a]` — `T.clamped(v)`, "defined as `min(max(v, lo), hi)` for an
    /// inclusive range and as the nearest representable value strictly inside
    /// a half-open one".
    ///
    /// For a float representation the C library's `fmin`/`fmax` are used
    /// rather than a conditional, because they are what makes `NaN` map to
    /// `lo` as `[RNG-3a]` requires: `fmax(NaN, lo)` is `lo`, while
    /// `NaN < lo ? lo : NaN` is `NaN`.
    fn range_clamp(&self, id: ember_types::RangeId, value: &str) -> String {
        let def = self.types.range_def(id);
        // D-316 — an `f16` range clamps in `double` between `f16` endpoints,
        // so the result is exactly one: `hi` of a half-open range is the
        // greatest `f16` below its bound.
        if self.half(def.repr) {
            let endpoint = |bound: ember_types::Bound| match bound {
                ember_types::Bound::Float(v) => v,
                ember_types::Bound::Int(v) => v as f64,
            };
            let lo = ember_types::f16_value(ember_types::f16_bits(endpoint(def.lo)));
            let bound = endpoint(def.hi);
            let mut hi_bits = ember_types::f16_bits(bound);
            if !def.inclusive && ember_types::f16_value(hi_bits) >= bound {
                hi_bits = match hi_bits {
                    0x0000 | 0x8000 => 0x8001,
                    bits if bits & 0x8000 == 0 => bits - 1,
                    bits => bits + 1,
                };
            }
            let hi = ember_types::f16_value(hi_bits);
            return format!(
                "{RT}f64_to_f16(fmin(fmax({RT}f16_to_f64({value}), {}), {}))",
                render_float(lo, false),
                render_float(hi, false)
            );
        }
        let is_float = matches!(self.types.kind(def.repr), TyKind::Float(_));
        let lo = self.bound_literal(def.lo, def.repr);
        let hi = if def.inclusive {
            self.bound_literal(def.hi, def.repr)
        } else {
            // "the nearest representable value strictly inside a half-open
            // one".
            match def.hi {
                ember_types::Bound::Int(v) => self.bound_literal(ember_types::Bound::Int(v - 1), def.repr),
                ember_types::Bound::Float(_) => {
                    let hi = self.bound_literal(def.hi, def.repr);
                    let lo_lit = self.bound_literal(def.lo, def.repr);
                    let f = if matches!(self.types.kind(def.repr), TyKind::Float(FloatTy::F64)) {
                        "nextafter"
                    } else {
                        "nextafterf"
                    };
                    format!("{f}({hi}, {lo_lit})")
                }
            }
        };
        if is_float {
            let (fmin, fmax) =
                if matches!(self.types.kind(def.repr), TyKind::Float(FloatTy::F64)) {
                    ("fmin", "fmax")
                } else {
                    ("fminf", "fmaxf")
                };
            format!("{fmin}({fmax}(({value}), {lo}), {hi})")
        } else {
            // Two compares, as the rule says. The value is named twice, so it
            // is parenthesised and the caller has already spilled anything
            // with a side effect into a temporary.
            format!("(({value}) < ({lo}) ? ({lo}) : (({value}) > ({hi}) ? ({hi}) : ({value})))")
        }
    }

    /// A range endpoint as a C literal of the representation type.
    fn bound_literal(&self, bound: ember_types::Bound, repr: Ty) -> String {
        match bound {
            ember_types::Bound::Int(v) => v.to_string(),
            ember_types::Bound::Float(v) => {
                let suffix =
                    if matches!(self.types.kind(repr), TyKind::Float(FloatTy::F64)) { "" } else { "f" };
                if v.fract() == 0.0 && v.is_finite() {
                    format!("{v:.1}{suffix}")
                } else {
                    format!("{v}{suffix}")
                }
            }
        }
    }

    /// What a view was built from, through the borrow `[SPN-1]`'s coercion
    /// takes of it.
    fn span_source(&self, ty: Ty) -> Ty {
        match self.types.kind(ty) {
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => self.span_source(*inner),
            _ => ty,
        }
    }

    /// The element type of a `Span[T]`/`MutSpan[T]`, through any references
    /// the receiver arrived behind.
    fn span_element(&self, ty: Ty) -> Ty {
        match self.types.kind(ty) {
            TyKind::Span { elem, .. } => *elem,
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => self.span_element(*inner),
            _ => ty,
        }
    }

    /// The element type of an `Array[T]`, given the receiver's type — which
    /// may be a `ref mut Array[T]`, because `push` takes the receiver by
    /// reference.
    fn element_of(&self, ty: Ty) -> Ty {
        match self.types.kind(ty) {
            TyKind::Vec { elem, .. } => *elem,
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => self.element_of(*inner),
            _ => ty,
        }
    }

    /// The generated name of a tuple or fixed-array type.
    ///
    /// `plan_types` walks every interned type, so a missing name is a bug in
    /// this backend rather than anything a program can cause — and a silent
    /// `void*` here would produce C that compiles and computes the wrong
    /// thing, so it fails loudly instead.
    fn structural_name(&self, ty: Ty) -> String {
        match self.structural.get(&ty) {
            Some(name) => name.clone(),
            None => panic!("no generated C type for `{}`", self.types.symbol_name(ty)),
        }
    }

    /// `[FN-6]` — callable parameter modes are ordinary ABI adjustments.
    /// They remain entirely in the type/signature metadata. Ordinary `mut T`
    /// uses the same pointer parameter as a declaration, while `[FN-1a]`
    /// keeps `mut MutSpan[T]` by value: the move-only view itself is the
    /// exclusive access and wrapping it in another pointer would change the
    /// established call boundary.
    fn callable_param_c_type(&self, param: FnParam) -> String {
        match param.mode {
            // `[BRW-8]` (ODR-024) — as the function it points at declares it.
            FnParamMode::Borrow if self.types.passed_by_address(param.ty) => {
                format!("{}*", self.c_type(param.ty))
            }
            FnParamMode::Borrow | FnParamMode::Owned => self.c_member_type(param.ty),
            FnParamMode::Mut if matches!(self.types.kind(param.ty), TyKind::Span { mutable: true, .. }) => {
                self.c_type(param.ty)
            }
            FnParamMode::Mut => format!("{}*", self.c_type(param.ty)),
        }
    }

    fn emit_prototypes(&mut self, bodies: &[Body]) {
        if bodies.is_empty() {
            return;
        }
        self.line("/* prototypes */");
        for body in bodies {
            if body.is_abstract {
                continue;
            }
            let signature = self.signature(body);
            // Not `static`: modules share one translation unit, and a private
            // helper another module never calls would be an unused `static`
            // function, which `-Wall` reports and `[CG-C-1]` forbids. The
            // mangled names carry the module, so nothing can collide.
            self.line(&format!("{signature};"));
        }
        self.line("");
    }

    fn signature(&self, body: &Body) -> String {
        let ret = self.c_type(body.return_ty());
        let params: Vec<String> = body
            .args()
            .map(|(id, decl)| format!("{} _{}", self.c_member_type(decl.ty), id.0))
            .collect();
        let params = if params.is_empty() { "void".to_string() } else { params.join(", ") };
        format!("{ret} {}({params})", body.symbol)
    }

    fn emit_body(&mut self, body: &Body) {
        let signature = self.signature(body);
        self.line(&format!("{signature} {{"));
        self.interface_caches = interface_cache_plan(body);

        // Locals. Parameters are already C parameters; the return slot and
        // every other local are declared here.
        for (index, decl) in body.locals.iter().enumerate() {
            if decl.kind == LocalKind::Arg {
                continue;
            }
            if self.is_void(decl.ty) {
                continue;
            }
            let comment = match &decl.name {
                Some(name) => format!("  /* {name} */"),
                None => String::new(),
            };
            self.line(&format!("    {} _{index};{comment}", self.c_type(decl.ty)));
        }
        if body.locals.iter().any(|d| d.kind != LocalKind::Arg && !self.is_void(d.ty)) {
            self.line("");
        }

        // `[DSP-3]` — a repeated call through an unchanged class-interface
        // parameter reads one TypeInfo entry at function entry and carries its
        // opaque table pointer in a hidden C local. This includes a `mut I`
        // parameter: its reborrow is a temporary, but the parameter's handle
        // storage is the stable lookup source on every entry path.
        for (key, cache) in self.interface_caches.clone() {
            let receiver = match self.types.kind(body.locals[key.local].ty) {
                TyKind::Ref { inner, .. }
                    if matches!(self.types.kind(*inner), TyKind::ClassInterface(_)) =>
                {
                    format!("(*_{})", key.local)
                }
                _ => format!("_{}", key.local),
            };
            self.line(&format!(
                "    const void* {cache} = {}(((const {RT}obj_header*){receiver})->ti, &{});",
                ember_branding::runtime("itable_lookup"),
                interface_id_symbol(&key.interface),
            ));
        }
        if !self.interface_caches.is_empty() {
            self.line("");
        }

        // A pattern may bind a payload the arm never reads — `Circle(r): return 1`
        // is legal Ember — and the assignment that binds it then trips
        // `-Wunused-but-set-variable`. `[CG-C-1]` requires warning-free output,
        // so each such local is discarded once, explicitly.
        // A `void` local is never declared, so it cannot be discarded either.
        let unread: Vec<usize> = unread_locals(body)
            .into_iter()
            .filter(|index| !self.is_void(body.locals[*index].ty))
            .collect();
        if !unread.is_empty() {
            let discards: Vec<String> =
                unread.iter().map(|index| format!("(void)_{index};")).collect();
            self.line(&format!("    {}", discards.join(" ")));
            self.line("");
        }

        // Only blocks something actually jumps to get a label: an unreferenced
        // label is a warning under `-Wall` and MSVC's C4102, and `[CG-C-1]`
        // requires the emitted C to compile without warnings.
        let referenced = referenced_blocks(body);
        for (index, block) in body.blocks.iter().enumerate() {
            if referenced.contains(&index) {
                // A label with no statement after it is a syntax error in C,
                // so every label is followed by at least `;`.
                self.line(&format!("bb{index}: ;"));
            }
            for stmt in &block.stmts {
                self.emit_stmt(stmt, body);
            }
            self.emit_terminator(&block.terminator, body, index);
        }

        self.line("}");
        self.line("");
        self.interface_caches.clear();
    }

    fn emit_stmt(&mut self, stmt: &Stmt, body: &Body) {
        match &stmt.kind {
            StmtKind::Assign { place, rvalue } => {
                let ty = self.place_ty(place, body);
                if self.is_void(ty) {
                    return;
                }
                self.emit_line_directive(stmt.span);
                let lhs = self.place_in(place, body);
                // `[OWN-7]` — a class-handle copy retains before the new
                // pointer becomes visible in its destination.  This is kept
                // in the backend's explicit ownership boundary rather than
                // relying on C's pointer assignment, which has no Ember
                // reference-count semantics.  The walk is recursive so a
                // copied aggregate cannot silently duplicate an owned handle.
                let mut retains = Vec::new();
                self.retain_lines_for_rvalue(rvalue, ty, body, &mut retains);
                for line in retains {
                    self.line(&format!("    {line}"));
                }
                // `[value; count]` fills the array with a loop rather than
                // `count` copies of the operand's text, so a large array costs
                // one statement here and one in the emitted C.
                if let Rvalue::Repeat { value, count } = rvalue {
                    let value = self.operand(value, body);
                    self.line(&format!(
                        "    for (size_t _i = 0; _i < {count}u; ++_i) {{ {lhs}._0[_i] = {value}; }}"
                    ));
                    return;
                }
                // MSVC expands the omitted payload of a unit variant's
                // compound initializer recursively. With a deeply nested
                // Option this exceeds its initializer-depth limit, even
                // though only the tag is live (D-360). Zero the storage at
                // this MIR assignment boundary, then set the discriminant.
                if let Rvalue::Aggregate { kind: AggregateKind::Enum(id, variant), operands } = rvalue
                    && operands.is_empty()
                    && !self.types.enum_def(*id).is_unit_only()
                    && self.types.option_niche(*id).is_none()
                {
                    let tag = self.types.enum_def(*id).variants[*variant].discriminant;
                    self.line(&format!("    memset(&({lhs}), 0, sizeof({lhs}));"));
                    self.line(&format!("    ({lhs}).tag = {tag};"));
                    return;
                }
                let rhs = self.rvalue(rvalue, body, ty);
                self.line(&format!("    {lhs} = {rhs};"));
            }
            StmtKind::BeginAccess { place, mutable } => {
                self.emit_line_directive(stmt.span);
                let operation = if *mutable { "begin_write" } else { "begin_read" };
                self.line(&format!("    {};", self.access_call(place, operation, stmt.span, body)));
            }
            StmtKind::BeginAccessTransfer { place, mutable } => {
                self.emit_line_directive(stmt.span);
                let operation = if *mutable { "begin_write" } else { "begin_read" };
                let object = self.access_object(place, body);
                let what = c_string_literal(&body.symbol);
                let location = self.location(stmt.span);
                self.line(&format!(
                    "    {RT}access_{operation}({object}, {what}, {location});"
                ));
            }
            StmtKind::EndAccess { place, mutable } => {
                self.emit_line_directive(stmt.span);
                let operation = if *mutable { "end_write" } else { "end_read" };
                self.line(&format!("    {};", self.access_call(place, operation, stmt.span, body)));
            }
            StmtKind::EndAccessTransfer { place, mutable } => {
                self.emit_line_directive(stmt.span);
                let operation = if *mutable { "end_write" } else { "end_read" };
                let object = self.access_object_from_payload(place, body);
                let what = c_string_literal(&body.symbol);
                let location = self.location(stmt.span);
                self.line(&format!(
                    "    {RT}access_{operation}({object}, {what}, {location});"
                ));
            }
            StmtKind::CheckedBinaryOp { dest, overflow, op, lhs, rhs } => {
                // `bool ember_ck_add_i32(a, b, &dest)` returns whether the
                // operation overflowed and writes the wrapped result either
                // way, so the value is defined on both paths.
                let ty = self.place_ty(dest, body);
                let helper = format!("{RT}ck_{}_{}", checked_name(*op), self.checked_suffix(ty));
                self.emit_line_directive(stmt.span);
                self.line(&format!(
                    "    {} = {helper}({}, {}, &{});",
                    self.place_in(overflow, body),
                    self.operand(lhs, body),
                    self.operand(rhs, body),
                    self.place_in(dest, body)
                ));
            }
            // `[OWN-2]`, `[DRP-2]` — the value's life ends here.
            StmtKind::Drop { place, flag } => {
                let ty = self.place_ty(place, body);
                let mut lines = Vec::new();
                self.drop_lines(&self.place_in(place, body), ty, &mut lines);
                if lines.is_empty() {
                    return;
                }
                self.emit_line_directive(stmt.span);
                // `[OWN-3]` — a value moved on some paths and not others is
                // dropped behind its flag.
                match flag {
                    Some(flag) => {
                        self.line(&format!("    if (_{}) {{ {} }}", flag.0, lines.join(" ")));
                    }
                    None => {
                        for line in lines {
                            self.line(&format!("    {line}"));
                        }
                    }
                }
            }
            // Storage markers carry no code in C; the borrow checker and drop
            // elaboration consume them before this point.
            StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
        }
    }

    /// Render the object pointer represented by a class access place. The
    /// current producer is the dereferenced `ref mut Class` receiver, but the
    /// place renderer remains the single source of truth for projections.
    /// The runtime call that begins or ends (`operation`) an access to
    /// `place`: a class field's own word (`[EXC-19]`), every word of a class
    /// object's dynamic class (`[EXC-15]`), or a `Shared` payload's header
    /// word (`[HEAP-5]`).
    fn access_call(&self, place: &Place, operation: &str, span: ember_span::Span, body: &Body) -> String {
        let location = self.location(span);
        // A class-typed place is the object, even when it is a field holding
        // a handle (`holder.child`), which has no word of its own.
        if matches!(self.types.kind(self.place_ty(place, body)), TyKind::Class(_) | TyKind::ClassInterface(_)) {
            return format!("{RT}object_{operation}({}, {location})", self.access_object(place, body));
        }
        if let Some((object, field)) = self.class_field_of_place(place, body) {
            let def = self.types.class_def(object);
            let what = c_string_literal(&format!("{}.{}", def.name, field));
            let handle = Place { local: place.local, projection: place.projection[..place.projection.len() - 1].to_vec() };
            return format!(
                "{RT}field_{operation}(&({})->{}, {what}, {location})",
                self.place_in(&handle, body),
                access_word_member(&field)
            );
        }
        let what = c_string_literal(&body.symbol);
        format!("{RT}access_{operation}({}, {what}, {location})", self.access_object(place, body))
    }

    /// When `place` is a class handle's field (`h.f`), the class and the
    /// field's name.
    fn class_field_of_place(&self, place: &Place, body: &Body) -> Option<(ClassId, String)> {
        let (Projection::Field(index), prefix) = place.projection.split_last()? else { return None };
        let handle = Place { local: place.local, projection: prefix.to_vec() };
        let TyKind::Class(id) = *self.types.kind(self.place_ty(&handle, body)) else { return None };
        Some((id, self.types.class_field_at(id, *index)?.name.to_string()))
    }

    fn access_object(&self, place: &Place, body: &Body) -> String {
        format!(
            "(({}*){})",
            ember_branding::runtime("obj_header"),
            self.place_in(place, body)
        )
    }

    /// A returned `Shared` payload reference carries a pointer to `T`, not an
    /// object-header pointer. The header lives at the alignment-defined offset
    /// used by `ember_obj_new_copy`, so the caller can close a transferred
    /// runtime access without a hidden owner word in the reference ABI.
    fn access_object_from_payload(&self, place: &Place, body: &Body) -> String {
        let payload = self.place_in(place, body);
        let payload_ty = self.c_type(self.place_ty(place, body));
        format!(
            "(({}*)((unsigned char*)&({payload}) - EMBER_OBJ_PAYLOAD_OFFSET({payload_ty_align})))",
            ember_branding::runtime("obj_header"), payload_ty_align = c_align(&payload_ty),
        )
    }

    /// Emit the strong-reference increments required by an Ember `Copy`
    /// operation. Class handles are the one language-level `Copy` category
    /// whose C representation cannot be copied bitwise without changing
    /// ownership. Moves transfer the existing reference and therefore do not
    /// retain. A copied aggregate is walked recursively so its nested class
    /// handles receive the same ownership treatment as a direct handle.
    fn retain_lines_for_value(&self, access: &str, ty: Ty, out: &mut Vec<String>) {
        match self.types.kind(ty) {
            TyKind::Class(_) | TyKind::ClassInterface(_) => {
                out.push(format!(
                    "{}(({}*){});",
                    ember_branding::runtime("retain"),
                    ember_branding::runtime("obj_header"),
                    access
                ));
            }
            TyKind::Struct(id) => {
                let def = self.types.struct_def(*id);
                if self.shared_inner_id(*id).is_some() {
                    out.push(format!(
                        "{}(({}*){access});",
                        ember_branding::runtime("retain"),
                        ember_branding::runtime("obj_header")
                    ));
                    return;
                }
                if self.weak_inner_id(*id).is_some() {
                    out.push(format!(
                        "{}(({}*){access}.value);",
                        ember_branding::runtime("weak_retain"),
                        ember_branding::runtime("obj_header")
                    ));
                    return;
                }
                // `[ARN-8]` — MaybeUninit's field is layout storage, not an
                // initialized owner. Copying it is a byte copy and must not
                // retain a value that may not exist yet.
                if def.name.as_str().starts_with("MaybeUninit_") || !def.drops_fields {
                    return;
                }
                let fields: Vec<(Symbol, Ty)> =
                    def.fields.iter().map(|field| (field.name, field.ty)).collect();
                for (name, field_ty) in fields {
                    self.retain_lines_for_value(&format!("{access}.{name}"), field_ty, out);
                }
            }
            TyKind::Tuple(items) => {
                let items = items.clone();
                for (index, item) in items.into_iter().enumerate() {
                    self.retain_lines_for_value(&format!("{access}._{index}"), item, out);
                }
            }
            TyKind::Array { elem, len } => {
                for index in 0..*len {
                    self.retain_lines_for_value(&format!("{access}._0[{index}]"), *elem, out);
                }
            }
            TyKind::Enum(id) => {
                let variants: Vec<(i128, Symbol, Vec<(Symbol, Ty)>)> = self
                    .types
                    .enum_def(*id)
                    .variants
                    .iter()
                    .map(|variant| {
                        (
                            variant.discriminant,
                            variant.name,
                            variant.fields.iter().map(|field| (field.name, field.ty)).collect(),
                        )
                    })
                    .collect();
                let mut arms = Vec::new();
                for (variant_index, (discriminant, _, fields)) in variants.into_iter().enumerate() {
                    let mut retains = Vec::new();
                    for (field_index, (_, field_ty)) in fields.into_iter().enumerate() {
                        let member = self.enum_member(*id, access, variant_index, field_index);
                        self.retain_lines_for_value(&member, field_ty, &mut retains);
                    }
                    if !retains.is_empty() {
                        arms.push(format!(
                            "case {discriminant}: {{ {} }} break;",
                            retains.join(" ")
                        ));
                    }
                }
                if !arms.is_empty() {
                    out.push(format!(
                        "switch ({}) {{ {} default: break; }}",
                        self.enum_tag(*id, access),
                        arms.join(" ")
                    ));
                }
            }
            // Views, raw pointers and scalar values have no owned class
            // handle hidden in their representation. Growable arrays are
            // move-only, so no valid `Copy` boundary reaches this arm.
            _ => {}
        }
    }

    fn retain_lines_for_rvalue(
        &self,
        rvalue: &Rvalue,
        target: Ty,
        body: &Body,
        out: &mut Vec<String>,
    ) {
        match rvalue {
            Rvalue::Use(Operand::Copy(place)) => {
                self.retain_lines_for_value(&self.place_in(place, body), target, out);
            }
            Rvalue::Aggregate { kind, operands } => {
                // Aggregate construction is still a copy boundary when an
                // operand is `Copy`. Derive each operand's type from the
                // aggregate shape so nested class handles are retained before
                // the C initializer duplicates their pointer bits.
                let operand_types: Vec<Ty> = match kind {
                    AggregateKind::Struct(id) => self
                        .types
                        .struct_def(*id)
                        .fields
                        .iter()
                        .map(|field| field.ty)
                        .collect(),
                    AggregateKind::Tuple => match self.types.kind(target) {
                        TyKind::Tuple(items) => items.clone(),
                        _ => Vec::new(),
                    },
                    AggregateKind::Array => match self.types.kind(target) {
                        TyKind::Array { elem, len } => vec![*elem; *len as usize],
                        _ => Vec::new(),
                    },
                    AggregateKind::Enum(id, variant) => self
                        .types
                        .enum_def(*id)
                        .variants
                        .get(*variant)
                        .map(|variant| variant.fields.iter().map(|field| field.ty).collect())
                        .unwrap_or_default(),
                };
                for (operand, operand_ty) in operands.iter().zip(operand_types) {
                    if matches!(operand, Operand::Copy(_)) {
                        let value = self.operand(operand, body);
                        self.retain_lines_for_value(&value, operand_ty, out);
                    }
                }
            }
            Rvalue::Cast { kind: CastKind::ClassUpcast, operand: Operand::Copy(place), .. } => {
                out.push(format!(
                    "{}(({}*){});",
                    ember_branding::runtime("retain"),
                    ember_branding::runtime("obj_header"),
                    self.place_in(place, body)
                ));
            }
            Rvalue::Cast {
                kind: CastKind::ClassInterfaceUpcast { .. },
                operand: Operand::Copy(place),
                ..
            } => {
                out.push(format!(
                    "{}(({}*){});",
                    ember_branding::runtime("retain"),
                    ember_branding::runtime("obj_header"),
                    self.place_in(place, body)
                ));
            }
            _ => {}
        }
    }

    /// `#line` maps every emitted statement back to its Ember source
    /// (Part XVIII §6, "Debug info").
    fn emit_line_directive(&mut self, span: ember_span::Span) {
        if !self.line_directives || span.is_dummy() || self.map.get(span.file).is_none() {
            return;
        }
        let file = self.map.file(span.file);
        let line = file.line_of(span.start);
        // Forward slashes: a backslash in a `#line` path is an escape.
        let path = file.path.display().to_string().replace(MAIN_SEPARATOR, "/");
        let _ = writeln!(self.out, "#line {line} \"{path}\"");
    }

    fn emit_terminator(&mut self, terminator: &Terminator, body: &Body, index: usize) {
        match terminator {
            Terminator::Goto(target) => {
                // A jump to the next block in order is a fallthrough.
                if target.0 as usize == index + 1 {
                    self.line("    /* fallthrough */");
                } else {
                    self.line(&format!("    goto bb{};", target.0));
                }
            }
            Terminator::SwitchInt { discr, targets, otherwise } => {
                // D-272 — C's `switch` takes no 128-bit value on MSVC.
                if let Some(suffix) = self.operand_type(discr, body).and_then(|ty| self.wide_int(ty)) {
                    let discr = self.operand(discr, body);
                    for (value, target) in targets {
                        let value = self.wide_constant(*value as u128, suffix);
                        self.line(&format!("    if ({RT}{suffix}_eq({discr}, {value})) {{ goto bb{}; }}", target.0));
                    }
                    self.line(&format!("    goto bb{};", otherwise.0));
                    return;
                }
                let discr_ty = self.operand_type(discr, body).filter(|ty| self.types.is_integral(*ty));
                let values: Vec<String> = targets
                    .iter()
                    .map(|(value, _)| match discr_ty {
                        Some(ty) => self.int_literal(*value as u128, ty),
                        None => value.to_string(),
                    })
                    .collect();
                let discr = self.operand(discr, body);
                // Phase 0 only produces two-way branches on a boolean.
                if let [(_, target)] = targets.as_slice() {
                    self.line(&format!(
                        "    if ({discr} == {}) {{ goto bb{}; }} else {{ goto bb{}; }}",
                        values[0], target.0, otherwise.0
                    ));
                } else {
                    self.line(&format!("    switch ({discr}) {{"));
                    for ((_, target), value) in targets.iter().zip(&values) {
                        self.line(&format!("    case {value}: goto bb{};", target.0));
                    }
                    self.line(&format!("    default: goto bb{};", otherwise.0));
                    self.line("    }");
                }
            }
            Terminator::Assert { cond, expected, msg, next, span } => {
                let cond = self.operand(cond, body);
                let negate = if *expected { "!" } else { "" };
                let location = self.location(*span);
                let call = match msg {
                    AssertKind::Overflow(op) => {
                        format!("{RT}panic_overflow(\"{}\", {location})", op.spelling())
                    }
                    AssertKind::SignedDivisionOverflow => {
                        format!("{RT}panic_overflow(\"/\", {location})")
                    }
                    AssertKind::ShiftTooLarge => {
                        format!("{RT}panic_overflow(\"shift\", {location})")
                    }
                    AssertKind::DivisionByZero => format!("{RT}panic_div_zero({location})"),
                    AssertKind::Panic { message } => {
                        let message = self.operand(message, body);
                        format!("{RT}panic((const char*)({message}).ptr, ({message}).len, {location})")
                    }
                    AssertKind::Bounds { len, index: at } => format!(
                        "{RT}panic_bounds({}, {}, {location})",
                        self.operand(at, body),
                        self.operand(len, body)
                    ),
                    // `[CELL-5]` — contention panics with the conflicting
                    // borrow's location, recorded in debug and release
                    // (`[CELL-9]`). `file` is the cell's `borrow_file`
                    // (`const uint8_t*`, cast back for the call) and `line`
                    // its `borrow_line`; `location` is the failing borrow.
                    AssertKind::RefCellBorrow { file, line } => format!(
                        "{RT}panic_refcell((const char*){}, {}, {location})",
                        self.operand(file, body),
                        self.operand(line, body)
                    ),
                    AssertKind::Downcast => format!(
                        "{RT}panic(\"invalid downcast\", sizeof(\"invalid downcast\") - 1, {location})"
                    ),
                };
                self.line(&format!("    if ({negate}{cond}) {{ {call}; }}"));
                if next.0 as usize == index + 1 {
                    self.line("    /* fallthrough */");
                } else {
                    self.line(&format!("    goto bb{};", next.0));
                }
            }
            Terminator::Return => {
                if self.is_void(body.return_ty()) {
                    self.line("    return;");
                } else {
                    self.line("    return _0;");
                }
            }
            Terminator::Unreachable => {
                self.line("    EMBER_UNREACHABLE();");
            }
            Terminator::Call {
                func: FuncRef::Builtin {
                    which: Builtin::MemReplace { elem },
                    ..
                },
                args,
                dest,
                next,
            } => {
                let target = self.operand(&args[0], body);
                let value = self.operand(&args[1], body);
                let dest = self.place_in(dest, body);
                let elem = self.c_type(*elem);
                self.line(&format!("    {dest} = *(({elem}*)({target}));"));
                self.line(&format!("    *(({elem}*)({target})) = {value};"));
                if next.0 as usize == index + 1 {
                    self.line("    /* fallthrough */");
                } else {
                    self.line(&format!("    goto bb{};", next.0));
                }
            }
            Terminator::Call {
                func: FuncRef::Builtin {
                    which: Builtin::MemSwap { elem },
                    ..
                },
                args,
                next,
                ..
            } => {
                let left = self.operand(&args[0], body);
                let right = self.operand(&args[1], body);
                let elem = self.c_type(*elem);
                let temp = format!("_ember_swap_{index}");
                self.line(&format!(
                    "    {{ {elem} {temp} = *(({elem}*)({left})); \
                     *(({elem}*)({left})) = *(({elem}*)({right})); \
                     *(({elem}*)({right})) = {temp}; }}"
                ));
                if next.0 as usize == index + 1 {
                    self.line("    /* fallthrough */");
                } else {
                    self.line(&format!("    goto bb{};", next.0));
                }
            }
            Terminator::Call {
                func: FuncRef::Builtin {
                    which: Builtin::MemForget { .. },
                    ..
                },
                args,
                next,
                ..
            } => {
                let value = self.operand(&args[0], body);
                self.line(&format!("    (void)({value}); /* mem.forget */"));
                if next.0 as usize == index + 1 {
                    self.line("    /* fallthrough */");
                } else {
                    self.line(&format!("    goto bb{};", next.0));
                }
            }
            Terminator::Call {
                func: func @ FuncRef::Builtin {
                    which: Builtin::ArrayPush,
                    arg_ty,
                },
                args,
                next,
                ..
            } => {
                // `[RC-1]` — `Array.push` consumes its value argument, but a
                // `Copy` value may contain class handles. The byte copy
                // performed by `ember_vec_push` therefore needs the same
                // recursive strong-reference increments as any other copy;
                // otherwise the caller's later release leaves the array with
                // a dangling handle.
                let elem = self.element_of(*arg_ty);
                if matches!(args.get(1), Some(Operand::Copy(_))) {
                    let value = self.operand(&args[1], body);
                    let mut retains = Vec::new();
                    self.retain_lines_for_value(&value, elem, &mut retains);
                    for line in retains {
                        self.line(&format!("    {line}"));
                    }
                }
                let call = self.call_expression(func, args, body);
                self.line(&format!("    {call};"));
                if next.0 as usize == index + 1 {
                    self.line("    /* fallthrough */");
                } else {
                    self.line(&format!("    goto bb{};", next.0));
                }
            }
            Terminator::Call { func, args, dest, next } => {
                if let FuncRef::Builtin { which: Builtin::CloneParts { ty }, .. } = func {
                    let source = self.operand(&args[0], body);
                    let target = self.place_in(dest, body);
                    let helper = self.clone_parts_helper(*ty);
                    self.line(&format!("    {helper}({source}, &({target}));"));
                    if next.0 as usize == index + 1 {
                        self.line("    /* fallthrough */");
                    } else {
                        self.line(&format!("    goto bb{};", next.0));
                    }
                    return;
                }
                // `[RC-1]`/`[FN-1]` — a class handle is `Copy`, but its C
                // representation is only one pointer word. Passing a copied
                // handle to an owned direct parameter therefore needs an
                // explicit retain before the callee becomes responsible for
                // releasing its parameter. A moved operand transfers an
                // existing owner and deliberately does not retain.
                let mut retains = Vec::new();
                for (index, argument) in args.iter().enumerate() {
                    if !self.call_argument_is_owned(func, index, body) {
                        continue;
                    }
                    let Operand::Copy(place) = argument else {
                        continue;
                    };
                    let value = self.place_in(place, body);
                    self.retain_lines_for_value(&value, self.place_ty(place, body), &mut retains);
                }
                for line in retains {
                    self.line(&format!("    {line}"));
                }
                // `[STD-10]` — `input` panics at end of input, at its own call.
                let call = if matches!(func, FuncRef::Builtin { which: Builtin::Input, .. }) {
                    let location = self.location(body.blocks[index].terminator_span);
                    format!("{RT}input({}, {location})", self.operand(&args[0], body))
                } else {
                    self.call_expression(func, args, body)
                };
                let dest_ty = self.place_ty(dest, body);
                if self.is_void(dest_ty) {
                    self.line(&format!("    {call};"));
                } else {
                    let dest_text = self.place_in(dest, body);
                    self.line(&format!("    {dest_text} = {call};"));
                }
                if next.0 as usize == index + 1 {
                    self.line("    /* fallthrough */");
                } else {
                    self.line(&format!("    goto bb{};", next.0));
                }
            }
        }
    }

    /// An `ember_loc` for a panic, from the span of the operation itself.
    fn location(&self, span: ember_span::Span) -> String {
        if span.is_dummy() || self.map.get(span.file).is_none() {
            return format!("{RT}loc_unknown()");
        }
        let file = self.map.file(span.file);
        let lc = file.line_col(span.start);
        let path = file.path.display().to_string().replace('\\', "/");
        format!("{RT}loc_at({}, {}, {})", c_string_literal(&path), lc.line, lc.col)
    }

    /// The `ember_ck_*` suffix for a type: the helpers are per width and
    /// signedness.
    /// The type of an operand, where the backend can name it.
    fn operand_type(&self, operand: &Operand, body: &Body) -> Option<Ty> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => Some(self.place_ty(place, body)),
            Operand::Const(Const::Int { ty, .. } | Const::Float { ty, .. }) => Some(*ty),
            Operand::Const(_) => None,
        }
    }

    fn checked_suffix(&self, ty: ember_types::Ty) -> String {
        match self.types.kind(ty) {
            TyKind::Int(i) => match i {
                IntTy::I8 => "i8",
                IntTy::I16 => "i16",
                IntTy::I32 => "i32",
                IntTy::I64 => "i64",
                IntTy::I128 => "i128",
                IntTy::Isize => "isize",
            },
            TyKind::Uint(u) => match u {
                UintTy::U8 => "u8",
                UintTy::U16 => "u16",
                UintTy::U32 => "u32",
                UintTy::U64 => "u64",
                UintTy::U128 => "u128",
                UintTy::Usize => "usize",
            },
            _ => "i64",
        }
        .to_string()
    }

    /// D-272 — `i128` and `u128` are the runtime's `ember_i128`/`ember_u128`,
    /// which are not C integers on every compiler (MSVC has no 128-bit
    /// integer), so every operation on one is a runtime helper named with
    /// this suffix.
    fn wide_int(&self, ty: Ty) -> Option<&'static str> {
        match self.types.kind(ty) {
            TyKind::Int(IntTy::I128) => Some("i128"),
            TyKind::Uint(UintTy::U128) => Some("u128"),
            _ => None,
        }
    }

    /// A C constant of a narrower integer type, from its two's-complement
    /// bits (sign-extended or not): unsigned ones with their suffix, negative
    /// ones negated in parentheses, and a signed type's least value as
    /// `(-MAX - 1)`, since C reads `-9223372036854775808` as a minus applied
    /// to a literal too large for any signed type (D-311).
    fn int_literal(&self, bits: u128, ty: Ty) -> String {
        let width = ember_types::bit_width(self.types, ty).unwrap_or(64);
        let suffix = match self.types.kind(ty) {
            TyKind::Int(IntTy::I64 | IntTy::Isize) => "LL",
            TyKind::Uint(UintTy::U64 | UintTy::Usize) => "ULL",
            TyKind::Uint(_) => "U",
            _ => "",
        };
        let low = if width >= 128 { bits } else { bits & ((1u128 << width) - 1) };
        if !matches!(self.types.kind(ty), TyKind::Int(_)) || low >> (width - 1) == 0 {
            return format!("{low}{suffix}");
        }
        let magnitude = (1u128 << width) - low;
        if magnitude == 1u128 << (width - 1) {
            return format!("(-{}{suffix} - 1)", magnitude - 1);
        }
        format!("(-{magnitude}{suffix})")
    }

    /// A 128-bit constant from its two's-complement bits.
    fn wide_constant(&self, bits: u128, suffix: &str) -> String {
        format!("{RT}{suffix}_make(UINT64_C({:#x}), UINT64_C({:#x}))", (bits >> 64) as u64, bits as u64)
    }

    /// A C value of the type `from` as a 128-bit value: sign-extended from a
    /// signed integer or an enum's discriminant, zero-extended from anything
    /// else, reinterpreted from the other 128-bit type.
    fn to_wide(&self, value: &str, from: Ty, suffix: &str) -> String {
        if let Some(from_suffix) = self.wide_int(from) {
            if from_suffix == suffix {
                return value.to_string();
            }
            return format!("{RT}{from_suffix}_to_{suffix}({value})");
        }
        match self.types.kind(from) {
            TyKind::Int(_) | TyKind::Enum(_) => format!("{RT}{suffix}_from_i64((int64_t)({value}))"),
            _ => format!("{RT}{suffix}_from_u64((uint64_t)({value}))"),
        }
    }

    /// The 128-bit type an operation on these operands works in, when either
    /// operand has one.
    fn wide_operands(&self, lhs: &Operand, rhs: &Operand, body: &Body) -> Option<Ty> {
        [lhs, rhs]
            .into_iter()
            .filter_map(|operand| self.operand_type(operand, body))
            .find(|ty| self.wide_int(*ty).is_some())
    }

    /// An operand as a value of the 128-bit type `wide`: a constant is
    /// rendered in it, anything of another type converted.
    fn wide_operand(&self, operand: &Operand, body: &Body, wide: Ty) -> String {
        let suffix = self.wide_int(wide).expect("a 128-bit type");
        match (operand, self.operand_type(operand, body)) {
            (_, Some(ty)) if ty == wide => self.operand(operand, body),
            (Operand::Const(Const::Int { value, .. }), _) => self.wide_constant(*value, suffix),
            (_, Some(ty)) => self.to_wide(&self.operand(operand, body), ty, suffix),
            (_, None) => self.operand(operand, body),
        }
    }

    /// A shift amount as the helpers take it: its low bits, which the checks
    /// before the shift have already bounded (`[TYP-10]`).
    fn shift_amount(&self, operand: &Operand, body: &Body) -> String {
        let amount = self.operand(operand, body);
        match self.operand_type(operand, body).and_then(|ty| self.wide_int(ty)) {
            Some(suffix) => format!("(uint32_t){RT}{suffix}_to_u64({amount})"),
            None => format!("(uint32_t)({amount})"),
        }
    }

    /// A binary operation in a 128-bit type: one runtime helper. Floor
    /// division and modulo are C's on an unsigned type, where they agree.
    fn wide_binary(&self, op: ember_mir::BinOp, lhs: &Operand, rhs: &Operand, wide: Ty, body: &Body) -> String {
        use ember_mir::BinOp;
        let suffix = self.wide_int(wide).expect("a 128-bit type");
        let signed = suffix == "i128";
        let a = self.wide_operand(lhs, body, wide);
        let name = match op {
            BinOp::Shl | BinOp::Shr => {
                let stem = if op == BinOp::Shl { "shl" } else { "shr" };
                return format!("{RT}{suffix}_{stem}({a}, {})", self.shift_amount(rhs, body));
            }
            BinOp::Add => "add",
            BinOp::Sub => "sub",
            BinOp::Mul => "mul",
            BinOp::Div => "div",
            BinOp::Rem => "rem",
            BinOp::FloorDiv => if signed { "floordiv" } else { "div" },
            BinOp::FloorRem => if signed { "floorrem" } else { "rem" },
            BinOp::BitAnd => "and",
            BinOp::BitOr => "or",
            BinOp::BitXor => "xor",
            BinOp::Eq | BinOp::Is => "eq",
            BinOp::Ne | BinOp::IsNot => "ne",
            BinOp::Lt => "lt",
            BinOp::Le => "le",
            BinOp::Gt => "gt",
            BinOp::Ge => "ge",
            BinOp::And | BinOp::Or => unreachable!("`and` and `or` take `bool`s"),
        };
        format!("{RT}{suffix}_{name}({a}, {})", self.wide_operand(rhs, body, wide))
    }

    /// D-316 — whether `ty` is `f16`, which C carries as its bits in a
    /// `uint16_t` and which every operation widens to `double`.
    fn half(&self, ty: Ty) -> bool {
        matches!(self.types.kind(ty), TyKind::Float(FloatTy::F16))
    }

    fn half_operands(&self, lhs: &Operand, rhs: &Operand, body: &Body) -> bool {
        [lhs, rhs].into_iter().any(|operand| self.operand_type(operand, body).is_some_and(|ty| self.half(ty)))
    }

    /// An `f16` operand as a `double`, exactly: a constant as its value.
    fn half_operand(&self, operand: &Operand, body: &Body) -> String {
        match operand {
            Operand::Const(Const::Float { value, .. }) => {
                render_float(ember_types::f16_value(ember_types::f16_bits(*value)), false)
            }
            _ => format!("{RT}f16_to_f64({})", self.operand(operand, body)),
        }
    }

    /// D-316 — a numeric cast to or from `f16`, through `double`: exact
    /// from `f16`, and rounded once to it (an integer too large for a
    /// `double`'s precision is far past `f16`'s largest value, so its own
    /// rounding changes nothing). A float to an integer saturates
    /// (`[TYP-6]`).
    fn half_cast(&self, operand: &Operand, from: Ty, to: Ty, body: &Body) -> String {
        let value = self.operand(operand, body);
        if self.half(from) && self.half(to) {
            return value;
        }
        let double = if self.half(from) {
            format!("{RT}f16_to_f64({value})")
        } else if let Some(suffix) = self.wide_int(from) {
            format!("{RT}{suffix}_to_f64({value})")
        } else {
            format!("((double)({value}))")
        };
        if self.half(to) {
            return format!("{RT}f64_to_f16({double})");
        }
        if let Some(suffix) = self.wide_int(to) {
            return format!("{RT}ftoi_{suffix}({double})");
        }
        if self.types.is_integral(to) {
            return format!("{RT}ftoi_{}({double})", self.checked_suffix(to));
        }
        format!("(({}){double})", self.c_type(to))
    }

    /// A numeric cast to or from a 128-bit type (`[TYP-6]`): a float
    /// saturates toward zero, a narrower integer is extended, and a narrower
    /// target takes the low bits, as every integer cast does.
    fn wide_cast(&self, operand: &Operand, from: Ty, to: Ty, body: &Body) -> String {
        let value = self.operand(operand, body);
        match (self.wide_int(from), self.wide_int(to)) {
            (_, Some(suffix)) if self.types.is_float(from) => format!("{RT}ftoi_{suffix}({value})"),
            (_, Some(suffix)) => self.to_wide(&value, from, suffix),
            (Some(suffix), None) if self.types.is_float(to) => {
                let float = if matches!(self.types.kind(to), TyKind::Float(FloatTy::F32)) { "f32" } else { "f64" };
                format!("{RT}{suffix}_to_{float}({value})")
            }
            (Some(suffix), None) => format!("(({}){RT}{suffix}_to_u64({value}))", self.c_type(to)),
            (None, None) => unreachable!("wide_cast is called for a 128-bit type"),
        }
    }

    /// Build the representation-level result shared by Array and Span split
    /// operations. Bounds and provenance are MIR facts; C emission only lays
    /// out the two half-open views and avoids arithmetic on a null base for a
    /// zero boundary.
    fn split_views_expression(
        &self,
        source: &str,
        boundary: &str,
        elem: Ty,
        pair: Ty,
        mutable: bool,
    ) -> String {
        let pair = self.c_type(pair);
        let elem = self.c_type(elem);
        let view = if mutable { format!("{RT}mutspan") } else { format!("{RT}span") };
        // A zero-sized element's C type is `void`, which C cannot step over
        // (D-354): every element is at the base pointer.
        let qualifier = if mutable { "" } else { "const " };
        let tail = format!(
            "({qualifier}void*){}",
            element_pointer(&format!("{source}.ptr"), &elem, boundary, mutable)
        );
        format!(
            "({pair}){{ ({view}){{ {source}.ptr, {boundary} }}, \
             ({view}){{ ({boundary} == 0 ? {source}.ptr : {tail}), \
             {source}.len - {boundary} }} }}"
        )
    }

    /// A mutable view operation receives either the Span value produced by a
    /// preceding expression or a pointer created by an explicit reborrow of a
    /// named Span place. Normalize those two MIR representations once.
    fn span_value_expression(&self, operand: &str, arg_ty: Ty) -> String {
        if matches!(self.types.kind(arg_ty), TyKind::Ref { .. }) {
            format!("(*{operand})")
        } else {
            format!("({operand})")
        }
    }

    /// Whether a call consumes this explicit argument. Direct-call modes cross
    /// the backend boundary in `Body`; an indirect call carries a function
    /// value whose type retains the same mode vector under `[FN-6a]`.
    fn call_argument_is_owned(&self, func: &FuncRef, index: usize, body: &Body) -> bool {
        match func {
            FuncRef::Direct { symbol, .. } => self
                .direct_param_modes
                .get(symbol)
                .and_then(|modes| modes.get(index))
                .is_some_and(|mode| *mode == ParameterMode::Owned),
            FuncRef::Indirect { operand, .. } => {
                let (Operand::Copy(place) | Operand::Move(place)) = operand else {
                    return false;
                };
                let TyKind::Fn { params, .. } = self.types.kind(self.place_ty(place, body)) else {
                    return false;
                };
                params
                    .get(index)
                    .is_some_and(|parameter| parameter.mode == FnParamMode::Owned)
            }
            FuncRef::Interface { param_modes, .. } => index
                .checked_sub(1)
                .and_then(|parameter| param_modes.get(parameter))
                .is_some_and(|mode| *mode == ParameterMode::Owned),
            FuncRef::Virtual { param_modes, .. } => param_modes
                .get(index)
                .is_some_and(|mode| *mode == ParameterMode::Owned),
            _ => false,
        }
    }

    fn call_expression(&self, func: &FuncRef, args: &[Operand], body: &Body) -> String {
        let rendered: Vec<String> = args.iter().map(|a| self.operand(a, body)).collect();
        match func {
            FuncRef::Direct { symbol, .. } => format!("{symbol}({})", rendered.join(", ")),
            FuncRef::Virtual { owner, slot, .. } => {
                let signature = self
                    .virtual_signatures
                    .get(&(*owner, *slot))
                    .expect("virtual call has a collected vtable signature");
                let receiver = rendered.first().expect("virtual call has a receiver");
                let receiver_ty = signature.params.first().copied();
                let object = match receiver_ty.map(|ty| self.types.kind(ty)) {
                    Some(TyKind::Ref { .. }) => format!("*({receiver})"),
                    _ => receiver.clone(),
                };
                let table = ember_branding::vtable(&self.types.class_def(*owner).name.to_string());
                format!(
                    "(((const struct {table}*)(((const {RT}obj_header*)({object}))->ti->vtable))->slot{slot})({})",
                    rendered.join(", ")
                )
            }
            FuncRef::Interface { interfaces, interface, slot, class_handle, .. } => {
                let receiver = rendered.first().expect("interface call has a receiver");
                let table_identity = dyn_table_identity(interfaces);
                let table = ember_branding::vtable(&format!("dyn_{table_identity}"));
                let call_args = rendered.iter().skip(1).cloned().collect::<Vec<_>>().join(", ");
                if *class_handle {
                    // A mutable one-word class-interface receiver reaches this
                    // boundary as `ref mut I` so ordinary mutable-place and
                    // containing-object access checks stay visible in MIR.
                    // The table ABI remains one object pointer, however, so
                    // peel that compiler-internal reference before lookup or
                    // adapter dispatch.
                    let receiver = match args.first() {
                        Some(Operand::Copy(place) | Operand::Move(place))
                            if matches!(
                                self.types.kind(self.place_ty(place, body)),
                                TyKind::Ref { inner, .. }
                                    if matches!(self.types.kind(*inner), TyKind::ClassInterface(_))
                            ) => format!("*({receiver})"),
                        _ => receiver.clone(),
                    };
                    let call_args = if call_args.is_empty() {
                        receiver.clone()
                    } else {
                        format!("{receiver}, {call_args}")
                    };
                    let table_pointer = self.interface_cache_for_call(func, args, body).unwrap_or_else(|| {
                        format!(
                            "{}(((const {RT}obj_header*)({receiver}))->ti, &{})",
                            ember_branding::runtime("itable_lookup"),
                            interface_id_symbol(&interface.to_string()),
                        )
                    });
                    return format!(
                        "(((const struct {table}*){table_pointer})->slot{slot})({call_args})",
                    );
                }
                let call_args = if call_args.is_empty() {
                    format!("{receiver}.data")
                } else {
                    format!("{receiver}.data, {call_args}")
                };
                format!(
                    "(((const struct {table}*)({receiver}.vtable))->slot{slot})({call_args})"
                )
            }
            FuncRef::DynBoxNew { concrete, boxed, interfaces, .. } => {
                let concrete_c = self.c_type(*concrete);
                let table_identity = dyn_table_identity(interfaces);
                let table = self.interface_adapter_table(*concrete, &table_identity);
                let data = if matches!(self.types.kind(*concrete), TyKind::Class(_)) {
                    rendered[0].clone()
                } else {
                    let source = if self.types.is_primitive_scalar(*concrete) {
                        format!("&(({concrete_c}){{{}}})", rendered[0])
                    } else {
                        format!("&{}", rendered[0])
                    };
                    format!(
                        "{RT}box_new_copy({concrete_c_size}, {concrete_c_align}, {source})", concrete_c_size = c_size(&concrete_c), concrete_c_align = c_align(&concrete_c)
                    )
                };
                format!(
                    "({}){{ .data = {data}, .vtable = &{table} }}",
                    self.c_type(*boxed),
                )
            }
            // `[CLO-3]` — a call through a value. In C a function value is
            // its address, so the callee expression is called directly.
            FuncRef::Indirect { operand: callee, .. } => {
                format!("({})({})", self.operand(callee, body), rendered.join(", "))
            }
            FuncRef::Builtin { which, arg_ty } => {
                // `Array` and `String` share one runtime buffer; the element
                // size is passed at each call, which is how a single
                // implementation serves every element type.
                match which {
                    // `[CELL-1]`, `[CELL-2]` — the cell operations are gone by
                    // now. Each is a move out of a field, a store into it, and
                    // sometimes a drop, and lowering writes exactly those, so
                    // the backend emits a struct field access and nothing else:
                    // "no overhead relative to a plain field" is not a target
                    // here, it is the only code there is. Reaching this arm
                    // would mean a lowering arm was lost, which is worth saying
                    // loudly rather than emitting a call to a function that
                    // does not exist.
                    Builtin::CellSet
                    | Builtin::CellReplace
                    | Builtin::CellIntoInner
                    | Builtin::CellUpdate
                    | Builtin::CellUpdateDefault { .. }
                    | Builtin::CellTake { .. }
                    | Builtin::MemReplace { .. }
                    | Builtin::MemTake { .. }
                    | Builtin::MemSwap { .. }
                    | Builtin::MemForget { .. }
                    | Builtin::MemDrop { .. }
                    | Builtin::AlignOf
                    | Builtin::UnsafeCellIntoInner
                    | Builtin::RefCellBorrow
                    | Builtin::RefCellBorrowMut
                    | Builtin::RefCellTryBorrow
                    | Builtin::RefCellTryBorrowMut
                    | Builtin::MaybeUninitWrite { .. }
                    | Builtin::MaybeUninitAssumeInit { .. }
                    | Builtin::MaybeUninitWriteAt { .. }
                    | Builtin::MaybeUninitSpanAssumeInit { .. }
                    | Builtin::ArenaArrayGet { .. }
                    | Builtin::ArenaArrayPush { .. }
                    | Builtin::ArenaArrayInsert { .. }
                    | Builtin::ArenaArrayRemove { .. }
                    | Builtin::ArenaArrayClear
                    | Builtin::ArenaArrayIterNext { .. }
                    | Builtin::ArenaMapGet { .. }
                    | Builtin::ArenaMapContains { .. }
                    | Builtin::ArenaMapInsert { .. }
                    | Builtin::ArenaMapRemove { .. }
                    | Builtin::ArenaMapClear
                    | Builtin::ArenaMapIterNext { .. }
                    | Builtin::SpanChunksNew { .. }
                    | Builtin::SpanWindowsNew { .. }
                    | Builtin::SpanWindowsNext { .. }
                    | Builtin::ClassSuperInit { .. } => {
                        unreachable!(
                            "`{}` is lowered to field accesses in MIR and never reaches the backend",
                            which.name()
                        );
                    }
                    // `[ARN-8]` — C needs a determinate carrier value even
                    // though Ember does not consider the payload initialized.
                    // A zeroed private wrapper avoids reading indeterminate C
                    // bytes; it does not establish `T: Zeroable` and no safe
                    // Ember operation can observe the payload as `T`.
                    Builtin::MaybeUninitUninit { .. } => {
                        let wrapper = self.c_type(*arg_ty);
                        return format!("({wrapper}){{0}}");
                    }
                    // `[UNS-10]` — the argument is an explicit shared borrow
                    // of the one-field wrapper. The cast removes C's `const`
                    // from that field address at the unsafe boundary; Ember
                    // still exposes only `*mut T`, never a safe `ref mut T`.
                    Builtin::UnsafeCellGet { inner } => {
                        let inner = self.c_type(*inner);
                        return format!("(({inner}*)&(({})->value))", rendered[0]);
                    }
                    Builtin::ArenaWithCapacity => {
                        let arena = self.c_type(*arg_ty);
                        return format!("({arena}){{ {RT}arena_new({}), 0 }}", rendered[0]);
                    }
                    Builtin::ArenaArrayWithCapacity { elem, array } => {
                        let array = self.c_type(*array);
                        let elem = self.c_type(*elem);
                        return format!(
                            "({array}){{ {}, ({elem}*){RT}arena_alloc_uninit(({})->state, {}, {elem_size}, {elem_align}), 0, {} }}",
                            rendered[0], rendered[0], rendered[1], rendered[1], elem_size = c_size(&elem), elem_align = c_align(&elem)
                        );
                    }
                    Builtin::ArenaMapWithCapacity { map, .. } => {
                        let map_ty = *map;
                        let map = self.c_type(map_ty);
                        let TyKind::Struct(map_id) = *self.types.kind(map_ty) else {
                            unreachable!("ArenaMap is represented by a struct")
                        };
                        let pointer = self.types.struct_def(map_id).fields[1].ty;
                        let TyKind::Ptr { inner: slot, .. } = *self.types.kind(pointer) else {
                            unreachable!("ArenaMap backing storage is a slot pointer")
                        };
                        let slot = self.c_type(slot);
                        return format!(
                            "({map}){{ {}, ({slot}*){RT}arena_alloc_zeroed(({})->state, {}, {slot_size}, {slot_align}), 0, {} }}",
                            rendered[0], rendered[0], rendered[1], rendered[1], slot_size = c_size(&slot), slot_align = c_align(&slot)
                        );
                    }
                    Builtin::ArenaAlloc { elem } => {
                        let elem_c = self.c_type(*elem);
                        return format!(
                            "({elem_c}*){RT}arena_alloc_copy(({})->state, {elem_c_size}, {elem_c_align}, &{})",
                            rendered[0], rendered[1], elem_c_size = c_size(&elem_c), elem_c_align = c_align(&elem_c)
                        );
                    }
                    Builtin::ArenaAllocUninit { elem } => {
                        let span = self.c_type(*arg_ty);
                        let slot = self.c_type(self.span_element(*arg_ty));
                        let elem_c = self.c_type(*elem);
                        return format!(
                            "({span}){{ ({slot}*){RT}arena_alloc_uninit(({})->state, {}, {elem_c_size}, {elem_c_align}), {} }}",
                            rendered[0], rendered[1], rendered[1], elem_c_size = c_size(&elem_c), elem_c_align = c_align(&elem_c)
                        );
                    }
                    Builtin::ArenaAllocArrayZeroed { elem } => {
                        let span = self.c_type(*arg_ty);
                        let elem_c = self.c_type(*elem);
                        return format!(
                            "({span}){{ ({elem_c}*){RT}arena_alloc_zeroed(({})->state, {}, {elem_c_size}, {elem_c_align}), {} }}",
                            rendered[0], rendered[1], rendered[1], elem_c_size = c_size(&elem_c), elem_c_align = c_align(&elem_c)
                        );
                    }
                    // `[ARN-3]`'s `Default` arm is constructed explicitly in
                    // MIR. This call reserves only raw storage; the following
                    // loop invokes the source constructor and initializes each
                    // element before the span can be observed.
                    Builtin::ArenaAllocArrayDefault { elem, .. } => {
                        let span = self.c_type(*arg_ty);
                        let elem_c = self.c_type(*elem);
                        return format!(
                            "({span}){{ ({elem_c}*){RT}arena_alloc_uninit(({})->state, {}, {elem_c_size}, {elem_c_align}), {} }}",
                            rendered[0], rendered[1], rendered[1], elem_c_size = c_size(&elem_c), elem_c_align = c_align(&elem_c)
                        );
                    }
                    Builtin::ArenaReset => {
                        return format!("{RT}arena_reset(({})->state)", rendered[0]);
                    }
                    Builtin::FixedArenaAlloc { elem } => {
                        let fixed = self.c_type(self.element_of(*arg_ty));
                        let elem_c = self.c_type(*elem);
                        return format!(
                            "({elem_c}*){RT}fixed_arena_alloc_copy((void*)(({fixed}*){})->buffer.ptr, (({fixed}*){})->buffer.len, &(({fixed}*){})->used, {elem_c_size}, {elem_c_align}, &{})",
                            rendered[0], rendered[0], rendered[0], rendered[1], elem_c_size = c_size(&elem_c), elem_c_align = c_align(&elem_c)
                        );
                    }
                    Builtin::FixedArenaReset => {
                        let fixed = self.c_type(self.element_of(*arg_ty));
                        return format!("((({fixed}*){})->used = 0)", rendered[0]);
                    }
                    Builtin::ArenaScope { scoped } => {
                        let parent = self.c_type(self.element_of(*arg_ty));
                        let scoped = self.c_type(*scoped);
                        return format!(
                            "({scoped}){{ &(({parent}*){})->token, (({parent}*){})->state, {RT}arena_mark((({parent}*){})->state), 0 }}",
                            rendered[0], rendered[0], rendered[0]
                        );
                    }
                    Builtin::ScopedArenaAlloc { elem } => {
                        let scoped = self.c_type(self.element_of(*arg_ty));
                        let elem_c = self.c_type(*elem);
                        return format!(
                            "({elem_c}*){RT}arena_alloc_copy((({scoped}*){})->state, {elem_c_size}, {elem_c_align}, &{})",
                            rendered[0], rendered[1], elem_c_size = c_size(&elem_c), elem_c_align = c_align(&elem_c)
                        );
                    }
                    // `[CLS-1]` — an empty class constructor allocates the
                    // complete object header through the runtime. The cast is
                    // the nominal class-handle boundary; the runtime returns
                    // the common header pointer after validating the emitted
                    // type information.
                    Builtin::ClassNew { class_id, .. } => {
                        let class_ty = self.c_type(*arg_ty);
                        let type_info = ember_branding::type_info(
                            &self.types.class_def(*class_id).name.to_string(),
                        );
                        return format!("(({class_ty}){RT}obj_new(&{type_info}))");
                    }
                    // `[DSP-4]` — the runtime walks the object's type-info
                    // base chain. The returned pointer is deliberately only
                    // the query result; MIR performs the owning retain when
                    // it constructs `Some` or the forced result.
                    Builtin::ClassDowncast { target, .. } => {
                        let class_ty = self.c_type(*arg_ty);
                        let type_info = ember_branding::type_info(
                            &self.types.class_def(*target).name.to_string(),
                        );
                        return format!(
                            "(({class_ty}){RT}downcast(({RT}obj_header*){}, &{type_info}))",
                            rendered[0]
                        );
                    }
                    Builtin::ArrayNew | Builtin::StringNew => {
                        return format!("{RT}vec_empty()");
                    }
                    // `[TYP-37]` — floats by totalOrder, everything else by `<`.
                    Builtin::TotalLess => {
                        let (a, b) = (&rendered[0], &rendered[1]);
                        if let Some(suffix) = self.wide_int(*arg_ty) {
                            return format!("{RT}{suffix}_lt({a}, {b})");
                        }
                        return match self.types.kind(*arg_ty) {
                            TyKind::Float(FloatTy::F64) => format!("{RT}total_lt_f64({a}, {b})"),
                            TyKind::Float(FloatTy::F16) => format!("{RT}total_lt_f16({a}, {b})"),
                            TyKind::Float(_) => format!("{RT}total_lt_f32({a}, {b})"),
                            TyKind::Str => format!("({RT}str_cmp({a}, {b}) < 0)"),
                            _ => format!("(({a}) < ({b}))"),
                        };
                    }
                    // `[STD-26]` — the count and the values are computed in
                    // 64 bits, signed or unsigned as the type is; each value
                    // lies between `start` and `stop`, so it converts back.
                    Builtin::RangeCount | Builtin::RangeNth => {
                        if let Some(suffix) = self.wide_int(*arg_ty) {
                            let stem = if matches!(which, Builtin::RangeCount) { "count" } else { "nth" };
                            return format!("{RT}range_{stem}_{suffix}({})", rendered.join(", "));
                        }
                        let unsigned = matches!(self.types.kind(*arg_ty), TyKind::Uint(_));
                        let (wide, suffix) = if unsigned { ("uint64_t", "u64") } else { ("int64_t", "i64") };
                        let widen = |v: &String| format!("({wide})({v})");
                        if matches!(which, Builtin::RangeCount) {
                            return format!(
                                "{RT}range_count_{suffix}({}, {}, {})",
                                widen(&rendered[0]),
                                widen(&rendered[1]),
                                widen(&rendered[2])
                            );
                        }
                        let ty = self.c_type(*arg_ty);
                        return format!(
                            "(({ty}){RT}range_nth_{suffix}({}, {}, {}))",
                            widen(&rendered[0]),
                            widen(&rendered[1]),
                            rendered[2]
                        );
                    }
                    Builtin::StrStartsWith => {
                        return format!("{RT}str_starts_with({}, {})", rendered[0], rendered[1]);
                    }
                    Builtin::StrEndsWith => {
                        return format!("{RT}str_ends_with({}, {})", rendered[0], rendered[1]);
                    }
                    Builtin::StrFind { reverse } => {
                        let function = if *reverse { "rfind" } else { "find" };
                        return format!("{RT}str_{function}({}, {})", rendered[0], rendered[1]);
                    }
                    Builtin::StrCount => {
                        return format!("{RT}str_count({}, {})", rendered[0], rendered[1]);
                    }
                    Builtin::StrReplace => {
                        return format!("{RT}str_replace({}, {}, {})", rendered[0], rendered[1], rendered[2]);
                    }
                    Builtin::StrRepeat => {
                        return format!("{RT}str_repeat({}, {})", rendered[0], rendered[1]);
                    }
                    Builtin::StrTrimStart => {
                        return format!("{RT}str_trim_start({})", rendered[0]);
                    }
                    Builtin::ParseStatus { kind } => {
                        use ember_mir::ParseKind;
                        return match kind {
                            ParseKind::Signed => {
                                format!("{RT}parse_signed_status({}, (int64_t)({}))", rendered[0], rendered[1])
                            }
                            ParseKind::Unsigned => {
                                format!("{RT}parse_unsigned_status({}, (uint64_t)({}))", rendered[0], rendered[1])
                            }
                            ParseKind::I128 => format!("{RT}parse_i128_status({})", rendered[0]),
                            ParseKind::U128 => format!("{RT}parse_u128_status({})", rendered[0]),
                            ParseKind::F16 | ParseKind::F32 | ParseKind::F64 => {
                                format!("{RT}parse_float_status({})", rendered[0])
                            }
                            ParseKind::Bool => format!("{RT}parse_bool_status({})", rendered[0]),
                            ParseKind::Char => format!("{RT}parse_char_status({})", rendered[0]),
                        };
                    }
                    Builtin::ParseValue { kind } => {
                        use ember_mir::ParseKind;
                        return match kind {
                            ParseKind::Signed => format!("{RT}parse_signed_value({})", rendered[0]),
                            ParseKind::Unsigned => format!("{RT}parse_unsigned_value({})", rendered[0]),
                            ParseKind::I128 => format!("{RT}parse_i128_value({})", rendered[0]),
                            ParseKind::U128 => format!("{RT}parse_u128_value({})", rendered[0]),
                            ParseKind::F16 => format!("{RT}parse_f16_value({})", rendered[0]),
                            ParseKind::F32 => format!("{RT}parse_f32_value({})", rendered[0]),
                            ParseKind::F64 => format!("{RT}parse_f64_value({})", rendered[0]),
                            ParseKind::Bool => format!("(({}).len == 4)", rendered[0]),
                            ParseKind::Char => format!("{RT}str_char_at({}, 0)", rendered[0]),
                        };
                    }
                    Builtin::StrToUpper => {
                        return format!("{RT}str_to_upper({})", rendered[0]);
                    }
                    Builtin::StrToLower => {
                        return format!("{RT}str_to_lower({})", rendered[0]);
                    }
                    Builtin::StrSliceOk => {
                        return format!("{RT}str_slice_ok({}, {}, {}, {})", rendered[0], rendered[1], rendered[2], rendered[3]);
                    }
                    Builtin::StrTrimEnd => {
                        return format!("{RT}str_trim_end({})", rendered[0]);
                    }
                    Builtin::StrCharAt => {
                        return format!("{RT}str_char_at({}, {})", rendered[0], rendered[1]);
                    }
                    Builtin::CharUtf8Len => {
                        return format!("{RT}char_utf8_len({})", rendered[0]);
                    }
                    Builtin::StrCharCount => {
                        return format!("{RT}str_char_count({})", rendered[0]);
                    }
                    Builtin::StrContains => {
                        return format!("{RT}str_contains({}, {})", rendered[0], rendered[1]);
                    }
                    // `[STD-15]` — the receiver arrives as a pointer to the
                    // array, as `push`'s does.
                    Builtin::ArraySort => {
                        let elem = self.element_of(*arg_ty);
                        let less = self.array_helper(ArrayHelper::Less, elem);
                        return format!("{RT}vec_sort({}, {}, {less})", rendered[0], c_size(&self.c_type(elem)));
                    }
                    Builtin::ArrayReverse => {
                        let elem = self.element_of(*arg_ty);
                        return format!("{RT}vec_reverse({}, {})", rendered[0], c_size(&self.c_type(elem)));
                    }
                    Builtin::ArrayClear => {
                        let elem = self.element_of(*arg_ty);
                        return format!("{}({})", self.array_helper(ArrayHelper::Clear, elem), rendered[0]);
                    }
                    Builtin::ArrayPop { option } => {
                        return format!("{}({})", self.array_helper(ArrayHelper::Pop, *option), rendered[0]);
                    }
                    Builtin::ArrayRemove => {
                        let elem = self.element_of(*arg_ty);
                        return format!("{}({}, {})", self.array_helper(ArrayHelper::Remove, elem), rendered[0], rendered[1]);
                    }
                    Builtin::ArrayDrain => {
                        let elem = self.element_of(*arg_ty);
                        let helper = self.array_helper(ArrayHelper::Drain, elem);
                        return format!("{helper}({}, {}, {})", rendered[0], rendered[1], rendered[2]);
                    }
                    Builtin::ArraySwapRemove => {
                        let elem = self.element_of(*arg_ty);
                        return format!("{}({}, {})", self.array_helper(ArrayHelper::SwapRemove, elem), rendered[0], rendered[1]);
                    }
                    Builtin::ArrayTruncate => {
                        let elem = self.element_of(*arg_ty);
                        return format!("{}({}, {})", self.array_helper(ArrayHelper::Truncate, elem), rendered[0], rendered[1]);
                    }
                    Builtin::ArrayExtend => {
                        let elem = self.element_of(*arg_ty);
                        return format!(
                            "{}({}, ({}).ptr, ({}).len)",
                            self.array_helper(ArrayHelper::Extend, elem),
                            rendered[0],
                            rendered[1],
                            rendered[1]
                        );
                    }
                    Builtin::ArrayCapacity => {
                        return format!("({}).cap", rendered[0]);
                    }
                    Builtin::ArrayReserve => {
                        let elem = self.element_of(*arg_ty);
                        return format!(
                            "{RT}vec_reserve({}, {}, ({})->len + ({}))",
                            rendered[0],
                            c_size(&self.c_type(elem)),
                            rendered[0],
                            rendered[1]
                        );
                    }
                    Builtin::ArraySwap => {
                        let elem = self.element_of(*arg_ty);
                        return format!(
                            "{RT}vec_swap({}, {}, {}, {})",
                            rendered[0],
                            c_size(&self.c_type(elem)),
                            rendered[1],
                            rendered[2]
                        );
                    }
                    Builtin::ArrayInsert => {
                        let elem = self.element_of(*arg_ty);
                        return format!(
                            "{RT}vec_insert({}, {}, {}, {})",
                            rendered[0],
                            c_size(&self.c_type(elem)),
                            rendered[2],
                            self.value_address(&rendered[1], elem)
                        );
                    }
                    Builtin::ArraySorted { elem } => {
                        return format!(
                            "{}(({}).ptr, ({}).len)",
                            self.array_helper(ArrayHelper::Sorted, *elem),
                            rendered[0],
                            rendered[0]
                        );
                    }
                    Builtin::Input => {
                        unreachable!("[STD-10] `input` is emitted with its location by the call terminator")
                    }
                    Builtin::IntOverflowing(_) => {
                        unreachable!("[STD-20] MIR computes `IntOverflowing` in place")
                    }
                    Builtin::ArrayClone { elem } => {
                        return format!(
                            "{}(({}).ptr, ({}).len)",
                            self.array_helper(ArrayHelper::Clone, *elem),
                            rendered[0],
                            rendered[0]
                        );
                    }
                    Builtin::CloneParts { .. } => {
                        unreachable!("CloneParts writes directly into its MIR destination")
                    }
                    Builtin::StrContainsChar => {
                        return format!("{RT}str_contains_char({}, {})", rendered[0], rendered[1]);
                    }
                    Builtin::FloatAbs if self.half(*arg_ty) => {
                        return format!("((uint16_t)({} & 0x7FFFu))", rendered[0]);
                    }
                    Builtin::FloatPow if self.half(*arg_ty) => {
                        return format!(
                            "{RT}f64_to_f16(pow({RT}f16_to_f64({}), {RT}f16_to_f64({})))",
                            rendered[0], rendered[1]
                        );
                    }
                    Builtin::FloatAbs => {
                        let function = match self.types.kind(*arg_ty) {
                            TyKind::Float(FloatTy::F64) => "fabs",
                            _ => "fabsf",
                        };
                        return format!("{function}({})", rendered[0]);
                    }
                    Builtin::FloatPow => {
                        let function = match self.types.kind(*arg_ty) {
                            TyKind::Float(FloatTy::F64) => "pow",
                            _ => "powf",
                        };
                        return format!("{function}({}, {})", rendered[0], rendered[1]);
                    }
                    // `[STD-27]` — `sqrt` or `sqrtf`; a classification macro
                    // is one name, and its `int` is made a `bool`.
                    // `[STD-20]` — over the value's own width, zero-extended.
                    Builtin::IntBits(bits) => {
                        let x = &rendered[0];
                        let method = bits.method();
                        if let Some(suffix) = self.wide_int(*arg_ty) {
                            let value = if suffix == "i128" { format!("{RT}i128_to_u128({x})") } else { x.clone() };
                            return format!("{RT}u128_{method}({value})");
                        }
                        let width = ember_types::bit_width(self.types, *arg_ty).unwrap_or(64);
                        let value = format!("(uint64_t)(uint{width}_t)({x})");
                        return match bits {
                            ember_mir::IntBits::CountOnes => format!("{RT}count_ones({value})"),
                            _ => format!("{RT}{method}({value}, {width})"),
                        };
                    }
                    Builtin::FloatLib(f) => {
                        let call = |suffix: &str| format!("{}{suffix}({})", f.c_name(), rendered.join(", "));
                        return match (f.is_predicate(), self.types.kind(*arg_ty)) {
                            (true, _) => format!("({} != 0)", call("")),
                            (false, TyKind::Float(FloatTy::F64)) => call(""),
                            (false, _) => call("f"),
                        };
                    }
                    // D-187 — the checker sent here only text (all six
                    // comparisons, by bytes) and `==`/`!=` on aggregates.
                    Builtin::ValueCompare { op } => {
                        let (a, b) = (&rendered[0], &rendered[1]);
                        if let Some((a, b)) = self.text_pair(a, b, *arg_ty) {
                            return format!("({RT}str_cmp({a}, {b}) {} 0)", op.spelling());
                        }
                        let equal = self.eq_expr(a, b, *arg_ty);
                        return if matches!(op, ember_mir::BinOp::Ne) { format!("(!{equal})") } else { equal };
                    }
                    // `[TYP-38]` — one allocation of exactly the literal's
                    // elements, moved (bitwise) out of the fixed array; MIR
                    // has marked the fixed array as moved, so it is not
                    // dropped a second time.
                    Builtin::ArrayFromLiteral => {
                        let TyKind::Array { elem, len } = *self.types.kind(*arg_ty) else {
                            unreachable!("[TYP-38] an Array literal is built from a fixed array")
                        };
                        let elem_c = self.c_type(elem);
                        return format!(
                            "{RT}vec_from_elems({elem_c_size}, ({})._0, {len})",
                            rendered[0], elem_c_size = c_size(&elem_c)
                        );
                    }
                    Builtin::BoxNew { elem, boxed } => {
                        let elem_c = self.c_type(*elem);
                        let boxed_c = self.c_type(*boxed);
                        return format!(
                            "(({boxed_c}){RT}box_new_copy({elem_c_size}, {elem_c_align}, {}))",
                            self.value_address(&rendered[0], *elem), elem_c_size = c_size(&elem_c), elem_c_align = c_align(&elem_c)
                        );
                    }
                    Builtin::SharedNew { elem, shared } => {
                        let elem_c = self.c_type(*elem);
                        let shared_c = self.c_type(*shared);
                        let TyKind::Struct(id) = self.types.kind(*shared) else {
                            unreachable!("SharedNew carries a compiler-known wrapper")
                        };
                        let info = self.shared_type_info_symbol(*id);
                        return format!(
                            "(({shared_c}){RT}obj_new_copy(&{info}, {elem_c_align}, {elem_c_size}, {}))",
                            self.value_address(&rendered[0], *elem), elem_c_align = c_align(&elem_c), elem_c_size = c_size(&elem_c)
                        );
                    }
                    Builtin::WeakNew { weak, .. } => {
                        let weak_c = self.c_type(*weak);
                        return format!(
                            "({}(({}*){}), ({weak_c}){{ {} }})",
                            ember_branding::runtime("weak_retain"),
                            ember_branding::runtime("obj_header"),
                            rendered[0],
                            rendered[0]
                        );
                    }
                    Builtin::WeakUpgrade { class, .. } => {
                        let class_c = self.c_type(*class);
                        return format!(
                            "(({class_c}){}(({}*)({}.value)))",
                            ember_branding::runtime("weak_upgrade"),
                            ember_branding::runtime("obj_header"),
                            rendered[0]
                        );
                    }
                    Builtin::WeakEmpty { weak } => {
                        let weak_c = self.c_type(*weak);
                        return format!("({weak_c}){{0}}");
                    }
                    Builtin::ArrayPush => {
                        let elem = self.element_of(*arg_ty);
                        return format!(
                            "{RT}vec_push({}, {}, {})",
                            rendered[0],
                            c_size(&self.c_type(elem)),
                            self.value_address(&rendered[1], elem)
                        );
                    }
                    Builtin::StringPush => {
                        return format!(
                            "{RT}vec_extend({}, {}.ptr, {}.len)",
                            rendered[0], rendered[1], rendered[1]
                        );
                    }
                    Builtin::ArrayLen | Builtin::StringLen => {
                        return format!("({}).len", rendered[0]);
                    }
                    // `[SPN-3]` — same checked split for an existing Span or
                    // MutSpan. MIR has already checked `boundary <= len` and
                    // made a named mutable receiver an explicit reborrow.
                    Builtin::SpanSplitAt { elem, pair, mutable } => {
                        let source = self.span_value_expression(&rendered[0], *arg_ty);
                        return self.split_views_expression(
                            &source,
                            &rendered[1],
                            *elem,
                            *pair,
                            *mutable,
                        );
                    }
                    Builtin::SpanReborrow => {
                        let source = self.span_value_expression(&rendered[0], *arg_ty);
                        return format!(
                            "(({RT}mutspan){{ {source}.ptr, {source}.len }})"
                        );
                    }
                    Builtin::SpanSharedReborrow => {
                        let source = self.span_value_expression(&rendered[0], *arg_ty);
                        return format!("(({RT}span){{ {source}.ptr, {source}.len }})");
                    }
                    // `[SPN-5]` — MIR has checked the cursor and advanced it
                    // before this representation-level extraction. The
                    // iterator's monotonic cursor is the disjointness proof
                    // for mutable items.
                    Builtin::SpanIterNext { elem, mutable } => {
                        let elem = self.c_type(*elem);
                        return element_pointer(&format!("({}).ptr", rendered[0]), &elem, &rendered[1], *mutable);
                    }
                    // `[SPN-6]` — MIR computed the bounded, non-zero advance;
                    // C emission only forms the half-open subview and avoids
                    // arithmetic on a null pointer when the start is zero.
                    Builtin::SpanChunksNext { elem, mutable } => {
                        let elem = self.c_type(*elem);
                        let source = &rendered[0];
                        let view = if *mutable {
                            format!("{RT}mutspan")
                        } else {
                            format!("{RT}span")
                        };
                        let qualifier = if *mutable { "" } else { "const " };
                        let tail = format!(
                            "({qualifier}void*){}",
                            element_pointer(&format!("({source}).ptr"), &elem, &rendered[1], *mutable)
                        );
                        return format!(
                            "(({view}){{ ({} == 0 ? ({}).ptr : {tail}), {} }})",
                            rendered[1], source, rendered[2]
                        );
                    }
                    // `[SPN-8]`, `[SPN-9]` — extraction is safe and does not
                    // create a reference. The result is a raw pointer whose
                    // later use remains governed by `unsafe`.
                    Builtin::SpanAsPtr { mutable } => {
                        let source = self.span_value_expression(&rendered[0], *arg_ty);
                        let elem = self.c_type(self.span_element(*arg_ty));
                        return if *mutable {
                            format!("(({elem}*){source}.ptr)")
                        } else {
                            format!("((const {elem}*){source}.ptr)")
                        };
                    }
                    Builtin::StringAsStr => {
                        // The argument is a borrow of the string, so it
                        // arrives as a pointer (as with `SpanFrom` above).
                        return format!("{RT}vec_as_str({})", rendered[0]);
                    }
                    Builtin::Format | Builtin::FormatWith(_) if self.is_display_aggregate(*arg_ty) => {
                        // `[TYP-39]` — an aggregate takes a spec only after
                        // `!r`/`!s` has made it text, and then the spec pads
                        // that text, as in Python.
                        let spec = match which {
                            Builtin::FormatWith(spec) => Some(*spec),
                            _ => None,
                        };
                        let debug = spec.is_some_and(|spec| spec.kind == Some('?'));
                        if !debug && let Some(name) = self.unit_display(&rendered[1], *arg_ty) {
                            return match spec.filter(|spec| hir_spec_is_more_than_a_conversion(spec)) {
                                None => format!("{RT}fmt_str({}, {name})", rendered[0]),
                                Some(spec) => format!(
                                    "{RT}fmt_spec_str({}, {name}, {})",
                                    rendered[0],
                                    fmt_spec_literal(&ember_mir::FormatSpec { kind: None, ..spec })
                                ),
                            };
                        }
                        return match spec.filter(|spec| hir_spec_is_more_than_a_conversion(spec)) {
                            None => format!("{}({}, &({}))", self.fmt_fn(*arg_ty), rendered[0], rendered[1]),
                            Some(spec) => format!(
                                "{}({}, &({}), {})",
                                self.fmt_spec_fn(*arg_ty),
                                rendered[0],
                                rendered[1],
                                fmt_spec_literal(&spec)
                            ),
                        };
                    }
                    Builtin::Format | Builtin::FormatWith(_) if matches!(self.types.kind(*arg_ty), TyKind::Void) => {
                        let text = format!("{RT}str_lit(\"()\", 2)");
                        // `!r` and `!s` both give its `Debug` text; a spec pads it.
                        let spec = match which {
                            Builtin::FormatWith(spec) => Some(*spec),
                            _ => None,
                        };
                        return match spec.filter(|spec| hir_spec_is_more_than_a_conversion(spec)) {
                            None => format!("{RT}fmt_str({}, {text})", rendered[0]),
                            Some(spec) => format!(
                                "{RT}fmt_spec_str({}, {text}, {})",
                                rendered[0],
                                fmt_spec_literal(&ember_mir::FormatSpec { kind: None, ..spec })
                            ),
                        };
                    }
                    Builtin::Format => {
                        return format!(
                            "{RT}fmt_{}({}, {})",
                            self.format_suffix(*arg_ty),
                            rendered[0],
                            rendered[1]
                        );
                    }
                    // `[LEX-19]` — the parsed spec travels as a struct literal.
                    Builtin::FormatWith(spec) => {
                        let spec_c = fmt_spec_literal(spec);
                        return format!(
                            "{RT}fmt_spec_{}({}, {}, {spec_c})",
                            self.format_suffix(*arg_ty),
                            rendered[0],
                            rendered[1]
                        );
                    }
                    // `[UNS-5]` — the raw memory primitives. `arg_ty` is the
                    // first argument's type, which for `alloc` and `size_of`
                    // is the element type itself.
                    Builtin::MemAlloc => {
                        let elem = self.element_of(*arg_ty);
                        let elem = self.c_type(elem);
                        return format!(
                            "({elem}*){RT}alloc({} * {elem_size}, {elem_align})",
                            rendered[0], elem_size = c_size(&elem), elem_align = c_align(&elem)
                        );
                    }
                    Builtin::MemFree => {
                        let elem = self.element_of(*arg_ty);
                        let elem = self.c_type(elem);
                        return format!(
                            "{RT}free({}, {} * {elem_size}, {elem_align})",
                            rendered[0], rendered[1], elem_size = c_size(&elem), elem_align = c_align(&elem)
                        );
                    }
                    Builtin::PtrRead => {
                        return format!("({})[{}]", rendered[0], rendered[1]);
                    }
                    Builtin::PtrWrite => {
                        return format!(
                            "({})[{}] = {}",
                            rendered[0], rendered[1], rendered[2]
                        );
                    }
                    Builtin::SizeOf => {
                        return c_size(&self.c_type(*arg_ty));
                    }
                    // `[RNG-3a]` — total: no failure mode, no `Panic`, no
                    // `RuntimeCheck(k)`. "On the C backend it lowers to two
                    // compares or the target's `min`/`max` instruction pair,
                    // strictly cheaper than `[RNG-3]`'s
                    // check-and-branch-to-panic."
                    Builtin::RangeClamped(id) => {
                        return self.range_clamp(*id, &rendered[0]);
                    }
                    // `[RNG-10]` — the `unsafe` route. `[RNG-9]` makes an
                    // out-of-range value undefined behaviour, so nothing is
                    // emitted: the caller's obligation is the whole check.
                    // The `debug_assert` `[RNG-10]` requires waits for
                    // `debug_assert` itself (Phase 1's assertion set).
                    Builtin::RangeNewUnchecked(_) => {
                        return format!("({})", rendered[0]);
                    }
                    // `[SPN-2]` — the length field of the view.
                    Builtin::SpanLen => {
                        return format!("({}).len", rendered[0]);
                    }
                    // Part VI's slice row — the bounds were checked in MIR.
                    // An empty view may have no buffer, so an offset of 0
                    // keeps the pointer as it is.
                    Builtin::Slice { text } => {
                        let (view, lo, hi) = (&rendered[0], &rendered[1], &rendered[2]);
                        return if *text {
                            format!("(({RT}str){{ ({lo} == 0 ? ({view}).ptr : ({view}).ptr + {lo}), {hi} - {lo} }})")
                        } else {
                            let elem = self.c_type(self.span_element(*arg_ty));
                            let tail = element_pointer(&format!("({view}).ptr"), &elem, lo, false);
                            format!("(({RT}span){{ ({lo} == 0 ? ({view}).ptr : (const void*){tail}), {hi} - {lo} }})")
                        };
                    }
                    Builtin::StrAsBytes => {
                        return format!("(({RT}span){{ ({}).ptr, ({}).len }})", rendered[0], rendered[0]);
                    }
                    Builtin::StrIsCharBoundary => {
                        return format!("{RT}str_is_char_boundary({}, {})", rendered[0], rendered[1]);
                    }
                    // `[SPN-2]` — `unsafe s.get_unchecked(i)`. No check, by
                    // construction: `[UNS-4]` makes the bound the caller's
                    // obligation.
                    Builtin::SpanGetUnchecked => {
                        let elem = self.span_element(*arg_ty);
                        return format!("&{}", self.buffer_element(&format!("({})", rendered[0]), elem, &rendered[1]));
                    }
                    // `[SPN-1]` — an `Array[T]` or a `[T; N]` viewed. A
                    // buffer keeps its pointer and length; a fixed array's
                    // length is in its type, and its storage is the wrapper
                    // struct's `_0` (Part IV.3 makes `[T; N]` a value, and a
                    // bare C array is not one).
                    Builtin::SpanFrom { mutable } => {
                        let view = if *mutable {
                            format!("{RT}mutspan")
                        } else {
                            format!("{RT}span")
                        };
                        // The argument is a borrow of the container, so it
                        // arrives as a pointer.
                        let container = self.span_source(*arg_ty);
                        let it = format!("(*{})", rendered[0]);
                        return match self.types.kind(container) {
                            TyKind::Array { len, .. } => {
                                format!("(({view}){{ {it}._0, {len} }})")
                            }
                            _ => format!("(({view}){{ {it}.ptr, {it}.len }})"),
                        };
                    }
                    // `[SPN-2]`'s `get` returns an `Option`, which is a branch
                    // and two aggregates rather than a C expression, so MIR
                    // lowers it and it never reaches here.
                    Builtin::SpanGet => {
                        unreachable!("[SPN-2] `get` is lowered in MIR")
                    }
                    // `[RNG-3]` — the fallible form. Lowered in MIR into a
                    // branch and two enum aggregates, so it never reaches
                    // here.
                    Builtin::RangeChecked(_) => {
                        unreachable!("[RNG-3] `checked` is lowered in MIR")
                    }
                    Builtin::Println | Builtin::Print | Builtin::EPrintln | Builtin::EPrint => {}
                    Builtin::Panic | Builtin::Assert => {
                        unreachable!("VI.6 — panics and assertions are MIR assertions")
                    }
                }
                if let Some(name) = self.unit_display(&rendered[0], *arg_ty) {
                    let printer = match which {
                        Builtin::Println => "println",
                        Builtin::EPrintln => "eprintln",
                        Builtin::EPrint => "eprint",
                        _ => "print",
                    };
                    return format!("{RT}{printer}_str({name})");
                }
                if self.is_display_aggregate(*arg_ty) {
                    let to_stderr = u8::from(matches!(which, Builtin::EPrint | Builtin::EPrintln));
                    return format!("{}(&({}), {to_stderr})", self.fmt_print_fn(*arg_ty), rendered[0]);
                }
                let suffix = self.builtin_suffix(*arg_ty);
                let name = match which {
                    Builtin::Println => "println",
                    Builtin::EPrintln => "eprintln",
                    Builtin::EPrint => "eprint",
                    _ => "print",
                };
                format!("{RT}{name}_{suffix}({})", rendered.join(", "))
            }
        }
    }

    /// Return the hidden table local for a repeated class-interface call.
    /// An unprojected class-interface parameter qualifies directly; `mut I`
    /// qualifies through its compiler-generated reborrow. Other receivers keep
    /// the direct lookup so a projection cannot leave a stale table behind.
    fn interface_cache_for_call(&self, func: &FuncRef, args: &[Operand], body: &Body) -> Option<String> {
        let FuncRef::Interface { interface, class_handle: true, .. } = func else {
            return None;
        };
        let Operand::Copy(place) = args.first()? else {
            return None;
        };
        let local = interface_cache_parameter_root(body, place)?;
        self.interface_caches
            .get(&InterfaceCacheKey {
                local,
                interface: interface.to_string(),
            })
            .cloned()
    }

    /// Which `ember_rt` printer a builtin call resolves to. Phase 0 has no
    /// `Display` interface, so the choice is made here from the argument type.
    /// Which `ember_fmt_*` an f-string piece appends through. A `String`
    /// piece is formatted as the `str` it borrows.
    fn format_suffix(&self, ty: Ty) -> &'static str {
        match self.types.kind(ty) {
            TyKind::Vec { text: true, .. } => "str",
            _ => self.builtin_suffix(ty),
        }
    }

    fn builtin_suffix(&self, ty: ember_types::Ty) -> &'static str {
        if let Some(suffix) = self.wide_int(ty) {
            return suffix;
        }
        match self.types.kind(ty) {
            TyKind::Bool => "bool",
            TyKind::Char => "char",
            TyKind::Int(_) => "i64",
            TyKind::Uint(_) => "u64",
            TyKind::Float(FloatTy::F64) => "f64",
            TyKind::Float(FloatTy::F16) => "f16",
            TyKind::Float(_) => "f32",
            TyKind::Str => "str",
            _ => "str",
        }
    }

    // -- places, operands, rvalues -------------------------------------------

    fn place_in(&self, place: &Place, body: &Body) -> String {
        let mut out = format!("_{}", place.local.0);
        let mut at = Cursor { ty: body.local(place.local).ty, variant: None };
        for projection in &place.projection {
            match projection {
                Projection::Field(index) => match (at.variant, self.types.kind(at.ty)) {
                    // After a downcast, a field is that variant's payload; a
                    // niche `Option`'s `Some` holds it as the whole value.
                    (Some(_), TyKind::Enum(id)) if self.types.option_niche(*id).is_some() => {}
                    (Some(variant), TyKind::Enum(id)) => {
                        let def = self.types.enum_def(*id);
                        let variant = &def.variants[variant];
                        out.push_str(&format!(
                            ".payload.{}.{}",
                            variant.name, variant.fields[*index].name
                        ));
                    }
                    (_, TyKind::Struct(id)) => {
                        let def = self.types.struct_def(*id);
                        // The checker models Box auto-deref as private field 0
                        // followed by `Deref`, while Part XIX.6 erases Box to
                        // `T*`. The logical field therefore contributes no C
                        // member access; the following projection emits
                        // `(*box)` directly.
                        if let Some(inner) = self.shared_inner_id(*id) {
                            debug_assert_eq!(*index, 0);
                            let inner_c = self.c_type(inner);
                            out = format!(
                                "(({}*){}(({}*){out}, {}))",
                                inner_c,
                                ember_branding::runtime("obj_payload"),
                                ember_branding::runtime("obj_header"),
                                c_align(&inner_c),
                            );
                        } else if self.box_inner_id(*id).is_none() {
                            out.push_str(&format!(".{}", def.fields[*index].name));
                        }
                    }
                    (_, TyKind::Class(id)) => {
                        if let Some(field) = self.types.class_field_at(*id, *index) {
                            out.push_str(&format!("->{}", field.name));
                        }
                    }
                    // An `Array[T]` is the runtime's buffer: pointer, length,
                    // capacity, in that order. A view is the same shape with
                    // no capacity (Part VII §7).
                    (_, TyKind::Vec { .. } | TyKind::Span { .. }) => {
                        out.push_str(match index {
                            0 => ".ptr",
                            1 => ".len",
                            _ => ".cap",
                        });
                    }
                    // A tuple's elements are the generated struct's `_0`,
                    // `_1`, … in order.
                    _ => out.push_str(&format!("._{index}")),
                },
                // A fixed array is a generated struct wrapping one C array, so
                // the subscript goes through that member. An `Array[T]` keeps
                // its elements behind a `void*`, so the subscript casts first.
                // A buffer and a view are both `{ptr, len, …}`, so both
                // index through `ptr`; a fixed array is the wrapper struct's
                // `_0` member (Part IV.3 makes `[T; N]` a value, and a bare C
                // array is not one).
                Projection::Index(local) => match self.types.kind(at.ty) {
                    TyKind::Vec { elem, .. } | TyKind::Span { elem, .. } => {
                        out = self.buffer_element(&out, *elem, &format!("_{}", local.0));
                    }
                    TyKind::Ptr { .. } => out = format!("({out})[_{}]", local.0),
                    _ => out.push_str(&format!("._0[_{}]", local.0)),
                },
                Projection::ConstIndex(i) => match self.types.kind(at.ty) {
                    TyKind::Vec { elem, .. } | TyKind::Span { elem, .. } => {
                        out = self.buffer_element(&out, *elem, &i.to_string());
                    }
                    TyKind::Ptr { .. } => out = format!("({out})[{i}]"),
                    _ => out.push_str(&format!("._0[{i}]")),
                },
                Projection::Deref => {
                    // `[CELL-1]`/`[CELL-2]` give `Cell` a deliberately narrow
                    // interior-mutation boundary: its shared methods may
                    // write the private payload even when the receiver came
                    // through `ref Cell[T]`. `Cell` lowering becomes ordinary
                    // MIR field assignments, so remove C's `const` exactly
                    // while crossing that compiler-known wrapper. No source
                    // program can expose the field, and every other shared
                    // reference keeps its ordinary `const` representation.
                    if matches!(
                        self.types.kind(at.ty),
                        TyKind::Ref { mutable: false, inner }
                            if matches!(self.types.kind(*inner), TyKind::Struct(id)
                                if self.cell_inner_id(*id).is_some())
                    ) {
                        let TyKind::Ref { inner, .. } = self.types.kind(at.ty) else {
                            unreachable!("matched a reference above")
                        };
                        out = format!("(*(({}*){out}))", self.c_type(*inner));
                    } else {
                        out = format!("(*{out})");
                    }
                }
                // A downcast writes nothing on its own; the `Field` after it
                // names the variant and the member together.
                Projection::Downcast(_) => {}
                Projection::Column(i) => out.push_str(&format!(".col{i}")),
            }
            at = self.project(at, projection);
        }
        out
    }

    fn place_ty(&self, place: &Place, body: &Body) -> Ty {
        let mut at = Cursor { ty: body.local(place.local).ty, variant: None };
        for projection in &place.projection {
            at = self.project(at, projection);
        }
        at.ty
    }

    /// Where one projection arrives. A projection that does not apply to the
    /// type leaves it alone; the type checker has already rejected that
    /// program, and the backend only has to stay on its feet.
    fn project(&self, at: Cursor, projection: &Projection) -> Cursor {
        let plain = |ty| Cursor { ty, variant: None };
        match (projection, self.types.kind(at.ty)) {
            (Projection::Downcast(variant), TyKind::Enum(_)) => {
                Cursor { ty: at.ty, variant: Some(*variant) }
            }
            (Projection::Field(index), TyKind::Enum(id)) => {
                let Some(variant) = at.variant else { return plain(at.ty) };
                let fields = &self.types.enum_def(*id).variants[variant].fields;
                plain(fields.get(*index).map(|f| f.ty).unwrap_or(at.ty))
            }
            (Projection::Field(index), TyKind::Struct(id)) => {
                let fields = &self.types.struct_def(*id).fields;
                plain(fields.get(*index).map(|f| f.ty).unwrap_or(at.ty))
            }
            (Projection::Field(index), TyKind::Class(id)) => plain(
                self.types
                    .class_field_at(*id, *index)
                    .map(|field| field.ty)
                    .unwrap_or(at.ty),
            ),
            (Projection::Field(index), TyKind::Tuple(items)) => {
                plain(items.get(*index).copied().unwrap_or(at.ty))
            }
            (
                Projection::Index(_) | Projection::ConstIndex(_),
                TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } | TyKind::Span { elem, .. },
            ) => plain(*elem),
            (
                Projection::Index(_) | Projection::ConstIndex(_),
                TyKind::Ptr { inner, .. },
            ) => plain(*inner),
            // `.len` and `.cap` on the runtime buffer are `usize`; `.ptr` is
            // never projected through, so it keeps the buffer's own type. A
            // view is the same shape with no `.cap`.
            (Projection::Field(index), TyKind::Vec { .. } | TyKind::Span { .. }) => {
                if *index == 0 { at } else { plain(self.usize_ty) }
            }
            (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => {
                plain(*inner)
            }
            _ => at,
        }
    }

    fn operand(&self, operand: &Operand, body: &Body) -> String {
        match operand {
            // A move and a copy generate the same C; the difference is a fact
            // the borrow checker uses, not a code shape.
            // A `void` local is never declared; its value is the byte `0`
            // a `void` member holds (D-263).
            Operand::Copy(p) | Operand::Move(p) if self.is_void(self.place_ty(p, body)) => "0".to_string(),
            Operand::Copy(p) | Operand::Move(p) => self.place_in(p, body),
            Operand::Const(c) => self.constant(c),
        }
    }

    fn constant(&self, constant: &Const) -> String {
        match constant {
            Const::Int { value, ty } => {
                if let Some(wide) = self.wide_int(*ty) {
                    return self.wide_constant(*value, wide);
                }
                if self.types.is_integral(*ty) {
                    return self.int_literal(*value, *ty);
                }
                let suffix = match self.types.kind(*ty) {
                    TyKind::Int(IntTy::I64 | IntTy::Isize) => "LL",
                    TyKind::Uint(UintTy::U64 | UintTy::Usize) => "ULL",
                    TyKind::Uint(_) => "U",
                    _ => "",
                };
                format!("{value}{suffix}")
            }
            // D-316 — an `f16` is its bits, rounded once from the value.
            Const::Float { value, ty } if self.half(*ty) => {
                format!("((uint16_t)0x{:04X}u)", ember_types::f16_bits(*value))
            }
            Const::Float { value, ty } => {
                let is_f32 = matches!(self.types.kind(*ty), TyKind::Float(FloatTy::F32));
                render_float(*value, is_f32)
            }
            Const::Bool(v) => if *v { "true" } else { "false" }.to_string(),
            Const::Str(text) => {
                format!("{RT}str_lit({}, {})", c_string_literal(text), text.len())
            }
            // `[CELL-5]` — a conflicting-borrow file path, stored into the
            // cell's `borrow_file` (`*u8`) field. The cast keeps
            // `-Wall -Wextra` quiet about the `char*` to `uint8_t*`
            // signedness difference; the pointer round-trips through the
            // field for the panic to name.
            Const::CStr(text) => {
                format!("((const uint8_t*){})", c_string_literal(text))
            }
            // `[FN-6]` — a function value is its C symbol, which is its
            // address.
            Const::Fn(symbol) => symbol.clone(),
            Const::Void => "0".to_string(),
        }
    }

    /// `target` is the type of the place being assigned. A tuple or array
    /// aggregate needs it to name the generated struct it is building.
    fn rvalue(&self, rvalue: &Rvalue, body: &Body, target: Ty) -> String {
        match rvalue {
            Rvalue::Use(o) => self.operand(o, body),
            // D-316 — an `f16` operation happens in `double` and rounds back
            // once; a comparison compares the widened values. `/` goes
            // through `fdiv`, since a zero divisor may be a constant (D-253).
            Rvalue::BinaryOp { op, lhs, rhs } if self.half_operands(lhs, rhs, body) => {
                use ember_mir::BinOp as B;
                let (a, b) = (self.half_operand(lhs, body), self.half_operand(rhs, body));
                match op {
                    B::Eq | B::Ne | B::Lt | B::Le | B::Gt | B::Ge => format!("({a} {} {b})", op.c_operator()),
                    B::Add | B::Sub | B::Mul => format!("{RT}f64_to_f16(({a} {} {b}))", op.c_operator()),
                    B::Div => format!("{RT}f64_to_f16({RT}fdiv_f64({a}, {b}))"),
                    B::FloorDiv => format!("{RT}f64_to_f16({RT}floordiv_f64({a}, {b}))"),
                    B::FloorRem => format!("{RT}f64_to_f16({RT}floorrem_f64({a}, {b}))"),
                    other => unreachable!("`{}` is not an `f16` operator", other.c_operator()),
                }
            }
            // D-272 — an operation on a 128-bit value is a runtime helper. A
            // shift's type is its shifted value's; only a 128-bit amount of
            // a narrower value needs converting (D-308).
            Rvalue::BinaryOp { op, lhs, rhs } if self.wide_operands(lhs, rhs, body).is_some() => {
                let shift = matches!(op, ember_mir::BinOp::Shl | ember_mir::BinOp::Shr);
                let shifted = self.operand_type(lhs, body).filter(|ty| self.wide_int(*ty).is_some());
                match (shift, shifted) {
                    (true, None) => {
                        format!("({} {} {})", self.operand(lhs, body), op.c_operator(), self.shift_amount(rhs, body))
                    }
                    (true, Some(wide)) => self.wide_binary(*op, lhs, rhs, wide, body),
                    (false, _) => {
                        let wide = self.wide_operands(lhs, rhs, body).expect("checked by the guard");
                        self.wide_binary(*op, lhs, rhs, wide, body)
                    }
                }
            }
            // `[TYP-29]`, ODR-021 — float floor division and modulo are the
            // runtime's Python-exact helpers; C has no operator for either.
            Rvalue::BinaryOp {
                op: op @ (ember_mir::BinOp::FloorDiv | ember_mir::BinOp::FloorRem),
                lhs,
                rhs,
            }
                if self.types.is_float(target) =>
            {
                let stem =
                    if *op == ember_mir::BinOp::FloorDiv { "floordiv" } else { "floorrem" };
                let suffix = match self.types.kind(target) {
                    TyKind::Float(FloatTy::F32) => "f32",
                    _ => "f64",
                };
                format!(
                    "{RT}{stem}_{suffix}({}, {})",
                    self.operand(lhs, body),
                    self.operand(rhs, body)
                )
            }
            // `[TYP-29]` gives a float division by zero IEEE's result, but
            // MSVC refuses a division by a constant zero (C2124, D-253), so a
            // literal zero divisor reaches the runtime's `fdiv` as a parameter.
            Rvalue::BinaryOp {
                op: ember_mir::BinOp::Div,
                lhs,
                rhs: rhs @ Operand::Const(ember_mir::Const::Float { value, .. }),
            } if *value == 0.0 && self.types.is_float(target) => {
                let suffix = match self.types.kind(target) {
                    TyKind::Float(FloatTy::F32) => "f32",
                    _ => "f64",
                };
                format!(
                    "{RT}fdiv_{suffix}({}, {})",
                    self.operand(lhs, body),
                    self.operand(rhs, body)
                )
            }
            Rvalue::BinaryOp { op, lhs, rhs } => {
                format!(
                    "({} {} {})",
                    self.operand(lhs, body),
                    op.c_operator(),
                    self.operand(rhs, body)
                )
            }
            Rvalue::UnaryOp { op: ember_mir::UnOp::Neg, operand }
                if self.operand_type(operand, body).is_some_and(|ty| self.half(ty)) =>
            {
                format!("((uint16_t)({} ^ 0x8000u))", self.operand(operand, body))
            }
            Rvalue::UnaryOp { op, operand } => {
                let wide = self.operand_type(operand, body).and_then(|ty| self.wide_int(ty));
                // A negated integer constant is one constant, the least
                // value included (D-311).
                if let (ember_mir::UnOp::Neg, Operand::Const(Const::Int { value, ty }), None) = (op, operand, wide)
                    && matches!(self.types.kind(*ty), TyKind::Int(_))
                {
                    return self.int_literal(value.wrapping_neg(), *ty);
                }
                let operand = self.operand(operand, body);
                if let Some(suffix) = wide {
                    return match op {
                        ember_mir::UnOp::Neg => format!("{RT}{suffix}_neg({operand})"),
                        ember_mir::UnOp::BitNot => format!("{RT}{suffix}_not({operand})"),
                        ember_mir::UnOp::Not => unreachable!("`not` takes a `bool`"),
                    };
                }
                match op {
                    ember_mir::UnOp::Neg => format!("(-{operand})"),
                    ember_mir::UnOp::Not => format!("(!{operand})"),
                    ember_mir::UnOp::BitNot => format!("(~{operand})"),
                }
            }
            Rvalue::Cast { kind, operand, to } => {
                if matches!(kind, CastKind::Widen | CastKind::Numeric)
                    && let Some(from) = self.operand_type(operand, body)
                    && (self.half(from) || self.half(*to))
                {
                    return self.half_cast(operand, from, *to, body);
                }
                if matches!(kind, CastKind::Widen | CastKind::Numeric)
                    && let Some(from) = self.operand_type(operand, body)
                    && (self.wide_int(from).is_some() || self.wide_int(*to).is_some())
                {
                    return self.wide_cast(operand, from, *to, body);
                }
                let value = self.operand(operand, body);
                let ty = self.c_type(*to);
                match kind {
                    CastKind::InterfaceUpcast { concrete, interfaces, .. } => {
                        let table_identity = dyn_table_identity(interfaces);
                        let table = self.interface_adapter_table(*concrete, &table_identity);
                        let data = if matches!(self.types.kind(*concrete), TyKind::Class(_)) {
                            format!("*({value})")
                        } else {
                            value
                        };
                        format!("({ty}){{ .data = (void*)({data}), .vtable = &{table} }}")
                    }
                    CastKind::ClassInterfaceUpcast { .. } => format!("(({ty}){value})"),
                    // `[TYP-6]` (0.9.9) — float to integer saturates and maps
                    // NaN to 0; C's own conversion is undefined outside the
                    // target's range.
                    CastKind::Numeric
                        if self.types.is_integral(*to)
                            && self.operand_type(operand, body).is_some_and(|from| {
                                matches!(self.types.kind(from), TyKind::Float(_))
                            }) =>
                    {
                        format!("{RT}ftoi_{}({value})", self.checked_suffix(*to))
                    }
                    // A widening is lossless by construction (`[TYP-5]`), so
                    // the C cast is exact.
                    CastKind::Widen
                    | CastKind::Numeric
                    | CastKind::ClassUpcast
                    | CastKind::ClassUpcastBorrowed => {
                        format!("(({ty}){value})")
                    }
                }
            }
            Rvalue::Aggregate { kind, operands } => {
                let values: Vec<String> = operands.iter().map(|o| self.operand(o, body)).collect();
                if let AggregateKind::Enum(id, variant) = kind {
                    return self.enum_value(*id, *variant, &values);
                }
                let name = match kind {
                    AggregateKind::Struct(id) => {
                        c_name(&self.types.struct_def(*id).name.to_string())
                    }
                    _ => self.c_type(target),
                };
                // An empty aggregate still has the padding member `[STR-4]`
                // gives it, and C wants an initialiser for it.
                if values.is_empty() {
                    return format!("({name}){{ 0 }}");
                }
                match kind {
                    // The array's elements sit inside the wrapper's one
                    // member, so they need their own brace level.
                    AggregateKind::Array => format!("({name}){{ {{ {} }} }}", values.join(", ")),
                    _ => format!("({name}){{ {} }}", values.join(", ")),
                }
            }
            // `[ENM-3]` — a unit-only enum is its tag, so reading the
            // discriminant is reading the value.
            Rvalue::Discriminant(place) => {
                let read = self.place_in(place, body);
                let ty = self.place_ty(place, body);
                match self.types.kind(ty) {
                    TyKind::Enum(id) if !self.types.enum_def(*id).is_unit_only() => self.enum_tag(*id, &read),
                    _ => read,
                }
            }
            Rvalue::Ref { place, .. } => {
                let address = format!("&{}", self.place_in(place, body));
                // A shared Ember borrow of a class handle is a read-only
                // handle borrow, not a C `const` object pointer.  The natural
                // C spelling has one extra pointer level (`&handle`), where
                // adding `const` at the outer type is not assignable from the
                // mutable local slot. The explicit cast preserves Ember's
                // checked shared-borrow boundary without leaking a C warning
                // into `[CG-C-1]` output.
                if matches!(
                    self.types.kind(target),
                    TyKind::Ref { mutable: false, inner }
                        if matches!(self.types.kind(*inner), TyKind::Class(_))
                ) {
                    format!("(({}){address})", self.c_type(target))
                } else {
                    address
                }
            }
            // `[value; count]` is emitted as a loop by `emit_stmt`; it is
            // built only ever as the right-hand side of an assignment.
            Rvalue::Repeat { .. } => {
                unreachable!("Rvalue::Repeat is emitted by emit_stmt, not as an expression")
            }
        }
    }

    /// One enum value. A unit-only enum is just its discriminant; a payload
    /// enum sets the tag and the one union member that variant uses.
    /// Designated initialisers name both, so nothing is left uninitialised
    /// and `-Wmissing-field-initializers` has nothing to say.
    fn enum_value(&self, id: EnumId, variant: usize, values: &[String]) -> String {
        if let Some(niche) = self.types.option_niche(id) {
            if variant == niche.some {
                return values[0].clone();
            }
            return self.niche_none(&niche);
        }
        let def = self.types.enum_def(id);
        let name = c_name(&def.name.to_string());
        let tag = def.variants[variant].discriminant;
        if def.is_unit_only() {
            return format!("(({name}){tag})");
        }
        if values.is_empty() {
            return format!("({name}){{ .tag = {tag} }}");
        }
        format!(
            "({name}){{ .tag = {tag}, .payload = {{ .{} = {{ {} }} }} }}",
            def.variants[variant].name,
            values.join(", ")
        )
    }

    /// The discriminant of the payload enum at `access`. `[TYP-13]` — an
    /// `Option` with a niche has no tag: it is `None` when its payload
    /// holds the niche.
    fn enum_tag(&self, id: EnumId, access: &str) -> String {
        let Some(niche) = self.types.option_niche(id) else {
            return format!("({access}).tag");
        };
        let variants = &self.types.enum_def(id).variants;
        format!(
            "({} ? {} : {})",
            self.niche_holds(&niche, access),
            variants[niche.none].discriminant,
            variants[niche.some].discriminant
        )
    }

    /// One field of one variant of the payload enum at `access`. A niche
    /// `Option`'s `Some` holds its one field as the whole value.
    fn enum_member(&self, id: EnumId, access: &str, variant: usize, field: usize) -> String {
        if self.types.option_niche(id).is_some() {
            return format!("({access})");
        }
        let variant = &self.types.enum_def(id).variants[variant];
        format!("({access}).payload.{}.{}", variant.name, variant.fields[field].name)
    }

    /// True when the niche `Option` at `access` is `None`: `NonZero`'s one
    /// field is 0, which no `NonZero` holds (`[STD-4]`).
    fn niche_holds(&self, niche: &Niche, access: &str) -> String {
        let TyKind::Struct(id) = *self.types.kind(niche.payload) else {
            unreachable!("a niche is a NonZero's")
        };
        let field = &self.types.struct_def(id).fields[0];
        let zero = self.constant(&Const::Int { value: 0, ty: field.ty });
        self.eq_expr(&format!("({access}).{}", field.name), &zero, field.ty)
    }

    /// A niche `Option`'s `None`: its payload holding the niche.
    fn niche_none(&self, niche: &Niche) -> String {
        let TyKind::Struct(id) = *self.types.kind(niche.payload) else {
            unreachable!("a niche is a NonZero's")
        };
        let field = &self.types.struct_def(id).fields[0];
        let zero = self.constant(&Const::Int { value: 0, ty: field.ty });
        format!("(({}){{ .{} = {zero} }})", self.c_type(niche.payload), field.name)
    }

    // -- types ----------------------------------------------------------------

    /// The address of a rendered value of type `ty`, for a runtime call that
    /// copies `sizeof` bytes from it. A `void` value is rendered `0`, which
    /// has no address; it copies nothing, so any byte does (D-355).
    fn value_address(&self, rendered: &str, ty: Ty) -> String {
        if self.is_void(ty) { "&(uint8_t){0}".to_string() } else { format!("&{rendered}") }
    }

    fn is_void(&self, ty: ember_types::Ty) -> bool {
        matches!(self.types.kind(ty), TyKind::Void | TyKind::Never | TyKind::Error)
    }

    /// A member's or a parameter's C type. Ember's `void` is a unit value, and
    /// C cannot declare a member or parameter of type `void`: one byte carries
    /// it, as an enum payload's does (D-263, D-355).
    fn c_member_type(&self, ty: ember_types::Ty) -> String {
        if self.is_void(ty) { "uint8_t".to_string() } else { self.c_type(ty) }
    }

    /// Part XVIII §6's type mapping table.
    /// Element `index` of a buffer or view `{ptr, len, …}` as a C place.
    /// A zero-sized element's C type is `void`, which C cannot index (GCC
    /// and Clang allow it as an extension, MSVC does not), so every such
    /// element is the byte at the base pointer (D-354): distinct elements of
    /// a zero-sized type need not have distinct addresses (ODR-068).
    fn buffer_element(&self, buffer: &str, elem: Ty, index: &str) -> String {
        let c = self.c_type(elem);
        if c == "void" {
            return format!("(*(char*){buffer}.ptr)");
        }
        format!("(({c}*){buffer}.ptr)[{index}]")
    }

    fn c_type(&self, ty: ember_types::Ty) -> String {
        match self.types.kind(ty) {
            TyKind::Bool => "bool".into(),
            TyKind::Char => "uint32_t".into(),
            TyKind::Int(i) => match i {
                IntTy::I8 => "int8_t".to_string(),
                IntTy::I16 => "int16_t".to_string(),
                IntTy::I32 => "int32_t".to_string(),
                IntTy::I64 => "int64_t".to_string(),
                // `[CG-C-*]` — MSVC has no `__int128`, so the runtime carries
                // a struct with helper ops under its own prefix.
                IntTy::I128 => format!("{RT}i128"),
                IntTy::Isize => "ptrdiff_t".to_string(),
            },
            TyKind::Uint(u) => match u {
                UintTy::U8 => "uint8_t".to_string(),
                UintTy::U16 => "uint16_t".to_string(),
                UintTy::U32 => "uint32_t".to_string(),
                UintTy::U64 => "uint64_t".to_string(),
                UintTy::U128 => format!("{RT}u128"),
                UintTy::Usize => "size_t".to_string(),
            },
            TyKind::Float(f) => match f {
                // `[TYP-9]` — f16 arithmetic is performed in f32 and rounded
                // on the C backend; the storage form is a bit pattern.
                FloatTy::F16 => "uint16_t",
                FloatTy::F32 => "float",
                FloatTy::F64 => "double",
            }
            .into(),
            TyKind::Void | TyKind::Never | TyKind::Error => "void".into(),
            TyKind::Str => format!("{RT}str"),
            // A view is a pointer and a length. One C struct serves
            // every element type, as `ember_vec` does: the element
            // type is recovered at each use, and `[TYP-11]`'s C
            // layout guarantee is about `struct`s the programmer
            // declares, not about this.
            TyKind::Span { mutable, .. } => {
                if *mutable { format!("{RT}mutspan") } else { format!("{RT}span") }
            }
            TyKind::Struct(id) => c_name(&self.types.struct_def(*id).name.to_string()),
            // Class handles are pointers to the compiler-generated object
            // struct. The name comes from the shared branding helper so a
            // future prefix change cannot create a mismatched type spelling.
            TyKind::Class(id) => format!(
                "struct {}*",
                ember_branding::object_struct(&self.types.class_def(*id).name.to_string())
            ),
            // `[OBJ-2]` — the erased class interface remains exactly one
            // object-header pointer. Its dynamic vtable is found through
            // `type_info`, never carried in this value's representation.
            TyKind::ClassInterface(_) => format!("{RT}obj_header*"),
            TyKind::Enum(id) => c_name(&self.types.enum_def(*id).name.to_string()),
            // `[COST-3]` — a range type is "not observable": erased to the
            // representation, with the construction site carrying the check.
            // It emits no C type of its own, so `Roughness` and `f32` are the
            // same bits and `[RNG-8]`'s FFI representation falls out.
            TyKind::Range(id) => self.c_type(self.types.range_def(*id).repr),
            // `[TYP-22]` — an interface reference is already a fat pointer;
            // do not add a second C indirection around its `{data*, vtable*}`
            // representation. Dispatch/vtable materialisation is a later
            // backend slice, but the storage shape is fixed here.
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }
                if matches!(self.types.kind(*inner), TyKind::Dyn { .. }) =>
            {
                format!("{RT}dyn")
            }
            // A shared `ref T` is not `const` in C: a `Cell` or `RefCell`
            // inside it is written through a shared borrow by design (IX.7),
            // and the checker, not C, enforces what a shared borrow may do.
            TyKind::Ref { inner, .. } => format!("{}*", self.c_type(*inner)),
            TyKind::Ptr { mutable, inner } => {
                let inner = self.c_type(*inner);
                if *mutable { format!("{inner}*") } else { format!("const {inner}*") }
            }
            TyKind::Array { .. } | TyKind::Tuple(_) => self.structural_name(ty),
            // Every `Array[T]` and `String` is the same buffer; the element
            // size travels with each runtime call instead of with the type.
            TyKind::Vec { .. } => format!("{RT}vec"),
            TyKind::Fn { .. } => self.structural_name(ty),
            // The semantic type is unsized, but references to it use the
            // runtime's fixed two-word fat-pointer carrier.
            TyKind::Dyn { .. } => format!("{RT}dyn"),
            // A generic parameter never reaches the backend: monomorphisation
            // substitutes it away, and a body still holding one was never
            // instantiated.
            TyKind::Param { .. } | TyKind::Assoc { .. } => "void*".into(),
            TyKind::Infer(_) | TyKind::IntLit | TyKind::FloatLit => "int32_t".into(),
        }
    }

    /// `[FN-8]` — the C entry point initialises the runtime, calls Ember's
    /// `main`, and shuts down. `[RT-2]`: there are no global constructors, so
    /// this is the only place initialisation can happen.
    fn emit_entry_point(&mut self, leak_check: bool) {
        self.line("int main(int argc, char** argv) {");
        self.line(&format!("    {RT}rt_config cfg = {RT}rt_config_default();"));
        if leak_check {
            self.line("    cfg.flags |= EMBER_RT_DEBUG_OBJECTS;");
        }
        self.line("    (void)argc; (void)argv;");
        self.line(&format!("    {RT}rt_init(&cfg);"));
        if leak_check {
            self.emit_debug_edge_registrations();
        }
        self.line(&format!("    {}();", ember_branding::mangled("main")));
        self.line(&format!("    {RT}rt_shutdown();"));
        self.line("    return 0;");
        self.line("}");
    }

    fn emit_debug_edge_registrations(&mut self) {
        for (_, def) in self.types.runtime_classes() {
            let info = ember_branding::type_info(&def.name.to_string());
            let edges = class_debug_edges_symbol(&def.name.to_string());
            self.line(&format!("    {RT}debug_register_type_edges(&{info}, &{edges});"));
        }
        for (id, _) in self.types.structs() {
            let Some(_) = self.shared_inner_id(id) else {
                continue;
            };
            let info = self.shared_type_info_symbol(id);
            let edges = self.shared_debug_edges_symbol(id);
            self.line(&format!("    {RT}debug_register_type_edges(&{info}, &{edges});"));
        }
    }
}

/// Work out what type definitions the module needs and in what order.
///
/// `[CG-C-2]` — the walk is over the interner and the struct table in their
/// own order, so the result is the same for the same input.
fn plan_types(types: &TypeTable) -> (Vec<TypeNode>, BTreeMap<Ty, String>) {
    let mut planner = Planner {
        types,
        order: Vec::new(),
        names: BTreeMap::new(),
        done: BTreeSet::new(),
        active: BTreeSet::new(),
        taken: types
            .structs()
            .map(|(_, d)| c_name(&d.name.to_string()))
            .chain(
                types
                    .classes()
                    .map(|(_, d)| ember_branding::object_struct(&d.name.to_string())),
            )
            .chain(types.enums().map(|(_, d)| c_name(&d.name.to_string())))
            .collect(),
    };
    // Named types first, so the output stays close to declaration order; each
    // pulls in whatever it contains.
    let structs: Vec<StructId> = types.structs().map(|(id, _)| id).collect();
    for id in structs {
        planner.visit(TypeNode::Struct(id));
    }
    let classes: Vec<ClassId> = types.runtime_classes().map(|(id, _)| id).collect();
    for id in classes {
        planner.visit(TypeNode::Class(id));
    }
    let enums: Vec<EnumId> = types.enums().map(|(id, _)| id).collect();
    for id in enums {
        planner.visit(TypeNode::Enum(id));
    }
    let structural: Vec<Ty> = types
        .all()
        .filter(|(_, k)| {
            matches!(k, TyKind::Tuple(_) | TyKind::Array { .. } | TyKind::Fn { .. })
        })
        .map(|(ty, _)| ty)
        .collect();
    for ty in structural {
        planner.visit(TypeNode::Structural(ty));
    }
    (planner.order, planner.names)
}

struct Planner<'a> {
    types: &'a TypeTable,
    order: Vec<TypeNode>,
    names: BTreeMap<Ty, String>,
    done: BTreeSet<TypeNode>,
    active: BTreeSet<TypeNode>,
    taken: BTreeSet<String>,
}

impl Planner<'_> {
    fn visit(&mut self, node: TypeNode) {
        // `!insert` means the node is already on the stack, which is a type
        // that contains itself by value. `E2200` reports that; stopping here
        // keeps the backend from recursing forever on a program that is
        // already rejected.
        if self.done.contains(&node) || !self.active.insert(node) {
            return;
        }
        match node {
            TypeNode::Struct(id) => {
                // `Weak[O]` always lowers to one pointer-valued field. Its
                // owner can therefore remain a forward declaration; making
                // the planner require the owner first reverses the C order
                // for `class Node { parent: Weak[Node] }` and leaves the
                // wrapper incomplete at the class-field declaration.
                let is_weak = matches!(
                    &self.types.struct_def(id).origin,
                    Some((name, args)) if name.is("Weak") && args.len() == 1
                );
                if !is_weak {
                    let fields: Vec<Ty> =
                        self.types.struct_def(id).fields.iter().map(|f| f.ty).collect();
                    for field in fields {
                        self.require(field);
                    }
                }
            }
            TypeNode::Class(id) => {
                if let Some(base) = self.types.class_def(id).base {
                    self.visit(TypeNode::Class(base));
                }
                let fields: Vec<Ty> = (0..self.types.class_field_count(id))
                    .filter_map(|index| self.types.class_field_at(id, index).map(|field| field.ty))
                    .collect();
                for field in fields {
                    self.require(field);
                }
            }
            TypeNode::Enum(id) => {
                let payloads: Vec<Ty> = self
                    .types
                    .enum_def(id)
                    .variants
                    .iter()
                    .flat_map(|v| v.fields.iter().map(|f| f.ty))
                    .collect();
                for payload in payloads {
                    self.require(payload);
                }
            }
            TypeNode::Structural(ty) => {
                let contains: Vec<Ty> = match self.types.kind(ty) {
                    TyKind::Tuple(items) => items.clone(),
                    TyKind::Array { elem, .. } => vec![*elem],
                    // A function pointer's parameter and return types are
                    // written out in its typedef, so they must be complete.
                    TyKind::Fn { params, ret, .. } => {
                        let mut all = params.iter().map(|param| param.ty).collect::<Vec<_>>();
                        all.push(*ret);
                        all
                    }
                    _ => Vec::new(),
                };
                for inner in contains {
                    self.require(inner);
                }
                let name = self.fresh_name(ty);
                self.names.insert(ty, name);
            }
        }
        self.active.remove(&node);
        self.done.insert(node);
        self.order.push(node);
    }

    /// A type held by value needs its definition first. Behind a reference or
    /// a pointer it does not, because every definition is forward-declared.
    fn require(&mut self, ty: Ty) {
        match self.types.kind(ty) {
            TyKind::Struct(id) => self.visit(TypeNode::Struct(*id)),
            TyKind::Class(id) => self.visit(TypeNode::Class(*id)),
            TyKind::Enum(id) => self.visit(TypeNode::Enum(*id)),
            TyKind::Tuple(_) | TyKind::Array { .. } | TyKind::Fn { .. } => {
                self.visit(TypeNode::Structural(ty))
            }
            // A function-pointer type is a `typedef`, which C cannot declare
            // ahead the way it forward-declares a struct, so a pointer to one
            // still needs it first: a closure capturing a callable local by
            // shared borrow holds `ref fn(A) -> R` (D-185).
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }
                if matches!(self.types.kind(*inner), TyKind::Fn { .. }) =>
            {
                self.visit(TypeNode::Structural(*inner))
            }
            _ => {}
        }
    }

    /// A readable name derived from how the type prints, made unique against
    /// every name already used. The suffix only ever appears when a user type
    /// happens to be named like a generated one.
    fn fresh_name(&mut self, ty: Ty) -> String {
        let stem = identifier_from(&self.types.symbol_name(ty));
        // `[MNG-5]` — the generated-type prefixes are the mangled prefix
        // plus a kind tag, not a second spelling of it.
        let kind = match self.types.kind(ty) {
            TyKind::Tuple(_) => "tup_",
            TyKind::Fn { .. } => "fn_",
            _ => "arr_",
        };
        let base = format!("{}{kind}{stem}", ember_branding::mangle_prefix());
        if self.taken.insert(base.clone()) {
            return base;
        }
        let mut n: u32 = 2;
        loop {
            let candidate = format!("{base}_{n}");
            if self.taken.insert(candidate.clone()) {
                return candidate;
            }
            n += 1;
        }
    }
}

/// Turn a printed type such as `(i32, f32)` or `[f32; 4]` into the readable
/// core of a C identifier: `i32_f32`, `f32_4`.
fn identifier_from(shown: &str) -> String {
    let mut out = String::new();
    for ch in shown.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_matches('_').to_string()
}

/// The C name of a declared Ember type. A name from another module carries a
/// dotted prefix (`math.ops.Point`), which C cannot spell.
fn c_name(name: &str) -> String {
    ember_branding::mangled(name)
}

/// The parameter which supplies one stable class-interface handle to this
/// receiver. A direct parameter is already the handle. A `mut I` parameter is
/// reborrowed as a short-lived MIR local of the form `&mut (*arg)` before each
/// call, so recover its original argument storage for the shared cache key.
fn interface_cache_parameter_root(body: &Body, receiver: &Place) -> Option<usize> {
    if !receiver.projection.is_empty() {
        return None;
    }
    let local = receiver.local.0 as usize;
    if body.locals.get(local).is_some_and(|decl| decl.kind == LocalKind::Arg) {
        return Some(local);
    }
    for block in &body.blocks {
        for statement in &block.stmts {
            let StmtKind::Assign {
                place,
                rvalue: Rvalue::Ref { place: source, mutable: true },
            } = &statement.kind
            else {
                continue;
            };
            if !place.projection.is_empty() || place.local.0 as usize != local {
                continue;
            }
            if !matches!(source.projection.as_slice(), [Projection::Deref]) {
                continue;
            }
            let root = source.local.0 as usize;
            if body.locals.get(root).is_some_and(|decl| decl.kind == LocalKind::Arg) {
                return Some(root);
            }
        }
    }
    None
}

/// `[DSP-3]` — cache a TypeInfo lookup when the same interface parameter is
/// dispatched more than once. Restricting this cache to function parameters is
/// intentional: they are initialized before every entry path; `mut I` uses
/// only the compiler-created reborrow of that same parameter. Ordinary locals
/// and projections remain direct lookups until they have an equally explicit
/// control-flow and invalidation proof.
fn interface_cache_plan(body: &Body) -> BTreeMap<InterfaceCacheKey, String> {
    let mut counts = BTreeMap::<InterfaceCacheKey, usize>::new();
    for block in &body.blocks {
        let Terminator::Call {
            func:
                FuncRef::Interface {
                    interface,
                    class_handle: true,
                    ..
                },
            args,
            ..
        } = &block.terminator
        else {
            continue;
        };
        let Some(Operand::Copy(place)) = args.first() else { continue };
        let Some(local) = interface_cache_parameter_root(body, place) else { continue };
        *counts
            .entry(InterfaceCacheKey {
                local,
                interface: interface.to_string(),
            })
            .or_default() += 1;
    }
    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(key, _)| {
            let name = ember_branding::mangled(&format!(
                "itable_cache_{}_{}",
                key.local, key.interface
            ));
            (key, name)
        })
        .collect()
}

/// Locals and parameters that are never read anywhere in the body.
///
/// A method that ignores its receiver — `fn sides(self) -> i32: return 4` — is
/// ordinary Ember, and so is a `_`-shaped parameter, but both trip
/// `-Wunused-parameter`.
fn unread_locals(body: &Body) -> Vec<usize> {
    let mut read = vec![false; body.locals.len()];

    fn read_place(place: &Place, read: &mut [bool]) {
        read[place.local.0 as usize] = true;
        for projection in &place.projection {
            if let Projection::Index(local) = projection {
                read[local.0 as usize] = true;
            }
        }
    }
    fn read_operand(operand: &Operand, read: &mut [bool]) {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => read_place(place, read),
            Operand::Const(_) => {}
        }
    }
    // A projection on the left of an assignment reads the local it starts
    // from: `x.f = 1` needs `x`.
    fn write_place(place: &Place, read: &mut [bool]) {
        if !place.projection.is_empty() {
            read_place(place, read);
        }
    }

    for block in &body.blocks {
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    write_place(place, &mut read);
                    match rvalue {
                        Rvalue::Use(o) | Rvalue::UnaryOp { operand: o, .. } => {
                            read_operand(o, &mut read)
                        }
                        Rvalue::Cast { operand, .. } => read_operand(operand, &mut read),
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            read_operand(lhs, &mut read);
                            read_operand(rhs, &mut read);
                        }
                        Rvalue::Aggregate { operands, .. } => {
                            operands.iter().for_each(|o| read_operand(o, &mut read))
                        }
                        Rvalue::Repeat { value, .. } => read_operand(value, &mut read),
                        Rvalue::Discriminant(place) => read_place(place, &mut read),
                        // Taking a local's address counts as using it: the
                        // callee may write through the reference.
                        Rvalue::Ref { place, .. } => {
                            read[place.local.0 as usize] = true;
                            read_place(place, &mut read);
                        }
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    write_place(dest, &mut read);
                    write_place(overflow, &mut read);
                    read_operand(lhs, &mut read);
                    read_operand(rhs, &mut read);
                }
                // A drop reads what it is dropping, and its flag.
                StmtKind::Drop { place, flag } => {
                    read_place(place, &mut read);
                    if let Some(flag) = flag {
                        read[flag.0 as usize] = true;
                    }
                }
                StmtKind::BeginAccess { place, .. }
                | StmtKind::BeginAccessTransfer { place, .. }
                | StmtKind::EndAccess { place, .. }
                | StmtKind::EndAccessTransfer { place, .. } => {
                    read_place(place, &mut read);
                }
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
            }
        }
        match &block.terminator {
            Terminator::SwitchInt { discr, .. } => read_operand(discr, &mut read),
            Terminator::Call { args, dest, .. } => {
                args.iter().for_each(|a| read_operand(a, &mut read));
                write_place(dest, &mut read);
            }
            Terminator::Assert { cond, msg, .. } => {
                read_operand(cond, &mut read);
                if let AssertKind::Bounds { len, index } = msg {
                    read_operand(len, &mut read);
                    read_operand(index, &mut read);
                }
                if let AssertKind::RefCellBorrow { file, line } = msg {
                    read_operand(file, &mut read);
                    read_operand(line, &mut read);
                }
                if let AssertKind::Panic { message } = msg {
                    read_operand(message, &mut read);
                }
            }
            Terminator::Return => read[RETURN_LOCAL.0 as usize] = true,
            Terminator::Goto(_) | Terminator::Unreachable => {}
        }
    }

    body.locals
        .iter()
        .enumerate()
        .filter(|(index, decl)| decl.kind != LocalKind::Return && !read[*index])
        .map(|(index, _)| index)
        .collect()
}

/// Which blocks are the target of a `goto` in the emitted C.
///
/// This must agree exactly with `emit_terminator`'s fallthrough rule: a jump
/// to the next block in order emits no `goto`, so it does not make that block
/// referenced.
fn referenced_blocks(body: &Body) -> std::collections::BTreeSet<usize> {
    let mut referenced = std::collections::BTreeSet::new();
    for (index, block) in body.blocks.iter().enumerate() {
        match &block.terminator {
            Terminator::Goto(target)
            | Terminator::Call { next: target, .. }
            | Terminator::Assert { next: target, .. } => {
                if target.0 as usize != index + 1 {
                    referenced.insert(target.0 as usize);
                }
            }
            Terminator::SwitchInt { targets, otherwise, .. } => {
                for (_, target) in targets {
                    referenced.insert(target.0 as usize);
                }
                referenced.insert(otherwise.0 as usize);
            }
            Terminator::Return | Terminator::Unreachable => {}
        }
    }
    referenced
}

/// `[LEX-19]` — a parsed format spec as a C struct literal.
fn fmt_spec_literal(spec: &ember_mir::FormatSpec) -> String {
    let letter = |c: Option<char>| c.map_or("0".to_string(), |c| format!("'{c}'"));
    format!(
        "(({RT}fmt_spec){{ {}u, {}, {}, {}, {}, {}u, {}, {}, {} }})",
        spec.fill as u32,
        letter(spec.align),
        letter(spec.sign),
        spec.alternate,
        spec.zero,
        spec.width,
        letter(spec.grouping),
        spec.precision.map_or(-1, i64::from),
        letter(spec.kind)
    )
}

/// Whether a spec does more than name a conversion (`!r` is kind `?`, `!s`
/// kind `s`): a fill, an alignment, a width or a precision.
fn hir_spec_is_more_than_a_conversion(spec: &ember_mir::FormatSpec) -> bool {
    (ember_mir::FormatSpec { kind: None, ..*spec }) != ember_mir::FormatSpec::PLAIN
}

/// `[TYP-39]` — the `index`th generated `Display` function.
fn fmt_fn_symbol(index: usize) -> String {
    ember_branding::mangled(&format!("fmt_{index}"))
}

/// D-187 — the `index`th generated equality function.
fn eq_fn_symbol(index: usize) -> String {
    ember_branding::mangled(&format!("eq_{index}"))
}

/// `parts` joined with `&&`; true when there are none.
fn conjunction(parts: Vec<String>) -> String {
    if parts.is_empty() { "1".to_string() } else { parts.join(" && ") }
}

/// D-182 — the `index`th out-of-line drop-glue function.
/// `sizeof` of a C type the backend names. C gives `void` no size (MSVC says
/// 0, GCC and Clang 1); Ember's `void` is zero-sized (`[TYP-1]`), so a buffer
/// of it stores nothing and every element is at its base pointer (D-354,
/// D-355). A `void` struct member is a byte (`c_member_type`); only buffers,
/// boxes and `size_of` see this size.
fn c_size(c: &str) -> String {
    if c == "void" { "0".to_string() } else { format!("sizeof({c})") }
}

/// A pointer to element `index` of the buffer at `pointer`, as a `const` or
/// mutable `elem_c*`. C cannot step a `void*`; every element of a zero-sized
/// type is at the base pointer (D-354, ODR-068).
fn element_pointer(pointer: &str, elem_c: &str, index: &str, mutable: bool) -> String {
    let qualifier = if mutable { "" } else { "const " };
    if elem_c == "void" {
        format!("(({qualifier}void*)({pointer}))")
    } else {
        format!("((({qualifier}{elem_c}*)({pointer})) + {index})")
    }
}

/// `_Alignof` of a C type the backend names; `void`'s is 1 (`[TYP-1]`).
fn c_align(c: &str) -> String {
    if c == "void" { "1".to_string() } else { format!("_Alignof({c})") }
}

/// `[EXC-19]` — the C member holding a class field's access word.
fn access_word_member(field: &str) -> String {
    format!("_access_{field}")
}

fn drop_glue_symbol(index: usize) -> String {
    ember_branding::mangled(&format!("drop_glue_{index}"))
}

/// The `ember_ck_*` stem for an operator.
fn checked_name(op: ember_mir::BinOp) -> &'static str {
    use ember_mir::BinOp;
    match op {
        BinOp::Add => "add",
        BinOp::Sub => "sub",
        BinOp::Mul => "mul",
        BinOp::Div => "div",
        BinOp::Rem => "rem",
        BinOp::FloorDiv => "floordiv",
        BinOp::FloorRem => "floorrem",
        // Nothing else is lowered through `CheckedBinaryOp`.
        _ => "add",
    }
}

/// Render a float so that the C compiler reads back exactly the value Ember
/// computed. Shortest round-tripping decimal, then the `f` suffix for `f32`.
fn render_float(value: f64, is_f32: bool) -> String {
    if value.is_nan() {
        return if is_f32 { "EMBER_NAN_F32".into() } else { "EMBER_NAN_F64".into() };
    }
    if value.is_infinite() {
        let sign = if value < 0.0 { "-" } else { "" };
        return format!("{sign}EMBER_INF_F64");
    }
    for precision in 1..=17 {
        let text = format!("{value:.*e}", precision);
        if text.parse::<f64>() == Ok(value) {
            let rendered = format!("{value:.*e}", precision);
            return if is_f32 { format!("(float){rendered}") } else { rendered };
        }
    }
    let text = format!("{value:e}");
    if is_f32 { format!("(float){text}") } else { text }
}

/// A C string literal with every byte escaped that needs it. Non-ASCII bytes
/// are emitted as hex escapes so the file stays ASCII whatever the compiler's
/// source charset is.
fn c_string_literal(text: &str) -> String {
    let mut out = String::from("\"");
    for byte in text.bytes() {
        match byte {
            b'"' => out.push_str("\\\""),
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            0x20..=0x7E => out.push(byte as char),
            // `\xNN` followed by a hex digit would keep consuming, so close
            // and reopen the literal.
            other => out.push_str(&format!("\\x{other:02x}\" \"")),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_round_trip_through_the_emitted_text() {
        for value in [0.1f64, 1.0, 5.0, 1e300, -2.5, 1.0 / 3.0] {
            let text = render_float(value, false);
            assert_eq!(text.parse::<f64>(), Ok(value), "{value} rendered as {text}");
        }
    }

    #[test]
    fn f32_constants_are_cast_so_the_c_compiler_does_not_widen() {
        let text = render_float(0.5, true);
        assert!(text.starts_with("(float)"), "{text}");
    }

    #[test]
    fn string_literals_escape_what_c_needs() {
        assert_eq!(c_string_literal("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
    }

    #[test]
    fn non_ascii_bytes_do_not_swallow_the_next_character() {
        // "\xc3\xa9" followed by 'a' must not lex as "\xc3a9a".
        let text = c_string_literal("éa");
        assert!(text.contains("\\xc3\" \""), "{text}");
        assert!(text.ends_with("a\""), "{text}");
    }
}
