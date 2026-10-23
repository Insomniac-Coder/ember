//! Annex B — what a reloadable image says of itself, decided once for both backends: the functions
//! its manifest lists and their slots (`[HR-5]`–`[HR-8]`), every type's schema (`[HR-11]`), what a
//! value it holds refers to (`[HR-13]`, `[HR-15]`), its statics (`[HR-4a]`, `[HR-17]`) and each
//! function's frame record (`[HR-3b]`). The decisions are the `Planner`'s; a backend writes them
//! through its `Writer`: the C backend as C initialisers (`reload.rs`), the LLVM backend as IR
//! constants. The vocabulary of what a row says is the runtime header's own (`EMBER_RL_KIND_*`,
//! `EMBER_RL_FIELD_*`, …; `constant` gives each one's value).

use std::collections::{BTreeMap, BTreeSet};

use ember_mir::{Body, Builtin, CastKind, Const, FuncRef, Operand, Place, Projection, Rvalue, StmtKind, Terminator};
use ember_span::SourceMap;
use ember_types::{Bound, ClassId, EnumId, FloatTy, IntTy, RangeDef, RangeId, StructId, Ty, TyKind, TypeTable, UintTy};

/// What the driver decides of a reloadable build, which the backend writes into the image.
#[derive(Clone, Debug, Default)]
pub struct ReloadImage {
    /// `[build] reload`'s mode as the manifest records it: 1 `opt-in`, 2 `all`, 3 `bodies`.
    pub mode: u32,
    /// `[HR-12]` — the 40-byte object header with its live-instance list.
    pub instances: bool,
    /// The package's name, which the manifest carries.
    pub package: String,
    /// `[HR-5]` — every function reached through a thunk, by symbol.
    pub reloadable: BTreeSet<String>,
    /// `[HR-10]` — every `@noreload` function, by symbol, with its body's hash.
    pub noreload: BTreeMap<String, u64>,
    /// `[HR-17]`, `[HR-17a]` — every static the package declares, by its name: its initialiser's
    /// hash, its declared type's, and whether it is `@reinit_on_reload`. One whose value the
    /// compiler folds into the code has no storage, but an edit of it is still refused.
    pub statics: BTreeMap<String, (u64, u64, bool)>,
    /// The package's own source files: the types declared in them have schemas ([HR-11]).
    pub files: BTreeSet<ember_span::FileId>,
    /// `[HR-14]` — each field with a declared default, by its type's name and its own, with the
    /// name of the function the driver wrote to compute the default.
    pub defaults: BTreeMap<(String, String), String>,
    /// `[HR-14]` — `@renamed_from("old")` on a field or a variant, by its type's name and its own.
    pub renamed: BTreeMap<(String, String), String>,
    /// `[HR-16]` — each type that declares `migrate_from`, with the name of the adapter the driver
    /// wrote, and the name of the struct its old schema became.
    pub migrations: BTreeMap<String, (String, String)>,
    /// `[HR-23]` — each C++ header the package imports, by name, with the content hash of it and
    /// every file it includes.
    pub headers: BTreeMap<String, String>,
}

impl ReloadImage {
    /// `[HR-12]` — the object header's size, in bytes, of every object of the image.
    pub fn header_bytes(&self) -> u64 {
        if self.instances { 40 } else { 24 }
    }
}

/// FNV-1a: a schema or signature hash, stable across builds and hosts.
pub fn fnv(parts: &[&str]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for part in parts {
        for byte in part.bytes().chain(std::iter::once(0xff)) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
    }
    hash
}

/// The image's own tables and manifest, named from the mangling prefix.
pub fn table(name: &str) -> String {
    format!("{}rl_{name}", ember_branding::mangle_prefix())
}

/// The manifest's symbol, which the image's link keeps (`[HR-8]`).
pub fn manifest_symbol() -> String {
    format!("{}reload_manifest", ember_branding::mangle_prefix())
}

/// The image's entry, which the runtime calls (`[HR-33]`).
pub fn entry_symbol() -> String {
    format!("{}rl_entry", ember_branding::mangle_prefix())
}

/// The suffix of a reloadable body's own symbol; the function's symbol is its thunk's.
const BODY_SUFFIX: &str = "__rl";

/// The symbol a reloadable function's body is defined under (`[HR-6]`).
pub fn body_symbol(symbol: &str) -> String {
    format!("{symbol}{BODY_SUFFIX}")
}

/// A class's identity record across images, by the class's mangled form (`[MNG-1]`).
pub fn class_identity_symbol(mangled_class: &str) -> String {
    format!("{}_{mangled_class}", ember_branding::generated("tid"))
}

/// `[HR-3b]` — a function's frame description, by its symbol.
pub fn frame_symbol(symbol: &str) -> String {
    format!("{}frame_{symbol}", ember_branding::mangle_prefix())
}

/// `[HR-14]` — the symbol of range type `id`'s test.
pub fn range_test_symbol(id: RangeId) -> String {
    table(&format!("in_range_{}", id.0))
}

/// `[HR-14]` — whether a range type of representation `kind` has a test of its range (`T.checked`'s,
/// `[RNG-3]`): every integer and `f32`/`f64`, not `f16`, whose change the reload refuses.
pub fn range_testable(kind: &TyKind, def: &RangeDef) -> bool {
    matches!(
        (kind, def.lo, def.hi),
        (TyKind::Int(IntTy::I8 | IntTy::I16 | IntTy::I32 | IntTy::I64 | IntTy::I128), Bound::Int(_), Bound::Int(_))
            | (TyKind::Uint(UintTy::U8 | UintTy::U16 | UintTy::U32 | UintTy::U64 | UintTy::U128), Bound::Int(_), Bound::Int(_))
            | (TyKind::Float(FloatTy::F32 | FloatTy::F64), Bound::Float(_), Bound::Float(_))
    )
}

/// The value of one of the runtime header's reload constants (`EMBER_RL_KIND_HANDLE`), by its name.
pub fn constant(name: &str) -> u32 {
    match name {
        "EMBER_RL_KIND_PLAIN" => 0,
        "EMBER_RL_KIND_HANDLE" => 1,
        "EMBER_RL_KIND_OPTION_HANDLE" => 2,
        "EMBER_RL_KIND_WEAK" => 3,
        "EMBER_RL_KIND_STRUCT" => 4,
        "EMBER_RL_KIND_ENUM" => 5,
        "EMBER_RL_KIND_ARRAY" => 6,
        "EMBER_RL_KIND_OPAQUE" => 7,
        "EMBER_RL_KIND_PAYLOAD" => 8,
        "EMBER_RL_KIND_TUPLE" => 9,
        "EMBER_RL_KIND_BOX" => 10,
        "EMBER_RL_KIND_CALLABLE" => 11,
        "EMBER_RL_KIND_FIXED" => 12,
        "EMBER_RL_KIND_SOA" => 13,
        "EMBER_RL_FIELD_DEFAULT" => 1,
        "EMBER_RL_FIELD_ZERO" => 2,
        "EMBER_RL_FIELD_SIGNED" => 4,
        "EMBER_RL_FIELD_UNSIGNED" => 8,
        "EMBER_RL_FIELD_FLOAT" => 16,
        "EMBER_RL_TYPE_STRUCT" => 1,
        "EMBER_RL_TYPE_CLASS" => 2,
        "EMBER_RL_TYPE_ENUM" => 3,
        "EMBER_RL_TYPE_INTERFACE" => 4,
        "EMBER_RL_LAYOUT_CONTAINED" => 1,
        "EMBER_RL_LAYOUT_UNIT" => 2,
        "EMBER_RL_LAYOUT_FOREIGN" => 4,
        "EMBER_RL_LAYOUT_ABI" => 8,
        "EMBER_RL_FN_RELOADABLE" => 1,
        "EMBER_RL_FN_NORELOAD" => 2,
        "EMBER_RL_FN_ADDRESS" => 4,
        "EMBER_RL_FN_EXPORT" => 8,
        "EMBER_RL_STATIC_REINIT" => 1,
        "EMBER_RL_STATIC_CONSTANT" => 2,
        other => panic!("`{other}` is not a reload constant of the runtime header"),
    }
}

/// The value of a list of the header's flags, or-ed.
pub fn flags_value(flags: &[&str]) -> u32 {
    flags.iter().fold(0, |value, flag| value | constant(flag))
}

/// What a size, an offset or an alignment is measured on: a value of a type, or a class's object
/// (its header and its fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holder {
    Ty(Ty),
    Object(ClassId),
}

/// A member of a holder: a struct's or a class's field by its index, a tuple's item, an enum's
/// payload field (variant, field), an enum's tag, or a field of the compiler's own struct by its
/// name (an `SoA`'s column pointer, `len`, `cap`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Member {
    Field(usize),
    Tuple(usize),
    Payload(usize, usize),
    Tag,
    Named(String),
}

/// A row's number, as each backend writes it: C as `sizeof`, `offsetof` and `_Alignof` of its own
/// declarations, IR as the number the type table's layout gives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Num {
    Zero,
    Size(Holder),
    MemberSize(Holder, Member),
    Align(Holder),
    /// The alignment a container's buffer of `Ty` is allocated with (`EMBER_VEC_ALIGN_OF`).
    VecAlign(Ty),
    Offset(Holder, Member),
    /// Where a field reached by these indices sits in a value of the struct `Ty`.
    Within(Ty, Vec<usize>),
    /// `sizeof(size_t)`.
    SizeT,
}

/// What a field holds, for the runtime's migration and reference walk ([HR-14], [HR-15]): its kind,
/// its flags, its type's name, and for an `Array` its elements'. `layout` and `inner` are the
/// symbols of tables this image describes a tuple's fields or an element's or pointee's value with.
#[derive(Clone, Debug)]
pub struct FieldShape {
    pub kind: &'static str,
    pub flags: Vec<&'static str>,
    pub type_name: String,
    pub element_kind: &'static str,
    pub element_size: Num,
    pub element_type: String,
    pub layout: Option<String>,
    pub inner: Option<String>,
    /// `[HR-14]` — for a range type, its test of a value of its representation (`T.checked`'s).
    pub range: Option<RangeId>,
    /// `[HR-13]` — for a container's element row, the alignment its buffer is allocated with.
    pub align: Num,
    /// `[HR-13]` — for an `SoA`'s column, where its value sits in a whole element.
    pub within: Num,
}

impl FieldShape {
    fn plain(type_name: String) -> FieldShape {
        FieldShape {
            kind: "EMBER_RL_KIND_PLAIN",
            flags: Vec::new(),
            type_name,
            element_kind: "EMBER_RL_KIND_PLAIN",
            element_size: Num::Zero,
            element_type: String::new(),
            layout: None,
            inner: None,
            range: None,
            align: Num::Zero,
            within: Num::Zero,
        }
    }
}

/// One `ember_rl_field`: its name, `@renamed_from`'s, its schema hash (`None` writes 0), where it
/// sits and its size, and what it holds (whose flags and type name are the row's).
#[derive(Clone, Debug)]
pub struct Row {
    pub name: String,
    pub renamed: Option<String>,
    pub schema: Option<u64>,
    pub offset: Num,
    pub size: Num,
    pub shape: FieldShape,
}

impl Row {
    /// An enum's tag as its type's one field.
    fn tag(ty: Ty) -> Row {
        Row {
            name: "tag".to_string(),
            renamed: None,
            schema: None,
            offset: Num::Offset(Holder::Ty(ty), Member::Tag),
            size: Num::MemberSize(Holder::Ty(ty), Member::Tag),
            shape: FieldShape::plain("tag".to_string()),
        }
    }
}

/// One `ember_rl_type`. `fields` and `variants` are a count and an array's symbol (`None` writes
/// `0, NULL`); `methods` and `attributes` `None` write 0.
#[derive(Clone, Debug)]
pub struct TypeRow {
    pub name: String,
    pub kind: &'static str,
    pub base: Option<String>,
    pub layout: Vec<&'static str>,
    pub size: Num,
    pub align: Num,
    pub schema: u64,
    pub methods: Option<u64>,
    pub type_info: Option<String>,
    pub identity: Option<String>,
    pub fields: Option<(usize, Option<String>)>,
    pub variants: Option<(usize, Option<String>)>,
    pub init: Option<String>,
    pub migrate: Option<String>,
    pub old_type: Option<String>,
    pub drop_field: Option<String>,
    pub attributes: Option<u64>,
    pub columns: Option<String>,
}

