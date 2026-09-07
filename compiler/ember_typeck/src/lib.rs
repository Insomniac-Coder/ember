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

use std::collections::HashMap;

use ember_ast as ast;
use ember_diag::{Diagnostic, Sink, codes};
use ember_hir::{
    BinOp, Block, Builtin, DefId, Expr, ExprKind, Function, LocalDecl, LocalId, Mode, Param,
    Program, Stmt, UnOp,
};
use ember_span::{Span, Symbol};
use ember_types::{CommonTypes, FieldDef, StructDef, StructId, Ty, TyKind, TypeTable, int_max};

pub fn check(module: &ast::Module, types: &mut TypeTable, common: &CommonTypes, sink: &mut Sink) -> Program {
    let mut checker = Checker::new(types, common, sink);
    checker.collect(module);
    checker.check_bodies(module)
}

struct Signature {
    params: Vec<(Symbol, Ty, Mode, Span)>,
    ret: Ty,
}

struct Checker<'a> {
    types: &'a mut TypeTable,
    common: &'a CommonTypes,
    sink: &'a mut Sink,
    struct_ids: HashMap<Symbol, StructId>,
    struct_types: HashMap<Symbol, Ty>,
    fn_ids: HashMap<Symbol, DefId>,
    signatures: Vec<Signature>,

    // Per-function state.
    locals: Vec<LocalDecl>,
    scopes: Vec<HashMap<Symbol, LocalId>>,
    ret_ty: Ty,
}

impl<'a> Checker<'a> {
    fn new(types: &'a mut TypeTable, common: &'a CommonTypes, sink: &'a mut Sink) -> Checker<'a> {
        let ret_ty = common.void;
        Checker {
            types,
            common,
            sink,
            struct_ids: HashMap::new(),
            struct_types: HashMap::new(),
            fn_ids: HashMap::new(),
            signatures: Vec::new(),
            locals: Vec::new(),
            scopes: Vec::new(),
            ret_ty,
        }
    }

    fn error(&mut self, code: ember_diag::Code, span: Span, message: impl Into<String>) {
        self.sink.emit(Diagnostic::error(code, span, message));
    }

    // -- collection ---------------------------------------------------------

    /// Two passes over the items: names first, so that a struct may refer to a
    /// struct declared later in the file, then fields and signatures
    /// (Part XVIII §4.4 step 1).
    fn collect(&mut self, module: &ast::Module) {
        for item in &module.items {
            if let ast::ItemKind::Struct(decl) = &item.kind {
                let name = decl.name.name;
                if self.struct_ids.contains_key(&name) {
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
                self.struct_types.insert(name, ty);
            }
        }

        for item in &module.items {
            match &item.kind {
                ast::ItemKind::Struct(decl) => {
                    let Some(&id) = self.struct_ids.get(&decl.name.name) else { continue };
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
                        let ty = self.struct_types[&decl.name.name];
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
                ast::ItemKind::Fn(decl) => {
                    let name = decl.name.name;
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
            ast::TypeKind::Path { segments, args } if args.is_empty() && segments.len() == 1 => {
                let name = segments[0].name;
                if let Some(ty) = self.scalar_named(name.as_str()) {
                    return ty;
                }
                if let Some(&ty) = self.struct_types.get(&name) {
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
            let Some(&def) = self.fn_ids.get(&decl.name.name) else { continue };

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
                let local = self.declare(Some(name), ty, span);
                params.push(Param { local, mode });
            }

            let body = match &decl.body {
                Some(block) => self.check_block(block),
                None => Block { stmts: Vec::new(), span: item.span },
            };

            let name = decl.name.name;
            if name.is("main") {
                main = Some(def);
            }
            functions.push(Function {
                def,
                name,
                symbol: mangle(name, name.is("main")),
                params,
                locals: std::mem::take(&mut self.locals),
                ret: self.ret_ty,
                body,
                span: item.span,
            });
        }

        Program { functions, main }
    }

    fn declare(&mut self, name: Option<Symbol>, ty: Ty, span: Span) -> LocalId {
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(LocalDecl { name, ty, span });
        if let Some(name) = name {
            self.scopes.last_mut().expect("a scope is open").insert(name, id);
        }
        id
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
            ast::StmtKind::While { cond, body, else_block, .. } => {
                if else_block.is_some() {
                    self.error(
                        codes::E1010,
                        stmt.span,
                        "a loop `else` is not supported yet in this phase",
                    );
                }
                let cond = self.check_condition(cond);
                let body = self.check_block(body);
                out.push(Stmt::While { cond, body });
            }
            ast::StmtKind::Break { .. } => out.push(Stmt::Break),
            ast::StmtKind::Continue { .. } => out.push(Stmt::Continue),
            _ => {
                self.error(
                    codes::E1010,
                    stmt.span,
                    "this statement is not supported yet in this phase of the compiler",
                );
            }
        }
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

            ast::ExprKind::Path { segments } if segments.len() == 1 => {
                let name = segments[0].name;
                if let Some(local) = self.lookup(name) {
                    let ty = self.locals[local.0 as usize].ty;
                    return Expr { ty, kind: ExprKind::Local(local), span };
                }
                self.error(codes::E1010, span, format!("cannot find `{name}` in this scope"));
                Expr { ty: self.common.error, kind: ExprKind::Error, span }
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

            ast::ExprKind::Call { callee, args } => self.synth_call(callee, args, span),

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

    fn synth_call(&mut self, callee: &ast::Expr, args: &[ast::Arg], span: Span) -> Expr {
        let ast::ExprKind::Path { segments } = &callee.kind else {
            self.error(codes::E1010, span, "only direct calls are supported in this phase");
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };
        if segments.len() != 1 {
            self.error(codes::E1010, span, "qualified calls are not supported yet in this phase");
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        }
        let name = segments[0].name;

        // `Vec3(1, 2, 3)` — the synthesised memberwise constructor (`[STR-1]`).
        if let Some(&id) = self.struct_ids.get(&name) {
            return self.synth_struct_literal(id, name, args, span);
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

        let Some(&def) = self.fn_ids.get(&name) else {
            self.error(codes::E1010, segments[0].span, format!("cannot find `{name}` in this scope"));
            return Expr { ty: self.common.error, kind: ExprKind::Error, span };
        };

        let signature: Vec<Ty> =
            self.signatures[def.0 as usize].params.iter().map(|(_, t, _, _)| *t).collect();
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
            .map(|(arg, &param_ty)| self.check_expr(&arg.value, param_ty))
            .collect();
        Expr { ty: ret, kind: ExprKind::Call { callee: def, args: checked }, span }
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

/// `[MNG-1]` — `em_<pkg>_<module>_<item>`. Phase 0 compiles one module at a
/// time with no package graph, so the package and module segments are fixed
/// until `ember_build` supplies them.
fn mangle(name: Symbol, is_main: bool) -> String {
    if is_main {
        // The C entry point calls this; `[MNG-2]` reserves the plain name.
        return "em_main".to_string();
    }
    format!("em_{name}")
}
