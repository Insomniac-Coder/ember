//! `[BRW-12]`, `[BRW-13]`, `[BRW-15]` (A2-3) — what a name is. A name bound to a part is a
//! read-only view of it; `=` on a view name points it elsewhere, checked against its new target;
//! `const ref mut` is never given another ref; and a change through a name that only reads is
//! `E3023`, with the line to write.

use super::*;

/// How a view name was made.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ViewKind {
    /// `boss = enemies[0]`: a name bound to a part, which only reads it (`[BRW-12]`).
    Part,
    /// `r = ref x`, `r = ref mut x`, `r = const ref mut x`.
    Ref { mutable: bool, constant: bool },
    /// `const ref mut dst: MutSpan[float]`: a parameter that is never given another ref
    /// (`[BRW-14]`).
    ConstParam,
    /// `d = tracks[name].as_mut_span()`: a plain name holding a `MutSpan`, which only reads its
    /// elements (`[BRW-15]`, `[SPN-3]`).
    SpanHolder,
}

/// A view name, by the line that made it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ViewName {
    pub kind: ViewKind,
    /// The line that made it (`boss = enemies[0]`).
    pub line: Span,
    /// What it names, as written after its word (`enemies[0]`, `x`).
    pub target: Span,
    /// Its value as written, word included (`const ref mut a`, `enemies[0]`).
    pub value: Span,
    /// The place it names may be changed by this function, so `ref mut` before it would be accepted.
    pub changeable: bool,
}

/// Whether `expr` is a call, whose result a name may view.
fn is_call(expr: &Expr) -> bool {
    matches!(expr.kind, ExprKind::Call { .. } | ExprKind::InterfaceCall { .. } | ExprKind::CallIndirect { .. } | ExprKind::Builtin { .. })
}

