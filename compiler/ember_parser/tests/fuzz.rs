//! `[IMP-11]` — the lexer and the parser, fuzzed. Each test runs a fixed number of inputs made
//! from a fixed seed, so a failure is the same on every run and every machine; the case's number
//! and input are printed with it. `EMBER_FUZZ_CASES` multiplies the number of cases and
//! `EMBER_FUZZ_SEED` picks another seed, for a longer search by hand; `EMBER_FUZZ_TRACE` prints
//! each case before it runs.
//!
//! The properties: neither the lexer nor the parser ever panics; every token lies inside its
//! source, in order; lexing is deterministic; every diagnostic has a code; a generated
//! well-formed program parses without a diagnostic, and the formatter's text of it parses to the
//! same tree and formats to itself.

use ember_diag::Sink;
use ember_span::{FileId, SourceMap};

/// SplitMix64: a small, well-mixed generator; the cases need no more.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }
}

fn seed() -> u64 {
    std::env::var("EMBER_FUZZ_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(0x0e4b_e2f0_9d11_0003)
}

fn cases(base: usize) -> usize {
    base * std::env::var("EMBER_FUZZ_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(1)
}

/// The compiler's own stack (`ember_driver`'s `COMPILER_STACK`): random nesting goes deep.
fn on_compiler_stack(run: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(run)
        .expect("a thread")
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
}

/// Lexes and parses `source`, checking what holds of any input. Returns the module, its dump and
/// whether there was an error.
fn lex_and_parse(case: &str, source: &str) -> (ember_ast::Module, Vec<ember_lexer::Comment>, bool) {
    // An abort (a failed allocation, a stack overflow) cannot be caught: `EMBER_FUZZ_TRACE`
    // prints each case before it runs, so the last one printed is the one that ended the run.
    if std::env::var_os("EMBER_FUZZ_TRACE").is_some() {
        eprintln!("{case}: {source:?}");
    }
    let mut map = SourceMap::new();
    let file = map.add("<fuzz>", source.to_string());
    let mut sink = Sink::new();
    let lexed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ember_lexer::lex(file, source, &mut Sink::new())))
        .unwrap_or_else(|_| panic!("{case}: the lexer panicked on:\n{source:?}"));
    // Every token inside the source; those of the text in order. (Documentation comments are
    // held back to stand before what they document, after a line's indentation tokens and a
    // directive.)
    let mut end = 0u32;
    for token in &lexed.tokens {
        assert!(token.span.start <= token.span.end && token.span.end as usize <= source.len(),
            "{case}: a token {:?} at {:?} lies outside its {}-byte source:\n{source:?}", token.kind, token.span, source.len());
        use ember_lexer::TokenKind::{Dedent, DocComment, Eof, Indent, Newline};
        if matches!(token.kind, Newline | Indent | Dedent | Eof | DocComment(_)) {
            continue;
        }
        assert!(token.span.start >= end,
            "{case}: a token {:?} at {:?} starts before the previous one ends ({end}):\n{source:?}", token.kind, token.span);
        end = token.span.end;
    }
    // Deterministic, with the same diagnostics.
    let again = ember_lexer::lex(file, source, &mut sink);
    assert_eq!(format!("{:?}", lexed.tokens), format!("{:?}", again.tokens), "{case}: lexing twice differed:\n{source:?}");
    let module = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut sink = Sink::new();
        ember_parser::parse(file, source, again.tokens.clone(), &mut sink)
    }))
    .unwrap_or_else(|_| panic!("{case}: the parser panicked on:\n{source:?}"));
    let module_again = ember_parser::parse(file, source, again.tokens, &mut sink);
    assert_eq!(ember_ast::dump(&module), ember_ast::dump(&module_again), "{case}: parsing twice differed:\n{source:?}");
    for diagnostic in sink.diagnostics() {
        assert!(diagnostic.code.is_some(), "{case}: a diagnostic without a code ({}):\n{source:?}", diagnostic.message);
    }
    (module_again, again.comments, sink.has_errors())
}

