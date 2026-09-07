//! MIR: a control-flow graph over places and operands (Part XVIII §4).
//!
//! MIR is where the borrow checker, drop elaboration and the effect analysis
//! will run (Phases 2–4), so it exists from Phase 0 even though Phase 0 only
//! lowers straight-line code. Codegen reads MIR, never HIR — writing the C
//! backend against HIR would mean writing it twice.

use ember_span::Span;
use ember_types::{StructId, Ty};

pub mod lower;
pub mod verify;

pub use lower::lower;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BasicBlockId(pub u32);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LocalId(pub u32);

/// The slot a function's return value is written into. Rust MIR's convention,
/// kept because it makes `Return` need no operand.
pub const RETURN_LOCAL: LocalId = LocalId(0);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LocalKind {
    /// The return slot, local 0.
    Return,
    /// A parameter. Parameters occupy locals `1..=arg_count`.
    Arg,
    /// A user-written local.
    User,
    /// A compiler-introduced temporary.
    Temp,
}

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub ty: Ty,
    pub kind: LocalKind,
    /// The name as written, for readable C and for diagnostics.
    pub name: Option<String>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Body {
    pub name: String,
    /// The mangled C symbol (`[MNG-1]`).
    pub symbol: String,
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<BasicBlock>,
    pub arg_count: usize,
    pub span: Span,
}

impl Body {
    pub fn local(&self, id: LocalId) -> &LocalDecl {
        &self.locals[id.0 as usize]
    }

    pub fn block(&self, id: BasicBlockId) -> &BasicBlock {
        &self.blocks[id.0 as usize]
    }

    pub fn return_ty(&self) -> Ty {
        self.locals[0].ty
    }

    /// Parameters, in declaration order.
    pub fn args(&self) -> impl Iterator<Item = (LocalId, &LocalDecl)> {
        (1..=self.arg_count).map(move |i| (LocalId(i as u32), &self.locals[i]))
    }
}

#[derive(Debug)]
pub struct BasicBlock {
    pub stmts: Vec<Stmt>,
    pub terminator: Terminator,
}

/// A memory location: a local with a chain of projections.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Place {
    pub local: LocalId,
    pub projection: Vec<Projection>,
}

impl Place {
    pub fn local(local: LocalId) -> Place {
        Place { local, projection: Vec::new() }
    }

    pub fn field(mut self, index: usize) -> Place {
        self.projection.push(Projection::Field(index));
        self
    }

