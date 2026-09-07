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

use ember_span::{SourceMap, Span};

pub mod codes;
pub use codes::Code;

/// How loud a diagnostic is. Only `Error` and `Warning` reach the top level;
/// `Note` and `Help` exist as sub-diagnostics, and `Lint` is what `ember lint`
/// emits (L-codes).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
#[derive(Clone, Debug)]
pub struct Label {
    pub span: Span,
    pub message: Option<String>,
}

impl Label {
    pub fn new(span: Span, message: impl Into<String>) -> Label {
        Label { span, message: Some(message.into()) }
    }

    /// A label that only points, with no text of its own.
    pub fn bare(span: Span) -> Label {
        Label { span, message: None }
    }
}

/// A machine-applicable edit. `[DIA-1]` asks for one wherever the fix is
/// unambiguous; `ember fix` (v1.1) and editors consume these.
#[derive(Clone, Debug)]
pub struct Suggestion {
    pub message: String,
    /// Replacement text for each span, applied together.
    pub edits: Vec<(Span, String)>,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<Code>,
    pub message: String,
    pub primary: Label,
    pub secondary: Vec<Label>,
    pub helps: Vec<String>,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
}

impl Diagnostic {
    pub fn error(code: Code, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            code: Some(code),
            message: message.into(),
            primary: Label::bare(span),
            secondary: Vec::new(),
            helps: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn warning(code: Code, span: Span, message: impl Into<String>) -> Diagnostic {
        Diagnostic { severity: Severity::Warning, ..Diagnostic::error(code, span, message) }
    }

    /// Text on the primary caret itself, distinct from the header message.
    pub fn primary_label(mut self, text: impl Into<String>) -> Diagnostic {
        self.primary.message = Some(text.into());
        self
    }

    pub fn secondary(mut self, span: Span, text: impl Into<String>) -> Diagnostic {
        self.secondary.push(Label::new(span, text));
        self
    }

    pub fn help(mut self, text: impl Into<String>) -> Diagnostic {
        self.helps.push(text.into());
        self
    }

    pub fn note(mut self, text: impl Into<String>) -> Diagnostic {
        self.notes.push(text.into());
        self
    }

    pub fn suggest(mut self, message: impl Into<String>, span: Span, text: impl Into<String>) -> Diagnostic {
        self.suggestions.push(Suggestion {
            message: message.into(),
            edits: vec![(span, text.into())],
        });
        self
    }

    pub fn is_fatal(&self) -> bool {
        self.severity.is_fatal()
    }
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

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn error_count(&self) -> usize {
        self.error_count
    }

    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0 || (self.deny_warnings && self.warning_count > 0)
    }

    pub fn take(&mut self) -> Vec<Diagnostic> {
        self.error_count = 0;
        self.warning_count = 0;
        std::mem::take(&mut self.diagnostics)
    }

    /// Human rendering of everything collected, in emission order.
    pub fn render(&self, map: &SourceMap) -> String {
        let mut out = String::new();
        for d in &self.diagnostics {
            out.push_str(&render(d, map));
            out.push('\n');
        }
        out
    }

    /// `--json`: one JSON object per diagnostic, newline-delimited
    /// (`[CLI-1]`). Line-delimited rather than one array so that a consumer can
    /// stream them while the compiler is still running.
    pub fn render_json(&self, map: &SourceMap) -> String {
        let mut out = String::new();
        for d in &self.diagnostics {
            let _ = writeln!(out, "{}", to_json(d, map));
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Human rendering
// ---------------------------------------------------------------------------

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

    // Labels that can actually be shown: same file as the primary, resolvable.
    let primary_file = d.primary.span.file;
    let mut labels: Vec<(&Label, bool)> = Vec::new();
    if !d.primary.span.is_dummy() && map.get(primary_file).is_some() {
        labels.push((&d.primary, true));
    }
    for label in &d.secondary {
        if label.span.file == primary_file && map.get(label.span.file).is_some() {
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

    let file = map.file(primary_file);
    let mut rows: Vec<(u32, &Label, bool)> = labels
        .iter()
        .map(|&(label, is_primary)| (file.line_of(label.span.start), label, is_primary))
        .collect();
    rows.sort_by_key(|&(line, _, is_primary)| (line, !is_primary));

    let last_line = rows.iter().map(|&(l, _, _)| l).max().unwrap_or(1);
    let gutter = last_line.to_string().len();
    let pad = " ".repeat(gutter);

    let _ = writeln!(out, "{pad}--> {}", map.location(d.primary.span));
    let _ = writeln!(out, "{pad} |");

    let mut previous_line: Option<u32> = None;
    for &(line, label, is_primary) in &rows {
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

    let _ = writeln!(out, "{pad} |");
    for help in &d.helps {
        let _ = writeln!(out, "{pad} = help: {help}");
    }
    for suggestion in &d.suggestions {
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
        "message": d.message,
        "labels": labels,
        "helps": d.helps,
        "notes": d.notes,
        "suggestions": d.suggestions.iter().map(|s| json!({
            "message": s.message,
            "edits": s.edits.iter().map(|(span, text)| json!({
                "span": span_json(*span, map),
                "replacement": text,
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
}
