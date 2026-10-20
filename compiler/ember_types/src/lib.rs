//! The type interner, the type table, and layout computation.
//!
//! Spec: Part IV (the type system) and Part XVIII §4.3 (the `TypeInfo` table).
//!
//! `[TYP-1]` — every concrete type has a compile-time-known `size`, `align`,
//! `is_copy`, `is_send`, `is_sync`, `needs_drop`, `is_view` and `has_niche`.
//! `[TYP-11]` — the default struct layout is C-compatible, deliberately, so
//! that every plain struct can cross an FFI boundary.

use std::collections::{HashMap, HashSet};
use std::fmt;

use ember_span::{Span, Symbol};

mod const_norm;
pub use const_norm::{ConstAtom, ConstError, ConstNorm, ConstOp};

/// An interned type. Equality is identity.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ty(u32);

/// `[IMP-7]` — the canonical identity exchanged by semantic passes for one
/// interned type. `Ty` remains the compact representation; this name makes the
/// contract explicit without creating a second type universe.
pub type TypeIdentity = Ty;

/// A type cannot enter an interface artifact until type checking has resolved
/// it to a real semantic identity. This deliberately rejects the diagnostic-
/// only placeholder kinds instead of serializing their presentation text.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CanonicalTypeError {
    InferenceVariable,
    IntegerLiteral,
    FloatLiteral,
    Error,
}

impl fmt::Display for CanonicalTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CanonicalTypeError::InferenceVariable => "unresolved inference variable",
            CanonicalTypeError::IntegerLiteral => "unresolved integer literal type",
            CanonicalTypeError::FloatLiteral => "unresolved float literal type",
            CanonicalTypeError::Error => "error type",
        })
    }
}

impl std::error::Error for CanonicalTypeError {}

/// Index of a user-declared struct in the type table.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct StructId(pub u32);

/// Index of a user-declared class in the type table.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ClassId(pub u32);

/// Index of a user-declared enum in the type table.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EnumId(pub u32);

/// Index of a user-declared range type (`[RNG-1]`) in the type table.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RangeId(pub u32);

/// An inference variable, resolved by `ember_typeck`'s union-find.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct InferId(pub u32);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum IntTy {
    I8,
    I16,
    I32,
    I64,
    I128,
    /// G8-4 — the 256-bit count (`[STD-19]`): what `len()` gives for a
    /// `..=` range of 128-bit numbers and `enumerate` numbers a 128-bit
    /// range with. Only a count: it adds, subtracts, compares, prints and
    /// converts with `as`; no ranges of it.
    I256,
    Isize,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum UintTy {
    U8,
    U16,
    U32,
    U64,
    U128,
    /// G8-4 — the unsigned 256-bit count (`[STD-19]`): what `len()` gives
    /// for a `..=` range of 128-bit numbers. Only a count, as `i256` is.
    U256,
    Usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FloatTy {
    F16,
    F32,
    F64,
}

/// `[FN-6a]` — one callable parameter's ordinary Ember passing mode. This is
/// part of canonical function-type identity and compile-time metadata, never
/// a runtime field or a separate ownership model.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FnParamMode {
    Borrow,
    Mut,
    Owned,
}

/// One mode-bearing parameter in `fn(A) -> R`, `fn(mut A) -> R`, or
/// `fn(owned A) -> R`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FnParam {
    pub ty: Ty,
    pub mode: FnParamMode,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TyKind {
    Bool,
    Char,
    Int(IntTy),
    Uint(UintTy),
    Float(FloatTy),
    /// The unit type, value `()`. Zero-sized.
    Void,
    /// `!` — coerces to every type (`[TYP-4]` table).
    Never,
    /// A view over UTF-8 bytes with a region; `Span[u8]` known to be valid.
    Str,
    /// A borrowed pointer to NUL-terminated C bytes, distinct from UTF-8 `str`.
    CStr,
    /// `Span[T]` and `MutSpan[T]` (Part VII §7): pointer plus length, with a
    /// compile-time region. Part IV §1 puts them in the **View** category
    /// beside `ref T` and `str`, which is why they are a kind here rather than
    /// a library struct over a raw pointer: a struct holding a `*T` carries no
    /// region, so `[TYP-15]` could not see it and `[UNS-4]`'s "no two views
    /// that are simultaneously live may overlap unless both are shared" would
    /// have nothing to hold.
    ///
    /// `[SPN-3]` — `Span[T]` is `Copy`; `MutSpan[T]` is move-only and
    /// reborrowable.
    Span { elem: Ty, mutable: bool },
    /// An incomplete C struct declared by `extern type`. It has identity but
    /// no value layout; only a pointer to it may cross the boundary.
    Opaque(Symbol),
    Struct(StructId),
    /// A class value is a non-null counted handle. Its source-level type is
    /// pointer-sized, but its allocation begins with an `ember_obj_header`
    /// and is described by the corresponding `ClassDef`/`TypeInfo` record.
    /// The distinction from `Ptr` is semantic: copying a class handle retains
    /// and dropping one releases; it is not a raw-pointer operation.
    Class(ClassId),
    /// `[OBJ-2]` — a class handle erased to one `dyn`-compatible interface.
    ///
    /// Unlike `Dyn`, this remains a sized counted handle: its concrete class
    /// object retains the runtime header, and `[DSP-3]` obtains the interface
    /// table from that header's type information. The interface identity is
    /// retained so method lookup and coercion remain statically checked.
    ClassInterface(Symbol),
    Enum(EnumId),
    /// `[RNG-1]` — a **nominal** numeric type over a representation,
    /// restricted to a range. `[RNG-2]` makes two of them distinct types even
    /// when representation and range are identical, which is why this carries
    /// an id and not the range itself: interning by structure would make
    /// `Roughness` and `Metallic` the same type, and telling them apart is the
    /// entire point.
    Range(RangeId),
    Tuple(Vec<Ty>),
    Ref { mutable: bool, inner: Ty },
    Ptr { mutable: bool, inner: Ty },
    Array { elem: Ty, len: u64 },
    /// `[TYP-41]` — `[T; n]` whose length names a `const` parameter, in
    /// generic code; a length that is a number is `Array`.
    ArrayN { elem: Ty, len: ConstNorm },
    /// `[TYP-41]` — a const generic argument where a type argument would be:
    /// its value's normal form, a number once every parameter is known.
    Const(ConstNorm),
    /// `Array[T]` — a growable, heap-allocated sequence. Part XX.1 makes this
    /// a compiler-known type until Phase 2's generics let the standard library
    /// write it in Ember. `String` is this with `u8` elements and `text` set:
    /// the same buffer, owned and dropped alike, but a type of its own, known
    /// to be UTF-8 (`[TXT-1]`) and handled as text (D-201). `Array[u8]` is
    /// an `Array` like any other.
    Vec { elem: Ty, text: bool },
    /// A mode-bearing callable type. `latebound` is a compile-time callable
    /// boundary fact; it is not runtime metadata or an ABI field.
    /// `stateless` (ODR-102): the type of a function, or of a lambda that
    /// captures nothing, used as a value — `Copy`, nothing to drop. Written
    /// `fn(A) -> R` outside a parameter is an owned callable value
    /// (`[CLO-3]`): it may hold captured state, so it moves and drops. Both
    /// have one representation, and the first converts to the second.
    /// `is_unsafe` (ODR-115): `unsafe fn(A) -> R`, whose call needs `unsafe`; the same value.
    Fn { abi: Option<Symbol>, latebound: bool, stateless: bool, is_unsafe: bool, params: Vec<FnParam>, ret: Ty },
    /// `[TYP-22]` — an unsized interface value. The interface identities are
    /// retained in declaration order; runtime fat-pointer/vtable lowering is
    /// a later backend slice, so this kind must never acquire a guessed
    /// ordinary value layout.
    Dyn { interfaces: Vec<Symbol> },
    /// `[TYP-16]` — a generic parameter, opaque while the body that declares
    /// it is checked. `[TYP-17]` allows only what its bounds provide, so the
    /// bound list travels with the declaration rather than with the type.
    /// Monomorphisation substitutes it away before MIR.
    Param { index: u32, name: Symbol },
    /// `[IFC-4]` — an associated type: `Iterator`'s `Item`, standing for
    /// whatever the implementing type declared it to be. Resolved once the
    /// receiver is concrete.
    Assoc { name: Symbol },
    /// An unsolved inference variable.
    Infer(InferId),
    /// `[LEX-16]` — an integer literal with no suffix, awaiting context.
    IntLit,
    /// `[LEX-17]` — a float literal with no suffix, awaiting context.
    FloatLit,
    /// A type that could not be determined. Absorbs errors so that one bad
    /// annotation does not produce a diagnostic per use.
    Error,
}

/// How a type is laid out in memory.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Layout {
    pub size: u64,
    pub align: u64,
    /// Byte offset of each field, in declaration order. Empty for non-structs.
    pub field_offsets: Vec<u64>,
}

/// `[IMP-7]` — the canonical target layout attached to a `TypeIdentity`.
/// `Layout` remains the representation so existing layout and ABI consumers
/// cannot drift onto a parallel descriptor.
pub type LayoutDescriptor = Layout;

/// `[LAY-2]` — a struct's or an enum's layout attributes: `@packed` (no padding) and
/// `@align(N)` (the alignment raised to `N`, so the size is a multiple of it). `align` is 1
/// when none is written.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LayoutAttrs {
    pub packed: bool,
    pub align: u64,
}

impl LayoutAttrs {
    pub const NONE: LayoutAttrs = LayoutAttrs { packed: false, align: 1 };

    pub fn is_none(self) -> bool {
        self == LayoutAttrs::NONE
    }
}

impl Layout {
    pub fn scalar(size: u64) -> Layout {
        Layout { size, align: size.max(1), field_offsets: Vec::new() }
    }

    pub const ZERO: Layout = Layout { size: 0, align: 1, field_offsets: Vec::new() };
}

/// `[MOD-2]`, `[MOD-7]` — how far a field is visible. Kept here rather than
/// taken from the AST so that `ember_types` does not depend on it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FieldVis {
    /// The default: readable and writable from the declaring module only.
    Private,
    /// `pub(package)`.
    Package,
    /// `pub`.
    Public,
}

#[derive(Clone, Debug)]
pub struct FieldDef {
    pub name: Symbol,
    pub ty: Ty,
    pub span: Span,
    /// The exact source range of the field's type annotation. Diagnostics use
    /// this rather than replacing a whole declaration when a type has an
    /// unambiguous machine-applicable repair.
    pub ty_span: Span,
    /// `[STR-2]` — whether the declaration supplied a default.
    pub has_default: bool,
    /// The source `let` modifier. `[CLS-9]` applies it to class fields.
    pub is_let: bool,
    /// `[MOD-7]` — declared `pub(read)` or `pub(package, read)`: readable
    /// wherever its visibility allows, writable only from the declaring
    /// module.
    pub read_only_outside: bool,
    /// `[MOD-2]` — "All items are private to their module unless `pub`."
    pub vis: FieldVis,
}

#[derive(Clone, Debug)]
pub struct StructDef {
    pub name: Symbol,
    pub fields: Vec<FieldDef>,
    pub span: Span,
    /// `[STR-3]` — `Copy` is never implicit; it comes from `@derive(Copy)`.
    pub derives_copy: bool,
    /// `[STR-3]` — a `drop` method or a `Drop` field makes the type move-only.
    pub has_drop: bool,
    /// Whether ordinary structural drop descends into the fields after any
    /// user destructor. This is true for source structs. Compiler-known
    /// storage wrappers such as `[ARN-8]`'s `MaybeUninit[T]` set it false:
    /// their field describes layout, not an initialized owned `T`.
    pub drops_fields: bool,
    /// `[TYP-16]` — when this struct is one instantiation of a generic, the
    /// generic it came from and the arguments it was built with. That is what
    /// lets `Buffer[T]` unify with `Buffer[i32]`.
    pub origin: Option<(Symbol, Vec<Ty>)>,
    /// `[MOD-7]` — the module that declared it, so that "outside its module"
    /// is a comparison rather than a guess at the qualified name's shape.
    pub declaring_module: usize,
}

impl StructDef {
    pub fn field(&self, name: Symbol) -> Option<(usize, &FieldDef)> {
        self.fields.iter().enumerate().find(|(_, f)| f.name == name)
    }
}

/// Whether a class may be extended or instantiated. This mirrors the AST
/// openness without making `ember_types` depend on the parser crate.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ClassOpenness {
    Final,
    Open,
    Abstract,
}

/// The compiler-side identity of a class declaration. The value type for a
/// class is its counted handle; the fields here describe the heap object that
/// handle addresses. Generic class instantiation remains a later consumer,
/// but `origin` makes the identity model ready for the same nominal recipe
/// used by generic structs.
#[derive(Clone, Debug)]
pub struct ClassDef {
    pub name: Symbol,
    pub fields: Vec<FieldDef>,
    pub span: Span,
    /// `[THR-1]`: handles may cross threads only for an explicitly `@sync` class.
    pub is_sync: bool,
    pub openness: ClassOpenness,
    pub base: Option<ClassId>,
    pub has_drop: bool,
    pub origin: Option<(Symbol, Vec<Ty>)>,
    pub declaring_module: usize,
}

impl ClassDef {
    pub fn field(&self, name: Symbol) -> Option<(usize, &FieldDef)> {
        self.fields.iter().enumerate().find(|(_, f)| f.name == name)
    }
}

/// One variant of an enum. A variant with no fields is a unit variant; its
/// value is its discriminant and nothing else.
#[derive(Clone, Debug)]
pub struct VariantDef {
    pub name: Symbol,
    /// The payload, in declaration order. `[ENM-1]` allows these to be named,
    /// so they are `FieldDef`s and not bare types; an unnamed one is `_0`,
    /// `_1`, … so that positional patterns and named patterns are the same
    /// lookup.
    pub fields: Vec<FieldDef>,
    /// `[TYP-12]` — the tag value. Assigned in declaration order unless the
    /// declaration gave one.
    pub discriminant: i128,
    pub span: Span,
}

impl VariantDef {
    pub fn field(&self, name: Symbol) -> Option<(usize, &FieldDef)> {
        self.fields.iter().enumerate().find(|(_, f)| f.name == name)
    }
}

#[derive(Clone, Debug)]
pub struct EnumDef {
    pub name: Symbol,
    pub variants: Vec<VariantDef>,
    pub span: Span,
    /// The generic recipe and concrete arguments that materialised this enum.
    /// A generic enum instance remains an ordinary nominal enum after this
    /// substitution, just as a generic struct instance does.
    pub origin: Option<(Symbol, Vec<Ty>)>,
    /// `[TYP-12]` — the integer type the tag is stored in.
    pub repr: Ty,
    /// Whether `@repr` was written. `[TYP-12]` requires it for FFI.
    pub repr_is_explicit: bool,
    /// `[ENM-4]` — a payload enum is `Copy` only via `@derive(Copy)`.
    pub derives_copy: bool,
    pub has_drop: bool,
    /// `[FFI-8]` — `@non_exhaustive`, a C enum: a unit-only `@repr` enum any value of whose
    /// integer type is a value of the enum (ODR-137).
    pub open: bool,
}

impl EnumDef {
    /// `[ENM-3]` — an enum whose every variant is a unit variant. These are
    /// just their discriminant: `Copy`, and convertible with `as`.
    pub fn is_unit_only(&self) -> bool {
        self.variants.iter().all(|v| v.fields.is_empty())
    }

    pub fn variant(&self, name: Symbol) -> Option<(usize, &VariantDef)> {
        self.variants.iter().enumerate().find(|(_, v)| v.name == name)
    }
}

/// One endpoint of a range type's `in` clause (`[RNG-1]`).
///
/// The two arms are kept apart rather than both stored as `f64` because
/// `[RNG-6]` makes float ranges obey strict IEEE semantics and `[TYP-8]`'s
/// integer overflow reasoning needs exact 128-bit integers: a `u64` endpoint
/// does not survive a round trip through `f64`.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Bound {
    Int(i128),
    Float(f64),
}

impl Bound {
    /// Ordering within one representation. Two bounds of different arms never
    /// arise: `[RNG-1]` requires both endpoints to be constants of the
    /// representation type, and a representation is integer or float.
    pub fn le(self, other: Bound) -> bool {
        match (self, other) {
            (Bound::Int(a), Bound::Int(b)) => a <= b,
            // `[RNG-6]` — NaN is in no range, and a NaN endpoint is rejected
            // by `[RNG-1]`'s constant check before it reaches here.
            (Bound::Float(a), Bound::Float(b)) => a <= b,
            _ => false,
        }
    }
}

/// `[RNG-1]` — a nominal numeric type over `repr`, restricted to `lo..hi`.
#[derive(Clone, Debug)]
pub struct RangeDef {
    pub name: Symbol,
    /// The representation. `[RNG-8]`: a range type erases to this at every
    /// coercion site and crosses an FFI boundary as this.
    pub repr: Ty,
    pub lo: Bound,
    pub hi: Bound,
    /// `..=` rather than `..`.
    pub inclusive: bool,
    pub span: Span,
}

impl RangeDef {
    /// Whether a constant lies in the declared range. `[RNG-3]`: a constant in
    /// range emits no check; one outside is `E2211`.
    pub fn contains(&self, v: Bound) -> bool {
        if !self.lo.le(v) {
            return false;
        }
        if self.inclusive { v.le(self.hi) } else { v.le(self.hi) && v != self.hi }
    }

    /// Whether every value of `other` is a value of this range — `[RNG-10]`(d),
    /// "a value whose `[RNG-4]` range is contained in the target's".
    pub fn contains_range(&self, other: &RangeDef) -> bool {
        if self.repr != other.repr {
            return false;
        }
        let lo_ok = self.lo.le(other.lo);
        let hi_ok = match (self.inclusive, other.inclusive) {
            // `a ..= b` contains `c .. d` iff d <= b + 1, which is not
            // expressible without knowing the representation's successor. The
            // conservative answer is the correct one: only compare like with
            // like, and say no otherwise.
            (true, true) | (false, false) => other.hi.le(self.hi),
            (true, false) => other.hi.le(self.hi),
            (false, true) => false,
        };
        lo_ok && hi_ok
    }
}

/// ODR-111 — a `Send` or `Sync` a program wrote by hand (`unsafe extend W[T] implements Send`): the
/// type, by its generic's name or its own, with what each of its arguments must be for it to hold.
#[derive(Clone, Debug)]
pub struct ManualMarker {
    pub name: Symbol,
    pub send: bool,
    pub sync: bool,
    pub args: Vec<MarkerArg>,
}

/// What a by-hand marker asks of one argument: this exact type, or any type with these markers.
#[derive(Clone, Copy, Debug)]
pub enum MarkerArg {
    Exact(Ty),
    Any { send: bool, sync: bool },
}

