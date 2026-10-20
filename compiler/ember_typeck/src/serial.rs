//! `[SER-1]`, `[SER-2]`, `[RNG-10a]` — `@derive(Serialize)` and `@derive(Deserialize)` on a struct or
//! an enum, written out before anything is checked as the `extend` blocks they stand for (as
//! `derives.rs` writes `Ord` and `Default`). A struct is written as its version and its fields by
//! name; an enum value as its variant's name and its payload's fields. Reading takes the fields in
//! any order, skips one it does not know, gives a missing one its default under `@ser(default)` or
//! `@ser(since=N)` (`MissingField` otherwise), names an unknown variant `UnknownVariant`, and checks
//! a range-typed field with `checked` (`OutOfRange`). Field attributes: `@ser(skip)`,
//! `@ser(rename="…")`, `@ser(default)`, `@ser(since=N)`.

use super::*;
use crate::errors::{call, expr, extend_item, fn_decl, member, method_call, named_type, owner_type, path, ret, stmt};

/// How one field is written and read.
struct SerField {
    /// Its name in the type (`_0` for an unnamed payload field).
    name: Symbol,
    /// Its name in the data.
    label: String,
    ty: ast::TypeExpr,
    skip: bool,
    /// Missing in the data, it takes its type's default.
    defaulted: bool,
    since: i64,
    /// `[RNG-10a]` — the representation of the range type it is, if it is one.
    range: Option<ast::TypeExpr>,
}

/// The `@ser(…)` attribute of a field, read; an argument it does not have is `E0104`.
fn ser_options(attrs: &[ast::Attribute], sink: &mut Sink) -> (Option<String>, bool, bool, i64) {
    let (mut rename, mut skip, mut defaulted, mut since) = (None, false, false, 0);
    for attr in attrs.iter().filter(|attr| attr.path.len() == 1 && attr.path[0].name.is("ser")) {
        for arg in &attr.args {
            match arg {
                ast::AttrArg::Expr(ast::Expr { kind: ast::ExprKind::Path { segments }, .. }) if segments.len() == 1 && segments[0].name.is("skip") => skip = true,
                ast::AttrArg::Expr(ast::Expr { kind: ast::ExprKind::Path { segments }, .. }) if segments.len() == 1 && segments[0].name.is("default") => defaulted = true,
                ast::AttrArg::Named { name, value: ast::Expr { kind: ast::ExprKind::Lit(ast::Literal::Str(text)), .. } } if name.name.is("rename") => rename = Some(text.clone()),
                ast::AttrArg::Named { name, value: ast::Expr { kind: ast::ExprKind::Lit(ast::Literal::Int { value, .. }), .. } } if name.name.is("since") => {
                    since = i64::try_from(*value).unwrap_or(i64::MAX);
                }
                _ => sink.emit(
                    Diagnostic::error(codes::E0104, attr.span, "`@ser` takes `skip`, `default`, `rename=\"…\"` and `since=N`")
                        .note("a field's serialisation is written as `[SER-1]` lists"),
                ),
            }
        }
    }
    (rename, skip, defaulted, since)
}

/// `[RNG-10a]` — the range types the program declares, by name, with their representations.
pub(crate) fn range_types(modules: &[LoadedModule]) -> HashMap<Symbol, ast::TypeExpr> {
    let mut ranges = HashMap::new();
    for loaded in modules {
        for item in &loaded.module.items {
            if let ast::ItemKind::TypeAlias(alias) = &item.kind
                && alias.range.is_some()
                && let Some(value) = &alias.value
            {
                ranges.insert(alias.name.name, value.clone());
            }
        }
    }
    ranges
}

fn range_of(ty: &ast::TypeExpr, ranges: &HashMap<Symbol, ast::TypeExpr>) -> Option<ast::TypeExpr> {
    match &ty.kind {
        ast::TypeKind::Path { segments, args } if args.is_empty() => ranges.get(&segments.last()?.name).cloned(),
        _ => None,
    }
}

