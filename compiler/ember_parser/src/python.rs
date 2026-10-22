//! `[DIA-21]` — Python's statement and expression forms, recognised where they are written and
//! answered with the Ember form as a fix-it (ODR-179). Where the fix is a rewrite of tokens, the
//! tokens are rewritten to it, so what follows parses as the fixed program would and nothing
//! cascades from one habit (`[DIA-14]`). Each form recognised here is one Ember never accepts.

use ember_ast::*;
use ember_diag::habits::Habit;
use ember_diag::{Diagnostic, Replacement, codes};
use ember_lexer::{Kw, Punct, Token, TokenKind};
use ember_span::{Span, Symbol};

use crate::Parser;

/// Python's built-in exception types: `raise ValueError(msg)` fails with a message.
const PYTHON_EXCEPTIONS: [&str; 14] = [
    "Exception", "ValueError", "TypeError", "KeyError", "IndexError", "RuntimeError", "ZeroDivisionError",
    "OverflowError", "AssertionError", "NotImplementedError", "FileNotFoundError", "IOError", "OSError",
    "LookupError",
];

impl Parser<'_> {
    fn at_word(&self, n: usize, word: &str) -> bool {
        matches!(self.peek_at(n), TokenKind::Ident(s) if s.is(word))
    }

    fn at_name_at(&self, n: usize) -> bool {
        matches!(self.peek_at(n), TokenKind::Ident(_) | TokenKind::RawIdent(_))
    }

    /// The span from the start of token `n` ahead to the start of the one after it: a word and the
    /// space after it, which a fix-it removes.
    fn word_and_space(&self, n: usize) -> Span {
        let here = self.tokens[self.pos + n].span;
        let next = self.tokens.get(self.pos + n + 1).map_or(here, |t| t.span);
        Span::new(here.file, here.start, next.start.max(here.end))
    }

    /// P2 — `def f(…):` declares a function with `fn`; the word is rewritten to it.
    pub(crate) fn python_def(&mut self) {
        if !(self.at_word(0, "def") && self.at_name_at(1) && (self.at_punct_at(2, Punct::LParen) || self.at_punct_at(2, Punct::LBracket))) {
            return;
        }
        let span = self.span();
        self.report(
            Diagnostic::error(codes::E0100, span, "`def` is Python's; Ember declares a function with `fn`")
                .habit(Habit::P2)
                .suggest("write `fn`", span, "fn"),
        );
        self.tokens[self.pos].kind = TokenKind::Keyword(Kw::Fn);
    }

    /// P9 — `let x = 5`, `var x = 5`: an assignment declares a name not in scope (`[GRM-4]`), so the
    /// word is dropped.
    pub(crate) fn python_declaration_word(&mut self) {
        let word = if self.at_kw(Kw::Let) {
            "let"
        } else if self.at_word(0, "var") {
            "var"
        } else {
            return;
        };
        if !(self.at_name_at(1) && (self.at_punct_at(2, Punct::Eq) || self.at_punct_at(2, Punct::Colon))) {
            return;
        }
        let removed = self.word_and_space(0);
        self.report(
            Diagnostic::error(codes::E0100, self.span(), format!("`{word}` is not needed: assigning a name declares it"))
                .habit(Habit::P9)
                .note("`x = 5` declares `x` where no `x` is in scope, and assigns it where one is [GRM-4]")
                .suggest(format!("remove `{word}`"), removed, ""),
        );
        self.tokens.remove(self.pos);
    }

    /// P13 — `lambda x, y: e` is `fn(x, y) => e`; the tokens are rewritten to it.
    pub(crate) fn python_lambda(&mut self) {
        if !self.at_word(0, "lambda") || !self.at_name_at(1) {
            return;
        }
        // The parameters: names separated by commas, up to the `:`.
        let mut at = 1;
        while self.at_name_at(at) {
            if self.at_punct_at(at + 1, Punct::Comma) {
                at += 2;
            } else if self.at_punct_at(at + 1, Punct::Colon) {
                break;
            } else {
                return;
            }
        }
        let colon = at + 1;
        if !self.at_punct_at(colon, Punct::Colon) {
            return;
        }
        let lambda = self.span();
        let first = self.tokens[self.pos + 1].span;
        let colon_span = self.tokens[self.pos + colon].span;
        self.report(
            Diagnostic::error(codes::E0100, lambda, "`lambda` is Python's; an Ember lambda is `fn(x) => e`")
                .habit(Habit::P13)
                .suggest_edits(
                    "write `fn(…) =>`",
                    vec![
                        (Span::new(lambda.file, lambda.start, first.start), Replacement::Text("fn(".to_string())),
                        (colon_span, Replacement::Text(") =>".to_string())),
                    ],
                ),
        );
        let token = |kind: TokenKind, span: Span| Token { kind, span };
        self.tokens[self.pos + colon] = token(TokenKind::Punct(Punct::FatArrow), colon_span);
        self.tokens.insert(self.pos + colon, token(TokenKind::Punct(Punct::RParen), colon_span.shrink_to_start()));
        self.tokens[self.pos] = token(TokenKind::Keyword(Kw::Fn), lambda);
        self.tokens.insert(self.pos + 1, token(TokenKind::Punct(Punct::LParen), first.shrink_to_start()));
    }

    /// P19 — `*args` or `**kwargs` in a signature: Ember has no variadic functions. The stars are
    /// skipped, so the parameter is read as the one it would be.
    pub(crate) fn python_star_param(&mut self) {
        let stars = if self.at_punct(Punct::Star) {
            1
        } else if self.at_punct(Punct::StarStar) {
            2
        } else {
            return;
        };
        if !self.at_name_at(1) {
            return;
        }
        let span = self.span().to(self.tokens[self.pos + 1].span);
        let help = if stars == 1 {
            "take the values as one parameter, an `Array[T]` or a `Span[T]`, and pass `[a, b]`"
        } else {
            "declare each keyword argument as a parameter with a default: `fn f(width: int = 80)`"
        };
        self.report(
            Diagnostic::error(codes::E0100, span, format!("`{}` is Python's; Ember has no variadic functions", "*".repeat(stars)))
                .habit(Habit::P19)
                .help(help)
                .note("a call passes exactly the parameters the signature declares (I.6)"),
        );
        self.bump();
    }

    /// P12, P18 — Python's `raise`, `try`/`except`/`finally`, `global` and `nonlocal`, at the start
    /// of a statement. `Some` when the statement was one of them: what it is read as.
    pub(crate) fn python_statement(&mut self) -> Option<StmtKind> {
        let starts_value = |kind: &TokenKind| matches!(kind, TokenKind::Ident(_) | TokenKind::RawIdent(_) | TokenKind::Lit(_) | TokenKind::Keyword(Kw::SelfValue));
        if self.at_word(0, "raise") && starts_value(self.peek_at(1)) {
            return Some(self.python_raise());
        }
        if (self.at_word(0, "try") || self.at_word(0, "finally")) && self.at_punct_at(1, Punct::Colon) && matches!(self.peek_at(2), TokenKind::Newline)
            || self.at_word(0, "except") && (self.at_punct_at(1, Punct::Colon) || self.at_name_at(1))
        {
            return Some(self.python_try());
        }
        if (self.at_word(0, "global") || self.at_word(0, "nonlocal")) && self.at_name_at(1) && matches!(self.peek_at(2), TokenKind::Newline | TokenKind::Punct(Punct::Comma)) {
            let word = if self.at_word(0, "global") { "global" } else { "nonlocal" };
            let span = self.span();
            self.report(
                Diagnostic::error(codes::E0100, span, format!("`{word}` is Python's; Ember declares no names as shared"))
                    .habit(Habit::P18)
                    .help("keep shared state in a `static` or a class field; a lambda or a local function captures a name of the function around it without a declaration [CLO-2]"),
            );
            while !self.at_newline() && !self.at_eof() {
                self.bump();
            }
            return Some(StmtKind::Pass);
        }
        None
    }

    /// P12 — `raise e` is `return Err(e)`, and `raise ValueError(msg)` a failure with a message,
    /// `return Err(AnyError.msg(msg))` (ODR-180).
    fn python_raise(&mut self) -> StmtKind {
        let raise = self.span();
        self.bump();
        let value = self.parse_expr();
        let message = match &value.kind {
            ExprKind::Call { callee, args } if args.len() == 1 && args[0].name.is_none() => match &callee.kind {
                ExprKind::Path { segments } if segments.len() == 1 && PYTHON_EXCEPTIONS.contains(&segments[0].name.as_str()) => {
                    Some((callee.span, args[0].value.span))
                }
                _ => None,
            },
            _ => None,
        };
        let diagnostic = Diagnostic::error(codes::E0100, raise, "`raise` is Python's; an Ember function fails by returning `Err`")
            .habit(Habit::P12)
            .note("a function that can fail returns a `Result`; `?` passes a failure on to the caller (Part XIII)");
        let start = Span::new(raise.file, raise.start, value.span.start);
        let edits = match message {
            Some((exception, text)) => vec![
                (start, Replacement::Text("return Err(AnyError.msg(".to_string())),
                (Span::new(raise.file, exception.start, text.start), Replacement::Text(String::new())),
                (Span::new(raise.file, text.end, value.span.end), Replacement::Text("))".to_string())),
            ],
            None => vec![(start, Replacement::Text("return Err(".to_string())), (value.span.shrink_to_end(), Replacement::Text(")".to_string()))],
        };
        self.report(diagnostic.suggest_edits("write `return Err(…)`", edits));
        let id = self.next_id();
        let err = Expr { id, kind: ExprKind::Path { segments: vec![Ident { name: Symbol::intern("Err"), span: raise }] }, span: raise };
        let span = raise.to(value.span);
        let call = Expr { id: self.next_id(), kind: ExprKind::Call { callee: Box::new(err), args: vec![Arg { name: None, span: value.span, value }] }, span };
        StmtKind::Expr(Expr { id: self.next_id(), kind: ExprKind::Jump(Jump::Return(Some(Box::new(call)))), span })
    }

    /// P12 — `try:` with its `except` and `finally` clauses: Ember has no exceptions. Reported
    /// once, at the first clause; the clauses are read and set aside.
    fn python_try(&mut self) -> StmtKind {
        let span = self.span();
        let word = if self.at_word(0, "try") { "try" } else if self.at_word(0, "except") { "except" } else { "finally" };
        self.report(
            Diagnostic::error(codes::E0100, span, format!("`{word}` is Python's; Ember has no exceptions"))
                .habit(Habit::P12)
                .help("a call that can fail returns a `Result`: `?` after it passes the failure on, `match r:` with `Ok(v):` and `Err(e):` arms handles it here, and `defer:` runs a block when its scope ends, as `finally` does (Part XIII)"),
        );
        while self.at_word(0, "try") || self.at_word(0, "except") || self.at_word(0, "finally") || (self.at_kw(Kw::Else) && self.at_punct_at(1, Punct::Colon)) {
            while !self.at_punct(Punct::Colon) && !self.at_newline() && !self.at_eof() {
                self.bump();
            }
            if !self.eat_punct(Punct::Colon) {
                break;
            }
            let sink_mark = self.sink.mark();
            let _ = self.parse_block();
            // The clauses' own mistakes are the same construct's (`[DIA-14]`).
            self.sink.rollback(sink_mark);
            self.eat_newlines();
        }
        StmtKind::Pass
    }

    /// P11 — `with open(p) as f:` binds first, `with f = fs.File.open(p)?:`. `value` is what was
    /// read after `with`; when it is Python's form (`e as name` before the `:`), the binding it
    /// means, with its value an error already reported, so the block checks without cascading.
    pub(crate) fn python_with(&mut self, value: &Expr) -> Option<(Pattern, Expr)> {
        let ExprKind::Cast { expr, ty } = &value.kind else { return None };
        let TypeKind::Path { segments, args } = &ty.kind else { return None };
        let ([name], true, true) = (segments.as_slice(), args.is_empty(), self.at_punct(Punct::Colon)) else { return None };
        let file = value.span.file;
        let opened = match &expr.kind {
            ExprKind::Call { callee, args } => match &callee.kind {
                ExprKind::Path { segments } if segments.len() == 1 && segments[0].name.is("open") && !args.is_empty() => {
                    let inside = Span::new(file, args[0].span.start, args[args.len() - 1].span.end);
                    Some(self.src[inside.start as usize..inside.end as usize].to_string())
                }
                _ => None,
            },
            _ => None,
        };
        // The whole `e as name`, to the end of the name.
        let written = Span::new(file, value.span.start, ty.span.end.max(value.span.end));
        let diagnostic = Diagnostic::error(codes::E0100, written, "`with … as name` is Python's; Ember binds first, `with name = …:`")
            .habit(Habit::P11)
            .note("what a `with` binds is closed or released when its block ends [THR-5]");
        let mut edits = Vec::new();
        match opened {
            Some(path) => {
                edits.push((written, Replacement::Text(format!("{} = fs.File.open({path})?", name.name))));
                if !self.fs_imported {
                    let (at, line) = match self.import_at {
                        Some((at, true)) => (at.shrink_to_end(), "\nimport std.fs"),
                        Some((at, false)) => (at, "import std.fs\n\n"),
                        None => (Span::new(file, 0, 0), "import std.fs\n\n"),
                    };
                    edits.push((at, Replacement::Text(line.to_string())));
                }
            }
            None => {
                edits.push((Span::new(file, value.span.start, value.span.start), Replacement::Text(format!("{} = ", name.name))));
                edits.push((Span::new(file, expr.span.end, written.end), Replacement::Text(String::new())));
            }
        }
        self.report(diagnostic.suggest_edits(format!("write `with {} = …:`", name.name), edits));
        let pattern = Pattern { id: self.next_id(), kind: PatternKind::Bind { name: name.clone(), by_ref: false, mutable: false, sub: None }, span: name.span };
        Some((pattern, Expr { id: self.next_id(), kind: ExprKind::Error, span: value.span }))
    }

    /// P14 — a method of a type whose body uses `self` and whose parameters do not take it: Python's
    /// `def m():` with `self` left out. The body's tokens are `body`; the receiver is added (borrowed),
    /// so the method checks as the fixed one would.
    pub(crate) fn python_missing_self(&mut self, lparen: Span, params: &mut Vec<Param>, body: std::ops::Range<usize>) {
        if params.iter().any(|param| matches!(param.kind, ParamKind::Receiver { .. })) {
            return;
        }
        let Some(used) = self.tokens[body].iter().find(|token| matches!(token.kind, TokenKind::Keyword(Kw::SelfValue))).map(|token| token.span) else {
            return;
        };
        let after = lparen.shrink_to_end();
        let text = if params.is_empty() { "self" } else { "self, " };
        self.sink.emit(
            Diagnostic::error(codes::E1010, used, "`self` in a method that does not take it")
                .habit(Habit::P14)
                .secondary(lparen, "the parameters, without `self`")
                .note("a method takes its receiver as its first parameter, `self`, as Python's does; one without it is an associated function [FN-4]")
                .suggest("take `self`", after, text),
        );
        let id = self.next_id();
        params.insert(0, Param { id, mode: Mode::Borrow, kind: ParamKind::Receiver { ty: None }, default: None, span: after });
    }

    /// P17 — `case` before a `match` arm's pattern, which Ember writes bare: the word and the space
    /// after it, removed, for one fix-it over the whole `match`.
    pub(crate) fn python_case(&mut self) -> Option<Span> {
        let pattern_follows = !matches!(self.peek_at(1), TokenKind::Punct(Punct::Colon | Punct::FatArrow | Punct::LParen | Punct::Dot) | TokenKind::Keyword(Kw::If));
        if !self.at_word(0, "case") || !pattern_follows || matches!(self.peek_at(1), TokenKind::Newline) {
            return None;
        }
        let removed = self.word_and_space(0);
        self.bump();
        Some(removed)
    }

    /// P17 — the one diagnostic for a `match` whose arms began with `case`.
    pub(crate) fn report_python_cases(&mut self, cases: Vec<Span>) {
        let Some(&first) = cases.first() else { return };
        let word = Span::new(first.file, first.start, first.start + 4);
        let edits = cases.into_iter().map(|span| (span, Replacement::Text(String::new()))).collect();
        self.sink.emit(
            Diagnostic::error(codes::E0100, word, "`case` is Python's; an Ember `match` arm is its pattern")
                .habit(Habit::P17)
                .note("`pattern:` and a block, or `pattern =>` and an expression [GRM-10]")
                .suggest_edits("remove `case`", edits),
        );
    }

    /// P28 — a statement (`x = …`, `pass`) after a `=>`, which takes an expression: the arms are
    /// statement arms, `pattern: statement` (`[GRM-10]`); the fix-it turns every arm's `=>` into `:`.
    pub(crate) fn report_python_statement_arms(&mut self, first: Span, arrows: Vec<Span>) {
        // The `=>` and the spaces before it: `Pat:` is written against its pattern.
        let edits = arrows
            .into_iter()
            .map(|span| {
                let mut start = span.start as usize;
                while start > 0 && self.src.as_bytes()[start - 1] == b' ' {
                    start -= 1;
                }
                (Span::new(span.file, start as u32, span.end), Replacement::Text(":".to_string()))
            })
            .collect();
        self.sink.emit(
            Diagnostic::error(codes::E0100, first, "a `=>` arm is an expression, and this is a statement")
                .habit(Habit::P28)
                .note("a `match` whose arms do something is written with `pattern:` arms, a statement on the line or a block below [GRM-10]")
                .suggest_edits("write every arm `pattern:`", edits),
        );
    }
}
