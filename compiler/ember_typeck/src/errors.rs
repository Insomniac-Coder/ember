//! Part XIII's error handling and the written `Display` it rests on (ADR-186).
//!
//! `[TYP-36]`'s table gives `Display` to the numbers, text and collections;
//! a struct, an enum or a class has it by implementing `std.core.Display`,
//! whose `fmt` writes through a `std.fmt.Formatter`. Every place that turns
//! a value into text (an f-string's hole, `print`'s argument, `main`'s
//! error) writes such a value with `std.fmt.write_display`, which calls its
//! `fmt` over the text being built.

use super::*;

impl<'a> Checker<'a> {
    /// ADR-186 — whether a struct, an enum or a class implements `Display`
    /// by a written `fmt`.
    pub(crate) fn written_display(&self, ty: Ty) -> bool {
        let display = Symbol::intern("std.core.Display");
        self.implemented.iter().any(|&(t, interface, _)| t == ty && interface == display)
    }

    /// ADR-186, `[STR-5]` — whether a type writes its own `Debug`, which
    /// replaces the field-wise one.
    pub(crate) fn written_debug(&self, ty: Ty) -> bool {
        let debug = Symbol::intern("std.core.Debug");
        self.implemented.iter().any(|&(t, interface, _)| t == ty && interface == debug)
    }

    /// ADR-186 — `std.fmt.write_debug`'s instance for each concrete type
    /// with a written `Debug` that has none yet. The backend calls it
    /// wherever the type is formatted (`TypeTable::debug_writer`); it is
    /// reached through no call of the program's own. Whether any was asked.
    pub(crate) fn request_debug_writers(&mut self) -> bool {
        let debug = Symbol::intern("std.core.Debug");
        let Some(def) = self.fn_ids.get(&Symbol::intern("std.fmt.write_debug")).copied() else { return false };
        let wanted: Vec<(Ty, Span)> = self
            .implemented
            .iter()
            .filter(|&&(ty, interface, _)| interface == debug && !self.types.is_generic(ty))
            .map(|&(ty, _, span)| (ty, span))
            .filter(|(ty, _)| !self.debug_writer_defs.iter().any(|(known, _)| known == ty))
            .collect();
        for &(ty, span) in &wanted {
            let writer = self.instantiate(def, &[ty], Vec::new(), Symbol::intern("write_debug"), span);
            self.debug_writer_defs.push((ty, writer));
        }
        // `[TYP-39]` (ADR-191) — and each `dyn` value formatted inside another, through its table.
        let holders: Vec<Ty> = self
            .dyn_debug_holders
            .iter()
            .copied()
            .filter(|ty| !self.debug_writer_defs.iter().any(|(known, _)| known == ty))
            .collect();
        for &ty in &holders {
            let writer = self.instantiate(def, &[ty], Vec::new(), Symbol::intern("write_debug"), Span::DUMMY);
            self.debug_writer_defs.push((ty, writer));
        }
        !wanted.is_empty() || !holders.is_empty()
    }

    /// ADR-186 — tell the backend each written `Debug`'s function, by the
    /// symbol its body was given.
    pub(crate) fn publish_debug_writers(&mut self, functions: &[Function]) {
        for &(ty, def) in &self.debug_writer_defs {
            if let Some(function) = functions.iter().find(|function| function.def == def) {
                self.types.set_debug_writer(ty, function.symbol.clone());
            }
        }
    }

    /// `[STR-5]` (ADR-186) — the `fmt_debug` of a `dyn` table (`dyn Error`,
    /// whose `Error` needs `Debug`) for a type that writes none: its
    /// field-wise text, through `std.fmt.implicit_fmt_debug`.
    pub(crate) fn implicit_fmt_debug(&mut self, concrete: Ty, name: Symbol, owner: Option<Symbol>) -> Option<hir::InterfaceAdapterSlot> {
        if !name.is("fmt_debug") || owner.is_none_or(|owner| owner.as_str() != "std.core.Debug") || !self.is_formattable(concrete) {
            return None;
        }
        let def = self.fn_ids.get(&Symbol::intern("std.fmt.implicit_fmt_debug")).copied()?;
        let span = self.signatures[def.0 as usize].params.first().map_or(Span::DUMMY, |param| param.3);
        let implementation = self.instantiate(def, &[concrete], Vec::new(), name, span);
        Some(hir::InterfaceAdapterSlot { implementation, receiver: Mode::Borrow })
    }

    /// ADR-186 — the f-string piece writing `value`'s plain text (`{x}`,
    /// `print(x)`): its written `Display`, reached through any `Box` or
    /// `Cell` that holds it (ODR-094), or a `dyn` value's through its table,
    /// or else the text the table of `[TYP-36]` gives it.
    pub(crate) fn plain_text_piece(&mut self, value: Expr) -> hir::FStringPart {
        let value = self.dyn_reference(value);
        if self.dyn_reaches(value.ty, "std.core.Display") {
            let ty = value.ty;
            return self.writer_piece("std.fmt.write_display", value, ty);
        }
        let mut reached = value.ty;
        while let Some(inner) = self.types.printed_as(reached) {
            reached = inner;
        }
        if !self.written_display(reached) {
            return hir::FStringPart::Value(self.read_through(value), None);
        }
        let mut value = value;
        while value.ty != reached {
            if self.box_inner(value.ty).is_some() {
                value = self.read_box_through(value);
                continue;
            }
            // A `Cell`'s value is written as a copy, as `get` reads it: its
            // `fmt` runs while the cell may be `set`.
            let Some(inner) = self.cell_inner(value.ty) else { break };
            let span = value.span;
            if !self.types.is_copy(inner) {
                self.cell_lends_its_value(value.ty, "Display", span);
                return hir::FStringPart::Value(Expr { ty: self.common.error, kind: ExprKind::Error, span }, None);
            }
            let field = Expr { ty: inner, kind: ExprKind::Field { base: Box::new(value), index: 0 }, span };
            let copy = self.declare(None, inner, span);
            value = Expr {
                ty: inner,
                kind: ExprKind::Block {
                    block: Block { stmts: vec![Stmt::Let { local: copy, init: Some(field) }], span },
                    value: Box::new(Expr { ty: inner, kind: ExprKind::Local(copy), span }),
                },
                span,
            };
        }
        self.writer_piece("std.fmt.write_display", value, reached)
    }

    /// `[TYP-39]`, `[CELL-2]` (ODR-109) — a `Cell[T]` shows the `T` it holds by `T`'s written
    /// `Display` or `Debug` only through a copy, as `get` reads it: that `fmt` is the program's code
    /// and may `set` the cell, and a `Cell` never lends its value. A `T` that is not `Copy` cannot be
    /// shown so (`E2040`); the text the compiler writes for it lends nothing and is always allowed.
    pub(crate) fn cell_lends_its_value(&mut self, cell: Ty, interface: &str, span: Span) {
        let shown = self.types.display(cell);
        let inner = self.cell_inner(cell).map(|inner| self.types.display(inner)).unwrap_or_default();
        self.sink.emit(
            Diagnostic::error(codes::E2040, span, format!("`{shown}` cannot show its value by `{inner}`'s written `{interface}`, which would be lent the cell's value"))
                .help(format!("hold the value in a `RefCell` and show `c.borrow()`, or make `{inner}` `Copy`"))
                .note("a `Cell` never lends its value [CELL-2]; it shows a copy of it, which needs `Copy` (ODR-109)"),
        );
    }

    /// ADR-186 — `{x!r}` of a `dyn` value whose interfaces include `Debug`:
    /// its `fmt_debug` through the table; `Err(value)` for any other value,
    /// which the table of `[TYP-36]` (or a written `Debug`) formats.
    pub(crate) fn dyn_debug_piece(&mut self, value: Expr) -> Result<hir::FStringPart, Expr> {
        let value = self.dyn_reference(value);
        if !self.dyn_reaches(value.ty, "std.core.Debug") {
            return Err(self.read_through(value));
        }
        let ty = value.ty;
        Ok(self.writer_piece("std.fmt.write_debug", value, ty))
    }

    /// `writer`'s instance for `ty` (`std.fmt.write_display` or
    /// `write_debug`), writing `value` into the text being built.
    fn writer_piece(&mut self, writer: &str, value: Expr, ty: Ty) -> hir::FStringPart {
        let Some(def) = self.fn_ids.get(&Symbol::intern(writer)).copied() else {
            return hir::FStringPart::Value(value, None);
        };
        let name = Symbol::intern(writer.rsplit('.').next().unwrap_or(writer));
        let writer = self.instantiate(def, &[ty], Vec::new(), name, value.span);
        let value = self.pass_argument(value, ty, Mode::Borrow);
        hir::FStringPart::Write { writer, value }
    }

    /// A `ref dyn I` that a formatting site read through, as the reference:
    /// the `dyn` value is written through it.
    fn dyn_reference(&mut self, value: Expr) -> Expr {
        match value.kind {
            ExprKind::Deref(inner) if matches!(self.types.kind(inner.ty), TyKind::Ref { inner: target, .. } if matches!(self.types.kind(*target), TyKind::Dyn { .. })) => *inner,
            kind => Expr { kind, ..value },
        }
    }