fn struct_fields(decl: &ast::StructDecl, ranges: &HashMap<Symbol, ast::TypeExpr>, sink: &mut Sink) -> Vec<SerField> {
    decl.members
        .iter()
        .filter_map(|member| match &member.kind {
            ast::MemberKind::Field(field) => {
                let (rename, skip, defaulted, since) = ser_options(&member.attrs, sink);
                Some(SerField {
                    name: field.name.name,
                    label: rename.unwrap_or_else(|| field.name.name.to_string()),
                    ty: field.ty.clone(),
                    skip,
                    defaulted,
                    since,
                    range: range_of(&field.ty, ranges),
                })
            }
            _ => None,
        })
        .collect()
}

fn payload_fields(variant: &ast::Variant, ranges: &HashMap<Symbol, ast::TypeExpr>) -> Vec<SerField> {
    variant
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| SerField {
            name: field.name.map_or_else(|| Symbol::intern(&format!("_{index}")), |name| name.name),
            label: field.name.map_or_else(|| index.to_string(), |name| name.name.to_string()),
            ty: field.ty.clone(),
            skip: false,
            defaulted: false,
            since: 0,
            range: range_of(&field.ty, ranges),
        })
        .collect()
}

fn sym(name: &str) -> Symbol {
    Symbol::intern(name)
}

fn ident(name: &str, span: Span) -> ast::Ident {
    ast::Ident { name: sym(name), span }
}

fn text(value: &str, span: Span) -> ast::Expr {
    expr(ast::ExprKind::Lit(ast::Literal::Str(value.to_string())), span)
}

fn int(value: i64, span: Span) -> ast::Expr {
    expr(ast::ExprKind::Lit(ast::Literal::Int { value: value.unsigned_abs() as u128, suffix: None }), span)
}

fn try_(value: ast::Expr, span: Span) -> ast::Expr {
    expr(ast::ExprKind::Try(Box::new(value)), span)
}

fn expr_stmt(value: ast::Expr, span: Span) -> ast::Stmt {
    stmt(ast::StmtKind::Expr(value), span)
}

fn local(name: &str, span: Span) -> ast::Expr {
    path(&[sym(name)], span)
}

fn option_of(ty: ast::TypeExpr, span: Span) -> ast::TypeExpr {
    ast::TypeExpr {
        id: ast::NodeId::DUMMY,
        kind: ast::TypeKind::Path { segments: vec![ident("Option", span)], args: vec![ast::GenericArg::Type(ty)] },
        span,
    }
}

fn result_of(ok: ast::TypeExpr, span: Span) -> ast::TypeExpr {
    ast::TypeExpr {
        id: ast::NodeId::DUMMY,
        kind: ast::TypeKind::Path {
            segments: vec![ident("Result", span)],
            args: vec![ast::GenericArg::Type(ok), ast::GenericArg::Type(named_type("std.ser.SerError", span))],
        },
        span,
    }
}

fn void_type(span: Span) -> ast::TypeExpr {
    ast::TypeExpr { id: ast::NodeId::DUMMY, kind: ast::TypeKind::Void, span }
}

fn generic(name: &str, bound: &str, span: Span) -> ast::GenericParam {
    ast::GenericParam { id: ast::NodeId::DUMMY, name: ident(name, span), bounds: vec![named_type(bound, span)], default: None, const_ty: None, span }
}

/// `std.ser.put(w, "label", value)?`; a range-typed value through a local of its representation,
/// which it converts to as it is stored (`[RNG-2]`).
fn put_field(field: &SerField, value: ast::Expr, span: Span) -> Vec<ast::Stmt> {
    let mut stmts = Vec::new();
    let value = match &field.range {
        Some(repr) => {
            let name = format!("{}·repr", field.name);
            let pattern = crate::errors::pattern_of(ast::PatternKind::Bind { name: ident(&name, span), by_ref: false, mutable: false, sub: None }, span);
            stmts.push(stmt(ast::StmtKind::Decl { pattern, ty: Some(repr.clone()), init: Some(value) }, span));
            local(&name, span)
        }
        None => value,
    };
    stmts.push(expr_stmt(try_(call(path(&[sym("std.ser.put")], span), vec![local("w", span), text(&field.label, span), value], span), span), span));
    stmts
}

