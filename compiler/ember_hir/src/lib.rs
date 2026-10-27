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
use ember_types::{ClassId, EnumId, FpMode, OverflowPolicy, RangeId, StructId, Ty};

/// A resolved item: a function, a struct, a constant.
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct DefId(pub u32);

/// A local slot within one function body. Parameters occupy the first slots,
/// in declaration order.
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LocalId(pub u32);

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Program {
    pub functions: Vec<Function>,
    /// The entry point, if this compilation unit declares one (`[FN-8]`).
    pub main: Option<DefId>,
    /// `[TST-3]` — the `@test` functions, in declaration order, module by
    /// module: what `ember test` runs, each in a process of its own.
    pub tests: Vec<TestFn>,
    /// `[EFF-7]` — the calls written `@assume_noalloc(call)`, by span: the effect analysis takes
    /// each to carry no `Alloc`.
    pub assume_noalloc: Vec<Span>,
    /// `[BLD-1]` — the functions of the modules whose checked form an earlier build kept: their
    /// bodies were not checked, and lowering leaves them out (the kept MIR stands for them).
    pub reused: std::collections::HashSet<DefId>,
}

/// `[TST-3]` — one `@test` function: a free function with no parameters
/// that returns nothing.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct TestFn {
    pub def: DefId,
    /// The module-qualified name the runner reports and filters by.
    pub name: Symbol,
    /// The C symbol the test binary's entry calls.
    pub symbol: String,
    /// `@should_panic`: the test passes only if it panics.
    pub should_panic: bool,
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
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Mode {
    Borrow,
    Mut,
    Owned,
}

/// `[LT-1a]` — one source `@borrows(…)` names: a parameter, by position, or (ODR-196) a view
/// field of the receiver, `self.f`, by the field's index in the receiver's type. The result of a
/// call borrows what that field borrows, and the call's loan on the receiver ends with the call.
/// Ordered by parameter, a whole parameter before its fields; a list names a parameter whole or
/// by fields, never both.
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct BorrowSource {
    pub parameter: usize,
    pub field: Option<usize>,
}

impl BorrowSource {
    pub fn whole(parameter: usize) -> BorrowSource {
        BorrowSource { parameter, field: None }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
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
    /// `[CTL-10]` — this arm's declaration is of a name hoisted beside the
    /// branch, to the local given: the two are one variable, set in the arm
    /// and read after the branch (D-490).
    pub hoisted_into: Option<LocalId>,
    /// `[SIMD-2]` — this local identifies a `@simd` loop (its counter, or its
    /// iterator when it is not counted): the loop's span and whether it is
    /// `@simd(assert)`.
    pub simd: Option<(Span, bool)>,
    /// `[SIMD-5]` — a running total a `@parallel(reduce=…)` declares: its
    /// floating-point sum may be reordered.
    pub declared_reduction: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Param {
    pub local: LocalId,
    pub mode: Mode,
}

/// The C ABI behind a hand-declared function that needs an Ember-to-C adapter:
/// `count(n)`, `fixed(N)`, or a borrowed aggregate passed by C value.
/// Indices name visible Ember parameters; count witnesses have no visible
/// parameter and are supplied from the associated span(s).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FfiCounted {
    pub foreign_symbol: String,
    /// `[FFI-9]` — the foreign function's calling convention (`"vectorcall"`), which its
    /// declaration and its call keep; the wrapper itself is an Ember function.
    pub abi: Option<String>,
    pub abi_params: Vec<FfiAbiParam>,
    /// A borrowed fixed-array result arrives as a C element pointer.
    pub result_array: Option<FfiArrayResult>,
    /// A counted result's length is supplied by this visible input span.
    pub result_span: Option<FfiSpanResult>,
    /// `[FFI-36a]` — a result contracted `returns_owned(destructor=f)`: the C
    /// pointer becomes a `ForeignBox` the runtime records.
    pub result_owned: Option<FfiOwnedResult>,
    /// `[FFI-11]`, `[FFI-16]` — a call that returns its `out` parameters, or a
    /// status as a `Result`, in place of C's own result.
    pub outputs: Option<FfiOutputs>,
    /// `[FFI-21]` — the callables C calls back, each named by an
    /// `FfiAbiParam::Callback` (its trampoline) and an
    /// `FfiAbiParam::CallbackData` (its user data).
    pub callbacks: Vec<FfiCallback>,
}

/// `[FFI-21]`, `[FFI-11]` — a parameter contracted `callback(mode, user_data=p)`:
/// a callable C calls through a trampoline, which finds it at the user data.
/// The callable is the instance's own (a lambda's environment, or a callable
/// value): the wrapper is monomorphised per callable, so a lambda is called
/// directly, as through any callable parameter (`[CLO-3]`).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FfiCallback {
    /// The visible parameter holding the callable.
    pub public_index: usize,
    /// Its type: a lambda's environment struct, or a callable value.
    pub callable: Ty,
    /// The lambda's body, when the callable is one: called directly with its
    /// environment. Otherwise the callable value's native entry is called.
    pub closure: Option<String>,
    /// The C function pointer type C calls (`extern "C" fn(…)`): its
    /// parameters, the user data's among them, its result and its calling
    /// convention.
    pub c_fn: Ty,
    /// Which of the C function pointer's parameters is the user data.
    pub slot: usize,
    /// The user data parameter's type as C has it: a pointer, or an integer.
    pub data: Ty,
    pub mode: FfiCallbackMode,
    /// `[FFI-22]`, `[FFI-33b]` — the threads C calls it on: under `main` or
    /// `creator` the debug profile checks each call.
    pub threads: ExportThreadPolicy,
    /// `[FFI-21]` (ODR-253) — `nullable`: the parameter's `Option` of the
    /// callable value (`callable` is its payload). `None` gives C a null
    /// function pointer and null user data, and registers nothing.
    pub nullable: Option<Ty>,
    /// `[FFI-21]` — for `retained`: the function the token's end calls to
    /// unregister it (`destroys(f)`), and the token's type.
    pub unregister: Option<FfiUnregister>,
}

/// `[FFI-21]` — how long C keeps a callback: for the call (`borrowed`, the
/// callable stays where the caller has it), until its one call (`once`), or
/// until the `Retained` token the call returns ends (`retained`).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiCallbackMode {
    Borrowed,
    Once,
    Retained,
}

/// `[FFI-21]` — `destroys(f)` of a function registering a `retained` callback:
/// `f`'s C symbol and result, what each of its parameters is given, and the
/// `Retained[ForeignCallback]` token the call returns.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FfiUnregister {
    pub symbol: String,
    pub ret: Ty,
    pub args: Vec<FfiUnregisterArg>,
    /// `f`'s parameter types, one per argument.
    pub params: Vec<Ty>,
    pub token: Ty,
    /// What the call returns for the callback: the token, or with `nullable`
    /// an `Option` of it (`None` when the call registered nothing).
    pub output: Ty,
}

/// What a parameter of `destroys(f)` is given: the trampoline, the user data,
/// what the registering call passed for one of its scalar parameters, or
/// (ODR-246) the result C returned from it, each kept with the callable.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum FfiUnregisterArg {
    Entry,
    Data,
    Saved { public_index: usize, ty: Ty },
    Result { ty: Ty },
}

/// `[FFI-11]`, `[FFI-16]` — C's own result type, and with a status the value
/// of its success code: the Ember result is `Result[T, N]` then, `T` the
/// `out` parameters (`void`, the one, or a tuple), and otherwise C's result
/// followed by them (ODR-137).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FfiOutputs {
    pub c_result: Ty,
    pub status_ok: Option<i128>,
}

/// `[FFI-36a]`, `[FFI-33b]` — a new foreign object the caller owns: freed by
/// `destructor` (whose result, when it has one, is dropped), `None` for null
/// when `nullable`, and its box records its creator thread when `creator`.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FfiOwnedResult {
    pub destructor: Symbol,
    pub destructor_ret: Ty,
    pub pointee: Ty,
    pub nullable: bool,
    pub creator: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FfiSpanResult {
    pub public_index: usize,
    pub elem: Ty,
    pub nullable: bool,
    pub mutable: bool,
    pub source_nullable: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FfiArrayResult {
    pub elem: Ty,
    pub mutable: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum FfiAbiParam {
    Value { public_index: usize, ty: Ty, mode: Mode },
    SpanPointer { public_index: usize, elem: Ty, mutable: bool, nullable: bool },
    ArrayPointer { public_index: usize, elem: Ty, mutable: bool, nullable: bool },
    /// A count witness: the span lengths at its ABI position, or with `inout`
    /// (`[FFI-11b]`'s `inout_count(p)`) the address of a local holding them,
    /// which C writes back and the call returns.
    Count { sources: Vec<FfiCountSource>, ty: Ty, inout: bool },
    /// `[FFI-11]` — `param(p, owned, one)`: the `ForeignBox` (an `Option` of
    /// one when `nullable`) gives C its pointer and leaves the runtime's
    /// record, freeing nothing.
    Owned { public_index: usize, nullable: bool },
    /// `[FFI-11]`, `[FFI-23]` — `param(p, retained, one)`: C keeps the
    /// address of the object a `Retained` token holds; the token stays.
    Retained { public_index: usize },
    /// `[FFI-11]` — `param(p, out, one)`: the address of a local C writes,
    /// which the call returns; not a parameter of the Ember function.
    Out { ty: Ty },
    /// `[FFI-21]` — the trampoline of `FfiCounted::callbacks[index]`.
    Callback { index: usize },
    /// `[FFI-21]` — the user data of `FfiCounted::callbacks[index]`: the
    /// callable's address, or its allocation's; not a parameter of the Ember
    /// function.
    CallbackData { index: usize },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FfiCountSource {
    pub public_index: usize,
    pub nullable: bool,
}

/// The resolved calling-thread contract of a defined C export (`[FFI-33]`).
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportThreadPolicy {
    #[default]
    Any,
    Main,
    Creator,
}

impl ExportThreadPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Main => "main",
            Self::Creator => "creator",
        }
    }
}

