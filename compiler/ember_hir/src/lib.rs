//! HIR: the AST after name resolution, type checking and desugaring
//! (Part XVIII §3).
//!
//! Differences from the AST, per the specification:
//!
//! * all names are `DefId`s or `LocalId`s; paths are resolved;
//! * every expression carries its `Ty`;
//! * `for` has become `while` plus iterator calls, operators on non-scalars
//!   have become interface method calls, `?` and `?.` have become `match`,
//!   f-strings have become `Formatter` calls, `elif` and the ternary have
//!   become nested `if`, named and default arguments are positional, and
//!   auto-ref/deref adjustments are explicit.
//!
//! Phase 0 populates the scalar and struct subset of this tree. The shape is
//! the full one so that later phases add cases rather than reshaping it.

use ember_span::{Span, Symbol};
use ember_types::{ClassId, EnumId, OverflowPolicy, RangeId, StructId, Ty};

/// A resolved item: a function, a struct, a constant.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DefId(pub u32);

/// A local slot within one function body. Parameters occupy the first slots,
/// in declaration order.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LocalId(pub u32);

#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
    /// The entry point, if this compilation unit declares one (`[FN-8]`).
    pub main: Option<DefId>,
}

impl Program {
    /// Functions are gathered module by module, and methods after that, so a
    /// `DefId` is not a position in this list — it is the identity assigned
    /// when the signature was collected.
    pub fn function(&self, def: DefId) -> &Function {
        self.functions
            .iter()
            .find(|f| f.def == def)
            .expect("every DefId names a function that was checked")
    }
}

/// `[FN-1]` — how a parameter is passed.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Mode {
    Borrow,
    Mut,
    Owned,
}

#[derive(Debug)]
pub struct LocalDecl {
    /// `None` for a compiler-introduced temporary.
    pub name: Option<Symbol>,
    pub ty: Ty,
    pub span: Span,
    /// `[CTL-2]` — this local is the iterator synthesized for a source `for`
    /// loop. The distinction survives desugaring so borrow diagnostics can
    /// report mutation of the loop's iterable as E3020/B2 instead of guessing
    /// from the compiler-private local's spelling.
    pub for_iterator: bool,
    /// `[RC-2e]` — this local is the borrowed counted-handle yield of a source
    /// `for` loop. An `owned fn` capture must copy the handle into its
    /// environment instead of preserving the compiler-internal reference.
    pub loop_borrowed_handle: bool,
}

#[derive(Debug)]
pub struct Param {
    pub local: LocalId,
    pub mode: Mode,
}

#[derive(Debug)]
pub struct Function {
    pub def: DefId,
    pub name: Symbol,
    /// `[CLS-2]` — this is a type-checked class `init` body whose direct
    /// field writes initialize freshly allocated storage. This is compiler
    /// metadata, not a source-level constructor trait or ABI flag.
    pub class_init: bool,
    /// `[CLS-2]` — field indices whose defaults are materialized
    /// before this class `init` body. Writes to these fields are ordinary
    /// overwrites and therefore retain `[OWN-5]` drop-before-store ordering.
    pub class_init_default_fields: Vec<usize>,
    /// The mangled symbol this function gets in the emitted C (`[MNG-1]`), or
    /// the name given by `@export` (`[MNG-2]`).
    pub symbol: String,
    /// Whether calling this function crosses Ember's explicit unsafe boundary.
    /// This remains part of an import-visible callable contract even when the
    /// body itself is otherwise unchanged.
    pub is_unsafe: bool,
    /// The declared external ABI, when this definition has one. `None` means
    /// Ember's ordinary callable ABI.
    pub abi: Option<String>,
    pub params: Vec<Param>,
    pub locals: Vec<LocalDecl>,
    pub ret: Ty,
    pub body: Block,
    pub span: Span,
    /// `[TYP-8]` — from `@overflow(...)` on this function, or the profile's
    /// default when the attribute is absent.
    pub overflow: OverflowPolicy,
    /// `[LT-1a]` — the parameter *positions* `@borrows(…)` names, when the
    /// attribute is written. `None` means elision decides (`[LT-1]`).
    ///
    /// Positions rather than names because that is what survives to MIR, where
    /// a parameter is a local and the borrow checker has to answer "which
    /// parameter did this reference come from".
    pub borrows: Option<Vec<usize>>,
    /// `[LT-1]` (ODR-024) — the source parameters' positions, from the
    /// declared signature (for an instantiation, the generic one).
    pub sources: Vec<usize>,
    /// Whether this is a lambda's body, which cannot carry `@borrows`.
    pub is_lambda: bool,
    /// An implicit derive (`[STR-5]`'s `clone`): emitted only when something
    /// the program keeps calls it (`[COST-1]`).
    pub emit_if_used: bool,
    /// Compiler-internal identity for a capturing closure's environment
    /// struct. It is absent for ordinary functions and capture-free closures.
    ///
    /// This is deliberately HIR/MIR metadata rather than a source-level type
    /// rule: region analysis uses it to connect a synthesized environment
    /// borrow to the verified access summary of the closure body (`[LT-42]`).
    pub closure_environment: Option<StructId>,
    /// Whether the generated closure environment owns its captured fields.
    ///
    /// This preserves the `owned fn` boundary through lowering. It is absent
    /// from ordinary function declarations and is compiler-internal: source
    /// observability comes from the closure's capture/move behavior, not from
    /// an ABI flag or a user-spellable environment type.
    pub closure_captures_by_move: bool,
    /// `[DSP-2]` — the class that declares this method, when this is a
    /// class method.  Kept as compiler metadata so MIR can select the
    /// runtime vtable without re-discovering the source declaration.
    pub class_owner: Option<ClassId>,
    /// `[DSP-2]` — the vtable slot assigned to a virtual or override method.
    /// `None` means ordinary static dispatch.
    pub class_virtual_slot: Option<usize>,
    /// A bodyless `virtual` declaration on an abstract class. It carries the
    /// checked callable and vtable signature but is never emitted as a C
    /// function; a concrete override supplies its slot implementation.
    pub is_abstract: bool,
}

impl Function {
    pub fn local(&self, id: LocalId) -> &LocalDecl {
        &self.locals[id.0 as usize]
    }
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug)]
pub enum Stmt {
    /// Bring a local into scope, optionally initialised.
    Let { local: LocalId, init: Option<Expr> },
    /// Store into a place.
    Assign { place: Expr, value: Expr },
    /// `[GRM-5]` — evaluate one aggregate value, then bind or assign its
    /// projected fields. `temp` is compiler-private and lives only for this
    /// source statement; keeping the operation explicit prevents a residual
    /// non-`Copy` field from being extended to block scope.
    Destructure { temp: LocalId, value: Expr, bindings: Vec<DestructureBinding> },
    Expr(Expr),
    Return(Option<Expr>),
    If { cond: Expr, then_block: Block, else_block: Option<Block> },
    /// `[CTL-4]` — `else` runs when the loop ends on its own, not on `break`.
    While { cond: Expr, body: Block, else_block: Option<Block> },
    /// `for i in a..b` over integers. `[CTL-3]` requires this to become a
    /// counted loop with no iterator object, so the range is kept apart here
    /// rather than desugared through an `Iterator`.
    ForRange {
        local: LocalId,
        start: Expr,
        end: Expr,
        inclusive: bool,
        body: Block,
        else_block: Option<Block>,
    },
    Block(Block),
    /// `[CTL-7]` — run this block at scope exit, last registered first.
    Defer(Block),
    /// How many loops out this targets: 0 is the innermost. A label is
    /// resolved to a depth while checking, so MIR never sees a name.
    Break { depth: usize },
    Continue { depth: usize },
}

#[derive(Debug)]
pub enum DestructureBinding {
    Let { local: LocalId, value: Expr },
    Assign { place: Expr, value: Expr },
}

#[derive(Debug)]
pub struct Expr {
    pub ty: Ty,
    pub kind: ExprKind,
    pub span: Span,
}

/// One ABI-visible slot in a `[TYP-22]` dynamic interface table. The HIR
/// retains this declaration-derived fact because interface declarations do
/// not otherwise cross the type checking to MIR boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceSlot {
    pub params: Vec<Ty>,
    pub ret: Ty,
}

/// One concrete method supplied to an erased `[TYP-22]` interface table.
/// The interface declaration establishes the erased slot signature; this
/// records only the concrete body and receiver mode that its adapter
/// must call.
#[derive(Clone, Debug)]
pub struct InterfaceAdapterSlot {
    pub implementation: DefId,
    pub receiver: Mode,
}

