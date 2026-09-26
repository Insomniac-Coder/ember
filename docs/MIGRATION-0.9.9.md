# Implementing Ember 0.9.9

**State as of 2026-09-23.** On 2026-09-23 the owner directed that the compiler be moved to the 0.9.9
language: "start working on the language development, add/modify/update the current implementation",
continuing until the language is complete or the owner says stop. This document is the plan and the
progress record for that work. It is not a second specification.

## 1. Target and authority

* **Target:** `docs/spec-source/Ember_v0.9.9_Hardened_4.md` (926 rules), pinned by hash in
  `docs/spec-source/development-target.json` so `tools/rule_index.py` accepts its rule
  citations. Its sources (`parts/`, and the frozen `parts-h1/`–`parts-h3/`), the three review
  passes, the change list, the sanity check and the build tools are under `tasks/spec-0.9.9/` on
  this branch, which is their canonical copy; `tasks/spec-0.9.9/HANDOFF.md` records the spec
  process. H1–H3 are frozen; a later hardening diffs against the frozen predecessor.
* **Tools for this work:** `tasks/impl-0.9.9/survey.py` runs `ember check` over every test file
  (about a minute) and lists what a change breaks; `tasks/impl-0.9.9/migrate_literals.py` applies
  the compiler's ODR-022 annotation help to a file.
* **One language.** `[VER-8]`: before 1.0 there is exactly one language, the current one. The
  compiler already implements one evolving language (the `#! language` directive is validated but
  never consulted), so each 0.9.9 change is made directly and the conformance corpus is migrated in
  the same slice.
* **Ambiguities.** Each one found while implementing becomes an **ODR** (next: ODR-021) in
  `docs/OWNER-QUEUE.md`. For 0.9.9 the owner has delegated the ruling: the ODR records the
  conflicting text, the options and the decision, and the decision is published as the next hardening
  of 0.9.9 (Hardened_3, Hardened_4, …), listed in its Appendix H. The compiler never moves the text
  silently, and never adopts a reading because it matches what is built.
* **Process.** `docs/HANDOFF.md` §0.0 governs: probe first, the five-way sort, the four-document
  write path for a defect, break every new test once, claim discipline.
