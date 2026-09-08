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
            self.expect_newline();
            return Some(Import {
                id,
                kind: ImportKind::Foreign { language, header, options },
                span: start.to(self.prev_span()),
            });
        }

        let path = self.parse_dotted_path();
        let alias = self.eat_kw(Kw::As).then(|| self.expect_ident());
        self.expect_newline();
        Some(Import {
            id,
            kind: ImportKind::Module { path, alias },
            span: start.to(self.prev_span()),
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
        while self.at_punct(Punct::Dot) && matches!(self.peek_at(1), TokenKind::Ident(_)) {
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
        Some(Item { id, attrs, vis, doc, kind, span: start.to(self.prev_span()) })
    }

    fn parse_item_kind(&mut self) -> Option<ItemKind> {
        // `unsafe extern "C":` and `unsafe fn`.
        if self.at_kw(Kw::Unsafe) && self.at_kw_at(1, Kw::Extern) {
            self.bump();
            return Some(ItemKind::ExternBlock(self.parse_extern_block(true)));
        }
        if self.at_kw(Kw::Extern) {
            return Some(ItemKind::ExternBlock(self.parse_extern_block(false)));
        }

        match self.peek() {
            TokenKind::Keyword(Kw::Fn | Kw::Unsafe | Kw::Virtual | Kw::Override) => {
                Some(ItemKind::Fn(self.parse_fn()))
            }
            TokenKind::Keyword(Kw::Struct) => Some(ItemKind::Struct(self.parse_struct())),
            TokenKind::Keyword(Kw::Class | Kw::Open) => Some(ItemKind::Class(self.parse_class())),
            TokenKind::Keyword(Kw::Enum) => Some(ItemKind::Enum(self.parse_enum())),
            TokenKind::Keyword(Kw::Interface) => {
                Some(ItemKind::Interface(self.parse_interface()))
            }
            TokenKind::Keyword(Kw::Extend) => Some(ItemKind::Extend(self.parse_extend())),
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
            TokenKind::Keyword(Kw::Type) => Some(ItemKind::TypeAlias(self.parse_type_alias())),
            // `abstract class` — `abstract` is contextual (`[LEX-15]`).
            TokenKind::Ident(s) if s.is("abstract") && self.at_kw_at(1, Kw::Class) => {
                Some(ItemKind::Class(self.parse_class()))
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
    fn parse_type_alias(&mut self) -> TypeAlias {
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
        self.expect_newline();
        TypeAlias { name, generics, value, bounds }
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
        ExternBlock { is_unsafe, abi, items }
    }

    pub(crate) fn parse_fn(&mut self) -> FnDecl {
        let is_unsafe = self.eat_kw(Kw::Unsafe);
        let dispatch = if self.eat_kw(Kw::Virtual) {
            Dispatch::Virtual
        } else if self.eat_kw(Kw::Override) {
            Dispatch::Override
        } else {
            Dispatch::Static
        };
        self.expect_kw(Kw::Fn);
        let name = self.expect_ident();
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
        let body = if self.eat_punct(Punct::Colon) { Some(self.parse_block()) } else { None };
        if body.is_none() {
            self.expect_newline();
        }

        FnDecl { name, is_unsafe, dispatch, generics, params, ret, where_clause, body }
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
        let openness = if self.eat_kw(Kw::Open) {
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
                    // A variant is a bare name; anything else is a member.
                    let is_variant = self.at_ident()
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
                // `Circle(radius: f32)` names its field; `Io(io.Error)` does not.
                let named = self.at_ident() && self.at_punct_at(1, Punct::Colon);
                let name = named.then(|| {
                    let ident = self.expect_ident();
                    self.bump(); // ':'
                    ident
                });
                let ty = self.parse_type();
                fields.push(VariantField { name, ty, span: field_start.to(self.prev_span()) });
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
        InterfaceDecl { name, generics, supertraits, where_clause, members }
    }

    fn parse_extend(&mut self) -> ExtendDecl {
        self.expect_kw(Kw::Extend);
        let generics = self.parse_generic_params();
        let target = self.parse_type();
        let implements = self.parse_implements();
        let where_clause = self.parse_where_clause();
        self.expect_punct(Punct::Colon);
        let members = self.parse_type_body();
        ExtendDecl { generics, target, implements, where_clause, members }
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
        self.expect_punct(Punct::Eq);
        let value = self.parse_expr();
        self.expect_newline();
        StaticDecl { name, is_mut, ty, value }
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
                MemberKind::Fn(self.parse_fn())
            }
            TokenKind::Keyword(Kw::Const) => MemberKind::Const(self.parse_const()),
            TokenKind::Keyword(Kw::Type) => MemberKind::TypeAlias(self.parse_type_alias()),
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
        let start = self.span();
        let id = self.next_id();

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
                    params.push(self.parse_type());
                    if !self.eat_punct(Punct::Comma) {
                        break;
                    }
                }
                self.expect_punct(Punct::RParen);
                let ret = self.eat_punct(Punct::Arrow).then(|| Box::new(self.parse_type()));
                TypeKind::Fn { abi, params, ret }
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
