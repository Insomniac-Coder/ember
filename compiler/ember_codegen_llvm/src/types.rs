//! `[CG-LL-2]` — the C backend's type mapping, as LLVM sees it. Every type keeps the layout the
//! type table gives it (`[TYP-11]`, `[LAY-2]`), which is the layout of the C type the C backend
//! declares for it: a scalar lives in a register as its own IR type, and everything else lives in
//! memory, reached by byte offsets, so no IR struct type has to repeat C's layout rules.

use crate::abi::{Scalar, Shape, Target};
use ember_types::{EnumId, FloatTy, IntTy, Ty, TyKind, TypeTable, UintTy};

/// A scalar as the IR holds it in a register. A `bool` is `i1` there and a byte in memory.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Sc {
    Bool,
    I(u32),
    F32,
    F64,
    F80,
    /// IEEE quad precision: AArch64 Linux's `long double`.
    F128,
    Ptr,
}

impl Sc {
    /// The register type.
    pub fn ir(self) -> String {
        match self {
            Sc::Bool => "i1".to_string(),
            Sc::I(bits) => format!("i{bits}"),
            Sc::F32 => "float".to_string(),
            Sc::F64 => "double".to_string(),
            Sc::F80 => "x86_fp80".to_string(),
            Sc::F128 => "fp128".to_string(),
            Sc::Ptr => "ptr".to_string(),
        }
    }

    /// The type a load or store of it in memory uses.
    pub fn mem(self) -> String {
        match self {
            Sc::Bool => "i8".to_string(),
            other => other.ir(),
        }
    }

    pub fn is_float(self) -> bool {
        matches!(self, Sc::F32 | Sc::F64 | Sc::F80 | Sc::F128)
    }

    pub fn bits(self) -> u32 {
        match self {
            Sc::Bool => 1,
            Sc::I(bits) => bits,
            Sc::F32 => 32,
            Sc::F64 | Sc::Ptr => 64,
            Sc::F80 => 80,
            Sc::F128 => 128,
        }
    }
}

/// How a value of a type is held.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Repr {
    /// No bytes (`void`, `Never`, an empty struct): nothing is stored or passed.
    Zst,
    Scalar(Sc),
    /// In memory, `size` bytes aligned to `align`.
    Mem { size: u64, align: u64 },
}

/// The type table, as the backend reads it.
pub struct Types<'a> {
    pub table: &'a TypeTable,
    pub target: Target,
    pub usize_ty: Ty,
    /// `[OBJ-1]`, `[HR-12]` — every object's header, in bytes: 24, or 40 in a reloadable image
    /// whose instances migrate (its live-instance links).
    pub header: u64,
}

impl<'a> Types<'a> {
    pub fn new(table: &'a TypeTable, target: Target) -> Types<'a> {
        let usize_ty = table
            .find(&TyKind::Uint(UintTy::Usize))
            .expect("the interner always holds `usize`");
        Types { table, target, usize_ty, header: 24 }
    }

    /// Each field's offset from a class object's start, after the header.
    pub fn object_field_offsets(&self, id: ember_types::ClassId) -> Vec<u64> {
        self.table.class_field_offsets_after(id, self.header)
    }

    /// `EMBER_OBJ_PAYLOAD_OFFSET` — where a shared value's payload starts after its object's header.
    pub fn payload_offset(&self, align: u64) -> u64 {
        (self.header + align - 1) & !(align - 1)
    }

    pub fn kind(&self, ty: Ty) -> &TyKind {
        self.table.kind(ty)
    }

    pub fn size_align(&self, ty: Ty) -> (u64, u64) {
        let layout = self.table.layout(ty);
        (layout.size, layout.align.max(1))
    }

    /// A unit-only enum's value is its discriminant's integer, and a niche `Option`'s its
    /// payload's; a range is its representation.
    pub fn repr(&self, ty: Ty) -> Repr {
        let int = |bits| Repr::Scalar(Sc::I(bits));
        match self.kind(ty) {
            TyKind::Bool => Repr::Scalar(Sc::Bool),
            TyKind::Char => int(32),
            TyKind::Int(i) => int(match i {
                IntTy::I8 => 8,
                IntTy::I16 => 16,
                IntTy::I32 => 32,
                IntTy::I64 | IntTy::Isize => 64,
                IntTy::I128 => 128,
                IntTy::I256 => 256,
            }),
            TyKind::Uint(u) => int(match u {
                UintTy::U8 => 8,
                UintTy::U16 => 16,
                UintTy::U32 => 32,
                UintTy::U64 | UintTy::Usize => 64,
                UintTy::U128 => 128,
                UintTy::U256 => 256,
            }),
            // `[TYP-9]` — an `f16` is its bits in storage, as in the C backend; arithmetic on it
            // goes through the runtime's conversions.
            TyKind::Float(FloatTy::F16) => int(16),
            TyKind::Float(FloatTy::F32) => Repr::Scalar(Sc::F32),
            TyKind::Float(FloatTy::F64) => Repr::Scalar(Sc::F64),
            TyKind::CLongDouble => match self.target {
                Target::SysV => Repr::Scalar(Sc::F80),
                Target::Win64 => Repr::Scalar(Sc::F64),
                Target::Aapcs64 => Repr::Scalar(Sc::F128),
            },
            TyKind::Void | TyKind::Never | TyKind::Error | TyKind::Opaque(_) => Repr::Zst,
            TyKind::CStr | TyKind::Class(_) | TyKind::ClassInterface(_) => Repr::Scalar(Sc::Ptr),
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } if !matches!(self.kind(*inner), TyKind::Dyn { .. }) => {
                Repr::Scalar(Sc::Ptr)
            }
            TyKind::Fn { abi: Some(_), .. } => Repr::Scalar(Sc::Ptr),
            // `[FFI-8]` — a `va_list` parameter is a pointer on both x86-64 ABIs: Windows' is a
            // `char*`, System V's an array of one `__va_list_tag`, which a parameter decays to a
            // pointer to (`va_list_layout`'s 8 bytes, which hold no scalar its shape could show).
            TyKind::VaList => Repr::Scalar(Sc::Ptr),
            TyKind::Range(id) => self.repr(self.table.range_def(*id).repr),
            // `[FFI-8]` — a newtype is its typedef's representation, passed as C passes it.
            TyKind::Newtype(id) => self.repr(self.table.newtype_def(*id).repr),
            TyKind::Enum(id) => self.enum_repr(*id, ty),
            _ => self.mem(ty),
        }
    }

