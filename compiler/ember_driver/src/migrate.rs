//! `[CLI-20]` — the 0.9.10 spellings M1 replaces, rewritten from a file's tokens and tree: the words
//! on parameters and before values (`owned` to `owns`, `mut x` to `ref mut x`, `x: ref mut T` to
//! `ref mut x: T`; `x: ref T` to `x: T` is M4's, `M1-plan.md`), `str` and `cstr` to `StrView` and `CStrView`, an `if Some(p) = e:` header to
//! `if e exists as p:`, `@borrows(p)` to `-> T from p`, the renamed attributes, `.copied()` and
//! `.cloned()` to `.copy_each()`, and type parameters on a line of their own with `needs` before
//! their bounds (`[GRM-40]`). The repository's own migration makes these alone (`migrate_file`); a
//! program's makes them in its first round, before the checker's.
//!
//! `x.clone()` is left: `copy of` a value holding a class object is not built until M4, which
//! replaces it whole (`M1-plan.md`).

use ember_ast as ast;
use ember_lexer::{Kw, Punct, Token, TokenKind};
use ember_span::{FileId, Span};

/// One file's rewrites: the edits; what each changed, by the place it is in the text read; and what
/// needs a choice the source does not say.
#[derive(Default)]
pub(crate) struct Rewrites {
    pub(crate) edits: Vec<(Span, String)>,
    pub(crate) changed: Vec<(Span, String)>,
    pub(crate) left: Vec<(Span, String)>,
}

impl Rewrites {
    fn rewrite(&mut self, span: Span, with: impl Into<String>, why: impl Into<String>) {
        self.edits.push((span, with.into()));
        self.changed.push((span, why.into()));
    }
}

pub(crate) fn spellings(module: &ast::Module, text: &str, file: FileId, tokens: &[Token]) -> Rewrites {
    let words = Words { tokens, text, file };
    let mut out = Rewrites::default();
    borrows_to_from(module, &mut out);
    this_for_self(&words, &mut out);
    owned_to_owns(&words, &mut out);
    // A foreign declaration's `x: ref mut T` is the pointer C takes, a type and not a mode (`[FFI-11]`).
    let foreign: Vec<Span> = module.items.iter().filter(|item| matches!(item.kind, ast::ItemKind::ExternBlock(_))).map(|item| item.span).collect();
    parameter_words(&words, &foreign, &mut out);
    view_type_names(module, &words, &mut out);
    option_headers(&words, &mut out);
    attribute_names(&words, &mut out);
    copy_each(&words, &mut out);
    language_directive(&words, &mut out);
    // A moved line is made from the text of its brackets, so it is moved once the rewrites inside
    // them are made: in the next round, when this one made any (each rewrite finds only old forms).
    if out.edits.is_empty() {
        type_parameters(&words, &mut out);
    }
    out
}

struct Words<'a> {
    tokens: &'a [Token],
    text: &'a str,
    file: FileId,
}

