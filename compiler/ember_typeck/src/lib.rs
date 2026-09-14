//! Type collection and bidirectional type checking: AST to HIR.
//!
//! Spec: Part IV §10 (inference), Part XVIII §4.4 (the algorithm).
//!
//! `[TYP-23]` — inference is local to a function body and bidirectional:
//! expected types flow downward in checking mode, and types flow upward in
//! synthesis mode. Untyped literals (`[LEX-16]`, `[LEX-17]`) are resolved last,
//! defaulting to `i32` and `f32`.
//!
//! Phase 0 covers the scalar and struct subset: no generics, no interfaces, no
//! classes, so there are no interface obligations to solve and no unification
//! variables. The bidirectional walk is the same one the full checker uses;
//! later phases add the obligation solver around it.

mod usefulness;

use std::collections::{HashMap, HashSet};

use ember_ast as ast;
use ember_hir as hir;
use ember_diag::{Diagnostic, Sink, codes};
use ember_hir::{
    BinOp, Block, Builtin, DefId, DestructureBinding, Expr, ExprKind, Function, LocalDecl,
    LocalId, Mode, Param, Program, Stmt, UnOp,
};
use ember_span::{Span, Symbol};
use ember_types::{
    Bound, ClassDef, ClassId, ClassOpenness, CommonTypes, EnumDef, EnumId, FieldDef, FieldVis,
    OverflowPolicy, RangeDef, StructDef, StructId, Ty, TyKind, TypeTable, UintTy, VariantDef,
    FnParam, FnParamMode, int_max,
};

/// One parsed module and where it sits in the package (`[MOD-1]`). The root
/// module's path is empty.
pub struct LoadedModule {
    pub path: Vec<String>,
    pub module: ast::Module,
}

/// The semantically resolved declaration surface a build artifact may expose
/// to importers. Unlike HIR/MIR bodies, this exists for a generic function
/// even when no concrete instantiation is emitted in this compilation.
#[derive(Clone, Debug)]
pub struct CallableDeclaration {
    pub span: Span,
    pub symbol: String,
    pub parameters: Vec<CallableDeclarationParameter>,
    pub result: Ty,
    pub borrows: Option<Vec<usize>>,
    pub generics: Vec<CallableDeclarationGeneric>,
    pub is_unsafe: bool,
    pub abi: Option<String>,
}

/// A declaration type that is already resolved by the checker, or the one
/// narrow source-level identity that has no runtime `Ty` yet: `self` on an
/// uninstantiated generic owner or an interface declaration. The latter is
/// still canonical semantic data, never a diagnostic spelling or generated C
/// symbol.
#[derive(Clone, Debug)]
pub enum CallableDeclarationType {
    Resolved(Ty),
    Canonical(String),
}

#[derive(Clone, Debug)]
pub struct CallableDeclarationParameter {
    pub mode: Mode,
    pub ty: CallableDeclarationType,
}

#[derive(Clone, Debug)]
pub struct CallableDeclarationGeneric {
    /// Interface names after import/name resolution, not the spelling local
    /// to the declaring source file.
    pub bounds: Vec<String>,
    /// The implicit static `Callable`/`CallableOnce` bound that a parameter
    /// written as `fn(A) -> R` introduces under `[CLO-3]`/`[CLO-6]`.
    pub callable: Option<CallableDeclarationCallableBound>,
}

#[derive(Clone, Debug)]
pub struct CallableDeclarationCallableBound {
    pub parameters: Vec<FnParam>,
    pub result: Ty,
    pub once: bool,
    /// `[FN-6b]` — whether this hidden callable boundary creates fresh
    /// invocation-local callback regions.
    pub latebound: bool,
}

/// The checker result separates executable HIR from the source declaration
/// interface. Generic declarations deliberately have no emitted HIR body
/// until instantiated, but their type/bound contract is still importer-visible.
pub struct CheckOutput {
    pub program: Program,
    pub callable_declarations: Vec<CallableDeclaration>,
}

pub fn check(
    modules: &[LoadedModule],
    types: &mut TypeTable,
    common: &CommonTypes,
    sink: &mut Sink,
    default_overflow: OverflowPolicy,
) -> CheckOutput {
    let mut checker = Checker::new(types, common, sink);
    checker.default_overflow = default_overflow;
    checker.prefixes = modules.iter().map(|m| m.path.join(".")).collect();
    checker.visible = vec![HashMap::new(); modules.len()];
    checker.namespaces = vec![HashMap::new(); modules.len()];

    // `RangeError` before anything else. `[RNG-3]` writes the signature
    // `T.checked(v) -> Result[T, RangeError]`, so a program that declares a
    // function of that type — which is `[RNG-3]`'s own worked example — needs
    // the name to resolve while **signatures** are being collected, long
    // before any body mentions `checked`. Creating it on first use meant it
    // did not exist yet and the example did not compile (D-025).
    //
    // It is a prelude type rather than a `std.core` declaration because the
    // compiler is what produces it: `checked` is a language-defined
    // construction under `[RNG-10]`, not a library function, so its error type
    // cannot depend on a module having been imported.
    checker.range_error_ty();
    // `[ARN-1]` — `Arena` is a prelude/compiler-known type just like the
    // Phase-2 `Cell` family. Register it before user declarations and before
    // signatures are resolved, so `fn f(a: Arena)` is valid even when no body
    // has constructed one yet.
    checker.arena_ty();
    checker.fixed_arena_ty();
    checker.scoped_arena_ty();

    // Names first, across every module, so that an import can name an item in
    // a module that has not been walked yet — `[MOD-4]` allows cycles.
    for (index, loaded) in modules.iter().enumerate() {
        checker.current_module = index;
        checker.declare_names(&loaded.module);
    }
    checker.bind_prelude(modules);
    checker.bind_imports(modules);
    // Type collection is whole-program and explicitly phased. Imported type
    // names are already bound above, but an imported nominal/generic type did
    // not formerly exist in `named_types`/`generic_structs` until that
    // module's per-module `collect` walk happened. Because the root is loaded
    // first, a root signature could therefore be checked before its imported
    // type existed. This is the same load-order defect the interface collector
    // below was split to avoid.
    for (index, loaded) in modules.iter().enumerate() {
        checker.current_module = index;
        checker.collect_type_headers(&loaded.module);
    }
    // Recipes may refer to earlier recipes in their declaring module. Walking
    // dependencies before importers also makes the normal module case
    // deterministic while all nominal headers are already globally visible.
    for (index, loaded) in modules.iter().enumerate().rev() {
        checker.current_module = index;
        checker.collect_generic_structs(&loaded.module);
    }
    for (index, loaded) in modules.iter().enumerate() {
        checker.current_module = index;
        checker.collect(&loaded.module);
    }
    // `[CLS-4]` depends on the completed openness of every nominal base, so
    // validate inheritance only after all modules have been collected.
    for (index, loaded) in modules.iter().enumerate() {
        checker.current_module = index;
        checker.validate_class_bases(&loaded.module);
    }
    for (index, loaded) in modules.iter().enumerate() {
        checker.current_module = index;
        checker.collect_interfaces(&loaded.module);
    }
    for (index, loaded) in modules.iter().enumerate() {
        checker.current_module = index;
        checker.collect_methods(&loaded.module);
    }
    checker.validate_class_methods(modules);
    // Conformance is a whole-program question. Checking it inside the loop
    // reports the same missing or mismatched member once per loaded module
    // and can run before a later module's extension has been collected.
    checker.check_implementations();
    let mut functions = Vec::new();
    let mut main = None;
    for (index, loaded) in modules.iter().enumerate() {
        checker.current_module = index;
        let program = checker.check_bodies(&loaded.module);
        main = main.or(program.main);
        functions.extend(program.functions);
    }
    // `[TYP-16]` — every instantiation reached while checking gets a real
    // body. One of those can reach another, so this drains until empty.
    functions.extend(checker.check_instantiations(modules));
    // `[CLO-1]` — the functions capture-free closures lowered to. They have no
    // name in source, so they are gathered as they are checked and appended
    // here rather than found by walking the modules again.
    functions.extend(std::mem::take(&mut checker.lambdas));
    let callable_declarations = checker.callable_declarations(modules);
    CheckOutput {
        program: Program { functions, main },
        callable_declarations,
    }
}

struct Signature {
    params: Vec<(Symbol, Ty, Mode, Span)>,
    ret: Ty,
    /// `[LT-1a]` — the parameter positions `@borrows(…)` names, when it is
    /// written. Part of the public contract (`[VER-2]`), so it travels with
    /// the signature rather than being re-read from the attributes later.
    borrows: Option<Vec<usize>>,
    /// `[TYP-16]` — the generic parameters this function declares, with the
    /// interfaces bounding each (`[TYP-17]`). Empty for an ordinary function.
    generics: Vec<GenericParam>,
}

/// One declared type parameter and what it is allowed to do.
#[derive(Clone)]
struct GenericParam {
    name: Symbol,
    /// `[TYP-17]` — only what these interfaces provide is permitted inside the
    /// body. There is no duck typing.
    bounds: Vec<Symbol>,
    /// `[CLO-3]` — the bound a parameter written `f: fn(A) -> R` carries.
    ///
    /// `Callable[Args, R]` is a *generic* interface and `InterfaceDef` has no
    /// generics in this phase, so the bound travels here rather than as a name
    /// in `bounds`. The check it performs is the same one `[TYP-17]` performs
    /// for any other bound — only what the bound provides is permitted — which
    /// is why it sits beside them.
    callable: Option<CallableBound>,
}

/// `[CLO-3]`, `[CLO-6]` — the signature a `fn(A) -> R` parameter may be called
/// with, and whether one call consumes it.
#[derive(Clone, Debug)]
struct CallableBound {
    params: Vec<FnParam>,
    ret: Ty,
    /// `[CLO-6]` — "A parameter written `owned f: fn(A) -> R` is a generic
    /// bounded by `CallableOnce`; `f: fn(A) -> R` and `mut f: fn(A) -> R` are
    /// bounded by `Callable`." Calling a `CallableOnce` consumes it, so a
    /// second call is `E3040` under `[OWN-3]` with no analysis of its own.
    once: bool,
    /// `[FN-6b]` — the callable-type boundary fact, distinct from the outer
    /// parameter mode that selects `Callable` versus `CallableOnce`.
    latebound: bool,
}

/// The callable capability a concrete closure environment provides. The
/// environment struct remains the source-level value; this is compiler-local
/// dispatch metadata derived from its checked body.
#[derive(Copy, Clone, Debug)]
struct ClosureCall {
    def: DefId,
    /// A closure that moves a non-`Copy` capture out of its owned environment
    /// implements `CallableOnce`, not `Callable` (`[CLO-2]`/`[CLO-6]`).
    once: bool,
}

/// The four owner-approved `[SPN-4]` iterator identities share one lowering
/// path while keeping their public nominal types distinct.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SpanIteratorKind {
    Elements { mutable: bool },
    Chunks { mutable: bool },
}

/// `[TYP-16]` — a struct declared with type parameters. Its fields are
/// resolved once with the parameters opaque, so instantiating one is a
/// substitution rather than another walk of the declaration.
#[derive(Clone)]
struct GenericStruct {
    params: Vec<Symbol>,
    /// The resolved bounds belonging to the owner's type parameters. A
    /// generic method's import-visible contract has to retain these alongside
    /// its own parameters: `$P0` in `Buffer[T].method` belongs to `Buffer`,
    /// not to a method-local binder.
    generic_params: Vec<GenericParam>,
    fields: Vec<FieldDef>,
    derives_copy: bool,
    /// `[MOD-7]` — carried to every instantiation, so a `pub(read)` field of
    /// `Buffer[T]` is read-only outside `Buffer`'s module for every `T`.
    declaring_module: usize,
    /// `[TYP-16]` — the methods declared in the body, resolved once with the
    /// type parameters left opaque. An instantiation substitutes them, the
    /// same way it substitutes the fields.
    methods: Vec<GenericMethod>,
}

/// One method of a generic struct: its shape in terms of the struct's type
/// parameters, and where its body is so an instantiation can be checked.
#[derive(Clone)]
struct GenericMethod {
    name: Symbol,
    /// `None` for an associated function on the generic type.
    receiver: Option<Mode>,
    /// The parameters after `self`. `self` itself is not here: its type is
    /// the instantiation, which does not exist until one is built.
    params: Vec<(Symbol, Ty, Mode, Span)>,
    ret: Ty,
    /// Type parameters declared by the method itself. Their `TyKind::Param`
    /// indices follow the owning generic type's parameters while this recipe
    /// is stored, then are rebased to zero when the owner is instantiated.
    generics: Vec<GenericParam>,
    borrows: Option<Vec<usize>>,
    /// Module, item and member index of the declaration.
    source: (usize, usize, usize),
    span: Span,
}

/// A method of one instantiation, waiting for its body to be checked against
/// the concrete types. The same work list as `pending`, for methods.
struct PendingMethod {
    def: DefId,
    owner: Ty,
    /// The generic it came from and the arguments it was built with, which is
    /// what binds `T` while the body is checked.
    origin: (Symbol, Vec<Ty>),
    source: (usize, usize, usize),
}

/// The declaration behind a source-defined method. Generic method instances
/// need this independently of ordinary function sources: the receiver type is
/// part of both body checking and deterministic symbol identity.
#[derive(Clone)]
struct MethodSource {
    owner: Ty,
    source: (usize, usize, usize),
    /// Bindings contributed by an instantiated generic owner, such as
    /// `Box[T]` with `T = i32`. Empty for a concrete owner.
    owner_bindings: Vec<(Symbol, Ty)>,
}

/// One instantiation of a generic function: which one, and with what.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Instance {
    def: DefId,
    args: Vec<Ty>,
    /// Capture-free function values supplied to implicit callable generic
    /// parameters. Static dispatch is by callable identity as well as type:
    /// two functions with the same signature may have different result
    /// provenance contracts at an `@latebound` boundary.
    callable_values: Vec<Option<DefId>>,
}

/// One method that can be found by `recv.name(...)`.
#[derive(Clone, Copy)]
struct MethodEntry {
    def: DefId,
    /// `[TYP-24]` — an inherent method always beats an interface method of the
    /// same name, and two interfaces offering one name is `E2070`.
    from_interface: Option<Symbol>,
    /// The receiver's mode, which decides how the receiver is passed.
    receiver: Mode,
}

/// A receiver-less function declared by a type or required by an interface.
/// Kept separate from `MethodEntry` so value-method lookup can never
/// accidentally treat a type-level operation as taking `self`.
struct AssociatedEntry {
    def: DefId,
    from_interface: Option<Symbol>,
}

/// One leaf of a `[GRM-5]` destructuring target and the aggregate projection
/// that supplies it. Nested parenthesised target lists append field indices.
struct DestructureLeaf<'a> {
    target: &'a ast::Expr,
    ty: Ty,
    projection: Vec<usize>,
}

/// The deliberately narrow first constructor slice for `[CLS-2]`. The
/// checker permits direct `self.field = ...` writes, field-only expressions,
/// `pass`, and nested `if` blocks for a class `init` with no base and no
/// defaulted fields. This is an implementation boundary, not a new language
/// rule: it keeps the current lowering from publishing a partially
/// initialized object until the remaining constructor control-flow forms are
/// connected.
#[derive(Clone)]
struct ClassInitState {
    owner: ClassId,
    receiver: LocalId,
    initialized: Vec<ClassFieldInit>,
    /// `[CLS-4]` — inherited storage is unavailable to the derived body until
    /// its direct base constructor has run on the same object.
    base_initialized: bool,
}

/// The constructor-local field lattice. `Maybe` is deliberately not treated
/// as a writable fresh slot: doing so would turn a write after a conditional
/// initialization into an overwrite on only some paths, before class field
/// drop flags exist.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClassFieldInit {
    Uninit,
    Init,
    Maybe,
}

impl ClassFieldInit {
    fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Uninit, Self::Uninit) => Self::Uninit,
            (Self::Init, Self::Init) => Self::Init,
            _ => Self::Maybe,
        }
    }
}

/// A declared interface: the methods a type must provide, and which of them
/// carry a default body.
struct InterfaceDef {
    /// Member name, the `DefId` its declared signature was given, its
    /// receiver mode (`None` for an associated function), and whether the
    /// interface supplies a body.
    ///
    /// The `DefId` exists so that `[TYP-17]`'s "only what the bounds provide"
    /// can be checked: inside `fn f[T: Shape]`, `x.area()` resolves to the
    /// interface's declaration and takes its type. Nothing calls it — a
    /// generic body is never emitted, only its instantiations are.
    methods: Vec<(Symbol, DefId, Option<Mode>, bool)>,
    supertraits: Vec<Symbol>,
}

struct Checker<'a> {
    types: &'a mut TypeTable,
    common: &'a CommonTypes,
    sink: &'a mut Sink,
    struct_ids: HashMap<Symbol, StructId>,
    class_ids: HashMap<Symbol, ClassId>,
    enum_ids: HashMap<Symbol, EnumId>,
    /// Every named type in scope — structs, classes, and enums.
    named_types: HashMap<Symbol, Ty>,
    fn_ids: HashMap<Symbol, DefId>,
    signatures: Vec<Signature>,
    /// `[CLO-1]` — the functions a capture-free closure lowers to. They have
    /// no name in source, so they are collected here and appended to the
    /// program rather than found by walking the module again.
    lambdas: Vec<Function>,
    /// Every method reachable as `value.name(...)`, keyed by the receiver's
    /// type and the method name.
    methods: HashMap<(Ty, Symbol), MethodEntry>,
    /// Receiver-less functions reachable as `Type.name(...)`.
    associated: HashMap<(Ty, Symbol), AssociatedEntry>,
    /// Interfaces by name, with what each requires.
    interfaces: HashMap<Symbol, InterfaceDef>,
    /// Which interfaces each type implements, for `[TYP-20]` coherence and to
    /// report a missing method against the right interface.
    implemented: Vec<(Ty, Symbol, Span)>,
    /// `const` and `static` values, substituted wherever their name is used.
    constants: HashMap<Symbol, Expr>,

    // -- modules (`[MOD-1]` … `[MOD-3]`) -------------------------------------
    /// Every name above is keyed by its **qualified** form, `a.b.Name`, so two
    /// modules may each declare `helper` without colliding.
    /// The dotted prefix of each module; the root module's is empty.
    prefixes: Vec<String>,
    /// Per module: the names it can write, mapped to the qualified name each
    /// one means. Its own items plus whatever it imported.
    visible: Vec<HashMap<Symbol, Symbol>>,
    /// Per module: names bound to a whole module by `import a.b.c`, so that
    /// `c.thing` resolves.
    namespaces: Vec<HashMap<Symbol, usize>>,
    /// Which module is being collected or checked right now.
    current_module: usize,
    /// `[TYP-16]` — the type parameters in scope right now: opaque while a
    /// generic body is checked, concrete while one is instantiated.
    type_params: HashMap<Symbol, Ty>,
    /// The generic parameters of the function being checked, so a method call
    /// on one can find its bounds (`[TYP-17]`).
    current_generics: Vec<GenericParam>,
    /// `[RNG-4]` — the interval each local is known to lie in, where one was
    /// derived at its initialiser.
    ///
    /// A range fact has to survive a binding or it is worth almost nothing:
    /// `half = r * 0.5` then `back: Roughness = half` is the shape `[RNG-10]`(d)
    /// exists for, and without this the fact dies at the `=` and the second
    /// line is `E2215`.
    ///
    /// A local that is written again loses its fact rather than joining it —
    /// conservative, and the conservative direction is a check that gets
    /// emitted rather than one that does not.
    local_ranges: HashMap<LocalId, (Bound, Bound)>,
    /// `[CLO-1]` — the function that runs each capturing closure, keyed by the
    /// anonymous struct that is its environment. A value of that struct type is
    /// callable, and this is what it calls.
    closure_calls: HashMap<StructId, ClosureCall>,
    /// `[CLO-2]` — set while a closure body is checked, so that a read of an
    /// enclosing local is seen as a capture rather than as an ordinary read.
    captures: Option<CaptureWatch>,
    /// What `Self` names right now: the concrete type inside a `struct`,
    /// `class`, `enum` or `extend` body, and `CommonTypes::self_ty` inside an
    /// `interface`, where the implementing type is not yet known. `None` where
    /// `Self` has no meaning, which is `E2020`.
    self_ty: Option<Ty>,
    /// `[IFC-4]` — the associated type names in scope while an interface
    /// declaration is read.
    assoc_scope: std::collections::BTreeSet<Symbol>,
    /// What each implementing type declared its associated types to be:
    /// `(the type, the name) -> the type it stands for`.
    assoc_values: HashMap<(Ty, Symbol), Ty>,
    /// Instantiations reached so far, so each is emitted once (`[MONO-1]`).
    instances: HashMap<Instance, DefId>,
    /// Structs declared with type parameters, by qualified name.
    generic_structs: HashMap<Symbol, GenericStruct>,
    /// `[CELL-1]` — every `Cell[T]` built so far, and the `T` it holds. A cell
    /// is an ordinary `StructId` everywhere else in the compiler, so this is
    /// what tells method dispatch that `set` on it is a builtin rather than a
    /// missing method.
    cells: HashMap<StructId, Ty>,
    /// `[DRP-6]`, IX.1 — every compiler-known `Box[T]` and its payload.
    /// The public value is an ordinary move-only struct containing one private
    /// pointer. This table supplies the ownership-aware constructor,
    /// auto-deref and drop semantics without making raw access public.
    boxes: HashMap<StructId, Ty>,
    /// `[CELL-5]` — every `RefCell[T]` built so far, and the `T` it holds.
    /// A transparent struct with a second field for the one-word borrow
    /// counter plus location fields for the conflicting borrow's source
    /// location (named in the panic in debug and release). Like `cells`, this
    /// is what tells method dispatch that `borrow` on it is a builtin.
    refcells: HashMap<StructId, Ty>,
    /// `[ARN-8]` — every compiler-known `MaybeUninit[T]` wrapper and its
    /// payload type. The wrapper has `T`'s layout but suppresses structural
    /// destruction, which cannot be inferred from its field alone.
    maybe_uninit: HashMap<StructId, Ty>,
    /// `[UNS-10]` — every compiler-known `std.mem.UnsafeCell[T]` and its
    /// payload. It remains an ordinary private one-field struct everywhere
    /// else; this side table supplies only its deliberately narrow method and
    /// trait behavior.
    unsafe_cells: HashMap<StructId, Ty>,
    /// `[CLS-2]` — field initialization facts while a narrow class
    /// constructor body is checked.
    class_init: Option<ClassInitState>,
    /// `[CLS-7]`/`[EXC-1]` — the receiver of the class method currently being
    /// checked when it is a `mut self` method.  This is separate from
    /// `class_init`: mutable methods may write through their own receiver,
    /// while constructor fields still have the definite-initialization
    /// lattice and its special first-write rule.
    class_method_receiver: Option<(ClassId, LocalId)>,
    /// Literal defaults retained for the narrow synthesized class constructor
    /// path. The full `[STR-2]` default-expression evaluator is still a later
    /// dependency; non-literal class defaults remain fail-closed.
    class_default_literals: HashMap<ClassId, Vec<Option<ast::Literal>>>,
    /// A field expression is a place, not a read, while the assignment target
    /// is being synthesized. This prevents constructor writes from tripping
    /// the read-before-initialization check on their own left-hand side.
    in_assignment_target: bool,
    /// `[CELL-7]` — every `Ref[T]`/`RefMut[T]` guard built so far, and the
    /// `T` plus mutability it views. A transparent one-field struct holding a
    /// `ref`/`ref mut`, so `[TYP-15]` applies via `is_view` and the region
    /// borrows the cell. `has_drop` is set so the guard's `drop` releases the
    /// borrow state; `derives_copy` is clear so guards are move-only
    /// (`[CELL-12]`'s reasoning for the cell applies to the guard's borrow).
    ref_guards: HashMap<StructId, (Ty, bool)>,
    /// `[ARN-1]` — the one compiler-known `Arena` struct. It owns an opaque
    /// runtime state pointer, is move-only, and has compiler-provided drop
    /// glue. A set keeps the recognition structural without teaching every
    /// compiler phase a new `TyKind`.
    arenas: HashSet<StructId>,
    /// `[ARN-4]` — compiler-known fixed arenas borrow a caller-provided byte
    /// span and never grow. Kept separate from `arenas` because their layout,
    /// destruction, and code generation are intentionally different.
    fixed_arenas: HashSet<StructId>,
    /// `[ARN-6]` — LIFO scope guards, recognized separately so method
    /// dispatch and compiler-provided rewind glue cannot be confused with a
    /// user struct that happens to contain references.
    scoped_arenas: HashSet<StructId>,
    /// Instantiations still to have their bodies checked.
    pending: Vec<(Instance, DefId)>,
    /// The same, for the methods of instantiated generic structs.
    pending_methods: Vec<PendingMethod>,
    /// Source declarations for generic methods, keyed by their uninstantiated
    /// method `DefId`.
    generic_method_sources: HashMap<DefId, MethodSource>,
    /// Import-visible member declarations collected from resolved source
    /// signatures. This is distinct from HIR/MIR bodies because interface and
    /// generic members can be semantically visible without an emitted body.
    member_callable_declarations: Vec<CallableDeclaration>,
    /// Concrete generic-method instances waiting for body checking/emission.
    pending_generic_methods: Vec<(Instance, DefId)>,
    /// Generic methods on a newly instantiated generic owner must still be
    /// checked once with their own parameters opaque, even if never called.
    pending_generic_method_validations: Vec<DefId>,

    // Per-function state.
    locals: Vec<LocalDecl>,
    scopes: Vec<HashMap<Symbol, LocalId>>,
    /// `[FN-1]` — locals introduced by default-mode parameters. Their values
    /// are shared borrows even though small `Copy` values may use a by-value
    /// ABI, so no write rooted at one is permitted. Keeping the mode here lets
    /// the type checker reject the write before that ABI detail reaches MIR.
    borrowed_params: HashSet<LocalId>,
    /// Locals declared by `owned f: fn(...) -> R`. The function type itself is
    /// the callable signature, not the ownership mode on `f`; retaining this
    /// fact lets an indirect call consume exactly a `CallableOnce` parameter.
    callable_once_locals: HashSet<LocalId>,
    /// Callable parameters retain their source spelling for diagnostics after
    /// monomorphisation replaces a `fn(...)` bound with a private closure type.
    callable_parameter_locals: HashSet<LocalId>,
    /// The expected callable boundary may be late-bound even after a generic
    /// parameter has been monomorphized to a concrete function or closure
    /// type. Keep that source-level fact alongside the local identity.
    latebound_callable_parameter_locals: HashSet<LocalId>,
    /// Known capture-free callable arguments for the current concrete generic
    /// body, keyed by source parameter position. This is compile-time-only
    /// static-dispatch information; it never reaches HIR/MIR layout or ABI.
    callable_value_params: HashMap<usize, DefId>,
    /// The same bindings after source parameters have become concrete locals.
    callable_value_bindings: HashMap<LocalId, DefId>,
    /// While checking the second and later alternatives of an `|` pattern:
    /// the locals the first alternative bound, which they must reuse.
    or_bindings: Option<HashMap<Symbol, LocalId>>,
    /// The loops currently open, innermost last, each with its label if it has
    /// one. `break`/`continue` index into this to find their target.
    loop_labels: Vec<Option<Symbol>>,
    /// `[CTL-7]` — inside a `defer` block, where control may not leave.
    in_defer: bool,
    /// `[UNS-1]`, `[UNS-2]` — inside an `unsafe:` block, where the raw memory
    /// operations are permitted.
    in_unsafe: bool,
    /// `[UNS-10b]` — `UnsafeCell` delegates an invariant to unsafe code and
    /// therefore cannot occur in a function that promises `@static_safe`.
    in_static_safe: bool,
    /// Keep one precise E3105 per function even when the forbidden type occurs
    /// in both its signature and body.
    static_safe_unsafe_cell_reported: bool,
    ret_ty: Ty,
    /// The profile's `[TYP-8]` policy, used when a function has no
    /// `@overflow(...)` of its own.
    default_overflow: OverflowPolicy,
}

impl<'a> Checker<'a> {
    fn new(types: &'a mut TypeTable, common: &'a CommonTypes, sink: &'a mut Sink) -> Checker<'a> {
        let ret_ty = common.void;
        Checker {
            types,
            common,
            sink,
            struct_ids: HashMap::new(),
            class_ids: HashMap::new(),
            enum_ids: HashMap::new(),
            named_types: HashMap::new(),
            fn_ids: HashMap::new(),
            signatures: Vec::new(),
            methods: HashMap::new(),
            associated: HashMap::new(),
            interfaces: HashMap::new(),
            implemented: Vec::new(),
            constants: HashMap::new(),
            prefixes: vec![String::new()],
            visible: vec![HashMap::new()],
            namespaces: vec![HashMap::new()],
            current_module: 0,
            type_params: HashMap::new(),
            current_generics: Vec::new(),
            local_ranges: HashMap::new(),
            closure_calls: HashMap::new(),
            lambdas: Vec::new(),
            captures: None,
            self_ty: None,
            assoc_scope: std::collections::BTreeSet::new(),
            assoc_values: HashMap::new(),
            instances: HashMap::new(),
            generic_structs: HashMap::new(),
            cells: HashMap::new(),
            boxes: HashMap::new(),
            refcells: HashMap::new(),
            maybe_uninit: HashMap::new(),
            unsafe_cells: HashMap::new(),
            class_init: None,
            class_method_receiver: None,
            class_default_literals: HashMap::new(),
            in_assignment_target: false,
            ref_guards: HashMap::new(),
            arenas: HashSet::new(),
            fixed_arenas: HashSet::new(),
            scoped_arenas: HashSet::new(),
            pending: Vec::new(),
            pending_methods: Vec::new(),
            generic_method_sources: HashMap::new(),
            member_callable_declarations: Vec::new(),
            pending_generic_methods: Vec::new(),
            pending_generic_method_validations: Vec::new(),
            locals: Vec::new(),
            scopes: Vec::new(),
            borrowed_params: HashSet::new(),
            callable_once_locals: HashSet::new(),
            callable_parameter_locals: HashSet::new(),
            latebound_callable_parameter_locals: HashSet::new(),
            callable_value_params: HashMap::new(),
            callable_value_bindings: HashMap::new(),
            or_bindings: None,
            loop_labels: Vec::new(),
            in_defer: false,
            in_unsafe: false,
            in_static_safe: false,
            static_safe_unsafe_cell_reported: false,
            ret_ty,
            default_overflow: OverflowPolicy::default(),
        }
    }

    fn error(&mut self, code: ember_diag::Code, span: Span, message: impl Into<String>) {
        self.sink.emit(Diagnostic::error(code, span, message));
    }

    /// A generic body is first checked with opaque parameters and then again
    /// for each concrete monomorphisation. Most second-pass diagnostics would
    /// duplicate the first pass and stay quiet. `[ARN-3]` is the first rule
    /// whose verdict can change only after substitution: `T` may become an
    /// `Array[U]` that needs drop. Preserve that concrete E3090 instead of
    /// silently accepting resource storage with no destructor bookkeeping.
    fn emit_concrete_instantiation_diagnostics(&mut self, diagnostics: Vec<Diagnostic>) {
        for diagnostic in diagnostics {
            let duplicate = self.sink.diagnostics().iter().any(|existing| {
                existing.code == diagnostic.code
                    && existing.primary.span == diagnostic.primary.span
                    && existing.message == diagnostic.message
            });
            if !duplicate {
                if diagnostic
                    .code
                    .is_some_and(|code| code.subsystem == ember_diag::codes::Subsystem::Ownership)
                {
                    self.sink.emit_classified(diagnostic);
                } else {
                    self.sink.emit(diagnostic);
                }
            }
        }
    }

    /// `[LT-1a]` — `@borrows(p₁, …, pₙ)` overrides the region that `[LT-1]`'s
    /// elision would give a view-typed return, so that the caller may keep
    /// using the parameters it does *not* name.
    ///
    /// "Naming a parameter that is not view-typed, or writing `@borrows` on a
    /// function whose return is not view-typed, is `E2031`." `[LT-4a]` adds
    /// one narrow exception: a growing `Arena` may be named as the provenance
    /// source of storage backing the returned view. It does not make Arena a
    /// view type. A name that
    /// matches no parameter is the same mistake and is reported the same way,
    /// with the parameters that would have been valid listed — the attribute
    /// is a contract (`[VER-2]` makes widening it a breaking change), so a
    /// typo in it is worth catching loudly.
    fn check_borrows_attribute(
        &mut self,
        attrs: &[ast::Attribute],
        params: &[(Symbol, Ty, Mode, Span)],
        ret: Ty,
    ) -> Option<Vec<usize>> {
        let Some(attr) = attrs
            .iter()
            .find(|a| a.path.len() == 1 && a.path[0].name.is("borrows"))
        else {
            return None;
        };

        if attr.args.is_empty() {
            self.sink.emit(
                Diagnostic::error(
                    codes::E2031,
                    attr.span,
                    "`@borrows` must name at least one parameter",
                )
                .primary_label("name one or more view-typed or Arena parameters")
                .help("name the parameter(s) that the returned view derives from"),
            );
            // Match the existing invalid-attribute paths above: report the
            // source error, then let later checks use ordinary elision rather
            // than manufacturing an empty provenance contract.
            return None;
        }

        if !self.types.is_view(ret) {
            let shown = self.types.display(ret);
            self.sink.emit(
                Diagnostic::error(
                    codes::E2031,
                    attr.span,
                    "`@borrows` is only meaningful on a function that returns a view",
                )
                .primary_label(format!("this function returns `{shown}`"))
                .help("remove the attribute, or return a `ref`, a `Span` or a `@view` struct")
                .note("`@borrows` chooses which parameter the *returned* view borrows (LT-1a)"),
            );
            return None;
        }

        let provenance_sources: Vec<String> = params
            .iter()
            .filter(|(_, ty, _, _)| self.types.is_view(*ty) || self.is_arena(*ty))
            .map(|(name, _, _, _)| name.to_string())
            .collect();

        let mut named_positions = Vec::new();
        for arg in &attr.args {
            let (named, arg_span) = match arg {
                ast::AttrArg::Expr(expr) => match &expr.kind {
                    ast::ExprKind::Path { segments } if segments.len() == 1 => {
                        (segments[0].name, expr.span)
                    }
                    _ => {
                        self.sink.emit(
                            Diagnostic::error(
                                codes::E2031,
                                expr.span,
                                "each `@borrows` argument must be a parameter name",
                            )
                            .primary_label("expected one parameter name")
                            .help("write `@borrows(parameter)` using a view-typed or Arena parameter"),
                        );
                        continue;
                    }
                },
                ast::AttrArg::Named { name, .. } => {
                    self.sink.emit(
                        Diagnostic::error(
                            codes::E2031,
                            name.span,
                            "each `@borrows` argument must be a parameter name",
                        )
                        .primary_label("named arguments are not valid here")
                        .help("write `@borrows(parameter)` using a view-typed or Arena parameter"),
                    );
                    continue;
                }
            };
            if let Some(position) = params.iter().position(|(name, _, _, _)| *name == named) {
                named_positions.push(position);
            }
            match params.iter().find(|(name, _, _, _)| *name == named) {
                None => {
                    let known = if provenance_sources.is_empty() {
                        "this function has no view-typed or Arena provenance parameter".to_string()
                    } else {
                        format!(
                            "the view-typed or Arena provenance parameters are {}",
                            provenance_sources.join(", ")
                        )
                    };
                    self.sink.emit(
                        Diagnostic::error(
                            codes::E2031,
                            arg_span,
                            format!("`@borrows` names `{named}`, which is not a parameter"),
                        )
                        .help(known),
                    );
                }
                Some((_, ty, _, _)) if self.is_arena(*ty) => {
                    // `[LT-4a]` — body checking still has to prove that the
                    // returned view actually derives from this arena. Merely
                    // naming it grants no region and no storage authority.
                }
                Some((_, ty, _, span)) if !self.types.is_view(*ty) => {
                    let shown = self.types.display(*ty);
                    self.sink.emit(
                        Diagnostic::error(
                            codes::E2031,
                            arg_span,
                            format!("`@borrows` names `{named}`, which is not view-typed"),
                        )
                        .secondary(*span, format!("`{named}` is `{shown}`, which borrows nothing"))
                        .help(concat!(
                            "name a parameter the return can point into, or drop the ",
                            "attribute and let elision tie the region (LT-1)"
                        )),
                    );
                }
                Some(_) => {}
            }
        }
        Some(named_positions)
    }

    /// `[TYP-15]` with `[LT-3]` — "a view-typed value MUST NOT be stored in a
    /// place whose region is not outlived by the view's region."
    ///
    /// The places named here have **no bounding region**, so the only view
    /// they may hold is one whose region is `static` — which `[LT-3]` gives to
    /// string literals, `static` items, and `Span`s over them. A static region
    /// outlives everything, including the place, so the rule's own condition
    /// is met rather than waived.
    ///
    /// This is the owner's resolution of ERR-044. `[TYP-15]` used to enumerate
    /// these places as "always forbidden", which contradicted `[LT-3]`'s
    /// statement that a `str` literal *may* be stored in a class field — and
    /// contradicted `[TYP-15]`'s own principle, since a static region does
    /// outlive the destination. The enumeration was the part that overreached.
    ///
    /// The exception is on the **view's region, not the destination type**:
    /// `class Foo: greeting: str = "hello"` is fine and
    /// `foo.greeting = s` for a parameter `s` is not, because `s` may carry a
    /// caller's region that does not outlive the field.
    ///
    /// It does not touch `[TYP-15a]`: an owning container instantiated at a
    /// view type — `Array[str]`, `Map[str, V]`, `Array[MutSpan[T]]` — stays
    /// rejected whatever the region, because that rejection is at the type and
    /// not at the region.
    fn reject_stored_view(&mut self, ty: Ty, span: Span, place: &str) {
        self.reject_stored_view_unless(ty, span, place, false)
    }

    /// As above, with `static_region` true where the value being stored is
    /// known to have `[LT-3]`'s `static` region.
    fn reject_stored_view_unless(
        &mut self,
        ty: Ty,
        span: Span,
        place: &str,
        static_region: bool,
    ) {
        if !self.types.is_view(ty) {
            return;
        }
        if static_region {
            return;
        }
        let shown = self.types.display(ty);
        self.sink.emit_classified(
            Diagnostic::error(
                codes::E3063,
                span,
                format!("`{shown}` is a view, so it may not be stored in {place}"),
            )
            .primary_label("stored here")
            .help(concat!(
                "store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — ",
                "and note that costs one allocation per element; or store a `u32` index ",
                "or a `Handle[T]` and name the container it indexes"
            ))
            .note(concat!(
                "this place has no bounding region, so only a view with the `static` ",
                "region may be stored in it (TYP-15, LT-3)"
            )),
        );
    }

    /// `[LEX-15a]` — `type Name = T` at item level. An alias may name another
    /// alias declared later in the file, so they are resolved to a fixpoint:
    /// one whose body still mentions an unknown name is deferred, and whatever
    /// survives no-further-progress is a cycle or a genuinely unknown type.
    fn collect_type_aliases(&mut self, module: &ast::Module) {
        let mut pending: Vec<&ast::TypeAlias> = module
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                ast::ItemKind::TypeAlias(decl) => Some(decl),
                _ => None,
            })
            .collect();

        loop {
            let before = pending.len();
            let mut deferred = Vec::new();
            for decl in std::mem::take(&mut pending) {
                let name = self.qualified(decl.name.name);
                if self.named_types.contains_key(&name) {
                    self.error(
                        codes::E1030,
                        decl.name.span,
                        format!("`{name}` is already declared in this module"),
                    );
                    continue;
                }
                // A generic alias needs the parameters in scope at every use,
                // which is substitution work `[TYP-16]` puts in Phase 2.
                if !decl.generics.is_empty() {
                    self.error(
                        codes::E1010,
                        decl.name.span,
                        "a generic type alias is not supported yet",
                    );
                    continue;
                }
                let Some(value) = &decl.value else {
                    self.error(
                        codes::E1010,
                        decl.name.span,
                        "a type alias needs a value: write `type Name = T`",
                    );
                    continue;
                };
                if !self.alias_body_is_resolvable(value) {
                    deferred.push(decl);
                    continue;
                }
                let repr = self.resolve_type(value);
                // `[RNG-1]` — an alias carrying an `in` clause declares a
                // **nominal** numeric type over `repr`; one without it is
                // transparent, exactly as `[LEX-15a]` specifies.
                let ty = match &decl.range {
                    Some(clause) => self.declare_range_type(decl, repr, clause),
                    None => repr,
                };
                self.named_types.insert(name, ty);
            }
            pending = deferred;
            if pending.is_empty() || pending.len() == before {
                break;
            }
        }

        for decl in pending {
            let Some(value) = &decl.value else { continue };
            // Resolving now emits the real "cannot find type" diagnostic and
            // names the member that could not be found, rather than a bare
            // "alias unresolved" that hides which name is at fault.
            let ty = self.resolve_type(value);
            let name = self.qualified(decl.name.name);
            self.named_types.insert(name, ty);
        }
    }

    /// `[RNG-1]` — build the nominal range type an `in` clause declares.
    ///
    /// The clause "takes a range expression (`a .. b` or `a ..= b`) whose
    /// endpoints are constant expressions of the representation type". Every
    /// way that can be false is `E2212`, except a non-numeric representation,
    /// which is `E2213` — the two codes IV.2a's diagnostic list gives.
    fn declare_range_type(
        &mut self,
        decl: &ast::TypeAlias,
        repr: Ty,
        clause: &ast::Expr,
    ) -> Ty {
        let is_int = matches!(
            self.types.kind(repr),
            TyKind::Int(_) | TyKind::Uint(_)
        );
        let is_float = matches!(self.types.kind(repr), TyKind::Float(_));
        if !is_int && !is_float {
            let shown = self.types.display(repr);
            self.error(
                codes::E2213,
                clause.span,
                format!("`{shown}` is not a numeric type, so it has no range"),
            );
            return repr;
        }

        let ast::ExprKind::Range { lo, hi, inclusive } = &clause.kind else {
            self.error(
                codes::E2212,
                clause.span,
                "an `in` clause takes a range: write `a ..= b` or `a .. b`",
            );
            return repr;
        };
        let (Some(lo_expr), Some(hi_expr)) = (lo.as_ref(), hi.as_ref()) else {
            self.error(
                codes::E2212,
                clause.span,
                "a range type needs both endpoints: `a ..` and `.. b` are open",
            );
            return repr;
        };

        let Some(lo_bound) = self.const_bound(lo_expr, is_float) else { return repr };
        let Some(hi_bound) = self.const_bound(hi_expr, is_float) else { return repr };

        // "or are inverted" — `[RNG-1]`'s other rejection.
        if !lo_bound.le(hi_bound) {
            self.error(
                codes::E2212,
                clause.span,
                "the range is inverted: the low endpoint is above the high one",
            );
            return repr;
        }
        // A half-open range whose endpoints are equal holds nothing, and
        // `[RNG-9]` makes every value of such a type invalid — a type no
        // program can construct a value of is a mistake, not a design.
        if !inclusive && lo_bound == hi_bound {
            self.error(
                codes::E2212,
                clause.span,
                "this range is empty, so the type has no valid value",
            );
            return repr;
        }

        let id = self.types.add_range(RangeDef {
            name: decl.name.name,
            repr,
            lo: lo_bound,
            hi: hi_bound,
            inclusive: *inclusive,
            span: decl.name.span,
        });
        self.types.intern(TyKind::Range(id))
    }

    /// One endpoint of an `in` clause, folded to a constant. `[RNG-1]` admits
    /// "constant expressions of the representation type"; this phase accepts a
    /// literal, a unary minus on one, and a `const` — which is what
    /// `const_len` already accepts for an array length, and for the same
    /// reason: the comptime interpreter that would evaluate the rest is
    /// Phase 4's (`[CT-1]`).
    fn const_bound(&mut self, expr: &ast::Expr, want_float: bool) -> Option<Bound> {
        let mut expr = expr;
        let mut negate = false;
        loop {
            match &expr.kind {
                ast::ExprKind::Paren(inner) => expr = inner,
                ast::ExprKind::Unary { op: ast::UnOp::Neg, operand } => {
                    negate = !negate;
                    expr = operand;
                }
                _ => break,
            }
        }
        let bound = match &expr.kind {
            ast::ExprKind::Lit(ast::Literal::Int { value, .. }) => {
                let v = i128::try_from(*value).ok()?;
                let v = if negate { -v } else { v };
                // `[LEX-16]` — an untyped integer literal takes the type
                // context wants, and here that is the representation.
                if want_float { Bound::Float(v as f64) } else { Bound::Int(v) }
            }
            ast::ExprKind::Lit(ast::Literal::Float { value, .. }) => {
                if !want_float {
                    self.error(
                        codes::E2212,
                        expr.span,
                        "a float endpoint on an integer representation",
                    );
                    return None;
                }
                // `[RNG-6]` — "NaN is in no range", so an endpoint that is one
                // describes nothing.
                if value.is_nan() {
                    self.error(codes::E2212, expr.span, "`NaN` is not a range endpoint");
                    return None;
                }
                Bound::Float(if negate { -*value } else { *value })
            }
            ast::ExprKind::Path { segments } if segments.len() == 1 => {
                match self.constants.get(&segments[0].name).map(|c| &c.kind) {
                    Some(hir::ExprKind::Int(value)) => {
                        let v = i128::try_from(*value).ok()?;
                        let v = if negate { -v } else { v };
                        if want_float { Bound::Float(v as f64) } else { Bound::Int(v) }
                    }
                    Some(hir::ExprKind::Float(value)) if want_float => {
                        let value = *value;
                        Bound::Float(if negate { -value } else { value })
                    }
                    _ => {
                        self.error(
                            codes::E2212,
                            expr.span,
                            "a range endpoint must be a constant of the representation type",
                        );
                        return None;
                    }
                }
            }
            _ => {
                self.error(
                    codes::E2212,
                    expr.span,
                    "a range endpoint must be a constant of the representation type",
                );
                return None;
            }
        };
        Some(bound)
    }

    /// Every named type mentioned in an alias body is already known. Used to
    /// order alias resolution without emitting diagnostics for the deferral.
    fn alias_body_is_resolvable(&self, ty: &ast::TypeExpr) -> bool {
        match &ty.kind {
            ast::TypeKind::Void | ast::TypeKind::Never | ast::TypeKind::SelfType => true,
            ast::TypeKind::Ref { inner, .. } | ast::TypeKind::Ptr { inner, .. } => {
                self.alias_body_is_resolvable(inner)
            }
            ast::TypeKind::Tuple(items) => {
                items.iter().all(|t| self.alias_body_is_resolvable(t))
            }
            ast::TypeKind::Array { elem, .. } => self.alias_body_is_resolvable(elem),
            ast::TypeKind::Path { segments, args } => {
                if !args.iter().all(|a| match a {
                    ast::GenericArg::Type(t) => self.alias_body_is_resolvable(t),
                    _ => true,
                }) {
                    return false;
                }
                if segments.len() != 1 {
                    return true;
                }
                let name = segments[0].name;
                self.scalar_named(name.as_str()).is_some()
                    || name.is("String")
                    || self.named_types.contains_key(&self.resolve_name(name))
                    || !args.is_empty()
            }
            _ => true,
        }
    }

    // -- modules -------------------------------------------------------------

    /// `[TYP-16]` — bring a declaration's type parameters into scope as opaque
    /// types, so the signature and the body are checked against the bounds
    /// rather than against whatever they are eventually instantiated with.
    fn declare_generics(&mut self, params: &[ast::GenericParam]) -> Vec<GenericParam> {
        self.type_params.clear();
        self.declare_generics_from(params, 0)
    }

    /// Declare parameters without discarding an enclosing generic type's
    /// bindings. Method parameters use indices after the owner's parameters
    /// until owner instantiation substitutes and rebases them.
    fn declare_generics_from(
        &mut self,
        params: &[ast::GenericParam],
        index_base: usize,
    ) -> Vec<GenericParam> {
        let mut declared = Vec::new();
        for (index, param) in params.iter().enumerate() {
            // A const generic is a value, not a type; `[TYP-16]`'s type
            // parameters are what this phase handles.
            if param.const_ty.is_some() {
                self.error(
                    codes::E1010,
                    param.span,
                    "a const generic is not supported yet in this phase",
                );
                continue;
            }
            let ty = self
                .types
                .intern(TyKind::Param {
                    index: (index_base + index) as u32,
                    name: param.name.name,
                });
            self.type_params.insert(param.name.name, ty);
            // The bound is recorded under the name the interface is
            // registered by, so `T: Ord` finds `std.core.Ord` when `Ord` was
            // imported. Storing what was written made an imported bound
            // resolve to nothing and report `[TYP-17]`'s "its bounds do not
            // provide one" about a bound that did.
            let bounds = param
                .bounds
                .iter()
                .filter_map(interface_name)
                .map(|name| self.resolve_name(name))
                .collect();
            declared.push(GenericParam { name: param.name.name, bounds, callable: None });
        }
        declared
    }

    /// `[CLO-3]` — "A parameter declared `f: fn(A) -> R` is a generic over
    /// `Callable` (static dispatch, monomorphised)."
    ///
    /// So a `fn(...)` type in **parameter position** is not a type at all: it
    /// is a bound on a type parameter the function did not write. This turns
    /// it into one, appending an implicit generic and giving the parameter
    /// that generic's opaque type.
    ///
    /// Parameter position only. `[FN-6]` keeps `fn(A) -> R` an ordinary type
    /// everywhere else — "functions are values of a unique zero-sized function
    /// type; they coerce to `fn(A) -> R`" — so a local, a field or a return
    /// type written that way is a capture-free callable, which is a function
    /// pointer and stays one. The two readings are what let
    /// `step: fn(i32) -> i32 = fn(x) => x * 3` and `apply(fn(x) => x * n, 2)`
    /// both work while meaning different things.
    ///
    /// Realising the parameter as a function pointer instead — which is what
    /// this compiler did until now — erases the environment, so every lambda
    /// that captures anything is rejected. That is ADR-018, and this is what
    /// closes it.
    fn callable_param_ty(
        &mut self,
        ty: &ast::TypeExpr,
        mode: ast::Mode,
        generics: &mut Vec<GenericParam>,
        index_base: usize,
    ) -> Ty {
        let resolved = self.resolve_type(ty);
        let TyKind::Fn { latebound, params, ret } = self.types.kind(resolved) else { return resolved };
        let bound = CallableBound {
            params: params.clone(),
            ret: *ret,
            // `[CLO-6]` — the mode already selects the bound.
            once: mode == ast::Mode::Owned,
            latebound: *latebound,
        };
        let index = (index_base + generics.len()) as u32;
        let name = Symbol::intern(&format!("Callable{index}"));
        let param_ty = self.types.intern(TyKind::Param { index, name });
        generics.push(GenericParam { name, bounds: Vec::new(), callable: Some(bound) });
        param_ty
    }

    /// Build the import-visible callable declaration set from the same
    /// resolved signatures body checking uses. This is intentionally not
    /// reconstructed from diagnostic strings or raw source: generic bounds
    /// have already been name-resolved and parameter types already carry
    /// their opaque generic identities here.
    fn callable_declarations(&self, modules: &[LoadedModule]) -> Vec<CallableDeclaration> {
        let mut declarations = self.member_callable_declarations.clone();
        for (module_index, loaded) in modules.iter().enumerate() {
            let prefix = &self.prefixes[module_index];
            for item in &loaded.module.items {
                match &item.kind {
                    ast::ItemKind::Fn(decl) if item.vis.kind != ast::VisKind::Private => {
                        let qualified = if prefix.is_empty() {
                            decl.name.name
                        } else {
                            Symbol::intern(&format!("{prefix}.{}", decl.name.name))
                        };
                        let Some(&def) = self.fn_ids.get(&qualified) else {
                            // A prior declaration error owns the diagnostic. Do not
                            // manufacture a partial interface from a rejected item.
                            continue;
                        };
                        let symbol = match &decl.abi {
                            Some(_) => decl.name.name.to_string(),
                            None => mangle(qualified, qualified.is("main")),
                        };
                        declarations.push(self.callable_declaration(
                            item.span,
                            symbol,
                            self.signature_parameters(&self.signatures[def.0 as usize]),
                            self.signatures[def.0 as usize].ret,
                            self.signatures[def.0 as usize].borrows.clone(),
                            &self.signatures[def.0 as usize].generics,
                            decl.is_unsafe,
                            decl.abi.clone(),
                        ));
                    }
                    // The owner of `Buffer[T].method` has no runtime `Ty`
                    // until a particular `Buffer[...]` is instantiated. Its
                    // source contract nevertheless belongs in EMIF now, with
                    // the owner's parameter list before the method's own
                    // parameter list. A generated specialization cannot stand
                    // in for that declaration.
                    ast::ItemKind::Struct(decl)
                        if !decl.generics.is_empty()
                            && item.vis.kind != ast::VisKind::Private =>
                    {
                        let qualified = if prefix.is_empty() {
                            decl.name.name
                        } else {
                            Symbol::intern(&format!("{prefix}.{}", decl.name.name))
                        };
                        let Some(generic) = self.generic_structs.get(&qualified) else {
                            continue;
                        };
                        let owner = canonical_generic_owner(qualified, generic.params.len());
                        for method in &generic.methods {
                            let Some(member) = decl.members.get(method.source.2) else {
                                continue;
                            };
                            if member.vis.kind == ast::VisKind::Private {
                                continue;
                            }
                            let ast::MemberKind::Fn(source) = &member.kind else {
                                continue;
                            };
                            let mut parameters = Vec::with_capacity(
                                method.params.len() + usize::from(method.receiver.is_some()),
                            );
                            if let Some(mode) = method.receiver {
                                parameters.push(CallableDeclarationParameter {
                                    mode,
                                    ty: CallableDeclarationType::Canonical(owner.clone()),
                                });
                            }
                            parameters.extend(method.params.iter().map(|(_, ty, mode, _)| {
                                CallableDeclarationParameter {
                                    mode: *mode,
                                    ty: CallableDeclarationType::Resolved(*ty),
                                }
                            }));
                            let mut generics = generic.generic_params.clone();
                            generics.extend(method.generics.clone());
                            declarations.push(self.callable_declaration(
                                method.span,
                                member_declaration_symbol(&format!("generic:{qualified}"), method.name),
                                parameters,
                                method.ret,
                                method.borrows.clone(),
                                &generics,
                                source.is_unsafe,
                                source.abi.clone(),
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
        declarations.sort_by(|left, right| left.symbol.cmp(&right.symbol));
        declarations
    }

    fn signature_parameters(&self, signature: &Signature) -> Vec<CallableDeclarationParameter> {
        signature
            .params
            .iter()
            .map(|(_, ty, mode, _)| CallableDeclarationParameter {
                mode: *mode,
                ty: CallableDeclarationType::Resolved(*ty),
            })
            .collect()
    }

    fn callable_declaration(
        &self,
        span: Span,
        symbol: String,
        parameters: Vec<CallableDeclarationParameter>,
        result: Ty,
        borrows: Option<Vec<usize>>,
        generics: &[GenericParam],
        is_unsafe: bool,
        abi: Option<String>,
    ) -> CallableDeclaration {
        CallableDeclaration {
            span,
            symbol,
            parameters,
            result,
            borrows,
            generics: generics
                .iter()
                .map(|parameter| CallableDeclarationGeneric {
                    bounds: parameter.bounds.iter().map(ToString::to_string).collect(),
                    callable: parameter.callable.as_ref().map(|bound| {
                        CallableDeclarationCallableBound {
                            parameters: bound.params.clone(),
                            result: bound.ret,
                            once: bound.once,
                            latebound: bound.latebound,
                        }
                    }),
                })
                .collect(),
            is_unsafe,
            abi,
        }
    }

    /// The qualified form of a name declared in the module being walked.
    fn qualified(&self, name: Symbol) -> Symbol {
        let prefix = &self.prefixes[self.current_module];
        if prefix.is_empty() {
            return name;
        }
        Symbol::intern(&format!("{prefix}.{name}"))
    }

    /// What a written name means here: an item of this module, or something it
    /// imported. An unknown name resolves to its own-module form so that the
    /// diagnostic names what the programmer wrote.
    fn resolve_name(&self, name: Symbol) -> Symbol {
        if let Some(&found) = self.visible[self.current_module].get(&name) {
            return found;
        }
        // A name that already carries a module prefix — one `resolve_qualified`
        // built from `a.b` — is what it says it is.
        if name.as_str().contains('.') {
            return name;
        }
        let prefix = &self.prefixes[self.current_module];
        if prefix.is_empty() { name } else { Symbol::intern(&format!("{prefix}.{name}")) }
    }

    /// `a.b` where `a` was bound by `import x.y.a` — the qualified name of
    /// `b` in that module.
    fn resolve_qualified(&self, segments: &[ast::Ident]) -> Option<Symbol> {
        if segments.len() != 2 {
            return None;
        }
        let module = *self.namespaces[self.current_module].get(&segments[0].name)?;
        let prefix = &self.prefixes[module];
        Some(if prefix.is_empty() {
            segments[1].name
        } else {
            Symbol::intern(&format!("{prefix}.{}", segments[1].name))
        })
    }

    /// Pass one: every item name in this module becomes visible to it, before
    /// any import is bound. `[MOD-4]` allows cycles, so this must happen for
    /// every module before any of them resolves anything.
    fn declare_names(&mut self, module: &ast::Module) {
        let index = self.current_module;
        for item in &module.items {
            // `[FFI-39]` parses (errata ERR-037 added the production) and
            // nothing past the parser understands it: Part XVI's C++ boundary
            // is Phase 7. Refusing it by name is the point — a declaration that
            // parses, is stored, and is then ignored by every later stage is
            // the shape three defects in this compiler have already taken, and
            // `extern class` would be the worst of them, since a base class
            // silently doing nothing produces a program that links and is
            // wrong.
            if let ast::ItemKind::ExternClass(decl) = &item.kind {
                let path = decl
                    .path
                    .iter()
                    .map(|s| s.name.to_string())
                    .collect::<Vec<_>>()
                    .join(".");
                self.sink.emit(
                    Diagnostic::error(
                        codes::E1010,
                        decl.span,
                        format!("`extern class {path}` is not supported yet in this phase"),
                    )
                    .help("declare an opaque `type` in an `extern` block if you only need a handle")
                    .note(concat!(
                        "a declared foreign base needs the C++ importer and a trampoline, ",
                        "which are Phase 7 [FFI-39]"
                    )),
                );
                continue;
            }
            let Some(name) = item_name(item) else { continue };
            let qualified = self.qualified(name);
            self.visible[index].insert(name, qualified);
        }
    }

    /// `[MOD-5]` — install only the source-backed names that the normative
    /// prelude exports. Merely loading `std.core` and `std.collections` must
    /// not leak every public declaration into user modules: in particular,
    /// `Hasher`, `DefaultHasher`, and the Arena collection types remain
    /// explicit imports. A local declaration wins, and an explicit import is
    /// bound afterwards and may deliberately replace the prelude spelling.
    fn bind_prelude(&mut self, modules: &[LoadedModule]) {
        const EXPORTS: &[(&str, &[&str])] = &[
            ("std.core", &["Eq", "Ord", "Default", "Iterator"]),
            ("std.collections", &["Hash"]),
        ];
        let by_path: HashMap<String, usize> =
            modules.iter().enumerate().map(|(i, m)| (m.path.join("."), i)).collect();

        for &(path, names) in EXPORTS {
            let Some(&module) = by_path.get(path) else { continue };
            for &name in names {
                let name = Symbol::intern(name);
                if !public_item(&modules[module].module, name)
                    .is_some_and(|visibility| visibility != ast::VisKind::Private)
                {
                    continue;
                }
                let qualified = Symbol::intern(&format!("{path}.{name}"));
                for visible in &mut self.visible {
                    visible.entry(name).or_insert(qualified);
                }
            }
        }
    }

    /// `[MOD-3]` — bind what each module imported. `[MOD-2]` — only a `pub`
    /// item may be imported.
    fn bind_imports(&mut self, modules: &[LoadedModule]) {
        let by_path: HashMap<String, usize> =
            modules.iter().enumerate().map(|(i, m)| (m.path.join("."), i)).collect();

        for (index, loaded) in modules.iter().enumerate() {
            self.current_module = index;
            for import in &loaded.module.imports {
                let (segments, binding) = match &import.kind {
                    ast::ImportKind::Module { path, alias } => (path, Some(alias)),
                    ast::ImportKind::Items { path, .. } => (path, None),
                    ast::ImportKind::Foreign { .. } => continue,
                };
                let key = segments
                    .iter()
                    .map(|s| s.name.to_string())
                    .collect::<Vec<_>>()
                    .join(".");
                let Some(&target) = by_path.get(&key) else {
                    // `[MOD-5]` — the prelude's names are compiler-known until
                    // the library can supply each one, so an import naming a
                    // `std` module with no file yet binds nothing rather than
                    // failing. A `std` module that *does* exist is resolved
                    // like any other, above.
                    if key == "std" || key.starts_with("std.") {
                        continue;
                    }
                    self.error(
                        codes::E1010,
                        import.span,
                        format!("cannot find module `{key}`"),
                    );
                    continue;
                };

                match (&import.kind, binding) {
                    // `import a.b.c [as d]` binds the last segment, or the
                    // alias, as a namespace.
                    (ast::ImportKind::Module { path, .. }, Some(alias)) => {
                        let bound = alias
                            .map(|a| a.name)
                            .or_else(|| path.last().map(|s| s.name));
                        if let Some(bound) = bound {
                            self.namespaces[index].insert(bound, target);
                        }
                    }
                    // `from a.b import x, y as z` binds each item by name.
                    (ast::ImportKind::Items { items, glob, .. }, _) => {
                        if *glob {
                            self.error(
                                codes::E1040,
                                import.span,
                                "`import *` is only allowed from a `@prelude` module",
                            );
                            continue;
                        }
                        for item in items {
                            let Some(vis) = public_item(&modules[target].module, item.name.name)
                            else {
                                self.error(
                                    codes::E1010,
                                    item.name.span,
                                    format!("`{key}` has no item named `{}`", item.name.name),
                                );
                                continue;
                            };
                            if vis == ast::VisKind::Private {
                                self.error(
                                    codes::E1020,
                                    item.name.span,
                                    format!("`{}` is private to `{key}`", item.name.name),
                                );
                                continue;
                            }
                            let prefix = &self.prefixes[target];
                            let qualified = if prefix.is_empty() {
                                item.name.name
                            } else {
                                Symbol::intern(&format!("{prefix}.{}", item.name.name))
                            };
                            let bound = item.alias.map(|a| a.name).unwrap_or(item.name.name);
                            self.visible[index].insert(bound, qualified);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // -- collection ---------------------------------------------------------

    /// Register every non-generic nominal type before any module resolves a
    /// field or signature. `[MOD-4]` permits import cycles, so module load
    /// order cannot be a semantic dependency.
    fn collect_type_headers(&mut self, module: &ast::Module) {
        for item in &module.items {
            match &item.kind {
                ast::ItemKind::Struct(decl) if decl.generics.is_empty() => {
                    // Every declared name is stored qualified, so two modules
                    // may both declare a `Point`.
                    let name = self.qualified(decl.name.name);
                    if self.named_types.contains_key(&name) {
                        self.error(
                            codes::E1030,
                            decl.name.span,
                            format!("`{name}` is already declared in this module"),
                        );
                        continue;
                    }
                    let id = self.types.add_struct(StructDef {
                        name,
                        fields: Vec::new(),
                        span: item.span,
                        derives_copy: has_derive(&item.attrs, "Copy"),
                        has_drop: false,
                        drops_fields: true,
                        origin: None,
                        declaring_module: self.current_module,
                    });
                    let ty = self.types.intern(TyKind::Struct(id));
                    self.struct_ids.insert(name, id);
                    self.named_types.insert(name, ty);
                }
                ast::ItemKind::Class(decl) if decl.generics.is_empty() => {
                    let name = self.qualified(decl.name.name);
                    if self.named_types.contains_key(&name) {
                        self.error(
                            codes::E1030,
                            decl.name.span,
                            format!("`{name}` is already declared in this module"),
                        );
                        continue;
                    }
                    let id = self.types.add_class(ClassDef {
                        name,
                        fields: Vec::new(),
                        span: item.span,
                        openness: class_openness(decl.openness),
                        base: None,
                        has_drop: false,
                        origin: None,
                        declaring_module: self.current_module,
                    });
                    let ty = self.types.intern(TyKind::Class(id));
                    self.class_ids.insert(name, id);
                    self.named_types.insert(name, ty);
                }
                ast::ItemKind::Enum(decl) => {
                    let name = self.qualified(decl.name.name);
                    if self.named_types.contains_key(&name) {
                        self.error(
                            codes::E1030,
                            decl.name.span,
                            format!("`{name}` is already declared in this module"),
                        );
                        continue;
                    }
                    let (repr, repr_is_explicit) = self.enum_repr(&item.attrs, decl);
                    let id = self.types.add_enum(EnumDef {
                        name,
                        variants: Vec::new(),
                        span: item.span,
                        repr,
                        repr_is_explicit,
                        derives_copy: has_derive(&item.attrs, "Copy"),
                        has_drop: false,
                    });
                    let ty = self.types.intern(TyKind::Enum(id));
                    self.enum_ids.insert(name, id);
                    self.named_types.insert(name, ty);
                }
                _ => {}
            }
        }
    }

    fn validate_class_bases(&mut self, module: &ast::Module) {
        for item in &module.items {
            let ast::ItemKind::Class(decl) = &item.kind else { continue };
            let Some(&id) = self.class_ids.get(&self.qualified(decl.name.name)) else {
                // Generic classes are outside the current collection path.
                continue;
            };
            let Some(base) = self.types.class_def(id).base else { continue };
            let base_def = self.types.class_def(base);
            if base_def.openness == ClassOpenness::Final {
                // There is no dedicated final-base diagnostic in the current
                // catalogue.  Keep the rejection on the existing type
                // diagnostic family rather than inventing an unregistered
                // error identity; the message names the precise CLS-4 rule.
                let base_name = base_def.name;
                self.error(
                    codes::E2020,
                    decl.base.as_ref().map_or(decl.name.span, |base| base.span),
                    format!(
                        "class `{}` cannot inherit from final class `{base_name}`",
                        decl.name.name
                    ),
                );
            }
        }
    }

    fn qualified_in_module(&self, module: usize, name: Symbol) -> Symbol {
        let prefix = &self.prefixes[module];
        if prefix.is_empty() {
            name
        } else {
            Symbol::intern(&format!("{prefix}.{name}"))
        }
    }

    fn declared_class_method_dispatch(
        &self,
        modules: &[LoadedModule],
        id: ClassId,
        name: Symbol,
    ) -> Option<ast::Dispatch> {
        for (module, loaded) in modules.iter().enumerate() {
            for item in &loaded.module.items {
                let ast::ItemKind::Class(decl) = &item.kind else { continue };
                if self.class_ids.get(&self.qualified_in_module(module, decl.name.name)) != Some(&id) {
                    continue;
                }
                for member in &decl.members {
                    if let ast::MemberKind::Fn(method) = &member.kind
                        && method.name.name == name
                    {
                        return Some(method.dispatch);
                    }
                }
            }
        }
        None
    }

    fn inherited_class_method_dispatch(
        &self,
        modules: &[LoadedModule],
        id: ClassId,
        name: Symbol,
    ) -> Option<ast::Dispatch> {
        let mut current = self.types.class_def(id).base;
        for _ in 0..=self.types.classes().count() {
            let Some(base) = current else { return None };
            if let Some(dispatch) = self.declared_class_method_dispatch(modules, base, name) {
                return Some(dispatch);
            }
            current = self.types.class_def(base).base;
        }
        None
    }

    /// CLS-4: override is meaningful only when it replaces a virtual method
    /// in the inherited class chain. This runs after all class declarations
    /// are collected, so a base in another module follows the same rule.
    fn validate_class_methods(&mut self, modules: &[LoadedModule]) {
        for (module, loaded) in modules.iter().enumerate() {
            for item in &loaded.module.items {
                let ast::ItemKind::Class(decl) = &item.kind else { continue };
                let Some(&id) = self.class_ids.get(&self.qualified_in_module(module, decl.name.name)) else {
                    continue;
                };
                for member in &decl.members {
                    let ast::MemberKind::Fn(method) = &member.kind else { continue };
                    if method.dispatch != ast::Dispatch::Override {
                        continue;
                    }
                    if self.inherited_class_method_dispatch(modules, id, method.name.name)
                        != Some(ast::Dispatch::Virtual)
                    {
                        self.error(
                            codes::E2110,
                            method.name.span,
                            "override of a method that is not virtual",
                        );
                    }
                }
            }
        }
    }

    /// `[TYP-16]` — collect generic struct recipes after nominal headers are
    /// globally visible and before any ordinary function signature is read.
    fn collect_generic_structs(&mut self, module: &ast::Module) {
        for (item_index, item) in module.items.iter().enumerate() {
            let ast::ItemKind::Struct(decl) = &item.kind else { continue };
            if decl.generics.is_empty() {
                continue;
            }
            let name = self.qualified(decl.name.name);
            let generic_params = self.declare_generics(&decl.generics);
            let params: Vec<Symbol> = generic_params.iter().map(|g| g.name).collect();
            let fields = decl
                .members
                .iter()
                .filter_map(|member| match &member.kind {
                    ast::MemberKind::Field(field) => Some(FieldDef {
                        name: field.name.name,
                        ty: self.resolve_type(&field.ty),
                        span: member.span,
                        has_default: field.default.is_some(),
                        read_only_outside: member.read_only_outside,
                        vis: field_vis(member.vis.kind),
                    }),
                    _ => None,
                })
                .collect();
            let mut methods = Vec::new();
            for (member_index, member) in decl.members.iter().enumerate() {
                let ast::MemberKind::Fn(fn_decl) = &member.kind else { continue };
                if fn_decl.body.is_none() {
                    continue;
                }
                let Some((receiver, signature)) = self.method_signature(
                    fn_decl,
                    None,
                    &member.attrs,
                    member.span,
                    params.len(),
                ) else {
                    continue;
                };
                methods.push(GenericMethod {
                    name: fn_decl.name.name,
                    receiver,
                    params: signature.params,
                    ret: signature.ret,
                    generics: signature.generics,
                    borrows: signature.borrows.map(|positions| {
                        positions
                            .into_iter()
                            .map(|position| position + usize::from(receiver.is_some()))
                            .collect()
                    }),
                    source: (self.current_module, item_index, member_index),
                    span: member.span,
                });
            }
            self.type_params.clear();
            self.generic_structs.insert(
                name,
                GenericStruct {
                    declaring_module: self.current_module,
                    params,
                    generic_params,
                    fields,
                    derives_copy: has_derive(&item.attrs, "Copy"),
                    methods,
                },
            );
        }
    }

    /// Resolve aliases, fields, variants, constants, statics, and function
    /// signatures after every type header and generic recipe exists.
    fn collect(&mut self, module: &ast::Module) {

        // Aliases are resolved between the two loops: they may name a struct or
        // enum (registered above) and may be named by a field (resolved below).
        self.collect_type_aliases(module);

        for item in &module.items {
            match &item.kind {
                ast::ItemKind::Struct(decl) => {
                    let Some(&id) = self.struct_ids.get(&self.qualified(decl.name.name)) else { continue };
                    let mut fields = Vec::new();
                    let mut has_drop = false;
                    for member in &decl.members {
                        match &member.kind {
                            ast::MemberKind::Field(field) => {
                                let ty = self.resolve_type(&field.ty);
                                fields.push(FieldDef {
                                    name: field.name.name,
                                    ty,
                                    span: member.span,
                                    has_default: field.default.is_some(),
                                    read_only_outside: member.read_only_outside,
                                    vis: field_vis(member.vis.kind),
                                });
                            }
                            // `[STR-3]` — a `drop` method makes the type
                            // move-only, whatever `@derive(Copy)` says.
                            ast::MemberKind::Fn(f) if f.name.name.is("drop") => has_drop = true,
                            _ => {}
                        }
                    }
                    // `[TYP-14]` — a struct carrying a borrow *is* a view
                    // type whatever it says; the attribute is required as
                    // documentation, because a reader has to know that the
                    // struct cannot be stored (`[TYP-15]`) without checking
                    // every field's type.
                    let carries_a_borrow: Vec<(Symbol, Span)> = fields
                        .iter()
                        .filter(|f| self.types.is_view(f.ty))
                        .map(|f| (f.name, f.span))
                        .collect();
                    if !carries_a_borrow.is_empty() && !has_attribute(&item.attrs, "view") {
                        let (field, field_span) = carries_a_borrow[0];
                        let name = decl.name.name;
                        self.sink.emit(
                            Diagnostic::error(
                                codes::E2030,
                                decl.name.span,
                                format!("`{name}` carries a borrow, so it is a view type"),
                            )
                            .secondary(field_span, format!("`{field}` is a borrow"))
                            .help("write `@view` on the declaration")
                            .note(concat!(
                                "a view type may live in a local, a parameter or a return ",
                                "value, and may not be stored in a field, a `static` or a ",
                                "container (TYP-15)"
                            )),
                        );
                    }

                    let def = self.types.struct_def_mut(id);
                    def.fields = fields;
                    def.has_drop = has_drop;

                    if has_derive(&item.attrs, "Copy") {
                        let ty = self.named_types[&self.qualified(decl.name.name)];
                        let offenders: Vec<(Symbol, Ty, Span)> = self
                            .types
                            .struct_def(id)
                            .fields
                            .iter()
                            .filter(|f| !self.types.is_copy(f.ty))
                            .map(|f| (f.name, f.ty, f.span))
                            .collect();
                        for (name, field_ty, span) in offenders {
                            let shown = self.types.display(field_ty);
                            self.error(
                                codes::E2080,
                                span,
                                format!("field `{name}` has type `{shown}`, which is not Copy"),
                            );
                        }
                        let _ = ty;
                    }
                }
                ast::ItemKind::Class(decl) => {
                    let Some(&id) = self.class_ids.get(&self.qualified(decl.name.name)) else {
                        // Generic classes are intentionally still outside
                        // this bootstrap collection path.
                        continue;
                    };
                    let fields = decl
                        .members
                        .iter()
                        .filter_map(|member| match &member.kind {
                            ast::MemberKind::Field(field) => Some(FieldDef {
                                name: field.name.name,
                                ty: self.resolve_type(&field.ty),
                                span: member.span,
                                has_default: field.default.is_some(),
                                read_only_outside: member.read_only_outside,
                                vis: field_vis(member.vis.kind),
                            }),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    let has_drop = decl.members.iter().any(|member| {
                        matches!(&member.kind, ast::MemberKind::Fn(f) if f.name.name.is("drop"))
                    });
                    let default_literals = decl
                        .members
                        .iter()
                        .filter_map(|member| match &member.kind {
                            ast::MemberKind::Field(field) => Some(
                                field.default.as_ref().and_then(|expr| match &expr.kind {
                                    ast::ExprKind::Lit(literal) => Some(literal.clone()),
                                    _ => None,
                                }),
                            ),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    let base = decl.base.as_ref().and_then(|base| {
                        let resolved = self.resolve_type(base);
                        match self.types.kind(resolved) {
                            TyKind::Class(base_id) => Some(*base_id),
                            _ if resolved == self.common.error => None,
                            _ => {
                                self.error(
                                    codes::E2020,
                                    base.span,
                                    "a class base must name another class",
                                );
                                None
                            }
                        }
                    });
                    let def = self.types.class_def_mut(id);
                    def.fields = fields;
                    def.base = base;
                    def.has_drop = has_drop;
                    self.class_default_literals.insert(id, default_literals);
                }
                ast::ItemKind::Enum(decl) => {
                    let Some(&id) = self.enum_ids.get(&self.qualified(decl.name.name)) else { continue };
                    let variants = self.collect_variants(decl);
                    let has_drop = decl.members.iter().any(|m| {
                        matches!(&m.kind, ast::MemberKind::Fn(f) if f.name.name.is("drop"))
                    });
                    let def = self.types.enum_def_mut(id);
                    def.variants = variants;
                    def.has_drop = has_drop;
                    self.check_enum_copy(id, &item.attrs);
                }
                // `const NAME: T = literal` — a name for a value, substituted
                // wherever it is used. Part XX.1 limits v1 to a literal until
                // comptime evaluation exists.
                ast::ItemKind::Const(decl) => {
                    let declared = decl.ty.as_ref().map(|t| self.resolve_type(t));
                    let value = match declared {
                        Some(ty) => self.check_expr(&decl.value, ty),
                        None => self.synth_committed(&decl.value),
                    };
                    if !matches!(
                        value.kind,
                        ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) | ExprKind::Str(_)
                    ) && value.ty != self.common.error
                    {
                        self.error(
                            codes::E2130,
                            decl.value.span,
                            "a `const` must be a literal in this phase of the compiler",
                        );
                        continue;
                    }
                    let const_name = self.qualified(decl.name.name);
                    if self.constants.contains_key(&const_name) {
                        self.error(
                            codes::E1030,
                            decl.name.span,
                            format!("`{}` is already declared in this module", decl.name.name),
                        );
                        continue;
                    }
                    self.constants.insert(const_name, value);
                }
                // `static NAME: T = literal` — one instance with a stable
                // address. `[STA-2]` — there are no runtime initialisers, so
                // the value has to be known here.
                ast::ItemKind::Static(decl) => {
                    let ty = self.resolve_type(&decl.ty);
                    // `[TYP-15]` with `[LT-3]` — a `static` has no bounding
                    // region, so what may be stored in one is a view whose
                    // region is `static`. `[STA-2]` already requires the
                    // initialiser to be a literal, so the question is decidable
                    // here and needs no region graph.
                    let static_region = has_static_region(&decl.value);
                    self.reject_stored_view_unless(
                        ty,
                        decl.ty.span,
                        "a `static`",
                        static_region,
                    );
                    let value = self.check_expr(&decl.value, ty);
                    if decl.is_mut {
                        self.error(
                            codes::E1010,
                            decl.name.span,
                            "`static mut` needs `unsafe`, which is not in this phase",
                        );
                        continue;
                    }
                    if !matches!(
                        value.kind,
                        ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) | ExprKind::Str(_)
                    ) && value.ty != self.common.error
                    {
                        self.error(
                            codes::E2130,
                            decl.value.span,
                            "a `static` initialiser must be comptime-evaluable, so a literal here",
                        );
                        continue;
                    }
                    let const_name = self.qualified(decl.name.name);
                    if self.constants.contains_key(&const_name) {
                        self.error(
                            codes::E1030,
                            decl.name.span,
                            format!("`{}` is already declared in this module", decl.name.name),
                        );
                        continue;
                    }
                    // With no runtime initialiser and no mutation, a `static`
                    // and a `const` behave identically; the difference is the
                    // stable address, which nothing can observe until
                    // references to globals exist.
                    self.constants.insert(const_name, value);
                }
                ast::ItemKind::Fn(decl) => {
                    let name = self.qualified(decl.name.name);
                    if self.fn_ids.contains_key(&name) {
                        // `[TYP-26]` — overloading is not supported.
                        self.error(
                            codes::E1030,
                            decl.name.span,
                            format!("`{name}` is already declared in this module"),
                        );
                        continue;
                    }
                    // `[TYP-16]` — the parameters are in scope for the
                    // signature as well as for the body.
                    let mut generics = self.declare_generics(&decl.generics);
                    let params: Vec<(Symbol, Ty, Mode, Span)> = decl
                        .params
                        .iter()
                        .filter_map(|p| match &p.kind {
                            ast::ParamKind::Named { name, ty } => {
                                let ty = self.callable_param_ty(ty, p.mode, &mut generics, 0);
                                Some((name.name, ty, mode_of(p.mode), p.span))
                            }
                            // A receiver outside a type body is meaningless;
                            // Phase 0 has no methods, so skip it.
                            ast::ParamKind::Receiver { .. } => None,
                        })
                        .collect();
                    let ret = decl
                        .ret
                        .as_ref()
                        .map(|t| self.resolve_type(t))
                        .unwrap_or(self.common.void);
                    let borrows = self.check_borrows_attribute(&item.attrs, &params, ret);
                    self.type_params.clear();
                    let def = DefId(self.signatures.len() as u32);
                    self.fn_ids.insert(name, def);
                    self.signatures.push(Signature { params, ret, generics, borrows });
                }
                _ => {}
            }
        }

    }

    /// `[TYP-12]` — the integer type the tag lives in. `@repr(u8)` names it;
    /// otherwise the compiler picks the smallest type up to 32 bits that holds
    /// every discriminant, which is what the spec's "`i32`-sized-or-smaller,
    /// chosen by the compiler" means.
    fn enum_repr(&mut self, attrs: &[ast::Attribute], decl: &ast::EnumDecl) -> (Ty, bool) {
        if let Some(name) = attr_argument(attrs, "repr") {
            if let Some(ty) = self.scalar_named(name.as_str()) {
                if self.types.is_integral(ty) {
                    return (ty, true);
                }
            }
            self.error(
                codes::E2020,
                decl.name.span,
                format!("`@repr({name})` is not an integer type"),
            );
        }

        // Walk the declared discriminants the same way `collect_variants`
        // will, so the chosen type is the one the values actually need.
        let mut next: i128 = 0;
        let mut lo: i128 = 0;
        let mut hi: i128 = 0;
        for variant in &decl.variants {
            if let Some(expr) = &variant.discriminant {
                if let Some(value) = literal_int(expr) {
                    next = value;
                }
            }
            lo = lo.min(next);
            hi = hi.max(next);
            next += 1;
        }
        let c = self.common;
        let ty = if lo >= 0 {
            if hi <= u8::MAX as i128 {
                c.u8
            } else if hi <= u16::MAX as i128 {
                c.u16
            } else {
                c.u32
            }
        } else if lo >= i8::MIN as i128 && hi <= i8::MAX as i128 {
            c.i8
        } else if lo >= i16::MIN as i128 && hi <= i16::MAX as i128 {
            c.i16
        } else {
            c.i32
        };
        (ty, false)
    }

    /// `[ENM-1]` — variants with their payloads. A payload field may be named;
    /// an unnamed one is `_0`, `_1`, … so that positional and named patterns
    /// are one lookup.
    fn collect_variants(&mut self, decl: &ast::EnumDecl) -> Vec<VariantDef> {
        let mut variants: Vec<VariantDef> = Vec::new();
        let mut next: i128 = 0;
        for variant in &decl.variants {
            if let Some(expr) = &variant.discriminant {
                match literal_int(expr) {
                    Some(value) => next = value,
                    None => self.error(
                        codes::E2131,
                        expr.span,
                        "a discriminant must be an integer literal in this phase of the compiler",
                    ),
                }
            }
            if variants.iter().any(|v| v.name == variant.name.name) {
                self.error(
                    codes::E1030,
                    variant.name.span,
                    format!("variant `{}` is declared twice", variant.name.name),
                );
                continue;
            }
            if let Some(other) = variants.iter().find(|v| v.discriminant == next) {
                let name = other.name;
                self.error(
                    codes::E1030,
                    variant.name.span,
                    format!("discriminant {next} is already used by `{name}`"),
                );
            }
            let fields = variant
                .fields
                .iter()
                .enumerate()
                .map(|(i, field)| FieldDef {
                    name: field.name.map(|n| n.name).unwrap_or_else(|| Symbol::intern(&format!("_{i}"))),
                    ty: self.resolve_type(&field.ty),
                    span: field.span,
                    has_default: false,
                    // `[MOD-7]`: "`read` applies to fields of structs and
                    // classes only" — a variant payload is neither, and a
                    // variant's payload is as visible as the enum.
                    read_only_outside: false,
                    vis: FieldVis::Public,
                })
                .collect();
            variants.push(VariantDef {
                name: variant.name.name,
                fields,
                discriminant: next,
                span: variant.span,
            });
            next += 1;
        }
        variants
    }

    /// `[ENM-4]` — a payload enum is `Copy` only via `@derive(Copy)` with all
    /// payloads `Copy`. A unit-only enum is `Copy` regardless (`[ENM-3]`), so
    /// there is nothing to check.
    fn check_enum_copy(&mut self, id: EnumId, attrs: &[ast::Attribute]) {
        if !has_derive(attrs, "Copy") {
            return;
        }
        let offenders: Vec<(Symbol, Ty, Span)> = self
            .types
            .enum_def(id)
            .variants
            .iter()
            .flat_map(|v| v.fields.iter())
            .filter(|f| !self.types.is_copy(f.ty))
            .map(|f| (f.name, f.ty, f.span))
            .collect();
        for (name, ty, span) in offenders {
            let shown = self.types.display(ty);
            self.error(
                codes::E2080,
                span,
                format!("payload `{name}` has type `{shown}`, which is not Copy"),
            );
        }
    }

    // -- interfaces and methods (block D) ------------------------------------

    /// Interfaces, across every module, before any `implements` is read.
    ///
    /// This ran per module beside the methods until 2026-09-09, so a module
    /// checked before the one declaring an interface could not `implement` it:
    /// `from lib import Eq` then `extend V implements Eq:` was "cannot find
    /// interface `Eq`", and which module won depended on the load order.
    /// `[MOD-4]` allows import cycles inside a package, so there is no order
    /// that would have made it work — the pass has to be whole-program, as the
    /// name pass above it already is.
    fn collect_interfaces(&mut self, module: &ast::Module) {
        for item in &module.items {
            if let ast::ItemKind::Interface(decl) = &item.kind {
                self.collect_interface(decl, item.span, item.vis.kind);
            }
        }
    }

    /// A third collection pass: every method on a type, with every interface
    /// already collected so that `implements` can name one.
    fn collect_methods(&mut self, module: &ast::Module) {
        for (item_index, item) in module.items.iter().enumerate() {
            match &item.kind {
                ast::ItemKind::Struct(decl) => {
                    let Some(&ty) = self.named_types.get(&self.qualified(decl.name.name)) else { continue };
                    self.collect_members(
                        ty,
                        &decl.members,
                        None,
                        item.vis.kind != ast::VisKind::Private,
                        item.span,
                        item_index,
                    );
                    self.collect_implements(ty, &decl.implements, &decl.members, item.span);
                }
                ast::ItemKind::Class(decl) => {
                    let Some(&ty) = self.named_types.get(&self.qualified(decl.name.name)) else { continue };
                    // `[CLS-4]` — virtual dispatch has no effect on a final
                    // class.  Keep the declaration accepted, but surface the
                    // existing warning so the author can either remove the
                    // modifier or opt the class into inheritance explicitly.
                    if decl.openness == ast::Openness::Final {
                        for member in &decl.members {
                            if let ast::MemberKind::Fn(method) = &member.kind
                                && method.dispatch == ast::Dispatch::Virtual
                            {
                                self.sink.emit(Diagnostic::warning(
                                    codes::W2111,
                                    method.name.span,
                                    "`virtual` has no effect in a final class",
                                ));
                            }
                        }
                    }
                    self.collect_members(
                        ty,
                        &decl.members,
                        None,
                        item.vis.kind != ast::VisKind::Private,
                        item.span,
                        item_index,
                    );
                    self.collect_implements(ty, &decl.implements, &decl.members, item.span);
                }
                ast::ItemKind::Enum(decl) => {
                    let Some(&ty) = self.named_types.get(&self.qualified(decl.name.name)) else { continue };
                    self.collect_members(
                        ty,
                        &decl.members,
                        None,
                        item.vis.kind != ast::VisKind::Private,
                        item.span,
                        item_index,
                    );
                    self.collect_implements(ty, &decl.implements, &decl.members, item.span);
                }
                ast::ItemKind::Extend(decl) => {
                    let ty = self.resolve_type(&decl.target);
                    if ty == self.common.error {
                        continue;
                    }
                    // `[IFC-1]` — `extend T:` with no `implements` adds
                    // inherent methods. With `implements`, the methods belong
                    // to that interface. Store the resolved interface identity,
                    // not the spelling written at this site: an explicit import
                    // and a prelude binding can give the same definition two
                    // spellings (`Ord` and `std.core.Ord`), but they must never
                    // become two interfaces during ambiguity checking.
                    let interface = decl
                        .implements
                        .first()
                        .and_then(interface_name)
                        .map(|name| self.resolve_name(name));
                    self.collect_members(
                        ty,
                        &decl.members,
                        interface,
                        true,
                        item.span,
                        item_index,
                    );
                    self.collect_implements(ty, &decl.implements, &decl.members, item.span);
                }
                _ => {}
            }
        }
        // A default body gives the implementing type a method it did not
        // write. These have to exist before any body is checked, or a call to
        // one would not resolve.
        self.register_defaults(module);
    }

    /// Every implementation has every method the interface requires, and every
    /// supertrait it names (`[IFC-3]`).
    fn check_implementations(&mut self) {
        for (ty, interface, span) in self.implemented.clone() {
            let Some(def) = self.interfaces.get(&interface) else { continue };
            let required = def.methods.clone();
            let supertraits = def.supertraits.clone();

            for (method, declaration, receiver, _) in required {
                let implementation = if receiver.is_some() {
                    self.methods.get(&(ty, method)).map(|entry| (entry.def, Some(entry.receiver)))
                } else {
                    self.associated.get(&(ty, method)).map(|entry| (entry.def, None))
                };
                let shown = self.types.display(ty);
                let Some((implementation, actual_receiver)) = implementation else {
                    self.error(
                        codes::E2040,
                        span,
                        format!("`{shown}` implements `{interface}` but does not define `{method}`"),
                    );
                    continue;
                };
                if !self.implementation_signature_matches(
                    ty,
                    declaration,
                    receiver,
                    implementation,
                    actual_receiver,
                ) {
                    self.error(
                        codes::E2040,
                        span,
                        format!(
                            "`{shown}.{method}` does not match the signature required by `{interface}`"
                        ),
                    );
                }
            }
            for parent in supertraits {
                if self.implemented.iter().any(|(t, i, _)| *t == ty && *i == parent) {
                    continue;
                }
                let shown = self.types.display(ty);
                self.error(
                    codes::E2040,
                    span,
                    format!(
                        "`{interface}` requires `{parent}`, which `{shown}` does not implement"
                    ),
                );
            }
        }
    }

    /// Interface conformance includes the complete callable contract, not
    /// merely the member name. In particular, Arena may call
    /// `Default.default()` without source-level arguments, so accepting a
    /// same-named function with parameters or the wrong result type would
    /// turn a type-checking omission into invalid MIR.
    fn implementation_signature_matches(
        &mut self,
        owner: Ty,
        declaration: DefId,
        required_receiver: Option<Mode>,
        implementation: DefId,
        actual_receiver: Option<Mode>,
    ) -> bool {
        if required_receiver != actual_receiver {
            return false;
        }

        let expected_generics = self.signatures[declaration.0 as usize].generics.clone();
        let actual_generics = self.signatures[implementation.0 as usize].generics.clone();
        if expected_generics.len() != actual_generics.len() {
            return false;
        }
        // Generic parameter names are not part of a callable signature. Map
        // both sides to one canonical parameter vector before comparing the
        // declared types, so `fn map[T]` and `fn map[U]` are alpha-equivalent.
        let canonical = expected_generics
            .iter()
            .enumerate()
            .map(|(index, param)| {
                self.types.intern(TyKind::Param {
                    index: index as u32,
                    name: param.name,
                })
            })
            .collect::<Vec<_>>();
        for (expected, actual) in expected_generics.iter().zip(&actual_generics) {
            let expected_bounds: HashSet<_> = expected.bounds.iter().copied().collect();
            let actual_bounds: HashSet<_> = actual.bounds.iter().copied().collect();
            if expected_bounds != actual_bounds
                || expected.callable.is_some() != actual.callable.is_some()
            {
                return false;
            }
            if let (Some(expected), Some(actual)) = (&expected.callable, &actual.callable) {
                if expected.once != actual.once || expected.params.len() != actual.params.len() {
                    return false;
                }
                for (expected, actual) in expected.params.iter().zip(&actual.params) {
                    if expected.mode != actual.mode {
                        return false;
                    }
                    let expected = self.types.substitute_self(expected.ty, owner);
                    let expected = self.resolve_assoc(expected, owner);
                    let expected = self.substitute_ty(expected, &canonical);
                    let actual = self.substitute_ty(actual.ty, &canonical);
                    if expected != actual {
                        return false;
                    }
                }
                let expected_ret = self.types.substitute_self(expected.ret, owner);
                let expected_ret = self.resolve_assoc(expected_ret, owner);
                let expected_ret = self.substitute_ty(expected_ret, &canonical);
                let actual_ret = self.substitute_ty(actual.ret, &canonical);
                if expected_ret != actual_ret {
                    return false;
                }
            }
        }

        let expected_params = self.signatures[declaration.0 as usize]
            .params
            .iter()
            .map(|(_, ty, mode, _)| (*ty, *mode))
            .collect::<Vec<_>>();
        let expected_ret = self.signatures[declaration.0 as usize].ret;
        let actual_params = self.signatures[implementation.0 as usize]
            .params
            .iter()
            .map(|(_, ty, mode, _)| (*ty, *mode))
            .collect::<Vec<_>>();
        let actual_ret = self.signatures[implementation.0 as usize].ret;

        let actual_written = if required_receiver.is_some() {
            let Some((self_ty, self_mode)) = actual_params.first() else { return false };
            if *self_ty != owner || Some(*self_mode) != required_receiver {
                return false;
            }
            &actual_params[1..]
        } else {
            actual_params.as_slice()
        };
        if expected_params.len() != actual_written.len() {
            return false;
        }

        for ((expected, expected_mode), (actual, actual_mode)) in
            expected_params.into_iter().zip(actual_written.iter().copied())
        {
            let expected = self.types.substitute_self(expected, owner);
            let expected = self.resolve_assoc(expected, owner);
            let expected = self.substitute_ty(expected, &canonical);
            let actual = self.substitute_ty(actual, &canonical);
            if expected != actual || expected_mode != actual_mode {
                return false;
            }
        }
        let expected_ret = self.types.substitute_self(expected_ret, owner);
        let expected_ret = self.resolve_assoc(expected_ret, owner);
        let expected_ret = self.substitute_ty(expected_ret, &canonical);
        let actual_ret = self.substitute_ty(actual_ret, &canonical);
        expected_ret == actual_ret
    }

    /// Register one method per (implementing type, defaulted interface method)
    /// that the type did not define itself.
    fn register_defaults(&mut self, module: &ast::Module) {
        let implementations = self.implemented.clone();
        for (item_index, item) in module.items.iter().enumerate() {
            let ast::ItemKind::Interface(decl) = &item.kind else { continue };
            let interface = self.qualified(decl.name.name);
            for (ty, _, _) in implementations.iter().filter(|(_, i, _)| *i == interface) {
                for (member_index, member) in decl.members.iter().enumerate() {
                    let ast::MemberKind::Fn(fn_decl) = &member.kind else { continue };
                    if fn_decl.body.is_none() {
                        continue;
                    }
                    let member_has_receiver = fn_decl
                        .params
                        .iter()
                        .any(|param| matches!(param.kind, ast::ParamKind::Receiver { .. }));
                    let already_exists = if member_has_receiver {
                        self.methods.contains_key(&(*ty, fn_decl.name.name))
                    } else {
                        self.associated.contains_key(&(*ty, fn_decl.name.name))
                    };
                    if already_exists {
                        continue;
                    }
                    let outer_self = self.self_ty.replace(*ty);
                    let signature =
                        self.method_signature(fn_decl, Some(*ty), &member.attrs, member.span, 0);
                    self.self_ty = outer_self;
                    let Some((receiver, signature)) = signature else {
                        continue;
                    };
                    let generic = !signature.generics.is_empty();
                    let registered = if let Some(receiver) = receiver {
                        self.register_method(
                            *ty,
                            fn_decl.name.name,
                            signature,
                            receiver,
                            Some(interface),
                            member.span,
                        )
                    } else {
                        self.register_associated(
                            *ty,
                            fn_decl.name.name,
                            signature,
                            Some(interface),
                            member.span,
                        )
                    };
                    if generic {
                        if let Some(def) = registered {
                            self.generic_method_sources.insert(
                                def,
                                MethodSource {
                                    owner: *ty,
                                    source: (self.current_module, item_index, member_index),
                                    owner_bindings: Vec::new(),
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    fn collect_interface(
        &mut self,
        decl: &ast::InterfaceDecl,
        span: Span,
        visibility: ast::VisKind,
    ) {
        // Part IV §8 — `Self` inside an `interface` is the implementing type,
        // which is unknown here, so it is a parameter until the interface is
        // used. `[TYP-22]` is what says a method returning it by value is not
        // `dyn`-compatible; statically it resolves like any other parameter.
        let outer_self = self.self_ty.replace(self.common.self_ty);
        self.collect_interface_inner(decl, span, visibility);
        self.self_ty = outer_self;
    }

    fn collect_interface_inner(
        &mut self,
        decl: &ast::InterfaceDecl,
        span: Span,
        visibility: ast::VisKind,
    ) {
        let name = self.qualified(decl.name.name);
        if self.interfaces.contains_key(&name) || self.named_types.contains_key(&name) {
            self.error(
                codes::E1030,
                decl.name.span,
                format!("`{name}` is already declared in this module"),
            );
            return;
        }
        // `[IFC-4]` — associated types first: a method signature may mention
        // one, so they have to be in scope before the signatures are read.
        let assoc: Vec<Symbol> = decl
            .members
            .iter()
            .filter_map(|m| match &m.kind {
                ast::MemberKind::TypeAlias(t) if t.value.is_none() => Some(t.name.name),
                _ => None,
            })
            .collect();
        let saved_assoc = std::mem::take(&mut self.assoc_scope);
        for name in &assoc {
            self.assoc_scope.insert(*name);
        }
        let mut methods = Vec::new();
        for member in &decl.members {
            let ast::MemberKind::Fn(f) = &member.kind else { continue };
            let Some((receiver, signature)) =
                self.method_signature(f, None, &member.attrs, member.span, 0)
            else {
                continue;
            };
            // A `DefId` for the declaration, so a call through a bound has a
            // signature to take its type from.
            let def = DefId(self.signatures.len() as u32);
            self.signatures.push(signature);
            if visibility != ast::VisKind::Private && member.vis.kind != ast::VisKind::Private {
                let signature = &self.signatures[def.0 as usize];
                let mut parameters = self.signature_parameters(signature);
                if let Some(mode) = receiver {
                    // Interface `Self` is deliberately outside ordinary
                    // function-generic binder space. Its explicit owner tag
                    // prevents it from being confused with a method-local
                    // `$P0` in the serialized declaration contract.
                    parameters.insert(
                        0,
                        CallableDeclarationParameter {
                            mode,
                            ty: CallableDeclarationType::Canonical(format!(
                                "interface:{name}::Self"
                            )),
                        },
                    );
                }
                self.member_callable_declarations.push(self.callable_declaration(
                    member.span,
                    member_declaration_symbol(&format!("interface:{name}"), f.name.name),
                    parameters,
                    signature.ret,
                    signature.borrows.clone(),
                    &signature.generics,
                    f.is_unsafe,
                    f.abi.clone(),
                ));
            }
            methods.push((f.name.name, def, receiver, f.body.is_some()));
        }
        // Resolved for the same reason the bounds and the implementation
        // record are: `interface Ord: Eq` in one module and
        // `implements Eq` in another name the same interface.
        let supertraits = decl
            .supertraits
            .iter()
            .filter_map(interface_name)
            .map(|name| self.resolve_name(name))
            .collect();
        let _ = span;
        self.assoc_scope = saved_assoc;
        // `[IFC-4]` — the associated names were in scope while the
        // signatures were read, which is all the declaration needs them for;
        // an implementation records what each one stands for.
        let _ = &assoc;
        self.interfaces.insert(name, InterfaceDef { methods, supertraits });
    }

    /// Turn a type/interface member into a signature. `self_ty` is `None`
    /// inside an interface, where `Self` remains the interface placeholder.
    /// A missing receiver denotes an associated function, not a malformed
    /// method; `Default.default()` is the first standard interface member
    /// whose semantics depend on this distinction.
    fn method_signature(
        &mut self,
        decl: &ast::FnDecl,
        self_ty: Option<Ty>,
        attrs: &[ast::Attribute],
        _span: Span,
        generic_index_base: usize,
    ) -> Option<(Option<Mode>, Signature)> {
        let saved_type_params = self.type_params.clone();
        let mut generics =
            self.declare_generics_from(&decl.generics, generic_index_base);
        let mut receiver = None;
        let mut params = Vec::new();
        for param in &decl.params {
            match &param.kind {
                ast::ParamKind::Receiver { .. } => {
                    if receiver.is_some() {
                        self.error(codes::E1030, param.span, "two receivers on one method");
                    }
                    receiver = Some(mode_of(param.mode));
                    if let Some(self_ty) = self_ty {
                        params.push((
                            Symbol::intern("self"),
                            self_ty,
                            mode_of(param.mode),
                            param.span,
                        ));
                    }
                }
                ast::ParamKind::Named { name, ty } => {
                    let ty = self.callable_param_ty(
                        ty,
                        param.mode,
                        &mut generics,
                        generic_index_base,
                    );
                    params.push((name.name, ty, mode_of(param.mode), param.span))
                }
            }
        }
        let ret = decl.ret.as_ref().map(|t| self.resolve_type(t)).unwrap_or(self.common.void);
        // `[LT-1a]` — the receiver is named as `self`, so a method's attribute
        // is resolved against the same parameter list the body will see.
        let borrows = self.check_borrows_attribute(attrs, &params, ret);
        self.type_params = saved_type_params;
        Some((receiver, Signature { params, ret, generics, borrows }))
    }

    /// Register every method in a type body or `extend` block.
    fn collect_members(
        &mut self,
        ty: Ty,
        members: &[ast::Member],
        from_interface: Option<Symbol>,
        owner_is_visible: bool,
        span: Span,
        item_index: usize,
    ) {
        // `Self` inside a type body or an `extend` block is that type.
        let outer_self = self.self_ty.replace(ty);
        self.collect_members_inner(
            ty,
            members,
            from_interface,
            owner_is_visible,
            span,
            item_index,
        );
        self.self_ty = outer_self;
    }

    fn collect_members_inner(
        &mut self,
        ty: Ty,
        members: &[ast::Member],
        from_interface: Option<Symbol>,
        owner_is_visible: bool,
        span: Span,
        item_index: usize,
    ) {
        // `[IFC-4]` — `type Item = i32` in an implementation says what the
        // interface's associated type is for this type.
        for member in members {
            let ast::MemberKind::TypeAlias(alias) = &member.kind else { continue };
            let Some(value) = &alias.value else { continue };
            let value = self.resolve_type(value);
            self.assoc_values.insert((ty, alias.name.name), value);
        }
        for (member_index, member) in members.iter().enumerate() {
            let ast::MemberKind::Fn(decl) = &member.kind else { continue };
            if decl.body.is_none() {
                continue;
            }
            let Some((receiver, signature)) =
                self.method_signature(decl, Some(ty), &member.attrs, member.span, 0)
            else {
                continue;
            };
            let generic = !signature.generics.is_empty();
            let registered = if let Some(receiver) = receiver {
                self.register_method(
                    ty,
                    decl.name.name,
                    signature,
                    receiver,
                    from_interface,
                    member.span,
                )
            } else {
                self.register_associated(
                    ty,
                    decl.name.name,
                    signature,
                    from_interface,
                    member.span,
                )
            };
            if generic {
                if let Some(def) = registered {
                    self.generic_method_sources.insert(
                        def,
                        MethodSource {
                            owner: ty,
                            source: (self.current_module, item_index, member_index),
                            owner_bindings: Vec::new(),
                        },
                    );
                }
            }
            if owner_is_visible && member.vis.kind != ast::VisKind::Private {
                if let Some(def) = registered {
                    let signature = &self.signatures[def.0 as usize];
                    self.member_callable_declarations.push(self.callable_declaration(
                        member.span,
                        method_symbol(&self.types.display(ty), decl.name.name),
                        self.signature_parameters(signature),
                        signature.ret,
                        signature.borrows.clone(),
                        &signature.generics,
                        decl.is_unsafe,
                        decl.abi.clone(),
                    ));
                }
            }
        }
        let _ = span;
    }

    fn register_method(
        &mut self,
        ty: Ty,
        name: Symbol,
        signature: Signature,
        receiver: Mode,
        from_interface: Option<Symbol>,
        span: Span,
    ) -> Option<DefId> {
        // `[TYP-26]` — one name per scope, except that an inherent method and
        // an interface method may coexist; `[TYP-24]` then prefers the
        // inherent one.
        if let Some(existing) = self.methods.get(&(ty, name)) {
            let shown = self.types.display(ty);
            match (&existing.from_interface, &from_interface) {
                (None, None) => {
                    self.error(
                        codes::E1030,
                        span,
                        format!("`{shown}` already has a method named `{name}`"),
                    );
                    return None;
                }
                // `[TYP-19]` — two interfaces offering the same name for one
                // type overlap; the call site reports `E2070` when it happens.
                (Some(_), Some(_)) => {}
                // An inherent method arriving later replaces the interface
                // one in the table, which is what `[TYP-24]` asks for.
                (Some(_), None) => {}
                (None, Some(_)) => return None,
            }
        }
        let def = DefId(self.signatures.len() as u32);
        self.signatures.push(signature);
        let _ = span;
        self.methods.insert((ty, name), MethodEntry { def, from_interface, receiver });
        Some(def)
    }

    fn register_associated(
        &mut self,
        ty: Ty,
        name: Symbol,
        signature: Signature,
        from_interface: Option<Symbol>,
        span: Span,
    ) -> Option<DefId> {
        if let Some(existing) = self.associated.get(&(ty, name)) {
            let shown = self.types.display(ty);
            match (&existing.from_interface, &from_interface) {
                (None, None) => {
                    self.error(
                        codes::E1030,
                        span,
                        format!("`{shown}` already has an associated function named `{name}`"),
                    );
                    return None;
                }
                (Some(_), Some(_)) | (Some(_), None) => {}
                (None, Some(_)) => return None,
            }
        }
        let def = DefId(self.signatures.len() as u32);
        self.signatures.push(signature);
        self.associated.insert((ty, name), AssociatedEntry { def, from_interface });
        Some(def)
    }

    /// Record which interfaces a type implements, and check that every
    /// required method is present.
    fn collect_implements(
        &mut self,
        ty: Ty,
        implements: &[ast::TypeExpr],
        members: &[ast::Member],
        span: Span,
    ) {
        for entry in implements {
            let Some(written) = interface_name(entry) else {
                self.error(codes::E1010, entry.span, "expected an interface name");
                continue;
            };
            // Recorded under the name the interface is registered by, so that
            // a bound written `T: Ord` on an imported `Ord` matches the
            // implementation written `implements Ord` in another module.
            let name = self.resolve_name(written);
            if !self.interfaces.contains_key(&name) {
                self.error(
                    codes::E1010,
                    entry.span,
                    format!("cannot find interface `{written}` in this scope"),
                );
                continue;
            }
            // `[TYP-19]` — the same interface implemented twice for one type.
            if self.implemented.iter().any(|(t, i, _)| *t == ty && *i == name) {
                let shown = self.types.display(ty);
                self.error(
                    codes::E2041,
                    entry.span,
                    format!("`{shown}` already implements `{written}`"),
                );
                continue;
            }
            // The required methods are checked once everything is collected:
            // a header may declare `implements` and a later `extend` block
            // supply the methods.
            self.implemented.push((ty, name, entry.span));
        }
        let _ = (members, span);
    }

    fn resolve_type(&mut self, ty: &ast::TypeExpr) -> Ty {
        match &ty.kind {
            ast::TypeKind::Void => self.common.void,
            ast::TypeKind::Never => self.common.never,
            // Part IV §8 — `interface Clone: fn clone(self) -> Self`. Inside a
            // type body or an `extend` block `Self` is that type; inside an
            // interface it is the implementing type, which is not known until
            // the interface is used, so it is a parameter until then.
            ast::TypeKind::SelfType => match self.self_ty {
                Some(ty) => ty,
                None => {
                    self.error(
                        codes::E2020,
                        ty.span,
                        "`Self` names the type being declared, and there is none here",
                    );
                    self.common.error
                }
            },
            ast::TypeKind::Ref { mutable, inner } => {
                let inner = self.resolve_type(inner);
                self.types.intern(TyKind::Ref { mutable: *mutable, inner })
            }
            ast::TypeKind::Ptr { mutable, inner } => {
                let inner = self.resolve_type(inner);
                self.types.intern(TyKind::Ptr { mutable: *mutable, inner })
            }
            ast::TypeKind::Tuple(items) => {
                let items: Vec<Ty> = items.iter().map(|t| self.resolve_type(t)).collect();
                self.types.intern(TyKind::Tuple(items))
            }
            // `[FN-6]` — "Functions are values of a unique zero-sized function
            // type; they coerce to `fn(A) -> R`". The written type is the
            // coercion target, which every named function fits and which
            // `[CLO-3]`'s closure parameter is a generic over.
            ast::TypeKind::Fn { abi, latebound, params, ret } => {
                if abi.is_some() {
                    // `extern "C" fn(…)` is `[FFI-9]`'s raw function pointer,
                    // which arrives with the rest of the boundary in Phase 5.
                    self.error(
                        codes::E1010,
                        ty.span,
                        "an `extern` function type is not supported yet in this phase",
                    );
                    return self.common.error;
                }
                let params = params
                    .iter()
                    .map(|param| FnParam {
                        ty: self.resolve_type(&param.ty),
                        mode: fn_param_mode(param.mode),
                    })
                    .collect();
                let ret = ret
                    .as_ref()
                    .map(|t| self.resolve_type(t))
                    .unwrap_or(self.common.void);
                self.types.intern(TyKind::Fn { latebound: *latebound, params, ret })
            }
            ast::TypeKind::Array { elem, len } => {
                let elem = self.resolve_type(elem);
                match self.const_len(len) {
                    Some(len) => self.types.intern(TyKind::Array { elem, len }),
                    None => self.common.error,
                }
            }
            // `[ERR-1]` — `Option[T]` and `Result[T, E]` are ordinary payload
            // enums, synthesised on demand. Part XX.1 makes them compiler-known
            // until Phase 2 gives the standard library generics of its own.
            ast::TypeKind::Path { segments, args }
                if segments.len() == 1 && !args.is_empty() =>
            {
                let name = segments[0].name;
                let mut resolved = Vec::with_capacity(args.len());
                for arg in args {
                    let ast::GenericArg::Type(t) = arg else {
                        let arg_span = match arg {
                            ast::GenericArg::Const(expr) => expr.span,
                            ast::GenericArg::Assoc { name, .. } => name.span,
                            ast::GenericArg::Type(_) => unreachable!(),
                        };
                        self.error(codes::E1010, arg_span, "expected a type argument");
                        return self.common.error;
                    };
                    resolved.push((self.resolve_type(t), t.span));
                }
                self.resolve_type_application(name, &resolved, ty.span)
            }

            ast::TypeKind::Path { segments, args } if args.is_empty() && segments.len() == 1 => {
                let name = segments[0].name;
                // A type parameter shadows everything: inside `fn f[T]`, `T`
                // is the parameter.
                if self.assoc_scope.contains(&name) {
                    return self.types.intern(TyKind::Assoc { name });
                }
                if let Some(&ty) = self.type_params.get(&name) {
                    return ty;
                }
                if let Some(ty) = self.scalar_named(name.as_str()) {
                    return ty;
                }
                // `String` is a growable buffer of UTF-8 bytes: `Array[u8]`
                // under a different name.
                if name.is("String") {
                    let u8_ty = self.common.u8;
                    return self.types.intern(TyKind::Vec { elem: u8_ty });
                }
                if name.is("Arena") {
                    return self.arena_ty();
                }
                if name.is("FixedArena") {
                    return self.fixed_arena_ty();
                }
                if name.is("ScopedArena") {
                    return self.scoped_arena_ty();
                }
                if let Some(&ty) = self.named_types.get(&self.resolve_name(name)) {
                    return ty;
                }
                self.error(
                    codes::E1010,
                    segments[0].span,
                    format!("cannot find type `{name}` in this scope"),
                );
                self.common.error
            }
            _ => {
                // Generics, `dyn`, arrays and function types arrive in later
                // phases; Phase 0 reports rather than guessing.
                self.error(
                    codes::E1010,
                    ty.span,
                    "this type is not supported yet in this phase of the compiler",
                );
                self.common.error
            }
        }
    }

    /// Resolve a named type application after bracket ambiguity has been
    /// settled. Both written type syntax (`Array[Cell[i32]]`) and the
    /// expression-shaped syntax preserved by `[GRM-8]` use this single path.
    fn resolve_type_application(&mut self, name: Symbol, args: &[(Ty, Span)], span: Span) -> Ty {
        let require = |this: &mut Self, expected: usize| {
            if args.len() == expected {
                true
            } else {
                let suffix = if expected == 1 { "one type argument".to_owned() } else {
                    format!("{expected} type arguments, found {}", args.len())
                };
                this.error(codes::E2020, span, format!("`{name}` takes {suffix}"));
                false
            }
        };

        // Compiler-known generic types share ordinary type-application
        // semantics; only their representation and invariants are special.
        if name.is("Span") || name.is("MutSpan") {
            if !require(self, 1) {
                return self.common.error;
            }
            let (elem, arg_span) = args[0];
            self.reject_stored_view(elem, arg_span, "a span element");
            return self.types.intern(TyKind::Span { elem, mutable: name.is("MutSpan") });
        }
        if name.is("Array") {
            if !require(self, 1) {
                return self.common.error;
            }
            let (elem, arg_span) = args[0];
            self.reject_stored_view(elem, arg_span, "a container element");
            return self.types.intern(TyKind::Vec { elem });
        }
        if name.is("Box") {
            if !require(self, 1) {
                return self.common.error;
            }
            // `[TYP-15]` is a value-region rule, not a prohibition on forming
            // the type: `Box[str]` is legal when the stored `str` has the
            // static region. Construction/storage checks the actual value.
            let (inner, _) = args[0];
            return self.box_of(inner);
        }
        if name.is("Cell") || name.is("RefCell") {
            if !require(self, 1) {
                return self.common.error;
            }
            let (inner, arg_span) = args[0];
            self.reject_stored_view(inner, arg_span, "a cell's contents");
            return if name.is("Cell") { self.cell_of(inner) } else { self.refcell_of(inner) };
        }
        if name.is("MaybeUninit") {
            if !require(self, 1) {
                return self.common.error;
            }
            return self.maybe_uninit_of(args[0].0);
        }
        if self.is_unsafe_cell_name(name) {
            if !require(self, 1) {
                return self.common.error;
            }
            let (inner, arg_span) = args[0];
            self.reject_stored_view(inner, arg_span, "an UnsafeCell's contents");
            let ty = self.unsafe_cell_of(inner);
            self.reject_unsafe_cell_in_static_safe(span);
            return ty;
        }
        if name.is("Ref") || name.is("RefMut") {
            if !require(self, 1) {
                return self.common.error;
            }
            return self.ref_guard_of(args[0].0, name.is("RefMut"));
        }
        let resolved_name = self.resolve_name(name);
        if let Some(decl) = self.generic_structs.get(&resolved_name).cloned() {
            let resolved = args.iter().map(|(ty, _)| *ty).collect::<Vec<_>>();
            return self.instantiate_struct(resolved_name, &decl, &resolved, span);
        }
        let arity = if name.is("Option") {
            1
        } else if name.is("Result") {
            2
        } else {
            let message = if self.named_types.contains_key(&resolved_name) {
                format!("`{name}` does not take type arguments in this phase")
            } else {
                format!("cannot find type `{name}` in this scope")
            };
            self.error(
                codes::E1010,
                span,
                message,
            );
            return self.common.error;
        };
        if !require(self, arity) {
            return self.common.error;
        }
        if name.is("Option") {
            self.option_of(args[0].0)
        } else {
            self.result_of(args[0].0, args[1].0)
        }
    }

    /// The `N` of `[T; N]`. Part IV.3 makes it a const generic; until const
    /// generics and `const` items exist it has to be written as a literal,
    /// and anything else is reported rather than guessed at.
    fn const_len(&mut self, expr: &ast::Expr) -> Option<u64> {
        let mut expr = expr;
        while let ast::ExprKind::Paren(inner) = &expr.kind {
            expr = inner;
        }
        if let ast::ExprKind::Lit(ast::Literal::Int { value, .. }) = &expr.kind {
            if let Ok(len) = u64::try_from(*value) {
                return Some(len);
            }
        }
        // A `const` is a value known at compile time, so it may be a length.
        if let ast::ExprKind::Path { segments } = &expr.kind {
            if segments.len() == 1 {
                if let Some(ExprKind::Int(value)) =
                    self.constants.get(&segments[0].name).map(|c| &c.kind)
                {
                    if let Ok(len) = u64::try_from(*value) {
                        return Some(len);
                    }
                }
            }
        }
        self.error(
            codes::E2131,
            expr.span,
            "an array length must be an integer literal or a `const` in this phase",
        );
        None
    }

    /// Whether the runtime has a formatter for this type. `Display` replaces
    /// this once interfaces carry generics.
    fn is_formattable(&self, ty: Ty) -> bool {
        matches!(
            self.types.kind(ty),
            TyKind::Bool
                | TyKind::Char
                | TyKind::Int(_)
                | TyKind::Uint(_)
                | TyKind::Float(_)
                | TyKind::Str
        ) || matches!(self.types.kind(ty), TyKind::Vec { elem } if matches!(self.types.kind(*elem), TyKind::Uint(UintTy::U8)))
    }

    /// Whether a type is one of the synthesised `Option`s.
    fn is_option(&self, ty: Ty) -> bool {
        match self.types.kind(ty) {
            TyKind::Enum(id) => self.types.enum_def(*id).name.as_str().starts_with("Option_"),
            _ => false,
        }
    }

    fn is_result(&self, ty: Ty) -> bool {
        match self.types.kind(ty) {
            TyKind::Enum(id) => self.types.enum_def(*id).name.as_str().starts_with("Result_"),
            _ => false,
        }
    }

    /// `[ERR-2]` — `e?` yields the success payload, or returns the failure
    /// from the enclosing function. It becomes a two-arm `match`: one arm
    /// gives the value, the other returns.
    fn synth_try(&mut self, inner: &ast::Expr, span: Span) -> Expr {
        let value = self.synth_committed(inner);
        let ret_ty = self.ret_ty;
        let is_option = self.is_option(value.ty);
        let is_result = self.is_result(value.ty);

        if !is_option && !is_result {
            if value.ty != self.common.error {
                let shown = self.types.display(value.ty);
                self.error(codes::E2180, span, format!("`?` needs an `Option` or a `Result`, not `{shown}`"));
            }
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        // `[ERR-2]` — the enclosing function has to be able to carry the
        // failure onwards.
        if (is_option && !self.is_option(ret_ty)) || (is_result && !self.is_result(ret_ty)) {
            let shown = self.types.display(value.ty);
            let returning = self.types.display(ret_ty);
            self.error(
                codes::E2180,
                span,
                format!("`?` on a `{shown}` needs the function to return one too, not `{returning}`"),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }

        let TyKind::Enum(id) = *self.types.kind(value.ty) else { unreachable!() };
        // `Option` is `None, Some`; `Result` is `Ok, Err`. The success variant
        // is the one carrying the payload in both.
        let (success, failure) = if is_option { (1usize, 0usize) } else { (0usize, 1usize) };
        let payload_ty = self.types.enum_def(id).variants[success].fields[0].ty;

        // The success arm binds the payload and yields it.
        let bound = self.declare(None, payload_ty, span);
        let ok_arm = hir::MatchArm {
            pattern: hir::Pattern {
                ty: value.ty,
                kind: hir::PatternKind::Variant {
                    enum_id: id,
                    variant: success,
                    fields: vec![hir::Pattern {
                        ty: payload_ty,
                        kind: hir::PatternKind::Bind { local: bound, sub: None },
                        span,
                    }],
                },
                span,
            },
            guard: None,
            body: hir::MatchArmBody::Expr(Expr {
                ty: payload_ty,
                kind: ExprKind::Local(bound),
                span,
            }),
            span,
        };

        // The failure arm rebuilds the failure in the return type and leaves.
        let TyKind::Enum(ret_id) = *self.types.kind(ret_ty) else { unreachable!() };
        let failure_fields = &self.types.enum_def(id).variants[failure].fields;
        let carried: Vec<Ty> = failure_fields.iter().map(|f| f.ty).collect();
        let ret_failure = &self.types.enum_def(ret_id).variants[failure].fields;
        let ret_carried: Vec<Ty> = ret_failure.iter().map(|f| f.ty).collect();
        if carried != ret_carried {
            let from = self.types.display(value.ty);
            let to = self.types.display(ret_ty);
            self.error(
                codes::E2180,
                span,
                format!("`?` cannot carry a `{from}` failure out of a function returning `{to}`"),
            );
            self.error_note_from_conversion(span);
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let mut binds = Vec::new();
        let mut reads = Vec::new();
        for &ty in &carried {
            let local = self.declare(None, ty, span);
            binds.push(hir::Pattern {
                ty,
                kind: hir::PatternKind::Bind { local, sub: None },
                span,
            });
            reads.push(Expr { ty, kind: ExprKind::Local(local), span });
        }
        let err_arm = hir::MatchArm {
            pattern: hir::Pattern {
                ty: value.ty,
                kind: hir::PatternKind::Variant { enum_id: id, variant: failure, fields: binds },
                span,
            },
            guard: None,
            body: hir::MatchArmBody::Block(Block {
                stmts: vec![Stmt::Return(Some(Expr {
                    ty: ret_ty,
                    kind: ExprKind::EnumLit {
                        enum_id: ret_id,
                        variant: failure,
                        fields: reads,
                    },
                    span,
                }))],
                span,
            }),
            span,
        };

        Expr {
            ty: payload_ty,
            kind: ExprKind::Match {
                scrutinee: Box::new(value),
                arms: vec![ok_arm, err_arm],
            },
            span,
        }
    }

    /// `[ERR-2]` says the failure is converted with `From`. Without generics
    /// there is no `From`, so the types have to match exactly and the
    /// diagnostic says so.
    fn error_note_from_conversion(&mut self, span: Span) {
        self.sink.emit(
            Diagnostic::error(
                codes::E2180,
                span,
                "the failure types must match exactly in this phase of the compiler",
            )
            .note("`[ERR-2]`'s `F.from(e)` conversion needs `From`, which needs generics"),
        );
    }

    /// `[OWN-6]` — `std.mem.take`, `replace`, `swap`, and `forget` are the sanctioned
    /// operations that mutate a place while transferring its old ownership
    /// without first dropping it. They are compiler-known while `std.mem` is
    /// staged, but their operands still use ordinary place, mutability,
    /// generic-inference, `Default`, borrow, and move rules.
    fn synth_mem_ownership_builtin(
        &mut self,
        name: Symbol,
        args: &[ast::Arg],
        explicit: &[Ty],
        span: Span,
    ) -> Option<Expr> {
        let resolved = self.resolve_name(name);
        let operation = if resolved.is("std.mem.replace") {
            "replace"
        } else if resolved.is("std.mem.take") {
            "take"
        } else if resolved.is("std.mem.swap") {
            "swap"
        } else if resolved.is("std.mem.forget") {
            "forget"
        } else {
            return None;
        };

        if explicit.len() > 1 {
            self.error(
                codes::E2020,
                span,
                format!("`mem.{operation}` takes one type argument, found {}", explicit.len()),
            );
            return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
        }
        let arity = if operation == "replace" || operation == "swap" { 2 } else { 1 };
        if args.len() != arity {
            self.error(
                codes::E2020,
                span,
                format!("`mem.{operation}` takes {arity} arguments, found {}", args.len()),
            );
            return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
        }

        let first = match explicit.first().copied() {
            Some(expected) => self.check_expr(&args[0].value, expected),
            None => self.synth_committed(&args[0].value),
        };
        if first.ty == self.common.error {
            return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
        }
        let elem = explicit.first().copied().unwrap_or(first.ty);
        if operation == "forget" {
            return Some(Expr {
                ty: self.common.void,
                kind: ExprKind::Builtin {
                    which: Builtin::MemForget { elem },
                    args: vec![first],
                },
                span,
            });
        }
        if !is_place(&first.kind) {
            self.error(
                codes::E2140,
                args[0].value.span,
                format!("`mem.{operation}` needs a mutable place as its first argument"),
            );
            return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
        }
        let first = self.pass_receiver(first, Mode::Mut, args[0].value.span);

        let result = match operation {
            "replace" => {
                let value = self.check_expr(&args[1].value, elem);
                Expr {
                    ty: elem,
                    kind: ExprKind::Builtin {
                        which: Builtin::MemReplace { elem },
                        args: vec![first, value],
                    },
                    span,
                }
            }
            "take" => {
                let shown = self.types.display(elem);
                let Some(constructor) = self.standard_associated_capability(
                    elem,
                    "std.core.Default",
                    "default",
                ) else {
                    self.sink.emit(
                        Diagnostic::error(
                            codes::E2040,
                            span,
                            format!(
                                "`{shown}` does not implement `std.core.Default`, which `T` requires"
                            ),
                        )
                        .help("implement `std.core.Default` or use `mem.replace(place, value)`")
                        .note("`mem.take` must leave an initialized value in the source place [OWN-6]"),
                    );
                    return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
                };
                Expr {
                    ty: elem,
                    kind: ExprKind::Builtin {
                        which: Builtin::MemTake { elem, constructor },
                        args: vec![first],
                    },
                    span,
                }
            }
            _ => {
                let second = self.check_expr(&args[1].value, elem);
                if !is_place(&second.kind) {
                    self.error(
                        codes::E2140,
                        args[1].value.span,
                        "`mem.swap` needs a mutable place as its second argument",
                    );
                    return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
                }
                let second = self.pass_receiver(second, Mode::Mut, args[1].value.span);
                Expr {
                    ty: self.common.void,
                    kind: ExprKind::Builtin {
                        which: Builtin::MemSwap { elem },
                        args: vec![first, second],
                    },
                    span,
                }
            }
        };
        Some(result)
    }

    /// `[UNS-5]` — `std.mem`'s raw primitives: `alloc[T]`, `free[T]`,
    /// `read[T]`, `write[T]`, `size_of[T]`, and `align_of[T]`. The two layout
    /// queries need no `unsafe`; the raw-memory operations do (`[UNS-1]`).
    fn synth_memory_builtin(
        &mut self,
        name: Symbol,
        args: &[ast::Arg],
        explicit: &[Ty],
        span: Span,
    ) -> Option<Expr> {
        if let Some(built) = self.synth_mem_ownership_builtin(name, args, explicit, span) {
            return Some(built);
        }
        let usize_ty = self.common.usize;
        let void = self.common.void;
        let resolved = self.resolve_name(name);
        let (which, arity, needs_unsafe) = if name.is("alloc") {
            (Builtin::MemAlloc, 1, true)
        } else if name.is("free") {
            (Builtin::MemFree, 2, true)
        } else if name.is("read") {
            (Builtin::PtrRead, 2, true)
        } else if name.is("write") {
            (Builtin::PtrWrite, 3, true)
        } else if name.is("size_of") || resolved.is("std.mem.size_of") {
            (Builtin::SizeOf, 0, false)
        } else if name.is("align_of") || resolved.is("std.mem.align_of") {
            (Builtin::AlignOf, 0, false)
        } else {
            return None;
        };

        if needs_unsafe && !self.in_unsafe {
            self.sink.emit(
                Diagnostic::error(
                    codes::E3100,
                    span,
                    format!("`{name}` needs an `unsafe` block"),
                )
                .note("`[UNS-1]` lists the operations that do")
                .help("wrap the call in `unsafe:`"),
            );
        }
        if args.len() != arity {
            self.error(
                codes::E2020,
                span,
                format!("`{name}` takes {arity} arguments, found {}", args.len()),
            );
            return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
        }

        // `alloc[T]` and the layout queries say their element type; `free`,
        // `read`, and `write` take it from the pointer they are given.
        let mut checked: Vec<Expr> = Vec::new();
        let elem = match which {
            Builtin::MemAlloc | Builtin::SizeOf | Builtin::AlignOf => match explicit.first() {
                Some(&ty) => ty,
                None => {
                    self.error(
                        codes::E2060,
                        span,
                        format!("write the type: `{name}[T](...)`"),
                    );
                    return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
                }
            },
            _ => {
                let pointer = self.synth_committed(&args[0].value);
                let elem = match self.types.kind(pointer.ty) {
                    TyKind::Ptr { inner, .. } => *inner,
                    _ => {
                        let shown = self.types.display(pointer.ty);
                        self.error(
                            codes::E2020,
                            args[0].value.span,
                            format!("`{name}` needs a raw pointer, not `{shown}`"),
                        );
                        return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
                    }
                };
                checked.push(pointer);
                elem
            }
        };

        let (rest, ret) = match which {
            Builtin::MemAlloc => {
                let ptr = self.types.intern(TyKind::Ptr { mutable: true, inner: elem });
                (vec![usize_ty], ptr)
            }
            Builtin::MemFree => (vec![usize_ty], void),
            Builtin::PtrRead => (vec![usize_ty], elem),
            Builtin::PtrWrite => (vec![usize_ty, elem], void),
            _ => (Vec::new(), usize_ty),
        };
        for (arg, &param_ty) in args.iter().skip(checked.len()).zip(rest.iter()) {
            let value = self.check_expr(&arg.value, param_ty);
            checked.push(value);
        }
        // Layout queries have no arguments, so the element type has to travel
        // somewhere; a zero-sized placeholder of that type carries it through
        // generic substitution.
        if matches!(which, Builtin::SizeOf | Builtin::AlignOf) {
            checked.push(Expr { ty: elem, kind: ExprKind::Error, span });
        }
        Some(Expr { ty: ret, kind: ExprKind::Builtin { which, args: checked }, span })
    }

    /// `Some(x)`, `Ok(x)`, `Err(e)`. `None` has no payload and is handled
    /// where a bare path is checked.
    fn synth_wrapper(
        &mut self,
        name: Symbol,
        args: &[ast::Arg],
        expected: Option<Ty>,
        span: Span,
    ) -> Option<Expr> {
        let which = if name.is("Some") {
            0
        } else if name.is("Ok") {
            1
        } else if name.is("Err") {
            2
        } else {
            return None;
        };
        if args.len() != 1 {
            self.error(codes::E2020, span, format!("`{name}` takes one value"));
            return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
        }

        // An expected `Option`/`Result` fixes both parameters, which is what
        // makes `Err(e)` work without naming the success type.
        let payload_ty = expected.and_then(|e| match self.types.kind(e) {
            TyKind::Enum(id) => {
                let def = self.types.enum_def(*id);
                let variant = match which {
                    0 => "Some",
                    1 => "Ok",
                    _ => "Err",
                };
                def.variant(Symbol::intern(variant)).and_then(|(_, v)| v.fields.first().map(|f| f.ty))
            }
            _ => None,
        });
        let value = match payload_ty {
            Some(ty) => self.check_expr(&args[0].value, ty),
            None => self.synth_committed(&args[0].value),
        };

        let ty = match (expected, which) {
            (Some(e), _) if matches!(self.types.kind(e), TyKind::Enum(_)) => e,
            (_, 0) => self.option_of(value.ty),
            // Without an expectation the other parameter is unknown; the void
            // placeholder keeps the error local instead of poisoning the call.
            (_, 1) => {
                let void = self.common.void;
                self.result_of(value.ty, void)
            }
            _ => {
                let void = self.common.void;
                self.result_of(void, value.ty)
            }
        };
        let TyKind::Enum(id) = *self.types.kind(ty) else {
            return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
        };
        let variant_name = match which {
            0 => "Some",
            1 => "Ok",
            _ => "Err",
        };
        let Some((index, _)) = self.types.enum_def(id).variant(Symbol::intern(variant_name)) else {
            let shown = self.types.display(ty);
            self.error(codes::E2020, span, format!("`{shown}` has no `{variant_name}`"));
            return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
        };
        Some(Expr {
            ty,
            kind: ExprKind::EnumLit { enum_id: id, variant: index, fields: vec![value] },
            span,
        })
    }

    /// Substitution that also rebuilds an instantiated generic struct:
    /// `Buffer[T]` with `T = i32` is `Buffer[i32]`, a different struct with a
    /// different layout, not the same one with its fields rewritten.
    fn substitute_ty(&mut self, ty: Ty, args: &[Ty]) -> Ty {
        match self.types.kind(ty).clone() {
            TyKind::Param { index, .. } => args.get(index as usize).copied().unwrap_or(ty),
            TyKind::Ref { mutable, inner } => {
                let inner = self.substitute_ty(inner, args);
                self.types.intern(TyKind::Ref { mutable, inner })
            }
            TyKind::Ptr { mutable, inner } => {
                let inner = self.substitute_ty(inner, args);
                self.types.intern(TyKind::Ptr { mutable, inner })
            }
            TyKind::Span { elem, mutable } => {
                let elem = self.substitute_ty(elem, args);
                self.types.intern(TyKind::Span { elem, mutable })
            }
            TyKind::Array { elem, len } => {
                let elem = self.substitute_ty(elem, args);
                self.types.intern(TyKind::Array { elem, len })
            }
            TyKind::Vec { elem } => {
                let elem = self.substitute_ty(elem, args);
                self.types.intern(TyKind::Vec { elem })
            }
            TyKind::Tuple(items) => {
                let items = items
                    .iter()
                    .map(|&item| self.substitute_ty(item, args))
                    .collect();
                self.types.intern(TyKind::Tuple(items))
            }
            TyKind::Fn { latebound, params, ret } => {
                let params = params
                    .iter()
                    .map(|param| FnParam {
                        ty: self.substitute_ty(param.ty, args),
                        mode: param.mode,
                    })
                    .collect();
                let ret = self.substitute_ty(ret, args);
                self.types.intern(TyKind::Fn { latebound, params, ret })
            }
            TyKind::Struct(id) => {
                if let Some(inner) = self.boxes.get(&id).copied() {
                    let inner = self.substitute_ty(inner, args);
                    return self.box_of(inner);
                }
                if let Some(inner) = self.cells.get(&id).copied() {
                    let inner = self.substitute_ty(inner, args);
                    return self.cell_of(inner);
                }
                if let Some(inner) = self.refcells.get(&id).copied() {
                    let inner = self.substitute_ty(inner, args);
                    return self.refcell_of(inner);
                }
                if let Some(inner) = self.maybe_uninit.get(&id).copied() {
                    let inner = self.substitute_ty(inner, args);
                    return self.maybe_uninit_of(inner);
                }
                if let Some(inner) = self.unsafe_cells.get(&id).copied() {
                    let inner = self.substitute_ty(inner, args);
                    return self.unsafe_cell_of(inner);
                }
                if let Some((inner, mutable)) = self.ref_guards.get(&id).copied() {
                    let inner = self.substitute_ty(inner, args);
                    return self.ref_guard_of(inner, mutable);
                }
                let origin = self.types.struct_def(id).origin.clone();
                let Some((name, generic_args)) = origin else { return ty };
                let concrete = generic_args
                    .iter()
                    .map(|&arg| self.substitute_ty(arg, args))
                    .collect::<Vec<_>>();
                if concrete == generic_args {
                    return ty;
                }
                let Some(decl) = self.generic_structs.get(&name).cloned() else { return ty };
                self.instantiate_struct(name, &decl, &concrete, Span::DUMMY)
            }
            TyKind::Enum(id) => {
                let def = self.types.enum_def(id);
                let name = def.name.as_str();
                if name.starts_with("Option_") && def.variants.len() == 2 {
                    let inner = def.variants[1].fields[0].ty;
                    let inner = self.substitute_ty(inner, args);
                    return self.option_of(inner);
                }
                if name.starts_with("Result_") && def.variants.len() == 2 {
                    let ok = def.variants[0].fields[0].ty;
                    let err = def.variants[1].fields[0].ty;
                    let ok = self.substitute_ty(ok, args);
                    let err = self.substitute_ty(err, args);
                    return self.result_of(ok, err);
                }
                ty
            }
            _ => ty,
        }
    }

    fn substitute_generic_param(
        &mut self,
        param: &GenericParam,
        args: &[Ty],
    ) -> GenericParam {
        let callable = param.callable.as_ref().map(|bound| CallableBound {
            params: bound
                .params
                .iter()
                .map(|param| FnParam {
                    ty: self.substitute_ty(param.ty, args),
                    mode: param.mode,
                })
                .collect(),
            ret: self.substitute_ty(bound.ret, args),
            once: bound.once,
            latebound: bound.latebound,
        });
        GenericParam { name: param.name, bounds: param.bounds.clone(), callable }
    }

    /// `[STR-1]`, `[TYP-18]` — the memberwise constructor of a generic
    /// struct. The type arguments come from the values, unless they were
    /// written out.
    fn synth_generic_struct_literal(
        &mut self,
        name: Symbol,
        decl: &GenericStruct,
        args: &[ast::Arg],
        explicit: &[Ty],
        span: Span,
    ) -> Expr {
        if explicit.len() > decl.params.len() {
            self.error(
                codes::E2020,
                span,
                format!(
                    "`{name}` takes {} type arguments, found {}",
                    decl.params.len(),
                    explicit.len()
                ),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let mut solved: Vec<Option<Ty>> = vec![None; decl.params.len()];
        for (slot, ty) in explicit.iter().enumerate() {
            if slot < solved.len() {
                solved[slot] = Some(*ty);
            }
        }
        // Unify each declared field type with the value given for it.
        for (arg, field) in args.iter().zip(decl.fields.iter()) {
            let value = self.synth_committed(&arg.value);
            self.types.unify_with_fixed(field.ty, value.ty, &mut solved, explicit.len());
        }

        let mut substitution = Vec::new();
        for (index, param) in decl.params.iter().enumerate() {
            match solved[index] {
                Some(ty) => substitution.push(ty),
                None => {
                    self.error(
                        codes::E2060,
                        span,
                        format!("cannot tell what `{param}` is here; write `{name}[...](...)`"),
                    );
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                }
            }
        }
        let ty = self.instantiate_struct(name, decl, &substitution, span);
        let TyKind::Struct(id) = *self.types.kind(ty) else {
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };
        let instance = self.types.struct_def(id).name;
        self.synth_struct_literal(id, instance, args, span)
    }

    /// `[TYP-16]` — one struct per set of type arguments. `Pair[i32, f32]`
    /// and `Pair[f32, i32]` are different types with different layouts, built
    /// once each and named after the arguments.
    fn instantiate_struct(
        &mut self,
        name: Symbol,
        decl: &GenericStruct,
        args: &[Ty],
        span: Span,
    ) -> Ty {
        if args.len() != decl.params.len() {
            self.error(
                codes::E2020,
                span,
                format!(
                    "`{name}` takes {} type arguments, found {}",
                    decl.params.len(),
                    args.len()
                ),
            );
            return self.common.error;
        }
        let stem: Vec<String> =
            args.iter().map(|&t| type_stem(&self.types.display(t))).collect();
        let instance = Symbol::intern(&format!("{name}_{}", stem.join("_")));
        if let Some(&ty) = self.named_types.get(&instance) {
            return ty;
        }

        // Register the name before the fields are resolved, so a struct that
        // holds a pointer to itself terminates.
        let id = self.types.add_struct(StructDef {
            name: instance,
            fields: Vec::new(),
            span,
            derives_copy: decl.derives_copy,
            has_drop: false,
            drops_fields: true,
            origin: Some((name, args.to_vec())),
            declaring_module: decl.declaring_module,
        });
        let ty = self.types.intern(TyKind::Struct(id));
        self.struct_ids.insert(instance, id);
        self.named_types.insert(instance, ty);

        // The fields were resolved once with the parameters left opaque, so
        // instantiating is a substitution rather than a re-resolve.
        let fields: Vec<FieldDef> = decl
            .fields
            .iter()
            .map(|field| FieldDef {
                name: field.name,
                ty: self.substitute_ty(field.ty, args),
                span: field.span,
                has_default: field.has_default,
                read_only_outside: field.read_only_outside,
                vis: field.vis,
            })
            .collect();
        self.types.struct_def_mut(id).fields = fields;

        // The methods are substituted the same way, each getting a `DefId` of
        // its own. Their bodies are queued rather than checked here: an
        // instantiation is usually reached in the middle of checking some
        // other body, which is not a place to start checking a new one.
        for method in &decl.methods {
            // Owner parameters occupy the first slots in the stored recipe;
            // method parameters follow them. Substitute the owner and map the
            // method parameters back to zero-based indices for ordinary call
            // inference/monomorphisation.
            let method_params: Vec<Ty> = method
                .generics
                .iter()
                .enumerate()
                .map(|(index, param)| {
                    self.types.intern(TyKind::Param {
                        index: index as u32,
                        name: param.name,
                    })
                })
                .collect();
            let mut combined = args.to_vec();
            combined.extend(method_params);
            let mut params = Vec::new();
            if let Some(receiver) = method.receiver {
                params.push((Symbol::intern("self"), ty, receiver, method.span));
            }
            for &(field_name, param_ty, mode, param_span) in &method.params {
                params.push((
                    field_name,
                    self.substitute_ty(param_ty, &combined),
                    mode,
                    param_span,
                ));
            }
            let ret = self.substitute_ty(method.ret, &combined);
            let generics = method
                .generics
                .iter()
                .map(|param| self.substitute_generic_param(param, &combined))
                .collect();
            let signature =
                Signature { params, ret, generics, borrows: method.borrows.clone() };
            let generic = !signature.generics.is_empty();
            let def = if let Some(receiver) = method.receiver {
                self.register_method(ty, method.name, signature, receiver, None, method.span)
            } else {
                self.register_associated(ty, method.name, signature, None, method.span)
            };
            let Some(def) = def else {
                continue;
            };
            // `[DRP-1]` — a generic that writes `fn drop` gives every one of
            // its instantiations a destructor.
            if method.name.is("drop") {
                self.types.struct_def_mut(id).has_drop = true;
            }
            if generic {
                self.generic_method_sources.insert(
                    def,
                    MethodSource {
                        owner: ty,
                        source: method.source,
                        owner_bindings: decl
                            .params
                            .iter()
                            .copied()
                            .zip(args.iter().copied())
                            .collect(),
                    },
                );
                self.pending_generic_method_validations.push(def);
            } else {
                self.pending_methods.push(PendingMethod {
                    def,
                    owner: ty,
                    origin: (name, args.to_vec()),
                    source: method.source,
                });
            }
        }
        ty
    }

    /// `Option[T]`, as a two-variant enum built once per `T`.
    fn option_of(&mut self, inner: Ty) -> Ty {
        let name = Symbol::intern(&format!("Option_{}", type_stem(&self.types.display(inner))));
        self.builtin_enum(name, &[(Symbol::intern("None"), Vec::new()), (Symbol::intern("Some"), vec![inner])])
    }

    /// IX.1, `[HEAP-1]`, `[DRP-6]` — one unique heap owner.
    ///
    /// The source-visible value contains only a compiler-private `*mut T`.
    /// `has_drop` makes every `Box[T]` move-only and gives drop elaboration an
    /// indivisible owner; `drops_fields: false` prevents the pointer field
    /// from pretending to own an inline `T`. The C backend supplies the real
    /// glue: drop `*value`, then free the allocation.
    fn box_of(&mut self, inner: Ty) -> Ty {
        let name = Symbol::intern(&format!("Box_{}", type_stem(&self.types.display(inner))));
        if let Some(&ty) = self.named_types.get(&name) {
            return ty;
        }
        let pointer = self.types.intern(TyKind::Ptr { mutable: true, inner });
        let id = self.types.add_struct(StructDef {
            name,
            fields: vec![FieldDef {
                name: Symbol::intern("value"),
                ty: pointer,
                span: Span::DUMMY,
                has_default: false,
                read_only_outside: false,
                vis: FieldVis::Private,
            }],
            span: Span::DUMMY,
            derives_copy: false,
            has_drop: true,
            drops_fields: false,
            origin: Some((Symbol::intern("Box"), vec![inner])),
            declaring_module: usize::MAX,
        });
        let ty = self.types.intern(TyKind::Struct(id));
        self.named_types.insert(name, ty);
        self.boxes.insert(id, inner);
        ty
    }

    fn box_inner(&self, ty: Ty) -> Option<Ty> {
        match self.types.kind(ty) {
            TyKind::Struct(id) => self.boxes.get(id).copied(),
            _ => None,
        }
    }

    /// The safe place reached by Box auto-deref. Raw-pointer syntax never
    /// reaches source: the private pointer projection and dereference are
    /// introduced together by the compiler, rooted at the Box owner so the
    /// ordinary borrow and move analyses retain the ownership relationship.
    fn read_box_through(&mut self, boxed: Expr) -> Expr {
        let Some(inner) = self.box_inner(boxed.ty) else { return boxed };
        let span = boxed.span;
        let TyKind::Struct(id) = *self.types.kind(boxed.ty) else { unreachable!() };
        let pointer = self.types.struct_def(id).fields[0].ty;
        let field = Expr {
            ty: pointer,
            kind: ExprKind::Field { base: Box::new(boxed), index: 0 },
            span,
        };
        Expr { ty: inner, kind: ExprKind::Deref(Box::new(field)), span }
    }

    /// `[CELL-1]`, `[CELL-2]`, `[CELL-4]` — `Cell[T]`, as a transparent
    /// one-field struct built once per `T`.
    ///
    /// ADR-019 makes it compiler-known rather than implemented in Ember on
    /// top of another primitive. That historical implementation choice remains
    /// in force after `[UNS-10]` separately defined `UnsafeCell`; neither type
    /// inherits the other's lowering rules. A **struct** rather than a `TyKind`
    /// of its own is what makes
    /// `[CELL-2]`'s "no overhead relative to a plain field" true by
    /// construction instead of by promise, and it hands `[CELL-4]` over
    /// whole: `is_copy` on a struct is `derives_copy && !has_drop && every
    /// field Copy`, and `needs_drop` is `has_drop || any field needs it`, so
    /// with `derives_copy` set and `has_drop` clear both questions reduce to
    /// the same question about `T`. "`Cell[T]` is `Copy` when `T: Copy` …
    /// move-only for a non-`Copy` `T`, and is `Drop` iff `T` is" then needs no
    /// code at all.
    ///
    /// **The field is unreachable from source, which is what `[CELL-2]` rests
    /// on** — "`Cell` never hands out a reference to its contents, so no
    /// aliasing rule can be violated". What enforces it is `[MOD-2]`'s own
    /// privacy: the field is private and the declaring module is `usize::MAX`,
    /// which no real module can be, so `check_field_visible` refuses it
    /// everywhere with `E1020`. Every route to the value is a builtin, so the
    /// borrow checker sees a call where a user write would otherwise be, and
    /// needs no exemption for it.
    ///
    /// The name is an ordinary identifier on purpose. An unspellable one —
    /// `$value`, which the Ember lexer cannot produce — was tried first and is
    /// wrong: the backend writes field names into the C verbatim, `$` in an
    /// identifier is a compiler extension, and `[CG-C-1]` says the emitted C
    /// must "not depend on compiler extensions". It compiled, and only
    /// `-pedantic` said so. Privacy is the mechanism; the name is just a name.
    fn cell_of(&mut self, inner: Ty) -> Ty {
        let name = Symbol::intern(&format!("Cell_{}", type_stem(&self.types.display(inner))));
        if let Some(&ty) = self.named_types.get(&name) {
            return ty;
        }
        let id = self.types.add_struct(StructDef {
            name,
            fields: vec![FieldDef {
                name: Symbol::intern("value"),
                ty: inner,
                span: Span::DUMMY,
                has_default: false,
                read_only_outside: false,
                vis: FieldVis::Private,
            }],
            span: Span::DUMMY,
            derives_copy: true,
            has_drop: false,
            drops_fields: true,
            origin: None,
            declaring_module: usize::MAX,
        });
        let ty = self.types.intern(TyKind::Struct(id));
        self.named_types.insert(name, ty);
        self.cells.insert(id, inner);
        ty
    }

    /// The payload type, if this is a `Cell`.
    fn cell_inner(&self, ty: Ty) -> Option<Ty> {
        match self.types.kind(ty) {
            TyKind::Struct(id) => self.cells.get(id).copied(),
            _ => None,
        }
    }

    /// `[ARN-8]`, `[ARN-8a]` — storage with exactly `T`'s layout which never
    /// structurally drops `T`.
    ///
    /// A one-field compiler-private struct preserves size, alignment and the
    /// field-derived `Copy iff T: Copy` rule. `drops_fields: false` is the
    /// essential semantic distinction: bytes in a `MaybeUninit[T]` slot do
    /// not own an initialized `T` until the explicit transition, so ordinary
    /// structural drop must never visit the payload. This is separate from
    /// both ordinary assignment and `Cell.set`'s replacement order.
    fn maybe_uninit_of(&mut self, inner: Ty) -> Ty {
        let name = Symbol::intern(&format!(
            "MaybeUninit_{}",
            type_stem(&self.types.display(inner))
        ));
        if let Some(&ty) = self.named_types.get(&name) {
            return ty;
        }
        let id = self.types.add_struct(StructDef {
            name,
            fields: vec![FieldDef {
                name: Symbol::intern("value"),
                ty: inner,
                span: Span::DUMMY,
                has_default: false,
                read_only_outside: false,
                vis: FieldVis::Private,
            }],
            span: Span::DUMMY,
            derives_copy: true,
            has_drop: false,
            drops_fields: false,
            origin: None,
            declaring_module: usize::MAX,
        });
        let ty = self.types.intern(TyKind::Struct(id));
        self.named_types.insert(name, ty);
        self.maybe_uninit.insert(id, inner);
        ty
    }

    fn maybe_uninit_inner(&self, ty: Ty) -> Option<Ty> {
        match self.types.kind(ty) {
            TyKind::Struct(id) => self.maybe_uninit.get(id).copied(),
            _ => None,
        }
    }

    /// `[UNS-10]` — the lowest-level interior-mutability primitive.
    ///
    /// The ordinary one-field representation gives `UnsafeCell[T]` exactly
    /// `T`'s size/alignment and ordinary field destruction. Unlike `Cell`, it
    /// is never `Copy`: the exception is semantic, so `derives_copy` stays
    /// false even when the payload is Copy. The field is compiler-private;
    /// the only source-level routes are `get` and `into_inner`.
    fn unsafe_cell_of(&mut self, inner: Ty) -> Ty {
        let name = Symbol::intern(&format!(
            "UnsafeCell_{}",
            type_stem(&self.types.display(inner))
        ));
        if let Some(&ty) = self.named_types.get(&name) {
            return ty;
        }
        let id = self.types.add_struct(StructDef {
            name,
            fields: vec![FieldDef {
                name: Symbol::intern("value"),
                ty: inner,
                span: Span::DUMMY,
                has_default: false,
                read_only_outside: false,
                vis: FieldVis::Private,
            }],
            span: Span::DUMMY,
            derives_copy: false,
            has_drop: false,
            drops_fields: true,
            // Preserve the generic family structurally so ordinary call-site
            // unification can infer T through `UnsafeCell[T]`.
            origin: Some((Symbol::intern("std.mem.UnsafeCell"), vec![inner])),
            declaring_module: usize::MAX,
        });
        let ty = self.types.intern(TyKind::Struct(id));
        self.named_types.insert(name, ty);
        self.unsafe_cells.insert(id, inner);
        ty
    }

    fn unsafe_cell_inner(&self, ty: Ty) -> Option<Ty> {
        match self.types.kind(ty) {
            TyKind::Struct(id) => self.unsafe_cells.get(id).copied(),
            _ => None,
        }
    }

    /// `UnsafeCell` is public but deliberately not in the prelude. Resolve
    /// through the module visibility table so aliases work and an unimported
    /// bare spelling does not become a compiler-only back door.
    fn is_unsafe_cell_name(&self, name: Symbol) -> bool {
        self.resolve_name(name).is("std.mem.UnsafeCell")
    }

    fn reject_unsafe_cell_in_static_safe(&mut self, span: Span) {
        if !self.in_static_safe || self.static_safe_unsafe_cell_reported {
            return;
        }
        self.static_safe_unsafe_cell_reported = true;
        self.sink.emit(
            Diagnostic::error(
                codes::E3105,
                span,
                "`UnsafeCell` is not permitted in `@static_safe` code",
            )
            .note(concat!(
                "`@static_safe` requires the relevant safety invariant to be established ",
                "statically; `UnsafeCell` delegates it to unsafe implementation code [UNS-10b]"
            )),
        );
    }

    fn ty_contains_unsafe_cell(&self, ty: Ty) -> bool {
        fn visit(this: &Checker<'_>, ty: Ty, seen: &mut HashSet<Ty>) -> bool {
            if !seen.insert(ty) {
                return false;
            }
            match this.types.kind(ty) {
                TyKind::Struct(id) => {
                    this.unsafe_cells.contains_key(id)
                        || this
                            .types
                            .struct_def(*id)
                            .fields
                            .iter()
                            .any(|field| visit(this, field.ty, seen))
                }
                TyKind::Enum(id) => this
                    .types
                    .enum_def(*id)
                    .variants
                    .iter()
                    .flat_map(|variant| variant.fields.iter())
                    .any(|field| visit(this, field.ty, seen)),
                TyKind::Ref { inner, .. }
                | TyKind::Ptr { inner, .. }
                | TyKind::Array { elem: inner, .. }
                | TyKind::Vec { elem: inner }
                | TyKind::Span { elem: inner, .. } => visit(this, *inner, seen),
                TyKind::Tuple(items) => items.iter().any(|&item| visit(this, item, seen)),
                TyKind::Fn { params, ret, .. } => {
                    params.iter().any(|param| visit(this, param.ty, seen))
                        || visit(this, *ret, seen)
                }
                TyKind::Range(id) => visit(this, this.types.range_def(*id).repr, seen),
                _ => false,
            }
        }

        visit(self, ty, &mut HashSet::new())
    }

    /// `[CELL-5]`, `[CELL-9]`, `[CELL-12]` — `RefCell[T]`, as a transparent
    /// struct built once per `T`.
    ///
    /// The shape carries over from `Cell` (see `cell_of`): a `StructDef`
    /// interned per `T`, a side table on the checker, privacy as the mechanism
    /// (`declaring_module: usize::MAX` + private fields refuse read/write/ref
    /// everywhere with `E1020`), and ordinary identifiers (never `$`, which
    /// `[CG-C-1]` forbids in emitted C — the backend writes field names
    /// verbatim and `$` needs `-pedantic` to be noticed).
    ///
    /// Fields, in order: `value: T` (at offset 0, so a guard's pointer to the
    /// value is also a pointer to the cell for the release), `borrow: isize`
    /// (the one-word counter: `0` unborrowed, `>0` shared count, `-1`
    /// mutably borrowed), `borrow_file: *u8` + `borrow_line: u32` (the
    /// conflicting borrow's source location, recorded in debug and release
    /// for `[CELL-5]`'s panic).
    ///
    /// `[CELL-12]` (S3/ADR-021, owner ruling — do not re-derive): `RefCell[T]`
    /// is never `Copy` whatever `T` is. `derives_copy` is therefore `false`
    /// here, deliberately overriding the field-derived `Copy` that came free
    /// for `Cell` (`[CELL-4]` explicitly does not reach here). Moving a
    /// `RefCell` transfers the whole cell, borrow state included, via the
    /// ordinary struct move. `needs_drop` stays field-derived (`has_drop`
    /// clear), so a `RefCell` needs drop iff `T` does.
    fn refcell_of(&mut self, inner: Ty) -> Ty {
        let name = Symbol::intern(&format!("RefCell_{}", type_stem(&self.types.display(inner))));
        if let Some(&ty) = self.named_types.get(&name) {
            return ty;
        }
        let u8_ty = self.common.u8;
        let file_ty = self.types.intern(TyKind::Ptr { mutable: false, inner: u8_ty });
        let id = self.types.add_struct(StructDef {
            name,
            fields: vec![
                FieldDef {
                    name: Symbol::intern("value"),
                    ty: inner,
                    span: Span::DUMMY,
                    has_default: false,
                    read_only_outside: false,
                    vis: FieldVis::Private,
                },
                FieldDef {
                    name: Symbol::intern("borrow"),
                    ty: self.common.isize,
                    span: Span::DUMMY,
                    has_default: false,
                    read_only_outside: false,
                    vis: FieldVis::Private,
                },
                FieldDef {
                    name: Symbol::intern("borrow_file"),
                    ty: file_ty,
                    span: Span::DUMMY,
                    has_default: false,
                    read_only_outside: false,
                    vis: FieldVis::Private,
                },
                FieldDef {
                    name: Symbol::intern("borrow_line"),
                    ty: self.common.u32,
                    span: Span::DUMMY,
                    has_default: false,
                    read_only_outside: false,
                    vis: FieldVis::Private,
                },
            ],
            span: Span::DUMMY,
            derives_copy: false,
            has_drop: false,
            drops_fields: true,
            origin: None,
            declaring_module: usize::MAX,
        });
        let ty = self.types.intern(TyKind::Struct(id));
        self.named_types.insert(name, ty);
        self.refcells.insert(id, inner);
        ty
    }

    /// The payload type, if this is a `RefCell`.
    fn refcell_inner(&self, ty: Ty) -> Option<Ty> {
        match self.types.kind(ty) {
            TyKind::Struct(id) => self.refcells.get(id).copied(),
            _ => None,
        }
    }

    /// `[CELL-7]` — `Ref[T]`/`RefMut[T]` guards, as transparent one-field
    /// structs built once per `T`.
    ///
    /// One field holding a `ref T`/`ref mut T` into the cell's `value`, so
    /// `is_view` holds (a struct carrying a borrow is a view) and `[TYP-15]`
    /// applies, and the region borrows the cell via the loan the builtin
    /// lowering writes down. `derives_copy` is clear (guards are move-only:
    /// copying one without incrementing the counter would double-release),
    /// `has_drop` is set so the guard is owned and its `drop` releases the
    /// borrow state — the backend special-cases these drops to a counter
    /// update rather than a `drop` method call.
    fn ref_guard_of(&mut self, inner: Ty, mutable: bool) -> Ty {
        let prefix = if mutable { "RefMut" } else { "Ref" };
        let name = Symbol::intern(&format!("{prefix}_{}", type_stem(&self.types.display(inner))));
        if let Some(&ty) = self.named_types.get(&name) {
            return ty;
        }
        let ref_ty = self.types.intern(TyKind::Ref { mutable, inner });
        let id = self.types.add_struct(StructDef {
            name,
            fields: vec![FieldDef {
                name: Symbol::intern("value"),
                ty: ref_ty,
                span: Span::DUMMY,
                has_default: false,
                read_only_outside: false,
                vis: FieldVis::Private,
            }],
            span: Span::DUMMY,
            derives_copy: false,
            has_drop: true,
            drops_fields: true,
            origin: None,
            declaring_module: usize::MAX,
        });
        let ty = self.types.intern(TyKind::Struct(id));
        self.named_types.insert(name, ty);
        self.ref_guards.insert(id, (inner, mutable));
        ty
    }

    /// The viewed type and mutability, if this is a `Ref`/`RefMut` guard.
    fn ref_guard_inner(&self, ty: Ty) -> Option<(Ty, bool)> {
        match self.types.kind(ty) {
            TyKind::Struct(id) => self.ref_guards.get(id).copied(),
            _ => None,
        }
    }

    /// `[ARN-1]` — the growing arena is one move-only, compiler-known owner
    /// of an opaque runtime state. It is deliberately not an
    /// interior-mutability type: shared access may bump-allocate, but the
    /// returned references carry the arena borrow and every reclaiming
    /// operation takes `mut self` (`[LT-4]`, `[ARN-7]`).
    fn arena_ty(&mut self) -> Ty {
        let name = Symbol::intern("Arena");
        if let Some(&ty) = self.named_types.get(&name) {
            if matches!(self.types.kind(ty), TyKind::Struct(id) if self.arenas.contains(id)) {
                return ty;
            }
        }
        let state = self.types.intern(TyKind::Ptr { mutable: true, inner: self.common.u8 });
        let id = self.types.add_struct(StructDef {
            name,
            fields: vec![FieldDef {
                name: Symbol::intern("state"),
                ty: state,
                span: Span::DUMMY,
                has_default: false,
                read_only_outside: false,
                vis: FieldVis::Private,
            }, FieldDef {
                // A scope borrows the parent as a whole in MIR, while the
                // generated representation retains this address as a token.
                // Each ScopedArena has its own token, so nesting is unbounded.
                name: Symbol::intern("token"),
                ty: self.common.u8,
                span: Span::DUMMY,
                has_default: false,
                read_only_outside: false,
                vis: FieldVis::Private,
            }],
            span: Span::DUMMY,
            derives_copy: false,
            // The backend supplies the destructor. No source-visible drop
            // method exists, and the pointer field itself owns nothing.
            has_drop: true,
            drops_fields: true,
            origin: None,
            declaring_module: usize::MAX,
        });
        let ty = self.types.intern(TyKind::Struct(id));
        self.named_types.insert(name, ty);
        self.arenas.insert(id);
        ty
    }

    fn is_arena(&self, ty: Ty) -> bool {
        matches!(self.types.kind(ty), TyKind::Struct(id) if self.arenas.contains(id))
    }

    /// `[ARN-4]` — a fixed arena is a view over a mutable byte span plus its
    /// bump offset. It owns no allocation and therefore needs no destructor;
    /// the mutable span makes the type a view and move-only structurally.
    fn fixed_arena_ty(&mut self) -> Ty {
        let name = Symbol::intern("FixedArena");
        if let Some(&ty) = self.named_types.get(&name) {
            if matches!(self.types.kind(ty), TyKind::Struct(id) if self.fixed_arenas.contains(id)) {
                return ty;
            }
        }
        let buffer = self.types.intern(TyKind::Span { elem: self.common.u8, mutable: true });
        let id = self.types.add_struct(StructDef {
            name,
            fields: vec![
                FieldDef {
                    name: Symbol::intern("buffer"),
                    ty: buffer,
                    span: Span::DUMMY,
                    has_default: false,
                    read_only_outside: false,
                    vis: FieldVis::Private,
                },
                FieldDef {
                    name: Symbol::intern("used"),
                    ty: self.common.usize,
                    span: Span::DUMMY,
                    has_default: false,
                    read_only_outside: false,
                    vis: FieldVis::Private,
                },
            ],
            span: Span::DUMMY,
            derives_copy: false,
            has_drop: false,
            drops_fields: true,
            origin: None,
            declaring_module: usize::MAX,
        });
        let ty = self.types.intern(TyKind::Struct(id));
        self.named_types.insert(name, ty);
        self.fixed_arenas.insert(id);
        ty
    }

    fn is_fixed_arena(&self, ty: Ty) -> bool {
        matches!(self.types.kind(ty), TyKind::Struct(id) if self.fixed_arenas.contains(id))
    }

    /// `[ARN-6]` — one scope guard shape serves every nesting depth. The
    /// `parent_token` reference is what carries the immediate parent's mutable
    /// borrow; `state` reaches the growing arena; `mark` identifies the bump
    /// position to restore; and this scope's own token supports another nested
    /// scope without a recursively sized value.
    fn scoped_arena_ty(&mut self) -> Ty {
        let name = Symbol::intern("ScopedArena");
        if let Some(&ty) = self.named_types.get(&name) {
            if matches!(self.types.kind(ty), TyKind::Struct(id) if self.scoped_arenas.contains(id)) {
                return ty;
            }
        }
        let token_ref = self.types.intern(TyKind::Ref { mutable: true, inner: self.common.u8 });
        let state = self.types.intern(TyKind::Ptr { mutable: true, inner: self.common.u8 });
        let id = self.types.add_struct(StructDef {
            name,
            fields: vec![
                FieldDef {
                    name: Symbol::intern("parent_token"),
                    ty: token_ref,
                    span: Span::DUMMY,
                    has_default: false,
                    read_only_outside: false,
                    vis: FieldVis::Private,
                },
                FieldDef {
                    name: Symbol::intern("state"),
                    ty: state,
                    span: Span::DUMMY,
                    has_default: false,
                    read_only_outside: false,
                    vis: FieldVis::Private,
                },
                FieldDef {
                    name: Symbol::intern("mark"),
                    ty: state,
                    span: Span::DUMMY,
                    has_default: false,
                    read_only_outside: false,
                    vis: FieldVis::Private,
                },
                FieldDef {
                    name: Symbol::intern("token"),
                    ty: self.common.u8,
                    span: Span::DUMMY,
                    has_default: false,
                    read_only_outside: false,
                    vis: FieldVis::Private,
                },
            ],
            span: Span::DUMMY,
            derives_copy: false,
            has_drop: true,
            drops_fields: true,
            origin: None,
            declaring_module: usize::MAX,
        });
        let ty = self.types.intern(TyKind::Struct(id));
        self.named_types.insert(name, ty);
        self.scoped_arenas.insert(id);
        ty
    }

    fn is_scoped_arena(&self, ty: Ty) -> bool {
        matches!(self.types.kind(ty), TyKind::Struct(id) if self.scoped_arenas.contains(id))
    }

    /// `Result[T, E]`, likewise.
    fn result_of(&mut self, ok: Ty, err: Ty) -> Ty {
        let name = Symbol::intern(&format!(
            "Result_{}_{}",
            type_stem(&self.types.display(ok)),
            type_stem(&self.types.display(err))
        ));
        self.builtin_enum(name, &[(Symbol::intern("Ok"), vec![ok]), (Symbol::intern("Err"), vec![err])])
    }

    /// Build a compiler-known enum, or return the one already built. The name
    /// carries the type arguments, so `Option[i32]` and `Option[f32]` are
    /// different types and are only built once each.
    fn builtin_enum(&mut self, name: Symbol, variants: &[(Symbol, Vec<Ty>)]) -> Ty {
        if let Some(&ty) = self.named_types.get(&name) {
            return ty;
        }
        let repr = self.common.u8;
        let variants: Vec<VariantDef> = variants
            .iter()
            .enumerate()
            .map(|(index, (variant, payload))| VariantDef {
                name: *variant,
                fields: payload
                    .iter()
                    .enumerate()
                    .map(|(i, &ty)| FieldDef {
                        name: Symbol::intern(&format!("_{i}")),
                        ty,
                        span: Span::DUMMY,
                        has_default: false,
                        read_only_outside: false,
                        vis: FieldVis::Public,
                    })
                    .collect(),
                discriminant: index as i128,
                span: Span::DUMMY,
            })
            .collect();
        let id = self.types.add_enum(EnumDef {
            name,
            variants,
            span: Span::DUMMY,
            repr,
            repr_is_explicit: false,
            derives_copy: true,
            has_drop: false,
        });
        let ty = self.types.intern(TyKind::Enum(id));
        self.enum_ids.insert(name, id);
        self.named_types.insert(name, ty);
        ty
    }

    /// The element type an expected array type asks for, if the expectation
    /// is an array at all.
    fn expected_elem(&self, expected: Option<Ty>) -> Option<Ty> {
        match self.types.kind(expected?) {
            TyKind::Array { elem, .. } => Some(*elem),
            _ => None,
        }
    }

    fn scalar_named(&self, name: &str) -> Option<Ty> {
        let c = self.common;
        Some(match name {
            "bool" => c.bool_,
            "char" => c.char_,
            "i8" => c.i8,
            "i16" => c.i16,
            "i32" => c.i32,
            "i64" => c.i64,
            "i128" => c.i128,
            "isize" => c.isize,
            "u8" => c.u8,
            "u16" => c.u16,
            "u32" => c.u32,
            "u64" => c.u64,
            "u128" => c.u128,
            "usize" => c.usize,
            "f16" => c.f16,
            "f32" => c.f32,
            "f64" => c.f64,
            "str" => c.str_,
            "void" => c.void,
            _ => return None,
        })
    }

    // -- bodies -------------------------------------------------------------

    fn check_bodies(&mut self, module: &ast::Module) -> Program {
        let mut functions = Vec::new();
        let mut main = None;

        for item in &module.items {
            let ast::ItemKind::Fn(decl) = &item.kind else { continue };
            let Some(&def) = self.fn_ids.get(&self.qualified(decl.name.name)) else { continue };

            // `[TYP-17]` — a generic body is checked **once**, with its
            // parameters opaque, so that using an operation its bounds do not
            // provide is an error here rather than at some instantiation.
            // Nothing is emitted for it: only its instantiations exist at run
            // time.
            if !self.signatures[def.0 as usize].generics.is_empty() {
                let outer_static_safe = self.in_static_safe;
                let outer_static_safe_reported = self.static_safe_unsafe_cell_reported;
                self.in_static_safe = has_attribute(&item.attrs, "static_safe");
                self.static_safe_unsafe_cell_reported = false;
                let generics = self.signatures[def.0 as usize].generics.clone();
                self.type_params.clear();
                self.current_generics = generics.clone();
                for (index, param) in generics.iter().enumerate() {
                    let ty = self
                        .types
                        .intern(TyKind::Param { index: index as u32, name: param.name });
                    self.type_params.insert(param.name, ty);
                }
                if let Some(block) = &decl.body {
                    self.locals = Vec::new();
                    self.scopes = vec![HashMap::new()];
                    self.borrowed_params.clear();
                    self.callable_once_locals.clear();
                    self.callable_parameter_locals.clear();
                    self.latebound_callable_parameter_locals.clear();
                    self.callable_value_bindings.clear();
                    self.ret_ty = self.signatures[def.0 as usize].ret;
                    let signature_params: Vec<(Symbol, Ty, Mode, Span)> = self.signatures
                        [def.0 as usize]
                        .params
                        .iter()
                        .map(|(n, t, m, s)| (*n, *t, *m, *s))
                        .collect();
                    for (index, (name, ty, mode, param_span)) in
                        signature_params.iter().copied().enumerate()
                    {
                        let local_ty = match mode {
                            Mode::Mut => self.mut_param_ty(ty),
                            _ => ty,
                        };
                        let local = self.declare(Some(name), local_ty, param_span);
                        if mode == Mode::Borrow {
                            self.borrowed_params.insert(local);
                        }
                        if decl.params.get(index).is_some_and(is_owned_callable_param) {
                            self.callable_once_locals.insert(local);
                        }
                        if decl.params.get(index).is_some_and(is_callable_param) {
                            self.callable_parameter_locals.insert(local);
                        }
                        if decl.params.get(index).is_some_and(is_latebound_callable_param) {
                            self.latebound_callable_parameter_locals.insert(local);
                        }
                    }
                    if self.in_static_safe
                        && (signature_params
                            .iter()
                            .any(|(_, ty, _, _)| self.ty_contains_unsafe_cell(*ty))
                            || self.ty_contains_unsafe_cell(self.ret_ty))
                    {
                        self.reject_unsafe_cell_in_static_safe(item.span);
                    }
                    self.check_block(block);
                }
                self.in_static_safe = outer_static_safe;
                self.static_safe_unsafe_cell_reported = outer_static_safe_reported;
                self.type_params.clear();
                self.current_generics.clear();
                continue;
            }

            self.locals = Vec::new();
            self.scopes = vec![HashMap::new()];
            self.borrowed_params.clear();
            self.ret_ty = self.signatures[def.0 as usize].ret;
            let outer_static_safe = self.in_static_safe;
            let outer_static_safe_reported = self.static_safe_unsafe_cell_reported;
            self.in_static_safe = has_attribute(&item.attrs, "static_safe");
            self.static_safe_unsafe_cell_reported = false;

            let mut params = Vec::new();
            let signature_params: Vec<(Symbol, Ty, Mode, Span)> = self.signatures[def.0 as usize]
                .params
                .iter()
                .map(|(n, t, m, s)| (*n, *t, *m, *s))
                .collect();
            for (name, ty, mode, span) in signature_params.iter().copied() {
                // A `mut` parameter is an inout: inside the function it is a
                // `ref mut T`, and every mention of its name reads through it.
                // Without this the callee writes to a copy and the caller
                // never sees it.
                let local_ty = match mode {
                    Mode::Mut => self.mut_param_ty(ty),
                    _ => ty,
                };
                let local = self.declare(Some(name), local_ty, span);
                if mode == Mode::Borrow {
                    self.borrowed_params.insert(local);
                }
                params.push(Param { local, mode });
            }

            if self.in_static_safe
                && (signature_params
                    .iter()
                    .any(|(_, ty, _, _)| self.ty_contains_unsafe_cell(*ty))
                    || self.ty_contains_unsafe_cell(self.ret_ty))
            {
                self.reject_unsafe_cell_in_static_safe(item.span);
            }

            let body = match &decl.body {
                Some(block) => self.check_block(block),
                None => Block { stmts: Vec::new(), span: item.span },
            };
            self.in_static_safe = outer_static_safe;
            self.static_safe_unsafe_cell_reported = outer_static_safe_reported;

            // `[MNG-1]` — the symbol carries the module, so two modules may
            // each declare a `helper`. Only the root module's `main` is the
            // entry point; another module's is `a.main`, which is not it.
            let name = self.qualified(decl.name.name);
            let is_main = name.is("main");
            if is_main {
                main = Some(def);
            }
            let overflow = self.overflow_policy(&item.attrs, item.span);
            // `extern "C" fn` DEFINES a function a host links against, so its
            // symbol is the name as written — `[MNG-1]`'s module-qualified
            // mangling would make it unfindable, which defeats the point.
            let symbol = match &decl.abi {
                Some(_) => decl.name.name.to_string(),
                None => mangle(name, is_main),
            };
            self.check_foreign_signature(decl, def);
            functions.push(Function {
                def,
                name,
                class_init: false,
                symbol,
                is_unsafe: decl.is_unsafe,
                abi: decl.abi.clone(),
                params,
                locals: std::mem::take(&mut self.locals),
                ret: self.ret_ty,
                body,
                span: item.span,
                overflow,
                borrows: self.signatures[def.0 as usize].borrows.clone(),
                closure_environment: None,
                closure_captures_by_move: false,
            });
        }

        functions.extend(self.check_method_bodies(module));
        Program { functions, main }
    }

    /// `[TYP-16]` — one real body per instantiation. Each is the generic body
    /// re-checked with its parameters bound to the concrete types, which is
    /// what monomorphisation means for a backend with no generics of its own.
    ///
    /// Checking an instantiation can reach another generic call, so this
    /// drains a queue rather than walking a list once.
    fn check_instantiations(&mut self, modules: &[LoadedModule]) -> Vec<Function> {
        let mut out = Vec::new();
        // Where each generic function's declaration lives, so its body can be
        // found again.
        let mut sources: HashMap<DefId, (usize, usize)> = HashMap::new();
        for (module_index, loaded) in modules.iter().enumerate() {
            for (item_index, item) in loaded.module.items.iter().enumerate() {
                let ast::ItemKind::Fn(decl) = &item.kind else { continue };
                self.current_module = module_index;
                if let Some(&def) = self.fn_ids.get(&self.qualified(decl.name.name)) {
                    sources.insert(def, (module_index, item_index));
                }
            }
        }

        // Errors were already reported when the generic body was checked
        // once; an instantiation must not repeat them.
        let mut quiet = Sink::new();
        // A generic struct's method body is never checked with its parameters
        // opaque — there is no opaque `Buffer[T]` for `self` to have — so the
        // first instantiation of each method is the one that reports. Later
        // ones are quiet, as generic function instantiations are.
        let mut reported: std::collections::HashSet<(Symbol, Symbol)> =
            std::collections::HashSet::new();
        while !self.pending.is_empty()
            || !self.pending_methods.is_empty()
            || !self.pending_generic_method_validations.is_empty()
            || !self.pending_generic_methods.is_empty()
        {
            while let Some((key, instance)) = self.pending.pop() {
                let Some(&(module_index, item_index)) = sources.get(&key.def) else { continue };
                let item = &modules[module_index].module.items[item_index];
                let ast::ItemKind::Fn(decl) = &item.kind else { continue };
                let Some(block) = &decl.body else { continue };

                self.current_module = module_index;
                // The parameters are now the concrete types.
                let generics = self.signatures[key.def.0 as usize].generics.clone();
                self.type_params.clear();
                for (param, &ty) in generics.iter().zip(key.args.iter()) {
                    self.type_params.insert(param.name, ty);
                }
                let source_params = self.signatures[key.def.0 as usize].params.clone();
                self.callable_value_params = source_params
                    .iter()
                    .enumerate()
                    .filter_map(|(parameter, (_, ty, _, _))| {
                        let TyKind::Param { index, .. } = *self.types.kind(*ty) else {
                            return None;
                        };
                        key.callable_values
                            .get(index as usize)
                            .and_then(|value| value.map(|def| (parameter, def)))
                    })
                    .collect();

                let quiet_before = quiet.diagnostics().len();
                let saved = std::mem::replace(self.sink, std::mem::take(&mut quiet));
                let function = self.check_one_function(decl, block, instance, &item.attrs, item.span);
                quiet = std::mem::replace(self.sink, saved);
                let concrete = quiet.diagnostics()[quiet_before..].to_vec();
                self.emit_concrete_instantiation_diagnostics(concrete);
                self.type_params.clear();
                self.callable_value_params.clear();
                out.push(function);
            }

            // `[TYP-16]` — the methods of instantiated generic structs, checked
            // with the struct's parameters bound to the arguments it was built
            // with and `self` bound to the instantiation.
            while let Some(job) = self.pending_methods.pop() {
                let (module_index, item_index, member_index) = job.source;
                let item = &modules[module_index].module.items[item_index];
                let ast::ItemKind::Struct(decl) = &item.kind else { continue };
                let Some(member) = decl.members.get(member_index) else { continue };
                let ast::MemberKind::Fn(fn_decl) = &member.kind else { continue };
                let Some(block) = &fn_decl.body else { continue };

                self.current_module = module_index;
                let (generic_name, args) = job.origin;
                let Some(generic) = self.generic_structs.get(&generic_name) else { continue };
                let params = generic.params.clone();
                self.type_params.clear();
                for (param, &ty) in params.iter().zip(args.iter()) {
                    self.type_params.insert(*param, ty);
                }

                let first = reported.insert((generic_name, fn_decl.name.name));
                let quiet_before = (!first).then(|| quiet.diagnostics().len());
                let saved = if first {
                    None
                } else {
                    Some(std::mem::replace(self.sink, std::mem::take(&mut quiet)))
                };
                let function = self.check_one_method(
                    job.owner,
                    fn_decl,
                    block,
                    job.def,
                    &member.attrs,
                    member.span,
                );
                if let Some(saved) = saved {
                    quiet = std::mem::replace(self.sink, saved);
                }
                if let Some(before) = quiet_before {
                    let concrete = quiet.diagnostics()[before..].to_vec();
                    self.emit_concrete_instantiation_diagnostics(concrete);
                }
                self.type_params.clear();
                if let Some(function) = function {
                    out.push(function);
                }
            }

            while let Some(def) = self.pending_generic_method_validations.pop() {
                let Some(source) = self.generic_method_sources.get(&def).cloned() else {
                    continue;
                };
                let (module_index, item_index, member_index) = source.source;
                let item = &modules[module_index].module.items[item_index];
                let Some(member) = item_members(item).and_then(|members| members.get(member_index))
                else {
                    continue;
                };
                let ast::MemberKind::Fn(decl) = &member.kind else { continue };
                let Some(block) = &decl.body else { continue };

                self.current_module = module_index;
                self.type_params.clear();
                for (name, ty) in &source.owner_bindings {
                    self.type_params.insert(*name, *ty);
                }
                let generics = self.signatures[def.0 as usize].generics.clone();
                self.current_generics = generics.clone();
                for (index, param) in generics.iter().enumerate() {
                    let ty = self.types.intern(TyKind::Param {
                        index: index as u32,
                        name: param.name,
                    });
                    self.type_params.insert(param.name, ty);
                }
                let _ = self.check_one_method(
                    source.owner,
                    decl,
                    block,
                    def,
                    &member.attrs,
                    member.span,
                );
                self.type_params.clear();
                self.current_generics.clear();
            }

            while let Some((key, instance)) = self.pending_generic_methods.pop() {
                let Some(source) = self.generic_method_sources.get(&key.def).cloned() else {
                    continue;
                };
                let (module_index, item_index, member_index) = source.source;
                let item = &modules[module_index].module.items[item_index];
                let Some(member) = item_members(item).and_then(|members| members.get(member_index))
                else {
                    continue;
                };
                let ast::MemberKind::Fn(decl) = &member.kind else { continue };
                let Some(block) = &decl.body else { continue };

                self.current_module = module_index;
                self.type_params.clear();
                for (name, ty) in &source.owner_bindings {
                    self.type_params.insert(*name, *ty);
                }
                let generics = self.signatures[key.def.0 as usize].generics.clone();
                self.current_generics = generics.clone();
                for (param, &ty) in generics.iter().zip(key.args.iter()) {
                    self.type_params.insert(param.name, ty);
                }

                let quiet_before = quiet.diagnostics().len();
                let saved = std::mem::replace(self.sink, std::mem::take(&mut quiet));
                let function = self.check_one_method(
                    source.owner,
                    decl,
                    block,
                    instance,
                    &member.attrs,
                    member.span,
                );
                quiet = std::mem::replace(self.sink, saved);
                let concrete = quiet.diagnostics()[quiet_before..].to_vec();
                self.emit_concrete_instantiation_diagnostics(concrete);
                self.type_params.clear();
                self.current_generics.clear();
                if let Some(mut function) = function {
                    function.symbol = format!("{}__{}", function.symbol, instance.0);
                    out.push(function);
                }
            }
        }
        out
    }

    /// One function body, checked into a `Function` with a given `DefId`.
    /// `[FFI-5]`, `[RNG-10b]` — what an `extern "C" fn` definition may
    /// mention in its signature.
    ///
    /// A foreign ABI means the type has to have a representation the other side
    /// agrees on, so every parameter and the return type must be FFI-safe.
    /// A range type is called out separately because it *is* representable —
    /// it erases to its representation — and is still refused: `[RNG-10b]` says
    /// a value arriving from foreign code "enters at the representation type
    /// and becomes a range value only through `[RNG-3]`", so admitting one here
    /// would let a foreign caller manufacture a range value that never passed a
    /// check, and `[RNG-9]` makes that undefined behaviour rather than a wrong
    /// number.
    fn check_foreign_signature(&mut self, decl: &ast::FnDecl, def: DefId) {
        if decl.abi.is_none() {
            return;
        }
        if decl.dispatch != ast::Dispatch::Static {
            self.error(
                codes::E0104,
                decl.name.span,
                "`virtual` and `override` are not admitted on an `extern` function",
            );
        }
        let signature: Vec<(Symbol, Ty, Span)> = self.signatures[def.0 as usize]
            .params
            .iter()
            .map(|(n, t, _, s)| (*n, *t, *s))
            .collect();
        let ret = self.signatures[def.0 as usize].ret;
let check = |this: &mut Self, ty: Ty, span: Span, what: String| {
            if matches!(this.types.kind(ty), TyKind::Range(_)) {
                let shown = this.types.display(ty);
                this.sink.emit(
                    Diagnostic::error(
                        codes::E5054,
                        span,
                        format!("`{shown}` is a range type, so it may not cross a foreign boundary"),
                    )
                    .primary_label(what.clone())
                    .help(format!(
                        "declare the representation and construct with `{shown}.checked(...)` in an Ember-side wrapper"
                    ))
                    .note(concat!(
                        "a value from foreign code enters at the representation type and becomes ",
                        "a range value only through a checked construction [RNG-10b]"
                    )),
                );
                return;
            }
            if !this.types.is_ffi_safe(ty) {
                let shown = this.types.display(ty);
                this.sink.emit(
                    Diagnostic::error(
                        codes::E5050,
                        span,
                        format!("`{shown}` has no foreign representation"),
                    )
                    .primary_label(what)
                    .note("every type in an `extern` signature must be FFI-safe [FFI-5]"),
                );
            }
        };
        for (name, ty, span) in signature {
            check(self, ty, span, format!("`{name}` is this parameter"));
        }
        if ret != self.common.void {
            check(self, ret, decl.name.span, "this is the return type".to_string());
        }
    }

    fn check_one_function(
        &mut self,
        decl: &ast::FnDecl,
        block: &ast::Block,
        def: DefId,
        attrs: &[ast::Attribute],
        span: Span,
    ) -> Function {
        self.locals = Vec::new();
        self.scopes = vec![HashMap::new()];
        self.borrowed_params.clear();
        self.callable_once_locals.clear();
        self.callable_parameter_locals.clear();
        self.latebound_callable_parameter_locals.clear();
        self.callable_value_bindings.clear();
        self.ret_ty = self.signatures[def.0 as usize].ret;
        let outer_static_safe = self.in_static_safe;
        let outer_static_safe_reported = self.static_safe_unsafe_cell_reported;
        self.in_static_safe = has_attribute(attrs, "static_safe");
        self.static_safe_unsafe_cell_reported = false;

        let signature_params: Vec<(Symbol, Ty, Mode, Span)> = self.signatures[def.0 as usize]
            .params
            .iter()
            .map(|(n, t, m, s)| (*n, *t, *m, *s))
            .collect();
        let mut params = Vec::new();
        for (index, (name, ty, mode, param_span)) in signature_params.iter().copied().enumerate() {
            let local_ty = match mode {
                Mode::Mut => self.mut_param_ty(ty),
                _ => ty,
            };
            let local = self.declare(Some(name), local_ty, param_span);
            if mode == Mode::Borrow {
                self.borrowed_params.insert(local);
            }
            if decl.params.get(index).is_some_and(is_owned_callable_param) {
                self.callable_once_locals.insert(local);
            }
            if decl.params.get(index).is_some_and(is_callable_param) {
                self.callable_parameter_locals.insert(local);
            }
            if decl.params.get(index).is_some_and(is_latebound_callable_param) {
                self.latebound_callable_parameter_locals.insert(local);
            }
            if let Some(&def) = self.callable_value_params.get(&index) {
                self.callable_value_bindings.insert(local, def);
            }
            params.push(Param { local, mode });
        }
        if self.in_static_safe
            && (signature_params.iter().any(|(_, ty, _, _)| self.ty_contains_unsafe_cell(*ty))
                || self.ty_contains_unsafe_cell(self.ret_ty))
        {
            self.reject_unsafe_cell_in_static_safe(span);
        }
        let body = self.check_block(block);
        let overflow = self.overflow_policy(attrs, span);
        self.in_static_safe = outer_static_safe;
        self.static_safe_unsafe_cell_reported = outer_static_safe_reported;
        // `[MONO-1]` — the symbol carries the instantiation, so two of them
        // never collide and identical ones dedupe at link time.
        let name = self.qualified(decl.name.name);
        Function {
            def,
            name,
            class_init: false,
            symbol: format!("{}__{}", ember_branding::mangled(name.as_str()), def.0),
            is_unsafe: decl.is_unsafe,
            abi: decl.abi.clone(),
            params,
            locals: std::mem::take(&mut self.locals),
            ret: self.ret_ty,
            body,
            span,
            overflow,
            borrows: self.signatures[def.0 as usize].borrows.clone(),
            closure_environment: None,
            closure_captures_by_move: false,
        }
    }

    /// Method bodies, in the same walk `collect_methods` used so the two
    /// cannot disagree about which methods exist.
    fn check_method_bodies(&mut self, module: &ast::Module) -> Vec<Function> {
        let mut out = Vec::new();
        for item in &module.items {
            let (members, owner) = match &item.kind {
                ast::ItemKind::Struct(decl) => {
                    (&decl.members, self.named_types.get(&self.qualified(decl.name.name)).copied())
                }
                ast::ItemKind::Class(decl) => {
                    (&decl.members, self.named_types.get(&self.qualified(decl.name.name)).copied())
                }
                ast::ItemKind::Enum(decl) => {
                    (&decl.members, self.named_types.get(&self.qualified(decl.name.name)).copied())
                }
                ast::ItemKind::Extend(decl) => {
                    let ty = self.resolve_type(&decl.target);
                    (&decl.members, (ty != self.common.error).then_some(ty))
                }
                _ => continue,
            };
            let Some(owner) = owner else { continue };
            for member in members {
                let ast::MemberKind::Fn(decl) = &member.kind else { continue };
                let Some(block) = &decl.body else { continue };
                let has_receiver = decl
                    .params
                    .iter()
                    .any(|param| matches!(param.kind, ast::ParamKind::Receiver { .. }));
                let def = if has_receiver {
                    let Some(entry) = self.methods.get(&(owner, decl.name.name)) else { continue };
                    entry.def
                } else {
                    let Some(entry) = self.associated.get(&(owner, decl.name.name)) else {
                        continue;
                    };
                    entry.def
                };
                if !self.signatures[def.0 as usize].generics.is_empty() {
                    let source = self.generic_method_sources.get(&def).cloned();
                    self.type_params.clear();
                    if let Some(source) = &source {
                        for (name, ty) in &source.owner_bindings {
                            self.type_params.insert(*name, *ty);
                        }
                    }
                    let generics = self.signatures[def.0 as usize].generics.clone();
                    self.current_generics = generics.clone();
                    for (index, param) in generics.iter().enumerate() {
                        let ty = self.types.intern(TyKind::Param {
                            index: index as u32,
                            name: param.name,
                        });
                        self.type_params.insert(param.name, ty);
                    }
                    let _ = self.check_one_method(
                        owner,
                        decl,
                        block,
                        def,
                        &member.attrs,
                        member.span,
                    );
                    self.type_params.clear();
                    self.current_generics.clear();
                    continue;
                }
                if let Some(function) =
                    self.check_one_method(owner, decl, block, def, &member.attrs, member.span)
                {
                    out.push(function);
                }
            }
        }
        out.extend(self.check_default_bodies(module));
        out
    }

    /// The bodies of the default methods `register_defaults` created: one copy
    /// per implementing type, checked with `self` bound to that type.
    fn check_default_bodies(&mut self, module: &ast::Module) -> Vec<Function> {
        let mut out = Vec::new();
        let implementations = self.implemented.clone();
        for item in &module.items {
            let ast::ItemKind::Interface(decl) = &item.kind else { continue };
            let interface = self.qualified(decl.name.name);
            for (ty, _, _) in implementations.iter().filter(|(_, i, _)| *i == interface) {
                for member in &decl.members {
                    let ast::MemberKind::Fn(fn_decl) = &member.kind else { continue };
                    let Some(block) = &fn_decl.body else { continue };
                    // Only the entries that came from this interface's default
                    // — a type that wrote its own method keeps that one.
                    let has_receiver = fn_decl
                        .params
                        .iter()
                        .any(|param| matches!(param.kind, ast::ParamKind::Receiver { .. }));
                    let def = if has_receiver {
                        let Some(entry) = self.methods.get(&(*ty, fn_decl.name.name)) else {
                            continue;
                        };
                        if entry.from_interface != Some(interface) {
                            continue;
                        }
                        entry.def
                    } else {
                        let Some(entry) = self.associated.get(&(*ty, fn_decl.name.name)) else {
                            continue;
                        };
                        if entry.from_interface != Some(interface) {
                            continue;
                        }
                        entry.def
                    };
                    if out.iter().any(|f: &Function| f.def == def) {
                        continue;
                    }
                    if !self.signatures[def.0 as usize].generics.is_empty() {
                        self.type_params.clear();
                        let generics = self.signatures[def.0 as usize].generics.clone();
                        self.current_generics = generics.clone();
                        for (index, param) in generics.iter().enumerate() {
                            let ty = self.types.intern(TyKind::Param {
                                index: index as u32,
                                name: param.name,
                            });
                            self.type_params.insert(param.name, ty);
                        }
                        let _ = self.check_one_method(
                            *ty,
                            fn_decl,
                            block,
                            def,
                            &member.attrs,
                            member.span,
                        );
                        self.type_params.clear();
                        self.current_generics.clear();
                        continue;
                    }
                    if let Some(function) =
                        self.check_one_method(*ty, fn_decl, block, def, &member.attrs, member.span)
                    {
                        out.push(function);
                    }
                }
            }
        }
        out
    }

    fn check_one_method(
        &mut self,
        owner: Ty,
        decl: &ast::FnDecl,
        block: &ast::Block,
        def: DefId,
        attrs: &[ast::Attribute],
        span: Span,
    ) -> Option<Function> {
        self.locals = Vec::new();
        self.scopes = vec![HashMap::new()];
        self.borrowed_params.clear();
        self.callable_once_locals.clear();
        self.callable_parameter_locals.clear();
        self.latebound_callable_parameter_locals.clear();
        self.ret_ty = self.signatures[def.0 as usize].ret;
        let outer_static_safe = self.in_static_safe;
        let outer_static_safe_reported = self.static_safe_unsafe_cell_reported;
        self.in_static_safe = has_attribute(attrs, "static_safe");
        self.static_safe_unsafe_cell_reported = false;

        let signature_params: Vec<(Symbol, Ty, Mode, Span)> = self.signatures[def.0 as usize]
            .params
            .iter()
            .map(|(n, t, m, s)| (*n, *t, *m, *s))
            .collect();
        let mut params = Vec::new();
        for (index, (name, ty, mode, param_span)) in signature_params.iter().copied().enumerate() {
            let local_ty = match mode {
                Mode::Mut => self.mut_param_ty(ty),
                _ => ty,
            };
            let local = self.declare(Some(name), local_ty, param_span);
            if mode == Mode::Borrow {
                self.borrowed_params.insert(local);
            }
            if decl.params.get(index).is_some_and(is_owned_callable_param) {
                self.callable_once_locals.insert(local);
            }
            if decl.params.get(index).is_some_and(is_callable_param) {
                self.callable_parameter_locals.insert(local);
            }
            if decl.params.get(index).is_some_and(is_latebound_callable_param) {
                self.latebound_callable_parameter_locals.insert(local);
            }
            params.push(Param { local, mode });
        }
        if self.in_static_safe
            && (signature_params.iter().any(|(_, ty, _, _)| self.ty_contains_unsafe_cell(*ty))
                || self.ty_contains_unsafe_cell(self.ret_ty))
        {
            self.reject_unsafe_cell_in_static_safe(span);
        }

        let outer_self = self.self_ty.replace(owner);
        let outer_class_init = self.class_init.take();
        let outer_class_method_receiver = self.class_method_receiver.take();
        self.class_method_receiver = match self.types.kind(owner) {
            TyKind::Class(id)
                if signature_params.first().is_some_and(|(_, _, mode, _)| *mode == Mode::Mut)
                    && decl.params.first().is_some_and(|param| {
                        matches!(param.kind, ast::ParamKind::Receiver { .. })
                    }) =>
            {
                params.first().map(|param| (*id, param.local))
            }
            _ => None,
        };
        let class_init = match self.types.kind(owner) {
            TyKind::Class(id)
                if decl.name.name.is("init")
                    && signature_params.first().is_some_and(|(_, _, mode, _)| *mode == Mode::Mut)
                    && decl.params.first().is_some_and(|param| {
                        matches!(param.kind, ast::ParamKind::Receiver { .. })
                    }) =>
            {
                let supports_state = {
                    let mut current = Some(*id);
                    let mut supported = true;
                    while let Some(class_id) = current {
                        let def = self.types.class_def(class_id);
                        if def.fields.iter().any(|field| field.has_default) {
                            supported = false;
                            break;
                        }
                        current = def.base;
                    }
                    supported
                };
                let def = self.types.class_def(*id);
                if !supports_state {
                    None
                } else {
                    let receiver = params
                        .first()
                        .map(|param| param.local)
                        .expect("a class init with mut self has a receiver local");
                    Some(ClassInitState {
                        owner: *id,
                        receiver,
                        initialized: vec![
                            ClassFieldInit::Uninit;
                            self.types.class_field_count(*id)
                        ],
                        base_initialized: def.base.is_none(),
                    })
                }
            }
            _ => None,
        };
        let is_class_init = class_init.is_some();
        self.class_init = class_init;
        if self.class_init.is_some() {
            self.validate_class_init_shape(block, span);
        }
        let body = self.check_block(block);
        if self.class_init.is_some() {
            self.report_missing_class_init_fields(span);
        }
        self.class_init = outer_class_init;
        self.class_method_receiver = outer_class_method_receiver;
        self.self_ty = outer_self;
        let overflow = self.overflow_policy(attrs, span);
        self.in_static_safe = outer_static_safe;
        self.static_safe_unsafe_cell_reported = outer_static_safe_reported;
        let name = decl.name.name;
        Some(Function {
            def,
            name,
            class_init: is_class_init,
            symbol: method_symbol(&self.types.display(owner), name),
            is_unsafe: decl.is_unsafe,
            abi: decl.abi.clone(),
            params,
            locals: std::mem::take(&mut self.locals),
            ret: self.ret_ty,
            body,
            span,
            overflow,
            borrows: self.signatures[def.0 as usize].borrows.clone(),
            closure_environment: None,
            closure_captures_by_move: false,
        })
    }

    /// Validate the first source-reachable constructor slice. The runtime
    /// object is allocated before the body runs, so this conservative shape
    /// admits direct field initialization, `pass`, nested `if` blocks,
    /// block-bodied exhaustive `match` arms, and `while`/`for` bodies with an
    /// optional `else`. Loop entry is always possible, so each loop merge
    /// remains conservative. Expression-bodied match arms and other
    /// control-flow forms remain outside the slice until their constructor
    /// dataflow is connected. Whole-`self` use is checked against the current
    /// field state below.
    fn validate_class_init_shape(&mut self, block: &ast::Block, span: Span) {
        if self.class_init.is_none() {
            return;
        }
        for stmt in &block.stmts {
            match &stmt.kind {
                ast::StmtKind::Pass => {}
                ast::StmtKind::Assign { targets, op: None, .. }
                    if targets.len() == 1 && Self::class_init_target_name(&targets[0]).is_some() => {}
                ast::StmtKind::Expr(_) => {}
                ast::StmtKind::If(if_stmt) => {
                    self.validate_class_init_shape(&if_stmt.then_block, span);
                    match if_stmt.else_block.as_deref() {
                        Some(ast::ElseBranch::Block(block)) => {
                            self.validate_class_init_shape(block, span)
                        }
                        Some(ast::ElseBranch::If(nested)) => {
                            self.validate_class_init_if(nested, span)
                        }
                        None => {}
                    }
                }
                ast::StmtKind::Match { arms, .. }
                    if arms.iter().all(|arm| {
                        arm.guard.is_none()
                            && matches!(&arm.body, ast::MatchArmBody::Block(_))
                    }) =>
                {
                    for arm in arms {
                        let ast::MatchArmBody::Block(block) = &arm.body else {
                            unreachable!()
                        };
                        self.validate_class_init_shape(block, span);
                    }
                }
                ast::StmtKind::While { body, .. } => {
                    self.validate_class_init_shape(body, span);
                }
                ast::StmtKind::For { body, .. } => {
                    self.validate_class_init_shape(body, span);
                }
                _ => {
                    self.error(
                        codes::E1010,
                        stmt.span,
                        "class `init` currently supports direct field assignments, `pass`, `if`, block-bodied `match` arms, and loop bodies",
                    );
                }
            }
        }
    }

    fn validate_class_init_if(&mut self, if_stmt: &ast::IfStmt, span: Span) {
        self.validate_class_init_shape(&if_stmt.then_block, span);
        match if_stmt.else_block.as_deref() {
            Some(ast::ElseBranch::Block(block)) => self.validate_class_init_shape(block, span),
            Some(ast::ElseBranch::If(nested)) => self.validate_class_init_if(nested, span),
            None => {}
        }
    }

    /// A direct field projection is a field access, not a use of the whole
    /// object. Every other occurrence of `self` is classified as a whole-self
    /// use so the constructor checker can require complete initialization
    /// before passing, storing, or otherwise publishing the handle.
    fn class_init_uses_whole_self(expr: &ast::Expr) -> bool {
        match &expr.kind {
            ast::ExprKind::SelfExpr => true,
            ast::ExprKind::Lit(_) | ast::ExprKind::Path { .. } | ast::ExprKind::Error => false,
            ast::ExprKind::Field { base, .. } | ast::ExprKind::TupleField { base, .. } => {
                !matches!(base.kind, ast::ExprKind::SelfExpr) && Self::class_init_uses_whole_self(base)
            }
            ast::ExprKind::IndexOrInstantiate { base, args } => {
                Self::class_init_uses_whole_self(base)
                    || args.iter().any(|arg| match arg {
                        ast::TypeOrExpr::Expr(expr) => Self::class_init_uses_whole_self(expr),
                        _ => false,
                    })
            }
            ast::ExprKind::Call { callee, args } => {
                Self::class_init_uses_whole_self(callee)
                    || args.iter().any(|arg| Self::class_init_uses_whole_self(&arg.value))
            }
            ast::ExprKind::MethodCall { recv, args, .. } => {
                Self::class_init_uses_whole_self(recv)
                    || args.iter().any(|arg| Self::class_init_uses_whole_self(&arg.value))
            }
            ast::ExprKind::Unary { operand, .. }
            | ast::ExprKind::Try(operand)
            | ast::ExprKind::RefOf { place: operand, .. }
            | ast::ExprKind::Paren(operand)
            | ast::ExprKind::Owned(operand) => Self::class_init_uses_whole_self(operand),
            ast::ExprKind::Binary { lhs, rhs, .. }
            | ast::ExprKind::Logical { lhs, rhs, .. } => {
                Self::class_init_uses_whole_self(lhs) || Self::class_init_uses_whole_self(rhs)
            }
            ast::ExprKind::Ternary { then_expr, cond, else_expr } => {
                Self::class_init_uses_whole_self(then_expr)
                    || Self::class_init_uses_whole_self(cond)
                    || Self::class_init_uses_whole_self(else_expr)
            }
            ast::ExprKind::Range { lo, hi, .. } => {
                lo.as_deref().is_some_and(Self::class_init_uses_whole_self)
                    || hi.as_deref().is_some_and(Self::class_init_uses_whole_self)
            }
            ast::ExprKind::Cast { expr, .. } | ast::ExprKind::Downcast { expr, .. } => {
                Self::class_init_uses_whole_self(expr)
            }
            ast::ExprKind::OptChain { base, args, .. } => {
                Self::class_init_uses_whole_self(base)
                    || args.as_ref().is_some_and(|args| {
                        args.iter().any(|arg| Self::class_init_uses_whole_self(&arg.value))
                    })
            }
            // A lambda may capture `self`; classify it as whole-self until the
            // ordinary capture/lifetime checks establish the safe boundary.
            ast::ExprKind::Lambda(_) => true,
            ast::ExprKind::Match { .. } => true,
            ast::ExprKind::Tuple(items) | ast::ExprKind::ArrayLit(items) => {
                items.iter().any(Self::class_init_uses_whole_self)
            }
            ast::ExprKind::ArrayRepeat { value, count } => {
                Self::class_init_uses_whole_self(value) || Self::class_init_uses_whole_self(count)
            }
            ast::ExprKind::FString(parts) => parts.iter().any(|part| match part {
                ast::FStringPart::Text(_) => false,
                ast::FStringPart::Expr { expr, .. } => Self::class_init_uses_whole_self(expr),
            }),
            ast::ExprKind::Jump(jump) => match jump {
                ast::Jump::Return(value) => value.as_deref().is_some_and(Self::class_init_uses_whole_self),
                ast::Jump::Break { .. } | ast::Jump::Continue { .. } => false,
            },
            ast::ExprKind::Yield(value) => value.as_deref().is_some_and(Self::class_init_uses_whole_self),
        }
    }

    fn class_init_is_complete(&self) -> bool {
        self.class_init.as_ref().is_some_and(|state| {
            state.base_initialized
                && state
                    .initialized
                    .iter()
                    .all(|status| matches!(status, ClassFieldInit::Init))
        })
    }

    /// `[CLS-2]` permits the receiver to be used as a whole only after every
    /// required field is initialized on the current path. Keep this check at
    /// expression boundaries rather than in `synth(SelfExpr)`: a direct
    /// `self.field` projection is a field access and is checked separately.
    fn check_class_init_whole_self_use(&mut self, expr: &ast::Expr) {
        if self.class_init.is_some()
            && !self.class_init_is_complete()
            && Self::class_init_uses_whole_self(expr)
        {
            self.error(
                codes::E2100,
                expr.span,
                "`self` is used before all class fields are initialized",
            );
        }
    }

    fn report_missing_class_init_fields(&mut self, span: Span) {
        let Some(state) = self.class_init.as_ref().cloned() else { return };
        if self.types.class_def(state.owner).base.is_some() && !state.base_initialized {
            self.error(
                codes::E2100,
                span,
                "derived class `init` must call `super.init(...)` exactly once before returning",
            );
        }
        let base_count = self
            .types
            .class_def(state.owner)
            .base
            .map_or(0, |base| self.types.class_field_count(base));
        let missing: Vec<Symbol> = state
            .initialized
            .iter()
            .enumerate()
            .filter_map(|(index, status)| {
                (index >= base_count && !matches!(status, ClassFieldInit::Init))
                    .then(|| self.types.class_field_at(state.owner, index).map(|f| f.name))
                    .flatten()
            })
            .collect();
        for field in missing {
            self.error(
                codes::E2100,
                span,
                format!("field `{field}` is not definitely initialized by class `init`"),
            );
        }
    }

    fn merge_class_init_paths(
        left: Option<ClassInitState>,
        right: Option<ClassInitState>,
    ) -> Option<ClassInitState> {
        match (left, right) {
            (Some(mut left), Some(right)) if left.owner == right.owner && left.receiver == right.receiver => {
                for (left, right) in left.initialized.iter_mut().zip(right.initialized) {
                    *left = left.join(right);
                }
                left.base_initialized &= right.base_initialized;
                Some(left)
            }
            (left, _) => left,
        }
    }

    fn merge_class_init_path_list(paths: Vec<ClassInitState>) -> Option<ClassInitState> {
        let mut paths = paths.into_iter();
        let mut merged = paths.next()?;
        for path in paths {
            merged = Self::merge_class_init_paths(Some(merged), Some(path))?;
        }
        Some(merged)
    }

    fn class_init_target_name(expr: &ast::Expr) -> Option<Symbol> {
        match &expr.kind {
            ast::ExprKind::Field { base, name }
                if matches!(base.kind, ast::ExprKind::SelfExpr) =>
            {
                Some(name.name)
            }
            _ => None,
        }
    }

    fn class_init_field_index(&self, expr: &Expr) -> Option<usize> {
        let state = self.class_init.as_ref()?;
        let ExprKind::Field { base, index } = &expr.kind else { return None };
        let ExprKind::Deref(receiver) = &base.kind else { return None };
        let ExprKind::Local(local) = &receiver.kind else { return None };
        if *local != state.receiver {
            return None;
        }
        matches!(self.types.kind(base.ty), TyKind::Class(id) if *id == state.owner)
            .then_some(*index)
    }

    /// Return the field index when `expr` is rooted in the current class
    /// method's `mut self` receiver.  Unlike the constructor helper above,
    /// this deliberately ignores the initialization lattice: a mutable class
    /// method may update an already initialized field, including through an
    /// index or another nested projection.  The caller walks to the root
    /// field before asking, so a write such as `self.items[i] = value` is
    /// covered without accidentally admitting `other.items[i]`.
    fn class_method_field_index(&self, expr: &Expr) -> Option<usize> {
        let (owner, receiver) = self.class_method_receiver?;
        let ExprKind::Field { base, index } = &expr.kind else { return None };
        let ExprKind::Deref(receiver_expr) = &base.kind else { return None };
        let ExprKind::Local(local) = &receiver_expr.kind else { return None };
        if *local != receiver {
            return None;
        }
        matches!(self.types.kind(base.ty), TyKind::Class(id) if *id == owner).then_some(*index)
    }

    /// Return whether a mutable call argument has a class object whose access
    /// place the current MIR slice can identify without re-evaluating an
    /// indexed class handle. Array/collection projections rooted in a class
    /// field are fine; an indexed class object remains fail-closed until its
    /// bounds-checking evaluation can be shared with the access interval.
    fn class_mut_argument_supported(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Field { base, .. } | ExprKind::Index { base, .. } | ExprKind::Deref(base) => {
                if matches!(self.types.kind(base.ty), TyKind::Class(_)) {
                    !matches!(base.kind, ExprKind::Index { .. })
                } else {
                    self.class_mut_argument_supported(base)
                }
            }
            _ => false,
        }
    }

    fn check_class_init_field_read(&mut self, index: usize, span: Span) {
        let Some(state) = self.class_init.as_ref() else { return };
        let base_count = self
            .types
            .class_def(state.owner)
            .base
            .map_or(0, |base| self.types.class_field_count(base));
        if index < base_count && !state.base_initialized {
            self.error(
                codes::E2100,
                span,
                "inherited class field is read before `super.init(...)`",
            );
            return;
        }
        if matches!(state.initialized.get(index), Some(ClassFieldInit::Init)) {
            return;
        }
        let Some(field) = self.types.class_field_at(state.owner, index) else { return };
        self.error(
            codes::E2100,
            span,
            format!("field `{}` is read before it is initialized", field.name),
        );
    }

    fn mark_class_init_field(&mut self, place: &Expr) {
        let Some(index) = self.class_init_field_index(place) else { return };
        let (owner, base_initialized) = self
            .class_init
            .as_ref()
            .map(|state| (state.owner, state.base_initialized))
            .expect("class-init field index implies constructor state");
        let base_count = self
            .types
            .class_def(owner)
            .base
            .map_or(0, |base| self.types.class_field_count(base));
        if index < base_count && !base_initialized {
            self.error(
                codes::E2100,
                place.span,
                "inherited class field is assigned before `super.init(...)`",
            );
            return;
        }
        let status = self
            .class_init
            .as_ref()
            .and_then(|state| state.initialized.get(index).copied());
        match status {
            Some(ClassFieldInit::Init) => {
                let field = self
                    .class_init
                    .as_ref()
                    .and_then(|state| self.types.class_field_at(state.owner, index))
                    .map(|field| field.name);
                if let Some(field) = field {
                    self.error(
                        codes::E1010,
                        place.span,
                        format!("class `init` assigns field `{field}` more than once in this phase"),
                    );
                }
            }
            Some(ClassFieldInit::Maybe) => {
                let field = self
                    .class_init
                    .as_ref()
                    .and_then(|state| self.types.class_field_at(state.owner, index))
                    .map(|field| field.name);
                if let Some(field) = field {
                    self.error(
                        codes::E1010,
                        place.span,
                        format!("class `init` cannot overwrite conditionally initialized field `{field}` in this phase"),
                    );
                }
            }
            Some(ClassFieldInit::Uninit) => {
                if let Some(state) = self.class_init.as_mut() {
                    if let Some(initialized) = state.initialized.get_mut(index) {
                        *initialized = ClassFieldInit::Init;
                    }
                }
            }
            None => {}
        }
    }

    /// Mark the direct base storage initialized after a validated
    /// `super.init(...)` call. A second call is rejected before it can make
    /// the base constructor run twice.
    fn mark_class_init_base(&mut self, span: Span) {
        let Some(state) = self.class_init.as_mut() else { return };
        if state.base_initialized {
            self.error(codes::E1010, span, "`super.init(...)` may be called only once");
            return;
        }
        let base_count = self
            .types
            .class_def(state.owner)
            .base
            .map_or(0, |base| self.types.class_field_count(base));
        for status in state.initialized.iter_mut().take(base_count) {
            *status = ClassFieldInit::Init;
        }
        state.base_initialized = true;
    }

    /// `[TYP-8]` — `@overflow(panic|wrap|saturate)` overrides the profile.
    fn overflow_policy(&mut self, attrs: &[ast::Attribute], span: Span) -> OverflowPolicy {
        let Some(attr) = attrs
            .iter()
            .find(|a| a.path.len() == 1 && a.path[0].name.is("overflow"))
        else {
            return self.default_overflow;
        };
        let named = attr.args.iter().find_map(|arg| match arg {
            ast::AttrArg::Expr(e) => match &e.kind {
                ast::ExprKind::Path { segments } if segments.len() == 1 => Some(segments[0].name),
                _ => None,
            },
            ast::AttrArg::Named { .. } => None,
        });
        let Some(name) = named else {
            self.error(codes::E0104, attr.span, "`@overflow` needs `panic`, `wrap` or `saturate`");
            return self.default_overflow;
        };
        match OverflowPolicy::from_name(name.as_str()) {
            Some(OverflowPolicy::Saturate) => {
                // The lowering for saturation needs each type's bounds as
                // constants, which the backend cannot yet render for signed
                // minimums. Rejected rather than silently treated as `wrap`.
                self.error(
                    codes::E0104,
                    attr.span,
                    "`@overflow(saturate)` is not supported yet in this phase of the compiler",
                );
                self.default_overflow
            }
            Some(policy) => policy,
            None => {
                let _ = span;
                self.error(
                    codes::E0104,
                    attr.span,
                    format!("`{name}` is not an overflow policy; use `panic`, `wrap` or `saturate`"),
                );
                self.default_overflow
            }
        }
    }

    fn declare(&mut self, name: Option<Symbol>, ty: Ty, span: Span) -> LocalId {
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(LocalDecl { name, ty, span, for_iterator: false });
        if let Some(name) = name {
            self.scopes.last_mut().expect("a scope is open").insert(name, id);
        }
        id
    }

    /// Reading a local by name. A `mut` parameter holds a reference, so the
    /// name means the thing it points at.
    /// `[TYP-14]` — "use of `r` in an expression of type `T` reads through".
    /// The rule is about a context that wants `T`; a context that wants
    /// `ref T` wants the reference itself, which is how a reference is
    /// returned or passed on. Without the expectation there is no way to
    /// name the reference at all, since every mention would deref.
    fn read_local_expecting(
        &mut self,
        local: LocalId,
        span: Span,
        expected: Option<Ty>,
    ) -> Expr {
        let ty = self.locals[local.0 as usize].ty;
        let read = Expr { ty, kind: ExprKind::Local(local), span };
        // `[CELL-7]` — a guard local reads through like a reference, unless
        // the context wants the guard itself: returning or passing on the
        // guard names it, every other mention names its contents while keeping
        // the guard alive (the unwrapped place derefs through it).
        if self.ref_guard_inner(ty).is_some() {
            if let Some(expected) = expected {
                if expected == ty {
                    return read;
                }
            }
            return self.read_guard_through(read);
        }
        let TyKind::Ref { inner, .. } = *self.types.kind(ty) else { return read };
        if let Some(expected) = expected {
            if expected == ty || matches!(self.types.kind(expected), TyKind::Ref { .. }) {
                return read;
            }
        }
        let _ = inner;
        Expr { ty: inner, kind: ExprKind::Deref(Box::new(read)), span }
    }

    /// `[TYP-14]` — "use of `r` in an expression of type `T` reads through",
    /// where the context wants a value and no expected type says so.
    ///
    /// `coerce` covers every site that has an expectation to compare against.
    /// An operand of `+` and a builtin's argument have none: they want *a
    /// value*, and a reference reaching one of them unread is a pointer in the
    /// emitted C. `println(f(ref n))` printed a `ref i32` as if it were a
    /// string until this existed.
    fn read_through(&mut self, expr: Expr) -> Expr {
        if self.ref_guard_inner(expr.ty).is_some() {
            return self.read_guard_through(expr);
        }
        let TyKind::Ref { inner, .. } = *self.types.kind(expr.ty) else { return expr };
        let span = expr.span;
        Expr { ty: inner, kind: ExprKind::Deref(Box::new(expr)), span }
    }

    /// `[CELL-7]` — reading through a `Ref[T]`/`RefMut[T]` guard to its
    /// contents. The guard holds a single private `ref` field; projecting to
    /// it and dereferencing yields `T` while keeping the guard alive (the
    /// resulting place derefs through the guard's local, so liveness and the
    /// region graph still see it).
    fn read_guard_through(&mut self, guard: Expr) -> Expr {
        let Some((inner, _)) = self.ref_guard_inner(guard.ty) else { return guard };
        let span = guard.span;
        let field_ty = self.types.struct_def(match self.types.kind(guard.ty) {
            TyKind::Struct(id) => *id,
            _ => unreachable!("guard is a struct"),
        }).fields[0].ty;
        let field = Expr { ty: field_ty, kind: ExprKind::Field { base: Box::new(guard), index: 0 }, span };
        Expr { ty: inner, kind: ExprKind::Deref(Box::new(field)), span }
    }

    fn lookup(&self, name: Symbol) -> Option<LocalId> {
        self.scopes.iter().rev().find_map(|scope| scope.get(&name).copied())
    }

    /// `[CLO-2]` — whether a name a closure body could not find is one the
    /// enclosing function declares, in which case it is a **capture** and not
    /// a typo.
    /// `[CLO-2]` — resolve a name the closure body did not declare and the
    /// enclosing function did.
    ///
    /// On the discovery pass this records the capture and hands back a
    /// placeholder of the right type, so the rest of the body checks as if the
    /// name were an ordinary local. On the real pass the environment exists:
    /// a normal closure reads through its shared-borrow field, while an
    /// `owned fn` reads its directly stored move/copy capture.
    fn resolve_capture(&mut self, name: Symbol, span: Span) -> Option<Expr> {
        let ((env_local, struct_id), captures_by_move) = {
            let watch = self.captures.as_mut()?;
            let ty = *watch.outer.get(&name)?;
            match watch.env {
                None => {
                    if !watch.found.iter().any(|(n, _)| *n == name) {
                        watch.found.push((name, ty));
                    }
                    return Some(Expr { ty, kind: ExprKind::Error, span });
                }
                Some(env) => (env, watch.captures_by_move),
            }
        };
        let index = self
            .captures
            .as_ref()?
            .found
            .iter()
            .position(|(n, _)| *n == name)?;
        let env_ty = self.types.intern(TyKind::Struct(struct_id));
        let field_ty = self.types.struct_def(struct_id).fields[index].ty;
        let base = self.read_local_expecting(env_local, span, Some(env_ty));
        let field = Expr {
            ty: field_ty,
            kind: ExprKind::Field { base: Box::new(base), index },
            span,
        };
        Some(if captures_by_move { field } else { self.read_through(field) })
    }

    /// A lambda body has no outer locals in its lexical scope. Before treating
    /// a bare assignment as a new local (`[GRM-4]`), preserve an outer name as
    /// a capture candidate so `counter = …` can mean mutation of `counter`.
    fn is_capture_candidate(&self, name: Symbol) -> bool {
        self.captures
            .as_ref()
            .is_some_and(|watch| watch.outer.contains_key(&name))
    }

    fn check_block(&mut self, block: &ast::Block) -> Block {
        // `[Part 0 #13]` — block scoping: names declared here leave scope at
        // the end of the block.
        self.scopes.push(HashMap::new());
        let mut stmts = Vec::new();
        for stmt in &block.stmts {
            self.check_stmt(stmt, &mut stmts);
        }
        self.scopes.pop();
        Block { stmts, span: block.span }
    }

    fn check_stmt(&mut self, stmt: &ast::Stmt, out: &mut Vec<Stmt>) {
        match &stmt.kind {
            ast::StmtKind::Pass => {}
            // `[GRM-16]` — a jump written as a statement arrives as an
            // expression statement, and is the one expression that lowers to
            // control flow rather than to a value.
            ast::StmtKind::Expr(expr) => {
                self.check_class_init_whole_self_use(expr);
                if let ast::ExprKind::Jump(jump) = &expr.kind {
                    self.lower_jump(jump, expr.span, out);
                    return;
                }
                let expr = self.synth(expr);
                out.push(Stmt::Expr(expr));
            }
            ast::StmtKind::Decl { pattern, ty, init } => {
                let Some(name) = binding_name(pattern) else {
                    self.error(codes::E1010, stmt.span, "this pattern is not supported yet");
                    return;
                };
                let declared = ty.as_ref().map(|t| self.resolve_type(t));
                let init = match (init, declared) {
                    (Some(expr), Some(expected)) => Some(self.check_expr(expr, expected)),
                    (Some(expr), None) => Some(self.synth_committed(expr)),
                    (None, _) => None,
                };
                let ty = declared
                    .or_else(|| init.as_ref().map(|e| e.ty))
                    .unwrap_or(self.common.error);
                if declared.is_none() && init.is_none() {
                    self.error(codes::E2060, stmt.span, format!("cannot infer the type of `{name}`"));
                }
                let local = self.declare(Some(name), ty, stmt.span);
                if let Some(range) = init.as_ref().and_then(|e| self.range_of(e)) {
                    self.local_ranges.insert(local, range);
                }
                out.push(Stmt::Let { local, init });
            }
            ast::StmtKind::Assign { targets, op, value } => {
                self.check_class_init_whole_self_use(value);
                let parenthesised = targets.len() == 1
                    && matches!(&targets[0].kind, ast::ExprKind::Tuple(_));
                if targets.len() != 1 || parenthesised {
                    if op.is_some() {
                        self.error(
                            codes::E1010,
                            stmt.span,
                            "tuple destructuring does not support augmented assignment",
                        );
                        return;
                    }
                    self.check_destructure(targets, value, stmt.span, out);
                    return;
                }
                let target = &targets[0];

                // `[GRM-4]` — a bare name that is not in scope declares.
                if let ast::ExprKind::Path { segments } = &target.kind {
                    if segments.len() == 1
                        && self.lookup(segments[0].name).is_none()
                        && !self.is_capture_candidate(segments[0].name)
                    {
                        if op.is_some() {
                            self.error(
                                codes::E1010,
                                target.span,
                                format!("cannot find `{}` in this scope", segments[0].name),
                            );
                            return;
                        }
                        let init = self.synth_committed(value);
                        let local = self.declare(Some(segments[0].name), init.ty, stmt.span);
                        if let Some(range) = self.range_of(&init) {
                            self.local_ranges.insert(local, range);
                        }
                        out.push(Stmt::Let { local, init: Some(init) });
                        return;
                    }
                }

                let previous_target = self.in_assignment_target;
                self.in_assignment_target = true;
                let place = self.synth(target);
                self.in_assignment_target = previous_target;
                // A written local no longer holds whatever `[RNG-4]` derived
                // at its initialiser. Dropped rather than joined: the
                // conservative direction here is a check that gets emitted.
                if let ExprKind::Local(local) = place.kind {
                    self.local_ranges.remove(&local);
                }
                // `[MOD-7]` — assignment and augmented assignment are both
                // writes.
                self.reject_readonly_write_in_assignment(&place, target.span);
                let through_shared_ref =
                    self.reject_write_through_shared_ref(&place, target.span);
                if !through_shared_ref {
                    self.reject_borrowed_parameter_write(&place, target.span, true);
                }
                let place_ty = place.ty;
                // `[RNG-5a1]` — "No `*Assign` form is generated: `r += 1.0`
                // would produce an `R` where a `T` is required and is
                // `E2214`". The fallback `a = a op b` of `[TYP-21]` is exactly
                // what produces that `R`, so it must not be reached.
                if let (Some(bin), TyKind::Range(id)) = (op, self.types.kind(place_ty)) {
                    let def = self.types.range_def(*id);
                    let name = def.name.to_string();
                    let repr = self.types.display(def.repr);
                    self.sink.emit(
                        Diagnostic::error(
                            codes::E2214,
                            stmt.span,
                            format!("`{}=` is not defined on `{name}`", bin.as_str()),
                        )
                        .primary_label(format!(
                            "this would produce a `{repr}`, and `{name}` is wanted"
                        ))
                        .help(match &target.kind {
                            ast::ExprKind::Path { segments } if segments.len() == 1 => {
                                let p = segments[0].name;
                                format!(
                                    "write the construction: `{p} = {name}.clamped({p} {} …)`, \
                                     or `{name}.checked(…)` where the difference matters",
                                    bin.as_str()
                                )
                            }
                            _ => format!(
                                "write the construction: assign `{name}.clamped(…)`, or \
                                 `{name}.checked(…)` where the difference matters"
                            ),
                        })
                        .note(
                            "arithmetic on a range type yields its representation, and \
                             producing a range value again is a construction [RNG-3]",
                        ),
                    );
                    return;
                }
                let value = match op {
                    // `a op= b` is `a = a op b` when no `AddAssign` exists
                    // (`[TYP-21]`); Phase 0 has scalars only, so it always is.
                    Some(bin) => {
                        let rhs = self.check_expr(value, place_ty);
                        let lhs = self.synth(target);
                        let op = convert_binop(*bin);
                        match op {
                            Some(op) => Expr {
                                ty: place_ty,
                                kind: ExprKind::Binary {
                                    op,
                                    lhs: Box::new(lhs),
                                    rhs: Box::new(rhs),
                                },
                                span: stmt.span,
                            },
                            None => {
                                self.error(
                                    codes::E1010,
                                    stmt.span,
                                    "this operator is not supported yet in this phase",
                                );
                                return;
                            }
                        }
                    }
                    None => self.check_expr(value, place_ty),
                };
                if op.is_none() {
                    self.mark_class_init_field(&place);
                }
                out.push(Stmt::Assign { place, value });
            }
            ast::StmtKind::If(if_stmt) => {
                let stmt = self.check_if(if_stmt);
                out.push(stmt);
            }
            ast::StmtKind::While { label, cond, body, else_block } => {
                let incoming_class_init = self.class_init.clone();
                match cond {
                    ast::Condition::Expr(expr) => self.check_class_init_whole_self_use(expr),
                    ast::Condition::Pattern { value, .. } => {
                        self.check_class_init_whole_self_use(value)
                    }
                }
                let cond = self.check_condition(cond);
                self.class_init = incoming_class_init.clone();
                self.loop_labels.push(label.map(|l| l.name));
                let body = self.check_block(body);
                self.loop_labels.pop();
                let body_class_init = self.class_init.clone();
                if else_block.is_some() && incoming_class_init.is_some() {
                    // A loop may execute zero times before its `else` runs.
                    // Check the else block from the join of loop entry and
                    // one-or-more body iterations, not from the body alone.
                    // Otherwise a field written only by the body would look
                    // definitely initialized in the else block.
                    self.class_init = Self::merge_class_init_paths(
                        incoming_class_init.clone(),
                        body_class_init.clone(),
                    );
                }
                let else_block = else_block.as_ref().map(|b| self.check_block(b));
                if incoming_class_init.is_some() && else_block.is_none() {
                    // A while body may execute zero times. Initialization
                    // performed only in the body is therefore not definite
                    // after the loop; the normal path is the join with the
                    // state at loop entry.
                    self.class_init = Self::merge_class_init_paths(
                        incoming_class_init,
                        body_class_init,
                    );
                }
                out.push(Stmt::While { cond, body, else_block });
            }
            ast::StmtKind::For { label, pattern, iter, body, else_block } => {
                let incoming_class_init = self.class_init.clone();
                self.check_class_init_whole_self_use(iter);
                if let Some(stmt) = self.check_for(*label, pattern, iter, body, else_block, stmt.span)
                {
                    let body_class_init = self.class_init.clone();
                    if incoming_class_init.is_some() && else_block.is_none() {
                        // A for loop may have zero iterations. The body can
                        // establish a field only on the iterated path, so
                        // merge it with the state before lowering/desugaring
                        // the loop.
                        self.class_init = Self::merge_class_init_paths(
                            incoming_class_init,
                            body_class_init,
                        );
                    }
                    out.push(stmt);
                }
            }
            // `[CTL-7]` — `defer:` runs at scope exit. Nothing about the block
            // itself is special; the ordering is applied when it is lowered.
            // `[UNS-2]` — an `unsafe:` block does not turn anything off. It
            // only permits `[UNS-1]`'s operations, which are refused
            // everywhere else.
            ast::StmtKind::Unsafe(block) => {
                let was = std::mem::replace(&mut self.in_unsafe, true);
                let block = self.check_block(block);
                self.in_unsafe = was;
                out.push(Stmt::Block(block));
            }
            ast::StmtKind::Defer(block) => {
                // `[CTL-7]` — control may not leave a `defer` block, so the
                // loops outside it are out of reach and `return` is `E2160`.
                let outer_loops = std::mem::take(&mut self.loop_labels);
                let was_in_defer = std::mem::replace(&mut self.in_defer, true);
                let block = self.check_block(block);
                self.in_defer = was_in_defer;
                self.loop_labels = outer_loops;
                out.push(Stmt::Defer(block));
            }
            // `[CTL-6]` — `with a = e:` binds `a` for the block. Without
            // destructors there is nothing to drop at the end yet, so this is
            // a scope with bindings in it.
            ast::StmtKind::With { items, body } => {
                self.scopes.push(HashMap::new());
                let mut stmts = Vec::new();
                for item in items {
                    let value = self.synth_committed(&item.value);
                    let name = item.pattern.as_ref().and_then(binding_name);
                    let local = self.declare(name, value.ty, item.value.span);
                    stmts.push(Stmt::Let { local, init: Some(value) });
                }
                let inner = self.check_block(body);
                self.scopes.pop();
                stmts.extend(inner.stmts);
                out.push(Stmt::Block(Block { stmts, span: stmt.span }));
            }
            // A statement `match` produces no value, so its arms are checked
            // against `void` and it rides in `Stmt::Expr`.
            ast::StmtKind::Match { scrutinee, arms } => {
                let void = self.common.void;
                let matched = self.check_match(scrutinee, arms, Some(void), stmt.span);
                out.push(Stmt::Expr(matched));
            }
            _ => {
                self.error(
                    codes::E1010,
                    stmt.span,
                    "this statement is not supported yet in this phase of the compiler",
                );
            }
        }
    }

    /// `[GRM-5]` — a target list is one aggregate assignment, not a sequence
    /// of unrelated assignments. The right-hand side is checked before any
    /// new binding enters scope, then represented by one compiler-private HIR
    /// temporary so MIR can evaluate it once and end its residual lifetime at
    /// this statement.
    fn check_destructure(
        &mut self,
        targets: &[ast::Expr],
        value: &ast::Expr,
        span: Span,
        out: &mut Vec<Stmt>,
    ) {
        let value = self.synth_committed(value);
        if value.ty == self.common.error {
            return;
        }

        let root_targets = if targets.len() == 1 {
            match &targets[0].kind {
                ast::ExprKind::Tuple(items) => items.as_slice(),
                _ => targets,
            }
        } else {
            targets
        };
        let mut leaves = Vec::new();
        if !self.collect_destructure_leaves(
            root_targets,
            value.ty,
            &mut Vec::new(),
            &mut leaves,
            span,
        ) {
            return;
        }

        // `[GRM-5]`'s consistency rule: `_` is neutral, while every other
        // leaf must be either a fresh bare name or an existing writable place.
        let mut declares = false;
        let mut assigns = false;
        let mut fresh = HashSet::new();
        for leaf in &leaves {
            if is_single_path(leaf.target, "_") {
                continue;
            }
            match &leaf.target.kind {
                ast::ExprKind::Path { segments } if segments.len() == 1 => {
                    let name = segments[0].name;
                    if self.lookup(name).is_some() {
                        assigns = true;
                    } else {
                        declares = true;
                        if !fresh.insert(name) {
                            self.error(
                                codes::E1020,
                                leaf.target.span,
                                format!("`{name}` is declared more than once in this destructuring"),
                            );
                            return;
                        }
                    }
                }
                _ => assigns = true,
            }
        }
        if declares && assigns {
            self.error(
                codes::E1010,
                span,
                "all names in a destructuring assignment must be declared or assigned consistently",
            );
            return;
        }

        let aggregate_ty = value.ty;
        let temp = self.declare(None, aggregate_ty, value.span);
        let mut bindings = Vec::new();
        for leaf in leaves {
            if is_single_path(leaf.target, "_") {
                continue;
            }
            let source = self.destructure_source(temp, aggregate_ty, &leaf.projection, leaf.target.span);
            if declares {
                let ast::ExprKind::Path { segments } = &leaf.target.kind else {
                    unreachable!("declaration consistency admitted a non-name target")
                };
                let local = self.declare(Some(segments[0].name), leaf.ty, leaf.target.span);
                bindings.push(DestructureBinding::Let { local, value: source });
                continue;
            }

            let place = self.synth(leaf.target);
            if place.ty == self.common.error {
                continue;
            }
            if !is_place(&place.kind) {
                self.error(codes::E2140, leaf.target.span, "assignment target is not a place");
                continue;
            }
            if let ExprKind::Local(local) = place.kind {
                self.local_ranges.remove(&local);
            }
            self.reject_readonly_write_in_assignment(&place, leaf.target.span);
            let through_shared_ref =
                self.reject_write_through_shared_ref(&place, leaf.target.span);
            if !through_shared_ref {
                self.reject_borrowed_parameter_write(&place, leaf.target.span, true);
            }
            let place_ty = place.ty;
            let source = self.coerce(source, place_ty);
            bindings.push(DestructureBinding::Assign { place, value: source });
        }
        out.push(Stmt::Destructure { temp, value, bindings });
    }

    /// Expand one aggregate target list into leaves while retaining the exact
    /// field path for each leaf. Tuples and structs both use declaration-order
    /// field indices, as `[GRM-5]` requires; parenthesised target lists recurse.
    fn collect_destructure_leaves<'b>(
        &mut self,
        targets: &'b [ast::Expr],
        aggregate: Ty,
        projection: &mut Vec<usize>,
        out: &mut Vec<DestructureLeaf<'b>>,
        span: Span,
    ) -> bool {
        let fields = match self.types.kind(aggregate).clone() {
            TyKind::Tuple(items) => items,
            TyKind::Struct(id) => {
                let fields: Vec<(Ty, FieldVis, Symbol)> = self
                    .types
                    .struct_def(id)
                    .fields
                    .iter()
                    .map(|field| (field.ty, field.vis, field.name))
                    .collect();
                for (_, vis, name) in &fields {
                    self.check_field_visible(id, *vis, *name, span);
                }
                fields.into_iter().map(|(ty, _, _)| ty).collect()
            }
            _ => {
                let shown = self.types.display(aggregate);
                self.error(
                    codes::E2020,
                    span,
                    format!("destructuring requires a tuple or struct, found `{shown}`"),
                );
                return false;
            }
        };
        if fields.len() != targets.len() {
            let shown = self.types.display(aggregate);
            self.error(
                codes::E2020,
                span,
                format!(
                    "`{shown}` has {} fields, but this destructuring has {} targets",
                    fields.len(),
                    targets.len()
                ),
            );
            return false;
        }

        for (index, (target, ty)) in targets.iter().zip(fields).enumerate() {
            projection.push(index);
            let ok = match &target.kind {
                ast::ExprKind::Tuple(items) => {
                    self.collect_destructure_leaves(items, ty, projection, out, target.span)
                }
                _ => {
                    out.push(DestructureLeaf {
                        target,
                        ty,
                        projection: projection.clone(),
                    });
                    true
                }
            };
            projection.pop();
            if !ok {
                return false;
            }
        }
        true
    }

    fn destructure_source(
        &self,
        temp: LocalId,
        aggregate: Ty,
        projection: &[usize],
        span: Span,
    ) -> Expr {
        let mut source = Expr { ty: aggregate, kind: ExprKind::Local(temp), span };
        for &index in projection {
            let ty = match self.types.kind(source.ty) {
                TyKind::Tuple(items) => items[index],
                TyKind::Struct(id) => self.types.struct_def(*id).fields[index].ty,
                _ => unreachable!("validated destructuring projection"),
            };
            source = Expr { ty, kind: ExprKind::Field { base: Box::new(source), index }, span };
        }
        source
    }

    // -- patterns and `match` -------------------------------------------------

    /// One `match`, as an expression. A statement `match` is this with an
    /// expected type of `void`, which `check_stmt` wraps in `Stmt::Expr`.
    /// `[GRM-16]` — lower `return e`, `break l` and `continue l` into the
    /// HIR's control-flow statements. A jump is always the whole of the
    /// expression it appears in, so every position that can hold one is
    /// statement-like: an expression statement, or a `=>` arm of a `match`.
    fn lower_jump(&mut self, jump: &ast::Jump, span: Span, out: &mut Vec<Stmt>) {
        match jump {
            ast::Jump::Return(value) => {
                // `[CTL-7]` — a jump may not leave a `defer` block.
                if self.in_defer {
                    self.error(codes::E2160, span, "control flow cannot leave a `defer` block");
                }
                let ret_ty = self.ret_ty;
                let value = match value {
                    Some(expr) => Some(self.check_expr(expr, ret_ty)),
                    None => {
                        if ret_ty != self.common.void {
                            let shown = self.types.display(ret_ty);
                            self.error(
                                codes::E2020,
                                span,
                                format!("this function returns `{shown}`, so `return` needs a value"),
                            );
                        }
                        None
                    }
                };
                out.push(Stmt::Return(value));
            }
            ast::Jump::Break { label } => {
                if self.in_defer {
                    self.error(codes::E2160, span, "control flow cannot leave a `defer` block");
                }
                if let Some(depth) = self.loop_depth(*label, "break", span) {
                    out.push(Stmt::Break { depth });
                }
            }
            ast::Jump::Continue { label } => {
                if self.in_defer {
                    self.error(codes::E2160, span, "control flow cannot leave a `defer` block");
                }
                if let Some(depth) = self.loop_depth(*label, "continue", span) {
                    out.push(Stmt::Continue { depth });
                }
            }
        }
    }

    fn check_match(
        &mut self,
        scrutinee: &ast::Expr,
        arms: &[ast::MatchArm],
        expected: Option<Ty>,
        span: Span,
    ) -> Expr {
        self.check_class_init_whole_self_use(scrutinee);
        let scrutinee = self.synth_committed(scrutinee);
        let scrutinee_ty = scrutinee.ty;

        let mut checked: Vec<hir::MatchArm> = Vec::new();
        let mut result_ty = expected;
        // Constructor definite-initialization is path-sensitive. Checking
        // match arms one after another must not let writes from an earlier
        // arm leak into a later arm; only the joined post-match state is
        // reachable. Guarded arms retain the incoming state as a possible
        // fall-through path because a guard may fail at run time.
        let incoming_class_init = self.class_init.clone();
        let mut arm_class_init = Vec::new();
        let mut has_guard = false;
        for arm in arms {
            if let Some(incoming) = incoming_class_init.as_ref() {
                self.class_init = Some(incoming.clone());
            }
            self.scopes.push(HashMap::new());
            let pattern = self.check_pattern(&arm.pattern, scrutinee_ty);
            let guard = arm.guard.as_ref().map(|g| {
                has_guard = true;
                let bool_ty = self.common.bool_;
                self.check_expr(g, bool_ty)
            });
            let body = match &arm.body {
                ast::MatchArmBody::Block(block) => {
                    hir::MatchArmBody::Block(self.check_block(block))
                }
                // `[GRM-16]` — `Circle(r) => return PI * r * r`. A jump has
                // type `!` and produces no value, so the arm lowers to a block
                // holding the control-flow statement, and settles no type.
                ast::MatchArmBody::Expr(expr) if matches!(expr.kind, ast::ExprKind::Jump(_)) => {
                    let ast::ExprKind::Jump(jump) = &expr.kind else { unreachable!() };
                    let mut stmts = Vec::new();
                    self.lower_jump(jump, expr.span, &mut stmts);
                    hir::MatchArmBody::Block(Block { stmts, span: expr.span })
                }
                ast::MatchArmBody::Expr(expr) => {
                    // The first arm settles the type; the rest are checked
                    // against it, so a mismatch points at the arm that differs.
                    let value = match result_ty {
                        Some(ty) => self.check_expr(expr, ty),
                        None => self.synth_committed(expr),
                    };
                    if result_ty.is_none() && value.ty != self.common.never {
                        result_ty = Some(value.ty);
                    }
                    hir::MatchArmBody::Expr(value)
                }
            };
            self.scopes.pop();
            if incoming_class_init.is_some() {
                arm_class_init.push(self.class_init.clone());
            }
            checked.push(hir::MatchArm { pattern, guard, body, span: arm.span });
        }

        self.report_match_coverage(&checked, scrutinee_ty, span);

        if let Some(incoming) = incoming_class_init {
            let mut paths = if has_guard { vec![incoming] } else { Vec::new() };
            paths.extend(arm_class_init.into_iter().flatten());
            self.class_init = Self::merge_class_init_path_list(paths);
        }

        let ty = result_ty.unwrap_or(self.common.void);
        Expr {
            ty,
            kind: ExprKind::Match { scrutinee: Box::new(scrutinee), arms: checked },
            span,
        }
    }

    /// `[ENM-2]` — every value the scrutinee can take must be covered, and an
    /// arm that can never run is `W2091`. A guarded arm covers nothing, since
    /// the guard may fail at run time.
    fn report_match_coverage(&mut self, arms: &[hir::MatchArm], ty: Ty, span: Span) {
        let mut seen: Vec<&hir::Pattern> = Vec::new();
        for arm in arms {
            if !usefulness::is_useful(self.types, &seen, &arm.pattern, ty) {
                self.sink.emit(
                    Diagnostic::warning(codes::W2091, arm.span, "this arm can never match")
                        .primary_label("the arms above already cover every value this matches"),
                );
            }
            if arm.guard.is_none() {
                seen.push(&arm.pattern);
            }
        }
        let missing = usefulness::missing_patterns(self.types, &seen, ty);
        if !missing.is_empty() {
            let shown = self.types.display(ty);
            let list = missing.join("`, `");
            self.sink.emit(
                Diagnostic::error(
                    codes::E2090,
                    span,
                    format!("`match` on `{shown}` does not cover every value"),
                )
                .primary_label(format!("`{list}` not covered"))
                .note("add the missing arms, or `_` as a catch-all [ENM-2]"),
            );
        }
    }

    fn check_pattern(&mut self, pattern: &ast::Pattern, expected: Ty) -> hir::Pattern {
        let span = pattern.span;
        let wild = |ty| hir::Pattern { ty, kind: hir::PatternKind::Wild, span };
        match &pattern.kind {
            ast::PatternKind::Wild => wild(expected),

            ast::PatternKind::Lit(lit) => self.check_literal_pattern(lit, expected, span),

            // `[GRM-12]` — an identifier is a unit variant if one is in scope
            // for this type, and a fresh binding otherwise.
            ast::PatternKind::Bind { name, sub, .. } => {
                if sub.is_none() {
                    if let Some((id, index)) = self.variant_named(&[*name], expected) {
                        return self.variant_pattern(id, index, &[], false, expected, span);
                    }
                }
                // Inside a later `|` alternative, a name already bound by the
                // first one is that same local, not a new one.
                let local = match self.or_bindings.as_ref().and_then(|b| b.get(&name.name)) {
                    Some(&existing) => existing,
                    None => self.declare(Some(name.name), expected, span),
                };
                let sub = sub.as_ref().map(|s| Box::new(self.check_pattern(s, expected)));
                hir::Pattern { ty: expected, kind: hir::PatternKind::Bind { local, sub }, span }
            }

            ast::PatternKind::Path { segments } => {
                match self.variant_named(segments, expected) {
                    Some((id, index)) => {
                        self.variant_pattern(id, index, &[], false, expected, span)
                    }
                    None => {
                        let shown = self.types.display(expected);
                        let path: Vec<String> =
                            segments.iter().map(|s| s.name.to_string()).collect();
                        self.error(
                            codes::E1010,
                            span,
                            format!("`{}` is not a variant of `{shown}`", path.join(".")),
                        );
                        wild(self.common.error)
                    }
                }
            }

            ast::PatternKind::Constructor { path, fields, has_rest } => {
                if let Some((id, index)) = self.variant_named(path, expected) {
                    return self.variant_pattern(id, index, fields, *has_rest, expected, span);
                }
                // `Point(x=1, y=2)` — a struct pattern.
                if let TyKind::Struct(struct_id) = *self.types.kind(expected) {
                    let named = self.types.struct_def(struct_id).name;
                    if path.len() == 1 && path[0].name == named {
                        let shape: Vec<(Symbol, Ty)> = self
                            .types
                            .struct_def(struct_id)
                            .fields
                            .iter()
                            .map(|f| (f.name, f.ty))
                            .collect();
                        let items = self.check_field_patterns(&shape, fields, *has_rest, span);
                        return hir::Pattern {
                            ty: expected,
                            kind: hir::PatternKind::Fields(items),
                            span,
                        };
                    }
                }
                let shown = self.types.display(expected);
                let path: Vec<String> = path.iter().map(|s| s.name.to_string()).collect();
                self.error(
                    codes::E1010,
                    span,
                    format!("`{}` does not name a constructor of `{shown}`", path.join(".")),
                );
                wild(self.common.error)
            }

            ast::PatternKind::Tuple(items) => {
                let element_types: Vec<Ty> = match self.types.kind(expected) {
                    TyKind::Tuple(tys) => tys.clone(),
                    _ => {
                        let shown = self.types.display(expected);
                        self.error(
                            codes::E2020,
                            span,
                            format!("a tuple pattern cannot match `{shown}`"),
                        );
                        return wild(self.common.error);
                    }
                };
                if element_types.len() != items.len() {
                    let shown = self.types.display(expected);
                    self.error(
                        codes::E2020,
                        span,
                        format!(
                            "`{shown}` has {} elements, but this pattern has {}",
                            element_types.len(),
                            items.len()
                        ),
                    );
                    return wild(self.common.error);
                }
                let items = items
                    .iter()
                    .zip(element_types)
                    .map(|(p, ty)| self.check_pattern(p, ty))
                    .collect();
                hir::Pattern { ty: expected, kind: hir::PatternKind::Fields(items), span }
            }

            // `[GRM-12]` — every alternative must bind the same names, so that
            // the arm body sees one set of bindings whichever one matched.
            ast::PatternKind::Or(items) => {
                let mut alternatives = Vec::new();
                let mut first: Option<Vec<Symbol>> = None;
                let outer = self.or_bindings.take();
                for item in items {
                    let before = self.locals.len();
                    let checked = self.check_pattern(item, expected);
                    let mut names: Vec<Symbol> = self.locals[before..]
                        .iter()
                        .filter_map(|l| l.name)
                        .collect();
                    // Bindings reused from the first alternative are not new
                    // locals, so gather the names from the scope instead.
                    if let Some(reuse) = &self.or_bindings {
                        names = reuse.keys().copied().collect();
                    }
                    names.sort_by_key(|n| n.to_string());
                    match &first {
                        None => {
                            // Every later alternative binds the same names to
                            // the same locals, so the body reads one set
                            // whichever alternative matched (`[GRM-12]`).
                            let bound: HashMap<Symbol, LocalId> = self.locals[before..]
                                .iter()
                                .enumerate()
                                .filter_map(|(i, l)| {
                                    l.name.map(|n| (n, LocalId((before + i) as u32)))
                                })
                                .collect();
                            first = Some(names);
                            self.or_bindings = Some(bound);
                        }
                        Some(expected_names) if *expected_names != names => self.error(
                            codes::E1010,
                            item.span,
                            "every alternative of an `|` pattern must bind the same names",
                        ),
                        Some(_) => {}
                    }
                    alternatives.push(checked);
                }
                self.or_bindings = outer;
                hir::Pattern { ty: expected, kind: hir::PatternKind::Or(alternatives), span }
            }

            ast::PatternKind::Error => wild(self.common.error),

            _ => {
                self.error(
                    codes::E1010,
                    span,
                    "this pattern is not supported yet in this phase of the compiler",
                );
                wild(self.common.error)
            }
        }
    }

    fn check_literal_pattern(
        &mut self,
        lit: &ast::Literal,
        expected: Ty,
        span: Span,
    ) -> hir::Pattern {
        let value = match lit {
            ast::Literal::Int { value, .. } if self.types.is_integral(expected) => {
                i128::try_from(*value).ok()
            }
            ast::Literal::Bool(v) if expected == self.common.bool_ => Some(i128::from(*v)),
            ast::Literal::Char(c) if expected == self.common.char_ => Some(*c as i128),
            _ => None,
        };
        match value {
            Some(value) => {
                hir::Pattern { ty: expected, kind: hir::PatternKind::Int(value), span }
            }
            None => {
                let shown = self.types.display(expected);
                self.error(
                    codes::E2020,
                    span,
                    format!("this literal cannot match a value of type `{shown}`"),
                );
                hir::Pattern { ty: self.common.error, kind: hir::PatternKind::Wild, span }
            }
        }
    }

    /// Resolve `Shape.Circle`, or bare `Circle` when the scrutinee's type says
    /// which enum is meant (`[ENM-1]`).
    fn variant_named(&self, path: &[ast::Ident], expected: Ty) -> Option<(EnumId, usize)> {
        let TyKind::Enum(id) = *self.types.kind(expected) else { return None };
        let def = self.types.enum_def(id);
        match path {
            [variant] => def.variant(variant.name).map(|(i, _)| (id, i)),
            [enum_name, variant] if def.name == enum_name.name => {
                def.variant(variant.name).map(|(i, _)| (id, i))
            }
            _ => None,
        }
    }

    fn variant_pattern(
        &mut self,
        id: EnumId,
        index: usize,
        fields: &[ast::FieldPattern],
        has_rest: bool,
        expected: Ty,
        span: Span,
    ) -> hir::Pattern {
        let shape: Vec<(Symbol, Ty)> = self.types.enum_def(id).variants[index]
            .fields
            .iter()
            .map(|f| (f.name, f.ty))
            .collect();
        let items = self.check_field_patterns(&shape, fields, has_rest, span);
        hir::Pattern {
            ty: expected,
            kind: hir::PatternKind::Variant { enum_id: id, variant: index, fields: items },
            span,
        }
    }

    /// Sub-patterns for a variant's payload or a struct's fields, positional
    /// or by name, filled with `_` wherever `..` or a shortfall leaves a gap.
    fn check_field_patterns(
        &mut self,
        shape: &[(Symbol, Ty)],
        fields: &[ast::FieldPattern],
        has_rest: bool,
        span: Span,
    ) -> Vec<hir::Pattern> {
        if !has_rest && !fields.is_empty() && fields.len() != shape.len() {
            self.error(
                codes::E2020,
                span,
                format!(
                    "this pattern has {} values, but there are {}",
                    fields.len(),
                    shape.len()
                ),
            );
        }
        let mut slots: Vec<Option<hir::Pattern>> = (0..shape.len()).map(|_| None).collect();
        for (position, field) in fields.iter().enumerate() {
            let slot = match field.name {
                Some(label) => match shape.iter().position(|(n, _)| *n == label.name) {
                    Some(slot) => slot,
                    None => {
                        self.error(
                            codes::E1010,
                            label.span,
                            format!("there is no field named `{}` here", label.name),
                        );
                        continue;
                    }
                },
                None => position,
            };
            if slot >= shape.len() {
                continue;
            }
            let checked = self.check_pattern(&field.pattern, shape[slot].1);
            if slots[slot].replace(checked).is_some() {
                self.error(
                    codes::E1030,
                    field.span,
                    format!("`{}` is matched twice", shape[slot].0),
                );
            }
        }
        slots
            .into_iter()
            .zip(shape)
            .map(|(slot, (_, ty))| {
                slot.unwrap_or(hir::Pattern { ty: *ty, kind: hir::PatternKind::Wild, span })
            })
            .collect()
    }

    /// How many loops out a `break` or `continue` targets: 0 is the innermost.
    /// A label names one of the loops currently open.
    fn loop_depth(
        &mut self,
        label: Option<ast::Ident>,
        what: &str,
        span: Span,
    ) -> Option<usize> {
        if self.loop_labels.is_empty() {
            self.error(codes::E1010, span, format!("`{what}` outside a loop"));
            return None;
        }
        let Some(label) = label else { return Some(0) };
        match self.loop_labels.iter().rev().position(|l| *l == Some(label.name)) {
            Some(depth) => Some(depth),
            None => {
                self.error(
                    codes::E1010,
                    label.span,
                    format!("no loop named `{}` is open here", label.name),
                );
                None
            }
        }
    }

    /// `[CTL-3]` — `for i in a..b` over integers becomes a counted loop, with
    /// no iterator object anywhere. Every other iterable needs `Iterator`,
    /// which arrives with interfaces.
    fn check_for(
        &mut self,
        label: Option<ast::Ident>,
        pattern: &ast::Pattern,
        iter: &ast::Expr,
        body: &ast::Block,
        else_block: &Option<ast::Block>,
        span: Span,
    ) -> Option<Stmt> {
        let incoming_class_init = self.class_init.clone();
        let ast::ExprKind::Range { lo: Some(lo), hi: Some(hi), inclusive } = &iter.kind else {
            // `[CTL-1]` — anything else is driven through `next()`.
            return self.check_for_iterator(label, pattern, iter, body, else_block, span);
        };

        // Either end may be an untyped literal, and it is the other end that
        // says what it should be: `for i in 0..xs.len()` counts in `usize`.
        let mut start = self.synth(lo);
        let mut end = self.synth(hi);
        match (
            self.types.is_untyped_literal(start.ty),
            self.types.is_untyped_literal(end.ty),
        ) {
            (true, false) => {
                let target = end.ty;
                start = self.coerce(start, target);
            }
            (false, true) => {
                let target = start.ty;
                end = self.coerce(end, target);
            }
            (true, true) => {
                start = self.commit(start);
                let target = start.ty;
                end = self.coerce(end, target);
            }
            (false, false) => {
                let target = start.ty;
                end = self.coerce(end, target);
            }
        }
        if !self.types.is_integral(start.ty) && start.ty != self.common.error {
            let shown = self.types.display(start.ty);
            self.error(codes::E2020, iter.span, format!("cannot count over `{shown}`"));
            return None;
        }

        // The loop variable is the counter, so it is in scope for the body
        // and nowhere else.
        self.scopes.push(HashMap::new());
        let local = self.declare(binding_name(pattern), start.ty, pattern.span);
        self.loop_labels.push(label.map(|l| l.name));
        let body = self.check_block(body);
        self.loop_labels.pop();
        self.scopes.pop();

        let body_class_init = self.class_init.clone();
        if else_block.is_some() && incoming_class_init.is_some() {
            self.class_init = Self::merge_class_init_paths(
                incoming_class_init.clone(),
                body_class_init,
            );
        }
        let else_block = else_block.as_ref().map(|b| self.check_block(b));
        let _ = span;
        Some(Stmt::ForRange {
            local,
            start,
            end,
            inclusive: *inclusive,
            body,
            else_block,
        })
    }

    /// `[CTL-1]` — `for x in xs` over an `Array[T]` borrows the place for the
    /// whole loop and yields `ref T`:
    ///
    /// ```text
    /// __xs: Span[T] = xs
    /// for __i in 0..__xs.len():
    ///     x = __xs[__i]
    ///     <body>
    /// ```
    ///
    /// The hidden span makes the borrow and yielded-reference types explicit
    /// while retaining the counted-loop lowering `[CTL-3]` asks for. Moving
    /// the Array into `__xs` would violate `[CTL-1]` and turn an in-loop
    /// mutation into E3050 instead of E3020 (D-070).
    fn check_for_array(
        &mut self,
        label: Option<ast::Ident>,
        pattern: &ast::Pattern,
        source: (Expr, Ty),

        body: &ast::Block,
        else_block: &Option<ast::Block>,
        span: Span,
    ) -> Option<Stmt> {
        let incoming_class_init = self.class_init.clone();
        let (iterable, elem) = source;
        let usize_ty = self.common.usize;
        let span_ty = self.types.intern(TyKind::Span { elem, mutable: false });
        let iterable_span = self.view_of(
            iterable,
            span_ty,
            false,
            Builtin::SpanFrom { mutable: false },
        );
        self.scopes.push(HashMap::new());
        let xs_local = self.declare(Some(Symbol::intern("__xs")), span_ty, span);
        self.locals[xs_local.0 as usize].for_iterator = true;
        let index_local = self.declare(Some(Symbol::intern("__i")), usize_ty, span);

        let xs = |ty: Ty| Expr { ty, kind: ExprKind::Local(xs_local), span };
        let length = Expr {
            ty: usize_ty,
            kind: ExprKind::Builtin {
                which: Builtin::SpanLen,
                args: vec![xs(span_ty)],
            },
            span,
        };

        // The loop variable is a shared reference to the element at the
        // current index, as `[CTL-1]` requires for a borrowed place.
        self.scopes.push(HashMap::new());
        let item_ty = self.types.intern(TyKind::Ref { mutable: false, inner: elem });
        let item_local = self.declare(binding_name(pattern), item_ty, pattern.span);
        self.loop_labels.push(label.map(|l| l.name));
        let mut inner = vec![Stmt::Let {
            local: item_local,
            init: Some(Expr {
                ty: item_ty,
                kind: ExprKind::Ref {
                    place: Box::new(Expr {
                        ty: elem,
                        kind: ExprKind::Index {
                            base: Box::new(xs(span_ty)),
                            index: Box::new(Expr {
                                ty: usize_ty,
                                kind: ExprKind::Local(index_local),
                                span,
                            }),
                        },
                        span,
                    }),
                    mutable: false,
                },
                span,
            }),
        }];
        let checked = self.check_block(body);
        self.loop_labels.pop();
        self.scopes.pop();
        inner.extend(checked.stmts);

        let body_class_init = self.class_init.clone();
        if else_block.is_some() && incoming_class_init.is_some() {
            self.class_init = Self::merge_class_init_paths(
                incoming_class_init,
                body_class_init,
            );
        }
        let else_block = else_block.as_ref().map(|b| self.check_block(b));
        self.scopes.pop();

        Some(Stmt::Block(Block {
            stmts: vec![
                Stmt::Let { local: xs_local, init: Some(iterable_span) },
                Stmt::ForRange {
                    local: index_local,
                    start: Expr { ty: usize_ty, kind: ExprKind::Int(0), span },
                    end: length,
                    inclusive: false,
                    body: Block { stmts: inner, span },
                    else_block,
                },
            ],
            span,
        }))
    }

    /// `[CTL-1]` — `for x in it` over anything that provides `next()`:
    ///
    /// ```text
    /// __it = it
    /// __done = false
    /// while not __done:
    ///     match __it.next():
    ///         Some(x): <body>
    ///         None:    __done = true
    /// else:
    ///     <else>
    /// ```
    ///
    /// Exhaustion ends the loop through the condition rather than through a
    /// `break`, so `[CTL-4]`'s `else` still tells the two apart.
    fn check_for_iterator(
        &mut self,
        label: Option<ast::Ident>,
        pattern: &ast::Pattern,
        iter: &ast::Expr,
        body: &ast::Block,
        else_block: &Option<ast::Block>,
        span: Span,
    ) -> Option<Stmt> {
        let incoming_class_init = self.class_init.clone();
        let iterable = self.synth_committed(iter);
        if iterable.ty == self.common.error {
            return None;
        }
        // `[CTL-3]`'s spirit for a collection: iterating an `Array[T]` is a
        // counted loop over its indices, with no iterator object at all.
        if let TyKind::Vec { elem } = *self.types.kind(iterable.ty) {
            return self.check_for_array(label, pattern, (iterable, elem), body, else_block, span);
        }

        // The iterator itself is a local, because `next` mutates it.
        self.scopes.push(HashMap::new());
        let it_local = self.declare(Some(Symbol::intern("__it")), iterable.ty, iter.span);
        self.locals[it_local.0 as usize].for_iterator = true;
        let bool_ty = self.common.bool_;
        let done_local = self.declare(Some(Symbol::intern("__done")), bool_ty, iter.span);

        // `__it.next()`, checked through ordinary method resolution so that a
        // type without one is reported the same way any missing method is.
        let receiver = Expr { ty: iterable.ty, kind: ExprKind::Local(it_local), span: iter.span };
        let (item_option, call) = if let Some((elem, kind)) = self.span_iterator(iterable.ty) {
            let item = match kind {
                SpanIteratorKind::Elements { mutable } => {
                    self.types.intern(TyKind::Ref { mutable, inner: elem })
                }
                SpanIteratorKind::Chunks { mutable } => {
                    self.types.intern(TyKind::Span { elem, mutable })
                }
            };
            let item_option = self.option_of(item);
            let which = match kind {
                SpanIteratorKind::Elements { mutable } => {
                    Builtin::SpanIterNext { elem, mutable }
                }
                SpanIteratorKind::Chunks { mutable } => {
                    Builtin::SpanChunksNext { elem, mutable }
                }
            };
            (
                item_option,
                Expr {
                    ty: item_option,
                    kind: ExprKind::Builtin { which, args: vec![receiver] },
                    span: iter.span,
                },
            )
        } else if let Some((elem, mutable)) = self.arena_array_iterator(iterable.ty) {
            let item = self.types.intern(TyKind::Ref { mutable, inner: elem });
            let item_option = self.option_of(item);
            let receiver = self.pass_receiver(receiver, Mode::Mut, iter.span);
            (
                item_option,
                Expr {
                    ty: item_option,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaArrayIterNext { elem, mutable },
                        args: vec![receiver],
                    },
                    span: iter.span,
                },
            )
        } else if let Some((key, value)) = self.arena_map_iterator(iterable.ty) {
            let key_ref = self.types.intern(TyKind::Ref { mutable: false, inner: key });
            let value_ref = self.types.intern(TyKind::Ref { mutable: false, inner: value });
            let item = self.types.intern(TyKind::Tuple(vec![key_ref, value_ref]));
            let item_option = self.option_of(item);
            let receiver = self.pass_receiver(receiver, Mode::Mut, iter.span);
            (
                item_option,
                Expr {
                    ty: item_option,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaMapIterNext { key, value },
                        args: vec![receiver],
                    },
                    span: iter.span,
                },
            )
        } else {
            let Some(entry) = self.methods.get(&(iterable.ty, Symbol::intern("next"))) else {
                let shown = self.types.display(iterable.ty);
                self.scopes.pop();
                self.error(
                    codes::E2040,
                    iter.span,
                    format!("`{shown}` cannot be iterated: it has no `next` method"),
                );
                return None;
            };
            let def = entry.def;
            let receiver_mode = entry.receiver;
            let raw_ret = self.signatures[def.0 as usize].ret;
            let item_option = self.resolve_assoc(raw_ret, iterable.ty);
            (
                item_option,
                Expr {
                    ty: item_option,
                    kind: ExprKind::Call {
                        callee: def,
                        arg_eval_order: None,
                        args: vec![self.pass_receiver(receiver, receiver_mode, iter.span)],
                        latebound: false,
                    },
                    span: iter.span,
                },
            )
        };

        let TyKind::Enum(option_id) = *self.types.kind(item_option) else {
            let shown = self.types.display(item_option);
            self.scopes.pop();
            self.error(
                codes::E2020,
                iter.span,
                format!("`next` must return an `Option`, not `{shown}`"),
            );
            return None;
        };
        let item_ty = self.types.enum_def(option_id).variants[1].fields[0].ty;

        // `Some(x): <body>` — the loop variable is the payload.
        self.scopes.push(HashMap::new());
        let bound = self.declare(binding_name(pattern), item_ty, pattern.span);
        self.loop_labels.push(label.map(|l| l.name));
        let body = self.check_block(body);
        self.loop_labels.pop();
        self.scopes.pop();

        let some_arm = hir::MatchArm {
            pattern: hir::Pattern {
                ty: item_option,
                kind: hir::PatternKind::Variant {
                    enum_id: option_id,
                    variant: 1,
                    fields: vec![hir::Pattern {
                        ty: item_ty,
                        kind: hir::PatternKind::Bind { local: bound, sub: None },
                        span: pattern.span,
                    }],
                },
                span: pattern.span,
            },
            guard: None,
            body: hir::MatchArmBody::Block(body),
            span,
        };
        let none_arm = hir::MatchArm {
            pattern: hir::Pattern {
                ty: item_option,
                kind: hir::PatternKind::Variant {
                    enum_id: option_id,
                    variant: 0,
                    fields: Vec::new(),
                },
                span,
            },
            guard: None,
            body: hir::MatchArmBody::Block(Block {
                stmts: vec![Stmt::Assign {
                    place: Expr { ty: bool_ty, kind: ExprKind::Local(done_local), span },
                    value: Expr { ty: bool_ty, kind: ExprKind::Bool(true), span },
                }],
                span,
            }),
            span,
        };

        let body_class_init = self.class_init.clone();
        if else_block.is_some() && incoming_class_init.is_some() {
            self.class_init = Self::merge_class_init_paths(
                incoming_class_init,
                body_class_init,
            );
        }
        let else_block = else_block.as_ref().map(|b| self.check_block(b));
        self.scopes.pop();

        let loop_body = Block {
            stmts: vec![Stmt::Expr(Expr {
                ty: self.common.void,
                kind: ExprKind::Match {
                    scrutinee: Box::new(call),
                    arms: vec![some_arm, none_arm],
                },
                span,
            })],
            span,
        };
        let condition = Expr {
            ty: bool_ty,
            kind: ExprKind::Unary {
                op: UnOp::Not,
                operand: Box::new(Expr {
                    ty: bool_ty,
                    kind: ExprKind::Local(done_local),
                    span,
                }),
            },
            span,
        };

        Some(Stmt::Block(Block {
            stmts: vec![
                Stmt::Let { local: it_local, init: Some(iterable) },
                Stmt::Let {
                    local: done_local,
                    init: Some(Expr { ty: bool_ty, kind: ExprKind::Bool(false), span }),
                },
                Stmt::While { cond: condition, body: loop_body, else_block },
            ],
            span,
        }))
    }

    fn check_if(&mut self, if_stmt: &ast::IfStmt) -> Stmt {
        match &if_stmt.cond {
            ast::Condition::Expr(expr) => self.check_class_init_whole_self_use(expr),
            ast::Condition::Pattern { value, .. } => self.check_class_init_whole_self_use(value),
        }
        let cond = self.check_condition(&if_stmt.cond);
        let incoming_class_init = self.class_init.clone();
        let then_block = self.check_block(&if_stmt.then_block);
        let then_class_init = self.class_init.clone();
        self.class_init = incoming_class_init.clone();
        let else_block = match if_stmt.else_block.as_deref() {
            Some(ast::ElseBranch::Block(b)) => Some(self.check_block(b)),
            Some(ast::ElseBranch::If(nested)) => {
                let stmt = self.check_if(nested);
                Some(Block { stmts: vec![stmt], span: if_stmt.then_block.span })
            }
            None => None,
        };
        let else_class_init = self.class_init.clone();
        self.class_init = Self::merge_class_init_paths(then_class_init, else_class_init);
        Stmt::If { cond, then_block, else_block }
    }

    /// `[CTL-0]` — the fix-it for a non-`bool` condition, chosen by type. The
    /// rule names four shapes; anything else gets the bare error, because a
    /// wrong suggestion costs more than none.
    fn truthiness_fix(&self, ty: Ty) -> Option<String> {
        match self.types.kind(ty) {
            TyKind::Vec { .. } | TyKind::Str | TyKind::Array { .. } => {
                Some("a container is not a condition: write `not xs.is_empty()`".to_string())
            }
            TyKind::Int(_) | TyKind::Uint(_) => {
                Some("a number is not a condition: write `x != 0`".to_string())
            }
            TyKind::Ptr { .. } => {
                Some("a pointer is not a condition: write `not p.is_null()`".to_string())
            }
            TyKind::Enum(id) => {
                let name = self.types.enum_def(*id).name;
                let name = name.as_str();
                if name.starts_with("Option") || name.ends_with(".Option") {
                    Some("write `x.is_some()`".to_string())
                } else if name.starts_with("Result") || name.ends_with(".Result") {
                    Some("write `x.is_ok()`".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn check_condition(&mut self, cond: &ast::Condition) -> Expr {
        match cond {
            ast::Condition::Expr(expr) => {
                let bool_ty = self.common.bool_;
                let checked = self.synth_committed(expr);
                // `[CTL-0]` — there is no truthiness conversion, and the fix
                // depends on what the writer actually wrote, so the diagnostic
                // names the call that turns this value into a `bool`.
                if checked.ty != bool_ty
                    && checked.ty != self.common.error
                    && checked.ty != self.common.never
                {
                    let shown = self.types.display(checked.ty);
                    let fix = self.truthiness_fix(checked.ty);
                    let mut diag = Diagnostic::error(
                        codes::E2035,
                        expr.span,
                        "condition must be `bool`",
                    )
                    .primary_label(format!("this is `{shown}`"));
                    if let Some(fix) = fix {
                        diag = diag.help(fix);
                    }
                    self.sink.emit(diag);
                    return Expr { ty: bool_ty, kind: ExprKind::Error, span: expr.span };
                }
                checked
            }
            ast::Condition::Pattern { value, .. } => {
                self.error(
                    codes::E1010,
                    value.span,
                    "a pattern condition is not supported yet in this phase",
                );
                Expr { ty: self.common.error, kind: ExprKind::Error, span: value.span }
            }
        }
    }

    // -- expressions ---------------------------------------------------------

    /// Synthesis mode, then default any untyped literal that survived
    /// (`[LEX-16]`, `[LEX-17]`): `i32` for integers, `f32` for floats.
    fn synth_committed(&mut self, ast_expr: &ast::Expr) -> Expr {
        let expr = self.synth(ast_expr);
        let committed = self.commit(expr);
        self.warn_if_literal_loses_precision(ast_expr, committed.ty);
        committed
    }

    /// `[LEX-17a]` — an unsuffixed float literal that takes `f32` from the
    /// *default* rather than from context, and that was written with more
    /// precision than `f32` holds, is `W2015`. A literal that receives `f32`
    /// from context is not diagnosed: the programmer chose the type.
    fn warn_if_literal_loses_precision(&mut self, expr: &ast::Expr, ty: Ty) {
        if ty != self.common.f32 {
            return;
        }
        let ast::ExprKind::Lit(ast::Literal::Float { value, suffix: None, digits }) = &expr.kind
        else {
            return;
        };
        // Nine is the most a decimal string can carry into `f32` without a
        // round trip changing it.
        if *digits <= 9 {
            return;
        }
        let as_f32 = *value as f32;
        self.sink.emit(
            Diagnostic::warning(
                codes::W2015,
                expr.span,
                "float literal loses precision at `f32`",
            )
            .primary_label(format!("`{value}` becomes `{as_f32}`"))
            .help(format!("write `{value}f64` to keep it, or annotate the binding `: f32` to accept it")),
        );
    }

    fn commit(&mut self, expr: Expr) -> Expr {
        if !self.types.is_untyped_literal(expr.ty) {
            return expr;
        }
        let default = if self.types.is_float(expr.ty) { self.common.f32 } else { self.common.i32 };
        Expr { ty: default, ..expr }
    }

    /// Checking mode: an expected type flows downward (`[TYP-23]` rule 3).
    fn check_expr(&mut self, expr: &ast::Expr, expected: Ty) -> Expr {
        let found = self.synth_with_expectation(expr, Some(expected));
        self.coerce(found, expected)
    }

    /// `[TYP-5]` — coercion sites allow lossless widening; a literal adopts
    /// the expected type; everything else is `E2020`.
    fn coerce(&mut self, expr: Expr, expected: Ty) -> Expr {
        if expr.ty == expected || expected == self.common.error || expr.ty == self.common.error {
            return expr;
        }
        // `[CLS-4]`/`[DSP-1]` — single-inheritance handles upcast to an
        // open/abstract base without changing the pointed-to object. Keep
        // this as a distinct compiler-inserted cast so C receives a pointer
        // of the base type and later ownership/dispatch passes can audit it.
        if let (TyKind::Class(derived), TyKind::Class(base)) =
            (self.types.kind(expr.ty), self.types.kind(expected))
            && self.types.class_is_subclass_of(*derived, *base)
        {
            let span = expr.span;
            return Expr {
                ty: expected,
                kind: ExprKind::Cast { expr: Box::new(expr), to: expected },
                span,
            };
        }
        // `[FN-6b]` — the boundary owns the late-bound fact. An ordinary
        // function value may therefore be supplied to an expected
        // `@latebound fn(...)` type without changing the function value's
        // ordinary identity or declaration semantics. The resulting HIR type
        // is the expected boundary so calls through a local retain it.
        if let (
            TyKind::Fn { latebound: false, params: found_params, ret: found_ret },
            TyKind::Fn { latebound: true, params: expected_params, ret: expected_ret },
        ) = (self.types.kind(expr.ty), self.types.kind(expected))
            && found_params.len() == expected_params.len()
            && found_params
                .iter()
                .zip(expected_params)
                .all(|(found, wanted)| found.mode == wanted.mode && found.ty == wanted.ty)
            && found_ret == expected_ret
        {
            return Expr { ty: expected, ..expr };
        }
        // `!` coerces to every type (`[TYP-4]` table).
        if expr.ty == self.common.never {
            return Expr { ty: expected, ..expr };
        }
        // `[TYP-14]` — a reference used where its referent is wanted reads
        // through, whatever produced it. Handling this only for a local read
        // left `v: i32 = f(r)` rejected, where `f` returns `ref i32`.
        if let TyKind::Ref { inner, .. } = *self.types.kind(expr.ty) {
            if inner == expected {
                let span = expr.span;
                return Expr { ty: inner, kind: ExprKind::Deref(Box::new(expr)), span };
            }
        }
        // `[CELL-7]` — a guard used where its contents are wanted reads
        // through (e.g. `x: i32 = g` where `g: Ref[i32]`). The unwrapped place
        // keeps the guard alive.
        if let Some((inner, _)) = self.ref_guard_inner(expr.ty) {
            if inner == expected {
                return self.read_guard_through(expr);
            }
        }
        // `[RNG-2]` — "two range types are distinct types even when
        // representation and range are identical". This is checked before
        // erasure, so the message names the two types rather than the one
        // representation they share, which is the confusion the feature exists
        // to catch.
        if let (TyKind::Range(found), TyKind::Range(wanted)) =
            (self.types.kind(expr.ty), self.types.kind(expected))
        {
            let (found, wanted) = (*found, *wanted);
            if found != wanted {
                let f = self.types.range_def(found);
                let w = self.types.range_def(wanted);
                let (fname, wname) = (f.name.to_string(), w.name.to_string());
                let same_repr = f.repr == w.repr;
                let span = expr.span;
                let mut d = Diagnostic::error(
                    codes::E2210,
                    span,
                    format!("`{fname}` is not `{wname}`"),
                )
                .primary_label(format!("this is `{fname}`"));
                if same_repr {
                    let repr = self.types.display(w.repr);
                    d = d.note(format!(
                        "both are `{repr}` underneath, and telling them apart is what a \
                         range type is for [RNG-2]"
                    ));
                }
                self.sink.emit(d);
                return Expr { ty: expected, kind: ExprKind::Error, span };
            }
        }
        // `[TYP-5]` **range erasure**: a value of a range type over `R`
        // coerces to `R`, and the step composes with widening — so
        // `Roughness → f32 → f64` and `Percent → u8 → u32` are coercions.
        // It is admitted at coercion sites only; `[RNG-2]` says a range type
        // never converts implicitly in operator position, which is
        // `[RNG-5a1]`'s generated impls instead.
        if let TyKind::Range(id) = self.types.kind(expr.ty) {
            let id = *id;
            let repr = self.types.range_def(id).repr;
            if repr == expected || self.types.widens_to(repr, expected) {
                let span = expr.span;
                let erased =
                    Expr { ty: repr, kind: ExprKind::EraseRange(Box::new(expr)), span };
                return if repr == expected { erased } else { self.coerce(erased, expected) };
            }
        }
        // `[SPN-1]` — "`Array[T]` coerces to `Span[T]` at borrow sites and to
        // `MutSpan[T]` at `mut` sites; `[T; N]` likewise". A view, not a
        // conversion: it points into the container, and the borrow checker
        // keeps the container borrowed for the view's region.
        if let TyKind::Span { elem: want, mutable } = *self.types.kind(expected) {
            let source = match *self.types.kind(expr.ty) {
                TyKind::Vec { elem } | TyKind::Array { elem, .. } => Some(elem),
                _ => None,
            };
            if source == Some(want) {
                return self.view_of(expr, expected, mutable, Builtin::SpanFrom { mutable });
            }
        }
        // `[SPN-1]` — `String` coerces to `str`. This must use the same
        // producer as the explicit `as_str()` spelling: the result points into
        // the string's buffer, so representing it as an ordinary conversion
        // would hide the source loan from region inference and borrow checking
        // (D-037). `String` is the compiler-known `Vec[u8]` representation.
        if expected == self.common.str_
            && matches!(
                *self.types.kind(expr.ty),
                TyKind::Vec { elem } if elem == self.common.u8
            )
        {
            return self.view_of(expr, expected, false, Builtin::StringAsStr);
        }
        // `[RNG-3]` — construction. A constant the compiler can place in range
        // needs no check; one it can place outside is `E2211`; anything else
        // is outside `[RNG-10]`'s closed set and is `E2215`.
        if let TyKind::Range(id) = self.types.kind(expected) {
            let id = *id;
            return self.construct_range(expr, id);
        }
        if self.types.is_untyped_literal(expr.ty) && self.literal_fits(&expr, expected) {
            return self.adopt_literal(expr, expected);
        }
        if self.types.widens_to(expr.ty, expected) {
            let span = expr.span;
            return Expr {
                ty: expected,
                kind: ExprKind::Widen { expr: Box::new(expr), to: expected },
                span,
            };
        }
        let found = self.types.display(expr.ty);
        let wanted = self.types.display(expected);
        let span = expr.span;
        self.sink.emit(
            Diagnostic::error(codes::E2020, span, format!("expected `{wanted}`, found `{found}`"))
                .primary_label(format!("this is `{found}`"))
                .note("Ember does not convert between numeric types implicitly [TYP-4]"),
        );
        Expr { ty: expected, kind: ExprKind::Error, span }
    }

    /// `[RNG-3]`/`[RNG-10]` — a range-typed value arises in Safe code only
    /// from a constant the compiler placed in range, `T.checked`, `T.clamped`,
    /// a value whose known range is contained in the target's, or a copy. This
    /// handles the first; the middle two are ordinary calls, the fourth waits
    /// for `[RNG-4]`'s range tracking, and the last is `Copy`.
    fn construct_range(&mut self, expr: Expr, id: ember_types::RangeId) -> Expr {
        let span = expr.span;
        let def = self.types.range_def(id).clone();
        let name = def.name.to_string();

        // The value must first be one of the representation, which for an
        // untyped literal it becomes by `[LEX-16]`/`[LEX-17]`.
        let value = if self.types.is_untyped_literal(expr.ty) {
            if !self.literal_fits(&expr, def.repr) {
                let repr = self.types.display(def.repr);
                self.error(
                    codes::E2020,
                    span,
                    format!("`{name}` holds a `{repr}`, and this literal is not one"),
                );
                return Expr { ty: self.types.intern(TyKind::Range(id)), kind: ExprKind::Error, span };
            }
            self.adopt_literal(expr, def.repr)
        } else if expr.ty == def.repr || self.types.widens_to(expr.ty, def.repr) {
            self.coerce(expr, def.repr)
        } else {
            let found = self.types.display(expr.ty);
            let repr = self.types.display(def.repr);
            self.error(
                codes::E2020,
                span,
                format!("`{name}` holds a `{repr}`, and this is a `{found}`"),
            );
            return Expr { ty: self.types.intern(TyKind::Range(id)), kind: ExprKind::Error, span };
        };

        let range_ty = self.types.intern(TyKind::Range(id));

        // `[RNG-10]`(d) — "a value whose `[RNG-4]` range is contained in the
        // target's" is one of the five ways a range value arises in Safe code,
        // and it needs no check: every value the expression can take is
        // already a value of the target. Tried before the constant path
        // because it subsumes it, and a non-constant that qualifies would
        // otherwise be `E2215`.
        if let Some((lo, hi)) = self.range_of(&value) {
            if def.contains(lo) && def.contains(hi) {
                return Expr { ty: range_ty, kind: value.kind, span };
            }
        }

        match self.constant_bound_of(&value) {
            Some(bound) if def.contains(bound) => {
                // `[RNG-3]` — "construction from a constant in range […] emits
                // no check", and `[COST-3]` classes the type itself as not
                // observable, so the value is the representation's.
                Expr { ty: range_ty, kind: value.kind, span }
            }
            Some(bound) => {
                // `E2211` — IV.2a's worked example is this diagnostic.
                let shown = show_bound(bound);
                let bounds = show_range(&def, self.types);
                self.sink.emit(
                    Diagnostic::error(
                        codes::E2211,
                        span,
                        format!("{shown} is outside `{name}`"),
                    )
                    .primary_label(format!("`{name}` holds {bounds}"))
                    .help(format!(
                        "clamp it, or take the fallible form: `{name}.checked({shown})`"
                    )),
                );
                Expr { ty: range_ty, kind: ExprKind::Error, span }
            }
            None => {
                // `[RNG-10]` — outside the closed construction set.
                self.sink.emit(
                    Diagnostic::error(
                        codes::E2215,
                        span,
                        format!("a `{name}` cannot be built from a value this is not known to be in range"),
                    )
                    .help(format!(
                        "`{name}.clamped(x)` is total; `{name}.checked(x)` returns a \
                         `Result` when the difference matters"
                    ))
                    .note(
                        "the ways to make a range value are a constant in range, `checked`, \
                         `clamped`, a value whose known range fits, and a copy [RNG-10]",
                    ),
                );
                Expr { ty: range_ty, kind: ExprKind::Error, span }
            }
        }
    }

    /// `[RNG-4]` — the interval an expression is known to lie in.
    ///
    /// "The compiler tracks a known range for every numeric expression it can
    /// — literals, `min`/`max`/`clamp`, the arms of an `if` or `match` that
    /// compared the value, and arithmetic on operands with known ranges."
    /// This covers the first and the last of those; the branch arms need a
    /// flow-sensitive pass and `min`/`max`/`clamp` need `std.math`, and both
    /// are still missing (D-018).
    ///
    /// The fact that carries most of the weight is not arithmetic at all: **a
    /// value of a range type is in its declared range**, because `[RNG-9]`
    /// makes that an invariant rather than a convention and `[RNG-4]` "MAY
    /// assume" it. That is what makes `[RNG-10]`(d) — "a value whose `[RNG-4]`
    /// range is contained in the target's" — mean anything, and until this
    /// existed (d) admitted a constant and nothing else.
    ///
    /// Returns `None` where nothing is known, which is always safe: the caller
    /// then emits the check it would have emitted anyway.
    fn range_of(&self, expr: &Expr) -> Option<(Bound, Bound)> {
        // Anything *at* a range type is in that range, whatever produced it.
        if let TyKind::Range(id) = *self.types.kind(expr.ty) {
            let def = self.types.range_def(id);
            return Some((def.lo, def.hi));
        }
        match &expr.kind {
            ExprKind::Int(_) | ExprKind::Float(_) => {
                let bound = self.constant_bound_of(expr)?;
                Some((bound, bound))
            }
            ExprKind::Widen { expr, .. } => self.range_of(expr),
            ExprKind::Local(local) => self.local_ranges.get(local).copied(),
            ExprKind::Deref(inner) => self.range_of(inner),
            // The erasure `[TYP-5]` inserts. The value is the representation's
            // now, and what is known about it is the range it came from.
            ExprKind::EraseRange(inner) => {
                let TyKind::Range(id) = *self.types.kind(inner.ty) else { return None };
                let def = self.types.range_def(id);
                Some((def.lo, def.hi))
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let a = self.range_of(lhs)?;
                let b = self.range_of(rhs)?;
                let (lo, hi) = interval(*op, a, b)?;
                // `[RNG-4a]` — "A range fact MUST NOT be derived from the
                // mathematical range of an operation that can overflow, **in
                // any profile**", because `[TYP-8]`'s policy differs between
                // `debug` and `release` and `[PRF-1]` forbids the set of checks
                // from depending on that difference. So a derived interval is
                // kept only where it provably fits the representation.
                self.fits_repr(expr.ty, lo, hi).then_some((lo, hi))
            }
            _ => None,
        }
    }

    /// Whether an interval lies inside what `repr` can hold. A float
    /// representation additionally requires both ends finite: an overflow to
    /// infinity is exactly the case `[RNG-4a]` refuses to derive a fact from.
    fn fits_repr(&self, repr: Ty, lo: Bound, hi: Bound) -> bool {
        match (lo, hi) {
            (Bound::Float(a), Bound::Float(b)) => {
                self.types.is_float(repr) && a.is_finite() && b.is_finite()
            }
            (Bound::Int(a), Bound::Int(b)) => match int_max(self.types, repr) {
                Some(max) => {
                    let max = i128::try_from(max).unwrap_or(i128::MAX);
                    let signed = matches!(self.types.kind(repr), TyKind::Int(_));
                    let min = if signed { -max - 1 } else { 0 };
                    a >= min && b <= max
                }
                None => false,
            },
            _ => false,
        }
    }

    /// The constant value of an already-checked expression, where it has one.
    fn constant_bound_of(&self, expr: &Expr) -> Option<Bound> {
        match &expr.kind {
            ExprKind::Int(value) => i128::try_from(*value).ok().map(|v| {
                if self.types.is_float(expr.ty) { Bound::Float(v as f64) } else { Bound::Int(v) }
            }),
            ExprKind::Float(value) => Some(Bound::Float(*value)),
            ExprKind::Widen { expr, .. } => self.constant_bound_of(expr),
            _ => None,
        }
    }

    fn literal_fits(&self, expr: &Expr, expected: Ty) -> bool {
        if self.types.is_float(expr.ty) {
            // A float literal takes a float type only.
            return self.types.is_float(expected);
        }
        // `[LEX-16]` — an integer literal may also take a float type by
        // context, which is what makes `Vec3(1, 2, 3)` work.
        self.types.is_integral(expected) || self.types.is_float(expected)
    }

    fn adopt_literal(&mut self, expr: Expr, expected: Ty) -> Expr {
        let span = expr.span;
        match expr.kind {
            ExprKind::Int(value) => {
                if let Some(max) = int_max(self.types, expected) {
                    if value > max {
                        let shown = self.types.display(expected);
                        self.error(
                            codes::E2010,
                            span,
                            format!("the literal `{value}` does not fit in `{shown}`"),
                        );
                    }
                    Expr { ty: expected, kind: ExprKind::Int(value), span }
                } else {
                    // Integer literal in a float context.
                    Expr { ty: expected, kind: ExprKind::Float(value as f64), span }
                }
            }
            ExprKind::Float(value) => Expr { ty: expected, kind: ExprKind::Float(value), span },
            other => Expr { ty: expected, kind: other, span },
        }
    }

    fn synth(&mut self, expr: &ast::Expr) -> Expr {
        self.synth_with_expectation(expr, None)
    }

    fn synth_with_expectation(&mut self, expr: &ast::Expr, expected: Option<Ty>) -> Expr {
        let span = expr.span;
        match &expr.kind {
            ast::ExprKind::Paren(inner) => self.synth_with_expectation(inner, expected),

            // `[CLO-1]` — a closure. One that captures nothing "is a plain
            // value type", and Part IV §9's `fn(A) -> R` is exactly that, so
            // it lowers to a synthesised function and becomes a `[FN-6]`
            // value. A capturing one is a unique anonymous struct implementing
            // `Callable`; it needs an environment and is the next slice.
            ast::ExprKind::Lambda(lambda) => self.synth_lambda(lambda, expected, span),

            ast::ExprKind::Lit(lit) => self.synth_literal(lit, span),

            // `self` inside a method is the receiver parameter, which is a
            // local like any other.
            ast::ExprKind::SelfExpr => {
                let name = Symbol::intern("self");
                match self.lookup(name) {
                    Some(local) => self.read_local_expecting(local, span, expected),
                    None => {
                        self.error(codes::E1010, span, "`self` outside a method");
                        Expr { ty: self.common.error, kind: ExprKind::Error, span }
                    }
                }
            }

            ast::ExprKind::Path { segments } if segments.len() == 1 => {
                let name = segments[0].name;
                if let Some(local) = self.lookup(name) {
                    return self.read_local_expecting(local, span, expected);
                }
                // A `const` or `static` is substituted where its name appears.
                if let Some(value) = self.constants.get(&self.resolve_name(name)) {
                    let kind = match &value.kind {
                        ExprKind::Int(v) => ExprKind::Int(*v),
                        ExprKind::Float(v) => ExprKind::Float(*v),
                        ExprKind::Bool(v) => ExprKind::Bool(*v),
                        ExprKind::Str(s) => ExprKind::Str(s.clone()),
                        _ => ExprKind::Error,
                    };
                    return Expr { ty: value.ty, kind, span };
                }
                // `None` carries nothing, so only the expected type can say
                // which `Option` it is.
                if name.is("None") {
                    return match expected.filter(|e| self.is_option(*e)) {
                        Some(ty) => {
                            let TyKind::Enum(id) = *self.types.kind(ty) else { unreachable!() };
                            Expr {
                                ty,
                                kind: ExprKind::EnumLit { enum_id: id, variant: 0, fields: Vec::new() },
                                span,
                            }
                        }
                        None => {
                            self.error(
                                codes::E2060,
                                span,
                                "cannot tell which `Option` this `None` is; annotate the type",
                            );
                            Expr { ty: self.common.error, kind: ExprKind::Error, span }
                        }
                    };
                }
                // `[FN-6]` — "Functions are values of a unique zero-sized
                // function type; they coerce to `fn(A) -> R`". A named
                // function in value position is that value.
                if let Some(value) = self.function_value(name, span) {
                    return value;
                }
                // `[CLO-2]` — inside a closure, a name the enclosing function
                // declares is a capture. Saying "cannot find it" would be
                // false and would send the reader looking for a typo.
                if let Some(captured) = self.resolve_capture(name, span) {
                    return captured;
                }
                self.error(codes::E1010, span, format!("cannot find `{name}` in this scope"));
                Expr { ty: self.common.error, kind: ExprKind::Error, span }
            }

            // `Color.Red` — a unit variant named through its enum.
            ast::ExprKind::Field { base, name }
                if self.enum_named(base).is_some() =>
            {
                let id = self.enum_named(base).expect("just checked");
                self.synth_variant(id, *name, &[], span)
            }

            ast::ExprKind::Field { base, name } => {
                let base = self.synth(base);
                // `[CELL-7]` — a temporary guard (e.g. `c.borrow().x`) reads
                // through like a local one does via `read_local_expecting`.
                let mut base = match self.ref_guard_inner(base.ty) {
                    Some(_) => self.read_guard_through(base),
                    None => base,
                };
                // IX.1 — field lookup auto-dereferences Box owners. Repeat
                // for nested boxes; every generated dereference remains
                // rooted at the owner place for borrow and move analysis.
                while self.box_inner(base.ty).is_some() {
                    base = self.read_box_through(base);
                }
                match *self.types.kind(base.ty) {
                    TyKind::Struct(id) => match self.types.struct_def(id).field(name.name) {
                        Some((index, field)) => {
                            let ty = field.ty;
                            // Resolve, then check, then build. One check on one
                            // path — see `check_field_visible`.
                            self.check_field_visible(id, field.vis, field.name, name.span);
                            Expr { ty, kind: ExprKind::Field { base: Box::new(base), index }, span }
                        }
                        None => {
                            let struct_name = self.types.struct_def(id).name;
                            self.error(
                                codes::E2020,
                                name.span,
                                format!("`{struct_name}` has no field `{}`", name.name),
                            );
                            Expr { ty: self.common.error, kind: ExprKind::Error, span }
                        }
                    },
                    TyKind::Class(id) => match self.types.class_field_info(id, name.name) {
                        Some((index, owner, field)) => {
                            let ty = field.ty;
                            self.check_class_field_visible(owner, field.vis, field.name, name.span);
                            let field_expr =
                                Expr { ty, kind: ExprKind::Field { base: Box::new(base), index }, span };
                            if !self.in_assignment_target
                                && self.class_init_field_index(&field_expr).is_some()
                            {
                                self.check_class_init_field_read(index, span);
                            }
                            field_expr
                        }
                        None => {
                            let class_name = self.types.class_def(id).name;
                            self.error(
                                codes::E2020,
                                name.span,
                                format!("`{class_name}` has no field `{}`", name.name),
                            );
                            Expr { ty: self.common.error, kind: ExprKind::Error, span }
                        }
                    },
                    _ => {
                        if base.ty != self.common.error {
                            let shown = self.types.display(base.ty);
                            self.error(
                                codes::E2020,
                                span,
                                format!("`{shown}` has no field `{}`", name.name),
                            );
                        }
                        Expr { ty: self.common.error, kind: ExprKind::Error, span }
                    }
                }
            }

            // `(a, b)` — Part IV.3. An expected tuple of the same arity flows
            // into the elements, so `(1, 2.0): (i64, f64)` works; otherwise
            // each element defaults on its own.
            ast::ExprKind::Tuple(items) => {
                let wanted = match expected {
                    Some(e) => match self.types.kind(e) {
                        TyKind::Tuple(tys) if tys.len() == items.len() => Some(tys.clone()),
                        _ => None,
                    },
                    None => None,
                };
                let elems: Vec<Expr> = match wanted {
                    Some(tys) => items
                        .iter()
                        .zip(tys)
                        .map(|(item, want)| self.check_expr(item, want))
                        .collect(),
                    None => items.iter().map(|item| self.synth_committed(item)).collect(),
                };
                let tys: Vec<Ty> = elems.iter().map(|e| e.ty).collect();
                let ty = self.types.intern(TyKind::Tuple(tys));
                Expr { ty, kind: ExprKind::TupleLit(elems), span }
            }

            // `[a, b, c]`. With no expected type the first element fixes the
            // element type and the rest are checked against it, which puts the
            // error on the element that disagrees rather than on the whole
            // literal.
            ast::ExprKind::ArrayLit(items) => {
                let mut elems: Vec<Expr> = Vec::with_capacity(items.len());
                let elem_ty = match self.expected_elem(expected) {
                    Some(ty) => ty,
                    None => {
                        let Some(first) = items.first() else {
                            self.error(
                                codes::E2060,
                                span,
                                "cannot infer the element type of an empty array",
                            );
                            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                        };
                        let first = self.synth_committed(first);
                        let ty = first.ty;
                        elems.push(first);
                        ty
                    }
                };
                for item in items.iter().skip(elems.len()) {
                    let elem = self.check_expr(item, elem_ty);
                    elems.push(elem);
                }
                let len = elems.len() as u64;
                let ty = self.types.intern(TyKind::Array { elem: elem_ty, len });
                Expr { ty, kind: ExprKind::ArrayLit(elems), span }
            }

            // `[value; count]`.
            ast::ExprKind::ArrayRepeat { value, count } => {
                let Some(count) = self.const_len(count) else {
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                };
                let value = match self.expected_elem(expected) {
                    Some(want) => self.check_expr(value, want),
                    None => self.synth_committed(value),
                };
                let ty = self.types.intern(TyKind::Array { elem: value.ty, len: count });
                Expr { ty, kind: ExprKind::ArrayRepeat { value: Box::new(value), count }, span }
            }

            // `t.0`. A tuple element is a `Field` in HIR, exactly like a
            // struct field; only the way the index is found differs.
            ast::ExprKind::TupleField { base, index } => {
                let mut base = self.synth(base);
                while self.box_inner(base.ty).is_some() {
                    base = self.read_box_through(base);
                }
                let index = *index as usize;
                let found = match self.types.kind(base.ty) {
                    TyKind::Tuple(items) => Some((items.len(), items.get(index).copied())),
                    _ => None,
                };
                match found {
                    Some((_, Some(ty))) => {
                        Expr { ty, kind: ExprKind::Field { base: Box::new(base), index }, span }
                    }
                    Some((len, None)) => {
                        self.error(
                            codes::E2020,
                            span,
                            format!("this tuple has {len} elements, so `.{index}` is out of range"),
                        );
                        Expr { ty: self.common.error, kind: ExprKind::Error, span }
                    }
                    None => {
                        if base.ty != self.common.error {
                            let shown = self.types.display(base.ty);
                            self.error(codes::E2020, span, format!("`{shown}` is not a tuple"));
                        }
                        Expr { ty: self.common.error, kind: ExprKind::Error, span }
                    }
                }
            }

            // `a[i]`. `[GRM-8]` leaves this ambiguous with a generic
            // instantiation until name resolution; an array base settles it.
            // The bounds check is added when this is lowered to MIR.
            ast::ExprKind::IndexOrInstantiate { base, args } => {
                let mut base = self.synth(base);
                while self.box_inner(base.ty).is_some() {
                    base = self.read_box_through(base);
                }
                let elem = match self.types.kind(base.ty) {
                    // `[SPN-2]` — "Indexing a `Span` is bounds-checked".
                    TyKind::Array { elem, .. }
                    | TyKind::Vec { elem }
                    | TyKind::Span { elem, .. } => Some(*elem),
                    _ => None,
                };
                let Some(elem) = elem else {
                    if base.ty != self.common.error {
                        let shown = self.types.display(base.ty);
                        self.error(codes::E2020, span, format!("cannot index `{shown}`"));
                    }
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                };
                if args.len() != 1 {
                    self.error(
                        codes::E2020,
                        span,
                        format!("an array index takes one value, but {} were given", args.len()),
                    );
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                }
                // `[GRM-8b]` — the node resolved to an index, so a type or
                // an associated-type binding among its arguments is an error
                // that names the argument rather than a type mismatch.
                let ast::TypeOrExpr::Expr(index_expr) = &args[0] else {
                    self.error(
                        codes::E2172,
                        args[0].span(),
                        "cannot index with a type",
                    );
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                };
                let usize_ty = self.common.usize;
                let index = self.check_expr(index_expr, usize_ty);
                Expr {
                    ty: elem,
                    kind: ExprKind::Index { base: Box::new(base), index: Box::new(index) },
                    span,
                }
            }

            ast::ExprKind::Match { scrutinee, arms } => {
                self.check_match(scrutinee, arms, expected, span)
            }

            // `[ERR-2]` — `e?` is the payload, or an early return of the
            // failure.
            ast::ExprKind::Try(inner) => self.synth_try(inner, span),

            // `[LEX-19]` — an f-string builds a `String`.
            ast::ExprKind::FString(parts) => {
                let mut checked = Vec::new();
                for part in parts {
                    match part {
                        ast::FStringPart::Text(text) => {
                            checked.push(hir::FStringPart::Text(text.clone()))
                        }
                        ast::FStringPart::Expr { expr, format_spec } => {
                            if format_spec.is_some() {
                                self.error(
                                    codes::E1010,
                                    expr.span,
                                    "a format spec is not supported yet in this phase",
                                );
                            }
                            let value = self.synth_committed(expr);
                            if !self.is_formattable(value.ty) && value.ty != self.common.error {
                                let shown = self.types.display(value.ty);
                                self.error(
                                    codes::E1010,
                                    expr.span,
                                    format!("`{shown}` cannot be formatted yet; `Display` needs generics"),
                                );
                            }
                            checked.push(hir::FStringPart::Value(value));
                        }
                    }
                }
                let u8_ty = self.common.u8;
                let ty = self.types.intern(TyKind::Vec { elem: u8_ty });
                let buffer_ref = self.types.intern(TyKind::Ref { mutable: true, inner: ty });
                Expr { ty, kind: ExprKind::FString { parts: checked, buffer_ref }, span }
            }

            // `Shape.Circle(1.0)` parses as a method call on `Shape`, because
            // the parser cannot know `Shape` is a type. If it names an enum,
            // this is a variant constructor (`[ENM-1]`).
            // `[MOD-3]` — `import a.b.ops` then `ops.add(x)`. The parser sees
            // a method call, because it cannot know `ops` is a module.
            ast::ExprKind::MethodCall { recv, name, generic_args, args }
                if self.namespace_named(recv).is_some() =>
            {
                let module = self.namespace_named(recv).expect("just checked");
                let prefix = &self.prefixes[module];
                let qualified = if prefix.is_empty() {
                    name.name
                } else {
                    Symbol::intern(&format!("{prefix}.{}", name.name))
                };
                let explicit = self.resolve_method_type_args(generic_args);
                self.synth_qualified_call(qualified, name.span, args, explicit, span)
            }

            ast::ExprKind::MethodCall { recv, name, generic_args, args }
                if self.enum_named(recv).is_some() =>
            {
                self.reject_method_type_args(name.name, generic_args, span);
                let id = self.enum_named(recv).expect("just checked");
                self.synth_variant(id, *name, args, span)
            }

            // `[ARN-4]` — `Arena.with_capacity(bytes)` is an associated
            // constructor. The parser cannot know that the name on the left
            // is a type, so it arrives in the same method-call shape as enum
            // and range constructors.
            ast::ExprKind::MethodCall { recv, name, generic_args, args }
                if is_single_path(recv, "Arena") && self.lookup(Symbol::intern("Arena")).is_none() =>
            {
                self.synth_arena_construction(*name, generic_args, args, span)
            }

            // `[RNG-10]`'s construction set: `Roughness.checked(x)`,
            // `Roughness.clamped(x)`, `unsafe Roughness.new_unchecked(x)`.
            // A type name on the left is an associated function, not a
            // receiver, so it is matched here beside the variant constructor
            // rather than in `[TYP-24]`'s method resolution.
            ast::ExprKind::MethodCall { recv, name, generic_args, args }
                if self.range_named(recv).is_some() =>
            {
                self.reject_method_type_args(name.name, generic_args, span);
                let id = self.range_named(recv).expect("just checked");
                self.synth_range_construction(id, *name, args, span)
            }

            // `[TYP-24]`, Part IV.11 — `recv.m(args)`.
            ast::ExprKind::MethodCall { recv, name, generic_args, args } => {
                // `[CLS-4]` — `super.init(...)` is constructor syntax, not
                // ordinary inherited method dispatch. It is checked against
                // the direct base constructor and records the base-field
                // initialization fact before the body continues.
                if is_single_path(recv, "super") {
                    return self.synth_super_init(*name, generic_args, args, span);
                }
                if let Some(ty) = self.maybe_uninit_named(recv) {
                    return self.synth_maybe_uninit_construction(
                        ty,
                        *name,
                        generic_args,
                        args,
                        span,
                    );
                }
                if let Some(owner) = self.associated_type_named(recv) {
                    return self.synth_associated_call(
                        owner,
                        *name,
                        generic_args,
                        args,
                        span,
                    );
                }
                self.synth_method_call(recv, *name, generic_args, args, span)
            }

            ast::ExprKind::Call { callee, args } => self.synth_call(callee, args, expected, span),

            ast::ExprKind::Binary { op, lhs, rhs } => self.synth_binary(*op, lhs, rhs, span),

            ast::ExprKind::Logical { op, lhs, rhs } => {
                let bool_ty = self.common.bool_;
                let lhs = self.check_expr(lhs, bool_ty);
                let rhs = self.check_expr(rhs, bool_ty);
                let op = match op {
                    ast::LogicalOp::And => BinOp::And,
                    ast::LogicalOp::Or => BinOp::Or,
                };
                Expr {
                    ty: bool_ty,
                    kind: ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                    span,
                }
            }

            ast::ExprKind::Unary { op, operand } => {
                let operand = self.synth(operand);
                if *op == ast::UnOp::Not && operand.ty != self.common.bool_ && operand.ty != self.common.error {
                    let shown = self.types.display(operand.ty);
                    self.error(codes::E2020, span, format!("`not` needs a `bool`, found `{shown}`"));
                }
                let (hir_op, ty) = match op {
                    ast::UnOp::Neg => (UnOp::Neg, operand.ty),
                    ast::UnOp::BitNot => (UnOp::BitNot, operand.ty),
                    ast::UnOp::Not => (UnOp::Not, self.common.bool_),
                };
                Expr {
                    ty,
                    kind: ExprKind::Unary { op: hir_op, operand: Box::new(operand) },
                    span,
                }
            }

            ast::ExprKind::Cast { expr: inner, ty } => {
                let to = self.resolve_type(ty);
                let inner = self.synth_committed(inner);
                // `[ENM-3]` — a unit-only enum casts to an integer, because it
                // is only its discriminant. The reverse needs `from_repr`,
                // since not every integer names a variant.
                if let TyKind::Enum(id) = *self.types.kind(inner.ty) {
                    if self.types.enum_def(id).is_unit_only() && self.types.is_integral(to) {
                        return Expr {
                            ty: to,
                            kind: ExprKind::Cast { expr: Box::new(inner), to },
                            span,
                        };
                    }
                }
                if !self.types.is_numeric(inner.ty) || !self.types.is_numeric(to) {
                    // `[TYP-7]` — pointer casts require `unsafe`, which Phase 0
                    // does not implement.
                    let from = self.types.display(inner.ty);
                    let shown = self.types.display(to);
                    self.error(
                        codes::E2020,
                        span,
                        format!("`{from}` cannot be cast to `{shown}` with `as`"),
                    );
                }
                Expr { ty: to, kind: ExprKind::Cast { expr: Box::new(inner), to }, span }
            }

            // `[BRW-*]` — `ref place` and `ref mut place`, the explicit forms.
            // They are needed only when initialising a `ref`-typed local or a
            // view struct's field; every other borrow is implicit in a
            // parameter mode. `synth` of the operand yields the place, and
            // for a `ref` local it yields the deref, which makes `ref mut r`
            // a reborrow (`[BRW-6]`) with no extra machinery.
            ast::ExprKind::RefOf { mutable, place } => {
                let inner = self.synth(place);
                if inner.ty == self.common.error {
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                }
                // `[MOD-7]` — "taking `ref mut h.value`" is a write.
                if *mutable {
                    self.reject_readonly_write(&inner, place.span);
                    let through_shared_ref =
                        self.reject_write_through_shared_ref(&inner, place.span);
                    if !through_shared_ref {
                        self.reject_borrowed_parameter_write(&inner, place.span, true);
                    }
                }
                if !is_place(&inner.kind) {
                    self.error(
                        codes::E2140,
                        place.span,
                        "only a place can be borrowed",
                    );
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                }
                let ty = self.types.intern(TyKind::Ref { mutable: *mutable, inner: inner.ty });
                Expr {
                    ty,
                    kind: ExprKind::Ref { place: Box::new(inner), mutable: *mutable },
                    span,
                }
            }

            // `[GRM-16]` — a jump has type `!` and produces no value. Every
            // position it can legally occupy is statement-like and is lowered
            // by `lower_jump` before reaching here: an expression statement or
            // a `=>` arm. Anything else (`f(return x)`, a ternary branch) needs
            // a jump in the HIR and MIR expression path, which is not built.
            ast::ExprKind::Jump(jump) => {
                let word = jump.keyword();
                self.error(
                    codes::E1010,
                    span,
                    format!("`{word}` is not usable in this position yet"),
                );
                Expr { ty: self.common.never, kind: ExprKind::Error, span }
            }

            _ => {
                self.error(
                    codes::E1010,
                    span,
                    "this expression is not supported yet in this phase of the compiler",
                );
                Expr { ty: self.common.error, kind: ExprKind::Error, span }
            }
        }
    }

    fn synth_literal(&mut self, lit: &ast::Literal, span: Span) -> Expr {
        match lit {
            ast::Literal::Int { value, suffix } => {
                let ty = match suffix {
                    Some(s) => self.int_suffix_ty(*s),
                    // `[LEX-16]` — untyped until context decides.
                    None => self.common.int_lit,
                };
                Expr { ty, kind: ExprKind::Int(*value), span }
            }
            ast::Literal::Float { value, suffix, .. } => {
                let ty = match suffix {
                    Some(s) => self.float_suffix_ty(*s),
                    None => self.common.float_lit,
                };
                Expr { ty, kind: ExprKind::Float(*value), span }
            }
            ast::Literal::Bool(v) => {
                Expr { ty: self.common.bool_, kind: ExprKind::Bool(*v), span }
            }
            // `[LEX-20]` — a string literal is `str` with static region.
            ast::Literal::Str(s) => {
                Expr { ty: self.common.str_, kind: ExprKind::Str(s.clone()), span }
            }
            ast::Literal::Char(c) => {
                Expr { ty: self.common.char_, kind: ExprKind::Int(*c as u128), span }
            }
            _ => {
                self.error(codes::E1010, span, "this literal is not supported yet in this phase");
                Expr { ty: self.common.error, kind: ExprKind::Error, span }
            }
        }
    }

    fn int_suffix_ty(&self, suffix: ast::ember_lexer_types::IntSuffix) -> Ty {
        use ast::ember_lexer_types::IntSuffix as S;
        let c = self.common;
        match suffix {
            S::I8 => c.i8,
            S::I16 => c.i16,
            S::I32 => c.i32,
            S::I64 => c.i64,
            S::I128 => c.i128,
            S::Isize => c.isize,
            S::U8 => c.u8,
            S::U16 => c.u16,
            S::U32 => c.u32,
            S::U64 => c.u64,
            S::U128 => c.u128,
            S::Usize => c.usize,
        }
    }

    fn float_suffix_ty(&self, suffix: ast::ember_lexer_types::FloatSuffix) -> Ty {
        use ast::ember_lexer_types::FloatSuffix as S;
        match suffix {
            S::F16 => self.common.f16,
            S::F32 => self.common.f32,
            S::F64 => self.common.f64,
        }
    }

    fn synth_call(&mut self, callee: &ast::Expr, args: &[ast::Arg], expected: Option<Ty>, span: Span) -> Expr {
        // `Shape.Circle(1.0)` — a variant constructor (`[ENM-1]`), which
        // parses as a call on a field access.
        if let ast::ExprKind::Field { base, name } = &callee.kind {
            if let Some(id) = self.enum_named(base) {
                return self.synth_variant(id, *name, args, span);
            }
            // `[RNG-10]`'s construction set: `Roughness.checked(x)`,
            // `Roughness.clamped(x)`, `unsafe Roughness.new_unchecked(x)`.
            // These parse the same way a variant constructor does.
            if let Some(id) = self.range_named(base) {
                return self.synth_range_construction(id, *name, args, span);
            }
        }
        // `[TYP-18]` — `f[i32](x)` names the instantiation explicitly, which
        // parses as a call on an index.
        let (callee, explicit): (&ast::Expr, Vec<Ty>) = match &callee.kind {
            ast::ExprKind::IndexOrInstantiate { base, args }
                if matches!(base.kind, ast::ExprKind::Path { .. }) =>
            {
                // `[GRM-8b]` — the node resolved to an instantiation, so each
                // argument parsed as an expression is reinterpreted as a type
                // or a const-generic argument by the ordinary rules.
                let tys = args
                    .iter()
                    .map(|a| match a {
                        ast::TypeOrExpr::Type(ty) => self.resolve_type(ty),
                        ast::TypeOrExpr::Expr(e) => self.type_from_expr(e),
                        ast::TypeOrExpr::Binding { name, .. } => {
                            let name = name.name;
                            self.error(
                                codes::E2173,
                                a.span(),
                                format!("`{name} = …` binds an associated type, which this instantiation does not take"),
                            );
                            self.common.error
                        }
                    })
                    .collect::<Vec<Ty>>();
                (base.as_ref(), tys)
            }
            _ => (callee, Vec::new()),
        };
        // `[CLO-3]` — "Calling: `f(args)`". A local of function type is
        // called through its value rather than by name. Checked before the
        // path lookup, because a local shadows an item of the same name.
        if let ast::ExprKind::Path { segments } = &callee.kind {
            if segments.len() == 1 {
                if let Some(local) = self.lookup(segments[0].name) {
                    let value = self.read_local_expecting(local, callee.span, None);
                    if matches!(self.types.kind(value.ty), TyKind::Fn { .. }) {
                        if let Some(&def) = self.callable_value_bindings.get(&local) {
                            let latebound =
                                self.latebound_callable_parameter_locals.contains(&local);
                            return self.synth_known_function_call(def, args, span, latebound);
                        }
                        let consumes_callee = self.callable_once_locals.contains(&local);
                        let latebound =
                            self.latebound_callable_parameter_locals.contains(&local);
                        return self.synth_indirect_call(
                            value,
                            args,
                            span,
                            consumes_callee,
                            latebound,
                        );
                    }
                    // `[CLO-3]` — inside a generic body a `fn(A) -> R`
                    // parameter is opaque, and what makes it callable is its
                    // bound rather than its type. `[TYP-17]` is the same rule
                    // as for any other bound: only what the bound provides is
                    // permitted, and calling it is what `Callable` provides.
                    if let Some((fn_ty, consumes_callee)) = self.callable_bound_of(value.ty) {
                        let callee = Expr { ty: fn_ty, ..value };
                        return self.synth_indirect_call(callee, args, span, consumes_callee, false);
                    }
                    // `[CLO-1]` — a capturing closure is an anonymous struct,
                    // and calling one is a **direct** call to its body with the
                    // environment passed first. This is the instantiated form
                    // of the branch above: once the generic parameter is bound
                    // to a concrete closure type there is nothing opaque left
                    // and nothing indirect about the call, which is the
                    // "static dispatch, monomorphised" `[CLO-3]` asks for and
                    // what lets `[COST-3]` report a direct call.
                    if let TyKind::Struct(id) = *self.types.kind(value.ty) {
                        if self.closure_calls.contains_key(&id) {
                            let latebound =
                                self.latebound_callable_parameter_locals.contains(&local);
                            return self.synth_closure_call(value, id, args, span, latebound);
                        }
                    }
                }
            }
        }
        let ast::ExprKind::Path { segments } = &callee.kind else {
            self.error(codes::E1010, span, "only direct calls are supported in this phase");
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };
        // `[MOD-3]` — `import a.b.c` binds `c` as a namespace, so `c.f(x)` is
        // a call into that module.
        let name = match segments.len() {
            1 => segments[0].name,
            2 => match self.resolve_qualified(segments) {
                Some(qualified) => qualified,
                None => {
                    self.error(
                        codes::E1010,
                        span,
                        format!("`{}` is not a module in scope", segments[0].name),
                    );
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                }
            },
            _ => {
                self.error(codes::E1010, span, "a path this long is not supported yet");
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
        };

        // `Vec3(1, 2, 3)` — the synthesised memberwise constructor (`[STR-1]`).
        if let Some(&id) = self.struct_ids.get(&self.resolve_name(name)) {
            return self.synth_struct_literal(id, name, args, span);
        }

        // `[CLS-1]` — the source-reachable class construction slice. Keep the
        // supported boundary explicit until user constructors, inheritance,
        // defaults, and recursive ownership/drop lowering are available: an
        // object must never become reachable with an invented or partial field
        // state. Empty classes and memberwise classes whose fields are all
        // already non-dropping values can be initialized directly after the
        // runtime allocates their header.
        if let Some(&id) = self.class_ids.get(&self.resolve_name(name)) {
            return self.synth_class_constructor(id, name, args, &explicit, span);
        }

        // `[ERR-1]` — `Some(x)`, `Ok(x)` and `Err(e)` build the compiler-known
        // enums. The expected type says which one when it is known; otherwise
        // the payload's own type decides and the other parameter stays open,
        // which needs an annotation.
        if let Some(built) = self.synth_wrapper(name, args, expected, span) {
            return built;
        }

        // IX.1 — `Box(owned v)`, or `Box[T](owned v)`. The allocation is an
        // explicit builtin rather than a private-field literal: MIR must see
        // the owned move, and the backend must route the allocation through
        // `[HEAP-1]`'s allocator rather than materialising an inline payload.
        if name.is("Box") {
            if explicit.len() > 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!("`Box` takes one type argument, found {}", explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            if args.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!("`Box` takes one argument, found {}", args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let expected_inner = expected.and_then(|ty| self.box_inner(ty));
            let hint = explicit.first().copied().or(expected_inner);
            let value = match hint {
                Some(inner) => self.check_expr(&args[0].value, inner),
                None => self.synth_committed(&args[0].value),
            };
            if value.ty == self.common.error {
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let inner = value.ty;
            // `[TYP-15]` is region-based: forming `Box[str]` is legal, and
            // whether this particular value may enter unbounded storage
            // depends on its inferred region rather than its syntax. The MIR
            // region pass checks the Box builtin after provenance has flowed
            // through locals and calls; doing it here would wrongly reject a
            // static view merely because it was first bound to a name.
            let boxed = self.box_of(inner);
            return Expr {
                ty: boxed,
                kind: ExprKind::Builtin {
                    which: Builtin::BoxNew { elem: inner, boxed },
                    args: vec![value],
                },
                span,
            };
        }

        // `[UNS-10]` — `std.mem.UnsafeCell(owned v)`. Its public name is
        // source-backed for module visibility, but its representation and two
        // operations are compiler-known while the unsafe standard-library
        // substrate is staged. Recognize it before the ordinary generic
        // struct constructor so the source declaration cannot expose its
        // private payload field as an implementation accident.
        if self.is_unsafe_cell_name(name) {
            if explicit.len() > 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!(
                        "`UnsafeCell` takes one type argument, found {}",
                        explicit.len()
                    ),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            if args.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!("`UnsafeCell` takes one argument, found {}", args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let hint = explicit
                .first()
                .copied()
                .or_else(|| expected.and_then(|ty| self.unsafe_cell_inner(ty)));
            let value = match hint {
                Some(inner) => self.check_expr(&args[0].value, inner),
                None => {
                    let synthesised = self.synth_committed(&args[0].value);
                    self.read_through(synthesised)
                }
            };
            if value.ty == self.common.error {
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let inner = value.ty;
            self.reject_stored_view(inner, args[0].value.span, "an UnsafeCell's contents");
            let ty = self.unsafe_cell_of(inner);
            self.reject_unsafe_cell_in_static_safe(span);
            let TyKind::Struct(id) = *self.types.kind(ty) else {
                unreachable!("UnsafeCell is compiler-known as a struct")
            };
            return Expr {
                ty,
                kind: ExprKind::StructLit { struct_id: id, fields: vec![value] },
                span,
            };
        }

        // `Pair(1, 2.5)` — a generic struct's constructor, with the type
        // arguments inferred from the values, or written as `Pair[i32, f32]`.
        let resolved_name = self.resolve_name(name);
        if let Some(decl) = self.generic_structs.get(&resolved_name).cloned() {
            return self.synth_generic_struct_literal(resolved_name, &decl, args, &explicit, span);
        }

        // `[UNS-5]`, `std.mem` — the raw memory primitives.
        if let Some(built) = self.synth_memory_builtin(name, args, &explicit, span) {
            return built;
        }

        // `[CELL-1]` — `Cell(v)`, or `Cell[T](v)` where the value alone does
        // not say what `T` is. The payload is `owned`: the cell takes the
        // value, it does not observe one.
        //
        // A plain struct literal, not a builtin. `[CELL-2]` asks for "no
        // overhead relative to a plain field", and the way to get that is to
        // emit the thing that has none.
        if name.is("Cell") {
            if args.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!("`Cell` takes one argument, found {}", args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            // Written `Cell[i32](0)`, expected from the annotation, or left to
            // the value. The written form wins, so a literal can be widened.
            let hint = explicit
                .first()
                .copied()
                .or_else(|| expected.and_then(|e| self.cell_inner(e)));
            let value = match hint {
                Some(inner) => self.check_expr(&args[0].value, inner),
                None => {
                    let synthesised = self.synth_committed(&args[0].value);
                    self.read_through(synthesised)
                }
            };
            if value.ty == self.common.error {
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let inner = value.ty;
            self.reject_stored_view(inner, args[0].value.span, "a cell's contents");
            let ty = self.cell_of(inner);
            let TyKind::Struct(id) = *self.types.kind(ty) else {
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            };
            return Expr {
                ty,
                kind: ExprKind::StructLit { struct_id: id, fields: vec![value] },
                span,
            };
        }

        // `[CELL-5]` — `RefCell(v)`, or `RefCell[T](v)`. As `Cell(v)`, a plain
        // struct literal, not a builtin: the borrow counter starts at `0`
        // (unborrowed) with no conflicting location, so the literal carries
        // the value plus three zero fields. `[CELL-9]`'s one-word counter is
        // field 1; fields 2–3 are its source location for `[CELL-5]`'s panic.
        if name.is("RefCell") {
            if args.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!("`RefCell` takes one argument, found {}", args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let hint = explicit
                .first()
                .copied()
                .or_else(|| expected.and_then(|e| self.refcell_inner(e)));
            let value = match hint {
                Some(inner) => self.check_expr(&args[0].value, inner),
                None => {
                    let synthesised = self.synth_committed(&args[0].value);
                    self.read_through(synthesised)
                }
            };
            if value.ty == self.common.error {
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let inner = value.ty;
            self.reject_stored_view(inner, args[0].value.span, "a cell's contents");
            let ty = self.refcell_of(inner);
            let TyKind::Struct(id) = *self.types.kind(ty) else {
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            };
            let def = self.types.struct_def(id).clone();
            // Fields 1–3 are the borrow state: `0`, null file, `0` line.
            let zero_isize = Expr { ty: def.fields[1].ty, kind: ExprKind::Int(0), span };
            // A null `*u8` as integer zero with the pointer type: C renders a
            // plain `0`, which is the null pointer constant (no cast, no
            // `-Wall` noise).
            let null_file = Expr { ty: def.fields[2].ty, kind: ExprKind::Int(0), span };
            let zero_u32 = Expr { ty: def.fields[3].ty, kind: ExprKind::Int(0), span };
            return Expr {
                ty,
                kind: ExprKind::StructLit {
                    struct_id: id,
                    fields: vec![value, zero_isize, null_file, zero_u32],
                },
                span,
            };
        }

        // `Array[T]()` and `String()` — the compiler-known constructors.
        if name.is("Array") || name.is("String") {
            if !args.is_empty() {
                self.error(codes::E2020, span, format!("`{name}()` takes no arguments"));
            }
            let ty = if name.is("String") {
                let u8_ty = self.common.u8;
                self.types.intern(TyKind::Vec { elem: u8_ty })
            } else {
                match expected.filter(|e| matches!(self.types.kind(*e), TyKind::Vec { .. })) {
                    Some(ty) => ty,
                    None => {
                        self.error(
                            codes::E2060,
                            span,
                            "cannot tell what this `Array` holds; annotate the variable",
                        );
                        return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                    }
                }
            };
            let which =
                if name.is("String") { Builtin::StringNew } else { Builtin::ArrayNew };
            return Expr { ty, kind: ExprKind::Builtin { which, args: Vec::new() }, span };
        }

        if let Some(builtin) = Builtin::from_name(name.as_str()) {
            let args: Vec<Expr> = args
                .iter()
                .map(|a| {
                    let arg = self.synth_committed(&a.value);
                    self.read_through(arg)
                })
                .collect();
            if args.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes one argument, found {}", builtin.name(), args.len()),
                );
            }
            return Expr {
                ty: self.common.void,
                kind: ExprKind::Builtin { which: builtin, args },
                span,
            };
        }

        let Some(&def) = self.fn_ids.get(&self.resolve_name(name)) else {
            self.error(codes::E1010, segments[0].span, format!("cannot find `{name}` in this scope"));
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };

        // `[TYP-16]`, `[TYP-18]` — a generic callee is instantiated here: the
        // arguments say what its parameters are, and the instance gets its own
        // symbol.
        if !self.signatures[def.0 as usize].generics.is_empty() {
            return self.synth_generic_call(def, name, args, explicit, span);
        }

        let signature: Vec<(Ty, Mode)> =
            self.signatures[def.0 as usize].params.iter().map(|(_, t, m, _)| (*t, *m)).collect();
        let ret = self.signatures[def.0 as usize].ret;
        if args.len() != signature.len() {
            self.error(
                codes::E2020,
                span,
                format!("`{name}` takes {} arguments, found {}", signature.len(), args.len()),
            );
        }
        let params = self.signatures[def.0 as usize].params.clone();
        let slots = self.call_argument_slots(name, args, &params);
        let checked = self.check_bound_call_arguments(args, &params, &slots);
        Expr {
            ty: ret,
            kind: ExprKind::Call {
                callee: def,
                arg_eval_order: Self::call_eval_order(&slots),
                args: checked,
                latebound: false,
            },
            span,
        }
    }

    /// `f[i32]` writes its type argument in expression position, so the
    /// argument arrives as an expression and has to be read back as a type.
    fn type_from_expr(&mut self, expr: &ast::Expr) -> Ty {
        match &expr.kind {
            ast::ExprKind::Path { segments } => {
                let ty = ast::TypeExpr {
                    id: ast::NodeId(0),
                    kind: ast::TypeKind::Path {
                        segments: segments.clone(),
                        args: Vec::new(),
                    },
                    span: expr.span,
                };
                self.resolve_type(&ty)
            }
            ast::ExprKind::IndexOrInstantiate { base, args } => {
                let ast::ExprKind::Path { segments } = &base.kind else {
                    self.error(codes::E1010, expr.span, "expected a type argument");
                    return self.common.error;
                };
                if segments.len() != 1 {
                    self.error(codes::E1010, expr.span, "expected a type argument");
                    return self.common.error;
                }
                let mut resolved = Vec::with_capacity(args.len());
                for arg in args {
                    let ty = match arg {
                        ast::TypeOrExpr::Type(ty) => self.resolve_type(ty),
                        ast::TypeOrExpr::Expr(expr) => self.type_from_expr(expr),
                        ast::TypeOrExpr::Binding { name, .. } => {
                            self.error(
                                codes::E2173,
                                name.span,
                                format!(
                                    "`{} = …` binds an associated type, which this type does not take",
                                    name.name
                                ),
                            );
                            return self.common.error;
                        }
                    };
                    resolved.push((ty, arg.span()));
                }
                self.resolve_type_application(segments[0].name, &resolved, expr.span)
            }
            _ => {
                self.error(codes::E1010, expr.span, "expected a type argument");
                self.common.error
            }
        }
    }

    /// Recognise the type receiver in
    /// `MaybeUninit[T].uninit()`. `[GRM-8]` deliberately leaves the brackets
    /// ambiguous, so the receiver arrives as an expression-shaped
    /// `IndexOrInstantiate` rather than as a `TypeExpr`.
    fn maybe_uninit_named(&mut self, expr: &ast::Expr) -> Option<Ty> {
        let ast::ExprKind::IndexOrInstantiate { base, args } = &expr.kind else {
            return None;
        };
        let ast::ExprKind::Path { segments } = &base.kind else {
            return None;
        };
        if segments.len() != 1
            || !segments[0].name.is("MaybeUninit")
            || self.lookup(Symbol::intern("MaybeUninit")).is_some()
        {
            return None;
        }
        if args.len() != 1 {
            self.error(
                codes::E2020,
                expr.span,
                "`MaybeUninit` takes one type argument",
            );
            return Some(self.common.error);
        }
        let inner = match &args[0] {
            ast::TypeOrExpr::Type(ty) => self.resolve_type(ty),
            ast::TypeOrExpr::Expr(expr) => self.type_from_expr(expr),
            ast::TypeOrExpr::Binding { name, .. } => {
                self.error(
                    codes::E2173,
                    name.span,
                    "`MaybeUninit` takes a type, not an associated-type binding",
                );
                self.common.error
            }
        };
        Some(self.maybe_uninit_of(inner))
    }

    /// `[GRM-8]`, `[TYP-18]` — bracket arguments on a method call are parsed
    /// before name resolution knows whether an identifier denotes a type or a
    /// const. Generic methods currently consume type parameters, so a path in
    /// expression form is reinterpreted exactly as it is for `f[T](...)`.
    fn resolve_method_type_args(&mut self, args: &[ast::GenericArg]) -> Vec<Ty> {
        args.iter()
            .map(|arg| match arg {
                ast::GenericArg::Type(ty) => self.resolve_type(ty),
                ast::GenericArg::Const(expr) => self.type_from_expr(expr),
                ast::GenericArg::Assoc { name, .. } => {
                    self.error(
                        codes::E2173,
                        name.span,
                        format!(
                            "`{} = …` binds an associated type, which this method instantiation does not take",
                            name.name
                        ),
                    );
                    self.common.error
                }
            })
            .collect()
    }

    fn reject_method_type_args(
        &mut self,
        name: Symbol,
        generic_args: &[ast::GenericArg],
        span: Span,
    ) -> bool {
        if generic_args.is_empty() {
            return false;
        }
        self.error(
            codes::E2020,
            span,
            format!("`{name}` takes no type arguments, found {}", generic_args.len()),
        );
        true
    }

    /// `[TYP-16]`, `[TYP-18]` — a call to a generic function. The parameters
    /// are inferred from the arguments (or given explicitly), the bounds are
    /// checked, and the instantiation gets its own `DefId` and symbol.
    fn synth_generic_call(
        &mut self,
        def: DefId,
        name: Symbol,
        args: &[ast::Arg],
        explicit: Vec<Ty>,
        span: Span,
    ) -> Expr {
        let generics = self.signatures[def.0 as usize].generics.clone();
        let declared = self.signatures[def.0 as usize].params.clone();
        let ret = self.signatures[def.0 as usize].ret;

        if explicit.len() > generics.len() {
            self.error(
                codes::E2020,
                span,
                format!(
                    "`{name}` takes {} type arguments, found {}",
                    generics.len(),
                    explicit.len()
                ),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }

        if args.len() != declared.len() {
            self.error(
                codes::E2020,
                span,
                format!("`{name}` takes {} arguments, found {}", declared.len(), args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let slots = self.call_argument_slots(name, args, &declared);
        let declared_modes = declared
            .iter()
            .map(|(_, ty, mode, _)| (*ty, *mode))
            .collect::<Vec<_>>();

        // Explicit arguments come first; the rest are inferred by unifying
        // each declared parameter type with what the argument actually is.
        let mut solved: Vec<Option<Ty>> = vec![None; generics.len()];
        for (slot, ty) in explicit.iter().enumerate() {
            if slot < solved.len() {
                solved[slot] = Some(*ty);
            }
        }
        let mut checked_args: Vec<Option<Expr>> = (0..args.len()).map(|_| None).collect();
        // Solve non-lambda arguments first. Type inference is not evaluation,
        // so this does not change source order; it lets a later ordinary
        // argument determine `T` before an earlier `fn(T) -> T` lambda needs
        // its parameter expectation.
        for (arg_index, arg) in args.iter().enumerate() {
            let Some(index) = slots[arg_index] else { continue };
            let param_ty = declared[index].1;
            if matches!(arg.value.kind, ast::ExprKind::Lambda(_)) {
                continue;
            }
            let value = self.synth_committed(&arg.value);
            if !self.unify_generic_argument(
                param_ty,
                value.ty,
                &generics,
                &mut solved,
                explicit.len(),
            )
            {
                let want = self.types.display(param_ty);
                let got = self.types.display(value.ty);
                self.error(
                    codes::E2020,
                    arg.value.span,
                    format!("`{name}` cannot take `{got}` where it expects `{want}`"),
                );
            }
            checked_args[index] = Some(value);
        }
        for (arg_index, arg) in args.iter().enumerate() {
            let Some(index) = slots[arg_index] else { continue };
            let param_ty = declared[index].1;
            if !matches!(arg.value.kind, ast::ExprKind::Lambda(_)) {
                continue;
            }
            // `[TYP-23]` rule 4 — "Lambda parameter types are inferred from the
            // expected function type". For a `fn(A) -> R` parameter the
            // expected type is now an opaque `Param`, which tells a lambda
            // nothing, so the *bound's* signature is handed over instead. It is
            // a hint and not a requirement: the argument may be a lambda whose
            // own type is its environment, and it is `unify` below that decides
            // whether what came back fits.
            let hint = self.callable_hint(param_ty, &generics, &solved);
            let value = match hint {
                Some(fn_ty) => self.synth_with_hint(&arg.value, fn_ty),
                None => self.synth_committed(&arg.value),
            };
            if !self.unify_generic_argument(
                param_ty,
                value.ty,
                &generics,
                &mut solved,
                explicit.len(),
            )
            {
                let want = self.types.display(param_ty);
                let got = self.types.display(value.ty);
                self.error(
                    codes::E2020,
                    arg.value.span,
                    format!("`{name}` cannot take `{got}` where it expects `{want}`"),
                );
            }
            checked_args[index] = Some(value);
        }
        let checked_args = checked_args.into_iter().flatten().collect::<Vec<_>>();

        // `[TYP-18]` — a parameter no argument mentions must be written out.
        let mut substitution = Vec::new();
        for (index, param) in generics.iter().enumerate() {
            match solved[index] {
                Some(ty) => substitution.push(ty),
                None => {
                    self.error(
                        codes::E2060,
                        span,
                        format!(
                            "cannot tell what `{}` is here; write it out, as `{name}[T](...)`",
                            param.name
                        ),
                    );
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                }
            }
        }

        // `[TYP-17]` — every bound must actually be implemented.
        for (param, &ty) in generics.iter().zip(substitution.iter()) {
            for bound in &param.bounds {
                if !self.implements(ty, *bound) {
                    let shown = self.types.display(ty);
                    self.error(
                        codes::E2040,
                        span,
                        format!("`{shown}` does not implement `{bound}`, which `{}` requires", param.name),
                    );
                }
            }
        }
        self.reject_once_closures_for_callable_bounds(
            args,
            &declared_modes,
            &generics,
            &checked_args,
            &slots,
        );

        // Preserve a known capture-free function value through static
        // dispatch. Its body may carry a more precise late-bound result
        // provenance contract than the callable type alone can express.
        let mut callable_values = vec![None; generics.len()];
        for ((_, param_ty, _, _), value) in declared.iter().zip(&checked_args) {
            let TyKind::Param { index, .. } = *self.types.kind(*param_ty) else { continue };
            if generics
                .get(index as usize)
                .and_then(|param| param.callable.as_ref())
                .is_none_or(|bound| bound.once || !bound.latebound)
            {
                continue;
            }
            if let ExprKind::FnValue(def) = value.kind {
                callable_values[index as usize] = Some(def);
            }
        }
        // Re-check the arguments against the substituted parameter types, so
        // an untyped literal adopts the right one and a mismatch is reported
        // where it happens.
        let instance = self.instantiate(def, &substitution, callable_values, name, span);
        let concrete: Vec<(Ty, Mode)> = declared
            .iter()
            .map(|&(_, ty, mode, _)| (self.substitute_ty(ty, &substitution), mode))
            .collect();
        // `hir::Expr` is not `Clone` — a checked argument is moved out of here
        // rather than copied.
        let mut reusable: Vec<Option<Expr>> = checked_args.into_iter().map(Some).collect();
        let mut checked = Vec::new();
        let mut arg_for_param = vec![None; concrete.len()];
        for (arg_index, slot) in slots.iter().copied().enumerate() {
            if let Some(slot) = slot {
                arg_for_param[slot] = Some(arg_index);
            }
        }
        for (index, &(param_ty, mode)) in concrete.iter().enumerate() {
            let Some(arg_index) = arg_for_param[index] else { continue };
            let arg = &args[arg_index];
            // A lambda is **not** re-checked. Checking it again would build a
            // second closure — a second environment struct, a second body, a
            // second `DefId` — and the instance was already created against the
            // first one, so the argument would then not have the type the
            // instance takes (`expected closure0_env, found closure1_env`). The
            // re-check exists so an untyped literal can adopt the substituted
            // parameter type; a closure has nothing to adopt.
            let already = matches!(arg.value.kind, ast::ExprKind::Lambda(_));
            match reusable.get_mut(index).and_then(|slot| slot.take()).filter(|_| already) {
                Some(value) => checked.push(value),
                None => checked.push(self.check_argument(&arg.value, param_ty, mode)),
            }
        }
        let ret = self.substitute_ty(ret, &substitution);
        Expr {
            ty: ret,
            kind: ExprKind::Call {
                callee: instance,
                arg_eval_order: Self::call_eval_order(&slots),
                args: checked,
                latebound: false,
            },
            span,
        }
    }

    /// `[TYP-18]` for a source-defined method. Inference, bound checks and
    /// argument re-checking intentionally mirror `synth_generic_call`; only
    /// the receiver prefix and method-specific instantiation queue differ.
    fn synth_generic_method_call(
        &mut self,
        def: DefId,
        name: Symbol,
        receiver: Option<(Expr, Mode, Span)>,
        signature_has_receiver: bool,
        args: &[ast::Arg],
        explicit: Vec<Ty>,
        span: Span,
        instantiate: bool,
    ) -> Expr {
        let generics = self.signatures[def.0 as usize].generics.clone();
        let declared: Vec<(Symbol, Ty, Mode, Span)> = self.signatures[def.0 as usize]
            .params
            .iter()
            .skip(usize::from(signature_has_receiver))
            .copied()
            .collect();
        let ret = self.signatures[def.0 as usize].ret;

        if explicit.len() > generics.len() {
            self.error(
                codes::E2020,
                span,
                format!(
                    "`{name}` takes {} type arguments, found {}",
                    generics.len(),
                    explicit.len()
                ),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if args.len() != declared.len() {
            self.error(
                codes::E2020,
                span,
                format!("`{name}` takes {} arguments, found {}", declared.len(), args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let slots = self.call_argument_slots(name, args, &declared);
        let declared_modes = declared
            .iter()
            .map(|(_, ty, mode, _)| (*ty, *mode))
            .collect::<Vec<_>>();

        let mut solved: Vec<Option<Ty>> = vec![None; generics.len()];
        for (slot, ty) in explicit.iter().enumerate() {
            solved[slot] = Some(*ty);
        }
        let mut checked_args: Vec<Option<Expr>> = (0..args.len()).map(|_| None).collect();
        for (arg_index, arg) in args.iter().enumerate() {
            let Some(index) = slots[arg_index] else { continue };
            let param_ty = declared[index].1;
            if matches!(arg.value.kind, ast::ExprKind::Lambda(_)) {
                continue;
            }
            let value = self.synth_committed(&arg.value);
            if !self.unify_generic_argument(
                param_ty,
                value.ty,
                &generics,
                &mut solved,
                explicit.len(),
            )
            {
                let want = self.types.display(param_ty);
                let got = self.types.display(value.ty);
                self.error(
                    codes::E2020,
                    arg.value.span,
                    format!("`{name}` cannot take `{got}` where it expects `{want}`"),
                );
            }
            checked_args[index] = Some(value);
        }
        for (arg_index, arg) in args.iter().enumerate() {
            let Some(index) = slots[arg_index] else { continue };
            let param_ty = declared[index].1;
            if !matches!(arg.value.kind, ast::ExprKind::Lambda(_)) {
                continue;
            }
            let hint = self.callable_hint(param_ty, &generics, &solved);
            let value = match hint {
                Some(fn_ty) => self.synth_with_hint(&arg.value, fn_ty),
                None => self.synth_committed(&arg.value),
            };
            if !self.unify_generic_argument(
                param_ty,
                value.ty,
                &generics,
                &mut solved,
                explicit.len(),
            )
            {
                let want = self.types.display(param_ty);
                let got = self.types.display(value.ty);
                self.error(
                    codes::E2020,
                    arg.value.span,
                    format!("`{name}` cannot take `{got}` where it expects `{want}`"),
                );
            }
            checked_args[index] = Some(value);
        }
        let checked_args = checked_args.into_iter().flatten().collect::<Vec<_>>();

        let mut substitution = Vec::new();
        for (index, param) in generics.iter().enumerate() {
            match solved[index] {
                Some(ty) => substitution.push(ty),
                None => {
                    self.error(
                        codes::E2060,
                        span,
                        format!(
                            "cannot tell what `{}` is here; write it out, as `.{name}[T](...)`",
                            param.name
                        ),
                    );
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                }
            }
        }

        for (param, &ty) in generics.iter().zip(substitution.iter()) {
            for bound in &param.bounds {
                if !self.implements(ty, *bound) {
                    let shown = self.types.display(ty);
                    self.error(
                        codes::E2040,
                        span,
                        format!(
                            "`{shown}` does not implement `{bound}`, which `{}` requires",
                            param.name
                        ),
                    );
                }
            }
        }
        self.reject_once_closures_for_callable_bounds(
            args,
            &declared_modes,
            &generics,
            &checked_args,
            &slots,
        );

        let arg_eval_order = if receiver.is_some() {
            Self::call_eval_order_with_receiver(&slots)
        } else {
            Self::call_eval_order(&slots)
        };
        let instance = if instantiate {
            self.instantiate_method(def, &substitution, name, span)
        } else {
            // A call through an opaque interface bound is checked for its
            // contract but never emitted. Concrete monomorphisation rechecks
            // the body and resolves the implementing method.
            def
        };
        let concrete = declared
            .iter()
            .map(|&(_, ty, mode, _)| (self.substitute_ty(ty, &substitution), mode))
            .collect::<Vec<_>>();
        let mut reusable: Vec<Option<Expr>> = checked_args.into_iter().map(Some).collect();
        let mut checked = Vec::new();
        if let Some((receiver, receiver_mode, receiver_span)) = receiver {
            checked.push(self.pass_receiver(receiver, receiver_mode, receiver_span));
        }
        let mut arg_for_param = vec![None; concrete.len()];
        for (arg_index, slot) in slots.iter().copied().enumerate() {
            if let Some(slot) = slot {
                arg_for_param[slot] = Some(arg_index);
            }
        }
        for (index, &(param_ty, mode)) in concrete.iter().enumerate() {
            let Some(arg_index) = arg_for_param[index] else { continue };
            let arg = &args[arg_index];
            let already = matches!(arg.value.kind, ast::ExprKind::Lambda(_));
            match reusable.get_mut(index).and_then(|slot| slot.take()).filter(|_| already) {
                Some(value) => checked.push(value),
                None => checked.push(self.check_argument(&arg.value, param_ty, mode)),
            }
        }
        let ret = self.substitute_ty(ret, &substitution);
        Expr {
            ty: ret,
            kind: ExprKind::Call {
                callee: instance,
                arg_eval_order,
                args: checked,
                latebound: false,
            },
            span,
        }
    }

    /// `[CLO-6]` — `fn(A) -> R` is an implicit `Callable` bound, while
    /// `owned f: fn(A) -> R` is `CallableOnce`. A concrete closure whose
    /// checked body moves a non-`Copy` capture satisfies only the latter. The
    /// rejection belongs at the generic-call boundary, before the erased
    /// `Param` is instantiated, because that is the sole point that still
    /// carries the source parameter's callable capability.
    fn reject_once_closures_for_callable_bounds(
        &mut self,
        args: &[ast::Arg],
        declared: &[(Ty, Mode)],
        generics: &[GenericParam],
        checked_args: &[Expr],
        slots: &[Option<usize>],
    ) {
        for (arg_index, arg) in args.iter().enumerate() {
            let Some(param_index) = slots[arg_index] else { continue };
            let Some(&(param_ty, _)) = declared.get(param_index) else { continue };
            let Some(value) = checked_args.get(param_index) else { continue };
            let TyKind::Param { index, .. } = *self.types.kind(param_ty) else {
                continue;
            };
            let Some(bound) = generics.get(index as usize).and_then(|param| param.callable.as_ref()) else {
                continue;
            };
            if bound.once {
                continue;
            }
            let TyKind::Struct(id) = *self.types.kind(value.ty) else { continue };
            if !self.closure_calls.get(&id).is_some_and(|closure| closure.once) {
                continue;
            }
            self.sink.emit_classified(
                Diagnostic::error(
                    codes::E3030,
                    arg.value.span,
                    "closure would move a captured value out",
                )
                .primary_label("this closure is callable only once")
                .help(
                    "declare the parameter `owned f: fn(...) -> ...` so it accepts `CallableOnce`, or use `mem.take` when the capture implements `Default`; clone only as a last resort",
                )
                .note("the closure moves a non-`Copy` capture out of its environment [CLO-2]"),
            );
        }
    }

    /// `[CLO-1]`, `[CLO-6]` — call a closure through its environment.
    ///
    /// The environment is the first argument, which is `call(self, args)` in
    /// `[CLO-6]`'s shape with `self` written out. Every other argument is
    /// checked against the closure's own parameter types, which are concrete by
    /// the time anything gets here.
    fn synth_closure_call(
        &mut self,
        env: Expr,
        id: StructId,
        args: &[ast::Arg],
        span: Span,
        latebound: bool,
    ) -> Expr {
        let closure = self.closure_calls[&id];
        let def = closure.def;
        let signature: Vec<(Ty, Mode)> =
            self.signatures[def.0 as usize].params.iter().map(|(_, t, m, _)| (*t, *m)).collect();
        let ret = self.signatures[def.0 as usize].ret;
        let arity = signature.len().saturating_sub(1);
        if args.len() != arity {
            self.error(
                codes::E2020,
                span,
                format!("this closure takes {arity} arguments, found {}", args.len()),
            );
            return Expr { ty: ret, kind: ExprKind::Error, span };
        }
        let environment = match signature.first().map(|(_, mode)| *mode) {
            Some(Mode::Mut) => self.pass_receiver(env, Mode::Mut, span),
            _ => env,
        };
        let mut call_args = vec![environment];
        for (arg, &(param_ty, mode)) in args.iter().zip(signature.iter().skip(1)) {
            call_args.push(self.check_argument(&arg.value, param_ty, mode));
        }
        Expr {
            ty: ret,
            kind: ExprKind::Call {
                callee: def,
                arg_eval_order: None,
                args: call_args,
                latebound,
            },
            span,
        }
    }

    /// The signature an opaque generic parameter may be called with, read from
    /// the generics of the body being checked. `None` for a parameter that
    /// carries no `[CLO-3]` bound, which is then not callable at all.
    fn callable_bound_of(&mut self, ty: Ty) -> Option<(Ty, bool)> {
        let TyKind::Param { index, .. } = *self.types.kind(ty) else { return None };
        let bound = self.current_generics.get(index as usize)?.callable.clone()?;
        let signature = self.types.intern(TyKind::Fn {
            latebound: bound.latebound,
            params: bound.params,
            ret: bound.ret,
        });
        Some((signature, bound.once))
    }

    /// Infer through an implicit `[CLO-3]` callable bound as well as through
    /// the parameter that carries the callable value.  The latter is a fresh,
    /// hidden generic parameter, so unifying only it learns the concrete
    /// function value but misses facts in the bound's signature such as the
    /// result `R` in `f: fn(A) -> R`.  `with_views` is the adversarial form:
    /// every input can solve `A`/`B`, while `R` appears only in the callback.
    ///
    /// `[FN-6a]` makes the whole mode-bearing `Fn` signature canonical here;
    /// `TypeTable::unify_with_fixed` rejects a mismatched mode vector rather
    /// than treating `mut` or `owned` as annotation that inference may drop.
    fn unify_generic_argument(
        &mut self,
        declared: Ty,
        actual: Ty,
        generics: &[GenericParam],
        solved: &mut Vec<Option<Ty>>,
        fixed: usize,
    ) -> bool {
        if !self.types.unify_with_fixed(declared, actual, solved, fixed) {
            return false;
        }
        let TyKind::Param { index, .. } = *self.types.kind(declared) else {
            return true;
        };
        let Some(bound) = generics.get(index as usize).and_then(|param| param.callable.clone())
        else {
            return true;
        };
        let actual_signature = match self.types.kind(actual).clone() {
            TyKind::Fn { params, ret, .. } => Some((params, ret)),
            // A concrete capturing closure's value is its environment, not a
            // `fn` pointer. Its generated call body nevertheless has the
            // same canonical callable signature after the environment
            // receiver, and that is the fact generic inference needs.
            TyKind::Struct(id) => self.closure_calls.get(&id).map(|closure| {
                let signature = &self.signatures[closure.def.0 as usize];
                let params = signature
                    .params
                    .iter()
                    .skip(1)
                    .map(|(_, ty, mode, _)| FnParam {
                        ty: *ty,
                        mode: fn_param_mode_from_hir(*mode),
                    })
                    .collect();
                (params, signature.ret)
            }),
            _ => None,
        };
        let Some((actual_params, actual_ret)) = actual_signature else {
            return true;
        };
        if bound.params.len() != actual_params.len() {
            return false;
        }
        // Still propagate the type facts when the modes disagree. The caller
        // emits the mode error, but withholding an otherwise evident callback
        // result would add a misleading secondary "cannot tell what R is"
        // diagnostic to the one source mistake.
        let modes_match = bound
            .params
            .iter()
            .zip(&actual_params)
            .all(|(expected, actual)| expected.mode == actual.mode);
        let types_match = bound
            .params
            .iter()
            .zip(&actual_params)
            .all(|(expected, actual)| {
                self.types.unify_with_fixed(expected.ty, actual.ty, solved, fixed)
            })
            && self.types.unify_with_fixed(bound.ret, actual_ret, solved, fixed);
        modes_match && types_match
    }

    /// The signature a `fn(A) -> R` parameter may be called with, when that
    /// parameter is one of `generics` and carries `[CLO-3]`'s bound.
    fn callable_hint(
        &mut self,
        param_ty: Ty,
        generics: &[GenericParam],
        solved: &[Option<Ty>],
    ) -> Option<Ty> {
        let TyKind::Param { index, .. } = *self.types.kind(param_ty) else { return None };
        let bound = generics.get(index as usize)?.callable.clone()?;
        // Earlier arguments may already have solved parameters used by the
        // callable signature. Feed those facts into the lambda expectation so
        // `fn apply[T](value: T, f: fn(T) -> T)` checks `f` against the actual
        // `T`, rather than emitting a closure body over an opaque placeholder.
        let substitution = generics
            .iter()
            .enumerate()
            .map(|(slot, param)| {
                solved.get(slot).and_then(|ty| *ty).unwrap_or_else(|| {
                    self.types.intern(TyKind::Param {
                        index: slot as u32,
                        name: param.name,
                    })
                })
            })
            .collect::<Vec<_>>();
        let params = bound
            .params
            .into_iter()
            .map(|param| FnParam {
                ty: self.substitute_ty(param.ty, &substitution),
                mode: param.mode,
            })
            .collect();
        let ret = self.substitute_ty(bound.ret, &substitution);
        Some(self.types.intern(TyKind::Fn {
            latebound: bound.latebound,
            params,
            ret,
        }))
    }

    /// Synthesise an expression that may want to know what is expected of it.
    ///
    /// This is not `check_expr`: the hint shapes the expression without
    /// constraining the result, which is what a lambda needs. A lambda takes
    /// its parameter types from the hint and then reports **its own** type,
    /// which for a capturing one is its environment and not the hint at all.
    fn synth_with_hint(&mut self, expr: &ast::Expr, hint: Ty) -> Expr {
        match &expr.kind {
            ast::ExprKind::Lambda(lambda) => self.synth_lambda(lambda, Some(hint), expr.span),
            _ => self.synth_committed(expr),
        }
    }

    /// `[IFC-4]` — replace every associated type in `ty` with what `owner`
    /// declared it to be. A name the owner never declared is left alone; the
    /// implementation check reports that separately.
    fn resolve_assoc(&mut self, ty: Ty, owner: Ty) -> Ty {
        match self.types.kind(ty).clone() {
            TyKind::Assoc { name } => {
                self.assoc_values.get(&(owner, name)).copied().unwrap_or(ty)
            }
            TyKind::Ref { mutable, inner } => {
                let inner = self.resolve_assoc(inner, owner);
                self.types.intern(TyKind::Ref { mutable, inner })
            }
            TyKind::Ptr { mutable, inner } => {
                let inner = self.resolve_assoc(inner, owner);
                self.types.intern(TyKind::Ptr { mutable, inner })
            }
            TyKind::Vec { elem } => {
                let elem = self.resolve_assoc(elem, owner);
                self.types.intern(TyKind::Vec { elem })
            }
            TyKind::Array { elem, len } => {
                let elem = self.resolve_assoc(elem, owner);
                self.types.intern(TyKind::Array { elem, len })
            }
            TyKind::Tuple(items) => {
                let items: Vec<Ty> =
                    items.iter().map(|&t| self.resolve_assoc(t, owner)).collect();
                self.types.intern(TyKind::Tuple(items))
            }
            // A synthesised `Option[Self.Item]` is a distinct enum per `Item`,
            // so it has to be rebuilt rather than patched.
            TyKind::Enum(id) => {
                let def = self.types.enum_def(id);
                if !def.name.as_str().starts_with("Option_") {
                    return ty;
                }
                let payload = def.variants[1].fields[0].ty;
                let resolved = self.resolve_assoc(payload, owner);
                if resolved == payload {
                    return ty;
                }
                self.option_of(resolved)
            }
            _ => ty,
        }
    }

    /// Whether a type implements an interface, for `[TYP-17]`'s bound check.
    fn implements(&self, ty: Ty, interface: Symbol) -> bool {
        if let TyKind::Param { index, .. } = *self.types.kind(ty) {
            return self
                .current_generics
                .get(index as usize)
                .is_some_and(|param| param.bounds.contains(&interface));
        }
        // `[ENM-3]`, `[HASH-4]` — scalar/range/unit-enum equality and
        // hashing are compiler-known standard capabilities. They must satisfy
        // ordinary generic bounds too; accepting `ArenaMap[i32, V]` directly
        // while rejecting the same type through `K: Eq + Hash` would make
        // generic substitution change the language contract.
        if self.has_builtin_eq_hash(ty)
            && matches!(interface.as_str(), "std.core.Eq" | "std.collections.Hash")
        {
            return true;
        }
        if self.implemented.iter().any(|(t, i, _)| *t == ty && *i == interface) {
            return true;
        }
        // `[ARN-5c]`, `[ARN-5d]`, `[SPN-4]` — the compiler-lowered, named
        // Arena-backed and Span iterators implement the one standard
        // associated-type Iterator contract. Their `next` operations are
        // synthesized directly so they can preserve view provenance in
        // HIR/MIR; that temporary implementation detail must not make the
        // ordinary library types fail an `I: Iterator` bound.
        if interface.as_str() == "std.core.Iterator"
            && (self.span_iterator(ty).is_some()
                || self.arena_array_iterator(ty).is_some()
                || self.arena_map_iterator(ty).is_some())
        {
            return true;
        }
        // A bound naming an interface nothing declares cannot be satisfied;
        // the declaration site already reported that.
        !self.interfaces.contains_key(&interface)
    }

    /// Find the concrete associated member that supplies one standard
    /// interface capability. Compiler-known APIs use the fully qualified
    /// interface identity so their behavior cannot depend on which short
    /// names happen to be imported at the call site.
    fn standard_associated_capability(
        &self,
        ty: Ty,
        interface: &str,
        member: &str,
    ) -> Option<DefId> {
        let interface = Symbol::intern(interface);
        self.implemented
            .iter()
            .any(|(candidate, implemented, _)| {
                *candidate == ty && *implemented == interface
            })
            .then(|| self.associated.get(&(ty, Symbol::intern(member))))
            .flatten()
            .map(|entry| entry.def)
    }

    /// `[MONO-1]` — one `DefId` per (function, type arguments), created once
    /// and named deterministically.
    fn instantiate(
        &mut self,
        def: DefId,
        args: &[Ty],
        callable_values: Vec<Option<DefId>>,
        name: Symbol,
        span: Span,
    ) -> DefId {
        let key = Instance { def, args: args.to_vec(), callable_values };
        if let Some(&existing) = self.instances.get(&key) {
            return existing;
        }
        let generic = &self.signatures[def.0 as usize];
        let params: Vec<(Symbol, Ty, Mode, Span)> = generic
            .params
            .iter()
            .map(|(n, t, m, s)| (*n, *t, *m, *s))
            .collect();
        let ret = generic.ret;
        let concrete_params = params
            .into_iter()
            .map(|(n, t, m, s)| (n, self.substitute_ty(t, args), m, s))
            .collect();
        let concrete_ret = self.substitute_ty(ret, args);

        let instance = DefId(self.signatures.len() as u32);
        let borrows = self.signatures[def.0 as usize].borrows.clone();
        self.signatures.push(Signature {
            params: concrete_params,
            ret: concrete_ret,
            generics: Vec::new(),
            borrows,
        });
        self.instances.insert(key.clone(), instance);
        self.pending.push((key, instance));
        let _ = (name, span);
        instance
    }

    /// Generic methods use the same `(definition, type arguments)` identity as
    /// generic functions, but their bodies must be checked with a concrete
    /// receiver and emitted under a receiver-qualified symbol.
    fn instantiate_method(
        &mut self,
        def: DefId,
        args: &[Ty],
        name: Symbol,
        span: Span,
    ) -> DefId {
        let key = Instance { def, args: args.to_vec(), callable_values: Vec::new() };
        if let Some(&existing) = self.instances.get(&key) {
            return existing;
        }
        let generic = &self.signatures[def.0 as usize];
        let params = generic
            .params
            .iter()
            .map(|(param_name, ty, mode, param_span)| {
                (*param_name, *ty, *mode, *param_span)
            })
            .collect::<Vec<_>>();
        let ret = generic.ret;
        let borrows = generic.borrows.clone();
        let concrete_params = params
            .into_iter()
            .map(|(param_name, ty, mode, param_span)| {
                (param_name, self.substitute_ty(ty, args), mode, param_span)
            })
            .collect();
        let concrete_ret = self.substitute_ty(ret, args);
        let instance = DefId(self.signatures.len() as u32);
        self.signatures.push(Signature {
            params: concrete_params,
            ret: concrete_ret,
            generics: Vec::new(),
            borrows,
        });
        self.instances.insert(key.clone(), instance);
        self.pending_generic_methods.push((key, instance));
        let _ = (name, span);
        instance
    }

    /// The module an expression names, if it is a bare path bound by
    /// `import a.b.c`. A local of the same name wins.
    fn namespace_named(&self, expr: &ast::Expr) -> Option<usize> {
        let ast::ExprKind::Path { segments } = &expr.kind else { return None };
        if segments.len() != 1 || self.lookup(segments[0].name).is_some() {
            return None;
        }
        self.namespaces[self.current_module].get(&segments[0].name).copied()
    }

    /// A call to a function named by its qualified name, which is what a
    /// namespace-qualified call resolves to.
    fn synth_qualified_call(
        &mut self,
        qualified: Symbol,
        name_span: Span,
        args: &[ast::Arg],
        explicit: Vec<Ty>,
        span: Span,
    ) -> Expr {
        // Module-qualified calls (`import std.mem; mem.take(...)`) are parsed
        // as method calls on a namespace and therefore do not pass through
        // `synth_call`. Give them the same compiler-known memory-operation
        // resolution as an unqualified imported name before instantiating the
        // source-backed public declaration.
        if let Some(built) = self.synth_memory_builtin(qualified, args, &explicit, span) {
            return built;
        }
        let Some(&def) = self.fn_ids.get(&qualified) else {
            self.error(codes::E1010, name_span, format!("cannot find `{qualified}`"));
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };
        if !self.signatures[def.0 as usize].generics.is_empty() {
            return self.synth_generic_call(def, qualified, args, explicit, span);
        }
        if !explicit.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`{qualified}` takes no type arguments, found {}", explicit.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let signature: Vec<(Ty, Mode)> =
            self.signatures[def.0 as usize].params.iter().map(|(_, t, m, _)| (*t, *m)).collect();
        let ret = self.signatures[def.0 as usize].ret;
        if args.len() != signature.len() {
            self.error(
                codes::E2020,
                span,
                format!("`{qualified}` takes {} arguments, found {}", signature.len(), args.len()),
            );
        }
        let params = self.signatures[def.0 as usize].params.clone();
        let slots = self.call_argument_slots(qualified, args, &params);
        let checked = self.check_bound_call_arguments(args, &params, &slots);
        Expr {
            ty: ret,
            kind: ExprKind::Call {
                callee: def,
                arg_eval_order: Self::call_eval_order(&slots),
                args: checked,
                latebound: false,
            },
            span,
        }
    }

    /// The enum an expression names, if it is a bare path naming one. A local
    /// of the same name wins, so shadowing behaves as it does everywhere else.
    fn enum_named(&self, expr: &ast::Expr) -> Option<EnumId> {
        let ast::ExprKind::Path { segments } = &expr.kind else { return None };
        if segments.len() != 1 {
            return None;
        }
        let name = segments[0].name;
        if self.lookup(name).is_some() {
            return None;
        }
        self.enum_ids.get(&self.resolve_name(name)).copied()
    }

    /// A path naming a range type, for `Roughness.checked(x)`.
    fn range_named(&self, expr: &ast::Expr) -> Option<ember_types::RangeId> {
        let ast::ExprKind::Path { segments } = &expr.kind else { return None };
        if segments.len() != 1 {
            return None;
        }
        let name = segments[0].name;
        if self.lookup(name).is_some() {
            return None;
        }
        let ty = *self.named_types.get(&self.resolve_name(name))?;
        match self.types.kind(ty) {
            TyKind::Range(id) => Some(*id),
            _ => None,
        }
    }

    /// A type named on the left of `Type.function(...)`. Name resolution must
    /// make this distinction before ordinary method synthesis, because a type
    /// is not a runtime receiver value. A local shadows a type exactly as it
    /// does for enum and range associated operations.
    fn associated_type_named(&mut self, expr: &ast::Expr) -> Option<Ty> {
        match &expr.kind {
            ast::ExprKind::Path { segments } if segments.len() == 1 => {
                let name = segments[0].name;
                if self.lookup(name).is_some() {
                    return None;
                }
                self.type_params
                    .get(&name)
                    .copied()
                    .or_else(|| self.scalar_named(name.as_str()))
                    .or_else(|| self.named_types.get(&self.resolve_name(name)).copied())
            }
            ast::ExprKind::IndexOrInstantiate { base, .. } => {
                let ast::ExprKind::Path { segments } = &base.kind else { return None };
                if segments.len() != 1 || self.lookup(segments[0].name).is_some() {
                    return None;
                }
                let name = self.resolve_name(segments[0].name);
                if !self.generic_structs.contains_key(&name) {
                    return None;
                }
                Some(self.type_from_expr(expr))
            }
            _ => None,
        }
    }

    fn synth_associated_call(
        &mut self,
        owner: Ty,
        name: ast::Ident,
        generic_args: &[ast::GenericArg],
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        if let Some(elem) = self.arena_array_element(owner) {
            return self.synth_arena_array_construction(
                owner,
                elem,
                name,
                generic_args,
                args,
                span,
            );
        }
        if let Some((key, value)) = self.arena_map_parts(owner) {
            return self.synth_arena_map_construction(
                owner,
                key,
                value,
                name,
                generic_args,
                args,
                span,
            );
        }
        if let TyKind::Param { index, name: param } = *self.types.kind(owner) {
            return self.synth_bound_associated(
                index,
                param,
                owner,
                name,
                generic_args,
                args,
                span,
            );
        }
        let Some(entry) = self.associated.get(&(owner, name.name)) else {
            let shown = self.types.display(owner);
            self.error(
                codes::E2020,
                span,
                format!("`{shown}` has no associated function `{}`", name.name),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };
        let def = entry.def;
        let explicit = self.resolve_method_type_args(generic_args);
        if !self.signatures[def.0 as usize].generics.is_empty() {
            return self.synth_generic_method_call(
                def,
                name.name,
                None,
                false,
                args,
                explicit,
                span,
                true,
            );
        }
        if !explicit.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!(
                    "`{}.{}` takes no type arguments, found {}",
                    self.types.display(owner),
                    name.name,
                    explicit.len()
                ),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let signature = self.signatures[def.0 as usize]
            .params
            .iter()
            .map(|(_, ty, mode, _)| (*ty, *mode))
            .collect::<Vec<_>>();
        let ret = self.signatures[def.0 as usize].ret;
        if args.len() != signature.len() {
            self.error(
                codes::E2020,
                span,
                format!(
                    "`{}.{}` takes {} arguments, found {}",
                    self.types.display(owner),
                    name.name,
                    signature.len(),
                    args.len()
                ),
            );
        }
        let params = self.signatures[def.0 as usize].params.clone();
        let params = params
            .into_iter()
            .map(|(param, ty, mode, param_span)| {
                (param, self.types.substitute_self(ty, owner), mode, param_span)
            })
            .collect::<Vec<_>>();
        let slots = self.call_argument_slots(name.name, args, &params);
        let checked = self.check_bound_call_arguments(args, &params, &slots);
        Expr {
            ty: ret,
            kind: ExprKind::Call {
                callee: def,
                arg_eval_order: Self::call_eval_order(&slots),
                args: checked,
                latebound: false,
            },
            span,
        }
    }

    /// Resolve `T.make(...)` inside a generic body from `T`'s declared
    /// interface bounds. The opaque validation call targets the interface
    /// declaration only while checking the generic recipe; each concrete
    /// monomorphisation is rechecked and calls its actual implementation.
    fn synth_bound_associated(
        &mut self,
        index: u32,
        param: Symbol,
        owner: Ty,
        name: ast::Ident,
        generic_args: &[ast::GenericArg],
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let bounds = self
            .current_generics
            .get(index as usize)
            .map(|generic| generic.bounds.clone())
            .unwrap_or_default();
        let mut found: Option<(DefId, Symbol)> = None;
        for bound in &bounds {
            let Some(interface) = self.interfaces.get(bound) else { continue };
            let member = interface.methods.iter().find(|(member, _, receiver, _)| {
                *member == name.name && receiver.is_none()
            });
            let Some((_, def, _, _)) = member else { continue };
            if let Some((_, first)) = found {
                self.error(
                    codes::E2070,
                    name.span,
                    format!("`{}` is offered by both `{first}` and `{bound}`", name.name),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            found = Some((*def, *bound));
        }
        let Some((def, _)) = found else {
            let candidate = self.interfaces.iter().find_map(|(interface_name, interface)| {
                interface
                    .methods
                    .iter()
                    .any(|(member, _, receiver, _)| {
                        *member == name.name && receiver.is_none()
                    })
                    .then_some(*interface_name)
            });
            let mut diagnostic = Diagnostic::error(
                codes::E2040,
                name.span,
                format!(
                    "`{param}` has no associated function `{}`; its bounds do not provide one",
                    name.name
                ),
            )
            .note("inside a generic body only the bounds' operations are available [TYP-17]");
            if let Some(bound) = candidate {
                diagnostic = diagnostic.help(format!("add the bound: `{param}: {bound}`"));
            }
            self.sink.emit(diagnostic);
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };
        let explicit = self.resolve_method_type_args(generic_args);
        if !self.signatures[def.0 as usize].generics.is_empty() {
            return self.synth_generic_method_call(
                def,
                name.name,
                None,
                false,
                args,
                explicit,
                span,
                false,
            );
        }
        if !explicit.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let signature = self.signatures[def.0 as usize]
            .params
            .iter()
            .map(|(_, ty, mode, _)| (self.types.substitute_self(*ty, owner), *mode))
            .collect::<Vec<_>>();
        let ret = self.types.substitute_self(self.signatures[def.0 as usize].ret, owner);
        if args.len() != signature.len() {
            self.error(
                codes::E2020,
                span,
                format!("`{}.{}` takes {} arguments, found {}", param, name.name, signature.len(), args.len()),
            );
        }
        let params = self.signatures[def.0 as usize].params.clone();
        let params = params
            .into_iter()
            .map(|(param, ty, mode, param_span)| {
                (param, self.types.substitute_self(ty, owner), mode, param_span)
            })
            .collect::<Vec<_>>();
        let slots = self.call_argument_slots(name.name, args, &params);
        let checked = self.check_bound_call_arguments(args, &params, &slots);
        Expr {
            ty: ret,
            kind: ExprKind::Call {
                callee: def,
                arg_eval_order: Self::call_eval_order(&slots),
                args: checked,
                latebound: false,
            },
            span,
        }
    }

    /// `[MOD-2]` — **the one place a field's visibility is checked.**
    ///
    /// "All items are private to their module unless `pub`." Every field
    /// access resolves the field and then comes here, in that order, so a
    /// reachable field access that skipped the check does not exist. It was
    /// unenforced for the whole of Phase 1, and the way that was discovered is
    /// worth keeping in mind: a test in `tests/run-pass/` was passing *because*
    /// of the hole, and adding the check turned it red. The fixture was the
    /// thing that was wrong.
    ///
    /// `pub(package)` reads as public here: a build is one package until
    /// `[MAN-2]`'s dependency graph exists, so there is no other package for it
    /// to be invisible to yet. When there is, this function is where that
    /// changes — not the call sites.
    ///
    /// Writing is separate and stricter: `pub(read)` permits the read this
    /// checks and refuses the write, which is `reject_readonly_write`, and
    /// constructing writes every field at once, which is
    /// `check_memberwise_constructor` under `[STR-1]`.
    fn check_field_visible(
        &mut self,
        id: StructId,
        vis: FieldVis,
        field: Symbol,
        span: Span,
    ) {
        if vis != FieldVis::Private {
            return;
        }
        let def = self.types.struct_def(id);
        if def.declaring_module == self.current_module {
            return;
        }
        let owner = def.name.to_string();
        self.sink.emit(
            Diagnostic::error(
                codes::E1020,
                span,
                format!("`{field}` is private to `{owner}`'s module"),
            )
            .help(format!(
                "declare it `pub {field}: …` to read it anywhere,                  or `pub(read) {field}: …` to make it readable and not writable"
            ))
            .note("a field is private unless it says otherwise [MOD-2]"),
        );
    }

    fn check_class_field_visible(
        &mut self,
        id: ClassId,
        vis: FieldVis,
        field: Symbol,
        span: Span,
    ) {
        if vis != FieldVis::Private {
            return;
        }
        let def = self.types.class_def(id);
        if def.declaring_module == self.current_module {
            return;
        }
        let owner = def.name.to_string();
        self.sink.emit(
            Diagnostic::error(
                codes::E1020,
                span,
                format!("`{field}` is private to `{owner}`'s module"),
            )
            .help(format!(
                "declare it `pub {field}: …` to read it anywhere, or `pub(read) {field}: …` to make it readable and not writable"
            ))
            .note("a field is private unless it says otherwise [MOD-2]"),
        );
    }

    /// `[STR-1]` — whether the memberwise constructor may be called here.
    ///
    /// "It is `pub` iff all fields are `pub` (a `pub(read)` field makes it
    /// private to the declaring module, since construction is a write)."
    fn check_memberwise_constructor(&mut self, id: StructId, span: Span) {
        let def = self.types.struct_def(id);
        if def.declaring_module == self.current_module {
            return;
        }
        let owner = def.name.to_string();
        let blocking = def
            .fields
            .iter()
            .find(|f| f.vis != FieldVis::Public || f.read_only_outside)
            .map(|f| (f.name.to_string(), f.vis, f.read_only_outside));
        let Some((field, vis, read_only)) = blocking else { return };
        let why = if read_only {
            format!("`{field}` is `pub(read)`, and construction is a write")
        } else if vis == FieldVis::Package {
            format!("`{field}` is `pub(package)`, not `pub`")
        } else {
            format!("`{field}` is private")
        };
        self.sink.emit(
            Diagnostic::error(
                codes::E1020,
                span,
                format!("`{owner}`'s memberwise constructor is private to its module"),
            )
            .primary_label(why)
            .help(format!(
                "call a function `{owner}`'s module exports, or make every field `pub`"
            ))
            .note(concat!(
                "the memberwise constructor is `pub` only when every field is, ",
                "because constructing writes them all [STR-1]"
            )),
        );
    }

    /// `[FN-1]` — what a `mut` parameter is inside the callee.
    ///
    /// Normally `ref mut T`: the mode is an inout borrow, so every mention of
    /// the name reads through it and the caller sees the writes.
    ///
    /// A `MutSpan[T]` is the owner-approved `[FN-1a]` exception: the view value
    /// already carries the mutable access and `[SPN-3]` makes it move-only, so
    /// another `ref mut` would be a reference to a reference. The mutable-place
    /// requirement applies to the storage from which the view was derived.
    /// ERR-041 and ADR-017 preserve the earlier ambiguity and its resolution;
    /// D5 is closed and `tests/conformance/FN-1a/` pins the ruling.
    fn mut_param_ty(&mut self, ty: Ty) -> Ty {
        if matches!(self.types.kind(ty), TyKind::Span { mutable: true, .. }) {
            return ty;
        }
        self.types.intern(TyKind::Ref { mutable: true, inner: ty })
    }

    /// Whether a checked expression denotes a place, seeing through the view
    /// `[SPN-1]`'s coercion may have wrapped it in.
    fn viewed_place(&self, expr: &Expr) -> bool {
        match &expr.kind {
            // `[SPN-1]`'s coercion wraps the container in the borrow it
            // takes of it, so the place is two levels down.
            ExprKind::Builtin { which: Builtin::SpanFrom { .. }, args } => {
                match args.first().map(|a| &a.kind) {
                    Some(ExprKind::Ref { place, .. }) => is_place(&place.kind),
                    Some(other) => is_place(other),
                    None => false,
                }
            }
            other => is_place(other),
        }
    }

    /// `[MOD-7]` — a `pub(read)` field is writable only from the declaring
    /// module.
    ///
    /// "Outside the declaring module, the following are errors `E1050`:
    /// assignment (`h.value = x`, augmented assignment), taking
    /// `ref mut h.value`, passing `h.value` to a `mut` parameter or `mut self`
    /// method, and destructuring it with a mutable binding."
    ///
    /// The whole projection chain is walked, not just its last step: writing
    /// `h.value.inner` takes a mutable borrow of `h.value`, which the rule
    /// names.
    /// `[BRW-1]` — "while shared borrows are live the owner may read (and
    /// copy) but **not write**". A write whose destination is reached through
    /// a shared `ref` is that forbidden write.
    ///
    /// ADR-010 settled that a reference local is not re-seatable: `m = ref mut
    /// y` writes *through* `m` rather than pointing it somewhere new, which is
    /// what lets regions be location-insensitive. That is sound for `ref mut`
    /// and is the whole of `[BRW-1]`'s prohibition for a shared `ref` — and
    /// the shared case was not checked. `r: ref i32 = ref x` then `r = 99`
    /// passed every Ember check, emitted `(*_2) = 99`, and was caught by
    /// **clang** rather than by the compiler:
    ///
    /// ```text
    /// error: read-only variable is not assignable
    /// ```
    ///
    /// So aliasing-XOR-mutability was being upheld by the backend's `const`
    /// rather than by the language. That is luck, not a rule: a backend that
    /// did not emit `const` would have mutated through a shared borrow in
    /// silence. It is rejected here, where the type alone answers the
    /// question and no flow analysis is needed.
    fn reject_write_through_shared_ref(&mut self, place: &Expr, span: Span) -> bool {
        let mut current = place;
        loop {
            if let TyKind::Ref { mutable: false, .. } = *self.types.kind(current.ty) {
                let shown = self.types.display(current.ty);
                self.sink.emit(
                    Diagnostic::error(
                        codes::E3021,
                        span,
                        "cannot write through a shared reference",
                    )
                    .primary_label(format!("this is `{shown}`, which only reads"))
                    .help(concat!(
                        "take the borrow as `ref mut` if the callee must write, ",
                        "or assign to the place itself rather than through the reference"
                    ))
                    .note(concat!(
                        "while a shared borrow is live the owner may read and copy ",
                        "but not write [BRW-1]"
                    )),
                );
                return true;
            }
            match &current.kind {
                // A path to a reference local reads as `Deref(Local)` with the
                // *pointee's* type, so the reference is one node further in
                // and the walk has to reach it — checking the target's own
                // type alone sees `i32` and says nothing.
                ExprKind::Field { base, .. }
                | ExprKind::Index { base, .. }
                | ExprKind::Deref(base) => current = base,
                _ => return false,
            }
        }
    }

    /// `[FN-1]`, `[BRW-1]`, `[CELL-10]` — a default-mode parameter is a
    /// shared borrow, even when its ABI is a small by-value copy. A write
    /// rooted at that parameter is shape B4: the caller remains the owner and
    /// the callee is asking for a second writer at a different program point.
    ///
    /// `offer_interior_mutability` is true only when this site itself proves
    /// that the accesses are not simultaneous in one expression. Calls can
    /// contain several `mut` arguments, so their structural diagnostic must
    /// not suggest `RefCell` until the whole call has established that fact.
    fn reject_borrowed_parameter_write(
        &mut self,
        place: &Expr,
        span: Span,
        offer_interior_mutability: bool,
    ) -> bool {
        let Some(local) = root_local(&place.kind) else { return false };
        if !self.borrowed_params.contains(&local) {
            return false;
        }

        let decl = &self.locals[local.0 as usize];
        let name = decl.name.unwrap_or_else(|| Symbol::intern("parameter"));
        let name = name.to_string();
        let declaration = decl.span;
        let ty = decl.ty;
        let shown = self.types.display(ty);
        let structural_help = if self.callable_parameter_locals.contains(&local) {
            format!(
                "declare this callable parameter `mut {name}: fn(...) -> ...` so it receives a mutable closure place"
            )
        } else {
            format!(
                "restructure to a single owner and declare this parameter `mut {name}: {shown}`"
            )
        };
        let mut diagnostic = Diagnostic::error(
            codes::E3023,
            span,
            format!("cannot mutate borrowed parameter `{name}`"),
        )
        .primary_label("this write needs mutable access")
        .secondary(declaration, format!("`{name}` is borrowed here"))
        .help(structural_help);

        if offer_interior_mutability {
            diagnostic = if self.types.is_copy(ty) {
                diagnostic.help(
                    "if the value is genuinely shared, use `Cell[T]` for the `Copy` payload or \
                     `RefCell[T]` for runtime-checked borrows",
                )
            } else {
                diagnostic.help(
                    "if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows",
                )
            };
            diagnostic = diagnostic
                .help("if the value has identity, make it a `class`")
                .note(concat!(
                    "`RefCell` adds a one-word runtime borrow-state check in every profile ",
                    "(CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, ",
                    "reference counting and dynamic exclusivity, and is not `@static_safe` ",
                    "(OBJ-1, RC-1, EXC-1, EFF-13)"
                ));
        }

        self.sink.emit_classified(diagnostic);
        true
    }

    fn reject_readonly_write(&mut self, place: &Expr, span: Span) {
        self.reject_readonly_write_inner(place, span, false, false);
    }

    /// `[EXC-1]` — a direct write of one scalar/`Copy` field through a class
    /// handle is instantaneous and needs no runtime access word. This is
    /// intentionally separate from `ref mut`, a `mut` argument, and a mutating
    /// method call, all of which are long-term accesses.
    fn reject_readonly_write_in_assignment(&mut self, place: &Expr, span: Span) {
        self.reject_readonly_write_inner(place, span, false, true);
    }

    /// `[EXC-1]` — a class field passed to a `mut` parameter is a permitted
    /// long-term access at the call boundary. It is not an ordinary direct
    /// assignment, so the caller's MIR lowering will surround the call with
    /// the runtime access interval.
    fn reject_readonly_write_in_mut_argument(&mut self, place: &Expr, span: Span) {
        self.reject_readonly_write_inner(place, span, true, false);
    }

    fn reject_readonly_write_inner(
        &mut self,
        place: &Expr,
        span: Span,
        allow_class_mut_argument: bool,
        allow_instantaneous_class_field: bool,
    ) {
        let mut current = place;
        loop {
            match &current.kind {
                ExprKind::Field { base, index } => {
                    if let TyKind::Class(id) = *self.types.kind(base.ty) {
                        if self.class_init_field_index(place).is_none()
                            && self.class_method_field_index(current).is_none()
                            && !(allow_class_mut_argument && self.class_mut_argument_supported(place))
                            && !(allow_instantaneous_class_field
                                && self.class_instantaneous_field(place))
                        {
                            self.error(
                                codes::E1010,
                                span,
                                "mutable class-field access requires a `mut self` class method in this phase",
                            );
                        }
                        if let Some((owner_id, field)) = self.types.class_field_at_info(id, *index) {
                            if field.read_only_outside
                                && self.types.class_def(owner_id).declaring_module
                                    != self.current_module
                            {
                                let owner = self.types.class_def(owner_id).name.to_string();
                                let field = field.name.to_string();
                                self.sink.emit(
                                    Diagnostic::error(
                                        codes::E1050,
                                        span,
                                        format!("`{owner}.{field}` is read-only outside its module"),
                                    )
                                    .primary_label("written here".to_string())
                                    .help(format!(
                                        "`{owner}` declares `{field}` as `pub(read)`: anyone may read it, and only `{owner}`'s own module may write it"
                                    ))
                                    .note(
                                        "a method on the declaring type is the way to change it from outside [MOD-7]",
                                    ),
                                );
                            }
                        }
                    } else if let TyKind::Struct(id) = *self.types.kind(base.ty) {
                        let def = self.types.struct_def(id);
                        if let Some(field) = def.fields.get(*index) {
                            if field.read_only_outside
                                && def.declaring_module != self.current_module
                            {
                                let owner = def.name.to_string();
                                let field = field.name.to_string();
                                self.sink.emit(
                                    Diagnostic::error(
                                        codes::E1050,
                                        span,
                                        format!("`{owner}.{field}` is read-only outside its module"),
                                    )
                                    .primary_label("written here".to_string())
                                    .help(format!(
                                        "`{owner}` declares `{field}` as `pub(read)`: anyone may \
                                         read it, and only `{owner}`'s own module may write it"
                                    ))
                                    .note(
                                        "a method on the declaring type is the way to change it \
                                         from outside [MOD-7]",
                                    ),
                                );
                            }
                        }
                    }
                    current = base;
                }
                ExprKind::Index { base, .. } | ExprKind::Deref(base) => current = base,
                _ => return,
            }
        }
    }

    fn class_instantaneous_field(&self, expr: &Expr) -> bool {
        let ExprKind::Field { base, .. } = &expr.kind else { return false };
        if !matches!(self.types.kind(base.ty), TyKind::Class(_)) || !self.types.is_copy(expr.ty) {
            return false;
        }
        match &base.kind {
            ExprKind::Local(_) => true,
            ExprKind::Deref(inner) => {
                matches!(inner.kind, ExprKind::Local(_))
                    && matches!(self.types.kind(inner.ty), TyKind::Ref { mutable: true, .. })
            }
            _ => false,
        }
    }

    /// `[RNG-3]`, `[RNG-3a]`, `[RNG-10]` — the three named constructors.
    ///
    /// `T.checked(v) -> Result[T, RangeError]` is the fallible form;
    /// `T.clamped(v) -> T` is total and introduces no `Panic` and no
    /// `RuntimeCheck(k)`; `unsafe T.new_unchecked(v) -> T` is the one route
    /// outside the closed set and carries `[RNG-9]`'s obligation.
    fn synth_range_construction(
        &mut self,
        id: ember_types::RangeId,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let def = self.types.range_def(id).clone();
        let range_ty = self.types.intern(TyKind::Range(id));
        let type_name = def.name.to_string();
        let error = |this: &mut Self, msg: String| -> Expr {
            this.error(codes::E2020, span, msg);
            Expr { ty: this.common.error, kind: ExprKind::Error, span }
        };

        let which = match name.name.as_str() {
            "checked" => 0,
            "clamped" => 1,
            "new_unchecked" => 2,
            other => {
                return error(
                    self,
                    format!(
                        "`{type_name}` has no `{other}`; a range type is built with \
                         `checked`, `clamped`, or `unsafe new_unchecked` [RNG-10]"
                    ),
                );
            }
        };

        if args.len() != 1 || args[0].name.is_some() {
            return error(
                self,
                format!("`{type_name}.{}` takes one value", name.name),
            );
        }

        // `[RNG-10]` — "Any other route is `unsafe`", so `new_unchecked`
        // needs the boundary `[TIER-1]` names.
        if which == 2 && !self.in_unsafe {
            self.sink.emit(
                Diagnostic::error(
                    codes::E3100,
                    span,
                    format!("`{type_name}.new_unchecked` requires `unsafe`"),
                )
                .help(format!(
                    "`{type_name}.clamped(x)` is total and needs no `unsafe`; \
                     `{type_name}.checked(x)` returns a `Result`"
                ))
                .note(
                    "a value of a range type outside its range is undefined behaviour, \
                     exactly as a `bool` that is not 0 or 1 is [RNG-9]",
                ),
            );
        }

        // The argument is a value of the representation. An untyped literal
        // adopts it; a range value of this same type erases to it.
        let value = self.check_expr(&args[0].value, def.repr);

        let builtin = match which {
            0 => hir::Builtin::RangeChecked(id),
            1 => hir::Builtin::RangeClamped(id),
            _ => hir::Builtin::RangeNewUnchecked(id),
        };
        let ty = if which == 0 {
            let err = self.range_error_ty();
            self.result_of(range_ty, err)
        } else {
            range_ty
        };
        Expr { ty, kind: ExprKind::Builtin { which: builtin, args: vec![value] }, span }
    }

    /// `RangeError`, the error half of `[RNG-3]`'s `Result`. A unit-only enum
    /// with one variant, so it is `Copy` and zero-cost; `std.core` declares it
    /// once the standard library can be written in Ember.
    fn range_error_ty(&mut self) -> Ty {
        let name = Symbol::intern("RangeError");
        self.builtin_enum(name, &[(Symbol::intern("OutOfRange"), vec![])])
    }

    /// `[ENM-1]` — a variant constructor, positional or by name. A unit
    /// variant is the same thing with no arguments.
    fn synth_variant(
        &mut self,
        id: EnumId,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let ty = self.types.intern(TyKind::Enum(id));
        let Some((index, variant)) = self.types.enum_def(id).variant(name.name) else {
            let enum_name = self.types.enum_def(id).name;
            self.error(
                codes::E1010,
                name.span,
                format!("`{enum_name}` has no variant `{}`", name.name),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };
        let field_names: Vec<Symbol> = variant.fields.iter().map(|f| f.name).collect();
        let field_types: Vec<Ty> = variant.fields.iter().map(|f| f.ty).collect();

        if args.len() != field_types.len() {
            let enum_name = self.types.enum_def(id).name;
            self.error(
                codes::E2020,
                span,
                format!(
                    "`{enum_name}.{}` takes {} payload values, found {}",
                    name.name,
                    field_types.len(),
                    args.len()
                ),
            );
            return Expr { ty, kind: ExprKind::Error, span };
        }

        // `[ENM-1]` allows `Shape.Circle(radius=1.0)`; a named argument goes
        // to its field wherever that field sits.
        let mut fields: Vec<Option<Expr>> = (0..field_types.len()).map(|_| None).collect();
        for (position, arg) in args.iter().enumerate() {
            let slot = match arg.name {
                Some(label) => match field_names.iter().position(|&f| f == label.name) {
                    Some(slot) => slot,
                    None => {
                        self.error(
                            codes::E1010,
                            label.span,
                            format!("`{}` has no payload named `{}`", name.name, label.name),
                        );
                        continue;
                    }
                },
                None => position,
            };
            let value = self.check_expr(&arg.value, field_types[slot]);
            if fields[slot].replace(value).is_some() {
                self.error(
                    codes::E1030,
                    arg.value.span,
                    format!("payload `{}` is given twice", field_names[slot]),
                );
            }
        }

        let fields = fields
            .into_iter()
            .enumerate()
            .map(|(slot, value)| match value {
                Some(value) => value,
                None => {
                    self.error(
                        codes::E2020,
                        span,
                        format!("payload `{}` is missing", field_names[slot]),
                    );
                    Expr { ty: field_types[slot], kind: ExprKind::Error, span }
                }
            })
            .collect();

        Expr { ty, kind: ExprKind::EnumLit { enum_id: id, variant: index, fields }, span }
    }

    fn class_ty(&self, id: ClassId) -> Option<Ty> {
        self.types.all().find_map(|(ty, kind)| {
            matches!(kind, TyKind::Class(candidate) if *candidate == id).then_some(ty)
        })
    }

    /// Find an inherent/interface method on a class or one of its bases.
    /// Dispatch remains a later Phase 3 consumer; this lookup only preserves
    /// the source-level fact that a derived handle sees inherited methods.
    fn lookup_method(&self, ty: Ty, name: Symbol) -> Option<MethodEntry> {
        if let Some(entry) = self.methods.get(&(ty, name)) {
            return Some(*entry);
        }
        let TyKind::Class(id) = *self.types.kind(ty) else { return None };
        let base = self.types.class_def(id).base?;
        let base_ty = self.class_ty(base)?;
        self.lookup_method(base_ty, name)
    }

    /// `[CLS-4]` — validate the one direct-base constructor call permitted in
    /// a derived `init`. The resulting HIR builtin is deliberately narrow:
    /// it carries the direct base constructor identity so MIR cannot turn
    /// this into general inherited dispatch.
    fn synth_super_init(
        &mut self,
        name: ast::Ident,
        generic_args: &[ast::GenericArg],
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = || Expr { ty: self.common.error, kind: ExprKind::Error, span };
        if !generic_args.is_empty() {
            self.error(codes::E2020, span, "`super.init` takes no type arguments");
            return error();
        }
        if !name.name.is("init") {
            self.error(codes::E1010, name.span, "`super` has no constructor other than `init`");
            return error();
        }
        let Some(state) = self.class_init.as_ref().cloned() else {
            self.error(
                codes::E1010,
                span,
                "`super.init(...)` is only valid in a supported derived class constructor",
            );
            return error();
        };
        let Some(base_id) = self.types.class_def(state.owner).base else {
            self.error(codes::E1010, span, "`super.init(...)` requires a base class");
            return error();
        };
        if state.base_initialized {
            self.error(codes::E1010, span, "`super.init(...)` may be called only once");
            return error();
        }
        let Some(base_ty) = self.class_ty(base_id) else {
            self.error(codes::E1010, span, "the direct base class type is unavailable");
            return error();
        };
        let Some(entry) = self.methods.get(&(base_ty, Symbol::intern("init"))).copied() else {
            self.error(codes::E1010, span, "the direct base class has no explicit `init`");
            return error();
        };
        let signature = self.signatures[entry.def.0 as usize].params.clone();
        if signature.first().is_none_or(|(_, _, mode, _)| *mode != Mode::Mut) {
            self.error(codes::E1010, span, "the direct base constructor must declare `mut self`");
            return error();
        }
        let params = &signature[1..];
        if args.len() != params.len() {
            self.error(
                codes::E2020,
                span,
                format!("`super.init(...)` takes {} arguments, found {}", params.len(), args.len()),
            );
            return error();
        }
        let values = args
            .iter()
            .zip(params.iter())
            .map(|(arg, (_, param_ty, mode, _))| self.check_argument(&arg.value, *param_ty, *mode))
            .collect();
        self.mark_class_init_base(span);
        Expr {
            ty: self.common.void,
            kind: ExprKind::Builtin {
                which: Builtin::ClassSuperInit { base_id, base_ty, init: entry.def },
                args: values,
            },
            span,
        }
    }

    /// Part IV.11 — `recv.m(args)`. The receiver's type decides which method
    /// runs; `[TYP-24]` prefers an inherent method over an interface one, and
    /// two interfaces offering the name is `E2070`.
    fn synth_method_call(
        &mut self,
        recv: &ast::Expr,
        name: ast::Ident,
        generic_args: &[ast::GenericArg],
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let receiver = self.synth_committed(recv);
        if receiver.ty == self.common.error {
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        // `[CELL-7]` — a guard reads through to its contents, so a method on
        // `T` is found through `Ref[T]`/`RefMut[T]`. Unwrap first; the
        // unwrapped receiver derefs through the guard's field, keeping the
        // guard alive for the borrow checker and the region graph.
        // Guards hold `ref`s (never guards: a view may not be put in a cell),
        // so one step suffices.
        let mut receiver = match self.ref_guard_inner(receiver.ty) {
            Some(_) => self.read_guard_through(receiver),
            None => receiver,
        };
        let explicit = self.resolve_method_type_args(generic_args);
        // IX.1 — `get` exposes the canonical shared reference. Other method
        // names are resolved after auto-dereferencing to the payload, so Box
        // does not duplicate the payload's method surface.
        if self.box_inner(receiver.ty).is_some() {
            if name.name.is("get") {
                if !explicit.is_empty() {
                    self.error(
                        codes::E2020,
                        span,
                        format!("`get` takes no type arguments, found {}", explicit.len()),
                    );
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                }
                return self.synth_box_get(receiver, name, args, span);
            }
            while self.box_inner(receiver.ty).is_some() {
                receiver = self.read_box_through(receiver);
            }
        }
        // `Array` and `String` carry their methods in the compiler until
        // Phase 2's generics let the standard library declare them.
        if let TyKind::Vec { elem } = *self.types.kind(receiver.ty) {
            if !explicit.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_vec_method(receiver, elem, name, args, span);
        }
        if let TyKind::Span { elem, mutable } = *self.types.kind(receiver.ty) {
            if !explicit.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_span_method(receiver, elem, mutable, name, args, span);
        }
        // `[CELL-1]` — a cell is an ordinary struct to every other part of the
        // compiler, so its methods are found here rather than in `self.methods`.
        if let Some(inner) = self.cell_inner(receiver.ty) {
            if !explicit.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_cell_method(receiver, inner, name, args, span);
        }
        // `[CELL-5]` — a `RefCell` is likewise an ordinary struct here; its
        // `borrow` family are builtins (ADR-019).
        if let Some(inner) = self.refcell_inner(receiver.ty) {
            if !explicit.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_refcell_method(receiver, inner, name, args, span);
        }
        if let Some(inner) = self.maybe_uninit_inner(receiver.ty) {
            if !explicit.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_maybe_uninit_method(receiver, inner, name, args, span);
        }
        if let Some(inner) = self.unsafe_cell_inner(receiver.ty) {
            if !explicit.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_unsafe_cell_method(receiver, inner, name, args, span);
        }
        if self.is_arena(receiver.ty) {
            return self.synth_arena_method(receiver, name, args, &explicit, span);
        }
        if self.is_fixed_arena(receiver.ty) {
            return self.synth_fixed_arena_method(receiver, name, args, &explicit, span);
        }
        if self.is_scoped_arena(receiver.ty) {
            return self.synth_scoped_arena_method(receiver, name, args, &explicit, span);
        }
        if let Some(elem) = self.arena_array_element(receiver.ty) {
            if !explicit.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_arena_array_method(receiver, elem, name, args, span);
        }
        if let Some((elem, mutable)) = self.arena_array_iterator(receiver.ty) {
            if !explicit.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_arena_array_iterator_method(
                receiver,
                elem,
                mutable,
                name,
                args,
                span,
            );
        }
        if let Some((elem, kind)) = self.span_iterator(receiver.ty) {
            if !explicit.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_span_iterator_method(receiver, elem, kind, name, args, span);
        }
        if let Some((key, value)) = self.arena_map_parts(receiver.ty) {
            if !explicit.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_arena_map_method(receiver, key, value, name, args, span);
        }
        if let Some((key, value)) = self.arena_map_iterator(receiver.ty) {
            if !explicit.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_arena_map_iterator_method(
                receiver,
                key,
                value,
                name,
                args,
                span,
            );
        }
        // `[TYP-17]` — on a generic parameter, only what its bounds provide
        // is permitted, and that is exactly what is looked up.
        if let TyKind::Param { index, name: param } = *self.types.kind(receiver.ty) {
            return self.synth_bound_method(
                index,
                param,
                receiver,
                name,
                args,
                explicit,
                span,
            );
        }
        // `[DRP-1]` — "`fn drop(mut self)` is invoked exactly once per value at
        // the end of its life. It may not be called explicitly (`E3070`)."
        //
        // "Exactly once" is the whole of it. An explicit call does not replace
        // the one at the end of the life, it *adds* to it — the value is still
        // live afterwards and is still dropped at scope end — so the
        // destructor runs twice and anything it frees is freed twice. For
        // `struct R: v: Array[i32]` that is a double free of the buffer, and
        // the second run reads it after freeing, with no `unsafe` anywhere in
        // the program.
        if name.name.is("drop") && self.lookup_method(receiver.ty, name.name).is_some() {
            let shown = self.types.display(receiver.ty);
            self.sink.emit(
                Diagnostic::error(
                    codes::E3070,
                    name.span,
                    format!("`{shown}`'s `drop` may not be called explicitly"),
                )
                .primary_label("this would run the destructor a second time")
                .help("`mem.drop(owned x)` ends the value's life early; the destructor then runs once, here")
                .note(concat!(
                    "`drop` is invoked exactly once per value, at the end of its life ",
                    "[DRP-1]"
                )),
            );
            return Expr { ty: self.common.void, kind: ExprKind::Error, span };
        }
        let Some(entry) = self.lookup_method(receiver.ty, name.name) else {
            let shown = self.types.display(receiver.ty);
            self.error(
                codes::E1010,
                name.span,
                format!("`{shown}` has no method named `{}`", name.name),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };
        let def = entry.def;
        let receiver_mode = entry.receiver;

        let inherited = !self.methods.contains_key(&(receiver.ty, name.name));
        if receiver_mode != Mode::Mut {
            if let Some((_, receiver_ty, _, _)) = self.signatures[def.0 as usize].params.first() {
                // The base receiver is the first parameter of the inherited
                // method. `coerce` inserts the nominal class upcast only when
                // the two class identities differ.
                receiver = self.coerce(receiver, *receiver_ty);
            }
        }

        if !self.signatures[def.0 as usize].generics.is_empty() {
            return self.synth_generic_method_call(
                def,
                name.name,
                Some((receiver, receiver_mode, recv.span)),
                true,
                args,
                explicit,
                span,
                true,
            );
        }
        if !explicit.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }

        // `[TYP-24]` — the same name reachable through two implemented
        // interfaces has to be disambiguated by the caller.
        if let Some(interface) = entry.from_interface {
            let others: Vec<Symbol> = self
                .implemented
                .iter()
                .filter(|(t, i, _)| *t == receiver.ty && *i != interface)
                .filter(|(_, i, _)| {
                    self.interfaces
                        .get(i)
                        .is_some_and(|d| d.methods.iter().any(|(m, _, _, _)| *m == name.name))
                })
                .map(|(_, i, _)| *i)
                .collect();
            if let Some(other) = others.first() {
                self.error(
                    codes::E2070,
                    name.span,
                    format!(
                        "`{}` is offered by both `{interface}` and `{other}`",
                        name.name
                    ),
                );
            }
        }

        let signature: Vec<(Ty, Mode)> =
            self.signatures[def.0 as usize].params.iter().map(|(_, t, m, _)| (*t, *m)).collect();
        let ret = self.signatures[def.0 as usize].ret;

        // The receiver is the first parameter; the written arguments are the
        // rest.
        let expected = signature.len().saturating_sub(1);
        if args.len() != expected {
            self.error(
                codes::E2020,
                span,
                format!("`{}` takes {expected} arguments, found {}", name.name, args.len()),
            );
        }
        // `[IFC-4]` — the receiver is concrete here, so an associated type in
        // the signature resolves to what this type declared it to be.
        let ret = self.resolve_assoc(ret, receiver.ty);
        let checked_receiver = self.pass_receiver(receiver, receiver_mode, recv.span);
        let checked_receiver = if inherited && receiver_mode == Mode::Mut {
            // `[CLS-4]` — an inherited mutable method operates on the same
            // object through the base receiver type.  The receiver is already
            // a mutable borrow of the derived place; adapt that borrow, not
            // the owning class handle, so no retain or ownership transfer is
            // introduced by the upcast.
            if let Some((_, receiver_ty, _, _)) = self.signatures[def.0 as usize].params.first() {
                let expected = self.types.intern(TyKind::Ref {
                    mutable: true,
                    inner: *receiver_ty,
                });
                if checked_receiver.ty != expected {
                    Expr {
                        ty: expected,
                        kind: ExprKind::Cast {
                            expr: Box::new(checked_receiver),
                            to: expected,
                        },
                        span: recv.span,
                    }
                } else {
                    checked_receiver
                }
            } else {
                checked_receiver
            }
        } else {
            checked_receiver
        };
        let mut checked = vec![checked_receiver];
        let params = self.signatures[def.0 as usize].params[1..].to_vec();
        let slots = self.call_argument_slots(name.name, args, &params);
        checked.extend(self.check_bound_call_arguments(args, &params, &slots));
        Expr {
            ty: ret,
            kind: ExprKind::Call {
                callee: def,
                arg_eval_order: Self::call_eval_order_with_receiver(&slots),
                args: checked,
                latebound: false,
            },
            span,
        }
    }

    /// `[TYP-17]` — a method call on a generic parameter. Only the interfaces
    /// bounding it may provide the method; there is no duck typing, so a
    /// missing bound is `E2040` with the bound that would fix it.
    fn synth_bound_method(
        &mut self,
        index: u32,
        param: Symbol,
        receiver: Expr,
        name: ast::Ident,
        args: &[ast::Arg],
        explicit: Vec<Ty>,
        span: Span,
    ) -> Expr {
        let bounds = self
            .current_generics
            .get(index as usize)
            .map(|p| p.bounds.clone())
            .unwrap_or_default();

        let mut found: Option<(DefId, Mode, Symbol)> = None;
        for bound in &bounds {
            let Some(def) = self.interfaces.get(bound) else { continue };
            if let Some((_, method, Some(receiver_mode), _)) =
                def.methods.iter().find(|(m, _, receiver, _)| {
                    *m == name.name && receiver.is_some()
                })
            {
                // `[TYP-24]` — two bounds offering the same name is ambiguous.
                if let Some((_, _, first)) = found {
                    self.error(
                        codes::E2070,
                        name.span,
                        format!("`{}` is offered by both `{first}` and `{bound}`", name.name),
                    );
                    break;
                }
                found = Some((*method, *receiver_mode, *bound));
            }
        }

        let Some((def, receiver_mode, _)) = found else {
            let candidates: Vec<Symbol> = self
                .interfaces
                .iter()
                .filter(|(_, d)| {
                    d.methods
                        .iter()
                        .any(|(m, _, receiver, _)| *m == name.name && receiver.is_some())
                })
                .map(|(name, _)| *name)
                .collect();
            let mut diagnostic = Diagnostic::error(
                codes::E2040,
                name.span,
                format!("`{param}` has no method `{}`; its bounds do not provide one", name.name),
            )
            .note("inside a generic body only the bounds' operations are available [TYP-17]");
            if let Some(bound) = candidates.first() {
                diagnostic = diagnostic
                    .help(format!("add the bound: `{param}: {bound}`"));
            }
            self.sink.emit(diagnostic);
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };

        if !self.signatures[def.0 as usize].generics.is_empty() {
            return self.synth_generic_method_call(
                def,
                name.name,
                Some((receiver, receiver_mode, span)),
                false,
                args,
                explicit,
                span,
                false,
            );
        }
        if !explicit.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`{}` takes no type arguments, found {}", name.name, explicit.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }

        // Part IV §8 — the interface was declared over `Self`; here the
        // implementing type is the receiver's, so `Self` becomes it. Without
        // this, `fn less(self, other: Self) -> bool` asks a `T` for a `Self`.
        let concrete = receiver.ty;
        let signature: Vec<(Ty, Mode)> = self.signatures[def.0 as usize]
            .params
            .iter()
            .map(|(_, t, m, _)| (*t, *m))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(t, m)| (self.types.substitute_self(t, concrete), m))
            .collect();
        let ret = self.signatures[def.0 as usize].ret;
        let ret = self.types.substitute_self(ret, concrete);
        // An interface declaration has no receiver in its parameter list, so
        // every declared parameter is a written argument.
        if args.len() != signature.len() {
            self.error(
                codes::E2020,
                span,
                format!("`{}` takes {} arguments, found {}", name.name, signature.len(), args.len()),
            );
        }
        let mut checked = vec![self.pass_receiver(receiver, receiver_mode, span)];
        let params = self.signatures[def.0 as usize].params.clone();
        let params = params
            .into_iter()
            .map(|(param, ty, mode, param_span)| {
                (param, self.types.substitute_self(ty, concrete), mode, param_span)
            })
            .collect::<Vec<_>>();
        let slots = self.call_argument_slots(name.name, args, &params);
        checked.extend(self.check_bound_call_arguments(args, &params, &slots));
        Expr {
            ty: ret,
            kind: ExprKind::Call {
                callee: def,
                arg_eval_order: Self::call_eval_order_with_receiver(&slots),
                args: checked,
                latebound: false,
            },
            span,
        }
    }

    /// The compiler-known methods on `Array[T]` and `String`.
    /// `[CLO-3]` — a call through a value of function type.
    fn synth_indirect_call(
        &mut self,
        callee: Expr,
        args: &[ast::Arg],
        span: Span,
        consumes_callee: bool,
        latebound_override: bool,
    ) -> Expr {
        let TyKind::Fn { latebound, params, ret } = self.types.kind(callee.ty).clone() else {
            unreachable!("checked by the caller")
        };
        let latebound = latebound || latebound_override;
        if args.len() != params.len() {
            let shown = self.types.display(callee.ty);
            self.error(
                codes::E2020,
                span,
                format!("`{shown}` takes {} arguments, found {}", params.len(), args.len()),
            );
            return Expr { ty: ret, kind: ExprKind::Error, span };
        }
        let mut checked = Vec::new();
        for (arg, param) in args.iter().zip(params.iter()) {
            if let Some(name) = arg.name {
                // `[TYP-25]`'s named arguments match parameter *names*, and a
                // function type has none.
                self.error(
                    codes::E2020,
                    name.span,
                    "a call through a function value takes positional arguments",
                );
            }
            checked.push(self.check_argument(&arg.value, param.ty, hir_mode(param.mode)));
        }
        Expr {
            ty: ret,
            kind: ExprKind::CallIndirect {
                callee: Box::new(callee),
                args: checked,
                consumes_callee,
                latebound,
            },
            span,
        }
    }

    /// Lower a statically known capture-free function value as a direct call
    /// inside a concrete generic instance. This preserves the function body's
    /// verified callable-region summary while retaining the expected
    /// `@latebound` boundary at the call site.
    fn synth_known_function_call(
        &mut self,
        def: DefId,
        args: &[ast::Arg],
        span: Span,
        latebound: bool,
    ) -> Expr {
        let signature: Vec<(Ty, Mode)> =
            self.signatures[def.0 as usize].params.iter().map(|(_, ty, mode, _)| (*ty, *mode)).collect();
        let ret = self.signatures[def.0 as usize].ret;
        if args.len() != signature.len() {
            self.error(
                codes::E2020,
                span,
                format!("this function takes {} arguments, found {}", signature.len(), args.len()),
            );
            return Expr { ty: ret, kind: ExprKind::Error, span };
        }
        let checked = args
            .iter()
            .zip(signature.iter())
            .map(|(arg, &(param_ty, mode))| {
                if let Some(name) = arg.name {
                    self.error(
                        codes::E2020,
                        name.span,
                        "a call through a function value takes positional arguments",
                    );
                }
                self.check_argument(&arg.value, param_ty, mode)
            })
            .collect();
        Expr {
            ty: ret,
            kind: ExprKind::Call {
                callee: def,
                arg_eval_order: None,
                args: checked,
                latebound,
            },
            span,
        }
    }

    /// `[CLO-1]`, `[CLO-2]`, `[TYP-23]` rule 4 — a closure.
    ///
    /// A capture-free closure is "a plain value type" (`[CLO-1]`), and Part IV
    /// §9's `fn(A) -> R` is that type, so it lowers to a synthesised function
    /// with a name of its own. A capturing one is refused by name rather than
    /// silently mis-compiled: the body is checked in a scope of its own, so a
    /// captured name is simply not found, and the diagnostic recognises that
    /// case from the enclosing scopes it kept.
    ///
    /// `expected` supplies the parameter types the closure omits: "Lambda
    /// parameter types are inferred from the expected function type; a lambda
    /// with unannotated parameters in a context without an expected type is
    /// `E2061`".
    fn synth_lambda(&mut self, lambda: &ast::Lambda, expected: Option<Ty>, span: Span) -> Expr {
        let wanted = expected.and_then(|ty| match self.types.kind(ty) {
            TyKind::Fn { params, ret, .. } => Some((params.clone(), *ret)),
            _ => None,
        });

        let mut params: Vec<(Symbol, Ty, Mode, Span)> = Vec::new();
        for (index, param) in lambda.params.iter().enumerate() {
            let ast::ParamKind::Named { name, ty } = &param.kind else {
                self.error(codes::E2020, param.span, "a closure has no receiver");
                continue;
            };
            let declared = match &ty.kind {
                ast::TypeKind::Infer => None,
                _ => Some(self.resolve_type(ty)),
            };
            let expected = wanted.as_ref().and_then(|(params, _)| params.get(index));
            let resolved = match (declared, expected) {
                (Some(ty), _) => ty,
                (None, Some(param)) => param.ty,
                (None, None) => {
                    self.sink.emit(
                        Diagnostic::error(
                            codes::E2061,
                            param.span,
                            format!("cannot tell what `{}` holds", name.name),
                        )
                        .help(
                            "annotate it, or write the closure where a `fn(…)` type is expected",
                        )
                        .note(
                            "a lambda's parameter types come from the expected function type \
                             [TYP-23]",
                        ),
                    );
                    self.common.error
                }
            };
            if let Some(expected) = expected
                && fn_param_mode(param.mode) != expected.mode
            {
                let expected = fn_param_mode_name(expected.mode);
                let found = ast_mode_name(param.mode);
                self.error(
                    codes::E2020,
                    param.span,
                    format!(
                        "callable parameter mode mismatch: expected `{expected}`, found `{found}`"
                    ),
                );
            }
            params.push((name.name, resolved, hir_mode(fn_param_mode(param.mode)), param.span));
        }

        let declared_ret = lambda.ret.as_ref().map(|t| self.resolve_type(t));
        // A callback may be the only argument that can solve its result type
        // (`with_views[A, B, R](..., f: fn(...) -> R)`).  Its parameter types
        // are still a useful bidirectional hint, but an unresolved generic
        // result must not be forced on the body before the body can infer it.
        // The post-synthesis callable-bound unification records that inferred
        // result in the enclosing generic call.
        let expected_ret = wanted
            .as_ref()
            .map(|(_, ret)| *ret)
            .filter(|ret| !self.types.is_generic(*ret));
        let ret = declared_ret.or(expected_ret);

        // `[CLO-1]`, `[CLO-2]` — two passes over the body.
        //
        // The first is a **probe**: the body is checked with an environment
        // that does not exist yet, and every name it reaches for that the
        // enclosing function declares is recorded rather than reported. Its
        // diagnostics are discarded, because it is a question and not a
        // compilation — leaving them in would double every error in the body.
        //
        // `[CLO-2]` decides what a capture is: normal read-only use captures a
        // shared borrow; `owned fn` captures by move/copy. The former remains
        // an ordinary loan, while the latter becomes an owned environment
        // field and therefore follows ordinary move/drop checking.
        let outer: HashMap<Symbol, Ty> = self
            .scopes
            .iter()
            .flat_map(|scope| scope.iter())
            .map(|(name, local)| (*name, self.locals[local.0 as usize].ty))
            .collect();

        let saved = self.sink.take();
        let probe = CaptureWatch {
            outer: outer.clone(),
            found: Vec::new(),
            env: None,
            captures_by_move: lambda.is_owned,
        };
        let (_, _, _, probe) = self.check_lambda_body(lambda, &params, ret, probe, None);
        let _discarded = self.sink.take();
        for diagnostic in saved {
            self.sink.emit(diagnostic);
        }
        let captured = probe.found;

        // A capture-free closure is "a plain value type" (`[CLO-1]`), and Part
        // IV §9's `fn(A) -> R` is that type, so it stays a function pointer and
        // goes everywhere one can — a `fn` local, a field, and the
        // `extern "C" fn` parameter that `[CLO-3]` says accepts "only
        // capture-free closures and named functions".
        if captured.is_empty() {
            let watch = CaptureWatch {
                outer,
                found: Vec::new(),
                env: None,
                captures_by_move: lambda.is_owned,
            };
            let (body, body_ty, locals, _) =
                self.check_lambda_body(lambda, &params, ret, watch, None);
            let ret = ret.unwrap_or(body_ty);
            let def = self.push_closure(&params, ret, locals, body, None, span);
            let ty = self.types.intern(TyKind::Fn {
                latebound: false,
                params: params
                    .iter()
                    .map(|(_, t, mode, _)| FnParam {
                        ty: *t,
                        mode: fn_param_mode_from_hir(*mode),
                    })
                    .collect(),
                ret,
            });
            return Expr { ty, kind: ExprKind::FnValue(def), span };
        }

        // A capture starts with a private probe environment. A non-owned probe
        // uses `ref mut` fields; an owned probe stores fields directly and
        // receives its whole environment by `owned`. Ordinary type checking
        // can then expose every actual write (assignment, a `mut` call, or a
        // mutable builtin receiver) before the final environment chooses
        // `ref` versus `ref mut` per non-owned field. The probe function is
        // never emitted; it prevents a source heuristic from deciding borrow
        // capability and keeps `[CLO-2]` attached to the typed HIR lowering
        // will consume.
        let (mutable_capture_fields, probe_once) = {
            let probe_fields: Vec<FieldDef> = captured
                .iter()
                .map(|(name, ty)| FieldDef {
                    name: *name,
                    ty: if lambda.is_owned {
                        *ty
                    } else {
                        self.types.intern(TyKind::Ref { mutable: true, inner: *ty })
                    },
                    span,
                    has_default: false,
                    read_only_outside: false,
                    vis: FieldVis::Private,
                })
                .collect();
            let probe_name = Symbol::intern(&format!(
                "closure{}_capture_probe_env",
                self.lambdas.len()
            ));
            let probe_id = self.types.add_struct(StructDef {
                name: probe_name,
                fields: probe_fields,
                span,
                derives_copy: false,
                has_drop: false,
                drops_fields: true,
                origin: None,
                declaring_module: self.current_module,
            });
            let probe_ty = self.types.intern(TyKind::Struct(probe_id));
            let probe_watch = CaptureWatch {
                outer: outer.clone(),
                found: captured.clone(),
                env: Some((LocalId(0), probe_id)),
                captures_by_move: lambda.is_owned,
            };
            let (probe_body, _, _, _) = self.check_lambda_body(
                lambda,
                &params,
                ret,
                probe_watch,
                Some((
                    Symbol::intern("env"),
                    probe_ty,
                    if lambda.is_owned { Mode::Owned } else { Mode::Mut },
                    span,
                )),
            );
            (
                self.closure_body_mutated_capture_fields(&probe_body, LocalId(0)),
                lambda.is_owned && self.closure_body_moves_capture(&probe_body, LocalId(0)),
            )
        };
        let requires_mut = !probe_once && !mutable_capture_fields.is_empty();

        // Otherwise it is "a unique anonymous struct implementing `Callable`"
        // (`[CLO-1]`), and this builds that struct: one field per capture, in
        // the order the body first mentioned them. An `owned fn` stores those
        // fields by move/copy; an ordinary closure stores a shared or mutable
        // borrow according to the typed probe above.
        let fields: Vec<FieldDef> = captured
            .iter()
            .enumerate()
            .map(|(index, (name, ty))| FieldDef {
                name: *name,
                ty: if lambda.is_owned {
                    *ty
                } else {
                    self.types.intern(TyKind::Ref {
                        mutable: mutable_capture_fields.contains(&index),
                        inner: *ty,
                    })
                },
                span,
                has_default: false,
                read_only_outside: false,
                vis: FieldVis::Private,
            })
            .collect();
        let env_name = Symbol::intern(&format!("closure{}_env", self.lambdas.len()));
        let struct_id = self.types.add_struct(StructDef {
            name: env_name,
            fields,
            span,
            derives_copy: if lambda.is_owned {
                captured.iter().all(|(_, ty)| self.types.is_copy(*ty))
            } else {
                !requires_mut
            },
            has_drop: false,
            drops_fields: true,
            origin: None,
            declaring_module: self.current_module,
        });
        let env_ty = self.types.intern(TyKind::Struct(struct_id));

        let watch = CaptureWatch {
            outer,
            found: captured.clone(),
            env: Some((LocalId(0), struct_id)),
            captures_by_move: lambda.is_owned,
        };
        let body_env_mode = if probe_once {
            Mode::Owned
        } else if requires_mut {
            Mode::Mut
        } else {
            Mode::Borrow
        };
        let (body, body_ty, locals, _) = self.check_lambda_body(
            lambda,
            &params,
            ret,
            watch,
            Some((Symbol::intern("env"), env_ty, body_env_mode, span)),
        );
        let ret = ret.unwrap_or(body_ty);
        // `[CLO-2]` — an `owned fn` stays reusable when it merely reads its
        // directly owned captures. It becomes `CallableOnce` only when the
        // typed body actually consumes a non-`Copy` capture.
        let once = lambda.is_owned && self.closure_body_moves_capture(&body, LocalId(0));
        debug_assert_eq!(once, probe_once, "closure probe and final body disagree about CallableOnce");
        let call_mode = if once {
            Mode::Owned
        } else if requires_mut {
            Mode::Mut
        } else {
            Mode::Borrow
        };
        let def = self.push_closure(
            &params,
            ret,
            locals,
            body,
            Some((
                Symbol::intern("env"),
                env_ty,
                call_mode,
                lambda.is_owned,
            )),
            span,
        );
        self.closure_calls.insert(struct_id, ClosureCall { def, once });

        // The closure *value* is its environment. Ordinary closures borrow
        // each captured local; `owned fn` moves or copies each capture into
        // the environment, so the source observes normal move semantics.
        let mut values = Vec::new();
        for (index, (name, ty)) in captured.iter().enumerate() {
            let Some(local) = self.lookup(*name) else { continue };
            let place = Expr { ty: *ty, kind: ExprKind::Local(local), span };
            if lambda.is_owned {
                values.push(place);
            } else {
                let mutable = mutable_capture_fields.contains(&index);
                let reference = self.types.intern(TyKind::Ref { mutable, inner: *ty });
                values.push(Expr {
                    ty: reference,
                    kind: ExprKind::Ref { place: Box::new(place), mutable },
                    span,
                });
            }
        }
        Expr { ty: env_ty, kind: ExprKind::StructLit { struct_id, fields: values }, span }
    }

    /// The body of a lambda, checked in a context of its own: fresh locals,
    /// fresh scopes, and the enclosing ones kept aside so `[CLO-2]` can tell a
    /// capture from a typo. Run twice per closure — once to discover the
    /// captures, once with the environment they live in.
    fn check_lambda_body(
        &mut self,
        lambda: &ast::Lambda,
        params: &[(Symbol, Ty, Mode, Span)],
        ret: Option<Ty>,
        watch: CaptureWatch,
        env: Option<(Symbol, Ty, Mode, Span)>,
    ) -> (Block, Ty, Vec<LocalDecl>, CaptureWatch) {
        let outer_locals = std::mem::take(&mut self.locals);
        let outer_scopes = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);
        let outer_borrowed_params = std::mem::take(&mut self.borrowed_params);
        let outer_callable_once_locals = std::mem::take(&mut self.callable_once_locals);
        let outer_callable_parameter_locals =
            std::mem::take(&mut self.callable_parameter_locals);
        let outer_latebound_callable_parameter_locals =
            std::mem::take(&mut self.latebound_callable_parameter_locals);
        let outer_callable_value_bindings =
            std::mem::take(&mut self.callable_value_bindings);
        let outer_ret = self.ret_ty;
        let outer_watch = self.captures.replace(watch);
        self.ret_ty = ret.unwrap_or(self.common.void);

        // The environment is parameter zero, so the emitted function has
        // `[CLO-6]`'s `call(self, args)` shape.
        if let Some((name, ty, mode, env_span)) = env {
            let local_ty = match mode {
                Mode::Mut => self.mut_param_ty(ty),
                Mode::Borrow | Mode::Owned => ty,
            };
            let local = self.declare(Some(name), local_ty, env_span);
            if mode == Mode::Borrow {
                self.borrowed_params.insert(local);
            }
        }
        for (name, ty, mode, param_span) in params {
            let local_ty = match mode {
                Mode::Mut => self.mut_param_ty(*ty),
                Mode::Borrow | Mode::Owned => *ty,
            };
            let local = self.declare(Some(*name), local_ty, *param_span);
            if *mode == Mode::Borrow {
                self.borrowed_params.insert(local);
            }
        }

        let (body, body_ty) = match &lambda.body {
            ast::LambdaBody::Expr(expr) => {
                // `[LEX-6a]`'s bracketed form is `fn(x): <small_stmt>`, and the
                // statement is very often `return e`. `[GRM-16]` makes that an
                // expression, so it arrives here as one; `fn(x): e` and
                // `fn(x) => e` mean the same thing and produce the same body.
                let expr = match &expr.kind {
                    ast::ExprKind::Jump(ast::Jump::Return(Some(inner))) => inner.as_ref(),
                    _ => expr.as_ref(),
                };
                let value = match ret {
                    Some(ty) if ty != self.common.void => self.check_expr(expr, ty),
                    _ => self.synth(expr),
                };
                let ty = value.ty;
                (Block { stmts: vec![Stmt::Return(Some(value))], span: expr.span }, ty)
            }
            ast::LambdaBody::Block(block) => {
                let checked = self.check_block(block);
                (checked, self.ret_ty)
            }
        };

        let locals = std::mem::replace(&mut self.locals, outer_locals);
        self.scopes = outer_scopes;
        self.borrowed_params = outer_borrowed_params;
        self.callable_once_locals = outer_callable_once_locals;
        self.callable_parameter_locals = outer_callable_parameter_locals;
        self.latebound_callable_parameter_locals = outer_latebound_callable_parameter_locals;
        self.callable_value_bindings = outer_callable_value_bindings;
        self.ret_ty = outer_ret;
        let watch = std::mem::replace(&mut self.captures, outer_watch)
            .expect("the watch was installed above");
        (body, body_ty, locals, watch)
    }

    /// Record a checked closure body as a function of its own, and return the
    /// `DefId` that names it. The environment, when there is one, is its first
    /// parameter.
    fn push_closure(
        &mut self,
        params: &[(Symbol, Ty, Mode, Span)],
        ret: Ty,
        locals: Vec<LocalDecl>,
        body: Block,
        env: Option<(Symbol, Ty, Mode, bool)>,
        span: Span,
    ) -> DefId {
        let def = DefId(self.signatures.len() as u32);
        let closure_environment = env.as_ref().and_then(|(_, ty, _, _)| match self.types.kind(*ty) {
            TyKind::Struct(id) => Some(*id),
            _ => None,
        });
        let closure_captures_by_move = env.as_ref().is_some_and(|(_, _, _, owned)| *owned);
        let mut signature_params: Vec<(Symbol, Ty, Mode, Span)> = Vec::new();
        if let Some((name, ty, mode, _)) = env {
            signature_params.push((name, ty, mode, span));
        }
        signature_params.extend(params.iter().map(|(name, ty, mode, span)| (*name, *ty, *mode, *span)));
        let hir_params: Vec<Param> = signature_params
            .iter()
            .enumerate()
            .map(|(index, (_, _, mode, _))| Param { local: LocalId(index as u32), mode: *mode })
            .collect();
        self.signatures.push(Signature {
            params: signature_params,
            ret,
            borrows: None,
            generics: Vec::new(),
        });
        let symbol = format!("{}closure{}", ember_branding::mangle_prefix(), self.lambdas.len());
        self.lambdas.push(Function {
            def,
            name: Symbol::intern(&symbol),
            class_init: false,
            symbol,
            is_unsafe: false,
            abi: None,
            params: hir_params,
            locals,
            ret,
            body,
            span,
            overflow: self.default_overflow,
            borrows: None,
            closure_environment,
            closure_captures_by_move,
        });
        def
    }

    /// `[CLO-2]` — determine whether an `owned fn` must be one-shot from the
    /// checked closure body, rather than from capture mode alone. An owned
    /// environment can be called repeatedly while its fields are only read;
    /// it is `CallableOnce` exactly when a non-`Copy` capture is used in a
    /// consuming value position.
    fn closure_body_moves_capture(&self, body: &Block, environment: LocalId) -> bool {
        body.stmts.iter().any(|stmt| self.closure_stmt_moves_capture(stmt, environment))
    }

    /// `[CLO-2]` — collect the environment fields that a non-owned closure
    /// actually mutates. This runs on a typed all-`ref mut` probe body, so a
    /// method receiver and a direct assignment travel through one ordinary
    /// representation: both form a `ref mut` rooted in the generated
    /// environment field.
    fn closure_body_mutated_capture_fields(
        &self,
        body: &Block,
        environment: LocalId,
    ) -> HashSet<usize> {
        let mut fields = HashSet::new();
        for stmt in &body.stmts {
            self.collect_mutated_capture_fields_stmt(stmt, environment, &mut fields);
        }
        fields
    }

    fn collect_mutated_capture_fields_stmt(
        &self,
        stmt: &Stmt,
        environment: LocalId,
        fields: &mut HashSet<usize>,
    ) {
        match stmt {
            Stmt::Let { init, .. } => {
                if let Some(init) = init {
                    self.collect_mutated_capture_fields_expr(init, environment, fields);
                }
            }
            Stmt::Assign { place, value } => {
                if let Some(field) = Self::closure_capture_field_index(place, environment) {
                    fields.insert(field);
                }
                self.collect_mutated_capture_fields_expr(place, environment, fields);
                self.collect_mutated_capture_fields_expr(value, environment, fields);
            }
            Stmt::Destructure { value, bindings, .. } => {
                self.collect_mutated_capture_fields_expr(value, environment, fields);
                for binding in bindings {
                    match binding {
                        DestructureBinding::Let { value, .. } => {
                            self.collect_mutated_capture_fields_expr(value, environment, fields);
                        }
                        DestructureBinding::Assign { place, value } => {
                            if let Some(field) =
                                Self::closure_capture_field_index(place, environment)
                            {
                                fields.insert(field);
                            }
                            self.collect_mutated_capture_fields_expr(place, environment, fields);
                            self.collect_mutated_capture_fields_expr(value, environment, fields);
                        }
                    }
                }
            }
            Stmt::Expr(expr) => self.collect_mutated_capture_fields_expr(expr, environment, fields),
            Stmt::Return(value) => {
                if let Some(value) = value {
                    self.collect_mutated_capture_fields_expr(value, environment, fields);
                }
            }
            Stmt::If { cond, then_block, else_block } => {
                self.collect_mutated_capture_fields_expr(cond, environment, fields);
                for stmt in &then_block.stmts {
                    self.collect_mutated_capture_fields_stmt(stmt, environment, fields);
                }
                if let Some(else_block) = else_block {
                    for stmt in &else_block.stmts {
                        self.collect_mutated_capture_fields_stmt(stmt, environment, fields);
                    }
                }
            }
            Stmt::While { cond, body, else_block } => {
                self.collect_mutated_capture_fields_expr(cond, environment, fields);
                for stmt in &body.stmts {
                    self.collect_mutated_capture_fields_stmt(stmt, environment, fields);
                }
                if let Some(else_block) = else_block {
                    for stmt in &else_block.stmts {
                        self.collect_mutated_capture_fields_stmt(stmt, environment, fields);
                    }
                }
            }
            Stmt::ForRange { start, end, body, else_block, .. } => {
                self.collect_mutated_capture_fields_expr(start, environment, fields);
                self.collect_mutated_capture_fields_expr(end, environment, fields);
                for stmt in &body.stmts {
                    self.collect_mutated_capture_fields_stmt(stmt, environment, fields);
                }
                if let Some(else_block) = else_block {
                    for stmt in &else_block.stmts {
                        self.collect_mutated_capture_fields_stmt(stmt, environment, fields);
                    }
                }
            }
            Stmt::Block(block) | Stmt::Defer(block) => {
                for stmt in &block.stmts {
                    self.collect_mutated_capture_fields_stmt(stmt, environment, fields);
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
        }
    }

    fn collect_mutated_capture_fields_expr(
        &self,
        expr: &Expr,
        environment: LocalId,
        fields: &mut HashSet<usize>,
    ) {
        if let ExprKind::Ref { place, mutable: true } = &expr.kind {
            if let Some(field) = Self::closure_capture_field_index(place, environment) {
                fields.insert(field);
            }
        }

        match &expr.kind {
            ExprKind::Call { args, .. }
            | ExprKind::Builtin { args, .. }
            | ExprKind::TupleLit(args)
            | ExprKind::ArrayLit(args)
            | ExprKind::EnumLit { fields: args, .. } => {
                for arg in args {
                    self.collect_mutated_capture_fields_expr(arg, environment, fields);
                }
            }
            ExprKind::ClassNew { args, .. } => {
                for arg in args {
                    self.collect_mutated_capture_fields_expr(arg, environment, fields);
                }
            }
            ExprKind::CallIndirect { callee, args, .. } => {
                self.collect_mutated_capture_fields_expr(callee, environment, fields);
                for arg in args {
                    self.collect_mutated_capture_fields_expr(arg, environment, fields);
                }
            }
            ExprKind::StructLit { fields: args, .. } => {
                for arg in args {
                    self.collect_mutated_capture_fields_expr(arg, environment, fields);
                }
            }
            ExprKind::ArrayRepeat { value, .. } => {
                self.collect_mutated_capture_fields_expr(value, environment, fields);
            }
            ExprKind::Match { scrutinee, arms } => {
                self.collect_mutated_capture_fields_expr(scrutinee, environment, fields);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.collect_mutated_capture_fields_expr(guard, environment, fields);
                    }
                    match &arm.body {
                        hir::MatchArmBody::Block(block) => {
                            for stmt in &block.stmts {
                                self.collect_mutated_capture_fields_stmt(stmt, environment, fields);
                            }
                        }
                        hir::MatchArmBody::Expr(expr) => {
                            self.collect_mutated_capture_fields_expr(expr, environment, fields);
                        }
                    }
                }
            }
            ExprKind::Field { base, .. }
            | ExprKind::Deref(base)
            | ExprKind::Ref { place: base, .. }
            | ExprKind::Cast { expr: base, .. }
            | ExprKind::Widen { expr: base, .. }
            | ExprKind::EraseRange(base) => {
                self.collect_mutated_capture_fields_expr(base, environment, fields);
            }
            ExprKind::Index { base, index } => {
                self.collect_mutated_capture_fields_expr(base, environment, fields);
                self.collect_mutated_capture_fields_expr(index, environment, fields);
            }
            ExprKind::FString { parts, .. } => {
                for part in parts {
                    if let hir::FStringPart::Value(value) = part {
                        self.collect_mutated_capture_fields_expr(value, environment, fields);
                    }
                }
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                self.collect_mutated_capture_fields_expr(lhs, environment, fields);
                self.collect_mutated_capture_fields_expr(rhs, environment, fields);
            }
            ExprKind::Unary { operand, .. } => {
                self.collect_mutated_capture_fields_expr(operand, environment, fields);
            }
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Str(_)
            | ExprKind::Local(_)
            | ExprKind::FnValue(_)
            | ExprKind::Error => {}
        }
    }

    fn closure_stmt_moves_capture(&self, stmt: &Stmt, environment: LocalId) -> bool {
        match stmt {
            Stmt::Let { init, .. } => init
                .as_ref()
                .is_some_and(|value| self.closure_expr_moves_capture(value, environment, true)),
            Stmt::Assign { place, value } => {
                self.closure_expr_moves_capture(place, environment, false)
                    || self.closure_expr_moves_capture(value, environment, true)
            }
            Stmt::Destructure { value, bindings, .. } => {
                self.closure_expr_moves_capture(value, environment, true)
                    || bindings.iter().any(|binding| match binding {
                        DestructureBinding::Let { value, .. } => {
                            self.closure_expr_moves_capture(value, environment, true)
                        }
                        DestructureBinding::Assign { place, value } => {
                            self.closure_expr_moves_capture(place, environment, false)
                                || self.closure_expr_moves_capture(value, environment, true)
                        }
                    })
            }
            Stmt::Expr(expr) => self.closure_expr_moves_capture(expr, environment, true),
            Stmt::Return(value) => value
                .as_ref()
                .is_some_and(|value| self.closure_expr_moves_capture(value, environment, true)),
            Stmt::If { cond, then_block, else_block } => {
                self.closure_expr_moves_capture(cond, environment, false)
                    || self.closure_body_moves_capture(then_block, environment)
                    || else_block
                        .as_ref()
                        .is_some_and(|block| self.closure_body_moves_capture(block, environment))
            }
            Stmt::While { cond, body, else_block } => {
                self.closure_expr_moves_capture(cond, environment, false)
                    || self.closure_body_moves_capture(body, environment)
                    || else_block
                        .as_ref()
                        .is_some_and(|block| self.closure_body_moves_capture(block, environment))
            }
            Stmt::ForRange { start, end, body, else_block, .. } => {
                self.closure_expr_moves_capture(start, environment, false)
                    || self.closure_expr_moves_capture(end, environment, false)
                    || self.closure_body_moves_capture(body, environment)
                    || else_block
                        .as_ref()
                        .is_some_and(|block| self.closure_body_moves_capture(block, environment))
            }
            Stmt::Block(block) | Stmt::Defer(block) => {
                self.closure_body_moves_capture(block, environment)
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => false,
        }
    }

    fn closure_expr_moves_capture(
        &self,
        expr: &Expr,
        environment: LocalId,
        consuming: bool,
    ) -> bool {
        if consuming
            && !self.types.is_copy(expr.ty)
            && Self::is_closure_capture_place(expr, environment)
        {
            return true;
        }

        match &expr.kind {
            ExprKind::Call { callee, args, .. } => {
                let Some(signature) = self.signatures.get(callee.0 as usize) else {
                    return args
                        .iter()
                        .any(|argument| self.closure_expr_moves_capture(argument, environment, false));
                };
                args.iter().enumerate().any(|(index, argument)| {
                    let consuming = signature
                        .params
                        .get(index)
                        .is_some_and(|(_, _, mode, _)| *mode == Mode::Owned);
                    self.closure_expr_moves_capture(argument, environment, consuming)
                })
            }
            ExprKind::CallIndirect { callee, args, consumes_callee, .. } => {
                self.closure_expr_moves_capture(callee, environment, *consumes_callee)
                    || args
                        .iter()
                        .any(|argument| self.closure_expr_moves_capture(argument, environment, false))
            }
            ExprKind::StructLit { fields, .. }
            | ExprKind::TupleLit(fields)
            | ExprKind::ArrayLit(fields)
            | ExprKind::EnumLit { fields, .. } => fields
                .iter()
                .any(|field| self.closure_expr_moves_capture(field, environment, true)),
            ExprKind::ArrayRepeat { value, .. } => {
                self.closure_expr_moves_capture(value, environment, true)
            }
            ExprKind::Match { scrutinee, arms } => {
                self.closure_expr_moves_capture(scrutinee, environment, false)
                    || arms.iter().any(|arm| {
                        arm.guard
                            .as_ref()
                            .is_some_and(|guard| self.closure_expr_moves_capture(guard, environment, false))
                            || match &arm.body {
                                hir::MatchArmBody::Block(block) => {
                                    self.closure_body_moves_capture(block, environment)
                                }
                                hir::MatchArmBody::Expr(value) => {
                                    self.closure_expr_moves_capture(value, environment, true)
                                }
                            }
                    })
            }
            ExprKind::Builtin { which, args } => args.iter().enumerate().any(|(index, argument)| {
                self.closure_expr_moves_capture(
                    argument,
                    environment,
                    Self::builtin_consumes_argument(*which, index),
                )
            }),
            ExprKind::ClassNew { init, args, .. } => {
                let Some(signature) = self.signatures.get(init.0 as usize) else {
                    return args
                        .iter()
                        .any(|argument| self.closure_expr_moves_capture(argument, environment, false));
                };
                args.iter().enumerate().any(|(index, argument)| {
                    let consuming = signature
                        .params
                        .get(index + 1)
                        .is_some_and(|(_, _, mode, _)| *mode == Mode::Owned);
                    self.closure_expr_moves_capture(argument, environment, consuming)
                })
            }
            ExprKind::Field { base, .. }
            | ExprKind::Deref(base)
            | ExprKind::Cast { expr: base, .. }
            | ExprKind::Widen { expr: base, .. }
            | ExprKind::EraseRange(base) => self.closure_expr_moves_capture(base, environment, false),
            ExprKind::Index { base, index } => {
                self.closure_expr_moves_capture(base, environment, false)
                    || self.closure_expr_moves_capture(index, environment, false)
            }
            ExprKind::Ref { place, .. } => self.closure_expr_moves_capture(place, environment, false),
            ExprKind::FString { parts, .. } => parts.iter().any(|part| match part {
                hir::FStringPart::Text(_) => false,
                hir::FStringPart::Value(value) => {
                    self.closure_expr_moves_capture(value, environment, false)
                }
            }),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.closure_expr_moves_capture(lhs, environment, false)
                    || self.closure_expr_moves_capture(rhs, environment, false)
            }
            ExprKind::Unary { operand, .. } => {
                self.closure_expr_moves_capture(operand, environment, false)
            }
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Str(_)
            | ExprKind::Local(_)
            | ExprKind::FnValue(_)
            | ExprKind::Error => false,
        }
    }

    fn is_closure_capture_place(expr: &Expr, environment: LocalId) -> bool {
        Self::closure_capture_field_index(expr, environment).is_some()
    }

    /// Return the first generated-environment field on a place projection.
    /// `(*env).field.subfield` and `(*env).field[index]` still belong to the
    /// same capture for both its mutable-borrow and move-out capability.
    fn closure_capture_field_index(expr: &Expr, environment: LocalId) -> Option<usize> {
        match &expr.kind {
            ExprKind::Field { base, index } => {
                if Self::is_closure_environment_base(base, environment) {
                    Some(*index)
                } else {
                    Self::closure_capture_field_index(base, environment)
                }
            }
            ExprKind::Deref(base)
            | ExprKind::Ref { place: base, .. }
            | ExprKind::Cast { expr: base, .. }
            | ExprKind::Widen { expr: base, .. }
            | ExprKind::EraseRange(base) => Self::closure_capture_field_index(base, environment),
            ExprKind::Index { base, .. } => Self::closure_capture_field_index(base, environment),
            _ => None,
        }
    }

    fn is_closure_environment_base(expr: &Expr, environment: LocalId) -> bool {
        matches!(expr.kind, ExprKind::Local(local) if local == environment)
            || matches!(&expr.kind, ExprKind::Deref(base) if matches!(base.kind, ExprKind::Local(local) if local == environment))
    }

    /// The compiler-known calls that receive an owned value. Ordinary
    /// user-defined calls carry their parameter modes in `Signature` above.
    fn builtin_consumes_argument(which: Builtin, index: usize) -> bool {
        match which {
            Builtin::BoxNew { .. }
            | Builtin::MemForget { .. }
            | Builtin::CellIntoInner
            | Builtin::MaybeUninitAssumeInit { .. }
            | Builtin::UnsafeCellIntoInner
            | Builtin::MaybeUninitSpanAssumeInit { .. } => index == 0,
            Builtin::MemReplace { .. }
            | Builtin::CellSet
            | Builtin::CellReplace
            | Builtin::MaybeUninitWrite { .. }
            | Builtin::ArenaAlloc { .. }
            | Builtin::FixedArenaAlloc { .. }
            | Builtin::ScopedArenaAlloc { .. }
            | Builtin::ArenaArrayPush { .. } => index == 1,
            Builtin::MaybeUninitWriteAt { .. } | Builtin::ArenaArrayInsert { .. } => index == 2,
            Builtin::ArenaMapInsert { .. } => index == 2 || index == 3,
            _ => false,
        }
    }

    /// `[FN-6]` — a named function used as a value.
    ///
    /// "Functions are values of a unique zero-sized function type; they coerce
    /// to `fn(A) -> R` (the generic callable bound)". The unique zero-sized
    /// type is not modelled: this phase gives the value the `fn(A) -> R` type
    /// directly, which is what every use of it wants and which loses only the
    /// ability to distinguish two functions of one signature at compile time —
    /// something `[TYP-26]`'s no-overloading rule already makes unnecessary.
    /// A generic function has no single signature and is not a value
    /// (`[TYP-18]` needs the arguments to pick one), so it is not offered.
    fn function_value(&mut self, name: Symbol, span: Span) -> Option<Expr> {
        let qualified = self.resolve_name(name);
        let def = *self.fn_ids.get(&qualified)?;
        let signature = &self.signatures[def.0 as usize];
        if !signature.generics.is_empty() {
            self.sink.emit(
                Diagnostic::error(
                    codes::E2060,
                    span,
                    format!("`{name}` is generic, so it is not one function"),
                )
                .help(format!(
                    "name the instantiation: `{name}[i32]` picks one, and that is a value"
                ))
                .note("a generic is a recipe; each instantiation is its own function [TYP-16]"),
            );
            return Some(Expr { ty: self.common.error, kind: ExprKind::Error, span });
        }
        let params = signature
            .params
            .iter()
            .map(|(_, ty, mode, _)| FnParam {
                ty: *ty,
                mode: fn_param_mode_from_hir(*mode),
            })
            .collect();
        let ret = signature.ret;
        let ty = self.types.intern(TyKind::Fn { latebound: false, params, ret });
        Some(Expr { ty, kind: ExprKind::FnValue(def), span })
    }

    /// **The only way a view is built.** `[SPN-1]`'s coercion and `[STD-*]`'s
    /// `as_span`/`as_mut_span` both come here, and anything else that produces
    /// a view must too. `as_str` is the third caller: a `str` built from a
    /// `String` points into it exactly as a `Span` points into its container
    /// (D-037 was this call missing — the view arrived with no borrow behind
    /// it and survived the container's next mutation).
    ///
    /// A view is not a conversion. It points **into** its container, so the
    /// container has to be borrowed for as long as the view lives — and the
    /// borrow has to be *written down*, because `Rvalue::Ref` is what
    /// `collect_loans` looks for and `[LT-1]`'s elision is what carries the
    /// loan's region out of the call. D-022 was exactly this borrow missing:
    /// `v: Span[i32] = a` followed by `a.push(…)` compiled, the push
    /// reallocated, and `v` pointed at freed memory — a use-after-free
    /// reachable from Safe Ember, which `[UNS-4]` and `[PHIL-10]` forbid.
    ///
    /// One producer is the point. A second view-making path that forgot the
    /// borrow would reintroduce the same defect silently, so there is no
    /// second path; `verify_views` in `ember_mir` then checks after the fact
    /// that no view in the finished MIR arrived without one.
    fn view_of(
        &mut self,
        container: Expr,
        view_ty: Ty,
        mutable: bool,
        which: hir::Builtin,
    ) -> Expr {
        let span = container.span;
        if mutable {
            self.reject_readonly_write(&container, span);
            let through_shared_ref = self.reject_write_through_shared_ref(&container, span);
            if !through_shared_ref {
                self.reject_borrowed_parameter_write(&container, span, true);
            }
        }
        let reference =
            self.types.intern(TyKind::Ref { mutable, inner: container.ty });
        let borrowed = Expr {
            ty: reference,
            kind: ExprKind::Ref { place: Box::new(container), mutable },
            span,
        };
        Expr {
            ty: view_ty,
            kind: ExprKind::Builtin { which, args: vec![borrowed] },
            span,
        }
    }

    /// `[ARN-8]` — the one associated constructor on
    /// `MaybeUninit[T]`. The type receiver is resolved before ordinary method
    /// dispatch because it is a type, not a runtime value.
    fn synth_maybe_uninit_construction(
        &mut self,
        ty: Ty,
        name: ast::Ident,
        generic_args: &[ast::GenericArg],
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        if ty == self.common.error {
            return error;
        }
        if self.reject_method_type_args(name.name, generic_args, span) {
            return error;
        }
        let Some(inner) = self.maybe_uninit_inner(ty) else {
            return error;
        };
        if !name.name.is("uninit") {
            self.error(
                codes::E1010,
                name.span,
                format!("`MaybeUninit[{}]` has no associated operation named `{}`", self.types.display(inner), name.name),
            );
            return error;
        }
        if !args.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`uninit` takes no arguments, found {}", args.len()),
            );
            return error;
        }
        Expr {
            ty,
            kind: ExprKind::Builtin {
                which: Builtin::MaybeUninitUninit { inner },
                args: Vec::new(),
            },
            span,
        }
    }

    /// `[ARN-8]`, `[ARN-8a]` — the transitions exposed by a
    /// `MaybeUninit[T]` place. `write` uses ordinary mutable-place borrowing
    /// but a dedicated lowering operation, because it must not read or drop
    /// the prior bytes. `assume_init` is the one unchecked read boundary.
    fn synth_maybe_uninit_method(
        &mut self,
        receiver: Expr,
        inner: Ty,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let method = name.name.as_str();
        let arity = match method {
            "write" => 1,
            "assume_init" => 0,
            _ => {
                self.error(
                    codes::E1010,
                    name.span,
                    format!(
                        "`MaybeUninit[{}]` has no method named `{}`",
                        self.types.display(inner),
                        name.name
                    ),
                );
                return error;
            }
        };
        if args.len() != arity {
            self.error(
                codes::E2020,
                span,
                format!("`{}` takes {arity} arguments, found {}", name.name, args.len()),
            );
            return error;
        }

        if method == "write" {
            let receiver = self.pass_receiver(receiver, Mode::Mut, span);
            let value = self.check_expr(&args[0].value, inner);
            let result = self.types.intern(TyKind::Ref { mutable: true, inner });
            return Expr {
                ty: result,
                kind: ExprKind::Builtin {
                    which: Builtin::MaybeUninitWrite { inner },
                    args: vec![receiver, value],
                },
                span,
            };
        }

        if !self.in_unsafe {
            self.sink.emit(
                Diagnostic::error(
                    codes::E3100,
                    span,
                    "`assume_init` needs an `unsafe` block",
                )
                .help("initialize the slot with `write`, then wrap `assume_init` in `unsafe:`")
                .note("the caller must establish that the slot contains a valid initialized value [ARN-8]"),
            );
        }
        Expr {
            ty: inner,
            kind: ExprKind::Builtin {
                which: Builtin::MaybeUninitAssumeInit { inner },
                args: vec![receiver],
            },
            span,
        }
    }

    /// `[UNS-10]` — the complete and intentionally tiny `UnsafeCell[T]`
    /// operation surface. `get` is the sole shared-access escape to a mutable
    /// raw pointer and therefore requires an unsafe context; it creates no
    /// safe mutable reference and no runtime borrow state. `into_inner`
    /// consumes the wrapper and follows ordinary ownership/destruction.
    fn synth_unsafe_cell_method(
        &mut self,
        receiver: Expr,
        inner: Ty,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let method = name.name.as_str();
        if !matches!(method, "get" | "into_inner") {
            self.error(
                codes::E1010,
                name.span,
                format!(
                    "`UnsafeCell[{}]` has no method named `{}`",
                    self.types.display(inner),
                    name.name
                ),
            );
            return error;
        }
        if !args.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`{method}` takes 0 arguments, found {}", args.len()),
            );
            return error;
        }

        self.reject_unsafe_cell_in_static_safe(span);

        if method == "into_inner" {
            return Expr {
                ty: inner,
                kind: ExprKind::Builtin {
                    which: Builtin::UnsafeCellIntoInner,
                    args: vec![receiver],
                },
                span,
            };
        }

        if !self.in_unsafe {
            self.sink.emit(
                Diagnostic::error(codes::E3100, span, "`UnsafeCell.get` needs an `unsafe` block")
                    .help("wrap the raw-pointer operation in `unsafe:`")
                    .note("`get` is the explicit unsafe boundary for shared interior mutation [UNS-10]"),
            );
        }
        let cell_ref = self.types.intern(TyKind::Ref {
            mutable: false,
            inner: receiver.ty,
        });
        let borrowed = Expr {
            ty: cell_ref,
            kind: ExprKind::Ref { place: Box::new(receiver), mutable: false },
            span,
        };
        let pointer = self.types.intern(TyKind::Ptr { mutable: true, inner });
        Expr {
            ty: pointer,
            kind: ExprKind::Builtin {
                which: Builtin::UnsafeCellGet { inner },
                args: vec![borrowed],
            },
            span,
        }
    }

    /// IX.1 — `Box[T].get() -> ref T`. This is intentionally an ordinary
    /// reference expression rather than a backend-only pointer extraction:
    /// the owner loan and returned region then use the same machinery as all
    /// other safe references.
    fn synth_box_get(
        &mut self,
        receiver: Expr,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let Some(inner) = self.box_inner(receiver.ty) else {
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };
        if !args.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`{}` takes 0 arguments, found {}", name.name, args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if !is_place(&receiver.kind) {
            self.error(
                codes::E2140,
                span,
                "`Box.get` needs an owner place so its returned reference cannot outlive a temporary",
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let payload = self.read_box_through(receiver);
        let ty = self.types.intern(TyKind::Ref { mutable: false, inner });
        Expr {
            ty,
            kind: ExprKind::Ref { place: Box::new(payload), mutable: false },
            span,
        }
    }

    /// `[CELL-1]` — the methods on `Cell[T]`.
    ///
    /// All of them take `self`, a **shared** borrow, and three of them mutate.
    /// Nothing in safe Ember can do that, so something has to be the exception,
    /// and ADR-019 records why it is these builtins rather than `UnsafeCell`.
    /// That primitive is now specified by `[UNS-10]`, but remains separate
    /// implementation work. What makes `Cell` sound is `[CELL-2]`: because
    /// no reference to the contents ever escapes, the write is not an aliasing
    /// question at all — so the borrow checker is told about a builtin call and
    /// is not weakened anywhere.
    ///
    /// `get` is `T: Copy` only. `take` resolves the standard `Default`
    /// capability and carries its concrete constructor into MIR.
    fn synth_cell_method(
        &mut self,
        receiver: Expr,
        inner: Ty,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let shown = self.types.display(inner);

        let arity = match name.name.as_str() {
            "get" | "into_inner" | "take" => 0,
            "set" | "replace" | "update" => 1,
            _ => {
                self.error(
                    codes::E1010,
                    name.span,
                    format!("`Cell[{shown}]` has no method named `{}`", name.name),
                );
                return error;
            }
        };
        if args.len() != arity {
            self.error(
                codes::E2020,
                span,
                format!("`{}` takes {arity} arguments, found {}", name.name, args.len()),
            );
            return error;
        }

        // `set`, `replace` and `update` write through the receiver, and
        // `[CELL-1]` makes that a borrow of it. A borrow needs something to
        // point at: writing into a value that dies at the end of the statement
        // is a mistake with no way to observe it.
        let writes = matches!(name.name.as_str(), "set" | "replace" | "update" | "take");
        if writes && !is_place(&receiver.kind) {
            self.error(
                codes::E2140,
                span,
                format!("`{}` needs a cell to write into, not a temporary", name.name),
            );
            return error;
        }

        // `[CELL-1]` — "`get(self) -> T` is provided only where `T: Copy`".
        // The reason is `[CELL-2]`: a non-`Copy` `T` could only be *handed*
        // out, which would either move the cell's contents away or alias them.
        if name.name.is("get") && !self.types.is_copy(inner) {
            self.sink.emit(
                Diagnostic::error(
                    codes::E2020,
                    name.span,
                    format!("`get` on `Cell[{shown}]` needs `{shown}: Copy`"),
                )
                .help("`c.replace(v)` takes the value out and puts `v` in its place")
                .note("a `Cell` hands out no reference, so a non-`Copy` value can only be replaced [CELL-1]"),
            );
            return error;
        }

        match name.name.as_str() {
            // A load of the field, which is all `[CELL-2]` says it is.
            "get" => Expr {
                ty: inner,
                kind: ExprKind::Field { base: Box::new(receiver), index: 0 },
                span,
            },
            "set" => {
                let value = self.check_expr(&args[0].value, inner);
                Expr {
                    ty: self.common.void,
                    kind: ExprKind::Builtin { which: Builtin::CellSet, args: vec![receiver, value] },
                    span,
                }
            }
            "replace" => {
                let value = self.check_expr(&args[0].value, inner);
                Expr {
                    ty: inner,
                    kind: ExprKind::Builtin {
                        which: Builtin::CellReplace,
                        args: vec![receiver, value],
                    },
                    span,
                }
            }
            "into_inner" => Expr {
                ty: inner,
                kind: ExprKind::Builtin { which: Builtin::CellIntoInner, args: vec![receiver] },
                span,
            },
            "take" => {
                let Some(constructor) = self.standard_associated_capability(
                    inner,
                    "std.core.Default",
                    "default",
                ) else {
                    self.sink.emit(
                        Diagnostic::error(
                            codes::E2020,
                            name.span,
                            format!("`take` on `Cell[{shown}]` needs `{shown}: Default`"),
                        )
                        .help("implement `std.core.Default` or use `c.replace(v)` with an explicit replacement")
                        .note("`take` leaves a default value in the cell [CELL-1]"),
                    );
                    return error;
                };
                Expr {
                    ty: inner,
                    kind: ExprKind::Builtin {
                        which: Builtin::CellTake { constructor },
                        args: vec![receiver],
                    },
                    span,
                }
            }
            // `update(f)` is `set(f(get()))`, and it needs the receiver twice —
            // once to read the old value and once to store the new one. A HIR
            // expression cannot be duplicated, so the pair travels to lowering,
            // which has a `Place` and can use it as often as it likes.
            _ => {
                let constructor = if self.types.is_copy(inner) {
                    None
                } else {
                    self.standard_associated_capability(
                        inner,
                        "std.core.Default",
                        "default",
                    )
                };
                if !self.types.is_copy(inner) && constructor.is_none() {
                    self.sink.emit(
                        Diagnostic::error(
                            codes::E2020,
                            name.span,
                            format!(
                                "`update` on `Cell[{shown}]` needs `{shown}: Copy` or `{shown}: Default`"
                            ),
                        )
                        .help("implement `std.core.Default` or use `replace` with an explicit value")
                        .note("the Default arm keeps the cell initialized while the callback runs [CELL-1]"),
                    );
                    return error;
                }
                let fn_ty = self.types.intern(TyKind::Fn {
                    latebound: false,
                    params: vec![FnParam { ty: inner, mode: FnParamMode::Borrow }],
                    ret: inner,
                });
                let f = self.check_expr(&args[0].value, fn_ty);
                Expr {
                    ty: self.common.void,
                    kind: ExprKind::Builtin {
                        which: match constructor {
                            Some(constructor) => Builtin::CellUpdateDefault { constructor },
                            None => Builtin::CellUpdate,
                        },
                        args: vec![receiver, f],
                    },
                    span,
                }
            }
        }
    }

    /// `[CELL-5]`, `[CELL-6]` — the methods on `RefCell[T]`.
    ///
    /// All take `self`, a shared borrow, and `borrow`/`borrow_mut` mutate the
    /// borrow state (the counter plus the conflicting location). Like `Cell`,
    /// the cell is an ordinary struct here, so its methods are found by the
    /// side table rather than in `self.methods` (ADR-019: compiler-known and
    /// deliberately not rebuilt on `[UNS-10]`'s separate `UnsafeCell`).
    ///
    /// `borrow(self) -> Ref[T]` succeeds unless a mutable borrow is active;
    /// `borrow_mut(self) -> RefMut[T]` succeeds unless any borrow is active;
    /// both panic with the conflicting borrow's location (`[CELL-5]`).
    /// `try_borrow`/`try_borrow_mut` return `Option` instead (`[CELL-6]`), and
    /// no profile may make them infallible (`[CELL-6a]`, `[PRF-1]` — the
    /// lowering below has no profile input at all).
    ///
    /// Borrowing needs a cell to point at, so like `Cell.set` a temporary is
    /// `E2140`.
    fn synth_refcell_method(
        &mut self,
        receiver: Expr,
        inner: Ty,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let shown = self.types.display(inner);
        let method = name.name.as_str();
        if !matches!(method, "borrow" | "borrow_mut" | "try_borrow" | "try_borrow_mut") {
            self.error(
                codes::E1010,
                name.span,
                format!("`RefCell[{shown}]` has no method named `{}`", name.name),
            );
            return error;
        }
        if !args.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`{method}` takes 0 arguments, found {}", args.len()),
            );
            return error;
        }
        if !is_place(&receiver.kind) {
            self.error(
                codes::E2140,
                span,
                format!("`{method}` needs a cell to borrow from, not a temporary"),
            );
            return error;
        }
        // The shared borrow of the cell the guard's region rests on
        // (`[CELL-7]`). Lowering writes this down as an `Rvalue::Ref` so
        // `collect_loans` sees it; the counter (not the borrow checker) is
        // what refuses overlapping guards at run time.
        let cell_ref_ty = self.types.intern(TyKind::Ref { mutable: false, inner: receiver.ty });
        let borrowed = Expr {
            ty: cell_ref_ty,
            kind: ExprKind::Ref { place: Box::new(receiver), mutable: false },
            span,
        };
        match method {
            "borrow" => {
                let guard = self.ref_guard_of(inner, false);
                Expr {
                    ty: guard,
                    kind: ExprKind::Builtin { which: Builtin::RefCellBorrow, args: vec![borrowed] },
                    span,
                }
            }
            "borrow_mut" => {
                let guard = self.ref_guard_of(inner, true);
                Expr {
                    ty: guard,
                    kind: ExprKind::Builtin {
                        which: Builtin::RefCellBorrowMut,
                        args: vec![borrowed],
                    },
                    span,
                }
            }
            "try_borrow" => {
                let guard = self.ref_guard_of(inner, false);
                let opt = self.option_of(guard);
                Expr {
                    ty: opt,
                    kind: ExprKind::Builtin {
                        which: Builtin::RefCellTryBorrow,
                        args: vec![borrowed],
                    },
                    span,
                }
            }
            _ => {
                let guard = self.ref_guard_of(inner, true);
                let opt = self.option_of(guard);
                Expr {
                    ty: opt,
                    kind: ExprKind::Builtin {
                        which: Builtin::RefCellTryBorrowMut,
                        args: vec![borrowed],
                    },
                    span,
                }
            }
        }
    }

    /// `[ARN-4]` — construct the growing arena with one initial chunk.
    fn synth_arena_construction(
        &mut self,
        name: ast::Ident,
        generic_args: &[ast::GenericArg],
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let arena = self.arena_ty();
        if name.name.is("fixed") {
            if !generic_args.is_empty() {
                self.error(codes::E2020, span, "`Arena.fixed` takes no type arguments");
            }
            if args.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!("`Arena.fixed` takes one argument, found {}", args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let buffer_ty = self.types.intern(TyKind::Span {
                elem: self.common.u8,
                mutable: true,
            });
            let buffer = self.check_expr(&args[0].value, buffer_ty);
            let fixed = self.fixed_arena_ty();
            let TyKind::Struct(id) = *self.types.kind(fixed) else {
                unreachable!("FixedArena is compiler-known as a struct")
            };
            let used = Expr { ty: self.common.usize, kind: ExprKind::Int(0), span };
            return Expr {
                ty: fixed,
                kind: ExprKind::StructLit { struct_id: id, fields: vec![buffer, used] },
                span,
            };
        }
        if !name.name.is("with_capacity") {
            self.error(
                codes::E1010,
                name.span,
                format!("`Arena` has no associated function `{}` in this phase", name.name),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if !generic_args.is_empty() {
            self.error(codes::E2020, span, "`Arena.with_capacity` takes no type arguments");
        }
        if args.len() != 1 {
            self.error(
                codes::E2020,
                span,
                format!("`Arena.with_capacity` takes one argument, found {}", args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let capacity = self.check_expr(&args[0].value, self.common.usize);
        Expr {
            ty: arena,
            kind: ExprKind::Builtin {
                which: Builtin::ArenaWithCapacity,
                args: vec![capacity],
            },
            span,
        }
    }

    /// The element type of the public `std.collections.ArenaArray[T]`
    /// instantiation, if `ty` is one. The source declaration owns the public
    /// identity and representation; this recognition supplies only the
    /// operations whose fixed-storage semantics require compiler lowering.
    fn arena_array_element(&self, ty: Ty) -> Option<Ty> {
        let TyKind::Struct(id) = *self.types.kind(ty) else { return None };
        let (origin, args) = self.types.struct_def(id).origin.as_ref()?;
        if origin.as_str() == "std.collections.ArenaArray" && args.len() == 1 {
            Some(args[0])
        } else {
            None
        }
    }

    fn arena_array_iterator(&self, ty: Ty) -> Option<(Ty, bool)> {
        let TyKind::Struct(id) = *self.types.kind(ty) else { return None };
        let (origin, args) = self.types.struct_def(id).origin.as_ref()?;
        if args.len() != 1 {
            return None;
        }
        match origin.as_str() {
            "std.collections.ArenaArrayIter" => Some((args[0], false)),
            "std.collections.ArenaArrayIterMut" => Some((args[0], true)),
            _ => None,
        }
    }

    fn span_iterator(&self, ty: Ty) -> Option<(Ty, SpanIteratorKind)> {
        let TyKind::Struct(id) = *self.types.kind(ty) else { return None };
        let (origin, args) = self.types.struct_def(id).origin.as_ref()?;
        if args.len() != 1 {
            return None;
        }
        let kind = match origin.as_str() {
            "std.collections.SpanIter" => SpanIteratorKind::Elements { mutable: false },
            "std.collections.MutSpanIter" => SpanIteratorKind::Elements { mutable: true },
            "std.collections.SpanChunks" => SpanIteratorKind::Chunks { mutable: false },
            "std.collections.MutSpanChunks" => SpanIteratorKind::Chunks { mutable: true },
            _ => return None,
        };
        Some((args[0], kind))
    }

    fn arena_map_parts(&self, ty: Ty) -> Option<(Ty, Ty)> {
        let TyKind::Struct(id) = *self.types.kind(ty) else { return None };
        let (origin, args) = self.types.struct_def(id).origin.as_ref()?;
        if origin.as_str() == "std.collections.ArenaMap" && args.len() == 2 {
            Some((args[0], args[1]))
        } else {
            None
        }
    }

    fn arena_map_iterator(&self, ty: Ty) -> Option<(Ty, Ty)> {
        let TyKind::Struct(id) = *self.types.kind(ty) else { return None };
        let (origin, args) = self.types.struct_def(id).origin.as_ref()?;
        if origin.as_str() == "std.collections.ArenaMapIter" && args.len() == 2 {
            Some((args[0], args[1]))
        } else {
            None
        }
    }

    /// Built-in keys whose `Eq + Hash` semantics are already compiler-known.
    /// The fixed map may choose a linear implementation, but its public bound
    /// remains the ordinary Map bound. Every other key must carry explicit
    /// `Eq` and `Hash` implementations; it is never compared bytewise or by a
    /// guessed structural rule.
    fn has_builtin_eq_hash(&self, ty: Ty) -> bool {
        match self.types.kind(ty) {
            TyKind::Bool | TyKind::Char | TyKind::Int(_) | TyKind::Uint(_) | TyKind::Str => true,
            TyKind::Range(_) => true,
            TyKind::Enum(id) => self.types.enum_def(*id).is_unit_only(),
            _ => false,
        }
    }

    /// `[HASH-4]` — select the equality operation for one ArenaMap key. The
    /// public requirement is always `Eq + Hash`, even though this fixed-size
    /// implementation is permitted to use a linear search and therefore does
    /// not need to cache hash buckets. Primitive/range/unit-enum equality is
    /// compiler-known; every other concrete key carries the exact `Eq.eq`
    /// implementation into HIR rather than falling back to byte comparison.
    fn arena_map_equality(&mut self, ty: Ty, span: Span) -> Option<Option<DefId>> {
        if self.has_builtin_eq_hash(ty) {
            return Some(None);
        }

        let eq = Symbol::intern("std.core.Eq");
        let hash = Symbol::intern("std.collections.Hash");
        let has_eq = self.implements(ty, eq);
        let has_hash = self.implements(ty, hash);
        if !has_eq || !has_hash {
            let requirement = match (has_eq, has_hash) {
                (false, false) => "Eq + Hash",
                (false, true) => "Eq",
                (true, false) => "Hash",
                (true, true) => unreachable!(),
            };
            self.sink.emit(
                Diagnostic::error(
                    codes::E2040,
                    span,
                    format!(
                        "`{}` does not implement `{requirement}`, which ArenaMap keys require",
                        self.types.display(ty)
                    ),
                )
                .help("implement the missing standard interface for the key type")
                .note("ArenaMap[K, V] requires K: Eq + Hash [ARN-5d, HASH-4]"),
            );
            return None;
        }

        // An opaque generic key is checked through its bounds now and is
        // rechecked with its concrete type when the generic body is
        // instantiated. Only that concrete pass needs a callable DefId.
        if matches!(self.types.kind(ty), TyKind::Param { .. }) {
            return Some(None);
        }

        let method = Symbol::intern("eq");
        self.methods.get(&(ty, method)).map(|entry| Some(entry.def)).or_else(|| {
            // `check_implementations` has already emitted the precise missing
            // member/signature diagnostic. Return an error expression here so
            // lowering never guesses equality for an incomplete impl.
            None
        })
    }

    fn instantiate_named_generic(&mut self, name: &str, args: &[Ty], span: Span) -> Ty {
        let name = Symbol::intern(name);
        let Some(decl) = self.generic_structs.get(&name).cloned() else {
            self.error(codes::E1010, span, format!("cannot find standard type `{name}`"));
            return self.common.error;
        };
        self.instantiate_struct(name, &decl, args, span)
    }

    fn capacity_error_ty(&self) -> Option<Ty> {
        self.named_types
            .get(&Symbol::intern("std.collections.CapacityError"))
            .copied()
    }

    /// `[ARN-5a]` — the only operation that obtains backing storage. The
    /// result contains an explicit `ref Arena` anchor, and the builtin call's
    /// first operand is that same borrow, so both the structural view checker
    /// and the region/loan analysis see the provenance rather than inferring
    /// it from a raw pointer.
    fn synth_arena_array_construction(
        &mut self,
        array: Ty,
        elem: Ty,
        name: ast::Ident,
        generic_args: &[ast::GenericArg],
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        if !name.name.is("with_capacity") {
            self.error(
                codes::E1010,
                name.span,
                format!(
                    "`{}` has no associated function `{}`",
                    self.types.display(array),
                    name.name
                ),
            );
            return error;
        }
        if !generic_args.is_empty() {
            self.error(codes::E2020, span, "`ArenaArray.with_capacity` takes no type arguments");
            return error;
        }
        if args.len() != 2 {
            self.error(
                codes::E2020,
                span,
                format!(
                    "`ArenaArray.with_capacity` takes 2 arguments, found {}",
                    args.len()
                ),
            );
            return error;
        }
        if self.types.needs_drop(elem) {
            let shown = self.types.display(elem);
            self.sink.emit_classified(
                Diagnostic::error(
                    codes::E3090,
                    span,
                    format!("`ArenaArray[{shown}]` requires `!needs_drop({shown})`"),
                )
                .primary_label("Arena-backed collection elements are not dropped individually")
                .help("use an owned `Array[T]` when elements need destruction")
                .note("Arena rewind/drop reclaims bytes without an element destructor walk [ARN-5e]"),
            );
            return error;
        }
        let arena_ty = self.arena_ty();
        let arena = self.check_expr(&args[0].value, arena_ty);
        if arena.ty == self.common.error {
            return error;
        }
        if !is_place(&arena.kind) {
            self.error(
                codes::E2140,
                args[0].value.span,
                "an Arena-backed container needs an Arena variable to borrow",
            );
            return error;
        }
        let arena_ref_ty = self.types.intern(TyKind::Ref { mutable: false, inner: arena_ty });
        let borrowed = Expr {
            ty: arena_ref_ty,
            kind: ExprKind::Ref { place: Box::new(arena), mutable: false },
            span: args[0].value.span,
        };
        let capacity = self.check_expr(&args[1].value, self.common.usize);
        Expr {
            ty: array,
            kind: ExprKind::Builtin {
                which: Builtin::ArenaArrayWithCapacity { elem, array },
                args: vec![borrowed, capacity],
            },
            span,
        }
    }

    fn arena_collection_receiver(&mut self, receiver: Expr, mutable: bool, span: Span) -> Expr {
        if mutable {
            return self.pass_receiver(receiver, Mode::Mut, span);
        }
        if !is_place(&receiver.kind) {
            self.error(codes::E2140, span, "this Arena collection operation needs a value to borrow");
            return receiver;
        }
        let ty = self.types.intern(TyKind::Ref { mutable: false, inner: receiver.ty });
        Expr {
            ty,
            kind: ExprKind::Ref { place: Box::new(receiver), mutable: false },
            span,
        }
    }

    /// `[ARN-5b]`–`[ARN-5g]` — the fixed-capacity Array surface. Operations
    /// are lowered over the already-reserved pointer/length/capacity fields;
    /// no method carries an Arena operand, which makes a later cursor mutation
    /// impossible by construction.
    fn synth_arena_array_method(
        &mut self,
        receiver: Expr,
        elem: Ty,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let method = name.name.as_str();
        let arity = match method {
            "len" | "capacity" | "is_empty" | "clear" | "iter" | "iter_mut" => 0,
            "get" | "get_mut" | "push" | "remove" => 1,
            "insert" => 2,
            _ => {
                self.error(
                    codes::E1010,
                    name.span,
                    format!(
                        "`{}` has no method named `{}`",
                        self.types.display(receiver.ty),
                        name.name
                    ),
                );
                return error;
            }
        };
        if args.len() != arity {
            self.error(
                codes::E2020,
                span,
                format!("`{method}` takes {arity} arguments, found {}", args.len()),
            );
            return error;
        }

        match method {
            "len" | "capacity" | "is_empty" => {
                let field = if method == "capacity" { 3 } else { 2 };
                let value = Expr {
                    ty: self.common.usize,
                    kind: ExprKind::Field { base: Box::new(receiver), index: field },
                    span,
                };
                if method == "is_empty" {
                    return Expr {
                        ty: self.common.bool_,
                        kind: ExprKind::Binary {
                            op: BinOp::Eq,
                            lhs: Box::new(value),
                            rhs: Box::new(Expr {
                                ty: self.common.usize,
                                kind: ExprKind::Int(0),
                                span,
                            }),
                        },
                        span,
                    };
                }
                value
            }
            "get" | "get_mut" => {
                let mutable = method == "get_mut";
                let receiver = self.arena_collection_receiver(receiver, mutable, span);
                let index = self.check_expr(&args[0].value, self.common.usize);
                let reference = self.types.intern(TyKind::Ref { mutable, inner: elem });
                let result = self.option_of(reference);
                Expr {
                    ty: result,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaArrayGet { elem, mutable },
                        args: vec![receiver, index],
                    },
                    span,
                }
            }
            "push" => {
                let receiver = self.arena_collection_receiver(receiver, true, span);
                let value = self.check_expr(&args[0].value, elem);
                let Some(capacity_error) = self.capacity_error_ty() else {
                    self.error(codes::E1010, span, "`std.collections.CapacityError` is unavailable");
                    return error;
                };
                let result = self.result_of(self.common.void, capacity_error);
                Expr {
                    ty: result,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaArrayPush { elem },
                        args: vec![receiver, value],
                    },
                    span,
                }
            }
            "insert" => {
                let receiver = self.arena_collection_receiver(receiver, true, span);
                let index = self.check_expr(&args[0].value, self.common.usize);
                let value = self.check_expr(&args[1].value, elem);
                let Some(capacity_error) = self.capacity_error_ty() else {
                    self.error(codes::E1010, span, "`std.collections.CapacityError` is unavailable");
                    return error;
                };
                let result = self.result_of(self.common.void, capacity_error);
                Expr {
                    ty: result,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaArrayInsert { elem },
                        args: vec![receiver, index, value],
                    },
                    span,
                }
            }
            "remove" => {
                let receiver = self.arena_collection_receiver(receiver, true, span);
                let index = self.check_expr(&args[0].value, self.common.usize);
                let result = self.option_of(elem);
                Expr {
                    ty: result,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaArrayRemove { elem },
                        args: vec![receiver, index],
                    },
                    span,
                }
            }
            "clear" => {
                let receiver = self.arena_collection_receiver(receiver, true, span);
                Expr {
                    ty: self.common.void,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaArrayClear,
                        args: vec![receiver],
                    },
                    span,
                }
            }
            "iter" | "iter_mut" => {
                let mutable = method == "iter_mut";
                let receiver = self.arena_collection_receiver(receiver, mutable, span);
                let iterator = self.instantiate_named_generic(
                    if mutable {
                        "std.collections.ArenaArrayIterMut"
                    } else {
                        "std.collections.ArenaArrayIter"
                    },
                    &[elem],
                    span,
                );
                let TyKind::Struct(iterator_id) = *self.types.kind(iterator) else {
                    return error;
                };
                Expr {
                    ty: iterator,
                    kind: ExprKind::StructLit {
                        struct_id: iterator_id,
                        fields: vec![
                            receiver,
                            Expr {
                                ty: self.common.usize,
                                kind: ExprKind::Int(0),
                                span,
                            },
                        ],
                    },
                    span,
                }
            }
            _ => unreachable!("all ArenaArray methods were classified above"),
        }
    }

    fn synth_arena_array_iterator_method(
        &mut self,
        receiver: Expr,
        elem: Ty,
        mutable: bool,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        if !name.name.is("next") {
            self.error(
                codes::E1010,
                name.span,
                format!(
                    "`{}` has no method named `{}`",
                    self.types.display(receiver.ty),
                    name.name
                ),
            );
            return error;
        }
        if !args.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`next` takes no arguments, found {}", args.len()),
            );
            return error;
        }
        let receiver = self.pass_receiver(receiver, Mode::Mut, span);
        let item = self.types.intern(TyKind::Ref { mutable, inner: elem });
        let result = self.option_of(item);
        Expr {
            ty: result,
            kind: ExprKind::Builtin {
                which: Builtin::ArenaArrayIterNext { elem, mutable },
                args: vec![receiver],
            },
            span,
        }
    }

    fn synth_span_iterator_method(
        &mut self,
        receiver: Expr,
        elem: Ty,
        kind: SpanIteratorKind,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        if !name.name.is("next") {
            self.error(
                codes::E1010,
                name.span,
                format!(
                    "`{}` has no method named `{}`",
                    self.types.display(receiver.ty),
                    name.name
                ),
            );
            return error;
        }
        if !args.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`next` takes no arguments, found {}", args.len()),
            );
            return error;
        }
        self.reject_readonly_write(&receiver, span);
        let through_shared_ref = self.reject_write_through_shared_ref(&receiver, span);
        if !through_shared_ref {
            self.reject_borrowed_parameter_write(&receiver, span, false);
        }
        if !is_place(&receiver.kind) {
            self.error(codes::E2140, span, "`next` needs an iterator variable to advance");
            return error;
        }
        let (item, which) = match kind {
            SpanIteratorKind::Elements { mutable } => (
                self.types.intern(TyKind::Ref { mutable, inner: elem }),
                Builtin::SpanIterNext { elem, mutable },
            ),
            SpanIteratorKind::Chunks { mutable } => (
                self.types.intern(TyKind::Span { elem, mutable }),
                Builtin::SpanChunksNext { elem, mutable },
            ),
        };
        let result = self.option_of(item);
        Expr {
            ty: result,
            kind: ExprKind::Builtin { which, args: vec![receiver] },
            span,
        }
    }

    fn synth_arena_map_construction(
        &mut self,
        map: Ty,
        key: Ty,
        value: Ty,
        name: ast::Ident,
        generic_args: &[ast::GenericArg],
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        if !name.name.is("with_capacity") {
            self.error(
                codes::E1010,
                name.span,
                format!(
                    "`{}` has no associated function `{}`",
                    self.types.display(map),
                    name.name
                ),
            );
            return error;
        }
        if !generic_args.is_empty() {
            self.error(codes::E2020, span, "`ArenaMap.with_capacity` takes no type arguments");
            return error;
        }
        if args.len() != 2 {
            self.error(
                codes::E2020,
                span,
                format!("`ArenaMap.with_capacity` takes 2 arguments, found {}", args.len()),
            );
            return error;
        }
        if self.types.is_view(key) || self.types.is_view(value) {
            self.error(
                codes::E2130,
                span,
                "an ArenaMap cannot store a view key or value",
            );
            return error;
        }
        if self.types.needs_drop(key) || self.types.needs_drop(value) {
            self.sink.emit_classified(
                Diagnostic::error(
                    codes::E3090,
                    span,
                    format!(
                        "`ArenaMap[{}, {}]` requires drop-free keys and values",
                        self.types.display(key),
                        self.types.display(value)
                    ),
                )
                .primary_label("Arena-backed map entries are not dropped individually")
                .help("use an owned `Map[K, V]` when keys or values need destruction")
                .note("Arena rewind/drop reclaims bytes without an entry destructor walk [ARN-5e]"),
            );
            return error;
        }
        if self.arena_map_equality(key, span).is_none() {
            return error;
        }
        let arena_ty = self.arena_ty();
        let arena = self.check_expr(&args[0].value, arena_ty);
        if arena.ty == self.common.error {
            return error;
        }
        if !is_place(&arena.kind) {
            self.error(
                codes::E2140,
                args[0].value.span,
                "an Arena-backed container needs an Arena variable to borrow",
            );
            return error;
        }
        let arena_ref_ty = self.types.intern(TyKind::Ref { mutable: false, inner: arena_ty });
        let borrowed = Expr {
            ty: arena_ref_ty,
            kind: ExprKind::Ref { place: Box::new(arena), mutable: false },
            span: args[0].value.span,
        };
        let capacity = self.check_expr(&args[1].value, self.common.usize);
        Expr {
            ty: map,
            kind: ExprKind::Builtin {
                which: Builtin::ArenaMapWithCapacity { key, value, map },
                args: vec![borrowed, capacity],
            },
            span,
        }
    }

    fn synth_arena_map_method(
        &mut self,
        receiver: Expr,
        key: Ty,
        value: Ty,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let method = name.name.as_str();
        let arity = match method {
            "len" | "capacity" | "is_empty" | "clear" | "iter" => 0,
            "get" | "get_mut" | "remove" | "contains_key" => 1,
            "insert" => 2,
            _ => {
                self.error(
                    codes::E1010,
                    name.span,
                    format!(
                        "`{}` has no method named `{}`",
                        self.types.display(receiver.ty),
                        name.name
                    ),
                );
                return error;
            }
        };
        if args.len() != arity {
            self.error(
                codes::E2020,
                span,
                format!("`{method}` takes {arity} arguments, found {}", args.len()),
            );
            return error;
        }
        let Some(equals) = self.arena_map_equality(key, span) else {
            return error;
        };
        match method {
            "len" | "capacity" | "is_empty" => {
                let field = if method == "capacity" { 3 } else { 2 };
                let length = Expr {
                    ty: self.common.usize,
                    kind: ExprKind::Field { base: Box::new(receiver), index: field },
                    span,
                };
                if method != "is_empty" {
                    return length;
                }
                Expr {
                    ty: self.common.bool_,
                    kind: ExprKind::Binary {
                        op: BinOp::Eq,
                        lhs: Box::new(length),
                        rhs: Box::new(Expr {
                            ty: self.common.usize,
                            kind: ExprKind::Int(0),
                            span,
                        }),
                    },
                    span,
                }
            }
            "get" | "get_mut" => {
                let mutable = method == "get_mut";
                let receiver = self.arena_collection_receiver(receiver, mutable, span);
                let wanted = self.check_expr(&args[0].value, key);
                let reference = self.types.intern(TyKind::Ref { mutable, inner: value });
                let result = self.option_of(reference);
                Expr {
                    ty: result,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaMapGet { key, value, mutable, equals },
                        args: vec![receiver, wanted],
                    },
                    span,
                }
            }
            "contains_key" => {
                let receiver = self.arena_collection_receiver(receiver, false, span);
                let wanted = self.check_expr(&args[0].value, key);
                Expr {
                    ty: self.common.bool_,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaMapContains { key, equals },
                        args: vec![receiver, wanted],
                    },
                    span,
                }
            }
            "insert" => {
                let receiver = self.arena_collection_receiver(receiver, true, span);
                let key_value = self.check_expr(&args[0].value, key);
                let mapped = self.check_expr(&args[1].value, value);
                let Some(capacity_error) = self.capacity_error_ty() else {
                    self.error(codes::E1010, span, "`std.collections.CapacityError` is unavailable");
                    return error;
                };
                let old = self.option_of(value);
                let result = self.result_of(old, capacity_error);
                Expr {
                    ty: result,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaMapInsert { key, value, equals },
                        args: vec![receiver, key_value, mapped],
                    },
                    span,
                }
            }
            "remove" => {
                let receiver = self.arena_collection_receiver(receiver, true, span);
                let wanted = self.check_expr(&args[0].value, key);
                let result = self.option_of(value);
                Expr {
                    ty: result,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaMapRemove { key, value, equals },
                        args: vec![receiver, wanted],
                    },
                    span,
                }
            }
            "clear" => {
                let receiver = self.arena_collection_receiver(receiver, true, span);
                Expr {
                    ty: self.common.void,
                    kind: ExprKind::Builtin {
                        which: Builtin::ArenaMapClear,
                        args: vec![receiver],
                    },
                    span,
                }
            }
            "iter" => {
                let receiver = self.arena_collection_receiver(receiver, false, span);
                let iterator = self.instantiate_named_generic(
                    "std.collections.ArenaMapIter",
                    &[key, value],
                    span,
                );
                let TyKind::Struct(iterator_id) = *self.types.kind(iterator) else {
                    return error;
                };
                Expr {
                    ty: iterator,
                    kind: ExprKind::StructLit {
                        struct_id: iterator_id,
                        fields: vec![
                            receiver,
                            Expr {
                                ty: self.common.usize,
                                kind: ExprKind::Int(0),
                                span,
                            },
                        ],
                    },
                    span,
                }
            }
            _ => unreachable!("all ArenaMap methods were classified above"),
        }
    }

    fn synth_arena_map_iterator_method(
        &mut self,
        receiver: Expr,
        key: Ty,
        value: Ty,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        if !name.name.is("next") {
            self.error(
                codes::E1010,
                name.span,
                format!(
                    "`{}` has no method named `{}`",
                    self.types.display(receiver.ty),
                    name.name
                ),
            );
            return error;
        }
        if !args.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`next` takes no arguments, found {}", args.len()),
            );
            return error;
        }
        let receiver = self.pass_receiver(receiver, Mode::Mut, span);
        let key_ref = self.types.intern(TyKind::Ref { mutable: false, inner: key });
        let value_ref = self.types.intern(TyKind::Ref { mutable: false, inner: value });
        let item = self.types.intern(TyKind::Tuple(vec![key_ref, value_ref]));
        let result = self.option_of(item);
        Expr {
            ty: result,
            kind: ExprKind::Builtin {
                which: Builtin::ArenaMapIterNext { key, value },
                args: vec![receiver],
            },
            span,
        }
    }

    /// `[ARN-1]`–`[ARN-4]`, `[ARN-7]` — the core operations on a growing
    /// arena. Allocation borrows `self` shared and returns a mutable view tied
    /// to that borrow; reset takes `mut self`, so the ordinary borrow checker
    /// is the mechanism that prevents invalidation of live views.
    fn synth_arena_method(
        &mut self,
        receiver: Expr,
        name: ast::Ident,
        args: &[ast::Arg],
        explicit: &[Ty],
        span: Span,
    ) -> Expr {
        let method = name.name.as_str();
        if method == "alloc_uninit" {
            if explicit.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!(
                        "`Arena.alloc_uninit` takes one type argument, found {}",
                        explicit.len()
                    ),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            if args.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!(
                        "`Arena.alloc_uninit` takes one argument, found {}",
                        args.len()
                    ),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            if !is_place(&receiver.kind) {
                self.error(codes::E2140, span, "arena allocation needs a variable to borrow");
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let inner = explicit[0];
            if self.types.is_view(inner) {
                self.error(
                    codes::E2130,
                    span,
                    "an uninitialized arena span cannot itself contain a view type",
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let count = self.check_expr(&args[0].value, self.common.usize);
            let slot = self.maybe_uninit_of(inner);
            let result = self.types.intern(TyKind::Span { elem: slot, mutable: true });
            let arena_ref = self.types.intern(TyKind::Ref {
                mutable: false,
                inner: receiver.ty,
            });
            let borrowed = Expr {
                ty: arena_ref,
                kind: ExprKind::Ref { place: Box::new(receiver), mutable: false },
                span,
            };
            return Expr {
                ty: result,
                kind: ExprKind::Builtin {
                    which: Builtin::ArenaAllocUninit { elem: inner },
                    args: vec![borrowed, count],
                },
                span,
            };
        }
        if method == "alloc_array" {
            if explicit.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!(
                        "`Arena.alloc_array` takes one type argument, found {}",
                        explicit.len()
                    ),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            if args.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!(
                        "`Arena.alloc_array` takes one argument, found {}",
                        args.len()
                    ),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            if !is_place(&receiver.kind) {
                self.error(codes::E2140, span, "arena allocation needs a variable to borrow");
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let elem = explicit[0];
            if self.types.needs_drop(elem) {
                let shown = self.types.display(elem);
                self.sink.emit_classified(
                    Diagnostic::error(
                        codes::E3090,
                        span,
                        format!(
                            "`{shown}` needs `drop` and cannot be allocated with `Arena.alloc_array`"
                        ),
                    )
                    .primary_label("arena array elements are never dropped individually")
                    .help("use `alloc_uninit` only when you will explicitly manage every initialized value")
                    .note("ordinary `alloc_array` always requires `!needs_drop(T)` [ARN-2, ARN-3]"),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let constructor = if self.types.is_builtin_zeroable(elem) {
                None
            } else {
                self.standard_associated_capability(
                    elem,
                    "std.core.Default",
                    "default",
                )
            };
            if !self.types.is_builtin_zeroable(elem) && constructor.is_none() {
                let shown = self.types.display(elem);
                self.sink.emit(
                    Diagnostic::error(
                        codes::E2040,
                        span,
                        format!(
                            "`{shown}` has no available `Zeroable` or `Default` capability"
                        ),
                    )
                    .help("use `alloc_uninit[T](count)`, write every slot, then assert initialization in `unsafe`")
                    .note("`alloc_array` requires a proven `Zeroable` representation or an implementation of `Default` [ARN-3, ARN-12]"),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let count = self.check_expr(&args[0].value, self.common.usize);
            let result = self.types.intern(TyKind::Span { elem, mutable: true });
            let arena_ref = self.types.intern(TyKind::Ref {
                mutable: false,
                inner: receiver.ty,
            });
            let borrowed = Expr {
                ty: arena_ref,
                kind: ExprKind::Ref { place: Box::new(receiver), mutable: false },
                span,
            };
            return Expr {
                ty: result,
                kind: ExprKind::Builtin {
                    which: match constructor {
                        Some(constructor) => {
                            Builtin::ArenaAllocArrayDefault { elem, constructor }
                        }
                        None => Builtin::ArenaAllocArrayZeroed { elem },
                    },
                    args: vec![borrowed, count],
                },
                span,
            };
        }
        if method == "scope" {
            if !explicit.is_empty() {
                self.error(codes::E2020, span, "`Arena.scope` takes no type arguments");
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_arena_scope(receiver, "Arena", args, span);
        }
        if method == "reset" {
            if !explicit.is_empty() {
                self.error(codes::E2020, span, "`Arena.reset` takes no type arguments");
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            if !args.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`Arena.reset` takes no arguments, found {}", args.len()),
                );
            }
            let receiver = self.pass_receiver(receiver, Mode::Mut, span);
            return Expr {
                ty: self.common.void,
                kind: ExprKind::Builtin { which: Builtin::ArenaReset, args: vec![receiver] },
                span,
            };
        }

        if !matches!(method, "alloc" | "alloc_nodrop") {
            self.error(
                codes::E1010,
                name.span,
                format!("`Arena` has no method named `{}`", name.name),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if explicit.len() > 1 {
            self.error(
                codes::E2020,
                span,
                format!("`Arena.{method}` takes one type argument, found {}", explicit.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if args.len() != 1 {
            self.error(
                codes::E2020,
                span,
                format!("`Arena.{method}` takes one argument, found {}", args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if !is_place(&receiver.kind) {
            self.error(codes::E2140, span, "arena allocation needs a variable to borrow");
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }

        let value = {
            let value = match explicit.first().copied() {
                Some(ty) => self.check_expr(&args[0].value, ty),
                None => self.synth_committed(&args[0].value),
            };
            self.read_through(value)
        };
        if value.ty == self.common.error {
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        self.reject_stored_view(value.ty, args[0].value.span, "an arena allocation");
        if method == "alloc" && self.types.needs_drop(value.ty) {
            let shown = self.types.display(value.ty);
            self.sink.emit_classified(
                Diagnostic::error(
                    codes::E3090,
                    args[0].value.span,
                    format!("`{shown}` needs `drop` and cannot be allocated with `Arena.alloc`"),
                )
                .primary_label("this value's destructor would never run")
                .help("use `alloc_nodrop` only when intentionally skipping `drop`, or allocate an owned value normally")
                .note("arena values are reclaimed as bytes and are not dropped individually (ARN-2, ARN-3)"),
            );
        }

        let arena_ref = self.types.intern(TyKind::Ref { mutable: false, inner: receiver.ty });
        let borrowed = Expr {
            ty: arena_ref,
            kind: ExprKind::Ref { place: Box::new(receiver), mutable: false },
            span,
        };
        let result = self.types.intern(TyKind::Ref { mutable: true, inner: value.ty });
        Expr {
            ty: result,
            kind: ExprKind::Builtin {
                which: Builtin::ArenaAlloc { elem: value.ty },
                args: vec![borrowed, value],
            },
            span,
        }
    }

    /// `[ARN-4]`, `[ARN-7]` — allocation and rewind for a fixed arena. The
    /// semantic shape matches a growing Arena, but exhaustion panics instead
    /// of allocating another chunk and drop owns no runtime storage.
    fn synth_fixed_arena_method(
        &mut self,
        receiver: Expr,
        name: ast::Ident,
        args: &[ast::Arg],
        explicit: &[Ty],
        span: Span,
    ) -> Expr {
        let method = name.name.as_str();
        if method == "reset" {
            if !explicit.is_empty() {
                self.error(codes::E2020, span, "`FixedArena.reset` takes no type arguments");
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            if !args.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`FixedArena.reset` takes no arguments, found {}", args.len()),
                );
            }
            let receiver = self.pass_receiver(receiver, Mode::Mut, span);
            return Expr {
                ty: self.common.void,
                kind: ExprKind::Builtin {
                    which: Builtin::FixedArenaReset,
                    args: vec![receiver],
                },
                span,
            };
        }
        if !matches!(method, "alloc" | "alloc_nodrop") {
            self.error(
                codes::E1010,
                name.span,
                format!("`FixedArena` has no method named `{}`", name.name),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if explicit.len() > 1 {
            self.error(
                codes::E2020,
                span,
                format!("`FixedArena.{method}` takes one type argument, found {}", explicit.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if args.len() != 1 {
            self.error(
                codes::E2020,
                span,
                format!("`FixedArena.{method}` takes one argument, found {}", args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if !is_place(&receiver.kind) {
            self.error(codes::E2140, span, "fixed-arena allocation needs a variable to borrow");
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let value = {
            let value = match explicit.first().copied() {
                Some(ty) => self.check_expr(&args[0].value, ty),
                None => self.synth_committed(&args[0].value),
            };
            self.read_through(value)
        };
        if value.ty == self.common.error {
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        self.reject_stored_view(value.ty, args[0].value.span, "an arena allocation");
        if method == "alloc" && self.types.needs_drop(value.ty) {
            let shown = self.types.display(value.ty);
            self.sink.emit_classified(
                Diagnostic::error(
                    codes::E3090,
                    args[0].value.span,
                    format!("`{shown}` needs `drop` and cannot be allocated with `FixedArena.alloc`"),
                )
                .primary_label("this value's destructor would never run")
                .help("use `alloc_nodrop` only when intentionally skipping `drop`, or allocate an owned value normally")
                .note("arena values are reclaimed as bytes and are not dropped individually (ARN-2, ARN-3)"),
            );
        }
        let arena_ref = self.types.intern(TyKind::Ref { mutable: false, inner: receiver.ty });
        let borrowed = Expr {
            ty: arena_ref,
            kind: ExprKind::Ref { place: Box::new(receiver), mutable: false },
            span,
        };
        let result = self.types.intern(TyKind::Ref { mutable: true, inner: value.ty });
        Expr {
            ty: result,
            kind: ExprKind::Builtin {
                which: Builtin::FixedArenaAlloc { elem: value.ty },
                args: vec![borrowed, value],
            },
            span,
        }
    }

    fn synth_arena_scope(
        &mut self,
        receiver: Expr,
        owner: &str,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        if !args.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`{owner}.scope` takes no arguments, found {}", args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if !is_place(&receiver.kind) {
            self.error(codes::E2140, span, "an arena scope needs a parent variable to borrow");
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let parent = self.pass_receiver(receiver, Mode::Mut, span);
        let scoped = self.scoped_arena_ty();
        Expr {
            ty: scoped,
            kind: ExprKind::Builtin {
                which: Builtin::ArenaScope { scoped },
                args: vec![parent],
            },
            span,
        }
    }

    /// `[ARN-6]` — a scope allocates from the same stable chunk chain and may
    /// create another scope. Its drop rewinds to its captured mark; no public
    /// reset is exposed because scope exit is the reclaim boundary.
    fn synth_scoped_arena_method(
        &mut self,
        receiver: Expr,
        name: ast::Ident,
        args: &[ast::Arg],
        explicit: &[Ty],
        span: Span,
    ) -> Expr {
        let method = name.name.as_str();
        if method == "scope" {
            if !explicit.is_empty() {
                self.error(codes::E2020, span, "`ScopedArena.scope` takes no type arguments");
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.synth_arena_scope(receiver, "ScopedArena", args, span);
        }
        if !matches!(method, "alloc" | "alloc_nodrop") {
            self.error(
                codes::E1010,
                name.span,
                format!("`ScopedArena` has no method named `{}`", name.name),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if explicit.len() > 1 {
            self.error(
                codes::E2020,
                span,
                format!("`ScopedArena.{method}` takes one type argument, found {}", explicit.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if args.len() != 1 {
            self.error(
                codes::E2020,
                span,
                format!("`ScopedArena.{method}` takes one argument, found {}", args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if !is_place(&receiver.kind) {
            self.error(codes::E2140, span, "scoped-arena allocation needs a variable to borrow");
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let value = {
            let value = match explicit.first().copied() {
                Some(ty) => self.check_expr(&args[0].value, ty),
                None => self.synth_committed(&args[0].value),
            };
            self.read_through(value)
        };
        if value.ty == self.common.error {
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        self.reject_stored_view(value.ty, args[0].value.span, "an arena allocation");
        if method == "alloc" && self.types.needs_drop(value.ty) {
            let shown = self.types.display(value.ty);
            self.sink.emit_classified(
                Diagnostic::error(
                    codes::E3090,
                    args[0].value.span,
                    format!("`{shown}` needs `drop` and cannot be allocated with `ScopedArena.alloc`"),
                )
                .primary_label("this value's destructor would never run")
                .help("use `alloc_nodrop` only when intentionally skipping `drop`, or allocate an owned value normally")
                .note("arena values are reclaimed as bytes and are not dropped individually (ARN-2, ARN-3)"),
            );
        }
        let arena_ref = self.types.intern(TyKind::Ref { mutable: false, inner: receiver.ty });
        let borrowed = Expr {
            ty: arena_ref,
            kind: ExprKind::Ref { place: Box::new(receiver), mutable: false },
            span,
        };
        let result = self.types.intern(TyKind::Ref { mutable: true, inner: value.ty });
        Expr {
            ty: result,
            kind: ExprKind::Builtin {
                which: Builtin::ScopedArenaAlloc { elem: value.ty },
                args: vec![borrowed, value],
            },
            span,
        }
    }

    /// `[SPN-2]`–`[SPN-9]` — the methods on `Span[T]` and `MutSpan[T]`.
    /// The four named iterator/chunk identities are declared by
    /// `std.collections`; synthesis here is temporary implementation machinery
    /// for their source-bounded construction and deterministic lowering.
    fn synth_span_method(
        &mut self,
        receiver: Expr,
        elem: Ty,
        mutable: bool,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        if mutable
            && self.maybe_uninit_inner(elem).is_some()
            && matches!(name.name.as_str(), "write_at" | "assume_init")
        {
            let inner = self.maybe_uninit_inner(elem).expect("checked above");
            return self.synth_maybe_uninit_span_method(receiver, inner, name, args, span);
        }
        let usize_ty = self.common.usize;
        if mutable && name.name.is("reborrow") {
            if !args.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`reborrow` takes 0 arguments, found {}", args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let receiver = if is_place(&receiver.kind) {
                self.pass_receiver(receiver, Mode::Mut, span)
            } else if self.viewed_place(&receiver) {
                receiver
            } else {
                self.error(
                    codes::E2140,
                    span,
                    "`reborrow` needs a mutable span variable or a view of a mutable place",
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            };
            let view = self.types.intern(TyKind::Span { elem, mutable: true });
            return Expr {
                ty: view,
                kind: ExprKind::Builtin {
                    which: Builtin::SpanReborrow,
                    args: vec![receiver],
                },
                span,
            };
        }
        if matches!(name.name.as_str(), "iter" | "iter_mut") {
            if !args.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes 0 arguments, found {}", name.name, args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let wants_mut = name.name.is("iter_mut");
            if wants_mut && !mutable {
                self.error(codes::E2020, name.span, "`Span` has no method `iter_mut`");
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let source = if mutable {
                self.reborrow_span(receiver, elem, wants_mut, span)
            } else {
                receiver
            };
            let iterator = self.instantiate_named_generic(
                if wants_mut {
                    "std.collections.MutSpanIter"
                } else {
                    "std.collections.SpanIter"
                },
                &[elem],
                span,
            );
            let TyKind::Struct(iterator_id) = *self.types.kind(iterator) else {
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            };
            return Expr {
                ty: iterator,
                kind: ExprKind::StructLit {
                    struct_id: iterator_id,
                    fields: vec![
                        source,
                        Expr { ty: usize_ty, kind: ExprKind::Int(0), span },
                    ],
                },
                span,
            };
        }
        if matches!(name.name.as_str(), "chunks" | "chunks_mut") {
            if args.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes 1 argument, found {}", name.name, args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let wants_mut = name.name.is("chunks_mut");
            if wants_mut && !mutable {
                self.error(codes::E2020, name.span, "`Span` has no method `chunks_mut`");
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let source = if mutable {
                self.reborrow_span(receiver, elem, wants_mut, span)
            } else {
                receiver
            };
            let width = self.check_expr(&args[0].value, usize_ty);
            let iterator = self.instantiate_named_generic(
                if wants_mut {
                    "std.collections.MutSpanChunks"
                } else {
                    "std.collections.SpanChunks"
                },
                &[elem],
                span,
            );
            return Expr {
                ty: iterator,
                kind: ExprKind::Builtin {
                    which: Builtin::SpanChunksNew { iterator, mutable: wants_mut },
                    args: vec![source, width],
                },
                span,
            };
        }
        if matches!(name.name.as_str(), "as_ptr" | "as_mut_ptr") {
            if !args.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes 0 arguments, found {}", name.name, args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let wants_mut = name.name.is("as_mut_ptr");
            if wants_mut && !mutable {
                self.error(codes::E2020, name.span, "`Span` has no method `as_mut_ptr`");
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let receiver = if wants_mut {
                if is_place(&receiver.kind) {
                    self.pass_receiver(receiver, Mode::Mut, span)
                } else if self.viewed_place(&receiver) {
                    receiver
                } else {
                    self.error(
                        codes::E2140,
                        span,
                        "`as_mut_ptr` needs a mutable span variable or a view of a mutable place",
                    );
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                }
            } else {
                receiver
            };
            let pointer = self.types.intern(TyKind::Ptr { mutable: wants_mut, inner: elem });
            return Expr {
                ty: pointer,
                kind: ExprKind::Builtin {
                    which: Builtin::SpanAsPtr { mutable: wants_mut },
                    args: vec![receiver],
                },
                span,
            };
        }
        if name.name.is("split_at") {
            if args.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!("`split_at` takes 1 argument, found {}", args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let index = self.check_expr(&args[0].value, usize_ty);
            let receiver = if mutable && is_place(&receiver.kind) {
                self.pass_receiver(receiver, Mode::Mut, span)
            } else if mutable && !self.viewed_place(&receiver) {
                self.error(
                    codes::E2140,
                    span,
                    "`split_at` needs a mutable span variable or a view of a mutable place",
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            } else {
                receiver
            };
            let view = self.types.intern(TyKind::Span { elem, mutable });
            let pair = self.types.intern(TyKind::Tuple(vec![view, view]));
            return Expr {
                ty: pair,
                kind: ExprKind::Builtin {
                    which: Builtin::SpanSplitAt { elem, pair, mutable },
                    args: vec![receiver, index],
                },
                span,
            };
        }
        let (which, arity, ret) = if name.name.is("len") {
            (Builtin::SpanLen, 0, usize_ty)
        } else if name.name.is("get") {
            // `[SPN-2]` — "`get(i) -> Option[ref T]` is the checked-without-
            // panic form". A `ref` into the view, so the region travels.
            let inner = self.types.intern(TyKind::Ref { mutable, inner: elem });
            (Builtin::SpanGet, 1, self.option_of(inner))
        } else if name.name.is("get_unchecked") {
            if !self.in_unsafe {
                self.sink.emit(
                    Diagnostic::error(
                        codes::E3100,
                        span,
                        "`get_unchecked` needs an `unsafe` block",
                    )
                    .help("`get(i)` returns an `Option`, and `s[i]` is bounds-checked")
                    .note("`[UNS-1]` lists the operations that need one"),
                );
            }
            let inner = self.types.intern(TyKind::Ref { mutable, inner: elem });
            (Builtin::SpanGetUnchecked, 1, inner)
        } else if name.name.is("is_empty") {
            (Builtin::SpanLen, 0, self.common.bool_)
        } else {
            let shown = self.types.display(receiver.ty);
            self.error(
                codes::E2020,
                name.span,
                format!("`{shown}` has no method `{}` in this phase", name.name),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };

        if args.len() != arity {
            self.error(
                codes::E2020,
                span,
                format!("`{}` takes {arity} arguments, found {}", name.name, args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let mut checked = vec![receiver];
        for arg in args {
            checked.push(self.check_expr(&arg.value, usize_ty));
        }
        // `is_empty` is `len() == 0`, written here rather than given a builtin
        // of its own: one fewer thing for the backend to know.
        if name.name.is("is_empty") {
            let len = Expr {
                ty: usize_ty,
                kind: ExprKind::Builtin { which, args: checked },
                span,
            };
            let zero = Expr { ty: usize_ty, kind: ExprKind::Int(0), span };
            return Expr {
                ty: self.common.bool_,
                kind: ExprKind::Binary {
                    op: BinOp::Eq,
                    lhs: Box::new(len),
                    rhs: Box::new(zero),
                },
                span,
            };
        }
        Expr { ty: ret, kind: ExprKind::Builtin { which, args: checked }, span }
    }

    /// `[SPN-5]`, `[SPN-6]` — derive the source view stored by a named
    /// iterator from a `MutSpan` without consuming the caller's view. Shared
    /// iteration takes a shared reborrow; mutable iteration uses the existing
    /// explicit mutable-reborrow operation from `[SPN-3]`.
    fn reborrow_span(&mut self, receiver: Expr, elem: Ty, mutable: bool, span: Span) -> Expr {
        let receiver = if is_place(&receiver.kind) {
            if mutable {
                self.pass_receiver(receiver, Mode::Mut, span)
            } else {
                let ty = self.types.intern(TyKind::Ref { mutable: false, inner: receiver.ty });
                Expr {
                    ty,
                    kind: ExprKind::Ref { place: Box::new(receiver), mutable: false },
                    span,
                }
            }
        } else if self.viewed_place(&receiver) {
            receiver
        } else {
            self.error(
                codes::E2140,
                span,
                "this operation needs a mutable span variable or a view of a mutable place",
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };
        let view = self.types.intern(TyKind::Span { elem, mutable });
        Expr {
            ty: view,
            kind: ExprKind::Builtin {
                which: if mutable { Builtin::SpanReborrow } else { Builtin::SpanSharedReborrow },
                args: vec![receiver],
            },
            span,
        }
    }

    /// `[ARN-9]` — initialization operations on
    /// `MutSpan[MaybeUninit[T]]`. Bounds and exclusivity stay in the ordinary
    /// span/borrow machinery; dedicated builtins preserve the no-drop store
    /// and the explicit unsafe initialization assertion.
    fn synth_maybe_uninit_span_method(
        &mut self,
        receiver: Expr,
        inner: Ty,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let method = name.name.as_str();
        let arity = if method == "write_at" { 2 } else { 0 };
        if args.len() != arity {
            self.error(
                codes::E2020,
                span,
                format!("`{method}` takes {arity} arguments, found {}", args.len()),
            );
            return error;
        }

        if method == "write_at" {
            if !is_place(&receiver.kind) {
                self.error(
                    codes::E2140,
                    span,
                    "`write_at` needs a mutable span variable to write through",
                );
                return error;
            }
            self.reject_borrowed_parameter_write(&receiver, span, false);
            let index = self.check_expr(&args[0].value, self.common.usize);
            let value = self.check_expr(&args[1].value, inner);
            let result = self.types.intern(TyKind::Ref { mutable: true, inner });
            return Expr {
                ty: result,
                kind: ExprKind::Builtin {
                    which: Builtin::MaybeUninitWriteAt { inner },
                    args: vec![receiver, index, value],
                },
                span,
            };
        }

        if !self.in_unsafe {
            self.sink.emit(
                Diagnostic::error(
                    codes::E3100,
                    span,
                    "`assume_init` needs an `unsafe` block",
                )
                .help("initialize every slot with `write_at`, then wrap `assume_init` in `unsafe:`")
                .note("the caller must establish that every exposed element is initialized [ARN-9]"),
            );
        }
        let result = self.types.intern(TyKind::Span { elem: inner, mutable: true });
        Expr {
            ty: result,
            kind: ExprKind::Builtin {
                which: Builtin::MaybeUninitSpanAssumeInit { inner },
                args: vec![receiver],
            },
            span,
        }
    }

    fn synth_vec_method(
        &mut self,
        receiver: Expr,
        elem: Ty,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let is_string = matches!(self.types.kind(elem), TyKind::Uint(UintTy::U8));
        let usize_ty = self.common.usize;
        let str_ty = self.common.str_;

        // `[BRW-5]`'s primary structural repair for two mutable indexed
        // borrows. The receiver is borrowed once; the result carries that
        // provenance as two disjoint `MutSpan`s. This is deliberately not
        // expressed as two independent `as_mut_span` calls, which would be
        // rejected correctly as overlapping mutable borrows.
        if name.name.is("split_at_mut") && !is_string {
            if args.len() != 1 {
                self.error(
                    codes::E2020,
                    span,
                    format!("`split_at_mut` takes 1 argument, found {}", args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            if !is_place(&receiver.kind) {
                self.error(
                    codes::E2140,
                    span,
                    "`split_at_mut` needs an Array variable to borrow",
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let index = self.check_expr(&args[0].value, usize_ty);
            let receiver = self.pass_receiver(receiver, Mode::Mut, span);
            let view = self.types.intern(TyKind::Span { elem, mutable: true });
            let pair = self.types.intern(TyKind::Tuple(vec![view, view]));
            return Expr {
                ty: pair,
                kind: ExprKind::Builtin {
                    which: Builtin::ArraySplitAtMut { elem, pair },
                    args: vec![receiver, index],
                },
                span,
            };
        }

        // `[STD-*]`'s Array table names `as_span` and `as_mut_span`, and Part
        // VII §7 writes `buf.as_mut_span()` in its own worked example. They are
        // the explicit spelling of `[SPN-1]`'s coercion and go through the one
        // producer, so the container is borrowed either way — writing the
        // conversion out does not opt out of the borrow.
        let explicit_view = if name.name.is("as_span") && !is_string {
            Some(false)
        } else if name.name.is("as_mut_span") && !is_string {
            Some(true)
        } else {
            None
        };
        if let Some(mutable) = explicit_view {
            if !args.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes 0 arguments, found {}", name.name, args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            // A view of a temporary would dangle the moment the statement
            // ended, and there would be nothing for the loan to name.
            if !is_place(&receiver.kind) {
                self.error(
                    codes::E2140,
                    span,
                    format!("`{}` needs a variable to point into", name.name),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let view = self.types.intern(TyKind::Span { elem, mutable });
            return self.view_of(
                receiver,
                view,
                mutable,
                Builtin::SpanFrom { mutable },
            );
        }
        // `as_str()` is `[SPN-1]`'s "String to `str`" spelling, and a `str`
        // points into its `String` — so it goes through the one producer too,
        // with a shared borrow. Before this it arrived with no borrow behind
        // it and survived the string's next mutation (D-037).
        if name.name.is("as_str") && is_string {
            if !args.is_empty() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{}` takes 0 arguments, found {}", name.name, args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            if !is_place(&receiver.kind) {
                self.error(
                    codes::E2140,
                    span,
                    format!("`{}` needs a variable to point into", name.name),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            return self.view_of(receiver, str_ty, false, Builtin::StringAsStr);
        }

        let (which, takes, ret) = if name.name.is("len") {
            let which = if is_string { Builtin::StringLen } else { Builtin::ArrayLen };
            (which, None, usize_ty)
        } else if name.name.is("push") && !is_string {
            (Builtin::ArrayPush, Some(elem), self.common.void)
        } else if name.name.is("push_str") && is_string {
            (Builtin::StringPush, Some(str_ty), self.common.void)
        } else if name.name.is("as_str") && is_string {
            (Builtin::StringAsStr, None, str_ty)
        } else {
            let shown = self.types.display(receiver.ty);
            self.error(
                codes::E1010,
                name.span,
                format!("`{shown}` has no method named `{}`", name.name),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };

        let expected_args = usize::from(takes.is_some());
        if args.len() != expected_args {
            self.error(
                codes::E2020,
                span,
                format!("`{}` takes {expected_args} arguments, found {}", name.name, args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }

        // `push` grows the buffer, so it needs the caller's variable, not a
        // copy of it.
        let mutates = matches!(which, Builtin::ArrayPush | Builtin::StringPush);
        let receiver = if mutates {
            self.pass_receiver(receiver, Mode::Mut, span)
        } else {
            receiver
        };

        let mut call_args = vec![receiver];
        if let Some(param_ty) = takes {
            call_args.push(self.check_expr(&args[0].value, param_ty));
        }
        Expr { ty: ret, kind: ExprKind::Builtin { which, args: call_args }, span }
    }

    /// Part IV.11 step 3 — adjust the receiver to the method's declared mode.
    /// `mut self` takes the address, so the method writes through.
    fn pass_receiver(&mut self, receiver: Expr, mode: Mode, span: Span) -> Expr {
        if mode != Mode::Mut {
            return receiver;
        }
        // `[EXC-1]` — a class-valued field receiver is a long-term mutable
        // access at the call boundary, just like passing a scalar class field
        // to a `mut` parameter. The direct method body still opens the access
        // interval for the actual receiver object; the caller-side argument
        // machinery protects the containing class field while its handle is
        // borrowed.
        self.reject_readonly_write_in_mut_argument(&receiver, span);
        let through_shared_ref = self.reject_write_through_shared_ref(&receiver, span);
        if !through_shared_ref {
            self.reject_borrowed_parameter_write(&receiver, span, false);
        }
        if !is_place(&receiver.kind) {
            self.error(
                codes::E2140,
                span,
                "a `mut self` method needs a variable to write through",
            );
            return receiver;
        }
        let ty = self.types.intern(TyKind::Ref { mutable: true, inner: receiver.ty });
        Expr { ty, kind: ExprKind::Ref { place: Box::new(receiver), mutable: true }, span }
    }

    /// `[FN-2a]`, diagnostic shape B10 — a callee's `mut` mode forms a
    /// mutable borrow at the call boundary. This is deliberately narrower
    /// than `[EXP-5]`'s general "value is not a place" diagnostic: naming the
    /// failed parameter-mode adjustment is what gives the caller the useful
    /// structural repair.
    fn emit_mut_argument_place_error(&mut self, span: Span) {
        self.sink.emit_classified(
            Diagnostic::error(codes::E3027, span, "a `mut` argument is not a mutable place")
                .primary_label("this value has no mutable storage for the callee to borrow")
                .help(concat!(
                    "bind the value to a local first, or pass the owner place directly ",
                    "(`f(obj.field)` rather than `f(obj.get_field())`)"
                ))
                .note(concat!(
                    "the callee's declared `mut` mode forms the mutable borrow; ",
                    "the call site does not write a mode [FN-2a]"
                )),
        );
    }

    /// `[TYP-25]` — bind source call arguments to declared parameters.
    ///
    /// The AST preserves arguments in source order because `[EXP-1]` requires
    /// named arguments to be evaluated in that order. HIR, however, must
    /// present operands in the callee's parameter order. Keep those concerns
    /// separate: this pass validates the name/position contract and returns a
    /// source-indexed parameter slot for the later type-checking pass.
    ///
    /// Function values deliberately do not use this helper: they have no
    /// source-level parameter names and remain positional at the callable
    /// boundary (`synth_indirect_call` and friends).
    fn call_argument_slots(
        &mut self,
        callee: Symbol,
        args: &[ast::Arg],
        params: &[(Symbol, Ty, Mode, Span)],
    ) -> Vec<Option<usize>> {
        let mut slots = vec![None; args.len()];
        let mut assigned = vec![false; params.len()];
        let mut next_positional = 0usize;
        let mut named_seen = false;

        for (arg_index, arg) in args.iter().enumerate() {
            let slot = if let Some(name) = arg.name {
                named_seen = true;
                match params.iter().position(|(param, _, _, _)| *param == name.name) {
                    Some(slot) => Some(slot),
                    None => {
                        self.error(
                            codes::E2020,
                            name.span,
                            format!("`{callee}` has no parameter `{}`", name.name),
                        );
                        None
                    }
                }
            } else {
                if named_seen {
                    self.error(
                        codes::E2020,
                        arg.span,
                        "positional arguments must come before named ones",
                    );
                }
                while next_positional < assigned.len() && assigned[next_positional] {
                    next_positional += 1;
                }
                let slot = (next_positional < params.len()).then_some(next_positional);
                next_positional = next_positional.saturating_add(1);
                slot
            };

            let Some(slot) = slot else { continue };
            if assigned[slot] {
                self.error(
                    codes::E1030,
                    arg.value.span,
                    format!("parameter `{}` is given twice", params[slot].0),
                );
                continue;
            }
            assigned[slot] = true;
            slots[arg_index] = Some(slot);
        }
        slots
    }

    /// Return the parameter slots in source evaluation order. Positional
    /// calls need no metadata; keeping the common case as `None` avoids
    /// changing their HIR shape and lowering path.
    fn call_eval_order(slots: &[Option<usize>]) -> Option<Vec<usize>> {
        let order = slots.iter().copied().flatten().collect::<Vec<_>>();
        let identity = order.iter().enumerate().all(|(index, slot)| index == *slot);
        (!identity).then_some(order)
    }

    /// As above, for a method call whose receiver is the first ABI operand.
    fn call_eval_order_with_receiver(slots: &[Option<usize>]) -> Option<Vec<usize>> {
        let order = Self::call_eval_order(slots)?;
        Some(std::iter::once(0).chain(order.into_iter().map(|slot| slot + 1)).collect())
    }

    /// Check direct-call arguments in source order, then return them in the
    /// declaration's parameter order for ABI/MIR lowering.
    fn check_bound_call_arguments(
        &mut self,
        args: &[ast::Arg],
        params: &[(Symbol, Ty, Mode, Span)],
        slots: &[Option<usize>],
    ) -> Vec<Expr> {
        let mut checked: Vec<Option<Expr>> = (0..params.len()).map(|_| None).collect();
        for (arg, slot) in args.iter().zip(slots.iter().copied()) {
            let Some(slot) = slot else { continue };
            let (_, param_ty, mode, _) = params[slot];
            checked[slot] = Some(self.check_argument(&arg.value, param_ty, mode));
        }
        checked.into_iter().flatten().collect()
    }

    /// One argument, in its parameter's mode. A `mut` parameter takes the
    /// address of a place, so the callee writes through to the caller's
    /// variable; everything else is passed by value.
    fn check_argument(&mut self, arg: &ast::Expr, param_ty: Ty, mode: Mode) -> Expr {
        if mode != Mode::Mut {
            return self.check_expr(arg, param_ty);
        }
        // `[SPN-1]`/`[SPN-3]` — a `MutSpan[T]` **is** the mutable access: it
        // carries the pointer, and `[SPN-3]` makes it move-only so there is
        // exactly one. Part VII §7's own example calls
        // `fn normalize(mut xs: MutSpan[f32])` as `normalize(buf)`, so the
        // `mut` mode's place requirement lands on the container being viewed
        // and the view itself is passed by value. Wrapping it in another
        // `ref mut` would be a reference to a reference.
        if matches!(self.types.kind(param_ty), TyKind::Span { mutable: true, .. }) {
            let view = self.check_expr(arg, param_ty);
            self.reject_borrowed_parameter_write(&view, arg.span, false);
            if view.ty != self.common.error && !self.viewed_place(&view) {
                self.emit_mut_argument_place_error(arg.span);
            }
            return view;
        }
        let place = self.check_expr(arg, param_ty);
        // `[MOD-7]` — "passing `h.value` to a `mut` parameter or `mut self`
        // method" is a write.
        self.reject_readonly_write_in_mut_argument(&place, arg.span);
        let through_shared_ref = self.reject_write_through_shared_ref(&place, arg.span);
        if !through_shared_ref {
            self.reject_borrowed_parameter_write(&place, arg.span, false);
        }
        if !is_place(&place.kind) && place.ty != self.common.error {
            self.emit_mut_argument_place_error(arg.span);
            return place;
        }
        let ty = self.types.intern(TyKind::Ref { mutable: true, inner: param_ty });
        let span = place.span;
        Expr { ty, kind: ExprKind::Ref { place: Box::new(place), mutable: true }, span }
    }

    fn synth_struct_literal(
        &mut self,
        id: StructId,
        name: Symbol,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        // `[STR-1]` — the synthesised memberwise constructor "is `pub` iff all
        // fields are `pub` (a `pub(read)` field makes it private to the
        // declaring module, since construction is a write; `[MOD-7]`)".
        self.check_memberwise_constructor(id, span);
        let field_info: Vec<(Symbol, Ty, bool)> = self
            .types
            .struct_def(id)
            .fields
            .iter()
            .map(|f| (f.name, f.ty, f.has_default))
            .collect();
        let ty = self.types.intern(TyKind::Struct(id));

        // `[STR-1]` — positional or named; fields with defaults may be omitted.
        let named = args.iter().any(|a| a.name.is_some());
        let mut values: Vec<Option<Expr>> = (0..field_info.len()).map(|_| None).collect();

        if named {
            for arg in args {
                let Some(arg_name) = arg.name else {
                    self.error(
                        codes::E2020,
                        arg.span,
                        "positional arguments must come before named ones",
                    );
                    continue;
                };
                match field_info.iter().position(|(n, _, _)| *n == arg_name.name) {
                    Some(index) => {
                        let expected = field_info[index].1;
                        values[index] = Some(self.check_expr(&arg.value, expected));
                    }
                    None => {
                        self.error(
                            codes::E2020,
                            arg_name.span,
                            format!("`{name}` has no field `{}`", arg_name.name),
                        );
                    }
                }
            }
        } else {
            if args.len() > field_info.len() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{name}` has {} fields, found {} arguments", field_info.len(), args.len()),
                );
            }
            for (index, arg) in args.iter().enumerate() {
                if index >= field_info.len() {
                    break;
                }
                let expected = field_info[index].1;
                values[index] = Some(self.check_expr(&arg.value, expected));
            }
        }

        let mut fields = Vec::with_capacity(field_info.len());
        for (index, (field_name, field_ty, has_default)) in field_info.iter().enumerate() {
            match values[index].take() {
                Some(expr) => fields.push(expr),
                None => {
                    if !has_default {
                        self.error(
                            codes::E2020,
                            span,
                            format!("field `{field_name}` of `{name}` has no value"),
                        );
                    }
                    // `[STR-2]` — a defaulted field is filled in at the call
                    // site. Phase 0 has no const evaluator, so a zero of the
                    // right type stands in; Phase 4 replaces this.
                    fields.push(self.zero_of(*field_ty, span));
                }
            }
        }

        Expr { ty, kind: ExprKind::StructLit { struct_id: id, fields }, span }
    }

    fn synth_class_constructor(
        &mut self,
        id: ClassId,
        name: Symbol,
        args: &[ast::Arg],
        explicit: &[Ty],
        span: Span,
    ) -> Expr {
        let ty = self.class_ty(id).unwrap_or(self.common.error);
        let (openness, fields, has_base) = {
            let def = self.types.class_def(id);
            (def.openness, def.fields.clone(), def.base.is_some())
        };
        let has_init = self
            .methods
            .contains_key(&(ty, Symbol::intern("init")));

        if !explicit.is_empty() {
            self.error(
                codes::E2020,
                span,
                format!("`{name}` does not take type arguments in this phase"),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if openness == ClassOpenness::Abstract {
            self.error(codes::E2020, span, format!("cannot instantiate abstract class `{name}`"));
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if !has_init && args.len() > fields.len() {
            self.error(
                codes::E2020,
                span,
                format!("`{name}()` has {} fields, found {} arguments", fields.len(), args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        if has_init {
            let init = self
                .methods
                .get(&(ty, Symbol::intern("init")))
                .map(|entry| entry.def)
                .expect("has_init implies a registered constructor");
            let signature_params = self.signatures[init.0 as usize].params.clone();
            let Some((_, _, receiver_mode, _)) = signature_params.first() else {
                self.error(codes::E1010, span, format!("class `{name}` has an invalid `init`"));
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            };
            if *receiver_mode != Mode::Mut {
                self.error(
                    codes::E1010,
                    span,
                    format!("class `{name}` constructor must declare `mut self`"),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            if fields.iter().any(|field| field.has_default) {
                self.error(
                    codes::E1010,
                    span,
                    format!("class construction for `{name}` with defaulted fields is not implemented yet in this phase"),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let params = &signature_params[1..];
            if args.len() != params.len() {
                self.error(
                    codes::E2020,
                    span,
                    format!("`{name}()` takes {} arguments, found {}", params.len(), args.len()),
                );
                return Expr { ty: self.common.error, kind: ExprKind::Error, span };
            }
            let slots = self.call_argument_slots(name, args, params);
            let values = self.check_bound_call_arguments(args, params, &slots);
            return Expr {
                ty,
                kind: ExprKind::ClassNew {
                    class_id: id,
                    init,
                    arg_eval_order: Self::call_eval_order(&slots),
                    args: values,
                },
                span,
            };
        }

        // `[CLS-4]` — memberwise construction does not know how to invoke a
        // base constructor. Derived classes therefore need an explicit init
        // with `super.init(...)`; never publish a partially initialized base.
        if has_base {
            self.error(
                codes::E1010,
                span,
                format!("class `{name}` with a base class requires an explicit `init`"),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }

        let defaults = self.class_default_literals.get(&id).cloned();
        // `[CLS-3]` — a memberwise class constructor follows the same
        // positional/named field binding as a struct constructor. Keep the
        // user-defined `init` path separate: its parameter names and defaults
        // are a function-call contract, not field names.
        let named = args.iter().any(|arg| arg.name.is_some());
        let mut values_by_field: Vec<Option<Expr>> = (0..fields.len()).map(|_| None).collect();
        if named {
            for arg in args {
                let Some(arg_name) = arg.name else {
                    self.error(
                        codes::E2020,
                        arg.span,
                        "positional arguments must come before named ones",
                    );
                    continue;
                };
                let Some(index) = fields.iter().position(|field| field.name == arg_name.name) else {
                    self.error(
                        codes::E2020,
                        arg_name.span,
                        format!("`{name}` has no field `{}`", arg_name.name),
                    );
                    continue;
                };
                let value = self.check_expr(&arg.value, fields[index].ty);
                if values_by_field[index].replace(value).is_some() {
                    self.error(
                        codes::E1030,
                        arg.value.span,
                        format!("field `{}` is given twice", fields[index].name),
                    );
                }
            }
        } else {
            for (index, arg) in args.iter().enumerate() {
                if index >= fields.len() {
                    break;
                }
                values_by_field[index] = Some(self.check_expr(&arg.value, fields[index].ty));
            }
        }

        let mut values = Vec::with_capacity(fields.len());
        let mut invalid = false;
        for (index, field) in fields.iter().enumerate() {
            if let Some(value) = values_by_field[index].take() {
                values.push(value);
                continue;
            }
            if !field.has_default {
                self.error(
                    codes::E2020,
                    span,
                    format!("field `{}` of `{name}` has no value", field.name),
                );
                invalid = true;
                continue;
            }
            let Some(Some(literal)) = defaults.as_ref().and_then(|defaults| defaults.get(index)) else {
                self.error(
                    codes::E1010,
                    field.span,
                    format!(
                        "default expression for class field `{}` is not implemented yet in this phase",
                        field.name
                    ),
                );
                invalid = true;
                continue;
            };
            let default = self.synth_literal(literal, field.span);
            values.push(self.coerce(default, field.ty));
        }
        if invalid {
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }

        Expr {
            ty,
            kind: ExprKind::Builtin { which: Builtin::ClassNew { class_id: id, init: None }, args: values },
            span,
        }
    }

    fn zero_of(&mut self, ty: Ty, span: Span) -> Expr {
        let kind = if self.types.is_float(ty) {
            ExprKind::Float(0.0)
        } else if self.types.is_integral(ty) {
            ExprKind::Int(0)
        } else if ty == self.common.bool_ {
            ExprKind::Bool(false)
        } else {
            ExprKind::Error
        };
        Expr { ty, kind, span }
    }

    /// `[TYP-21]` — the interface call an operator desugars to, once the
    /// method has been found on the left operand's type.
    fn call_operator(&mut self, method: &str, lhs: Expr, rhs: Expr, span: Span) -> Expr {
        let name = Symbol::intern(method);
        let entry = &self.methods[&(lhs.ty, name)];
        let def = entry.def;
        let receiver_mode = entry.receiver;
        let signature: Vec<(Ty, Mode)> =
            self.signatures[def.0 as usize].params.iter().map(|(_, t, m, _)| (*t, *m)).collect();
        let ret = self.signatures[def.0 as usize].ret;

        if signature.len() != 2 {
            self.error(
                codes::E2020,
                span,
                format!("`{method}` must take one argument to be used as an operator"),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let rhs = self.coerce(rhs, signature[1].0);
        let receiver = self.pass_receiver(lhs, receiver_mode, span);
        Expr {
            ty: ret,
            kind: ExprKind::Call {
                callee: def,
                arg_eval_order: None,
                args: vec![receiver, rhs],
                latebound: false,
            },
            span,
        }
    }

    /// `[RNG-5]`/`[RNG-5a1]` — what an operator does when a range type is one
    /// of its operands. `None` where neither is one.
    ///
    /// `[RNG-5a1]` specifies this as generated impls in the type's declaring
    /// module — `T: Add[T, Output = R]`, `T: Add[R, Output = R]` and
    /// `R: Add[T, Output = R]` — so that `[TYP-20]`'s orphan rule is satisfied
    /// without an exemption. This compiler's scalar operators are built in
    /// rather than resolved through `Add` (Part IV §8's interfaces cover user
    /// types only), so the impls have no place to live yet. The **observable**
    /// rule is implemented exactly: which programs are accepted, and the type
    /// of every result. ADR-016 records the deviation and what closes it.
    fn range_binary(
        &mut self,
        hir_op: hir::BinOp,
        op: ast::BinOp,
        lhs: &Expr,
        rhs: &Expr,
        span: Span,
    ) -> Option<RangeBinary> {
        let l = match self.types.kind(lhs.ty) {
            TyKind::Range(id) => Some(*id),
            _ => None,
        };
        let r = match self.types.kind(rhs.ty) {
            TyKind::Range(id) => Some(*id),
            _ => None,
        };
        let (l, r) = match (l, r) {
            (None, None) => return None,
            pair => pair,
        };

        // Two **distinct** nominal range types resolve to no generated impl.
        // `[RNG-5]`: "Arithmetic between two distinct nominal range types is
        // rejected (`E2214`) unless at least one operand is explicitly
        // converted to its representation type."
        if let (Some(a), Some(b)) = (l, r) {
            if a != b {
                let (an, bn) = (
                    self.types.range_def(a).name.to_string(),
                    self.types.range_def(b).name.to_string(),
                );
                let repr = self.types.display(self.types.range_def(a).repr);
                self.sink.emit(
                    Diagnostic::error(
                        codes::E2214,
                        span,
                        format!(
                            "`{}` is not defined between `{an}` and `{bn}`",
                            op.as_str()
                        ),
                    )
                    .primary_label("these are two different range types".to_string())
                    .help(format!(
                        "convert one side explicitly: `x as {repr}`"
                    ))
                    .note(
                        "arithmetic on a range type yields its representation, and two \
                         distinct range types share no operator [RNG-5]",
                    ),
                );
                return Some(RangeBinary::Rejected);
            }
        }

        let id = l.or(r).expect("one side is a range type");
        let repr = self.types.range_def(id).repr;

        // The other side must be the same range type, an untyped literal, or
        // a value of the representation — `[RNG-5]`: "A range value MAY
        // participate directly in arithmetic with an ordinary value of its
        // representation type."
        let other = if l.is_some() { rhs } else { lhs };
        let other_ok = matches!(self.types.kind(other.ty), TyKind::Range(_))
            || self.types.is_untyped_literal(other.ty)
            || other.ty == repr
            || other.ty == self.common.error;
        if !other_ok {
            let name = self.types.range_def(id).name.to_string();
            let found = self.types.display(other.ty);
            let shown_repr = self.types.display(repr);
            self.sink.emit(
                Diagnostic::error(
                    codes::E2214,
                    span,
                    format!("`{}` is not defined between `{name}` and `{found}`", op.as_str()),
                )
                .help(format!(
                    "`{name}` erases to `{shown_repr}`, so both sides must be `{shown_repr}`"
                )),
            );
            return Some(RangeBinary::Rejected);
        }

        let _ = hir_op;
        Some(RangeBinary::ToRepr(repr))
    }

    /// Erase one operand of a range-typed operator to the representation.
    /// `[RNG-5a1]` defines the generated impls "by erasing each operand to
    /// `R` and applying `R`'s operator", which is this.
    fn erase_operand(&mut self, expr: Expr, repr: Ty) -> Expr {
        if matches!(self.types.kind(expr.ty), TyKind::Range(_)) {
            let span = expr.span;
            return Expr { ty: repr, kind: ExprKind::EraseRange(Box::new(expr)), span };
        }
        expr
    }

    fn synth_binary(
        &mut self,
        op: ast::BinOp,
        lhs: &ast::Expr,
        rhs: &ast::Expr,
        span: Span,
    ) -> Expr {
        let Some(hir_op) = convert_binop(op) else {
            self.error(codes::E1010, span, "this operator is not supported yet in this phase");
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };

        let mut lhs = self.synth(lhs);
        let mut rhs = self.synth(rhs);
        // `[TYP-14]` — an operand wants a value. This is before the operator
        // method lookup on purpose: `ref Vec3 + Vec3` should find `Vec3`'s
        // `add`, not fail to find one on a reference.
        lhs = self.read_through(lhs);
        rhs = self.read_through(rhs);

        // `[TYP-21]` — an operator on a non-scalar is an interface method
        // call. `a + b` on a `Vec3` is `a.add(b)`, with both sides passed in
        // the modes the interface declared.
        if let Some(method) = operator_method(op) {
            if self.methods.contains_key(&(lhs.ty, Symbol::intern(method))) {
                return self.call_operator(method, lhs, rhs, span);
            }
        }

        // `[RNG-5]`/`[RNG-5a1]` — operators on range types. This runs before
        // the literal rules below because `[RNG-5a2]` requires the operand
        // types to be matched **exactly** before any `[TYP-5]` coercion:
        // without it `Roughness + Roughness` matches both the generated
        // `Roughness: Add[Roughness]` and, after erasure, `f32: Add[f32]`,
        // and resolution is ambiguous.
        if let Some(result) = self.range_binary(hir_op, op, &lhs, &rhs, span) {
            match result {
                RangeBinary::Rejected => {
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                }
                RangeBinary::ToRepr(repr) => {
                    // `[RNG-5]` — "arithmetic involving a range type normally
                    // yields its **representation**, not the range type".
                    // Erasing both sides is exactly what `[RNG-5a1]`'s
                    // generated impls do: "defined by erasing each operand to
                    // `R` and applying `R`'s operator".
                    lhs = self.erase_operand(lhs, repr);
                    rhs = self.erase_operand(rhs, repr);
                }
            }
        }

        // `[TYP-4]` — no implicit conversion between scalar types in
        // operators. An untyped literal is not a conversion: it adopts the
        // other side's type.
        let lhs_untyped = self.types.is_untyped_literal(lhs.ty);
        let rhs_untyped = self.types.is_untyped_literal(rhs.ty);
        match (lhs_untyped, rhs_untyped) {
            (true, false) => {
                let target = rhs.ty;
                if self.literal_fits(&lhs, target) {
                    lhs = self.adopt_literal(lhs, target);
                }
            }
            (false, true) => {
                let target = lhs.ty;
                if self.literal_fits(&rhs, target) {
                    rhs = self.adopt_literal(rhs, target);
                }
            }
            // Both untyped: `1 + 2` stays untyped and is defaulted later.
            (true, true) => {
                if self.types.is_float(lhs.ty) || self.types.is_float(rhs.ty) {
                    let float_lit = self.common.float_lit;
                    lhs = self.adopt_literal(lhs, float_lit);
                    rhs = self.adopt_literal(rhs, float_lit);
                }
            }
            (false, false) => {}
        }

        if lhs.ty != rhs.ty && lhs.ty != self.common.error && rhs.ty != self.common.error {
            let left = self.types.display(lhs.ty);
            let right = self.types.display(rhs.ty);
            self.sink.emit(
                Diagnostic::error(
                    codes::E2020,
                    span,
                    format!("`{}` cannot be applied to `{left}` and `{right}`", op.as_str()),
                )
                .note("Ember does not convert between numeric types implicitly [TYP-4]")
                .help(format!("cast one side, for example `x as {right}`")),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }

        let operand_ty = lhs.ty;
        if !hir_op.is_comparison() && !self.types.is_numeric(operand_ty) && operand_ty != self.common.error {
            let shown = self.types.display(operand_ty);
            self.error(
                codes::E2020,
                span,
                format!("`{}` cannot be applied to `{shown}`", op.as_str()),
            );
        }

        let ty = if hir_op.is_comparison() { self.common.bool_ } else { operand_ty };
        Expr { ty, kind: ExprKind::Binary { op: hir_op, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span }
    }
}

/// `[TYP-21]` — the interface method each operator desugars to.
fn operator_method(op: ast::BinOp) -> Option<&'static str> {
    Some(match op {
        ast::BinOp::Add => "add",
        ast::BinOp::Sub => "sub",
        ast::BinOp::Mul => "mul",
        ast::BinOp::Div => "div",
        ast::BinOp::Rem => "rem",
        ast::BinOp::BitAnd => "bitand",
        ast::BinOp::BitOr => "bitor",
        ast::BinOp::BitXor => "bitxor",
        ast::BinOp::Shl => "shl",
        ast::BinOp::Shr => "shr",
        // `Eq` and `Ord` give `==` and `<` their meaning on a user type; the
        // rest of the comparisons are derived from them, which needs
        // `[TYP-21]`'s full desugaring and arrives with `Ord`.
        ast::BinOp::Eq => "eq",
        ast::BinOp::Ne => "ne",
        ast::BinOp::Lt => "lt",
        ast::BinOp::Le => "le",
        ast::BinOp::Gt => "gt",
        ast::BinOp::Ge => "ge",
        _ => return None,
    })
}

fn convert_binop(op: ast::BinOp) -> Option<BinOp> {
    Some(match op {
        ast::BinOp::Add => BinOp::Add,
        ast::BinOp::Sub => BinOp::Sub,
        ast::BinOp::Mul => BinOp::Mul,
        ast::BinOp::Div => BinOp::Div,
        ast::BinOp::Rem => BinOp::Rem,
        ast::BinOp::BitAnd => BinOp::BitAnd,
        ast::BinOp::BitOr => BinOp::BitOr,
        ast::BinOp::BitXor => BinOp::BitXor,
        ast::BinOp::Shl => BinOp::Shl,
        ast::BinOp::Shr => BinOp::Shr,
        ast::BinOp::Eq => BinOp::Eq,
        ast::BinOp::Ne => BinOp::Ne,
        ast::BinOp::Lt => BinOp::Lt,
        ast::BinOp::Le => BinOp::Le,
        ast::BinOp::Gt => BinOp::Gt,
        ast::BinOp::Ge => BinOp::Ge,
        // `**`, `is`, `in` desugar to interface calls, which Phase 0 lacks.
        _ => return None,
    })
}

fn mode_of(mode: ast::Mode) -> Mode {
    match mode {
        ast::Mode::Borrow => Mode::Borrow,
        ast::Mode::Mut => Mode::Mut,
        ast::Mode::Owned => Mode::Owned,
    }
}

fn class_openness(openness: ast::Openness) -> ClassOpenness {
    match openness {
        ast::Openness::Final => ClassOpenness::Final,
        ast::Openness::Open => ClassOpenness::Open,
        ast::Openness::Abstract => ClassOpenness::Abstract,
    }
}

/// `[FN-6]`/`[FN-6a]` — callable type parameters use precisely the ordinary
/// parameter-mode vocabulary.  Keep this conversion at the AST/HIR boundary
/// so the type representation never needs to know about parser details.
fn fn_param_mode(mode: ast::Mode) -> FnParamMode {
    match mode {
        ast::Mode::Borrow => FnParamMode::Borrow,
        ast::Mode::Mut => FnParamMode::Mut,
        ast::Mode::Owned => FnParamMode::Owned,
    }
}

fn hir_mode(mode: FnParamMode) -> Mode {
    match mode {
        FnParamMode::Borrow => Mode::Borrow,
        FnParamMode::Mut => Mode::Mut,
        FnParamMode::Owned => Mode::Owned,
    }
}

fn fn_param_mode_from_hir(mode: Mode) -> FnParamMode {
    match mode {
        Mode::Borrow => FnParamMode::Borrow,
        Mode::Mut => FnParamMode::Mut,
        Mode::Owned => FnParamMode::Owned,
    }
}

fn fn_param_mode_name(mode: FnParamMode) -> &'static str {
    match mode {
        FnParamMode::Borrow => "borrowed",
        FnParamMode::Mut => "mut",
        FnParamMode::Owned => "owned",
    }
}

fn ast_mode_name(mode: ast::Mode) -> &'static str {
    match mode {
        ast::Mode::Borrow => "borrowed",
        ast::Mode::Mut => "mut",
        ast::Mode::Owned => "owned",
    }
}

/// `[LT-3]` — whether an expression's value has the `static` region.
///
/// "String literals, `static` items, and `Span`s over them have the `static`
/// region, which outlives everything." Answered on the syntax rather than by
/// analysis, and deliberately **conservative**: anything not obviously static
/// is treated as not static, which errs towards rejecting a program rather
/// than storing a view that outlives its source.
///
/// A region graph would answer this in general. It is not needed for the case
/// the rule is about, because `[STA-2]` restricts a `static`'s initialiser to a
/// literal, and a literal either is one of these or is not a view at all.
fn has_static_region(expr: &ast::Expr) -> bool {
    match &expr.kind {
        // A string literal is the case `[LT-3]` names first.
        ast::ExprKind::Lit(ast::Literal::Str(_)) => true,
        // `[LEX-19]`'s adjacent-literal concatenation is still literals.
        ast::ExprKind::Binary { lhs, rhs, .. } => {
            has_static_region(lhs) && has_static_region(rhs)
        }
        _ => false,
    }
}

/// The name an item declares, if it declares one.
fn item_name(item: &ast::Item) -> Option<Symbol> {
    Some(match &item.kind {
        ast::ItemKind::Fn(d) => d.name.name,
        ast::ItemKind::Struct(d) => d.name.name,
        ast::ItemKind::Enum(d) => d.name.name,
        ast::ItemKind::Interface(d) => d.name.name,
        ast::ItemKind::Const(d) => d.name.name,
        ast::ItemKind::Static(d) => d.name.name,
        ast::ItemKind::Class(d) => d.name.name,
        ast::ItemKind::TypeAlias(d) => d.name.name,
        ast::ItemKind::Extend(_)
        | ast::ItemKind::ExternBlock(_)
        | ast::ItemKind::ExternClass(_)
        | ast::ItemKind::Comptime(_) => return None,
    })
}

/// Members belonging to any item form that can provide a method body.
fn item_members(item: &ast::Item) -> Option<&[ast::Member]> {
    match &item.kind {
        ast::ItemKind::Struct(decl) => Some(&decl.members),
        ast::ItemKind::Class(decl) => Some(&decl.members),
        ast::ItemKind::Enum(decl) => Some(&decl.members),
        ast::ItemKind::Interface(decl) => Some(&decl.members),
        ast::ItemKind::Extend(decl) => Some(&decl.members),
        _ => None,
    }
}

/// The visibility of a named item in a module, or `None` if it has no such
/// item.
fn public_item(module: &ast::Module, name: Symbol) -> Option<ast::VisKind> {
    module
        .items
        .iter()
        .find(|item| item_name(item) == Some(name))
        .map(|item| item.vis.kind)
}

/// A printed type squeezed into an identifier, so that a synthesised name
/// such as `Option_i32` is unique per type argument.
fn type_stem(shown: &str) -> String {
    let mut out = String::new();
    for ch in shown.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_matches('_').to_string()
}

/// `[MNG-1]` — a method's C symbol carries the type it is on, so two types
/// may both have a `length`.
fn method_symbol(owner: &str, name: Symbol) -> String {
    let owner: String = owner
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    ember_branding::mangled(&format!("{}_{name}", owner.trim_matches('_')))
}

/// A source-level generic owner has no concrete `Ty` before one of its
/// instantiations is reached. EMIF still needs a stable receiver identity, so
/// spell its already-resolved qualified declaration name with its positional
/// parameter binders. This is cache metadata only; it is never a C type name.
fn canonical_generic_owner(name: Symbol, parameters: usize) -> String {
    let arguments = (0..parameters)
        .map(|index| format!("$P{index}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("{name}[{arguments}]")
}

/// Declaration-only member identities must not collide with generated C
/// symbols. A generic or interface member can be import-visible without any
/// body being emitted, and a concrete specialization is not its source
/// declaration. The branded mangle keeps the artifact key deterministic while
/// preserving the runtime/C namespace boundary.
fn member_declaration_symbol(owner: &str, name: Symbol) -> String {
    ember_branding::mangled(&format!("__ember_declaration_member__{owner}__{name}"))
}

/// The name of an interface written in an `implements` list or a supertrait
/// position. Generic interfaces arrive with generics, so only a bare name is
/// understood here.
fn interface_name(ty: &ast::TypeExpr) -> Option<Symbol> {
    match &ty.kind {
        ast::TypeKind::Path { segments, args } if args.is_empty() && segments.len() == 1 => {
            Some(segments[0].name)
        }
        _ => None,
    }
}

/// Whether an expression names a location rather than a value — the test
/// `[EXP-5]` applies to an assignment target and to a `mut` argument.
fn is_place(kind: &ExprKind) -> bool {
    match kind {
        ExprKind::Local(_) | ExprKind::Index { .. } => true,
        ExprKind::Field { base, .. } => is_place(&base.kind),
        ExprKind::Deref(_) => true,
        _ => false,
    }
}

fn is_single_path(expr: &ast::Expr, wanted: &str) -> bool {
    matches!(
        &expr.kind,
        ast::ExprKind::Path { segments }
            if segments.len() == 1 && segments[0].name.is(wanted)
    )
}

/// The local whose storage owns a place. This deliberately follows only the
/// place-forming HIR nodes accepted by [`is_place`]; value-producing wrappers
/// must not make a temporary look like a borrowed parameter.
fn root_local(kind: &ExprKind) -> Option<LocalId> {
    match kind {
        ExprKind::Local(local) => Some(*local),
        ExprKind::Field { base, .. }
        | ExprKind::Index { base, .. }
        | ExprKind::Deref(base) => root_local(&base.kind),
        _ => None,
    }
}

fn binding_name(pattern: &ast::Pattern) -> Option<Symbol> {
    match &pattern.kind {
        ast::PatternKind::Bind { name, .. } => Some(name.name),
        _ => None,
    }
}

/// `@view`, `@packed` and the rest: an attribute by bare name.
fn has_attribute(attrs: &[ast::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path.len() == 1 && attr.path[0].name.is(name))
}

fn has_derive(attrs: &[ast::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| {
        attr.path.len() == 1
            && attr.path[0].name.is("derive")
            && attr.args.iter().any(|arg| match arg {
                ast::AttrArg::Expr(e) => match &e.kind {
                    ast::ExprKind::Path { segments } => {
                        segments.len() == 1 && segments[0].name.is(name)
                    }
                    _ => false,
                },
                ast::AttrArg::Named { .. } => false,
            })
    })
}

/// The single path argument of an attribute, as in `@repr(u8)`.
fn attr_argument(attrs: &[ast::Attribute], name: &str) -> Option<Symbol> {
    attrs.iter().find(|a| a.path.len() == 1 && a.path[0].name.is(name)).and_then(|attr| {
        attr.args.iter().find_map(|arg| match arg {
            ast::AttrArg::Expr(e) => match &e.kind {
                ast::ExprKind::Path { segments } if segments.len() == 1 => Some(segments[0].name),
                _ => None,
            },
            ast::AttrArg::Named { .. } => None,
        })
    })
}

/// An integer written directly, with an optional leading `-`. Enough for a
/// discriminant and an array length until `const` items exist.
fn literal_int(expr: &ast::Expr) -> Option<i128> {
    match &expr.kind {
        ast::ExprKind::Paren(inner) => literal_int(inner),
        ast::ExprKind::Lit(ast::Literal::Int { value, .. }) => i128::try_from(*value).ok(),
        ast::ExprKind::Unary { op: ast::UnOp::Neg, operand } => {
            literal_int(operand).map(|v| -v)
        }
        _ => None,
    }
}

/// `[MNG-1]` — `em_<pkg>_<module>_<item>`. Phase 0 compiles one module at a
/// time with no package graph, so the package and module segments are fixed
/// until `ember_build` supplies them.
fn mangle(name: Symbol, is_main: bool) -> String {
    if is_main {
        // The C entry point calls this; `[MNG-2]` reserves the plain name.
        return ember_branding::mangled("main");
    }
    // A qualified name carries dots, which C does not allow in an identifier.
    ember_branding::mangled(name.as_str())
}

/// `[RNG-4]` — the interval an arithmetic operation produces, given its
/// operands' intervals.
///
/// Only `+`, `-` and `*` are derived. Division is left out on purpose: the
/// interval depends on whether the divisor's range straddles zero, and a
/// divisor that can be zero panics under `[TYP-8]` rather than producing a
/// value, so the fact would describe a program that does not reach the
/// statement. `%`, shifts and the bitwise operators are the "bit-operation
/// facts" D-018 lists as later work.
fn interval(op: BinOp, a: (Bound, Bound), b: (Bound, Bound)) -> Option<(Bound, Bound)> {
    let corners = |f: fn(f64, f64) -> f64, g: fn(i128, i128) -> Option<i128>| {
        match (a.0, a.1, b.0, b.1) {
            (Bound::Float(a0), Bound::Float(a1), Bound::Float(b0), Bound::Float(b1)) => {
                let vals = [f(a0, b0), f(a0, b1), f(a1, b0), f(a1, b1)];
                let lo = vals.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                Some((Bound::Float(lo), Bound::Float(hi)))
            }
            (Bound::Int(a0), Bound::Int(a1), Bound::Int(b0), Bound::Int(b1)) => {
                let vals = [g(a0, b0)?, g(a0, b1)?, g(a1, b0)?, g(a1, b1)?];
                Some((
                    Bound::Int(*vals.iter().min()?),
                    Bound::Int(*vals.iter().max()?),
                ))
            }
            _ => None,
        }
    };
    match op {
        BinOp::Add => corners(|x, y| x + y, |x, y| x.checked_add(y)),
        BinOp::Sub => corners(|x, y| x - y, |x, y| x.checked_sub(y)),
        BinOp::Mul => corners(|x, y| x * y, |x, y| x.checked_mul(y)),
        _ => None,
    }
}

/// A range endpoint, rendered the way a diagnostic should show it.
fn show_bound(bound: Bound) -> String {
    match bound {
        Bound::Int(v) => v.to_string(),
        Bound::Float(v) => {
            // `1` reads as an integer where the range is over floats, and
            // IV.2a's example prints `1.4`, so a whole float keeps its point.
            if v.fract() == 0.0 && v.is_finite() {
                format!("{v:.1}")
            } else {
                v.to_string()
            }
        }
    }
}

/// `0.0 ..= 1.0`, as `[RNG-1]` writes it.
fn show_range(def: &RangeDef, _types: &TypeTable) -> String {
    let op = if def.inclusive { "..=" } else { ".." };
    format!("{} {op} {}", show_bound(def.lo), show_bound(def.hi))
}

/// What `[RNG-5]` does with an operator one of whose operands is a range type.
enum RangeBinary {
    /// Both operands erase to the representation and `R`'s operator applies
    /// (`[RNG-5a1]`).
    ToRepr(Ty),
    /// Two distinct range types, or a mismatched representation: no generated
    /// impl matches and the operator is `E2214`.
    Rejected,
}

/// `[MOD-2]` — the AST's visibility as the type table records it.
fn field_vis(kind: ast::VisKind) -> FieldVis {
    match kind {
        ast::VisKind::Private => FieldVis::Private,
        ast::VisKind::Package => FieldVis::Package,
        ast::VisKind::Public => FieldVis::Public,
    }
}

/// `[CLO-6]` — this syntax creates the implicit `CallableOnce` bound. The
/// callable's own `fn(...)` type records its argument modes; the outer
/// `owned` mode selects consumption of the callee value.
fn is_owned_callable_param(param: &ast::Param) -> bool {
    param.mode == ast::Mode::Owned && is_callable_param(param)
}

fn is_callable_param(param: &ast::Param) -> bool {
    matches!(
        param.kind,
        ast::ParamKind::Named { ty: ast::TypeExpr { kind: ast::TypeKind::Fn { .. }, .. }, .. }
    )
}

/// `[FN-6b]` — identify a callable parameter whose expected type explicitly
/// owns a late-bound callback boundary. This source fact must survive generic
/// monomorphization, where the parameter's resolved type is no longer a
/// `TyKind::Param` carrying the original bound.
fn is_latebound_callable_param(param: &ast::Param) -> bool {
    matches!(
        param.kind,
        ast::ParamKind::Named {
            ty: ast::TypeExpr {
                kind: ast::TypeKind::Fn { latebound: true, .. },
                ..
            },
            ..
        }
    )
}

/// `[CLO-2]` — the scopes outside the closure being checked, kept so that a
/// name the body cannot find can be reported as a capture rather than as a
/// typo. A closure body sees none of them: `[CLO-1]`'s capture-free case is
/// the only one this phase compiles, so a name from outside is an error, and
/// the only question is which error.
struct CaptureWatch {
    /// Every name the enclosing function has in scope, with its type. Types
    /// and not `LocalId`s, because the enclosing locals are swapped out while
    /// the closure body is checked and an id into them would dangle.
    outer: HashMap<Symbol, Ty>,
    /// Pass one: the captures discovered, in the order first mentioned. That
    /// order is the environment's field order.
    found: Vec<(Symbol, Ty)>,
    /// Pass two: the environment parameter and its type, once built. `None`
    /// during discovery, when a captured name resolves to a placeholder.
    env: Option<(LocalId, StructId)>,
    /// `owned fn` stores captures directly; every other closure captures a
    /// shared borrow. This is carried across both body-check passes so capture
    /// discovery and the final body use the same representation.
    captures_by_move: bool,
}
