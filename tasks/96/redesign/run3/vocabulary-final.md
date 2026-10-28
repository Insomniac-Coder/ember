# The final vocabulary (run 3, phase 3; agent 14, combiner; reviewed by agent 15)

**This is `vocabulary-final.md`**: `vocabulary-combined.md` copied by agent 15 (vocabulary reviewer)
with `cp`, then edited. Every row 15 changed carries **15:** and says why; section 2a lists them;
section 6 (new) lists every spec use of a word whose spelling or meaning changes and what it becomes,
so no old use is left meaning something else. The model it goes with is `RUN3/combined-v2.md`. The
text below this note, up to section 2a, is 14's.

One vocabulary for the combined memory model (`RUN3/combined.md`). It is 06's approved table
(`RUN3/vocabulary.md`) with every row the model changes rewritten, the placeholders given their final
spellings (07, `RUN3/planned-words.md`), the areas' rulings applied, and the words of phases 2 and 3
added (section 3.7). Built by `target/redesign3/14-combiner/vocab.py`. A rewritten row keeps 06's
"today it means" cell and carries **14:** in its verdict; a row without **14:** is exactly 06's. "R1" ..
"R17" are the rules of `combined.md` section 1.1; "clash N" is its section 3. `RUN3` means
`tasks/96/redesign/run3/`. Inside the table, "section 5.x", "settlement N" and "C-N" are
`vocabulary.md`'s own sections 5, 2 and 6.1.

The history behind the rows (the readers' disagreements and how 05 settled them, 05's and 06's probes
with their real `ember check` output, and 06's changes C-1 .. C-10) is in `RUN3/vocabulary.md` sections 2,
5 and 6; 07's reasons for each new word and its probes are in `RUN3/planned-words.md`. The readers' line
lists of every use in the spec are in `RUN3/01-reader-1.md` .. `RUN3/04-reader-4.md`.

## 0. Terms used here

- **word**: anything a programmer writes to say something: a keyword (`owned`), an `@` attribute
  (`@noalloc`), or a library type or function that acts as a language word (`mem.take`).
- **the owner's test**: for every name in a line, the word that says what it is (the owner's own thing, a
  copy, or handed over) and the word that says whether it may change it (brief).
- **view**: a value that points into something else and may not outlive it (`ref T`, `Span[T]`,
  `StrView`, a plain parameter or loop name).