/// The interner and type table for one compilation.
pub struct TypeTable {
    kinds: Vec<TyKind>,
    lookup: HashMap<TyKind, Ty>,
    structs: Vec<StructDef>,
    classes: Vec<ClassDef>,
    enums: Vec<EnumDef>,
    ranges: Vec<RangeDef>,
    /// D-232 — how a compiler-built generic struct (`Cell_i32`) is written
    /// (`Cell[i32]`). Kept apart from `origin`, which also decides how a
    /// type resolves.
    written_as: HashMap<StructId, (Symbol, Vec<Ty>)>,
    /// `[TYP-32]` — how a struct the compiler made for a `some I` return type is shown to the
    /// user: as the `some I` it stands for, not by its own name.
    shown_as: HashMap<StructId, String>,
    /// `[TYP-32]` — whether a `some` wrapper shows (`Display`) and debug-prints (`Debug`) as the
    /// value it holds, by its interface.
    some_prints: HashMap<StructId, (bool, bool)>,
    /// `[TYP-35]` — for a generic instance, the arguments of the parameters no
    /// field names (its phantom parameters).
    phantoms: HashMap<Ty, Vec<Ty>>,
    /// `[ARN-11]` — the structs declared `@derive(Zeroable)`, by declared
    /// name (a generic struct's instances share its name as their origin).
    zeroable_derived: HashSet<Symbol>,
    /// `[SOA-1]` — the fields marked `@soa(flatten)`: (declared struct name, field name).
    soa_flattened: HashSet<(Symbol, Symbol)>,
    /// `[LAY-2]` — each struct's and enum's written layout attributes, by declared name (a
    /// generic one's instances share it as their origin).
    layout_attrs: HashMap<Symbol, LayoutAttrs>,
    /// `[THR-6]` — the structs and classes declared `@must_drop`, by declared
    /// name (a generic one's instances share it as their origin).
    must_drop_declared: HashSet<Symbol>,
    /// ADR-186, `[STR-5]` — the function writing a type's written `Debug`
    /// text (`std.fmt.write_debug`'s instance for it, `(value, mut out:
    /// String)`), by symbol: formatting the type, alone or inside another
    /// value, calls it in place of the field-wise text.
    debug_writers: HashMap<Ty, String>,
    /// ODR-111 — each `Send` or `Sync` a program wrote by hand (`unsafe extend`).
    manual_markers: Vec<ManualMarker>,
    /// `[CORO-3]` — each `gen fn`'s frame struct, by declared name (a generic
    /// one's instances share it as their origin): how each parameter field's
    /// parameter was passed (`settle_coroutine_frame`).
    coroutine_frames: HashMap<Symbol, Vec<FrameParam>>,
    next_infer: u32,
    /// Pointer width of the target, in bytes. 8 for every v1 target.
    pointer_size: u64,
    /// `[CT-5]`, `[STA-3]` — the statics whose values the compile-time
    /// evaluator computed, placed in the image as constant data: each name,
    /// type and value (in the evaluator's text).
    constant_statics: Vec<(Symbol, Ty, String)>,
    /// `[FFI-26]` — each `@export_table` of the program.
    export_tables: Vec<ExportTable>,
}

/// `[FFI-26]` — an `@export_table("Name", protocol=N)`: the struct exported as a constant table of
/// function pointers under the C symbol `name_vN` (`symbol`), its protocol version `N`, and the C
/// symbol of the function each field holds, in field order.
#[derive(Clone, Debug)]
pub struct ExportTable {
    pub ty: Ty,
    pub name: String,
    pub protocol: u32,
    pub entries: Vec<String>,
}

impl ExportTable {
    /// `[ABI-1]`, `[FFI-26]` (ODR-129) — the table's C symbol, its protocol version in its name
    /// (`RvScriptApi_v3`): a host built for another version does not find it.
    pub fn symbol(&self) -> String {
        format!("{}_v{}", self.name, self.protocol)
    }
}

/// `[CORO-1]`, `[CORO-6]` — how a `gen fn`'s parameter was passed, which says
/// how its frame holds it: an `owned` one by value; a borrowed one by value
/// when the caller passes it as a copy, else as the `ref` the caller passes;
/// a `mut` one as its `ref mut` (a `MutSpan` as itself).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FrameParam {
    Owned,
    Borrowed,
    Mut,
}

/// Well-known types, interned once at construction so that common lookups are
/// a field read rather than a hash.
pub struct CommonTypes {
    pub bool_: Ty,
    pub char_: Ty,
    pub i8: Ty,
    pub i16: Ty,
    pub i32: Ty,
    pub i64: Ty,
    pub i128: Ty,
    pub i256: Ty,
    pub u256: Ty,
    pub isize: Ty,
    pub u8: Ty,
    pub u16: Ty,
    pub u32: Ty,
    pub u64: Ty,
    pub u128: Ty,
    pub usize: Ty,
    pub f16: Ty,
    pub f32: Ty,
    pub f64: Ty,
    pub void: Ty,
    pub never: Ty,
    pub str_: Ty,
    pub cstr: Ty,
    /// `String` (`[TXT-1]`): `Vec { elem: u8, text: true }`.
    pub string: Ty,
    pub int_lit: Ty,
    pub float_lit: Ty,
    pub error: Ty,
    /// `Self` inside an `interface` declaration, where the implementing type
    /// is not yet known (Part IV §8: `interface Clone: fn clone(self) -> Self`).
    ///
    /// A `Param` rather than a kind of its own, because that is exactly what
    /// it is — a type the body may not look inside, substituted when the
    /// interface is used. `SELF_PARAM` keeps it out of any real parameter
    /// list's index space.
    pub self_ty: Ty,
}

/// The index `CommonTypes::self_ty` occupies. No generic parameter list
/// reaches it, so a substitution over one cannot collide with `Self`.
pub const SELF_PARAM: u32 = u32::MAX;

/// The `declaring_module` of a type the compiler makes (`Box`, `Cell`,
/// `Shared`, the arenas) rather than a program declares.
pub const COMPILER_MODULE: usize = usize::MAX;

/// `[TYP-13]` — how an `Option` keeps `None` in its payload's niche: the
/// variants' indices, and the payload type, which is the `Option`'s layout.
#[derive(Clone, Copy, Debug)]
pub struct Niche {
    pub none: usize,
    pub some: usize,
    pub payload: Ty,
}

impl TypeTable {
    /// `[CT-5]` — a static placed in the image with this value.
    pub fn add_constant_static(&mut self, name: Symbol, ty: Ty, value: String) {
        self.constant_statics.push((name, ty, value));
    }

    pub fn constant_statics(&self) -> &[(Symbol, Ty, String)] {
        &self.constant_statics
    }

    /// `[FFI-26]` — an exported table of function pointers.
    pub fn add_export_table(&mut self, table: ExportTable) {
        self.export_tables.push(table);
    }

    pub fn export_tables(&self) -> &[ExportTable] {
        &self.export_tables
    }

    pub fn new() -> (TypeTable, CommonTypes) {
        let mut table = TypeTable {
            kinds: Vec::new(),
            lookup: HashMap::new(),
            structs: Vec::new(),
            classes: Vec::new(),
            enums: Vec::new(),
            ranges: Vec::new(),
            written_as: HashMap::new(),
            shown_as: HashMap::new(),
            some_prints: HashMap::new(),
            phantoms: HashMap::new(),
            zeroable_derived: HashSet::new(),
            soa_flattened: HashSet::new(),
            layout_attrs: HashMap::new(),
            must_drop_declared: HashSet::new(),
            debug_writers: HashMap::new(),
            manual_markers: Vec::new(),
            coroutine_frames: HashMap::new(),
            next_infer: 0,
            constant_statics: Vec::new(),
            export_tables: Vec::new(),
            pointer_size: 8,
        };
        let u8_ty = table.intern(TyKind::Uint(UintTy::U8));
        let common = CommonTypes {
            bool_: table.intern(TyKind::Bool),
            char_: table.intern(TyKind::Char),
            i8: table.intern(TyKind::Int(IntTy::I8)),
            i16: table.intern(TyKind::Int(IntTy::I16)),
            i32: table.intern(TyKind::Int(IntTy::I32)),
            i64: table.intern(TyKind::Int(IntTy::I64)),
            i128: table.intern(TyKind::Int(IntTy::I128)),
            i256: table.intern(TyKind::Int(IntTy::I256)),
            u256: table.intern(TyKind::Uint(UintTy::U256)),
            isize: table.intern(TyKind::Int(IntTy::Isize)),
            u8: u8_ty,
            u16: table.intern(TyKind::Uint(UintTy::U16)),
            u32: table.intern(TyKind::Uint(UintTy::U32)),
            u64: table.intern(TyKind::Uint(UintTy::U64)),
            u128: table.intern(TyKind::Uint(UintTy::U128)),
            usize: table.intern(TyKind::Uint(UintTy::Usize)),
            f16: table.intern(TyKind::Float(FloatTy::F16)),
            f32: table.intern(TyKind::Float(FloatTy::F32)),
            f64: table.intern(TyKind::Float(FloatTy::F64)),
            void: table.intern(TyKind::Void),
            never: table.intern(TyKind::Never),
            str_: table.intern(TyKind::Str),
            cstr: table.intern(TyKind::CStr),
            string: table.intern(TyKind::Vec { elem: u8_ty, text: true }),
            self_ty: table.intern(TyKind::Param {
                index: SELF_PARAM,
                name: Symbol::intern("Self"),
            }),
            int_lit: table.intern(TyKind::IntLit),
            float_lit: table.intern(TyKind::FloatLit),
            error: table.intern(TyKind::Error),
        };
        (table, common)
    }

    /// The type of this shape if some earlier stage interned it.
    pub fn find(&self, kind: &TyKind) -> Option<Ty> {
        self.lookup.get(kind).copied()
    }

    pub fn intern(&mut self, kind: TyKind) -> Ty {
        if let Some(&ty) = self.lookup.get(&kind) {
            return ty;
        }
        let ty = Ty(self.kinds.len() as u32);
        self.kinds.push(kind.clone());
        self.lookup.insert(kind, ty);
        ty
    }

    pub fn kind(&self, ty: Ty) -> &TyKind {
        &self.kinds[ty.0 as usize]
    }

    /// The direct primitive values that can supply an ordinary interface
    /// implementation through an `extend` declaration. This deliberately
    /// excludes view, raw, unsized, and compiler-generated storage types.
    pub fn is_primitive_scalar(&self, ty: Ty) -> bool {
        matches!(
            self.kind(ty),
            TyKind::Bool
                | TyKind::Char
                | TyKind::Int(_)
                | TyKind::Uint(_)
                | TyKind::Float(_)
        )
    }

    /// Whether a type mentions any generic parameter, and so still has to be
    /// substituted before it means anything at run time.
    pub fn is_generic(&self, ty: Ty) -> bool {
        self.is_generic_in(ty, &mut HashSet::new())
    }

    /// `seen` holds the nominal types already being examined. A type may
    /// reach itself through an owning indirection (`struct Tree: kids:
    /// Array[Tree]`); meeting it again adds nothing, and without the set the
    /// walk never ends (D-182).
    fn is_generic_in(&self, ty: Ty, seen: &mut HashSet<Ty>) -> bool {
        match self.kind(ty) {
            TyKind::Param { .. } | TyKind::ArrayN { .. } => true,
            TyKind::Const(value) => value.mentions_params(),
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => {
                self.is_generic_in(*inner, seen)
            }
            TyKind::Array { elem, .. } | TyKind::Vec { elem, .. } => self.is_generic_in(*elem, seen),
            TyKind::Span { elem, .. } => self.is_generic_in(*elem, seen),
            TyKind::Tuple(items) => items.iter().any(|&t| self.is_generic_in(t, seen)),
            TyKind::Fn { params, ret, .. } => {
                params.iter().any(|param| self.is_generic_in(param.ty, seen))
                    || self.is_generic_in(*ret, seen)
            }
            // An instance over a parameter is generic whether or not a field holds it: a phantom
            // argument (`Not[T]`, `[TYP-35]`) and `SoA[T]`'s element (its columns are known only
            // per instance) are arguments no field shows.
            TyKind::Struct(id) => {
                seen.insert(ty)
                    && (self
                        .struct_def(*id)
                        .fields
                        .iter()
                        .any(|field| self.is_generic_in(field.ty, seen))
                        || self.struct_def(*id).origin.as_ref().is_some_and(|(_, args)| args.iter().any(|&arg| self.is_generic_in(arg, seen))))
            }
            TyKind::Class(id) => {
                seen.insert(ty)
                    && (self
                        .class_def(*id)
                        .fields
                        .iter()
                        .any(|field| self.is_generic_in(field.ty, seen))
                        || self.class_def(*id).origin.as_ref().is_some_and(|(_, args)| args.iter().any(|&arg| self.is_generic_in(arg, seen))))
            }
            // `Option[U]`, `Result[T, F]`: a payload may hold a parameter.
            TyKind::Enum(id) => {
                seen.insert(ty)
                    && (self
                        .enum_def(*id)
                        .variants
                        .iter()
                        .any(|variant| variant.fields.iter().any(|field| self.is_generic_in(field.ty, seen)))
                        || self.enum_def(*id).origin.as_ref().is_some_and(|(_, args)| args.iter().any(|&arg| self.is_generic_in(arg, seen))))
            }
            _ => false,
        }
    }

    /// Whether `ty` contains an unsized component in a by-value position.
    /// References/pointers are the intentional indirection boundary for
    /// `[TYP-22]`; owning wrappers such as `Box[dyn I]` are represented by a
    /// sized compiler-known struct and therefore do not recurse through that
    /// wrapper here.
    pub fn has_unsized_by_value(&self, ty: Ty) -> bool {
        self.has_unsized_by_value_in(ty, &mut HashSet::new())
    }