/// `[CG-C-3a]` — `@inline` is binding: the function is always inlined.
/// `@noinline`, `@cold` and `@hot` are hints passed to the C compiler.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InlineHint {
    pub always: bool,
    pub never: bool,
    pub cold: bool,
    pub hot: bool,
}

/// `[EFF-1]` — an effect set: which kinds of effect occur, never where (Part X.1). The
/// `RuntimeCheck(k)` effects are one bit per kind (`[EFF-9]`).
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Effects(pub u16);

impl Effects {
    pub const NONE: Effects = Effects(0);
    pub const ALLOC: Effects = Effects(1 << 0);
    pub const SYNC: Effects = Effects(1 << 1);
    pub const LOCK: Effects = Effects(1 << 2);
    pub const BLOCK: Effects = Effects(1 << 3);
    pub const IO: Effects = Effects(1 << 4);
    /// `Panic(Explicit)`: the panics the programmer writes, and allocation failure.
    pub const PANIC: Effects = Effects(1 << 5);
    pub const CHECK_ALIASING: Effects = Effects(1 << 6);
    pub const CHECK_BOUNDS: Effects = Effects(1 << 7);
    pub const CHECK_STALE: Effects = Effects(1 << 8);
    pub const CHECK_ARITHMETIC: Effects = Effects(1 << 9);
    pub const UNSAFE: Effects = Effects(1 << 10);
    pub const FFI: Effects = Effects(1 << 11);
    pub const NONDET: Effects = Effects(1 << 12);
    /// What a call contributes when nothing declares its effects (`[EFF-2]`).
    pub const ALL: Effects = Effects((1 << 13) - 1);
    /// `[EFF-19]` — `@realtime`'s set when the manifest declares none.
    pub const REALTIME_DEFAULT: Effects = Effects(Self::ALLOC.0 | Self::LOCK.0 | Self::BLOCK.0 | Self::PANIC.0);