    pub fn is_local(&self) -> bool {
        self.projection.is_empty()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Projection {
    Field(usize),
    /// Indexing by a local's value. Phase 2 adds the bounds `Assert`.
    Index(LocalId),
    ConstIndex(u64),
    Deref,
    /// An `SoA` column (`[SOA-2]`). Reserved; Phase 6 emits it.
    Column(usize),
}

#[derive(Clone, Debug)]
pub enum Operand {
    /// Read a place without disturbing it. Only valid for `Copy` types.
    Copy(Place),
    /// Read a place and leave it uninitialised.
    Move(Place),
    Const(Const),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Const {
    Int { value: u128, ty: Ty },
    Float { value: f64, ty: Ty },
    Bool(bool),
    /// A string literal with static region (`[LEX-20]`).
    Str(String),
    /// The unit value.
    Void,
}

#[derive(Clone, Debug)]
pub enum Rvalue {
    Use(Operand),
    BinaryOp { op: BinOp, lhs: Operand, rhs: Operand },
    UnaryOp { op: UnOp, operand: Operand },
    Cast { kind: CastKind, operand: Operand, to: Ty },
    /// Building a struct or tuple from its fields, in declaration order.
    Aggregate { kind: AggregateKind, operands: Vec<Operand> },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum AggregateKind {
    Struct(StructId),
    Tuple,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CastKind {
    /// `[TYP-6]` — truncation, float-to-int saturation, int-to-float rounding.
    Numeric,
    /// A lossless widening inserted implicitly (`[TYP-5]`).
    Widen,
}

pub use ember_hir::{BinOp, Builtin, UnOp};

#[derive(Debug)]
pub enum Stmt {
    Assign { place: Place, rvalue: Rvalue },
    /// A local comes into scope. Drives `[DRP-2]`'s reverse-order drops and,
    /// from Phase 2, the borrow checker's loan-kill analysis.
    StorageLive(LocalId),
    StorageDead(LocalId),
    Nop,
}

#[derive(Debug)]
pub enum Terminator {
    Goto(BasicBlockId),
    /// A conditional branch on an integer or boolean discriminant.
    SwitchInt { discr: Operand, targets: Vec<(u128, BasicBlockId)>, otherwise: BasicBlockId },
    Return,
    Unreachable,
    Call { func: FuncRef, args: Vec<Operand>, dest: Place, next: BasicBlockId },
}

#[derive(Clone, Debug)]
pub enum FuncRef {
    /// A direct call to a body in this compilation unit.
    Direct { symbol: String },
    /// A call the compiler provides itself, lowered to an `ember_rt` entry.
    Builtin { which: ember_hir::Builtin, arg_ty: Ty },
}

/// `--emit=mir`: a stable textual form, for snapshot tests.
pub fn dump(bodies: &[Body], types: &ember_types::TypeTable) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for body in bodies {
        let _ = writeln!(out, "fn {}:", body.symbol);
        for (i, local) in body.locals.iter().enumerate() {
            let name = local.name.as_deref().unwrap_or("");
            let _ = writeln!(
                out,
                "  let _{i}: {}   // {:?} {name}",
                types.display(local.ty),
                local.kind
            );
        }
        for (i, block) in body.blocks.iter().enumerate() {
            let _ = writeln!(out, "  bb{i}:");
            for stmt in &block.stmts {
                let _ = writeln!(out, "    {}", dump_stmt(stmt, types));
            }
            let _ = writeln!(out, "    {}", dump_terminator(&block.terminator, types));
        }
    }
    out
}

fn dump_stmt(stmt: &Stmt, types: &ember_types::TypeTable) -> String {
    match stmt {
        Stmt::Assign { place, rvalue } => {
            format!("{} = {}", dump_place(place), dump_rvalue(rvalue, types))
        }
        Stmt::StorageLive(l) => format!("StorageLive(_{})", l.0),
        Stmt::StorageDead(l) => format!("StorageDead(_{})", l.0),
        Stmt::Nop => "nop".to_string(),
    }
}

fn dump_place(place: &Place) -> String {
    let mut out = format!("_{}", place.local.0);
    for projection in &place.projection {
        match projection {
            Projection::Field(i) => out.push_str(&format!(".{i}")),
            Projection::Index(l) => out.push_str(&format!("[_{}]", l.0)),
            Projection::ConstIndex(i) => out.push_str(&format!("[{i}]")),
            Projection::Deref => out = format!("(*{out})"),
            Projection::Column(i) => out.push_str(&format!(".col{i}")),
        }
    }
    out
}

fn dump_operand(operand: &Operand, types: &ember_types::TypeTable) -> String {
    match operand {
        Operand::Copy(p) => format!("copy {}", dump_place(p)),
        Operand::Move(p) => format!("move {}", dump_place(p)),
        Operand::Const(c) => match c {
            Const::Int { value, ty } => format!("const {value}_{}", types.display(*ty)),
            Const::Float { value, ty } => format!("const {value:?}_{}", types.display(*ty)),
            Const::Bool(b) => format!("const {b}"),
            Const::Str(s) => format!("const {s:?}"),
            Const::Void => "const ()".to_string(),
        },
    }
}

fn dump_rvalue(rvalue: &Rvalue, types: &ember_types::TypeTable) -> String {
    match rvalue {
        Rvalue::Use(o) => dump_operand(o, types),
        Rvalue::BinaryOp { op, lhs, rhs } => format!(
            "{} {} {}",
            dump_operand(lhs, types),
            op.c_operator(),
            dump_operand(rhs, types)
        ),
        Rvalue::UnaryOp { op, operand } => format!("{op:?} {}", dump_operand(operand, types)),
        Rvalue::Cast { kind, operand, to } => {
            format!("{:?}({}) as {}", kind, dump_operand(operand, types), types.display(*to))
        }
        Rvalue::Aggregate { kind, operands } => {
            let inner: Vec<String> = operands.iter().map(|o| dump_operand(o, types)).collect();
            let name = match kind {
                AggregateKind::Struct(id) => types.struct_def(*id).name.to_string(),
                AggregateKind::Tuple => "tuple".to_string(),
            };
            format!("{name}({})", inner.join(", "))
        }
    }
}

fn dump_terminator(terminator: &Terminator, types: &ember_types::TypeTable) -> String {
    match terminator {
        Terminator::Goto(bb) => format!("goto bb{}", bb.0),
        Terminator::SwitchInt { discr, targets, otherwise } => {
            let arms: Vec<String> =
                targets.iter().map(|(v, bb)| format!("{v} -> bb{}", bb.0)).collect();
            format!(
                "switchInt({}) [{}, otherwise -> bb{}]",
                dump_operand(discr, types),
                arms.join(", "),
                otherwise.0
            )
        }
        Terminator::Return => "return".to_string(),
        Terminator::Unreachable => "unreachable".to_string(),
        Terminator::Call { func, args, dest, next } => {
            let inner: Vec<String> = args.iter().map(|a| dump_operand(a, types)).collect();
            let name = match func {
                FuncRef::Direct { symbol } => symbol.clone(),
                FuncRef::Builtin { which, .. } => which.name().to_string(),
            };
            format!("{} = {name}({}) -> bb{}", dump_place(dest), inner.join(", "), next.0)
        }
    }
}
