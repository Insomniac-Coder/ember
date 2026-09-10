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
use ember_types::{EnumId, OverflowPolicy, RangeId, StructId, Ty};

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
    /// The mangled symbol this function gets in the emitted C (`[MNG-1]`), or
    /// the name given by `@export` (`[MNG-2]`).
    pub symbol: String,
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
pub struct Expr {
    pub ty: Ty,
    pub kind: ExprKind,
    pub span: Span,
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
    /// A direct call to a known function.
    Call { callee: DefId, args: Vec<Expr> },
    /// `[FN-6]` — a named function used as a value. Its type is the
    /// `fn(A) -> R` it coerces to.
    FnValue(DefId),
    /// A call through a value of function type, rather than to a name.
    CallIndirect { callee: Box<Expr>, args: Vec<Expr> },
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
    /// A subexpression that failed to check. Absorbs errors.
    Error,
}

/// One piece of an f-string: literal text, or a value to format into it.
#[derive(Debug)]
pub enum FStringPart {
    Text(String),
    Value(Expr),
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
    Bind { local: LocalId, sub: Option<Box<Pattern>> },
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

/// Functions the compiler provides itself in Phase 0, before the standard
/// library is written in Ember. Each lowers to one `ember_rt` call.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Builtin {
    /// `println(x)` for a scalar or `str`.
    Println,
    /// `print(x)` — the same without the newline.
    Print,
    /// `Array[T]()` — an empty growable array. Part XX.1 makes `Array` a
    /// compiler-known type until Phase 2's generics.
    ArrayNew,
    /// `a.push(x)`. The receiver is a `ref mut`, so it grows in place.
    ArrayPush,
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
}

impl Builtin {
    pub fn from_name(name: &str) -> Option<Builtin> {
        match name {
            "println" => Some(Builtin::Println),
            "print" => Some(Builtin::Print),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Builtin::Println => "println",
            Builtin::Print => "print",
            Builtin::ArrayNew => "Array",
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
            // `[RNG-10]`'s construction set. Named as they are
            // written, so a diagnostic quoting one reads as source.
            Builtin::SpanLen => "len",
            Builtin::SpanGet => "get",
            Builtin::SpanGetUnchecked => "get_unchecked",
            Builtin::SpanFrom { mutable } => {
                if mutable { "as_mut_span" } else { "as_span" }
            }
            Builtin::RangeChecked(_) => "checked",
            Builtin::RangeClamped(_) => "clamped",
            Builtin::RangeNewUnchecked(_) => "new_unchecked",
            Builtin::CellSet => "set",
            Builtin::CellReplace => "replace",
            Builtin::CellIntoInner => "into_inner",
            Builtin::CellUpdate => "update",
            Builtin::RefCellBorrow => "borrow",
            Builtin::RefCellBorrowMut => "borrow_mut",
            Builtin::RefCellTryBorrow => "try_borrow",
            Builtin::RefCellTryBorrowMut => "try_borrow_mut",
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
    Div,
    Rem,
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
    /// `and` / `or`, which short-circuit (`[EXP-3]`).
    And,
    Or,
}

impl BinOp {
    pub fn is_comparison(self) -> bool {
        matches!(self, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge)
    }

    pub fn is_short_circuit(self) -> bool {
        matches!(self, BinOp::And | BinOp::Or)
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
                format!("{mode}{name}: {}", types.display(decl.ty))
            })
            .collect();
        out.push_str(&format!(
            "fn {} ({}) -> {}\n",
            function.symbol,
            params.join(", "),
            types.display(function.ret)
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
                out.push_str(&format!("{pad}let {name}: {}", types.display(decl.ty)));
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
        PatternKind::Bind { local, sub } => {
            let name = function
                .local(*local)
                .name
                .map(|n| n.to_string())
                .unwrap_or_else(|| format!("_{}", local.0));
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
        ExprKind::Call { callee, args } => {
            let inner: Vec<String> = args.iter().map(|a| dump_expr(a, function, types)).collect();
            format!("call#{}({})", callee.0, inner.join(", "))
        }
        ExprKind::FnValue(def) => format!("fn#{}", def.0),
        ExprKind::CallIndirect { callee, args } => {
            let inner: Vec<String> = args.iter().map(|a| dump_expr(a, function, types)).collect();
            format!("({})({})", dump_expr(callee, function, types), inner.join(", "))
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
            format!("({} as {})", dump_expr(inner, function, types), types.display(*to))
        }
        ExprKind::EraseRange(inner) => {
            format!("(erase {})", dump_expr(inner, function, types))
        }
        ExprKind::Widen { expr: inner, to } => {
            format!("widen({} -> {})", dump_expr(inner, function, types), types.display(*to))
        }
        ExprKind::Builtin { which, args } => {
            let inner: Vec<String> = args.iter().map(|a| dump_expr(a, function, types)).collect();
            format!("{}({})", which.name(), inner.join(", "))
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
                    FStringPart::Value(e) => format!("{{{}}}", dump_expr(e, function, types)),
                })
                .collect();
            format!("f({})", inner.join(" "))
        }
        ExprKind::Error => "<error>".to_string(),
    };
    format!("{body}:{}", types.display(expr.ty))
}
