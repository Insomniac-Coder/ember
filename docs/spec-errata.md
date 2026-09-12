# Specification errata

Places where the specification is silent, self-contradictory, or contradicted
by its own examples, together with what the implementation does about it.

Ground rule 3 of Part XX.1: any deviation from this document requires an ADR
and an entry here. An entry is a **proposal to the owner** unless it is marked
*decided*; the implementation follows the entry so that work can continue, and
each entry states exactly what would change if the owner rules the other way.

## The document is never edited (owner, 2026-09-09)

**`docs/spec-source/ember-spec.md` is byte-identical to
`docs/spec-source/as-received/Ember_v0.8.3_spec.md` and stays that way.** The
specification is the source of truth and the product design; it is not adjusted
to fit the compiler, and it is not adjusted to fit a tool either.

This reverses how errata were handled up to this point. Fourteen passages had
been *edited in place* to carry the reading each erratum records — the document
and the errata file agreeing at the cost of the document no longer being the
owner's. Those edits are reverted; the hashes match again.

An erratum therefore now does three things and no more:

1. **quotes** the defective passage as the document actually writes it;
2. **states the reading** the implementation follows, and what changes if the
   owner rules otherwise;
3. **names the exception** a tool carries, if the defect makes a gate fire.

A defect that makes a gate fire goes in that gate's baseline with the erratum
id beside it, never into the document as an annotation. Two do:

* **`tools/spec_check_baseline.json`** — XVI.4 and XVI.7a fence overlay-language
  source as `ember`, and Part III defines no `overlay_decl`, so it cannot parse
  (ERR-032). Previously fenced `ember,ignore` by an edit; now baselined.
* **`tools/rule_index_baseline.json`** — XX §6's prose names `E4050`, `E4057`,
  `E4060`, `E4064` and `W4001`, which 0.6.2 removed with the contract and
  verification layer and which the registry no longer defines (ERR-027).
  Previously struck from the prose by an edit; now baselined.

Both baselines grew by exactly this, once, for this reason. `[TST-4c]`'s ratchet
otherwise holds: they may shrink and never grow.

| ID | Rule | Status |
|---|---|---|
| ERR-001 | `E0010`, `E0011`, `E0020` | **decided** — renumbered; carried by v0.5 |
| ERR-002 | `[LEX-17]`, Part II §5 grammar | **closed by v0.5** `OQ-24` — `1f32` legal, as implemented |
| ERR-003 | Part II §6 operator table | **closed by v0.5** `OQ-25` — `;` is punctuation, never a separator |
| ERR-004 | `[CLS-9]`, Part II §4 | **closed by v0.5** `OQ-26` — `let` fully reserved; **reverses** this entry |
| ERR-005 | `[LEX-8]` with `[LEX-11]` | **closed by v0.5** `OQ-12` — `[LEX-11a]` is normative |
| ERR-006 | Part II §3 with `[TST-1]` | **decided** — annotations moved to `#$`; carried by v0.5 |
| ERR-007 | `[LEX-11]` | **decided** — a dangling doc comment is silent; carried by v0.5 |
| ERR-008 | Part III §109 with Part IV §2, Appendix A | **closed by v0.5** `OQ-14` — jumps are expressions; **reverses** this entry |
| ERR-009 | `[LEX-15]`, `[LEX-15a]`, `[LEX-14a]`, Part II §4 | **decided** — `type` is a v1 keyword; 49 entries |
| ERR-010 | `[EFF-12]`, Part X §2 | **decided** — the two statements merged into one |
| ERR-011 | `[PAR-1]`, `[PAR-2]`, Part XI §4 | **decided** — `[PAR-2]` split out and de-duplicated |
| ERR-012 | `[MOD-6]`, Part V §1 | **decided** — first half restored |
| ERR-013 | `[CLO-2]`, Part VI §5 | **decided** — capture-mode rule restored |
| ERR-014 | `[FFI-6]`, Part XVI §2 | **decided** — the fragment repaired and the macro list brought up to date; **the contradiction I claimed was not one** |
| ERR-015 | `[EFF-11]`, Part X §1.1 | **decided** — fifth reason code defined |
| ERR-016 | Part IV §8 interface list, `[GRM-18]`, `[TST-7]` | **decided** — block rewritten in indented form |
| ERR-017 | `[LEX-15]`, Part II §4, Part IV §8 `From` | **decided** — `from` is contextual; reserved set back to 48 |
| ERR-018 | Part VI §6 with Part XIV §1 and Appendix A | **decided** — `assert` takes parentheses |
| ERR-019 | `[RT-5]` | **withdrawn** — not a defect; the header is *generated*. Our hand-written one is an implementation gap |
| ERR-020 | `[TST-6]` | **decided** — the stale description corrected; `compile-pass` left as the requirement |
| ERR-021 | `[DIA-7a]`, XIX.6.1 shapes B1..B10 | **decided** — E3021-E3027 allocated and in the table |
| ERR-022 | `[DIA-7]` with `[DIA-7a]` | **withdrawn** — `[DIA-7a]` sits under `[DIA-7]`, which scopes it |
| ERR-023 | `[TYP-14]`, Part IV §2 | **decided** — reading through is a property of the type in a value context, not of the form |
| ERR-024 | Part XVIII §4.7 step 6 | **decided** — a by-value parameter's storage ends with the frame, so `E3060` covers it |

**Against v0.8.3**, installed 2026-09-09. ERR-001..ERR-024 were raised against
v0.5 and are **superseded**: v0.8.3 is authored from 0.8.2c and carries its own
resolution of each. They are kept because a resolution reversed once (ERR-004,
ERR-008) is worth being able to read again.

| ID | Rule | Status |
|---|---|---|
| ERR-025 | `[LEX-15b]` with Part II §4's reserved-for-future list | **withdrawn — not a defect** — `[LEX-15b]` resolves it in the document: "this rule supersedes that count". The table is stale, the rule governs, and the compiler's 49 keywords already match it |
| ERR-026 | `[GRM-23]` with `[ATT-1]`, `[DIA-6a]` | **decided** — `a in b in c` is `E0102`, not `E0104` |
| ERR-027 | `[UNS-7]`, `[STD-6]`, `[EFF-17]`, `[EFF-18]`, XX §6's registry paragraph | **decided** — four leftovers from the layer 0.6.2 removed, struck |
| ERR-028 | `[EFF-18]` with Part X §1 and `[DET-2]` | **decided** — the effect set has ten members; `Nondet` was added after `[EFF-18]` was written |
| ERR-029 | `[BLD-2]`, `[FFI-34]`, `[FFI-38]`, `[FFI-2a]`, `[BLD-11]`, `[TCB-5]`, `[FFI-33b]`, `[TST-13]`, `[RNG-7]`, `[RNG-8]` | **decided** — the nine editorial instructions read as carried out; `[RNG-8]`'s truncated head was recovered verbatim from both 0.6 source copies and declared in `spec-amendments.md`, matching the decision recorded in this entry's body |
| ERR-030 | `[GRM-16]` with Part III §4's `small_stmt` | **decided** — the rule supersedes the production; already implemented that way |
| ERR-031 | `[TST-11]` | **decided** — a merge artefact; the obligations read as the surrounding clauses state them |
| ERR-032 | `[TST-7]` with XVI.4 and XVI.7a's fenced blocks | **decided** — the two overlay blocks are fenced `ember,ignore`, as the rule says they are |
| ERR-033 | `[LEX-15a]`, `[GRM-20]`, `[GRM-8d]`, `[FFI-34a]` | **decided** — each states its rule twice; the second copy is redundant, not a second rule |
| ERR-034 | `[FFI-17d]` citing `[FFI-17b]` | **closed** — amendment A7 defines `@ffi(no_virtual_dtor)` in `[FFI-17d]` itself, since the rule it cited defines it nowhere |
| ERR-035 | Section placement in Parts XV, XVI, XX, XXI | **decided** — recorded, not moved; the split follows the source |
| ERR-036 | Part III §2 `fn_header` with XVI.10, `[FFI-26]`, `[FFI-31b]` | **closed** — amendment A8 adds `["extern" string_lit]` to `fn_header`; the compiler parses it and emits an unmangled symbol |
| ERR-037 | Part III §2 with `[FFI-39]` | **closed** — amendment A9 adds `extern_class`; it parses and is then refused by name, because the C++ importer is Phase 7 (deviation D3) |
| ERR-038 | `E2213` in IV.2a and `[GRM-8d]` | **decided** — one title covers both conditions; `[RNG-1]` is the defining rule |
| ERR-039 | `E9010` in `[TYP-9c]` and `[MAN-3]` | **decided** — `[TYP-9c]` keeps `E9010`; `[MAN-3]` takes `E9012` |
| ERR-040 | `[CLI-9]` with `[GRM-8d]` | **decided** — `--syntax-only` reports what the front end produces; the code ranges describe the stages, not a filter |
| ERR-041 | `[FN-1]` with Part VII §7's worked example | **DECIDED by the owner 2026-09-10** — the worked example governs; the place requirement applies to what the view was taken of. Amendment S4, `[FN-1a]` in 0.8.5. Deviation D5 closes: the compiler was right and no code moved |
| ERR-042 | `[TYP-26]`, `[IFC-2]`, `[HND-2]`, `[GPU-7]`, `[IDE-*]` | **WITHDRAWN 2026-09-10 — the entry was wrong.** The owner ordered an inventory; it found **zero** genuine gaps. Every id is defined, deliberately reserved, or a historical citation. The list was also stale. See the entry |
| ERR-043 | `[CELL-9]` with `[UNS-5]` | **DECIDED by the owner 2026-09-10** — `UnsafeCell` is retained and becomes the language's lowest-level interior-mutability primitive, in `std.mem`. Amendment S2; `[UNS-10]`/`[UNS-10a]`/`[UNS-10b]` in 0.8.5 |
| ERR-044 | `[TYP-15]` with `[LT-3]` | **decided by the owner, 2026-09-09** — `[LT-3]`'s semantics govern: a view may be stored where its region outlives the destination, so a static-region view is admitted and every other stays refused. Amendment S1 |
| ERR-045 | `[LT-1]`'s example with `[LT-1a]` and Part III §2 | **decided** — `[LT-1a]` and the grammar govern; `[LT-1]`'s trailing-suffix example does not parse |
| ERR-046 | 0.9.5 H5 `[LT-8]` with `[LT-11]` / `[TST-16]` | **DECIDED by the owner 2026-09-12.** H6 adds explicit all-mutable `_mut` helpers using canonical `MutSpan[T]`; no mixed overloads are implied. ODR-005 closed |
| ERR-047 | H6/H7 `[LT-8]` with `fn_type`, `[FN-1]`, `[FN-2a]` | **DECIDED by the owner 2026-09-12.** H7 adds callable modes; H8 makes mutable helper inputs `mut` reborrows and forbids consuming them. ODR-006 closed |
| ERR-048 | H7 `[FN-6]` with `[CLO-3]` / `Callable[Args, R]` | **DECIDED by the owner 2026-09-12.** H8 preserves the full mode vector as compiler-known canonical metadata through the existing abstraction. ODR-007 closed |
| ERR-049 | H8 `[LT-4]` with `[LT-1a]` | **DECIDED by the owner 2026-09-12.** H9 permits the narrow `@borrows(arena)` provenance contract for Arena-backed returned views. `Arena` remains a non-view; arbitrary non-view parameters remain forbidden. ODR-008 closed |
| ERR-050 | H9 `[ARN-3]` with `[UNS-1]`, `[UNS-5]`, and the Phase 2/4 plan | **DECIDED by the owner 2026-09-12.** H10 defines deterministic bulk initialization, exact `Zeroable` validity, canonical `MaybeUninit` APIs/state transitions, `!needs_drop`, rollback, and Phase 2 availability. ODR-009 closed; H9 unchanged |

---

## ERR-001 — Three parser errors were numbered into the lexer's range

**Status: decided by the owner, 2026-09-07. Renumbered.**

**Where.** Part XIX §6's registry gives `E0000–E0099` to "lexer / indentation /
directives" and `E0100–E0499` to the parser. Three codes sat in the lexer range
although all three need a parsed tree to detect:

| Code | Rule | Why it is a parser error |
|---|---|---|
| `E0010` | Part III §5 | `a < b < c` — needs the expression tree to see the chain |
| `E0011` | `[GRM-10]` | mixed `:` and `=>` match arms — needs the arm list |
| `E0020` | `[ATT-1]` | unknown attribute — needs the parsed attribute path |

**Decision.** Renumbered into the parser's range:

| Was | Is now | Title |
|---|---|---|
| `E0010` | **`E0102`** | chained comparison |
| `E0011` | **`E0103`** | match arms mix statement and expression form |
| `E0020` | **`E0104`** | unknown attribute |

`E0100` (unexpected token) and `E0101` (unclosed delimiter) already occupied
the first two parser slots, so the three follow them.

**Applied to.** `ember_diag::codes`, the parser's emission sites, the parser
tests, and the three sentences in `docs/spec/part-03-grammar.md` that name the
old numbers (Part III §5's precedence table, `[GRM-10]`, and Part III §7's
attribute paragraph). The original numbers are recorded in this entry, so
nothing is lost.

The exception list that `ember_diag::codes` carried for these three is gone:
every code now sits in the range its subsystem owns, and the test that checks
that has no exemptions left.

**Why it was worth doing now rather than later.** `[TST-4]` requires a
`tests/conformance/<rule-id>/` directory and a `docs/errors/EXXXX.md` page for
every code. None exist yet. Once they do, a renumber means moving directories
and rewriting cross-references in prose — the cost rises with every phase.

---

## ERR-002 — `1f32` is not in the grammar

**Where.** Part II §5 attaches `float_suffix` to `float_lit`, and `float_lit`
requires a fractional part or an exponent:

```
float_lit   := dec_lit "." dec_lit exponent? | dec_lit exponent | dec_lit "."
float_suffix:= "f16" | "f32" | "f64"
```

