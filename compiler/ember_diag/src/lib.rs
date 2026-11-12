//! The diagnostic model, its renderer, and the error-code registry.
//!
//! Spec: Part XIX §6. The rendered shape is normative — `[DIA-1]` (code, one
//! primary span, secondary labels, `help`, `note`), `[DIA-2]` (lowercase, no
//! trailing period, name the thing, never say "you"), `[CLI-1]` (`--json`
//! emits the same structure).
//!
//! Nothing in this crate reads the filesystem; rendering takes a
//! [`SourceMap`](ember_span::SourceMap) so that synthesised sources render the
//! same way real files do.

use std::fmt::Write as _;

use ember_span::{FileId, SourceMap, Span};

pub mod shapes;

/// `[UNS-7]` (verdict 125) — the `@safety` the C importer writes on a function callers call in unsafe
/// code whose overlay entry gives none: what C requires of a call is not stated, and the type checker
/// reports `L3015` at the function's first call.
pub const UNSTATED_CONTRACT: &str =
    "the header states no contract: each pointer's ownership, count, nullability and lifetime are the caller's to keep ([FFI-38])";
/// `[UNS-7]`, `[FFI-8]` — the same, for a variadic C function.
pub const UNSTATED_VARIADIC: &str =
    "C reads each further argument as the type its format or count names: the caller matches them ([FFI-8])";
pub mod habits;
pub mod codes;
pub use codes::Code;

/// How loud a diagnostic is. Only `Error` and `Warning` reach the top level;
/// `Note` and `Help` exist as sub-diagnostics, and `Lint` is what `ember lint`
/// emits (L-codes).
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum Severity {
    Error,
    Warning,
    Lint,
    Note,
    Help,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Lint => "lint",
            Severity::Note => "note",
            Severity::Help => "help",
        }
    }

    pub fn is_fatal(self) -> bool {
        matches!(self, Severity::Error)
    }
}

/// A span with an optional message, rendered as a caret run under the source.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Label {
    pub span: Span,
    pub message: Option<String>,
}

impl Label {
    pub fn new(span: Span, message: impl Into<String>) -> Label {
        Label { span, message: Some(articles(message.into())) }
    }

    /// A label that only points, with no text of its own.
    pub fn bare(span: Span) -> Label {
        Label { span, message: None }
    }
}

/// A machine-applicable edit. `[DIA-1]` asks for one wherever the fix is
/// unambiguous; `ember fix` (v1.1) and editors consume these.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Suggestion {
    pub message: String,
    /// What replaces each span, applied together; edits at one place apply in this order.
    pub edits: Vec<(Span, Replacement)>,
    /// `[CLI-20]` — a rewrite of source written for 0.9.8 that `ember fmt --migrate` applies;
    /// not rendered, since the diagnostic's own help says it.
    pub migration: bool,
}

/// The text an edit puts in place of its span.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum Replacement {
    Text(String),
    /// The source text of another span (a rewrite that moves an expression).
    Source(Span),
}