    /// ADR-186 — whether `ty` is a `dyn` value (`ref dyn I`, `Box[dyn I]`)
    /// whose interfaces, or their parents, include `interface`.
    pub(crate) fn dyn_reaches(&self, ty: Ty, interface: &str) -> bool {
        let interfaces = match self.types.kind(ty) {
            TyKind::Dyn { interfaces } => interfaces.clone(),
            TyKind::Ref { inner, .. } => match self.types.kind(*inner) {
                TyKind::Dyn { interfaces } => interfaces.clone(),
                _ => return false,
            },
            _ => match self.box_inner(ty).map(|inner| self.types.kind(inner).clone()) {
                Some(TyKind::Dyn { interfaces }) => interfaces,
                _ => return false,
            },
        };
        let mut pending = interfaces;
        let mut seen = HashSet::new();
        while let Some(name) = pending.pop() {
            if name.as_str() == interface {
                return true;
            }
            if seen.insert(name)
                && let Some(def) = self.interfaces.get(&name)
            {
                pending.extend(def.supertraits.iter().copied());
            }
        }
        false
    }

    /// ADR-186 — `value` as the `String` of its written `Display` text, for a
    /// place that prints one value (`print`'s argument, `main`'s error);
    /// `Err(value)` when its type has no written `Display`.
    pub(crate) fn written_display_text(&mut self, value: Expr) -> Result<Expr, Expr> {
        let span = value.span;
        match self.plain_text_piece(value) {
            hir::FStringPart::Value(value, _) => Err(value),
            piece => {
                let string_ty = self.common.string;
                let buffer_ref = self.types.intern(TyKind::Ref { mutable: true, inner: string_ty });
                Ok(Expr { ty: string_ty, kind: ExprKind::FString { parts: vec![piece], buffer_ref }, span })
            }
        }
    }

    /// `[ERR-8]` (ADR-186) — `std.core`'s two compiler-known helpers of
    /// `AnyError`: `type_identity[T]()`, the 128-bit FNV-1a hash of `T`'s
    /// canonical name (the same for one type in every module and every
    /// compilation; as Rust's `TypeId`, two types share one only by a
    /// collision of a 128-bit hash), and
    /// `dyn_payload[E](b)`, the `Box[dyn Error]` `b`, borrowed, read as the
    /// `ref E` it was made from (`unsafe`: its value must be an `E`).
    pub(crate) fn synth_error_builtin(&mut self, name: Symbol, args: &[ast::Arg], explicit: &[Ty], span: Span) -> Option<Expr> {
        let resolved = self.resolve_name(name);
        if resolved.as_str() == "std.fmt.append_debug" {
            return Some(self.synth_append_debug(args, span));
        }
        if resolved.as_str() == "std.fmt.format_to" {
            return Some(self.synth_format_to(args, span));
        }
        let helper = match resolved.as_str() {
            "std.core.type_identity" => "type_identity",
            "std.core.dyn_payload" => "dyn_payload",
            _ => return None,
        };
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let wanted = usize::from(helper == "dyn_payload");
        let [ty] = explicit else {
            self.error(codes::E2020, span, format!("`{helper}` takes one type argument, found {}", explicit.len()));
            return Some(error);
        };
        if args.len() != wanted {
            self.error(codes::E2020, span, format!("`{helper}` takes {wanted} argument(s), found {}", args.len()));
            return Some(error);
        }
        if helper == "type_identity" {
            // A parameter of an opaque body: never emitted.
            let text = self.types.canonical_name(*ty).unwrap_or_default();
            let hash = text.bytes().fold(0x6c62_272e_07bb_0142_62b8_2175_6295_c58d_u128, |hash, byte| {
                (hash ^ u128::from(byte)).wrapping_mul(0x0000_0000_0100_0000_0000_0000_0000_013b)
            });
            return Some(Expr { ty: self.common.u128, kind: ExprKind::Int(hash), span });
        }
        if !self.unsafe_context() {
            self.refuse_unsafe(span, crate::UnsafeStep::Call { callee: helper.to_string(), declared: true },
                Diagnostic::error(codes::E3100, span, "calling an `unsafe fn` requires an `unsafe` block"));
        }
        let boxed = self.synth_committed(&args[0].value);
        if !self.box_inner(boxed.ty).is_some_and(|inner| matches!(self.types.kind(inner), TyKind::Dyn { .. })) {
            let shown = self.types.display(boxed.ty);
            self.error(codes::E2020, args[0].value.span, format!("`dyn_payload` reads a `Box[dyn …]`, not `{shown}`"));
            return Some(error);
        }
        let borrowed = self.types.intern(TyKind::Ref { mutable: false, inner: boxed.ty });
        let boxed = Expr { ty: borrowed, kind: ExprKind::Ref { place: Box::new(boxed), mutable: false }, span };
        let to = self.types.intern(TyKind::Ref { mutable: false, inner: *ty });
        Some(Expr { ty: to, kind: ExprKind::Cast { expr: Box::new(boxed), to }, span })
    }

    /// `[STD-18]` — `std.fmt.format_to(buf, a, b, …, sep=" ", end="")`, compiler-known: the
    /// arguments' text, as `print` writes it (`[STD-9]`), written into `buf`; `Ok` of a view of what
    /// was written, or `Err(FmtError.Failed)` when it does not fit. The text is a `String` over
    /// `buf` that never grows (`FmtSinkOpen`): each piece is appended to it as an f-string's is (a
    /// written `Display` as its text, as `print` writes one), and `FmtSinkClose` forgets it and gives
    /// its length. The buffer is evaluated first, then every argument where it is written, before
    /// any is appended (`[EXP-1]`).
    fn synth_format_to(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let Some(first) = args.first().filter(|arg| arg.name.is_none()) else {
            self.error(codes::E2020, span, "`format_to` takes the buffer to write into first");
            return error;
        };
        let (u8_ty, str_ty, usize_ty, bool_ty, void, string_ty) =
            (self.common.u8, self.common.str_, self.common.usize, self.common.bool_, self.common.void, self.common.string);
        let buf_ty = self.types.intern(TyKind::Span { elem: u8_ty, mutable: true });
        let view_ty = self.types.intern(TyKind::Span { elem: u8_ty, mutable: false });
        let buf = self.check_expr(&first.value, buf_ty);
        // A span variable is lent for the call, as a call's `MutSpan` argument is, not moved.
        let buf = if is_place(&buf.kind) && buf.ty == buf_ty {
            let at = buf.span;
            Expr { ty: buf_ty, kind: ExprKind::Builtin { which: Builtin::SpanReborrow, args: vec![buf] }, span: at }
        } else {
            buf
        };
        let Some(error_ty) = self.named_types.get(&Symbol::intern("std.fmt.FmtError")).copied() else {
            self.error(codes::E1010, span, "cannot find `std.fmt.FmtError`");
            return error;
        };
        let TyKind::Enum(error_id) = *self.types.kind(error_ty) else { return error };
        let result_ty = self.result_of(str_ty, error_ty);
        let TyKind::Enum(result_id) = *self.types.kind(result_ty) else { return error };
        let variant = |types: &ember_types::TypeTable, id, name: &str| types.enum_def(id).variants.iter().position(|variant| variant.name.is(name));
        let (Some(ok), Some(err), Some(failed)) =
            (variant(&self.types, result_id, "Ok"), variant(&self.types, result_id, "Err"), variant(&self.types, error_id, "Failed"))
        else {
            return error;
        };
        let local = |id, ty| Expr { ty, kind: ExprKind::Local(id), span };
        let b = self.declare(None, buf_ty, span);
        let mut stmts = vec![Stmt::Let { local: b, init: Some(buf) }];
        // The pieces exactly as `print` takes them (`sep`, `end` and all).
        let printed = self.synth_print_pieces(Builtin::FmtSinkOpen, &args[1..], span);
        let pieces = match printed.kind {
            ExprKind::Builtin { args, .. } => args,
            ExprKind::Block { block, value } => {
                stmts.extend(block.stmts);
                match value.kind {
                    ExprKind::Builtin { args, .. } => args,
                    _ => return error,
                }
            }
            _ => return error,
        };
        let mut reads = Vec::new();
        for piece in pieces {
            if matches!(&piece.kind, ExprKind::Str(text) if text.is_empty()) {
                continue;
            }
            let read = self.declare(None, piece.ty, piece.span);
            reads.push((read, piece.ty, piece.span));
            stmts.push(Stmt::Let { local: read, init: Some(piece) });
        }
        let sink = self.declare(None, string_ty, span);
        let reborrow = Expr { ty: buf_ty, kind: ExprKind::Builtin { which: Builtin::SpanReborrow, args: vec![local(b, buf_ty)] }, span };
        let open = Expr { ty: string_ty, kind: ExprKind::Builtin { which: Builtin::FmtSinkOpen, args: vec![reborrow] }, span };
        stmts.push(Stmt::Let { local: sink, init: Some(open) });
        let sink_ref = self.types.intern(TyKind::Ref { mutable: true, inner: string_ty });
        for (read, ty, at) in reads {
            // A place `print` takes borrowed is appended as what it reaches.
            let value = match *self.types.kind(ty) {
                TyKind::Ref { inner, .. } => Expr { ty: inner, kind: ExprKind::Deref(Box::new(local(read, ty))), span: at },
                _ => local(read, ty),
            };
            let target = Expr { ty: sink_ref, kind: ExprKind::Ref { place: Box::new(local(sink, string_ty)), mutable: true }, span: at };
            stmts.push(Stmt::Expr(Expr { ty: void, kind: ExprKind::Builtin { which: Builtin::Format, args: vec![target, value] }, span: at }));
        }
        let written = self.declare(None, usize_ty, span);
        let sink_read = self.types.intern(TyKind::Ref { mutable: false, inner: string_ty });
        let lent = Expr { ty: sink_read, kind: ExprKind::Ref { place: Box::new(local(sink, string_ty)), mutable: false }, span };
        let close = Expr { ty: usize_ty, kind: ExprKind::Builtin { which: Builtin::FmtSinkClose, args: vec![lent] }, span };
        stmts.push(Stmt::Let { local: written, init: Some(close) });
        // Its bytes are the caller's: the text is forgotten, never dropped.
        let forget = Builtin::MemForget { elem: string_ty };
        stmts.push(Stmt::Expr(Expr { ty: void, kind: ExprKind::Builtin { which: forget, args: vec![local(sink, string_ty)] }, span }));
        let fit = Expr {
            ty: bool_ty,
            kind: ExprKind::Binary {
                op: BinOp::Ne,
                lhs: Box::new(local(written, usize_ty)),
                rhs: Box::new(Expr { ty: usize_ty, kind: ExprKind::Int(u128::from(u64::MAX)), span }),
            },
            span,
        };
        let shared = Expr { ty: view_ty, kind: ExprKind::Builtin { which: Builtin::SpanSharedReborrow, args: vec![local(b, buf_ty)] }, span };
        let start = Expr { ty: usize_ty, kind: ExprKind::Int(0), span };
        let sliced = Expr {
            ty: view_ty,
            kind: ExprKind::Builtin { which: Builtin::Slice { text: false, nonempty: false }, args: vec![shared, start, local(written, usize_ty)] },
            span,
        };
        let text = Expr { ty: str_ty, kind: ExprKind::Builtin { which: Builtin::SpanToStr, args: vec![sliced] }, span };
        let done = Expr { ty: result_ty, kind: ExprKind::EnumLit { enum_id: result_id, variant: ok, fields: vec![text] }, span };
        let reason = Expr { ty: error_ty, kind: ExprKind::EnumLit { enum_id: error_id, variant: failed, fields: vec![] }, span };
        let refused = Expr { ty: result_ty, kind: ExprKind::EnumLit { enum_id: result_id, variant: err, fields: vec![reason] }, span };
        let arm = |kind, body| hir::MatchArm { pattern: hir::Pattern { ty: bool_ty, kind, span }, guard: None, body: hir::MatchArmBody::Expr(body), span };
        let value = Expr {
            ty: result_ty,
            kind: ExprKind::Match { scrutinee: Box::new(fit), arms: vec![arm(hir::PatternKind::Int(1), done), arm(hir::PatternKind::Wild, refused)] },
            span,
        };
        Expr { ty: result_ty, kind: ExprKind::Block { block: Block { stmts, span }, value: Box::new(value) }, span }
    }