/// Pieces of text the lexer has rules for, and their near misses.
const FRAGMENTS: &[&str] = &[
    "x", "y1", "_", "_t", "café", "π", "Ünïcode", "Self", "self", "None", "True", "def",
    "0", "42", "1_000", "0x1F", "0xg", "0b101", "0b2", "0o17", "1.5", "1e9", "2.5e-3", "1.", ".5", "1__2", "1_",
    "99999999999999999999999", "0u8", "255u8", "1i64", "3.0f32", "1e", "1e+", "0x", "007",
    "\"a\"", "\"\\n\\t\\\\\"", "\"\\u{1F600}\"", "\"\\q\"", "\"unterminated", "\"\"", "\"\\", "\"multi\nline\"",
    "f\"{x}\"", "f\"{x:>8}\"", "f\"{x!r}\"", "f\"{{}}\"", "f\"{\"", "f\"}\"", "f\"{f\"{y}\"}\"", "f\"{x + 1:08.3f}\"",
    "r\"raw\\n\"", "b\"bytes\"", "c\"cstr\"", "'c'", "'\\n'", "'ab'", "''", "'", "'\\u{263A}'",
    "# comment", "## doc", "#$ error[E0100]: x", "#! language \"0.9.9\"", "#! threads main", "#!", "#",
    " ", "  ", "    ", "\t", "\n", "\r\n", "\n    ", "\n        ", "\n\t", "\\\n", "\r",
    "\u{200B}", "\u{FEFF}", "😀", "\u{0}", "\u{7F}", "\u{202E}", "é", "ﬁ",
    "(", ")", "[", "]", "{", "}", ":", "::", ";", "`", "$", "?", "!", "@", "\\", "->", "=>", "..", "..=", "...",
    "as?", "as!", "?.", "**=", "//=", "<<=", ">>=",
];

/// A random string: fragments, keywords and operators, joined with or without space.
fn random_text(rng: &mut Rng) -> String {
    let keywords: Vec<&str> = ember_lexer::Kw::ALL.iter().map(|kw| kw.as_str()).collect();
    let reserved: Vec<&str> = ember_lexer::Reserved::ALL.iter().map(|kw| kw.as_str()).collect();
    let puncts: Vec<&str> = ember_lexer::Punct::by_length_desc().into_iter().map(|(text, _)| text).collect();
    let mut text = String::new();
    for _ in 0..rng.below(40) + 1 {
        let piece = match rng.below(10) {
            0..=3 => *rng.pick(FRAGMENTS),
            4..=5 => *rng.pick(&keywords),
            6 => *rng.pick(&reserved),
            7 => *rng.pick(ember_lexer::CONTEXTUAL_KEYWORDS),
            _ => *rng.pick(&puncts),
        };
        text.push_str(piece);
        if rng.chance(60) {
            text.push(' ');
        }
    }
    text
}

#[test]
fn the_lexer_and_parser_survive_any_text() {
    on_compiler_stack(|| {
        let mut rng = Rng::new(seed());
        for case in 0..cases(3000) {
            let text = random_text(&mut rng);
            lex_and_parse(&format!("[IMP-11] text case {case}"), &text);
        }
    });
}

// ---------------------------------------------------------------------------
// Well-formed programs
// ---------------------------------------------------------------------------

/// A generator of programs the grammar of Part III produces (not necessarily well typed).
struct Program<'a> {
    rng: &'a mut Rng,
    out: String,
}

const NAMES: &[&str] = &["a", "b", "count", "total", "xs", "item", "name", "p", "q", "value", "élan"];
const TYPES: &[&str] = &["int", "i32", "u8", "float", "f32", "bool", "str", "String", "char", "Array[int]",
    "Map[str, int]", "Option[int]", "Result[int, str]", "Span[u8]", "ref int", "ref mut Array[i32]", "(int, bool)",
    "fn(int) -> int", "[i32; 4]", "Box[dyn Shape]", "Self"];

