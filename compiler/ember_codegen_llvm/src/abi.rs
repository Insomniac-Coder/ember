//! `[CG-LL-2]`, `[FFI-9]` — the platform C ABI's classification of arguments and results,
//! done by the backend itself: Windows x64 and System V x86-64, exactly as Clang classifies
//! the same C types, so a function the IR defines and one the C runtime defines call each other
//! with nothing between. Every Ember function uses it, so a function value, a runtime callback
//! and an export are one convention.

use std::fmt;

/// The ABI the program is built for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    /// Windows x64 (`x86_64-pc-windows-msvc`).
    Win64,
    /// System V x86-64 (Linux, the BSDs).
    SysV,
}

impl Target {
    /// The host's: the IR is compiled by the Clang the build finds, for its default target.
    pub fn host() -> Target {
        if cfg!(windows) { Target::Win64 } else { Target::SysV }
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
    Ptr,
}

impl Scalar {
    pub fn size(self) -> u64 {
        match self {
            Scalar::Bool => 1,
            Scalar::Int { bytes, .. } => bytes as u64,
            Scalar::F32 => 4,
            Scalar::F64 | Scalar::Ptr => 8,
            Scalar::F80 => 16,
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
    /// A struct, union or array: its size, its alignment and each scalar leaf at its offset;
    /// `unaligned` when a leaf lies off its natural alignment (a `@packed` struct's).
    Aggregate { size: u64, align: u64, leaves: Vec<(u64, Scalar)>, unaligned: bool },
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
    /// `[CG-LL-3]` — a pointer the type system proves non-null (a reference).
    NonNull,
    /// `[CG-LL-3]`, `[SIMD-3]` — a view's elements a derived fact proves apart from every other
    /// pointer the function reaches them by (the C backend's `restrict`).
    NoAlias,
}

impl Ext {
    pub fn attr(self) -> &'static str {
        match self {
            Ext::None => "",
            Ext::Zero => " zeroext",
            Ext::Sign => " signext",
            Ext::NonNull => " nonnull",
            Ext::NoAlias => " noalias",
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
        },
        Target::SysV => sysv(ret, params),
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
    Signature { ret, params }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agg(size: u64, align: u64, leaves: &[(u64, Scalar)]) -> Shape {
        Shape::Aggregate { size, align, leaves: leaves.to_vec(), unaligned: false }
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
}