#[derive(Debug)]
pub enum ExprKind {
    /// An integer literal, already narrowed to `ty`.
    Int(u128),
    Float(f64),
    Bool(bool),
    Str(String),
    /// Reading a local, or naming it as a place.
    Local(LocalId),
    /// Field access by resolved index, not by name.
    Field { base: Box<Expr>, index: usize },
    /// A compiler-synthesized projection of one payload field from a known enum
    /// variant. Source programs select payloads through patterns; generated
    /// bodies use this form so `@derive(Clone)` can borrow a payload without
    /// binding and moving it out of its borrowed receiver.
    EnumField { base: Box<Expr>, variant: usize, index: usize },
    /// A direct call to a known function. `latebound` is set when a
    /// monomorphized callable parameter is statically dispatched to a known
    /// closure body; it preserves the expected callable boundary for region
    /// analysis even though the runtime call is direct.
    Call {
        callee: DefId,
        /// Operands are stored in declaration order for the callee ABI. When
        /// named arguments were written out of order, this records the
        /// parameter slots in source evaluation order so MIR can lower those
        /// expressions without changing `[EXP-1]`.
        arg_eval_order: Option<Vec<usize>>,
        args: Vec<Expr>,
        latebound: bool,
    },
    /// `[TYP-22]` — a call through a `ref dyn I` fat pointer.  Interface
    /// methods do not have an ordinary function body `DefId` at the call
    /// boundary: the receiver selects the concrete implementation through
    /// the interface vtable.  The slot is assigned from the interface's
    /// declaration order (including supertraits) during type checking.
    InterfaceCall {
        /// The complete ordered bound list of the erased carrier.  A call
        /// selects one interface's slot, but its table is the composition of
        /// every bound in `dyn I + J`.
        interfaces: Vec<Symbol>,
        interface: Symbol,
        slot: usize,
        /// Every declared slot in the selected interface table. A sized-only
        /// default is not callable through `dyn`, so it keeps its stable slot
        /// number but has no erased callable signature at this boundary.
        layout: Vec<Option<InterfaceSlot>>,
        receiver: Box<Expr>,
        args: Vec<Expr>,
        modes: Vec<Mode>,
        /// Source-order slots for named arguments; `None` is the common
        /// positional case. The receiver is evaluated before this list.
        arg_eval_order: Option<Vec<usize>>,
    },
    /// `[TYP-22]` — borrow a concrete implementer as `ref dyn I` or
    /// `ref mut dyn I`. This is distinct from an ordinary cast because
    /// lowering must retain the checked implementation table that turns the
    /// concrete receiver ABI into the interface's erased `void*` slot ABI.
    InterfaceUpcast {
        concrete: Ty,
        /// Every interface bound carried by the resulting erased value.
        interfaces: Vec<Symbol>,
        layout: Vec<Option<InterfaceSlot>>,
        implementations: Vec<Option<InterfaceAdapterSlot>>,
        expr: Box<Expr>,
    },
    /// `[TYP-22]` — allocate one concrete value behind an owning
    /// `Box[dyn I]`, retaining the checked dispatch/drop adapter metadata.
    DynBoxNew {
        concrete: Ty,
        /// Every interface bound carried by the resulting erased value.
        interfaces: Vec<Symbol>,
        layout: Vec<Option<InterfaceSlot>>,
        implementations: Vec<Option<InterfaceAdapterSlot>>,
        value: Box<Expr>,
    },
    /// `[FN-6]` — a named function used as a value. Its type is the
    /// `fn(A) -> R` it coerces to.
    FnValue(DefId),
    /// A call through a value of function type, rather than to a name. An
    /// `owned f: fn(...) -> R` parameter is `CallableOnce`, so this records
    /// that the callee itself must be consumed; its argument modes remain the
    /// callable type's own modes.
    CallIndirect {
        callee: Box<Expr>,
        args: Vec<Expr>,
        consumes_callee: bool,
        /// `[FN-6b]` — this call crosses an explicitly late-bound callable
        /// boundary and therefore gets fresh invocation-local regions in
        /// region analysis. It is compile-time metadata only.
        latebound: bool,
    },
    /// `Vec3(1, 2, 3)` — the memberwise constructor (`[STR-1]`). Arguments are
    /// in declaration order with defaults already filled in.
    StructLit { struct_id: StructId, fields: Vec<Expr> },
    /// `(a, b)` — a tuple value. Its elements are read by `Field`, whose
    /// index is the tuple position.
    TupleLit(Vec<Expr>),
    /// `[a, b, c]` — an array value with each element written out.
    ArrayLit(Vec<Expr>),
    /// `[value; count]` — an array value with one element repeated. `count`
    /// is already evaluated; it is the same `N` as the type's length.
    ArrayRepeat { value: Box<Expr>, count: u64 },
    /// `base[index]` on a fixed array. The bounds check is inserted during
    /// MIR lowering, not here.
    Index { base: Box<Expr>, index: Box<Expr> },
    /// `Shape.Circle(1.0)` — one enum variant with its payload in field
    /// order (`[ENM-1]`). A unit variant has no fields.
    EnumLit { enum_id: EnumId, variant: usize, fields: Vec<Expr> },
    /// `match` (`[ENM-2]`). A statement `match` is this with type `void`,
    /// wrapped in `Stmt::Expr`.
    Match { scrutinee: Box<Expr>, arms: Vec<MatchArm> },
    /// The address of a place. A `mut` argument is passed this way, which is
    /// what makes `mut` an inout parameter rather than a copy.
    Ref { place: Box<Expr>, mutable: bool },
    /// Reading through a reference. A `mut` parameter is a `ref mut T` inside
    /// the function, so every mention of its name is one of these.
    Deref(Box<Expr>),
    /// `[LEX-19]` — an f-string. Building it needs several statements, so it
    /// stays a single node until MIR, which has somewhere to put them.
    /// `buffer_ref` is `ref mut String`, interned here because MIR cannot
    /// intern types of its own.
    FString { parts: Vec<FStringPart>, buffer_ref: Ty },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Unary { op: UnOp, operand: Box<Expr> },
    /// `x as T` — an explicit numeric conversion (`[TYP-6]`).
    Cast { expr: Box<Expr>, to: Ty },
    /// A lossless widening inserted at a coercion site (`[TYP-5]`). Kept
    /// distinct from `Cast` because it is implicit and always lossless.
    Widen { expr: Box<Expr>, to: Ty },
    /// `[TYP-5]` **range erasure**: a value of a range type read as its
    /// representation. Kept distinct from `Widen` because it changes no bits
    /// — `[COST-3]` classes a range type as *not observable* — and from
    /// `Cast` because it is implicit and cannot fail. Lowering drops it.
    EraseRange(Box<Expr>),
    /// A call the compiler knows about directly, before `std` exists.
    Builtin { which: Builtin, args: Vec<Expr> },
    /// A block used as an expression: its statements run, then `value` is
    /// the result, and the block's locals end after it is read. The checker
    /// builds these for calls that loop or bind (`sum`, `min`, `[STD-26]`),
    /// so each argument is evaluated once.
    Block { block: Block, value: Box<Expr> },
    /// `[CLS-1]` — allocate a class object and invoke its checked constructor.
    /// The arguments are stored in constructor-parameter order; when source
    /// named arguments were out of order, `arg_eval_order` tells MIR how to
    /// materialize them without changing `[EXP-1]` source evaluation order.
    /// `default_fields` contains the field defaults that are
    /// initialized before a user-defined constructor body runs.
    ClassNew {
        class_id: ClassId,
        init: DefId,
        arg_eval_order: Option<Vec<usize>>,
        default_fields: Vec<Option<Expr>>,
        args: Vec<Expr>,
    },
    /// A subexpression that failed to check. Absorbs errors.
    Error,
}

/// One piece of an f-string: literal text, or a value to format into it.
#[derive(Debug)]
pub enum FStringPart {
    Text(String),
    /// A value, and how `{x:spec}`, `{x!r}` or `{x=}` asks for it to be
    /// written (`[LEX-19]`); `None` is its plain text, as `print` writes it.
    Value(Expr, Option<FormatSpec>),
}

/// `[TXT-10]` (ODR-029) — what `parse` reads.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ParseKind {
    Signed,
    Unsigned,
    /// D-272 — `i128` and `u128`, whose bounds no 64-bit argument carries.
    I128,
    U128,
    F32,
    F64,
    Bool,
    Char,
}

/// `[LEX-19]` — a format spec in Python's mini-language,
/// `[[fill]align][sign][#][0][width][,|_][.precision][type]`, parsed. A
/// `kind` of `?` is `Debug` (`{x!r}`).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct FormatSpec {
    pub fill: char,
    pub align: Option<char>,
    pub sign: Option<char>,
    pub alternate: bool,
    pub zero: bool,
    pub width: u32,
    pub grouping: Option<char>,
    pub precision: Option<u32>,
    pub kind: Option<char>,
}

impl FormatSpec {
    /// No spec at all: the value's plain text.
    pub const PLAIN: FormatSpec = FormatSpec {
        fill: ' ',
        align: None,
        sign: None,
        alternate: false,
        zero: false,
        width: 0,
        grouping: None,
        precision: None,
        kind: None,
    };

