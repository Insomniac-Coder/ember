# Helper V — the first week's diagnostics (`[DIA-21]`, the catalogue, `[DIA-2]`), 2026-10-22

Branch from `aae4a993`. The first-week rate (`tests/firstweek/README.md`): **0 of 30 as written,
3 → 21 of 30 after the catalogue's helps.** The 9 left: 8 meet a compiler gap (G1 01/11/23, G2 10,
G3+G4 04, G5 02, G6+ODR-181 08, ODR-181 05: helper W's, G5 the lead's), and 16 a move-only element of
a comprehension that no help of shape O2 moves (Q-V10). No rejection of the corpus is in no
catalogue any more (`marks.json` has no `none:`).

## Done (and the tests)

### `[DIA-21]`: the Python habits, catalogued shapes P1..P30 (ODR-179)

`ember_diag::habits::Habit` holds the table: the id, what was written, the codes. A diagnostic that
answers one carries it (`Diagnostic::habit`), and `--json` gives it as `"shape"`.
`tests/ui/python/<id>/` holds a case per habit (36 cases); the new test
`committed_python_habit_snapshots_match_and_fixes_compile` (`compiler/ember_driver/tests/ui.rs`)
checks each: its `.stderr`, exactly one error, filed under its habit and one of its codes; where the
diagnostic has a fix-it, `.fixed.em` is the source with the first one applied byte for byte; and
`.fixed.em` compiles. `cargo test --profile ci -p ember_driver --test ui`: 11 passed.

