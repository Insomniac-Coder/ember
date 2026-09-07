//! The abstract syntax tree (Part XVIII §2).
//!
//! The AST is a faithful, span-carrying tree: it records what was written,
//! not what it means. Desugaring happens on the way to HIR, so the formatter
//! and the linter can both work from this tree without losing source shape.
//!
//! `[AST-1]` — every node carries a `span` and a dense per-file `NodeId` used
//! by the side tables that later stages build.

use ember_span::{Span, Symbol};

pub mod print;
pub use print::dump;

/// Dense per-file node identity. Side tables (types, resolutions) index by it.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NodeId(pub u32);

impl NodeId {
    pub const DUMMY: NodeId = NodeId(u32::MAX);
}

/// An identifier as written, with the span it occupied.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Ident {
    pub name: Symbol,
    pub span: Span,
}

/// One parsed source file.
#[derive(Debug)]
pub struct Module {
    pub directive: Option<Directive>,
    pub imports: Vec<Import>,
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Directive {
    pub name: Ident,
    pub value: String,
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Attributes and visibility
// ---------------------------------------------------------------------------

/// `@name(args)` (Part III §7). `[ATT-1]` — an unknown attribute in an
/// unregistered namespace is an error, checked after parsing.
#[derive(Debug)]
pub struct Attribute {
    pub id: NodeId,
    /// `derive`, or a namespaced `ragev.field`.
    pub path: Vec<Ident>,
    pub args: Vec<AttrArg>,
    pub span: Span,
}

#[derive(Debug)]
pub enum AttrArg {
    Expr(Expr),
    Named { name: Ident, value: Expr },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Visibility {
    pub kind: VisKind,
    pub span: Span,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum VisKind {
    /// Private to the declaring module (`[MOD-2]`).
    #[default]
    Private,
    /// `pub(package)`.
    Package,
    /// `pub`.
    Public,
}

impl Default for Visibility {
    fn default() -> Visibility {
        Visibility::private()
    }
}

impl Visibility {
    /// `[MOD-7]` — `pub(read)` and `pub(package, read)` make a field readable
    /// at the stated level but writable only from the declaring module.
    pub fn private() -> Visibility {
        Visibility { kind: VisKind::Private, span: Span::DUMMY }
    }
}

// ---------------------------------------------------------------------------
// Items
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct Item {
    pub id: NodeId,
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub doc: Option<String>,
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum ItemKind {
    Fn(FnDecl),
    Struct(StructDecl),
    Class(ClassDecl),
    Enum(EnumDecl),
    Interface(InterfaceDecl),
    Extend(ExtendDecl),
    Const(ConstDecl),
    Static(StaticDecl),
    TypeAlias(TypeAlias),
    ExternBlock(ExternBlock),
    Comptime(Block),
}

#[derive(Debug)]
pub struct Import {
    pub id: NodeId,
    pub kind: ImportKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum ImportKind {
    /// `import a.b.c [as d]`
    Module { path: Vec<Ident>, alias: Option<Ident> },
    /// `from a.b import x, y as z` / `from a.b import *`
    Items { path: Vec<Ident>, items: Vec<ImportItem>, glob: bool },
    /// `import c "header.h" with (...)` / `import cpp "header.h" with (...)`
    Foreign { language: ForeignLanguage, header: String, options: Vec<AttrArg> },
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ForeignLanguage {
    C,
    Cpp,
}

#[derive(Debug)]
pub struct ImportItem {
    pub name: Ident,
    pub alias: Option<Ident>,
}

#[derive(Debug)]
pub struct FnDecl {
    pub name: Ident,
    pub is_unsafe: bool,
    pub dispatch: Dispatch,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,
    pub where_clause: Vec<Bound>,
    /// `None` inside an `interface` or `extern` block.
    pub body: Option<Block>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Dispatch {
    Static,
    Virtual,
    Override,
}

/// `[FN-1]` — how a parameter is passed. The call site never writes `&`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Mode {
    /// `x: T` — shared borrow. The default.
    Borrow,
    /// `mut x: T` — mutable borrow (inout).
    Mut,
    /// `owned x: T` — consumed.
    Owned,
}

#[derive(Debug)]
pub struct Param {
    pub id: NodeId,
    pub mode: Mode,
    pub kind: ParamKind,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug)]
pub enum ParamKind {
    /// `self`, `mut self`, `owned self`, or `self: Box[Self]`.
    Receiver { ty: Option<TypeExpr> },
    Named { name: Ident, ty: TypeExpr },
}

#[derive(Debug)]
pub struct GenericParam {
    pub id: NodeId,
    pub name: Ident,
    pub bounds: Vec<TypeExpr>,
    pub default: Option<TypeExpr>,
    /// `const N: usize` — a const generic.
    pub const_ty: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Bound {
    pub subject: TypeExpr,
    pub bounds: Vec<TypeExpr>,
    pub span: Span,
}

#[derive(Debug)]
pub struct StructDecl {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub implements: Vec<TypeExpr>,
    pub where_clause: Vec<Bound>,
    pub members: Vec<Member>,
}

#[derive(Debug)]
pub struct ClassDecl {
    pub name: Ident,
    pub openness: Openness,
    pub generics: Vec<GenericParam>,
    /// `[GRM-1]` — at most one base class, written in parentheses.
    pub base: Option<TypeExpr>,
    pub implements: Vec<TypeExpr>,
    pub where_clause: Vec<Bound>,
    pub members: Vec<Member>,
}

/// `[CLS-4]` — a class is final unless declared `open` or `abstract`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Openness {
    Final,
    Open,
    Abstract,
}

#[derive(Debug)]
pub struct EnumDecl {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    pub implements: Vec<TypeExpr>,
    pub variants: Vec<Variant>,
    pub members: Vec<Member>,
}

#[derive(Debug)]
pub struct Variant {
    pub id: NodeId,
    pub attrs: Vec<Attribute>,
    pub doc: Option<String>,
    pub name: Ident,
    pub fields: Vec<VariantField>,
    /// `Forward = 0` — an explicit discriminant.
    pub discriminant: Option<Expr>,
    pub span: Span,
}

#[derive(Debug)]
pub struct VariantField {
    pub name: Option<Ident>,
    pub ty: TypeExpr,
    pub span: Span,
}

#[derive(Debug)]
pub struct InterfaceDecl {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    /// `interface Ord: Eq` — supertraits (`[IFC-3]`).
    pub supertraits: Vec<TypeExpr>,
    pub where_clause: Vec<Bound>,
    pub members: Vec<Member>,
}

#[derive(Debug)]
pub struct ExtendDecl {
    pub generics: Vec<GenericParam>,
    pub target: TypeExpr,
    pub implements: Vec<TypeExpr>,
    pub where_clause: Vec<Bound>,
    pub members: Vec<Member>,
}

#[derive(Debug)]
pub struct ConstDecl {
    pub name: Ident,
    pub ty: Option<TypeExpr>,
    pub value: Expr,
}

#[derive(Debug)]
pub struct StaticDecl {
    pub name: Ident,
    /// `[STA-1]` — any access to a `static mut` requires `unsafe`.
    pub is_mut: bool,
    pub ty: TypeExpr,
    pub value: Expr,
}

#[derive(Debug)]
pub struct TypeAlias {
    pub name: Ident,
    pub generics: Vec<GenericParam>,
    /// `None` for an associated type declaration inside an interface.
    pub value: Option<TypeExpr>,
    pub bounds: Vec<TypeExpr>,
}

#[derive(Debug)]
pub struct ExternBlock {
    pub is_unsafe: bool,
    pub abi: String,
    pub items: Vec<Item>,
}

/// A member of a struct, class, enum, interface or `extend` body.
#[derive(Debug)]
pub struct Member {
    pub id: NodeId,
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    /// `[MOD-7]` — `pub(read)` / `pub(package, read)`.
    pub read_only_outside: bool,
    pub doc: Option<String>,
    pub kind: MemberKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum MemberKind {
    Field(FieldDecl),
    Fn(FnDecl),
    Const(ConstDecl),
    TypeAlias(TypeAlias),
}

#[derive(Debug)]
pub struct FieldDecl {
    pub name: Ident,
    pub ty: TypeExpr,
    pub default: Option<Expr>,
    /// `[CLS-9]` — `let name: T` is immutable after `init`.
    pub is_let: bool,
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct TypeExpr {
    pub id: NodeId,
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum TypeKind {
    /// `std.math.Vec3`, `Array[T]`, `Iterator[Item = i32]`.
    Path { segments: Vec<Ident>, args: Vec<GenericArg> },
    /// `ref T` / `ref mut T`.
    Ref { mutable: bool, inner: Box<TypeExpr> },
    /// `*T` / `*mut T`.
    Ptr { mutable: bool, inner: Box<TypeExpr> },
    Tuple(Vec<TypeExpr>),
    Fn { abi: Option<String>, params: Vec<TypeExpr>, ret: Option<Box<TypeExpr>> },
    Dyn(Vec<TypeExpr>),
    /// `[T; N]`.
    Array { elem: Box<TypeExpr>, len: Box<Expr> },
    SelfType,
    Void,
    /// `!`.
    Never,
    /// A hole for inference; only produced by desugaring, never parsed.
    Infer,
}

#[derive(Debug)]
pub enum GenericArg {
    Type(TypeExpr),
    /// A const-generic argument.
    Const(Expr),
    /// `Item = i32` — an associated type binding.
    Assoc { name: Ident, ty: TypeExpr },
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct Block {
    pub id: NodeId,
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Stmt {
    pub id: NodeId,
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum StmtKind {
    /// `x: T = e` — always a declaration (`[GRM-4]`). `init` may be absent.
    Decl { pattern: Pattern, ty: Option<TypeExpr>, init: Option<Expr> },
    /// `a, b = e` and augmented assignment. Whether this declares or assigns
    /// is decided in name resolution (`[GRM-4]`, `[GRM-5]`).
    Assign { targets: Vec<Expr>, op: Option<BinOp>, value: Expr },
    Expr(Expr),
    Return(Option<Expr>),
    Break { label: Option<Ident> },
    Continue { label: Option<Ident> },
    Pass,
    If(IfStmt),
    While { label: Option<Ident>, cond: Condition, body: Block, else_block: Option<Block> },
    For { label: Option<Ident>, pattern: Pattern, iter: Expr, body: Block, else_block: Option<Block> },
    Match { scrutinee: Expr, arms: Vec<MatchArm> },
    With { items: Vec<WithItem>, body: Block },
    Defer(Block),
    Unsafe(Block),
    Comptime(Block),
}

#[derive(Debug)]
pub struct IfStmt {
    pub cond: Condition,
    pub then_block: Block,
    /// `elif` chains parse as a nested `If` in the else position.
    pub else_block: Option<Box<ElseBranch>>,
}

#[derive(Debug)]
pub enum ElseBranch {
    Block(Block),
    If(IfStmt),
}

/// `if cond:` or the pattern form `if Some(x) = opt:`.
#[derive(Debug)]
pub enum Condition {
    Expr(Expr),
    Pattern { pattern: Pattern, value: Expr },
}

#[derive(Debug)]
pub struct WithItem {
    pub pattern: Option<Pattern>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug)]
pub struct MatchArm {
    pub id: NodeId,
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: MatchArmBody,
    pub span: Span,
}

/// `[GRM-10]` — statement arms use `:` and a block, expression arms use `=>`.
/// Mixing the two in one `match` is `E0103`.
#[derive(Debug)]
pub enum MatchArmBody {
    Block(Block),
    Expr(Expr),
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct Expr {
    pub id: NodeId,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum ExprKind {
    Lit(Literal),
    /// A bare name or a `::`/`.`-qualified path.
    Path { segments: Vec<Ident> },
    SelfExpr,
    Field { base: Box<Expr>, name: Ident },
    TupleField { base: Box<Expr>, index: u32 },
    /// `[GRM-8]` — `name[...]` is an index or a generic instantiation, and
    /// which one is not known until name resolution.
    IndexOrInstantiate { base: Box<Expr>, args: Vec<Expr> },
    Call { callee: Box<Expr>, args: Vec<Arg> },
    MethodCall { recv: Box<Expr>, name: Ident, generic_args: Vec<GenericArg>, args: Vec<Arg> },
    Unary { op: UnOp, operand: Box<Expr> },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    /// `and` / `or`, kept apart from `Binary` because they short-circuit
    /// (`[EXP-3]`).
    Logical { op: LogicalOp, lhs: Box<Expr>, rhs: Box<Expr> },
    /// `x if c else y`.
    Ternary { then_expr: Box<Expr>, cond: Box<Expr>, else_expr: Box<Expr> },
    Range { lo: Option<Box<Expr>>, hi: Option<Box<Expr>>, inclusive: bool },
    Cast { expr: Box<Expr>, ty: TypeExpr },
    /// `h as? D` and `h as! D` — a dynamic downcast of a class handle.
    Downcast { expr: Box<Expr>, ty: TypeExpr, forced: bool },
    /// `e?` — early return on `None`/`Err` (`[ERR-2]`).
    Try(Box<Expr>),
    /// `a?.f` / `a?.m()`.
    OptChain { base: Box<Expr>, name: Ident, args: Option<Vec<Arg>> },
    Lambda(Lambda),
    Match { scrutinee: Box<Expr>, arms: Vec<MatchArm> },
    Tuple(Vec<Expr>),
    ArrayLit(Vec<Expr>),
    /// `[value; count]`.
    ArrayRepeat { value: Box<Expr>, count: Box<Expr> },
    FString(Vec<FStringPart>),
    /// `ref place` / `ref mut place`, needed only when initialising a
    /// `ref`-typed local or a view struct field.
    RefOf { mutable: bool, place: Box<Expr> },
    Paren(Box<Expr>),
    /// A node the parser could not build. Keeps later stages from cascading.
    Error,
}

#[derive(Debug)]
pub enum FStringPart {
    Text(String),
    Expr { expr: Box<Expr>, format_spec: Option<String> },
}

#[derive(Debug)]
pub struct Arg {
    /// `f(x=1)` — a named argument (`[TYP-25]`).
    pub name: Option<Ident>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug)]
pub struct Lambda {
    /// `owned fn(…)` captures by move/copy/retain and may escape (`[CLO-1]`).
    pub is_owned: bool,
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,
    pub body: LambdaBody,
}

#[derive(Debug)]
pub enum LambdaBody {
    Expr(Box<Expr>),
    Block(Block),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Literal {
    /// `[LEX-16]` — untyped until the type checker resolves it.
    Int { value: u128, suffix: Option<ember_lexer_types::IntSuffix> },
    Float { value: f64, suffix: Option<ember_lexer_types::FloatSuffix> },
    Bool(bool),
    Char(char),
    Str(String),
    Bytes(Vec<u8>),
    CStr(Vec<u8>),
}

/// Re-exported literal suffix types, so `ember_ast` does not force every
/// consumer to depend on the lexer.
pub mod ember_lexer_types {
    pub use ember_lexer::token::{FloatSuffix, IntSuffix};
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum UnOp {
    /// `-`
    Neg,
    /// `not`
    Not,
    /// `~`
    BitNot,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    /// `is` / `is not` — handle identity (`[VI.3]`).
    Is,
    IsNot,
    /// `in` / `not in` — `Contains`.
    In,
    NotIn,
}

impl BinOp {
    pub fn as_str(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Pow => "**",
            BinOp::BitAnd => "&",
            BinOp::BitOr => "|",
            BinOp::BitXor => "^",
            BinOp::Shl => "<<",
            BinOp::Shr => ">>",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Le => "<=",
            BinOp::Ge => ">=",
            BinOp::Is => "is",
            BinOp::IsNot => "is not",
            BinOp::In => "in",
            BinOp::NotIn => "not in",
        }
    }

    /// `[III.5]` — comparison operators are non-associative, so `a < b < c` is
    /// `E0102` rather than Python's chaining.
    pub fn is_comparison(self) -> bool {
        matches!(
            self,
            BinOp::Eq
                | BinOp::Ne
                | BinOp::Lt
                | BinOp::Gt
                | BinOp::Le
                | BinOp::Ge
                | BinOp::Is
                | BinOp::IsNot
                | BinOp::In
                | BinOp::NotIn
        )
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum LogicalOp {
    And,
    Or,
}

// ---------------------------------------------------------------------------
// Patterns
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct Pattern {
    pub id: NodeId,
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum PatternKind {
    /// `_`
    Wild,
    Lit(Literal),
    Range { lo: Literal, hi: Literal, inclusive: bool },
    /// `[GRM-12]` — an identifier resolves to a unit variant or `const` if one
    /// is in scope, and is a fresh binding otherwise. Resolution decides.
    Bind { name: Ident, by_ref: bool, mutable: bool, sub: Option<Box<Pattern>> },
    Path { segments: Vec<Ident> },
    /// `Shape.Circle(r)` / `Point(x=1, y=2)`.
    Constructor { path: Vec<Ident>, fields: Vec<FieldPattern>, has_rest: bool },
    Tuple(Vec<Pattern>),
    Slice { prefix: Vec<Pattern>, rest: Option<Option<Ident>>, suffix: Vec<Pattern> },
    Or(Vec<Pattern>),
    Error,
}

#[derive(Debug)]
pub struct FieldPattern {
    pub name: Option<Ident>,
    pub pattern: Pattern,
    pub span: Span,
}

/// Hands out `NodeId`s for one file.
#[derive(Default)]
pub struct NodeIds {
    next: u32,
}

impl NodeIds {
    pub fn new() -> NodeIds {
        NodeIds::default()
    }

    pub fn next(&mut self) -> NodeId {
        let id = NodeId(self.next);
        self.next += 1;
        id
    }

    pub fn count(&self) -> u32 {
        self.next
    }
}
