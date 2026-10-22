//! `[DIA-21]` — the answers to Python habits (`ember_diag::habits`, ODR-179). Each is computed from
//! what the program means where it failed — the type of the argument a Python conversion was given,
//! the receiver a method was called on, the type a place expects — and carries the Ember form as a
//! machine-applicable fix-it where one edit says it. Nothing here runs unless checking already
//! failed, so a program that compiles pays nothing for it.

use super::*;
use ember_diag::Replacement;
use ember_diag::habits::Habit;

/// The edits that keep only `inner` of `outer` and append `suffix` to it: `max(xs)` becomes
/// `xs.iter().max()`. `inner` is bracketed when a postfix operator would not bind to all of it.
pub(crate) fn receiver_edits(outer: Span, inner: &ast::Expr, suffix: &str) -> Vec<(Span, Replacement)> {
    let (open, close) = if binds_as_operand(inner) { ("", "") } else { ("(", ")") };
    vec![
        (Span::new(outer.file, outer.start, inner.span.start), Replacement::Text(open.to_string())),
        (Span::new(outer.file, inner.span.end, outer.end), Replacement::Text(format!("{close}{suffix}"))),
    ]
}

/// Whether `expr` binds tighter than any postfix (`.m()`, `[i]`) or `as` applied to it: a name, a
/// literal, a call, a field, an index, a bracketed form.
pub(crate) fn binds_as_operand(expr: &ast::Expr) -> bool {
    matches!(
        expr.kind,
        ast::ExprKind::Lit(_)
            | ast::ExprKind::Path { .. }
            | ast::ExprKind::SelfExpr
            | ast::ExprKind::Field { .. }
            | ast::ExprKind::TupleField { .. }
            | ast::ExprKind::IndexOrInstantiate { .. }
            | ast::ExprKind::Call { .. }
            | ast::ExprKind::MethodCall { .. }
            | ast::ExprKind::Paren(_)
            | ast::ExprKind::Tuple(_)
            | ast::ExprKind::ArrayLit(_)
            | ast::ExprKind::MapLit(_)
            | ast::ExprKind::SetLit(_)
            | ast::ExprKind::FString(_)
            | ast::ExprKind::Try(_)
    ) || matches!(&expr.kind, ast::ExprKind::Comprehension { kind, .. } if !matches!(kind, ast::ComprehensionKind::Generator))
}

/// The same question of a checked expression, for a fix-it on a value that was already checked.
pub(crate) fn checked_binds_as_operand(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::Local(_)
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Str(_)
            | ExprKind::Field { .. }
            | ExprKind::Call { .. }
            | ExprKind::FString { .. }
    )
}

/// `suffix` appended to the checked expression at `span`, bracketed where it must be.
pub(crate) fn suffix_edits(span: Span, operand: bool, suffix: &str) -> Vec<(Span, Replacement)> {
    if operand {
        vec![(span.shrink_to_end(), Replacement::Text(suffix.to_string()))]
    } else {
        vec![
            (span.shrink_to_start(), Replacement::Text("(".to_string())),
            (span.shrink_to_end(), Replacement::Text(format!("){suffix}"))),
        ]
    }
}