impl Checker<'_> {
    /// Whether `expr` is a part of a place, what `[BRW-12]` makes a read-only view of: a field or an
    /// element of a place, what a read-only view reads (`x = r` for a `ref T` name or a plain
    /// parameter), or what a call that gives a view gives. A part of a temporary is not
    /// (`make().name` hands the field out of a value no name holds), and neither is what a `ref mut`
    /// name or an `owns T` reads, which `=` hands over as a whole name.
    pub(crate) fn is_part(&self, expr: &Expr) -> bool {
        fn rooted(expr: &Expr) -> bool {
            match &expr.kind {
                ExprKind::Local(_) | ExprKind::Deref(_) => true,
                ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => rooted(base),
                _ => false,
            }
        }
        match &expr.kind {
            ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => rooted(base),
            // `[OWN-3]` — a plain parameter is a view, and one holding a class object arrives as the
            // handle itself, not as a reference to it.
            ExprKind::Local(local) => {
                self.borrowed_params.contains(local) && matches!(self.types.kind(expr.ty), TyKind::Class(_) | TyKind::ClassInterface(_))
            }
            ExprKind::Deref(inner) => match *self.types.kind(inner.ty) {
                TyKind::Ref { mutable: false, .. } => matches!(inner.kind, ExprKind::Local(_)) || is_call(inner),
                TyKind::Ref { mutable: true, .. } => is_call(inner),
                _ => false,
            },
            _ => false,
        }
    }

    /// Whether this function may change the place `expr` names: it is not reached through a
    /// read-only view or a plain parameter.
    pub(crate) fn changeable_place(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Local(local) => !self.borrowed_params.contains(local),
            ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => {
                !matches!(self.types.kind(base.ty), TyKind::Span { mutable: false, .. }) && self.changeable_place(base)
            }
            // A call's view (`m[k]`, `first(…)`) is asked for again with `ref mut` before the call.
            ExprKind::Deref(inner) if is_call(inner) => true,
            ExprKind::Deref(inner) => match *self.types.kind(inner.ty) {
                TyKind::Ref { mutable, .. } => mutable && !matches!(&inner.kind, ExprKind::Local(local) if self.borrowed_params.contains(local)),
                _ => self.changeable_place(inner),
            },
            _ => false,
        }
    }

    /// `[BRW-12]` — a name bound to a part of a type that is not `Copy` is a read-only view of it:
    /// `boss = enemies[0]` is `boss = ref enemies[0]`, never a hand-over and never a copy. A part of a
    /// `Copy` type is copied, as `=` copies a whole name of that type.
    pub(crate) fn part_as_view(&mut self, init: Expr) -> (Expr, Option<bool>) {
        if !self.is_part(&init) || init.ty == self.common.error || self.copied_by_equals(init.ty) {
            return (init, None);
        }
        let changeable = self.changeable_place(&init);
        // What a view reads through is named by that view: `x = r` for a `ref T` local is one more
        // view of what `r` reads.
        if let ExprKind::Deref(inner) = &init.kind
            && let TyKind::Ref { mutable: false, .. } = *self.types.kind(inner.ty)
        {
            let ExprKind::Deref(inner) = init.kind else { unreachable!("matched above") };
            return (*inner, Some(changeable));
        }
        let ty = self.types.intern(TyKind::Ref { mutable: false, inner: init.ty });
        let span = init.span;
        (Expr { ty, kind: ExprKind::Ref { place: Box::new(init), mutable: false }, span }, Some(changeable))
    }

    /// Whether `=` copies a value of `ty` (`[OWN-7]`): a `Copy` type, a type parameter by its bounds
    /// included.
    pub(crate) fn copied_by_equals(&self, ty: Ty) -> bool {
        self.types.is_copy(ty) || self.implements(ty, Symbol::intern("std.core.Copy"))
    }

    /// Records the view name a declaration made: `r = ref x` and its kinds, a part name, or a plain
    /// name holding a `MutSpan`.
    pub(crate) fn note_view_name(&mut self, local: LocalId, value: &ast::Expr, line: Span, part: Option<bool>) {
        let ty = self.locals[local.0 as usize].ty;
        if matches!(self.types.kind(ty), TyKind::Span { mutable: true, .. }) {
            self.note_span_holder(local, value, line);
            return;
        }
        let (kind, target, changeable) = match (&value.kind, part) {
            (ast::ExprKind::RefOf { mutable, constant, place }, _) => (ViewKind::Ref { mutable: *mutable, constant: *constant }, place.span, true),
            (_, Some(changeable)) => (ViewKind::Part, value.span, changeable),
            _ => return,
        };
        self.view_names.insert(local, ViewName { kind, line, target, value: value.span, changeable });
    }

    /// `[BRW-15]`, `[SPN-3]` — a name given a `MutSpan` (`d = …`, `left, right = …`) writes its
    /// elements only where `ref mut` stands before the value; otherwise it only reads them.
    pub(crate) fn note_span_holder(&mut self, local: LocalId, value: &ast::Expr, line: Span) {
        if matches!(value.kind, ast::ExprKind::RefOf { mutable: true, .. }) {
            self.view_names.remove(&local);
            return;
        }
        self.view_names.insert(local, ViewName { kind: ViewKind::SpanHolder, line, target: value.span, value: value.span, changeable: true });
    }

    /// `[BRW-15]` — a write of an element through a plain name holding a `MutSpan`: `E3023`, with the
    /// line that gives the name `ref mut` (rewritten so by `ember __migrate --checker`: 0.9.10's
    /// span wrote through any name).
    pub(crate) fn span_holder_write(&self, local: LocalId, write: Span) -> Option<Diagnostic> {
        let view = self.view_names.get(&local).copied().filter(|view| view.kind == ViewKind::SpanHolder)?;
        let name = self.locals.get(local.0 as usize)?.name?;
        let value = self.written_line(view.value)?;
        let change = self.line_code(write).unwrap_or_else(|| "this".to_string());
        // What the span's elements belong to: the value a span was made from (`tracks[dst_name]`
        // for `tracks[dst_name].as_mut_span()`), else the span as written.
        let owner = match value.strip_suffix(".as_mut_span()") {
            Some(owner) => owner.to_string(),
            None => match value.rfind(".split_at(") {
                Some(at) if value.ends_with(')') => value[..at].strip_suffix(".as_mut_span()").unwrap_or(&value[..at]).to_string(),
                _ => value.clone(),
            },
        };
        let fix = self.line_with(view.value, &format!("ref mut {value}"))?;
        let message = format!("`{change}` writes an element of `{owner}`, but `{name}` only reads it; write `{fix}`");
        let at = Span::new(view.value.file, view.value.start, view.value.start);
        Some(
            Diagnostic::error(codes::E3023, write, message)
                .help(fix)
                .note("a plain name holding a `MutSpan` only reads its elements; `ref mut` before the value says it writes them [BRW-15], [SPN-3]")
                .migrate_edits("`ref mut` before a `MutSpan` a name writes through", vec![(at, ember_diag::Replacement::Text("ref mut ".to_string()))]),
        )
    }

    /// A call of the first method the interfaces declare, as the line to write in place of a whole-value `=`:
    /// `v.scale(2.0)`. Each argument is an example value of its parameter's type.
    fn dyn_change_example(&self, name: &str, interfaces: &[Symbol]) -> String {
        for interface in interfaces {
            let Some(def) = self.interfaces.get(interface) else { continue };
            let Some(&(method, declaration, ..)) = def.methods.iter().find(|(_, _, receiver, _)| receiver.is_some()) else { continue };
            let arguments: Vec<String> = self.signatures[declaration.0 as usize]
                .params
                .iter()
                .filter(|(parameter, ..)| !parameter.is("this"))
                .map(|(parameter, ty, ..)| {
                    if self.types.is_float(*ty) {
                        "2.0".to_string()
                    } else if self.types.is_integral(*ty) {
                        "1".to_string()
                    } else if matches!(self.types.kind(*ty), TyKind::Bool) {
                        "true".to_string()
                    } else {
                        parameter.to_string()
                    }
                })
                .collect();
            return format!("{name}.{method}({})", arguments.join(", "));
        }
        format!("{name}.method()")
    }

    /// `[BRW-14]` — a `const ref mut` parameter is never given another ref.
    pub(crate) fn note_const_parameter(&mut self, local: LocalId, span: Span) {
        self.view_names.insert(local, ViewName { kind: ViewKind::ConstParam, line: span, target: span, value: span, changeable: true });
    }

    /// `[BRW-13]` — `=` on a view name (`target`, the name `local`). A `ref mut` name given `ref mut
    /// b` now refers to `b`; a read-only view name given a view (`ref b`, a part, another view's
    /// name) views that instead; each is checked against its new target by `[BRW-1]`. A `ref mut`
    /// name given `ref b` is `E3023`, and so is a `const ref mut` name given any ref. `true` when
    /// the line was handled here; otherwise it writes the place the name stands for (`r = 20`).
    pub(crate) fn repoint(&mut self, local: LocalId, target: &ast::Expr, value: &ast::Expr, span: Span, out: &mut Vec<Stmt>) -> bool {
        let decl_ty = self.locals[local.0 as usize].ty;
        let Some(name) = self.locals[local.0 as usize].name else { return false };
        // A loop, pattern or opened name keeps what it was given (`[BRW-14]`): refused below.
        if self.kept_header(local).is_some() {
            return false;
        }
        let view = self.view_names.get(&local).copied();
        let given = match &value.kind {
            ast::ExprKind::RefOf { mutable, place, .. } => Some((*mutable, &**place)),
            _ => None,
        };
        if let Some(view) = view
            && matches!(view.kind, ViewKind::Ref { constant: true, .. } | ViewKind::ConstParam)
        {
            let Some((_, place)) = given else { return false };
            let message = self.const_ref_message(name, view, place.span, value.span);
            self.sink.emit(Diagnostic::error(codes::E3023, span, message).note("a `const ref mut` name always refers to the place it was made with [BRW-13]"));
            return true;
        }
        // A plain name holding a `MutSpan` given another one writes through it only if the line says
        // `ref mut`.
        if matches!(self.types.kind(decl_ty), TyKind::Span { mutable: true, .. }) && !self.borrowed_params.contains(&local) {
            self.note_span_holder(local, value, span);
            return false;
        }
        let TyKind::Ref { mutable, inner } = *self.types.kind(decl_ty) else { return false };
        // `[TYP-14]` — `=` on a `ref mut` name of a `dyn I` type never gives the place a whole value: its own
        // kind is known only while running, and its size is the kind's.
        if mutable && matches!(self.types.kind(inner), TyKind::Dyn { .. }) && given.is_none() {
            let TyKind::Dyn { interfaces } = self.types.kind(inner).clone() else { return false };
            let call = self.dyn_change_example(&name.to_string(), &interfaces);
            let boxed = self.types.display(inner);
            let message = format!(
                "`{name}` views a value of a kind known only while running; change it through its methods, `{call}`, or take the value over, `owns {name}: owns {boxed}`, and return the new one"
            );
            self.sink.emit(Diagnostic::error(codes::E3023, span, message).note("the place's own kind is known only while running, and its size is the kind's [TYP-14]"));
            return true;
        }
        // A plain parameter keeps what the caller gave (`[BRW-14]`): the line is refused there.
        if self.borrowed_params.contains(&local) {
            return false;
        }
        let new = match given {
            Some((false, place)) if mutable => {
                let target_text = self.written_line(place.span).unwrap_or_else(|| "it".to_string());
                let new_name = self.free_view_name(name, &target_text);
                let message = format!(
                    "`{name}` may change what it refers to, and `ref {target_text}` only reads `{target_text}`; to point `{name}` at `{target_text}` write `{name} = ref mut {target_text}`, or for a read-only name `{new_name} = ref {target_text}`"
                );
                self.sink.emit(Diagnostic::error(codes::E3023, span, message).note("the line that gives a `ref mut` name another ref says `ref mut` again [BRW-13]"));
                return true;
            }
            Some(_) => self.check_expr(value, decl_ty),
            // `r = 20` writes the place a `ref mut` name stands for.
            None if mutable => return false,
            None => {
                let quiet = self.sink.mark();
                let probe = self.synth(value);
                self.sink.rollback(quiet);
                if !self.is_view_value(&probe) {
                    return false;
                }
                let found = self.synth(value);
                let found = self.as_read_only_ref(found);
                match *self.types.kind(found.ty) {
                    TyKind::Ref { inner: viewed, .. } if viewed == inner => found,
                    _ => self.coerce(found, decl_ty),
                }
            }
        };
        let place = Expr { ty: decl_ty, kind: ExprKind::Local(local), span: target.span };
        out.push(Stmt::Assign { place, value: new });
        true
    }

    /// Whether a value found for a read-only view name is a view (a reference, what one reads, or a
    /// part that is not copied), which the name then views, rather than a value it would write.
    fn is_view_value(&self, expr: &Expr) -> bool {
        if matches!(self.types.kind(expr.ty), TyKind::Ref { .. }) {
            return true;
        }
        // What a view reads is a view too, unless it is copied (a number read through a plain
        // parameter is the number, however the parameter is passed, `[BRW-8]`).
        if let ExprKind::Deref(inner) = &expr.kind
            && matches!(self.types.kind(inner.ty), TyKind::Ref { .. })
        {
            return !self.copied_by_equals(expr.ty);
        }
        self.is_part(expr) && !self.copied_by_equals(expr.ty)
    }

    /// The read-only reference a view value gives: a reference as it is (a `ref mut` one read only),
    /// what a reference reads as that reference, a part as a view of it.
    fn as_read_only_ref(&mut self, expr: Expr) -> Expr {
        let span = expr.span;
        match *self.types.kind(expr.ty) {
            TyKind::Ref { mutable: false, .. } => return expr,
            TyKind::Ref { mutable: true, inner } => {
                let ty = self.types.intern(TyKind::Ref { mutable: false, inner });
                let read = Expr { ty: inner, kind: ExprKind::Deref(Box::new(expr)), span };
                return Expr { ty, kind: ExprKind::Ref { place: Box::new(read), mutable: false }, span };
            }
            _ => {}
        }
        if let ExprKind::Deref(inner) = &expr.kind
            && let TyKind::Ref { mutable: false, .. } = *self.types.kind(inner.ty)
        {
            let ExprKind::Deref(inner) = expr.kind else { unreachable!("matched above") };
            return *inner;
        }
        let ty = self.types.intern(TyKind::Ref { mutable: false, inner: expr.ty });
        Expr { ty, kind: ExprKind::Ref { place: Box::new(expr), mutable: false }, span }
    }

    /// `[BRW-13]` (Q4) — the read-only name a message offers: the refused name followed by the root
    /// name of its new target (`r` and `b`: `rb`; `t` and `p.pos`: `tp`), or the first free of `rb_2`,
    /// `rb_3`, … where that name is taken.
    fn free_view_name(&self, name: Symbol, target: &str) -> String {
        let root: String = target.chars().take_while(|c| *c == '_' || c.is_alphanumeric()).collect();
        let base = format!("{name}{root}");
        let mut candidate = base.clone();
        let mut number = 1;
        while self.lookup(Symbol::intern(&candidate)).is_some() {
            number += 1;
            candidate = format!("{base}_{number}");
        }
        candidate
    }

    /// `[BRW-13]` — the message for a `const ref mut` name given another ref (`place`, written as
    /// `value`): the new name for the other ref, or the name made with plain `ref mut`.
    fn const_ref_message(&self, name: Symbol, view: ViewName, place: Span, value: Span) -> String {
        let new_target = self.written_line(place).unwrap_or_else(|| "it".to_string());
        let new_value = self.written_line(value).map_or_else(|| format!("ref mut {new_target}"), |written| {
            written.strip_prefix("const ").unwrap_or(&written).to_string()
        });
        let new_name = self.free_view_name(name, &new_target);
        match view.kind {
            ViewKind::ConstParam => format!(
                "`{name}` is `const ref mut`, so it always refers to what the caller gave; to refer to `{new_target}` use a new name, `{new_name} = {new_value}`, or make the parameter plain `ref mut`"
            ),
            _ => {
                let made = self.written_line(view.value).unwrap_or_else(|| "const ref mut".to_string());
                let old_target = self.written_line(view.target).unwrap_or_else(|| "it".to_string());
                format!(
                    "`{name}` is `{made}`, so it always refers to `{old_target}`; to refer to `{new_target}` use a new name, `{new_name} = {new_value}`, or make `{name}` with plain `ref mut`: `{name} = ref mut {old_target}`"
                )
            }
        }
    }
}

