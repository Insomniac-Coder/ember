//! A stable textual dump of the AST, for `--emit=ast` and snapshot tests.
//!
//! `[HIR-2]`'s sibling for the AST: the form is deterministic and structural,
//! not source-reproducing. The source-reproducing printer belongs to the
//! formatter (`[FMT-1]`), which is Phase 1.

use std::fmt::Write as _;

use super::*;

pub fn dump(module: &Module) -> String {
    let mut p = Printer { out: String::new(), depth: 0 };
    p.line("Module");
    p.depth += 1;
    if let Some(d) = &module.directive {
        p.line(&format!("Directive {} = {:?}", d.name.name, d.value));
    }
    for import in &module.imports {
        p.import(import);
    }
    for item in &module.items {
        p.item(item);
    }
    p.out
}

struct Printer {
    out: String,
    depth: usize,
}

impl Printer {
    fn line(&mut self, text: &str) {
        for _ in 0..self.depth {
            self.out.push_str("  ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn nest(&mut self, text: &str, body: impl FnOnce(&mut Printer)) {
        self.line(text);
        self.depth += 1;
        body(self);
        self.depth -= 1;
    }

    fn import(&mut self, import: &Import) {
        match &import.kind {
            ImportKind::Module { path, alias } => {
                let path = join(path);
                match alias {
                    Some(a) => self.line(&format!("Import {path} as {}", a.name)),
                    None => self.line(&format!("Import {path}")),
                }
            }
            ImportKind::Items { path, items, glob } => {
                let names: Vec<String> = items
                    .iter()
                    .map(|i| match &i.alias {
                        Some(a) => format!("{} as {}", i.name.name, a.name),
                        None => i.name.name.to_string(),
                    })
                    .collect();
                let list = if *glob { "*".to_string() } else { names.join(", ") };
                self.line(&format!("From {} import {list}", join(path)));
            }
            ImportKind::Foreign { language, header, .. } => {
                self.line(&format!("ImportForeign {language:?} {header:?}"));
            }
        }
    }

    fn item(&mut self, item: &Item) {
        if let Some(doc) = &item.doc {
            self.line(&format!("Doc {:?}", doc));
        }
        for attr in &item.attrs {
            self.line(&format!("@{}", join(&attr.path)));
        }
        match &item.kind {
            ItemKind::Fn(f) => self.fn_decl(f, item.vis),
            ItemKind::Struct(s) => self.nest(
                &format!("Struct {}{}", s.name.name, vis_suffix(item.vis)),
                |p| {
                    for m in &s.members {
                        p.member(m);
                    }
                },
            ),
            ItemKind::Class(c) => {
                let base = c.base.as_ref().map(|b| format!("({})", type_str(b))).unwrap_or_default();
                self.nest(
                    &format!("Class {}{base} {:?}{}", c.name.name, c.openness, vis_suffix(item.vis)),
                    |p| {
                        for m in &c.members {
                            p.member(m);
                        }
                    },
                )
            }
            ItemKind::Enum(e) => self.nest(&format!("Enum {}", e.name.name), |p| {
                for v in &e.variants {
                    p.line(&format!("Variant {}", v.name.name));
                }
                for m in &e.members {
                    p.member(m);
                }
            }),
            ItemKind::Interface(i) => self.nest(&format!("Interface {}", i.name.name), |p| {
                for m in &i.members {
                    p.member(m);
                }
            }),
            ItemKind::Extend(e) => self.nest(&format!("Extend {}", type_str(&e.target)), |p| {
                for m in &e.members {
                    p.member(m);
                }
            }),
            ItemKind::Const(c) => self.nest(&format!("Const {}", c.name.name), |p| {
                p.expr(&c.value);
            }),
            ItemKind::Static(s) => {
                let m = if s.is_mut { " mut" } else { "" };
                self.nest(&format!("Static{m} {}: {}", s.name.name, type_str(&s.ty)), |p| {
                    p.expr(&s.value);
                })
            }
            ItemKind::TypeAlias(t) => self.line(&format!("TypeAlias {}", t.name.name)),
            ItemKind::ExternBlock(b) => self.nest(&format!("Extern {:?}", b.abi), |p| {
                for i in &b.items {
                    p.item(i);
                }
            }),
            ItemKind::Comptime(b) => self.nest("Comptime", |p| p.block(b)),
        }
    }

    fn fn_decl(&mut self, f: &FnDecl, vis: Visibility) {
        let params: Vec<String> = f
            .params
            .iter()
            .map(|param| {
                let mode = match param.mode {
                    Mode::Borrow => "",
                    Mode::Mut => "mut ",
                    Mode::Owned => "owned ",
                };
                match &param.kind {
                    ParamKind::Receiver { .. } => format!("{mode}self"),
                    ParamKind::Named { name, ty } => {
                        format!("{mode}{}: {}", name.name, type_str(ty))
                    }
                }
            })
            .collect();
        let ret = f.ret.as_ref().map(|t| format!(" -> {}", type_str(t))).unwrap_or_default();
        let dispatch = match f.dispatch {
            Dispatch::Static => "",
            Dispatch::Virtual => "virtual ",
            Dispatch::Override => "override ",
        };
        let header =
            format!("{dispatch}Fn {}({}){ret}{}", f.name.name, params.join(", "), vis_suffix(vis));
        match &f.body {
            Some(body) => self.nest(&header, |p| p.block(body)),
            None => self.line(&format!("{header} (no body)")),
        }
    }

    fn member(&mut self, member: &Member) {
        if let Some(doc) = &member.doc {
            self.line(&format!("Doc {:?}", doc));
        }
        for attr in &member.attrs {
            self.line(&format!("@{}", join(&attr.path)));
        }
        match &member.kind {
            MemberKind::Field(f) => {
                let kw = if f.is_let { "let " } else { "" };
                let ro = if member.read_only_outside { " (read)" } else { "" };
                let default = f.default.is_some().then_some(" = …").unwrap_or_default();
                self.line(&format!(
                    "{kw}Field {}: {}{default}{}{ro}",
                    f.name.name,
                    type_str(&f.ty),
                    vis_suffix(member.vis)
                ));
            }
            MemberKind::Fn(f) => self.fn_decl(f, member.vis),
            MemberKind::Const(c) => self.line(&format!("Const {}", c.name.name)),
            MemberKind::TypeAlias(t) => self.line(&format!("TypeAlias {}", t.name.name)),
        }
    }

    fn block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.stmt(stmt);
        }
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Decl { pattern, ty, init } => {
                let ty = ty.as_ref().map(|t| format!(": {}", type_str(t))).unwrap_or_default();
                self.nest(&format!("Decl {}{ty}", pattern_str(pattern)), |p| {
                    if let Some(e) = init {
                        p.expr(e);
                    }
                });
            }
            StmtKind::Assign { targets, op, value } => {
                let op = op.map(|o| o.as_str()).unwrap_or("");
                let names: Vec<String> = targets.iter().map(expr_summary).collect();
                self.nest(&format!("Assign {} {op}=", names.join(", ")), |p| p.expr(value));
            }
            StmtKind::Expr(e) => self.nest("ExprStmt", |p| p.expr(e)),
            StmtKind::Return(e) => self.nest("Return", |p| {
                if let Some(e) = e {
                    p.expr(e);
                }
            }),
            StmtKind::Break { label } => {
                self.line(&format!("Break{}", label_str(label)));
            }
            StmtKind::Continue { label } => {
                self.line(&format!("Continue{}", label_str(label)));
            }
            StmtKind::Pass => self.line("Pass"),
            StmtKind::If(if_stmt) => self.if_stmt(if_stmt),
            StmtKind::While { cond, body, else_block, label } => {
                self.nest(&format!("While{}", label_str(label)), |p| {
                    p.condition(cond);
                    p.nest("Body", |p| p.block(body));
                    if let Some(b) = else_block {
                        p.nest("Else", |p| p.block(b));
                    }
                });
            }
            StmtKind::For { pattern, iter, body, else_block, label } => {
                self.nest(&format!("For {}{}", pattern_str(pattern), label_str(label)), |p| {
                    p.nest("In", |p| p.expr(iter));
                    p.nest("Body", |p| p.block(body));
                    if let Some(b) = else_block {
                        p.nest("Else", |p| p.block(b));
                    }
                });
            }
            StmtKind::Match { scrutinee, arms } => {
                self.nest("Match", |p| {
                    p.expr(scrutinee);
                    for arm in arms {
                        p.nest(&format!("Arm {}", pattern_str(&arm.pattern)), |p| match &arm.body {
                            MatchArmBody::Block(b) => p.block(b),
                            MatchArmBody::Expr(e) => p.expr(e),
                        });
                    }
                });
            }
            StmtKind::With { items, body } => {
                self.nest("With", |p| {
                    for item in items {
                        p.expr(&item.value);
                    }
                    p.nest("Body", |p| p.block(body));
                });
            }
            StmtKind::Defer(b) => self.nest("Defer", |p| p.block(b)),
            StmtKind::Unsafe(b) => self.nest("Unsafe", |p| p.block(b)),
            StmtKind::Comptime(b) => self.nest("Comptime", |p| p.block(b)),
        }
    }

    fn if_stmt(&mut self, if_stmt: &IfStmt) {
        self.nest("If", |p| {
            p.condition(&if_stmt.cond);
            p.nest("Then", |p| p.block(&if_stmt.then_block));
            match if_stmt.else_block.as_deref() {
                Some(ElseBranch::Block(b)) => p.nest("Else", |p| p.block(b)),
                Some(ElseBranch::If(nested)) => p.nest("Elif", |p| p.if_stmt(nested)),
                None => {}
            }
        });
    }

    fn condition(&mut self, cond: &Condition) {
        match cond {
            Condition::Expr(e) => self.nest("Cond", |p| p.expr(e)),
            Condition::Pattern { pattern, value } => {
                self.nest(&format!("CondPattern {}", pattern_str(pattern)), |p| p.expr(value));
            }
        }
    }

    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Lit(lit) => self.line(&format!("Lit {}", literal_str(lit))),
            ExprKind::Path { segments } => self.line(&format!("Path {}", join(segments))),
            ExprKind::SelfExpr => self.line("Self"),
            ExprKind::Field { base, name } => {
                self.nest(&format!("Field .{}", name.name), |p| p.expr(base))
            }
            ExprKind::TupleField { base, index } => {
                self.nest(&format!("TupleField .{index}"), |p| p.expr(base))
            }
            ExprKind::IndexOrInstantiate { base, args } => {
                self.nest("IndexOrInstantiate", |p| {
                    p.expr(base);
                    for a in args {
                        p.expr(a);
                    }
                });
            }
            ExprKind::Call { callee, args } => self.nest("Call", |p| {
                p.expr(callee);
                for a in args {
                    p.arg(a);
                }
            }),
            ExprKind::MethodCall { recv, name, args, .. } => {
                self.nest(&format!("MethodCall .{}", name.name), |p| {
                    p.expr(recv);
                    for a in args {
                        p.arg(a);
                    }
                })
            }
            ExprKind::Unary { op, operand } => {
                self.nest(&format!("Unary {op:?}"), |p| p.expr(operand))
            }
            ExprKind::Binary { op, lhs, rhs } => {
                self.nest(&format!("Binary {}", op.as_str()), |p| {
                    p.expr(lhs);
                    p.expr(rhs);
                })
            }
            ExprKind::Logical { op, lhs, rhs } => self.nest(&format!("Logical {op:?}"), |p| {
                p.expr(lhs);
                p.expr(rhs);
            }),
            ExprKind::Ternary { then_expr, cond, else_expr } => self.nest("Ternary", |p| {
                p.expr(cond);
                p.expr(then_expr);
                p.expr(else_expr);
            }),
            ExprKind::Range { lo, hi, inclusive } => {
                let op = if *inclusive { "..=" } else { ".." };
                self.nest(&format!("Range {op}"), |p| {
                    if let Some(e) = lo {
                        p.expr(e);
                    }
                    if let Some(e) = hi {
                        p.expr(e);
                    }
                })
            }
            ExprKind::Cast { expr, ty } => {
                self.nest(&format!("Cast as {}", type_str(ty)), |p| p.expr(expr))
            }
            ExprKind::Downcast { expr, ty, forced } => {
                let op = if *forced { "as!" } else { "as?" };
                self.nest(&format!("Downcast {op} {}", type_str(ty)), |p| p.expr(expr))
            }
            ExprKind::Try(e) => self.nest("Try", |p| p.expr(e)),
            ExprKind::OptChain { base, name, .. } => {
                self.nest(&format!("OptChain ?.{}", name.name), |p| p.expr(base))
            }
            ExprKind::Lambda(l) => self.nest(
                if l.is_owned { "OwnedLambda" } else { "Lambda" },
                |p| match &l.body {
                    LambdaBody::Expr(e) => p.expr(e),
                    LambdaBody::Block(b) => p.block(b),
                },
            ),
            ExprKind::Match { scrutinee, arms } => self.nest("MatchExpr", |p| {
                p.expr(scrutinee);
                for arm in arms {
                    p.nest(&format!("Arm {}", pattern_str(&arm.pattern)), |p| match &arm.body {
                        MatchArmBody::Block(b) => p.block(b),
                        MatchArmBody::Expr(e) => p.expr(e),
                    });
                }
            }),
            ExprKind::Tuple(items) => self.nest("Tuple", |p| {
                for e in items {
                    p.expr(e);
                }
            }),
            ExprKind::ArrayLit(items) => self.nest("Array", |p| {
                for e in items {
                    p.expr(e);
                }
            }),
            ExprKind::ArrayRepeat { value, count } => self.nest("ArrayRepeat", |p| {
                p.expr(value);
                p.expr(count);
            }),
            ExprKind::FString(parts) => self.nest("FString", |p| {
                for part in parts {
                    match part {
                        FStringPart::Text(t) => p.line(&format!("Text {t:?}")),
                        FStringPart::Expr { expr, format_spec } => {
                            let spec = format_spec
                                .as_ref()
                                .map(|s| format!(" :{s}"))
                                .unwrap_or_default();
                            p.nest(&format!("Hole{spec}"), |p| p.expr(expr));
                        }
                    }
                }
            }),
            ExprKind::RefOf { mutable, place } => {
                let kw = if *mutable { "ref mut" } else { "ref" };
                self.nest(&format!("RefOf {kw}"), |p| p.expr(place))
            }
            ExprKind::Paren(e) => self.expr(e),
            ExprKind::Error => self.line("Error"),
        }
    }

    fn arg(&mut self, arg: &Arg) {
        match &arg.name {
            Some(name) => self.nest(&format!("Arg {}=", name.name), |p| p.expr(&arg.value)),
            None => self.expr(&arg.value),
        }
    }
}