So `1f32` is an integer literal followed by an unrecognised suffix — an error.
`[LEX-16]` does say an untyped integer "may also take a float type by context",
but that is inference from context, not a suffix a programmer may write.

**What the implementation does.** Accepts `1f32` as a float literal.

**Why.** It cannot be ambiguous with anything, every neighbouring language
accepts it, and rejecting it costs a papercut in exactly the numeric code Ember
is aimed at — where `1f32` and `1.0f32` mean the same thing to the reader.

**If the owner rules the other way.** One condition in
`ember_lexer::Lexer::lex_number`, plus a diagnostic suggesting `1.0f32`. This
is the least consequential entry here.

**Closed by v0.5, `OQ-24`.** `1f32` and `1.0f32` are both float literals;
`1.f32` remains a method call on `1`, because `float_lit`'s `dec_lit "."` form
requires that no identifier character follow. `float_suffix` was added to
`float_lit` in the document. The implementation already did this and needs no
change.

---

## ERR-003 — `;` is used by the grammar but is not an operator

**Where.** Part II §6's operator and punctuation table lists no semicolon. Part
III uses one in three productions:

- `simple_stmt := small_stmt {";" small_stmt} [";"]`
- `array_type := "[" type ";" expression "]"`
- `array_lit := "[" expression ";" expression "]"`

It also appears in code the specification presents as valid: `[0.0; 16]` in
Part IX §2, `[f32; 16]` in Part III §3, and both again in Appendix A. A lexer
built from the §6 table alone cannot tokenise the specification's own examples.

**What the implementation does.** Adds `;` to the punctuation set.

**If the owner rules the other way.** Removing `;` means rewriting three
grammar productions and giving `[T; N]` new syntax — a far larger change than
adding one token, which is why the implementation went this way.

**Side effect worth a decision of its own.** `simple_stmt` permits
`a = 1; b = 2` on one line. That is now legal Ember. Whether the formatter
should ever *produce* it is unspecified — `[FMT-1]` says nothing — and is a
Phase 1 question.

**Closed by v0.5, `OQ-25`.** `;` stays in the Part II §6 table, as this entry
proposed, but the side effect above was ruled the other way: `simple_stmt` loses
its `{";" small_stmt}` tail, one line carries one statement, and `a = 1; b = 2`
is `E0105` (`[GRM-18]`). `[FMT-3]` settles the formatter question — it never
emits `;` outside `[T; N]` and `[v; N]`. The parser already never accepted `;`
as a separator, so the work is the `E0105` code and its mandated help, not a
grammar change.

---

## ERR-004 — `let` has a meaning but is not a keyword

**Where.** `[CLS-9]` gives `let` a job: "`let name: T` declares an **immutable**
field, assignable only in `init`". `[MOD-7]`'s visibility table has a
`let value: T` row, and Part V §5's example class opens with
`let entity: Entity`.

Part II §4 lists 47 reserved keywords; `let` is not among them. It then lists
five contextual keywords — `abstract`, `final`, `lazy`, `test`, `bench` — and
`let` is not there either. By the lexical rules as written, `let` is an
ordinary identifier, so `let entity: Entity` parses as a field named `let`
followed by a syntax error.

**What the implementation does.** Adds `let` to the contextual list: a keyword
at the start of a type member, an ordinary identifier everywhere else.

**If the owner rules the other way.** Making `let` fully reserved breaks any
program that uses it as a variable name — which the current keyword list
explicitly permits, so somebody will. Contextual is the conservative choice and
leaves the reserved list free to take `let` later.

**Closed by v0.5, `OQ-26` — and it reverses this entry.** `let` is **fully
reserved**: a keyword in every position, with `r#let` required to use the word
as a name. The owner's reason is that `let name: T` should never depend on
position, at the cost of one identifier nobody can use unescaped. Part II §4's
v1 table gains `let`, and `[LEX-15]`'s count moves 47 → 48 (and then to 49 with
`type`, ERR-009).

**What changes.** `let` moves from `CONTEXTUAL_KEYWORDS` to `Kw` in
`compiler/ember_lexer/src/token.rs`, the `Kw::ALL.len()` assertion follows, and
`decls.rs`'s `at_contextual("let")` becomes a keyword test. A program using
`let` as a name now fails where it previously compiled; nothing in `tests/` or
`std/` does.

---

## ERR-005 — A doc comment at a block boundary is emitted on the wrong side of it

**The two rules.** Both are individually correct and neither should change.

- `[LEX-8]`: "Blank lines and comment-only lines do not affect indentation."
  This is Python's rule and it is necessary — otherwise a comment written at
  the wrong indentation would break the file around it.
- `[LEX-11]`: a `##` doc comment "attaches to the next declaration", and one
  that is not followed by a declaration is `W0001`.

**The mechanism.** The lexer emits tokens as it scans, line by line.
`INDENT` and `DEDENT` are produced while processing the *indentation* of a
line — and by `[LEX-8]` a comment-only line has no indentation processing at
all. So a `DocComment` token is emitted at the moment its own line is scanned,
which is strictly **before** any `INDENT`/`DEDENT` that the next content line
will produce.

**Failure one — the first member of a block.** Source:

```ember
class A:
    ## the field
    x: i32
```

Token stream, reading strictly in scan order:

```
kw:class  A  :  NEWLINE  DocComment("the field")  INDENT  x  :  i32  NEWLINE  DEDENT  EOF
                                 ^^^^^^^^^^^^^^  ^^^^^^
                                 emitted here    …but the block opens here
```

The doc comment sits outside the class body. A parser that collects doc
comments where it expects an *item* would attach "the field" to the class `A`
itself, and `x` would be undocumented.

**Failure two — the item after a block.** The mirror case:

```ember
class A:
    x: i32
## documents f
fn f(): pass
```

```
…  x  :  i32  NEWLINE  DocComment("documents f")  DEDENT  kw:fn  f  …
                       ^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^^^
                       emitted inside the body   …which closes only here
```

Now the doc comment is *inside* the class body, arriving where the parser
expects either another member or the end of the block.

**The scope is narrower than it first looks — a correction.** An earlier
version of this entry claimed every doc comment inside an indented block is
misplaced. That is wrong. A doc comment in the *middle* of a block is fine:

```ember
class A:
    x: i32
    ## the second field
    y: i32
```

Line 3 is comment-only, so no indentation processing; line 4's indentation
equals the stack top, so it emits no `INDENT` or `DEDENT` either. Nothing is
interposed, and `DocComment` lands immediately before `y`, exactly where it
belongs.

So the defect fires **only where the indent stack changes** — at the first
member of a block, and at the first item after a block ends. Those are also
the two most common places to write documentation, which is why it matters.

**What the implementation does.** A doc comment read from a comment-only line
is held in a buffer rather than emitted immediately, and flushed straight after
the `INDENT`/`DEDENT` tokens generated by the next line that carries content.
`[LEX-8]` is untouched — indentation still ignores comment-only lines
completely. Only the *emission order* of a token changes, and only relative to
structural tokens that carry no source text of their own.

The two failure cases above then read:

```
kw:class  A  :  NEWLINE  INDENT  DocComment("the field")  x  :  i32  …
…  x  :  i32  NEWLINE  DEDENT  DocComment("documents f")  kw:fn  f  …
```

Covered by the `ember_lexer` test
`a_doc_comment_inside_a_block_lands_after_the_indent`.

**Proposed spec text.** One sentence appended to `[LEX-11]`:

> A doc comment on a comment-only line is emitted after the `INDENT`/`DEDENT`
> tokens generated by the next line that carries content.

**If the owner rules the other way.** The alternative is to make
doc-comment-only lines significant for indentation, so that the `##` line
itself opens the block. That contradicts `[LEX-8]` as written, and it makes a
doc comment at an unexpected indentation a hard error rather than a harmless
formatting slip — a worse trade for a language whose whole comment story is
that comments never affect parsing.

---

## ERR-006 — `#!` meant two different things

**Status: decided by the owner, 2026-09-08. Test annotations moved to `#$`.**

**Where.** Part II §3 defined `#!` once, as a directive on the first line of a
file — `#! language "0.2"`, whose meaning is `[MOD-6]`'s language-version
check. Part XIX §5's `[TST-1]` and `[TST-2]` then used `#!` for something
entirely different: test expectations, on any line of a `.em` file. Test files
are ordinary compilation units, so the lexer met both in the same file and had
no way to tell them apart.

**Decision.** Test annotations get their own marker: **`#$`**.

```ember
#$ test: compile-fail
#$ rules: BRW-1, OWN-3
fn main():
    a.push(1)          #$ error[E3040]: use of moved value `a`
```

`$` was chosen because it is the only ASCII symbol with no meaning anywhere
else in Ember — it appears in no operator, no literal form, no identifier rule.
An annotation therefore cannot collide with source, now or after any future
grammar change.

To the compiler `#$` is an ordinary line comment (`[LEX-10]`), so an annotation
never affects compilation. `#!` is left with exactly one meaning.

**Applied to.** `docs/spec/part-19-toolchain.md` (`[TST-1]`, `[TST-2]`, and a
new `[TST-0]` stating that annotations are read from raw text),
`docs/spec/part-02-lexical-structure.md` (§II.3's comment table),
`docs/spec/part-20-implementation-plan.md` (the four milestone programs of
§XX.3), the test harness, and every `.em` file under `tests/`.

Part XX was missed when the decision was first applied and was caught on
2026-09-08 while merging the owner's v0.2 memory-safety update. That update was
written against the pre-decision text and reverts `#$` to `#!` in Part XIX §5
and in the §XX.3 milestones; those reversions were **not** taken. Everything
else in it was. A future reader diffing the owner's file against `docs/spec/`
will find exactly three differences: this one, ERR-008's `match` example in
Appendix A, and `[TST-0]`/`assert-c`, which the update predates.

**`[TST-0]` is new and load-bearing.** The harness reads annotations from the
raw source text, never from the token stream, because a `compile-fail` test may
be expected to fail *at the lexer* — `E0001` invalid UTF-8, `E0002` a tab in
indentation. Its expectations have to be readable even when the file does not
tokenise at all.

---

## ERR-007 — A dangling doc comment is silent, not a warning

**Status: decided by the owner, 2026-09-08.**

**Where.** `[LEX-11]`:

> A `##` comment that is not immediately followed (ignoring blank lines) by a
> declaration is a warning `W0001 dangling doc comment`.

**Decision.** No diagnostic at all. A comment never affects compilation, and
that includes producing a warning. The three positions where one can end up
documenting nothing — at the end of a code line, as the last line of a file,
and where a statement was expected — all discard it in silence.

**Consequences.** `W0001` stays in the code registry, because the registry
lists every code the specification names, but nothing emits it. If `[LEX-11]`
is ever amended in `docs/spec/`, that sentence is the one to change.

`-Dwarnings` therefore cannot fail a build over a comment, which was the
practical argument: a stray `##` should never be the thing that stops a
release build.

## ERR-008 — Is `return` a statement or an expression?

**Status: decided by the owner, 2026-09-08. It stays a statement.**

**Where.** Three places disagree.

Part III's grammar puts it among the statements:

```
small_stmt := var_decl | assignment | expression | "return" [expression] | "break" [label]
```

Part IV §2's type table gives it a type, which only a value has:

> `!` — never type; coerces to every type; result of `panic`, `return`,
> `break`, `continue`, infinite `while true`

And Appendix A's `match` example uses it where `[GRM-10]` requires an
expression:

```ember
match s:
    Circle(r) => return PI * r * r
```

**Decision.** The grammar wins. `return`, `break` and `continue` are
statements, and `=>` arms take an expression, so that example does not
compile. The statement form is what to write:

```ember
match s:
    Circle(r):
        return PI * r * r
```

**Consequences.** Appendix A's example is corrected to the statement form.
Part IV §2's row is left alone: `!` is still the type of an expression that
never produces a value, which `panic` and an infinite `while true` still are.

The owner's v0.2 memory-safety update of 2026-09-08 carries Appendix A along
unchanged, with the `=>` form restored — it predates this decision, and its own
change table does not list Appendix A as changed. The corrected form was kept.

**If the owner rules the other way**, `return`/`break`/`continue` become
expressions of type `!`. The type system already supports it — `!` coerces to
every type, so an arm yielding `!` sits beside arms yielding `f32` with no
further work. The change is in the parser: `parse_prefix` would need to accept
these three keywords, and `[GRM-10]`'s expression arms would then take them.

**Closed by v0.5, `OQ-14` — and it reverses this entry.** They *are*
expressions: type `!`, parsed at the **lowest** precedence, parallel to
`ternary` and never as an `atom`. `[GRM-16]` removes `"return" [expression]`,
`"break" [label]` and `"continue" [label]` from `small_stmt` — a jump written
as a statement is now an expression statement — and adds `E0107 a jump
expression may not be an operand` so that `a + return b` is still rejected.
`block_expr` is deleted from `atom` in the same rule.

The owner's reason is the one this entry recorded as the alternative: Part IV
§2's `!` producer list already implied it. The paragraph above about the v0.2
amendment "carrying Appendix A along unchanged" is superseded — v0.5 rewrites
that example as `return match s:` with `=>` arms, which parses either way.

**What changes.** `parse_prefix` accepts the three keywords at the lowest
precedence; `body.rs`'s three statement arms become expression statements;
`E0107` is added. The `!`-coercion side is already there.

---

# Defects found in v0.5 itself (ERR-009 … ERR-015)

The seven entries below were found while applying v0.5 on 2026-09-08, before any
of it reached code. **Six of the seven have one cause**: 0.4's and 0.5's
amendments were *appended* to each Part rather than *substituted into* the rule
they amend. That leaves two signatures, and both are mechanically detectable:

- a rule id stated twice, which `[XXII.4]` now makes a hard CI failure; and
- a rule whose body begins with `…`, which is an amendment that has been
  separated from the sentence it was meant to continue — and, in two cases,
  which **deleted** the original sentence outright.

`tools/rule_index.py` must carry both checks when it is written. The ellipsis
test costs four lines and catches an orphaned amendment the moment it lands,
which is the only point at which the original text is still recoverable.

**Recoverability is the reason `docs/spec-source/as-received/` exists.** Two of
these rules (`[MOD-6]`, `[CLO-2]`) lost text that survives nowhere in v0.5. It
was restored from the committed split of v0.2 — which would have been
overwritten had `docs/spec/` been regenerated before the diff was read.

---

## ERR-009 — `type` is in both keyword lists

**Where.** Part II §4 lists `type` under "reserved for future use (lexed as
keywords, `E0005` if used)". `[LEX-15a]`, added in 0.4, says the opposite:

> `type` is a v1 keyword: it introduces a type alias (`type_alias`), an
> associated type in an `interface` (`interface_member`), and an opaque foreign
> type in an `extern` block (`extern_item`).

Part III §2 needs it to be a keyword — all three productions are in the v1
grammar. And `[LEX-14a]` requires the `E0005` message for a reserved-future
word to "name the version that will introduce it", which cannot be written for
a word introduced now.

**Decision (owner, 2026-09-08).** `type` moves into the v1 reserved table.
`[LEX-15]`'s count goes from 48 to **49**.

**Why full reservation rather than contextual.** It is the same call `OQ-26`
made for `let` one ruling earlier, for the same stated reason: the meaning
never depends on position. It also makes `[LEX-14]` land — that rule's
motivating example for raw identifiers is literally "imported C symbols such as
a field called `type`", and `r#type` is only the anticipated escape if `type` is
reserved.

**The cost, stated plainly.** `event.type` must be written `event.r#type`, and
that lands on hand-written gameplay code, not only on FFI. The alternative
considered was making `type` contextual — joining `abstract`, `final`, `lazy`,
`test`, `bench`, keeping the count at 48, and resolving `type X = Y` against
`type = 5` on the second token, which is within the LL(2) the grammar claims.
That is what Python did for `type X = …` in 3.12. It was not taken, because
positional keywords are what `OQ-26` had just rejected.

**Applied to.** Part II §4's two keyword blocks and `[LEX-15]`'s sentence.

**Not yet applied to the compiler.** `Type` moves from `Reserved` to `Kw` in
`compiler/ember_lexer/src/token.rs`, and `Kw::ALL.len()` goes 47 → 49 (`let`,
ERR-004, is the other one).

**If the owner rules the other way**, `type` joins `CONTEXTUAL_KEYWORDS`
instead, `[LEX-15]`'s count stays 48, and the parser recognises `type` by text
in the three declaration positions. A relaxation worth considering under either
ruling: admit any keyword token in field- and method-name position after `.`,
where no keyword can begin an expression. That makes `event.type` legal with
`type` fully reserved and confines `r#` to bindings. It is a language addition,
not a defect fix, so it is not taken here.

---

## ERR-010 — `[EFF-12]` is stated twice, and neither statement is complete

**Where.** Part X §2 states `[EFF-12]` in two places:

- in the hard-contracts bullet list, beside `@noalloc`, `@nosync` and
  `@noblock`, with the carve-out that `@static_safe` does **not** forbid
  `RuntimeCheck(Bounds)`, `(Stale)` or `(Overflow)`, and the `@no_runtime_checks`
  v2 reservation;
- again fourteen lines later, with the `establishes_static_fact` exception that
  `OQ-11` decides — and ending in a literal `…`.

`[XXII.4]` makes a duplicate rule id a hard CI failure, so the document fails
its own new invariant. Worse than the duplication: the trailing `…` means the
second statement was never meant to stand alone, and the first is missing the
exception, so **neither is the rule**.

**Decision.** Merge, keeping the bullet's position in the contract list — where
a reader comparing `@static_safe` against `@noalloc` will look — and deleting
the later duplicate. The merged rule opens with the second statement's sentence
including the exception, then continues with the first's carve-out and
reservation.

**Consequences.** One definition. `OQ-11`'s decision is now stated where the
contract is introduced rather than fourteen lines below it, and `[EFF-15]`'s
citation of `[EFF-12]` resolves to one rule.

**Note.** The merge makes `[EFF-12]` depend on a reason code that `[EFF-11]`
did not define. See ERR-015.

---

## ERR-011 — `[PAR-2]` is stated twice, once inside `[PAR-1]`'s paragraph

**Where.** Part XI §4. `[PAR-1]`'s bullet states two rules: its own sentence
about the loop body being compiled as a closure, and then `[PAR-2]`'s
independence requirement in the same paragraph. `[PAR-2]` is then stated again
five bullets later, prefixed `…`, with the fuller (a)/(b)/(c) clauses.

Two consequences beyond the duplicate id: `[PAR-2a]` and `[PAR-2b]` both cite
"clause (b)", which only the later statement defines, and they appeared *before*
it; and the two statements disagree — the first requires the write index to be
"exactly `i` or `i + const`", the second requires every access to `P` including
**reads** to share one constant offset, which is strictly stronger.

**Decision.** `[PAR-2]` becomes its own bullet immediately after `[PAR-1]`,
carrying the later, stronger text. `[PAR-1]` keeps only its own sentence. The
trailing duplicate is deleted. `[PAR-2a]` and `[PAR-2b]` now follow the clause
they cite.

**Why the stronger reading.** It is the later text, it is the one `[PAR-2a]`
and `[SIMD-5]` are written against — `[SIMD-5]`'s vectorisable form cites
"`[PAR-2]`(b)" by name — and a loop-carried dependency through a *read* is
exactly the case a weaker rule would admit and `@parallel` cannot survive.

---

## ERR-012 — `[MOD-6]` lost its first half

**Where.** Part V §1. v0.5 states the rule as:

> `[MOD-6]` … Mismatch with the compiler's supported set is `E0006`. …

The `…` stands where v0.2 said what the rule was *about*: that a module may
declare `#! language "0.2"` on its first line, and that the package's
`ember.toml` `language` key is the default. Without it, `[MOD-6]` names a
mismatch between two things it no longer identifies, and the `#!` directive —
which Part III §1's grammar still defines, and which `[LEX-14]`/ERR-006 turn on
— has no normative rule anywhere in the document.

**Decision.** Restore the first half verbatim from the committed split of v0.2,
with the example version updated to `"0.5"`. The 0.4 sentence that follows is
kept unchanged.

**Provenance.** The restored text exists in no copy of v0.5. It was taken from
`docs/spec/part-05-declarations-and-semantics.md` as committed at `58cb057`.

---

## ERR-013 — `[CLO-2]` lost its capture-mode rule

**Where.** Part VI §5. v0.5 states:

> `[CLO-2]` … A closure that moves a captured non-`Copy` value out of its own
> storage implements `CallableOnce` but not `Callable` (`[CLO-6]`).

The `…` replaced the rule that decides **how closures capture at all**: read-only
use ⇒ shared borrow; mutation ⇒ mutable borrow, so the closure needs a mutable
place to call; `owned fn` ⇒ move, copy or retain. Nothing else in the document
states it. `[CLO-1]` says a closure "is a view type if it captures anything by
reference (the default)" and `[CLO-4]` depends on which captures are borrows,
but neither says which variables are captured which way.

**Decision.** Restore the capture-mode sentence, keeping 0.4's `CallableOnce`
tail — which correctly supersedes v0.2's trailing "`E3030` in v1
(once-callable closures are not supported)".

**Why this one matters most of the six.** Block E — the NLL borrow checker,
the largest remaining piece of Phase 2 — has to implement exactly this rule, and
`[CLO-6]`'s once-callable design is built on top of it. Had `docs/spec/` been
regenerated from v0.5 before the diff was read, the rule would have been lost
with no copy anywhere.

---

## ERR-014 — `[FFI-6]` is a fragment, and its pipeline contradicts `[FFI-6b]`

**Status: fragment repaired; the contradiction is a proposal, deferred to
Phase 5.**

**Where.** Part XVI §2. `[FFI-6]`'s body is the import pipeline diagram, which
says the AST walk takes:

> object-like macros that expand to integer/float/string literals, function-like
> macros are ignored (W5001)

0.4 then appended a `…`-prefixed `[FFI-6]` widening the object-like case to any
constant expression "after full macro expansion" — integer, float, string,
null-pointer-constant or pointer/handle cast — and added `[FFI-6b]`, under which
a **function-like** macro *may* be exposed as a function when an overlay declares
its signature. The diagram and the rules now disagree on both halves.

**What was applied.** Only the fragment repair: `…` becomes **Macro import.**,
so `[FFI-6]` reads as a rule. The diagram is left as written.

**What is left open.** Whether the pipeline diagram is amended to defer to
`[FFI-6]`/`[FFI-6b]`, and what `W5001` means once `[FFI-6b]` exists — a warning
for every un-overlaid function-like macro would fire on every real header. This
is Phase 5 work and nothing before it depends on the answer.

---

## ERR-015 — `establishes_static_fact` is used four times and defined nowhere

**Where.** `[EFF-11]`'s reason-code table has exactly four rows:
`not_provable_in_principle`, `not_proven_by_analysis`, `requested_by_type`,
`inherent_to_mechanism`. The rule says "Every emitted-check entry carries
**exactly one** reason, and diagnostics MUST use its wording".

A fifth code, `establishes_static_fact`, is used by `[DSJ-5]`, by `[DSJ-6]`, by
`[EFF-12]` as merged in ERR-010, and by `OQ-11`'s decision text — and 0.4's own
change log row 4 calls it "the new reason code". It is in no table.

As written the rule is unimplementable: `[EFF-10]`'s side table must carry one
reason per site, `[EFF-12]` must test for this one, and Phase 4's exit criteria
fix the expected reason for every check site in `tests/safety/reasons/`.

**Decision.** Add the fifth row:

> | `establishes_static_fact` | the check verifies a property once and returns
> proof-carrying values, so the property is static from there on (`[DSJ-1]`,
> `[DSJ-5]`) | nothing; the check is what makes the code after it checkable, and
> `[EFF-12]` permits it under `@static_safe` |

**Consequences.** `[EFF-11a]`'s rule — that reporting the wrong reason is a
diagnostic bug — now has a fifth code to get right, and `[DIA-11]`'s S1 shape
has a fifth case: when the reason is `establishes_static_fact` the check is not
a contract violation at all, so S1 must not fire.

**Why it is a decision and not a proposal.** The wording is new text, but the
code's existence and meaning are already settled by `OQ-11` and by 0.4's change
log. Only the table row was missing.

---

## ERR-016 — The standard-interface list cannot be parsed

**Where.** Part IV §8's fenced block listing the interfaces the compiler knows
about — `Clone`, `Drop`, `Eq`, `Ord`, `Add`, `Index`, `Iterator`, `Iterable`,
`Callable`, `Error`, `From` and the rest. It is laid out as an aligned table,
every declaration on one line:

```
interface Iterator:              type Item; fn next(mut self) -> Option[Item]
interface IntoIterator:          type Item; type Iter: Iterator[Item = Item]; fn into_iter(owned self) -> Iter
```

**Two independent reasons it does not parse.**

1. `[GRM-18]` (`OQ-25`) removes `;` as a statement separator, so the six rows
   that use one are `E0105`. This is v0.5 outlawing a form its own normative
   listing depends on — the ruling is right and the block was not revisited.
2. Underneath that, `interface_decl` is
   `"interface" identifier … ":" NEWLINE INDENT {interface_member} DEDENT`.
   There is **no same-line body form** for an interface, unlike `fn_decl`, whose
   `block` admits `simple_stmt NEWLINE`. So *every* row is unparseable, not only
   the six with semicolons, and always was. `[LEX-9]`'s same-line carve-out does
   not reach it either: that permits "a single simple statement", and an
   interface member is not a statement.

**Why it is not cosmetic.** `[TST-7]` requires every fenced `ember` block in
Parts I–XVII to pass `ember check --syntax-only`, with a recorded baseline of
blocks excused by `,ignore`. The document has 48 `ember` blocks and exactly one
is excused; this is not it. So the block that defines what the compiler must
implement — `a + b` lowers to `Add.add`, `for x in v` to `Iterable.iter`
(`[TYP-21]`, `[CTL-1]`) — is a block the conformance gate will reject.

**Decision.** Rewrite the block in the indented form, one member per line,
keeping every declaration and every comment. Twenty rows become sixty-odd lines;
the column alignment is lost and nothing else changes. The two-line comment
listing the other operator interfaces (`Sub`, `Mul`, … `Not`) becomes a `#`
comment above `Add`, since it documented a group rather than a declaration.

**The alternative, rejected.** `[TST-7]` permits excusing a block whose reason
is "a `std` signature sketch", which this is, and that costs one line instead of
sixty. Rejected because it exempts from checking the one list whose exactness the
whole operator and iteration story depends on — and the block was already wrong,
in two ways, with nobody noticing.

**Checked across the whole document afterwards.** No other `interface`,
`struct`, `class`, `enum` or `extend` declaration in any fenced block puts its
body on the header line, and no other block uses `;` as a separator (the one
remaining `;` is inside a string literal in a `comptime` example).

**Not applied to the compiler.** The parser has never accepted the same-line
interface form, so nothing there changes. `E0105` remains outstanding under
ERR-003.

---

## ERR-017 — `From`'s required method could not be declared

**Status: decided by the owner, 2026-09-08. `from` is contextual.**

**Where.** Part IV §8 declares the interface `?` is specified in terms of:

```ember
interface From[T]:
    fn from(owned value: T) -> Self
```

`[ERR-2]` lowers `expr?` to `Err(F.from(e))`, `[ERR-7]` supplies the identity
conversion, and `[ERR-8]` the erasure to `Box[dyn Error]`. All three name the
method `from`. But `from` was in Part II §4's reserved table, because
`from a.b import x` begins with it — so `fn from(…)` did not parse, and neither
did a user's `extend MyError implements From[io.Error]:`.