impl Words<'_> {
    fn kind(&self, at: usize) -> Option<&TokenKind> {
        self.tokens.get(at).map(|token| &token.kind)
    }

    fn kw(&self, at: usize, kw: Kw) -> bool {
        matches!(self.kind(at), Some(TokenKind::Keyword(k)) if *k == kw)
    }

    fn punct(&self, at: usize, punct: Punct) -> bool {
        matches!(self.kind(at), Some(TokenKind::Punct(p)) if *p == punct)
    }

    fn ident(&self, at: usize, name: &str) -> bool {
        matches!(self.kind(at), Some(TokenKind::Ident(s)) if s.is(name))
    }

    fn is_name(&self, at: usize) -> bool {
        matches!(self.kind(at), Some(TokenKind::Ident(_) | TokenKind::RawIdent(_)))
    }

    /// A function's name after `fn`: a name, or `is` and `not`, which a method may be named
    /// (`[LEX-15]`).
    fn is_function_name(&self, at: usize) -> bool {
        self.is_name(at) || self.kw(at, Kw::Is) || self.kw(at, Kw::Not)
    }

    fn span(&self, at: usize) -> Span {
        self.tokens[at].span
    }

    /// The text from the start of token `first` to the end of token `last`.
    fn between(&self, first: usize, last: usize) -> &str {
        &self.text[self.span(first).start as usize..self.span(last).end as usize]
    }

    /// The token closing the bracket opened at `open`.
    fn closing(&self, open: usize) -> Option<usize> {
        let mut depth = 0usize;
        for at in open..self.tokens.len() {
            match self.kind(at) {
                Some(TokenKind::Punct(Punct::LParen | Punct::LBracket | Punct::LBrace)) => depth += 1,
                Some(TokenKind::Punct(Punct::RParen | Punct::RBracket | Punct::RBrace)) => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(at);
                    }
                }
                Some(TokenKind::Eof) | None => return None,
                _ => {}
            }
        }
        None
    }

    /// The tokens from `first` up to (not including) the first at bracket depth zero that `stop`
    /// accepts, or the end of the logical line.
    fn until(&self, first: usize, stop: impl Fn(&Self, usize) -> bool) -> usize {
        let mut depth = 0usize;
        let mut at = first;
        while at < self.tokens.len() {
            match self.kind(at) {
                Some(TokenKind::Punct(Punct::LParen | Punct::LBracket | Punct::LBrace)) => depth += 1,
                Some(TokenKind::Punct(Punct::RParen | Punct::RBracket | Punct::RBrace)) if depth > 0 => depth -= 1,
                Some(TokenKind::Newline | TokenKind::Eof) if depth == 0 => return at,
                _ if depth == 0 && stop(self, at) => return at,
                _ => {}
            }
            at += 1;
        }
        at
    }

    fn at(&self, offset: u32) -> Span {
        Span::new(self.file, offset, offset)
    }
}

/// `[FN-4]` — `self` is `this` where `object_to_this` does not reach: a parameter's default
/// (`delta: int = this.total`), and a body whose method lists no object any more. A listed object
/// (`self`, `mut self`, `self: Box[Self]`) is `object_to_this`'s, and an attribute's `self` goes
/// with the attribute (`borrows_to_from`).
fn this_for_self(words: &Words, out: &mut Rewrites) {
    let mut kept = vec![false; words.tokens.len()];
    for at in 0..words.tokens.len() {
        // An attribute's arguments.
        if words.punct(at, Punct::At) && words.is_name(at + 1) && words.punct(at + 2, Punct::LParen) {
            if let Some(close) = words.closing(at + 2) {
                kept[at..=close].iter_mut().for_each(|k| *k = true);
            }
        }
        // A listed object: the first parameter, after its words.
        if words.kw(at, Kw::Fn) && words.is_function_name(at + 1) {
            let mut open = at + 2;
            if words.punct(open, Punct::LBracket) {
                open = words.closing(open).map_or(open, |close| close + 1);
            }
            if words.punct(open, Punct::LParen) {
                let mut first = open + 1;
                while words.kw(first, Kw::Mut) || words.kw(first, Kw::Owned) || words.kw(first, Kw::Owns) || words.kw(first, Kw::Ref) || words.kw(first, Kw::Const) {
                    first += 1;
                }
                if words.kw(first, Kw::SelfValue) {
                    kept[first] = true;
                }
            }
        }
    }
    for (at, token) in words.tokens.iter().enumerate() {
        if matches!(token.kind, TokenKind::Keyword(Kw::SelfValue)) && !kept[at] {
            out.rewrite(token.span, "this", "inside a method its object is `this`");
        }
    }
}

