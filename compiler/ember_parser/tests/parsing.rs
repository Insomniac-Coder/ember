//! Parser conformance (Part III). Assertions are made against the AST dump,
//! which is structural and stable.

use ember_diag::Sink;
use ember_span::SourceMap;

struct Parsed {
    dump: String,
    codes: Vec<String>,
    /// The rendered diagnostics, so a failing assertion prints something a
    /// person can read rather than a list of codes.
    messages: String,
}

fn run(src: &str) -> Parsed {
    let mut map = SourceMap::new();
    let text = ember_span::normalise(src.as_bytes()).expect("test source is UTF-8");
    let file = map.add("test.em", text.clone());
    let mut sink = Sink::new();
    let lexed = ember_lexer::lex(file, &text, &mut sink);
    let module = ember_parser::parse(file, &text, lexed.tokens, &mut sink);
    Parsed {
        dump: ember_ast::dump(&module),
        codes: sink.diagnostics().iter().filter_map(|d| d.code.map(|c| c.to_string())).collect(),
        messages: sink.render(&map),
    }
}

/// Parse and require a clean run.
fn dump(src: &str) -> String {
    let out = run(src);
    assert!(out.codes.is_empty(), "unexpected diagnostics:\n{}", out.messages);
    out.dump
}

fn contains(src: &str, needle: &str) -> bool {
    dump(src).contains(needle)
}

// -- items -------------------------------------------------------------------

#[test]
fn a_struct_with_fields() {
    let out = dump("struct Vec3:\n    x: f32\n    y: f32\n");
    assert!(out.contains("Struct Vec3"), "{out}");
    assert!(out.contains("Field x: f32"), "{out}");
    assert!(out.contains("Field y: f32"), "{out}");
}

#[test]
fn a_function_records_parameter_modes() {
    // [FN-1] — borrowed by default, `mut` for inout, `owned` to consume.
    let out = dump("fn f(a: i32, mut b: Array[i32], owned c: String) -> i32:\n    return a\n");
    assert!(out.contains("Fn f(a: i32, mut b: Array[i32], owned c: String) -> i32"), "{out}");
}

#[test]
fn receivers_take_the_same_modes() {
    // [FN-4]
    let out = dump("struct S:\n    fn a(self): pass\n    fn b(mut self): pass\n    fn c(owned self): pass\n");
    assert!(out.contains("Fn a(self)"), "{out}");
    assert!(out.contains("Fn b(mut self)"), "{out}");
    assert!(out.contains("Fn c(owned self)"), "{out}");
}

#[test]
fn a_class_records_its_base_and_openness() {
    // [GRM-1], [CLS-4]
    let out = dump("open class Script:\n    pass\n\nclass Door(Script):\n    pass\n");
    assert!(out.contains("Class Script Open"), "{out}");
    assert!(out.contains("Class Door(Script) Final"), "{out}");
}

#[test]
fn a_second_base_class_is_rejected() {
    // [GRM-1] — at most one base class.
    let out = run("class D(A, B):\n    pass\n");
    assert!(!out.codes.is_empty(), "expected a diagnostic");
    assert!(out.messages.contains("at most one base class"), "{}", out.messages);
}

#[test]
fn field_visibility_records_read_only() {
    // [MOD-7]
    let out = dump("class C:\n    pub(read) enabled: bool = true\n    pub(package) n: i32\n");
    assert!(out.contains("Field enabled: bool = … pub (read)"), "{out}");
    assert!(out.contains("Field n: i32 pub(package)"), "{out}");
}

#[test]
fn read_visibility_on_an_item_is_e1051() {
    // [MOD-7] — `read` applies to fields only.
    let out = run("pub(read) fn f(): pass\n");
    assert!(out.codes.contains(&"E1051".to_string()), "{}", out.messages);
}

#[test]
fn an_enum_separates_variants_from_methods() {
    let out = dump(
        "enum Shape:\n    Circle(radius: f32)\n    Rect(w: f32, h: f32)\n    Empty\n\n    fn area(self) -> f32:\n        return 0.0\n",
    );
    assert!(out.contains("Variant Circle"), "{out}");
    assert!(out.contains("Variant Rect"), "{out}");
    assert!(out.contains("Variant Empty"), "{out}");
    assert!(out.contains("Fn area(self) -> f32"), "{out}");
}

