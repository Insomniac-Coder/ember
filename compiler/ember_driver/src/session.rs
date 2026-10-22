//! `[IDE-3]`, `[IDE-4]`, `[IDE-6]` — the compiler as an editor uses it: a [`Session`] answers
//! hover, completion and the symbols of a file from parsing, name resolution and type checking
//! alone — never code generation or a C compiler — in a file with errors too, where an
//! unresolvable name or an untypable expression is an error placeholder that satisfies everything.
//! Everything it knows is in it: dropping it, or [`Session::clear`], frees it all, and asking again
//! rebuilds what is needed. (`ember lsp`, the language server over these, is reserved.)

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use ember_ast as ast;
use ember_diag::{Severity, Sink};
use ember_hir as hir;
use ember_span::{FileId, SourceMap, Span};
use ember_types::{CommonTypes, Ty, TyKind, TypeTable};

/// A place in a file: its 1-based line, and its 1-based column in characters, as a diagnostic
/// names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

/// What is at a place: its description (`total: i64`, a function's signature) and the text it
/// describes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hover {
    pub text: String,
    pub start: Position,
    pub end: Position,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Function,
    Method,
    Struct,
    Class,
    Enum,
    Variant,
    Interface,
    Field,
    Const,
    Static,
    TypeAlias,
    Local,
    Module,
}

/// One thing that could be written at a place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    pub label: String,
    pub kind: Kind,
    /// Its type or signature.
    pub detail: String,
}

/// One declaration of a file, with those inside it (a type's fields, methods and variants).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: Kind,
    /// Its declaration's head as written (`fn area(self) -> f64`, `x: f64`).
    pub detail: String,
    pub start: Position,
    pub end: Position,
    pub children: Vec<Symbol>,
}

/// One diagnostic of the file asked about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub error: bool,
    pub code: Option<String>,
    pub message: String,
    pub start: Position,
}

/// What one check of a file found: everything the queries read.
struct Analysis {
    path: PathBuf,
    text: String,
    map: SourceMap,
    file: FileId,
    root: ast::Module,
    types: TypeTable,
    program: hir::Program,
    problems: Vec<Problem>,
}

/// The editor's compiler. Each query checks the file (its text the editor's buffer when one is
/// set, else the file on disk) unless the last check was of the same text, and answers from that.
#[derive(Default)]
pub struct Session {
    buffers: HashMap<PathBuf, String>,
    last: Option<Analysis>,
}

impl Session {
    pub fn new() -> Session {
        Session::default()
    }

    /// The editor's text for `path`, read instead of the file.
    pub fn set_buffer(&mut self, path: impl Into<PathBuf>, text: impl Into<String>) {
        self.buffers.insert(path.into(), text.into());
    }

    pub fn close_buffer(&mut self, path: &Path) {
        self.buffers.remove(path);
    }

    /// `[IDE-6]` — forget every check; the next query rebuilds what it needs.
    pub fn clear(&mut self) {
        self.last = None;
    }

    /// The file's diagnostics, from parsing, name resolution and type checking.
    pub fn problems(&mut self, path: &Path) -> Vec<Problem> {
        self.analyze(path).problems.clone()
    }