    fn mem(&self, ty: Ty) -> Repr {
        let (size, align) = self.size_align(ty);
        if size == 0 { Repr::Zst } else { Repr::Mem { size, align } }
    }

    fn enum_repr(&self, id: EnumId, ty: Ty) -> Repr {
        if let Some(niche) = self.table.option_niche(id) {
            // `None` of an `Option[bool]` is the byte 2, which an `i1` cannot hold.
            return match self.repr(niche.payload) {
                Repr::Scalar(Sc::Bool) => Repr::Scalar(Sc::I(8)),
                other => other,
            };
        }
        if self.table.enum_is_bare(id) {
            return self.repr(self.table.enum_def(id).repr);
        }
        self.mem(ty)
    }

    pub fn is_zst(&self, ty: Ty) -> bool {
        self.repr(ty) == Repr::Zst
    }

    pub fn is_signed(&self, ty: Ty) -> bool {
        match self.kind(ty) {
            TyKind::Int(_) => true,
            TyKind::Range(id) => self.is_signed(self.table.range_def(*id).repr),
            TyKind::Newtype(id) => self.is_signed(self.table.newtype_def(*id).repr),
            TyKind::Enum(id) if self.table.enum_is_bare(*id) => self.is_signed(self.table.enum_def(*id).repr),
            _ => false,
        }
    }

    /// The representation under a range type or a newtype (`[FFI-8]`).
    pub fn through_range(&self, ty: Ty) -> Ty {
        match self.kind(ty) {
            TyKind::Range(id) => self.through_range(self.table.range_def(*id).repr),
            TyKind::Newtype(id) => self.through_range(self.table.newtype_def(*id).repr),
            _ => ty,
        }
    }

    pub fn is_f16(&self, ty: Ty) -> bool {
        matches!(self.kind(self.through_range(ty)), TyKind::Float(FloatTy::F16))
    }

    pub fn is_float(&self, ty: Ty) -> bool {
        matches!(self.kind(self.through_range(ty)), TyKind::Float(_) | TyKind::CLongDouble)
    }

    // -- the ABI's view ---------------------------------------------------------------------------

    /// The type's shape as the C ABI classifies its C type.
    pub fn shape(&self, ty: Ty) -> Shape {
        match self.repr(ty) {
            Repr::Zst => Shape::Aggregate { size: 0, align: 1, leaves: Vec::new(), unaligned: false, vectors: false },
            Repr::Scalar(sc) => match self.scalar_shape(ty, sc) {
                Some(scalar) => Shape::Scalar(scalar),
                None => self.aggregate_shape(ty),
            },
            Repr::Mem { .. } => self.aggregate_shape(ty),
        }
    }

