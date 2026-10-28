//! `[CORO-1]`–`[CORO-13]`, `[GRM-21]` — a `gen fn`, written out before checking
//! as the ordinary declarations it stands for (ADR-193).
//!
//! ```text
//! gen fn countdown(n: int) -> Generator[int]:     @no_derive(Clone, Eq, Debug)
//!     i = n                                       struct countdown.Generator:          (the frame)
//!     while i > 0:                                    n: ref int                     (settled: `int`)
//!         yield i                                 extend countdown.Generator implements Iterator:
//!         i -= 1                                      type Item = int
//!                                                     fn next(mut self) -> Option[int]: return None
//!                                                 extend countdown.Generator:
//!                                                     fn drop(mut self): pass
//!                                                     fn body(mut self, n: int):
//!                                                         std.core.__coroutine_start()
//!                                                         i = n
//!                                                         while i > 0:
//!                                                             std.core.__coroutine_yield[int](i)
//!                                                             i -= 1
//!                                                 fn countdown(n: int) -> countdown.Generator:
//!                                                     return countdown.Generator(n)
//! ```
//!
//! Calling `countdown` builds the frame and runs none of the body (`[CORO-1]`):
//! the frame type is the function's own (`[CORO-3]`), and implements `Iterator`
//! through a `next` the checker sees as a stub. The body is checked as the
//! function it is, returning `R`, with each `yield e` a call taking `e` as a
//! `Y`; it is a method of the frame, so a generic generator's is made with each
//! frame its constructor makes, and it is never called: after the borrow check
//! the coroutine transform (`ember_analysis::coroutines`) makes the stubs' real
//! bodies of it, a state machine over the frame, and makes the constructor's
//! literal the frame's first state. A frame field for a borrowed parameter is
//! written `ref T` and settled once `T` is known
//! (`TypeTable::settle_coroutine_frame`). A generator method's `self` is the
//! body's `__self`, and `Self` its type. `yield` anywhere else is left for the
//! checker, which reports it (`E2220`).

use std::collections::HashSet;

use ember_ast as ast;
use ember_diag::{Diagnostic, Sink, codes};
use ember_span::{Span, Symbol};
use ember_types::FrameParam;

use crate::{LoadedModule, has_attribute};

/// The modules with every `gen fn` written out, and each frame struct's
/// qualified name with how each of its parameter fields' parameters is passed;
/// each generator written in a block, by its name's span, as it was lifted.
pub(crate) struct Desugared {
    pub(crate) modules: Vec<LoadedModule>,
    pub(crate) frames: Vec<(Symbol, Vec<FrameParam>)>,
    pub(crate) locals: Vec<(Span, LocalLift)>,
}

/// `[GRM-28]`, `[GRM-21]` (ODR-268) — a generator written in a block, lifted to its module twice,
/// generic over what its calls pass: `count.local.env[…, ·E](owned ·env: ·E, n: int)`, whose `·E`
/// is the environment each call makes, `ref` or `ref mut` of each place its body captures as the
/// declaration's check finds (`[CLO-2]`: `ref mut` what it writes), and
/// `count.local.plain[…, ·E](n: int)` for one that captures nothing, whose `·E` is phantom (the
/// calls give `void`). Each takes the enclosing declarations'
/// type parameters first, by name. The checker makes an instance of one of them per call; neither
/// is checked on its own (`Checker::local_frames`).
#[derive(Clone, Debug)]
pub(crate) struct LocalLift {
    /// The local generator's name in its block.
    pub(crate) name: Symbol,
    /// The two constructors' names in the module, unqualified.
    pub(crate) env_fn: Symbol,
    pub(crate) plain_fn: Symbol,
    /// Their frames' qualified names.
    pub(crate) env_frame: Symbol,
    pub(crate) plain_frame: Symbol,
    /// The enclosing declarations' type parameters, which come first.
    pub(crate) enclosing: Vec<Symbol>,
}

/// The type parameter a lifted local generator takes for its environment, and the parameter
/// holding it: names no program can write.
pub(crate) const LOCAL_ENV_TYPE: &str = "\u{b7}E";
pub(crate) const LOCAL_ENV_PARAM: &str = "\u{b7}env";

/// The suspension points the coroutine transform rewrites (`ember_mir::lower::coroutines`).
const START: &str = "std.core.__coroutine_start";
const YIELD: &str = "std.core.__coroutine_yield";
const RESUMED: &str = "std.core.__coroutine_resumed";

/// A generator method's receiver, as its body's parameter.
const RECEIVER: &str = "__self";

/// The attributes the body keeps from its `gen fn`: what is checked of a
/// generator's body (`[CORO-8]`: its contracts and effects) and how its code is
/// built, and the `@safety` of an `unsafe gen fn`, whose body is `unsafe` with
/// its obligation (`[UNS-7]`). The rest describe the function a caller calls.
const BODY_ATTRIBUTES: &[&str] = &[
    "noalloc", "nosync", "noblock", "noio", "nolock", "nopanic", "realtime", "deterministic", "overflow", "fastmath", "fp",
    "static_safe", "inline", "noinline", "cold", "hot", "safety",
];

