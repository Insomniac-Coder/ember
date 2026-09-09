//! Statements, expressions and patterns (Part III §§4–6).
//!
//! Expressions use Pratt parsing over the precedence table in Part III §5.
//! Binding powers are `2 * level`, so the table can be read straight off the
//! specification: a higher level binds tighter.

use ember_ast::*;
use ember_diag::{Diagnostic, codes};
use ember_lexer::{Kw, Lit, Punct, TokenKind};

use crate::Parser;

/// Binding power for a precedence level in Part III §5.
const fn bp(level: u8) -> u8 {
    level * 2
}

const BP_TERNARY: u8 = bp(1);
const BP_OR: u8 = bp(2);
const BP_AND: u8 = bp(3);
const BP_NOT: u8 = bp(4);
const BP_COMPARE: u8 = bp(5);
const BP_RANGE: u8 = bp(6);
const BP_BITOR: u8 = bp(7);
const BP_BITXOR: u8 = bp(8);
const BP_BITAND: u8 = bp(9);
const BP_SHIFT: u8 = bp(10);
const BP_ADD: u8 = bp(11);
const BP_MUL: u8 = bp(12);
const BP_UNARY: u8 = bp(13);
const BP_POW: u8 = bp(14);
const BP_CAST: u8 = bp(15);

