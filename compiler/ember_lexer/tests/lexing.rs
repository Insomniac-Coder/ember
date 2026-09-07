//! Lexer conformance (Part II). Each test names the rules it covers.

use ember_diag::Sink;
use ember_lexer::{FStrPart, Kw, Lit, Punct, Reserved, TokenKind, lex};
use ember_span::SourceMap;

struct Lexed {
    kinds: Vec<TokenKind>,
    diagnostics: Vec<String>,
    codes: Vec<String>,
}

fn run(src: &str) -> Lexed {
    let mut map = SourceMap::new();
    let text = ember_span::normalise(src.as_bytes()).expect("test source is UTF-8");
    let file = map.add("test.em", text.clone());
    let mut sink = Sink::new();
    let result = lex(file, &text, &mut sink);
    Lexed {
        kinds: result.tokens.into_iter().map(|t| t.kind).collect(),
        codes: sink.diagnostics().iter().filter_map(|d| d.code.map(|c| c.to_string())).collect(),
        diagnostics: sink.diagnostics().iter().map(|d| d.message.clone()).collect(),
    }
}

/// A compact one-line rendering, so a test can assert the whole stream.
fn shape(src: &str) -> String {
    run(src).kinds.iter().map(describe).collect::<Vec<_>>().join(" ")
}

fn describe(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Newline => "NL".to_string(),
        TokenKind::Indent => "IN".to_string(),
        TokenKind::Dedent => "DE".to_string(),
        TokenKind::Eof => "EOF".to_string(),
        TokenKind::Ident(s) => s.to_string(),
        TokenKind::RawIdent(s) => format!("r#{s}"),
        TokenKind::Keyword(k) => format!("kw:{}", k.as_str()),
        TokenKind::Reserved(r) => format!("res:{}", r.as_str()),
        TokenKind::Punct(p) => p.as_str().to_string(),
        TokenKind::DocComment(t) => format!("doc({t})"),
        TokenKind::Directive { name, value } => format!("directive({name}={value})"),
        TokenKind::Error => "ERR".to_string(),
        TokenKind::Lit(lit) => match lit {
            Lit::Int { value, suffix } => match suffix {
                Some(s) => format!("int({value}:{s:?})"),
                None => format!("int({value})"),
            },
            Lit::Float { value, suffix } => match suffix {
                Some(s) => format!("float({value}:{s:?})"),
                None => format!("float({value})"),
            },
            Lit::Char(c) => format!("char({c})"),
            Lit::Str(s) => format!("str({s:?})"),
            Lit::Bytes(b) => format!("bytes({b:?})"),
            Lit::CStr(b) => format!("cstr({b:?})"),
            Lit::FStr(parts) => {
                let inner: Vec<String> = parts
                    .iter()
                    .map(|p| match p {
                        FStrPart::Text(t) => format!("{t:?}"),
                        FStrPart::Expr { format_spec: Some(spec), .. } => format!("<expr:{spec}>"),
                        FStrPart::Expr { .. } => "<expr>".to_string(),
                    })
                    .collect();
                format!("f[{}]", inner.join(","))
            }
        },
    }
}

// -- indentation -------------------------------------------------------------

#[test]
fn indent_and_dedent_follow_the_python_algorithm() {
    // [LEX-5]
    let src = "fn a():\n    b()\n    c()\nfn d():\n    pass\n";
    assert_eq!(
        shape(src),
        "kw:fn a ( ) : NL IN b ( ) NL c ( ) NL DE kw:fn d ( ) : NL IN kw:pass NL DE EOF"
    );
}

#[test]
fn nested_blocks_close_one_dedent_per_level() {
    // [LEX-5]
    let src = "a:\n    b:\n        c\nd\n";
    assert_eq!(shape(src), "a : NL IN b : NL IN c NL DE DE d NL EOF");
}