/// `[OWN-9]` — the mode `owned` is `owns`, on a parameter and before a `for` iterable or a `match`
/// scrutinee, and in a C contract (verdict 102). `owned fn` is left (its capture words are the CLO
/// area's), and `owned self`, which `object_to_this` takes out.
fn owned_to_owns(words: &Words, out: &mut Rewrites) {
    for (at, token) in words.tokens.iter().enumerate() {
        if matches!(token.kind, TokenKind::Keyword(Kw::Owned)) && !words.kw(at + 1, Kw::Fn) && !words.kw(at + 1, Kw::SelfValue) {
            out.rewrite(token.span, "owns", "`owned` is `owns`");
        }
    }
}

/// `[FN-1]`, `[GRM-41]` — in each parameter list (a function's, a lambda's, a callable type's): `mut`
/// alone is `ref mut`, and a mode written in the type (`c: ref mut Counter`) goes on the name
/// (`ref mut c: Counter`). `x: ref T` is left: as `x: T` it would be passed as a copy where `T` is a
/// `Copy` type, a class handle among them until M4, and a view into it would end with the call.
fn parameter_words(words: &Words, foreign: &[Span], out: &mut Rewrites) {
    for at in 0..words.tokens.len() {
        if !words.kw(at, Kw::Fn) {
            continue;
        }
        let in_foreign = foreign.iter().any(|span| span.start <= words.span(at).start && words.span(at).end <= span.end);
        // `fn(`, `fn name(`, `fn name[T](`.
        let mut open = at + 1;
        if words.is_function_name(open) {
            open += 1;
        }
        if words.punct(open, Punct::LBracket) {
            let Some(close) = words.closing(open) else { continue };
            open = close + 1;
        }
        if !words.punct(open, Punct::LParen) {
            continue;
        }
        let Some(close) = words.closing(open) else { continue };
        let mut start = open + 1;
        while start < close {
            let end = words.until(start, |w, i| w.punct(i, Punct::Comma) || i >= close);
            parameter(words, start, end.min(close), in_foreign, out);
            start = end + 1;
        }
    }
}

fn parameter(words: &Words, first: usize, end: usize, in_foreign: bool, out: &mut Rewrites) {
    if first >= end {
        return;
    }
    if words.kw(first, Kw::Mut) && !words.kw(first + 1, Kw::SelfValue) {
        let at = words.span(first).start;
        out.rewrite(words.at(at), "ref ", "`mut` on a parameter is `ref mut`");
        return;
    }
    // `x: ref mut T`, with no word before the name.
    if !in_foreign
        && words.is_name(first) && words.punct(first + 1, Punct::Colon) && words.kw(first + 2, Kw::Ref) && words.kw(first + 3, Kw::Mut) {
        let ty = first + 4;
        if ty >= end {
            return;
        }
        let word = Span::new(words.file, words.span(first + 2).start, words.span(ty).start);
        out.rewrite(word, "", "the mode goes on the name: `ref mut x: T`");
        out.edits.push((words.at(words.span(first).start), "ref mut ".to_string()));
    }
}

/// `[TXT-1]`, `[TXT-3]` — `str` and `cstr` name `StrView` and `CStrView`. A file that declares a
/// type of its own named `str` keeps its name: what to call it is the file's choice.
fn view_type_names(module: &ast::Module, words: &Words, out: &mut Rewrites) {
    let own = module.items.iter().find_map(|item| {
        let name = match &item.kind {
            ast::ItemKind::Struct(decl) => decl.name,
            ast::ItemKind::Class(decl) => decl.name,
            ast::ItemKind::Enum(decl) => decl.name,
            ast::ItemKind::TypeAlias(decl) => decl.name,
            _ => return None,
        };
        (name.name.is("str") || name.name.is("cstr")).then_some(name)
    });
    if let Some(name) = own {
        out.left.push((name.span, format!("this file declares its own `{}`: 0.9.15's prelude names the view `StrView`; rename it, or shadow `StrView`", name.name)));
        return;
    }
    for (at, token) in words.tokens.iter().enumerate() {
        let new = if words.ident(at, "str") {
            "StrView"
        } else if words.ident(at, "cstr") {
            "CStrView"
        } else {
            continue;
        };
        // A member (`x.str`) is no type, and a call (`str(x)`, Python's habit) is what it is.
        if (at > 0 && words.punct(at - 1, Punct::Dot)) || words.punct(at + 1, Punct::LParen) {
            continue;
        }
        out.rewrite(token.span, new, format!("`{}` is `{new}`", words.between(at, at)));
    }
}

