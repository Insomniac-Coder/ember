# Specification errata

Places where the specification is silent, self-contradictory, or contradicted
by its own examples, together with what the implementation does about it.

Ground rule 3 of Part XX.1: any deviation from this document requires an ADR
and an entry here. An entry is a **proposal to the owner** unless it is marked
*decided*; the implementation follows the entry so that work can continue, and
each entry states exactly what would change if the owner rules the other way.

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
| ERR-014 | `[FFI-6]`, `[FFI-6b]`, Part XVI §2 | proposed — fragment repaired, macro contradiction open |
| ERR-015 | `[EFF-11]`, Part X §1.1 | **decided** — fifth reason code defined |
| ERR-016 | Part IV §8 interface list, `[GRM-18]`, `[TST-7]` | **decided** — block rewritten in indented form |
| ERR-017 | `[LEX-15]`, Part II §4, Part IV §8 `From` | **decided** — `from` is contextual; reserved set back to 48 |
| ERR-018 | Part VI §6 with Part XIV §1 and Appendix A | **decided** — `assert` takes parentheses |
| ERR-019 | `[RT-5]` | **decided** — the runtime's own two files are exempt |

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