#[test]
fn an_indentation_not_on_the_stack_is_e0003() {
    // [LEX-5]
    let src = "a:\n        b\n    c\n";
    let out = run(src);
    assert!(out.codes.contains(&"E0003".to_string()), "{:?}", out.diagnostics);
    // The stream still closes cleanly, so the parser gets a usable shape.
    assert!(out.kinds.last() == Some(&TokenKind::Eof));
}

#[test]
fn a_tab_in_indentation_is_e0002_and_lexing_continues() {
    // [LEX-4]
    let src = "a:\n\tb\n";
    let out = run(src);
    assert_eq!(out.codes, vec!["E0002"]);
    assert_eq!(shape(src), "a : NL IN b NL DE EOF");
}

#[test]
fn blank_and_comment_only_lines_do_not_affect_indentation() {
    // [LEX-8]
    let src = "a:\n\n    # note\n\n    b\n";
    assert_eq!(shape(src), "a : NL IN b NL DE EOF");
}

#[test]
fn a_trailing_dedent_is_emitted_at_end_of_file() {
    // [LEX-5]
    assert_eq!(shape("a:\n    b\n"), "a : NL IN b NL DE EOF");
}

#[test]
fn a_file_without_a_final_newline_still_closes() {
    assert_eq!(shape("a:\n    b"), "a : NL IN b NL DE EOF");
}

// -- line joining ------------------------------------------------------------

#[test]
fn newlines_inside_brackets_are_not_line_ends() {
    // [LEX-6]
    let src = "f(\n    a,\n    b,\n)\n";
    assert_eq!(shape(src), "f ( a , b , ) NL EOF");
}

#[test]
fn a_backslash_joins_two_physical_lines() {
    // [LEX-7]
    let src = "a = b \\\n    + c\n";
    assert_eq!(shape(src), "a = b + c NL EOF");
}

// -- comments ----------------------------------------------------------------

#[test]
fn an_ordinary_comment_is_not_a_token() {
    let src = "a  # trailing\n";
    assert_eq!(shape(src), "a NL EOF");
}

#[test]
fn a_doc_comment_is_a_token() {
    // [LEX-11] — the parser attaches it to the next declaration.
    assert_eq!(shape("## docs\nfn a(): pass\n"), "doc(docs) kw:fn a ( ) : kw:pass NL EOF");
}

#[test]
fn a_doc_comment_inside_a_block_lands_after_the_indent() {
    // A comment-only line does not change the indent stack ([LEX-8]), so the
    // doc token is held back until the block has been opened. Otherwise it
    // would arrive before INDENT and read as documentation of the block.
    let src = "class A:\n    ## the field\n    x: i32\n";
    assert_eq!(shape(src), "kw:class A : NL IN doc(the field) x : i32 NL DE EOF");
}

#[test]
fn there_are_no_block_comments() {
    // [LEX-10] — `###` is a doc comment whose body starts with `#`.
    assert_eq!(shape("### x\na\n"), "doc(# x) a NL EOF");
}

#[test]
fn a_directive_is_recognised_on_the_first_line_only() {
    let out = run("#! language \"0.2\"\nfn a(): pass\n");
    assert!(matches!(out.kinds[0], TokenKind::Directive { .. }));
    assert_eq!(describe(&out.kinds[0]), "directive(language=0.2)");
    // Elsewhere `#!` is an ordinary comment, which is what lets test files
    // carry `#! error[...]` annotations ([TST-1]).
    assert_eq!(shape("a\n#! error[E3040]: nope\nb\n"), "a NL b NL EOF");
}

// -- identifiers and keywords ------------------------------------------------

#[test]
fn keywords_are_distinct_from_identifiers() {
    assert_eq!(shape("fn owned mut self Self\n"), "kw:fn kw:owned kw:mut kw:self kw:Self NL EOF");
}

#[test]
fn future_keywords_lex_as_reserved() {
    // [Part II §4] — so that using one is a clear error, not a parse failure.
    let out = run("async\n");
    assert_eq!(out.kinds[0], TokenKind::Reserved(Reserved::Async));
}

#[test]
fn contextual_keywords_are_identifiers() {
    // [LEX-15]
    assert_eq!(shape("abstract final lazy test bench\n"), "abstract final lazy test bench NL EOF");
}