fn label_str(label: &Option<Ident>) -> String {
    label.map(|l| format!(" '{}", l.name)).unwrap_or_default()
}

fn vis_suffix(vis: Visibility) -> &'static str {
    match vis.kind {
        VisKind::Private => "",
        VisKind::Package => " pub(package)",
        VisKind::Public => " pub",
    }
}

fn join(path: &[Ident]) -> String {
    path.iter().map(|i| i.name.to_string()).collect::<Vec<_>>().join(".")
}

pub fn literal_str(lit: &Literal) -> String {
    match lit {
        Literal::Int { value, suffix } => match suffix {
            Some(s) => format!("{value}{s:?}"),
            None => value.to_string(),
        },
        Literal::Float { value, suffix } => match suffix {
            Some(s) => format!("{value}{s:?}"),
            None => value.to_string(),
        },
        Literal::Bool(b) => b.to_string(),
        Literal::Char(c) => format!("{c:?}"),
        Literal::Str(s) => format!("{s:?}"),
        Literal::Bytes(b) => format!("b{b:?}"),
        Literal::CStr(b) => format!("c{b:?}"),
    }
}

pub fn type_str(ty: &TypeExpr) -> String {
    match &ty.kind {
        TypeKind::Path { segments, args } => {
            let base = join(segments);
            if args.is_empty() {
                base
            } else {
                let inner: Vec<String> = args
                    .iter()
                    .map(|a| match a {
                        GenericArg::Type(t) => type_str(t),
                        GenericArg::Const(_) => "<const>".to_string(),
                        GenericArg::Assoc { name, ty } => format!("{} = {}", name.name, type_str(ty)),
                    })
                    .collect();
                format!("{base}[{}]", inner.join(", "))
            }
        }
        TypeKind::Ref { mutable, inner } => {
            let kw = if *mutable { "ref mut " } else { "ref " };
            format!("{kw}{}", type_str(inner))
        }
        TypeKind::Ptr { mutable, inner } => {
            let kw = if *mutable { "*mut " } else { "*" };
            format!("{kw}{}", type_str(inner))
        }
        TypeKind::Tuple(items) => {
            format!("({})", items.iter().map(type_str).collect::<Vec<_>>().join(", "))
        }
        TypeKind::Fn { abi, params, ret } => {
            let abi = abi.as_ref().map(|a| format!("extern {a:?} ")).unwrap_or_default();
            let ret = ret.as_ref().map(|r| format!(" -> {}", type_str(r))).unwrap_or_default();
            format!("{abi}fn({}){ret}", params.iter().map(type_str).collect::<Vec<_>>().join(", "))
        }
        TypeKind::Dyn(bounds) => {
            format!("dyn {}", bounds.iter().map(type_str).collect::<Vec<_>>().join(" + "))
        }
        TypeKind::Array { elem, .. } => format!("[{}; N]", type_str(elem)),
        TypeKind::SelfType => "Self".to_string(),
        TypeKind::Void => "void".to_string(),
        TypeKind::Never => "!".to_string(),
        TypeKind::Infer => "_".to_string(),
    }
}

