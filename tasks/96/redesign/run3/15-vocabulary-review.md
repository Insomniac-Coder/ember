# 15 vocabulary reviewer: report (run 3, phase 3)

Power: `PowerLineStatus` printed `Online` at the start and at the end. No compiler change, no git
command, no Ember program run (one `ember check`, pasted in change 6). Files written: this report,
`RUN3/combined-v2.md`, `RUN3/vocabulary-final.md` (both made with `cp` from `combined.md` and
`vocabulary-combined.md`, then edited; the originals are untouched), and the scripts and outputs under
`target/redesign3/15-vocabulary-review/` (section 7).

Terms used here:
- **word**: anything a programmer writes to say something: a keyword (`owned`), an `@` attribute
  (`@view`), or a library type or function that acts as a language word (`thread.spawn`).
- **NEW block**: the code in point 3 of an answer in `combined.md` (the proposed spelling).
- **mode**: what a parameter's name says the call does with the argument: read it (no word), change
  it (`ref mut`), take it over (`owned`).
- **the four tests** (06's, `vocabulary.md` 6.1): (1) the line says what each name is and whether it
  may change it; (2) a word means the same for every type; (3) no word has two meanings and no two
  words share one; (4) a word reads right where it sits and names what it describes.
- **15:** marks every place I changed, in both copies.

## 1. What I did

1. Read the brief, `JOB.md`, my duty, `14-combiner.md`, all of `vocabulary-combined.md`, sections 0-9
   of `combined.md`, every area's terms and rules, the four readers' reports, and 06's section 6.
2. Pulled every NEW block out of `combined.md` with `extract_new.py`: 81 code blocks in 80 answers,
   443 distinct names and words (`new_blocks.md`, `tokens.txt`). The four answers with no NEW code block
   (AN-16, message text; AN-24 and AN-29, "the OLD program, unchanged, is accepted"; AN-38, "as AN-37")
   I read by hand. I read every NEW block, line by line, against the vocabulary.
3. Scanned the parts of `combined.md` that state the new model (rules, NEW, the owner's test, still
   refused; not OLD, not "means exactly", not pasted output) for old words and old meanings with
   `scan_old_words.py` (230 hits, `hits.txt`), and read each hit.
4. Listed, with `spec_uses.py`, every code use in spec lines 1-6930 of every word whose spelling or
   meaning changes (`spec_uses.txt`), checked the lists against the readers' line lists, read the rule
   paragraphs that state an old meaning, and wrote the result into `vocabulary-final.md` section 6.
5. Fixed what failed, in the two copies, and checked: every agreed spelling of rows 3-9 still appears
   in `combined-v2.md` as often as in `combined.md` (11 spellings, counts equal; the check is
   14's `build.py` list, run by hand); every word-table row of `vocabulary-final.md` has its seven cells
   (`check_tables.py`: 274 rows, 0 bad; `vocabulary-combined.md`: 270 rows, 0 bad).

## 2. Every change, with its reason

| # | word | where | before | after | why (test) |
|---|---|---|---|---|---|
| 1 | `shared`, `thread_shared` on a parameter | `combined-v2.md` R4, area C's C5, R10, AN-48 (`transfer` and its owner's test), AN-78 (owner's test); vocabulary rows `shared`, `thread_shared` | "`x: shared T` / `x: thread_shared T` makes the callee one more owner"; C5: "(the caller writes `shared y` or hands an owner over)"; AN-48's `transfer(a: shared Player, b: shared Player)` changes `b` with "the function holds an owner of each" | the mode is on the name, as for every type: `x: shared T` reads the caller's `shared` value (a view, no count, kept alive by R10), `ref mut x: shared T` may change it, `owned x: shared T` takes one owner (the caller writes `shared y` or hands its own over); AN-48 becomes `transfer(a: shared Player, ref mut b: shared Player)` | (2), (3): the type word acted as a mode for these two types only, the form 06's C-2 refuses for `owned` ("`fn f(m: owned Mesh)` and `fn f(owned m: Mesh)` ... mean 'read it' and 'take it'"); C5's own principle is "two facts, two places: the mode on the name, the holding in the type". The old rule also contradicted AN-78's NEW (`token = setup(game)` then `println(game.score)` would be a use after a hand-over). A programmer cares: under the old rule every call to a function taking a shared object handed it over or paid a count pair; now a call reads it for free, as every other parameter does. AN-48's C is unchanged (pointers, no count) |
| 2 | `@view` | R7; vocabulary `@view` | "A struct that holds a view is `@view`" (reads as a required mark) | a struct holding a view is itself a view (`[TYP-34]`); its view fields' types already say so; `@view` may say it again and is checked | (3): AN-22's `Parser` (`src: StrView`) has no `@view`, `[TYP-34]` made the mark optional, and a required mark would say twice what the field types say (area E's reason for removing `@sync`) |
| 3 | `StrView` field | R7; vocabulary `str` row | "A `StrView` field says what it points into"; "a `StrView` field with no `from`" refused | the `from` rule is for a field of a class (never a view); a struct holding a `StrView` with no `from` is a view of the text it was made from; `from static` may sit on a struct's field too and then does not make it a view | (2): the row's refusal contradicted AN-22's accepted `Parser`; AN-26 itself says "A `StrView` field **of a class**" |
| 4 | `SAFETY:`, `@safety`, `unsafe fn` | R15; AN-79 (NEW, rule text, owner's test line numbers, means-exactly label, still refused); vocabulary `SAFETY:`, `@safety`, `unsafe fn` | AN-79's NEW kept the OLD line `# SAFETY: the caller guarantees ...` above `unsafe fn`, beside `@safety(...)` saying the same; the `SAFETY:` row said "above an `unsafe` block" without the category form | `# SAFETY:` (or `# SAFETY(<category>):`) says why one block or `unsafe` implementation is sound; an `unsafe fn` carries only `@safety` (L3015, every `unsafe fn`); a note on the `unsafe fn` line gets a warning with the line to delete. AN-79's NEW loses the line; its "means exactly" file keeps it only because today's `[UNS-8]` asks for it (change 6's output) | (3), (4): one comment with two meanings (why this block is sound / what callers must make true), and two notes on one declaration, against AN-79's own rule ("one word, two places, each with its own sentence"); AN-84's NEW writes `# SAFETY(ffi):`, which the row did not cover |
| 5 | `override`, `virtual`, `changes` | R5 (new point); vocabulary `override`, `virtual`, `changes` | nothing said how `changes` meets `virtual` | a call through a virtual (or interface) method is checked against that method's first line, so an override's `changes` stays within it; a base meant for overrides that change their object says `virtual fn on_update(mut self, dt: float):`; refused with the line to write | (1), (3): without it, `changes` on a virtual would not mean what it says at a call through the base (a loop over `self.players` calling `p.on_update(dt)` would trust a line the override breaks). The spec's own `Door` example (1704, 6160) writes `angle` from `virtual fn on_update(self, ...)` |
| 6 | (evidence for 4) | `target/redesign3/15-vocabulary-review/an79_no_fn_note.em` | | the AN-79 NEW text without the note on the `unsafe fn` line; `ember check` prints `warning[W3012]: this \`unsafe\` has no safety note` at `3:1` (`unsafe fn read_u32_le(...)`), help "write `# SAFETY: <why it is sound>` on the line before it ...", then `exit=0` (pasted in full in `combined-v2.md`, AN-79) | today's W3012 asks for the note on an `unsafe fn`; in the model that check is L3015's |
| 7 | "handle" | AN-24 intent; area D rule D5; vocabulary `Handle` | AN-24: "keep a list of handles that each read one entity"; D5: "A handle read from a place ..."; the row: "'handle' is used only for `Handle[T]`" | "a list of views"; "An object reached through a place"; the row: "handle" names a token for something elsewhere, `Handle[T]` and the existing written words `JoinHandle` and `@ffi(handle)`, never a class object or a link | (3): AN-50's ruling ("handle" only for `Handle[T]`) was broken by the model's own text for a view and for an object; and the ruling as written would have outlawed two existing written words that already mean a token (reader 03's finding) |
| 8 | `cast` (a plain name from it) | AN-22 still refused | "`pool = cast self.pools[k] as Pool[T]` (no `ref mut`): 'this would take the pool out of `self.pools`'" | "`pool` reads the pool in `self.pools` and cannot change it; to return a changeable element, write `pool = ref mut cast self.pools[k] as Pool[T]`" | (3): the message used the old meaning of `=` on a part (area C's partial hand-over), which clash 1 removed: by R2 and R13 a plain name bound from a `cast` of a place is a read-only view |
| 9 | `String(...)` / `.to_string()` | AN-54, AN-56, AN-59 (NEW); vocabulary: new row `.to_string()` | `"ann".to_string()`, `"a".to_string()`, `"b".to_string()` beside `String("tick")`, `String("key")` elsewhere | `String("ann")`, `String("a")`, `String("b")`; the row: `String(s)` is the model's spelling for your own `String` made from text, `s.to_string()` (the `Display` method for any value) means the same on a text view and is kept, not refused | (3): two spellings for one meaning in the NEW code; row 5 agreed `String(...)`. Not refused at a use, unlike `.clone()`: its general meaning (any value written as text) has no other spelling |
| 10 | `owned` | vocabulary `owned` | silent on a generic `owned v: T` given a `Copy` value | stated: the value is copied bit for bit and the caller's name stays (06's C-2, which keeps generic code working); `mem.drop` ends the name for every type by its own rule; on a parameter `owned` on the name says what the call does, in the type how the value is held | (2): the row said "for every type that is not `Copy`" and "refused where the type is known to be `Copy`", leaving the generic case (AN-42's `emplace[T](..., owned value: T)` called with `30`) unstated |
| 11 | `Copy` | vocabulary `Copy` | "**Not** class handles, `shared` values or `Weak`" | "class objects, `shared` values or `weak` links"; R2's copied `weak` part noted | (3): old spellings inside the new meaning |
| 12 | `threads` | vocabulary `threads` | places: after `#!`, in `@ffi`/overlay | adds `@export(threads=any \| main \| creator)` (`[FFI-33]` 4964) | AN-84's NEW writes it; the row lacked the place |
| 13 | `Commands` | vocabulary `Commands` | "where it is written: in a system's parameters" | "reached through the system's query, `q.commands()`" (`[ECS-5]` 3838) | (4): the spec and AN-06 never write it as a parameter |
| 14 | `SoA.get` | vocabulary `SoA.get` | "stays refused: as today" | `ps.get(i) -> Option[SoARef[T]]`; `load()` is the method `copied ps[i]` runs, and `ps[0].load()` written in a program gets "write `copied ps[0]`" | AN-13 refuses it, as the `.clone()` row does for `clone`; the row did not say so |
| 15 | `get_mut` | vocabulary `get_mut` | "same" (today: "into a `Shared` or a container element") | a container element only; `Shared.get_mut` goes with `Shared` | (3): "same" kept a removed type's method |
| 16 | `CppShared` | vocabulary `CppShared` | "if `Shared` becomes `shared`, 07 decides whether it follows" | keeps its name, a C++ count is not Ember's (07, `planned-words.md` 341-342) | the decision existed; the row was stale |
| 17 | `ForeignCallback`, `thread.spawn`, `range` | vocabulary: three new rows | no rows | `ForeignCallback`: today's std type a `Retained` callback token holds (`std/src/ffi.em:225-228`); `thread.spawn`: its function's capture list says what it holds (R14, E7004); `range(n)`: `0..n`, so R8 (c)'s counting-loop rule covers `for i in range(xs.len()):` | words in the NEW code (AN-78; AN-55, AN-71, AN-75, AN-76, AN-80; AN-51) with no row, two of them carrying a rule of the model |
| 18 | (headers) | top of `combined-v2.md` and `vocabulary-final.md`; `vocabulary-final.md` section 2a | | each copy says what it is and how 15's changes are marked; section 2a lists 15's row changes | |

No agreed row (3-9) was touched: their 11 spellings appear in `combined-v2.md` exactly as often as in
`combined.md`. No "means exactly" translation changed: change 1 keeps AN-48's C (two pointers, no
count) and AN-78's means-exactly; change 4 explains why AN-79's means-exactly file keeps its line;
change 9 changes a literal's spelling only.

## 3. Check 1: does every word read right where it sits and name what it describes?

Every word the model adds or moves, read in its NEW lines:

| word | where it sits | reads | verdict |
|---|---|---|---|
| `changes`, `reads` | the method's first line, before `->` | "`fn note(self, msg: String) changes log:`" | passes (agreed rows 3, 7) |
| `copied`, `shared`, `thread_shared`, `weak` before a value | on the value they make | "`snapshot = copied items`", "`st = thread_shared Stats()`", "`cam.target = weak player`" | passes |
| the capture words | after the lambda's parameters, one per name | "`fn() with owned name, copied score:`" | passes (row 6's shape) |
| `owned` in a type | on the field's type | "`mesh: owned Mesh`" | passes (the owner's ruling) |
| `shared`/`thread_shared` in a parameter's type | on the type | after change 1, says what the value is, not what the call does | passes after change 1 |
| `set by` | after a `weak` field's type | "`parent: weak Node set by children`" | passes |
| `guarded by` | after a field's or `static mut`'s type | "`words: int guarded by lock = 0`" | passes |
| `from` | after a result type; after a class's view field | "`-> StrView from src`", "`words: Array[StrView] from text`", "`kind: StrView from static`" | passes; `from static` keeps C's existing `from(static)` meaning (3), spelled as a clause |
| `@cached` | above the method | "`@cached fn radius(self) reads verts -> f64:`" | passes |
| `Lock`, `with x.lock:` | a field's type; the bare `with` | "`with self.lock:`" = with the lock held | passes ("hold this until the block ends", settlement 10) |
| `@no_alias_checks`, `@fp(allow_fma)`, `StrView`, `CStrView`, `check_disjoint`/`assert_disjoint`, `ThreadShared` | as today's words they replace | each says what it does | pass |
| `# SAFETY:` | on an `unsafe fn` line | repeated `@safety` | **failed; fixed** (change 4) |
| "handle" in the model's text | a view (AN-24), an object (D5) | named the wrong thing | **failed; fixed** (change 7) |
| `@view` | read as required on every view struct | said again what the field types say | **failed; fixed** (change 2) |

## 4. Check 2: does every word mean one thing, everywhere, for every type?

**In the NEW code.** I read every use of every word that changes meaning against its row, type by
type: `copied` (a list, a `String` element, a `Copy` struct part, a class with `Clone`, a generic `T`,
an `SoA` proxy, a `comptime` value), `shared` (a list, a struct, a class, `dyn`, a capture), `owned` (a
parameter of a class, a struct, a `String`, a function, a generic; a type; a capture), `weak`,
`ref mut` (parameter, local, loop name, pattern, query component, kept view field, callable type),
`with`, `from`, `changes`, `self`/`mut self`, `cast`, `MutSpan`, `Span`. Failures found and fixed:
changes 1 (`shared` on a parameter), 3 (`StrView` field), 4 (`SAFETY:`), 5 (`changes` through a
virtual), 8 (`=` on a cast part), 9 (`String(...)`), 10 (`owned` with a generic `Copy` value).

**At every use in the spec.** `vocabulary-final.md` section 6 lists, for every word whose spelling or
meaning changes, every code use in spec lines 1-6930 and what it becomes (6.1: 25 spelling changes,
each with its line list, from `Box` (35 lines) to `Sync`-the-property (61 lines, with the 8 lines where
the effect keeps `Sync` named apart)), and every rule or example that states an old meaning, with the
model's rule that replaces it (6.2: 38 entries, from `[TIER-1]` and `[FN-1]` to Annex A and the
glossary). 06's own list of this kind (`vocabulary.md` 6.3) is folded in. The spec is not edited in
this run; section 6 is what its rewrite works from, so no old use is left meaning something else
unnoticed. Three uses the readers did not list, found while checking: `[CLO-7]`'s "captures by move"
(2123-2128), the glossary's "Safe Ember" (6916, which `unsafe fn` bodies change), and `[OWN-8]`'s
"cloning a class handle copies the handle" (2273-2274).

## 5. Check 3: words in the NEW blocks that are not in the vocabulary

Of the 443 names and words in the NEW blocks, the program's own names (types, fields, locals,
functions it declares) are not words. The rest were each found in a row of the vocabulary, except:
- **given rows by 15** (change 17, change 9): `ForeignCallback`, `thread.spawn`, `range`,
  `.to_string()`; and new places for rows that existed: `threads` in `@export(threads=...)`,
  `SAFETY(<category>):` (AN-84), `q.commands()`.
- **left without a row**, ordinary library names that say nothing about who owns or changes a value
  and follow their spec meaning: `len` (`[STD-26]`, `len(x)` is `x.len()`), `println`, `panic`,
  `sorted`, `max`, `abs`, `collect`, `split_whitespace`, `to_lower`, `trim`, `get_or`, `entry` and
  `or_default` (R6 already names `counts.entry(k).or_default()` as a place), `iter`, `inspect` (the
  adapter), `fetch_add` and `load` on `Atomic`, `join`, `as_span`, `as_ptr`, `with_capacity`,
  `c_int`, `World`, `Component`, `Vec3`, `Neg`, `Add`. The `@ffi` contract words in AN-78 (`param`,
  `callback(retained, user_data=...)`, `destroys(f)`) are covered by the `@ffi` row and reader 03's
  contract-word table ("keep all").

## 6. Passed on, not changed (for 16, 17, 18 and the owner)

These are not word fixes I could make without deciding the model; each is a place to look.
1. **The type `fn()` does not say whether a call changes the function value's own state.** D3 asks for
   a changeable name to call a function value that changes its own state (`copied`/`owned` captures),
   and AN-54 refuses passing such a value to a read-only `f: fn()`. But a value stored in a field or a
   list of type `fn()` and called through a plain name (AN-53 `click(self): for h in self.handlers:
   h()`; AN-62 `self.on_visit(p)` from plain `self`) loses that knowledge. Either storing such a value
   into a `fn()` place needs a changeable name for later calls, or the call through a plain name must
   be refused for every stored value. For area D's owner and 16 (a function value's own state changed
   while another view of it is used).
2. **E3021 and E3023** (section 8) both mean "a change through a name that only reads", split by the
   kind of name (element name versus parameter, loop or pattern name), while R2 makes all of them
   read-only views. AN-84's rule is one meaning per code. For 18.
3. **`Pool` names three things in the NEW examples**: an interface (AN-40, the owner's own
   `no_share(dyn Pool)` example), a program's generic struct with `dense`/`sparse` (AN-22, AN-42, AN-67),
   and the standard library's generation-checked `Pool[T]` (the vocabulary row). They are program
   names, so not changed; a reader of the model may take one for another. For 18.
4. **`=` and `weak` links**: R2 copies a `weak` link bound from a part (`a = n.parent`), while R1 hands
   a whole `weak` name over (`w2 = w1` ends `w1`, since `weak` is not `Copy`). Stated in the `Copy`
   row; for 18 to judge against "never copies silently".
5. **AN-52 (a) `ws[i].target = shared players[r]`**: `shared` of a list element needs the element in a
   counted block; for a `class Player` that is its own block, for a struct element it is not stated.
   For area C's owner and 16.
6. **The call-site word** for an argument given to `ref mut` (clash 9) is still open for the owner;
   change 1 does not touch it (a call writes no mode; it may write a value word, `shared y` or
   `copied y`, as row 9 does).

## 7. Files

- `C:\Users\ism19\Code\ember\tasks\96\redesign\run3\combined-v2.md`: the model, with 22 lines carrying
  **15:** (changes 1-5, 7, 8, 9 and 18 of section 2).
- `C:\Users\ism19\Code\ember\tasks\96\redesign\run3\vocabulary-final.md`: the vocabulary, with 15's
  row changes marked **15:**, section 2a (15's changes) and section 6 (the spec's uses).
- Under `C:\Users\ism19\Code\ember\target\redesign3\15-vocabulary-review\`:
  `extract_new.py` (NEW blocks and their words; output `new_blocks.md`, `tokens.txt`),
  `scan_old_words.py` (old words in the new model's text; output `hits.txt`), `spec_uses.py` (spec uses
  of changed words; output `spec_uses.txt`), `check_tables.py` (seven cells per vocabulary row),
  `agreed_list.txt` (14's list of agreed spellings, checked by hand in section 1), `an79_no_fn_note.em`
  (change 6), `section6.md` (the text appended as `vocabulary-final.md` section 6). `target/` there is
  `ember check`'s own cache (277 KB).