    /// `seen` breaks cycles through owning indirections, as in
    /// `is_generic_in` (D-182).
    fn has_unsized_by_value_in(&self, ty: Ty, seen: &mut HashSet<Ty>) -> bool {
        match self.kind(ty) {
            TyKind::Dyn { .. } | TyKind::Opaque(_) => true,
            TyKind::Ref { .. } | TyKind::Ptr { .. } => false,
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. }
            | TyKind::Vec { elem, .. }
            | TyKind::Span { elem, .. } => self.has_unsized_by_value_in(*elem, seen),
            TyKind::Tuple(items) => {
                items.iter().any(|&item| self.has_unsized_by_value_in(item, seen))
            }
            TyKind::Fn { params, ret, .. } => {
                params.iter().any(|param| self.has_unsized_by_value_in(param.ty, seen))
                    || self.has_unsized_by_value_in(*ret, seen)
            }
            TyKind::Struct(id) => {
                seen.insert(ty)
                    && self
                        .struct_def(*id)
                        .fields
                        .iter()
                        .any(|field| self.has_unsized_by_value_in(field.ty, seen))
            }
            TyKind::Class(id) => {
                seen.insert(ty)
                    && self
                        .class_def(*id)
                        .fields
                        .iter()
                        .any(|field| self.has_unsized_by_value_in(field.ty, seen))
            }
            TyKind::Enum(id) => {
                seen.insert(ty)
                    && self
                        .enum_def(*id)
                        .variants
                        .iter()
                        .flat_map(|variant| variant.fields.iter())
                        .any(|field| self.has_unsized_by_value_in(field.ty, seen))
            }
            _ => false,
        }
    }

    /// `[TYP-16]` — replace each parameter with the type it was instantiated
    /// with. `args` is indexed by the parameter's position.
    /// Replace `Self` (`CommonTypes::self_ty`) with a concrete type.
    ///
    /// Part IV §8's interfaces are declared over `Self`; a use of one — a
    /// method call through a bound, a default body copied into an impl —
    /// substitutes the implementing type for it. Kept apart from
    /// `substitute`, which maps by parameter *index*: `SELF_PARAM` is
    /// `u32::MAX` precisely so no real index list reaches it, and indexing an
    /// argument vector with it would be a bug rather than a lookup.
    pub fn substitute_self(&mut self, ty: Ty, concrete: Ty) -> Ty {
        match self.kind(ty).clone() {
            TyKind::Param { index, .. } if index == SELF_PARAM => concrete,
            TyKind::Ref { mutable, inner } => {
                let inner = self.substitute_self(inner, concrete);
                self.intern(TyKind::Ref { mutable, inner })
            }
            TyKind::Ptr { mutable, inner } => {
                let inner = self.substitute_self(inner, concrete);
                self.intern(TyKind::Ptr { mutable, inner })
            }
            TyKind::Array { elem, len } => {
                let elem = self.substitute_self(elem, concrete);
                self.intern(TyKind::Array { elem, len })
            }
            TyKind::Vec { elem, text } => {
                let elem = self.substitute_self(elem, concrete);
                self.intern(TyKind::Vec { elem, text })
            }
            TyKind::Tuple(items) => {
                let items: Vec<Ty> =
                    items.iter().map(|&t| self.substitute_self(t, concrete)).collect();
                self.intern(TyKind::Tuple(items))
            }
            TyKind::Fn { abi, latebound, stateless, is_unsafe, params, ret } => {
                let params = params
                    .iter()
                    .map(|param| FnParam {
                        ty: self.substitute_self(param.ty, concrete),
                        mode: param.mode,
                    })
                    .collect();
                let ret = self.substitute_self(ret, concrete);
                self.intern(TyKind::Fn { abi, latebound, stateless, is_unsafe, params, ret })
            }
            _ => ty,
        }
    }

    /// `[TYP-41]` — `[elem; len]`: an `Array` when the length is a number
    /// (`None` when it is not one an array can have), else an `ArrayN`.
    pub fn array_of(&mut self, elem: Ty, len: ConstNorm) -> Option<Ty> {
        match len.as_constant() {
            Some(value) => Some(self.intern(TyKind::Array { elem, len: u64::try_from(value).ok()? })),
            None => Some(self.intern(TyKind::ArrayN { elem, len })),
        }
    }

    /// `[TYP-41]` — a const argument's form with each parameter replaced by
    /// the argument `args` gives it, where that is a const argument.
    pub fn substitute_const(&self, norm: &ConstNorm, args: &[Ty]) -> Result<ConstNorm, ConstError> {
        norm.substitute(&|index| match args.get(index as usize).map(|&arg| self.kind(arg)) {
            Some(TyKind::Const(value)) => Some(value.clone()),
            Some(TyKind::Param { index, name }) => Some(ConstNorm::param(*index, *name)),
            _ => None,
        })
    }

    pub fn substitute(&mut self, ty: Ty, args: &[Ty]) -> Ty {
        match self.kind(ty).clone() {
            // A length that does not compute was reported where the argument
            // is written (`[TYP-41]` rule 1); here it stays as it was.
            TyKind::ArrayN { elem, len } => {
                let elem = self.substitute(elem, args);
                match self.substitute_const(&len, args) {
                    Ok(len) => self.array_of(elem, len).unwrap_or(ty),
                    Err(_) => ty,
                }
            }
            TyKind::Const(value) => match self.substitute_const(&value, args) {
                Ok(value) => self.intern(TyKind::Const(value)),
                Err(_) => ty,
            },
            TyKind::Param { index, .. } => {
                args.get(index as usize).copied().unwrap_or(ty)
            }
            TyKind::Ref { mutable, inner } => {
                let inner = self.substitute(inner, args);
                self.intern(TyKind::Ref { mutable, inner })
            }
            TyKind::Ptr { mutable, inner } => {
                let inner = self.substitute(inner, args);
                self.intern(TyKind::Ptr { mutable, inner })
            }
            TyKind::Array { elem, len } => {
                let elem = self.substitute(elem, args);
                self.intern(TyKind::Array { elem, len })
            }
            TyKind::Vec { elem, text } => {
                let elem = self.substitute(elem, args);
                self.intern(TyKind::Vec { elem, text })
            }
            TyKind::Span { elem, mutable } => {
                let elem = self.substitute(elem, args);
                self.intern(TyKind::Span { elem, mutable })
            }
            TyKind::Tuple(items) => {
                let items: Vec<Ty> = items.iter().map(|&t| self.substitute(t, args)).collect();
                self.intern(TyKind::Tuple(items))
            }
            TyKind::Fn { abi, latebound, stateless, is_unsafe, params, ret } => {
                let params = params
                    .iter()
                    .map(|param| FnParam {
                        ty: self.substitute(param.ty, args),
                        mode: param.mode,
                    })
                    .collect();
                let ret = self.substitute(ret, args);
                self.intern(TyKind::Fn { abi, latebound, stateless, is_unsafe, params, ret })
            }
            _ => ty,
        }
    }

    /// `[TYP-18]` — match a declared parameter type against the type an
    /// argument actually has, filling in `args` where a parameter is met.
    /// Returns false only on a shape mismatch the caller should report.
    pub fn unify(&self, declared: Ty, actual: Ty, args: &mut Vec<Option<Ty>>) -> bool {
        self.unify_with_fixed(declared, actual, args, 0)
    }

    /// Call-site unification with an explicit prefix. A written type argument
    /// is already fixed by the programmer, so inference must not reject an
    /// untyped literal merely because it has not yet reached the later
    /// coercion check. The substituted argument check remains authoritative.
    pub fn unify_with_fixed(
        &self,
        declared: Ty,
        actual: Ty,
        args: &mut Vec<Option<Ty>>,
        fixed: usize,
    ) -> bool {
        match (self.kind(declared).clone(), self.kind(actual).clone()) {
            (TyKind::Param { index, .. }, _) => {
                let slot = index as usize;
                if slot >= args.len() {
                    return false;
                }
                if slot < fixed {
                    return true;
                }
                match args[slot] {
                    // A parameter met twice must be met with the same type.
                    Some(existing) => existing == actual,
                    None => {
                        args[slot] = Some(actual);
                        true
                    }
                }
            }
            (TyKind::Ref { inner: a, .. }, TyKind::Ref { inner: b, .. })
            | (TyKind::Ptr { inner: a, .. }, TyKind::Ptr { inner: b, .. })
            | (TyKind::Array { elem: a, .. }, TyKind::Array { elem: b, .. }) => {
                self.unify_with_fixed(a, b, args, fixed)
            }
            // `[TYP-41]` — a length that is one `const` parameter is solved by
            // the length met; any other is checked once substituted.
            (TyKind::ArrayN { elem: a, len }, TyKind::Array { elem: b, len: met }) => {
                if let Some(slot) = len.as_param().map(|index| index as usize)
                    && slot < args.len()
                    && slot >= fixed
                {
                    // The table cannot make the constant here, so the slot
                    // holds the array met, whose length the checker reads.
                    match args[slot] {
                        Some(existing) => {
                            let agrees = match self.kind(existing) {
                                TyKind::Const(value) => value.as_constant() == Some(i128::from(met)),
                                TyKind::Array { len: other, .. } => *other == met,
                                _ => false,
                            };
                            if !agrees {
                                return false;
                            }
                        }
                        None => args[slot] = Some(actual),
                    }
                }
                self.unify_with_fixed(a, b, args, fixed)
            }
            (TyKind::ArrayN { elem: a, .. }, TyKind::ArrayN { elem: b, .. }) => self.unify_with_fixed(a, b, args, fixed),
            // `[TYP-41]` — a const argument that is one `const` parameter is
            // solved by the argument met (`Matrix[N, N]` against `Matrix[2, 2]`).
            (TyKind::Const(form), TyKind::Const(_)) => {
                let Some(slot) = form.as_param().map(|index| index as usize) else { return true };
                if slot >= args.len() || slot < fixed {
                    return true;
                }
                match args[slot] {
                    Some(existing) => existing == actual,
                    None => {
                        args[slot] = Some(actual);
                        true
                    }
                }
            }
            // A `String` is no `Array[T]` (D-201).
            (TyKind::Vec { elem: a, text: a_text }, TyKind::Vec { elem: b, text: b_text }) if a_text == b_text => {
                self.unify_with_fixed(a, b, args, fixed)
            }
            // ODR-116 — a `MutSpan[T]` given for a `Span[T]` converts (`[SPN-3]`), so its
            // elements still solve the declared view's.
            (
                TyKind::Span { elem: a, mutable: a_mut },
                TyKind::Span { elem: b, mutable: b_mut },
            ) if a_mut == b_mut || !a_mut => self.unify_with_fixed(a, b, args, fixed),
            // Two instantiations of the same generic struct unify argument
            // by argument.
            (TyKind::Struct(a), TyKind::Struct(b)) => {
                match (&self.struct_def(a).origin, &self.struct_def(b).origin) {
                    (Some((na, aa)), Some((nb, ab))) if na == nb && aa.len() == ab.len() => {
                        aa.iter()
                            .zip(ab.iter())
                            .all(|(&x, &y)| self.unify_with_fixed(x, y, args, fixed))
                    }
                    _ => true,
                }
            }
            // Two instantiations of the same generic enum (`Option[T]` against
            // `Option[String]`) unify argument by argument, as structs do.
            (TyKind::Enum(a), TyKind::Enum(b)) => {
                let (da, db) = (self.enum_def(a), self.enum_def(b));
                match (&da.origin, &db.origin) {
                    (Some((na, aa)), Some((nb, ab))) if na == nb && aa.len() == ab.len() => {
                        aa.iter()
                            .zip(ab.iter())
                            .all(|(&x, &y)| self.unify_with_fixed(x, y, args, fixed))
                    }
                    // `Option` and `Result` are compiler-known and named by
                    // their payloads (`Option_T`, `Result_T_E`), with no origin.
                    _ => {
                        let compiler_known = |prefix: &str| {
                            da.name.as_str().starts_with(prefix) && db.name.as_str().starts_with(prefix)
                        };
                        if !(compiler_known("Option_") || compiler_known("Result_"))
                            || da.variants.len() != db.variants.len()
                        {
                            return true;
                        }
                        let (va, vb) = (da.variants.clone(), db.variants.clone());
                        va.iter().zip(&vb).all(|(x, y)| {
                            x.fields.len() != y.fields.len()
                                || x.fields.iter().zip(&y.fields).all(|(fx, fy)| {
                                    self.unify_with_fixed(fx.ty, fy.ty, args, fixed)
                                })
                        })
                    }
                }
            }
            // A class handle is nominal just like a struct. Generic class
            // recipes are not instantiated yet, but preserving this shape
            // prevents class identity from collapsing into pointer identity.
            (TyKind::Class(a), TyKind::Class(b)) => {
                match (&self.class_def(a).origin, &self.class_def(b).origin) {
                    (Some((na, aa)), Some((nb, ab))) if na == nb && aa.len() == ab.len() => {
                        aa.iter()
                            .zip(ab.iter())
                            .all(|(&x, &y)| self.unify_with_fixed(x, y, args, fixed))
                    }
                    _ => a == b,
                }
            }
            (TyKind::ClassInterface(a), TyKind::ClassInterface(b)) => a == b,
            (TyKind::Tuple(a), TyKind::Tuple(b)) if a.len() == b.len() => {
                a.iter()
                    .zip(b.iter())
                    .all(|(&x, &y)| self.unify_with_fixed(x, y, args, fixed))
            }
            // `[FN-6a]` — callable parameter modes are part of the function
            // type's identity.  Inference may solve the nested types, but it
            // must never erase `borrow`/`mut`/`owned` on its way there.
            (
                TyKind::Fn { abi: a_abi, latebound: a_latebound, params: a, ret: a_ret, .. },
                TyKind::Fn { abi: b_abi, latebound: b_latebound, params: b, ret: b_ret, .. },
            ) if a.len() == b.len()
                    && a_abi == b_abi
                    && a_latebound == b_latebound
                    && a.iter().zip(&b).all(|(left, right)| left.mode == right.mode) =>
            {
                a.iter()
                    .zip(&b)
                    .all(|(left, right)| {
                        self.unify_with_fixed(left.ty, right.ty, args, fixed)
                    })
                    && self.unify_with_fixed(a_ret, b_ret, args, fixed)
            }
            // A concrete declared type has nothing to infer; the ordinary
            // coercion check decides whether the argument fits.
            _ => true,
        }
    }

    pub fn fresh_infer(&mut self) -> Ty {
        let id = InferId(self.next_infer);
        self.next_infer += 1;
        self.intern(TyKind::Infer(id))
    }

    pub fn add_struct(&mut self, def: StructDef) -> StructId {
        let id = StructId(self.structs.len() as u32);
        self.structs.push(def);
        id
    }

    pub fn struct_def(&self, id: StructId) -> &StructDef {
        &self.structs[id.0 as usize]
    }

    pub fn struct_def_mut(&mut self, id: StructId) -> &mut StructDef {
        &mut self.structs[id.0 as usize]
    }

    pub fn structs(&self) -> impl Iterator<Item = (StructId, &StructDef)> {
        self.structs.iter().enumerate().map(|(i, d)| (StructId(i as u32), d))
    }

    pub fn add_class(&mut self, def: ClassDef) -> ClassId {
        let id = ClassId(self.classes.len() as u32);
        self.classes.push(def);
        id
    }

    pub fn class_def(&self, id: ClassId) -> &ClassDef {
        &self.classes[id.0 as usize]
    }

    pub fn class_def_mut(&mut self, id: ClassId) -> &mut ClassDef {
        &mut self.classes[id.0 as usize]
    }

    /// Whether `derived` is the same class as `base` or inherits from it.
    ///
    /// Class inheritance is single-parent in v1, so this is the complete
    /// nominal upcast relation. The bounded walk also keeps malformed cyclic
    /// declarations from making type checking loop forever; cycle diagnostics
    /// remain a separate class-collection concern.
    pub fn class_is_subclass_of(&self, derived: ClassId, base: ClassId) -> bool {
        let mut current = derived;
        for _ in 0..=self.classes.len() {
            if current == base {
                return true;
            }
            let Some(parent) = self.class_def(current).base else {
                return false;
            };
            current = parent;
        }
        false
    }

    /// The classes that exist at run time: every class but an instance over
    /// generic parameters, which the checker makes only to check a generic
    /// class's methods once (`[TYP-17]`) and which has no method bodies.
    pub fn runtime_classes(&self) -> impl Iterator<Item = (ClassId, &ClassDef)> {
        self.classes().filter(|(_, def)| {
            !def.origin.as_ref().is_some_and(|(_, args)| args.iter().any(|&arg| self.is_generic(arg)))
        })
    }

    pub fn classes(&self) -> impl Iterator<Item = (ClassId, &ClassDef)> {
        self.classes.iter().enumerate().map(|(i, d)| (ClassId(i as u32), d))
    }

    /// Look up a field in a class or one of its single-inheritance bases.
    /// The returned index is the object-layout index: base fields precede
    /// derived fields, matching `[CLS-4]` and the generated object layout.
    pub fn class_field(&self, id: ClassId, name: Symbol) -> Option<(usize, &FieldDef)> {
        self.class_field_info(id, name)
            .map(|(index, _, field)| (index, field))
    }

    /// Like [`TypeTable::class_field`], retaining the declaration that owns
    /// the field so module-visibility checks can follow inherited fields.
    pub fn class_field_info(
        &self,
        id: ClassId,
        name: Symbol,
    ) -> Option<(usize, ClassId, &FieldDef)> {
        let def = self.class_def(id);
        if let Some((index, field)) = def.field(name) {
            let base_fields = def.base.map_or(0, |base| self.class_field_count(base));
            return Some((base_fields + index, id, field));
        }
        def.base.and_then(|base| self.class_field_info(base, name))
    }

    /// Resolve a class object-layout field index, including inherited fields.
    pub fn class_field_at(&self, id: ClassId, index: usize) -> Option<&FieldDef> {
        self.class_field_at_info(id, index).map(|(_, field)| field)
    }

    /// Resolve an object-layout field index and retain its declaring class.
    pub fn class_field_at_info(&self, id: ClassId, index: usize) -> Option<(ClassId, &FieldDef)> {
        let def = self.class_def(id);
        let base_count = def.base.map_or(0, |base| self.class_field_count(base));
        if index < base_count {
            return def.base.and_then(|base| self.class_field_at_info(base, index));
        }
        def.fields
            .get(index - base_count)
            .map(|field| (id, field))
    }

    /// `[EXC-19]` — whether field `index` of class `id` (object layout, bases
    /// first) has its own access word: every non-`Copy` field. A `Copy`
    /// field's reads and writes are instantaneous and never checked
    /// (`[EXC-17]`). Immutable `@sync` objects need no access words.
    pub fn class_field_has_access_word(&self, id: ClassId, index: usize) -> bool {
        !self.class_def(id).is_sync
            && self.class_field_at(id, index).is_some_and(|field| !self.is_copy(field.ty))
    }

    /// Number of fields physically present in the object, including bases.
    pub fn class_field_count(&self, id: ClassId) -> usize {
        let def = self.class_def(id);
        def.fields.len() + def.base.map_or(0, |base| self.class_field_count(base))
    }

    /// `[RNG-2]` — every declaration gets its own id, so two range types
    /// over the same representation with the same bounds are distinct types.
    pub fn add_range(&mut self, def: RangeDef) -> RangeId {
        let id = RangeId(self.ranges.len() as u32);
        self.ranges.push(def);
        id
    }

    pub fn range_def(&self, id: RangeId) -> &RangeDef {
        &self.ranges[id.0 as usize]
    }

    pub fn ranges(&self) -> impl Iterator<Item = (RangeId, &RangeDef)> {
        self.ranges.iter().enumerate().map(|(i, d)| (RangeId(i as u32), d))
    }

    /// `[RNG-8]`/`[TYP-5]` — the representation a range type erases to, or the
    /// type itself where it is not one. Written once so that every coercion
    /// site, every operator and the backend all erase the same way.
    pub fn erase_range(&self, ty: Ty) -> Ty {
        match self.kind(ty) {
            TyKind::Range(id) => self.range_def(*id).repr,
            _ => ty,
        }
    }

    pub fn add_enum(&mut self, def: EnumDef) -> EnumId {
        let id = EnumId(self.enums.len() as u32);
        self.enums.push(def);
        id
    }

    pub fn enum_def(&self, id: EnumId) -> &EnumDef {
        &self.enums[id.0 as usize]
    }

    pub fn enum_def_mut(&mut self, id: EnumId) -> &mut EnumDef {
        &mut self.enums[id.0 as usize]
    }

    pub fn enums(&self) -> impl Iterator<Item = (EnumId, &EnumDef)> {
        self.enums.iter().enumerate().map(|(i, d)| (EnumId(i as u32), d))
    }

    /// Every type that has been interned, in interning order.
    ///
    /// The C backend walks this to find the structural types — tuples and
    /// fixed arrays — that need a generated `struct` definition. Interning
    /// order is not relied upon for correctness; the backend sorts what it
    /// finds by containment.
    pub fn all(&self) -> impl Iterator<Item = (Ty, &TyKind)> {
        self.kinds.iter().enumerate().map(|(i, k)| (Ty(i as u32), k))
    }

    pub fn pointer_size(&self) -> u64 {
        self.pointer_size
    }

    // -- properties ---------------------------------------------------------

    /// `[TYP-11]` — C-compatible layout: fields in declaration order, each at
    /// its natural alignment, with trailing padding to the struct's alignment.
    /// No reordering (`@layout(rust)` is v2).
    pub fn layout(&self, ty: Ty) -> Layout {
        match self.kind(ty) {
            // `[TYP-41]` — generic code's only; an instance's are numbers.
            TyKind::ArrayN { .. } | TyKind::Const(_) => Layout::ZERO,
            TyKind::Bool => Layout::scalar(1),
            TyKind::Char => Layout::scalar(4),
            // G8-4 — four 64-bit words, aligned as an `i128` is.
            TyKind::Int(IntTy::I256) => Layout { size: 32, align: 16, field_offsets: Vec::new() },
            TyKind::Int(int) => Layout::scalar(match int {
                IntTy::I8 => 1,
                IntTy::I16 => 2,
                IntTy::I32 => 4,
                IntTy::I64 => 8,
                IntTy::I128 => 16,
                IntTy::I256 => 32,
                IntTy::Isize => self.pointer_size,
            }),
            TyKind::Uint(UintTy::U256) => Layout { size: 32, align: 16, field_offsets: Vec::new() },
            TyKind::Uint(uint) => Layout::scalar(match uint {
                UintTy::U8 => 1,
                UintTy::U16 => 2,
                UintTy::U32 => 4,
                UintTy::U64 => 8,
                UintTy::U128 => 16,
                UintTy::U256 => 32,
                UintTy::Usize => self.pointer_size,
            }),
            TyKind::Float(float) => Layout::scalar(match float {
                FloatTy::F16 => 2,
                FloatTy::F32 => 4,
                FloatTy::F64 => 8,
            }),
            TyKind::Void | TyKind::Never => Layout::ZERO,
            // A view: pointer plus length.
            TyKind::Str | TyKind::Span { .. } => Layout {
                size: self.pointer_size * 2,
                align: self.pointer_size,
                field_offsets: vec![0, self.pointer_size],
            },
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. }
                if matches!(self.kind(*inner), TyKind::Dyn { .. }) =>
            {
                Layout {
                    size: self.pointer_size * 2,
                    align: self.pointer_size,
                    field_offsets: vec![0, self.pointer_size],
                }
            }
            // `[CLO-10]` — an owned callable value: its entry (held in the
            // value, so a call is one indirect call), its descriptor and three
            // words of captured state.
            TyKind::Fn { abi: None, .. } => Layout {
                size: self.pointer_size * 5,
                align: self.pointer_size,
                field_offsets: vec![0, self.pointer_size, self.pointer_size * 2],
            },
            TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Fn { .. } | TyKind::CStr => {
                Layout::scalar(self.pointer_size)
            }
            TyKind::Array { elem, len } => {
                let inner = self.layout(*elem);
                Layout { size: inner.size * len, align: inner.align, field_offsets: Vec::new() }
            }
            TyKind::Tuple(items) => self.aggregate_layout(items.iter().copied(), LayoutAttrs::NONE),
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                // `[SIMD-1]` — a vector or mask is aligned to its size.
                if let Some(shape) = SimdShape::of(def.name.as_str()) {
                    let size = shape.bytes().into();
                    return Layout { size, align: size, field_offsets: vec![0] };
                }
                self.aggregate_layout(def.fields.iter().map(|f| f.ty), self.layout_attrs(ty))
            }
            // A class expression is a non-null handle. The heap object's
            // header and fields are described by `ClassDef`, not by the
            // source-level handle layout.
            TyKind::Class(_) | TyKind::ClassInterface(_) => Layout::scalar(self.pointer_size),
            TyKind::Enum(id) => self.enum_layout(*id, self.layout_attrs(ty)),
            // `[RNG-8]` — a range type erases to its representation, so
            // it is laid out as one and crosses an FFI boundary as one.
            TyKind::Range(id) => self.layout(self.range_def(*id).repr),
            // A pointer and two lengths, whatever the element type.
            TyKind::Vec { .. } => Layout {
                size: self.pointer_size * 3,
                align: self.pointer_size,
                field_offsets: vec![0, self.pointer_size, self.pointer_size * 2],
            },
            // A parameter has no layout until it is substituted away.
            TyKind::Param { .. } | TyKind::Assoc { .. } => Layout::ZERO,
            // `[TYP-22]` — `dyn I` is unsized and may only occur behind an
            // indirection such as `ref` or `Box`.
            TyKind::Dyn { .. } | TyKind::Opaque(_) => Layout::ZERO,
            TyKind::Infer(_) | TyKind::IntLit | TyKind::FloatLit | TyKind::Error => Layout::ZERO,
        }
    }

    /// `[TYP-12]` — a unit-only enum is its discriminant. A payload enum is
    /// `{tag, union of variants}`, tag first. A niche `Option[T]` has the
    /// same layout as `T` and keeps `None` in a value unavailable to `T`.
    /// `[LAY-2]` — `@packed` puts the union straight after the tag and packs
    /// each variant; `@align(N)` raises the alignment, rounding the size up.
    fn enum_layout(&self, id: EnumId, attrs: LayoutAttrs) -> Layout {
        if let Some(niche) = self.option_niche(id) {
            return self.layout(niche.payload);
        }
        let def = self.enum_def(id);
        let tag = self.layout(def.repr);
        let (offset, size, align) = if def.is_unit_only() {
            (tag.size, 0, tag.align)
        } else {
            let variant = LayoutAttrs { packed: attrs.packed, align: 1 };
            let mut payload_size = 0u64;
            let mut payload_align = 1u64;
            for variant_def in &def.variants {
                let inner = self.aggregate_layout(variant_def.fields.iter().map(|f| f.ty), variant);
                payload_size = payload_size.max(inner.size);
                payload_align = payload_align.max(inner.align);
            }
            if attrs.packed {
                (tag.size, payload_size, 1)
            } else {
                (align_to(tag.size, payload_align), payload_size, tag.align.max(payload_align))
            }
        };
        let align = align.max(attrs.align);
        let field_offsets = if def.is_unit_only() { Vec::new() } else { vec![0, offset] };
        Layout { size: align_to(offset + size, align), align, field_offsets }
    }

    /// `[TYP-11]` — fields in order, each at its natural alignment, then trailing padding;
    /// `[LAY-2]` — under `@packed` no padding at all, and `@align(N)` raises the alignment.
    fn aggregate_layout(&self, fields: impl Iterator<Item = Ty>, attrs: LayoutAttrs) -> Layout {
        let mut offset = 0u64;
        let mut align = 1u64;
        let mut field_offsets = Vec::new();
        for field in fields {
            let inner = self.layout(field);
            if !attrs.packed {
                offset = align_to(offset, inner.align);
                align = align.max(inner.align);
            }
            field_offsets.push(offset);
            offset += inner.size;
        }
        let align = align.max(attrs.align);
        Layout { size: align_to(offset, align), align, field_offsets }
    }

    /// `[OWN-7]`, `[STR-3]` — `Copy` values duplicate bitwise on use. Scalars,
    /// views, raw pointers and function pointers are always `Copy`; a struct
    /// is `Copy` only if it was declared `@derive(Copy)`, so that adding a
    /// field later cannot silently change semantics.
    /// `[BRW-8]` (ODR-024) — whether a borrowed parameter of this type is
    /// passed by address, so the callee reads the caller's place: a type
    /// that is not `Copy`, or a `Copy` one holding a `Cell` or `UnsafeCell`
    /// (a write through it must reach the caller). A view or reference is
    /// passed as itself. The choice depends only on the type.
    pub fn passed_by_address(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Span { .. } | TyKind::Str | TyKind::Error => false,
            TyKind::Dyn { .. } => false,
            // A view (a `@view` struct, a tuple holding one) is already a
            // borrow and passes as itself, keeping each field's own region.
            _ if self.is_view(ty) => false,
            _ => !self.is_copy(ty) || self.holds_a_cell(ty),
        }
    }

    /// Whether a value of this type holds a `Cell` or `UnsafeCell` in its
    /// own storage.
    pub fn holds_a_cell(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                let name = def.name.as_str();
                name.starts_with("Cell_")
                    || name.starts_with("UnsafeCell_")
                    || def.fields.iter().any(|f| self.holds_a_cell(f.ty))
            }
            TyKind::Tuple(items) => items.iter().any(|&t| self.holds_a_cell(t)),
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => self.holds_a_cell(*elem),
            TyKind::Enum(id) => self
                .enum_def(*id)
                .variants
                .iter()
                .any(|v| v.fields.iter().any(|f| self.holds_a_cell(f.ty))),
            _ => false,
        }
    }

    pub fn is_copy(&self, ty: Ty) -> bool {
        self.copy_with(ty, false)
    }

    /// `[THR-8]`/`[THR-9]` — structural thread properties. Compiler-owned
    /// wrappers use their logical payload, not their private pointer field.
    /// ODR-111 — record a `Send` or `Sync` written by hand.
    pub fn add_manual_marker(&mut self, marker: ManualMarker) {
        self.manual_markers.push(marker);
    }

    /// ODR-111 — the name a by-hand marker is recorded under, and the type's arguments: a generic
    /// struct's or enum's instance by its generic's name, any other by its own.
    pub fn marker_key(&self, ty: Ty) -> Option<(Symbol, Vec<Ty>)> {
        match self.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                Some(def.origin.clone().unwrap_or((def.name, Vec::new())))
            }
            TyKind::Enum(id) => {
                let def = self.enum_def(*id);
                Some(def.origin.clone().unwrap_or((def.name, Vec::new())))
            }
            _ => None,
        }
    }

    pub fn is_send(&self, ty: Ty) -> bool {
        self.thread_properties(ty, &mut HashSet::new()).0
    }

    pub fn is_sync(&self, ty: Ty) -> bool {
        self.thread_properties(ty, &mut HashSet::new()).1
    }

    fn thread_properties(&self, ty: Ty, visiting: &mut HashSet<Ty>) -> (bool, bool) {
        if !visiting.insert(ty) {
            // Recursive value storage is impossible without indirection. A
            // recursive @sync handle is safe once its declaration is checked.
            return (true, true);
        }
        let both = |child, visiting: &mut HashSet<Ty>| self.thread_properties(child, visiting);
        let result = match self.kind(ty) {
            // `[TYP-41]` — a number, sent and shared as one is.
            TyKind::Const(_) => (true, true),
            // ODR-102 — an owned callable value hides what it captured, so it
            // stays on the thread that made it.
            TyKind::Fn { abi: None, stateless: false, .. } => (false, false),
            TyKind::Bool | TyKind::Char | TyKind::Int(_) | TyKind::Uint(_)
            | TyKind::Float(_) | TyKind::Void | TyKind::Never | TyKind::Str
            | TyKind::Range(_) | TyKind::Fn { .. } | TyKind::Error => (true, true),
            TyKind::Class(id) => {
                let safe = self.class_def(*id).is_sync;
                (safe, safe)
            }
            TyKind::CStr => (false, false),
            TyKind::Ref { mutable, inner } | TyKind::Span { mutable, elem: inner } => {
                let (send, sync) = both(*inner, visiting);
                if *mutable { (send, false) } else { (sync, sync) }
            }
            TyKind::Ptr { .. } | TyKind::ClassInterface(_) | TyKind::Dyn { .. } | TyKind::Opaque(_)
            | TyKind::Param { .. } | TyKind::Assoc { .. } | TyKind::Infer(_)
            | TyKind::IntLit | TyKind::FloatLit => (false, false),
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                if let Some((name, args)) = &def.origin {
                    if name.is("Shared") {
                        (false, false) // `[HEAP-10]`
                    } else if name.is("SyncShared") && args.len() == 1 {
                        // `[THR-8]`, `[THR-9]` — any thread may read the value and drop the last owner.
                        let (send, sync) = both(args[0], visiting);
                        (send && sync, send && sync)
                    } else if name.is("Box") && args.len() == 1 {
                        both(args[0], visiting)
                    } else if name.is("Weak") && args.len() == 1 {
                        let (_, sync) = both(args[0], visiting);
                        (sync, sync)
                    } else if name.is("std.mem.UnsafeCell") && args.len() == 1 {
                        (both(args[0], visiting).0, false)
                    } else if name.is("std.sync.Mutex") && args.len() == 1 {
                        // `[THR-9]` — one thread at a time holds its `ref mut T`.
                        let send = both(args[0], visiting).0;
                        (send, send)
                    } else if name.is("std.sync.MutexGuard") && args.len() == 1 {
                        // `[THR-3]` — a guard is not `Send`: it is unlocked where it was taken.
                        (false, both(args[0], visiting).1)
                    } else if (name.is("SoARef") || name.is("SoAMut")) && args.len() == 1 {
                        // `[SOA-6]` — a reference to the `SoA` and an index.
                        let (send, sync) = both(args[0], visiting);
                        if name.is("SoARef") { (sync, sync) } else { (send, false) }
                    } else if name.is("SoA") && args.len() == 1 {
                        // `[SOA-1]` — its columns are `T`'s fields, behind private pointers.
                        both(args[0], visiting)
                    } else if name.is("std.thread.Scope") || name.is("std.thread.ScopedJoinHandle")
                        || name.is("std.jobs.JobScope") || name.is("std.jobs.ScopedJob")
                    {
                        // `[THR-5]` — they stay in the block that made them.
                        (false, false)
                    } else if name.is("std.thread.JoinHandle") {
                        // `[THR-10]` — the result is `Send`, and only `join` takes it.
                        (true, true)
                    } else if name.is("std.sync.Atomic") {
                        // `[THR-9]` — every access is atomic; it holds plain values only.
                        (true, true)
                    } else if name.is("std.sync.RwLock") && args.len() == 1 {
                        // `[THR-9]` — several threads hold a `ref T` at once.
                        let (send, sync) = both(args[0], visiting);
                        (send, send && sync)
                    } else if (name.is("std.sync.RwLockReadGuard") || name.is("std.sync.RwLockWriteGuard"))
                        && args.len() == 1
                    {
                        (false, both(args[0], visiting).1)
                    } else {
                        def.fields.iter().map(|field| both(field.ty, visiting))
                            .fold((true, true), |a, b| (a.0 && b.0, a.1 && b.1))
                    }
                } else if def.name.as_str().starts_with("Cell_")
                    || def.name.as_str().starts_with("RefCell_")
                {
                    // `[CELL-3]`, `[CELL-8]` — `Send` is the payload's (`value`);
                    // a `RefCell`'s borrow-site pointer is its own bookkeeping.
                    (both(def.fields[0].ty, visiting).0, false)
                } else {
                    def.fields.iter().map(|field| both(field.ty, visiting))
                        .fold((true, true), |a, b| (a.0 && b.0, a.1 && b.1))
                }
            }
            TyKind::Enum(id) => self.enum_def(*id).variants.iter()
                .flat_map(|variant| &variant.fields)
                .map(|field| both(field.ty, visiting))
                .fold((true, true), |a, b| (a.0 && b.0, a.1 && b.1)),
            TyKind::Tuple(items) => items.iter().map(|&item| both(item, visiting))
                .fold((true, true), |a, b| (a.0 && b.0, a.1 && b.1)),
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } | TyKind::Vec { elem, .. } => both(*elem, visiting),
        };
        // `[TYP-35]` — a phantom parameter counts as a field of its type.
        let mut result = match self.kind(ty) {
            TyKind::Struct(_) | TyKind::Enum(_) => self.phantoms(ty).iter()
                .map(|&arg| both(arg, visiting))
                .fold(result, |a, b| (a.0 && b.0, a.1 && b.1)),
            _ => result,
        };
        // ODR-111 — and what the program wrote by hand, where each argument is what it asks.
        if let Some((name, args)) = self.marker_key(ty) {
            for marker in self.manual_markers.iter().filter(|marker| marker.name == name && marker.args.len() == args.len()) {
                let holds = marker.args.iter().zip(&args).all(|(wanted, &arg)| match *wanted {
                    MarkerArg::Exact(exact) => exact == arg,
                    MarkerArg::Any { send, sync } => {
                        let (is_send, is_sync) = both(arg, visiting);
                        (!send || is_send) && (!sync || is_sync)
                    }
                });
                if holds {
                    result = (result.0 || marker.send, result.1 || marker.sync);
                }
            }
        }
        visiting.remove(&ty);
        result
    }

    /// `[TYP-13]`, `[STD-4]` — an `Option[T]` that keeps `None` inside a
    /// `T`, when `T` has a niche: a value no `T` ever holds. This covers
    /// non-null pointers, spare scalar and enum values, constrained ranges,
    /// and the impossible pointer/length pair of a view.
    pub fn option_niche(&self, id: EnumId) -> Option<Niche> {
        let def = self.enum_def(id);
        if !def.name.as_str().starts_with("Option_") || def.variants.len() != 2 {
            return None;
        }
        let none = def.variants.iter().position(|v| v.fields.is_empty())?;
        let some = def.variants.iter().position(|v| v.fields.len() == 1)?;
        let payload = def.variants[some].fields[0].ty;
        (self.is_nonzero(payload)
            || matches!(self.kind(payload), TyKind::Class(_) | TyKind::ClassInterface(_))
            || matches!(self.kind(payload), TyKind::Bool | TyKind::Char)
            || matches!(self.kind(payload), TyKind::Span { .. } | TyKind::Str | TyKind::CStr)
            || matches!(self.kind(payload), TyKind::Ref { .. })
            || matches!(self.kind(payload), TyKind::Fn { abi: Some(_), .. })
            || matches!(self.kind(payload), TyKind::Struct(id) if self.compiler_box_inner(*id).is_some())
            || matches!(self.kind(payload), TyKind::Enum(id) if self.enum_unused_discriminant(*id).is_some())
            || matches!(self.kind(payload), TyKind::Range(id) if self.range_unused_integer(*id).is_some()
                || matches!(self.kind(self.range_def(*id).repr), TyKind::Float(_))))
        .then_some(Niche { none, some, payload })
    }

    /// `[RNG-7]` — a contiguous integer range leaves a niche if it excludes
    /// zero or either end of its representation. Return the sentinel's raw
    /// bits, so an unsigned 128-bit maximum and a signed minimum both fit.
    pub fn range_unused_integer(&self, id: RangeId) -> Option<u128> {
        let def = self.range_def(id);
        let Bound::Int(_) = def.lo else { return None };
        if !def.contains(Bound::Int(0)) {
            return Some(0);
        }
        let max = int_max(self, def.repr)?;
        if max > i128::MAX as u128 || !def.contains(Bound::Int(max as i128)) {
            return Some(max);
        }
        if is_signed(self, def.repr) == Some(true) {
            let bits = bit_width(self, def.repr)?;
            let min = if bits == 128 { i128::MIN } else { -(1i128 << (bits - 1)) };
            if !def.contains(Bound::Int(min)) {
                return Some(min as u128);
            }
        }
        None
    }

    /// A unit enum may leave an integer value unused. Check nonnegative
    /// values first, then negative ones when its representation is signed.
    /// At most one more candidate than there are variants is needed.
    pub fn enum_unused_discriminant(&self, id: EnumId) -> Option<i128> {
        let def = self.enum_def(id);
        // `[FFI-8]` — an open enum has no unused value: C may send any.
        if !def.is_unit_only() || def.open {
            return None;
        }
        let used = def.variants.iter().map(|variant| variant.discriminant).collect::<HashSet<_>>();
        let max = int_max(self, def.repr)?;
        let count = def.variants.len().saturating_add(1);
        for value in 0..count {
            let Ok(candidate) = i128::try_from(value) else { break };
            if candidate as u128 > max {
                break;
            }
            if !used.contains(&candidate) {
                return Some(candidate);
            }
        }
        if is_signed(self, def.repr) == Some(true) {
            let bits = bit_width(self, def.repr)?;
            let min = if bits == 128 { i128::MIN } else { -(1i128 << (bits - 1)) };
            for value in 1..=count {
                let Ok(offset) = i128::try_from(value) else { break };
                let candidate = -offset;
                if candidate < min {
                    break;
                }
                if !used.contains(&candidate) {
                    return Some(candidate);
                }
            }
        }
        None
    }

    /// The payload of the compiler-known `Box[T]` struct `id`. Its origin is
    /// `Box`, and the compiler made it: a root module's own `Box[T]` has the
    /// same origin name (D-305).
    pub fn compiler_box_inner(&self, id: StructId) -> Option<Ty> {
        let def = self.struct_def(id);
        match &def.origin {
            Some((name, args)) if name.is("Box") && args.len() == 1 && def.declaring_module == COMPILER_MODULE => {
                Some(args[0])
            }
            _ => None,
        }
    }

    /// The payload of the compiler-known `Cell[T]` struct `id` (`cell_of`):
    /// the compiler made it, and it is written `Cell[T]`.
    pub fn compiler_cell_inner(&self, id: StructId) -> Option<Ty> {
        let def = self.struct_def(id);
        match self.written_as.get(&id) {
            Some((name, args)) if name.is("Cell") && args.len() == 1 && def.declaring_module == COMPILER_MODULE => {
                Some(args[0])
            }
            _ => None,
        }
    }

    /// ODR-094 — the type a compiler-known wrapper prints as: `Box[T]` and
    /// `Cell[T]` print as the `T` they hold (`Box(5)` prints `5`). A boxed
    /// `dyn` value has no type of its own to print as.
    pub fn printed_as(&self, ty: Ty) -> Option<Ty> {
        let TyKind::Struct(id) = *self.kind(ty) else { return None };
        // `[TYP-32]` — a `some Display` (or `Debug`) prints as the value it holds.
        if let Some(&(display, debug)) = self.some_prints.get(&id) {
            return self.struct_def(id).fields.first().filter(|_| display || debug).map(|field| field.ty);
        }
        if let Some(inner) = self.compiler_box_inner(id) {
            return (!matches!(self.kind(inner), TyKind::Dyn { .. })).then_some(inner);
        }
        self.compiler_cell_inner(id)
    }

    /// `[STD-4]` — `std.core.NonZero[T]`, whose one field is never 0: only
    /// `NonZero.new` makes one, after testing.
    pub fn is_nonzero(&self, ty: Ty) -> bool {
        matches!(
            self.kind(ty),
            TyKind::Struct(id) if matches!(&self.struct_def(*id).origin, Some((name, _)) if name.is("std.core.NonZero"))
        )
    }

    /// `[LT-1]` (ODR-024) — `is_copy` with every type parameter taken to be
    /// `Copy`, so whether a parameter is a source is fixed by the declared
    /// signature and is the same for every instantiation.
    pub fn is_copy_for_elision(&self, ty: Ty) -> bool {
        self.copy_with(ty, true)
    }

    /// `[LT-1]` (ODR-024) — whether a parameter of this type, borrowed or
    /// `mut` (`by_reference`) or `owned`, is a source parameter: a reference or
    /// view in any mode, or a borrowed or `mut` parameter whose type is not
    /// `Copy` with each type parameter taken to be `Copy`.
    pub fn is_source_parameter(&self, ty: Ty, by_reference: bool) -> bool {
        self.is_view(ty) || (by_reference && !self.is_copy_for_elision(ty))
    }

    /// `[LT-7]` — the source parameters of a callable type, which a call
    /// through a value of that type borrows (`None` for anything else).
    pub fn callable_sources(&self, callee: Ty) -> Option<Vec<usize>> {
        let TyKind::Fn { params, .. } = self.kind(callee) else { return None };
        Some(
            params
                .iter()
                .enumerate()
                .filter(|(_, param)| self.is_source_parameter(param.ty, param.mode != FnParamMode::Owned))
                .map(|(index, _)| index)
                .collect(),
        )
    }

    fn copy_with(&self, ty: Ty, params: bool) -> bool {
        match self.kind(ty) {
            TyKind::Const(_) => true,
            // `[CLO-3]` — an owned callable value owns its captured state,
            // so it moves; only `extern fn` is `Copy` (Part VI's list).
            TyKind::Fn { abi: None, stateless, .. } => *stateless,
            TyKind::Bool
            | TyKind::Char
            | TyKind::Int(_)
            | TyKind::Uint(_)
            | TyKind::Float(_)
            | TyKind::Void
            | TyKind::Never
            | TyKind::Str
            | TyKind::CStr
            | TyKind::Ptr { .. }
            | TyKind::Fn { .. }
            | TyKind::IntLit
            | TyKind::FloatLit
            | TyKind::Error => true,
            // `[SPN-3]` — "`Span[T]` is `Copy`; `MutSpan[T]` is move-only
            // and reborrowable", for the same reason `ref` and `ref mut`
            // differ: two live copies of a writable view are two writers.
            TyKind::Span { mutable, .. } => !mutable,
            // `ref T` is Copy; `ref mut T` is move-only and reborrowable.
            TyKind::Ref { mutable, .. } => !mutable,
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => self.copy_with(*elem, params),
            TyKind::Tuple(items) => items.iter().all(|&t| self.copy_with(t, params)),
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                def.derives_copy && !def.has_drop && def.fields.iter().all(|f| self.copy_with(f.ty, params))
            }
            // `[OWN-7]` explicitly makes class handles Copy, with retain and
            // release emitted by the ownership/codegen stages rather than a
            // raw bitwise copy.
            TyKind::Class(_) | TyKind::ClassInterface(_) => true,
            // `[ENM-3]` — a unit-only enum is `Copy` automatically, because it
            // is only its discriminant. `[ENM-4]` — a payload enum needs
            // `@derive(Copy)`, like a struct.
            TyKind::Enum(id) => {
                let def = self.enum_def(*id);
                def.is_unit_only()
                    || (def.derives_copy
                        && !def.has_drop
                        && def
                            .variants
                            .iter()
                            .all(|v| v.fields.iter().all(|f| self.copy_with(f.ty, params))))
            }
            // `[RNG-1]`/`[RNG-10]`(e) — a range type is its representation
            // with a narrower set of valid values, and every representation is
            // a scalar, so it is `Copy` and a copy is one of the five ways a
            // range-typed value may arise.
            TyKind::Range(_) => true,
            // An `Array` owns its buffer, so copying it would share one
            // allocation between two owners.
            TyKind::Vec { .. } => false,
            // `[TYP-17]` — whether a parameter is `Copy` is what its bounds
            // say, which the checker consults rather than the type table.
            TyKind::Param { .. } | TyKind::Assoc { .. } => params,
            TyKind::Infer(_) => false,
            TyKind::Dyn { .. } | TyKind::Opaque(_) => false,
        }
    }

    /// `[CORO-3]` — record a `gen fn`'s frame struct and how each of its
    /// parameter fields (the first ones, in order) was passed.
    pub fn declare_coroutine_frame(&mut self, name: Symbol, params: Vec<FrameParam>) {
        self.coroutine_frames.insert(name, params);
    }

    /// `[CORO-3]` — whether `ty` is a `gen fn`'s frame.
    pub fn is_coroutine_frame(&self, ty: Ty) -> bool {
        matches!(self.kind(ty), TyKind::Struct(id) if self.coroutine_frame_params(*id).is_some())
    }

    /// `[CORO-3]` — whether a struct is a `gen fn`'s frame.
    pub fn is_coroutine_frame_struct(&self, id: StructId) -> bool {
        self.coroutine_frame_params(id).is_some()
    }

    fn coroutine_frame_params(&self, id: StructId) -> Option<&Vec<FrameParam>> {
        if self.coroutine_frames.is_empty() {
            return None;
        }
        let def = self.struct_def(id);
        self.coroutine_frames.get(&def.origin.as_ref().map_or(def.name, |(origin, _)| *origin))
    }

    /// `[CORO-1]`, `[CORO-6]` — a frame holds what each parameter was
    /// passed as. The checker writes a borrowed parameter's field `ref T` and
    /// a `mut` one's `ref mut T`, which this settles once `T` is known: a
    /// borrowed one passed as a copy (`passed_by_address`, ODR-024) is held as
    /// that copy, and a `MutSpan` is its own reborrow. Its drop is the one the
    /// coroutine transform writes, which drops what the frame holds at its
    /// state, so the fields are not dropped again after it.
    pub fn settle_coroutine_frame(&mut self, id: StructId) {
        let Some(params) = self.coroutine_frame_params(id).cloned() else { return };
        let fields: Vec<Ty> = self.struct_def(id).fields.iter().map(|field| field.ty).collect();
        for (index, (mode, ty)) in params.iter().zip(fields).enumerate() {
            let settled = match (mode, self.kind(ty).clone()) {
                (FrameParam::Borrowed, TyKind::Ref { mutable: false, inner }) if !self.passed_by_address(inner) => inner,
                (FrameParam::Mut, TyKind::Ref { mutable: true, inner })
                    if matches!(self.kind(inner), TyKind::Span { mutable: true, .. }) =>
                {
                    inner
                }
                _ => ty,
            };
            self.structs[id.0 as usize].fields[index].ty = settled;
        }
        self.structs[id.0 as usize].drops_fields = false;
    }

    /// `[THR-6]` — record a struct's or class's `@must_drop`.
    pub fn declare_must_drop(&mut self, name: Symbol) {
        self.must_drop_declared.insert(name);
    }

    /// `[THR-6]` — a struct or class declared `@must_drop` itself (not one
    /// that only holds such a value).
    pub fn is_declared_must_drop(&self, ty: Ty) -> bool {
        let declared = match self.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                def.origin.as_ref().map_or(def.name, |(origin, _)| *origin)
            }
            TyKind::Class(id) => {
                let def = self.class_def(*id);
                def.origin.as_ref().map_or(def.name, |(origin, _)| *origin)
            }
            _ => return false,
        };
        self.must_drop_declared.contains(&declared)
    }

    /// `[THR-6]` — a value whose drop a safety guarantee depends on: a type
    /// declared `@must_drop`, or one owning such a value (a struct, enum,
    /// tuple, array or generic type holding one). A reference to one is not.
    pub fn is_must_drop(&self, ty: Ty) -> bool {
        self.must_drop_with(ty, &mut HashSet::new())
    }

    fn must_drop_with(&self, ty: Ty, seen: &mut HashSet<Ty>) -> bool {
        if self.must_drop_declared.is_empty() || !seen.insert(ty) {
            return false;
        }
        match self.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                let declared = def.origin.as_ref().map_or(def.name, |(origin, _)| *origin);
                self.must_drop_declared.contains(&declared)
                    || def.fields.iter().any(|field| self.must_drop_with(field.ty, seen))
            }
            TyKind::Class(id) => {
                let def = self.class_def(*id);
                let declared = def.origin.as_ref().map_or(def.name, |(origin, _)| *origin);
                self.must_drop_declared.contains(&declared)
            }
            TyKind::Enum(id) => self.enum_def(*id).variants.iter()
                .any(|variant| variant.fields.iter().any(|field| self.must_drop_with(field.ty, seen))),
            TyKind::Tuple(items) => items.iter().any(|&item| self.must_drop_with(item, seen)),
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } | TyKind::Vec { elem, .. } => self.must_drop_with(*elem, seen),
            _ => false,
        }
    }

    /// `[THR-6]` — the `Box`, `Shared`, `SyncShared` or collection in `ty` (or
    /// in a tuple, array or enum it is made of) that would hold a `@must_drop`
    /// value. A program's own struct holding one is checked where it is
    /// declared, so its fields are not searched here.
    pub fn must_drop_container(&self, ty: Ty) -> Option<Ty> {
        if self.must_drop_declared.is_empty() {
            return None;
        }
        match self.kind(ty) {
            TyKind::Vec { elem, .. } => {
                if self.is_must_drop(*elem) { Some(ty) } else { self.must_drop_container(*elem) }
            }
            TyKind::Struct(id) => {
                let (name, args) = self.struct_def(*id).origin.as_ref()?;
                let container = matches!(name.as_str(), "Box" | "Shared" | "SyncShared")
                    || name.as_str().starts_with("std.collections.");
                if container && args.iter().any(|&arg| self.is_must_drop(arg)) {
                    return Some(ty);
                }
                args.iter().find_map(|&arg| self.must_drop_container(arg))
            }
            TyKind::Enum(id) => self.enum_def(*id).variants.iter()
                .flat_map(|variant| variant.fields.iter())
                .find_map(|field| self.must_drop_container(field.ty)),
            TyKind::Tuple(items) => items.iter().find_map(|&item| self.must_drop_container(item)),
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => self.must_drop_container(*elem),
            _ => None,
        }
    }

    /// ADR-186 — record the function writing `ty`'s written `Debug`.
    pub fn set_debug_writer(&mut self, ty: Ty, symbol: String) {
        self.debug_writers.insert(ty, symbol);
    }

    /// ADR-186 — the function writing `ty`'s written `Debug`, if it has one.
    pub fn debug_writer(&self, ty: Ty) -> Option<&str> {
        self.debug_writers.get(&ty).map(String::as_str)
    }

    /// ADR-186 — the written `Debug` writers formatting a `ty` runs: its own,
    /// or those of the parts its field-wise text shows (an element, a field,
    /// a payload, what a reference, a `Box` or a `Cell` holds).
    pub fn debug_writers_reached(&self, ty: Ty) -> Vec<&str> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let mut pending = vec![ty];
        while let Some(ty) = pending.pop() {
            if !seen.insert(ty) {
                continue;
            }
            if let Some(writer) = self.debug_writer(ty) {
                out.push(writer);
                continue;
            }
            match self.kind(ty) {
                TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => pending.push(*inner),
                TyKind::Vec { elem, .. } | TyKind::Span { elem, .. } | TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => {
                    pending.push(*elem)
                }
                TyKind::Tuple(items) => pending.extend(items.iter().copied()),
                TyKind::Struct(id) => {
                    pending.extend(self.printed_as(ty));
                    pending.extend(self.struct_def(*id).fields.iter().map(|field| field.ty));
                }
                TyKind::Enum(id) => {
                    pending.extend(self.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)))
                }
                _ => {}
            }
        }
        out
    }

    /// `[LAY-2]` — record a struct's or an enum's `@packed` and `@align(N)`.
    pub fn declare_layout(&mut self, name: Symbol, attrs: LayoutAttrs) {
        if !attrs.is_none() {
            self.layout_attrs.insert(name, attrs);
        }
    }

    /// `[LAY-2]` — the layout attributes a struct or an enum was declared with.
    pub fn layout_attrs(&self, ty: Ty) -> LayoutAttrs {
        let declared = match self.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                def.origin.as_ref().map_or(def.name, |(origin, _)| *origin)
            }
            TyKind::Enum(id) => {
                let def = self.enum_def(*id);
                def.origin.as_ref().map_or(def.name, |(origin, _)| *origin)
            }
            _ => return LayoutAttrs::NONE,
        };
        self.layout_attrs.get(&declared).copied().unwrap_or(LayoutAttrs::NONE)
    }

    /// `[LAY-2]` — what the inline storage of `ty` holds whose alignment is stated rather than
    /// natural: whether a type declared `@align(N)`, and whether a SIMD vector (`[SIMD-1]`).
    /// A pointer, a reference or a heap buffer holds its target elsewhere.
    pub fn stated_alignment(&self, ty: Ty) -> (bool, bool) {
        let (mut aligned, mut simd) = (false, false);
        let mut fields: Vec<Ty> = Vec::new();
        match self.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                if SimdShape::of(def.name.as_str()).is_some() {
                    return (false, true);
                }
                aligned = self.layout_attrs(ty).align > 1;
                fields.extend(def.fields.iter().map(|field| field.ty));
            }
            TyKind::Enum(id) => {
                aligned = self.layout_attrs(ty).align > 1;
                fields.extend(self.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)));
            }
            TyKind::Tuple(items) => fields.extend(items.iter().copied()),
            TyKind::Array { elem, .. } => fields.push(*elem),
            _ => {}
        }
        for field in fields {
            let (a, v) = self.stated_alignment(field);
            aligned |= a;
            simd |= v;
        }
        (aligned, simd)
    }

    /// `[SOA-1]` — record a field's `@soa(flatten)`.
    pub fn mark_soa_flatten(&mut self, structure: Symbol, field: Symbol) {
        self.soa_flattened.insert((structure, field));
    }

    /// `[SOA-1]` — the columns of `SoA[T]` for the struct `element`, in field order: one per field,
    /// and for a field marked `@soa(flatten)` whose type is a struct, one per field of its own (by
    /// the same rule, so a flattened field of it is split again). Each column is the path of field
    /// indices from the element to it, its name (the field names joined by `__`), and its type.
    pub fn soa_columns(&self, element: StructId) -> Vec<SoaColumn> {
        let mut out = Vec::new();
        self.soa_columns_into(element, &mut Vec::new(), &mut String::new(), &mut out);
        out
    }

    fn soa_columns_into(&self, structure: StructId, path: &mut Vec<usize>, name: &mut String, out: &mut Vec<SoaColumn>) {
        let def = self.struct_def(structure);
        let declared = def.origin.as_ref().map_or(def.name, |(origin, _)| *origin);
        for (index, field) in def.fields.iter().enumerate() {
            let at = name.len();
            if !name.is_empty() {
                name.push_str("__");
            }
            name.push_str(field.name.as_str());
            path.push(index);
            match self.kind(field.ty) {
                TyKind::Struct(inner) if self.soa_flattened.contains(&(declared, field.name)) => {
                    self.soa_columns_into(*inner, path, name, out)
                }
                _ => out.push(SoaColumn { path: path.clone(), name: name.clone(), ty: field.ty }),
            }
            path.pop();
            name.truncate(at);
        }
    }

    /// `[SOA-1]` — whether `structure`'s field `field` is marked `@soa(flatten)`.
    pub fn soa_flattened(&self, structure: StructId, field: Symbol) -> bool {
        let def = self.struct_def(structure);
        let declared = def.origin.as_ref().map_or(def.name, |(origin, _)| *origin);
        self.soa_flattened.contains(&(declared, field))
    }

    /// `[ARN-11]` — record a struct's or an enum's `@derive(Zeroable)`; each instance is `Zeroable`
    /// when every field of it is (of an enum, of its variant whose discriminant is zero).
    pub fn derive_zeroable(&mut self, name: Symbol) {
        self.zeroable_derived.insert(name);
    }

    /// `[ARN-11]`, `[ARN-12]` — whether an all-zero object representation is
    /// a valid initialized value of this type.
    ///
    /// This is intentionally conservative. It contains only representation
    /// facts the compiler can prove without consulting a user implementation;
    /// general `@derive(Zeroable)` remains a later extension. In particular,
    /// a raw pointer may be null, but a reference, view or callable may not be
    /// manufactured from zero bytes. A range is zeroable exactly when zero is
    /// inside its declared validity interval.
    pub fn is_builtin_zeroable(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            TyKind::Const(_) => true,
            TyKind::Bool
            | TyKind::Char
            | TyKind::Int(_)
            | TyKind::Uint(_)
            | TyKind::Float(_)
            | TyKind::Ptr { .. } => true,
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => self.is_builtin_zeroable(*elem),
            TyKind::Tuple(items) => items.iter().all(|&item| self.is_builtin_zeroable(item)),
            // The compiler-known `MaybeUninit[T]` wrapper accepts every byte
            // pattern because it makes no validity claim about its payload.
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                if def.name.as_str().starts_with("MaybeUninit_") {
                    return !def.drops_fields;
                }
                // `[ARN-11]` — a struct only by `@derive(Zeroable)`, which
                // the compiler proves field by field: its invariants (a
                // `NonZero`'s) are the author's to vouch for.
                let declared = def.origin.as_ref().map_or(def.name, |(origin, _)| *origin);
                self.zeroable_derived.contains(&declared)
                    && !def.has_drop
                    && def.fields.iter().all(|field| self.is_builtin_zeroable(field.ty))
            }
            // `[ARN-11]` — an enum only by `@derive(Zeroable)`: zero bytes are the variant whose
            // discriminant is zero, with every field of it zero.
            TyKind::Enum(id) => {
                let def = self.enum_def(*id);
                let declared = def.origin.as_ref().map_or(def.name, |(origin, _)| *origin);
                self.zeroable_derived.contains(&declared)
                    && !def.has_drop
                    && def
                        .variants
                        .iter()
                        .find(|variant| variant.discriminant == 0)
                        .is_some_and(|variant| variant.fields.iter().all(|field| self.is_builtin_zeroable(field.ty)))
            }
            // A class handle is non-null; zero bytes cannot manufacture a
            // valid live object handle.
            TyKind::Class(_) | TyKind::ClassInterface(_) => false,
            TyKind::Range(id) => {
                let def = self.range_def(*id);
                match (def.lo, def.hi) {
                    (Bound::Int(lo), Bound::Int(hi)) => lo <= 0 && 0 <= hi,
                    (Bound::Float(lo), Bound::Float(hi)) => lo <= 0.0 && 0.0 <= hi,
                    _ => false,
                }
            }
            // The runtime representation of an empty growable buffer is
            // `{null, 0, 0}`. It is valid even though the value owns storage
            // once grown and therefore is not Copy.
            TyKind::Vec { .. } => true,
            TyKind::Error => true,
            TyKind::Void
            | TyKind::Never
            | TyKind::Str
            | TyKind::CStr
            | TyKind::Span { .. }
            | TyKind::Ref { .. }
            | TyKind::Fn { .. }
            | TyKind::Param { .. }
            | TyKind::Assoc { .. }
            | TyKind::Dyn { .. }
            | TyKind::Opaque(_)
            | TyKind::Infer(_)
            | TyKind::IntLit
            | TyKind::FloatLit => false,
        }
    }

    /// `[TYP-1]` — whether a value of this type needs destruction. Drives drop
    /// elaboration (Part XVIII §4.9).
    pub fn needs_drop(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                // `[WK-2]` — compiler-known weak handles are Copy wrappers
                // with custom control-block release glue. `has_drop` stays
                // false so Copy remains valid; their canonical origin is the
                // separate signal that drop elaboration must run.
                matches!(
                    &def.origin,
                    Some((name, args))
                        if (name.is("Weak") || name.is("Shared") || name.is("SyncShared")) && args.len() == 1
                ) || def.has_drop
                    || (def.drops_fields && def.fields.iter().any(|f| self.needs_drop(f.ty)))
            }
            // Dropping a class handle releases its strong reference.
            TyKind::Class(_) | TyKind::ClassInterface(_) => true,
            TyKind::Enum(id) => {
                let def = self.enum_def(*id);
                def.has_drop
                    || def
                        .variants
                        .iter()
                        .any(|v| v.fields.iter().any(|f| self.needs_drop(f.ty)))
            }
            TyKind::Tuple(items) => items.iter().any(|&t| self.needs_drop(t)),
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => self.needs_drop(*elem),
            // An `Array[T]` or a `String` always owns a heap buffer, whatever
            // the element type is.
            TyKind::Vec { .. } => true,
            TyKind::Dyn { .. } => true,
            // `[CLO-10]` — whatever state it captured ends with it.
            TyKind::Fn { abi: None, stateless: false, .. } => true,
            _ => false,
        }
    }

    /// `[CLO-1]` — a lambda's own type: the environment struct the checker
    /// makes for a capturing lambda, `closure{n}_env`.
    pub fn is_closure_environment(&self, ty: Ty) -> bool {
        matches!(self.kind(ty), TyKind::Struct(id) if {
            let name = self.struct_def(*id).name;
            name.as_str().starts_with("closure") && name.as_str().ends_with("_env")
        })
    }

    /// `[TYP-14]` — a type carrying a borrow. View types may not be stored in
    /// class fields, statics, containers or across threads (`[TYP-15]`).
    pub fn is_view(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            // Part IV §1's View category, in full.
            TyKind::Ref { .. } | TyKind::Str | TyKind::CStr | TyKind::Span { .. } => true,
            TyKind::Tuple(items) => items.iter().any(|&t| self.is_view(t)),
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => self.is_view(*elem),
            TyKind::Struct(id) => {
                self.struct_def(*id).fields.iter().any(|f| self.is_view(f.ty))
            }
            // The handle itself is not a view; a projected field may be a
            // view and is classified when that field's type is inspected.
            TyKind::Class(_) | TyKind::ClassInterface(_) => false,
            TyKind::Dyn { .. } => false,
            TyKind::Enum(id) => self
                .enum_def(*id)
                .variants
                .iter()
                .any(|v| v.fields.iter().any(|f| self.is_view(f.ty))),
            _ => false,
        }
    }

    /// Whether a view reachable in `ty`, as [`TypeTable::is_view`] walks it,
    /// lends mutably: a `ref mut` or a `MutSpan`, or one inside a tuple, a
    /// fixed array, a struct or an enum's payload. A shared `ref` lends
    /// nothing mutably, whatever it points at.
    pub fn has_mutable_view(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            TyKind::Ref { mutable, .. } | TyKind::Span { mutable, .. } => *mutable,
            TyKind::Tuple(items) => items.iter().any(|&t| self.has_mutable_view(t)),
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => self.has_mutable_view(*elem),
            TyKind::Struct(id) => self.struct_def(*id).fields.iter().any(|f| self.has_mutable_view(f.ty)),
            TyKind::Enum(id) => {
                self.enum_def(*id).variants.iter().any(|v| v.fields.iter().any(|f| self.has_mutable_view(f.ty)))
            }
            _ => false,
        }
    }

    /// Whether a type may cross a C boundary unchanged (`[FFI-5]`). Every
    /// scalar and every `@layout(c)` struct of FFI-safe fields qualifies.
    pub fn is_ffi_safe(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            TyKind::Bool
            | TyKind::Char
            | TyKind::Int(_)
            | TyKind::Uint(_)
            | TyKind::Float(_)
            | TyKind::Void
            | TyKind::Ptr { .. } => true,
            TyKind::CStr => true,
            TyKind::Fn { abi: Some(_), .. } => true,
            // `[FFI-8]` — a C enum is its `@repr` integer; an exhaustive enum has none, as C may
            // hand back a value no variant names (ODR-137).
            TyKind::Enum(id) if self.enum_def(*id).open => true,
            TyKind::Enum(id) => self.option_niche(*id).is_some_and(|niche| {
                matches!(self.kind(niche.payload), TyKind::Fn { abi: Some(_), .. } | TyKind::CStr)
            }),
            TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } => self.is_ffi_safe(*elem),
            TyKind::Struct(id) => {
                self.struct_def(*id).fields.iter().all(|f| self.is_ffi_safe(f.ty))
            }
            // Class handles are owning runtime values, not `@layout(c)`
            // value types. Opaque foreign handles have a separate boundary.
            TyKind::Class(_) | TyKind::ClassInterface(_) => false,
            TyKind::Dyn { .. } => false,
            _ => false,
        }
    }

    pub fn is_numeric(&self, ty: Ty) -> bool {
        matches!(
            self.kind(ty),
            TyKind::Int(_) | TyKind::Uint(_) | TyKind::Float(_) | TyKind::IntLit | TyKind::FloatLit
        )
    }

    pub fn is_integral(&self, ty: Ty) -> bool {
        matches!(self.kind(ty), TyKind::Int(_) | TyKind::Uint(_) | TyKind::IntLit)
    }

    pub fn is_float(&self, ty: Ty) -> bool {
        matches!(self.kind(ty), TyKind::Float(_) | TyKind::FloatLit)
    }

    /// `[LEX-16]`/`[LEX-17]` — a literal whose type context has not been seen
    /// yet. Resolved last, to `i32` or `f32` if nothing else decides.
    pub fn is_untyped_literal(&self, ty: Ty) -> bool {
        matches!(self.kind(ty), TyKind::IntLit | TyKind::FloatLit)
    }

    /// `[TYP-5]` — lossless widening, permitted at coercion sites only
    /// (assignment, argument, return, field initialiser, array element).
    /// Nothing converts to or from `bool` or `char` implicitly.
    pub fn widens_to(&self, from: Ty, to: Ty) -> bool {
        if from == to {
            return true;
        }
        let (lhs, rhs) = (self.kind(from), self.kind(to));
        match (lhs, rhs) {
            (TyKind::Int(a), TyKind::Int(b)) => int_bits(*a, self.pointer_size) < int_bits(*b, self.pointer_size),
            (TyKind::Uint(a), TyKind::Uint(b)) => {
                uint_bits(*a, self.pointer_size) < uint_bits(*b, self.pointer_size)
            }
            // `uN -> iM` only when the wider signed type can hold every value.
            (TyKind::Uint(a), TyKind::Int(b)) => {
                uint_bits(*a, self.pointer_size) < int_bits(*b, self.pointer_size)
            }
            (TyKind::Float(FloatTy::F32), TyKind::Float(FloatTy::F64)) => true,
            (TyKind::Float(FloatTy::F16), TyKind::Float(FloatTy::F32 | FloatTy::F64)) => true,
            _ => false,
        }
    }

    /// Render a type the way a diagnostic should show it.
    /// `[TYP-35]` — record a generic instance's phantom arguments.
    pub fn set_phantoms(&mut self, ty: Ty, args: Vec<Ty>) {
        if !args.is_empty() {
            self.phantoms.insert(ty, args);
        }
    }

    /// `[TYP-35]` — a generic instance's phantom arguments: each counts for
    /// `Send` and `Sync` as a field of its type would.
    pub fn phantoms(&self, ty: Ty) -> &[Ty] {
        self.phantoms.get(&ty).map_or(&[], Vec::as_slice)
    }

    /// `[TYP-32]` — a `some` wrapper's `Display` and `Debug`: what it holds, where its interface
    /// gives it them.
    pub fn set_some_prints(&mut self, id: StructId, display: bool, debug: bool) {
        self.some_prints.insert(id, (display, debug));
    }

    /// `[TYP-32]` — whether `ty` is a `some` wrapper, and if so whether it shows and debug-prints.
    pub fn some_prints(&self, ty: Ty) -> Option<(bool, bool)> {
        let TyKind::Struct(id) = *self.kind(ty) else { return None };
        self.some_prints.get(&id).copied()
    }

    /// `[TYP-32]` — show the struct `id` to the user as `shown` (`some Iterator[Item = i64]`).
    pub fn set_shown_as(&mut self, id: StructId, shown: String) {
        self.shown_as.insert(id, shown);
    }

    /// D-232 — record how a compiler-built generic struct is written.
    pub fn set_written_as(&mut self, id: StructId, generic: &str, args: Vec<Ty>) {
        self.written_as.insert(id, (Symbol::intern(generic), args));
    }

    /// How a type is written in Ember, for the user (`[DIA-2]`): a generic
    /// instance is `Range[i64]`, and a prelude type has its prelude name.
    pub fn display(&self, ty: Ty) -> String {
        self.render(ty, true)
    }

    /// A class's name as the user writes it: a generic instance is its
    /// generic's name and arguments (`Bag[int]`), never its instance's
    /// symbol (`[DIA-23]`).
    pub fn class_display_name(&self, id: ClassId) -> String {
        let def = self.class_def(id);
        self.render_named(def.name, def.origin.as_ref(), true)
    }

    /// The type's compiler identity: a generic instance by its instance
    /// name (`std.core.Range_i64`). Symbols and generated C names are built
    /// from this, so they never move with how a diagnostic spells a type.
    pub fn symbol_name(&self, ty: Ty) -> String {
        self.render(ty, false)
    }

    /// A nominal type's name: for the user, a generic instance is its
    /// generic's name and arguments, and `std.core.` (the prelude) is left
    /// off; otherwise the declared or instance name.
    fn render_named(&self, name: Symbol, origin: Option<&(Symbol, Vec<Ty>)>, user: bool) -> String {
        if !user {
            return name.to_string();
        }
        // `root.` marks a root module's type named like a compiler-known one
        // (D-305); it is written `Box`.
        let source = |name: Symbol| {
            let name = name.as_str();
            name.strip_prefix("std.core.").or_else(|| name.strip_prefix("root.")).unwrap_or(name).to_string()
        };
        match origin {
            // A compiler-known type that is not generic (`std.thread.Scope`) is marked by an origin
            // with no arguments, and is written without brackets.
            Some((generic, args)) if args.is_empty() => source(*generic),
            Some((generic, args)) => {
                let args: Vec<String> = args.iter().map(|&arg| self.render(arg, user)).collect();
                format!("{}[{}]", source(*generic), args.join(", "))
            }
            None => source(name),
        }
    }

    fn render(&self, ty: Ty, user: bool) -> String {
        match self.kind(ty) {
            TyKind::Bool => "bool".into(),
            TyKind::Char => "char".into(),
            TyKind::Int(i) => match i {
                IntTy::I8 => "i8",
                IntTy::I16 => "i16",
                IntTy::I32 => "i32",
                IntTy::I64 => "i64",
                IntTy::I128 => "i128",
                IntTy::I256 => "i256",
                IntTy::Isize => "isize",
            }
            .into(),
            TyKind::Uint(u) => match u {
                UintTy::U8 => "u8",
                UintTy::U16 => "u16",
                UintTy::U32 => "u32",
                UintTy::U64 => "u64",
                UintTy::U128 => "u128",
                UintTy::U256 => "u256",
                UintTy::Usize => "usize",
            }
            .into(),
            TyKind::Float(f) => match f {
                FloatTy::F16 => "f16",
                FloatTy::F32 => "f32",
                FloatTy::F64 => "f64",
            }
            .into(),
            TyKind::Void => "void".into(),
            TyKind::Never => "!".into(),
            TyKind::Str => "str".into(),
            TyKind::CStr => "cstr".into(),
            TyKind::Span { elem, mutable } => {
                let name = if *mutable { "MutSpan" } else { "Span" };
                format!("{name}[{}]", self.render(*elem, user))
            }
            TyKind::Opaque(name) => self.render_named(*name, None, user),
            // `[DIA-2]` (D-555) — a closure's environment is the compiler's
            // name for a lambda's own type, never shown.
            TyKind::Struct(_) if user && self.is_closure_environment(ty) => "lambda".to_string(),
            TyKind::Struct(id) if user && self.shown_as.contains_key(id) => self.shown_as[id].clone(),
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                self.render_named(def.name, def.origin.as_ref().or(self.written_as.get(id)), user)
            }
            TyKind::Class(id) => {
                let def = self.class_def(*id);
                self.render_named(def.name, def.origin.as_ref(), user)
            }
            TyKind::ClassInterface(interface) => interface.to_string(),
            TyKind::Enum(id) => {
                // The prelude's `Option` and `Result` are built once per
                // payload type; they print as they are written.
                let def = self.enum_def(*id);
                if user && def.origin.is_some() {
                    return self.render_named(def.name, def.origin.as_ref(), user);
                }
                let name = def.name.as_str();
                // D-732 — the payloads are rendered only for `Option` and `Result`: rendering
                // every enum's never ended for one that holds itself (`Node(Box[Tree])`).
                let payload = |index: usize| {
                    def.variants.get(index).and_then(|v| v.fields.first()).map(|f| self.render(f.ty, user))
                };
                let named = |a: &str, b: &str| {
                    def.variants.len() == 2 && def.variants[0].name.is(a) && def.variants[1].name.is(b)
                };
                if name.starts_with("Option_") && named("None", "Some") && def.variants[0].fields.is_empty()
                    && let Some(inner) = payload(1)
                {
                    return format!("Option[{inner}]");
                }
                if name.starts_with("Result_") && named("Ok", "Err")
                    && let (Some(ok), Some(err)) = (payload(0), payload(1))
                {
                    return format!("Result[{ok}, {err}]");
                }
                self.render_named(def.name, None, user)
            }
            TyKind::Range(id) => self.range_def(*id).name.to_string(),
            // D-306 — an instance name holds the slot too: `Option[U]` over
            // a function's first parameter and over a method's second are two
            // types, and named alike they were one.
            TyKind::Param { name, index } if !user => format!("{name}{index}"),
            TyKind::Param { name, .. } => name.to_string(),
            TyKind::Assoc { name } => format!("Self.{name}"),
            TyKind::Dyn { interfaces } => format!(
                "dyn {}",
                interfaces
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" + ")
            ),
            TyKind::Vec { text: true, .. } => "String".into(),
            TyKind::Vec { elem, .. } => format!("Array[{}]", self.render(*elem, user)),
            TyKind::Tuple(items) => {
                let inner: Vec<String> = items.iter().map(|&t| self.render(t, user)).collect();
                format!("({})", inner.join(", "))
            }
            TyKind::Ref { mutable, inner } => {
                let kw = if *mutable { "ref mut " } else { "ref " };
                format!("{kw}{}", self.render(*inner, user))
            }
            TyKind::Ptr { mutable, inner } => {
                let kw = if *mutable { "*mut " } else { "*" };
                format!("{kw}{}", self.render(*inner, user))
            }
            TyKind::Array { elem, len } => format!("[{}; {len}]", self.render(*elem, user)),
            TyKind::ArrayN { elem, len } => format!("[{}; {len}]", self.render(*elem, user)),
            TyKind::Const(value) => value.to_string(),
            TyKind::Fn { abi, latebound, is_unsafe, params, ret, .. } => {
                let inner: Vec<String> = params
                    .iter()
                    .map(|param| {
                        let mode = match param.mode {
                            FnParamMode::Borrow => "",
                            FnParamMode::Mut => "mut ",
                            FnParamMode::Owned => "owned ",
                        };
                        format!("{mode}{}", self.render(param.ty, user))
                    })
                    .collect();
                let is_unsafe = if *is_unsafe { "unsafe " } else { "" };
                format!(
                    "{}fn({}) -> {}",
                    if let Some(abi) = abi { format!("{is_unsafe}extern \"{abi}\" ") }
                    else if *latebound { format!("@latebound {is_unsafe}") } else { is_unsafe.to_string() },
                    inner.join(", "),
                    self.render(*ret, user)
                )
            }
            TyKind::Infer(_) => "_".into(),
            // A diagnostic should never name these; if one does, say something
            // a reader recognises rather than an internal name.
            TyKind::IntLit => "an integer".into(),
            TyKind::FloatLit => "a float".into(),
            TyKind::Error => "<error>".into(),
        }
    }

    /// A stable semantic spelling for a resolved type in compiler artifacts.
    /// Unlike [`Self::display`], this is not diagnostic prose: generic
    /// parameters use their binder index, `Array[u8]` does not acquire the
    /// diagnostic-only `String` nickname, and unresolved placeholder kinds are
    /// rejected. Nominal names are the checker-owned qualified names stored in
    /// the type table, so two modules' `Thing` types cannot collide.
    pub fn canonical_name(&self, ty: Ty) -> Result<String, CanonicalTypeError> {
        Ok(match self.kind(ty) {
            TyKind::Bool => "bool".into(),
            TyKind::Char => "char".into(),
            TyKind::Int(kind) => match kind {
                IntTy::I8 => "i8",
                IntTy::I16 => "i16",
                IntTy::I32 => "i32",
                IntTy::I64 => "i64",
                IntTy::I128 => "i128",
                IntTy::I256 => "i256",
                IntTy::Isize => "isize",
            }
            .into(),
            TyKind::Uint(kind) => match kind {
                UintTy::U8 => "u8",
                UintTy::U16 => "u16",
                UintTy::U32 => "u32",
                UintTy::U64 => "u64",
                UintTy::U128 => "u128",
                UintTy::U256 => "u256",
                UintTy::Usize => "usize",
            }
            .into(),
            TyKind::Float(kind) => match kind {
                FloatTy::F16 => "f16",
                FloatTy::F32 => "f32",
                FloatTy::F64 => "f64",
            }
            .into(),
            TyKind::Void => "void".into(),
            TyKind::Never => "!".into(),
            TyKind::Str => "str".into(),
            TyKind::CStr => "cstr".into(),
            TyKind::Span { elem, mutable } => format!(
                "{}[{}]",
                if *mutable { "MutSpan" } else { "Span" },
                self.canonical_name(*elem)?
            ),
            TyKind::Struct(id) => self.struct_def(*id).name.to_string(),
            TyKind::Opaque(name) => name.to_string(),
            TyKind::Class(id) => self.class_def(*id).name.to_string(),
            TyKind::ClassInterface(interface) => interface.to_string(),
            TyKind::Enum(id) => self.enum_def(*id).name.to_string(),
            TyKind::Range(id) => self.range_def(*id).name.to_string(),
            TyKind::Tuple(items) => {
                let mut out = String::from("(");
                for (index, item) in items.iter().enumerate() {
                    if index != 0 {
                        out.push(',');
                    }
                    out.push_str(&self.canonical_name(*item)?);
                }
                if items.len() == 1 {
                    out.push(',');
                }
                out.push(')');
                out
            }
            TyKind::Dyn { interfaces } => format!(
                "dyn {}",
                interfaces
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("+")
            ),
            TyKind::Ref { mutable, inner } => format!(
                "{}{}",
                if *mutable { "ref mut " } else { "ref " },
                self.canonical_name(*inner)?
            ),
            TyKind::Ptr { mutable, inner } => format!(
                "{}{}",
                if *mutable { "*mut " } else { "*" },
                self.canonical_name(*inner)?
            ),
            TyKind::Array { elem, len } => {
                format!("[{};{len}]", self.canonical_name(*elem)?)
            }
            TyKind::ArrayN { elem, len } => format!("[{};{len}]", self.canonical_name(*elem)?),
            TyKind::Const(value) => format!("const {value}"),
            TyKind::Vec { text: true, .. } => "String".to_string(),
            TyKind::Vec { elem, .. } => format!("Array[{}]", self.canonical_name(*elem)?),
            TyKind::Fn { abi, latebound, stateless, is_unsafe, params, ret } => {
                let mut out = if let Some(abi) = abi {
                    format!("extern \"{abi}\" fn(")
                } else if *latebound {
                    String::from("@latebound fn(")
                } else if *stateless {
                    String::from("plain fn(")
                } else {
                    String::from("fn(")
                };
                if *is_unsafe {
                    out.insert_str(0, "unsafe ");
                }
                for (index, param) in params.iter().enumerate() {
                    if index != 0 {
                        out.push(',');
                    }
                    match param.mode {
                        FnParamMode::Borrow => {}
                        FnParamMode::Mut => out.push_str("mut "),
                        FnParamMode::Owned => out.push_str("owned "),
                    }
                    out.push_str(&self.canonical_name(param.ty)?);
                }
                out.push_str(")->");
                out.push_str(&self.canonical_name(*ret)?);
                out
            }
            TyKind::Param { index, .. } => format!("$P{index}"),
            TyKind::Assoc { name } => format!("Self.{name}"),
            TyKind::Infer(_) => return Err(CanonicalTypeError::InferenceVariable),
            TyKind::IntLit => return Err(CanonicalTypeError::IntegerLiteral),
            TyKind::FloatLit => return Err(CanonicalTypeError::FloatLiteral),
            TyKind::Error => return Err(CanonicalTypeError::Error),
        })
    }
}