/// `fn serialize[W: std.ser.Writer](self, mut w: W) -> Result[void, std.ser.SerError]: body`.
fn serialize_member(body: Vec<ast::Stmt>, span: Span) -> ast::Member {
    let mut decl = ast::FnDecl { name: ident("serialize", span), ..fn_decl(span) };
    decl.generics = vec![generic("W", "std.ser.Writer", span)];
    decl.params = vec![crate::errors::receiver(span), crate::errors::named_param(sym("w"), ast::Mode::Mut, named_type("W", span), span)];
    decl.ret = Some(result_of(void_type(span), span));
    member(decl, crate::errors::block(body, span), span)
}

/// `fn deserialize[R: std.ser.Reader](mut r: R) -> Result[Self, std.ser.SerError]: body`.
fn deserialize_member(target: &ast::TypeExpr, body: Vec<ast::Stmt>, span: Span) -> ast::Member {
    let mut decl = ast::FnDecl { name: ident("deserialize", span), ..fn_decl(span) };
    decl.generics = vec![generic("R", "std.ser.Reader", span)];
    decl.params = vec![crate::errors::named_param(sym("r"), ast::Mode::Mut, named_type("R", span), span)];
    decl.ret = Some(result_of(target.clone(), span));
    member(decl, crate::errors::block(body, span), span)
}

/// The slots a reader fills, the loop that fills them from the fields in the data, and each
/// field's value afterwards (`None` for one written from nothing: a skipped field's default).
fn read_fields(fields: &[SerField], span: Span) -> (Vec<ast::Stmt>, Vec<ast::Expr>) {
    let mut stmts = Vec::new();
    let slot = |field: &SerField| format!("{}·slot", field.name);
    for field in fields.iter().filter(|field| !field.skip) {
        let ty = option_of(field.range.clone().unwrap_or_else(|| field.ty.clone()), span);
        let pattern = crate::errors::pattern_of(ast::PatternKind::Bind { name: ident(&slot(field), span), by_ref: false, mutable: false, sub: None }, span);
        stmts.push(stmt(ast::StmtKind::Decl { pattern, ty: Some(ty), init: Some(local("None", span)) }, span));
    }
    // `while Some(name·) = r.next_field()?:` and, by name, each slot taken or the value skipped.
    let skip = expr_stmt(try_(method_call(local("r", span), ident("skip_value", span), Vec::new(), span), span), span);
    let mut chain: Option<ast::IfStmt> = None;
    for field in fields.iter().filter(|field| !field.skip).rev() {
        let cond = expr(
            ast::ExprKind::Binary { op: ast::BinOp::Eq, lhs: Box::new(local("name·", span)), rhs: Box::new(text(&field.label, span)) },
            span,
        );
        let take = expr_stmt(try_(call(path(&[sym("std.ser.take")], span), vec![local("r", span), local(&slot(field), span)], span), span), span);
        let else_block = Some(Box::new(match chain.take() {
            Some(next) => ast::ElseBranch::If(next),
            None => ast::ElseBranch::Block(crate::errors::block(vec![skip.clone()], span)),
        }));
        chain = Some(ast::IfStmt { cond: ast::Condition::Expr(cond), then_block: crate::errors::block(vec![take], span), else_block });
    }
    let named = chain.is_some();
    let body = match chain {
        Some(chain) => vec![stmt(ast::StmtKind::If(chain), span)],
        None => vec![skip],
    };
    // With no field to read, every name is skipped unread.
    let name_pattern = match named {
        true => ast::PatternKind::Bind { name: ident("name·", span), by_ref: false, mutable: false, sub: None },
        false => ast::PatternKind::Wild,
    };
    let some_name = crate::errors::pattern_of(
        ast::PatternKind::Constructor {
            path: vec![ident("Some", span)],
            fields: vec![ast::FieldPattern {
                name: None,
                pattern: crate::errors::pattern_of(name_pattern, span),
                span,
            }],
            has_rest: false,
        },
        span,
    );
    let next = try_(method_call(local("r", span), ident("next_field", span), Vec::new(), span), span);
    stmts.push(stmt(
        ast::StmtKind::While {
            label: None,
            cond: ast::Condition::Pattern { pattern: some_name, value: next },
            body: crate::errors::block(body, span),
            else_block: None,
        },
        span,
    ));
    let values = fields
        .iter()
        .map(|field| {
            if field.skip {
                // `std.core.default_of[T]()`.
                let default_of = expr(
                    ast::ExprKind::IndexOrInstantiate { base: Box::new(path(&[sym("std.core.default_of")], span)), args: vec![ast::TypeOrExpr::Type(field.ty.clone())] },
                    span,
                );
                return call(default_of, Vec::new(), span);
            }
            let slot_value = local(&slot(field), span);
            let raw = if field.defaulted || field.since > 0 {
                call(path(&[sym("std.ser.or_default")], span), vec![slot_value], span)
            } else {
                try_(call(path(&[sym("std.ser.required")], span), vec![slot_value, text(&field.label, span)], span), span)
            };
            match &field.ty.kind {
                // `[RNG-10a]` — `std.ser.in_range(T.checked(raw), "label")?`.
                ast::TypeKind::Path { segments, .. } if field.range.is_some() => {
                    let checked = method_call(path(&segments.iter().map(|segment| segment.name).collect::<Vec<_>>(), span), ident("checked", span), vec![raw], span);
                    try_(call(path(&[sym("std.ser.in_range")], span), vec![checked, text(&field.label, span)], span), span)
                }
                _ => raw,
            }
        })
        .collect();
    (stmts, values)
}