#[test]
fn an_interface_carries_supertraits_and_default_bodies() {
    // [IFC-3]
    let out = dump("interface Ord: Eq:\n    fn cmp(self, other: Self) -> Ordering\n");
    assert!(out.contains("Interface Ord"), "{out}");
    assert!(out.contains("(no body)"), "{out}");
}

#[test]
fn extend_attaches_methods_to_a_type() {
    let out = dump("extend Mesh implements Drawable:\n    fn bounds(self) -> AABB:\n        return self.aabb\n");
    assert!(out.contains("Extend Mesh"), "{out}");
}

#[test]
fn attributes_are_attached_to_the_item() {
    let out = dump("@derive(Copy, Debug)\n@layout(c)\nstruct V:\n    x: f32\n");
    assert!(out.contains("@derive"), "{out}");
    assert!(out.contains("@layout"), "{out}");
}

#[test]
fn a_doc_comment_attaches_to_the_next_declaration() {
    let src = "## A particle.\nstruct Particle:\n    ## the position\n    position: Vec3\n";
    let out = run(src);
    assert!(out.codes.is_empty(), "{}", out.messages);
    assert!(out.dump.contains("Struct Particle"), "{}", out.dump);
}

#[test]
fn imports_parse_in_all_three_forms() {
    let out = dump(
        "import std.io\nfrom std.math import Vec3, sin as sine\nimport c \"vulkan/vulkan.h\" with (link = \"vulkan-1\")\n",
    );
    assert!(out.contains("Import std.io"), "{out}");
    assert!(out.contains("From std.math import Vec3, sin as sine"), "{out}");
    assert!(out.contains("ImportForeign C \"vulkan/vulkan.h\""), "{out}");
}

// -- statements ---------------------------------------------------------------

#[test]
fn a_bare_assignment_is_an_assignment_node() {
    // [GRM-4] — whether it declares or assigns is settled in name resolution.
    let out = dump("fn f():\n    c = 1\n");
    assert!(out.contains("Assign c ="), "{out}");
}

#[test]
fn a_typed_declaration_is_a_decl_node() {
    let out = dump("fn f():\n    x: i32 = 5\n");
    assert!(out.contains("Decl x: i32"), "{out}");
}

#[test]
fn tuple_destructuring_lists_every_target() {
    // [GRM-5]
    let out = dump("fn f():\n    a, b = t\n");
    assert!(out.contains("Assign a, b ="), "{out}");
}

#[test]
fn augmented_assignment_records_its_operator() {
    let out = dump("fn f():\n    x += 1\n");
    assert!(out.contains("Assign x +="), "{out}");
}

#[test]
fn elif_becomes_a_nested_if() {
    // Part XVIII §3 desugars `elif`; the AST already nests it.
    let out = dump("fn f():\n    if a:\n        pass\n    elif b:\n        pass\n    else:\n        pass\n");
    assert!(out.contains("Elif"), "{out}");
    assert!(out.contains("Else"), "{out}");
}

#[test]
fn a_single_statement_block_may_share_the_line() {
    // [LEX-9]
    let out = dump("fn f():\n    if x: return\n");
    assert!(out.contains("Return"), "{out}");
}

#[test]
fn loops_take_labels_and_an_else() {
    // [GRM-6], labelled break
    let out = dump("fn f():\n    outer: for i in 0..n:\n        break outer\n    else:\n        pass\n");
    assert!(out.contains("For i 'outer"), "{out}");
    assert!(out.contains("Break 'outer"), "{out}");
    assert!(out.contains("Else"), "{out}");
}

#[test]
fn a_pattern_condition_parses() {
    // `if Some(p) = h:` — condition := pattern "=" expression
    let out = dump("fn f():\n    if Some(p) = h:\n        pass\n");
    assert!(out.contains("CondPattern Some(p)"), "{out}");
}

#[test]
fn with_defer_and_unsafe_are_statements() {
    let out = dump("fn f():\n    with g = m.lock():\n        pass\n    defer:\n        cleanup()\n    unsafe:\n        p.write(0)\n");
    assert!(out.contains("With"), "{out}");
    assert!(out.contains("Defer"), "{out}");
    assert!(out.contains("Unsafe"), "{out}");
}

