//! Items, members, generics and type expressions (Part III §§1–3).

use ember_ast::*;
use ember_diag::{Diagnostic, codes};
use ember_lexer::{Kw, Lit, Punct, TokenKind};

use crate::Parser;

impl Parser<'_> {
    // -- imports ------------------------------------------------------------

    pub(crate) fn parse_import(&mut self) -> Option<Import> {
        let start = self.span();
        let id = self.next_id();

        // ERR-017 — `from` is contextual: a keyword only here, where it
        // begins an import. Everywhere else it is an ordinary name, which is
        // what lets `interface From[T]` declare `fn from(…)`.
        if self.at_contextual("from") {
            self.bump();
            let path = self.parse_dotted_path();
            self.expect_kw(Kw::Import);
            let (items, glob) = self.parse_import_list();
            self.expect_newline();
            return Some(Import {
                id,
                kind: ImportKind::Items { path, items, glob },
                span: start.to(self.prev_span()),
                public: false,
            });
        }

        self.expect_kw(Kw::Import);

        // `import c "header.h"` / `import cpp "header.h"`
        let foreign = match self.peek() {
            TokenKind::Ident(s) if s.is("c") || s.is("cpp") => {
                matches!(self.peek_at(1), TokenKind::Lit(Lit::Str(_)))
            }
            _ => false,
        };
        if foreign {
            let language = match self.peek() {
                TokenKind::Ident(s) if s.is("cpp") => ForeignLanguage::Cpp,
                _ => ForeignLanguage::C,
            };
            self.bump();
            let TokenKind::Lit(Lit::Str(header)) = self.peek().clone() else {
                unreachable!("checked above")
            };
            self.bump();
            let mut options = Vec::new();
            if self.eat_kw(Kw::With) {
                self.expect_punct(Punct::LParen);
                while !self.at_punct(Punct::RParen) && !self.at_eof() {
                    if let Some(arg) = self.parse_attr_arg() {
                        options.push(arg);
                    }
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::RParen);
            }
            let alias = self.eat_kw(Kw::As).then(|| self.expect_ident());
            self.expect_newline();
            return Some(Import {
                id,
                kind: ImportKind::Foreign { language, header, options, alias },
                span: start.to(self.prev_span()),
                public: false,
            });
        }

        let path = self.parse_dotted_path();
        let alias = self.eat_kw(Kw::As).then(|| self.expect_ident());
        self.expect_newline();
        Some(Import {
            id,
            kind: ImportKind::Module { path, alias },
            span: start.to(self.prev_span()),
            public: false,
        })
    }

    fn parse_import_list(&mut self) -> (Vec<ImportItem>, bool) {
        // `[MOD-3]` — `import *` is permitted only from a `@prelude` module,
        // which name resolution checks; the grammar accepts it here.
        if self.eat_punct(Punct::Star) {
            return (Vec::new(), true);
        }
        let parenthesised = self.eat_punct(Punct::LParen);
        let mut items = Vec::new();
        loop {
            if self.at_newline() || self.at_eof() {
                break;
            }
            if parenthesised && self.at_punct(Punct::RParen) {
                break;
            }
            let name = self.expect_ident();
            let alias = self.eat_kw(Kw::As).then(|| self.expect_ident());
            items.push(ImportItem { name, alias });
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        if parenthesised {
            self.expect_punct(Punct::RParen);
        }
        (items, false)
    }

    pub(crate) fn parse_dotted_path(&mut self) -> Vec<Ident> {
        let mut path = vec![self.expect_ident()];
        // `[LEX-14]` — a raw identifier is a segment like any other (`cpp.detail.r#where`).
        while self.at_punct(Punct::Dot) && matches!(self.peek_at(1), TokenKind::Ident(_) | TokenKind::RawIdent(_)) {
            self.bump();
            path.push(self.expect_ident());
        }
        path
    }

    // -- attributes and visibility -------------------------------------------

    pub(crate) fn parse_attributes(&mut self) -> Vec<Attribute> {
        let mut attrs = Vec::new();
        while self.at_punct(Punct::At) {
            let start = self.span();
            self.bump();
            let id = self.next_id();
            let path = self.parse_dotted_path();
            let mut args = Vec::new();
            if self.eat_punct(Punct::LParen) {
                let in_ffi = path.len() == 1 && path[0].name.is("ffi");
                let outer_ffi = std::mem::replace(&mut self.in_ffi_attribute, in_ffi);
                while !self.at_punct(Punct::RParen) && !self.at_eof() {
                    match self.parse_attr_arg() {
                        Some(arg) => args.push(arg),
                        None => break,
                    }
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::RParen);
                self.in_ffi_attribute = outer_ffi;
            }
            attrs.push(Attribute { id, path, args, span: start.to(self.prev_span()) });
            self.eat_newlines();
        }
        attrs
    }

    pub(crate) fn parse_attr_arg(&mut self) -> Option<AttrArg> {
        if self.at_ident() && self.at_punct_at(1, Punct::Eq) {
            let name = self.expect_ident();
            self.bump(); // '='
            // `[PAR-3]` — `reduce=[total: +, best: max]`: each a name and an operator, kept as
            // `(name, operator)` pairs in a list, the operator a path of its own spelling.
            if name.name.is("reduce") && self.at_punct(Punct::LBracket) {
                let start = self.span();
                self.bump();
                let mut items = Vec::new();
                while !self.at_punct(Punct::RBracket) {
                    let var = self.expect_ident();
                    self.expect_punct(Punct::Colon);
                    let op_span = self.span();
                    let op = match self.peek().clone() {
                        TokenKind::Punct(Punct::Plus) => "+".to_string(),
                        TokenKind::Punct(Punct::Star) => "*".to_string(),
                        TokenKind::Punct(Punct::Amp) => "&".to_string(),
                        TokenKind::Punct(Punct::Pipe) => "|".to_string(),
                        TokenKind::Punct(Punct::Caret) => "^".to_string(),
                        TokenKind::Keyword(Kw::And) => "and".to_string(),
                        TokenKind::Keyword(Kw::Or) => "or".to_string(),
                        TokenKind::Ident(word) => word.as_str().to_string(),
                        _ => {
                            self.expect_ident();
                            break;
                        }
                    };
                    self.bump();
                    let path = |ident: Ident| Expr { id: NodeId::DUMMY, kind: ExprKind::Path { segments: vec![ident] }, span: ident.span };
                    let op = Ident { name: op.as_str().into(), span: op_span };
                    let span = var.span.to(op_span);
                    items.push(Expr { id: NodeId::DUMMY, kind: ExprKind::Tuple(vec![path(var), path(op)]), span });
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::RBracket);
                let value = Expr { id: NodeId::DUMMY, kind: ExprKind::ArrayLit(items), span: start.to(self.prev_span()) };
                return Some(AttrArg::Named { name, value });
            }
            let value = self.parse_expr();
            return Some(AttrArg::Named { name, value });
        }
        Some(AttrArg::Expr(self.parse_expr()))
    }

    /// `pub`, `pub(package)`, `pub(read)`, `pub(package, read)` (`[MOD-7]`).
    /// Returns the visibility and whether `read` was present.
    fn parse_visibility(&mut self) -> (Visibility, bool) {
        let start = self.span();
        if !self.eat_kw(Kw::Pub) {
            return (Visibility::private(), false);
        }
        let mut kind = VisKind::Public;
        let mut read = false;
        if self.eat_punct(Punct::LParen) {
            loop {
                if self.at_contextual("package") {
                    self.bump();
                    kind = VisKind::Package;
                } else if self.at_contextual("read") {
                    self.bump();
                    read = true;
                } else {
                    let span = self.span();
                    self.report_code(
                        codes::E0100,
                        span,
                        "expected `package` or `read` in a visibility qualifier",
                    );
                    self.bump();
                }
                if !self.eat_punct(Punct::Comma) {
                    break;
                }
            }
            self.expect_punct(Punct::RParen);
        }
        (Visibility { kind, span: start.to(self.prev_span()) }, read)
    }

    // -- items ----------------------------------------------------------------

    /// `[GRM-2]` (0.9.9) — whether the next tokens begin an item, exactly as
    /// `parse_item_kind` dispatches. Anything else at file scope is a script
    /// statement. `unsafe:` is a statement; `unsafe fn` and `unsafe extern`
    /// are items.
    pub(crate) fn at_item_start(&self) -> bool {
        match self.peek() {
            TokenKind::DocComment(_) | TokenKind::Reserved(_) => true,
            TokenKind::Punct(Punct::At) => true,
            TokenKind::Keyword(
                Kw::Pub
                | Kw::Extern
                | Kw::Fn
                | Kw::Virtual
                | Kw::Override
                | Kw::Struct
                | Kw::Class
                | Kw::Enum
                | Kw::Interface
                | Kw::Const
                | Kw::Static
                | Kw::Comptime
                | Kw::Type,
            ) => true,
            TokenKind::Keyword(Kw::Unsafe) => {
                self.at_fn_at(1)
                    || self.at_kw_at(1, Kw::Extern)
                    || matches!(self.peek_at(1), TokenKind::Ident(s) if s.is("extend"))
                    || self.at_overlay_decl(1)
            }
            TokenKind::Ident(s) if s.is("abstract") || s.is("open") => self.at_kw_at(1, Kw::Class),
            TokenKind::Ident(s) if s.is("safe") => self.at_fn_at(1),
            TokenKind::Ident(_) => {
                self.at_gen_fn() || self.at_extend_decl() || self.at_union_decl() || self.at_overlay_decl(0)
            }
            _ => false,
        }
    }

    /// ODR-035 — where a `union` declaration would begin: `union` and a name.
    /// The word is reserved there and an identifier everywhere else.
    pub(crate) fn at_union_decl(&self) -> bool {
        matches!(self.peek(), TokenKind::Ident(s) if s.is("union"))
            && matches!(self.peek_at(1), TokenKind::Ident(_))
    }

    pub(crate) fn parse_item(&mut self) -> Option<Item> {
        let doc_span = self.span();
        let doc = self.take_doc();
        // A `##` comment with no declaration after it is discarded in silence
        // (ERR-007). Running out of file here is that case, not a syntax error.
        if doc.is_some() && self.at_eof() {
            self.discard_dangling_doc(doc);
            return None;
        }
        let _ = doc_span;
        let attrs = self.parse_attributes();
        let start = self.span();
        let (vis, read) = self.parse_visibility();
        if read {
            let span = start.to(self.prev_span());
            self.report_code(codes::E1051, span, "`read` visibility is valid on fields only");
        }

        let id = self.next_id();
        let kind = self.parse_item_kind()?;
        self.check_fn_bodies(&kind);
        Some(Item { id, attrs, vis, doc, kind, span: start.to(self.prev_span()) })
    }

    /// `[GRM-33]` — a bodiless `fn_decl` is legal only in an `interface`, an
    /// `extern` block, or as a `virtual` method of an `abstract class`.
    fn check_fn_bodies(&mut self, kind: &ItemKind) {
        // The bodiless methods among `members`, but a virtual one of an
        // abstract class.
        let bodiless = |members: &[Member], abstract_class: bool| -> Vec<Ident> {
            members
                .iter()
                .filter_map(|member| match &member.kind {
                    MemberKind::Fn(decl)
                        if decl.body.is_none() && !(abstract_class && decl.dispatch == Dispatch::Virtual) =>
                    {
                        Some(decl.name.clone())
                    }
                    _ => None,
                })
                .collect()
        };
        let names = match kind {
            ItemKind::Fn(decl) if decl.body.is_none() && !self.in_extern_block => vec![decl.name.clone()],
            ItemKind::Struct(decl) => bodiless(&decl.members, false),
            ItemKind::Enum(decl) => bodiless(&decl.members, false),
            ItemKind::Extend(decl) => bodiless(&decl.members, false),
            ItemKind::Class(decl) => bodiless(&decl.members, decl.openness == Openness::Abstract),
            _ => Vec::new(),
        };
        let names: Vec<Ident> = names.into_iter().filter(|name| !self.damaged_signatures.contains(&name.span)).collect();
        for name in names {
            self.report(
                Diagnostic::error(codes::E0110, name.span, format!("`{}` is declared without a body", name.name))
                    .primary_label("a function here needs a body")
                    .help("add ':' and a body"),
            );
        }
    }

    fn parse_item_kind(&mut self) -> Option<ItemKind> {
        // Three item forms open with `extern`, and the token after the ABI
        // string is what tells them apart:
        //
        //     extern "C":              a block, DECLARING foreign functions
        //     extern "C" fn f(...):    a definition with a foreign ABI (XVI.10)
        //     extern class Path:       a declared foreign base ([FFI-39])
        //
        // Only the first existed. The second is what `@export` attaches to and
        // is the shape a host embeds against; the third carries the whole of
        // `[FFI-39]`'s bounded inheritance model. Neither had a production —
        // errata ERR-036 and ERR-037 — so neither parsed, though the document
        // writes both.
        if self.at_kw(Kw::Extern) && self.at_kw_at(1, Kw::Class) {
            return Some(ItemKind::ExternClass(self.parse_extern_class()));
        }
        // `[GRM-35]` — `[unsafe] overlay c "header":`; `overlay` is a keyword only
        // at the start of an item (`[LEX-15]`).
        if self.at_overlay_decl(0) {
            return Some(ItemKind::Overlay(self.parse_overlay(false)));
        }
        if self.at_kw(Kw::Unsafe) && self.at_overlay_decl(1) {
            self.bump();
            return Some(ItemKind::Overlay(self.parse_overlay(true)));
        }
        // `[ALC-2]` (ODR-175) — `unsafe interface Allocator:`, one whose implementation is `unsafe`.
        if self.at_kw(Kw::Unsafe) && self.at_kw_at(1, Kw::Interface) {
            self.bump();
            let mut decl = self.parse_interface();
            decl.is_unsafe = true;
            return Some(ItemKind::Interface(decl));
        }
        // ODR-111 — `unsafe extend T implements Send:`, a marker interface written by hand.
        if self.at_kw(Kw::Unsafe) && matches!(self.peek_at(1), TokenKind::Ident(s) if s.is("extend")) {
            self.bump();
            if self.at_extend_decl() {
                let mut decl = self.parse_extend();
                decl.is_unsafe = true;
                return Some(ItemKind::Extend(decl));
            }
        }
        // `unsafe extern "C":` and `unsafe extern "C" fn`.
        if self.at_kw(Kw::Unsafe) && self.at_kw_at(1, Kw::Extern) {
            if self.at_fn_at(3) {
                return Some(ItemKind::Fn(self.parse_fn()));
            }
            self.bump();
            return Some(ItemKind::ExternBlock(self.parse_extern_block(true)));
        }
        if self.at_kw(Kw::Extern) {
            if self.at_fn_at(2) {
                return Some(ItemKind::Fn(self.parse_fn()));
            }
            return Some(ItemKind::ExternBlock(self.parse_extern_block(false)));
        }

        match self.peek() {
            TokenKind::Keyword(Kw::Fn | Kw::Unsafe | Kw::Virtual | Kw::Override) => {
                Some(ItemKind::Fn(self.parse_fn()))
            }
            // `[GRM-21]` — a `gen fn` is admitted wherever `fn_decl` is.
            TokenKind::Ident(_) if self.at_gen_fn() => Some(ItemKind::Fn(self.parse_fn())),
            TokenKind::Ident(s) if s.is("safe") && self.at_fn_at(1) => {
                Some(ItemKind::Fn(self.parse_fn()))
            }
            TokenKind::Keyword(Kw::Struct) => Some(ItemKind::Struct(self.parse_struct())),
            TokenKind::Keyword(Kw::Class) => Some(ItemKind::Class(self.parse_class())),
            // `[LEX-15]` (ODR-134) — `open` is a keyword only before `class`.
            TokenKind::Ident(s) if s.is("open") && self.at_kw_at(1, Kw::Class) => {
                Some(ItemKind::Class(self.parse_class()))
            }
            TokenKind::Keyword(Kw::Enum) => Some(ItemKind::Enum(self.parse_enum())),
            TokenKind::Keyword(Kw::Interface) => {
                Some(ItemKind::Interface(self.parse_interface()))
            }
            TokenKind::Ident(_) if self.at_extend_decl() => Some(ItemKind::Extend(self.parse_extend())),
            TokenKind::Keyword(Kw::Const) => Some(ItemKind::Const(self.parse_const())),
            TokenKind::Keyword(Kw::Static) => Some(ItemKind::Static(self.parse_static())),
            TokenKind::Keyword(Kw::Comptime) => {
                self.bump();
                self.expect_punct(Punct::Colon);
                Some(ItemKind::Comptime(self.parse_block()))
            }
            // `[LEX-15a]` — `type` introduces a type alias at item level and an
            // opaque foreign type inside an `extern` block, where it carries no
            // `= T`. Both are `type_alias` nodes; the value distinguishes them.
            TokenKind::Keyword(Kw::Type) => Some(ItemKind::TypeAlias(self.parse_type_alias(true))),
            // `abstract class` — `abstract` is contextual (`[LEX-15]`).
            TokenKind::Ident(s) if s.is("abstract") && self.at_kw_at(1, Kw::Class) => {
                Some(ItemKind::Class(self.parse_class()))
            }
            // ODR-035 — `union` is contextual: reserved only where a
            // declaration would begin, an item `union` and a name.
            TokenKind::Ident(_) if self.at_union_decl() => {
                let span = self.span();
                self.bump();
                self.report(
                    Diagnostic::error(codes::E0005, span, "`union` is reserved for a later version")
                        .help("rename the item, or write `r#union` to use the word as an identifier"),
                );
                None
            }
            TokenKind::Reserved(reserved) => {
                let reserved = *reserved;
                let span = self.span();
                self.bump();
                // `[LEX-14a]` — the message MUST name the version that will
                // introduce the word, so a reservation is never read as a typo.
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
                        "rename the item, or write `r#{}` to use the word as an identifier",
                        reserved.as_str()
                    )),
                );
                None
            }
            other => {
                let found = other.to_string();
                let span = self.span();
                self.report(
                    Diagnostic::error(codes::E0100, span, "expected a declaration")
                        .primary_label(format!("found {found}"))
                        .help("items are `fn`, `struct`, `class`, `enum`, `interface`, `extend`, `const`, `static` or `extern`"),
                );
                None
            }
        }
    }

    /// `type Name = T` (alias), `type Item` / `type Item: Bound` (associated
    /// type in an interface), `type Name` (opaque foreign type in an `extern`
    /// block). One production serves all three (`[LEX-15a]`); the caller knows
    /// which context it is in, and `value` distinguishes an alias from the rest.
    /// `type_alias := "type" identifier [generic_params] "=" type [range_clause] NEWLINE`
    ///
    /// `at_item_level` decides whether the `range_clause` is admitted:
    /// `[GRM-8d]` allows it only where the alias appears as an `item`, so an
    /// associated type in an `interface` and an opaque type in an `extern`
    /// block are `E2213`. The clause is parsed either way, because a rejection
    /// that names the clause reads better than one that stops at `in`.
    fn parse_type_alias(&mut self, at_item_level: bool) -> TypeAlias {
        self.expect_kw(Kw::Type);
        let name = self.expect_ident();
        let generics = self.parse_generic_params();
        let mut bounds = Vec::new();
        if self.eat_punct(Punct::Colon) {
            bounds.push(self.parse_type());
            while self.eat_punct(Punct::Plus) {
                bounds.push(self.parse_type());
            }
        }
        let value = self.eat_punct(Punct::Eq).then(|| self.parse_type());
        let range = self.parse_range_clause(&name, at_item_level, &generics);
        self.expect_newline();
        TypeAlias { name, generics, is_foreign_decl: false, value, bounds, range }
    }

    /// `range_clause := "in" expression` — `[RNG-1]`.
    ///
    /// `[GRM-8d]` makes the production LL(2): no other `type_alias` may be
    /// followed by `in`, and a `type_alias` never appears where a `for_stmt`
    /// may begin, so `for x in …` is untouched. The parser therefore commits
    /// on the enclosing construct and never on the token, which is what
    /// `[GRM-23]` requires of every use of `in`.
    fn parse_range_clause(
        &mut self,
        name: &Ident,
        at_item_level: bool,
        generics: &[GenericParam],
    ) -> Option<Expr> {
        if !self.at_kw(Kw::In) {
            return None;
        }
        let clause_start = self.span();
        self.bump();
        let expr = self.parse_expr_no_block();
        let span = clause_start.to(self.prev_span());

        if !at_item_level {
            self.report(
                Diagnostic::error(
                    codes::E2213,
                    span,
                    "a range type may only be declared at item level",
                )
                .help(
                    "declare it as a module-level `type`, and name that type here"
                        .to_string(),
                ),
            );
            return None;
        }
        // `[GRM-8d]` — "a range type is over a concrete representation", so an
        // alias carrying generic parameters cannot carry a range.
        if !generics.is_empty() {
            self.report(
                Diagnostic::error(
                    codes::E2213,
                    name.span.to(span),
                    "a range type may not be generic",
                )
                .help(
                    "a range is over one concrete representation; declare one \
                     `type` per representation"
                        .to_string(),
                ),
            );
            return None;
        }
        Some(expr)
    }

    /// `[FFI-39]` — `extern class cpp.RageV.Layer:`.
    ///
    /// The body is an ordinary `type_body`, so members parse exactly as a
    /// class's do. Nothing past the parser understands one yet: Part XVI's C++
    /// boundary is Phase 7, and the type checker refuses it by name rather than
    /// accepting it into a stage that would ignore it.
    fn parse_extern_class(&mut self) -> ExternClass {
        let start = self.span();
        self.expect_kw(Kw::Extern);
        self.expect_kw(Kw::Class);
        let mut path = vec![self.expect_ident()];
        while self.eat_punct(Punct::Dot) {
            path.push(self.expect_ident());
        }
        let name = *path.last().expect("at least one segment");
        self.expect_punct(Punct::Colon);
        let members = self.parse_type_body();
        ExternClass { name, path, members, span: start.to(self.prev_span()) }
    }

    /// The ABI string after `extern`. `[FFI-1]` admits `"C"` and `"C++"`.
    fn expect_abi_string(&mut self) -> String {
        match self.peek().clone() {
            TokenKind::Lit(Lit::Str(value)) => {
                self.bump();
                value
            }
            _ => {
                self.report_code(
                    codes::E0100,
                    self.span(),
                    "expected an ABI string, as `extern \"C\" fn`",
                );
                String::from("C")
            }
        }
    }

    fn parse_extern_block(&mut self, is_unsafe: bool) -> ExternBlock {
        self.expect_kw(Kw::Extern);
        let abi = match self.peek().clone() {
            TokenKind::Lit(Lit::Str(s)) => {
                self.bump();
                s
            }
            _ => "C".to_string(),
        };
        self.expect_punct(Punct::Colon);
        let mut items = Vec::new();
        let outer_extern = std::mem::replace(&mut self.in_extern_block, true);
        if self.at_newline() {
            self.bump();
            if self.eat_indent() {
                while !self.at_dedent() && !self.at_eof() {
                    self.eat_newlines();
                    if self.at_dedent() || self.at_eof() {
                        break;
                    }
                    let before = self.pos;
                    match self.parse_item() {
                        Some(item) => items.push(item),
                        None => {
                            if self.pos == before {
                                self.bump();
                            }
                            self.recover_to_line_end();
                        }
                    }
                }
                self.eat_dedent();
            }
        }
        self.in_extern_block = outer_extern;
        ExternBlock { is_unsafe, abi, items }
    }

    /// `overlay c "header"` begins at token `n`.
    fn at_overlay_decl(&self, n: usize) -> bool {
        matches!(self.peek_at(n), TokenKind::Ident(word) if word.is("overlay"))
            && matches!(self.peek_at(n + 1), TokenKind::Ident(language) if language.is("c") || language.is("cpp"))
            && matches!(self.peek_at(n + 2), TokenKind::Lit(Lit::Str(_)))
    }

    /// `[GRM-35]` — `overlay ("c" | "cpp") STRING ":" NEWLINE INDENT {overlay_item} DEDENT`,
    /// after any `unsafe`.
    fn parse_overlay(&mut self, is_unsafe: bool) -> OverlayDecl {
        let start = self.span();
        self.bump(); // `overlay`
        let language = match self.peek() {
            TokenKind::Ident(language) if language.is("cpp") => ForeignLanguage::Cpp,
            _ => ForeignLanguage::C,
        };
        self.bump();
        let TokenKind::Lit(Lit::Str(header)) = self.peek().clone() else { unreachable!("checked by at_overlay_decl") };
        self.bump();
        self.expect_punct(Punct::Colon);
        let span = start.to(self.prev_span());
        let mut items = Vec::new();
        if self.at_newline() {
            self.bump();
            if self.eat_indent() {
                while !self.at_dedent() && !self.at_eof() {
                    self.eat_newlines();
                    if self.at_dedent() || self.at_eof() {
                        break;
                    }
                    let before = self.pos;
                    match self.parse_overlay_item() {
                        Some(item) => items.push(item),
                        None => {
                            if self.pos == before {
                                self.bump();
                            }
                            self.recover_to_line_end();
                        }
                    }
                }
                self.eat_dedent();
            }
        } else {
            self.expect_newline();
        }
        OverlayDecl { span, is_unsafe, language, header, items }
    }

    /// `overlay_item := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
    /// | "rename" IDENT "as" IDENT NEWLINE | "hide" IDENT NEWLINE | extend_decl | overlay_class)`;
    /// `rename` and `hide` are keywords only here (`[LEX-15]`).
    fn parse_overlay_item(&mut self) -> Option<OverlayItem> {
        // `[FFI-30c]` (ODR-149) — `override` replaces an earlier overlay's entry: the item begins
        // with it, before or after its attributes.
        let leading = self.eat_kw(Kw::Override);
        let attrs = self.parse_attributes();
        let start = self.span();
        let is_override = self.eat_kw(Kw::Override) || leading;
        // `[FFI-6b]` (ODR-147) — an entry for a function-like macro writes Ember types.
        let macro_fn = attrs.iter().any(|attr| {
            attr.path.len() == 1 && attr.path[0].name.is("ffi")
                && attr.args.iter().any(|arg| matches!(arg, AttrArg::Expr(Expr { kind: ExprKind::Path { segments }, .. })
                    if segments.len() == 1 && segments[0].name.is("macro_fn")))
        });
        let kind = match self.peek().clone() {
            TokenKind::Keyword(Kw::Fn) if macro_fn => {
                self.bump();
                let name = self.expect_ident();
                self.expect_punct(Punct::LParen);
                let mut params = Vec::new();
                while !self.at_punct(Punct::RParen) && !self.at_eof() && !self.at_newline() {
                    let name = self.expect_ident();
                    self.expect_punct(Punct::Colon);
                    params.push((name, self.parse_type()));
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                    self.eat_newlines();
                }
                self.expect_punct(Punct::RParen);
                let result = self.eat_punct(Punct::Arrow).then(|| self.parse_type());
                self.expect_newline();
                OverlayItemKind::MacroFn { name, params, result }
            }
            TokenKind::Keyword(Kw::Fn) => {
                self.bump();
                let name = self.expect_ident();
                self.expect_punct(Punct::LParen);
                let mut params = Vec::new();
                while !self.at_punct(Punct::RParen) && !self.at_eof() && !self.at_newline() {
                    let name = self.expect_ident();
                    let contract = if self.eat_punct(Punct::Colon) { self.parse_contract_words() } else { Vec::new() };
                    params.push(OverlayParam { name, contract });
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                    self.eat_newlines();
                }
                self.expect_punct(Punct::RParen);
                let result = if self.eat_punct(Punct::Arrow) { self.parse_contract_words() } else { Vec::new() };
                self.expect_newline();
                OverlayItemKind::Fn { name, params, result }
            }
            TokenKind::Keyword(Kw::Type) => {
                self.bump();
                let name = self.expect_ident();
                self.expect_newline();
                OverlayItemKind::Type(name)
            }
            TokenKind::Keyword(Kw::Enum) => {
                self.bump();
                let name = self.expect_ident();
                self.expect_newline();
                OverlayItemKind::Enum(name)
            }
            TokenKind::Ident(word) if word.is("rename") => {
                self.bump();
                let from = self.expect_ident();
                self.expect_kw(Kw::As);
                let to = self.expect_ident();
                self.expect_newline();
                OverlayItemKind::Rename { from, to }
            }
            TokenKind::Ident(word) if word.is("hide") => {
                self.bump();
                let name = self.expect_ident();
                self.expect_newline();
                OverlayItemKind::Hide(name)
            }
            TokenKind::Ident(_) if self.at_extend_decl() => OverlayItemKind::Extend(self.parse_extend()),
            // `[FFI-40a]` (ODR-160) — `class C:` and a method line for each method it restates.
            TokenKind::Keyword(Kw::Class) => {
                self.bump();
                let name = self.expect_ident();
                self.expect_punct(Punct::Colon);
                let mut methods = Vec::new();
                if self.at_newline() {
                    self.bump();
                    if self.eat_indent() {
                        while !self.at_dedent() && !self.at_eof() {
                            self.eat_newlines();
                            if self.at_dedent() || self.at_eof() {
                                break;
                            }
                            if !self.at_kw(Kw::Fn) {
                                let span = self.span();
                                self.report(
                                    Diagnostic::error(codes::E0100, span, "expected a method line")
                                        .help("an overlay's `class` section restates methods, as `fn name(self, …) -> T` [GRM-35]"),
                                );
                                self.bump();
                                self.recover_to_line_end();
                                continue;
                            }
                            let method = self.parse_method();
                            if let Some(body) = &method.body {
                                self.report(
                                    Diagnostic::error(codes::E0100, body.span, "a method line of an overlay has no body")
                                        .help("the C++ class's own method runs; the line states its signature [FFI-40a]"),
                                );
                            }
                            methods.push(method);
                        }
                        self.eat_dedent();
                    }
                } else {
                    self.expect_newline();
                }
                OverlayItemKind::Class { name, methods }
            }
            other => {
                let found = other.to_string();
                let span = self.span();
                self.report(
                    Diagnostic::error(codes::E0100, span, "expected an overlay item")
                        .primary_label(format!("found {found}"))
                        .help("an overlay holds `fn`, `type`, `enum`, `rename`, `hide`, `extend` and `class` items [GRM-35]"),
                );
                return None;
            }
        };
        Some(OverlayItem { attrs, is_override, kind, span: start.to(self.prev_span()) })
    }

    /// `contract {contract}`, `contract := IDENT ["(" [attr_arg {"," attr_arg}] ")"]`: words
    /// separated by spaces, until a `,`, a `)` or the end of the line. `unsafe` is a word here
    /// (`[FFI-11]`: leave the call unsafe).
    fn parse_contract_words(&mut self) -> Vec<Contract> {
        let mut words = Vec::new();
        loop {
            let word = match self.peek() {
                TokenKind::Ident(_) | TokenKind::RawIdent(_) => self.expect_ident(),
                // `unsafe` and `owned` are keywords elsewhere and contract words here (`[FFI-11]`).
                &TokenKind::Keyword(keyword @ (Kw::Unsafe | Kw::Owned)) => {
                    let span = self.span();
                    self.bump();
                    Ident { name: keyword.as_str().into(), span }
                }
                _ => break,
            };
            let args = if self.eat_punct(Punct::LParen) {
                let outer_ffi = std::mem::replace(&mut self.in_ffi_attribute, true);
                let mut args = Vec::new();
                while !self.at_punct(Punct::RParen) && !self.at_eof() {
                    match self.parse_attr_arg() {
                        Some(arg) => args.push(arg),
                        None => break,
                    }
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::RParen);
                self.in_ffi_attribute = outer_ffi;
                Some(args)
            } else {
                None
            };
            words.push(Contract { word, args, span: word.span.to(self.prev_span()) });
        }
        if words.is_empty() {
            let span = self.span();
            self.report_code(codes::E0100, span, "expected a contract word, as `borrowed one`");
        }
        words
    }

    /// True where a `gen fn` begins. `[LEX-15b]` makes `gen` contextual — a
    /// keyword only immediately before `fn` — so the lookahead is exactly one
    /// token and a variable or field named `gen` is unaffected.
    pub(crate) fn at_gen_fn(&self) -> bool {
        self.at_gen_fn_at(0)
    }

    fn at_gen_fn_at(&self, n: usize) -> bool {
        matches!(self.peek_at(n), TokenKind::Ident(s) if s.is("gen"))
            && matches!(self.peek_at(n + 1), TokenKind::Keyword(Kw::Fn))
    }

    /// `fn`, or `gen fn`, `n` tokens ahead: `fn_header`'s last two words.
    fn at_fn_at(&self, n: usize) -> bool {
        self.at_kw_at(n, Kw::Fn) || self.at_gen_fn_at(n)
    }

    /// True where an `extend` block begins. `extend` is contextual (ODR-030):
    /// a keyword only before the type it extends — a name, or generic
    /// parameters and then a name — so `xs.extend(ys)` is a call and a script
    /// may still write `extend = 1` or `extend[0] = 1`.
    pub(crate) fn at_extend_decl(&self) -> bool {
        if !self.at_contextual("extend") {
            return false;
        }
        let mut n = 1;
        if self.at_punct_at(1, Punct::LBracket) {
            let mut depth = 0;
            loop {
                match self.peek_at(n) {
                    TokenKind::Punct(Punct::LBracket) => depth += 1,
                    TokenKind::Punct(Punct::RBracket) => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    TokenKind::Newline | TokenKind::Eof => return false,
                    _ => {}
                }
                n += 1;
            }
            n += 1;
        }
        // A tuple type (`extend[A, B] (A, B) implements I:`): a declaration only when the line
        // ends with its `:`, so a call `extend[i](x)` stays one.
        if matches!(self.peek_at(n), TokenKind::Punct(Punct::LParen)) && n > 1 {
            let mut last = None;
            let mut at = n;
            loop {
                match self.peek_at(at) {
                    TokenKind::Newline | TokenKind::Eof => break,
                    other => last = Some(other.clone()),
                }
                at += 1;
            }
            return matches!(last, Some(TokenKind::Punct(Punct::Colon)));
        }
        matches!(self.peek_at(n), TokenKind::Ident(_) | TokenKind::RawIdent(_))
    }

    /// A function standing alone (an item or a local function): its name is an identifier.
    pub(crate) fn parse_fn(&mut self) -> FnDecl {
        self.parse_fn_decl(false)
    }

    /// A method (a member of a type, an `extend` block or an interface): its name may also be
    /// `not` or `is` ([LEX-15]).
    fn parse_method(&mut self) -> FnDecl {
        self.parse_fn_decl(true)
    }

    fn parse_fn_decl(&mut self, method: bool) -> FnDecl {
        let is_safe = matches!(self.peek(), TokenKind::Ident(s) if s.is("safe"));
        if is_safe {
            self.bump();
        }
        let unsafe_site = self.span();
        let is_unsafe = self.eat_kw(Kw::Unsafe);
        // `["extern" string_lit]` — the ABI a definition uses. It sits after
        // `unsafe`, matching `unsafe extern "C":` for the block form.
        let abi = if self.at_kw(Kw::Extern) {
            self.bump();
            Some(self.expect_abi_string())
        } else {
            None
        };
        let dispatch = if self.eat_kw(Kw::Virtual) {
            Dispatch::Virtual
        } else if self.eat_kw(Kw::Override) {
            Dispatch::Override
        } else {
            Dispatch::Static
        };
        // `[GRM-21]` — `fn_header := ["extern" STRING] ["unsafe"] ["virtual" |
        // "override"] ["gen"] "fn" …`: `gen` comes last, right before `fn`
        // (D-660: it was taken first, so `extern "C" gen fn` did not parse).
        let is_gen = self.at_gen_fn();
        if is_gen {
            self.bump();
        }
        self.expect_kw(Kw::Fn);
        let signature_start = self.pos;
        let name = if method { self.expect_member_name() } else { self.expect_fn_name() };
        let generics = self.parse_generic_params();

        self.expect_punct(Punct::LParen);
        let mut params = Vec::new();
        while !self.at_punct(Punct::RParen) && !self.at_eof() {
            params.push(self.parse_param());
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        self.expect_punct(Punct::RParen);

        let ret = self.eat_punct(Punct::Arrow).then(|| self.parse_type());
        let where_clause = self.parse_where_clause();

        // A body is present unless this is an interface or extern signature.
        // `[UNS-8]` — an `unsafe fn` with one needs a safety note.
        if is_unsafe && self.at_punct(Punct::Colon) && !self.in_extern_block {
            self.unsafe_sites.push(unsafe_site);
        }
        let body = if self.eat_punct(Punct::Colon) { Some(self.parse_block()) } else { None };
        if body.is_none() {
            if self.tokens[signature_start..=self.pos.min(self.tokens.len() - 1)]
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Error))
            {
                self.damaged_signatures.push(name.span);
            }
            self.expect_newline();
        }

        FnDecl { name, is_unsafe, is_safe, is_foreign_decl: false, abi, is_gen, dispatch, generics, params, ret, where_clause, body }
    }

    fn parse_param(&mut self) -> Param {
        let start = self.span();
        let id = self.next_id();
        let mode = if self.eat_kw(Kw::Mut) {
            Mode::Mut
        } else if self.eat_kw(Kw::Owned) {
            Mode::Owned
        } else {
            Mode::Borrow
        };

        // `[FN-4]` — the receiver follows the same modes as any parameter.
        if self.at_kw(Kw::SelfValue) {
            self.bump();
            let ty = self.eat_punct(Punct::Colon).then(|| self.parse_type());
            return Param {
                id,
                mode,
                kind: ParamKind::Receiver { ty },
                default: None,
                span: start.to(self.prev_span()),
            };
        }

        let name = self.expect_ident();
        self.expect_punct(Punct::Colon);
        let ty = self.parse_type();
        let default = self.eat_punct(Punct::Eq).then(|| self.parse_expr());
        // `[CLO-6]` — calling a `once fn` parameter consumes it, so it is taken over as an
        // `owned` one is; `owned` is implied and need not be written.
        let mode = if matches!(ty.kind, TypeKind::Fn { once: true, .. }) { Mode::Owned } else { mode };
        Param {
            id,
            mode,
            kind: ParamKind::Named { name, ty },
            default,
            span: start.to(self.prev_span()),
        }
    }

    fn parse_struct(&mut self) -> StructDecl {
        self.expect_kw(Kw::Struct);
        let name = self.expect_ident();
        let generics = self.parse_generic_params();
        let implements = self.parse_implements();
        let where_clause = self.parse_where_clause();
        self.expect_punct(Punct::Colon);
        let members = self.parse_type_body();
        StructDecl { name, generics, implements, where_clause, members }
    }

    fn parse_class(&mut self) -> ClassDecl {
        let openness = if self.at_contextual("open") {
            self.bump();
            Openness::Open
        } else if self.at_contextual("abstract") {
            self.bump();
            Openness::Abstract
        } else {
            Openness::Final
        };
        self.expect_kw(Kw::Class);
        let name = self.expect_ident();
        let generics = self.parse_generic_params();
        // `[GRM-1]` — the base class is written in parentheses, at most one.
        let base = if self.eat_punct(Punct::LParen) {
            let base = self.parse_type();
            if self.at_punct(Punct::Comma) {
                let span = self.span();
                self.report_code(codes::E0100, span, "a class has at most one base class");
                while self.eat_punct(Punct::Comma) {
                    self.parse_type();
                }
            }
            self.expect_punct(Punct::RParen);
            Some(base)
        } else {
            None
        };
        let implements = self.parse_implements();
        let where_clause = self.parse_where_clause();
        self.expect_punct(Punct::Colon);
        let members = self.parse_type_body();
        ClassDecl { name, openness, generics, base, implements, where_clause, members }
    }

    fn parse_enum(&mut self) -> EnumDecl {
        self.expect_kw(Kw::Enum);
        let name = self.expect_ident();
        let generics = self.parse_generic_params();
        let implements = self.parse_implements();
        self.expect_punct(Punct::Colon);

        let mut variants = Vec::new();
        let mut members = Vec::new();
        if self.at_newline() {
            self.bump();
            if self.eat_indent() {
                while !self.at_dedent() && !self.at_eof() {
                    self.eat_newlines();
                    if self.at_dedent() || self.at_eof() {
                        break;
                    }
                    let doc = self.take_doc();
                    let attrs = self.parse_attributes();
                    // A variant is a bare name; anything else is a member,
                    // a `gen fn` among them (`[GRM-21]`, D-660).
                    let is_variant = self.at_ident()
                        && !self.at_gen_fn()
                        && !self.at_punct_at(1, Punct::Colon)
                        && !matches!(self.peek_at(1), TokenKind::Punct(Punct::Dot));
                    if is_variant {
                        variants.push(self.parse_variant(doc, attrs));
                    } else {
                        let before = self.pos;
                        match self.parse_member_with(doc, attrs) {
                            Some(m) => members.push(m),
                            None => {
                                if self.pos == before {
                                    self.bump();
                                }
                                self.recover_to_line_end();
                            }
                        }
                    }
                }
                self.eat_dedent();
            }
        }
        EnumDecl { name, generics, implements, variants, members }
    }

    fn parse_variant(&mut self, doc: Option<String>, attrs: Vec<Attribute>) -> Variant {
        let start = self.span();
        let id = self.next_id();
        let name = self.expect_ident();
        let mut fields = Vec::new();
        if self.eat_punct(Punct::LParen) {
            while !self.at_punct(Punct::RParen) && !self.at_eof() {
                let field_start = self.span();
                let attrs = self.parse_attributes();
                // `Circle(radius: f32)` names its field; `Io(io.Error)` does not.
                let named = self.at_ident() && self.at_punct_at(1, Punct::Colon);
                let name = named.then(|| {
                    let ident = self.expect_ident();
                    self.bump(); // ':'
                    ident
                });
                let ty = self.parse_type();
                fields.push(VariantField { attrs, name, ty, span: field_start.to(self.prev_span()) });
                if !self.eat_punct(Punct::Comma) {
                    break;
                }
            }
            self.expect_punct(Punct::RParen);
        }
        let discriminant = self.eat_punct(Punct::Eq).then(|| self.parse_expr());
        self.expect_newline();
        Variant { id, attrs, doc, name, fields, discriminant, span: start.to(self.prev_span()) }
    }

    fn parse_interface(&mut self) -> InterfaceDecl {
        self.expect_kw(Kw::Interface);
        let name = self.expect_ident();
        let generics = self.parse_generic_params();
        // `interface Ord: Eq:` — supertraits sit between the name and the body
        // colon, so a `:` here is only a supertrait list when a type follows.
        let mut supertraits = Vec::new();
        if self.at_punct(Punct::Colon) && !self.at_newline_at(1) {
            self.bump();
            loop {
                supertraits.push(self.parse_type());
                if !self.eat_punct(Punct::Plus) {
                    break;
                }
            }
        }
        let where_clause = self.parse_where_clause();
        self.expect_punct(Punct::Colon);
        let members = self.parse_type_body();
        InterfaceDecl { is_unsafe: false, name, generics, supertraits, where_clause, members }
    }

    fn parse_extend(&mut self) -> ExtendDecl {
        // The contextual `extend` that `at_extend_decl` saw.
        self.bump();
        let generics = self.parse_generic_params();
        let target = self.parse_type();
        let implements = self.parse_implements();
        let where_clause = self.parse_where_clause();
        self.expect_punct(Punct::Colon);
        let members = self.parse_type_body();
        ExtendDecl { is_unsafe: false, generics, target, implements, where_clause, members, blanket: Vec::new() }
    }

    fn parse_const(&mut self) -> ConstDecl {
        self.expect_kw(Kw::Const);
        let name = self.expect_ident();
        let ty = self.eat_punct(Punct::Colon).then(|| self.parse_type());
        self.expect_punct(Punct::Eq);
        let value = self.parse_expr();
        self.expect_newline();
        ConstDecl { name, ty, value }
    }

    fn parse_static(&mut self) -> StaticDecl {
        self.expect_kw(Kw::Static);
        let is_mut = self.eat_kw(Kw::Mut);
        let name = self.expect_ident();
        self.expect_punct(Punct::Colon);
        let ty = self.parse_type();
        let value = if self.in_extern_block {
            if self.eat_punct(Punct::Eq) {
                self.report_code(codes::E0100, self.span(), "a foreign static declaration has no initializer");
                Some(self.parse_expr())
            } else {
                None
            }
        } else {
            self.expect_punct(Punct::Eq);
            Some(self.parse_expr())
        };
        self.expect_newline();
        StaticDecl { name, is_mut, is_foreign_decl: self.in_extern_block, ty, value }
    }

    fn parse_implements(&mut self) -> Vec<TypeExpr> {
        if !self.eat_kw(Kw::Implements) {
            return Vec::new();
        }
        let mut types = vec![self.parse_type()];
        while self.eat_punct(Punct::Comma) {
            types.push(self.parse_type());
        }
        types
    }

    fn parse_where_clause(&mut self) -> Vec<Bound> {
        if !self.eat_kw(Kw::Where) {
            return Vec::new();
        }
        let mut bounds = Vec::new();
        loop {
            let start = self.span();
            let subject = self.parse_type();
            self.expect_punct(Punct::Colon);
            let mut list = vec![self.parse_type()];
            while self.eat_punct(Punct::Plus) {
                list.push(self.parse_type());
            }
            bounds.push(Bound { subject, bounds: list, span: start.to(self.prev_span()) });
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        bounds
    }

    fn parse_generic_params(&mut self) -> Vec<GenericParam> {
        if !self.at_punct(Punct::LBracket) {
            return Vec::new();
        }
        self.bump();
        let mut params = Vec::new();
        while !self.at_punct(Punct::RBracket) && !self.at_eof() {
            let start = self.span();
            let id = self.next_id();
            let const_ty = if self.eat_kw(Kw::Const) {
                let name = self.expect_ident();
                self.expect_punct(Punct::Colon);
                let ty = self.parse_type();
                params.push(GenericParam {
                    id,
                    name,
                    bounds: Vec::new(),
                    default: None,
                    const_ty: Some(ty),
                    span: start.to(self.prev_span()),
                });
                if !self.eat_punct(Punct::Comma) {
                    break;
                }
                continue;
            } else {
                None
            };
            let name = self.expect_ident();
            let mut bounds = Vec::new();
            if self.eat_punct(Punct::Colon) {
                bounds.push(self.parse_type());
                while self.eat_punct(Punct::Plus) {
                    bounds.push(self.parse_type());
                }
            }
            let default = self.eat_punct(Punct::Eq).then(|| self.parse_type());
            params.push(GenericParam {
                id,
                name,
                bounds,
                default,
                const_ty,
                span: start.to(self.prev_span()),
            });
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        self.expect_punct(Punct::RBracket);
        params
    }

    // -- type bodies and members ---------------------------------------------

    fn parse_type_body(&mut self) -> Vec<Member> {
        let mut members = Vec::new();
        if !self.at_newline() {
            // `struct Marker: pass` on one line (`[STR-4]`).
            if self.eat_kw(Kw::Pass) {
                self.expect_newline();
            } else {
                let span = self.span();
                self.report_code(codes::E0004, span, "expected an indented block");
                self.recover_to_line_end();
            }
            return members;
        }
        self.bump(); // NEWLINE
        if !self.eat_indent() {
            let span = self.prev_span().shrink_to_end();
            self.report(
                Diagnostic::error(codes::E0004, span, "expected an indented block")
                    .help("write `pass` for an empty body"),
            );
            return members;
        }
        while !self.at_dedent() && !self.at_eof() {
            self.eat_newlines();
            if self.at_dedent() || self.at_eof() {
                break;
            }
            if self.eat_kw(Kw::Pass) {
                self.expect_newline();
                continue;
            }
            let before = self.pos;
            let doc = self.take_doc();
            let attrs = self.parse_attributes();
            match self.parse_member_with(doc, attrs) {
                Some(member) => members.push(member),
                None => {
                    if self.pos == before {
                        self.bump();
                    }
                    self.recover_to_line_end();
                }
            }
        }
        self.eat_dedent();
        members
    }

    fn parse_member_with(
        &mut self,
        doc: Option<String>,
        attrs: Vec<Attribute>,
    ) -> Option<Member> {
        let start = self.span();
        let (vis, read_only_outside) = self.parse_visibility();
        let id = self.next_id();

        let kind = match self.peek() {
            TokenKind::Keyword(Kw::Fn | Kw::Unsafe | Kw::Virtual | Kw::Override) => {
                MemberKind::Fn(self.parse_method())
            }
            TokenKind::Ident(_) if self.at_gen_fn() => MemberKind::Fn(self.parse_method()),
            TokenKind::Keyword(Kw::Const) => MemberKind::Const(self.parse_const()),
            TokenKind::Keyword(Kw::Type) => MemberKind::TypeAlias(self.parse_type_alias(false)),
            _ => {
                // `name: T [= default]` or `let name: T`. `let` is a keyword
                // in every position since `OQ-26` (errata ERR-004).
                let is_let = self.at_kw(Kw::Let);
                if is_let {
                    self.bump();
                }
                if !self.at_ident() {
                    let found = self.peek().to_string();
                    let span = self.span();
                    self.report(
                        Diagnostic::error(codes::E0100, span, "expected a field or method")
                            .primary_label(format!("found {found}")),
                    );
                    return None;
                }
                let name = self.expect_ident();
                self.expect_punct(Punct::Colon);
                let ty = self.parse_type();
                let default = self.eat_punct(Punct::Eq).then(|| self.parse_expr());
                self.expect_newline();
                MemberKind::Field(FieldDecl { name, ty, default, is_let })
            }
        };

        Some(Member {
            id,
            attrs,
            vis,
            read_only_outside,
            doc,
            kind,
            span: start.to(self.prev_span()),
        })
    }

    // -- types -----------------------------------------------------------------

    pub(crate) fn parse_type(&mut self) -> TypeExpr {
        if !self.enter_nesting() {
            self.leave_nesting();
            let span = self.span();
            return TypeExpr { id: self.next_id(), kind: TypeKind::Infer, span };
        }
        let ty = self.parse_type_nested();
        self.leave_nesting();
        ty
    }

    fn parse_type_nested(&mut self) -> TypeExpr {
        let start = self.span();
        let id = self.next_id();

        // `[ATT-1]`, Appendix H.2 (0.9.9) — `@latebound` was removed: every
        // callable type now gets fresh regions at each call (`[LT-7]`), which
        // is what the modifier used to ask for. Report it and read the type.
        let latebound = false;
        if self.at_punct(Punct::At)
            && matches!(self.peek_at(1), TokenKind::Ident(name) if name.is("latebound"))
        {
            let start = self.span();
            self.bump();
            self.bump();
            let span = start.to(self.prev_span());
            self.report(
                Diagnostic::error(codes::E0104, span, "`@latebound` is not an attribute")
                    .primary_label("removed in 0.9.9")
                    .note("every callable type gets fresh regions at each call, which is what `@latebound` asked for [LT-7]")
                    .help("delete `@latebound`")
                    .migrate("`@latebound` removed", span, ""),
            );
        }

        let kind = match self.peek() {
            TokenKind::Keyword(Kw::Ref) => {
                self.bump();
                let mutable = self.eat_kw(Kw::Mut);
                TypeKind::Ref { mutable, inner: Box::new(self.parse_type()) }
            }
            TokenKind::Punct(Punct::Star) => {
                self.bump();
                let mutable = self.eat_kw(Kw::Mut);
                TypeKind::Ptr { mutable, inner: Box::new(self.parse_type()) }
            }
            TokenKind::Punct(Punct::Bang) => {
                self.bump();
                TypeKind::Never
            }
            TokenKind::Keyword(Kw::Void) => {
                self.bump();
                TypeKind::Void
            }
            TokenKind::Keyword(Kw::SelfType) => {
                self.bump();
                TypeKind::SelfType
            }
            // `[GRM-31]` — `some I`: `some` before a type names one the body chooses.
            TokenKind::Ident(name) if name.is("some") && matches!(self.peek_at(1), TokenKind::Ident(_)) => {
                self.bump();
                TypeKind::SomeType(Box::new(self.parse_type()))
            }
            TokenKind::Keyword(Kw::Dyn) => {
                self.bump();
                let mut bounds = vec![self.parse_type()];
                while self.eat_punct(Punct::Plus) {
                    bounds.push(self.parse_type());
                }
                TypeKind::Dyn(bounds)
            }
            TokenKind::Punct(Punct::LParen) => {
                self.bump();
                let mut items = Vec::new();
                while !self.at_punct(Punct::RParen) && !self.at_eof() {
                    items.push(self.parse_type());
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::RParen);
                TypeKind::Tuple(items)
            }
            TokenKind::Punct(Punct::LBracket) => {
                // `[T; N]` — a fixed-size inline array.
                self.bump();
                let elem = Box::new(self.parse_type());
                self.expect_punct(Punct::Semi);
                let len = Box::new(self.parse_expr());
                self.expect_punct(Punct::RBracket);
                TypeKind::Array { elem, len }
            }
            // `[FN-6]` — `once fn(…) -> R`, a callable called at most once (`[CLO-6]`); `once`
            // is a word only here.
            TokenKind::Ident(word)
                if word.is("once") && matches!(self.peek_at(1), TokenKind::Keyword(Kw::Fn | Kw::Unsafe)) =>
            {
                self.bump();
                match self.parse_type().kind {
                    TypeKind::Fn { abi, latebound, is_unsafe, params, ret, .. } => {
                        TypeKind::Fn { abi, latebound, once: true, is_unsafe, params, ret }
                    }
                    other => other,
                }
            }
            // `[FN-6]` (ODR-115) — `unsafe fn(…) -> R`, a callable whose call is an `unsafe`
            // operation; it comes before `extern "C"`, as `[GRM-1]`'s `fn_type` orders them.
            TokenKind::Keyword(Kw::Unsafe)
                if matches!(self.peek_at(1), TokenKind::Keyword(Kw::Fn | Kw::Extern)) =>
            {
                self.bump();
                match self.parse_type().kind {
                    TypeKind::Fn { abi, latebound, once, params, ret, .. } => {
                        TypeKind::Fn { abi, latebound, once, is_unsafe: true, params, ret }
                    }
                    other => other,
                }
            }
            TokenKind::Keyword(Kw::Extern) | TokenKind::Keyword(Kw::Fn) => {
                let abi = if self.eat_kw(Kw::Extern) {
                    match self.peek().clone() {
                        TokenKind::Lit(Lit::Str(s)) => {
                            self.bump();
                            Some(s)
                        }
                        _ => Some("C".to_string()),
                    }
                } else {
                    None
                };
                self.expect_kw(Kw::Fn);
                self.expect_punct(Punct::LParen);
                let mut params = Vec::new();
                while !self.at_punct(Punct::RParen) && !self.at_eof() {
                    let param_start = self.span();
                    let mode = if self.eat_kw(Kw::Mut) {
                        Mode::Mut
                    } else if self.eat_kw(Kw::Owned) {
                        Mode::Owned
                    } else {
                        Mode::Borrow
                    };
                    let ty = self.parse_type();
                    params.push(CallableTypeParam {
                        mode,
                        span: param_start.to(ty.span),
                        ty,
                    });
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::RParen);
                let ret = self.eat_punct(Punct::Arrow).then(|| Box::new(self.parse_type()));
                TypeKind::Fn { abi, latebound, once: false, is_unsafe: false, params, ret }
            }
            _ => {
                let segments = self.parse_dotted_path();
                let args = self.parse_generic_args();
                TypeKind::Path { segments, args }
            }
        };

        TypeExpr { id, kind, span: start.to(self.prev_span()) }
    }

    /// `[T]`, `[T, U]`, `[Item = i32]`, `[N]` for a const argument.
    fn parse_generic_args(&mut self) -> Vec<GenericArg> {
        if !self.at_punct(Punct::LBracket) {
            return Vec::new();
        }
        self.bump();
        let mut args = Vec::new();
        while !self.at_punct(Punct::RBracket) && !self.at_eof() {
            if self.at_ident() && self.at_punct_at(1, Punct::Eq) {
                let name = self.expect_ident();
                self.bump(); // '='
                args.push(GenericArg::Assoc { name, ty: self.parse_type() });
            } else if matches!(self.peek(), TokenKind::Lit(_)) {
                args.push(GenericArg::Const(self.parse_expr()));
            } else {
                args.push(GenericArg::Type(self.parse_type()));
            }
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        self.expect_punct(Punct::RBracket);
        args
    }

    // -- structural token helpers ---------------------------------------------

    pub(crate) fn at_dedent(&self) -> bool {
        matches!(self.peek(), TokenKind::Dedent)
    }

    pub(crate) fn at_newline_at(&self, n: usize) -> bool {
        matches!(self.peek_at(n), TokenKind::Newline)
    }

    pub(crate) fn eat_indent(&mut self) -> bool {
        if matches!(self.peek(), TokenKind::Indent) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub(crate) fn eat_dedent(&mut self) -> bool {
        if self.at_dedent() {
            self.bump();
            true
        } else {
            false
        }
    }
}