/// `Ok(value)` returned.
fn return_ok(value: ast::Expr, span: Span) -> ast::Stmt {
    ret(Some(call(local("Ok", span), vec![value], span)), span)
}

fn struct_serialize(decl: &ast::StructDecl, fields: &[SerField], span: Span) -> ast::Item {
    let version = fields.iter().map(|field| field.since).max().unwrap_or(0);
    let mut body = vec![expr_stmt(
        try_(method_call(local("w", span), ident("begin_struct", span), vec![text(&decl.name.name.to_string(), span), int(version, span)], span), span),
        span,
    )];
    for field in fields.iter().filter(|field| !field.skip) {
        let value = expr(ast::ExprKind::Field { base: Box::new(crate::errors::self_expr(span)), name: ast::Ident { name: field.name, span } }, span);
        body.extend(put_field(field, value, span));
    }
    body.push(ret(Some(method_call(local("w", span), ident("end_struct", span), Vec::new(), span)), span));
    let target = owner_type(decl.name, &decl.generics);
    extend_item(&bounded_by(&decl.generics, "std.ser.Serialize", span), target, vec![named_type("std.ser.Serialize", span)], vec![serialize_member(body, span)], span)
}

fn struct_deserialize(decl: &ast::StructDecl, fields: &[SerField], span: Span) -> ast::Item {
    let target = owner_type(decl.name, &decl.generics);
    let mut body = vec![expr_stmt(
        try_(method_call(local("r", span), ident("begin_struct", span), Vec::new(), span), span),
        span,
    )];
    let (slots, values) = read_fields(fields, span);
    body.extend(slots);
    body.push(expr_stmt(try_(method_call(local("r", span), ident("end_struct", span), Vec::new(), span), span), span));
    let args = fields.iter().zip(values).map(|(field, value)| ast::Arg { name: Some(ast::Ident { name: field.name, span }), value, span }).collect();
    let made = expr(ast::ExprKind::Call { callee: Box::new(local("Self", span)), args }, span);
    body.push(return_ok(made, span));
    extend_item(&bounded_by(&decl.generics, "std.ser.Deserialize", span), target.clone(), vec![named_type("std.ser.Deserialize", span)], vec![deserialize_member(&target, body, span)], span)
}