impl Checker<'_> {
    /// `[DIA-21]` P1 — `True` and `False`, which Ember spells in lowercase.
    pub(crate) fn python_constant(&self, name: Symbol, span: Span) -> Option<Diagnostic> {
        let ember = match name.as_str() {
            "True" => "true",
            "False" => "false",
            _ => return None,
        };
        Some(
            Diagnostic::error(codes::E1010, span, format!("cannot find `{name}` in this scope: Ember's boolean is `{ember}`"))
                .habit(Habit::P1)
                .suggest(format!("write `{ember}`"), span, ember),
        )
    }

    /// `[DIA-21]` P1 — `None` where the type expected is not an `Option`: Python's "no value" is an
    /// `Option`'s in Ember, so the place must be declared one.
    pub(crate) fn none_without_option(&self, expected: Ty, span: Span) -> Diagnostic {
        // Python's `return None` from a function with no result: Ember's is a bare `return`.
        if expected == self.common.void {
            return Diagnostic::error(codes::E2020, span, "expected nothing, found `None`")
                .primary_label("Python's `None` for no result")
                .habit(Habit::P1)
                .help("a function with no result returns with a bare `return`; `None` is a value of an `Option` only");
        }
        let wanted = self.types.display(expected);
        Diagnostic::error(codes::E2020, span, format!("expected `{wanted}`, found `None`"))
            .primary_label("`None` is a value of an `Option` only")
            .habit(Habit::P1)
            .help(format!("`None` needs an `Option` type: declare the place `Option[{wanted}]`, or give it a value of `{wanted}`"))
    }

    /// `[DIA-21]` P3 (ODR-184) — `str(x)`, `int(x)` and `float(x)`, answered by what `x` is: text is
    /// parsed (`s.parse[int]()`, ending in `?` where the function's `Result` takes the failure and
    /// `.unwrap()` elsewhere, as Python raises), a number is converted with `as`, and anything is
    /// made text with `to_string()`. False when the name is not one of these; true when it is, and
    /// the answer, or the argument's own error, is reported.
    pub(crate) fn python_conversion(&mut self, name: Symbol, args: &[ast::Arg], name_span: Span, span: Span) -> bool {
        let target = match name.as_str() {
            "str" => None,
            "int" => Some("int"),
            "float" => Some("float"),
            _ => return false,
        };
        let [arg] = args else { return false };
        if arg.name.is_some() {
            return false;
        }
        let mark = self.sink.mark();
        let value = self.synth(&arg.value);
        if value.ty == self.common.error && self.sink.mark() != mark {
            // `[DIA-14]` — the argument's error is the one reported.
            return true;
        }
        self.sink.rollback(mark);
        let ty = match *self.types.kind(value.ty) {
            TyKind::Ref { inner, .. } => inner,
            _ => value.ty,
        };
        let shown = self.types.display(ty);
        let diagnostic = Diagnostic::error(codes::E1010, name_span, format!("cannot find `{name}` in this scope"))
            .primary_label("a Python conversion")
            .habit(Habit::P3);
        let diagnostic = match target {
            None => {
                let edits = receiver_edits(span, &arg.value, ".to_string()");
                diagnostic.suggest_edits(format!("`{shown}` is made text with `to_string()`"), edits)
            }
            Some(target) if self.is_text(ty) => {
                let ending = if self.failure_returns_parse_error() { "?" } else { ".unwrap()" };
                let edits = receiver_edits(span, &arg.value, &format!(".parse[{target}](){ending}"));
                let why = if ending == "?" {
                    "the function's `Result` takes the failure"
                } else {
                    "`unwrap` stops the program on text that is not a number, as Python raises"
                };
                diagnostic.suggest_edits(format!("text is parsed: `parse[{target}]()` gives a `Result`, and {why}"), edits)
            }
            Some(target) if self.types.is_numeric(ty) || self.types.is_untyped_literal(ty) => {
                let edits = receiver_edits(span, &arg.value, &format!(" as {target}"));
                diagnostic.suggest_edits(format!("a number is converted with `as`: `as {target}`"), edits)
            }
            Some(target) => diagnostic.help(format!("Ember converts a number with `as {target}` and parses text with `parse[{target}]()`; a `{shown}` is neither")),
        };
        self.sink.emit(diagnostic);
        true
    }

    /// Whether a `?` in the function being checked passes a `ParseError` on: its result is a
    /// `Result` whose error is an `AnyError` (`[ERR-8]`) or a `ParseError`.
    fn failure_returns_parse_error(&self) -> bool {
        let TyKind::Enum(id) = *self.types.kind(self.ret_ty) else { return false };
        if !self.is_result(self.ret_ty) {
            return false;
        }
        let error = self.types.enum_def(id).variants.get(1).and_then(|v| v.fields.first()).map(|f| f.ty);
        let named = |name: &str| self.named_types.get(&Symbol::intern(name)).copied();
        error.is_some_and(|error| Some(error) == named("std.core.AnyError") || Some(error) == named("std.string.ParseError"))
    }

    /// `[DIA-21]` P5 — `min(xs)` or `max(xs)` of one collection or iterator: its iterator's `min()`
    /// or `max()`, which is `None` when it is empty, unwrapped, as Python raises on an empty one.
    /// `None` when the one argument is neither.
    pub(crate) fn min_max_of_one(&mut self, name: &str, arg: &ast::Expr, span: Span) -> Option<Diagnostic> {
        let mark = self.sink.mark();
        let value = self.synth(arg);
        self.sink.rollback(mark);
        if value.ty == self.common.error {
            return None;
        }
        let ty = match *self.types.kind(value.ty) {
            TyKind::Ref { inner, .. } => inner,
            _ => value.ty,
        };
        let iterator = self.implements(ty, Symbol::intern("std.core.Iterator")) || self.lookup_method(ty, Symbol::intern("next")).is_some();
        let collection = !iterator && (self.lookup_method(ty, Symbol::intern("iter")).is_some()
            || matches!(self.types.kind(ty), TyKind::Vec { text: false, .. } | TyKind::Span { .. } | TyKind::Array { .. } | TyKind::ArrayN { .. }));
        if !iterator && !collection {
            return None;
        }
        let through = if iterator { "" } else { ".iter()" };
        let suffix = format!("{through}.{name}().unwrap()");
        let diagnostic = Diagnostic::error(codes::E2020, span, format!("`{name}` of one collection is its iterator's `{name}()`"))
            .primary_label(format!("Python's `{name}` of a collection"))
            .habit(Habit::P5)
            .note(format!("the prelude's `{name}(a, b)` takes two values [STD-26]; `{name}()` of an iterator is `None` when it is empty"));
        Some(diagnostic.suggest_edits(format!("write `{}{suffix}`", if iterator { "it" } else { "xs" }), receiver_edits(span, arg, &suffix)))
    }

    /// `[DIA-14]` — whether `ty` is an error or holds one (`Array[<error>]`, `(<error>, int)`): a
    /// type a failed expression gave, about which nothing more is reported. Asked on failing paths.
    pub(crate) fn mentions_error(&self, ty: Ty) -> bool {
        if ty == self.common.error {
            return true;
        }
        match self.types.kind(ty) {
            TyKind::Vec { elem, .. } | TyKind::Array { elem, .. } | TyKind::ArrayN { elem, .. } | TyKind::Span { elem, .. } => self.mentions_error(*elem),
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => self.mentions_error(*inner),
            TyKind::Tuple(items) => items.iter().any(|&item| self.mentions_error(item)),
            TyKind::Struct(id) => self.types.struct_def(*id).origin.as_ref().is_some_and(|(_, args)| args.iter().any(|&arg| self.mentions_error(arg))),
            TyKind::Enum(id) => self.types.enum_def(*id).origin.as_ref().is_some_and(|(_, args)| args.iter().any(|&arg| self.mentions_error(arg))),
            _ => false,
        }
    }

    /// `[DIA-21]` P22 — the interfaces named as types before the implementations were known, asked
    /// now: each one implemented by values only is reported where it was named.
    pub(crate) fn report_early_interface_types(&mut self) {
        for (interface, span) in std::mem::take(&mut self.early_interface_types) {
            if let Some(diagnostic) = self.interface_of_values(interface, span) {
                self.values_only_interfaces.insert(interface);
                self.sink.emit(diagnostic);
            }
        }
    }

    /// The generic type `ty` is an instance of, by its qualified name (`std.collections.Map`).
    fn origin_of(&self, ty: Ty) -> Option<Symbol> {
        match *self.types.kind(ty) {
            TyKind::Struct(id) => self.types.struct_def(id).origin.as_ref().map(|(origin, _)| *origin),
            TyKind::Class(id) => self.types.class_def(id).origin.as_ref().map(|(origin, _)| *origin),
            _ => None,
        }
    }

    /// `[DIA-21]` P6 — a call Python writes with arguments Ember's method of that name does not
    /// take, where Ember spells the meaning with another method (Appendix E): `xs.sort(key=f)` is
    /// `xs.sort_by_key(f)`, `m.get(k, d)` is `m.get_or(k, d)`, `s.split()` is
    /// `s.split_whitespace()`. Each shape is an arity error in Ember, so a program that compiles
    /// never matches one. `Some(None)`: the shape, on a receiver whose type holds an error, about
    /// which nothing more is said (`[DIA-14]`).
    pub(crate) fn python_call_shape(&self, receiver: &Expr, name: ast::Ident, args: &[ast::Arg]) -> Option<Option<Diagnostic>> {
        let answer = self.python_call_shape_of(receiver, name, args)?;
        Some((!self.mentions_error(receiver.ty)).then_some(answer))
    }

    fn python_call_shape_of(&self, receiver: &Expr, name: ast::Ident, args: &[ast::Arg]) -> Option<Diagnostic> {
        let ty = receiver.ty;
        let shown = self.types.display(ty);
        let rename = |ember: &str, why: &str| {
            Diagnostic::error(codes::E2020, name.span, format!("`{shown}`'s `{}` does not take these arguments", name.name))
                .primary_label(format!("Python's `{}`", name.name))
                .habit(Habit::P6)
                .note(format!("{why} (Appendix E) [STD-13]"))
                .suggest(format!("write `{ember}`"), name.span, ember)
        };
        match name.name.as_str() {
            "sort" if matches!(self.types.kind(ty), TyKind::Vec { text: false, .. }) || self.mentions_error(ty) => {
                let key = args.iter().find(|arg| arg.name.is_some_and(|n| n.name.is("key")))?;
                if args.len() != 1 {
                    return Some(
                        Diagnostic::error(codes::E2020, name.span, format!("`{shown}`'s `sort` does not take these arguments"))
                            .primary_label("Python's `sort`")
                            .habit(Habit::P6)
                            .help("sort by the key with `sort_by_key(f)`, then call `reverse()` for the descending order (Appendix E) [STD-13]"),
                    );
                }
                let keyword = key.name.expect("found by its name");
                let mut diagnostic = rename("sort_by_key", "Ember sorts by a key with `sort_by_key(f)`, which takes the key function itself");
                if let Some(fix) = diagnostic.suggestions.last_mut() {
                    fix.edits.push((Span::new(keyword.span.file, keyword.span.start, key.value.span.start), Replacement::Text(String::new())));
                }
                Some(diagnostic)
            }
            "get" if args.len() == 2 && args.iter().all(|arg| arg.name.is_none()) => {
                (self.origin_of(ty)?.as_str() == "std.collections.Map" || self.mentions_error(ty))
                    .then(|| rename("get_or", "Python's `d.get(k, default)` is `m.get_or(k, default)`; `m.get(k)` gives an `Option`"))
            }
            "split" if args.is_empty() && self.is_text(ty) => {
                Some(rename("split_whitespace", "Python's `s.split()` with no separator splits at runs of white space: `s.split_whitespace()`"))
            }
            _ => None,
        }
    }

    /// `[STD-13]`, `[DIA-12]` N3 — `ty` has no method `name`: `E2072`, whose help is Ember's name for
    /// a Python one (`[DIA-21]` P6, P20), the `collect` a list operation on an iterator needs (P25),
    /// or the nearest members, and the interface that would supply it.
    /// `None` when the mistake was answered already (`[DIA-14]`).
    pub(crate) fn no_method(&mut self, receiver: &Expr, name: ast::Ident, args: &[ast::Arg], span: Span) -> Option<Diagnostic> {
        let ty = receiver.ty;
        let shown = self.types.display(ty);
        let diagnostic = Diagnostic::error(codes::E2072, name.span, format!("`{shown}` has no method named `{}`", name.name));
        if let Some(answered) = self.python_method(receiver, name, args, span, diagnostic.clone()) {
            return Some(answered);
        }
        if name.name.is("len") && let Some(answered) = self.list_operation_on_iterator(receiver, diagnostic.clone()) {
            return answered;
        }
        let mut diagnostic = diagnostic;
        for candidate in self.method_name_suggestions(ty, name.name, name.span) {
            diagnostic = diagnostic.suggest(format!("did you mean `{candidate}`?"), name.span, candidate);
        }
        Some(self.note_unmet_extension_bound(diagnostic, ty, name.name))
    }

    /// `[DIA-21]` P6, P20 — a Python method name, for the receiver kind Python has it on.
    fn python_method(&self, receiver: &Expr, name: ast::Ident, args: &[ast::Arg], span: Span, diagnostic: Diagnostic) -> Option<Diagnostic> {
        let ty = receiver.ty;
        let text = self.is_text(ty);
        let list = matches!(self.types.kind(ty), TyKind::Vec { text: false, .. });
        let character = matches!(self.types.kind(ty), TyKind::Char);
        let ember = match name.name.as_str() {
            "append" if list => "push",
            "index" if list => "index_of",
            "strip" if text => "trim",
            "lstrip" if text => "trim_start",
            "rstrip" if text => "trim_end",
            "splitlines" if text => "lines",
            "startswith" if text => "starts_with",
            "endswith" if text => "ends_with",
            "upper" if text => "to_upper",
            "lower" if text => "to_lower",
            // ODR-183 — Python's character classes, Python's meaning.
            "isdigit" if character => "is_digit",
            "isalpha" if character => "is_alphabetic",
            "isalnum" if character => "is_alphanumeric",
            "isspace" if character => "is_whitespace",
            "join" if text => return Some(self.python_join(receiver, args, span, diagnostic)),
            "format" if text => return Some(self.python_format(receiver, args, span, diagnostic)),
            _ => return None,
        };
        Some(
            diagnostic
                .habit(Habit::P6)
                .note("a Python method Ember spells otherwise has one name, Ember's (Appendix E) [STD-13]")
                .suggest(format!("Ember's name for it is `{ember}`"), name.span, ember),
        )
    }

    /// `[DIA-21]` P6 — `sep.join(xs)` is `xs.join(sep)`: a collection or an iterator joins its
    /// items' text.
    fn python_join(&self, receiver: &Expr, args: &[ast::Arg], span: Span, diagnostic: Diagnostic) -> Diagnostic {
        let diagnostic = diagnostic.habit(Habit::P6).note("Ember joins from the items: `xs.join(sep)` (Appendix E) [STD-13]");
        let [items] = args else { return diagnostic.help("write `xs.join(sep)`") };
        if items.name.is_some() {
            return diagnostic.help("write `xs.join(sep)`");
        }
        let separator = receiver.span;
        let (open, close) = if binds_as_operand(&items.value) { ("", "") } else { ("(", ")") };
        let edits = vec![
            (Span::new(span.file, span.start, separator.start), Replacement::Text(open.to_string())),
            (separator, Replacement::Source(items.value.span)),
            (Span::new(span.file, separator.end, items.value.span.start), Replacement::Text(format!("{close}.join("))),
            (items.value.span, Replacement::Source(separator)),
        ];
        diagnostic.suggest_edits("write `xs.join(sep)`", edits)
    }

    /// `[DIA-21]` P20 — `"{} and {}".format(a, b)` is the f-string `f"{a} and {b}"` (`[LEX-19]`):
    /// each `{}` takes the next argument, a format spec after it kept.
    fn python_format(&self, receiver: &Expr, args: &[ast::Arg], span: Span, diagnostic: Diagnostic) -> Diagnostic {
        let diagnostic = diagnostic.habit(Habit::P20).note("Ember formats with f-strings, which name each value where it goes [LEX-19]");
        let help = "write the values into an f-string: `f\"{a} and {b}\"`";
        let ExprKind::Str(text) = &receiver.kind else { return diagnostic.help(help) };
        let holes = positional_holes(text);
        let literal = receiver.span;
        // The literal is written as its text in quotes (no escape changes an offset).
        let plain = literal.len() as usize == text.len() + 2;
        match holes {
            Some(holes) if plain && holes.len() == args.len() && args.iter().all(|a| a.name.is_none()) => {
                let at = |offset: usize| {
                    let byte = literal.start + 1 + offset as u32;
                    Span::new(literal.file, byte, byte)
                };
                let mut edits = vec![(literal.shrink_to_start(), Replacement::Text("f".to_string()))];
                edits.extend(holes.iter().zip(args).map(|(&hole, arg)| (at(hole), Replacement::Source(arg.value.span))));
                edits.push((Span::new(span.file, literal.end, span.end), Replacement::Text(String::new())));
                diagnostic.suggest_edits("write an f-string", edits)
            }
            _ => diagnostic.help(help),
        }
    }

    /// `[DIA-21]` P25 (ODR-187) — a list operation (`len()`, `[i]`) on an iterator, which yields its
    /// items once and has neither: `.collect[Array[T]]()` makes the list, where the iterator is made
    /// when a local holds it. `Some(None)` for another such use of a local already answered: one
    /// fix, one error (`[DIA-14]`).
    pub(crate) fn list_operation_on_iterator(&mut self, receiver: &Expr, diagnostic: Diagnostic) -> Option<Option<Diagnostic>> {
        let ty = match *self.types.kind(receiver.ty) {
            TyKind::Ref { inner, .. } => inner,
            _ => receiver.ty,
        };
        if !self.implements(ty, Symbol::intern("std.core.Iterator")) {
            return None;
        }
        let item = self.project(ty, Symbol::intern("Item"))?;
        let collect = format!(".collect[Array[{}]]()", self.types.display(item));
        if let ExprKind::Local(id) = receiver.kind
            && !self.iterators_answered.insert(id)
        {
            return Some(None);
        }
        let made_at = match receiver.kind {
            ExprKind::Local(id) if self.plain_declared.contains(&id) => self.locals.get(id.0 as usize).map(|local| local.span),
            ExprKind::Local(_) => None,
            _ => Some(receiver.span),
        };
        let diagnostic = diagnostic
            .habit(Habit::P25)
            .note("an iterator gives its items one at a time; a list of them is collected once [STD-19]");
        Some(Some(match made_at {
            Some(at) => diagnostic.suggest_edits(format!("collect the items into a list: `{collect}`"), vec![(at.shrink_to_end(), Replacement::Text(collect))]),
            None => diagnostic.help(format!("collect the items into a list where the iterator is made: `{collect}`")),
        }))
    }

    /// `[DIA-21]` P25, P29, P30 — `x[i]` on a type with no `Index`: an iterator (collect it), text
    /// (no index by character, `[TXT-4]`) or a tuple (its fields are `.0`, `.1`, …).
    pub(crate) fn index_habit(&mut self, base: &Expr, args: &[ast::TypeOrExpr], span: Span, diagnostic: Diagnostic) -> Option<Diagnostic> {
        if self.is_text(base.ty) {
            return Some(diagnostic
                .habit(Habit::P29)
                .help("for the characters, iterate `s.chars()`, or collect them once, `cs = s.chars().collect[Array[char]]()`, and index `cs[i]`; for the bytes, `s.as_bytes()[i]`")
                .note("there is no index by character position, which would be O(n); `s[a..b]` slices by byte offset [TXT-4]"));
        }
        if let TyKind::Tuple(items) = self.types.kind(base.ty) {
            let fields = items.len();
            let diagnostic = diagnostic.habit(Habit::P30).note("a tuple's fields are `.0`, `.1`, …, each with a type of its own, so a position is written, never computed (IV.3)");
            return Some(match args {
                [ast::TypeOrExpr::Expr(index)] if let ast::ExprKind::Lit(ast::Literal::Int { value, suffix: None }) = index.kind && (value as usize) < fields => {
                    let field = format!(".{value}");
                    diagnostic.suggest_edits(format!("write `{field}`"), vec![(Span::new(span.file, base.span.end, span.end), Replacement::Text(field))])
                }
                _ => diagnostic.help("name the field by its position: `p.0`, `p.1`, …"),
            });
        }
        self.list_operation_on_iterator(base, diagnostic.clone()).unwrap_or(Some(diagnostic))
    }

    /// `[DIA-21]` P20, P23, P27 — an operator Python applies to these operands and Ember does not:
    /// `"%d" % n` (formatting), `s * n` and `[x] * n` (repetition), `c == "x"` (a character against a
    /// string).
    pub(crate) fn operator_habit(&self, op: ast::BinOp, (lhs_ast, rhs_ast): (&ast::Expr, &ast::Expr), lhs: &Expr, rhs: &Expr, span: Span) -> Option<Diagnostic> {
        let shown = |ty: Ty| self.types.display(ty);
        let base = || Diagnostic::error(codes::E2020, span, format!("`{}` cannot be applied to `{}` and `{}`", op.as_str(), shown(lhs.ty), shown(rhs.ty)));
        let character = |ty: Ty| matches!(self.types.kind(ty), TyKind::Char);
        let count = |ty: Ty| self.types.is_integral(ty) || ty == self.common.int_lit;
        match op {
            ast::BinOp::Mul if count(rhs.ty) && (self.is_text(lhs.ty) || character(lhs.ty)) => {
                let diagnostic = base().habit(Habit::P23).primary_label("Python's string repetition");
                let edits = match &lhs_ast.kind {
                    ast::ExprKind::Lit(ast::Literal::Char(c)) => {
                        vec![(lhs_ast.span, Replacement::Text(format!("\"{}\"", c.escape_default()))), (Span::new(span.file, lhs_ast.span.end, rhs_ast.span.start), Replacement::Text(".repeat(".to_string())), (rhs_ast.span.shrink_to_end(), Replacement::Text(")".to_string()))]
                    }
                    _ if character(lhs.ty) => return Some(diagnostic.help("make the character text first: `c.to_string().repeat(n)` [TXT-10]")),
                    _ => {
                        let (open, close) = if binds_as_operand(lhs_ast) { ("", "") } else { ("(", ")") };
                        vec![
                            (lhs_ast.span.shrink_to_start(), Replacement::Text(open.to_string())),
                            (Span::new(span.file, lhs_ast.span.end, rhs_ast.span.start), Replacement::Text(format!("{close}.repeat("))),
                            (rhs_ast.span.shrink_to_end(), Replacement::Text(")".to_string())),
                        ]
                    }
                };
                Some(diagnostic.suggest_edits("repeat the text with `s.repeat(n)` [TXT-10]", edits))
            }
            ast::BinOp::Mul if count(rhs.ty) && matches!(self.types.kind(lhs.ty), TyKind::Vec { text: false, .. }) => {
                let diagnostic = base().habit(Habit::P23).primary_label("Python's list repetition");
                // `[x] * n`: a comprehension makes each element, cloning nothing behind the reader's back.
                if let ast::ExprKind::ArrayLit(items) = &lhs_ast.kind
                    && let [item] = items.as_slice()
                {
                    let edits = vec![
                        (Span::new(span.file, lhs_ast.span.start + 1, item.span.start), Replacement::Text(String::new())),
                        (Span::new(span.file, item.span.end, rhs_ast.span.start), Replacement::Text(" for _ in 0..".to_string())),
                        (rhs_ast.span.shrink_to_end(), Replacement::Text("]".to_string())),
                    ];
                    let edits = if binds_as_operand(rhs_ast) { edits } else {
                        let mut edits = edits;
                        edits[1].1 = Replacement::Text(" for _ in 0..(".to_string());
                        edits[2].1 = Replacement::Text(")]".to_string());
                        edits
                    };
                    return Some(diagnostic.suggest_edits("make each element with a comprehension: `[x for _ in 0..n]`", edits));
                }
                Some(diagnostic.help("make the elements with a comprehension, `[x for _ in 0..n]`, or repeat the list's items with `xs.iter().cycle().take(xs.len() * n)`"))
            }
            ast::BinOp::Rem if self.is_text(lhs.ty) => {
                let diagnostic = base().habit(Habit::P20).primary_label("Python's `%` formatting").note("Ember formats with f-strings, which name each value where it goes [LEX-19]");
                let ast::ExprKind::Lit(ast::Literal::Str(text)) = &lhs_ast.kind else { return Some(diagnostic.help("write the values into an f-string: `f\"{n} items\"`")) };
                let values: Vec<&ast::Expr> = match &rhs_ast.kind {
                    ast::ExprKind::Tuple(items) => items.iter().collect(),
                    ast::ExprKind::Paren(inner) => vec![inner],
                    _ => vec![rhs_ast],
                };
                let plain = lhs_ast.span.len() as usize == text.len() + 2;
                match percent_holes(text) {
                    Some(holes) if plain && holes.len() == values.len() => {
                        let literal = lhs_ast.span;
                        let mut edits = vec![(literal.shrink_to_start(), Replacement::Text("f".to_string()))];
                        for ((start, end, spec), value) in holes.into_iter().zip(values) {
                            // `%` becomes `{`, the conversion the value, and its spec and `}` follow.
                            let at = |offset: usize| literal.start + 1 + offset as u32;
                            edits.push((Span::new(literal.file, at(start), at(start) + 1), Replacement::Text("{".to_string())));
                            edits.push((Span::new(literal.file, at(start) + 1, at(end)), Replacement::Source(value.span)));
                            edits.push((Span::new(literal.file, at(end), at(end)), Replacement::Text(format!("{spec}}}"))));
                        }
                        edits.push((Span::new(span.file, literal.end, span.end), Replacement::Text(String::new())));
                        Some(diagnostic.suggest_edits("write an f-string", edits))
                    }
                    _ => Some(diagnostic.help("write the values into an f-string: `f\"{n} items\"`")),
                }
            }
            ast::BinOp::Eq | ast::BinOp::Ne if (character(lhs.ty) && self.is_text(rhs.ty)) || (self.is_text(lhs.ty) && character(rhs.ty)) => {
                let diagnostic = base().habit(Habit::P27).note("a `char` is one character and a `str` text; Python has only the second [TXT-10]");
                let text_side = if character(lhs.ty) { rhs_ast } else { lhs_ast };
                if let ast::ExprKind::Lit(ast::Literal::Str(text)) = &text_side.kind
                    && let mut chars = text.chars()
                    && let (Some(c), None) = (chars.next(), chars.next())
                {
                    let literal = format!("'{}'", c.escape_default());
                    return Some(diagnostic.suggest(format!("compare with the character `{literal}`"), text_side.span, literal));
                }
                let char_side = if character(lhs.ty) { lhs_ast } else { rhs_ast };
                Some(diagnostic.suggest_edits("compare the character's text: `c.to_string()`", receiver_edits(char_side.span, char_side, ".to_string()")))
            }
            _ => None,
        }
    }

    /// `[DIA-12]` N9 (D-822 D8) — the help for a type missing a bound: the derive that writes it,
    /// for a type the program declares (with the module where an `extend` may be written instead,
    /// `[TYP-20]`); for a `char` where text is the key (`[DIA-21]` P27), its text.
    pub(crate) fn bound_help(&self, diagnostic: Diagnostic, ty: Ty, bound: Symbol) -> Diagnostic {
        let (origin, args) = match self.open_interface_origin.get(&bound) {
            Some((origin, args)) => (*origin, args.clone()),
            None => (bound, Vec::new()),
        };
        let interface = origin.as_str().rsplit('.').next().unwrap_or_default().to_string();
        if matches!(self.types.kind(ty), TyKind::Char)
            && interface == "AsKey"
            && args.first().is_some_and(|&key| self.is_text(key))
        {
            return diagnostic
                .habit(Habit::P27)
                .help("look the key up by its text: `c.to_string()`")
                .note("a `char` is one character and the keys are text; Python has only text [TXT-10]");
        }
        const DERIVABLE: [&str; 6] = ["Eq", "Hash", "Ord", "Clone", "Debug", "Default"];
        // A struct the program declares (not a generic's instance, not the standard library's).
        let declared = match *self.types.kind(ty) {
            TyKind::Struct(id) => {
                let def = self.types.struct_def(id);
                def.origin.is_none() && self.prefixes.get(def.declaring_module).is_some_and(|prefix| prefix != "std" && !prefix.starts_with("std."))
            }
            _ => false,
        };
        let shown = self.types.display(ty);
        if declared && DERIVABLE.contains(&interface.as_str()) {
            return diagnostic
                .help(format!("derive it, `@derive({interface})` on `{shown}`, or write `extend {shown} implements {interface}` in `{shown}`'s module [TYP-20]"));
        }
        diagnostic
    }

    /// `[DIA-21]` P26 — `s += t` on a `str` local, Python's way of building text: a `str` is a view
    /// that does not grow, and text grows in a `String`; the local is declared one, which a literal
    /// initialises (`[TXT-9]`).
    pub(crate) fn growing_a_str(&self, place: &Expr, diagnostic: Diagnostic) -> Diagnostic {
        let diagnostic = diagnostic.habit(Habit::P26).note("a `str` is borrowed text and does not grow; `+=` appends to a `String` [TXT-11]");
        if let ExprKind::Local(id) = place.kind
            && self.plain_declared.contains(&id)
            && let Some(local) = self.locals.get(id.0 as usize)
            && let Some(name) = local.name
        {
            let at = local.span.start + name.as_str().len() as u32;
            return diagnostic.suggest_edits(format!("declare it a `String`: `{name}: String = …`"), vec![(Span::new(local.span.file, at, at), Replacement::Text(": String".to_string()))]);
        }
        diagnostic.help("build the text in a `String`: declare the place `String`")
    }

    /// `[DIA-21]` P22 (ODR-182) — an interface's name in a type position is a class handle typed by
    /// it (IV.1, `[OBJ-2]`); when the types implementing it are values (structs, enums) and none is a
    /// class, a place of any of them is written another way, which the help names.
    pub(crate) fn interface_of_values(&self, interface: Symbol, span: Span) -> Option<Diagnostic> {
        let mut implementers = self.implemented.iter().filter(|(_, each, _)| *each == interface).map(|&(ty, ..)| ty).peekable();
        implementers.peek()?;
        let mut implementers = implementers;
        if implementers.any(|ty| matches!(self.types.kind(ty), TyKind::Class(_))) {
            return None;
        }
        Some(self.interface_of_values_diagnostic(interface, Diagnostic::error(codes::E2173, span, format!("`{}` is an interface, and the types implementing it are values, not classes", self.interface_shown(interface)))))
    }

    fn interface_of_values_diagnostic(&self, interface: Symbol, diagnostic: Diagnostic) -> Diagnostic {
        let shown = self.interface_shown(interface);
        diagnostic
            .habit(Habit::P22)
            .help(format!("hold such a value behind a pointer: `Box[dyn {shown}]` in a field or a collection (`Box(v)` makes one), `ref dyn {shown}` for a parameter; or take a type parameter, `[T: {shown}]`, and pass the value itself"))
            .note(format!("an interface's name as a type is a class handle typed by it [OBJ-2]; a value of a type implementing `{shown}` has its own size, which `dyn {shown}` keeps behind a pointer (ODR-182)"))
    }

    /// `[DIA-21]` P21, P26, P27 and `[DIA-12]` N4 — what a value of `expr.ty` where `expected` is
    /// wanted means in Python's terms: a `str` kept as a `String` (`to_string()`, `[TXT-9]`), a
    /// `char` where text is wanted, a message where an `AnyError` is wanted (`AnyError.msg`,
    /// ODR-180), a number of another type (the `as` that converts it).
    pub(crate) fn mismatch_habit(&self, expr: &Expr, expected: Ty, diagnostic: Diagnostic) -> Diagnostic {
        let found = expr.ty;
        let string = matches!(self.types.kind(expected), TyKind::Vec { text: true, .. });
        let operand = checked_binds_as_operand(expr);
        if string && found == self.common.str_ {
            return diagnostic
                .habit(Habit::P26)
                .note("a `str` value is borrowed text; a `String` owns its text, so keeping one copies it [TXT-9]")
                .suggest_edits("make a `String` of it: `.to_string()`", suffix_edits(expr.span, operand, ".to_string()"));
        }
        if (string || expected == self.common.str_) && matches!(self.types.kind(found), TyKind::Char) {
            return diagnostic
                .habit(Habit::P27)
                .note("a `char` is one character and text a `str` or a `String`; Python has only text [TXT-10]")
                .suggest_edits("make text of the character: `.to_string()`", suffix_edits(expr.span, operand, ".to_string()"));
        }
        if matches!(self.types.kind(expected), TyKind::Char)
            && let ExprKind::Str(text) = &expr.kind
            && let mut chars = text.chars()
            && let (Some(c), None) = (chars.next(), chars.next())
        {
            let literal = format!("'{}'", c.escape_default());
            return diagnostic
                .habit(Habit::P27)
                .note("a `char` is one character, written in single quotes; Python has only text [TXT-10]")
                .suggest(format!("write the character `{literal}`"), expr.span, literal);
        }
        if Some(&expected) == self.named_types.get(&Symbol::intern("std.core.AnyError")) && self.is_text(found) {
            let made = matches!(expr.kind, ExprKind::Str(_)) || found != self.common.str_;
            let close = if made { ")" } else { ".to_string())" };
            let edits = vec![(expr.span.shrink_to_start(), Replacement::Text("AnyError.msg(".to_string())), (expr.span.shrink_to_end(), Replacement::Text(close.to_string()))];
            return diagnostic
                .habit(Habit::P21)
                .note("a function returning `Result[T]` fails with an `AnyError`, which `AnyError.msg` makes of a message (ODR-180) [ERR-9]")
                .suggest_edits("make the message an error: `AnyError.msg(…)`", edits);
        }
        if let TyKind::ClassInterface(interface) = *self.types.kind(expected)
            && matches!(self.types.kind(found), TyKind::Struct(_) | TyKind::Enum(_))
        {
            return self.interface_of_values_diagnostic(interface, diagnostic);
        }
        let number = |ty: Ty| self.types.is_numeric(ty);
        if number(found) && number(expected) && !(self.whole(found) && self.whole(expected)) {
            let wanted = self.types.display(expected);
            let what = match (self.types.is_float(found), self.types.is_float(expected)) {
                (true, true) => format!("rounds it to the nearest `{wanted}`"),
                (true, false) => "rounds it toward zero, saturating".to_string(),
                _ => format!("rounds it to the nearest `{wanted}` where it has more digits than `{wanted}` keeps"),
            };
            return diagnostic.suggest_edits(format!("convert it: `as {wanted}` {what} [TYP-6]"), suffix_edits(expr.span, operand, &format!(" as {wanted}")));
        }
        diagnostic
    }
}

