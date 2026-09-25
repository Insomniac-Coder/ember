# Ember Programming Language — Specification

**Version:** 0.9.9_Hardened_13
**Supersedes:** 0.9.9_Hardened_12, 0.9.9_Hardened_11, 0.9.9_Hardened_10, 0.9.9_Hardened_9, 0.9.9_Hardened_8, 0.9.9_Hardened_7, 0.9.9_Hardened_6, 0.9.9_Hardened_5, 0.9.9_Hardened_4, 0.9.9_Hardened_3, 0.9.9_Hardened_2, 0.9.9_Hardened_1, 0.9.8_Hardened_3 (development target) and 0.8.5_Hardened_1 (adopted). This document is
the single specification of Ember; the earlier files are history.
**Status:** Consolidated language revision. Language rules are complete for the Core and Systems
profiles; the Native and Dynamic profiles are specified in annexes. Implementation and conformance
are tracked separately (§XVII.9) and are not claimed by this text.
**Authority:** Authored at the owner's direction of 2026-09-23 with delegated authority to decide open
language questions against the project goal. Every decision that changes 0.9.8's meaning is listed in
Appendix H, and every finding of the 2026-09-23 research pass (`tasks/audit/FINDINGS.md`, F-001–F-214)
is resolved in Appendix G. Hardened_2 applies the memory-safety, consistency-and-speed and
ergonomics passes over Hardened_1 (`tasks/spec-0.9.9/passes/`); Appendix H §H.4 lists every change.
Hardened_3 and later record the ODRs raised while implementing 0.9.9 (from ODR-021, ruled under the
owner's delegation); Appendix H §H.5 lists them.

---

## The goal, and how this document is judged

Ember is built to meet three requirements at once:

1. **Fast like C.** Code that uses values, views and plain loops compiles to what a careful C
   programmer would write, with every remaining cost named in §X.4.
2. **Types like Python.** Programs read like Python: indentation, few annotations, inference inside
   functions, literals for lists, maps and sets, f-strings, generators, comprehensions, and errors
   that name the fix. Where Python's meaning and C's meaning differ, Ember takes Python's unless doing
   so costs speed that cannot be recovered, and says so.
3. **Memory safety like Rust, without Rust's ceremony.** Safe Ember cannot use freed memory, race on
   data, or read out of bounds. Ember gets there without lifetime names, without trait-import rules,
   without orphan-rule surprises, and without asking the programmer to prove what the compiler can
   infer.

Each goal has an instrument, and a release is judged by the instruments rather than by this prose:
the performance suite (`[TST-28]`), the first-week newcomer corpus (`[TST-8]`), and the Safe Ember
invariant (`[PHIL-10]`) with its conformance tests.

## How to read this document

| Part | Contents |
|---|---|
| I | Overview, principles, the Safe Ember invariant |
| II | Lexical structure |
| III | Grammar (complete) |
| IV | Types |
| V | Declarations: modules, functions, structs, enums, classes, interfaces, statics, attributes |
| VI | Expressions, statements, closures, generators |
| VII | Ownership, borrowing, regions, destruction, views |
| VIII | Classes and reference counting |
| IX | Memory facilities: heap types, arenas, raw pointers, interior mutability |
| X | Effects, contracts, determinism, the cost model |
| XI | Concurrency |
| XII | Data-oriented programming |
| XIII | Error handling |
| XIV | Compile-time programming |
| XV | Standard library (normative surface) |
| XVI | Foreign function interface (C) |
| XVII | Toolchain, diagnostics and conformance |
| XVIII | Requirements on implementations |
| Annex A | Syntax quick reference |
| Annex B | Hot reload (Dynamic profile) |
| Annex C | C++ interoperation (Native profile) |
| Annex D | GPU host model (library) |
| Appendix E | Coming from Python (non-normative) |
| Appendix F | Glossary |
| Appendix G | Resolution of findings F-001–F-214 |
| Appendix H | Changes from 0.9.8_Hardened_3, from 0.9.9_Hardened_1 (§H.4) and from 0.9.9_Hardened_2 (§H.5) |
| Appendix I | Rule index |

**Normative words.** MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are used as in RFC 2119. Text marked
*Note* or *Example* is not normative.

**Rule identifiers.** Normative rules carry identifiers of the form `[XXX-n]`. A rule identifier is
never reused for a different meaning. Rules carried from 0.9.8 keep their identifiers; a rule whose
meaning this revision changes is marked *(changed in 0.9.9)*; new rules take numbers above every
number the family has used before. Appendix I lists every rule.

**Examples.** Every ` ```ember ` block is a valid program or sequence of items under this
specification and passes `ember check`. A block marked ` ```ember,fragment ` parses but names items it
does not declare; ` ```ember,overlay ` is overlay source (`[GRM-35]`); ` ```ember,ignore ` carries a
stated reason (`[TST-7]`). Blocks marked ` ```text ` are not Ember.

**Silence.** Where this document is silent on a matter of language meaning, an implementation MUST
NOT choose an answer by analogy with another language. It rejects the construct with the
not-yet-specified diagnostic `E0901` (`[PHIL-12]`) and the gap is raised with the owner.

## Versioning

* `[VER-1]` *(changed in 0.9.9)* Three version numbers exist and move independently: the **language
  version** (this document), the **compiler version**, and the **runtime ABI version**
  (`EMBER_RUNTIME_ABI`).
* `[VER-8]` *(new in 0.9.9)* Before 1.0 there is exactly one language: the current one. A source file does not select
  a language version, and `#! language` directives are accepted only if they name the current version
  (`E0006` otherwise, whose help says to delete the line). Selectable language versions begin at 1.0,
  under `[VER-2]`.
* `[VER-2]` From 1.0: source compatibility within a major language version; a breaking change needs a
  new major version that a package opts into with the manifest `language` key.
* `[VER-3]` From 1.0: deprecation through `@deprecated(since, note)`, removal no earlier than the next
  major version.
* `[VER-4]` The runtime ABI (object header, `ember_type_info`, every entry point in `ember_rt.h`) is
  stable within a major version; any change bumps `EMBER_RUNTIME_ABI`. `ember_rt_init` compares the
  runtime's ABI version with the one its caller was compiled against and fails, naming both, on a
  mismatch; fields may be added to `ember_rt_config` only at its end, behind a leading size field.
* `[VER-9]` *(new in 0.9.9)* A language revision that changes the set of accepted programs or their
  meaning moves the language version and resets the hardening number to 1. From 1.0, a hardening adds
  precision without changing any accepted program's meaning (`[VER-2]`). Before 1.0 there is one
  language (`[VER-8]`), so a hardening may change the language, provided its change log lists every
  such change as a change to the language and `ember fmt --migrate` rewrites every program it can. The
  change log of each revision (Appendix H for this one) lists every rule it adds, removes or changes;
  a change without a row is a defect in the document.

---

# Part I — Language Overview

## I.1 One-paragraph description

Ember is a statically typed, ahead-of-time compiled language with Python's indentation syntax and
Python's everyday vocabulary — list, map and set literals, f-strings, comprehensions, generators,
keyword arguments, `in`, `is None`, chained comparisons — compiled to native code with no garbage
collector. Values have ownership; borrowing is checked by the compiler and never written at call
sites; lifetimes are inferred and never named. Classes are reference-counted objects with
deterministic destruction. There is no null, no exceptions, and no undefined behaviour outside
`unsafe`. Performance-critical code uses value types, views, structure-of-arrays containers, arenas
and enforceable contracts (`@noalloc`, `@nosync`). C headers are imported directly. The reference
compiler emits C11.

## I.2 The three tiers

| Tier | How you enter it | What the compiler guarantees | Typical use |
|---|---|---|---|
| **Safe** (default) | nothing | memory safety, null safety, bounds safety, lifetime safety, data-race freedom, deterministic destruction | applications, gameplay, tools |
| **Contract** | `@noalloc`, `@nosync`, `@noblock`, `@nopanic(explicit)`, `@static_safe`, `@deterministic` | everything in Safe, plus the declared contract, checked at compile time | hot loops, real-time and lock-step code |
| **Unsafe** | `unsafe:` block, `unsafe fn`, `unsafe extern` | type checking and diagnostics only; the programmer upholds `[UNS-4]` | allocators, FFI shims, intrinsics |

* `[TIER-1]` *(changed in 0.9.9)* Safe code MUST NOT invoke an operation with an unverifiable precondition unless an
  `unsafe` boundary in the same package discharges it. There are exactly three such boundaries: an
  `unsafe:` block or `unsafe fn` lexically enclosing the call; an `unsafe extern` declaration
  (`[FFI-10]`); an `unsafe overlay` (`[GRM-35]`, `[FFI-2]`). No attribute, profile, manifest key or contract
  introduces a fourth.

## I.3 Principles the language is held to

* `[PHIL-1]` Two syntactically identical declarations in the same context have identical storage,
  lifetime and synchronisation behaviour.
* `[PHIL-2]` *(changed in 0.9.9)* No heap allocation happens that the source does not show. The
  allocating forms are: constructing a type documented to allocate (`class` instances, `Array`,
  `String`, `Map`, `Set`, `Box`, `Shared`); a list, map or set literal or comprehension in a position
  that produces a heap collection; a string literal in a position that produces a `String`
  (`[TXT-9]`); an f-string; a callable value whose captures exceed its inline capacity (`[CLO-10]`);
  and container growth. Each carries the `Alloc` effect and is listed by `ember inspect --alloc`.
* `[PHIL-3]` No implicit copy of a non-`Copy` value occurs. Copies of `Copy` values are bitwise.
* `[PHIL-4]` No implicit synchronisation occurs except through types documented to synchronise
  (`Mutex`, `RwLock`, `Atomic`, channels, `Sync` class handles).
* `[PHIL-5]` Every safety check the compiler removes, it removes because it proved the check
  unnecessary. Declining to look is not a proof.
* `[PHIL-6]` Every expensive or dangerous conversion at an FFI boundary is explicit in source or
  reported by the compiler.
* `[PHIL-7]` Panics are for programmer errors. Recoverable failures use `Result`.
* `[PHIL-8]` **The enforcement ladder.** Ember guarantees memory safety, lifetime safety and data-race
  freedom for Safe code. It proves a property statically where practical; where it cannot but the
  property can be enforced safely at run time, it emits a check; an operation that neither can make
  safe requires `unsafe`. A program is rejected only when no safe enforcement exists **and** the
  intent is expressible some other way — and the diagnostic MUST name that way (`[DIA-7]`).
* `[PHIL-8a]` Every rejection shape in §XVII.6 has a mandated `help` that, applied literally to the
  rejected program, produces a program that compiles. `tests/ui/` holds a before/after pair per shape.
* `[PHIL-9]` Class instances carry a header and can therefore carry dynamic checks; value types
  have no hidden state, because `[TYP-11]` gives every plain struct C layout. Value types that need
  aliased mutation opt into a checked type explicitly (`Cell`, `RefCell`, or a `class`).
* `[PHIL-12]` *(new in 0.9.9)* **No silent acceptance.** Every construct a program writes — an attribute, an import,
  a directive, a contract, a manifest key, a statement form — either has exactly the effect this
  specification gives it, or the program is rejected. An implementation that has not yet built a
  construct rejects it with `E0900 not supported yet by this compiler`, naming the construct and the
  rule; a construct this specification does not define is `E0901`. Accepting a construct and
  ignoring it is a defect of the worst class, because the programmer believes the effect holds.
* `[PHIL-13]` *(new in 0.9.9)* **One meaning per program.** No build profile, compiler flag, manifest key or
  optimisation level changes what a Safe program computes, which branch it takes, whether it panics,
  or whether it is accepted. Profiles change only speed, debug information, and the amount of detail
  in diagnostics and panic messages (`[PRF-1]`). `debug_assert` is the one check a profile turns off; it
  may not have side effects, so it cannot change what a program whose assertions hold computes.
* `[PHIL-14]` *(new in 0.9.9)* **Python spelling, Python meaning.** Where Ember accepts a spelling that Python also
  has — `/`, `//`, `%`, `**`, `a < b < c`, `x in xs`, `x is None`, `[1, 2]`, `{"k": v}`, f-strings,
  `for … else`, keyword arguments — it gives it Python's meaning, restricted to Ember's types. Where
  Ember cannot give Python's meaning at C speed, it rejects the spelling with a fix-it rather than
  giving it a different meaning silently. Integer `/` is the example: rejected, with `//` and a float
  conversion as the fixes (`[TYP-28]`).
* `[PHIL-15]` *(new in 0.9.9)* **Costs are named.** Every cost the language can insert without the programmer writing
  it — a bounds check, an overflow check, a reference-count operation, a dynamic exclusivity check, an
  indirect call, an allocation — has a row in §X.4 and is reported per site by `ember inspect`.

`[PHIL-8]` is what distinguishes Ember from a language whose only safety mechanism is static proof:
it is a rule about **how** a guarantee is met, never about **whether**.

## I.3a The Safe Ember invariant

* `[PHIL-10]` *(changed in 0.9.9)* A program containing no `unsafe` block, no `unsafe fn` and no
  false foreign contract (`[FFI-1]`) cannot perform an invalid ownership operation, a use-after-free,
  a double release, a data race, an out-of-bounds access, an invalid range-type construction
  (`[RNG-9]`), a read of uninitialised memory, an access through an invalid reference, or an
  arithmetic operation whose result C would leave undefined. **This holds in every build profile and
  under every manifest setting**; no configuration disables a check that this invariant relies on.
* `[PHIL-11]` What remains possible, and is therefore not a defect of the guarantee: resource
  exhaustion (a panic, `[ALC-4]`); deadlock; a logical race between correctly synchronised
  operations; a reference cycle leaking (`[WK-1]`); a bug in foreign code reached through a correctly
  declared boundary; a hardware fault; deliberate termination through `panic` or `abort`; and any
  violated obligation inside `unsafe` or an inaccurate `asserted` foreign fact.

## I.4 Static proof and run-time enforcement

Which mechanism enforces a guarantee is decided per property and per program point by the compiler,
never by the programmer — but it is always visible (`ember inspect --safety`) and restrictable
(`@static_safe`, `[EFF-12]`).

| Guarantee | Statically proven when | Checked at run time when | Mechanism |
|---|---|---|---|
| No use-after-free, no dangling reference | always, for references and views | never | borrow checker; zero cost |
| Value aliasing (`ref mut` xor `ref`) | always, for value types | never | borrow checker; zero cost |
| Object aliasing (long-term access through a class handle) | the accesses go through one handle local and are statically ordered (`[EXC-3]`) | otherwise | the field's access word (`[EXC-19]`); a load, a compare and a store |
| Interior mutability of a value | never (the programmer chose it) | always | `Cell` (no check needed) / `RefCell` borrow counter |
| Index in bounds | the index comes from the container's own range, or a range fact (`[RNG-4]`) | otherwise | compare and branch |
| Integer arithmetic does not overflow | a range fact proves it | otherwise | overflow flag test (`[TYP-8]`) |
| Resource handle is live | never (generation is run-time data) | always, every profile | generation compare (`[HND-1]`) |
| Object lifetime | escape analysis promotes to the stack (`[OPT-1]`) | otherwise | reference counts, elided per `[RC-2]` |
| No data race | always, via `Send`/`Sync` | never | type system; zero cost |

## I.5 A complete small program

```ember
from std.math import Vec3

## A particle in a fountain simulation: a plain value, 28 bytes.
@derive(Copy)
struct Particle:
    position: Vec3
    velocity: Vec3
    lifetime: f32

## A reference type with identity; handles are reference counted.
class Emitter:
    name: String
    particles: Array[Particle] = []
    spawn_rate: f32 = 100.0

    fn init(self, name: String):
        self.name = name

    fn spawn(self, count: int):
        for _ in 0..count:
            self.particles.push(Particle(
                position=Vec3.ZERO,
                velocity=Vec3(0, 9.8, 0),
                lifetime=2.0))

## Contract tier: proven not to allocate.
@noalloc
fn integrate(mut particles: MutSpan[Particle], dt: f32):
    for p in particles.iter_mut():
        p.velocity.y -= 9.81 * dt
        p.position += p.velocity * dt
        p.lifetime -= dt

fn main():
    fountain = Emitter("fountain")
    fountain.spawn(10_000)
    for _ in 0..600:
        integrate(fountain.particles, 1.0 / 60.0)
        fountain.particles.retain(fn(p) => p.lifetime > 0.0)
    println(f"{fountain.name}: {fountain.particles.len()} alive")
```

*Note.* `Emitter("fountain")` passes a string literal where a `String` is expected; the literal
allocates the `String` there (`[TXT-9]`). `particles: Array[Particle] = []` is an empty list literal
in an `Array` position. `integrate` receives the class field as a `MutSpan` through a checked write
access to the emitter's `particles` field (`[EXC-1]`, `[EXC-19]`). `1.0 / 60.0` takes `f32` from the parameter it is passed to.

## I.6 Non-goals

Ember deliberately has none of these: a tracing garbage collector or a cycle collector (cycles are
broken with `Weak` and reported, `[WK-15]`); exceptions or unwinding (failures are values or panics,
Part XIII); implicit numeric narrowing; function overloading and variadic functions, apart from the output
functions (`[TYP-26]`); macros (`macro` is reserved); named lifetimes, ever (`[LEX-22]`); specialisation or
higher-kinded types; dynamic or structural typing (`[TYP-40]`); a safety check that depends on the
profile (`[PRF-1]`); a correctness property that depends on whole-program optimisation or LTO; a stable
Ember-to-Ember ABI (the C ABI is the stable boundary, Part XVI); shader compilation and `async` in this
version (both reserved).

---

# Part II — Lexical Structure

## II.1 Source encoding

* `[LEX-1]` Source files are UTF-8. A byte-order mark is accepted and ignored; the formatter removes
  it. An invalid UTF-8 sequence is `E0001`.
* `[LEX-2]` *(changed in 0.9.9)* Line endings are LF or CRLF, both normalised to LF before
  tokenisation. The formatter always writes LF unless the manifest sets `[format] line_endings =
  "crlf"`; output never depends on the host platform.
* `[LEX-3]` The source extension is `.em`.

## II.2 Indentation and line structure

Ember is indentation-sensitive exactly as Python 3 is, with stricter rules:

* `[LEX-4]` Indentation MUST use spaces. A tab at the start of a logical line is `E0002`, with a
  fix-it replacing it by four spaces.
* `[LEX-5]` The lexer emits `NEWLINE`, `INDENT` and `DEDENT` with Python's algorithm: an indentation
  stack starting at `[0]`; a deeper line pushes and emits `INDENT`; a shallower line pops until equal,
  emitting one `DEDENT` per pop; an indentation not on the stack is `E0003 inconsistent dedent`.
* `[LEX-6]` Inside `(`, `[` and `{`, and inside a triple-quoted string, newlines do not end a logical
  line and indentation is not significant.
* `[LEX-6a]` Inside brackets, a lambda's `:` body is a single simple statement ended by the enclosing
  closing bracket or by a `,` at the same bracket depth. `f(fn(x): total += x, 4)` passes two
  arguments. A lambda body needing more than one statement is written as a local function.
* `[LEX-7]` A backslash at the end of a physical line joins it to the next. The formatter never
  emits one.
* `[LEX-8]` Blank lines and comment-only lines do not affect indentation.
* `[LEX-9]` An indented block MUST be introduced by a line ending in `:`. A `:` not followed by an
  `INDENT` is `E0004 expected an indented block`, unless the block is one simple statement on the same
  line (`if x: return`).

## II.3 Comments

```text
# line comment
## doc comment: attaches to the next declaration; Markdown body
# SAFETY: justification for the `unsafe` block that follows ([LEX-23])
#$ test-harness annotation; an ordinary comment to the compiler
#! language "0.9.9"  (only before the imports: a directive, [GRM-37])
```

* `[LEX-10]` *(changed in 0.9.9)* There are no block comments. A line beginning `#!` before the first import or item is a
  directive (`[GRM-37]`), not a comment; anywhere else it is a comment.
* `[LEX-11]` *(changed in 0.9.9)* A `##` comment attaches to the next declaration, ignoring blank
  lines. One not followed by a declaration documents nothing and is discarded in silence. Apart from
  `[LEX-23]`, a comment never affects compilation, including by producing a diagnostic.
* `[LEX-11a]` A `##` comment on a comment-only line is emitted after the `INDENT`/`DEDENT` tokens of
  the next content line, so a doc comment written inside a block attaches inside that block.
* `[LEX-23]` *(new in 0.9.9)* A line comment whose text begins `SAFETY:` or `SAFETY(<category>):`, placed on the line
  immediately before an `unsafe:` block or `unsafe fn`, or at the end of the `unsafe:` line itself,
  is that construct's **safety note**. The lexer records it in the comment side table (§II.7);
  `[UNS-8]` reads it. `<category>` is one of `ffi`, `layout`, `aliasing`, `intrinsic`, `performance`,
  `uninit`.

## II.4 Identifiers and keywords

```text
identifier := XID_Start XID_Continue*          (Unicode; NFC-normalised)
```

* `[LEX-12]` Identifiers are NFC-normalised; two identifiers are the same iff their NFC forms are
  byte-equal.
* `[LEX-13]` A lone `_` is the discard pattern and never names a variable.
* `[LEX-14]` A raw identifier `r#name` uses a keyword as a name (for imported C fields such as
  `type`).

**Keywords** (reserved everywhere; 48):

```text
and        as         break      class      comptime   const      continue
defer      dyn        elif       else       enum       extern     false
fn         for        if         implements import     in         interface
is         let        match      mut        not        open       or
override   owned      pass       pub        ref        return     self
Self       static     struct     super      true       type       unsafe
virtual    void       where      while      with       yield
```

* `[LEX-15]` *(changed in 0.9.9)* The table above is the complete reserved set. **Contextual
  keywords** are keywords only in the stated position and identifiers everywhere else: `abstract`
  before `class`; `from` at the start of an import; `extend` at the start of an item when the type
  it extends follows, directly or after generic parameters (`[GRM-34]`); `gen` and `once` immediately before `fn`; `safe` before `fn` in an extern block; `some`
  at the start of a type; `c` and `cpp` between `import` or `overlay` and a string; `language` and
  `threads` after `#!`; `overlay` at the start of an item, and `rename` and `hide` inside an overlay
  (`[GRM-35]`); `union` at the start of an item followed by a name, where it is reserved for a future
  version (`E0005`), so `Set` can have its `union` method (`[STD-16]`). **Reserved for a future
  version** (lexed as keywords; using one as a name is `E0005`, whose message names the
  reservation): `async`, `await`, `macro`.
* `[LEX-22]` *(changed in 0.9.9)* Ember has no lifetime syntax and never will. A `'` begins a
  character literal and nothing else; an unterminated one is `E0008`.

## II.5 Literals

```text
int_lit     := dec_lit | hex_lit | oct_lit | bin_lit
dec_lit     := digit ("_"? digit)*
hex_lit     := "0x" "_"? hexdigit ("_"? hexdigit)*
oct_lit     := "0o" "_"? octdigit ("_"? octdigit)*
bin_lit     := "0b" "_"? bindigit ("_"? bindigit)*
int_suffix  := "i8"|"i16"|"i32"|"i64"|"i128"|"u8"|"u16"|"u32"|"u64"|"u128"|"isize"|"usize"
float_lit   := dec_lit "." dec_lit exponent? | dec_lit exponent | dec_lit float_suffix
exponent    := ("e"|"E") ("+"|"-")? dec_lit
float_suffix:= "f16" | "f32" | "f64"
char_lit    := "'" char_body "'"                       -- exactly one Unicode scalar value
string_lit  := '"' string_body* '"'
multiline   := '"""' … '"""'                            -- common leading indentation removed
raw_string  := 'r"' … '"' | 'r#"' … '"#'  (up to 8 #)
bytes_lit   := 'b"' byte_body* '"'                      -- type Span[u8], static region
cstr_lit    := 'c"' byte_body* '"'                      -- type cstr, NUL-terminated, static
fstring     := 'f"' (text | "{" expression ("=")? ("!r")? (":" format_spec)? "}")* '"'
escape      := "\n" | "\r" | "\t" | "\0" | "\\" | "\"" | "\'" | "\x" hex hex | "\u{" hex{1,6} "}"
```

* `[LEX-16]` *(changed in 0.9.9)* An integer literal without a suffix is an **untyped integer**: it
  takes the type its context expects (any integer type, or a float type — `Vec3(1, 2, 3)`), and with
  no context it is `int` (`i64`). A value that does not fit the resulting type is `E2010`.
* `[LEX-17]` *(changed in 0.9.9)* A float literal without a suffix is an **untyped float**: it takes
  the float type its context expects, and with no context it is `float` (`f64`). Literals are the only
  values that convert implicitly (`[TYP-4]`).
* `[LEX-17a]` *(changed in 0.9.9)* A float literal that receives `f32` or `f16` and has more significant
  digits than the type keeps (more than 9 for `f32`, 5 for `f16`) produces
  `W2015 literal loses precision at f32`, offering the `f64` spelling. `0.1` at `f32` is not warned: the
  program chose the type, and every decimal literal is rounded to it.
* `[LEX-24]` *(new in 0.9.9)* A unary minus applied directly to an untyped integer literal forms a negative constant
  of the literal's eventual type. If that type is unsigned the program is rejected with `E2010`
  (`-1` does not fit `u32`), and where the context is an index (`xs[-1]`) with `E2011` whose fix-its
  are `xs.last()` and `xs[xs.len() - 1]` (`[TYP-31]`).
* `[LEX-18]` `1.` followed by an identifier character is a method call on `1`; `1.0` is a float.
* `[LEX-19]` *(changed in 0.9.9)* An f-string `{…}` contains a full expression. `{{` and `}}` are
  literal braces. `{expr=}` prints the expression's source text, `=`, then its value (Python's
  debugging form). `{expr}` uses `Display`, or `Debug` when the type has no `Display`, as printing
  does (`[STD-9]`); `{expr!r}` always uses `Debug`. The format spec follows Python's
  mini-language: `[[fill]align][sign][#][0][width][,|_][.precision][type]`, `align ∈ {<,>,^}`,
  `type ∈ {d,x,X,b,o,e,E,f,F,g,G,%,s,?}` (`?` is `Debug`). A spec that does not apply to the value's
  type is `E2250`, naming both.
* `[LEX-20]` *(changed in 0.9.9)* A string literal has type `str` with the static region. At a site
  expecting `String` it produces a `String` (`[TXT-9]`).
* `[LEX-25]` *(new in 0.9.9)* A raw string contains no escapes; a `r#"…"#` raw string may contain `"`. A multiline
  string removes the longest common run of leading spaces from its lines and drops a first line that
  is empty.

## II.6 Operators and punctuation

```text
Arithmetic:   +  -  *  /  //  %  **
Bitwise:      &  |  ^  ~  <<  >>
Comparison:   ==  !=  <  >  <=  >=  is  is not  in  not in
Logical:      and  or  not
Assignment:   =  +=  -=  *=  /=  //=  %=  **=  &=  |=  ^=  <<=  >>=
Range:        ..  ..=
Access:       .  ?.  [ ]  ( )
Cast:         as  as?  as!
Other:        ,  :  ;  ->  =>  @  ?  { }
```

* `[LEX-21]` *(changed in 0.9.9)* Tokenisation is maximal munch: `//=` before `//` before `/`,
  `**=` before `**`, `..=` before `..`, `?.` is one token. `as?` and `as!` are single tokens when `as`
  is followed immediately (no space) by `?` or `!`; `!` has no other use. There is no `::` token; a
  `::` in source is `E0100` with the help `use '.' for paths` (`[GRM-24]`).

## II.7 Token stream contract

The lexer produces tokens `{kind, span, flags}`. `INDENT`, `DEDENT` and `NEWLINE` are real tokens;
doc comments are tokens; other comments are dropped from the stream and recorded, with their spans, in
a side table used by the formatter and by `[LEX-23]`. The lexer never stops at the first error: an
invalid character produces one `Error` token per maximal run of invalid characters (`[DIA-20]`), and
parsing continues.
---

# Part III — Grammar

This is the complete grammar. `{x}` is zero or more, `[x]` optional, `|` alternation; terminals are
quoted or UPPERCASE (`IDENT`, `INT`, `FLOAT`, `STRING`, `CHAR`, `NEWLINE`, `INDENT`, `DEDENT`). The
parser is recursive descent with precedence climbing for expressions; the grammar is LL(2) except
where a note says otherwise. **Every construct used anywhere in this specification appears here**;
a construct in an example that this Part does not produce is a defect in the example.

## III.1 Compilation unit and imports

```text
file          := {directive} {NEWLINE} {import_decl} {item}
directive     := "#!" ("language" STRING | "threads" IDENT) NEWLINE
import_decl   := "import" module_path ["as" IDENT] NEWLINE
               | "from" module_path "import" import_list NEWLINE
               | ["pub"] "import" ("c" | "cpp") STRING [ffi_opts] ["as" IDENT] NEWLINE
module_path   := IDENT {"." IDENT}
import_list   := import_item {"," import_item} [","]
               | "(" import_item {"," import_item} [","] ")"
import_item   := IDENT ["as" IDENT] | "*"
ffi_opts      := "with" "(" ffi_opt {"," ffi_opt} [","] ")"
ffi_opt       := IDENT "=" expression
```

* `[GRM-2]` *(changed in 0.9.9)* A file contains imports and items and, in the **entry file** only
  (the file given to `ember run`/`ember build`, or the manifest's `entry`), statements at file scope,
  which form the implicit `main` (`[FN-8]`). A statement at file scope in any other module is `E0100`
  with the help `move it into a function`.
* `[GRM-37]` *(new in 0.9.9)* A **directive** is a line beginning `#!` before the imports.
  `#! language "<version>"` is accepted only for the current version (`[VER-8]`, `E0006`);
  `#! threads main|any|creator` sets the default thread contract of the file's exports (`[FFI-33]`).
  Any other directive is `E0104`. `import cpp` is Annex C.

## III.2 Items

```text
item          := {attribute} [visibility] item_body
attribute     := "@" attr_name ["(" [attr_arg {"," attr_arg} [","]] ")"] NEWLINE
attr_name     := IDENT {"." IDENT}                   -- dotted names are plugin namespaces
attr_arg      := expression | IDENT "=" expression
visibility    := "pub" ["(" ("package" | "read" | "package" "," "read") ")"]
item_body     := fn_decl | struct_decl | class_decl | enum_decl | interface_decl
               | extend_decl | const_decl | static_decl | type_alias | extern_block
               | comptime_block

fn_decl       := fn_header ":" block
               | fn_header NEWLINE                     -- bodiless: interface, extern, abstract only
fn_header     := ["extern" STRING] ["unsafe"] ["virtual" | "override"] ["gen"]
                 "fn" IDENT [generic_params] "(" [param_list] ")" ["->" type] [where_clause]
generic_params:= "[" generic_param {"," generic_param} [","] "]"
generic_param := IDENT [":" bound_list] ["=" type]
               | "const" IDENT ":" type ["=" expression]
bound_list    := type {"+" type}
where_clause  := "where" type ":" bound_list {"," type ":" bound_list}
param_list    := param {"," param} [","]
param         := ["mut" | "owned"] "self"
               | ["mut" | "owned"] IDENT ":" type ["=" expression]

struct_decl   := "struct" IDENT [generic_params] [implements] [where_clause] ":" type_body
class_decl    := ["open" | "abstract"] "class" IDENT [generic_params] ["(" type ")"]
                 [implements] [where_clause] ":" type_body
implements    := "implements" type {"," type}
type_body     := NEWLINE INDENT {member} DEDENT | "pass" NEWLINE
member        := {attribute} [visibility] (field_decl | fn_decl | const_decl | type_alias
               | "pass" NEWLINE)
field_decl    := ["let"] IDENT ":" type ["=" expression] NEWLINE

enum_decl     := "enum" IDENT [generic_params] [implements] [where_clause] ":"
                 NEWLINE INDENT {enum_member} DEDENT
enum_member   := {attribute} (variant | fn_decl | const_decl)
variant       := IDENT ["(" variant_field {"," variant_field} [","] ")"] ["=" expression] NEWLINE
variant_field := {attribute} [IDENT ":"] type

interface_decl:= "interface" IDENT [generic_params] [":" bound_list] [where_clause] ":"
                 NEWLINE INDENT {interface_member} DEDENT
interface_member := {attribute} (fn_decl | assoc_type | const_decl)
assoc_type    := "type" IDENT [":" bound_list] ["=" type] NEWLINE

extend_decl   := "extend" [generic_params] type [implements] [where_clause] ":" type_body
const_decl    := "const" IDENT [":" type] "=" expression NEWLINE
static_decl   := "static" ["mut"] IDENT ":" type "=" expression NEWLINE
type_alias    := "type" IDENT [generic_params] "=" type ["in" expression] NEWLINE
extern_block  := ["unsafe"] "extern" STRING ":" NEWLINE INDENT {extern_item} DEDENT
extern_item   := {attribute} (["safe"] fn_header NEWLINE
               | "static" ["mut"] IDENT ":" type NEWLINE
               | "type" IDENT NEWLINE)
comptime_block:= "comptime" ":" block
```

* `[GRM-1]` A class has at most one base class, written in parentheses: `class Door(Script):`.
* `[GRM-33]` *(new in 0.9.9)* A bodiless `fn_decl` is legal only in an `interface`, an `extern_block`, or as a
  `virtual` method of an `abstract class`. Elsewhere it is `E0110` with the help `add ':' and a body`.
* `[GRM-34]` *(new in 0.9.9)* `extend [T: B] Array[T] implements I:` declares a generic extension; the generic
  parameters after `extend` scope over the whole block.
* `[GRM-8d]` A `type_alias` with an `in` clause declares a range type (`[RNG-1]`). The clause is
  legal only on an item-level alias with no generic parameters (`E2213` otherwise).

## III.3 Types

```text
type          := path_type | "ref" ["mut"] type | "*" ["mut"] type | tuple_type | fn_type
               | "dyn" bound_list | "some" bound_list | "[" type ";" expression "]"
               | "Self" | "void"
path_type     := path_segment {"." path_segment}
path_segment  := IDENT [generic_args]
generic_args  := "[" generic_arg {"," generic_arg} [","] "]"
generic_arg   := type | expression | IDENT "=" type          -- const argument; associated-type binding
tuple_type    := "(" ")" | "(" type "," [type {"," type} [","]] ")"
fn_type       := ["once"] ["extern" STRING] "fn" "(" [fn_type_param {"," fn_type_param} [","]] ")"
                 ["->" type]
fn_type_param := ["mut" | "owned"] type
```

* `[GRM-3]` `Array[T]`, `Map[K, V]`, `Option[T]`, `Result[T, E]`, `Span[T]`, `Box[T]`, and the other
  library types are ordinary `path_type`s; the grammar does not special-case them. Generic arguments
  may appear on any path segment: `SoA[Particle].Ref`.
* `[GRM-31]` *(new in 0.9.9)* A `some` type (`[TYP-32]`) is legal only as a function's return type, or nested inside
  one; elsewhere it is `E2260`.

## III.4 Statements

```text
block         := NEWLINE INDENT {statement} DEDENT | simple_stmt NEWLINE
statement     := {attribute} (simple_stmt NEWLINE | compound_stmt)
simple_stmt   := var_decl | assignment | expr_list | "pass"
var_decl      := IDENT ":" type ["=" expr_list]
assignment    := target_list "=" expr_list
               | target augassign expression
target_list   := target {"," target} [","]
target        := IDENT | postfix_expr | "_" | "(" target_list ")"
expr_list     := expression {"," expression} [","]
augassign     := "+=" | "-=" | "*=" | "/=" | "//=" | "%=" | "**=" | "&=" | "|=" | "^="
               | "<<=" | ">>="

compound_stmt := if_stmt | while_stmt | for_stmt | match_stmt | with_stmt | defer_stmt
               | unsafe_stmt | comptime_stmt | labeled_stmt | fn_decl
if_stmt       := "if" condition ":" block {"elif" condition ":" block} ["else" ":" block]
condition     := expression | pattern "=" expression
while_stmt    := "while" condition ":" block ["else" ":" block]
for_stmt      := "for" for_target "in" expression ":" block ["else" ":" block]
for_target    := pattern {"," pattern}
labeled_stmt  := IDENT ":" (while_stmt | for_stmt)
match_stmt    := "match" expression ":" NEWLINE INDENT {match_arm} DEDENT
match_arm     := pattern ["if" expression] ":" block
with_stmt     := "with" with_item {"," with_item} ":" block
with_item     := [IDENT "="] expression
defer_stmt    := "defer" ":" block
unsafe_stmt   := "unsafe" ":" block
comptime_stmt := "comptime" ":" block
```

* `[GRM-4]` *(changed in 0.9.9)* `x = e` where no `x` is in scope declares `x` with the type of `e`;
  where `x` is in scope it assigns. `x: T = e` always declares; a declaration may shadow a name from an
  enclosing block, and redeclaring a name in the same block is `E1020`. A name declared in every arm
  of an exhaustive branch is hoisted to the enclosing block (`[CTL-10]`).
* `[GRM-5]` `a, b = e` destructures a tuple, a struct or a fixed array. The right side is evaluated
  once, into a temporary, before any target is written (`[EXP-2]`).
* `[GRM-29]` *(new in 0.9.9)* An `expr_list` of two or more expressions, or of one expression followed by a comma, is a
  tuple: `a, b = b, a` swaps, and `return x, y` returns a tuple.
* `[GRM-6]` The optional `else` of `while` and `for` runs when the loop ends without `break`.
* `[GRM-7]` `defer` blocks run in reverse order at the exit of the enclosing block, on every path out
  of it, including `return`, `break` and `continue`. A `defer` block may not itself jump out (`E2160`).
* `[GRM-28]` *(new in 0.9.9)* A `fn_decl` inside a block declares a **local function**. It may capture locals of the
  enclosing function exactly as a lambda does (`[CLO-2]`), may call itself recursively, and is in scope
  from its declaration to the end of the block.
* `[GRM-18]` `;` never separates statements. `a = 1; b = 2` is `E0105`, whose help puts each statement
  on its own line. `;` appears only inside `[T; N]` and `[v; N]`.
* `[GRM-19]` *(changed in 0.9.9)* The pattern of a `condition` MUST be refutable. An irrefutable one
  is `E2036`; when it is a bare name already in scope (`if x = 5:`), the primary help is
  `did you mean 'x == 5'?`.
* `[GRM-20]` The attributes before a statement are limited to `@parallel`, `@unroll`, `@simd` and
  `@allow` (`[ATT-2]`).
* `[ATT-3]` A statement attribute attaches to the next compound statement, never to a simple one
  (`E0108`); `@simd`, `@parallel` and `@unroll` require a `for` (`E0108`). Several may precede one
  statement, one per line, in any order.

## III.5 Expressions

Precedence, lowest first. Operators on one row associate left unless noted.

| Level | Operators | Notes |
|---|---|---|
| 1 | `return e`, `break`, `continue`, `yield e` | jump expressions; must be a whole expression statement or a whole arm/branch body (`[GRM-16]`) |
| 2 | `fn(…) => e`, `owned fn …`, `once fn …` | lambda; body extends as far right as possible |
| 3 | `x if c else y` | right-associative |
| 4 | `or` | |
| 5 | `and` | |
| 6 | `not` (prefix) | |
| 7 | `==` `!=` `<` `>` `<=` `>=` · `is` `is not` · `in` `not in` | the six comparisons chain (`[GRM-25]`); `is`, `in` do not |
| 8 | `..` `..=` | either operand may be omitted; non-associative |
| 9 | `\|` | |
| 10 | `^` | |
| 11 | `&` | |
| 12 | `<<` `>>` | |
| 13 | `+` `-` | |
| 14 | `*` `/` `//` `%` | |
| 15 | `as` `as?` `as!` | postfix type conversion |
| 16 | `-` `~` `ref` `ref mut` (prefix) | `-x as u8` is `(-x) as u8`; `ref a.b[i]` borrows `a.b[i]` |
| 17 | `**` | right-associative; binds tighter than a prefix `-` on its left: `-2**2 == -4` |
| 18 | call, index, field, method, `?.`, postfix `?` | |
| 19 | atoms | |

```text
expression    := jump_expr | lambda | ternary
jump_expr     := "return" [expr_list] | "break" [IDENT] | "continue" [IDENT] | "yield" [expression]
lambda        := ["owned" | "once"] "fn" "(" [lambda_param {"," lambda_param} [","]] ")"
                 ["->" type] ("=>" expression | ":" block)
lambda_param  := ["mut" | "owned"] IDENT [":" type]
ternary       := or_expr ["if" or_expr "else" ternary]
or_expr       := and_expr {"or" and_expr}
and_expr      := not_expr {"and" not_expr}
not_expr      := "not" not_expr | comparison
comparison    := range_expr {comp_op range_expr}
comp_op       := "==" | "!=" | "<" | ">" | "<=" | ">=" | "is" ["not"] | ["not"] "in"
range_expr    := [bitor] (".." | "..=") [bitor] | bitor
bitor         := bitxor {"|" bitxor}
bitxor        := bitand {"^" bitand}
bitand        := shift {"&" shift}
shift         := additive {("<<" | ">>") additive}
additive      := multiplicative {("+" | "-") multiplicative}
multiplicative:= cast {("*" | "/" | "//" | "%") cast}
cast          := unary {("as" | "as?" | "as!") type}
unary         := ("-" | "~") unary | "ref" ["mut"] unary | power
power         := postfix_expr ["**" unary]
postfix_expr  := atom {postfix}
postfix       := "(" [arg {"," arg} [","]] ")" | "(" expression comp_clauses ")"
               | "[" expr_list "]" | "." IDENT | "." INT
               | "?." IDENT | "?"
arg           := expression | IDENT "=" expression
atom          := literal | IDENT | "self" | "Self" | "(" expression ")" | tuple_lit | list_lit
               | map_lit | set_lit | comprehension | match_expr | "comptime" "(" expression ")"
tuple_lit     := "(" ")" | "(" expression "," [expression {"," expression} [","]] ")"
list_lit      := "[" [expression {"," expression} [","]] "]" | "[" expression ";" expression "]"
map_lit       := "{" "}" | "{" expression ":" expression {"," expression ":" expression} [","] "}"
set_lit       := "{" expression {"," expression} [","] "}"
comprehension := "[" expression comp_clauses "]"
               | "(" expression comp_clauses ")"
               | "{" expression ":" expression comp_clauses "}"
               | "{" expression comp_clauses "}"
comp_clauses  := "for" for_target "in" or_expr {"for" for_target "in" or_expr | "if" or_expr}
match_expr    := "match" expression ":" NEWLINE INDENT {pattern ["if" expression] "=>" expression NEWLINE}
                 DEDENT
```

* `[GRM-8]` `name[…]` in expression position is resolved during name resolution: if `name` denotes a
  generic function or type it is an instantiation, otherwise an index. `f[int](x)` instantiates then
  calls.
* `[GRM-8a]` Inside `[ ]` in expression position the parser commits to a type argument when the next
  token is `ref`, `*`, `dyn`, `some`, `fn`, `once`, `extern` or `void`; otherwise it parses an
  expression and name resolution reinterprets it as a type where `[GRM-8]` resolved an instantiation.
* `[GRM-8b]` An index whose argument is a type is `E2172`; an instantiation argument that is neither a
  type nor a constant is `E2173`.
* `[GRM-8c]` `IDENT = type` inside `[ ]` is an associated-type binding (`Iterator[Item = int]`).
* `[GRM-10]` A `match` whose arms use `pattern: block` is a statement; one whose arms use
  `pattern => expression` is an expression. Mixing the forms is `E0103`.
* `[GRM-11]` There are no block expressions. A value computed by several statements is written with a
  `match` expression, a conditional expression, a local function, or `comptime(…)`.
* `[GRM-16]` *(changed in 0.9.9)* `return`, `break` and `continue` are expressions of type `Never`;
  `yield e` is an expression of type `void` (`[CORO-3]`). A jump is either a whole expression statement or the whole
  body of a lambda, a match arm, or a conditional-expression branch; `a + return b` is `E0107`.
* `[GRM-15]` `owned e` in expression position is legal only as the iterable of a `for` or the
  scrutinee of a `match`, where it consumes `e`; elsewhere it is `E0109`.
* `[GRM-17]` A lambda with a `:` body inside brackets holds exactly one simple statement (`[LEX-6a]`).
  More is `E0106`, whose help is to declare a local function on the preceding line (`[GRM-28]`).
* `[GRM-21]` `gen fn` declares a generator (`[CORO-1]`). It is legal wherever `fn` is, except in an
  `extern` block.
* `[GRM-23]` `x in c` and `x not in c` are membership tests at comparison precedence. `not in` is one
  operator: `x not in xs` never parses as `(not x) in xs`. Membership does not chain: `a in b in c` is
  `E0102`.
* `[GRM-24]` *(new in 0.9.9)* Ember has one path separator, `.`. A path resolves left to right: package, module, type,
  associated item or variant. `::` is not a token (`[LEX-21]`).
* `[GRM-25]` *(new in 0.9.9)* A chain of the comparison operators `==`, `!=`, `<`, `>`, `<=`, `>=` means what it means in
  Python: `a < b <= c` is `a < b and b <= c`, each operand is evaluated at most once and left to right,
  and evaluation stops at the first false comparison. `is` and `in` may not appear in a chain
  (`E0102`).
* `[GRM-26]` *(new in 0.9.9)* A `{…}` atom is a map literal if its first element is followed by `:`, and a set literal
  otherwise. `{}` is an empty map; an empty set is written `Set[T]()`.
* `[GRM-27]` *(new in 0.9.9)* A comprehension is shorthand for an iterator pipeline and has exactly its meaning and
  cost: `[f(x) for x in xs if p(x)]` is `xs.iter().filter(fn(x) => p(x)).map(fn(x) => f(x)).collect()`
  into an `Array`; `{…}` forms collect into a `Set` or a `Map`. Clauses nest left to right as in
  Python. The loop variables are scoped to the comprehension.
* `[GRM-38]` *(new in 0.9.9)* A parenthesised comprehension `(e for x in it if c)` is a **generator
  expression**: the pipeline of `[GRM-27]` without the `collect`, a lazy `Iterator` that borrows what it
  iterates and allocates nothing. As the only argument of a call it needs no extra parentheses:
  `sum(x * x for x in xs)`, `any(p.hp <= 0 for p in players)`. `(e)` without `for` is a parenthesised
  expression, and `(e,)` a tuple.
* `[GRM-30]` *(new in 0.9.9)* `as?` and `as!` are single tokens (`[LEX-21]`): `h as? D` is a checked downcast
  (`[TYP-6]`).
* `[GRM-32]` *(new in 0.9.9)* `comptime(e)` evaluates the expression `e` at compile time (`[CT-6]`).
* `[GRM-36]` *(new in 0.9.9)* `ref e` and `ref mut e` borrow the place `e` (`[BRW-1]`). The operand MUST
  be a place — a local, a field, an index or a dereferenced reference — and `ref` of any other
  expression is `E0111`, whose help is to bind the value to a local first. `ref` binds like unary
  minus: `ref a.b[i]` borrows `a.b[i]`.

## III.6 Patterns

```text
pattern       := bind_pattern {"|" bind_pattern}
bind_pattern  := IDENT "@" primary_pattern | primary_pattern
primary_pattern := "_" | literal_pattern | range_pattern | IDENT
               | path "(" [field_pattern {"," field_pattern}] ["," ".."] ")"
               | path
               | "(" [pattern {"," pattern} [","]] ")"
               | "[" [pattern {"," pattern}] ["," ".." [IDENT]] "]"
               | "ref" ["mut"] IDENT
literal_pattern := ["-"] INT | STRING | CHAR | "true" | "false"
range_pattern := literal_pattern (".." | "..=") literal_pattern
field_pattern := pattern | IDENT "=" pattern
path          := IDENT {"." IDENT}
```

* `[GRM-12]` An identifier in pattern position names a unit variant or a `const` if one of that name is
  in scope; otherwise it binds a new name. A binding that shadows a variant of another enum in scope is
  `W1002`.
* `[GRM-13]` Matching a place that is not consumed binds `Copy` fields by value and other fields by
  reference; `match owned e:` binds by move.
* `[ENM-1]` *(changed in 0.9.9)* Inside a pattern whose scrutinee type is known, a variant may be
  written unqualified (`Circle(r)`); in an expression it is qualified (`Shape.Circle(1.0)`) unless
  imported.
---

# Part IV — Type System

## IV.1 Type categories

| Category | Declared by | Stored | `b = a` | Destroyed |
|---|---|---|---|---|
| **Scalar** | built in | inline | copy | trivially |
| **Value** | `struct`, `enum`, tuple, `[T; N]`, closure | inline | copy if `Copy`, else move | end of scope, reverse order |
| **Collection** | `Array`, `String`, `Map`, `Set`, … (library) | inline header + heap buffer | move | end of scope; frees its buffer |
| **Handle** | `class` | pointer to a counted heap object | copy (retain) | when the strong count reaches 0 |
| **View** | `ref T`, `Span[T]`, `MutSpan[T]`, `str`, any struct holding one | inline pointer(s) + compile-time region | copy (`MutSpan`, `ref mut`: move) | never (no destructor) |
| **Raw** | `*T`, `*mut T`, `extern fn` | inline pointer | copy | never |
| **Existential** | `dyn I`, `Box[dyn I]`, a class handle typed by an interface | pointer + table | per underlying | per underlying |
| **Unit / Never** | `void`, `Never` | zero-size | trivial | trivial |

* `[TYP-1]` Every concrete type has, at compile time, a size, an alignment, and the properties
  `Copy`, `Send`, `Sync`, needs-drop, view and niche. `size_of[T]()` and `align_of[T]()` report them.

## IV.2 Scalar types

| Type | Size | Notes |
|---|---|---|
| `i8 i16 i32 i64 i128` | 1 2 4 8 16 | two's complement |
| `u8 u16 u32 u64 u128` | 1 2 4 8 16 | |
| `isize usize` | pointer | for FFI and raw memory; not the size type (`[TYP-31]`) |
| `int` | 8 | prelude alias of `i64`; the default integer and the size and index type |
| `f16 f32 f64` | 2 4 8 | IEEE 754 binary16/32/64 |
| `float` | 8 | prelude alias of `f64`; the default float |
| `bool` | 1 | `true` or `false`; any other bit pattern is invalid (`[TYP-2]`) |
| `char` | 4 | a Unicode scalar value; surrogates are invalid (`[TYP-3]`) |
| `void` | 0 | the unit type; its one value is written `()` |
| `Never` | 0 | the type of an expression that does not complete: `return`, `panic(…)`, an infinite loop; coerces to every type |

* `[TYP-2]` Producing a `bool` other than 0 or 1 is undefined behaviour and possible only in `unsafe`.
* `[TYP-3]` Producing a `char` outside the Unicode scalar values is undefined behaviour and possible
  only in `unsafe`.
* `[TYP-27]` *(new in 0.9.9)* `()` is the value of type `void`; `Ok(())` is the success value of `Result[void, E]`. A
  function with no `->` returns `void`.

### Conversions

* `[TYP-4]` *(changed in 0.9.9)* No value converts implicitly between scalar types in an operator.
  `i32 + i64` is `E2020`, whose note appears only when both operands are numeric and names the exact
  `as` cast on the narrower operand. Untyped literals are not values yet and take their type from
  context (`[LEX-16]`, `[LEX-17]`).
* `[TYP-5]` *(changed in 0.9.9)* **Coercions — the complete list.** At a coercion site (assignment or
  initialisation, argument, return, field initialiser, collection element, default value), and only
  there, a value converts implicitly by exactly these rules:
  1. lossless integer widening: `iN → iM`, `uN → uM`, `uN → iM` for M > N;
  2. float widening: `f16 → f32 → f64`;
  3. range erasure: a range type to its representation (`[RNG-2]`);
  4. string literal to `String` (`[TXT-9]`), and `String` to `str` (a borrow, `[SPN-1]`);
  5. collection literal to the collection type the site expects (`[TYP-38]`);
  6. `Array[T]`, `[T; N]` to `Span[T]` (shared borrow) or `MutSpan[T]` at a `mut` site (`[SPN-1]`);
  7. `ref T` from a place (auto-borrow), and reading through a `ref` where a `T` is expected (`[TYP-14]`);
  8. a class handle to a base class or to an interface it implements (upcast);
  9. `Never` to any type;
  10. a function or lambda to a callable type it satisfies (`[CLO-3]`);
  11. a value of type `T` to `Option[T]`, as `Some(value)`, where `T` is not itself an `Option`: one
      level only, so nothing becomes `Option[Option[T]]` (`return v` in a function returning
      `Option[T]`, `f(5)` for a parameter `x: Option[int] = None`).
  Each rule composes with the others only where stated (widening after range erasure, and rule 11
  after rules 1–4). Nothing
  converts implicitly to or from `bool` or `char`.
* `[TYP-6]` *(changed in 0.9.9)* `x as T` converts explicitly:
  * integer → integer: keeps the low bits (truncation or sign/zero extension);
  * float → integer: rounds toward zero and **saturates** at the target's bounds; NaN becomes 0;
  * integer → float: rounds to nearest, ties to even;
  * float → float: rounds to nearest;
  * `char → u32` and `u8 → char`; other integers become a `char` only through `char.from_u32(x) ->
    Option[char]`;
  * `h as? D` downcasts a class handle: `Option[D]`; `h as! D` downcasts or panics.
  A checked conversion that fails instead of truncating is `T.try_from(x) -> Result[T, RangeError]`.
* `[TYP-7]` `as` between pointer types, or between a pointer and an integer, requires `unsafe`.

### Integer arithmetic

* `[TYP-8]` *(changed in 0.9.9)* **Integer overflow panics in every profile.** An arithmetic operation
  (`+ - * // % **`, unary `-`, and the compound assignments) whose mathematical result does not fit its
  type panics with `integer overflow in '<op>'` naming the operator actually written. There is no
  profile in which overflow wraps silently (`[PHIL-13]`). Three explicit ways to get other behaviour:
  the methods `wrapping_<op>`, `checked_<op> -> Option[T]`, `saturating_<op>`, `overflowing_<op> ->
  (T, bool)`; the attribute `@overflow(wrap)` or `@overflow(saturate)` on a function or module, which
  changes the operators inside it for every profile; and range facts (`[RNG-4]`), which remove a check
  the compiler proves cannot fire. Wrapping arithmetic is two's-complement and is never undefined
  behaviour in the generated code (`[CG-C-1]`).
* `[TYP-28]` *(new in 0.9.9)* **Division.** `/` is **true division** and applies to floats. `/` with two integer
  operands is `E2240`, whose fix-its are `//` (floor division) and converting an operand to `float`.
  For integers:
  * `a // b` is floor division: the quotient rounded toward negative infinity (Python). `-7 // 2 ==
    -4`.
  * `a % b` is floor modulo: `a - (a // b) * b`, which has the sign of `b` (Python). `-7 % 2 == 1`,
    `x % -1 == 0`.
  * `a.div_trunc(b)` and `a.rem_trunc(b)` give C's truncating quotient and remainder, for code that
    wants C's exact meaning.
  * a zero divisor panics (`division by zero`); `MIN // -1` and `MIN.div_trunc(-1)` overflow and panic
    per `[TYP-8]`.
* `[TYP-29]` *(new in 0.9.9)* For floats, `//` and `%` are Python's (ODR-021). `a % b` is the
  exact value of `a - b * floor(a / b)`, rounded once to the float type: it has the sign of `b`, and
  is computed exactly (as `fmod` is) rather than by evaluating the formula in rounded steps. `a // b`
  is the floor quotient consistent with it, so `a == (a // b) * b + a % b` to within one rounding:
  `1.0 // 0.1 == 9.0` and `1.0 % 0.1 == 0.09999999999999995`, because `0.1` is slightly more than a
  tenth. Zero divisors, infinities and NaN follow IEEE.
* `[TYP-10]` *(changed in 0.9.9)* **Shifts.** `a << n` and `a >> n` accept any integer type for `n`.
  If `n` is negative or not less than the bit width of `a`'s type the shift panics, in every profile.
  Bits shifted out are discarded and are not an overflow. `>>` on a signed type is arithmetic
  (sign-extending); on an unsigned type logical. The methods `wrapping_shl`/`wrapping_shr` mask the
  amount instead.
* `[TYP-30]` *(new in 0.9.9)* **Powers.** `a ** b` with integer `a` and non-negative integer `b` is exact, and
  overflows per `[TYP-8]`; a negative exponent is `E2151` when it is a constant and a panic when it is
  not. A float base with a float or integer exponent is `pow`. Other combinations are `E2020`.
* `[TYP-31]` *(new in 0.9.9)* **Sizes and indices are `int`.** Every standard container's `len()` returns `int`.
  Indexing accepts an index of any integer type; an index that is negative or not less than the length
  panics (`index out of bounds`). There is no Python-style negative indexing: a negative literal index
  is `E2011` (`[LEX-24]`). `usize` and `isize` remain for foreign interfaces and raw memory.

### Floating point

* `[TYP-9]` Floating point is strict IEEE 754: no reassociation, no contraction of `a * b + c` into a
  fused operation, no assumption that values are finite, unless a function is `@fastmath` (relaxes all
  of these within it) or `@fp(contract)` (permits contraction only). `NaN == NaN` is false. The
  operators `<`, `<=`, `>`, `>=`, `==`, `!=` on floats are IEEE comparisons; `Ord.cmp` on floats is the
  IEEE totalOrder (`[TYP-37]`).
* `[TYP-9a]` Contraction is off by default and the implementation MUST turn it off in the host C
  compiler (`[CG-C-11]`); a toolchain on which it cannot be turned off is rejected with `E9011`.
* `[TYP-9b]` `@fp(contract)` permits, and requires the backend to enable, fused multiply-add within
  that function only.
* `[TYP-9c]` *(changed in 0.9.9)* A toolchain that cannot honour `@fastmath` or `@fp(…)` for one function is `E9041`,
  naming the toolchain and the attribute. The attribute is never ignored (`[PHIL-12]`).

## IV.2a Range types

A range type is a nominal numeric type that carries its own bounds:

```ember
type Roughness = f32 in 0.0 ..= 1.0
type Metallic  = f32 in 0.0 ..= 1.0
type Percent   = u8  in 0 ..= 100

fn from_slider(x: f32) -> Result[Roughness, RangeError]:
    return Roughness.checked(x)

fn demo():
    r: Roughness = 0.5            # a constant in range: no check emitted
    p = Percent.clamped(250)      # total: 100
    println(f"{r} {p}")
```

* `[RNG-1]` A `type` alias with an `in` clause declares a nominal numeric type over the named
  representation, restricted to the range. The endpoints are constant expressions of the
  representation. A `type` alias without one is transparent.
* `[RNG-2]` Two range types are distinct even with equal representation and range (`E2210`). A range
  value converts to its representation at a coercion site (`[TYP-5]`); the reverse is a construction.
* `[RNG-3]` Construction from a value not known to be in range is `T.checked(v) -> Result[T,
  RangeError]`. `RangeError` is a prelude type. Construction from a constant in range, or from a value
  whose known range lies inside the target's, emits no check; a constant outside the range is `E2211`.
* `[RNG-3a]` Every range type with finite endpoints has `T.clamped(v) -> T`, total, with no panic path;
  NaN clamps to the lower endpoint.
* `[RNG-4]` *(changed in 0.9.9)* The compiler tracks a known range for numeric expressions — literals,
  `min`, `max`, `clamp`, the arms of a comparison, and arithmetic on known ranges — and uses it to remove
  `[RNG-3]`'s check, `[TYP-8]`'s overflow check and bounds checks (`[OPT-2]`). The variable of a range
  loop `for i in a..b` carries the fact `a <= i < b` in the body (`a <= i <= b` for `a..=b`), and its
  increment has no overflow check. A fact is derived from an operation's mathematical result only when
  that result provably fits; never inside `@fastmath`.
* `[RNG-4a]` For floats, a range fact comes only from the true arm of a comparison and records whether
  NaN is excluded; a fact that does not exclude NaN never removes a construction check.
* `[RNG-5]` Arithmetic on a range value yields its representation: `r * 2.0` is `f32`. Arithmetic
  between two different range types is `E2214`. `min`, `max` and `clamp` over one range type and
  in-range constants keep the range type.
* `[RNG-5a1]` *(changed in 0.9.9)* The operators on range types come from compiler-generated
  implementations of `Add`, `Sub`, `Mul`, `Div`, `FloorDiv`, `Rem`, `Neg`, `Eq` and `Ord`, placed in
  the type's declaring module; exact-type implementations are chosen before coercion (`[RNG-5a2]`).
  No compound-assignment operator is generated: `r += 0.1` is `E2214` with the fix-it
  `r = T.clamped(r + 0.1)`.
* `[RNG-5a2]` Overload resolution picks an implementation matching the operand types exactly before
  considering any coercion.
* `[RNG-6]` NaN is in no range. A range from `-0.0` to `+0.0` contains both zeros.
* `[RNG-7]` A range type whose range does not cover its whole representation supplies a niche:
  `Option[Percent]` is one byte. One whose range covers every value supplies none.
* `[RNG-8]` A range type is `Copy` when its representation is, has its representation's layout, and
  crosses a foreign boundary as its representation: a value arriving from C becomes a range value only
  through `checked` or `clamped` (`[RNG-3]`, `[RNG-10b]`).
* `[RNG-9]` A range value outside its range is invalid; producing one is undefined behaviour and needs
  `unsafe` (`T.new_unchecked(v)`, which asserts the range in `debug`).
* `[RNG-10]` In Safe code a range value arises only from a constant in range, `checked`, `clamped`, a
  value whose known range fits, or a copy of a valid value. Anything else is `E2215`.
* `[RNG-10a]` A derived `Deserialize` checks every range-typed field with `checked` and maps failure to
  `SerError`.
* `[RNG-10b]` A range type may not appear in a foreign signature, directly or inside an aggregate or
  behind a pointer (`E5054`); the foreign side uses the representation.

Diagnostics: `E2210` wrong range type; `E2211` constant out of range; `E2212` bad or inverted
endpoints; `E2213` `in` clause on a non-numeric or generic alias; `E2214` arithmetic mixing range types;
`E2215` construction outside `[RNG-10]`.

## IV.3 Compound value types

* **Tuples** `(A, B, C)`: fields `.0`, `.1`, …; `Copy` iff every element is.
* **Fixed arrays** `[T; N]`: inline, `N` a constant; indexing is bounds-checked; `Copy` iff `T` is.
* **Structs** (Part V.3) and **enums** (Part V.4).
* `[TYP-11]` A struct's default layout is C's: declaration order, natural alignment, trailing padding.
  `@layout(c)` states it explicitly.
* `[TYP-12]` A unit-only enum is an integer (`@repr(u8)` and friends choose it; the default is the
  smallest signed type that fits). An enum crossing a foreign boundary MUST carry `@repr`. A payload
  enum is a tag plus a union of variants.
* `[TYP-13]` *(changed in 0.9.9)* `Option[T]` has the size of `T` when `T` has a niche: a class
  handle, `Box`, `ref`, `Span`, `str`, a non-nullable `extern fn`, `bool`, `char`, a range type per
  `[RNG-7]`, or an enum with unused discriminants. For handles, `Box`, `ref` and `extern fn` this is
  guaranteed, so `Option[Handle]` is a nullable pointer across an FFI boundary.

## IV.4 References and views

* `ref T` and `ref mut T` are first-class references: non-null, aligned, pointing to a live `T` for
  their region. `ref T` is `Copy`; `ref mut T` is move-only and reborrowable.
* `[TYP-14]` *(changed in 0.9.9)* A reference, and a `Box[T]`, is read through wherever a `T` is wanted —
  a field access, a method call, an operand, an argument to a borrowed parameter. So a recursive enum
  whose payload is `Box[Tree]` is traversed by passing the payload to a function taking `Tree`. `r = e`
  on a `ref mut` local writes through it; a reference local is never re-seated (`[BRW-1]`).
* `Span[T]` (read) and `MutSpan[T]` (read-write) are a pointer and a length; `Span` is `Copy`,
  `MutSpan` is move-only and reborrowable. Their API is in §VII.7.
* `str` is a `Span[u8]` known to be valid UTF-8.
* `[TYP-34]` *(new in 0.9.9)* *(replaces the 0.9.8 `@view` requirement)* A struct, enum or tuple that contains a
  reference, a `Span`, a `MutSpan`, a `str` or another view is a **view type**. The compiler infers
  this; `@view` on the declaration is optional documentation, and `@view` on a type that is not a view
  is `E2030`.
* `[TYP-15]` *(changed in 0.9.9)* **Where views may be stored.** A view value may be stored only in a
  place whose lifetime is bounded by every region the view carries. Locals, parameters, return values,
  and fields of other view types are bounded. Class fields, statics, `Box` and `Shared` contents,
  heap-collection elements and `owned fn` captures are not bounded, so they may hold only views whose
  every region is `static` — which admits string literals, `bytes` literals and views of `static`
  items: `names = ["ann", "bob"]` is an `Array[str]` of static strings. Any other stored view is
  `E3063 stored view may not outlive its source`, shape B12.
* `[TYP-15a]` The arena-backed containers (`ArenaArray`, `ArenaMap`, §IX.2) and the view containers
  `BorrowList[T]` may hold views bounded by the container's own region.

## IV.5 Raw types

`*T` and `*mut T` are nullable, possibly unaligned, untracked pointers. Creating one from a reference
is safe; dereferencing, offsetting, reading and writing need `unsafe`. `null[T]()` is the null
pointer. `extern "C" fn(i32) -> i32` is a C function pointer: `Copy`, no captures.

## IV.6 Class handles

A `class C` declaration introduces the type `C` whose values are **handles**: non-null pointers to a
counted heap object. `Option[C]` is the nullable form; `Weak[C]` the weak form. Handles are `Copy`
(copying retains). Upcasting to a base class or to `dyn I` is implicit; downcasting is `as?`/`as!`
(`[TYP-6]`). Part VIII gives the semantics.

## IV.7 Generics

* `[TYP-16]` Generic functions and types are monomorphised: each distinct instantiation is a distinct
  function or type in the output. `ember build --report=instantiations` counts them (`[MONO-2]`).
* `[TYP-17]` Type parameters are bounded by interfaces: `fn sum[T: Add[Output = T] + Default](xs:
  Span[T]) -> T`. Inside a generic body only the operations the bounds provide are available, plus
  copy for `T: Copy`, move, drop and `size_of`. A missing bound is `E2040` naming the bound to add.
* `[TYP-40]` *(new in 0.9.9)* **Interfaces are nominal.** A type implements an interface only through an
  `implements` clause or an `extend … implements` block; having methods of the right names and types is
  not enough. This is what gives every implementation one place of declaration (`[TYP-20]`) and one
  method table for `dyn`.
* `[TYP-18]` Generic arguments are inferred from the arguments of a call. Explicit arguments may be
  written: `f[int](x)`, `Array[f32]()`. A constructor's explicit type arguments always fix the
  instantiation, whatever else the context says.
* `[TYP-19]` There is no specialisation, no higher-kinded type and no variadic generic. Two
  implementations whose applicable types overlap are `E2041`.
* `[TYP-35]` *(new in 0.9.9)* A type parameter need not appear in any field (a **phantom** parameter): `struct
  Handle[Tag]: index: u32` is legal. A phantom parameter affects type identity and counts for `Send` and
  `Sync` as a field of that type would, so a marker type can make a type thread-confined without a
  field; it does not make the type a view.
* Const generics: `fn zeros[const N: int]() -> [f32; N]`. Associated types: `interface Iterator: type
  Item`. Default type parameters: `interface Add[Rhs = Self]`.

## IV.8 Interfaces

An `interface` declares methods, associated types and constants, and may give default method bodies.
A type implements an interface in its header (`struct P implements Display:`) or in an `extend P
implements Display:` block.

* `[TYP-20]` *(changed in 0.9.9)* **Coherence is per package.** An implementation of interface `I`
  for type `T` may appear in any module of the package that declares `I` or the package that declares
  `T`. Two implementations of one interface for one type anywhere in a program are `E2041`, naming
  both. (Separate compilation relies on this: an implementation cannot appear from a third package.)
* `[TYP-24]` *(changed in 0.9.9)* An interface method is found through any implementation visible in
  the program; the interface need not be imported. When two interfaces supply a method of one name for
  one type, the call is `E2070` and is disambiguated as `I.m(recv, …)`.

**Marker interfaces**, derived automatically (implementable by hand only with `unsafe extend`):

| Marker | Holds when |
|---|---|
| `Copy` | declared with `@derive(Copy)` on a struct or enum whose fields are all `Copy` and which has no `drop`; always for scalars, `ref T`, `Span`, `str`, raw pointers, `extern fn`, handles, tuples and fixed arrays of `Copy` elements |
| `Send` | every field is `Send`; raw pointers and `ref` are not; a class handle is `Send` iff the class is `Sync` |
| `Sync` | every field is `Sync`; `ref mut`, `Cell`, `RefCell` and `UnsafeCell` are not; see `[THR-1]` for classes |
| `Sized` | everything except `dyn I` and unsized foreign types |

**Standard interfaces.** These are the definitions; `std.core` declares them and the prelude exports
them (`[MOD-5]`).

```ember,ignore
## signature sketch: the standard interfaces as std.core declares them
interface Clone:
    fn clone(self) -> Self

interface Drop:
    fn drop(mut self)

interface Default:
    fn default() -> Self

interface Eq:
    fn eq(self, other: Self) -> bool                 # ==, !=

interface Ord: Eq:
    fn cmp(self, other: Self) -> Ordering            # <, <=, >, >= in generic code; sort, min, max

enum Ordering:
    Less
    Equal
    Greater

interface Hash:
    fn hash[H: Hasher](self, mut h: H)

interface Display:
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]        # f"{x}", print(x)

interface Debug:
    fn fmt_debug(self, mut f: Formatter) -> Result[void, FmtError]  # f"{x!r}", f"{x:?}"

interface Iterator:
    type Item
    fn next(mut self) -> Option[Item]

interface Iterable:
    type Item
    type Iter: Iterator[Item = Item]
    fn iter(self) -> Iter                                           # for x in c

interface IntoIterator:
    type Item
    type Iter: Iterator[Item = Item]
    fn into_iter(owned self) -> Iter                                # for x in owned c

interface Contains[T]:
    fn contains(self, item: T) -> bool                              # x in c

interface Add[Rhs = Self]:                                          # likewise Sub, Mul, Div,
    type Output                                                     # FloorDiv, Rem, Pow, BitAnd,
    fn add(self, rhs: Rhs) -> Output                                # BitOr, BitXor, Shl, Shr

interface AddAssign[Rhs = Self]:                                    # likewise for each operator
    fn add_assign(mut self, rhs: Rhs)

interface Neg:                                                      # likewise Not (~)
    type Output
    fn neg(self) -> Output

interface Index[Idx]:
    type Output
    fn index(self, i: Idx) -> ref Output                            # a[i] read

interface IndexMut[Idx]: Index[Idx]:
    fn index_mut(mut self, i: Idx) -> ref mut Output                # a[i] write

interface IndexSet[Idx, V]:
    fn index_set(mut self, i: Idx, owned v: V)                      # a[i] = v (Map inserts)

interface Error: Debug + Display:
    fn source(self) -> Option[ref dyn Error]                        # default: None

interface From[T]:
    fn from(owned value: T) -> Self                                 # `?` conversion; Into is derived
```

* `[TYP-36]` *(new in 0.9.9)* **Which types implement which interfaces.** The table is normative; a `—` means the
  implementation does not exist and a bound requiring it is `E2040`.

| Type | Clone | Copy | Eq | Ord | Hash | Default | Display | Debug |
|---|---|---|---|---|---|---|---|---|
| integers, `int` | ✓ | ✓ | ✓ | ✓ | ✓ | 0 | ✓ | ✓ |
| floats, `float` | ✓ | ✓ | IEEE `==` | totalOrder | — | 0.0 | ✓ | ✓ |
| `bool` | ✓ | ✓ | ✓ | `false < true` | ✓ | `false` | ✓ | ✓ |
| `char` | ✓ | ✓ | ✓ | by scalar value | ✓ | `'\0'` | ✓ | ✓ |
| `void` | ✓ | ✓ | ✓ | ✓ | ✓ | `()` | — | ✓ |
| `str` | ✓ | ✓ | ✓ | by bytes | ✓ | `""` | ✓ | ✓ |
| `String` | ✓ | — | ✓ | by bytes | ✓ | empty | ✓ | ✓ |
| tuples, `[T; N]` | if all | if all | if all | lexicographic, if all | if all | if all | if all `Debug` | if all |
| `Option[T]`, `Result[T, E]` | if `T`(,`E`) | if `T`(,`E`) | if `T`(,`E`) | `None < Some` | if `T`(,`E`) | `None` / — | if `Debug` | if `T`(,`E`) |
| `Array[T]` | if `T` | — | if `T` | lexicographic | if `T` | empty | if `T: Debug` | if `T` |
| `Map[K, V]`, `Set[T]` | if all | — | as sets of entries | — | — | empty | if all `Debug` | if all |
| class handles | handle copy | ✓ | identity (`is`) | — | identity | — | — | ✓ (class and address) |
| structs, enums | derived per `[STR-5]` | `@derive(Copy)` | derived per `[STR-5]` | `@derive(Ord)` | `@derive(Hash)` | `@derive(Default)` | explicit | derived per `[STR-5]` |

* `[TYP-39]` *(new in 0.9.9)* Collections, tuples and `Option`/`Result` implement `Display` the way Python's `str()`
  shows them: `[1, 2, 3]`, `{'a': 1}`, `{1, 2}`, `(1, 'x')`, `Some(3)`, `None`, with each element
  formatted by its `Debug` implementation (strings quoted). So `println(xs)` prints a list as Python
  would. A `Map` prints its entries in insertion order and an empty one prints `{}`; an empty `Set`
  prints `set()`. A collection's `Debug` text is its `Display` text. A string's `Debug` text is
  Python's `repr`: single quotes, or double quotes when the text holds a `'` and no `"`; `\`, the
  chosen quote, `\n`, `\r` and `\t` are escaped, and any other control character is `\xNN`.
* `[TYP-37]` *(new in 0.9.9)* Floats implement `Eq` with IEEE `==` (so `NaN != NaN`) and `Ord` with the IEEE-754
  totalOrder predicate (`-NaN < -inf < … < -0.0 < +0.0 < … < +inf < +NaN`). The comparison
  *operators* on float values keep IEEE meaning (`[TYP-9]`); generic code bounded by `Ord`, `sort()`,
  `min()` and `max()` use `cmp`, so sorting floats is total and needs no extra step. Floats do not
  implement `Hash`; a float map key is written with `f.to_bits()`.
* `[TYP-21]` Operators desugar to these interfaces for non-scalar operands; `a + b` calls
  `Add.add(a, b)` with both operands borrowed. `a += b` calls `AddAssign.add_assign` if implemented,
  else `a = a + b`. Scalar operators are built in.
* `[HASH-1]` `Hash.hash` is generic over `H: Hasher` and monomorphised; hashing a value MUST feed a
  deterministic representation of it into the hasher, and values equal under `Eq` MUST hash equally.
  `Hasher` (in `std.collections`, not the prelude) is a move-only hashing state with `write_bytes`,
  `write_u8`, `write_u16`, `write_u32`, `write_u64`, `write_i8`, `write_i16`, `write_i32`,
  `write_i64`, `write_usize`, `write_isize` and `finish(owned self) -> u64`. A user type may
  implement it and serve as a map's hasher.
* `[HASH-2]` *(changed in 0.9.9)* `std.collections.DefaultHasher` is a fixed-seed hasher: the same
  keys hash the same way in every run and on every machine. It is a fast non-cryptographic hash, no
  slower on integer keys than one multiply and one rotate per word (FxHash's class). `std.collections.
  RandomState` is a per-process randomly seeded hasher for maps keyed by untrusted input; a `Map[K, V,
  RandomState]` still iterates in insertion order (`[STD-11]`), so the seed never becomes visible.
  `DefaultHasher`'s values are the same for one Ember version on every target (its state is 64 bits
  on a 32-bit target too), so a table built at compile time is valid at run time; they may change
  between versions, and a program MUST NOT depend on them.
* `[HASH-3]` *(changed in 0.9.9)* `Map` and `Set` MUST NOT weaken equality to compensate for an
  incoherent `Hash`. An incoherent `Hash`, a key changed through a `Cell`, or an inconsistent `Ord`
  given to `sort` gives wrong answers (a missing key, any permutation of the elements) but never
  undefined behaviour, an out-of-bounds access or an operation that does not terminate. A `hash` or
  `eq` that panics aborts the process (`[PAN-1]`), so no map is seen half-changed; one that reaches
  the map again can do so only through a `RefCell`, whose borrow panics.
* `[HASH-4]` `Map[K, V]` requires `K: Eq + Hash`. A map may call `hash` any number of times per
  operation. Safe map APIs never expose `ref mut K`.

## IV.9 Existential and opaque types

* `dyn I` is unsized and used behind `ref dyn I`, `Box[dyn I]` or a class handle. It is a data pointer
  and a table pointer (a class handle reaches its table through the object header, `[OBJ-2]`).
* `[TYP-22]` An interface is usable as `dyn` only if every method has a receiver, is not generic, and
  does not return `Self` by value (methods marked `where Self: Sized` excepted). Violations are
  `E2050`, naming the method and the clause.
* `[TYP-32]` *(new in 0.9.9)* `some I` in a return type means "one concrete type, chosen by the function body, that
  implements `I`". The caller sees only `I`; the compiler knows the type and monomorphises through it,
  so there is no indirection. Every `return` in the body must produce the same concrete type
  (`E2261` otherwise). It is how a function returns an iterator pipeline whose type cannot be written:

```ember
fn evens(xs: Span[int]) -> some Iterator[Item = int]:
    return xs.iter().copied().filter(fn(x) => x % 2 == 0)
```

## IV.10 Type inference

* `[TYP-23]` *(changed in 0.9.9)* Inference is local to a function body and bidirectional. Function
  signatures, fields, statics and constants are annotated (an omitted return type is `void`). A local
  declared by `x = e` takes `e`'s type; a type left open by `e` (`[]`, `Map()`, `None`) is fixed by
  later uses in the same function, and one still open at the end is `E2060`, highlighting the first
  use. In particular `x = None` followed by `x = v` with `v: T` gives `x` the type `Option[T]`, and the
  later assignment converts by `[TYP-5]` rule 11. An untyped literal is never left open: its context
  is the type expected at the literal itself, and an unannotated declaration supplies none, so
  `total = 0` declares an `int` whatever the later uses (ODR-022); a later use that needs another
  type is `E2020`, whose help names the declaration and the annotation (`total: i32 = 0`). Expected
  types flow into literals, lambdas, `None`, `Ok(…)`, `Err(…)` and generic calls; a branch of a
  conditional expression that cannot type itself (`None`, `[]`) takes the other branch's type. A
  lambda with unannotated parameters and no expected callable type is `E2061`. An unannotated lambda
  parameter also takes `owned` from the expected callable type (ODR-025); `mut` is never inferred, so
  a lambda that writes its parameter says `mut` (`E2228`, shape B15). Within one expression, untyped
  literals are resolved last (`1 + x` takes `x`'s type).

## IV.11 Method resolution and calls

For `recv.m(args)`:

1. Strip references and handle indirection from `recv`'s type `R`, and one level of `Box`/`Shared`.
2. Look for an inherent method `m` on `R`, then on `R`'s base classes nearest first, then in any
   interface `R` implements (`[TYP-24]`), then a field of `R` whose type is callable (`[CLO-11]`). An
   inherent method beats an interface method of the same name.
3. Adjust the receiver to the method's declared mode: `self` borrows, `mut self` borrows mutably,
   `owned self` moves (for a class handle, each of these is a copy of the handle, `[CLS-7]`).

* `[TYP-25]` Arguments may be positional or named; positional arguments come first; a parameter with a
  default may be omitted.
* `[TYP-26]` *(changed in 0.9.9)* There is no overloading: two functions of one name in one scope are
  `E1030`, except operator-interface implementations and `extend` blocks for different types. The
  compiler-known output functions `print`, `println`, `eprint` and `eprintln` (`[STD-9]`) are the only calls with a
  variable number of arguments.
---

# Part V — Declarations

## V.1 Modules, packages, imports, visibility

* `[MOD-1]` A **package** is a directory tree with an `ember.toml` at its root. A **module** is one
  `.em` file. A module's path is the package name followed by its path under `src/`, with `/` replaced
  by `.` and the extension removed; `src/lib.em` or `src/main.em` is the package's root module, and
  `src/a/mod.em` is module `a` when `a/` has submodules. A single `.em` file with no manifest is a
  package of its own (`[CLI-4]`), and the directory containing it is its source root.
* `[MOD-2]` *(changed in 0.9.9)* Items are private to their module unless marked. `pub(package)` makes an
  item visible within the package; `pub` makes it visible to dependants. Using an item, field or
  constructor that is not visible is `E1052`, naming where it is declared and its visibility; `E1020`
  means only a name declared twice in one block (`[GRM-4]`).
* `[MOD-3]` *(changed in 0.9.9)* `import a.b.c` binds the name `c` to module `a.b.c`, and
  `import a.b.c as d` binds `d`. `from a.b import x, y as z` binds items. A standard module may be
  named without `std.`, as in Python: `import math` is `import std.math`, and `from random import Rng`
  is `from std.random import Rng`; when the package itself or one of its dependencies has that name,
  the package wins, with warning `W1003` naming both. A module path that names no module is
  `E1060 no module named 'a.b.c'` at the import, whose help lists the nearest existing module paths;
  an item missing from an existing module is `E1061`. Neither is ever deferred to the first use
  (`[PHIL-12]`).
* `[MOD-8]` *(new in 0.9.9)* `from m import *` binds every `pub` item of `m`. A name bound by two glob imports and then
  used is `E1031`, naming both sources; an explicit import or a local declaration shadows a glob.
* `[MOD-4]` Import cycles within a package are allowed; cycles between packages are `E1041`.
* `[MOD-7]` A field declared `pub(read)` (or `pub(package, read)`) may be read wherever a `pub`
  (`pub(package)`) field could be, and written only from its declaring module (`E1050` otherwise).
  Construction is a write, so a `pub(read)` field makes the memberwise constructor private to the
  module.
* `[MOD-5]` *(changed in 0.9.9)* **The prelude.** Every module implicitly imports these names from
  `std`, and this list is the only definition of the prelude:

  | Group | Names |
  |---|---|
  | Scalar aliases | `int` (`i64`), `float` (`f64`), `Never` |
  | Options and errors | `Option`, `Some`, `None`, `Result`, `Ok`, `Err`, `Error` (interface), `AnyError`, `RangeError` |
  | Text | `String`, `str`, `format` |
  | Collections | `Array`, `Map`, `Set`, `Span`, `MutSpan` |
  | Ranges | `Range`, `RangeInclusive`, `RangeFrom`, `RangeTo` |
  | Heap and cells | `Box`, `Shared`, `Weak`, `Cell`, `RefCell` |
  | Interfaces | `Copy`, `Clone`, `Drop`, `Default`, `Eq`, `Ord`, `Ordering`, `Hash`, `Debug`, `Display`, `Iterator`, `Iterable`, `IntoIterator`, `Generator`, `Contains`, `From`, `Send`, `Sync`, and the operator interfaces `Add`, `Sub`, `Mul`, `Div`, `FloorDiv`, `Rem`, `Pow`, `Neg`, `Not`, `BitAnd`, `BitOr`, `BitXor`, `Shl`, `Shr`, `Index`, `IndexMut`, `IndexSet`, with the `…Assign` forms of the arithmetic and bitwise interfaces |
  | Functions | `print`, `println`, `eprint`, `eprintln`, `input`, `assert`, `assert_eq`, `assert_ne`, `debug_assert`, `panic`, `todo`, `unreachable`, `min`, `max`, `abs`, `clamp`, and Python's `len`, `range`, `sum`, `sorted`, `enumerate`, `zip`, `reversed`, `any`, `all` (`[STD-26]`) |
  | Modules | `mem` (the module `std.mem`) |

  A local declaration or explicit import shadows a prelude name. Everything else in the standard
  library is imported explicitly (`from std.math import Vec3`).

## V.2 Functions

```ember
pub fn blend(dst: Array[f32], mut out: Array[f32], owned scratch: Array[f32], weight: f32 = 0.5) -> int:
    for i in 0..out.len():
        out[i] = dst[i] * weight
    return scratch.len()
```

* `[FN-1]` *(changed in 0.9.9)* **Parameter modes.**
  * `a: A` — **borrowed** (the default). The callee reads the caller's `a`, not a copy of it, and
    cannot move it or assign through it; a write through a `Cell` or `RefCell` inside it changes the
    caller's value. Some parameters are passed as a copy instead, such as a small `Copy` value in
    registers (`[BRW-8]`); the meaning is the same.
  * `mut b: B` — **in-out**. The argument must be a mutable place, or a mutable view derived from one
    (`[FN-1a]`). The callee may change it; it cannot move it out except by `mem.replace`/`mem.take`.
  * `owned c: C` — **consumed**. The argument is moved in (copied if `Copy`; a handle is copied without
    a retain). The callee owns it.
* `[FN-1a]` A `mut` parameter whose type is itself a view (`MutSpan[T]`) accepts a view-producing
  expression such as `buf.as_mut_span()` or an `Array` place (coerced, `[SPN-1]`); the mutable-place
  requirement applies to the place the view is taken of.
* `[FN-2]` An omitted mode is borrowed. There is no by-copy mode; a callee that needs its own copy
  takes `owned` and the caller passes `x.clone()` (or `x`, if `Copy`).
* `[FN-2a]` **A call site never writes a mode.** `f(x)` is written whatever mode `f` declares; the
  compiler forms the borrow or move the declaration requires and reports shape B10 if `x` is not a
  suitable place. The editor shows the mode as an inlay hint.
* `[FN-9]` *(new in 0.9.9)* A mode on a **class-handle** parameter governs the handle, not the object. A borrowed handle
  parameter may still be used to read and write the object's fields and call its methods, subject to
  Part VIII's exclusivity rules; `owned` transfers the reference without a retain; `mut` lets the
  callee re-point the caller's handle variable.
* `[FN-3]` *(changed in 0.9.9)* A function returns by move. Returning a reference or view requires its region to derive
  from a parameter the result may borrow from (`[LT-1]`, `[LT-1a]`), or to be `static` (`[LT-3]`).
* `[FN-4]` A method's receiver follows the same modes: `self`, `mut self`, `owned self`. A function in
  a type body with no receiver is an associated function, called `Type.name(…)`.
* `[FN-5]` Default argument expressions are evaluated at each call, after the other arguments are
  bound, and may refer to earlier parameters.
* `[FN-6]` *(changed in 0.9.9)* **Callable types.** A function is a value. A callable type is written
  `fn(A, mut B, owned C) -> R`, with the same parameter modes as a declaration; `once fn(…) -> R` is a
  callable that may be called at most once (`[CLO-6]`). Callable types are compared by their parameter
  types, modes and result. A function coerces to any callable type it satisfies; a capture-free one
  also coerces to `extern "C" fn(…)` when every type is FFI-safe. The source parameters (`[LT-1]`) of a
  callable type get fresh regions at each call, and the result's regions follow `[LT-1]` applied to
  the callable type's own parameters (`[LT-7]`). What a callable type means in each position is
  `[CLO-3]`.
* `[FN-7]` Recursion is permitted; tail calls are not guaranteed to be eliminated.
* `[FN-8]` *(changed in 0.9.9)* `main` is `fn main()`, `fn main() -> Result[void, E]` for any
  `E: Display`, or either form with a first parameter `args: Array[String]`. `args` holds the command
  line, decoded as UTF-8 with invalid bytes replaced by U+FFFD; `std.process.args_os()` returns the raw
  bytes. An `Err` returned from `main` is printed with `Display` to standard error and the process
  exits with status 1. **Scripts:** the statements at file scope of the entry file (`[GRM-2]`) form, in
  source order, the body of an implicit `fn main()`, or `fn main() -> Result[void, AnyError]` when one
  of them uses `?`; items anywhere in the file are declared as usual. A file with such statements that
  also declares `main` has two items named `main` (`E1030`). The names the statements bind are locals
  of the implicit `main`: a function of the file that uses one is `E1010`, whose help names a
  parameter, a `const` or a `static` (unlike Python, where it would read a global). `return` and
  `yield` at file scope are `E0100`, as in Python; `process.exit(code)` ends a script early.
* `[FN-10]` *(new in 0.9.9)* A function whose return type is `Result[void, E]` returns `Ok(())` when
  control reaches the end of its body, as a `void` function returns; an explicit `return Ok(())` is
  still allowed. No other return type has an implicit value: a body that can reach its end is
  `E2182`, reported at the function's name (ODR-023).

## V.3 Structs

```ember
@derive(Copy)
pub struct Vec2:
    pub x: f32
    pub y: f32

    const ZERO: Vec2 = Vec2(0, 0)

    fn length(self) -> f32:
        return (self.x * self.x + self.y * self.y).sqrt()

    fn scale(mut self, k: f32):
        self.x *= k
        self.y *= k
```

* `[STR-1]` Every struct has a **memberwise constructor** `Name(field0, field1, …)` taking positional
  or named arguments in field order; a field with a default may be omitted. It is `pub` iff every
  field is `pub`. A declared `fn init(self, …)` replaces it (`[STR-6]`).
* `[STR-2]` *(changed in 0.9.9)* A field default is any expression; it is evaluated at each
  construction that omits the field, in field order, and may allocate (`items: Array[int] = []`).
* `[STR-3]` A struct with a `drop` method, or with a field that needs drop, is move-only. `Copy` is
  never implicit: `@derive(Copy)` requests it, and is `E2080` if a field is not `Copy`.
* `[STR-4]` A struct may have no fields (`struct Marker: pass`).
* `[STR-5]` *(changed in 0.9.9)* **Implicit derives.** A struct or enum implements `Eq`, `Debug` and
  `Clone` field-wise, automatically, when every field implements the interface; `@no_derive(Eq)`
  (or `Debug`, `Clone`) opts out, and a hand-written implementation replaces the implicit one. A type
  that declares `drop` is not implicitly `Clone`: a field-wise copy would release twice what its
  `drop` releases, so it is `Clone` through `@derive(Clone)` or a written `clone` (ODR-026).
  `Copy`, `Ord`, `Hash`, `Default` and `Display` are never implicit and are requested with
  `@derive(…)` or written by hand.
* `[STR-7]` *(new in 0.9.9)* Inside the body of a `struct`, `enum`, `class` or `extend` block, `Self`
  names the type being declared with its own generic parameters, and the type's name may be written
  with any arguments: in `struct Pair[T]:`, a method may return `Self`, `Pair[T]` or `Pair[int]`.
* `[STR-6]` *(new in 0.9.9)* `fn init(self, …)` on a struct is its constructor: every field without a default MUST be
  assigned on every path before `self` is used as a whole (`E2100`). `mut self` is accepted in an
  `init` and means the same.

## V.4 Enums

```ember
@repr(u8)
enum RenderMode:
    Forward = 0
    Deferred = 1

enum Shape:
    Circle(radius: f32)
    Rect(w: f32, h: f32)
    Empty

    fn area(self) -> f32:
        return match self:
            Circle(r) => 3.14159 * r * r
            Rect(w, h) => w * h
            Empty => 0.0
```

* `[ENM-2]` A `match` on an enum MUST be exhaustive (`E2090` lists the missing variants); `_` covers
  the rest.
* `[ENM-3]` A unit-only enum is `Copy`, `Eq`, `Ord` (declaration order), `Hash` and `Debug`
  automatically, converts to its representation with `as`, and converts back with
  `Mode.from_repr(x) -> Option[Mode]`.
* `[ENM-4]` A payload enum is `Copy` only by `@derive(Copy)` with every payload `Copy`; `[STR-5]`'s
  implicit derives apply to it.

## V.5 Classes

```ember
open class Script:
    let entity: int
    pub(read) enabled: bool = true

    fn init(self, entity: int):
        self.entity = entity

    virtual fn on_update(self, dt: f32):
        pass

class Door(Script):
    angle: f32 = 0.0

    override fn on_update(self, dt: f32):
        self.angle = min(self.angle + 90.0 * dt, 90.0)
```

* `[CLS-1]` A class instance lives on the heap with the header of `[OBJ-1]`. `Name(args)` allocates,
  runs the constructor and returns a handle with strong count 1.
* `[CLS-2]` *(changed in 0.9.9)* `fn init(self, …)` is the constructor. Before any `init` body runs,
  every field default of the class and of its bases is evaluated and stored. Every field without a
  default MUST then be assigned on every path before `self` is used as a whole (`E2100`); `self` is
  used as a whole by a method call on it, by passing, storing or returning it, and by a closure
  capturing it. Several constructors are written as associated functions that call `Self(…)`.
* `[CLS-3]` A class with no `init` and no base class gets a memberwise constructor as a struct does.
* `[CLS-10]` *(new in 0.9.9)* A derived class with no `init` gets its base's constructor: `Door(entity)`
  evaluates `Door`'s field defaults and then runs `Script.init(entity)`. A derived class with a field
  that has no default MUST declare an `init` (`E2101`).
* `[CLS-4]` *(changed in 0.9.9)* A class is final unless declared `open` or `abstract`. Methods are
  non-virtual unless `virtual`; replacing one requires `override`; overriding a non-virtual method is
  `E2110`. A method with a receiver named like an inherited one replaces it (ODR-028): over a virtual
  method without `override` it is `E2111`, and over a non-virtual one it is `E2110`, `override` or
  not; `init`, `drop` and associated functions are each class's own. An `override` is itself virtual,
  so a further subclass may override it again. A derived `init` MUST call `super.init(…)` exactly once
  (`[CLS-11]`). Fields cannot be overridden. Base fields are laid out first; there is at most one base.
* `[CLS-11]` *(new in 0.9.9)* Construction is two-phase. In a derived `init`, the code before
  `super.init(…)` MUST assign every field the class itself declares without a default, and MUST NOT
  read an inherited field or use `self` as a whole; the code after it may do anything (`E2102`,
  naming the field or the use). So when any method runs, even a `virtual` method called from a base
  `init`, every field of the object holds a value.
* `[CLS-5]` Interface calls on a class are dispatched statically unless the receiver is `dyn`.
* `[CLS-6]` Destruction runs the derived `drop`, then the base's, then drops the fields in reverse
  declaration order, then frees the memory once no weak handle remains.
* `[CLS-7]` *(changed in 0.9.9)* Inside a class method, `self` is a handle. **Any method may read and
  write the object's fields through `self`**; each access is checked on its own by Part VIII's rules,
  so a class method needs no `mut` to mutate its object (reference semantics, as in Python). `mut self`
  on a class method declares that the method holds a write access to every field of the object for its
  whole duration, which the compiler may use to remove the per-access checks inside it (`[EXC-15]`).
* `[CLS-7a]` Inside `drop`, `self` MUST NOT be stored anywhere that outlives the call (`E3016`).
* `[CLS-8]` *(changed in 0.9.9)* A class is `Sync` only as `[THR-1]` allows; every field of a `Sync`
  class is immutable after `init`.
* `[CLS-9]` A `let` field is assignable only in `init`.
* `[CLS-9a]` A `let` field of a non-`Copy` type may still be mutated *through* (`self.items.push(v)`);
  `let` fixes the binding, not the value.

## V.6 Interfaces and `extend`

```ember
pub interface Shape2D:
    fn area(self) -> float
    fn describe(self) -> String:
        return f"shape with area {self.area():.2f}"

struct Square:
    side: float

extend Square implements Shape2D:
    fn area(self) -> float:
        return self.side * self.side
```

* `[IFC-1]` *(changed in 0.9.9)* `extend T:` without `implements` adds inherent methods to `T`; it is
  legal in any module of `T`'s package.
* `[IFC-2]` Adding inherent methods to a type from another package is `E2120`; use an interface or a
  wrapper type.
* `[IFC-3]` `interface Ord: Eq` requires implementers of `Ord` to implement `Eq`.
* `[IFC-4]` Interfaces may declare associated types and constants, not statics.

## V.7 Constants and statics

* `const NAME: T = e` is a compile-time constant, inlined at each use; `e` MUST be evaluable at compile
  time (Part XIV). `T` MUST be `Copy`, `str`, a `bytes` literal type, or a fixed array of those
  (`E2130` otherwise, whose help suggests a `static`).
* `[STA-1]` *(changed in 0.9.9)* `static NAME: T = e` is one value per program with a stable address.
  A `static` is immutable; mutation goes through a `Sync` interior type (`Atomic`, `Mutex`, `RwLock`,
  `Once`). `T` MUST be `Sync` (`E7002`). `static mut` exists only for foreign interfaces and every
  access to it requires `unsafe`.
* `[STA-3]` *(new in 0.9.9)* A static whose initialiser can be evaluated at compile time is placed in the image with no
  initialisation code. Any other initialiser runs **once**, on first access, exactly once even under
  concurrent first access; the first access blocks until it completes. A panic during initialisation
  aborts. An initialiser that reaches its own static is `static initialisation cycle`, a panic naming
  the chain.

```ember
from std.sync import Mutex

static GREETING: str = "hello"
static REGISTRY: Mutex[Map[String, int]] = Mutex({})
```

* `[STA-2]` There is no static-initialisation-order problem: statics are initialised at compile time
  or on first access, never by a global constructor.

## V.8 Attributes

* `[ATT-1]` *(changed in 0.9.9)* An attribute that is neither in the table below nor a visible
  `@attribute` struct (`[RFL-3]`) is `E0104`. An attribute on a declaration it does not apply to is
  `E0104` naming the positions it does apply to.
* `[ATT-4]` One attribute per line.
* `[ATT-6]` *(new in 0.9.9)* Every attribute in the table has exactly the effect its rule gives. An implementation that
  has not built an attribute's effect rejects the attribute with `E0900` (`[PHIL-12]`); it never
  accepts and ignores it.
* `[ATT-2]` A statement may carry only `@parallel`, `@unroll`, `@simd` and `@allow`; the first three
  only on a `for` statement (`E0108`).

| Attribute | Applies to | Rule |
|---|---|---|
| `@derive(A, …)`, `@no_derive(A, …)` | struct, enum, class | `[STR-5]`, `[DRV-1]` |
| `@layout(c)`, `@packed`, `@align(N)`, `@repr(T)` | struct, enum | `[LAY-2]` |
| `@gpu_layout(std140\|std430\|scalar)` | struct | Annex D |
| `@noalloc`, `@nosync`, `@noblock`, `@noio`, `@nolock`, `@nopanic(explicit)`, `@static_safe`, `@deterministic`, `@realtime` | fn, interface method; `@deterministic` also module and fn type | Part X |
| `@overflow(panic\|wrap\|saturate)` | fn, module | `[TYP-8]` |
| `@fastmath`, `@fp(contract)` | fn | `[TYP-9]` |
| `@inline`, `@noinline`, `@cold`, `@hot` | fn | `[CG-C-3a]` |
| `@simd`, `@simd(assert)`, `@parallel(…)`, `@unroll(N)` | `for` statement | Part XII |
| `@must_use` | fn, type | `[ERR-5]` |
| `@deprecated(since, note)` | any item | `[VER-3]` |
| `@export("symbol")`, `@export_table("Name", protocol=N)` | fn, static; struct | `[FFI-26]` |
| `@ffi(…)` | extern item, overlay item | Part XVI |
| `@test`, `@bench`, `@should_panic` | fn | `[TST-3]` |
| `@view` | struct, enum | `[TYP-34]` (documentation only) |
| `@sync` | class | `[THR-1]` |
| `@reflect` | type | `[RFL-2]` |
| `@borrows(p, …)` | fn | `[LT-1a]` |
| `@safety("…")` | unsafe fn | `[UNS-7]` |
| `@must_drop` | struct, class | `[THR-6]` |
| `@allow(code, …)` | any item or statement | suppresses the named `W`/`L` diagnostics |
| `@non_exhaustive` | enum | `[FFI-8]` |
| `@component(layout=soa\|aos)`, `@soa(flatten)` | struct; field | Part XII |
| `@always_specialize`, `@never_specialize` | generic fn or type | `[MONO-7]` |
| `@reloadable`, `@noreload`, `@renamed_from("n")`, `@reinit_on_reload`, `@allow_reload_terminate` | see Annex B | Annex B |
| `@prelude` | module (standard library only) | `[MOD-5]` |

*Reserved* (recognised and rejected with `E0104` naming the version): `@nopanic` (without
`(explicit)`), `@no_runtime_checks`, `@allocator(Name)`, `@gpu`.
---

# Part VI — Expressions and Statements

## VI.1 Evaluation order

* `[EXP-1]` Operands, arguments, and the elements of tuple, list, map and set literals are evaluated
  left to right, completely, before the operation or call. Named arguments are evaluated in the order
  written.
* `[EXP-2]` *(changed in 0.9.9)* An assignment evaluates its right side first, into a temporary if it
  has several elements, then the target places, then stores left to right. `a[i], a[j] = a[j], a[i]`
  swaps. An augmented assignment `a[i] += x` evaluates `x`, then the place once, then reads, operates
  and writes.
* `[EXP-3]` `and` and `or` short-circuit. `x if c else y` evaluates `c` and then exactly one branch. A
  comparison chain stops at the first false link (`[GRM-25]`).
* `[EXP-4]` *(changed in 0.9.9)* A temporary created while evaluating an expression statement is
  dropped at the end of that statement, in reverse order of creation. A temporary created while
  evaluating the **condition** of an `if`, `elif` or `while` is dropped before the chosen block runs,
  unless the condition is a pattern test whose bindings refer to it, in which case it lives to the end
  of that block only — never into an `elif` or `else`. A temporary bound by `with` lives to the end of
  the `with` block; one created by a `for` iterable lives to the end of the loop.

## VI.2 Places, values and moves

A **place** denotes memory: a local, a field of a place, an element of a place, the target of a
reference or `Box`, a field of a class object reached through a handle, a `static`. Everything else is
a value.

* `[EXP-5]` Mutation, a mutable borrow and a move require a place; `f(x).y = 1` is `E2140` unless `f`
  returns `ref mut`.
* `[EXP-6]` A place of a non-`Copy` type used as a value (assigned, passed to `owned`, returned,
  captured by `owned fn`) is **moved**, and is uninitialised until assigned again. Moving out of a
  field of a type with `drop` is `E3010`; out of an element of an array, `Array` or `Span` is `E3011`
  (use `mem.take`, `mem.replace`, `swap`, `pop`, `remove`); out of a class field is `E3012`; out of
  anything reached through a reference is `E3013`. Moving out of a plain struct's field leaves the
  struct partially moved; it cannot be used whole until the field is assigned again (`E3042`).

## VI.3 Operators and literals

| Expression | Meaning | Notes |
|---|---|---|
| `a + b`, `a - b`, `a * b` | arithmetic | overflow panics (`[TYP-8]`) |
| `a / b` | true division | floats only; integers are `E2240` (`[TYP-28]`) |
| `a // b`, `a % b` | floor division and modulo | Python meaning |
| `a ** b` | power | `[TYP-30]` |
| `a == b`, `a != b` | `Eq` | built in for scalars |
| `a < b` etc. | comparison | scalars built in; generic code through `Ord.cmp`; chains (`[GRM-25]`) |
| `x in c`, `x not in c` | `c.contains(x)` | `Contains` (`[STD-8]`) |
| `a is b`, `a is not b` | handle identity | class handles and references (`[EXP-9]`) |
| `x is None`, `x is not None` | option test | `Option` (`[EXP-9]`) |
| `a[i]` | `Index` / `IndexMut` / `IndexSet` | read; written in place; `a[i] = v` per `[STD-17]` |
| `a[i..j]`, `a[..j]`, `a[i..]` | slice | `Span`/`MutSpan`/`str`, bounds-checked |
| `a?.f`, `a?.m()` | optional chaining | on `Option`: `None` propagates |
| `e?` | early return | on `Option` or `Result` (`[ERR-2]`) |
| `x as T`, `h as? D`, `h as! D` | conversion | `[TYP-6]` |
| `f"…{e}…"` | formatting | allocates a `String`; `E4001` in `@noalloc` (use `format_to`) |
| `[a, b]`, `{k: v}`, `{a, b}` | collection literals | `[TYP-38]` |

* `[EXP-9]` *(new in 0.9.9)* *(changed in 0.9.9)* `a is b` compares two class handles (or two references) for identity.
  `x is None` and `x is not None` test an `Option` for absence and presence, and are the only uses of
  `is` on a non-handle type. Any other operand of `is` is `E2150`, whose help names `==`.
* `[TYP-38]` *(new in 0.9.9)* **Collection literals.**
  * A list literal `[a, b, c]` has the type its context expects: `Array[T]` (allocates), `[T; N]` (no
    allocation) or `Span[T]` (a view of a temporary fixed array). With no context it is `Array[T]`.
    `[v; N]` is always the fixed array of `N` copies of `v`. A list literal is never a `Set`: in a
    `Set` position it is `E2020`, whose help is `{a, b}`.
  * A map literal `{k: v, …}` is a `Map[K, V]` (insertion-ordered, `[STD-11]`); a set literal
    `{a, …}` is a `Set[T]`. A repeated key keeps its first position and its last value, as Python's
    `dict` does; `{1, 1}` is `{1}`. With no context a text key, value or element is a `String`, since
    a map holds no views (`[STD-11]`).
  * `[]` and `{}` need their element type from the context or from later uses in the function
    (`[TYP-23]`); left open, they are `E2060`.
  * Every element converts to the element type by `[TYP-5]`, so `["ann", "bob"]` in an
    `Array[String]` position is two `String`s, and in an `Array[str]` position is two static `str`s.
  * A literal that allocates carries `Alloc`; in a `@noalloc` function it is `E4001`, whose help names
    the fixed-array form. A generator expression (`[GRM-38]`) is not a collection and never allocates.

```ember
fn main():
    counts: Map[String, int] = {}
    for word in "the cat and the hat".split(" "):
        counts[word] = counts.get_or(word, 0) + 1
    squares = [n * n for n in 0..10 if n % 2 == 0]
    seen = {"ann", "bob"}
    if "ann" in seen and 0 <= squares[1] < 10:
        println(counts, squares)
```

## VI.4 Control flow

* `[CTL-0]` The condition of `if`, `elif`, `while` and a match guard MUST be a `bool`; there is no
  truthiness. `E2035` carries a type-directed fix-it: `not xs.is_empty()` for a container or string,
  `x is not None` for an `Option`, `x != 0` for a number.
* `[CTL-1]` *(changed in 0.9.9)* `for pattern in e:` iterates:
  * a place `e` whose type is `Iterable`: borrows `e` for the loop and calls `e.iter()`; elements are
    `ref T` (read through wherever a `T` is expected);
  * `owned e`, or a value `e` whose type is `IntoIterator`: consumes `e`;
  * an `Iterator` value: calls `next` until `None`.
  `Array`, `[T; N]`, `Span`, `MutSpan`, `Set`, `Map`, ranges and generators are iterable; a `Map`
  yields its keys, as Python's `dict` does (`for k, v in m.items():` gives pairs, `m.values()` the
  values), and `for k in owned m:` yields them owned, dropping the values (`into_items()` keeps
  both); a `str` yields its `char`s. `for x in
  xs.iter_mut():` yields `ref mut T`.
* `[CTL-2]` The iterated place is borrowed for the whole loop; mutating it inside the loop is `E3020`,
  whose help names `retain`, `drain`, collecting first, or an index loop.
* `[CTL-3]` `a..b` (`Range`), `a..=b` (`RangeInclusive`) and `a..` (`RangeFrom`) over integers are
  iterable and compile to a counted loop with no iterator object. `a..=MAX` terminates after `MAX`;
  `a..` has no end, and counting up to its type's maximum is an overflow (`[TYP-8]`). A range is a
  value (ODR-027): `Range[T]`, `RangeInclusive[T]`, `RangeFrom[T]` and `RangeTo[T]` (`..b`) are
  prelude structs with public bounds, `start` and `end` (a `RangeFrom` has only `start`, a `RangeTo`
  only `end`), and are `Copy` when `T` is. A `for` over a range value counts over a copy of its
  bounds, so the range is unchanged and can be iterated again, as Python's `range` can.
* `[CTL-3b]` *(changed in 0.9.9)* Iteration over ranges, `Span`, `MutSpan`, `Array`, `[T; N]`, `SoA`
  columns, and `enumerate`, `zip`, `take`, `skip`, `step_by`, `copied` and `rev` composed over them,
  MUST compile to an induction-variable loop over base pointers and lengths, with no iterator object in
  memory and no call per element, independent of the host C compiler's optimiser. `step_by(k)` becomes
  the increment `k` (a `k` that is not a positive constant is checked once before the loop).
  `tests/conformance/CTL-3b/` asserts it on the emitted C.
* `[CTL-4]` The `else` of `while` or `for` runs when the loop ends without `break`.
* `[CTL-5]` A `match` tests arms top to bottom; the first that matches runs; a guard is evaluated after
  binding; the arms MUST be exhaustive (`E2090`); an unreachable arm is `W2091`.
* `[CTL-6]` `with a = e1, b = e2:` binds `a` and `b` for the block and drops them in reverse order when
  it ends. `with e:` keeps the temporary alive for the block (a guard).
* `[CTL-7]` `defer:` registers a block to run when the enclosing block exits, last registered first.
  It may refer to locals declared before it (borrowing them until the block ends). It cannot `return`,
  `break` or `continue` (`E2160`).
* `[CTL-8]` On every exit from a block, its `defer` blocks run first and then its locals are dropped in
  reverse declaration order.
* `[CTL-9]` `pass` does nothing; an empty block is written `pass`.
* `[CTL-10]` *(new in 0.9.9)* **Names assigned in every branch.** In an `if`/`elif`/`else` that has an `else`, or an
  exhaustive `match` statement, a name that is not in scope before the statement and is declared (by
  `x = e`) in **every** arm that completes normally, at the **same type**, is declared in the enclosing
  block and is initialised after the statement. An arm that ends in `return`, `break`, `continue` or a
  panic does not need to declare it. Different types in different arms are `E2230`, naming each arm.

```ember
fn describe(n: int) -> String:
    if n < 0:
        kind = "negative"
    elif n == 0:
        kind = "zero"
    else:
        kind = "positive"
    return f"{n} is {kind}"
```

## VI.5 Closures and callable values

```ember
fn apply_twice(f: fn(int) -> int, x: int) -> int:
    return f(f(x))

fn make_adder(n: int) -> fn(int) -> int:
    return owned fn(x: int) => x + n

struct Button:
    label: String
    on_click: fn()

fn main():
    step = 3
    println(apply_twice(fn(x) => x + step, 1))
    add5 = make_adder(5)
    b = Button(label="ok", on_click=owned fn() => println("clicked"))
    b.on_click()
    println(add5(1))
```

* `[CLO-1]` A lambda or local function has a unique anonymous type implementing its callable type. If
  it captures anything by reference it is a view type; if it is `owned fn` or captures nothing it is a
  plain value.
* `[CLO-2]` *(changed in 0.9.9)* Captures are inferred per variable: read only ⇒ shared borrow; written ⇒
  mutable borrow; `owned fn` ⇒ each captured variable is moved in (copied if `Copy`, retained if a
  handle). A closure that writes its captures is called from a mutable place (`mut f` for a
  parameter). An **owned callable value** (`[CLO-3]`) is always called from a mutable place, because
  its type does not say whether it writes its captured state; one stored in a class field is called
  under a write access to that field (§VIII.3), so the closure cannot replace or free itself while it
  runs.
* `[CLO-13]` *(new in 0.9.9)* A read-only capture of a `Copy` variable whose storage would end while
  the closure is still live, and which is not assigned after the closure is created, is captured by
  **copy** instead of by borrow (for a handle, a retain). The difference cannot be observed; it lets a
  closure created in a loop use the loop variable after the iteration ends. A captured variable that is
  assigned later keeps the borrow rule, and its error's help is `owned fn`.
* `[CLO-3]` *(changed in 0.9.9)* **What `fn(A) -> R` means depends on where it is written.**
  * As a **parameter type** it is a bound, not a representation: the parameter is an implicit generic
    constrained by the callable type, each argument monomorphises the callee, and a call through it is
    a direct call. This is the zero-cost form. It accepts any lambda, local function or function,
    including non-`owned` lambdas that borrow the caller's locals.
  * In **any other position** — a field, a local annotation, a return type, a collection element, a
    `static` — it is an **owned callable value**: a function pointer plus the callee's captured state,
    owning that state. A value of this kind may be stored anywhere an owned value may. It is filled
    from a function, a capture-free lambda, or an `owned fn` lambda; a lambda that borrows is a view
    and may only be stored where `[TYP-15]` permits (`E3063` otherwise, whose help is `owned fn`).
  * `extern "C" fn(A) -> R` is a C function pointer: capture-free functions only.
* `[CLO-14]` *(new in 0.9.9)* A callable type may also be written as an explicit generic bound,
  `fn apply[F: fn(int) -> int](f: F, x: int)`, with the meaning of the parameter form of `[CLO-3]`;
  a parameter so bounded is called like a function. `Callable[…]` is not source syntax.
* `[CLO-10]` *(new in 0.9.9)* An owned callable value holds up to **three pointer-sized words** of captured state inline
  and makes no allocation; larger captured state is placed in one heap allocation, which carries the
  `Alloc` effect and is reported by `ember inspect --alloc`. Calling one is one indirect call.
* `[CLO-4]` *(changed in 0.9.9)* A non-`owned` lambda cannot outlive what it borrows: storing it,
  returning it or passing it to an `owned` parameter follows the view rules (`[TYP-15]`), except that a
  lambda written directly as the argument of a consumed callable parameter captures by move
  (`[CLO-15]`).
* `[CLO-5]` A closure capturing a class handle holds a strong reference; the usual cycle caution
  applies (`[WK-1]`).
* `[CLO-6]` *(changed in 0.9.9)* A lambda that moves one of its captures out of itself (into an
  `owned` parameter, a return, a field) can be called only once. Its type satisfies `once fn(A) -> R`
  but not `fn(A) -> R`; supplying it where `fn` is required is `E3030`, shape O5. A parameter `f: once
  fn(A) -> R` accepts both kinds and calling `f` consumes it; a second call is `E3040`.
* `[CLO-6a]` *(changed in 0.9.9)* An owned `once fn` value, including one inside a `Box` or a
  collection, is callable: the call moves the value out of its place (the place becomes empty, and a
  collection element is removed by the calling API, e.g. `jobs.pop_front()`).
* `[CLO-7]` *(changed in 0.9.9)* Standard-library APIs that store or send a callback (`thread.spawn`,
  `jobs.submit`, event registries) take it as a
  consumed callable parameter (`owned f: fn(A) -> R`) or `once fn`, so a lambda written at the call
  site captures by move (`[CLO-15]`).
* `[CLO-15]` *(new in 0.9.9)* A lambda written directly as the argument of a consumed callable
  parameter (`owned f: fn(A) -> R`, which `thread.spawn`, `jobs.submit` and callback registries
  declare, `[CLO-7]`) captures by move, as if written `owned fn`: `thread.spawn(fn() => work(x))` moves
  `x` into the thread. `owned fn` is needed only for a lambda that is stored before it is passed, or
  stored in a field or a local.
* `[CLO-11]` *(new in 0.9.9)* A call `recv.name(args)` where `recv`'s type has no method `name` but has a field `name`
  of callable type calls that field. A method of that name takes precedence.
* `[CLO-12]` *(new in 0.9.9)* A local function (`[GRM-28]`) is a named closure: it captures like a lambda and follows
  `[CLO-1]`–`[CLO-6]`.

## VI.5a Generators

A **generator** is a function whose body runs step by step, producing values with `yield`. It is
Python's generator, and it is also how Ember writes frame-spanning gameplay sequences.

```ember
gen fn countdown(n: int) -> Generator[int]:
    i = n
    while i > 0:
        yield i
        i -= 1

gen fn evens(xs: Span[int]) -> Generator[int]:
    for x in xs:
        if x % 2 == 0:
            yield x

fn main():
    for t in countdown(3):
        println(t)
    data = [1, 2, 3, 4]
    total = evens(data).sum()
    println(total)
```

* `[CORO-1]` *(changed in 0.9.9)* A `gen fn` declares a generator. Calling it runs none of its body; it
  returns the generator's **frame**, a value holding the parameters and suspended state. The declared
  return type is `Generator[Y]` or `Generator[Y, R]`, where `Y` is the type of each yielded value and
  `R` (default `void`) the type of the final `return` value.
* `[CORO-2]` `yield e` suspends the generator and produces `e`. `yield` outside a `gen fn` is `E2220`.
  A `gen fn` with no `yield` is accepted and produces `L2004`.
* `[CORO-3]` *(changed in 0.9.9)* `Generator[Y, R]` in a signature names the function's own frame type
  opaquely, as `some Iterator[Item = Y]` would: each `gen fn` has a distinct, sized, move-only frame
  type that implements `Iterator[Item = Y]`. `next()` resumes the body to the next `yield` (returning
  `Some(y)`) or to its end (returning `None`, after which `result() -> Option[R]` gives the returned
  value). Calling `next()` after the end returns `None` again. Generators of different functions are
  kept together as `Box[dyn Iterator[Item = Y]]`. `yield` as an expression has type `void`.
* `[CORO-4]` The compiler rewrites the body into a state machine over the frame: locals live across a
  `yield` become frame fields, others stay on the stack. No run-time machinery exists beyond the frame.
* `[CORO-5]` **A generator allocates nothing.** The frame's size is a compile-time constant; it is an
  ordinary move-only value that may live in a local, a field, an `Array` or an arena. `Box` it only to
  store generators of different functions together. A `gen fn` carries `Alloc` only if its body does.
* `[CORO-6]` *(changed in 0.9.9)* A reference or view **to a local of the generator's own frame** may
  not be live across a `yield`: moving the suspended frame would leave it dangling (`E2221`, whose help
  names iterating `owned` the collection, or iterating by index). References and views that came in
  **as parameters** or were derived from them, and views derived from a source parameter (`[LT-1]`),
  may be held across `yield`; the frame is then a view
  type bound by their regions, exactly as a returned view would be (`[LT-1]`). `evens` above is such a
  generator.
* `[CORO-7]` Dropping a suspended generator drops exactly the locals live at its suspension point, in
  reverse declaration order.
* `[CORO-8]` A generator's effect set is the union over its whole body; contracts apply to it as to any
  function.
* `[CORO-9]` A frame never points into itself in Safe code; `unsafe` code that builds one states it in
  `@safety`.
* `[CORO-10]` A `gen fn` may not be `extern`, `@export`ed or passed to C (`E2222`).
* `[CORO-12]` *(new in 0.9.9)* A `gen fn` **method of a class** takes `self` (the frame retains the handle). Each access
  to the object inside it is checked on its own; a long-term access to a field of the object
  (`[EXC-1]`) may not be live across a `yield` (`E2229`). `mut self` is not permitted on a `gen fn` (`E2229`).
* `[CORO-13]` *(new in 0.9.9)* A `yield` while a `@must_drop` value (`[THR-6]`) is live is `E2231`: a
  suspended generator can be forgotten or leaked, so the value's `drop`, on which a safety guarantee
  depends, might never run (a `thread.scope()` whose tasks borrow the generator's parameters). A scope
  that closes before the next `yield` is unaffected.
* `[CORO-11]` `std.coroutine` builds gameplay sequencing on generators with no further compiler
  support: `Scheduler` resumes a set of `Generator[Wait]` once per frame and drops the finished ones;
  `wait(seconds)`, `wait_frames(n)` and `wait_until(pred)` produce `Wait` values.

```ember
from std.coroutine import Wait, wait

class Door:
    open_angle: float = 0.0

    gen fn swing_open(self) -> Generator[Wait]:
        yield wait(0.5)
        for _ in 0..60:
            self.open_angle += 1.5
            yield Wait.NextFrame
```

## VI.6 Assertions and panics

* `assert(cond)`, `assert(cond, msg)`, `assert_eq(a, b)`, `assert_ne(a, b)` are checked in every
  profile; `debug_assert(…)` only in `debug`, and it MUST NOT have side effects that change a program's
  result (`W2016` if its argument calls a function with effects other than `Panic`).
* `panic(msg)`, `todo()`, `unreachable()` have type `Never`.
* `[PAN-1]` A panic prints `panic at <file>:<line>:<col>: <message>` (and a backtrace outside
  `shipping`) to standard error and calls `abort()`. There is no unwinding in this version; `defer`
  blocks and destructors do not run on panic.
* `[PAN-2]` Formatting a panic message allocates only for an f-string; `panic("literal")` and
  `panic(some_str)` are `@noalloc`.
* `[PAN-3]` A panic inside a `drop` that runs while the process is already panicking aborts at once.
---

# Part VII — Ownership, Borrowing and Regions

This Part specifies the **value world**: structs, enums, collections and views. Class objects are
Part VIII; here a class handle is simply a `Copy` value whose copy retains.

## VII.1 Ownership

* `[OWN-1]` Every value has exactly one owner: a local, a field of an owned value, an element of an
  owned collection, a temporary, or a `static`.
* `[OWN-2]` A value is **dropped** when its owner goes out of scope (block end, reverse declaration
  order), when its owner is overwritten, or at the end of the statement that created it as a
  temporary. Dropping runs the value's `drop` method, if any, then drops its fields in reverse
  declaration order.
* `[OWN-3]` A move transfers ownership and leaves the source uninitialised. Using a moved-from place is
  `E3040`, labelled at the move and at the use, with the help `.clone()` when the type is `Clone`. A
  value moved on only some paths is tracked with a hidden drop flag; conditional moves are legal.
* `[OWN-4]` A loop body that moves a value declared outside the loop is `E3041`, unless every path
  assigns it again before the next iteration.
* `[OWN-5]` Assigning to a place that holds a live value evaluates the new value, then drops the old
  one, then stores the new one. `Cell.set` stores first and drops after (`[CELL-1]`); `MaybeUninit.write`
  stores without dropping (`[ARN-8]`). These three orders are distinct and none generalises the others.
* `[OWN-6]` *(changed in 0.9.9)* `mem.take(mut place: T) -> T` (leaves `Default`),
  `mem.replace(mut place: T, owned new: T) -> T`, `mem.swap(mut a: T, mut b: T)`, `mem.drop(owned x: T)`
  (drops now) and `mem.forget(owned x: T)` (never drops; `E3015` for `@must_drop` types, `[THR-6]`) are
  the operations that move values out of places a move may not otherwise leave empty. Calls are
  written without modes: `mem.take(self.items)` (`[FN-2a]`).

## VII.2 Copy and Clone

* `[OWN-7]` A `Copy` value is duplicated bitwise on use and the source stays valid. A `Copy` type has no
  `drop`. A class handle is `Copy`, but copying it retains and dropping it releases (`[RC-1]`).
* `[OWN-8]` `Clone.clone(self) -> Self` is the explicit deep copy. Cloning a class handle copies the
  handle; copying the object is a method the class writes.

## VII.3 Borrows

A **borrow** creates a reference, or a view containing one, to a place without taking ownership.
Borrows are created implicitly when a place is passed to a borrowed or `mut` parameter or receiver,
iterated, indexed or coerced to a view, and explicitly with `ref place` / `ref mut place` (needed only
to initialise a `ref`-typed local or field).

* `[BRW-1]` **Aliasing xor mutation.** At every program point a place has any number of live shared
  borrows, or exactly one live mutable borrow. While a mutable borrow is live the owner cannot read,
  write, move or drop the place; while shared borrows are live the owner can read but not write, move
  or drop. A reference local is never re-seated: `r = e` writes through `r` (and is legal only if `r`
  is `ref mut`).
* `[BRW-2]` **Liveness.** A borrow is live from its creation to the last use of anything derived from
  it — the reference, a reborrow, a view built from it, a value returned from a call that borrowed it.
  Scope does not matter: `n = v.len(); v.push(n)` is legal.
* `[BRW-3]` *(changed in 0.9.9)* **Two-phase borrows.** For a method call whose receiver is a place, or
  an argument passed to a `mut` parameter that is a place, the mutable borrow is **reserved** when the
  argument list begins and **activated** at the call. Between reservation and activation the place may
  be read, and values computed from it may be passed (`v.push(v.len())`); but a borrow of the place that
  is still live at activation — because it is itself an argument, or is held by one — conflicts with the
  activation and is `E3021`. For `fn f(mut v: Array[int], x: ref int)`, the call `f(v, v[0])` is
  therefore rejected; for `fn f(mut v: Array[int], x: int)` it is accepted, because `v[0]` is copied
  before activation.
* `[BRW-11]` *(new in 0.9.9)* **All borrows a call makes are live together.** The borrows formed for every argument and
  the receiver of one call are live simultaneously for the duration of the call. Two of them that
  conflict under `[BRW-1]` — two `mut` arguments naming overlapping places, or a `mut` argument and a
  shared view of the same place — are `E3022` (two mutable) or `E3021` (mutable and shared).
* `[BRW-4]` *(changed in 0.9.9)* **Disjoint fields.** `ref mut a.x` and `ref mut a.y` may be live
  together when `x` and `y` are different fields, through any depth of field projection. Accesses to a
  class object's fields are governed by Part VIII instead.
* `[BRW-10]` *(new in 0.9.9)* **Methods borrow the fields they use.** A call to a method of a struct or enum that is not
  visible outside its package and not `virtual` borrows only the fields its body reads or writes (and
  their transitive callees of the same kind), as computed by the compiler; so `w.bump()` may be called
  while a loop iterates `w.names` if `bump` touches only `w.count`. A method visible outside its package
  borrows all of `self`, so that changing its body cannot break a caller in another package.
* `[BRW-5]` **Indices are not disjoint.** `ref mut a[i]` and `ref mut a[j]` conflict unless both indices
  are constants and differ. `split_at`, `chunks_mut`, `iter_mut`, `swap(i, j)` and `SoA` columns are the
  ways to hold two mutable views of one container (shape B1).
* `[BRW-6]` **Reborrows.** From `r: ref mut T`, `ref r.f` freezes `r` while it lives and `ref mut r.f`
  suspends `r`. Passing a `ref mut` local to a `mut` parameter reborrows it; it does not move it.
* `[BRW-7]` Borrowing a moved or uninitialised place is `E3050`.
* `[BRW-8]` *(changed in 0.9.9)* A borrowed parameter is passed by address: the callee reads the
  caller's place (`[FN-1]`, ODR-024). Two kinds are passed as a copy instead:
  * a reference or view (a `ref`, `Span`, `MutSpan`, `str` or `@view` struct), which is passed as
    itself and keeps the regions it carries;
  * a `Copy` value that holds no `Cell` or `UnsafeCell`, which goes in registers when it is no larger
    than two pointers, except the receiver of a method whose result is a reference or view
    (`[LT-1]` rule 1).
  A reference into the own storage of a parameter passed as a copy ends with the call (`E3060`). The
  choice depends only on the declared signature, never on the body; it is an ABI decision and cannot
  be observed.
* `[BRW-9]` A reference in Safe code is never null, dangling or unaligned.

## VII.4 Regions and their inference

Ember has **no lifetime syntax**. Every reference and view carries a **region** — the set of program
points at which it must be valid — and the compiler infers every region.

* `[LT-5]` Regions of locals are inferred by non-lexical liveness (§XVIII.4). Diagnostics describe
  them in source terms: "the borrow of `x` on line 12 is still needed on line 19".
* `[LT-1]` *(changed in 0.9.9)* **Signature elision.** A **source parameter** (ODR-024) is:
  * a parameter whose type is a reference or view, in any mode; or
  * a borrowed or `mut` parameter whose type is not `Copy` even when each of its type parameters is
    taken to be `Copy`: an `Array[T]`, `String`, `Map`, `Box`, `RefCell` or arena, a struct or enum
    without `@derive(Copy)`, or a tuple or fixed array holding one of these.
  A source parameter of the second kind is the caller's place (`[FN-1]`, `[BRW-8]`): a result that
  borrows it may point anywhere in it, into its own storage or into storage it owns, and the call's
  loan is on the argument (`[BCK-1]`), shared for a borrowed parameter and exclusive for `mut`. A
  `Copy` parameter that is not a reference or view is not a source parameter (so neither is a plain
  `x: T`, nor a callable parameter, `[CLO-3]`), and nor is an `owned` parameter that is not a
  reference or view. Which parameters are sources depends only on the declared signature.
  For a function whose result is a reference or view:
  1. if it has a borrowed receiver (`self` or `mut self`), of any type, the result borrows from the
     receiver;
  2. else, if exactly one parameter is a source parameter, the result borrows from it;
  3. else, the result borrows from **all** source parameters together (the caller keeps all of them
     borrowed while the result lives).
  Rule 3 always type-checks; it may borrow more than the function needs. A returned reference or view
  that borrows any other parameter is rejected: `E3060` when that parameter is `owned` or is passed
  as a copy (`[BRW-8]`), and `E3062` otherwise.
* `[LT-1a]` *(changed in 0.9.9)* `@borrows(p, …)`, on its own line before the function, replaces the rule the compiler would
  apply: the result borrows from exactly the named parameters (`self` names the receiver). Returning
  a value that borrows from anything else is `E3062`. Each named parameter is a source parameter (`[LT-1]`)
  or a `mut` parameter; naming a borrowed `Copy` parameter, or an `owned` parameter that is not a
  reference or view, is `E2031`. It is needed only where rule 1 or 3 borrows more than the caller can
  afford, or to let the result borrow a `mut` parameter of a `Copy` type:

```ember
@borrows(haystack)
fn find_word(haystack: str, needle: str) -> Option[str]:
    i = haystack.find(needle)?
    return Some(haystack[i..i + needle.len()])
```

* `[LT-1b]` *(changed in 0.9.9)* The opt-in lint `L3014` reports rule 3 applying to more than one source parameter and
  names `@borrows` as the way to narrow it.
* `[LT-3]` String literals, `bytes` literals, `static` items and views of them have the `static`
  region, which outlives everything.
* `[LT-4]` *(changed in 0.9.9)* Arena allocations borrow the arena (`[ARN-1]`).
* `[LT-44]` *(new in 0.9.9)* A borrowed or `mut` `Arena`, `FixedArena` or `ScopedArena` parameter is
  a source parameter because it is not `Copy` (`[LT-1]`): a function whose only source parameter is
  one arena returns views that borrow it, with no annotation. `@borrows(arena)` is needed only when
  the function has other source parameters too.
* `[LT-6]` *(changed in 0.9.9)* Named lifetimes are not part of Ember and will not be added. Where a
  relationship between regions cannot be inferred, the diagnostic names the restructuring that
  expresses it: an owned result, an index instead of a reference, a view struct, or `@borrows`.
* `[LT-7]` *(changed in 0.9.9)* **Callable types.** Each call through a value or parameter of callable
  type `fn(P1, …, Pn) -> R` gets fresh regions for its source parameters (`[LT-1]`). `R` may borrow
  from them only as `[LT-1]` would allow for a function declared with that signature, and then only for
  the duration the caller keeps the arguments. So a callee may lend a callback views of its own
  locals, and a callback may return a view of an argument to its caller:

```ember
fn with_local(f: fn(Span[int]) -> int) -> int:
    tmp = [41]
    return f(tmp)

fn first_of(a: Span[int], b: Span[int], pick: fn(Span[int], Span[int]) -> Span[int]) -> int:
    return pick(a, b)[0]

fn main():
    println(with_local(fn(s) => s[0] + 1))
    xs = [1, 2]
    ys = [3]
    println(first_of(xs, ys, fn(a, b) => a))
```

### Views with several regions

A struct, enum or tuple holding views (a **view type**, `[TYP-34]`) may hold views of independent
sources. The compiler keeps one region per borrowed field — a **region vector** — and never forces
them to be equal.

```ember
struct Bodies:
    positions: MutSpan[f32]
    velocities: Span[f32]

fn integrate(mut b: Bodies, dt: f32):
    for i in 0..b.positions.len():
        b.positions[i] += b.velocities[i] * dt

fn main():
    pos: Array[f32] = [0.0, 1.0]
    vel: Array[f32] = [1.0, 1.0]
    b = Bodies(positions=pos, velocities=vel)
    integrate(b, 0.5)
    println(pos)
```

* `[LT-14]` A view type carries a compiler-internal region vector with one slot per borrowed field
  (fields proven to share a source may share a slot). It is never written in source (`[LT-6]`).
* `[LT-16]` Constructing a view value keeps each field's region; it never replaces them with their
  intersection.
* `[LT-17]` **Validity is conjunctive.** A view value is usable only while every region a used field
  needs is valid. A field whose source has ended cannot be used even if other fields are still valid.
* `[LT-18]` Every borrowed field is an ordinary loan under `[BRW-1]`–`[BRW-11]`; multiple regions add no
  new aliasing rule and weaken none.
* `[LT-20]` Moving, copying, destructuring, passing and returning a view preserves each field's region.
  Projecting one field carries only that field's region.
* `[LT-21]` Assigning a new view into a field recomputes that field's region; the old source is
  released on that path and the new one constrained. Merges of control flow keep every region that may
  reach a field.
* `[LT-22]` A function returning a view type gets, from its body, a summary of which result field
  borrows from which parameter (and which parameter fields each call accesses, `[LT-35]`). Callers use
  the summary; source never states it. A function whose result provenance cannot be established soundly
  is `E3065` (shape B14): return an owned value or separate views instead.
* `[LT-23]` `@borrows` on a function returning a multi-region view MUST NOT collapse it into fewer
  regions than its fields need (`E3065`).
* `[LT-24]` A region slot ends at the last use of its own field, not of the whole value.
* `[LT-25]` A view field may not borrow another field of the value that contains it (shape B5).
* `[LT-26]` A view extends no lifetime: constructing, copying or storing it retains nothing. (A view of
  a class object's field keeps the object alive only through `[RC-5]`.)
* `[LT-27]` Different regions never prove that two views do not overlap in memory; `noalias` comes
  only from `[DSJ-*]` and `[SIMD-3]`.
* `[LT-30]` Regions exist only at compile time: two values of one view type with different regions have
  one layout, one ABI, one symbol and one type identity, so region inference causes no
  code duplication.
* `[LT-34]` A view type whose fields all borrow from one source behaves exactly as a single-region
  view.
* `[LT-35]` For each function that can receive a view value, the compiler knows which fields the body
  may read, write, move, return or publish. A call requires only the regions of those fields. Where
  the target is not statically known — dynamic dispatch, a foreign function, an opaque callable — every
  field is assumed accessed and retained.
* `[LT-36]` Treating the view as a whole — passing it where every field may be used, comparing,
  hashing or formatting it, copying or moving it — requires every region to be valid.
* `[LT-38]` Moving one field out moves only that field's constraint; the remaining fields keep theirs.
* `[LT-39]` An operation that selects a field by a run-time value (reflection, a field descriptor)
  requires every region it could select (`E3065` if a region it needs has ended).
* `[LT-42]` A non-`owned` closure capturing a view records the regions of the fields it uses; an
  `owned fn` capturing a view requires every region to be `static` (`[TYP-15]`).
* `[LT-43]` Region tracking never lets a view survive a `yield` that `[CORO-6]` forbids.

## VII.5 Destruction

* `[DRP-1]` `fn drop(mut self)` runs exactly once per value, at the end of its life. Calling it
  explicitly is `E3070`; `mem.drop(x)` ends a value early.
* `[DRP-2]` Locals drop at the end of their block in reverse declaration order; fields after their
  owner's `drop`, in reverse declaration order; collection elements in index order; the active variant
  of an enum; tuple elements in reverse order.
* `[DRP-3]` Temporaries drop at the end of the statement that created them (`[EXP-4]`).
* `[DRP-4]` *(changed in 0.9.9)* A panic inside `drop` aborts the process (`[PAN-1]`). A `drop` SHOULD
  NOT block; `@noalloc` on a type's `drop` is honoured wherever that type is dropped.
* `[DRP-5]` A `drop` body may not move fields out of `self` (`[EXP-6]`); it uses `mem.take` on
  `Option` or `Default` fields.
* `[DRP-6]` Dropping a `Box[T]` drops the `T` and frees; dropping a class handle or `Shared` releases;
  dropping a view does nothing.
* `[DRP-7]` *(new in 0.9.9)* A value whose `drop` may read through a reference it holds must be dropped while that
  reference's region is valid. A generic type parameter is assumed to be read by `drop` unless the type
  has no `drop` of its own. Violations are `E3060` (shape B7).

## VII.6 Views over containers: `Span` and `MutSpan`

```ember
fn normalize(mut xs: MutSpan[f32]):
    total = xs.iter().sum()
    for x in xs.iter_mut():
        x /= total

fn main():
    buf: Array[f32] = [1, 2, 3]
    normalize(buf)
    left, right = buf.as_mut_span().split_at(1)
    left[0] = right[0]
    println(buf)
```

* `[SPN-1]` An `Array[T]`, a `[T; N]` or a `String` converts to `Span[T]`/`str` where one is expected,
  and to `MutSpan[T]` where one is expected (a `mut` parameter, or a `MutSpan`-typed local or field). **The conversion takes a borrow** of the source place with the
  ordinary rules — the source cannot be mutated, moved or dropped while the view lives — whether it is
  written implicitly, as `as_span()`/`as_mut_span()`, or produced by a call.
* `[SPN-2]` Indexing a view is bounds-checked; `get(i) -> Option[ref T]` does not panic;
  `unsafe: s.get_unchecked(i)` does not check.
* `[SPN-3]` `Span[T]` is `Copy`; `MutSpan[T]` is move-only and reborrowable (`s.reborrow()`, or
  implicitly at a `mut` site).
* `[SPN-4]` `iter()` on a `Span` or `MutSpan` yields `ref T`; `iter_mut()` on a `MutSpan` yields
  `ref mut T` and reborrows it until the iterator's last use. `chunks(n)` and `chunks_mut(n)` yield
  consecutive non-overlapping views of `n` elements (the last may be shorter); `n == 0` panics in every
  profile.
* `[SPN-5]` *(changed in 0.9.9)* `split_at(i)` on a `Span` returns two `Span`s; on a `MutSpan` it
  consumes (reborrows) the view and returns two disjoint `MutSpan`s. `i > len` panics. (There is no
  separate `split_at_mut`.)
* `[SPN-8]` `as_ptr()` and `as_mut_ptr()` return raw pointers; extracting one is safe, using one needs
  `unsafe`, and extraction extends no region.

## VII.7 What a newcomer sees

```ember
class Player:
    name: String
    health: float = 100.0

fn total_health(players: Array[Player]) -> float:      # borrows the array
    total = 0.0
    for p in players:
        total += p.health
    return total

fn heal_all(players: Array[Player]):                   # handles: the objects can change
    for p in players:
        p.health = 100.0

fn take(owned players: Array[Player]) -> int:          # consumes
    return players.len()

fn main():
    ps: Array[Player] = [Player("a"), Player("b")]
    println(total_health(ps))
    heal_all(ps)
    n = take(ps)
    println(n)
```

Using `ps` after `take(ps)` is `E3040`, labelled at the move. Nothing in this program names a mode at a
call, a lifetime, or a borrow.
---

# Part VIII — Classes and Reference Counting

Classes give Ember the object graphs Python programmers expect — shared objects, parent pointers,
observers — with deterministic destruction and no garbage collector.

```ember
class Node:
    name: String
    children: Array[Node] = []
    parent: Weak[Node] = Weak.empty()

    fn add(self, child: Node):
        child.parent = Weak(self)
        self.children.push(child)

fn main():
    root = Node("root")
    root.add(Node("leaf"))
    for c in root.children:
        match c.parent.upgrade():
            Some(p) => println(f"{c.name} -> {p.name}")
            None => println("orphan")
```

## VIII.1 Object representation

Every counted allocation — a class instance or a `Shared[T]` payload — is one heap block:

```text
offset  size  field
0       4     strong   u32   atomic iff the class is Sync or the block is a SyncShared
4       4     weak     u32   same atomicity
8       4     access   u32   dynamic exclusivity of a Shared payload (bit 31 writer, bits 0..30 reader count);
                             unused by class objects, whose fields carry their own (`[EXC-19]`)
12      4     flags    u32   bit 0 deinitialising, bit 1 pinned by foreign code
16      8     type     *const TypeInfo
24      …     base-class fields, then own fields (or the Shared payload), naturally aligned;
              each non-`Copy` field of a class that is not `@sync` is preceded by its u32 access word
```

* `[OBJ-1]` *(changed in 0.9.9)* The header is 24 bytes on 64-bit targets and is part of the runtime ABI
  (`[VER-4]`). Foreign code never reads it; it calls `ember_rt`. The per-field access words of
  `[EXC-19]` belong to the class's layout, not the header.
* `[OBJ-2]` A handle points at offset 0. An interface-typed handle is the same single pointer: the
  interface table is reached through the header's type information, so `Option[Handle]` is one pointer
  wide.
* `[OBJ-3]` `weak` starts at 1 on behalf of all strong handles. When `strong` reaches 0 the object is
  deinitialised (its `drop` chain and field drops run) and `weak` is decremented; the block is freed
  when `weak` reaches 0.
* `[OBJ-4]` Objects are allocated through the runtime allocator (`[RT-1]`).
* `[OBJ-5]` The runtime sets the deinitialising flag before the `drop` chain and checks `strong`
  after it and after the field drops; a nonzero count means the object was resurrected, which panics
  in every profile.

## VIII.2 Reference counting and its elision

* `[RC-1]` Copying a handle retains; dropping one releases; the release that reaches zero
  deinitialises. A strong count that would exceed its maximum panics (`[RT-7]`).
* `[RC-2]` **Guaranteed elisions.** No retain or release is emitted in the cases `[RC-2a]`–`[RC-2e]`;
  each is tested by counting retains in the emitted C.
* `[RC-2a]` Passing a handle to a borrowed parameter.
* `[RC-2b]` *(new in 0.9.9)* A handle read from a place and used only within one expression while that place is not
  written.
* `[RC-2c]` *(new in 0.9.9)* A retain immediately followed by a release of the same handle with no call or store between.
* `[RC-2d]` A handle stored into a field from a temporary (a move, not a retain).
* `[RC-2e]` Handles yielded by a `for` over a borrowed collection, unless the body stores, returns,
  consumes or `owned fn`-captures them.
* `[RC-3]` *(changed in 0.9.9)* Further elisions are allowed only when semantics are preserved (`[PHIL-5]`). An object may
  be deinitialised **earlier** than the last syntactic use of a handle whose value is provably unneeded
  and through which no loan is live (`[RC-5]`). A program that needs an object to live to a point binds
  it with `with h:` or calls `mem.keep_alive(h)`; `L3019` suggests this where a handle to a class with
  an observable `drop` is bound and never read again. `unsafe` code that keeps a raw pointer into an
  object MUST keep a handle to it alive the same way, because the object may be deinitialised after
  its last Safe use.
* `[RC-4]` *(changed in 0.9.9)* A non-`Sync` class's counts, and a `Shared`'s, use plain loads and
  stores; a `Sync` class's, and a `SyncShared`'s, use a relaxed increment and an acquire-release
  decrement (`[RT-8]`).
* `[RC-5]` A borrow whose place goes through a class handle or `Shared` (`ref h.f`, a `Span` of an
  object's `Array` field, a `RefCell` guard obtained from one, a view built from these) is also a
  shared loan of the handle: the object stays alive until the borrow's last use, and a handle whose
  storage ends first is `E3060`.
* `[RC-6]` `ember inspect` lists every retain and release that survives inside a loop, with the reason
  it could not be removed, because a surviving count operation blocks vectorisation.

## VIII.3 Exclusivity

Handles alias: two handles can reach one object. Ember enforces Swift's **law of exclusivity** for
the accesses that could otherwise invalidate memory, checking at run time what it cannot prove.

**Instantaneous accesses** — reading a field of a `Copy` type, or assigning a value to a field of a
`Copy` type (`h.x = 1.0`, `y = h.x`, `h.count += 1`) — complete in one step, cannot leave a dangling
reference, and are never checked.

**Long-term accesses** begin and end at run time and are checked. They are:

| Access | Kind | Duration |
|---|---|---|
| calling a method on a field (`h.items.push(x)`, `h.name.len()`) | write if the method takes `mut self`, else read | the call |
| calling an owned callable value stored in a field (`b.on_click(e)`, `[CLO-11]`) | write | the call |
| passing a field to a parameter (`f(h.items)`) | write for `mut`, read for borrowed, `E3012` for `owned` | the call; when the result borrows that parameter (`[LT-1]`), until the result's loan dies (`[EXC-18]`) |
| `ref h.f` / `ref mut h.f`, or a view of a field | read / write | until the loan dies (`[EXC-18]`) |
| iterating a field (`for x in h.items`) | read (write for `iter_mut`) | the loop |
| **assigning a field of a non-`Copy` type** (`h.items = []`, `h.name = other`) | write | the store and the drop of the old value |
| calling a `mut self` class method (`[CLS-7]`) | write, every field | the call |

Each access is to one field of the class's own (`h.inner.items` is an access to `inner`), and only
accesses to the same field conflict (`[EXC-19]`).

* `[EXC-1]` *(changed in 0.9.9)* Beginning a **write** access to a field while any access to the same
  field is active panics: `exclusivity violation: write access to Node.children while a read access
  to Node.children is active`, in every profile. There is no setting that removes this check
  (`[PHIL-13]`).
* `[EXC-2]` *(changed in 0.9.9)* Beginning a **read** access to a field while a write access to it is
  active panics likewise.
* `[EXC-19]` *(new in 0.9.9)* **Access state is per field.** Each non-`Copy` field of a class that is
  not `@sync` has its own access word (bit 31 writer, bits 0..30 reader count), stored in front of the
  field. A long-term access to a field checks and updates that word only, so reading one field while
  writing another never conflicts: `for c in self.children: self.log.push(c.name)` is accepted. A
  whole-object write (a `mut self` method, `[EXC-15]`) checks every field word of the object's
  **dynamic** class at entry — for an `open` class, through the list of access-word offsets in its type
  information, so a derived override reached from a base method is covered too — marks each as
  written, and clears them at return; a `mut self` call on `self` inside it is a reborrow (`[EXC-5]`)
  and marks nothing again. The cost is one word per non-`Copy` field, one check per such field on
  entry to a `mut self` method, and nothing per access. `Copy` fields have no word
  (`[EXC-17]`), and the fields of a `@sync` class need none (`[THR-1]`).
* `[EXC-16]` *(new in 0.9.9)* Assigning a value to a class field whose type is not `Copy` is a write access (table
  above): it conflicts with, for example, a `for` loop over the same field, which would otherwise read
  a buffer freed by the assignment.
* `[EXC-17]` *(new in 0.9.9)* Instantaneous writes are not checked against long-term accesses, so a
  `ref` or view of a `Copy` field (including a fixed-array field) observes writes made through other
  handles while it is live, as a C pointer would. It never dangles: a `Copy` field's storage lasts as
  long as the object, which the loan keeps alive (`[RC-5]`). For the same reason such a view gets no
  alias fact the other handles could break (`[SIMD-3]`).
* `[EXC-18]` *(new in 0.9.9)* The dynamic access that a borrow through a class handle begins — a
  `ref h.f`, a view of a field, a guard obtained from one — ends where the borrow's loan dies
  (`[BCK-2]`), in whichever function that is. A view of a field returned from a method carries its
  access to the caller and ends at the caller's last use of the view, on every path.
* `[EXC-3]` The compiler MAY remove a check only when it proves no conflicting access can occur: every
  access to the object in the interval goes through one handle local that is not reassigned, their
  intervals are ordered, and either the interval contains no call, no virtual or interface dispatch and
  no call through a callable value, or escape analysis proves that local is the only handle to the
  object. Conflicting accesses through one local that the compiler can see are rejected at compile
  time instead (`E3080`, shape X1).
* `[EXC-3a]` Every removed check is recorded, with the condition that justified it, and reported by
  `ember inspect --safety --elided-only`.
* `[EXC-4]` A `let` field is subject to the same rules: `let` fixes the binding, not the value
  (`[CLS-9a]`).
* `[EXC-5]` Accesses nested inside the same `mut self` method are reborrows and are not checked again.
* `[EXC-6]` *(changed in 0.9.9)* A panic from `[EXC-1]`/`[EXC-2]` names both the offending access and
  the active one it conflicts with (file, line, column, field). The `debug` runtime keeps a per-thread
  stack of active accesses for this; other profiles always name the offending access and its field,
  and name the active one where they can without any per-access cost.
* `[EXC-7]` *(changed in 0.9.9)* The opt-in lint `L3013` reports a long-term access held across a
  virtual or `dyn` call within one `open` class hierarchy, or across a call into C++ that may call back
  into Ember (`[FFI-39d]`).
* `[EXC-8]` When a loop makes repeated long-term accesses to one object whose identity, access kind and
  conflict set are loop-invariant, the compiler performs one check in the loop preheader and holds the
  access for the whole loop.
* `[EXC-9]` `[EXC-8]` applies only when the receiver's identity is loop-invariant, the access does not
  escape the loop, nothing in the loop can replace or publish the receiver, no call the compiler cannot
  see through can begin a conflicting access, and the hoisted access starts and ends where the original
  accesses did. If any condition is unknown, the per-access checks stay.
* `[EXC-10]` An inner loop reuses an outer loop's hoisted access when its accesses are a subset of it.
* `[EXC-11]` A hoisted access is invisible to programs: it cannot be named, stored or observed.
* `[EXC-12]` `ember inspect --safety` reports each check as `STATIC`, `DYNAMIC_PER_ACCESS` or
  `DYNAMIC_HOISTED_LOOP`, naming the loop and the proof for a hoisted one.
* `[EXC-15]` *(new in 0.9.9)* A `mut self` class method holds a write access to every field of the
  object (`[EXC-19]`) from entry to return; accesses to `self`'s fields inside it are covered by it and
  not checked individually.
* `[EXC-14]` *(changed in 0.9.9)* The 0.9.8 `exclusivity = "unchecked"` setting is removed. Code that
  cannot afford a check per access uses value types, `mut self` methods (`[EXC-15]`), or a query
  yielding `ref mut` (Part XII).

## VIII.4 Inheritance and dispatch

* `[DSP-1]` A call is dispatched statically when the receiver's static type is a final class or the
  method is not `virtual`.
* `[DSP-2]` A virtual call loads its slot from the object's type table; slots are assigned in
  declaration order, base first, and an `override` reuses the base's slot.
* `[DSP-3]` A call through an interface-typed handle finds the interface's table through the type
  information and caches the lookup for repeated calls on one handle.
* `[DSP-4]` `h as? D` walks the base chain; `a is b` compares addresses.
* `[DSP-5]` With the whole program visible, a virtual call with exactly one reachable implementation
  may become a direct call; `--emit-optimization-report` lists each.

## VIII.5 Weak handles and cycles

* `[WK-1]` A cycle of strong handles among class instances and `Shared` payloads is never collected;
  it leaks. This is a documented property. `Weak` breaks cycles; the compiler warns about the cycles it
  can see (`[WK-6]`); the debug runtime reports the ones it finds (`[WK-15]`).
* `[WK-11]` *(changed in 0.9.9)* `Weak(h)` creates a weak handle to a class object or a `Shared` or
  `SyncShared` payload without retaining it strongly; `Weak[O].empty()` (or `Weak.empty()` where the
  type is known) creates one that points nowhere. `Weak[O]` is `Copy`: copying increments the weak
  count, dropping decrements it.
* `[WK-2]` An object is deinitialised when its strong count reaches zero, whatever weak handles remain;
  those weak handles then fail to upgrade.
* `[WK-3]` `upgrade` returns `None` while the object's deinitialising flag is set, so a `drop` body
  cannot resurrect its object through a weak handle.
* `[WK-12]` *(changed in 0.9.9)* `w.upgrade() -> Option[O]` returns a retained strong handle while the
  object is alive and not deinitialising, and `None` otherwise. On a `@sync` object (or a
  `SyncShared`) it is a compare-exchange loop that fails once the strong count has reached zero, so an
  object another thread is releasing is never resurrected.
* `[WK-13]` A `Weak[Shared[T]]` refers to the same block as its `Shared[T]`.
* `[WK-14]` Ember's `Weak` and `Shared` never convert to or from C++'s `std::weak_ptr`/`std::shared_ptr`
  (`[SEL-2]`).
* `[WK-5]` The compiler builds a graph of strong ownership among class fields (class handles,
  `Shared`, and collections of them, after generic substitution); `Weak`, raw pointers, views and
  foreign handles are not strong edges; a foreign edge whose ownership is unknown is marked unknown.
* `[WK-6]` A cycle of strong edges in that graph is warning `L3001` at the field that closes the
  shortest cycle, naming the whole cycle and the field to make `Weak`, with a machine-applicable fix-it
  where the replacement changes no declared ownership contract. It never changes whether a program is
  accepted.
* `[WK-7]` Cycle analysis is conservative: a possible cycle suffices for `L3001`, it is never reported
  as certain without run-time evidence, generic types are analysed after substitution, and an opaque
  foreign edge is `unknown`, never assumed weak or strong.
* `[WK-8]` The run-time report (`[WK-15]`) lists each leaked strongly connected component with its
  class and field edges, whether static analysis predicted it, and the edge to weaken.
* `[WK-4]` The leak report names, for each leaked object on a cycle, the shortest strong cycle through
  it as a path of `Type.field` edges and the edge to weaken (`L3017`).
* `[WK-15]` *(new in 0.9.9)* `ember run` and `ember test` in the `debug` profile report leaked objects
  and their cycles when the program exits, by default; `--no-leak-check` turns the report off.
* `[WK-9]` `ember explain --cycle <path> <Class[.field]>` explains one cycle-capable class or field;
  `ember inspect --cycle <path>` prints the whole graph. A cycle through a generic container is shown with
  its instantiated types, and where no static cycle can be established the command says the edge is
  only dynamically cycle-capable.
* `[CLI-18]` For both commands `<path>` is a package directory (its manifest and imports) or a single
  `.em` file (`[CLI-4]`), and both analyse exactly the same ownership graph (`[WK-5]`), classifying
  each edge strong, weak or unknown and showing the shortest static cycle when there is one.

## VIII.6 Stack promotion

* `[OPT-1]` When escape analysis proves that no handle to an object outlives the function that
  created it, the compiler MAY place the object in the function's frame with the same header and run
  its `drop` at scope end. This cannot be observed.
---

# Part IX — Memory Facilities

## IX.0 Choosing a storage mechanism

The declared type decides storage and lifetime; the compiler never picks between them. This table is
the choice, in the order to try (`[SEL-1]`): a design that reaches for a `class` where a `struct` does
pays a heap allocation, a header, reference counting and exclusivity checks for nothing.

| Mechanism | Ownership | Aliasing | Destroyed | Reach for it when |
|---|---|---|---|---|
| `struct`, `enum`, tuple | unique, moved | borrow-checked | end of scope | **the default**: data with no identity |
| `Array`, `Map`, `Set`, `String` | unique, moved | borrow-checked | end of scope | collections |
| `Box[T]` | unique, heap | borrow-checked | on drop | one owner, but it must be on the heap: recursion, a large value, `dyn` |
| `class` | shared, counted | dynamic exclusivity | at count 0 | the thing has identity and many places refer to it |
| `Shared[T]` | shared, counted | borrow-checked + dynamic exclusivity | at count 0 | shared ownership of a plain value without declaring a class, on one thread |
| `SyncShared[T]` | shared, atomically counted | read-only; mutation through `T`'s locks or atomics | at count 0 | the same across threads (`[HEAP-10]`) |
| `Weak[O]` | none | — | never | back-pointers; observing without keeping alive |
| `Arena`, `FixedArena` | region | borrow-checked | all at once | many values with one lifetime: a frame, a parse |
| `Pool[T]` + `Handle[T]` | the pool | handles are plain `Copy` values | the pool decides | resources the program recycles: entities, GPU objects |
| `Cell[T]`, `RefCell[T]` | the owner's | interior | with the owner | mutating a value you hold only a shared borrow of |
| `Mutex[T]`, `RwLock[T]`, `Atomic[T]` | the owner's | synchronised | with the owner | mutation shared across threads |

* `[SEL-1]` The order above is the order to try. `ember inspect --alloc` reports the mechanism each
  declaration uses.
* `[SEL-2]` `Shared`/`Weak` and C++'s `std::shared_ptr`/`std::weak_ptr` (`CppShared`, `CppWeak`)
  never convert into each other: they use different counts, and conflating them frees twice (`E5065`).

## IX.1 The heap types

* `[HEAP-1]` Every heap type allocates through `ember_alloc`/`ember_realloc`/`ember_free` and carries
  the `Alloc` effect where it allocates.
* `[HEAP-2]` Growable buffers double, from a minimum of four elements; `shrink_to_fit` is explicit.
* `[HEAP-8]` *(new in 0.9.9)* Capacity arithmetic is checked: a requested capacity whose byte size
  overflows `usize`, or exceeds the allocator's limit, panics with `capacity overflow` before any
  allocation. No length or capacity computation in the runtime wraps.
* `[HEAP-9]` *(new in 0.9.9)* Heap storage for elements of type `T` is aligned to at least
  `align_of[T]()`, whatever that alignment is.
* `Box[T]`: one owner; `Box(v)`, auto-deref, `b.get()`; `Box[dyn I]` holds any implementer. Move-only.
* `[HEAP-3]` `Shared(value) -> Shared[T]` allocates one counted block holding `value`.
* `[HEAP-4]` *(changed in 0.9.9)* `s.get() -> ref T` borrows the payload: it begins a checked read
  access (`[EXC-2]`) that ends where the loan dies (`[EXC-18]`), and the loan keeps the block alive
  (`[RC-5]`).
* `[HEAP-5]` *(changed in 0.9.9)* `s.get_mut() -> ref mut T` begins a checked write access
  (`[EXC-1]`) for the loan's life; it does not require the `Shared` to be unique, and conflicts with
  any live `get` or `get_mut` through any copy.
* `[HEAP-6]` `Shared[T]` is `Copy`: copying retains, dropping releases.
* `[HEAP-7]` *(changed in 0.9.9)* `Weak[O]` exists for `O` a class handle, a `Shared[T]` or a
  `SyncShared[T]` (`[WK-11]`).
* `[HEAP-10]` *(new in 0.9.9)* `Shared[T]` is never `Send` or `Sync`: its counts and access state are
  plain memory. Shared ownership across threads uses `std.sync.SyncShared[T]` (`T: Sync`), whose counts
  are atomic and which offers `get() -> ref T` only; mutation goes through `T`'s own synchronisation,
  as for a `@sync` class (`[THR-1]`).

## IX.2 Arenas

```ember
struct Cmd:
    id: int
    cost: f32

fn build(frame: Arena, n: int) -> MutSpan[Cmd]:
    cmds = frame.alloc_array[Cmd](n)        # borrows `frame`: no annotation needed ([LT-44])
    for i in 0..n:
        cmds[i].id = i
    return cmds

fn main():
    frame = Arena.with_capacity(64 * 1024)
    cmds = build(frame, 8)
    println(cmds.len())
    frame.reset()                           # legal: `cmds` is no longer used
```

* `[ARN-1]` `Arena` is a move-only struct. Its `alloc*` methods take `self` (a shared borrow), so many
  allocations are outstanding at once, and return views that borrow the arena. `reset()` and dropping
  take `mut self`, so no view survives them (`[BRW-1]`).
* `[ARN-2]` Values in an arena are never dropped individually. Allocating a type that needs drop is
  `E3090`, unless through `alloc_nodrop`, which acknowledges that its `drop` will never run.
* `[ARN-3]` `alloc(v) -> ref mut T` moves `v` in. `alloc_array[T](n) -> MutSpan[T]` requires `T` not to
  need drop and initialises every element: zeroed if `T: Zeroable`, else with `T.default()`, else
  `E2040`. `alloc_uninit[T](n) -> MutSpan[MaybeUninit[T]]` initialises nothing.
* `[ARN-4]` *(changed in 0.9.9)* A growing `Arena` carries `Alloc` on its growth path. `FixedArena`
  (made by `Arena.fixed(buffer: MutSpan[u8])` or `FixedArena.with_capacity(n)` at start-up) never
  grows, carries no `Alloc` on allocation, and panics on exhaustion (`try_alloc` returns `Option`).
  `FixedArena` is the arena for `@noalloc` code.
* `[ARN-5]` `ArenaArray[T]` and `ArenaMap[K, V]` are fixed-capacity containers whose storage is taken
  from an arena once, at construction; they never grow or move, so views of their elements stay valid
  while the element and the arena do. Adding past capacity returns `Err(CapacityError.Full)` and changes
  nothing. Their elements, keys and values must not need drop. They live in `std.collections` and are
  not in the prelude.
* `[ARN-5c]` `ArenaArray[T]` provides `len`, `capacity`, `is_empty`, `get(i) -> Option[ref T]`,
  `get_mut`, `push(v) -> Result[void, CapacityError]`, `insert(i, v) -> Result[void, CapacityError]`,
  `remove(i) -> Option[T]`, `clear`, `iter` and `iter_mut` (in index order); indices follow the `Array`
  rules.
* `[ARN-5d]` `ArenaMap[K, V]` (with `K: Eq + Hash`) provides `len`, `capacity`, `is_empty`, `get`,
  `get_mut`, `insert(k, v) -> Result[Option[V], CapacityError]` (replacing returns the old value),
  `remove`, `contains_key`, `clear` and `iter`, which follows insertion order like `Map` (`[STD-11]`).
* `[ARN-5g]` No operation of either container touches the arena after construction: nothing grows,
  moves the arena's cursor or allocates elsewhere.
* `[ARN-6]` `arena.scope() -> ScopedArena` takes a mutable borrow of the arena for the scope's life;
  `with s = frame.scope():` allocates from `s` and releases everything at the block's end, last in first
  out. Using the parent while a scope is live is `E3096`, whose help is to allocate from the scope.
  Scopes nest: `s.scope()` opens one inside another.
* `[ARN-7]` No operation lowers an arena's allocation pointer or reuses its bytes while a view into it
  is live; every such operation takes `mut self`.
* `[ARN-8]` `MaybeUninit[T]` has the size and alignment of `T` and does not claim to hold a valid `T`;
  dropping it never drops a `T`. `MaybeUninit[T].uninit()` is safe; `write(mut self, owned v) -> ref
  mut T` stores without dropping previous bytes; `unsafe assume_init(owned self) -> T` asserts
  initialisation. `MutSpan[MaybeUninit[T]]` has `write_at(i, v)` and `unsafe assume_init(owned self)
  -> MutSpan[T]`. There is no other conversion.
* `[ARN-10]` A panic inside `T.default()` during `alloc_array` aborts (`[PAN-1]`); no partially
  initialised span becomes reachable.
* `[ARN-11]` `Zeroable` is an `unsafe` marker: all-zero bytes are a valid `T`. The compiler derives it
  only when it can prove it field by field; a `ref`, a range excluding zero, or an enum whose zero
  discriminant is invalid is never `Zeroable`.

## IX.3 Allocators

* `[ALC-1]` `Array[T, A: Allocator = Global]`, `Map`, `Set` and `Box` accept an allocator type
  parameter; an allocator instance is passed at construction (`Array.new_in(a)`). In `Map[K, V, H,
  A]` and `Set[T, H, A]` it follows the hasher (`[STD-11]`).
* `[ALC-2]` Implementing `Allocator` is `unsafe`: the implementer promises valid, aligned, unaliased
  memory.
* `[ALC-3]` *(changed in 0.9.9)* A binary package may replace the global allocator with
  `[build] global_allocator = "module.NAME"` naming a `static` whose type implements `Allocator`; an
  embedding host may pass allocation functions to `ember_rt_init` (`[FFI-27]`).
* `[ALC-4]` Allocation failure in a standard container panics (`out of memory`). `try_reserve` and
  `try_push` report it as a value.

## IX.4 Raw pointers and `unsafe`

```ember
# SAFETY: the caller guarantees `p` points to four readable bytes
unsafe fn read_u32_le(p: *u8) -> u32:
    return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4:
        return None
    # SAFETY: the length check above guarantees four readable bytes
    unsafe:
        return Some(read_u32_le(data.as_ptr()))
```

* `[UNS-1]` An `unsafe` context is required to: dereference, read or write through `*T`/`*mut T`;
  offset a pointer; cast between pointers or pointers and integers; call an `unsafe fn`; call an
  `extern` function without a verified contract (Part XVI); access a `static mut`; `transmute`; call
  `get_unchecked` or `assume_init`; implement an `unsafe` interface.
* `[UNS-2]` `unsafe` permits exactly those operations. It does not turn off borrow checking, bounds
  checks on safe types, overflow checks or type checking.
* `[UNS-3]` `L3010` reports an `unsafe` block containing statements that need no `unsafe`.
* `[UNS-4]` Unsafe code MUST uphold what safe code assumes: every reference is non-null, aligned, points
  to a live initialised value of its type and is not aliased by a live `ref mut`; every view's length is
  within its allocation; no two live views overlap unless both are shared; every class handle points to
  a live object with a correct header; every `str` is UTF-8; every value is valid for its type (`bool`,
  `char`, range types, enums); `Send` and `Sync` are respected. A raw pointer derived from a reference
  is not used after that reference's region ends.
* `[UNS-5]` `std.mem` provides `Volatile[*T]` (volatile reads and writes), `transmute[A, B]`,
  `copy_nonoverlapping`, `zeroed[T]()` (requires
  `T: Zeroable`) and `MaybeUninit`.
* `[UNS-6]` Inline assembly is `unsafe asm("…", …)` on toolchains that support it and `E5090`
  otherwise.
* `[UNS-7]` Every `pub unsafe fn` carries `@safety("…")` stating the caller's obligation; without it,
  `L3015`. The standard library carries one on every unsafe function it exports.
* `[UNS-8]` *(changed in 0.9.9)* Every `unsafe:` block and `unsafe fn` carries a safety note
  (`[LEX-23]`): `# SAFETY: …` or `# SAFETY(category): …` on the line before it or at the end of its first
  line. Without one, `W3012`. `ember tcb` lists every block with its note, its category and the
  obligations of the unsafe functions it calls.
* `[UNS-10]` `UnsafeCell[T]` (in `std.mem`) is the primitive beneath every interior-mutability type:
  `UnsafeCell(v)`, `unsafe get(self) -> *mut T`, `into_inner(owned self) -> T`. It hands out no safe
  reference, checks nothing, is never `Copy` and never `Sync` (it is `Send` when `T` is), and is refused
  in `@static_safe` code (`E3105`). Diagnostics never suggest it.
* `[UNS-10a]` `UnsafeCell` suspends no rule globally: borrow, region, type and bounds checking all still
  apply around it. Unsafe code may break the aliasing rules inside an abstraction built on it, and must
  never let a conflicting reference escape into Safe Ember.

## IX.5 Layout

* `[LAY-2]` *(new in 0.9.9)* Layout attributes change layout exactly as stated, in every profile and on
  every backend, and are never ignored (`[ATT-6]`):
  * `@layout(c)` — the target C ABI layout (the default for every struct, `[TYP-11]`);
  * `@packed` — no padding; unaligned fields are read and written by byte copies; taking a reference
    to one is `E2170`;
  * `@align(N)` — raises the type's alignment to `N` (a power of two up to 4096); `size_of` becomes a
    multiple of `N`, and every place and heap buffer holding the type honours it (`[HEAP-9]`);
  * `@repr(u8|u16|u32|u64|i8|i16|i32|i64)` — an enum's discriminant type.
* `size_of[T]()`, `align_of[T]()` and `offset_of[T](field)` are compile-time functions.

## IX.6 Handles and pools

```ember
from std.collections import Pool, Handle

struct Texture:
    width: int
    height: int

fn main():
    pool = Pool[Texture]()
    h = pool.insert(Texture(width=64, height=64))
    println(pool.get(h).is_some())
    pool.remove(h)
    println(pool.get(h).is_some())        # stale handle: None
```

* `[HND-1]` A `Handle[T]` is a plain `Copy` value (index and generation). Every lookup compares the
  generation, in every profile, and a stale handle yields `None` (or panics in `pool[h]`).
  `unsafe: pool.get_unchecked(h)` skips the comparison.
* `[HND-2]` *(changed in 0.9.9)* `Handle[T]` is a `u64`: 32 bits of index and 32 of generation.
  `Pool[T, Bits]` may narrow it (`Pool[T, Split20]` gives a 20-bit index and a 12-bit generation in a
  `u32`).
* `[HND-3]` *(new in 0.9.9)* A slot whose generation is exhausted is retired and never reused, so an old handle can never
  become valid again. When every slot is retired or live, `insert` panics and `try_insert` returns
  `Err(CapacityError.Full)`.

## IX.7 Interior mutability: `Cell` and `RefCell`

```ember
struct Sprite:
    frame: Cell[int]

fn advance(s: Sprite):                 # `s` is borrowed, not `mut`
    s.frame.set(s.frame.get() + 1)

struct Scene:
    entities: RefCell[Array[int]]

fn add(s: Scene, e: int):
    with list = s.entities.borrow_mut():
        list.push(e)
```

* `[CELL-1]` `Cell[T]` holds a `T` that can be replaced through a shared borrow: `Cell(v)`, `set(v)`,
  `replace(v) -> T`, `take() -> T` (for `T: Default`), `into_inner()`, `get() -> T` for `T: Copy`, and
  `update(f)` for `T: Copy` or `T: Default`. `set` and `replace` store the new value before dropping the
  old one, since the old value's `drop` may read the cell.
* `[CELL-2]` `Cell` never hands out a reference to its contents, so it needs no run-time check; `get` is
  a load and `set` a store.
* `[CELL-3]` `Cell[T]` is not `Sync`; it is `Send` if `T` is.
* `[CELL-4]` `Cell[T]` is `Copy` when `T` is.
* `[CELL-5]` `RefCell[T]` keeps a one-word borrow counter beside `T`. `borrow() -> Ref[T]` succeeds
  unless a mutable borrow is active; `borrow_mut() -> RefMut[T]` succeeds unless any borrow is active;
  failure panics, naming the location of the conflicting borrow.
* `[CELL-6]` `try_borrow()` and `try_borrow_mut()` return `None` on conflict, in every profile.
* `[CELL-7]` `Ref[T]` and `RefMut[T]` are views of the cell; dropping one ends its borrow. `L3011` warns
  when a guard is live across a call that can reach the same cell — not across calls that provably
  cannot (a method on the guard itself, `print`).
* `[CELL-8]` `RefCell[T]` is not `Sync`; `Mutex[T]` and `RwLock[T]` are its thread-safe counterparts,
  with the same `with g = m.lock():` shape.
* `[CELL-9]` The `RefCell` check exists in every profile.
* `[CELL-10]` A borrow diagnostic suggests `RefCell` only after the structural fixes, and only when the
  conflicting accesses are provably not simultaneous (`[DIA-9]`).
* `[CELL-12]` `RefCell[T]` is never `Copy`.

## IX.8 Establishing disjointness

When two views come from unrelated sources that the programmer knows do not overlap, and the
structural fixes (`split_at`, `chunks_mut`, SoA columns) do not apply:

```ember
from std.mem import assert_disjoint

fn blend(mut dst: MutSpan[f32], src: Span[f32]):
    match assert_disjoint(dst, src):
        Ok((d, s)):
            for i in 0..d.len():
                d[i] = d[i] * 0.5 + s[i] * 0.5
        Err((d, s)):
            for i in 0..d.len():
                d[i] = (d[i] + s[i]) * 0.5
```

* `[DSJ-1]` *(changed in 0.9.9)* `assert_disjoint(a, b) -> Result[(A, B), (A, B)]` compares the two
  views' address ranges. It consumes (reborrows) both and returns them, carrying a proven disjointness
  fact in `Ok`, and unchanged in `Err`, so the overlapping path can still use them. The cost is two
  comparisons.
* `[DSJ-2]` The fact belongs to the returned values, not to a program point; reassigning them drops it.
* `[DSJ-3]` The borrow checker treats the returned views as non-overlapping, and the backend marks them
  `restrict`/`noalias` (`[SIMD-3]`).
* `[DSJ-4]` It applies to `Span`, `MutSpan`, `SoA` columns and arena views; two single-object
  references are `E3095`.
* `[DSJ-5]` `assert_disjoint_or_panic(a, b) -> (A, B)` panics on overlap. Both forms record a
  `RuntimeCheck(Aliasing)` site whose reason is `establishes_static_fact` (`[EFF-11]`), unless the
  compiler already knows the ranges are disjoint, in which case no comparison is emitted.
* `[DSJ-6]` `assert_disjoint` is usable in `@noalloc`, `@nosync` and `@static_safe` code: it
  establishes a fact rather than deferring a check.
* `[DSJ-7]` `unsafe assume_disjoint(a, b) -> (A, B)` asserts without checking; violating it is
  undefined behaviour.
* `[DSJ-8]` There is no form that checks in one profile and assumes in another (`[PHIL-13]`).
* `[DSJ-9]` `assert_disjoint_all(v1, …, vn)` handles 2 to 8 views pairwise.
---

# Part X — Effects, Contracts, Determinism and Cost

## X.1 Effects

The compiler infers, for every function, an **effect set** drawn from:

| Effect | Introduced by |
|---|---|
| `Alloc` | a heap allocation: constructing an allocating type, container growth, an allocating literal or f-string, a boxed callable value, arena growth |
| `Sync` | an atomic read-modify-write stronger than relaxed, a lock, a channel operation, thread spawn or join, retaining or releasing a `Sync` handle |
| `Lock` | acquiring a `Mutex`/`RwLock` (including `try_lock`) |
| `Block` | an operation that may wait: a blocking lock, `join`, `sleep`, a blocking receive or read |
| `Io` | file, socket and console operations |
| `Panic(Explicit)` | `panic`, `assert*`, `unwrap`, `expect`, `todo`, `unreachable`, `as!`, allocation failure |
| `RuntimeCheck(k)` | a safety check the compiler emitted rather than proved away, for `k ∈ {Aliasing, Bounds, Stale, Arithmetic}` |
| `Unsafe` | the body contains `unsafe` or the function is `unsafe fn` |
| `FFI` | a call to an `extern` function |
| `Nondet` | a result that may differ between runs or machines (`[DET-2]`) |

* `[EFF-1]` Effects are computed from each function's body and its callees', over the call graph, with
  recursive groups taking the union of their members.
* `[EFF-2]` *(changed in 0.9.9)* A call through a callable **parameter** (a generic bound, `[CLO-3]`)
  contributes the effects of the function actually passed, per instantiation. A call through an owned
  callable value, a `dyn` interface or a function pointer contributes the effects declared on its type
  (`@noalloc fn(A) -> R`), and all effects if none are declared. An interface method may declare
  contracts; its implementations MUST satisfy them (`E4010`).
* `[EFF-3]` An `extern` function carries the effects its contract declares (`@ffi(effects=[…])`), `FFI`
  at least.
* `[EFF-4]` Effects are part of a function's interface for incremental builds: a change to a callee's
  effect set re-checks its callers' contracts.
* `[EFF-9]` *(changed in 0.9.9)* `RuntimeCheck(k)` enters a function's effect set when a check of kind
  `k` survives the compiler's profile-independent proofs in its body. `Arithmetic` covers integer
  overflow, division by zero, shift amounts and negative integer exponents; `Bounds` covers indices,
  slices and chunk sizes; `Aliasing` covers class exclusivity and `RefCell`; `Stale` covers handle
  generations and `Weak.upgrade`.
* `[EFF-10]` The effect set records only which kinds occur. Per-site detail — every emitted check with
  its kind, location, mechanism and reason, and every removed check with the proof that removed it — is
  written to the safety side table that `ember inspect --safety` reads.
* `[EFF-11]` Every emitted check carries one reason, and diagnostics use it:

  | Reason | Meaning | What the programmer can do |
  |---|---|---|
  | `not_provable_in_principle` | the property depends on run-time data | nothing; the check is permanent |
  | `not_proven_by_analysis` | the property may hold but was not proved | restructure per the hint, or report the gap |
  | `requested_by_type` | the programmer chose a checked type (`RefCell`) | change the type to change the cost |
  | `inherent_to_mechanism` | the check is what the mechanism is (a generation compare) | use another mechanism |
  | `establishes_static_fact` | the check proves a property once and returns proof-carrying values (`[DSJ-1]`) | nothing |

* `[EFF-18]` Effects are independent: a blocking `Mutex.lock` has `Sync + Lock + Block`; `try_lock` has
  `Sync + Lock`; a blocking read has `Io + Block`.

## X.2 Contracts

A contract attribute states an effect the function must not have.

| Contract | Forbids | Code |
|---|---|---|
| `@noalloc` | `Alloc` | `E4001` |
| `@nosync` | `Sync` | `E4002` |
| `@noblock` | `Block` | `E4003` |
| `@noio` | `Io` | `E4041` |
| `@nolock` | `Lock` | `E4042` |
| `@nopanic(explicit)` | `Panic(Explicit)` | `E4040` |
| `@static_safe` | `RuntimeCheck(Aliasing)`, except from sites whose reason is `establishes_static_fact` | `E4030` |
| `@deterministic` | `Nondet` (Part X.3) | `E4070` |
| `@realtime` | the manifest's set, by default `@noalloc @nolock @noblock @nopanic(explicit)` | per contract |

* `[EFF-5]` Each contract in the table forbids its effect in the function's effect set; a violation is
  the listed error.
* `[EFF-6]` A contract is checked over the whole reachable call graph — callees, drop glue, default
  arguments, operator and interface implementations reached statically — and a violation names the full
  chain to the offending operation:

  ```text
  error[E4001]: @noalloc function `cull` reaches an allocation
    cull -> collect_visible -> Array.push -> ember_alloc
  ```

* `[EFF-12]` `@static_safe` permits `RuntimeCheck(Bounds)`, `(Arithmetic)` and `(Stale)`; it forbids
  only dynamic aliasing checks, because those are the checks a data-oriented inner loop can always be
  restructured to avoid.
* `[EFF-17]` `@nopanic(explicit)` is spelled with its argument; bare `@nopanic` is `E0104` with a fix-it,
  so that no reader takes it to mean "cannot abort" (`[EFF-16]`).
* `[EFF-20]` `@noio` forbids `Io`, including console output; `println` in a `@noio` function is `E4041`.
* `[EFF-21]` `@nolock` forbids `Lock`, including `try_lock`, which never blocks but still takes a lock.
* `[EFF-6a]` A `@static_safe` function may not take a parameter whose type forces a dynamic check at
  the call site (a `RefCell` by value, a `Ref`/`RefMut` guard).
* `[EFF-7]` `unsafe: @assume_noalloc(expr)` overrides the analysis for one call the compiler cannot see
  into.
* `[EFF-8]` An `override` inherits its base method's contracts.
* `[EFF-13]` A long-term access through a class handle loaded from memory is dynamically checked
  (`[EXC-3]`), so most class-heavy code cannot be `@static_safe`; the diagnostic says why and names the
  fixes (hoist the handle into a local; use value types or a query yielding `ref mut`).
* `[EFF-14]` Contracts compose; the usual inner-loop set is `@static_safe @noalloc @nosync
  @nopanic(explicit)`, one per line.
* `[EFF-15]` *(changed in 0.9.9)* Effects and contract verdicts are computed once, after the
  profile-independent proofs, and are identical in every profile (`[PHIL-13]`). The proofs that count
  are exactly those this document specifies — `[EXC-3]`, `[EXC-8]`, `[RC-2]`, `[OPT-2]`, `[RNG-4]`,
  `[SIMD-7]` and `[DSJ-1]` — and a verdict is computed **as if every one of them were applied wherever
  its conditions hold**, whether or not the emitted code keeps the check a proof would remove. So every
  implementation reaches the same verdict; an implementation may remove further checks from the code it
  emits, but that never changes a verdict. Adding a proof to the list is a language revision
  (`[VER-9]`) and makes more programs satisfy a contract, never fewer.
* `[EFF-16]` *(changed in 0.9.9)* `@nopanic(explicit)` forbids only the panics the programmer writes and
  allocation failure. Arithmetic, bounds, aliasing and stale checks are `RuntimeCheck` kinds and are
  allowed under it; the diagnostics and error page for `@nopanic(explicit)` MUST say so. A function that
  can abort in no way at all is `@nopanic(explicit)` **and** free of `RuntimeCheck` — two facts
  `ember inspect --safety` reports together. (`NonZero[T]` divisors and range facts remove division
  checks entirely, `[STD-4]`.)
* `[EFF-19]` `@realtime` names the contract set declared by the manifest of the package that declares
  the function (`[realtime] contracts = […]`), by default `@noalloc @nolock @noblock
  @nopanic(explicit)`. It is a marker for that set and not a timing guarantee. A diagnostic arising from
  it names both the failed contract and `@realtime`.
* `[EFF-23]` *(new in 0.9.9)* A contract attribute whose checking an implementation has not built is rejected with
  `E0900` (`[PHIL-12]`); it is never accepted unchecked.

## X.3 Determinism

Lock-step networking, replays and golden-image tests need two runs of a program on the same input to
compute the same bits, on one machine and across machines running the same binary.

* `[DET-1]` `@deterministic` on a function or module is a contract: `Nondet` MUST NOT appear in its
  effect set (`E4070`, naming the operation and the chain).
* `[DET-8]` `@deterministic` on a module applies to every function in it, with no opt-out. A foreign
  function is deterministic only if its declaration says `@ffi(deterministic)`, which is an asserted
  fact reported by `ember tcb`; `@deterministic` on an `extern` block is `E0104`.
* `[DET-9]` `ember inspect --deterministic <item>` prints whether the item satisfies `[DET-1]` and, if
  not, the shortest chain to each `Nondet` source; a function that relies on asserted foreign facts is
  reported as such, not as verified.
* `[DET-2]` *(changed in 0.9.9)* `Nondet` is introduced by exactly: floating-point contraction,
  reassociation or any `@fastmath` relaxation; a transcendental function from the platform library
  instead of `std.math.det`; observing a pointer or handle as an integer, or hashing one; making a
  `RandomState`, directly or as a `Map`'s or `Set`'s hasher; the wall clock, the monotonic clock, the system random source, thread and job
  completion order; reading uninitialised or padding bytes; any `extern` function not declared
  `@ffi(deterministic)`. Iterating a `Map` or `Set` is **not** `Nondet`: their order is insertion order
  (`[STD-11]`).
* `[DET-3]` A `@deterministic` function may call only `Nondet`-free functions; a callable parameter is
  checked per instantiation (`[EFF-2]`).
* `[DET-4]` `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt` for
  `f32` and `f64` returning the same bits on every supported target.
* `[DET-5]` Inside a `@deterministic` function the backend never contracts, reassociates or fuses; a
  `@fastmath` or `@fp(contract)` function there is `E4072`.
* `[DET-6]` `@deterministic` constrains results, not timing.
* `[DET-7]` The cross-machine claim holds for one binary. `ember build --build-id` prints a hash over the
  inputs of a reproducible build (`[BLD-13]`) so peers can confirm they run the same one.
* `[DET-10]` *(new in 0.9.9)* A deterministic program runs with a defined floating-point environment: round to nearest,
  ties to even; no flush-to-zero or denormals-are-zero; no trapping exceptions. The runtime establishes
  it at start-up and on every thread it creates, and foreign code that changes it is outside the
  guarantee.

## X.4 The cost model

* `[COST-1]` **Zero cost, defined.** An abstraction is zero-cost *for a use* whose dynamic checks are
  all proved away when the emitted code contains no instruction that equivalent hand-written C —
  C upholding the same invariant — would not contain.
* `[COST-2]` Every implicit cost is one of: **guaranteed elided** (emitting it is a defect);
  **guaranteed present**; **conditionally elided** (the compiler MAY remove it with a proof and reports
  whether it did); **implementation-defined** (not constrained, such as code size; two conforming
  implementations may differ); **not observable** (erased).
* `[COST-3]` *(changed in 0.9.9)* **The costs.**

| Cost | Class | Condition, and what the programmer can do |
|---|---|---|
| Bounds check on `a[i]` | conditionally elided | removed by loop versioning (`[OPT-2]`) and range facts; `get_unchecked` in `unsafe` |
| Overflow check on integer arithmetic | conditionally elided, **every profile** | removed by range facts; checked once per group in vectorised loops (`[SIMD-7]`); `@overflow(wrap)` or `wrapping_*` for code that wants wrapping |
| Floor `//` and `%` on signed integers | guaranteed present, small | one compare-and-adjust over C's truncation; free for a positive constant divisor; `div_trunc`/`rem_trunc` give C's |
| Division-by-zero and shift-range checks | conditionally elided | removed for constants, `NonZero` divisors and range facts |
| Retain / release pair | guaranteed elided | the cases of `[RC-2]` |
| Retain / release, surviving | guaranteed present | reported per site; atomic iff the class is `Sync` |
| Dynamic exclusivity check | conditionally elided | removed per `[EXC-3]`; hoisted per `[EXC-8]`; covered by `mut self` (`[EXC-15]`) |
| Per-field access word | guaranteed present | one `u32` per non-`Copy` field of a non-`@sync` class, and one check per such field on entry to a `mut self` method (`[EXC-19]`) |
| Stale-handle check | guaranteed present, **every profile** | the guarantee is the check; `get_unchecked` in `unsafe` |
| `RefCell` borrow | guaranteed present | one counter update |
| `Cell` access | not observable | a load or a store |
| Interface call through `dyn` | guaranteed present | one indirect call |
| Generic call, monomorphised | not observable | a direct, inlinable call |
| Generic call, shared (`[MONO-6]`) | guaranteed present | one indirect call per bound method; only above a declared instantiation budget |
| Callable parameter (`fn(A) -> R` as a parameter) | not observable | a direct call |
| Owned callable value | guaranteed present | one indirect call; one allocation if captures exceed three words (`[CLO-10]`) |
| `some I` return | not observable | the concrete type is known |
| Generator resume | guaranteed present | one jump on the state; the frame never allocates |
| Collection literal producing a heap collection | guaranteed present | one allocation (`Alloc`); a fixed-array context allocates nothing |
| String literal producing a `String` | guaranteed present | one allocation and copy; a `str` context allocates nothing |
| f-string | guaranteed present | allocates; `format_to` into a buffer does not |
| `?` converting an error into `AnyError` | guaranteed present, failure path only | one allocation (`[ERR-11]`); a concrete error type moves without allocating |
| Contract and range-type annotations | not observable | compile time only |
| Floating-point contraction off | guaranteed present | `a * b + c` is two roundings and two instructions, where C compilers often fuse them; `fma(a, b, c)`, `x.mul_add(b, c)` or `@fp(contract)` give one (`[TYP-9]`) |
| 64-bit `int` | guaranteed present | `int` is `i64`, so an `Array[int]` moves twice the bytes of C's 32-bit `int`; dense numeric data uses `i32`/`f32` elements (`L4003`) |
| Sorting floats | not observable | `sort` on floats compares by IEEE totalOrder (`[TYP-37]`), a few integer instructions per compare |
| Arena allocation | guaranteed present | a pointer bump |
| Stack probe | guaranteed present | one touch per page of a frame larger than a page (`[RT-12]`); none for smaller frames |
| FFI call | guaranteed present | one direct call for an ABI-direct C function |

* `[OPT-2]` For a counted loop over `a..b` that indexes views at `i + c` for constants `c`, with the
  views' bases and lengths invariant in the loop, the compiler MUST emit one entry test that all indices
  the loop can produce are in bounds, an unchecked body taken when it passes, and the checked body
  otherwise. Behaviour is unchanged in every case.
* `[OPT-3]` The loop bound in `[OPT-2]` may be written `s.len()`, a separate local the loop does not
  assign, or a constant; the three are treated alike.
* `[COST-4]` `ember inspect --cost <item>` prints every row that applies to an item, and for each
  conditionally elided cost whether it was removed and by which proof.
* `[COST-5]` A rule that introduces an implicit cost MUST add a row to this table; `rule_index.py` fails
  CI on a new implicit cost with no row.
---

# Part XI — Concurrency and Parallelism

Ember has OS threads, scoped tasks that may borrow the caller's data, a job system, and parallel
loops. Safe Ember has no data races (`[PHIL-10]`): two threads touch one memory location at the same
time only when both accesses read it, or when the location is an `Atomic`, or is protected by a
`Mutex`, `RwLock` or channel. The rules below are how that holds without a garbage collector and
without a check on every access.

```ember
import std.thread
from std.sync import Atomic, Mutex

@sync
class Stats:
    words: Atomic[int] = Atomic(0)
    tags: Mutex[Array[String]] = Mutex([])

fn scan(stats: Stats, text: str):
    for w in text.split_whitespace():
        stats.words.fetch_add(1)
        if w.starts_with("#"):
            with tags = stats.tags.lock():
                tags.push(w.to_string())

fn main():
    stats = Stats()
    texts = ["#ember is fast", "no tags here", "#python #c"]
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn() => scan(stats, t))
    println(stats.words.load(), "words")
```

## XI.1 `Send` and `Sync`

* `[THR-8]` *(new in 0.9.9)* A type is **`Send`** when a value of it may be moved to another thread.
  Structs, enums, tuples, fixed arrays and collections are `Send` when every component is. Raw
  pointers are not `Send`. `ref T` and `Span[T]` are `Send` when `T` is `Sync`; `ref mut T` and
  `MutSpan[T]` when `T` is `Send`. A class handle, and `Weak` of one, is `Send` only if the class is
  `@sync` (`[THR-2]`). `Shared[T]` is never `Send` (`[HEAP-10]`); `SyncShared[T]` and its `Weak` are.
* `[THR-9]` *(new in 0.9.9)* A type is **`Sync`** when several threads may read one value of it at
  the same time. Structural types are `Sync` when every component is. `Cell`, `RefCell`, `UnsafeCell`,
  raw pointers, `Shared[T]`, and handles (and `Weak`s) of classes that are not `@sync` are not `Sync`.
  `Atomic[T]`, `Once`, `SyncShared[T]`, `Mutex[T]`/`RwLock[T]` for a `Send` `T`, and handles of `@sync`
  classes are `Sync`.
* `[THR-1]` *(changed in 0.9.9)* **A class is `Sync` only when it is declared `@sync class`.** Every
  other class is neither `Sync` nor `Send`, whatever its fields, and uses plain (non-atomic) counts.
  In a `@sync` class:
  * every field's type is `Sync` (`E7001`, naming the field and the component that is not);
  * **every field is immutable after `init`.** Assigning a field, or beginning any write access to one
    (a `mut self` method on it, `ref mut`, `iter_mut`, passing it to a `mut` parameter), outside `init`
    is `E7003`; inside `init`, a field assignment after `self` has been used as a whole is `E7003`;
  * a method may not be declared `mut self` (`E7003`);
  * a base class and every derived class are `@sync` too (`E7001`).

  Mutation of a `@sync` object's state therefore happens only through its fields' own
  synchronisation — `Atomic`, `Mutex`, `RwLock`, `Once`, channel ends, or handles to other `@sync`
  objects. Every access to a `@sync` object is a read, so it needs no exclusivity check and its fields
  have no access words (`[EXC-19]`). The error's help names the field to wrap:
  `wrap it in Atomic[int]` for an integer, `Mutex[…]` otherwise.
* `[THR-2]` *(changed in 0.9.9)* Handles of a `@sync` class are `Send` and `Sync`. A handle of any other
  class cannot reach another thread by any Safe route — moving, capturing or borrowing into a task
  is `E7004` — so its objects need no atomic counts and no cross-thread exclusivity checks. The
  diagnostic offers `@sync` when the class could satisfy `[THR-1]`, and names the disqualifying field
  when it could not.
* `[THR-7]` `@sync` means exactly two things: the class's handles may cross threads, and its
  reference counts are atomic (`[RC-4]`). It does not make methods atomic: two threads calling
  `stats.words.fetch_add(1)` are each atomic, but a read followed by a write through two fields is
  two operations. A diagnostic MUST NOT describe `@sync` or `Sync` as "thread-safe".
* `[THR-13]` *(new in 0.9.9)* **The data-race guarantee.** In Safe Ember a memory location is reachable
  from two threads only through (a) a `@sync` object, whose fields are immutable after construction
  (`[THR-1]`); (b) a `static`, which is immutable and `Sync` (`[STA-1]`); (c) a `SyncShared[T]`,
  which gives only shared access; (d) a borrow handed to a scoped task, which the borrow rules make either shared and of a
  `Sync` type, or exclusive (`[THR-11]`); (e) a value moved to the other thread, which the sender no
  longer has. In each case concurrent mutation goes through `Atomic`, `Mutex`, `RwLock` or a channel.

## XI.2 Threads and synchronisation (`std.thread`, `std.sync`)

```ember
import std.thread
from std.sync import Sender, channel

fn produce(tx: Sender[int]):
    for i in 0..10:
        tx.send(i * i).expect("receiver is gone")

fn main():
    tx, rx = channel[int](capacity=64)
    producer = thread.spawn(owned fn() => produce(tx))
    total = 0
    for v in rx:
        total += v
    producer.join()
    println(total)
```

* `[THR-10]` *(new in 0.9.9)* `thread.spawn(owned f: fn() -> R) -> JoinHandle[R]` runs `f` on a new
  thread. The closure's captured state and `R` MUST be `Send` (`E7004`, naming the capture) and MUST
  carry only the static region: a thread may outlive every local of its creator, so a captured view of
  a local is `E3063`, whose help is `thread.scope()` (`[THR-5]`). `h.join() -> R` waits and returns the
  result. Dropping a `JoinHandle` detaches the thread.
* `[THR-16]` *(new in 0.9.9)* When `main` returns, or `process.exit` is called, the process ends: other
  threads stop without running destructors, no static is freed and no drop runs. The `debug` leak
  report (`[WK-15]`) runs only if no other thread is still running, and otherwise says how many were.
* `[THR-3]` *(changed in 0.9.9)* `Mutex[T]` is a value type. `m.lock()` blocks and returns a
  `MutexGuard[T]`, a view borrowing `m` that reads through to `ref mut T` (`[TYP-14]`); the mutex is
  released when the guard is dropped, which `with g = m.lock():` places at the end of the block.
  `m.try_lock() -> Option[MutexGuard[T]]` never blocks. A guard is not `Send`. There is no poisoning:
  a panic aborts the process (`[PAN-1]`). `RwLock[T]` has `read()` (shared guard, `ref T`) and
  `write()` (exclusive guard). `Once` runs an initialiser exactly once (`[STA-3]` is built on it).
* `[THR-14]` *(new in 0.9.9)* `Atomic[T]` exists for the integer types, `bool` and raw pointers.
  Every operation takes `order: MemoryOrder = MemoryOrder.SeqCst` (`Relaxed`, `Acquire`, `Release`,
  `AcqRel`, `SeqCst`); an order an operation does not support (`Acquire` on a store) is `E7006`.
  `fetch_add`/`fetch_sub` **panic when the result overflows**, like `+` (`[TYP-8]`); the overflowed
  value has been stored, and another thread may observe it before the process aborts.
  `checked_fetch_add` never stores an overflowed value (it is a compare-exchange loop and returns
  `Option`), and `wrapping_fetch_add` wraps without a check.
* `[THR-15]` *(new in 0.9.9)* `channel[T](capacity=n) -> (Sender[T], Receiver[T])` is a bounded
  many-producer, one-consumer queue; `channel_unbounded[T]()` has no bound and allocates as it grows.
  `Sender` is `Send`, `Sync` and `Clone`; `Receiver` is `Send`. `tx.send(v) -> Result[void, SendError[T]]`
  blocks while the queue is full and returns the value when the receiver is gone; `rx.recv() ->
  Option[T]` blocks and returns `None` once every sender has been dropped and the queue is empty;
  `try_send`/`try_recv` never block. A `Receiver` is iterable; the loop ends at `None`.
* `[THR-4]` In the `debug` profile the runtime records the order in which each thread takes locks and
  prints `warning: lock order inversion` with both acquisition sites when two threads take two locks
  in opposite orders. It never changes a program's behaviour.
* `[THR-6]` *(changed in 0.9.9)* A type whose `drop` a safety guarantee depends on is `@must_drop`. A
  `@must_drop` value MUST NOT be passed to `mem.forget`, stored in a class field, `Shared`,
  `SyncShared`, `Box`, a collection or an `owned fn` capture, held across a `yield` (`[CORO-13]`), or returned from the function that created it (`E3015`). The standard library
  applies it to `Scope` and `JobScope` only.

## XI.3 Scoped tasks

A scoped task may borrow the caller's locals, because the scope does not end until every task in it
has finished.

```ember
import std.thread

fn sum_halves(xs: Span[int]) -> int:
    left, right = xs.split_at(xs.len() // 2)
    with scope = thread.scope():
        a = scope.spawn(fn() => left.iter().sum())
        b = scope.spawn(fn() => right.iter().sum())
        return a.join() + b.join()
```

* `[THR-5]` *(changed in 0.9.9)* `thread.scope()` returns a `Scope`, which MUST be bound by a `with`
  (`E3014` otherwise). The `with` block is ordinary code of the enclosing function: `return`,
  `break`, `continue` and `?` inside it mean what they mean anywhere else. On **every** exit from the
  block — falling off its end, a jump, or `?` — the scope first joins every task spawned in it that
  has not been joined, and then the exit completes; the value of a `return` is computed before the
  join. The `Scope` binding is a view whose region is the block and is `@must_drop` (`[THR-6]`): it
  cannot be moved, stored, returned, captured by an `owned fn` or forgotten.
* `[THR-11]` *(new in 0.9.9)* `scope.spawn(f: fn() -> R) -> ScopedJoinHandle[R]` accepts a closure
  that borrows any place whose storage outlives the `with` block. Each borrow a spawned closure makes
  is live until the end of the block, so the ordinary borrow rules (`[BRW-2]`) reject two tasks that
  write one place, or one that writes a place another reads (`E3021`/`E3022`, naming both spawns). A
  shared borrow requires the borrowed type to be `Sync`, a mutable borrow requires it to be `Send`,
  and captured values and `R` MUST be `Send` (`E7004`/`E7005`). `h.join() -> R` waits early.
* `[THR-12]` *(new in 0.9.9)* Tasks that must run in sequence and share data use two scopes in
  sequence: the join at the end of the first is the ordering, and the borrows of the first end there.

## XI.4 Parallel loops

```ember
fn integrate(pos: MutSpan[float], vel: Span[float], dt: float):
    @parallel(chunk=4096)
    for i in 0..pos.len():
        pos[i] += vel[i] * dt
```

* `[PAR-1]` A `@parallel` loop's body runs as a closure over index chunks on the job system (§XI.5).
* `[PAR-2]` **Iterations MUST be independent.** The body MUST NOT contain `break`, `return`, `yield`,
  or a `continue` naming an outer loop, and its captures are `Send`. For each place `P` written in the
  body, the compiler collects every index expression through which `P` is accessed in the body,
  **reads included**. The loop is accepted only if (a) each has the form `i + k` for a constant `k`;
  (b) all of them share **one** `k`; and (c) `P` is a `MutSpan` or an `SoA` column captured by the
  loop, not a place reached through a class handle, `Cell`, `RefCell` or raw pointer. Otherwise
  `E7010`, with the alternatives `Atomic`, `chunks_mut` and `@parallel(reduce=…)`.
* `[PAR-2a]` A violation of clause (b) is `E7011`, `parallel loop has a loop-carried dependency on
  <place>`, naming both offsets and offering: two `@parallel` loops, a reduction, or a serial loop.
* `[PAR-2b]` The analysis runs after the body's calls are inlined. A call it cannot see into that
  receives a captured `MutSpan` or column defeats the proof for everything reachable through that
  argument (`E7010`).
* `[PAR-3]` *(changed in 0.9.9)* `@parallel(reduce=[total: +, best: max])` declares variables combined
  after the loop; inside the body each chunk has its own copy. Each chunk reduces its iterations left
  to right, and the chunk results are then combined in chunk order, so the result is the same on
  every machine (`[PAR-5]`); for floats it can differ from the serial left-to-right sum, which is
  why a float reduction must be declared.
* `[PAR-4]` *(changed in 0.9.9)* A `@parallel` loop has the effects `Sync` and `Block` (it waits for
  its chunks) and not `Alloc`: its chunk descriptors live in the calling frame and the job queues have
  a fixed capacity; when they are full the calling thread runs the chunk itself.
* `[PAR-5]` *(new in 0.9.9)* **Parallel results do not depend on the machine.** Chunk boundaries are a
  function of the trip count and `chunk` only (a default `chunk` is a function of the trip count
  only), and reductions combine per-chunk results in chunk order. A `@parallel` loop therefore
  computes the same bits whatever the worker count and scheduling, and is not `Nondet` (`[DET-2]`).

## XI.5 The job system (`std.jobs`)

The job system is a pool of worker threads with work-stealing queues. The runtime starts it before
`main` in any program that uses `@parallel` or `std.jobs`, with the worker count from the manifest
(`[jobs] workers = "auto"`, meaning logical cores minus one).

```ember
import std.jobs

fn step(pos: MutSpan[Vec3], vel: Span[Vec3], life: MutSpan[float], dt: float):
    with s = jobs.scope():
        s.submit(fn() => integrate(pos, vel, dt))
        s.submit(fn() => age(life, dt))
```

* `[JOB-1]` A job's captured state is stored inline in its queue slot when it fits in 64 bytes and in
  one heap allocation otherwise (`Alloc`, reported by `ember inspect --alloc`).
* `[JOB-2]` *(changed in 0.9.9)* `jobs.scope()` follows `[THR-5]` and `[THR-11]`: jobs submitted to a `JobScope` may borrow
  the caller's data, and the block's exit joins them. `jobs.submit(owned f: fn())` outside a scope
  takes an owned, `Send` closure carrying only the static region (as `[THR-10]`) and returns a
  `JobHandle`; `jobs.wait(h)` waits for it.
* `[JOB-3]` *(changed in 0.9.9)* `s.submit_after(deps, f)` starts `f` only after the jobs in `deps` have
  finished. It orders execution; it does not relax the borrow rules, which treat every job of a scope
  as running at once. (The 0.9.8 declared read/write access sets, verified only in debug builds, are
  removed: a safety property checked in one profile is not a guarantee, `[PHIL-13]`.)
* `[JOB-5]` `jobs.local_arena()` returns an arena view for the current job, reset when the job ends;
  allocations from it cannot escape the job.

## XI.6 Async

`async` and `await` are reserved words (§II.4). Ember's workloads are covered by threads, scoped tasks,
jobs and generators (§VI.5a); a later version may add structured async over the job system, in which
a borrow may not cross an `await` unless the future is scoped. Generators are designed so that this
remains possible: a generator frame is a sized value, and the same frame lowering serves an
`async fn`.
---

# Part XII — Data-Oriented Programming

Data-oriented code keeps each field of many objects in its own contiguous array, walks those arrays
in tight loops, and lets the machine's vector units do several elements per instruction. Ember makes
that layout a one-word change and makes the loops provably free of aliasing, so they run at C speed
with the safety checks either proved away or hoisted out.

## XII.1 `SoA[T]`

```ember
from std.math import Vec3

struct Particle:
    pos: Vec3
    vel: Vec3
    life: f32

fn integrate(pos: MutSpan[Vec3], vel: Span[Vec3], dt: f32):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn main():
    ps = SoA[Particle].with_capacity(100_000)
    ps.push(Particle(pos=Vec3.ZERO, vel=Vec3(1.0, 0.0, 0.0), life=2.0))
    integrate(ps.pos, ps.vel, 0.016)        # two columns borrowed at once: disjoint places
    ps[0].life -= 0.016                     # writes one element of the `life` column
    p = ps.get(0)                           # Option[Particle]: gathers a copy
```

* `[SOA-1]` *(changed in 0.9.9)* `SoA[T]` is a **compiler-known type constructor**, like `Cell`: for
  any struct `T` it is a growable container holding one column per field of `T`. No derive is needed.
  `T` MUST be a struct (`E2071` otherwise). A field marked `@soa(flatten)` whose type is a struct is
  split into one column per field of its own (`ps.pos.x`); otherwise a struct field is one column.
* `[SOA-2]` `ps.f` names the column of field `f`. It is a place of type `MutSpan[F]` where `ps` is
  mutable and `Span[F]` otherwise, and it supports everything those views support. Columns are
  **disjoint places** under `[BRW-4]`, so any number of different columns may be borrowed at once,
  mutably or not. A length-changing operation (`push`, `swap_remove`, `clear`) borrows all of `ps`.
* `[SOA-6]` *(new in 0.9.9)* `ps[i]` is an **element proxy**: `SoARef[T]` where `ps` is read-only here
  and `SoAMut[T]` where it is mutable. A proxy's field `ps[i].f` is a place in column `f` at index `i`;
  reading or writing it touches that column only, and no `T` is assembled. `ps.get(i) -> Option[T]`
  and `ps[i].load() -> T` gather a value (for `T: Clone`); `ps[i] = v` scatters one. `for p in ps:`
  yields `SoARef[T]`, `for p in ps.iter_mut():` yields `SoAMut[T]`. A `SoAMut[T]` borrows all of `ps`
  mutably for the proxy's life and a `SoARef[T]` borrows it shared, so a column borrowed separately
  conflicts with a live proxy.
* `[SOA-3]` `SoA[T]` provides `len`, `is_empty`, `push`, `pop`, `swap_remove`, `retain`, `clear`,
  `reserve`, `with_capacity`, and `sort_by_key` over any column; every operation keeps the columns
  aligned element for element.
* `[SOA-7]` *(new in 0.9.9)* All columns of one `SoA[T]` live in **one heap block**, each column
  starting at an address aligned to the larger of 64 bytes and its element type's alignment. Growth
  reallocates the block once. A column's base pointer is therefore suitable for aligned vector loads.
* `[SOA-4]` `ArenaSoA[T]` is the fixed-capacity, arena-backed form, under the rules of `[ARN-5]`.

## XII.2 SIMD

```ember
from std.simd import f32x8

fn scale(xs: Span[f32], out: MutSpan[f32], k: f32):
    assert(xs.len() == out.len() and xs.len() % 8 == 0)
    kv = f32x8.splat(k)
    for i in (0..xs.len()).step_by(8):
        v = f32x8.load(xs[i..i + 8])
        (v * kv).store(out[i..i + 8])
```

* `[SIMD-1]` `std.simd` provides vector types `f32x4`, `f32x8`, `f32x16`, `f64x2`, `f64x4`, `f64x8`,
  the signed and unsigned integer vectors of 8 to 64-bit lanes, and mask types. They are `Copy` value
  types aligned to their size. On the C backend they lower to the target's intrinsics through
  `ember_simd.h`, with a scalar fallback on targets without them. `f32x8.LANES` is the lane count;
  `simd.native_lanes[T]()` is the target's preferred count, a compile-time constant.
* `[SIMD-8]` *(new in 0.9.9)* Vector operators are lane-wise and follow the scalar rules: integer
  lanes panic on overflow (`[TYP-8]`) unless the function is `@overflow(wrap)` or the code calls
  `wrapping_add` and friends; float lanes are IEEE with no contraction (`[TYP-9]`).
* `[SIMD-4]` Horizontal operations (`reduce_add`, `reduce_min`, `reduce_max`), shuffles with a
  constant lane list, gathers, scatters, `fma`, and approximations named `_approx` (`rsqrt_approx`,
  `rcp_approx`) are provided.
* `[SIMD-9]` *(new in 0.9.9)* SIMD memory operations are bounds-checked like indexing. `V.load(s)` and
  `v.store(s)` require `s.len() >= V.LANES`; `V.gather(s, idx)` and `v.scatter(s, idx)` check every
  lane's index against `s.len()`; each failure is a `Bounds` panic naming the lane. The unchecked forms
  (`load_unchecked`, `store_unchecked`, `gather_unchecked`, `scatter_unchecked`) are `unsafe fn`. A
  `load` or `store` at a loop's induction variable is covered by the entry test of `[OPT-2]`.
* `[SIMD-2]` `@simd` on a `for` loop is a request and a report: the compiler tries to vectorise the
  loop and `--emit-optimization-report` says whether it did and why not. It never changes meaning.
  `@simd(assert)` makes failure `E4020`.
* `[SIMD-3]` *(changed in 0.9.9)* Alias facts given to the backend (`restrict`) MUST be derived, never assumed. Two views
  are marked disjoint only when (a) the borrow checker related them to one owner and proved their
  ranges disjoint; (b) they come from owners proved distinct in the same function; (c) they are
  results of `split_at`, `chunks_mut` or distinct `SoA` columns; or (d) they carry a fact established
  by `assert_disjoint` (`[DSJ-1]`) or asserted by `unsafe assume_disjoint`. A view built by `unsafe`
  code or received through FFI may alias until (d) is applied. A view reached through a class handle
  or a `Shared` gets an alias fact from (a)–(c) only when the loop contains no call that is not
  inlined, no call through a callable value, no virtual or interface dispatch and no access through
  another handle (the conditions of `[EXC-3]`), because an unchecked write through another handle may
  change what it views (`[EXC-17]`). Two views being live at once is not a proof, and neither are
  parameter modes.
* `[SIMD-5]` *(changed in 0.9.9)* **Vectorisable form**, which `@simd(assert)` checks, is computed by
  Ember and never delegated to the C compiler. A loop is in vectorisable form when: its trip count is
  known before entry; every memory access is to a local or to a view at a unit-stride affine index of
  the induction variable; every written place is accessed at one constant offset (`[PAR-2]`(b)); every
  written view is disjoint from every other accessed view by `[SIMD-3]`; every call in the body is
  inlined (`[CG-C-3]`); the body has none of `Alloc`, `Sync`, `Block`, `Io`, `FFI`,
  `Panic(Explicit)`; every `RuntimeCheck(Bounds)` has been removed by `[OPT-2]`; every
  `RuntimeCheck(Arithmetic)` is removed, or is one `[SIMD-7]` permits grouping (grouping is then
  required); the loop has one exit; and every floating-point reduction is declared by `@parallel(reduce=…)` or the function is `@fastmath`.
  `E4020` names the first clause that fails, and for a remaining check its reason (`[EFF-11]`).
* `[SIMD-7]` *(new in 0.9.9)* **Grouped overflow checks.** In a loop whose body has none of `Sync`,
  `Io`, `FFI`, `Unsafe` and `Block`, the compiler MAY check integer overflow once per group of
  iterations (a vector's width times the unroll factor) instead of once per operation, and in a loop in
  vectorisable form (`[SIMD-5]`) it MUST: it computes the group with wrapping arithmetic, accumulates the lanes' overflow bits, and panics at the end of the
  group, reporting the source location of the first overflowing iteration. This is not observable:
  the body's writes go only to memory this thread owns, and the process aborts before anything reads
  them. Bounds checks are never grouped. An integer **reduction** (`total += xs[i]`) is vectorised
  under checked arithmetic only when no partial sum can overflow in any grouping; the compiler proves
  that from the element and accumulator widths, versioning the loop on the trip count as `[OPT-2]`
  does (a sum of 32-bit elements into an `int` is safe for fewer than 2³¹ elements).
* `[SIMD-6]` The C compiler's own vectorisation report is corroborating evidence only.
  `--emit-optimization-report` prints it labelled as the host compiler's opinion, after Ember's
  vectorisable-form verdict; its absence or disagreement is never an error.

## XII.3 ECS (`std.ecs`)

The entity-component system is an ordinary library built on `SoA`, generational handles and
compile-time reflection. It has no compiler support of its own.

```ember
from std.ecs import World, Query, Mut, Component
from std.math import Vec3

@derive(Copy, Component)
struct Position:
    value: Vec3

@derive(Copy, Component)
struct Velocity:
    value: Vec3

@noalloc
fn integrate(mut q: Query[(Mut[Position], Velocity)], dt: f32):
    for pos, vel in q.iter_mut():           # pos: ref mut Position, vel: ref Velocity
        pos.value += vel.value * dt

fn main():
    world = World()
    e = world.create()
    world.add(e, Position(Vec3.ZERO))
    world.add(e, Velocity(Vec3(1.0, 0.0, 0.0)))
    world.run(integrate, 0.016)
```

* `[ECS-1]` *(changed in 0.9.9)* `Entity` is a 32-bit generational handle: a 20-bit index and a 12-bit
  generation (`Handle` with `Split20`, `[HND-2]`); `Entity.NULL` is all ones. The compact split is kept
  so entity ids match engines that use it; general `Pool` handles are 64-bit (`[HND-1]`).
* `[ECS-2]` Each component type is stored as a sparse set: a dense `Array[T]` or `SoA[T]`
  (`@component(layout=soa)`; the default is `aos`), the entities in dense order, and a sparse index by
  entity. A component type's identity is the 64-bit FNV-1a hash of its fully qualified name
  (`ecs.type_hash[T]()`), stable across builds and modules.
* `[ECS-3]` *(changed in 0.9.9)* `Query[(A, Mut[B], Option[C], Not[D])]` is a view over a world's
  storages. `Mut[T]`, `Not[T]` and `Option[T]` in a query are **ordinary types**: `Mut` and `Not` are
  marker structs with a phantom parameter (`[TYP-35]`), and the library maps each parameter to its
  item type (`ref T`, `ref mut T`, `Option[ref T]`, nothing) through an interface with an associated
  type. A query iterates the smallest required storage and probes the others; `iter` borrows `q`
  shared and yields read-only items; `iter_mut` yields `ref mut` for `Mut` parameters and requires
  `mut q`. There is no access-mode generic kind in the
  language.
* `[ECS-4]` *(changed in 0.9.9)* A query's read and write sets are compile-time constants computed from
  its type. `world.run(system, args…)` borrows exactly those storages. `world.run_parallel([s1, s2, …])`
  runs systems at once when their sets do not conflict; a conflict between two systems known at
  compile time is `E7020`, naming both systems and the component. Systems passed as values, whose sets
  are not known at compile time, are checked against each other before any of them starts, and a
  conflict panics, naming both systems and the component.
* `[ECS-5]` Adding, removing and destroying during iteration go through a `Commands` buffer
  (`q.commands().destroy(e)`) that is applied after the system returns.
* `[ECS-6]` Iteration order is insertion order with swap-remove holes: deterministic for a given
  sequence of operations.
* `[ECS-7]` `@derive(Component)` registers the type in a compile-time component registry that
  serialisation and reflection-based tools read (Part XIV).
---

# Part XIII — Error Handling

A failure the caller can handle is a **value**: a function that can fail returns `Result`, and `?`
passes a failure up. A failure that means the program is wrong — an index out of range, an overflow,
a broken invariant — is a **panic**, which stops the process (`[PAN-1]`). There are no exceptions and
no hidden control flow: every place a function can return early is marked by `?` or `return`.

```ember
import std.fs

@derive(Error)
enum ConfigError:
    @error("line {line}: expected `key = value`")
    Syntax(line: int)
    @error("line {line}: `{key}` is not a number")
    NotNumber(line: int, key: String)

fn parse(text: str) -> Result[Map[String, int], ConfigError]:
    out: Map[String, int] = {}
    for n, line in text.lines().enumerate():
        key, sep, value = line.partition("=")
        if sep.is_empty():
            return Err(ConfigError.Syntax(line=n + 1))
        match value.trim().parse[int]():
            Ok(v):
                out[key.trim()] = v
            Err(_):
                return Err(ConfigError.NotNumber(line=n + 1, key=key.trim().to_string()))
    return Ok(out)

fn main() -> Result[void]:
    text = fs.read_to_string("app.cfg")?            # io.Error -> AnyError
    cfg = parse(text).context("reading app.cfg")?   # ConfigError -> AnyError, with a message
    println(cfg)
    return Ok(())
```

## XIII.1 `Result`, `Option` and `?`

* `[ERR-1]` *(changed in 0.9.9)* `Option[T]` is absence: `Some(v)` or `None`. `Result[T, E = AnyError]`
  is success or failure: `Ok(v)` or `Err(e)`, where `E: Error`. Both are ordinary enums in the prelude.
* `[ERR-9]` *(new in 0.9.9)* The error parameter defaults to `AnyError` (§XIII.2), so
  `fn load(path: str) -> Result[Texture]` is a function that can fail with any error. A library that
  wants callers to `match` on its failures names a concrete error type.
* `[ERR-2]` *(changed in 0.9.9)* `e?` on a `Result[T, E]` in a function returning `Result[U, F]`
  evaluates to the `Ok` payload, or returns `Err(F.from(err))`. On an `Option[T]` in a function
  returning `Option[U]` it evaluates to the `Some` payload or returns `None`. When `E` and `F` are the
  same type the conversion is a move with no call. `?` in a function returning neither is `E2180`.
  `?` on an `Option` in a function returning `Result`, or on a `Result` in a function returning
  `Option`, is `E2181`, whose fix-its are `.ok_or(err)?` and `.ok()?`.
* `[ERR-4]` *(changed in 0.9.9)* `Option` and `Result` provide `is_some`/`is_none`, `is_ok`/`is_err`,
  `unwrap`, `expect`, `unwrap_or`, `unwrap_or_else`, `unwrap_or_default`, `map`, `map_err`,
  `and_then`, `or_else`, `ok`, `err`, `ok_or`, `ok_or_else`, `filter`, `take`, `replace`, `as_ref`,
  `as_mut`, `iter`, and `context` (`[ERR-10]`). `unwrap` and `expect` panic with the error's `Display`
  text. The methods that take a function are eager: each consumes its receiver and calls the function
  at most once, before it returns, moving the payload into it except for `filter` (ODR-025):
  * `Option[T]`: `map[U](owned self, f: once fn(owned T) -> U) -> Option[U]`,
    `and_then[U](owned self, f: once fn(owned T) -> Option[U]) -> Option[U]`,
    `filter(owned self, f: once fn(T) -> bool) -> Option[T]`,
    `or_else(owned self, f: once fn() -> Option[T]) -> Option[T]`,
    `unwrap_or_else(owned self, f: once fn() -> T) -> T` and
    `ok_or_else[E](owned self, f: once fn() -> E) -> Result[T, E]`;
  * `Result[T, E]`: `map[U](owned self, f: once fn(owned T) -> U) -> Result[U, E]`,
    `map_err[F](owned self, f: once fn(owned E) -> F) -> Result[T, F]`,
    `and_then[U](owned self, f: once fn(owned T) -> Result[U, E]) -> Result[U, E]`,
    `or_else[F](owned self, f: once fn(owned E) -> Result[T, F]) -> Result[T, F]` and
    `unwrap_or_else(owned self, f: once fn(owned E) -> T) -> T`.
* `[ERR-5]` `Result` is `@must_use`: discarding one is `W2190` (an error under `-D warnings`). A value
  is used when it is bound, returned, passed, matched, or has `?` or a method applied.
* `[ERR-7]` **Identity conversion.** The prelude provides `From[T]` for every `T` (`from(v) = v`), so
  `?` propagates an unchanged error type. A user `extend T implements From[T]` is `E2041`.

## XIII.2 Error types and `AnyError`

* `[ERR-3]` *(changed in 0.9.9)* `Error` is the interface of error types: `Display + Debug` with
  `source(self) -> Option[ref dyn Error]` (default `None`) (§IV.8). `@derive(Error)` on an enum or
  struct implements `Display` from `@error("…")` attributes — one per variant, whose `{name}` and
  `{0}` placeholders name the variant's fields — and implements `Debug` if it is not already derived.
  A variant marked `@from` whose single field is an error type gets `From` for that type and returns it
  from `source`.
* `[ERR-8]` *(changed in 0.9.9)* **`AnyError`** is the prelude's "any error" type: an owned, boxed
  `dyn Error` plus an optional chain of context messages. It implements `From[E]` for every `E: Error`,
  so `?` converts any error into it, and `Display`, `Debug`, `source`, `downcast_ref[E]() -> Option[ref
  E]` and `is[E]() -> bool`. **`AnyError` does not implement `Error`** — otherwise `[ERR-7]` and
  `[ERR-8]` would both supply `From[AnyError]` for `AnyError`. An `AnyError` passed through `?` into
  another `AnyError` is moved (`[ERR-7]`).
* `[ERR-10]` *(new in 0.9.9)* `r.context(msg)` on a `Result[T, E]` returns a `Result[T, AnyError]`
  whose error, if any, displays as `msg: <original>`; `r.with_context(fn() => …)` builds the message
  only on failure. Context messages stack.
* `[ERR-11]` *(new in 0.9.9)* **The cost of `AnyError`.** Converting an error into `AnyError`
  allocates, once, on the failure path; that path carries `Alloc` (`[EFF-1]`). A concrete error type
  moves through `?` without allocating, which is why `@noalloc` code names its error types.
* `[ERR-12]` *(new in 0.9.9)* An `Err` returned from `main` (`[FN-8]`) prints `error: <Display>` to
  standard error, then one `caused by: <Display>` line for each `source` in the chain, and exits with
  status 1.

## XIII.3 Panics or errors

* `[ERR-13]` *(new in 0.9.9)* The standard library follows one convention, and user code SHOULD: an
  operation whose failure is a bug in the caller panics (`xs[i]` out of range, `m[k]` for a missing
  key, integer overflow, `unwrap` on `None`); an operation whose failure depends on the world or on
  input returns `Result` or `Option` (`xs.get(i)`, `m.get(k)`, `parse`, every I/O call). Each
  panicking accessor has a non-panicking twin, and the panic message names it
  (`index 7 out of range for length 3; use .get(i) for an Option`). The one exception is console input,
  which panics at end of input (`[STD-10]`), as Python's `input` raises.
* `[ERR-6]` A foreign function's status code becomes a `Result` at the binding (Part XVI): an
  `@ffi(status=VkResult, ok=VK_SUCCESS)` overlay turns a `VkResult` return into
  `Result[void, VkError]`.

Python's `try`, `except`, `raise` and `finally` are not Ember syntax; each is diagnosed with the Ember
form (`[DIA-21]`): `raise e` → `return Err(e)`, a `try` body → `?` on each fallible call, `except E`
→ `match` on the result, `finally` → `defer:`.
---

# Part XIV — Compile-Time Programming

Ember runs ordinary Ember code at compile time. There is no second language for it, no macro system,
and no `comptime fn` marker: a function can run at compile time when what it does can.

```ember
from std.math import sin, TAU

@layout(c)
struct Vertex:
    pos: [f32; 3]
    normal: [f32; 3]
    uv: [f32; 2]

fn sine_table() -> [float; 256]:
    t = [0.0; 256]
    for i in 0..256:
        t[i] = sin(TAU * i as float / 256.0)
    return t

const SINE: [float; 256] = comptime(sine_table())

comptime:
    assert(mem.size_of[Vertex]() == 32, "Vertex layout changed; update the shader")
```

## XIV.1 Compile-time evaluation

* `[CT-1]` *(changed in 0.9.9)* These are evaluated during compilation: `comptime(e)`; `comptime:`
  blocks; `const` initialisers; array lengths and `const` generic arguments; and `static`
  initialisers that qualify under `[STA-3]`. Any function may be called there if its effect set
  contains nothing beyond `Alloc`, `Panic(Explicit)` and `RuntimeCheck` — no `Io`, `FFI`, `Sync`,
  `Block`, `Unsafe` or `Nondet`. Allocation is served by the evaluator's own heap.
* `[CT-6]` *(new in 0.9.9)* `comptime(e)` is an expression whose value is `e` evaluated at compile
  time. `e` may refer to constants, generic parameters, and statics initialised at compile time, not
  to run-time locals (`E6005`).
* `[CT-7]` *(new in 0.9.9)* A `comptime:` block at item level, or as a statement, runs once during
  compilation for its checks. A failed `assert` or any other panic there is `E6004`, which shows the
  panic message and the compile-time call chain.
* `[CT-2]` *(changed in 0.9.9)* An operation that cannot run at compile time — an `extern` call, a
  thread, a clock, `unsafe` pointer access outside the evaluator's heap, I/O — is `E6010`, naming the
  call chain. Two inputs are provided: `comptime.read_file(path)` (relative to the package root, and only inside the
  package directory, `E6010` otherwise, so a dependency's build cannot read the builder's files) and
  `comptime.env(name)`. Both are recorded as build inputs.
* `[CT-3]` Each evaluation is limited to 10⁸ steps and 256 MB of evaluator heap by default
  (`[comptime]` in the manifest); exceeding either is `E6001`.
* `[CT-4]` *(changed in 0.9.9)* Compile-time evaluation is deterministic by construction: the evaluator
  has no source of `Nondet` (`[DET-2]`), uses `std.math.det` for transcendental functions, and
  computes integer and floating-point results exactly as the target does, including overflow panics
  and the target's pointer size, endianness and layout. A result is cached with its inputs (source,
  `read_file` contents, `env` values).
* `[CT-5]` *(changed in 0.9.9)* **Where results live.** A compile-time result is placed in the image as
  constant data. A value that owns heap memory (`Array`, `String`, `Map`, `Box`, …) is handled so that
  no run-time code ever grows or frees constant data:
  * in a `static` whose type gives Safe code no way to obtain `ref mut` to the value (no `Mutex`,
    `RwLock`, `Cell`, `RefCell` or `Atomic` on the path to it), the value is materialised in
    read-only memory; statics are never dropped;
  * in any other `static` (`static LOG: Mutex[Array[String]] = Mutex([])`), the initialiser runs at
    run time on first access (`[STA-3]`), producing an ordinary heap value;
  * a `comptime(e)` whose type owns heap memory is materialised once, and each run-time evaluation of
    the expression **clones** it into a fresh allocation (`Alloc`), so the program owns an ordinary
    value it may grow. Binding the value to a `static` and borrowing it avoids the copy.

  A `const` cannot own heap memory (§V.7).

## XIV.2 Reflection

* `[RFL-1]` `reflect[T]()` at compile time returns a `TypeDesc`: name, kind, size, alignment, fields
  (name, type, offset, attributes), variants, and attributes. It is not available at run time.
* `[RFL-2]` `@reflect` on a type emits run-time type information: `type_info[T]()`, and
  `h.type_info()` on a class handle, return a `ref TypeInfo` with the same field descriptors. Tables
  no code reaches are removed at link time.
* `[RFL-3]` *(changed in 0.9.9)* **User attributes are declared.** A struct marked `@attribute` may be
  written as an attribute on the fields and declarations of `@reflect` types: `@Range(min=0.0,
  max=1.0)` is checked exactly like the constructor call `Range(min=0.0, max=1.0)`, and its value is
  readable through `TypeDesc` and `TypeInfo`. An attribute that names no built-in attribute and no
  visible `@attribute` struct is `E0104` (`[PHIL-12]`).

## XIV.3 Derives

* `[DRV-1]` *(changed in 0.9.9)* `@derive(…)` requests generated implementations. The built-in derives,
  and exactly what each generates, are:

  | Derive | Generates |
  |---|---|
  | `Copy` | the marker; every field MUST be `Copy` and the type MUST have no `drop` (`E2080`) |
  | `Clone` | field-wise clone (implicit per `[STR-5]`) |
  | `Eq` | field-wise equality in declaration order; enums compare variant first (implicit per `[STR-5]`) |
  | `Debug` | `Point(x=1.0, y=2.0)` for a struct, `Shape.Circle(r=1.0)` for a variant, `Mode.Fast` for a unit variant; strings quoted (implicit per `[STR-5]`) |
  | `Display` | on a unit-only enum, the variant name; on a struct or payload enum, the `Debug` text |
  | `Ord` | lexicographic over fields in declaration order; enums by variant order, then fields |
  | `Hash` | feeds each field in declaration order; enums feed the variant index first |
  | `Default` | each field's declared default, else its type's `Default`; for an enum, the variant marked `@default` |
  | `Error` | `[ERR-3]` |
  | `Serialize`, `Deserialize` | §XIV.4 |
  | `Zeroable` | `[ARN-11]` |
  | `Component` | `[ECS-7]` |

  A derive whose requirement a field does not meet is `E2080`, naming the field. The generated code
  is ordinary Ember that `ember inspect --expand <type>` prints.
* `[DRV-2]` User-defined derives are not in this version; `macro` is reserved for them.

## XIV.4 Serialisation (`std.ser`)

* `[SER-1]` *(new in 0.9.9)* `@derive(Serialize, Deserialize)` implements `serialize[W: ser.Writer](self,
  mut w: W) -> Result[void, SerError]` and `deserialize[R: ser.Reader](mut r: R) -> Result[Self,
  SerError]` over a field-tagged, versioned format; both are monomorphised per format.
  `std.ser.binary` and `std.ser.json` provide a writer and reader each. Field attributes: `@ser(skip)`, `@ser(rename="…")`,
  `@ser(default)`, `@ser(since=N)`.
* `[SER-2]` *(new in 0.9.9)* Deserialising never produces an invalid value: a range-typed field is
  checked (`[RNG-10a]`), an unknown enum tag is an error, and a missing field without `@ser(default)`
  or `@ser(since=…)` is an error.
---

# Part XV — The Standard Library

The standard library is one package, `std`. This Part fixes what every implementation provides and
what each operation means; `std/**/*.em` holds the exact signatures and becomes normative for
anything this Part leaves open.

## XV.1 Organisation and conventions

* `[STD-6]` `std` is layered, and each layer depends only on the ones before it: **core** (scalars,
  `Option`, `Result`, views, fixed-capacity containers, math, SIMD), **alloc** (`Array`, `String`,
  `Box`, `Map`, `Set`, `Shared`), **sync** (atomics, locks, channels, threads, jobs), **io** (console,
  files, time, processes) and **ffi**. A package that declares `[package] layers = ["core"]` may use
  only core; this is how firmware and kernels use Ember.
* `[STD-1]` Every function in `std.core`, `std.mem`, `std.math`, `std.simd` and `std.arena` is
  `@noalloc` unless its documentation says it allocates.
* `[STD-13]` *(new in 0.9.9)* **One naming convention.** Names are `snake_case` words joined by
  underscores (`starts_with`, `to_upper`, `push`, `is_empty`). Where Python's name for a **method**
  differs (`append`, `strip`, `startswith`, `upper`, `splitlines`), there is no second name: the call
  is `E2072`, `no method 'append' on Array[int]`, whose help names the Ember method
  (the full table is Appendix E). A Python name that Ember keeps (`partition`, `items`, `count`, and
  the built-in functions of `[STD-26]`) keeps Python's meaning (`[PHIL-14]`).
* `[STD-26]` *(new in 0.9.9)* **Python's built-in functions.** The prelude has these functions, with
  Python's meaning; each borrows its argument as a `for` loop would (`[CTL-1]`):
  * `len(x) -> int` is `x.len()` for a collection, view or range. `len` of a `str` or `String` is
    `E2073`: Python counts characters and Ember's `s.len()` counts bytes, so the help offers both
    `s.char_count()` and `s.len()`.
  * `range(stop)`, `range(start, stop)`, `range(start, stop, step)` count as Python's do, including a
    negative `step`; `step == 0` panics. `range(n)` is `0..n`, and every form compiles to a counted
    loop (`[CTL-3b]`).
  * `enumerate(it, start=0)`, `zip(a, b)` (stops at the shorter) and `reversed(x)` (for what can run
    backwards) return lazy iterators, as Python 3's do.
  * `sum(it, start=0)`, `any(it)` and `all(it)` consume an iterable (of numbers, of `bool`s);
    `any(x > 0 for x in xs)` uses a generator expression (`[GRM-38]`).
  * `sorted(it, key=None, reverse=false) -> Array[T]` returns a new sorted array (stable).
  `min` and `max` stay two-argument functions; `xs.iter().min()` gives an `Option` for a collection.
* `[STD-14]` *(new in 0.9.9)* Failure follows `[ERR-13]`: a caller's bug panics and has a
  non-panicking twin; a failure of the world returns `Result` or `Option`. Every allocating function
  is documented as allocating and carries `Alloc`.

| Module | Contents |
|---|---|
| prelude | the names of `[MOD-5]` |
| `std.core` | scalars, `Option`, `Result`, `AnyError`, `Ordering`, ranges, `NonZero`, the standard interfaces (§IV.8), iterator adapters |
| `std.mem` | `size_of`, `align_of`, `offset_of`, `swap`, `replace`, `take`, `drop`, `forget`, `keep_alive`, `MaybeUninit`, `transmute`, `zeroed`, `copy`, `copy_nonoverlapping`, `Volatile`, `Layout`, `Allocator`, `Global`, `ptr`, `assert_disjoint`, `assert_disjoint_or_panic`, `assume_disjoint` (Part IX) |
| `std.cell` | `Cell`, `RefCell`, `Ref`, `RefMut`, `UnsafeCell` |
| `std.collections` | `Array`, `Map`, `Set`, `Deque`, `SmallArray`, `BitSet`, `FixedArray`, `FixedString`, `RingBuffer`, `Pool`, `Handle`, `SoA`, `ArenaArray`, `ArenaMap`, `ArenaSoA`, `DefaultHasher`, `RandomState`, `Hasher`, `CapacityError`; the iterator types `SpanIter`, `MutSpanIter`, `SpanChunks`, `MutSpanChunks`, `ArenaArrayIter`, `ArenaArrayIterMut`, `ArenaMapIter` (public, not in the prelude) |
| `std.string` | `String`, `str` methods, `StringBuilder`, `ParseError`, `Utf8Error` |
| `std.fmt` | `Formatter`, `FmtError`, `format`, `format_to`, the format-spec mini-language (`[LEX-19]`) |
| `std.math` | constants, scalar functions, `Vec2/3/4`, `IVec2/3/4`, `UVec2/3/4`, `Mat2/3/4`, `Quat`, `Transform`, `Aabb`, `Sphere`, `Ray`, `Plane`, `Frustum`, `KahanSum`, `det` (`[DET-4]`) |
| `std.simd`, `std.cpu` | Part XII; `prefetch`, `pause`, `cycle_counter`, `cache_line_size`, `logical_cores` |
| `std.arena` | `Arena`, `FixedArena`, `ScopedArena`, `ThreadArena` |
| `std.io`, `std.fs`, `std.path` | `stdin`, `stdout`, `stderr`, `Read`/`Write`, `BufReader`, `BufWriter`, files, directories, `Path`, `PathBuf`, `io.Error` |
| `std.time` | `Instant`, `Duration`, `SystemTime`, `sleep` |
| `std.process`, `std.env` | `args`, `args_os`, `exit`, `abort`, `Command`; `var`, `vars`, `current_dir` |
| `std.random` | `Rng` (seeded, deterministic), `Rng.from_entropy()` (`Nondet`) |
| `std.thread`, `std.sync`, `std.jobs` | Part XI |
| `std.ecs` | Part XII |
| `std.ser` | Part XIV |
| `std.testing`, `std.debug` | `assert_approx_eq`, `expect_panic`, the `bench` harness (Part XVII); `backtrace`, `leak_report`, `alloc_stats`, and profiler zones `with debug.zone("name"):` |
| `std.ffi` | `CString`, `cstr`, `c_int` and the other C aliases, `c_wchar`, `WideString`, `ForeignBox`, `Retained`, `Callback`, `adopt` (Part XVI) |
| `std.gpu` | Annex D |

## XV.2 Console output, input and formatting

* `[STD-9]` *(new in 0.9.9)* `print(a, b, …, sep=" ", end="")` and `println(a, b, …, sep=" ",
  end="\n")` write their arguments' text to standard output separated by `sep`;
  `eprint`/`eprintln` write to standard error. They are compiler-known and are the only calls with a
  variable number of positional arguments (`[TYP-26]`). An argument's text is its `Display` text when
  its type has `Display`, and its `Debug` text otherwise, which is Python's `str` → `repr` fallback; a
  type with neither (a class without them) is `E2040`, whose help is to implement `Display`. `println()` prints an empty line.
* `[STD-2]` Printing allocates only to build an f-string argument; `println("literal")`,
  `println(some_str)` and `println(n)` for a number are `@noalloc`.
* `[STD-10]` *(new in 0.9.9)* `input(prompt="") -> String` prints the prompt, flushes, reads one line
  and returns it without the line ending. The prelude's console functions treat a console failure as
  fatal, as Python does when the exception is not caught: end of input or a closed stream panics, and
  the message names the `Result` forms in `std.io` (`stdin().read_line()`).
* `[STD-18]` *(new in 0.9.9)* `format(value, spec="") -> String` formats one value with a format spec
  (`[LEX-19]`), like Python's `format`. `format_to(buf: MutSpan[u8], …) -> Result[str, FmtError]` formats
  into a caller's buffer and never allocates. An f-string lowers to a `StringBuilder` sized from its
  literal parts.

## XV.3 Text

* `[TXT-1]` `str` is a borrowed `Span[u8]` known to be valid UTF-8; `String` owns a heap buffer with the
  same guarantee. Safe code may rely on the invariant (`[UNS-4]`).
* `[TXT-2]` Nothing becomes a `str` without validation: every conversion from bytes or foreign text
  returns `Result[str, Utf8Error]`, or a `_lossy` form that substitutes U+FFFD. An importer or binding
  that would hand foreign bytes to a `str` without validation is `E5063`.
* `[TXT-3]` `str` and `String` are a pointer and a length, not null-terminated, and may contain `\0`. A
  C boundary uses `cstr`/`CString` (Part XVI); a `c"…"` literal containing a NUL is `E5064`.
* `[TXT-4]` *(changed in 0.9.9)* Slicing a string, `s[a..b]`, is by byte offset and panics in every
  profile when a bound is not on a character boundary; `s.get(a..b) -> Option[str]` is the total form.
  There is no indexing by character position, because it would be O(n).
* `[TXT-5]` `str` → `String` allocates and copies; `String` → `str` is free; `Span[u8]` → `str`
  validates in O(n) without allocating; `str` → `CString` allocates.
* `[TXT-6]` Across the C ABI a `str` is `{const uint8_t *ptr; size_t len}`.
* `[TXT-7]` Text is UTF-8 everywhere. `std.ffi.WideString` converts, explicitly, for APIs that need
  UTF-16.
* `[TXT-8]` `str` is a view type and carries a region, like every `Span`.
* `[TXT-9]` *(new in 0.9.9)* **A string literal initialises a `String`.** Wherever a `String` is
  expected — an initialiser, an argument, a field, a collection element, a return value — a string
  literal produces a new `String` holding its text. The allocation happens at the literal and is
  reported there (`Alloc`); where a `str` is expected, a literal is a static `str` and allocates
  nothing. A `str` *value* never converts implicitly: `s.to_string()` or `String.from(s)` is written.
* `[TXT-10]` *(new in 0.9.9)* **`str` operations.** `len()` is the length in bytes and
  `char_count()` in characters; `is_empty`, `chars`, `char_indices`, `bytes`, `as_bytes`, `lines`,
  `split(sep)`, `split_whitespace()` (Python's `split()` with no argument), `split_once(sep) ->
  Option[(str, str)]`, `partition(sep) -> (str, str, str)` (Python's), `trim`, `trim_start`, `trim_end`,
  `starts_with`, `ends_with`, `find(sub) -> Option[int]`, `rfind`, `count(sub)`, `replace(old, new)
  -> String`, `to_upper`, `to_lower`, `repeat(n)`, `parse[T]() -> Result[T, ParseError]`,
  `to_string`, `get(range)`, `is_char_boundary(i)`, and `x in s` (`[STD-8b]`). Methods returning
  several strings return iterators of `str` borrowing `s`. Case mapping and `char_count` are
  Unicode-aware; nothing depends on the locale. `parse[T]()` (ODR-029) reads the whole text as an
  integer type, a float type, `bool` or `char`, skipping no white space (`s.trim().parse[int]()`): an
  integer is an optional sign (`-` only for a signed type) and ASCII decimal digits, and must fit `T`;
  a float is an optional sign and `inf`, `infinity` or `nan` in any case, or decimal digits with an
  optional `.`, fraction and exponent (`e` or `E`, an optional sign, digits), rounded to the nearest
  `T`; a `bool` is `true` or `false`; a `char` is exactly one character. `ParseError`, in `std.string`,
  is a unit-only enum, `Empty`, `Invalid` or `Overflow`, naming the first problem from the left.
* `[TXT-11]` *(new in 0.9.9)* **`String` operations.** Everything `str` has (by read-through), plus
  `String()`, `with_capacity(n)`, `push(c: char)`, `push_str(s)`, `insert`, `remove`, `truncate`,
  `clear`, `capacity`, `reserve`, and `as_str`. `a + b` for `a: String, b: str` consumes `a` and
  returns it extended; `s += t` appends. `str + str` is `E2040` whose help is an f-string.

## XV.4 Collections

* `[STD-15]` *(new in 0.9.9)* **`Array[T]`** is a growable contiguous list (Python's `list`).
  `len`, `is_empty`, `capacity`, `reserve`, `push`, `pop -> Option[T]`, `insert(i, v)`, `remove(i) -> T`,
  `swap_remove(i) -> T`, `extend(iterable)`, `truncate`, `clear`, `retain(pred)`,
  `drain(r) -> Array[T]`, `dedup`, `reverse`, `sort` (stable; `T: Ord`; an inconsistent order
  permutes but never corrupts, `[HASH-3]`), `sort_by_key[K: Ord](f: fn(T) -> K)`,
  `sort_by(cmp: fn(T, T) -> Ordering)`, `sorted() -> Array[T]`, `binary_search(x) -> Result[int, int]`,
  `index_of(x) -> Option[int]`, `first`, `last`, `get(i) -> Option[ref T]`, `get_mut`, `iter`,
  `iter_mut`, `split_at`, `chunks(n)`, `chunks_mut(n)`, `windows(n)`, `join(sep)` (for elements that
  are `Display`), and `x in xs` (`[STD-8]`). `xs[i]` takes any integer and panics when `i < 0` or
  `i >= len` (`[TYP-31]`). `sort_by` and `sort_by_key` are stable, and `sort_by_key` calls `f` once
  for each element, in order. `windows(n)` yields every run of `n` neighbours, one step apart, as
  shared views (there is no `windows_mut`); none when `n > len`, and `n == 0` panics as `chunks(0)`
  does (`[SPN-4]`). `drain(r)` moves the elements in `r`, any range of integers (`a..b`, `a..=b`,
  `a..`, `..b`), out into a new `Array` in order, and the rest close up; a range reaching outside
  `0..=len`, or starting after it ends, panics.
* `[STD-11]` *(new in 0.9.9)* **`Map[K, V, H = DefaultHasher, A = Global]`** is a hash map that **iterates in
  insertion order**, like Python's `dict`. Re-assigning an existing key keeps its position; removing
  a key keeps the order of the others. Lookup, insertion and removal take expected constant time,
  amortised: a map may grow, or close up removed entries, inside an insertion or a removal;
  iteration is linear in the number of entries. `H: Hasher + Default`, and a map makes a fresh `H`
  for each hash; `A` is its allocator (`[ALC-1]`). `Set[T, H = DefaultHasher, A = Global]` is the
  same with no values. Keys, values and elements are not view types (`E3063`, whose help names
  `String`): a view inside would make the map a view (`[TYP-34]`), confined to locals (`[TYP-15]`). Because order is fixed by the program's operations and the default hasher is
  fixed-seed (`[HASH-2]`), iterating a `Map` or `Set` is not `Nondet`.
* `[STD-16]` *(new in 0.9.9)* **`Map` operations.** `m[k]` reads the value and panics when the key is
  missing (`key not found: <Debug of k>; use .get(k) for an Option`); `m[k] = v` inserts or replaces
  (`[STD-17]`); `m[k] += 1` requires the key to exist and panics with the same message. The
  lookups take any `q: Q` with `Q: AsKey[K]` (`[STD-12]`): `get(q) -> Option[ref V]`,
  `get_mut(q) -> Option[ref mut V]`, `get_or(q, default) -> V` (for `V: Copy`),
  `contains_key(q) -> bool`, `remove(q) -> Option[V]`, `m[q]` and `q in m`. `insert(k, v) ->
  Option[V]` and `entry(k)` take the key itself; `entry(k)` is a `MapEntry` with `or_insert(v)`,
  `or_insert_with(f: fn() -> V)` and `or_default()` (for `V: Default`), each returning `ref mut V`.
  `keys()`, `values()`, `values_mut()` and `items()` are the view iterators
  `MapKeys`, `MapValues`, `MapValuesMut` and `MapItems`, yielding `ref K`, `ref V`, `ref mut V` and
  `(ref K, ref V)` in insertion order (`for k, v in m.items():`); `for k in m:` is by key, as
  Python's `dict`. Also `Map[K, V]()`, `with_capacity(n)`, `len`, `is_empty`, `capacity`,
  `reserve(n)`, `try_reserve(n)`, `clear`, `retain(keep: fn(ref K, ref V) -> bool)`,
  `pop_item() -> Option[(K, V)]` (the newest entry, as `dict.popitem`), `update(other)` (inserts
  `other`'s entries in its order) and `into_items() -> Array[(K, V)]` (consumes the map, in order).
  `m1 == m2` compares as sets of entries: the same length, and every key of one in the other with an
  equal value; order does not matter. `sorted(m)` sorts the keys into an `Array[K]` of clones.
  `Set`: `Set[T]()`, `with_capacity(n)`, `len`, `is_empty`, `capacity`, `reserve(n)`,
  `try_reserve(n)`, `clear`; `add(x) -> bool` (whether `x` was new; an element already there keeps
  its position); taking `q: AsKey[T]`, `remove(q) -> bool`, `contains(q) -> bool` and `q in s`;
  `retain(keep: fn(ref T) -> bool)`, `pop() -> Option[T]` (the newest), `iter()` (a `SetIter`
  yielding `ref T`); `union`, `intersection`, `difference` and `symmetric_difference` of a borrowed
  `Set` return a new `Set` (for `T: Clone`), this set's elements first in its order and then the
  other's; `is_subset`, `is_superset`, `is_disjoint`; and the operators `|`, `&`, `-` and `^`.
  `s1 == s2` compares as sets.
* `[STD-12]` *(new in 0.9.9)* **Borrowed keys.** Lookup methods, `m[k]` and `k in m` accept any key
  type `Q` with `Q: AsKey[K]`: `K` itself, and `str` for `K = String`. `interface AsKey[K]: Hash` (in
  `std.collections`, not the prelude) has `is_key(self, key: K) -> bool` and `to_key(self) -> K`. A
  lookup hashes `q` and compares it with `is_key`, so it never converts; `m[k] = v` converts with
  `to_key()` only when the key is new. Every `K: Eq + Hash` is `AsKey[K]` (`is_key` is `==`;
  `to_key` clones, so `m[k] = v` with a `K` needs `K: Clone`, where `insert` moves the key); std
  implements `AsKey[String]` for `str`. An implementation MUST hash and compare as `to_key()` would;
  one that does not gives `[HASH-3]`'s wrong answers, never undefined behaviour.
* `[STD-17]` *(new in 0.9.9)* **Index assignment.** `a[i] = v` calls `IndexSet.index_set(i, v)` when
  the type implements `IndexSet[Idx, V]`, and otherwise assigns through `IndexMut` (dropping the old
  value). `Map` implements `IndexSet`; `Array` does not need to. A compound assignment `a[i] op= v`
  always goes through `IndexMut`.
* `[STD-7]` Fixed-capacity containers with no heap allocation: `FixedArray[T, N]`, `FixedString[N]`
  and `RingBuffer[T, N]`; `push` returns `Result[void, CapacityError]` and `push_or_drop` discards when
  full. Also `Deque[T]`, `SmallArray[T, N]` (inline up to `N`, then heap), `BitSet`.
* `[STD-8]` **`Contains`.** `x in c` requires `c: Contains[typeof(x)]` (`E2226` otherwise, whose help
  is `c.iter().any(fn(e) => e == x)`). `std` implements it for `Set` and `Map` (by key); for `Array`,
  `Span`, `MutSpan` and `[T; N]` with `T: Eq`, as a linear scan; for `str`/`String`, as a substring or
  character test; for ranges, as two comparisons. The compiler never synthesises an implementation.
* `[STD-8a]` `ember inspect --cost` reports which `contains` a use selects and its complexity, so
  `x in array` is visibly linear and `x in set` visibly constant.
* `[STD-8b]` `str` implements `Contains[char]` and `Contains[str]` only; matching happens at character
  boundaries, so a character never matches inside another's encoding. A byte needle (`Span[u8] in s`)
  is `E2226`; byte search is on `s.as_bytes()`. `x not in c` is `not c.contains(x)`, with each operand
  evaluated once.

## XV.5 Iterators

* `[STD-19]` *(new in 0.9.9)* Every `Iterator` has the adapters `map`, `filter`, `filter_map`,
  `enumerate`, `zip`, `chain`, `take`, `skip`, `take_while`, `skip_while`, `step_by`, `flat_map`,
  `flatten`, `peekable`, `copied`, `cloned`, `inspect`, and `rev` where the iterator can run backwards;
  and the consumers `count`, `sum`, `product`, `min`, `max`, `min_by_key`, `max_by_key`, `fold`,
  `reduce`, `any`, `all`, `find`, `position`, `last`, `nth`, `min_by`, `max_by`, `for_each`,
  `collect[C]()`, `to_array()`,
  and `join(sep)` for `Display` items. Adapters are lazy and allocate nothing; a chain of adapters in a
  `for` loop compiles to one loop with no iterator object in memory (`[CTL-3]`). The adapters and
  consumers can also be called directly on an `Iterable` value, borrowing it as `iter()` would:
  `xs.enumerate()` is `xs.iter().enumerate()`, and a method of the collection's own of the same name
  (`Array.join`) takes precedence.
* `[STD-5]` *(changed in 0.9.9)* `sum` and `product` combine elements left to right in the element
  type; an empty sum is zero. Integer sums panic on overflow (`[TYP-8]`). Float sums are never
  reassociated outside `@fastmath` (`[TYP-9]`), so they are deterministic; `sum_f64()` accumulates
  `f32` elements in `f64`, and `math.KahanSum` gives compensated summation.

## XV.6 Numbers and math

* `[STD-20]` *(new in 0.9.9)* Integer methods: `abs`, `pow`, `signum`, `div_trunc`, `rem_trunc`,
  `checked_*`, `wrapping_*`, `saturating_*`, `overflowing_*` for each arithmetic operator,
  `count_ones`, `leading_zeros`, `trailing_zeros`, `is_power_of_two`, `next_power_of_two`, and the
  constants `MIN` and `MAX`. Float methods: `abs`, `sqrt`, `floor`, `ceil`, `trunc`, `fract`,
  `round` (**half to even**, as Python's `round` and IEEE's default), `round_half_away` (C's `round`),
  `is_nan`, `is_finite`, `is_infinite`, `copysign`, `mul_add`, and the constants `INF`, `NAN`, `EPSILON`,
  `MIN`, `MAX`. A float's `Display` is the shortest text that reads back as the same value, with `.0`
  for an integral value, as Python's `repr`: `0.1 + 0.2` prints `0.30000000000000004`, `1.0` prints
  `1.0`, and `1e300` prints `1e+300`; a format spec overrides it.
* `[STD-3]` `math.fma(a, b, c)` (and `mul_add`) is a fused multiply-add with a single rounding, on every
  target; it is how fused arithmetic is written without `@fp(contract)`. `std.math` uses it for matrix
  products and transforms.
* `[STD-4]` `NonZero[T]` for each integer `T` is a `Copy` wrapper with a niche (`Option[NonZero[T]]` is
  `T`-sized), made by `NonZero.new(v) -> Option[NonZero[T]]`. Dividing by a `NonZero` needs no
  division-by-zero check.
* `[STD-21]` *(new in 0.9.9)* `std.math` provides `PI`, `TAU`, `E`; `sin`, `cos`, `tan`, `asin`, `acos`,
  `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `exp`, `exp2`, `ln`, `log2`, `log10`, `pow`, `sqrt`, `rsqrt`, `cbrt`,
  `hypot`, `lerp`, `smoothstep`, generic over `f32` and `f64`; and the vector types `Vec2`, `Vec3`,
  `Vec4` (of `f32`, `@layout(c)`), `IVec2/3/4`, `Mat3`, `Mat4` (column-major), `Quat`, `Transform`,
  `Aabb`, `Ray`, `Plane`, `Frustum`, with `dot`, `cross`, `length`, `normalize` (panics on a zero
  vector; `normalize_or_zero` does not) and the arithmetic operators.

## XV.7 I/O, files, time and processes

* `[STD-22]` *(new in 0.9.9)* Every operation that touches the outside world returns `Result[T,
  io.Error]` and carries `Io`: `io.stdin().read_line() -> Result[Option[String], io.Error]` (`None` at
  end of input), `lines()`; `fs.read(path) -> Result[Array[u8], io.Error]`, `fs.read_to_string`,
  `fs.write(path, data)`, `fs.append`, `File.open`, `File.create`, `fs.exists`, `fs.remove`,
  `fs.create_dir_all`, `fs.read_dir`, `fs.metadata`. A `File` closes when dropped; `BufReader` and
  `BufWriter` buffer, and `BufWriter` flushes when dropped.
* `[STD-23]` *(new in 0.9.9)* `time.Instant.now()` is monotonic and `Nondet`; `Duration` is an integer
  count of nanoseconds with checked arithmetic; `time.sleep(d)` has `Block`.
* `[STD-24]` *(new in 0.9.9)* `process.exit(code) -> Never` flushes the standard streams and exits
  without running destructors or `defer` blocks; returning from `main` is the clean exit.
  `process.args()` and `args_os()` follow `[FN-8]`; `env.var(name) -> Option[String]`.
* `[STD-25]` *(new in 0.9.9)* `random.Rng.seeded(seed)` is a deterministic generator (the same seed
  gives the same sequence on every machine); `Rng.from_entropy()` seeds from the system and is `Nondet`.
  `rng.int_in(a..b)`, `rng.float()` (in `[0, 1)`), `rng.choice(span) -> Option[ref T]`,
  `rng.shuffle(mut_span)`.
---

# Part XVI — Calling C

Being as fast as C includes using C's libraries with no glue. Ember calls a C function with one
direct call, lays out `@layout(c)` types exactly as the C compiler does, and reads C headers
directly. What a header cannot say — who owns a pointer, how many elements it points to, whether it
may be null — is stated once, in a **contract**, after which calls are safe. C++ interoperation is
Annex C.

```ember
from std.ffi import c_int, c_double

unsafe extern "C":
    @ffi(effects=[])
    safe fn abs(x: c_int) -> c_int
    @ffi(effects=[])
    safe fn cos(x: c_double) -> c_double

fn main():
    println(abs(-5), cos(0.0))
```

## XVI.1 Principles

* `[FFI-1]` *(changed in 0.9.9)* Every foreign declaration has a **contract**: for each pointer-typed
  parameter and result, its ownership, mutability, nullability, count and lifetime (§XVI.4), and for
  the function its effects, its thread rule and whether it is **safe to call** at all. Facts derivable
  from the C declaration are derived (`[FFI-2a]`); the rest are `unknown` until a contract supplies
  them.
* `[FFI-2]` *(changed in 0.9.9)* A call to a foreign function whose contract contains an `unknown` fact
  requires `unsafe`. Supplying the facts, in an `@ffi(…)` attribute or an overlay (§XVI.4), makes the
  call safe. The safe-to-call fact is never derived, because a scalar can be an address
  (`void free_addr(uintptr_t)`): it is asserted by `safe fn` in an extern block (`[FFI-10]`), or by an
  overlay entry for the function, which asserts that a call within its contract cannot break memory
  safety; such an overlay is an `unsafe overlay` (`[GRM-35]`). A header function with no overlay entry
  therefore stays unsafe to call. The diagnostic,
  `E5002`, names each missing fact and the overlay line that supplies it (`[DIA-18]`).
* `[FFI-2a]` Derived facts need no `unsafe`: `const T*` is a read-only borrow, `T*` a mutable one; the
  layouts of structs and unions, enum underlying types, packing and alignment, and the calling
  convention. In **result** position a pointer's lifetime is never derived.
* `[FFI-4]` No Ember panic crosses into foreign code (`[FFI-25]`).
* `[FFI-5]` Layouts are verified, not assumed: the importer records `sizeof`, `alignof` and field
  offsets from Clang for the project's target and flags, and the compiler checks its own layout
  against them (`E5001`, naming the type, field and both numbers).
* `[FFI-5a]` The generated shim (`[FFI-29]`) contains a `_Static_assert` for the size and alignment of
  every imported aggregate Ember constructs, passes, returns or embeds, and for the offset of every
  field Ember accesses, so the project's own C compiler re-verifies the layout on every build.

## XVI.2 Declaring C functions by hand

* `[FFI-10]` *(changed in 0.9.9)* An `unsafe extern "C":` block declares foreign functions, statics
  and opaque types; `unsafe` records that the programmer asserts the signatures. The declared names
  are items of the enclosing module. A function in the block is **safe to call** only when it is
  declared `safe fn` (a contextual keyword, `[LEX-15]`) and every parameter and result is a scalar, a
  `@layout(c)` value type, or a pointer whose contract is complete (given with `@ffi(…)` on the
  declaration); `safe` on any other function is `E5002`, naming the fact that is missing. Every other
  call is in `unsafe`. `safe fn` is an asserted fact, listed by `ember tcb` (`[TCB-1]`). Its effects are
  those its `@ffi(effects=[…])` states plus `FFI`, and every effect when none are stated (`[EFF-3]`).
* `[FFI-9]` `extern "C"` is the platform C ABI (SysV AMD64, Windows x64, AAPCS64). `extern "system"`
  is `stdcall` on 32-bit Windows and C elsewhere; `extern "vectorcall"` and `extern "fastcall"` exist
  on x86. Struct arguments and results follow the ABI exactly. An ABI-direct call is one `call`
  instruction and nothing else.
* `[FFI-49]` *(new in 0.9.9)* `@ffi(link_name="sym")` binds a declaration to a differently named symbol.
  Libraries are linked by `[link] libs = ["m", "vulkan-1"]` in the manifest or `link = [...]` on an
  `import c` (`[BLD-FFI-3]`).

## XVI.3 Importing C headers

```ember
import c "vulkan/vulkan.h" with (
    include_paths = ["vendor/Vulkan-Headers/include"],
    defines = ["VK_NO_PROTOTYPES"],
    overlay = "overlays/vulkan.em",
) as vk
```

* `[FFI-6]` `import c "header" with (…) [as name]` parses the header with libclang under the project's
  target, language standard, defines and include paths, and exposes its functions, structs, unions,
  enums, typedefs, globals and constant macros as a module (`c`, or `name`). Options: `include_paths`,
  `defines`, `link`, `overlay`, `implementation` (`[FFI-29b]`). An unknown option is `E0104`.
* `[FFI-6a]` An object-like macro is imported when, after expansion, it is a constant expression of
  integer, floating, string, null-pointer or pointer-cast type as Clang evaluates it; its type is the
  C type of that value. Any other macro is skipped with `W5001`, giving Clang's reason.
* `[FFI-6b]` A function-like macro is exposed only when an overlay declares its signature
  (`@ffi(macro_fn)`); the importer emits a C wrapper function for it into the shim. Ember gains no
  macro facility. When every argument is a constant the importer may fold the call, so the result can
  initialise a `const`.
* `[FFI-7]` A header is re-parsed only when its content, its overlay, or a setting that can change a
  binding's meaning or ABI changes (defines, target, language standard, packing, the C runtime); the
  result is cached as a `.embind` file (`[FFI-14]`).
* `[FFI-20a]` A declaration the importer cannot represent is recorded with the construct that defeated
  it and a workaround, and reported as `W5002`. A reference to it is a diagnostic saying it was skipped
  and why, never "unknown identifier". No declaration is silently absent.
* `[FFI-38]` The importer rejects rather than guesses: a fact it cannot establish leaves the
  declaration uncontracted (its calls need `unsafe`) and is reported; convenient-but-unsound is never
  an import mode.
* `[FFI-8]` *(changed in 0.9.9)* **Type mapping.**

  | C | Ember |
  |---|---|
  | `_Bool`, `char`, `signed/unsigned char` | `bool`, `c_char` (`i8` or `u8` per target), `i8`, `u8` |
  | `short`, `int`, `long`, `long long` and unsigned forms | `i16`, `c_int`, `c_long`, `i64`, and `u16`, `c_uint`, `c_ulong`, `u64` |
  | `size_t`, `ptrdiff_t`, `intptr_t`, `uintptr_t` | `usize`, `isize`, `isize`, `usize` |
  | `float`, `double`, `long double` | `f32`, `f64`, `c_longdouble` (opaque bytes; no arithmetic) |
  | `__int128`, `unsigned __int128`, `_Float16` | `i128`, `u128`, `f16` |
  | `wchar_t`, `char16_t`, `char32_t` | `c_wchar` (`u16` on Windows, `i32` elsewhere — not portable), `u16`, `u32` (C may hand back a surrogate or a value above U+10FFFF; `char.from_u32(v) -> Option[char]` converts) |
  | `volatile T*` | `Volatile[*T]` with `read_volatile`/`write_volatile` |
  | `__attribute__((packed))`, `#pragma pack`, `alignas` | `@packed`, `@align(N)` |
  | `T*`, `const T*` | `*mut T`, `*T`; with a contract, `ref`, `MutSpan`/`Span`, `Option[…]`, `ForeignBox[T]`, `cstr` |
  | `T[N]` field | `[T; N]` |
  | complete `struct`, `union` | `@layout(c) struct` (`Copy` when every field is plain data); a union's fields are read only in `unsafe` |
  | incomplete `struct` | `extern type S`, used only behind pointers |
  | `enum` | `@repr(<underlying>) enum`, `@non_exhaustive` (any value may arrive) |
  | `typedef` | a type alias, or a new type with `@ffi(newtype)` |
  | `static const` global, `extern` global | `extern static`; reading it needs `unsafe` unless `@ffi(immutable)` |
  | `#define S "text"` | `const S: cstr = c"text"` |
  | function pointer | `extern "C" fn(…) -> R`, `Option[…]` when nullable; another calling convention is `extern "<conv>" fn` (`[FFI-9]`), and an unsupported one is skipped with `W5002` |
  | bit-field | `get_<f>`/`set_<f>` methods; not addressable |
  | anonymous member | a generated type `<Parent>_anon<N>` with transparent field access |
  | flexible array member | an unsized struct (never by value, `E5017`) with `unsafe fn tail(self, n) -> Span[T]` |
  | `_Atomic T` | `Atomic[T]` when layouts match, else `E5016` |
  | `va_list`, variadic `...` | `VaList`, forwardable only and never constructed (`E5018`); variadic calls only in `unsafe` with explicit argument types |
  | an identifier that is an Ember keyword | a raw identifier `r#type` (`[LEX-14]`) |

* `[FFI-29]` For every `import c` the importer emits a **shim** C file, compiled with that import's
  flags and linked into the package, containing the layout assertions (`[FFI-5a]`), an external
  wrapper for every `static inline` function Ember calls, and the wrappers of `[FFI-6b]`.
* `[FFI-29a]` A binding never names a symbol with internal linkage.
* `[FFI-29b]` `implementation = ["MINIAUDIO_IMPLEMENTATION"]` compiles a single-header library's
  implementation exactly once per build; two packages requesting it for one header is `E5014`.
* `[FFI-29c]` A call to a wrapped `static inline` function costs one extra call unless the shim takes
  part in LTO; `ember inspect` reports it as `wrapped (static inline)`. A variadic `static inline`
  function cannot be wrapped and is skipped with `W5002`.
* `[FFI-30]` Two imports of one C declaration (by Clang's USR, through typedefs) with the same layout
  are **one** Ember entity, whichever module, alias or overlay imports them; the same declaration with
  different layouts in one build is `E5011`, whose help is to give one import its own alias, making its
  entities distinct.
* `[FFI-30a]` An overlay changes the signatures, safety and names of *functions* and adds wrappers; it
  never changes a type's identity, layout or fields, so packages with different overlays for one header
  still exchange its values.
* `[FFI-30b]` `pub import c "…" as vk` re-exports the module under the ordinary visibility rules; the
  package's interface hash (`[BLD-2]`) includes the imported layouts, so a header change rebuilds
  dependants.
* `[FFI-30c]` A package may distribute an overlay for a foreign module, and several overlays for one
  module compose in a declared order; they may add aliases and wrappers and replace an entry only with
  an explicit `override`. Incompatible entries are an error, never a silent choice, and no overlay may
  change a foreign type's identity or layout.

## XVI.4 Contracts and overlays

An **overlay** is an Ember file that states contracts for the declarations of a header without
editing it.

```ember,overlay
unsafe overlay c "vulkan/vulkan.h":
    @ffi(handle)
    type VkBuffer
    @ffi(handle)
    type VkDevice
    @ffi(status, ok=VK_SUCCESS)
    enum VkResult

    @ffi(effects=[FFI, Alloc])
    fn vkCreateBuffer(device: borrowed, pCreateInfo: borrowed one,
                      pAllocator: nullable borrowed one, pBuffer: out one) -> status

    fn vkCmdSetViewport(commandBuffer: borrowed, firstViewport, viewportCount,
                        pViewports: borrowed count(viewportCount))

    rename VkPhysicalDeviceFeatures2 as PhysicalDeviceFeatures2
    hide vkAllocationFunction
```

* `[GRM-35]` *(new in 0.9.9)* **Overlay grammar.** An overlay is a file whose items are:

  ```text
  overlay_decl  := ["unsafe"] "overlay" ("c" | "cpp") STRING ":" NEWLINE INDENT {overlay_item} DEDENT
  overlay_item  := {attribute} (overlay_fn | "type" IDENT NEWLINE | "enum" IDENT NEWLINE
                   | "rename" IDENT "as" IDENT NEWLINE | "hide" IDENT NEWLINE | extend_decl)
  overlay_fn    := "fn" IDENT "(" [overlay_param {"," overlay_param} [","]] ")"
                   ["->" contract {contract}] NEWLINE
  overlay_param := IDENT [":" contract {contract}]
  contract      := IDENT ["(" [attr_arg {"," attr_arg}] ")"]
  ```

  Contract words take the place of types and are separated by spaces. `overlay`, `rename` and `hide`
  are contextual keywords (`[LEX-15]`). An overlay whose items state any fact — a contract word, an
  `@ffi(…)` attribute, or a function entry, which asserts the function safe to call (`[FFI-2]`) — is an
  assertion about foreign code and MUST be declared `unsafe overlay` (`E5067`), the third boundary of
  `[TIER-1]`; an overlay of only `rename` and `hide` items needs no `unsafe`.
* `[FFI-11]` *(changed in 0.9.9)* **Contract vocabulary.** A pointer contract has five axes;
  mutability comes from `const` and is never written.

  | Axis | Words | Ember type produced |
  |---|---|---|
  | ownership | `borrowed` (valid for the call), `owned` (callee takes it), `returns_owned(destructor=f)`, `retained` (callee keeps it, `[FFI-23]`) | `ref`/`Span`; a moved `ForeignBox[T]`; a `ForeignBox[T]` whose drop calls `f`; `Retained[T]` |
  | count | `one`, `count(n)` (as many elements as parameter `n`, which then leaves the signature), `nul_terminated`, `fixed(N)`, `inout_count(p)` (`[FFI-11b]`) | `ref T`, `Span[T]`/`MutSpan[T]`, `cstr`, `ref [T; N]` |
  | nullability | `nullable` | `Option[…]` |
  | lifetime (results) | `from(self)`, `from(p)`, `from(static)` | the region of the receiver, of `p`, or static (`[LT-1a]`) |
  | aliasing | `exclusive`, `aliased` | `ref mut` allowed only with `exclusive`; a result is `aliased` unless stated |

  Other contract words: `out` (an out-parameter, returned instead); `status` (with `@ffi(status,
  ok=X)` on the enum, `[FFI-16]`); `handle`; `callback(borrowed | retained | once, user_data=p)`
  (`[FFI-21]`); `threads(any | main | creator)`; `destroys(f)`; `unsafe` (leave the call unsafe).
  `@ffi(effects=[…])` states effects.
* `[FFI-11a]` A pointer contract with no count word is `E5012`, which lists the five count words and
  names any sibling parameter that looks like a length (an unsigned integer, or a name ending `Count`,
  `Len`, `Size` or `N`).
* `[FFI-11b]` The two-call enumeration idiom (call once for the count, again to fill) is `pCount:
  inout_count(pItems)` with `pItems: nullable count(pCount)`; the Ember function takes
  `Option[MutSpan[T]]` and returns the count.
* `[FFI-11c]` `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` and `TODO(lifetime)` are contract words
  that mark a fact as still unknown: the declaration stays uncontracted and its calls stay `unsafe`.
  A header can be adopted on the first day with everything behind `unsafe:`, and made safe one
  function at a time.
* `[FFI-12]` An overlay entry whose name, parameter count or parameter names do not match the header is
  `E5010`, so overlays cannot drift.
* `[FFI-13]` An overlay may contain `extend` blocks with ordinary Ember wrapper methods; this is how an
  idiomatic API is layered over a raw one.
* `[FFI-14]` The importer's result is a `.embind` file (versioned CBOR: header hash, Clang
  configuration, declarations with layouts, overlay hash, contracts), a content-addressed build input.
  `ember bind --explain header.h fn` prints a function's derived contract and why.
* `[FFI-35a]` A parameter with no `retained` word is recorded as "does not retain" in the foreign-trust
  report (`ember bind --report`), which lists every such assumption per module, and every
  result-position pointer with no lifetime word.

## XVI.5 Strings, status codes and handles

* `[FFI-15]` `cstr` is a borrowed, NUL-terminated C string (`c"…"` literals are `cstr`); `CString`
  owns one. `s.to_cstring()` allocates and fails on an interior NUL; `c.to_str() -> Result[str,
  Utf8Error]` validates without copying. Passing a `str` where a `cstr` is expected is `E5020`, whose
  fix-its are `.to_cstring()` and a `c"…"` literal.
* `[FFI-16]` An enum marked `@ffi(status, ok=X)` generates `struct <E>Error(code: E)` implementing
  `Error`; every function contracted `-> status` returns `Result[T, <E>Error]`, where `T` is the tuple
  of its `out` parameters (or `void`).
* `[FFI-36a]` `std.ffi.adopt[T](h) -> ForeignBox[T]` is an `unsafe fn` that takes ownership of a foreign
  object, reading its destructor from the contract of `T`'s declaration. Where the contract already
  records the transfer (`returns_owned(destructor=f)`), the import returns a `ForeignBox[T]` directly and
  no `adopt` is written.
* `[FFI-36b]` Adopting a type whose overlay says `adopt = false` is `E5052`. Adopting one live object
  twice is caught at run time in every profile (a panic naming both sites) and, when both calls are on
  one local in one function, at compile time.

## XVI.6 Callbacks and foreign retention

* `[FFI-21]` *(changed in 0.9.9)* A C function-pointer parameter accepts a capture-free Ember function
  (as an `extern "C" fn`) or an `extern "C" fn` value. A capturing closure is `E5040` unless the
  parameter's contract names a `user_data` parameter, in which case the importer generates the
  trampoline: the closure is boxed and passed as `user_data`. With `callback(borrowed)` the box is freed
  after the call and the closure may capture anything a call argument may. With `callback(retained)`
  or `callback(once)` the closure outlives the call, so it MUST be an `owned fn` whose captures carry
  only the static region (`E3063`); `retained` returns a `Retained` token whose drop unregisters
  through the declared `destroys(f)` (`E5041` if none is declared), and with `once` the trampoline
  frees the box after the first call.
* `[FFI-22]` *(changed in 0.9.9)* Every exported function and trampoline first attaches the calling
  thread to the runtime (`ember_rt_thread_attach()`, a thread-local check after the first time). A
  callback's captures, and a handle passed to foreign code as `retained`, MUST be `Send` (`[THR-8]`, `E7004`)
  unless the contract says `threads(main)` or `threads(creator)`: the compiler cannot see which thread C
  will call from, so the contract is the only evidence, and it is an asserted fact (`[TCB-1]`).
* `[FFI-23]` `Retained.pin(v)` hands an Ember-owned object to C for keeping: for a class handle it
  retains the object and sets the header's pinned flag; for a `Box` it takes ownership. `tok.ptr()`
  is the raw pointer and dropping the token releases it. It is the only way to let foreign code keep
  Ember memory.
* `[FFI-33b]` `returns_owned`, `Retained` and callbacks may carry `threads(creator)`: a `ForeignBox`
  records its creating thread and, in `debug`, panics when dropped on another — the rule that makes
  GPU, GL-context and COM handles safe to hold in ordinary values.
* `[FFI-33a]` Attaching a thread (`[FFI-22]`) gives it no right to touch another thread's objects; in
  `debug` a retain or release of a non-`@sync` object from a thread other than its creator panics,
  naming the class and both threads.

## XVI.7 Exporting Ember to C

```ember
@export("game_on_update")
pub extern "C" fn on_update(entity: u64, dt: f32) -> i32:
    return 0
```

* `[FFI-26]` `@export("symbol")` gives a function a stable C symbol with the C ABI.
  `@export_table("Name", protocol=N)` on a struct of `extern "C" fn` fields exports a table of function
  pointers under that name and protocol version, for hosts that load a module through one entry point. `ember build
  --emit-header` writes `<package>.h` with a prototype for each export, a typedef for each
  `@layout(c)` type they use, and each export's thread contract as a comment.
* `[FFI-25]` *(changed in 0.9.9)* A panic in an exported function, or anywhere below it, aborts the
  process (`[PAN-1]`); no panic unwinds into C. `@export(on_panic=abort)` is the only form in this
  version.
* `[FFI-31b]` No owning Ember value crosses a module boundary: class handles, `Box`, `Shared`, `Weak`,
  `String`, `Array`, `Map` and any type with drop glue MUST NOT appear in an exported signature or be
  reachable through a pointer in one (`E5015`). `@layout(c)` values, scalars, opaque handles, `cstr`
  and `extern "C" fn` may.
* `[FFI-31c]` With `[build] runtime = "shared"`, several Ember modules in one process share one runtime
  (hot reload requires it, `[HR-29]`), and the rule of `[FFI-31b]` still applies across their exported boundaries in
  this version.
* `[FFI-33]` `@export(threads=any | main | creator)` states which threads may call an export; the
  default is the file's `#! threads` directive (`[GRM-37]`), else `any`. Under `any`, everything the export reaches is checked as if it ran on any thread: every
  static it reaches is `Sync` and no non-`@sync` class handle is reachable from a static (`E7010`).
* `[FFI-33c]` *(new in 0.9.9)* Under `threads = main`, the exported wrapper checks, in every profile, that
  the calling thread is the one that initialised the module, and panics otherwise; in exchange the body
  may use thread-confined state. An `@export_table` may set `threads` per field.
* `[FFI-27]` The runtime `ember_rt` is a C11 static library with no global constructors. A C host calls
  `ember_rt_init(&cfg)` once (optionally routing allocation and logging to its own functions),
  `ember_rt_thread_attach()` on each thread that calls Ember, and `ember_rt_shutdown()` at the end,
  which runs at-exit hooks and, in `debug`, prints the leak report (`[WK-15]`).
* `[FFI-28]` A package built as `kind = "staticlib"` produces a library and header to link into a C or
  C++ program; `kind = "cdylib"` a shared library exporting only its `@export` symbols plus
  `ember_module_init` and `ember_module_shutdown`.
* `[FFI-31]` A `cdylib` links its own copy of the runtime. The host calls `ember_module_init` before
  using it and `ember_module_shutdown` before unloading it; shutdown runs at-exit hooks, detaches
  every thread the module attached, releases its thread-local state and reports leaks in `debug`.
* `[FFI-31a]` Its thread detachment never relies on a TLS destructor in the module's code.

## XVI.8 Build integration

* `[BLD-FFI-1]` The manifest's `[c]` section names the C compiler and flags used for shims and C source
  files (`import c "./bridge.c"` compiles `bridge.c` and imports `bridge.h`). By default they are the
  compiler Ember uses as its backend.
* `[BLD-FFI-1a]` Each `import c` is parsed and its shim compiled with the package's C flags plus the
  import's own `defines` and `include_paths`, and no others.
* `[BLD-FFI-2]` *(changed in 0.9.9)* The toolchain ships a CMake module: `ember_add_library(name KIND
  staticlib|cdylib SOURCES …)` and `ember_add_executable(…)` build Ember packages as ordinary targets of
  a C or C++ project, taking the target's compiler and flags with `--cc-flags-from-target <target>` or,
  for C++, through the CMake File API (Annex C).
* `[BLD-FFI-3]` Existing libraries are linked by name (`link = ["vulkan-1"]`) or path; link order
  follows declaration order.
---

# Part XVII — Toolchain, Diagnostics and Conformance

## XVII.1 The `ember` command

```text
ember new <name> [--lib | --bin | --cdylib]    create a package
ember run [file.em] [args…] [--hot | --interp] build and run; a single file needs no manifest
ember build [--profile <name>] [--target <triple>] [--backend c]
            [--emit tokens|ast|hir|mir|c|obj|header] [--out-dir <dir>]
            [--emit-optimization-report] [--report=engine|instantiations] [--timings[=json]]
            [--build-id] [--reload [--explain]] [-D warnings]
ember check [--syntax-only] [--timings]        type, borrow and effect checks without code generation
ember test [filter] [--doc] [--no-leak-check]  run @test functions and documentation examples
ember bench [filter] [--compare]               benchmarks; --compare runs the paired C programs
ember fmt [--check | --migrate]
ember lint
ember doc
ember update [package]                         re-resolve dependencies and rewrite ember.lock
ember explain <E-code | rule-id>               the error page or rule text
ember explain --borrow <file>:<line>           why each borrow live at that line is still live
ember explain --cycle <path> <Class[.field]>   one cycle-capable class or field
ember inspect <item> [--safety [--elided-only] | --alloc | --cost | --deterministic | --cycle
                      | --expand]
ember why --alloc|--block|--io|--lock|--sync|--unsafe|--ffi <item>   the call chain to that effect
ember calls --foreign <item>                   every foreign function the item can reach
ember bind <header> [--init [--merge] | --report [--baseline <file> | --write-baseline]
                     | --check <overlay> | --explain <fn> | --emit-embind]
ember shader-bind <reflection.json>            Annex D
ember tcb [--module <m>]                       the trusted-base report (Annex C)
ember toolchain list|install|default           manage installed C toolchains
ember clean
ember --help | --version [--matrix]
```

* `[CLI-1]` Every command accepts `--json` and exits non-zero on error.
* `[CLI-4]` *(changed in 0.9.9)* `ember run file.em` and `ember build file.em` accept a single file
  with no manifest, as a package named after the file with default settings. A first program needs no
  manifest and no `main`: `println("hello")` on its own is a complete program (`[FN-8]`).
* `[CLI-2]` `ember build --emit c --out-dir <dir>` writes the C sources without invoking a C compiler,
  for build systems that compile them themselves. `--emit tokens|ast|hir|mir|obj` write the other
  stages, for tools and for reporting compiler defects.
* `[CLI-9]` `ember check --syntax-only` lexes and parses only, reporting `E00xx` and `E01xx`.
* `[CLI-12]` `ember why --alloc <item>` (and `--block`, `--io`, `--lock`, `--sync`, `--unsafe`, `--ffi`)
  prints the shortest call chain from the item to an operation with that effect, whether or not the
  item carries a contract; with `--unsafe` it prints the obligation of every unsafe function on the
  chain.
* `[CLI-13]` `ember calls --foreign <item>` lists every foreign function the item can reach, with the
  contract of each.
* `[CLI-15]` `ember --help` lists every command and flag of this section; one missing from it, or from
  §XVII.1's listing, is a defect.
* `[CLI-20]` *(new in 0.9.9)* `ember fmt --migrate` rewrites source written for 0.9.8 into 0.9.9 where
  the change is mechanical — `::` to `.`, integer `/` to `//`, `@view` and `with_views` removal,
  `Box[dyn fn]` to an owned callable type, `for k, v in m:` over a `Map` to `for k, v in m.items():`
  (`[CTL-1]`) — and lists what it could not rewrite.
* `[CLI-21]` *(new in 0.9.9)* `ember run --interp` runs a program on the compile-time evaluator instead
  of compiling it: no C compiler is needed; console and file I/O work; foreign calls and threads are
  unavailable (`E6010`); results equal a compiled run's.
* `[TOOL-2]` `ember toolchain install cc` installs a pinned Clang and linker and selects it with `[build]
  c_compiler = "bundled"`; `ember toolchain list` and `default` show and choose among installed ones.
  A working Ember installation never requires a separately installed C toolchain; a system compiler may
  be chosen explicitly, and its configuration is recorded with each build.
* `[TOOL-3]` When no C compiler is found, `E9002`'s message says so and its help names the remedy for
  the host (on Windows, the Visual Studio Build Tools C++ workload, or `ember toolchain install cc`);
  a bare missing path is never the primary message.
* `[CLI-11]` `ember audit` prints a one-page summary of a package's safety surface: `unsafe` blocks
  and their notes, the effect totals, the foreign boundary and its contracts, and the trusted-base
  report of Annex C.
* `[TOOL-1]` Each release publishes a self-contained toolchain archive per supported host (Windows
  x64, Linux x64, macOS arm64) with `ember`, the runtime, `std` and editor support, and a one-line
  installer that puts `ember` on `PATH`.
* `[TOOL-4]` `ember --version` prints the compiler version, the language version, the backend, and the
  resolved C compiler and linker, so a bug report carries its environment. **The first hour:** on a
  clean Windows or Linux machine, installing Ember, `ember new hello`, `cd hello` and `ember run` print
  `hello, world` in under five minutes and at most six typed commands; a regression blocks a release.
* `[CLI-19]` *(new in 0.9.9)* **The implementation matrix.** `ember --version --matrix` prints, for
  every Part, rule family, command and flag of this document, whether this compiler implements it.
  A command, flag, attribute or construct the compiler does not implement is rejected with `E0900`
  naming it (`[PHIL-12]`), never ignored.
* `[CLI-5]` `ember bind --init <header>` writes a starter overlay: every derived contract, every unknown
  fact as a `TODO(…)` word (`[FFI-11c]`), every skipped declaration as a comment with its reason. It
  compiles unchanged and never overwrites a file; with `--merge` it adds only the declarations missing
  from an existing overlay, keeping hand-written contracts and comments.
* `[CLI-6]` `ember bind --report` lists every skipped declaration and every declaration still
  `unsafe` with its unknown facts, and exits non-zero on a skip (`--baseline` limits that to new ones).
* `[CLI-7]` `ember bind --check <overlay>` checks `[FFI-12]` and prints how many declarations are
  safe, still `unsafe`, and skipped.
* `[CLI-10]` `ember build --report=engine` is a report, not a profile: it changes nothing about the
  program, and summarises allocations, surviving count operations and runtime checks with their
  reasons, vectorised loops and why others were not, FFI wrapper costs, inlining decisions, and classes
  that could be structs (advisory; never applied automatically).

## XVII.2 The manifest (`ember.toml`)

```toml
[package]
name = "particles"
version = "0.1.0"
language = "0.9.9"
kind = "bin"                      # bin | lib | staticlib | cdylib
layers = ["core", "alloc", "sync", "io"]
edition_lints = "strict"          # warnings listed in [lints] as "error" fail the build

[dependencies]
geometry = { path = "../geometry" }
parser = { git = "https://example.org/parser.git", rev = "4f2a9c1" }
std = "0.9.9"                     # implicit; may be pinned

[build]
entry = "src/main.em"
target = "native"
backend = "c"
c_compiler = "auto"               # auto | bundled | msvc | clang | gcc
runtime = "static"                # static | shared (shared is required by hot reload)
max_instantiations = 200

[profiles.release]
lto = "thin"

[profiles.profiling]
inherits = "release"
debug_info = "full"

[comptime]
max_steps = 100000000
max_heap_mb = 256

[jobs]
workers = "auto"

[realtime]
contracts = ["noalloc", "nolock", "noblock", "nopanic(explicit)"]

[link]
libs = ["m"]

[lints]
unused = "warn"
potential_cycle = "warn"
large_copy = { level = "warn", threshold = 256 }
```

* `[MAN-1]` *(changed in 0.9.9)* An invalid manifest, including one with an unknown key, is `E9001`,
  naming the key and the keys that section accepts.
* `[MAN-2]` `ember.lock` records the resolved dependencies with content hashes; `--locked` fails if it
  would change.
* `[MAN-3]` *(changed in 0.9.9)* Every key in `[lints]` names a lint the compiler defines (`E9010`
  otherwise); its value is a level (`allow`, `warn`, `error`) or a table with `level` and the lint's
  parameters.
* `[MAN-8]` *(new in 0.9.9)* The sections are: `[package]` (`name`, `version`, `language`, `kind`,
  `layers`, `edition_lints`); `[dependencies]` (a version requirement, `{ path }`, or `{ git, rev }`);
  `[build]` (`entry`, `target`, `backend`, `c_compiler`, `runtime`, `max_instantiations` `[MONO-3]`,
  `reload` `[MAN-7]`); `[profiles.<name>]` (`[PRF-3]`); `[c]` (`[BLD-FFI-1]`); `[link]`; `[jobs]`
  (`[JOB-1]`); `[realtime]` (`[EFF-19]`); `[comptime]` (`max_steps`, `max_heap_mb`, `[CT-3]`);
  `[lints]` (`[MAN-3]`); and `[cpp.<project>]` and `[ffi]` (Annex C). **No manifest key turns off a
  safety check or changes what a program means** (`[PRF-1]`).
* `[MAN-7]` `[build] reload` is `"opt-in"`, `"all"`, `"bodies"` or `"none"`; it is forbidden in
  `shipping` (Annex B).
* `[VER-5]` Package versions are semantic; a requirement `"1.2"` means `>= 1.2.0, < 2.0.0`; resolution
  picks the highest version satisfying every requirement, and `ember update [package]` recomputes it.
  Two majors of one package may coexist, distinguished in symbol names.

## XVII.3 Builds

* `[BLD-1]` The module (one file) is the unit of front-end caching; the package is the unit of
  monomorphisation and code generation.
* `[BLD-2]` A module's front-end result is keyed by its source, the compiler and language versions,
  the package configuration and the **interface hashes** of what it imports (signatures, layouts,
  effect sets, inline bodies), so editing a private body recompiles that module and re-checks the
  contracts of callers only if the body's effect set changed (`[EFF-4]`).
* `[BLD-8]` Within a module, editing one function body re-checks that function alone.
* `[BLD-4]` The C compiler and linker run through a generated Ninja file, so C compilation is
  incremental too.
* `[BLD-5]` Output goes to `target/<profile>/{bin,lib,c,obj,bind,inspect}`.
* `[BLD-7]` Front-end stages run in parallel across modules by default (`-j`, default the number of
  physical cores).
* `[BLD-6]` `lto = "off" | "thin" | "on"`. A value the C toolchain does not support is replaced by the
  nearest one it does, and the build record says so. No correctness property depends on LTO.
* `[BLD-9]` `--timings` writes a per-stage, per-module timing report (`--timings=json` for tools).
* `[BLD-10]` *(changed in 0.9.9)* The compile-time budgets below are release gates.
* `[BUD-1]` They are measured on a recorded reference machine (an 8-core laptop CPU of 2020 or later,
  16 GB, NVMe, warm caches) over `bench/bigpkg`, a generated 50k-line package of 400 modules with
  15 % generic code and one C header import of Vulkan's size.
* `[BUD-2]` Each figure is the median of 15 runs after 3 warm-up runs:

  | # | Operation | Budget |
  |---|---|---|
  | B1 | `ember check` after a one-function-body edit | ≤ 250 ms |
  | B2 | `ember build --reload` after a one-function-body edit | ≤ 1 s |
  | B3 | `ember build` after a one-function-body edit, `debug` | ≤ 1.5 s |
  | B4 | `ember build` after a signature change in a leaf module | ≤ 2.5 s |
  | B5 | `ember build` after a change to a widely imported type | ≤ 8 s |
  | B6 | clean `debug` build, cold cache | ≤ 40 s |
  | B7 | clean `release` build | ≤ 90 s |
  | B8 | `.embind` regeneration for a Vulkan-sized header | ≤ 3 s |
  | B9 | compiler peak memory, clean build | ≤ 3 GB |

* `[BUD-3]` A figure above its budget fails CI, and so does one more than 15 % above the recorded
  baseline even inside its budget.
* `[BUD-3a]` CI normalises its measurements by a fixed calibration workload, and a run whose
  calibration varies by more than 10 % is inconclusive, not failed.
* `[BUD-3b]` A gate applies only to a figure whose measured spread is below the gate, and every figure
  is published with its spread.
* `[BUD-5]` A proposed language or compiler feature states its measured effect on B3 and is rejected if
  it adds more than 5 percentage points.
* `[BLD-11]` A package that omits a `[STD-6]` layer cannot use it or depend on a package that does.
* `[BLD-13]` Builds are reproducible: the same inputs give a byte-identical artefact, with no
  timestamps, absolute paths, host names or hash-order dependence. `--build-id` prints the hash of the
  inputs (`[DET-7]`).

## XVII.4 Profiles

* `[PRF-1]` *(changed in 0.9.9)* **A profile never changes what a program means.** Every profile
  accepts the same programs, performs the same safety checks (bounds, overflow, exclusivity, stale
  handles, `RefCell`), panics in the same places with the same messages, and computes the same results.
  The one check that differs is `debug_assert`, checked only where `[PRF-3]` says: it may not have
  side effects (`W2016`), so a program whose assertions hold computes the same results in every
  profile. Only the following differ between profiles.
* `[PRF-3]` *(new in 0.9.9)* **What a profile may set.**

  | Setting (key) | `debug` | `release` | `shipping` |
  |---|---|---|---|
  | optimisation (`opt`) | 0 | 2 | 3 |
  | debug information (`debug_info`: `full`, `lines`, `none`) | full | lines | none |
  | symbols stripped (`strip`) | no | no | yes |
  | backtrace on panic (`backtrace`) | yes | yes | no |
  | `debug_assert` (`debug_assert`) | checked | compiled out | compiled out |
  | leak and cycle report at exit (`leak_report`, `[WK-15]`) | on | off | off |
  | lock-order report (`lock_order`, `[THR-4]`) | on | off | off |
  | C sanitizers (`sanitizers = […]`) | allowed | allowed | not allowed |
  | link-time optimisation (`lto`) | off | thin | on |

  A package may define more profiles, each `inherits` one of these and overrides keys of this table.
  A key outside the table is `E9001`.
* `[PRF-2]` Hot reload is a build mode admitted in `debug` and `release`, not a profile (Annex B).

## XVII.5 Tests, gates and conformance

```ember
#$ test: compile-fail
fn main():
    a = [1, 2, 3]
    b = a
    a.push(4)          #$ error[E3040]: use of moved value `a`
```

* `[TST-0]` Test annotations are line comments beginning `#$`, read from the raw source text, so a test
  may expect a failure in the lexer.
* `[TST-1]` `#$ error[E…]: text`, `#$ warning[…]` and `#$ note` assert a diagnostic whose primary span
  starts on that line; unexpected and missing diagnostics both fail.
* `[TST-2]` `#$ stdout:`, `#$ exit: N`, and `#$ assert-c: contains("…")` check output, exit status and the emitted C.
* `[TST-3]` `@test` functions run in the test binary, each isolated; `@should_panic` expects a panic.
* `[TST-4]` Every rule of this document has a directory `tests/conformance/<RULE-ID>/`, listed by
  `tools/rule_index.py`, which fails CI for a rule with none.
* `[TST-4a]` Each rule's directory has an **accept** case and, for each diagnostic code the rule names,
  a **reject** case with the exact expected output.
* `[TST-4b]` Which rules need a reject case is decided mechanically: a rule that names a code needs one
  per code; a rule naming none (a layout guarantee, a permission to optimise) is waived, and the
  generated waiver list is committed.
* `[TST-11]` *(changed in 0.9.9)* Besides the per-rule cases, the suite covers these scenarios:
  exclusivity (scalar reads, `Copy` field writes, iteration through `let` fields, mutation during
  iteration, non-`Copy` field assignment during a loop); loop-hoisted checks (one stable receiver, a
  loop-invariant receiver, an escaping receiver, a virtual call that defeats the proof, two receivers
  that must not merge); callables (fresh regions per call, escape rejection, mode mismatches);
  multi-region views (independent owners, one field's source dying, storage in a class or static,
  region inequality never used as `noalias`); arenas (return provenance, `CapacityError.Full`, no
  cursor movement after construction, uninitialised allocation); `Span` and `MutSpan` (iteration,
  reborrow reuse, `chunks(0)` panics, `split_at`); `Shared`/`Weak` (upgrade after release, upgrade
  during `drop`, atomic upgrade for `@sync`); cycles (direct, three-node, through generic containers,
  weak and foreign edges, a runtime cycle static analysis missed).
* `[TST-6]` *(changed in 0.9.9)* Appendix A's code is generated from a fixture that is compiled in CI.
  The fixture declares the items the printed fragment names, so the printed block stays a fragment
  (`ember,fragment`) while the code in it is compiled.
* `[TST-7]` *(changed in 0.9.9)* Every ` ```ember ` block in this document is extracted and must pass
  `ember check` (`--syntax-only` for blocks that name items they do not declare, marked
  ` ```ember,fragment `). A block may opt out as ` ```ember,ignore ` only with a stated reason: a
  signature sketch or a deliberate error example. Overlay source is marked ` ```ember,overlay ` and
  checked as an overlay (`[GRM-35]`).
* `[TST-8]` `tests/firstweek/` holds at least 24 first-draft programs a newcomer plausibly writes in
  week one (a text adventure, a CSV summariser, a scene graph with parent links, an event bus, an
  inventory, a path finder…), each written by someone who has read only Part I and Annex A and
  committed unmodified.
* `[TST-9]` Each is marked `accepted` or `rejected(<shape>)`; a rejection whose shape is in no
  catalogue (§XVII.6) blocks the release, and the help of the shape, applied literally, must make the
  program compile.
* `[TST-10]` The acceptance rate is published with each release; a release that lowers it says why.
* `[TST-27]` *(new in 0.9.9)* **The C gate.** Every accepted program in the test suite is compiled
  through the C compiler with warnings as errors on each supported host compiler. A C compiler error
  on an accepted program is a compiler defect and fails CI (`[CG-C-2]`).
* `[TST-28]` *(new in 0.9.9)* **The performance gate.** `tests/perf/` holds benchmark programs, each
  with an equivalent C program, a checker of identical output, and a threshold (for example, within
  10 % of C at `-O2`). A regression past a threshold fails CI. This is the instrument for "as fast as
  C"; no performance claim in this document is considered met until its benchmark exists.
  `ember bench --compare` runs the pairs.
* `[BEN-1]` Each benchmark runs at least 3 untimed and 30 timed repetitions on one pinned core and
  reports the median and the 95 % confidence interval of the median for each side and their ratio.
* `[BEN-2]` A gate fails only when the lower bound of the ratio's interval exceeds the threshold; a
  point estimate above it whose interval includes it is inconclusive and re-run.
* `[BEN-3]` Each run also measures the C program against a second copy of itself; if that ratio's
  interval excludes 1.00 ± 0.02 the run is void and the machine unfit for gating.
* `[BEN-4]` Each benchmark records retired instructions; a change beyond ± 0.5 % against the baseline
  fails regardless of timing.
* `[BEN-5]` Both sides are built with the same optimisation level, LTO setting, floating-point model
  and target-CPU flags; a mismatch voids the run.
* `[BEN-6]` Thresholds: scalar and tight loops ≤ 1.05× C; SoA and SIMD code ≤ 1.10×; `@noalloc` paths
  assert allocation counts exactly; a direct FFI call executes the same instructions as C's call;
  counted-object code ≤ 1.15× a C++ intrusive count (non-atomic, or atomic for `@sync` classes).
* `[EXC-13]` The performance suite contains class-handle loops with one stable receiver, a
  loop-invariant receiver, an escaping receiver, a virtual call that defeats the proof and two
  receivers that must not merge, and asserts one hoisted check for the first two and per-access checks
  for the rest (`[EXC-8]`).
* `[TST-29]` *(new in 0.9.9)* **Honest baselines.** A gate with a baseline of known failures reports the
  baseline size beside its result; the baseline only shrinks, and a gate whose baseline covers more
  than a tenth of its cases reports itself as `baselined`, not green.
* `[TST-30]` *(new in 0.9.9)* The test runner reports every failure of a run, not only the first, and a
  test that fails intermittently is quarantined by name and listed in the run summary.
* `[TST-5]` A scripted debugger session (breakpoint by Ember line, stepping, locals by Ember name) runs
  in CI on every supported host; a host where it cannot run is reported as debug-unverified, with a
  recorded waiver.

## XVII.6 Diagnostics

* `[DIA-1]` A diagnostic has a code, one primary span, labelled secondary spans, an optional `help`
  (with a machine-applicable fix-it where possible) and notes. `--json` gives the same structure.
* `[DIA-2]` Messages start lowercase, have no trailing period, name the thing, and say what is wrong;
  `help` says what to do; a message never says "you" and never blames the programmer. The primary
  message explains the failure in terms of the source program; compiler-internal facts (region
  numbers, internal names) may appear only as secondary evidence.
* `[DIA-3]` Ownership and borrow errors include the "later used here" label and a concrete fix from the
  catalogue (§XVII.6.1).
* `[DIA-4]` Contract errors print the whole call chain (`[EFF-6]`).
* `[DIA-5]` FFI errors name the header and the C declaration.
* `[DIA-6]` Every code has a page, `docs/errors/EXXXX.md`, with a program that triggers it, the
  rendered diagnostic, why the rule exists and the fix; `ember explain` prints it. A code without a
  page fails CI, and each page's examples are built by `ember test --doc`: the failing one must fail
  with that code and the fixed one must compile.
* `[DIA-6a]` The code registry is exhaustive in both directions: every code this document names is
  registered, and every registered code is named here (§XVII.9). Each code is defined by exactly one
  rule.
* `[DIA-7]` Every ownership and borrow error is classified into a shape of §XVII.6.1 and emits that
  shape's help.
* `[DIA-7a]` Every code in `E3000`–`E3499` belongs to exactly one shape of §XVII.6.1; a code with no
  row is never emitted, and `tools/rule_index.py` fails CI on one.
* `[DIA-8]` `ember explain --borrow <file>:<line>` prints, for each loan live at that line, where it was
  created, the lines its region covers and the later use that keeps it alive: `borrow of v created at
  12:9, live through 19, because s is used at 19:14`. It reads the borrow checker's own loans
  (`[BCK-1]`).
* `[DIA-10]` The help of shapes B1, B2, B4, B5 and B9 names a concrete API or construct (`split_at`,
  `retain`, `Weak`, `owned fn`), never a category such as "consider restructuring".
* `[DIA-11]` For shape S1 the diagnostic reports the check's reason (`[EFF-11]`); when it is
  `not_provable_in_principle` or `inherent_to_mechanism`, the suggestion is to remove the contract or
  change the data structure, never to restructure code that cannot be improved.
* `[DIA-13]` Every shape has a rendered snapshot under `tests/ui/` and a `.fixed.em` companion showing
  the help applied, which must compile.
* `[DIA-15]` Suggestions are computed from data the compiler already holds (scope tables, type
  information, an index of exported names); none requires speculative type checking.
* `[DIA-18]` A foreign call rejected because its contract has unknown facts (`E5002`) names each missing
  fact and prints the overlay line that supplies it.
* `[DIA-12]` Every name and type error is classified into a shape of §XVII.6.2. An unclassified error
  is logged, and CI fails on the log.
* `[DIA-9]` A diagnostic never suggests `unsafe`, `Cell`, `RefCell`, `Shared` or `clone()` first when a
  structural fix exists.
* `[DIA-16]` When a diagnostic suggests moving a value into a class, `Shared` or `RefCell`, a note names the run-time cost.
* `[DIA-14]` Only the first error of a cascade is reported: nothing is reported about an expression
  whose type is already an error, a name bound to a failed import, or a call to a failed declaration.
* `[DIA-20]` *(new in 0.9.9)* A run of invalid bytes or characters is one diagnostic, not one per byte.
* `[DIA-21]` *(new in 0.9.9)* **Python habits.** Each of these is recognised and answered with the
  Ember form as a machine-applicable fix-it:

  | Written | Help |
  |---|---|
  | `True`, `False`, `None` as a value where no `Option` is expected | `true`, `false`; `None` needs an `Option` type |
  | `def f():` | `fn f():` |
  | `str(x)`, `int(s)`, `float(s)`, `int(x)` | `x.to_string()` or `f"{x}"`, `s.parse[int]()`, `s.parse[float]()`, `x as int` |
  | `len(s)` for a string (`E2073`) | `s.char_count()` (Python's count) or `s.len()` (bytes) |
  | `min(xs)`, `max(xs)` of one collection | `xs.iter().min()`, `xs.iter().max()` (an `Option`) |
  | `xs.append(x)`, `s.strip()`, `s.startswith(p)`, `s.upper()`, … | the `[STD-13]` table (Appendix E) |
  | `xs[a:b]` | `xs[a..b]` |
  | `if xs:` | `if not xs.is_empty():` (`[CTL-0]`) |
  | `let x = 5`, `var x = 5` | `x = 5` |
  | `x == None` | `x is None` |
  | `with open(p) as f:` | `with f = fs.File.open(p)?:` |
  | `raise e`, `try:`, `except E:`, `finally:` | `return Err(e)`, `?`, `match`, `defer:` (Part XIII) |
  | `lambda x: e` | `fn(x) => e` |
  | `self` missing from a method's parameters | add `self` |
  | `a / b` on integers | `a // b`, or `a as float / b` (`[TYP-28]`) |
  | `if x = 5:` | `if x == 5:` |
  | `case p:` inside `match` | `p:` (or `p =>`), without `case` |
  | `global x`, `nonlocal x` | a `static` or a class field for shared state; a closure captures `x` without a declaration (`[CLO-2]`) |
  | `*args`, `**kwargs` in a signature | an `Array` or `Span` parameter, or keyword arguments with defaults; Ember has no variadic functions (§I.6) |
  | `"%d" % n`, `"{}".format(n)` | `f"{n}"` (`[LEX-19]`) |

* `[DIA-22]` *(new in 0.9.9)* A diagnostic caused by a callee's contract or signature is reported at the
  call site in the programmer's code, with the callee's declaration as a secondary span.
* `[DIA-23]` *(new in 0.9.9)* A run-time panic names Ember entities — the class, field, variable, file
  and line — never a C symbol or a mangled name.
* `[DIA-24]` *(new in 0.9.9)* **Name suggestions (shape N1).** A candidate is suggested when its
  Damerau–Levenshtein distance from the unknown name is at most 1 for names of up to 4 characters and
  at most 2 otherwise, and it is of the right kind for the position (a type where a type is expected,
  a value where a value is). At most three are suggested, closest first, then same file before other
  files, then by qualified name.

### XVII.6.1 Ownership and borrow shapes

| Shape | Situation | Codes | Primary help |
|---|---|---|---|
| O1 | use after move | `E3040` | `x.clone()` if `Clone`; else reorder so the last use precedes the move; for a collection, a note that assignment moves a list where Python's aliases it (Appendix E) |
| O2 | move out of a container or field | `E3010`–`E3013` | `mem.take`, `mem.replace`, `swap`; `pop`, `swap_remove`, `remove`, `drain` |
| O3 | move inside a loop | `E3041` | declare inside the loop, clone per iteration, or `mem.take` |
| O4 | whole use after a partial move | `E3042` | reassign the field, or destructure up front |
| O5 | closure moves out a capture | `E3030` | declare the parameter `once fn` |
| O6 | borrow of an uninitialised place | `E3050` | name the path on which it is uninitialised |
| O7 | explicit `x.drop()` | `E3070` | `mem.drop(x)` |
| O8 | scope guard moved or leaked | `E3014`, `E3015` | bind it with `with` (`[THR-5]`) |
| O9 | `self` escapes its `drop` | `E3016` | `mem.take` the data out |
| B1 | two mutable borrows of one place, e.g. two indices | `E3022` | `split_at`, `chunks_mut`, `iter_mut`, `swap(i, j)`; `SoA` columns |
| B2 | mutation while iterating | `E3020` | `retain`, `drain`, collect first, or an index loop |
| B3 | shared and mutable overlap | `E3021` | end the shared borrow first, or copy the value out |
| B4 | two writers of one value | `E3023` | a single owner passing `mut`; `Cell` for a counter |
| B5 | self-referential struct | `E3024` | store an index or `Handle`; split the struct |
| B6 | returned view not derived from a parameter | `E3062` | return an owned value, or `@borrows(p)` |
| B7 | borrow outlives its source | `E3060` | move the source outward, return an owned value, or bind with `with`; for a parameter, borrow it instead of taking it `owned`, or take it as `Span[T]`, `str` or `ref T` |
| B8 | a method takes all of `self` | `E3025` | `[BRW-10]` covers private methods; else take the fields as parameters |
| B9 | closure outlives its captures | `E3026` | `owned fn`, or a scope (`[THR-5]`) |
| B10 | `mut` argument that is not a mutable place | `E3027` | bind to a local first |
| B11 | disjointness not provable | `E3095` | `assert_disjoint` (`[DSJ-1]`) |
| B12 | view stored where it outlives its source | `E3063` | store an owned value or an index |
| B14 | multi-region result provenance cannot be inferred | `E3065` | return an owned value, or the views as separate results |
| B15 | callable parameter-mode mismatch | `E2228` | the expected `fn` type, spelled |
| X1 | static exclusivity conflict | `E3080` | end the first access first; copy the value out |
| S1 | `@static_safe` violated | `E4030` | the exact access and why it is dynamic (`[EFF-13]`, `[DIA-11]`) |
| A1 | arena view outlives the arena; arena allocation of a type with `drop`; parent used during a scope | `E3061`, `E3090`, `E3096` | move the arena outward, copy out, or end the scope first |
| U1 | operation needs `unsafe` | `E3100`, `E3105` | the safe API that does the same, else an `unsafe:` block with a `# SAFETY:` note |
| R1 | the reason is far from the error | — | `ember explain --borrow <file>:<line>` |

### XVII.6.2 Name and type shapes

| Shape | Situation | Primary help |
|---|---|---|
| N1 | unknown name | up to three candidates (`[DIA-24]`) |
| N2 | name exported elsewhere | the import line, as a fix-it |
| N3 | unknown method | nearest member; the interface that would supply it; a Python name per `[DIA-21]` |
| N4 | scalar mismatch | the exact lossless `as`, and which side set the expected type; for an `int` and a `float` operand (`i * 0.5`), `i as float * 0.5` as a machine-applicable fix-it |
| N5 | literal does not fit (`E2010`) | the suffix or annotation that fits |
| N6 | wrong number of arguments | the signature with names and modes |
| N7 | mutating an immutable place | the declaration to change, with its location |
| N8 | argument does not fit the parameter's mode | the mode in the callee's signature and the caller's fix (`.clone()`, a mutable local, or ending the value's use before the call); a call site never writes a mode (`[FN-2a]`) |
| N9 | missing bound (`E2040`) | the bound to add, and the one module where `extend` may be written (`[TYP-20]`) |
| N10 | not usable as `dyn` (`E2050`) | the method and the clause it violates |
| N11 | ambiguous method (`E2070`) | `I.m(recv, …)` for each candidate |
| N12 | indentation (`E0002`–`E0004`) | the expected column and the line that set it |
| N13 | a field assigned (`self.x = …`) but not declared (`E2072`) | `field 'x' is not declared in class C`, with a machine-applicable fix-it adding `x: <inferred type>` to the class body |

### XVII.6.3 Code ranges

| Range | Area |
|---|---|
| `E0000`–`E0099` | lexing, indentation, directives, `E0900`/`E0901` (`[PHIL-12]`) |
| `E0100`–`E0499` | parsing |
| `E1000`–`E1499` | names, modules, visibility |
| `E2000`–`E2499` | types, inference, interfaces, patterns |
| `E3000`–`E3499` | ownership, borrows, regions, exclusivity, drops |
| `E4000`–`E4499` | effects and contracts, SIMD |
| `E5000`–`E5499` | FFI and interop |
| `E6000`–`E6499` | compile-time evaluation |
| `E7000`–`E7499` | concurrency |
| `E8000`–`E8499` | layout and GPU layout |
| `E9000`–`E9499` | build, manifest, toolchain |
| `L…`, `W…` | lints and warnings |

## XVII.7 Formatter, linter and documentation

* `[FMT-1]` *(changed in 0.9.9)* `ember fmt` writes LF line endings, four-space indentation and at most 100
  columns, whatever the platform.
* `[FMT-2]` The formatter uses the `=>` form of a lambda whose body is one expression.
* `[FMT-3]` The formatter never emits `;` outside `[T; N]` and `[v; N]`.
* `[LNT-1]` `L1001 unused binding`: a local never read on any path (names starting `_` are exempt).
* `[LNT-2]` `L1002 assignment declares a new binding`: an unread new name within distance 2 of a
  mutable binding in scope, which is usually a typo; its fix-it rewrites the name, and it is an error
  under `edition_lints = "strict"`.
* `[LNT-3]` `L1001` and `L1002` are reported by `ember build` and `ember check`, not only `ember lint`.
* `[LNT-4]` `L2004`: a `gen fn` with no `yield`.
* `[LNT-5]` `L2005`: a `@noreload` function calling a reloadable one in a loop.
* `[LNT-6]` *(new in 0.9.9)* `ember lint` also reports, at `warn` unless `[lints]` says otherwise:
  `L2001` unnecessary clone; `L2002` large `Copy` value passed by value (`large_copy = { threshold =
  256 }`); `L3002` borrow held longer than necessary; `L4001` allocation in a hot loop; `L4002`
  dynamic dispatch on a final type; `L4003` an `Array[int]` or `Array[float]` of more than 64 KiB
  indexed in a hot loop whose values provably fit 32 bits; `L5001` `unsafe extern` declaration with no contract; `L5002`
  copying conversion at the FFI boundary; `L7001` lock held across a call that may block; and `L3001`
  potential reference cycle (`potential_cycle`, `[WK-6]`).
* `[DOC-1]` Error pages ship with their errors.
* `[DOC-2]` The user guide (`docs/book/`), with a "coming from Python" chapter built on Appendix E, is
  published with each release, and every sample in it is run by `ember test --doc`.
* `[DOC-3]` `ember doc` presents, for every function that produces or consumes a view, its generated
  contract: ownership, what the result borrows, effects and allocation, derived from the signature.
* `[DOC-4]` The guide, the error pages and this specification are published together for each release.

## XVII.7a Editor support

* `[IDE-3]` Every compiler stage after parsing produces a complete result for a file with errors: an
  unresolvable name and an untypable expression become error placeholders that satisfy everything, so
  hover, completion and symbols keep working in a broken file.
* `[IDE-4]` Every editor request is answered from parsing, name resolution and type checking alone,
  never from code generation or a C compiler; borrow- and effect-derived information is shown from the
  last completed `ember check` and never blocks a response.
* `[IDE-6]` The compiler keeps all its state in a session object that can be dropped and rebuilt, so a
  long-running editor process does not grow without bound.
* `ember lsp`, a language server over stdio, is reserved for a named milestone before 1.0.

## XVII.8 Conformance profiles and ABI versions

* `[CONF-1]` *(changed in 0.9.9)* A compiler declares the profile it implements, and claims it only
  when every conformance test of every rule in it passes; a partial implementation claims the profile
  below, never the one above with exceptions.
* `[CONF-2]` **Ember Core**: Parts II–VII, X, XIII and XV's core and alloc layers.
* `[CONF-3]` **Ember Systems**: adds Parts VIII, IX, XI, XII and XIV.
* `[CONF-4]` **Ember Native**: adds Part XVI. Annex C (C++) is a separate, optional claim.
* `[CONF-5]` **Ember Dynamic**: adds Annex B (hot reload).
* `[ABI-1]` The runtime ABI, the hot-reload protocol, the module protocol and the export-table protocol
  are versioned independently of the language; each version is a linked symbol whose name encodes the
  number, so a mismatch fails at link time, and `ember_module_init` checks the runtime ABI first.
* `[ABI-2]` A protocol that cannot be checked at link time (hot-reload images, loaded at run time) is
  checked on load and refused with `E9037`.
* `[ABI-3]` `ember_module_init` checks the runtime ABI version before anything else and returns a
  distinct failure code on a mismatch.
* `[ABI-4]` A release notes every protocol whose version changed, and why.
* `[ABI-5]` The protocol versions are independent of the language version: two compilers of one
  language version may differ in the reload protocol (and must not be mixed in one process), never in
  the C ABI of exports.
The runtime ABI (the object header, `TypeInfo`, `ember_rt.h`) is stable within a major version
(`[VER-4]`).

## XVII.9 The diagnostic code registry

Every diagnostic code, what it reports and the rule it enforces (`[DIA-6a]`). Codes are never
reused; retired codes are listed with the reason.

| Code | Reports | Rule |
|---|---|---|
| `E0001` | invalid UTF-8 in source | `[LEX-1]` |
| `E0002` | tab used for indentation | `[LEX-4]` |
| `E0003` | inconsistent dedent | `[LEX-5]` |
| `E0004` | expected an indented block | `[LEX-9]` |
| `E0005` | reserved keyword used as an identifier | `[LEX-14]` |
| `E0006` | a `#!` language directive names a version other than the current one | `[VER-8]` |
| `E0008` | unterminated character literal | `[LEX-22]` |
| `E0100` | unexpected token | `[GRM-2]` |
| `E0101` | unclosed delimiter | `[LEX-6]` |
| `E0102` | membership test chained (`a in b in c`) | `[GRM-23]` |
| `E0103` | match arms mix statement and expression form | `[GRM-10]` |
| `E0104` | attribute or construct not admitted here | `[ATT-1]`, `[PHIL-12]` |
| `E0105` | `;` is not a statement separator | `[GRM-18]` |
| `E0106` | a multi-statement closure cannot be written inside brackets | `[GRM-17]` |
| `E0107` | a jump expression may not be an operand | `[GRM-16]` |
| `E0108` | attribute is not permitted on this statement | `[GRM-20]` |
| `E0109` | `owned` is not permitted in expression position | `[GRM-15]` |
| `E0110` | function declared without a body outside an interface, extern block or abstract class | `[GRM-33]` |
| `E0111` | `ref` of an expression that is not a place | `[GRM-36]` |
| `E0900` | construct not implemented by this compiler | `[PHIL-12]`, `[CLI-19]` |
| `E0901` | construct this specification leaves unspecified | `[PHIL-12]` |
| `E1010` | cannot find name in this scope | `[MOD-3]` |
| `E1020` | name is already declared in this block | `[GRM-4]` |
| `E1021` | name is not linked in this build | `[BLD-11]` |
| `E1030` | two items with the same name in one scope | `[TYP-26]` |
| `E1031` | a name bound by two glob imports is used | `[MOD-8]` |
| `E1041` | import cycle between packages | `[MOD-4]` |
| `E1050` | field is read-only outside its module | `[MOD-7]` |
| `E1051` | `read` visibility is valid on fields only | `[MOD-7]` |
| `E1052` | item, field or constructor not visible here | `[MOD-2]` |
| `E1060` | no module of that name | `[MOD-3]` |
| `E1061` | no item of that name in the module | `[MOD-3]` |
| `E2010` | literal does not fit its type | `[LEX-16]` |
| `E2011` | negative literal index | `[TYP-31]`, `[LEX-24]` |
| `E2020` | mismatched types | `[TYP-4]` |
| `E2030` | `@view` on a type that is not a view | `[TYP-34]` |
| `E2031` | `@borrows` names a parameter the result cannot borrow from (a borrowed `Copy` parameter, or an `owned` one that is not a reference or view) | `[LT-1a]` |
| `E2035` | condition must be `bool` | `[CTL-0]` |
| `E2036` | this pattern always matches | `[GRM-19]` |
| `E2040` | unsatisfied interface bound | `[TYP-17]` |
| `E2041` | overlapping `extend` implementations | `[TYP-19]` |
| `E2050` | interface is not `dyn`-compatible | `[TYP-22]` |
| `E2060` | cannot infer type | `[TYP-23]` |
| `E2061` | lambda parameter types cannot be inferred here | `[TYP-23]` |
| `E2062` | ambiguous type | `[TYP-23]` |
| `E2070` | ambiguous interface method | `[TYP-24]` |
| `E2071` | `SoA[T]` of a type that is not a struct | `[SOA-1]` |
| `E2072` | no method or field of that name (with the Ember name for a Python one; shape N13 for an undeclared field) | `[STD-13]` |
| `E2073` | `len` of a string (Python counts characters, `s.len()` bytes) | `[STD-26]` |
| `E2080` | `@derive(Copy)` on a type with a field that is not Copy | `[STR-3]` |
| `E2090` | non-exhaustive match | `[ENM-2]` |
| `E2100` | field read before it is initialised | `[CLS-2]` |
| `E2101` | derived class with a field that has no default needs an `init` | `[CLS-10]` |
| `E2102` | derived `init` uses `self` or an inherited field before `super.init`, or leaves an own field unassigned | `[CLS-11]` |
| `E2110` | override of a method that is not virtual | `[CLS-4]` |
| `E2111` | a method that replaces an inherited virtual one without `override` | `[CLS-4]` |
| `E2120` | inherent extension of a type from another package | `[IFC-2]` |
| `E2130` | a `const` of a type that owns heap memory; use a `static` | `[TYP-1]` |
| `E2131` | array length must be a constant | `[CT-1]` |
| `E2140` | cannot assign to a value expression | `[EXP-5]` |
| `E2150` | `is` on an operand that is neither a handle, a reference nor an `Option` | `[EXP-9]` |
| `E2151` | integer `**` with a constant negative exponent | `[TYP-30]` |
| `E2160` | control flow cannot leave a `defer` block | `[CTL-7]` |
| `E2170` | reference to a field of a packed struct | `[LAY-2]` |
| `E2172` | cannot index with a type | `[GRM-8b]` |
| `E2173` | not a type or const-generic argument | `[GRM-8b]` |
| `E2180` | `?` outside a function returning `Option` or `Result` | `[ERR-2]` |
| `E2181` | `?` on an `Option` in a `Result` function, or the reverse | `[ERR-2]` |
| `E2182` | a function that returns a value can reach the end of its body | `[FN-10]` |
| `E2200` | type has infinite size (a recursive value type without `Box`) | `[TYP-14]` |
| `E2210` | a value of one range type where another was expected | `[RNG-2]` |
| `E2211` | constant outside the target range type | `[RNG-3]` |
| `E2212` | range endpoints are not constants of the representation, or are inverted | `[RNG-1]` |
| `E2213` | invalid `in` clause on a type alias | `[RNG-1]` |
| `E2214` | arithmetic between two distinct range types | `[RNG-5]` |
| `E2215` | range-typed value constructed outside the permitted set | `[RNG-10]` |
| `E2220` | `yield` outside a `gen fn` | `[CORO-2]` |
| `E2221` | borrow held across a `yield` | `[CORO-6]` |
| `E2222` | coroutine where an ordinary function is required | `[CORO-10]` |
| `E2223` | `@never_specialize` on a generic that is not shareable | `[MONO-7]` |
| `E2225` | `migrate_from` may not panic | `[HR-35]` |
| `E2226` | `in` on a type that does not implement `Contains` | `[STD-8]` |
| `E2228` | callable parameter-mode mismatch (shape B15) | `[CLO-3]` |
| `E2229` | class generator method takes `mut self` or holds an access across `yield` | `[CORO-12]` |
| `E2230` | a name assigned in every branch at different types | `[CTL-10]` |
| `E2231` | `yield` while a `@must_drop` value is live | `[CORO-13]` |
| `E2240` | `/` on two integers | `[TYP-28]` |
| `E2250` | format spec does not apply to the value's type | `[LEX-19]` |
| `E2260` | `some` type outside a return position | `[TYP-32]` |
| `E2261` | returns of a `some` function have different types | `[TYP-32]` |
| `E3010` | cannot move out of a field of a type with `drop` | `[EXP-6]` |
| `E3011` | cannot move out of an array or span element | `[EXP-6]` |
| `E3012` | cannot move out of a class field | `[EXP-6]` |
| `E3013` | cannot move out of a reference | `[EXP-6]` |
| `E3014` | scope binding may not be moved | `[THR-5]` |
| `E3015` | value whose drop is required may not be leaked | `[THR-6]` |
| `E3016` | `self` escapes its own drop | `[CLS-7a]` |
| `E3020` | iterable is mutated while the loop borrows it | `[CTL-2]` |
| `E3021` | a shared and a mutable borrow overlap | `[BRW-1]` |
| `E3022` | two mutable borrows of the same place | `[BRW-1]` |
| `E3023` | two writers of one value (shape B4) | `[BRW-1]` |
| `E3024` | a struct field would borrow another field of the same struct (shape B5) | `[TYP-15]` |
| `E3025` | a method takes all of `self` (shape B8) | `[BRW-10]` |
| `E3026` | a closure outlives what it captures (shape B9) | `[CLO-4]` |
| `E3027` | a `mut` argument is not a mutable place (shape B10) | `[FN-2a]` |
| `E3030` | closure would move a captured value out | `[CLO-2]` |
| `E3040` | use of moved value | `[OWN-3]` |
| `E3041` | value moved in a previous loop iteration | `[OWN-4]` |
| `E3042` | partial move then use of the whole value | `[EXP-6]` |
| `E3050` | use of an uninitialised or moved place | `[BRW-7]` |
| `E3060` | borrowed value does not live long enough | `[LT-3]`, `[BRW-8]` |
| `E3061` | arena allocation cannot outlive its arena | `[LT-4]` |
| `E3062` | returned view does not derive from a parameter (shape B6) | `[LT-1]` |
| `E3063` | stored view may not outlive its source | `[TYP-15]` |
| `E3065` | multi-region result provenance cannot be inferred | `[LT-35]` |
| `E3070` | `drop` cannot be called explicitly | `[DRP-1]` |
| `E3080` | overlapping access through the same handle | `[EXC-3]` |
| `E3090` | arena allocation of a type that needs `drop` | `[ARN-3]` |
| `E3095` | disjointness is not establishable for these operands | `[DSJ-1]` |
| `E3096` | arena is scoped here | `[ARN-6]` |
| `E3100` | this operation requires an `unsafe` block | `[UNS-1]` |
| `E3105` | `UnsafeCell` in `@static_safe` code | `[UNS-10]` |
| `E4001` | `@noalloc` function reaches an allocation | `[EFF-5]` |
| `E4002` | `@nosync` function reaches a synchronising operation | `[EFF-5]` |
| `E4003` | `@noblock` function reaches a blocking operation | `[EFF-5]` |
| `E4010` | implementation does not satisfy the interface's contract | `[EFF-2]` |
| `E4020` | `@simd(assert)` loop did not vectorise | `[SIMD-2]` |
| `E4030` | `@static_safe` function performs a dynamically checked access | `[EFF-12]` |
| `E4040` | `@nopanic(explicit)` function reaches a panic | `[EFF-17]` |
| `E4041` | `@noio` function reaches I/O | `[EFF-20]` |
| `E4042` | `@nolock` function acquires a lock | `[EFF-21]` |
| `E4070` | `@deterministic` function reaches a nondeterministic operation | `[DET-1]` |
| `E4072` | `@fastmath` or `@fp(contract)` inside `@deterministic` | `[DET-5]` |
| `E5001` | imported type layout does not match | `[FFI-5]` |
| `E5002` | foreign call requires `unsafe`: its contract is incomplete, or `safe fn` on a function whose contract is | `[FFI-2]`, `[FFI-10]` |
| `E5010` | overlay does not match the C declaration | `[FFI-12]` |
| `E5011` | one C identity imported with two different layouts | `[FFI-30]` |
| `E5012` | pointer contract has no count | `[FFI-11]` |
| `E5014` | two packages request the same implementation macro | `[FFI-29]` |
| `E5015` | type may not cross the boundary | `[FFI-31]` |
| `E5016` | `_Atomic` layout does not match `Atomic[T]` | `[FFI-8]` |
| `E5017` | struct with a flexible array member is unsized | `[FFI-8]` |
| `E5018` | `va_list` may not be constructed | `[FFI-8]` |
| `E5020` | `str` is not NUL-terminated | `[FFI-15]` |
| `E5030` | `std::function` as a parameter | `[FFI-17a]` |
| `E5031` | C++ parameter cannot be mapped | `[FFI-17]` |
| `E5032` | opaque C++ type may not be constructed | `[FFI-32]` |
| `E5034` | unsupported C++ construct | `[FFI-17]` |
| `E5040` | capturing closure passed where a C function pointer is expected | `[FFI-21]` |
| `E5041` | retained callback needs a release function in the overlay | `[FFI-21]` |
| `E5050` | a foreign fact claims a grade whose evidence is absent or stale | `[TCB-1]` |
| `E5051` | a foreign callee retains a pointer its contract does not declare `retained` | `[FFI-35a]` |
| `E5052` | `adopt` on a handle whose overlay declares `adopt = false` | `[FFI-36b]` |
| `E5053` | an instrumented run contradicted a declared foreign effect | `[FFI-37]` |
| `E5054` | range type in a foreign signature | `[RNG-10]` |
| `E5055` | an Ember generic may not instantiate a C++ template | `[FFI-17b]` |
| `E5056` | override of a C++ virtual the overlay does not name | `[FFI-39]` |
| `E5057` | the overlay names a method that is not virtual in the header | `[FFI-39]` |
| `E5058` | C++ base has no default constructor and no declared `init` | `[FFI-39]` |
| `E5059` | C++ trampoline base has no virtual destructor | `[FFI-39]` |
| `E5060` | upcast in a context that cannot hold the `Retained` token | `[FFI-39]` |
| `E5061` | imported C++ function has no exception policy and none can be derived | `[FFI-24]` |
| `E5062` | contradictory C++ exception policies | `[FFI-43]` |
| `E5063` | foreign bytes reach `str` without validation | `[TXT-2]` |
| `E5064` | interior NUL in a value converted to `cstr` | `[FFI-15]` |
| `E5065` | `Shared`/`Weak` and `CppShared`/`CppWeak` do not interconvert | `[WK-14]` |
| `E5066` | an overlay marks a C++ function both `noexcept` and throwing | `[FFI-43]` |
| `E5067` | an overlay that states facts is not declared `unsafe overlay` | `[GRM-35]`, `[TIER-1]` |
| `E5090` | inline assembly is not supported by the C backend | `[UNS-6]` |
| `E6001` | comptime evaluation exceeded its limits | `[CT-3]` |
| `E6004` | panic during compile-time evaluation | `[CT-7]` |
| `E6005` | `comptime(e)` refers to a run-time local | `[CT-6]` |
| `E6010` | operation not available at compile time | `[CT-2]` |
| `E7001` | `@sync` class with a field that is not `Sync`, or a non-`@sync` base or derived class | `[THR-1]` |
| `E7002` | `static` of a type that is not `Sync` | `[STA-1]` |
| `E7003` | write to a field of a `@sync` class after `init`, or a `mut self` method on one | `[THR-1]` |
| `E7004` | value crossing a thread boundary is not `Send` | `[THR-10]`, `[THR-11]`, `[FFI-22]` |
| `E7005` | value shared with a task is not `Sync` | `[THR-11]` |
| `E7006` | memory order an atomic operation does not support | `[THR-14]` |
| `E7010` | parallel loop writes to a shared place | `[PAR-2]` |
| `E7011` | parallel loop has a loop-carried dependency | `[PAR-2a]` |
| `E7020` | systems in one parallel run have conflicting access sets | `[ECS-4]` |
| `E8001` | GPU layout does not match the CPU layout | `[GPU-10]` |
| `E9001` | invalid manifest, including an unknown key | `[MAN-1]`, `[PRF-3]` |
| `E9002` | no C compiler found, or it failed; the help names the host's remedy | `[BLD-FFI-1]` |
| `E9003` | invalid command line | `[CLI-1]` |
| `E9010` | `[lints]` names a lint the compiler does not define | `[MAN-3]` |
| `E9011` | the toolchain cannot disable floating-point contraction | `[CG-C-11]` |
| `E9013` | invalid `[ffi]` manifest section | `[TCB-1]` |
| `E9020` | translation units of one target disagree on an inherited flag | `[BLD-FFI-1a]` |
| `E9021` | C++ standard library and CRT heap could not be determined | `[BLD-FFI-1b]` |
| `E9030` | hot reload refused | `[HR-18]` |
| `E9031` | invalid `reload` value | `[MAN-7]` |
| `E9033` | `reload` is forbidden in `shipping` | `[PRF-2]` |
| `E9034` | invalid `max_instantiations` value | `[MONO-3]` |
| `E9035` | packages in one process disagree about the object-header layout | `[HR-12a]` |
| `E9036` | a reloadable package may not link the runtime statically | `[HR-29]` |
| `E9037` | reload ABI mismatch on image load | `[ABI-1]` |
| `E9040` | two generic instances hash to one symbol | `[MNG-1]` |
| `E9041` | the toolchain cannot honour `@fastmath` or `@fp(…)` for a function | `[TYP-9c]` |
| `L1001` | unused binding | `[LNT-1]` |
| `L1002` | assignment declares a new binding | `[LNT-2]` |
| `L2001` | unnecessary clone | `[LNT-6]` |
| `L2002` | large `Copy` value passed by value | `[LNT-6]` |
| `L2003` | fallible construction where a total one exists | `[RNG-3a]` |
| `L2004` | `gen fn` with no `yield` | `[LNT-4]` |
| `L2005` | `@noreload` function calls a reloadable one in a loop | `[LNT-5]` |
| `L3001` | potential reference cycle | `[WK-1]` |
| `L3002` | borrow held longer than necessary | `[LNT-6]` |
| `L3010` | `unsafe` block larger than necessary | `[UNS-3]` |
| `L3011` | `RefCell` guard held across a call | `[CELL-7]` |
| `L3013` | long-term access held across a call | `[EXC-7]` |
| `L3014` | return region is the intersection of several parameters | `[LT-1b]` |
| `L3015` | undocumented unsafe obligation | `[UNS-7]` |
| `L3016` | `@safety` text still reads `TODO` | `[UNS-7]` |
| `L3017` | reference cycle detected | `[WK-4]` |
| `L3019` | handle to an object with an observable `drop` is bound and never read | `[RC-3]` |
| `L4001` | allocation in a hot loop | `[LNT-6]` |
| `L4002` | dynamic dispatch on a final type | `[LNT-6]` |
| `L4003` | large `Array[int]`/`Array[float]` in a hot loop whose values fit 32 bits | `[LNT-6]` |
| `L5001` | `unsafe extern` declaration with no contract | `[LNT-6]` |
| `L5002` | conversion at the FFI boundary copies | `[LNT-6]` |
| `L7001` | lock held across a call that may block | `[LNT-6]` |
| `W1002` | binding shadows an enum variant of the same name | `[GRM-12]` |
| `W1003` | a package module shadows the standard module of the same name | `[MOD-3]` |
| `W2015` | float literal has more digits than its type keeps | `[LEX-17a]` |
| `W2016` | `debug_assert` argument has side effects | `[PAN-1]` |
| `W2091` | unreachable match arm | `[CTL-5]` |
| `W2111` | `virtual` has no effect in a final class | `[CLS-4]` |
| `W2190` | unused `Result` | `[ERR-5]` |
| `W2220` | instantiation ceiling exceeded | `[MONO-3]` |
| `W3012` | unsafe block with no SAFETY note | `[UNS-8]` |
| `W5001` | function-like macro ignored | `[FFI-6]` |
| `W5002` | declaration not imported (unsupported convention or construct) | `[FFI-20a]` |
| `W5031` | C++ declaration skipped: the header could not be parsed | `[FFI-20a]` |
| `W5033` | `&&`-qualified member skipped | `[FFI-40]` |
| `W5034` | anonymous-namespace entity skipped | `[FFI-41]` |
| `W5050` | unbacked or stale grade | `[TCB-1]` |
| `W5054` | unexercised foreign fact | `[FFI-37]` |
| `W9030` | reload requires a restart | `[HR-18]` |

**Retired codes.**

| Code | Why |
|---|---|
| `E0007` | no lifetime syntax exists; a stray `'` is an unterminated character literal (`E0008`) |
| `E1040` | glob imports are allowed from any module (`[MOD-8]`) |
| `E2224` | folded into `E2225`: `migrate_from` sees a read-only view |
| `E2227` | folded into `[HR-35]`: foreign failure becomes `ReloadError.Foreign` |
| `E3064` | multi-region view structs are legal |
| `E4071` | an undeclared `extern` is `Nondet` (`[DET-2]`), reported as `E4070` |
| `E4073` | generator frames never allocate (`[CORO-1]`) |
| `E6002` | compile-time evaluation is deterministic by construction (`[CT-4]`) |
| `E9012` | reserved |
| `E9032` | reserved |
| `L3018` | the reason category is part of the `# SAFETY(…):` note and optional (`[UNS-8]`) |
| `W0001` | a dangling doc comment is discarded in silence (`[LEX-11]`) |
---

# Part XVIII — Requirements on Implementations

This Part states what every implementation MUST guarantee about the code it produces and the
programs it accepts. How a compiler is organised internally is not specified.

## XVIII.1 The C backend

The reference implementation compiles Ember to C11 and hands it to the platform's C compiler.

* `[CG-C-1]` *(changed in 0.9.9)* **The emitted C has no undefined behaviour.** Checked signed
  arithmetic uses the compiler's overflow builtins (`__builtin_add_overflow` and friends; checked
  helpers on MSVC); wrapping arithmetic is done in the unsigned type and converted back; shifts are
  range-checked before they execute; `MIN // -1` and `MIN % -1` are handled before the C operator
  runs; type punning uses `memcpy`; no object is accessed through an lvalue of an incompatible type.
  The C compiles without warnings under `-std=c11 -Wall -Wextra` (Clang, GCC) and `/W3` (MSVC).
* `[CG-C-2]` *(changed in 0.9.9)* **Accepted programs compile.** A program Ember accepts never produces
  C that the C compiler rejects or warns about; such a case is a compiler defect (`[TST-27]`), as is
  an internal compiler error on any input. The emitted C is deterministic: the same input gives the
  same text.
* `[CG-C-11]` *(new in 0.9.9)* **Floating-point flags.** Every translation unit begins with
  `#pragma STDC FP_CONTRACT OFF` and is compiled with `-ffp-contract=off -fno-fast-math` (Clang, GCC)
  or `/fp:precise` with `#pragma fp_contract(off)` (MSVC), with SSE2 rather than x87 on 32-bit x86,
  and without flush-to-zero. A `@fastmath` or `@fp(contract)` function is emitted in a separate
  translation unit compiled with the relaxed flags, and is not placed in the inline header of
  `[CG-C-3]`, where it would lose them. This is what makes `[TYP-9]` and `[DET-5]` true.
* `[CG-C-3]` **Cross-module inlining.** Because each module is one translation unit, the backend emits
  per package an inline header, included by every module of the package and of its dependants, holding
  a `static inline` definition of every function that is `@inline`; an operator implementation on a
  type of at most 64 bytes; `len`, `is_empty`, `as_span`, `iter`, `next`, a field accessor or a
  read-through method of a standard view or container; or any other function of at most 40 statements
  that makes no foreign call. So the C compiler inlines across modules without LTO. A function emitted
  there is not also emitted with external linkage unless it is exported or its address is taken.
* `[CG-C-3b]` The bodies it exports are part of the interface hash (`[BLD-2]`).
* `[CG-C-3a]` `@inline` is binding: the function is emitted with `__forceinline` or
  `__attribute__((always_inline))`. `@noinline`, `@cold` and `@hot` are passed to the C compiler as
  hints.
* `[CG-C-4]` *(changed in 0.9.9)* **Aliasing facts.** For each loop, the base pointer of every view whose base and length
  are invariant in the loop is hoisted into a local declared `T *restrict` exactly when `[SIMD-3]`
  (including its condition for views reached through a class handle) proves it disjoint from every other view the loop writes or reads; its length is hoisted into a local
  as well, and a view reassigned inside the loop is not hoisted.
* `[CG-C-5]` Every panic function is `_Noreturn` and cold, and each check branches forward to its panic
  call, so the check costs one compare and one predicted branch on the fast path.
* `[CG-C-6]` A loop in vectorisable form (`[SIMD-5]`) is preceded by the host compiler's vectorisation
  pragma.
* `[CG-C-7]` Locals keep their Ember names in the C (transliterated per `[MNG-3]`), so debuggers show
  them.
* `[CG-C-8]` Every emitted statement is preceded by a `#line` naming the Ember construct that
  produced it, and all C lowered from one Ember statement shares one `#line`, so stepping advances one
  Ember statement at a time. Desugared constructs (`for`, `?`, `with`, f-strings, operator calls) are
  attributed to the source syntax, not to their expansion.
* `[CG-C-9]` The build emits debugger visualisers (`.natvis`, GDB and LLDB scripts) that show `Option`,
  `Result`, enums, strings, collections and class handles as Ember values.
* `[CG-C-10]` Stack traces print Ember function paths and `file:line:col`; `ember demangle` converts C
  stacks.

## XVIII.2 Symbol names

* `[MNG-1]` *(changed in 0.9.9)* **Mangling is injective.** A symbol is `em_` followed by each path
  component (package, modules, item) written as its length in decimal and then its text, then for a
  generic instance `G` and the first 16 hex digits of the BLAKE3 hash of the canonical spelling of its
  arguments: module `lib/m_x.em`'s `f` is `em_3lib3m_x1f` and module `lib/m.em`'s `x_f` is
  `em_3lib1m3x_f`. Two distinct items therefore never share a name. A hash collision between two
  instances is reported as `E9040`, naming both, never as an internal error.
* `[MNG-2]` `@export("name")` sets the symbol exactly.
* `[MNG-3]` Non-ASCII identifier characters are transliterated as `_uXXXX_` before the length is taken.
* `[MNG-4]` Object structs, method tables and type information are named `em_obj_`, `em_vt_` and
  `em_ti_` followed by the mangled type.

## XVIII.3 Monomorphisation

* `[MONO-1]` Generic code is instantiated per distinct set of type arguments, from the program's roots
  (`main`, exports, tests, statics); instances are named deterministically and deduplicated at link
  time.
* `[MONO-2]` The compiler records, per generic, how many instances it produced and the time they took;
  `ember build --report=instantiations` prints it.
* `[MONO-3]` `[build] max_instantiations = N` sets a per-generic ceiling (unset by default); exceeding it
  is warning `W2220`, naming the generic and its newest instances.
* `[MONO-5]` A generic is **shareable** at a parameter `T` when `T` appears in its body only as the
  receiver of calls to methods of `T`'s bounds.
* `[MONO-6]` Only for a shareable generic whose instance count exceeds the ceiling of `[MONO-3]`, and
  none of whose calls lies in a loop the compiler judges hot, the compiler MAY emit one shared body
  taking a method table in place of `T`. With no ceiling set, nothing is shared.
* `[MONO-8]` A shared body computes exactly what the specialised ones would; it costs one indirect
  call per bound-method call and allocates nothing.
* `[MONO-9]` A shared body is its own symbol and is deduplicated like any instance; an exported or
  `extern` function is never shared.
* `[MONO-7]` `@always_specialize` forbids sharing for a generic; `@never_specialize` requires it, and is
  `E2223` on a generic that is not shareable, naming the use of `T` that prevents it. Neither changes
  what any program means, and `ember inspect` reports for each generic whether it was specialised or
  shared, and why.

## XVIII.4 Borrow checking

Borrow checking is specified exactly, so that every implementation accepts the same programs
(`[PHIL-13]`). It runs on each function's control-flow graph after type checking, with every
expression broken into single operations (reads, writes, moves, borrows, calls, drops) at points.

* `[BCK-1]` *(new in 0.9.9)* **Loans.** Each borrow expression at a point creates a **loan** of a place,
  shared or mutable. Each reference, view or borrowing closure value carries the set of loans it may
  have been derived from; copying, reborrowing, projecting, passing or returning a value carries its
  loans with it, and a call's result carries the loans of the arguments it may borrow from (`[LT-1]`).
* `[BCK-2]` *(new in 0.9.9)* **Live loans, location-sensitive.** A loan is **live** at a point when some
  value carrying it is used at a later point on some path from that point, with no intervening
  assignment of that value. A loan carried by a function's result is live on the paths that reach
  the `return`, and only on those paths.
* `[BCK-3]` *(new in 0.9.9)* **Conflicts.** At each point, an access to a place is checked against the
  loans live there whose places overlap it (a place overlaps its prefixes and extensions; distinct
  fields, distinct `SoA` columns and the halves of a split do not, `[BRW-4]`): a write, move or drop
  conflicts with any live loan; a read conflicts with a live mutable loan; a new mutable loan
  conflicts with any live loan; a new shared loan conflicts with a live mutable loan. A conflict is an
  error in the `E3020`–`E3029` range, classified per §XVII.6.1.
* `[BCK-4]` *(new in 0.9.9)* **Storage end.** A place whose storage ends (scope exit, `StorageDead`)
  while a loan of it is live is `E3060`/`E3061`.
* `[BCK-5]` *(new in 0.9.9)* **Two-phase borrows.** The mutable loan made for a method's `mut self`
  receiver or a `mut` argument is *reserved* while the call's other arguments are evaluated and
  *activated* when the call starts; during reservation it conflicts only with writes (`[BRW-3]`).
* `[BCK-6]` *(new in 0.9.9)* **Required acceptances.** Because liveness is location-sensitive, these are
  accepted: a borrow whose last use precedes a later mutation (`[BRW-2]`); a borrow returned on one
  branch while the other branch mutates the borrowed place, as in

  ```ember
  fn first_even(xs: MutSpan[int]) -> ref mut int:
      for i in 0..xs.len():
          if xs[i] % 2 == 0:
              return ref mut xs[i]
      xs[0] = 0
      return ref mut xs[0]
  ```

  and a loop that conditionally stores a borrow in a local declared outside it and mutates the
  source on iterations where it did not.
* `[BCK-7]` *(new in 0.9.9)* **Class handles.** An access through a class handle is not a loan of the
  handle's local; it is governed by the exclusivity rules of §VIII.3. A borrow of a field reached
  through a handle creates a loan of that path and additionally keeps the object alive (`[RC-5]`).

## XVIII.5 The runtime

* `[RT-1]` *(changed in 0.9.9)* The runtime `ember_rt` is C11 depending on libc and the OS only.
  Allocation uses the system `malloc`/`realloc`/`free` for alignments up to `alignof(max_align_t)` and
  an aligned allocator above it; `realloc` grows in place where the allocator can. A host may supply
  its own allocator (`[FFI-27]`).
* `[RT-2]` The runtime has no global constructors; the generated `main` calls `ember_rt_init`.
* `[RT-3]` `TypeInfo` holds size, alignment, flags, name, base, drop functions, method tables and, for
  `@reflect` types, field descriptors.
* `[RT-4]` A panic prints `panic at <file>:<line>:<col>: <message>` naming Ember entities (`[DIA-23]`),
  a backtrace outside `shipping`, calls the host's panic hook if set, and aborts.
* `[RT-6]` The runtime ABI version is `EMBER_RUNTIME_ABI`.
* `[RT-5]` Every runtime symbol, macro and header name derives from one constant,
  `EMBER_SYMBOL_PREFIX` (default `ember`); `ember_rt.h` is generated with the literal names so C
  embedders can read it.
* `[RT-7]` *(changed in 0.9.9)* A reference count never wraps: a retain that would overflow panics with
  `reference count overflow on <Class>`.
* `[RT-8]` Counts of `@sync` objects use a relaxed increment for retain and an acquire-release decrement
  for release (weak counts likewise); publication of an initialised object is a release and its
  acquisition an acquire.
* `[RT-10]` *(new in 0.9.9)* **Counting is inline.** The fast path of retain (one increment and an
  overflow test) and release (one decrement and a test for zero) is defined in `ember_rt.h` and
  inlined into the caller; only deinitialisation is out of line. Copying an existing strong handle
  of a `@sync` object is one atomic `fetch_add`, never a compare-exchange loop; `Weak.upgrade` is the
  one retain that may start from zero, and it is a compare-exchange (`[WK-12]`). The deinitialising and resurrection checks of `[OBJ-5]`
  run on the release-to-zero path only.
* `[RT-11]` *(new in 0.9.9)* Allocation statistics are kept per thread, or only in `debug`; no
  allocation updates a shared non-atomic counter.
* `[RT-12]` *(new in 0.9.9)* **Stack overflow faults.** The backend compiles with stack probes
  (`-fstack-clash-protection` where the C compiler has it; MSVC probes by default), so a frame larger
  than a page touches each page in order; the runtime gives every thread it creates a guard page;
  and overflow aborts with `stack overflow in <function>` in every profile. A stack write can never
  land beyond the guard page.

## XVIII.6 Correctness of the implementation

* `[IMP-11]` *(new in 0.9.9)* The reference implementation verifies its intermediate representation
  after every pass in its own debug builds; runs every `run-pass` test through every backend and the
  compile-time evaluator where applicable, requiring identical output; fuzzes the lexer, parser, type
  checker and borrow checker; and compiles the layout tests of `[FFI-5a]` with every supported C
  compiler.
---

# Annex A — Syntax Quick Reference

Every construct below parses under Part III. The block is generated from the conformance fixture
`docs/spec-source/appendix-a.em` (`[TST-6]`).

```ember,fragment
import std.io                                   # module import: binds `io`
import std.thread
from std.math import Vec3, sin as sine          # item import
import c "zlib.h" with (link=["z"]) as zlib     # C header import (Part XVI)

const MAX: int = 1024                           # compile-time constant
static HITS: Atomic[int] = Atomic(0)            # one global value; Sync
static NAMES: Array[str] = ["ann", "bob"]       # initialised once

@derive(Copy)
struct Point:                                   # value type; Eq, Debug, Clone implicit
    x: float
    y: float = 0.0                              # field default
    fn length(self) -> float:                   # borrowed receiver
        return (self.x * self.x + self.y * self.y).sqrt()
    fn scale(mut self, k: float):               # mutable receiver
        self.x *= k
        self.y *= k

open class Script:                              # reference type, subclassable
    let entity: int                             # assigned only in init
    pub(read) health: float = 100.0             # readable everywhere, written here
    fn init(self, entity: int):
        self.entity = entity
    virtual fn on_update(self, dt: float):
        pass

class Door(Script):                             # inherits Script's init
    angle: float = 0.0
    override fn on_update(self, dt: float):
        self.angle = min(self.angle + 90.0 * dt, 90.0)

@sync
class Counter:                                  # shareable across threads; fields fixed after init
    hits: Atomic[int] = Atomic(0)

enum Shape:                                     # tagged union
    Circle(r: float)
    Rect(w: float, h: float)
    Empty

interface Drawable:
    fn draw(self) -> String

extend Shape implements Drawable:
    fn draw(self) -> String:
        return f"{self!r}"

fn area(s: Shape) -> float:                     # parameters are borrowed by default
    return match s:                             # match expression: `=>` arms
        Circle(r) => 3.14159 * r * r
        Rect(w, h) => w * h
        Empty => 0.0

fn fill(mut buf: MutSpan[float], v: float):     # `mut` parameter: in-out
    for x in buf.iter_mut():
        x = v                                   # writes through `ref mut`

fn total(owned xs: Array[int]) -> int:          # `owned`: moved in
    return xs.iter().sum()

fn parse_pair(s: str) -> Result[(int, int)]:    # error type defaults to AnyError
    a, _, b = s.partition(",")
    return Ok((a.trim().parse[int]()?, b.trim().parse[int]()?))    # `?` converts ParseError

gen fn countdown(n: int) -> Generator[int]:     # generator
    for i in (0..=n).rev():
        yield i

fn evens(xs: Span[int]) -> some Iterator[Item = int]:    # opaque return type
    return xs.iter().copied().filter(fn(x) => x % 2 == 0)

fn demo(xs: Array[int], m: Map[String, int], opt: Option[Point]):
    squares = [x * x for x in xs if x > 0]      # list comprehension
    total = sum(x * x for x in xs)              # generator expression, no allocation
    index = {name: i for i, name in NAMES.iter().enumerate()}    # map comprehension
    seen = {1, 2, 3}                            # set literal
    for name, count in m.items():               # a Map iterates keys; items() gives pairs
        println(f"{name}: {count}")
    q = 7 // 2                                  # floor division; `7 / 2` is an error
    if 0 <= q < 10 and q in seen:               # chained comparison, membership
        println(f"{q=} {q:>8}")                 # f-string: `=` form and format spec
    if opt is None:
        return
    if Some(p) = opt:                           # pattern condition
        println(p.length())
    label = "big" if q > 3 else "small"         # conditional expression
    r = ref xs[0]                               # explicit borrow
    double = fn(x: int) => x * 2                # closure (borrows)
    task = owned fn() => println("done")        # closure (owns its captures)
    with scope = thread.scope():                # scoped threads; joins at block end
        scope.spawn(fn() => println(xs.len()))
    defer: println("leaving demo")              # runs at scope exit
    comptime: assert(mem.size_of[Point]() == 16)    # compile-time check
    unsafe: ptr_write_example()                 # unsafe block
    @parallel(chunk=256)
    for i in 0..xs.len():
        consume_item(xs[i])
```
---

# Annex B — Hot Reload (the Dynamic profile)

Hot reload turns a source edit into running behaviour in a live process, keeping existing objects,
statics and open resources. It is part of the Ember Dynamic conformance profile (`[CONF-5]`), is
available in `debug` and `release` builds, and is forbidden in `shipping`.

* `[HR-1]` An edit to one function body in a 50k-line reloadable package MUST be running in the process
  within one second on the reference machine (`[BUD-1]`), with all live object state preserved: budget
  B2 (`[BUD-2]`) for the build, plus load, plan, prepare and commit, measured together over a live set
  of 10,000 instances (`[BEN-8]`).
* `[BEN-8]` The end-to-end reload benchmark times three edits — a function body, a field added with a
  default, and a field whose type changes through `migrate_from` — from save to the new code running.

## B.1 Model

The unit of reload is the package. A package built with reload enabled is a shared library whose
functions are reached through permanent thunks and whose types carry schemas. A reload proceeds in
phases: **plan** (compare schemas; refuse here with a reason), **prepare** (build every migrated
instance; may fail, changes nothing), **commit** (publish; cannot fail), **reclaim** (drop what the
old image owned).

* `[HR-2]` **Reload is transactional.** A failure in plan or prepare leaves the process running the old
  image, with no instance mutated, moved from or dropped, and a report naming the cause. Commit performs
  only pointer stores into memory reserved in prepare: no allocation, check, callback or other fallible
  operation. A failure in reclaim is an ordinary panic after the reload has succeeded.
* `[HR-2a]` Plan reads schemas and the registered set and runs no user code. Prepare allocates every new
  instance in the transaction arena (`[HR-37]`), runs every `migrate_from` against a read-only view of
  the old instance, and computes the old-to-new address map. Commit publishes the instances, patches
  type information, applies the map and repoints the thunks. Reclaim drops the old instances and
  removed fields, running user `drop` code.
* `[HR-3]` A reload is applied only inside `ember_reload_poll()`, and only when no registered thread is
  executing Ember code (each thread's Ember-depth counter is zero); otherwise the poll returns
  `EMBER_RELOAD_UNSAFE_POINT` and does nothing.
* `[HR-3a]` Every thread that can run Ember code is registered: host threads on attach (`[FFI-22]`),
  `thread.spawn` threads, and job workers. `debug` builds assert on entry to Ember code from an
  unregistered thread.
* `[HR-42]` **Publication.** Entering Ember code increments the thread's depth counter with acquire
  ordering and leaving decrements it with release. Commit reads every counter with acquire, writes the
  new thunk targets and type information, and publishes them with one release store to a reload
  generation counter, which a thread entering Ember code next acquires.
* `[HR-42a]` The counter is not a lock: entering Ember code is one relaxed increment and one acquire
  fence whatever the number of threads; the reload side pays the scan over threads.
* `[HR-4]` Old images are never unloaded, so a stale return address or retired thunk target stays
  mapped.
* `[HR-4a]` Statics live in a runtime-owned table, never in image memory.
* `[HR-8]` Each image carries a reload manifest section listing every reloadable function by mangled
  name, every type schema, every static and the protocol version; the runtime reads it rather than the
  platform's export table.

## B.2 Calls and function addresses

* `[HR-5]` A call to a reloadable function goes through its thunk.
* `[HR-6]` Each reloadable function gets one thunk, at a fixed address for the life of the process,
  that loads the current target from the runtime's call table and jumps to it; a reload rewrites the
  table, never the thunk. A reloadable function's address, wherever it can be observed, is its thunk's.
* `[HR-6a]` Hence function values, closure code pointers, method tables, drop glue and `extern "C" fn`
  values stay ordinary code pointers; nothing else in the language changes.
* `[HR-7]` Call-table slots are identified by mangled name (`[MNG-1]`), never by index. A function
  removed by an edit keeps its thunk, now pointing at a stub that panics naming it; a reload that would
  leave such a thunk reachable is refused.
* `[HR-9]` A call through a thunk costs one load and one indirect jump; the measured overhead on the
  performance suite with reload enabled is reported with its configuration and must not exceed 3 %.
* `[HR-9a]` A reloadable function is never inlined, devirtualised or placed in the inline header of
  `[CG-C-3]`, since an inlined body cannot be swapped.
* `[HR-28]` `ember inspect --safety` reports the thunk indirection of each function beside its safety
  checks.
* `[HR-10]` `@noreload fn` is called directly and may be inlined; changing its body requires a restart
  (the reload is refused, naming it). Hot loops and `@static_safe`, `@simd` and `@parallel` bodies should
  be `@noreload`.
* `[HR-10a]` A `@noreload` function may call reloadable functions, through their thunks.
* `[HR-21]` A foreign table of exported function pointers stays valid across every reload, because each
  exported function's address is its thunk. Adding, removing or changing the signature of an
  `@export` function changes the module protocol and is refused: that is a rebuild of the host.

## B.3 Types, schemas and migration

* `[HR-11]` Every type in a reloadable package has a **schema**: its kind, base, layout attributes, and
  the ordered list of fields (name, type schema hash, offset, size) or variants (name, discriminant,
  payload).
* `[HR-11a]` Enum values are migrated by variant name, never by discriminant.
* `[HR-12]` In a reloadable build, class instances are registered in a live-instance list, which adds
  16 bytes to the object header (40 bytes instead of 24); non-reloadable builds keep `[OBJ-1]`'s
  layout.
* `[HR-12a]` The header size is a whole-process property: every package in a process agrees on whether
  reload is enabled, enforced at link time (`E9035`).
* `[HR-13]` Value-typed data (structs in arrays, `SoA` columns, ECS storage, arena contents) is migrated
  through its container, which registers itself with its element type's schema.
* `[HR-13a]` Standard-library containers of a type from a reloadable package register themselves; the
  instantiating package emits the registration, so `std` needs no reload support.
* `[HR-13b]` Value data reachable only through raw pointers or foreign memory cannot be migrated; if its
  schema changed, the reload is refused.
* `[HR-14]` *(changed in 0.9.9)* **Migration is by name**, and this table is exhaustive:

  | Change | Behaviour |
  |---|---|
  | field added with a default | initialised to that default |
  | field added, its type is `Default` | initialised to `Default.default()` |
  | field added, neither | refused, naming the field and suggesting a default |
  | field removed | old value dropped in reclaim |
  | field renamed with `@renamed_from("old")` | value carried over |
  | field renamed without it | a removal and an addition |
  | field type widened losslessly (`[TYP-5]`) | converted |
  | field type changed to a range type | refused unless every value passes `T.checked` |
  | field type changed otherwise | refused unless the type declares `migrate_from` |
  | field of a migrated type | migrated depth-first |
  | fields reordered, `let` changed | carried over by name |
  | methods added, removed or changed | no instance change |
  | base class changed | refused |
  | class removed while instances live | refused, naming the class and the count |
  | enum variant added, reordered or renamed with `@renamed_from` | discriminants rewritten |
  | enum variant removed while a value holds it | refused, naming the variant and the count |
  | enum variant payload changed | the field rows, applied to the payload |
  | layout attributes changed on a type used across FFI | refused |
  | static's initialiser changed | refused unless the static is `@reinit_on_reload` (`[HR-17a]`) |

* `[HR-15]` Migration keeps an instance's address where the new size fits; otherwise commit rewrites
  every reference to it that the runtime can enumerate: strong handles in live instances and
  registered containers, `Weak` handles, and interior references in registered containers.
* `[HR-15a]` A reference the runtime cannot enumerate — in foreign memory, behind a raw pointer — cannot
  be rewritten, and the reload is refused unless `[HR-20]` covers it.
* `[HR-15b]` Every refusal is decided in plan, before prepare runs.
* `[HR-16]` `fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]` replaces field-wise
  migration for its type. `OldSelf` is the previous schema as a generated struct `Old<Name>`, visible
  only in that function.
* `[HR-34]` Prepare cannot panic: every operation it performs is either free of panics or reported as
  a `ReloadError`.
* `[HR-35]` *(changed in 0.9.9)* `migrate_from` cannot abort the process. It may not contain an
  explicit panic (`panic`, `unwrap`, `expect`, `assert`: `E2225`, whose help names the fallible form);
  and every run-time check inside it — overflow, bounds, stale handle — returns
  `Err(ReloadError.Migration(type, instance, message))` instead of panicking.
* `[HR-39]` A foreign call inside `migrate_from` returns its failure as `ReloadError.Foreign`; a call to
  a C++ function asserted `noexcept`, which could terminate the process, is rejected at compile time.
* `[HR-43]` `@allow_reload_terminate` on a `migrate_from` admits such calls and states that this
  migration may end the process rather than refuse; `ember tcb` lists every use.
* `[HR-36]` Allocation during prepare comes from the transaction arena; exhausting it fails the reload
  with `ReloadError.Allocation`, so construction inside `migrate_from` needs no special spelling.
* `[HR-37]` The transaction arena holds everything prepare allocates and is released whole if the
  reload fails.
* `[HR-38]` `std.hot.ReloadError` is `SchemaRefused(type, reason)`, `Migration(type, instance, message)`,
  `Allocation`, `Foreign(error)` or `Timeout`.
* `[HR-17]` A static whose type and initialiser are unchanged keeps its value; one whose type changed
  migrates by `[HR-14]`; a new static is initialised normally.
* `[HR-17a]` *(changed in 0.9.9)* Editing a static's initialiser changes its schema: the reload is
  refused, naming the static, unless the static is `@reinit_on_reload`, which re-runs the initialiser
  and discards the old value. Keeping the old value silently is never an outcome.
* Regions (`[LT-30]`) are compile-time facts and play no part in schemas or migration.

## B.4 Refusal, scope and cost

* `[HR-18]` A refusal returns a report naming each type and change that caused it; it is never a crash.
* `[HR-18a]` A refusal is never partial: nothing of the new image is live.
* `[HR-18b]` `ember build --reload --explain` predicts the outcome against the running process without
  applying anything.
* `[HR-41]` **Failure matrix.**

  | What happened | Detected in | Outcome |
  |---|---|---|
  | source does not compile, or fails checking | before the poll | old image keeps running |
  | reload protocol mismatch | load | refused, `E9037` |
  | C++ header ABI changed | plan | refused (`[HR-23]`) |
  | schema change with no defined outcome | plan | refused (`[HR-14]`) |
  | `@noreload` body changed | plan | refused (`[HR-10]`) |
  | a reference that cannot be rewritten must move | plan | refused (`[HR-15a]`) |
  | allocation exhausted, `migrate_from` fails, or a foreign call fails | prepare | discarded; old image keeps running |
  | GPU resource still in flight past the bound | prepare | discarded (`[HR-24]`) |
  | panic in `migrate_from`, or a failure in commit | cannot occur | forbidden by `[HR-35]` and `[HR-2]` |
  | panic in a `drop` during reclaim | reclaim | the reload has succeeded; the panic aborts as any panic does |

* `[HR-19]` `reload = "bodies"` admits only function-body changes and changes that need no instance
  migration; it needs no schemas and no larger header, and is the first tier an implementation
  provides.
* `[HR-25]` `reload` is `"all"`, `"opt-in"`, `"bodies"` or `"none"` (`[MAN-7]`); `@reloadable` and
  `@noreload` apply to modules and items, item level winning.
* `[HR-20]` *(changed in 0.9.9)* An object pinned by a `Retained` token that foreign code holds is never
  moved: if its new size does not fit, the reload is refused, unless the token was created with an
  `on_relocate` callback, which the runtime calls with the new address during commit.
* `[HR-22]` Foreign objects owned by Ember are carried across unchanged.
* `[HR-23]` A changed C++ header refuses the reload and names it (the host must be rebuilt); a changed
  overlay does not, because overlay edits change only Ember code; where the header is unchanged the
  previous thunks are reused.
* `[HR-24]` An object in use by an in-flight GPU frame is migrated in place when it fits; otherwise the
  reload waits at most `frames_in_flight + 1` polls and is then refused, naming the resource.
* `[HR-27]` A `shipping` build is bit-identical whether or not the source uses any reload attribute.
* `[HR-29]` With reload enabled, one shared runtime serves every image in the process
  (`[build] runtime = "shared"`); a reloadable package that links its own runtime copy is `E9036`.
* `[HR-30]` **Host contract:** `ember_reload_init(&config)` once; `ember_reload_poll()` at a point where
  no Ember frame is live (between frames, for a game); `ember_reload_stats()` for timing;
  `ember_reload_shutdown()`.
* `[HR-31]` Polling never blocks; compilation runs in the background.
* `[HR-32]` `ember_reload_stats()` reports the last reload split into compile, load, plan, prepare,
  commit and reclaim, with the instances migrated and any refusal.
* `[HR-33]` `ember run --hot` is the toolchain's own host: it builds under `debug`, runs the program,
  watches the sources and polls at the point the program declares with `std.hot.checkpoint()`; a
  program that declares none is told so after ten seconds rather than appearing to hang.
---

# Annex C — C++ Interoperation (the Native profile, optional)

Calling C++ is a separate, optional conformance claim (`[CONF-4]`). An implementation provides Part
XVI (C) first; nothing in Parts I–XVIII depends on this annex.

* `[FFI-3]` Ember never links against C++ mangled symbols. The importer generates `extern "C"` thunks,
  compiled by the project's own C++ compiler with the project's flags, and Ember calls the thunks.
* `[FFI-17]` `import cpp "Header.hpp" with (project="engine", overlay="…", instantiate=[…])` parses the
  header with libclang in the configuration of the named `[cpp.<project>]` section and generates one
  thunk per imported function, method, constructor and destructor. Overlays for C++ use
  `overlay cpp "Header.hpp":` with the grammar of `[GRM-35]`.
* `[FFI-50]` *(new in 0.9.9)* The manifest section `[cpp.<project>]` names the C++ project: `compiler`
  (`msvc`, `clang-cl`, `clang`, `gcc`), `standard`, `defines`, `include_paths`, `flags`, and `cmake =
  { build_dir, target }` to read the flags from the CMake File API (`[BLD-FFI-2]`). `ember bind
  --emit-cpp` writes the thunk translation unit for inspection.
* `[FFI-17b]` Templates are available only as explicit instantiations listed in `instantiate=[…]`; an
  Ember generic cannot instantiate a C++ template (`E5055`).
* `[BLD-FFI-1b]` The MSVC runtime-library switch, `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` are inherited
  byte for byte from the C++ project, because they change the layout of standard-library types;
  translation units that disagree are `E9020`, and an undeterminable C++ runtime is `E9021`.
* `[BLD-FFI-4]` Ember's emitted C and the project's C++ are compiled by the same compiler, so the
  project's LTO setting inlines calls across the boundary like any other call.
* `[BLD-FFI-5]` `ember build --emit header` with `--cpp` also writes a C++ header whose declarations
  take `std::string_view` and `std::span` where the C header takes pointer-and-length pairs.
* `[BLD-FFI-5a]` The C++ header gives a copyable smart handle only to `@sync` classes, whose counts are
  atomic; any other class gets a handle type that is neither copyable nor movable across threads, so C++
  code cannot count it from two threads.

## C.1 What imports, and how

* `[FFI-44]` *(changed in 0.9.9)* *Automatic* means no overlay is needed; *overlay* means a contract
  must be written; *native island* means it stays in C++ behind a hand-written boundary.

  | C++ construct | Import |
  |---|---|
  | `extern "C"` functions, POD structs | automatic (Part XVI) |
  | standard-layout, trivially copyable classes | automatic, by value (`[FFI-32]`) |
  | other classes | automatic, opaque behind a pointer; methods through thunks |
  | the standard-library types of `[FFI-17a]` | automatic |
  | explicit template instantiations | automatic (`[FFI-17b]`) |
  | other templates, metaprogramming, concepts, compiler extensions | native island |
  | function-like macros, `std::function` | overlay |
  | single inheritance from a C++ base | automatic, bounded (`[FFI-39]`) |
  | multiple or virtual inheritance | native island (`[FFI-48]`) |
  | exceptions | automatic, with a derived or declared policy (`[FFI-24]`) |
  | custom allocators | overlay or native island |
  | ownership the header does not state | unsafe until `adopt` or a contract (`[FFI-36]`) |

* `[FFI-17a]` *(changed in 0.9.9)*

  | C++ | Ember |
  |---|---|
  | `std::span<T>`, `std::span<const T>` | `MutSpan[T]`, `Span[T]`, zero-copy; a result needs a lifetime word (`[FFI-11]`) |
  | `std::string_view` | `Span[u8]`; `.to_str()` is the fallible conversion to `str` (`[TXT-2]`) |
  | `std::string` | `CppString`; `.to_str() -> Result[str, Utf8Error]`; `String.from(c)` copies |
  | `std::vector<T>` | `CppVector[T]` with `.span()`, `.span_mut()`, `.to_array()`, `push_back`, `len` |
  | `std::unique_ptr<T>` | `ForeignBox[T]`, dropped through the deleter |
  | `std::optional<T>` | `Option[T]` for a `T` that maps by value |
  | `std::shared_ptr<T>`, `std::weak_ptr<T>` | `CppShared[T]`, `CppWeak[T]` — never `Shared`/`Weak` (`[WK-14]`, `E5065`) |
  | `std::variant<…>` | a generated enum, when every alternative maps and the overlay names each |
  | `std::function` | an opaque owned object, or `@ffi(std_function, signature=…)`; never a parameter type (`E5030`) |

* `[FFI-17e]` The bridge types `CppVector[T]`, `CppString` and `CppShared[T]` are declared in `std.ffi`;
  they are not `Copy`, `Send` or `Sync`; the first two drop through their C++ destructors; and a span
  or string obtained from one borrows it, so it cannot outlive it.
* `[FFI-17f]` `ember inspect` reports each bridge operation that is a thunk call rather than an inlined
  one (`CppShared` clone and drop, `CppVector` `push_back`, `len` and drop, `CppString` drop).
* `[FFI-32]` A C++ class that is standard-layout and trivially copyable, all of whose members map, is
  imported as a `@layout(c) struct` with `@derive(Copy)` and the same field offsets; an opaque class
  cannot be constructed from Ember except through its imported constructors (`E5032`), and a parameter
  that maps to nothing is `E5031`.
* `[FFI-11f]` A result lifetime derived from `[[clang::lifetimebound]]` imports as an asserted fact:
  the attribute is not enforced by C++, so it never counts as checked.
* `[FFI-48]` **Unsupported constructs** (`E5034`, naming the row):

  | Construct | Why not | Instead |
  |---|---|---|
  | multiple inheritance, virtual bases | the pointer adjustment is ABI-private and differs between MSVC and Itanium | a single-inheritance facade in C++ |
  | overriding a virtual not named in `virtuals=[…]` | the trampoline has no slot for it | name it (`E5056`) |
  | C++20 modules | no stable AST for a compiled module interface | parse the headers |
  | C++ coroutines | promise and frame layout are implementation-defined | a callback or completion handle |
  | overloads differing only in return type | Ember has no return-type overloading | rename one in the overlay |
  | casts across a hierarchy the importer did not model | no Ember equivalent of `dynamic_cast` there | cast in C++ and export the result |
  | non-type template parameters of class type | mangling differs between compilers | instantiate in C++ and export a typedef |
  | allocator-parameterised containers | the allocator is part of the type and opaque | expose a span, or keep it native |

## C.2 Exceptions

* `[FFI-24]` *(changed in 0.9.9)* **Every imported C++ function has an exception policy, with a
  default.** The policy is derived: a function whose declaration is non-throwing (`noexcept`, a
  `noexcept(expr)` that evaluates true, a destructor, a defaulted special member) is called directly
  and returns `T`. Every other function returns `Result[T, CppError]`: its thunk catches every
  exception and returns it as a `CppError` holding the type name and `what()`.
* `[FFI-24b]` An overlay may assert `@ffi(noexcept)` for a function the header leaves throwing; its
  thunk still catches, and a throw then panics naming the declaration — never an unexplained
  `std::terminate`.
* `[FFI-24c]` `ember inspect` reports each C++ call's mode (`noexcept (derived)`, `noexcept
  (asserted)`, `catching`), and `ember bind --report` lists catching calls on `@noalloc` or main-thread
  paths.
* `[FFI-24d]` A header that adds or removes `noexcept` changes the Ember signature; the diagnostic in
  dependent code names the header, the declaration and the change as the cause, and `ember bind
  --report` lists it under API changes.
* `[FFI-43]` An overlay that marks a function both `noexcept` and throwing, or two composed overlays
  that disagree, is `E5062`; a function whose policy cannot be derived and is not declared is `E5061`.
* `[FFI-39e]` A C++ exception never crosses into Ember code, and an Ember panic never crosses into C++:
  both abort at the boundary if they would.

## C.3 Classes and inheritance

* `[FFI-36]` A C++ function returning a raw pointer with no ownership contract yields `*mut T`;
  `ffi.adopt[T](p)` (`unsafe`) turns it into a `ForeignBox[T]` using the contract's destructor.
* `[FFI-39]` An Ember class may derive from a C++ class the overlay declares with `@ffi(trampoline,
  virtuals=[…])`, which makes the base open and sized; the importer generates a C++ subclass whose named
  virtuals call the Ember overrides (naming a method that is not virtual is `E5057`).
  Its constructor is an ordinary `fn init(self, …)` calling `super.init(…)` exactly once.
* `[FFI-39a]` The trampoline calls the overrides through their permanent thunks, so overrides survive
  hot reload (`[HR-6]`).
* `[FFI-39b]` `super.init(…)` selects the C++ base constructor by arity and argument types and runs it
  before any Ember field is initialised; a base with no default constructor and no declared `init` is
  `E5058`.
* `[FFI-39c]` Passing `self` to a C++ API upcasts it and creates a `Retained` token (`[FFI-23]`) that
  keeps the object alive while C++ holds the pointer; an upcast where the token cannot be kept is
  `E5060`.
* `[FFI-17c]` Destruction of an Ember class derived from a C++ class is derived-first, as `[CLS-6]`
  says for every class: the Ember `drop`, then the Ember fields, and only then the C++ base destructor.
* `[FFI-17d]` `@ffi(trampoline, owner="ember")`, the default, makes the Ember count own the object;
  `owner="foreign"` makes a foreign `delete` destroy it. A trampoline base without a virtual destructor
  is `E5059`.
* `[FFI-39d]` *(changed in 0.9.9)* Re-entrant calls from C++ into the same Ember object are expected.
  The trampoline itself takes no long-term access, but accesses the Ember caller holds across the call
  into C++ stay active and are checked: an override that begins a conflicting access panics
  (`[EXC-1]`), so it can never free what the caller still views. `L3013` warns where a long-term access
  is held across a C++ call that may call back (`[EXC-7]`); the supported idiom is to make such calls
  outside long-term accesses.
* `[FFI-40]` `const` methods import as `self`, non-`const` methods as `mut self`; `&&`-qualified members
  are skipped (`W5033`). A `T&` parameter imports as `mut T`, `const T&` as a borrowed `T`, and `T&&` as
  `owned T` where the type is movable; a `T&` result is a `ref` borrowing the receiver (`[LT-1]`).
* `[FFI-40a]` `const` is not an aliasing guarantee in C++: a `const` method that mutates `mutable` state
  or invalidates iterators must be declared `@ffi(invalidates)`, which gives it `mut self`, and the
  importer assumes `@ffi(invalidates)` for every `const` method of a type with a `mutable` member unless
  the overlay says otherwise (an asserted fact).
* `[FFI-41]` Static member functions import as associated functions and static data members as
  `extern static`; nested types sit under their outer type's name; namespaces become module paths;
  entities in anonymous namespaces are skipped (`W5034`).
* `[FFI-42]` Overloaded operators map to Ember operator interfaces where one exists (`operator[]` to
  `Index`/`IndexMut`, `operator*`/`->` on a smart-pointer-like type to read-through); C++ iterators with
  `begin`/`end` become `Iterable`.
* `[FFI-42a]` Iterating an imported C++ container borrows it mutably for the loop, so any method that
  could invalidate its iterators — every `mut self` method, including `@ffi(invalidates)` ones — is
  rejected inside the loop by the ordinary borrow rules.

## C.4 Trust and evidence

* `[TCB-1]` *(changed in 0.9.9)* Every fact an overlay states about foreign code carries a grade:
  **asserted** (someone wrote it; the default), **instrumented** (a test run observed no
  counterexample on the paths exercised), **checked** (verified against the header) or **proven**.
  The grade is written as an attribute on the overlay item, `@grade(instrumented)`. `ember tcb
  [--module m]` prints every assumption the program's safety rests on, grouped by module. A claimed
  grade whose evidence is missing or stale is `W5050`, and `E5050` under `ember build
  --require-evidence` or `ember tcb --require`. Evidence records live under `[ffi] evidence` (default
  `.ember/ffi-evidence`).
* `[TCB-2]` The report separates what the language guarantees from what external components supply,
  and writing a grade never raises it: no fact above asserted comes from declaration alone.
* `[TCB-3]` Every assumption a guarantee relies on appears in the report with its source and grade.
* `[TCB-4]` Entries are categorised — language, compiler, runtime, standard library, unsafe, C FFI,
  C++ bridge, external library, driver, hardware — and the compiler is in the list, with its version
  and known-defect list. The unsafe section lists each `unsafe` block and function with its note.
* `[TCB-5]` An instrumented fact is valid only for the foreign library, header, overlay, toolchain and
  test binary it was measured against; its evidence record stores their identities and hashes.
* `[TCB-6]` A change to any identity input makes the record stale; a change to an environment input
  (machine, timing) is reported but does not.
* `[FFI-37]` `ember test --instrument-ffi` runs the tests with the allocation, lock, wait and I/O entry
  points intercepted, to grade declared effect facts as `instrumented`.
* `[FFI-37a]` The intercepted set is published and recorded in the evidence; an allocation through a
  mechanism outside it (a custom pool, a driver) is reported as structurally unobservable, never as
  evidence.
* `[FFI-37b]` The evidence records how often each foreign function was called and from how many call
  sites; a fact never exercised is reported `unexercised` (`W5054`) and does not count.
* `[FFI-37c]` `instrumented` means "no counterexample was observed on the paths exercised", is shown as
  partial when not every call site ran, and is never shown as "holds"; a contradicted fact is `E5053`.
* `[FFI-37d]` Only effect facts (`Alloc`, `Block`, `Lock`, `Io`) can be `instrumented`; ownership,
  nullability, lifetime, aliasing and exception facts stay asserted unless checked or proven.
* `[CXX-1]` *(changed in 0.9.9)* The conformance suite for this annex includes a corpus of real headers
  and at least one third-party header-only library, each with its expected outcome recorded (by value,
  opaque, behind an overlay, or refused with a named diagnostic) and its generated thunks committed and
  diffed; it runs under MSVC and Clang, with debug and release runtimes and RTTI on and off, and two
  configurations that disagree must fail with `E9020`, never bind differently.
---

# Annex D — GPU Host Model (`std.gpu`)

`std.gpu` is a library for driving a GPU from Ember — handles, frames, command recording, deferred
destruction and shader interfaces — over an abstract device that a host renderer implements through
the FFI, or that an Ember backend implements directly. It adds nothing to the language. Ember does not
compile shaders in this version.

```ember,fragment
type TextureHandle = Handle[gpu.Texture]
type BufferHandle = Handle[gpu.Buffer]

interface Device:
    fn begin_frame(mut self) -> Option[Frame]           # None: skip this frame; no end_frame
    fn end_frame(mut self, owned frame: Frame)
    fn create_texture(mut self, desc: TextureDesc) -> Result[TextureHandle, GpuError]
    fn create_buffer(mut self, desc: BufferDesc) -> Result[BufferHandle, GpuError]
    fn destroy(mut self, h: TextureHandle)                 # physical destruction is deferred
    fn frames_in_flight(self) -> int
```

* `[GPU-1]` *(changed in 0.9.9)* GPU objects are named by generational handles (`[HND-1]`), which are
  `Copy` and `Send`. A stale handle is detected by its generation **in every profile**
  (`panic: stale TextureHandle (generation 7, current 9)`); there is no setting that removes the
  check.
* `[GPU-2]` `device.begin_frame() -> Option[Frame]`; `None` is normal (a minimised window). `Frame` is
  move-only and `end_frame(frame)` consumes it, so ending a frame that never began is a type error.
* `[GPU-3]` `frame.arena()` is the frame's transient allocator; its region is the `Frame`, so nothing
  allocated from it outlives `end_frame`.

## D.1 Recording and access states

```ember,fragment
fn draw(mut frame: Frame, pipeline: PipelineHandle, vbo: BufferHandle, index_count: int):
    with cmd = frame.command_list():
        cmd.begin_render_pass(RenderPassBegin(target=None))    # None: the swapchain
        cmd.bind_pipeline(pipeline)
        cmd.bind_vertex_buffer(0, vbo)
        cmd.draw_indexed(index_count)
        cmd.end_render_pass()
```

Each resource has an access state the device tracks: owned by the CPU; recorded in frame N; in
flight in frame N until that frame's fence is observed; retired (destroyed while in flight, freed
later).

* `[GPU-4]` *(changed in 0.9.9)* Writing or mapping a resource that the GPU may still be reading is a
  panic in every profile (`buffer is in flight (frame 42); use a per-frame ring or wait`).
  `Ring[T, N]` holds one resource per frame in flight (`ring.current(frame)`).
* `[GPU-5]` Binding a resource declares its access; `cmd.read(h)`/`cmd.write(h)` declare it
  explicitly; barriers are derived from the declarations; `cmd.barrier(…)` remains available.
  `unsafe: cmd.native()` exposes the backend's command buffer.
* `[GPU-6]` `device.destroy(h)` invalidates the handle at once and destroys the object after the GPU
  has finished the frames that may use it. Dropping the device waits for the GPU and flushes every
  retirement list.
* `[GPU-7]` In `debug`, dropping a device while handles are still live reports each leaked handle
  with the place it was created.
* `[GPU-8]` `History[T]` owns the per-frame copies of a temporal resource
  (`gpu.History[TextureHandle].new(device, desc, count=2)`) and gives `(prev, cur)` each frame,
  forbidding reads of `cur` and writes of `prev` within a frame.
* `[GPU-9]` `std.gpu.graph` is an optional render-graph layer over `CommandList`: passes declare reads
  and writes, `graph.compile()` orders them, aliases transient resources with disjoint lifetimes and
  inserts transitions, and `pass.native(fn(cmd) => …)` records directly inside a graph.

## D.2 Shader interfaces

* `[GPU-10]` `ember shader-bind <reflection.json>`, reading the reflection a SPIR-V toolchain produces,
  generates a module with a `@gpu_layout(std140 | std430)` struct per uniform or storage block, with
  compile-time layout assertions (`E8001` on a mismatch), a typed binding interface, the push-constant
  struct and the vertex-input layout; binding a resource set built for another interface is a type
  error.
* `[GPU-11]` Ember assumes no shader language: anything that produces SPIR-V and reflection works.
* A kernel language (`@gpu fn`) is reserved for a later version; `@gpu` and `std.gpu.kernel` are
  reserved names.
---

# Appendix E — Coming from Python *(non-normative)*

Ember reads like Python and runs like C. Most Python habits carry over unchanged; the ones that do not
are rejected with a fix-it (`[DIA-21]`) rather than given a different meaning (`[PHIL-14]`).

## E.1 What carries over

| Python | Ember | Notes |
|---|---|---|
| indentation, `:` blocks, `#` comments | same | spaces only (`[LEX-4]`) |
| `x = 1` declares | same | a name assigned in every branch is declared after the `if` (`[CTL-10]`) |
| `a, b = b, a`; `return a, b` | same | tuples (`[GRM-29]`) |
| `[1, 2]`, `{"k": v}`, `{1, 2}` | same | `Array`, `Map` (insertion-ordered like `dict`), `Set` |
| `[x * x for x in xs if x > 0]` | same | also map and set comprehensions |
| `for x in xs:` … `else:` | same | |
| `for i, x in enumerate(xs):` | same | also `xs.enumerate()` |
| `len(xs)`, `range(n)`, `sum`, `sorted`, `zip`, `reversed`, `any`, `all` | same | Python's meaning (`[STD-26]`); `len` of a string is rejected, because Ember's counts bytes |
| `for k in d:`, `for k, v in d.items():` | same | a `Map` iterates its keys; `m.values()` its values |
| `x in xs`, `x not in xs` | same | cost reported by `ember inspect --cost` |
| `a < b < c` | same | `b` evaluated once |
| `x is None` | same | on `Option` |
| `7 // 2`, `-7 // 2 == -4`, `-7 % 2 == 1` | same | floor semantics |
| `7 / 2` on ints | `7 // 2` or `7 as float / 2` | integer `/` is rejected (`[TYP-28]`) |
| `2 ** 10` | same | overflow panics instead of growing |
| f-strings with `=`, `!r`, format specs | same | |
| `print(a, b, sep=", ", end="")` | same | |
| `input("> ")` | same | panics at end of input (`[STD-10]`) |
| keyword arguments, defaults | same | |
| `lambda x: x + 1` | `fn(x) => x + 1` | |
| generators, `yield` | `gen fn f() -> Generator[T]:` | |
| `with open(p) as f:` | `with f = fs.File.open(p)?:` | the file closes at the end of the block, or whenever it is dropped |
| `try` / `except` / `raise` | `Result`, `?`, `match`, `return Err(e)` | Part XIII |
| classes, inheritance, `super()` | `class`, `open class`, `super.init(…)` | handles are reference-counted; `Weak` for back-pointers |
| `@dataclass` | `struct` | equality, debug printing and cloning are automatic |
| `list.sort()`, `sorted(xs)` | same, or `xs.sorted()` | stable |
| `dict.get(k, d)` | `m.get_or(k, d)` | `m.get(k)` returns an `Option` |
| `d[k] = v`, `d[k]` | same | a missing key panics like `KeyError` |

## E.2 Names that differ (`[STD-13]`)

| Python | Ember |
|---|---|
| `len(s)` for a string | `s.char_count()` (characters, Python's count) or `s.len()` (bytes) |
| `str(x)`, `repr(x)` | `x.to_string()` or `f"{x}"`; `f"{x!r}"` |
| `int(s)`, `float(s)` | `s.parse[int]()`, `s.parse[float]()` (a `Result`) |
| `int(x)` for a float | `x as int` (saturates; NaN is 0) |
| `range(a, b)` in a loop | `a..b` is the usual spelling; `range` also works |
| `xs.append(x)`, `xs.extend(ys)` | `xs.push(x)`, `xs.extend(ys)` |
| `xs.pop()` | `xs.pop()` returns an `Option` |
| `xs.index(x)` | `xs.index_of(x)` (an `Option`) |
| `xs[-1]` | `xs.last()` or `xs[xs.len() - 1]` (negative indices panic) |
| `xs[a:b]` | `xs[a..b]` |
| `s.strip()`, `s.lstrip()`, `s.rstrip()` | `s.trim()`, `s.trim_start()`, `s.trim_end()` |
| `s.split()` | `s.split_whitespace()` |
| `s.splitlines()` | `s.lines()` |
| `s.startswith(p)`, `s.endswith(p)` | `s.starts_with(p)`, `s.ends_with(p)` |
| `s.upper()`, `s.lower()` | `s.to_upper()`, `s.to_lower()` |
| `s.find(t)` (returns -1) | `s.find(t)` (returns `Option[int]`) |
| `", ".join(xs)` | `xs.join(", ")` |
| `True`, `False`, `None` | `true`, `false`, `None` (only as an `Option`) |
| `def f():` | `fn f():` |
| `if xs:` | `if not xs.is_empty():` |
| `abs`, `min`, `max`, `round` | `abs`, `min`, `max`; `x.round()` rounds half to even, as Python's does |

## E.3 What is new

* **Types are checked before the program runs.** Most are inferred; function parameters and struct
  fields are written.
* **Values have one owner.** Assigning or passing a list moves it unless the parameter only borrows it
  (the default). A moved name cannot be used again; the error says where it moved and offers
  `.clone()`.
* **Objects are freed when their last handle goes**, at a predictable point, not by a collector. A
  cycle of strong handles leaks; the debug build reports it at exit, with the field to make `Weak`.
* **Integers are 64-bit and overflow panics.** `@overflow(wrap)` or `wrapping_add` when wrapping is
  wanted.
* **Threads run in parallel.** There is no global lock; the compiler rejects data races instead
  (Part XI).

## E.4 One program, side by side

A dice game in Python:

```python
import random

class Player:
    def __init__(self, name):
        self.name = name
        self.hp = 10
        self.inventory = []

def roll(rng):
    return rng.randint(1, 6)

rng = random.Random(42)
player = Player("Ada")
counts = {}
for turn in range(5):
    r = roll(rng)
    counts[r] = counts.get(r, 0) + 1
    if r == 6:
        player.inventory.append("gem")
    else:
        player.hp -= r // 2
print(f"{player.name}: hp={player.hp}, items={len(player.inventory)}")
for face in sorted(counts):
    print(face, counts[face])
```

The same game in Ember:

```ember
import random

class Player:
    name: String
    hp: int = 10
    inventory: Array[String] = []

fn roll(mut rng: random.Rng) -> int:
    return rng.int_in(1..7)

rng = random.Rng.seeded(42)
player = Player("Ada")
counts: Map[int, int] = {}
for turn in range(5):
    r = roll(rng)
    counts[r] = counts.get_or(r, 0) + 1
    if r == 6:
        player.inventory.push("gem")
    else:
        player.hp -= r // 2
println(f"{player.name}: hp={player.hp}, items={len(player.inventory)}")
for face in sorted(counts):
    println(face, counts[face])
```

What changed: fields are declared, with types, in the class body; `roll` says it changes `rng`
(`mut`); `append` is `push` and `dict.get(k, d)` is `get_or`; the generator is seeded explicitly and
draws from a half-open range; `print` with a newline is `println`. Everything else — the script at
the top level, `range`, `len`, `sorted`, iterating a map's keys, f-strings, `//` — reads as in Python.
The two programs print different numbers only because their generators differ.
---

# Appendix F — Glossary

| Term | Meaning |
|---|---|
| **access (long-term, instantaneous)** | a use of a class object's field; long-term accesses span time and are checked for exclusivity, instantaneous ones (`Copy` field reads and writes) are not (§VIII.3) |
| **access state** | a GPU resource's lifecycle: owned by the CPU, recorded, in flight, retired (Annex D) |
| **arena** | an allocator that hands out memory by bumping a pointer and frees it all at once (§IX) |
| **borrow** | a reference to a place that does not own it; shared (read) or mutable (read and write) |
| **class** | a reference type: instances live on the heap, handles are counted, identity is observable (Part VIII) |
| **contract** | an attribute stating an effect a function must not have (`@noalloc`), checked over the call graph (§X.2); for FFI, the facts about a foreign pointer that a header does not state (§XVI.4) |
| **effect** | a kind of thing a function may do — allocate, block, lock, perform I/O, panic, run a run-time check — inferred by the compiler (§X.1) |
| **`.embind`** | the cached, content-addressed result of importing a header with its flags and overlay (`[FFI-14]`) |
| **enforcement ladder** | prove a property statically, else check it at run time, else require `unsafe` (`[PHIL-8]`) |
| **exclusivity** | the rule that a write access to an object never overlaps another access to it (§VIII.3) |
| **fix-it** | a suggested edit attached to a diagnostic that tools can apply mechanically |
| **generator frame** | a `gen fn`'s suspended state as a sized value; resuming runs the body to the next `yield` (`[CORO-1]`) |
| **handle** | a class reference (counted), or a generational index into a `Pool` (`Handle[T]`) |
| **interior mutability** | mutation through a shared borrow, permitted by `Cell`, `RefCell`, `Atomic` and the locks |
| **loan** | the record a borrow creates, which the borrow checker follows to the borrow's last use (§XVIII.4) |
| **mode** | how a parameter is passed: borrowed (the default), `mut` or `owned` (`[FN-2]`) |
| **move** | transfer of a value's ownership; the source can no longer be used |
| **native island** | a subsystem kept in C or C++ behind a typed boundary rather than imported (Annex C) |
| **niche** | an invalid bit pattern of a type that `Option` uses for `None`, so `Option[T]` costs no space |
| **`Nondet`** | the effect of an operation whose result may differ between runs or machines (`[DET-2]`) |
| **overlay** | an Ember file that states contracts, renames and wrappers for an imported header (§XVI.4) |
| **owned callable value** | a function value that owns its captures, stored in a field, local or collection (`[CLO-3]`) |
| **panic** | a failure that means the program is wrong; it prints a message and aborts the process (`[PAN-1]`) |
| **permanent thunk** | the fixed address standing for a reloadable function; a reload changes its target, never its address (`[HR-6]`) |
| **place** | an expression denoting a storage location: a local, field, element or dereferenced reference |
| **profile** | `debug`, `release` or `shipping`: optimisation and diagnostic settings that never change meaning (`[PRF-1]`) |
| **proof-carrying value** | a value returned by a verifying operation (`assert_disjoint`) that carries the fact it established |
| **reason code** | why a run-time check exists (`[EFF-11]`) |
| **region** | the stretch of a program during which a borrow is live; never written in source |
| **region vector** | the compiler-internal regions of a view type with several borrowed fields; erased before code generation (`[LT-14]`) |
| **reload manifest, reload transaction, safe point** | the image section listing what can be reloaded (`[HR-8]`); the plan and prepare phases that may fail without effect (`[HR-2]`); the poll at which no thread runs Ember code (`[HR-3]`) |
| **Safe Ember** | code outside `unsafe` blocks and functions; the memory-safety guarantee covers it (`[PHIL-10]`) |
| **`Send`, `Sync`** | may be moved to another thread; may be read from several threads at once (§XI.1) |
| **shape** | a category of error with a required help text (§XVII.6) |
| **shareable generic** | one whose type parameter is used only to call its bounds' methods, so one shared body can serve every instance (`[MONO-5]`) |
| **SoA** | structure of arrays: one array per field (`SoA[T]`, §XII.1) |
| **source parameter** | a parameter a returned reference or view may borrow from without `@borrows`: a reference or view, or a borrowed or `mut` parameter whose type is not `Copy` (`[LT-1]`) |
| **static** | a program-wide value with a fixed address; immutable, with interior synchronisation for mutation |
| **`@sync` class** | a class whose handles may cross threads; its fields are immutable after construction (`[THR-1]`) |
| **trampoline subclass** | the generated C++ subclass through which an Ember class extends a C++ base (`[FFI-39]`) |
| **value world, object world** | code over structs, views and containers, checked statically; code over class handles, counted and checked at run time |
| **view** | a value that borrows: a reference, `Span`, `MutSpan`, `str`, or a struct holding one (`[TYP-34]`) |
| **zero-cost** | for a use whose checks are all discharged, emitted code with nothing equivalent C would not contain (`[COST-1]`) |
---

# Appendix G — Resolution of Findings F-001–F-214

Every finding of the 2026-09-23 research pass (`tasks/audit/FINDINGS.md`) and how this revision
resolves it. **SPEC**: the language text changed or gained a rule. **IMPL**: the rule stands or was
clarified and the implementation must meet it. **GATE**: a test or CI obligation now in the text.
**OUT**: outside a language specification, with the reason. **WDN**: withdrawn.

| Finding | Kind | Resolution | Rules |
|---|---|---|---|
| F-001 — `tasks/audit/TASKS.md` is cited by every reproducer and exists on no branch | OUT | Audit bookkeeping, not language text; `FINDINGS.md` Part D rebuilt the missing index. | — |
| F-002 — `()` is not accepted as a value of type `void` | IMPL | `()` is the value of `void`; `Ok(())` is the success value of `Result[void, E]`. | `[TYP-27]` |
| F-003 — `import a.b.c` does not bind `c` | IMPL | `import a.b.c` binds `c`; unknown modules are `E1060`. | `[MOD-3]` |
| F-004 — importing a module that does not exist is silently accepted | SPEC | An import of a missing module is `E1060`; nothing is accepted silently. | `[MOD-3]`, `[PHIL-12]` |
| F-005 — every E2020 type mismatch carries a note about numeric conversion | SPEC | The numeric-conversion note appears only when both operands are numeric. | `[TYP-4]` |
| F-006 — N1 suggestion quality | SPEC | Suggestion threshold scales with name length and kind. | `[DIA-24]` |
| F-007 — the showcase program passes a `str` literal where a `String` is required | SPEC | A string literal initialises a `String` where one is expected. | `[TXT-9]` |
| F-008 — one bad character reports once per byte | SPEC | A run of invalid characters is one diagnostic. | `[DIA-20]` |
| F-009 — "not implemented in this phase" reuses `E1010` | SPEC | Unimplemented constructs are `E0900`, never a name-resolution code. | `[PHIL-12]`, `[CLI-19]` |
| F-010 — cascades after a failed import | SPEC | Names bound to a failed import produce no further errors. | `[DIA-14]` |
| F-011 — cascade after a private constructor | SPEC | One error per failed construction; privacy has its own code. | `[DIA-14]`, `[MOD-2]` |
| F-012 — `println` of any `String`, including every f-string, emits C that does not compile | IMPL | `print`/`println` are specified; an accepted program that yields bad C is a compiler defect, caught by the C gate. | `[STD-9]`, `[CG-C-2]`, `[TST-27]` |
| F-013 — "accepted by Ember, rejected by the C compiler" is a whole defect class with no gate | GATE | Every accepted test program is compiled by each host C compiler with warnings as errors. | `[TST-27]`, `[CG-C-2]` |
| F-014 — a name assigned in every branch of an `if`/`else` is not visible after it | SPEC | A name assigned in every branch at one type is declared after the branch. | `[CTL-10]` |
| F-015 — `if x = 5:` (a `==` typo) gets a misleading fix | SPEC | `if x = 5:` gets the `==` fix-it. | `[DIA-21]` |
| F-016 — the 0.8.2c change log says there is no if-let; the grammar has one | SPEC | Pattern conditions are in the grammar; 0.9.9 carries no stale change log. | `[GRM-19]` |
| F-017 — the EBNF still contains what `[GRM-16]` and `[GRM-11]` delete | SPEC | Part III is a complete grammar with no deleted productions. | `[GRM-11]`, `[GRM-16]` |
| F-018 — two path separators, `.` and `::` | SPEC | One path separator, `.`; `::` is not a token. | `[GRM-24]`, `[LEX-21]` |
| F-019 — `as` binds tighter than unary minus | SPEC | Prefix `-`/`~` bind tighter than `as`. | `[TYP-6]` |
| F-020 — `@deprecated` has two signatures | SPEC | `@deprecated(since, note)` is the one form. | `[VER-3]` |
| F-021 — `@nopanic` is listed as v2 in the attribute table, while the 0.8.4_Hardened_1 change log calls `@nopanic(explicit)` one of Ember's current c | SPEC | `@nopanic(explicit)` is a current contract; bare `@nopanic` is `E0104`. | `[EFF-17]` |
| F-022 — a pasted editorial instruction inside `[BLD-11]` | SPEC | Rule rewritten; no editorial instructions remain in rule text (end check). | `[BLD-11]` |
| F-023 — `E1020` means two things in the compiler | SPEC | `E1020` means only a duplicate declaration; visibility errors are `E1052`. | `[GRM-4]`, `[MOD-2]` |
| F-024 — `[GRM-14]` adds a third kind of generic parameter (`access P`) for one library type, `std.ecs.Query` | SPEC | `Query` uses the ordinary marker type `Mut[T]`; no access-mode generic kind. | `[ECS-3]` |
| F-025 — `[LEX-22]` reserves `'a` for "v2 named lifetimes" | SPEC | Ember has no lifetime syntax, now or later. | `[LEX-22]` |
| F-026 — `[LEX-2]`: the formatter writes the platform's line ending by default | SPEC | The formatter writes LF everywhere. | `[FMT-1]` |
| F-027 — `[LEX-15]` still says "The reserved set therefore has 48 entries" | SPEC | The reserved-word count is stated once, correctly. | `[LEX-15]` |
| F-028 — the Part I.4 table still promises "~2 ns per access pair" | SPEC | The I.4 table states check mechanisms, not nanosecond promises. | `[PHIL-15]` |
| F-029 — `xs = Array[i32]()` is rejected: the explicit type argument is ignored | IMPL | Explicit constructor type arguments fix the type. | `[TYP-18]` |
| F-030 — `min`, `max` and `clamp` are used as free functions and declared nowhere | SPEC | `min`, `max`, `abs`, `clamp` are prelude functions. | `[MOD-5]` |
| F-031 — strings are the biggest Python-ergonomics gap | SPEC | Literal-to-`String` coercion, `String + str`, `+=`, `to_string`. | `[TXT-9]`, `[TXT-11]` |
| F-032 — `[TYP-15]` and `[TYP-15a]` disagree about containers of static views | SPEC | One rule for where views may be stored; containers of static `str` are legal. | `[TYP-15]` |
| F-033 — `<<` overflow is undefined by the text | SPEC | Shift amount checked; bits shifted out are not overflow; `>>` arithmetic on signed. | `[TYP-10]` |
| F-034 — `[TYP-13]` guarantees a niche for `*fn` | SPEC | The niche list names `extern fn`, which exists. | `[TYP-13]` |
| F-035 — `as?` / `as!` are used in IV.6 and absent from the grammar | SPEC | `as?` and `as!` are single tokens in the grammar. | `[GRM-30]`, `[LEX-21]` |
| F-036 — `@view` is required on a struct the compiler already knows is a view | SPEC | View-ness is inferred; `@view` is optional documentation. | `[TYP-34]` |
| F-037 — the default integer is `i32` | SPEC | `int` is `i64`, `float` is `f64`; literals default to them after context. | `[TYP-1]`, `[LEX-16]` |
| F-038 — floats have no `Ord`, so sorting a list of floats needs ceremony | SPEC | Floats implement `Ord` by totalOrder; operators stay IEEE; `sort()` works. | `[TYP-37]` |
| F-039 — `[RNG-10]` carries `[RNG-10a]`, `[RNG-10b]` and `[RNG-10c]` inline in one bullet | SPEC | Each range rule is its own bullet. | `[RNG-10]` |
| F-040 — the orphan rule `[TYP-20]` is not enforced, and as written it is stricter than Rust's | SPEC | Coherence is per package: an `extend` may sit in any module of the package declaring the type or interface. | `[TYP-20]` |
| F-041 — "interface methods need the interface imported" (`[TYP-24]` step 2) is not enforced | SPEC | Interface methods resolve without importing the interface when one visible implementation provides them. | `[TYP-24]` |
| F-042 — no table says which standard interfaces the built-in types implement | SPEC | A normative table of which built-in types implement which interfaces. | `[TYP-36]` |
| F-043 — which interface `<` uses is unclear, and `Ord` excludes floats | SPEC | Comparison operators are built in for scalars and `Ord.cmp` in generic code; no `PartialOrd`. | `[TYP-37]` |
| F-044 — `Formatter`, `FmtError`, `Ordering` and `Into` are used by the interface sketch and declared nowhere | SPEC | `Ordering` is in the prelude; `Formatter`, `FmtError` are declared in `std.fmt`. | `[MOD-5]`, `[STD-18]` |
| F-045 — three iteration interfaces | SPEC | Three interfaces: `Iterator`, `Iterable`, `IntoIterator`; `iter_mut` is a method convention. | `[CTL-1]`, `[TYP-36]` |
| F-046 — eight selectable language versions before 1.0 | SPEC | Before 1.0 there is one language; no version selectors. | `[VER-8]` |
| F-047 — the `[MOD-5]` prelude list is incomplete against the rest of the document | SPEC | The prelude table is the single authoritative list. | `[MOD-5]` |
| F-048 — `Map` and `Set` are not in the prelude | SPEC | `Map` and `Set` are prelude names. | `[MOD-5]` |
| F-049 — Python chained comparison is rejected | SPEC | Comparisons chain with Python meaning; the middle operand is evaluated once. | `[GRM-25]` |
| F-050 — `[GRM-23]` gives `a in b in c` code `E0104` | SPEC | `a in b in c` is `E0102`. | `[GRM-23]` |
| F-051 — `extend[T: Display] Array[T] implements Display` (V.6) is not in the grammar | SPEC | `extend [T: B] Array[T] implements I:` is in the grammar. | `[GRM-34]` |
| F-052 — impl granularity is inconsistent | SPEC | Inherent and interface extensions both follow package granularity. | `[IFC-1]`, `[TYP-20]` |
| F-053 — three rules stated twice, verbatim, inside themselves | SPEC | Rules are written once; the rewritten parts carry no duplicates. | `[GRM-20]`, `[GRM-8d]` |
| F-054 — every struct needs `@derive(...)` boilerplate | SPEC | `Eq`, `Debug`, `Clone` are implicit when fields allow; `@no_derive` opts out. | `[STR-5]` |
| F-055 — no runtime-initialised globals in v1 | SPEC | Statics may have run-time initialisers: lazy, once, thread-safe. | `[STA-1]`, `[STA-3]` |
| F-056 — `fn main(args: Span[str])` | SPEC | `main(args: Array[String])` decodes lossily; raw bytes via `args_os()`. | `[FN-8]` |
| F-057 — undeclared names in normative examples | SPEC | Examples either compile or are marked `ember,fragment`; math names are imported or methods. | `[TST-7]` |
| F-058 — a closure cannot be returned or boxed; the spec's own form is rejected | SPEC | Callable types outside parameters are owned callable values; returning a closure is `-> fn(A) -> R`. | `[CLO-3]`, `[CLO-10]` |
| F-059 — what a `fn(A) -> R` type means outside a parameter is unspecified | SPEC | Position-dependent meaning of `fn(A) -> R` is specified. | `[CLO-3]` |
| F-060 — calling a callable field needs a temporary | SPEC | `b.on_click()` calls a callable field when no method of that name exists. | `[CLO-11]` |
| F-061 — a boxed once-callable is not callable (`[CLO-6a]`) | SPEC | An owned `once fn`, boxed or not, is callable; the call consumes it. | `[CLO-6a]` |
| F-062 — a class `gen fn` with `mut self` contradicts `[CORO-6]` or `[CLS-7]`, and the text does not say which | SPEC | Class generator methods take `self`; no class long-term access spans a `yield`. | `[CORO-12]` |
| F-063 — `Coroutine[R]` names one type parameter; a coroutine has two | SPEC | `Generator[Y, R]` names both the yield and the return type. | `[CORO-1]` |
| F-064 — temporaries in an `if`-condition live through the `else` | SPEC | Temporaries of an `if` condition are dropped before `else`. | `[EXP-4]` |
| F-065 — `%` follows C (sign of the dividend) while the syntax follows Python | SPEC | `%` and `//` have Python floor semantics; `rem_trunc`/`div_trunc` give C's. | `[TYP-28]` |
| F-066 — integer `**` with a run-time negative exponent | SPEC | `**` on integers: negative exponent panics (constant: `E2151`); overflow panics. | `[TYP-30]` |
| F-067 — a method call borrows all of `self`, so the classic Rust partial-borrow error is back | SPEC | Private methods borrow only the fields they touch. | `[BRW-10]` |
| F-068 — the Python swap `a[i], a[j] = a[j], a[i]` is a parse error | SPEC | Unparenthesised tuples on the right of `=` and after `return`. | `[GRM-29]` |
| F-069 — premise disproved by probe; see F-168 | WDN | Withdrawn: premise disproved by probe (see F-168). | — |
| F-070 — `@borrows(arena)` is mandatory on every Arena wrapper, at every nesting level | SPEC | Arena elision: a view returned from a function whose only view source is one `Arena` borrows it. | `[LT-44]` |
| F-071 — the `@borrows` example contradicts the rule and its own semantics | SPEC | `@borrows` stands on its own line; the example no longer contradicts it. | `[LT-1a]` |
| F-072 — `[DRP-4]` is unenforceable as written | SPEC | A panic in `drop` aborts; stated as behaviour, not an unenforceable obligation. | `[DRP-4]` |
| F-073 — writing a class field through a handle from outside a method fails | SPEC | Class fields are written through `self` or any handle without `mut`; each access is checked. | `[CLS-7]`, `[FN-9]` |
| F-074 — no way to return an iterator without naming its concrete type | SPEC | Opaque `some I` returns and generators. | `[TYP-32]`, `[CORO-1]` |
| F-075 — `split_at_mut` is named by `[BRW-5]` and `[TST-25]` (item 8); the API is `split_at` | SPEC | One API name: `split_at`. | `[SPN-5]` |
| F-076 — VII.7's example has statements at file scope | SPEC | The example is inside `main`. | `[GRM-2]` |
| F-077 — spec examples call APIs that do not exist | SPEC | The APIs examples use are specified in Part XV. | `[STD-15]`, `[STD-19]` |
| F-078 — the text contradicts itself on whether a `Sync` class has dynamic exclusivity at all, which decides whether Safe Ember has a data race | SPEC | Only `@sync` classes are shareable, and their fields are immutable after `init`; no data race. | `[THR-1]`, `[THR-13]` |
| F-079 — assigning a non-`Copy` field through a handle is neither an "instantaneous" nor a "long-term" access | SPEC | Assigning a non-`Copy` field through a handle is a write access. | `[EXC-16]` |
| F-080 — `String(literal)` does not exist | SPEC | `String.from(s)`, `s.to_string()` and literal coercion are specified. | `[TXT-9]`, `[TXT-11]` |
| F-081 — writing a field through a borrowed class-handle parameter is rejected (update to F-073) | SPEC | Borrowed class-handle parameters may be written through. | `[CLS-7]`, `[FN-9]` |
| F-082 — reference cycles leak silently in release, and Python users build cycles | SPEC | Debug runs report leaked cycles at exit by default. | `[WK-15]` |
| F-083 — objects may die before their last syntactic use | SPEC | Early deinitialisation is specified with `with h:` and lint `L3019`. | `[RC-3]` |
| F-084 — the VIII.7 idiom uses three things the compiler lacks or rejects | SPEC | Callable fields, pattern conditions and owned callable values make the idiom writable. | `[CLO-3]`, `[CLO-11]` |
| F-085 — the development target's examples are not checked by any gate, and half of them do not parse | GATE | Every `ember` block of the current document is extracted and checked. | `[TST-7]` |
| F-086 — `[ARN-4]` makes `@noalloc`-cleanliness a property of a value, which the effect system cannot see | SPEC | `FixedArena` is the `@noalloc` arena type; `Arena.fixed` returns it. | `[ARN-4]` |
| F-087 — `Shared[T]` overlaps `class` almost completely | SPEC | Kept, with the selection table saying when each applies. | `[SEL-1]` |
| F-088 — almost none of the Part IX.1 heap library exists | IMPL | The heap library is specified; building it is implementation work. | `[HEAP-1]`, `[STD-15]`, `[STD-16]` |
| F-089 — `[UNS-8]` and `[LEX-11]` contradict each other about whether a comment can produce a warning | SPEC | Safety notes are the one stated exception to "comments never affect compilation". | `[LEX-11]`, `[LEX-23]`, `[UNS-8]` |
| F-090 — the `assert_disjoint` example uses `dst` after moving it | SPEC | `assert_disjoint` returns a `Result` that gives the views back on failure. | `[DSJ-1]` |
| F-091 — `[ALC-3]` declares `static ALLOC: dyn Allocator` | SPEC | The global allocator is named in the manifest; no unsized static. | `[ALC-3]` |
| F-092 — phantom type parameters | SPEC | Phantom type parameters are legal. | `[TYP-35]` |
| F-093 — `Map`/`Set` iterate in unspecified order; Python's `dict` iterates in insertion order | SPEC | `Map`/`Set` iterate in insertion order and are not `Nondet`. | `[STD-11]`, `[DET-2]` |
| F-094 — is `DefaultHasher` seeded per process? | SPEC | Fixed-seed `DefaultHasher`; `RandomState` for untrusted keys. | `[HASH-2]` |
| F-095 — `[EFF-2]`: a `fn(A) -> R` parameter "is assumed to carry all effects unless written `@noalloc fn(A) -> R`" | SPEC | Callable parameters contribute the passed function's effects, per instantiation. | `[EFF-2]` |
| F-096 — `RuntimeCheck` has four kinds; `[EFF-17]` says `@no_runtime_checks` "excludes all five kinds" | SPEC | Four `RuntimeCheck` kinds, named. | `[EFF-9]` |
| F-097 — `[EFF-16]`'s last two sentences have no clear antecedent | SPEC | What `@nopanic(explicit)` allows is stated plainly. | `[EFF-16]` |
| F-098 — `@realtime`'s default set includes `@nopanic(explicit)`, and `[EFF-16]` makes every division by a non-constant, every shift by a variable, a | SPEC | `RuntimeCheck` is allowed under `@nopanic(explicit)` and `@realtime`; `NonZero` removes division checks. | `[EFF-16]`, `[EFF-19]`, `[STD-4]` |
| F-099 — the attribute table omits `@nopanic(explicit)` (refines F-021) | SPEC | The attribute table lists `@nopanic(explicit)`. | `[ATT-1]` |
| F-100 — the first-hour path does not exist | GATE | Single-file programs need no manifest; the user guide and first-week corpus are release artefacts. | `[CLI-4]`, `[DOC-2]`, `[TST-8]` |
| F-101 — is the stale-handle check removed in `shipping`? | SPEC | Stale-handle checks run in every profile. | `[HND-1]`, `[COST-3]` |
| F-102 — "release wraps and emits nothing" is only true if the C is written so that wrapping is defined | SPEC | Overflow panics in every profile; the emitted C is UB-free. | `[TYP-8]`, `[CG-C-1]` |
| F-103 — `[TOOL-1]`–`[TOOL-4]` sit inside X.3 "Inspection" | SPEC | Toolchain rules live in Part XVII. | `[CLI-4]` |
| F-104 — `[THR-1]` admits a `Sync` class with a plain mutable scalar field, and scalar field writes through a handle are unchecked | SPEC | Plain mutable fields cannot exist in a `@sync` class. | `[THR-1]` |
| F-105 — the job-system example breaks two rules | SPEC | The job example uses a scope, borrows legally, and writes no call-site modes. | `[JOB-2]`, `[FN-2a]` |
| F-106 — what `return`, `break`, `continue` and `?` mean inside `with scope = thread.scope():` | SPEC | A scope is a `with` block; jumps keep their meaning and join first. | `[THR-5]` |
| F-107 — async is v2 | SPEC | `async` stays reserved; generators are designed not to block it. | `[LEX-15]` |
| F-108 — `[CT-1]` excludes allocation from comptime and then supports allocating types | SPEC | Compile-time code may allocate. | `[CT-1]` |
| F-109 — what happens when comptime-materialised heap data is mutated at run time | SPEC | Heap results are read-only static data, run-time initialised, or cloned per evaluation. | `[CT-5]` |
| F-110 — `comptime` is used as an expression with a block value, which the grammar does not have | SPEC | `comptime(e)` is the expression form. | `[CT-6]`, `[GRM-32]` |
| F-111 — more syntax in examples that the grammar lacks | SPEC | Examples use grammar that exists: proxies are `SoARef`, `@from` is on the variant, no call-site `mut`. | `[SOA-6]`, `[ERR-3]`, `[FN-2a]` |
| F-112 — `@derive(SoA)` generates a different `SoA[T]` per `T`, which is specialisation | SPEC | `SoA[T]` is a compiler-known type constructor. | `[SOA-1]` |
| F-113 — a default error type would shorten most signatures | SPEC | `Result[T, E = AnyError]`. | `[ERR-9]`, `[ERR-8]` |
| F-114 — `[DRV-1]`: every derive is specified as a hand-written `extend` in `std/derive/*.em`, and the generator "MUST produce the same MIR" | SPEC | The spec text itself defines what each derive generates. | `[DRV-1]` |
| F-115 — the build order is inverted against the goal | SPEC | Conformance profiles put the core language and C before C++; C++ is an optional annex. | `[CONF-2]`, `[CONF-4]` |
| F-116 — `min`/`max`/`clamp`/`abs`/`sqrt`/`sin` are declared in `std.math`, but the spec's examples call them unimported | SPEC | Prelude has `min`/`max`/`abs`/`clamp`; `sqrt` is a method; other math is imported. | `[MOD-5]`, `[STD-20]` |
| F-117 — `[STD-8]`'s mandated help text is Rust syntax | SPEC | The mandated help is Ember syntax. | `[STD-8]` |
| F-118 — `[TXT-2]` still names `CppString.as_str()` | SPEC | Conversions from foreign text are fallible `to_str()`. | `[TXT-2]` |
| F-119 — `print`/`println` have no signature | SPEC | `print`/`println` signature, `sep=`, `end=`, several arguments. | `[STD-9]`, `[TYP-26]` |
| F-120 — `unsafe(reason = "…")` (`[UNS-9]`) is not in the grammar | SPEC | No `unsafe(reason=…)`; the category goes in the `# SAFETY(…):` note. | `[LEX-23]`, `[UNS-8]` |
| F-121 — sections filed under the wrong Part | SPEC | Each rule sits in its Part. | `[STD-4]` |
| F-122 — the `[MOD-5]` prelude list omits names Part XV puts in the prelude | SPEC | The prelude table includes them. | `[MOD-5]` |
| F-123 — Ember cannot call C at all today, and the C-import directive is silently ignored | SPEC | Manual declarations are module items; a function declared `safe fn` with a complete contract is safe to call (an asserted fact). | `[FFI-10]` |
| F-124 — two more unapplied editorial instructions inside normative rules | SPEC | Rewritten without editorial instructions; grades move to Annex C. | `[TCB-1]` |
| F-125 — `std::string` → `CppString` "with `.as_str()`" | SPEC | `CppString.to_str()` everywhere. | `[FFI-17a]` |
| F-126 — the overlay language has no grammar | SPEC | The overlay language has a grammar. | `[GRM-35]` |
| F-127 — scope: the C++ importer is a project the size of the rest of the compiler | SPEC | C++ is an optional annex after C. | `[CONF-4]`, `[FFI-3]` |
| F-128 — the "five count contracts" are never listed | SPEC | The count axis is listed: `one`, `count(n)`, `nul_terminated`, `fixed(N)`, `inout_count(p)`. | `[FFI-11]`, `[FFI-11a]` |
| F-129 — the `[FFI-39]` example uses constructor syntax the grammar does not have | SPEC | Constructors are `fn init(self, …)` everywhere, including C++ subclasses. | `[FFI-39]`, `[CLS-2]` |
| F-130 — two package settings turn memory-safety checks off for Safe code, and `[PHIL-10]` lists no such exception | SPEC | No setting removes a safety check; the two keys are gone. | `[PRF-1]`, `[EXC-14]`, `[GPU-1]` |
| F-131 — XVII.5 writes `mut` at a call site | SPEC | No call-site modes in examples. | `[FN-2a]` |
| F-132 — the mandated refusal report suggests a signature the rules reject | SPEC | Refusal reports do not suggest signatures; `migrate_from` returns `Result`. | `[HR-16]`, `[HR-18]` |
| F-133 — `migrate_from` may not contain a single integer `+` | SPEC | Run-time check failures in `migrate_from` become `ReloadError.Migration`. | `[HR-35]` |
| F-134 — is a base class's `init` inherited? | SPEC | A derived class with no `init` inherits its base's. | `[CLS-10]` |
| F-135 — wrong or loose citations | SPEC | Citations rewritten; one attribute per line. | `[HR-3]`, `[ATT-4]` |
| F-136 — the mangling scheme is not injective, and a collision is an internal compiler error | SPEC | Mangling is length-prefixed and injective; a hash collision is `E9040`. | `[MNG-1]` |
| F-137 — `for x in span:` is rejected | IMPL | `for x in span:` iterates a `Span`. | `[CTL-1]` |
| F-138 — `[CTL-3b]`'s guaranteed lowering is not implemented | IMPL | The guaranteed counted-loop lowering stands. | `[CTL-3]` |
| F-139 — float contraction is never disabled | SPEC | Floating-point flags and pragmas are required of the backend. | `[CG-C-11]`, `[TYP-9]` |
| F-140 — there is no performance suite; "fast like C" is unmeasured | GATE | A performance suite against equivalent C gates releases. | `[TST-28]` |
| F-141 — several promised backend/runtime pieces do not exist | SPEC | Inline header, allocator and backtrace requirements restated; mimalloc no longer required. | `[CG-C-3]`, `[RT-1]`, `[CG-C-10]` |
| F-142 — `compiler/ember_typeck/src/lib.rs` is 20,935 lines in one file | OUT | Source-file size in the compiler is not language text; noted for the compiler. | — |
| F-143 — XIX says effect analysis both runs after monomorphisation (XIX.1: *"Effect analysis runs  | SPEC | Effects are computed once, per instantiation for callable parameters. | `[EFF-2]`, `[EFF-15]` |
| F-144 — the instrument that measures the goal does not exist | GATE | First-week corpus and published acceptance rate. | `[TST-8]`, `[TST-10]` |
| F-145 — `E9010` has two meanings | SPEC | `E9010` is an unknown manifest key; an unhonourable float attribute is `E9041`. | `[MAN-1]`, `[TYP-9c]` |
| F-146 — most of the CLI in XX.1 is missing | SPEC | The CLI is listed; anything unimplemented is `E0900` and shown by `--matrix`. | `[CLI-19]` |
| F-147 — the example manifest makes shipping builds UB on an exclusivity violation by default | SPEC | The manifest example has no safety keys; none exist. | `[MAN-8]` |
| F-148 — `xs[-1]` compiles and panics at run time with index 18446744073709551615 | SPEC | Indices are `int`; a negative literal index is `E2011`; a negative index panics with a clear message. | `[TYP-31]`, `[LEX-24]` |
| F-149 — `x is None` gives two wrong errors instead of the fix | SPEC | `x is None` is legal. | `[EXP-9]` |
| F-150 — `let x = 5` gives a raw parse error | SPEC | `let x = 5` gets the `x = 5` fix-it. | `[DIA-21]` |
| F-151 — `a < b < c` fix-it is incomplete | SPEC | Chained comparisons are legal, so no fix-it is needed. | `[GRM-25]` |
| F-152 — `[DIA-7a]`'s table is flattened into one line | SPEC | Diagnostic tables are real tables. | `[DIA-7]` |
| F-153 — the N1 suggestion threshold admits `io` -> `Eq` (the rule behind F-006) | SPEC | Suggestion distance: 1 up to 4 characters, 2 above. | `[DIA-24]` |
| F-154 — stale conditional and dead codes in the diagnostics part | SPEC | Shape table rewritten; N8 no longer asks for a call-site mode. | `[DIA-7]`, `[FN-2a]` |
| F-155 — integer `/` is the silent Python trap | SPEC | Integer `/` is rejected with fix-its; `//` is floor division. | `[TYP-28]` |
| F-156 — no user guide and no "coming from Python" chapter | GATE | User guide with a Python chapter ships each release; Appendix E is its basis. | `[DOC-2]` |
| F-157 — C++ exception policy: "no default" vs a default | SPEC | One C++ exception policy, with a default. | `[FFI-24]` |
| F-158 — more amendment text appended instead of applied | SPEC | Rewritten without appended amendments. | `[TCB-1]` |
| F-159 — Phase 1 is recorded as complete, but its exit criterion ("conformance for Parts II–VI except closures/generics") is not met | OUT | Phase status is project tracking; `ember --version --matrix` makes implementation status visible. | `[CLI-19]` |
| F-160 — indexing does not produce a place that can be returned by reference, so milestone M2 cannot be written as specified | IMPL | Indexing yields a place; a `ref` result auto-borrows it. | `[TYP-5]`, `[GRM-36]` |
| F-161 — every contract attribute is accepted and ignored, and so is any made-up attribute | SPEC | Unbuilt contracts and unknown attributes are rejected, never ignored. | `[PHIL-12]`, `[EFF-23]` |
| F-162 — milestone M3 fails: `mem` is not in the prelude | SPEC | `mem` is in the prelude. | `[MOD-5]` |
| F-163 — XXI.5 lists `book/` as "(user guide, v1.1)" | SPEC | The user guide is a release artefact. | `[DOC-2]` |
| F-164 — the Stage 0 example uses syntax and semantics the rest of the document rejects | OUT | The engine-specific Stage 0 example is outside the language document. | — |
| F-165 — stale lists in XXIII | SPEC | New glossary and reserved-word statement. | `[LEX-15]` |
| F-166 — three owner decisions bear directly on the goal and should be re-put with the evidence from this pass | SPEC | Decided in this revision: 64-bit number defaults, branch-declared names, no `::`. | `[TYP-1]`, `[CTL-10]`, `[GRM-24]` |
| F-167 — the quick reference does not type-check, and one of its errors is in the reference itself | GATE | Annex A is generated from a checked fixture. | `[TST-6]` |
| F-168 — `with_views*` and `@latebound`: a chain of features whose net effect is to reject safe programs | SPEC | `with_views*` and `@latebound` are removed; callable types get fresh regions per call. | `[LT-7]` |
| F-169 — diagnostics that originate in a callee's contract are reported at the callee, not the caller | SPEC | Diagnostics from a callee's contract point at the call site. | `[DIA-22]` |
| F-170 — the headline multi-region example does not compile under the document's own rules | SPEC | The multi-region example takes `mut b`. | `[FN-2a]` |
| F-171 — the machine-readable implementation matrix the spec requires does not exist | GATE | The implementation matrix is a compiler output. | `[CLI-19]` |
| F-172 — Part XXIV's normative rules have no category | OUT | Part XXIV (compiler internals) is not in 0.9.9; its user-visible obligations are Part XVIII. | `[IMP-11]` |
| F-173 — one 870 KB file is three documents | OUT | 0.9.9 is the language document only; compiler design and history are elsewhere. | — |
| F-174 — `[GEN-COH-1]` makes impl ownership a *package* matter | SPEC | One coherence rule, at package granularity. | `[TYP-20]` |
| F-175 — what happens when a `Pool` runs out of generations | SPEC | 64-bit handles by default; exhausted slots retire; full pool is `CapacityError`. | `[HND-2]`, `[HND-3]` |
| F-176 — `[HR-IMPL-2]` describes reclaiming an old image | SPEC | Old images are never unloaded; statics live outside images. | `[HR-4]`, `[HR-4a]` |
| F-177 — Appendix B is a "mandatory CI" checklist that fails against the file it is in | OUT | No self-checking checklist in the document; the end checks are tools. | `[TST-4]` |
| F-178 — Appendix H is 730 lines of audit records that "bind nobody" | OUT | Appendix H is a change list, not audit records. | — |
| F-179 — 1.0 is tied to a full RageV production migration | OUT | 1.0 is defined by conformance and gates, not by a host migration. | `[CONF-1]` |
| F-180 — the conditional expression `a if c else b` is not implemented | IMPL | Conditional expressions are in the grammar. | `[GRM-11]` |
| F-181 — built-in scalars do not implement `Ord`, so no generic comparison can be written | SPEC | Scalars implement `Ord`. | `[TYP-36]` |
| F-182 — `Option`/`Array`/`str` basics are missing | IMPL | The combinator and container APIs are specified. | `[ERR-4]`, `[STD-15]`, `[TXT-10]` |
| F-183 — a single-statement block lambda inside brackets is rejected | IMPL | A one-statement lambda body in brackets is legal. | `[GRM-17]` |
| F-184 — range types reach the C compiler broken | IMPL | Range types print and compile; the C gate catches broken C. | `[CG-C-2]`, `[TST-27]` |
| F-185 — a parent/child class pair with a constructor overflows the compiler's stack | IMPL | An internal compiler error is always a defect. | `[CG-C-2]` |
| F-186 — the runtime exclusivity panic names a C symbol, not the Ember object | SPEC | Panics name Ember entities. | `[DIA-23]`, `[EXC-6]` |
| F-187 — `L3011` fires on calls that cannot re-enter the cell | SPEC | `L3011` fires only for calls that can reach the cell. | `[CELL-7]` |
| F-188 — a view escaping its source gets two errors, the first wrong | SPEC | Storage end with a live loan is one diagnostic, shape B7. | `[BCK-4]`, `[DIA-7]` |
| F-189 — `Array.iter()` does not exist | IMPL | `Array.iter()` is specified. | `[STD-15]` |
| F-190 — growable-buffer arithmetic is unchecked | SPEC | Capacity arithmetic is checked. | `[HEAP-8]` |
| F-191 — `Array[T]` storage is aligned to 16 bytes whatever `T` needs | SPEC | Heap storage honours element alignment. | `[HEAP-9]` |
| F-192 — `@align(N)` is silently ignored | SPEC | Layout attributes are honoured or rejected. | `[LAY-2]`, `[PHIL-12]` |
| F-193 — every reference-count operation is an out-of-line call, and a `Sync` retain is a CAS loop | SPEC | Count fast paths are inline; a `@sync` retain is one `fetch_add`. | `[RT-10]` |
| F-194 — every allocation goes through `_aligned_malloc` (MSVC) / `aligned_alloc`, and `ember_realloc` always allocates-copies-frees | SPEC | System allocator for ordinary alignment; per-thread statistics. | `[RT-1]`, `[RT-11]` |
| F-195 — reference-count overflow reports the wrong reason | SPEC | Count overflow panics with its own message. | `[RT-7]` |
| F-196 — redeclaring a name in the same block is accepted | IMPL | Redeclaring in one block is `E1020`. | `[GRM-4]` |
| F-197 — raw strings, byte strings and f-string format specs are not implemented | IMPL | Raw strings and format specs are specified. | `[LEX-19]`, `[LEX-25]` |
| F-198 — the ten most common Python habits get no help at all | SPEC | A table of Python habits with fix-its. | `[DIA-21]`, `[STD-13]` |
| F-199 — the ledgers say "none open" while ~80 defects reproduce | OUT | Defect ledgers are project process; Appendix G lists every open finding with its resolution. | — |
| F-200 — specification churn far outpaces the compiler | OUT | Process: 0.9.9 consolidates to one revision and a smaller document. | — |
| F-201 — `docs/HANDOFF.md` is 10,698 lines | OUT | Repository handoff size is not language text. | — |
| F-202 — `examples/` has only `hello.em` | GATE | Documentation samples are compiled and run. | `[DOC-2]` |
| F-203 — a generic struct's methods cannot name their own type | SPEC | `Self` and the type's own name work inside its declaration. | `[STR-7]` |
| F-204 — the textbook recursive enum cannot be traversed | SPEC | `Box[T]` reads through like a reference. | `[TYP-14]` |
| F-205 — calling a parameter whose type is a generic bounded by `Callable` fails | SPEC | Callable bounds are written `F: fn(A) -> R`; `Callable[…]` is not source. | `[CLO-14]` |
| F-206 — 32 error pages for 210 registered codes | GATE | Every code has an error page; a missing page fails CI. | `[DIA-6]` |
| F-207 — the six "green" gates are green because their baselines absorb almost everything | GATE | Baselines are reported and a heavily baselined gate is not called green. | `[TST-29]` |
| F-208 — range slicing `xs[a..b]` is not implemented | IMPL | Slicing is specified; its failure must not cascade. | `[DIA-14]` |
| F-209 — comprehensions | SPEC | List, set and map comprehensions. | `[GRM-27]` |
| F-210 — interfaces are nominal only | SPEC | Interfaces stay nominal, with the reason stated. | `[TYP-40]` |
| F-211 — `enumerate` over an `Array` cannot be written | IMPL | `enumerate` over an array is specified. | `[STD-19]` |
| F-212 — two errata are marked open that the grammar already fixed | OUT | The errata ledger is outside the document. | — |
| F-213 — `i32.MIN % -1` | SPEC | `MIN % -1 == 0`; `MIN // -1` panics as overflow, naming the right operator. | `[TYP-28]` |
| F-214 — the test suite is green (263 tests) but has a flaky UI pair, and `cargo test` hides everything after the first failure | GATE | The runner reports all failures and quarantines flaky tests by name. | `[TST-30]` |
---

# Appendix H — Changes from 0.9.8_Hardened_3

## H.1 What changed, by goal

**Memory safety.**
* Only `@sync` classes are shareable, and their fields are immutable after `init`; Safe Ember has no
  data race (`[THR-1]`, `[THR-13]`).
* Assigning a non-`Copy` field through a class handle is a checked write access (`[EXC-16]`).
* No setting removes a safety check: `exclusivity = "unchecked"` and `gpu.validate` are gone, overflow
  and stale-handle checks run in every profile (`[PRF-1]`, `[TYP-8]`, `[HND-1]`, `[GPU-1]`).
* Checked capacity arithmetic and honoured alignment in the runtime (`[HEAP-8]`, `[HEAP-9]`).
* Compile-time heap data can never be grown or freed at run time (`[CT-5]`).

**C-like speed.**
* UB-free C, floating-point flags, injective mangling, inline count fast paths (`[CG-C-1]`, `[CG-C-11]`,
  `[MNG-1]`, `[RT-10]`).
* Grouped overflow checks keep checked arithmetic vectorisable (`[SIMD-7]`).
* A performance gate against equivalent C (`[TST-28]`).

**Python ergonomics.**
* `int` is `i64`, `float` is `f64`; integer `/` is rejected, `//` and `%` are floor (`[TYP-1]`,
  `[TYP-28]`).
* String literals initialise `String`; collection literals and comprehensions; chained comparisons;
  `x is None`; unparenthesised tuples; names assigned in every branch; implicit `Eq`/`Debug`/`Clone`;
  lazy statics; generators and `some` returns; `Result[T]` with `AnyError`; ordered `Map`; `print`
  with several arguments; callable fields; Python-habit fix-its (`[TXT-9]`, `[TYP-38]`, `[GRM-25]`,
  `[EXP-9]`, `[GRM-29]`, `[CTL-10]`, `[STR-5]`, `[STA-3]`, `[CORO-1]`, `[TYP-32]`, `[ERR-9]`,
  `[STD-11]`, `[STD-9]`, `[CLO-11]`, `[DIA-21]`).
* Rust-style friction removed: no `::`, no lifetime syntax ever, `@view` inferred, coherence per
  package, `with_views*` and `@latebound` removed, field-sensitive private methods, arena elision,
  location-sensitive borrow checking (`[GRM-24]`, `[LEX-22]`, `[TYP-34]`, `[TYP-20]`, `[LT-7]`,
  `[BRW-10]`, `[LT-44]`, `[BCK-2]`).

**Coherence.**
* No silent acceptance; one meaning per program; Python spelling means Python meaning; costs are named
  (`[PHIL-12]`–`[PHIL-15]`).
* One language before 1.0 (`[VER-8]`). The document is the language only: compiler architecture, the
  implementation plan and the host-engine plan are no longer in it; C++, hot reload and GPU are
  annexes.

## H.2 Removed constructs

`::`; `'a` lifetime reservation; `with_views`, `with_views2..4`, `@latebound`; `@thread_local`;
`@derive(SoA)` and `columns_mut`; `Callable[…]` in source; `PartialEq`/`PartialOrd`; `unsafe(reason =
…)`; `#! language` selectors other than the current version; the manifest keys `exclusivity`,
`overflow`, `bounds_checks` and `gpu.validate`; job access-set declarations verified only in debug.

## H.3 Rule identifiers no longer defined

Generated: every 0.9.8 identifier that this revision does not define, grouped by family. An identifier
is never reused for another meaning.

| Family | Identifiers | Why |
|---|---|---|
| ARN | ARN-5a, ARN-5b, ARN-5e, ARN-5f, ARN-8a, ARN-9, ARN-12, ARN-13 | sub-rules folded into `[ARN-5]` and `[ARN-8]` |
| AST | AST-1, AST-2 | compiler internals; out of the language document |
| ATT | ATT-5 | folded into the attribute table and `[GRM-20]` |
| BEN | BEN-7 | benchmark plan; replaced by the performance gate `[TST-28]` |
| BLD | BLD-3, BLD-12 | build-system internals; the user-visible parts are §XVII.3 |
| BLD-FFI | BLD-FFI-4a | C++ build integration; see Annex C |
| BUD | BUD-1a, BUD-2a, BUD-4, BUD-4a, BUD-5a, BUD-6, BUD-7, BUD-8 | compile-time budget details; condensed into `[BLD-10]` |
| CAT | CAT-1, CAT-2, CAT-3, CAT-4, CAT-5 | rule categories retired; conformance profiles (`[CONF-*]`) replace them |
| CELL | CELL-6a, CELL-11 | folded into `[CELL-6]` and the prelude (`[MOD-5]`) |
| CLI | CLI-3, CLI-14, CLI-16, CLI-17 | condensed into the command listing of §XVII.1 and `[CLI-19]` |
| CLO-ABI | CLO-ABI-1, CLO-ABI-2 | compiler internals |
| CMP | CMP-1, CMP-2, CMP-3 | compiler internals |
| COMP | COMP-1 | compiler internals |
| CONF | CONF-6 | folded into `[CONF-1]` |
| COR-ABI | COR-ABI-1, COR-ABI-2 | compiler internals |
| CTL | CTL-3a, CTL-3c | folded into `[CTL-3]`; its conformance obligations are `[TST-4a]` |
| CTR | CTR-11 | retired in 0.6.2 |
| CXX | CXX-2, CXX-3, CXX-4, CXX-5, CXX-6, CXX-7 | C++ corpus details; `[CXX-1]` in Annex C |
| DET-IMPL | DET-IMPL-1, DET-IMPL-2 | compiler internals |
| DIA | DIA-17, DIA-19 | condensed into §XVII.6 (shape tables, `[DIA-21]`, `[DIA-24]`) |
| EFF | EFF-11a, EFF-19a, EFF-19b, EFF-22 | folded into `[EFF-11]`, `[EFF-16]` and `[EFF-19]` |
| EXP | EXP-7 | replaced by floor semantics, `[TYP-28]` |
| FFI | FFI-11d, FFI-11e, FFI-18, FFI-19, FFI-20, FFI-24a, FFI-32a, FFI-32b, FFI-32c, FFI-32d, FFI-32e, FFI-34, FFI-34a, FFI-35, FFI-37e, FFI-37f, FFI-43a | C++ interop details (Annex C) or folded into the five-axis contract `[FFI-11]` |
| FFI-CB | FFI-CB-1 | folded into `[FFI-21]` |
| FFI-IMPL | FFI-IMPL-1, FFI-IMPL-3 | compiler internals |
| FN | FN-6a, FN-6b | removed with `@latebound` (F-168); callable types elide regions per call (`[LT-7]`) |
| GATE | GATE-1, GATE-2, GATE-3, GATE-4, GATE-5, GATE-6, GATE-7, GATE-8, GATE-8a | implementation plan; out of the language document |
| GEN-COH | GEN-COH-1, GEN-COH-2 | folded into `[TYP-20]` |
| GRM | GRM-9, GRM-14, GRM-22 | folded into Part III (`GRM-14` retired with the access-mode generic kind, F-024) |
| HIR | HIR-1, HIR-2 | compiler internals |
| HOT | HOT-1, HOT-3, HOT-5, HOT-10 | compiler internals of hot reload |
| HR | HR-12b, HR-21a, HR-22a, HR-23a, HR-26, HR-40 | condensed into Annex B |
| HR-IMPL | HR-IMPL-1, HR-IMPL-3 | compiler internals of hot reload |
| IDE | IDE-1, IDE-2, IDE-5, IDE-7, IDE-10 | language-server design; out of the language document |
| IMP | IMP-1, IMP-2, IMP-3, IMP-7, IMP-8, IMP-9, IMP-10 | implementation plan; out of the language document (`[IMP-11]` is new) |
| JOB | JOB-4 | debug-only access-set verification removed (a check in one profile is not a guarantee, `[PHIL-13]`) |
| LAY | LAY-1 | replaced by `[LAY-2]` |
| LEX | LEX-14a, LEX-15a, LEX-15b | folded into `[LEX-14]` and `[LEX-15]` |
| LT | LT-2, LT-2a, LT-4a, LT-4b, LT-8, LT-8a, LT-9, LT-10, LT-11, LT-11a, LT-12, LT-13, LT-15, LT-19, LT-28, LT-29, LT-31, LT-31a, LT-32, LT-33, LT-37, LT-40, LT-41 | `with_views*` removed (F-168); multi-region rules condensed into §VII.4 |
| MAN | MAN-4, MAN-5, MAN-6 | folded into `[MAN-8]` |
| MIR | MIR-1, MIR-2, MIR-3, MIR-4, MIR-5, MIR-6 | compiler internals |
| MIR-REG | MIR-REG-1 | compiler internals |
| MNG | MNG-5 | folded into `[MNG-1]` |
| MOD | MOD-6, MOD-6a | language-version selectors removed (`[VER-8]`) |
| MONO | MONO-4 | folded into `[MONO-2]` and `[MONO-8]` |
| OPT | OPT-2a | conformance obligation folded into `[TST-4a]` |
| OQ8 | OQ8-1, OQ8-4, OQ8-7 | open questions closed by this revision |
| PRV | PRV-8 | retired with the prover in 0.6.2 |
| RNG | RNG-5a, RNG-10c | folded into `[RNG-5]` and `[RNG-10]` |
| RT | RT-9 | folded into `[RT-1]` and `[HND-3]` |
| RV | RV-1, RV-2, RV-3, RV-4 | host-engine integration plan; out of the language document |
| SOA | SOA-5 | `columns_mut` retired: columns are disjoint places (`[SOA-2]`) |
| SPN | SPN-6, SPN-7, SPN-9, SPN-10 | folded into `[SPN-4]` and Part XV |
| STD | STD-7a | folded into `[STD-7]` |
| STD-IMPL | STD-IMPL-1, STD-IMPL-2 | compiler internals |
| TST | TST-4c, TST-12, TST-13, TST-14, TST-15, TST-16, TST-17, TST-18, TST-19, TST-20, TST-21, TST-22, TST-23, TST-24, TST-25, TST-26 | condensed into §XVII.5 |
| UNS | UNS-9, UNS-9a, UNS-10b | the reason category moved into the `# SAFETY(…):` note (`[LEX-23]`, `[UNS-8]`) |
| VER | VER-7 | folded into `[VER-2]` |
| VERIFY | VERIFY-1, VERIFY-2, VERIFY-3 | compiler internals |
| WK | WK-10 | folded into `[WK-4]`, `[WK-6]` and `[WK-15]` |

## H.4 Changes from 0.9.9_Hardened_1 to 0.9.9_Hardened_2

Hardened_2 applies the change list drawn from three passes over Hardened_1 (memory safety; consistency
and speed; ergonomics). Before 1.0 a hardening may change the language when this log lists each such
change (`[VER-9]`). These change which programs are accepted, or what they do:

| Change | Effect on existing programs | Rules |
|---|---|---|
| a thread or unscoped job carries only the static region | a spawn capturing a view of a local is rejected; use `thread.scope()` | `[THR-10]`, `[JOB-2]` |
| `Shared[T]` is one-thread; `SyncShared[T]` crosses threads; `get`/`get_mut` are checked accesses | sending a `Shared` is rejected; overlapping `get_mut` panics | `[HEAP-4]`, `[HEAP-5]`, `[HEAP-10]`, `[THR-8]`, `[THR-9]` |
| two-phase construction | a derived `init` that calls `super.init` before assigning its own fields is rejected (`E2102`) | `[CLS-2]`, `[CLS-4]`, `[CLS-10]`, `[CLS-11]` |
| retained and once C callbacks own static captures; callback captures are `Send` unless the contract names a thread | such callbacks capturing borrows or non-`Send` handles are rejected | `[FFI-21]`, `[FFI-22]` |
| foreign functions are safe to call only when asserted: `safe fn`, or an overlay entry | calls to unlisted header functions and to extern-block functions not marked `safe` need `unsafe` | `[FFI-1]`, `[FFI-2]`, `[FFI-10]` |
| `yield` while a `@must_drop` value is live | rejected (`E2231`) | `[CORO-13]` |
| `char32_t` is `u32` | foreign signatures using it change type | `[FFI-8]` |
| SIMD memory operations are bounds-checked | out-of-range `load`/`store`/`gather`/`scatter` panic | `[SIMD-9]` |
| calling a callable field is a write access; owned callable values are called from a mutable place | a callback that replaces itself while running panics | §VIII.3, `[CLO-2]` |
| accesses held across a call into C++ stay checked | a conflicting re-entrant override panics | `[FFI-39d]`, `[EXC-7]` |
| `run_parallel` checks systems given as values | a conflict panics before any system starts | `[ECS-4]` |
| phantom parameters count for `Send`/`Sync`; `comptime.read_file` stays inside the package | fewer types are `Send`; reads outside the package are `E6010` | `[TYP-35]`, `[CT-2]` |
| exclusivity is tracked per field | reading one field while writing another no longer panics | `[EXC-19]`, `[EXC-1]`, `[EXC-2]`, `[EXC-15]` |
| a `Map` iterates its keys | `for k, v in m:` becomes `for k, v in m.items():` (`ember fmt --migrate`) | `[CTL-1]`, `[STD-16]` |
| the entry file may hold top-level statements | scripts need no `main` | `[GRM-2]`, `[FN-8]` |
| `len`, `range`, `sum`, `sorted`, `enumerate`, `zip`, `reversed`, `any`, `all` in the prelude | accepted with Python's meaning; `len` of a string is `E2073` | `[STD-26]`, `[MOD-5]` |
| generator expressions | accepted | `[GRM-38]` |
| printing falls back from `Display` to `Debug` | more values print; none prints differently | `[STD-9]`, `[LEX-19]` |
| `T` converts to `Option[T]`; `None` then `T` infers `Option[T]` | accepted | `[TYP-5]`, `[TYP-23]` |
| a `Result[void, E]` function returns `Ok(())` at its end | accepted | `[FN-10]` |
| a lambda passed to a consumed callable parameter captures by move | accepted without `owned fn` | `[CLO-15]` |
| standard modules importable without `std.` | accepted; a package module of the same name wins (`W1003`) | `[MOD-3]` |
| iterator adapters on any `Iterable` | accepted | `[STD-19]` |
| float `Display` is the shortest round-trip text | printed floats may change | `[STD-20]` |
| `W2015` only for more digits than the type keeps | fewer warnings | `[LEX-17a]` |
| contract verdicts assume every listed proof applied; grouping required in vectorisable loops | verdicts agree between implementations | `[EFF-15]`, `[SIMD-7]`, `[SIMD-5]` |

Generated: every rule whose text differs between the two hardenings.

| Rule | H1 → H2 |
|---|---|
| `[BRW-3]` | changed |
| `[CG-C-4]` | changed |
| `[CLI-4]` | changed |
| `[CLI-20]` | changed |
| `[CLO-2]` | changed |
| `[CLO-4]` | changed |
| `[CLO-7]` | changed |
| `[CLO-15]` | added |
| `[CLS-2]` | changed |
| `[CLS-4]` | changed |
| `[CLS-7]` | changed |
| `[CLS-10]` | changed |
| `[CLS-11]` | added |
| `[CORO-12]` | changed |
| `[CORO-13]` | added |
| `[CT-2]` | changed |
| `[CTL-1]` | changed |
| `[CTL-3b]` | changed |
| `[ECS-3]` | changed |
| `[ECS-4]` | changed |
| `[EFF-15]` | changed |
| `[ERR-13]` | changed |
| `[EXC-1]` | changed |
| `[EXC-2]` | changed |
| `[EXC-6]` | changed |
| `[EXC-7]` | changed |
| `[EXC-15]` | changed |
| `[EXC-17]` | added |
| `[EXC-18]` | added |
| `[EXC-19]` | added |
| `[FFI-1]` | changed |
| `[FFI-2]` | changed |
| `[FFI-8]` | changed |
| `[FFI-10]` | changed |
| `[FFI-21]` | changed |
| `[FFI-22]` | changed |
| `[FFI-39d]` | changed |
| `[FN-8]` | changed |
| `[FN-10]` | added |
| `[GRM-2]` | changed |
| `[GRM-38]` | added |
| `[HASH-2]` | changed |
| `[HASH-3]` | changed |
| `[HEAP-4]` | changed |
| `[HEAP-5]` | changed |
| `[HEAP-7]` | changed |
| `[HEAP-10]` | added |
| `[HND-2]` | changed |
| `[JOB-2]` | changed |
| `[LEX-15]` | changed |
| `[LEX-17a]` | changed |
| `[LEX-19]` | changed |
| `[LNT-6]` | changed |
| `[MOD-3]` | changed |
| `[OBJ-1]` | changed |
| `[OWN-6]` | changed |
| `[PAR-3]` | changed |
| `[PHIL-13]` | changed |
| `[PRF-1]` | changed |
| `[RC-3]` | changed |
| `[RC-4]` | changed |
| `[RNG-4]` | changed |
| `[RT-10]` | changed |
| `[RT-12]` | added |
| `[SIMD-3]` | changed |
| `[SIMD-5]` | changed |
| `[SIMD-7]` | changed |
| `[SIMD-9]` | added |
| `[SOA-6]` | changed |
| `[STD-9]` | changed |
| `[STD-13]` | changed |
| `[STD-15]` | changed |
| `[STD-16]` | changed |
| `[STD-19]` | changed |
| `[STD-20]` | changed |
| `[STD-26]` | added |
| `[THR-6]` | changed |
| `[THR-8]` | changed |
| `[THR-9]` | changed |
| `[THR-10]` | changed |
| `[THR-13]` | changed |
| `[THR-16]` | added |
| `[TIER-1]` | changed |
| `[TST-6]` | changed |
| `[TYP-5]` | changed |
| `[TYP-23]` | changed |
| `[TYP-35]` | changed |
| `[TYP-38]` | changed |
| `[VER-9]` | changed |
| `[WK-11]` | changed |
| `[WK-12]` | changed |

## H.5 Changes from 0.9.9_Hardened_2

Each ambiguity found while implementing 0.9.9 is an owner decision request (`docs/OWNER-QUEUE.md`),
ruled under the owner's delegation and recorded here. Hardened_3 carries ODR-021 and ODR-022;
Hardened_4 adds ODR-023; Hardened_5 adds ODR-024; Hardened_6 adds ODR-025; Hardened_7 adds ODR-026;
Hardened_8 adds ODR-027; Hardened_9 adds ODR-028;
Hardened_10 adds ODR-029; Hardened_11 adds ODR-030; Hardened_12 adds ODR-031; Hardened_13 adds ODR-032 to ODR-036.

| ODR | Ruling | Rules |
|---|---|---|
| ODR-021 | float `//` and `%` are Python's: the exact floor modulo rounded once, and the floor quotient consistent with it | `[TYP-29]` |
| ODR-022 | an untyped literal is never left open: `total = 0` declares an `int` whatever the later uses | `[TYP-23]` |
| ODR-023 | a function returning a value that can reach the end of its body is `E2182` (Hardened_4) | `[FN-10]`, §XVII.9 |
| ODR-024 | a borrowed or `mut` parameter whose type is not `Copy` is a source parameter a returned view may borrow, and a borrowed parameter is passed by address except a view or a `Copy` value holding no `Cell` (Hardened_5) | `[LT-1]`, `[LT-1a]`, `[LT-1b]`, `[LT-7]`, `[LT-44]`, `[FN-1]`, `[FN-3]`, `[FN-6]`, `[BRW-8]`, `[CORO-6]`, §VIII.3, §XVII.6 B7, §XVII.9, Appendix F |
| ODR-025 | `[ERR-4]`'s methods that take a function are eager, move the payload in (`filter` borrows it) and take `once fn`; an unannotated lambda parameter takes `owned` from the expected callable type, never `mut` (Hardened_6) | `[ERR-4]`, `[CLO-7]`, `[TYP-23]` |
| ODR-026 | a type that declares `drop` is not implicitly `Clone`; `@derive(Clone)` or a written `clone` gives it one (Hardened_7) | `[STR-5]` |
| ODR-027 | a range is a value: the prelude's range types are structs with public bounds, `Copy` when the bound is, and a `for` over one counts over a copy of its bounds, leaving it unchanged; `a..` overflows at its type's maximum (Hardened_8) | `[CTL-3]`, `[STD-8]`, `[STD-26]` |
| ODR-028 | a method named like an inherited one replaces it: without `override` over a virtual method it is `E2111`, over a non-virtual one `E2110`; an `override` is itself virtual (Hardened_9) | `[CLS-4]` |
| ODR-029 | `parse[T]()` reads the whole text strictly (no white space; an optional sign and digits for integers, Rust's float grammar with `inf`/`nan`, `true`/`false`, one character), and `ParseError` is `Empty`, `Invalid` or `Overflow` (Hardened_10) | `[TXT-10]` |
| ODR-030 | `extend` is a contextual keyword, a keyword only at the start of an item before the type it extends, so `Array` can have its `extend` method (Hardened_11) | `[LEX-15]` |
| ODR-031 | `Array`'s `sort_by(cmp: fn(T, T) -> Ordering)` and `sort_by_key[K: Ord](f: fn(T) -> K)` are stable, and `f` is called once per element; `windows(n)` yields shared, overlapping views one step apart, none when `n > len`, and panics on `0`; `drain(r) -> Array[T]` takes any integer range and panics outside `0..=len` (Hardened_12) | `[STD-15]` |
| ODR-032 | `Map[K, V, H, A]` and `Set[T, H, A]`: the hasher before the allocator; the full method lists with signatures; `AsKey[K]: Hash` has `is_key` and `to_key`, and every `K: Eq + Hash` is `AsKey[K]`; owned iteration of a `Map` yields its keys (Hardened_13) | `[STD-11]`, `[STD-12]`, `[STD-16]`, `[ALC-1]`, `[CTL-1]` |
| ODR-033 | the `Hasher` methods, and users may implement it; `DefaultHasher`'s values are fixed for one version on every target; making a `RandomState` is `Nondet`; a panicking `hash` or `eq` aborts; `Map`'s constant time is amortised (Hardened_13) | `[HASH-1]`, `[HASH-2]`, `[HASH-3]`, `[DET-2]`, `[STD-11]` |
| ODR-034 | an empty `Map` prints `{}` and an empty `Set` `set()`; strings `Debug` as Python's `repr`; `Map`/`Set` `==` compares as sets; a repeated literal key keeps its first position and last value; a list literal is never a `Set`; `sorted(m)` sorts the keys (Hardened_13) | `[TYP-39]`, `[STD-16]`, `[TYP-38]` |
| ODR-035 | `union` is contextual: reserved only where an item would begin `union` and a name, so `Set` can have its `union` method (Hardened_13) | `[LEX-15]` |
| ODR-036 | a `Map`'s keys and values and a `Set`'s elements are not views (`E3063`); text in a `{…}` literal with no context is `String` (Hardened_13) | `[STD-11]`, `[TYP-38]` |

Generated: every rule whose text differs from Hardened_2.

| Rule | H2 → now |
|---|---|
| `[ALC-1]` | changed |
| `[BRW-8]` | changed |
| `[CLO-7]` | changed |
| `[CLS-4]` | changed |
| `[CORO-6]` | changed |
| `[CTL-1]` | changed |
| `[CTL-3]` | changed |
| `[DET-2]` | changed |
| `[ERR-4]` | changed |
| `[FN-1]` | changed |
| `[FN-3]` | changed |
| `[FN-6]` | changed |
| `[FN-10]` | changed |
| `[HASH-1]` | changed |
| `[HASH-2]` | changed |
| `[HASH-3]` | changed |
| `[LEX-15]` | changed |
| `[LT-1]` | changed |
| `[LT-1a]` | changed |
| `[LT-1b]` | changed |
| `[LT-7]` | changed |
| `[LT-44]` | changed |
| `[STD-11]` | changed |
| `[STD-12]` | changed |
| `[STD-15]` | changed |
| `[STD-16]` | changed |
| `[STR-5]` | changed |
| `[TXT-10]` | changed |
| `[TYP-23]` | changed |
| `[TYP-29]` | changed |
| `[TYP-38]` | changed |
| `[TYP-39]` | changed |
---

# Appendix I — Rule Index

Every rule of this document, by family, with the Part that defines it.

## ABI

| Rule | Part | Begins |
|---|---|---|
| `[ABI-1]` | XVII | The runtime ABI, the hot-reload protocol, the module protocol and the … |
| `[ABI-2]` | XVII | A protocol that cannot be checked at link time (hot-reload images, … |
| `[ABI-3]` | XVII | `ember_module_init` checks the runtime ABI version before anything … |
| `[ABI-4]` | XVII | A release notes every protocol whose version changed, and why. |
| `[ABI-5]` | XVII | The protocol versions are independent of the language version: two … |

## ALC

| Rule | Part | Begins |
|---|---|---|
| `[ALC-1]` | IX | `Array[T, A: Allocator = Global]`, `Map`, `Set` and `Box` accept an … |
| `[ALC-2]` | IX | Implementing `Allocator` is `unsafe`: the implementer promises valid, … |
| `[ALC-3]` | IX | A binary package may replace the global allocator with `[build] … |
| `[ALC-4]` | IX | Allocation failure in a standard container panics (`out of memory`). … |

## ARN

| Rule | Part | Begins |
|---|---|---|
| `[ARN-1]` | IX | `Arena` is a move-only struct. Its `alloc*` methods take `self` (a … |
| `[ARN-2]` | IX | Values in an arena are never dropped individually. Allocating a type … |
| `[ARN-3]` | IX | `alloc(v) -> ref mut T` moves `v` in. `alloc_array[T](n) -> … |
| `[ARN-4]` | IX | A growing `Arena` carries `Alloc` on its growth path. `FixedArena` … |
| `[ARN-5]` | IX | `ArenaArray[T]` and `ArenaMap[K, V]` are fixed-capacity containers … |
| `[ARN-5c]` | IX | `ArenaArray[T]` provides `len`, `capacity`, `is_empty`, `get(i) -> … |
| `[ARN-5d]` | IX | `ArenaMap[K, V]` (with `K: Eq + Hash`) provides `len`, `capacity`, … |
| `[ARN-5g]` | IX | No operation of either container touches the arena after … |
| `[ARN-6]` | IX | `arena.scope() -> ScopedArena` takes a mutable borrow of the arena … |
| `[ARN-7]` | IX | No operation lowers an arena's allocation pointer or reuses its bytes … |
| `[ARN-8]` | IX | `MaybeUninit[T]` has the size and alignment of `T` and does not claim … |
| `[ARN-10]` | IX | A panic inside `T.default()` during `alloc_array` aborts (`[PAN-1]`); … |
| `[ARN-11]` | IX | `Zeroable` is an `unsafe` marker: all-zero bytes are a valid `T`. The … |

## ATT

| Rule | Part | Begins |
|---|---|---|
| `[ATT-1]` | V | An attribute that is neither in the table below nor a visible … |
| `[ATT-2]` | V | A statement may carry only `@parallel`, `@unroll`, `@simd` and … |
| `[ATT-3]` | III | A statement attribute attaches to the next compound statement, never … |
| `[ATT-4]` | V | One attribute per line. |
| `[ATT-6]` | V | Every attribute in the table has exactly the effect its rule gives. … |

## BCK

| Rule | Part | Begins |
|---|---|---|
| `[BCK-1]` | XVIII | Loans. Each borrow expression at a point creates a loan of a place, … |
| `[BCK-2]` | XVIII | Live loans, location-sensitive. A loan is live at a point when some … |
| `[BCK-3]` | XVIII | Conflicts. At each point, an access to a place is checked against the … |
| `[BCK-4]` | XVIII | Storage end. A place whose storage ends (scope exit, `StorageDead`) … |
| `[BCK-5]` | XVIII | Two-phase borrows. The mutable loan made for a method's `mut self` … |
| `[BCK-6]` | XVIII | Required acceptances. Because liveness is location-sensitive, these … |
| `[BCK-7]` | XVIII | Class handles. An access through a class handle is not a loan of the … |

## BEN

| Rule | Part | Begins |
|---|---|---|
| `[BEN-1]` | XVII | Each benchmark runs at least 3 untimed and 30 timed repetitions on … |
| `[BEN-2]` | XVII | A gate fails only when the lower bound of the ratio's interval … |
| `[BEN-3]` | XVII | Each run also measures the C program against a second copy of itself; … |
| `[BEN-4]` | XVII | Each benchmark records retired instructions; a change beyond ± 0.5 % … |
| `[BEN-5]` | XVII | Both sides are built with the same optimisation level, LTO setting, … |
| `[BEN-6]` | XVII | Thresholds: scalar and tight loops ≤ 1.05× C; SoA and SIMD code ≤ … |
| `[BEN-8]` | Annex B | The end-to-end reload benchmark times three edits — a function body, … |

## BLD

| Rule | Part | Begins |
|---|---|---|
| `[BLD-1]` | XVII | The module (one file) is the unit of front-end caching; the package … |
| `[BLD-2]` | XVII | A module's front-end result is keyed by its source, the compiler and … |
| `[BLD-4]` | XVII | The C compiler and linker run through a generated Ninja file, so C … |
| `[BLD-5]` | XVII | Output goes to `target/<profile>/{bin,lib,c,obj,bind,inspect}`. |
| `[BLD-6]` | XVII | `lto = "off" \| "thin" \| "on"`. A value the C toolchain does not … |
| `[BLD-7]` | XVII | Front-end stages run in parallel across modules by default (`-j`, … |
| `[BLD-8]` | XVII | Within a module, editing one function body re-checks that function … |
| `[BLD-9]` | XVII | `--timings` writes a per-stage, per-module timing report … |
| `[BLD-10]` | XVII | The compile-time budgets below are release gates. |
| `[BLD-11]` | XVII | A package that omits a `[STD-6]` layer cannot use it or depend on a … |
| `[BLD-13]` | XVII | Builds are reproducible: the same inputs give a byte-identical … |

## BLD-FFI

| Rule | Part | Begins |
|---|---|---|
| `[BLD-FFI-1]` | XVI | The manifest's `[c]` section names the C compiler and flags used for … |
| `[BLD-FFI-1a]` | XVI | Each `import c` is parsed and its shim compiled with the package's C … |
| `[BLD-FFI-1b]` | Annex C | The MSVC runtime-library switch, `_DEBUG` and `_ITERATOR_DEBUG_LEVEL` … |
| `[BLD-FFI-2]` | XVI | The toolchain ships a CMake module: `ember_add_library(name KIND … |
| `[BLD-FFI-3]` | XVI | Existing libraries are linked by name (`link = ["vulkan-1"]`) or … |
| `[BLD-FFI-4]` | Annex C | Ember's emitted C and the project's C++ are compiled by the same … |
| `[BLD-FFI-5]` | Annex C | `ember build --emit header` with `--cpp` also writes a C++ header … |
| `[BLD-FFI-5a]` | Annex C | The C++ header gives a copyable smart handle only to `@sync` classes, … |

## BRW

| Rule | Part | Begins |
|---|---|---|
| `[BRW-1]` | VII | Aliasing xor mutation. At every program point a place has any number … |
| `[BRW-2]` | VII | Liveness. A borrow is live from its creation to the last use of … |
| `[BRW-3]` | VII | Two-phase borrows. For a method call whose receiver is a place, or an … |
| `[BRW-4]` | VII | Disjoint fields. `ref mut a.x` and `ref mut a.y` may be live together … |
| `[BRW-5]` | VII | Indices are not disjoint. `ref mut a[i]` and `ref mut a[j]` conflict … |
| `[BRW-6]` | VII | Reborrows. From `r: ref mut T`, `ref r.f` freezes `r` while it lives … |
| `[BRW-7]` | VII | Borrowing a moved or uninitialised place is `E3050`. |
| `[BRW-8]` | VII | A borrowed parameter is passed by address: the callee reads the … |
| `[BRW-9]` | VII | A reference in Safe code is never null, dangling or unaligned. |
| `[BRW-10]` | VII | Methods borrow the fields they use. A call to a method of a struct or … |
| `[BRW-11]` | VII | All borrows a call makes are live together. The borrows formed for … |

## BUD

| Rule | Part | Begins |
|---|---|---|
| `[BUD-1]` | XVII | They are measured on a recorded reference machine (an 8-core laptop … |
| `[BUD-2]` | XVII | Each figure is the median of 15 runs after 3 warm-up runs: |
| `[BUD-3]` | XVII | A figure above its budget fails CI, and so does one more than 15 % … |
| `[BUD-3a]` | XVII | CI normalises its measurements by a fixed calibration workload, and a … |
| `[BUD-3b]` | XVII | A gate applies only to a figure whose measured spread is below the … |
| `[BUD-5]` | XVII | A proposed language or compiler feature states its measured effect on … |

## CELL

| Rule | Part | Begins |
|---|---|---|
| `[CELL-1]` | IX | `Cell[T]` holds a `T` that can be replaced through a shared borrow: … |
| `[CELL-2]` | IX | `Cell` never hands out a reference to its contents, so it needs no … |
| `[CELL-3]` | IX | `Cell[T]` is not `Sync`; it is `Send` if `T` is. |
| `[CELL-4]` | IX | `Cell[T]` is `Copy` when `T` is. |
| `[CELL-5]` | IX | `RefCell[T]` keeps a one-word borrow counter beside `T`. `borrow() -> … |
| `[CELL-6]` | IX | `try_borrow()` and `try_borrow_mut()` return `None` on conflict, in … |
| `[CELL-7]` | IX | `Ref[T]` and `RefMut[T]` are views of the cell; dropping one ends its … |
| `[CELL-8]` | IX | `RefCell[T]` is not `Sync`; `Mutex[T]` and `RwLock[T]` are its … |
| `[CELL-9]` | IX | The `RefCell` check exists in every profile. |
| `[CELL-10]` | IX | A borrow diagnostic suggests `RefCell` only after the structural … |
| `[CELL-12]` | IX | `RefCell[T]` is never `Copy`. |

## CG-C

| Rule | Part | Begins |
|---|---|---|
| `[CG-C-1]` | XVIII | The emitted C has no undefined behaviour. Checked signed arithmetic … |
| `[CG-C-2]` | XVIII | Accepted programs compile. A program Ember accepts never produces C … |
| `[CG-C-3]` | XVIII | Cross-module inlining. Because each module is one translation unit, … |
| `[CG-C-3a]` | XVIII | `@inline` is binding: the function is emitted with `__forceinline` or … |
| `[CG-C-3b]` | XVIII | The bodies it exports are part of the interface hash (`[BLD-2]`). |
| `[CG-C-4]` | XVIII | Aliasing facts. For each loop, the base pointer of every view whose … |
| `[CG-C-5]` | XVIII | Every panic function is `_Noreturn` and cold, and each check branches … |
| `[CG-C-6]` | XVIII | A loop in vectorisable form (`[SIMD-5]`) is preceded by the host … |
| `[CG-C-7]` | XVIII | Locals keep their Ember names in the C (transliterated per … |
| `[CG-C-8]` | XVIII | Every emitted statement is preceded by a `#line` naming the Ember … |
| `[CG-C-9]` | XVIII | The build emits debugger visualisers (`.natvis`, GDB and LLDB … |
| `[CG-C-10]` | XVIII | Stack traces print Ember function paths and `file:line:col`; `ember … |
| `[CG-C-11]` | XVIII | Floating-point flags. Every translation unit begins with `#pragma … |

## CLI

| Rule | Part | Begins |
|---|---|---|
| `[CLI-1]` | XVII | Every command accepts `--json` and exits non-zero on error. |
| `[CLI-2]` | XVII | `ember build --emit c --out-dir <dir>` writes the C sources without … |
| `[CLI-4]` | XVII | `ember run file.em` and `ember build file.em` accept a single file … |
| `[CLI-5]` | XVII | `ember bind --init <header>` writes a starter overlay: every derived … |
| `[CLI-6]` | XVII | `ember bind --report` lists every skipped declaration and every … |
| `[CLI-7]` | XVII | `ember bind --check <overlay>` checks `[FFI-12]` and prints how many … |
| `[CLI-9]` | XVII | `ember check --syntax-only` lexes and parses only, reporting `E00xx` … |
| `[CLI-10]` | XVII | `ember build --report=engine` is a report, not a profile: it changes … |
| `[CLI-11]` | XVII | `ember audit` prints a one-page summary of a package's safety … |
| `[CLI-12]` | XVII | `ember why --alloc <item>` (and `--block`, `--io`, `--lock`, … |
| `[CLI-13]` | XVII | `ember calls --foreign <item>` lists every foreign function the item … |
| `[CLI-15]` | XVII | `ember --help` lists every command and flag of this section; one … |
| `[CLI-18]` | VIII | For both commands `<path>` is a package directory (its manifest and … |
| `[CLI-19]` | XVII | The implementation matrix. `ember --version --matrix` prints, for … |
| `[CLI-20]` | XVII | `ember fmt --migrate` rewrites source written for 0.9.8 into 0.9.9 … |
| `[CLI-21]` | XVII | `ember run --interp` runs a program on the compile-time evaluator … |

## CLO

| Rule | Part | Begins |
|---|---|---|
| `[CLO-1]` | VI | A lambda or local function has a unique anonymous type implementing … |
| `[CLO-2]` | VI | Captures are inferred per variable: read only ⇒ shared borrow; … |
| `[CLO-3]` | VI | What `fn(A) -> R` means depends on where it is written. * As a … |
| `[CLO-4]` | VI | A non-`owned` lambda cannot outlive what it borrows: storing it, … |
| `[CLO-5]` | VI | A closure capturing a class handle holds a strong reference; the … |
| `[CLO-6]` | VI | A lambda that moves one of its captures out of itself (into an … |
| `[CLO-6a]` | VI | An owned `once fn` value, including one inside a `Box` or a … |
| `[CLO-7]` | VI | Standard-library APIs that store or send a callback (`thread.spawn`, … |
| `[CLO-10]` | VI | An owned callable value holds up to three pointer-sized words of … |
| `[CLO-11]` | VI | A call `recv.name(args)` where `recv`'s type has no method `name` but … |
| `[CLO-12]` | VI | A local function (`[GRM-28]`) is a named closure: it captures like a … |
| `[CLO-13]` | VI | A read-only capture of a `Copy` variable whose storage would end … |
| `[CLO-14]` | VI | A callable type may also be written as an explicit generic bound, `fn … |
| `[CLO-15]` | VI | A lambda written directly as the argument of a consumed callable … |

## CLS

| Rule | Part | Begins |
|---|---|---|
| `[CLS-1]` | V | A class instance lives on the heap with the header of `[OBJ-1]`. … |
| `[CLS-2]` | V | `fn init(self, …)` is the constructor. Before any `init` body runs, … |
| `[CLS-3]` | V | A class with no `init` and no base class gets a memberwise … |
| `[CLS-4]` | V | A class is final unless declared `open` or `abstract`. Methods are … |
| `[CLS-5]` | V | Interface calls on a class are dispatched statically unless the … |
| `[CLS-6]` | V | Destruction runs the derived `drop`, then the base's, then drops the … |
| `[CLS-7]` | V | Inside a class method, `self` is a handle. Any method may read and … |
| `[CLS-7a]` | V | Inside `drop`, `self` MUST NOT be stored anywhere that outlives the … |
| `[CLS-8]` | V | A class is `Sync` only as `[THR-1]` allows; every field of a `Sync` … |
| `[CLS-9]` | V | A `let` field is assignable only in `init`. |
| `[CLS-9a]` | V | A `let` field of a non-`Copy` type may still be mutated *through* … |
| `[CLS-10]` | V | A derived class with no `init` gets its base's constructor: … |
| `[CLS-11]` | V | Construction is two-phase. In a derived `init`, the code before … |

## CONF

| Rule | Part | Begins |
|---|---|---|
| `[CONF-1]` | XVII | A compiler declares the profile it implements, and claims it only … |
| `[CONF-2]` | XVII | Ember Core: Parts II–VII, X, XIII and XV's core and alloc layers. |
| `[CONF-3]` | XVII | Ember Systems: adds Parts VIII, IX, XI, XII and XIV. |
| `[CONF-4]` | XVII | Ember Native: adds Part XVI. Annex C (C++) is a separate, optional … |
| `[CONF-5]` | XVII | Ember Dynamic: adds Annex B (hot reload). |

## CORO

| Rule | Part | Begins |
|---|---|---|
| `[CORO-1]` | VI | A `gen fn` declares a generator. Calling it runs none of its body; it … |
| `[CORO-2]` | VI | `yield e` suspends the generator and produces `e`. `yield` outside a … |
| `[CORO-3]` | VI | `Generator[Y, R]` in a signature names the function's own frame type … |
| `[CORO-4]` | VI | The compiler rewrites the body into a state machine over the frame: … |
| `[CORO-5]` | VI | A generator allocates nothing. The frame's size is a compile-time … |
| `[CORO-6]` | VI | A reference or view to a local of the generator's own frame may not … |
| `[CORO-7]` | VI | Dropping a suspended generator drops exactly the locals live at its … |
| `[CORO-8]` | VI | A generator's effect set is the union over its whole body; contracts … |
| `[CORO-9]` | VI | A frame never points into itself in Safe code; `unsafe` code that … |
| `[CORO-10]` | VI | A `gen fn` may not be `extern`, `@export`ed or passed to C (`E2222`). |
| `[CORO-11]` | VI | `std.coroutine` builds gameplay sequencing on generators with no … |
| `[CORO-12]` | VI | A `gen fn` method of a class takes `self` (the frame retains the … |
| `[CORO-13]` | VI | A `yield` while a `@must_drop` value (`[THR-6]`) is live is `E2231`: … |

## COST

| Rule | Part | Begins |
|---|---|---|
| `[COST-1]` | X | Zero cost, defined. An abstraction is zero-cost *for a use* whose … |
| `[COST-2]` | X | Every implicit cost is one of: guaranteed elided (emitting it is a … |
| `[COST-3]` | X | The costs. |
| `[COST-4]` | X | `ember inspect --cost <item>` prints every row that applies to an … |
| `[COST-5]` | X | A rule that introduces an implicit cost MUST add a row to this table; … |

## CT

| Rule | Part | Begins |
|---|---|---|
| `[CT-1]` | XIV | These are evaluated during compilation: `comptime(e)`; `comptime:` … |
| `[CT-2]` | XIV | An operation that cannot run at compile time — an `extern` call, a … |
| `[CT-3]` | XIV | Each evaluation is limited to 10⁸ steps and 256 MB of evaluator heap … |
| `[CT-4]` | XIV | Compile-time evaluation is deterministic by construction: the … |
| `[CT-5]` | XIV | Where results live. A compile-time result is placed in the image as … |
| `[CT-6]` | XIV | `comptime(e)` is an expression whose value is `e` evaluated at … |
| `[CT-7]` | XIV | A `comptime:` block at item level, or as a statement, runs once … |

## CTL

| Rule | Part | Begins |
|---|---|---|
| `[CTL-0]` | VI | The condition of `if`, `elif`, `while` and a match guard MUST be a … |
| `[CTL-1]` | VI | `for pattern in e:` iterates: * a place `e` whose type is `Iterable`: … |
| `[CTL-2]` | VI | The iterated place is borrowed for the whole loop; mutating it inside … |
| `[CTL-3]` | VI | `a..b` (`Range`), `a..=b` (`RangeInclusive`) and `a..` (`RangeFrom`) … |
| `[CTL-3b]` | VI | Iteration over ranges, `Span`, `MutSpan`, `Array`, `[T; N]`, `SoA` … |
| `[CTL-4]` | VI | The `else` of `while` or `for` runs when the loop ends without … |
| `[CTL-5]` | VI | A `match` tests arms top to bottom; the first that matches runs; a … |
| `[CTL-6]` | VI | `with a = e1, b = e2:` binds `a` and `b` for the block and drops them … |
| `[CTL-7]` | VI | `defer:` registers a block to run when the enclosing block exits, … |
| `[CTL-8]` | VI | On every exit from a block, its `defer` blocks run first and then its … |
| `[CTL-9]` | VI | `pass` does nothing; an empty block is written `pass`. |
| `[CTL-10]` | VI | Names assigned in every branch. In an `if`/`elif`/`else` that has an … |

## CXX

| Rule | Part | Begins |
|---|---|---|
| `[CXX-1]` | Annex C | The conformance suite for this annex includes a corpus of real … |

## DET

| Rule | Part | Begins |
|---|---|---|
| `[DET-1]` | X | `@deterministic` on a function or module is a contract: `Nondet` MUST … |
| `[DET-2]` | X | `Nondet` is introduced by exactly: floating-point contraction, … |
| `[DET-3]` | X | A `@deterministic` function may call only `Nondet`-free functions; a … |
| `[DET-4]` | X | `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, … |
| `[DET-5]` | X | Inside a `@deterministic` function the backend never contracts, … |
| `[DET-6]` | X | `@deterministic` constrains results, not timing. |
| `[DET-7]` | X | The cross-machine claim holds for one binary. `ember build … |
| `[DET-8]` | X | `@deterministic` on a module applies to every function in it, with no … |
| `[DET-9]` | X | `ember inspect --deterministic <item>` prints whether the item … |
| `[DET-10]` | X | A deterministic program runs with a defined floating-point … |

## DIA

| Rule | Part | Begins |
|---|---|---|
| `[DIA-1]` | XVII | A diagnostic has a code, one primary span, labelled secondary spans, … |
| `[DIA-2]` | XVII | Messages start lowercase, have no trailing period, name the thing, … |
| `[DIA-3]` | XVII | Ownership and borrow errors include the "later used here" label and a … |
| `[DIA-4]` | XVII | Contract errors print the whole call chain (`[EFF-6]`). |
| `[DIA-5]` | XVII | FFI errors name the header and the C declaration. |
| `[DIA-6]` | XVII | Every code has a page, `docs/errors/EXXXX.md`, with a program that … |
| `[DIA-6a]` | XVII | The code registry is exhaustive in both directions: every code this … |
| `[DIA-7]` | XVII | Every ownership and borrow error is classified into a shape of … |
| `[DIA-7a]` | XVII | Every code in `E3000`–`E3499` belongs to exactly one shape of … |
| `[DIA-8]` | XVII | `ember explain --borrow <file>:<line>` prints, for each loan live at … |
| `[DIA-9]` | XVII | A diagnostic never suggests `unsafe`, `Cell`, `RefCell`, `Shared` or … |
| `[DIA-10]` | XVII | The help of shapes B1, B2, B4, B5 and B9 names a concrete API or … |
| `[DIA-11]` | XVII | For shape S1 the diagnostic reports the check's reason (`[EFF-11]`); … |
| `[DIA-12]` | XVII | Every name and type error is classified into a shape of §XVII.6.2. An … |
| `[DIA-13]` | XVII | Every shape has a rendered snapshot under `tests/ui/` and a … |
| `[DIA-14]` | XVII | Only the first error of a cascade is reported: nothing is reported … |
| `[DIA-15]` | XVII | Suggestions are computed from data the compiler already holds (scope … |
| `[DIA-16]` | XVII | When a diagnostic suggests moving a value into a class, `Shared` or … |
| `[DIA-18]` | XVII | A foreign call rejected because its contract has unknown facts … |
| `[DIA-20]` | XVII | A run of invalid bytes or characters is one diagnostic, not one per … |
| `[DIA-21]` | XVII | Python habits. Each of these is recognised and answered with the … |
| `[DIA-22]` | XVII | A diagnostic caused by a callee's contract or signature is reported … |
| `[DIA-23]` | XVII | A run-time panic names Ember entities — the class, field, variable, … |
| `[DIA-24]` | XVII | Name suggestions (shape N1). A candidate is suggested when its … |

## DOC

| Rule | Part | Begins |
|---|---|---|
| `[DOC-1]` | XVII | Error pages ship with their errors. |
| `[DOC-2]` | XVII | The user guide (`docs/book/`), with a "coming from Python" chapter … |
| `[DOC-3]` | XVII | `ember doc` presents, for every function that produces or consumes a … |
| `[DOC-4]` | XVII | The guide, the error pages and this specification are published … |

## DRP

| Rule | Part | Begins |
|---|---|---|
| `[DRP-1]` | VII | `fn drop(mut self)` runs exactly once per value, at the end of its … |
| `[DRP-2]` | VII | Locals drop at the end of their block in reverse declaration order; … |
| `[DRP-3]` | VII | Temporaries drop at the end of the statement that created them … |
| `[DRP-4]` | VII | A panic inside `drop` aborts the process (`[PAN-1]`). A `drop` SHOULD … |
| `[DRP-5]` | VII | A `drop` body may not move fields out of `self` (`[EXP-6]`); it uses … |
| `[DRP-6]` | VII | Dropping a `Box[T]` drops the `T` and frees; dropping a class handle … |
| `[DRP-7]` | VII | A value whose `drop` may read through a reference it holds must be … |

## DRV

| Rule | Part | Begins |
|---|---|---|
| `[DRV-1]` | XIV | `@derive(…)` requests generated implementations. The built-in … |
| `[DRV-2]` | XIV | User-defined derives are not in this version; `macro` is reserved for … |

## DSJ

| Rule | Part | Begins |
|---|---|---|
| `[DSJ-1]` | IX | `assert_disjoint(a, b) -> Result[(A, B), (A, B)]` compares the two … |
| `[DSJ-2]` | IX | The fact belongs to the returned values, not to a program point; … |
| `[DSJ-3]` | IX | The borrow checker treats the returned views as non-overlapping, and … |
| `[DSJ-4]` | IX | It applies to `Span`, `MutSpan`, `SoA` columns and arena views; two … |
| `[DSJ-5]` | IX | `assert_disjoint_or_panic(a, b) -> (A, B)` panics on overlap. Both … |
| `[DSJ-6]` | IX | `assert_disjoint` is usable in `@noalloc`, `@nosync` and … |
| `[DSJ-7]` | IX | `unsafe assume_disjoint(a, b) -> (A, B)` asserts without checking; … |
| `[DSJ-8]` | IX | There is no form that checks in one profile and assumes in another … |
| `[DSJ-9]` | IX | `assert_disjoint_all(v1, …, vn)` handles 2 to 8 views pairwise. |

## DSP

| Rule | Part | Begins |
|---|---|---|
| `[DSP-1]` | VIII | A call is dispatched statically when the receiver's static type is a … |
| `[DSP-2]` | VIII | A virtual call loads its slot from the object's type table; slots are … |
| `[DSP-3]` | VIII | A call through an interface-typed handle finds the interface's table … |
| `[DSP-4]` | VIII | `h as? D` walks the base chain; `a is b` compares addresses. |
| `[DSP-5]` | VIII | With the whole program visible, a virtual call with exactly one … |

## ECS

| Rule | Part | Begins |
|---|---|---|
| `[ECS-1]` | XII | `Entity` is a 32-bit generational handle: a 20-bit index and a 12-bit … |
| `[ECS-2]` | XII | Each component type is stored as a sparse set: a dense `Array[T]` or … |
| `[ECS-3]` | XII | `Query[(A, Mut[B], Option[C], Not[D])]` is a view over a world's … |
| `[ECS-4]` | XII | A query's read and write sets are compile-time constants computed … |
| `[ECS-5]` | XII | Adding, removing and destroying during iteration go through a … |
| `[ECS-6]` | XII | Iteration order is insertion order with swap-remove holes: … |
| `[ECS-7]` | XII | `@derive(Component)` registers the type in a compile-time component … |

## EFF

| Rule | Part | Begins |
|---|---|---|
| `[EFF-1]` | X | Effects are computed from each function's body and its callees', over … |
| `[EFF-2]` | X | A call through a callable parameter (a generic bound, `[CLO-3]`) … |
| `[EFF-3]` | X | An `extern` function carries the effects its contract declares … |
| `[EFF-4]` | X | Effects are part of a function's interface for incremental builds: a … |
| `[EFF-5]` | X | Each contract in the table forbids its effect in the function's … |
| `[EFF-6]` | X | A contract is checked over the whole reachable call graph — callees, … |
| `[EFF-6a]` | X | A `@static_safe` function may not take a parameter whose type forces … |
| `[EFF-7]` | X | `unsafe: @assume_noalloc(expr)` overrides the analysis for one call … |
| `[EFF-8]` | X | An `override` inherits its base method's contracts. |
| `[EFF-9]` | X | `RuntimeCheck(k)` enters a function's effect set when a check of kind … |
| `[EFF-10]` | X | The effect set records only which kinds occur. Per-site detail — … |
| `[EFF-11]` | X | Every emitted check carries one reason, and diagnostics use it: |
| `[EFF-12]` | X | `@static_safe` permits `RuntimeCheck(Bounds)`, `(Arithmetic)` and … |
| `[EFF-13]` | X | A long-term access through a class handle loaded from memory is … |
| `[EFF-14]` | X | Contracts compose; the usual inner-loop set is `@static_safe @noalloc … |
| `[EFF-15]` | X | Effects and contract verdicts are computed once, after the … |
| `[EFF-16]` | X | `@nopanic(explicit)` forbids only the panics the programmer writes … |
| `[EFF-17]` | X | `@nopanic(explicit)` is spelled with its argument; bare `@nopanic` is … |
| `[EFF-18]` | X | Effects are independent: a blocking `Mutex.lock` has `Sync + Lock + … |
| `[EFF-19]` | X | `@realtime` names the contract set declared by the manifest of the … |
| `[EFF-20]` | X | `@noio` forbids `Io`, including console output; `println` in a … |
| `[EFF-21]` | X | `@nolock` forbids `Lock`, including `try_lock`, which never blocks … |
| `[EFF-23]` | X | A contract attribute whose checking an implementation has not built … |

## ENM

| Rule | Part | Begins |
|---|---|---|
| `[ENM-1]` | III | Inside a pattern whose scrutinee type is known, a variant may be … |
| `[ENM-2]` | V | A `match` on an enum MUST be exhaustive (`E2090` lists the missing … |
| `[ENM-3]` | V | A unit-only enum is `Copy`, `Eq`, `Ord` (declaration order), `Hash` … |
| `[ENM-4]` | V | A payload enum is `Copy` only by `@derive(Copy)` with every payload … |

## ERR

| Rule | Part | Begins |
|---|---|---|
| `[ERR-1]` | XIII | `Option[T]` is absence: `Some(v)` or `None`. `Result[T, E = … |
| `[ERR-2]` | XIII | `e?` on a `Result[T, E]` in a function returning `Result[U, F]` … |
| `[ERR-3]` | XIII | `Error` is the interface of error types: `Display + Debug` with … |
| `[ERR-4]` | XIII | `Option` and `Result` provide `is_some`/`is_none`, `is_ok`/`is_err`, … |
| `[ERR-5]` | XIII | `Result` is `@must_use`: discarding one is `W2190` (an error under … |
| `[ERR-6]` | XIII | A foreign function's status code becomes a `Result` at the binding … |
| `[ERR-7]` | XIII | Identity conversion. The prelude provides `From[T]` for every `T` … |
| `[ERR-8]` | XIII | `AnyError` is the prelude's "any error" type: an owned, boxed `dyn … |
| `[ERR-9]` | XIII | The error parameter defaults to `AnyError` (§XIII.2), so `fn … |
| `[ERR-10]` | XIII | `r.context(msg)` on a `Result[T, E]` returns a `Result[T, AnyError]` … |
| `[ERR-11]` | XIII | The cost of `AnyError`. Converting an error into `AnyError` … |
| `[ERR-12]` | XIII | An `Err` returned from `main` (`[FN-8]`) prints `error: <Display>` to … |
| `[ERR-13]` | XIII | The standard library follows one convention, and user code SHOULD: an … |

## EXC

| Rule | Part | Begins |
|---|---|---|
| `[EXC-1]` | VIII | Beginning a write access to a field while any access to the same … |
| `[EXC-2]` | VIII | Beginning a read access to a field while a write access to it is … |
| `[EXC-3]` | VIII | The compiler MAY remove a check only when it proves no conflicting … |
| `[EXC-3a]` | VIII | Every removed check is recorded, with the condition that justified … |
| `[EXC-4]` | VIII | A `let` field is subject to the same rules: `let` fixes the binding, … |
| `[EXC-5]` | VIII | Accesses nested inside the same `mut self` method are reborrows and … |
| `[EXC-6]` | VIII | A panic from `[EXC-1]`/`[EXC-2]` names both the offending access and … |
| `[EXC-7]` | VIII | The opt-in lint `L3013` reports a long-term access held across a … |
| `[EXC-8]` | VIII | When a loop makes repeated long-term accesses to one object whose … |
| `[EXC-9]` | VIII | `[EXC-8]` applies only when the receiver's identity is … |
| `[EXC-10]` | VIII | An inner loop reuses an outer loop's hoisted access when its accesses … |
| `[EXC-11]` | VIII | A hoisted access is invisible to programs: it cannot be named, stored … |
| `[EXC-12]` | VIII | `ember inspect --safety` reports each check as `STATIC`, … |
| `[EXC-13]` | XVII | The performance suite contains class-handle loops with one stable … |
| `[EXC-14]` | VIII | The 0.9.8 `exclusivity = "unchecked"` setting is removed. Code that … |
| `[EXC-15]` | VIII | A `mut self` class method holds a write access to every field of the … |
| `[EXC-16]` | VIII | Assigning a value to a class field whose type is not `Copy` is a … |
| `[EXC-17]` | VIII | Instantaneous writes are not checked against long-term accesses, so a … |
| `[EXC-18]` | VIII | The dynamic access that a borrow through a class handle begins — a … |
| `[EXC-19]` | VIII | Access state is per field. Each non-`Copy` field of a class that is … |

## EXP

| Rule | Part | Begins |
|---|---|---|
| `[EXP-1]` | VI | Operands, arguments, and the elements of tuple, list, map and set … |
| `[EXP-2]` | VI | An assignment evaluates its right side first, into a temporary if it … |
| `[EXP-3]` | VI | `and` and `or` short-circuit. `x if c else y` evaluates `c` and then … |
| `[EXP-4]` | VI | A temporary created while evaluating an expression statement is … |
| `[EXP-5]` | VI | Mutation, a mutable borrow and a move require a place; `f(x).y = 1` … |
| `[EXP-6]` | VI | A place of a non-`Copy` type used as a value (assigned, passed to … |
| `[EXP-9]` | VI | `a is b` compares two class handles (or two references) for identity. … |

## FFI

| Rule | Part | Begins |
|---|---|---|
| `[FFI-1]` | XVI | Every foreign declaration has a contract: for each pointer-typed … |
| `[FFI-2]` | XVI | A call to a foreign function whose contract contains an `unknown` … |
| `[FFI-2a]` | XVI | Derived facts need no `unsafe`: `const T*` is a read-only borrow, … |
| `[FFI-3]` | Annex C | Ember never links against C++ mangled symbols. The importer generates … |
| `[FFI-4]` | XVI | No Ember panic crosses into foreign code (`[FFI-25]`). |
| `[FFI-5]` | XVI | Layouts are verified, not assumed: the importer records `sizeof`, … |
| `[FFI-5a]` | XVI | The generated shim (`[FFI-29]`) contains a `_Static_assert` for the … |
| `[FFI-6]` | XVI | `import c "header" with (…) [as name]` parses the header with … |
| `[FFI-6a]` | XVI | An object-like macro is imported when, after expansion, it is a … |
| `[FFI-6b]` | XVI | A function-like macro is exposed only when an overlay declares its … |
| `[FFI-7]` | XVI | A header is re-parsed only when its content, its overlay, or a … |
| `[FFI-8]` | XVI | Type mapping. |
| `[FFI-9]` | XVI | `extern "C"` is the platform C ABI (SysV AMD64, Windows x64, … |
| `[FFI-10]` | XVI | An `unsafe extern "C":` block declares foreign functions, statics and … |
| `[FFI-11]` | XVI | Contract vocabulary. A pointer contract has five axes; mutability … |
| `[FFI-11a]` | XVI | A pointer contract with no count word is `E5012`, which lists the … |
| `[FFI-11b]` | XVI | The two-call enumeration idiom (call once for the count, again to … |
| `[FFI-11c]` | XVI | `TODO(count)`, `TODO(nullable)`, `TODO(ownership)` and … |
| `[FFI-11f]` | Annex C | A result lifetime derived from `[[clang::lifetimebound]]` imports as … |
| `[FFI-12]` | XVI | An overlay entry whose name, parameter count or parameter names do … |
| `[FFI-13]` | XVI | An overlay may contain `extend` blocks with ordinary Ember wrapper … |
| `[FFI-14]` | XVI | The importer's result is a `.embind` file (versioned CBOR: header … |
| `[FFI-15]` | XVI | `cstr` is a borrowed, NUL-terminated C string (`c"…"` literals are … |
| `[FFI-16]` | XVI | An enum marked `@ffi(status, ok=X)` generates `struct <E>Error(code: … |
| `[FFI-17]` | Annex C | `import cpp "Header.hpp" with (project="engine", overlay="…", … |
| `[FFI-17a]` | Annex C |  |
| `[FFI-17b]` | Annex C | Templates are available only as explicit instantiations listed in … |
| `[FFI-17c]` | Annex C | Destruction of an Ember class derived from a C++ class is … |
| `[FFI-17d]` | Annex C | `@ffi(trampoline, owner="ember")`, the default, makes the Ember count … |
| `[FFI-17e]` | Annex C | The bridge types `CppVector[T]`, `CppString` and `CppShared[T]` are … |
| `[FFI-17f]` | Annex C | `ember inspect` reports each bridge operation that is a thunk call … |
| `[FFI-20a]` | XVI | A declaration the importer cannot represent is recorded with the … |
| `[FFI-21]` | XVI | A C function-pointer parameter accepts a capture-free Ember function … |
| `[FFI-22]` | XVI | Every exported function and trampoline first attaches the calling … |
| `[FFI-23]` | XVI | `Retained.pin(v)` hands an Ember-owned object to C for keeping: for a … |
| `[FFI-24]` | Annex C | Every imported C++ function has an exception policy, with a default. … |
| `[FFI-24b]` | Annex C | An overlay may assert `@ffi(noexcept)` for a function the header … |
| `[FFI-24c]` | Annex C | `ember inspect` reports each C++ call's mode (`noexcept (derived)`, … |
| `[FFI-24d]` | Annex C | A header that adds or removes `noexcept` changes the Ember signature; … |
| `[FFI-25]` | XVI | A panic in an exported function, or anywhere below it, aborts the … |
| `[FFI-26]` | XVI | `@export("symbol")` gives a function a stable C symbol with the C … |
| `[FFI-27]` | XVI | The runtime `ember_rt` is a C11 static library with no global … |
| `[FFI-28]` | XVI | A package built as `kind = "staticlib"` produces a library and header … |
| `[FFI-29]` | XVI | For every `import c` the importer emits a shim C file, compiled with … |
| `[FFI-29a]` | XVI | A binding never names a symbol with internal linkage. |
| `[FFI-29b]` | XVI | `implementation = ["MINIAUDIO_IMPLEMENTATION"]` compiles a … |
| `[FFI-29c]` | XVI | A call to a wrapped `static inline` function costs one extra call … |
| `[FFI-30]` | XVI | Two imports of one C declaration (by Clang's USR, through typedefs) … |
| `[FFI-30a]` | XVI | An overlay changes the signatures, safety and names of *functions* … |
| `[FFI-30b]` | XVI | `pub import c "…" as vk` re-exports the module under the ordinary … |
| `[FFI-30c]` | XVI | A package may distribute an overlay for a foreign module, and several … |
| `[FFI-31]` | XVI | A `cdylib` links its own copy of the runtime. The host calls … |
| `[FFI-31a]` | XVI | Its thread detachment never relies on a TLS destructor in the … |
| `[FFI-31b]` | XVI | No owning Ember value crosses a module boundary: class handles, … |
| `[FFI-31c]` | XVI | With `[build] runtime = "shared"`, several Ember modules in one … |
| `[FFI-32]` | Annex C | A C++ class that is standard-layout and trivially copyable, all of … |
| `[FFI-33]` | XVI | `@export(threads=any \| main \| creator)` states which threads may … |
| `[FFI-33a]` | XVI | Attaching a thread (`[FFI-22]`) gives it no right to touch another … |
| `[FFI-33b]` | XVI | `returns_owned`, `Retained` and callbacks may carry … |
| `[FFI-33c]` | XVI | Under `threads = main`, the exported wrapper checks, in every … |
| `[FFI-35a]` | XVI | A parameter with no `retained` word is recorded as "does not retain" … |
| `[FFI-36]` | Annex C | A C++ function returning a raw pointer with no ownership contract … |
| `[FFI-36a]` | XVI | `std.ffi.adopt[T](h) -> ForeignBox[T]` is an `unsafe fn` that takes … |
| `[FFI-36b]` | XVI | Adopting a type whose overlay says `adopt = false` is `E5052`. … |
| `[FFI-37]` | Annex C | `ember test --instrument-ffi` runs the tests with the allocation, … |
| `[FFI-37a]` | Annex C | The intercepted set is published and recorded in the evidence; an … |
| `[FFI-37b]` | Annex C | The evidence records how often each foreign function was called and … |
| `[FFI-37c]` | Annex C | `instrumented` means "no counterexample was observed on the paths … |
| `[FFI-37d]` | Annex C | Only effect facts (`Alloc`, `Block`, `Lock`, `Io`) can be … |
| `[FFI-38]` | XVI | The importer rejects rather than guesses: a fact it cannot establish … |
| `[FFI-39]` | Annex C | An Ember class may derive from a C++ class the overlay declares with … |
| `[FFI-39a]` | Annex C | The trampoline calls the overrides through their permanent thunks, so … |
| `[FFI-39b]` | Annex C | `super.init(…)` selects the C++ base constructor by arity and … |
| `[FFI-39c]` | Annex C | Passing `self` to a C++ API upcasts it and creates a `Retained` token … |
| `[FFI-39d]` | Annex C | Re-entrant calls from C++ into the same Ember object are expected. … |
| `[FFI-39e]` | Annex C | A C++ exception never crosses into Ember code, and an Ember panic … |
| `[FFI-40]` | Annex C | `const` methods import as `self`, non-`const` methods as `mut self`; … |
| `[FFI-40a]` | Annex C | `const` is not an aliasing guarantee in C++: a `const` method that … |
| `[FFI-41]` | Annex C | Static member functions import as associated functions and static … |
| `[FFI-42]` | Annex C | Overloaded operators map to Ember operator interfaces where one … |
| `[FFI-42a]` | Annex C | Iterating an imported C++ container borrows it mutably for the loop, … |
| `[FFI-43]` | Annex C | An overlay that marks a function both `noexcept` and throwing, or two … |
| `[FFI-44]` | Annex C | *Automatic* means no overlay is needed; *overlay* means a contract … |
| `[FFI-48]` | Annex C | Unsupported constructs (`E5034`, naming the row): |
| `[FFI-49]` | XVI | `@ffi(link_name="sym")` binds a declaration to a differently named … |
| `[FFI-50]` | Annex C | The manifest section `[cpp.<project>]` names the C++ project: … |

## FMT

| Rule | Part | Begins |
|---|---|---|
| `[FMT-1]` | XVII | `ember fmt` writes LF line endings, four-space indentation and at … |
| `[FMT-2]` | XVII | The formatter uses the `=>` form of a lambda whose body is one … |
| `[FMT-3]` | XVII | The formatter never emits `;` outside `[T; N]` and `[v; N]`. |

## FN

| Rule | Part | Begins |
|---|---|---|
| `[FN-1]` | V | Parameter modes. * `a: A` — borrowed (the default). The callee reads … |
| `[FN-1a]` | V | A `mut` parameter whose type is itself a view (`MutSpan[T]`) accepts … |
| `[FN-2]` | V | An omitted mode is borrowed. There is no by-copy mode; a callee that … |
| `[FN-2a]` | V | A call site never writes a mode. `f(x)` is written whatever mode `f` … |
| `[FN-3]` | V | A function returns by move. Returning a reference or view requires … |
| `[FN-4]` | V | A method's receiver follows the same modes: `self`, `mut self`, … |
| `[FN-5]` | V | Default argument expressions are evaluated at each call, after the … |
| `[FN-6]` | V | Callable types. A function is a value. A callable type is written … |
| `[FN-7]` | V | Recursion is permitted; tail calls are not guaranteed to be … |
| `[FN-8]` | V | `main` is `fn main()`, `fn main() -> Result[void, E]` for any `E: … |
| `[FN-9]` | V | A mode on a class-handle parameter governs the handle, not the … |
| `[FN-10]` | V | A function whose return type is `Result[void, E]` returns `Ok(())` … |

## GPU

| Rule | Part | Begins |
|---|---|---|
| `[GPU-1]` | Annex D | GPU objects are named by generational handles (`[HND-1]`), which are … |
| `[GPU-2]` | Annex D | `device.begin_frame() -> Option[Frame]`; `None` is normal (a … |
| `[GPU-3]` | Annex D | `frame.arena()` is the frame's transient allocator; its region is the … |
| `[GPU-4]` | Annex D | Writing or mapping a resource that the GPU may still be reading is a … |
| `[GPU-5]` | Annex D | Binding a resource declares its access; `cmd.read(h)`/`cmd.write(h)` … |
| `[GPU-6]` | Annex D | `device.destroy(h)` invalidates the handle at once and destroys the … |
| `[GPU-7]` | Annex D | In `debug`, dropping a device while handles are still live reports … |
| `[GPU-8]` | Annex D | `History[T]` owns the per-frame copies of a temporal resource … |
| `[GPU-9]` | Annex D | `std.gpu.graph` is an optional render-graph layer over `CommandList`: … |
| `[GPU-10]` | Annex D | `ember shader-bind <reflection.json>`, reading the reflection a … |
| `[GPU-11]` | Annex D | Ember assumes no shader language: anything that produces SPIR-V and … |

## GRM

| Rule | Part | Begins |
|---|---|---|
| `[GRM-1]` | III | A class has at most one base class, written in parentheses: `class … |
| `[GRM-2]` | III | A file contains imports and items and, in the entry file only (the … |
| `[GRM-3]` | III | `Array[T]`, `Map[K, V]`, `Option[T]`, `Result[T, E]`, `Span[T]`, … |
| `[GRM-4]` | III | `x = e` where no `x` is in scope declares `x` with the type of `e`; … |
| `[GRM-5]` | III | `a, b = e` destructures a tuple, a struct or a fixed array. The right … |
| `[GRM-6]` | III | The optional `else` of `while` and `for` runs when the loop ends … |
| `[GRM-7]` | III | `defer` blocks run in reverse order at the exit of the enclosing … |
| `[GRM-8]` | III | `name[…]` in expression position is resolved during name resolution: … |
| `[GRM-8a]` | III | Inside `[ ]` in expression position the parser commits to a type … |
| `[GRM-8b]` | III | An index whose argument is a type is `E2172`; an instantiation … |
| `[GRM-8c]` | III | `IDENT = type` inside `[ ]` is an associated-type binding … |
| `[GRM-8d]` | III | A `type_alias` with an `in` clause declares a range type (`[RNG-1]`). … |
| `[GRM-10]` | III | A `match` whose arms use `pattern: block` is a statement; one whose … |
| `[GRM-11]` | III | There are no block expressions. A value computed by several … |
| `[GRM-12]` | III | An identifier in pattern position names a unit variant or a `const` … |
| `[GRM-13]` | III | Matching a place that is not consumed binds `Copy` fields by value … |
| `[GRM-15]` | III | `owned e` in expression position is legal only as the iterable of a … |
| `[GRM-16]` | III | `return`, `break` and `continue` are expressions of type `Never`; … |
| `[GRM-17]` | III | A lambda with a `:` body inside brackets holds exactly one simple … |
| `[GRM-18]` | III | `;` never separates statements. `a = 1; b = 2` is `E0105`, whose help … |
| `[GRM-19]` | III | The pattern of a `condition` MUST be refutable. An irrefutable one is … |
| `[GRM-20]` | III | The attributes before a statement are limited to `@parallel`, … |
| `[GRM-21]` | III | `gen fn` declares a generator (`[CORO-1]`). It is legal wherever `fn` … |
| `[GRM-23]` | III | `x in c` and `x not in c` are membership tests at comparison … |
| `[GRM-24]` | III | Ember has one path separator, `.`. A path resolves left to right: … |
| `[GRM-25]` | III | A chain of the comparison operators `==`, `!=`, `<`, `>`, `<=`, `>=` … |
| `[GRM-26]` | III | A `{…}` atom is a map literal if its first element is followed by … |
| `[GRM-27]` | III | A comprehension is shorthand for an iterator pipeline and has exactly … |
| `[GRM-28]` | III | A `fn_decl` inside a block declares a local function. It may capture … |
| `[GRM-29]` | III | An `expr_list` of two or more expressions, or of one expression … |
| `[GRM-30]` | III | `as?` and `as!` are single tokens (`[LEX-21]`): `h as? D` is a … |
| `[GRM-31]` | III | A `some` type (`[TYP-32]`) is legal only as a function's return type, … |
| `[GRM-32]` | III | `comptime(e)` evaluates the expression `e` at compile time (`[CT-6]`). |
| `[GRM-33]` | III | A bodiless `fn_decl` is legal only in an `interface`, an … |
| `[GRM-34]` | III | `extend [T: B] Array[T] implements I:` declares a generic extension; … |
| `[GRM-35]` | XVI | Overlay grammar. An overlay is a file whose items are: |
| `[GRM-36]` | III | `ref e` and `ref mut e` borrow the place `e` (`[BRW-1]`). The operand … |
| `[GRM-37]` | III | A directive is a line beginning `#!` before the imports. `#! language … |
| `[GRM-38]` | III | A parenthesised comprehension `(e for x in it if c)` is a generator … |

## HASH

| Rule | Part | Begins |
|---|---|---|
| `[HASH-1]` | IV | `Hash.hash` is generic over `H: Hasher` and monomorphised; hashing a … |
| `[HASH-2]` | IV | `std.collections.DefaultHasher` is a fixed-seed hasher: the same keys … |
| `[HASH-3]` | IV | `Map` and `Set` MUST NOT weaken equality to compensate for an … |
| `[HASH-4]` | IV | `Map[K, V]` requires `K: Eq + Hash`. A map may call `hash` any number … |

## HEAP

| Rule | Part | Begins |
|---|---|---|
| `[HEAP-1]` | IX | Every heap type allocates through … |
| `[HEAP-2]` | IX | Growable buffers double, from a minimum of four elements; … |
| `[HEAP-3]` | IX | `Shared(value) -> Shared[T]` allocates one counted block holding … |
| `[HEAP-4]` | IX | `s.get() -> ref T` borrows the payload: it begins a checked read … |
| `[HEAP-5]` | IX | `s.get_mut() -> ref mut T` begins a checked write access (`[EXC-1]`) … |
| `[HEAP-6]` | IX | `Shared[T]` is `Copy`: copying retains, dropping releases. |
| `[HEAP-7]` | IX | `Weak[O]` exists for `O` a class handle, a `Shared[T]` or a … |
| `[HEAP-8]` | IX | Capacity arithmetic is checked: a requested capacity whose byte size … |
| `[HEAP-9]` | IX | Heap storage for elements of type `T` is aligned to at least … |
| `[HEAP-10]` | IX | `Shared[T]` is never `Send` or `Sync`: its counts and access state … |

## HND

| Rule | Part | Begins |
|---|---|---|
| `[HND-1]` | IX | A `Handle[T]` is a plain `Copy` value (index and generation). Every … |
| `[HND-2]` | IX | `Handle[T]` is a `u64`: 32 bits of index and 32 of generation. … |
| `[HND-3]` | IX | A slot whose generation is exhausted is retired and never reused, so … |

## HR

| Rule | Part | Begins |
|---|---|---|
| `[HR-1]` | Annex B | An edit to one function body in a 50k-line reloadable package MUST be … |
| `[HR-2]` | Annex B | Reload is transactional. A failure in plan or prepare leaves the … |
| `[HR-2a]` | Annex B | Plan reads schemas and the registered set and runs no user code. … |
| `[HR-3]` | Annex B | A reload is applied only inside `ember_reload_poll()`, and only when … |
| `[HR-3a]` | Annex B | Every thread that can run Ember code is registered: host threads on … |
| `[HR-4]` | Annex B | Old images are never unloaded, so a stale return address or retired … |
| `[HR-4a]` | Annex B | Statics live in a runtime-owned table, never in image memory. |
| `[HR-5]` | Annex B | A call to a reloadable function goes through its thunk. |
| `[HR-6]` | Annex B | Each reloadable function gets one thunk, at a fixed address for the … |
| `[HR-6a]` | Annex B | Hence function values, closure code pointers, method tables, drop … |
| `[HR-7]` | Annex B | Call-table slots are identified by mangled name (`[MNG-1]`), never by … |
| `[HR-8]` | Annex B | Each image carries a reload manifest section listing every reloadable … |
| `[HR-9]` | Annex B | A call through a thunk costs one load and one indirect jump; the … |
| `[HR-9a]` | Annex B | A reloadable function is never inlined, devirtualised or placed in … |
| `[HR-10]` | Annex B | `@noreload fn` is called directly and may be inlined; changing its … |
| `[HR-10a]` | Annex B | A `@noreload` function may call reloadable functions, through their … |
| `[HR-11]` | Annex B | Every type in a reloadable package has a schema: its kind, base, … |
| `[HR-11a]` | Annex B | Enum values are migrated by variant name, never by discriminant. |
| `[HR-12]` | Annex B | In a reloadable build, class instances are registered in a … |
| `[HR-12a]` | Annex B | The header size is a whole-process property: every package in a … |
| `[HR-13]` | Annex B | Value-typed data (structs in arrays, `SoA` columns, ECS storage, … |
| `[HR-13a]` | Annex B | Standard-library containers of a type from a reloadable package … |
| `[HR-13b]` | Annex B | Value data reachable only through raw pointers or foreign memory … |
| `[HR-14]` | Annex B | Migration is by name, and this table is exhaustive: |
| `[HR-15]` | Annex B | Migration keeps an instance's address where the new size fits; … |
| `[HR-15a]` | Annex B | A reference the runtime cannot enumerate — in foreign memory, behind … |
| `[HR-15b]` | Annex B | Every refusal is decided in plan, before prepare runs. |
| `[HR-16]` | Annex B | `fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]` … |
| `[HR-17]` | Annex B | A static whose type and initialiser are unchanged keeps its value; … |
| `[HR-17a]` | Annex B | Editing a static's initialiser changes its schema: the reload is … |
| `[HR-18]` | Annex B | A refusal returns a report naming each type and change that caused … |
| `[HR-18a]` | Annex B | A refusal is never partial: nothing of the new image is live. |
| `[HR-18b]` | Annex B | `ember build --reload --explain` predicts the outcome against the … |
| `[HR-19]` | Annex B | `reload = "bodies"` admits only function-body changes and changes … |
| `[HR-20]` | Annex B | An object pinned by a `Retained` token that foreign code holds is … |
| `[HR-21]` | Annex B | A foreign table of exported function pointers stays valid across … |
| `[HR-22]` | Annex B | Foreign objects owned by Ember are carried across unchanged. |
| `[HR-23]` | Annex B | A changed C++ header refuses the reload and names it (the host must … |
| `[HR-24]` | Annex B | An object in use by an in-flight GPU frame is migrated in place when … |
| `[HR-25]` | Annex B | `reload` is `"all"`, `"opt-in"`, `"bodies"` or `"none"` (`[MAN-7]`); … |
| `[HR-27]` | Annex B | A `shipping` build is bit-identical whether or not the source uses … |
| `[HR-28]` | Annex B | `ember inspect --safety` reports the thunk indirection of each … |
| `[HR-29]` | Annex B | With reload enabled, one shared runtime serves every image in the … |
| `[HR-30]` | Annex B | Host contract: `ember_reload_init(&config)` once; … |
| `[HR-31]` | Annex B | Polling never blocks; compilation runs in the background. |
| `[HR-32]` | Annex B | `ember_reload_stats()` reports the last reload split into compile, … |
| `[HR-33]` | Annex B | `ember run --hot` is the toolchain's own host: it builds under … |
| `[HR-34]` | Annex B | Prepare cannot panic: every operation it performs is either free of … |
| `[HR-35]` | Annex B | `migrate_from` cannot abort the process. It may not contain an … |
| `[HR-36]` | Annex B | Allocation during prepare comes from the transaction arena; … |
| `[HR-37]` | Annex B | The transaction arena holds everything prepare allocates and is … |
| `[HR-38]` | Annex B | `std.hot.ReloadError` is `SchemaRefused(type, reason)`, … |
| `[HR-39]` | Annex B | A foreign call inside `migrate_from` returns its failure as … |
| `[HR-41]` | Annex B | Failure matrix. |
| `[HR-42]` | Annex B | Publication. Entering Ember code increments the thread's depth … |
| `[HR-42a]` | Annex B | The counter is not a lock: entering Ember code is one relaxed … |
| `[HR-43]` | Annex B | `@allow_reload_terminate` on a `migrate_from` admits such calls and … |

## IDE

| Rule | Part | Begins |
|---|---|---|
| `[IDE-3]` | XVII | Every compiler stage after parsing produces a complete result for a … |
| `[IDE-4]` | XVII | Every editor request is answered from parsing, name resolution and … |
| `[IDE-6]` | XVII | The compiler keeps all its state in a session object that can be … |

## IFC

| Rule | Part | Begins |
|---|---|---|
| `[IFC-1]` | V | `extend T:` without `implements` adds inherent methods to `T`; it is … |
| `[IFC-2]` | V | Adding inherent methods to a type from another package is `E2120`; … |
| `[IFC-3]` | V | `interface Ord: Eq` requires implementers of `Ord` to implement `Eq`. |
| `[IFC-4]` | V | Interfaces may declare associated types and constants, not statics. |

## IMP

| Rule | Part | Begins |
|---|---|---|
| `[IMP-11]` | XVIII | The reference implementation verifies its intermediate representation … |

## JOB

| Rule | Part | Begins |
|---|---|---|
| `[JOB-1]` | XI | A job's captured state is stored inline in its queue slot when it … |
| `[JOB-2]` | XI | `jobs.scope()` follows `[THR-5]` and `[THR-11]`: jobs submitted to a … |
| `[JOB-3]` | XI | `s.submit_after(deps, f)` starts `f` only after the jobs in `deps` … |
| `[JOB-5]` | XI | `jobs.local_arena()` returns an arena view for the current job, reset … |

## LAY

| Rule | Part | Begins |
|---|---|---|
| `[LAY-2]` | IX | Layout attributes change layout exactly as stated, in every profile … |

## LEX

| Rule | Part | Begins |
|---|---|---|
| `[LEX-1]` | II | Source files are UTF-8. A byte-order mark is accepted and ignored; … |
| `[LEX-2]` | II | Line endings are LF or CRLF, both normalised to LF before … |
| `[LEX-3]` | II | The source extension is `.em`. |
| `[LEX-4]` | II | Indentation MUST use spaces. A tab at the start of a logical line is … |
| `[LEX-5]` | II | The lexer emits `NEWLINE`, `INDENT` and `DEDENT` with Python's … |
| `[LEX-6]` | II | Inside `(`, `[` and `{`, and inside a triple-quoted string, newlines … |
| `[LEX-6a]` | II | Inside brackets, a lambda's `:` body is a single simple statement … |
| `[LEX-7]` | II | A backslash at the end of a physical line joins it to the next. The … |
| `[LEX-8]` | II | Blank lines and comment-only lines do not affect indentation. |
| `[LEX-9]` | II | An indented block MUST be introduced by a line ending in `:`. A `:` … |
| `[LEX-10]` | II | There are no block comments. A line beginning `#!` before the first … |
| `[LEX-11]` | II | A `##` comment attaches to the next declaration, ignoring blank … |
| `[LEX-11a]` | II | A `##` comment on a comment-only line is emitted after the … |
| `[LEX-12]` | II | Identifiers are NFC-normalised; two identifiers are the same iff … |
| `[LEX-13]` | II | A lone `_` is the discard pattern and never names a variable. |
| `[LEX-14]` | II | A raw identifier `r#name` uses a keyword as a name (for imported C … |
| `[LEX-15]` | II | The table above is the complete reserved set. Contextual keywords are … |
| `[LEX-16]` | II | An integer literal without a suffix is an untyped integer: it takes … |
| `[LEX-17]` | II | A float literal without a suffix is an untyped float: it takes the … |
| `[LEX-17a]` | II | A float literal that receives `f32` or `f16` and has more significant … |
| `[LEX-18]` | II | `1.` followed by an identifier character is a method call on `1`; … |
| `[LEX-19]` | II | An f-string `{…}` contains a full expression. `{{` and `}}` are … |
| `[LEX-20]` | II | A string literal has type `str` with the static region. At a site … |
| `[LEX-21]` | II | Tokenisation is maximal munch: `//=` before `//` before `/`, `=` … |
| `[LEX-22]` | II | Ember has no lifetime syntax and never will. A `'` begins a character … |
| `[LEX-23]` | II | A line comment whose text begins `SAFETY:` or `SAFETY(<category>):`, … |
| `[LEX-24]` | II | A unary minus applied directly to an untyped integer literal forms a … |
| `[LEX-25]` | II | A raw string contains no escapes; a `r#"…"#` raw string may contain … |

## LNT

| Rule | Part | Begins |
|---|---|---|
| `[LNT-1]` | XVII | `L1001 unused binding`: a local never read on any path (names … |
| `[LNT-2]` | XVII | `L1002 assignment declares a new binding`: an unread new name within … |
| `[LNT-3]` | XVII | `L1001` and `L1002` are reported by `ember build` and `ember check`, … |
| `[LNT-4]` | XVII | `L2004`: a `gen fn` with no `yield`. |
| `[LNT-5]` | XVII | `L2005`: a `@noreload` function calling a reloadable one in a loop. |
| `[LNT-6]` | XVII | `ember lint` also reports, at `warn` unless `[lints]` says otherwise: … |

## LT

| Rule | Part | Begins |
|---|---|---|
| `[LT-1]` | VII | Signature elision. A source parameter (ODR-024) is: * a parameter … |
| `[LT-1a]` | VII | `@borrows(p, …)`, on its own line before the function, replaces the … |
| `[LT-1b]` | VII | The opt-in lint `L3014` reports rule 3 applying to more than one … |
| `[LT-3]` | VII | String literals, `bytes` literals, `static` items and views of them … |
| `[LT-4]` | VII | Arena allocations borrow the arena (`[ARN-1]`). |
| `[LT-5]` | VII | Regions of locals are inferred by non-lexical liveness (§XVIII.4). … |
| `[LT-6]` | VII | Named lifetimes are not part of Ember and will not be added. Where a … |
| `[LT-7]` | VII | Callable types. Each call through a value or parameter of callable … |
| `[LT-14]` | VII | A view type carries a compiler-internal region vector with one slot … |
| `[LT-16]` | VII | Constructing a view value keeps each field's region; it never … |
| `[LT-17]` | VII | Validity is conjunctive. A view value is usable only while every … |
| `[LT-18]` | VII | Every borrowed field is an ordinary loan under `[BRW-1]`–`[BRW-11]`; … |
| `[LT-20]` | VII | Moving, copying, destructuring, passing and returning a view … |
| `[LT-21]` | VII | Assigning a new view into a field recomputes that field's region; the … |
| `[LT-22]` | VII | A function returning a view type gets, from its body, a summary of … |
| `[LT-23]` | VII | `@borrows` on a function returning a multi-region view MUST NOT … |
| `[LT-24]` | VII | A region slot ends at the last use of its own field, not of the whole … |
| `[LT-25]` | VII | A view field may not borrow another field of the value that contains … |
| `[LT-26]` | VII | A view extends no lifetime: constructing, copying or storing it … |
| `[LT-27]` | VII | Different regions never prove that two views do not overlap in … |
| `[LT-30]` | VII | Regions exist only at compile time: two values of one view type with … |
| `[LT-34]` | VII | A view type whose fields all borrow from one source behaves exactly … |
| `[LT-35]` | VII | For each function that can receive a view value, the compiler knows … |
| `[LT-36]` | VII | Treating the view as a whole — passing it where every field may be … |
| `[LT-38]` | VII | Moving one field out moves only that field's constraint; the … |
| `[LT-39]` | VII | An operation that selects a field by a run-time value (reflection, a … |
| `[LT-42]` | VII | A non-`owned` closure capturing a view records the regions of the … |
| `[LT-43]` | VII | Region tracking never lets a view survive a `yield` that `[CORO-6]` … |
| `[LT-44]` | VII | A borrowed or `mut` `Arena`, `FixedArena` or `ScopedArena` parameter … |

## MAN

| Rule | Part | Begins |
|---|---|---|
| `[MAN-1]` | XVII | An invalid manifest, including one with an unknown key, is `E9001`, … |
| `[MAN-2]` | XVII | `ember.lock` records the resolved dependencies with content hashes; … |
| `[MAN-3]` | XVII | Every key in `[lints]` names a lint the compiler defines (`E9010` … |
| `[MAN-7]` | XVII | `[build] reload` is `"opt-in"`, `"all"`, `"bodies"` or `"none"`; it … |
| `[MAN-8]` | XVII | The sections are: `[package]` (`name`, `version`, `language`, `kind`, … |

## MNG

| Rule | Part | Begins |
|---|---|---|
| `[MNG-1]` | XVIII | Mangling is injective. A symbol is `em_` followed by each path … |
| `[MNG-2]` | XVIII | `@export("name")` sets the symbol exactly. |
| `[MNG-3]` | XVIII | Non-ASCII identifier characters are transliterated as `_uXXXX_` … |
| `[MNG-4]` | XVIII | Object structs, method tables and type information are named … |

## MOD

| Rule | Part | Begins |
|---|---|---|
| `[MOD-1]` | V | A package is a directory tree with an `ember.toml` at its root. A … |
| `[MOD-2]` | V | Items are private to their module unless marked. `pub(package)` makes … |
| `[MOD-3]` | V | `import a.b.c` binds the name `c` to module `a.b.c`, and `import … |
| `[MOD-4]` | V | Import cycles within a package are allowed; cycles between packages … |
| `[MOD-5]` | V | The prelude. Every module implicitly imports these names from `std`, … |
| `[MOD-7]` | V | A field declared `pub(read)` (or `pub(package, read)`) may be read … |
| `[MOD-8]` | V | `from m import *` binds every `pub` item of `m`. A name bound by two … |

## MONO

| Rule | Part | Begins |
|---|---|---|
| `[MONO-1]` | XVIII | Generic code is instantiated per distinct set of type arguments, from … |
| `[MONO-2]` | XVIII | The compiler records, per generic, how many instances it produced and … |
| `[MONO-3]` | XVIII | `[build] max_instantiations = N` sets a per-generic ceiling (unset by … |
| `[MONO-5]` | XVIII | A generic is shareable at a parameter `T` when `T` appears in its … |
| `[MONO-6]` | XVIII | Only for a shareable generic whose instance count exceeds the ceiling … |
| `[MONO-7]` | XVIII | `@always_specialize` forbids sharing for a generic; … |
| `[MONO-8]` | XVIII | A shared body computes exactly what the specialised ones would; it … |
| `[MONO-9]` | XVIII | A shared body is its own symbol and is deduplicated like any … |

## OBJ

| Rule | Part | Begins |
|---|---|---|
| `[OBJ-1]` | VIII | The header is 24 bytes on 64-bit targets and is part of the runtime … |
| `[OBJ-2]` | VIII | A handle points at offset 0. An interface-typed handle is the same … |
| `[OBJ-3]` | VIII | `weak` starts at 1 on behalf of all strong handles. When `strong` … |
| `[OBJ-4]` | VIII | Objects are allocated through the runtime allocator (`[RT-1]`). |
| `[OBJ-5]` | VIII | The runtime sets the deinitialising flag before the `drop` chain and … |

## OPT

| Rule | Part | Begins |
|---|---|---|
| `[OPT-1]` | VIII | When escape analysis proves that no handle to an object outlives the … |
| `[OPT-2]` | X | For a counted loop over `a..b` that indexes views at `i + c` for … |
| `[OPT-3]` | X | The loop bound in `[OPT-2]` may be written `s.len()`, a separate … |

## OWN

| Rule | Part | Begins |
|---|---|---|
| `[OWN-1]` | VII | Every value has exactly one owner: a local, a field of an owned … |
| `[OWN-2]` | VII | A value is dropped when its owner goes out of scope (block end, … |
| `[OWN-3]` | VII | A move transfers ownership and leaves the source uninitialised. Using … |
| `[OWN-4]` | VII | A loop body that moves a value declared outside the loop is `E3041`, … |
| `[OWN-5]` | VII | Assigning to a place that holds a live value evaluates the new value, … |
| `[OWN-6]` | VII | `mem.take(mut place: T) -> T` (leaves `Default`), `mem.replace(mut … |
| `[OWN-7]` | VII | A `Copy` value is duplicated bitwise on use and the source stays … |
| `[OWN-8]` | VII | `Clone.clone(self) -> Self` is the explicit deep copy. Cloning a … |

## PAN

| Rule | Part | Begins |
|---|---|---|
| `[PAN-1]` | VI | A panic prints `panic at <file>:<line>:<col>: <message>` (and a … |
| `[PAN-2]` | VI | Formatting a panic message allocates only for an f-string; … |
| `[PAN-3]` | VI | A panic inside a `drop` that runs while the process is already … |

## PAR

| Rule | Part | Begins |
|---|---|---|
| `[PAR-1]` | XI | A `@parallel` loop's body runs as a closure over index chunks on the … |
| `[PAR-2]` | XI | Iterations MUST be independent. The body MUST NOT contain `break`, … |
| `[PAR-2a]` | XI | A violation of clause (b) is `E7011`, `parallel loop has a … |
| `[PAR-2b]` | XI | The analysis runs after the body's calls are inlined. A call it … |
| `[PAR-3]` | XI | `@parallel(reduce=[total: +, best: max])` declares variables combined … |
| `[PAR-4]` | XI | A `@parallel` loop has the effects `Sync` and `Block` (it waits for … |
| `[PAR-5]` | XI | Parallel results do not depend on the machine. Chunk boundaries are a … |

## PHIL

| Rule | Part | Begins |
|---|---|---|
| `[PHIL-1]` | I | Two syntactically identical declarations in the same context have … |
| `[PHIL-2]` | I | No heap allocation happens that the source does not show. The … |
| `[PHIL-3]` | I | No implicit copy of a non-`Copy` value occurs. Copies of `Copy` … |
| `[PHIL-4]` | I | No implicit synchronisation occurs except through types documented to … |
| `[PHIL-5]` | I | Every safety check the compiler removes, it removes because it proved … |
| `[PHIL-6]` | I | Every expensive or dangerous conversion at an FFI boundary is … |
| `[PHIL-7]` | I | Panics are for programmer errors. Recoverable failures use `Result`. |
| `[PHIL-8]` | I | The enforcement ladder. Ember guarantees memory safety, lifetime … |
| `[PHIL-8a]` | I | Every rejection shape in §XVII.6 has a mandated `help` that, applied … |
| `[PHIL-9]` | I | Class instances carry a header and can therefore carry dynamic … |
| `[PHIL-10]` | I | A program containing no `unsafe` block, no `unsafe fn` and no false … |
| `[PHIL-11]` | I | What remains possible, and is therefore not a defect of the … |
| `[PHIL-12]` | I | No silent acceptance. Every construct a program writes — an … |
| `[PHIL-13]` | I | One meaning per program. No build profile, compiler flag, manifest … |
| `[PHIL-14]` | I | Python spelling, Python meaning. Where Ember accepts a spelling that … |
| `[PHIL-15]` | I | Costs are named. Every cost the language can insert without the … |

## PRF

| Rule | Part | Begins |
|---|---|---|
| `[PRF-1]` | XVII | A profile never changes what a program means. Every profile accepts … |
| `[PRF-2]` | XVII | Hot reload is a build mode admitted in `debug` and `release`, not a … |
| `[PRF-3]` | XVII | What a profile may set. |

## RC

| Rule | Part | Begins |
|---|---|---|
| `[RC-1]` | VIII | Copying a handle retains; dropping one releases; the release that … |
| `[RC-2]` | VIII | Guaranteed elisions. No retain or release is emitted in the cases … |
| `[RC-2a]` | VIII | Passing a handle to a borrowed parameter. |
| `[RC-2b]` | VIII | A handle read from a place and used only within one expression while … |
| `[RC-2c]` | VIII | A retain immediately followed by a release of the same handle with no … |
| `[RC-2d]` | VIII | A handle stored into a field from a temporary (a move, not a retain). |
| `[RC-2e]` | VIII | Handles yielded by a `for` over a borrowed collection, unless the … |
| `[RC-3]` | VIII | Further elisions are allowed only when semantics are preserved … |
| `[RC-4]` | VIII | A non-`Sync` class's counts, and a `Shared`'s, use plain loads and … |
| `[RC-5]` | VIII | A borrow whose place goes through a class handle or `Shared` (`ref … |
| `[RC-6]` | VIII | `ember inspect` lists every retain and release that survives inside a … |

## RFL

| Rule | Part | Begins |
|---|---|---|
| `[RFL-1]` | XIV | `reflect[T]()` at compile time returns a `TypeDesc`: name, kind, … |
| `[RFL-2]` | XIV | `@reflect` on a type emits run-time type information: … |
| `[RFL-3]` | XIV | User attributes are declared. A struct marked `@attribute` may be … |

## RNG

| Rule | Part | Begins |
|---|---|---|
| `[RNG-1]` | IV | A `type` alias with an `in` clause declares a nominal numeric type … |
| `[RNG-2]` | IV | Two range types are distinct even with equal representation and range … |
| `[RNG-3]` | IV | Construction from a value not known to be in range is `T.checked(v) … |
| `[RNG-3a]` | IV | Every range type with finite endpoints has `T.clamped(v) -> T`, … |
| `[RNG-4]` | IV | The compiler tracks a known range for numeric expressions — literals, … |
| `[RNG-4a]` | IV | For floats, a range fact comes only from the true arm of a comparison … |
| `[RNG-5]` | IV | Arithmetic on a range value yields its representation: `r * 2.0` is … |
| `[RNG-5a1]` | IV | The operators on range types come from compiler-generated … |
| `[RNG-5a2]` | IV | Overload resolution picks an implementation matching the operand … |
| `[RNG-6]` | IV | NaN is in no range. A range from `-0.0` to `+0.0` contains both zeros. |
| `[RNG-7]` | IV | A range type whose range does not cover its whole representation … |
| `[RNG-8]` | IV | A range type is `Copy` when its representation is, has its … |
| `[RNG-9]` | IV | A range value outside its range is invalid; producing one is … |
| `[RNG-10]` | IV | In Safe code a range value arises only from a constant in range, … |
| `[RNG-10a]` | IV | A derived `Deserialize` checks every range-typed field with `checked` … |
| `[RNG-10b]` | IV | A range type may not appear in a foreign signature, directly or … |

## RT

| Rule | Part | Begins |
|---|---|---|
| `[RT-1]` | XVIII | The runtime `ember_rt` is C11 depending on libc and the OS only. … |
| `[RT-2]` | XVIII | The runtime has no global constructors; the generated `main` calls … |
| `[RT-3]` | XVIII | `TypeInfo` holds size, alignment, flags, name, base, drop functions, … |
| `[RT-4]` | XVIII | A panic prints `panic at <file>:<line>:<col>: <message>` naming Ember … |
| `[RT-5]` | XVIII | Every runtime symbol, macro and header name derives from one … |
| `[RT-6]` | XVIII | The runtime ABI version is `EMBER_RUNTIME_ABI`. |
| `[RT-7]` | XVIII | A reference count never wraps: a retain that would overflow panics … |
| `[RT-8]` | XVIII | Counts of `@sync` objects use a relaxed increment for retain and an … |
| `[RT-10]` | XVIII | Counting is inline. The fast path of retain (one increment and an … |
| `[RT-11]` | XVIII | Allocation statistics are kept per thread, or only in `debug`; no … |
| `[RT-12]` | XVIII | Stack overflow faults. The backend compiles with stack probes … |

## SEL

| Rule | Part | Begins |
|---|---|---|
| `[SEL-1]` | IX | The order above is the order to try. `ember inspect --alloc` reports … |
| `[SEL-2]` | IX | `Shared`/`Weak` and C++'s `std::shared_ptr`/`std::weak_ptr` … |

## SER

| Rule | Part | Begins |
|---|---|---|
| `[SER-1]` | XIV | `@derive(Serialize, Deserialize)` implements `serialize[W: … |
| `[SER-2]` | XIV | Deserialising never produces an invalid value: a range-typed field is … |

## SIMD

| Rule | Part | Begins |
|---|---|---|
| `[SIMD-1]` | XII | `std.simd` provides vector types `f32x4`, `f32x8`, `f32x16`, `f64x2`, … |
| `[SIMD-2]` | XII | `@simd` on a `for` loop is a request and a report: the compiler tries … |
| `[SIMD-3]` | XII | Alias facts given to the backend (`restrict`) MUST be derived, never … |
| `[SIMD-4]` | XII | Horizontal operations (`reduce_add`, `reduce_min`, `reduce_max`), … |
| `[SIMD-5]` | XII | Vectorisable form, which `@simd(assert)` checks, is computed by Ember … |
| `[SIMD-6]` | XII | The C compiler's own vectorisation report is corroborating evidence … |
| `[SIMD-7]` | XII | Grouped overflow checks. In a loop whose body has none of `Sync`, … |
| `[SIMD-8]` | XII | Vector operators are lane-wise and follow the scalar rules: integer … |
| `[SIMD-9]` | XII | SIMD memory operations are bounds-checked like indexing. `V.load(s)` … |

## SOA

| Rule | Part | Begins |
|---|---|---|
| `[SOA-1]` | XII | `SoA[T]` is a compiler-known type constructor, like `Cell`: for any … |
| `[SOA-2]` | XII | `ps.f` names the column of field `f`. It is a place of type … |
| `[SOA-3]` | XII | `SoA[T]` provides `len`, `is_empty`, `push`, `pop`, `swap_remove`, … |
| `[SOA-4]` | XII | `ArenaSoA[T]` is the fixed-capacity, arena-backed form, under the … |
| `[SOA-6]` | XII | `ps[i]` is an element proxy: `SoARef[T]` where `ps` is read-only here … |
| `[SOA-7]` | XII | All columns of one `SoA[T]` live in one heap block, each column … |

## SPN

| Rule | Part | Begins |
|---|---|---|
| `[SPN-1]` | VII | An `Array[T]`, a `[T; N]` or a `String` converts to `Span[T]`/`str` … |
| `[SPN-2]` | VII | Indexing a view is bounds-checked; `get(i) -> Option[ref T]` does not … |
| `[SPN-3]` | VII | `Span[T]` is `Copy`; `MutSpan[T]` is move-only and reborrowable … |
| `[SPN-4]` | VII | `iter()` on a `Span` or `MutSpan` yields `ref T`; `iter_mut()` on a … |
| `[SPN-5]` | VII | `split_at(i)` on a `Span` returns two `Span`s; on a `MutSpan` it … |
| `[SPN-8]` | VII | `as_ptr()` and `as_mut_ptr()` return raw pointers; extracting one is … |

## STA

| Rule | Part | Begins |
|---|---|---|
| `[STA-1]` | V | `static NAME: T = e` is one value per program with a stable address. … |
| `[STA-2]` | V | There is no static-initialisation-order problem: statics are … |
| `[STA-3]` | V | A static whose initialiser can be evaluated at compile time is placed … |

## STD

| Rule | Part | Begins |
|---|---|---|
| `[STD-1]` | XV | Every function in `std.core`, `std.mem`, `std.math`, `std.simd` and … |
| `[STD-2]` | XV | Printing allocates only to build an f-string argument; … |
| `[STD-3]` | XV | `math.fma(a, b, c)` (and `mul_add`) is a fused multiply-add with a … |
| `[STD-4]` | XV | `NonZero[T]` for each integer `T` is a `Copy` wrapper with a niche … |
| `[STD-5]` | XV | `sum` and `product` combine elements left to right in the element … |
| `[STD-6]` | XV | `std` is layered, and each layer depends only on the ones before it: … |
| `[STD-7]` | XV | Fixed-capacity containers with no heap allocation: `FixedArray[T, … |
| `[STD-8]` | XV | `Contains`. `x in c` requires `c: Contains[typeof(x)]` (`E2226` … |
| `[STD-8a]` | XV | `ember inspect --cost` reports which `contains` a use selects and its … |
| `[STD-8b]` | XV | `str` implements `Contains[char]` and `Contains[str]` only; matching … |
| `[STD-9]` | XV | `print(a, b, …, sep=" ", end="")` and `println(a, b, …, sep=" ", … |
| `[STD-10]` | XV | `input(prompt="") -> String` prints the prompt, flushes, reads one … |
| `[STD-11]` | XV | `Map[K, V, H = DefaultHasher, A = Global]` is a hash map that … |
| `[STD-12]` | XV | Borrowed keys. Lookup methods, `m[k]` and `k in m` accept any key … |
| `[STD-13]` | XV | One naming convention. Names are `snake_case` words joined by … |
| `[STD-14]` | XV | Failure follows `[ERR-13]`: a caller's bug panics and has a … |
| `[STD-15]` | XV | `Array[T]` is a growable contiguous list (Python's `list`). `len`, … |
| `[STD-16]` | XV | `Map` operations. `m[k]` reads the value and panics when the key is … |
| `[STD-17]` | XV | Index assignment. `a[i] = v` calls `IndexSet.index_set(i, v)` when … |
| `[STD-18]` | XV | `format(value, spec="") -> String` formats one value with a format … |
| `[STD-19]` | XV | Every `Iterator` has the adapters `map`, `filter`, `filter_map`, … |
| `[STD-20]` | XV | Integer methods: `abs`, `pow`, `signum`, `div_trunc`, `rem_trunc`, … |
| `[STD-21]` | XV | `std.math` provides `PI`, `TAU`, `E`; `sin`, `cos`, `tan`, `asin`, … |
| `[STD-22]` | XV | Every operation that touches the outside world returns `Result[T, … |
| `[STD-23]` | XV | `time.Instant.now()` is monotonic and `Nondet`; `Duration` is an … |
| `[STD-24]` | XV | `process.exit(code) -> Never` flushes the standard streams and exits … |
| `[STD-25]` | XV | `random.Rng.seeded(seed)` is a deterministic generator (the same seed … |
| `[STD-26]` | XV | Python's built-in functions. The prelude has these functions, with … |

## STR

| Rule | Part | Begins |
|---|---|---|
| `[STR-1]` | V | Every struct has a memberwise constructor `Name(field0, field1, …)` … |
| `[STR-2]` | V | A field default is any expression; it is evaluated at each … |
| `[STR-3]` | V | A struct with a `drop` method, or with a field that needs drop, is … |
| `[STR-4]` | V | A struct may have no fields (`struct Marker: pass`). |
| `[STR-5]` | V | Implicit derives. A struct or enum implements `Eq`, `Debug` and … |
| `[STR-6]` | V | `fn init(self, …)` on a struct is its constructor: every field … |
| `[STR-7]` | V | Inside the body of a `struct`, `enum`, `class` or `extend` block, … |

## TCB

| Rule | Part | Begins |
|---|---|---|
| `[TCB-1]` | Annex C | Every fact an overlay states about foreign code carries a grade: … |
| `[TCB-2]` | Annex C | The report separates what the language guarantees from what external … |
| `[TCB-3]` | Annex C | Every assumption a guarantee relies on appears in the report with its … |
| `[TCB-4]` | Annex C | Entries are categorised — language, compiler, runtime, standard … |
| `[TCB-5]` | Annex C | An instrumented fact is valid only for the foreign library, header, … |
| `[TCB-6]` | Annex C | A change to any identity input makes the record stale; a change to an … |

## THR

| Rule | Part | Begins |
|---|---|---|
| `[THR-1]` | XI | A class is `Sync` only when it is declared `@sync class`. Every other … |
| `[THR-2]` | XI | Handles of a `@sync` class are `Send` and `Sync`. A handle of any … |
| `[THR-3]` | XI | `Mutex[T]` is a value type. `m.lock()` blocks and returns a … |
| `[THR-4]` | XI | In the `debug` profile the runtime records the order in which each … |
| `[THR-5]` | XI | `thread.scope()` returns a `Scope`, which MUST be bound by a `with` … |
| `[THR-6]` | XI | A type whose `drop` a safety guarantee depends on is `@must_drop`. A … |
| `[THR-7]` | XI | `@sync` means exactly two things: the class's handles may cross … |
| `[THR-8]` | XI | A type is `Send` when a value of it may be moved to another thread. … |
| `[THR-9]` | XI | A type is `Sync` when several threads may read one value of it at the … |
| `[THR-10]` | XI | `thread.spawn(owned f: fn() -> R) -> JoinHandle[R]` runs `f` on a new … |
| `[THR-11]` | XI | `scope.spawn(f: fn() -> R) -> ScopedJoinHandle[R]` accepts a closure … |
| `[THR-12]` | XI | Tasks that must run in sequence and share data use two scopes in … |
| `[THR-13]` | XI | The data-race guarantee. In Safe Ember a memory location is reachable … |
| `[THR-14]` | XI | `Atomic[T]` exists for the integer types, `bool` and raw pointers. … |
| `[THR-15]` | XI | `channel[T](capacity=n) -> (Sender[T], Receiver[T])` is a bounded … |
| `[THR-16]` | XI | When `main` returns, or `process.exit` is called, the process ends: … |

## TIER

| Rule | Part | Begins |
|---|---|---|
| `[TIER-1]` | I | Safe code MUST NOT invoke an operation with an unverifiable … |

## TOOL

| Rule | Part | Begins |
|---|---|---|
| `[TOOL-1]` | XVII | Each release publishes a self-contained toolchain archive per … |
| `[TOOL-2]` | XVII | `ember toolchain install cc` installs a pinned Clang and linker and … |
| `[TOOL-3]` | XVII | When no C compiler is found, `E9002`'s message says so and its help … |
| `[TOOL-4]` | XVII | `ember --version` prints the compiler version, the language version, … |

## TST

| Rule | Part | Begins |
|---|---|---|
| `[TST-0]` | XVII | Test annotations are line comments beginning `#$`, read from the raw … |
| `[TST-1]` | XVII | `#$ error[E…]: text`, `#$ warning[…]` and `#$ note` assert a … |
| `[TST-2]` | XVII | `#$ stdout:`, `#$ exit: N`, and `#$ assert-c: contains("…")` check … |
| `[TST-3]` | XVII | `@test` functions run in the test binary, each isolated; … |
| `[TST-4]` | XVII | Every rule of this document has a directory … |
| `[TST-4a]` | XVII | Each rule's directory has an accept case and, for each diagnostic … |
| `[TST-4b]` | XVII | Which rules need a reject case is decided mechanically: a rule that … |
| `[TST-5]` | XVII | A scripted debugger session (breakpoint by Ember line, stepping, … |
| `[TST-6]` | XVII | Appendix A's code is generated from a fixture that is compiled in CI. … |
| `[TST-7]` | XVII | Every ` ```ember ` block in this document is extracted and must pass … |
| `[TST-8]` | XVII | `tests/firstweek/` holds at least 24 first-draft programs a newcomer … |
| `[TST-9]` | XVII | Each is marked `accepted` or `rejected(<shape>)`; a rejection whose … |
| `[TST-10]` | XVII | The acceptance rate is published with each release; a release that … |
| `[TST-11]` | XVII | Besides the per-rule cases, the suite covers these scenarios: … |
| `[TST-27]` | XVII | The C gate. Every accepted program in the test suite is compiled … |
| `[TST-28]` | XVII | The performance gate. `tests/perf/` holds benchmark programs, each … |
| `[TST-29]` | XVII | Honest baselines. A gate with a baseline of known failures reports … |
| `[TST-30]` | XVII | The test runner reports every failure of a run, not only the first, … |

## TXT

| Rule | Part | Begins |
|---|---|---|
| `[TXT-1]` | XV | `str` is a borrowed `Span[u8]` known to be valid UTF-8; `String` owns … |
| `[TXT-2]` | XV | Nothing becomes a `str` without validation: every conversion from … |
| `[TXT-3]` | XV | `str` and `String` are a pointer and a length, not null-terminated, … |
| `[TXT-4]` | XV | Slicing a string, `s[a..b]`, is by byte offset and panics in every … |
| `[TXT-5]` | XV | `str` → `String` allocates and copies; `String` → `str` is free; … |
| `[TXT-6]` | XV | Across the C ABI a `str` is `{const uint8_t *ptr; size_t len}`. |
| `[TXT-7]` | XV | Text is UTF-8 everywhere. `std.ffi.WideString` converts, explicitly, … |
| `[TXT-8]` | XV | `str` is a view type and carries a region, like every `Span`. |
| `[TXT-9]` | XV | A string literal initialises a `String`. Wherever a `String` is … |
| `[TXT-10]` | XV | `str` operations. `len()` is the length in bytes and `char_count()` … |
| `[TXT-11]` | XV | `String` operations. Everything `str` has (by read-through), plus … |

## TYP

| Rule | Part | Begins |
|---|---|---|
| `[TYP-1]` | IV | Every concrete type has, at compile time, a size, an alignment, and … |
| `[TYP-2]` | IV | Producing a `bool` other than 0 or 1 is undefined behaviour and … |
| `[TYP-3]` | IV | Producing a `char` outside the Unicode scalar values is undefined … |
| `[TYP-4]` | IV | No value converts implicitly between scalar types in an operator. … |
| `[TYP-5]` | IV | Coercions — the complete list. At a coercion site (assignment or … |
| `[TYP-6]` | IV | `x as T` converts explicitly: * integer → integer: keeps the low bits … |
| `[TYP-7]` | IV | `as` between pointer types, or between a pointer and an integer, … |
| `[TYP-8]` | IV | Integer overflow panics in every profile. An arithmetic operation (`+ … |
| `[TYP-9]` | IV | Floating point is strict IEEE 754: no reassociation, no contraction … |
| `[TYP-9a]` | IV | Contraction is off by default and the implementation MUST turn it off … |
| `[TYP-9b]` | IV | `@fp(contract)` permits, and requires the backend to enable, fused … |
| `[TYP-9c]` | IV | A toolchain that cannot honour `@fastmath` or `@fp(…)` for one … |
| `[TYP-10]` | IV | Shifts. `a << n` and `a >> n` accept any integer type for `n`. If `n` … |
| `[TYP-11]` | IV | A struct's default layout is C's: declaration order, natural … |
| `[TYP-12]` | IV | A unit-only enum is an integer (`@repr(u8)` and friends choose it; … |
| `[TYP-13]` | IV | `Option[T]` has the size of `T` when `T` has a niche: a class handle, … |
| `[TYP-14]` | IV | A reference, and a `Box[T]`, is read through wherever a `T` is wanted … |
| `[TYP-15]` | IV | Where views may be stored. A view value may be stored only in a place … |
| `[TYP-15a]` | IV | The arena-backed containers (`ArenaArray`, `ArenaMap`, §IX.2) and the … |
| `[TYP-16]` | IV | Generic functions and types are monomorphised: each distinct … |
| `[TYP-17]` | IV | Type parameters are bounded by interfaces: `fn sum[T: Add[Output = T] … |
| `[TYP-18]` | IV | Generic arguments are inferred from the arguments of a call. Explicit … |
| `[TYP-19]` | IV | There is no specialisation, no higher-kinded type and no variadic … |
| `[TYP-20]` | IV | Coherence is per package. An implementation of interface `I` for type … |
| `[TYP-21]` | IV | Operators desugar to these interfaces for non-scalar operands; `a + … |
| `[TYP-22]` | IV | An interface is usable as `dyn` only if every method has a receiver, … |
| `[TYP-23]` | IV | Inference is local to a function body and bidirectional. Function … |
| `[TYP-24]` | IV | An interface method is found through any implementation visible in … |
| `[TYP-25]` | IV | Arguments may be positional or named; positional arguments come … |
| `[TYP-26]` | IV | There is no overloading: two functions of one name in one scope are … |
| `[TYP-27]` | IV | `()` is the value of type `void`; `Ok(())` is the success value of … |
| `[TYP-28]` | IV | Division. `/` is true division and applies to floats. `/` with two … |
| `[TYP-29]` | IV | For floats, `//` and `%` are Python's (ODR-021). `a % b` is the exact … |
| `[TYP-30]` | IV | Powers. `a  b` with integer `a` and non-negative integer `b` is … |
| `[TYP-31]` | IV | Sizes and indices are `int`. Every standard container's `len()` … |
| `[TYP-32]` | IV | `some I` in a return type means "one concrete type, chosen by the … |
| `[TYP-34]` | IV | *(replaces the 0.9.8 `@view` requirement)* A struct, enum or tuple … |
| `[TYP-35]` | IV | A type parameter need not appear in any field (a phantom parameter): … |
| `[TYP-36]` | IV | Which types implement which interfaces. The table is normative; a `—` … |
| `[TYP-37]` | IV | Floats implement `Eq` with IEEE `==` (so `NaN != NaN`) and `Ord` with … |
| `[TYP-38]` | VI | Collection literals. * A list literal `[a, b, c]` has the type its … |
| `[TYP-39]` | IV | Collections, tuples and `Option`/`Result` implement `Display` the way … |
| `[TYP-40]` | IV | Interfaces are nominal. A type implements an interface only through … |

## UNS

| Rule | Part | Begins |
|---|---|---|
| `[UNS-1]` | IX | An `unsafe` context is required to: dereference, read or write … |
| `[UNS-2]` | IX | `unsafe` permits exactly those operations. It does not turn off … |
| `[UNS-3]` | IX | `L3010` reports an `unsafe` block containing statements that need no … |
| `[UNS-4]` | IX | Unsafe code MUST uphold what safe code assumes: every reference is … |
| `[UNS-5]` | IX | `std.mem` provides `Volatile[*T]` (volatile reads and writes), … |
| `[UNS-6]` | IX | Inline assembly is `unsafe asm("…", …)` on toolchains that support it … |
| `[UNS-7]` | IX | Every `pub unsafe fn` carries `@safety("…")` stating the caller's … |
| `[UNS-8]` | IX | Every `unsafe:` block and `unsafe fn` carries a safety note … |
| `[UNS-10]` | IX | `UnsafeCell[T]` (in `std.mem`) is the primitive beneath every … |
| `[UNS-10a]` | IX | `UnsafeCell` suspends no rule globally: borrow, region, type and … |

## VER

| Rule | Part | Begins |
|---|---|---|
| `[VER-1]` | 0 | Three version numbers exist and move independently: the language … |
| `[VER-2]` | 0 | From 1.0: source compatibility within a major language version; a … |
| `[VER-3]` | 0 | From 1.0: deprecation through `@deprecated(since, note)`, removal no … |
| `[VER-4]` | 0 | The runtime ABI (object header, `ember_type_info`, every entry point … |
| `[VER-5]` | XVII | Package versions are semantic; a requirement `"1.2"` means `>= 1.2.0, … |
| `[VER-8]` | 0 | Before 1.0 there is exactly one language: the current one. A source … |
| `[VER-9]` | 0 | A language revision that changes the set of accepted programs or … |

## WK

| Rule | Part | Begins |
|---|---|---|
| `[WK-1]` | VIII | A cycle of strong handles among class instances and `Shared` payloads … |
| `[WK-2]` | VIII | An object is deinitialised when its strong count reaches zero, … |
| `[WK-3]` | VIII | `upgrade` returns `None` while the object's deinitialising flag is … |
| `[WK-4]` | VIII | The leak report names, for each leaked object on a cycle, the … |
| `[WK-5]` | VIII | The compiler builds a graph of strong ownership among class fields … |
| `[WK-6]` | VIII | A cycle of strong edges in that graph is warning `L3001` at the field … |
| `[WK-7]` | VIII | Cycle analysis is conservative: a possible cycle suffices for … |
| `[WK-8]` | VIII | The run-time report (`[WK-15]`) lists each leaked strongly connected … |
| `[WK-9]` | VIII | `ember explain --cycle <path> <Class[.field]>` explains one … |
| `[WK-11]` | VIII | `Weak(h)` creates a weak handle to a class object or a `Shared` or … |
| `[WK-12]` | VIII | `w.upgrade() -> Option[O]` returns a retained strong handle while the … |
| `[WK-13]` | VIII | A `Weak[Shared[T]]` refers to the same block as its `Shared[T]`. |
| `[WK-14]` | VIII | Ember's `Weak` and `Shared` never convert to or from C++'s … |
| `[WK-15]` | VIII | `ember run` and `ember test` in the `debug` profile report leaked … |