    pub fn parse(text: &str) -> Result<FormatSpec, String> {
        fn number(chars: &[char], at: &mut usize) -> Result<Option<u32>, String> {
            let start = *at;
            while chars.get(*at).is_some_and(char::is_ascii_digit) {
                *at += 1;
            }
            if *at == start {
                return Ok(None);
            }
            let digits: String = chars[start..*at].iter().collect();
            digits.parse().map(Some).map_err(|_| format!("`{digits}` is too large"))
        }
        let chars: Vec<char> = text.chars().collect();
        let mut spec = FormatSpec::PLAIN;
        let mut at = 0;
        let is_align = |c: char| matches!(c, '<' | '>' | '^');
        if chars.len() >= 2 && is_align(chars[1]) {
            spec.fill = chars[0];
            spec.align = Some(chars[1]);
            at = 2;
        } else if chars.first().is_some_and(|&c| is_align(c)) {
            spec.align = Some(chars[0]);
            at = 1;
        }
        if let Some(&c) = chars.get(at)
            && matches!(c, '+' | '-' | ' ')
        {
            spec.sign = Some(c);
            at += 1;
        }
        if chars.get(at) == Some(&'#') {
            spec.alternate = true;
            at += 1;
        }
        if chars.get(at) == Some(&'0') {
            spec.zero = true;
            at += 1;
        }
        spec.width = number(&chars, &mut at)?.unwrap_or(0);
        if let Some(&c) = chars.get(at)
            && matches!(c, ',' | '_')
        {
            spec.grouping = Some(c);
            at += 1;
        }
        if chars.get(at) == Some(&'.') {
            at += 1;
            match number(&chars, &mut at)? {
                Some(precision) => spec.precision = Some(precision),
                None => return Err("a `.` needs a precision after it".to_string()),
            }
        }
        if let Some(&c) = chars.get(at)
            && "dxXboeEfFgG%s?".contains(c)
        {
            spec.kind = Some(c);
            at += 1;
        }
        if at != chars.len() {
            let rest: String = chars[at..].iter().collect();
            return Err(format!("`{rest}` is not part of a format spec"));
        }
        Ok(spec)
    }
}

#[derive(Debug)]
pub struct MatchArm {
    pub pattern: Pattern,
    /// `case p if c:` — a guard runs after the pattern matches, and a guard
    /// that fails falls through to the arms below.
    pub guard: Option<Expr>,
    pub body: MatchArmBody,
    pub span: Span,
}

/// `[GRM-10]` — statement arms are blocks, expression arms are expressions.
/// One `match` never mixes them; the parser rejects that with `E0103`.
#[derive(Debug)]
pub enum MatchArmBody {
    Block(Block),
    Expr(Expr),
}

/// A pattern with its type resolved and its bindings already given locals.
#[derive(Debug)]
pub struct Pattern {
    pub ty: Ty,
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum PatternKind {
    /// `_`, and anything that always matches.
    Wild,
    /// Binds whatever it matched to a local, then keeps testing `sub`.
    /// `by_ref` is `Some(mutable)` when the local is a reference to the
    /// matched place rather than a copy or move of it (`[GRM-13]`).
    Bind { local: LocalId, sub: Option<Box<Pattern>>, by_ref: Option<bool> },
    /// An integer, `bool` or `char` literal, already narrowed to `ty`.
    Int(i128),
    /// One enum variant, with a sub-pattern per payload field in order.
    Variant { enum_id: EnumId, variant: usize, fields: Vec<Pattern> },
    /// A tuple, struct or fixed array: one sub-pattern per element, in order.
    /// The type says which, so the shape does not have to be repeated here.
    Fields(Vec<Pattern>),
    /// `a | b` — matches if any alternative does. Every alternative binds the
    /// same names, which `[GRM-12]` requires.
    Or(Vec<Pattern>),
    Error,
}

/// `[STD-20]` (ODR-039) — the integer operators the `checked_`,
/// `wrapping_`, `saturating_` and `overflowing_` methods exist for, named as
/// their operator interfaces' methods are (`floordiv` is `//`, `rem` is `%`).
/// `pow` is built from `Mul`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum IntOp {
    Add,
    Sub,
    Mul,
    FloorDiv,
    Rem,
    Neg,
    Shl,
    Shr,
}

impl IntOp {
    pub fn named(name: &str) -> Option<IntOp> {
        Some(match name {
            "add" => IntOp::Add,
            "sub" => IntOp::Sub,
            "mul" => IntOp::Mul,
            "floordiv" => IntOp::FloorDiv,
            "rem" => IntOp::Rem,
            "neg" => IntOp::Neg,
            "shl" => IntOp::Shl,
            "shr" => IntOp::Shr,
            _ => return None,
        })
    }

    pub fn overflowing(self) -> &'static str {
        match self {
            IntOp::Add => "overflowing_add",
            IntOp::Sub => "overflowing_sub",
            IntOp::Mul => "overflowing_mul",
            IntOp::FloorDiv => "overflowing_floordiv",
            IntOp::Rem => "overflowing_rem",
            IntOp::Neg => "overflowing_neg",
            IntOp::Shl => "overflowing_shl",
            IntOp::Shr => "overflowing_shr",
        }
    }
}

/// `[STD-20]` — an integer's bit counts.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum IntBits {
    CountOnes,
    LeadingZeros,
    TrailingZeros,
}

impl IntBits {
    pub fn named(name: &str) -> Option<IntBits> {
        Some(match name {
            "count_ones" => IntBits::CountOnes,
            "leading_zeros" => IntBits::LeadingZeros,
            "trailing_zeros" => IntBits::TrailingZeros,
            _ => return None,
        })
    }

    pub fn method(self) -> &'static str {
        match self {
            IntBits::CountOnes => "count_ones",
            IntBits::LeadingZeros => "leading_zeros",
            IntBits::TrailingZeros => "trailing_zeros",
        }
    }
}

/// `[STD-20]`, `[STD-27]` — the float methods that are one C library function
/// each: the `f64` one by `c_name`, the `f32` one with an `f` after it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum FloatLib {
    Sqrt,
    Cbrt,
    Exp,
    Exp2,
    Ln,
    Log2,
    Log10,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Floor,
    Ceil,
    Trunc,
    /// Half to even, as Python's `round`: C's `nearbyint` in the default
    /// rounding mode.
    Round,
    /// C's `round`.
    RoundHalfAway,
    Atan2,
    Hypot,
    Pow,
    Copysign,
    /// C's `fma`: one rounding (`[STD-3]`).
    MulAdd,
    /// C's classification macros, the same name for both widths; `bool`.
    IsNan,
    IsFinite,
    IsInfinite,
}

impl FloatLib {
    /// The method, as Ember writes it, and the function's arity with the
    /// receiver.
    pub const ALL: [(FloatLib, &'static str, usize); 29] = [
        (FloatLib::Sqrt, "sqrt", 1),
        (FloatLib::Cbrt, "cbrt", 1),
        (FloatLib::Exp, "exp", 1),
        (FloatLib::Exp2, "exp2", 1),
        (FloatLib::Ln, "ln", 1),
        (FloatLib::Log2, "log2", 1),
        (FloatLib::Log10, "log10", 1),
        (FloatLib::Sin, "sin", 1),
        (FloatLib::Cos, "cos", 1),
        (FloatLib::Tan, "tan", 1),
        (FloatLib::Asin, "asin", 1),
        (FloatLib::Acos, "acos", 1),
        (FloatLib::Atan, "atan", 1),
        (FloatLib::Sinh, "sinh", 1),
        (FloatLib::Cosh, "cosh", 1),
        (FloatLib::Tanh, "tanh", 1),
        (FloatLib::Floor, "floor", 1),
        (FloatLib::Ceil, "ceil", 1),
        (FloatLib::Trunc, "trunc", 1),
        (FloatLib::Round, "round", 1),
        (FloatLib::RoundHalfAway, "round_half_away", 1),
        (FloatLib::Atan2, "atan2", 2),
        (FloatLib::Hypot, "hypot", 2),
        (FloatLib::Pow, "pow", 2),
        (FloatLib::Copysign, "copysign", 2),
        (FloatLib::MulAdd, "mul_add", 3),
        (FloatLib::IsNan, "is_nan", 1),
        (FloatLib::IsFinite, "is_finite", 1),
        (FloatLib::IsInfinite, "is_infinite", 1),
    ];

    pub fn named(method: &str) -> Option<(FloatLib, usize)> {
        FloatLib::ALL.iter().find(|(_, name, _)| *name == method).map(|&(f, _, arity)| (f, arity))
    }

    pub fn method(self) -> &'static str {
        FloatLib::ALL.iter().find(|(f, _, _)| *f == self).map_or("?", |(_, name, _)| name)
    }

    /// The `f64` function's C name.
    pub fn c_name(self) -> &'static str {
        match self {
            FloatLib::Ln => "log",
            FloatLib::Round => "nearbyint",
            FloatLib::RoundHalfAway => "round",
            FloatLib::MulAdd => "fma",
            FloatLib::IsNan => "isnan",
            FloatLib::IsFinite => "isfinite",
            FloatLib::IsInfinite => "isinf",
            other => other.method(),
        }
    }

    /// A classification macro: one name for both widths, and an `int`.
    pub fn is_predicate(self) -> bool {
        matches!(self, FloatLib::IsNan | FloatLib::IsFinite | FloatLib::IsInfinite)
    }
}