| Id | Written | Code | Answer |
|---|---|---|---|
| P1 | `True`, `False`; `None` where no `Option` is expected | E1010; E2020 / E2060 | fix-it `true`/`false`; `None` needs an `Option` type (help) |
| P2 | `def f():` | E0100 | fix-it `fn`; the token is read as `fn` |
| P3 | `str(x)`, `int(x)`, `float(x)` | E1010 | by the argument's type (ODR-184): text `s.parse[int]()` and `?` where the function's `Result` takes the failure, else `.unwrap()`; a number `x as float`; anything `x.to_string()` |
| P4 | `len(s)` of text | E2073 | fix-its `s.char_count()`, `s.len()` |
| P5 | `min(xs)`, `max(xs)` | E2020 | fix-it `xs.iter().max().unwrap()` (an iterator: `it.max().unwrap()`) |
| P6 | Python method names and call shapes | E2072, E2020 | fix-its: `append`→`push`, `strip`…, `isdigit`→`is_digit` (ODR-183), `sep.join(xs)`→`xs.join(sep)`, `sort(key=f)`→`sort_by_key(f)`, `m.get(k, d)`→`get_or`, `s.split()`→`split_whitespace()` |
| P7 | `xs[a:b]` | E0100 | fix-it `..` (built before; now tagged) |
| P8 | `if xs:` (also numbers, `Option`, pointers) | E2035 | fix-it `not xs.is_empty()`, `x != 0`, `x is not None` |
| P9 | `let x = 5`, `var x = 5` | E0100 | fix-it removes the word |
| P10 | `x == None` on an `Option` without `Eq` | E2040 | fix-it `x is None` (Q-V3) |
| P11 | `with open(p) as f:` | E0100 | fix-it `with f = fs.File.open(p)?:`, adding `import std.fs` when missing |
| P12 | `raise e`; `try`/`except`/`finally` | E0100 | fix-it `return Err(e)`, `raise ValueError(m)` → `return Err(AnyError.msg(m))`; `try` one error, help |
| P13 | `lambda x, y: e` | E0100 | fix-it `fn(x, y) => e`; the tokens are rewritten to it |
| P14 | `self` missing from a method | E1010 | fix-it adds `self`; the receiver is added, so no cascade |
| P15 | `a / b` on integers | E2240 | fix-it `//` (the migration's edit) |
| P16 | `if x = 5:` | E2036 | fix-it `x == 5` (built before; now tagged) |
| P17 | `case p:` in `match` | E0100 | one error per `match`, a fix-it removing every `case` |
| P18 | `global x`, `nonlocal x` | E0100 | help (a `static`, a class field, a capture) |
| P19 | `*args`, `**kwargs` | E0100 | help (an `Array`/`Span` parameter; parameters with defaults) |
| P20 | `"{}".format(n)`, `"%d" % n` | E2072, E2020 | fix-it the f-string (holes `{}`/`{:spec}`; `%d %s %r %5.2f`…) |
| P21 | `Err("…")` into `AnyError` (ODR-180) | E2020 | fix-it `AnyError.msg(…)` |
| P22 | an interface as a type (ODR-182) | E2173 (E2020 at a use) | help `Box[dyn I]`, `ref dyn I`, `[T: I]` (Q-V2) |
| P23 | `s * n`, `[x] * n` (ODR-185) | E2020 | fix-its `s.repeat(n)`, `[x for _ in 0..n]` |
| P24 | `'abc'`, `''` (ODR-186) | E0008 | fix-it `"abc"`; the token is that string |
| P25 | `len()`, `[i]` on an iterator (ODR-187) | E2072, E2020 | fix-it `.collect[Array[T]]()` where the local is made; once per local |
| P26 | a `str` kept as a `String` (A1); `+=` on a `str` | E2020 | fix-its `.to_string()`; `name: String = …` |
| P27 | `char` against `str` (A4) | E2020, E2040 | fix-its `'S'`, `c.to_string()`, `s.push(c)`; a `char` key of `Set[String]` (help) |
| P28 | a statement after `=>` (A8) | E0100 | one error per `match`, a fix-it writing every arm `pattern:` |
| P29 | `s[i]` (A9) | E2020 | help (`chars().collect…`, `as_bytes()`; `[TXT-4]`) |
| P30 | `p[1]` on a tuple (A10) | E2020 | fix-it `p.1` |

A2..A10 are P24, P23, P27, P25, P21, P22, P28, P29, P30; A1 is P26; A11 is B1 below.

### B1..B6: helps that, applied literally, did not compile (D-823)

* B1: `max(xs)` gives the element (`.unwrap()`, P5).
* B2: a parameter given a new value (`letter = letter.lower()`): the help binds a new local, and
  `mut` only if the caller is to see it; no `Cell`/`RefCell`/class helps for it.
* B3: O2's help (`E3011`, `E3012`, `E3013`) names the borrow (`ref`, `ref mut`) first (`[DIA-9]`),
  then the catalogue's `mem.take`, `mem.replace`, `swap`, `pop`, `swap_remove`, `remove`, `drain`,
  then `.clone()` for a value the owner keeps. (16's case: Q-V10.)
* B4: `r = v` on a `ref` local (Python's rebinding; a reference is never re-pointed, ADR-010) is no
  longer offered `ref mut`, which would have written `v` into the element: the help keeps a copy or
  an index. `tests/conformance/BRW-1/reject_a_reference_is_not_re_pointed.em`. (`m[k].iter_mut()`
  is G4, W's.)
* `return None` in a function with no result: P1, "returns with a bare `return`".
* B5: P6's call shapes (`sort(key=)`, `get(k, d)`, `split()`).
* B6: P3 by the argument's type (ODR-184).
* B7 (G5's "best so far"): the lead's gap; its help stands until then.

### D1..D10: `[DIA-2]` breaches, missing helps (D-822)

* D1: a callable parameter where a `fn` place is wanted is named (`found the callable parameter
  `handler``), with the help `owned fn(…) => handler(…)` (W's G3 then accepts the push).
* D2 / D10 (`[DIA-14]`): a type holding an error (`Array[<error>]`, `(<error>, int)`) is reported
  where the error was and nowhere else: coercions, binary operands, literal patterns, missed
  bounds, method calls on it (`mentions_error`). `tests/conformance/DIA-14/reject_an_operand_in_error_is_one_error.em`.
* D3: a loan on the `for` loop's iterator is "the `for` loop's iterator", never `__it`
  (`borrows.rs` `quoted_place`; the keeper of a loan is never it). Seen in 08's G5 errors; no
  conformance case, since the lead's G5 fix removes them.
* D4: a standard type's private field is not shown: the owner stands for it (`self.items`, not
  `self.items.entries`), as `Box`'s is. `tests/conformance/EXC-3/reject_a_map_field_is_named_as_the_program_writes_it.em`.
  Three FFI tests that quoted `*bag.pointer` / `*theirs.pointer` now read `bag` / `theirs`.
* D5: an aggregate's statement is at the first value moved out of a place, else the aggregate;
  a plain assignment at the value it reads (`self.slots[code]`, not the index `code`)
  (`ember_mir/src/lower.rs`). `tests/conformance/EXP-6/reject_a_moved_field_is_the_span.em`; the B5 ui
  snapshot now points at `Parsed(t, t[0..5])`, which its label names.
* D6: `[DIA-21]` built (above); `True` never gets "did you mean `Tree`".
* D7: an unknown method is `E2072` everywhere (25 sites), with N3's help (the Python name, P25, the
  nearest methods). 20 conformance expectations moved from `E1010` to `E2072`; STD-13's and
  `docs/errors/E2072.md`'s message is the one shape, "`T` has no method named `m`".
* D8: N9's help: `@derive(X)` on the program's own struct, or the `extend`'s module (`[TYP-20]`);
  a `char` key of `Set[String]` is P27. `tests/conformance/DIA-12/reject_n9_and_n4_name_their_fix.em`.
* D9: N4's help: the `as` and what it does (rounds, rounds toward zero), as a fix-it.
* `Generator[T]` outside a `gen fn`'s return type: help `[I: Iterator[Item = T]]` (`[CORO-3]`).

### std

* `AnyError.msg(owned text: String)` (`std/src/core.em`, a private `Message` error).
  `tests/conformance/ERR-8/accept_a_message_is_an_any_error.em`.
* `char.is_alphabetic`, `is_digit`, `is_alphanumeric`, `is_whitespace` and the `is_ascii_*` forms
  (`std/src/string.em`): ASCII inline, past it a runtime binary search (`ember_char_class`,
  `RuntimeFn::CharClass`) of tables `tools/unicode_case.py` now also writes (Unicode 16.0, as Python
  3.14 gives them; `--check` passes). `tests/conformance/TXT-10/accept_char_classes_as_python_gives_them.em`.

### `E0101` (ODR-186's second half) — separate last commit `4aa9adfc`

`E0101` keeps the unclosed delimiters (strings, raw strings, an f-string's `{`); a malformed literal
(an escape, a character's value, a raw string's prefix, a byte string's character, an f-string's
lone `}`) is the new `E0009` ("malformed literal", `docs/errors/E0009.md`); a NUL in `c"…"` is
`E5064`, as `[TXT-3]` says. `tests/conformance/LEX-25/reject_malformed_literals_are_their_own_code.em`.
**Until the specification carries `E0009` and its rule, `ember_diag`'s
`the_registry_and_the_specification_name_the_same_codes` and `tools/rule_index.py` fail on
`E0009`** (Q-V4). The commit can be dropped alone, with `docs/errors/E5064.md` (commit `c48143d3`),
whose failing example is the c-string NUL the lexer reports as `E5064` only after it.

### The first-week corpus

`marks.json` re-marked and `helped/` regenerated by `scratchpad/V/remark.py` (the compiler's fix-its
round by round, then the text helps applied literally, then for a gap the workaround S used); each
helped copy checks clean. The harness knows P1..P30 from `ember_diag::habits`, and checks a habit
mark against `--json`'s `"shape"`. README: rate 21 of 30, a history line. `--test firstweek`: passes.
`docs/phase-status.json`: `DIA-21` left `unfinished` (every row built); `TST-9`'s entry says what is
left. `python tools/phase_progress.py --check`: passes.

## Compiler changes

* `ember_diag`: `habits.rs` (new), `Diagnostic.habit`, `--json` `"shape"`, `Sink::emit_all`;
  `codes.rs` `E0009` (last commit).
* `ember_typeck`: `habits.rs` (new: the P answers, `no_method`, `mismatch_habit`, `operator_habit`,
  `index_habit`, `bound_help`, `mentions_error`, P22's check); `lib.rs` hooks (names, calls, methods,
  operators, indexing, coercion, `+=`, conditions, `/`, `None`, `Generator`, callable parameters,
  B4's rebinding help, deferred P22).
* `ember_parser`: `python.rs` (new: P2, P9, P11, P12, P13, P14, P17, P18, P19, P28; tokens rewritten
  for recovery); hooks in `body.rs`, `decls.rs`, `lib.rs`; P7, P16 tagged.
* `ember_lexer`: P24 (single-quoted strings become the string they mean); the `E0101` split.
* `ember_analysis`: `drops.rs` O2's helps; `borrows.rs` `quoted_place`, std private fields.
* `ember_mir`: `lower.rs` the aggregate and assignment spans (D5).
* `ember_hir`: `RuntimeFn::CharClass`. Runtime: `ember_char_class` and its tables.

## Defects found

* **The interface cache's summary of `std.core` depended on the program** when a non-generic std
  function called a generic one (`AnyError.msg` written as `AnyError.from(Message(text))`): two
  programs in one directory then failed with the ICE "[LT-40] cannot load interface cache:
  cached callable-region metadata disagrees with the current verified summary for `std.core`"
  (repro: `scratchpad/V/pair.py` with that `msg` body; any two programs, one using it). `msg` now
  builds the `AnyError` directly, which avoids it; the cache's program-dependence is open (`[LT-40]`).
* `Box[dyn I]` has no `get()` (IX.1 lists `b.get()`), and `ref Box[dyn I]` does not become
  `ref dyn I` (not in `[TYP-5]`'s list), so P22's `ref dyn I` parameter cannot take a boxed value
  (16 uses a `Box[dyn Shape]` parameter instead).
* A comprehension's `for` gives references even over `drain`, and `owned` is refused there
  (`E0109`): a filtered list of move-only elements cannot be made (16, Q-V10).

## Spec questions (options; recommendation)

* **Q-V1. The new rows.** P21..P30 are not in `[DIA-21]`'s table, and `E2072`'s, `E2035`'s… habits
  are not written. Recommend adding the ten rows as the table above gives them, the habit ids in
  a column (ODR-179), and Appendix E rows for `isdigit`/`isalpha`/`isalnum`/`isspace` (ODR-183),
  `xs.sort(key=f)` → `xs.sort_by_key(f)`, `"%d" % n` and `"{}".format(n)` → f-strings,
  `s * n` → `s.repeat(n)`, `[x] * n` → `[x for _ in 0..n]`.
* **Q-V2. ODR-182 and `[OBJ-2]`.** A bare interface name in a type position already means "a class
  handle typed by an interface" (IV.1's existential row, `[OBJ-2]`, `[DSP-3]`; DSP-3, EXC-15,
  EXC-18, RC-2e, RC-3, TYP-13 tests use `Array[Shape]` of classes). Built: it is `E2173` (P22) when
  the interface's implementations are values and none is a class; otherwise the class handle, and a
  struct given for one gets P22's help at the use. Options: (a) as built; (b) the ruling literally,
  with a new spelling for the class-handle existential (`dyn I` held by a class handle?).
  Recommend (a), and a line in `[OBJ-2]` saying the bare name is that handle. `E2173` stretches
  `[GRM-8b]` (an instantiation argument); recommend its row read "an argument or type position
  that is neither a type nor a constant (an interface implemented by values only: ODR-182)".
* **Q-V3. `x == None`** compiles when `T: Eq` (Python's meaning), so the row answers it only where
  it is refused. Options: (a) as built; (b) a lint with the `is None` fix-it; (c) reject it.
  Recommend (a) (`[PHIL-14]`), the row reading "`x == None` where the `Option` has no `Eq`".
* **Q-V4. `E0009`.** Rows for §XVII.9: `| \`E0009\` | malformed literal: an escape, a character's
  value, a raw string's prefix, a byte string's character, an f-string's lone `}` | \`[LEX-26]\` |`,
  and in II.5: "`[LEX-26]` A literal is written as the grammar above gives it: an escape is one of
  `escape`'s forms, a character literal one Unicode scalar value, a raw string's prefix at most
  eight `#` and a `"`, a byte or C string ASCII (or escapes), and an f-string's `}` doubled outside a
  hole; anything else is `E0009`. An unclosed literal is `E0101`." `E0101`'s row could then read
  "unclosed delimiter or literal". Recommend exactly these.
* **Q-V5. `char`'s classes.** Built as Python's (`isalpha`: Lu/Ll/Lt/Lm/Lo; `isdigit`: digits of any
  script and superscripts; `isalnum`: alphabetic or numeric; `isspace`: what `split_whitespace`
  splits at), the `is_ascii_*` forms their restriction to ASCII. The alternative is Rust's
  properties (Alphabetic, White_Space, `is_digit(radix)`). Recommend Python's, written in `[TXT-10]`.
* **Q-V6. `AnyError.msg`'s signature**: `owned text: String` (a literal makes one, an f-string moves
  in). Recommend `[ERR-8]` list it: "`AnyError.msg(text)` is an error whose text is `text`".
* **Q-V7. `int(s)`'s fix-it** ends in `?` (in a function whose `Result` takes a `ParseError`) or
  `.unwrap()`, so it gives the `int` Python's does; the table writes `s.parse[int]()`. Recommend the
  row say so, as B1's `max(xs)` → `xs.iter().max().unwrap()` (Python raises on an empty one).
* **Q-V8. Unknown fields** are still `E1010` ("cannot find field"), N1; `E2072`'s row covers fields
  ("no method or field of that name"). Options: move them to `E2072` (shape N3; the N1 ui cases
  move with them) or change `E2072`'s wording. Recommend moving them.
* **Q-V9. `[DIA-24]` and type names**: a type name stays a value candidate (`Color` before
  `.Red`); `True`/`False` are P1 now, so the reported `Tree` is gone. Recommend `[DIA-24]` say a
  type is of the right kind before `.`.
* **Q-V10. O2's help.** The catalogue's O2 help has no borrow, and `[DIA-9]` wants the structural fix
  first; built: the borrow first. And a comprehension over a borrowed collection of move-only
  elements (16) has no O2 help that applies (`.clone()` needs `Clone`, `drain` still gives
  references). Options: let a comprehension collect references (`Array[ref T]`) or consume an
  `owned` iterable; or a help naming `iter().filter(…)`. Recommend the first (a language question
  for the owner).
* **Q-V11. B4 for a rebinding**: the catalogue's B4 help ("a single owner passing `mut`") does not
  fit Python's rebinding of a parameter; built: a new local first. Recommend B4's row add it.

## Process notes

* Every check ran through `ember check` or the harness; no compiled program was run directly.
* Scripts (not in the repository): `scratchpad/V/remark.py` (+ `manual.py`, `post.py`: the text
  helps and the gap workarounds applied, round by round), `bless.py` (ui snapshots and fix-its),
  `autohelp.py`, `pair.py` (the cache repro).