    /// `[STR-5]` (ADR-191) — `std.fmt.append_debug(value, out)`, compiler-known: `value`'s `Debug`
    /// text appended to `out` (a `ref mut String`), as `{value!r}` appends it to the text an f-string
    /// builds, so a `dyn` table's field-wise `fmt_debug` writes into the `Formatter`'s text with no
    /// text of its own. `value` is read, not consumed.
    fn synth_append_debug(&mut self, args: &[ast::Arg], span: Span) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let [value, out] = args else {
            self.error(codes::E2020, span, format!("`append_debug` takes 2 arguments, found {}", args.len()));
            return error;
        };
        let value = self.synth(&value.value);
        let out = self.synth(&out.value);
        let string_ty = self.common.string;
        let wanted = self.types.intern(TyKind::Ref { mutable: true, inner: string_ty });
        if out.ty != wanted {
            let shown = self.types.display(out.ty);
            self.error(codes::E2020, out.span, format!("`append_debug` appends to a `ref mut String`, not `{shown}`"));
            return error;
        }
        if value.ty == self.common.error || !self.check_formattable(value.ty, value.span, "formatting") {
            return error;
        }
        // Lent again for the append, so the `ref mut` stays where it is (`f.out`).
        let out_span = out.span;
        let target = Expr { ty: string_ty, kind: ExprKind::Deref(Box::new(out)), span: out_span };
        let out = Expr { ty: wanted, kind: ExprKind::Ref { place: Box::new(target), mutable: true }, span: out_span };
        let spec = hir::FormatSpec { kind: Some('?'), ..hir::FormatSpec::PLAIN };
        Expr { ty: self.common.void, kind: ExprKind::Builtin { which: Builtin::FormatWith(spec), args: vec![out, value] }, span }
    }

    /// `[ERR-10]` — `r.context(msg)` and `r.with_context(f)` on a
    /// `Result[T, E]` whose `E` converts into `AnyError`: the error, if any,
    /// converted (`AnyError.from(e)`, `[ERR-8]`; one already an `AnyError` is
    /// kept) and given the message outermost, by `std.core`'s
    /// `result_context`/`result_with_context`.
    pub(crate) fn synth_result_context(
        &mut self,
        receiver: Expr,
        name: ast::Ident,
        args: &[ast::Arg],
        expected: Option<Ty>,
        span: Span,
    ) -> Expr {
        let error = Expr { ty: self.common.error, kind: ExprKind::Error, span };
        let Some(&any) = self.named_types.get(&Symbol::intern("std.core.AnyError")) else {
            self.error(codes::E1010, name.span, format!("`{}` is not available: the prelude is missing", name.name));
            return error;
        };
        let TyKind::Enum(id) = *self.types.kind(receiver.ty) else { return error };
        let (ok_ty, err_ty) = {
            let variants = &self.types.enum_def(id).variants;
            (variants[0].fields[0].ty, variants[1].fields[0].ty)
        };
        let receiver = if err_ty == any {
            receiver
        } else {
            let target = self.result_of(ok_ty, any);
            let Some((failure, converted)) = self.convert_failure(err_ty, any, receiver.ty, target, span) else {
                return error;
            };
            let TyKind::Enum(target_id) = *self.types.kind(target) else { return error };
            let success = self.declare(None, ok_ty, span);
            let bind = |local, ty| hir::Pattern { ty, kind: hir::PatternKind::Bind { local, sub: None, by_ref: None }, span };
            let arm = |variant: usize, field: hir::Pattern, value: Expr| hir::MatchArm {
                pattern: hir::Pattern {
                    ty: receiver.ty,
                    kind: hir::PatternKind::Variant { enum_id: id, variant, fields: vec![field] },
                    span,
                },
                guard: None,
                body: hir::MatchArmBody::Expr(Expr {
                    ty: target,
                    kind: ExprKind::EnumLit { enum_id: target_id, variant, fields: vec![value] },
                    span,
                }),
                span,
            };
            let arms = vec![
                arm(0, bind(success, ok_ty), Expr { ty: ok_ty, kind: ExprKind::Local(success), span }),
                arm(1, bind(failure, err_ty), converted),
            ];
            Expr { ty: target, kind: ExprKind::Match { scrutinee: Box::new(receiver), arms }, span }
        };
        self.synth_wrapper_callback(receiver, "result", name, args, expected, span)
    }

    /// `[ERR-12]` (ADR-186) — how `main` reports its `Err`, held in `bound`:
    /// an `E: Error` or an `AnyError` by `std.core`'s `report_error` or
    /// `report_any_error` (`error: <Display>`, then a `caused by:` line for
    /// each cause); one with a written `Display` that is not an `Error`, its
    /// text alone (ODR-050). `None` for the table's types, which `main_entry`
    /// prints itself.
    pub(crate) fn main_error_report(&mut self, error: Ty, bound: LocalId, span: Span) -> Option<Expr> {
        let local = Expr { ty: error, kind: ExprKind::Local(bound), span };
        let any = self.named_types.get(&Symbol::intern("std.core.AnyError")).copied();
        let helper = if Some(error) == any {
            Some("std.core.report_any_error")
        } else if self.implements_or_blanket(error, Symbol::intern("std.core.Error")) {
            Some("std.core.report_error")
        } else {
            None
        };
        if let Some(helper) = helper {
            let def = self.fn_ids.get(&Symbol::intern(helper)).copied()?;
            let callee = match self.signatures[def.0 as usize].generics.is_empty() {
                true => def,
                false => self.instantiate(def, &[error], Vec::new(), Symbol::intern("report_error"), span),
            };
            let arg = self.pass_argument(local, error, Mode::Borrow);
            return Some(Expr {
                ty: self.common.void,
                kind: ExprKind::Call { callee, arg_eval_order: None, default_arg_locals: None, args: vec![arg], latebound: false },
                span,
            });
        }
        let text = self.written_display_text(local).ok()?;
        let str_ty = self.common.str_;
        let text = self.coerce(text, str_ty);
        let piece = |piece: &str| Expr { ty: str_ty, kind: ExprKind::Str(piece.to_string()), span };
        Some(Expr {
            ty: self.common.void,
            kind: ExprKind::Builtin { which: Builtin::EPrint, args: vec![piece("error: "), text, piece("\n")] },
            span,
        })
    }

    /// `[ERR-2]` — `e?` whose failure is a `from` and the function's a `to`:
    /// the local the failure is bound to, and `to.from(failure)`, chosen as
    /// a written call of it is (an instance of `From` chosen by the
    /// argument's type, `[TYP-24]`). `None`, reported, when `to` does not
    /// implement `From[from]`.
    pub(crate) fn convert_failure(&mut self, from: Ty, to: Ty, value_ty: Ty, ret_ty: Ty, span: Span) -> Option<(LocalId, Expr)> {
        let interface = Symbol::intern("std.core.From");
        let instance = match self.interfaces.get(&interface).cloned() {
            Some(definition) => self.instantiate_interface(interface, &definition, &[from], span),
            None => None,
        };
        let (shown_from, shown_to) = (self.types.display(from), self.types.display(to));
        if !instance.is_some_and(|instance| self.implements_or_blanket(to, instance)) {
            let (carried, returning) = (self.types.display(value_ty), self.types.display(ret_ty));
            self.sink.emit(
                Diagnostic::error(
                    codes::E2040,
                    span,
                    format!("`?` cannot carry the failure of a `{carried}` out of a function returning `{returning}`"),
                )
                .primary_label(format!("`{shown_to}` does not implement `From[{shown_from}]`"))
                .help(format!("implement `From[{shown_from}]` for `{shown_to}`, or convert the failure first with `.map_err(…)`"))
                .note("`?` gives a failure to the function's error type's `from` [ERR-2]"),
            );
            return None;
        }
        self.scopes.push(HashMap::new());
        let hidden = Symbol::intern("$failure");
        let local = self.declare(Some(hidden), from, span);
        let read = ast::Expr {
            id: ast::NodeId::DUMMY,
            kind: ast::ExprKind::Path { segments: vec![ast::Ident { name: hidden, span }] },
            span,
        };
        let arg = ast::Arg { name: None, value: read, span };
        let call = self.synth_associated_call(to, ast::Ident { name: Symbol::intern("from"), span }, &[], &[arg], span);
        self.scopes.pop();
        Some((local, call))
    }

    /// `[ERR-2]` — `E2181`: `?` on an `Option` in a function returning a
    /// `Result`, or on a `Result` in one returning an `Option`, with the
    /// conversion that makes it fit.
    pub(crate) fn report_try_across(&mut self, value_ty: Ty, ret_ty: Ty, inner: Span, span: Span) {
        let (shown, returning) = (self.types.display(value_ty), self.types.display(ret_ty));
        let at = inner.shrink_to_end();
        let diagnostic = if self.is_option(value_ty) {
            Diagnostic::error(codes::E2181, span, format!("`?` on an `{shown}` in a function returning `{returning}`"))
                .suggest("give the `None` an error first: `.ok_or(err)?`", at, ".ok_or(err)")
        } else {
            Diagnostic::error(codes::E2181, span, format!("`?` on a `{shown}` in a function returning `{returning}`"))
                .suggest("let the error go first: `.ok()?`", at, ".ok()")
        };
        self.sink.emit(diagnostic.note(
            "`?` passes on an `Option`'s `None` out of a function returning an `Option`, and a `Result`'s `Err` out of one returning a `Result` [ERR-2]",
        ));
    }

    /// ADR-186 — whether `ty` has text to print: a written `Display`, or
    /// what the table formats.
    pub(crate) fn has_text(&self, ty: Ty) -> bool {
        let mut reached = ty;
        while let Some(inner) = self.types.printed_as(reached) {
            reached = inner;
        }
        self.written_display(reached)
            || self.dyn_reaches(ty, "std.core.Display")
            || self.dyn_reaches(ty, "std.core.Debug")
            || self.is_formattable(ty)
    }
}