pub(crate) fn desugar_generators(modules: &[LoadedModule], sink: &mut Sink) -> Option<Desugared> {
    let gen_member = |members: &[ast::Member]| members.iter().any(|m| matches!(&m.kind, ast::MemberKind::Fn(f) if f.is_gen));
    let wanted = |module: &ast::Module| {
        module.local_generators
            || module.items.iter().any(|item| match &item.kind {
                ast::ItemKind::Fn(decl) => decl.is_gen,
                ast::ItemKind::Struct(decl) => gen_member(&decl.members),
                ast::ItemKind::Class(decl) => gen_member(&decl.members),
                ast::ItemKind::Enum(decl) => gen_member(&decl.members),
                ast::ItemKind::Extend(decl) => gen_member(&decl.members),
                ast::ItemKind::Interface(decl) => gen_member(&decl.members),
                _ => false,
            })
    };
    if !modules.iter().any(|loaded| wanted(&loaded.module)) {
        return None;
    }
    let mut out = modules.to_vec();
    let mut frames = Vec::new();
    let mut locals = Vec::new();
    for loaded in out.iter_mut().filter(|loaded| wanted(&loaded.module)) {
        let mut writer = Writer {
            prefix: loaded.path.join("."),
            items: Vec::new(),
            frames: &mut frames,
            locals: &mut locals,
            names: HashSet::new(),
            sink: &mut *sink,
        };
        let module = std::sync::Arc::make_mut(&mut loaded.module);
        // `[GRM-28]` — the generators written in blocks first, innermost first, so a generator's
        // body holds none when it is written out.
        if module.local_generators {
            for item in &mut module.items {
                writer.lift_in_item(item);
            }
        }
        for item in &mut module.items {
            let ast::Item { attrs, vis, kind, .. } = item;
            match kind {
                ast::ItemKind::Fn(decl) if decl.is_gen => {
                    writer.generator(decl, attrs, *vis, None, None, true);
                }
                ast::ItemKind::Struct(decl) => {
                    let owner = Owner::named(decl.name, &decl.generics, &decl.where_clause);
                    writer.members(&mut decl.members, &owner);
                }
                ast::ItemKind::Class(decl) => {
                    let owner = Owner::named(decl.name, &decl.generics, &decl.where_clause);
                    writer.members(&mut decl.members, &owner);
                }
                ast::ItemKind::Enum(decl) => {
                    let owner = Owner::named(decl.name, &decl.generics, &[]);
                    writer.members(&mut decl.members, &owner);
                }
                ast::ItemKind::Extend(decl) => {
                    let owner = Owner {
                        name: target_name(&decl.target),
                        ty: decl.target.clone(),
                        generics: decl.generics.clone(),
                        where_clause: decl.where_clause.clone(),
                    };
                    writer.members(&mut decl.members, &owner);
                }
                ast::ItemKind::Interface(decl) => {
                    for member in &mut decl.members {
                        if let ast::MemberKind::Fn(function) = &mut member.kind
                            && function.is_gen
                        {
                            function.is_gen = false;
                            writer.sink.emit(
                                Diagnostic::error(codes::E0900, function.name.span, "a generator method in an interface is not supported yet")
                                    .note("each `gen fn` has a frame type of its own [CORO-3], which one interface signature cannot name; give the interface an associated type the implementations set, or return `Box[dyn Iterator[Item = …]]`"),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
        let generated = std::mem::take(&mut writer.items);
        module.items.extend(generated);
    }
    Some(Desugared { modules: out, frames, locals })
}

/// The declaration a generator method belongs to: its type, written as the
/// frame's items name it, and the parameters and bounds in scope there.
struct Owner {
    name: Symbol,
    ty: ast::TypeExpr,
    generics: Vec<ast::GenericParam>,
    where_clause: Vec<ast::Bound>,
}

impl Owner {
    fn named(name: ast::Ident, generics: &[ast::GenericParam], where_clause: &[ast::Bound]) -> Owner {
        Owner { name: name.name, ty: generic_type(name.name, generics, name.span), generics: generics.to_vec(), where_clause: where_clause.to_vec() }
    }
}

/// What a generator written in a block is inside: the type parameters of the declarations around
/// it, outermost first, with their bounds, and what `Self` names there.
#[derive(Clone, Default)]
struct Enclosing {
    generics: Vec<ast::GenericParam>,
    where_clause: Vec<ast::Bound>,
    self_ty: Option<ast::TypeExpr>,
}

struct Writer<'s> {
    /// The module's path, `a.b`, empty for the root module: the frame is
    /// named by its qualified name, which resolves wherever it is written.
    prefix: String,
    items: Vec<ast::Item>,
    frames: &'s mut Vec<(Symbol, Vec<FrameParam>)>,
    locals: &'s mut Vec<(Span, LocalLift)>,
    names: HashSet<String>,
    sink: &'s mut Sink,
}

impl Writer<'_> {
    fn members(&mut self, members: &mut [ast::Member], owner: &Owner) {
        for member in members.iter_mut() {
            let ast::Member { attrs, vis, kind, .. } = member;
            if let ast::MemberKind::Fn(decl) = kind
                && decl.is_gen
            {
                self.generator(decl, attrs, *vis, Some(owner), None, true);
            }
        }
    }

    /// `[GRM-28]` — lift each generator written in a block of `item`'s functions.
    fn lift_in_item(&mut self, item: &mut ast::Item) {
        let (members, owner) = match &mut item.kind {
            ast::ItemKind::Fn(decl) => return self.lift_in_fn(decl, &Enclosing::default()),
            ast::ItemKind::Struct(decl) => (&mut decl.members, Some(Owner::named(decl.name, &decl.generics, &decl.where_clause))),
            ast::ItemKind::Class(decl) => (&mut decl.members, Some(Owner::named(decl.name, &decl.generics, &decl.where_clause))),
            ast::ItemKind::Enum(decl) => (&mut decl.members, Some(Owner::named(decl.name, &decl.generics, &[]))),
            ast::ItemKind::Extend(decl) => {
                let owner = Owner {
                    name: target_name(&decl.target),
                    ty: decl.target.clone(),
                    generics: decl.generics.clone(),
                    where_clause: decl.where_clause.clone(),
                };
                (&mut decl.members, Some(owner))
            }
            // An interface's `Self` is each implementation's, which a lifted body cannot name.
            ast::ItemKind::Interface(decl) => (&mut decl.members, None),
            _ => return,
        };
        let enclosing = owner.map_or_else(Enclosing::default, |owner| Enclosing {
            generics: owner.generics,
            where_clause: owner.where_clause,
            self_ty: Some(owner.ty),
        });
        for member in members.iter_mut() {
            if let ast::MemberKind::Fn(decl) = &mut member.kind {
                self.lift_in_fn(decl, &enclosing);
            }
        }
    }

    fn lift_in_fn(&mut self, decl: &mut ast::FnDecl, outer: &Enclosing) {
        let Some(body) = &mut decl.body else { return };
        let mut enclosing = outer.clone();
        enclosing.generics.extend(decl.generics.iter().cloned());
        enclosing.where_clause.extend(decl.where_clause.iter().cloned());
        self.lift_block(body, &mut enclosing);
    }

    fn lift_block(&mut self, block: &mut ast::Block, enclosing: &mut Enclosing) {
        for stmt in &mut block.stmts {
            self.lift_stmt(stmt, enclosing);
        }
    }

    fn lift_stmt(&mut self, stmt: &mut ast::Stmt, enclosing: &mut Enclosing) {
        match &mut stmt.kind {
            ast::StmtKind::Decl { init, .. } => {
                if let Some(init) = init {
                    self.lift_expr(init, enclosing);
                }
            }
            ast::StmtKind::Assign { targets, value, .. } => {
                for target in targets {
                    self.lift_expr(target, enclosing);
                }
                self.lift_expr(value, enclosing);
            }
            ast::StmtKind::Expr(e) => self.lift_expr(e, enclosing),
            ast::StmtKind::Pass => {}
            ast::StmtKind::If(branch) => self.lift_if(branch, enclosing),
            ast::StmtKind::While { cond, body, else_block, .. } => {
                self.lift_condition(cond, enclosing);
                self.lift_block(body, enclosing);
                if let Some(block) = else_block {
                    self.lift_block(block, enclosing);
                }
            }
            ast::StmtKind::For { iter, body, else_block, .. } => {
                self.lift_expr(iter, enclosing);
                self.lift_block(body, enclosing);
                if let Some(block) = else_block {
                    self.lift_block(block, enclosing);
                }
            }
            ast::StmtKind::Match { scrutinee, arms } => {
                self.lift_expr(scrutinee, enclosing);
                self.lift_arms(arms, enclosing);
            }
            ast::StmtKind::With { items, body } => {
                for item in items {
                    self.lift_expr(&mut item.value, enclosing);
                }
                self.lift_block(body, enclosing);
            }
            ast::StmtKind::Defer(block) | ast::StmtKind::Unsafe(block) | ast::StmtKind::Comptime(block) => {
                self.lift_block(block, enclosing);
            }
            ast::StmtKind::LocalFn(decl) => {
                let (generics, bounds) = (enclosing.generics.len(), enclosing.where_clause.len());
                enclosing.generics.extend(decl.generics.iter().cloned());
                enclosing.where_clause.extend(decl.where_clause.iter().cloned());
                if let Some(body) = &mut decl.body {
                    self.lift_block(body, enclosing);
                }
                enclosing.generics.truncate(generics);
                enclosing.where_clause.truncate(bounds);
                if decl.is_gen {
                    self.lift_local(decl, enclosing);
                }
            }
        }
    }

    fn lift_if(&mut self, branch: &mut ast::IfStmt, enclosing: &mut Enclosing) {
        self.lift_condition(&mut branch.cond, enclosing);
        self.lift_block(&mut branch.then_block, enclosing);
        match branch.else_block.as_deref_mut() {
            Some(ast::ElseBranch::Block(block)) => self.lift_block(block, enclosing),
            Some(ast::ElseBranch::If(inner)) => self.lift_if(inner, enclosing),
            None => {}
        }
    }

    fn lift_condition(&mut self, cond: &mut ast::Condition, enclosing: &mut Enclosing) {
        match cond {
            ast::Condition::Expr(e) | ast::Condition::Pattern { value: e, .. } => self.lift_expr(e, enclosing),
        }
    }

    fn lift_arms(&mut self, arms: &mut [ast::MatchArm], enclosing: &mut Enclosing) {
        for arm in arms {
            if let Some(guard) = &mut arm.guard {
                self.lift_expr(guard, enclosing);
            }
            match &mut arm.body {
                ast::MatchArmBody::Block(block) => self.lift_block(block, enclosing),
                ast::MatchArmBody::Expr(e) => self.lift_expr(e, enclosing),
            }
        }
    }

    /// The blocks an expression holds are its lambdas' and its `match` arms'.
    fn lift_expr(&mut self, e: &mut ast::Expr, enclosing: &mut Enclosing) {
        match &mut e.kind {
            ast::ExprKind::Lambda(lambda) => match &mut lambda.body {
                ast::LambdaBody::Expr(body) => self.lift_expr(body, enclosing),
                ast::LambdaBody::Block(body) => self.lift_block(body, enclosing),
            },
            ast::ExprKind::Match { scrutinee, arms } => {
                self.lift_expr(scrutinee, enclosing);
                self.lift_arms(arms, enclosing);
            }
            ast::ExprKind::Ternary { then_expr, cond, else_expr } => {
                for e in [cond, then_expr, else_expr] {
                    self.lift_expr(e, enclosing);
                }
            }
            ast::ExprKind::Yield(Some(base))
            | ast::ExprKind::Jump(ast::Jump::Return(Some(base)))
            | ast::ExprKind::Cast { expr: base, .. }
            | ast::ExprKind::Downcast { expr: base, .. }
            | ast::ExprKind::Field { base, .. }
            | ast::ExprKind::TupleField { base, .. }
            | ast::ExprKind::Try(base)
            | ast::ExprKind::Paren(base)
            | ast::ExprKind::Owned(base)
            | ast::ExprKind::Comptime(base)
            | ast::ExprKind::AssumeNoalloc(base)
            | ast::ExprKind::Unary { operand: base, .. }
            | ast::ExprKind::RefOf { place: base, .. } => self.lift_expr(base, enclosing),
            ast::ExprKind::IndexOrInstantiate { base, args } => {
                self.lift_expr(base, enclosing);
                for arg in args {
                    if let ast::TypeOrExpr::Expr(e) = arg {
                        self.lift_expr(e, enclosing);
                    }
                }
            }
            ast::ExprKind::Call { callee: base, args } | ast::ExprKind::MethodCall { recv: base, args, .. } => {
                self.lift_expr(base, enclosing);
                for arg in args {
                    self.lift_expr(&mut arg.value, enclosing);
                }
            }
            ast::ExprKind::OptChain { base, args, .. } => {
                self.lift_expr(base, enclosing);
                for arg in args.iter_mut().flatten() {
                    self.lift_expr(&mut arg.value, enclosing);
                }
            }
            ast::ExprKind::Binary { lhs, rhs, .. } | ast::ExprKind::Logical { lhs, rhs, .. } => {
                self.lift_expr(lhs, enclosing);
                self.lift_expr(rhs, enclosing);
            }
            ast::ExprKind::CompareChain { first, rest } => {
                self.lift_expr(first, enclosing);
                for (_, e) in rest {
                    self.lift_expr(e, enclosing);
                }
            }
            ast::ExprKind::Range { lo, hi, .. } => {
                for e in [lo, hi].into_iter().flatten() {
                    self.lift_expr(e, enclosing);
                }
            }
            ast::ExprKind::Tuple(items) | ast::ExprKind::ArrayLit(items) | ast::ExprKind::SetLit(items) => {
                for item in items {
                    self.lift_expr(item, enclosing);
                }
            }
            ast::ExprKind::MapLit(pairs) => {
                for (key, value) in pairs {
                    self.lift_expr(key, enclosing);
                    self.lift_expr(value, enclosing);
                }
            }
            ast::ExprKind::Comprehension { element, clauses, .. } => {
                self.lift_expr(element, enclosing);
                for clause in clauses {
                    match clause {
                        ast::CompClause::For { iter: e, .. } | ast::CompClause::If(e) => self.lift_expr(e, enclosing),
                    }
                }
            }
            ast::ExprKind::ArrayRepeat { value, count } => {
                self.lift_expr(value, enclosing);
                self.lift_expr(count, enclosing);
            }
            ast::ExprKind::FString(parts) => {
                for part in parts {
                    if let ast::FStringPart::Expr { expr: e, .. } = part {
                        self.lift_expr(e, enclosing);
                    }
                }
            }
            ast::ExprKind::Yield(None)
            | ast::ExprKind::Jump(_)
            | ast::ExprKind::Lit(_)
            | ast::ExprKind::Path { .. }
            | ast::ExprKind::SelfExpr
            | ast::ExprKind::Error => {}
        }
    }

    /// `[GRM-28]`, `[GRM-21]` (ODR-268) — a generator written in a block: its body, each `yield`
    /// the call the transform suspends at, stays where it is, for the checker to check as the
    /// named closure it is ([CLO-12]) and so find what it captures; and it is written out at module
    /// level twice (`LocalLift`), generic over the environment its calls pass and over the
    /// enclosing declarations' type parameters, `Self` written out as what it names here.
    fn lift_local(&mut self, decl: &mut ast::FnDecl, enclosing: &Enclosing) {
        let span = decl.name.span;
        let refused = match generator_types(decl.ret.as_ref()) {
            None => Some(
                Diagnostic::error(codes::E2020, decl.ret.as_ref().map_or(span, |ty| ty.span), "a `gen fn` returns `Generator[Y]` or `Generator[Y, R]`")
                    .note("`Y` is the type each `yield` produces and `R` (by default `void`) the type of the final `return` [CORO-1]"),
            ),
            Some(_) if decl.abi.is_some() => Some(
                Diagnostic::error(codes::E2222, span, "a `gen fn` cannot be `extern` or `@export`ed")
                    .note("calling a generator builds its frame, a value only Ember code resumes; C calls an ordinary function [CORO-10]"),
            ),
            Some(_) => None,
        };
        if let Some(diagnostic) = refused {
            // Reported: what is left is a local function that does nothing.
            self.sink.emit(diagnostic);
            decl.is_gen = false;
            decl.abi = None;
            decl.ret = None;
            decl.body = Some(block(vec![stmt(ast::StmtKind::Pass, span)], span));
            return;
        }
        let Some((yielded, _)) = generator_types(decl.ret.as_ref()) else { return };
        // A missing body is the parser's error; what is left is a local function without one.
        let Some(body) = &mut decl.body else {
            decl.is_gen = false;
            return;
        };
        let mut rewriter = Rewriter {
            yielded: &yielded,
            owner: None,
            receiver: false,
            count: 0,
            lambdas: 0,
            parallel: false,
            deferred: false,
            sink: &mut *self.sink,
        };
        rewriter.block(body);
        if rewriter.count == 0 {
            self.sink.emit(
                Diagnostic::lint(codes::L2004, span, format!("`gen fn {}` never yields", decl.name.name))
                    .note("a generator with no `yield` produces no items; an ordinary `fn` does the same work [LNT-4]"),
            );
        }
        let mut stem = format!("{}.local", decl.name.name);
        let mut counter = 1;
        while self.names.contains(&format!("{stem}.env")) {
            counter += 1;
            stem = format!("{}.local{counter}", decl.name.name);
        }
        let hidden = ast::GenericParam {
            id: ast::NodeId::DUMMY,
            name: ast::Ident { name: Symbol::intern(LOCAL_ENV_TYPE), span },
            bounds: Vec::new(),
            default: None,
            const_ty: None,
            span,
        };
        let mut lifted_names = Vec::new();
        for form in ["env", "plain"] {
            let name = format!("{stem}.{form}");
            self.names.insert(name.clone());
            let mut lifted = decl.clone();
            // Each callable is told apart by its span (`[BLD-2]`): the second form's are the
            // empty ones where the first's start.
            let at = |span: Span| if form == "env" { span } else { Span::new(span.file, span.start, span.start) };
            lifted.name = ast::Ident { name: Symbol::intern(&name), span: at(span) };
            if let Some(ret) = &mut lifted.ret {
                ret.span = at(ret.span);
            }
            lifted.generics = enclosing.generics.iter().cloned().chain([hidden.clone()]).chain(decl.generics.iter().cloned()).collect();
            for param in &mut lifted.generics {
                param.default = None;
            }
            lifted.where_clause = enclosing.where_clause.iter().cloned().chain(decl.where_clause.iter().cloned()).collect();
            if form == "env" {
                lifted.params.insert(
                    0,
                    ast::Param {
                        id: ast::NodeId::DUMMY,
                        mode: ast::Mode::Owned,
                        kind: ast::ParamKind::Named {
                            name: ast::Ident { name: Symbol::intern(LOCAL_ENV_PARAM), span },
                            ty: named_type(LOCAL_ENV_TYPE, span),
                        },
                        default: None,
                        span,
                    },
                );
            }
            let item_span = lifted.name.span;
            let frame = self.generator(&mut lifted, &mut Vec::new(), ast::Visibility::private(), None, enclosing.self_ty.as_ref(), false);
            lifted_names.push((Symbol::intern(&name), frame.unwrap_or(Symbol::intern(&name))));
            self.items.push(ast::Item {
                id: ast::NodeId::DUMMY,
                attrs: Vec::new(),
                vis: ast::Visibility::private(),
                doc: None,
                kind: ast::ItemKind::Fn(lifted),
                span: item_span,
            });
        }
        let [(env_fn, env_frame), (plain_fn, plain_frame)] = [lifted_names[0], lifted_names[1]];
        self.locals.push((
            span,
            LocalLift {
                name: decl.name.name,
                env_fn,
                plain_fn,
                env_frame,
                plain_frame,
                enclosing: enclosing.generics.iter().map(|param| param.name.name).collect(),
            },
        ));
    }

    /// Write out one `gen fn`: the frame, its implementations and the body
    /// become items of the module, and `decl` becomes the function building
    /// the frame, whose qualified name this gives. `self_ty` is what `Self`
    /// names where a lifted local generator was written (`lift_local`), and
    /// `lint` whether a body with no `yield` is reported here.
    fn generator(
        &mut self,
        decl: &mut ast::FnDecl,
        attrs: &mut Vec<ast::Attribute>,
        vis: ast::Visibility,
        owner: Option<&Owner>,
        self_ty: Option<&ast::TypeExpr>,
        lint: bool,
    ) -> Option<Symbol> {
        decl.is_gen = false;
        let span = decl.name.span;
        let Some((yielded, result)) = generator_types(decl.ret.as_ref()) else {
            self.sink.emit(
                Diagnostic::error(codes::E2020, decl.ret.as_ref().map_or(span, |ty| ty.span), "a `gen fn` returns `Generator[Y]` or `Generator[Y, R]`")
                    .note("`Y` is the type each `yield` produces and `R` (by default `void`) the type of the final `return` [CORO-1]"),
            );
            decl.body = None;
            decl.ret = None;
            return None;
        };
        let foreign = decl.abi.is_some() || has_attribute(attrs, "export");
        if foreign {
            self.sink.emit(
                Diagnostic::error(codes::E2222, span, "a `gen fn` cannot be `extern` or `@export`ed")
                    .note("calling a generator builds its frame, a value only Ember code resumes; C calls an ordinary function [CORO-10]"),
            );
        }
        let dispatched = decl.dispatch != ast::Dispatch::Static;
        if dispatched {
            self.sink.emit(
                Diagnostic::error(codes::E0900, span, "a `virtual` or `override` generator method is not supported yet")
                    .note("each `gen fn` has a frame type of its own [CORO-3], so an override would return another type than the method it replaces"),
            );
        }
        let typed_receiver = decl.params.iter().any(|param| matches!(param.kind, ast::ParamKind::Receiver { ty: Some(_) }));
        if typed_receiver {
            self.sink.emit(Diagnostic::error(codes::E0900, span, "a generator method with a typed receiver (`self: Box[Self]`) is not supported yet"));
        }
        if foreign || dispatched || typed_receiver {
            // Reported: what is left is a function that does nothing.
            attrs.retain(|attr| !(attr.path.len() == 1 && attr.path[0].name.is("export")));
            decl.abi = None;
            decl.dispatch = ast::Dispatch::Static;
            decl.ret = None;
            decl.body = Some(block(vec![stmt(ast::StmtKind::Pass, span)], span));
            return None;
        }
        let Some(mut body) = decl.body.take() else {
            self.sink.emit(Diagnostic::error(codes::E2020, span, "a `gen fn` needs a body"));
            decl.ret = None;
            return None;
        };

        // The frame's name: the function's, and its type's for a method. A
        // `.` is in no written name, so it is no other declaration's.
        let stem = match owner {
            Some(owner) => format!("{}.{}.Generator", owner.name, decl.name.name),
            None => format!("{}.Generator", decl.name.name),
        };
        let mut base = stem.clone();
        let mut counter = 1;
        while !self.names.insert(base.clone()) {
            counter += 1;
            base = format!("{stem}{counter}");
        }
        let qualified = match self.prefix.is_empty() {
            true => base.clone(),
            false => format!("{}.{base}", self.prefix),
        };
        let owner_ty = owner.map(|owner| &owner.ty).or(self_ty);
        let yielded = substitute_self(&yielded, owner_ty);
        let result = result.map(|ty| substitute_self(&ty, owner_ty));

        // The frame's parameters: the owner's, then the function's own.
        let mut generics: Vec<ast::GenericParam> = owner.map(|owner| owner.generics.clone()).unwrap_or_default();
        generics.extend(decl.generics.iter().cloned());
        for param in &mut generics {
            param.default = None;
        }
        let mut where_clause: Vec<ast::Bound> = owner.map(|owner| owner.where_clause.clone()).unwrap_or_default();
        where_clause.extend(decl.where_clause.iter().cloned());
        let frame_ty = generic_type(Symbol::intern(&qualified), &generics, span);

        // One field per parameter, the receiver first, each as passed; the
        // body takes the same parameters after the frame.
        let mut fields = Vec::new();
        let mut modes = Vec::new();
        let mut args = Vec::new();
        let mut body_params = vec![receiver(ast::Mode::Mut, span)];
        for param in &decl.params {
            let (name, field, ty, value) = match &param.kind {
                ast::ParamKind::Receiver { .. } => {
                    let Some(owner) = owner else { continue };
                    let name = ast::Ident { name: Symbol::intern(RECEIVER), span: param.span };
                    (name, Symbol::intern("self"), owner.ty.clone(), expr(ast::ExprKind::SelfExpr, param.span))
                }
                ast::ParamKind::Named { name, ty } => (*name, name.name, substitute_self(ty, owner_ty), path(name.name, name.span)),
            };
            let (field_ty, mode) = match param.mode {
                ast::Mode::Owned => (ty.clone(), FrameParam::Owned),
                ast::Mode::Borrow => (reference(ty.clone(), false), FrameParam::Borrowed),
                ast::Mode::Mut => (reference(ty.clone(), true), FrameParam::Mut),
            };
            fields.push(ast::Member {
                id: ast::NodeId::DUMMY,
                attrs: Vec::new(),
                vis: ast::Visibility::private(),
                read_only_outside: false,
                doc: None,
                kind: ast::MemberKind::Field(ast::FieldDecl { name: ast::Ident { name: field, span: param.span }, ty: field_ty, default: None, is_let: false }),
                span: param.span,
            });
            modes.push(mode);
            args.push(value);
            body_params.push(ast::Param { id: ast::NodeId::DUMMY, mode: param.mode, kind: ast::ParamKind::Named { name, ty }, default: None, span: param.span });
        }
        self.frames.push((Symbol::intern(&qualified), modes));
        // `[CORO-3]` — a frame is move-only, and what a copy, a comparison or
        // a printing of one would be is nothing it has.
        let no_derive = ast::Attribute {
            id: ast::NodeId::DUMMY,
            path: vec![ast::Ident { name: Symbol::intern("no_derive"), span }],
            args: ["Clone", "Eq", "Debug"].iter().map(|name| ast::AttrArg::Expr(path(Symbol::intern(name), span))).collect(),
            span,
        };
        self.items.push(ast::Item {
            id: ast::NodeId::DUMMY,
            attrs: vec![no_derive],
            vis,
            doc: None,
            kind: ast::ItemKind::Struct(ast::StructDecl {
                name: ast::Ident { name: Symbol::intern(&base), span },
                generics: generics.clone(),
                implements: Vec::new(),
                where_clause: where_clause.clone(),
                members: fields,
            }),
            span,
        });

        // `[CORO-3]` — the frame is an `Iterator[Item = Y]`; `next`, `drop`
        // and `result` are the transform's to write.
        // Each callable a module may export is told apart by its span
        // (`[BLD-2]`): `next` is the return type's, `result` the name's; the
        // rest are private.
        let option = |ty: ast::TypeExpr| generic_of("Option", vec![ty], span);
        let mut next = method("next", vec![receiver(ast::Mode::Mut, span)], Some(option(yielded.clone())), return_none(span), span);
        next.span = decl.ret.as_ref().map_or(span, |ty| ty.span);
        let item_type = ast::Member {
            id: ast::NodeId::DUMMY,
            attrs: Vec::new(),
            vis: public(),
            read_only_outside: false,
            doc: None,
            kind: ast::MemberKind::TypeAlias(ast::TypeAlias {
                name: ast::Ident { name: Symbol::intern("Item"), span },
                generics: Vec::new(),
                is_foreign_decl: false,
                value: Some(yielded.clone()),
                bounds: Vec::new(),
                range: None,
            }),
            span,
        };
        // The two `extend` blocks are told apart by their spans too (a generic
        // extension's members are declared under its place in the file).
        let ret_span = decl.ret.as_ref().map_or(span, |ty| ty.span);
        self.items.push(extend(&generics, &where_clause, frame_ty.clone(), vec![named_type("std.core.Iterator", span)], vec![item_type, next], ret_span));

        // The body, checked as the function it is.
        let mut rewriter = Rewriter {
            yielded: &yielded,
            owner: owner.map(|owner| (owner.name, &owner.ty)).or(self_ty.map(|ty| (target_name(ty), ty))),
            receiver: owner.is_some(),
            count: 0,
            lambdas: 0,
            parallel: false,
            deferred: false,
            sink: &mut *self.sink,
        };
        rewriter.block(&mut body);
        if lint && rewriter.count == 0 {
            self.sink.emit(
                Diagnostic::lint(codes::L2004, span, format!("`gen fn {}` never yields", decl.name.name))
                    .note("a generator with no `yield` produces no items; an ordinary `fn` does the same work [LNT-4]"),
            );
        }
        let start = stmt(ast::StmtKind::Expr(call(path(Symbol::intern(START), body.span), Vec::new(), body.span)), body.span);
        body.stmts.insert(0, start);
        let mut body_method = method("body", body_params, result.clone(), body, span);
        body_method.vis = ast::Visibility::private();
        body_method.attrs = attrs.iter().filter(|attr| attr.path.len() == 1 && BODY_ATTRIBUTES.iter().any(|name| attr.path[0].name.is(name))).cloned().collect();
        if let ast::MemberKind::Fn(function) = &mut body_method.kind {
            function.is_unsafe = decl.is_unsafe;
        }
        let mut drop = method("drop", vec![receiver(ast::Mode::Mut, span)], None, block(vec![stmt(ast::StmtKind::Pass, span)], span), span);
        drop.vis = ast::Visibility::private();
        let mut inherent = vec![drop];
        if let Some(result) = &result {
            let mut taken = method("result", vec![receiver(ast::Mode::Mut, span)], Some(option(result.clone())), return_none(span), span);
            taken.vis = vis;
            inherent.push(taken);
        }
        inherent.push(body_method);
        self.items.push(extend(&generics, &where_clause, frame_ty.clone(), Vec::new(), inherent, span));

        // `[CORO-1]` — calling the function builds the frame.
        let literal = call(path(Symbol::intern(&qualified), span), args, span);
        decl.ret = Some(frame_ty);
        decl.body = Some(block(vec![stmt(ast::StmtKind::Expr(expr(ast::ExprKind::Jump(ast::Jump::Return(Some(Box::new(literal)))), span)), span)], span));
        Some(Symbol::intern(&qualified))
    }
}

/// `Generator[Y]` or `Generator[Y, R]`: `(Y, Some(R))`.
pub(crate) fn generator_types(ret: Option<&ast::TypeExpr>) -> Option<(ast::TypeExpr, Option<ast::TypeExpr>)> {
    let ast::TypeKind::Path { segments, args } = &ret?.kind else { return None };
    if !(segments.len() == 1 && segments[0].name.is("Generator")) {
        return None;
    }
    let types: Vec<&ast::TypeExpr> = args
        .iter()
        .filter_map(|arg| match arg {
            ast::GenericArg::Type(ty) => Some(ty),
            _ => None,
        })
        .collect();
    match (types.as_slice(), args.len()) {
        ([yielded], 1) => Some(((*yielded).clone(), None)),
        ([yielded, result], 2) => Some(((*yielded).clone(), Some((*result).clone()))),
        _ => None,
    }
}

/// The name of the type an `extend` block extends, for its methods' frames.
fn target_name(target: &ast::TypeExpr) -> Symbol {
    match &target.kind {
        ast::TypeKind::Path { segments, .. } if !segments.is_empty() => segments[segments.len() - 1].name,
        _ => Symbol::intern("extension"),
    }
}

/// `Self` in a method's signature names its owner; in the frame's items, which
/// are the frame's, it is written out.
fn substitute_self(ty: &ast::TypeExpr, owner: Option<&ast::TypeExpr>) -> ast::TypeExpr {
    let mut ty = ty.clone();
    if let Some(owner) = owner {
        replace_self(&mut ty, owner);
    }
    ty
}

fn replace_self(ty: &mut ast::TypeExpr, owner: &ast::TypeExpr) {
    match &mut ty.kind {
        ast::TypeKind::SelfType => *ty = owner.clone(),
        ast::TypeKind::Path { segments, args } => {
            if segments.first().is_some_and(|first| first.name.is("Self")) {
                if segments.len() == 1 && args.is_empty() {
                    *ty = owner.clone();
                    return;
                }
                // `Self.Item`: the owner's, by its name.
                if let ast::TypeKind::Path { segments: owner_segments, .. } = &owner.kind
                    && let Some(last) = owner_segments.last()
                {
                    segments[0].name = last.name;
                }
            }
            for arg in args {
                match arg {
                    ast::GenericArg::Type(inner) | ast::GenericArg::Assoc { ty: inner, .. } => replace_self(inner, owner),
                    ast::GenericArg::Const(_) => {}
                }
            }
        }
        ast::TypeKind::Ref { inner, .. } | ast::TypeKind::Ptr { inner, .. } => replace_self(inner, owner),
        ast::TypeKind::Array { elem, .. } => replace_self(elem, owner),
        ast::TypeKind::Tuple(items) | ast::TypeKind::Dyn(items) => {
            for item in items {
                replace_self(item, owner);
            }
        }
        ast::TypeKind::Fn { params, ret, .. } => {
            for param in params {
                replace_self(&mut param.ty, owner);
            }
            if let Some(ret) = ret {
                replace_self(ret, owner);
            }
        }
        _ => {}
    }
}

/// Where a `yield` stands, which `[GRM-16]` decides: the whole of a
/// statement, the whole body of a `=>` arm or a conditional's branch, or an
/// operand (`E0107`).
#[derive(Copy, Clone, PartialEq, Eq)]
enum Place {
    Statement,
    Branch,
    Operand,
}

/// `[CORO-2]` — each `yield e` of a generator's own body becomes the call the
/// transform suspends at. A lambda's or a local function's body is a function
/// of its own: a `yield` there is left for the checker (`E2220`). In a
/// generator method, `self` is the body's `__self` and `Self` the owner.
struct Rewriter<'a> {
    yielded: &'a ast::TypeExpr,
    /// What `Self` names, and its name.
    owner: Option<(Symbol, &'a ast::TypeExpr)>,
    /// A generator method's: `self` is the body's `__self`. A lifted local generator's `self` is
    /// its enclosing method's, which it captures.
    receiver: bool,
    count: usize,
    /// How many lambdas and local functions deep the walk is.
    lambdas: usize,
    /// `[PAR-2]` — inside a `@parallel` loop's body.
    parallel: bool,
    /// `[CTL-7]` — inside a `defer` block.
    deferred: bool,
    sink: &'a mut Sink,
}

impl Rewriter<'_> {
    fn block(&mut self, block: &mut ast::Block) {
        for stmt in &mut block.stmts {
            self.stmt(stmt);
        }
    }

    fn ty(&mut self, ty: &mut ast::TypeExpr) {
        if let Some((_, owner)) = self.owner {
            replace_self(ty, owner);
        }
    }

    fn pattern(&mut self, pattern: &mut ast::Pattern) {
        let Some((name, _)) = self.owner else { return };
        match &mut pattern.kind {
            ast::PatternKind::Path { segments } | ast::PatternKind::Constructor { path: segments, .. } => {
                if let Some(first) = segments.first_mut()
                    && first.name.is("Self")
                {
                    first.name = name;
                }
            }
            _ => {}
        }
        match &mut pattern.kind {
            ast::PatternKind::Bind { sub: Some(sub), .. } => self.pattern(sub),
            ast::PatternKind::Constructor { fields, .. } => {
                for field in fields {
                    self.pattern(&mut field.pattern);
                }
            }
            ast::PatternKind::Tuple(items) | ast::PatternKind::Or(items) => {
                for item in items {
                    self.pattern(item);
                }
            }
            ast::PatternKind::Slice { prefix, suffix, .. } => {
                for item in prefix.iter_mut().chain(suffix.iter_mut()) {
                    self.pattern(item);
                }
            }
            _ => {}
        }
    }

    fn generic_args(&mut self, args: &mut [ast::GenericArg]) {
        for arg in args {
            match arg {
                ast::GenericArg::Type(ty) | ast::GenericArg::Assoc { ty, .. } => self.ty(ty),
                ast::GenericArg::Const(e) => self.expr(e, Place::Operand),
            }
        }
    }

    fn stmt(&mut self, stmt: &mut ast::Stmt) {
        match &mut stmt.kind {
            ast::StmtKind::Decl { pattern, ty, init } => {
                self.pattern(pattern);
                if let Some(ty) = ty {
                    self.ty(ty);
                }
                if let Some(init) = init {
                    self.expr(init, Place::Operand);
                }
            }
            ast::StmtKind::Assign { targets, value, .. } => {
                for target in targets {
                    self.expr(target, Place::Operand);
                }
                self.expr(value, Place::Operand);
            }
            ast::StmtKind::Expr(e) => self.expr(e, Place::Statement),
            ast::StmtKind::Pass => {}
            // Run while compiling, not by the generator.
            ast::StmtKind::Comptime(block) => {
                self.lambdas += 1;
                self.block(block);
                self.lambdas -= 1;
            }
            ast::StmtKind::LocalFn(function) => {
                for param in &mut function.params {
                    if let ast::ParamKind::Named { ty, .. } = &mut param.kind {
                        self.ty(ty);
                    }
                }
                if let Some(ret) = &mut function.ret {
                    self.ty(ret);
                }
                if let Some(body) = &mut function.body {
                    self.lambdas += 1;
                    self.block(body);
                    self.lambdas -= 1;
                }
            }
            ast::StmtKind::If(branch) => self.if_stmt(branch),
            ast::StmtKind::While { cond, body, else_block, .. } => {
                self.condition(cond);
                self.block(body);
                if let Some(block) = else_block {
                    self.block(block);
                }
            }
            ast::StmtKind::For { pattern, iter, body, else_block, .. } => {
                self.pattern(pattern);
                self.expr(iter, Place::Operand);
                let outer = self.parallel;
                self.parallel |= stmt.attrs.iter().any(|attr| attr.path.len() == 1 && attr.path[0].name.is("parallel"));
                self.block(body);
                self.parallel = outer;
                if let Some(block) = else_block {
                    self.block(block);
                }
            }
            ast::StmtKind::Match { scrutinee, arms } => {
                self.expr(scrutinee, Place::Operand);
                self.arms(arms);
            }
            ast::StmtKind::With { items, body } => {
                for item in items {
                    if let Some(pattern) = &mut item.pattern {
                        self.pattern(pattern);
                    }
                    self.expr(&mut item.value, Place::Operand);
                }
                self.block(body);
            }
            ast::StmtKind::Defer(block) => {
                let outer = std::mem::replace(&mut self.deferred, true);
                self.block(block);
                self.deferred = outer;
            }
            ast::StmtKind::Unsafe(block) => self.block(block),
        }
    }

    fn if_stmt(&mut self, branch: &mut ast::IfStmt) {
        self.condition(&mut branch.cond);
        self.block(&mut branch.then_block);
        match branch.else_block.as_deref_mut() {
            Some(ast::ElseBranch::Block(block)) => self.block(block),
            Some(ast::ElseBranch::If(inner)) => self.if_stmt(inner),
            None => {}
        }
    }

    fn condition(&mut self, cond: &mut ast::Condition) {
        match cond {
            ast::Condition::Expr(e) => self.expr(e, Place::Operand),
            ast::Condition::Pattern { pattern, value } => {
                self.pattern(pattern);
                self.expr(value, Place::Operand);
            }
        }
    }

    fn arms(&mut self, arms: &mut [ast::MatchArm]) {
        for arm in arms {
            self.pattern(&mut arm.pattern);
            if let Some(guard) = &mut arm.guard {
                self.expr(guard, Place::Operand);
            }
            match &mut arm.body {
                ast::MatchArmBody::Block(block) => self.block(block),
                ast::MatchArmBody::Expr(e) => self.expr(e, Place::Branch),
            }
        }
    }

    fn args(&mut self, args: &mut [ast::Arg]) {
        for arg in args {
            self.expr(&mut arg.value, Place::Operand);
        }
    }

    fn expr(&mut self, e: &mut ast::Expr, place: Place) {
        let span = e.span;
        match &mut e.kind {
            ast::ExprKind::Yield(_) if self.lambdas > 0 => {
                if let ast::ExprKind::Yield(Some(value)) = &mut e.kind {
                    self.expr(value, Place::Operand);
                }
            }
            ast::ExprKind::Yield(value) => {
                if let Some(value) = value {
                    self.expr(value, Place::Operand);
                }
                let refused = if self.parallel {
                    Some(Diagnostic::error(codes::E7010, span, "a `@parallel` loop's body may not `yield`").note("each chunk of the loop runs on its own, and none can suspend the generator [PAR-2]"))
                } else if self.deferred {
                    Some(Diagnostic::error(codes::E2160, span, "a `defer` block cannot `yield`").note("a deferred block runs as its scope ends, which a suspension would leave half done [CTL-7]"))
                } else if place == Place::Operand {
                    Some(
                        Diagnostic::error(codes::E0107, span, "`yield` must be a whole statement")
                            .primary_label("`yield` is a jump expression, of type `void`")
                            .help("write `yield e` on a line of its own [GRM-16]"),
                    )
                } else {
                    None
                };
                if let Some(diagnostic) = refused {
                    self.sink.emit(diagnostic);
                    e.kind = ast::ExprKind::Tuple(Vec::new());
                    return;
                }
                self.count += 1;
                let value = match value.take() {
                    Some(value) => *value,
                    None => expr(ast::ExprKind::Tuple(Vec::new()), span),
                };
                let callee = expr(
                    ast::ExprKind::IndexOrInstantiate {
                        base: Box::new(path(Symbol::intern(YIELD), span)),
                        args: vec![ast::TypeOrExpr::Type(self.yielded.clone())],
                    },
                    span,
                );
                let suspend = call(callee, vec![value], span);
                // As a statement its answer is dropped; elsewhere the `yield`
                // is a `void` expression (`[CORO-3]`).
                *e = match place {
                    Place::Statement => suspend,
                    _ => call(path(Symbol::intern(RESUMED), span), vec![suspend], span),
                };
            }
            ast::ExprKind::SelfExpr => {
                if self.receiver {
                    e.kind = ast::ExprKind::Path { segments: vec![ast::Ident { name: Symbol::intern(RECEIVER), span }] };
                }
            }
            ast::ExprKind::Path { segments } => {
                if let Some((name, _)) = self.owner
                    && let Some(first) = segments.first_mut()
                    && first.name.is("Self")
                {
                    first.name = name;
                }
            }
            ast::ExprKind::Lambda(lambda) => {
                for param in &mut lambda.params {
                    if let ast::ParamKind::Named { ty, .. } = &mut param.kind {
                        self.ty(ty);
                    }
                }
                if let Some(ret) = &mut lambda.ret {
                    self.ty(ret);
                }
                self.lambdas += 1;
                match &mut lambda.body {
                    ast::LambdaBody::Expr(body) => self.expr(body, Place::Branch),
                    ast::LambdaBody::Block(body) => self.block(body),
                }
                self.lambdas -= 1;
            }
            ast::ExprKind::Ternary { then_expr, cond, else_expr } => {
                self.expr(cond, Place::Operand);
                self.expr(then_expr, Place::Branch);
                self.expr(else_expr, Place::Branch);
            }
            ast::ExprKind::Match { scrutinee, arms } => {
                self.expr(scrutinee, Place::Operand);
                self.arms(arms);
            }
            ast::ExprKind::Jump(ast::Jump::Return(Some(value))) => self.expr(value, Place::Operand),
            ast::ExprKind::Lit(_) | ast::ExprKind::Jump(_) | ast::ExprKind::Error => {}
            ast::ExprKind::Cast { expr: base, ty } | ast::ExprKind::Downcast { expr: base, ty, .. } => {
                self.expr(base, Place::Operand);
                self.ty(ty);
            }
            ast::ExprKind::Field { base, .. }
            | ast::ExprKind::TupleField { base, .. }
            | ast::ExprKind::Try(base)
            | ast::ExprKind::Paren(base)
            | ast::ExprKind::Owned(base)
            | ast::ExprKind::Comptime(base)
            | ast::ExprKind::AssumeNoalloc(base)
            | ast::ExprKind::Unary { operand: base, .. }
            | ast::ExprKind::RefOf { place: base, .. } => self.expr(base, Place::Operand),
            ast::ExprKind::IndexOrInstantiate { base, args } => {
                self.expr(base, Place::Operand);
                for arg in args {
                    match arg {
                        ast::TypeOrExpr::Expr(e) => self.expr(e, Place::Operand),
                        ast::TypeOrExpr::Type(ty) | ast::TypeOrExpr::Binding { ty, .. } => self.ty(ty),
                    }
                }
            }
            ast::ExprKind::Call { callee, args } => {
                self.expr(callee, Place::Operand);
                self.args(args);
            }
            ast::ExprKind::MethodCall { recv, generic_args, args, .. } => {
                self.expr(recv, Place::Operand);
                self.generic_args(generic_args);
                self.args(args);
            }
            ast::ExprKind::OptChain { base, generic_args, args, .. } => {
                self.expr(base, Place::Operand);
                self.generic_args(generic_args);
                if let Some(args) = args {
                    self.args(args);
                }
            }
            ast::ExprKind::Binary { lhs, rhs, .. } | ast::ExprKind::Logical { lhs, rhs, .. } => {
                self.expr(lhs, Place::Operand);
                self.expr(rhs, Place::Operand);
            }
            ast::ExprKind::CompareChain { first, rest } => {
                self.expr(first, Place::Operand);
                for (_, e) in rest {
                    self.expr(e, Place::Operand);
                }
            }
            ast::ExprKind::Range { lo, hi, .. } => {
                for e in [lo, hi].into_iter().flatten() {
                    self.expr(e, Place::Operand);
                }
            }
            ast::ExprKind::Tuple(items) | ast::ExprKind::ArrayLit(items) | ast::ExprKind::SetLit(items) => {
                for item in items {
                    self.expr(item, Place::Operand);
                }
            }
            ast::ExprKind::MapLit(pairs) => {
                for (key, value) in pairs {
                    self.expr(key, Place::Operand);
                    self.expr(value, Place::Operand);
                }
            }
            ast::ExprKind::Comprehension { element, clauses, .. } => {
                self.expr(element, Place::Operand);
                for clause in clauses {
                    match clause {
                        ast::CompClause::For { pattern, iter } => {
                            self.pattern(pattern);
                            self.expr(iter, Place::Operand);
                        }
                        ast::CompClause::If(cond) => self.expr(cond, Place::Operand),
                    }
                }
            }
            ast::ExprKind::ArrayRepeat { value, count } => {
                self.expr(value, Place::Operand);
                self.expr(count, Place::Operand);
            }
            ast::ExprKind::FString(parts) => {
                for part in parts {
                    if let ast::FStringPart::Expr { expr: e, .. } = part {
                        self.expr(e, Place::Operand);
                    }
                }
            }
        }
    }
}

// -- building the tree ----------------------------------------------------

fn expr(kind: ast::ExprKind, span: Span) -> ast::Expr {
    ast::Expr { id: ast::NodeId::DUMMY, kind, span }
}

fn path(name: Symbol, span: Span) -> ast::Expr {
    expr(ast::ExprKind::Path { segments: vec![ast::Ident { name, span }] }, span)
}

fn call(callee: ast::Expr, args: Vec<ast::Expr>, span: Span) -> ast::Expr {
    let args = args.into_iter().map(|value| ast::Arg { name: None, span: value.span, value }).collect();
    expr(ast::ExprKind::Call { callee: Box::new(callee), args }, span)
}

fn stmt(kind: ast::StmtKind, span: Span) -> ast::Stmt {
    ast::Stmt { id: ast::NodeId::DUMMY, attrs: Vec::new(), kind, span }
}

fn block(stmts: Vec<ast::Stmt>, span: Span) -> ast::Block {
    ast::Block { id: ast::NodeId::DUMMY, stmts, span }
}

fn return_none(span: Span) -> ast::Block {
    let none = path(Symbol::intern("None"), span);
    block(vec![stmt(ast::StmtKind::Expr(expr(ast::ExprKind::Jump(ast::Jump::Return(Some(Box::new(none)))), span)), span)], span)
}

fn public() -> ast::Visibility {
    ast::Visibility { kind: ast::VisKind::Public, span: Span::DUMMY }
}

fn named_type(name: &str, span: Span) -> ast::TypeExpr {
    generic_of(name, Vec::new(), span)
}

fn generic_of(name: &str, args: Vec<ast::TypeExpr>, span: Span) -> ast::TypeExpr {
    ast::TypeExpr {
        id: ast::NodeId::DUMMY,
        kind: ast::TypeKind::Path {
            segments: vec![ast::Ident { name: Symbol::intern(name), span }],
            args: args.into_iter().map(ast::GenericArg::Type).collect(),
        },
        span,
    }
}

/// `Name` or `Name[T, U]` over the declaration's own parameters.
fn generic_type(name: Symbol, generics: &[ast::GenericParam], span: Span) -> ast::TypeExpr {
    let args = generics.iter().map(|param| named_type(param.name.name.as_str(), param.span)).collect();
    generic_of(name.as_str(), args, span)
}

fn reference(ty: ast::TypeExpr, mutable: bool) -> ast::TypeExpr {
    let span = ty.span;
    ast::TypeExpr { id: ast::NodeId::DUMMY, kind: ast::TypeKind::Ref { mutable, inner: Box::new(ty) }, span }
}

fn receiver(mode: ast::Mode, span: Span) -> ast::Param {
    ast::Param { id: ast::NodeId::DUMMY, mode, kind: ast::ParamKind::Receiver { ty: None }, default: None, span }
}

/// `fn name(params) [-> ret]: body`.
fn method(name: &str, params: Vec<ast::Param>, ret: Option<ast::TypeExpr>, body: ast::Block, span: Span) -> ast::Member {
    ast::Member {
        id: ast::NodeId::DUMMY,
        attrs: Vec::new(),
        vis: public(),
        read_only_outside: false,
        doc: None,
        kind: ast::MemberKind::Fn(ast::FnDecl {
            name: ast::Ident { name: Symbol::intern(name), span },
            is_unsafe: false,
            is_safe: false,
            is_foreign_decl: false,
            abi: None,
            is_gen: false,
            dispatch: ast::Dispatch::Static,
            generics: Vec::new(),
            params,
            ret,
            where_clause: Vec::new(),
            body: Some(body),
        }),
        span,
    }
}

fn extend(
    generics: &[ast::GenericParam],
    where_clause: &[ast::Bound],
    target: ast::TypeExpr,
    implements: Vec<ast::TypeExpr>,
    members: Vec<ast::Member>,
    span: Span,
) -> ast::Item {
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
            where_clause: where_clause.to_vec(),
            members,
            blanket: Vec::new(),
        }),
        span,
    }
}

/// `[CORO-10]` — what the checker says of a generator given where a C
/// function pointer is wanted: its frame is no C value.
pub(crate) fn given_to_c(span: Span) -> Diagnostic {
    Diagnostic::error(codes::E2222, span, "a `gen fn` cannot be passed to C")
        .primary_label("calling this builds a generator's frame")
        .note("a frame is a value only Ember code resumes [CORO-10]; C calls an ordinary function")
}

/// `[CORO-2]` — what the checker says of a `yield` no generator's body holds.
pub(crate) fn stray_yield(span: Span) -> Diagnostic {
    Diagnostic::error(codes::E2220, span, "`yield` outside a `gen fn`")
        .primary_label("this function is not a generator")
        .help("declare the function `gen fn … -> Generator[Y]` [CORO-1]; a lambda or a local function inside a generator is a function of its own, and cannot yield for it [CORO-2]")
}
