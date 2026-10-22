# Helper S — the first-week corpus (`[TST-8]`, `[TST-9]`, `[TST-10]`), 2026-10-22

## What was done

* **Phase A** (commit `955d0dc7`): 30 first drafts in `tests/firstweek/NN_*.em`, written after reading
  only Part I and Annex A (the brief's `part1_and_annexA.md`), as a Python programmer would: 56–108
  lines each, one comment line on top, no `#$` lines. Nothing was compiled before the commit; the files
  have not been touched since.
* **Phase B**: `ember check` (the `ci` build of this worktree, at `b8c4df20`) on each original;
  `tests/firstweek/marks.json` records each mark; `tests/firstweek/helped/NN_*.em` is each program with
  the helps applied round by round until it checks clean (all 30 do). The per-round log of every help
  applied, and whether the compiler emitted it, is in the section "Chains" below.
* **Phase C**: `compiler/ember_driver/tests/firstweek.rs` and `tests/firstweek/README.md` (section
  "Phase C" below).

## The marks

**Accepted as written: 0 of 30 (0 %).** **After the catalogue's helps alone, applied literally: 3 of 30
(10 %)** — 17, 18 and 26, which needed only `[DIA-21]` fixes (`True`/`False`, `str(v)`, `", ".join(xs)`)
that the compiler does **not** emit; counting only the helps the compiler printed, 0 of 30. All 30
check clean after the catalogue's helps plus the fixes outside any catalogue recorded below.

The 144 errors of the first checks (`scratch/classify.py`):

| Shape | Errors | With a help | What they were |
|---|---|---|---|
| no catalogue | 50 | 0 | `str` value where `String` is expected (15); single-quoted strings (8); `split()` result indexed (6); `"s" * n`, `[x] * n` (5); `char` vs `str` (5); an interface name as a type (5); a statement or `pass` in a `=>` arm (3); `Err("…")` into `AnyError` (2); a cascade (1) |
| N1 | 38 | 1 (wrong) | `True`/`False` (23), `float(x)`/`str(x)` (14), `Generator[int]` as a parameter type (1) |
| N3 | 14 | 5 | `.lower()` (5, help given); `", ".join(xs)` (7) and `Split.len()` (2) reported as `E1010`, no help |
| gap (spec accepts) | 11 | 0 | string literal patterns (10), a `for` over a temporary's `iter()` (1) |
| N6 | 10 | 0 | `max(xs)`/`min(xs)`, `sort(key=…)`, `m.get(k, d)`, `s.split()` |
| `[DIA-21]` slice | 8 | 8 | `xs[a:b]` |
| N9 | 6 | 1 | `Hash` for a map key, `ref String: Ord`, `char` as a `Set[String]` key, `str + str` |
| N4 | 6 | 1 | `f64` into `Vec3`'s `f32` |
| B4 | 1 | 1 | `letter = letter.lower()` on a parameter |

Later rounds (the helped copies) added: O2 ×13 programs, O1, B3, B10, B12, X1, `E2060`, `E2250`, and the
gaps G3–G6 below.

| Program | First shape | Codes | Rounds | After the helps |
|---|---|---|---|---|
| 01_text_adventure | none `E0101` | E0101 | 7 | needs fixes outside the catalogue |
| 02_csv_summariser | none `E0101` | E0008 E0101 | 4 | same |
| 03_scene_graph | none `E2020` | E2020 | 2 | same |
| 04_event_bus | none `E0100` | E0100 | 3 | same |
| 05_inventory | none `E2020` | E1010 E2020 E2040 | 3 | same |
| 06_path_finder | N9 | E1010 E2020 E2040 | 1 | same |
| 07_todo_list | none `E2020` | E1010 E2020 | 2 | same |
| 08_word_frequencies | `[DIA-21]` | E0100 | 5 | same |
| 09_bank_account | N1 | E1010 E2020 | 2 | same |
| 10_grade_book | gap `E2140` | E1010 E2020 E2140 | 3 | same |
| 11_rpn_calculator | none `E2020` | E2020 | 1 | same |
| 12_tic_tac_toe | none `E2020` | E1010 E2020 | 2 | same |
| 13_game_of_life | N1 | E1010 E2020 | 1 | same |
| 14_queue_simulation | N1 | E1010 E2020 | 1 | same (only `float(n)`, see Q6) |
| 15_library_catalogue | N3 | E2020 E2072 | 2 | same |
| 16_shapes_interface | none `E2020` | E2020 | 4 | same |
| 17_linked_list | N1 | E1010 | 1 | **accepted** |
| 18_traffic_light | N1 | E1010 | 1 | **accepted** |
| 19_tokenizer | `[DIA-21]` | E0100 | 3 | needs fixes outside the catalogue |
| 20_matrix | none `E2020` | E1010 E2020 E2040 | 1 | same |
| 21_particles | N1 | E1010 E2020 | 1 | same (N4 has no lossless `as` for `f64` → `f32`) |
| 22_undo_stack | `[DIA-21]` | E0100 | 3 | same |
| 23_recipe_scaler | N3 | E1010 E2020 | 4 | same |
| 24_dice_game | none `E2020` | E2020 | 1 | same |
| 25_prime_generators | N1 | E1010 | 1 | same |
| 26_game_entities | N1 | E1010 | 1 | **accepted** |
| 27_vending_machine | none `E2020` | E2020 | 3 | same |
| 28_hangman | none `E2020` | E1010 E2020 E2040 E2072 E3023 | 4 | same |
| 29_temperature_stats | N3 | E1010 E2020 | 4 | same |
| 30_binary_search_tree | none `E0101` | E0101 | 3 | same |

What a newcomer gets right first time, by this corpus: classes with `init`, `open class`/`virtual`/
`override`, `let` and `pub(read)` fields, `Weak` parent links with `upgrade()`, `Option` class links
(linked list, tree), enums with payloads and `match` expressions, `Result` with `?`, `gen fn`
generators and generator expressions, comprehensions, f-strings with format specs,
`@derive(Copy)`, `std.math.Vec3`, keyword construction of structs, chained comparisons.

## Release-blocking findings

Repros are in the session scratchpad,
`C:\Users\ism19\AppData\Local\Temp\claude\C--Users-ism19-Code\ae3a2898-3f0f-4361-99e2-9bbbb0976123\scratchpad\repro\`
(each three to nine lines; quoted inline below so nothing depends on that folder).

### A. Rejections whose shape is in no catalogue (`[TST-9]`)

* **A1. A `str` value where a `String` is expected** — 15 sites in 8 originals (05, 07, 09, 10, 11, 15,
  27, 28), and in later rounds of 01, 02, 19, 22, 23, 29, 30. `error[E2020]: expected `String`, found `str``
  with no help. `[TXT-9]` says the fix is `s.to_string()` or `String.from(s)`. The most common
  uncatalogued rejection in the corpus: every `Item(name=name)` with `name: str`.
* **A2. A single-quoted string** (`f"{', '.join(xs)}"`, `f"{'region':<8}"`; 01, 02, 30): `error[E0101]: a
  character literal holds exactly one character`, `error[E0008]: unterminated character literal` for
  `''`. No help (the fix is `"…"`). Also: `E0101` is registered as "unclosed delimiter" (`[LEX-6]`) but
  the lexer uses it for about twenty different messages (`ember_lexer/src/lib.rs`: character literal,
  raw string, escape, f-string brace, byte string…) — `[DIA-6a]` wants each code defined by one rule.
* **A3. Sequence repetition** `"  " * depth`, `"#" * n`, `'*' * n`, `[0] * 7`, `[0.0] * (r * c)`,
  `[Mark.Empty] * 9` (03, 08, 10, 12, 19, 20, 24): `error[E2020]: `*` cannot be applied to `str` and
  `i64`` (note only: "a type has `*` by implementing `Mul`"). The fixes are `.repeat(n)` (`[TXT-10]`)
  and a comprehension; `[v; N]` is not one — it is always a fixed array (`[TYP-38]`), so the second
  try in 12 got `expected `Array[Mark]`, found `[Mark; 9]`` with no help either.
* **A4. `char` against `str`** — `for c in line:` yields `char`, then `c == "S"`, `row.push(c)` into
  `Array[String]`, `game.guess(letter)` with a `str` parameter (06, 08, 13, 19, 28):
  `error[E2020]: `==` cannot be applied to `char` and `str``, no help.
* **A5. A `split` result used as a list** (`parts.len()`, `parts[0]`; 01, 02, 23, 29):
  `error[E1010]: `std.string.Split` has no method named `len`` (code: see D7) and
  `error[E2020]: cannot index `std.string.Split``, no help. The fix (`.collect()`) then gives
  `E2060 cannot tell what `C` is here; write it out, as `.collect[T](...)`` (in no catalogue, help
  given), and an `Array[str]` from a temporary gives B12.
* **A6. `Err("message")` in a `Result[T]` function** (02, 23): `error[E2020]: expected `AnyError`, found
  `String``, no help — and the specification gives no way to make an `AnyError` from a message: the
  only route is a user `@derive(Error)` type and `AnyError.from(…)` (what the helped copies do).
  `[PHIL-8]`: a rejection must name the way. See Q2.
* **A7. An interface name as a type** (16): `shapes: Array[Shape] = [Circle(radius=1.5), …]` →
  `error[E2020]: expected `Shape`, found `Circle`` ×5, no help. The declarations `fn describe(s:
  Shape)` and `fn total_area(shapes: Array[Shape])` are accepted silently, and the next try
  (`Array[Box[dyn Shape]]`) then fails at `describe(s)`: `expected `Shape`, found `dyn Shape``.
  Part IV §9 defines `dyn I`, `Box[dyn I]`, `some I`; a bare interface name in a type position is not
  defined, so `[PHIL-12]` wants `E0901` (or a defined meaning). See Q4.
* **A8. An assignment or `pass` in a `=>` arm of a statement `match`** (04):
  `PlayerJoined(name) => self.scores[name] = 0` → `error[E0100]: expected the end of the line … found
  `=``; `_ => pass` → `error[E0100]: expected an expression … found `pass``. No help; the fix is the
  statement form `PlayerJoined(name): …` (`[GRM-10]`).
* **A9. Indexing a `str`** (`source[i]`; 19): `error[E2020]: cannot index `str``, no help. `[TXT-4]`
  forbids it on purpose; the help would name `chars()` or `as_bytes()`.
* **A10. Tuple indexing** `p[1]` (08): `error[E2020]: cannot index `(String, i64)``, no help (`p.1`).
* **A11. `E2250` and `E2020` after the `[DIA-21]` help for `min(xs)`/`max(xs)`** — see B1.

### B. Helps that, applied literally, do not make the program compile

* **B1. `[DIA-21]` `min(xs)` / `max(xs)` → `xs.iter().min()`** (10, 29; the compiler does not emit it):
  the result is an `Option[ref T]`, so in `fn best(self) -> int: return self.scores.iter().max()` it is
  `E2020 expected `i64`, found `Option[ref i64]``, and in `f"{temps.iter().min():.1f}"` it is
  `E2250 format spec `:.1f` does not apply to `Option[ref f64]``. It needs `.unwrap()` (or
  `.iter().max().unwrap()` in the table). In 14 it compiles but prints `Some(…)`.
* **B2. B4 (`E3023`) for a Python parameter rebinding** (28): `letter = letter.lower()` on `letter: str`
  → help `restructure to a single owner and declare this parameter `mut letter: str``. Applied, the
  parameter becomes in-out: the caller's `game.guess(letter.to_string())` is then
  `E3027 a `mut` argument is not a mutable place` (B10), whose help "bind the value to a local first",
  applied (`one = letter.to_string(); game.guess(one)`), gives the same `E3027` again — a `String` local
  is not a mutable `str` place. The fix is a new local (`low = letter.to_lower()`).
* **B3. O2 (`E3013`) "borrow it instead of moving it; if the function needs ownership, take an `owned`
  parameter"** — the emitted help is not O2's catalogue help (`mem.take`, `mem.replace`, `swap`; `pop`,
  `swap_remove`, `remove`, `drain`), and in most corpus sites neither applies: a comprehension building
  an owned array from a collection (`return [t for t in self.tasks if …]`, 05, 07, 10, 15, 16, 29), a
  field moved into a new value (15, 23, 27), a move-only `Box` (16). In 22 the second half,
  `fn inverse(owned e: Edit)`, leaves both `E3013` (the `match` still binds by reference; it needs
  `match owned e:`) and adds an `E3040` at the caller. The fix in every case was `.clone()`.
* **B4. B3 (`E3021`) "take the borrow as `ref mut`"**: in 29, `hottest = ref readings[0]` then
  `hottest = r` is a write *through* the reference; applied, the help would overwrite `readings[0]`
  (it compiles, with a different meaning). In 04 `m[k].iter_mut()` has no literal edit (G4).
* **B5. N6 for Python keyword habits** (02, 07, 08, 10, 11, 16): `sort(key=fn…)`, `m.get(k, 0)`,
  `s.split()` → `takes 0 argument(s), found 1` etc. with no help; N6's catalogue help (the signature)
  applied literally drops the argument and changes the meaning (`sort()` would then need `Task: Ord`).
  The Ember forms are `sort_by_key`, `get_or`, `split_whitespace` (Appendix E §E.1/E.2 — not in
  `[DIA-21]`'s table).
* **B6. `[DIA-21]` `float(s)` → `s.parse[float]()`** is the help for a string. The corpus wrote
  `float(n)` for an integer 15 times (7 programs: 02, 05, 10, 14, 21, 23, 29); applied literally the
  help is wrong for an `int` (`n as float` is the fix, N4's help). The table has no row for it (Q6).
* **B7. "best so far" (`E3021`, G5)**: help "end the borrow before this: `best` is what keeps it alive,
  so shorten its last use" cannot apply — the use after the loop is the point.

### C. Programs the specification accepts that the compiler rejects

* **G1. String literal patterns** (01, 11, 23; 10 sites). III.6: `literal_pattern := ["-"] INT | STRING |
  CHAR | "true" | "false"`. `match word: "north" | "n" => …` on `word: str` →
  `error[E2020]: this literal cannot match a value of type `str`` (`ember_typeck/src/lib.rs`
  `check_literal_pattern` handles only integers, `bool` and `char`). Not listed in
  `docs/NOT-IMPLEMENTED.md`; an unbuilt construct should be `E0900` (`[PHIL-12]`). Repro:
  ```ember
  fn code(word: str) -> int:
      return match word:
          "north" => 1
          _ => 0
  ```
* **G2. A `for` over a temporary's iterator** (10): `for rank, s in book.ranking().iter().enumerate():`
  → `error[E2140]: `iter` needs an Array variable to point into`. `[EXP-4]`: "one created by a `for`
  iterable lives to the end of the loop". `E2140` is registered as "cannot assign to a value
  expression" (`[EXP-5]`). Repro: `for i, x in make().iter().enumerate():` with `fn make() -> Array[int]`.
* **G3. Storing a consumed callable parameter** (04): `fn subscribe(self, topic: str, owned handler:
  fn(Event)): self.subscribers[topic].push(handler)` into `Map[String, Array[fn(Event)]]` →
  `error[E2020]: expected `fn(Event) -> void`, found `Callable0``. `[CLO-7]` describes exactly this
  (callback registries take `owned f: fn(A) -> R` and keep it) and `[TYP-5]` rule 10 converts a function
  or lambda to a callable type it satisfies; whether that covers the implicit generic of `[CLO-3]` is
  not spelled out — if it does not, the spec gives user code no way to write a registry (see Q11).
  `push(owned fn(e) => handler(e))` is accepted. Also D1.
* **G4. `m[k].iter_mut()` as a `for` iterable** (04): `for handler in self.subscribers[topic].iter_mut():`
  → `error[E3021]: cannot write through a shared reference … this is `ref Array[fn(Event) -> void]`,
  which only reads`, while `self.subscribers[topic].push(x)` is accepted. `Map` implements `IndexMut`
  (`[STD-17]`). Repro: `m: Map[String, Array[int]] = {"x": [1, 2]}` then `for v in m["x"].iter_mut():`.
  `get_mut(k).unwrap().iter_mut()` is accepted.
* **G5. A borrow of a loop element kept across iterations** (02, 08) — the "longest/best so far" idiom:
  ```ember
  fn main():
      xs: Array[String] = ["a", "bb"]
      best = ""
      for x in xs:
          if x.len() > best.len():
              best = x
      println(best)
  ```
  → `error[E3021]: `*x` cannot be written while it is borrowed`, primary span on the `for`. Nothing
  writes the element; the borrow is of `xs`'s element, not of the loop variable. Over `m.keys()` (08) the
  same code gives three errors: this one plus `E3025 cannot call a method on `__it` while `__it` is
  mutably borrowed` (help: "inline the field access, take the two fields as separate parameters, or
  split the method") and `E3060 `__it` does not live long enough` — the yielded reference is tied to
  the desugared iterator instead of the map.
* **G6. Tuples are not `Ord`** (08): `pairs.sort_by_key(fn(p) => (-p.1, p.0))` →
  `error[E2040]: `(i64, String)` does not implement `Ord`, which `K` requires`. `[TYP-36]`: tuples are
  Ord "lexicographic, if all". Repro: `pairs: Array[(int, String)] = …; pairs.sort()`.
* **G7 (side finding, from a repro, not the corpus). `E0106` on a one-statement lambda in brackets**:
  `xs.iter().for_each(fn(x): total += x)` → `a multi-statement closure cannot be written inside
  brackets`; `[GRM-17]` allows exactly one simple statement there.

### D. Diagnostics that break `[DIA-2]` (internal names as the primary message) or give no mandated help

* **D1. `Callable0`** in a primary message (G3).
* **D2. `<error>`** in primary messages: `this literal cannot match a value of type `<error>`` ×5 (23,
  after `parts[2]` failed) and ``(<error>, <error>)` does not implement `Ord`` (08) — both also
  `[DIA-14]` cascades.
* **D3. `__it`** (the desugared `for` iterator) in `E3025` and `E3060` primary messages (08, G5).
* **D4. `self.items.entries`, `self.slots.entries`** — `Map`'s private field — in `E3080` (X1) primary
  messages: "`self.items.entries` is borrowed here and mutably borrowed elsewhere" (05, 27).
* **D5. Wrong primary spans for `E3013`** (23): in `return R(title=self.title, n=1)` of a struct whose
  later field has a default (`items: Array[int] = []`), the span is the default's `[]` on the field's
  line, not `self.title`; in `out.push(I(name=x.name, unit=x.unit))` (with `unit: int`) it is `x.unit`,
  not the moved `x.name`, and stays there after `x.unit` is cloned. The `E3040` that follows repeats
  the wrong line. Repro `move_span.em`:
  ```ember
  struct R:
      title: String
      n: int
      items: Array[int] = []
      fn copy(self) -> R:
          return R(title=self.title, n=1)
  ```
* **D6. `[DIA-21]` is built for one row of nineteen** (the slice). No `True`/`False` help (23 sites), no
  `str(x)`/`float(x)` help (14), no `min(xs)`/`max(xs)` help, no `xs.join`. Worse, `True` gets a
  machine-applicable fix-it "did you mean `Tree`?" (30: a class, the wrong kind for a value position —
  `[DIA-24]`) and `False` gets "did you mean `value`?" (17). `docs/phase-status.json` does not list
  `DIA-21` as unfinished, so the phase table counts it done on the strength of
  `tests/conformance/DIA-21/` (the slice row only).
* **D7. An unknown method is reported as `E1010`** ("`str` has no method named `join`", "`Split` has
  no method named `len`", "`char` has no method named `isalpha`") instead of `E2072` ("no method or
  field of that name", `[STD-13]`), and gets no N3 help — 9 sites in the first round, more later.
* **D8. N9 (`E2040`) without its mandated help** in 5 of 6 first-round sites: `Pos` used as a `Map` key
  (06: the help would be `@derive(Hash)`), `ref String: Ord` for `sorted` (05), `char` as a
  `Set[String]` key (28).
* **D9. N4 without its help** in 4 of 5 `f64` → `f32` sites (21): a note only.
* **D10. `[DIA-14]` cascades**: a failed `" ".join(row)` inside `"[" + … + " ]"` also gives `E2020 `+`
  cannot be applied to `str`` and `E2040 `str` does not implement `Add`` for the same expression (20);
  each `str + str` gives two errors (`E2020` and `E2040`) (19).

## Questions (options and a recommendation; not mine to rule)

* **Q1. Are `[DIA-21]`'s rows shapes?** `[TST-9]` blocks a rejection "in no catalogue (§XVII.6)";
  `[DIA-21]`'s table is in §XVII.6 but its rows have no ids, and `[DIA-12]` classifies name and type
  errors into §XVII.6.2 only. (a) Treat the table as a catalogue as it stands; (b) give each row an id
  (P1–P19) and a code; (c) fold each row into N1/N3/N6 as a help. **Recommend (b)**: marks, the
  harness and `[DIA-13]`'s `tests/ui/` pairs can then name them. `marks.json` uses `DIA-21` for now.
* **Q2. How does a `Result[T]` (`AnyError`) function fail with a message?** (a) a std constructor,
  `AnyError.msg(text)`; (b) `String`/`str` implement `Error`, so `Err(AnyError.from("…"))`; (c) a
  coercion: a string in an `AnyError` position becomes one (like `[TXT-9]`). **Recommend (a)** plus a
  catalogued shape whose help names it; (c) is closest to Python's `raise ValueError("…")` but adds an
  allocation rule the reader cannot see.
* **Q3. Does `ref T` implement `Eq`/`Ord`/`Hash` when `T` does?** `[TYP-36]` has no row for references;
  `sorted(m.keys())` (`[STD-26]`, Python's meaning) and `sorted(by_length.keys())` need `ref String:
  Ord` and `ref i64: Ord`, and are rejected (05, 08). **Recommend yes**, a `[TYP-36]` row "`ref T`: as
  `T`" for `Eq`, `Ord`, `Hash`, `Display`, `Debug`.
* **Q4. What is a bare interface name in a type position** (`s: Shape`, `Array[Shape]`)? (a) an error
  with a catalogued help naming `dyn Shape` / `Box[dyn Shape]` / `[T: Shape]`; (b) sugar for
  `ref dyn Shape` in a parameter and `Box[dyn Shape]` in a container. **Recommend (a)**; today it is
  accepted in declarations and fails at the uses.
* **Q5. Character classification.** Neither the specification nor std gives `char` an `is_digit`,
  `is_alphabetic`, `is_alphanumeric` or `is_whitespace` (Python's `isdigit`, `isalpha`…; 08, 19 wrote
  them). **Recommend** adding them to `[TXT-10]` (Unicode-aware, plus `is_ascii_digit` etc.) with
  Appendix E rows.
* **Q6. `float(n)` and `int(n)` for numbers in `[DIA-21]`.** The table maps `float(s)` to `parse` and
  `int(x)` for a float to `as`; `float(n)` for an integer (the common case) has no row. **Recommend**
  the help follow the argument's type: text → `parse`, number → `as`. With it, 14 would count as
  accepted after the helps (4 of 30).
* **Q7. Sequence repetition** (`s * n`, `[x] * n`). (a) reject with a catalogued help (`s.repeat(n)`,
  `[x for _ in 0..n]`); (b) give `str * int` and `Array * int` Python's meaning. **Recommend (a)**
  (`[PHIL-14]`): `[x] * n` copying a non-`Copy` `x` would hide clones.
* **Q8. Single-quoted strings** (`'abc'`, `''`): **recommend** a `[DIA-21]` row whose fix-it is
  `"abc"`.
* **Q9. `split()` results**: **recommend** that a list operation on an iterator (`len`, `[i]`) get the
  help `.collect[Array[str]]()` (or that `split` be listed in Appendix E as "returns an iterator").
* **Q10. "After the helps" counts the catalogue's helps** (3 of 30) — the helps the specification
  mandates — not only those the compiler printed (0 of 30). **Recommend** publishing the first, as the
  README does, and tracking the gap through D6.
* **Q11. Does a consumed callable parameter convert to an owned callable value in its function's body**
  (G3)? `[CLO-3]` makes the parameter an implicit generic; `[CLO-7]`/`[CLO-15]` describe std registries
  keeping such arguments. **Recommend yes** (state it in `[CLO-3]`): otherwise no user code can write
  the registry Part I's example list suggests.

## Chains (round by round, as applied to `tests/firstweek/helped/`)

Tags: a catalogued shape with `[emitted]` or `[missing]` help; `NONE:<code>` in no catalogue; `GAP` the
specification accepts it. "R2" is the second `ember check`.

* **01** R1 NONE:E0101 `', '` → `", "`. R2 GAP string patterns → if-chain; N3[emitted] `lower` →
  `to_lower`; N1[missing] `True`/`False`; NONE:E2020 str→String ×5 → `to_string()`; NONE:E2020
  `item in room.items` (str in `Array[String]`; `[STD-8]`'s `E2226` help applied:
  `iter().any(fn(e) => e == item)`); N3[missing, `E1010`] `", ".join(xs)` → `xs.join(", ")`. R3 N3
  `Split.len` + NONE index `Split` → `.collect()`. R4 NONE:E2060[emitted] → `collect[Array[str]]()`.
  R5 O2[emitted "borrow it"] `room = self.rooms[…]` ×3 → `ref`; O1[emitted] → `clone()`; B12[emitted]
  → `map(to_string).collect[Array[String]]()`. R6 B3[emitted] → `ref mut`. R7 O2(E3011)[emitted]
  `words[0]` → `.clone()`. Clean.
* **02** R1 NONE:E0101 ×5 + E0008 single quotes in f-strings. R2 N3/NONE `Split` → collect; NONE A6
  `Err(f"…")` → `@derive(Error) enum CsvError` + `AnyError.from`; N6[missing] `get(k, 0)` → `get_or`;
  N1[missing] `float(x)` → `x as float`. R3 NONE str→String ×2. R4 GAP G5 `best = product` →
  `best: String` + `clone()` (a `clone()` into the `str` local first gave `E3060`). Clean.
* **03** R1 NONE `"  " * n` → `repeat`. R2 O2(E3012)[emitted] → `clone()`. Clean.
* **04** R1 NONE:E0100 ×3 → statement arms `Pat:`. R2 GAP G3 (`Callable0`) → `push(owned fn(e) =>
  handler(e))`; B3[emitted] calling a stored owned callable through a shared loop ref → `iter_mut()`.
  R3 GAP G4 → `get_mut(topic).unwrap().iter_mut()`. Clean.
* **05** R1 NONE str→String ×3; N1[missing] `float` ×2; N9[missing] `sorted(m.keys())` → `sorted(m)`
  (`[STD-16]`). R2 O2 ×3: two → `ref` (works), the comprehension → `clone()`. R3 X1(E3080)[emitted]
  (D4) → copy `price` out before the write. Clean.
* **06** R1 N9[missing] `Pos` as a key → `@derive(Copy, Hash)`; N3[missing] `"".join`; NONE char
  vs str → `'S'`; NONE `push(char)` → `c.to_string()`; N1[missing] `False`. Clean.
* **07** R1 NONE str→String; N1 ×2; NONE `tag in t.tags` → `iter().any`; N6[missing] `sort(key=)` →
  `sort_by_key`. R2 O2[emitted, parameter form] `tags=tags` → `owned tags`; O2 ×2 comprehensions →
  `clone()`. Clean.
* **08** R1 `[DIA-21]`[emitted] `pairs[:10]` → `pairs[..10]`. R2 N3[emitted] `lower`; NONE `result = ""`
  returned as `String` → `result: String = ""`; N6[missing] `split()` → `split_whitespace()`, `sort(key=)`
  → `sort_by_key`, `get(k, 0)` → `get_or`; NONE `"#" * n` → `repeat`; N9[missing] `ref i64: Ord` →
  `sorted(m)`. R3 N3[missing, `E1010`] `char.isalpha()` (Q5) → `'a' <= c <= 'z'`; NONE char vs str;
  NONE `String += char` → `push(c)`; NONE `p[1]` → `p.1`; D2 cascade. R4 GAP G6 `(i64, String)` not
  `Ord` → two stable `sort_by_key` passes. R5 O2 ×2 → `clone()`; GAP G5 with D3 (`__it`) → `longest:
  String` + `clone()`. Clean.
* **09** R1 N1[missing] ×3; NONE str→String. R2 O2(E3012)[emitted] ×2 → `clone()`. Clean.
* **10** R1 GAP G2 (`E2140`) → bind `ranked` first; NONE `'*' * n` → `"*".repeat(n)`; N1[missing]
  `float` ×3; N6[missing] `max(xs)` → `xs.iter().max()`; NONE str→String; N6 `sort(key=)` →
  `sort_by_key`. R2 B1: `expected i64, found Option[ref i64]` → `.unwrap()`. R3 O2 comprehension →
  `clone()`. Clean.
* **11** R1 NONE str→String ×3; GAP G1 → if-chain; N6[missing] `split()` → `split_whitespace()`. Clean.
* **12** R1 NONE `[Mark.Empty] * 9` → tried `[Mark.Empty; 9]`; N3[missing] `" ".join`. R2 NONE
  `[v; N]` is a fixed array → `[Mark.Empty for _ in 0..9]`. Clean.
* **13** R1 N1[missing] ×3; NONE char vs str; N3[missing] `"".join`. Clean.
* **14** R1 N1[missing] `float` ×2, `True`/`False` ×3; N6[missing] `max(waits)` → `waits.iter().max()`
  (compiles; prints `Some(…)`). Clean.
* **15** R1 N3[emitted] `lower` ×3; NONE str→String ×3. R2 B3[emitted] `self.books[book.isbn] = book`
  → copy the key out first; O2 ×3 → `clone()`; O1[emitted] `push(loan)` then `return Ok(loan)` →
  `clone()`. Clean.
* **16** R1 NONE A7 → `Array[Box[dyn Shape]]` + `Box(…)`; N6 `sort(key=)` → `sort_by_key`. R2 NONE
  `describe(s)` (`expected Shape, found dyn Shape`) → tried `ref dyn Shape` (`expected ref dyn Shape,
  found ref Box[dyn Shape]`) → `s: Box[dyn Shape]`. R3 O2 `[s for s in shapes if …]` of boxes → count
  with `iter().filter().count()`. Clean.
* **17** R1 N1[missing] `True`/`False`, `str(v)` → `v.to_string()`; N3[missing] `" -> ".join(parts)`.
  Clean — catalogue helps only.
* **18** R1 N1[missing] `True`/`False` ×3. Clean — catalogue helps only.
* **19** R1 `[DIA-21]`[emitted] slices ×2. R2 NONE `source[i]` ×5 → `chars = source.chars().collect[
  Array[char]]()`; N9[emitted] `?` `ParseError` into `LexError` → `.map_err(…)?`; NONE str→String; N3
  `" ".join`; D10 cascade; N9[emitted f-string] `str + str` ×2; NONE `" " * n`; char literals. R3 N3
  `isdigit`/`isalpha`/`isalnum` (Q5) → local `is_digit`/`is_alpha`. Clean.
* **20** R1 NONE `[0.0] * n` → comprehension; N3[missing] `" ".join`; D10 cascade (`E2020` + `E2040`
  on one expression) → `f"[{row.join(" ")} ]"`. Clean.
* **21** R1 N1[missing] `float` ×2; N4 ×5 (`f64` into `Vec3`'s `f32`; one help emitted) → `as f32` ×4,
  `best: f32`. Clean.
* **22** R1 `[DIA-21]`[emitted] ×5. R2 N9[emitted f-string] `str + str` ×2; NONE str→String ×2;
  N1[missing] ×4. R3 O2 ×2 in `match` arms rebuilding an enum: the emitted help's `owned e: Edit`
  fails (B3 above) → reverted; `clone()` in the arms. Clean.
* **23** R1 N3/NONE `Split` → collect; NONE A6 ×2 → `@derive(Error) enum RecipeError`; GAP G1 + D2
  cascade → if-chain; N1 `float` ×2. R2 NONE str→String. R3 O2 with D5 wrong spans → `clone()` ×3.
  Clean.
* **24** R1 NONE `[0] * 7` → comprehension. Clean.
* **25** R1 N1[missing] `g: Generator[int]` as a parameter (`[CORO-3]`: it names a `gen fn`'s own
  frame only; the message does not say so) → `[I: Iterator[Item = int]](owned g: I, …)`; N3
  `", ".join(<generator>)` → collect then `join`; N1 ×3. Clean.
* **26** R1 N1[missing] ×3. Clean — catalogue helps only.
* **27** R1 NONE str→String. R2 O2 `slot = self.slots[code]` → `ref`. R3 O2 ×2 → `clone()`;
  X1(E3080)[emitted] (D4) → copy `price` and `product` out before the write. Clean.
* **28** R1 NONE char → str parameter → `letter.to_string()`; N3[emitted] `lower` ×2; N3[missing]
  `" ".join`; NONE str→String; N9[missing] `char` in `Set[String]` → `c.to_string()`; B4[emitted] →
  `mut letter: str`. R2 B10[emitted] → bind to a local. R3 B10 again (B2 above) → reverted the B4
  help; `low = letter.to_lower()`. Clean.
* **29** R1 N3/NONE `Split` → collect; N1 `float` ×2; N6[missing] `min`/`max` ×4 →
  `xs.iter().min()`. R2 B1 (`E2250` ×4, `E2020`) → `.unwrap()`; NONE str→String. R3 O2 ×3: `ref` for
  `values`, `clone()` for `r.day`; O2(E3011) `hottest = readings[0]` → tried `ref`. R4 B3 (`hottest = r`
  writes through the reference; B4 above) → `clone()` both. Clean.
* **30** R1 NONE:E0101 `', '`. R2 N1[missing] ×3 (with the `Tree` fix-it, D6); NONE str→String ×3;
  N3[missing] `", ".join`. R3 O2(E3012)[emitted] → `clone()`. Clean.

## Phase C — the harness and the published rate

* `compiler/ember_driver/tests/firstweek.rs`, one test, `the_first_week_corpus_keeps_its_marks`
  (doc comment cites `[TST-8]`, `[TST-9]`, `[TST-10]`): at least 24 programs, one mark each, a `# `
  first line and no `#$` lines; each original's `ember check --json` errors equal its recorded codes
  and the first fits its shape (a table of §XVII.6's shapes and codes; `none:`/`gap:` name the code);
  `helped/` holds exactly the rejected programs and each checks clean; the rate the marks give (0 and 3
  of 30) equals `tests/firstweek/README.md`'s `Acceptance rate:` line and its history's last row, and a
  change fails with "the rate … rose/fell from A to B of N", a fall asking for the reason. It prints
  the number of programs with a release-blocking step (24). The checks run on parallel threads: 2 s.
* Break tests: a wrong published rate (5 for 3) → "the rate after the helps fell from 5 to 3 of 30";
  wrong codes and a wrong shape in `marks.json`, and an un-helped `helped/24` → each named.
* `cargo test --profile ci -p ember_driver --test firstweek`: passes.
* `cargo test --profile ci -p ember_driver --test milestones the_conformance_suite_runs`, once, with the
  corpus in place: `test result: ok. 1 passed … finished in 387.52s` — it reads `tests/conformance/`
  only, so `tests/firstweek/` does not trip it.
* **For the lead:** with this test in, `python tools/phase_progress.py --check` fails (phase 8 goes
  69 % → 75 %: `[TST-8]`, `[TST-9]`, `[TST-10]` now have a test); without it the check passes. Run
  `--write` when merging — and consider listing `TST-9` under `unfinished` in `docs/phase-status.json`,
  since 24 programs meet a rejection in no catalogue or a compiler gap, which `[TST-9]` says blocks a
  release (`[DIA-21]` likewise: one row of nineteen is built, D6).
* Other tools that walk `tests/**/*.em`: `phase_progress.py` and `rule_index.py` read only `#$`
  lines (the corpus has none); `c_warning_sweep.py` (manual) will sweep the 30 helped copies too;
  `check_branding.py` passes.

## Process notes

* The scratchpad folder is shared with earlier sessions: writing my `ed.py` there replaced an older
  file of that name (unused by me in the end). Nothing in the repository was affected.
* No compiled program was run; only `ember check`.
