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
use ember_types::{StructId, Ty};

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
    pub fn function(&self, def: DefId) -> &Function {
        &self.functions[def.0 as usize]
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
    While { cond: Expr, body: Block },
    Block(Block),
    Break,
    Continue,
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
    /// `Vec3(1, 2, 3)` — the memberwise constructor (`[STR-1]`). Arguments are
    /// in declaration order with defaults already filled in.
    StructLit { struct_id: StructId, fields: Vec<Expr> },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Unary { op: UnOp, operand: Box<Expr> },
    /// `x as T` — an explicit numeric conversion (`[TYP-6]`).
    Cast { expr: Box<Expr>, to: Ty },
    /// A lossless widening inserted at a coercion site (`[TYP-5]`). Kept
    /// distinct from `Cast` because it is implicit and always lossless.
    Widen { expr: Box<Expr>, to: Ty },
    /// A call the compiler knows about directly, before `std` exists.
    Builtin { which: Builtin, args: Vec<Expr> },
    /// A subexpression that failed to check. Absorbs errors.
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
            Stmt::While { cond, body } => {
                out.push_str(&format!("{pad}while {}:\n", dump_expr(cond, function, types)));
                dump_block(body, function, types, depth + 1, out);
            }
            Stmt::Block(b) => {
                out.push_str(&format!("{pad}block:\n"));
                dump_block(b, function, types, depth + 1, out);
            }
            Stmt::Break => out.push_str(&format!("{pad}break\n")),
            Stmt::Continue => out.push_str(&format!("{pad}continue\n")),
        }
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
        ExprKind::Widen { expr: inner, to } => {
            format!("widen({} -> {})", dump_expr(inner, function, types), types.display(*to))
        }
        ExprKind::Builtin { which, args } => {
            let inner: Vec<String> = args.iter().map(|a| dump_expr(a, function, types)).collect();
            format!("{}({})", which.name(), inner.join(", "))
        }
        ExprKind::Error => "<error>".to_string(),
    };
    format!("{body}:{}", types.display(expr.ty))
}
