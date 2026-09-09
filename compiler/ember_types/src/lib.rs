//! The type interner, the type table, and layout computation.
//!
//! Spec: Part IV (the type system) and Part XVIII §4.3 (the `TypeInfo` table).
//!
//! `[TYP-1]` — every concrete type has a compile-time-known `size`, `align`,
//! `is_copy`, `is_send`, `is_sync`, `needs_drop`, `is_view` and `has_niche`.
//! `[TYP-11]` — the default struct layout is C-compatible, deliberately, so
//! that every plain struct can cross an FFI boundary.

use std::collections::HashMap;
use std::fmt;

use ember_span::{Span, Symbol};

/// An interned type. Equality is identity.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ty(u32);

/// Index of a user-declared struct in the type table.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct StructId(pub u32);

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
    Isize,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum UintTy {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum FloatTy {
    F16,
    F32,
    F64,
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
    Struct(StructId),
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
    /// `Array[T]` — a growable, heap-allocated sequence. Part XX.1 makes this
    /// a compiler-known type until Phase 2's generics let the standard library
    /// write it in Ember. `String` is this with `u8` elements.
    Vec { elem: Ty },
    Fn { params: Vec<Ty>, ret: Ty },
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
    /// `[STR-2]` — whether the declaration supplied a default.
    pub has_default: bool,
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
    /// `[TYP-12]` — the integer type the tag is stored in.
    pub repr: Ty,
    /// Whether `@repr` was written. `[TYP-12]` requires it for FFI.
    pub repr_is_explicit: bool,
    /// `[ENM-4]` — a payload enum is `Copy` only via `@derive(Copy)`.
    pub derives_copy: bool,
    pub has_drop: bool,
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

/// The interner and type table for one compilation.
pub struct TypeTable {
    kinds: Vec<TyKind>,
    lookup: HashMap<TyKind, Ty>,
    structs: Vec<StructDef>,
    enums: Vec<EnumDef>,
    ranges: Vec<RangeDef>,
    next_infer: u32,
    /// Pointer width of the target, in bytes. 8 for every v1 target.
    pointer_size: u64,
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

impl TypeTable {
    pub fn new() -> (TypeTable, CommonTypes) {
        let mut table = TypeTable {
            kinds: Vec::new(),
            lookup: HashMap::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            ranges: Vec::new(),
            next_infer: 0,
            pointer_size: 8,
        };
        let common = CommonTypes {
            bool_: table.intern(TyKind::Bool),
            char_: table.intern(TyKind::Char),
            i8: table.intern(TyKind::Int(IntTy::I8)),
            i16: table.intern(TyKind::Int(IntTy::I16)),
            i32: table.intern(TyKind::Int(IntTy::I32)),
            i64: table.intern(TyKind::Int(IntTy::I64)),
            i128: table.intern(TyKind::Int(IntTy::I128)),
            isize: table.intern(TyKind::Int(IntTy::Isize)),
            u8: table.intern(TyKind::Uint(UintTy::U8)),
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

    /// Whether a type mentions any generic parameter, and so still has to be
    /// substituted before it means anything at run time.
    pub fn is_generic(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            TyKind::Param { .. } => true,
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => self.is_generic(*inner),
            TyKind::Array { elem, .. } | TyKind::Vec { elem } => self.is_generic(*elem),
            TyKind::Tuple(items) => items.iter().any(|&t| self.is_generic(t)),
            TyKind::Fn { params, ret } => {
                params.iter().any(|&t| self.is_generic(t)) || self.is_generic(*ret)
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
            TyKind::Vec { elem } => {
                let elem = self.substitute_self(elem, concrete);
                self.intern(TyKind::Vec { elem })
            }
            TyKind::Tuple(items) => {
                let items: Vec<Ty> =
                    items.iter().map(|&t| self.substitute_self(t, concrete)).collect();
                self.intern(TyKind::Tuple(items))
            }
            TyKind::Fn { params, ret } => {
                let params: Vec<Ty> =
                    params.iter().map(|&t| self.substitute_self(t, concrete)).collect();
                let ret = self.substitute_self(ret, concrete);
                self.intern(TyKind::Fn { params, ret })
            }
            _ => ty,
        }
    }

    pub fn substitute(&mut self, ty: Ty, args: &[Ty]) -> Ty {
        match self.kind(ty).clone() {
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
            TyKind::Vec { elem } => {
                let elem = self.substitute(elem, args);
                self.intern(TyKind::Vec { elem })
            }
            TyKind::Tuple(items) => {
                let items: Vec<Ty> = items.iter().map(|&t| self.substitute(t, args)).collect();
                self.intern(TyKind::Tuple(items))
            }
            TyKind::Fn { params, ret } => {
                let params: Vec<Ty> = params.iter().map(|&t| self.substitute(t, args)).collect();
                let ret = self.substitute(ret, args);
                self.intern(TyKind::Fn { params, ret })
            }
            _ => ty,
        }
    }

    /// `[TYP-18]` — match a declared parameter type against the type an
    /// argument actually has, filling in `args` where a parameter is met.
    /// Returns false only on a shape mismatch the caller should report.
    pub fn unify(&self, declared: Ty, actual: Ty, args: &mut Vec<Option<Ty>>) -> bool {
        match (self.kind(declared).clone(), self.kind(actual).clone()) {
            (TyKind::Param { index, .. }, _) => {
                let slot = index as usize;
                if slot >= args.len() {
                    return false;
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
            | (TyKind::Array { elem: a, .. }, TyKind::Array { elem: b, .. })
            | (TyKind::Vec { elem: a }, TyKind::Vec { elem: b }) => self.unify(a, b, args),
            // Two instantiations of the same generic struct unify argument
            // by argument.
            (TyKind::Struct(a), TyKind::Struct(b)) => {
                match (&self.struct_def(a).origin, &self.struct_def(b).origin) {
                    (Some((na, aa)), Some((nb, ab))) if na == nb && aa.len() == ab.len() => {
                        aa.iter().zip(ab.iter()).all(|(&x, &y)| self.unify(x, y, args))
                    }
                    _ => true,
                }
            }
            (TyKind::Tuple(a), TyKind::Tuple(b)) if a.len() == b.len() => {
                a.iter().zip(b.iter()).all(|(&x, &y)| self.unify(x, y, args))
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
            TyKind::Bool => Layout::scalar(1),
            TyKind::Char => Layout::scalar(4),
            TyKind::Int(int) => Layout::scalar(match int {
                IntTy::I8 => 1,
                IntTy::I16 => 2,
                IntTy::I32 => 4,
                IntTy::I64 => 8,
                IntTy::I128 => 16,
                IntTy::Isize => self.pointer_size,
            }),
            TyKind::Uint(uint) => Layout::scalar(match uint {
                UintTy::U8 => 1,
                UintTy::U16 => 2,
                UintTy::U32 => 4,
                UintTy::U64 => 8,
                UintTy::U128 => 16,
                UintTy::Usize => self.pointer_size,
            }),
            TyKind::Float(float) => Layout::scalar(match float {
                FloatTy::F16 => 2,
                FloatTy::F32 => 4,
                FloatTy::F64 => 8,
            }),
            TyKind::Void | TyKind::Never => Layout::ZERO,
            // A view: pointer plus length.
            TyKind::Str => Layout {
                size: self.pointer_size * 2,
                align: self.pointer_size,
                field_offsets: vec![0, self.pointer_size],
            },
            TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Fn { .. } => {
                Layout::scalar(self.pointer_size)
            }
            TyKind::Array { elem, len } => {
                let inner = self.layout(*elem);
                Layout { size: inner.size * len, align: inner.align, field_offsets: Vec::new() }
            }
            TyKind::Tuple(items) => self.aggregate_layout(items.iter().copied()),
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                self.aggregate_layout(def.fields.iter().map(|f| f.ty))
            }
            TyKind::Enum(id) => self.enum_layout(*id),
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
            TyKind::Infer(_) | TyKind::IntLit | TyKind::FloatLit | TyKind::Error => Layout::ZERO,
        }
    }

    /// `[TYP-12]` — a unit-only enum is its discriminant. A payload enum is
    /// `{tag, union of variants}`, tag first. `[TYP-13]`'s niche optimisation
    /// is not applied yet, so `Option[T]` is still tag-plus-payload.
    fn enum_layout(&self, id: EnumId) -> Layout {
        let def = self.enum_def(id);
        let tag = self.layout(def.repr);
        if def.is_unit_only() {
            return tag;
        }
        let mut payload_size = 0u64;
        let mut payload_align = 1u64;
        for variant in &def.variants {
            let inner = self.aggregate_layout(variant.fields.iter().map(|f| f.ty));
            payload_size = payload_size.max(inner.size);
            payload_align = payload_align.max(inner.align);
        }
        let align = tag.align.max(payload_align);
        let offset = align_to(tag.size, payload_align);
        Layout {
            size: align_to(offset + payload_size, align),
            align,
            field_offsets: vec![0, offset],
        }
    }

    fn aggregate_layout(&self, fields: impl Iterator<Item = Ty>) -> Layout {
        let mut offset = 0u64;
        let mut align = 1u64;
        let mut field_offsets = Vec::new();
        for field in fields {
            let inner = self.layout(field);
            offset = align_to(offset, inner.align);
            field_offsets.push(offset);
            offset += inner.size;
            align = align.max(inner.align);
        }
        Layout { size: align_to(offset, align), align, field_offsets }
    }

    /// `[OWN-7]`, `[STR-3]` — `Copy` values duplicate bitwise on use. Scalars,
    /// views, raw pointers and function pointers are always `Copy`; a struct
    /// is `Copy` only if it was declared `@derive(Copy)`, so that adding a
    /// field later cannot silently change semantics.
    pub fn is_copy(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            TyKind::Bool
            | TyKind::Char
            | TyKind::Int(_)
            | TyKind::Uint(_)
            | TyKind::Float(_)
            | TyKind::Void
            | TyKind::Never
            | TyKind::Str
            | TyKind::Ptr { .. }
            | TyKind::Fn { .. }
            | TyKind::IntLit
            | TyKind::FloatLit
            | TyKind::Error => true,
            // `ref T` is Copy; `ref mut T` is move-only and reborrowable.
            TyKind::Ref { mutable, .. } => !mutable,
            TyKind::Array { elem, .. } => self.is_copy(*elem),
            TyKind::Tuple(items) => items.iter().all(|&t| self.is_copy(t)),
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                def.derives_copy && !def.has_drop && def.fields.iter().all(|f| self.is_copy(f.ty))
            }
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
                            .all(|v| v.fields.iter().all(|f| self.is_copy(f.ty))))
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
            TyKind::Param { .. } | TyKind::Assoc { .. } | TyKind::Infer(_) => false,
        }
    }

    /// `[TYP-1]` — whether a value of this type needs destruction. Drives drop
    /// elaboration (Part XVIII §4.9).
    pub fn needs_drop(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.struct_def(*id);
                def.has_drop || def.fields.iter().any(|f| self.needs_drop(f.ty))
            }
            TyKind::Enum(id) => {
                let def = self.enum_def(*id);
                def.has_drop
                    || def
                        .variants
                        .iter()
                        .any(|v| v.fields.iter().any(|f| self.needs_drop(f.ty)))
            }
            TyKind::Tuple(items) => items.iter().any(|&t| self.needs_drop(t)),
            TyKind::Array { elem, .. } => self.needs_drop(*elem),
            // An `Array[T]` or a `String` always owns a heap buffer, whatever
            // the element type is.
            TyKind::Vec { .. } => true,
            _ => false,
        }
    }

    /// `[TYP-14]` — a type carrying a borrow. View types may not be stored in
    /// class fields, statics, containers or across threads (`[TYP-15]`).
    pub fn is_view(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            TyKind::Ref { .. } | TyKind::Str => true,
            TyKind::Tuple(items) => items.iter().any(|&t| self.is_view(t)),
            TyKind::Array { elem, .. } => self.is_view(*elem),
            TyKind::Struct(id) => {
                self.struct_def(*id).fields.iter().any(|f| self.is_view(f.ty))
            }
            TyKind::Enum(id) => self
                .enum_def(*id)
                .variants
                .iter()
                .any(|v| v.fields.iter().any(|f| self.is_view(f.ty))),
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
            | TyKind::Ptr { .. }
            | TyKind::Fn { .. } => true,
            TyKind::Array { elem, .. } => self.is_ffi_safe(*elem),
            TyKind::Struct(id) => {
                self.struct_def(*id).fields.iter().all(|f| self.is_ffi_safe(f.ty))
            }
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
    pub fn display(&self, ty: Ty) -> String {
        match self.kind(ty) {
            TyKind::Bool => "bool".into(),
            TyKind::Char => "char".into(),
            TyKind::Int(i) => match i {
                IntTy::I8 => "i8",
                IntTy::I16 => "i16",
                IntTy::I32 => "i32",
                IntTy::I64 => "i64",
                IntTy::I128 => "i128",
                IntTy::Isize => "isize",
            }
            .into(),
            TyKind::Uint(u) => match u {
                UintTy::U8 => "u8",
                UintTy::U16 => "u16",
                UintTy::U32 => "u32",
                UintTy::U64 => "u64",
                UintTy::U128 => "u128",
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
            TyKind::Struct(id) => self.struct_def(*id).name.to_string(),
            TyKind::Enum(id) => self.enum_def(*id).name.to_string(),
            TyKind::Range(id) => self.range_def(*id).name.to_string(),
            TyKind::Param { name, .. } => name.to_string(),
            TyKind::Assoc { name } => format!("Self.{name}"),
            // `String` prints as itself, not as `Array[u8]`.
            TyKind::Vec { elem } if matches!(self.kind(*elem), TyKind::Uint(UintTy::U8)) => {
                "String".into()
            }
            TyKind::Vec { elem } => format!("Array[{}]", self.display(*elem)),
            TyKind::Tuple(items) => {
                let inner: Vec<String> = items.iter().map(|&t| self.display(t)).collect();
                format!("({})", inner.join(", "))
            }
            TyKind::Ref { mutable, inner } => {
                let kw = if *mutable { "ref mut " } else { "ref " };
                format!("{kw}{}", self.display(*inner))
            }
            TyKind::Ptr { mutable, inner } => {
                let kw = if *mutable { "*mut " } else { "*" };
                format!("{kw}{}", self.display(*inner))
            }
            TyKind::Array { elem, len } => format!("[{}; {len}]", self.display(*elem)),
            TyKind::Fn { params, ret } => {
                let inner: Vec<String> = params.iter().map(|&t| self.display(t)).collect();
                format!("fn({}) -> {}", inner.join(", "), self.display(*ret))
            }
            TyKind::Infer(_) => "_".into(),
            // A diagnostic should never name these; if one does, say something
            // a reader recognises rather than an internal name.
            TyKind::IntLit => "an integer".into(),
            TyKind::FloatLit => "a float".into(),
            TyKind::Error => "<error>".into(),
        }
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
/// The profile chooses the default (`debug` panics, `release` and `shipping`
/// wrap); `@overflow(panic|wrap|saturate)` on a function or module overrides
/// it. `[PRF-1]` names this as one of only three things a profile may change
/// about a program's semantics.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum OverflowPolicy {
    /// Every overflowing `+ - * <<` panics.
    #[default]
    Panic,
    /// `+ - *` and `<<` wrap two's-complement; shift amounts are masked
    /// (`[TYP-10]`).
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

/// Whether an integer type is signed. `None` for anything that is not an
/// integer.
pub fn is_signed(table: &TypeTable, ty: Ty) -> Option<bool> {
    match table.kind(ty) {
        TyKind::Int(_) => Some(true),
        TyKind::Uint(_) => Some(false),
        _ => None,
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
        TyKind::Int(i) => Some(1u128 << (int_bits(*i, table.pointer_size()) - 1)),
        _ => None,
    }
}

/// The largest value an integer type can hold, for `[LEX-16]`'s range check.
pub fn int_max(table: &TypeTable, ty: Ty) -> Option<u128> {
    match table.kind(ty) {
        TyKind::Int(i) => {
            let bits = int_bits(*i, table.pointer_size());
            Some((1u128 << (bits - 1)) - 1)
        }
        TyKind::Uint(u) => {
            let bits = uint_bits(*u, table.pointer_size());
            Some(if bits == 128 { u128::MAX } else { (1u128 << bits) - 1 })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ember_span::Span;

    fn field(name: &str, ty: Ty) -> FieldDef {
        FieldDef {
            name: Symbol::intern(name),
            ty,
            span: Span::DUMMY,
            has_default: false,
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
            origin: None,
            declaring_module: 0,
        });
        let ty = table.intern(TyKind::Struct(id));
        assert!(!table.is_copy(ty));
        assert!(table.needs_drop(ty));
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