* **Branch.** Work happens on `main` (owner, 2026-09-23: "no need for a separate branch ... this
  language is not close to release"). Commit and push about every five features, with CI green;
  `tasks/impl-0.9.9/ci_status.py` reports the Actions runs without `gh`.

## 2. Probe results, 2026-09-23 (compiler at `761ce7c`)

| Probe | 0.9.9 rule | Result |
|---|---|---|
| `println("hello")` at file scope | `[GRM-2]`, `[FN-8]` | `E0100 expected a declaration` |
| `7 // 2`, `-7 // 2`, `-7 % 2` | `[TYP-28]` | `//` does not lex as an operator |
| `x = 7 / 2` on integers | `[TYP-28]` (`E2240`) | accepted, prints `3` |
| `x: int`, `z: float` | `[TYP-1]` | `E1010 cannot find type int` |
| comprehension, `{}` map, `{1, 2}` set, `0 <= q < 10` | `[TYP-38]`, `[GRM-25]`–`[GRM-27]` | comprehension does not parse |
| `return 1, 2`; `a, b = b, a` | `[GRM-29]` | unparenthesised tuple does not parse |
| `f"{x=} {x:>8} {name!r}"` | `[LEX-19]` | `a format spec is not supported yet` |
| `Node("a")` with a `String` field | `[TXT-9]` | `E2020 expected String, found str` |
| field default `children: Array[Node] = []` | `[TYP-38]`, `[STR-2]` | `E2060` (the field type does not reach the literal) |
| class holding `Array[Node]`, then `self.children.push(c)` | Part VIII example | **compiler stack overflow** (D-182) |
| enum `Color.Red` with `match` | `[GRM-24]` | works |

## 3. Plan

Slices are ordered so that each one makes more of the specification's own examples compile, first
the programs of `[TST-8]` (a first week of Python-style code), then the Parts in dependency order.
Every slice: probe → sort → implement → conformance cases (each broken once) → ledgers → all tests
and gates green.

**M0 — foundations**
1. D-182: recursive nominal types overflow the type walks. **Done.**
2. Baseline: the full workspace suite on the branch (green, 12m50s). **Done.** The development
   target manifest now pins 0.9.9_Hardened_3. **Done.**

**M1 — the first program** (Parts II–VI, XV basics)
1. `int`/`float` prelude aliases; untyped literals default to `int`/`float` (`[TYP-1]`, `[LEX-16]`,
   `[LEX-17]`) — migrates every case that relied on the `i32`/`f32` default. **Done** (ODR-022;
   expected types reach generic calls and constructors; `W2015` per `[LEX-17a]`).
2. `//`, floor `//` and `%` on signed integers, integer `/` is `E2240` (`[TYP-28]`), float floor
   operations (`[TYP-29]`). **Done** (ODR-021).
3. String literals initialise `String` (`[TXT-9]`). **Done.**
4. Unparenthesised tuples (`[GRM-29]`). **Done.**
5. Collection literals and comprehensions, `{}`/set literals, field defaults take the field type
   (`[TYP-38]`, `[GRM-26]`, `[GRM-27]`, `[STR-2]`); chained comparisons (`[GRM-25]`) **done**
   (a middle operand that is not re-readable is `E0900` for now: bind it to a local). `Map` and
   `Set` do not exist yet (COLD-START: later-phase work), so their literals wait for them.
6. Scripts: top-level statements in the entry file (`[GRM-2]`, `[FN-8]`). **Done** (a script
   using `?` still needs `AnyError`, so it gets `E2180` for now).
7. f-string specs, `=`, `!r` (`[LEX-19]`); `print`/`println` `sep`/`end` **done**, `Display` →
   `Debug` fallback (`[STD-9]`), float `Display` shortest round trip (`[STD-20]`).
8. `Map` iterates keys, `items()`, `values()` (`[CTL-1]`, `[STD-16]`).
9. Prelude built-ins `len`, `range`, `sum`, `sorted`, `enumerate`, `zip`, `reversed`, `any`, `all`
   (`[STD-26]`); generator expressions (`[GRM-38]`); adapters on `Iterable` (`[STD-19]`).
10. `T` → `Option[T]`, `None` inference (`[TYP-5]` rule 11, `[TYP-23]`); implicit `Ok(())`
    (`[FN-10]`); `Result[T]` default error (`[ERR-1]`).

**Next batch** (from `docs/AUDIT-0.9.9.md`, defects first):
1. ~~`-x as T` binds as `(-x) as T`~~ (done).
2. ~~`==`/`!=` field-wise, text by bytes (D-187)~~ (done; ordering on tuples, arrays and `Option` is not).
3. ~~The ternary `x if c else y`~~ (done).
4. ~~`[TYP-31]`: `len()` is `int`, any integer index, `E2011` for a negative literal index~~ (done).
5. ~~Removed constructs: `::`, `#! language`, `@latebound`, `std.borrow`, `@thread_local`~~ (done).
6. ~~Prelude: `mem`, `panic`, `todo`, `unreachable`; `Option.unwrap`/`unwrap_or`~~ (done, with the
   assertions and `eprint`).

**Next batch:** ~~f-string specs (`[LEX-19]`)~~ (done); ~~float `Display` (`[STD-20]`)~~ (done); the rest of `[ERR-4]`
(~~`is_some`, `Result`'s methods~~ (done); the ones taking a function: `map`, `and_then`, …); ~~`sorted` and `Array.sort`~~ (done); ~~ranges as values~~
(done, ODR-027); generator expressions (`[GRM-38]`) as values, which need `[STD-19]`'s adapters
(comprehensions themselves are done); ~~`input`~~ (done); `Map`/`Set`.

**M2 — classes and exclusivity** (Part VIII): two-phase init (`[CLS-11]`), per-field access words
(`[EXC-19]`), `@sync`, `Weak` upgrade, callable fields.

**M3 onward** — effects and contracts (X), concurrency (XI), data-oriented design (XII), errors
(XIII), compile time (XIV), standard library (XV), C interop (XVI), toolchain (XVII), then the
annexes. Each Part gets its own probe sweep when it becomes current; the sweep's table is added here.

## 4. ODRs raised by this work

| ODR | Question | Ruling | Hardening |
|---|---|---|---|
| ODR-021 | float `//`/`%`: `[TYP-29]`'s formula in IEEE arithmetic, or Python's result? | Python's: exact floor modulo rounded once, consistent quotient | H3 |
| ODR-022 | does a later use change the type of `x = 0` (`[LEX-16]` vs `[TYP-23]`)? | no: the declaration fixes `int`; `E2020` help names the annotation | H3 |
| ODR-023 | which diagnostic reports a value function that can reach its end (`[FN-10]`)? | `E2182`, with the path's last statement labelled | H4 |
| ODR-024 | may a returned view borrow a borrowed parameter that is not itself a view (`[LT-1]` vs `[FN-1]`, M2)? | yes: a borrowed or `mut` non-`Copy` parameter is a source, passed by address | H5 |
| ODR-025 | what do `[ERR-4]`'s function-taking methods accept (`[CLO-7]` named `Option.map` as a stored callback)? | eager; payload moved in (`filter` borrows); `once fn`; a lambda infers `owned`, never `mut` | H6 |
| ODR-026 | is a type with its own `drop` implicitly `Clone` (`[STR-5]`)? | no: a field-wise copy would release twice; `@derive(Clone)` or a written `clone` | H7 |
| ODR-027 | what is a range value (`[CTL-3]`)? | a `Copy` value with public bounds; a `for` counts over a copy of them | H8 |
| ODR-028 | a method named like an inherited one (`[CLS-4]`)? | it replaces it: `E2111` without `override` over a virtual one, `E2110` over a non-virtual one | H9 |
| ODR-029 | what does `parse[T]()` accept (`[TXT-10]`)? | strict (no white space); `ParseError` is `Empty`, `Invalid` or `Overflow` | H10 |
| ODR-030 | how can `Array` have a method named `extend` (`[LEX-15]`)? | `extend` is a contextual keyword | H11 |
| ODR-031 | what do `sort_by`, `sort_by_key`, `windows` and `drain` take and give (`[STD-15]`)? | stable sorts, `f` once per element; shared windows, panic on `0`; `drain(r) -> Array[T]` over any integer range | H12 |
| ODR-032 | what are `Map`'s and `Set`'s parameters and methods (`[STD-11]`, `[STD-12]`, `[STD-16]`, `[ALC-1]`, `[CTL-1]`)? | `Map[K, V, H, A]`; full method lists; `AsKey[K]: Hash` with `is_key` and `to_key`, every `K: Eq + Hash` is `AsKey[K]`; owned iteration yields keys | H13 |
| ODR-033 | what does hashing promise, and when is it `Nondet` (`[HASH-1]`–`[HASH-3]`, `[DET-2]`)? | `Hasher`'s methods, user hashers allowed; `DefaultHasher` fixed per version on every target; making a `RandomState` is `Nondet`; amortised costs | H13 |
| ODR-036 | may a `Map` hold views, as an `Array` of static `str`s may (`[TYP-15]`, `[STD-11]`)? | no: keys, values and elements are not views (`E3063`); `{…}` text with no context is `String` | H13 |
| ODR-035 | how does `Set` have a method named `union` (`[LEX-15]`)? | `union` is contextual: reserved only where an item would begin `union` and a name | H13 |
| ODR-034 | how do `Map`/`Set` print, compare and read as literals (`[TYP-39]`, `[TYP-38]`)? | `{}` and `set()`; Python's `repr` for strings; `==` as sets; a repeated key keeps its first position and last value; `[a, b]` is never a `Set` | H13 |

The next number is ODR-027.

## 5. Progress log

* **2026-09-23 — D-182 fixed.** `TypeTable::is_generic` and `has_unsized_by_value` walked the
  fields of structs, classes and enums with no cycle guard, so any type that reaches itself through
  an owning indirection (`struct Tree: kids: Array[Tree]`, the Part VIII `Node` example) overflowed
  the compiler's stack as soon as a method was called on the `Array`. Both walks now carry a visited
  set of the nominal types being examined, and `Array`/`Box` element drops go through out-of-line
  glue. The specification was right; only the compiler moved.
* **2026-09-23 — M1 slices 1 and 2.** `int`/`float`; literals default to `i64`/`f64`; `//`, `//=`,
  floor `%` (runtime `ember_ck_floordiv_*`/`ember_ck_floorrem_*`, Python-exact float helpers);
  `E2240` with its page; `W2015` for literals given `f32`/`f16`; the overflow panic names `//`.
  Found and fixed on the way: D-183 (nested literal typing, printed `-14` for `-7.5 * 2.0`),
  D-184 (the harness ignored `#$ error[CODE]` without a message), D-185 (calling a captured
  callable). Inference: an untyped literal argument waits for the typed arguments and the expected
  result, in generic calls, generic constructors and `Arena.alloc`. Two rulings, ODR-021/022, cut
  0.9.9_Hardened_3. Corpus migration: 45 files, each through the compiler's own help or by hand
  (annotations on literal locals, `//` for integer `/`, `Array[int]` where a loop counts in
  `int`); `tasks/survey.py` runs `ember check` over the whole corpus in about a minute.
* **Known gap, not yet addressed:** `[TYP-10]` says a shift amount out of range panics in every
  profile; `@overflow(wrap)` still masks it (`tests/run-pass/integer_semantics.em` prints `2` for
  `1 << 33` under it). The rule is clear, so this is compiler work for the `[TYP-8]`/`[TYP-10]`
  slice, not an ODR.
* **2026-09-23 — M1 slices 3–5.** `[TXT-9]`: a string literal where a `String` is expected is a
  one-part f-string, so it allocates through the existing lowering. `[GRM-29]`: `return x, y`,
  `a, b = b, a`, `t = 1, 2`, `one = 5,`. `[GRM-25]`: comparison chains, desugared to `and` in the
  type checker; `E0102` now also covers `is`/`in` in a chain (0.9.9 `[GRM-23]`), and
  `GRM-23/reject_chained_comparison_keeps_its_own_code.em` was retired because the rule it tested
  no longer exists. `E0900` registered (`[CLI-19]`). The directory tests collect every failing
  case before reporting. Drop glue is out of line only for a type that owns itself.
* **2026-09-23 — M1 slices 6 and 7a.** Scripts: the parser gathers file-scope statements into a
  synthesised `fn main()` (`Module::script` records where each statement stood, so `ember fmt`
  reprints the script unchanged and idempotently); the driver rejects them outside the entry file
  (`E0100`), and a declared `main` beside them is `E1030`. `print`/`println` take several arguments,
  `sep=` (a string literal for now, else `E0900`) and `end=`: the checker produces one `print`
  builtin of pieces, and MIR evaluates every argument before printing each piece with its typed
  printer. `return a, b` makes a tuple only outside brackets, where `[LEX-6a]` lets a `,` end a
  colon-bodied lambda.
* **2026-09-23 — M1 slices 7b–9.** `[FN-10]`: `Result[void, E]` gets its `Ok(())` at the end of the
  body, and any other value function that can reach its end is `E2182` (ODR-023, H4); before this
  it fell off its end into undefined C (D-186). `[TYP-38]`: a list literal with no context or an
  `Array` expectation is an `Array[T]`, one allocation of exactly its elements. `[TYP-6]`:
  float-to-integer `as` saturates, `NaN` to `0` (D-188: it was C's undefined conversion).
  `[TYP-8]`: overflow panics in every profile. Pushed to `main` at `d4e81ca`/`3d3966d`.
* **2026-09-23 — precedence and D-187.** The binding powers follow the 0.9.9 Part III table:
  `as` is looser than a prefix `-`, so `-3.5 as u8` is `(-3.5) as u8` (`0`). D-187 fixed: a
  comparison C cannot do is a `ValueCompare` builtin. `str`/`String` take all six comparisons by
  bytes (runtime `ember_str_cmp`; a `String` beside a `str` is borrowed, `[TYP-21]`+`[TYP-5]` rule
  4, so nothing allocates). Structs, tuples, fixed arrays, `Array`s and payload enums whose every
  component has `Eq` take `==`/`!=` through one generated `static bool` function per type, emitted
  to a fixpoint like drop glue. Ordering a struct is `E2040` (`Ord` is never implicit). Known
  gaps: lexicographic `<` on tuples, arrays and `Array`, and `None < Some`, are `E2040` until
  `Ord` is generated; a component with a hand-written `eq` makes the aggregate `E2040` (fails
  closed) until generated equality can call a method; `@no_derive(Eq)` is not parsed yet.
* **2026-09-23 — ternary, `[TYP-31]`, D-189.** `x if c else y` (`[EXP-3]`) is a two-arm `match` on
  the condition, so MIR lowers it as it lowers any `match`; with no expected type an untyped literal
  branch takes the other branch's type, and a `!` branch takes none. `[TYP-31]`: `len()` and
  `capacity()` read the runtime's `usize` and are `int` to the program; an index, a size or a count
  of any integer type is converted with `as`'s wrap, so a negative index fails the bounds check and
  the runtime prints it back as `index -1`. A negative literal index is `E2011` (new page) with the
  fix-its `xs.last()` and `xs[xs.len() - 1]`. Corpus: 17 files moved from `usize` counters to `int`
  (raw-memory `usize` kept, as `[TYP-31]` says), and the std hasher's loop. D-189: parallel
  compilations raced on the `[LT-40]` interface cache; records are now written aside and renamed
  into place. `tasks/impl-0.9.9/survey.py` now prints the exit status and output of a rejection
  that reported no `error[...]`, which is how D-189 was found.
* **2026-09-23 — removed constructs.** `[LEX-21]`/`[GRM-24]`: a `::` is `E0100 `::` is not a path
  separator` with the help `use '.' for paths` (the parser still reads it as a path, so the rest
  resolves); `.` now reaches a module through another's namespace (`support.io.print`), a type
  and a variant through a module (`inner.Shape.Square(2)`, in patterns too), and a misspelt item
  after a `.` path gets the same N1 suggestions the `::` form had, printed with `.`. Name
  suggestions sort by `.`-qualified name (`[DIA-24]`). The cycle tool's target is
  `<Class[.field]>` with `.` only, resolved left to right; a `::` target is refused. `[VER-8]`:
  the only accepted directive is `#! language "0.9.9"`; any other is `E0006` whose help is to
  delete the line; the std modules' directives are deleted. `@latebound` is `E0104`: probing
  showed an ordinary callable type already behaves as `[LT-7]` says, so the modifier had nothing
  left to do. `std.borrow` (`with_views*`) is removed. Corpus: `DIA-12` and two UI tests move to
  `.` paths (`ioo.print(1)` now reads as an unknown name `ioo`, so its message is `cannot find
  `ioo``); `FN-6b` and `LT-10`'s `@latebound` tests are re-homed where they still mean something
  (`LT-7/accept_a_callable_type_needs_no_modifier`, `ATT-1/reject_the_removed_latebound_modifier`,
  `TYP-15/reject_a_callback_view_result_stored_in_a_box`, and `LT-10`'s nested escape, which
  0.9.9 accepts, as `LT-7/accept_a_callback_result_borrows_the_callers_view`); the 18 tests of
  `with_views*` itself are retired with it (their generic-inference case is covered by three
  `TYP-18` tests), as is `LT-10/reject_latebound_owned_capture_publication` (the question it now
  raises is in the audit). New: `LEX-21`, `GRM-24`, `VER-8` and two `LT-7` cases (its example,
  and a callee's local escaping through a callback). Found: D-190 (open).
  `tasks/impl-0.9.9/annotations.py` checks a directory's `help`/`error` annotations in seconds.
* **2026-09-23 — attributes (`[ATT-1]`, `[ATT-6]`).** No pass had checked attribute names: an
  unknown one (`@thread_local`, `@frobnicate`) was accepted silently, and `@export("symbol")` in
  `[FFI-26]`'s own test was accepted and ignored (the symbol stayed `on_update`). Now every
  attribute on an item, member or variant is checked against the Part V table: an unknown or
  removed one is `E0104`, a reserved one is `E0104 reserved for a later version`, one on a
  declaration it does not apply to is `E0104` naming where it does, and one whose effect is not
  built is `E0900`. Built: `@derive(Copy, Clone)` (and `Eq`, which is implicit), `@repr` on
  enums, `@layout(c)` (the C layout the backend always emits, `[LAY-2]`), `@view`
  (documentation), `@static_safe`, `@overflow`, `@borrows`. Statement attributes keep the parser's
  name and place checks (`[ATT-2]`, `[ATT-3]`); the checker rejects all four with `E0900`, since
  none is built, so `--syntax-only` still passes valid syntax. `FFI-26`'s test drops the ignored
  `@export`. `spec_check`'s baseline (it checks the adopted 0.8.5 document's examples): Appendix
  A's block is newly failing by design (`#! language "0.5"` is `E0006` under `[VER-8]`), and five
  blocks that failed now parse.
* **2026-09-23 — the prelude's panics and assertions (`[MOD-5]`, VI.6).** `panic(msg)`, `todo()`,
  `unreachable()` have type `Never` and lower to a MIR assertion that cannot hold
  (`AssertKind::Panic`, whose message is an operand every analysis visits beside the
  `RefCellBorrow` operands), then a block nothing reaches, as after a `return`. `assert(cond)`,
  `assert(cond, msg)`, `assert_eq`, `assert_ne` are the same assertion on a condition, in every
  profile; `debug_assert` is checked in `debug` only (`[PRF-3]`), and elsewhere its arguments are
  type-checked and not evaluated (`W2016`, which needs effects, is not built). A function of the
  same name shadows each (`[MOD-5]`). `eprint`/`eprintln` share `print`'s pieces; the runtime has
  one printer per type taking its stream. `mem` is a prelude namespace (`std.mem` is always
  loaded). `Option.unwrap`, `unwrap_or`, `expect` (`[ERR-4]`) are a `match` on the value, or on the
  pair of value and argument, so the argument is evaluated first, as any argument is. Found and
  fixed: D-191 (`println` of one `String` emitted invalid C). `annotations.py` now also checks
  `stdout` and `panics` for run tests, so a directory's runtime behaviour is checked in seconds.
* **2026-09-24 — `[STD-26]`, block expressions, D-190, D-192, D-193.** HIR gained
  `ExprKind::Block { block, value }` (its statements, then its value, then its drops), which the
  checker uses to evaluate each argument once and to put a loop inside an expression. `len` is
  `x.len()` for a collection or view, and `E2073` for a string. `min`, `max`, `clamp` compare with
  a `TotalLess` builtin (IEEE totalOrder for floats, `[TYP-37]`; Python's rule that the first of
  two equal arguments wins); `clamp` panics when `lo > hi` (`[ERR-13]`); `abs` is `0 - x` below
  zero for integers (so the minimum panics) and `fabs` for floats. `sum`, `any`, `all` are a
  counted loop over a view of the collection. In a `for` header, `range(n)` and `range(a, b)` are
  `a..b`, `range(a, b, step)` is a counted loop over the number of values (runtime
  `ember_range_count_*`, each value computed from its index so nothing overflows; a zero step is
  an assertion); `enumerate`, `zip` and `reversed` are one counted loop over views. A shared
  `Span` and a fixed array now iterate. Defects: D-190 (a returned view of a local reported
  `E3021` beside the right `E3060`; the drop at a `return` is left to the escape check), D-192
  (`-i64.MIN` did not panic), D-193 (`for x in [1, 2]` was rejected). The conformance harness does
  not fail on unexpected diagnostics (`[TST-1]` says it must); recorded in the audit, with a UI test
  pinning D-190's exact code list meanwhile.  `[TXT-10]`: `str` had no methods; `len()` (bytes), `char_count()` (characters) and `is_empty()`
  are built for `str`, and for `String` through its `str` (`E2073`'s page needed them).
* **2026-09-24 — float `Display` (`[STD-20]`).** A float prints as Python's `repr`: the fewest
  digits that read back as the same value (17 at most for `f64`, 9 for `f32`), written in fixed
  notation when the decimal exponent is in [-4, 16) with `.0` on an integral value, and in exponent
  notation otherwise (`1e+300`, `1e-05`); `nan`, `inf`, `-inf` whatever the C library spells. It
  serves `print` and f-strings alike. 13 corpus files printed integral floats and were moved to the
  new text by a script that changes a line only when every differing token is the same number
  (`9` to `9.0`); three that print several values with no separator were checked by hand.
  `annotations.py` now matches the harness: an error's text may be anywhere in the diagnostic, a
  help's anywhere in a help line, and an empty `#$ stdout:` opens a block.
* **2026-09-24 — f-string specs (`[LEX-19]`), D-194.** The lexer recognises Python's `{x=}` (the
  source text through the `=` is written first, and the value as `Debug` unless a spec or
  conversion is given) and `{x!r}` (`Debug`); `!s` is accepted and anything else is `E0100`. The
  checker parses the spec (`[[fill]align][sign][#][0][width][,|_][.precision][type]`; a malformed one
  is `E0100`) into `hir::FormatSpec` and checks it against the value's type, `E2250` (new page)
  naming both: integers take the integer and float kinds, floats the float kinds, and `str`,
  `String`, `bool`, `char` `s` and a truncating precision. The C backend passes the spec as a
  struct literal to one runtime formatter per type, which follows CPython: fill and alignment
  (numbers right, text left, `^` with the odd fill on the right), sign, `#` prefixes, `0` padding
  that is grouped with the digits (`{n:012,}` is `0,001,234,567`), `,`/`_` grouping, `%`,
  a precision with no type as CPython's `g` with at least one digit after the point
  (`{100.0:.3}` is `1e+02`), and `repr` quoting for `?`. Every probe was compared with Python's
  own output. D-194: an f-string moved a `String` it wrote, and then emitted invalid C for it.
  Still open in `[LEX-19]`: `Debug` for aggregates and classes (`[TYP-39]`), and Python's escaping
  of non-printable Unicode in `repr` (the runtime keeps it as is).
* **2026-09-24 — comprehensions (`[GRM-27]`, `[GRM-38]`).** The parser reads `[e for …]`, `(e for …)`
  and a generator as the only argument of a call (`sum(x * x for x in xs)`; a bare generator
  followed by another argument is `E0100`, as in Python), with `for target in or_expr` and
  `if or_expr` clauses sharing the `for` statement's target parser. The checker lowers a
  comprehension to a block expression holding its loop nest: each `for` clause a counted loop over
  a view of a collection (elements borrowed, `[CTL-1]`) or over a range (`a..b`, `range(n)`,
  `range(a, b)`), each `if` a test, left to right, the loop variables in scopes of their own. The
  innermost step pushes onto a new `Array`, adds (`sum`, with `start`), or tests and leaves the
  whole nest (`any`, `all`). The accumulator's type is fixed when the element is typed at the
  innermost level. Not yet: a generator stored or passed elsewhere, a stepped `range` in a
  clause, `{…}` comprehensions (they need `Map`/`Set`). `tests/milestones` had one more
  integral float (`5` is now `5.0`).
* **2026-09-24 — `Array` methods (`[STD-15]`), membership (`[STD-8]`), D-195.** Reading methods are
  checker desugars: `is_empty`, `contains`/`index_of` (a linear scan through the same equality
  `==` uses), and `get`/`first`/`last` through `Span.get`, so an empty array's `last()` is `None`.
  Methods that move elements call per-element-type helpers the C backend generates and emits
  like the equality functions (`ArrayHelper`: a sort's comparison, `pop`, `remove`, `clear` with
  each element's drop, `sorted`), and the runtime gained a stable bottom-up merge sort, `reverse`
  and `insert`. `sort` orders scalars (floats by totalOrder) and text; `sorted` copies, so it
  needs `Copy` elements until `Clone` is built. `remove`/`insert` bind their arguments before the
  mutable borrow (`xs.insert(len(xs), v)`) and assert the index (`[ERR-13]`). `x in c` and
  `x not in c` scan collections, test text for a `str` or `char` needle (`[STD-8b]`), and compare
  twice for a range written in place; anything else is `E2226`. `sorted(xs)` from `[STD-26]`.
  D-195: `String +=` emitted invalid C, and `String + str` was rejected. The runtime header now
  includes `<string.h>` for the generated helpers.
* **2026-09-24 — `Option`/`Result` methods (`[ERR-4]`), D-196.** `is_some`/`is_none`,
  `is_ok`/`is_err`, `unwrap`, `expect`, `unwrap_or`, `unwrap_or_default`, `ok()`, `err()` and
  `ok_or(e)` are checker desugars into a `match` on the value, so they need no library code. A
  failed `unwrap` or `expect` panics with the error's `Display` text, after `expect`'s own message
  (`reading the answer: `x` is not a number`); an error type with no text gives a fixed message.
  `unwrap_or` and `ok_or` evaluate their argument before the match (a tuple scrutinee), as a call
  would. The methods that take a function wait for closures to be callable from a desugar.
  D-196: a temporary made in one arm of a `match` or conditional (the `str` view of an f-string,
  say) was reported as `E3050` at its end-of-statement drop; a drop is no longer a read.
* **2026-09-24 — `T` to `Option[T]` (`[TYP-5]` rule 11), defaults (`[FN-5]`), open `None`/`[]`
  (`[TYP-23]`), D-197.** Rule 11 is the last step of `coerce`: after the type itself, a literal
  that fits, widening, range erasure and the text rules, a non-`Option` value becomes
  `Some(value)`; an `Option` is never wrapped again. Parameter defaults were parsed and silently
  dropped; now each is checked once where it is declared, with no locals in scope, and a direct
  call that leaves a parameter out checks the default again at the call, silently (a `Sink`
  mark and rollback), and evaluates it after the written arguments. A default that reads an
  earlier parameter, and one on a generic function, are `E0900`. `x = None` and `xs = []`
  declare an open local; the first assignment, `push`/`insert`, or coercion site expecting a
  type fixes it, and the body is checked again with the type written in (the first pass's
  diagnostics and per-body state are undone). Passes repeat while each fixes another local; one
  still open is `E2060` at its first use. D-197: `println` of one value skipped the printer check
  and sent an `Array` to C as a string. Messages now show `Option[i64]` and `Result[i64, str]`
  rather than the per-payload names the compiler builds.
* **2026-09-24 — `print(xs)` shows a list as Python does (`[TYP-39]`).** The C backend generates
  one `Display` function per printed or formatted aggregate (an `Array` that is not a `String`, a
  view, a fixed array, a tuple, `Option`, `Result`), requested and emitted like the equality
  functions, plus a wrapper for the ones printed that formats into a buffer and writes it to
  stdout or stderr. Elements show by their `Debug`: numbers and `bool` as they display, text and
  `char` quoted by the runtime's `repr` (new `fmt_repr_str`/`fmt_repr_char`), so `['a', "it's"]`,
  `(7,)`, `Some('hi')`, `Err('bad')`. A spec on an aggregate is `E2250` (Python refuses one too);
  `!r` and `=` give the same text. Printing a class handle is `E2040` (`[TYP-36]`: it has `Debug`,
  not `Display`). Found on the way: D-198 (`Array[str]` rejected at formation), slicing
  `xs[a..b]` still not built (already listed), and `xs[a..]` does not parse.
* **2026-09-24 — slicing (Part VI's slice row, `[TXT-4]`, `[SPN-2]`).** `a..` parses with no upper
  bound. The checker turns `a[range]` into a `Slice` builtin over a view (an `Array` or fixed array
  through the existing `Span` coercion, a `String` through `as_str`); MIR evaluates the view and
  both bounds once, asserts `lo <= hi <= len` with the ordinary bounds panic, asserts that a text
  slice's bounds start characters (`ember_str_is_char_boundary`), then takes the sub-view. The
  borrow checker treats the result as borrowing its view argument, as `split_at`'s halves do, so an
  `Array` cannot grow while a slice of it is used. Open question found on the way (ODR-024
  candidate): `[LT-1]` lets a returned view borrow from "reference or view" parameters; a borrowed
  `Array`/`String` parameter is neither by type, so `fn head(xs: Array[int]) -> Span[int]:
  return xs[..2]` (and `return xs` alone) is `E3060` today.
* **2026-09-24 — review of printing and slicing (workflow, 9 agents: 3 reviewers, 6 skeptics).**
  Six findings confirmed, nine more checked by hand. Fixed before commit: a slice read its view
  twice — the length before the bounds, the pointer after — so a bound that changed the view
  read past its buffer (now one snapshot, `[EXP-1]`); `{xs!r:>10}` pads the conversion's text as
  in Python (a bare `{xs:>10}` stays `E2250`); a class handle prints its `Debug`, `<Token at
  0x…>`, with the object's own class (`[TYP-36]`'s table, `[STD-9]`'s fallback), rather than
  `E2040`; the generated formatting functions take a pointer, so a large fixed array no longer
  overflows the stack; a type holding an error no longer adds a second error; `xs[a:b]` gets
  `[DIA-21]`'s fix-it; a `for` over a slice of a temporary keeps the temporary (`[EXP-4]`).
  Older defects fixed: D-199, D-200, D-203, D-204. Recorded open: D-201 (`String` is
  `Array[u8]`), D-202 (views of class fields start no access), `[MNG-1]` (mangling not injective,
  which also lets `fn fmt_0` collide with a generated helper). One finding was wrong:
  `println(())` printing `()` is `[TYP-36]`'s `Debug` of `void` through `[STD-9]`'s fallback.
* **2026-09-24 — `[TST-1]` in the harness (D-205).** The conformance harness now fails on
  unexpected diagnostics as well as missing ones, reading `--json` so a source excerpt cannot
  satisfy an annotation, and tying a trailing annotation to its line. Migrating the corpus (72
  files) found real defects: `L3011` across `print` (D-206), `E3041` at the wrong line (D-207),
  and eleven cascades (D-208), all fixed; and stale tests written before 0.9.9's `/` and `int`
  changes. Every rule directory and suite now passes with exact diagnostics.
* **2026-09-24 — ODR-024 ruled (panel of 4 agents).** A borrowed or `mut` parameter whose type
  is not `Copy` is a source of a returned view (`[LT-1]` rules 2–3) and is passed by address;
  rule 1 keeps any borrowed receiver. It also fixes by-copy defects the panel reproduced: `Cell`
  writes lost through a borrowed parameter, and `RefCell` through one corrupting the heap. To be
  implemented next, with Hardened_5.
* **2026-09-24 — ODR-024 built; Hardened_5 cut.** A borrowed parameter whose type is not `Copy`
  (or holds a `Cell`) is passed by address as the caller's place, in every kind of call; views
  pass as themselves. The source set for `[LT-1]` rules 2–3 is fixed by the declared signature,
  with type parameters counted as `Copy`; `@borrows` may name a source or `mut` parameter, and
  `E2031` covers a borrowed `Copy` or an `owned` non-view one. `E3060` for a view of a temporary
  argument names the temporary. Fixed D-209–D-215; recorded open D-216 (`[TYP-5]` rule 7),
  D-217 (a reference to a view parameter's own slot escapes) and D-218 (class getters need
  `[EXC-18]`). `0.9.9_Hardened_5` carries the amended `[LT-1]`, `[LT-1a]`, `[LT-1b]`, `[LT-7]`,
  `[LT-44]`, `[FN-1]`, `[FN-3]`, `[FN-6]`, `[BRW-8]` and `[CORO-6]`, and is the development target.
* **2026-09-24 — D-217 fixed.** A reference to a view parameter's own slot (`return ref x` for
  `x: str`) escaped: the escape check exempted every loan rooted at a view parameter. It now
  exempts only loans through the view, and a loan taken only to feed a view reborrow (a `mut
  self` of `MutSpan`) no longer carries the slot's storage obligation, so `span.iter_mut()`'s
  items still outlive the slot. `E3060` stands aside where `E3062` already names the parameter.
* **2026-09-24 — `[TYP-5]` rule 7 built (D-216).** A place of type `T` auto-borrows where a
  shared `ref T` is expected: arguments, initialisers, returns, view-struct fields, and generic
  `ref T` parameters, whose `T` is now inferred through the borrow (before this, even `same(r)`
  with `r: ref int` could not infer `T`). A temporary or a `ref mut` site stays `E2020`, with a
  help.
* **2026-09-24 — `L3014` counts source parameters (`[LT-1b]`).** The opt-in lint counted
  view-typed parameters, so a function over two `Array`s never got it after ODR-024; it now
  counts the source set, and covers methods whose receiver is not borrowed.
* **2026-09-24 — `[LT-7]` calls through a callable value (D-219).** The result of `g(a, b)`
  borrows the callable type's source parameters, not every argument, so a `mut int` passed
  through a function value is free again once the call returns. A function whose `@borrows`
  names a non-source parameter is refused as a value (`E2020`), since the callable type cannot
  say so.
* **2026-09-24 — review of the D-216/D-217/D-219/`L3014` batch (workflow, 6 agents, 2 at a time:
  3 reviewers, then 3 skeptics).** All 17 findings confirmed, plus six new items. Fixed before
  commit: two soundness holes older than the batch, a reference into a view parameter's own
  `Array`/`Box` (D-224) and two-phase activation (D-223, which also makes `[BRW-3]`'s accepted
  `f(v, v[0])` compile); D-217's first version regressed `split_at` and the drop of an `owned`
  view struct (now any span built-in hands out the target, and a drop conflicts only with loans
  on its storage), gave a slot reference `E3062` with a help that led to `E3060` (the slot is
  now `E3060`, with a help that follows the result type), and still charged views held in
  locals to the local (fixed); auto-borrow now works for `mut` parameters and `mut self`; helps
  that over-promised were narrowed; `L3014` is reported once for an interface default; an
  `owned self` view is under rule 3 (D-225); two duplicate diagnostics (D-226). Recorded open:
  D-220 (a capturing closure through a callable parameter, over-strict), D-221 (`f[int]` as a
  value), D-222 (callable-field calls), D-227 (printing `Option[ref T]`).
* **2026-09-24 — ODR-025 and `[ERR-4]`'s methods that take a function; Hardened_6 cut.** `map`,
  `map_err`, `and_then`, `or_else`, `unwrap_or_else`, `ok_or_else` and `filter` are `std.core`
  generics the type checker routes to, so a lambda argument is inferred and borrow-checked as any
  argument is. A lambda's unannotated parameter takes `owned` from the expected type. Fixed on the
  way: generic inference through `Option`/`Result` (D-229), an enum-nested generic result on a
  lambda, and a ternary's `None` branch (D-230). Recorded: DEVIATIONS D6 (`fn` where the ruling says
  `once fn`, until `once fn` is built), D-228 (`Clone` for `Array`/`String`, next).
* **2026-09-24 — `Clone` for `Array` and `String` (D-228).** `xs.clone()` copies the buffer and
  clones each element (nested arrays, strings, structs through their `clone`, class handles by
  retaining); derived `Clone` accepts such fields. The `E3040`/`E3030` helps that say to clone now
  work for lists and text. Testing it found the nested drop-glue loops sharing one index (D-231),
  which crashed any `Array` of structs owning an `Array` of strings at scope end; fixed.
* **2026-09-24 — `[STR-5]` implicit `Clone`; ODR-026; Hardened_7 cut.** Every struct and enum whose
  fields are all cloneable is `Clone` with nothing written, generic instances included; a written
  `clone` replaces it and `@no_derive(Clone)` opts out. ODR-026 keeps it off a type that declares
  `drop` (a field-wise copy of a pointer it frees is a double free reachable from Safe code); a
  missing `clone` there says so. `@no_derive(Eq)`/`(Debug)` stay `E0900`.  An implicit `clone` nothing calls is pruned after lowering (`[COST-1]`), so a program pays for
  one only where it clones (`STR-5/accept_an_unused_implicit_clone_is_not_emitted.em`).
* **2026-09-24 — `[STR-5]` implicit `Debug`.** Structs and enums print with nothing written, in
  `[TYP-36]`'s forms: `Point(x=1, y=2.5)`, `Shape.Circle(1.5)`, `Mode.Fast`; a struct or payload
  enum displays as its `Debug`, a unit-only enum as its variant name (`{m!r}` for the `Debug`). A
  generic struct shows its name alone; recursive types print to any depth. The compiler-known
  wrappers (`Box`, `Cell`, …) still have no format, and `@no_derive(Debug)` stays `E0900`.
* **2026-09-24 — `input` (`[STD-10]`).** `input(prompt="")` prints the prompt, flushes, and returns
  one line of standard input without its `
` or `
`; end of input panics at the call, naming
  `std.io.stdin().read_line()`. The harnesses gained `#$ stdin:` (one input line each); a run test
  without it gets a closed standard input, as before.
* **2026-09-24 — `@no_derive(Eq)`/`(Debug)`; printing through references (D-227).** The three
  implicit interfaces can each be opted out of; comparing or printing an opted-out type (or a
  value containing one) is `E2040`, naming the type and the attribute. A `ref T` inside a printed
  value prints what it points to (`Array.get`'s `Option[ref T]`). Recorded open: D-232 (internal
  names of generic instances in diagnostics).
* **2026-09-24 — ranges as values (`[CTL-3]`, `[STD-8]`, `[STD-26]`); ODR-027; Hardened_8 cut;
  D-232, D-233.** `a..b`, `a..=b`, `a..` and `..b` are values of the prelude's `Range`,
  `RangeInclusive`, `RangeFrom` and `RangeTo`, which `std/src/core.em` declares as
  `@derive(Copy)` structs with public `start`/`end`; ODR-027 rules them Python-style values (a
  `for` counts over a copy of the bounds, so a range can be iterated again). An expected range
  type gives the bound's type. A `for` over a range value, over `a..` (a `while` whose counter
  moves before the body, so counting to the maximum is `[TYP-8]`'s overflow) and over
  `range(n)`/`range(a, b)` values is a counted loop. Range values have `x in r`/`r.contains(x)`
  and `len(r)`/`r.len()` (`RangeCount`, then a check that the count fits an `int`). Not built:
  a stepped `range` as a value, `len` over `i128`/`u128`, and a range's `Iterator` methods
  (`[STD-19]`). D-232: diagnostics spell a generic instance as written (`Range[u8]`, `Cell[i32]`)
  while every symbol keeps the instance name (`symbol_name`); eight tests had pinned the old
  names. D-233: a help with 18 stray spaces. `annotations.py` now also flags a conformance
  directory with no `accept_*` case (`[TST-4a]`): the cargo harness stops at the first one, so
  every directory after it goes unchecked.
* **2026-09-24 — callable fields (`[CLO-11]`, D-222); D-234, D-236.** `h.f(x)` calls a field of
  callable type when there is no method `f` (a method takes precedence); any other callee that is a
  value is called by its type: `(h.f)(x)`, `pick(b)(x)`, `fs[1](x)`, a lambda written in place.
  Calling a local that is not callable says so (D-234). A generic struct's constructor maps
  arguments to fields by name and checks each once (D-236: named arguments in another order were
  rejected, errors were reported twice, and an inline lambda became two closures), and a generic
  class infers from `init`'s parameters (D-237). Recorded open: D-235 (a class `init` with a
  callable parameter cannot be constructed).
* **2026-09-24 — an instantiation is a value (`[TYP-18]`, D-221).** `id[int]` is the function
  value of that instantiation, passed, stored and called like any function (`[FN-6]`), with D-219's
  `@borrows` guard. It must give every type parameter, a callable parameter's implicit one
  included (`E2060` otherwise, with the count).
* **2026-09-24 — a constructor takes a callable (D-235).** An `init` whose parameter is `fn(...)`
  is generic (`[CLO-3]`), and constructing the class now instantiates it for the argument, as a
  generic method call does. Storing a callable parameter in a field needs `[CLO-3]`'s owned
  callable value, which is not built (AUDIT).
* **2026-09-24 — powers, inherited constructors, text iteration and three smaller gaps.**
  `[TYP-30]`: `a ** b` on integers is exact by square and multiply through the checked `*`, so it
  panics exactly when the true power overflows; a literal negative exponent is `E2151` and a
  computed one panics; a float base takes a float or integer exponent through C's `pow`
  (`FloatPow`); `a **= b` too. `[CLS-10]`: a derived class with no `init` runs the nearest base
  `init` on the new object after its own field defaults (MIR upcasts the receiver as `super.init`
  does); a derived field with no default is `E2101` (new, with an error page) at the declaration,
  and three tests that declared one were given defaults. `[CTL-1]`: `for c in s` over a `str` or
  `String` yields `char`s (`StrCharAt`, `CharUtf8Len`, runtime `str_char_at`/`char_utf8_len`).
  `[TYP-14]`: an operand reads through a `Box`. `[DIA-20]`: a run of invalid characters is one
  `E0100`, and the parser reports nothing at a token the lexer rejected (`[DIA-14]`).
* **2026-09-24 — struct field defaults are evaluated (D-238).** Found probing `[STR-2]`: a zero
  stood in for every struct field default, and an omitted `String` or `Array` field's `0` was spread
  by C's brace elision over the fields after it. Each default is now checked at the construction
  that omits it, in field order, with its names resolved in the declaring module; a mistake in one
  is reported once. The fixture of a UI test (`N1/inherited_class_field_typo`) also gave its
  derived field a default, as `[CLS-10]` requires.
* **2026-09-24 — visibility (`E1052`), overrides (ODR-028, Hardened_9), base memberwise
  constructors.** Probing `[MOD-2]`: a module's private function could be called through the module
  (`tools.helper()`); it is now `E1052` (new, with a page), naming where the item is declared, and
  every privacy error (a private import, field or memberwise constructor) uses `E1052`, which 0.9.9
  gives them (`E1020` is only a name declared twice); six tests were moved to the new code.
  Probing `[CLS-4]`: a method named like an inherited one was checked only when it said `override`.
  ODR-028 rules it: `E2111` (new) for a missing `override` over a virtual method, `E2110` over a
  non-virtual one; an `override` is virtual in turn (D-239: a grandchild's `override` was refused).
  `[CLS-10]`: with no `init` in the chain, a derived class gets its base's memberwise constructor
  (inherited fields by position or name, its own fields by default).
* **2026-09-24 — three diagnostics from the audit probes.** `[GRM-19]`: `if x = 5:` offers
  `did you mean `x == 5`?` first, as a fix-it, with the declaration as a note. `[TYP-4]`: the
  numeric-mismatch note appears only for two numbers and names the narrower operand's cast
  (`a as i64`; an integer becomes a float). A value is no longer offered an interface's name as a
  suggestion (`did you mean `Eq`?` for `y`); the `[DIA-12]` prelude-candidate test now uses `Range`.
  Recorded gaps from the same probes: `[CTL-10]` hoisting, a jump as a conditional branch
  (`[GRM-16]`), `I.m(recv)` (`[TYP-24]`), qualified type paths (`m.T`).
  D-240: a name declared twice in one block (`z: int = …` twice) compiled; it is `E1020` now, which
  also gives `E1020` the conformance test the registry check asks for.
* **2026-09-25 — names assigned in every branch (`[CTL-10]`).** In an `if` with an `else`
  (through any `elif` chain) and an exhaustive statement `match`, a name that is not in scope
  before and that every arm completing normally declares by `x = e`, at one type, is declared
  after the statement: the checker declares it beside the statement uninitialised and sets it at
  the end of each such arm (`hoist_branch_names`; a nested branch hoists into its arm first, and
  the compiler's own hidden names never hoist). Different types are `E2230` (new, with a page),
  and the name is declared anyway so later uses add nothing. `str.to_string()`, which `[TXT-9]`
  names, is not built (found while probing).
* **2026-09-25 — `to_string`, `String.from` and the first `str` methods (`[TXT-9]`, `[TXT-10]`).**
  `x.to_string()` on any value with text is `f"{x}"` (one f-string part, so `Display` else
  `Debug`); `String.from(s)` copies a `str`. On `str` (and `String` through it): `starts_with`,
  `ends_with`, `find`/`rfind` (a byte offset in an `Option[int]`), `count` and `replace` (Python's
  meaning, an empty needle matching between characters), `repeat` (empty below one), and `trim`,
  `trim_start`, `trim_end` (Unicode white space; a view of the text, through the `Slice` builtin, so
  the borrow is the ordinary one). Runtime `str_starts_with` … `str_trim_end`. Not built yet:
  `to_upper`/`to_lower` (Unicode case tables), `split`, `lines`, `chars` and the other methods that
  return iterators, `parse`, `get(range)`.
* **2026-09-25 — a jump as a conditional branch (`[GRM-16]`).** `v = x if ok else return 7` (and
  `continue`, `break`): the parser reads a jump as a whole `else` branch, and the checker makes it
  that arm of the `match` the ternary lowers to, as a `match` arm's jump is. `annotations.py` now
  trims `#$` continuation lines as the harness does.
* **2026-09-25 — `I.m(recv)` (`[TYP-24]`), D-241.** `I.m(x, …)` with `I` an interface calls `I`'s
  `m` on `x` (the ordinary method-call path, told the interface by the call's span, so a call inside
  the arguments is not affected); a type without it is `E2040`. D-241: two interfaces offering one
  method name for a type shared one implementation; each interface's methods are now kept apart
  (`interface_methods`) for body checking, `dyn` adapters, implementation checks and C symbols.
* **2026-09-25 — `E0008` and standard modules without `std.`.** `[LEX-22]`: an unterminated
  character literal is `E0008` (new, with a page); `'a` is no longer a "named lifetime" (`E0007`,
  retired in 0.9.9) but that literal, with a note naming `[LT-6]`'s restructurings; the `[LT-6]`
  test was moved to it. `[MOD-3]`: `import math` and `from mem import swap` find `std.math` and
  `std.mem` when the package has no module of the name (the driver's loader and the checker's
  import binding).
* **2026-09-25 — `get(range)`, `to_upper`, `to_lower` (`[TXT-10]`).** `s.get(range) ->
  Option[str]` for any prelude range: `None` for a range out of order, past the text or off a
  character boundary, never a panic (an inclusive end is checked before `+ 1`). `to_upper` and
  `to_lower` use Unicode 16.0 full case mappings (runtime tables generated by the new
  `tools/unicode_case.py` from Python's own mappings, with `--check`), and a word-final `Σ` lowers
  to `ς` (Python also skips case-ignorable characters around it; this looks only at the neighbours).
  `parse` waits for ODR-029 (`ParseError` is named by the spec but not defined).
* **2026-09-25 — `parse[T]()` (ODR-029, Hardened_10).** Strict, as Rust's: the whole text, no
  white space; integers (sign and digits, fitting `T`), floats (with `inf`/`nan` and exponents),
  `bool`, `char`. `ParseError` (`Empty`, `Invalid`, `Overflow`) is in the new `std/src/string.em`,
  loaded with the prelude modules. The runtime validates (`ember_parse_*_status`) and reads the value
  only once the text is known good. Not built: 128-bit integers.
* **2026-09-25 — `partition`, `split_once`, `as_bytes`, `is_char_boundary` (`[TXT-10]`).**
  `partition(sep)` is Python's (`(s, "", "")` when absent; an empty separator panics);
  `split_once(sep) -> Option[(str, str)]`; both give views of the text through `Slice`.
  `as_bytes()` reads the text as its `Span[u8]` (a new `StrAsBytes`, listed with `Slice` among the
  builtins whose result borrows their argument, so the bytes of a temporary cannot outlive it);
  `is_char_boundary(i)`. The splitting methods that return iterators wait for the iteration design.
* **2026-09-25 — `extend` is contextual (ODR-030, Hardened_11).** `[STD-15]` names an `Array`
  method `extend` and the keyword table reserved the word, so `xs.extend(ys)` could not parse.
  Under the owner's rule for such collisions `extend` is now a contextual keyword: it begins an item
  only before the type it extends (after any generic parameters), and is a name everywhere else.
  The keyword table has 48 words.
* **2026-09-25 — generic extensions of every generic type (D-242, D-244).** `extend[T]` worked only
  on generic classes. Structs, enums and classes now share one path, and an instance made before
  the extensions were read gets them afterwards. `Array`, `Span`, `Option` and `Result` take their
  extensions when a method is first looked up on an instance (they have no instantiation step), and
  `implements` reads the recipes. `Array`/`Span` try the compiler's methods first (`[TYP-24]`). An
  interface method arriving after an inherent one of its name is kept for its interface (D-244).
  Open: D-243 (`E2120` is never reported) and D-245 (in a generic body, a bound's method can reach
  an inherent method of the same name once the body is instantiated).
* **2026-09-25 — bounds met by the compiler (D-246), and `Array` methods (`[STD-15]`).** `T: Ord`,
  `T: Clone` and `T: Eq` accept what the compiler provides (numbers and text, `Copy` and cloneable
  types, implicit `Eq`); `x.clone()` of a `Copy` value, `a.cmp(b)` on numbers and text, and
  `Ordering` in the prelude; `min`/`max`/`clamp` take `str`. `Array` gains `capacity`, `reserve`,
  `truncate`, `swap_remove`, `swap`, `extend` (a span's elements, cloned), and `get_mut`,
  `split_at`, `iter`, `iter_mut`, `chunks`, `chunks_mut` through a view of the array. `retain`,
  `dedup` and `binary_search` are Ember, `extend[T] Array[T]:` blocks in `std/src/core.em`, the first
  standard-library methods written that way; one a program does not call is not emitted. D-247 (the
  interface cache recorded a generic extension's specializations) fixed on the way. Not built:
  `sort_by`, `sort_by_key` (a comparator inside the runtime's sort), `windows`, `drain`, `join`.
* **2026-09-25 — `E2120` (D-243) and bound calls (D-245).** An `extend` with no `implements` for
  another package's type is `E2120` (`[IFC-2]`): the language's and the standard library's types
  take new methods through an interface of one's own (`[TYP-20]`) or a wrapper. A call a generic
  function makes through a bound reaches that interface's method in every instantiation, even where
  the type has an inherent method of the name. Found on the way, open: D-248, a generic type's
  methods are checked only per instance and so are duck-typed.
* **2026-09-25 — `m.T`, `Array.join`, and `Display`/`Debug`/`Copy` bounds (D-249).** A type named
  through its module works in type position (`geometry.Pair[int]`), and a struct or class is
  constructed through it (`geometry.Point(3, 4)`), with `[MOD-2]`'s visibility. `xs.join(sep)` is
  Ember in `std.core`, bounded by `Display`. Bounds on the prelude's `Display`, `Debug` and `Copy`,
  which the standard library does not declare yet, are answered from `[TYP-36]`'s table instead of
  accepting every type, and a generic body formats a parameter so bounded. A built-in instance
  takes every matching extension at its first method call; an inherent method's body is now checked
  only when it is called, so an unused `join` does not check against an element type it cannot
  format.
* **2026-09-25 — no duck typing in generic types (D-248, D-250).** Each generic struct's, enum's
  and class's methods, and each generic extension's, are checked once against the type over its own
  opaque parameters, with their bounds in scope, before any concrete instance (`[TYP-17]`); a call
  made through a bound reaches that interface's method in every instance. An instance over generic
  parameters (a generic signature's `Wrapper[T]`) has signatures only: no bodies, no per-instance
  derive or abstract-method checks, no class glue in the C output. Two `TYP-22` tests that relied
  on duck typing (a `T` field returned as `i32`) were corrected.
* **2026-09-25 — the test runs are fast.** Owner-approved speedups A, B and C, measured on 24
  cores. The full suite (`cargo test --workspace --no-fail-fast`) went from about 11 minutes to
  55 s, and `milestones.rs` alone from 650 s to 53 s. The quick check (`annotations.py` over
  every directory) went from 323 s to 27 s. **A:** the harness runs its cases in parallel
  (`run_cases`: one worker per core, failures reported in file order, and a global permit that
  caps cases across all test functions). Each case builds in a folder named after its whole
  relative path, because rule directories share file names. **B:** `annotations.py` checks files
  in a thread pool, each `ember run` in a folder of its own. **C:** with clang and gcc, the runtime
  is compiled once per toolchain and profile into a machine-wide cache and linked as an object
  (ADR-046); MSVC still compiles it with each program. Found on the way: D-251. MSVC detection
  never succeeds (a quoting defect), and nothing reads CI's `EMBER_CC`, so every build so far, in
  CI and here, has used clang.
* **2026-09-25 — MSVC works (D-251 to D-254).** Once found, MSVC is the default on Windows
  (`[MAN-1]`'s `auto`). It showed three things clang had let through:
  * A panic in a `debug` build opened the C runtime's "abort() has been called" window and waited
    for a click (D-252). The first parallel run opened one per panicking test, on the owner's
    screen.
  * A float division by a literal zero, and a struct cast in class vtable adapters, did not
    compile (D-253).
  * The runtime named `max_align_t`, which MSVC's C mode does not declare.

  Fixed: every panic ends in `runtime_abort`, which switches off both of the C runtime's reports,
  and MSVC's `debug` links the release C runtime. The driver reads `EMBER_CC`, and `--cc` accepts
  `clang-cl` and `auto`. MSVC's environment is cached and its runtime object reused (ADR-046), so
  a hello build takes 0.21 s. The full suite passes under MSVC (39 s), clang-cl (51 s) and clang
  (55 s). D-254: the speedups' long per-case folder names had pushed CI's longest paths past
  Windows' 260 characters.
* **2026-09-25 — MSVC on any PATH (D-255).** `vcvars64.bat` failed on a long PATH (cmd's
  8,191-character lines). It now runs on Windows' own short PATH, with telemetry skipped, and
  each build appends its own PATH to what the batch file added. An x64 Visual Studio developer
  environment is used as it is. A read-only reviewer found three more (an x86 prompt was reused, a
  shell's Visual Studio variables could reach the cache, the runtime object's key missed `INCLUDE`
  and `LIB`), all fixed.
* **2026-09-25 — the rest of `[STD-15]` (ODR-031, Hardened_12; D-256 to D-258).**
  * `sort_by(cmp)` and `sort_by_key(f)` are Ember in `std.core`: a stable merge over indices,
    then the permutation applied with `swap`. They take any element type, and `sort_by_key`
    calls `f` once per element, in order.
  * `sort()` and `sorted()` take any `T: Ord` (`sorted` also needs `Clone`) (D-256); numbers
    and text keep the runtime's sort.
  * `windows(n)` yields shared, overlapping views, as `chunks` does, and panics on `0`.
  * `drain(r) -> Array[T]` takes any integer range.
  * Writing the sorts needed D-257: a callable parameter can now be passed on to another
    function, and a lambda in a generic body is no longer emitted over opaque types.
  * Every method `[STD-15]` lists is now built.
  * Two read-only reviewers then broke the first D-257 fix: it crashed the compiler on a lambda
    capturing only concrete locals in a generic body, and could give two closures one C name.
    Revised: such lambdas are marked rather than cut, and closures are numbered from a counter
    that never goes back. They also found `drain(0..=u64.MAX)` draining nothing and `sorted` of
    `Ord` types refused, both fixed. A test they prompted exposed D-258: a `drop` in an `extend`
    block did not stop the implicit `Clone` (ODR-026).
* **2026-09-25 — what `Map` needs first (D-259 to D-274).** The Map/Set work began with two
  read-only agents (a spec checklist of 89 requirements, and a map of the machinery to reuse) and
  18 probe programs; nine of the probes failed. Fixed:
  * `[GRM-13]` (D-259): a `match` on a place not written `owned` binds non-`Copy` parts by
    reference; `ref x`/`ref mut x` in a pattern borrow; `match owned e:` binds by move. std's
    `option_*`/`result_*` helpers and five tests that moved out of a matched local now say
    `match owned`. An arm ending in a `match` whose arms all leave no longer hoists a name
    (D-260).
  * Type-parameter defaults (`H = DefaultHasher`) work in types and constructors (D-266); `[]`
    and `None` take a generic constructor's field type once the parameters are known (D-264);
    constructors accept positional arguments before named ones, as calls do (D-269).
  * `Hash` for every type in `[TYP-36]`'s column but class handles (D-265, D-273 open), and
    `@derive(Hash)`; `DefaultHasher` is now FxHash's class and `Default` (ADR-047).
  * `char as u32` and `u8 as char` (D-271); a `ref Array` field indexes (D-262); `@view` is
    optional documentation, and `E2030` is `@view` on a type that is not a view (D-274).
  * Found and left open: D-261 (a bound on a generic interface with arguments, `Q: AsKey[K]`),
    D-263 (`()` as a `void` value), D-267 (a field default naming a type parameter), D-268
    (implicit `Eq` of a type holding one with a written `eq`), D-270 (a generic enum's variant
    without type arguments), D-272 (`i128`/`u128` in C), D-273.
* **2026-09-25 — `Map` and `Set` (ODR-032 to ODR-036, Hardened_13; D-261, D-267, D-268, D-275
  to D-286).**
  * `Map[K, V, H = DefaultHasher]` and `Set[T, H]` are Ember in `std/src/collections.em`, in the
    prelude: insertion order in `entries`, an open-addressed `slots` table probed from the hash's
    high bits, removed entries closed up once half are gone, a fresh `H.default()` per hash. The
    methods are ODR-032's; `AsKey[K]` has `is_key` and `to_key`, every `K: Eq + Hash` is its own
    `AsKey`, and std gives `str` `AsKey[String]`.
  * The routes are general, keyed on the interface methods' names, not on `Map`: `m[k]` is
    `*m.index(k)` (`index_mut` where written), `m[k] = v` calls `index_set`, `m[k] op= v` holds
    the `index_mut` reference once, `k in m` calls `contains`, `len(m)` calls `len`, and a `for`
    over a type with `iter()` and no `next` iterates `iter()`. `sorted(m)` sorts the keys.
  * `{k: v}` and `{a, b}` literals (`{}` needs a type); a repeated key keeps its first position
    and last value; text with no context is `String` (ODR-036). `{… for …}` is `E0900`.
  * Printing: `{'a': 1}`, `{}`, `{1, 2}`, `set()`; `==` compares as sets of entries.
  * ODR-035: `union` is contextual. ODR-036: no view keys or values (`E3063`).
  * Found and fixed on the way: `Q: AsKey[K]` bounds (D-261), field defaults naming type
    parameters (D-267), implicit `Eq` over a written `eq` (D-268), `!=` through `eq` (D-275),
    `o == None` (D-276), one `L1001` per declaration (D-277), `Self` in generic types' methods
    and `Self(…)` (D-278), `[TYP-36]`'s `Default` (D-281), struct `init` (D-283, part), and
    instances made by signatures missing extensions (D-285). Open: D-279, D-280, D-282, D-284,
    D-286.
* **2026-09-25 — four open defects closed (D-263, D-279, D-282, D-286).** `()` is the value of
  `void`, so `return Ok(())` and `Option[void]` work, and `void` prints as `()` (D-263). A generic
  struct may name a generic type declared after it, collected on demand (D-279). `@borrows(self)`
  names the receiver, in generic types' methods too (D-282). `println`'s arguments are evaluated
  where they are written, as a call's: `println(xs, xs.pop())` is `E3021`, and printing a moved
  value is `E3050` as borrowing one is (D-286); the `[EXC-3]` elision now lets its closing access
  follow plain statements, which the copied arguments add.
* **2026-09-25 — the phase-2 reviewers' findings (D-287 to D-305).** The library reviewer's six
  (D-287 to D-292) and the compiler reviewer's seventeen, all verified. Behaviour a program can
  see: moving out of a class field is `E3012`, and a `match` on one binds by move, so
  `match h.f.clone():` is the way to read an `Option` field (D-302); `.clone()` exists on tuples,
  `Option` and `Result` (D-303); `a[i] op= x` evaluates `x`, then the place once (D-296); a
  `for` loop keeps the temporary it iterates alive to its end (D-299); `ref mut` pattern
  bindings are writes (D-295); scans call a written `eq` (D-294); a generic struct's bounds are
  checked where it is named (D-301); `@view` does not depend on declaration order (D-300); an
  enum's associated functions can be called (D-304). Open: D-305. Every Hardened_8 to 13 header
  now names its own hardening; the cut writes it (`appx_h.py`'s `write_header`).
* **2026-09-25 — D-280 and D-306 fixed.** A lambda passed to a callable parameter from inside a
  generic type's own methods takes its type from the call (`self.items.retain(fn(e) => …)`): a
  method's own type parameters are numbered after the caller's (D-280). A generic type's methods
  with type parameters of their own are checked once with every parameter opaque. An instance
  over a type parameter is one per slot, so `fn f[U](…, g: fn(…) -> U) -> Option[U]` returns an
  `Option[U]` of its own `U` (D-306, present before the 0.9.9 work).
* **2026-09-25 — `std.math` takes any number type (ODR-037, ODR-038; Hardened_14, _15).**
  `math.sqrt`, `sin`, `pow`, `hypot`, `lerp`, `smoothstep` and the rest take any number type: an
  integer's answer is an `f64` (`math.sqrt(9)` is `3.0`), an `f32`'s an `f32`, an `f64`'s an `f64`.
  `std.math.Number` and `std.math.Float` are declared in `std/src/math.em`, every membership
  written there as an `extend … implements` block. `T.Real` names a type parameter's associated type
  (`[IFC-4]`). The float methods of `[STD-20]` exist (`x.sqrt()`, `round` half to even,
  `mul_add`, `is_nan`, …); `PI`, `TAU`, `E` and any `const` written without a type and with a
  literal value are untyped constants; `math.PI` resolves through the module. The per-type
  `min_i32`/`clamp_f32`/`lerp_f32` are gone: the prelude's `min`, `max`, `clamp` replace them and
  carry `[RNG-4]`'s intervals. D-307: a struct's implicit `Eq` meets `interface Ord: Eq`.
* **2026-09-26 — `i128` and `u128` reach C (D-272, ADR-048).** Both work on every compiler: C's
  `__int128` with GCC and Clang, two 64-bit halves with MSVC and clang-cl, every operation a
  runtime helper. `parse`, `len`, `get` and `drain` take them; they are `std.math` numbers.
* **2026-09-26 — shifts and negative literals (D-308 to D-312).** A shift amount may be any
  integer type, and a negative one panics (`[TYP-10]`). `-128` is an `i8` and `-1` no `u8`
  (`[LEX-24]`); `-5 =>` is a pattern, and a pattern literal must fit the scrutinee's type.
* **2026-09-26 — `[STD-20]`'s integer methods and the scalar constants (ODR-039; Hardened_16).**
  Every integer type has `checked_`, `wrapping_`, `saturating_` and `overflowing_` forms of `add`,
  `sub`, `mul`, `floordiv`, `rem`, `pow` and `neg` (and of the shifts, but not saturating), `abs`,
  `pow`, `signum`, `div_trunc`, `rem_trunc`, `count_ones`, `leading_zeros`, `trailing_zeros`,
  `is_power_of_two`, `next_power_of_two`, and `T.MIN`/`T.MAX`; `f32`/`f64` have `INF`, `NAN`,
  `EPSILON`, `MIN` (`-MAX`) and `MAX`. `math.fma` is a fused multiply-add (`[STD-3]`).
* **2026-09-26 — the operator interfaces (ODR-040; Hardened_17; ADR-049).** `std.core` declares
  Part IV §8's `Add`, `Sub`, `Mul`, `Div`, `FloorDiv`, `Rem`, `Pow`, `Neg`, `Not`, `BitAnd`,
  `BitOr`, `BitXor`, `Shl`, `Shr` and their `…Assign` forms, and the prelude exports them. Every
  number type implements those of its operators, so `[TYP-17]`'s `sum[T: Add[Output = T] +
  Default]` takes `i32`, `f64` and a program's type alike; `3.add(4)` is `3 + 4`. A bound binds an
  associated type (`Add[Output = T]`), checked at each call; `a + b`, `-a`, `a ** b` and `a += b`
  on a type parameter go through its bounds. A type implements two instances of one interface
  (`Mul[Vec2]` and `Mul[Mat2]`, D-313). `a += b` calls `add_assign`, else is `a = a + b`. An
  operator needs its interface (D-315): a method that merely shares the name no longer gives one,
  so a `Set`'s `add` is not its `+`, and `Set` implements `BitOr`, `BitAnd`, `Sub` and `BitXor`.
  `-x` on an unsigned value panics unless `x` is 0 (D-314). A generic extension may state an
  associated type (D-317). `not` is a method name after `fn` and `.`.
* **2026-09-26 — `f16` works (D-316, ADR-050; ODR-041, Hardened_18).** `f16` arithmetic, comparisons,
  casts, printing, formatting, `parse`, sorting, `min`/`max`/`abs`/`clamp` and `f16` ranges are
  IEEE binary16, each operation rounded once (they were integer operations on the truncated value).
  `f16` has `INF`, `NAN`, `EPSILON`, `MAX` and `MIN`, and is a `std.math` number answering in `f32`
  (`math.sqrt(x)` of an `f16` is an `f32`). A float range type is no `Hash` (D-318), so
  `R.checked(x)`'s `Result` has its methods again.
* **2026-09-26 — indexing through `Index`, `IndexMut` and `IndexSet` (ODR-042; Hardened_19; ADR-051).**
  `std.core` declares the three and the prelude exports them. `a[i]` needs an `Index`
  implementation (a method named `index` no longer does, D-320), `a[i] op= v` an `IndexMut` one,
  and `a[i] = v` calls `IndexSet.index_set` where the type has it. An `extend` parameter only the
  interfaces name makes a blanket implementation (D-321): `Map[K, V]` implements `Index[Q]`,
  `IndexMut[Q]` and `IndexSet[Q, V]` for every `Q: AsKey[K]`. `Array` and `MutSpan` implement
  `Index[int]` and `IndexMut[int]`, `Span` `Index[int]`, so generic code bounded by them takes
  both, and a bound brings its parents (`C: IndexMut[int]` reads `c[i]` too).
* **2026-09-26 — `std.math`'s vectors, matrices, rotations and shapes, and `KahanSum` (ODR-043 to
  ODR-045; Hardened_20; ADR-052, ADR-053).** `Vec2/3/4`, `IVec2/3/4`, `UVec2/3/4`, `Mat2/3/4`,
  `Quat`, `Transform`, `Aabb`, `Sphere`, `Ray`, `Plane` and `Frustum` are Ember in
  `std/src/math.em` (`[STD-28]`): public components, the operators with a scalar on either side
  (`2.0 * v` is `f32`'s `Mul[Vec3]`), matrix products by fused multiply-adds (`[STD-3]`),
  right-handed projections with depth in `[0, 1]`. `math.KahanSum` is compensated summation. What
  they needed of the language:
  * **Constants** (D-323): a `const` is any constant expression, evaluated while compiling
    (`E6004` for a panic there), in any order (`E6001` for a cycle); a type's `const` is
    `Vec2.ZERO`, private unless `pub`; `E2130` for a type that owns heap memory.
  * **`x is None`** (D-322): the option test works, `a is b` compares two references, and any
    other `is` on a value is `E2150`.
  * **Floating point** (D-325): the C compiler no longer fuses `a * b + c` (`[CG-C-11]`, ODR-044).
  * **`W2015`** shows what the type keeps (D-324) and reaches a literal under a minus or inside
    arithmetic (D-326).
* **2026-09-26 — `std.math.det` (ODR-046; Hardened_21; ADR-054).** `import math.det`, then
  `det.sin(x)`, `det.cos`, `det.tan`, `det.exp`, `det.log`, `det.pow`, `det.atan2` and `det.sqrt`
  for `f32` or `f64`: the same bits on every target (`[DET-4]`), within one unit in the last place
  (`atan2` 1.3). fdlibm's algorithms in Ember, `std/src/math/det.em`; `std.math` itself moved to
  `std/src/math/mod.em` (`[MOD-1]`). Fixed on the way: a negative literal `const` (`-5`, `-1.5`)
  compiles (D-327), and a `const` may bound a range type (D-329). Found and open: a method without
  `pub` is callable from other modules (D-328).
* **2026-09-26 — the compiler's stack (D-330, ADR-055).** The compiler runs on a thread with a 256 MB
  stack on every host: an expression nested more than about fifteen levels deep, which overflowed
  the stack on Windows, compiles there as on Linux. Open: past about four thousand levels it still stops without a diagnostic (D-331).
* **2026-09-26 — `NonZero[T]` (`[STD-4]`, ODR-047; Hardened_22; ADR-056).** `from std.core import
  NonZero`, then `NonZero.new(v)`: `None` for 0, else `Some`; `n.get()` is the integer. `x // d` and
  `x % d` with `d: NonZero[T]` and `x: T` have no zero check, and `Option[NonZero[T]]` is the size of
  `T`. `T` is an integer type; `NonZero[f64]` is `E2040`. Fixed on the way:
  * `i128`, `u128` and `f16` implement `Default` (D-332).
  * `Self` inside another type, `Option[Self]`, in a generic type's methods (D-333).
  * `Pair.make(5)` finds `Pair`'s arguments as a generic function's are found (D-334).
  * A generic type's bound brings its parents (D-335), and a missed bound is one error (D-337).
  * A `@derive(Clone)` over a field with a `drop` is refused however early an instance is made
    (D-336).
  * `x.rem(y)` on a number stays `%` beside another instance of `Rem` (D-338).
  * The standard library's unused extensions of the built-in types are no longer emitted into every
    program (D-339).
* **2026-09-26 — `for x in owned e:` (`[CTL-1]`).** A loop consumes an `Array`, a `Set` or a `Map`
  and yields owned values: the elements, or a map's keys in insertion order with each value dropped
  as its key is taken. `std.core` declares `IntoIterator` (a prelude name, `[MOD-5]`); `Array`,
  `Set` and `Map` implement it. Fixed on the way:
  * A program's own `Cell`, `Box`, `Option` and the other compiler-known names shadow the prelude's
    in type position and in construction, and no longer clash with them inside the compiler
    (D-305).
  * A generic body gets a bounded extension of an instance over its own parameter (D-340).
  * `Maybe.Just(3)` and `Maybe.Nothing` find a generic enum's arguments (D-341).
* **2026-09-26 — a generic body means the same for every instance (D-284, ODR-048; Hardened_23).**
  A generic method or function that returns a reference into a parameter (`fn first(self) -> ref T:
  return ref self.item`) compiles for every type argument, a view such as `str` included: the
  instance passes that parameter as the declaration does. A parameter an instance's type makes a
  view is a source of that instance. Found and open: a temporary without a destructor may be
  borrowed past its statement (D-342).
* **2026-09-26 — private methods, and a class handle as a key (`[MOD-2]`, `[TYP-36]`).**
  * A method or associated function without `pub` is private to its module, as a field is:
    calling it from another module is `E1052` "`name` is private to `module`" (D-328). A method
    implementing an interface is as visible as the interface, `pub` or not. Mark a method `pub fn`
    to call it from elsewhere. `DefaultHasher.new()` is now `pub`.
  * A class handle is `Hash` by identity, so it can key a `Map` or fill a `Set` (D-273).
  * `Either.Right("r")` finds a generic enum's arguments from the expected type (D-270, fixed by
    D-341).
