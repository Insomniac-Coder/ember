//! The foreign boundary as the driver reports it: each declaration of an `unsafe extern` block
//! with the facts it asserts about foreign code (`[FFI-10]`, `[TCB-1]`–`[TCB-3]`), for `ember tcb`
//! and `ember calls --foreign` (`[CLI-13]`).
//!
//! A declaration's facts are what a programmer wrote: its signature, `safe fn`, and each clause of
//! its `@ffi(…)` contract. Each is graded **asserted** (`[TCB-1]`): nothing written raises a grade
//! (`[TCB-2]`); a grade above asserted needs evidence, which a hand declaration has no way to claim.

use ember_ast::{AttrArg, Item, ItemKind};
use ember_span::{SourceMap, Span};
use ember_typeck::LoadedModule;

/// One fact a foreign declaration asserts, with where it is written and its grade.
pub struct ForeignFact {
    pub text: String,
    pub source: Span,
    pub grade: &'static str,
}

/// One declaration of an `unsafe extern` block.
pub struct ForeignDecl {
    pub module: String,
    /// The name the program uses, qualified by its module as the checker qualifies it.
    pub qualified: String,
    pub name: String,
    /// `fn`, `static` or `type`.
    pub kind: &'static str,
    pub abi: String,
    pub safe: bool,
    pub source: Span,
    pub facts: Vec<ForeignFact>,
}

/// Every foreign declaration of the program's own modules (the standard library's are its own
/// trusted base), in source order. `module_name` names a loaded module as the report shows it.
pub fn declarations(modules: &[LoadedModule], map: &SourceMap, module_name: &dyn Fn(&LoadedModule) -> String) -> Vec<ForeignDecl> {
    let mut out = Vec::new();
    for loaded in modules {
        if loaded.path.first().is_some_and(|package| package == ember_branding::STD_PACKAGE) {
            continue;
        }
        let module = module_name(loaded);
        let prefix = if loaded.path.is_empty() { String::new() } else { format!("{}.", loaded.path.join(".")) };
        for item in &loaded.module.items {
            let ItemKind::ExternBlock(block) = &item.kind else { continue };
            for inner in &block.items {
                if let Some(decl) = declaration(inner, &block.abi, &module, &prefix, map) {
                    out.push(decl);
                }
            }
        }
    }
    out
}

fn declaration(item: &Item, abi: &str, module: &str, prefix: &str, map: &SourceMap) -> Option<ForeignDecl> {
    let text = |span: Span| -> String {
        let file = map.file(span.file);
        file.text.get(span.start as usize..span.end as usize).unwrap_or("").split_whitespace().collect::<Vec<_>>().join(" ")
    };
    let asserted = |text: String, source: Span| ForeignFact { text, source, grade: "asserted" };
    let (name, kind, safe, signature) = match &item.kind {
        ItemKind::Fn(decl) => (decl.name.name.to_string(), "fn", decl.is_safe, "its signature matches the C declaration"),
        ItemKind::Static(decl) => (decl.name.name.to_string(), "static", false, "its type matches the C object's"),
        ItemKind::TypeAlias(alias) => (alias.name.name.to_string(), "type", false, "it names a C type, used only behind pointers"),
        _ => return None,
    };
    // The symbol it binds, when `@ffi(link_name=…)` renames it (`[FFI-49]`).
    let symbol = item.attrs.iter()
        .filter(|attr| attr.path.len() == 1 && attr.path[0].name.is("ffi"))
        .flat_map(|attr| &attr.args)
        .find_map(|arg| match arg {
            AttrArg::Named { name, value } if name.name.is("link_name") => Some(text(value.span)),
            _ => None,
        })
        .map(|symbol| format!(", C symbol {symbol}"))
        .unwrap_or_default();
    let mut facts = vec![asserted(format!("{signature} (`unsafe extern \"{abi}\"`{symbol})"), item.span)];
    if safe {
        facts.push(asserted("safe to call: a call within its contract cannot break memory safety (`safe fn`)".to_string(), item.span));
    }
    for attr in item.attrs.iter().filter(|attr| attr.path.len() == 1 && attr.path[0].name.is("ffi")) {
        for arg in &attr.args {
            let span = match arg {
                AttrArg::Expr(expr) => expr.span,
                AttrArg::Named { name, value } => name.span.to(value.span),
            };
            let written = text(span);
            let meaning = match arg {
                AttrArg::Named { name, .. } if name.name.is("link_name") => continue,
                AttrArg::Named { name, .. } if name.name.is("effects") => "its effects",
                AttrArg::Expr(expr) => match &expr.kind {
                    ember_ast::ExprKind::Path { segments } if segments.len() == 1 && segments[0].name.is("immutable") => {
                        "a `const` C object that never changes"
                    }
                    ember_ast::ExprKind::Path { segments } if segments.len() == 1 && segments[0].name.is("deterministic") => {
                        "the same result on every run"
                    }
                    _ => "its contract",
                },
                AttrArg::Named { .. } => "its contract",
            };
            facts.push(asserted(format!("{meaning}: `{written}`"), span));
        }
    }
    Some(ForeignDecl {
        module: module.to_string(),
        qualified: format!("{prefix}{name}"),
        name,
        kind,
        abi: abi.to_string(),
        safe,
        source: item.span,
        facts,
    })
}

/// The foreign facts as JSON, for `ember tcb --json` and `ember calls --foreign --json`.
pub fn json(decl: &ForeignDecl, map: &SourceMap) -> serde_json::Value {
    serde_json::json!({
        "module": decl.module,
        "name": decl.name,
        "kind": decl.kind,
        "abi": decl.abi,
        "safe": decl.safe,
        "source": map.location(decl.source),
        "facts": decl.facts.iter().map(|fact| serde_json::json!({
            "fact": fact.text,
            "source": map.location(fact.source),
            "grade": fact.grade,
        })).collect::<Vec<_>>(),
    })
}

/// The facts of one declaration, as `ember tcb` and `ember calls --foreign` print them.
pub fn print(decl: &ForeignDecl, map: &SourceMap, indent: &str) {
    let safety = if decl.kind == "fn" { if decl.safe { " safe" } else { " unsafe to call" } } else { "" };
    println!("{indent}{}  {} `{}`{safety}", map.location(decl.source), decl.kind, decl.name);
    for fact in &decl.facts {
        println!("{indent}  {}  [{}]  {}", fact.text, fact.grade, map.location(fact.source));
    }
    if decl.kind == "fn" && !decl.facts.iter().any(|fact| fact.text.starts_with("its effects")) {
        println!("{indent}  its effects: none stated, so every effect [EFF-3]");
    }
}
