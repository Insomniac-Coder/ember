//! `ember fmt` — the canonical printer (`[FMT-1]`).
//!
//! > deterministic and configuration-free except line width (default 100).
//! > 4-space indentation; one blank line between methods, two between items;
//! > trailing commas in multi-line argument/element lists; spaces around
//! > binary operators, none around `**` when both operands are atoms;
//! > `x: T = v` spacing; imports sorted and merged; attributes one per line;
//! > the formatter preserves comments and blank-line groups (max 2
//! > consecutive); `pass` inserted for empty blocks.
//!
//! `[FMT-1]` also demands `fmt(fmt(x)) == fmt(x)` and `parse(fmt(x)) ≡
//! parse(x)`. Both are tested over the corpus by `ember_driver`'s harness.
//!
//! Comments are not in the tree — `[II.7]` makes them non-tokens — but the
//! lexer records each one's span and whether it had a line to itself, which is
//! exactly enough to put them back. Every printed construct carries the span
//! it came from, and comments are flushed in span order as the printer passes
//! them.

use ember_ast as ast;
use ember_lexer::Comment;
use ember_span::Span;

/// `[FMT-1]`'s only configuration.
pub const LINE_WIDTH: usize = 100;

const INDENT: &str = "    ";

pub fn format(module: &ast::Module, source: &str, comments: &[Comment]) -> String {
    let mut printer = Printer {
        source,
        comments: comments.to_vec(),
        next_comment: 0,
        out: String::new(),
        depth: 0,
    };
    printer.module(module);
    printer.finish()
}

struct Printer<'a> {
    source: &'a str,
    comments: Vec<Comment>,
    /// How many comments have been emitted already. Comments are visited in
    /// span order, so one cursor is enough.
    next_comment: usize,
    out: String,
    depth: usize,
}