/// `[BRW-14]` — a name that keeps what it was given for its whole scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeptKind {
    /// Under `for x in e:`.
    Loop,
    /// Under `match e:`, or opened with `exists as` (`[GRM-44]`).
    Pattern,
}

/// Where the names of one header that keep their place are declared.
#[derive(Clone, Copy, Debug)]
pub(crate) struct KeptHeader {
    pub kind: KeptKind,
    /// The header's pattern (`p`, `State.Walking(path)`).
    pub pattern: Span,
    /// What it walks or matches, as written (`players`, `n.state`).
    pub source: Span,
}

impl Checker<'_> {
    /// Records a header whose names keep their place: a `for` over `source`, or a `match` (or an
    /// opening) of a place that is not handed over.
    pub(crate) fn note_kept_header(&mut self, kind: KeptKind, pattern: Span, source: Span) {
        if pattern.file == Span::DUMMY.file {
            return;
        }
        self.kept_headers.insert((pattern.file.0, pattern.start), KeptHeader { kind, pattern, source });
    }

    /// The header `local` was declared by, when it is a name that keeps its place there: a loop,
    /// pattern or opened name, never a copy of one's own (`exists as copy of v`).
    pub(crate) fn kept_header(&self, local: LocalId) -> Option<KeptHeader> {
        let decl = self.locals.get(local.0 as usize)?;
        let name = decl.name?;
        // The compiler's own names (`__xs`, `·value`, `$total`) are none of the program's.
        let written = name.as_str();
        if written.starts_with("__") || !written.starts_with(|c: char| c == '_' || c.is_alphabetic()) || self.own_copies.contains(&decl.span) {
            return None;
        }
        let span = decl.span;
        // A name written `ref mut` changes what it names (`for ref mut p in players:`, built with
        // A2-6); it is not one that only reads.
        // `mut v` alone is `E0113`'s, already reported.
        if self.written_line(span).is_some_and(|written| written.starts_with("ref mut ") || written.starts_with("mut ")) {
            return None;
        }
        // Until A2-6 builds `for ref mut x in e:`, a walk that gives names that may change (`iter_mut()`,
        // `chunks_mut(n)`: a `ref mut T`, a `MutSpan`) is how a loop changes what it walks.
        if matches!(self.types.kind(decl.ty), TyKind::Ref { mutable: true, .. } | TyKind::Span { mutable: true, .. }) {
            return None;
        }
        let (_, header) = self.kept_headers.range(..=(span.file.0, span.start)).next_back()?;
        (header.pattern.file == span.file && header.pattern.start <= span.start && span.end <= header.pattern.end).then_some(*header)
    }
}

