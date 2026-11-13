//! `[TYP-47]` (verdicts 115 and 116) — packs: any number of types (`[Ts...]`), and of values
//! (`values: Ts...`).
//!
//! How a pack is represented. In generic code a pack is a `TyKind::Param` whose name ends in
//! `ember_types::PACK_MARK` (`Ts…`, written `Ts...`): it stands for the pack's types spelled out
//! where it is written, among type arguments (`View[Cs...]`), a tuple's elements (`(Entity,
//! Cs...)`) and a function type's parameters (`fn(Ts...)`), and as the type of the parameter that
//! takes the pack's values. One value of the pack (`v` in `for v in values:`) is the parameter of
//! the same slot and plain name (`Ts`), which carries the pack's needs, so the two never meet. In an
//! instance the pack's argument is a `TyKind::Pack` of its types, which a substitution spells out
//! in each list it lands in (`TypeTable::tuple_of`, `pack_of`, `spelled_params`): a tuple of two or
//! more elements is a tuple, of one is that type, of none is `void` (`[TYP-27]`).
//!
//! A call of a function over a pack is checked as a call of the function made for that number of
//! values (`pack_arity`): the pack's slot replaced by one ordinary type parameter per value, each
//! with the pack's needs, and the pack's parameter by one parameter per value, each with its word.
//! So every value is checked as an ordinary argument is, its word included, and the instance it
//! asks for is the function's own over the pack's types (`[MONO-1]`), its symbol spelled with them
//! (`[MNG-6]`). Inside that instance the values are ordinary parameters, named `values[0]`,
//! `values[1]`, … (no program can write those names); `values...` passes them on, `values.len()`
//! is their number, and `for v in values:` is written out once per value (`check_pack_for`).

use super::*;

/// `[TYP-47]` — an open `for v in values:`, written out: the entries of `loop_labels` its jumps
/// reach. `outer` is the loop a `break` leaves (there when the body has one), `inner` the one
/// around each copy that a `continue` leaves (there when the body has one).
#[derive(Clone, Copy)]
pub(crate) struct PackLoop {
    pub(crate) outer: Option<usize>,
    pub(crate) inner: Option<usize>,
}