- **`Copy` type**: a type whose values are copied bit for bit on assignment and whose copy is
  independent: numbers, `bool`, `ref T`, `Span`, structs of such (06's C-3). Class objects, `shared`
  values and `weak` links are not `Copy`.
- **class object** (was "class handle" in 06's table): an object of a `class` type, in its own heap
  block, with one owner unless a line says `shared` (R1).
- Verdicts: **keep**, **rename**, **merge**, **remove**, **new**, as in `vocabulary.md`.

## 1. The agreed rows: every word they use keeps its agreed spelling and meaning

| row | words it uses | spelling and meaning, as agreed (unchanged here) |
|---|---|---|
| 3 | `fn`, `self`, `changes` | `fn note(self, msg: String) changes log:`: the method's first line says which fields it changes; checked against its body |
| 4 | `return`, `ref`, `with`, `lock()` | `return ref g[0]` inside `with g = scores.lock():` is refused: "this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block" |
| 5 | `String(...)` | `word = line.to_lower().trim()` keeps the lowered string until `word`'s last use; returning such a view stays refused: "return `String(...)` to hand back your own copy" |
| 6 | `fn`, `with`, `copied` | `b.on_click(fn(): score += 1)` changes `game`'s real `score`; a copy is made only when named: `fn() with copied score:` |
| 7 | `reads` | `fn load_assets(self) reads renderer -> Array[String]:`: the helper says what it reads; checked against its body and at the call |
| 8 | `ref mut` (and `mut` refused in a pattern) | `State.Walking(ref mut path):` stays; plain `path` then `path.pop()` gets "to change that list through `path`, write `State.Walking(ref mut path):`"; `State.Walking(mut path):` is refused with the same advice |
| 9 | `copied`, `shared` | `snapshot = copied items` copies; `snapshot = items` hands the list over; the error names both ways out (`copied`, or `shared`) |

The rows below that touch these words (`self`, `changes`, `ref`, `with`, `String`, `copied`, `reads`,
`ref mut`, `mut`, `shared`) repeat the agreed meaning and only add what the row leaves open; none
changes a spelling or a meaning the owner agreed. Today none of the new spellings compiles (section 5.3).

## 2. What changed from `vocabulary.md`

| word | change | by |
|---|---|---|
| `class` | keep the word: meaning set by area C under 06's condition (C-3): no second name for an object is made without a word | C (AN-44, C8) |
| `dyn` | keep: confirmed (area D) | D (AN-66, AN-67) |
| `fn` | keep: capture words confirmed (07, area D) | 07, D |
| `for` | keep: A-3 and its counting-loop rule (area A); the `while` loop (area D, clash 12) | A (A-3), D (AN-60), E (AN-77) |
| `mut self` | keep: confirmed (area B) | B (AN-18) |
| `owned` | keep: 06's C-2, C-3 and the owner's `Box` ruling, with area C's rulings (refused on a class type; `owned m: owned Mesh`) and 07's foreign-type position | C (AN-40, AN-43, AN-47), 07 |
| `ref` | keep: R2 (clash 1) | A, C (clash 1) |
| `ref mut` | keep: with area A (positions, AN-01, AN-03), area B (a call is a place, AN-23) and area C (kept views, AN-52 (b); clash 3) | A, B, C |
| `static` | keep: `static mut` made safe (area E); `from static` accepted as a new position (area B) | E (AN-76), B (AN-26) |
| `unsafe` | keep: the two directions are told apart by position and by their two notes; no second spelling (07, area E) | E (AN-79) |
| `unsafe fn` | keep: body rule ruled by area E | E (AN-79) |
| `with` | keep: the capture words and `with x.lock:` add uses, not meanings | 07, D, E |
| `yield` | keep: area E's rule | E (AN-77) |
| `from` | keep: final (07); the field and `static` positions are area B's | 07, B (AN-22, AN-25, AN-26) |
| `@sync` | **remove**: 06 renamed it `@thread_shared`; area E removed the marker (AN-80) | E (AN-80) |
| `@static_safe` | **rename**: final (07, area E, AN-81) | 07, E (AN-81) |
| `@fp` | **rename** the argument: final (07, area E, AN-83) | 07, E (AN-83) |
| `@borrows` | **rename**: final (07, area B, AN-22) | 07, B (AN-22) |
| `@no_runtime_checks` | **remove**: message by area E (AN-82) | E (AN-82) |
| `Box` | **merge** into `owned`: confirmed (area C, AN-40) | C (AN-40) |
| `Shared` | **merge** into `shared` (settlement 16): confirmed (area C) | C |
| `Weak` | **rename**: (07, area D): the link word sits beside `owned` and `shared` in the type | 07, D (AN-63, AN-65) |
| `RefCell` | **remove**: 06 renamed it `RuntimeChecked`; area C removed the type: nothing is left that needs a run-time borrow check | C (AN-49) |
| `Cell` | keep: with area B's condition | B (AN-20) |
| `str` | **rename**: final (07, area B) | 07, B (AN-26, AN-29) |
| `Handle` | keep: messages by area C | C (AN-50) |
| `Send` | keep: inferred (area E) | E |
| `Sync` | **rename**: 06's C-4; inferred (area E, AN-80) | E (AN-80) |
| `cstr` | **rename**: final (07) | 07 |
| `Ref` | **remove**: (area C) | C (AN-49) |
| `RefMut` | **remove**: (area C) | C (AN-49) |
| `borrow` | **remove**: (area C) | C (AN-49) |
| `borrow_mut` | **remove**: (area C) | C (AN-49) |
| `SyncShared` | **merge**: spelled by 07, used by area E | 07, E |
| `ForeignBox` | **merge** into `owned`: 07, area C; 06's placeholder `Foreign[T]` dropped | 07, C (AN-40) |
| `upgrade` | **remove**: 07, area D; 06's placeholder `get` dropped | 07, D (AN-65) |
| `assert_disjoint` | **rename**: final (07, area A) | 07, A (AN-14) |
| `Commands` | keep: `create()` added (area A) | A (AN-06) |
| `cast` | keep: one rule for areas B and D (clash 4) | B (AN-22), D (AN-67, AN-68) |
| `get_mut` | keep: area D's refusal | D (AN-61) |
| `shared` | **new** (row 9): area C's rules | C (AN-34, AN-39, AN-48) |
| `changes` | **new** (row 3, spelling agreed): free-function position (07, area E); order and E2105 (clashes 4, 8) | B, E (AN-76), clashes 4, 8 |
| `reads` | **new** (row 7, spelling agreed): confirmed (area B) | B (AN-17, AN-31) |
| `thread_shared` | added (section 3.7) | accepted; used by area E (AN-71, AN-74, AN-75, AN-80) |
| `weak` | added (section 3.7) | accepted; area D (AN-60 .. AN-65) |
| `set by` | added (section 3.7) | accepted, with area D's rules D7 |
| `@cached` | added (section 3.7) | accepted; area B (AN-21), area E (threads) |
| `guarded by` | added (section 3.7) | accepted (`combined.md` section 2) |
| `Lock` | added (section 3.7) | accepted (`combined.md` section 2); `Mutex[T]` stays for one value behind a lock |
| `Commands.create()` | added (section 3.7) | accepted (`combined.md` section 2) |
| `from static` | added (section 3.7) | accepted (`combined.md` section 2) |

## 2a. What 15 changed (vocabulary reviewer)

Each row below carries **15:** in the table, with its reason; the matching edits to the model are
marked **15:** in `combined-v2.md`. The test a row failed is one of the four of `vocabulary.md` 6.1:
(1) the line says what each name is and whether it may change it; (2) one meaning for every type;
(3) no word with two meanings, no two words for one; (4) reads right where it sits.

| word | change | test |
|---|---|---|
| `shared`, `thread_shared` | on a parameter the mode is on the name: `x: shared T` reads (a view, no count), `ref mut x: shared T` changes, `owned x: shared T` takes one owner; was "`x: shared T` makes the callee one more owner" (R4, C5) | (2), (3): the type word acted as a mode only for these two types, the form 06's C-2 refuses for `owned`; AN-78's NEW contradicted it |
| `owned` | the generic `Copy` instance and the type-versus-name places stated | (2): the row was silent on `owned v: T` with `T = int` |
| `Copy` | "class handles ... `Weak`" -> "class objects ... `weak` links"; R2's copied `weak` part noted | (3): old spellings in the new meaning |
| `Handle` | "handle" = a token for something elsewhere (`Handle[T]`, `JoinHandle`, `@ffi(handle)`), never an object or link | (3): the model's own text used it for a view (AN-24) and an object (D5) |
| `@view` | optional, checked | (3): R7 read as a required mark that AN-22 does not write |
| `str` (`StrView`) | the `from` rule is for a class's field; a struct holding a view is a view | (2): the row's refusal contradicted AN-22 |
| `SAFETY:` | `SAFETY(<category>):` form added; not on an `unsafe fn` line | (3), (4): one comment, two meanings (why a block is sound / what callers must ensure, which is `@safety`) |
| `@safety`, `unsafe fn` | `@safety` is an `unsafe fn`'s one note, asked of every `unsafe fn` | (3) |
| `override`, `virtual`, `changes` | an override's first line stays within the virtual's | (1), (3): a call through the base is checked against the base's line |
| `threads` | the `@export(threads=...)` place added | a word in the NEW code with no row for its place |
| `Commands` | "where it is written": `q.commands()` | (4): the row named a place the spec does not use |
| `SoA.get` | `load()` at a use refused, as AN-13 says | the row said "as today" |
| `get_mut` | `Shared.get_mut` removed with `Shared` | (3): "same" kept a removed type's method |
| `CppShared` | 07's ruling written in | the row still said "07 decides" |
| `.to_string()` | **new row**; the model writes `String(s)` | (3): two spellings of one meaning in the NEW code |
| `ForeignCallback` | **new row** | a word in the NEW code with no row |
| `thread.spawn`, `range` | **new rows**: the capture-list rule of R14 on `thread.spawn`; R8 (c)'s counting-loop rule on `range(n)` | words in the NEW code, each carrying a rule of the model, with no row |

## 3. The table

Columns: **word**; **today it means** (with spec lines); **it will mean**; **where it is written**;
**replaces** (which of today's words it replaces, if any); **stays refused**; **verdict and reason**.
"Same" in "it will mean" = unchanged. "As today" in "stays refused" = no change to what is refused.

### 3.1 Reserved keywords (II.4, 47), and the phrases `ref mut` and `unsafe fn`

| word | today it means | it will mean | where it is written | replaces | stays refused | verdict and reason |
|---|---|---|---|---|---|---|
| `and` | logical and (467, 695) | same | between two conditions | — | as today | keep: one meaning, Python's |
| `as` | import alias (503); conversion `x as T` (705, 938-947; narrowing keeps the low bits) | same | after an import name; after a value | — | as today; `T.try_from` stays the checked conversion | keep: two uses, each plain English (as Python's `import ... as`) |
| `break` | leave a loop (668, 713) | same | in a loop | — | as today | keep |
| `class` | a type whose values are shared, counted handles to one heap object (261, 854, 1162) | an object in its own heap block, with identity (`is`) and subclasses (`open`, `virtual`), and an 8-byte header (a 32-bit count, a 32-bit type index); **one owner unless a line says `shared`**, as every type: `p2 = player` hands the object over, `p2 = shared player` makes a second owner, `p2 = copied player` a new object (combined R1, R3; area C's C8, AN-44) | before a type name | the meaning "a shared, counted handle" (`[OBJ-1]`, `[RC-1]`) | using the name after it was handed over (E3040, naming `copied` and `shared`); `owned` on a class type ("a `Player` object always lives in its own block; remove `owned`") | keep the word **14:** meaning set by area C under 06's condition (C-3): no second name for an object is made without a word |
| `comptime` | code run when compiling (584, 739, 3991-4016) | same; a heap value copied at each run-time use (4038-4040) gets a written word (AN-46) | before a block or in `comptime(e)` | — | as today | keep |
| `const` | constant fixed when compiling; const generic parameter (549, 577, 1197) | same | before a name | — | as today | keep |
| `continue` | next round of the loop (691) | same | in a loop | — | as today | keep |
| `defer` | run a block when the enclosing block exits (655, 2015) | same | before a block | — | as today | keep |
| `dyn` | an interface value whose concrete type is chosen while running, behind a pointer (608, 857, 1435) | same; who owns the value is written: `owned dyn I` (one owner; replaces `Box[dyn I]`), `shared dyn I` (several owners, counted), `dyn I` as a parameter's type (a read-only view; `ref mut v: dyn I` may change it). A generic method whose type parameter is only the type of a `ref`/`ref mut` parameter gets one `dyn` entry (R13; area D, D8, AN-66, AN-67) | before an interface name, in a type | `Box[dyn I]`; a bare interface name used as a type | a bare interface name as a type ("say who owns the shapes: `Array[owned dyn Shape]` or `Array[shared dyn Shape]`"); `dyn I` of an interface whose generic method returns or stores its type parameter (E2050, with the line to write) | keep **14:** confirmed (area D) |
| `elif` | else-if (645) | same | after an `if` block | — | as today | keep |
| `else` | `if`'s else; loop `else` (runs when no `break`); conditional expression (645-648, 693) | same | after a block | — | as today | keep: Python's three uses |
| `enum` | a type with variants (565, 852) | same | before a type name | — | as today | keep |
| `extern` | defined outside, C calling convention: foreign block, `extern "C" fn` type, `extern type`, `extern static` (544, 615, 4728-4731) | same | before a block, a type or a function | — | as today | keep |
| `false` | the `bool` value false | same | as a value | — | as today | keep |
| `fn` | function, method, lambda, callable type (546, 615, 692) | same; a lambda's capture list follows its parameters and names, one word per name, what the function holds: `with copied x`, `with owned x`, `with shared x`, `with thread_shared x`, `with weak x`; a name not listed is the real variable (row 6), whose place (an address, or a one-thread counted box) the compiler chooses without changing the meaning. A lambda parameter's mode is what is written on it (`fn(owned s)`, `fn(ref mut ui)`); a plain one reads (R3, R12; area D, D1, D2, AN-56) | before a function or lambda | the `owned fn` prefix | `owned fn(...)` ("name what the function takes or copies on each name: `fn() with owned name, copied score:`"); `with copied a, b` ("say what the function does with `b`"); `with ref mut x` ("remove `ref mut x`: the function already uses the real `x`"); `with owned x` of a `Copy` value ("write `with copied x`") | keep **14:** capture words confirmed (07, area D) |
| `for` | loop over an iterable; the loop name is a read-only reference to each element, but for class elements a handle that may write the object (648, 1966-1997, 2566) | same: the loop name reads each element; `for ref mut v in xs:` may change it (C-1). The list may not change while the loop walks it, directly or through any call that may change it (the owner's limitation; R8 (c)); `for i in 0..xs.len():` (its range fixed when it starts) may add to `xs` but not remove from it. A `while` loop that tests its position against the current length is not a walk and may do both. In a generator, a `for` over a list walks by position across `yield` (area E, AN-77) | before a loop | — | a change to the walked list inside the loop ("`enemies` may not change while this loop walks it; ... after the loop: `enemies.retain(fn(e) => e.hp > 0)`"); a removal inside `for i in 0..xs.len():`; `for v in xs.iter_mut():` ("write `for ref mut v in xs:`") | keep **14:** A-3 and its counting-loop rule (area A); the `while` loop (area D, clash 12) |
| `if` | condition; `if pattern = e`; guard; conditional expression; comprehension filter (645-652, 693, 749) | same | before a condition | — | as today | keep |
| `implements` | a type implements an interface (556, 1229) | same | in a type header or `extend` | — | as today | keep |
| `import` | import a module; `import c "h.h"` (503-505) | same | at the top of a file | — | as today | keep |
| `in` | membership `x in c`; `for x in`; range type `f32 in 0.0..=1.0` (579, 648, 697) | same | between two values; in `for`; in a type alias | — | as today | keep: three uses, each English "in" |
| `interface` | declares an interface (571) | same | before an interface name | — | as today | keep |
| `is` | identity, `is None`, `is not` (466, 697); method name `is[E]()` (397) | same | between two values | — | as today | keep |
| `let` | a class field assignable only in `init`; its contents may still change (`self.items.push(v)`, `[CLS-9]` 1748, `[CLS-9a]` 1749); a local `let x = 5` is refused (E0100, good help) | **06 (stated as one rule):** after `init`, **nothing is changed through this field**, for every type: a number or struct field never changes; a list field is never pushed to; a class-object field is never re-pointed and its object is never changed through it (other owners of a `shared` object are `shared`'s business, not `let`'s) | before a field | — | assigning or changing a `let` field outside `init`, with a message that gives the line (E2104's new help: "set it in `init`, or remove `let` from its declaration") | keep the spelling, change the meaning (settlement 4): fails "says whether it may change, for every type" today (01, 02, 04; 02's t2 accepts `push`) **06: keep; meaning reworded so it is one rule for every type** (the draft said "contents included" for a list but only "never re-pointed" for a handle field: two meanings by type; C-8) |
| `match` | pattern match; `match owned e:` binds by move (651, 839) | same | before a value | — | as today; `mut` in a pattern (row 8) | keep |
| `mut` | in-out parameter (the caller's own value changes, 1558); a class-handle parameter whose handle the callee may re-point (`[FN-9]`); `mut self`; `static mut`; inside `ref mut` and `*mut T`; a lambda parameter that writes (716); accepted in a pattern though the grammar lacks it, binding read-only (01 t1) | **06:** "may change", and nothing else, in exactly these places: `mut self` (the word `self` says what it is: the value the method was called on), `ref mut` (row 8), `static mut`, `*mut T`. **Not on a parameter's or lambda parameter's name**: the in-out parameter is written `ref mut b: B` (C-1) | in `mut self`; after `ref`; after `static`; after `*` | — | `mut` in a pattern (row 8: "write `State.Walking(ref mut path):`"); **06:** `mut b: B` on a parameter and `fn(mut n)` on a lambda parameter, with "to change the caller's `b` through `b`, write `ref mut b: B`" (the same advice shape as row 8); `mut self` on `init` ("`init` may always set fields; write `fn init(self, ...)`") | **06: keep the word, move its parameter use to `ref mut` (C-1).** Row 8's own test, applied to a parameter: in `fn bump(mut b: Counter)` the word `mut` says "may change" and no word says "the caller's own `b`", exactly as `State.Walking(mut path):` failed (in Rust `fn f(mut b: B)` is a changeable copy). Today `mut` on a struct parameter changes the caller's fields and on a class parameter re-points the caller's handle and "opens no access on the object" (`[FN-9]` 1570), while `mut self` on a class changes the object's fields (settlement 20): two meanings by type (`k4_mut_param_two_meanings.em`, accepted). With `ref mut` on the name the one meaning is row 8's: a name for someone else's place that may change it |
| `mut self` | struct: may change any field (1633); class: holds a write access on every field for the whole call (`[CLS-7]` 1740, `[EXC-15]`); `init`: accepted, means nothing more (1658, 1717) | **may change any field of the receiver, for every type**; `changes <fields>` (row 3) narrows it; checked when compiling only: no access words while running (area B, AN-18) | on a method's receiver | the class meaning (whole-call write access) | `mut self` on `init` (02); on a `@sync`/`@thread_shared` class (E7003, as today) | keep **14:** confirmed (area B) |
| `not` | logical not; `not in`; method name `not` (467, 697, 396) | same | before a condition | — | as today | keep |
| `or` | logical or | same | between conditions | — | as today | keep |
| `override` | this method replaces a virtual one; an overlay entry replacing an earlier one (545, 4762) | same; an override's first line says no more than the virtual's: it changes only what the virtual's `changes` or `mut self` lets it (R5) | before `fn` | — | an override that changes more than the virtual says ("`Door.on_update` changes `angle`, but it overrides `Script.on_update`, whose first line changes nothing, and callers of `Script` rely on that; write `virtual fn on_update(mut self, dt: float):` in `Script`") | keep **15:** a call through the virtual is checked against the virtual's first line, so without this an override could change what the base's line says it does not, and `changes` would not mean what it says there (the spec's own `Door` example, 1704 and 6160, writes `angle` from a `virtual fn on_update(self, ...)`) |
| `owned` | (a) parameter "consumed": moved in, copied if `Copy`, a class handle copied without a retain (`[FN-1]` 1561) or as a retained copy the caller keeps (`[FN-9]` 1573); (b) `owned self`; (c) `for x in owned xs`, `match owned e`; (d) `fn(owned T)` callable-type parameter; (e) the `owned fn` lambda prefix (row `owned fn`); (f) C contract word "callee takes it" (4826); (g) a C++ `T&&` parameter (6592); (h) a lambda parameter inferred `owned` (`[TYP-23]` 1468) | **ownership handed over, nothing else**, for every type that is not `Copy`: an `owned` parameter (`owned x: T`), receiver, `for`/`match` source, capture (`with owned x`). In a type, **one owner in its own heap block**, replacing `Box[T]`, `Box[dyn I]` and `ForeignBox[T]` (`mesh: owned Mesh`, `Option[owned Tree]`, `owned dyn I`, `db: owned SqliteDb` for a foreign type with `@ffi(destroys(f))`); a plain value stored into an `owned T` place is put in its block at that store. `owned m: owned Mesh` takes over a one-owner block and keeps it (R3, R4; area C, C3 to C5). **15:** in generic code (`fn emplace[T](self, e: int, owned value: T)`, AN-42) a `Copy` instance is copied bit for bit and the caller's name stays usable: the two values are independent and equal, so nothing the program computes tells it from a hand-over (06's C-2, which keeps generic code working); `mem.drop` ends the name for every type by its own rule. On a parameter `owned` is on the name and says what the call does; in the parameter's type it says only how the value is held (`owned x: shared T` takes one owner of a shared value) | on a parameter's name, a receiver, a `for`/`match` source, a field or type (a foreign type included), a capture name | `Box[T]`, `Box[dyn I]`, `ForeignBox[T]`, the `owned fn` prefix | a name used after it was handed over (row 9's message); `owned` before a value (E0109: "`b = a` already hands `a` over"); `owned` where the type is known to be `Copy` (E2085: "an `int` is copied, not handed over; remove `owned`"; a capture: "write `with copied score`"); `x: owned T` as a parameter's type with no mode on the name; `owned` on a class type ("a `Player` object always lives in its own block; remove `owned`") | keep **14:** 06's C-2, C-3 and the owner's `Box` ruling, with area C's rulings (refused on a class type; `owned m: owned Mesh`) and 07's foreign-type position |
| `owned fn` | a lambda that moves its non-`Copy` captures and copies its `Copy` ones (`[CLO-2]` 2064; 692, 3495, 6221); accepted silently for an `int` (01 t2, 02 t1, 03 t3) | nothing: removed. A capture is the real variable unless named: `with copied x` (row 6) or `with owned x` (placeholder) | — | — | `owned fn(...)` itself, refused with "name what the function copies or takes: `fn() with copied score:`" | **remove**: the owner's named failure (JOB): one keyword copies an `int` and hands over a `String`, and sits on `fn`, not on the captured names |
| `pass` | empty statement or body (560, 633) | same | as a statement | — | as today | keep |
| `pub` | visible to dependants; `pub(package)`, `pub(read)` (538) | same | before an item or field | — | as today | keep |
| `ref` | a reference type `ref T` (read-only); a borrow expression `ref e`; a pattern `ref x`; a result `-> ref T` (607, 706, 828, 1112, 1336); as a parameter's type it means the same as no word (01 t6) | **a name for someone else's place that only reads it** (rows 4, 8). Binding a name to a part with plain `=` is already a read-only view (combined R2), so `ref` before a part states that default and is accepted; it is needed where the default would copy (a number: `r = ref counts[0]`) | before a type, a place or a pattern name | — | `x: ref T` and `ref x: T` on a parameter ("a parameter already reads the caller's value; write `x: T`"); `return ref g[0]` out of a lock (row 4); `r = ref b` on a reference name (AN-28) | keep **14:** R2 (clash 1) |
| `ref mut` | a reference that may change what it points at: type `ref mut T`, expression `ref mut e`, pattern `ref mut x`, result `-> ref mut T` (607, 706, 828, 1112-1117, 1906, 2280-2320; 52 uses in lines 1-6930); `r = e` on a `ref mut` local writes through it (`[BRW-1]` 2286, `[TYP-14]` 1116); as a parameter's type it cannot be written through (01 t3) | **a name for someone else's place that may change it** (row 8), in every position: a pattern name, an expression, a result (`-> ref mut T`), a local, a parameter's name (`ref mut b: B`, the in-out parameter), a lambda parameter, a callable type's parameter, a loop name (`for ref mut v in xs:`), an ECS query (`Query[(ref mut Position, Velocity)]`), a view kept in a struct or list. A call whose result type is `ref mut T` is a place (`first(xs) += 1`). Two `ref mut` arguments of one call, or one and another view argument, are different places, proved or compared once (R4); kept `ref mut` views may name one object, checked by R8 (b) (R9). An element name made with `ref mut xs[i]` is a position (R8 (a)) | before a type, a place, a pattern name, a parameter or loop name, a component in a query | `Mut[T]` in ECS queries; the in-out parameter `mut b: B`; `for v in xs.iter_mut():` in a loop header | `x: ref mut T` as a parameter's type ("write `ref mut x: T`"); `r = ref mut b` on a reference name ("`r` always refers to `a`; this copies `b`'s value into `a`; to refer to `b`, use a new name: `rb = ref mut b`"); one place passed to two `ref mut` parameters, known when compiling ("pass two different bodies, or write a one-body version") | keep **14:** with area A (positions, AN-01, AN-03), area B (a call is a place, AN-23) and area C (kept views, AN-52 (b); clash 3) |
| `return` | return from a function | same | as a statement | — | returning a view of a temporary (row 5) or into a lock (row 4) | keep |
| `self` | the receiver: read-only on a struct (02 t14), may write fields on a class (`[CLS-7]`; 01 t5, 02 t4); the frame-held handle in a `gen fn` | **the receiver, read-only for every type**; a method that changes fields says so with `changes <fields>` (row 3) or `mut self` (any field) | first parameter of a method | — | writing a field through plain `self` on a class (new): "`open` changes `self.angle`; write its first line as `fn open(self) changes angle:`" | keep the word (all readers); the class default is the change row 3 makes (AN-19) |
| `Self` | the implementing type (609) | same | in a type | — | as today | keep |
| `static` | lives for the whole program: a static item, the static region (578, 1130, 3478) | same; plus **`static mut NAME: T = e`**, a module variable that may change, with no `unsafe` (a foreign `static mut` keeps `[STA-1]`'s `unsafe`): a function whose body changes it says `changes NAME`; it is reached only by the main thread and its scoped tasks unless `guarded by` a `static` `Lock` (R14; area E, AN-76). And **`from static`** after a view field's type: the view points only into text or data written in the program (`[LT-3]`) (R7; area B, AN-26) | before an item; after `from` | — | a function that changes a module variable its first line does not name (E2105); a `static mut` not `guarded by` a lock reached from another thread (E7012, naming `guarded by` and `Atomic`); a `from static` field given a view of something else ("`kind` holds only text written in the program, and `name` comes from the caller; write `self.kind = String(name)` with `kind: String`") | keep **14:** `static mut` made safe (area E); `from static` accepted as a new position (area B) |
| `struct` | a value type (556, 852) | same | before a type name | — | as today | keep |
| `super` | the base class (`super.init(...)`, 1728) | same | in a derived class | — | as today | keep |
| `true` | the `bool` value true | same | as a value | — | as today | keep |
| `type` | type alias, range type, associated type, opaque foreign type (574-583, 1024, 4728) | same | before a name | — | as today | keep |
| `unsafe` | a block where the programmer vouches for what the compiler cannot check (656); on `unsafe fn` / `unsafe fn(...)` / `unsafe interface` / `unsafe extern` / `unsafe extend` / `unsafe overlay`: whoever calls, implements or relies on it must vouch (149-154, 545, 571-576) | one word, two places, one meaning each: before a declaration (`unsafe fn`, `unsafe interface`, an `unsafe fn(...)` type) "whoever uses this must vouch", with `@safety` saying what they must make true; before a block or an implementation (`unsafe:`, `unsafe extend`, `unsafe extern`, `unsafe overlay`) "this code vouches", with `# SAFETY:` saying why (R15; area E, AN-79) | before a block or a declaration | — | an unsafe operation outside `unsafe:` (E3100), in an `unsafe fn` body too | keep **14:** the two directions are told apart by position and by their two notes; no second spelling (07, area E) |
| `unsafe fn` | a function whose caller must vouch; its whole body is also an unchecked zone with no mark per operation (`[TIER-1]` 152; glossary 6916) | same declaration; inside its body every unsafe operation still sits in its own `unsafe:` block with a `# SAFETY:` note (Rust 2024's rule, made an error, E3100); `@safety` asked of every `unsafe fn` (L3015), not only `pub` ones, and it is the declaration's only note (**15:** no `# SAFETY:` on the `unsafe fn` line itself; see `SAFETY:`) | before `fn` | — | calling it outside `unsafe`; an unsafe operation in its body outside an `unsafe:` block (E3100) | keep **14:** body rule ruled by area E |
| `virtual` | the method may be overridden; bodiless only in an `abstract class` (545, 589) | same; its first line is what every override must keep to (a virtual whose overrides change their object says `mut self`; **15:**, see `override`) | before `fn` | — | `virtual` in a class that cannot be subclassed (today W2111, accepted; AN-70): refused with "write `open class C` or remove `virtual`" | keep (settlement 5) |
| `void` | the unit type; its value is `()` (609, 1615) | same | in a type | — | as today | keep |
| `where` | generic bounds clause (551) | same | after a signature | — | as today | keep |
| `while` | loop | same | before a loop | — | as today | keep |
| `with` | (a) bind for a block and end it at the block's end (`with g = m.lock():`, `with scope = thread.scope():`, bare `with debug.zone("x"):`, `with h:`; 653, 2013, 2663, 3422, 4184); (b) options of `import c ... with (...)` (510, 4682) | (a) bind for a block and end it at the block's end (`with g = m.lock():`, `with scope = thread.scope():`), and the bare form "hold this until the block ends" (`with debug.zone("x"):`, `with h:`, and `with x.lock:` for a `Lock`, area E); (b) options of `import c ... with (...)`; (c) the capture list of a lambda (row 6): `fn() with copied score:`, and 07's `with owned x`, `with shared x`, `with thread_shared x`, `with weak x`, one word per name; a name not listed is the real variable | before a block; after an import; after a lambda's parameters | — | as today | keep **14:** the capture words and `with x.lock:` add uses, not meanings |
| `yield` | give a value from a generator (691, 2152) | same; a `for` over a list, string, map or set whose body yields walks by position, reading the collection again after each resume, so no address is held across the suspension (area E, AN-77) | in a `gen fn` | — | a reference to the generator's own inline local across `yield` (E2221, with the line to write); the in-place consumer changing the walked field (E3020) | keep **14:** area E's rule |

### 3.2 Contextual and reserved-for-later words (`[LEX-15]`, 19), and the phrase `some I`

| word | today it means | it will mean | where it is written | replaces | stays refused | verdict and reason |
|---|---|---|---|---|---|---|
| `abstract` | `abstract class`: has bodiless `virtual` methods, cannot be built (557) | same | before `class` | — | as today | keep |
| `open` | `open class`: may be subclassed (557) | same | before `class` | — | as today | keep |
| `from` | `from m import x` (504); C result lifetime `from(self)` (4669); conversion `From[E]`, `String.from(s)` | every use reads "comes from": `from m import x`, C's `from(self)`, `From[E]`; and the clause that says what a view points into: after a result type (`-> StrView from src`, replaces `@borrows`; without it a method's result points into its object, a function's into the parameter it could come from), after a view field's type naming a `let` field of the same object whose contents live in their own heap block (`words: Array[StrView] from text`), and `from static` (text written in the program) (R6, R7; area B) | at the start of an import; after a result type; after a view field's type | `@borrows` | a result pointing into something its `from` does not name (E3062, with the line); `from` naming a field that is not `let` ("`words` points into `text`, so `text` must never change after `init`; write `let text: String`"); `from` on a fixed array or inline struct field; a parameter and a field of one name ("rename the parameter") | keep **14:** final (07); the field and `static` positions are area B's |
| `extend` | add methods or an interface implementation to a type (576, 1237) | same | at the start of an item | — | as today | keep |
| `gen` | `gen fn`: a generator (545, 778) | same | before `fn` | — | as today | keep |
| `once` | `once fn`: may be called at most once; `callback(once)` (615, 4836) | same | before `fn` in a type | — | a closure that gives a capture away passed where it may run many times, with the help "declare the parameter `once fn()`" (AN-59) | keep |
| `safe` | `safe fn` in an extern block: the programmer asserts the call cannot break memory safety (581, 4594) | same | before `fn` in an `unsafe extern` block | — | as today | keep |
| `some` | at the start of a type: `some I` (392, 608) | same | at the start of a return type | — | see `some I` | keep |
| `some I` | "one concrete type, chosen by the function body, that implements `I`"; the caller sees only `I`, the compiler monomorphises through it (`[TYP-32]` 1443-1451; `[GRM-31]` 621-624; examples 1450, 2175, 3373, 6201); only in a return type (E2260), the same type at every `return` (E2261) | same | in a function's or method's return type | — | `some I` outside a return type (E2260, its help gives the line: "take a type parameter bounded by the interface: `fn f[T: I](x: T)`"); different types at two `return`s (E2261, its help says "return a `Box[dyn I]`", which becomes "`owned dyn I`") | keep, judged here (section 5.2): one meaning, plain English ("returns some iterator"), on the type it describes; it names no ownership or change, so the owner's test has nothing to ask of it |
| `c` | the C language: `import c "h"`, `@layout(c)`, `c"..."` literal (505, 4220) | same | after `import`/`overlay`; in `@layout` | — | as today | keep |
| `cpp` | the C++ language: `import cpp "h"` (529, 6458) | same | after `import`/`overlay` | — | as today | keep |
| `language` | `#! language "v"` directive (108, 501) | same | after `#!` | — | as today | keep |
| `threads` | `#! threads main` (or `any`, `creator`); C contract `threads(...)` (520, 4608); `@export(threads=any \| main \| creator)` (`[FFI-33]` 4964) | same | after `#!`; in `@ffi`/overlay; in `@export(threads=...)` | — | as today | keep **15:** the `@export` place added (AN-84's NEW writes `@export(threads=any)`; reader 03 listed it) |
| `overlay` | overlay file over a C or C++ header (154, 4774) | same | at the start of an item | — | a fact-stating overlay not marked `unsafe overlay` (E5067) | keep |
| `rename` | inside an overlay: rename a foreign name; `@ser(rename=...)` (4789, 1871) | same | inside an overlay; in `@ser` | — | as today | keep |
| `hide` | inside an overlay: hide a foreign name (4790) | same | inside an overlay | — | as today | keep |
| `union` | reserved at the start of an item; `@layout(c, union)` (395, 4727) | same | at the start of an item | — | a `union` item (E0005, reserved) | keep |
| `async` | reserved for a later version (401) | same | — | — | as today (E0005); AN-77 | keep |
| `await` | reserved for a later version (401) | same | — | — | as today (E0005) | keep |
| `macro` | reserved for user derives (401, 4105) | same | — | — | as today (E0005) | keep |

### 3.3 Attributes (every `@name` of lines 1-6930 on the must-cover list)

| word | today it means | it will mean | where it is written | replaces | stays refused | verdict and reason |
|---|---|---|---|---|---|---|
| `@sync` | a class whose handles may cross threads, whose counts are atomic and whose fields are fixed after `init` (`[THR-1]`, `[THR-7]` 3451-3475) | nothing: **removed**. What it said on the class is said on the value where it is shared (`thread_shared Stats()`) and on each field that changes (`guarded by lock`, `Atomic`) | — | — | `@sync class Stats` ("`@sync` is gone: a class needs no mark to be shared between threads; share the object where it is shared, `st = thread_shared Stats()`, and say which lock guards each field that changes (`words: int guarded by lock`) or make it `Atomic[int]`") | **remove** **14:** 06 renamed it `@thread_shared`; area E removed the marker (AN-80) |
| `@ffi` | one attribute for every fact about foreign code: `effects=`, `param(...)`, `result(...)`, `link_name=`, `handle`, `destroys(f)`... (1808, 3208, 4593-4884, 6511) | same | on a foreign item | — | as today | keep |
| `@derive` | ask the compiler to write `Copy`, `Clone`, `Eq`... (254, 1623) | same | on a type | — | `@derive(Copy)` with a non-`Copy` field (E2080, message to give the line) | keep |
| `@noalloc` | contract: no heap allocation (140, 3239) | same | on a function | — | as today | keep |
| `@parallel` | loop runs in chunks on the job system; `chunk=`, `reduce=` (679, 3577) | same | on a `for` | — | a parallel loop writing a shared place (E7010; AN-84 for its code) | keep |
| `@static_safe` | contract: no run-time *aliasing* checks; bounds, overflow and stale checks still run (`[EFF-12]` 3245, 3260) | the same contract, spelled **`@no_alias_checks`**: no run-time alias check inside, which in this model means no compare of two `ref mut` arguments (R4) and no `assert_disjoint`/`check_disjoint`; bounds and overflow checks still run | on a function | — | `@static_safe` ("is now `@no_alias_checks`: it forbids run-time alias checks only"); a run-time alias check inside (E4030, naming the call and how to prove it) | **rename** **14:** final (07, area E, AN-81) |
| `@nopanic` | `@nopanic(explicit)`: no written panic; the bare form is reserved (E0104, good help) (3244, 3263) | same | on a function | — | the bare form, as today | keep |
| `@layout` | `@layout(c)`, `(c, union)`, `(c, flexible)`: C field order (1100, 4727) | same | on a struct | — | as today | keep |
| `@deterministic` | contract: no nondeterministic effect (148, 3307) | same | on a function | — | as today | keep |
| `@simd` | ask for vectorisation; `@simd(assert)` makes failure an error (681, 3729) | same | on a `for` | — | as today | keep |
| `@fastmath` | relaxed float rules in one function (1002, 3316) | same | on a function | — | inside `@deterministic` (E4072) | keep |
| `@ser` | serialisation options: `skip`, `rename=`, `default`, `since=` (1871, 4076) | same | on a field | — | as today | keep |
| `@fp` | `@fp(contract)`: allow fused multiply-add (1003, 3332, 4492) | same, the argument spelled **`@fp(allow_fma)`**: the compiler may fuse `a * b + c` into one fused multiply-add (`fma` is the library's own name for it) | on a function | — | `@fp(contract)` ("is now `@fp(allow_fma)`"); `@fp(allow_fma)` inside `@deterministic` (E4072) | **rename** the argument **14:** final (07, area E, AN-83) |
| `@borrows` | the result borrows exactly these parameters (2372-2398) | nothing on its own line: the `from` clause after the result type (`-> StrView from src`) | after the result type, as `from` | — | `@borrows(...)` on its own line ("write `from a` after the result type") | **rename** **14:** final (07, area B, AN-22) |
| `@noreload` | hot reload: this function is never reloaded (1866, 6311) | same | on a function | — | as today | keep |
| `@must_drop` | the value's `drop` must run: no `forget`, no `yield` while live (1861, 3533) | same | on a type | — | leaking it (E3015) | keep |
| `@export` | C symbol for an Ember function (1853, 4940) | same | on a function | — | as today | keep |
| `@view` | checked documentation that a type holds a view (1121-1124, 2325) | same: **optional**, checked documentation; a struct holding a view is a view whether or not it says `@view`, because its view fields' types already say so (`src: StrView`); `ember fmt --migrate` must not remove it (04: `[CLI-20]` 5067 contradicts `[TYP-34]`) | on a type | — | `@view` on a type that holds no view (E2030, help: "remove `@view`") | keep **15:** "optional" written into the row: 14's R7 said "A struct that holds a view is `@view`", which read as a required mark, while AN-22's `Parser` has none; R7 is reworded in `combined-v2.md` (a required mark would say a second time what the field types say, the reason area E gave for removing `@sync`) |
| `@reflect` | run-time type information (1858, 4048) | same | on a type | — | as today | keep |
| `@nosync` | contract forbidding the effect `Sync`: atomics stronger than relaxed, locks, channels, thread spawn/join, retaining a `Sync` handle (140, 3184, 3240) | same | on a function | — | as today (E4002) | **06: keep (C-4; the draft renamed it to `@nosynchronise`).** "no sync" reads as "no synchronisation", which is exactly what it forbids (locks, atomics stronger than relaxed, channels, spawn and join, 3184-3186). The draft's reason, that it reads as "not `@sync`", goes away once `@sync` is `@thread_shared`; renaming the effect to `Synchronise` instead would keep `Sync` (the property) and `Synchronise` (the effect) as one word and its abbreviation with two meanings |
| `@repr` | the integer type of an enum (1101) | same | on an enum | — | as today | keep |
| `@packed` | no padding in a struct (3055) | same | on a struct | — | a reference to a packed field (E2170) | keep |
| `@overflow` | `wrap` / `saturate` / `panic` for the operators in a function (956) | same | on a function or module | — | as today | keep |
| `@nolock` | contract: no lock (3243) | same | on a function | — | as today | keep |
| `@noblock` | contract: no blocking operation (148, 3242) | same | on a function | — | as today | keep |
| `@align` | alignment of a type (3055) | same | on a type | — | as today | keep |
| `@realtime` | the manifest's `[realtime]` contract set (198, 3247, 5151) | same; `ember inspect` shows the set it stands for (01) | on a function | — | as today | keep (settlement 8) |
| `@noio` | contract: no I/O (3241) | same | on a function | — | as today | keep |
| `@inline` | binding inline request (1849, 5925) | same; a note where it has no effect (04) | on a function | — | as today | keep |
| `@attribute` | marks a struct usable as a user attribute (1828, 4051) | same | on a struct | — | as today | keep |
| `@unroll` | unroll a loop (679) | same | on a `for` | — | as today | keep |
| `@safety` | what callers of an `unsafe fn` must ensure (1860, 5850) | same; the one note an `unsafe fn` carries, asked of every `unsafe fn`, not only `pub` ones (AN-79) | on an `unsafe fn` | — | an `unsafe fn` without it (L3015, now for every `unsafe fn`); `TODO` left in (L3016) | keep **15:** "every `unsafe fn`" written into the row (it was only in the `unsafe fn` row) |
| `@export_table` | a table of function pointers for C (2198, 4940) | same | on an item | — | as today | keep |
| `@error` | an error variant's display text (1869, 3857) | same | on a variant | — | as today | keep |
| `@test` | a test function (1855) | same | on a function | — | as today | keep |
| `@renamed_from` | hot reload: the old name of a field (6381) | same | on a field | — | as today | keep |
| `@reinit_on_reload` | hot reload: re-run this static's initialiser (6347) | same | on a static | — | as today | keep |
| `@never_specialize` | one shared body for a generic (1865, 6011) | same | on a generic function | — | where one body cannot serve every type (E2223) | keep |
| `@gpu` | reserved (1876, 6729) | same | — | — | as today | keep (reserved) |
| `@assume_noalloc` | inside `unsafe` only: treat a call as allocation-free (740, 1873, 3269) | same | on an expression inside `unsafe` | — | outside `unsafe` | keep ("assume" says it is a promise); 02's placement note (ATT-2/ATT-4) goes to 15 |
| `@allow` | silence named warnings (680) | same | on an item or a statement | — | as today | keep |
| `@soa` | `@soa(flatten)`: split a struct field into columns (1864, 3677) | same | on a field | — | as today | keep |
| `@should_panic` | a test that must panic (1855) | same | on a test | — | as today | keep |
| `@reloadable` | hot reload: this function may be reloaded (1866, 6361) | same | on a function | — | as today | keep |
| `@non_exhaustive` | an enum may gain variants; any C value may arrive (1863, 4724) | same | on an enum | — | as today | keep |
| `@noinline` | do not inline (1849) | same | on a function | — | as today | keep |
| `@no_derive` | opt out of an implicit implementation (1842) | same | on a type | — | as today | keep |
| `@must_use` | the result must be used (1851, 3919) | same | on a function or type | — | as today (W2190) | keep |
| `@hot` | hint: often called (1849) | same | on a function | — | as today | keep |
| `@grade` | trust grade of an overlay fact (1872, 6625) | same | on an overlay fact | — | as today | keep |
| `@gpu_layout` | GPU buffer layout `std140`/`std430` (1845, 6724) | same | on a struct | — | as today | keep |
| `@from` | an error variant converts from its field's error type (1869, 3930) | same | on a variant | — | as today | keep |
| `@deprecated` | `@deprecated(since, note)` (113, 1852) | same | on an item | — | as today | keep |
| `@default` | an enum's default variant (1870, 4097) | same | on a variant | — | as today | keep |
| `@component` | `@component(layout=soa)`: an ECS component's layout (1864, 3815) | same | on a struct | — | as today | keep |
| `@cold` | hint: rarely called (1849) | same | on a function | — | as today | keep |
| `@always_specialize` | one body per instance (1865) | same | on a generic function | — | as today | keep |
| `@allow_reload_terminate` | hot reload may end this thread (1866) | same | on a function | — | as today | keep |
| `@prelude` | a standard-library prelude module (1867) | same | on a module | — | as today | keep |
| `@no_runtime_checks` | reserved "naming the version" (1875-1876), while `[EXC-1]` (2708) and `[DSJ-8]` (3173) say no such switch exists | nothing | — | — | the name ("there is no switch that turns safety checks off, in any build; to forbid run-time alias checks here write `@no_alias_checks`; to skip one bounds check you have proved, write `xs.get_unchecked(i)` inside `unsafe:` with a `# SAFETY:` note") | **remove** **14:** message by area E (AN-82) |
| `@name` | not an attribute: the grammar's placeholder in `@name(args)` (524) | — | — | — | — | keep (nothing to judge) |
| `@dataclass` | not an Ember word: Python's, quoted in Appendix E (6766), where Ember writes `struct` | — | — | — | — | keep (nothing to judge) |
| `@bench` | a benchmark function (1855) | same | on a function | — | as today | keep |
| `@allocator` | reserved (1876) | same | — | — | as today | keep (reserved) |

### 3.4 Library words that act as language words (must-cover list)

| word | today it means | it will mean | where it is written | replaces | stays refused | verdict and reason |
|---|---|---|---|---|---|---|
| `Box` | one owner on the heap: `Box[T]`, `Box[dyn I]`, `Box[T, A]` with an allocator (163, 857, 1105-1117, 1354, 1476, 2179, 2986) | nothing: `owned T` (owner's ruling). `Box[dyn I]` -> `owned dyn I`; `Option[Box[T]]` -> `Option[owned T]`; `b.get()` -> an `owned` field reads as its `T`. The allocator form `Box[T, A]` (`[ALC-1]`) has no spelling yet (open for the owner, `combined.md` section 9) | — | — | `Box` itself ("`Box` is not part of Ember; for a mesh held alone in its own block write `mesh: owned Mesh`; for `Box[dyn Pool]` write `owned dyn Pool`"); `.get()` on an `owned` field ("an `owned Mesh` reads as a `Mesh`; remove `.get()`") | **merge** into `owned` **14:** confirmed (area C, AN-40) |
| `Box[dyn I]` | a one-owner value of some type implementing `I` (857, 1435, 5457) | `owned dyn I` | in a field, parameter or result type | `Box[dyn I]` | as for `Box` | **merge** into `owned dyn I` (owner: `no_share(dyn Pool)` "or a better word") |
| `Shared` | `Shared[T]`: counted shared ownership on one thread; `get`/`get_mut` checked while running (163, 2611-2621, 2901) | `shared` (row 9's word), written on a value (`other = shared items`) or a type (`mesh: shared Mesh`): several names own one value, which lives until the last one ends | before a value; in a field or type | `Shared[T]` | changing a `shared` value from two threads without a lock (area E) | **merge** into `shared` (settlement 16) **14:** confirmed (area C) |
| `Weak` | a handle that does not keep its object alive (301, 1163, 2594) | **`weak`** (07): in a type (`target: weak Player`, `Array[weak City]`), before a value (`cam.target = weak player`), in a capture list (`with weak self`); reading gives `Option` directly (`if Some(p) = cam.target:`), one test of the target's header per read; `weak T set by f` for a back link the language keeps | in a type, before a value, in a capture list | `Weak[T]`, `Weak(x)`, `Weak.empty()`, `upgrade` | `Weak[T]`, `Weak(x)`, `w.upgrade()` ("write `weak T`", "write `weak x`", "write `if Some(p) = w:`"); `cam.target = player` into a `weak` field ("`=` alone would hand `player` over; write `cam.target = weak player`"); `cam.target.health` with no test | **rename** **14:** (07, area D): the link word sits beside `owned` and `shared` in the type |
| `RefCell` | a value whose borrows are counted while running; a second conflicting borrow panics (185, 241, 1257, 3099-3137) | nothing: **removed**, with `Ref`, `RefMut`, `borrow`, `borrow_mut` and 06's placeholders `RuntimeChecked[T]`, `ReadGuard`, `WriteGuard`, `read`, `write`. A change is said on the method that makes it (`changes f`) and checked when compiling, for one owner and for `shared` owners alike (R5, R9; area C, C9, AN-49) | — | — | `RefCell` ("`RefCell` is not part of Ember; write the field plainly, `entities: Array[int]`, and the method's first line as `fn add(self, e: int) changes entities:`; for a scene several objects own write `shared Scene`") | **remove** **14:** 06 renamed it `RuntimeChecked`; area C removed the type: nothing is left that needs a run-time borrow check |
| `Cell` | a value replaced whole through a shared path; never lends a reference (185, 1383, 3099-3124) | same: a value replaced whole by `.set`, never lending a reference. **Setting it needs a name that may change** (`ref mut s`, `mut self`, `changes frame`), or an owner of a `shared` value (area B's ruling on 06's C-10) | in a type | — | `s.frame.set(...)` through a plain `s` ("`tick` sets `s.frame`, but `s` only reads the caller's clock; write `fn tick(ref mut s: Clock):`") | keep **14:** with area B's condition |
| `Span` | a read-only view of contiguous elements: pointer and length (1118) | same | in a type | — | storing one where its source may end (as today) | keep |
| `MutSpan` | a changeable view; today a `MutSpan` parameter cannot be written through without `mut` as well (01-04; 04 t18-t21; spec 3576 and `[BCK-6]` 6045 write it without `mut`) | **a view whose elements may be written, with no other word**; **06:** re-pointing the caller's own view is `ref mut s: MutSpan[T]` (C-1) | in a type | — | as today otherwise | keep (settlement 13); the compiler's refusal of the spec's own examples is AN-11 **06: keep, amended for C-1** |
| `str` | a borrowed view of UTF-8 text (449, 855, 1120, 1130-1140, 4214) | same type, spelled **`StrView`** (final). A `String` place or a `ref String` reads as a `StrView` wherever one is wanted, inside a lambda too (area B, AN-29). A `StrView` field **of a class** says what it points into (`from <let field>` or `from static`); a struct holding a `StrView` with no `from` is itself a view of the text it was made from (AN-22's `Parser`, R7) (**15:** "of a class" added: the row said every `StrView` field, which AN-22 contradicts) | in a type | — | `str` ("`str` is now `StrView`, a view into someone else's text; for your own text write `String`"); a `StrView` field of a class with no `from` ("to keep its own copy write `name: String` and `self.name = String(name)`") | **rename** **14:** final (07, area B) |
| `String` | an owned, growable text (163, 262, 853) | same; plus the constructor `String(x)` (row 5) | in a type; as a constructor | — | as today | keep (all readers; row 5 uses it) |
| `.to_string()` | **15:** (added; no reader row) the `Display` method that writes any value as text (`5.to_string()`); on a text view, your own `String` (4235: "`s.to_string()` or `String.from(s)` is written") | same; for your own `String` made from text the spelling the model writes is `String(s)` (row 5), and `s.to_string()` on a text view means the same, as `String.from(s)` does (the method the constructor runs) | after a value | — | as today | keep **15:** the NEW code used both `String("tick")` and `"ann".to_string()` for one meaning; `combined-v2.md` writes `String(...)` everywhere (AN-54, AN-56, AN-59). Not refused at a use (unlike `.clone()`): its general meaning, any value written as text, has no other spelling; the spec's P26 help (5461) should give `String(...)`, as row 5's message does |
| `.clone()` | a deep copy for values; a second handle (a share) for a class (797, 1277, 2273-2274) | nothing at the use: callers write `copied x` (rows 6, 9); `clone` stays the method a type writes when it implements `Clone`, and `copied` runs it | — | — | `x.clone()` at a use site, with "write `copied x`" | **merge** into `copied` (settlement 15) |
| `.copy()` | not used anywhere in the spec (searched: no match); `std.mem.copy` (4169) is a raw memory copy; last run proposed `copy(x)` for a copy (`phase3b-ownership.md:57`) | nothing: a copy is `copied x` (rows 6, 9) | — | — | — | **merge** into `copied`: one spelling for a copy |
| `mem.take` | move a value out and leave the default (1559, 2260) | same | as a call | — | as today | keep |
| `mem.replace` | move a value out and put a new one in (1559, 2261) | same | as a call | — | as today | keep |
| `mem.swap` | exchange two places (2261) | same | as a call | — | as today | keep |
| `mem.drop` | end a value now; for a class handle it ends the caller's name, unlike an ordinary `owned` parameter (2261-2263, 1575; 02 t7) | **ends the name `x` now, for every type** (settlement 17): a statement about the name, not just an `owned` parameter | as a call | — | using `x` after it, with "use it before line N, or remove the `mem.drop`" (02's message) | keep **06:** this is a rule of `mem.drop` itself, not of its `owned` parameter: by C-2 an `owned` parameter of a `Copy` instance copies, so the spec's signature `mem.drop(owned x: T)` (2261) must state that the call ends the name for every type. Today `mem.drop(a)` of an `int` followed by `println(a)` is accepted (`k3_mem_drop_int.em`) |
| `Option` | maybe a value (620, 924) | same | in a type | — | as today | keep |
| `None` | the empty `Option` | same | as a value | — | as today | keep |
| `Mutex` | a lock around a value; `with g = m.lock():` (170, 1804, 3406) | same | in a type | — | a reference out of its guard (row 4) | keep: `lock()` names what happens, so `with g = m.lock():` passes the owner's test (03) |
| `RwLock` | a read/write lock (170, 2511) | same | in a type | — | a reference out of its read guard (row 4; today accepted, `phase3b-threads.md:1230`) | keep |
| `Retained` | a token: C keeps an Ember object; dropping the token releases or unregisters (4185, 4900, 4923) | same | in a type; `Retained.pin(v)` | — | as today | keep |
| `Pool` | slots reached by generation-checked handles (2869, 3075) | same | in a type | — | as today | keep |
| `Handle` | `Handle[T]`: an index and a generation into a `Pool` (244, 3089) | same; "handle" is used only for a token that names something elsewhere: `Handle[T]`, and the existing written words `JoinHandle` and `@ffi(handle)` (a C object's handle, the overlay example at spec 4775); never for a class object, which is "the `Player` object `p`", or a link in a field, "the link `o.bag`", in every message and in the model's own text (area C, AN-50; **15:** "only for `Handle[T]`" widened to the two written words that already meant a token, as reader 03 found; area B's AN-24 intent and area D's rule D5 said "handle" for a view and for an object and are reworded in `combined-v2.md`) | in a type | — | a stale handle (a stop: "`pool[h]`: `h` refers to a value already removed from `pool`; use `pool.get(h)`") | keep **14:** messages by area C |
| `Send` | a type that may be moved to another thread (246, 3434) | same; **inferred** from the words: a value handed over may go to another thread when nothing reachable from it is a one-thread owner, a `weak` link of one, or bound to one thread; written only by the author of a raw-pointer wrapper, `unsafe extend T implements Send:` (area E) | in a bound; in `unsafe extend` | — | a value holding a one-thread owner sent away (E7015); a thread-bound value sent away (E7008) | keep **14:** inferred (area E) |
| `Sync` | (a) a property: several threads may read one value at once (`[THR-9]` 3445); (b) the name of an effect: atomics, locks, channels, thread work (3184-3186) | (a) only, spelled **`ThreadShared`**: several threads may hold and read one value of it at once; **inferred** for every type from the words, as Rust infers `Sync`; written only in bounds (`T: ThreadShared`) and messages | in a bound | — | `T: Sync` in a bound ("`Sync` is the effect `@nosync` forbids, not a property of a type; for "threads may share it" write `T: ThreadShared`") | **rename** **14:** 06's C-4; inferred (area E, AN-80) |
| `Sync(effect)` | the effect `@nosync` forbids (3184, 3240), written in `effects=[...]` lists (3208, 4593) | same, spelling **`Sync`** kept (06) | in `@ffi(effects=[...])`, in messages | — | as today | **06: keep (C-4; the draft renamed it to `Synchronise`).** As an effect, "sync" says plainly what it is: the function synchronises with other threads. Once the property is `ThreadShared`, `Sync` has this one meaning |

`ref`, `ref mut` and `dyn`, also on this part of the must-cover list, have their one row each in 3.1
(`ref mut` replaces `Mut[T]`, see `Mut` in 3.5).

### 3.5 Words the readers added (not on the must-cover list)

| word | today it means | it will mean | where it is written | replaces | stays refused | verdict and reason |
|---|---|---|---|---|---|---|
| `init` | the constructor (266, 1640) | same; `mut self` on it is refused | as a method name | — | `fn init(mut self ...)` (AN-32) | keep |
| `read` | `pub(read)`: others may read a field, not change it (538, 6153) | same; it does not clash with row 7's `reads` (both mean "read"; one says who may, the other what a method does) (04) | inside `pub(...)` | — | changing it outside its module (E1050) | keep |
| `package` | `pub(package)` (538) | same | inside `pub(...)` | — | as today | keep |
| `module` | `#! module overflow(saturate)` (501, 523) | same | after `#!` | — | as today | keep |
| `SAFETY:` | a comment the compiler reads above an `unsafe:` block or an `unsafe fn` (`[LEX-23]` 359, `[UNS-8]` 3041), also written `SAFETY(<category>):` (`ffi`, `layout`, `aliasing`, `intrinsic`, `performance`, `uninit`) | **why this code is sound**: on the line before an `unsafe:` block or at the end of its line, or on an `unsafe` implementation (`unsafe extend`, `unsafe extern`, `unsafe overlay`); `SAFETY(<category>):` names its kind (AN-84's `# SAFETY(ffi):`). **Not on an `unsafe fn`**: what its callers must make true is its `@safety` (R15) | as a comment | — | an `unsafe:` block without it (W3012); a note on an `unsafe fn` line, which says nothing the model reads (a warning: "`read_u32_le` says what its callers must make true with `@safety`; a `# SAFETY:` note goes above each `unsafe:` block inside it: remove this line") | keep **15:** the category form added (AN-84 uses it; reader 01 listed it); the `unsafe fn` place removed, so the comment has one meaning: today's `[UNS-8]` asks for it on an `unsafe fn` too, where it can only repeat `@safety`, and AN-79's NEW had both on one declaration |
| `r#name` | a raw name for a keyword (374) | same | as a name | — | as today | keep |
| `Copy` | values copied bit for bit on assignment; covers class handles, `Shared`, `Weak`, whose copy is a share (167, 852, 2240, 2270) | **06:** values whose copy is independent of the original, copied bit for bit on assignment: numbers, `bool`, `ref T`, `Span`, structs of such. **Not** class objects, `shared` values or `weak` links (**15:** was "class handles, `shared` values or `Weak`", the old spellings): their second name is a share, not a copy (C-3). R2's one exception, stated there: a `weak` link bound from a part (`a = n.parent`) is copied, as a number is | in a bound or `@derive` | — | as today | **06: keep the word, narrow the meaning (C-3).** Today `Copy` covers class handles, `Shared` and `Weak`, "whose copy is a share" (02; `[OWN-7]`, ODR-051): one word, two meanings, and the share is made silently. A struct holding a class handle is then not `Copy` either (`@derive(Copy)` on it is E2080). AN-44 (area C) is answered under this meaning |
| `Clone` | the interface that makes an independent copy (1277) | the interface `copied` uses | in a bound, `@derive`, `extend` | — | `copied` on a class that does not implement it | keep |
| `.copied()` | iterator adapter yielding copies (1451, 4367, 6202) | same: agrees with the keyword `copied` | after an iterator | — | as today | keep |
| `drop` | the clean-up method a type writes (1280, 2248) | same | as a method name | — | calling `drop` directly (E3070, help names `mem.drop(x)`) | keep |
| `Drop` | the interface of types with a clean-up method (1537, 1643) | same | in a bound | — | as today | keep |
| `Default` | the type has a default value (1537, 2260) | same | in a bound or `@derive` | — | as today | keep |
| `Some` | an `Option` holding a value | same | as a value | — | as today | keep |
| `Result` | success or error (175, 896) | same | in a type | — | as today | keep |
| `Ok` | the success value | same | as a value | — | as today | keep |
| `Err` | the error value | same | as a value | — | as today | keep |
| `Never` | the type of an expression that never finishes (768) | same | in a type | — | as today | keep |
| `Sized` | a type with a known size (1258) | same | in a bound | — | as today | keep |
| `cstr` | a borrowed C string view, non-null (422, 1105) | same type, spelled **`CStrView`** (final) | in a type | — | as today | **rename** **14:** final (07) |
| `CString` | owned NUL-terminated C text (4866) | same | in a type | — | as today | keep |
| `UnsafeCell` | the unchecked base of the cells (1257, 3045) | same | in a type | — | as today (`unsafe`) | keep: the name says "unsafe" (01) |
| `Ref` | read guard of a `RefCell` (2510, 3125) | nothing: removed with `RefCell` | — | — | the name, with `RefCell`'s message | **remove** **14:** (area C) |
| `RefMut` | write guard of a `RefCell` (3125) | nothing: removed with `RefCell` | — | — | the name, with `RefCell`'s message | **remove** **14:** (area C) |
| `borrow` | open a read guard on a `RefCell` (3112) | nothing: removed with `RefCell` | — | — | the name, with `RefCell`'s message | **remove** **14:** (area C) |
| `borrow_mut` | open a write guard on a `RefCell` (3125) | nothing: removed with `RefCell` | — | — | the name, with `RefCell`'s message | **remove** **14:** (area C) |
| `SyncShared` | `Shared` across threads, atomic count, read-only payload (2615, 3441) | **`thread_shared T`** (07): `log = thread_shared Mutex(lines)`, `log: thread_shared Log`, `with thread_shared log`; atomic count; no owner changes it directly | before a value, in a type, in a capture list | `SyncShared[T]` | `SyncShared[T]` ("write `thread_shared T`") | **merge** **14:** spelled by 07, used by area E |
| `ForeignBox` | a C object this value owns and destroys through its contract's destructor (4185, 4725, 4826, 6508) | **`owned T`** on a foreign type (`db: owned SqliteDb` with `@ffi(destroys(sqlite3_close))`; `std.ffi.adopt[T](p) -> owned T`) | in a type | `ForeignBox[T]` | `ForeignBox[T]` ("write `owned T`") | **merge** into `owned` **14:** 07, area C; 06's placeholder `Foreign[T]` dropped |
| `adopt` | an `unsafe fn` taking ownership of a raw foreign pointer (4883, 6563) | same | as a call | — | adopting twice (E5068) | keep |
| `ForeignCallback` | **15:** (added; no reader row) today's standard-library type a registered C callback is kept as: the registering call returns a `Retained[ForeignCallback]` token whose drop unregisters it (`std/src/ffi.em:225-228`; `[FFI-21]` 4899 names only "a `Retained` token") | same | in a type: `-> Retained[ForeignCallback]` | — | as today | keep: AN-78's NEW writes it; it says what the token holds, and the vocabulary had no row for it |
| `upgrade` | `Weak`: the object if it is still alive (2604, 2660) | nothing: reading a `weak` link gives `Option` directly (`if Some(p) = cam.target:`); no method | — | `Weak.upgrade` | `w.upgrade()` ("a `weak` link is read directly: `if Some(p) = w:`") | **remove** **14:** 07, area D; 06's placeholder `get` dropped |
| `mem.forget` | never drop (2263) | same | as a call | — | on a `@must_drop` value | keep |
| `mem.keep_alive` | keep a handle alive to here (2663) | same | as a call | — | as today | keep (settlement 10) |
| `mem.copy` | a raw memory copy (4169) | same | as a call | — | as today | keep |
| `Atomic` | an atomic value (170, 1804) | same | in a type | — | as today | keep |
| `Once` | a run-once cell (1805, 3448) | same | in a type | — | as today | keep |
| `MutexGuard` | the guard of a lock; reads through to `ref mut T` (3512) | same | in a type | — | a reference outliving it (row 4) | keep |
| `MemoryOrder` | the order of an atomic operation (3516) | same | as an argument | — | an unsupported order (E7006) | keep |
| `Arena` | bump allocator, all freed at once (2402) | same | in a type | — | as today | keep |
| `FixedArena` | an arena that never grows (2945) | same | in a type | — | as today | keep |
| `ScopedArena` | an arena scope freed at block end (2963) | same | in a type | — | as today | keep |
| `BorrowList` | a container that may hold views bounded by its own region (1144, `[TYP-15a]`); not implemented (phase 1 defect 5) | same, or replaced by area A's answer to AN-07 | in a type | — | as today | keep |
| `ArenaArray` | a list in an arena (1145) | same | in a type | — | as today | keep |
| `MaybeUninit` | maybe-uninitialised storage (2969) | same | in a type | — | as today (`unsafe`) | keep |
| `Zeroable` | all-zero bytes are valid (2976) | same | in a bound or `@derive` | — | as today | keep |
| `Generator` | a generator's frame type (2149) | same | in a type | — | as today | keep |
| `Iterator` | `next()` (1983) | same | in a bound | — | as today | keep |
| `get_mut` | a changeable reference into a `Shared` or a container element (2901, 2955) | a changeable reference into a container element (`Array`, `Map`, 4310); the `Shared.get_mut` use goes with `Shared` (an owner of a `shared` value changes it directly, R9) (**15:** the row said "same", which kept `Shared.get_mut`); a `ref mut` view from it may not live across a call that is given a changeable name reaching the same list (area D, AN-61) | as a method | — | `h = self.handlers.get_mut(i).unwrap()` then `h(self)` ("`h` points into `self.handlers`, and `h(self)` lets the handler change `self.handlers`; call it by position: `self.handlers[i](self)`") | keep **14:** area D's refusal |
| `iter_mut` | iterate yielding `ref mut` (2313, 2522) | same | as a method | — | as today | keep **06:** kept for iterator chains (`xs.iter_mut().for_each(...)`, where the result type carries the meaning, as row 5's `word = line.to_lower().trim()` does); in a `for` header the one spelling is `for ref mut v in xs:` (C-1). Today `for v in xs.iter_mut(): v += 1` is accepted (`k9_iter_mut.em`) |
| `as_mut` | `Option.as_mut(mut self) -> Option[ref mut T]` (3903) | same | as a method | — | as today | keep |
| `as_mut_span` | a changeable view of an `Array` (1563) | same | as a method | — | as today | keep |
| `reborrow` | lend a `MutSpan` on again (2539) | same | as a method | — | as today | keep (rarely written) |
| `get_unchecked` | index with no check, inside `unsafe` (2538) | same | as a method | — | outside `unsafe` | keep |
| `assert_disjoint` | check two views do not overlap and return `Ok`/`Err`; never panics (3145-3169) | spelled **`check_disjoint`**: reports whether two views overlap and hands both back; **`assert_disjoint(a, b)`** (today's `assert_disjoint_or_panic`) stops the program on overlap, and afterwards the two names are treated as not overlapping (noalias) (area A, AN-14) | as a call | — | `check_disjoint`'s result unused, or a view used after it took it ("to stop when they overlap, write `assert_disjoint(dst, src)`; to handle both cases, write `match check_disjoint(dst, src):`") | **rename** **14:** final (07, area A) |
| `assume_disjoint` | an unchecked promise, inside `unsafe` (3171) | same | as a call | — | outside `unsafe` | keep |
| `transmute` | reinterpret bits, inside `unsafe` (3023) | same | as a call | — | outside `unsafe` | keep (expert, unsafe) |
| `asm` | inline assembly (3037) | same | in `unsafe asm` | — | on the C backend (E5090) | keep |
| `Allocator` | the allocator interface (2986) | same | in a bound | — | as today | keep |
| `size_of` | compile-time layout query (3060) | same | as a call | — | as today | keep |
| `null` | the null raw pointer of a type (1150) | same | as a value | — | as today | keep |
| `ref_to_ptr` | a raw pointer from a reference (1155) | same | as a call | — | as today | keep |
| `checked` | build a range value, checked (1029) | same | as a call | — | as today | keep |
| `clamped` | build a range value, clamped (1033) | same | as a call | — | as today | keep |
| `debug_assert` | the one check a profile turns off (200) | same | as a call | — | an argument with side effects (W2016) | keep |
| `SoA` | column storage of a struct (3668) | same | in a type | — | as today | keep |
| `SoA.get` | returns a **copy** of element `i`, while `Array.get(i)` returns a reference (3672, 3686 vs 4285; 03 t6) | a reference proxy, as on `Array`: `ps.get(i) -> Option[SoARef[T]]`; a copy is written `copied ps[i]` (03); `ps[i].load()` stays the method `copied ps[i]` runs, as `clone` is for `copied` (area A, AN-13) | as a method | — | `ps[i].load()` written in a program ("write `copied ps[0]`", AN-13) (**15:** the row said "as today"; AN-13 refuses `load()` at a use, as the `.clone()` row does for `clone`) | keep the spelling, change the meaning (03): the same method name copies on one type and borrows on another, and the copy is not named (rows 6, 9: "a copy is only made when named"). AN-13 |
| `SoARef` | the read-only element proxy of a `SoA` (3668-3697) | same | in a type | — | as today | keep |
| `SoAMut` | the changeable element proxy of a `SoA` (3668-3697) | same | in a type | — | as today | keep |
| `Query` | an ECS query (3787) | same | in a type | — | as today | keep |
| `Mut` | ECS query marker: `Mut[T]` gives `ref mut T` (3787-3838) | `ref mut T` in the query, if area A allows a reference type as a query argument (`[ECS-3]` 3826 says the language has no access-mode generic) | in a query type | — | as today | **merge** into `ref mut` (03): two spellings of one meaning. AN-15 |
| `Not` | ECS query marker excluding a component (3787) | same | in a query type | — | as today | keep |
| `Commands` | ECS: changes queued until the loop ends (3787) | same: changes queued until the system returns (the agreed "delete after the loop", Bevy's `commands.entity(e).despawn()`); plus **`create()`**: a new entity id at once, whose entity exists when the queue is applied (area A, AN-06) | reached through the system's query, `q.commands()` (`[ECS-5]` 3838; AN-06) | — | as today | keep **14:** `create()` added (area A) **15:** "where it is written" corrected: it said "in a system's parameters", but the spec and AN-06 reach it as `q.commands()` |
| `Scope` | `thread.scope()`: tasks joined at the block's end (3503) | same | after `with` | — | as today | keep |
| `JobScope` | `jobs.scope()` (3619) | same | after `with` | — | as today | keep |
| `JoinHandle` | a spawned thread's result (3503) | same | in a type | — | as today | keep |
| `thread.spawn` | **15:** (added; no reader row) start a thread that may outlive the function that started it; takes its function `owned` (`[CLO-7]` 2123) | same; what the function holds is written in its capture list (`with owned`, `with thread_shared`, `with copied`); a name not listed is the real variable, which the thread may outlive (R14, AN-71, AN-75) | as a call | — | a name used with no capture word (E7004, naming the words); a one-thread owner captured (E7015) | keep: R14 and E7004 are rules about this call, and AN-55, AN-71, AN-75, AN-76 and AN-80 write it; `[CLO-15]`'s capture by move at this call is gone (section 6) |
| `range` | **15:** (added; no reader row) `range(n)` is `0..n`; `range(a, b, step)` as Python's (`[STD-26]` 4147-4149) | same; `for i in range(xs.len()):` is a counting loop, so R8 (c)'s rule for `for i in 0..xs.len():` applies to it (it may add to `xs`, not remove) | as a call in a `for` | — | a removal from `xs` inside `for i in range(xs.len()):` (R8 (c), as for `0..xs.len()`) | keep: AN-51's NEW writes `range(ws.len())` where R8 (c) is written with `0..`; one rule for both spellings |
| `AnyError` | an owned `dyn` error with a context chain (3932) | same | in a type | — | as today | keep |
| `ObjectPtr` | the address of a class object for C (4926) | same | in a type | — | as today | keep |
| `Entity` | a 32-bit ECS entity (3811) | same | in a type | — | as today | keep |
| `OldSelf` | the previous version of a type in hot reload (6370) | same | in `migrate_from` | — | as today | keep |
| `migrate_from` | hot reload: build the new value from the old (6370) | same | as a method | — | as today | keep |
| `CppShared` | a C++ `shared_ptr` (6506) | same: keeps its name, a C++ count is not Ember's (07, `planned-words.md` 341-342) (**15:** the row still said "07 decides") | in a type | — | not convertible to `shared` (E5065) | keep |
| `CppWeak` | a C++ `weak_ptr` (6506) | same | in a type | — | as `CppShared` | keep |
| `borrowed` | C contract word: valid for the call (4826) | same | in an overlay or `@ffi(param(...))` | — | as today | keep (03: every contract word has one meaning, stated in a table) |
| `returns_owned` | C contract word: the result is owned and freed by `destructor=f` (4826) | same | as above | — | as today | keep |
| `retained` | C contract word: the callee keeps the pointer (4826) | same | as above | — | a kept pointer not declared `retained` (E5051) | keep |
| `exclusive` | C contract word: the pointer is the only one (4830) | same | as above | — | as today | keep |
| `aliased` | C contract word: other pointers may reach it (4830) | same | as above | — | as today | keep |
| `TODO(...)` | an unknown foreign fact in a starter overlay (5101) | same | in an overlay | — | `@safety("TODO")` left in (L3016) | keep |
| `as?` | the spec's checked downcast to `Option` (810, 945, 1164) | nothing: today's compiler refuses it for `cast x if D` (section 5.4) | — | — | `as?`, with today's help (it already gives the line) | **merge** into `cast` (the spec text is behind the compiler, ODR-280) |
| `as!` | the spec's downcast or panic (810, 945) | nothing: `cast x as D` | — | — | as `as?` | **merge** into `cast` |
| `cast` | today's compiler only (ODR-280): `cast x as D` gives the `D` or panics, `cast x if D` gives `Some(d)` or `None` (section 5.4); not in the spec text | today's `cast x as D` (gives the `D` or stops) and `cast x if D` (gives `Some(d)` or `None`), also on `owned dyn` and `shared dyn` values; **a `cast` of a place is a place**: the name bound from it is read-only unless written `ref mut` (`pool = ref mut cast self.pools[k] as Pool[T]`; `if Some(ref mut pool) = cast self.pools[k] if Pool[T]:`); one type-id compare (R13; areas B, D) | before a value | `as?`, `as!` | a cast between unrelated types (E2020, naming `owned dyn AnyPool` where `Box[dyn AnyPool]` was the cause); `if c = cast s if Circle:` ("`c = ...` names `c` but tests nothing; to run the block only when `s` is a `Circle`, write `if Some(c) = cast s if Circle:`") | keep **14:** one rule for areas B and D (clash 4) |

### 3.6 New words from the agreed rows and the owner's rulings

| word | today it means | it will mean | where it is written | replaces | stays refused | verdict and reason |
|---|---|---|---|---|---|---|
| `copied` | not a word: `snapshot = copied items` is E1010 "cannot find `copied`" (section 5.3) | **a new, independent value equal to the old one; nothing done to one is seen through the other** (rows 6, 9). Written before a value (`snapshot = copied items`, `return copied g[0]`) and in a lambda's capture list (`fn() with copied score:`) | before a value; after `with` in a capture list | `.clone()`, `.copy()`, the copy half of `owned fn` | `copied` of a class object whose class does not implement `Clone`; `copied` of a view (`str`, `Span`), whose text belongs to someone else: "to make your own copy write `String(word)`" (row 5's spelling), so `copied` never means two things; **06:** for a `Span[T]` the message names the owned list ("to make your own copy write an `Array[T]` of it"), as E3063's help already does. (Today's `s.to_array()` cannot be called on a `Span` at all: E3063 inside `std/src/core.em:988`, `k8_copied_span.em`; a library defect for the lead, not a word) | **new** (rows 6 and 9; owner: "use the copied keyword for it") **06: passes** every test: it sits on the value or captured name it copies, means one thing for every type it is allowed on, and is refused where it could not |
| `shared` | not a word: `other = shared items` is E1010 (section 5.3) | **several names own one value, which lives until the last of them ends** (row 9), all on one thread, with a one-thread count. Before a value (`other = shared items`, `game = shared Game()`), in a type (`mesh: shared Mesh`, `shared dyn I`), in a capture list (`with shared log`). **Any owner may change it**, written as any change is written (R5); nothing is checked while running; views of it follow R8 (b) and R10 (area C, C6, C7, AN-48). A `class` object is not shared unless a line says `shared`. **15:** on a parameter the mode is on the name, as for every type (R4): `x: shared T` reads the caller's `shared` value (a view, no count), `ref mut x: shared T` may change it, `owned x: shared T` makes the callee one more owner (the caller writes `shared y`, or hands its own over) | before a value; in a field or type (a parameter's type included); in a capture list | `Shared[T]` | a view used after a change, through any owner, that may reshape what it points into ("use it before, or keep your own copy: `copied ...`"); a `shared` value sent to another thread (E7015: "write `thread_shared`") | **new** (row 9) **14:** area C's rules **15:** the parameter rule stated: 14's R4 and area C's C5 had `x: shared T` make the callee an owner with no mode on the name, the form 06's C-2 refuses for `x: owned T` ("`fn f(m: owned Mesh)` and `fn f(owned m: Mesh)` ... mean 'read it' and 'take it'"), and AN-78 called `setup(game)` then used `game`, which that rule would refuse; read as every other parameter, the word means one thing in every place and a call costs no count |
| `changes` | not a word: `fn note(self, msg: String) changes log:` is E0100/E0110 (section 5.3) | **the fields (or module variables) this function may change**, besides through its `ref mut` parameters (row 3): on a method, and (07) on a free function naming a `static mut` (`fn tick() changes FRAME, LOG:`). A method that says `changes f` may change anything reached through `f`. Checked against the body both ways and at every call; a loop over a field may call it when it does not name that field. Order on the first line: parameters, `reads`, `changes`, `->`, `from`. **15:** a call through a `virtual` or interface method is checked against that method's line, so an override's or implementation's `changes` stays within it (R5; see `override`) | after a function's parameters, after `reads`, before `->` or `:` | the class meaning of `mut self` (whole-call write access); `[BRW-10]`'s field set inferred only for private methods | the body changing something its first line does not name, or naming something it does not change (E2105, with the corrected line); calling it while a loop walks a field it names | **new** (row 3, spelling agreed) **14:** free-function position (07, area E); order and E2105 (clashes 4, 8) |
| `reads` | not a word: `fn load_assets(self) reads renderer -> Array[String]:` is E0100/E0110 (section 5.3) | **the fields this method reads** (row 7); checked against its body and at the call: `init` may call it once those fields are set. A `pub` method with no `reads` is taken by callers in other packages to read every field (area B; combined R16). `@cached` depends on it | after a method's parameters, before `->` | — | calling it before a named field is set (with E2100's new help); the body reading a field not named | **new** (row 7, spelling agreed) **14:** confirmed (area B) |
| `String(...)` | refused: `String(word)` is E2020 "`String()` takes no arguments", no help (03 t9, 04 t13); `String.from(s)` and `s.to_string()` work | **your own `String` made from a text view** (the same as `String.from(s)`) | as a constructor | — | — | **new** (row 5's message names it; the agreed spelling needs the constructor) **06: passes**; `String.from(s)` stays as the `From` method that `String(s)` runs (as `clone` is the method `copied` runs), so the two are the interface and its use, not two spellings at the use |

The one-owner type `owned T` (`mesh: owned Mesh`, the owner's ruling) is not a new word but a new use
of `owned`; it is in the `owned` row of 3.1. Today `mesh: owned Mesh` is E0100 (section 5.3).

### 3.7 Words of phases 2 and 3 (07's planned words and the areas' new words)

| word | today it means | it will mean | where it is written | replaces | stays refused | verdict and reason |
|---|---|---|---|---|---|---|
| `thread_shared` | not a word: `log = thread_shared Mutex(lines)` is E1010 (07's `p01`) | **one of several owners, which may be on different threads**; it lives until the last owner ends; atomic count; no owner changes it directly, only through `Atomic`, `Mutex`, `RwLock`, a channel, or a `guarded by` field under its lock. **15:** on a parameter, as `shared`: `log: thread_shared Log` reads the caller's value (a view, no count; AN-75's `start`), `owned log: thread_shared Log` takes one owner | before a value (`thread_shared Stats()`), in a type (`log: thread_shared Log`), in a capture list (`with thread_shared log`) | `SyncShared[T]`; Rust's `Arc` | a direct change (E7014: "keep the list in a field `guarded by` a lock and change it inside `with names.lock:`"); a `shared` value sent to another thread (E7015, naming `thread_shared`) | **new** (07) **14:** accepted; used by area E (AN-71, AN-74, AN-75, AN-80) |
| `weak` | not a word: `cam.target = weak player` is E1010 (07's `p03`) | **a link that does not keep its target alive**; reading it gives `Option` (one test of the target's header); a name bound from the `Some` is a read-only view kept alive under R10; `Some(ref mut p)` may change the target; the target lives on its own (a class object, or a value held `shared`/`thread_shared`) | in a type, before a value, in a capture list | `Weak[T]`, `Weak(x)`, `Weak.empty()`, `upgrade` | see the `Weak` row; `weak pos` of a part ("a weak link needs an object that lives on its own; link to the player: `weak player`") | **new** (07) **14:** accepted; area D (AN-60 .. AN-65) |
| `set by` | not a word (07's `p02`) | **the language keeps this `weak` link equal to the object whose field holds this one**: set when it is put into that field, cleared when it leaves, never assigned by the program, read as a plain load; the holder field is changed only by its own operations | after a `weak` field's type: `parent: weak Node set by children`; `into: Array[weak City] set by roads` | last run's `= backlink(children)`; hand-kept `Weak` back links | `leaf.parent = root` ("`parent` is set by `children`; write `root.children.push(leaf)`"); `set by` without `weak`; a single link set by a field whose elements are `shared`; lending the holder as a changeable list | **new** (07) **14:** accepted, with area D's rules D7 |
| `@cached` | not a word: E0104 (07's `p04`) | **the method's result is kept after the first call and cleared by every change to a field its `reads` clause names**, through any name; a `Copy` result is returned by value, any other as a `ref T` view into the kept value; on an object several threads reach, the slot is filled once under a once-flag | on a method, above `fn`, whose receiver is plain `self` and which has `reads` | a hand-kept `Cell[Option[T]]` cache | without `reads`; with other parameters; reading a `shared`, `thread_shared`, `weak` or `Cell` field; with `changes` or `mut self`; reading a field other threads change under a lock (each with area B's or E's message) | **new** (07) **14:** accepted; area B (AN-21), area E (threads) |
| `guarded by` | not a word (area E's `new/an74_new.em`: `Lock` is E1061, `guarded` not reached) | **this field (or module variable) is read and changed only inside `with x.lock:` on the same object, on the thread holding the lock, outside `init`**; several fields may name one lock | after a field's or a `static mut`'s type: `words: int guarded by lock = 0`; `static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []` | last run's inferred lock sets; a `Mutex` around a separate struct of the fields | use outside the lock (E7013: "`words` is guarded by `lock`; change it inside `with stats.lock:`"); the lock of another object (E7013) | **new** (area E) **14:** accepted (`combined.md` section 2) |
| `Lock` | not a type: `from std.sync import Lock` is E1061 | **a lock with no value inside**; `with x.lock:` holds it for the block (the bare `with`) | in a field or `static` type: `lock: Lock = Lock()` | `Mutex[void]` | a guarded field used without it (E7013) | **new** (area E) **14:** accepted (`combined.md` section 2); `Mutex[T]` stays for one value behind a lock |
| `Commands.create()` | not a method: E2072 (area A's `an06b`) | **a new entity id, given at once, whose entity exists once the system's queued changes are applied after it returns**; one atomic add in a `run_parallel` system | a call on a system's `Commands`: `n = q.commands().create()` | — | — (the query does not see `n` until the queue is applied, as Bevy says) | **new** (area A, a library method) **14:** accepted (`combined.md` section 2) |
| `from static` | `static` exists; `from` after a field's type does not (07's `p08`) | **the view points only into text or data written in the program** (the static region, `[LT-3]`) | after a view field's type: `kind: StrView from static` | — | a view of something else stored in it ("`kind` holds only text written in the program ...; write `self.kind = String(name)` with `kind: String`") | **new position** of two words (area B) **14:** accepted (`combined.md` section 2) |

## 4. Placeholders and their final spellings

| 06's placeholder | final | by |
|---|---|---|
| `StrView` (for `str`) | `StrView` | 07 |
| `CStrView` (for `cstr`) | `CStrView` | 07 |
| `RuntimeChecked[T]` (for `RefCell`), `ReadGuard`, `WriteGuard`, `read`, `write` | removed, with `RefCell` | area C (AN-49) |
| `Foreign[T]` (for `ForeignBox[T]`) | `owned T` on a foreign type | 07, area C |
| `get` (for `upgrade`) | dropped: a `weak` link is read directly | 07, area D |
| `check_disjoint`; `assert_disjoint` for the stopping form | kept | 07, area A |
| `@thread_shared` (for `@sync`) | removed | area E (AN-80) |
| `ThreadShared` (for the property `Sync`) | kept, inferred, written only in bounds | area E |
| `@no_alias_checks` (for `@static_safe`) | kept | 07, area E |
| `@fp(fuse)` (for `@fp(contract)`) | `@fp(allow_fma)` | 07 |
| `from p` (for `@borrows`) | `from`, after a result type or a view field's type; `from static` | 07, area B |
| `with owned x` | kept, with the other capture words | 07, area D |
| `SyncShared[T]` merged into `shared` | `thread_shared T` | 07 |

## 5. Words still open for the owner

- `Box[T, A]` with an allocator (`[ALC-1]`, spec 2986): no spelling once `Box` goes; no annoyance needs it
  (`combined.md` section 9).
- A word at the call site for an argument given to a `ref mut` parameter: none, by clash 9 of
  `combined.md`; listed there for the owner.

## 6. The spec's uses of every word whose spelling or meaning changes (15)

The duty of 15 includes "a word whose meaning changes must not leave old uses meaning something
else". This run edits no spec (brief), so this section is the list the spec rewrite works from: every
use, in spec lines 1-6930 (the readers' four slices), of a word this vocabulary renames, removes,
merges or gives a new meaning, and what each use becomes. The code uses (inside an `ember` block or a
backtick span) were found by `target/redesign3/15-vocabulary-review/spec_uses.py` (its output, every
line with its text, is `target/redesign3/15-vocabulary-review/spec_uses.txt`) and checked against the
readers' lists (01 sections 2.1-2.4, 02 sections 2a-2c, 03 sections 2-5, 04 sections 1.3-1.5); the
rules that state an old meaning in prose were read one by one. 06's own list of this kind
(`vocabulary.md` 6.3, seven entries) is folded in.

### 6.1 Spelling changes: every listed line takes the new spelling

| word today | becomes | spec lines (code uses) | notes |
|---|---|---|---|
| `Box[T]`, `Box(v)`, `Box[dyn I]` | `owned T` in a type, the value stored as it is; `owned dyn I` | 163 620 857 1105 1106 1114 1116 1127 1354 1355 1383 1435 1442 1476 1536 1902 2120 2179 2183 2348 2509 2863 2893 2895 2986 3535 4031 4129 4924 4926 4957 5068 5457 5704 6366 | 2893 `b.get()`: an `owned` field reads as its `T`; 1476 (method lookup strips one `Box`): strips one `owned`; 5704 E2200 "without `Box`" -> "without `owned`"; 2986 `Box[T, A]` stays open (section 5) |
| `Shared[T]` | `shared T` (type), `shared x` (value) | 163 1127 1476 1536 2270 2509 2611 2617 2621 2665 2668 2801 2804 2816 2817 2820 2865 2880 2897 2902 2904 2905 2907 3441 3447 3534 3738 4129 4957 5321 5422 5424 5801 6510 | 2897-2905, `get`/`get_mut` checked while running: gone (an owner changes it directly, R9); 5422 and 5424 (`[DIA-9]`, `[DIA-16]`): the cost note becomes "one count per owner made or ended" |
| `SyncShared[T]` | `thread_shared T` | 2615 2666 2805 2814 2866 2906 2908 3441 3449 3478 3535 | "read-only payload" becomes "changed only through `Atomic`, a lock, a channel or a `guarded by` field" (R9) |
| `Weak[T]`, `Weak(x)`, `Weak.empty()` | `weak T`, `weak x`, `None` | 301 1163 1536 2270 2594 2597 2660 2802 2804 2805 2806 2816 2817 2820 2823 2867 2880 2905 3216 3440 3441 3447 4957 5321 5410 5801 6083 6365 6510 6765 6809 | 2594-2606 (the `Node` example): `parent: weak Node set by children` and `fn add(self, owned child: Node) changes children:` with no assignment to `parent`; 6809 (the cycle warning) names the link to make `weak` only for a cycle of `shared` owners (R11) |
| `upgrade` | reading the link: `if Some(p) = w:`, `match w:` | 2604 2660 2810 2812 3216 6083 | 2604 `match c.parent.upgrade():` becomes `match c.parent:` |
| `RefCell`, `Ref`, `RefMut`, `borrow`, `borrow_mut` | removed: the field is written plainly and the change is said on the method (`changes entities`, AN-49) | 185 241 1257 1387 1429 1536 1555 2348 2510 2669 2870 3099 3109 3112 3125 3126 3129 3132 3134 3135 3137 3215 3226 3268 3365 3446 3588 4034 4170 4195 5259 5422 5424 5846 6898 | 3108-3113 (the `Scene` example) becomes `fn add(self, e: int) changes entities:` on `struct Scene`; 622 (`SoA[Particle].Ref`, a path segment) is not the guard and stays |
| `ForeignBox[T]` | `owned T` on a foreign type | 4185 4725 4826 4883 4886 4930 4979 6508 6523 6563 | 6508: `std::unique_ptr<T>` imports as `owned T` |
| `str` (the type) | `StrView` | 449 855 916 1105 1120 1122 1136 1140 1188 1255 1366 1504 1533 1797 1818 1928 1954 1955 1959 1987 2325 2335 2357 2361 2392 2510 2533 2877 3031 3376 3418 3862 3891 4144 4172 4194 4207 4214 4216 4217 4218 4219 4222 4224 4225 4227 4230 4234 4235 4236 4243 4257 4259 4260 4302 4332 4339 4354 4358 4867 5461 5462 5497 5778 5799 5966 6139 6193 6207 6505 6506 6926 | only where `str` is the Ember type; Python's `str(x)` in Appendix E (5438, 6776) stays |
| `cstr` | `CStrView` | 422 1105 1107 1108 4185 4220 4725 4732 4827 4866 4869 4870 4874 4959 5800 5966 | |
| `@sync class` | removed: the value says `thread_shared`, each field `guarded by` or `Atomic` (AN-80) | 1266 1857 2622 2713 2730 2813 2910 3361 3413 3441 3447 3450 3451 3453 3460 3462 3463 3464 3467 3470 3472 3475 3477 4934 4966 5322 5368 5810 5812 6077 6083 6164 6475 6923 | E7001, E7003, E7005 retire (`combined-v2.md` section 8) |
| `@static_safe` | `@no_alias_checks` | 148 234 1846 3048 3169 3245 3260 3267 3275 5506 5756 5762 6315 | `[EFF-12]` 3260's own text ("forbids only aliasing checks") becomes the name |
| `@borrows(p)` | `from p` after the result type | 1859 2372 2391 2398 2404 2408 2470 5496 5665 6921 | 6921 (glossary, "source parameter"): "without `from`" |
| `@fp(contract)` | `@fp(allow_fma)` | 1003 1014 1848 3332 3380 4492 5767 5920 | |
| `@no_runtime_checks` | removed from the reserved list | 1876 | AN-82's message for the name |
| `as?`, `as!` | `cast x if D`, `cast x as D` (today's compiler, ODR-280) | 471 476 705 730 810 945 1164 1931 2795 3190 3363 | 471, 476, 705, 730 and 810 are the lexer and grammar entries: they go |
| `Mut[T]` in a query | `ref mut T` | 3799 3818 3819 | `[ECS-3]` 3819-3826 ("ordinary types; the language has no access-mode generic"): `ref mut T` is the language's own spelling inside a query (AN-15) |
| `owned fn` | the capture list after the parameters, `fn() with owned x, copied y:`; a name not listed is the real variable | 692 1129 1908 2045 2055 2061 2064 2074 2088 2090 2131 2136 2494 2656 3495 3535 3563 4899 5410 5499 6221 | 2045 `return owned fn(x: int) => x + n` becomes `return fn(x: int) with copied n => x + n`; 3495 `thread.spawn(owned fn() => produce(tx))` becomes `thread.spawn(fn() with owned tx => produce(tx))`; 6221 `task = owned fn() => println("done")` becomes `task = fn() => println("done")` (it captures nothing, AN-47); B9's help (5499) becomes "`with owned x`, `with shared x`, or a scope" |
| `.clone()` at a use | `copied x` | 798 1566 2252 4378 5482 5483 5522 6807 | `clone` stays the method `copied` runs (`[OWN-8]` 2273) |
| `assert_disjoint` (returns a `Result`) | `check_disjoint` | 3145 3148 3157 3169 3364 3736 4169 5501 6911 | |
| `assert_disjoint_or_panic` | `assert_disjoint` | 3166 | `[DSJ-5]` |
| `Sync`, the property | `ThreadShared` | 170 246 861 1195 1251 1256 1257 1261 1262 1266 1537 1746 1804 1805 1806 2107 2615 2665 2666 2907 2908 3032 3047 3123 3132 3359 3434 3438 3439 3442 3445 3446 3447 3449 3450 3451 3452 3454 3467 3475 3478 3480 3526 3568 3832 4731 4966 5810 5811 5814 6515 6608 6917 | the effect keeps `Sync` at 3186 3230 3231 3240 3601 3749 3757 4002; in 3186 "retaining or releasing a `Sync` handle" becomes "making or ending a `thread_shared` owner" |
| `mut x: T` on a parameter's name | `ref mut x: T` | 1136 1298 1301 1304 1547 2260 2261 2296 2297 2441 6690 6856; grammar 553-554 (`param`), 617 (`fn_type_param`), 716 (`lambda_param`) | in the three grammar rules `"mut"` becomes `"ref" "mut"` (06's 6.3); a `MutSpan` parameter drops `mut` instead, 278 2520 3147 3618 3663 3704 6186 (AN-11); 3799 becomes `ref mut q: Query[(ref mut Position, Velocity)]` |
| `x: ref T`, `x: ref mut T` as a parameter's type | `x: T`, `ref mut x: T` | 1150 2296 6370 | 1150 `ref_to_ptr(r: ref T)` becomes `ref_to_ptr(r: T)`; 6370 `migrate_from(old: ref OldSelf)` becomes `migrate_from(old: OldSelf)` |
| `for x in xs.iter_mut():` | `for ref mut x in xs:` | 279 2522 3688 3800 6187 | 3688 (`SoA`): `for ref mut p in ps:` yields `SoAMut[T]`; 3800: `for ref mut pos, vel in q:` |
| `with h:` (keep alive) | unchanged | 2014 2663 | the bare `with` form (settlement 10) |

### 6.2 Meaning changes: the rules and examples that state the old meaning

| spec (lines) | says today | becomes (the model's rule) | word |
|---|---|---|---|
| `[PHIL-2]` 163 | `Box`, `Shared` among the allocating forms | `owned T` and `shared T` in a type, and `copied`, `shared`, `thread_shared` before a value | `owned`, `shared` |
| `[TIER-1]` 151-154; glossary "Safe Ember" 6916 | an `unsafe fn` body is an unchecked zone with no mark per operation | inside an `unsafe fn` body every unsafe operation sits in its own `unsafe:` block (E3100, AN-79); "Safe Ember" is code outside `unsafe:` blocks, `unsafe fn` bodies included | `unsafe fn` |
| `[LEX-23]` 359-363; `[UNS-8]` 3041-3044; the example's 3008 | a `# SAFETY:` note on every `unsafe:` block and `unsafe fn` | on every `unsafe:` block and `unsafe` implementation; an `unsafe fn` carries `@safety` instead (W3012 for blocks, L3015 for functions); 3008's line goes | `SAFETY:` |
| `[UNS-7]` 3039 | `@safety` on every `pub unsafe fn` | on every `unsafe fn` | `@safety` |
| `[FN-1]` 1553-1561 | `mut b: B` in-out; `owned c: C` "copied if `Copy`; a handle is copied without a retain"; a write through a `Cell` or `RefCell` in a borrowed parameter changes the caller's value | `ref mut b: B` in-out; `owned c: C` hands over, refused where `C` is known to be `Copy` (a generic `Copy` instance is copied, row `owned`); a class object is not `Copy`; setting a `Cell` needs a changeable name (B's ruling); `RefCell` gone | `mut`, `owned`, `Cell` |
| `[FN-1a]` 1562-1564 | a `mut` parameter whose type is a view | a `MutSpan[T]` parameter writes its elements with no other word; `ref mut s: MutSpan[T]` re-points the caller's view | `MutSpan` |
| `[FN-2]` 1565-1566 | "the caller passes `x.clone()`" | "the caller passes `copied x`" | `copied` |
| `[FN-9]` 1570-1578 | a mode on a class-handle parameter governs the handle: a borrowed handle writes the object; `owned` is a retained copy the caller keeps; `mut` re-points only | a class object has one owner and its parameters work as every type's: `x: T` reads the object, `ref mut x: T` may change it (and replace it), `owned x: T` hands it over (the caller's name ends) | `mut`, `owned`, `class` |
| `[TYP-23]` 1468-1469 | an unannotated lambda parameter takes `owned` from the expected callable type; a lambda that writes its parameter says `mut` | a lambda parameter's mode is what is written on it; a plain one reads (AN-56); one that changes the caller's value says `ref mut` | `owned`, `mut` |
| `[TYP-14]` 1114-1117 | a `Box[Tree]` payload is read through where a `Tree` is wanted | an `owned Tree` reads as a `Tree` | `owned` |
| `[CLS-7]` 1738-1744; examples 1704-1705 and 6159-6161 | any class method writes fields through `self`; `mut self` holds a write access on every field for the whole call; re-pointing a caller's handle is a `mut` parameter | plain `self` reads; `self ... changes f` changes `f`; `mut self` any field, checked when compiling; replacing the caller's object is a `ref mut` parameter. The `Door` examples become `virtual fn on_update(mut self, dt)` in the base and `override fn on_update(self, dt) changes angle:` in `Door` (row `override`) | `self`, `mut self`, `override` |
| `[CLS-8]` 1746 | a `Sync` class has every field immutable after `init` | removed with `@sync` (a field says `guarded by` or is `Atomic`) | `@sync` |
| `[CLS-9a]` 1749-1750 | a `let` field's contents may still change | nothing is changed through a `let` field after `init` (06's C-8) | `let` |
| `[OWN-6]` 2260-2263 | `mem.take(mut place: T)` ...; `mem.drop(owned x: T)` moves "a class handle's too though a handle is `Copy`" | `mem.take(ref mut place: T)`, `mem.replace(ref mut place: T, owned new: T)`, `mem.swap(ref mut a: T, ref mut b: T)`; `mem.drop` ends the name for every type, by its own rule (06's C-2) | `mut`, `mem.drop` |
| `[OWN-7]` 2269-2272; 168, 2240 | a `Copy` copy retains every counted handle in it | `Copy` is an independent bit copy; class objects, `shared` values and `weak` links are not `Copy` (06's C-3) | `Copy` |
| `[OWN-8]` 2273-2274 | cloning a class handle copies the handle | `copied p` of a class object runs its `Clone` and makes a new object (AN-45) | `copied`, `class` |
| `[CLO-1]`, `[CLO-2]` 2060-2069 | captures inferred per variable; `owned fn` moves (copies a `Copy`); a closure that writes its captures is called from a `mut` place | a capture is the real variable unless the capture list names it (row 6); a function value that changes its own state (`copied`, `owned` captures) is called through a changeable name (`ref mut f`, D3) | `fn`, `with` |
| `[CLO-7]` 2123-2128; `[CLO-15]` 2129-2137 | a lambda written at a consumed callable parameter that keeps it captures by move, "as if written `owned fn`" | removed: the capture list says what a kept function holds, and a missing word is E7004 with the line to write (AN-55, AN-71) | `owned fn` |
| example 2556-2568 | `fn heal_all(players: Array[Player]): for p in players: p.health = 100.0` ("handles: the objects can change") | `fn heal_all(ref mut players: Array[Player]): for ref mut p in players:` (AN-20) | `for`, `class` |
| `[OBJ-1]` 2626-2628 | a 24-byte header; per-field access words | an 8-byte header (a 32-bit count, a 32-bit type index); no access words (C8) | `class` |
| `[OBJ-2]` 2629-2634; P22 5457 | a bare interface name in a type is a counted class handle | refused: `owned dyn I` or `shared dyn I`; P22's help "`ref dyn Shape`, `Box[dyn Shape]`" becomes "`dyn Shape` as a parameter's type, `owned dyn Shape`" (D8) | `dyn` |
| `[RC-*]` 2646-2700; `[EXC-1]` .. `[EXC-19]` 2704-2790 | every class object counted; exclusivity checked on class fields while running | counts only for `shared` and `thread_shared`; no exclusivity check while running (R9) | `class`, `mut self` |
| `[BRW-10]` 2306-2310 | a private method borrows only the fields its body touches; a `pub` one all of `self` | a method's first line says what it changes and reads (`changes`, `reads`); a `pub` method with no `reads` is taken to read every field (R5, R16) | `changes`, `reads` |
| `[CELL-1]` .. `[CELL-9]` 3099-3137; example 3105-3106 | `Cell` set through a shared borrow; `RefCell` checked while running | `RefCell` removed; setting a `Cell` needs a changeable name: `fn advance(ref mut s: Sprite):` | `Cell` |
| `[STA-1]` 1803-1808 | `static mut` only for foreign interfaces, every access `unsafe` | an Ember `static mut` is a module variable a function names with `changes NAME`, reached only by the main thread and its scoped tasks unless `guarded by` a `static` lock (E7012); a foreign `static mut` keeps `unsafe`; `T` must be `ThreadShared` | `static` |
| `[THR-1]`, `[THR-2]`, `[THR-7]`, `[THR-9]` 3445-3480 | `@sync` classes; `Sync` handles | `thread_shared` values; `guarded by`; `ThreadShared` inferred (AN-80) | `@sync`, `Sync` |
| `[SOA-6]` 3684-3690; example 3672 | `ps.get(i) -> Option[T]` gathers a copy; `load()` gathers | `ps.get(i) -> Option[SoARef[T]]`; a copy is `copied ps[i]` (AN-13) | `SoA.get` |
| `[DSJ-1]`, `[DSJ-5]`, `[DSJ-6]` 3157-3170 | `assert_disjoint` returns a `Result`; `assert_disjoint_or_panic` stops | `check_disjoint` returns; `assert_disjoint` stops (AN-14) | `assert_disjoint` |
| `[FFI-21]` 4893-4905 | a retained callback "MUST be an `owned fn`" | its capture list says what C keeps: `with shared` or `with weak` under `threads(creator)` or `threads(main)`, otherwise `with thread_shared`, `with owned`, `with copied` (AN-78) | `owned fn`, `with` |
| `[FFI-23]` 4920-4928; `[FFI-33b]` 4929-4931 | `Retained.pin` of a `Box`; a `ForeignBox` records its creating thread | of an `owned T`; an `owned` foreign value records it | `owned` |
| `[CLI-20]` 5067 | `ember fmt --migrate` removes `@view` | keeps it (row `@view`) | `@view` |
| W2111 5863 (`[CLS-4]`) | `virtual` in a final class: a warning | an error, "write `open class Door:` or remove `virtual`" (AN-70) | `virtual` |
| the shapes 5482-5524 (`[DIA-21]`) | O1 and N8 "`x.clone()`"; B4 "`mut`; `Cell` for a counter"; B6 and B7 "`@borrows(p)`", "`str`"; B9 "`owned fn`" | `copied x`; `ref mut`; `from p`; `StrView`; the capture list | several |
| P26 5461 | "a `str` kept where a `String` is wanted": help `.to_string()` | help `String(name)` (row 5's spelling; row `.to_string()`) | `String(...)` |
| E0109 5643; E2104 5686; E2200 5704; E2261 5724 | `owned` "not permitted in expression position" (while 6221 writes one); no help; "without `Box`"; "return a `Box[dyn I]`" | E0109 for `owned` before a value and for `owned fn`, with AN-47's help; E2104 "set it in `init`, or remove `let`"; "without `owned`"; "return an `owned dyn I`" | `owned`, `let` |
| Annex A 6140-6229 | `open class Script` "reference type"; `override fn on_update(self, ...)` writing `angle`; `@sync class Counter`; `fn fill(mut buf: MutSpan[float])` with `iter_mut`; `s: str`; `Map[str, int]`; `task = owned fn() => ...` | "one owner unless `shared`"; the `Door` rewrite above; `thread_shared` and `Atomic`; `fn fill(buf: MutSpan[float], v: float): for ref mut x in buf:`; `StrView`; `task = fn() => println("done")` | several |
| Annex C 6591-6592 | a C++ `T&` parameter imports as `mut T`; `T&&` as `owned T` | `ref mut x: T`; `owned x: T` unchanged | `mut` |
| glossary 6889 6897 6898 6900 6917 6921 6923 6926; Appendix E 6765 6807 6809 | class = counted handles; handle = a class reference or a `Handle[T]`; interior mutability via `RefCell`; the modes `mut` and `owned`; `Sync`; `@borrows`; `@sync`; a view includes `str`; "handles are reference-counted; `Weak` for back-pointers" | class = one owner unless `shared`; handle = `Handle[T]` (and `JoinHandle`, `@ffi(handle)`); `Cell`, `Atomic` and the locks; the modes `ref mut` and `owned`; `ThreadShared`; `from`; removed; `StrView`; "`shared` for a second owner, `weak T set by` for back links" | several |