impl Checker<'_> {
    /// `[BRW-14]` (Q5) — the noun a message gives a value of `ty`: "text", "number", "list",
    /// "span", "map", "set", a type's own name in lower case with its words apart ("game clock"), a
    /// link's or an `owns T`'s target's, and "value" for the rest.
    pub(crate) fn value_noun(&self, ty: Ty) -> String {
        if self.types.is_numeric(ty) {
            return "number".to_string();
        }
        match *self.types.kind(ty) {
            TyKind::Ref { inner, .. } => return self.value_noun(inner),
            TyKind::Str => return "text".to_string(),
            TyKind::Vec { text: true, .. } => return "text".to_string(),
            TyKind::Vec { .. } | TyKind::Array { .. } | TyKind::ArrayN { .. } => return "list".to_string(),
            TyKind::Span { .. } => return "span".to_string(),
            _ => {}
        }
        if let Some(held) = self.box_inner(ty) {
            return self.value_noun(held);
        }
        let shown = self.types.display(ty);
        let head: String = shown.chars().take_while(|c| *c == '_' || c.is_alphanumeric()).collect();
        match head.as_str() {
            "SoA" | "ArenaSoA" => return "list".to_string(),
            "Map" => return "map".to_string(),
            "Set" => return "set".to_string(),
            "StrView" | "String" => return "text".to_string(),
            "Span" | "MutSpan" => return "span".to_string(),
            "Array" => return "list".to_string(),
            _ => {}
        }
        if !matches!(self.types.kind(ty), TyKind::Struct(_) | TyKind::Class(_) | TyKind::Enum(_)) || head.is_empty() {
            return "value".to_string();
        }
        let mut words = String::new();
        for (at, c) in head.char_indices() {
            if c.is_uppercase() && at > 0 {
                words.push(' ');
            }
            words.extend(c.to_lowercase());
        }
        words
    }

    /// The code of the line `span` starts on, as written: what a message quotes as the change.
    pub(crate) fn line_code(&self, span: Span) -> Option<String> {
        let src = &self.sources.get(&span.file)?.text;
        let start = span.start as usize;
        let line_start = src.get(..start)?.rfind('\n').map_or(0, |at| at + 1);
        let line_end = src.get(start..)?.find('\n').map_or(src.len(), |at| start + at);
        let code = Self::code_of(&src[line_start..line_end]).trim();
        (!code.is_empty()).then(|| code.to_string())
    }

    /// `[BRW-15]` — a change through `local`, a name that only reads what it names: `E3023`, with the
    /// line to write. `None` where `local` is no such name. A part name's line is rewritten with
    /// `ref mut` by `ember __migrate --checker`: 0.9.10 moved the part out into the name, so a valid
    /// program never used the part again, and the name changing the part does what the line did.
    pub(crate) fn read_only_change(&self, local: LocalId, write: Span) -> Option<Diagnostic> {
        let name = self.locals.get(local.0 as usize)?.name?;
        let change = self.line_code(write).unwrap_or_else(|| "this".to_string());
        // `[FN-2a]` — `ref mut` before a name that only reads is `E3027`, not a change through it.
        let given_to = self.ref_mut_argument.filter(|(at, _)| *at == write).and_then(|(_, declared)| self.declaring_function(declared));
        if let Some(view) = self.view_names.get(&local).copied() {
            if !matches!(view.kind, ViewKind::Part | ViewKind::Ref { mutable: false, .. }) {
                return None;
            }
            let target = self.written_line(view.target)?;
            let line = self.line_of(view.line).map_or_else(String::new, |line| format!(" in place of line {line}"));
            let message = if view.changeable {
                format!(
                    "`{change}` changes `{target}`, but `{name}` only reads it; to change `{target}` through `{name}`, write `{name} = ref mut {target}`{line}"
                )
            } else {
                // What it names is only read here too: the way out is a value of its own.
                let a_noun = a_noun(&self.value_noun(self.locals[local.0 as usize].ty));
                format!(
                    "`{change}` changes `{target}`, but `{name}` only reads it, and so does this function; for {a_noun} of your own write `{name} = copy of {target}`{line}"
                )
            };
            let mut diagnostic = match &given_to {
                Some(callee) => {
                    let target = self.written_line(view.target)?;
                    let help = if view.changeable {
                        format!("to change `{target}` through `{name}`, write `{name} = ref mut {target}`{line}")
                    } else {
                        format!("what it names is only read here too: for a value of your own write `{name} = copy of {target}`{line}")
                    };
                    Diagnostic::error(codes::E3027, write, format!("`{name}` only reads `{target}`, so it cannot be given to `{callee}` as `ref mut`")).help(help)
                }
                None => Diagnostic::error(codes::E3023, write, message),
            }
                .secondary(view.line, format!("`{name}` is made here, a view that only reads"))
                .note("a name bound to a part, or written `ref`, only reads it [BRW-12], [BRW-15]");
            let at = Span::new(view.target.file, view.target.start, view.target.start);
            diagnostic = match view.kind {
                ViewKind::Part if view.changeable => diagnostic.migrate_edits(
                    "`ref mut` before a part a name changes, which 0.9.10 moved out into the name",
                    vec![(at, ember_diag::Replacement::Text("ref mut ".to_string()))],
                ),
                // 0.9.10 copied what a read-only name reads, a value with no `drop` (a type that has
                // one could not be moved out of it): the copy is the line's old meaning.
                ViewKind::Part => diagnostic.migrate_edits(
                    "`copy of` before a part a name changes, which 0.9.10 copied out of a name that only reads",
                    vec![(at, ember_diag::Replacement::Text("copy of ".to_string()))],
                ),
                _ => diagnostic,
            };
            return Some(diagnostic);
        }
        let header = self.kept_header(local)?;
        let decl = &self.locals[local.0 as usize];
        let noun = self.value_noun(decl.ty);
        let source = self.written_line(header.source).unwrap_or_else(|| "it".to_string());
        let fix = self.line_with(decl.span, &format!("ref mut {name}"))?;
        let message = format!(
            "`{change}` changes the {noun} in `{source}`, but `{name}` only reads it; to change that {noun} through `{name}`, write `{fix}`"
        );
        if let Some(callee) = given_to {
            return Some(
                Diagnostic::error(codes::E3027, write, format!("`{name}` only reads the {noun} in `{source}`, so it cannot be given to `{callee}` as `ref mut`"))
                    .help(format!("to change that {noun} through `{name}`, write `{fix}`"))
                    .secondary(decl.span, format!("`{name}` reads the {noun} here"))
                    .note("a loop, pattern or opened name views what it names and only reads it [BRW-14], [BRW-15]"),
            );
        }
        Some(
            Diagnostic::error(codes::E3023, write, message)
                .secondary(decl.span, format!("`{name}` reads the {noun} here"))
                .note("a loop, pattern or opened name views what it names and only reads it [BRW-14], [BRW-15]"),
        )
    }

    /// `[BRW-14]` — `=` on a loop, pattern or opened name, which keeps what it was given for its whole
    /// scope: `E3023`.
    pub(crate) fn kept_name_given_a_value(&self, local: LocalId, header: KeptHeader, write: Span) -> Diagnostic {
        let decl = &self.locals[local.0 as usize];
        let name = decl.name.map_or_else(|| "this name".to_string(), |name| name.to_string());
        let noun = self.value_noun(decl.ty);
        let source = self.written_line(header.source).unwrap_or_else(|| "it".to_string());
        let a_noun = a_noun(&noun);
        let (scope, whose) = match header.kind {
            KeptKind::Loop => ("round", format!("the loop over `{source}` gives")),
            KeptKind::Pattern => ("block", format!("`{source}` holds")),
        };
        Diagnostic::error(
            codes::E3023,
            write,
            format!("`{name}` is the {noun} {whose} and stays that {noun} for the whole {scope}; for {a_noun} of your own write `new_{name} = {name}` first and change `new_{name}`"),
        )
        .secondary(decl.span, format!("`{name}` is given its {noun} here"))
        .note("a loop, pattern or opened name keeps what it was given [BRW-14]")
    }
}