impl Checker<'_> {
    /// `[TYP-47]` — the name of a pack's `Param`: its own name and `PACK_MARK`.
    pub(crate) fn pack_param_name(name: Symbol) -> Symbol {
        Symbol::intern(&format!("{name}{}", ember_types::PACK_MARK))
    }

    /// `[TYP-47]` — a pack is the last of its type parameters, and a declaration has one: anywhere
    /// else, and a second, are `E0100`.
    pub(crate) fn check_pack_placement(&mut self, params: &[ast::GenericParam]) {
        let packs: Vec<&ast::GenericParam> = params.iter().filter(|param| param.pack).collect();
        let Some(first) = packs.first() else { return };
        if let Some(second) = packs.get(1) {
            self.sink.emit(
                Diagnostic::error(
                    codes::E0100,
                    second.span,
                    format!("a declaration takes one pack, and `{}...` is one already", first.name.name),
                )
                .help("write the function over the pack it takes, and form the other types in its body")
                .note("a pack is the last of its type parameters [TYP-47]"),
            );
            return;
        }
        if params.last().is_some_and(|last| last.id != first.id) {
            let written: Vec<String> = params
                .iter()
                .filter(|param| param.id != first.id)
                .chain(std::iter::once(*first))
                .map(|param| self.written_line(param.span).unwrap_or_else(|| param.name.name.to_string()))
                .collect();
            self.sink.emit(
                Diagnostic::error(codes::E0100, first.span, format!("a pack is the last of its type parameters, and `{}...` is not", first.name.name))
                    .help(format!("write `[{}]`", written.join(", ")))
                    .note("a pack takes every type that follows it, so nothing can come after it [TYP-47]"),
            );
        }
        if let Some(default) = &first.default {
            self.sink.emit(
                Diagnostic::error(codes::E0100, default.span, format!("`{}...` is a pack, any number of types, none included, so it has no default", first.name.name))
                    .help(format!("remove `= …` after `{}...`", first.name.name)),
            );
        }
    }

    /// `[TYP-47]` — `name...` among a list's types: the pack's types, spelled out there. In generic
    /// code that is the pack parameter itself; in an instance, the `Pack` of its types.
    pub(crate) fn resolve_spread(&mut self, name: ast::Ident, span: Span) -> Ty {
        match self.type_params.get(&name.name).copied() {
            Some(ty) if self.types.is_pack_param(ty) || matches!(self.types.kind(ty), TyKind::Pack(_)) => ty,
            Some(_) => {
                self.sink.emit(
                    Diagnostic::error(codes::E0100, span, format!("`{0}...` spells a pack's types out, and `{0}` is one type", name.name))
                        .help(format!("to take any number of types, make it a pack: `[{}...]`", name.name))
                        .note("`...` is written after a pack's name only [TYP-47]"),
                );
                self.common.error
            }
            None => {
                self.sink.emit(
                    Diagnostic::error(codes::E0100, span, format!("`{0}...` spells a pack's types out, and `{0}` is no pack", name.name))
                        .help(format!("a pack is a type parameter written with `...`: `[{}...]` above the function, struct or class", name.name))
                        .note("`...` is written after a pack's name only [TYP-47]"),
                );
                self.common.error
            }
        }
    }

    /// `[TYP-47]` — one of a list's types: `Cs...` spells the pack out, anything else is a type.
    pub(crate) fn resolve_list_item(&mut self, ty: &ast::TypeExpr) -> Ty {
        match &ty.kind {
            ast::TypeKind::Spread(name) => self.resolve_spread(*name, ty.span),
            _ => self.resolve_type(ty),
        }
    }

    /// `[TYP-47]` — a type argument: `Cs...` spells the pack out, anything else is as written.
    pub(crate) fn resolve_type_argument(&mut self, ty: &ast::TypeExpr) -> Ty {
        match &ty.kind {
            ast::TypeKind::Spread(name) => self.resolve_spread(*name, ty.span),
            _ => self.generic_type_argument(ty),
        }
    }

    /// Whether `ty` is a pack's types: a `Pack`, or a pack parameter in generic code.
    pub(crate) fn is_pack_like(&self, ty: Ty) -> bool {
        self.types.is_pack_param(ty) || matches!(self.types.kind(ty), TyKind::Pack(_))
    }

    /// `[TYP-47]` — a `Cs...` among the type arguments of a generic that takes no pack there:
    /// `E0100`. `None` when every argument fits.
    pub(crate) fn refuse_pack_argument(&mut self, generic: Symbol, args: &[(Ty, Span)]) -> Option<Ty> {
        let (_, span) = *args.iter().find(|(ty, _)| self.is_pack_like(*ty))?;
        let takes = self
            .generic_structs
            .get(&generic)
            .map(|decl| decl.generic_params.iter().any(|param| param.pack))
            .or_else(|| self.generic_classes.get(&generic).map(|decl| decl.generic_params.iter().any(|param| param.pack)))
            .unwrap_or(false);
        if takes {
            return None;
        }
        let shown = generic.as_str().rsplit('.').next().unwrap_or(generic.as_str()).to_string();
        self.sink.emit(
            Diagnostic::error(codes::E0100, span, format!("`{shown}` takes no pack, and a pack's types spelled out may be any number"))
                .help(format!("name each of the types `{shown}` takes"))
                .note("a pack is spelled out among the type arguments of a function, struct or class that takes one, a tuple's elements and a function type's parameters [TYP-47]"),
        );
        Some(self.common.error)
    }

    /// `[TYP-47]` — a declared tuple with a pack among its elements (`(int, Cs...)`), met by
    /// `actual`: the pairs of the other elements to meet, the pack, and the `Pack` of the elements
    /// they leave (`(int, f64, StrView)` leaves two, `int` none). `None` when `declared` has no pack
    /// or `actual` has too few elements.
    pub(crate) fn pack_tuple_parts(&mut self, declared: Ty, actual: Ty) -> Option<(Vec<(Ty, Ty)>, Ty, Ty)> {
        let TyKind::Tuple(items) = self.types.kind(declared).clone() else { return None };
        let at = items.iter().position(|&item| self.types.is_pack_param(item))?;
        let met: Vec<Ty> = match self.types.kind(actual).clone() {
            TyKind::Tuple(have) => have,
            TyKind::Void => Vec::new(),
            _ => vec![actual],
        };
        let after = items.len() - at - 1;
        if met.len() < at + after {
            return None;
        }
        let tail = met.len() - after;
        let pairs = items[..at].iter().copied().zip(met[..at].iter().copied()).chain(items[at + 1..].iter().copied().zip(met[tail..].iter().copied())).collect();
        let pack = self.types.pack_of(met[at..tail].to_vec());
        Some((pairs, items[at], pack))
    }

    /// `[TYP-47]` — a generic's arguments with those of its pack (the last of `params`) made one
    /// `Pack`, as every use of it names the instance: `View[A, B]` and the `View[Cs...]` a
    /// substitution gives are one argument list, `[Pack(A, B)]`. `None` when it has no pack.
    pub(crate) fn group_pack_arguments(&mut self, params: &[GenericParam], args: &[Ty]) -> Option<Vec<Ty>> {
        let at = params.iter().position(|param| param.pack)?;
        if args.len() < at {
            return None;
        }
        let mut grouped: Vec<Ty> = args[..at].to_vec();
        grouped.push(self.types.pack_of(args[at..].to_vec()));
        Some(grouped)
    }

    /// `[TYP-47]` — a function's parameters over its pack, where they are written: one takes the
    /// pack's values, the function's last positional one; one after it has a default and is passed
    /// by name only. Anywhere else, or two, is `E0100`.
    pub(crate) fn check_pack_values_placement(&mut self, decl: &ast::FnDecl) {
        fn named(param: &ast::Param) -> Option<(ast::Ident, &ast::TypeExpr)> {
            match &param.kind {
                ast::ParamKind::Named { name, ty } => Some((*name, ty)),
                ast::ParamKind::Receiver { .. } => None,
            }
        }
        let mut values: Option<(usize, ast::Ident)> = None;
        for (index, param) in decl.params.iter().enumerate() {
            let Some((name, ty)) = named(param) else { continue };
            let ast::TypeKind::Spread(pack) = &ty.kind else { continue };
            if let Some((_, first)) = values {
                self.sink.emit(
                    Diagnostic::error(codes::E0100, param.span, format!("a function takes one pack's values, and `{}` takes them already", first.name))
                        .note("a pack's values are the function's last positional parameter [TYP-47]"),
                );
                continue;
            }
            // A name that is no pack at all is reported where the type is read (`resolve_spread`).
            if !decl.generics.iter().any(|generic| generic.name.name == pack.name)
                && self.type_params.get(&pack.name).is_some_and(|&ty| self.is_pack_like(ty))
            {
                self.sink.emit(
                    Diagnostic::error(codes::E0100, ty.span, format!("`{}` takes a pack's values, and `{}` is not a pack of `{}`", name.name, pack.name, decl.name.name))
                        .help(format!("declare the pack above the function: `[{}...]`", pack.name))
                        .note("a function takes the values of a pack of its own [TYP-47]"),
                );
            }
            if let Some(default) = &param.default {
                self.sink.emit(
                    Diagnostic::error(codes::E0100, default.span, format!("`{}` takes a pack's values, which have no default: a call gives any number, none included", name.name))
                        .help(format!("remove `= …` after `{}`", name.name)),
                );
            }
            values = Some((index, name));
        }
        let Some((at, values)) = values else { return };
        for param in &decl.params[at + 1..] {
            let Some((name, _)) = named(param) else { continue };
            if param.default.is_none() {
                self.sink.emit(
                    Diagnostic::error(
                        codes::E0100,
                        param.span,
                        format!("`{}` follows `{}`, a pack's values, so it is passed by name and needs a default", name.name, values.name),
                    )
                    .help(format!("give `{}` a default (`{}: … = …`), or move it before `{}`", name.name, name.name, values.name))
                    .note("a parameter after a pack's values has a default and is passed by name only, as after Python's `*args` [TYP-47]"),
                );
            }
        }
    }

    /// `[TYP-47]` — `def` made for `n` values of its pack: the pack's slot replaced by `n` ordinary
    /// type parameters, each with the pack's needs, and its values' parameter by `n` parameters,
    /// each with the parameter's word. A call is checked against it as against any generic; its
    /// instances are `def`'s (`instantiate`). `None` when `def` takes no pack.
    pub(crate) fn pack_arity(&mut self, def: DefId, n: usize) -> Option<DefId> {
        if let Some(&made) = self.pack_arity_of.get(&(def, n)) {
            return Some(made);
        }
        let template = self.signatures[def.0 as usize].clone();
        let at = template.generics.iter().position(|param| param.pack)?;
        let pack = template.generics[at].clone();
        // What each of the template's parameters is in the function made for `n`.
        let mut map = Vec::with_capacity(template.generics.len());
        for (index, param) in template.generics.iter().enumerate() {
            let moved = if index > at { index + n - 1 } else { index } as u32;
            let ty = if index == at {
                let values: Vec<Ty> =
                    (0..n).map(|j| self.types.intern(TyKind::Param { index: (at + j) as u32, name: pack.name })).collect();
                self.types.pack_of(values)
            } else if param.const_ty.is_some() {
                self.types.intern(TyKind::Const(ConstNorm::param(moved, param.name)))
            } else {
                self.types.intern(TyKind::Param { index: moved, name: param.name })
            };
            map.push(ty);
        }
        let mut generics = Vec::with_capacity(template.generics.len() + n);
        for (index, param) in template.generics.iter().enumerate() {
            let substituted = self.substitute_generic_param(param, &map);
            if index == at {
                for _ in 0..n {
                    generics.push(GenericParam { pack: false, ..substituted.clone() });
                }
            } else {
                generics.push(substituted);
            }
        }
        let mut params = Vec::with_capacity(template.params.len() + n);
        for &(name, ty, mode, span) in &template.params {
            if self.types.is_pack_param(ty) {
                for j in 0..n {
                    params.push((name, self.types.intern(TyKind::Param { index: (at + j) as u32, name: pack.name }), mode, span));
                }
            } else {
                params.push((name, self.substitute_ty(ty, &map), mode, span));
            }
        }
        let ret = self.substitute_ty(template.ret, &map);
        let made = DefId(self.signatures.len() as u32);
        self.signatures.push(Signature { params, ret, generics, ..template });
        if let Some(defaults) = self.param_defaults.get(&def).cloned() {
            self.param_defaults.insert(made, defaults);
        }
        self.pack_arity_of.insert((def, n), made);
        self.pack_arities.insert(made, (def, at, n));
        Some(made)
    }

    /// `[TYP-47]` — an instance's arguments as `def`'s made for `n` values gives them, with the
    /// pack's `n` made one `Pack` in its slot: the arguments of `def`'s own instance.
    pub(crate) fn pack_instance_arguments(&mut self, at: usize, n: usize, args: &[Ty], callable_values: &[Option<DefId>]) -> (Vec<Ty>, Vec<Option<DefId>>) {
        let mut grouped = args[..at].to_vec();
        grouped.push(self.types.pack_of(args[at..at + n].to_vec()));
        grouped.extend_from_slice(&args[at + n..]);
        let mut values: Vec<Option<DefId>> = callable_values.iter().take(at).copied().collect();
        values.push(None);
        values.extend(callable_values.iter().skip(at + n).copied());
        values.resize(grouped.len(), None);
        (grouped, values)
    }

    /// `[TYP-47]` — a call of `def`, a function over a pack: checked as the call of the function
    /// made for the number of values it passes (or of types its type arguments give the pack).
    /// `None` when `def` is no such function.
    pub(crate) fn synth_pack_call(
        &mut self,
        def: DefId,
        name: Symbol,
        args: &[ast::Arg],
        explicit: &[Ty],
        expected: Option<Ty>,
        span: Span,
    ) -> Option<Expr> {
        if self.pack_arities.contains_key(&def) {
            return None;
        }
        let signature = &self.signatures[def.0 as usize];
        let at = signature.generics.iter().position(|param| param.pack)?;
        let pack_name = signature.generics[at].name;
        let values_at = signature.params.iter().position(|&(_, ty, ..)| self.types.is_pack_param(ty));
        let values_name = values_at.map(|index| signature.params[index].0);
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let args = self.expand_spread_args(args);
        if let Some(values_name) = values_name
            && let Some(arg) = args.iter().find(|arg| arg.name.is_some_and(|written| written.name == values_name))
        {
            self.sink.emit(
                Diagnostic::error(codes::E2020, arg.span, format!("`{values_name}` takes a pack's values, which are passed by position"))
                    .help(format!("write the values as `{name}`'s last positional arguments, without `{values_name}=`")),
            );
            return Some(error);
        }
        let (explicit, _) = self.types.spelled_out(explicit.to_vec());
        let positional = args.iter().filter(|arg| arg.name.is_none()).count();
        let n = match (values_at, explicit.len() > at) {
            (Some(values_at), given) => {
                let Some(n) = positional.checked_sub(values_at) else {
                    {
                        // The parameters before the pack's values are the ones positional arguments fill.
                        let leading: Vec<Symbol> = self.signatures[def.0 as usize].params.iter().take(values_at).map(|(name, ..)| *name).collect();
                        self.report_arity_of(def, format!("`{name}`"), &leading, positional, span, &args);
                    }
                    return Some(error);
                };
                if given && explicit.len() - at != n {
                    self.error(
                        codes::E2020,
                        span,
                        format!(
                            "`{name}` is given {} types for `{pack_name}` and {n} values for them",
                            explicit.len() - at
                        ),
                    );
                    return Some(error);
                }
                n
            }
            (None, _) => explicit.len().saturating_sub(at),
        };
        let arity = self.pack_arity(def, n)?;
        for arg in &args {
            if let ast::ExprKind::Spread(_) = arg.value.kind {
                self.pack_spreads.insert(arg.value.span);
            }
        }
        Some(self.synth_generic_call_checked(arity, name, &args, explicit, expected, span))
    }

    /// `[TYP-47]` — `values...` among a call's arguments: in an instance, each of the pack's values
    /// as an argument of its own, in order; in generic code it stays, the whole pack in one place.
    pub(crate) fn expand_spread_args(&mut self, args: &[ast::Arg]) -> Vec<ast::Arg> {
        if !args.iter().any(|arg| matches!(arg.value.kind, ast::ExprKind::Spread(_))) {
            return args.to_vec();
        }
        let mut out = Vec::with_capacity(args.len());
        for arg in args {
            let ast::ExprKind::Spread(name) = arg.value.kind else {
                out.push(arg.clone());
                continue;
            };
            let Some(pack) = self.lookup(name.name).filter(|local| self.pack_values.contains_key(local)) else {
                self.refuse_spread(name, arg.value.span);
                out.push(ast::Arg { name: arg.name, value: ast::Expr { id: ast::NodeId::DUMMY, kind: ast::ExprKind::Error, span: arg.value.span }, span: arg.span });
                continue;
            };
            // In an instance the name stands for no value (`void`); in generic code, for them all.
            if self.locals[pack.0 as usize].ty != self.common.void {
                out.push(arg.clone());
                continue;
            }
            let count = self.pack_values[&pack].len();
            for index in 0..count {
                let value = ast::Ident { name: Self::pack_value_name(name.name, index), span: name.span };
                out.push(ast::Arg {
                    name: arg.name,
                    value: ast::Expr { id: ast::NodeId::DUMMY, kind: ast::ExprKind::Path { segments: vec![value] }, span: arg.value.span },
                    span: arg.span,
                });
            }
        }
        out
    }

    /// `[TYP-47]` — the name the `index`th value of the pack `values` has in an instance's body,
    /// which no program can write.
    pub(crate) fn pack_value_name(values: Symbol, index: usize) -> Symbol {
        Symbol::intern(&format!("{values}[{index}]"))
    }

    /// `[TYP-47]` — `values...` where nothing takes a pack's values.
    pub(crate) fn refuse_spread(&mut self, name: ast::Ident, span: Span) {
        let holds = self.lookup(name.name).is_some_and(|local| self.pack_values.contains_key(&local));
        let message = if holds {
            format!("`{}...` passes a pack's values on, to a function whose last positional parameter takes a pack's values", name.name)
        } else {
            format!("`{0}...` passes a pack's values on, and `{0}` holds none", name.name)
        };
        self.sink.emit(
            Diagnostic::error(codes::E2020, span, message)
                .help("pass the values one by one, or call a function over a pack: `[Ts...]` above it, `values: Ts...`")
                .note("`values...` passes on the values of a function's own `values: Ts...` [TYP-47]"),
        );
    }

    /// `[TYP-47]` — the local a pack's values are bound to, when `expr` names it.
    pub(crate) fn pack_named(&self, expr: &ast::Expr) -> Option<LocalId> {
        let ast::ExprKind::Path { segments } = &expr.kind else { return None };
        let [name] = segments.as_slice() else { return None };
        self.lookup(name.name).filter(|local| self.pack_values.contains_key(local))
    }

    /// `[TYP-47]` — `values.len()`: how many values the pack holds, a number known when compiling,
    /// as an untyped integer literal, so it takes the type its use wants.
    pub(crate) fn pack_len(&mut self, pack: LocalId, args: &[ast::Arg], span: Span) -> Expr {
        if !args.is_empty() {
            self.error(codes::E2020, span, "`len()` takes no arguments");
        }
        let count = self.pack_values[&pack].len();
        Expr { ty: self.types.intern(TyKind::IntLit), kind: ExprKind::Int(count as u128), span }
    }

    /// `[TYP-47]` — the name of a pack's values used otherwise than one at a time.
    pub(crate) fn refuse_pack_value(&mut self, name: Symbol, span: Span) -> Expr {
        self.sink.emit(
            Diagnostic::error(codes::E2020, span, format!("`{name}` holds a pack's values, each of its own type, so it is no one value"))
                .help(format!("take them one at a time with `for v in {name}:`, pass them on with `{name}...`, or count them with `{name}.len()`"))
                .note("a pack's values are used one at a time [TYP-47]"),
        );
        Expr { ty: self.common.error, kind: ExprKind::Error, span }
    }

    /// `[TYP-47]` — `for v in values:`, written out once per value, each `v` of its own type: `v`
    /// is the value's own parameter there, so nothing runs for the loop itself. A `continue` in it
    /// ends that copy (each copy is then a loop that runs once) and a `break` leaves them all (they
    /// are then inside one loop that runs once, whose `else` is the loop's).
    pub(crate) fn check_pack_for(
        &mut self,
        label: Option<ast::Ident>,
        pattern: &ast::Pattern,
        pack: LocalId,
        body: &ast::Block,
        else_block: &Option<ast::Block>,
        span: Span,
    ) -> Option<Stmt> {
        let name = match &pattern.kind {
            ast::PatternKind::Bind { name, .. } => Some(name.name),
            ast::PatternKind::Wild => None,
            _ => {
                self.sink.emit(
                    Diagnostic::error(codes::E2020, pattern.span, "a pack's values are taken one at a time, each by one name")
                        .help("write `for v in values:`, and take `v` apart in the body"),
                );
                return None;
            }
        };
        let values = self.pack_values[&pack].clone();
        let label_name = label.map(|label| label.name);
        let (breaks, continues) = pack_loop_jumps(&body.stmts, label_name);
        let bool_ty = self.common.bool_;
        let outer = breaks.then(|| {
            self.loop_labels.push(label_name);
            self.loop_labels.len() - 1
        });
        self.pack_loops.push(PackLoop { outer, inner: continues.then_some(self.loop_labels.len()) });
        let mut copies = Vec::with_capacity(values.len());
        for &value in &values {
            let mut scope = HashMap::new();
            if let Some(name) = name {
                scope.insert(name, value);
            }
            self.scopes.push(scope);
            if continues {
                self.loop_labels.push(if breaks { None } else { label_name });
            }
            let mut block = self.check_block(body);
            if continues {
                self.loop_labels.pop();
            }
            self.scopes.pop();
            if continues {
                block.stmts.push(Stmt::Break { depth: 0 });
                let yes = Expr { ty: bool_ty, kind: ExprKind::Bool(true), span };
                copies.push(Stmt::While { cond: yes, body: block, else_block: None });
            } else {
                copies.push(Stmt::Block(block));
            }
        }
        self.pack_loops.pop();
        if breaks {
            self.loop_labels.pop();
        }
        let else_block = else_block.as_ref().map(|block| self.check_block(block));
        if !breaks {
            copies.extend(else_block.map(Stmt::Block));
            return Some(Stmt::Block(Block { stmts: copies, span }));
        }
        // A loop that runs once: its `else` runs unless a `break` left it.
        let once = self.declare(None, bool_ty, span);
        let read = Expr { ty: bool_ty, kind: ExprKind::Local(once), span };
        let mut stmts = vec![Stmt::Assign { place: read.clone(), value: Expr { ty: bool_ty, kind: ExprKind::Bool(false), span } }];
        stmts.extend(copies);
        Some(Stmt::Block(Block {
            stmts: vec![
                Stmt::Let { local: once, init: Some(Expr { ty: bool_ty, kind: ExprKind::Bool(true), span }) },
                Stmt::While { cond: read, body: Block { stmts, span }, else_block },
            ],
            span,
        }))
    }

    /// `[STD-9]` (verdict 117) — `value.fmt_print(ref mut f)`, the compiler's `Printable` for a
    /// type with `Display` or `Debug`: the value's text, as `print` writes it (its `Display`, else
    /// its `Debug`), appended to the text `f` is building; `Ok(())`.
    pub(crate) fn synth_fmt_print(&mut self, receiver: Expr, formatter: &ast::Expr, span: Span) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let value = self.read_through(receiver);
        if !self.check_formattable(value.ty, value.span, "printing") {
            return error;
        }
        let written = match &formatter.kind {
            ast::ExprKind::RefOf { mutable: true, place } => place.as_ref(),
            _ => formatter,
        };
        let f = self.synth_committed(written);
        let f = self.read_through(f);
        let (Some(formatter_ty), Some(error_ty)) = (
            self.named_types.get(&Symbol::intern("std.fmt.Formatter")).copied(),
            self.named_types.get(&Symbol::intern("std.fmt.FmtError")).copied(),
        ) else {
            return error;
        };
        if f.ty != formatter_ty {
            if f.ty != self.common.error {
                let shown = self.types.display(f.ty);
                self.error(codes::E2020, formatter.span, format!("`fmt_print` writes through a `ref mut Formatter`, not a `{shown}`"));
            }
            return error;
        }
        let TyKind::Struct(id) = *self.types.kind(formatter_ty) else { return error };
        let Some(index) = self.types.struct_def(id).fields.iter().position(|field| field.name.is("out")) else { return error };
        let string_ty = self.common.string;
        let out_ty = self.types.struct_def(id).fields[index].ty;
        let at = formatter.span;
        let out = Expr { ty: out_ty, kind: ExprKind::Field { base: Box::new(f), index }, span: at };
        let text = Expr { ty: string_ty, kind: ExprKind::Deref(Box::new(out)), span: at };
        let target_ty = self.types.intern(TyKind::Ref { mutable: true, inner: string_ty });
        let target = Expr { ty: target_ty, kind: ExprKind::Ref { place: Box::new(text), mutable: true }, span: at };
        // A written `Display` is appended as the text its `fmt` writes, as `print` writes it.
        let value = match self.written_display_text(value) {
            Ok(text) => {
                let str_ty = self.common.str_;
                self.coerce(text, str_ty)
            }
            Err(value) => value,
        };
        let void = self.common.void;
        let append = Expr { ty: void, kind: ExprKind::Builtin { which: Builtin::Format, args: vec![target, value] }, span };
        let result_ty = self.result_of(void, error_ty);
        let TyKind::Enum(result_id) = *self.types.kind(result_ty) else { return error };
        let Some(ok) = self.types.enum_def(result_id).variants.iter().position(|variant| variant.name.is("Ok")) else { return error };
        let unit = self.void_value(span);
        let done = Expr { ty: result_ty, kind: ExprKind::EnumLit { enum_id: result_id, variant: ok, fields: vec![unit] }, span };
        Expr { ty: result_ty, kind: ExprKind::Block { block: Block { stmts: vec![Stmt::Expr(append)], span }, value: Box::new(done) }, span }
    }

    /// `[STD-9]` (verdict 117) — whether `ty` is `Printable`: whether printing a value of it is
    /// allowed, as `print` decides for each value (`synth_print_pieces`): its written `Display`
    /// (through a `Box`, or a `Cell` of a `Copy` value, ODR-109), or a `dyn` value's, or else the
    /// text the table gives it (`Display`'s, else `Debug`'s).
    pub(crate) fn printable(&self, ty: Ty) -> bool {
        let mut ty = ty;
        while let TyKind::Ref { inner, .. } = *self.types.kind(ty) {
            ty = inner;
        }
        if self.dyn_reaches(ty, "std.core.Display") {
            return true;
        }
        let mut reached = ty;
        let mut lent = false;
        while let Some(inner) = self.types.printed_as(reached) {
            lent |= self.cell_inner(reached).is_some_and(|held| !self.types.is_copy(held));
            reached = inner;
        }
        if self.written_display(reached) {
            return !lent;
        }
        self.is_formattable(ty)
    }

    /// `[STD-9]` — why `ty` is not `Printable`, as printing a value of it reports it.
    pub(crate) fn report_not_printable(&mut self, ty: Ty, span: Span) {
        let mut cell = ty;
        while let TyKind::Ref { inner, .. } = *self.types.kind(cell) {
            cell = inner;
        }
        let mut reached = cell;
        while let Some(inner) = self.types.printed_as(reached) {
            if self.cell_inner(reached).is_some_and(|held| !self.types.is_copy(held)) {
                cell = reached;
            }
            reached = inner;
        }
        if self.written_display(reached) && self.cell_inner(cell).is_some() {
            self.cell_lends_its_value(cell, "Display", span);
            return;
        }
        let _ = self.check_formattable(ty, span, "printing");
    }

    /// `[TYP-47]` — the loop a `break` or `continue` `depth` loops out reaches, when it is a written
    /// out `for v in values:`: the jump that leaves it.
    pub(crate) fn pack_jump(&self, depth: usize, is_break: bool) -> Option<Stmt> {
        let target = self.loop_labels.len().checked_sub(depth + 1)?;
        let found = self.pack_loops.iter().rev().find(|pack| pack.outer == Some(target) || pack.inner == Some(target))?;
        let at_inner = found.inner == Some(target);
        Some(match (is_break, at_inner) {
            // The copy's own loop is the innermost of the two: a `break` leaves the one around it.
            (true, true) => Stmt::Break { depth: depth + 1 },
            (true, false) => Stmt::Break { depth },
            (false, true) => Stmt::Break { depth },
            (false, false) => Stmt::Break { depth: depth - 1 },
        })
    }
}