    /// Each effect, with the name Part X.1 gives it.
    pub const NAMED: [(Effects, &'static str); 13] = [
        (Self::ALLOC, "Alloc"),
        (Self::SYNC, "Sync"),
        (Self::LOCK, "Lock"),
        (Self::BLOCK, "Block"),
        (Self::IO, "Io"),
        (Self::PANIC, "Panic(Explicit)"),
        (Self::CHECK_ALIASING, "RuntimeCheck(Aliasing)"),
        (Self::CHECK_BOUNDS, "RuntimeCheck(Bounds)"),
        (Self::CHECK_STALE, "RuntimeCheck(Stale)"),
        (Self::CHECK_ARITHMETIC, "RuntimeCheck(Arithmetic)"),
        (Self::UNSAFE, "Unsafe"),
        (Self::FFI, "FFI"),
        (Self::NONDET, "Nondet"),
    ];

    /// `[EFF-5]`, `[EFF-19]` — each contract, by the attribute that writes it (and the name a
    /// manifest's `[realtime] contracts` gives it), with the effect it forbids.
    pub const CONTRACTS: [(Effects, &'static str); 7] = [
        (Self::ALLOC, "noalloc"),
        (Self::SYNC, "nosync"),
        (Self::LOCK, "nolock"),
        (Self::BLOCK, "noblock"),
        (Self::IO, "noio"),
        (Self::PANIC, "nopanic(explicit)"),
        (Self::NONDET, "deterministic"),
    ];

    /// The contracts that forbid these effects, as attributes: `@noalloc @nolock`.
    pub fn contract_names(self) -> String {
        let names: Vec<String> =
            Self::CONTRACTS.iter().filter(|(effect, _)| self.contains(*effect)).map(|(_, name)| format!("@{name}")).collect();
        names.join(" ")
    }

    pub fn contains(self, other: Effects) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn intersects(self, other: Effects) -> bool {
        self.0 & other.0 != 0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The single effects in this set, in Part X.1's order.
    pub fn each(self) -> impl Iterator<Item = (Effects, &'static str)> {
        Self::NAMED.into_iter().filter(move |(effect, _)| self.contains(*effect))
    }
}

impl std::ops::BitOr for Effects {
    type Output = Effects;
    fn bitor(self, other: Effects) -> Effects {
        Effects(self.0 | other.0)
    }
}

impl std::ops::BitOrAssign for Effects {
    fn bitor_assign(&mut self, other: Effects) {
        self.0 |= other.0;
    }
}

impl std::ops::BitAnd for Effects {
    type Output = Effects;
    fn bitand(self, other: Effects) -> Effects {
        Effects(self.0 & other.0)
    }
}

/// `[EFF-5]` — a function's contracts (Part X.2): the effects it may not have, and whether
/// `@realtime` named some of them (`[EFF-19]`, whose diagnostics name both).
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Contracts {
    pub forbidden: Effects,
    pub realtime: Effects,
    /// `[EFF-3]`, `[FFI-10]` — for a foreign declaration, the effects its
    /// contract gives it: those `@ffi(effects=[…])` states and `FFI`, every
    /// effect when it states none, and `Nondet` unless `@ffi(deterministic)`
    /// (`[DET-2]`). `None` for a function with an Ember body.
    pub foreign: Option<Effects>,
    /// `[FFI-1]`, `[FFI-11]` — for a foreign declaration, the threads its
    /// contract lets it be called on (`@ffi(threads(…))`); `None` when its
    /// contract states none, so its thread rule is unknown.
    pub foreign_threads: Option<ExportThreadPolicy>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
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
    /// `[FFI-33]` — resolved from the attribute or its declaration file.
    pub export_thread_policy: ExportThreadPolicy,
    pub params: Vec<Param>,
    pub locals: Vec<LocalDecl>,
    pub ret: Ty,
    pub body: Block,
    pub span: Span,
    /// `[TYP-8]` — from `@overflow(...)` on this function, or the profile's
    /// default when the attribute is absent.
    pub overflow: OverflowPolicy,
    /// `[TYP-9]` — from `@fastmath` or `@fp(contract)`; a lambda's is its
    /// enclosing function's.
    pub fp: FpMode,
    /// `[CG-C-3a]` — from `@inline`, `@noinline`, `@cold` and `@hot`.
    pub inline: InlineHint,
    /// `[EFF-12]`, `[EFF-13]` — declared `@static_safe`: no dynamic aliasing
    /// check may remain in its body.
    pub static_safe: bool,
    /// `[EFF-5]` — the contract attributes written on it (`@noalloc`, …).
    pub contracts: Contracts,
    /// `[LT-1a]` — what `@borrows(…)` names, by parameter *position* (and a
    /// receiver's field by index), when the attribute is written. `None`
    /// means elision decides (`[LT-1]`).
    ///
    /// Positions rather than names because that is what survives to MIR, where
    /// a parameter is a local and the borrow checker has to answer "which
    /// parameter did this reference come from".
    pub borrows: Option<Vec<BorrowSource>>,
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
    /// `[CG-C-10]` — the function's Ember path where `name` is not it: a method's, its type's path
    /// and its name (`geo.Grid.at`). A backtrace names the function by it.
    pub path: Option<Symbol>,
    pub class_owner: Option<ClassId>,
    /// `[DSP-2]` — the vtable slot assigned to a virtual or override method.
    /// `None` means ordinary static dispatch.
    pub class_virtual_slot: Option<usize>,
    /// `[BRW-10]` — a struct or enum method not visible outside its package:
    /// a call to it borrows only the fields of its receiver its body uses.
    pub borrows_used_fields: bool,
    /// A bodyless `virtual` declaration on an abstract class. It carries the
    /// checked callable and vtable signature but is never emitted as a C
    /// function; a concrete override supplies its slot implementation.
    pub is_abstract: bool,
    /// A bodyless declaration in `unsafe extern`: emit a C prototype and
    /// resolve calls against the foreign linker symbol, never a C body.
    pub is_extern_declaration: bool,
    /// `[FFI-8]` — a variadic C function (`@ffi(variadic)`): its prototype ends `...`, and a call
    /// passes further arguments after the declared parameters.
    pub variadic: bool,
    /// ODR-074 and `[FFI-9]`: an Ember wrapper around a differing C signature.
    pub ffi_counted: Option<FfiCounted>,
}

impl Function {
    pub fn local(&self, id: LocalId) -> &LocalDecl {
        &self.locals[id.0 as usize]
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum DestructureBinding {
    Let { local: LocalId, value: Expr },
    Assign { place: Expr, value: Expr },
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Expr {
    pub ty: Ty,
    pub kind: ExprKind,
    pub span: Span,
}

/// One ABI-visible slot in a `[TYP-22]` dynamic interface table. The HIR
/// retains this declaration-derived fact because interface declarations do
/// not otherwise cross the type checking to MIR boundary.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InterfaceSlot {
    pub params: Vec<Ty>,
    pub ret: Ty,
}

/// One concrete method supplied to an erased `[TYP-22]` interface table.
/// The interface declaration establishes the erased slot signature; this
/// records only the concrete body and receiver mode that its adapter
/// must call.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct InterfaceAdapterSlot {
    pub implementation: DefId,
    pub receiver: Mode,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum ExprKind {
    /// An integer literal, already narrowed to `ty`.
    Int(u128),
    Float(f64),
    Bool(bool),
    Str(String),
    CStr(Vec<u8>),
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
        /// `[FN-5]` — caller locals bound to evaluated argument slots before
        /// an omitted default reads an earlier parameter. Present only for a
        /// call with defaults; indices include a method receiver at slot 0.
        default_arg_locals: Option<Vec<LocalId>>,
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
    /// `[CLO-3]`, `[CLO-10]` — an `owned fn` closure where an owned callable
    /// value is wanted: `closure` is its body, and the value takes over its
    /// environment `env`. Its type is the `fn(A) -> R`.
    MakeCallable { closure: DefId, env: Box<Expr> },
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
    /// `[FN-5]`, `[TYP-8]` — an omitted argument is evaluated in the caller
    /// but its operators retain the declaring function's lexical policy.
    OverflowScope { policy: OverflowPolicy, expr: Box<Expr> },
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

/// `[IDE-4]` — every expression of `block`, each before those inside it: what an editor reads to
/// find what is at a place.
pub fn visit_block<'a>(block: &'a Block, f: &mut dyn FnMut(&'a Expr)) {
    for stmt in &block.stmts {
        visit_stmt(stmt, f);
    }
}

pub fn visit_stmt<'a>(stmt: &'a Stmt, f: &mut dyn FnMut(&'a Expr)) {
    match stmt {
        Stmt::Let { init, .. } => init.iter().for_each(|init| visit_expr(init, f)),
        Stmt::Assign { place, value } => {
            visit_expr(place, f);
            visit_expr(value, f);
        }
        Stmt::Destructure { value, bindings, .. } => {
            visit_expr(value, f);
            for binding in bindings {
                match binding {
                    DestructureBinding::Let { value, .. } => visit_expr(value, f),
                    DestructureBinding::Assign { place, value } => {
                        visit_expr(place, f);
                        visit_expr(value, f);
                    }
                }
            }
        }
        Stmt::Expr(expr) => visit_expr(expr, f),
        Stmt::Return(value) => value.iter().for_each(|value| visit_expr(value, f)),
        Stmt::If { cond, then_block, else_block } | Stmt::While { cond, body: then_block, else_block } => {
            visit_expr(cond, f);
            visit_block(then_block, f);
            else_block.iter().for_each(|block| visit_block(block, f));
        }
        Stmt::ForRange { start, end, body, else_block, .. } => {
            visit_expr(start, f);
            visit_expr(end, f);
            visit_block(body, f);
            else_block.iter().for_each(|block| visit_block(block, f));
        }
        Stmt::Block(block) | Stmt::Defer(block) => visit_block(block, f),
        Stmt::Break { .. } | Stmt::Continue { .. } => {}
    }
}

pub fn visit_expr<'a>(expr: &'a Expr, f: &mut dyn FnMut(&'a Expr)) {
    f(expr);
    match &expr.kind {
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Str(_)
        | ExprKind::CStr(_)
        | ExprKind::Local(_)
        | ExprKind::FnValue(_)
        | ExprKind::Error => {}
        ExprKind::Field { base, .. } | ExprKind::EnumField { base, .. } => visit_expr(base, f),
        ExprKind::Call { args, .. }
        | ExprKind::Builtin { args, .. }
        | ExprKind::StructLit { fields: args, .. }
        | ExprKind::TupleLit(args)
        | ExprKind::ArrayLit(args)
        | ExprKind::EnumLit { fields: args, .. } => args.iter().for_each(|arg| visit_expr(arg, f)),
        ExprKind::InterfaceCall { receiver, args, .. } => {
            visit_expr(receiver, f);
            args.iter().for_each(|arg| visit_expr(arg, f));
        }
        ExprKind::CallIndirect { callee, args, .. } => {
            visit_expr(callee, f);
            args.iter().for_each(|arg| visit_expr(arg, f));
        }
        ExprKind::InterfaceUpcast { expr: inner, .. }
        | ExprKind::DynBoxNew { value: inner, .. }
        | ExprKind::MakeCallable { env: inner, .. }
        | ExprKind::ArrayRepeat { value: inner, .. }
        | ExprKind::Ref { place: inner, .. }
        | ExprKind::Deref(inner)
        | ExprKind::Unary { operand: inner, .. }
        | ExprKind::Cast { expr: inner, .. }
        | ExprKind::Widen { expr: inner, .. }
        | ExprKind::EraseRange(inner)
        | ExprKind::OverflowScope { expr: inner, .. } => visit_expr(inner, f),
        ExprKind::Index { base, index } => {
            visit_expr(base, f);
            visit_expr(index, f);
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            visit_expr(lhs, f);
            visit_expr(rhs, f);
        }
        ExprKind::Match { scrutinee, arms } => {
            visit_expr(scrutinee, f);
            for arm in arms {
                arm.guard.iter().for_each(|guard| visit_expr(guard, f));
                match &arm.body {
                    MatchArmBody::Block(block) => visit_block(block, f),
                    MatchArmBody::Expr(body) => visit_expr(body, f),
                }
            }
        }
        ExprKind::FString { parts, .. } => {
            for part in parts {
                match part {
                    FStringPart::Text(_) => {}
                    FStringPart::Value(value, _) | FStringPart::Write { value, .. } => visit_expr(value, f),
                }
            }
        }
        ExprKind::Block { block, value } => {
            visit_block(block, f);
            visit_expr(value, f);
        }
        ExprKind::ClassNew { default_fields, args, .. } => {
            default_fields.iter().flatten().for_each(|field| visit_expr(field, f));
            args.iter().for_each(|arg| visit_expr(arg, f));
        }
    }
}

/// One piece of an f-string: literal text, or a value to format into it.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum FStringPart {
    Text(String),
    /// A value, and how `{x:spec}`, `{x!r}` or `{x=}` asks for it to be
    /// written (`[LEX-19]`); `None` is its plain text, as `print` writes it.
    Value(Expr, Option<FormatSpec>),
    /// ADR-186 — a value whose type has a written `Display`: `writer` is
    /// `std.fmt.write_display`'s instance for its type, called with `value`
    /// (passed as that borrowed parameter takes it) and the text being built.
    Write { writer: DefId, value: Expr },
}

impl FStringPart {
    /// The value this piece writes, if it is not literal text.
    pub fn value(&self) -> Option<&Expr> {
        match self {
            FStringPart::Text(_) => None,
            FStringPart::Value(value, _) | FStringPart::Write { value, .. } => Some(value),
        }
    }
}

/// `[TXT-10]` (ODR-029) — what `parse` reads.
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ParseKind {
    Signed,
    Unsigned,
    /// D-272 — `i128` and `u128`, whose bounds no 64-bit argument carries.
    I128,
    U128,
    /// G8-4 — the 256-bit counts, read over their whole range.
    I256,
    U256,
    /// D-316 — rounded once from the text, as the runtime's
    /// `ember_parse_f16_value` does.
    F16,
    F32,
    F64,
    Bool,
    Char,
}

/// `[LEX-19]` — a format spec in Python's mini-language,
/// `[[fill]align][sign][#][0][width][,|_][.precision][type]`, parsed. A
/// `kind` of `?` is `Debug` (`{x!r}`).
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
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
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum MatchArmBody {
    Block(Block),
    Expr(Expr),
}

/// A pattern with its type resolved and its bindings already given locals.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Pattern {
    pub ty: Ty,
    pub kind: PatternKind,
    pub span: Span,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum PatternKind {
    /// `_`, and anything that always matches.
    Wild,
    /// Binds whatever it matched to a local, then keeps testing `sub`.
    /// `by_ref` is `Some(mutable)` when the local is a reference to the
    /// matched place rather than a copy or move of it (`[GRM-13]`).
    Bind { local: LocalId, sub: Option<Box<Pattern>>, by_ref: Option<bool> },
    /// An integer, `bool` or `char` literal, already narrowed to `ty`.
    Int(i128),
    /// III.6 — a string literal against text (`str` or `String`): equal when the
    /// lengths are and then the bytes.
    Str(String),
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

/// `[SOA-1]`..`[SOA-7]` — the operations on an `SoA[T]`. Arguments are listed with each; `ps` is
/// the `SoA` (borrowed as the operation needs), `i` an `int`.
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum SoAOp {
    /// `SoA[T].with_capacity(n)`: `[n]`.
    WithCapacity,
    /// `ps.f`: `[borrow of the column's pointer field, the length]`; gives `Span[F]` or `MutSpan[F]`.
    Column { mutable: bool },
    /// `ps[i]`, which only a field access (`ps[i].f`) takes apart: `[ps, i]`.
    Element { mutable: bool },
    /// `[ref mut ps, value]`.
    Push,
    /// `[ref mut ps]`, the length known above zero: moves the last element out.
    TakeLast,
    /// `[ref ps]`: the length, an `int`.
    Len,
    /// `[ref mut ps]`.
    Clear,
    /// `[ref mut ps, n]`.
    Reserve,
    /// `[ref mut ps, i]`: moves element `i` out, the last into its place; panics out of range.
    SwapRemove,
    /// `ps.iter()`/`ps.iter_mut()`, which only a `for` header takes: `[ps]`.
    Iter { mutable: bool },
    /// std's own: `[ref mut ps, i, j]`, both in range: elements `i` and `j` change places.
    Swap,
    /// std's own: `[ref mut ps, n]`: the elements from `n` on are dropped.
    Truncate,
    /// std's own, for `retain`: `[ref mut ps, from, to]`: element `from` moved over slot `to`,
    /// which holds nothing (moved out or dropped).
    MoveSlot,
    /// std's own, for `retain`: `[ref mut ps, i]`: element `i`'s fields dropped, its slot left empty.
    DropAt,
    /// std's own, for `retain`: `[ref mut ps, n]`: the length set, the slots past it already empty.
    SetLen,
    /// `[SOA-4]` — `ArenaSoA[T].with_capacity(arena, n)`: `[ref arena, n]`; the block is taken
    /// from the arena once.
    WithCapacityIn,
    /// `[SOA-4]` — `[ref mut ps, value]`: `true` when it was full and nothing changed.
    TryPush,
}

/// `[SIMD-1]`, `[SIMD-4]`, `[SIMD-8]`, `[SIMD-9]` — an operation of a `std.simd` vector or mask
/// type. The arguments are listed with each; `v` is a vector (or mask) by value, `s` a `Span` or
/// `MutSpan` of its lane type, `idx` an integer vector of as many lanes.
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum SimdOp {
    /// `V.splat(x)`: `[x]`.
    Splat,
    /// `V.load(s)`, `V.load_unchecked(s)`: `[s]`.
    Load { checked: bool },
    /// `v.store(s)`, `v.store_unchecked(s)`: `[v, s]`.
    Store { checked: bool },
    /// `V.gather(s, idx)`, `V.gather_unchecked(s, idx)`: `[s, idx]`; `index` is `idx`'s type.
    Gather { checked: bool, index: Ty },
    /// `v.scatter(s, idx)`, `v.scatter_unchecked(s, idx)`: `[v, s, idx]`.
    Scatter { checked: bool, index: Ty },
    /// A lane-wise `+`, `-`, `*`, `/`, `&`, `|` or `^`: `[a, b]`; integer lanes under `policy`
    /// (`[SIMD-8]`: the function's, or `Wrap` for `wrapping_*`).
    Binary { op: BinOp, policy: OverflowPolicy },
    /// Unary `-`: `[v]`.
    Neg { policy: OverflowPolicy },
    /// A lane-wise comparison into the mask type: `[a, b]`.
    Compare { op: BinOp },
    /// `v.reduce_add()`: `[v]`, lane 0 first, then each next one.
    ReduceAdd { policy: OverflowPolicy },
    /// `v.reduce_min()`: `[v]`.
    ReduceMin,
    /// `v.reduce_max()`: `[v]`.
    ReduceMax,
    /// `v.shuffle(lanes)`: `[v, lanes]`, `lanes` a constant array of lane numbers.
    Shuffle,
    /// `v.fma(b, c)`: `[v, b, c]`, `v * b + c` rounded once.
    Fma,
    /// `v.min(w)`, `v.max(w)`: `[v, w]`.
    Min,
    Max,
    /// `v.abs()`: `[v]`; an integer lane's `MIN` under `policy`.
    Abs { policy: OverflowPolicy },
    /// `v.sqrt()`, `v.rsqrt_approx()`, `v.rcp_approx()`: `[v]`.
    Sqrt,
    RsqrtApprox,
    RcpApprox,
    /// A mask's `any()`, `all()`: `[m]`.
    Any,
    All,
    /// `m.select(a, b)`: `[m, a, b]`; the operation is named for `a`'s vector type.
    Select,
    /// `M.splat(b)`: `[b]`, a mask with every lane set or clear.
    MaskSplat,
}

impl SimdOp {
    pub fn name(self) -> &'static str {
        match self {
            SimdOp::Splat | SimdOp::MaskSplat => "splat",
            SimdOp::Load { checked: true } => "load",
            SimdOp::Load { checked: false } => "load_unchecked",
            SimdOp::Store { checked: true } => "store",
            SimdOp::Store { checked: false } => "store_unchecked",
            SimdOp::Gather { checked: true, .. } => "gather",
            SimdOp::Gather { checked: false, .. } => "gather_unchecked",
            SimdOp::Scatter { checked: true, .. } => "scatter",
            SimdOp::Scatter { checked: false, .. } => "scatter_unchecked",
            SimdOp::Binary { .. } => "operator",
            SimdOp::Neg { .. } => "negation",
            SimdOp::Compare { .. } => "comparison",
            SimdOp::ReduceAdd { .. } => "reduce_add",
            SimdOp::ReduceMin => "reduce_min",
            SimdOp::ReduceMax => "reduce_max",
            SimdOp::Shuffle => "shuffle",
            SimdOp::Fma => "fma",
            SimdOp::Min => "min",
            SimdOp::Max => "max",
            SimdOp::Abs { .. } => "abs",
            SimdOp::Sqrt => "sqrt",
            SimdOp::RsqrtApprox => "rsqrt_approx",
            SimdOp::RcpApprox => "rcp_approx",
            SimdOp::Any => "any",
            SimdOp::All => "all",
            SimdOp::Select => "select",
        }
    }
}

impl SoAOp {
    pub fn name(self) -> &'static str {
        match self {
            SoAOp::WithCapacity => "with_capacity",
            SoAOp::Column { .. } => "column",
            SoAOp::Element { .. } => "index",
            SoAOp::Push => "push",
            SoAOp::TakeLast => "pop",
            SoAOp::Len => "len",
            SoAOp::Clear => "clear",
            SoAOp::Reserve => "reserve",
            SoAOp::SwapRemove => "swap_remove",
            SoAOp::Iter { mutable } => if mutable { "iter_mut" } else { "iter" },
            SoAOp::Swap => "swap",
            SoAOp::Truncate => "truncate",
            SoAOp::MoveSlot => "move_slot",
            SoAOp::DropAt => "drop_at",
            SoAOp::SetLen => "set_len",
            SoAOp::WithCapacityIn => "with_capacity",
            SoAOp::TryPush => "push",
        }
    }
}

/// `[THR-14]` — the atomic operations the C compilers give, which `Atomic[T]`'s methods are built
/// from. `FetchAdd` and `FetchSub` wrap.
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum AtomicOp {
    Load,
    Store,
    Swap,
    CompareExchange,
    FetchAdd,
    FetchSub,
    FetchAnd,
    FetchOr,
    FetchXor,
    /// `[THR-9]` — `std` only, on a 32-bit `Atomic`: sleep while the value is the operand (or
    /// until woken; callers check again), and wake one or every sleeper.
    Wait,
    WakeOne,
    WakeAll,
}

impl AtomicOp {
    pub fn name(self) -> &'static str {
        match self {
            AtomicOp::Load => "load",
            AtomicOp::Store => "store",
            AtomicOp::Swap => "swap",
            AtomicOp::CompareExchange => "compare_exchange",
            AtomicOp::FetchAdd => "wrapping_fetch_add",
            AtomicOp::FetchSub => "wrapping_fetch_sub",
            AtomicOp::FetchAnd => "fetch_and",
            AtomicOp::FetchOr => "fetch_or",
            AtomicOp::FetchXor => "fetch_xor",
            AtomicOp::Wait => "wait",
            AtomicOp::WakeOne => "wake_one",
            AtomicOp::WakeAll => "wake_all",
        }
    }

    /// `[THR-14]` — the orders it does not support (`E7006`): a load has no release half, a store
    /// no acquire half.
    pub fn refuses(self, order: MemoryOrder) -> bool {
        match self {
            AtomicOp::Load => matches!(order, MemoryOrder::Release | MemoryOrder::AcqRel),
            AtomicOp::Store => matches!(order, MemoryOrder::Acquire | MemoryOrder::AcqRel),
            _ => false,
        }
    }
}

/// `[THR-14]` — `std.sync.MemoryOrder`'s variants, in their declared order.
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum MemoryOrder {
    Relaxed,
    Acquire,
    Release,
    AcqRel,
    SeqCst,
}

impl MemoryOrder {
    pub const ALL: [MemoryOrder; 5] =
        [MemoryOrder::Relaxed, MemoryOrder::Acquire, MemoryOrder::Release, MemoryOrder::AcqRel, MemoryOrder::SeqCst];

    pub fn name(self) -> &'static str {
        match self {
            MemoryOrder::Relaxed => "Relaxed",
            MemoryOrder::Acquire => "Acquire",
            MemoryOrder::Release => "Release",
            MemoryOrder::AcqRel => "AcqRel",
            MemoryOrder::SeqCst => "SeqCst",
        }
    }

    /// A compare-exchange that fails only loads: its order without the release half.
    pub fn on_failure(self) -> MemoryOrder {
        match self {
            MemoryOrder::AcqRel => MemoryOrder::Acquire,
            MemoryOrder::Release => MemoryOrder::Relaxed,
            order => order,
        }
    }
}

/// `[STD-20]` (ODR-039) — the integer operators the `checked_`,
/// `wrapping_`, `saturating_` and `overflowing_` methods exist for, named as
/// their operator interfaces' methods are (`floordiv` is `//`, `rem` is `%`).
/// `pow` is built from `Mul`.
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
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
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Hash, Debug)]
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
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
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
/// `[DSJ-1]`, `[DSJ-5]`, `[DSJ-7]` — what `assert_disjoint` and its siblings
/// do with two views that overlap.
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum DisjointMode {
    /// `assert_disjoint`: `Ok((a, b))`, or `Err((a, b))` when they overlap.
    Result,
    /// `assert_disjoint_or_panic`: `(a, b)`, or a panic.
    OrPanic,
    /// `unsafe assume_disjoint`: `(a, b)`, no comparison.
    Assumed,
    /// `[DSJ-9]` `assert_disjoint_all`: two to eight views compared pairwise,
    /// `Ok` or `Err` as `assert_disjoint`.
    All,
}