impl fmt::Debug for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ty#{}", self.0)
    }
}

fn int_bits(ty: IntTy, pointer_size: u64) -> u64 {
    match ty {
        IntTy::I8 => 8,
        IntTy::I16 => 16,
        IntTy::I32 => 32,
        IntTy::I64 => 64,
        IntTy::I128 => 128,
        IntTy::I256 => 256,
        IntTy::Isize => pointer_size * 8,
    }
}

fn uint_bits(ty: UintTy, pointer_size: u64) -> u64 {
    match ty {
        UintTy::U8 => 8,
        UintTy::U16 => 16,
        UintTy::U32 => 32,
        UintTy::U64 => 64,
        UintTy::U128 => 128,
        UintTy::U256 => 256,
        UintTy::Usize => pointer_size * 8,
    }
}

/// Round `offset` up to a multiple of `align`.
pub fn align_to(offset: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two(), "alignment {align} is not a power of two");
    (offset + align - 1) & !(align - 1)
}

/// `[TYP-8]` — what an arithmetic overflow does.
///
/// Panic is the default in every profile. An explicit function attribute or
/// module directive selects wrapping or saturation for arithmetic operators;
/// shift-amount checks and named integer methods have fixed contracts.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum OverflowPolicy {
    /// Overflowing arithmetic operators panic.
    #[default]
    Panic,
    /// Arithmetic operators wrap two's-complement; shift amounts still check
    /// (`[TYP-10]`). Only `wrapping_shl/shr` methods mask their amounts.
    Wrap,
    /// Results clamp to the type's bounds.
    Saturate,
}