impl Program<'_> {
    fn name(&mut self) -> &'static str {
        self.rng.pick(NAMES)
    }

    fn ty(&mut self) -> &'static str {
        self.rng.pick(TYPES)
    }

    fn line(&mut self, indent: usize, text: &str) {
        for _ in 0..indent {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn literal(&mut self) -> String {
        match self.rng.below(9) {
            0 => self.rng.below(1000).to_string(),
            1 => format!("{}.{}", self.rng.below(100), self.rng.below(100)),
            2 => "true".to_string(),
            3 => "false".to_string(),
            4 => "\"text\"".to_string(),
            5 => format!("f\"{{{}}} and {{{} + 1}}\"", self.name(), self.name()),
            6 => "'c'".to_string(),
            7 => "0x2A".to_string(),
            _ => "1_000_000".to_string(),
        }
    }

    /// An expression, at most `depth` deep.
    fn expr(&mut self, depth: usize) -> String {
        if depth == 0 || self.rng.chance(25) {
            return if self.rng.chance(50) { self.literal() } else { self.name().to_string() };
        }
        let d = depth - 1;
        match self.rng.below(24) {
            0..=3 => {
                let op = *self.rng.pick(&["+", "-", "*", "//", "%", "**", "/", "&", "|", "^", "<<", ">>"]);
                format!("{} {op} {}", self.operand(d), self.operand(d))
            }
            4 => {
                let op = *self.rng.pick(&["==", "!=", "<", ">", "<=", ">="]);
                format!("{} {op} {}", self.operand(d), self.operand(d))
            }
            5 => format!("{} < {} <= {}", self.operand(d), self.operand(d), self.operand(d)),
            6 => format!("{} and {}", self.operand(d), self.operand(d)),
            7 => format!("not {}", self.operand(d)),
            8 => format!("-{}", self.operand(d)),
            9 => format!("{}({})", self.name(), self.args(d)),
            10 => format!("{}.{}({})", self.operand(d), self.name(), self.args(d)),
            // `[GRM-8a]`: after `fn` (and `ref`) the brackets hold a type, so an index that
            // starts so is parenthesised.
            11 => {
                let index = self.expr(d);
                let index = if index.starts_with("fn(") { format!("({index})") } else { index };
                format!("{}[{index}]", self.operand(d))
            }
            12 => format!("{}.{}", self.operand(d), self.name()),
            13 => format!("[{}]", self.list(d)),
            14 => format!("{{\"k\": {}, \"j\": {}}}", self.expr(d), self.expr(d)),
            15 => format!("({}, {})", self.expr(d), self.expr(d)),
            16 => format!("fn({}) => {}", self.name(), self.expr(d)),
            17 => format!("{} if {} else {}", self.operand(d), self.operand(d), self.operand(d)),
            18 => format!("[{} for {} in {} if {}]", self.expr(d), self.name(), self.operand(d), self.operand(d)),
            19 => format!("{} as {}", self.operand(d), self.rng.pick(&["int", "u8", "f64", "i64"])),
            20 => format!("{}?", self.operand(d)),
            21 => format!("{} is None", self.operand(d)),
            22 => format!("{} in {}", self.operand(d), self.operand(d)),
            _ => format!("{}..{}", self.operand(d), self.operand(d)),
        }
    }

    /// An expression that can stand as an operand: parenthesised unless it is an atom.
    fn operand(&mut self, depth: usize) -> String {
        let expr = self.expr(depth);
        if expr.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
            expr
        } else {
            format!("({expr})")
        }
    }

    fn args(&mut self, depth: usize) -> String {
        let count = self.rng.below(4);
        let mut args: Vec<String> = (0..count).map(|_| self.expr(depth)).collect();
        if self.rng.chance(30) {
            let name = self.name();
            args.push(format!("{name}={}", self.expr(depth)));
        }
        args.join(", ")
    }

    fn list(&mut self, depth: usize) -> String {
        let count = self.rng.below(4);
        (0..count).map(|_| self.expr(depth)).collect::<Vec<_>>().join(", ")
    }

    fn pattern(&mut self) -> String {
        match self.rng.below(7) {
            0 => "_".to_string(),
            1 => self.rng.below(10).to_string(),
            2 => format!("Some({})", self.name()),
            3 => "None".to_string(),
            4 => format!("Shape.Circle({})", self.name()),
            5 => format!("({}, _)", self.name()),
            _ => format!("{}..={}", self.rng.below(5), 5 + self.rng.below(5)),
        }
    }

    /// A block's statements at `indent`, at least one.
    fn block(&mut self, indent: usize, depth: usize) {
        for _ in 0..self.rng.below(4) + 1 {
            self.statement(indent, depth);
        }
    }

    fn statement(&mut self, indent: usize, depth: usize) {
        let d = depth.saturating_sub(1);
        let compound = depth > 0;
        match self.rng.below(if compound { 16 } else { 7 }) {
            0 => {
                let (name, value) = (self.name(), self.expr(3));
                self.line(indent, &format!("{name} = {value}"));
            }
            1 => {
                let (name, ty, value) = (self.name(), self.ty(), self.expr(3));
                self.line(indent, &format!("{name}: {ty} = {value}"));
            }
            2 => {
                let (name, op, value) = (self.name(), *self.rng.pick(&["+=", "-=", "*=", "//=", "|="]), self.expr(2));
                self.line(indent, &format!("{name} {op} {value}"));
            }
            3 => {
                let call = format!("{}({})", self.name(), self.args(2));
                self.line(indent, &call);
            }
            4 => {
                let value = self.expr(3);
                self.line(indent, &format!("return {value}"));
            }
            5 => self.line(indent, "pass"),
            6 => {
                let (a, b, value) = (self.name(), self.name(), self.expr(2));
                self.line(indent, &format!("{a}, {b} = {value}"));
            }
            7 | 8 => {
                let condition = self.expr(3);
                self.line(indent, &format!("if {condition}:"));
                self.block(indent + 1, d);
                if self.rng.chance(40) {
                    let condition = self.expr(2);
                    self.line(indent, &format!("elif {condition}:"));
                    self.block(indent + 1, d);
                }
                if self.rng.chance(50) {
                    self.line(indent, "else:");
                    self.block(indent + 1, d);
                }
            }
            9 => {
                let condition = self.expr(3);
                self.line(indent, &format!("while {condition}:"));
                self.block(indent + 1, d);
            }
            10 => {
                let (name, iterable) = (self.name(), self.expr(2));
                self.line(indent, &format!("for {name} in {iterable}:"));
                self.block(indent + 1, d);
                if self.rng.chance(20) {
                    self.line(indent, "else:");
                    self.block(indent + 1, d);
                }
            }
            11 => {
                let scrutinee = self.expr(2);
                self.line(indent, &format!("match {scrutinee}:"));
                for _ in 0..self.rng.below(3) + 1 {
                    let pattern = self.pattern();
                    self.line(indent + 1, &format!("{pattern}:"));
                    self.block(indent + 2, d);
                }
            }
            12 => {
                self.line(indent, "defer:");
                self.block(indent + 1, d);
            }
            13 => {
                let (name, ty) = (self.name(), self.ty());
                self.line(indent, &format!("fn {name}({name}: {ty}) -> {ty}:"));
                self.block(indent + 1, d);
            }
            14 => {
                let value = self.expr(2);
                self.line(indent, &format!("with f = {value}:"));
                self.block(indent + 1, d);
            }
            _ => {
                let label = self.name();
                self.line(indent, &format!("{label}: while true:"));
                self.line(indent + 1, &format!("break {label}"));
            }
        }
    }

    fn item(&mut self) {
        match self.rng.below(7) {
            0 | 1 | 2 => {
                let name = self.name();
                let params: Vec<String> = (0..self.rng.below(3)).map(|i| format!("p{i}: {}", self.ty())).collect();
                let ret = if self.rng.chance(50) { format!(" -> {}", self.ty()) } else { String::new() };
                if self.rng.chance(20) {
                    self.line(0, "@inline");
                }
                let generics = if self.rng.chance(20) { "[T: Eq + Hash]" } else { "" };
                self.line(0, &format!("fn {name}{generics}({}){ret}:", params.join(", ")));
                self.block(1, 3);
            }
            3 => {
                if self.rng.chance(40) {
                    self.line(0, "@derive(Copy)");
                }
                let n = self.rng.below(100);
                self.line(0, &format!("struct S{n}:"));
                for i in 0..self.rng.below(3) + 1 {
                    let ty = self.ty();
                    self.line(1, &format!("f{i}: {ty}"));
                }
            }
            4 => {
                let n = self.rng.below(100);
                self.line(0, &format!("enum E{n}:"));
                self.line(1, "First");
                self.line(1, "Second(int, str)");
                self.line(1, "Third = 7");
            }
            5 => {
                let n = self.rng.below(100);
                self.line(0, &format!("class C{n}:"));
                let ty = self.ty();
                self.line(1, &format!("field: {ty} = 0"));
                self.line(1, "fn method(mut self, other: int) -> int:");
                self.block(2, 2);
            }
            _ => {
                let n = self.rng.below(100);
                self.line(0, &format!("interface I{n}:"));
                self.line(1, "fn area(self) -> float");
                self.line(1, "type Item");
            }
        }
    }
}

