//! Recursive-descent parser with Pratt expression parsing (Part III).
//!
//! `[AST-2]` — the parser recovers at statement and item boundaries. It never
//! produces fewer than one diagnostic for a malformed region and never more
//! than three for the same region, so a single typo cannot bury the real
//! error under a cascade.

use ember_ast::{Ident, Module, NodeId, NodeIds};
use ember_diag::{Code, Diagnostic, Sink, codes};
use ember_lexer::{Kw, Punct, Token, TokenKind};
use ember_span::{FileId, Span, Symbol};

mod decls;
mod body;

/// `[MOD-6]`, `[VER-1]` — the language versions whose source this compiler
/// accepts. The set MUST include every version still accepted, so that pinning
/// an older one stays valid; it holds one entry because v0.5 changed the
/// grammar under `let`, `type`, `;` and the jump expressions, and no earlier
/// source survives those changes. It grows when `[VER-2]`'s compatibility
/// promise comes into force at 1.0.
pub const LANGUAGE_VERSIONS: &[&str] = &["0.5"];

/// Parse one file's token stream into a [`Module`].
///
/// `src` is the same normalised text the tokens came from; the parser needs it
/// to re-lex f-string interpolations (`[LEX-19]`).
pub fn parse(file: FileId, src: &str, tokens: Vec<Token>, sink: &mut Sink) -> Module {
    let mut parser = Parser::new(file, src, tokens, sink);
    parser.parse_module()
}

pub(crate) struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    /// Token streams suspended while a nested one is parsed.
    suspended: Vec<(Vec<Token>, usize)>,
    src: &'a str,
    file: FileId,
    ids: NodeIds,
    sink: &'a mut Sink,
    /// How many diagnostics have been reported without the parser making
    /// progress past them. `[AST-2]` caps this at three.
    region_errors: u32,
    /// The token index at which the current error region started.
    region_start: usize,
}