/// `[STD-22]`, `[STD-24]`, `[STD-25]` — a standard function whose body is the runtime's: a call to
/// the declaration at `path` is a call to the runtime's function `runtime` (with its prefix). The
/// declaration's own body is a staging body that never runs; its signature is the call's.
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum RuntimeFn {
    Entropy,
    ProcessExit,
    ProcessAbort,
    ProcessArgs,
    ProcessArgsOs,
    EnvVar,
    FsRead,
    FsWrite,
    FsExists,
    FsMetadata,
    FsRemove,
    FsCreateDirAll,
    FsReadDir,
    FileOpen,
    FileRead,
    FileWrite,
    FileFlush,
    FileClose,
    StdinReadLine,
    IoErrorKind,
    IoErrorMessage,
    TimeNow,
    TimeSleep,
    CharClass,
    /// `[ALC-2]` (ODR-175) — `std.mem.Global`'s allocation, as the runtime allocates: null when
    /// there is no memory.
    GlobalAllocate,
    GlobalDeallocate,
    GlobalReallocate,
    /// `[ALC-2]` (ODR-239) — `std.mem.System`'s: the platform's allocator, which a configured
    /// global allocator never reaches.
    SystemAllocate,
    SystemDeallocate,
    SystemReallocate,
}

impl RuntimeFn {
    pub const ALL: [RuntimeFn; 30] = [
        RuntimeFn::Entropy,
        RuntimeFn::ProcessExit,
        RuntimeFn::ProcessAbort,
        RuntimeFn::ProcessArgs,
        RuntimeFn::ProcessArgsOs,
        RuntimeFn::EnvVar,
        RuntimeFn::FsRead,
        RuntimeFn::FsWrite,
        RuntimeFn::FsExists,
        RuntimeFn::FsMetadata,
        RuntimeFn::FsRemove,
        RuntimeFn::FsCreateDirAll,
        RuntimeFn::FsReadDir,
        RuntimeFn::FileOpen,
        RuntimeFn::FileRead,
        RuntimeFn::FileWrite,
        RuntimeFn::FileFlush,
        RuntimeFn::FileClose,
        RuntimeFn::StdinReadLine,
        RuntimeFn::IoErrorKind,
        RuntimeFn::IoErrorMessage,
        RuntimeFn::TimeNow,
        RuntimeFn::TimeSleep,
        RuntimeFn::CharClass,
        RuntimeFn::GlobalAllocate,
        RuntimeFn::GlobalDeallocate,
        RuntimeFn::GlobalReallocate,
        RuntimeFn::SystemAllocate,
        RuntimeFn::SystemDeallocate,
        RuntimeFn::SystemReallocate,
    ];