fn enum_serialize(decl: &ast::EnumDecl, ranges: &HashMap<Symbol, ast::TypeExpr>, span: Span) -> ast::Item {
    let mut arms = Vec::new();
    for variant in &decl.variants {
        let fields = payload_fields(variant, ranges);
        let bindings: Vec<Symbol> = fields.iter().map(|field| Symbol::intern(&format!("{}·value", field.name))).collect();
        let pattern = variant_bound(decl.name, variant, &bindings, span);
        let mut body = vec![expr_stmt(
            try_(method_call(local("w", span), ident("begin_variant", span), vec![text(&variant.name.name.to_string(), span)], span), span),
            span,
        )];
        for (field, binding) in fields.iter().zip(&bindings) {
            body.extend(put_field(field, path(&[*binding], span), span));
        }
        body.push(ret(Some(method_call(local("w", span), ident("end_variant", span), Vec::new(), span)), span));
        arms.push(ast::MatchArm { id: ast::NodeId::DUMMY, pattern, guard: None, body: ast::MatchArmBody::Block(crate::errors::block(body, span)), span });
    }
    let body = vec![stmt(ast::StmtKind::Match { scrutinee: crate::errors::self_expr(span), arms }, span)];
    let target = owner_type(decl.name, &decl.generics);
    extend_item(&bounded_by(&decl.generics, "std.ser.Serialize", span), target, vec![named_type("std.ser.Serialize", span)], vec![serialize_member(body, span)], span)
}

fn enum_deserialize(decl: &ast::EnumDecl, ranges: &HashMap<Symbol, ast::TypeExpr>, span: Span) -> ast::Item {
    let target = owner_type(decl.name, &decl.generics);
    let tag = crate::errors::pattern_of(ast::PatternKind::Bind { name: ident("tag·", span), by_ref: false, mutable: false, sub: None }, span);
    let mut body = vec![stmt(
        ast::StmtKind::Decl { pattern: tag, ty: None, init: Some(try_(method_call(local("r", span), ident("begin_variant", span), Vec::new(), span), span)) },
        span,
    )];
    for variant in &decl.variants {
        let fields = payload_fields(variant, ranges);
        let (slots, values) = read_fields(&fields, span);
        let mut then = slots;
        then.push(expr_stmt(try_(method_call(local("r", span), ident("end_variant", span), Vec::new(), span), span), span));
        let made = expr(ast::ExprKind::Field { base: Box::new(path(&[decl.name.name], span)), name: variant.name }, span);
        let made = match values.is_empty() {
            true => made,
            false => {
                let args = fields
                    .iter()
                    .zip(values)
                    .zip(&variant.fields)
                    .map(|((field, value), written)| ast::Arg { name: written.name.map(|_| ast::Ident { name: field.name, span }), value, span })
                    .collect();
                expr(ast::ExprKind::Call { callee: Box::new(made), args }, span)
            }
        };
        then.push(return_ok(made, span));
        let cond = expr(
            ast::ExprKind::Binary { op: ast::BinOp::Eq, lhs: Box::new(local("tag·", span)), rhs: Box::new(text(&variant.name.name.to_string(), span)) },
            span,
        );
        body.push(stmt(ast::StmtKind::If(ast::IfStmt { cond: ast::Condition::Expr(cond), then_block: crate::errors::block(then, span), else_block: None }), span));
    }
    // `[SER-2]` — a variant the enum does not have.
    let variant = expr(ast::ExprKind::Field { base: Box::new(path(&[sym("std.ser.SerError")], span)), name: ident("UnknownVariant", span) }, span);
    let unknown = call(variant, vec![local("tag·", span)], span);
    body.push(ret(Some(call(local("Err", span), vec![unknown], span)), span));
    extend_item(&bounded_by(&decl.generics, "std.ser.Deserialize", span), target.clone(), vec![named_type("std.ser.Deserialize", span)], vec![deserialize_member(&target, body, span)], span)
}