impl Replacement {
    pub fn text<'a>(&'a self, map: &'a SourceMap) -> &'a str {
        match self {
            Replacement::Text(text) => text,
            Replacement::Source(span) => map.snippet(*span),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<Code>,
    pub message: String,
    pub primary: Label,
    pub secondary: Vec<Label>,
    pub helps: Vec<String>,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
    /// `[DIA-21]` — the Python habit this diagnostic answers (ODR-179).
    pub habit: Option<habits::Habit>,
}

/// `[DIA-2]` — "a `owns Player`" reads "an `owns Player`": the article before a quoted name is chosen
/// from the word that follows it, for every message however it was built. Only a standalone `a` or
/// `an` directly before a backtick is read; a name that does not begin with a letter (`[int; 3]`,
/// `*mut T`) leaves its article as written.
pub fn articles(text: String) -> String {
    if !text.contains("a `") && !text.contains("an `") && !text.contains("A `") && !text.contains("An `") {
        return text;
    }
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len() + 4);
    let mut at = 0;
    while at < text.len() {
        let rest = &text[at..];
        let standalone = at == 0 || !(bytes[at - 1].is_ascii_alphanumeric() || matches!(bytes[at - 1], b'`' | b'_' | b'.' | b'-' | b'\''));
        let article = ["An `", "an `", "a `", "A `"].into_iter().find(|a| rest.starts_with(*a));
        if standalone && let Some(article) = article {
            let word: String = rest[article.len()..].chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if let Some(vowel) = starts_with_vowel_sound(&word) {
                let upper = article.starts_with(|c: char| c.is_uppercase());
                out.push_str(match (vowel, upper) {
                    (true, false) => "an",
                    (true, true) => "An",
                    (false, false) => "a",
                    (false, true) => "A",
                });
                out.push_str(" `");
                at += article.len();
                continue;
            }
        }
        let c = rest.chars().next().expect("a character");
        out.push(c);
        at += c.len_utf8();
    }
    out
}

/// Whether `word` is said with a vowel sound first: `owns`, `int`, `Option`, `f32` (eff), `unsafe` do;
/// `u8`, `unit`, `once`, `String` do not; nor do `Set` or `Map`, while the acronyms `SoA` and `Rc` do. `None` where it does not begin with a letter.
fn starts_with_vowel_sound(word: &str) -> Option<bool> {
    let first = word.chars().next().filter(|c| c.is_alphabetic())?.to_ascii_lowercase();
    let lower = word.to_ascii_lowercase();
    // A one-letter name is said as its letter: `n` is "en", `b` is "bee".
    if word.chars().count() == 1 {
        return Some(matches!(first, 'a' | 'e' | 'f' | 'h' | 'i' | 'l' | 'm' | 'n' | 'o' | 'r' | 's' | 'x'));
    }
    Some(match first {
        'a' | 'e' | 'i' => !lower.starts_with("eu"),
        'o' => !(lower.starts_with("one") || lower.starts_with("once")),
        'u' => lower.starts_with("un") && !lower.starts_with("uni"),
        // A short acronym spells its first letter's name: `SoA`, `SIMD`, `FFI`, `NaN`, `Rc` (ess, em, eff, en, ar).
        // A lowercase name may be a word (`list`, `set`) or letters (`lto`, `msvc`), which its spelling does
        // not tell: its article stays as the message has it.
        'f' | 'h' | 'l' | 'm' | 'n' | 'r' | 's' | 'x' => {
            let acronym = word.starts_with(|c: char| c.is_uppercase()) && (2..=4).contains(&word.len()) && word[1..].chars().filter(|c| c.is_lowercase()).count() <= 1;
            let float = first == 'f' && word.len() > 1 && word[1..].chars().all(|c| c.is_ascii_digit());
            if !acronym && !float && word.chars().all(|c| !c.is_uppercase()) {
                return None;
            }
            acronym || float
        }
        _ => false,
    })
}

impl Diagnostic {
    pub fn error(code: Code, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            code: Some(code),
            message: articles(message.into()),
            primary: Label::bare(span),
            secondary: Vec::new(),
            helps: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
            habit: None,
        }
    }

    pub fn warning(code: Code, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic { severity: Severity::Warning, ..Diagnostic::error(code, span, message) }
    }

    /// An `L`-code. Rendered as a warning; kept a separate constructor so a
    /// lint cannot be raised to an error by accident, and so `[MAN-3]`'s
    /// `[lints]` table has something to name.
    pub fn lint(code: Code, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic { severity: Severity::Warning, ..Diagnostic::error(code, span, message) }
    }

    /// `[DIA-21]` — this diagnostic answers a Python habit (ODR-179).
    pub fn habit(mut self, habit: habits::Habit) -> Diagnostic {
        self.habit = Some(habit);
        self
    }

    /// Text on the primary caret itself, distinct from the header message.
    pub fn primary_label(mut self, text: impl Into<String>) -> Diagnostic {
        self.primary.message = Some(articles(text.into()));
        self
    }

    pub fn secondary(mut self, span: Span, text: impl Into<String>) -> Diagnostic {
        self.secondary.push(Label::new(span, text));
        self
    }

    pub fn help(mut self, text: impl Into<String>) -> Diagnostic {
        self.helps.push(articles(text.into()));
        self
    }

    /// Each note `notes` holds (none, or one an `Option` gives).
    pub fn notes_from(mut self, notes: impl IntoIterator<Item = String>) -> Diagnostic {
        self.notes.extend(notes.into_iter().map(articles));
        self
    }

    pub fn note(mut self, text: impl Into<String>) -> Diagnostic {
        self.notes.push(articles(text.into()));
        self
    }

    pub fn suggest(mut self, message: impl Into<String>, span: Span, text: impl Into<String>) -> Diagnostic {
        self.suggestions.push(Suggestion {
            message: articles(message.into()),
            edits: vec![(span, Replacement::Text(text.into()))],
            migration: false,
        });
        self
    }

