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

use std::collections::HashMap;

use ember_ast as ast;
use ember_hir as hir;
use ember_diag::{Diagnostic, Sink, codes};
use ember_hir::{
    BinOp, Block, Builtin, DefId, Expr, ExprKind, Function, LocalDecl, LocalId, Mode, Param,
    Program, Stmt, UnOp,
};
use ember_span::{Span, Symbol};
use ember_types::{
    Bound, CommonTypes, EnumDef, EnumId, FieldDef, FieldVis, OverflowPolicy, RangeDef, StructDef,
    StructId, Ty, TyKind, TypeTable, UintTy, VariantDef, int_max,
};

/// One parsed module and where it sits in the package (`[MOD-1]`). The root
/// module's path is empty.
pub struct LoadedModule {
    pub path: Vec<String>,
    pub module: ast::Module,
}

pub fn check(
    modules: &[LoadedModule],
    types: &mut TypeTable,
    common: &CommonTypes,
    sink: &mut Sink,
    default_overflow: OverflowPolicy,
) -> Program {
    let mut checker = Checker::new(types, common, sink);
    checker.default_overflow = default_overflow;
    checker.prefixes = modules.iter().map(|m| m.path.join(".")).collect();
    checker.visible = vec![HashMap::new(); modules.len()];
    checker.namespaces = vec![HashMap::new(); modules.len()];

    // Names first, across every module, so that an import can name an item in
    // a module that has not been walked yet — `[MOD-4]` allows cycles.
    for (index, loaded) in modules.iter().enumerate() {
        checker.current_module = index;
        checker.declare_names(&loaded.module);
    }
    checker.bind_imports(modules);
    for (index, loaded) in modules.iter().enumerate() {
        checker.current_module = index;
        checker.collect(&loaded.module);
    }
    for (index, loaded) in modules.iter().enumerate() {
        checker.current_module = index;
        checker.collect_interfaces(&loaded.module);
    }
    for (index, loaded) in modules.iter().enumerate() {
        checker.current_module = index;
        checker.collect_methods(&loaded.module);
    }
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
    Program { functions, main }
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
}

/// `[TYP-16]` — a struct declared with type parameters. Its fields are
/// resolved once with the parameters opaque, so instantiating one is a
/// substitution rather than another walk of the declaration.
#[derive(Clone)]
struct GenericStruct {
    params: Vec<Symbol>,
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
    receiver: Mode,
    /// The parameters after `self`. `self` itself is not here: its type is
    /// the instantiation, which does not exist until one is built.
    params: Vec<(Symbol, Ty, Mode, Span)>,
    ret: Ty,
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

/// One instantiation of a generic function: which one, and with what.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Instance {
    def: DefId,
    args: Vec<Ty>,
}

/// One method that can be found by `recv.name(...)`.
struct MethodEntry {
    def: DefId,
    /// `[TYP-24]` — an inherent method always beats an interface method of the
    /// same name, and two interfaces offering one name is `E2070`.
    from_interface: Option<Symbol>,
    /// The receiver's mode, which decides how the receiver is passed.
    receiver: Mode,
}

/// A declared interface: the methods a type must provide, and which of them
/// carry a default body.
struct InterfaceDef {
    /// Method name, the `DefId` its declared signature was given, the
    /// receiver's mode, and whether the interface supplies a body.
    ///
    /// The `DefId` exists so that `[TYP-17]`'s "only what the bounds provide"
    /// can be checked: inside `fn f[T: Shape]`, `x.area()` resolves to the
    /// interface's declaration and takes its type. Nothing calls it — a
    /// generic body is never emitted, only its instantiations are.
    methods: Vec<(Symbol, DefId, Mode, bool)>,
    supertraits: Vec<Symbol>,
}