/// `[TYP-47]` — whether `stmts` hold a `break` and a `continue` that reach the loop they are the
/// body of: unlabeled outside any loop of their own, or naming `label`. A lambda's or a local
/// function's are their own.
fn pack_loop_jumps(stmts: &[ast::Stmt], label: Option<Symbol>) -> (bool, bool) {
    let mut found = (false, false);
    for stmt in stmts {
        jumps_in_stmt(stmt, label, 0, &mut found);
    }
    found
}

fn jumps_in_stmt(stmt: &ast::Stmt, label: Option<Symbol>, depth: usize, found: &mut (bool, bool)) {
    let block = |block: &ast::Block, depth: usize, found: &mut (bool, bool)| {
        for stmt in &block.stmts {
            jumps_in_stmt(stmt, label, depth, found);
        }
    };
    match &stmt.kind {
        ast::StmtKind::Expr(expr) => jumps_in_expr(expr, label, depth, found),
        ast::StmtKind::If(if_stmt) => jumps_in_if(if_stmt, label, depth, found),
        ast::StmtKind::While { body, else_block, .. } | ast::StmtKind::For { body, else_block, .. } => {
            block(body, depth + 1, found);
            if let Some(else_block) = else_block {
                block(else_block, depth, found);
            }
        }
        ast::StmtKind::Match { arms, .. } => {
            for arm in arms {
                match &arm.body {
                    ast::MatchArmBody::Block(body) => block(body, depth, found),
                    ast::MatchArmBody::Expr(expr) => jumps_in_expr(expr, label, depth, found),
                }
            }
        }
        ast::StmtKind::With { body, .. }
        | ast::StmtKind::Defer(body)
        | ast::StmtKind::Unsafe(body)
        | ast::StmtKind::Safe(body)
        | ast::StmtKind::Comptime(body) => block(body, depth, found),
        _ => {}
    }
}