**Decision.** `from` joins the contextual keywords (`[LEX-15]`): a keyword only
where it begins an import at item level, an ordinary identifier everywhere
else. The reserved set goes back to **48** — `type` joined under ERR-009 and
`from` leaves here.

**Why this and not the alternatives.** Requiring `r#from` costs nothing in the
compiler but makes the specification's own text wrong and puts an escape
sequence in the first interface a beginner meets. Renaming the method to `of`
or `convert` parses most easily and reads worst: every neighbouring language
calls it `from`. The contextual reading is the one under which the document is
already correct, and the position is unambiguous — an import is the only thing
that may start with `from` at item level, and `parse_module` already dispatches
on the first token of a line.

**The general rule the owner set with it:** when a reserved word collides with
a name the standard library must be able to declare, make the word contextual
rather than renaming the member or demanding `r#`.

**Applied to.** Part II §4's table and `[LEX-15]`'s sentence; `Kw` and
`CONTEXTUAL_KEYWORDS` in `ember_lexer::token`; `parse_import` and
`parse_module` in `ember_parser`. Covered by
`from_is_contextual_so_that_From_can_declare_it`.

**Found by** `tools/spec_check.py`, written this session to satisfy `[TST-7]`.
It compiles every fenced `ember` block in the document, which is how a
five-line interface nobody had tried to parse turned out to be unparseable.

---

## ERR-018 — Two `assert` examples are written as a statement, not a call

**Status: decided, 2026-09-08. Both examples corrected.**

**Where.** Part VI §6 specifies the form: `assert(cond)`, `assert(cond,
"msg")`, `assert_eq(a, b)`, `assert_ne(a, b)`. Part XV lists `assert*` among
the functions `std.core` provides, and `assert` is not in the keyword table, so
it is an ordinary call.

Two examples wrote it as a keyword taking a bare expression:

- Part XIV §1: `assert size_of[Vertex]() == 32, "Vertex layout changed; …"`
- Appendix A: `comptime: assert size_of[Vec3]() == 12`

Neither parses. Appendix A is `[TST-6]`'s compile-pass fixture, so its failing
is a CI failure by construction.

**Decision.** The grammar wins, as it did for ERR-008: `assert` is a function
and the examples take parentheses. Both corrected in the source document, and
Appendix A's block now parses.

**The state of `[TST-7]`'s gate.** Of the 38 `ember` blocks in Parts I–XVII and
Appendix A, **14 parse and 24 do not**. The 24 are fragments — bare statements
at file level, which `[GRM-2]` forbids — and are recorded in
`tools/spec_check_baseline.json` so the gate fails only on new breakage.
Shrinking that baseline is what `[TST-7]` exists to drive.

---

## ERR-019 — `[RT-5]` forbids the runtime from spelling its own symbols

**Status: decided, 2026-09-08. The runtime's own two files are exempt.**

**Where.** `[RT-5]` says: "No file in the compiler, **runtime**, CMake module,
examples or test corpus may hard-code the symbol prefix …". Its next two
sentences say the opposite for the runtime specifically: "`ember_rt.h` is a
**generation output** carrying literal identifiers, not a header of macro
concatenations — it is the interface document C embedders read, and it must
stay readable."

Both cannot hold. `ember_rt.h` declares `ember_alloc`, `ember_retain` and
seventy more; `ember_rt.c` defines exactly those. Routing them through a macro
is the thing the second sentence forbids, and it would make the header
ungreppable for the embedders it is written for.

**Decision.** `runtime/ember_rt/include/ember_rt.h` and
`runtime/ember_rt/src/ember_rt.c` are exempt from `tools/check_branding.py`,
and the prefix is defined once on the C side as `EMBER_SYMBOL_PREFIX` in the
header. The compiler holds the same constant once, in `ember_branding`. The
two definitions must agree; nothing mechanical enforces that today, and the
first symbol that disagrees is a link error — worth a build-time assertion when
the runtime gains a generation step.

**What the rule buys elsewhere.** Occurrences outside those two files went from
96 to 13. The 13 are fixture file names inside tests (`"test.em"`,
`"src/main.em"`, milestone paths), recorded in
`tools/check_branding_baseline.json`. They are left because a rename breaks
them *loudly* — a failing test is the opposite of the silent link error the
rule exists to prevent.

**Proof the refactor changed nothing.** The emitted C for all 17 test programs
is byte-for-byte identical before and after. That check is also what caught the
first attempt: `{RT}` in a string passed to `line()` rather than to `format!`
emitted the braces literally, and no test would have noticed.

---

## ERR-020 — `[TST-6]`'s Appendix A fixture describes a different appendix

**Status: fixture created; the `compile-pass` requirement is deferred and the
two stale descriptions are recorded.**

**Where.** `[TST-6]`: "`docs/spec-source/appendix-a.em` is a conformance
fixture annotated `#$ test: compile-pass`, and Appendix A's code block is
generated from it by `tools/spec_check.py --emit-appendix`. The fixture's
directive is `#! language "0.3"`; `...` bodies are `pass`; the statement tail
is wrapped in `fn demo():`; `match` arms are written in statement form."

Three of those describe an appendix that v0.5 no longer has:

- **the directive.** The fixture would say `0.3`; Appendix A says `0.5`, and
  `[MOD-6]`'s supported set — now actually checked, `E0006` — holds only
  `0.5`. A `0.3` fixture would fail to compile on the first line.
- **`match` arms in statement form.** v0.5 rewrote the example as
  `return match s:` with `=>` arms, because `OQ-14` made jumps expressions.
  The sentence describes the shape ERR-008 produced and `OQ-14` replaced.
- **`compile-pass`.** The appendix names `Entity`, `CommandList`, `Formatter`,
  `SoA[Particle]`, `Arena` and `Mutex`. None exist: they are Phase 2 to Phase 6
  library types. A full `ember check` reports 29 unresolved names.

**What was done.** The fixture exists and is faithful — `--emit-appendix`
regenerates Appendix A from it byte-for-byte, which is the property `[TST-6]`
actually wants: the quick reference cannot drift away from something that
parses. It carries `0.5` and the `=>` arms, matching the document rather than
the description of it.

It is annotated `#$ test: syntax-pass` and held to `ember check --syntax-only`,
which is what `[TST-7]`'s gate runs over every block in the document. It
becomes `compile-pass` when `std` supplies the types it names, and the
annotation says so in the file rather than in a note nobody reads.

**The two stale sentences in `[TST-6]` are left as written.** They describe the
document's history accurately and correcting them would mean rewriting a rule
to match a fixture, which is the wrong direction. This entry is the record.

---

## ERR-021 — Seven borrow shapes have no error code, and `[DIA-7a]` forbids inventing one

**Status: decided, 2026-09-08. `E3021`–`E3027` allocated.**

**Where.** `[DIA-7a]` keys every code in `E3000–E3499` to a diagnostic shape
and ends: "A code absent from this table MUST NOT be emitted."

Its table keys fourteen codes. §XIX.6.1 lists twenty-one shapes. Seven of the
shapes have no code anywhere in the document:

| Shape | What it is |
|---|---|
| **B1** | two mutable borrows through computed indices |
| **B3** | a shared borrow live across a mutating call |
| **B4** | aliased mutation of a value type |
| **B5** | a self-referential struct |
| **B8** | a method takes all of `self`, defeating `[BRW-4]` |
| **B9** | a closure outlives its captures |
| **B10** | a `mut` argument is not a mutable place |

**B1 and B3 are `[BRW-1]` itself** — aliasing XOR mutability, the rule the
whole value world rests on. As the document stands, a compiler that rejects
`ref mut n` twice cannot say so: every code it might use is forbidden by
`[DIA-7a]`, and the shape it must cite has none.

**Decision.** Allocated beside `E3020`, which is the one aliasing error the
table does key:

| Code | Shape | Title |
|---|---|---|
| `E3021` | B3 | a shared and a mutable borrow overlap |
| `E3022` | B1 | two mutable borrows of the same place |
| `E3023` | B4 | aliased mutation of a value type |
| `E3024` | B5 | a struct field would borrow another field of the same struct |
| `E3025` | B8 | a method takes all of `self` |
| `E3026` | B9 | a closure outlives what it captures |
| `E3027` | B10 | a `mut` argument is not a mutable place |

All seven are in the registry now, because `[DIA-6a]` makes it exhaustive
whether or not the compiler emits a code yet. `E3021` and `E3022` are emitted
by the borrow checker; the rest wait on the constructs they describe.

**Not a fork.** There was no second sensible answer: the codes had to exist,
the range and the neighbouring number were determined, and each maps to exactly
one shape §XIX.6.1 already specifies down to its required `help`. Recorded
rather than asked about, and `[DIA-7a]`'s table should gain these seven rows in
the next revision the owner writes.

---

## ERR-022 — `[DIA-7]` and `[DIA-7a]` disagree about what the classifier covers

**Status: decided, 2026-09-08. `[DIA-7]`'s scope is the real one.**

**Where.** `[DIA-7]`: "Every **ownership or borrow error** MUST be classified
into one of the shapes in §XIX.6.1". `[DIA-7a]`: "The table below maps **every
error code in `E3000–E3499`** to the shape whose `help` it MUST emit. A code
absent from this table MUST NOT be emitted."

Those are different sets. The range also holds `E3100`, `[UNS-1]`'s "this
operation requires an `unsafe` block" — which is not an ownership error, is not
a borrow error, and which none of §XIX.6.1's twenty-six shapes describes. Under
`[DIA-7a]` read literally, the compiler may not emit it at all.

**Decision.** `[DIA-7]`'s scope governs: the classifier covers ownership and
borrow errors, not every code that happens to sit in the numeric range.
`compiler/ember_diag/src/shapes.rs` carries the exception explicitly, as a
named list with this entry cited, so that it is a decision rather than an
oversight.

**Why not renumber `E3100` instead.** It would be the tidier answer — the code
belongs in a range for `unsafe`, which the registry does not have — but ERR-001
already records what renumbering costs, and that cost rises with every phase.
`E3100` is emitted, tested and documented. One line of exception is cheaper
than moving it, and the exception is checked: a test asserts every *other*
ownership code has a shape, so a genuinely unclassified borrow error cannot be
added without the build going red.

**Found by** writing that test. The list of exceptions is one entry long, which
is the useful outcome: it says the classifier's coverage is otherwise complete.

---

## ERR-023 — `[TYP-14]` reads as though only a *named* reference dereferences

**Status: decided, 2026-09-09. Amended in the document.**

**Where.** Part IV §2: "Auto-dereferenced: `r.field`, `r.method()`, and use of
`r` in an expression of type `T` all read through."

Every example names `r`, a reference held in a variable. Read literally that is
a rule about a *name*, and the compiler implemented exactly that: reading a
local of reference type where a value was wanted read through, and nothing else
did. A call that returns `ref i32` was then neither an operand nor an argument:

```ember
fn give(a: ref i32) -> ref i32:
    return a

m: i32 = give(ref n) + 1     ## E2020: `+` cannot be applied to `ref i32`
println(give(ref n))         ## emitted C passing `const int32_t *` as `ember_str`
```

The second is the one that matters. It is not rejected — it compiles, and the
*C compiler* catches it. An Ember program should not be able to reach a C type
error, and `[CG-C-1]` says the emitted C is warning-free, which this was not.

**Decision.** The rule is about the type in a value context, not the form the
reference was written in. Amended to say so, naming the cases: a named
reference, a call that returns one, a field read, an operand, an argument.

**What would change if the owner rules the other way.** If reading through is
meant to be a property of names only, then a call returning `ref T` needs a
written dereference at every use, and the language needs a spelling for it —
there is none today, and `[LT-6]` reserves nothing for it either. The
implementation would go back to rejecting both lines above, and `println` would
need an explicit overload for reference types rather than a coercion.

**Found by** writing a test that printed a returned reference. Nothing in the
corpus returned one before the borrow checker made them expressible.

---

## ERR-024 — §4.7 step 6 says "a local", and a by-value parameter is not one

**Status: decided, 2026-09-09. Amended in the document.**

**Where.** Part XVIII §4.7 step 6: "A loan whose region extends beyond the
borrowed place's storage (`StorageDead`/`Drop` of **a local** while a loan on it
is in scope) is `E3060`."

Beside it, `[LT-1]`: a returned view's region is a **view-typed** parameter's.
Read together the intent is clear, but step 6 is the sentence a borrow checker
is written from, and its parenthesis names locals only. The implementation
exempted every parameter on exactly that reading, with a comment saying "the
caller owns what it points at" — true of a view-typed parameter and false of a
copy:

```ember
fn peek(self) -> ref i32:
    return ref self.n        ## compiled; returns a pointer into a dead frame
```

`self` here is passed by value. The copy lives in the callee's frame and dies
with it, so the borrow is exactly as short-lived as a borrow of a local.

**Decision.** Step 6 names a by-value parameter alongside a local, and states
that only a view-typed parameter names storage the caller keeps. This is a
clarification, not a change of rule: nothing in `[LT-1]`, `[LT-3]` or `[TYP-15]`
ever said an owned parameter could be borrowed and returned.

**What would change if the owner rules the other way.** Nothing in the
implementation — there is no reading under which the program above is sound.
The amendment exists so the next person writing to step 6 does not repeat the
exemption.

**Found by** the same test run as ERR-023.

---

## Two defects that needed no amendment, recorded so the question is not re-asked

`[FMT-1]` ("the formatter preserves comments", `parse(fmt(x)) ≡ parse(x)`) and
`[DIA-3]` ("MUST include the 'later used here' label") were both **right and
unambiguous** while the compiler did neither. The formatter deleted doc
comments on methods and variants; no borrow diagnostic carried the label. Those
are implementation gaps and are in `docs/DEFECTS.md`, not here. An
implementation gap is not a spec defect — ERR-014, ERR-019 and ERR-022 are what
that mistake costs when it is made the other way.

---

## What is in the document, and what was withdrawn