/// One `ember_rl_variant`; `payload` `None` writes 0.
#[derive(Clone, Debug)]
pub struct VariantRow {
    pub name: String,
    pub renamed: Option<String>,
    pub discriminant: u64,
    pub payload: Option<u64>,
    pub fields: (usize, Option<String>),
}

/// One case of a type's `init_field`: field `index` at `offset` of a value of `ty`, written with
/// the value its default function `function` returns.
#[derive(Clone, Debug)]
pub struct InitCase {
    pub index: u32,
    pub holder: Holder,
    pub member: Member,
    pub ty: Ty,
    pub function: String,
}

/// One case of a type's `drop_field`: field `index`, a value of `ty` at its member, dropped.
#[derive(Clone, Debug)]
pub struct DropCase {
    pub index: u32,
    pub holder: Holder,
    pub member: Member,
    pub ty: Ty,
}

/// One function of the manifest.
#[derive(Clone, Debug)]
pub enum FnRow {
    /// Reached through its thunk ([HR-5]): its body, thunk and value's descriptor are the image's.
    Reloadable { symbol: String, descriptor: Option<String>, signature: u64, body_hash: u64, flags: Vec<&'static str> },
    /// `[HR-10]` — listed for its body's hash.
    NoReload { symbol: String, body_hash: u64 },
}

/// One static of the manifest: one with storage, its value's shape the table `shape`, constant
/// data when it has no initialisation code (`constant`); or one the compiler folded into the code.
#[derive(Clone, Debug)]
pub enum StaticRow {
    Stored { name: String, ty: Ty, schema: u64, initialiser: u64, flags: Vec<&'static str>, constant: bool, shape: String },
    Folded { name: String, declared: u64, initialiser: u64, reinit: bool },
}

/// `[HR-8]` — the manifest, with every table it points at.
#[derive(Clone, Debug)]
pub struct Manifest {
    pub mode: u32,
    pub package: String,
    pub instances: bool,
    pub fns: Vec<FnRow>,
    pub statics: Vec<StaticRow>,
    pub types: Vec<TypeRow>,
    pub identities: Vec<String>,
    pub headers: Vec<(String, String)>,
    pub has_entry: bool,
}

/// `[HR-3b]` — a function's frame description: its name and, for each of its calls in block order,
/// the line and the types the rest of the function depends on; with each call's site by block.
#[derive(Clone, Debug)]
pub struct FrameInfo {
    pub symbol: String,
    pub function: String,
    pub sites: Vec<(u32, Vec<String>)>,
    pub by_block: BTreeMap<usize, u32>,
}

/// How a backend writes what the planner decides, in the order it decides it (each table before
/// anything that names it), and the facts only the backend knows.
pub trait Writer {
    /// `[HR-14]` — range type `id`'s test, named `symbol`.
    fn range_test(&mut self, symbol: &str, id: RangeId);
    /// One field's row as an object of its own.
    fn field(&mut self, symbol: &str, row: &Row);
    /// Rows as an array.
    fn fields(&mut self, symbol: &str, rows: &[Row]);
    /// An enum's tag as an array of one row.
    fn tag(&mut self, symbol: &str, row: &Row);
    /// A type's row as an object of its own.
    fn type_row(&mut self, symbol: &str, row: &TypeRow);
    fn variants(&mut self, symbol: &str, rows: &[VariantRow]);
    /// `[HR-14]` — `void symbol(void* object, uint32_t index)`, writing field `index`'s default.
    fn init_fn(&mut self, symbol: &str, cases: &[InitCase]);
    /// `[HR-2a]` — `void symbol(void* object, uint32_t index)`, dropping field `index`.
    fn drop_fn(&mut self, symbol: &str, cases: &[DropCase]);
    /// `[HR-16]` — `int symbol(void* out, const void* old, char* report, size_t report_size)`, calling
    /// the driver's wrapper `wrapper`, which returns the `Option` `ret` (its `Some` variant `some`).
    fn migrate_fn(&mut self, symbol: &str, wrapper: &str, ret: Ty, option: EnumId, some: usize);
    /// `[HR-3b]` — a function's frame description.
    fn frame_info(&mut self, info: &FrameInfo);
    fn manifest(&mut self, manifest: &Manifest);
    /// The function laying out an `SoA`'s one block, which a reload rebuilding it calls (`[HR-13]`).
    fn soa_columns(&mut self, soa: StructId) -> String;
    /// A function's signature hash: what a slot keeps while its address may be held.
    fn signature_hash(&self, body: &Body) -> u64;
    /// A class's method tables' layout, hashed: instances keep their data when it changes, but not
    /// their type information.
    fn methods_hash(&self, class: ClassId) -> u64;
    /// The descriptor of a function's value, when the image makes values of it.
    fn descriptor(&self, symbol: &str) -> Option<String>;
    /// The interface identities the image defines.
    fn interface_identities(&self) -> Vec<String>;
    /// Each owned closure value's captured state's type.
    fn closure_envs(&self) -> Vec<Ty>;
    /// Each type made into an interface value (`Box[dyn I]`, a `dyn` borrow).
    fn dyn_concretes(&self) -> Vec<Ty>;
}

/// `[STA-3]` — every static initialised on first access, by its initialiser's symbol: its type and
/// its name.
pub fn lazy_statics(bodies: &[Body]) -> BTreeMap<String, (Ty, String)> {
    bodies
        .iter()
        .flat_map(|body| &body.blocks)
        .filter_map(|block| match &block.terminator {
            Terminator::Call { func: FuncRef::Builtin { which: Builtin::LazyStatic { name, init, ty }, .. }, .. } => {
                Some((init.to_string(), (*ty, name.to_string())))
            }
            _ => None,
        })
        .collect()
}

/// A struct the compiler supplies, which no edit changes.
fn is_compiler_struct(name: &str) -> bool {
    // ODR-275 — not `UnsafeCell_`: a `Mutex`'s or an `Atomic`'s value is kept in one, which has a
    // schema of its own when it holds the package's types, so a reload follows into it.
    ["Box_", "Shared_", "Weak_", "Cell_", "RefCell_", "MaybeUninit_", "closure", "Option_", "Result_"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// `[HR-7]` — the functions whose address the program takes: a function value, a table a `dyn`
/// value points at. Data may hold their thunks, so removing one is refused.
pub fn address_taken_symbols(bodies: &[Body]) -> BTreeSet<&str> {
    let mut taken = BTreeSet::new();
    for body in bodies {
        for block in &body.blocks {
            for stmt in &block.stmts {
                if let StmtKind::Assign { rvalue, .. } = &stmt.kind {
                    match rvalue {
                        Rvalue::Use(Operand::Const(Const::Fn(symbol))) => {
                            taken.insert(symbol.as_str());
                        }
                        Rvalue::Aggregate { operands, .. } => {
                            for operand in operands {
                                if let Operand::Const(Const::Fn(symbol)) = operand {
                                    taken.insert(symbol.as_str());
                                }
                            }
                        }
                        Rvalue::Cast { kind: CastKind::InterfaceUpcast { implementations, .. } | CastKind::ClassInterfaceUpcast { implementations, .. }, .. } => {
                            taken.extend(implementations.iter().flatten().map(|method| method.symbol.as_str()));
                        }
                        _ => {}
                    }
                }
            }
            if let Terminator::Call { func, args, .. } = &block.terminator {
                if let FuncRef::DynBoxNew { implementations, .. } = func {
                    taken.extend(implementations.iter().flatten().map(|method| method.symbol.as_str()));
                }
                for arg in args {
                    if let Operand::Const(Const::Fn(symbol)) = arg {
                        taken.insert(symbol.as_str());
                    }
                }
            }
        }
    }
    taken
}

/// A type's schema hash ([HR-11]): its kind, name, layout and each field's or variant's name
/// and schema, so equal hashes mean nothing to migrate. A type met again inside itself (behind
/// a pointer) counts by its name. `[LAY-2]` — its layout attributes too, when it has any: a
/// change to them is a schema change, which migrates by name, or is refused for a type a raw
/// pointer can reach (`[HR-14]`'s last row, through `[HR-13b]`).
pub fn type_schema(types: &TypeTable, ty: Ty, seen: &mut Vec<Ty>) -> u64 {
    if seen.contains(&ty) {
        return fnv(&["again", &types.display(ty)]);
    }
    seen.push(ty);
    let hash = match types.kind(ty) {
        TyKind::Struct(id) => {
            let def = types.struct_def(*id);
            let mut parts = vec!["struct".to_string(), def.name.to_string()];
            parts.extend(layout_schema(types, ty));
            for field in &def.fields {
                parts.push(field.name.to_string());
                parts.push(type_schema(types, field.ty, seen).to_string());
            }
            fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
        }
        TyKind::Enum(id) => {
            let def = types.enum_def(*id);
            let mut parts = vec!["enum".to_string(), def.name.to_string(), types.display(def.repr)];
            parts.extend(layout_schema(types, ty));
            for variant in &def.variants {
                parts.push(variant.name.to_string());
                for field in &variant.fields {
                    parts.push(field.name.to_string());
                    parts.push(type_schema(types, field.ty, seen).to_string());
                }
            }
            fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
        }
        // Held inside another value, a class is a handle: its own layout is not the holder's.
        TyKind::Class(id) if seen.len() > 1 => fnv(&["handle", &types.class_def(*id).name.to_string()]),
        TyKind::Class(id) => {
            let mut parts = vec!["class".to_string(), types.class_def(*id).name.to_string()];
            if let Some(base) = types.class_def(*id).base {
                parts.push(types.class_def(base).name.to_string());
            }
            for index in 0..types.class_field_count(*id) {
                if let Some(field) = types.class_field_at(*id, index) {
                    parts.push(field.name.to_string());
                    parts.push(type_schema(types, field.ty, seen).to_string());
                }
            }
            fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
        }
        TyKind::Tuple(items) => {
            let mut parts = vec!["tuple".to_string()];
            for item in items.clone() {
                parts.push(type_schema(types, item, seen).to_string());
            }
            fnv(&parts.iter().map(String::as_str).collect::<Vec<_>>())
        }
        TyKind::Ptr { inner, .. } => {
            let inner = *inner;
            fnv(&["pointer", &types.display(ty), &type_schema(types, inner, seen).to_string()])
        }
        // `[HR-13]` — an `Array`'s or a fixed array's elements are in its schema: a container of
        // a type that changed holds values to migrate. A class element is a handle (above).
        TyKind::Vec { elem, text: false } => {
            let elem = *elem;
            fnv(&["array", &type_schema(types, elem, seen).to_string()])
        }
        TyKind::Array { elem, len } => {
            let (elem, len) = (*elem, *len);
            fnv(&["fixed", &format!("{len:?}"), &type_schema(types, elem, seen).to_string()])
        }
        // `[HR-14]` — a range type's range is in its schema: a value of the old one may lie
        // outside the new one.
        TyKind::Range(id) => {
            let def = types.range_def(*id);
            fnv(&["range", &def.name.to_string(), &types.display(def.repr), &format!("{:?}", def.lo), &format!("{:?}", def.hi), if def.inclusive { "..=" } else { ".." }])
        }
        _ => fnv(&[&types.display(ty)]),
    };
    seen.pop();
    hash
}

/// `[LAY-2]` — a type's layout attributes in its schema; nothing for a type without them, so
/// every other schema hash is as it was.
fn layout_schema(types: &TypeTable, ty: Ty) -> Option<String> {
    let attrs = types.layout_attrs(ty);
    (!attrs.is_none()).then(|| format!("layout packed={} align={}", attrs.packed, attrs.align))
}

/// The decisions of one reloadable image, and what deciding them has made so far: the tables
/// already written (each once), and each body's frame dependencies.
pub struct Planner<'a> {
    types: &'a TypeTable,
    pub reload: ReloadImage,
    /// Each function the manifest lists, by symbol, with its index in the image's tables.
    pub slots: BTreeMap<String, usize>,
    /// The statics with storage, by name, each with its type and, for a lazily initialised one,
    /// its initialiser's symbol.
    statics: Vec<(String, Ty, Option<String>)>,
    /// Every body's name, symbol and return type, for the functions the driver wrote.
    bodies: Vec<(String, String, Ty)>,
    usize_ty: Ty,
    /// `[HR-15]` — the tables describing a value no schema names, by type and whether the table
    /// lists the type's fields; and the types being described, met again inside themselves.
    shapes: BTreeMap<(Ty, bool), String>,
    shapes_open: BTreeSet<Ty>,
    /// `[HR-13]` — the rows describing a container's element by its schema, by element type, with
    /// the row's kind.
    elements: BTreeMap<Ty, (String, &'static str)>,
    /// `[HR-3b]` — what each body depends on with its direct callees, and what any old body may (a
    /// call through a function value).
    frame_layouts: BTreeMap<String, BTreeSet<String>>,
    old_layouts: BTreeSet<String>,
}

impl<'a> Planner<'a> {
    /// The image's plan: its slots, its statics, and when its instances migrate, each body's frame
    /// dependencies.
    pub fn new(types: &'a TypeTable, reload: &ReloadImage, bodies: &[Body], lazy: &BTreeMap<String, (Ty, String)>) -> Planner<'a> {
        let usize_ty = types.find(&TyKind::Uint(UintTy::Usize)).expect("the interner always holds `usize`");
        let mut statics: Vec<(String, Ty, Option<String>)> =
            types.constant_statics().iter().map(|(name, ty, _)| (name.to_string(), *ty, None)).collect();
        statics.extend(lazy.iter().map(|(init, (ty, name))| (name.clone(), *ty, Some(init.clone()))));
        statics.sort_by(|a, b| a.0.cmp(&b.0));
        statics.dedup_by(|a, b| a.0 == b.0);
        let mut planner = Planner {
            types,
            reload: reload.clone(),
            slots: BTreeMap::new(),
            statics,
            bodies: bodies.iter().map(|body| (body.name.clone(), body.symbol.clone(), body.return_ty())).collect(),
            usize_ty,
            shapes: BTreeMap::new(),
            shapes_open: BTreeSet::new(),
            elements: BTreeMap::new(),
            frame_layouts: BTreeMap::new(),
            old_layouts: BTreeSet::new(),
        };
        planner.plan_slots(bodies);
        if reload.instances {
            planner.plan_frame_layouts(bodies);
        }
        planner
    }

    pub fn is_reloadable(&self, symbol: &str) -> bool {
        self.reload.reloadable.contains(symbol)
    }

    /// The manifest's functions, in order: the reloadable ones and the `@noreload` ones, each
    /// with its index into the image's tables.
    fn plan_slots(&mut self, bodies: &[Body]) {
        let mut slots = BTreeMap::new();
        for body in bodies {
            if self.reload.reloadable.contains(&body.symbol) && !slots.contains_key(&body.symbol) {
                let next = slots.len();
                slots.insert(body.symbol.clone(), next);
            }
        }
        // `[HR-10]` — every `@noreload` function, whether or not its body was inlined away: a
        // reload that changes one is refused.
        for symbol in self.reload.noreload.keys() {
            let next = slots.len();
            slots.entry(symbol.clone()).or_insert(next);
        }
        self.slots = slots;
    }

    /// The statics the image keeps in the runtime's storage, by name, each with its type and,
    /// for a lazily initialised one, its initialiser's symbol.
    pub fn static_names(&self) -> &[(String, Ty, Option<String>)] {
        &self.statics
    }

    /// The manifest's statics: those with storage, then those the compiler folded away.
    pub fn static_entries(&self) -> usize {
        let folded = self.reload.statics.keys().filter(|name| !self.statics.iter().any(|(known, _, _)| known == *name)).count();
        self.statics.len() + folded
    }

    pub fn static_index(&self, name: &str) -> usize {
        self.statics.iter().position(|(known, _, _)| known == name).expect("a static of this image")
    }

    // -- types ------------------------------------------------------------------------------------

    fn is_void(&self, ty: Ty) -> bool {
        matches!(self.types.kind(ty), TyKind::Void | TyKind::Never | TyKind::Error)
    }

    fn box_inner_id(&self, id: StructId) -> Option<Ty> {
        self.types.compiler_box_inner(id)
    }

    fn shared_inner_id(&self, id: StructId) -> Option<Ty> {
        match &self.types.struct_def(id).origin {
            Some((name, args)) if (name.is("Shared") || name.is("SyncShared")) && args.len() == 1 => Some(args[0]),
            _ => None,
        }
    }

    fn weak_inner_id(&self, id: StructId) -> Option<Ty> {
        match &self.types.struct_def(id).origin {
            Some((name, args)) if name.is("Weak") && args.len() == 1 => Some(args[0]),
            _ => None,
        }
    }

    pub fn type_schema(&self, ty: Ty, seen: &mut Vec<Ty>) -> u64 {
        type_schema(self.types, ty, seen)
    }

    /// Whether a value of `ty` can hold a reference to an object, which a reload must follow.
    pub fn holds_objects(&self, ty: Ty, seen: &mut Vec<Ty>) -> bool {
        if seen.contains(&ty) {
            return false;
        }
        seen.push(ty);
        let holds = match self.types.kind(ty) {
            TyKind::Class(_) | TyKind::ClassInterface(_) | TyKind::Dyn { .. } | TyKind::Fn { .. } => true,
            TyKind::Struct(id) => {
                self.shared_inner_id(*id).is_some()
                    || self.weak_inner_id(*id).is_some()
                    || self.box_inner_id(*id).is_some_and(|inner| self.holds_objects(inner, seen))
                    || self.types.struct_def(*id).fields.iter().map(|field| field.ty).collect::<Vec<_>>().into_iter().any(|field| self.holds_objects(field, seen))
            }
            TyKind::Enum(id) => self
                .types
                .enum_def(*id)
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| field.ty))
                .collect::<Vec<_>>()
                .into_iter()
                .any(|field| self.holds_objects(field, seen)),
            TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } => self.holds_objects(*elem, seen),
            TyKind::Tuple(items) => items.clone().into_iter().any(|item| self.holds_objects(item, seen)),
            _ => false,
        };
        seen.pop();
        holds
    }

    /// `[HR-13]` — the package structs a value of which sits inside something that is not a
    /// package type's field: an `Array`, a tuple, a payload without a schema, a standard type. A
    /// reload cannot yet migrate such values, so a change to one is refused. Only a holder whose
    /// values can outlive a call counts: one held by another type, a closure's captured state or a
    /// static's value; a tuple a function makes and takes apart (`(self, default)` in a `match`) is
    /// gone before any poll.
    fn contained_structs(&self, closure_envs: &[Ty], dyn_concretes: &[Ty]) -> BTreeSet<String> {
        // A struct's or an enum's: an enum's values a reload cannot reach keep their old
        // discriminants as surely as a struct's keep their old layout.
        let schema_struct = |ty: Ty| -> Option<String> {
            match self.types.kind(ty) {
                TyKind::Struct(id) if self.has_schema(ty) => Some(self.types.struct_def(*id).name.to_string()),
                TyKind::Enum(id) if self.has_schema(ty) => Some(self.types.enum_def(*id).name.to_string()),
                _ => None,
            }
        };
        let mut held: BTreeSet<Ty> = closure_envs.iter().copied().collect();
        held.extend(self.statics.iter().map(|(_, ty, _)| *ty));
        for (_, kind) in self.types.all() {
            match kind {
                TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } => {
                    held.insert(*elem);
                }
                TyKind::Tuple(items) => held.extend(items.iter().copied()),
                TyKind::Struct(id) => held.extend(self.types.struct_def(*id).fields.iter().map(|field| field.ty)),
                TyKind::Enum(id) => held.extend(self.types.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty))),
                TyKind::Class(id) => held.extend(self.types.class_def(*id).fields.iter().map(|field| field.ty)),
                _ => {}
            }
        }
        // What a value of `ty` holds by value: an `SoA[T]`'s columns are its `T`s.
        let members = |ty: Ty| -> Vec<Ty> {
            match self.types.kind(ty) {
                TyKind::Vec { elem, text: false } | TyKind::Array { elem, .. } => vec![*elem],
                TyKind::Tuple(items) => items.clone(),
                TyKind::Enum(id) => self.types.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)).collect(),
                TyKind::Struct(id) => match &self.types.struct_def(*id).origin {
                    Some((origin, args)) if matches!(origin.as_str(), "SoA" | "ArenaSoA") && args.len() == 1 => args.clone(),
                    _ => self.types.struct_def(*id).fields.iter().map(|field| field.ty).collect(),
                },
                TyKind::Class(id) => self.types.class_def(*id).fields.iter().map(|field| field.ty).collect(),
                _ => Vec::new(),
            }
        };
        // Where a reload cannot reach: inside a tuple, or a struct, an enum or a class with no
        // schema. A struct, an enum with a schema and an `Array` or a fixed array migrate what
        // they hold (`[HR-14]`, `[HR-13]`), unless they sit where a reload cannot reach themselves.
        // A value made an interface value (`Box[dyn I]`) is behind its interface's methods, which a
        // reload repoints to code built for the new layout; the runtime cannot tell its type there.
        let mut unreached: Vec<Ty> =
            dyn_concretes.iter().copied().filter(|concrete| !matches!(self.types.kind(*concrete), TyKind::Class(_))).collect();
        for (ty, kind) in self.types.all() {
            // An instance of a class lives on the heap, held or not.
            if !held.contains(&ty) && !matches!(kind, TyKind::Class(_)) {
                continue;
            }
            let migrates = match kind {
                TyKind::Tuple(_) => false,
                TyKind::Class(id) => self.class_has_schema(*id),
                TyKind::Struct(_) => self.has_schema(ty) || self.soa_element(ty).is_some(),
                TyKind::Enum(_) => self.has_schema(ty),
                _ => true,
            };
            if !migrates {
                unreached.extend(members(ty));
            }
        }
        let mut seen = BTreeSet::new();
        let mut out = BTreeSet::new();
        while let Some(ty) = unreached.pop() {
            if !seen.insert(ty) {
                continue;
            }
            out.extend(schema_struct(ty));
            // A class is a handle: its own fields migrate with its instances.
            if !matches!(self.types.kind(ty), TyKind::Class(_)) {
                unreached.extend(members(ty));
            }
        }
        out
    }

    /// Whether a type mentions one the package declares, by value or through a container.
    fn mentions_package(&self, ty: Ty, seen: &mut Vec<Ty>) -> bool {
        if seen.contains(&ty) {
            return false;
        }
        seen.push(ty);
        let reload = &self.reload;
        let mentions = match self.types.kind(ty) {
            TyKind::Class(id) => reload.files.contains(&self.types.class_def(*id).span.file),
            TyKind::Struct(id) => {
                let def = self.types.struct_def(*id);
                // An `SoA[T]` holds its `T`s in columns, behind its fields' pointers.
                let soa = match &def.origin {
                    Some((origin, args)) if origin.as_str() == "SoA" => args.clone(),
                    _ => Vec::new(),
                };
                reload.files.contains(&def.span.file)
                    || def.fields.iter().map(|field| field.ty).collect::<Vec<_>>().into_iter().any(|field| self.mentions_package(field, seen))
                    || soa.into_iter().any(|element| self.mentions_package(element, seen))
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(*id);
                reload.files.contains(&def.span.file)
                    || def.variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)).collect::<Vec<_>>().into_iter()
                        .any(|field| self.mentions_package(field, seen))
            }
            TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } => self.mentions_package(*elem, seen),
            TyKind::Tuple(items) => items.clone().into_iter().any(|item| self.mentions_package(item, seen)),
            _ => false,
        };
        seen.pop();
        mentions
    }

    /// `[HR-13b]` — the package types a raw pointer can reach: a value behind one may sit in
    /// foreign memory, which no reload can find, so a change to its schema is refused.
    fn pointed_structs(&self, bodies: &[Body]) -> BTreeSet<String> {
        // `[HR-13]` — a pointer type that only an `SoA`'s own struct holds is its column, whose
        // values migrate through it: every other mention counts, in a type or a body's local.
        // A reference to a pointer is a place's (a column projected while it is read), not a
        // value's: the pointer it names is counted where it is held.
        let mut mentioned: BTreeSet<Ty> = bodies.iter().flat_map(|body| body.locals.iter().map(|local| local.ty)).collect();
        for (_, kind) in self.types.all() {
            match kind {
                TyKind::Struct(id) if matches!(&self.types.struct_def(*id).origin, Some((origin, _)) if origin.as_str() == "SoA") => {}
                TyKind::Struct(id) => mentioned.extend(self.types.struct_def(*id).fields.iter().map(|field| field.ty)),
                TyKind::Class(id) => mentioned.extend(self.types.class_def(*id).fields.iter().map(|field| field.ty)),
                TyKind::Enum(id) => mentioned.extend(self.types.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty))),
                TyKind::Tuple(items) => mentioned.extend(items.iter().copied()),
                TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } | TyKind::Ptr { inner: elem, .. } => {
                    mentioned.insert(*elem);
                }
                TyKind::Fn { params, ret, .. } => {
                    mentioned.extend(params.iter().map(|param| param.ty));
                    mentioned.insert(*ret);
                }
                _ => {}
            }
        }
        let mut out = BTreeSet::new();
        for (ty, kind) in self.types.all() {
            let TyKind::Ptr { inner, .. } = kind else { continue };
            if !mentioned.contains(&ty) {
                continue;
            }
            match self.types.kind(*inner) {
                TyKind::Struct(id) if self.has_schema(*inner) => {
                    out.insert(self.types.struct_def(*id).name.to_string());
                }
                TyKind::Class(id) if self.reload.files.contains(&self.types.class_def(*id).span.file) => {
                    out.insert(self.types.class_def(*id).name.to_string());
                }
                _ => {}
            }
        }
        out
    }

    /// Whether a type has a schema in this image: a struct, a class or a unit enum the package
    /// declares, and `[HR-13a]` a standard-library struct holding the package's types (a
    /// `Mutex[Array[Particle]]`), whose schema the instantiating package emits so a reload can
    /// follow and migrate what it holds.
    pub fn has_schema(&self, ty: Ty) -> bool {
        match self.types.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.types.struct_def(*id);
                // An `SoA`'s struct is its columns' pointers: its elements migrate by their own
                // schema (`soa_element`), never by its fields.
                !is_compiler_struct(&def.name.to_string())
                    && !matches!(&def.origin, Some((origin, _)) if matches!(origin.as_str(), "SoA" | "ArenaSoA"))
                    && self.structural_type_emitted(ty)
                    && (self.reload.files.contains(&def.span.file) || self.mentions_package(ty, &mut Vec::new()))
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(*id);
                if self.types.enum_is_bare(*id) {
                    self.reload.files.contains(&def.span.file)
                } else {
                    // `[HR-15]` — a payload enum (`Option[MapSlot[int, Particle]]` in a `Map`) has
                    // a schema too, so a reload can follow the references its payloads hold.
                    self.types.option_niche(*id).is_none()
                        && (self.reload.files.contains(&def.span.file) || self.mentions_package(ty, &mut Vec::new()))
                }
            }
            _ => false,
        }
    }

    /// The row a value of type `ty` placed as a field gets: its kind, its flags, its type's name,
    /// and for an `Array` its elements'.
    fn field_shape(&mut self, ty: Ty, w: &mut dyn Writer) -> FieldShape {
        let mut shape = FieldShape::plain(self.types.display(ty));
        // `[HR-15]` — a tuple, a `Box` and a callable value are described by this image's own
        // tables (ODR-126); as an `Array`'s element too, through `inner`.
        if let Some(own) = self.described_shape(ty, w) {
            return own;
        }
        let element = |this: &Self, elem: Ty| -> (&'static str, String) {
            match this.types.kind(elem) {
                TyKind::Class(_) => ("EMBER_RL_KIND_HANDLE", String::new()),
                TyKind::Enum(id) if this.types.option_niche(*id).is_some_and(|niche| matches!(this.types.kind(niche.payload), TyKind::Class(_))) => {
                    ("EMBER_RL_KIND_OPTION_HANDLE", String::new())
                }
                TyKind::Struct(id) if this.weak_inner_id(*id).is_some() => ("EMBER_RL_KIND_WEAK", String::new()),
                TyKind::Struct(id) if this.shared_inner_id(*id).is_some() => ("EMBER_RL_KIND_HANDLE", String::new()),
                TyKind::Struct(id) if this.has_schema(elem) => ("EMBER_RL_KIND_STRUCT", this.types.struct_def(*id).name.to_string()),
                TyKind::Enum(id) if !this.types.enum_is_bare(*id) && this.has_schema(elem) => {
                    ("EMBER_RL_KIND_PAYLOAD", this.types.enum_def(*id).name.to_string())
                }
                _ if this.holds_objects(elem, &mut Vec::new()) => ("EMBER_RL_KIND_OPAQUE", String::new()),
                _ => ("EMBER_RL_KIND_PLAIN", String::new()),
            }
        };
        match self.types.kind(ty) {
            TyKind::Int(_) => shape.flags.extend(["EMBER_RL_FIELD_SIGNED", "EMBER_RL_FIELD_ZERO"]),
            TyKind::Uint(_) => shape.flags.extend(["EMBER_RL_FIELD_UNSIGNED", "EMBER_RL_FIELD_ZERO"]),
            TyKind::Float(_) => shape.flags.extend(["EMBER_RL_FIELD_FLOAT", "EMBER_RL_FIELD_ZERO"]),
            TyKind::Bool => shape.flags.push("EMBER_RL_FIELD_ZERO"),
            // `[HR-14]` — a range type is its representation's number with a test of its range,
            // and never `Default` (`[RNG-8]`).
            TyKind::Range(id) => {
                let def = self.types.range_def(*id);
                let repr = self.types.kind(def.repr);
                if range_testable(repr, def) {
                    shape.flags.push(match repr {
                        TyKind::Uint(_) => "EMBER_RL_FIELD_UNSIGNED",
                        TyKind::Float(_) => "EMBER_RL_FIELD_FLOAT",
                        _ => "EMBER_RL_FIELD_SIGNED",
                    });
                    shape.range = Some(*id);
                }
            }
            TyKind::Class(_) => shape.kind = "EMBER_RL_KIND_HANDLE",
            TyKind::Vec { text: true, .. } => shape.flags.push("EMBER_RL_FIELD_ZERO"),
            TyKind::Vec { elem, text: false } => {
                let elem = *elem;
                shape.kind = "EMBER_RL_KIND_ARRAY";
                shape.flags.push("EMBER_RL_FIELD_ZERO");
                shape.element_size = Num::Size(Holder::Ty(elem));
                let nested = matches!(self.types.kind(elem), TyKind::Vec { text: false, .. }) && self.holds_objects(elem, &mut Vec::new());
                if let Some(own) = self.described_shape(elem, w) {
                    shape.element_kind = own.kind;
                    shape.inner = Some(self.shape_table(elem, w));
                } else if self.element_migrates(elem) {
                    // `[HR-13]` — the elements migrate through their container, by the element's
                    // schema, which the element's row names.
                    let (_, name) = element(self, elem);
                    let (inner, kind) = self.element_table(elem, w);
                    shape.element_kind = kind;
                    shape.element_type = name;
                    shape.inner = Some(inner);
                } else if nested {
                    // An `Array` of `Array`s: each element described by this image's own table.
                    shape.inner = Some(self.shape_table(elem, w));
                    shape.element_kind = "EMBER_RL_KIND_ARRAY";
                } else {
                    let (kind, name) = element(self, elem);
                    shape.element_kind = kind;
                    shape.element_type = name;
                }
            }
            TyKind::Enum(id) => {
                if self.types.option_niche(*id).is_some_and(|niche| matches!(self.types.kind(niche.payload), TyKind::Class(_))) {
                    shape.kind = "EMBER_RL_KIND_OPTION_HANDLE";
                } else if self.has_schema(ty) {
                    shape.kind = if self.types.enum_is_bare(*id) { "EMBER_RL_KIND_ENUM" } else { "EMBER_RL_KIND_PAYLOAD" };
                    shape.type_name = self.types.enum_def(*id).name.to_string();
                } else if self.holds_objects(ty, &mut Vec::new()) {
                    // `[HR-15]` — no schema, but this image describes what it holds, so the
                    // references in it are followed (a standard type's, a `Map`'s slots).
                    shape = self.contained_shape(ty, w);
                }
            }
            TyKind::Struct(id) => {
                if self.weak_inner_id(*id).is_some() {
                    shape.kind = "EMBER_RL_KIND_WEAK";
                } else if self.shared_inner_id(*id).is_some() {
                    shape.kind = "EMBER_RL_KIND_HANDLE";
                } else if let Some(element) = self.soa_element(ty) {
                    // `[HR-13]` — an `SoA[T]`'s elements migrate through it, column by column.
                    shape.kind = "EMBER_RL_KIND_SOA";
                    shape.element_type = self.types.struct_def(element).name.to_string();
                    shape.layout = Some(self.soa_table(ty, element, w));
                } else if self.has_schema(ty) {
                    shape.kind = "EMBER_RL_KIND_STRUCT";
                    shape.type_name = self.types.struct_def(*id).name.to_string();
                } else if self.holds_objects(ty, &mut Vec::new()) {
                    // `[HR-15]` — described by this image, so the references in it are followed.
                    shape = self.contained_shape(ty, w);
                }
            }
            // `[HR-13]` — a fixed array whose elements migrate holds them in place, each by the
            // element's row; any other is plain bytes, or opaque when it holds objects.
            TyKind::Array { elem, .. } if self.element_migrates(*elem) && self.described_shape_kind(*elem).is_none() => {
                let elem = *elem;
                shape.kind = "EMBER_RL_KIND_FIXED";
                shape.element_size = Num::Size(Holder::Ty(elem));
                let (_, name) = element(self, elem);
                let (inner, kind) = self.element_table(elem, w);
                shape.element_kind = kind;
                shape.element_type = name;
                shape.inner = Some(inner);
            }
            _ if self.holds_objects(ty, &mut Vec::new()) => shape.kind = "EMBER_RL_KIND_OPAQUE",
            _ => {}
        }
        shape
    }

    /// `described_shape`'s kind alone, which writes no table.
    fn described_shape_kind(&self, ty: Ty) -> Option<&'static str> {
        if !self.holds_objects(ty, &mut Vec::new()) {
            return None;
        }
        let boxed = |this: &Self, ty: Ty| match this.types.kind(ty) {
            TyKind::Struct(id) => this.box_inner_id(*id).filter(|inner| !matches!(this.types.kind(*inner), TyKind::Dyn { .. })),
            _ => None,
        };
        let pointee = match self.types.kind(ty) {
            TyKind::Struct(_) => boxed(self, ty),
            TyKind::Enum(id) => self.types.option_niche(*id).and_then(|niche| boxed(self, niche.payload)),
            _ => None,
        };
        match (self.types.kind(ty), pointee) {
            (_, Some(_)) => Some("EMBER_RL_KIND_BOX"),
            (TyKind::Tuple(_), None) => Some("EMBER_RL_KIND_TUPLE"),
            (TyKind::Fn { abi: None, .. }, None) => Some("EMBER_RL_KIND_CALLABLE"),
            _ => None,
        }
    }

    /// `[HR-15]` — what a tuple, a `Box` or a callable value holds, when it holds objects: `None`
    /// for every other type, which `field_shape` describes by its schema's name.
    fn described_shape(&mut self, ty: Ty, w: &mut dyn Writer) -> Option<FieldShape> {
        if !self.holds_objects(ty, &mut Vec::new()) {
            return None;
        }
        let boxed = |this: &Self, ty: Ty| match this.types.kind(ty) {
            TyKind::Struct(id) => this.box_inner_id(*id).filter(|inner| !matches!(this.types.kind(*inner), TyKind::Dyn { .. })),
            _ => None,
        };
        let pointee = match self.types.kind(ty) {
            TyKind::Struct(_) => boxed(self, ty),
            TyKind::Enum(id) => self.types.option_niche(*id).and_then(|niche| boxed(self, niche.payload)),
            _ => None,
        };
        let mut shape = FieldShape::plain(self.types.display(ty));
        match (self.types.kind(ty).clone(), pointee) {
            (_, Some(pointee)) => {
                shape.kind = "EMBER_RL_KIND_BOX";
                shape.inner = Some(self.shape_table(pointee, w));
            }
            (TyKind::Tuple(_), None) => {
                shape.kind = "EMBER_RL_KIND_TUPLE";
                shape.layout = Some(self.layout_table(ty, w));
            }
            (TyKind::Fn { abi: None, .. }, None) => shape.kind = "EMBER_RL_KIND_CALLABLE",
            _ => return None,
        }
        Some(shape)
    }

    /// `[HR-15]` — a value's description that names no type, for what a tuple, a `Box` or a
    /// closure's captured state holds (ODR-126): a struct is described by its fields here too, so
    /// the runtime reads the value by the layout of the image that made it, and an enum's variants
    /// are listed with their payloads. A type met again inside itself is `OPAQUE`: a reload that
    /// must move what it holds refuses ([HR-15a]).
    fn contained_shape(&mut self, ty: Ty, w: &mut dyn Writer) -> FieldShape {
        let mut shape = FieldShape::plain(self.types.display(ty));
        if !self.holds_objects(ty, &mut Vec::new()) {
            return shape;
        }
        shape.kind = "EMBER_RL_KIND_OPAQUE";
        if self.shapes_open.contains(&ty) {
            return shape;
        }
        if let Some(own) = self.described_shape(ty, w) {
            return own;
        }
        match self.types.kind(ty).clone() {
            TyKind::Class(_) => shape.kind = "EMBER_RL_KIND_HANDLE",
            // A niche `Option` is its payload's bits, null for `None`, which a handle's or a `Box`'s
            // walk passes over.
            TyKind::Enum(id) => match self.types.option_niche(id) {
                Some(niche) => {
                    let own = self.contained_shape(niche.payload, w);
                    if matches!(own.kind, "EMBER_RL_KIND_HANDLE" | "EMBER_RL_KIND_WEAK" | "EMBER_RL_KIND_BOX" | "EMBER_RL_KIND_OPTION_HANDLE") {
                        return FieldShape { type_name: shape.type_name, ..own };
                    }
                }
                None => {
                    shape.kind = "EMBER_RL_KIND_PAYLOAD";
                    shape.layout = Some(self.layout_table(ty, w));
                }
            },
            TyKind::Struct(id) if self.weak_inner_id(id).is_some() => shape.kind = "EMBER_RL_KIND_WEAK",
            TyKind::Struct(id) if self.shared_inner_id(id).is_some() => shape.kind = "EMBER_RL_KIND_HANDLE",
            TyKind::Struct(id) if self.box_inner_id(id).is_none() => {
                shape.kind = "EMBER_RL_KIND_TUPLE";
                shape.layout = Some(self.layout_table(ty, w));
            }
            TyKind::Vec { elem, text: false } => {
                let own = self.contained_shape(elem, w);
                if own.kind != "EMBER_RL_KIND_PLAIN" && own.kind != "EMBER_RL_KIND_OPAQUE" {
                    shape.kind = "EMBER_RL_KIND_ARRAY";
                    shape.element_kind = own.kind;
                    shape.element_size = Num::Size(Holder::Ty(elem));
                    shape.inner = Some(self.shape_table(elem, w));
                }
            }
            _ => {}
        }
        shape
    }

    /// `[HR-11]`, `[HR-13a]` — whether a class has a schema in this image: one the package declares,
    /// or a standard-library class holding the package's types (an ECS storage of a component),
    /// whose schema the instantiating package emits so its instances migrate and are followed; or
    /// `[HR-15]` one holding references (an ECS `World`'s storages), so a reload follows them.
    fn class_has_schema(&self, id: ClassId) -> bool {
        let def = self.types.class_def(id);
        self.reload.files.contains(&def.span.file)
            || (0..self.types.class_field_count(id)).filter_map(|index| self.types.class_field_at(id, index)).any(|field| {
                self.mentions_package(field.ty, &mut Vec::new())
                    || self.soa_element(field.ty).is_some()
                    || self.holds_objects(field.ty, &mut Vec::new())
            })
    }

    /// `[HR-13]` — whether a container's elements of type `ty` migrate through it: a struct or an
    /// enum with a schema, or a container of such.
    fn element_migrates(&self, ty: Ty) -> bool {
        match self.types.kind(ty) {
            TyKind::Struct(id) => self.structural_type_emitted(ty) && self.box_inner_id(*id).is_none() && self.has_schema(ty),
            TyKind::Enum(_) => self.has_schema(ty),
            TyKind::Vec { elem, text: false } | TyKind::Array { elem, .. } => self.element_migrates(*elem),
            _ => false,
        }
    }

    /// `[HR-13]` — the row describing one element of type `ty` as a field at offset 0 is
    /// (`field_shape`), with the alignment its container's buffer has; written once. With it, the
    /// row's kind.
    fn element_table(&mut self, ty: Ty, w: &mut dyn Writer) -> (String, &'static str) {
        if let Some(known) = self.elements.get(&ty) {
            return known.clone();
        }
        let mut shape = self.field_shape(ty, w);
        shape.align = Num::VecAlign(ty);
        let symbol = table(&format!("element_{}", self.elements.len()));
        let kind = shape.kind;
        let row = Row { name: String::new(), renamed: None, schema: Some(self.type_schema(ty, &mut Vec::new())), offset: Num::Zero, size: Num::Size(Holder::Ty(ty)), shape };
        w.field(&symbol, &row);
        let known = (symbol, kind);
        self.elements.insert(ty, known.clone());
        known
    }

    /// `[HR-13]` — an `SoA[T]` whose element `T` has a schema: `T`'s struct, which its columns
    /// split.
    fn soa_element(&self, ty: Ty) -> Option<StructId> {
        let TyKind::Struct(id) = self.types.kind(ty) else { return None };
        let (origin, args) = self.types.struct_def(*id).origin.as_ref()?;
        if origin.as_str() != "SoA" || args.len() != 1 || !self.has_schema(args[0]) {
            return None;
        }
        match self.types.kind(args[0]) {
            TyKind::Struct(element) => Some(*element),
            _ => None,
        }
    }

    /// `[HR-13]` — the description of an `SoA[T]`'s own struct, written once: each column's pointer
    /// (where it lives in the struct, the size and alignment of its values, and where its value
    /// sits in a whole `T`, `within`), then `len` and `cap`; and the function laying out its one
    /// block, which a reload rebuilding the block calls.
    fn soa_table(&mut self, ty: Ty, element: StructId, w: &mut dyn Writer) -> String {
        if let Some(symbol) = self.shapes.get(&(ty, true)) {
            return symbol.clone();
        }
        let TyKind::Struct(id) = *self.types.kind(ty) else { unreachable!("an `SoA` is a struct") };
        let element_ty = self.types.find(&TyKind::Struct(element)).expect("an `SoA`'s element is interned");
        let mut rows = Vec::new();
        for column in self.types.soa_columns(element) {
            let mut shape = self.field_shape(column.ty, w);
            shape.align = Num::Align(Holder::Ty(column.ty));
            shape.within = Num::Within(element_ty, column.path.clone());
            let pointer = format!("col_{}", column.name);
            rows.push(Row {
                name: pointer.clone(),
                renamed: None,
                schema: Some(self.type_schema(column.ty, &mut Vec::new())),
                offset: Num::Offset(Holder::Ty(ty), Member::Named(pointer)),
                size: Num::Size(Holder::Ty(column.ty)),
                shape,
            });
        }
        for count in ["len", "cap"] {
            rows.push(Row {
                name: count.to_string(),
                renamed: None,
                schema: None,
                offset: Num::Offset(Holder::Ty(ty), Member::Named(count.to_string())),
                size: Num::SizeT,
                shape: FieldShape::plain("usize".to_string()),
            });
        }
        let symbol = table(&format!("soa_{}", self.shapes.len()));
        let fields = format!("{symbol}_fields");
        w.fields(&fields, &rows);
        let columns = w.soa_columns(id);
        w.type_row(&symbol, &TypeRow {
            name: self.types.display(ty),
            kind: "EMBER_RL_TYPE_STRUCT",
            base: None,
            layout: Vec::new(),
            size: Num::Size(Holder::Ty(ty)),
            align: Num::Align(Holder::Ty(ty)),
            schema: self.type_schema(ty, &mut Vec::new()),
            methods: None,
            type_info: None,
            identity: None,
            fields: Some((rows.len(), Some(fields))),
            variants: None,
            init: None,
            migrate: None,
            old_type: None,
            drop_field: None,
            attributes: None,
            columns: Some(columns),
        });
        self.shapes.insert((ty, true), symbol.clone());
        symbol
    }

    /// The table describing one value of `ty` (`contained_shape`), written once.
    fn shape_table(&mut self, ty: Ty, w: &mut dyn Writer) -> String {
        if let Some(symbol) = self.shapes.get(&(ty, false)) {
            return symbol.clone();
        }
        let shape = self.contained_shape(ty, w);
        let symbol = table(&format!("value_{}", self.shapes.len()));
        let row = Row { name: String::new(), renamed: None, schema: Some(self.type_schema(ty, &mut Vec::new())), offset: Num::Zero, size: Num::Size(Holder::Ty(ty)), shape };
        w.field(&symbol, &row);
        self.shapes.insert((ty, false), symbol.clone());
        symbol
    }

    /// The table listing the fields of a tuple or a struct that hold objects, each described by
    /// `contained_shape`, written once: what a `TUPLE` value's walk reads.
    fn layout_table(&mut self, ty: Ty, w: &mut dyn Writer) -> String {
        if let Some(symbol) = self.shapes.get(&(ty, true)) {
            return symbol.clone();
        }
        // A struct's or a tuple's members; an enum's, by variant: its discriminant, and each payload
        // field's name and member.
        let mut groups: Vec<(Option<(String, u64)>, Vec<(String, Member, Ty)>)> = Vec::new();
        match self.types.kind(ty).clone() {
            TyKind::Tuple(items) => groups.push((None, items.into_iter().enumerate().map(|(index, item)| (format!("_{index}"), Member::Tuple(index), item)).collect())),
            TyKind::Struct(id) => groups.push((
                None,
                self.types.struct_def(id).fields.iter().enumerate().map(|(index, field)| (field.name.to_string(), Member::Field(index), field.ty)).collect(),
            )),
            TyKind::Enum(id) => {
                for (index, variant) in self.types.enum_def(id).variants.clone().into_iter().enumerate() {
                    let members = variant
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(field_index, field)| (field.name.to_string(), Member::Payload(index, field_index), field.ty))
                        .collect();
                    groups.push((Some((variant.name.to_string(), variant.discriminant as u64)), members));
                }
            }
            _ => {}
        }
        self.shapes_open.insert(ty);
        let mut described = Vec::new();
        for (index, (variant, members)) in groups.into_iter().enumerate() {
            let mut rows = Vec::new();
            for (name, member, item) in members {
                if self.is_void(item) || !self.holds_objects(item, &mut Vec::new()) {
                    continue;
                }
                let shape = self.contained_shape(item, w);
                rows.push(Row {
                    name,
                    renamed: None,
                    schema: Some(self.type_schema(item, &mut Vec::new())),
                    offset: Num::Offset(Holder::Ty(ty), member.clone()),
                    size: Num::MemberSize(Holder::Ty(ty), member),
                    shape,
                });
            }
            described.push((variant, index, rows));
        }
        // Named once what it holds is described: a table written for a member (a type met inside
        // this one) takes a name of its own first.
        let symbol = table(&format!("layout_{}", self.shapes.len()));
        let tables: Vec<(Option<(String, u64)>, String, Vec<Row>)> =
            described.into_iter().map(|(variant, index, rows)| (variant, format!("{symbol}_fields_{index}"), rows)).collect();
        self.shapes_open.remove(&ty);
        for (_, name, rows) in &tables {
            if !rows.is_empty() {
                w.fields(name, rows);
            }
        }
        let reference = |name: &String, rows: &Vec<Row>| if rows.is_empty() { None } else { Some(name.clone()) };
        let (fields, variants) = if tables.iter().any(|(variant, _, _)| variant.is_some()) {
            // `[HR-15]` — an enum with payloads: its tag as its one field, and each variant's payload,
            // which the walk reads for the variant the tag names.
            let tag = format!("{symbol}_tag");
            w.tag(&tag, &Row::tag(ty));
            let rows: Vec<VariantRow> = tables
                .iter()
                .map(|(variant, name, rows)| {
                    let (variant, discriminant) = variant.clone().expect("every group of an enum is a variant");
                    VariantRow { name: variant, renamed: None, discriminant, payload: None, fields: (rows.len(), reference(name, rows)) }
                })
                .collect();
            let variants = format!("{symbol}_variants");
            w.variants(&variants, &rows);
            (Some((1, Some(tag))), Some((tables.len(), Some(variants))))
        } else {
            let (_, name, rows) = &tables[0];
            (Some((rows.len(), reference(name, rows))), None)
        };
        w.type_row(&symbol, &TypeRow {
            name: self.types.display(ty),
            kind: "EMBER_RL_TYPE_STRUCT",
            base: None,
            layout: Vec::new(),
            size: Num::Size(Holder::Ty(ty)),
            align: Num::Align(Holder::Ty(ty)),
            schema: self.type_schema(ty, &mut Vec::new()),
            methods: None,
            type_info: None,
            identity: None,
            fields,
            variants,
            init: None,
            migrate: None,
            old_type: None,
            drop_field: None,
            attributes: None,
            columns: None,
        });
        self.shapes.insert((ty, true), symbol.clone());
        symbol
    }

    /// `[CLO-10]`, `[HR-15]` — what a closure's descriptor says its captured state holds: the symbol
    /// of the table describing it, `None` when it holds no object.
    pub fn captures(&mut self, env: Ty, w: &mut dyn Writer) -> Option<String> {
        if !self.holds_objects(env, &mut Vec::new()) {
            return None;
        }
        Some(self.shape_table(env, w))
    }

    /// `[HR-14]` — the package types that cross a foreign boundary by value: a parameter or the
    /// result of a function with a C ABI (an export, a foreign declaration, a C function pointer's
    /// type), and what such a value holds by value. The foreign side was compiled for their layout
    /// attributes, so a change to those is refused.
    fn abi_types(&self, bodies: &[Body]) -> BTreeSet<String> {
        let mut roots: Vec<Ty> = Vec::new();
        for body in bodies.iter().filter(|body| body.abi.is_some() || body.is_extern_declaration) {
            roots.push(body.return_ty());
            roots.extend((1..=body.arg_count).filter_map(|index| body.locals.get(index).map(|local| local.ty)));
        }
        for (_, kind) in self.types.all() {
            if let TyKind::Fn { abi: Some(_), params, ret, .. } = kind {
                roots.extend(params.iter().map(|param| param.ty));
                roots.push(*ret);
            }
        }
        let mut out = BTreeSet::new();
        let mut seen = BTreeSet::new();
        while let Some(ty) = roots.pop() {
            if !seen.insert(ty) {
                continue;
            }
            match self.types.kind(ty) {
                TyKind::Struct(id) if self.has_schema(ty) => {
                    out.insert(self.types.struct_def(*id).name.to_string());
                    roots.extend(self.types.struct_def(*id).fields.iter().map(|field| field.ty));
                }
                TyKind::Enum(id) if self.has_schema(ty) => {
                    out.insert(self.types.enum_def(*id).name.to_string());
                    roots.extend(self.types.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)));
                }
                TyKind::Array { elem, .. } => roots.push(*elem),
                TyKind::Tuple(items) => roots.extend(items.iter().copied()),
                // A parameter borrowed by default crosses by value, and one the foreign side reads
                // through a pointer relies on the layout as much.
                TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => roots.push(*inner),
                _ => {}
            }
        }
        out
    }

    /// `[HR-14]`, `[LAY-2]` — a type's layout attributes alone, hashed: 0 for none.
    fn attributes_hash(&self, ty: Ty) -> u64 {
        let repr = match self.types.kind(ty) {
            TyKind::Enum(id) => self.types.display(self.types.enum_def(*id).repr),
            _ => String::new(),
        };
        match layout_schema(self.types, ty) {
            None if repr.is_empty() => 0,
            layout => fnv(&[&repr, layout.as_deref().unwrap_or("")]),
        }
    }

    /// `[HR-11]` — every package type's schema: a class's and a struct's fields (name, renamed-from
    /// name, type, schema hash, offset, size, what it holds), a unit enum's variants; and, for a
    /// type with declared field defaults, the function that writes one into a new instance
    /// (`[HR-14]`). The rows of the manifest's type table, and every class's identity.
    fn type_schemas(&mut self, bodies: &[Body], w: &mut dyn Writer) -> (Vec<TypeRow>, Vec<String>) {
        let mut types = Vec::new();
        let mut identities = Vec::new();
        let abi = self.abi_types(bodies);
        let contained = self.contained_structs(&w.closure_envs(), &w.dyn_concretes());
        let foreign = self.pointed_structs(bodies);
        // Each class and struct with a schema: its fields, its mangled form (`[MNG-1]`), which its
        // tables are named by, what its sizes are measured on, and the type itself.
        let mut layouts: Vec<(Option<ClassId>, String, String, Vec<(String, Ty)>, Holder, Option<Ty>)> = Vec::new();
        let classes: Vec<ClassId> = self.types.runtime_classes().map(|(id, _)| id).collect();
        for id in classes {
            let def = self.types.class_def(id);
            let name = def.name.to_string();
            let mangled = self.types.mangled_class(id);
            identities.push(class_identity_symbol(&mangled));
            if !self.class_has_schema(id) {
                continue;
            }
            let fields: Vec<(String, Ty)> = (0..self.types.class_field_count(id))
                .filter_map(|index| self.types.class_field_at(id, index))
                .map(|field| (field.name.to_string(), field.ty))
                .collect();
            layouts.push((Some(id), name, mangled, fields, Holder::Object(id), self.types.find(&TyKind::Class(id))));
        }
        let structs: Vec<(StructId, String)> = self.types.structs().map(|(id, def)| (id, def.name.to_string())).collect();
        for (id, name) in structs {
            let Some(ty) = self.types.find(&TyKind::Struct(id)) else { continue };
            if !self.has_schema(ty) {
                continue;
            }
            let fields: Vec<(String, Ty)> = self.types.struct_def(id).fields.iter().map(|field| (field.name.to_string(), field.ty)).collect();
            layouts.push((None, name, self.types.mangled_struct(id), fields, Holder::Ty(ty), Some(ty)));
        }
        for (class, name, mangled, fields, holder, ty) in layouts {
            let table_name = table(&format!("fields_{mangled}"));
            let mut rows = Vec::new();
            let mut cases = Vec::new();
            for (index, (field, field_ty)) in fields.iter().enumerate() {
                let mut shape = self.field_shape(*field_ty, w);
                // A declared default's function, or `[HR-14]` a `Default` type's, which the checker
                // built only where the type is `Default`.
                if let Some(function) = self.reload.defaults.get(&(name.clone(), field.clone())).and_then(|function| self.function_symbol(function)) {
                    shape.flags.push("EMBER_RL_FIELD_DEFAULT");
                    cases.push(InitCase { index: index as u32, holder, member: Member::Field(index), ty: *field_ty, function });
                }
                rows.push(Row {
                    name: field.clone(),
                    renamed: self.reload.renamed.get(&(name.clone(), field.clone())).cloned(),
                    schema: Some(self.type_schema(*field_ty, &mut Vec::new())),
                    offset: Num::Offset(holder, Member::Field(index)),
                    size: Num::MemberSize(holder, Member::Field(index)),
                    shape,
                });
            }
            if !rows.is_empty() {
                w.fields(&table_name, &rows);
            }
            let init = if cases.is_empty() {
                None
            } else {
                let init_name = table(&format!("init_{mangled}"));
                w.init_fn(&init_name, &cases);
                Some(init_name)
            };
            let fields_ref = (!rows.is_empty()).then_some(table_name);
            let migrate = self.migrate_adapter(&name, &mangled, w);
            let old_type = self.reload.migrations.get(&name).map(|(_, old)| old.clone());
            // `[HR-2a]` — reclaim drops the fields an edit removed, one by one.
            let drops: Vec<DropCase> = fields
                .iter()
                .enumerate()
                .filter(|(_, (_, field_ty))| self.types.needs_drop(*field_ty))
                .map(|(index, (_, field_ty))| DropCase { index: index as u32, holder, member: Member::Field(index), ty: *field_ty })
                .collect();
            let drop_field = if drops.is_empty() {
                None
            } else {
                let drop_name = table(&format!("drop_{mangled}"));
                w.drop_fn(&drop_name, &drops);
                Some(drop_name)
            };
            let mut layout = match (class.is_none() && contained.contains(&name), foreign.contains(&name)) {
                (_, true) => vec!["EMBER_RL_LAYOUT_FOREIGN"],
                (true, false) => vec!["EMBER_RL_LAYOUT_CONTAINED"],
                (false, false) => Vec::new(),
            };
            if abi.contains(&name) {
                layout.push("EMBER_RL_LAYOUT_ABI");
            }
            let schema = ty.map_or(0, |ty| self.type_schema(ty, &mut Vec::new()));
            match class {
                Some(id) => {
                    let def = self.types.class_def(id);
                    types.push(TypeRow {
                        name: name.clone(),
                        kind: "EMBER_RL_TYPE_CLASS",
                        base: def.base.map(|base| self.types.class_def(base).name.to_string()),
                        layout: Vec::new(),
                        size: Num::Size(holder),
                        align: Num::Align(holder),
                        schema,
                        methods: Some(w.methods_hash(id)),
                        type_info: Some(ember_branding::type_info(&mangled)),
                        identity: Some(class_identity_symbol(&mangled)),
                        fields: Some((rows.len(), fields_ref)),
                        variants: None,
                        init,
                        migrate,
                        old_type,
                        drop_field,
                        attributes: None,
                        columns: None,
                    });
                }
                None => types.push(TypeRow {
                    name: name.clone(),
                    kind: "EMBER_RL_TYPE_STRUCT",
                    base: None,
                    layout,
                    size: Num::Size(holder),
                    align: Num::Align(holder),
                    schema,
                    methods: None,
                    type_info: None,
                    identity: None,
                    fields: Some((rows.len(), fields_ref)),
                    variants: None,
                    init,
                    migrate,
                    old_type,
                    drop_field,
                    attributes: Some(ty.map_or(0, |ty| self.attributes_hash(ty))),
                    columns: None,
                }),
            }
        }
        let enums: Vec<(EnumId, String)> = self.types.enums().map(|(id, def)| (id, def.name.to_string())).collect();
        for (id, name) in enums {
            let Some(ty) = self.types.find(&TyKind::Enum(id)) else { continue };
            let def = self.types.enum_def(id).clone();
            if !self.reload.files.contains(&def.span.file) && !self.has_schema(ty) {
                continue;
            }
            let mangled = self.types.mangled_enum(id);
            // `[HR-11]`, `[HR-15]` — a payload enum's schema: its tag as its one field, and each
            // variant's payload fields, which a reload walks for the variant the tag names.
            let payloads = !self.types.enum_is_bare(id) && self.has_schema(ty);
            let tag_name = table(&format!("tag_{mangled}"));
            if payloads {
                w.tag(&tag_name, &Row::tag(ty));
            }
            let mut rows = Vec::new();
            // `[HR-14]` — a payload's fields migrate by the rows a type's take: each one's default
            // (its type's `Default`, `init_field`) and its drop (`drop_field`) are numbered over
            // every variant's payload fields in turn.
            let mut cases = Vec::new();
            let mut drops = Vec::new();
            let mut numbered = 0u32;
            for (variant_index, variant) in def.variants.iter().enumerate() {
                let renamed = self.reload.renamed.get(&(name.clone(), variant.name.to_string())).cloned();
                let payload: Vec<String> = variant.fields.iter().map(|field| self.type_schema(field.ty, &mut Vec::new()).to_string()).collect();
                let mut fields = Vec::new();
                let held: Vec<(usize, ember_types::FieldDef)> =
                    variant.fields.iter().enumerate().filter(|(_, field)| payloads && !self.is_void(field.ty)).map(|(index, field)| (index, field.clone())).collect();
                for (field_index, field) in &held {
                    let mut shape = self.field_shape(field.ty, w);
                    let member = Member::Payload(variant_index, *field_index);
                    let key = (name.clone(), format!("{}.{}", variant.name, field.name));
                    if let Some(function) = self.reload.defaults.get(&key).and_then(|function| self.function_symbol(function)) {
                        shape.flags.push("EMBER_RL_FIELD_DEFAULT");
                        cases.push(InitCase { index: numbered, holder: Holder::Ty(ty), member: member.clone(), ty: field.ty, function });
                    }
                    if self.types.needs_drop(field.ty) {
                        drops.push(DropCase { index: numbered, holder: Holder::Ty(ty), member: member.clone(), ty: field.ty });
                    }
                    numbered += 1;
                    fields.push(Row {
                        name: field.name.to_string(),
                        renamed: self.reload.renamed.get(&key).cloned(),
                        schema: Some(self.type_schema(field.ty, &mut Vec::new())),
                        offset: Num::Offset(Holder::Ty(ty), member.clone()),
                        size: Num::MemberSize(Holder::Ty(ty), member),
                        shape,
                    });
                }
                let payload_table = table(&format!("payload_{mangled}_{}", ember_branding::mangled_path(variant.name.as_str())));
                if !fields.is_empty() {
                    w.fields(&payload_table, &fields);
                }
                rows.push(VariantRow {
                    name: variant.name.to_string(),
                    renamed,
                    discriminant: variant.discriminant as u64,
                    payload: Some(fnv(&payload.iter().map(String::as_str).collect::<Vec<_>>())),
                    fields: (fields.len(), (!fields.is_empty()).then_some(payload_table)),
                });
            }
            let table_name = table(&format!("variants_{mangled}"));
            if !rows.is_empty() {
                w.variants(&table_name, &rows);
            }
            let init = if cases.is_empty() {
                None
            } else {
                let init_name = table(&format!("init_{mangled}"));
                w.init_fn(&init_name, &cases);
                Some(init_name)
            };
            let drop_field = if drops.is_empty() {
                None
            } else {
                let drop_name = table(&format!("drop_{mangled}"));
                w.drop_fn(&drop_name, &drops);
                Some(drop_name)
            };
            let mut layout = if self.types.enum_is_bare(id) { vec!["EMBER_RL_LAYOUT_UNIT"] } else { Vec::new() };
            for (flag, holds) in [("EMBER_RL_LAYOUT_ABI", abi.contains(&name)), ("EMBER_RL_LAYOUT_CONTAINED", contained.contains(&name))] {
                if holds {
                    layout.push(flag);
                }
            }
            types.push(TypeRow {
                name: name.clone(),
                kind: "EMBER_RL_TYPE_ENUM",
                base: None,
                layout,
                size: Num::Size(Holder::Ty(ty)),
                align: Num::Align(Holder::Ty(ty)),
                schema: self.type_schema(ty, &mut Vec::new()),
                methods: None,
                type_info: None,
                identity: None,
                fields: payloads.then(|| (1, Some(tag_name))),
                variants: Some((rows.len(), (!rows.is_empty()).then_some(table_name))),
                init,
                migrate: None,
                old_type: None,
                drop_field,
                attributes: Some(self.attributes_hash(ty)),
                columns: None,
            });
        }
        (types, identities)
    }

    /// A function the driver wrote for the reload (a field's default, a migration's wrapper),
    /// found by its name in the package: its symbol and its return type.
    fn function(&self, name: &str) -> Option<(String, Ty)> {
        let dotted = format!(".{name}");
        self.bodies
            .iter()
            .find(|(body_name, _, _)| body_name == name || body_name.ends_with(&dotted))
            .map(|(_, symbol, ret)| (symbol.clone(), *ret))
    }

    fn function_symbol(&self, name: &str) -> Option<String> {
        self.function(name).map(|(symbol, _)| symbol)
    }

    /// `[HR-16]` — the adapter between the runtime and a type's `migrate_from`: the runtime builds
    /// the old value as `Old<Name>` from the old instance, field by name, and passes its address;
    /// the driver's wrapper (`fn(old: ref Old<Name>, report: mut String) -> Option[<Name>]`) calls
    /// `migrate_from` and gives the new value, or `None` with the `ReloadError` written into the
    /// report. `None` for a type without `migrate_from`.
    fn migrate_adapter(&mut self, name: &str, mangled: &str, w: &mut dyn Writer) -> Option<String> {
        let (wrapper, _) = self.reload.migrations.get(name)?;
        let (symbol, ret) = self.function(wrapper)?;
        let TyKind::Enum(option) = *self.types.kind(ret) else { return None };
        let some = self.types.enum_def(option).variants.iter().position(|variant| variant.name.as_str() == "Some")?;
        let adapter = table(&format!("migrate_{mangled}"));
        w.migrate_fn(&adapter, &symbol, ret, option, some);
        Some(adapter)
    }

    /// `[HR-8]` — the manifest: every reloadable function by mangled name, every `@noreload` one
    /// with its body's hash, every static, every type's schema and every identity, with the
    /// protocol's version, in the image's own section. The tables it points at are the runtime's
    /// to fill.
    pub fn manifest(&mut self, bodies: &[Body], has_entry: bool, w: &mut dyn Writer) {
        // `[HR-14]` — each range type's test, which a field's row names.
        let ranges: Vec<(RangeId, RangeDef)> = self.types.ranges().map(|(id, def)| (id, def.clone())).collect();
        for (id, def) in ranges {
            if range_testable(self.types.kind(def.repr), &def) {
                w.range_test(&range_test_symbol(id), id);
            }
        }
        let address_taken = address_taken_symbols(bodies);
        let mut fns = Vec::new();
        for body in bodies {
            let Some(&index) = self.slots.get(&body.symbol) else { continue };
            if fns.len() != index || !self.reload.reloadable.contains(&body.symbol) {
                continue;
            }
            let mut flags = vec!["EMBER_RL_FN_RELOADABLE"];
            if address_taken.contains(body.symbol.as_str()) || body.abi.is_some() {
                flags.push("EMBER_RL_FN_ADDRESS");
            }
            // `[HR-21]` — a host reaches an export through its permanent thunk.
            if body.abi.is_some() {
                flags.push("EMBER_RL_FN_EXPORT");
            }
            fns.push(FnRow::Reloadable {
                symbol: body.symbol.clone(),
                descriptor: w.descriptor(&body.symbol),
                signature: w.signature_hash(body),
                body_hash: self.reload.noreload.get(&body.symbol).copied().unwrap_or(0),
                flags,
            });
        }
        let mut fixed: Vec<(String, u64)> = self.reload.noreload.iter().map(|(symbol, hash)| (symbol.clone(), *hash)).collect();
        fixed.sort_by_key(|(symbol, _)| self.slots.get(symbol).copied());
        for (symbol, body_hash) in fixed {
            if self.slots.get(&symbol) != Some(&fns.len()) {
                continue;
            }
            fns.push(FnRow::NoReload { symbol, body_hash });
        }
        // `[HR-15]` — what each stored static's value holds, as a field at offset 0.
        let stored = self.statics.clone();
        let mut shapes = Vec::new();
        for (index, (name, ty, _)) in stored.iter().enumerate() {
            let shape = self.field_shape(*ty, w);
            let symbol = table(&format!("shape_{index}"));
            let row = Row { name: name.clone(), renamed: None, schema: Some(self.type_schema(*ty, &mut Vec::new())), offset: Num::Zero, size: Num::Size(Holder::Ty(*ty)), shape };
            w.field(&symbol, &row);
            shapes.push(symbol);
        }
        let mut statics: Vec<StaticRow> = stored
            .iter()
            .zip(shapes)
            .map(|((name, ty, init), shape)| {
                let (initialiser, _, reinit) = self.reload.statics.get(name).copied().unwrap_or((0, 0, false));
                let mut flags = Vec::new();
                if reinit {
                    flags.push("EMBER_RL_STATIC_REINIT");
                }
                if init.is_none() {
                    flags.push("EMBER_RL_STATIC_CONSTANT");
                }
                StaticRow::Stored {
                    name: name.clone(),
                    ty: *ty,
                    schema: self.type_schema(*ty, &mut Vec::new()),
                    initialiser,
                    flags,
                    constant: init.is_none(),
                    shape,
                }
            })
            .collect();
        for (name, (initialiser, declared, reinit)) in &self.reload.statics {
            if stored.iter().any(|(known, _, _)| known == name) {
                continue;
            }
            // `[HR-17a]` — a folded static's code holds its value and nothing else: a value that
            // changed with its initialiser's text unchanged (a function it calls was edited) is a
            // changed initialiser too, refused unless the static is `@reinit_on_reload`.
            let value = self.types.folded_statics().iter().find(|(known, _)| known.as_str() == name).map(|(_, value)| value.as_str());
            let initialiser = value.map_or(*initialiser, |value| fnv(&[&initialiser.to_string(), value]));
            statics.push(StaticRow::Folded { name: name.clone(), declared: *declared, initialiser, reinit: *reinit });
        }
        let (types, mut identities) = self.type_schemas(bodies, w);
        identities.extend(w.interface_identities());
        let manifest = Manifest {
            mode: self.reload.mode,
            package: self.reload.package.clone(),
            instances: self.reload.instances,
            fns,
            statics,
            types,
            identities,
            headers: self.reload.headers.iter().map(|(name, hash)| (name.clone(), hash.clone())).collect(),
            has_entry,
        };
        w.manifest(&manifest);
    }

    // -- frames (`[HR-3b]`, A1, ODR-133) ------------------------------------------------------------
    //
    // Each function of a reloadable image with instances keeps a frame record while it runs (pushed
    // on entry, popped on return): which function it is and which of its calls it is in. For each
    // call, the image lists the package types whose layout the rest of the function depends on once
    // the call returns, so a reload at `std.hot.checkpoint()` applies a layout change the old code
    // still running below the checkpoint no longer uses, and refuses one it does.
    //
    // What the rest of a function depends on: every type whose fields its old code projects (a
    // class's through a handle, a struct's or a tuple's by value), every value it builds, holds,
    // passes or gets back by value (with what that value holds by value), every element type it
    // indexes, every class it allocates, and what the functions it calls directly depend on: those
    // run old code too. A call through a thunk runs the new image's code, which depends on nothing
    // old; a call through a function value may reach any old code, so it depends on what every old
    // body does.

    /// The type of `place` in `body`, a projection that does not apply leaving it alone.
    fn place_ty(&self, place: &Place, body: &Body) -> Ty {
        let (mut ty, mut variant) = (body.local(place.local).ty, None::<usize>);
        for projection in &place.projection {
            let at = ty;
            let mut next_variant = None;
            ty = match (projection, self.types.kind(at)) {
                (Projection::Downcast(chosen), TyKind::Enum(_)) => {
                    next_variant = Some(*chosen);
                    at
                }
                (Projection::Field(index), TyKind::Enum(id)) => match variant {
                    None => at,
                    Some(variant) => self.types.enum_def(*id).variants[variant].fields.get(*index).map(|f| f.ty).unwrap_or(at),
                },
                (Projection::Field(index), TyKind::Struct(id)) => self.types.struct_def(*id).fields.get(*index).map(|f| f.ty).unwrap_or(at),
                (Projection::Field(index), TyKind::Class(id)) => self.types.class_field_at(*id, *index).map(|field| field.ty).unwrap_or(at),
                (Projection::Field(index), TyKind::Tuple(items)) => items.get(*index).copied().unwrap_or(at),
                (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } | TyKind::Span { elem, .. }) => *elem,
                (Projection::Index(_) | Projection::ConstIndex(_), TyKind::Ptr { inner, .. }) => *inner,
                (Projection::Field(1), TyKind::Str) => self.usize_ty,
                (Projection::Field(index), TyKind::Vec { .. } | TyKind::Span { .. }) => {
                    if *index == 0 {
                        next_variant = variant;
                        at
                    } else {
                        self.usize_ty
                    }
                }
                (Projection::Deref, TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }) => *inner,
                _ => {
                    next_variant = variant;
                    at
                }
            };
            variant = next_variant;
        }
        ty
    }

    fn element_of(&self, ty: Ty) -> Ty {
        match self.types.kind(ty) {
            TyKind::Vec { elem, .. } => *elem,
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => self.element_of(*inner),
            _ => ty,
        }
    }

    /// The package types whose layout a value of `ty` held by value depends on: its own and what it
    /// holds by value. A handle, a pointer and a buffer's elements are behind an address.
    fn by_value_layouts(&self, ty: Ty, out: &mut BTreeSet<String>, seen: &mut Vec<Ty>) {
        if seen.contains(&ty) {
            return;
        }
        seen.push(ty);
        match self.types.kind(ty).clone() {
            TyKind::Struct(id) if self.weak_inner_id(id).is_none() && self.shared_inner_id(id).is_none() && self.box_inner_id(id).is_none() => {
                let def = self.types.struct_def(id);
                out.insert(def.name.to_string());
                for field in def.fields.iter().map(|field| field.ty).collect::<Vec<_>>() {
                    self.by_value_layouts(field, out, seen);
                }
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(id).clone();
                out.insert(def.name.to_string());
                for field in def.variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)) {
                    self.by_value_layouts(field, out, seen);
                }
            }
            TyKind::Tuple(items) => {
                for item in items {
                    self.by_value_layouts(item, out, seen);
                }
            }
            TyKind::Array { elem, .. } => self.by_value_layouts(elem, out, seen),
            _ => {}
        }
        seen.pop();
    }

    /// What reading or writing `place` depends on: each step's projection into a value's layout.
    fn place_layouts(&self, place: &Place, body: &Body, out: &mut BTreeSet<String>) {
        let mut prefix = Place::local(place.local);
        for step in &place.projection {
            let ty = self.place_ty(&prefix, body);
            match (step, self.types.kind(ty)) {
                (Projection::Field(_) | Projection::Downcast(_), TyKind::Class(id)) => {
                    out.insert(self.types.class_def(*id).name.to_string());
                }
                (Projection::Field(_) | Projection::Downcast(_), _) => self.by_value_layouts(ty, out, &mut Vec::new()),
                (Projection::Index(_) | Projection::ConstIndex(_) | Projection::Column(_), _) => {
                    let elem = self.element_of(ty);
                    self.by_value_layouts(elem, out, &mut Vec::new());
                }
                (Projection::Deref, _) => {}
            }
            prefix.projection.push(step.clone());
        }
    }

    fn operand_layouts(&self, operand: &Operand, body: &Body, out: &mut BTreeSet<String>) {
        if let Operand::Copy(place) | Operand::Move(place) = operand {
            self.place_layouts(place, body, out);
            self.by_value_layouts(self.place_ty(place, body), out, &mut Vec::new());
        }
    }

    /// What one block's code depends on, the bodies it calls directly (old code after a swap), and
    /// whether it calls through a function value.
    fn block_layouts(&self, body: &Body, block: usize, out: &mut BTreeSet<String>, direct: &mut BTreeSet<String>, indirect: &mut bool) {
        for stmt in &body.blocks[block].stmts {
            match &stmt.kind {
                StmtKind::Assign { place, rvalue } => {
                    self.place_layouts(place, body, out);
                    self.by_value_layouts(self.place_ty(place, body), out, &mut Vec::new());
                    match rvalue {
                        Rvalue::Use(operand) | Rvalue::UnaryOp { operand, .. } | Rvalue::Cast { operand, .. } => {
                            self.operand_layouts(operand, body, out);
                        }
                        Rvalue::Repeat { value, .. } => self.operand_layouts(value, body, out),
                        Rvalue::BinaryOp { lhs, rhs, .. } => {
                            self.operand_layouts(lhs, body, out);
                            self.operand_layouts(rhs, body, out);
                        }
                        Rvalue::Aggregate { operands, .. } => {
                            for operand in operands {
                                self.operand_layouts(operand, body, out);
                            }
                        }
                        Rvalue::Discriminant(place) => {
                            self.place_layouts(place, body, out);
                            self.by_value_layouts(self.place_ty(place, body), out, &mut Vec::new());
                        }
                        Rvalue::Ref { place, .. } => self.place_layouts(place, body, out),
                    }
                }
                StmtKind::CheckedBinaryOp { dest, overflow, lhs, rhs, .. } => {
                    for place in [dest, overflow] {
                        self.place_layouts(place, body, out);
                    }
                    self.operand_layouts(lhs, body, out);
                    self.operand_layouts(rhs, body, out);
                }
                // Dropping a value runs its old drop glue; a handle's runs the object's own.
                StmtKind::Drop { place, .. } => {
                    self.place_layouts(place, body, out);
                    self.by_value_layouts(self.place_ty(place, body), out, &mut Vec::new());
                }
                // An access word lives in the object.
                StmtKind::BeginAccess { place, .. }
                | StmtKind::BeginAccessTransfer { place, .. }
                | StmtKind::EndAccess { place, .. }
                | StmtKind::EndAccessTransfer { place, .. }
                | StmtKind::LendAccess { place, .. } => {
                    self.place_layouts(place, body, out);
                    if let TyKind::Class(id) = self.types.kind(body.locals[place.local.0 as usize].ty) {
                        out.insert(self.types.class_def(*id).name.to_string());
                    }
                }
                StmtKind::StorageLive(_) | StmtKind::StorageDead(_) | StmtKind::Nop => {}
            }
        }
        match &body.blocks[block].terminator {
            Terminator::Call { func, args, dest, .. } => {
                for arg in args {
                    self.operand_layouts(arg, body, out);
                }
                self.place_layouts(dest, body, out);
                self.by_value_layouts(self.place_ty(dest, body), out, &mut Vec::new());
                match func {
                    FuncRef::Direct { symbol, .. } if !self.is_reloadable(symbol) => {
                        direct.insert(symbol.clone());
                    }
                    FuncRef::Indirect { operand, .. } => {
                        self.operand_layouts(operand, body, out);
                        *indirect = true;
                    }
                    FuncRef::Builtin { which: Builtin::ClassNew { class_id, .. }, .. } => {
                        out.insert(self.types.class_def(*class_id).name.to_string());
                    }
                    _ => {}
                }
            }
            Terminator::SwitchInt { discr, .. } => self.operand_layouts(discr, body, out),
            Terminator::Assert { cond, .. } => self.operand_layouts(cond, body, out),
            Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable => {}
        }
    }

    /// What each body of the image depends on, with what the bodies it calls directly depend on;
    /// and what any old body may depend on, for a call through a function value.
    fn plan_frame_layouts(&mut self, bodies: &[Body]) {
        let mut own: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>, bool)> = BTreeMap::new();
        for body in bodies {
            let (mut uses, mut direct, mut indirect) = (BTreeSet::new(), BTreeSet::new(), false);
            for block in 0..body.blocks.len() {
                self.block_layouts(body, block, &mut uses, &mut direct, &mut indirect);
            }
            own.insert(body.symbol.clone(), (uses, direct, indirect));
        }
        // A direct call reaches only bodies that are not reloadable, so their own uses together are
        // what any old code may depend on.
        let old: BTreeSet<String> = own.iter().filter(|(symbol, _)| !self.is_reloadable(symbol)).flat_map(|(_, (uses, _, _))| uses.iter().cloned()).collect();
        let mut closed: BTreeMap<String, BTreeSet<String>> = own
            .iter()
            .map(|(symbol, (uses, _, indirect))| {
                let mut set = uses.clone();
                if *indirect {
                    set.extend(old.iter().cloned());
                }
                (symbol.clone(), set)
            })
            .collect();
        loop {
            let mut changed = false;
            for (symbol, (_, direct, _)) in &own {
                let reached: BTreeSet<String> = direct.iter().filter_map(|callee| closed.get(callee)).flatten().cloned().collect();
                let set = closed.get_mut(symbol).expect("every body has a set");
                for name in reached {
                    changed |= set.insert(name);
                }
            }
            if !changed {
                break;
            }
        }
        self.frame_layouts = closed;
        self.old_layouts = old;
    }

    /// What the rest of `body` depends on once the call ending `block` returns: the call's own
    /// result, and every block reachable from `next`.
    fn continuation_layouts(&self, body: &Body, block: usize, next: usize) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        if let Terminator::Call { dest, .. } = &body.blocks[block].terminator {
            self.place_layouts(dest, body, &mut out);
            self.by_value_layouts(self.place_ty(dest, body), &mut out, &mut Vec::new());
        }
        let mut seen = BTreeSet::new();
        let mut pending = vec![next];
        let (mut direct, mut indirect) = (BTreeSet::new(), false);
        while let Some(at) = pending.pop() {
            if at >= body.blocks.len() || !seen.insert(at) {
                continue;
            }
            self.block_layouts(body, at, &mut out, &mut direct, &mut indirect);
            pending.extend(frame_successors(&body.blocks[at].terminator));
        }
        for callee in &direct {
            if let Some(set) = self.frame_layouts.get(callee) {
                out.extend(set.iter().cloned());
            }
        }
        if indirect {
            out.extend(self.old_layouts.iter().cloned());
        }
        out
    }

    /// `body`'s frame description: its name and, for each call, the source line and the types the
    /// rest of the body depends on. Its call sites are numbered in block order.
    pub fn frame_info(&self, body: &Body, map: &SourceMap) -> FrameInfo {
        let mut sites = Vec::new();
        let mut by_block = BTreeMap::new();
        for (block, data) in body.blocks.iter().enumerate() {
            let Terminator::Call { next, .. } = &data.terminator else { continue };
            let types = self.continuation_layouts(body, block, next.0 as usize);
            let line = if data.terminator_span.is_dummy() || map.get(data.terminator_span.file).is_none() {
                0
            } else {
                map.file(data.terminator_span.file).line_col(data.terminator_span.start).line
            };
            by_block.insert(block, sites.len() as u32);
            sites.push((line, types.into_iter().collect()));
        }
        FrameInfo { symbol: frame_symbol(&body.symbol), function: body.name.clone(), sites, by_block }
    }

    /// Whether `ty` is a struct the image defines (not a view, an alias or an erased one).
    fn structural_type_emitted(&self, ty: Ty) -> bool {
        let TyKind::Struct(id) = self.types.kind(ty) else { return false };
        self.box_inner_id(*id).is_none() && self.shared_inner_id(*id).is_none() && self.weak_inner_id(*id).is_none()
    }
}

fn frame_successors(terminator: &Terminator) -> Vec<usize> {
    match terminator {
        Terminator::Goto(next) | Terminator::Call { next, .. } | Terminator::Assert { next, .. } => vec![next.0 as usize],
        Terminator::SwitchInt { targets, otherwise, .. } => targets.iter().map(|(_, target)| target.0 as usize).chain([otherwise.0 as usize]).collect(),
        Terminator::Return | Terminator::Unreachable => Vec::new(),
    }
}
