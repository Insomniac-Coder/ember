//! The lexer: source text to a token stream (Part II).
//!
//! Indentation is significant in exactly the way Python 3's is, with the
//! stricter rules of `[LEX-4]` (spaces only) and `[LEX-9]` (a block must
//! follow a `:`). `[II.7]` — the lexer never fails fatally: an unrecognised
//! character becomes an `Error` token and lexing continues, so the parser
//! still sees the shape of the file.

use ember_diag::{Diagnostic, Sink, codes};
use ember_span::{FileId, Span, Symbol};
use unicode_normalization::UnicodeNormalization;

pub mod token;
pub use token::{CONTEXTUAL_KEYWORDS, FStrPart, IntSuffix, Kw, Lit, Punct, Reserved, Token, TokenKind};

/// A comment the lexer discarded. `[II.7]` — ordinary comments are not tokens,
/// but their spans are recorded so the formatter can put them back.
#[derive(Clone, Debug)]
pub struct Comment {
    pub span: Span,
    /// True when the comment is the only thing on its line, which the
    /// formatter needs in order to decide between a leading and a trailing
    /// comment.
    pub own_line: bool,
}

pub struct LexResult {
    pub tokens: Vec<Token>,
    pub comments: Vec<Comment>,
}

/// Tokenise one already-normalised source file (`[LEX-2]`: LF endings, no BOM).
pub fn lex(file: FileId, src: &str, sink: &mut Sink) -> LexResult {
    let mut lexer = Lexer::new(file, src, sink);
    lexer.run();
    LexResult { tokens: lexer.tokens, comments: lexer.comments }
}

/// Tokenise one sub-range of a file, for re-parsing an f-string's
/// interpolations (`[LEX-19]`).
///
/// The source is truncated at `end` rather than sliced, so every span the
/// sub-stream carries is still an absolute offset into the file and
/// diagnostics land on the real line.
pub fn lex_range(
    file: FileId,
    src: &str,
    start: usize,
    end: usize,
    sink: &mut Sink,
) -> LexResult {
    let mut lexer = Lexer::new(file, &src[..end.min(src.len())], sink);
    lexer.pos = start.min(end);
    // An interpolation is a bracketed context: indentation is not significant
    // inside it, and a newline would be a line join rather than a line end.
    lexer.bracket_depth = 1;
    lexer.run();
    LexResult { tokens: lexer.tokens, comments: lexer.comments }
}

struct Lexer<'a> {
    file: FileId,
    src: &'a str,
    pos: usize,
    tokens: Vec<Token>,
    comments: Vec<Comment>,
    /// `[LEX-5]` — the indent stack, starting at `[0]`.
    indents: Vec<u32>,
    /// `[LEX-6]` — inside brackets, newlines are not logical line ends.
    bracket_depth: u32,
    /// Whether anything on the current logical line would justify a `NEWLINE`.
    line_has_tokens: bool,
    /// Doc comments read from comment-only lines, held back until the next
    /// content line's `INDENT`/`DEDENT` have been emitted, so a doc comment
    /// written inside a block lands inside that block.
    pending_docs: Vec<Token>,
    /// Punctuation spellings, longest first (`[LEX-21]`).
    puncts: Vec<(&'static str, Punct)>,
    sink: &'a mut Sink,
}