/// `[GRM-19]`, `[GRM-44]` — `if Some(p) = e:`, `elif` and `while` alike, is `if e exists as p:`, the
/// names keeping their words (`mut v`, a copy of one's own, is `copy of v`). A pattern inside the
/// `Some` that tests more than whether `e` holds a value (`Some(Mode.Fast)`, `Some(0)`) is left.
fn option_headers(words: &Words, out: &mut Rewrites) {
    for at in 0..words.tokens.len() {
        if !(words.kw(at, Kw::If) || words.kw(at, Kw::Elif) || words.kw(at, Kw::While)) {
            continue;
        }
        if !(words.ident(at + 1, "Some") && words.punct(at + 2, Punct::LParen)) {
            continue;
        }
        let Some(close) = words.closing(at + 2) else { continue };
        if !words.punct(close + 1, Punct::Eq) || close == at + 3 {
            continue;
        }
        let value_end = words.until(close + 2, |w, i| w.punct(i, Punct::Colon));
        if value_end <= close + 2 || !words.punct(value_end, Punct::Colon) {
            continue;
        }
        let Some(names) = opened_names(words, at + 3, close) else {
            let span = Span::new(words.file, words.span(at + 1).start, words.span(close).end);
            out.left.push((span, "this header tests a value inside the `Option`, not only that it holds one: write it as a `match` [GRM-19]".to_string()));
            continue;
        };
        let value = words.between(close + 2, value_end - 1);
        let span = Span::new(words.file, words.span(at + 1).start, words.span(value_end - 1).end);
        out.rewrite(span, format!("{value} exists as {names}"), "an `Option` is opened with `exists as`");
    }
}

/// The names of a `Some(...)` pattern, as `exists as` writes them, if it holds only names: a name
/// (`p`, `ref p`, `ref mut p`, `mut p` as `copy of p`), `_`, or a tuple of them.
fn opened_names(words: &Words, first: usize, close: usize) -> Option<String> {
    let mut written = String::new();
    let mut at = first;
    while at < close {
        let token = &words.tokens[at];
        match &token.kind {
            TokenKind::Keyword(Kw::Mut) if !(at > first && words.kw(at - 1, Kw::Ref)) => written.push_str("copy of "),
            TokenKind::Keyword(Kw::Mut) => written.push_str("mut "),
            TokenKind::Keyword(Kw::Ref) => written.push_str("ref "),
            // A capital name may be a variant or a constant, which a pattern tests.
            TokenKind::Ident(name) if !name.as_str().starts_with(|c: char| c.is_uppercase()) => written.push_str(words.between(at, at)),
            TokenKind::RawIdent(_) => written.push_str(words.between(at, at)),
            TokenKind::Punct(Punct::LParen) => written.push('('),
            TokenKind::Punct(Punct::RParen) => written.push(')'),
            TokenKind::Punct(Punct::Comma) => written.push_str(", "),
            _ => return None,
        }
        at += 1;
    }
    Some(written)
}

/// `[ATT-1]` — `@static_safe` is `@no_alias_checks`, `@fp(contract)` `@fp(allow_fma)`.
fn attribute_names(words: &Words, out: &mut Rewrites) {
    for at in 1..words.tokens.len() {
        if !words.punct(at - 1, Punct::At) {
            continue;
        }
        if words.ident(at, "static_safe") {
            out.rewrite(words.span(at), "no_alias_checks", "`@static_safe` is `@no_alias_checks`");
        } else if words.ident(at, "fp") && words.punct(at + 1, Punct::LParen) && words.ident(at + 2, "contract") {
            out.rewrite(words.span(at + 2), "allow_fma", "`@fp(contract)` is `@fp(allow_fma)`");
        }
    }
}

