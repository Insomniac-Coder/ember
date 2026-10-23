//! `[CG-LL-2]`, `[FFI-9]` — the platform C ABI's classification of arguments and results,
//! done by the backend itself: Windows x64 (and its `__vectorcall`), System V x86-64 and AArch64's
//! AAPCS64, exactly as Clang classifies the same C types, so a function the IR defines and one the
//! C runtime defines call each other with nothing between. Every Ember function uses it, so a
//! function value, a runtime callback and an export are one convention.

use std::fmt;

/// The ABI the program is built for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    /// Windows x64 (`x86_64-pc-windows-msvc`).
    Win64,
    /// System V x86-64 (Linux, the BSDs).
    SysV,
    /// AArch64's procedure call standard (`aarch64-linux-gnu`): no small integer is extended,
    /// a floating-point aggregate of up to four members goes in SIMD registers, any other
    /// aggregate of up to 16 bytes in general ones, a larger one behind a pointer to a copy.
    Aapcs64,
}

impl Target {
    /// The host's: the IR is compiled by the Clang the build finds, for its default target.
    pub fn host() -> Target {
        if cfg!(windows) {
            Target::Win64
        } else if cfg!(target_arch = "aarch64") {
            Target::Aapcs64
        } else {
            Target::SysV
        }
    }
}

/// A function's calling convention: the platform's C one, or `__vectorcall` (`[FFI-9]`, Windows
/// x64), which passes floating-point aggregates in vector registers.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Conv {
    #[default]
    C,
    VectorCall,
}

impl Conv {
    /// The IR's calling convention keyword, with its space, before a definition's, a
    /// declaration's or a call's result type.
    pub fn ir(self) -> &'static str {
        match self {
            Conv::C => "",
            Conv::VectorCall => "x86_vectorcallcc ",
        }
    }
}

/// A machine scalar as the ABI sees it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scalar {
    /// C's `bool`: one byte holding 0 or 1, `i1 zeroext` in a register.
    Bool,
    Int { bytes: u8, signed: bool },
    F32,
    F64,
    /// x87's 80-bit `long double` (System V only; Windows' is a `double`).
    F80,
    /// IEEE quad precision, AArch64 Linux's `long double`.
    F128,
    Ptr,
}

impl Scalar {
    pub fn size(self) -> u64 {
        match self {
            Scalar::Bool => 1,
            Scalar::Int { bytes, .. } => bytes as u64,
            Scalar::F32 => 4,
            Scalar::F64 | Scalar::Ptr => 8,
            Scalar::F80 | Scalar::F128 => 16,
        }
    }

    /// The type the scalar has in IR registers.
    pub fn ir(self) -> PartTy {
        match self {
            Scalar::Bool => PartTy::I(1),
            Scalar::Int { bytes, .. } => PartTy::I(bytes as u32 * 8),
            Scalar::F32 => PartTy::F32,
            Scalar::F64 => PartTy::F64,
            Scalar::F80 => PartTy::F80,
            Scalar::F128 => PartTy::F128,
            Scalar::Ptr => PartTy::Ptr,
        }
    }

    fn is_int_class(self) -> bool {
        matches!(self, Scalar::Bool | Scalar::Int { .. } | Scalar::Ptr)
    }
}

/// A value's shape for classification.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Scalar(Scalar),
    /// A struct, union or array: its size, its alignment and each scalar leaf at its offset (listed
    /// up to 32 bytes, the largest floating-point aggregate); `unaligned` when a leaf lies off its
    /// natural alignment (a `@packed` struct's); `vectors` when it holds a `std.simd` vector, which
    /// C holds in a union of its lanes and its registers, never a homogeneous aggregate.
    Aggregate { size: u64, align: u64, leaves: Vec<(u64, Scalar)>, unaligned: bool, vectors: bool },
}

impl Shape {
    pub fn size(&self) -> u64 {
        match self {
            Shape::Scalar(scalar) => scalar.size(),
            Shape::Aggregate { size, .. } => *size,
        }
    }

    pub fn align(&self) -> u64 {
        match self {
            Shape::Scalar(scalar) => scalar.size().max(1),
            Shape::Aggregate { align, .. } => *align,
        }
    }
}

/// The IR type of one register's worth of a value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PartTy {
    I(u32),
    F32,
    F64,
    F80,
    Ptr,
    /// Two floats in one SSE register (System V).
    V2F32,
    F128,
    /// An aggregate passed as one: `[N x T]` (AAPCS64's pairs of words and floating-point
    /// aggregates), or `{ T, T }` (a floating-point aggregate returned by AAPCS64, or passed or
    /// returned by `__vectorcall`).
    Array(Elem, u8),
    Struct(Elem, u8),
}