fn well_formed_program(rng: &mut Rng) -> String {
    let mut program = Program { rng, out: String::new() };
    if program.rng.chance(30) {
        program.line(0, "import std.math");
    }
    if program.rng.chance(30) {
        program.line(0, "from std.collections import Map, Set");
    }
    for _ in 0..program.rng.below(3) + 1 {
        program.item();
    }
    program.out
}

/// The tree's dump with its imports in one order: the formatter sorts them (`[FMT-1]`), which
/// changes no meaning.
fn dump_with_sorted_imports(module: &ember_ast::Module) -> String {
    let dump = ember_ast::dump(module);
    let (mut imports, rest): (Vec<&str>, Vec<&str>) =
        dump.lines().partition(|line| line.starts_with("  Import ") || line.starts_with("  From "));
    imports.sort();
    imports.into_iter().chain(rest).collect::<Vec<_>>().join("\n")
}

/// The formatter's text of a parsed program, parsed again, is the same tree, and formats to itself.
fn round_trips(case: &str, source: &str) {
    let (module, comments, failed) = lex_and_parse(case, source);
    assert!(!failed, "{case}: a well-formed program did not parse:\n{source}\n{}", render(source));
    let formatted = ember_fmt::format(&module, source, &comments);
    let (again, again_comments, failed) = lex_and_parse(case, &formatted);
    assert!(!failed, "{case}: the formatter's text did not parse:\n{formatted}\n{}\nfrom:\n{source}", render(&formatted));
    assert_eq!(dump_with_sorted_imports(&module), dump_with_sorted_imports(&again),
        "{case}: the formatter changed the program:\n{source}\nbecame:\n{formatted}");
    assert_eq!(formatted, ember_fmt::format(&again, &formatted, &again_comments), "{case}: formatting is not idempotent:\n{formatted}");
}