/// ADR-186 — whether a method of the interface `decl` names `dyn name` in
/// its signature (`[ERR-3]`'s `Error` names `dyn Error`).
pub(crate) fn members_name(decl: &ast::InterfaceDecl, name: Symbol) -> bool {
    fn names_dyn(ty: &ast::TypeExpr, name: Symbol) -> bool {
        match &ty.kind {
            ast::TypeKind::Dyn(items) => items.iter().any(|item| {
                matches!(&item.kind, ast::TypeKind::Path { segments, .. } if segments.len() == 1 && segments[0].name == name)
                    || names_dyn(item, name)
            }),
            ast::TypeKind::Path { args, .. } => args.iter().any(|arg| match arg {
                ast::GenericArg::Type(inner) | ast::GenericArg::Assoc { ty: inner, .. } => names_dyn(inner, name),
                ast::GenericArg::Const(_) => false,
            }),
            ast::TypeKind::Ref { inner, .. } | ast::TypeKind::Ptr { inner, .. } | ast::TypeKind::Array { elem: inner, .. } => {
                names_dyn(inner, name)
            }
            ast::TypeKind::Tuple(items) => items.iter().any(|item| names_dyn(item, name)),
            ast::TypeKind::Fn { params, ret, .. } => {
                params.iter().any(|param| names_dyn(&param.ty, name)) || ret.as_deref().is_some_and(|ret| names_dyn(ret, name))
            }
            _ => false,
        }
    }
    let mentions = |ty: &ast::TypeExpr| names_dyn(ty, name);
    decl.members.iter().any(|member| match &member.kind {
        ast::MemberKind::Fn(f) => {
            f.ret.as_ref().is_some_and(mentions)
                || f.params.iter().any(|param| match &param.kind {
                    ast::ParamKind::Named { ty, .. } | ast::ParamKind::Receiver { ty: Some(ty) } => mentions(ty),
                    ast::ParamKind::Receiver { ty: None } => false,
                })
        }
        _ => false,
    })
}

impl InterfaceDef {
    /// ADR-186 — an interface entered before its members are read, so they
    /// may name it.
    pub(crate) fn entered(declaring_module: usize) -> InterfaceDef {
        InterfaceDef {
            generic_params: Vec::new(),
            methods: Vec::new(),
            defaults: Vec::new(),
            supertraits: Vec::new(),
            assoc: Vec::new(),
            assoc_bindings: Vec::new(),
            assoc_defaults: Vec::new(),
            supertrait_exprs: Vec::new(),
            declaring_module,
            dyn_sized_defaults: HashSet::new(),
        }
    }
}

impl<'a> Checker<'a> {
    /// ADR-186 — an interface naming itself was read as `dyn`-compatible
    /// while its methods were still to come; now they are known.
    pub(crate) fn check_self_dyn(&mut self, name: Symbol, span: Span) {
        if let Some(reason) = self.dyn_incompatibility(name) {
            self.sink.emit(
                Diagnostic::error(
                    codes::E2050,
                    span,
                    format!("interface `{name}` names itself in its methods, but is not `dyn`-compatible"),
                )
                .primary_label(reason),
            );
        }
    }
}

/// Whether `name` is the parameter a universal extension's target is.
fn universal_target_is(decl: &ast::ExtendDecl, name: Symbol) -> bool {
    matches!(&decl.target.kind, ast::TypeKind::Path { segments, .. } if segments.len() == 1 && segments[0].name == name)
}

/// `[TYP-19]` (ADR-186) — whether `decl` is `extend[T] T implements I[…]`:
/// its target is one of its own parameters, standing alone.
pub(crate) fn universal_target(decl: &ast::ExtendDecl) -> bool {
    let ast::TypeKind::Path { segments, args } = &decl.target.kind else { return false };
    let [single] = segments.as_slice() else { return false };
    args.is_empty()
        && !decl.implements.is_empty()
        && decl.generics.iter().chain(&decl.blanket).any(|param| param.name.name == single.name)
}