/// An element of an aggregate part.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Elem {
    I64,
    F32,
    F64,
}

impl Elem {
    fn ir(self) -> &'static str {
        match self {
            Elem::I64 => "i64",
            Elem::F32 => "float",
            Elem::F64 => "double",
        }
    }

    fn size(self) -> u64 {
        if self == Elem::F32 { 4 } else { 8 }
    }
}

impl fmt::Display for PartTy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartTy::I(bits) => write!(f, "i{bits}"),
            PartTy::F32 => f.write_str("float"),
            PartTy::F64 => f.write_str("double"),
            PartTy::F80 => f.write_str("x86_fp80"),
            PartTy::Ptr => f.write_str("ptr"),
            PartTy::V2F32 => f.write_str("<2 x float>"),
            PartTy::F128 => f.write_str("fp128"),
            PartTy::Array(elem, count) => write!(f, "[{count} x {}]", elem.ir()),
            PartTy::Struct(elem, count) => write!(f, "{{ {} }}", vec![elem.ir(); usize::from(*count)].join(", ")),
        }
    }
}

impl PartTy {
    pub fn size(self) -> u64 {
        match self {
            PartTy::I(bits) => bits.div_ceil(8) as u64,
            PartTy::F32 => 4,
            PartTy::F64 | PartTy::Ptr | PartTy::V2F32 => 8,
            PartTy::F80 => 10,
            PartTy::F128 => 16,
            PartTy::Array(elem, count) | PartTy::Struct(elem, count) => elem.size() * u64::from(count),
        }
    }
}

/// How a small integer is widened in a register (System V's de facto rule, which Clang's
/// callers keep and its callees rely on).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ext {
    None,
    Zero,
    Sign,
    /// `[CG-LL-3]` — a reference: never null (`[BRW-9]`); its referent's `size` bytes allocated for
    /// the whole call (ODR-276: no callee can end what a borrowed parameter points into, the
    /// caller's loan keeping it alive); and `readonly` when it is shared and its referent holds no
    /// cell (ODR-275: nothing a shared reference reaches changes but in an `UnsafeCell` or a
    /// `Cell`).
    Ref { size: u64, readonly: bool },
    /// `[CG-LL-3]`, `[SIMD-3]` — a view's elements a derived fact proves apart from every other
    /// pointer the function reaches them by (the C backend's `restrict`).
    NoAlias,
    /// `[FFI-9]` — a `__vectorcall` floating-point aggregate, in vector registers.
    InReg,
    /// AAPCS64 — the stack alignment of a floating-point aggregate that finds no register left.
    AlignStack(u8),
}

impl Ext {
    pub fn attr(self) -> String {
        match self {
            Ext::None => String::new(),
            Ext::Zero => " zeroext".to_string(),
            Ext::Sign => " signext".to_string(),
            Ext::Ref { size, readonly } => format!(" nonnull dereferenceable({size}){}", if readonly { " readonly" } else { "" }),
            Ext::NoAlias => " noalias".to_string(),
            Ext::InReg => " inreg".to_string(),
            Ext::AlignStack(align) => format!(" alignstack({align})"),
        }
    }
}