/// Functions the compiler provides itself in Phase 0, before the standard
/// library is written in Ember. Each lowers to one `ember_rt` call.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Builtin {
    /// `println(x)` for a scalar or `str`.
    Println,
    /// `[STD-9]` — `eprintln(x)` and `eprint(x)`: the same, to standard error.
    EPrintln,
    EPrint,
    /// VI.6, `[PAN-1]` (0.9.9) — `panic(msg)`, `todo()`, `unreachable()`:
    /// type `Never`; its one argument is the `str` message.
    Panic,
    /// VI.6 — `assert(cond, msg)` and the assertions built on it: panics with
    /// `msg` when `cond` is false.
    Assert,
    /// `[TYP-37]` — `a < b` in `Ord`'s order: IEEE totalOrder for floats, so
    /// `-0.0 < +0.0` and NaN sits at an end; `<` for every other scalar. What
    /// `min`, `max` and `clamp` compare with.
    TotalLess,
    /// `abs` of a float: clears the sign, so `abs(-0.0)` is `+0.0`.
    FloatAbs,
    /// `[TYP-30]` — a float `**`: C's `pow` or `powf`.
    FloatPow,
    /// `[STD-20]`, `[STD-27]` — a float method that is one C library
    /// function: the receiver, then the method's arguments.
    FloatLib(FloatLib),
    /// `[STD-20]` (ODR-039) — an integer operation as a `(T, bool)`: the
    /// result modulo 2^N and whether it wrapped. A zero divisor still panics.
    /// MIR computes it in place, as the checked operators do.
    IntOverflowing(IntOp),
    /// `[STD-20]` — a bit count of an integer over its type's width, as an
    /// `int`.
    IntBits(IntBits),
    /// `[STD-26]` — how many values `range(start, stop, step)` has, and the
    /// `k`th of them; a zero step panics.
    RangeCount,
    RangeNth,
    /// `[TXT-10]` — `s.char_count()`: the number of Unicode scalar values.
    StrCharCount,
    /// `[TXT-10]` — the `str` searches and builders of the runtime.
    StrStartsWith,
    StrEndsWith,
    StrFind { reverse: bool },
    StrCount,
    StrReplace,
    StrRepeat,
    StrTrimStart,
    StrTrimEnd,
    StrSliceOk,
    StrToUpper,
    /// `[TXT-10]` (ODR-029) — `parse`: the status (0 when the text is a
    /// literal of the kind that fits) and, once it is 0, the value.
    ParseStatus { kind: ParseKind },
    ParseValue { kind: ParseKind },
    StrToLower,
    /// `[CTL-1]` — the `char` that starts at a byte index of a `str`.
    StrCharAt,
    /// How many bytes UTF-8 gives a `char`.
    CharUtf8Len,
    /// `[LEX-19]` — append a value to an f-string's buffer as its spec says.
    FormatWith(FormatSpec),
    /// `[STD-8b]` — `needle in text`, for a `str` needle and a `char` one.
    StrContains,
    StrContainsChar,
    /// `[STD-15]` — `xs.sort()` (stable, in `Ord`'s order), `xs.reverse()`,
    /// `xs.clear()` (dropping each element), `xs.pop() -> Option[T]`,
    /// `xs.remove(i) -> T`, `xs.insert(i, v)` (its arguments are the array,
    /// the value, then the index), and `sorted(xs) -> Array[T]` over an
    /// `Array` or a view.
    ArraySort,
    ArrayReverse,
    ArrayClear,
    ArrayPop { option: Ty },
    ArrayRemove,
    /// `[STD-15]` (ODR-031) — `xs.drain(r)` with the range already checked
    /// and resolved to `lo`, `hi`: the elements `lo..hi` moved out, in order,
    /// into a new `Array`, and the rest closed up.
    ArrayDrain,
    ArrayInsert,
    /// `[STD-15]` — `xs.capacity()`: the slots allocated.
    ArrayCapacity,
    /// `[STD-15]` — `xs.reserve(n)`: room for `n` more without reallocating.
    ArrayReserve,
    /// `[STD-15]` — `xs.truncate(n)`: the elements from `n` on dropped.
    ArrayTruncate,
    /// `[STD-15]` — `xs.swap_remove(i)`: the element at `i`, the last one
    /// moved into its place.
    ArraySwapRemove,
    /// `[STD-15]`, `[BRW-5]` — `xs.swap(i, j)`.
    ArraySwap,
    /// `[STD-15]` — `xs.extend(items)`: a clone of each element of a span
    /// appended.
    ArrayExtend,
    ArraySorted { elem: Ty },
    /// `[OWN-8]` — `xs.clone()` on an `Array[T]` (or a `String`): a new
    /// buffer holding a clone of each element.
    ArrayClone { elem: Ty },
    /// `[STD-10]` — `input(prompt)`: a line of standard input as a `String`.
    Input,
    /// `print(x)` — the same without the newline.
    Print,
    /// `Array[T]()` — an empty growable array. Part XX.1 makes `Array` a
    /// compiler-known type until Phase 2's generics.
    ArrayNew,
    /// `[TYP-38]` (0.9.9) — `[a, b, c]` where an `Array[T]` is expected, or
    /// with no context: one allocation holding exactly the elements, moved
    /// out of the fixed array that is the only argument.
    ArrayFromLiteral,
    /// `[STR-5]`, `[TYP-37]` (0.9.9) — a comparison C cannot perform on the
    /// values themselves: the six comparisons of `str`/`String` (by bytes),
    /// and `==`/`!=` of structs, tuples, arrays, `Array`s and payload enums
    /// (field by field). The two operands are borrowed, never moved.
    ValueCompare { op: BinOp },
    /// `[CLS-1]` — allocate a class object, optionally followed by its
    /// compiler-known `init` method. The nominal class identity travels with
    /// the builtin so the backend can select the matching `TypeInfo` record;
    /// `init` is invoked after allocation and before the value is returned to
    /// source code. `None` retains the memberwise-construction path.
    ClassNew { class_id: ClassId, init: Option<DefId> },
    /// `[DSP-4]` — query a class handle's runtime type-info base chain.
    /// MIR performs the owning `Option` construction or forced-cast assertion.
    ClassDowncast {
        target: ClassId,
        target_ty: Ty,
        option: Option<EnumId>,
        forced: bool,
    },
    /// `[CLS-4]` — invoke the direct base constructor on an already allocated
    /// derived object. MIR lowers this to a borrowed base-handle adjustment
    /// plus the ordinary constructor call; it never reaches the backend.
    ClassSuperInit { base_id: ClassId, base_ty: Ty, init: DefId },
    /// `[HEAP-1]`, `[DRP-6]` — `Box(owned value)`. The payload and concrete
    /// box type travel with the operation so the backend can allocate exactly
    /// one `T` without recovering a compiler-private wrapper relationship.
    BoxNew { elem: Ty, boxed: Ty },
    /// `[HEAP-3]`, `[HEAP-6]` — `Shared(owned value)`. The payload and
    /// compiler-known handle type travel together so the backend can allocate
    /// one counted object with the ordinary class-header control block.
    SharedNew { elem: Ty, shared: Ty },
    /// `[WK-1]`/`[WK-2]` — `Weak(class_handle)`. The class and wrapper types
    /// travel with the operation so the backend can retain the control block
    /// without increasing the object's strong reference count.
    WeakNew { class: Ty, weak: Ty },
    /// `[WK-3]` — `weak.upgrade() -> Option[C]`. MIR branches on the runtime
    /// query and constructs the owned option payload exactly once.
    WeakUpgrade { class: Ty, option: EnumId },
    /// `[WK-1]` — `Weak[C].empty()` has no object/control-block allocation.
    WeakEmpty { weak: Ty },
    /// `a.push(x)`. The receiver is a `ref mut`, so it grows in place.
    ArrayPush,
    /// Part VI's slice row, `[TXT-4]` — `v[lo..hi]` of a view (`Span` or
    /// `str`), as a view of the same kind; `args` are the view and the two
    /// bounds, already checked. `text` slices a `str`.
    Slice { text: bool },
    /// `[TXT-4]` — whether a byte offset of a `str` starts a character (or
    /// is its end).
    StrIsCharBoundary,
    /// `[TXT-10]` — a `str` read as its `Span[u8]`: the same bytes, borrowed as
    /// the text is.
    StrAsBytes,
    /// `a.len()`.
    ArrayLen,
    /// `String()` — an empty string.
    StringNew,
    /// `s.push_str(other)`, appending UTF-8 bytes.
    StringPush,
    /// `s.len()` in bytes.
    StringLen,
    /// A `String` borrowed as a `str`, which is what `println` takes.
    StringAsStr,
    /// Append one formatted value to a `String`, for `[LEX-19]`'s f-strings.
    /// The runtime formatter is chosen from the value's type, as `println`'s
    /// is, until `Display` can be written.
    Format,
    /// `[UNS-5]`, `std.mem` — the raw memory primitives the collections are
    /// written on top of. Every one of these needs `unsafe` (`[UNS-1]`).
    /// `alloc[T](count) -> *mut T`.
    MemAlloc,
    /// `free[T](p, count)`.
    MemFree,
    /// `read[T](p, index) -> T`.
    PtrRead,
    /// `write[T](p, index, value)`.
    PtrWrite,
    /// `size_of[T]() -> usize`, which needs no `unsafe`.
    SizeOf,
    /// `align_of[T]() -> usize`, folded from the canonical target layout.
    AlignOf,
    /// `[OWN-6]` — atomically move the old value out of a mutable place and
    /// install an owned replacement without dropping the old value.
    MemReplace { elem: Ty },
    /// `[OWN-6]` — construct `T.default()`, then perform `MemReplace`.
    MemTake { elem: Ty, constructor: DefId },
    /// `[OWN-6]` — exchange two mutably borrowed places without moving either
    /// value into an Ember-visible temporary.
    MemSwap { elem: Ty },
    /// `[OWN-6]` — consume one value and deliberately suppress its drop.
    MemForget { elem: Ty },
    /// `[SPN-2]` — `s.len()`. The length field of the view.
    SpanLen,
    /// `[SPN-2]` — `s.get(i) -> Option[ref T]`, "the checked-without-panic
    /// form".
    SpanGet,
    /// `[SPN-2]` — `unsafe s.get_unchecked(i)`.
    SpanGetUnchecked,
    /// `[SPN-1]` — an `Array[T]` or a `[T; N]` viewed as a `Span[T]` or a
    /// `MutSpan[T]`. Not a conversion: the view points into the container, and
    /// the borrow checker keeps the container borrowed for the view's region.
    SpanFrom { mutable: bool },
    /// `[BRW-5]`, `[SPN-3]` — split one mutable borrow of an `Array[T]` into
    /// two disjoint `MutSpan[T]` values at a checked boundary. The tuple type
    /// travels with the builtin so the C backend can construct the structural
    /// result without inventing a public helper ABI.
    ArraySplitAtMut { elem: Ty, pair: Ty },
    /// `[SPN-3]` — split one Span borrow into two same-kind, disjoint views at
    /// a checked boundary. A mutable receiver is reborrowed rather than moved.
    SpanSplitAt { elem: Ty, pair: Ty, mutable: bool },
    /// `[SPN-3]` — explicitly reborrow a move-only mutable span.
    SpanReborrow,
    /// `[SPN-5]`, `[SPN-6]` — form a shared `Span[T]` reborrow from a
    /// `MutSpan[T]`. The result remains tied to the source, so holding a
    /// shared iterator/chunk iterator freezes conflicting mutation without
    /// consuming the original mutable view.
    SpanSharedReborrow,
    /// `[SPN-5]` — advance a named Span element iterator and yield the next
    /// shared or mutable element reference.
    SpanIterNext { elem: Ty, mutable: bool },
    /// `[SPN-6]`, `[SPN-7]` — check a non-zero width and construct the named
    /// shared or mutable chunk iterator. `iterator` is the ordinary public
    /// library struct type receiving the source/cursor/width fields.
    SpanChunksNew { iterator: Ty, mutable: bool },
    /// `[SPN-6]` — advance a chunk cursor and yield one final-partial-capable
    /// shared or mutable subspan.
    SpanChunksNext { elem: Ty, mutable: bool },
    /// `[STD-15]` (ODR-031) — `windows(n)`: check a non-zero width and
    /// construct the shared window iterator, lowered as `SpanChunksNew` is.
    SpanWindowsNew { iterator: Ty },
    /// `[STD-15]` (ODR-031) — advance a window cursor by one and yield the
    /// next full window as a shared subspan, or nothing once fewer than the
    /// width remain.
    SpanWindowsNext { elem: Ty },
    /// `[SPN-8]`, `[SPN-9]` — safely extract a raw pointer from a Span. The
    /// pointer result deliberately carries no safe borrow or lifetime tie.
    SpanAsPtr { mutable: bool },
    /// `[RNG-3]` — `T.checked(v) -> Result[T, RangeError]`. Two compares and a
    /// branch; the `Ok` payload is the value unchanged, because `[COST-3]`
    /// makes a range type its representation's bits.
    RangeChecked(RangeId),
    /// `[RNG-3a]` — `T.clamped(v) -> T`, **total**: no failure mode, no
    /// `Panic`, no `RuntimeCheck(k)`. "On the C backend it lowers to two
    /// compares or the target's `min`/`max` instruction pair, strictly cheaper
    /// than `[RNG-3]`'s check-and-branch-to-panic."
    RangeClamped(RangeId),
    /// `[RNG-10]` — `unsafe T.new_unchecked(v) -> T`, the one route outside
    /// the closed construction set. `[RNG-9]` makes an out-of-range value
    /// undefined behaviour, which is the caller's obligation.
    RangeNewUnchecked(RangeId),
    /// `[CELL-1]` — `c.set(owned v)`. Three of `Cell`'s five members need a
    /// carrier of their own rather than an ordinary expression, and all three
    /// need it for the same reason: they move the payload out of the cell and
    /// put something back, which is a sequence of statements and not a value.
    ///
    /// `set` in particular could not be an assignment even if it were one
    /// statement. `[OWN-5]` makes an assignment drop the old value **before**
    /// storing the new one, and `[CELL-1]` requires the opposite order —
    /// "`set` and `replace` MUST store the new value before dropping the old
    /// one", because a drop can re-enter the same cell and read it while it is
    /// torn. Lowering these itself is what keeps the two rules apart; see
    /// ADR-020's closing paragraph.
    CellSet,
    /// `[CELL-1]` — `c.replace(owned v) -> T`. As `set`, except the old value
    /// is handed back rather than dropped, so the cell is never torn at all.
    CellReplace,
    /// `[CELL-1]` — `c.into_inner() -> T`, taking `owned self`. The cell is
    /// consumed, so its payload is moved out and the cell itself must not be
    /// dropped afterwards — the payload is the only thing it owned.
    CellIntoInner,
    /// `[CELL-1]` — `c.update(f)`, which is `set(f(get()))` and so needs the
    /// receiver twice. A HIR expression cannot be duplicated, so the cell and
    /// the function travel here as a pair and lowering, which holds a `Place`,
    /// uses it for both the read and the store.
    CellUpdate,
    /// `[CELL-1]` — the non-`Copy`, `T: Default` update arm. Lowering moves
    /// the old payload behind a default replacement before invoking the
    /// callback, so callback re-entry never observes an uninitialized cell.
    CellUpdateDefault { constructor: DefId },
    /// `[CELL-1]` — `c.take()`, available for `T: Default`. The selected
    /// receiver-less constructor is carried into MIR so lowering can replace
    /// the payload without rediscovering interface dispatch.
    CellTake { constructor: DefId },
    /// `[ARN-8]` — `MaybeUninit[T].uninit()`. The value owns storage with
    /// `T`'s layout without claiming that an initialized `T` is present.
    MaybeUninitUninit { inner: Ty },
    /// `[ARN-8]` — `slot.write(owned value) -> ref mut T`. Lowering performs
    /// a store into the payload field without reading or dropping its prior
    /// bytes, then returns a reference to the newly initialized value.
    MaybeUninitWrite { inner: Ty },
    /// `[ARN-8]`, `[ARN-8a]` — unsafe extraction of the initialized payload.
    /// The wrapper is consumed and its field becomes ordinary owned `T`.
    MaybeUninitAssumeInit { inner: Ty },
    /// `[UNS-10]` — derive a mutable raw pointer to an `UnsafeCell[T]` payload
    /// through shared access. Type checking admits this only in `unsafe`; the
    /// result is a raw pointer, never a safe reference or borrow-checker
    /// exemption.
    UnsafeCellGet { inner: Ty },
    /// `[UNS-10]` — consume an `UnsafeCell[T]` and move out its payload.
    UnsafeCellIntoInner,
    /// `[ARN-9]` — initialize one element of a
    /// `MutSpan[MaybeUninit[T]]` without dropping prior bytes.
    MaybeUninitWriteAt { inner: Ty },
    /// `[ARN-9]` — consume a fully initialized uninitialized-storage span and
    /// expose the same allocation as `MutSpan[T]`.
    MaybeUninitSpanAssumeInit { inner: Ty },
    /// `[CELL-5]` — `c.borrow() -> Ref[T]`. A shared borrow of the cell plus
    /// a runtime check against the borrow counter; panics with the conflicting
    /// borrow's source location when a mutable borrow is active. Lowered in
    /// MIR to a counter check, a location store and a guard, so the check is
    /// visible to every analysis rather than hidden in the backend.
    RefCellBorrow,
    /// `[CELL-5]` — `c.borrow_mut() -> RefMut[T]`. As `borrow`, except it
    /// succeeds only when no borrow is active.
    RefCellBorrowMut,
    /// `[CELL-6]` — `c.try_borrow() -> Option[Ref[T]]`. The non-panicking
    /// form: `None` on contention in every profile (`[CELL-6a]`, `[PRF-1]`).
    RefCellTryBorrow,
    /// `[CELL-6]` — `c.try_borrow_mut() -> Option[RefMut[T]]`.
    RefCellTryBorrowMut,
    /// `[ARN-1]`, `[ARN-4]` — `Arena.with_capacity(bytes)`. The Arena itself
    /// is a compiler-known move-only struct; the runtime state it owns is
    /// opaque to Ember source.
    ArenaWithCapacity,
    /// `[ARN-1]`–`[ARN-4]` — `arena.alloc(owned value)` and
    /// `arena.alloc_nodrop(owned value)`. Type checking distinguishes the two
    /// spellings for `[ARN-3]`; both lower to an aligned bump allocation and
    /// a move of the value into the returned storage.
    ArenaAlloc { elem: Ty },
    /// `[ARN-9]` — allocate `count` slots without making an initialization
    /// claim and return them as `MutSpan[MaybeUninit[T]]`.
    ArenaAllocUninit { elem: Ty },
    /// `[ARN-3]`, `[ARN-11]` — allocate and zero-initialize `count` elements
    /// after type checking has proved the element's zero representation valid.
    ArenaAllocArrayZeroed { elem: Ty },
    /// `[ARN-3]`, `[ARN-10]`, `[ARN-12]` — allocate raw storage, then invoke
    /// the selected `Default.default()` implementation once per element.
    /// The constructor identity is resolved in type checking rather than
    /// rediscovered by lowering or the backend.
    ArenaAllocArrayDefault { elem: Ty, constructor: DefId },
    /// `[ARN-1]`, `[ARN-7]` — `arena.reset()`. Its receiver is `mut self`, so
    /// ordinary borrowing prevents a rewind while any allocation view lives.
    ArenaReset,
    /// `[ARN-4]` — bump allocation from an `Arena.fixed` buffer. The element
    /// type is explicit so lowering keeps both the receiver type and the
    /// copied value's layout without relying on a backend heuristic.
    FixedArenaAlloc { elem: Ty },
    /// `[ARN-7]` — rewind a fixed arena after its allocation views end.
    FixedArenaReset,
    /// `[ARN-6]` — create a LIFO scope while holding a mutable borrow of its
    /// parent. The result type is carried explicitly because the same builtin
    /// constructs a scope from both `Arena` and `ScopedArena` parents.
    ArenaScope { scoped: Ty },
    /// `[ARN-1]`, `[ARN-6]` — allocate through a scope. Its element type is
    /// explicit for the same reason as the other arena allocation builtins.
    ScopedArenaAlloc { elem: Ty },
    /// `[ARN-5]`–`[ARN-5g]` — reserve the one fixed backing allocation and
    /// construct an empty `ArenaArray[T]` view anchored to its Arena.
    ArenaArrayWithCapacity { elem: Ty, array: Ty },
    /// `[ARN-5c]` — checked shared/mutable access into fixed backing storage.
    ArenaArrayGet { elem: Ty, mutable: bool },
    /// `[ARN-5b]`, `[ARN-5c]` — append without growth; capacity exhaustion is
    /// represented by `Result`, never panic or another allocation.
    ArenaArrayPush { elem: Ty },
    /// `[ARN-5c]` — fixed-storage insertion, including the ordinary Array
    /// insertion-index check and in-place suffix shift.
    ArenaArrayInsert { elem: Ty },
    /// `[ARN-5c]` — optional move-out followed by an in-place suffix shift.
    ArenaArrayRemove { elem: Ty },
    /// `[ARN-5c]` — logically empty the initialized prefix. Elements satisfy
    /// `[ARN-5e]`, so this requires no destructor walk.
    ArenaArrayClear,
    /// `[ARN-5c]` — advance one named Array iterator and yield a shared or
    /// mutable element view in increasing index order.
    ArenaArrayIterNext { elem: Ty, mutable: bool },
    /// `[ARN-5a]`, `[ARN-5d]` — reserve one zeroed slot allocation and build
    /// an empty fixed-capacity ArenaMap view.
    ArenaMapWithCapacity { key: Ty, value: Ty, map: Ty },
    /// `[ARN-5d]`, `[HASH-4]` — deterministic lookup over the compact
    /// occupied prefix. A user-defined key carries the concrete `Eq.eq`
    /// implementation selected by type checking; compiler-known scalar keys
    /// use `None` and lower to a direct scalar comparison.
    ArenaMapGet { key: Ty, value: Ty, mutable: bool, equals: Option<DefId> },
    ArenaMapContains { key: Ty, equals: Option<DefId> },
    /// `[ARN-5b]`, `[ARN-5d]` — replace in place or append without growth.
    ArenaMapInsert { key: Ty, value: Ty, equals: Option<DefId> },
    /// `[ARN-5d]` — optional value move-out and compact suffix shift.
    ArenaMapRemove { key: Ty, value: Ty, equals: Option<DefId> },
    ArenaMapClear,
    /// `[ARN-5d]` — advance the named map iterator and yield key/value views.
    ArenaMapIterNext { key: Ty, value: Ty },
}