Every entry above is applied to `docs/spec-source/ember-spec.md`, which is the
normative copy. Five were held back at first and patched on 2026-09-08 when the
owner asked for them:

- **ERR-014** — **the contradiction recorded here did not exist, and the owner
  said so.** The entry claimed the import diagram and `[FFI-6b]` disagreed
  about function-like macros. They do not: the diagram describes the *Clang AST
  walk*, and the overlay is applied at a later step of the same pipeline. A
  macro ignored during the walk and supplied by an overlay afterwards is
  sequential, not contradictory, and `[FFI-6b]` says exactly that — "MAY be
  exposed … when an overlay declares its signature".

  A first attempt to "fix" it made things worse by moving the function-like
  case into the walk step, which misdescribes the ordering. That is reverted:
  the walk ignores them with `W5001`, and the overlay step now says it is where
  `[FFI-6b]` acts, which was the one thing the diagram left implicit.

  What *was* stale is smaller and real: the diagram described object-like
  macros as ones "that expand to integer/float/string literals", while
  `[FFI-6]` widened them to any constant expression of integer, floating,
  string-literal, null-pointer-constant or pointer/handle-cast type, after full
  macro expansion. The diagram now defers to `[FFI-6]` for that list rather
  than restating a narrower one.
- **ERR-019 is withdrawn: it was not a defect.** The entry claimed `[RT-5]`
  forbids the runtime from spelling its own prefix while requiring the header
  to carry literal identifiers, and that both cannot hold. They can, and the
  rule says how in its first sentence: the header is **generated**. A generator
  writes the literal identifiers from `EMBER_SYMBOL_PREFIX`, so the header
  greps and no *source* file spells anything.

  The change made on the misreading is reverted, because it excused the runtime
  from ever being generated — it turned an implementation gap into a permanent
  exception to a rule that was already correct.

  What is true is smaller: our `ember_rt.h` and `ember_rt.c` are hand-written.
  `tools/check_branding.py` exempts them and says why, and `BACKLOG.md` carries
  it as `RT-GEN-1`.
- **ERR-020** — `[TST-6]` described an Appendix A with a `0.3` directive and
  statement-form `match` arms, neither of which v0.5 has. Corrected to describe
  the appendix that exists.

  **The `compile-pass` requirement is left as written.** A first attempt
  softened it to `syntax-pass` "until `std` catches up", which is rewriting a
  requirement to match what the implementation can do — the wrong direction.
  The fixture is held to `--syntax-only` today; that is the implementation
  falling short of the rule, recorded as `TST-6-1` in `BACKLOG.md`, not the
  rule being wrong.
- **ERR-021** — `[DIA-7a]`'s table gained the seven rows for `E3021`–`E3027`,
  so the codes the compiler emits for `[BRW-1]` are keyed where the rule
  requires.
- **ERR-022 is withdrawn.** `[DIA-7a]` sits directly under `[DIA-7]`, which
  scopes the classifier to "every ownership or borrow error"; read together,
  `[DIA-7a]`'s "every error code in `E3000–E3499`" means every *such* code in
  that range. `E3100` is in the range and is neither, so no shape describes it
  and none should. That reading needs no change to the document, and the edit
  that narrowed the rule is reverted.

  `compiler/ember_diag/src/shapes.rs` still names the exception rather than
  omitting it, so a genuinely unclassified borrow error fails the build. That
  was always the useful part.

`docs/spec-source/as-received/Ember_v0.5_spec.md` is still byte-identical to
what the owner sent, so every one of these divergences can be diffed and each
has a reason written above it.

---

## ERR-025 — `yield` is a v1 keyword, and the table that lists it says otherwise

**Status: decided. `[LEX-15b]` governs; the reserved set is 49.**

**Where.** Part II §4 lists `yield` under **"Reserved for future use (lexed as
keywords, `E0005` if used)"**, alongside `actor`, `async`, `await`, `macro`,
`move`, `trait`, `union`, `loop` and `unless`. `[LEX-14a]` requires `E0005`'s
message to **name the version that will introduce** the word.

`[LEX-15b]`, added by 0.6.3, says the opposite and says it explicitly:

> `yield` is **fully reserved** — a keyword everywhere, with `r#yield`
> (`[LEX-14]`) required to use the name — because `[CORO-2]` makes it an
> expression form, and an expression keyword cannot be contextual without
> ambiguity at the start of a statement. […] `[LEX-15]`'s reserved set
> therefore has **49** entries, not 48; **this rule supersedes that count and
> no other part of it.**

`[CORO-2]` then uses `yield` as a v1 expression and gives `yield` outside a
`gen fn` its own code, `E2220`. A word cannot simultaneously be `E0005`
("reserved for a future version") and `E2220` ("used outside a `gen fn`").

**Decision.** `[LEX-15b]` is later, is explicit that it supersedes, and is the
only one of the two that can be implemented — `E0005`'s message has no version
to name for `yield`, because the version that introduces it is this one.

* the reserved keyword set is **49**: Part II §4's 48 plus `yield`;
* the reserved-for-future list is **9**: `actor`, `async`, `await`, `macro`,
  `move`, `trait`, `union`, `loop`, `unless`;
* `gen` is **contextual** — a keyword only immediately before `fn`, an
  ordinary identifier everywhere else, so a field or variable named `gen` is
  unaffected. It is in neither list.

**If the owner rules the other way** — that `yield` stays future-reserved —
then `[CORO-2]`'s `yield` expression, `[GRM-22]`'s `yield_expr` and `E2220` all
go with it, and coroutines lose their suspension syntax. That is a language
change, not an editorial one.

**Applied to.** `ember_lexer::token`'s keyword table and its keyword-count
assertion, the future-word table, and `E0005`'s message.

---

## ERR-026 — `a in b in c` is reported with the unknown-attribute code

**Status: decided. `E0102`.**

**Where.** `[GRM-23]`:

> `membership := expression ("in" | "not" "in") expression`, at **comparison
> precedence** — the same level as `==` and `<` — and **non-associative**:
> `a in b in c` is `E0104` […]

`E0104` is `[ATT-1]`'s **unknown attribute**. `[DIA-6a]` is unambiguous: "**A
code MUST be defined by exactly one rule.** `tools/rule_index.py` MUST fail CI
when two rules name the same code with different titles."

**Decision.** `E0102 chained comparison` already exists, is already in the
parser's range, and already covers exactly this: Part III §5's precedence table
puts `in` and `not in` on the same non-associative row as `==` `!=` `<` `>`
`<=` `>=` `is` `is not`, and says of that whole row "chaining `a < b < c` is
`E0102` (no Python chaining)". `a in b in c` is a chained comparison by the
same rule that gives `in` its precedence. No new code is needed and none is
invented.

**Applied to.** `[GRM-23]`'s sentence in the source document; the parser emits
`E0102` for a chained membership operator exactly as it does for `<`.

---

## ERR-027 — Four leftovers from the contract layer 0.6.2 removed

**Status: decided. Struck.**

**Where.** 0.6.2's change log row 1 is exhaustive: `@requires`, `@ensures`,
`@invariant`, `@decreases`, `@verified`, `@assume`, the `[CTR-*]` and `[PRV-*]`
rules, sections X.2a and X.2b, proof manifests, the `--contracts` and
`--verify` flags and the *contract* `proven` grade all go. `OQ-27` confirms it
as an owner decision. Four sites still refer to what went:

1. **`[UNS-7]`** — "where an obligation *is* expressible as a contract
   expression the function SHOULD additionally carry `@requires`". There is no
   `@requires`, it is in no table in Part III §7, and `[ATT-1]` makes an
   attribute absent from that table `E0104`. The sentence asks for a program
   the compiler must reject.
2. **`[STD-6]`** — the layer list ends "**verify** (proof-only helpers)". The
   proof-only helpers were the prover's.
3. **`[EFF-18]`** — "`RuntimeCheck(k)`'s kinds are `{Bounds, Overflow,
   Aliasing, Stale, Contract}`", and **`[EFF-17]`** — "Bare `@nopanic` (v2) […]
   does not forbid `RuntimeCheck(Contract)`" and "`@no_runtime_checks` (v2,
   reserved) excludes all five kinds". §X.1.1's table defines **four** kinds and
   `[EFF-9]`, `[EFF-11]`, `[EFF-12]` and `[COST-3]` all reason over four. A
   `Contract` check is a check of a contract expression, and there are none.
4. **XX §6's registry paragraph** — "Codes added by 0.6 and 0.6.1 […]
   `E4050`–`E4057` and `E4060`–`E4064` (contracts and verification)", and the
   warning `W4001` (`result`/`old` shadowing). `result` and `old` are
   `@ensures` vocabulary. `[DIA-6a]` requires every registry entry to cite a
   rule that exists; these thirteen codes and one warning cite rules 0.6.2
   deleted.

**Decision.** All four struck. The kinds are four: `Aliasing`, `Bounds`,
`Stale`, `Overflow`. The layers are five: core, alloc, sync, io, ffi.
`[UNS-7]`'s `@safety("…")` prose obligation stands unchanged and is the whole
of that rule. `E4050`–`E4057`, `E4060`–`E4064` and `W4001` are **not**
registered and are not reused, in the manner XXIII.4 requires of a superseded
id.

**Consequence.** `@no_runtime_checks` (v2, reserved) excludes **four** kinds,
not five. `[EFF-17]`'s sentence about `RuntimeCheck(Contract)` says nothing
once the kind is gone and is struck rather than reworded.

---

## ERR-028 — `[EFF-18]` enumerates the effect set without `Nondet`

**Status: decided. Ten effects.**

**Where.** Part X §1's opening sentence:

> The compiler infers, for every function, an **effect set** ⊆ `{Alloc, Sync,
> Lock, Io, Panic, Unsafe, FFI, Block, Nondet, RuntimeCheck(k)}`.

`[EFF-18]`, five paragraphs later:

> The full set is `{Alloc, Sync, Lock, Io, Panic, Unsafe, FFI, Block,
> RuntimeCheck(k)}`.

**Decision.** `[EFF-18]` was written for 0.6, which added `Io` and `Lock`.
`Nondet` arrived in 0.6.3 with `[DET-1]`/`[DET-2]`, and 0.6.3's change-log row 3
adds it to X.1 without revisiting `[EFF-18]`'s parenthetical count. X.1's set
governs and `[DET-2]` enumerates `Nondet`'s sources exhaustively. `[EFF-18]`'s
own sentence — "`[EFF-18]` does not remove an effect previously attached to any
operation; it refines the effect model" — says it is not trying to be an
exclusive list.

**Applied to.** `[EFF-18]`'s enumeration gains `Nondet`. Nothing else changes:
`@noio`, `@nolock`, `@noblock` and `@nosync` remain independent, which is what
the rule is for.

---

## ERR-029 — Ten editorial instructions shipped inside rule bodies

**Status: decided. Read as carried out.**

**Where.** 0.8.1's change-log row 2 records this as the third time the document
family has done it, names `[STD-7]` and `[FFI-17]` as the earlier two, and asks
`[DIA-6a]`'s completeness pass to grow a check for imperative second-person
prose in rule bodies. Ten sites in v0.8.3 carry one:

| Rule | Shape |
|---|---|
| `[BLD-2]` | a trailing fragment adding the `[verify]` package-config section to the cache key — and `[verify]` is ERR-027's removed layer |
| `[FFI-34]` | *Replace "Claiming a grade whose evidence is absent is `E5050`" with: "…"* |
| `[FFI-38]` | *Strike "or exception behaviour" … and add: "…"* |
| `[FFI-2a]` | *After "…calling convention", insert: "…"* |
| `[BLD-11]` | *replace "(`E1020`)" with "(`E1021 …`)"* |
| `[TCB-5]` | a sentence relocating the evidence record to `.ember/ffi-evidence/`, appended after the rule already gave `target/<profile>/ffi-evidence/` |
| `[FFI-33b]` | an appended quoted clause fixing when the creating thread is recorded |
| `[TST-13]` | *add: an instrumented run in which …* |
| `[RNG-7]` | the amendment appended in quotation marks, opening with an ellipsis |
| `[RNG-8]` | the whole rule body is a quoted fragment opening with an ellipsis |

**Decision.** In every one of the ten the replacement text is present in full
beside the instruction, so the rule's meaning is recoverable and the
instruction is read as carried out. Specifically:

* `[BLD-2]`'s trailing fragment is struck (ERR-027 removes what it adds);
* `[FFI-34]`'s operative text is the quoted replacement — an unbacked or stale
  grade is `W5050`, the fact is reported and used at `asserted`, and `E5050` is
  raised only under `--require-evidence` / `tcb --require` / `audit --require`;
* `[FFI-38]` no longer covers exception behaviour, which `[FFI-24]`'s catch-all
  establishes universally;
* `[FFI-2a]`'s unknown-filling list gains `borrowed`, `from =` and the aliasing
  words **in return position only**;
* `[BLD-11]` reports `E1021`, and `E1020` stays `[GRM-4]`'s;
* `[TCB-5]`'s default evidence path is `.ember/ffi-evidence/`, relocatable by
  `[MAN-5]`'s `[ffi] evidence` key;
* `[FFI-33b]`, `[TST-13]`, `[RNG-7]` and `[RNG-8]` read with their appended
  clauses as part of the rule.

**Why this is recorded rather than silently applied.** `[PHIL-5]`'s discipline
applies to reading a document as much as to an analysis: an instruction is not
its result, and the next revision authored from this one will carry these
forward again unless someone can point at the list.

---

## ERR-030 — `[GRM-16]` deletes three `small_stmt` alternatives that III.4 still lists

**Status: decided. The rule governs; already implemented that way.**

**Where.** Part III §4:

```
small_stmt      := var_decl | assignment | expression | "return" [expression] | "break" [label]
                 | "continue" [label] | "pass" | "defer" ":" ...