/// One IR parameter (or result element) made of the value's bytes at `offset`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Part {
    pub offset: u64,
    pub ty: PartTy,
    pub ext: Ext,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Pass {
    /// Nothing is passed (a value of no bytes).
    Ignore,
    /// In registers (or the stack slots that stand for them): each part loaded from the
    /// value's bytes. A scalar is its one part.
    Direct(Vec<Part>),
    /// Behind a pointer to a copy the caller makes; System V marks it `byval`, which makes the
    /// copy the callee's own stack slot.
    Indirect { size: u64, align: u64, byval: bool },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Return {
    Void,
    /// In registers: one part, or a literal struct of two.
    Direct(Vec<Part>),
    /// Into memory the caller gives, its address the hidden first parameter.
    Sret { size: u64, align: u64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Signature {
    pub ret: Return,
    pub params: Vec<Pass>,
    pub conv: Conv,
}

impl Return {
    /// The IR return type.
    pub fn ir(&self) -> String {
        match self {
            Return::Void | Return::Sret { .. } => "void".to_string(),
            Return::Direct(parts) if parts.len() == 1 => parts[0].ty.to_string(),
            Return::Direct(parts) => {
                format!("{{ {} }}", parts.iter().map(|part| part.ty.to_string()).collect::<Vec<_>>().join(", "))
            }
        }
    }
}

/// The signature of a function returning `ret` (`None`: C's `void`) and taking `params`.
pub fn classify(target: Target, ret: Option<&Shape>, params: &[Shape]) -> Signature {
    match target {
        Target::Win64 => Signature {
            ret: ret.map_or(Return::Void, win64_return),
            params: params.iter().map(win64_param).collect(),
            conv: Conv::C,
        },
        Target::SysV => sysv(ret, params),
        Target::Aapcs64 => Signature {
            ret: ret.map_or(Return::Void, aapcs64_return),
            params: params.iter().map(aapcs64_param).collect(),
            conv: Conv::C,
        },
    }
}

/// A homogeneous floating-point aggregate: one to four leaves of one floating-point type, one
/// after the other, the whole value (C's and Clang's rule for `__vectorcall`'s vector aggregates
/// and AAPCS64's floating-point ones); never one holding a `std.simd` vector.
fn homogeneous(shape: &Shape) -> Option<(Elem, u8)> {
    let Shape::Aggregate { size, leaves, unaligned: false, vectors: false, .. } = shape else { return None };
    let &(_, first) = leaves.first()?;
    let elem = match first {
        Scalar::F32 => Elem::F32,
        Scalar::F64 => Elem::F64,
        _ => return None,
    };
    let count = leaves.len() as u64;
    let packed = leaves.iter().enumerate().all(|(index, &(at, scalar))| scalar == first && at == index as u64 * elem.size());
    (count <= 4 && packed && *size == count * elem.size()).then_some((elem, count as u8))
}

// -- Windows x64's `__vectorcall` -------------------------------------------------------------

/// `[FFI-9]` — `__vectorcall` on Windows x64, as Clang classifies it (`WinX86_64ABIInfo`): the
/// first six arguments may take vector registers, six of them in all, a floating-point scalar its
/// position's; a homogeneous floating-point aggregate takes as many as it has members, from those
/// left once every scalar has its own, else it goes behind a pointer to a copy; a result of up to
/// four members comes back in them. Anything else is the C convention's.
pub fn classify_vectorcall(ret: Option<&Shape>, params: &[Shape]) -> Signature {
    let ret = match ret.map(|shape| (shape, homogeneous(shape))) {
        None => Return::Void,
        Some((_, Some((elem, count)))) => Return::Direct(vec![Part { offset: 0, ty: PartTy::Struct(elem, count), ext: Ext::None }]),
        Some((shape, None)) => win64_return(shape),
    };
    let mut free = 6u8;
    let mut passes: Vec<Option<Pass>> = params
        .iter()
        .enumerate()
        .map(|(position, shape)| match (shape, homogeneous(shape)) {
            // An aggregate's turn comes once the scalars have theirs.
            (Shape::Aggregate { .. }, Some(_)) => None,
            (Shape::Scalar(Scalar::F32 | Scalar::F64), _) => {
                if position < 6 && free > 0 {
                    free -= 1;
                }
                Some(win64_param(shape))
            }
            _ => Some(win64_param(shape)),
        })
        .collect();
    for (pass, shape) in passes.iter_mut().zip(params) {
        if pass.is_some() {
            continue;
        }
        let (elem, count) = homogeneous(shape).expect("left for this pass");
        *pass = Some(if free >= count {
            free -= count;
            Pass::Direct(vec![Part { offset: 0, ty: PartTy::Struct(elem, count), ext: Ext::InReg }])
        } else {
            Pass::Indirect { size: shape.size(), align: shape.align(), byval: false }
        });
    }
    Signature { ret, params: passes.into_iter().map(|pass| pass.expect("every parameter classified")).collect(), conv: Conv::VectorCall }
}

/// `[FFI-9]` — the symbol a `__vectorcall` function has on Windows x64: its name, `@@`, and the
/// bytes its parameters take, each rounded up to eight (`scale@@16`), as MSVC decorates it.
pub fn vectorcall_symbol(name: &str, params: &[Shape]) -> String {
    let bytes: u64 = params.iter().map(|shape| shape.size().max(1).div_ceil(8) * 8).sum();
    format!("{name}@@{bytes}")
}

// -- AArch64's AAPCS64 ------------------------------------------------------------------------

/// AAPCS64 (Clang's `AArch64ABIInfo` for Linux): a scalar as itself, never extended; a
/// floating-point aggregate as an array of its members (on the stack aligned to 8, or 16 for one
/// aligned so); another aggregate of up to 16 bytes as one or two 64-bit words (one 128-bit word
/// when it is aligned to 16); anything larger behind a pointer to the caller's copy.
fn aapcs64_param(shape: &Shape) -> Pass {
    match shape {
        Shape::Scalar(scalar) => Pass::Direct(vec![Part { offset: 0, ty: scalar.ir(), ext: Ext::None }]),
        Shape::Aggregate { size: 0, .. } => Pass::Ignore,
        Shape::Aggregate { align, .. } if let Some((elem, count)) = homogeneous(shape) => {
            let stack = if *align >= 16 { 16 } else { 8 };
            Pass::Direct(vec![Part { offset: 0, ty: PartTy::Array(elem, count), ext: Ext::AlignStack(stack) }])
        }
        Shape::Aggregate { size, align, .. } if *size <= 16 => {
            let word = if *align >= 16 { 16 } else { 8 };
            let words = size.div_ceil(word);
            let ty = match (word, words) {
                (16, _) => PartTy::I(128),
                (_, 1) => PartTy::I(64),
                _ => PartTy::Array(Elem::I64, words as u8),
            };
            Pass::Direct(vec![Part { offset: 0, ty, ext: Ext::None }])
        }
        Shape::Aggregate { size, align, .. } => Pass::Indirect { size: *size, align: *align, byval: false },
    }
}

/// AAPCS64's results: a floating-point aggregate as a struct of its members; another aggregate
/// of up to 8 bytes as the integer of its size, of up to 16 as two 64-bit words (one 128-bit word
/// when it is aligned to 16); anything larger into memory the caller gives.
fn aapcs64_return(shape: &Shape) -> Return {
    match shape {
        Shape::Scalar(scalar) => Return::Direct(vec![Part { offset: 0, ty: scalar.ir(), ext: Ext::None }]),
        Shape::Aggregate { size: 0, .. } => Return::Void,
        _ if let Some((elem, count)) = homogeneous(shape) => {
            Return::Direct(vec![Part { offset: 0, ty: PartTy::Struct(elem, count), ext: Ext::None }])
        }
        Shape::Aggregate { size, .. } if *size <= 8 => Return::Direct(vec![Part { offset: 0, ty: PartTy::I(*size as u32 * 8), ext: Ext::None }]),
        Shape::Aggregate { size, align, .. } if *size <= 16 => {
            let ty = if *align >= 16 { PartTy::I(128) } else { PartTy::Array(Elem::I64, 2) };
            Return::Direct(vec![Part { offset: 0, ty, ext: Ext::None }])
        }
        Shape::Aggregate { size, align, .. } => Return::Sret { size: *size, align: *align },
    }
}

fn scalar_part(scalar: Scalar) -> Part {
    let ext = match scalar {
        Scalar::Bool => Ext::Zero,
        _ => Ext::None,
    };
    Part { offset: 0, ty: scalar.ir(), ext }
}

// -- Windows x64 -------------------------------------------------------------------------------

/// Windows x64: a value of 1, 2, 4 or 8 bytes in one register (an aggregate as the integer of
/// its size); anything else behind a pointer to the caller's copy. A 128-bit integer is the
/// runtime's struct of two halves there (`ember_i128`), passed as one.
fn win64_param(shape: &Shape) -> Pass {
    match shape {
        Shape::Scalar(Scalar::Int { bytes: 16, .. }) => Pass::Indirect { size: 16, align: 16, byval: false },
        Shape::Scalar(scalar) => Pass::Direct(vec![scalar_part(*scalar)]),
        Shape::Aggregate { size: 0, .. } => Pass::Ignore,
        Shape::Aggregate { size: size @ (1 | 2 | 4 | 8), .. } => {
            Pass::Direct(vec![Part { offset: 0, ty: PartTy::I(*size as u32 * 8), ext: Ext::None }])
        }
        Shape::Aggregate { size, align, .. } => Pass::Indirect { size: *size, align: *align, byval: false },
    }
}

fn win64_return(shape: &Shape) -> Return {
    match win64_param(shape) {
        Pass::Ignore => Return::Void,
        Pass::Direct(parts) => Return::Direct(parts),
        Pass::Indirect { size, align, .. } => Return::Sret { size, align },
    }
}

// -- System V x86-64 ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Class {
    NoClass,
    Integer,
    Sse,
    X87,
    Memory,
}

fn merge(a: Class, b: Class) -> Class {
    use Class::*;
    match (a, b) {
        _ if a == b => a,
        (NoClass, other) | (other, NoClass) => other,
        (Memory, _) | (_, Memory) => Memory,
        (Integer, _) | (_, Integer) => Integer,
        (X87, _) | (_, X87) => Memory,
        _ => Sse,
    }
}

/// The two eightbytes' classes.
fn sysv_classes(shape: &Shape) -> (Class, Class) {
    match shape {
        Shape::Scalar(Scalar::Int { bytes: 16, .. }) => (Class::Integer, Class::Integer),
        Shape::Scalar(Scalar::F80) => (Class::X87, Class::NoClass),
        Shape::Scalar(scalar) if scalar.is_int_class() => (Class::Integer, Class::NoClass),
        Shape::Scalar(_) => (Class::Sse, Class::NoClass),
        Shape::Aggregate { size: 0, .. } => (Class::NoClass, Class::NoClass),
        Shape::Aggregate { size, unaligned, .. } if *size > 16 || *unaligned => (Class::Memory, Class::Memory),
        Shape::Aggregate { leaves, .. } => {
            let mut classes = [Class::NoClass; 2];
            for &(offset, scalar) in leaves {
                let class = match scalar {
                    Scalar::F80 => Class::Memory,
                    Scalar::F32 | Scalar::F64 => Class::Sse,
                    _ => Class::Integer,
                };
                let first = (offset / 8) as usize;
                let last = ((offset + scalar.size() - 1) / 8) as usize;
                for eightbyte in first..=last.min(1) {
                    classes[eightbyte] = merge(classes[eightbyte], class);
                }
            }
            if classes.contains(&Class::Memory) {
                return (Class::Memory, Class::Memory);
            }
            (classes[0], classes[1])
        }
    }
}

/// The IR type of the eightbyte at `offset` of an aggregate in `class` (Clang's
/// `GetINTEGERTypeAtOffset` and `GetSSETypeAtOffset`).
fn eightbyte_type(shape: &Shape, offset: u64, class: Class) -> PartTy {
    let Shape::Aggregate { size, leaves, .. } = shape else {
        unreachable!("only an aggregate is split into eightbytes")
    };
    let inside: Vec<(u64, Scalar)> =
        leaves.iter().copied().filter(|(at, scalar)| *at < offset + 8 && at + scalar.size() > offset).collect();
    match class {
        Class::Sse => match inside.as_slice() {
            [(at, Scalar::F64)] if *at == offset => PartTy::F64,
            [(a, Scalar::F32), (b, Scalar::F32)] if *a == offset && *b == offset + 4 => PartTy::V2F32,
            [(at, Scalar::F32)] if *at == offset => PartTy::F32,
            _ => PartTy::F64,
        },
        _ => {
            if let [(at, Scalar::Ptr)] = inside.as_slice()
                && *at == offset
            {
                return PartTy::Ptr;
            }
            // A 1-, 2- or 4-byte integer is the type when nothing but padding follows it in the
            // eightbyte; else an integer as wide as what is left of the value, at most 8 bytes.
            if let Some(&(at, first)) = inside.first()
                && at == offset
                && first.is_int_class()
                && matches!(first.size(), 1 | 2 | 4)
                && inside.len() == 1
            {
                return if first == Scalar::Bool { PartTy::I(8) } else { first.ir() };
            }
            if let Some(&(at, Scalar::Int { bytes: 8, .. })) = inside.first()
                && at == offset
            {
                return PartTy::I(64);
            }
            PartTy::I(((size - offset).min(8) * 8) as u32)
        }
    }
}

fn sysv_parts(shape: &Shape, classes: (Class, Class)) -> Vec<Part> {
    match shape {
        Shape::Scalar(Scalar::Int { bytes: 16, signed }) => {
            let _ = signed;
            vec![
                Part { offset: 0, ty: PartTy::I(64), ext: Ext::None },
                Part { offset: 8, ty: PartTy::I(64), ext: Ext::None },
            ]
        }
        Shape::Scalar(scalar) => {
            let ext = match scalar {
                Scalar::Bool => Ext::Zero,
                Scalar::Int { bytes: 1 | 2, signed: true } => Ext::Sign,
                Scalar::Int { bytes: 1 | 2, signed: false } => Ext::Zero,
                _ => Ext::None,
            };
            vec![Part { offset: 0, ty: scalar.ir(), ext }]
        }
        Shape::Aggregate { .. } => {
            let mut parts = Vec::new();
            for (index, class) in [classes.0, classes.1].into_iter().enumerate() {
                if class == Class::NoClass {
                    continue;
                }
                let offset = index as u64 * 8;
                parts.push(Part { offset, ty: eightbyte_type(shape, offset, class), ext: Ext::None });
            }
            parts
        }
    }
}

fn sysv(ret: Option<&Shape>, params: &[Shape]) -> Signature {
    let mut free_int = 6u32;
    let mut free_sse = 8u32;
    let ret = match ret {
        None => Return::Void,
        Some(shape) => match sysv_classes(shape) {
            (Class::NoClass, Class::NoClass) => Return::Void,
            (Class::Memory, _) => {
                // The hidden pointer takes the first integer register.
                free_int -= 1;
                Return::Sret { size: shape.size(), align: shape.align() }
            }
            (Class::X87, _) => Return::Direct(vec![Part { offset: 0, ty: PartTy::F80, ext: Ext::None }]),
            classes => Return::Direct(sysv_parts(shape, classes)),
        },
    };
    let params = params
        .iter()
        .map(|shape| {
            let classes = sysv_classes(shape);
            match classes {
                (Class::NoClass, Class::NoClass) => return Pass::Ignore,
                // `long double` is passed in memory; LLVM places an `x86_fp80` there itself.
                (Class::X87, _) => return Pass::Direct(vec![Part { offset: 0, ty: PartTy::F80, ext: Ext::None }]),
                (Class::Memory, _) => {
                    return Pass::Indirect { size: shape.size(), align: shape.align().max(8), byval: true };
                }
                _ => {}
            }
            let count = |class| [classes.0, classes.1].iter().filter(|c| **c == class).count() as u32;
            let (need_int, need_sse) = (count(Class::Integer), count(Class::Sse));
            if need_int <= free_int && need_sse <= free_sse {
                free_int -= need_int;
                free_sse -= need_sse;
                return Pass::Direct(sysv_parts(shape, classes));
            }
            match shape {
                // Clang passes an aggregate of at most eight bytes that finds no register left as
                // the integer of its size, which LLVM places on the stack as `byval` would.
                Shape::Aggregate { size, align, .. } if free_int == 0 && *size <= 8 && *align <= 8 => {
                    Pass::Direct(vec![Part { offset: 0, ty: PartTy::I(*size as u32 * 8), ext: Ext::None }])
                }
                // A scalar LLVM places on the stack itself.
                Shape::Scalar(scalar) => {
                    let mut parts = sysv_parts(shape, classes);
                    if let Scalar::Int { bytes: 16, .. } = scalar {
                        parts = vec![Part { offset: 0, ty: PartTy::I(128), ext: Ext::None }];
                    }
                    Pass::Direct(parts)
                }
                Shape::Aggregate { .. } => Pass::Indirect { size: shape.size(), align: shape.align().max(8), byval: true },
            }
        })
        .collect();
    Signature { ret, params, conv: Conv::C }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agg(size: u64, align: u64, leaves: &[(u64, Scalar)]) -> Shape {
        Shape::Aggregate { size, align, leaves: leaves.to_vec(), unaligned: false, vectors: false }
    }
    const I8: Scalar = Scalar::Int { bytes: 1, signed: true };
    const U16: Scalar = Scalar::Int { bytes: 2, signed: false };
    const U8: Scalar = Scalar::Int { bytes: 1, signed: false };
    const U32: Scalar = Scalar::Int { bytes: 4, signed: false };
    const I32: Scalar = Scalar::Int { bytes: 4, signed: true };
    const I64: Scalar = Scalar::Int { bytes: 8, signed: true };
    const U64: Scalar = Scalar::Int { bytes: 8, signed: false };

    fn loc() -> Shape { agg(16, 8, &[(0, Scalar::Ptr), (8, U32), (12, U32)]) }
    fn str_() -> Shape { agg(16, 8, &[(0, Scalar::Ptr), (8, U64)]) }
    fn vec_() -> Shape { agg(24, 8, &[(0, Scalar::Ptr), (8, U64), (16, U64)]) }
    fn b1() -> Shape { agg(1, 1, &[(0, U8)]) }
    fn b3() -> Shape { agg(4, 2, &[(0, U16), (2, U8)]) }
    fn f2() -> Shape { agg(8, 4, &[(0, Scalar::F32), (4, Scalar::F32)]) }
    fn di() -> Shape { agg(16, 8, &[(0, Scalar::F64), (8, I32)]) }
    fn f3() -> Shape { agg(12, 4, &[(0, Scalar::F32), (4, Scalar::F32), (8, Scalar::F32)]) }
    fn id() -> Shape { agg(16, 8, &[(0, I64), (8, Scalar::F64)]) }

    fn render(signature: &Signature) -> String {
        let ret = match &signature.ret {
            Return::Sret { .. } => "sret".to_string(),
            other => other.ir(),
        };
        let params: Vec<String> = signature
            .params
            .iter()
            .map(|pass| match pass {
                Pass::Ignore => "-".to_string(),
                Pass::Direct(parts) => parts.iter().map(|part| format!("{}{}", part.ty, part.ext.attr())).collect::<Vec<_>>().join(", "),
                Pass::Indirect { byval: true, .. } => "byval".to_string(),
                Pass::Indirect { byval: false, .. } => "ptr*".to_string(),
            })
            .collect();
        format!("{ret} ({})", params.join(", "))
    }

    /// The probe `take(bool, int8_t, uint16_t, loc, str, vec, b1, b3, f2, di, f3, id)` as Clang 22
    /// (Windows) and Clang 21 (Linux) lower it.
    #[test]
    fn cg_ll_2_matches_clang_on_both_targets() {
        let params = [
            Shape::Scalar(Scalar::Bool), Shape::Scalar(I8), Shape::Scalar(U16), loc(), str_(), vec_(), b1(), b3(), f2(), di(), f3(), id(),
        ];
        assert_eq!(
            render(&classify(Target::Win64, None, &params)),
            "void (i1 zeroext, i8, i16, ptr*, ptr*, ptr*, i8, i32, i64, ptr*, ptr*, ptr*)"
        );
        assert_eq!(
            render(&classify(Target::SysV, None, &params)),
            "void (i1 zeroext, i8 signext, i16 zeroext, ptr, i64, byval, byval, i8, i32, <2 x float>, byval, <2 x float>, float, byval)"
        );
        let returns = [(str_(), "{ ptr, i64 }", "sret"), (loc(), "{ ptr, i64 }", "sret"), (b3(), "i32", "i32"), (f2(), "<2 x float>", "i64"),
            (f3(), "{ <2 x float>, float }", "sret"), (di(), "{ double, i32 }", "sret")];
        for (shape, sysv, win64) in returns {
            assert_eq!(render(&classify(Target::SysV, Some(&shape), &[])), format!("{sysv} ()"));
            assert_eq!(render(&classify(Target::Win64, Some(&shape), &[])), format!("{win64} ()"));
        }
        // The fourth `str` finds no two registers left.
        let many = [str_(), str_(), str_(), str_()];
        assert_eq!(render(&classify(Target::SysV, None, &many)), "void (ptr, i64, ptr, i64, ptr, i64, byval)");
        // `__int128` in two registers on System V.
        let wide = Shape::Scalar(Scalar::Int { bytes: 16, signed: true });
        assert_eq!(render(&classify(Target::SysV, Some(&wide), std::slice::from_ref(&wide))), "{ i64, i64 } (i64, i64)");
        assert_eq!(render(&classify(Target::SysV, Some(&Shape::Scalar(Scalar::F80)), &[Shape::Scalar(Scalar::F80)])), "x86_fp80 (x86_fp80)");
        assert_eq!(render(&classify(Target::SysV, Some(&Shape::Scalar(Scalar::Bool)), &[])), "i1 zeroext ()".replace(" zeroext", ""));
    }

    const F32: Scalar = Scalar::F32;
    const F64: Scalar = Scalar::F64;

    fn hva2() -> Shape { agg(16, 8, &[(0, F64), (8, F64)]) }
    fn hva4() -> Shape { agg(32, 8, &[(0, F64), (8, F64), (16, F64), (24, F64)]) }
    fn hfa3() -> Shape { agg(12, 4, &[(0, F32), (4, F32), (8, F32)]) }
    /// Past 32 bytes no leaf is listed (`Types::aggregate_shape`).
    fn big5() -> Shape { agg(40, 8, &[]) }

    /// `[FFI-9]` — `__vectorcall` on Windows x64, as Clang 22 lowers the same C
    /// (`--target=x86_64-pc-windows-msvc`): a floating-point aggregate in vector registers while
    /// six are free (`inreg`), else behind a pointer; a scalar of the first six positions takes
    /// its own first; an aggregate of up to four floating-point members comes back in registers;
    /// MSVC's symbol counts each parameter's bytes, an aggregate passed by address its own.
    #[test]
    fn ffi_9_vectorcall_matches_clang() {
        let d = Shape::Scalar(F64);
        let mixed = agg(16, 8, &[(0, I32), (8, F64)]);
        let cases: [(&str, Option<Shape>, Vec<Shape>, &str, &str); 11] = [
            ("scale", Some(d.clone()), vec![d.clone(), d.clone()], "double (double, double)", "scale@@16"),
            ("ints", Some(d.clone()), vec![Shape::Scalar(I32), Shape::Scalar(I64), d.clone(), Shape::Scalar(I32), Shape::Scalar(F32)],
                "double (i32, i64, double, i32, float)", "ints@@40"),
            ("take_hva2", Some(d.clone()), vec![hva2()], "double ({ double, double } inreg)", "take_hva2@@16"),
            ("take_hva4", Some(d.clone()), vec![hva4()], "double ({ double, double, double, double } inreg)", "take_hva4@@32"),
            ("take_hfa3", Some(Shape::Scalar(F32)), vec![hfa3()], "float ({ float, float, float } inreg)", "take_hfa3@@16"),
            ("take_big5", Some(d.clone()), vec![big5()], "double (ptr*)", "take_big5@@40"),
            ("take_mixed", Some(d.clone()), vec![mixed], "double (ptr*)", "take_mixed@@16"),
            ("give_hva2", Some(hva2()), vec![d.clone()], "{ double, double } (double)", "give_hva2@@8"),
            ("give_hva4", Some(hva4()), vec![d.clone()], "{ double, double, double, double } (double)", "give_hva4@@8"),
            ("many", Some(d.clone()), vec![d.clone(), d.clone(), d.clone(), d.clone(), d.clone(), d.clone(), d.clone(), hva2()],
                "double (double, double, double, double, double, double, double, ptr*)", "many@@72"),
            ("after", Some(d.clone()), vec![Shape::Scalar(I32), hva2(), d.clone(), hfa3()],
                "double (i32, { double, double } inreg, double, { float, float, float } inreg)", "after@@48"),
        ];
        for (name, ret, params, ir, symbol) in cases {
            let signature = classify_vectorcall(ret.as_ref(), &params);
            assert_eq!(signature.conv, Conv::VectorCall);
            assert_eq!(render(&signature), ir, "{name}");
            assert_eq!(vectorcall_symbol(name, &params), symbol);
        }
    }

    /// `[CG-LL-2]` — AAPCS64, as Clang 22 lowers the same C (`--target=aarch64-linux-gnu`): no small
    /// integer extended; a floating-point aggregate as an array of its members; another of up to
    /// 16 bytes as one or two 64-bit words (one 128-bit word aligned to 16); a larger one behind a
    /// pointer to a copy; results likewise, a floating-point aggregate as a struct, a small one as
    /// the integer of its size, a large one into memory.
    #[test]
    fn cg_ll_2_aapcs64_matches_clang() {
        let d4 = hva4();
        let fa4 = agg(16, 4, &[(0, F32), (4, F32), (8, F32), (12, F32)]);
        let wide = agg(16, 16, &[(0, Scalar::Int { bytes: 16, signed: true })]);
        let i3 = agg(12, 4, &[(0, I32), (4, I32), (8, I32)]);
        let params = [
            Shape::Scalar(Scalar::Bool), Shape::Scalar(I8), Shape::Scalar(U16), loc(), str_(), vec_(), b1(), b3(), f2(), di(), f3(), id(),
            d4.clone(), big5(), fa4, wide, i3.clone(),
        ];
        assert_eq!(
            render(&classify(Target::Aapcs64, None, &params)),
            "void (i1, i8, i16, [2 x i64], [2 x i64], ptr*, i64, i64, [2 x float] alignstack(8), [2 x i64], [3 x float] alignstack(8), \
             [2 x i64], [4 x double] alignstack(8), ptr*, [4 x float] alignstack(8), i128, [2 x i64])"
        );
        let returns = [
            (str_(), "[2 x i64]"), (loc(), "[2 x i64]"), (b3(), "i32"), (f2(), "{ float, float }"), (f3(), "{ float, float, float }"),
            (di(), "[2 x i64]"), (d4.clone(), "{ double, double, double, double }"), (big5(), "sret"), (i3, "[2 x i64]"),
            (Shape::Scalar(Scalar::Int { bytes: 16, signed: true }), "i128"), (Shape::Scalar(Scalar::F128), "fp128"),
        ];
        for (shape, ir) in returns {
            assert_eq!(render(&classify(Target::Aapcs64, Some(&shape), &[])), format!("{ir} ()"));
        }
        // Five pairs of words: the fifth on the stack, still one array.
        assert_eq!(render(&classify(Target::Aapcs64, None, &[str_(), str_(), str_(), str_(), str_()])),
            "void ([2 x i64], [2 x i64], [2 x i64], [2 x i64], [2 x i64])");
        let d = Shape::Scalar(F64);
        let floats = [d.clone(), d.clone(), d.clone(), d.clone(), d.clone(), d.clone(), d.clone(), d.clone(), d4, Shape::Scalar(F32)];
        assert_eq!(render(&classify(Target::Aapcs64, None, &floats)),
            "void (double, double, double, double, double, double, double, double, [4 x double] alignstack(8), float)");
    }
}