#[test]
fn a_raw_identifier_carries_a_keyword_as_a_name() {
    // [LEX-14] — needed for C fields called `type`.
    assert_eq!(shape("r#match\n"), "r#match NL EOF");
    assert_eq!(shape("x.r#type\n"), "x . r#type NL EOF");
}

#[test]
fn underscore_is_the_discard() {
    // [LEX-13]
    let out = run("_\n");
    match out.kinds[0] {
        TokenKind::Ident(s) => assert!(s.is_discard()),
        ref other => panic!("expected an identifier, got {other:?}"),
    }
}

#[test]
fn identifiers_are_nfc_normalised() {
    // [LEX-12] — "é" as one scalar and as "e" + combining acute are the same
    // identifier.
    let composed = run("café\n").kinds[0].clone();
    let decomposed = run("cafe\u{0301}\n").kinds[0].clone();
    assert_eq!(composed, decomposed);
}

// -- numbers -----------------------------------------------------------------

#[test]
fn integer_bases_and_underscores() {
    assert_eq!(shape("10_000\n"), "int(10000) NL EOF");
    assert_eq!(shape("0xFF_FF\n"), "int(65535) NL EOF");
    assert_eq!(shape("0o77\n"), "int(63) NL EOF");
    assert_eq!(shape("0b1010\n"), "int(10) NL EOF");
}

#[test]
fn suffixes_are_recorded_not_applied() {
    // [LEX-16] — an unsuffixed literal stays untyped until the type checker.
    assert_eq!(shape("42u8\n"), "int(42:U8) NL EOF");
    assert_eq!(shape("1.5f64\n"), "float(1.5:F64) NL EOF");
}

#[test]
fn a_trailing_dot_is_a_float_only_when_no_identifier_follows() {
    // [LEX-18]
    assert_eq!(shape("1.\n"), "float(1) NL EOF");
    assert_eq!(shape("1.abs()\n"), "int(1) . abs ( ) NL EOF");
}

#[test]
fn a_range_after_an_integer_is_not_a_float() {
    // [LEX-18] with [LEX-21]: `0..count` must not eat the first dot.
    assert_eq!(shape("0..count\n"), "int(0) .. count NL EOF");
    assert_eq!(shape("0..=n\n"), "int(0) ..= n NL EOF");
}

#[test]
fn exponents_make_a_float() {
    assert_eq!(shape("1e3\n"), "float(1000) NL EOF");
    assert_eq!(shape("1.5e-2\n"), "float(0.015) NL EOF");
}

#[test]
fn an_unknown_numeric_suffix_is_reported() {
    let out = run("1foo\n");
    assert!(!out.codes.is_empty(), "expected a diagnostic for `1foo`");
}

// -- strings and characters --------------------------------------------------