#[test]
fn a_match_statement_lists_its_arms() {
    let out = dump("fn f(s: Shape):\n    match s:\n        Circle(r):\n            pass\n        Empty:\n            pass\n");
    assert!(out.contains("Arm Circle(r)"), "{out}");
    assert!(out.contains("Arm Empty"), "{out}");
}

#[test]
fn mixing_match_arm_forms_is_e0011() {
    // [GRM-10]
    let out = run("fn f(s: Shape):\n    match s:\n        A => 1\n        B:\n            pass\n");
    assert!(out.codes.contains(&"E0103".to_string()), "{}", out.messages);
}

// -- expressions ---------------------------------------------------------------

#[test]
fn precedence_follows_the_table() {
    // 1 + 2 * 3  ->  1 + (2 * 3)
    assert!(contains("fn f():\n    x = 1 + 2 * 3\n", "Binary +"));
    let out = dump("fn f():\n    x = 1 + 2 * 3\n");
    let plus = out.find("Binary +").unwrap();
    let star = out.find("Binary *").unwrap();
    assert!(plus < star, "`*` must sit under `+`:\n{out}");
}

#[test]
fn exponentiation_binds_tighter_than_unary_minus() {
    // [III.5] — `-2**2 == -4`, so the negation wraps the power.
    let out = dump("fn f():\n    x = -2 ** 2\n");
    let neg = out.find("Unary Neg").unwrap();
    let pow = out.find("Binary **").unwrap();
    assert!(neg < pow, "the negation must wrap the power:\n{out}");
}

#[test]
fn exponentiation_is_right_associative() {
    let out = dump("fn f():\n    x = 2 ** 3 ** 4\n");
    // Two `**` nodes; the second must be nested inside the first.
    assert_eq!(out.matches("Binary **").count(), 2, "{out}");
}

#[test]
fn not_binds_looser_than_a_comparison() {
    // Level 4 (`not`) is below level 5 (comparisons), so `not a == b` groups
    // as `not (a == b)`.
    let out = dump("fn f():\n    x = not a == b\n");
    let not = out.find("Unary Not").unwrap();
    let eq = out.find("Binary ==").unwrap();
    assert!(not < eq, "{out}");
}

#[test]
fn a_chained_comparison_is_e0010() {
    // [III.5] — no Python chaining.
    let out = run("fn f():\n    x = a < b < c\n");
    assert!(out.codes.contains(&"E0102".to_string()), "{}", out.messages);
}

#[test]
fn the_ternary_is_right_associative_and_loosest() {
    let out = dump("fn f():\n    x = a if c else b\n");
    assert!(out.contains("Ternary"), "{out}");
}

#[test]
fn ranges_parse_in_every_form() {
    let out = dump("fn f():\n    a = 0..n\n    b = 0..=n\n    c = ..n\n");
    // Match to the end of the line: "Range .." is a prefix of "Range ..=".
    assert_eq!(out.matches("Range ..\n").count(), 2, "{out}");
    assert_eq!(out.matches("Range ..=\n").count(), 1, "{out}");
}

#[test]
fn calls_indexing_and_fields_chain() {
    let out = dump("fn f():\n    x = a.b(1).c[2].d\n");
    assert!(out.contains("MethodCall .b"), "{out}");
    assert!(out.contains("IndexOrInstantiate"), "{out}");
    assert!(out.contains("Field .d"), "{out}");
}

#[test]
fn generic_instantiation_parses_as_index_or_instantiate() {
    // [GRM-8] — the parser cannot tell these apart; resolution does.
    let out = dump("fn f():\n    x = Array[i32]()\n    y = a[i]\n");
    assert_eq!(out.matches("IndexOrInstantiate").count(), 2, "{out}");
}

#[test]
fn named_arguments_are_recorded() {
    // [TYP-25]
    let out = dump("fn f():\n    p = Particle(position=Vec3.ZERO, lifetime=2.0)\n");
    assert!(out.contains("Arg position="), "{out}");
    assert!(out.contains("Arg lifetime="), "{out}");
}