impl<'a> Lexer<'a> {
    fn new(file: FileId, src: &'a str, sink: &'a mut Sink) -> Lexer<'a> {
        Lexer {
            file,
            src,
            pos: 0,
            tokens: Vec::new(),
            comments: Vec::new(),
            indents: vec![0],
            bracket_depth: 0,
            line_has_tokens: false,
            pending_docs: Vec::new(),
            puncts: Punct::by_length_desc(),
            sink,
        }
    }

    // -- cursor -------------------------------------------------------------

    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }

    fn peek_nth(&self, n: usize) -> Option<char> {
        self.src[self.pos..].chars().nth(n)
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += c.len_utf8();
            true
        } else {
            false
        }
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn at_eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn span(&self, start: usize) -> Span {
        Span::new(self.file, start as u32, self.pos as u32)
    }

    fn push(&mut self, kind: TokenKind, start: usize) {
        let span = self.span(start);
        self.line_has_tokens = true;
        self.tokens.push(Token { kind, span });
    }

    fn push_structural(&mut self, kind: TokenKind, at: usize) {
        let span = Span::new(self.file, at as u32, at as u32);
        self.tokens.push(Token { kind, span });
    }

    // -- driver -------------------------------------------------------------

    fn run(&mut self) {
        self.lex_directive();

        while !self.at_eof() {
            if self.bracket_depth == 0 && self.at_line_start() {
                if self.lex_indentation() {
                    continue;
                }
            }
            self.lex_token();
        }

        // Close the file: a trailing NEWLINE, then one DEDENT per open level.
        let end = self.src.len();
        if self.line_has_tokens {
            self.push_structural(TokenKind::Newline, end);
            self.line_has_tokens = false;
        }
        while self.indents.len() > 1 {
            self.indents.pop();
            self.push_structural(TokenKind::Dedent, end);
        }
        let docs = std::mem::take(&mut self.pending_docs);
        self.tokens.extend(docs);
        self.push_structural(TokenKind::Eof, end);
    }

    /// True when `pos` sits at the first byte of a physical line.
    fn at_line_start(&self) -> bool {
        self.pos == 0 || self.src.as_bytes()[self.pos - 1] == b'\n'
    }

    /// `#! name "value"` — recognised on the first line of a file only. `#!`
    /// anywhere else is an ordinary comment, which is what lets test files
    /// carry `#! error[...]` annotations that the harness reads out of band
    /// (`[TST-1]`).
    fn lex_directive(&mut self) {
        if !self.rest().starts_with("#!") {
            return;
        }
        let start = self.pos;
        self.pos += 2;
        let line_end = self.rest().find('\n').map_or(self.src.len(), |i| self.pos + i);
        let body = self.src[self.pos..line_end].trim();
        self.pos = line_end;

        let mut parts = body.splitn(2, char::is_whitespace);
        let name = parts.next().unwrap_or("").trim();
        let value = parts.next().unwrap_or("").trim().trim_matches('"').to_string();
        if name.is_empty() {
            let span = self.span(start);
            let d = Diagnostic::error(codes::E0006, span, "empty directive");
            self.sink.emit(d);
        } else {
            let span = self.span(start);
            let kind = TokenKind::Directive { name: Symbol::intern(name), value };
            self.tokens.push(Token { kind, span });
        }
        self.eat('\n');
    }

    /// Handle the indentation of one physical line.
    ///
    /// Returns true when the whole line was consumed here (a blank or
    /// comment-only line, `[LEX-8]`), so the caller should start over.
    fn lex_indentation(&mut self) -> bool {
        let indent_start = self.pos;
        let mut width = 0u32;
        let mut tab_seen = false;
        loop {
            match self.peek() {
                Some(' ') => {
                    self.pos += 1;
                    width += 1;
                }
                Some('\t') => {
                    // `[LEX-4]`: a tab in indentation is E0002. Keep going,
                    // treating it as the fix-it would, so one stray tab does
                    // not cascade into indentation errors on every later line.
                    tab_seen = true;
                    self.pos += 1;
                    width = (width / 4 + 1) * 4;
                }
                _ => break,
            }
        }

        if tab_seen {
            let span = Span::new(self.file, indent_start as u32, self.pos as u32);
            let replacement = " ".repeat(width as usize);
            let d = Diagnostic::error(codes::E0002, span, "tab used for indentation")
                .primary_label("indentation must use spaces")
                .suggest("replace the tab with spaces", span, replacement);
            self.sink.emit(d);
        }

        // `[LEX-8]`: blank and comment-only lines do not touch the stack.
        match self.peek() {
            None => return true,
            Some('\n') => {
                self.pos += 1;
                return true;
            }
            Some('#') => {
                self.lex_comment(true);
                return true;
            }
            _ => {}
        }

        let top = *self.indents.last().expect("indent stack is never empty");
        if width > top {
            self.indents.push(width);
            self.push_structural(TokenKind::Indent, self.pos);
        } else if width < top {
            while *self.indents.last().unwrap() > width {
                self.indents.pop();
                self.push_structural(TokenKind::Dedent, self.pos);
            }
            if *self.indents.last().unwrap() != width {
                // `[LEX-5]`: an indentation that is not on the stack.
                let span = Span::new(self.file, indent_start as u32, self.pos as u32);
                let expected = *self.indents.last().unwrap();
                let d = Diagnostic::error(codes::E0003, span, "inconsistent dedent")
                    .primary_label(format!(
                        "this line is indented {width} spaces, which does not match any enclosing block"
                    ))
                    .help(format!("the nearest enclosing block is indented {expected} spaces"));
                self.sink.emit(d);
                // Resynchronise so the rest of the file still parses.
                self.indents.push(width);
            }
        }

        let docs = std::mem::take(&mut self.pending_docs);
        self.tokens.extend(docs);
        false
    }

    fn lex_token(&mut self) {
        match self.peek() {
            None => {}
            Some(' ') | Some('\t') | Some('\r') => {
                self.pos += 1;
            }
            Some('\n') => {
                let at = self.pos;
                self.pos += 1;
                if self.bracket_depth == 0 && self.line_has_tokens {
                    self.push_structural(TokenKind::Newline, at);
                    self.line_has_tokens = false;
                }
            }
            Some('\\') if self.peek_nth(1) == Some('\n') => {
                // `[LEX-7]`: a backslash joins this physical line to the next.
                self.pos += 2;
                while matches!(self.peek(), Some(' ') | Some('\t')) {
                    self.pos += 1;
                }
            }
            Some('#') => self.lex_comment(false),
            Some('"') => self.lex_string_like(StringFlavour::Str),
            Some('\'') => self.lex_char_or_lifetime(),
            Some(c) if c.is_ascii_digit() => self.lex_number(),
            Some(c) if is_ident_start(c) => self.lex_ident_or_prefixed_literal(),
            Some(_) => self.lex_punct(),
        }
    }

    // -- comments -----------------------------------------------------------

    fn lex_comment(&mut self, own_line: bool) {
        let start = self.pos;
        self.pos += 1; // '#'
        let is_doc = self.eat('#');
        let line_end = self.rest().find('\n').map_or(self.src.len(), |i| self.pos + i);
        let text = self.src[self.pos..line_end].to_string();
        self.pos = line_end;
        let span = Span::new(self.file, start as u32, self.pos as u32);

        if is_doc {
            let tok = Token { kind: TokenKind::DocComment(text.trim().to_string()), span };
            if own_line {
                // Held back so the doc lands after the block's INDENT.
                self.pending_docs.push(tok);
            } else {
                self.line_has_tokens = true;
                self.tokens.push(tok);
            }
        } else {
            self.comments.push(Comment { span, own_line });
        }

        if own_line {
            self.eat('\n');
        }
    }

    // -- identifiers and keywords -------------------------------------------

    fn lex_ident_or_prefixed_literal(&mut self) {
        let start = self.pos;

        // Literal prefixes: r"…", r#"…"#, b"…", c"…", f"…", and r#ident.
        if let Some(c) = self.peek() {
            let next = self.peek_nth(1);
            match (c, next) {
                ('r', Some('"')) | ('r', Some('#')) => {
                    // `r#ident` (`[LEX-14]`) versus a raw string `r#"…"#`.
                    let is_raw_ident = next == Some('#')
                        && self.peek_nth(2).is_some_and(|c2| is_ident_start(c2));
                    if is_raw_ident {
                        self.pos += 2; // r#
                        let name_start = self.pos;
                        self.eat_ident_run();
                        let text = normalise_ident(&self.src[name_start..self.pos]);
                        self.push(TokenKind::RawIdent(Symbol::intern(&text)), start);
                        return;
                    }
                    self.lex_string_like(StringFlavour::Raw);
                    return;
                }
                ('b', Some('"')) => {
                    self.lex_string_like(StringFlavour::Bytes);
                    return;
                }
                ('c', Some('"')) => {
                    self.lex_string_like(StringFlavour::CStr);
                    return;
                }
                ('f', Some('"')) => {
                    self.lex_string_like(StringFlavour::FStr);
                    return;
                }
                _ => {}
            }
        }

        self.eat_ident_run();
        let raw = &self.src[start..self.pos];
        let text = normalise_ident(raw);

        if let Some(kw) = Kw::from_str(&text) {
            self.push(TokenKind::Keyword(kw), start);
        } else if let Some(res) = Reserved::from_str(&text) {
            self.push(TokenKind::Reserved(res), start);
        } else {
            self.push(TokenKind::Ident(Symbol::intern(&text)), start);
        }
    }

    fn eat_ident_run(&mut self) {
        while let Some(c) = self.peek() {
            if is_ident_continue(c) {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    // -- numbers ------------------------------------------------------------

    fn lex_number(&mut self) {
        let start = self.pos;

        let radix = if self.peek() == Some('0') {
            match self.peek_nth(1) {
                Some('x') | Some('X') => 16,
                Some('o') | Some('O') => 8,
                Some('b') | Some('B') => 2,
                _ => 10,
            }
        } else {
            10
        };

        if radix != 10 {
            self.pos += 2;
            let digits_start = self.pos;
            while let Some(c) = self.peek() {
                if c == '_' || c.is_digit(radix) {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let digits: String = self.src[digits_start..self.pos].chars().filter(|&c| c != '_').collect();
            let suffix = self.lex_number_suffix();
            if digits.is_empty() {
                let span = self.span(start);
                let d = Diagnostic::error(codes::E0100, span, "integer literal has no digits")
                    .primary_label(format!("expected digits after the base-{radix} prefix"));
                self.sink.emit(d);
                self.push(TokenKind::Error, start);
                return;
            }
            let value = u128::from_str_radix(&digits, radix).unwrap_or_else(|_| {
                let span = self.span(start);
                let d = Diagnostic::error(codes::E2010, span, "integer literal is too large for any integer type");
                self.sink.emit(d);
                0
            });
            self.finish_number(start, Lit::Int { value, suffix: int_suffix(&suffix) }, &suffix, false);
            return;
        }

        // Decimal: integer part, then an optional fraction and exponent.
        self.eat_decimal_run();
        let mut is_float = false;

        if self.peek() == Some('.') {
            let after = self.peek_nth(1);
            // `[LEX-18]`: `1.` is a float only when a fraction can follow.
            // `1..5` is a range and `1.abs()` is a method call.
            let is_fraction = !matches!(after, Some('.')) && !after.is_some_and(is_ident_start_or_underscore);
            if is_fraction {
                is_float = true;
                self.pos += 1;
                self.eat_decimal_run();
            }
        }

        if matches!(self.peek(), Some('e') | Some('E')) {
            let sign_len = usize::from(matches!(self.peek_nth(1), Some('+') | Some('-')));
            let digit_at = 1 + sign_len;
            if self.peek_nth(digit_at).is_some_and(|c| c.is_ascii_digit()) {
                is_float = true;
                self.pos += 1 + sign_len;
                self.eat_decimal_run();
            }
        }

        let text: String = self.src[start..self.pos].chars().filter(|&c| c != '_').collect();
        let suffix = self.lex_number_suffix();

        // `1f32` is accepted as a float literal even though Part II's grammar
        // attaches float suffixes to float literals only. See docs/spec-errata.md
        // ERR-002; rejecting it would be a papercut with nothing bought.
        if is_float || float_suffix(&suffix).is_some() {
            let value: f64 = text.parse().unwrap_or(0.0);
            self.finish_number(start, Lit::Float { value, suffix: float_suffix(&suffix) }, &suffix, true);
        } else {
            let value = text.parse::<u128>().unwrap_or_else(|_| {
                let span = self.span(start);
                let d = Diagnostic::error(codes::E2010, span, "integer literal is too large for any integer type");
                self.sink.emit(d);
                0
            });
            self.finish_number(start, Lit::Int { value, suffix: int_suffix(&suffix) }, &suffix, false);
        }
    }

    fn eat_decimal_run(&mut self) {
        while let Some(c) = self.peek() {
            if c == '_' || c.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn lex_number_suffix(&mut self) -> String {
        let start = self.pos;
        if self.peek().is_some_and(is_ident_start_or_underscore) {
            self.eat_ident_run();
        }
        self.src[start..self.pos].to_string()
    }

    fn finish_number(&mut self, start: usize, lit: Lit, suffix: &str, is_float: bool) {
        if !suffix.is_empty() && int_suffix(suffix).is_none() && float_suffix(suffix).is_none() {
            let span = self.span(start);
            let d = Diagnostic::error(codes::E0100, span, format!("`{suffix}` is not a numeric suffix"))
                .help("valid suffixes are i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 isize usize f16 f32 f64");
            self.sink.emit(d);
        } else if is_float && int_suffix(suffix).is_some() {
            let span = self.span(start);
            let d = Diagnostic::error(codes::E2010, span, format!("`{suffix}` is an integer suffix on a float literal"))
                .help("remove the fractional part, or use a float suffix");
            self.sink.emit(d);
        }
        self.push(TokenKind::Lit(lit), start);
    }

    // -- characters and lifetimes -------------------------------------------

    fn lex_char_or_lifetime(&mut self) {
        let start = self.pos;

        // `[LT-6]`: `'ident` is a named lifetime, reserved for v2.
        if self.peek_nth(1).is_some_and(is_ident_start_or_underscore) {
            let mut probe = self.pos + 1;
            while let Some(c) = self.src[probe..].chars().next() {
                if is_ident_continue(c) {
                    probe += c.len_utf8();
                } else {
                    break;
                }
            }
            if self.src[probe..].chars().next() != Some('\'') {
                self.pos = probe;
                let span = self.span(start);
                let d = Diagnostic::error(codes::E0007, span, "named lifetimes are not supported in this version")
                    .help("restructure using `@borrows` or a view struct");
                self.sink.emit(d);
                self.push(TokenKind::Error, start);
                return;
            }
        }

        self.pos += 1; // opening quote
        let value = match self.peek() {
            None | Some('\n') => {
                let span = self.span(start);
                let d = Diagnostic::error(codes::E0101, span, "unterminated character literal");
                self.sink.emit(d);
                self.push(TokenKind::Error, start);
                return;
            }
            Some('\\') => self.lex_escape(EscapeContext::Text),
            Some(c) => {
                self.pos += c.len_utf8();
                Some(c as u32)
            }
        };

        if !self.eat('\'') {
            // Consume to the closing quote or end of line so one bad literal
            // does not cascade.
            while !matches!(self.peek(), None | Some('\'') | Some('\n')) {
                self.bump();
            }
            let unterminated = self.peek() != Some('\'');
            self.eat('\'');
            let span = self.span(start);
            let message = if unterminated {
                "unterminated character literal"
            } else {
                "a character literal holds exactly one character"
            };
            let d = Diagnostic::error(codes::E0101, span, message);
            self.sink.emit(d);
            self.push(TokenKind::Error, start);
            return;
        }

        match value.and_then(char::from_u32) {
            Some(c) => self.push(TokenKind::Lit(Lit::Char(c)), start),
            None => {
                let span = self.span(start);
                let d = Diagnostic::error(codes::E0101, span, "not a Unicode scalar value");
                self.sink.emit(d);
                self.push(TokenKind::Error, start);
            }
        }
    }

    // -- strings ------------------------------------------------------------

    fn lex_string_like(&mut self, flavour: StringFlavour) {
        let start = self.pos;
        match flavour {
            StringFlavour::Raw => self.lex_raw_string(start),
            StringFlavour::Str => {
                if self.rest().starts_with("\"\"\"") {
                    self.lex_multiline_string(start)
                } else {
                    self.lex_quoted(start, flavour)
                }
            }
            _ => {
                self.pos += 1; // the b / c / f prefix
                self.lex_quoted(start, flavour)
            }
        }
    }

    fn lex_raw_string(&mut self, start: usize) {
        self.pos += 1; // 'r'
        let mut hashes = 0usize;
        while self.eat('#') {
            hashes += 1;
        }
        if hashes > 8 {
            let span = self.span(start);
            let d = Diagnostic::error(codes::E0101, span, "a raw string may have at most 8 hashes");
            self.sink.emit(d);
        }
        if !self.eat('"') {
            let span = self.span(start);
            let d = Diagnostic::error(codes::E0101, span, "expected `\"` after a raw string prefix");
            self.sink.emit(d);
            self.push(TokenKind::Error, start);
            return;
        }

        let body_start = self.pos;
        let terminator = format!("\"{}", "#".repeat(hashes));
        match self.rest().find(&terminator) {
            Some(offset) => {
                let body = self.src[body_start..body_start + offset].to_string();
                self.pos = body_start + offset + terminator.len();
                self.push(TokenKind::Lit(Lit::Str(body)), start);
            }
            None => {
                self.pos = self.src.len();
                let span = self.span(start);
                let d = Diagnostic::error(codes::E0101, span, "unterminated raw string");
                self.sink.emit(d);
                self.push(TokenKind::Error, start);
            }
        }
    }

    fn lex_multiline_string(&mut self, start: usize) {
        self.pos += 3;
        let body_start = self.pos;
        match self.rest().find("\"\"\"") {
            Some(offset) => {
                let body = &self.src[body_start..body_start + offset];
                self.pos = body_start + offset + 3;
                let text = strip_common_indent(body);
                // Escapes still apply inside a triple-quoted string.
                let span = self.span(start);
                let decoded = decode_escapes(&text, EscapeContext::Text, span, self.sink);
                self.push(TokenKind::Lit(Lit::Str(decoded)), start);
            }
            None => {
                self.pos = self.src.len();
                let span = self.span(start);
                let d = Diagnostic::error(codes::E0101, span, "unterminated string");
                self.sink.emit(d);
                self.push(TokenKind::Error, start);
            }
        }
    }

    fn lex_quoted(&mut self, start: usize, flavour: StringFlavour) {
        self.pos += 1; // opening quote
        let mut text = String::new();
        let mut bytes: Vec<u8> = Vec::new();
        let mut parts: Vec<FStrPart> = Vec::new();
        let context = match flavour {
            StringFlavour::Bytes | StringFlavour::CStr => EscapeContext::Bytes,
            _ => EscapeContext::Text,
        };

        loop {
            match self.peek() {
                None | Some('\n') => {
                    let span = self.span(start);
                    let d = Diagnostic::error(codes::E0101, span, "unterminated string");
                    self.sink.emit(d);
                    self.push(TokenKind::Error, start);
                    return;
                }
                Some('"') => {
                    self.pos += 1;
                    break;
                }
                Some('\\') => {
                    if let Some(value) = self.lex_escape(context) {
                        match flavour {
                            StringFlavour::Bytes | StringFlavour::CStr => bytes.push(value as u8),
                            _ => {
                                if let Some(c) = char::from_u32(value) {
                                    text.push(c);
                                }
                            }
                        }
                    }
                }
                Some('{') if flavour == StringFlavour::FStr => {
                    if self.peek_nth(1) == Some('{') {
                        self.pos += 2;
                        text.push('{');
                        continue;
                    }
                    if !text.is_empty() {
                        parts.push(FStrPart::Text(std::mem::take(&mut text)));
                    }
                    if let Some(part) = self.lex_fstring_hole() {
                        parts.push(part);
                    } else {
                        self.push(TokenKind::Error, start);
                        return;
                    }
                }
                Some('}') if flavour == StringFlavour::FStr => {
                    if self.peek_nth(1) == Some('}') {
                        self.pos += 2;
                        text.push('}');
                        continue;
                    }
                    let at = self.pos;
                    self.pos += 1;
                    let span = self.span(at);
                    let d = Diagnostic::error(codes::E0101, span, "unmatched `}` in an f-string")
                        .help("write `}}` for a literal brace");
                    self.sink.emit(d);
                }
                Some(c) => {
                    self.pos += c.len_utf8();
                    match flavour {
                        StringFlavour::Bytes | StringFlavour::CStr => {
                            if c.is_ascii() {
                                bytes.push(c as u8);
                            } else {
                                let span = self.span(start);
                                let d = Diagnostic::error(codes::E0101, span, "a byte string may hold ASCII only",
                                );
                                self.sink.emit(d);
                            }
                        }
                        _ => text.push(c),
                    }
                }
            }
        }

        let lit = match flavour {
            StringFlavour::Bytes => Lit::Bytes(bytes),
            StringFlavour::CStr => {
                if bytes.contains(&0) {
                    let span = self.span(start);
                    let d = Diagnostic::error(codes::E0101, span, "a C string literal may not contain a NUL byte")
                        .note("the terminating NUL is added by the compiler");
                    self.sink.emit(d);
                }
                Lit::CStr(bytes)
            }
            StringFlavour::FStr => {
                if !text.is_empty() {
                    parts.push(FStrPart::Text(text));
                }
                Lit::FStr(parts)
            }
            _ => Lit::Str(text),
        };
        self.push(TokenKind::Lit(lit), start);
    }

    /// The `{expr:spec}` of an f-string. `pos` is on the `{`.
    ///
    /// `[LEX-19]` — the expression is a full expression, so the hole is found
    /// by bracket matching rather than by scanning for the next `}`. The
    /// format spec starts at the first `:` at bracket depth zero that is not
    /// part of a `::` path separator.
    fn lex_fstring_hole(&mut self) -> Option<FStrPart> {
        let brace = self.pos;
        self.pos += 1;
        let expr_start = self.pos;
        let mut depth = 0i32;
        let mut spec_at: Option<usize> = None;

        loop {
            match self.peek() {
                None | Some('\n') => {
                    let span = Span::new(self.file, brace as u32, self.pos as u32);
                    let d = Diagnostic::error(codes::E0101, span, "unterminated `{` in an f-string");
                    self.sink.emit(d);
                    return None;
                }
                Some('"') => {
                    // Skip a nested string literal whole, so its braces and
                    // colons do not confuse the scan.
                    self.pos += 1;
                    while let Some(c) = self.peek() {
                        self.pos += c.len_utf8();
                        if c == '\\' {
                            if let Some(next) = self.peek() {
                                self.pos += next.len_utf8();
                            }
                        } else if c == '"' {
                            break;
                        }
                    }
                }
                Some(c @ ('(' | '[' | '{')) => {
                    depth += 1;
                    self.pos += c.len_utf8();
                }
                Some('}') if depth == 0 => break,
                Some(c @ (')' | ']' | '}')) => {
                    depth -= 1;
                    self.pos += c.len_utf8();
                }
                Some(':') if depth == 0 && spec_at.is_none() => {
                    if self.peek_nth(1) == Some(':') {
                        self.pos += 2; // a path separator, not a format spec
                    } else {
                        spec_at = Some(self.pos);
                        self.pos += 1;
                    }
                }
                Some(c) => self.pos += c.len_utf8(),
            }
        }

        let hole_end = self.pos;
        self.pos += 1; // '}'

        let expr_end = spec_at.unwrap_or(hole_end);
        let format_spec = spec_at.map(|at| self.src[at + 1..hole_end].to_string());
        Some(FStrPart::Expr {
            span: Span::new(self.file, expr_start as u32, expr_end as u32),
            format_spec,
        })
    }

    /// Consume one escape sequence, returning its scalar value.
    fn lex_escape(&mut self, context: EscapeContext) -> Option<u32> {
        let start = self.pos;
        match scan_escape(self.src, &mut self.pos, context) {
            Ok(value) => Some(value),
            Err(err) => {
                let span = self.span(start);
                self.sink.emit(err.into_diagnostic(span));
                None
            }
        }
    }

    // -- punctuation --------------------------------------------------------

    fn lex_punct(&mut self) {
        let start = self.pos;
        let rest = self.rest();
        for &(text, punct) in &self.puncts {
            if rest.starts_with(text) {
                self.pos += text.len();
                if punct.is_open_bracket() {
                    self.bracket_depth += 1;
                } else if punct.is_close_bracket() {
                    self.bracket_depth = self.bracket_depth.saturating_sub(1);
                }
                self.push(TokenKind::Punct(punct), start);
                return;
            }
        }

        let c = self.bump().expect("lex_punct called at end of file");
        let span = self.span(start);
        let d = Diagnostic::error(codes::E0100, span, format!("unexpected character `{c}`"))
            .primary_label("this character has no meaning in Ember");
        self.sink.emit(d);
        self.push(TokenKind::Error, start);
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum StringFlavour {
    Str,
    Raw,
    Bytes,
    CStr,
    FStr,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum EscapeContext {
    /// A `str` or `char`: `\x` is limited to 0x7F.
    Text,
    /// A byte or C string: any `\x` value.
    Bytes,
}

fn int_suffix(text: &str) -> Option<IntSuffix> {
    IntSuffix::from_str(text)
}

fn float_suffix(text: &str) -> Option<token::FloatSuffix> {
    token::FloatSuffix::from_str(text)
}

fn is_ident_start(c: char) -> bool {
    c == '_' || unicode_ident::is_xid_start(c)
}

fn is_ident_start_or_underscore(c: char) -> bool {
    is_ident_start(c)
}

fn is_ident_continue(c: char) -> bool {
    c == '_' || unicode_ident::is_xid_continue(c)
}

/// `[LEX-12]` — identifiers are NFC-normalised, and two identifiers are the
/// same iff their NFC forms are byte-equal. ASCII is already NFC, so the
/// common case costs one scan.
fn normalise_ident(text: &str) -> String {
    if text.is_ascii() { text.to_string() } else { text.nfc().collect() }
}

/// Strip the common leading indentation of a triple-quoted string, and drop a
/// leading newline and a trailing whitespace-only line, so that
///
/// ```text
///     """
///     one
///       two
///     """
/// ```
///
/// yields `"one\n  two\n"`.
fn strip_common_indent(body: &str) -> String {
    let body = body.strip_prefix('\n').unwrap_or(body);
    let lines: Vec<&str> = body.split('\n').collect();

    // The closing delimiter's own line, if it holds nothing but spaces, sets
    // the indentation and is not content.
    let (content, closing_indent) = match lines.split_last() {
        Some((last, rest)) if last.trim().is_empty() && !rest.is_empty() => (rest, Some(last.len())),
        _ => (&lines[..], None),
    };

    let common = content
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start_matches(' ').len())
        .chain(closing_indent)
        .min()
        .unwrap_or(0);

    let mut out = String::new();
    for line in content {
        if line.len() >= common {
            out.push_str(&line[common..]);
        } else {
            out.push_str(line.trim_start_matches(' '));
        }
        out.push('\n');
    }
    out
}

// ---------------------------------------------------------------------------
// Escape sequences
// ---------------------------------------------------------------------------

/// What went wrong in an escape sequence. Kept separate from the diagnostic so
/// that both the in-place lexer and the de-indented triple-quoted path can
/// report against whichever span makes sense for them.
#[derive(Debug)]
enum EscapeError {
    Truncated,
    HexDigits,
    HexAboveAscii,
    MissingBrace,
    UnicodeDigits,
    NotScalar(u32),
    Unknown(char),
}

const VALID_ESCAPES: &str = r#"valid escapes are \n \r \t \0 \\ \" \' \xHH \u{…}"#;

impl EscapeError {
    fn into_diagnostic(self, span: Span) -> Diagnostic {
        match self {
            EscapeError::Truncated => {
                Diagnostic::error(codes::E0101, span, "escape sequence is cut off")
            }
            EscapeError::HexDigits => {
                Diagnostic::error(codes::E0101, span, r"`\x` needs exactly two hex digits")
            }
            EscapeError::HexAboveAscii => {
                Diagnostic::error(codes::E0101, span, r"`\x` above 0x7F is not valid in a string")
                    .help(r"use `\u{…}` for a non-ASCII character")
            }
            EscapeError::MissingBrace => {
                Diagnostic::error(codes::E0101, span, r"`\u` must be followed by `{`")
            }
            EscapeError::UnicodeDigits => {
                Diagnostic::error(codes::E0101, span, r"`\u{…}` takes 1 to 6 hex digits")
            }
            EscapeError::NotScalar(v) => Diagnostic::error(
                codes::E0101,
                span,
                format!("{v:#x} is not a Unicode scalar value"),
            )
            .note("surrogates D800-DFFF are not scalar values [TYP-3]"),
            EscapeError::Unknown(c) => {
                Diagnostic::error(codes::E0101, span, format!(r"unknown escape `\{c}`"))
                    .help(VALID_ESCAPES)
            }
        }
    }
}

/// Scan one escape sequence starting at `*pos` (which is on the backslash),
/// advancing `*pos` past it.
fn scan_escape(src: &str, pos: &mut usize, context: EscapeContext) -> Result<u32, EscapeError> {
    let mut chars = src[*pos..].chars();
    chars.next(); // the backslash
    let c = chars.next().ok_or(EscapeError::Truncated)?;
    *pos += 1 + c.len_utf8();

    let simple = match c {
        'n' => Some('\n' as u32),
        'r' => Some('\r' as u32),
        't' => Some('\t' as u32),
        '0' => Some(0),
        '\\' => Some('\\' as u32),
        '"' => Some('"' as u32),
        '\'' => Some('\'' as u32),
        _ => None,
    };
    if let Some(value) = simple {
        return Ok(value);
    }

    match c {
        'x' => {
            let digits: String =
                src[*pos..].chars().take(2).take_while(|c| c.is_ascii_hexdigit()).collect();
            if digits.len() != 2 {
                return Err(EscapeError::HexDigits);
            }
            *pos += 2;
            let value = u32::from_str_radix(&digits, 16).expect("two hex digits");
            if context == EscapeContext::Text && value > 0x7F {
                return Err(EscapeError::HexAboveAscii);
            }
            Ok(value)
        }
        'u' => {
            if !src[*pos..].starts_with('{') {
                return Err(EscapeError::MissingBrace);
            }
            *pos += 1;
            let digits: String = src[*pos..].chars().take_while(|c| c.is_ascii_hexdigit()).collect();
            if !(1..=6).contains(&digits.len()) {
                return Err(EscapeError::UnicodeDigits);
            }
            *pos += digits.len();
            if !src[*pos..].starts_with('}') {
                return Err(EscapeError::UnicodeDigits);
            }
            *pos += 1;
            let value = u32::from_str_radix(&digits, 16).map_err(|_| EscapeError::UnicodeDigits)?;
            if char::from_u32(value).is_none() {
                return Err(EscapeError::NotScalar(value));
            }
            Ok(value)
        }
        other => Err(EscapeError::Unknown(other)),
    }
}

/// Resolve escapes in text that has already been extracted from the source (a
/// triple-quoted body, after de-indenting). Errors report against `span`, the
/// literal as a whole, because the de-indented offsets no longer line up with
/// the file.
fn decode_escapes(text: &str, context: EscapeContext, span: Span, sink: &mut Sink) -> String {
    if !text.contains('\\') {
        return text.to_string();
    }
    let mut out = String::new();
    let mut pos = 0usize;
    while let Some(c) = text[pos..].chars().next() {
        if c == '\\' {
            match scan_escape(text, &mut pos, context) {
                Ok(value) => {
                    if let Some(ch) = char::from_u32(value) {
                        out.push(ch);
                    }
                }
                Err(err) => {
                    sink.emit(err.into_diagnostic(span));
                    pos += 1;
                }
            }
        } else {
            pos += c.len_utf8();
            out.push(c);
        }
    }
    out
}