impl<'a> Checker<'a> {
    /// `[TYP-19]`, `[TYP-20]` (ADR-186) — record `extend[T] T implements
    /// I[…]`, which every type meeting `T`'s bounds has. `std.core` writes
    /// `[ERR-7]`'s `From[T]` this way. A type is given its methods and the
    /// implementation the first time one of them, or the interface, is asked
    /// of it (`apply_universal`), as an instance is given a generic
    /// extension's (`apply_generic_extensions`).
    pub(crate) fn collect_universal_extension(&mut self, decl: &ast::ExtendDecl, item: &ast::Item, item_index: usize) {
        let params = self.declare_generics(&decl.generics);
        // A parameter besides the target is a blanket's when an implemented interface names it
        // (`[GRM-34]`, moved to `decl.blanket` and its methods); one nothing names means nothing.
        if let Some(unnamed) = decl.generics.iter().find(|param| !universal_target_is(decl, param.name.name)) {
            self.error(
                codes::E2020,
                unnamed.name.span,
                format!("`{}` is named by neither the type extended nor an interface it implements", unnamed.name.name),
            );
            self.type_params.clear();
            return;
        }
        let target = self.resolve_type(&decl.target);
        // `[GRM-34]` (ADR-191) — `extend[T, U] T implements I[U]`: every type has its methods, each
        // generic over `U`, and `I[X]` for each `X` the bounds admit, given as a blanket
        // implementation's are when it is asked (`apply_blanket`).
        if !decl.blanket.is_empty() {
            let methods = self.universal_methods(decl, item_index, params.len());
            let recorded = self.record_blanket(decl, params.clone(), item.span);
            self.type_params.clear();
            self.projection_params.clear();
            self.count_op_params.clear();
            if recorded {
                self.universal_interfaces.push(Vec::new());
                self.universal_extensions.push(GenericExtension {
                    params,
                    target_args: vec![target],
                    methods,
                    interface: None,
                    interfaces: Vec::new(),
                    implements: Vec::new(),
                    assoc: Vec::new(),
                    span: item.span,
                    declaring_module: self.current_module,
                });
            }
            return;
        }
        // ODR-097 — a parameter standing alone is no package's type, so only
        // the interface's own package may write one.
        let mut patterns = Vec::new();
        for entry in &decl.implements {
            let Some(instance) = self.resolve_interface_use_for(entry, target) else { continue };
            let (origin, args) = self.open_interface_origin.get(&instance).cloned().unwrap_or((instance, Vec::new()));
            patterns.push((origin, args.clone()));
            if !self.placement_allowed_over(None, origin, &args) {
                let shown = self.types.display(target);
                let named = self.interface_shown_over(origin, &args);
                self.report_misplaced_implementation(shown, named, entry.span);
                self.type_params.clear();
                return;
            }
        }
        let methods = self.universal_methods(decl, item_index, params.len());
        let mut assoc = Vec::new();
        for member in &decl.members {
            if let ast::MemberKind::TypeAlias(alias) = &member.kind
                && let Some(value) = &alias.value
            {
                assoc.push((alias.name.name, self.resolve_type(value)));
            }
        }
        let interfaces: Vec<Symbol> =
            decl.implements.iter().filter_map(interface_name).map(|name| self.resolve_name(name)).collect();
        self.universal_interfaces.push(patterns);
        self.universal_extensions.push(GenericExtension {
            params,
            target_args: vec![target],
            methods,
            interface: interfaces.first().copied(),
            interfaces,
            implements: decl.implements.clone(),
            assoc,
            span: item.span,
            declaring_module: self.current_module,
        });
        self.type_params.clear();
        self.projection_params.clear();
        self.count_op_params.clear();
    }

    /// The methods of a universal extension, generic over its parameters (`base` of them).
    fn universal_methods(&mut self, decl: &ast::ExtendDecl, item_index: usize, base: usize) -> Vec<GenericMethod> {
        let mut methods = Vec::new();
        for (member_index, member) in decl.members.iter().enumerate() {
            let ast::MemberKind::Fn(fn_decl) = &member.kind else { continue };
            let Some((receiver, signature)) = self.generic_method_signature(fn_decl, &member.attrs, member.span, base) else {
                continue;
            };
            methods.push(GenericMethod {
                name: fn_decl.name.name,
                dispatch: fn_decl.dispatch,
                is_unsafe: fn_decl.is_unsafe,
                has_body: fn_decl.body.is_some(),
                receiver,
                params: signature.params,
                defaults: Self::declared_defaults(fn_decl),
                ret: signature.ret,
                generics: signature.generics,
                borrows: signature.borrows,
                overflow: signature.overflow,
                fp: signature.fp,
                source: (self.current_module, item_index, member_index),
                vis: member.vis.kind,
                span: member.span,
            });
        }
        methods
    }

    /// ADR-186 — give `ty` each universal implementation offering the method
    /// or associated function `name`.
    pub(crate) fn apply_universal_member(&mut self, ty: Ty, name: Symbol) {
        let offering: Vec<usize> = (0..self.universal_extensions.len())
            .filter(|&index| self.universal_extensions[index].methods.iter().any(|method| method.name == name))
            .collect();
        self.apply_universal(ty, offering);
    }

    /// ADR-186 — give `ty` each universal implementation of the interface
    /// instance `interface`'s origin.
    pub(crate) fn apply_universal_interface(&mut self, ty: Ty, interface: Symbol) {
        let origin = self.open_interface_origin.get(&interface).map_or(interface, |(origin, _)| *origin);
        let offering: Vec<usize> = (0..self.universal_interfaces.len())
            .filter(|&index| self.universal_interfaces[index].iter().any(|(implemented, _)| *implemented == origin))
            .collect();
        self.apply_universal(ty, offering);
    }

    /// ADR-186 — what `apply_generic_extensions` does for an instance, done
    /// for `ty` with each of the universal implementations `offering` whose
    /// bounds `ty` meets, once each. One the type already has from its own
    /// `extend` is not given beside it: that one is `E2041` where it is
    /// written (`report_universal_overlaps`).
    pub(crate) fn apply_universal(&mut self, ty: Ty, offering: Vec<usize>) {
        if offering.is_empty() {
            return;
        }
        // A type's own implementations are all collected first: one given
        // before would hide the `E2041` of a written one.
        if !self.bounds_known {
            return;
        }
        if matches!(self.types.kind(ty), TyKind::Param { .. } | TyKind::Error | TyKind::Never | TyKind::Infer(_)) {
            return;
        }
        for index in offering {
            if self.applied_universal.contains(&(ty, index)) {
                continue;
            }
            self.applied_universal.insert((ty, index));
            let extension = self.universal_extensions[index].clone();
            if !self.generic_extension_bounds_hold(&extension, &[ty]) {
                continue;
            }
            let mut bindings = vec![(extension.params[0].name, ty)];
            let owner_params = std::mem::replace(&mut self.type_params, bindings.iter().copied().collect());
            let owner_module = std::mem::replace(&mut self.current_module, extension.declaring_module);
            let instances: Vec<Symbol> =
                extension.implements.iter().filter_map(|entry| self.resolve_interface_use_for(entry, ty)).collect();
            self.type_params = owner_params;
            self.current_module = owner_module;
            if instances.iter().any(|&instance| self.implemented.iter().any(|&(t, i, _)| t == ty && i == instance)) {
                continue;
            }
            let interfaces = self.apply_extension_bindings(ty, &extension, &mut bindings);
            let report_owner = Symbol::intern(&format!("universal@{}", extension.span.start));
            for (method, interface) in extension.methods.iter().zip(interfaces) {
                if let Some(def) = self.register_recipe_method(ty, report_owner, method, &bindings, interface, false) {
                    self.emit_if_used_methods.insert(def);
                    self.universal_method_defs.insert(def, index);
                }
            }
            let owner_params = std::mem::replace(&mut self.type_params, bindings.iter().copied().collect());
            let implementations =
                self.collect_instantiated_implements(ty, &extension.implements, extension.span, extension.declaring_module);
            self.type_params = owner_params;
            self.record_extension_assoc(ty, &extension, &bindings, &implementations);
            for &(implemented, interface, _) in &implementations {
                let saved_instance = self.assoc_instance.replace(interface);
                self.fill_assoc_defaults(implemented, interface);
                self.assoc_instance = saved_instance;
            }
            self.register_instantiated_defaults(ty, &implementations);
            for (implemented_ty, interface, interface_span) in implementations {
                self.check_implementation(implemented_ty, interface, interface_span);
            }
        }
    }