/// The offsets of `{}`/`{:spec}` holes in a `str.format` text, each just after its `{`; `None` when a
/// hole names an argument (`{0}`, `{name}`), which the help leaves to the reader.
fn positional_holes(text: &str) -> Option<Vec<usize>> {
    let bytes = text.as_bytes();
    let (mut at, mut holes) = (0, Vec::new());
    while at < bytes.len() {
        match bytes[at] {
            b'{' if bytes.get(at + 1) == Some(&b'{') => at += 2,
            b'}' if bytes.get(at + 1) == Some(&b'}') => at += 2,
            b'{' => {
                if !matches!(bytes.get(at + 1), Some(b'}' | b':')) {
                    return None;
                }
                holes.push(at + 1);
                at += 1;
            }
            _ => at += 1,
        }
    }
    Some(holes)
}

/// The `%` conversions of a `"%d" % n` text: each one's byte range and the format spec it becomes
/// (`%5.2f` is `:5.2f`, `%d` and `%s` none, `%r` is `!r`); `None` for one the help cannot say
/// (`%(name)s`, `%*d`, `%c`).
fn percent_holes(text: &str) -> Option<Vec<(usize, usize, String)>> {
    let bytes = text.as_bytes();
    let (mut at, mut holes) = (0, Vec::new());
    while at < bytes.len() {
        if bytes[at] != b'%' {
            at += 1;
            continue;
        }
        if bytes.get(at + 1) == Some(&b'%') {
            return None;
        }
        let start = at;
        at += 1;
        let flags_from = at;
        while at < bytes.len() && (bytes[at].is_ascii_digit() || matches!(bytes[at], b'.' | b'-' | b'+' | b' ' | b'#')) {
            at += 1;
        }
        let flags = &text[flags_from..at];
        let spec = match bytes.get(at)? {
            b'd' | b'i' | b's' | b'u' if flags.is_empty() => String::new(),
            b'd' | b'i' | b'u' => format!(":{}", flags.replace('-', "<")),
            b's' => format!(":{}", flags.replace('-', "<")),
            b'f' | b'F' | b'e' | b'E' | b'g' | b'G' | b'x' | b'X' | b'o' => format!(":{}{}", flags.replace('-', "<"), bytes[at] as char),
            b'r' if flags.is_empty() => "!r".to_string(),
            _ => return None,
        };
        at += 1;
        holes.push((start, at, spec));
    }
    Some(holes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_holes_are_found_after_each_brace() {
        assert_eq!(positional_holes("{} and {:>4}"), Some(vec![1, 8]));
        assert_eq!(positional_holes("{{literal}} {}"), Some(vec![13]));
        assert_eq!(positional_holes("{0}"), None);
    }

    #[test]
    fn percent_conversions_become_format_specs() {
        assert_eq!(percent_holes("%d items"), Some(vec![(0, 2, String::new())]));
        assert_eq!(percent_holes("%5.2f|%s|%r"), Some(vec![(0, 5, ":5.2f".to_string()), (6, 8, String::new()), (9, 11, "!r".to_string())]));
        assert_eq!(percent_holes("100%%"), None);
    }
}