```

`[GRM-16]`:

> Part III §4's `small_stmt` alternatives `"return" [expression]`,
> `"break" [label]` and `"continue" [label]` **are removed**; a jump written as
> a statement is an expression statement.

**Decision.** `[GRM-16]` is the rule and the production is the text it amends;
`OQ-14` confirms jumps are expressions of type `!` at the lowest precedence.
The compiler already parses them as `ExprKind::Jump` and there is nothing to
change — this entry exists so that a later reading of III.4 does not "repair"
the parser backwards. ERR-008 recorded the same reversal against v0.5.

---

## ERR-031 — `[TST-11]`'s v0.6 obligations carry a merge artefact

**Status: decided. Read as the surrounding clauses state.**

**Where.** `[TST-11]` ends with two sentence fragments spliced together, the
splice falling inside the words "v0.6" and "std.ser.yaml". The surviving pieces
name their rules, so the obligations are legible:

* `[TCB-6]`'s **evidence invalidation** after a foreign identity or contract
  hash changes;
* a **`std.ser.yaml`** round trip with an out-of-range field producing
  `SerError` rather than an invalid value — `[RNG-10a]`'s derive obligation;
* `[RNG-10b]`'s rejection of a range type in an `extern` block and behind a
  pointer parameter;
* `[RNG-7]`'s `Option[Full]` occupying two bytes.

**Decision.** The four obligations above are the v0.6 regression set and are
what `tests/conformance/TST-11/` must contain, alongside the v0.5 set the rule
lists in full before the artefact.

---

## ERR-032 — `[TST-7]` says two blocks carry `,ignore`; they do not

**Status: decided. The blocks are fenced as the rule describes.**

**Where.** `[TST-7]`, repaired by 0.8.1 precisely to close this:

> A fourth permitted reason is **overlay-language source, until Part III
> defines `overlay_decl`**; XVI.4's `overlay c "vulkan/vulkan.h":` block and
> XVI.7a's `overlay cpp "RageV/VulkanBackend.hpp":` block **carry it**, so the
> baseline is explicit rather than implied.

Both blocks are fenced as ordinary `ember` in v0.8.3. `tools/spec_check.py`
therefore reports them as newly failing, which is exactly the outcome 0.8.1's
row said it had prevented — the instruction was applied to the rule's prose and
not to the fences.

**Decision.** Both are fenced `ember,ignore` with the one-line reason on the
preceding line, which is what the rule says of them. Part III still defines no
`overlay_decl`, so the opt-out is the correct state and not a concession.

---

## ERR-033 — Four rules state themselves twice

**Status: decided. The second copy is redundant.**

`[LEX-15a]`, `[GRM-20]`, `[GRM-8d]` and `[FFI-34a]` each contain their rule
text twice, the second copy differing only in wording. `tools/rule_index.py`
counts a definition once — the id opens the bullet — so this trips no gate, and
in all four the two copies agree.

**Decision.** Read as one rule. Recorded because a future revision editing one
copy and not the other produces exactly the class of defect 0.8.2b's rows 1 and
2 were fixing, and because `[LEX-15a]`'s two copies already differ in what they
list: the first names `extern_item` and the second does not.

---

## ERR-034 — `[FFI-17d]` cites a rule that does not say what it cites it for

**Status: open. Reported to the owner.**

**Where.** `[FFI-17d]`:

> A `@ffi(trampoline)` type whose base has no virtual destructor MUST declare
> `owner=`; **`[FFI-17b]`'s `@ffi(no_virtual_dtor)`** covers the remaining
> case, and omitting both is `E5059`.

`[FFI-17b]` is "Templates are available only as explicit instantiations". It
says nothing about destructors and does not mention `@ffi(no_virtual_dtor)`.
That attribute is named nowhere else in the document: it is in no table in
Part III §7, so `[ATT-1]` makes writing it `E0104`.

**Why this is not resolved here.** The other nine `[FFI-*]` citations in the
same paragraph resolve correctly, so this is a wrong id rather than a missing
rule — but there is no way to tell *which* rule was meant, because no rule
defines the attribute. The two readings are (a) `@ffi(no_virtual_dtor)` is a
real attribute whose defining rule was lost, in which case Part III §7 needs a
row and some `[FFI-*]` id needs the text; or (b) it was dropped in favour of
`owner=` alone, in which case `[FFI-17d]`'s clause goes and `E5059` fires
whenever `owner=` is absent on such a base.

**Provisional choice, per Part XXI §1 ground rule 3 and the guiding sentence in
"How to use this document":** reading (b). `owner=` alone is sufficient — under
`owner="foreign"` Ember never runs the C++ destructor, and under
`owner="ember"` the pair is destroyed through the trampoline subclass, whose
own destructor is the derived one. `E5059` is then "a `@ffi(trampoline)` base
with no virtual destructor and no `owner=`", which is what its registry title
already says. This is Phase 7 work; the decision is recorded now so it is not
taken silently later. `docs/DECISIONS.md` carries it as ADR-013.

---

## ERR-035 — Five sections sit outside the Part they are numbered for

**Status: decided. Recorded, not moved.**

| Section | Numbered for | Physically inside |
|---|---|---|
| `## VIII.5a Cycle diagnosis` | Part VIII | Part XV |
| `## IX.5a Unsafe categorisation` | Part IX | Part XV |
| `## XX.13 The C++ importer corpus and migration gate` | Part XX | Part XVI |
| `## XVI.7a C++ exception policy` | Part XVI | Part XX |
| `## Compile-time budget` (`[BUD-*]`) | — (unnumbered) | Part XX |

`[IDE-1]`'s bullet also sits between Part XXI's heading and §XXI.1, before the
part's own first section.

**Decision.** `tools/split_spec.py` cuts on `# Part` headings only, so each of
these lands in the file for the Part it is physically inside. That is a
faithful split of the normative document and is left alone: moving text in the
source to tidy the split would be a hand edit of the specification, which
Part XXI §1 ground rule 3 forbids for exactly this reason. Rule ids are
unaffected — `[WK-4]`, `[UNS-9]`, `[CXX-*]`, `[FFI-43]` and `[BUD-*]` are found
by id, not by file. `docs/spec/README.md` records the placement so a reader
looking for `[WK-4]` in Part VIII is told where it is.

---

## ERR-036 — Part III admits no item-level `extern "C" fn` declaration

**Status: open. Reported to the owner. Two blocks in `[TST-7]`'s baseline.**

**Where.** XVI.10 writes, and `[FFI-26]`, `[FFI-28]`, `[FFI-31b]` and `[HR-21]`
all depend on:

```
@export("rv_script_on_update")
pub extern "C" fn on_update(entity: u64, dt: f32) -> i32: pass
```

Part III §2's `fn_header` is:

```
fn_header       := ["unsafe"] ["virtual" | "override"] "fn" identifier [generic_params]
                   "(" [param_list] ")" ["->" type] [where_clause]
```

There is no `["extern" string_lit]` prefix. The only place `extern` may precede
`fn` in the grammar is `fn_type` (a function *type*, `["extern" string_lit]
"fn" "(" …`) and `extern_block` (`["unsafe"] "extern" string_lit ":"`, which
declares foreign functions Ember calls, not Ember functions foreign code
calls). `pub extern "C" fn on_update(…)` therefore parses under no production.

**Why it matters and why it is not just an example's slip.** `[FN-6]` says a
capture-free function "coerce[s] to `extern "C" fn(A) -> R` when their types are
FFI-safe" — a coercion, not a declaration form. But `@export` needs a
*definition* with the C calling convention, and `[FFI-31b]`'s `E5015` is
specified over "an `@export` or `@export_table` signature", which presupposes
one. The Appendix A fixture does not exercise it, which is why the gap survived.

**Two readings.**

* **(a) The prefix belongs on `fn_header`.** `fn_header` gains
  `["extern" string_lit]` after the optional `unsafe`, admitted only at item
  level and only on a function with no generic parameters (the calling
  convention has no meaning for an uninstantiated generic). This is the
  smallest change and matches what every example writes.
* **(b) `@export` supplies the convention.** `extern "C"` is dropped from the
  example and `@export("name")` alone means "C ABI, C symbol", since `[MNG-2]`
  already says `@export` overrides the symbol entirely. This is fewer tokens but
  makes the convention invisible at the declaration, which `[PHIL-6]` argues
  against for boundary-crossing costs.

**Provisional choice: (a).** It is what the document's own examples write in
three places, it keeps the convention visible at the declaration, and it leaves
`[FN-6]`'s coercion untouched. Recorded as ADR-014. This is Phase 5 work; the
grammar change is not made until the owner rules, and until then XVI.10's block
sits in `[TST-7]`'s baseline rather than being fenced `,ignore` — the baseline
is where a gap the gate exists to expose belongs.

---

## ERR-037 — Part III admits no `extern class` declaration

**Status: open. Reported to the owner. In `[TST-7]`'s baseline.**

**Where.** `[FFI-39]`, forty lines of v1 specification, writes:

```
@ffi(trampoline, virtuals=["OnAttach", "OnDetach", "OnUpdate", "OnEvent"])
extern class cpp.RageV.Layer:
    init(name: CppString)                       # names a C++ base constructor
```

Part III §2 has `class_decl`, which admits no `extern` and whose name is an
`identifier`, not a dotted path; and `extern_block`, whose `extern_item` is
`fn_header NEWLINE | static_decl | "type" identifier NEWLINE` — no class. The
member `init(name: CppString)` is also written without `fn`, which
`type_member` does not admit either.

`[FFI-39]` is explicit that this is not the opaque form — "`[FFI-8]`'s
opaque-`extern type` form remains available and remains unsized; the two are
different declarations and only this one may be inherited" — so it cannot be
read as `extern type` with attributes.

**What is missing**, precisely: a production for a foreign base declaration; a
rule that its name may be a path into a synthetic `cpp` module; and a member
form declaring a base constructor's signature without a body.

**Provisional choice.** None taken. Unlike ERR-036 there is no smallest-change
reading: the declaration introduces a name that is not an Ember item (it is a
view of a foreign type, per `[FFI-30a]`), its members are signatures of foreign
constructors, and `[CLS-4]`'s "at most one base class, written in parentheses"
has to be reconciled with `class DebugOverlay(cpp.RageV.Layer)` naming one.
Writing a production without the owner's ruling would be deciding the shape of
the C++ inheritance surface silently, which Part XXI §1 ground rule 3 forbids.

`[FFI-39]`'s block sits in `[TST-7]`'s baseline. This is Phase 7 work and
nothing before it depends on the answer.

---

## ERR-038 — `E2213` is defined by two rules

**Status: decided. One title covers both.**

**Where.** IV.2a's diagnostic list:

> `E2213` an `in` clause on a non-numeric representation.

`[GRM-8d]`:

> a `range_clause` on an `interface_member` associated type or on an
> `extern_item` opaque type is `E2213`. A `type_alias` carrying both
> `generic_params` and a `range_clause` is `E2213`.

`[DIA-6a]`: "**A code MUST be defined by exactly one rule.**
`tools/rule_index.py` MUST fail CI when two rules name the same code with
different titles."

**Decision.** Neither site states a *title*; both state a condition, and the
registry supplies the title. The three conditions are one thing — the `in`
clause is not admissible here — differing only in why: the representation is
not numeric, the position admits no range clause, or the alias is generic and a
range is over a concrete representation. The registry entry is

> `E2213` — invalid `in` clause on a type alias — `[RNG-1]`

and `[RNG-1]` is cited as the defining rule, because it is `[RNG-1]` that says
what an `in` clause is and what it may be written over; `[GRM-8d]` constrains
where it may appear, which is a constraint on the same rule's construct.

**Why not split it.** Allocating a second code would be inventing a number the
document does not name, and `[DIA-6a]`'s check is satisfied by one title. The
three conditions are distinguished in the message, which is where a programmer
reads them.

---

## ERR-039 — `E9010` is defined by two rules

**Status: decided. `[TYP-9c]` keeps `E9010`; `[MAN-3]` takes `E9012`.**

**Where.** `[TYP-9c]`:

> If the host toolchain cannot honour `@fastmath` or `@fp(…)` at function
> granularity, the compiler MUST report `E9010` naming the toolchain and the
> attribute.

`[MAN-3]`:

> Every key in `[lints]` MUST name a lint the compiler defines (`E9010`
> otherwise).

Two unrelated errors, one number — the same class `[BLD-11]`'s `E1020`/`E1021`
was fixed for in 0.6, and the one `[DIA-6a]` polices.

**Decision.** `[TYP-9c]` keeps `E9010`. It is the older claim, it is stated in
Part IV where the float-control rules live, and `E9011` beside it is
`[TYP-9a]`'s companion, so the pair reads as one subject.

`[MAN-3]` takes **`E9012`**. XX §6 allocates `E9012`/`E9013` to 0.6 and
describes them only as "(manifest sections)", assigning neither a rule nor a
meaning. `[lints]` is a manifest section and `[MAN-3]` is a rule about one, so
the code is put to the use its own description names rather than a new number
being invented.

`E9013`, the other half of that pair, is assigned to `[MAN-5]`'s `[ffi]`
section — the only other manifest section 0.6 and 0.6.1 introduced — as
"invalid `[ffi]` manifest section".

**If the owner rules otherwise**, the two registry entries change and nothing
else does: neither code is emitted by any code path yet, and both are manifest
diagnostics, which `[CAT-1]` categorises `TOOLCHAIN-NORMATIVE` rather than
language-normative.
---

## ERR-040 — `--syntax-only`'s code ranges no longer describe what the parser emits

**Status: decided. The stages are the constraint, not the ranges.**

**Where.** `[CLI-9]`:

> `ember check --syntax-only <file>` lexes and parses the file and reports only
> `E00xx` and `E01xx` diagnostics. It does not resolve names, so an example
> naming undeclared types still passes.

`[GRM-8d]`, added by 0.6:

> a `range_clause` on an `interface_member` associated type or on an
> `extern_item` opaque type is `E2213`. A `type_alias` carrying both
> `generic_params` and a `range_clause` is `E2213`.