/// `E.V(a, b)` binding each payload field by name, or `E.V` for a unit variant.
fn variant_bound(enum_name: ast::Ident, variant: &ast::Variant, bindings: &[Symbol], span: Span) -> ast::Pattern {
    let segments = vec![ast::Ident { name: enum_name.name, span }, ast::Ident { name: variant.name.name, span }];
    if variant.fields.is_empty() {
        return crate::errors::pattern_of(ast::PatternKind::Path { segments }, span);
    }
    let fields = variant
        .fields
        .iter()
        .zip(bindings)
        .map(|(field, &binding)| ast::FieldPattern {
            name: field.name,
            pattern: crate::errors::pattern_of(ast::PatternKind::Bind { name: ast::Ident { name: binding, span }, by_ref: true, mutable: false, sub: None }, span),
            span,
        })
        .collect();
    crate::errors::pattern_of(ast::PatternKind::Constructor { path: segments, fields, has_rest: false }, span)
}

/// The declaration's parameters, each type parameter bounded by `bound` as well.
fn bounded_by(generics: &[ast::GenericParam], bound: &str, span: Span) -> Vec<ast::GenericParam> {
    generics
        .iter()
        .cloned()
        .map(|mut param| {
            if param.const_ty.is_none() {
                param.bounds.push(named_type(bound, span));
            }
            param
        })
        .collect()
}

/// `[SER-1]` — the `extend` blocks a struct's or an enum's `@derive(Serialize)` and
/// `@derive(Deserialize)` stand for, the derives removed from the attribute.
pub(crate) fn serialisation(item: &mut ast::Item, ranges: &HashMap<Symbol, ast::TypeExpr>, sink: &mut Sink) -> Vec<ast::Item> {
    let wants = |item: &ast::Item, name: &str| {
        item.attrs.iter().any(|attr| {
            attr.path.len() == 1
                && attr.path[0].name.is("derive")
                && attr.args.iter().any(|arg| matches!(arg, ast::AttrArg::Expr(ast::Expr { kind: ast::ExprKind::Path { segments }, .. }) if segments.len() == 1 && segments[0].name.is(name)))
        })
    };
    let (serialize, deserialize) = (wants(item, "Serialize"), wants(item, "Deserialize"));
    if !serialize && !deserialize {
        return Vec::new();
    }
    crate::errors::remove_attribute_argument(&mut item.attrs, "derive", "Serialize");
    crate::errors::remove_attribute_argument(&mut item.attrs, "derive", "Deserialize");
    let span = item.span;
    let mut out = Vec::new();
    match &item.kind {
        ast::ItemKind::Struct(decl) => {
            let fields = struct_fields(decl, ranges, sink);
            if serialize {
                out.push(struct_serialize(decl, &fields, span));
            }
            if deserialize {
                out.push(struct_deserialize(decl, &fields, span));
            }
        }
        ast::ItemKind::Enum(decl) => {
            if serialize {
                out.push(enum_serialize(decl, ranges, span));
            }
            if deserialize {
                out.push(enum_deserialize(decl, ranges, span));
            }
        }
        _ => {}
    }
    out
}

/// `[SER-1]` — every struct's and enum's serialisation derives, written out.
pub(crate) fn desugar(modules: &[LoadedModule], sink: &mut Sink) -> Option<Vec<LoadedModule>> {
    let wanted = |item: &ast::Item| {
        matches!(item.kind, ast::ItemKind::Struct(_) | ast::ItemKind::Enum(_))
            && item.attrs.iter().any(|attr| {
                attr.path.len() == 1
                    && attr.path[0].name.is("derive")
                    && attr.args.iter().any(|arg| matches!(arg, ast::AttrArg::Expr(ast::Expr { kind: ast::ExprKind::Path { segments }, .. })
                        if segments.len() == 1 && (segments[0].name.is("Serialize") || segments[0].name.is("Deserialize"))))
            })
    };
    if !modules.iter().any(|loaded| loaded.module.items.iter().any(wanted)) {
        return None;
    }
    let ranges = range_types(modules);
    let mut out = modules.to_vec();
    for loaded in &mut out {
        let mut generated = Vec::new();
        for item in &mut loaded.module.items {
            if wanted(item) {
                generated.extend(serialisation(item, &ranges, sink));
            }
        }
        loaded.module.items.extend(generated);
    }
    Some(out)
}