impl Printer<'_> {
    fn finish(mut self) -> String {
        // Anything after the last item — a trailing comment — still belongs in
        // the output.
        while self.next_comment < self.comments.len() {
            let comment = self.comments[self.next_comment].clone();
            self.next_comment += 1;
            let text = self.text(comment.span).to_string();
            self.line(&text);
        }
        // `[FMT-1]` — one trailing newline, and no blank lines before it.
        while self.out.ends_with("\n\n") {
            self.out.pop();
        }
        if !self.out.ends_with('\n') && !self.out.is_empty() {
            self.out.push('\n');
        }
        self.out
    }

    fn text(&self, span: Span) -> &str {
        self.source.get(span.start as usize..span.end as usize).unwrap_or("")
    }

    fn line(&mut self, text: &str) {
        if !text.is_empty() {
            for _ in 0..self.depth {
                self.out.push_str(INDENT);
            }
            self.out.push_str(text);
        }
        self.out.push('\n');
    }

    fn blank(&mut self) {
        // `[FMT-1]` — never more than two blank lines in a row, and none at
        // the very start.
        if self.out.is_empty() || self.out.ends_with("\n\n\n") {
            return;
        }
        self.out.push('\n');
    }

    /// Emit every comment that started before `span`, so comments land where
    /// they were written.
    fn comments_before(&mut self, span: Span) {
        while self.next_comment < self.comments.len() {
            let comment = self.comments[self.next_comment].clone();
            if comment.span.start >= span.start {
                break;
            }
            self.next_comment += 1;
            let text = self.text(comment.span).trim_end().to_string();
            if comment.own_line {
                self.line(&text);
            } else {
                // A trailing comment goes on the line just written.
                if self.out.ends_with('\n') {
                    self.out.pop();
                }
                self.out.push(' ');
                self.out.push_str(&text);
                self.out.push('\n');
            }
        }
    }

    /// Drop the comments inside a span without printing them. Used where a
    /// construct is printed from the tree and its interior comments have
    /// already been placed.
    fn skip_comments_in(&mut self, span: Span) {
        while self.next_comment < self.comments.len()
            && self.comments[self.next_comment].span.start < span.end
        {
            self.next_comment += 1;
        }
    }

    // -- module --------------------------------------------------------------

    fn module(&mut self, module: &ast::Module) {
        if let Some(directive) = &module.directive {
            self.line(&format!("#! {} {:?}", directive.name.name, directive.value));
            self.blank();
        }
        self.imports(&module.imports);

        for (index, item) in module.items.iter().enumerate() {
            if index > 0 || !module.imports.is_empty() {
                // `[FMT-1]` — two blank lines between items. The separation
                // comes first, so a comment written above an item stays
                // attached to it rather than floating above the gap.
                self.blank();
                self.blank();
            }
            self.comments_before(item.span);
            self.item(item);
        }
    }

    /// `[FMT-1]` — imports sorted (`std` first, then dependencies, then local)
    /// and merged.
    fn imports(&mut self, imports: &[ast::Import]) {
        if imports.is_empty() {
            return;
        }
        let mut lines: Vec<(u8, String)> = Vec::new();
        for import in imports {
            self.skip_comments_in(import.span);
            let (rank, text) = match &import.kind {
                ast::ImportKind::Module { path, alias } => {
                    let path = dotted(path);
                    let rank = import_rank(&path);
                    let text = match alias {
                        Some(alias) => format!("import {path} as {}", alias.name),
                        None => format!("import {path}"),
                    };
                    (rank, text)
                }
                ast::ImportKind::Items { path, items, glob } => {
                    let path = dotted(path);
                    let rank = import_rank(&path);
                    let text = if *glob {
                        format!("from {path} import *")
                    } else {
                        let names: Vec<String> = items
                            .iter()
                            .map(|i| match i.alias {
                                Some(alias) => format!("{} as {}", i.name.name, alias.name),
                                None => i.name.name.to_string(),
                            })
                            .collect();
                        format!("from {path} import {}", names.join(", "))
                    };
                    (rank, text)
                }
                ast::ImportKind::Foreign { language, header, .. } => {
                    let language = match language {
                        ast::ForeignLanguage::C => "c",
                        ast::ForeignLanguage::Cpp => "cpp",
                    };
                    (3, format!("import {language} {header:?}"))
                }
            };
            lines.push((rank, text));
        }
        lines.sort();
        lines.dedup();
        for (_, text) in lines {
            self.line(&text);
        }
    }

    // -- items ---------------------------------------------------------------

    fn item(&mut self, item: &ast::Item) {
        for attr in &item.attrs {
            // `[FMT-1]` — attributes one per line.
            self.line(&self.attribute(attr));
        }
        let vis = visibility(&item.vis);
        match &item.kind {
            ast::ItemKind::Fn(decl) => self.function(&vis, decl),
            ast::ItemKind::Struct(decl) => {
                let header = self.type_header(&vis, "struct", decl.name, &decl.implements);
                self.line(&header);
                self.members(&decl.members, item.span);
            }
            ast::ItemKind::Enum(decl) => {
                let header = self.type_header(&vis, "enum", decl.name, &decl.implements);
                self.line(&header);
                self.variants(&decl.variants, &decl.members, item.span);
            }
            ast::ItemKind::Interface(decl) => {
                let mut header = format!("{vis}interface {}", decl.name.name);
                if !decl.supertraits.is_empty() {
                    let names: Vec<String> =
                        decl.supertraits.iter().map(|t| self.type_expr(t)).collect();
                    header.push_str(&format!(": {}", names.join(" + ")));
                }
                header.push(':');
                self.line(&header);
                self.members(&decl.members, item.span);
            }
            ast::ItemKind::Extend(decl) => {
                let mut header = format!("extend {}", self.type_expr(&decl.target));
                if !decl.implements.is_empty() {
                    let names: Vec<String> =
                        decl.implements.iter().map(|t| self.type_expr(t)).collect();
                    header.push_str(&format!(" implements {}", names.join(", ")));
                }
                header.push(':');
                self.line(&header);
                self.members(&decl.members, item.span);
            }
            ast::ItemKind::Const(decl) => {
                let ty = decl
                    .ty
                    .as_ref()
                    .map(|t| format!(": {}", self.type_expr(t)))
                    .unwrap_or_default();
                let value = self.expr(&decl.value);
                self.line(&format!("{vis}const {}{ty} = {value}", decl.name.name));
            }
            ast::ItemKind::Static(decl) => {
                let mutable = if decl.is_mut { "mut " } else { "" };
                let ty = self.type_expr(&decl.ty);
                let value = self.expr(&decl.value);
                self.line(&format!(
                    "{vis}static {mutable}{}: {ty} = {value}",
                    decl.name.name
                ));
            }
            ast::ItemKind::TypeAlias(decl) => {
                let target = decl.value.as_ref().map(|t| format!(" = {}", self.type_expr(t))).unwrap_or_default();
                self.line(&format!("{vis}type {}{target}", decl.name.name));
            }
            // A construct the formatter does not reshape is copied through as
            // it was written, which keeps `parse(fmt(x)) ≡ parse(x)` true even
            // where the printer has nothing to say.
            _ => {
                let text = self.text(item.span).to_string();
                self.skip_comments_in(item.span);
                for line in text.lines() {
                    self.line(line.trim_end());
                }
            }
        }
    }

    fn type_header(
        &mut self,
        vis: &str,
        keyword: &str,
        name: ast::Ident,
        implements: &[ast::TypeExpr],
    ) -> String {
        let mut header = format!("{vis}{keyword} {}", name.name);
        if !implements.is_empty() {
            let names: Vec<String> = implements.iter().map(|t| self.type_expr(t)).collect();
            header.push_str(&format!(" implements {}", names.join(", ")));
        }
        header.push(':');
        header
    }

    fn function(&mut self, vis: &str, decl: &ast::FnDecl) {
        let params: Vec<String> = decl.params.iter().map(|p| self.param(p)).collect();
        let ret = decl
            .ret
            .as_ref()
            .map(|t| format!(" -> {}", self.type_expr(t)))
            .unwrap_or_default();
        let unsafe_ = if decl.is_unsafe { "unsafe " } else { "" };
        let header = format!(
            "{vis}{unsafe_}fn {}({}){ret}:",
            decl.name.name,
            params.join(", ")
        );

        match &decl.body {
            Some(body) => {
                // `[FMT-1]`'s width applies to the header; a long parameter
                // list breaks one per line with a trailing comma.
                if header.len() + self.depth * INDENT.len() > LINE_WIDTH && !params.is_empty() {
                    self.line(&format!("{vis}{unsafe_}fn {}(", decl.name.name));
                    self.depth += 1;
                    for param in &params {
                        self.line(&format!("{param},"));
                    }
                    self.depth -= 1;
                    self.line(&format!("){ret}:"));
                } else {
                    self.line(&header);
                }
                self.block(body);
            }
            // An interface method has no body, so the colon would open a block
            // that never comes.
            None => {
                let mut header = header;
                header.pop();
                self.line(&header);
            }
        }
    }

    fn param(&mut self, param: &ast::Param) -> String {
        let mode = match param.mode {
            ast::Mode::Borrow => "",
            ast::Mode::Mut => "mut ",
            ast::Mode::Owned => "owned ",
        };
        match &param.kind {
            ast::ParamKind::Receiver { .. } => format!("{mode}self"),
            ast::ParamKind::Named { name, ty } => {
                format!("{mode}{}: {}", name.name, self.type_expr(ty))
            }
        }
    }

    fn members(&mut self, members: &[ast::Member], owner: Span) {
        if members.is_empty() {
            // `[CTL-9]`, `[FMT-1]` — an empty block is `pass`.
            self.depth += 1;
            self.line("pass");
            self.depth -= 1;
            return;
        }
        self.depth += 1;
        let mut previous_was_fn = false;
        for (index, member) in members.iter().enumerate() {
            let is_fn = matches!(member.kind, ast::MemberKind::Fn(_));
            // `[FMT-1]` — one blank line between methods, before any comment
            // that introduces the next one.
            if index > 0 && (is_fn || previous_was_fn) {
                self.blank();
            }
            self.comments_before(member.span);
            previous_was_fn = is_fn;
            self.member(member);
        }
        self.depth -= 1;
        let _ = owner;
    }

    fn member(&mut self, member: &ast::Member) {
        for attr in &member.attrs {
            self.line(&self.attribute(attr));
        }
        let vis = visibility(&member.vis);
        match &member.kind {
            ast::MemberKind::Field(field) => {
                let ty = self.type_expr(&field.ty);
                let default = field
                    .default
                    .as_ref()
                    .map(|d| format!(" = {}", self.expr(d)))
                    .unwrap_or_default();
                self.line(&format!("{vis}{}: {ty}{default}", field.name.name));
            }
            ast::MemberKind::Fn(decl) => self.function(&vis, decl),
            ast::MemberKind::Const(decl) => {
                let ty = decl
                    .ty
                    .as_ref()
                    .map(|t| format!(": {}", self.type_expr(t)))
                    .unwrap_or_default();
                let value = self.expr(&decl.value);
                self.line(&format!("{vis}const {}{ty} = {value}", decl.name.name));
            }
            ast::MemberKind::TypeAlias(decl) => {
                let target = decl.value.as_ref().map(|t| format!(" = {}", self.type_expr(t))).unwrap_or_default();
                self.line(&format!("{vis}type {}{target}", decl.name.name));
            }
        }
    }

    fn variants(&mut self, variants: &[ast::Variant], members: &[ast::Member], owner: Span) {
        if variants.is_empty() && members.is_empty() {
            self.depth += 1;
            self.line("pass");
            self.depth -= 1;
            return;
        }
        self.depth += 1;
        for variant in variants {
            self.comments_before(variant.span);
            for attr in &variant.attrs {
                self.line(&self.attribute(attr));
            }
            let mut text = variant.name.name.to_string();
            if !variant.fields.is_empty() {
                let fields: Vec<String> = variant
                    .fields
                    .iter()
                    .map(|f| match f.name {
                        Some(name) => format!("{}: {}", name.name, self.type_expr(&f.ty)),
                        None => self.type_expr(&f.ty),
                    })
                    .collect();
                text.push_str(&format!("({})", fields.join(", ")));
            }
            if let Some(discriminant) = &variant.discriminant {
                text.push_str(&format!(" = {}", self.expr(discriminant)));
            }
            self.line(&text);
        }
        self.depth -= 1;
        if !members.is_empty() {
            self.members(members, owner);
        }
    }

    fn attribute(&self, attr: &ast::Attribute) -> String {
        let path: Vec<String> = attr.path.iter().map(|s| s.name.to_string()).collect();
        if attr.args.is_empty() {
            return format!("@{}", path.join("."));
        }
        let args: Vec<String> = attr
            .args
            .iter()
            .map(|arg| match arg {
                ast::AttrArg::Expr(e) => self.expr_pure(e),
                ast::AttrArg::Named { name, value } => {
                    format!("{}={}", name.name, self.expr_pure(value))
                }
            })
            .collect();
        format!("@{}({})", path.join("."), args.join(", "))
    }

    // -- statements ----------------------------------------------------------

    fn block(&mut self, block: &ast::Block) {
        self.depth += 1;
        if block.stmts.is_empty() {
            self.line("pass");
            self.depth -= 1;
            return;
        }
        for stmt in &block.stmts {
            self.comments_before(stmt.span);
            self.stmt(stmt);
        }
        self.depth -= 1;
    }

    fn stmt(&mut self, stmt: &ast::Stmt) {
        match &stmt.kind {
            ast::StmtKind::Pass => self.line("pass"),
            ast::StmtKind::Expr(expr) => {
                let text = self.expr(expr);
                self.line(&text);
            }
            ast::StmtKind::Return(value) => match value {
                Some(value) => {
                    let text = self.expr(value);
                    self.line(&format!("return {text}"));
                }
                None => self.line("return"),
            },
            // `[FMT-1]` — `x: T = v` spacing.
            ast::StmtKind::Decl { pattern, ty, init } => {
                let pattern = self.pattern(pattern);
                let ty = ty
                    .as_ref()
                    .map(|t| format!(": {}", self.type_expr(t)))
                    .unwrap_or_default();
                let init = init
                    .as_ref()
                    .map(|e| format!(" = {}", self.expr(e)))
                    .unwrap_or_default();
                self.line(&format!("{pattern}{ty}{init}"));
            }
            ast::StmtKind::Assign { targets, op, value } => {
                let targets: Vec<String> = targets.iter().map(|t| self.expr(t)).collect();
                let value = self.expr(value);
                let op = op.map(|o| o.as_str()).unwrap_or("");
                self.line(&format!("{} {op}= {value}", targets.join(", ")));
            }
            ast::StmtKind::If(if_stmt) => self.if_stmt(if_stmt, "if"),
            ast::StmtKind::While { label, cond, body, else_block } => {
                let label = label.map(|l| format!("{}: ", l.name)).unwrap_or_default();
                let cond = self.condition(cond);
                self.line(&format!("{label}while {cond}:"));
                self.block(body);
                if let Some(else_block) = else_block {
                    self.line("else:");
                    self.block(else_block);
                }
            }
            ast::StmtKind::For { label, pattern, iter, body, else_block } => {
                let label = label.map(|l| format!("{}: ", l.name)).unwrap_or_default();
                let pattern = self.pattern(pattern);
                let iter = self.expr(iter);
                self.line(&format!("{label}for {pattern} in {iter}:"));
                self.block(body);
                if let Some(else_block) = else_block {
                    self.line("else:");
                    self.block(else_block);
                }
            }
            ast::StmtKind::Match { scrutinee, arms } => {
                let scrutinee = self.expr(scrutinee);
                self.line(&format!("match {scrutinee}:"));
                self.match_arms(arms);
            }
            ast::StmtKind::With { items, body } => {
                let items: Vec<String> = items
                    .iter()
                    .map(|item| match &item.pattern {
                        Some(pattern) => {
                            format!("{} = {}", self.pattern(pattern), self.expr(&item.value))
                        }
                        None => self.expr(&item.value),
                    })
                    .collect();
                self.line(&format!("with {}:", items.join(", ")));
                self.block(body);
            }
            ast::StmtKind::Defer(block) => {
                self.line("defer:");
                self.block(block);
            }
            ast::StmtKind::Break { label } => {
                let label = label.map(|l| format!(" {}", l.name)).unwrap_or_default();
                self.line(&format!("break{label}"));
            }
            ast::StmtKind::Continue { label } => {
                let label = label.map(|l| format!(" {}", l.name)).unwrap_or_default();
                self.line(&format!("continue{label}"));
            }
            // Anything the printer does not reshape is copied through.
            _ => {
                let text = self.text(stmt.span).to_string();
                self.skip_comments_in(stmt.span);
                for line in text.lines() {
                    self.line(line.trim());
                }
            }
        }
    }

    fn if_stmt(&mut self, if_stmt: &ast::IfStmt, keyword: &str) {
        let cond = self.condition(&if_stmt.cond);
        self.line(&format!("{keyword} {cond}:"));
        self.block(&if_stmt.then_block);
        match if_stmt.else_block.as_deref() {
            Some(ast::ElseBranch::Block(block)) => {
                self.line("else:");
                self.block(block);
            }
            // `elif` prints as `elif`, not as a nested `else: if`.
            Some(ast::ElseBranch::If(nested)) => self.if_stmt(nested, "elif"),
            None => {}
        }
    }

    fn condition(&mut self, cond: &ast::Condition) -> String {
        match cond {
            ast::Condition::Expr(expr) => self.expr(expr),
            ast::Condition::Pattern { pattern, value } => {
                format!("{} = {}", self.pattern(pattern), self.expr(value))
            }
        }
    }

    fn match_arms(&mut self, arms: &[ast::MatchArm]) {
        self.depth += 1;
        for arm in arms {
            self.comments_before(arm.span);
            let pattern = self.pattern(&arm.pattern);
            let guard = arm
                .guard
                .as_ref()
                .map(|g| format!(" if {}", self.expr(g)))
                .unwrap_or_default();
            match &arm.body {
                ast::MatchArmBody::Block(block) => {
                    self.line(&format!("{pattern}{guard}:"));
                    self.block(block);
                }
                ast::MatchArmBody::Expr(expr) => {
                    let value = self.expr(expr);
                    self.line(&format!("{pattern}{guard} => {value}"));
                }
            }
        }
        self.depth -= 1;
    }

    // -- expressions, types, patterns ----------------------------------------

    /// Expressions are printed on one line. Anything the printer has no rule
    /// for is copied from the source, so `parse(fmt(x)) ≡ parse(x)` holds for
    /// the whole language rather than only the part with rules.
    fn expr(&mut self, expr: &ast::Expr) -> String {
        self.skip_comments_in(expr.span);
        self.expr_pure(expr)
    }

    fn expr_pure(&self, expr: &ast::Expr) -> String {
        match &expr.kind {
            ast::ExprKind::Paren(inner) => format!("({})", self.expr_pure(inner)),
            ast::ExprKind::Path { segments } => dotted(segments),
            ast::ExprKind::SelfExpr => "self".to_string(),
            ast::ExprKind::Field { base, name } => {
                format!("{}.{}", self.expr_pure(base), name.name)
            }
            ast::ExprKind::TupleField { base, index } => {
                format!("{}.{index}", self.expr_pure(base))
            }
            ast::ExprKind::Call { callee, args } => {
                let args: Vec<String> = args.iter().map(|a| self.arg(a)).collect();
                format!("{}({})", self.expr_pure(callee), args.join(", "))
            }
            ast::ExprKind::MethodCall { recv, name, args, .. } => {
                let args: Vec<String> = args.iter().map(|a| self.arg(a)).collect();
                format!("{}.{}({})", self.expr_pure(recv), name.name, args.join(", "))
            }
            ast::ExprKind::IndexOrInstantiate { base, args } => {
                let args: Vec<String> = args.iter().map(|a| self.expr_pure(a)).collect();
                format!("{}[{}]", self.expr_pure(base), args.join(", "))
            }
            ast::ExprKind::Unary { op, operand } => {
                let op = match op {
                    ast::UnOp::Neg => "-",
                    ast::UnOp::Not => "not ",
                    ast::UnOp::BitNot => "~",
                };
                format!("{op}{}", self.expr_pure(operand))
            }
            // `[FMT-1]` — spaces around binary operators, none around `**`
            // when both operands are atoms.
            ast::ExprKind::Binary { op, lhs, rhs } => {
                let left = self.expr_pure(lhs);
                let right = self.expr_pure(rhs);
                if *op == ast::BinOp::Pow && is_atom(lhs) && is_atom(rhs) {
                    format!("{left}**{right}")
                } else {
                    format!("{left} {} {right}", op.as_str())
                }
            }
            ast::ExprKind::Logical { op, lhs, rhs } => {
                let op = match op {
                    ast::LogicalOp::And => "and",
                    ast::LogicalOp::Or => "or",
                };
                format!("{} {op} {}", self.expr_pure(lhs), self.expr_pure(rhs))
            }
            ast::ExprKind::Ternary { then_expr, cond, else_expr } => format!(
                "{} if {} else {}",
                self.expr_pure(then_expr),
                self.expr_pure(cond),
                self.expr_pure(else_expr)
            ),
            ast::ExprKind::Range { lo, hi, inclusive } => {
                let lo = lo.as_ref().map(|e| self.expr_pure(e)).unwrap_or_default();
                let hi = hi.as_ref().map(|e| self.expr_pure(e)).unwrap_or_default();
                let dots = if *inclusive { "..=" } else { ".." };
                format!("{lo}{dots}{hi}")
            }
            ast::ExprKind::Cast { expr: inner, ty } => {
                format!("{} as {}", self.expr_pure(inner), self.type_expr(ty))
            }
            ast::ExprKind::Downcast { expr: inner, ty, forced } => {
                let mark = if *forced { "!" } else { "?" };
                format!("{} as{mark} {}", self.expr_pure(inner), self.type_expr(ty))
            }
            ast::ExprKind::Try(inner) => format!("{}?", self.expr_pure(inner)),
            ast::ExprKind::Tuple(items) => {
                let items: Vec<String> = items.iter().map(|e| self.expr_pure(e)).collect();
                // A one-element tuple keeps its comma, or it is just a paren.
                if items.len() == 1 {
                    format!("({},)", items[0])
                } else {
                    format!("({})", items.join(", "))
                }
            }
            ast::ExprKind::ArrayLit(items) => {
                let items: Vec<String> = items.iter().map(|e| self.expr_pure(e)).collect();
                format!("[{}]", items.join(", "))
            }
            ast::ExprKind::ArrayRepeat { value, count } => {
                format!("[{}; {}]", self.expr_pure(value), self.expr_pure(count))
            }
            ast::ExprKind::RefOf { mutable, place } => {
                let kind = if *mutable { "ref mut " } else { "ref " };
                format!("{kind}{}", self.expr_pure(place))
            }
            // A literal, an f-string, a lambda or a `match` expression is
            // reproduced from the source: its spelling is already canonical,
            // and re-printing one risks changing what it means.
            _ => self.text(expr.span).trim().to_string(),
        }
    }

    fn arg(&self, arg: &ast::Arg) -> String {
        match arg.name {
            Some(name) => format!("{}={}", name.name, self.expr_pure(&arg.value)),
            None => self.expr_pure(&arg.value),
        }
    }

    fn type_expr(&self, ty: &ast::TypeExpr) -> String {
        match &ty.kind {
            ast::TypeKind::Void => "void".to_string(),
            ast::TypeKind::Never => "!".to_string(),
            ast::TypeKind::Ref { mutable, inner } => {
                let kind = if *mutable { "ref mut " } else { "ref " };
                format!("{kind}{}", self.type_expr(inner))
            }
            ast::TypeKind::Ptr { mutable, inner } => {
                let kind = if *mutable { "*mut " } else { "*" };
                format!("{kind}{}", self.type_expr(inner))
            }
            ast::TypeKind::Tuple(items) => {
                let items: Vec<String> = items.iter().map(|t| self.type_expr(t)).collect();
                format!("({})", items.join(", "))
            }
            ast::TypeKind::Array { elem, len } => {
                format!("[{}; {}]", self.type_expr(elem), self.expr_pure(len))
            }
            ast::TypeKind::Path { segments, args } => {
                let path = dotted(segments);
                if args.is_empty() {
                    return path;
                }
                let args: Vec<String> = args
                    .iter()
                    .map(|arg| match arg {
                        ast::GenericArg::Type(t) => self.type_expr(t),
                        ast::GenericArg::Const(e) => self.expr_pure(e),
                        ast::GenericArg::Assoc { name, ty } => {
                            format!("{} = {}", name.name, self.type_expr(ty))
                        }
                    })
                    .collect();
                format!("{path}[{}]", args.join(", "))
            }
            _ => self.text(ty.span).trim().to_string(),
        }
    }

    fn pattern(&self, pattern: &ast::Pattern) -> String {
        match &pattern.kind {
            ast::PatternKind::Wild => "_".to_string(),
            ast::PatternKind::Bind { name, by_ref, mutable, sub } => {
                let by_ref = if *by_ref { "ref " } else { "" };
                let mutable = if *mutable { "mut " } else { "" };
                match sub {
                    Some(sub) => format!("{by_ref}{mutable}{} @ {}", name.name, self.pattern(sub)),
                    None => format!("{by_ref}{mutable}{}", name.name),
                }
            }
            ast::PatternKind::Path { segments } => dotted(segments),
            ast::PatternKind::Constructor { path, fields, has_rest } => {
                let mut items: Vec<String> = fields
                    .iter()
                    .map(|f| match f.name {
                        Some(name) => format!("{}={}", name.name, self.pattern(&f.pattern)),
                        None => self.pattern(&f.pattern),
                    })
                    .collect();
                if *has_rest {
                    items.push("..".to_string());
                }
                format!("{}({})", dotted(path), items.join(", "))
            }
            ast::PatternKind::Tuple(items) => {
                let items: Vec<String> = items.iter().map(|p| self.pattern(p)).collect();
                format!("({})", items.join(", "))
            }
            ast::PatternKind::Or(items) => {
                let items: Vec<String> = items.iter().map(|p| self.pattern(p)).collect();
                items.join(" | ")
            }
            _ => self.text(pattern.span).trim().to_string(),
        }
    }
}

fn visibility(vis: &ast::Visibility) -> String {
    match vis.kind {
        ast::VisKind::Private => String::new(),
        ast::VisKind::Package => "pub(package) ".to_string(),
        ast::VisKind::Public => "pub ".to_string(),
    }
}

fn dotted(segments: &[ast::Ident]) -> String {
    segments.iter().map(|s| s.name.to_string()).collect::<Vec<_>>().join(".")
}

/// `[FMT-1]` — `**` binds tightly enough to lose its spaces when both sides
/// are atoms: `x**2`, but `(a + b) ** c`.
fn is_atom(expr: &ast::Expr) -> bool {
    matches!(
        expr.kind,
        ast::ExprKind::Lit(_)
            | ast::ExprKind::Path { .. }
            | ast::ExprKind::SelfExpr
            | ast::ExprKind::Field { .. }
            | ast::ExprKind::TupleField { .. }
    )
}

/// `[FMT-1]` — `std` first, then dependencies, then local imports.
fn import_rank(path: &str) -> u8 {
    if path == "std" || path.starts_with("std.") {
        0
    } else if path.contains('.') {
        1
    } else {
        2
    }
}