    /// `[TYP-19]`, `[ERR-7]` (ADR-186) — an implementation that a universal
    /// one also gives some type is `E2041`, reported where it is written: the
    /// universal one is every type's (`From[B]` for `B`, a program's
    /// `extend B implements From[B]`).
    pub(crate) fn report_universal_overlaps(&mut self, records: &[ImplRecord]) {
        for (index, extension) in self.universal_extensions.clone().into_iter().enumerate() {
            let target = extension.target_args[0];
            for (origin, patterns) in self.universal_interfaces[index].clone() {
                for record in records {
                    let Some(whole) = record.whole else { continue };
                    if record.interface != origin
                        || record.interface_args.len() != patterns.len()
                        || self.overlap_reported_at.contains(&record.span)
                    {
                        continue;
                    }
                    let mut bound = HashMap::new();
                    let one = self.unify_sides(target, 0, whole, 1, &mut bound)
                        && patterns.iter().zip(&record.interface_args).all(|(&x, &y)| self.unify_sides(x, 0, y, 1, &mut bound));
                    if !one
                        || !self.known_bounds_hold(&extension.params, 0, &bound)
                        || !self.known_bounds_hold(&record.params, 1, &bound)
                    {
                        continue;
                    }
                    self.overlaps_reported.insert((extension.span, record.span));
                    self.overlaps_reported.insert((record.span, extension.span));
                    self.overlap_reported_at.insert(record.span);
                    let interface = self.interface_shown_over(record.interface, &record.interface_args);
                    self.sink.emit(
                        Diagnostic::error(
                            codes::E2041,
                            record.span,
                            format!("`{}` already implements `{interface}`", record.shown),
                        )
                        .secondary(extension.span, "every type implements it here")
                        .note("two implementations of one interface may not apply to one type [TYP-19]; the prelude converts every type from itself [ERR-7]"),
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// `@derive(Error)`
// ---------------------------------------------------------------------------

/// `[ERR-3]` (ADR-186) — `@derive(Error)` written out, before anything is
/// checked, as the `extend` blocks it stands for, so they are checked,
/// lowered and borrow-checked as written ones are:
///
/// * `extend T implements Display, Error`, whose `fmt` writes the `@error("…")`
///   text of the variant the value is (of the struct, for a struct), each
///   `{name}` or `{0}` the field it names, written by its `Display`; and whose
///   `source` is the payload of an `@from` variant, `None` for the others.
/// * for each `@from` variant `V(X)`, `extend T implements From[X]`, whose
///   `from` makes a `V`.
///
/// `Debug` is the type's field-wise one (`[STR-5]`); a `@no_derive(Debug)` is
/// overruled, since `Error` needs it. A misplaced `@error` or `@from`, a
/// variant without its text, or a text naming no field is reported here and
/// dropped, so nothing is checked twice.
/// `[FFI-16]` — `@ffi(status=E, ok=X, error=N)` on a C enum `E` declares `struct N(code: E)`
/// implementing `Error`, as these items (ODR-137):
///
/// ```text
/// @derive(Copy)
/// struct N:
///     pub code: E
/// extend N implements Display:
///     fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
///         return implicit_fmt_debug(self.code, f)
/// extend N implements Error
/// ```
///
/// with the enum's visibility. Its `Display` is the code's `Debug` (`Status.Busy`, or `Status(7)`
/// for a code no variant names); its `Debug` and `Eq` are a struct's own. The attribute's other
/// checks are the checker's (`Checker::check_status_enum`).
pub(crate) fn desugar_status_errors(modules: &[LoadedModule]) -> Option<Vec<LoadedModule>> {
    let marked = |item: &ast::Item| matches!(item.kind, ast::ItemKind::Enum(_)) && ffi_status_attr(&item.attrs).is_some();
    if !modules.iter().any(|loaded| loaded.module.items.iter().any(marked)) {
        return None;
    }
    let ffi_loaded = modules.iter().any(|loaded| loaded.path == ["std", "ffi"]);
    let mut out = modules.to_vec();
    for loaded in &mut out {
        let mut generated = Vec::new();
        for item in loaded.module.items.iter().filter(|item| marked(item)) {
            let ast::ItemKind::Enum(decl) = &item.kind else { continue };
            let Some(status) = ffi_status_attr(&item.attrs) else { continue };
            let span = status.error.span;
            let code = ast::Member {
                id: ast::NodeId::DUMMY,
                attrs: Vec::new(),
                vis: ast::Visibility { kind: ast::VisKind::Public, span },
                read_only_outside: false,
                doc: None,
                kind: ast::MemberKind::Field(ast::FieldDecl {
                    name: ast::Ident { name: Symbol::intern("code"), span },
                    ty: owner_type(ast::Ident { name: decl.name.name, span }, &[]),
                    default: None,
                    is_let: false,
                }),
                span,
            };
            let derive_copy = ast::Attribute {
                id: ast::NodeId::DUMMY,
                path: vec![ast::Ident { name: Symbol::intern("derive"), span }],
                args: vec![ast::AttrArg::Expr(path(&[Symbol::intern("Copy")], span))],
                span,
            };
            generated.push(ast::Item {
                id: ast::NodeId::DUMMY,
                attrs: vec![derive_copy],
                vis: ast::Visibility { kind: item.vis.kind, span },
                doc: Some(format!("The error of `{}`'s status codes: any but `{}` (`[FFI-16]`).", decl.name.name, status.ok.name)),
                kind: ast::ItemKind::Struct(ast::StructDecl {
                    name: status.error,
                    generics: Vec::new(),
                    implements: Vec::new(),
                    where_clause: Vec::new(),
                    members: vec![code],
                }),
                span,
            });
            let f = path(&[Symbol::intern("f")], span);
            let read = expr(ast::ExprKind::Field { base: Box::new(self_expr(span)), name: ast::Ident { name: Symbol::intern("code"), span } }, span);
            let body = ret(Some(call(path(&[Symbol::intern("std.fmt.implicit_fmt_debug")], span), vec![read, f], span)), span);
            generated.push(extend_item(&[], owner_type(status.error, &[]), vec![named_type("std.core.Display", span)], vec![fmt_member(body, span)], span));
            generated.push(extend_item(&[], owner_type(status.error, &[]), vec![named_type("std.core.Error", span)], Vec::new(), span));
            // `[HR-39]` — a foreign call's failure, which `?` makes `ReloadError.Foreign` inside a
            // `migrate_from`; named where `std.ffi` is part of the build.
            if ffi_loaded {
                generated.push(extend_item(&[], owner_type(status.error, &[]), vec![named_type("std.ffi.ForeignFailure", span)], Vec::new(), span));
            }
        }
        if !generated.is_empty() {
            std::sync::Arc::make_mut(&mut loaded.module).items.extend(generated);
        }
    }
    Some(out)
}

pub(crate) fn desugar_error_derives(modules: &[LoadedModule], sink: &mut Sink) -> Option<Vec<LoadedModule>> {
    let marked = |attrs: &[ast::Attribute]| {
        has_derive(attrs, "Error") || attrs.iter().any(|attr| error_attribute(attr).is_some())
    };
    let wanted = |module: &ast::Module| {
        module.items.iter().any(|item| {
            marked(&item.attrs)
                || matches!(&item.kind, ast::ItemKind::Enum(decl) if decl.variants.iter().any(|variant| marked(&variant.attrs)))
        })
    };
    if !modules.iter().any(|loaded| wanted(&loaded.module)) {
        return None;
    }
    let mut out = modules.to_vec();
    for loaded in out.iter_mut().filter(|loaded| wanted(&loaded.module)) {
        let module = std::sync::Arc::make_mut(&mut loaded.module);
        let mut generated = Vec::new();
        for item in &mut module.items {
            let ast::Item { attrs, kind, span, .. } = item;
            let derives = has_derive(attrs, "Error");
            if derives {
                remove_attribute_argument(attrs, "derive", "Error");
                remove_attribute_argument(attrs, "no_derive", "Debug");
            }
            match kind {
                ast::ItemKind::Enum(decl) if derives => {
                    refuse_error_attributes(attrs, "an enum: `@error` is written on each variant", sink);
                    generated.extend(derive_enum_error(decl, *span, sink));
                }
                ast::ItemKind::Struct(decl) if derives => {
                    let texts = take_error_attributes(attrs, "error");
                    for from in take_error_attributes(attrs, "from") {
                        misplaced(from.span, "`@from` is written on a variant of an enum deriving `Error`", sink);
                    }
                    generated.extend(derive_struct_error(decl, texts, *span, sink));
                }
                _ => {
                    if derives {
                        sink.emit(
                            Diagnostic::error(codes::E0104, *span, "`@derive(Error)` applies to a struct or an enum")
                                .note("`@derive(Error)` writes `Display` from `@error(\"…\")` attributes [ERR-3]"),
                        );
                    }
                    refuse_error_attributes(attrs, "an item deriving `Error`", sink);
                    if let ast::ItemKind::Enum(decl) = kind {
                        for variant in &mut decl.variants {
                            refuse_error_attributes(&mut variant.attrs, "a variant of an enum deriving `Error`", sink);
                        }
                    }
                }
            }
        }
        module.items.extend(generated);
    }
    Some(out)
}

/// `@error` or `@from`, by name.
fn error_attribute(attr: &ast::Attribute) -> Option<&'static str> {
    match attr.path.as_slice() {
        [single] if single.name.is("error") => Some("error"),
        [single] if single.name.is("from") => Some("from"),
        _ => None,
    }
}

/// Remove `name` from each `@attribute(…)`, and the attribute once empty.
pub(crate) fn remove_attribute_argument(attrs: &mut Vec<ast::Attribute>, attribute: &str, name: &str) {
    for attr in attrs.iter_mut().filter(|attr| attr.path.len() == 1 && attr.path[0].name.is(attribute)) {
        attr.args.retain(|arg| {
            !matches!(arg, ast::AttrArg::Expr(ast::Expr { kind: ast::ExprKind::Path { segments }, .. })
                if segments.len() == 1 && segments[0].name.is(name))
        });
    }
    attrs.retain(|attr| !(attr.path.len() == 1 && attr.path[0].name.is(attribute) && attr.args.is_empty()));
}

/// The `@error`s (or `@from`s) among `attrs`, taken out.
fn take_error_attributes(attrs: &mut Vec<ast::Attribute>, name: &str) -> Vec<ast::Attribute> {
    let (taken, kept) = std::mem::take(attrs).into_iter().partition(|attr| error_attribute(attr) == Some(name));
    *attrs = kept;
    taken
}

/// `@error` and `@from` where no `@derive(Error)` reads them: reported and
/// dropped.
fn refuse_error_attributes(attrs: &mut Vec<ast::Attribute>, place: &str, sink: &mut Sink) {
    for name in ["error", "from"] {
        for attr in take_error_attributes(attrs, name) {
            misplaced(attr.span, &format!("`@{name}` applies to {place}"), sink);
        }
    }
}

fn misplaced(span: Span, message: &str, sink: &mut Sink) {
    sink.emit(
        Diagnostic::error(codes::E0104, span, message.to_string())
            .note("`@error(\"…\")` is written on each variant of an enum deriving `Error` (on a struct deriving it, on the struct), `@from` on a variant with one field [ERR-3]"),
    );
}

/// One field a text may name: its name (`None` when positional) and the
/// expression reading it in the generated `fmt`.
struct NamedField {
    name: Option<Symbol>,
    read: ast::Expr,
}

/// The `@error("…")` text of one variant or struct, as the f-string writing
/// it, with the positions of the fields it reads; `None`, reported, when
/// there is no single string or it names no field.
fn error_text(texts: &[ast::Attribute], fields: &[NamedField], missing: (Span, &str), sink: &mut Sink) -> Option<(ast::Expr, HashSet<usize>)> {
    let Some(attr) = texts.first() else {
        sink.emit(
            Diagnostic::error(codes::E0104, missing.0, format!("`@derive(Error)` needs the text of {}", missing.1))
                .help("write it before it: `@error(\"what went wrong\")`")
                .note("`@derive(Error)` writes `Display` from an `@error(\"…\")` on each variant, or on the struct [ERR-3]"),
        );
        return None;
    };
    for extra in &texts[1..] {
        misplaced(extra.span, "one `@error` gives a variant its text; this is a second", sink);
    }
    let text = match attr.args.as_slice() {
        [ast::AttrArg::Expr(ast::Expr { kind: ast::ExprKind::Lit(ast::Literal::Str(text)), .. })] => text.clone(),
        _ => {
            misplaced(attr.span, "`@error` takes one string: `@error(\"…\")`", sink);
            return None;
        }
    };
    let span = attr.span;
    let mut parts = Vec::new();
    let mut read = HashSet::new();
    let mut literal = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                literal.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                literal.push('}');
            }
            '}' => {
                misplaced(span, "a `}` in an `@error` text is written `}}`", sink);
                return None;
            }
            '{' => {
                let mut hole = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some(c) => hole.push(c),
                        None => {
                            misplaced(span, "an `@error` text's `{` has no `}`", sink);
                            return None;
                        }
                    }
                }
                let (name, spec) = match hole.split_once(':') {
                    Some((name, spec)) => (name, Some(spec.to_string())),
                    None => (hole.as_str(), None),
                };
                let (name, conversion) = match name.split_once('!') {
                    Some((name, conversion)) => {
                        let mut letters = conversion.chars();
                        match (letters.next(), letters.next()) {
                            (Some(letter), None) => (name, Some(letter)),
                            _ => {
                                misplaced(span, &format!("`!{conversion}` is not a conversion"), sink);
                                return None;
                            }
                        }
                    }
                    None => (name, None),
                };
                let name = name.trim();
                let index = match name.parse::<usize>() {
                    Ok(index) => (index < fields.len()).then_some(index),
                    Err(_) => fields.iter().position(|field| field.name.is_some_and(|field| field.as_str() == name)),
                };
                let Some(field) = index.map(|index| &fields[index]) else {
                    sink.emit(
                        Diagnostic::error(codes::E1010, span, format!("the `@error` text names `{{{name}}}`, which is none of the fields"))
                            .note("a placeholder names a field, `{name}`, or its position, `{0}` [ERR-3]"),
                    );
                    return None;
                };
                read.extend(index);
                if !literal.is_empty() {
                    parts.push(ast::FStringPart::Text(std::mem::take(&mut literal)));
                }
                parts.push(ast::FStringPart::Expr {
                    expr: Box::new(field.read.clone()),
                    format_spec: spec,
                    echo: None,
                    conversion,
                });
            }
            c => literal.push(c),
        }
    }
    if !literal.is_empty() {
        parts.push(ast::FStringPart::Text(literal));
    }
    Some((expr(ast::ExprKind::FString(parts), span), read))
}