impl Checker<'_> {
    /// D-747 — whether a place is reached through a class object's field, which the object's own
    /// access rules govern however its handle is reached (Part VIII).
    pub(crate) fn through_class_field(&self, place: &Expr) -> bool {
        let mut current = place;
        loop {
            if let ExprKind::Field { base, .. } = &current.kind
                && matches!(self.types.kind(base.ty), TyKind::Class(_) | TyKind::ClassInterface(_))
            {
                return true;
            }
            current = match &current.kind {
                ExprKind::Field { base, .. } | ExprKind::Index { base, .. } | ExprKind::Deref(base) => base,
                _ => return false,
            };
        }
    }
}

/// `[BRW-15]` — a change through a plain parameter, whose message names each call to rewrite once
/// every call has been checked (`Checker::report_parameter_changes`).
#[derive(Clone, Debug)]
pub(crate) struct ParameterChange {
    /// Where the change is, the diagnostic's primary span.
    pub write: Span,
    /// The parameter's declaration (`s: Clock`), the key of its calls.
    pub declaration: Span,
    /// The function's name as written.
    pub function: String,
    /// The parameter's name.
    pub name: String,
    /// What the caller's value is called ("clock", "span").
    pub noun: String,
    /// The changed place as written (`s.frame`), where a field is set.
    pub field: Option<String>,
    /// An element is written (`dst[i] += src[i]`); otherwise the value is changed by a call.
    pub element: bool,
    /// The change as written (`dst[i] += src[i]`).
    pub change: String,
    /// The function's first line with `ref mut` before the parameter.
    pub first_line: String,
}