impl OverflowPolicy {
    pub fn from_name(name: &str) -> Option<OverflowPolicy> {
        Some(match name {
            "panic" => OverflowPolicy::Panic,
            "wrap" => OverflowPolicy::Wrap,
            "saturate" => OverflowPolicy::Saturate,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            OverflowPolicy::Panic => "panic",
            OverflowPolicy::Wrap => "wrap",
            OverflowPolicy::Saturate => "saturate",
        }
    }
}

/// `[TYP-9]` — how a function's floating point may be compiled: strict IEEE
/// 754 (the default), with a multiply and an add fused (`@fp(contract)`), or
/// with every relaxation `@fastmath` allows. `[CG-C-11]` emits each relaxed
/// function in a translation unit of its own, compiled with the matching
/// flags.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub enum FpMode {
    #[default]
    Strict,
    Contract,
    Fast,
}

impl FpMode {
    pub fn name(self) -> &'static str {
        match self {
            FpMode::Strict => "strict",
            FpMode::Contract => "contract",
            FpMode::Fast => "fastmath",
        }
    }
}

/// Whether an integer type is signed. `None` for anything that is not an
/// integer.
pub fn is_signed(table: &TypeTable, ty: Ty) -> Option<bool> {
    match table.kind(ty) {
        TyKind::Int(_) => Some(true),
        TyKind::Uint(_) => Some(false),
        _ => None,
    }
}