Both conditions are decidable from the parse tree alone, and `E2213` is in
neither range `[CLI-9]` names. Reading `[CLI-9]` as a filter would make
`--syntax-only` accept `type Bad[T] = T in 0 ..= 1`, which `[GRM-8d]` refuses,
and `[TST-7]` — which runs every fenced block through exactly this command —
would then be unable to see the class of error `[GRM-8d]` exists to catch.

**Decision.** `--syntax-only` runs the lexer and the parser and reports
everything those two stages produce. `[CLI-9]`'s second sentence is the
operative constraint and is unaffected: no names are resolved, so an example
naming undeclared types still passes. The code ranges were a shorthand for
"what the front end produces", written before `[GRM-8d]` gave the parser an
`E2xxx` code to emit.

**Consequence for the test harness.** `tests/conformance/` gains the annotation
kinds `parse-pass` and `parse-fail`, which run this command. They are how a
rule whose *grammar* has landed ahead of its semantics gets a real `[TST-4a]`
accept-and-reject pair rather than a directory holding an aspiration.

---

## ERR-041 — `[FN-1]` and Part VII §7 disagree about a `mut` view argument

**Status: decided. Part VII §7's example governs; ADR-017 records the reading.**

**Where.** `[FN-1]` on the `mut` mode:

> `mut b: B` — **inout** (mutable borrow). The argument MUST be a mutable
> place; the callee may mutate; no move out (except by `mem.replace`/`take`).

Part VII §7, four lines of its own worked example:

```ember
fn normalize(mut xs: MutSpan[f32]):
    …
normalize(buf.as_mut_span())          # or simply normalize(buf)
left, right = buf.as_mut_span().split_at(1)
```

`buf.as_mut_span()` is a **call result**. It is not a place, so under `[FN-1]`
read literally the document's own example is `E2140`, and `split_at` — which
`[SPN-*]` names as "the sanctioned way to obtain multiple mutable borrows into
one container" — cannot be called on one either.

**Decision.** Where a `mut` parameter's declared type is `MutSpan[T]`, the view
is passed **by value**, and `[FN-1]`'s mutable-place requirement lands on
whatever the view was taken *of*.

**Why this is the reading and not a relaxation.** A `MutSpan[T]` already **is**
a mutable borrow: it carries the pointer, and `[SPN-3]` makes it move-only
precisely so that there is exactly one of it — which is the guarantee `[BRW-1]`
obtains from `ref mut`. Wrapping one in a `ref mut` would make a reference to a
reference whose second level guarantees nothing the first does not, and
`[BRW-6]`'s "passing a `ref mut` local to a `mut` parameter reborrows rather
than moving it" is the same observation about the same shape.

**What a rule would say.** If the owner wants this written down rather than
inferred, the minimal form is an amendment to `[FN-1]`: *"Where `B` is itself a
borrow — `ref mut T`, `MutSpan[T]`, or a `@view struct` carrying one — the
argument is that borrow and the place requirement applies to what it was taken
of."* That is one sentence and it covers `split_at`'s result, `chunks_mut`'s,
and `columns_mut`'s, all three of which Part VII §7 and `[BRW-5]` name.

**If the owner rules the other way** — that `[FN-1]` is literal and a `mut`
view parameter takes a `ref mut MutSpan[T]` — then Part VII §7's three example
lines change, `split_at` grows a binding before every use, and the fix is one
line in `mut_param_ty`.

---

## ERR-042 — Nine rule ids are cited and defined by no rule — **WITHDRAWN**

**Status: withdrawn 2026-09-10. The entry was wrong, and this says so rather
than quietly restating a different outcome.** The owner ordered an inventory
(ruling 5, 2026-09-10): classify each id as a genuinely missing rule, a stale
reference, or an accidental citation, repair where the semantics are already
established, and flag anything needing a new semantic decision as an owner
question.

**The inventory found no genuine gap, and the entry's own list was stale.** The
tool's live `dangling_references` set is **twelve**, not nine: it does not
contain `[IDE-1]` or `[IDE-7]` (both since resolved), and it does contain
`[CTL-3a]`, `[HOT-10]`, `[RC-2a]`, `[RC-2d]` and `[VER-7]`, which the entry
never mentioned. All twelve:

| ids | what they actually are | repair |
|---|---|---|
| `[TYP-26]`, `[IFC-2]`, `[HND-2]`, `[GPU-7]` | **defined**, as the *second* rule stated on a line it shares with its predecessor (`[TYP-25]`, `[IFC-1]`, `[HND-1]`, `[GPU-6]`), after a semicolon | none — the rule is stated in full |
| `[VER-7]`, `[CTL-3a]` | **defined** inline in a paragraph and in a parenthetical respectively, with no bullet of their own | none |
| `[RC-2a]`, `[RC-2d]` | **defined collectively.** `[RC-2]` says in as many words: *"Its lettered clauses are individually citable as `[RC-2a]`..`[RC-2d]` in the order written."* The document grants the ids explicitly | none |
| `[IDE-2]`, `[IDE-5]`, `[IDE-10]` | **deliberately reserved.** Part XXI: *"`[IDE-1]`, `[IDE-2]`, `[IDE-5]`, `[IDE-7]`..`[IDE-10]` are **reserved** for the language server itself, a named milestone before 1.0"* | none, ever — reserving an id is not failing to define it |
| `[HOT-10]` | a **historical citation** in Part 0's change history, naming `[HOT-1]`..`[HOT-10]` from a revision that "replaced them entirely". Part XVIII defines no `HOT-*` rule because the family was renamed `HR-*` | none — a change log that names superseded rules is doing its job |

**Why the checker reports them, and why that is not a defect either.**
`rule_index.py`'s `rule_definitions` recognises a definition structurally: the
id must open a bullet, and **only the first id on a bullet counts**. Its own
comment says why — *"A later one is cited by it, even when the citation reads
like a rule. This is `[XXII.4]`'s 'a reference is not a definition' made
mechanical."* That strictness is deliberate and correct: loosening it would
admit false negatives, and a genuinely undefined rule slipping through is worse
than a known-good one sitting in a baseline.

**So neither side moves.** The document states these rules; the checker cannot
see the forms they are stated in; the twelve are baselined and the ratchet
records them. Nothing here required an edit to the specification, and none was
made.

**The one thing left for the owner**, raised as a question rather than acted on:
six of the twelve (`[TYP-26]`, `[IFC-2]`, `[HND-2]`, `[GPU-7]`, `[VER-7]`,
`[CTL-3a]`) *could* be made structurally findable by splitting each onto its own
bullet with its text verbatim — an `EDITORIAL REPAIR` that changes no meaning.
That is a layout edit to the owner's prose for a tool's benefit, which is the
wrong direction by this project's cardinal rule, so it was **not** done. If the
owner wants the checker to find them, that is the cheapest route; the
alternative is teaching the detector the inline and granting-sentence forms and
accepting the false-negative risk.

**What the entry got wrong, kept for the pattern.** It asserted "defined by no
rule" for nine ids without reading the lines they appear on. Four are defined in
the very sentence that cites them. This is the ERR-014 / ERR-019 / ERR-022
family — a claimed contradiction that dissolves on reading both halves — and it
is the fourth time. `feedback-implementation-gap-is-not-a-spec-defect`'s test
applies verbatim: *if the entry cannot quote the two sentences side by side and
say why they cannot both hold, there is no defect.* ERR-042 quoted none.

---

## ERR-042 — the original entry, kept for the record

**Status: open. Reported to the owner.** Found mechanically, by the check
fix-list item 17 asked for.

**How it was found.** ERR-034 was one instance of this: `[FFI-17d]` cited
`@ffi(no_virtual_dtor)` as something "`[FFI-17b]` covers", and `[FFI-17b]` is
about template instantiation and covers no such thing — following the citation
to the right rule was impossible, because **no rule defined the attribute at
all**. That is a defect a reader cannot even diagnose: the trail simply ends.

`tools/rule_index.py` now checks that every rule id the document mentions is
also stated somewhere, and reports the ones that are not. Nine survive as
genuine.

**The nine.** Each appears **exactly once** in the whole document, in a
citation, and nowhere as a rule:

| Cited | At | Cited for |
|---|---|---|
| `[TYP-26]` | Part IV, the parameter-mode list | the no-overloading rule that `function_value` depends on |
| `[IFC-2]` | Part IV §8 | the orphan rule for interface impls |
| `[HND-2]` | Part IX | the index/generation split of a `Pool` handle |
| `[GPU-7]` | Part XVII | resource state after a full wait-idle |
| `[IDE-1]`, `[IDE-2]`, `[IDE-5]`, `[IDE-7]`, `[IDE-10]` | Part XX | **the entire IDE surface** |

The `IDE-*` family is the one to look at first. Part XXIII's rule-index
paragraph lists `IDE` among the prefixes and says where it is specified —
Part XX — and Part XX cites five of its rules in a single line and defines
none of them. So the language-server contract is referenced, budgeted for and
never written.

`[TYP-26]` is the one that already cost something: `function_value`'s comment
reasons from "`[TYP-26]`'s no-overloading rule" to justify giving a named
function the `fn(A) -> R` type directly rather than a unique zero-sized one.
That reasoning may well be right, and the rule it rests on cannot be read.

**What is not in the list, and why.** Three shapes look like this and are not
defects, so the check excludes them: a range written `[RC-2a]`..`[RC-2d]`,
where only the endpoints are written out; a deliberate historical mention
(`[HOT-1]`..`[HOT-10]`, which Part XXIII says to ignore precisely because they
were replaced); and an id defined in a shape the detector reads as a citation.
That last one took three passes to get right — a test tight enough to reject
every citation also rejected `**Integer overflow** `[TYP-8]`:` and
`* Parameter modes `[FN-1]`:`, reporting thirty rules as undefined that the
document defines perfectly well. The detector now over-counts definitions on
purpose: a missed dangling reference costs less than a gate nobody trusts.

**What this needs from the owner.** Either the nine rules, or a note that the
citation is to a rule the document no longer carries. Nothing in the compiler
turns on any of them today, so this blocks nothing — but `[IDE-*]` is a
Part XX deliverable with no text behind it, and `[TYP-26]` is load-bearing for
a decision already taken.

---

## ERR-043 — `UnsafeCell` is named as the primitive and defined by no rule

**Status: open. Reported to the owner.**

**Where.** `[CELL-9]`, once, in the whole document:

> A package that requires an interior-mutability primitive with no check uses
> `unsafe` (`UnsafeCell`, `[UNS-*]`), which is visible in review and in `grep`.

That is the only occurrence of the word. `[UNS-5]` enumerates what `std.mem`
provides — `MaybeUninit[T]`, `transmute[A, B]`, `ptr.copy_nonoverlapping`,
`mem.zeroed[T]()` — and `UnsafeCell` is not among them. `[UNS-4]`'s invariant
list does not mention it either.

**Why this one is not a hardening.** `RangeError` was named and undeclared too,
and that *was* a hardening (A3): its meaning was never in doubt, only its home.
`UnsafeCell` is the opposite. It is the one construct that suspends `[BRW-1]` —
"aliasing XOR mutability", the guarantee the whole language is built to keep —
and nothing in the document says what suspending it permits, what the `unsafe`
block then owes under `[UNS-4]`, whether the contents may be reached as a `ref`
or only as a raw pointer, or how `[TYP-15]`'s storage rules see it. Writing that
would be deciding what Ember means.

**What is blocked, and what is not.** Nothing in `std` is blocked: ADR-019 takes
the other route, making `Cell`, `RefCell` and `Arena` compiler-known as `Array`
and `Span` already are, which `[CELL-2]`'s "no aliasing rule can be violated"
fully specifies. What is blocked is a **third-party package writing its own**
interior-mutability primitive, which is precisely what `[CELL-9]`'s sentence is
about.

**What it needs from the owner.** A definition — at minimum: what `UnsafeCell[T]`
exposes, what the `unsafe` block promises in exchange, and how `[UNS-4]`'s
"no two views that are simultaneously live may overlap unless both are shared"
is discharged around it. It is a small rule and it cannot be guessed.

---

## ERR-044 — `[TYP-15]` and `[LT-3]` disagree about where a static-region view may be stored

**Status: decided by the owner, 2026-09-09. Amendment S1 accepted; `[LT-3]`'s semantics govern — see the resolution at the end of this entry. The current rules are `[TYP-15]` / `[LT-3]` as written in Ember 0.8.4.**

**Where.** `[TYP-15]` states a principle and then an enumeration:

> A view-typed value MUST NOT be stored in a place whose region is not outlived
> by the view's region. Class fields, non-view struct fields, `static`s,
> `Box[T]` and `Shared[T]` contents, container elements and `owned fn` captures
> have no bounding region and are therefore **always forbidden** (`E3063`).

`[LT-3]` says the opposite for the static-region case, in as many words:

> String literals, `static` items, and `Span`s over them have the `static`
> region, which outlives everything and **satisfies `[TYP-15]`'s storage
> restrictions** (a `str` literal **may** be stored in a class field because
> its type is `str` with static region — the compiler records region `static`
> in the field's type; a non-static `str` cannot be stored there: `E3060`…)

**The contradiction is exact.** `[TYP-15]` lists class fields among the places
that are *always* forbidden; `[LT-3]` says a `str` literal *may* be stored in
one. Both are normative and neither is marked as governing.

**And the two rules name different codes** for the same rejection — `[TYP-15]`
says `E3063`, `[LT-3]` says `E3060` — which is a second, smaller inconsistency
in the same pair.

**Which side is coherent.** `[TYP-15]`'s own principle sides with `[LT-3]`: a
place is forbidden when its region "is not outlived by the view's region", and
a static-region view outlives everything, including a `static`. It is the
*enumeration* that overreaches, by assuming every listed place has no bounding
region — true of a class field holding a borrowed view, false of one holding a
literal, which is precisely the case `[LT-3]` calls out.

**What the compiler does.** It follows `[TYP-15]`'s enumeration:

```ember
static GREETING: str = "hi"
error[E3063]: `str` is a view, so it may not be stored in a `static`
```

That is defensible against the letter of one rule and wrong against the letter
of the other, which is why it is not being changed.

**What is not affected.** A `@view struct` holding a `str` works today, because
`[TYP-14]` gives it a bounding region and no exemption is needed. Classes are
Phase 3 and unbuilt, so `[LT-3]`'s own example cannot be written yet either
way. Nothing is blocked; what is at stake is whether

```ember
static GREETING: str = "hi"
class Label:
    text: str
```

are legal Ember, and the document currently says both yes and no.

**Resolved by the owner, 2026-09-09 — `[LT-3]`'s semantics govern.**

> Long-lived storage is not inherently incompatible with views. A view may be
> stored there when the view's region outlives the destination.

So `[TYP-15]`'s enumeration was the half that overreached, and it now states the
condition rather than a blanket prohibition: those places have no bounding
region, so the only view they may hold is one whose region is `static`.

The exception is on **the view's region, not the destination type**, which is
what keeps it narrow:

```ember
class Foo:
    greeting: str = "hello"        # admitted: "hello" is static-region

fn set(mut foo: Foo, s: str):
    foo.greeting = s               # refused: `s` may be a caller's region
```

**`[TYP-15a]` is untouched.** An owning container at a view type — `Array[str]`,
`Map[str, V]`, `Array[MutSpan[T]]` — stays rejected whatever the region, because
that rejection is at the *type*, and `BorrowList[T]`/`ViewList[T]` remain the
specialised model. A conformance case holds that line.

**The codes are reconciled.** `[LT-3]` named `E3060`; `[DIA-7a]` keys `E3060` to
shape B7 (a borrowed value that does not live long enough) and `E3063` to B12 (a
view stored in a place that outlives it). This is B12, so `[LT-3]` now says
`E3063` — which is what the compiler already emitted.

Recorded as amendment **S1**, class `OWNER-APPROVED SEMANTIC CHANGE`. That class
is the one a hardening may not contain, so which version it lands in is the
owner's call: it either cuts v0.8.4 or rides in Hardened_2 carrying that flag.

---

## ERR-045 — `[LT-1]`'s own example of `@borrows` does not parse

**Status: decided.** `[LT-1a]` and the grammar govern. No compiler change; no
document change.

**Where.** `[LT-1]` closes with an example written as a trailing suffix:

> `@borrows(param)` on a function overrides rule 3 to tie the return to one
> named parameter (e.g. `fn longest(a: str, b: str) -> str @borrows(a)`), which
> lets the caller keep using `b`.

`[LT-1a]` states the position normatively, and differently:

> The attribute `@borrows(p₁, …, pₙ)`, **written on its own line preceding
> the function declaration**, overrides the region that rules 1–3 would assign.

**Why this one is not ERR-044's kind.** Two things settle it, and both point
the same way, so nothing is owed to the owner. Part III §2 admits attributes
only before an item — `item := {attribute} [visibility] item_body` — and there
is no production for an attribute after a return type. And `[LT-1a]` exists
*specifically* to state where the attribute goes, which an example in passing
does not. Written as `[LT-1]` shows it:

```text
error[E0100]: expected the end of the line
```

**What the implementation does.** The form `[LT-1a]` prescribes works and is
`tests/conformance/LT-1a/`. The suffix form is rejected by the parser, which is
the only thing it can be.

**What would change if the owner disagrees** and wants the suffix form: a
production for a trailing attribute on `fn_header`, which would be the first
place in the grammar an attribute may follow the thing it attaches to.

---

## ERR-046 — `[LT-11]` requires mutable-helper behavior that `[LT-8]` does not expose

**Status: decided by the owner, 2026-09-12; resolved in
0.9.5_Hardened_6. ODR-005 closed.** This does not alter the
repository-normative 0.8.5 source.

**Where.** The owner-supplied `[LT-8]` definitions give structural signatures
for `with_views2`, `with_views3`, and `with_views4` whose inputs and callback
parameters are all `Span`. `[LT-11]` then says:

> Two mutable inputs whose sources may alias are rejected by the ordinary
> borrow checker.

`[TST-16]` also requires mutable-alias rejection coverage.

**The conflict.** The safety behavior is coherent, but no public signature says
how a mutable input reaches any helper. `Span` and `MutSpan` are distinct in
Part IV: the former is `Copy`/read-only and the latter is move-only/read-write.
The phrase “exact public spelling MAY use the module's overload convention”
does not define which mutable combinations exist or their callback types.

**What remains fixed.** The shared-`Span` signatures, independent late-bound
regions, no-escape rule, ordinary alias checks, and `@noalloc` requirement are
not in question. H5 recovers them exactly. Nor may this ambiguity be used to
weaken `[LT-11]` for any mutable form the owner chooses.

**Owner resolution.** Keep the shared `with_views2/3/4` family and add distinct
`with_views2_mut`, `with_views3_mut`, and `with_views4_mut` helpers. Every input
and callback parameter in a `_mut` signature is `MutSpan`; mixed shared/mutable
overloads are not part of the ruling. `[LT-11]` remains the governing ordinary
exclusive-borrow and alias-rejection rule.

The ruling wrote `SpanMut[T]`; H6 normalizes that to `MutSpan[T]`, the only
mutable-span type Ember defines and the spelling the ruling was referring to.
This is terminology reconciliation, not a new type or an inferred overload.
Because H5 was already frozen, ADR-023 requires the resolution to be issued as
H6 rather than editing H5 in place.

---

## ERR-047 — the mutable helper signatures cannot express their parameter modes

**Status: decided by the owner, 2026-09-12; fully resolved in
0.9.5_Hardened_8. ODR-006 closed.**

**Where.** H6 `[LT-8]` defines the owner-selected `_mut` family with `MutSpan`
inputs and `fn(MutSpan[...])` callbacks. `[FN-1]` makes every unmarked parameter
a shared borrow that the callee cannot mutate; `[FN-2a]` says the callee's
signature determines the mode. But Part III defines:

```ebnf
fn_type := ["extern" string_lit] "fn" "(" [type {"," type}] ")" ["->" type]
```

No callback parameter mode can be written there.

**Why the type name does not settle it.** `MutSpan[T]` is the correct mutable-
view type, but Ember still distinguishes the type from the parameter mode. The
specification's own `normalize(mut xs: MutSpan[f32])` example uses `mut`, and
`[FN-1a]` exists specifically to admit a mutable view-producing expression at
that `mut` parameter. Treating `a: MutSpan[T]` as silently mutable would create
a special exception to the general shared-parameter rule.

**What is already decided.** ODR-005 remains closed: there are separate shared
and all-mutable helper families, the mutable type is `MutSpan`, and no mixed
overload matrix is implied. This entry does not reopen any of those choices.

**Owner resolution of the callback half.** Callable types now admit the same
borrowed/default, `mut`, and `owned` parameter modes as ordinary declarations.
H7 changes each `_mut` callback to `fn(mut MutSpan[...])`, strengthens
`[LT-11]`, and adds `[LT-11a]` plus `[TST-20]`. A `MutSpan[T]` type does not
silently supply mutable authority. H6 remains frozen as the evidence of the
original boundary. The ruling's mnemonic `[TST-LT-MUT]` is normalized to the
next unused numeric test-rule ID so the conformance obligation is actually
visible to the normative rule index.

**What still needed the owner after H7.** The helper functions' own inputs remained written
`a: MutSpan[A]`. Under `[FN-1]` that is a shared borrow, which cannot provide
the `mut` callback argument. ODR-006 therefore remains open only over whether
those inputs are `mut` reborrows or `owned` consumed values. That choice changes
post-call usability and accepted call sites, so H7 does not infer it.

**Final owner resolution.** Mutable helper inputs are `mut` reborrows, not
`owned` values; shared helper inputs retain the default borrowed mode. H8 makes
those signatures explicit, states that no helper owns or extends the lifetime
of source storage, and adds `[LT-8a]` plus `[TST-21]`. The caller's view remains
usable after the invocation. The supplied `SpanMut[T]` and `[TST-LT-MODE]`
spellings are normalized to canonical `MutSpan[T]` and the next unused numeric
test-rule ID.

---

## ERR-048 — mode-bearing callable types collapse at the `Callable` bridge

**Status: decided by the owner, 2026-09-12; resolved in
0.9.5_Hardened_8. ODR-007 closed.**

**Where.** H7 `[FN-6]` distinguishes `fn(A) -> R`, `fn(mut A) -> R`, and
`fn(owned A) -> R`. `[CLO-3]` still describes `fn(A) -> R` as an implicit
`Callable[(A), R]` bound, while the documented `Callable[Args, R]` and
`CallableOnce[Args, R]` interfaces receive an ordinary tuple of argument
types, not a parameter-mode vector.

**The conflict.** The three callable types must differ for checking and
invocation, yet the specified bridge maps them to the same public argument
tuple. The owner's instruction that callable modes use the ordinary ownership
model rules out silently treating the mode as a new runtime convention, but it
does not decide whether the distinction is compiler-known bound metadata or
part of a revised interface signature.

**What is not in question.** Parameter modes are normative in callable types,
mode mismatches are type errors, no runtime mode dispatch is introduced, and
the mutable callbacks use explicit `mut`. This erratum does not reopen
ADR-026.

**Owner resolution.** Preserve the full mode vector as compiler-known canonical
type metadata through the existing `Callable[Args, R]` abstraction. The public
two-parameter interface stays; the ordinary `Args` tuple notation does not
erase the internal signature. `[FN-6a]`, `[CLO-3]`, and `[CLO-6]` require the
metadata to survive generic bounds, checking, overload resolution, and
monomorphisation, while remaining absent at runtime. No second ownership model,
runtime dispatch, or new public mode-vector generic is introduced.

---

## ERR-049 — an Arena-backed wrapper could not express return provenance

**Status: decided by the owner, 2026-09-12; resolved in
0.9.5_Hardened_9. ODR-008 closed.**

**Where.** `[LT-4]` says an Arena allocation's returned view carries the
arena's borrow region. `[LT-1a]` made `@borrows` the public mechanism for tying
a returned view to a parameter, but also made naming every non-view parameter
E2031. Since `Arena` is intentionally not a view type, the minimal valid
wrapper had no expressible signature:

```ember
fn allocate_one(arena: Arena) -> ref mut i32:
    return arena.alloc(1)
```

**The conflict.** Rejecting every such wrapper makes the Arena borrow
relationship impossible to preserve across an ordinary function boundary;
accepting it with no signature contract lets a caller reset or drop the arena
while retaining the returned reference. Neither is a faithful implementation
of `[LT-4]`.

**Owner resolution.** A growing `Arena` parameter is one narrow exception to
`[LT-1a]`'s view-typed-parameter requirement. A wrapper returning storage
owned by that arena writes `@borrows(arena)`. H9 adds `[LT-4a]`, `[LT-4b]`, and
`[TST-22]`. The annotation records provenance only and does not make Arena a
view, extend a lifetime, transfer ownership, or weaken normal borrow checking.
Omitted provenance is E3061, false provenance is E3062, and arbitrary non-view
parameters remain E2031.

**Normalization.** The ruling's examples used undefined `alloc_span` and
`alloc_mut_span` names. H9 uses the existing `alloc` and `alloc_array` API to
state the same provenance rule and normalizes `[TST-LT-ARENA-RETURN]` to the
numeric `[TST-22]`. No extra Arena method is inferred.

---

## ERR-050 — Arena bulk initialization lacks a complete safety contract

**Status: DECIDED by the owner 2026-09-12; ODR-009 closed in H10. H9 is unchanged.**

**Where.** `[ARN-3]` promises an `alloc_array[T]` result initialized by
`Zeroable` or `Default`, plus `alloc_uninit[u8]` returning
`MutSpan[MaybeUninit[u8]]`. `[UNS-5]` lists `MaybeUninit` and `mem.zeroed`, and
calls `Zeroable` an unsafe marker auto-derived for “all-scalar/POD structs”.
`[UNS-1]` makes `assume_init` and unsafe-interface implementation unsafe.

**The gap.** The document supplies no exact `MaybeUninit` layout, drop/copy,
write, or initialization-transition API and no bit-valid eligibility rule for
`Zeroable`. Treating “scalar” as “all-zero is valid” would be unsound for
non-null references and can be wrong for range or enum validity. The
`alloc_array` fallback/bound, neither-capability behavior, partial-default
initialization, and interaction with `[ARN-3]`'s `needs_drop` rejection are
also unspecified. Phase 2 requires Arena while Phase 4 schedules `Zeroable`
derives, with no bridge stated.

This is not repaired by the compiler's separate inability to parse/check an
explicitly instantiated generic method: the current minimal
`arena.alloc_array[Pixel](2)` probe reports E1010, “only direct calls are
supported in this phase”. That implementation dependency is `GEN-METHOD-1`.

**Why work stopped at H9.** These choices govern valid bit patterns, safe reads of
uninitialized storage, destruction obligations, diagnostics, and the accepted
API. Implementing any one reading would invent unsafe language semantics. The
required owner decision and recommended options were recorded in ODR-009.

**Owner resolution.** H10 extends `[ARN-2]`/`[ARN-3]` and adds
`[ARN-8]`–`[ARN-13]` plus `[TST-23]`: `Zeroable` precedes `Default`; E2040 is
the neither-capability diagnostic; ordinary bulk allocation rejects every
`needs_drop(T)`; Default construction rolls back the cursor; `Zeroable` means
all-zero representation validity; and `MaybeUninit` has exact layout,
non-dropping ownership, safe-write, unsafe-consumption, and full-span rules.
Core `Zeroable` support is Phase 2 while general derives may remain Phase 4.

The follow-up owner ruling fixes the canonical API names. H10 makes the stated
move/consume semantics explicit with ordinary Ember parameter modes and uses
unused continuation IDs because the supplied headings collided with frozen H9
Arena rules. ADR-029 and HC-095-09 preserve both the ruling and normalization.