    /// What is at `at`: a local's or a field's name and type, a call's or a declaration's
    /// signature, else the expression's type.
    pub fn hover(&mut self, path: &Path, at: Position) -> Option<Hover> {
        let analysis = self.analyze(path);
        let offset = analysis.offset(at)?;
        // A declaration's name: its head as written.
        if let Some((span, text)) = declaration_at(&analysis.root, offset, &analysis.map) {
            return Some(analysis.hover(span, text));
        }
        // The innermost expression the checker typed there.
        let mut best: Option<(&hir::Function, &hir::Expr)> = None;
        for function in analysis.program.functions.iter().filter(|f| f.span.file == analysis.file) {
            hir::visit_block(&function.body, &mut |expr| {
                if expr.span.file == analysis.file
                    && expr.span.start <= offset
                    && offset < expr.span.end.max(expr.span.start + 1)
                    && best.is_none_or(|(_, inner)| expr.span.end - expr.span.start <= inner.span.end - inner.span.start)
                {
                    best = Some((function, expr));
                }
            });
        }
        // A binding where it is made (a `let`, a parameter, a pattern's name).
        for function in analysis.program.functions.iter().filter(|f| f.span.file == analysis.file) {
            for local in &function.locals {
                if let Some(name) = local.name
                    && local.span.file == analysis.file
                    && local.span.start <= offset
                    && offset < local.span.end
                    && best.is_none_or(|(_, expr)| local.span.end - local.span.start <= expr.span.end - expr.span.start)
                {
                    return Some(analysis.hover(local.span, format!("{name}: {}", analysis.types.display(local.ty))));
                }
            }
        }
        let (function, expr) = best?;
        let text = match &expr.kind {
            hir::ExprKind::Local(local) => {
                let decl = &function.locals[local.0 as usize];
                match decl.name {
                    Some(name) => format!("{name}: {}", analysis.types.display(decl.ty)),
                    None => analysis.types.display(expr.ty),
                }
            }
            hir::ExprKind::Field { base, index } => match analysis.field_name(base.ty, *index) {
                Some(name) => format!("{name}: {}", analysis.types.display(expr.ty)),
                None => analysis.types.display(expr.ty),
            },
            hir::ExprKind::Call { callee, .. } | hir::ExprKind::FnValue(callee) => analysis.signature(*callee),
            _ => analysis.types.display(expr.ty),
        };
        Some(analysis.hover(expr.span, text))
    }

    /// What could be written at `at`: after `value.`, its fields and methods; elsewhere the
    /// locals bound before it in its function, and the module's declarations and imports. Each
    /// starts with what is already typed of the name.
    pub fn complete(&mut self, path: &Path, at: Position) -> Vec<Completion> {
        let analysis = self.analyze(path);
        let Some(offset) = analysis.offset(at) else { return Vec::new() };
        let text = analysis.map.file(analysis.file).text.as_str();
        let before = &text[..offset as usize];
        let typed: String = before.chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_').collect::<Vec<_>>().into_iter().rev().collect();
        let word_start = offset - typed.len() as u32;
        let mut out = Vec::new();
        if text[..word_start as usize].ends_with('.') {
            let dot = word_start - 1;
            if let Some(ty) = analysis.type_ending_at(dot) {
                out = analysis.members(ty);
            }
        } else {
            out = analysis.in_scope(word_start);
        }
        let mut seen = HashSet::new();
        out.retain(|c| c.label.starts_with(typed.as_str()) && seen.insert(c.label.clone()));
        out
    }

    /// The file's declarations, in order, each with its members.
    pub fn symbols(&mut self, path: &Path) -> Vec<Symbol> {
        let analysis = self.analyze(path);
        analysis.root.items.iter().filter_map(|item| analysis.item_symbol(item)).collect()
    }

    /// The check of `path`'s current text: the last one when the text is unchanged.
    fn analyze(&mut self, path: &Path) -> &Analysis {
        let text = match self.buffers.get(path) {
            Some(text) => text.clone(),
            None => std::fs::read(path).ok().and_then(|bytes| ember_span::normalise(&bytes)).unwrap_or_default(),
        };
        let fresh = self.last.as_ref().is_some_and(|last| last.path == path && last.text == text);
        if !fresh {
            // The old check goes before the new one is made: one at a time is all a session holds.
            self.last = None;
            self.last = Some(Analysis::of(path, text));
        }
        self.last.as_ref().expect("just made")
    }
}

impl Analysis {
    /// `[IDE-3]` — parsing, the modules the file imports, name resolution and type checking,
    /// each run whatever the one before reported.
    fn of(path: &Path, text: String) -> Analysis {
        let source = ember_span::normalise(text.as_bytes()).unwrap_or_else(|| text.clone());
        let mut map = SourceMap::new();
        let mut sink = Sink::new();
        let file = map.add(path.to_path_buf(), source.clone());
        let lexed = ember_lexer::lex(file, &source, &mut sink);
        let module = ember_parser::parse(file, &source, lexed.tokens, &mut sink);
        let root = module.clone();
        let root_dir = crate::package_source_root(path);
        let modules = crate::load_modules(module, &root_dir, &mut map, &mut sink);
        let (mut types, common) = TypeTable::new();
        let exports = crate::Exports::new(path, &root_dir);
        let checked = check(&modules, &mut types, &common, &mut sink, &exports);
        let problems = sink
            .take()
            .into_iter()
            .filter(|diagnostic| diagnostic.primary.span.file == file)
            .map(|diagnostic| Problem {
                error: diagnostic.severity == Severity::Error,
                code: diagnostic.code.map(|code| code.to_string()),
                message: diagnostic.message,
                start: position(&map, diagnostic.primary.span),
            })
            .collect();
        Analysis { path: path.to_path_buf(), text, map, file, root, types, program: checked.program, problems }
    }