fn jumps_in_if(if_stmt: &ast::IfStmt, label: Option<Symbol>, depth: usize, found: &mut (bool, bool)) {
    for stmt in &if_stmt.then_block.stmts {
        jumps_in_stmt(stmt, label, depth, found);
    }
    match if_stmt.else_block.as_deref() {
        Some(ast::ElseBranch::Block(block)) => {
            for stmt in &block.stmts {
                jumps_in_stmt(stmt, label, depth, found);
            }
        }
        Some(ast::ElseBranch::If(nested)) => jumps_in_if(nested, label, depth, found),
        None => {}
    }
}

fn jumps_in_expr(expr: &ast::Expr, label: Option<Symbol>, depth: usize, found: &mut (bool, bool)) {
    let reaches = |written: &Option<ast::Ident>| match written {
        Some(written) => Some(written.name) == label,
        None => depth == 0,
    };
    match &expr.kind {
        ast::ExprKind::Jump(ast::Jump::Break { label: written }) if reaches(written) => found.0 = true,
        ast::ExprKind::Jump(ast::Jump::Continue { label: written }) if reaches(written) => found.1 = true,
        ast::ExprKind::Match { arms, .. } => {
            for arm in arms {
                match &arm.body {
                    ast::MatchArmBody::Block(body) => {
                        for stmt in &body.stmts {
                            jumps_in_stmt(stmt, label, depth, found);
                        }
                    }
                    ast::MatchArmBody::Expr(expr) => jumps_in_expr(expr, label, depth, found),
                }
            }
        }
        ast::ExprKind::Paren(inner) => jumps_in_expr(inner, label, depth, found),
        _ => {}
    }
}