impl Parser<'_> {
    // -- blocks ---------------------------------------------------------------

    /// Parse a block, with the introducing `:` already consumed.
    ///
    /// `[LEX-9]` — either an indented block, or a single simple statement on
    /// the same line (`if x: return`).
    pub(crate) fn parse_block(&mut self) -> Block {
        let start = self.span();
        let id = self.next_id();
        let mut stmts = Vec::new();

        if !self.at_newline() {
            if let Some(stmt) = self.parse_simple_statement() {
                stmts.push(stmt);
            }
            self.expect_newline();
            return Block { id, stmts, span: start.to(self.prev_span()) };
        }

        self.bump(); // NEWLINE
        if !self.eat_indent() {
            let span = self.prev_span().shrink_to_end();
            self.report(
                Diagnostic::error(codes::E0004, span, "expected an indented block")
                    .help("write `pass` for an empty block"),
            );
            return Block { id, stmts, span: start.to(self.prev_span()) };
        }

        while !self.at_dedent() && !self.at_eof() {
            self.eat_newlines();
            if self.at_dedent() || self.at_eof() {
                break;
            }
            let before = self.pos;
            match self.parse_statement() {
                Some(stmt) => stmts.push(stmt),
                None => {
                    if self.pos == before {
                        self.bump();
                    }
                    self.recover_to_line_end();
                }
            }
        }
        self.eat_dedent();
        Block { id, stmts, span: start.to(self.prev_span()) }
    }

    // -- statements -----------------------------------------------------------

    fn parse_statement(&mut self) -> Option<Stmt> {
        // A doc comment where a statement was expected documents nothing; it
        // is discarded in silence (ERR-007). Parsing then continues with the
        // statement that follows — returning `None` here made the caller treat
        // the comment as a parse failure and skip the next line, so a comment
        // deleted the statement under it. A comment never changes what a
        // program does.
        while matches!(self.peek(), TokenKind::DocComment(_)) {
            let doc = self.take_doc();
            self.discard_dangling_doc(doc);
            self.eat_newlines();
        }
        if self.at_dedent() || self.at_eof() {
            return None;
        }

        // `[ATT-3]` — a statement attribute attaches to the next compound
        // statement. Parsed before the label so `@parallel` may precede
        // `outer: for …`.
        let attrs = self.parse_attributes();

        let start = self.span();
        let id = self.next_id();

        // `outer: for …` / `outer: while …` — a labelled loop.
        if self.at_ident() && self.at_punct_at(1, Punct::Colon) {
            let labelled = self.at_kw_at(2, Kw::For) || self.at_kw_at(2, Kw::While);
            if labelled {
                let label = self.expect_ident();
                self.bump(); // ':'
                let kind = if self.at_kw(Kw::For) {
                    self.parse_for(Some(label))
                } else {
                    self.parse_while(Some(label))
                };
                self.check_stmt_attrs(&attrs, Some(&kind));
                return Some(Stmt { id, attrs, kind, span: start.to(self.prev_span()) });
            }
        }

        let kind = match self.peek() {
            TokenKind::Keyword(Kw::If) => StmtKind::If(self.parse_if()),
            TokenKind::Keyword(Kw::While) => self.parse_while(None),
            TokenKind::Keyword(Kw::For) => self.parse_for(None),
            TokenKind::Keyword(Kw::Match) => {
                let (scrutinee, arms) = self.parse_match_tail();
                StmtKind::Match { scrutinee, arms }
            }
            TokenKind::Keyword(Kw::With) => {
                self.bump();
                let mut items = Vec::new();
                loop {
                    let item_start = self.span();
                    // `with g = m.lock():` binds; `with lock.acquire():` does not.
                    let binds = self.at_ident() && self.at_punct_at(1, Punct::Eq);
                    let pattern = binds.then(|| {
                        let p = self.parse_pattern();
                        self.bump(); // '='
                        p
                    });
                    let value = self.parse_expr();
                    items.push(WithItem {
                        pattern,
                        value,
                        span: item_start.to(self.prev_span()),
                    });
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::Colon);
                StmtKind::With { items, body: self.parse_block() }
            }
            TokenKind::Keyword(Kw::Defer) => {
                self.bump();
                self.expect_punct(Punct::Colon);
                StmtKind::Defer(self.parse_block())
            }
            TokenKind::Keyword(Kw::Unsafe) => {
                self.bump();
                self.expect_punct(Punct::Colon);
                StmtKind::Unsafe(self.parse_block())
            }
            TokenKind::Keyword(Kw::Comptime) => {
                self.bump();
                self.expect_punct(Punct::Colon);
                StmtKind::Comptime(self.parse_block())
            }
            _ => {
                // `[ATT-3]` — an attribute may not precede a simple statement.
                self.check_stmt_attrs(&attrs, None);
                let stmt = self.parse_simple_statement()?;
                self.expect_newline();
                return Some(stmt);
            }
        };

        self.check_stmt_attrs(&attrs, Some(&kind));
        Some(Stmt { id, attrs, kind, span: start.to(self.prev_span()) })
    }

    /// `[ATT-2]`, `[ATT-3]` — only four attributes are permitted on a
    /// statement, only on a compound one, and three of the four only on a
    /// `for`. `kind` is `None` when the statement turned out to be simple.
    fn check_stmt_attrs(&mut self, attrs: &[Attribute], kind: Option<&StmtKind>) {
        const LOOP_ONLY: [&str; 3] = ["simd", "parallel", "unroll"];
        for attr in attrs {
            let name = attr.path.last().map(|s| s.name).unwrap();
            let permitted = LOOP_ONLY.iter().any(|p| name.is(p)) || name.is("allow");
            if !permitted {
                self.report(
                    Diagnostic::error(
                        codes::E0104,
                        attr.span,
                        format!("`@{name}` is not permitted on a statement"),
                    )
                    .help("only `@simd`, `@parallel`, `@unroll` and `@allow` are"),
                );
                continue;
            }
            let Some(kind) = kind else {
                self.report(
                    Diagnostic::error(
                        codes::E0108,
                        attr.span,
                        format!("`@{name}` must precede a compound statement"),
                    )
                    .help("attach it to the `for`, `while`, `if` or `match` it describes"),
                );
                continue;
            };
            if LOOP_ONLY.iter().any(|p| name.is(p)) && !matches!(kind, StmtKind::For { .. }) {
                self.report(
                    Diagnostic::error(
                        codes::E0108,
                        attr.span,
                        format!("`@{name}` applies to a `for` loop"),
                    )
                    .help("move it onto the `for` statement"),
                );
            }
        }
    }

    /// A statement that fits on one line and is terminated by `NEWLINE`.
    fn parse_simple_statement(&mut self) -> Option<Stmt> {
        let start = self.span();
        let id = self.next_id();

        let kind = match self.peek() {
            TokenKind::Keyword(Kw::Pass) => {
                self.bump();
                StmtKind::Pass
            }
            // `[GRM-16]` removed the jump alternatives from `small_stmt`: a
            // jump written as a statement is an expression statement, which
            // `parse_expr_statement` produces.
            _ => self.parse_expr_statement()?,
        };

        Some(Stmt { id, attrs: Vec::new(), kind, span: start.to(self.prev_span()) })
    }

    /// A declaration, an assignment, or a bare expression.
    ///
    /// `[GRM-4]` — `x = e` declares when `x` is not in scope and assigns when
    /// it is, which name resolution decides; the parser produces `Assign`
    /// either way. `x: T = e` is always a declaration.
    fn parse_expr_statement(&mut self) -> Option<StmtKind> {
        let first = self.parse_expr();

        // `x: T [= e]` — a typed declaration.
        if self.at_punct(Punct::Colon) {
            self.bump();
            let pattern = self.expr_to_pattern(first);
            let ty = Some(self.parse_type());
            let init = self.eat_punct(Punct::Eq).then(|| self.parse_expr());
            return Some(StmtKind::Decl { pattern, ty, init });
        }

        // `a, b = e` — tuple destructuring (`[GRM-5]`).
        let mut targets = vec![first];
        while self.at_punct(Punct::Comma) {
            self.bump();
            targets.push(self.parse_expr());
        }

        if let Some(op) = self.augmented_assign_op() {
            self.bump();
            let value = self.parse_expr();
            return Some(StmtKind::Assign { targets, op: Some(op), value });
        }
        if self.eat_punct(Punct::Eq) {
            let value = self.parse_expr();
            return Some(StmtKind::Assign { targets, op: None, value });
        }

        if targets.len() > 1 {
            let span = targets[0].span.to(self.prev_span());
            self.report_code(codes::E0100, span, "expected `=` after a list of assignment targets");
        }
        Some(StmtKind::Expr(targets.pop().expect("at least one expression")))
    }

    fn augmented_assign_op(&self) -> Option<BinOp> {
        let TokenKind::Punct(p) = self.peek() else { return None };
        Some(match p {
            Punct::PlusEq => BinOp::Add,
            Punct::MinusEq => BinOp::Sub,
            Punct::StarEq => BinOp::Mul,
            Punct::SlashEq => BinOp::Div,
            Punct::PercentEq => BinOp::Rem,
            Punct::StarStarEq => BinOp::Pow,
            Punct::AmpEq => BinOp::BitAnd,
            Punct::PipeEq => BinOp::BitOr,
            Punct::CaretEq => BinOp::BitXor,
            Punct::ShlEq => BinOp::Shl,
            Punct::ShrEq => BinOp::Shr,
            _ => return None,
        })
    }

    fn parse_if(&mut self) -> IfStmt {
        self.expect_kw(Kw::If);
        let cond = self.parse_condition();
        self.expect_punct(Punct::Colon);
        let then_block = self.parse_block();

        // `elif` is a nested `if` in the else position (Part XVIII §3).
        let else_block = if self.at_kw(Kw::Elif) {
            self.bump_as_if();
            Some(Box::new(ElseBranch::If(self.parse_if_after_keyword())))
        } else if self.eat_kw(Kw::Else) {
            self.expect_punct(Punct::Colon);
            Some(Box::new(ElseBranch::Block(self.parse_block())))
        } else {
            None
        };

        IfStmt { cond, then_block, else_block }
    }

    /// `elif` has already been consumed; parse the rest as an `if`.
    fn parse_if_after_keyword(&mut self) -> IfStmt {
        let cond = self.parse_condition();
        self.expect_punct(Punct::Colon);
        let then_block = self.parse_block();
        let else_block = if self.at_kw(Kw::Elif) {
            self.bump_as_if();
            Some(Box::new(ElseBranch::If(self.parse_if_after_keyword())))
        } else if self.eat_kw(Kw::Else) {
            self.expect_punct(Punct::Colon);
            Some(Box::new(ElseBranch::Block(self.parse_block())))
        } else {
            None
        };
        IfStmt { cond, then_block, else_block }
    }

    fn bump_as_if(&mut self) {
        self.bump(); // `elif`
    }

    /// `expression` or the pattern form `Some(x) = opt`.
    ///
    /// `=` is not an expression operator, so an expression is parsed first and
    /// reinterpreted as a pattern when `=` follows.
    fn parse_condition(&mut self) -> Condition {
        let start = self.span();
        let expr = self.parse_expr_no_block();
        if self.at_punct(Punct::Eq) {
            self.bump();
            let pattern = self.expr_to_pattern(expr);
            // `[GRM-19]` — the pattern in a condition MUST be refutable. A
            // binding or `_` matches everything, so the branch is not a
            // branch at all and the writer meant a plain declaration.
            if pattern_is_irrefutable(&pattern) {
                let span = start.to(self.prev_span());
                self.report(
                    Diagnostic::error(codes::E2036, span, "this pattern always matches")
                        .help("write `x = e` on the preceding line"),
                );
            }
            let value = self.parse_expr_no_block();
            return Condition::Pattern { pattern, value };
        }
        Condition::Expr(expr)
    }

    fn parse_while(&mut self, label: Option<Ident>) -> StmtKind {
        self.expect_kw(Kw::While);
        let cond = self.parse_condition();
        self.expect_punct(Punct::Colon);
        let body = self.parse_block();
        let else_block = self.parse_loop_else();
        StmtKind::While { label, cond, body, else_block }
    }

    /// `[GRM-15]` — `owned e` is an expression form legal in exactly two
    /// places. Parsed here rather than in `parse_prefix` so that everywhere
    /// else it reaches `E0109` with the fix named.
    /// `[GRM-8a]`, `[GRM-8c]` — one argument inside `name[…]` in expression
    /// position.
    ///
    /// The parser commits to a type on exactly seven tokens: `ref`, `*`,
    /// `dyn`, `fn`, `extern`, `void`, `!`. The set is unambiguous because
    /// Ember has no prefix `*` and no prefix `!` — `not` and `~` are the
    /// operators — and the rest are keywords, so committing on them cannot
    /// misparse an expression. Everything else is parsed as an expression and
    /// reinterpreted by name resolution if the node turns out to be an
    /// instantiation.
    fn parse_type_or_expr(&mut self) -> TypeOrExpr {
        // `Item = T` binds an associated type in both positions, and is
        // never a named argument or an assignment (`[GRM-8c]`).
        if self.at_ident() && self.at_punct_at(1, Punct::Eq) {
            let name = self.expect_ident();
            self.bump();
            return TypeOrExpr::Binding { name, ty: self.parse_type() };
        }
        let type_only = matches!(
            self.peek(),
            TokenKind::Keyword(Kw::Ref | Kw::Dyn | Kw::Fn | Kw::Extern | Kw::Void)
        ) || self.at_punct(Punct::Star)
            || self.at_punct(Punct::Bang);
        if type_only {
            return TypeOrExpr::Type(self.parse_type());
        }
        TypeOrExpr::Expr(self.parse_expr())
    }

    fn parse_consumable_expr(&mut self) -> Expr {
        if self.at_kw(Kw::Owned) && !self.at_kw_at(1, Kw::Fn) {
            let start = self.span();
            self.bump();
            let inner = self.parse_expr_no_block();
            return Expr {
                id: self.next_id(),
                kind: ExprKind::Owned(Box::new(inner)),
                span: start.to(self.prev_span()),
            };
        }
        self.parse_expr_no_block()
    }

    fn parse_for(&mut self, label: Option<Ident>) -> StmtKind {
        self.expect_kw(Kw::For);
        let pattern = self.parse_pattern();
        self.expect_kw(Kw::In);
        let iter = self.parse_consumable_expr();
        self.expect_punct(Punct::Colon);
        let body = self.parse_block();
        let else_block = self.parse_loop_else();
        StmtKind::For { label, pattern, iter, body, else_block }
    }

    /// `[CTL-4]`/`[GRM-6]` — the loop `else` runs when the loop ends without
    /// `break`.
    fn parse_loop_else(&mut self) -> Option<Block> {
        if !self.eat_kw(Kw::Else) {
            return None;
        }
        self.expect_punct(Punct::Colon);
        Some(self.parse_block())
    }

    /// `match e:` and its arms. `[GRM-10]` — arms are all `:` blocks or all
    /// `=>` expressions; mixing is `E0103`.
    fn parse_match_tail(&mut self) -> (Expr, Vec<MatchArm>) {
        self.expect_kw(Kw::Match);
        let scrutinee = self.parse_consumable_expr();
        self.expect_punct(Punct::Colon);

        let mut arms = Vec::new();
        let mut expression_form: Option<bool> = None;
        if self.at_newline() {
            self.bump();
            if self.eat_indent() {
                while !self.at_dedent() && !self.at_eof() {
                    self.eat_newlines();
                    if self.at_dedent() || self.at_eof() {
                        break;
                    }
                    let start = self.span();
                    let id = self.next_id();
                    let pattern = self.parse_pattern();
                    let guard = self.eat_kw(Kw::If).then(|| self.parse_expr_no_block());

                    let is_expression_arm = self.at_punct(Punct::FatArrow);
                    match expression_form {
                        None => expression_form = Some(is_expression_arm),
                        Some(previous) if previous != is_expression_arm => {
                            let span = self.span();
                            self.report(
                                Diagnostic::error(
                                    codes::E0103,
                                    span,
                                    "match arms mix statement and expression form",
                                )
                                .help("use `:` and a block for every arm, or `=>` and an expression for every arm"),
                            );
                        }
                        _ => {}
                    }

                    let body = if is_expression_arm {
                        self.bump();
                        let expr = self.parse_expr();
                        self.expect_newline();
                        MatchArmBody::Expr(expr)
                    } else {
                        self.expect_punct(Punct::Colon);
                        MatchArmBody::Block(self.parse_block())
                    };
                    arms.push(MatchArm { id, pattern, guard, body, span: start.to(self.prev_span()) });
                }
                self.eat_dedent();
            }
        }
        (scrutinee, arms)
    }

    // -- expressions ------------------------------------------------------------

    pub(crate) fn parse_expr(&mut self) -> Expr {
        self.parse_expr_bp(0, true)
    }

    /// An expression in a position terminated by `:` — a condition, a `for`
    /// iterable, a `match` scrutinee. Lambdas with block bodies are not
    /// allowed to swallow the introducing colon here.
    pub(crate) fn parse_expr_no_block(&mut self) -> Expr {
        self.parse_expr_bp(0, false)
    }

    /// `[GRM-16]` — `return`, `break` and `continue` are expressions of type
    /// `!` at the lowest precedence, parallel to `ternary` and never atoms. A
    /// jump is therefore always the whole of the expression it appears in;
    /// meeting one where an operand is expected is `E0107`.
    fn at_jump(&self) -> bool {
        matches!(
            self.peek(),
            TokenKind::Keyword(Kw::Return | Kw::Break | Kw::Continue)
        )
    }

    /// True where a `return` with no value ends: a line end, a closing
    /// bracket, a separator, or the `:` of an enclosing construct.
    fn at_expr_end(&self) -> bool {
        self.at_newline()
            || self.at_eof()
            || self.at_dedent()
            || self.at_punct(Punct::RParen)
            || self.at_punct(Punct::RBracket)
            || self.at_punct(Punct::RBrace)
            || self.at_punct(Punct::Comma)
            || self.at_punct(Punct::Colon)
    }

    /// `[GRM-22]` — `yield` occupies `return`'s grammatical position and
    /// precedence. It is **not** a jump: its type is the coroutine's resume
    /// type, so `x = yield e` is legal where `x = return e` is not, and it is
    /// therefore admitted wherever a whole expression is admitted rather than
    /// being restricted to being one.
    fn at_yield(&self) -> bool {
        matches!(self.peek(), TokenKind::Keyword(Kw::Yield))
    }

    fn parse_yield(&mut self, allow_block_lambda: bool) -> Expr {
        let start = self.span();
        let id = self.next_id();
        self.expect_kw(Kw::Yield);
        // "A bare `yield` is `yield ()`" — `[GRM-22]`.
        let value = (!self.at_expr_end())
            .then(|| Box::new(self.parse_expr_bp(0, allow_block_lambda)));
        Expr { id, kind: ExprKind::Yield(value), span: start.to(self.prev_span()) }
    }

    fn parse_jump(&mut self, allow_block_lambda: bool) -> Expr {
        let start = self.span();
        let id = self.next_id();
        let jump = match self.peek() {
            TokenKind::Keyword(Kw::Return) => {
                self.bump();
                let value = (!self.at_expr_end())
                    .then(|| Box::new(self.parse_expr_bp(0, allow_block_lambda)));
                Jump::Return(value)
            }
            TokenKind::Keyword(Kw::Break) => {
                self.bump();
                Jump::Break { label: self.at_ident().then(|| self.expect_ident()) }
            }
            _ => {
                self.expect_kw(Kw::Continue);
                Jump::Continue { label: self.at_ident().then(|| self.expect_ident()) }
            }
        };
        Expr { id, kind: ExprKind::Jump(jump), span: start.to(self.prev_span()) }
    }

    fn parse_expr_bp(&mut self, min_bp: u8, allow_block_lambda: bool) -> Expr {
        if self.at_jump() {
            if min_bp > 0 {
                let span = self.span();
                let word = match self.peek() {
                    TokenKind::Keyword(kw) => kw.as_str(),
                    _ => "return",
                };
                self.report(
                    Diagnostic::error(
                        codes::E0107,
                        span,
                        "a jump expression may not be an operand",
                    )
                    .primary_label(format!("`{word}` is the whole expression or nothing"))
                    .help(format!(
                        "put `{word}` on its own, or bind the operand first and `{word}` it"
                    )),
                );
            }
            // Parsed either way, so one mistake does not cascade.
            return self.parse_jump(allow_block_lambda);
        }

        if self.at_yield() {
            if min_bp > 0 {
                // `[GRM-22]` gives `yield` `return`'s precedence, which is the
                // lowest, so it cannot be an operand of anything. The document
                // names no code for this — `E0107` is `[GRM-16]`'s and says
                // "jump", which `yield` is not — so it is the parser's ordinary
                // `E0100`. Recorded in `docs/DECISIONS.md` as ADR-015.
                let span = self.span();
                self.report(
                    Diagnostic::error(
                        codes::E0100,
                        span,
                        "`yield` may not be an operand",
                    )
                    .primary_label(
                        "`yield` binds at `return`'s precedence, the lowest there is"
                            .to_string(),
                    )
                    .help(
                        "bind it first: `v = yield e`, then use `v`".to_string(),
                    ),
                );
            }
            return self.parse_yield(allow_block_lambda);
        }

        let start = self.span();
        let mut lhs = self.parse_prefix(allow_block_lambda);

        loop {
            // Postfix: calls, indexing, fields, `?`, `as`.
            lhs = self.parse_postfix(lhs, min_bp, allow_block_lambda);

            let Some((op, lbp, rbp, non_assoc)) = self.infix_op() else { break };
            if lbp < min_bp {
                break;
            }
            self.bump_infix(op);
            let rhs = self.parse_expr_bp(rbp, allow_block_lambda);

            if non_assoc && matches!(op, InfixOp::Bin(b) if b.is_comparison()) {
                if let Some((next, _, _, _)) = self.infix_op() {
                    if matches!(next, InfixOp::Bin(b) if b.is_comparison()) {
                        let span = self.span();
                        self.report(
                            Diagnostic::error(codes::E0102, span, "chained comparison")
                                .help("write `a < b and b < c`"),
                        );
                    }
                }
            }

            let id = self.next_id();
            let span = start.to(self.prev_span());
            lhs = match op {
                InfixOp::Bin(bin) => Expr {
                    id,
                    kind: ExprKind::Binary { op: bin, lhs: Box::new(lhs), rhs: Box::new(rhs) },
                    span,
                },
                InfixOp::Logical(logical) => Expr {
                    id,
                    kind: ExprKind::Logical {
                        op: logical,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    span,
                },
                InfixOp::Range { inclusive } => Expr {
                    id,
                    kind: ExprKind::Range {
                        lo: Some(Box::new(lhs)),
                        hi: Some(Box::new(rhs)),
                        inclusive,
                    },
                    span,
                },
            };
        }

        // `x if c else y` — the ternary, right-associative and loosest.
        if min_bp <= BP_TERNARY && self.at_kw(Kw::If) {
            self.bump();
            let cond = self.parse_expr_bp(BP_TERNARY + 1, allow_block_lambda);
            self.expect_kw(Kw::Else);
            let else_expr = self.parse_expr_bp(BP_TERNARY, allow_block_lambda);
            let id = self.next_id();
            return Expr {
                id,
                kind: ExprKind::Ternary {
                    then_expr: Box::new(lhs),
                    cond: Box::new(cond),
                    else_expr: Box::new(else_expr),
                },
                span: start.to(self.prev_span()),
            };
        }

        lhs
    }

    fn infix_op(&self) -> Option<(InfixOp, u8, u8, bool)> {
        // (operator, left binding power, right binding power, non-associative)
        let left = |op, l: u8| Some((op, l, l + 1, false));
        match self.peek() {
            TokenKind::Keyword(Kw::Or) => left(InfixOp::Logical(LogicalOp::Or), BP_OR),
            TokenKind::Keyword(Kw::And) => left(InfixOp::Logical(LogicalOp::And), BP_AND),
            TokenKind::Keyword(Kw::Is) => {
                let op = if self.at_kw_at(1, Kw::Not) { BinOp::IsNot } else { BinOp::Is };
                Some((InfixOp::Bin(op), BP_COMPARE, BP_COMPARE + 1, true))
            }
            TokenKind::Keyword(Kw::In) => {
                Some((InfixOp::Bin(BinOp::In), BP_COMPARE, BP_COMPARE + 1, true))
            }
            TokenKind::Keyword(Kw::Not) if self.at_kw_at(1, Kw::In) => {
                Some((InfixOp::Bin(BinOp::NotIn), BP_COMPARE, BP_COMPARE + 1, true))
            }
            TokenKind::Punct(p) => {
                let compare = |op| Some((InfixOp::Bin(op), BP_COMPARE, BP_COMPARE + 1, true));
                match p {
                    Punct::EqEq => compare(BinOp::Eq),
                    Punct::NotEq => compare(BinOp::Ne),
                    Punct::Lt => compare(BinOp::Lt),
                    Punct::Gt => compare(BinOp::Gt),
                    Punct::LtEq => compare(BinOp::Le),
                    Punct::GtEq => compare(BinOp::Ge),
                    Punct::DotDot => {
                        Some((InfixOp::Range { inclusive: false }, BP_RANGE, BP_RANGE + 1, true))
                    }
                    Punct::DotDotEq => {
                        Some((InfixOp::Range { inclusive: true }, BP_RANGE, BP_RANGE + 1, true))
                    }
                    Punct::Pipe => left(InfixOp::Bin(BinOp::BitOr), BP_BITOR),
                    Punct::Caret => left(InfixOp::Bin(BinOp::BitXor), BP_BITXOR),
                    Punct::Amp => left(InfixOp::Bin(BinOp::BitAnd), BP_BITAND),
                    Punct::Shl => left(InfixOp::Bin(BinOp::Shl), BP_SHIFT),
                    Punct::Shr => left(InfixOp::Bin(BinOp::Shr), BP_SHIFT),
                    Punct::Plus => left(InfixOp::Bin(BinOp::Add), BP_ADD),
                    Punct::Minus => left(InfixOp::Bin(BinOp::Sub), BP_ADD),
                    Punct::Star => left(InfixOp::Bin(BinOp::Mul), BP_MUL),
                    Punct::Slash => left(InfixOp::Bin(BinOp::Div), BP_MUL),
                    Punct::Percent => left(InfixOp::Bin(BinOp::Rem), BP_MUL),
                    // `**` is right-associative, so the right binding power
                    // equals the left one.
                    Punct::StarStar => Some((InfixOp::Bin(BinOp::Pow), BP_POW, BP_POW, false)),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn bump_infix(&mut self, op: InfixOp) {
        self.bump();
        // `is not` and `not in` are two tokens.
        if matches!(op, InfixOp::Bin(BinOp::IsNot) | InfixOp::Bin(BinOp::NotIn)) {
            self.bump();
        }
    }

    fn parse_prefix(&mut self, allow_block_lambda: bool) -> Expr {
        let start = self.span();

        if self.at_kw(Kw::Not) {
            self.bump();
            let operand = self.parse_expr_bp(BP_NOT + 1, allow_block_lambda);
            let id = self.next_id();
            return Expr {
                id,
                kind: ExprKind::Unary { op: UnOp::Not, operand: Box::new(operand) },
                span: start.to(self.prev_span()),
            };
        }

        if self.at_punct(Punct::Minus) || self.at_punct(Punct::Tilde) {
            let op = if self.at_punct(Punct::Minus) { UnOp::Neg } else { UnOp::BitNot };
            self.bump();
            // `[III.5]` — `**` binds tighter than a unary minus on its left,
            // so `-2**2` is `-(2**2)`.
            let operand = self.parse_expr_bp(BP_UNARY + 1, allow_block_lambda);
            let id = self.next_id();
            return Expr {
                id,
                kind: ExprKind::Unary { op, operand: Box::new(operand) },
                span: start.to(self.prev_span()),
            };
        }

        // A range with no lower bound: `..n`, `..=n`, `..`.
        if self.at_punct(Punct::DotDot) || self.at_punct(Punct::DotDotEq) {
            let inclusive = self.at_punct(Punct::DotDotEq);
            self.bump();
            let hi = self
                .starts_expression()
                .then(|| Box::new(self.parse_expr_bp(BP_RANGE + 1, allow_block_lambda)));
            let id = self.next_id();
            return Expr {
                id,
                kind: ExprKind::Range { lo: None, hi, inclusive },
                span: start.to(self.prev_span()),
            };
        }

        if self.at_kw(Kw::Ref) {
            self.bump();
            let mutable = self.eat_kw(Kw::Mut);
            let place = self.parse_expr_bp(BP_UNARY, allow_block_lambda);
            let id = self.next_id();
            return Expr {
                id,
                kind: ExprKind::RefOf { mutable, place: Box::new(place) },
                span: start.to(self.prev_span()),
            };
        }

        self.parse_atom(allow_block_lambda)
    }

    fn parse_atom(&mut self, allow_block_lambda: bool) -> Expr {
        let start = self.span();
        let id = self.next_id();

        let kind = match self.peek().clone() {
            TokenKind::Lit(lit) => {
                self.bump();
                match lit {
                    Lit::FStr(parts) => ExprKind::FString(self.build_fstring(parts)),
                    other => ExprKind::Lit(convert_literal(other)),
                }
            }
            TokenKind::Keyword(Kw::True) => {
                self.bump();
                ExprKind::Lit(Literal::Bool(true))
            }
            TokenKind::Keyword(Kw::False) => {
                self.bump();
                ExprKind::Lit(Literal::Bool(false))
            }
            TokenKind::Keyword(Kw::SelfValue) => {
                self.bump();
                ExprKind::SelfExpr
            }
            TokenKind::Keyword(Kw::SelfType) => {
                let span = self.span();
                self.bump();
                ExprKind::Path { segments: vec![Ident { name: "Self".into(), span }] }
            }
            TokenKind::Keyword(Kw::Super) => {
                let span = self.span();
                self.bump();
                ExprKind::Path { segments: vec![Ident { name: "super".into(), span }] }
            }
            TokenKind::Keyword(Kw::Match) => {
                let (scrutinee, arms) = self.parse_match_tail();
                ExprKind::Match { scrutinee: Box::new(scrutinee), arms }
            }
            TokenKind::Keyword(Kw::Fn) | TokenKind::Keyword(Kw::Owned)
                if self.at_kw(Kw::Fn) || self.at_kw_at(1, Kw::Fn) =>
            {
                ExprKind::Lambda(self.parse_lambda(allow_block_lambda))
            }
            // `[GRM-15]` — `owned` reaching expression position here means it
            // is not a `for` iterable, a `match` scrutinee or an `owned fn`,
            // which are the only three places it may appear.
            TokenKind::Keyword(Kw::Owned) => {
                let span = self.span();
                self.bump();
                self.report(
                    Diagnostic::error(
                        codes::E0109,
                        span,
                        "`owned` is not permitted in expression position",
                    )
                    .help(
                        "`owned` marks a parameter, a receiver, a closure or a consumed \
                         scrutinee; to move a value, pass it to an `owned` parameter",
                    ),
                );
                let inner = self.parse_expr_bp(BP_CAST, allow_block_lambda);
                ExprKind::Owned(Box::new(inner))
            }
            TokenKind::Punct(Punct::LParen) => {
                self.bump();
                if self.eat_punct(Punct::RParen) {
                    ExprKind::Tuple(Vec::new())
                } else {
                    let first = self.parse_expr();
                    if self.at_punct(Punct::Comma) {
                        let mut items = vec![first];
                        while self.eat_punct(Punct::Comma) {
                            if self.at_punct(Punct::RParen) {
                                break;
                            }
                            items.push(self.parse_expr());
                        }
                        self.expect_punct(Punct::RParen);
                        ExprKind::Tuple(items)
                    } else {
                        self.expect_punct(Punct::RParen);
                        ExprKind::Paren(Box::new(first))
                    }
                }
            }
            TokenKind::Punct(Punct::LBracket) => {
                self.bump();
                if self.eat_punct(Punct::RBracket) {
                    ExprKind::ArrayLit(Vec::new())
                } else {
                    let first = self.parse_expr();
                    if self.eat_punct(Punct::Semi) {
                        // `[value; count]`
                        let count = self.parse_expr();
                        self.expect_punct(Punct::RBracket);
                        ExprKind::ArrayRepeat {
                            value: Box::new(first),
                            count: Box::new(count),
                        }
                    } else {
                        let mut items = vec![first];
                        while self.eat_punct(Punct::Comma) {
                            if self.at_punct(Punct::RBracket) {
                                break;
                            }
                            items.push(self.parse_expr());
                        }
                        self.expect_punct(Punct::RBracket);
                        ExprKind::ArrayLit(items)
                    }
                }
            }
            TokenKind::Ident(_) | TokenKind::RawIdent(_) => {
                let mut segments = vec![self.expect_ident()];
                // `Shape::Circle` — an explicit path (`.` also works, and is
                // handled by postfix field access).
                while self.at_punct(Punct::ColonColon) {
                    self.bump();
                    segments.push(self.expect_ident());
                }
                ExprKind::Path { segments }
            }
            TokenKind::Reserved(reserved) => {
                let reserved = reserved;
                let span = self.span();
                self.bump();
                // `[LEX-14a]` — name the version that takes the word.
                self.report(
                    Diagnostic::error(
                        codes::E0005,
                        span,
                        format!(
                            "`{}` is reserved for {}",
                            reserved.as_str(),
                            reserved.planned_version()
                        ),
                    )
                    .help(format!(
                        "write `r#{}` to use the word as an identifier",
                        reserved.as_str()
                    )),
                );
                ExprKind::Error
            }
            other => {
                let found = other.to_string();
                let span = self.span();
                self.report(
                    Diagnostic::error(codes::E0100, span, "expected an expression")
                        .primary_label(format!("found {found}")),
                );
                if !matches!(other, TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof) {
                    self.bump();
                }
                ExprKind::Error
            }
        };

        Expr { id, kind, span: start.to(self.prev_span()) }
    }

    /// Two tuple indices that the lexer joined into one float literal.
    ///
    /// After a `.`, a token like `0.1` can only be a pair of tuple indices —
    /// the lexer cannot tell that without knowing what came before the dot,
    /// so the split happens here. The token's text is read from the source
    /// rather than from the literal's `f64`, which no longer distinguishes
    /// `.1` from `.10`. Nothing is consumed unless the split succeeds.
    fn eat_tuple_index_pair(&mut self) -> Option<(u32, u32)> {
        let TokenKind::Lit(Lit::Float { suffix: None, .. }) = self.peek() else {
            return None;
        };
        let span = self.span();
        let text = self.src.get(span.start as usize..span.end as usize)?;
        let (outer, inner) = text.split_once('.')?;
        let pair = (outer.parse::<u32>().ok()?, inner.parse::<u32>().ok()?);
        self.bump();
        Some(pair)
    }

    fn parse_postfix(&mut self, mut expr: Expr, min_bp: u8, allow_block_lambda: bool) -> Expr {
        loop {
            let start = expr.span;
            let id = self.next_id();
            let kind = match self.peek() {
                TokenKind::Punct(Punct::LParen) => {
                    self.bump();
                    let args = self.parse_args();
                    // A call on a field access is a method call.
                    match expr.kind {
                        ExprKind::Field { base, name } => ExprKind::MethodCall {
                            recv: base,
                            name,
                            generic_args: Vec::new(),
                            args,
                        },
                        _ => ExprKind::Call { callee: Box::new(expr), args },
                    }
                }
                TokenKind::Punct(Punct::LBracket) => {
                    // `[GRM-8]` — index or generic instantiation; resolution
                    // decides which.
                    self.bump();
                    let mut args = Vec::new();
                    while !self.at_punct(Punct::RBracket) && !self.at_eof() {
                        args.push(self.parse_type_or_expr());
                        if !self.eat_punct(Punct::Comma) {
                            break;
                        }
                    }
                    self.expect_punct(Punct::RBracket);
                    ExprKind::IndexOrInstantiate { base: Box::new(expr), args }
                }
                TokenKind::Punct(Punct::Dot) => {
                    self.bump();
                    if let TokenKind::Lit(Lit::Int { value, .. }) = self.peek() {
                        let index = *value as u32;
                        self.bump();
                        ExprKind::TupleField { base: Box::new(expr), index }
                    } else if let Some((outer, inner)) = self.eat_tuple_index_pair() {
                        // `t.0.1` reaches into a nested tuple, but the lexer
                        // has already read `0.1` as one float literal, so the
                        // pair is taken apart here.
                        let first = Expr {
                            id: self.next_id(),
                            kind: ExprKind::TupleField { base: Box::new(expr), index: outer },
                            span: start.to(self.prev_span()),
                        };
                        ExprKind::TupleField { base: Box::new(first), index: inner }
                    } else {
                        let name = self.expect_ident();
                        ExprKind::Field { base: Box::new(expr), name }
                    }
                }
                TokenKind::Punct(Punct::QuestionDot) => {
                    self.bump();
                    let name = self.expect_ident();
                    let args = self.at_punct(Punct::LParen).then(|| {
                        self.bump();
                        self.parse_args()
                    });
                    ExprKind::OptChain { base: Box::new(expr), name, args }
                }
                TokenKind::Punct(Punct::Question) if min_bp <= BP_CAST => {
                    self.bump();
                    ExprKind::Try(Box::new(expr))
                }
                TokenKind::Keyword(Kw::As) if min_bp <= BP_CAST => {
                    self.bump();
                    // `as?` and `as!` are downcasts of a class handle.
                    if self.eat_punct(Punct::Question) {
                        ExprKind::Downcast {
                            expr: Box::new(expr),
                            ty: self.parse_type(),
                            forced: false,
                        }
                    } else if self.eat_punct(Punct::Bang) {
                        ExprKind::Downcast {
                            expr: Box::new(expr),
                            ty: self.parse_type(),
                            forced: true,
                        }
                    } else {
                        ExprKind::Cast { expr: Box::new(expr), ty: self.parse_type() }
                    }
                }
                _ => return expr,
            };
            let _ = allow_block_lambda;
            expr = Expr { id, kind, span: start.to(self.prev_span()) };
        }
    }

    fn parse_args(&mut self) -> Vec<Arg> {
        let mut args = Vec::new();
        while !self.at_punct(Punct::RParen) && !self.at_eof() {
            let start = self.span();
            // `f(x=1)` — a named argument (`[TYP-25]`).
            let name = (self.at_ident() && self.at_punct_at(1, Punct::Eq)).then(|| {
                let ident = self.expect_ident();
                self.bump(); // '='
                ident
            });
            // `[LEX-6a]` — inside brackets a lambda's `:` body is a single
            // `small_stmt`, not a block: indentation is not significant here,
            // so there is nothing for a second statement to belong to.
            // `[GRM-17]` is the diagnostic when one is written anyway.
            let value = self.parse_expr_no_block();
            args.push(Arg { name, value, span: start.to(self.prev_span()) });
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        self.expect_punct(Punct::RParen);
        args
    }

    fn parse_lambda(&mut self, allow_block_lambda: bool) -> Lambda {
        let is_owned = self.eat_kw(Kw::Owned);
        self.expect_kw(Kw::Fn);
        self.expect_punct(Punct::LParen);
        let mut params = Vec::new();
        while !self.at_punct(Punct::RParen) && !self.at_eof() {
            let start = self.span();
            let id = self.next_id();
            let mode = if self.eat_kw(Kw::Mut) {
                Mode::Mut
            } else if self.eat_kw(Kw::Owned) {
                Mode::Owned
            } else {
                Mode::Borrow
            };
            let name = self.expect_ident();
            // A lambda parameter's type is optional; it is inferred from the
            // expected function type (`[TYP-23]` rule 4).
            let ty = if self.eat_punct(Punct::Colon) {
                self.parse_type()
            } else {
                TypeExpr { id: self.next_id(), kind: TypeKind::Infer, span: name.span }
            };
            params.push(Param {
                id,
                mode,
                kind: ParamKind::Named { name, ty },
                default: None,
                span: start.to(self.prev_span()),
            });
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        self.expect_punct(Punct::RParen);
        let ret = self.eat_punct(Punct::Arrow).then(|| self.parse_type());

        let body = if self.eat_punct(Punct::FatArrow) {
            LambdaBody::Expr(Box::new(self.parse_expr()))
        } else if allow_block_lambda && self.eat_punct(Punct::Colon) {
            LambdaBody::Block(self.parse_block())
        } else if !allow_block_lambda && self.at_punct(Punct::Colon) {
            // `[LEX-6a]` — "Inside brackets, a lambda's `:` body is a single
            // `small_stmt`, terminated by the enclosing closing bracket or by
            // a `,` at the same bracket depth. No `NEWLINE` is required or
            // emitted." So the `:` form **is** admitted here; what is not is a
            // body of more than one statement, which `[GRM-17]` reports —
            // indentation means nothing inside brackets, so there is no block
            // for the rest of it to belong to.
            let colon = self.span();
            self.bump();
            let body = self.parse_expr_no_block();
            if !self.at_punct(Punct::Comma)
                && !self.at_punct(Punct::RParen)
                && !self.at_punct(Punct::RBracket)
                && !self.at_punct(Punct::RBrace)
                && !self.at_eof()
            {
                self.report(
                    Diagnostic::error(
                        codes::E0106,
                        colon.to(self.span()),
                        "a multi-statement closure cannot be written inside brackets",
                    )
                    .help("bind it on a preceding line: `h = fn(e): …` then pass `h`")
                    .note("indentation is not significant inside brackets [LEX-6]"),
                );
            }
            LambdaBody::Expr(Box::new(body))
        } else {
            let span = self.span();
            self.report(
                Diagnostic::error(codes::E0100, span, "expected `=>` or `:` after a lambda's parameters")
                    .help("`fn(x) => x * 2` for an expression body, `fn(x): …` for a block"),
            );
            LambdaBody::Expr(Box::new(Expr {
                id: self.next_id(),
                kind: ExprKind::Error,
                span,
            }))
        };
        Lambda { is_owned, params, ret, body }
    }

    /// Re-parse each interpolation of an f-string. The spans the lexer
    /// recorded point into the real file, so diagnostics land correctly.
    fn build_fstring(&mut self, parts: Vec<ember_lexer::FStrPart>) -> Vec<FStringPart> {
        parts
            .into_iter()
            .map(|part| match part {
                ember_lexer::FStrPart::Text(text) => FStringPart::Text(text),
                ember_lexer::FStrPart::Expr { span, format_spec } => {
                    let expr = self.parse_subexpression(span);
                    FStringPart::Expr { expr: Box::new(expr), format_spec }
                }
            })
            .collect()
    }

    fn starts_expression(&self) -> bool {
        !matches!(
            self.peek(),
            TokenKind::Newline
                | TokenKind::Dedent
                | TokenKind::Indent
                | TokenKind::Eof
                | TokenKind::Punct(
                    Punct::RParen | Punct::RBracket | Punct::RBrace | Punct::Comma | Punct::Colon
                )
        )
    }

    // -- patterns ---------------------------------------------------------------

    pub(crate) fn parse_pattern(&mut self) -> Pattern {
        let start = self.span();
        let first = self.parse_pattern_single();
        if !self.at_punct(Punct::Pipe) {
            return first;
        }
        let mut alts = vec![first];
        while self.eat_punct(Punct::Pipe) {
            alts.push(self.parse_pattern_single());
        }
        Pattern { id: self.next_id(), kind: PatternKind::Or(alts), span: start.to(self.prev_span()) }
    }

    fn parse_pattern_single(&mut self) -> Pattern {
        let start = self.span();
        let id = self.next_id();

        let kind = match self.peek().clone() {
            TokenKind::Ident(name) if name.is_discard() => {
                self.bump();
                PatternKind::Wild
            }
            TokenKind::Lit(lit) => {
                self.bump();
                let lo = convert_literal(lit);
                if self.at_punct(Punct::DotDot) || self.at_punct(Punct::DotDotEq) {
                    let inclusive = self.at_punct(Punct::DotDotEq);
                    self.bump();
                    let TokenKind::Lit(hi) = self.peek().clone() else {
                        let span = self.span();
                        self.report_code(codes::E0100, span, "expected a literal after `..`");
                        return Pattern { id, kind: PatternKind::Error, span };
                    };
                    self.bump();
                    PatternKind::Range { lo, hi: convert_literal(hi), inclusive }
                } else {
                    PatternKind::Lit(lo)
                }
            }
            TokenKind::Keyword(Kw::True) => {
                self.bump();
                PatternKind::Lit(Literal::Bool(true))
            }
            TokenKind::Keyword(Kw::False) => {
                self.bump();
                PatternKind::Lit(Literal::Bool(false))
            }
            TokenKind::Keyword(Kw::Ref) => {
                self.bump();
                let mutable = self.eat_kw(Kw::Mut);
                let name = self.expect_ident();
                PatternKind::Bind { name, by_ref: true, mutable, sub: None }
            }
            TokenKind::Keyword(Kw::Mut) => {
                self.bump();
                let name = self.expect_ident();
                PatternKind::Bind { name, by_ref: false, mutable: true, sub: None }
            }
            TokenKind::Punct(Punct::LParen) => {
                self.bump();
                let mut items = Vec::new();
                while !self.at_punct(Punct::RParen) && !self.at_eof() {
                    items.push(self.parse_pattern());
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::RParen);
                PatternKind::Tuple(items)
            }
            TokenKind::Punct(Punct::LBracket) => {
                self.bump();
                let mut prefix = Vec::new();
                let mut suffix = Vec::new();
                let mut rest = None;
                while !self.at_punct(Punct::RBracket) && !self.at_eof() {
                    if self.at_punct(Punct::DotDot) {
                        self.bump();
                        rest = Some(self.at_ident().then(|| self.expect_ident()));
                    } else if rest.is_some() {
                        suffix.push(self.parse_pattern());
                    } else {
                        prefix.push(self.parse_pattern());
                    }
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::RBracket);
                PatternKind::Slice { prefix, rest, suffix }
            }
            TokenKind::Ident(_) | TokenKind::RawIdent(_) => {
                let mut path = vec![self.expect_ident()];
                while self.at_punct(Punct::Dot) || self.at_punct(Punct::ColonColon) {
                    self.bump();
                    path.push(self.expect_ident());
                }
                if self.eat_punct(Punct::LParen) {
                    let (fields, has_rest) = self.parse_field_patterns();
                    PatternKind::Constructor { path, fields, has_rest }
                } else if path.len() > 1 {
                    PatternKind::Path { segments: path }
                } else {
                    // `name @ subpattern`
                    let name = path[0];
                    let sub = self
                        .eat_punct(Punct::At)
                        .then(|| Box::new(self.parse_pattern_single()));
                    // `[GRM-12]` — whether this is a binding or a unit variant
                    // is decided in name resolution.
                    PatternKind::Bind { name, by_ref: false, mutable: false, sub }
                }
            }
            other => {
                let found = other.to_string();
                let span = self.span();
                self.report(
                    Diagnostic::error(codes::E0100, span, "expected a pattern")
                        .primary_label(format!("found {found}")),
                );
                if !matches!(other, TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof) {
                    self.bump();
                }
                PatternKind::Error
            }
        };

        Pattern { id, kind, span: start.to(self.prev_span()) }
    }

    fn parse_field_patterns(&mut self) -> (Vec<FieldPattern>, bool) {
        let mut fields = Vec::new();
        let mut has_rest = false;
        while !self.at_punct(Punct::RParen) && !self.at_eof() {
            if self.eat_punct(Punct::DotDot) {
                has_rest = true;
                let _ = self.eat_punct(Punct::Comma);
                break;
            }
            let start = self.span();
            let name = (self.at_ident() && self.at_punct_at(1, Punct::Eq)).then(|| {
                let ident = self.expect_ident();
                self.bump(); // '='
                ident
            });
            let pattern = self.parse_pattern();
            fields.push(FieldPattern { name, pattern, span: start.to(self.prev_span()) });
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        self.expect_punct(Punct::RParen);
        (fields, has_rest)
    }

    /// Reinterpret an already-parsed expression as a pattern.
    ///
    /// `=` is not an expression operator, so `if Some(x) = opt:` and
    /// `x: T = e` are parsed as expressions first and converted here. This
    /// avoids unbounded lookahead at every statement.
    fn expr_to_pattern(&mut self, expr: Expr) -> Pattern {
        let span = expr.span;
        let kind = match expr.kind {
            ExprKind::Path { segments } if segments.len() == 1 => {
                let name = segments[0];
                if name.name.is_discard() {
                    PatternKind::Wild
                } else {
                    PatternKind::Bind { name, by_ref: false, mutable: false, sub: None }
                }
            }
            ExprKind::Path { segments } => PatternKind::Path { segments },
            ExprKind::Lit(lit) => PatternKind::Lit(lit),
            ExprKind::Tuple(items) => {
                PatternKind::Tuple(items.into_iter().map(|e| self.expr_to_pattern(e)).collect())
            }
            ExprKind::Paren(inner) => return self.expr_to_pattern(*inner),
            ExprKind::Call { callee, args } => {
                let path = match callee.kind {
                    ExprKind::Path { segments } => segments,
                    ExprKind::Field { base, name } => match base.kind {
                        ExprKind::Path { mut segments } => {
                            segments.push(name);
                            segments
                        }
                        _ => {
                            self.report_code(codes::E0100, span, "this is not a pattern");
                            return Pattern { id: self.next_id(), kind: PatternKind::Error, span };
                        }
                    },
                    _ => {
                        self.report_code(codes::E0100, span, "this is not a pattern");
                        return Pattern { id: self.next_id(), kind: PatternKind::Error, span };
                    }
                };
                let fields = args
                    .into_iter()
                    .map(|arg| {
                        let arg_span = arg.span;
                        FieldPattern {
                            name: arg.name,
                            pattern: self.expr_to_pattern(arg.value),
                            span: arg_span,
                        }
                    })
                    .collect();
                PatternKind::Constructor { path, fields, has_rest: false }
            }
            ExprKind::ArrayLit(items) => PatternKind::Slice {
                prefix: items.into_iter().map(|e| self.expr_to_pattern(e)).collect(),
                rest: None,
                suffix: Vec::new(),
            },
            ExprKind::Error => PatternKind::Error,
            _ => {
                self.report(
                    Diagnostic::error(codes::E0100, span, "this is not a pattern")
                        .help("a pattern is a name, a literal, a tuple, or a constructor call"),
                );
                PatternKind::Error
            }
        };
        Pattern { id: self.next_id(), kind, span }
    }
}

#[derive(Copy, Clone)]
enum InfixOp {
    Bin(BinOp),
    Logical(LogicalOp),
    Range { inclusive: bool },
}

/// `[GRM-19]` — a pattern that matches every value of its type. Decided
/// syntactically: a binding, `_`, and tuples built only from those. A
/// `Constructor` or `Path` may still be irrefutable when the type has one
/// variant, which needs types and is checked in `ember_typeck`.
fn pattern_is_irrefutable(pattern: &Pattern) -> bool {
    match &pattern.kind {
        PatternKind::Wild => true,
        PatternKind::Bind { sub, .. } => match sub {
            Some(sub) => pattern_is_irrefutable(sub),
            None => true,
        },
        PatternKind::Tuple(items) => items.iter().all(pattern_is_irrefutable),
        _ => false,
    }
}

fn convert_literal(lit: Lit) -> Literal {
    match lit {
        Lit::Int { value, suffix } => Literal::Int { value, suffix },
        Lit::Float { value, suffix, digits } => Literal::Float { value, suffix, digits },
        Lit::Char(c) => Literal::Char(c),
        Lit::Str(s) => Literal::Str(s),
        Lit::Bytes(b) => Literal::Bytes(b),
        Lit::CStr(b) => Literal::CStr(b),
        // An f-string never reaches here: `parse_atom` handles it, and a
        // pattern cannot be an f-string.
        Lit::FStr(_) => Literal::Str(String::new()),
    }
}