/// `[ERR-3]` — the blocks `@derive(Error)` on an enum stands for.
fn derive_enum_error(decl: &mut ast::EnumDecl, span: Span, sink: &mut Sink) -> Vec<ast::Item> {
    let enum_name = decl.name;
    let mut fmt_arms = Vec::new();
    let mut source_arms = Vec::new();
    let mut froms = Vec::new();
    let mut complete = true;
    for variant in &mut decl.variants {
        let texts = take_error_attributes(&mut variant.attrs, "error");
        let from = take_error_attributes(&mut variant.attrs, "from");
        let at = variant.span;
        let bindings: Vec<Symbol> = variant
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| field.name.map_or_else(|| Symbol::intern(&format!("${index}")), |name| name.name))
            .collect();
        let fields: Vec<NamedField> = variant
            .fields
            .iter()
            .zip(&bindings)
            .map(|(field, &binding)| NamedField { name: field.name.map(|name| name.name), read: path(&[binding], at) })
            .collect();
        let what = format!("`{}.{}`", enum_name.name, variant.name.name);
        match error_text(&texts, &fields, (variant.name.span, &what), sink) {
            Some((text, read)) => {
                let pattern = variant_pattern(enum_name, variant, &bindings, &read, at);
                fmt_arms.push(arm(pattern, write_str(text, at), at));
            }
            None => complete = false,
        }
        if let Some(attr) = from.first() {
            for extra in &from[1..] {
                misplaced(extra.span, "one `@from` marks a variant; this is a second", sink);
            }
            let [field] = variant.fields.as_slice() else {
                misplaced(attr.span, "`@from` marks a variant with one field, the error it converts from", sink);
                continue;
            };
            let some = call(path(&[Symbol::intern("Some")], attr.span), vec![path(&[bindings[0]], attr.span)], attr.span);
            let pattern = variant_pattern(enum_name, variant, &bindings, &HashSet::from([0]), attr.span);
            source_arms.push(arm(pattern, ret(Some(some), attr.span), attr.span));
            froms.push((field.ty.clone(), variant.name, attr.span));
        }
    }
    let target = owner_type(enum_name, &decl.generics);
    let mut items = Vec::new();
    if complete {
        let mut members = vec![fmt_member(stmt(ast::StmtKind::Match { scrutinee: self_expr(span), arms: fmt_arms }, span), span)];
        if !source_arms.is_empty() {
            if source_arms.len() < decl.variants.len() {
                let none = path(&[Symbol::intern("None")], span);
                source_arms.push(arm(pattern_of(ast::PatternKind::Wild, span), ret(Some(none), span), span));
            }
            members.push(source_member(stmt(ast::StmtKind::Match { scrutinee: self_expr(span), arms: source_arms }, span), span));
        }
        items.push(extend_item(&decl.generics, target.clone(), vec![named_type("std.core.Display", span), named_type("std.core.Error", span)], members, span));
    }
    for (field_ty, variant, at) in froms {
        let value = Symbol::intern("value");
        let made = method_call(path(&[enum_name.name], at), variant, vec![path(&[value], at)], at);
        let from = ast::FnDecl {
            name: ast::Ident { name: Symbol::intern("from"), span: at },
            params: vec![named_param(value, ast::Mode::Owned, field_ty.clone(), at)],
            ret: Some(target.clone()),
            ..fn_decl(at)
        };
        let from = member(from, block(vec![ret(Some(made), at)], at), at);
        let interface = ast::TypeExpr {
            id: ast::NodeId::DUMMY,
            kind: ast::TypeKind::Path {
                segments: vec![ast::Ident { name: Symbol::intern("std.core.From"), span: at }],
                args: vec![ast::GenericArg::Type(field_ty)],
            },
            span: at,
        };
        items.push(extend_item(&decl.generics, target.clone(), vec![interface], vec![from], at));
    }
    items
}

/// `[ERR-3]` — the block `@derive(Error)` on a struct stands for: its one
/// `@error` text, whose placeholders read `self`'s fields.
fn derive_struct_error(decl: &ast::StructDecl, texts: Vec<ast::Attribute>, span: Span, sink: &mut Sink) -> Vec<ast::Item> {
    let fields: Vec<NamedField> = decl
        .members
        .iter()
        .filter_map(|member| match &member.kind {
            ast::MemberKind::Field(field) => Some(NamedField {
                name: Some(field.name.name),
                read: expr(ast::ExprKind::Field { base: Box::new(self_expr(span)), name: field.name }, span),
            }),
            _ => None,
        })
        .collect();
    let what = format!("`{}`", decl.name.name);
    let Some((text, _)) = error_text(&texts, &fields, (span, &what), sink) else { return Vec::new() };
    let members = vec![fmt_member(write_str(text, span), span)];
    let target = owner_type(decl.name, &decl.generics);
    vec![extend_item(&decl.generics, target, vec![named_type("std.core.Display", span), named_type("std.core.Error", span)], members, span)]
}