impl Builtin {
    pub fn from_name(name: &str) -> Option<Builtin> {
        match name {
            "println" => Some(Builtin::Println),
            "print" => Some(Builtin::Print),
            "eprintln" => Some(Builtin::EPrintln),
            "eprint" => Some(Builtin::EPrint),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Builtin::Println => "println",
            Builtin::Print => "print",
            Builtin::EPrintln => "eprintln",
            Builtin::EPrint => "eprint",
            Builtin::Panic => "panic",
            Builtin::Assert => "assert",
            Builtin::TotalLess => "cmp",
            Builtin::FloatAbs => "abs",
            Builtin::FloatPow => "pow",
            Builtin::FloatLib(f) => f.method(),
            Builtin::IntOverflowing(op) => op.overflowing(),
            Builtin::IntBits(bits) => bits.method(),
            Builtin::RangeCount | Builtin::RangeNth => "range",
            Builtin::StrCharCount => "char_count",
            Builtin::StrStartsWith => "starts_with",
            Builtin::StrEndsWith => "ends_with",
            Builtin::StrFind { reverse } => if reverse { "rfind" } else { "find" },
            Builtin::StrCount => "count",
            Builtin::StrReplace => "replace",
            Builtin::StrRepeat => "repeat",
            Builtin::StrTrimStart => "trim_start",
            Builtin::StrTrimEnd => "trim_end",
            Builtin::StrSliceOk => "slice_ok",
            Builtin::StrToUpper => "to_upper",
            Builtin::ParseStatus { .. } | Builtin::ParseValue { .. } => "parse",
            Builtin::StrToLower => "to_lower",
            Builtin::StrCharAt => "char_at",
            Builtin::CharUtf8Len => "len_utf8",
            Builtin::FormatWith(_) => "format",
            Builtin::StrContains | Builtin::StrContainsChar => "contains",
            Builtin::ArraySort => "sort",
            Builtin::ArrayReverse => "reverse",
            Builtin::ArrayClear => "clear",
            Builtin::ArrayPop { .. } => "pop",
            Builtin::ArrayRemove => "remove",
            Builtin::ArrayDrain => "drain",
            Builtin::ArrayInsert => "insert",
            Builtin::ArrayCapacity => "capacity",
            Builtin::ArrayReserve => "reserve",
            Builtin::ArrayTruncate => "truncate",
            Builtin::ArraySwapRemove => "swap_remove",
            Builtin::ArraySwap => "swap",
            Builtin::ArrayExtend => "extend",
            Builtin::ArraySorted { .. } => "sorted",
            Builtin::ArrayClone { .. } => "clone",
            Builtin::Input => "input",
            Builtin::ArrayNew => "Array",
            Builtin::ArrayFromLiteral => "Array",
            Builtin::ValueCompare { .. } => "compare",
            Builtin::ClassNew { .. } => "class",
            Builtin::ClassDowncast { forced: true, .. } => "as!",
            Builtin::ClassDowncast { forced: false, .. } => "as?",
            Builtin::ClassSuperInit { .. } => "super.init",
            Builtin::BoxNew { .. } => "Box",
            Builtin::SharedNew { .. } => "Shared",
            Builtin::WeakNew { .. } => "Weak",
            Builtin::WeakUpgrade { .. } => "upgrade",
            Builtin::WeakEmpty { .. } => "empty",
            Builtin::ArrayPush => "push",
            Builtin::ArrayLen => "len",
            Builtin::StringNew => "String",
            Builtin::StringPush => "push_str",
            Builtin::StringLen => "len",
            Builtin::StringAsStr => "as_str",
            Builtin::Format => "format",
            Builtin::MemAlloc => "alloc",
            Builtin::MemFree => "free",
            Builtin::PtrRead => "read",
            Builtin::PtrWrite => "write",
            Builtin::SizeOf => "size_of",
            Builtin::AlignOf => "align_of",
            Builtin::MemReplace { .. } => "replace",
            Builtin::MemTake { .. } => "take",
            Builtin::MemSwap { .. } => "swap",
            Builtin::MemForget { .. } => "forget",
            // `[RNG-10]`'s construction set. Named as they are
            // written, so a diagnostic quoting one reads as source.
            Builtin::SpanLen => "len",
            Builtin::SpanGet => "get",
            Builtin::SpanGetUnchecked => "get_unchecked",
            Builtin::SpanFrom { mutable } => {
                if mutable { "as_mut_span" } else { "as_span" }
            }
            Builtin::ArraySplitAtMut { .. } => "split_at_mut",
            Builtin::Slice { .. } => "slice",
            Builtin::StrIsCharBoundary => "is_char_boundary",
            Builtin::StrAsBytes => "as_bytes",
            Builtin::SpanSplitAt { .. } => "split_at",
            Builtin::SpanReborrow => "reborrow",
            Builtin::SpanSharedReborrow => "reborrow",
            Builtin::SpanIterNext { .. } => "next",
            Builtin::SpanChunksNew { mutable, .. } => {
                if mutable { "chunks_mut" } else { "chunks" }
            }
            Builtin::SpanChunksNext { .. } => "next",
            Builtin::SpanWindowsNew { .. } => "windows",
            Builtin::SpanWindowsNext { .. } => "next",
            Builtin::SpanAsPtr { mutable } => {
                if mutable { "as_mut_ptr" } else { "as_ptr" }
            }
            Builtin::RangeChecked(_) => "checked",
            Builtin::RangeClamped(_) => "clamped",
            Builtin::RangeNewUnchecked(_) => "new_unchecked",
            Builtin::CellSet => "set",
            Builtin::CellReplace => "replace",
            Builtin::CellIntoInner => "into_inner",
            Builtin::CellUpdate => "update",
            Builtin::CellUpdateDefault { .. } => "update",
            Builtin::CellTake { .. } => "take",
            Builtin::MaybeUninitUninit { .. } => "uninit",
            Builtin::MaybeUninitWrite { .. } => "write",
            Builtin::MaybeUninitAssumeInit { .. } => "assume_init",
            Builtin::MaybeUninitWriteAt { .. } => "write_at",
            Builtin::MaybeUninitSpanAssumeInit { .. } => "assume_init",
            Builtin::UnsafeCellGet { .. } => "get",
            Builtin::UnsafeCellIntoInner => "into_inner",
            Builtin::RefCellBorrow => "borrow",
            Builtin::RefCellBorrowMut => "borrow_mut",
            Builtin::RefCellTryBorrow => "try_borrow",
            Builtin::RefCellTryBorrowMut => "try_borrow_mut",
            Builtin::ArenaWithCapacity => "with_capacity",
            Builtin::ArenaAlloc { .. }
            | Builtin::FixedArenaAlloc { .. }
            | Builtin::ScopedArenaAlloc { .. } => "alloc",
            Builtin::ArenaAllocUninit { .. } => "alloc_uninit",
            Builtin::ArenaAllocArrayZeroed { .. }
            | Builtin::ArenaAllocArrayDefault { .. } => "alloc_array",
            Builtin::ArenaReset => "reset",
            Builtin::FixedArenaReset => "reset",
            Builtin::ArenaScope { .. } => "scope",
            Builtin::ArenaArrayWithCapacity { .. } => "with_capacity",
            Builtin::ArenaArrayGet { mutable, .. } => {
                if mutable { "get_mut" } else { "get" }
            }
            Builtin::ArenaArrayPush { .. } => "push",
            Builtin::ArenaArrayInsert { .. } => "insert",
            Builtin::ArenaArrayRemove { .. } => "remove",
            Builtin::ArenaArrayClear => "clear",
            Builtin::ArenaArrayIterNext { .. } => "next",
            Builtin::ArenaMapWithCapacity { .. } => "with_capacity",
            Builtin::ArenaMapGet { mutable, .. } => {
                if mutable { "get_mut" } else { "get" }
            }
            Builtin::ArenaMapContains { .. } => "contains_key",
            Builtin::ArenaMapInsert { .. } => "insert",
            Builtin::ArenaMapRemove { .. } => "remove",
            Builtin::ArenaMapClear => "clear",
            Builtin::ArenaMapIterNext { .. } => "next",
        }
    }
}