/// The marker of a message `report_parameter_changes` writes once every call is checked.
pub(crate) const PROVISIONAL: char = '\u{3}';

impl Checker<'_> {
    /// `[BRW-15]` — a call of a plain parameter's function: its argument, for the message that
    /// names each call to rewrite with `ref mut`.
    pub(crate) fn note_plain_argument(&mut self, declared: Span, arg: Span) {
        let calls = self.plain_arguments.entry(declared).or_default();
        if !calls.contains(&arg) {
            calls.push(arg);
        }
    }

    /// `[BRW-15]` — what the message of a change through a plain parameter needs, before the calls
    /// are known. `None` where the source cannot be read so.
    pub(crate) fn parameter_change(&self, name: &str, ty: Ty, declaration: Span, place: &Expr, write: Span) -> Option<ParameterChange> {
        let written = self.written_line(declaration)?;
        // `ref n: Node` reads as `n: Node` does: the word that changes it replaces `ref`.
        let written = written.strip_prefix("ref ").unwrap_or(&written);
        let first_line = self.line_with(declaration, &format!("ref mut {written}"))?;
        if !first_line.starts_with("fn ") {
            return None;
        }
        let function: String = first_line[3..].chars().take_while(|c| *c == '_' || c.is_alphanumeric()).collect();
        if function.is_empty() {
            return None;
        }
        let field = match &place.kind {
            ExprKind::Field { .. } => Some(self.written_line(place.span)?),
            _ => None,
        };
        let element = matches!(place.kind, ExprKind::Index { .. });
        Some(ParameterChange {
            write,
            declaration,
            function,
            name: name.to_string(),
            noun: self.value_noun(ty),
            field,
            element,
            change: self.line_code(write)?,
            first_line,
        })
    }

    /// The message of a change through a plain parameter, with `calls` (the arguments given to it):
    /// "`tick` sets `s.frame`, but `s` only reads the caller's clock; write `fn tick(ref mut s:
    /// Clock):` and call `tick(ref mut clock)`", each call as written with `ref mut` before the
    /// argument, and each one's line where there are several (`[DIA-25]`, verdict 134).
    fn parameter_change_message(&self, change: &ParameterChange, calls: &[Span]) -> String {
        let mut rewritten: Vec<(String, Option<usize>)> = Vec::new();
        for &arg in calls {
            if let Some(call) = self.call_with_ref_mut(arg) {
                rewritten.push((call, self.line_of(arg)));
            }
        }
        let calls = match rewritten.as_slice() {
            [] => String::new(),
            [(one, _)] => format!("`{one}`"),
            several => several
                .iter()
                .map(|(call, line)| line.map_or_else(|| format!("`{call}`"), |line| format!("`{call}` on line {line}")))
                .collect::<Vec<_>>()
                .join(" and "),
        };
        let (name, noun, first) = (&change.name, &change.noun, &change.first_line);
        if let Some(message) = self.using_change_message(change.write, name, first) {
            return message;
        }
        let (head, call) = match &change.field {
            Some(field) => (format!("`{}` sets `{field}`, but `{name}` only reads the caller's {noun}; write `{first}`", change.function), "call"),
            None if change.element => (format!("`{}` writes an element of the caller's {noun}, but `{name}` only reads it; write `{first}`", change.change), "call it"),
            None => (format!("`{}` changes the caller's {noun}, but `{name}` only reads it; write `{first}`", change.change), "call it"),
        };
        if rewritten.is_empty() {
            format!("{head}, and `ref mut` before the argument at each call")
        } else {
            format!("{head} and {call} {calls}")
        }
    }

    /// The call `arg` is given to, as written, with `ref mut` before it (`tick(ref mut clock)`).
    fn call_with_ref_mut(&self, arg: Span) -> Option<String> {
        let src = &self.sources.get(&arg.file)?.text;
        let bytes = src.as_bytes();
        let (start, end) = (arg.start as usize, arg.end as usize);
        // Back to the bracket that opens the call's arguments, and on to the one that closes them.
        let mut depth = 0i32;
        let mut open = None;
        for at in (0..start).rev() {
            match bytes[at] {
                b')' | b']' => depth += 1,
                b'(' | b'[' if depth > 0 => depth -= 1,
                b'(' => {
                    open = Some(at);
                    break;
                }
                b'\n' => return None,
                _ => {}
            }
        }
        let open = open?;
        let callee_start = src[..open].rfind(|c: char| !(c == '_' || c == '.' || c.is_alphanumeric())).map_or(0, |at| at + 1);
        depth = 0;
        let mut close = None;
        for (at, byte) in bytes.iter().enumerate().skip(end) {
            match byte {
                b'(' | b'[' => depth += 1,
                b')' if depth == 0 => {
                    close = Some(at);
                    break;
                }
                b')' | b']' => depth -= 1,
                b'\n' => return None,
                _ => {}
            }
        }
        let close = close?;
        let written = src.get(start..end)?;
        Some(format!("{}ref mut {written}{}", &src[callee_start..start], &src[end..=close]))
    }

    /// `[BRW-15]` — once every call is checked, each change through a plain parameter names the
    /// calls to rewrite. A diagnostic a quiet check took back is not found, and nothing is amended.
    pub(crate) fn report_parameter_changes(&mut self) {
        let changes = std::mem::take(&mut self.parameter_changes);
        for change in changes {
            let calls = self.plain_arguments.get(&change.declaration).cloned().unwrap_or_default();
            let message = self.parameter_change_message(&change, &calls);
            let found = self
                .sink
                .diagnostics()
                .iter()
                .position(|d| d.code == Some(codes::E3023) && d.primary.span == change.write && d.message.starts_with(PROVISIONAL));
            if let Some(at) = found {
                self.sink.amend(at, |diagnostic| diagnostic.message = message);
            }
        }
        // The lead's review: a message whose change was not recorded keeps its words; the marker
        // never reaches the reader.
        let left: Vec<usize> =
            self.sink.diagnostics().iter().enumerate().filter(|(_, d)| d.message.starts_with(PROVISIONAL)).map(|(at, _)| at).collect();
        for at in left {
            self.sink.amend(at, |diagnostic| diagnostic.message = diagnostic.message.trim_start_matches(PROVISIONAL).to_string());
        }
    }
}