pub(crate) fn expr(kind: ast::ExprKind, span: Span) -> ast::Expr {
    ast::Expr { id: ast::NodeId::DUMMY, kind, span }
}

pub(crate) fn self_expr(span: Span) -> ast::Expr {
    expr(ast::ExprKind::SelfExpr, span)
}

pub(crate) fn path(segments: &[Symbol], span: Span) -> ast::Expr {
    expr(ast::ExprKind::Path { segments: segments.iter().map(|&name| ast::Ident { name, span }).collect() }, span)
}

pub(crate) fn arg(value: ast::Expr) -> ast::Arg {
    let span = value.span;
    ast::Arg { name: None, value, span }
}

pub(crate) fn call(callee: ast::Expr, args: Vec<ast::Expr>, span: Span) -> ast::Expr {
    expr(ast::ExprKind::Call { callee: Box::new(callee), args: args.into_iter().map(arg).collect() }, span)
}

pub(crate) fn method_call(recv: ast::Expr, name: ast::Ident, args: Vec<ast::Expr>, span: Span) -> ast::Expr {
    let name = ast::Ident { name: name.name, span };
    expr(
        ast::ExprKind::MethodCall { recv: Box::new(recv), name, generic_args: Vec::new(), args: args.into_iter().map(arg).collect() },
        span,
    )
}

pub(crate) fn stmt(kind: ast::StmtKind, span: Span) -> ast::Stmt {
    ast::Stmt { id: ast::NodeId::DUMMY, attrs: Vec::new(), kind, span }
}

pub(crate) fn ret(value: Option<ast::Expr>, span: Span) -> ast::Stmt {
    stmt(ast::StmtKind::Expr(expr(ast::ExprKind::Jump(ast::Jump::Return(value.map(Box::new))), span)), span)
}

/// `return f.write_str(text)`.
pub(crate) fn write_str(text: ast::Expr, span: Span) -> ast::Stmt {
    let f = path(&[Symbol::intern("f")], span);
    ret(Some(method_call(f, ast::Ident { name: Symbol::intern("write_str"), span }, vec![text], span)), span)
}

pub(crate) fn block(stmts: Vec<ast::Stmt>, span: Span) -> ast::Block {
    ast::Block { id: ast::NodeId::DUMMY, stmts, span }
}

pub(crate) fn pattern_of(kind: ast::PatternKind, span: Span) -> ast::Pattern {
    ast::Pattern { id: ast::NodeId::DUMMY, kind, span }
}

pub(crate) fn arm(pattern: ast::Pattern, body: ast::Stmt, span: Span) -> ast::MatchArm {
    ast::MatchArm { id: ast::NodeId::DUMMY, pattern, guard: None, body: ast::MatchArmBody::Block(block(vec![body], span)), span }
}

/// `E.V(ref a, _)`, each field in `read` bound by reference to its name in
/// `bindings`, the others `_`; `E.V` for a unit variant.
fn variant_pattern(enum_name: ast::Ident, variant: &ast::Variant, bindings: &[Symbol], read: &HashSet<usize>, span: Span) -> ast::Pattern {
    let segments = vec![ast::Ident { name: enum_name.name, span }, ast::Ident { name: variant.name.name, span }];
    if variant.fields.is_empty() {
        return pattern_of(ast::PatternKind::Path { segments }, span);
    }
    let fields = variant
        .fields
        .iter()
        .zip(bindings)
        .enumerate()
        .map(|(index, (field, &binding))| ast::FieldPattern {
            name: field.name,
            pattern: match read.contains(&index) {
                true => pattern_of(
                    ast::PatternKind::Bind { name: ast::Ident { name: binding, span }, by_ref: true, mutable: false, sub: None },
                    span,
                ),
                false => pattern_of(ast::PatternKind::Wild, span),
            },
            span,
        })
        .collect();
    pattern_of(ast::PatternKind::Constructor { path: segments, fields, has_rest: false }, span)
}

/// A type named by its qualified name, which resolves wherever it is written.
pub(crate) fn named_type(name: &str, span: Span) -> ast::TypeExpr {
    ast::TypeExpr {
        id: ast::NodeId::DUMMY,
        kind: ast::TypeKind::Path { segments: vec![ast::Ident { name: Symbol::intern(name), span }], args: Vec::new() },
        span,
    }
}

/// `E` or `E[T, U]`, over the declaration's own parameters.
pub(crate) fn owner_type(name: ast::Ident, generics: &[ast::GenericParam]) -> ast::TypeExpr {
    let args = generics
        .iter()
        .map(|param| ast::GenericArg::Type(named_type(param.name.name.as_str(), param.span)))
        .collect();
    ast::TypeExpr { id: ast::NodeId::DUMMY, kind: ast::TypeKind::Path { segments: vec![name], args }, span: name.span }
}

pub(crate) fn fn_decl(span: Span) -> ast::FnDecl {
    ast::FnDecl {
        name: ast::Ident { name: Symbol::intern("_"), span },
        is_unsafe: false,
        is_safe: false,
        is_foreign_decl: false,
        abi: None,
        is_gen: false,
        dispatch: ast::Dispatch::Static,
        generics: Vec::new(),
        params: Vec::new(),
        ret: None,
        where_clause: Vec::new(),
        body: None,
    }
}

pub(crate) fn receiver(span: Span) -> ast::Param {
    ast::Param { id: ast::NodeId::DUMMY, mode: ast::Mode::Borrow, kind: ast::ParamKind::Receiver { ty: None }, default: None, span }
}

pub(crate) fn named_param(name: Symbol, mode: ast::Mode, ty: ast::TypeExpr, span: Span) -> ast::Param {
    ast::Param { id: ast::NodeId::DUMMY, mode, kind: ast::ParamKind::Named { name: ast::Ident { name, span }, ty }, default: None, span }
}

pub(crate) fn member(mut decl: ast::FnDecl, body: ast::Block, span: Span) -> ast::Member {
    decl.body = Some(body);
    // `[STD-1]` — a derived `Display` or `Debug` writes into its formatter's text, which allocates.
    let doc = matches!(decl.name.name.as_str(), "fmt" | "fmt_debug")
        .then(|| "Written by a derive. It allocates: it writes into the formatter's text.".to_string());
    ast::Member {
        id: ast::NodeId::DUMMY,
        attrs: Vec::new(),
        vis: ast::Visibility::private(),
        read_only_outside: false,
        doc,
        kind: ast::MemberKind::Fn(decl),
        span,
    }
}

/// `fn fmt(self, mut f: Formatter) -> Result[void, FmtError]: <body>`.
pub(crate) fn fmt_member(body: ast::Stmt, span: Span) -> ast::Member {
    let result = ast::TypeExpr {
        id: ast::NodeId::DUMMY,
        kind: ast::TypeKind::Path {
            segments: vec![ast::Ident { name: Symbol::intern("Result"), span }],
            args: vec![
                ast::GenericArg::Type(ast::TypeExpr { id: ast::NodeId::DUMMY, kind: ast::TypeKind::Void, span }),
                ast::GenericArg::Type(named_type("std.fmt.FmtError", span)),
            ],
        },
        span,
    };
    let decl = ast::FnDecl {
        name: ast::Ident { name: Symbol::intern("fmt"), span },
        params: vec![receiver(span), named_param(Symbol::intern("f"), ast::Mode::Mut, named_type("std.fmt.Formatter", span), span)],
        ret: Some(result),
        ..fn_decl(span)
    };
    member(decl, block(vec![body], span), span)
}

/// `fn source(self) -> Option[ref dyn Error]: <body>`.
fn source_member(body: ast::Stmt, span: Span) -> ast::Member {
    let error = ast::TypeExpr { id: ast::NodeId::DUMMY, kind: ast::TypeKind::Dyn(vec![named_type("std.core.Error", span)]), span };
    let reference = ast::TypeExpr { id: ast::NodeId::DUMMY, kind: ast::TypeKind::Ref { mutable: false, inner: Box::new(error) }, span };
    let option = ast::TypeExpr {
        id: ast::NodeId::DUMMY,
        kind: ast::TypeKind::Path {
            segments: vec![ast::Ident { name: Symbol::intern("Option"), span }],
            args: vec![ast::GenericArg::Type(reference)],
        },
        span,
    };
    let decl = ast::FnDecl {
        name: ast::Ident { name: Symbol::intern("source"), span },
        params: vec![receiver(span)],
        ret: Some(option),
        ..fn_decl(span)
    };
    member(decl, block(vec![body], span), span)
}

pub(crate) fn extend_item(generics: &[ast::GenericParam], target: ast::TypeExpr, implements: Vec<ast::TypeExpr>, members: Vec<ast::Member>, span: Span) -> ast::Item {
    ast::Item {
        id: ast::NodeId::DUMMY,
        attrs: Vec::new(),
        vis: ast::Visibility::private(),
        doc: None,
        kind: ast::ItemKind::Extend(ast::ExtendDecl {
            is_unsafe: false,
            generics: generics.to_vec(),
            target,
            implements,
            where_clause: Vec::new(),
            members,
            blanket: Vec::new(),
        }),
        span,
    }
}
