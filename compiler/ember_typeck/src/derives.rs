//! `[DRV-1]` — `@derive(Ord)`, `@derive(Default)` and `@derive(Display)` on a struct or an enum,
//! written out before anything is checked as the `extend` blocks they stand for, so they are
//! checked, lowered and borrow-checked as written ones are (as `@derive(Error)` is, `errors.rs`).
//! What each derive needs of the fields, `@derive(Eq)` and `@derive(Debug)` included, is checked
//! once implementations are known (`Checker::check_derived_requirements`): a field that falls short
//! is `E2080`, naming it, and the block written for that derive is not checked.

use super::*;
use crate::errors::{
    arm, block, call, expr, extend_item, fmt_member, fn_decl, member, method_call, named_type, owner_type, path, pattern_of, remove_attribute_argument, ret,
    self_expr, stmt, write_str,
};

/// A derive whose requirement on the fields is checked once implementations are known.
pub(crate) struct DeriveRequest {
    pub module: usize,
    pub name: Symbol,
    pub derive: &'static str,
    /// The interface each field must implement.
    pub requires: &'static str,
    /// Fields that need nothing (for `Default`, those with a declared default).
    pub exempt: Vec<Symbol>,
    /// The derive's name where it is written: the block written for it has this span.
    pub span: Span,
}

const WRITTEN: [&str; 3] = ["Ord", "Default", "Display"];
const CHECKED: [&str; 2] = ["Eq", "Debug"];

pub(crate) fn desugar_derives(modules: &[LoadedModule], sink: &mut Sink) -> (Option<Vec<LoadedModule>>, Vec<DeriveRequest>) {
    let mut requests = Vec::new();
    let named = |item: &ast::Item| matches!(item.kind, ast::ItemKind::Struct(_) | ast::ItemKind::Enum(_));
    let any = modules.iter().any(|loaded| {
        loaded.module.items.iter().any(|item| named(item) && WRITTEN.iter().chain(&CHECKED).any(|derive| derive_span(&item.attrs, derive).is_some()))
    });
    if !any {
        return (None, requests);
    }
    let mut out = modules.to_vec();
    for (module, loaded) in out.iter_mut().enumerate() {
        let mut generated = Vec::new();
        for item in &mut loaded.module.items {
            if !named(item) {
                continue;
            }
            for derive in CHECKED {
                if let Some(span) = derive_span(&item.attrs, derive) {
                    let name = item_name(item);
                    requests.push(DeriveRequest { module, name, derive, requires: interface(derive), exempt: Vec::new(), span });
                }
            }
            for derive in WRITTEN {
                let Some(span) = derive_span(&item.attrs, derive) else { continue };
                remove_attribute_argument(&mut item.attrs, "derive", derive);
                let name = item_name(item);
                let (block, requires, exempt) = match (&mut item.kind, derive) {
                    (ast::ItemKind::Struct(decl), "Ord") => (Some(struct_ord(decl, span)), Some("Ord"), Vec::new()),
                    (ast::ItemKind::Enum(decl), "Ord") => (Some(enum_ord(decl, span)), Some("Ord"), Vec::new()),
                    (ast::ItemKind::Struct(decl), "Default") => {
                        let exempt = fields(decl).filter(|field| field.default.is_some()).map(|field| field.name.name).collect();
                        (Some(struct_default(decl, span)), Some("Default"), exempt)
                    }
                    (ast::ItemKind::Enum(decl), "Default") => (enum_default(decl, span, sink), None, Vec::new()),
                    (ast::ItemKind::Enum(decl), "Display") if decl.variants.iter().all(|variant| variant.fields.is_empty()) => {
                        (Some(unit_display(decl, span)), None, Vec::new())
                    }
                    (ast::ItemKind::Struct(decl), "Display") => (Some(debug_display(decl.name, &decl.generics, span)), Some("Debug"), Vec::new()),
                    (ast::ItemKind::Enum(decl), "Display") => (Some(debug_display(decl.name, &decl.generics, span)), Some("Debug"), Vec::new()),
                    _ => (None, None, Vec::new()),
                };
                generated.extend(block);
                if let Some(requires) = requires {
                    requests.push(DeriveRequest { module, name, derive, requires: interface(requires), exempt, span });
                }
            }
        }
        loaded.module.items.extend(generated);
    }
    (Some(out), requests)
}