struct Checker<'a> {
    types: &'a mut TypeTable,
    common: &'a CommonTypes,
    sink: &'a mut Sink,
    struct_ids: HashMap<Symbol, StructId>,
    enum_ids: HashMap<Symbol, EnumId>,
    /// Every named type in scope — structs and enums both.
    named_types: HashMap<Symbol, Ty>,
    fn_ids: HashMap<Symbol, DefId>,
    signatures: Vec<Signature>,
    /// Every method reachable as `value.name(...)`, keyed by the receiver's
    /// type and the method name.
    methods: HashMap<(Ty, Symbol), MethodEntry>,
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
    /// Instantiations still to have their bodies checked.
    pending: Vec<(Instance, DefId)>,
    /// The same, for the methods of instantiated generic structs.
    pending_methods: Vec<PendingMethod>,

    // Per-function state.
    locals: Vec<LocalDecl>,
    scopes: Vec<HashMap<Symbol, LocalId>>,
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
            enum_ids: HashMap::new(),
            named_types: HashMap::new(),
            fn_ids: HashMap::new(),
            signatures: Vec::new(),
            methods: HashMap::new(),
            interfaces: HashMap::new(),
            implemented: Vec::new(),
            constants: HashMap::new(),
            prefixes: vec![String::new()],
            visible: vec![HashMap::new()],
            namespaces: vec![HashMap::new()],
            current_module: 0,
            type_params: HashMap::new(),
            current_generics: Vec::new(),
            self_ty: None,
            assoc_scope: std::collections::BTreeSet::new(),
            assoc_values: HashMap::new(),
            instances: HashMap::new(),
            generic_structs: HashMap::new(),
            pending: Vec::new(),
            pending_methods: Vec::new(),
            locals: Vec::new(),
            scopes: Vec::new(),
            or_bindings: None,
            loop_labels: Vec::new(),
            in_defer: false,
            in_unsafe: false,
            ret_ty,
            default_overflow: OverflowPolicy::default(),
        }
    }

    fn error(&mut self, code: ember_diag::Code, span: Span, message: impl Into<String>) {
        self.sink.emit(Diagnostic::error(code, span, message));
    }

    /// `[LT-1a]` — `@borrows(p₁, …, pₙ)` overrides the region that `[LT-1]`'s
    /// elision would give a view-typed return, so that the caller may keep
    /// using the parameters it does *not* name.
    ///
    /// "Naming a parameter that is not view-typed, or writing `@borrows` on a
    /// function whose return is not view-typed, is `E2031`." A name that
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

        let viewable: Vec<String> = params
            .iter()
            .filter(|(_, ty, _, _)| self.types.is_view(*ty))
            .map(|(name, _, _, _)| name.to_string())
            .collect();

        let mut named_positions = Vec::new();
        for arg in &attr.args {
            let ast::AttrArg::Expr(expr) = arg else { continue };
            let ast::ExprKind::Path { segments } = &expr.kind else { continue };
            if segments.len() != 1 {
                continue;
            }
            let named = segments[0].name;
            if let Some(position) = params.iter().position(|(name, _, _, _)| *name == named) {
                named_positions.push(position);
            }
            match params.iter().find(|(name, _, _, _)| *name == named) {
                None => {
                    let known = if viewable.is_empty() {
                        "this function has no view-typed parameter".to_string()
                    } else {
                        format!("the view-typed parameters are {}", viewable.join(", "))
                    };
                    self.sink.emit(
                        Diagnostic::error(
                            codes::E2031,
                            expr.span,
                            format!("`@borrows` names `{named}`, which is not a parameter"),
                        )
                        .help(known),
                    );
                }
                Some((_, ty, _, span)) if !self.types.is_view(*ty) => {
                    let shown = self.types.display(*ty);
                    self.sink.emit(
                        Diagnostic::error(
                            codes::E2031,
                            expr.span,
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

    /// `[TYP-15]` — "a view-typed value MUST NOT be stored in a place whose
    /// region is not outlived by the view's region. Class fields, non-view
    /// struct fields, `static`s, `Box[T]` and `Shared[T]` contents, container
    /// elements and `owned fn` captures have no bounding region and are
    /// therefore always forbidden."
    ///
    /// The cases here are the ones decidable without regions: the place has no
    /// region at all, so no analysis can make it work. A view escaping through
    /// a *return* is `[LT-1]`'s job and needs the region graph.
    fn reject_stored_view(&mut self, ty: Ty, span: Span, place: &str) {
        if !self.types.is_view(ty) {
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
                "a view borrows something, and this place outlives whatever it could ",
                "borrow (TYP-15)"
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
                .intern(TyKind::Param { index: index as u32, name: param.name.name });
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
            declared.push(GenericParam { name: param.name.name, bounds });
        }
        declared
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
            let Some(name) = item_name(item) else { continue };
            let qualified = self.qualified(name);
            self.visible[index].insert(name, qualified);
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

    /// Two passes over the items: names first, so that a struct may refer to a
    /// struct declared later in the file, then fields and signatures
    /// (Part XVIII §4.4 step 1).
    fn collect(&mut self, module: &ast::Module) {
        for (item_index, item) in module.items.iter().enumerate() {
            match &item.kind {
                // `[TYP-16]` — a struct with type parameters is not a type; it
                // is a recipe. Each `Pair[i32, f32]` builds one.
                ast::ItemKind::Struct(decl) if !decl.generics.is_empty() => {
                    let name = self.qualified(decl.name.name);
                    let params: Vec<Symbol> =
                        decl.generics.iter().map(|g| g.name.name).collect();
                    self.declare_generics(&decl.generics);
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
                    // `[TYP-16]` — the methods, resolved once here. `self` is
                    // left out of the signature because its type is the
                    // instantiation, and no instantiation exists yet.
                    let mut methods = Vec::new();
                    for (member_index, member) in decl.members.iter().enumerate() {
                        let ast::MemberKind::Fn(fn_decl) = &member.kind else { continue };
                        if fn_decl.body.is_none() {
                            continue;
                        }
                        let Some((receiver, signature)) =
                            self.method_signature(fn_decl, None, &member.attrs, member.span)
                        else {
                            continue;
                        };
                        methods.push(GenericMethod {
                            name: fn_decl.name.name,
                            receiver,
                            params: signature.params,
                            ret: signature.ret,
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
                            fields,
                            derives_copy: has_derive(&item.attrs, "Copy"),
                            methods,
                        },
                    );
                }
                ast::ItemKind::Struct(decl) => {
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
                        origin: None,
                        declaring_module: self.current_module,
                    });
                    let ty = self.types.intern(TyKind::Struct(id));
                    self.struct_ids.insert(name, id);
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
                    // `[TYP-15]` — a `static` has no bounding region, so a
                    // view stored in one can outlive anything.
                    self.reject_stored_view(ty, decl.ty.span, "a `static`");
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
                    let generics = self.declare_generics(&decl.generics);
                    let params: Vec<(Symbol, Ty, Mode, Span)> = decl
                        .params
                        .iter()
                        .filter_map(|p| match &p.kind {
                            ast::ParamKind::Named { name, ty } => {
                                Some((name.name, self.resolve_type(ty), mode_of(p.mode), p.span))
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
                self.collect_interface(decl, item.span);
            }
        }
    }

    /// A third collection pass: every method on a type, with every interface
    /// already collected so that `implements` can name one.
    fn collect_methods(&mut self, module: &ast::Module) {
        for item in &module.items {
            match &item.kind {
                ast::ItemKind::Struct(decl) => {
                    let Some(&ty) = self.named_types.get(&self.qualified(decl.name.name)) else { continue };
                    self.collect_members(ty, &decl.members, None, item.span);
                    self.collect_implements(ty, &decl.implements, &decl.members, item.span);
                }
                ast::ItemKind::Enum(decl) => {
                    let Some(&ty) = self.named_types.get(&self.qualified(decl.name.name)) else { continue };
                    self.collect_members(ty, &decl.members, None, item.span);
                    self.collect_implements(ty, &decl.implements, &decl.members, item.span);
                }
                ast::ItemKind::Extend(decl) => {
                    let ty = self.resolve_type(&decl.target);
                    if ty == self.common.error {
                        continue;
                    }
                    // `[IFC-1]` — `extend T:` with no `implements` adds
                    // inherent methods. With `implements`, the methods belong
                    // to that interface.
                    let interface = decl.implements.first().and_then(interface_name);
                    self.collect_members(ty, &decl.members, interface, item.span);
                    self.collect_implements(ty, &decl.implements, &decl.members, item.span);
                }
                _ => {}
            }
        }
        // A default body gives the implementing type a method it did not
        // write. These have to exist before any body is checked, or a call to
        // one would not resolve.
        self.register_defaults(module);
        // Only now is every method known: a type may declare `implements` in
        // its header and define the methods in a later `extend` block.
        self.check_implementations();
    }

    /// Every implementation has every method the interface requires, and every
    /// supertrait it names (`[IFC-3]`).
    fn check_implementations(&mut self) {
        for (ty, interface, span) in self.implemented.clone() {
            let Some(def) = self.interfaces.get(&interface) else { continue };
            let required: Vec<Symbol> = def.methods.iter().map(|(n, _, _, _)| *n).collect();
            let supertraits = def.supertraits.clone();

            for method in required {
                if self.methods.contains_key(&(ty, method)) {
                    continue;
                }
                let shown = self.types.display(ty);
                self.error(
                    codes::E2040,
                    span,
                    format!("`{shown}` implements `{interface}` but does not define `{method}`"),
                );
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

    /// Register one method per (implementing type, defaulted interface method)
    /// that the type did not define itself.
    fn register_defaults(&mut self, module: &ast::Module) {
        let implementations = self.implemented.clone();
        for item in &module.items {
            let ast::ItemKind::Interface(decl) = &item.kind else { continue };
            let interface = decl.name.name;
            for (ty, _, _) in implementations.iter().filter(|(_, i, _)| *i == interface) {
                for member in &decl.members {
                    let ast::MemberKind::Fn(fn_decl) = &member.kind else { continue };
                    if fn_decl.body.is_none() {
                        continue;
                    }
                    if self.methods.contains_key(&(*ty, fn_decl.name.name)) {
                        continue;
                    }
                    let Some((receiver, signature)) =
                        self.method_signature(fn_decl, Some(*ty), &member.attrs, member.span)
                    else {
                        continue;
                    };
                    self.register_method(
                        *ty,
                        fn_decl.name.name,
                        signature,
                        receiver,
                        Some(interface),
                        member.span,
                    );
                }
            }
        }
    }

    fn collect_interface(&mut self, decl: &ast::InterfaceDecl, span: Span) {
        // Part IV §8 — `Self` inside an `interface` is the implementing type,
        // which is unknown here, so it is a parameter until the interface is
        // used. `[TYP-22]` is what says a method returning it by value is not
        // `dyn`-compatible; statically it resolves like any other parameter.
        let outer_self = self.self_ty.replace(self.common.self_ty);
        self.collect_interface_inner(decl, span);
        self.self_ty = outer_self;
    }

    fn collect_interface_inner(&mut self, decl: &ast::InterfaceDecl, span: Span) {
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
                self.method_signature(f, None, &member.attrs, member.span)
            else {
                continue;
            };
            // A `DefId` for the declaration, so a call through a bound has a
            // signature to take its type from.
            let def = DefId(self.signatures.len() as u32);
            self.signatures.push(signature);
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

    /// Turn a method declaration into a signature. `self_ty` is `None` inside
    /// an interface, where the receiver's type is not yet known.
    fn method_signature(
        &mut self,
        decl: &ast::FnDecl,
        self_ty: Option<Ty>,
        attrs: &[ast::Attribute],
        span: Span,
    ) -> Option<(Mode, Signature)> {
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
                    params.push((name.name, self.resolve_type(ty), mode_of(param.mode), param.span))
                }
            }
        }
        let Some(receiver) = receiver else {
            self.error(
                codes::E1010,
                span,
                "a method needs a `self` receiver in this phase of the compiler",
            );
            return None;
        };
        let ret = decl.ret.as_ref().map(|t| self.resolve_type(t)).unwrap_or(self.common.void);
        // `[LT-1a]` — the receiver is named as `self`, so a method's attribute
        // is resolved against the same parameter list the body will see.
        let borrows = self.check_borrows_attribute(attrs, &params, ret);
        Some((receiver, Signature { params, ret, generics: Vec::new(), borrows }))
    }

    /// Register every method in a type body or `extend` block.
    fn collect_members(
        &mut self,
        ty: Ty,
        members: &[ast::Member],
        from_interface: Option<Symbol>,
        span: Span,
    ) {
        // `Self` inside a type body or an `extend` block is that type.
        let outer_self = self.self_ty.replace(ty);
        self.collect_members_inner(ty, members, from_interface, span);
        self.self_ty = outer_self;
    }

    fn collect_members_inner(
        &mut self,
        ty: Ty,
        members: &[ast::Member],
        from_interface: Option<Symbol>,
        span: Span,
    ) {
        // `[IFC-4]` — `type Item = i32` in an implementation says what the
        // interface's associated type is for this type.
        for member in members {
            let ast::MemberKind::TypeAlias(alias) = &member.kind else { continue };
            let Some(value) = &alias.value else { continue };
            let value = self.resolve_type(value);
            self.assoc_values.insert((ty, alias.name.name), value);
        }
        for member in members {
            let ast::MemberKind::Fn(decl) = &member.kind else { continue };
            if decl.body.is_none() {
                continue;
            }
            let Some((receiver, signature)) =
                self.method_signature(decl, Some(ty), &member.attrs, member.span)
            else {
                continue;
            };
            self.register_method(ty, decl.name.name, signature, receiver, from_interface, member.span);
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
                // `Span[T]` / `MutSpan[T]` — Part IV §1's View category, so
                // a compiler-known type like `str` rather than a library
                // struct: a struct over a raw pointer carries no region, and
                // the region is what `[TYP-15]` and `[UNS-4]` rest on.
                if name.is("Span") || name.is("MutSpan") {
                    let mutable = name.is("MutSpan");
                    if args.len() != 1 {
                        self.error(
                            codes::E2020,
                            ty.span,
                            format!("`{name}` takes one type argument"),
                        );
                        return self.common.error;
                    }
                    let ast::GenericArg::Type(t) = &args[0] else {
                        self.error(codes::E1010, ty.span, "expected a type argument");
                        return self.common.error;
                    };
                    let elem = self.resolve_type(t);
                    // `[TYP-15]` — a view of views has two regions and
                    // `[LT-2]` gives a type one.
                    self.reject_stored_view(elem, t.span, "a span element");
                    return self.types.intern(TyKind::Span { elem, mutable });
                }
                // `Array[T]` — a compiler-known growable sequence (Part XX.1).
                if name.is("Array") {
                    if args.len() != 1 {
                        self.error(codes::E2020, ty.span, "`Array` takes one type argument");
                        return self.common.error;
                    }
                    let ast::GenericArg::Type(t) = &args[0] else {
                        self.error(codes::E1010, ty.span, "expected a type argument");
                        return self.common.error;
                    };
                    let elem = self.resolve_type(t);
                    // `[TYP-15]` — "arbitrary owning containers instantiated
                    // at a view type (`Array[str]`, `Array[MutSpan[T]]`)
                    // remain rejected". `[TYP-15a]`'s `BorrowList`/`ViewList`
                    // are the sanctioned exception and are not built.
                    self.reject_stored_view(elem, t.span, "a container element");
                    return self.types.intern(TyKind::Vec { elem });
                }
                // `[TYP-16]` — a user generic struct, instantiated on demand:
                // `Pair[i32, f32]` is its own struct with its own layout.
                if let Some(decl) = self.generic_structs.get(&self.resolve_name(name)).cloned() {
                    let mut resolved = Vec::new();
                    for arg in args {
                        let ast::GenericArg::Type(t) = arg else {
                            self.error(codes::E1010, ty.span, "expected a type argument");
                            return self.common.error;
                        };
                        resolved.push(self.resolve_type(t));
                    }
                    return self.instantiate_struct(name, &decl, &resolved, ty.span);
                }
                let arity = if name.is("Option") {
                    1
                } else if name.is("Result") {
                    2
                } else {
                    self.error(
                        codes::E1010,
                        ty.span,
                        format!("`{name}` does not take type arguments in this phase"),
                    );
                    return self.common.error;
                };
                if args.len() != arity {
                    self.error(
                        codes::E2020,
                        ty.span,
                        format!("`{name}` takes {arity} type arguments, found {}", args.len()),
                    );
                    return self.common.error;
                }
                let mut resolved = Vec::new();
                for arg in args {
                    let ast::GenericArg::Type(t) = arg else {
                        self.error(codes::E1010, ty.span, "expected a type argument");
                        return self.common.error;
                    };
                    resolved.push(self.resolve_type(t));
                }
                if name.is("Option") {
                    self.option_of(resolved[0])
                } else {
                    self.result_of(resolved[0], resolved[1])
                }
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

    /// `[UNS-5]` — `std.mem`'s raw primitives: `alloc[T]`, `free[T]`,
    /// `read[T]`, `write[T]` and `size_of[T]`. Everything but `size_of` needs
    /// an `unsafe` block (`[UNS-1]`); these are what the collections will be
    /// written on top of.
    fn synth_memory_builtin(
        &mut self,
        name: Symbol,
        args: &[ast::Arg],
        explicit: &[Ty],
        span: Span,
    ) -> Option<Expr> {
        let usize_ty = self.common.usize;
        let void = self.common.void;
        let (which, arity, needs_unsafe) = if name.is("alloc") {
            (Builtin::MemAlloc, 1, true)
        } else if name.is("free") {
            (Builtin::MemFree, 2, true)
        } else if name.is("read") {
            (Builtin::PtrRead, 2, true)
        } else if name.is("write") {
            (Builtin::PtrWrite, 3, true)
        } else if name.is("size_of") {
            (Builtin::SizeOf, 0, false)
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

        // `alloc[T]` and `size_of[T]` say their element type; `free`, `read`
        // and `write` take it from the pointer they are given.
        let mut checked: Vec<Expr> = Vec::new();
        let elem = match which {
            Builtin::MemAlloc | Builtin::SizeOf => match explicit.first() {
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
        // `size_of` has no arguments, so the element type has to travel
        // somewhere; a zero-sized placeholder of that type carries it.
        if which == Builtin::SizeOf {
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
        if let TyKind::Struct(id) = *self.types.kind(ty) {
            let origin = self.types.struct_def(id).origin.clone();
            if let Some((name, generic_args)) = origin {
                let concrete: Vec<Ty> = generic_args
                    .iter()
                    .map(|&a| self.substitute_ty(a, args))
                    .collect();
                if concrete == generic_args {
                    return ty;
                }
                let Some(decl) = self.generic_structs.get(&name).cloned() else { return ty };
                return self.instantiate_struct(name, &decl, &concrete, Span::DUMMY);
            }
            return ty;
        }
        self.types.substitute(ty, args)
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
        let mut solved: Vec<Option<Ty>> = vec![None; decl.params.len()];
        for (slot, ty) in explicit.iter().enumerate() {
            if slot < solved.len() {
                solved[slot] = Some(*ty);
            }
        }
        // Unify each declared field type with the value given for it.
        for (arg, field) in args.iter().zip(decl.fields.iter()) {
            let value = self.synth_committed(&arg.value);
            self.types.unify(field.ty, value.ty, &mut solved);
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
            let mut params =
                vec![(Symbol::intern("self"), ty, method.receiver, method.span)];
            for &(field_name, param_ty, mode, param_span) in &method.params {
                params.push((field_name, self.substitute_ty(param_ty, args), mode, param_span));
            }
            let ret = self.substitute_ty(method.ret, args);
            let signature = Signature { params, ret, generics: Vec::new(), borrows: None };
            let Some(def) = self.register_method(
                ty,
                method.name,
                signature,
                method.receiver,
                None,
                method.span,
            ) else {
                continue;
            };
            // `[DRP-1]` — a generic that writes `fn drop` gives every one of
            // its instantiations a destructor.
            if method.name.is("drop") {
                self.types.struct_def_mut(id).has_drop = true;
            }
            self.pending_methods.push(PendingMethod {
                def,
                owner: ty,
                origin: (name, args.to_vec()),
                source: method.source,
            });
        }
        ty
    }

    /// `Option[T]`, as a two-variant enum built once per `T`.
    fn option_of(&mut self, inner: Ty) -> Ty {
        let name = Symbol::intern(&format!("Option_{}", type_stem(&self.types.display(inner))));
        self.builtin_enum(name, &[(Symbol::intern("None"), Vec::new()), (Symbol::intern("Some"), vec![inner])])
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
                    self.ret_ty = self.signatures[def.0 as usize].ret;
                    let signature_params: Vec<(Symbol, Ty, Mode, Span)> = self.signatures
                        [def.0 as usize]
                        .params
                        .iter()
                        .map(|(n, t, m, s)| (*n, *t, *m, *s))
                        .collect();
                    for (name, ty, mode, param_span) in signature_params {
                        let local_ty = match mode {
                            Mode::Mut => self.mut_param_ty(ty),
                            _ => ty,
                        };
                        self.declare(Some(name), local_ty, param_span);
                    }
                    self.check_block(block);
                }
                self.type_params.clear();
                self.current_generics.clear();
                continue;
            }

            self.locals = Vec::new();
            self.scopes = vec![HashMap::new()];
            self.ret_ty = self.signatures[def.0 as usize].ret;

            let mut params = Vec::new();
            let signature_params: Vec<(Symbol, Ty, Mode, Span)> = self.signatures[def.0 as usize]
                .params
                .iter()
                .map(|(n, t, m, s)| (*n, *t, *m, *s))
                .collect();
            for (name, ty, mode, span) in signature_params {
                // A `mut` parameter is an inout: inside the function it is a
                // `ref mut T`, and every mention of its name reads through it.
                // Without this the callee writes to a copy and the caller
                // never sees it.
                let local_ty = match mode {
                    Mode::Mut => self.mut_param_ty(ty),
                    _ => ty,
                };
                let local = self.declare(Some(name), local_ty, span);
                params.push(Param { local, mode });
            }

            let body = match &decl.body {
                Some(block) => self.check_block(block),
                None => Block { stmts: Vec::new(), span: item.span },
            };

            // `[MNG-1]` — the symbol carries the module, so two modules may
            // each declare a `helper`. Only the root module's `main` is the
            // entry point; another module's is `a.main`, which is not it.
            let name = self.qualified(decl.name.name);
            let is_main = name.is("main");
            if is_main {
                main = Some(def);
            }
            let overflow = self.overflow_policy(&item.attrs, item.span);
            functions.push(Function {
                def,
                name,
                symbol: mangle(name, is_main),
                params,
                locals: std::mem::take(&mut self.locals),
                ret: self.ret_ty,
                body,
                span: item.span,
                overflow,
                borrows: self.signatures[def.0 as usize].borrows.clone(),
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
        while !self.pending.is_empty() || !self.pending_methods.is_empty() {
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

                let saved = std::mem::replace(self.sink, std::mem::take(&mut quiet));
                let function = self.check_one_function(decl, block, instance, &item.attrs, item.span);
                quiet = std::mem::replace(self.sink, saved);
                self.type_params.clear();
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
                    &item.attrs,
                    member.span,
                );
                if let Some(saved) = saved {
                    quiet = std::mem::replace(self.sink, saved);
                }
                self.type_params.clear();
                if let Some(function) = function {
                    out.push(function);
                }
            }
        }
        out
    }

    /// One function body, checked into a `Function` with a given `DefId`.
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
        self.ret_ty = self.signatures[def.0 as usize].ret;

        let signature_params: Vec<(Symbol, Ty, Mode, Span)> = self.signatures[def.0 as usize]
            .params
            .iter()
            .map(|(n, t, m, s)| (*n, *t, *m, *s))
            .collect();
        let mut params = Vec::new();
        for (name, ty, mode, param_span) in signature_params {
            let local_ty = match mode {
                Mode::Mut => self.mut_param_ty(ty),
                _ => ty,
            };
            let local = self.declare(Some(name), local_ty, param_span);
            params.push(Param { local, mode });
        }
        let body = self.check_block(block);
        let overflow = self.overflow_policy(attrs, span);
        // `[MONO-1]` — the symbol carries the instantiation, so two of them
        // never collide and identical ones dedupe at link time.
        let name = self.qualified(decl.name.name);
        Function {
            def,
            name,
            symbol: format!("{}__{}", ember_branding::mangled(name.as_str()), def.0),
            params,
            locals: std::mem::take(&mut self.locals),
            ret: self.ret_ty,
            body,
            span,
            overflow,
            borrows: self.signatures[def.0 as usize].borrows.clone(),
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
                let Some(entry) = self.methods.get(&(owner, decl.name.name)) else { continue };
                let def = entry.def;
                if let Some(function) =
                    self.check_one_method(owner, decl, block, def, &item.attrs, member.span)
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
            let interface = decl.name.name;
            for (ty, _, _) in implementations.iter().filter(|(_, i, _)| *i == interface) {
                for member in &decl.members {
                    let ast::MemberKind::Fn(fn_decl) = &member.kind else { continue };
                    let Some(block) = &fn_decl.body else { continue };
                    // Only the entries that came from this interface's default
                    // — a type that wrote its own method keeps that one.
                    let Some(entry) = self.methods.get(&(*ty, fn_decl.name.name)) else { continue };
                    if entry.from_interface != Some(interface) {
                        continue;
                    }
                    let def = entry.def;
                    if out.iter().any(|f: &Function| f.def == def) {
                        continue;
                    }
                    if let Some(function) =
                        self.check_one_method(*ty, fn_decl, block, def, &item.attrs, member.span)
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
        self.ret_ty = self.signatures[def.0 as usize].ret;

        let signature_params: Vec<(Symbol, Ty, Mode, Span)> = self.signatures[def.0 as usize]
            .params
            .iter()
            .map(|(n, t, m, s)| (*n, *t, *m, *s))
            .collect();
        let mut params = Vec::new();
        for (name, ty, mode, param_span) in signature_params {
            let local_ty = match mode {
                Mode::Mut => self.mut_param_ty(ty),
                _ => ty,
            };
            let local = self.declare(Some(name), local_ty, param_span);
            params.push(Param { local, mode });
        }

        let body = self.check_block(block);
        let overflow = self.overflow_policy(attrs, span);
        let name = decl.name.name;
        Some(Function {
            def,
            name,
            symbol: method_symbol(&self.types.display(owner), name),
            params,
            locals: std::mem::take(&mut self.locals),
            ret: self.ret_ty,
            body,
            span,
            overflow,
            borrows: self.signatures[def.0 as usize].borrows.clone(),
        })
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
        self.locals.push(LocalDecl { name, ty, span });
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
        let TyKind::Ref { inner, .. } = *self.types.kind(expr.ty) else { return expr };
        let span = expr.span;
        Expr { ty: inner, kind: ExprKind::Deref(Box::new(expr)), span }
    }

    fn lookup(&self, name: Symbol) -> Option<LocalId> {
        self.scopes.iter().rev().find_map(|scope| scope.get(&name).copied())
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
                out.push(Stmt::Let { local, init });
            }
            ast::StmtKind::Assign { targets, op, value } => {
                if targets.len() != 1 {
                    self.error(
                        codes::E1010,
                        stmt.span,
                        "tuple destructuring is not supported yet in this phase",
                    );
                    return;
                }
                let target = &targets[0];

                // `[GRM-4]` — a bare name that is not in scope declares.
                if let ast::ExprKind::Path { segments } = &target.kind {
                    if segments.len() == 1 && self.lookup(segments[0].name).is_none() {
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
                        out.push(Stmt::Let { local, init: Some(init) });
                        return;
                    }
                }

                let place = self.synth(target);
                // `[MOD-7]` — assignment and augmented assignment are both
                // writes.
                self.reject_readonly_write(&place, target.span);
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
                out.push(Stmt::Assign { place, value });
            }
            ast::StmtKind::If(if_stmt) => {
                let stmt = self.check_if(if_stmt);
                out.push(stmt);
            }
            ast::StmtKind::While { label, cond, body, else_block } => {
                let cond = self.check_condition(cond);
                self.loop_labels.push(label.map(|l| l.name));
                let body = self.check_block(body);
                self.loop_labels.pop();
                let else_block = else_block.as_ref().map(|b| self.check_block(b));
                out.push(Stmt::While { cond, body, else_block });
            }
            ast::StmtKind::For { label, pattern, iter, body, else_block } => {
                if let Some(stmt) = self.check_for(*label, pattern, iter, body, else_block, stmt.span)
                {
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
        let scrutinee = self.synth_committed(scrutinee);
        let scrutinee_ty = scrutinee.ty;

        let mut checked: Vec<hir::MatchArm> = Vec::new();
        let mut result_ty = expected;
        for arm in arms {
            self.scopes.push(HashMap::new());
            let pattern = self.check_pattern(&arm.pattern, scrutinee_ty);
            let guard = arm.guard.as_ref().map(|g| {
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
            checked.push(hir::MatchArm { pattern, guard, body, span: arm.span });
        }

        self.report_match_coverage(&checked, scrutinee_ty, span);

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

    /// `for x in xs` over an `Array[T]`:
    ///
    /// ```text
    /// __xs = xs
    /// for __i in 0..__xs.len():
    ///     x = __xs[__i]
    ///     <body>
    /// ```
    ///
    /// A counted loop, so iterating a collection costs an index and a bounds
    /// check rather than an iterator object — the same shape `[CTL-3]` asks
    /// for over a range.
    fn check_for_array(
        &mut self,
        label: Option<ast::Ident>,
        pattern: &ast::Pattern,
        source: (Expr, Ty),

        body: &ast::Block,
        else_block: &Option<ast::Block>,
        span: Span,
    ) -> Option<Stmt> {
        let (iterable, elem) = source;
        let usize_ty = self.common.usize;
        self.scopes.push(HashMap::new());
        let xs_local = self.declare(Some(Symbol::intern("__xs")), iterable.ty, span);
        let index_local = self.declare(Some(Symbol::intern("__i")), usize_ty, span);

        let xs = |ty: Ty| Expr { ty, kind: ExprKind::Local(xs_local), span };
        let length = Expr {
            ty: usize_ty,
            kind: ExprKind::Builtin {
                which: Builtin::ArrayLen,
                args: vec![xs(iterable.ty)],
            },
            span,
        };

        // The loop variable is the element, read at the index.
        self.scopes.push(HashMap::new());
        let item_local = self.declare(binding_name(pattern), elem, pattern.span);
        self.loop_labels.push(label.map(|l| l.name));
        let mut inner = vec![Stmt::Let {
            local: item_local,
            init: Some(Expr {
                ty: elem,
                kind: ExprKind::Index {
                    base: Box::new(xs(iterable.ty)),
                    index: Box::new(Expr {
                        ty: usize_ty,
                        kind: ExprKind::Local(index_local),
                        span,
                    }),
                },
                span,
            }),
        }];
        let checked = self.check_block(body);
        self.loop_labels.pop();
        self.scopes.pop();
        inner.extend(checked.stmts);

        let else_block = else_block.as_ref().map(|b| self.check_block(b));
        self.scopes.pop();

        Some(Stmt::Block(Block {
            stmts: vec![
                Stmt::Let { local: xs_local, init: Some(iterable) },
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
        let bool_ty = self.common.bool_;
        let done_local = self.declare(Some(Symbol::intern("__done")), bool_ty, iter.span);

        // `__it.next()`, checked through ordinary method resolution so that a
        // type without one is reported the same way any missing method is.
        let receiver = Expr { ty: iterable.ty, kind: ExprKind::Local(it_local), span: iter.span };
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

        let call = Expr {
            ty: item_option,
            kind: ExprKind::Call {
                callee: def,
                args: vec![self.pass_receiver(receiver, receiver_mode, iter.span)],
            },
            span: iter.span,
        };

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
        let cond = self.check_condition(&if_stmt.cond);
        let then_block = self.check_block(&if_stmt.then_block);
        let else_block = match if_stmt.else_block.as_deref() {
            Some(ast::ElseBranch::Block(b)) => Some(self.check_block(b)),
            Some(ast::ElseBranch::If(nested)) => {
                let stmt = self.check_if(nested);
                Some(Block { stmts: vec![stmt], span: if_stmt.then_block.span })
            }
            None => None,
        };
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
                let span = expr.span;
                return Expr {
                    ty: expected,
                    kind: ExprKind::Builtin {
                        which: Builtin::SpanFrom { mutable },
                        args: vec![expr],
                    },
                    span,
                };
            }
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

    /// The constant value of an already-checked expression, where it has one.
    /// This is the seed of `[RNG-4]`'s range tracking: the analysis that
    /// derives a range for a non-constant expression is not built, and until
    /// it is, "a value whose known range is contained in the target's" means a
    /// constant.
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
                let TyKind::Struct(id) = *self.types.kind(base.ty) else {
                    if base.ty != self.common.error {
                        let shown = self.types.display(base.ty);
                        self.error(
                            codes::E2020,
                            span,
                            format!("`{shown}` has no field `{}`", name.name),
                        );
                    }
                    return Expr { ty: self.common.error, kind: ExprKind::Error, span };
                };
                match self.types.struct_def(id).field(name.name) {
                    Some((index, field)) => {
                        let ty = field.ty;
                        let (vis, fname) = (field.vis, field.name);
                        // `[MOD-2]` — "All items are private to their module
                        // unless `pub`." `pub(package)` is visible everywhere
                        // in this build, since a build is one package until
                        // `[MAN-2]`'s dependency graph exists.
                        if vis == FieldVis::Private
                            && self.types.struct_def(id).declaring_module != self.current_module
                        {
                            let owner = self.types.struct_def(id).name.to_string();
                            self.sink.emit(
                                Diagnostic::error(
                                    codes::E1020,
                                    name.span,
                                    format!("`{fname}` is private to `{owner}`'s module"),
                                )
                                .help(format!(
                                    "declare it `pub {fname}: …` to read it anywhere, or                                      `pub(read) {fname}: …` to make it readable and not writable"
                                ))
                                .note("a field is private unless it says otherwise [MOD-2]"),
                            );
                        }
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
                let base = self.synth(base);
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
                let base = self.synth(base);
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
            ast::ExprKind::MethodCall { recv, name, args, .. }
                if self.namespace_named(recv).is_some() =>
            {
                let module = self.namespace_named(recv).expect("just checked");
                let prefix = &self.prefixes[module];
                let qualified = if prefix.is_empty() {
                    name.name
                } else {
                    Symbol::intern(&format!("{prefix}.{}", name.name))
                };
                self.synth_qualified_call(qualified, name.span, args, span)
            }

            ast::ExprKind::MethodCall { recv, name, args, .. }
                if self.enum_named(recv).is_some() =>
            {
                let id = self.enum_named(recv).expect("just checked");
                self.synth_variant(id, *name, args, span)
            }

            // `[RNG-10]`'s construction set: `Roughness.checked(x)`,
            // `Roughness.clamped(x)`, `unsafe Roughness.new_unchecked(x)`.
            // A type name on the left is an associated function, not a
            // receiver, so it is matched here beside the variant constructor
            // rather than in `[TYP-24]`'s method resolution.
            ast::ExprKind::MethodCall { recv, name, args, .. }
                if self.range_named(recv).is_some() =>
            {
                let id = self.range_named(recv).expect("just checked");
                self.synth_range_construction(id, *name, args, span)
            }

            // `[TYP-24]`, Part IV.11 — `recv.m(args)`.
            ast::ExprKind::MethodCall { recv, name, args, .. } => {
                self.synth_method_call(recv, *name, args, span)
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

        // `[ERR-1]` — `Some(x)`, `Ok(x)` and `Err(e)` build the compiler-known
        // enums. The expected type says which one when it is known; otherwise
        // the payload's own type decides and the other parameter stays open,
        // which needs an annotation.
        if let Some(built) = self.synth_wrapper(name, args, expected, span) {
            return built;
        }

        // `Pair(1, 2.5)` — a generic struct's constructor, with the type
        // arguments inferred from the values, or written as `Pair[i32, f32]`.
        if let Some(decl) = self.generic_structs.get(&self.resolve_name(name)).cloned() {
            return self.synth_generic_struct_literal(name, &decl, args, &explicit, span);
        }

        // `[UNS-5]`, `std.mem` — the raw memory primitives.
        if let Some(built) = self.synth_memory_builtin(name, args, &explicit, span) {
            return built;
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
        let checked = args
            .iter()
            .zip(signature.iter())
            .map(|(arg, &(param_ty, mode))| self.check_argument(&arg.value, param_ty, mode))
            .collect();
        Expr { ty: ret, kind: ExprKind::Call { callee: def, args: checked }, span }
    }

    /// `f[i32]` writes its type argument in expression position, so the
    /// argument arrives as an expression and has to be read back as a type.
    fn type_from_expr(&mut self, expr: &ast::Expr) -> Ty {
        let ast::ExprKind::Path { segments } = &expr.kind else {
            self.error(codes::E1010, expr.span, "expected a type argument");
            return self.common.error;
        };
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
        let declared: Vec<(Ty, Mode)> =
            self.signatures[def.0 as usize].params.iter().map(|(_, t, m, _)| (*t, *m)).collect();
        let ret = self.signatures[def.0 as usize].ret;

        if args.len() != declared.len() {
            self.error(
                codes::E2020,
                span,
                format!("`{name}` takes {} arguments, found {}", declared.len(), args.len()),
            );
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }

        // Explicit arguments come first; the rest are inferred by unifying
        // each declared parameter type with what the argument actually is.
        let mut solved: Vec<Option<Ty>> = vec![None; generics.len()];
        for (slot, ty) in explicit.iter().enumerate() {
            if slot < solved.len() {
                solved[slot] = Some(*ty);
            }
        }
        let mut checked_args: Vec<Expr> = Vec::new();
        for (arg, &(param_ty, _)) in args.iter().zip(declared.iter()) {
            let value = self.synth_committed(&arg.value);
            if !self.types.unify(param_ty, value.ty, &mut solved) {
                let want = self.types.display(param_ty);
                let got = self.types.display(value.ty);
                self.error(
                    codes::E2020,
                    arg.value.span,
                    format!("`{name}` cannot take `{got}` where it expects `{want}`"),
                );
            }
            checked_args.push(value);
        }

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

        // Re-check the arguments against the substituted parameter types, so
        // an untyped literal adopts the right one and a mismatch is reported
        // where it happens.
        let instance = self.instantiate(def, &substitution, name, span);
        let concrete: Vec<(Ty, Mode)> = declared
            .iter()
            .map(|&(ty, mode)| (self.substitute_ty(ty, &substitution), mode))
            .collect();
        let checked = args
            .iter()
            .zip(concrete.iter())
            .map(|(arg, &(param_ty, mode))| self.check_argument(&arg.value, param_ty, mode))
            .collect();
        let ret = self.substitute_ty(ret, &substitution);
        let _ = checked_args;
        Expr { ty: ret, kind: ExprKind::Call { callee: instance, args: checked }, span }
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
        if self.implemented.iter().any(|(t, i, _)| *t == ty && *i == interface) {
            return true;
        }
        // A bound naming an interface nothing declares cannot be satisfied;
        // the declaration site already reported that.
        !self.interfaces.contains_key(&interface)
    }

    /// `[MONO-1]` — one `DefId` per (function, type arguments), created once
    /// and named deterministically.
    fn instantiate(
        &mut self,
        def: DefId,
        args: &[Ty],
        name: Symbol,
        span: Span,
    ) -> DefId {
        let key = Instance { def, args: args.to_vec() };
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
        span: Span,
    ) -> Expr {
        let Some(&def) = self.fn_ids.get(&qualified) else {
            self.error(codes::E1010, name_span, format!("cannot find `{qualified}`"));
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };
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
        let checked = args
            .iter()
            .zip(signature.iter())
            .map(|(arg, &(param_ty, mode))| self.check_argument(&arg.value, param_ty, mode))
            .collect();
        Expr { ty: ret, kind: ExprKind::Call { callee: def, args: checked }, span }
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
    /// A `MutSpan[T]` is the exception, and Part VII §7's own example is what
    /// forces it: `fn normalize(mut xs: MutSpan[f32])` is called as
    /// `normalize(buf.as_mut_span())`, whose argument is a call result and not
    /// a place at all. A `MutSpan` **is** the mutable access — it carries the
    /// pointer, and `[SPN-3]` makes it move-only so there is exactly one — so
    /// `mut` on one means "you may write through it", and the place
    /// requirement lands on whatever the view was taken of. Wrapping it would
    /// make a reference to a reference and reject the document's own example.
    /// ADR-017.
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
            ExprKind::Builtin { which: Builtin::SpanFrom { .. }, args } => {
                args.first().is_some_and(|a| is_place(&a.kind))
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
    fn reject_readonly_write(&mut self, place: &Expr, span: Span) {
        let mut current = place;
        loop {
            match &current.kind {
                ExprKind::Field { base, index } => {
                    if let TyKind::Struct(id) = *self.types.kind(base.ty) {
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

    /// Part IV.11 — `recv.m(args)`. The receiver's type decides which method
    /// runs; `[TYP-24]` prefers an inherent method over an interface one, and
    /// two interfaces offering the name is `E2070`.
    fn synth_method_call(
        &mut self,
        recv: &ast::Expr,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let receiver = self.synth_committed(recv);
        if receiver.ty == self.common.error {
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        // `Array` and `String` carry their methods in the compiler until
        // Phase 2's generics let the standard library declare them.
        if let TyKind::Vec { elem } = *self.types.kind(receiver.ty) {
            return self.synth_vec_method(receiver, elem, name, args, span);
        }
        if let TyKind::Span { elem, mutable } = *self.types.kind(receiver.ty) {
            return self.synth_span_method(receiver, elem, mutable, name, args, span);
        }
        // `[TYP-17]` — on a generic parameter, only what its bounds provide
        // is permitted, and that is exactly what is looked up.
        if let TyKind::Param { index, name: param } = *self.types.kind(receiver.ty) {
            return self.synth_bound_method(index, param, receiver, name, args, span);
        }
        let Some(entry) = self.methods.get(&(receiver.ty, name.name)) else {
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
        let mut checked = vec![self.pass_receiver(receiver, receiver_mode, recv.span)];
        for (arg, &(param_ty, mode)) in args.iter().zip(signature.iter().skip(1)) {
            checked.push(self.check_argument(&arg.value, param_ty, mode));
        }
        Expr { ty: ret, kind: ExprKind::Call { callee: def, args: checked }, span }
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
            if let Some((_, method, receiver_mode, _)) =
                def.methods.iter().find(|(m, _, _, _)| *m == name.name)
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
                .filter(|(_, d)| d.methods.iter().any(|(m, _, _, _)| *m == name.name))
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
        for (arg, &(param_ty, mode)) in args.iter().zip(signature.iter()) {
            checked.push(self.check_argument(&arg.value, param_ty, mode));
        }
        Expr { ty: ret, kind: ExprKind::Call { callee: def, args: checked }, span }
    }

    /// The compiler-known methods on `Array[T]` and `String`.
    /// `[SPN-2]`, `[SPN-3]` — the methods on `Span[T]` and `MutSpan[T]`.
    ///
    /// Part VII §7 names the full set (`.len()`, `.iter()`, `.iter_mut()`,
    /// `.split_at(i)`, `.chunks(n)`, `.as_ptr()`); this is the part the
    /// compiler can answer without closures or an `Iterator` written in Ember,
    /// and a name it does not know is an error rather than a silent miss.
    fn synth_span_method(
        &mut self,
        receiver: Expr,
        elem: Ty,
        mutable: bool,
        name: ast::Ident,
        args: &[ast::Arg],
        span: Span,
    ) -> Expr {
        let usize_ty = self.common.usize;
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
            if view.ty != self.common.error && !self.viewed_place(&view) {
                self.error(
                    codes::E2140,
                    arg.span,
                    "a `mut` view must be taken of a variable, not of a value",
                );
            }
            return view;
        }
        let place = self.check_expr(arg, param_ty);
        // `[MOD-7]` — "passing `h.value` to a `mut` parameter or `mut self`
        // method" is a write.
        self.reject_readonly_write(&place, arg.span);
        if !is_place(&place.kind) && place.ty != self.common.error {
            self.error(
                codes::E2140,
                arg.span,
                "a `mut` argument must be a variable, not a value",
            );
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
        Expr { ty: ret, kind: ExprKind::Call { callee: def, args: vec![receiver, rhs] }, span }
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
        | ast::ItemKind::Comptime(_) => return None,
    })
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