/// `[OWN-7]`, `[BRW-12]` — a `Copy` value with parts copied with `=` from a name (`b = a`, `p =
/// pts[0]`), which is not then changed through the new name while the source is still used.
#[derive(Default)]
pub(crate) struct CopiedValues {
    /// The copy, by its local: the source's local, the line that copied it, and the value as written.
    pub copies: HashMap<LocalId, (LocalId, Span, Span)>,
    /// The sources, whose mentions are recorded.
    pub sources: HashSet<LocalId>,
    /// Each mention of a source.
    pub mentions: Vec<(LocalId, Span)>,
    /// Each change through a copy: the copy, the statement, and the loops around it.
    pub changes: Vec<(LocalId, Span, Vec<Span>)>,
}

impl Checker<'_> {
    /// Records `local`, declared by `line` from `value` (as written) whose checked value is `init`,
    /// when it copies a `Copy` value with parts out of a name.
    pub(crate) fn note_copied_value(&mut self, local: LocalId, init: &Expr, value: &ast::Expr, line: Span) {
        if value.id == ast::NodeId::DUMMY || matches!(value.kind, ast::ExprKind::CopyOf(_) | ast::ExprKind::RefOf { .. }) || !is_place(&init.kind) {
            return;
        }
        let with_parts = matches!(self.types.kind(init.ty), TyKind::Struct(_) | TyKind::Tuple(_) | TyKind::ArrayN { .. });
        if !with_parts || !self.copied_by_equals(init.ty) {
            return;
        }
        let Some(source) = root_local(&init.kind) else { return };
        if self.locals[source.0 as usize].name.is_none() {
            return;
        }
        self.copied.copies.insert(local, (source, line, value.span));
        self.copied.sources.insert(source);
    }

    /// A mention of `local`, where it is the source of a copy.
    pub(crate) fn note_copy_source_mention(&mut self, local: LocalId, span: Span) {
        if self.copied.sources.contains(&local) {
            self.copied.mentions.push((local, span));
        }
    }

    /// A change of `place` by the statement `stmt`, where `place` is reached through a copy.
    pub(crate) fn note_copy_change(&mut self, place: &Expr, stmt: Span) {
        if let Some(local) = root_local(&place.kind)
            && self.copied.copies.contains_key(&local)
        {
            let loops = self.loop_spans.clone();
            self.copied.changes.push((local, stmt, loops));
        }
    }

    /// `E3023` for each change through a copy while its source is used after it: later in the body,
    /// or earlier in a loop around the change that the copy was made outside of. The line that made
    /// the copy is rewritten `copy of` by `ember __migrate --checker` (what 0.9.10's `=` made).
    pub(crate) fn report_copied_value_changes(&mut self) {
        let copied = std::mem::take(&mut self.copied);
        let mut reported: HashSet<LocalId> = HashSet::new();
        for (copy, stmt, loops) in &copied.changes {
            let Some(&(source, line, value)) = copied.copies.get(copy) else { continue };
            let contains = |outer: Span, inner: Span| outer.file == inner.file && outer.start <= inner.start && inner.end <= outer.end;
            let later = copied.mentions.iter().find(|(local, at)| {
                *local == source
                    && !contains(line, *at)
                    && !contains(*stmt, *at)
                    && (at.start >= stmt.end || loops.iter().any(|around| contains(*around, *at) && !contains(*around, line)))
            });
            let Some(&(_, used)) = later else { continue };
            let (Some(name), Some(source_name)) = (self.locals[copy.0 as usize].name, self.locals[source.0 as usize].name) else { continue };
            let change = self.line_code(*stmt).unwrap_or_else(|| "this".to_string());
            let written = self.written_line(value).unwrap_or_else(|| source_name.to_string());
            let made = self.line_of(line).map_or_else(String::new, |line| format!(" (line {line})"));
            let use_line = self.line_of(used).map_or_else(String::new, |line| format!(" on line {line}"));
            let message = format!(
                "`{change}` changes `{name}`, a copy of `{written}`{made}, while `{source_name}` is still used{use_line}; for a copy of your own write `{name} = copy of {written}`, or to change `{written}` write `{name} = ref mut {written}`"
            );
            let at = Span::new(value.file, value.start, value.start);
            let mut diagnostic = Diagnostic::error(codes::E3023, *stmt, message)
                .secondary(used, format!("`{source_name}` is used here"))
                .note("a `Copy` value with parts copied with `=` is not changed through the new name while the source is used, so that no copy is changed by mistake [OWN-7]");
            if reported.insert(*copy) {
                diagnostic = diagnostic.migrate_edits("`copy of` where 0.9.10's `=` copied a value that is then changed", vec![(at, ember_diag::Replacement::Text("copy of ".to_string()))]);
            }
            self.sink.emit(diagnostic);
        }
    }
}

impl Checker<'_> {
    /// Whether a change of `place` goes through a view that may change what it views (a `ref mut`, a
    /// pointer, a `MutSpan`'s element): a change of what the view names, not of the name's own
    /// storage. An element proxy a walk gives (`for q in ns.iter_mut():`) is such a view.
    pub(crate) fn through_a_changing_view(&self, place: &Expr) -> bool {
        let mut current = place;
        loop {
            match &current.kind {
                ExprKind::Deref(inner) if matches!(self.types.kind(inner.ty), TyKind::Ref { mutable: true, .. } | TyKind::Ptr { .. }) => return true,
                ExprKind::Index { base, .. } if matches!(self.types.kind(base.ty), TyKind::Span { mutable: true, .. }) => return true,
                ExprKind::Field { base, .. } | ExprKind::Index { base, .. } | ExprKind::Deref(base) => current = base,
                _ => return false,
            }
        }
    }
}

/// The noun with its article: "a number", "an enemy".
pub(crate) fn a_noun(noun: &str) -> String {
    let article = if noun.starts_with(['a', 'e', 'i', 'o', 'u']) { "an" } else { "a" };
    format!("{article} {noun}")
}