    /// The C scalar a register value of `ty` is, or `None` when its C type is a struct
    /// (`ember_i256`, an `@align` enum).
    fn scalar_shape(&self, ty: Ty, sc: Sc) -> Option<Scalar> {
        Some(match sc {
            Sc::Bool => Scalar::Bool,
            Sc::I(256) => return None,
            Sc::I(bits) => Scalar::Int { bytes: (bits / 8) as u8, signed: self.is_signed(ty) },
            Sc::F32 => Scalar::F32,
            Sc::F64 => Scalar::F64,
            Sc::F80 => Scalar::F80,
            Sc::F128 => Scalar::F128,
            Sc::Ptr => Scalar::Ptr,
        })
    }

    fn aggregate_shape(&self, ty: Ty) -> Shape {
        let (size, align) = self.size_align(ty);
        let (mut leaves, mut vectors) = (Vec::new(), false);
        // Past four eightbytes only the size is read (System V passes it in memory, Windows behind
        // a pointer, AAPCS64 and `__vectorcall` hold no floating-point aggregate larger), so a
        // large array's leaves are never listed.
        if size <= 32 {
            self.leaves(ty, 0, &mut leaves, &mut vectors);
        }
        let unaligned = leaves.iter().any(|(offset, scalar)| offset % scalar.size().clamp(1, 8) != 0);
        Shape::Aggregate { size, align, leaves, unaligned, vectors }
    }

    /// Every scalar of a value of `ty` placed at `base`, as C lays out its C type; `vectors` set
    /// when it holds a `std.simd` vector.
    fn leaves(&self, ty: Ty, base: u64, out: &mut Vec<(u64, Scalar)>, vectors: &mut bool) {
        let word = Scalar::Int { bytes: 8, signed: false };
        match self.kind(ty) {
            TyKind::Str | TyKind::Span { .. } => out.extend([(base, Scalar::Ptr), (base + 8, word)]),
            TyKind::Vec { .. } => out.extend([(base, Scalar::Ptr), (base + 8, word), (base + 16, word)]),
            TyKind::Ref { .. } | TyKind::Ptr { .. } | TyKind::Dyn { .. } if matches!(self.repr(ty), Repr::Mem { .. }) => {
                out.extend([(base, Scalar::Ptr), (base + 8, Scalar::Ptr)])
            }
            TyKind::Fn { abi: None, .. } => out.extend((0..5).map(|i| (base + 8 * i, Scalar::Ptr))),
            TyKind::Int(IntTy::I256) | TyKind::Uint(UintTy::U256) => out.extend((0..4).map(|i| (base + 8 * i, word))),
            TyKind::Struct(id) => {
                *vectors |= ember_types::SimdShape::of(self.table.struct_def(*id).name.as_str()).is_some();
                let offsets = self.table.layout(ty).field_offsets;
                for (field, offset) in self.table.struct_def(*id).fields.iter().zip(offsets) {
                    self.leaves(field.ty, base + offset, out, vectors);
                }
            }
            TyKind::Tuple(items) => {
                let offsets = self.table.layout(ty).field_offsets;
                for (item, offset) in items.iter().zip(offsets) {
                    self.leaves(*item, base + offset, out, vectors);
                }
            }
            TyKind::Array { elem, len } => {
                let (size, _) = self.size_align(*elem);
                for i in 0..*len {
                    self.leaves(*elem, base + i * size, out, vectors);
                }
            }
            TyKind::Enum(id) if matches!(self.repr(ty), Repr::Mem { .. }) && self.table.option_niche(*id).is_none() => {
                let def = self.table.enum_def(*id);
                self.leaves(def.repr, base, out, vectors);
                for (variant, offsets) in def.variants.iter().zip(self.table.variant_field_offsets(*id)) {
                    for (field, offset) in variant.fields.iter().zip(offsets) {
                        self.leaves(field.ty, base + offset, out, vectors);
                    }
                }
            }
            TyKind::Enum(id) => match self.table.option_niche(*id) {
                Some(niche) => self.leaves(niche.payload, base, out, vectors),
                None => self.leaves(self.table.enum_def(*id).repr, base, out, vectors),
            },
            TyKind::Range(id) => self.leaves(self.table.range_def(*id).repr, base, out, vectors),
            TyKind::Newtype(id) => self.leaves(self.table.newtype_def(*id).repr, base, out, vectors),
            _ => {
                if let Repr::Scalar(sc) = self.repr(ty)
                    && let Some(scalar) = self.scalar_shape(ty, sc)
                {
                    out.push((base, scalar));
                }
            }
        }
    }
}

/// The largest power of two dividing `offset` (an offset of 0 is aligned to anything).
pub fn offset_align(offset: u64) -> u64 {
    if offset == 0 { u64::MAX } else { 1 << offset.trailing_zeros() }
}