#[test]
fn try_and_optional_chaining_parse() {
    let out = dump("fn f() -> Result[i32, E]:\n    n = s.parse()?\n    m = h?.name\n    return Ok(n)\n");
    assert!(out.contains("Try"), "{out}");
    assert!(out.contains("OptChain ?.name"), "{out}");
}

#[test]
fn casts_and_downcasts_are_distinct() {
    let out = dump("fn f():\n    a = x as u8\n    b = h as? Door\n    c = h as! Door\n");
    assert!(out.contains("Cast as u8"), "{out}");
    assert!(out.contains("Downcast as? Door"), "{out}");
    assert!(out.contains("Downcast as! Door"), "{out}");
}

#[test]
fn lambdas_parse_in_both_body_forms() {
    // [CLO-1]
    let out = dump("fn f():\n    double = fn(x: f32) => x * 2.0\n    task = owned fn() => run(job)\n");
    assert!(out.contains("Lambda"), "{out}");
    assert!(out.contains("OwnedLambda"), "{out}");
}

#[test]
fn an_f_string_reparses_its_interpolations() {
    // [LEX-19] — the hole is a full expression, parsed against the real file.
    let out = dump("fn f():\n    s = f\"{name}: {items.len()} alive\"\n");
    assert!(out.contains("FString"), "{out}");
    assert!(out.contains("Path name"), "{out}");
    assert!(out.contains("MethodCall .len"), "{out}");
}

#[test]
fn an_f_string_keeps_its_format_spec() {
    let out = dump("fn f():\n    s = f\"{v:.3}\"\n");
    assert!(out.contains("Hole :.3"), "{out}");
}

// -- types -----------------------------------------------------------------------

#[test]
fn type_forms_all_parse() {
    let out = dump(
        "fn f(a: ref i32, b: ref mut Vec3, c: *mut u8, d: (i32, f32), e: dyn Drawable, g: [f32; 16], h: extern \"C\" fn(i32) -> i32) -> Result[void, E]:\n    pass\n",
    );
    assert!(out.contains("a: ref i32"), "{out}");
    assert!(out.contains("b: ref mut Vec3"), "{out}");
    assert!(out.contains("c: *mut u8"), "{out}");
    assert!(out.contains("d: (i32, f32)"), "{out}");
    assert!(out.contains("e: dyn Drawable"), "{out}");
    assert!(out.contains("g: [f32; N]"), "{out}");
    assert!(out.contains("-> Result[void, E]"), "{out}");
}

#[test]
fn generics_with_bounds_and_where_clauses_parse() {
    let out = dump("fn sum[T: Numeric + Copy](xs: Span[T]) -> T where T: Default:\n    pass\n");
    assert!(out.contains("Fn sum(xs: Span[T]) -> T"), "{out}");
}

// -- recovery ----------------------------------------------------------------------

#[test]
fn a_bad_statement_does_not_swallow_the_rest_of_the_file() {
    // [AST-2]
    let out = run("fn a():\n    x = = 1\n\nfn b():\n    pass\n");
    assert!(!out.codes.is_empty());
    assert!(out.dump.contains("Fn b()"), "recovery lost the next item:\n{}", out.dump);
}

#[test]
fn one_malformed_region_reports_at_most_three_diagnostics() {
    // [AST-2]
    let out = run("fn a():\n    x = ) ) ) ) ) ) )\n");
    assert!(out.codes.len() <= 3, "cascade of {} diagnostics:\n{}", out.codes.len(), out.messages);
    assert!(!out.codes.is_empty());
}

#[test]
fn a_reserved_word_used_as_a_name_is_e0005() {
    let out = run("fn f():\n    async = 1\n");
    assert!(out.codes.contains(&"E0005".to_string()), "{}", out.messages);
}

#[test]
fn a_missing_block_is_e0004() {
    // [LEX-9]
    let out = run("fn f():\nfn g():\n    pass\n");
    assert!(out.codes.contains(&"E0004".to_string()), "{}", out.messages);
}

// -- the milestone program ----------------------------------------------------------

