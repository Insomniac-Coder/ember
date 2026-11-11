//! Items, members, generics and type expressions (Part III §§1–3).

use ember_ast::*;
use ember_diag::{Diagnostic, codes};
use ember_lexer::{Kw, Lit, Punct, TokenKind};

use crate::{Parser, TYPE_PARAMS_LINE};

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
            self.slot_ending_with(&[Kw::Import]);
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

        self.slot_ending_with(&[Kw::As]);
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
            self.slot_ending_with(&[Kw::As]);
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

    /// `[UNS-8]` — the `unsafe` here begins an implementation (`unsafe extern`, `unsafe overlay`).
    fn push_unsafe_implementation(&mut self) {
        let span = self.span();
        self.unsafe_sites.push(ember_ast::UnsafeSite { span, kind: ember_ast::UnsafeSiteKind::Implementation });
    }

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
        self.safety_attribute = attrs.iter().any(|attr| attr.path.len() == 1 && attr.path[0].name.is("safety"));
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
            TokenKind::Punct(Punct::LBracket) => self.at_type_params_line(),
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
            // `safe unsafe fn` is parsed whole, so that its refusal names it (`E5002`, verdict 133).
            TokenKind::Ident(s) if s.is("safe") => self.at_fn_at(1) || (self.at_kw_at(1, Kw::Unsafe) && self.at_fn_at(2)),
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
        if self.refuse_bracket_line() {
            return None;
        }
        self.parse_type_params_line();
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
            self.push_unsafe_implementation();
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
            let site = self.span();
            self.bump();
            if self.at_extend_decl() {
                self.unsafe_sites.push(ember_ast::UnsafeSite { span: site, kind: ember_ast::UnsafeSiteKind::Implementation });
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
            self.push_unsafe_implementation();
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
            // `static fn` outside a type body, refused by `parse_fn_decl` (`E2115`).
            TokenKind::Keyword(Kw::Static) if self.at_static_fn() => Some(ItemKind::Fn(self.parse_fn())),
            // `[GRM-21]` — a `gen fn` is admitted wherever `fn_decl` is.
            TokenKind::Ident(_) if self.at_gen_fn() => Some(ItemKind::Fn(self.parse_fn())),
            TokenKind::Ident(s) if s.is("safe") && (self.at_fn_at(1) || (self.at_kw_at(1, Kw::Unsafe) && self.at_fn_at(2))) => {
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
            bounds = self.parse_needs();
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

    /// `[GRM-35]` — `overlay ("c" | "cpp") STRING ":" NEWLINE INDENT {overlay_item} DEDENT`, after any
    /// `unsafe`. The header an overlay was written for is recorded by Ember, never in it (`[FFI-55]`).
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
        // `[FFI-55]` (verdict 121) — 0.9.15_Hardened_3's `with (header_hash = …)` is gone.
        if self.at_kw(Kw::With) {
            let at = self.span();
            self.report_code(
                codes::E0104,
                at,
                "an overlay takes no options: Ember records the header it was written for in `ember.overlays`, beside the package's manifest [FFI-55]",
            );
            self.bump();
            if self.eat_punct(Punct::LParen) {
                while !self.at_punct(Punct::RParen) && !self.at_eof() && !self.at_newline() {
                    self.bump();
                }
                self.eat_punct(Punct::RParen);
            }
        }
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
                self.parse_overlay_fn(false)
            }
            // `[GRM-35]` (verdict 130) — an entry of a function unsafe to call.
            TokenKind::Keyword(Kw::Unsafe) if matches!(self.peek_at(1), TokenKind::Keyword(Kw::Fn)) => {
                self.bump();
                self.bump();
                self.parse_overlay_fn(true)
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
                self.slot_ending_with(&[Kw::As]);
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
            // `[GRM-35]` (verdict 131) — `class C:` and the entries of its members.
            TokenKind::Keyword(Kw::Class) => {
                self.bump();
                let name = self.expect_ident();
                self.expect_punct(Punct::Colon);
                let mut members = Vec::new();
                if self.at_newline() {
                    self.bump();
                    if self.eat_indent() {
                        while !self.at_dedent() && !self.at_eof() {
                            self.eat_newlines();
                            if self.at_dedent() || self.at_eof() {
                                break;
                            }
                            let before = self.pos;
                            let Some(member) = self.parse_overlay_item() else {
                                // A line that begins no item: past it, as the overlay's own loop does.
                                if self.pos == before {
                                    self.bump();
                                }
                                self.recover_to_line_end();
                                continue;
                            };
                            if matches!(member.kind, OverlayItemKind::Fn { .. }) {
                                members.push(member);
                            } else {
                                self.report(
                                    Diagnostic::error(codes::E0100, member.span, format!("a `class` section holds the entries of `{}`'s members", name.name))
                                        .help("write each member as an entry, `fn name(…)` or `unsafe fn name(…)`; other items go outside the section [GRM-35]"),
                                );
                            }
                        }
                        self.eat_dedent();
                    }
                } else {
                    self.expect_newline();
                }
                OverlayItemKind::Class { name, members }
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
    /// `overlay_fn`, after its `fn` (and its `unsafe`): the name, each parameter with its contract
    /// words, and the result's.
    fn parse_overlay_fn(&mut self, is_unsafe: bool) -> OverlayItemKind {
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
        OverlayItemKind::Fn { name, params, result, is_unsafe }
    }

    fn parse_contract_words(&mut self) -> Vec<Contract> {
        let mut words = Vec::new();
        loop {
            let word = match self.peek() {
                TokenKind::Ident(_) | TokenKind::RawIdent(_) => self.expect_ident(),
                // `unsafe` and `owns` (0.9.10's `owned`) are keywords elsewhere and contract words here
                // (`[FFI-11]`).
                &TokenKind::Keyword(keyword @ (Kw::Unsafe | Kw::Owns | Kw::Owned)) => {
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

    pub(crate) fn at_gen_fn_at(&self, n: usize) -> bool {
        matches!(self.peek_at(n), TokenKind::Ident(s) if s.is("gen"))
            && matches!(self.peek_at(n + 1), TokenKind::Keyword(Kw::Fn))
    }

    /// `fn`, or `gen fn`, `n` tokens ahead: `fn_header`'s last two words.
    pub(crate) fn at_fn_at(&self, n: usize) -> bool {
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
    /// `static fn` or `static gen fn`.
    pub(crate) fn at_static_fn(&self) -> bool {
        self.at_kw(Kw::Static) && (self.at_kw_at(1, Kw::Fn) || self.at_gen_fn_at(1))
    }

    pub(crate) fn parse_fn(&mut self) -> FnDecl {
        self.parse_fn_decl(false)
    }

    /// A method (a member of a type, an `extend` block or an interface): its name may also be
    /// `not` or `is` ([LEX-15]).
    fn parse_method(&mut self) -> FnDecl {
        self.parse_fn_decl(true)
    }

    fn parse_fn_decl(&mut self, method: bool) -> FnDecl {
        // Taken now: its parameters' attributes are parsed before its site is recorded.
        let safety_attribute = std::mem::take(&mut self.safety_attribute);
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
        // `[GRM-41]`, `[FN-4]` — `static fn`: a function of a type body with no object, called on the type.
        let static_span = self.span();
        let is_static = self.at_static_fn();
        if is_static {
            self.bump();
            if !method {
                self.report(
                    Diagnostic::error(codes::E2115, static_span, "`static fn` is a function of a type body")
                        .primary_label("not in a type body")
                        .help("remove `static`: a function outside a type body has no object"),
                );
            }
        }
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
        let generics = self.parse_declared_generics();

        let lparen = self.span();
        self.expect_punct(Punct::LParen);
        let mut params = Vec::new();
        while !self.at_punct(Punct::RParen) && !self.at_eof() {
            params.push(self.parse_param());
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        self.expect_punct(Punct::RParen);

        // `[GRM-41]`, `[FN-4]` — `owns this` after the parameters: the method takes its object over.
        let owns_this_span = self.span();
        let owns_this = self.at_kw(Kw::Owns) && self.at_kw_at(1, Kw::This);
        if owns_this {
            self.bump();
            self.bump();
        }
        let explicit_receiver = params.iter().any(|param| matches!(param.kind, ParamKind::Receiver { .. }));
        if is_static && explicit_receiver {
            let span = params[0].span;
            self.report(
                Diagnostic::error(codes::E1010, span, format!("`{}` is a `static fn`: it has no object", name.name))
                    .help("remove the object from its parameters, or `static` to make it a method"),
            );
        }
        if owns_this && (!method || is_static || explicit_receiver) {
            let span = owns_this_span.to(self.prev_span());
            let why = if !method || is_static { "has no object" } else { "takes its object in its parameters" };
            self.report(Diagnostic::error(codes::E1010, span, format!("`owns this` on a function that {why}")).help("remove `owns this`"));
        }
        // A method that does not list its object: it is `this`, its mode worked out from the body
        // (`[EXC-15]`). 0.9.10 read such a function as one with no object; M1's migration writes
        // `static` before those (`M1-plan.md`).
        if method && !is_static && !explicit_receiver {
            let id = self.next_id();
            let mode = if owns_this { Mode::Owned } else { Mode::Mut };
            let kind = ParamKind::Receiver { ty: None, implicit: true };
            params.insert(0, Param { id, mode, is_const: false, kind, default: None, span: name.span });
        }

        let ret = self.eat_punct(Punct::Arrow).then(|| self.parse_type());
        // `[GRM-41]` — `from` and what the result points into come after its type.
        let from = (ret.is_some() && self.at_contextual("from")).then(|| {
            self.bump();
            self.expect_ident()
        });
        let where_clause = self.parse_where_clause();

        // A body is present unless this is an interface or extern signature.
        // `[UNS-7]`, `[UNS-8]` — an `unsafe fn` with one says what its callers vouch for with `@safety`.
        if is_unsafe && self.at_punct(Punct::Colon) && !self.in_extern_block {
            let kind = ember_ast::UnsafeSiteKind::Fn { name: name.name, safety: safety_attribute };
            self.unsafe_sites.push(ember_ast::UnsafeSite { span: unsafe_site, kind });
        }
        let body_start = self.pos;
        let body = if self.eat_punct(Punct::Colon) { Some(self.parse_block()) } else { None };
        // `[FN-4]`, `[LEX-15]` — `self` anywhere is `E1010`: a function standing alone has no object,
        // and a `self` it lists with a type is a parameter of that name, checked as written.
        if !method {
            for param in &mut params {
                if let ParamKind::Receiver { ty, implicit: false } = &param.kind {
                    let ty = ty.clone();
                    self.report(Diagnostic::error(codes::E1010, param.span, "inside a method its object is `this`"));
                    if let Some(ty) = ty {
                        param.kind = ParamKind::Named { name: Ident { name: ember_span::Symbol::intern("self"), span: param.span }, ty };
                    }
                }
            }
        }
        // `[DIA-21]` P14 — `self` in a method, listed or used; an interface's method lists it too.
        if method {
            let body_end = self.pos;
            self.python_missing_self(lparen, &mut params, body_start..body_end);
        }
        if body.is_none() {
            if self.tokens[signature_start..=self.pos.min(self.tokens.len() - 1)]
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Error))
            {
                self.damaged_signatures.push(name.span);
            }
            self.expect_newline();
        }

        FnDecl { name, is_unsafe, is_safe, is_foreign_decl: false, written_unsafe: is_unsafe, abi, is_gen, dispatch, generics, params, ret, from, where_clause, body }
    }

    fn parse_param(&mut self) -> Param {
        // `[DIA-21]` P19 — `*args`, `**kwargs`.
        self.python_star_param();
        let start = self.span();
        let id = self.next_id();
        let before_word = self.pos;
        // `[LEX-14]` — `ref: int` names the parameter with a keyword: no word before the name.
        let (mode, is_const) = if self.at_keyword_written_as_name(Punct::Colon) { (Mode::Borrow, false) } else { self.parse_param_word(false) };
        let worded = self.pos != before_word;

        // `[FN-4]` — the receiver follows the same modes as any parameter.
        if self.at_kw(Kw::SelfValue) {
            self.bump();
            let ty = self.eat_punct(Punct::Colon).then(|| self.parse_type());
            return Param {
                id,
                mode,
                is_const,
                kind: ParamKind::Receiver { ty, implicit: false },
                default: None,
                span: start.to(self.prev_span()),
            };
        }

        let name = self.expect_ident();
        self.expect_punct(Punct::Colon);
        let mode = if !worded && self.refuse_mode_in_type(name) { Mode::Mut } else { mode };
        // `[LEX-15]`, `[LT-48]` — `runtime` is a keyword only after the `:` and directly before `dyn`.
        if self.at_contextual("runtime") && self.at_kw_at(1, Kw::Dyn) {
            self.bump();
            self.runtime_params.push(id);
        }
        let ty = self.parse_type();
        let default = self.eat_punct(Punct::Eq).then(|| self.parse_expr());
        // `[CLO-6]` — calling a `once fn` parameter consumes it, so it is taken over as an
        // `owned` one is; `owned` is implied and need not be written.
        let mode = if matches!(ty.kind, TypeKind::Fn { once: true, .. }) { Mode::Owned } else { mode };
        Param {
            id,
            mode,
            is_const,
            kind: ParamKind::Named { name, ty },
            default,
            span: start.to(self.prev_span()),
        }
    }

    /// `[FN-1]` — the word before a parameter: `ref` (reads, the default, which may be left out),
    /// `ref mut`, `const ref mut` or `owns`, and whether it is `const ref mut`. 0.9.10's `mut` and
    /// `owned` are read too until M1's migration has respelled every use (`M1-plan.md`, step 4).
    /// In a callable type (`in_type`) a `ref` without `mut` is left to the type: `fn(ref K)` keeps
    /// 0.9.10's reference type, which reads as the word does.
    /// `[GRM-41]` — `c: ref mut Counter`, the mode written in the type, after `name:`: `E0117`, the
    /// line with the mode on the name. The type is read as written, so the parameter checks as it meant.
    /// In a foreign declaration, `x: ref mut T` is the pointer C takes, which its contract describes
    /// (`[FFI-11]`): a type, not a mode.
    /// Refused, the words are taken and the parameter is read as the line written right is (`ref mut
    /// c: Counter`), so what is checked after is what was meant: whether it was.
    /// Called for a name with no word before it: `ref mut x: ref mut T` is a parameter of a view type.
    pub(crate) fn refuse_mode_in_type(&mut self, name: Ident) -> bool {
        if !(self.at_kw(Kw::Ref) && self.at_kw_at(1, Kw::Mut)) || self.in_extern_block {
            return false;
        }
        let words = self.span().to(self.tokens[self.pos + 1].span);
        let written = ember_span::Span::new(name.span.file, name.span.start, self.tokens[self.pos + 2].span.start);
        let line = self.line_with(written, &format!("ref mut {}: ", name.name));
        self.report(
            Diagnostic::error(codes::E0117, words, "the mode goes on the name, not in its type")
                .help(format!("write `{line}`")),
        );
        self.bump();
        self.bump();
        true
    }

    pub(crate) fn parse_param_word(&mut self, in_type: bool) -> (Mode, bool) {
        if self.at_kw(Kw::Const) && self.at_kw_at(1, Kw::Ref) && self.at_kw_at(2, Kw::Mut) {
            self.bump();
            self.bump();
            self.bump();
            return (Mode::Mut, true);
        }
        if self.at_kw(Kw::Ref) && self.at_kw_at(1, Kw::Mut) {
            self.bump();
            self.bump();
            return (Mode::Mut, false);
        }
        if !in_type && self.eat_kw(Kw::Ref) {
            return (Mode::Borrow, false);
        }
        if self.eat_kw(Kw::Owns) {
            return (Mode::Owned, false);
        }
        // `[MOD-3]` — 0.9.10's `owned` is `owns`; `owned: T` is the word written as a name
        // (`[LEX-15]`), which `expect_ident` takes.
        if self.at_kw(Kw::Owned) && !self.at_punct_at(1, Punct::Colon) {
            let span = self.span();
            self.bump();
            self.report(
                Diagnostic::error(codes::E1010, span, "`owned` is gone from Ember: the word that hands a value over is `owns`")
                    .help(format!("write `{}`", self.line_with(span, "owns"))),
            );
            return (Mode::Owned, false);
        }
        // `[GRM-41]` — `mut` alone says nothing of what is passed: `ref mut`.
        if self.at_kw(Kw::Mut) {
            let span = self.span();
            self.bump();
            if !self.at_kw(Kw::SelfValue) {
                self.report(
                    Diagnostic::error(codes::E0113, span, "`mut` alone is no word for a parameter: one the function may change is `ref mut`")
                        .help(format!("write `{}`", self.line_with(span, "ref mut"))),
                );
            }
            return (Mode::Mut, false);
        }
        (Mode::Borrow, false)
    }

    fn parse_struct(&mut self) -> StructDecl {
        self.expect_kw(Kw::Struct);
        self.slot_ending_with(&[Kw::Implements, Kw::Where]);
        let name = self.expect_ident();
        let generics = self.parse_declared_generics();
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
        self.slot_ending_with(&[Kw::Implements, Kw::Where]);
        let name = self.expect_ident();
        let generics = self.parse_declared_generics();
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
        self.slot_ending_with(&[Kw::Implements, Kw::Where]);
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
        self.slot_ending_with(&[Kw::Where]);
        let name = self.expect_ident();
        let generics = self.parse_generic_params();
        // `interface Ord: Eq:` — supertraits sit between the name and the body
        // colon, so a `:` here is only a supertrait list when a type follows.
        let mut supertraits = Vec::new();
        if self.at_punct(Punct::Colon) && !self.at_newline_at(1) {
            self.bump();
            supertraits = self.parse_needs();
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
            let list = self.parse_needs();
            bounds.push(Bound { subject, bounds: list, span: start.to(self.prev_span()) });
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        bounds
    }

    /// `[GRM-40]` — whether this line holds type parameters: it begins with `[`, and the line after it
    /// begins a function, struct or class declaration, with `pub`, `open`, `abstract`, `static`,
    /// `virtual`, `override`, `gen`, `unsafe` or `extern` before it, or none. The one place the
    /// parser looks past a line's end.
    pub(crate) fn at_type_params_line(&self) -> bool {
        if !self.at_punct(Punct::LBracket) {
            return false;
        }
        let mut depth = 0usize;
        let mut n = 0;
        loop {
            match self.peek_at(n) {
                TokenKind::Punct(p) if p.is_open_bracket() => depth += 1,
                TokenKind::Punct(p) if p.is_close_bracket() => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => return false,
                _ => {}
            }
            n += 1;
        }
        if !matches!(self.peek_at(n + 1), TokenKind::Newline) {
            return false;
        }
        n += 2;
        loop {
            match self.peek_at(n) {
                TokenKind::Keyword(Kw::Fn | Kw::Struct | Kw::Class) => return true,
                TokenKind::Keyword(Kw::Static | Kw::Virtual | Kw::Override | Kw::Unsafe) => n += 1,
                TokenKind::Keyword(Kw::Pub) => {
                    n += 1;
                    // `pub(package)`, `pub(package, read)`
                    if matches!(self.peek_at(n), TokenKind::Punct(Punct::LParen)) {
                        while !matches!(self.peek_at(n), TokenKind::Punct(Punct::RParen) | TokenKind::Newline | TokenKind::Eof) {
                            n += 1;
                        }
                        n += 1;
                    }
                }
                TokenKind::Keyword(Kw::Extern) => {
                    n += 1;
                    if matches!(self.peek_at(n), TokenKind::Lit(ember_lexer::Lit::Str(_))) {
                        n += 1;
                    }
                }
                TokenKind::Ident(s) if s.is("open") || s.is("abstract") || s.is("gen") || s.is("safe") => n += 1,
                _ => return false,
            }
        }
    }

    /// `[GRM-40]` — reads a type-parameter line, if this is one, for the declaration under it.
    /// A blank or comment line between it and the declaration is `E0100`; the parameters are still
    /// the declaration's, so what follows checks as meant.
    pub(crate) fn parse_type_params_line(&mut self) {
        if !self.at_type_params_line() {
            return;
        }
        let (start, first) = (self.span(), self.pos);
        let params = self.parse_generic_params();
        let span = start.to(self.prev_span());
        self.expect_newline();
        // From the end of the line's last token to the declaration: one line end, or a blank or
        // comment line is between them. A line holding a token the lexer reported is that
        // error's (`[DIA-14]`).
        let line = &self.tokens[first..self.pos];
        let lexed_wrong = line.iter().any(|token| matches!(token.kind, TokenKind::Error));
        let end = line.iter().rev().find(|token| !matches!(token.kind, TokenKind::Newline)).map_or(span.end, |token| token.span.end);
        let between = self.src.get(end as usize..self.span().start as usize).unwrap_or("");
        if !lexed_wrong && between.matches('\n').count() > 1 {
            self.report(Diagnostic::error(codes::E0100, span, TYPE_PARAMS_LINE));
        }
        self.type_params_line = Some((params, span));
    }

    /// `[GRM-40]` — a line that begins with `[` and is not a type-parameter line (`[T]` above an
    /// `enum` or an attribute, two `[T]` lines in a row, `[10, 20].len()` in a body): `E0100`, and
    /// the line is skipped up to its end. Above an enum, an interface or a `type` alias the help
    /// gives the brackets after the name, as the program writes both.
    pub(crate) fn refuse_bracket_line(&mut self) -> bool {
        if !self.at_punct(Punct::LBracket) || self.at_type_params_line() {
            return false;
        }
        let help = self.brackets_after_name();
        let start = self.span();
        // `[DIA-14]` — a line holding a token the lexer reported is that error's.
        let mut lexed_wrong = false;
        while !matches!(self.peek(), TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof) {
            lexed_wrong |= matches!(self.peek(), TokenKind::Error);
            self.bump();
        }
        if lexed_wrong {
            return true;
        }
        let span = start.to(self.prev_span());
        let mut diagnostic = Diagnostic::error(codes::E0100, span, TYPE_PARAMS_LINE);
        if let Some(help) = help {
            diagnostic = diagnostic.help(help);
        }
        self.report(diagnostic);
        true
    }

    /// The help for brackets on the line above an `enum`, `interface` or `type` alias: the line
    /// below with the brackets after its name (`enum E[T]:`).
    fn brackets_after_name(&self) -> Option<String> {
        let mut depth = 0usize;
        let mut n = 0;
        loop {
            match self.peek_at(n) {
                TokenKind::Punct(p) if p.is_open_bracket() => depth += 1,
                TokenKind::Punct(p) if p.is_close_bracket() => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof => return None,
                _ => {}
            }
            n += 1;
        }
        let close = self.tokens[self.pos + n].span;
        if !matches!(self.peek_at(n + 1), TokenKind::Newline) {
            return None;
        }
        n += 2;
        while matches!(self.peek_at(n), TokenKind::Keyword(Kw::Pub | Kw::Unsafe)) {
            n += 1;
            if matches!(self.peek_at(n), TokenKind::Punct(Punct::LParen)) {
                while !matches!(self.peek_at(n), TokenKind::Punct(Punct::RParen) | TokenKind::Newline | TokenKind::Eof) {
                    n += 1;
                }
                n += 1;
            }
        }
        let brackets = self.src.get(self.span().start as usize..close.end as usize)?;
        // `extend[T] Shape:` — an `extend` block's brackets go after `extend`.
        if matches!(self.peek_at(n), TokenKind::Ident(word) if word.is("extend")) {
            let word = self.tokens[self.pos + n].span;
            return Some(format!("write the brackets after `extend`: `{}`", self.line_with(word, &format!("extend{brackets}"))));
        }
        if !matches!(self.peek_at(n), TokenKind::Keyword(Kw::Enum | Kw::Interface | Kw::Type))
            || !matches!(self.peek_at(n + 1), TokenKind::Ident(_) | TokenKind::RawIdent(_))
        {
            return None;
        }
        let name = self.tokens[self.pos + n + 1].span;
        let written = self.src.get(name.start as usize..name.end as usize)?;
        Some(format!("write the brackets after the name: `{}`", self.line_with(name, &format!("{written}{brackets}"))))
    }

    /// The type parameters of a function, struct or class: those of the line above it (`[GRM-40]`).
    /// 0.9.10's brackets after its name are `E0100`, its help the two lines to write, and read so the
    /// declaration checks as it meant.
    fn parse_declared_generics(&mut self) -> Vec<GenericParam> {
        let open = self.span();
        let outer = std::mem::replace(&mut self.misplaced_generics, true);
        let after_name = self.parse_generic_params();
        self.misplaced_generics = outer;
        let Some((line, span)) = self.type_params_line.take() else {
            if !after_name.is_empty() {
                let brackets = open.to(self.prev_span());
                let written = self.type_params_written(&after_name);
                let declaration = self.line_with(brackets, "");
                self.report(
                    Diagnostic::error(codes::E0100, brackets, "type parameters go on a line of their own, above the declaration")
                        .help(format!("write `{written}` on the line above `{declaration}`"))
                        .note("a function's, struct's or class's type parameters are not written after its name [GRM-40]"),
                );
            }
            return after_name;
        };
        if !after_name.is_empty() {
            self.report(
                Diagnostic::error(codes::E0100, span, "type parameters are written twice")
                    .help("keep the line above the declaration and remove the brackets after its name"),
            );
        }
        line
    }

    /// `[GRM-40]`, `[TYP-17]` — type parameters as their line is written: `[T: needs (A, B) = D]`.
    fn type_params_written(&self, params: &[GenericParam]) -> String {
        let text = |span: ember_span::Span| self.src.get(span.start as usize..span.end as usize).unwrap_or("T").to_string();
        let written: Vec<String> = params
            .iter()
            .map(|param| {
                if let Some(ty) = &param.const_ty {
                    return format!("const {}: {}", param.name.name, text(ty.span));
                }
                let mut out = param.name.name.to_string();
                let bounds: Vec<String> = param.bounds.iter().map(|bound| text(bound.span)).collect();
                match bounds.as_slice() {
                    [] => {}
                    // A tuple is one bound: `needs ((int, int))`.
                    [one] if !one.starts_with('(') => out.push_str(&format!(": needs {one}")),
                    _ => out.push_str(&format!(": needs ({})", bounds.join(", "))),
                }
                if let Some(default) = &param.default {
                    out.push_str(&format!(" = {}", text(default.span)));
                }
                out
            })
            .collect();
        format!("[{}]", written.join(", "))
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
                bounds = self.parse_needs();
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
        // ODR-281 — at their count, as a block's statements are (`parse_block_nested`).
        members.shrink_to_fit();
        members
    }

    fn parse_member_with(
        &mut self,
        doc: Option<String>,
        attrs: Vec<Attribute>,
    ) -> Option<Member> {
        if self.refuse_bracket_line() {
            return None;
        }
        self.parse_type_params_line();
        // `[DIA-21]` P2 — `def m(self):` in a type's body is a method.
        self.python_def();
        let start = self.span();
        let (vis, read_only_outside) = self.parse_visibility();
        let id = self.next_id();

        // `[LEX-14]` — `class: int`, `type: int`: a field named with a keyword, refused as a name.
        let keyword_field = self.at_keyword_written_as_name(Punct::Colon);
        let kind = match self.peek() {
            TokenKind::Keyword(Kw::Fn | Kw::Unsafe | Kw::Virtual | Kw::Override) if !keyword_field => {
                MemberKind::Fn(self.parse_method())
            }
            TokenKind::Keyword(Kw::Static) if self.at_static_fn() => MemberKind::Fn(self.parse_method()),
            TokenKind::Ident(_) if self.at_gen_fn() => MemberKind::Fn(self.parse_method()),
            TokenKind::Keyword(Kw::Const) if !keyword_field => MemberKind::Const(self.parse_const()),
            TokenKind::Keyword(Kw::Type) if !keyword_field => MemberKind::TypeAlias(self.parse_type_alias(false)),
            _ => {
                // `name: T [= default]` or `let name: T`. `let` is a keyword
                // in every position since `OQ-26` (errata ERR-004).
                let is_let = self.at_kw(Kw::Let) && !keyword_field;
                if is_let {
                    self.bump();
                }
                if !self.at_name() && !self.at_keyword_written_as_name(Punct::Colon) {
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
        let suffix = !std::mem::take(&mut self.no_send_suffix);
        if !self.enter_nesting() {
            self.leave_nesting();
            let span = self.span();
            return TypeExpr { id: self.next_id(), kind: TypeKind::Infer, span };
        }
        let ty = self.parse_type_nested(suffix);
        self.leave_nesting();
        ty
    }

    /// What a type parameter, an interface, an associated type or a `where` subject needs, after
    /// its `:` (`[TYP-17]`): `needs B`, or several in round brackets, `needs (A, B)`. 0.9.10's `A + B`
    /// is `E0100`, its help the bound as written after `needs`, and read so the bound checks as meant.
    fn parse_needs(&mut self) -> Vec<TypeExpr> {
        if !self.at_contextual("needs") {
            let mut bounds = vec![self.parse_bound()];
            while self.eat_punct(Punct::Plus) {
                bounds.push(self.parse_bound());
            }
            let span = bounds[0].span.to(self.prev_span());
            // Inside brackets after a name, the one refusal of those writes the whole line.
            if self.misplaced_generics {
                return bounds;
            }
            let texts: Vec<&str> = bounds.iter().map(|bound| self.src.get(bound.span.start as usize..bound.span.end as usize).unwrap_or("B")).collect();
            let needs = if texts.len() == 1 { format!("needs {}", texts[0]) } else { format!("needs ({})", texts.join(", ")) };
            self.report(
                Diagnostic::error(codes::E0100, span, "what a type needs is written after `needs`")
                    .help(format!("write `{needs}`"))
                    .note("several are listed in round brackets: `T: needs (Add, Default)` [TYP-17]"),
            );
            return bounds;
        }
        self.bump();
        if !self.eat_punct(Punct::LParen) {
            return vec![self.parse_bound()];
        }
        let mut bounds = Vec::new();
        while !self.at_punct(Punct::RParen) && !self.at_eof() {
            bounds.push(self.parse_bound());
            if !self.eat_punct(Punct::Comma) {
                break;
            }
        }
        self.expect_punct(Punct::RParen);
        bounds
    }

    /// One bound of a list (`T: A + B`, `dyn A + B`), where `+` separates bounds: `F: fn() +
    /// Send` bounds `F` by the callable type and by `Send`, as it always has.
    fn parse_bound(&mut self) -> TypeExpr {
        self.no_send_suffix = true;
        self.parse_type()
    }

    /// `suffix`: whether a callable type here may end in `+ Send` (ODR-264); a prefix (`ref`,
    /// `once`) passes it on to the type it prefixes.
    fn parse_type_nested(&mut self, suffix: bool) -> TypeExpr {
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

        // `[TYP-45]`, `[HEAP-9]` — `owns T`, the grammar's `type := "owns" base_type | ...`.
        if self.at_kw(Kw::Owns) {
            self.bump();
            let inner = self.parse_type_nested(suffix);
            return TypeExpr { id, kind: TypeKind::Owns(Box::new(inner)), span: start.to(self.prev_span()) };
        }
        // `[TYP-45]`, `[LEX-14]` — the other words before a type that say who owns the value
        // (`weakref Node`, `shares mut Game`): never a name, and not built by this compiler yet (M4);
        // the type after it is read as written.
        if let TokenKind::Keyword(word @ (Kw::Shares | Kw::ThreadShared | Kw::Weakref | Kw::Sharedref)) = self.peek().clone() {
            let at = self.span();
            self.bump();
            if matches!(word, Kw::Shares | Kw::Sharedref) {
                self.eat_kw(Kw::Mut);
            }
            let written = self.src.get(at.start as usize..self.prev_span().end as usize).unwrap_or("owns").to_string();
            self.report(Diagnostic::error(codes::E0900, at.to(self.prev_span()), format!("`{written}` before a type is not built by this compiler yet")));
            return self.parse_type_nested(suffix);
        }

        let kind = match self.peek() {
            TokenKind::Keyword(Kw::Ref) => {
                self.bump();
                let mutable = self.eat_kw(Kw::Mut);
                self.no_send_suffix = !suffix;
                TypeKind::Ref { mutable, inner: Box::new(self.parse_type()) }
            }
            TokenKind::Punct(Punct::Star) => {
                self.bump();
                let mutable = self.eat_kw(Kw::Mut);
                self.no_send_suffix = !suffix;
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
                let mut bounds = vec![self.parse_bound()];
                while self.eat_punct(Punct::Plus) {
                    bounds.push(self.parse_bound());
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
                self.no_send_suffix = !suffix;
                match self.parse_type().kind {
                    TypeKind::Fn { abi, latebound, is_unsafe, params, ret, send, .. } => {
                        TypeKind::Fn { abi, latebound, once: true, is_unsafe, params, ret, send }
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
                self.no_send_suffix = !suffix;
                match self.parse_type().kind {
                    TypeKind::Fn { abi, latebound, once, params, ret, send, .. } => {
                        TypeKind::Fn { abi, latebound, once, is_unsafe: true, params, ret, send }
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
                    let (mode, is_const) = self.parse_param_word(true);
                    let ty = self.parse_type();
                    params.push(CallableTypeParam {
                        mode,
                        is_const,
                        span: param_start.to(ty.span),
                        ty,
                    });
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::RParen);
                let ret = self.eat_punct(Punct::Arrow).then(|| {
                    self.no_send_suffix = true;
                    Box::new(self.parse_type())
                });
                let send = suffix && self.parse_send_suffix();
                TypeKind::Fn { abi, latebound, once: false, is_unsafe: false, params, ret, send }
            }
            _ => {
                let segments = self.parse_dotted_path();
                let args = self.parse_generic_args();
                TypeKind::Path { segments, args }
            }
        };

        TypeExpr { id, kind, span: start.to(self.prev_span()) }
    }

    /// `[CLO-10]`, `[THR-8]` (ODR-264) — `+ Send` after a callable type: its values are `Send`.
    /// After a return type it is the callable's whose return type it ends (`fn() -> fn() + Send`
    /// is a `Send` callable giving a `fn()`), as in a bound (`F: fn() -> fn() + Send`). A callable
    /// value is never `Sync`.
    fn parse_send_suffix(&mut self) -> bool {
        if !self.at_punct(Punct::Plus) {
            return false;
        }
        let word = match self.peek_at(1) {
            TokenKind::Ident(word) if word.is("Send") || word.is("Sync") => *word,
            _ => return false,
        };
        let start = self.span();
        self.bump();
        self.bump();
        if word.is("Sync") {
            let span = start.to(self.prev_span());
            self.report(
                Diagnostic::error(codes::E7005, span, "a callable type is never `Sync`")
                    .primary_label("written here")
                    .note("a call may change what the value captured, so two threads may not call one value at once [CLO-10], [THR-9]")
                    .help("delete `+ Sync`; `+ Send` lets the value move to another thread"),
            );
            return false;
        }
        true
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