/// `[SOA-1]` — one column of an `SoA[T]` (`TypeTable::soa_columns`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SoaColumn {
    /// The field indices from `T` to the column's field.
    pub path: Vec<usize>,
    /// The field names joined by `__` (`pos__x`).
    pub name: String,
    pub ty: Ty,
}

/// `[SIMD-1]` — what a lane of a `std.simd` type is.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SimdLane {
    Float,
    Signed,
    Unsigned,
    /// A mask's lane: all ones (set) or all zeros (clear).
    Mask,
}

/// `[SIMD-1]` — the shape of a `std.simd` vector or mask type, read from its
/// name (`std.simd.f32x8`, `std.simd.mask32x8`): the lane kind, the lane's
/// width in bits, and the lane count.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SimdShape {
    pub lane: SimdLane,
    pub bits: u32,
    pub lanes: u32,
}

impl SimdShape {
    pub fn of(qualified_name: &str) -> Option<SimdShape> {
        let name = qualified_name.strip_prefix("std.simd.")?;
        let (lane, rest) = if let Some(rest) = name.strip_prefix("mask") {
            (SimdLane::Mask, rest)
        } else {
            let lane = match name.as_bytes().first()? {
                b'f' => SimdLane::Float,
                b'i' => SimdLane::Signed,
                b'u' => SimdLane::Unsigned,
                _ => return None,
            };
            (lane, &name[1..])
        };
        let (bits, lanes) = rest.split_once('x')?;
        let (bits, lanes): (u32, u32) = (bits.parse().ok()?, lanes.parse().ok()?);
        let width = bits * lanes;
        let valid = matches!(bits, 8 | 16 | 32 | 64)
            && matches!(width, 128 | 256 | 512)
            && (lane != SimdLane::Float || matches!(bits, 32 | 64));
        valid.then_some(SimdShape { lane, bits, lanes })
    }