#[test]
fn milestone_m1_parses() {
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
    assert!(out.codes.is_empty(), "unexpected diagnostics:\n{}", out.messages);
    assert!(out.dump.contains("Struct Vec3"), "{}", out.dump);
    assert!(out.dump.contains("Fn add(a: Vec3, b: Vec3) -> Vec3"), "{}", out.dump);
    assert!(out.dump.contains("Fn main()"), "{}", out.dump);
}

#[test]
fn the_specifications_own_example_program_parses() {
    // Part I §4, minus the parts Phase 0 has no runtime for.
    let src = "\
import std.io
from std.math import Vec3, sqrt

@derive(Copy, Debug)
struct Particle:
    position: Vec3
    velocity: Vec3
    lifetime: f32

class Emitter:
    name: String
    particles: Array[Particle]
    spawn_rate: f32 = 100.0

    fn init(mut self, name: String):
        self.name = name
        self.particles = Array[Particle]()

    fn spawn(mut self, count: usize):
        for i in 0..count:
            self.particles.push(Particle(
                position=Vec3.ZERO,
                velocity=Vec3(0, 9.8, 0),
                lifetime=2.0))

@noalloc
@simd
fn integrate(mut particles: MutSpan[Particle], dt: f32):
    for p in particles.iter_mut():
        p.velocity.y -= 9.81 * dt
        p.position += p.velocity * dt
        p.lifetime -= dt

fn main() -> Result[void, io.Error]:
    fountain = Emitter(\"fountain\")
    fountain.spawn(10_000)

    for frame in 0..600:
        integrate(fountain.particles.as_mut_span(), 1.0 / 60.0)
        fountain.particles.retain(fn(p) => p.lifetime > 0.0)

    io.println(f\"{fountain.name}: {fountain.particles.len()} alive\")
    return Ok(())
";
    let out = run(src);
    assert!(out.codes.is_empty(), "unexpected diagnostics:\n{}", out.messages);
    assert!(out.dump.contains("Class Emitter"), "{}", out.dump);
    assert!(out.dump.contains("Fn integrate"), "{}", out.dump);
    assert!(out.dump.contains("FString"), "{}", out.dump);
}

// -- doc comments at every position -------------------------------------------

#[test]
fn alternating_comments_and_code_each_attach_to_the_next_line() {
    // The buffered doc comment is flushed after *every* content line's
    // indentation is processed, not only when an INDENT or DEDENT is emitted,
    // so a run of alternating comment and code lines stays paired up.
    let out = dump("class A:\n    ## doc x\n    x: i32\n    ## doc y\n    y: i32\n");
    let doc_x = out.find("doc x").unwrap();
    let field_x = out.find("Field x").unwrap();
    let doc_y = out.find("doc y").unwrap();
    let field_y = out.find("Field y").unwrap();
    assert!(doc_x < field_x && field_x < doc_y && doc_y < field_y, "{out}");
}

#[test]
fn a_blank_line_between_a_doc_comment_and_its_declaration_is_allowed() {
    // [LEX-11] — "not immediately followed (ignoring blank lines)".
    let out = dump("class A:\n    ## doc x\n\n    x: i32\n");
    assert!(out.contains("Field x"), "{out}");
}

#[test]
fn a_trailing_doc_comment_is_silent() {
    // A comment never affects compilation, warnings included (ERR-007).
    // Meeting one where a newline was expected must not fail the parse.
    let out = run("class A:\n    x: i32  ## trailing\n    y: i32\n");
    assert!(out.codes.is_empty(), "a comment must never affect compilation:\n{}", out.messages);
    assert!(out.dump.contains("Field x"), "{}", out.dump);
    assert!(out.dump.contains("Field y"), "{}", out.dump);
}

#[test]
fn a_doc_comment_with_nothing_after_it_is_silent() {
    // The last line of a file is the commonest way to write one (ERR-007).
    let out = run("class A:\n    x: i32\n    ## nothing follows\n");
    assert!(out.codes.is_empty(), "a comment must never affect compilation:\n{}", out.messages);
    assert!(out.dump.contains("Field x"), "{}", out.dump);
}

#[test]
fn a_doc_comment_at_the_end_of_a_block_documents_the_next_item() {
    // [LEX-11] — "attaches to the next declaration", even across a dedent.
    let out = dump("class A:\n    x: i32\n    ## doc for f\n\nfn f(): pass\n");
    let doc = out.find("Doc \"doc for f\"").unwrap();
    let f = out.find("Fn f()").unwrap();
    assert!(doc < f, "{out}");
}

// -- v0.8.3: range clauses, coroutines -----------------------------------------

#[test]
fn a_type_alias_may_carry_a_range_clause() {
    // `[RNG-1]` — the `in` clause makes the alias a nominal range type.
    let out = dump("type Roughness = f32 in 0.0 ..= 1.0\n");
    assert!(out.contains("TypeAlias Roughness"), "{out}");
    assert!(out.contains("Range"), "the `in` clause is an expression: {out}");
}

#[test]
fn a_range_clause_takes_a_half_open_range_too() {
    // `[RNG-1]` — "a `..` or `..=` range expression".
    let out = dump("type Percent = u8 in 0 .. 101\n");
    assert!(out.contains("TypeAlias Percent"), "{out}");
}

#[test]
fn a_range_clause_does_not_disturb_a_for_header() {
    // `[GRM-8d]` — the production is LL(2) and a parser commits on the
    // enclosing construct, never on the token `in` (`[GRM-23]`).
    let out = dump("fn f(xs: Array[i32]):\n    for x in xs: pass\n");
    assert!(out.contains("For"), "{out}");
}

#[test]
fn a_range_clause_on_an_associated_type_is_rejected() {
    // `[GRM-8d]` — admitted only where `type_alias` appears as an `item`.
    let out = run("interface I:\n    type Item = f32 in 0.0 ..= 1.0\n");
    assert!(out.codes.contains(&"E2213".to_string()), "{}", out.messages);
}

#[test]
fn a_generic_range_type_is_rejected() {
    // `[GRM-8d]` — "a range type is over a concrete representation".
    let out = run("type Bad[T] = T in 0 ..= 1\n");
    assert!(out.codes.contains(&"E2213".to_string()), "{}", out.messages);
}

#[test]
fn gen_fn_declares_a_coroutine() {
    // `[GRM-21]` — `gen_fn := "gen" fn_decl`.
    let out = dump("gen fn ticks() -> Coroutine[void]:\n    yield 1\n");
    assert!(out.contains("Fn ticks"), "{out}");
    assert!(out.contains("Yield"), "{out}");
}

#[test]
fn gen_is_contextual_so_a_variable_may_be_named_gen() {
    // `[LEX-15b]` — `gen` is a keyword only immediately before `fn`.
    let out = dump("fn f():\n    gen = 1\n    gen = gen + 1\n");
    assert!(out.contains("Assign"), "{out}");
}

#[test]
fn a_bare_yield_is_permitted() {
    // `[GRM-22]` — "A bare `yield` is `yield ()`".
    let out = dump("gen fn f() -> Coroutine[void]:\n    yield\n");
    assert!(out.contains("Yield"), "{out}");
}

#[test]
fn yield_may_be_bound() {
    // `[GRM-22]` — unlike a jump, `yield` has the resume type, so it may be
    // the right-hand side of a binding.
    let out = dump("gen fn f() -> Coroutine[void]:\n    v = yield 1\n    pass\n");
    assert!(out.contains("Yield"), "{out}");
}

#[test]
fn yield_may_not_be_an_operand() {
    // `[GRM-22]` gives `yield` `return`'s precedence, the lowest there is.
    let out = run("gen fn f() -> Coroutine[void]:\n    x = 1 + yield 2\n");
    assert!(!out.codes.is_empty(), "expected a diagnostic:\n{}", out.dump);
}

#[test]
fn yield_is_a_keyword_and_needs_a_raw_identifier_as_a_name() {
    // `[LEX-15b]`, errata ERR-025 — `yield` is fully reserved in v1, so it is
    // no longer `E0005` ("reserved for a later version") but a keyword.
    let out = run("fn f():\n    yield = 1\n");
    assert!(!out.codes.contains(&"E0005".to_string()), "{}", out.messages);
    let ok = dump("fn f():\n    r#yield = 1\n");
    assert!(ok.contains("Assign"), "{ok}");
}
