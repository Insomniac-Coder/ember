//! `[STA-1]`, `[STA-3]`, `[GRM-4]` (A1-17) — module variables. A `static mut` is one storage the
//! program owns, changed by its name with no word and no `unsafe`: every use is the place itself,
//! reached through `ref mut` (`Builtin::ConstStatic` for a value placed in the image's writable
//! data, `Builtin::LazyStatic` for one built on first access). A `static` never changes: `HOOK = v`
//! in a function is `E3023`, never a local that hides it.

use super::*;

impl Checker<'_> {
    /// The Ember `static` an assignment's target names by its bare name or through its module (no
    /// local or capture of the name in scope), with whether it is a `static mut`.
    pub(crate) fn native_static_target(&self, target: &ast::Expr) -> Option<(Symbol, bool)> {
        let qualified = match &target.kind {
            ast::ExprKind::Path { segments } if segments.len() == 1 => {
                let name = segments[0].name;
                if self.lookup(name).is_some() || self.is_capture_candidate(name) {
                    return None;
                }
                self.resolve_name(name)
            }
            ast::ExprKind::Field { base, name } => {
                let module = self.namespace_named(base)?;
                self.qualified_in_module(module, name.name)
            }
            _ => return None,
        };
        self.native_statics.get(&qualified).map(|&mutable| (qualified, mutable))
    }

    /// `[STA-1]` — `HOOK = v` (or `HOOK += v`) where `HOOK` is a `static`: it never changes, and the
    /// line makes no local that hides it (`E3023`).
    pub(crate) fn refuse_static_assignment(&mut self, name: Symbol, target: &ast::Expr) {
        let shown = name.as_str().rsplit('.').next().unwrap_or(name.as_str()).to_string();
        let help = format!("for a module variable that functions change declare it `static mut {shown}`; for a local of your own use another name");
        self.sink.emit(
            Diagnostic::error(codes::E3023, target.span, format!("`{shown}` is a `static`, which never changes"))
                .primary_label("this line would change it")
                .help(help)
                .note("a `static` never changes; a `static mut` is a module variable a function changes by its name [STA-1]"),
        );
    }

    /// `[CT-1a]` — a `static mut` named in what compile time evaluates: the program's value of it
    /// exists only while it runs. A static's own initialiser that names one is not evaluated while
    /// compiling (it runs on first access, `[STA-3]`); `comptime(e)` and a `const` are `E6010`.
    pub(crate) fn module_variable_at_compile_time(&mut self, name: Symbol, span: Span) -> Option<Expr> {
        if self.comptime_outer_names.is_none() || !self.native_statics.get(&name).copied().unwrap_or(false) {
            return None;
        }
        let shown = name.as_str().rsplit('.').next().unwrap_or(name.as_str()).to_string();
        self.sink.emit(
            Diagnostic::error(codes::E6010, span, format!("`{shown}` is a `static mut`, and a value made when compiling cannot see what the program will hold while running"))
                .help("pass the value in, or make it a `static` if it never changes")
                .note("what compile time evaluates reaches no module variable that lives at run time [CT-1a]"),
        );
        Some(Expr { ty: self.common.error, kind: ExprKind::Error, span })
    }
}