    /// The standard declaration a call names.
    pub fn path(self) -> &'static str {
        match self {
            RuntimeFn::Entropy => "std.random.entropy",
            RuntimeFn::ProcessExit => "std.process.exit_now",
            RuntimeFn::ProcessAbort => "std.process.abort_now",
            RuntimeFn::ProcessArgs => "std.process.args_utf8",
            RuntimeFn::ProcessArgsOs => "std.process.args_bytes",
            RuntimeFn::EnvVar => "std.env.lookup",
            RuntimeFn::FsRead => "std.fs.read_raw",
            RuntimeFn::FsWrite => "std.fs.write_raw",
            RuntimeFn::FsExists => "std.fs.exists_raw",
            RuntimeFn::FsMetadata => "std.fs.metadata_raw",
            RuntimeFn::FsRemove => "std.fs.remove_raw",
            RuntimeFn::FsCreateDirAll => "std.fs.create_dir_all_raw",
            RuntimeFn::FsReadDir => "std.fs.read_dir_raw",
            RuntimeFn::FileOpen => "std.fs.file_open",
            RuntimeFn::FileRead => "std.fs.file_read",
            RuntimeFn::FileWrite => "std.fs.file_write",
            RuntimeFn::FileFlush => "std.fs.file_flush",
            RuntimeFn::FileClose => "std.fs.file_close",
            RuntimeFn::StdinReadLine => "std.io.stdin_read_line",
            RuntimeFn::IoErrorKind => "std.io.error_kind",
            RuntimeFn::IoErrorMessage => "std.io.error_message",
            RuntimeFn::TimeNow => "std.time.monotonic_now",
            RuntimeFn::TimeSleep => "std.time.sleep_nanos",
            RuntimeFn::CharClass => "std.string.char_class",
            RuntimeFn::GlobalAllocate => "std.mem.global_allocate",
            RuntimeFn::GlobalDeallocate => "std.mem.global_deallocate",
            RuntimeFn::GlobalReallocate => "std.mem.global_reallocate",
            RuntimeFn::SystemAllocate => "std.mem.system_allocate",
            RuntimeFn::SystemDeallocate => "std.mem.system_deallocate",
            RuntimeFn::SystemReallocate => "std.mem.system_reallocate",
        }
    }

    /// The runtime function, without its prefix.
    pub fn runtime(self) -> &'static str {
        match self {
            RuntimeFn::Entropy => "entropy",
            RuntimeFn::ProcessExit => "process_exit",
            RuntimeFn::ProcessAbort => "process_abort",
            RuntimeFn::ProcessArgs => "process_args",
            RuntimeFn::ProcessArgsOs => "process_args_os",
            RuntimeFn::EnvVar => "env_var",
            RuntimeFn::FsRead => "fs_read",
            RuntimeFn::FsWrite => "fs_write",
            RuntimeFn::FsExists => "fs_exists",
            RuntimeFn::FsMetadata => "fs_metadata",
            RuntimeFn::FsRemove => "fs_remove",
            RuntimeFn::FsCreateDirAll => "fs_create_dir_all",
            RuntimeFn::FsReadDir => "fs_read_dir",
            RuntimeFn::FileOpen => "file_open",
            RuntimeFn::FileRead => "file_read",
            RuntimeFn::FileWrite => "file_write",
            RuntimeFn::FileFlush => "file_flush",
            RuntimeFn::FileClose => "file_close",
            RuntimeFn::StdinReadLine => "stdin_read_line",
            RuntimeFn::IoErrorKind => "io_error_kind",
            RuntimeFn::IoErrorMessage => "io_error_message",
            RuntimeFn::TimeNow => "time_now",
            RuntimeFn::TimeSleep => "time_sleep",
            RuntimeFn::CharClass => "char_class",
            RuntimeFn::GlobalAllocate => "try_alloc",
            RuntimeFn::GlobalDeallocate => "free",
            RuntimeFn::GlobalReallocate => "try_realloc",
            RuntimeFn::SystemAllocate => "system_alloc",
            RuntimeFn::SystemDeallocate => "system_free",
            RuntimeFn::SystemReallocate => "system_realloc",
        }
    }

    /// `[EFF-1]` — what a call does, and how a contract violation names it.
    pub fn effects(self) -> (Effects, &'static str) {
        match self {
            // `[DET-2]` — the system random source.
            RuntimeFn::Entropy => (Effects::NONDET, "the system random source"),
            RuntimeFn::ProcessExit => (Effects::IO, "`process.exit`"),
            RuntimeFn::ProcessAbort => (Effects::IO, "`process.abort`"),
            RuntimeFn::ProcessArgs | RuntimeFn::ProcessArgsOs => (Effects::ALLOC, "the command line"),
            RuntimeFn::EnvVar => (Effects(Effects::IO.0 | Effects::ALLOC.0), "`env.var`"),
            // `[STD-22]` — every operation on the outside world carries `Io`.
            RuntimeFn::FsRead => (Effects(Effects::IO.0 | Effects::ALLOC.0), "`std.fs`"),
            RuntimeFn::FsWrite => (Effects(Effects::IO.0 | Effects::ALLOC.0), "`std.fs`"),
            RuntimeFn::FsExists => (Effects(Effects::IO.0 | Effects::ALLOC.0), "`std.fs`"),
            RuntimeFn::FsMetadata => (Effects(Effects::IO.0 | Effects::ALLOC.0), "`std.fs`"),
            RuntimeFn::FsRemove => (Effects(Effects::IO.0 | Effects::ALLOC.0), "`std.fs`"),
            RuntimeFn::FsCreateDirAll => (Effects(Effects::IO.0 | Effects::ALLOC.0), "`std.fs`"),
            RuntimeFn::FsReadDir => (Effects(Effects::IO.0 | Effects::ALLOC.0), "`std.fs`"),
            RuntimeFn::FileOpen => (Effects(Effects::IO.0 | Effects::ALLOC.0), "a `File`"),
            RuntimeFn::FileRead => (Effects(Effects::IO.0 | Effects::ALLOC.0), "a `File`"),
            RuntimeFn::FileWrite => (Effects(Effects::IO.0 | Effects::ALLOC.0), "a `File`"),
            RuntimeFn::FileFlush => (Effects(Effects::IO.0 | Effects::ALLOC.0), "a `File`"),
            RuntimeFn::FileClose => (Effects(Effects::IO.0 | Effects::ALLOC.0), "a `File`"),
            RuntimeFn::StdinReadLine => (Effects(Effects::IO.0 | Effects::ALLOC.0 | Effects::BLOCK.0), "standard input"),
            RuntimeFn::IoErrorKind => (Effects::NONE, ""),
            RuntimeFn::IoErrorMessage => (Effects::ALLOC, "an `io.Error`'s message"),
            // `[STD-23]`, `[DET-2]` — the monotonic clock is `Nondet`; sleeping waits.
            RuntimeFn::TimeNow => (Effects::NONDET, "the monotonic clock"),
            RuntimeFn::TimeSleep => (Effects::BLOCK, "`time.sleep`"),
            // ODR-183 — a lookup in constant tables.
            RuntimeFn::CharClass => (Effects::NONE, ""),
            RuntimeFn::GlobalAllocate | RuntimeFn::GlobalDeallocate | RuntimeFn::GlobalReallocate => (Effects::ALLOC, "the global allocator"),
            RuntimeFn::SystemAllocate | RuntimeFn::SystemDeallocate | RuntimeFn::SystemReallocate => (Effects::ALLOC, "the platform's allocator"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Builtin {
    /// A standard function whose body is the runtime's (`RuntimeFn`).
    Runtime(RuntimeFn),
    /// `[DSJ-1]` — establish that two views do not overlap (`result` is the
    /// call's type).
    AssertDisjoint { mode: DisjointMode, result: Ty },
    /// `[DSJ-1]`'s two comparisons: whether views of types `a` and `b`
    /// overlap. Empty views overlap nothing.
    ViewsOverlap { a: Ty, b: Ty },
    /// `[FFI-10]`, `[STA-1]` — a reference to a hand-declared C global at its
    /// linker symbol: `ref T`, or `ref mut T` for a `static mut`. Every read
    /// and write of the global goes through it, so the global is a place like
    /// any other (a field, an element of an array in it, a whole non-`Copy`
    /// record) and C's storage is what changes. `immutable` is
    /// `@ffi(immutable)` (ODR-082): the C declaration is `extern const T`.
    /// `thunk` (`[FFI-41]`, ODR-161): `symbol` is a C++ variable's thunk, a
    /// function giving its address, which every use calls.
    ForeignStatic { symbol: Symbol, ty: Ty, immutable: bool, thunk: bool },
    /// `[FFI-36a]`, `[FFI-36b]` — record the foreign object at the pointer
    /// (the one argument, also the result) as one Ember owns, with where;
    /// a second owner of a live one panics naming both places.
    ForeignAdopt,
    /// `[FFI-36b]`, `[FFI-33b]` — a `ForeignBox`'s end, or its object given
    /// to C: out of the record. Arguments: the pointer, the creator thread
    /// (0 for none), checked in the debug profile.
    ForeignRelease,
    /// `[FFI-33b]` — the number of the thread running it, recorded by a box
    /// whose contract says `threads(creator)`.
    ForeignThread,
    /// `[FFI-36a]` — the foreign function `symbol` (the argument is its
    /// value) as a box's destructor, of type `fn_ty` (`extern "C" fn(*mut
    /// T)`): itself when it returns nothing (`ret` void), else a C thunk that
    /// calls it and drops its result.
    ForeignDestructor { symbol: Symbol, pointee: Ty, ret: Ty, fn_ty: Ty },
    /// `[FFI-23]` — `Retained.pin`'s mark: a class object's header gets its
    /// pinned flag; a `Box` needs none. The arguments are the value pinned and
    /// its `on_relocate` callback (`[HR-20]`, ODR-194): given one, a class
    /// object is recorded with it instead of pinned. The result is the
    /// record's number (`0`: none), which `RetainedUnpin` takes.
    RetainedPin,
    /// `[HR-20]` (ODR-194) — a `Retained` token's end: its `on_relocate`
    /// record goes. The argument is the number `RetainedPin` gave.
    RetainedUnpin,
    /// `[FFI-23]` — the address of the object a class handle or a `Box`
    /// holds, as `*mut u8`.
    RetainedAddress,
    /// `[FFI-39]` — in a module `import cpp` wrote: a class object's address
    /// (the argument, a handle), which a trampoline keeps without a count.
    CppObjectAddress,
    /// `[UNS-5]`, `[FFI-8]` — a volatile read of the `T` a raw pointer (the
    /// argument) points to: it happens, in order, as written.
    VolatileRead,
    /// `[UNS-5]`, `[FFI-8]` — a volatile write of a `T` (the second argument)
    /// through a raw pointer (the first).
    VolatileWrite,
    /// `[FFI-8]` — a C union's value with one field set (the argument), the
    /// rest of its storage zero: `Number(i = 5)`.
    UnionLit { union: StructId, field: usize },
    /// `[FFI-39]` (ODR-221) — in a module `import cpp` wrote: whether the
    /// object's own class (the argument, a handle of `owner` or below it)
    /// overrides `owner`'s virtual method in `slot`, so that its trampoline
    /// passes on exactly the virtuals the object's class overrides.
    CppOverrides { owner: ClassId, slot: usize },
    /// `[FFI-39]` — the handle of the class object at an address a trampoline
    /// kept (the argument): a new count of it, or, with `adopt`, the count C++
    /// held, which must be the object's only one (`[FFI-17d]`).
    CppObjectHandle { adopt: bool },
    /// `[FFI-17d]` — a count of a class object (the argument, a handle) for
    /// C++ to hold, its header pinned, as `Retained.pin` does.
    CppObjectGive,
    /// `[FFI-17e]` — a view of a C++ container's elements: the owner (a
    /// reference, which the view borrows), their address and their count.
    CppSpan { mutable: bool },
    /// `[FFI-17e]` — the object a C++ smart pointer holds, as a reference:
    /// the owner (a reference, which it borrows) and the object's address.
    CppPointee,
    /// `[STA-3]` — a reference to the one value of a `static` whose
    /// initialiser runs on first access: `init`, a function of no
    /// arguments, the first time any thread asks. `name` is the static's, for
    /// the initialisation-cycle panic.
    LazyStatic { name: Symbol, init: Symbol, ty: Ty },
    /// `[CT-5]`, `[STA-3]` — a reference to the one value of a `static` the
    /// compile-time evaluator computed, placed in the image as constant data
    /// (`TypeTable::constant_statics`): no initialisation runs.
    ConstStatic { name: Symbol, ty: Ty },
    /// `[CT-2]` — `comptime.read_file(path)`: the file's text, read while
    /// compiling from inside the package directory (`[path]`).
    ComptimeReadFile,
    /// `[CT-2]` — `comptime.env(name)`: the build environment's variable, an
    /// `option` (`Option[String]`), read while compiling (`[name]`).
    ComptimeEnv { option: Ty },
    /// `println(x)` for a scalar or `str`.
    Println,
    /// `[STD-9]` — `eprintln(x)` and `eprint(x)`: the same, to standard error.
    EPrintln,
    EPrint,
    /// `[STD-9]` (D-900) — one piece of a print statement, as MIR writes it: the first takes its
    /// stream (standard error with `stderr`), the last lets it go, and each is written with it
    /// held, so another thread's print is written before or after the statement, never into it.
    PrintPiece { stderr: bool, first: bool, last: bool },
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
    /// `[TXT-10]` — find the first separator and return its two borrowed
    /// sides, or `None`. Successful search already proves their bounds and
    /// UTF-8 boundaries. The result borrows only the searched text.
    StrSplitOnce { none: usize, some: usize },
    StrCount,
    StrReplace,
    StrRepeat,
    StrTrimStart,
    StrTrimEnd,
    StrSliceOk,
    StrToUpper,
    /// `[TXT-2]` — validate bytes before exposing them as text.
    Utf8Valid,
    /// `[TXT-5]` — view already validated bytes as text without copying.
    SpanToStr,
    /// `[FFI-15]` — view the bytes before a C string's terminator.
    CStrToSpan,
    /// `[FFI-15]` — borrow an owned C string's terminated buffer.
    CStringAsCStr,
    /// `[TXT-10]` (ODR-029) — `parse`: the status (0 when the text is a
    /// literal of the kind that fits) and, once it is 0, the value.
    ParseStatus { kind: ParseKind },
    ParseValue { kind: ParseKind },
    StrToLower,
    /// `[CTL-1]` — the `char` that starts at byte `at` of a `str`, moving
    /// `at` past it through a mutable reference. Std's iterator state is
    /// `int`; a native text loop's hidden cursor is `usize`, with its public
    /// byte offset copied as `int`. Both keep `at` on a character boundary
    /// before the end and dispatch to a helper with the same pointer type.
    StrCharNext,
    /// D-498 (ADR-155) — `StrCharNext` at a cursor whose lead byte, the third
    /// argument, is already read and 0x80 or above. Only the late ASCII step
    /// (`ascii_runs`) makes it, for MSVC's C, past every analysis.
    StrCharNextWide,
    /// `[FN-8]` — the command line as `main`'s `args`, an `Array[String]`.
    /// Only the compiler-written entry (`main_entry`) calls it.
    ProcessArgs,
    /// `[LEX-19]` — append a value to an f-string's buffer as its spec says.
    FormatWith(FormatSpec),
    /// `[STD-18]` — `format_to`'s text: a `String` over the bytes of the `MutSpan[u8]` argument,
    /// which never grows (a write that would not fit is refused and remembered), registered with
    /// the runtime until `FmtSinkClose`.
    FmtSinkOpen,
    /// `[STD-18]` — the end of `FmtSinkOpen`'s text, read through a `ref` (the text is then
    /// forgotten: its bytes are the caller's): how many bytes were written, or `usize.MAX` when a
    /// write did not fit.
    FmtSinkClose,
    /// `[STD-8b]` — `needle in text`, for a `str` needle and a `char` one.
    StrContains,
    StrContainsChar,
    /// `[HASH-2]` — word `which` of the process's hash key, drawn once from
    /// the operating system: `std.collections.process_key`.
    ProcessKey,
    /// `[RFL-2]`, `[RT-3]` — `h.type_info()` on a `@reflect` class handle: the description of the
    /// object's own class, which its type information reaches. `result` is `ref TypeInfo`.
    ClassTypeInfo { result: Ty },
    /// `[HR-33]` — `std.hot.checkpoint()`: where the program lets a waiting reload apply.
    HotCheckpoint,
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
    /// Never written: the room a counted loop's pushes will need, asked for
    /// before it (`reserve_pushed_lists_all`). Its operands are the list,
    /// the loop's counter and its limit; the turns are `limit - counter`
    /// (one more when `inclusive`), each pushing `per_turn` at least. A hint
    /// that never fails and changes nothing but the capacity.
    ArrayReserveHint { per_turn: u32, inclusive: bool },
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
    /// `[STR-5]` — clone a tuple, Option or Result into an existing
    /// destination from a borrowed source. The backend's per-type helper
    /// avoids one large temporary for every nesting level (D-360).
    CloneParts { ty: Ty },
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
    Slice {
        text: bool,
        /// The view is known not to be empty (`[OPT-2]`'s unchecked copy of a
        /// loop whose entry test proved the slice's end at least 1), so its
        /// pointer is never null and the part's start is the pointer plus
        /// `lo` with no test for an empty view's null pointer.
        nonempty: bool,
    },
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
    /// `s.push(c)`, appending one Unicode scalar encoded as UTF-8.
    StringPushChar,
    /// `[TXT-11]` (ODR-098) — UTF-8 mutation by byte offset, preserving
    /// character boundaries and the owned buffer's capacity when shrinking.
    StringInsert,
    StringRemove,
    StringTruncate,
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
    /// Test a raw pointer's nullness without dereferencing it.
    PtrIsNull,
    /// `[UNS-1]` — `p.add(n)`: the pointer `n` elements further on, which needs `unsafe`.
    PtrAdd,
    /// `size_of[T]() -> usize`, which needs no `unsafe`.
    SizeOf,
    /// `align_of[T]() -> usize`, folded from the canonical target layout.
    AlignOf,
    /// `[ECS-4]` (ODR-218, ODR-255) — `std.ecs`'s `type_is_send[T]()` and `type_is_sync[T]()`:
    /// whether `T` is `Send` (`Sync`), a constant each instance decides (`[THR-8]`, `[THR-9]`).
    TypeIsSend,
    TypeIsSync,
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
    /// `[OWN-6]` — `mem.drop(x)`: ends `x`'s ownership now. For a `Copy`
    /// value that needs drop (a class handle), passing it `owned` would pass
    /// a retained copy and end nothing (D-347, ODR-063), so the place is
    /// moved. MIR lowers it; it never reaches the backend.
    MemDrop { elem: Ty },
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
    /// `[STD-19]` (ODR-091) — take a Span element iterator's last item left:
    /// the source view's length goes down by one and the item is there.
    SpanIterNextBack { elem: Ty, mutable: bool },
    /// `[STD-19]` (ODR-091) — how many items a Span element iterator has
    /// left, as an `int`.
    SpanIterLen,
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
    /// `[RNG-3]` — whether a float `v` is in the range, NaN never: the test
    /// `RangeChecked` makes in a `@fastmath` or `@fp(contract)` function,
    /// which the runtime does strictly (`[CG-C-11]`, ODR-090).
    RangeContains(RangeId),
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
    /// `[THR-3]` — `m.lock() -> MutexGuard[T]`: waits for the lock and takes it. In MIR, the
    /// runtime's `ember_mutex_lock` on the lock word.
    MutexLock,
    /// `[THR-3]` — `m.try_lock() -> Option[MutexGuard[T]]`: never waits. In MIR,
    /// `ember_mutex_try_lock`, which gives whether it took the lock.
    MutexTryLock,
    /// `[THR-3]` — `rw.read()` (`write` false) or `rw.write()`: waits and takes the lock, giving
    /// a guard. In MIR, `ember_rwlock_acquire` on the lock word.
    RwLockAcquire { write: bool },
    /// `[THR-3]` — `rw.try_read()` or `rw.try_write()`: never waits. In MIR,
    /// `ember_rwlock_try_acquire`, which gives whether it took the lock.
    RwLockTryAcquire { write: bool },
    /// `[THR-14]` — one atomic operation on an `Atomic[T]`'s value. The first argument borrows the
    /// `Atomic`, the operands follow, then — when `order` is `None` — the `MemoryOrder` value,
    /// which was not a constant. A compare-exchange's operands are `ref mut` the expected value
    /// (replaced by the value found when it fails) and the new one; it gives whether it stored.
    Atomic { op: AtomicOp, order: Option<MemoryOrder> },
    /// `[THR-10]` — `thread.spawn(f)`: the argument is the owned callable value the new thread
    /// runs; it gives the `JoinHandle[R]`.
    ThreadSpawn,
    /// `[THR-10]` — `h.join()`: the argument is the handle, moved; it gives what the thread's
    /// function returned.
    ThreadJoin,
    /// `[THR-5]` — `thread.scope()`: gives the `Scope`, whose end joins its threads.
    ThreadScope,
    /// `[THR-11]` — `scope.spawn(f)`: the arguments are a borrow of the `Scope` and either the
    /// closure itself (`closure`: its body; the closure's borrows flow into the `Scope`, so they
    /// last until it ends) or, for a function, its owned callable value. Gives the
    /// `ScopedJoinHandle[R]`.
    ScopeSpawn { closure: Option<DefId> },
    /// `[THR-11]` — `h.join()` on a `ScopedJoinHandle`: the handle, moved; gives the result.
    ScopedJoin,
    /// `[JOB-2]` — `jobs.scope()`: gives the `JobScope`, whose end waits for its jobs.
    JobScope,
    /// `[JOB-2]`, `[JOB-3]` — `s.submit(f)` and `s.submit_after(deps, f)` (`after`): the
    /// arguments are a borrow of the `JobScope`, the `deps` (a `Span[ScopedJob]`) when `after`,
    /// and either the closure itself (`closure`: its body; its borrows flow into the scope) or,
    /// for a function, its owned callable value. Gives the `ScopedJob`.
    JobScopeSubmit { closure: Option<DefId>, after: bool },
    /// `[JOB-1]`, `[JOB-2]` — `jobs.submit(f)`: the closure itself (`closure`: its body; it owns
    /// what it captured) or an owned callable value. Gives the `JobHandle`.
    JobSubmit { closure: Option<DefId> },
    /// `[JOB-2]` — `jobs.wait(h)`: the `JobHandle`.
    JobWait,
    /// `[PAR-1]`, `[PAR-3]` — a `@parallel` loop: `[start, end, chunk, body]`, and with reductions
    /// `ref mut` the running results last. `body` is the closure over one chunk, `fn(lo, hi)`,
    /// giving its reductions (`closure`, its function; `None` when it captures nothing and is the
    /// function's value); `combine`, a function folding one chunk's results into the running ones.
    ParallelFor { closure: Option<DefId>, combine: Option<DefId> },
    /// `[SOA-1]`..`[SOA-7]` — an operation on `soa`, an `SoA[T]` type.
    SoA { op: SoAOp, soa: Ty },
    /// `[SIMD-1]` — an operation of `vector`, a `std.simd` vector or mask type (the receiver's,
    /// or for a constructor the result's; for `select`, the selected vectors').
    Simd { op: SimdOp, vector: Ty },
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
    /// `[JOB-5]` — `jobs.local_arena()`: a scope of the running job's arena, which the job system
    /// rewinds when the job ends. Its argument is a mutable reference to a hidden local of the
    /// calling function, the view's anchor, so nothing allocated from it outlives that call.
    JobLocalArena { scoped: Ty },
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
            Builtin::Runtime(function) => function.runtime(),
            Builtin::ForeignStatic { .. } => "foreign_static",
            Builtin::ForeignAdopt => "ffi_adopt",
            Builtin::ForeignRelease => "ffi_release",
            Builtin::ForeignThread => "ffi_thread",
            Builtin::ForeignDestructor { .. } => "ffi_destructor",
            Builtin::RetainedPin => "retained_pin",
            Builtin::RetainedUnpin => "retained_unpin",
            Builtin::RetainedAddress => "retained_address",
            Builtin::CppObjectAddress => "cpp_object_address",
            Builtin::CppOverrides { .. } => "cpp_overrides",
            Builtin::UnionLit { .. } => "union_literal",
            Builtin::VolatileRead => "volatile_read",
            Builtin::VolatileWrite => "volatile_write",
            Builtin::CppObjectHandle { .. } => "cpp_object_handle",
            Builtin::CppObjectGive => "cpp_object_give",
            Builtin::CppSpan { .. } => "cpp_span",
            Builtin::CppPointee => "cpp_pointee",
            Builtin::LazyStatic { .. } => "lazy_static",
            Builtin::ConstStatic { .. } => "constant_static",
            Builtin::ComptimeReadFile => "read_file",
            Builtin::ComptimeEnv { .. } => "env",
            Builtin::Println => "println",
            Builtin::Print => "print",
            Builtin::EPrintln => "eprintln",
            Builtin::EPrint => "eprint",
            Builtin::PrintPiece { .. } => "print_piece",
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
            Builtin::StrSplitOnce { .. } => "split_once",
            Builtin::StrCount => "count",
            Builtin::StrReplace => "replace",
            Builtin::StrRepeat => "repeat",
            Builtin::StrTrimStart => "trim_start",
            Builtin::StrTrimEnd => "trim_end",
            Builtin::StrSliceOk => "slice_ok",
            Builtin::StrToUpper => "to_upper",
            Builtin::Utf8Valid | Builtin::SpanToStr | Builtin::CStrToSpan => "to_str",
            Builtin::CStringAsCStr => "as_cstr",
            Builtin::ParseStatus { .. } | Builtin::ParseValue { .. } => "parse",
            Builtin::StrToLower => "to_lower",
            Builtin::StrCharNext => "next_char",
            Builtin::StrCharNextWide => "next_char_wide",
            Builtin::ProcessArgs => "args",
            Builtin::FormatWith(_) => "format",
            Builtin::StrContains | Builtin::StrContainsChar => "contains",
            Builtin::ProcessKey => "process_key",
            Builtin::ClassTypeInfo { .. } => "type_info",
            Builtin::HotCheckpoint => "checkpoint",
            Builtin::ArraySort => "sort",
            Builtin::ArrayReverse => "reverse",
            Builtin::ArrayClear => "clear",
            Builtin::ArrayPop { .. } => "pop",
            Builtin::ArrayRemove => "remove",
            Builtin::ArrayDrain => "drain",
            Builtin::ArrayInsert => "insert",
            Builtin::ArrayCapacity => "capacity",
            Builtin::ArrayReserve => "reserve",
            Builtin::ArrayReserveHint { .. } => "reserve_hint",
            Builtin::ArrayTruncate => "truncate",
            Builtin::ArraySwapRemove => "swap_remove",
            Builtin::ArraySwap => "swap",
            Builtin::ArrayExtend => "extend",
            Builtin::ArraySorted { .. } => "sorted",
            Builtin::ArrayClone { .. } => "clone",
            Builtin::CloneParts { .. } => "clone",
            Builtin::Input => "input",
            Builtin::ArrayNew => "Array",
            Builtin::ArrayFromLiteral => "Array",
            Builtin::ValueCompare { .. } => "compare",
            Builtin::ClassNew { .. } => "class",
            Builtin::ClassDowncast { forced: true, .. } => "cast as",
            Builtin::ClassDowncast { forced: false, .. } => "cast if",
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
            Builtin::StringPushChar => "push",
            Builtin::StringInsert => "insert",
            Builtin::StringRemove => "remove",
            Builtin::StringTruncate => "truncate",
            Builtin::StringLen => "len",
            Builtin::StringAsStr => "as_str",
            Builtin::Format => "format",
            Builtin::FmtSinkOpen | Builtin::FmtSinkClose => "format_to",
            Builtin::MemAlloc => "alloc",
            Builtin::MemFree => "free",
            Builtin::PtrRead => "read",
            Builtin::PtrWrite => "write",
            Builtin::PtrIsNull => "is_null",
            Builtin::PtrAdd => "add",
            Builtin::SizeOf => "size_of",
            Builtin::AlignOf => "align_of",
            Builtin::TypeIsSend => "type_is_send",
            Builtin::TypeIsSync => "type_is_sync",
            Builtin::MemReplace { .. } => "replace",
            Builtin::MemTake { .. } => "take",
            Builtin::MemSwap { .. } => "swap",
            Builtin::AssertDisjoint { mode: DisjointMode::Result, .. } => "assert_disjoint",
            Builtin::AssertDisjoint { mode: DisjointMode::OrPanic, .. } => "assert_disjoint_or_panic",
            Builtin::AssertDisjoint { mode: DisjointMode::Assumed, .. } => "assume_disjoint",
            Builtin::AssertDisjoint { mode: DisjointMode::All, .. } => "assert_disjoint_all",
            Builtin::ViewsOverlap { .. } => "views_overlap",
            Builtin::MemForget { .. } => "forget",
            Builtin::MemDrop { .. } => "drop",
            // `[RNG-10]`'s construction set. Named as they are
            // written, so a diagnostic quoting one reads as source.
            Builtin::SpanLen => "len",
            Builtin::SpanGet => "get",
            Builtin::SpanGetUnchecked => "get_unchecked",
            Builtin::SpanFrom { mutable } => {
                if mutable { "as_mut_span" } else { "as_span" }
            }
            Builtin::Slice { .. } => "slice",
            Builtin::StrIsCharBoundary => "is_char_boundary",
            Builtin::StrAsBytes => "as_bytes",
            Builtin::SpanSplitAt { .. } => "split_at",
            Builtin::SpanReborrow => "reborrow",
            Builtin::SpanSharedReborrow => "reborrow",
            Builtin::SpanIterNext { .. } => "next",
            Builtin::SpanIterNextBack { .. } => "next_back",
            Builtin::SpanIterLen => "len",
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
            Builtin::RangeContains(_) => "contains",
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
            Builtin::MutexLock => "lock",
            Builtin::MutexTryLock => "try_lock",
            Builtin::RwLockAcquire { write: false } => "read",
            Builtin::RwLockAcquire { write: true } => "write",
            Builtin::RwLockTryAcquire { write: false } => "try_read",
            Builtin::RwLockTryAcquire { write: true } => "try_write",
            Builtin::Atomic { op, .. } => op.name(),
            Builtin::ThreadSpawn => "spawn",
            Builtin::ThreadJoin => "join",
            Builtin::ThreadScope => "scope",
            Builtin::ScopeSpawn { .. } => "spawn",
            Builtin::ScopedJoin => "join",
            Builtin::JobScope => "scope",
            Builtin::JobScopeSubmit { after: false, .. } | Builtin::JobSubmit { .. } => "submit",
            Builtin::JobScopeSubmit { after: true, .. } => "submit_after",
            Builtin::JobWait => "wait",
            Builtin::ParallelFor { .. } => "parallel for",
            Builtin::SoA { op, .. } => op.name(),
            Builtin::Simd { op, .. } => op.name(),
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
            Builtin::JobLocalArena { .. } => "local_arena",
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
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
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

#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
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
        PatternKind::Str(text) => format!("{text:?}"),
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
        ExprKind::CStr(bytes) => format!("c{bytes:?}"),
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
        ExprKind::MakeCallable { closure, env } => {
            format!("callable#{}({})", closure.0, dump_expr(env, function, types))
        }
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
        ExprKind::OverflowScope { policy, expr } => {
            format!("overflow({}; {})", policy.name(), dump_expr(expr, function, types))
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
                    FStringPart::Write { writer, value } => {
                        format!("{{{} by {}}}", dump_expr(value, function, types), writer.0)
                    }
                })
                .collect();
            format!("f({})", inner.join(" "))
        }
        ExprKind::Error => "<error>".to_string(),
    };
    format!("{body}:{}", types.symbol_name(expr.ty))
}