    /// The byte offset of a position in the file.
    fn offset(&self, at: Position) -> Option<u32> {
        let file = self.map.file(self.file);
        if at.line == 0 || at.line > file.line_count() {
            return None;
        }
        let (start, end) = file.line_range(at.line);
        let line = &file.text[start as usize..end as usize];
        let within: usize = line.chars().take(at.column.saturating_sub(1) as usize).map(char::len_utf8).sum();
        Some(start + within as u32)
    }

    fn hover(&self, span: Span, text: String) -> Hover {
        Hover { text, start: position(&self.map, span), end: position(&self.map, Span::new(span.file, span.end, span.end)) }
    }

    fn field_name(&self, ty: Ty, index: usize) -> Option<String> {
        match self.types.kind(self.referent(ty)) {
            TyKind::Struct(id) => self.types.struct_def(*id).fields.get(index).map(|f| f.name.to_string()),
            TyKind::Class(id) => self.types.class_field_at(*id, index).map(|f| f.name.to_string()),
            TyKind::Tuple(_) => Some(index.to_string()),
            _ => None,
        }
    }

    fn referent(&self, ty: Ty) -> Ty {
        match self.types.kind(ty) {
            TyKind::Ref { inner, .. } | TyKind::Ptr { inner, .. } => self.referent(*inner),
            _ => ty,
        }
    }

    /// `fn name(a: T, …) -> R`, as checked.
    fn signature(&self, def: hir::DefId) -> String {
        let Some(function) = self.program.functions.iter().find(|f| f.def == def) else { return String::new() };
        let params: Vec<String> = function
            .params
            .iter()
            .map(|param| {
                let local = &function.locals[param.local.0 as usize];
                let name = local.name.map_or_else(|| "_".to_string(), |name| name.to_string());
                format!("{name}: {}", self.types.display(local.ty))
            })
            .collect();
        let ret = if matches!(self.types.kind(function.ret), TyKind::Void) { String::new() } else { format!(" -> {}", self.types.display(function.ret)) };
        format!("fn {}({}){ret}", function.name, params.join(", "))
    }

    /// The type of the expression that ends at `end` (the one before a `.`), the widest such.
    fn type_ending_at(&self, end: u32) -> Option<Ty> {
        let mut found: Option<(u32, Ty)> = None;
        for function in self.program.functions.iter().filter(|f| f.span.file == self.file) {
            hir::visit_block(&function.body, &mut |expr| {
                if expr.span.file == self.file && expr.span.end == end && !matches!(expr.kind, hir::ExprKind::Error) {
                    let width = end - expr.span.start;
                    if found.is_none_or(|(widest, _)| width > widest) {
                        found = Some((width, expr.ty));
                    }
                }
            });
            for local in function.locals.iter().filter(|local| local.name.is_some() && local.span.file == self.file && local.span.end == end) {
                found.get_or_insert((0, local.ty));
            }
        }
        found.map(|(_, ty)| ty)
    }

    /// A type's fields and the methods checked for it.
    fn members(&self, ty: Ty) -> Vec<Completion> {
        let ty = self.referent(ty);
        let mut out = Vec::new();
        match self.types.kind(ty) {
            TyKind::Struct(id) => {
                for field in &self.types.struct_def(*id).fields {
                    out.push(Completion { label: field.name.to_string(), kind: Kind::Field, detail: self.types.display(field.ty) });
                }
            }
            TyKind::Class(id) => {
                for index in 0..self.types.class_field_count(*id) {
                    if let Some(field) = self.types.class_field_at(*id, index) {
                        out.push(Completion { label: field.name.to_string(), kind: Kind::Field, detail: self.types.display(field.ty) });
                    }
                }
            }
            _ => {}
        }
        for function in &self.program.functions {
            let Some(param) = function.params.first() else { continue };
            let receiver = self.referent(function.locals[param.local.0 as usize].ty);
            let is_receiver = function.locals[param.local.0 as usize].name.is_some_and(|name| name.is("self"));
            if is_receiver && (receiver == ty || self.is_base(receiver, ty)) {
                let name = function.name.as_str().rsplit('.').next().unwrap_or(function.name.as_str()).to_string();
                out.push(Completion { label: name, kind: Kind::Method, detail: self.signature(function.def) });
            }
        }
        out.sort_by(|a, b| a.label.cmp(&b.label));
        out
    }

