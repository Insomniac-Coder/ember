# Specification errata

Places where the specification is silent, self-contradictory, or contradicted
by its own examples, together with what the implementation does about it.

Ground rule 3 of Part XX.1: any deviation from this document requires an ADR
and an entry here. An entry is a **proposal to the owner** unless it is marked
*decided*; the implementation follows the entry so that work can continue, and
each entry states exactly what would change if the owner rules the other way.

| ID | Rule | Status |
|---|---|---|
| ERR-001 | `E0010`, `E0011`, `E0020` | **decided** — renumbered |
| ERR-002 | `[LEX-17]`, Part II §5 grammar | proposed — accepted as an extension |
| ERR-003 | Part II §6 operator table | proposed — `;` added |
| ERR-004 | `[CLS-9]`, Part II §4 | proposed — `let` made contextual |
| ERR-005 | `[LEX-8]` with `[LEX-11]` | proposed — resolved by buffering |
| ERR-006 | Part II §3 with `[TST-1]` | **decided** — annotations moved to `#$` |
| ERR-007 | `[LEX-11]` | **decided** — a dangling doc comment is silent |
| ERR-008 | Part III §109 with Part IV §2, Appendix A | **decided** — `return` stays a statement |

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
`docs/spec/part-02-lexical-structure.md` (§II.3's comment table), the test
harness, and every `.em` file under `tests/`.

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

**If the owner rules the other way**, `return`/`break`/`continue` become
expressions of type `!`. The type system already supports it — `!` coerces to
every type, so an arm yielding `!` sits beside arms yielding `f32` with no
further work. The change is in the parser: `parse_prefix` would need to accept
these three keywords, and `[GRM-10]`'s expression arms would then take them.