/// `[VER-8]` — a file naming 0.9.10, the language it is migrated from, names the current one.
fn language_directive(words: &Words, out: &mut Rewrites) {
    for (at, token) in words.tokens.iter().enumerate() {
        if let TokenKind::Directive { name, value } = &token.kind
            && name.is("language")
            && value == "0.9.10"
        {
            let current = ember_parser::LANGUAGE_VERSION;
            out.rewrite(words.span(at), format!("#! language \"{current}\""), format!("the language is {current}"));
        }
    }
}

/// `[STD-19]` — `.copied()` and `.cloned()` are `.copy_each()`.
fn copy_each(words: &Words, out: &mut Rewrites) {
    for at in 1..words.tokens.len() {
        if words.punct(at - 1, Punct::Dot)
            && (words.ident(at, "copied") || words.ident(at, "cloned"))
            && words.punct(at + 1, Punct::LParen)
            && words.punct(at + 2, Punct::RParen)
        {
            let old = words.between(at, at).to_string();
            out.rewrite(words.span(at), "copy_each", format!("`.{old}()` is `.copy_each()`"));
        }
    }
}

/// `[LT-1a]`, `[GRM-41]`, `[LT-22]` — `@borrows(p)` is `-> T from p`, `@borrows(self.f)` `-> T from f`;
/// `@borrows(self)` goes, as rule 1 ties every method's result to `this`, and one naming several sources
/// goes, as callers use the sources the body gives (`[LT-22]`).
fn borrows_to_from(module: &ast::Module, out: &mut Rewrites) {
    let mut functions: Vec<(&[ast::Attribute], &ast::FnDecl)> = Vec::new();
    for item in &module.items {
        let members: &[ast::Member] = match &item.kind {
            ast::ItemKind::Fn(decl) => {
                functions.push((&item.attrs, decl));
                continue;
            }
            ast::ItemKind::Struct(decl) => &decl.members,
            ast::ItemKind::Class(decl) => &decl.members,
            ast::ItemKind::Enum(decl) => &decl.members,
            ast::ItemKind::Interface(decl) => &decl.members,
            ast::ItemKind::Extend(decl) => &decl.members,
            _ => continue,
        };
        for member in members {
            if let ast::MemberKind::Fn(decl) = &member.kind {
                functions.push((&member.attrs, decl));
            }
        }
    }
    for (attrs, decl) in functions {
        let Some(attr) = attrs.iter().find(|attr| attr.path.len() == 1 && attr.path[0].name.is("borrows")) else { continue };
        let named = match attr.args.as_slice() {
            [ast::AttrArg::Expr(expr)] => match &expr.kind {
                ast::ExprKind::Path { segments } if segments.len() == 1 => Some(Some(segments[0].name)),
                ast::ExprKind::Field { base, name } if matches!(base.kind, ast::ExprKind::SelfExpr) => Some(Some(name.name)),
                ast::ExprKind::SelfExpr => Some(None),
                _ => None,
            },
            _ => None,
        };
        match (named, decl.ret.as_ref()) {
            (Some(named), Some(ret)) => {
                out.rewrite(attr.span, "", "`@borrows` is `from` after the result type");
                if let Some(name) = named {
                    out.edits.push((Span::new(ret.span.file, ret.span.end, ret.span.end), format!(" from {name}")));
                }
            }
            _ if attr.args.len() > 1 => out.rewrite(attr.span, "", "a result points into the sources its body gives it (`[LT-22]`): `@borrows` goes"),
            _ => out.left.push((attr.span, "`from` names one parameter or field the result points into: say which, after the result type".to_string())),
        }
    }
}