    /// A fix-it of several edits, applied together.
    pub fn suggest_edits(mut self, message: impl Into<String>, edits: Vec<(Span, Replacement)>) -> Diagnostic {
        self.suggestions.push(Suggestion { message: message.into(), edits, migration: false });
        self
    }

    /// `[CLI-20]` — the mechanical rewrite of a 0.9.8 form, which `ember fmt --migrate` applies.
    pub fn migrate(self, message: impl Into<String>, span: Span, text: impl Into<String>) -> Diagnostic {
        self.migrate_edits(message, vec![(span, Replacement::Text(text.into()))])
    }

    pub fn migrate_edits(mut self, message: impl Into<String>, edits: Vec<(Span, Replacement)>) -> Diagnostic {
        self.suggestions.push(Suggestion { message: message.into(), edits, migration: true });
        self
    }

    pub fn is_fatal(&self) -> bool {
        self.severity.is_fatal()
    }
}

/// `[UNS-7]` (verdicts 125, 134) — an `L3015` for a C or C++ function is given for each call, and
/// reported once, at its first call by place, naming the others (`Sink::ordered`): its message,
/// which names the function, is the key. A lambda's names none: each is its own.
pub fn once_per_function(diagnostic: &Diagnostic) -> Option<&str> {
    (diagnostic.code == Some(codes::L3015) && !diagnostic.message.starts_with("this lambda")).then_some(diagnostic.message.as_str())
}

/// Collects diagnostics for one compiler invocation.
///
/// A stage reports into the sink and keeps going where it can; the driver
/// decides when to stop. `[AST-2]` limits cascades at the parser, not here.
#[derive(Default)]
pub struct Sink {
    diagnostics: Vec<Diagnostic>,
    error_count: usize,
    warning_count: usize,
    /// `-Dwarnings`: warnings are counted as errors for the exit code.
    pub deny_warnings: bool,
    /// `[DIA-7]` — ownership or borrow errors the classifier could not place.
    unclassified: Vec<String>,
}

impl Sink {
    pub fn new() -> Sink {
        Sink::default()
    }

