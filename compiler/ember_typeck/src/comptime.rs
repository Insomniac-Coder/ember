//! `[CT-1]`–`[CT-7]`, `[TYP-41]` — what the checker asks the compile-time
//! evaluator, and how the answer comes back.
//!
//! Each expression evaluated while compiling becomes a function of its own
//! (`comptime<N>`, no parameters, returning the expression's value). The
//! driver builds those functions with what they call, through the ordinary
//! backend, and runs them in a process of its own, so a value is computed by
//! the target's own arithmetic and layout (`[CT-4]`). The driver then checks
//! the program again with the answers: each one is text, read against the
//! type the expression has in the pass that reads it (`const_eval::read_value`),
//! so a request is known by its key, never by a type of one pass's table.

use std::collections::HashMap;

use ember_span::Span;

/// One expression this pass needs evaluated, which no earlier pass answered.
#[derive(Clone, Debug)]
pub struct ComptimeRequest {
    /// The same in every pass that checks the same expression in the same
    /// context (its position, and the arguments of the instance around it).
    pub key: String,
    /// Where the expression is written: a failure is reported here.
    pub span: Span,
    /// The function that computes it, among the pass's functions.
    pub symbol: String,
    /// What the value is for, for a diagnostic: "this const argument",
    /// "`LIMIT`", "this `comptime` block".
    pub what: String,
    /// A `comptime:` block, run for its checks: it has no value.
    pub block: bool,
    /// `[STA-3]` — a static's initialiser: if it cannot be evaluated while
    /// compiling, it runs on first access instead, and nothing is reported.
    pub quiet: bool,
}

/// What the evaluator gave for a request.
#[derive(Clone, Debug)]
pub enum ComptimeAnswer {
    /// The value, as `ember_codegen_c`'s dump prints it.
    Value(String),
    /// It could not be evaluated; the driver reported why at the request's
    /// span (`E6001`, `E6004`, `E6010`), or the functions it needs have
    /// errors of their own, reported where they are.
    Failed,
}

pub type ComptimeAnswers = HashMap<String, ComptimeAnswer>;