/// Binary operators on scalars. Operators on non-scalar types desugar to
/// interface method calls (`[TYP-21]`) and never reach here.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    /// True division on floats; C's truncating division on integers, which
    /// only compiler-lowered forms (`div_trunc`) produce (`[TYP-28]`).
    Div,
    /// C's truncating remainder (`rem_trunc`).
    Rem,
    /// `//` — floor division: the quotient rounded toward negative infinity
    /// (`[TYP-28]`), `floor(a / b)` on floats (`[TYP-29]`).
    FloorDiv,
    /// `%` — floor modulo, with the sign of the divisor (`[TYP-28]`,
    /// `[TYP-29]`).
    FloorRem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    /// `[CLS-4]`/Part VIII — class-handle identity, not value equality.
    Is,
    IsNot,
    /// `and` / `or`, which short-circuit (`[EXP-3]`).
    And,
    Or,
}

impl BinOp {
    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            BinOp::Eq
                | BinOp::Ne
                | BinOp::Lt
                | BinOp::Le
                | BinOp::Gt
                | BinOp::Ge
                | BinOp::Is
                | BinOp::IsNot
        )
    }

    pub fn is_short_circuit(self) -> bool {
        matches!(self, BinOp::And | BinOp::Or)
    }

    /// The Ember spelling, which a panic message names (`[TYP-8]`: "naming
    /// the operator actually written").
    pub fn spelling(self) -> &'static str {
        match self {
            BinOp::FloorDiv => "//",
            other => other.c_operator(),
        }
    }

    /// The C spelling. Ember's semantics match C's for every operator here
    /// except division and remainder by zero, which panic (`[TYP-8]`).
    pub fn c_operator(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            // Only unsigned operands reach C through this spelling: floor and
            // truncation agree there. Signed operands use the checked runtime
            // helpers and floats the `floor` helpers.
            BinOp::FloorDiv => "/",
            BinOp::FloorRem => "%",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Is => "==",
            BinOp::IsNot => "!=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
}