pub fn pattern_str(pattern: &Pattern) -> String {
    match &pattern.kind {
        PatternKind::Wild => "_".to_string(),
        PatternKind::Lit(lit) => literal_str(lit),
        PatternKind::Range { lo, hi, inclusive } => {
            let op = if *inclusive { "..=" } else { ".." };
            format!("{}{op}{}", literal_str(lo), literal_str(hi))
        }
        PatternKind::Bind { name, by_ref, mutable, sub } => {
            let mut out = String::new();
            if *by_ref {
                out.push_str("ref ");
            }
            if *mutable {
                out.push_str("mut ");
            }
            let _ = write!(out, "{}", name.name);
            if let Some(sub) = sub {
                let _ = write!(out, " @ {}", pattern_str(sub));
            }
            out
        }
        PatternKind::Path { segments } => join(segments),
        PatternKind::Constructor { path, fields, has_rest } => {
            let mut inner: Vec<String> = fields
                .iter()
                .map(|f| match &f.name {
                    Some(name) => format!("{}={}", name.name, pattern_str(&f.pattern)),
                    None => pattern_str(&f.pattern),
                })
                .collect();
            if *has_rest {
                inner.push("..".to_string());
            }
            format!("{}({})", join(path), inner.join(", "))
        }
        PatternKind::Tuple(items) => {
            format!("({})", items.iter().map(pattern_str).collect::<Vec<_>>().join(", "))
        }
        PatternKind::Slice { prefix, rest, suffix } => {
            let mut parts: Vec<String> = prefix.iter().map(pattern_str).collect();
            if let Some(binding) = rest {
                parts.push(match binding {
                    Some(name) => format!("..{}", name.name),
                    None => "..".to_string(),
                });
            }
            parts.extend(suffix.iter().map(pattern_str));
            format!("[{}]", parts.join(", "))
        }
        PatternKind::Or(alts) => alts.iter().map(pattern_str).collect::<Vec<_>>().join(" | "),
        PatternKind::Error => "<error>".to_string(),
    }
}

/// A one-line summary of an expression, for assignment targets in the dump.
fn expr_summary(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Path { segments } => join(segments),
        ExprKind::Field { base, name } => format!("{}.{}", expr_summary(base), name.name),
        ExprKind::TupleField { base, index } => format!("{}.{index}", expr_summary(base)),
        ExprKind::IndexOrInstantiate { base, .. } => format!("{}[…]", expr_summary(base)),
        ExprKind::SelfExpr => "self".to_string(),
        _ => "<expr>".to_string(),
    }
}