/// `[GRM-40]`, `[TYP-17]` — a function's, struct's or class's type parameters go on a line of their
/// own above it, at its indentation; every bound is written after `needs` (`T: needs (A, B)`), in
/// those lines, in the brackets an enum, interface, alias or `extend` keeps, in a `where` clause and
/// in an interface's first line.
fn type_parameters(words: &Words, out: &mut Rewrites) {
    let mut at = 0;
    while at < words.tokens.len() {
        let declares = words.kw(at, Kw::Fn) || words.kw(at, Kw::Struct) || words.kw(at, Kw::Class);
        let keeps = words.kw(at, Kw::Enum) || words.kw(at, Kw::Interface) || words.kw(at, Kw::Type);
        if (declares || keeps) && words.is_function_name(at + 1) && words.punct(at + 2, Punct::LBracket) {
            let open = at + 2;
            let Some(close) = words.closing(open) else { break };
            let list = parameter_list(words, open + 1, close);
            if declares {
                let line_start = words.text[..words.span(at).start as usize].rfind('\n').map_or(0, |n| n + 1);
                let indent: String = words.text[line_start..].chars().take_while(|c| *c == ' ' || *c == '\t').collect();
                // The name and its brackets become the name: an empty edit would take the space after.
                let name = Span::new(words.file, words.span(at + 1).start, words.span(close).end);
                out.edits.push((name, words.between(at + 1, at + 1).to_string()));
                out.rewrite(words.at(line_start as u32), format!("{indent}[{list}]\n"), "type parameters go on a line of their own above the declaration");
            } else if list != words.between(open + 1, close - 1) {
                out.rewrite(Span::new(words.file, words.span(open + 1).start, words.span(close - 1).end), list, "a bound is written after `needs`");
            }
            if words.kw(at, Kw::Interface) {
                interface_needs(words, close + 1, out);
            }
            at = close;
        } else if words.ident(at, "extend") && words.punct(at + 1, Punct::LBracket) {
            let Some(close) = words.closing(at + 1) else { break };
            if close > at + 2 {
                let list = parameter_list(words, at + 2, close);
                if list != words.between(at + 2, close - 1) {
                    out.rewrite(Span::new(words.file, words.span(at + 2).start, words.span(close - 1).end), list, "a bound is written after `needs`");
                }
            }
            at = close;
        } else if words.kw(at, Kw::Type) && words.is_name(at + 1) && words.punct(at + 2, Punct::Colon) && !words.ident(at + 3, "needs") {
            // An interface's associated type: `type Count: ItemCount = int`.
            let end = words.until(at + 3, |w, i| w.punct(i, Punct::Eq));
            if end > at + 3 {
                let bounds = needs(words, at + 3, end);
                out.rewrite(Span::new(words.file, words.span(at + 3).start, words.span(end - 1).end), bounds, "a bound is written after `needs`");
            }
            at = end;
        } else if words.kw(at, Kw::Where) {
            at = where_clause(words, at + 1, out);
        } else if words.kw(at, Kw::Interface) && words.is_name(at + 1) {
            interface_needs(words, at + 2, out);
            at += 1;
        } else {
            at += 1;
        }
    }
}

/// `interface Named: Display:` — the interfaces it needs, from the colon at `colon` to the colon
/// ending the line, written after `needs`.
fn interface_needs(words: &Words, colon: usize, out: &mut Rewrites) {
    if !words.punct(colon, Punct::Colon) || matches!(words.kind(colon + 1), Some(TokenKind::Newline)) || words.ident(colon + 1, "needs") {
        return;
    }
    let end = words.until(colon + 1, |w, i| w.punct(i, Punct::Colon));
    if end > colon + 1 {
        let bounds = needs(words, colon + 1, end);
        out.rewrite(Span::new(words.file, words.span(colon + 1).start, words.span(end - 1).end), bounds, "what an interface needs is written after `needs`");
    }
}