/// `--emit=hir` (`[HIR-2]`): a stable textual form for snapshot tests.
pub fn dump(program: &Program, types: &ember_types::TypeTable) -> String {
    let mut out = String::new();
    for function in &program.functions {
        let params: Vec<String> = function
            .params
            .iter()
            .map(|p| {
                let decl = function.local(p.local);
                let mode = match p.mode {
                    Mode::Borrow => "",
                    Mode::Mut => "mut ",
                    Mode::Owned => "owned ",
                };
                let name = decl.name.map(|n| n.to_string()).unwrap_or_else(|| "_".into());
                format!("{mode}{name}: {}", types.symbol_name(decl.ty))
            })
            .collect();
        out.push_str(&format!(
            "fn {} ({}) -> {}\n",
            function.symbol,
            params.join(", "),
            types.symbol_name(function.ret)
        ));
        dump_block(&function.body, function, types, 1, &mut out);
    }
    out
}

fn dump_block(
    block: &Block,
    function: &Function,
    types: &ember_types::TypeTable,
    depth: usize,
    out: &mut String,
) {
    for stmt in &block.stmts {
        let pad = "  ".repeat(depth);
        match stmt {
            Stmt::Let { local, init } => {
                let decl = function.local(*local);
                let name = decl.name.map(|n| n.to_string()).unwrap_or_else(|| format!("_{}", local.0));
                out.push_str(&format!("{pad}let {name}: {}", types.symbol_name(decl.ty)));
                match init {
                    Some(e) => out.push_str(&format!(" = {}\n", dump_expr(e, function, types))),
                    None => out.push('\n'),
                }
            }
            Stmt::Assign { place, value } => out.push_str(&format!(
                "{pad}{} = {}\n",
                dump_expr(place, function, types),
                dump_expr(value, function, types)
            )),
            Stmt::Destructure { temp, value, bindings } => {
                let name = function
                    .local(*temp)
                    .name
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| format!("_{}", temp.0));
                out.push_str(&format!(
                    "{pad}destructure {name}: {} = {}\n",
                    types.symbol_name(function.local(*temp).ty),
                    dump_expr(value, function, types)
                ));
                for binding in bindings {
                    match binding {
                        DestructureBinding::Let { local, value } => {
                            let decl = function.local(*local);
                            let target = decl
                                .name
                                .map(|n| n.to_string())
                                .unwrap_or_else(|| format!("_{}", local.0));
                            out.push_str(&format!("{pad}  let {target}: {}", types.symbol_name(decl.ty)));
                            out.push_str(&format!(" = {}", dump_expr(value, function, types)));
                            out.push('\n');
                        }
                        DestructureBinding::Assign { place, value } => out.push_str(&format!(
                            "{pad}  {} = {}\n",
                            dump_expr(place, function, types),
                            dump_expr(value, function, types)
                        )),
                    }
                }
            }
            Stmt::Expr(e) => out.push_str(&format!("{pad}{}\n", dump_expr(e, function, types))),
            Stmt::Return(e) => match e {
                Some(e) => out.push_str(&format!("{pad}return {}\n", dump_expr(e, function, types))),
                None => out.push_str(&format!("{pad}return\n")),
            },
            Stmt::If { cond, then_block, else_block } => {
                out.push_str(&format!("{pad}if {}:\n", dump_expr(cond, function, types)));
                dump_block(then_block, function, types, depth + 1, out);
                if let Some(b) = else_block {
                    out.push_str(&format!("{pad}else:\n"));
                    dump_block(b, function, types, depth + 1, out);
                }
            }
            Stmt::While { cond, body, else_block } => {
                out.push_str(&format!("{pad}while {}:\n", dump_expr(cond, function, types)));
                dump_block(body, function, types, depth + 1, out);
                if let Some(b) = else_block {
                    out.push_str(&format!("{pad}else:\n"));
                    dump_block(b, function, types, depth + 1, out);
                }
            }
            Stmt::ForRange { local, start, end, inclusive, body, else_block } => {
                let name = function
                    .local(*local)
                    .name
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| format!("_{}", local.0));
                out.push_str(&format!(
                    "{pad}for {name} in {}..{}{}:\n",
                    dump_expr(start, function, types),
                    if *inclusive { "=" } else { "" },
                    dump_expr(end, function, types)
                ));
                dump_block(body, function, types, depth + 1, out);
                if let Some(b) = else_block {
                    out.push_str(&format!("{pad}else:\n"));
                    dump_block(b, function, types, depth + 1, out);
                }
            }
            Stmt::Block(b) => {
                out.push_str(&format!("{pad}block:\n"));
                dump_block(b, function, types, depth + 1, out);
            }
            Stmt::Defer(b) => {
                out.push_str(&format!("{pad}defer:\n"));
                dump_block(b, function, types, depth + 1, out);
            }
            Stmt::Break { depth: out_of } => {
                out.push_str(&format!("{pad}break {out_of}\n"));
            }
            Stmt::Continue { depth: out_of } => {
                out.push_str(&format!("{pad}continue {out_of}\n"));
            }
        }
    }
}