fn render(source: &str) -> String {
    let mut map = SourceMap::new();
    let file: FileId = map.add("<fuzz>", source.to_string());
    let mut sink = Sink::new();
    let lexed = ember_lexer::lex(file, source, &mut sink);
    ember_parser::parse(file, source, lexed.tokens, &mut sink);
    sink.render(&map)
}

#[test]
fn generated_programs_parse_and_round_trip_through_the_formatter() {
    on_compiler_stack(|| {
        let mut rng = Rng::new(seed() ^ 0x5157);
        for case in 0..cases(600) {
            let program = well_formed_program(&mut rng);
            round_trips(&format!("[IMP-11] program case {case}"), &program);
        }
    });
}

/// A well-formed program with a few of its tokens deleted, repeated, swapped or replaced: the
/// mistakes of a program being typed, which an editor hands the parser on every keystroke.
#[test]
fn mutated_programs_never_break_the_lexer_or_parser() {
    on_compiler_stack(|| {
        let mut rng = Rng::new(seed() ^ 0xa11e);
        for case in 0..cases(2000) {
            let program = well_formed_program(&mut rng);
            let mut sink = Sink::new();
            let tokens = ember_lexer::lex(FileId(0), &program, &mut sink).tokens;
            let spans: Vec<(usize, usize)> = tokens.iter()
                .filter(|token| token.span.end > token.span.start)
                .map(|token| (token.span.start as usize, token.span.end as usize))
                .collect();
            if spans.is_empty() {
                continue;
            }
            let mut text = program.clone();
            // Edits from the end, so earlier spans stay where they are.
            let mut edits: Vec<(usize, usize, String)> = (0..rng.below(3) + 1).map(|_| {
                let (start, end) = *rng.pick(&spans);
                let replacement = match rng.below(4) {
                    0 => String::new(),
                    1 => program[start..end].repeat(2),
                    2 => rng.pick(FRAGMENTS).to_string(),
                    _ => format!("{} ", rng.pick(&["(", "[", ":", "\n    ", "fn", "=", ","])),
                };
                (start, end, replacement)
            }).collect();
            edits.sort_by(|a, b| b.0.cmp(&a.0));
            edits.dedup_by_key(|edit| edit.0);
            let mut last = usize::MAX;
            for (start, end, replacement) in edits {
                if end > last {
                    continue;
                }
                text.replace_range(start..end, &replacement);
                last = start;
            }
            lex_and_parse(&format!("[IMP-11] mutation case {case}"), &text);
        }
    });
}