#[test]
fn escapes_are_resolved() {
    assert_eq!(shape(r#""a\nb\t\x41\u{1F600}""#), "str(\"a\\nb\\t\u{41}\u{1F600}\") NL EOF");
}

#[test]
fn a_raw_string_keeps_its_backslashes() {
    assert_eq!(shape(r##"r"C:\path\n""##), "str(\"C:\\\\path\\\\n\") NL EOF");
    assert_eq!(shape(r###"r#"has "quotes""#"###), "str(\"has \\\"quotes\\\"\") NL EOF");
}

#[test]
fn byte_and_c_strings_have_their_own_types() {
    assert_eq!(shape(r#"b"AB""#), "bytes([65, 66]) NL EOF");
    assert_eq!(shape(r#"c"AB""#), "cstr([65, 66]) NL EOF");
}

#[test]
fn a_c_string_may_not_contain_an_interior_nul() {
    let out = run(r#"c"a\0b""#);
    assert!(!out.codes.is_empty(), "expected a diagnostic: {:?}", out.diagnostics);
}

#[test]
fn a_triple_quoted_string_strips_common_indentation() {
    let src = "x = \"\"\"\n    one\n      two\n    \"\"\"\n";
    assert_eq!(shape(src), "x = str(\"one\\n  two\\n\") NL EOF");
}

#[test]
fn a_character_literal_holds_one_scalar_value() {
    assert_eq!(shape("'a'\n"), "char(a) NL EOF");
    assert_eq!(shape(r"'\n'"), "char(\n) NL EOF");
    let out = run("'ab'\n");
    assert!(!out.codes.is_empty(), "expected a diagnostic for a two-character literal");
}

#[test]
fn a_named_lifetime_is_e0007() {
    // [LT-6] — reserved for v2, with a message that names the alternative.
    let out = run("fn f['a]():\n    pass\n");
    assert!(out.codes.contains(&"E0007".to_string()), "{:?}", out.diagnostics);
}

// -- f-strings ---------------------------------------------------------------

#[test]
fn an_f_string_splits_into_text_and_expressions() {
    // [LEX-19]
    assert_eq!(shape(r#"f"a{x}b""#), "f[\"a\",<expr>,\"b\"] NL EOF");
}

#[test]
fn an_f_string_carries_its_format_spec() {
    assert_eq!(shape(r#"f"{v:.3}""#), "f[<expr:.3>] NL EOF");
    assert_eq!(shape(r#"f"{v:?}""#), "f[<expr:?>] NL EOF");
}

#[test]
fn doubled_braces_are_literal() {
    // [LEX-19]
    assert_eq!(shape(r#"f"{{x}}""#), "f[\"{x}\"] NL EOF");
}

#[test]
fn a_path_separator_inside_a_hole_is_not_a_format_spec() {
    assert_eq!(shape(r#"f"{Shape::Circle}""#), "f[<expr>] NL EOF");
}

#[test]
fn brackets_inside_a_hole_are_matched() {
    assert_eq!(shape(r#"f"{a[i].m(1, 2)}""#), "f[<expr>] NL EOF");
}

// -- punctuation -------------------------------------------------------------

#[test]
fn punctuation_uses_maximal_munch() {
    // [LEX-21]
    assert_eq!(shape("a **= b\n"), "a **= b NL EOF");
    assert_eq!(shape("a ** b\n"), "a ** b NL EOF");
    assert_eq!(shape("a <<= b\n"), "a <<= b NL EOF");
    assert_eq!(shape("a <= b\n"), "a <= b NL EOF");
    assert_eq!(shape("a?.b\n"), "a ?. b NL EOF");
    assert_eq!(shape("A::B\n"), "A :: B NL EOF");
    assert_eq!(shape("a -> b => c\n"), "a -> b => c NL EOF");
}

#[test]
fn an_unrecognised_character_becomes_an_error_token() {
    // [II.7] — the lexer never fails fatally.
    let out = run("a $ b\n");
    assert_eq!(shape("a $ b\n"), "a ERR b NL EOF");
    assert!(!out.codes.is_empty());
}

// -- a whole program ---------------------------------------------------------

#[test]
fn the_milestone_program_lexes() {
    let src = "\
struct Vec3:
    x: f32
    y: f32
    z: f32

fn add(a: Vec3, b: Vec3) -> Vec3:
    return Vec3(a.x + b.x, a.y + b.y, a.z + b.z)

fn main():
    c = add(Vec3(1, 2, 3), Vec3(4, 5, 6))
    println(c.x)
";
    let out = run(src);
    assert!(out.codes.is_empty(), "unexpected diagnostics: {:?}", out.diagnostics);
    assert_eq!(out.kinds.last(), Some(&TokenKind::Eof));
    assert_eq!(out.kinds[0], TokenKind::Keyword(Kw::Struct));
    // Every block opened is closed.
    let indents = out.kinds.iter().filter(|k| **k == TokenKind::Indent).count();
    let dedents = out.kinds.iter().filter(|k| **k == TokenKind::Dedent).count();
    assert_eq!(indents, 3);
    assert_eq!(indents, dedents);
    assert!(out.kinds.contains(&TokenKind::Punct(Punct::Arrow)));
}