    pub fn emit(&mut self, d: Diagnostic) {
        match d.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning | Severity::Lint => self.warning_count += 1,
            _ => {}
        }
        self.diagnostics.push(d);
    }

    /// Each diagnostic `diagnostics` holds (none, or one an `Option` gives).
    pub fn emit_all(&mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) {
        for diagnostic in diagnostics {
            self.emit(diagnostic);
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// A point to `rollback` to.
    pub fn mark(&self) -> usize {
        self.diagnostics.len()
    }

    /// Amend the diagnostic at `at` (a `mark`) once more is known: its message, helps and notes. Its
    /// severity, and so the counts, stay.
    pub fn amend(&mut self, at: usize, amend: impl FnOnce(&mut Diagnostic)) {
        let severity = self.diagnostics[at].severity;
        amend(&mut self.diagnostics[at]);
        assert_eq!(self.diagnostics[at].severity, severity, "an amended diagnostic keeps its severity");
    }

    /// Forget what was emitted after `mark`, for a check whose diagnostics are
    /// reported elsewhere. Returns whether an error was among them.
    pub fn rollback(&mut self, mark: usize) -> bool {
        let mut errors = false;
        for d in self.diagnostics.drain(mark..) {
            match d.severity {
                Severity::Error => {
                    self.error_count -= 1;
                    errors = true;
                }
                Severity::Warning | Severity::Lint => self.warning_count -= 1,
                _ => {}
            }
        }
        errors
    }

    /// `[DIA-7]` — emit an ownership or borrow diagnostic, which MUST carry
    /// one of §XIX.6.1's shapes. A code the classifier cannot place is
    /// recorded rather than silently emitted, because "an unexplained
    /// rejection is the single largest usability cost of static aliasing
    /// rules" and the rule exists to make adding one impossible by accident.
    pub fn emit_classified(&mut self, diagnostic: Diagnostic) {
        if let Some(code) = diagnostic.code {
            if shapes::shape_for(code).is_none() {
                self.unclassified.push(format!(
                    "{code}: {} at {:?}",
                    diagnostic.message, diagnostic.primary.span
                ));
            }
        }
        self.emit(diagnostic);
    }

    /// The borrow errors no shape could be found for. `[DIA-7]` writes these
    /// to `target/<profile>/unclassified-borrow-errors.log` and fails CI when
    /// the conformance suite produces any.
    pub fn unclassified(&self) -> &[String] {
        &self.unclassified
    }

    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0 || (self.deny_warnings && self.warning_count > 0)
    }

    /// `[BLD-7]` — what `other` holds, after what this sink holds: its diagnostics, in its order,
    /// and the borrow errors it could not place.
    pub fn absorb(&mut self, other: Sink) {
        for diagnostic in other.diagnostics {
            self.emit(diagnostic);
        }
        self.unclassified.extend(other.unclassified);
    }

    pub fn take(&mut self) -> Vec<Diagnostic> {
        self.error_count = 0;
        self.warning_count = 0;
        std::mem::take(&mut self.diagnostics)
    }

    /// Everything collected, in the program's order: by file (as the build read them), then by
    /// place; diagnostics at one place in the order they were emitted. `[BLD-1]` — so a build that
    /// checks some modules again and reports the rest's kept diagnostics reports what a clean
    /// build does, in the same order.
    /// Everything collected, in the program's order: by file, in the order the files were read, then
    /// by place. `[UNS-7]` (verdict 134) — the `L3015`s of one C or C++ function, one for each call,
    /// are reported as one: at its first call, with a note naming each other call, in that order.
    pub fn ordered(&self, map: &SourceMap) -> Vec<std::borrow::Cow<'_, Diagnostic>> {
        let mut ordered: Vec<&Diagnostic> = self.diagnostics.iter().collect();
        ordered.sort_by_key(|d| (d.primary.span.file, d.primary.span.start, d.primary.span.end));
        // Each function's first diagnostic's place in `out`, and the calls it already names.
        let mut first: std::collections::HashMap<&str, (usize, Vec<Span>)> = std::collections::HashMap::new();
        let mut out: Vec<std::borrow::Cow<'_, Diagnostic>> = Vec::with_capacity(ordered.len());
        for d in ordered {
            // A line number a pass without the source wrote as `line_mark(span)`.
            if has_line_marks(d) {
                let mut marked = d.clone();
                expand_line_marks(&mut marked, map);
                out.push(std::borrow::Cow::Owned(marked));
                continue;
            }
            let Some(key) = once_per_function(d) else {
                out.push(std::borrow::Cow::Borrowed(d));
                continue;
            };
            if let Some((at, calls)) = first.get_mut(key) {
                if !calls.contains(&d.primary.span) {
                    calls.push(d.primary.span);
                    // Next to the call it names, before the warning's own notes.
                    out[*at].to_mut().notes.insert(calls.len() - 2, format!("also called at {}", map.location(d.primary.span)));
                }
            } else {
                first.insert(key, (out.len(), vec![d.primary.span]));
                out.push(std::borrow::Cow::Borrowed(d));
            }
        }
        out
    }

    /// Human rendering of everything collected, in the program's order (`ordered`).
    pub fn render(&self, map: &SourceMap) -> String {
        let mut out = String::new();
        for d in self.ordered(map) {
            out.push_str(&render(&d, map));
            out.push('\n');
        }
        out
    }

    /// `--json`: one JSON object per diagnostic, newline-delimited
    /// (`[CLI-1]`). Line-delimited rather than one array so that a consumer can
    /// stream them while the compiler is still running.
    pub fn render_json(&self, map: &SourceMap) -> String {
        let mut out = String::new();
        for d in self.ordered(map) {
            let _ = writeln!(out, "{}", to_json(&d, map));
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Human rendering
// ---------------------------------------------------------------------------

/// The line `span` starts on, for a message written where the source text is not at hand (the
/// analyses): a marker the renderer (`Sink::ordered`) replaces by the number.
pub fn line_mark(span: Span) -> String {
    format!("\u{1}{}:{}\u{2}", span.file.0, span.start)
}

/// The program's own text of `span`, for a message written where the source text is not at hand
/// (the analyses; `[DIA-2]`: a refusal names what was refused as the program wrote it): a marker
/// the renderer replaces by the span's text, a run of whitespace shown as one space.
pub fn text_mark(span: Span) -> String {
    format!("\u{1}={}:{}:{}\u{2}", span.file.0, span.start, span.end)
}

fn has_line_marks(d: &Diagnostic) -> bool {
    let marked = |text: &str| text.contains('\u{1}');
    marked(&d.message)
        || d.helps.iter().any(|t| marked(t))
        || d.notes.iter().any(|t| marked(t))
        || d.primary.message.as_deref().is_some_and(marked)
        || d.secondary.iter().any(|label| label.message.as_deref().is_some_and(marked))
}

/// `text` with each `line_mark` replaced by its line number: what every place that shows a message
/// outside `Sink::ordered` (the migrator's list of what is left) writes.
pub fn lined(text: &str, map: &SourceMap) -> String {
    expand_marks(text, map)
}

fn expand_line_marks(d: &mut Diagnostic, map: &SourceMap) {
    let expand = expand_marks;
    d.message = expand(&d.message, map);
    for text in d.helps.iter_mut().chain(d.notes.iter_mut()) {
        *text = expand(text, map);
    }
    if let Some(text) = d.primary.message.as_mut() {
        *text = expand(text, map);
    }
    for label in &mut d.secondary {
        if let Some(text) = label.message.as_mut() {
            *text = expand(text, map);
        }
    }
}

fn expand_marks(text: &str, map: &SourceMap) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('\u{1}') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('\u{2}') else { break };
        if let Some(range) = after[..close].strip_prefix('=') {
            let mut parts = range.split(':').map(|part| part.parse::<u32>().ok());
            let text = match (parts.next().flatten(), parts.next().flatten(), parts.next().flatten()) {
                (Some(file), Some(start), Some(end)) => map
                    .get(FileId(file))
                    .and_then(|file| file.text.get(start as usize..end as usize))
                    .map(|text| text.split_whitespace().collect::<Vec<_>>().join(" ")),
                _ => None,
            };
            out.push_str(&text.unwrap_or_else(|| "?".to_string()));
            rest = &after[close + 1..];
            continue;
        }
        let line = after[..close].split_once(':').and_then(|(file, start)| {
            let file = map.get(FileId(file.parse().ok()?))?;
            Some(file.line_of(start.parse().ok()?))
        });
        out.push_str(&line.map_or_else(|| "?".to_string(), |line| line.to_string()));
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}

/// Render one diagnostic in the format of XIX §6.
///
/// ```text
/// error[E3040]: use of moved value `ps`
///   --> src/main.em:12:11
///    |
/// 10 |     n = take(ps)
///    |              -- value moved into `take` here
/// 12 |     print(ps.len())
///    |           ^^ value used here after move
///    |
///    = help: if you need `ps` afterwards, pass a clone: `take(ps.clone())`
/// ```
pub fn render(d: &Diagnostic, map: &SourceMap) -> String {
    let mut out = String::new();

    match d.code {
        Some(code) => {
            let _ = writeln!(out, "{}[{}]: {}", d.severity.label(), code, d.message);
        }
        None => {
            let _ = writeln!(out, "{}: {}", d.severity.label(), d.message);
        }
    }

    // Labels that can actually be shown: resolvable. D-411 — a label in
    // another file than the primary's is shown under that file's own
    // header; it was dropped, so a collision across modules named one side.
    let primary_file = d.primary.span.file;
    let mut labels: Vec<(&Label, bool)> = Vec::new();
    if !d.primary.span.is_dummy() && map.get(primary_file).is_some() {
        labels.push((&d.primary, true));
    }
    for label in &d.secondary {
        if !label.span.is_dummy() && map.get(label.span.file).is_some() {
            labels.push((label, false));
        }
    }

    if labels.is_empty() {
        // No renderable span. `[CMP-2]` forbids this after parsing, but a
        // manifest or CLI error legitimately has nowhere to point.
        for help in &d.helps {
            let _ = writeln!(out, "  = help: {help}");
        }
        for note in &d.notes {
            let _ = writeln!(out, "  = note: {note}");
        }
        return out;
    }

    // One group per file: the first label's (the primary's when it has a
    // place) first, then the others in the order their labels come.
    let mut files: Vec<FileId> = Vec::new();
    for &(label, _) in &labels {
        if !files.contains(&label.span.file) {
            files.push(label.span.file);
        }
    }
    let groups: Vec<(FileId, Vec<(u32, &Label, bool)>)> = files
        .iter()
        .map(|&id| {
            let file = map.file(id);
            let mut rows: Vec<(u32, &Label, bool)> = labels
                .iter()
                .filter(|(label, _)| label.span.file == id)
                .map(|&(label, is_primary)| (file.line_of(label.span.start), label, is_primary))
                .collect();
            rows.sort_by_key(|&(line, _, is_primary)| (line, !is_primary));
            (id, rows)
        })
        .collect();

    let last_line = groups.iter().flat_map(|(_, rows)| rows.iter().map(|&(l, _, _)| l)).max().unwrap_or(1);
    let gutter = last_line.to_string().len();
    let pad = " ".repeat(gutter);

    for (index, (id, rows)) in groups.iter().enumerate() {
        let file = map.file(*id);
        if index == 0 {
            // At the primary's place when it has one, as before.
            let at = if !d.primary.span.is_dummy() && *id == primary_file { d.primary.span } else { rows[0].1.span };
            let _ = writeln!(out, "{pad}--> {}", map.location(at));
        } else {
            let _ = writeln!(out, "{pad} |");
            let _ = writeln!(out, "{pad}::: {}", map.location(rows[0].1.span));
        }
        let _ = writeln!(out, "{pad} |");

        let mut previous_line: Option<u32> = None;
        for &(line, label, is_primary) in rows {
            match previous_line {
                // Bridge a one-line gap with the source; elide anything wider.
                Some(prev) if line == prev + 2 => {
                    let _ = writeln!(out, "{:>gutter$} | {}", prev + 1, file.line_text(prev + 1));
                }
                Some(prev) if line > prev + 2 => {
                    let _ = writeln!(out, "...");
                }
                _ => {}
            }

            if previous_line != Some(line) {
                let _ = writeln!(out, "{line:>gutter$} | {}", file.line_text(line));
            }

            let (caret_col, width) = caret_extent(file, label.span);
            let marker = if is_primary { '^' } else { '-' };
            let underline: String = std::iter::repeat_n(marker, width.max(1) as usize).collect();
            let indent = " ".repeat(caret_col.saturating_sub(1) as usize);
            match &label.message {
                Some(text) => {
                    let _ = writeln!(out, "{pad} | {indent}{underline} {text}");
                }
                None => {
                    let _ = writeln!(out, "{pad} | {indent}{underline}");
                }
            }
            previous_line = Some(line);
        }
    }

    let _ = writeln!(out, "{pad} |");
    for help in &d.helps {
        let _ = writeln!(out, "{pad} = help: {help}");
    }
    for suggestion in d.suggestions.iter().filter(|suggestion| !suggestion.migration) {
        let _ = writeln!(out, "{pad} = help: {}", suggestion.message);
    }
    for note in &d.notes {
        let _ = writeln!(out, "{pad} = note: {note}");
    }
    out
}

/// Where a label's underline starts and how wide it is, in display columns.
///
/// A span that crosses a line boundary is clamped to the end of its first
/// line: the caret run stays on one line, which is what the format allows for.
fn caret_extent(file: &ember_span::SourceFile, span: Span) -> (u32, u32) {
    let start = file.line_col(span.start);
    let line_end = file.line_range(start.line).1;
    let end_offset = span.end.min(line_end);
    let end_col = file.line_col(end_offset).col;
    (start.col, end_col.saturating_sub(start.col))
}

// ---------------------------------------------------------------------------
// JSON rendering
// ---------------------------------------------------------------------------

fn span_json(span: Span, map: &SourceMap) -> serde_json::Value {
    use serde_json::json;
    if span.is_dummy() || map.get(span.file).is_none() {
        return json!(null);
    }
    let file = map.file(span.file);
    let start = file.line_col(span.start);
    let end = file.line_col(span.end);
    json!({
        "file": file.path.display().to_string(),
        "byte_start": span.start,
        "byte_end": span.end,
        "line_start": start.line,
        "column_start": start.col,
        "line_end": end.line,
        "column_end": end.col,
    })
}

fn label_json(label: &Label, map: &SourceMap, is_primary: bool) -> serde_json::Value {
    use serde_json::json;
    json!({
        "span": span_json(label.span, map),
        "message": label.message,
        "primary": is_primary,
    })
}

pub fn to_json(d: &Diagnostic, map: &SourceMap) -> serde_json::Value {
    use serde_json::json;
    let mut labels = vec![label_json(&d.primary, map, true)];
    labels.extend(d.secondary.iter().map(|l| label_json(l, map, false)));
    json!({
        "severity": d.severity.label(),
        "code": d.code.map(|c| c.to_string()),
        "shape": d.habit.map(|h| h.to_string()),
        "message": d.message,
        "labels": labels,
        "helps": d.helps,
        "notes": d.notes,
        "suggestions": d.suggestions.iter().map(|s| json!({
            "message": s.message,
            "migration": s.migration,
            "edits": s.edits.iter().map(|(span, text)| json!({
                "span": span_json(*span, map),
                "replacement": text.text(map),
            })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "rendered": render(d, map),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ember_span::SourceMap;

    fn fixture() -> (SourceMap, ember_span::FileId) {
        let mut map = SourceMap::new();
        let id = map.add(
            "src/main.em",
            "fn main():\n    ps = Array[i32]()\n    n = take(ps)\n    print(n)\n    print(ps.len())\n",
        );
        (map, id)
    }

    #[test]
    fn header_carries_code_and_message() {
        let (map, id) = fixture();
        let d = Diagnostic::error(codes::E3040, Span::new(id, 73, 75), "use of moved value `ps`");
        let text = render(&d, &map);
        assert!(text.starts_with("error[E3040]: use of moved value `ps`\n"), "{text}");
    }

    #[test]
    fn location_line_points_at_the_primary_span() {
        let (map, id) = fixture();
        // byte 76 is `ps` inside `print(ps.len())` on line 5.
        let d = Diagnostic::error(codes::E3040, Span::new(id, 73, 75), "use of moved value `ps`");
        let text = render(&d, &map);
        assert!(text.contains("--> src/main.em:5:11"), "{text}");
    }

    #[test]
    fn primary_uses_carets_and_secondary_uses_dashes() {
        let (map, id) = fixture();
        let d = Diagnostic::error(codes::E3040, Span::new(id, 73, 75), "use of moved value `ps`")
            .primary_label("value used here after move")
            .secondary(Span::new(id, 46, 48), "value moved into `take` here");
        let text = render(&d, &map);
        assert!(text.contains("-- value moved into `take` here"), "{text}");
        assert!(text.contains("^^ value used here after move"), "{text}");
    }

    #[test]
    fn a_one_line_gap_is_bridged_and_wider_gaps_elide() {
        let (map, id) = fixture();
        // Labels on line 3 and line 5: line 4 is shown as context.
        let bridged = Diagnostic::error(codes::E3040, Span::new(id, 73, 75), "moved")
            .secondary(Span::new(id, 46, 48), "moved here");
        assert!(render(&bridged, &map).contains("print(n)"));

        // Labels on line 1 and line 5: the middle is elided.
        let elided = Diagnostic::error(codes::E3040, Span::new(id, 73, 75), "moved")
            .secondary(Span::new(id, 3, 7), "declared here");
        let text = render(&elided, &map);
        assert!(text.contains("..."), "{text}");
        assert!(!text.contains("print(n)"), "{text}");
    }

    /// D-411 — a label in another file is shown under that file's own
    /// header, not dropped: a collision across modules names both sides.
    #[test]
    fn a_label_in_another_file_has_its_own_header() {
        let (mut map, id) = fixture();
        let other = map.add("lib/shapes", "struct Wrap:\n    x: int\n\nextend Wrap implements Named:\n");
        let d = Diagnostic::error(codes::E2041, Span::new(id, 73, 75), "`Wrap` already implements `Named`")
            .secondary(Span::new(other, 32, 36), "the first implementation is here");
        let text = render(&d, &map);
        let primary = format!("--> {}", map.location(Span::new(id, 73, 75)));
        assert!(text.contains(&primary), "{text}");
        assert!(text.contains("::: lib/shapes:4:8"), "{text}");
        assert!(text.contains("extend Wrap implements Named:"), "{text}");
        assert!(text.contains("---- the first implementation is here"), "{text}");
        let main_at = text.find(&primary).expect("the primary's header");
        let other_at = text.find("::: lib/shapes").expect("the other file's header");
        assert!(main_at < other_at, "{text}");
    }

    #[test]
    fn help_and_note_are_footers() {
        let (map, id) = fixture();
        let d = Diagnostic::error(codes::E3040, Span::new(id, 73, 75), "use of moved value `ps`")
            .help("pass a clone: `take(ps.clone())`")
            .note("`Array[i32]` is not Copy because it owns heap memory");
        let text = render(&d, &map);
        assert!(text.contains("= help: pass a clone: `take(ps.clone())`"), "{text}");
        assert!(text.contains("= note: `Array[i32]` is not Copy"), "{text}");
    }

    /// `[DIA-1]` — a diagnostic has a code, one primary span, labelled secondary spans, a help
    /// with a machine-applicable fix-it, and notes; `--json` gives the same structure, field for
    /// field, beside the rendered text.
    #[test]
    fn json_carries_the_same_structure_as_the_rendering() {
        let (map, id) = fixture();
        let d = Diagnostic::error(codes::E3040, Span::new(id, 73, 75), "use of moved value `ps`")
            .primary_label("value used here after move")
            .secondary(Span::new(id, 46, 48), "value moved into `take` here")
            .help("pass a clone")
            .suggest("pass a clone", Span::new(id, 46, 48), "ps.clone()")
            .note("`Array[i32]` is not Copy");
        let json = to_json(&d, &map);
        assert_eq!(json["code"], "E3040");
        assert_eq!(json["message"], "use of moved value `ps`");
        let labels = json["labels"].as_array().expect("labels");
        assert_eq!(labels.len(), 2);
        assert_eq!(labels.iter().filter(|label| label["primary"] == true).count(), 1);
        assert_eq!(labels[0]["message"], "value used here after move");
        assert_eq!(labels[0]["span"]["line_start"], 5);
        assert_eq!(labels[1]["message"], "value moved into `take` here");
        assert_eq!(labels[1]["span"]["line_start"], 3);
        assert_eq!(json["helps"][0], "pass a clone");
        assert_eq!(json["notes"][0], "`Array[i32]` is not Copy");
        let edit = &json["suggestions"][0]["edits"][0];
        assert_eq!(edit["replacement"], "ps.clone()");
        assert_eq!(edit["span"]["byte_start"], 46);
        assert_eq!(json["rendered"], render(&d, &map));
    }

    #[test]
    fn a_diagnostic_with_no_span_still_renders() {
        let map = SourceMap::new();
        let d = Diagnostic::error(codes::E9001, Span::DUMMY, "unknown key `bakcend` in ember.toml")
            .help("did the manifest mean `backend`");
        let text = render(&d, &map);
        assert!(text.contains("unknown key `bakcend`"), "{text}");
        assert!(text.contains("= help:"), "{text}");
    }

    #[test]
    fn json_carries_the_same_facts_as_the_rendering() {
        let (map, id) = fixture();
        let d = Diagnostic::error(codes::E3040, Span::new(id, 73, 75), "use of moved value `ps`")
            .secondary(Span::new(id, 46, 48), "moved here");
        let v = to_json(&d, &map);
        assert_eq!(v["code"], "E3040");
        assert_eq!(v["severity"], "error");
        assert_eq!(v["labels"][0]["line_start"], serde_json::Value::Null); // nested under "span"
        assert_eq!(v["labels"][0]["span"]["line_start"], 5);
        assert_eq!(v["labels"][0]["primary"], true);
        assert_eq!(v["labels"][1]["primary"], false);
        assert!(v["rendered"].as_str().unwrap().contains("error[E3040]"));
    }

    #[test]
    fn sink_counts_and_deny_warnings() {
        let (map, id) = fixture();
        let mut sink = Sink::new();
        sink.emit(Diagnostic::warning(codes::W0001, Span::new(id, 0, 2), "dangling doc comment"));
        assert!(!sink.has_errors());
        sink.deny_warnings = true;
        assert!(sink.has_errors());
        assert_eq!(sink.warning_count(), 1);
        assert!(sink.render(&map).contains("warning[W0001]"));
    }

    /// `[DIA-2]` — the article before a quoted name follows the word after it.
    #[test]
    fn articles_follow_the_next_word() {
        let fixed = |text: &str| articles(text.to_string());
        assert_eq!(fixed("a `owns Player` here"), "an `owns Player` here");
        assert_eq!(fixed("a `int`, a `Option[int]`, a `f32`, a `unsafe fn`"), "an `int`, an `Option[int]`, an `f32`, an `unsafe fn`");
        assert_eq!(fixed("an `String`, an `u8`, an `unit`, an `once fn`, an `Utf8Error`"), "a `String`, a `u8`, a `unit`, a `once fn`, a `Utf8Error`");
        assert_eq!(fixed("A `owns T` and a `[int; 3]` and `a` `x`"), "An `owns T` and a `[int; 3]` and `a` `x`");
        assert_eq!(fixed("a `SoA`, a `SIMD` lane, a `Rc`, an `Set`, a `Mat2`"), "an `SoA`, an `SIMD` lane, an `Rc`, a `Set`, a `Mat2`");
        assert_eq!(fixed("data `x`, extra `y`"), "data `x`, extra `y`");
        assert_eq!(fixed("a `n`, a `x`, an `b`, a `e`"), "an `n`, an `x`, a `b`, an `e`");
        assert_eq!(fixed("an `lto` value, a `list`"), "an `lto` value, a `list`");
    }
}