fn item_name(item: &ast::Item) -> Symbol {
    match &item.kind {
        ast::ItemKind::Struct(decl) => decl.name.name,
        ast::ItemKind::Enum(decl) => decl.name.name,
        _ => unreachable!("only structs and enums derive here"),
    }
}

fn interface(derive: &str) -> &'static str {
    match derive {
        "Ord" => "std.core.Ord",
        "Default" => "std.core.Default",
        "Eq" => "std.core.Eq",
        _ => "std.core.Debug",
    }
}

/// Where `@derive(…, name, …)` names `name`.
fn derive_span(attrs: &[ast::Attribute], name: &str) -> Option<Span> {
    attrs
        .iter()
        .filter(|attr| attr.path.len() == 1 && attr.path[0].name.is("derive"))
        .flat_map(|attr| &attr.args)
        .find_map(|arg| match arg {
            ast::AttrArg::Expr(e) => match &e.kind {
                ast::ExprKind::Path { segments } if segments.len() == 1 && segments[0].name.is(name) => Some(e.span),
                _ => None,
            },
            _ => None,
        })
}

fn fields(decl: &ast::StructDecl) -> impl Iterator<Item = &ast::FieldDecl> {
    decl.members.iter().filter_map(|member| match &member.kind {
        ast::MemberKind::Field(field) => Some(field),
        _ => None,
    })
}

fn ident(name: &str, span: Span) -> ast::Ident {
    ast::Ident { name: Symbol::intern(name), span }
}