/// A list of type parameters (`T: A + B, U, const N: usize`), each bound written after `needs`.
fn parameter_list(words: &Words, first: usize, close: usize) -> String {
    let mut parts = Vec::new();
    let mut start = first;
    while start < close {
        let end = words.until(start, |w, i| w.punct(i, Punct::Comma) || i >= close).min(close);
        if end > start {
            parts.push(type_parameter(words, start, end));
        }
        start = end + 1;
    }
    parts.join(", ")
}

/// One type parameter: `T: A + B = D` as `T: needs (A, B) = D`; a `const` one as written.
fn type_parameter(words: &Words, first: usize, end: usize) -> String {
    let written = words.between(first, end - 1).to_string();
    if words.kw(first, Kw::Const) || !words.punct(first + 1, Punct::Colon) || words.ident(first + 2, "needs") {
        return written;
    }
    let bounds_end = words.until(first + 2, |w, i| w.punct(i, Punct::Eq) || i >= end).min(end);
    if bounds_end <= first + 2 {
        return written;
    }
    let name = words.between(first, first);
    let rest = if bounds_end < end { format!(" {}", words.between(bounds_end, end - 1)) } else { String::new() };
    format!("{name}: {}{rest}", needs(words, first + 2, bounds_end))
}

/// `A + B` as `needs (A, B)`, `A` as `needs A`.
fn needs(words: &Words, first: usize, end: usize) -> String {
    let mut bounds = Vec::new();
    let mut start = first;
    while start < end {
        let stop = words.until(start, |w, i| w.punct(i, Punct::Plus) || i >= end).min(end);
        if stop > start {
            bounds.push(words.between(start, stop - 1).to_string());
        }
        start = stop + 1;
    }
    match bounds.as_slice() {
        // A tuple is one bound, not a list of them: `needs ((int, int))`.
        [one] if one.starts_with('(') => format!("needs ({one})"),
        [one] => format!("needs {one}"),
        _ => format!("needs ({})", bounds.join(", ")),
    }
}

/// `where T: A + B, U: C` with each bound after `needs`, up to the colon ending the line; the token
/// after it.
fn where_clause(words: &Words, first: usize, out: &mut Rewrites) -> usize {
    let mut start = first;
    loop {
        // The subject, up to its colon.
        let colon = words.until(start, |w, i| w.punct(i, Punct::Colon));
        if !words.punct(colon, Punct::Colon) {
            return colon;
        }
        // The colon ending the line: the last of the clause.
        if matches!(words.kind(colon + 1), Some(TokenKind::Newline | TokenKind::Indent | TokenKind::Eof)) {
            return colon + 1;
        }
        let end = words.until(colon + 1, |w, i| w.punct(i, Punct::Comma) || w.punct(i, Punct::Colon));
        if end > colon + 1 && !words.ident(colon + 1, "needs") {
            let bounds = needs(words, colon + 1, end);
            out.rewrite(Span::new(words.file, words.span(colon + 1).start, words.span(end - 1).end), bounds, "a bound is written after `needs`");
        }
        if !words.punct(end, Punct::Comma) {
            return end;
        }
        start = end + 1;
    }
}

#[cfg(test)]
mod tests {
    /// Each rewrite on a 0.9.10 file, applied round by round as `migrate_file` applies them; what
    /// the first round left.
    fn migrated(source: &str) -> (String, Vec<String>) {
        let mut text = source.to_string();
        let mut left = None;
        for _ in 0..8 {
            let mut map = ember_span::SourceMap::new();
            let file = map.add(ember_branding::source_file("t"), text.clone());
            let mut sink = ember_diag::Sink::new();
            let lexed = ember_lexer::lex(file, &text, &mut sink);
            let tokens = lexed.tokens.clone();
            let module = ember_parser::parse(file, &text, lexed.tokens, &mut sink);
            let rewrites = super::spellings(&module, &text, file, &tokens);
            left.get_or_insert_with(|| rewrites.left.iter().map(|(_, why)| why.clone()).collect());
            if rewrites.edits.is_empty() {
                break;
            }
            text = crate::apply_migration_edits(&text, rewrites.edits).0;
        }
        (text, left.unwrap_or_default())
    }