    /// Whether class `base` is one of `ty`'s bases.
    fn is_base(&self, base: Ty, ty: Ty) -> bool {
        let (&TyKind::Class(base), &TyKind::Class(mut id)) = (self.types.kind(base), self.types.kind(ty)) else { return false };
        while let Some(next) = self.types.class_def(id).base {
            if next == base {
                return true;
            }
            id = next;
        }
        false
    }

    /// The names in scope at `offset`: the enclosing function's locals bound before it, then the
    /// module's declarations and imported modules.
    fn in_scope(&self, offset: u32) -> Vec<Completion> {
        let mut out = Vec::new();
        let enclosing = self
            .program
            .functions
            .iter()
            .filter(|f| f.span.file == self.file && f.span.start <= offset && offset <= f.span.end)
            .min_by_key(|f| f.span.end - f.span.start);
        if let Some(function) = enclosing {
            for local in &function.locals {
                if let Some(name) = local.name
                    && local.span.end <= offset
                {
                    out.push(Completion { label: name.to_string(), kind: Kind::Local, detail: self.types.display(local.ty) });
                }
            }
        }
        for symbol in self.root.items.iter().filter_map(|item| self.item_symbol(item)) {
            out.push(Completion { label: symbol.name, kind: symbol.kind, detail: symbol.detail });
        }
        for import in &self.root.imports {
            if let ast::ImportKind::Module { path, alias } = &import.kind
                && let Some(name) = alias.or(path.last().copied())
            {
                out.push(Completion { label: name.name.to_string(), kind: Kind::Module, detail: self.map.snippet(import.span).to_string() });
            }
        }
        out
    }

    fn item_symbol(&self, item: &ast::Item) -> Option<Symbol> {
        let head = |span: Span| self.head(span);
        let symbol = |name: &ast::Ident, kind: Kind, span: Span, children: Vec<Symbol>| Symbol {
            name: name.name.to_string(),
            kind,
            detail: head(span),
            start: position(&self.map, name.span),
            end: position(&self.map, Span::new(span.file, span.end, span.end)),
            children,
        };
        let members = |members: &[ast::Member], methods: Kind| -> Vec<Symbol> {
            members
                .iter()
                .filter_map(|member| match &member.kind {
                    ast::MemberKind::Field(field) => Some(symbol(&field.name, Kind::Field, member.span, Vec::new())),
                    ast::MemberKind::Fn(decl) => Some(symbol(&decl.name, methods, member.span, Vec::new())),
                    ast::MemberKind::Const(decl) => Some(symbol(&decl.name, Kind::Const, member.span, Vec::new())),
                    ast::MemberKind::TypeAlias(decl) => Some(symbol(&decl.name, Kind::TypeAlias, member.span, Vec::new())),
                })
                .collect()
        };
        Some(match &item.kind {
            ast::ItemKind::Fn(decl) => symbol(&decl.name, Kind::Function, item.span, Vec::new()),
            ast::ItemKind::Struct(decl) => symbol(&decl.name, Kind::Struct, item.span, members(&decl.members, Kind::Method)),
            ast::ItemKind::Class(decl) => symbol(&decl.name, Kind::Class, item.span, members(&decl.members, Kind::Method)),
            ast::ItemKind::Interface(decl) => symbol(&decl.name, Kind::Interface, item.span, members(&decl.members, Kind::Method)),
            ast::ItemKind::Enum(decl) => {
                let mut children: Vec<Symbol> = decl.variants.iter().map(|v| symbol(&v.name, Kind::Variant, v.span, Vec::new())).collect();
                children.extend(members(&decl.members, Kind::Method));
                symbol(&decl.name, Kind::Enum, item.span, children)
            }
            ast::ItemKind::Const(decl) => symbol(&decl.name, Kind::Const, item.span, Vec::new()),
            ast::ItemKind::Static(decl) => symbol(&decl.name, Kind::Static, item.span, Vec::new()),
            ast::ItemKind::TypeAlias(decl) => symbol(&decl.name, Kind::TypeAlias, item.span, Vec::new()),
            _ => return None,
        })
    }