    /// Its size in bytes, which is also its alignment.
    pub fn bytes(self) -> u32 {
        self.bits * self.lanes / 8
    }

    /// The mask type's name for a vector of this shape (`mask32x8`).
    pub fn mask_name(self) -> String {
        format!("std.simd.mask{}x{}", self.bits, self.lanes)
    }
}

/// The width of an integer type in bits, for `[TYP-10]`'s shift check.
pub fn bit_width(table: &TypeTable, ty: Ty) -> Option<u64> {
    match table.kind(ty) {
        TyKind::Int(i) => Some(int_bits(*i, table.pointer_size())),
        TyKind::Uint(u) => Some(uint_bits(*u, table.pointer_size())),
        _ => None,
    }
}

/// The most negative value a signed integer type can hold, as a magnitude.
///
/// `[TYP-8]` — `i32.MIN / -1` always panics, whatever the overflow policy is,
/// because the true result is not representable.
pub fn signed_min_magnitude(table: &TypeTable, ty: Ty) -> Option<u128> {
    match table.kind(ty) {
        // G8-4 — an `i256` constant is a sign-extended 128-bit one, so the
        // least a literal or constant of it can be is `i128`'s.
        TyKind::Int(IntTy::I256) => Some(1u128 << 127),
        TyKind::Int(i) => Some(1u128 << (int_bits(*i, table.pointer_size()) - 1)),
        _ => None,
    }
}

/// The largest value an integer type can hold, for `[LEX-16]`'s range check.
pub fn int_max(table: &TypeTable, ty: Ty) -> Option<u128> {
    match table.kind(ty) {
        // G8-4 — the most a literal or constant of an `i256` can be: one
        // is a sign-extended 128-bit constant.
        TyKind::Int(IntTy::I256) => Some(i128::MAX as u128),
        TyKind::Int(i) => {
            let bits = int_bits(*i, table.pointer_size());
            Some((1u128 << (bits - 1)) - 1)
        }
        TyKind::Uint(u) => {
            // G8-4 — a `u256` constant is a zero-extended 128-bit one.
            let bits = uint_bits(*u, table.pointer_size());
            Some(if bits >= 128 { u128::MAX } else { (1u128 << bits) - 1 })
        }
        _ => None,
    }
}

/// D-316, `[TYP-9]` — the `f16` nearest `value`, ties to even, as its bits:
/// what the runtime's `ember_f64_to_f16` computes, for a constant. Past the
/// largest finite `f16` it is the infinity of `value`'s sign; NaN is a quiet
/// NaN.
pub fn f16_bits(value: f64) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 48) & 0x8000) as u16;
    let biased = ((bits >> 52) & 0x7FF) as i32;
    let mantissa = bits & 0x000F_FFFF_FFFF_FFFF;
    if biased == 0x7FF {
        if mantissa == 0 {
            return sign | 0x7C00;
        }
        return sign | 0x7E00 | ((mantissa >> 42) & 0x3FF) as u16;
    }
    if biased == 0 {
        // Zero, or a subnormal `f64`: far below half the least `f16`.
        return sign;
    }
    let e = biased - 1023;
    if e > 15 {
        return sign | 0x7C00;
    }
    // The value is `significand * 2^(e - 52)`; the `f16` quantum is
    // `2^(e - 10)` for a normal result and `2^-24` for a subnormal one.
    let significand = mantissa | (1 << 52);
    let quantum = if e >= -14 { e - 10 } else { -24 };
    let shift = quantum - (e - 52);
    if shift >= 54 {
        // Below half the least subnormal, 2^-25.
        return sign;
    }
    let mut q = significand >> shift;
    let rest = significand & ((1u64 << shift) - 1);
    let half = 1u64 << (shift - 1);
    if rest > half || (rest == half && q & 1 == 1) {
        q += 1;
    }
    if e < -14 {
        // A subnormal; a carry into bit 10 is the least normal, as it should be.
        return sign | q as u16;
    }
    let mut exponent = e + 15;
    if q == 1 << 11 {
        q >>= 1;
        exponent += 1;
    }
    if exponent >= 31 {
        return sign | 0x7C00;
    }
    sign | ((exponent as u16) << 10) | (q & 0x3FF) as u16
}

/// D-319, `[LEX-17]` — the `f16` nearest the decimal literal `text` (digits,
/// an optional fraction and exponent, no sign), ties to even, rounded once.
/// Rounding the literal's `f64` again can go wrong only where that `f64` is
/// exactly halfway between two `f16`s while the decimal is not; then the
/// decimal is compared with the midpoint exactly.
pub fn f16_bits_of_decimal(text: &str) -> u16 {
    let Ok(value) = text.parse::<f64>() else { return f16_bits(0.0) };
    let bits = f16_bits(value);
    if !value.is_finite() || value < 0.0 || bits & 0x7FFF == 0 && value == 0.0 {
        return bits;
    }
    let rounded = f16_value(bits);
    if rounded == value {
        return bits;
    }
    // The two `f16`s either side of `value` (the one past the largest finite
    // is 65536, which rounds to infinity).
    let (low, high) = if rounded < value {
        (bits, bits + 1)
    } else {
        (bits - 1, bits)
    };
    let value_of = |b: u16| if b == 0x7C00 { 65536.0 } else { f16_value(b) };
    if value - value_of(low) != value_of(high) - value {
        return bits;
    }
    match compare_decimal(text, value) {
        Some(std::cmp::Ordering::Greater) => high,
        Some(std::cmp::Ordering::Less) => low,
        _ => bits,
    }
}