/// The declaration's parameters, each type parameter bounded by `bound` as well.
fn bounded(generics: &[ast::GenericParam], bound: &str, span: Span) -> Vec<ast::GenericParam> {
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

/// `E.V`: the variant `V` read from the enum `E`'s name, as the parser writes it.
fn variant_of(enum_name: Symbol, variant: ast::Ident, span: Span) -> ast::Expr {
    expr(ast::ExprKind::Field { base: Box::new(path(&[enum_name], span)), name: variant }, span)
}

fn ordering(variant: &str, span: Span) -> ast::Expr {
    variant_of(Symbol::intern("std.core.Ordering"), ident(variant, span), span)
}

/// `a.cmp(b)` for each pair in turn, the next only while they are `Equal`.
fn compare_in_order(mut pairs: Vec<(ast::Expr, ast::Expr)>, span: Span) -> Vec<ast::Stmt> {
    let Some((a, b)) = pairs.pop() else { return vec![ret(Some(ordering("Equal", span)), span)] };
    let order = Symbol::intern("$order");
    let cmp = |a, b| method_call(a, ident("cmp", span), vec![b], span);
    let mut stmts = Vec::new();
    for (a, b) in pairs {
        stmts.push(stmt(ast::StmtKind::Assign { targets: vec![path(&[order], span)], op: None, value: cmp(a, b) }, span));
        let differs = expr(ast::ExprKind::Binary { op: ast::BinOp::Ne, lhs: Box::new(path(&[order], span)), rhs: Box::new(ordering("Equal", span)) }, span);
        stmts.push(stmt(
            ast::StmtKind::If(ast::IfStmt { cond: ast::Condition::Expr(differs), then_block: block(vec![ret(Some(path(&[order], span)), span)], span), else_block: None }),
            span,
        ));
    }
    stmts.push(ret(Some(cmp(a, b)), span));
    stmts
}

/// `fn cmp(self, other: Self) -> Ordering: <body>`.
fn cmp_member(body: Vec<ast::Stmt>, target: &ast::TypeExpr, span: Span) -> ast::Member {
    let decl = ast::FnDecl {
        name: ident("cmp", span),
        params: vec![crate::errors::receiver(span), crate::errors::named_param(Symbol::intern("other"), ast::Mode::Borrow, target.clone(), span)],
        ret: Some(named_type("std.core.Ordering", span)),
        ..fn_decl(span)
    };
    member(decl, block(body, span), span)
}

/// `[DRV-1]` — `Ord` on a struct: lexicographic over the fields in declaration order.
fn struct_ord(decl: &ast::StructDecl, span: Span) -> ast::Item {
    let other = path(&[Symbol::intern("other")], span);
    let pairs = fields(decl)
        .map(|field| {
            let of = |base: ast::Expr| expr(ast::ExprKind::Field { base: Box::new(base), name: field.name }, span);
            (of(self_expr(span)), of(other.clone()))
        })
        .collect();
    let target = owner_type(decl.name, &decl.generics);
    let body = compare_in_order(pairs, span);
    extend_item(&bounded(&decl.generics, "std.core.Ord", span), target.clone(), vec![named_type("std.core.Ord", span)], vec![cmp_member(body, &target, span)], span)
}

/// `E.V(a, b)` binding each field to `prefix` and its position; `E.V` for a unit variant, `E.V(..)`
/// when nothing is bound.
fn variant_pattern(enum_name: ast::Ident, variant: &ast::Variant, prefix: Option<&str>, span: Span) -> ast::Pattern {
    let segments = vec![ast::Ident { name: enum_name.name, span }, ast::Ident { name: variant.name.name, span }];
    if variant.fields.is_empty() {
        return pattern_of(ast::PatternKind::Path { segments }, span);
    }
    let Some(prefix) = prefix else {
        return pattern_of(ast::PatternKind::Constructor { path: segments, fields: Vec::new(), has_rest: true }, span);
    };
    let fields = (0..variant.fields.len())
        .map(|index| ast::FieldPattern {
            name: variant.fields[index].name,
            pattern: pattern_of(
                ast::PatternKind::Bind { name: ident(&format!("{prefix}{index}"), span), by_ref: true, mutable: false, sub: None },
                span,
            ),
            span,
        })
        .collect();
    pattern_of(ast::PatternKind::Constructor { path: segments, fields, has_rest: false }, span)
}

/// `[DRV-1]` — `Ord` on an enum: by variant order, then the fields of the variant.
fn enum_ord(decl: &ast::EnumDecl, span: Span) -> ast::Item {
    let self_ = || self_expr(span);
    let other = || path(&[Symbol::intern("other")], span);
    let mut arms = Vec::new();
    for variant in &decl.variants {
        let pairs = (0..variant.fields.len())
            .map(|index| (path(&[Symbol::intern(&format!("$a{index}"))], span), path(&[Symbol::intern(&format!("$b{index}"))], span)))
            .collect();
        let same = ast::MatchArm {
            id: ast::NodeId::DUMMY,
            pattern: variant_pattern(decl.name, variant, Some("$b"), span),
            guard: None,
            body: ast::MatchArmBody::Block(block(compare_in_order(pairs, span), span)),
            span,
        };
        let pass = arm(pattern_of(ast::PatternKind::Wild, span), stmt(ast::StmtKind::Pass, span), span);
        let inner = stmt(ast::StmtKind::Match { scrutinee: other(), arms: vec![same, pass] }, span);
        arms.push(arm(variant_pattern(decl.name, variant, Some("$a"), span), inner, span));
    }
    let rank = |value: ast::Expr| {
        let arms = decl
            .variants
            .iter()
            .enumerate()
            .map(|(index, variant)| ast::MatchArm {
                id: ast::NodeId::DUMMY,
                pattern: variant_pattern(decl.name, variant, None, span),
                guard: None,
                body: ast::MatchArmBody::Expr(expr(ast::ExprKind::Lit(ast::Literal::Int { value: index as u128, suffix: None }), span)),
                span,
            })
            .collect();
        expr(ast::ExprKind::Match { scrutinee: Box::new(value), arms }, span)
    };
    let (left, right) = (Symbol::intern("$left"), Symbol::intern("$right"));
    let mut body = vec![stmt(ast::StmtKind::Match { scrutinee: self_(), arms }, span)];
    body.push(stmt(ast::StmtKind::Assign { targets: vec![path(&[left], span)], op: None, value: rank(self_()) }, span));
    body.push(stmt(ast::StmtKind::Assign { targets: vec![path(&[right], span)], op: None, value: rank(other()) }, span));
    body.push(ret(Some(method_call(path(&[left], span), ident("cmp", span), vec![path(&[right], span)], span)), span));
    let target = owner_type(decl.name, &decl.generics);
    extend_item(&bounded(&decl.generics, "std.core.Ord", span), target.clone(), vec![named_type("std.core.Ord", span)], vec![cmp_member(body, &target, span)], span)
}

/// `fn default() -> Self: return <value>`.
fn default_member(value: ast::Expr, target: &ast::TypeExpr, span: Span) -> ast::Member {
    let decl = ast::FnDecl { name: ident("default", span), ret: Some(target.clone()), ..fn_decl(span) };
    member(decl, block(vec![ret(Some(value), span)], span), span)
}

/// `[DRV-1]` — `Default` on a struct: each field's declared default, else its type's `Default`.
fn struct_default(decl: &ast::StructDecl, span: Span) -> ast::Item {
    let args = fields(decl)
        .filter(|field| field.default.is_none())
        .map(|field| {
            let default_of = expr(
                ast::ExprKind::IndexOrInstantiate {
                    base: Box::new(path(&[Symbol::intern("std.core.default_of")], span)),
                    args: vec![ast::TypeOrExpr::Type(field.ty.clone())],
                },
                span,
            );
            ast::Arg { name: Some(field.name), value: call(default_of, Vec::new(), span), span }
        })
        .collect();
    let made = expr(ast::ExprKind::Call { callee: Box::new(path(&[Symbol::intern("Self")], span)), args }, span);
    let target = owner_type(decl.name, &decl.generics);
    extend_item(&bounded(&decl.generics, "std.core.Default", span), target.clone(), vec![named_type("std.core.Default", span)], vec![default_member(made, &target, span)], span)
}

/// `[DRV-1]` — `Default` on an enum: the one unit variant marked `@default`.
fn enum_default(decl: &mut ast::EnumDecl, span: Span, sink: &mut Sink) -> Option<ast::Item> {
    let mut marked = Vec::new();
    for variant in &mut decl.variants {
        let before = variant.attrs.len();
        variant.attrs.retain(|attr| !(attr.path.len() == 1 && attr.path[0].name.is("default") && attr.args.is_empty()));
        if variant.attrs.len() != before {
            marked.push((variant.name, variant.fields.is_empty(), variant.span));
        }
    }
    let refuse = |at: Span, message: String, sink: &mut Sink| {
        sink.emit(
            Diagnostic::error(codes::E2080, at, message)
                .help("mark one unit variant `@default`")
                .note("an enum's derived `Default` is the variant marked `@default` [DRV-1]"),
        );
    };
    let name = decl.name.name;
    match marked.as_slice() {
        [(variant, true, _)] => {
            let target = owner_type(decl.name, &decl.generics);
            let value = variant_of(name, *variant, span);
            Some(extend_item(&decl.generics, target.clone(), vec![named_type("std.core.Default", span)], vec![default_member(value, &target, span)], span))
        }
        [(variant, false, at)] => {
            refuse(*at, format!("`{name}.{}` is marked `@default` but has fields", variant.name), sink);
            None
        }
        [] => {
            refuse(span, format!("`{name}` derives `Default` but marks no variant `@default`"), sink);
            None
        }
        [_, (_, _, at), ..] => {
            refuse(*at, format!("`{name}` marks more than one variant `@default`"), sink);
            None
        }
    }
}

/// `[DRV-1]` — `Display` on a unit-only enum: the variant's name.
fn unit_display(decl: &ast::EnumDecl, span: Span) -> ast::Item {
    let arms = decl
        .variants
        .iter()
        .map(|variant| {
            let text = expr(ast::ExprKind::Lit(ast::Literal::Str(variant.name.name.to_string())), span);
            arm(variant_pattern(decl.name, variant, None, span), write_str(text, span), span)
        })
        .collect();
    let body = stmt(ast::StmtKind::Match { scrutinee: self_expr(span), arms }, span);
    extend_item(&decl.generics, owner_type(decl.name, &decl.generics), vec![named_type("std.core.Display", span)], vec![fmt_member(body, span)], span)
}

/// `[DRV-1]` — `Display` on a struct or a payload enum: its `Debug` text, written into the
/// `Formatter` directly.
fn debug_display(name: ast::Ident, generics: &[ast::GenericParam], span: Span) -> ast::Item {
    let f = path(&[Symbol::intern("f")], span);
    let body = ret(Some(call(path(&[Symbol::intern("std.fmt.implicit_fmt_debug")], span), vec![self_expr(span), f], span)), span);
    extend_item(&bounded(generics, "std.core.Debug", span), owner_type(name, generics), vec![named_type("std.core.Display", span)], vec![fmt_member(body, span)], span)
}