    /// A declaration's first line as written, without the `:` that opens its body.
    fn head(&self, span: Span) -> String {
        let text = self.map.snippet(span);
        let line = text.lines().next().unwrap_or_default().trim_end();
        line.strip_suffix(':').unwrap_or(line).trim_end().to_string()
    }
}

/// `[IDE-4]` — type checking alone, once: what it asks to evaluate while compiling is left
/// unevaluated (a constant's value is not shown), and nothing is lowered or built.
fn check(modules: &[ember_typeck::LoadedModule], types: &mut TypeTable, common: &CommonTypes, sink: &mut Sink, exports: &crate::Exports) -> ember_typeck::CheckOutput {
    let mark = sink.mark();
    let checked = ember_typeck::check(
        modules,
        types,
        common,
        sink,
        crate::overflow_policy(ember_build::Profile::Debug),
        false,
        true,
        &ember_typeck::ComptimeAnswers::new(),
        None,
        exports,
        &HashSet::new(),
    );
    // What a pass that asks for values reports is the next pass's to say (as in `compile`).
    if !checked.comptime.is_empty() {
        sink.rollback(mark);
    }
    checked
}

fn position(map: &SourceMap, span: Span) -> Position {
    if span.is_dummy() || map.get(span.file).is_none() {
        return Position { line: 0, column: 0 };
    }
    let at = map.line_col(span);
    Position { line: at.line, column: at.col }
}

/// The declaration whose name is at `offset`: its span and head.
fn declaration_at(module: &ast::Module, offset: u32, map: &SourceMap) -> Option<(Span, String)> {
    let named = |name: &ast::Ident, span: Span| {
        (name.span.start <= offset && offset < name.span.end).then(|| {
            let text = map.snippet(span);
            let line = text.lines().next().unwrap_or_default().trim_end();
            (name.span, line.strip_suffix(':').unwrap_or(line).trim_end().to_string())
        })
    };
    for item in &module.items {
        let members: &[ast::Member] = match &item.kind {
            ast::ItemKind::Fn(decl) => {
                if let Some(found) = named(&decl.name, item.span) {
                    return Some(found);
                }
                continue;
            }
            ast::ItemKind::Struct(decl) => {
                if let Some(found) = named(&decl.name, item.span) {
                    return Some(found);
                }
                &decl.members
            }
            ast::ItemKind::Class(decl) => {
                if let Some(found) = named(&decl.name, item.span) {
                    return Some(found);
                }
                &decl.members
            }
            ast::ItemKind::Enum(decl) => {
                if let Some(found) = named(&decl.name, item.span) {
                    return Some(found);
                }
                for variant in &decl.variants {
                    if let Some(found) = named(&variant.name, variant.span) {
                        return Some(found);
                    }
                }
                &decl.members
            }
            ast::ItemKind::Interface(decl) => {
                if let Some(found) = named(&decl.name, item.span) {
                    return Some(found);
                }
                &decl.members
            }
            ast::ItemKind::Extend(decl) => &decl.members,
            ast::ItemKind::Const(decl) => {
                if let Some(found) = named(&decl.name, item.span) {
                    return Some(found);
                }
                continue;
            }
            ast::ItemKind::Static(decl) => {
                if let Some(found) = named(&decl.name, item.span) {
                    return Some(found);
                }
                continue;
            }
            _ => continue,
        };
        for member in members {
            let found = match &member.kind {
                ast::MemberKind::Field(field) => named(&field.name, member.span),
                ast::MemberKind::Fn(decl) => named(&decl.name, member.span),
                ast::MemberKind::Const(decl) => named(&decl.name, member.span),
                ast::MemberKind::TypeAlias(decl) => named(&decl.name, member.span),
            };
            if found.is_some() {
                return found;
            }
        }
    }
    None
}
