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
    CommonTypes, EnumDef, EnumId, FieldDef, OverflowPolicy, StructDef, StructId, Ty, TyKind, UintTy,
    TypeTable, VariantDef, int_max,
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
    Program { functions, main }
}

struct Signature {
    params: Vec<(Symbol, Ty, Mode, Span)>,
    ret: Ty,
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
    /// Method name, its signature, the receiver's mode, and whether the
    /// interface supplies a body.
    methods: Vec<(Symbol, Signature, Mode, bool)>,
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
            locals: Vec::new(),
            scopes: Vec::new(),
            or_bindings: None,
            loop_labels: Vec::new(),
            in_defer: false,
            ret_ty,
            default_overflow: OverflowPolicy::default(),
        }
    }

    fn error(&mut self, code: ember_diag::Code, span: Span, message: impl Into<String>) {
        self.sink.emit(Diagnostic::error(code, span, message));
    }

    // -- modules -------------------------------------------------------------

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
                if key.starts_with("std") {
                    // `[MOD-5]` — the prelude is implicit; its names are
                    // already built in.
                    continue;
                }
                let Some(&target) = by_path.get(&key) else {
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
        for item in &module.items {
            match &item.kind {
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
                                });
                            }
                            // `[STR-3]` — a `drop` method makes the type
                            // move-only, whatever `@derive(Copy)` says.
                            ast::MemberKind::Fn(f) if f.name.name.is("drop") => has_drop = true,
                            _ => {}
                        }
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
                    let params = decl
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
                    let def = DefId(self.signatures.len() as u32);
                    self.fn_ids.insert(name, def);
                    self.signatures.push(Signature { params, ret });
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

    /// A third collection pass: interfaces first, so that `implements` can
    /// name them, then every method on a type.
    fn collect_methods(&mut self, module: &ast::Module) {
        for item in &module.items {
            if let ast::ItemKind::Interface(decl) = &item.kind {
                self.collect_interface(decl, item.span);
            }
        }
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
                        self.method_signature(fn_decl, Some(*ty), member.span)
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
        let name = self.qualified(decl.name.name);
        if self.interfaces.contains_key(&name) || self.named_types.contains_key(&name) {
            self.error(
                codes::E1030,
                decl.name.span,
                format!("`{name}` is already declared in this module"),
            );
            return;
        }
        let mut methods = Vec::new();
        for member in &decl.members {
            let ast::MemberKind::Fn(f) = &member.kind else { continue };
            let Some((receiver, signature)) = self.method_signature(f, None, member.span) else {
                continue;
            };
            methods.push((f.name.name, signature, receiver, f.body.is_some()));
        }
        let supertraits = decl.supertraits.iter().filter_map(interface_name).collect();
        let _ = span;
        self.interfaces.insert(name, InterfaceDef { methods, supertraits });
    }

    /// Turn a method declaration into a signature. `self_ty` is `None` inside
    /// an interface, where the receiver's type is not yet known.
    fn method_signature(
        &mut self,
        decl: &ast::FnDecl,
        self_ty: Option<Ty>,
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
        Some((receiver, Signature { params, ret }))
    }

    /// Register every method in a type body or `extend` block.
    fn collect_members(
        &mut self,
        ty: Ty,
        members: &[ast::Member],
        from_interface: Option<Symbol>,
        span: Span,
    ) {
        for member in members {
            let ast::MemberKind::Fn(decl) = &member.kind else { continue };
            if decl.body.is_none() {
                continue;
            }
            let Some((receiver, signature)) = self.method_signature(decl, Some(ty), member.span)
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
            let Some(name) = interface_name(entry) else {
                self.error(codes::E1010, entry.span, "expected an interface name");
                continue;
            };
            if !self.interfaces.contains_key(&self.resolve_name(name)) {
                self.error(
                    codes::E1010,
                    entry.span,
                    format!("cannot find interface `{name}` in this scope"),
                );
                continue;
            }
            // `[TYP-19]` — the same interface implemented twice for one type.
            if self.implemented.iter().any(|(t, i, _)| *t == ty && *i == name) {
                let shown = self.types.display(ty);
                self.error(
                    codes::E2041,
                    entry.span,
                    format!("`{shown}` already implements `{name}`"),
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
                    return self.types.intern(TyKind::Vec { elem });
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
                    Mode::Mut => self.types.intern(TyKind::Ref { mutable: true, inner: ty }),
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
            });
        }

        functions.extend(self.check_method_bodies(module));
        Program { functions, main }
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
                Mode::Mut => self.types.intern(TyKind::Ref { mutable: true, inner: ty }),
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
    fn read_local(&mut self, local: LocalId, span: Span) -> Expr {
        let ty = self.locals[local.0 as usize].ty;
        let read = Expr { ty, kind: ExprKind::Local(local), span };
        match *self.types.kind(ty) {
            TyKind::Ref { mutable: true, inner } => {
                Expr { ty: inner, kind: ExprKind::Deref(Box::new(read)), span }
            }
            _ => read,
        }
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
            ast::StmtKind::Expr(expr) => {
                let expr = self.synth(expr);
                out.push(Stmt::Expr(expr));
            }
            ast::StmtKind::Return(value) => {
                if self.in_defer {
                    self.error(
                        codes::E2160,
                        stmt.span,
                        "control flow cannot leave a `defer` block",
                    );
                }
                let ret_ty = self.ret_ty;
                let value = match value {
                    Some(expr) => Some(self.check_expr(expr, ret_ty)),
                    None => {
                        if ret_ty != self.common.void {
                            let shown = self.types.display(ret_ty);
                            self.error(
                                codes::E2020,
                                stmt.span,
                                format!("this function returns `{shown}`, so `return` needs a value"),
                            );
                        }
                        None
                    }
                };
                out.push(Stmt::Return(value));
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
                let place_ty = place.ty;
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
            ast::StmtKind::Break { label, .. } => {
                if let Some(depth) = self.loop_depth(*label, "break", stmt.span) {
                    out.push(Stmt::Break { depth });
                }
            }
            ast::StmtKind::Continue { label, .. } => {
                if let Some(depth) = self.loop_depth(*label, "continue", stmt.span) {
                    out.push(Stmt::Continue { depth });
                }
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
            self.error(
                codes::E1010,
                iter.span,
                "`for` supports a range with both ends in this phase of the compiler",
            );
            return None;
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

    fn check_condition(&mut self, cond: &ast::Condition) -> Expr {
        match cond {
            ast::Condition::Expr(expr) => {
                let bool_ty = self.common.bool_;
                self.check_expr(expr, bool_ty)
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
    fn synth_committed(&mut self, expr: &ast::Expr) -> Expr {
        let expr = self.synth(expr);
        self.commit(expr)
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
                    Some(local) => self.read_local(local, span),
                    None => {
                        self.error(codes::E1010, span, "`self` outside a method");
                        Expr { ty: self.common.error, kind: ExprKind::Error, span }
                    }
                }
            }

            ast::ExprKind::Path { segments } if segments.len() == 1 => {
                let name = segments[0].name;
                if let Some(local) = self.lookup(name) {
                    return self.read_local(local, span);
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
                    TyKind::Array { elem, .. } | TyKind::Vec { elem } => Some(*elem),
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
                let usize_ty = self.common.usize;
                let index = self.check_expr(&args[0], usize_ty);
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
            ast::Literal::Float { value, suffix } => {
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

        // `[ERR-1]` — `Some(x)`, `Ok(x)` and `Err(e)` build the compiler-known
        // enums. The expected type says which one when it is known; otherwise
        // the payload's own type decides and the other parameter stays open,
        // which needs an annotation.
        if let Some(built) = self.synth_wrapper(name, args, expected, span) {
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
            let args: Vec<Expr> = args.iter().map(|a| self.synth_committed(&a.value)).collect();
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
        let mut checked = vec![self.pass_receiver(receiver, receiver_mode, recv.span)];
        for (arg, &(param_ty, mode)) in args.iter().zip(signature.iter().skip(1)) {
            checked.push(self.check_argument(&arg.value, param_ty, mode));
        }
        Expr { ty: ret, kind: ExprKind::Call { callee: def, args: checked }, span }
    }

    /// The compiler-known methods on `Array[T]` and `String`.
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
        let place = self.check_expr(arg, param_ty);
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

        // `[TYP-21]` — an operator on a non-scalar is an interface method
        // call. `a + b` on a `Vec3` is `a.add(b)`, with both sides passed in
        // the modes the interface declared.
        if let Some(method) = operator_method(op) {
            if self.methods.contains_key(&(lhs.ty, Symbol::intern(method))) {
                return self.call_operator(method, lhs, rhs, span);
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
    format!("em_{}_{name}", owner.trim_matches('_'))
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
        return "em_main".to_string();
    }
    // A qualified name carries dots, which C does not allow in an identifier.
    format!("em_{}", name.as_str().replace('.', "_"))
}