impl<'a> Parser<'a> {
    fn new(file: FileId, src: &'a str, tokens: Vec<Token>, sink: &'a mut Sink) -> Parser<'a> {
        Parser {
            tokens,
            pos: 0,
            suspended: Vec::new(),
            src,
            file,
            ids: NodeIds::new(),
            sink,
            region_errors: 0,
            region_start: usize::MAX,
        }
    }

    // -- cursor -------------------------------------------------------------

    pub(crate) fn next_id(&mut self) -> NodeId {
        self.ids.next()
    }

    fn peek(&self) -> &TokenKind {
        self.tokens.get(self.pos).map_or(&TokenKind::Eof, |t| &t.kind)
    }

    fn peek_at(&self, n: usize) -> &TokenKind {
        self.tokens.get(self.pos + n).map_or(&TokenKind::Eof, |t| &t.kind)
    }

    fn span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map_or_else(|| Span::new(self.file, 0, 0), |t| t.span)
    }

    /// The span of the token just consumed, for "expected X after Y" carets.
    fn prev_span(&self) -> Span {
        if self.pos == 0 { self.span() } else { self.tokens[self.pos - 1].span }
    }

    fn bump(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    /// Parse one expression from a sub-range of the file, then restore the
    /// outer token stream. Used for f-string interpolations.
    pub(crate) fn parse_subexpression(&mut self, span: Span) -> ember_ast::Expr {
        let sub = ember_lexer::lex_range(
            self.file,
            self.src,
            span.start as usize,
            span.end as usize,
            self.sink,
        );
        let outer = std::mem::replace(&mut self.tokens, sub.tokens);
        let outer_pos = std::mem::replace(&mut self.pos, 0);
        self.suspended.push((outer, outer_pos));
        let expr = self.parse_expr();
        let (tokens, pos) = self.suspended.pop().expect("a suspended stream to restore");
        self.tokens = tokens;
        self.pos = pos;
        expr
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), TokenKind::Eof)
    }

    fn at_kw(&self, kw: Kw) -> bool {
        matches!(self.peek(), TokenKind::Keyword(k) if *k == kw)
    }

    fn at_kw_at(&self, n: usize, kw: Kw) -> bool {
        matches!(self.peek_at(n), TokenKind::Keyword(k) if *k == kw)
    }

    fn at_punct(&self, punct: Punct) -> bool {
        matches!(self.peek(), TokenKind::Punct(p) if *p == punct)
    }

    fn at_punct_at(&self, n: usize, punct: Punct) -> bool {
        matches!(self.peek_at(n), TokenKind::Punct(p) if *p == punct)
    }

    fn at_newline(&self) -> bool {
        matches!(self.peek(), TokenKind::Newline)
    }

    fn at_ident(&self) -> bool {
        matches!(self.peek(), TokenKind::Ident(_) | TokenKind::RawIdent(_))
    }

    /// `[LEX-15]` — a contextual keyword is an ordinary identifier that the
    /// grammar treats as a keyword in one position.
    fn at_contextual(&self, text: &str) -> bool {
        matches!(self.peek(), TokenKind::Ident(s) if s.is(text))
    }

    fn eat_kw(&mut self, kw: Kw) -> bool {
        if self.at_kw(kw) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn eat_punct(&mut self, punct: Punct) -> bool {
        if self.at_punct(punct) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn eat_newlines(&mut self) {
        while self.at_newline() {
            self.bump();
        }
    }

    // -- expectations and recovery ------------------------------------------

    fn expect_punct(&mut self, punct: Punct) -> bool {
        if self.eat_punct(punct) {
            return true;
        }
        let found = self.peek().to_string();
        let span = self.span();
        self.report(
            Diagnostic::error(codes::E0100, span, format!("expected `{}`", punct.as_str()))
                .primary_label(format!("found {found}")),
        );
        false
    }

    fn expect_kw(&mut self, kw: Kw) -> bool {
        if self.eat_kw(kw) {
            return true;
        }
        let found = self.peek().to_string();
        let span = self.span();
        self.report(
            Diagnostic::error(codes::E0100, span, format!("expected `{}`", kw.as_str()))
                .primary_label(format!("found {found}")),
        );
        false
    }

    fn expect_newline(&mut self) {
        // A `##` comment at the end of a code line documents nothing. It is
        // consumed silently: a comment never affects whether code compiles
        // (owner, 2026-09-08; overrides `[LEX-11]`'s W0001 — see
        // docs/spec-errata.md ERR-007).
        while matches!(self.peek(), TokenKind::DocComment(_)) {
            self.bump();
        }
        if self.at_newline() {
            self.bump();
            return;
        }
        if self.at_eof() || matches!(self.peek(), TokenKind::Dedent) {
            return;
        }
        // A statement whose expression ended by closing an indented block —
        // `n = match d:` and its arms — consumed the line's end together with
        // that block's `Dedent`, so there is no newline left to expect.
        if self.pos > 0 && matches!(self.tokens[self.pos - 1].kind, TokenKind::Dedent) {
            return;
        }
        // `[GRM-18]` (`OQ-25`) — `;` is punctuation inside `[T; N]` and
        // `[v; N]` and nowhere else. One line carries one statement, so a `;`
        // between two of them gets its own code and its own fix rather than a
        // bare "expected the end of the line".
        if self.at_punct(Punct::Semi) {
            let span = self.span();
            self.report(
                Diagnostic::error(codes::E0105, span, "`;` is not a statement separator")
                    .primary_label("one line carries one statement")
                    .help("put each statement on its own line"),
            );
            self.bump();
            return;
        }
        let found = self.peek().to_string();
        let span = self.span();
        self.report(
            Diagnostic::error(codes::E0100, span, "expected the end of the line")
                .primary_label(format!("found {found}")),
        );
        self.recover_to_line_end();
    }

    fn expect_ident(&mut self) -> Ident {
        match self.peek() {
            TokenKind::Ident(name) | TokenKind::RawIdent(name) => {
                let name = *name;
                let span = self.span();
                self.bump();
                Ident { name, span }
            }
            other => {
                let found = other.to_string();
                let span = self.span();
                self.report(
                    Diagnostic::error(codes::E0100, span, "expected a name")
                        .primary_label(format!("found {found}")),
                );
                Ident { name: Symbol::intern("<error>"), span }
            }
        }
    }

    /// Emit a diagnostic, respecting `[AST-2]`'s cascade limit.
    fn report(&mut self, diagnostic: Diagnostic) {
        if self.region_start == usize::MAX || self.pos > self.region_start {
            // Real progress since the last error: a new region.
            self.region_start = self.pos;
            self.region_errors = 0;
        }
        self.region_errors += 1;
        if self.region_errors <= 3 {
            self.sink.emit(diagnostic);
        }
    }

    fn report_code(&mut self, code: Code, span: Span, message: impl Into<String>) {
        self.report(Diagnostic::error(code, span, message));
    }

    /// Skip to just past the next `NEWLINE` at the current bracket level.
    fn recover_to_line_end(&mut self) {
        while !self.at_eof() {
            match self.peek() {
                TokenKind::Newline => {
                    self.bump();
                    return;
                }
                TokenKind::Dedent | TokenKind::Indent => return,
                _ => {
                    self.bump();
                }
            }
        }
    }

    /// Skip to the start of something that could begin an item, so one broken
    /// declaration does not swallow the rest of the file.
    fn recover_to_item(&mut self) {
        while !self.at_eof() {
            if matches!(
                self.peek(),
                TokenKind::Keyword(
                    Kw::Fn
                        | Kw::Struct
                        | Kw::Class
                        | Kw::Enum
                        | Kw::Interface
                        | Kw::Extend
                        | Kw::Const
                        | Kw::Static
                        | Kw::Pub
                        | Kw::Import
                        | Kw::Extern
                        | Kw::Open
                )
            ) {
                return;
            }
            if matches!(self.peek(), TokenKind::Punct(Punct::At)) {
                return;
            }
            self.bump();
        }
    }

    /// Consume any run of doc comments and join them, so that
    ///
    /// ```text
    /// ## first line
    /// ## second line
    /// fn f(): pass
    /// ```
    ///
    /// documents `f` with both lines.
    fn take_doc(&mut self) -> Option<String> {
        let mut lines: Vec<String> = Vec::new();
        while let TokenKind::DocComment(text) = self.peek() {
            lines.push(text.clone());
            self.bump();
            self.eat_newlines();
        }
        (!lines.is_empty()).then(|| lines.join("\n"))
    }

    /// A doc comment with no declaration after it is discarded in silence.
    ///
    /// `[LEX-11]` specifies `W0001` here. The owner overrode that on
    /// 2026-09-08: a comment must never affect compilation, warnings included.
    /// See docs/spec-errata.md ERR-007.
    fn discard_dangling_doc(&mut self, doc: Option<String>) {
        let _ = doc;
    }

    // -- entry point --------------------------------------------------------

    fn parse_module(&mut self) -> Module {
        let start = self.span();
        let directive = self.parse_directive();
        self.eat_newlines();

        let mut imports = Vec::new();
        let mut items = Vec::new();

        loop {
            self.eat_newlines();
            if self.at_eof() {
                break;
            }
            if self.at_kw(Kw::Import) || self.at_contextual("from") {
                if let Some(import) = self.parse_import() {
                    imports.push(import);
                }
                continue;
            }
            let before = self.pos;
            match self.parse_item() {
                Some(item) => items.push(item),
                None => {
                    if self.pos == before {
                        self.bump();
                    }
                    self.recover_to_item();
                }
            }
        }

        Module { directive, imports, items, span: start.to(self.prev_span()) }
    }

    fn parse_directive(&mut self) -> Option<ember_ast::Directive> {
        let span = self.span();
        let TokenKind::Directive { name, value } = self.peek().clone() else {
            return None;
        };
        self.bump();
        // `[MOD-6]` — the directive was parsed and then read by nobody, so a
        // file could pin any version at all and compile.
        if name.is("language") {
            if !LANGUAGE_VERSIONS.contains(&value.as_str()) {
                self.report(
                    Diagnostic::error(
                        codes::E0006,
                        span,
                        format!("this compiler does not support language version `{value}`"),
                    )
                    .help(format!("it supports {}", LANGUAGE_VERSIONS.join(", "))),
                );
            }
        } else {
            self.report(
                Diagnostic::error(
                    codes::E0006,
                    span,
                    format!("`#! {name}` is not a directive"),
                )
                .help("the only directive is `#! language \"<version>\"`"),
            );
        }
        Some(ember_ast::Directive { name: Ident { name, span }, value, span })
    }
}