fn dump_pattern(
    pattern: &Pattern,
    function: &Function,
    types: &ember_types::TypeTable,
) -> String {
    match &pattern.kind {
        PatternKind::Wild => "_".to_string(),
        PatternKind::Bind { local, sub, by_ref } => {
            let name = function
                .local(*local)
                .name
                .map(|n| n.to_string())
                .unwrap_or_else(|| format!("_{}", local.0));
            let name = match by_ref {
                Some(true) => format!("ref mut {name}"),
                Some(false) => format!("ref {name}"),
                None => name,
            };
            match sub {
                Some(sub) => format!("{name} @ {}", dump_pattern(sub, function, types)),
                None => name,
            }
        }
        PatternKind::Int(value) => value.to_string(),
        PatternKind::Variant { enum_id, variant, fields } => {
            let def = types.enum_def(*enum_id);
            let name = format!("{}.{}", def.name, def.variants[*variant].name);
            if fields.is_empty() {
                name
            } else {
                let inner: Vec<String> =
                    fields.iter().map(|f| dump_pattern(f, function, types)).collect();
                format!("{name}({})", inner.join(", "))
            }
        }
        PatternKind::Fields(items) => {
            let inner: Vec<String> =
                items.iter().map(|f| dump_pattern(f, function, types)).collect();
            format!("({})", inner.join(", "))
        }
        PatternKind::Or(items) => {
            let inner: Vec<String> =
                items.iter().map(|f| dump_pattern(f, function, types)).collect();
            inner.join(" | ")
        }
        PatternKind::Error => "<error>".to_string(),
    }
}

fn dump_expr(expr: &Expr, function: &Function, types: &ember_types::TypeTable) -> String {
    let body = match &expr.kind {
        ExprKind::Int(v) => v.to_string(),
        ExprKind::Float(v) => format!("{v:?}"),
        ExprKind::Bool(v) => v.to_string(),
        ExprKind::Str(s) => format!("{s:?}"),
        ExprKind::Local(id) => function
            .local(*id)
            .name
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("_{}", id.0)),
        ExprKind::Field { base, index } => {
            format!("{}.{index}", dump_expr(base, function, types))
        }
        ExprKind::EnumField { base, variant, index } => {
            format!("{}.<variant {variant}>.{index}", dump_expr(base, function, types))
        }
        ExprKind::Call { callee, args, latebound, .. } => {
            let inner: Vec<String> = args.iter().map(|a| dump_expr(a, function, types)).collect();
            let boundary = if *latebound { "@latebound " } else { "" };
            format!("{boundary}call#{}({})", callee.0, inner.join(", "))
        }
        ExprKind::InterfaceCall { interface, slot, receiver, args, .. } => {
            let mut inner = vec![dump_expr(receiver, function, types)];
            inner.extend(args.iter().map(|a| dump_expr(a, function, types)));
            format!("@dyn {interface}::slot{slot}({})", inner.join(", "))
        }
        ExprKind::FnValue(def) => format!("fn#{}", def.0),
        ExprKind::CallIndirect { callee, args, consumes_callee, latebound } => {
            let inner: Vec<String> = args.iter().map(|a| dump_expr(a, function, types)).collect();
            let mode = if *consumes_callee { "owned " } else { "" };
            let boundary = if *latebound { "@latebound " } else { "" };
            format!("{boundary}{mode}({})({})", dump_expr(callee, function, types), inner.join(", "))
        }
        ExprKind::StructLit { struct_id, fields } => {
            let inner: Vec<String> = fields.iter().map(|f| dump_expr(f, function, types)).collect();
            format!("{}({})", types.struct_def(*struct_id).name, inner.join(", "))
        }
        ExprKind::Binary { op, lhs, rhs } => format!(
            "({} {} {})",
            dump_expr(lhs, function, types),
            op.c_operator(),
            dump_expr(rhs, function, types)
        ),
        ExprKind::Unary { op, operand } => {
            format!("({op:?} {})", dump_expr(operand, function, types))
        }
        ExprKind::Cast { expr: inner, to } => {
            format!("({} as {})", dump_expr(inner, function, types), types.symbol_name(*to))
        }
        ExprKind::InterfaceUpcast { interfaces, expr: inner, .. } => {
            let bounds = interfaces.iter().map(ToString::to_string).collect::<Vec<_>>().join(" + ");
            format!("@dyn {bounds}({})", dump_expr(inner, function, types))
        }
        ExprKind::DynBoxNew { interfaces, value, .. } => {
            let bounds = interfaces.iter().map(ToString::to_string).collect::<Vec<_>>().join(" + ");
            format!("Box[dyn {bounds}]({})", dump_expr(value, function, types))
        }
        ExprKind::EraseRange(inner) => {
            format!("(erase {})", dump_expr(inner, function, types))
        }
        ExprKind::Widen { expr: inner, to } => {
            format!("widen({} -> {})", dump_expr(inner, function, types), types.symbol_name(*to))
        }
        ExprKind::Builtin { which, args } => {
            let inner: Vec<String> = args.iter().map(|a| dump_expr(a, function, types)).collect();
            format!("{}({})", which.name(), inner.join(", "))
        }
        ExprKind::ClassNew { class_id, args, .. } => {
            let inner: Vec<String> = args.iter().map(|a| dump_expr(a, function, types)).collect();
            format!("class#{}({})", class_id.0, inner.join(", "))
        }
        ExprKind::Block { block, value } => {
            format!("{{ {} statements; {} }}", block.stmts.len(), dump_expr(value, function, types))
        }
        ExprKind::TupleLit(items) => {
            let inner: Vec<String> = items.iter().map(|e| dump_expr(e, function, types)).collect();
            format!("({})", inner.join(", "))
        }
        ExprKind::ArrayLit(items) => {
            let inner: Vec<String> = items.iter().map(|e| dump_expr(e, function, types)).collect();
            format!("[{}]", inner.join(", "))
        }
        ExprKind::ArrayRepeat { value, count } => {
            format!("[{}; {count}]", dump_expr(value, function, types))
        }
        ExprKind::Index { base, index } => format!(
            "{}[{}]",
            dump_expr(base, function, types),
            dump_expr(index, function, types)
        ),
        ExprKind::EnumLit { enum_id, variant, fields } => {
            let def = types.enum_def(*enum_id);
            let name = format!("{}.{}", def.name, def.variants[*variant].name);
            if fields.is_empty() {
                name
            } else {
                let inner: Vec<String> =
                    fields.iter().map(|f| dump_expr(f, function, types)).collect();
                format!("{name}({})", inner.join(", "))
            }
        }
        ExprKind::Match { scrutinee, arms } => {
            let inner: Vec<String> = arms
                .iter()
                .map(|a| dump_pattern(&a.pattern, function, types))
                .collect();
            format!("match {} [{}]", dump_expr(scrutinee, function, types), inner.join(" | "))
        }
        ExprKind::Ref { place, mutable } => {
            let kind = if *mutable { "ref mut " } else { "ref " };
            format!("{kind}{}", dump_expr(place, function, types))
        }
        ExprKind::Deref(inner) => format!("(*{})", dump_expr(inner, function, types)),
        ExprKind::FString { parts, .. } => {
            let inner: Vec<String> = parts
                .iter()
                .map(|p| match p {
                    FStringPart::Text(text) => format!("{text:?}"),
                    FStringPart::Value(e, _) => format!("{{{}}}", dump_expr(e, function, types)),
                })
                .collect();
            format!("f({})", inner.join(" "))
        }
        ExprKind::Error => "<error>".to_string(),
    };
    format!("{body}:{}", types.symbol_name(expr.ty))
}