    #[test]
    fn a_file_naming_the_old_language_names_the_current_one() {
        let (text, _) = migrated("#! language \"0.9.10\"\n\nfn main():\n    pass\n");
        assert_eq!(text, format!("#! language \"{}\"\n\nfn main():\n    pass\n", ember_parser::LANGUAGE_VERSION));
    }

    #[test]
    fn the_spellings_m1_replaces_are_rewritten() {
        let (text, left) = migrated(
            "struct Pair[T: Clone + Eq]:\n    a: T\n\n    @borrows(self.a)\n    fn first() -> ref T:\n        return ref this.a\n\n\
             @borrows(xs)\nfn pick[T](xs: ref Array[T], ys: ref Array[T]) -> ref T:\n    return ref xs[0]\n\n\
             @static_safe\nfn bump(mut n: int, owned s: String, c: ref mut int) where T: Ord + Hash:\n    n += 1\n\n\
             fn main():\n    name: str = \"a\"\n    h: Option[int] = Some(1)\n    if Some(mut v) = h:\n        v += 1\n    \
             while Some(x) = it.next():\n        pass\n    if Some(Mode.Fast) = m:\n        pass\n    \
             for x in owned xs:\n        pass\n    total = xs.iter().copied().sum()\n",
        );
        let expected = "[T: needs (Clone, Eq)]\nstruct Pair:\n    a: T\n\n    fn first() -> ref T from a:\n        return ref this.a\n\n\
             [T]\nfn pick(xs: ref Array[T], ys: ref Array[T]) -> ref T from xs:\n    return ref xs[0]\n\n\
             @no_alias_checks\nfn bump(ref mut n: int, owns s: String, ref mut c: int) where T: needs (Ord, Hash):\n    n += 1\n\n\
             fn main():\n    name: StrView = \"a\"\n    h: Option[int] = Some(1)\n    if h exists as copy of v:\n        v += 1\n    \
             while it.next() exists as x:\n        pass\n    if Some(Mode.Fast) = m:\n        pass\n    \
             for x in owns xs:\n        pass\n    total = xs.iter().copy_each().sum()\n";
        assert_eq!(text, expected);
        assert_eq!(left.len(), 1, "{left:?}");
    }

    #[test]
    fn self_outside_a_body_is_this() {
        let (text, _) = migrated("struct T:
    total: int

    fn add(delta: int = self.total):
        this.total += delta
");
        assert_eq!(text, "struct T:
    total: int

    fn add(delta: int = this.total):
        this.total += delta
");
    }

    #[test]
    fn brackets_leave_the_rest_of_the_line_and_borrows_with_no_from_go() {
        let (text, left) = migrated(
            "class Pixel[T] implements Render:\n    v: T\n\n    @borrows(self)\n    fn all() owns this -> ref T:\n        return ref this.v\n\n    @borrows(self.a, self.b)\n    fn both() -> ref T:\n        return ref this.v\n",
        );
        let expected = "[T]\nclass Pixel implements Render:\n    v: T\n\n    fn all() owns this -> ref T:\n        return ref this.v\n\n    fn both() -> ref T:\n        return ref this.v\n";
        assert_eq!(text, expected);
        assert!(left.is_empty(), "{left:?}");
    }

    #[test]
    fn an_associated_type_needs_its_bounds() {
        let (text, _) = migrated("interface Sized:\n    type Count: ItemCount = int\n    type Iter: Iterator + Clone\n");
        assert_eq!(text, "interface Sized:\n    type Count: needs ItemCount = int\n    type Iter: needs (Iterator, Clone)\n");
    }

    #[test]
    fn a_file_with_its_own_str_keeps_it() {
        let (text, left) = migrated("struct str:\n    n: int\n\nfn f(s: str) -> int:\n    return s.n\n");
        assert!(text.contains("fn f(s: str)"), "{text}");
        assert_eq!(left.len(), 1);
    }
}