/// The decimal `text` compared with `value` exactly (both non-negative).
fn compare_decimal(text: &str, value: f64) -> Option<std::cmp::Ordering> {
    let (mantissa, exponent) = match text.find(['e', 'E']) {
        Some(at) => (&text[..at], text[at + 1..].parse::<i64>().ok()?),
        None => (text, 0),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let mut decimal = BigNat::default();
    for c in whole.chars().chain(fraction.chars()) {
        decimal.mul_add(10, c.to_digit(10)?);
    }
    let ten_power = exponent - fraction.len() as i64;
    // `value` is `significand * 2^two_power` exactly.
    let raw = value.to_bits();
    let biased = ((raw >> 52) & 0x7FF) as i64;
    let (significand, two_power) = if biased == 0 {
        (raw & 0x000F_FFFF_FFFF_FFFF, -1074)
    } else {
        ((raw & 0x000F_FFFF_FFFF_FFFF) | (1 << 52), biased - 1075)
    };
    let mut binary = BigNat::default();
    binary.mul_add(1 << 26, (significand >> 26) as u32);
    binary.mul_add(1 << 26, (significand & ((1 << 26) - 1)) as u32);
    // decimal * 10^ten_power against binary * 2^two_power, in integers.
    if ten_power >= 0 {
        decimal.mul_pow(10, ten_power as u64);
    } else {
        binary.mul_pow(10, (-ten_power) as u64);
    }
    if two_power >= 0 {
        binary.mul_pow(2, two_power as u64);
    } else {
        decimal.mul_pow(2, (-two_power) as u64);
    }
    Some(decimal.cmp_with(&binary))
}

/// Just enough of an unbounded natural number for [`compare_decimal`].
#[derive(Default)]
struct BigNat {
    /// Base 2^32, least significant first.
    limbs: Vec<u32>,
}

impl BigNat {
    fn mul_add(&mut self, factor: u32, addend: u32) {
        let mut carry = u64::from(addend);
        for limb in &mut self.limbs {
            let product = u64::from(*limb) * u64::from(factor) + carry;
            *limb = product as u32;
            carry = product >> 32;
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
    }

    fn mul_pow(&mut self, base: u32, mut times: u64) {
        while times > 0 {
            self.mul_add(base, 0);
            times -= 1;
        }
    }

    fn cmp_with(&self, other: &BigNat) -> std::cmp::Ordering {
        let trim = |limbs: &[u32]| limbs.len() - limbs.iter().rev().take_while(|&&limb| limb == 0).count();
        let (a, b) = (trim(&self.limbs), trim(&other.limbs));
        a.cmp(&b).then_with(|| self.limbs[..a].iter().rev().cmp(other.limbs[..b].iter().rev()))
    }
}

/// D-316 — the value of the `f16` whose bits are `bits`, exactly.
pub fn f16_value(bits: u16) -> f64 {
    let sign = if bits & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exponent = i32::from((bits >> 10) & 0x1F);
    let mantissa = f64::from(bits & 0x3FF);
    match exponent {
        0x1F if mantissa == 0.0 => sign * f64::INFINITY,
        0x1F => f64::NAN,
        0 => sign * mantissa / 16_777_216.0,
        _ => sign * (1024.0 + mantissa) * 2f64.powi(exponent - 25),
    }
}

/// D-316 — the fewest digits that read back as the `f16` whose bits are
/// `bits` (five at most), as W2015 names the value a literal becomes.
pub fn f16_text(bits: u16) -> String {
    let value = f16_value(bits);
    for digits in 1..=5 {
        let text = format!("{value:.*e}", digits - 1);
        if let Ok(back) = text.parse::<f64>()
            && f16_bits(back) == bits
        {
            return back.to_string();
        }
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ember_span::Span;

    #[test]
    fn an_enum_niche_stays_inside_its_integer_representation() {
        let (mut table, _) = TypeTable::new();
        let variants = |count: i128| {
            (0..count)
                .map(|value| VariantDef {
                    name: Symbol::intern(&format!("V{value}")),
                    fields: Vec::new(),
                    discriminant: value,
                    span: Span::DUMMY,
                })
                .collect()
        };
        let signed = table.intern(TyKind::Int(IntTy::I8));
        let signed_id = table.add_enum(EnumDef {
            name: Symbol::intern("SignedFullPositive"),
            variants: variants(128),
            span: Span::DUMMY,
            origin: None,
            repr: signed,
            repr_is_explicit: true,
            derives_copy: false,
            has_drop: false,
            open: false,
        });
        assert_eq!(table.enum_unused_discriminant(signed_id), Some(-1));

        let unsigned = table.intern(TyKind::Uint(UintTy::U8));
        let unsigned_id = table.add_enum(EnumDef {
            name: Symbol::intern("UnsignedFull"),
            variants: variants(256),
            span: Span::DUMMY,
            origin: None,
            repr: unsigned,
            repr_is_explicit: true,
            derives_copy: false,
            has_drop: false,
            open: false,
        });
        assert_eq!(table.enum_unused_discriminant(unsigned_id), None);
    }

    #[test]
    fn f16_text_is_the_fewest_digits_that_read_back() {
        assert_eq!(f16_text(f16_bits(0.123456)), "0.1235");
        assert_eq!(f16_text(f16_bits(0.1)), "0.1");
        assert_eq!(f16_text(0x7BFF), "65500");
        assert_eq!(f16_text(0x3C01), "1.001");
        for bits in 0..0x7C00u16 {
            let back: f64 = f16_text(bits).parse().unwrap();
            assert_eq!(f16_bits(back), bits, "{bits:#06x}");
        }
    }

    #[test]
    fn f16_of_a_decimal_is_rounded_once() {
        // 1 + 2^-11 is halfway between 1 and the next `f16`, 1 + 2^-10.
        assert_eq!(f16_bits_of_decimal("1.00048828125"), 0x3C00);
        // Just above the midpoint, but not by enough for an `f64` to tell:
        // its `f64` is the midpoint itself, which rounds to even (down).
        assert_eq!(f16_bits("1.00048828125000000000001".parse().unwrap()), 0x3C00);
        assert_eq!(f16_bits_of_decimal("1.00048828125000000000001"), 0x3C01);
        assert_eq!(f16_bits_of_decimal("1.00048828124999999999999"), 0x3C00);
        // Halfway between the largest finite `f16` and 65536: infinity only
        // at or past the midpoint.
        assert_eq!(f16_bits_of_decimal("65520"), 0x7C00);
        assert_eq!(f16_bits_of_decimal("65519.99999999999999999"), 0x7BFF);
        assert_eq!(f16_bits_of_decimal("0.1"), f16_bits(0.1));
        assert_eq!(f16_bits_of_decimal("2.5e-1"), f16_bits(0.25));
        assert_eq!(f16_bits_of_decimal("0"), 0);
    }

    #[test]
    fn f16_bits_rounds_to_nearest_even() {
        // Every `f16` is its own nearest.
        for bits in 0..=u16::MAX {
            let value = f16_value(bits);
            if value.is_nan() {
                assert_eq!(f16_bits(value) & 0x7C00, 0x7C00);
                continue;
            }
            assert_eq!(f16_bits(value), bits, "{bits:#06x}");
        }
        // Halfway between two `f16`s goes to the even one; just past it, up.
        assert_eq!(f16_bits(1.0 + f64::powi(2.0, -11)), 0x3C00);
        assert_eq!(f16_bits(1.0 + 3.0 * f64::powi(2.0, -11)), 0x3C02);
        assert_eq!(f16_bits(f64::from_bits((1.0 + f64::powi(2.0, -11)).to_bits() + 1)), 0x3C01);
        // The largest finite `f16` is 65504; 65520 is halfway to 2^16 and
        // rounds to infinity; half the least subnormal rounds to zero.
        assert_eq!(f16_bits(65519.0), 0x7BFF);
        assert_eq!(f16_bits(65520.0), 0x7C00);
        assert_eq!(f16_bits(-1e300), 0xFC00);
        assert_eq!(f16_bits(f64::powi(2.0, -25)), 0x0000);
        assert_eq!(f16_bits(f64::powi(2.0, -25) * 1.0000001), 0x0001);
        assert_eq!(f16_bits(0.1), 0x2E66);
        assert_eq!(f16_bits(-0.0), 0x8000);
    }

    fn field(name: &str, ty: Ty) -> FieldDef {
        FieldDef {
            name: Symbol::intern(name),
            ty,
            span: Span::DUMMY,
            ty_span: Span::DUMMY,
            has_default: false,
            is_let: false,
            read_only_outside: false,
            vis: FieldVis::Public,
        }
    }

    #[test]
    fn interning_gives_the_same_type_the_same_identity() {
        let (mut table, common) = TypeTable::new();
        assert_eq!(table.intern(TyKind::Int(IntTy::I32)), common.i32);
        assert_ne!(common.i32, common.u32);
    }

    #[test]
    fn explicit_sync_handle_has_thread_properties_and_no_access_words() {
        let (mut table, c) = TypeTable::new();
        let array = table.intern(TyKind::Vec { elem: c.i64, text: false });
        let local = table.add_class(ClassDef {
            name: Symbol::intern("Local"), fields: vec![field("value", array)],
            span: Span::DUMMY, is_sync: false, openness: ClassOpenness::Final,
            base: None, has_drop: false, origin: None, declaring_module: 0,
        });
        let shared = table.add_class(ClassDef {
            name: Symbol::intern("Shared"), fields: vec![field("value", array)],
            span: Span::DUMMY, is_sync: true, openness: ClassOpenness::Final,
            base: None, has_drop: false, origin: None, declaring_module: 0,
        });
        let local_ty = table.intern(TyKind::Class(local));
        let shared_ty = table.intern(TyKind::Class(shared));
        let ptr = table.intern(TyKind::Ptr { mutable: false, inner: c.i64 });
        assert!(!table.is_send(local_ty) && !table.is_sync(local_ty));
        assert!(table.is_send(shared_ty) && table.is_sync(shared_ty));
        assert!(!table.is_send(ptr) && !table.is_sync(ptr));
        assert!(table.class_field_has_access_word(local, 0));
        assert!(!table.class_field_has_access_word(shared, 0));
    }

    #[test]
    fn canonical_artifact_names_are_resolved_and_not_diagnostic_aliases() {
        let (mut table, common) = TypeTable::new();
        let bytes = table.intern(TyKind::Vec { elem: common.u8, text: false });
        let span = table.intern(TyKind::Span {
            elem: common.i32,
            mutable: false,
        });
        let unresolved = table.intern(TyKind::Infer(InferId(17)));

        // `display` preserves source-friendly aliases such as `String`; an
        // interface identity must retain the actual semantic constructor.
        assert_eq!(table.canonical_name(bytes).unwrap(), "Array[u8]");
        // D-201 — `String` is a type of its own, not `Array[u8]`.
        assert_eq!(table.canonical_name(common.string).unwrap(), "String");
        assert_eq!(table.display(common.string), "String");
        assert_eq!(table.display(bytes), "Array[u8]");
        assert_eq!(table.canonical_name(span).unwrap(), "Span[i32]");
        assert_eq!(
            table.canonical_name(unresolved),
            Err(CanonicalTypeError::InferenceVariable)
        );
    }

    #[test]
    fn scalar_layouts_match_the_table_in_part_iv() {
        let (table, c) = TypeTable::new();
        assert_eq!(table.layout(c.bool_), Layout::scalar(1));
        assert_eq!(table.layout(c.char_).size, 4);
        assert_eq!(table.layout(c.i128).size, 16);
        assert_eq!(table.layout(c.f16).size, 2);
        assert_eq!(table.layout(c.usize).size, 8);
        assert_eq!(table.layout(c.void), Layout::ZERO);
        assert_eq!(table.layout(c.never), Layout::ZERO);
    }

    #[test]
    fn a_struct_uses_c_layout_in_declaration_order() {
        // [TYP-11] — no reordering, natural alignment, trailing padding.
        let (mut table, c) = TypeTable::new();
        let id = table.add_struct(StructDef {
            name: Symbol::intern("Vec3"),
            fields: vec![field("x", c.f32), field("y", c.f32), field("z", c.f32)],
            span: Span::DUMMY,
            derives_copy: true,
            has_drop: false,
            drops_fields: true,
            origin: None,
            declaring_module: 0,
        });
        let ty = table.intern(TyKind::Struct(id));
        let layout = table.layout(ty);
        assert_eq!(layout.size, 12);
        assert_eq!(layout.align, 4);
        assert_eq!(layout.field_offsets, vec![0, 4, 8]);
    }

    #[test]
    fn class_identity_is_a_pointer_sized_counted_handle() {
        // `[OBJ-1]`, `[OBJ-2]`, `[OWN-7]`, and `[DRP-6]`: the source-level
        // class value is a handle, while the heap object's fields are not
        // folded into the handle layout or mistaken for a raw pointer.
        let (mut table, c) = TypeTable::new();
        let base = table.add_class(ClassDef {
            is_sync: false,
            name: Symbol::intern("Entity"),
            fields: vec![field("id", c.u64)],
            span: Span::DUMMY,
            openness: ClassOpenness::Open,
            base: None,
            has_drop: false,
            origin: None,
            declaring_module: 0,
        });
        let derived = table.add_class(ClassDef {
            is_sync: false,
            name: Symbol::intern("Player"),
            fields: vec![field("health", c.f32)],
            span: Span::DUMMY,
            openness: ClassOpenness::Final,
            base: Some(base),
            has_drop: true,
            origin: None,
            declaring_module: 0,
        });
        let entity = table.intern(TyKind::Class(base));
        let player = table.intern(TyKind::Class(derived));

        assert_eq!(table.layout(player), Layout::scalar(8));
        assert_eq!(table.display(player), "Player");
        assert_eq!(table.canonical_name(player).unwrap(), "Player");
        assert!(table.is_copy(entity));
        assert!(table.is_copy(player));
        assert!(table.needs_drop(entity));
        assert!(table.needs_drop(player));
        assert!(!table.is_view(player));
        assert!(!table.is_ffi_safe(player));
        assert!(!table.is_builtin_zeroable(player));
        assert_ne!(entity, player, "distinct classes must not collapse to pointer identity");
        let (base_index, base_field) = table.class_field(derived, Symbol::intern("id")).unwrap();
        assert_eq!(base_index, 0);
        assert_eq!(base_field.ty, c.u64);
        let (derived_index, derived_field) =
            table.class_field(derived, Symbol::intern("health")).unwrap();
        assert_eq!(derived_index, 1);
        assert_eq!(derived_field.ty, c.f32);
        assert_eq!(table.class_field_count(derived), 2);
        assert_eq!(table.class_field_at(derived, 0).unwrap().name, Symbol::intern("id"));
        assert_eq!(table.class_field_at(derived, 1).unwrap().name, Symbol::intern("health"));
        assert!(table.class_is_subclass_of(derived, base));
        assert!(!table.class_is_subclass_of(base, derived));
    }

    #[test]
    fn padding_follows_the_c_abi_not_field_order_optimisation() {
        // { u8, u32, u8 } is 12 bytes in C, not the 8 a reordering layout
        // would give. [TYP-11] chooses C compatibility deliberately.
        let (mut table, c) = TypeTable::new();
        let id = table.add_struct(StructDef {
            name: Symbol::intern("S"),
            fields: vec![field("a", c.u8), field("b", c.u32), field("c", c.u8)],
            span: Span::DUMMY,
            derives_copy: true,
            has_drop: false,
            drops_fields: true,
            origin: None,
            declaring_module: 0,
        });
        let ty = table.intern(TyKind::Struct(id));
        let layout = table.layout(ty);
        assert_eq!(layout.field_offsets, vec![0, 4, 8]);
        assert_eq!(layout.size, 12);
        assert_eq!(layout.align, 4);
    }

    #[test]
    fn copy_is_never_implicit_for_a_struct() {
        // [STR-3] — adding a field later must not silently change semantics.
        let (mut table, c) = TypeTable::new();
        let plain = table.add_struct(StructDef {
            name: Symbol::intern("Plain"),
            fields: vec![field("x", c.f32)],
            span: Span::DUMMY,
            derives_copy: false,
            has_drop: false,
            drops_fields: true,
            origin: None,
            declaring_module: 0,
        });
        let ty = table.intern(TyKind::Struct(plain));
        assert!(!table.is_copy(ty), "a struct without @derive(Copy) is not Copy");
        table.struct_def_mut(plain).derives_copy = true;
        assert!(table.is_copy(ty));
    }

    #[test]
    fn a_type_with_drop_is_move_only() {
        // [STR-3]
        let (mut table, c) = TypeTable::new();
        let id = table.add_struct(StructDef {
            name: Symbol::intern("File"),
            fields: vec![field("fd", c.i32)],
            span: Span::DUMMY,
            derives_copy: true,
            has_drop: true,
            drops_fields: true,
            origin: None,
            declaring_module: 0,
        });
        let ty = table.intern(TyKind::Struct(id));
        assert!(!table.is_copy(ty));
        assert!(table.needs_drop(ty));
    }

    #[test]
    fn a_no_drop_storage_wrapper_keeps_layout_and_copy_but_suppresses_field_drop() {
        // `[ARN-8]` — this is the representation contract used by
        // compiler-known `MaybeUninit[T]`: one field gives it exactly `T`'s
        // layout and ordinary Copy derivation, while `drops_fields` prevents
        // bytes that may not hold an initialized `T` from being destroyed.
        let (mut table, c) = TypeTable::new();
        let owning = table.intern(TyKind::Vec { elem: c.i32, text: false });
        let wrapper = table.add_struct(StructDef {
            name: Symbol::intern("MaybeUninit_Array_i32"),
            fields: vec![field("value", owning)],
            span: Span::DUMMY,
            derives_copy: true,
            has_drop: false,
            drops_fields: false,
            origin: None,
            declaring_module: usize::MAX,
        });
        let wrapped = table.intern(TyKind::Struct(wrapper));
        let wrapped_layout = table.layout(wrapped);
        let owning_layout = table.layout(owning);
        assert_eq!(wrapped_layout.size, owning_layout.size);
        assert_eq!(wrapped_layout.align, owning_layout.align);
        assert!(!table.is_copy(wrapped));
        assert!(table.needs_drop(owning));
        assert!(!table.needs_drop(wrapped));

        let scalar_wrapper = table.add_struct(StructDef {
            name: Symbol::intern("MaybeUninit_i32"),
            fields: vec![field("value", c.i32)],
            span: Span::DUMMY,
            derives_copy: true,
            has_drop: false,
            drops_fields: false,
            origin: None,
            declaring_module: usize::MAX,
        });
        let scalar_wrapped = table.intern(TyKind::Struct(scalar_wrapper));
        assert!(table.is_copy(scalar_wrapped));
        let scalar_wrapped_layout = table.layout(scalar_wrapped);
        let scalar_layout = table.layout(c.i32);
        assert_eq!(scalar_wrapped_layout.size, scalar_layout.size);
        assert_eq!(scalar_wrapped_layout.align, scalar_layout.align);
    }

    #[test]
    fn builtin_zeroable_is_a_validity_fact_not_a_copy_or_drop_fact() {
        // `[ARN-11]` — primitive zero is valid, references are not, and an
        // owning empty buffer may be zero-valid without becoming Copy.
        let (mut table, c) = TypeTable::new();
        let raw = table.intern(TyKind::Ptr { mutable: true, inner: c.i32 });
        let reference = table.intern(TyKind::Ref { mutable: false, inner: c.i32 });
        let buffer = table.intern(TyKind::Vec { elem: c.i32, text: false });
        assert!(table.is_builtin_zeroable(c.i32));
        assert!(table.is_builtin_zeroable(raw));
        assert!(!table.is_builtin_zeroable(reference));
        assert!(table.is_builtin_zeroable(buffer));
        assert!(!table.is_copy(buffer));
        assert!(table.needs_drop(buffer));
    }

    #[test]
    fn shared_refs_are_copy_and_mutable_refs_are_not() {
        // [TYP-14], [SPN-3]
        let (mut table, c) = TypeTable::new();
        let shared = table.intern(TyKind::Ref { mutable: false, inner: c.i32 });
        let unique = table.intern(TyKind::Ref { mutable: true, inner: c.i32 });
        assert!(table.is_copy(shared));
        assert!(!table.is_copy(unique));
    }

    #[test]
    fn a_struct_holding_a_reference_is_a_view() {
        // [TYP-14] — the `@view` attribute documents this; the property is
        // structural.
        let (mut table, c) = TypeTable::new();
        let r = table.intern(TyKind::Ref { mutable: false, inner: c.f32 });
        let id = table.add_struct(StructDef {
            name: Symbol::intern("Window"),
            fields: vec![field("data", r)],
            span: Span::DUMMY,
            derives_copy: false,
            has_drop: false,
            drops_fields: true,
            origin: None,
            declaring_module: 0,
        });
        let ty = table.intern(TyKind::Struct(id));
        assert!(table.is_view(ty));
        assert!(!table.is_view(c.f32));
    }

    #[test]
    fn widening_is_lossless_only() {
        // [TYP-5]
        let (table, c) = TypeTable::new();
        assert!(table.widens_to(c.i32, c.i64));
        assert!(table.widens_to(c.u8, c.u32));
        assert!(table.widens_to(c.u8, c.i32));
        assert!(table.widens_to(c.f32, c.f64));
        assert!(!table.widens_to(c.i64, c.i32), "narrowing is never implicit");
        assert!(!table.widens_to(c.i32, c.u32), "signed to unsigned is not lossless");
        assert!(!table.widens_to(c.u64, c.i64), "u64 does not fit in i64");
        assert!(!table.widens_to(c.f64, c.f32));
        assert!(!table.widens_to(c.bool_, c.i32), "nothing converts to bool implicitly");
        assert!(!table.widens_to(c.char_, c.u32));
    }

    #[test]
    fn ffi_safety_follows_the_fields() {
        let (mut table, c) = TypeTable::new();
        let plain = table.add_struct(StructDef {
            name: Symbol::intern("Vector3"),
            fields: vec![field("x", c.f32), field("y", c.f32)],
            span: Span::DUMMY,
            derives_copy: true,
            has_drop: false,
            drops_fields: true,
            origin: None,
            declaring_module: 0,
        });
        let plain_ty = table.intern(TyKind::Struct(plain));
        assert!(table.is_ffi_safe(plain_ty));

        let with_str = table.add_struct(StructDef {
            name: Symbol::intern("Named"),
            fields: vec![field("name", c.str_)],
            span: Span::DUMMY,
            derives_copy: false,
            has_drop: false,
            drops_fields: true,
            origin: None,
            declaring_module: 0,
        });
        let with_str_ty = table.intern(TyKind::Struct(with_str));
        assert!(!table.is_ffi_safe(with_str_ty), "a view is not FFI-safe on its own");
    }

    #[test]
    fn arrays_are_elements_end_to_end() {
        let (mut table, c) = TypeTable::new();
        let ty = table.intern(TyKind::Array { elem: c.f32, len: 16 });
        let layout = table.layout(ty);
        assert_eq!(layout.size, 64);
        assert_eq!(layout.align, 4);
    }

    #[test]
    fn integer_bounds_are_available_for_literal_range_checks() {
        // [LEX-16] / E2010
        let (table, c) = TypeTable::new();
        assert_eq!(int_max(&table, c.u8), Some(255));
        assert_eq!(int_max(&table, c.i8), Some(127));
        assert_eq!(int_max(&table, c.u128), Some(u128::MAX));
        assert_eq!(int_max(&table, c.f32), None);
    }

    #[test]
    fn display_names_types_the_way_a_reader_writes_them() {
        let (mut table, c) = TypeTable::new();
        let r = table.intern(TyKind::Ref { mutable: true, inner: c.f32 });
        assert_eq!(table.display(r), "ref mut f32");
        let t = table.intern(TyKind::Tuple(vec![c.i32, c.f32]));
        assert_eq!(table.display(t), "(i32, f32)");
        let a = table.intern(TyKind::Array { elem: c.f32, len: 4 });
        assert_eq!(table.display(a), "[f32; 4]");
    }
}
