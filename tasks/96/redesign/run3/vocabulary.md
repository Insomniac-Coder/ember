# The approved vocabulary (run 3, phase 1; checked by 06)

This is the 05 builder's draft (`RUN3/vocabulary-draft.md`, copied with `cp`) after the checker (06)
put every row through the owner's tests: does the line say what the name is and whether it may change
it; does the word mean the same for every type; does any word mean two things, or do two words mean one
thing; does it read right where it sits. Same columns as the draft. Every row the checker changed
carries **06:** in its verdict cell, and section 6 lists every change with its reason and its evidence.
Rows with no **06:** mark passed as the builder wrote them. Agreed rows 3-9 (section 1) are exactly as
in `inputs/JOB.md`.

Power: `PowerLineStatus` printed `Online` for the builder, and again for the checker at the start and
the end of the check.

## 0. Terms used here

- **word**: anything a programmer writes to say something: a keyword (`owned`), an `@` attribute
  (`@noalloc`), or a library type or function that acts as a language word (`Box`, `mem.take`).
- **the owner's test**: for every name in a line, the word in the line that says what it is (the
  owner's own thing, a copy, or handed over) and the word that says whether it may change it (brief).
- **plain English**: a programmer who knows Python, not Rust, can tell what the word does from the word.
- **on the thing**: the word sits on what it describes and names it (`fn() with copied score:` passes,
  `copied fn():` does not).
- **handed over**: the giver's name may not be used afterwards; the receiver owns the value.
- **view**: a value that points into something else and may not outlive it (`ref T`, `Span[T]`, `str`).
- **class handle**: what a variable of a `class` type holds: a counted pointer to one shared object.
- **`Copy` type**: a type whose values are copied bit for bit on assignment (numbers, `bool`, `ref T`,
  `Span`, today also class handles).
- **placeholder**: a spelling proposed here only so the row can be read; 06 checks it and 07 (the word
  planner) fixes the final spelling. Agreed spellings (rows 3-9) are not placeholders.
- **area**: the phase-2 design area that owns a question: A lists, loops, temporary values; B references
  kept or returned, locks, objects still being built; C ownership and sharing, `owned` replacing `Box`,
  copies, hand-over, state machines; D linked objects, callbacks, interface types; E threads, locks
  across threads, `unsafe`.
- **AN-nn**: an annoyance id in `RUN3/annoyances.md`.

Verdicts: **keep** (the default), **rename**, **merge** (two words for one meaning become one),
**remove**, and **new** (a word the agreed rows or the owner's rulings bring). A rename, merge or
remove names the test that failed and why a programmer would care.

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

## 2. Disagreements between readers, and how each was settled

| # | word | readers said | settled | why |
|---|---|---|---|---|
| 1 | `Sync`, `@sync`, `@nosync`, the effect `Sync` (**06: overturned, see C-4**: the property `Sync` -> `ThreadShared` and `@sync` -> `@thread_shared`, one family; the effect `Sync` and `@nosync` kept, as reader 03 said) | 01: rename `@nosync` and the effect, keep `Sync`, `@sync`. 02: defers to 03. 03: keep the effect and `@nosync`, rename `@sync` and the property `Sync`. 04: rename the effect and `@nosync`, keep the rest | **effect `Sync` -> `Synchronise`, `@nosync` -> `@nosynchronise` (placeholders); `Sync` (the property) kept; `@sync` renamed (placeholder `@thread_shared`)** | All four agree the one word `Sync` has three meanings (spec 3184-3186 effect; 3445 property; 3451-3475 marker). Renaming the effect touches the fewest uses (7 `@nosync`, the `effects=[...]` lists) and leaves Rust's well-known `Sync` with its Rust meaning, as 01 and 04 say. 03's evidence against `@sync` itself stands on its own: the spec has to forbid the word's natural reading (`[THR-7]`, 3473-3475: "does not make methods atomic", "MUST NOT describe `@sync` ... as thread-safe"), so the attribute fails plain English and a programmer reading `@sync class Stats` as "synchronised" writes a read-then-write race across two `Atomic` fields. The property `Sync` is rarely written (bounds, messages) and is Rust's exact term; it is kept, and area E may remove all three from what programmers write (last run's area E wrote none, `phase3b-threads.md:24-40`) |
| 2 | `str` | 01, 02, 04 rename; 03 keep ("the capital letter is the only sign"; row 5 uses `String`) | **rename (placeholder `StrView`)** | Three of four; plain English fails in the worst direction: Python's `str` is the owned string, Ember's is a borrowed view; the spec keeps a whole catalogued Python-habit error for it (P26, 5461). 03's reason concerns `String`, which stays; row 5 writes no `str` |
| 3 | `Cell` | 02 rename (`Settable`); 01, 03, 04 keep | **keep** | No hidden cost and no panic; the change is visible at the use (`.set(...)`). 02's real point, that `s.frame.set(...)` changes `s` with no word on `s`, is the same problem as any class object changed through a plain parameter: AN-20, area B |
| 4 | `let` (field) | 01 rename or one meaning for every type; 02 keep, change the meaning for value-type fields; 04 rename (`fixed`) | **keep the spelling; it will mean "never changes after `init`, contents included", for every type** | The failed test is the meaning (`let items: Array[int]` still takes `push`, `[CLS-9a]` 1749), not the spelling: Swift's `let` already means exactly this (a `let` array cannot be appended to). 04's `fixed` stays the alternative for 06 if the checker finds the JavaScript reading ("`let` = changeable") weighs more |
| 5 | `virtual` | 01 merge into `open`; 02, 04 keep | **keep** | One meaning, on the method it describes; the merge is a style change outside the memory model. 04's W2111 point (accepted where it does nothing) is kept as AN-70 |
| 6 | `@static_safe` | 01, 03, 04 rename (`@no_alias_checks`); 02 keep (defers) | **rename (placeholder `@no_alias_checks`)** | Says "all safety proved when compiling"; forbids only aliasing checks (`[EFF-12]`, 3260); a programmer who writes it on a hot loop still gets bounds checks |
| 7 | `@fp(contract)` | 03, 04 rename the argument (`@fp(fuse)`); 01, 02 keep | **rename the argument (placeholder `@fp(fuse)`), keep `@fp`** | "contract" means a checked promise everywhere else in Ember (Part X, glossary 6890) and here means "allow fused multiply-add", which reads as the opposite ("strict") |
| 8 | `@realtime` | 03 rename (`@realtime_safe`); 01, 04 keep | **keep** | It names the manifest's contract set; 01's condition stands: `ember inspect` shows the set it stands for. 03's alternative is noted for 06 |
| 9 | `@borrows` | 02 rename (a clause on the signature, `from p`); 04 keep | **rename (placeholder: a `from p` clause after the result type)** | It describes the result but sits on its own line above the function; agreed rows 3 and 7 put what a function changes or reads after the parameters, so the same place for what the result comes from reads as one family: `fn find_word(haystack: str, needle: str) -> Option[str] from haystack:`. Area B may infer it instead (last run did, `phase3b-references.md:26`) |
| 10 | `with h:` (bare, keeps a handle alive) | 02 merge into `mem.keep_alive(h)`; others keep `with` | **keep** | The bare block form means "hold this until the block ends" for every value: `with debug.zone("name"):` (4184) and `with h:` (2663) are the same meaning, as Python's `with lock:` is. `mem.keep_alive(h)` stays as the one-line form |
| 11 | `Handle` | 01 rename one meaning; 02, 03, 04 keep the type | **keep `Handle[T]`; the prose term "class handle" changes** | Only `Handle[T]` is written in code; the clash is in the spec's prose and in messages (E2150, E3080, `[PRF-1]`), which 15 rewrites. AN-50 |
| 12 | `assert_disjoint` | 02 rename (`check_disjoint`; `assert_disjoint` for the panicking form); 04 keep (as E3095's help) | **rename (placeholder `check_disjoint`)** | It returns `Result` and never stops the program (3145-3174); a Python reader expects `assert` to stop it. 04 judged the name in a help line, not its behaviour |
| 13 | `MutSpan` with `mut` (**06: amended, see C-1**: re-pointing the caller's view is written `ref mut s: MutSpan[T]`) | 02 keep both, flag; 03, 04 merge | **keep both words, each with one meaning: `MutSpan` alone allows writing its elements; `mut` on any parameter still means "the caller's own variable may be changed"** | That is what the spec's own examples already assume (`[BCK-6]` 6045-6052, the `@parallel` example 3576), which today's compiler refuses (AN-11). The double word was the compiler's, not the spec's |
| 14 | `ref`/`ref mut` as a parameter's type vs `mut x: T` (**06: overturned, see C-1**: the in-out parameter is written `ref mut x: T`, row 8's words on the name; `mut x: T` and `x: ref mut T` are refused pointing at it) | 01 refuse the parameter-type form; 03 pick one spelling | **`x: ref T` and `x: ref mut T` as a parameter's type are refused, pointing at `x: T` / `mut x: T`** | `mut x: T` is the spelling Part V uses everywhere (`[FN-1]`); `x: ref mut int` cannot be written through today and its help does not compile (01 t3, t3c) |
| 15 | `.clone()` | 01, 02, 04 merge into `copied`; 03 merge, the method stays as what `Clone` provides | **merge: callers write `copied x`; `Clone` stays as the interface and its `clone` method is what `copied` runs** | One spelling at the use (agreed row 9); the interface is how a type says how it is copied |
| 16 | `Shared[T]` vs agreed `shared` | 01, 04 keep, the word must agree; 02 make them one word; 03 merge | **merge into `shared`** (row 9's placeholder), written on a value or a type, as `owned` replaces `Box` | One spelling for one meaning; `mesh: shared Mesh` and `mesh: owned Mesh` then read as a pair |
| 17 | `owned` on a parameter, by type (**06: amended, see C-2 and C-3**: `owned` is refused where the type is known to be `Copy`; a class handle is not `Copy`, so `owned` hands it over) | 01: for a `Copy` type and a class handle the copy cannot be told from a hand-over; 02: three meanings by type, make it hand-over for every type | **hand-over for every type that is not `Copy`; a `Copy` value is copied by its type's rule, as at `b = a`; `mem.drop(x)` ends `x` for every type** | 01 is right for a number and wrong for a class handle: the giver still sees the object change through its own handle. The meaning per type then differs only by the `Copy` property, which `b = a` shares; whether a class handle stays `Copy` (a copy that shares) is AN-44, area C. Today's text even disagrees with itself (`[FN-1]` 1561 "without a retain", `[FN-9]` 1573 "retained") |
| 18 | `unsafe` | 01, 04 keep the word, split the two directions (area E); 02, 03 keep | **keep; the split goes to area E** | All keep the word; AN-79 |
| 19 | `RefCell` placeholder | 02 `Checked[T]` with `read`/`write`; 03 `RuntimeChecked[T]`; 01, 04 any name saying "checked while running" | **rename (placeholder `RuntimeChecked[T]`, guards `ReadGuard`/`WriteGuard`, methods `read`/`write`)** | 03's name says the cost; 02's method names are `RwLock`'s, which `[CELL-8]` (3132) says have the same shape. Area C may remove the type (last run did, `phase3b-ownership.md:62`) |
| 20 | `mut self` on a class (**06: kept; `mut` on any other name moves to `ref mut`, see C-1**) | 01: row 3's job; 02: replace with a `changes all` clause, refuse `mut self` on a class | **`mut self` keeps one meaning for every type: "this method may change any field of the receiver"; `changes <fields>` (row 3) narrows it** | `mut` on any parameter already means "may change the caller's own value"; a separate `changes all` would be a second spelling of that. Its class-only meaning today, "hold a write access on every field for the whole call" (`[CLS-7]` 1740), goes |

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
| `class` | a type whose values are shared, counted handles to one heap object (261, 854, 1162) | same declaration; what `=` does with a class value is AN-44 (area C) **06:** whatever area C decides, a second name for a class object made with no word (`p2 = player`) may not silently share it: the brief allows a name with no word only where it "never aliases"; it hands over (as row 9's `snapshot = items`) or is refused naming `copied` and `shared` | before a type name | — | as today | keep: says "shared object" once, on the declaration, as Python's classes do (01, 02, 04) **06: keep, with the condition in "it will mean"** (C-3) |
| `comptime` | code run when compiling (584, 739, 3991-4016) | same; a heap value copied at each run-time use (4038-4040) gets a written word (AN-46) | before a block or in `comptime(e)` | — | as today | keep |
| `const` | constant fixed when compiling; const generic parameter (549, 577, 1197) | same | before a name | — | as today | keep |
| `continue` | next round of the loop (691) | same | in a loop | — | as today | keep |
| `defer` | run a block when the enclosing block exits (655, 2015) | same | before a block | — | as today | keep |
| `dyn` | an interface value whose concrete type is chosen while running, behind a pointer (608, 857, 1435) | same; one owner is written `owned dyn I` (replaces `Box[dyn I]`); a shared one gets a word too (AN-66, area D) **06:** the shared one is `shared dyn I`, from the `shared` row: AN-66 needs no new word | before an interface name | — | `dyn I` of an interface whose method is generic and returns or stores its type parameter (E2050), as Rust | keep: the owner wrote it himself (`no_share(dyn Pool)`, JOB) **06: keep** (C-9) |
| `elif` | else-if (645) | same | after an `if` block | — | as today | keep |
| `else` | `if`'s else; loop `else` (runs when no `break`); conditional expression (645-648, 693) | same | after a block | — | as today | keep: Python's three uses |
| `enum` | a type with variants (565, 852) | same | before a type name | — | as today | keep |
| `extern` | defined outside, C calling convention: foreign block, `extern "C" fn` type, `extern type`, `extern static` (544, 615, 4728-4731) | same | before a block, a type or a function | — | as today | keep |
| `false` | the `bool` value false | same | as a value | — | as today | keep |
| `fn` | function, method, lambda, callable type (546, 615, 692) | same; a lambda's capture list follows its parameters (`fn() with copied score:`, row 6) **06:** a lambda parameter that changes the caller's value is `fn(ref mut n: int)` (C-1) | before a function or lambda | — | the `owned fn` prefix (row `owned fn`) | keep **06: keep** |
| `for` | loop over an iterable; the loop name is a read-only reference to each element, but for class elements a handle that may write the object (648, 1966-1997, 2566) | same; **06:** the loop name, like every other name, never changes anything unless it is written `ref mut`: `for ref mut p in players: p.health = 100.0` (row 8's words on the loop name; C-1). Today this form is accepted and silently read-only (`k6_for_ref_mut.em`, E3021 at the write, section 6.2): it must do what its words say | before a loop | — | changing the list being walked (an understandable limitation; delete after the loop, JOB); **06:** `for v in xs.iter_mut():`, with "write `for ref mut v in xs:`" (one spelling in a loop header, on the name) | keep **06: keep; AN-20's loop case needs no new word** (C-1) |
| `if` | condition; `if pattern = e`; guard; conditional expression; comprehension filter (645-652, 693, 749) | same | before a condition | — | as today | keep |
| `implements` | a type implements an interface (556, 1229) | same | in a type header or `extend` | — | as today | keep |
| `import` | import a module; `import c "h.h"` (503-505) | same | at the top of a file | — | as today | keep |
| `in` | membership `x in c`; `for x in`; range type `f32 in 0.0..=1.0` (579, 648, 697) | same | between two values; in `for`; in a type alias | — | as today | keep: three uses, each English "in" |
| `interface` | declares an interface (571) | same | before an interface name | — | as today | keep |
| `is` | identity, `is None`, `is not` (466, 697); method name `is[E]()` (397) | same | between two values | — | as today | keep |
| `let` | a class field assignable only in `init`; its contents may still change (`self.items.push(v)`, `[CLS-9]` 1748, `[CLS-9a]` 1749); a local `let x = 5` is refused (E0100, good help) | **06 (stated as one rule):** after `init`, **nothing is changed through this field**, for every type: a number or struct field never changes; a list field is never pushed to; a class-object field is never re-pointed and its object is never changed through it (other owners of a `shared` object are `shared`'s business, not `let`'s) | before a field | — | assigning or changing a `let` field outside `init`, with a message that gives the line (E2104's new help: "set it in `init`, or remove `let` from its declaration") | keep the spelling, change the meaning (settlement 4): fails "says whether it may change, for every type" today (01, 02, 04; 02's t2 accepts `push`) **06: keep; meaning reworded so it is one rule for every type** (the draft said "contents included" for a list but only "never re-pointed" for a handle field: two meanings by type; C-8) |
| `match` | pattern match; `match owned e:` binds by move (651, 839) | same | before a value | — | as today; `mut` in a pattern (row 8) | keep |
| `mut` | in-out parameter (the caller's own value changes, 1558); a class-handle parameter whose handle the callee may re-point (`[FN-9]`); `mut self`; `static mut`; inside `ref mut` and `*mut T`; a lambda parameter that writes (716); accepted in a pattern though the grammar lacks it, binding read-only (01 t1) | **06:** "may change", and nothing else, in exactly these places: `mut self` (the word `self` says what it is: the value the method was called on), `ref mut` (row 8), `static mut`, `*mut T`. **Not on a parameter's or lambda parameter's name**: the in-out parameter is written `ref mut b: B` (C-1) | in `mut self`; after `ref`; after `static`; after `*` | — | `mut` in a pattern (row 8: "write `State.Walking(ref mut path):`"); **06:** `mut b: B` on a parameter and `fn(mut n)` on a lambda parameter, with "to change the caller's `b` through `b`, write `ref mut b: B`" (the same advice shape as row 8); `mut self` on `init` ("`init` may always set fields; write `fn init(self, ...)`") | **06: keep the word, move its parameter use to `ref mut` (C-1).** Row 8's own test, applied to a parameter: in `fn bump(mut b: Counter)` the word `mut` says "may change" and no word says "the caller's own `b`", exactly as `State.Walking(mut path):` failed (in Rust `fn f(mut b: B)` is a changeable copy). Today `mut` on a struct parameter changes the caller's fields and on a class parameter re-points the caller's handle and "opens no access on the object" (`[FN-9]` 1570), while `mut self` on a class changes the object's fields (settlement 20): two meanings by type (`k4_mut_param_two_meanings.em`, accepted). With `ref mut` on the name the one meaning is row 8's: a name for someone else's place that may change it |
| `mut self` | struct: may change any field (1633); class: holds a write access on every field for the whole call (`[CLS-7]` 1740, `[EXC-15]`); `init`: accepted, means nothing more (1658, 1717) | **may change any field of the receiver, for every type**; `changes <fields>` narrows it (row 3) | on a method's receiver | the class meaning (whole-call write access) | `mut self` on `init` (02); on a `@sync`/`@thread_shared` class (E7003, as today) | keep (settlement 20) **06: keep**: `self` is the word that says what the name is (the value the method was called on), `mut` the word that says it may change; the same meaning as `ref mut` on any other name |
| `not` | logical not; `not in`; method name `not` (467, 697, 396) | same | before a condition | — | as today | keep |
| `or` | logical or | same | between conditions | — | as today | keep |
| `override` | this method replaces a virtual one; an overlay entry replacing an earlier one (545, 4762) | same | before `fn` | — | as today | keep |
| `owned` | (a) parameter "consumed": moved in, copied if `Copy`, a class handle copied without a retain (`[FN-1]` 1561) or as a retained copy the caller keeps (`[FN-9]` 1573); (b) `owned self`; (c) `for x in owned xs`, `match owned e`; (d) `fn(owned T)` callable-type parameter; (e) the `owned fn` lambda prefix (row `owned fn`); (f) C contract word "callee takes it" (4826); (g) a C++ `T&&` parameter (6592); (h) a lambda parameter inferred `owned` (`[TYP-23]` 1468) | **ownership handed over, nothing else** (JOB): the receiver owns what it gets and the giver's name ends, for every type that is not `Copy` (a `Copy` value is copied by its type's rule, as at `b = a`; settlement 17). Also **the one-owner type that replaces `Box`**: `mesh: owned Mesh` is one owner, kept in its own heap block (what `Box` did: an allocation the source shows, `[PHIL-2]`; an unsized value, `owned dyn I`; a recursive payload, `Option[owned Tree]`; one pointer in `Option[owned T]`; read through where a `T` is wanted, `[TYP-14]`). A capture handed over is written `fn() with owned names:` (02, 04; spelling for 07) | on a parameter, a receiver, a `for`/`match` source, a field or type, a capture name | `Box[T]`, `Box[dyn I]`, `ForeignBox[T]` (with `Foreign`), the `owned fn` prefix | using a name after it was handed over, with a message naming `copied` and `shared` (row 9's pattern); `owned` in an expression (`b = owned a`, E0109) with the help "`b = a` already hands `a` over"; **06:** `owned` written where the type is known to be `Copy` (`owned n: int`, `with owned score` for an `int`, `match owned e` of a number), because there it can only copy: "an `int` is copied, not handed over; remove `owned`" (for a capture: "write `with copied score`"). Today `owned n: int` is accepted and the caller goes on using `a` (`k1_owned_int.em`). **06:** `x: owned T` as a parameter's type with no mode on the name, which reads as a hand-over and only reads someone else's one-owner value: "to take it over write `owned x: T`; to read it write `x: T`" | keep (all readers), with the `owned fn` use removed and the `Box` use added by the owner's ruling **06: keep, amended (C-2, C-3).** A class handle is not `Copy` (C-3), so `owned h: Node` hands the handle over and the caller's `h` ends; today it is a retained copy the caller keeps using (`[FN-9]` 1570; `k2_owned_handle.em`, accepted), which is a share, not a hand-over. In a generic body (`owned v: T`) an instance whose `T` is `Copy` gives the callee its own copy and the giver keeps its own, as `b = a` does: nothing the program computes can tell the two apart there, unlike the owner's `owned fn` failure, where the copy took the change away from `game`'s `score` |
| `owned fn` | a lambda that moves its non-`Copy` captures and copies its `Copy` ones (`[CLO-2]` 2064; 692, 3495, 6221); accepted silently for an `int` (01 t2, 02 t1, 03 t3) | nothing: removed. A capture is the real variable unless named: `with copied x` (row 6) or `with owned x` (placeholder) | — | — | `owned fn(...)` itself, refused with "name what the function copies or takes: `fn() with copied score:`" | **remove**: the owner's named failure (JOB): one keyword copies an `int` and hands over a `String`, and sits on `fn`, not on the captured names |
| `pass` | empty statement or body (560, 633) | same | as a statement | — | as today | keep |
| `pub` | visible to dependants; `pub(package)`, `pub(read)` (538) | same | before an item or field | — | as today | keep |
| `ref` | a reference type `ref T` (read-only); a borrow expression `ref e`; a pattern `ref x`; a result `-> ref T` (607, 706, 828, 1112, 1336); as a parameter's type it means the same as no word (01 t6) | **a name for someone else's place that only reads it** (rows 4 and 8 use it) | before a type, a place or a pattern name | — | `x: ref T` as a parameter's type (settlement 14: "a parameter already reads the caller's value; write `x: T`"); `return ref g[0]` out of a lock (row 4); **06:** also `ref x: T` on a parameter's name, with the same advice | keep (all readers; agreed rows 4, 8) **06: keep** |
| `ref mut` | a reference that may change what it points at: type `ref mut T`, expression `ref mut e`, pattern `ref mut x`, result `-> ref mut T` (607, 706, 828, 1112-1117, 1906, 2280-2320; 52 uses in lines 1-6930); `r = e` on a `ref mut` local writes through it (`[BRW-1]` 2286, `[TYP-14]` 1116); as a parameter's type it cannot be written through (01 t3) | **a name for someone else's place that may change it** (row 8: "`ref mut` makes sense"): the same in a pattern, an expression, a result, a local, **and (06) on a parameter's name (`ref mut b: B`, the in-out parameter), a lambda parameter's name and a `for` name (`for ref mut v in xs:`)** | before a type, a place, a pattern name, a parameter name or a loop name | `Mut[T]` in ECS queries (with `Mut`); **06:** the in-out parameter `mut b: B`; `for v in xs.iter_mut():` in a loop header | `x: ref mut T` as a parameter's type (**06:** "write `ref mut x: T`": the word goes on the name, as in row 8); `r = ref mut b` on a `ref mut` local, accepted today and silently copying `b` into `r`'s target (section 5.1, AN-28): refused with "`r` always refers to `a`; this copies `b`'s value into `a`; to refer to `b`, use a new name: `rb = ref mut b`" | keep, judged here (section 5.1): one meaning in every position the owner's test looks at; the two forms that do not do what the words say are refused **06: keep, one more position each for parameters and loop names (C-1).** Today `fn bump(ref mut x: int)` is E0100 "expected a name" (`k5`) and `for ref mut v in xs` parses but binds a read-only `ref i64` (`k6`); both must do what row 8's words say |
| `return` | return from a function | same | as a statement | — | returning a view of a temporary (row 5) or into a lock (row 4) | keep |
| `self` | the receiver: read-only on a struct (02 t14), may write fields on a class (`[CLS-7]`; 01 t5, 02 t4); the frame-held handle in a `gen fn` | **the receiver, read-only for every type**; a method that changes fields says so with `changes <fields>` (row 3) or `mut self` (any field) | first parameter of a method | — | writing a field through plain `self` on a class (new): "`open` changes `self.angle`; write its first line as `fn open(self) changes angle:`" | keep the word (all readers); the class default is the change row 3 makes (AN-19) |
| `Self` | the implementing type (609) | same | in a type | — | as today | keep |
| `static` | lives for the whole program: a static item, the static region (578, 1130, 3478) | same | before an item | — | changing a static that is not `Atomic`/`Mutex` (AN-76, area E; message to fix) | keep |
| `struct` | a value type (556, 852) | same | before a type name | — | as today | keep |
| `super` | the base class (`super.init(...)`, 1728) | same | in a derived class | — | as today | keep |
| `true` | the `bool` value true | same | as a value | — | as today | keep |
| `type` | type alias, range type, associated type, opaque foreign type (574-583, 1024, 4728) | same | before a name | — | as today | keep |
| `unsafe` | a block where the programmer vouches for what the compiler cannot check (656); on `unsafe fn` / `unsafe fn(...)` / `unsafe interface` / `unsafe extern` / `unsafe extend` / `unsafe overlay`: whoever calls, implements or relies on it must vouch (149-154, 545, 571-576) | same word; area E decides whether the two directions ("I vouch here" / "you must vouch") get two spellings (Rust 2024 split them) | before a block or a declaration | — | an unsafe operation outside `unsafe` (E3100, with the line to write) | keep (all readers); AN-79 |
| `unsafe fn` | a function whose caller must vouch; its whole body is also an unchecked zone with no mark per operation (`[TIER-1]` 152; glossary 6916) | same declaration; area E decides whether its body must still mark each unsafe operation (Rust 2024's `unsafe_op_in_unsafe_fn`) | before `fn` | — | calling it outside `unsafe` | keep (01, 04): the declaration says what it does; the body rule is AN-79 |
| `virtual` | the method may be overridden; bodiless only in an `abstract class` (545, 589) | same | before `fn` | — | `virtual` in a class that cannot be subclassed (today W2111, accepted; AN-70): refused with "write `open class C` or remove `virtual`" | keep (settlement 5) |
| `void` | the unit type; its value is `()` (609, 1615) | same | in a type | — | as today | keep |
| `where` | generic bounds clause (551) | same | after a signature | — | as today | keep |
| `while` | loop | same | before a loop | — | as today | keep |
| `with` | (a) bind for a block and end it at the block's end (`with g = m.lock():`, `with scope = thread.scope():`, bare `with debug.zone("x"):`, `with h:`; 653, 2013, 2663, 3422, 4184); (b) options of `import c ... with (...)` (510, 4682) | (a) and (b) the same; (c) **new, row 6: the capture list of a lambda, `fn() with copied score:`** (and `with owned x`, placeholder): what the function copies or takes is named there, and anything not named is the real variable | before a block; after an import; after a lambda's parameters | — | as today | keep (all readers): three uses, each English "with these"; the block form (`with x = e:`) and the capture list (`fn() with copied x:`) cannot be confused (one binds `name = expr`, the other follows `fn(...)`) |
| `yield` | give a value from a generator (691, 2152) | same | in a `gen fn` | — | a borrow of a generator's own local list across `yield` (E2221; AN-77, area E) | keep |

### 3.2 Contextual and reserved-for-later words (`[LEX-15]`, 19), and the phrase `some I`

| word | today it means | it will mean | where it is written | replaces | stays refused | verdict and reason |
|---|---|---|---|---|---|---|
| `abstract` | `abstract class`: has bodiless `virtual` methods, cannot be built (557) | same | before `class` | — | as today | keep |
| `open` | `open class`: may be subclassed (557) | same | before `class` | — | as today | keep |
| `from` | `from m import x` (504); C result lifetime `from(self)` (4669); conversion `From[E]`, `String.from(s)` | same; plus (placeholder, settlement 9) the clause that says what a function's result points into: `-> Option[str] from haystack:` (replaces `@borrows`) | at the start of an import; after a result type | `@borrows` (placeholder) | as today | keep: every use reads "comes from" (03) |
| `extend` | add methods or an interface implementation to a type (576, 1237) | same | at the start of an item | — | as today | keep |
| `gen` | `gen fn`: a generator (545, 778) | same | before `fn` | — | as today | keep |
| `once` | `once fn`: may be called at most once; `callback(once)` (615, 4836) | same | before `fn` in a type | — | a closure that gives a capture away passed where it may run many times, with the help "declare the parameter `once fn()`" (AN-59) | keep |
| `safe` | `safe fn` in an extern block: the programmer asserts the call cannot break memory safety (581, 4594) | same | before `fn` in an `unsafe extern` block | — | as today | keep |
| `some` | at the start of a type: `some I` (392, 608) | same | at the start of a return type | — | see `some I` | keep |
| `some I` | "one concrete type, chosen by the function body, that implements `I`"; the caller sees only `I`, the compiler monomorphises through it (`[TYP-32]` 1443-1451; `[GRM-31]` 621-624; examples 1450, 2175, 3373, 6201); only in a return type (E2260), the same type at every `return` (E2261) | same | in a function's or method's return type | — | `some I` outside a return type (E2260, its help gives the line: "take a type parameter bounded by the interface: `fn f[T: I](x: T)`"); different types at two `return`s (E2261, its help says "return a `Box[dyn I]`", which becomes "`owned dyn I`") | keep, judged here (section 5.2): one meaning, plain English ("returns some iterator"), on the type it describes; it names no ownership or change, so the owner's test has nothing to ask of it |
| `c` | the C language: `import c "h"`, `@layout(c)`, `c"..."` literal (505, 4220) | same | after `import`/`overlay`; in `@layout` | — | as today | keep |
| `cpp` | the C++ language: `import cpp "h"` (529, 6458) | same | after `import`/`overlay` | — | as today | keep |
| `language` | `#! language "v"` directive (108, 501) | same | after `#!` | — | as today | keep |
| `threads` | `#! threads main` (or `any`, `creator`); C contract `threads(...)` (520, 4608) | same | after `#!`; in `@ffi`/overlay | — | as today | keep |
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
| `@sync` | a class whose handles may cross threads, whose counts are atomic and whose fields are fixed after `init` (`[THR-1]`, `[THR-7]` 3451-3475) | same meaning, placeholder spelling **`@thread_shared`**: the class's values are `ThreadShared` (06: the property, renamed from `Sync`, C-4); area E may remove the marker | on a class | — | writing such a class's field after `init` (E7003; help names `Atomic`/`Mutex`) | **rename** (03): fails plain English; the spec must forbid its natural reading ("does not make methods atomic", "MUST NOT describe ... as thread-safe", 3473-3475). **06: kept as a rename; the property it grants is renamed with it, so attribute and property are one family with one meaning (C-4)** |
| `@ffi` | one attribute for every fact about foreign code: `effects=`, `param(...)`, `result(...)`, `link_name=`, `handle`, `destroys(f)`... (1808, 3208, 4593-4884, 6511) | same | on a foreign item | — | as today | keep |
| `@derive` | ask the compiler to write `Copy`, `Clone`, `Eq`... (254, 1623) | same | on a type | — | `@derive(Copy)` with a non-`Copy` field (E2080, message to give the line) | keep |
| `@noalloc` | contract: no heap allocation (140, 3239) | same | on a function | — | as today | keep |
| `@parallel` | loop runs in chunks on the job system; `chunk=`, `reduce=` (679, 3577) | same | on a `for` | — | a parallel loop writing a shared place (E7010; AN-84 for its code) | keep |
| `@static_safe` | contract: no run-time *aliasing* checks; bounds, overflow and stale checks still run (`[EFF-12]` 3245, 3260) | the same contract, placeholder spelling **`@no_alias_checks`** | on a function | — | an aliasing check inside it (E4030) | **rename** (01, 03, 04): says "all safety proved when compiling" and forbids one kind of check |
| `@nopanic` | `@nopanic(explicit)`: no written panic; the bare form is reserved (E0104, good help) (3244, 3263) | same | on a function | — | the bare form, as today | keep |
| `@layout` | `@layout(c)`, `(c, union)`, `(c, flexible)`: C field order (1100, 4727) | same | on a struct | — | as today | keep |
| `@deterministic` | contract: no nondeterministic effect (148, 3307) | same | on a function | — | as today | keep |
| `@simd` | ask for vectorisation; `@simd(assert)` makes failure an error (681, 3729) | same | on a `for` | — | as today | keep |
| `@fastmath` | relaxed float rules in one function (1002, 3316) | same | on a function | — | inside `@deterministic` (E4072) | keep |
| `@ser` | serialisation options: `skip`, `rename=`, `default`, `since=` (1871, 4076) | same | on a field | — | as today | keep |
| `@fp` | `@fp(contract)`: allow fused multiply-add (1003, 3332, 4492) | same, the argument spelled **`@fp(fuse)`** (placeholder) | on a function | — | as today | **rename** the argument (settlement 7): "contract" means a checked promise elsewhere |
| `@borrows` | the result borrows exactly these parameters (2372-2398) | the same fact as a clause after the result type, placeholder **`from p`** (settlement 9); area B may make it inferred | after the result type | — | the result pointing into something the clause does not name (E2031, message to give the line) | **rename** (place and spelling): describes the result and does not sit on it (02) |
| `@noreload` | hot reload: this function is never reloaded (1866, 6311) | same | on a function | — | as today | keep |
| `@must_drop` | the value's `drop` must run: no `forget`, no `yield` while live (1861, 3533) | same | on a type | — | leaking it (E3015) | keep |
| `@export` | C symbol for an Ember function (1853, 4940) | same | on a function | — | as today | keep |
| `@view` | checked documentation that a type holds a view (1121-1124, 2325) | same; `ember fmt --migrate` must not remove it (04: `[CLI-20]` 5067 contradicts `[TYP-34]`) | on a type | — | `@view` on a type that holds no view (E2030, help: "remove `@view`") | keep |
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
| `@safety` | what callers of an `unsafe fn` must ensure (1860, 5850) | same | on an `unsafe fn` | — | as today (L3015, L3016) | keep |
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
| `@no_runtime_checks` | reserved "naming the version" (1875-1876), while `[EXC-1]` (2708) and `[DSJ-8]` (3173) say no such switch exists | nothing | — | — | the name, as an unknown attribute | **remove** (02): reads as a promise of a switch that turns safety checks off, which the spec says will never exist |
| `@name` | not an attribute: the grammar's placeholder in `@name(args)` (524) | — | — | — | — | keep (nothing to judge) |
| `@dataclass` | not an Ember word: Python's, quoted in Appendix E (6766), where Ember writes `struct` | — | — | — | — | keep (nothing to judge) |
| `@bench` | a benchmark function (1855) | same | on a function | — | as today | keep |
| `@allocator` | reserved (1876) | same | — | — | as today | keep (reserved) |

### 3.4 Library words that act as language words (must-cover list)

| word | today it means | it will mean | where it is written | replaces | stays refused | verdict and reason |
|---|---|---|---|---|---|---|
| `Box` | one owner on the heap: `Box[T]`, `Box[dyn I]`, `Box[T, A]` with an allocator (163, 857, 1105-1117, 1354, 1476, 2179, 2986) | nothing: `owned T` (owner's ruling, JOB). `Box[dyn I]` -> `owned dyn I`; `Option[Box[T]]` -> `Option[owned T]`; the allocator form and `b.get()` (2893, 2986) need a place in the new form (02) | — | — | `Box` itself, with "write `owned T`" | **merge** into `owned`: the owner's ruling; plain English fails ("box" says nothing about one owner) |
| `Box[dyn I]` | a one-owner value of some type implementing `I` (857, 1435, 5457) | `owned dyn I` | in a field, parameter or result type | `Box[dyn I]` | as for `Box` | **merge** into `owned dyn I` (owner: `no_share(dyn Pool)` "or a better word") |
| `Shared` | `Shared[T]`: counted shared ownership on one thread; `get`/`get_mut` checked while running (163, 2611-2621, 2901) | `shared` (row 9's word), written on a value (`other = shared items`) or a type (`mesh: shared Mesh`): several names own one value, which lives until the last one ends | before a value; in a field or type | `Shared[T]` | changing a `shared` value from two threads without a lock (area E) | **merge** into `shared` (settlement 16) |
| `Weak` | a handle that does not keep its object alive (301, 1163, 2594) | same; area D may make it a field word (last run: `weak`) | in a type | — | as today | keep: says "does not keep it alive" (JOB names that distinction) |
| `RefCell` | a value whose borrows are counted while running; a second conflicting borrow panics (185, 241, 1257, 3099-3137) | the same type, placeholder spelling **`RuntimeChecked[T]`**; area C may remove it | in a type | — | a second conflicting access (panic, as today) | **rename** (all four readers): the name hides the run-time check and the panic |
| `Cell` | a value replaced whole through a shared path; never lends a reference (185, 1383, 3099-3124) | same | in a type | — | as today | keep (settlement 3) **06: keep, with a condition for area B (C-10):** a `Cell` field is changed through a plain name today (`k10_cell_through_plain.em`, `fn tick(s: S): s.frame.set(...)`, accepted), so the default "a plain name never changes anything" (brief) is only true if the rule says so: either the default is stated as "never changes anything except a `Cell` field, which its declaration says may be set through any name", or setting it needs `ref mut s` / `changes frame`. Area B rules with AN-20 |
| `Span` | a read-only view of contiguous elements: pointer and length (1118) | same | in a type | — | storing one where its source may end (as today) | keep |
| `MutSpan` | a changeable view; today a `MutSpan` parameter cannot be written through without `mut` as well (01-04; 04 t18-t21; spec 3576 and `[BCK-6]` 6045 write it without `mut`) | **a view whose elements may be written, with no other word**; **06:** re-pointing the caller's own view is `ref mut s: MutSpan[T]` (C-1) | in a type | — | as today otherwise | keep (settlement 13); the compiler's refusal of the spec's own examples is AN-11 **06: keep, amended for C-1** |
| `str` | a borrowed view of UTF-8 text (449, 855, 1120, 1130-1140, 4214) | same type, placeholder spelling **`StrView`** | in a type | — | storing it in a field fed by a `String` (E3063, as today; AN-26) | **rename** (settlement 2) |
| `String` | an owned, growable text (163, 262, 853) | same; plus the constructor `String(x)` (row 5) | in a type; as a constructor | — | as today | keep (all readers; row 5 uses it) |
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
| `Handle` | `Handle[T]`: an index and a generation into a `Pool` (244, 3089) | same; the prose term "class handle" in the spec and messages gets another word (15) | in a type | — | a stale handle (panic, as today) | keep (settlement 11) |
| `Send` | a type that may be moved to another thread (246, 3434) | same; area E may stop programmers writing it | in a bound | — | as today | keep (all readers) |
| `Sync` | (a) a property: several threads may read one value at once (`[THR-9]` 3445); (b) the name of an effect: atomics, locks, channels, thread work (3184-3186) | (a) only, placeholder spelling **`ThreadShared`** (06): "several threads may hold and read one value of it at once"; `@thread_shared` on a class makes it so | in a bound | — | as today | **06: rename (C-4; the draft kept it).** The draft renamed `@sync` because "sync" reads as "synchronised"; the property is spelled with the same word and means the same thing as `@sync` (a `@sync` class's handles are `Sync`, `[THR-9]` 3444-3448), so it fails the same test, and keeping it beside `@thread_shared` would give one meaning two words. Rarely written (bounds, messages), so the rename is cheap |
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
| `SAFETY:` | a comment the compiler reads above an `unsafe` block (347, 359) | same | as a comment | — | an `unsafe` block without it (W3012) | keep |
| `r#name` | a raw name for a keyword (374) | same | as a name | — | as today | keep |
| `Copy` | values copied bit for bit on assignment; covers class handles, `Shared`, `Weak`, whose copy is a share (167, 852, 2240, 2270) | **06:** values whose copy is independent of the original, copied bit for bit on assignment: numbers, `bool`, `ref T`, `Span`, structs of such. **Not** class handles, `shared` values or `Weak`: their second name is a share, not a copy (C-3) | in a bound or `@derive` | — | as today | **06: keep the word, narrow the meaning (C-3).** Today `Copy` covers class handles, `Shared` and `Weak`, "whose copy is a share" (02; `[OWN-7]`, ODR-051): one word, two meanings, and the share is made silently. A struct holding a class handle is then not `Copy` either (`@derive(Copy)` on it is E2080). AN-44 (area C) is answered under this meaning |
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
| `cstr` | a borrowed C string view, non-null (422, 1105) | same type, placeholder spelling **`CStrView`** (06) | in a type | — | as today | **06: rename (C-5; the draft kept it and left it to 07).** `cstr` beside `CString` is the same pair as `str` beside `String`, told apart only by length and a capital, and fails plain English the same way (a borrowed view named like the owned text); `str` -> `StrView` without `cstr` -> `CStrView` would make the two text families read differently |
| `CString` | owned NUL-terminated C text (4866) | same | in a type | — | as today | keep |
| `UnsafeCell` | the unchecked base of the cells (1257, 3045) | same | in a type | — | as today (`unsafe`) | keep: the name says "unsafe" (01) |
| `Ref` | read guard of a `RefCell` (2510, 3125) | `ReadGuard` (placeholder, follows `RefCell`) | in a type | — | as `RefCell` | **rename** with `RefCell` (02) |
| `RefMut` | write guard of a `RefCell` (3125) | `WriteGuard` (placeholder) | in a type | — | as `RefCell` | **rename** with `RefCell` (02) |
| `borrow` | open a read guard on a `RefCell` (3112) | `read` (placeholder; `RwLock`'s name, same shape, `[CELL-8]`) | as a method | — | as `RefCell` | **rename** with `RefCell` (02): "borrow" is Rust jargon |
| `borrow_mut` | open a write guard on a `RefCell` (3125) | `write` (placeholder) | as a method | — | as `RefCell` | **rename** with `RefCell` (02) |
| `SyncShared` | `Shared` across threads, atomic count, read-only payload (2615, 3441) | `shared`, with the thread family's word (**06:** `ThreadShared`, C-4) for a value shared between threads; 07 spells the combination | in a type | — | as today | **merge** into `shared` (settlement 16; inherits settlement 1) **06: keep the merge** |
| `ForeignBox` | a C object this value owns and destroys through its contract's destructor (4185, 4725, 4826, 6508) | the same thing, placeholder spelling **`Foreign[T]`** (one owner, like any non-`Copy` value) | in a type | — | as today | **rename** with `Box` (03, 04): named after the word the owner removed |
| `adopt` | an `unsafe fn` taking ownership of a raw foreign pointer (4883, 6563) | same | as a call | — | adopting twice (E5068) | keep |
| `upgrade` | `Weak`: the object if it is still alive (2604, 2660) | same, placeholder spelling **`get`** (returns `Option`) | as a method | — | as today | **rename** (02): Rust jargon; Python's `weakref` is simply called. **06: kept as a rename, with a condition (C-6):** `get` elsewhere (`Array.get`, `SoA.get`) gives a reference that does not keep anything alive; if area D makes `w.get()` hand back a new owner that keeps the target alive, the name `p` in `p = w.get()` has no word saying so, and the spelling must say it |
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
| `get_mut` | a changeable reference into a `Shared` or a container element (2901, 2955) | same | as a method | — | as today | keep |
| `iter_mut` | iterate yielding `ref mut` (2313, 2522) | same | as a method | — | as today | keep **06:** kept for iterator chains (`xs.iter_mut().for_each(...)`, where the result type carries the meaning, as row 5's `word = line.to_lower().trim()` does); in a `for` header the one spelling is `for ref mut v in xs:` (C-1). Today `for v in xs.iter_mut(): v += 1` is accepted (`k9_iter_mut.em`) |
| `as_mut` | `Option.as_mut(mut self) -> Option[ref mut T]` (3903) | same | as a method | — | as today | keep |
| `as_mut_span` | a changeable view of an `Array` (1563) | same | as a method | — | as today | keep |
| `reborrow` | lend a `MutSpan` on again (2539) | same | as a method | — | as today | keep (rarely written) |
| `get_unchecked` | index with no check, inside `unsafe` (2538) | same | as a method | — | outside `unsafe` | keep |
| `assert_disjoint` | check two views do not overlap and return `Ok`/`Err`; never panics (3145-3169) | same, placeholder spelling **`check_disjoint`**; the panicking `assert_disjoint_or_panic` becomes `assert_disjoint` | as a call | — | as today | **rename** (settlement 12) |
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
| `SoA.get` | returns a **copy** of element `i`, while `Array.get(i)` returns a reference (3672, 3686 vs 4285; 03 t6) | a reference proxy, as on `Array`; a copy is written `copied ps[i]` (03) | as a method | — | as today | keep the spelling, change the meaning (03): the same method name copies on one type and borrows on another, and the copy is not named (rows 6, 9: "a copy is only made when named"). AN-13 |
| `SoARef` | the read-only element proxy of a `SoA` (3668-3697) | same | in a type | — | as today | keep |
| `SoAMut` | the changeable element proxy of a `SoA` (3668-3697) | same | in a type | — | as today | keep |
| `Query` | an ECS query (3787) | same | in a type | — | as today | keep |
| `Mut` | ECS query marker: `Mut[T]` gives `ref mut T` (3787-3838) | `ref mut T` in the query, if area A allows a reference type as a query argument (`[ECS-3]` 3826 says the language has no access-mode generic) | in a query type | — | as today | **merge** into `ref mut` (03): two spellings of one meaning. AN-15 |
| `Not` | ECS query marker excluding a component (3787) | same | in a query type | — | as today | keep |
| `Commands` | ECS: changes queued until the loop ends (3787) | same: the agreed "delete after the loop" (JOB, Bevy's `commands.entity(e).despawn()`) | in a system's parameters | — | as today | keep |
| `Scope` | `thread.scope()`: tasks joined at the block's end (3503) | same | after `with` | — | as today | keep |
| `JobScope` | `jobs.scope()` (3619) | same | after `with` | — | as today | keep |
| `JoinHandle` | a spawned thread's result (3503) | same | in a type | — | as today | keep |
| `AnyError` | an owned `dyn` error with a context chain (3932) | same | in a type | — | as today | keep |
| `ObjectPtr` | the address of a class object for C (4926) | same | in a type | — | as today | keep |
| `Entity` | a 32-bit ECS entity (3811) | same | in a type | — | as today | keep |
| `OldSelf` | the previous version of a type in hot reload (6370) | same | in `migrate_from` | — | as today | keep |
| `migrate_from` | hot reload: build the new value from the old (6370) | same | as a method | — | as today | keep |
| `CppShared` | a C++ `shared_ptr` (6506) | same; if `Shared` becomes `shared`, 07 decides whether it follows | in a type | — | not convertible to `shared` (E5065) | keep |
| `CppWeak` | a C++ `weak_ptr` (6506) | same | in a type | — | as `CppShared` | keep |
| `borrowed` | C contract word: valid for the call (4826) | same | in an overlay or `@ffi(param(...))` | — | as today | keep (03: every contract word has one meaning, stated in a table) |
| `returns_owned` | C contract word: the result is owned and freed by `destructor=f` (4826) | same | as above | — | as today | keep |
| `retained` | C contract word: the callee keeps the pointer (4826) | same | as above | — | a kept pointer not declared `retained` (E5051) | keep |
| `exclusive` | C contract word: the pointer is the only one (4830) | same | as above | — | as today | keep |
| `aliased` | C contract word: other pointers may reach it (4830) | same | as above | — | as today | keep |
| `TODO(...)` | an unknown foreign fact in a starter overlay (5101) | same | in an overlay | — | `@safety("TODO")` left in (L3016) | keep |
| `as?` | the spec's checked downcast to `Option` (810, 945, 1164) | nothing: today's compiler refuses it for `cast x if D` (section 5.4) | — | — | `as?`, with today's help (it already gives the line) | **merge** into `cast` (the spec text is behind the compiler, ODR-280) |
| `as!` | the spec's downcast or panic (810, 945) | nothing: `cast x as D` | — | — | as `as?` | **merge** into `cast` |
| `cast` | today's compiler only (ODR-280): `cast x as D` gives the `D` or panics, `cast x if D` gives `Some(d)` or `None` (section 5.4); not in the spec text | same; the spec must gain it; area D answers its annoyances (AN-67, AN-68) | before a value | `as?`, `as!` | a cast between unrelated types (E2020) | keep |

### 3.6 New words from the agreed rows and the owner's rulings

| word | today it means | it will mean | where it is written | replaces | stays refused | verdict and reason |
|---|---|---|---|---|---|---|
| `copied` | not a word: `snapshot = copied items` is E1010 "cannot find `copied`" (section 5.3) | **a new, independent value equal to the old one; nothing done to one is seen through the other** (rows 6, 9). Written before a value (`snapshot = copied items`, `return copied g[0]`) and in a lambda's capture list (`fn() with copied score:`) | before a value; after `with` in a capture list | `.clone()`, `.copy()`, the copy half of `owned fn` | `copied` of a class object whose class does not implement `Clone`; `copied` of a view (`str`, `Span`), whose text belongs to someone else: "to make your own copy write `String(word)`" (row 5's spelling), so `copied` never means two things; **06:** for a `Span[T]` the message names the owned list ("to make your own copy write an `Array[T]` of it"), as E3063's help already does. (Today's `s.to_array()` cannot be called on a `Span` at all: E3063 inside `std/src/core.em:988`, `k8_copied_span.em`; a library defect for the lead, not a word) | **new** (rows 6 and 9; owner: "use the copied keyword for it") **06: passes** every test: it sits on the value or captured name it copies, means one thing for every type it is allowed on, and is refused where it could not |
| `shared` | not a word: `other = shared items` is E1010 (section 5.3) | **several names own one value, which lives until the last of them ends** (row 9's other way out). Written before a value (`other = shared items`) and on a field or type (`mesh: shared Mesh`, `shared dyn I`) **06:** after `other = shared items` both names own the one list; the word sits on `items`, the name whose value becomes shared, so the line says it. A class object is shared by its declaration today; that default is AN-44's, under the `class` row's condition (C-3). `shared dyn I` answers AN-66 (C-9). Between threads, the family is `ThreadShared` / `@thread_shared` (C-4) | before a value; in a field or type | `Shared[T]`, `SyncShared[T]` | sharing between threads without the word area E gives it | **new** (row 9 names it; settlement 16) **06: passes** |
| `changes` | not a word: `fn note(self, msg: String) changes log:` is E0100/E0110 (section 5.3) | **the fields this method may change**; checked against its body; a loop over another field may call it (row 3) | after a method's parameters, before `->` or `:` | the class meaning of `mut self` (whole-call write access); `[BRW-10]`'s field set inferred only for private methods | the body changing a field not named; calling it while a loop walks a named field | **new** (row 3, spelling agreed) |
| `reads` | not a word: `fn load_assets(self) reads renderer -> Array[String]:` is E0100/E0110 (section 5.3) | **the fields this helper reads**; checked against its body and at the call: `init` may call it once those fields are set (row 7) | after a method's parameters, before `->` | — | calling it before a named field is set (with E2100's new help); the body reading a field not named | **new** (row 7, spelling agreed) |
| `String(...)` | refused: `String(word)` is E2020 "`String()` takes no arguments", no help (03 t9, 04 t13); `String.from(s)` and `s.to_string()` work | **your own `String` made from a text view** (the same as `String.from(s)`) | as a constructor | — | — | **new** (row 5's message names it; the agreed spelling needs the constructor) **06: passes**; `String.from(s)` stays as the `From` method that `String(s)` runs (as `clone` is the method `copied` runs), so the two are the interface and its use, not two spellings at the use |

The one-owner type `owned T` (`mesh: owned Mesh`, the owner's ruling) is not a new word but a new use
of `owned`; it is in the `owned` row of 3.1. Today `mesh: owned Mesh` is E0100 (section 5.3).

## 4. What this vocabulary hands to the next agents (rewritten by 06)

- **Placeholders, after the check** (07 fixes final spellings; each passed the owner's tests as a
  spelling): `StrView`, `CStrView` (06, C-5), `RuntimeChecked`, `ReadGuard`, `WriteGuard`, `read`,
  `write`, `Foreign`, `get` for `upgrade` (with C-6's condition), `check_disjoint`, `@thread_shared`,
  `ThreadShared` (06, C-4), `@no_alias_checks`, `@fp(fuse)`, the `from p` clause, and `with owned x`
  (refused on a `Copy` capture, C-2). Dropped by 06: `@nosynchronise` and `Synchronise` (the effect
  keeps `Sync` and `@nosync`, C-4).
- **To 07 (word planner):** two of the draft's requests need no new word after the check: a class
  object changed through a parameter or a `for` name is `ref mut` on the name (AN-20, C-1), and a
  shared interface value is `shared dyn I` (AN-66, C-9). Still open: the spelling of a value shared
  between threads, from the `ThreadShared` family (C-4); whatever replaces `Weak` as a field word
  (AN-63). Words last run invented and not adopted here (07's to judge): `backlink(...)`, `weak` as a
  field word, `global`, `@cached`, `isinstance`, `copy(x)`.
- **To area C:** `Copy` no longer covers class handles, `shared` values or `Weak` (C-3); AN-44 is
  answered under that meaning and the `class` row's condition. `owned h: Node` hands the handle over
  (C-2). Whether `owned x: owned T` (a hand-over of a one-owner heap value) is written that way, or
  `owned x: T` takes it, is area C's.
- **To area B:** `ref mut` on a parameter's name is the in-out parameter (C-1); plain names change
  nothing, so AN-20 is answered with `ref mut`, not a new word; the `Cell` condition (C-10).
- **Meaning changes, not spelling changes, as checked by 06:** `let` (one rule: nothing is changed
  through the field after `init`, C-8), `self` (read-only for every type: passes, row 3), `mut self`
  (one meaning, passes), `mut` (only in `mut self`, `ref mut`, `static mut`, `*mut T`; C-1), `owned`
  (hand-over; refused on a known `Copy` type; C-2), `Copy` (independent copies only; C-3), `mem.drop`
  (ends the name for every type, as its own rule), `MutSpan` (writable without `mut`; re-pointed with
  `ref mut`), `SoA.get` (a reference, not a copy: passes). The spec text at every use the readers
  listed is rewritten by 15; a use that would keep the old meaning is listed in section 6.3.

## 5. Evidence for the words judged here (real output)

Every file is in `target/redesign3/05-builder/`; every output below is from
`target/redesign3/bin/ember.exe check <file>`, saved in full in
`target/redesign3/05-builder/check_output.txt`.

### 5.1 `ref mut`

Uses in the spec, lines 1-6930 (52 lines): 239 706 813 855 1112 1113 1117 1150 1152 1256 1257 1339
1431 1906 1988 2280 2287 2303 2311 2314 2319 2320 2379 2543 2698 2785 2901 2938 3029 3276 3439 3448
3457 3512 3800 3821 3825 3903 4033 4286 4310 4313 4315 4451 4830 4832 5483 6046 6049 6051 6188 6504.
Positions: a type (`ref mut T`, 607, 855, 1112), an expression (`ref mut e`, 706, 2280), a pattern
(`ref mut IDENT`, grammar 828), a result (`-> ref mut T`, 1906), a local whose assignment writes
through (`[TYP-14]` 1116, `[BRW-1]` 2286-2287).

`b1_ref_mut_forms.em` uses all four positions; the only refusal is a call result used as a place:

```text
error[E2140]: assignment target is not a place
  --> b1_ref_mut_forms.em:20:5
   |
20 |     first(xs) += 1
   |     ^^^^^^^^^ this is a value, not somewhere to store one
   |

error: could not compile due to 1 error(s)
exit=1
```

`first` returns `ref mut int`, so `first(xs)` is a place; this is phase 1's defect 3 in another form
(AN-23). With the result bound first (`f = first(xs)`, `f += 1`; `b1b_ref_mut_forms_bound.em`) the
file is accepted: `exit=0`, no output. So the pattern `State.Walking(ref mut path):`, the expression
`r = ref mut xs[1]`, the write-through `r = 20` and the result type all work today.

`b2_ref_mut_rebind.em`, `a = 1; b = 2; r = ref mut a; r = ref mut b; println(a, b)`: accepted,
`exit=0`, no output. By `[BRW-1]` the last assignment writes through `r`, so it copies `b`'s value into
`a` and `r` still refers to `a`. The line reads as "`r` now refers to `b`" (what it means in Rust with
`r = &mut b`, and Python's `r = b` rebinding) and does something else: a form accepted that does not
do what its words say elsewhere, which the error-message rule says is refused (AN-28). The parameter
form `x: ref mut int` is reader-1's t3/t3c (01-reader-1.md lines 249-300).

Verdict: **keep** `ref mut`. It means one thing at every position the owner's test looks at (a name for
someone else's place that may change it), it sits on the name it describes, and the owner approved it
on row 8. The two forms that do not do what the words say are refused: `r = ref mut b` on a reference
local, and `ref mut` as a parameter's type.

### 5.2 `some I`

Uses: grammar `"some" bound_list` (608); `[GRM-31]` (621-624); `[TYP-32]` (1443-1451); 2175 (a `gen fn`
frame is opaque "as `some Iterator[Item = Y]` would"); 3373 (effects table); 6201 (Annex A); E2260
and E2261 (5723-5724).

`b3_some_return.em` (the spec's own example at 1450, `fn evens(xs: Span[int]) -> some
Iterator[Item = int]`): `exit=0`, no output.

`b4_some_param.em` (`fn total(it: some Iterator[Item = int]) -> int`):

```text
error[E2260]: a `some` type is legal only in a function's return type
 --> b4_some_param.em:2:14
  |
2 | fn total(it: some Iterator[Item = int]) -> int:
  |              ^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = help: name the type, or take a type parameter bounded by the interface: `fn f[T: I](x: T)`
  = note: `some I` is the one type a function's body returns, which only its return type can name [GRM-31]

error: could not compile due to 1 error(s)
exit=1
```

`b5_some_two_types.em` (two `return`s of different iterator types):

```text
error[E2261]: this returns `Copied[std.collections.SpanIter[i64], i64]`, but an earlier `return` gave `Filter[Copied[std.collections.SpanIter[i64], i64], fn(i64) -> bool]`
 --> b5_some_two_types.em:5:12
  |
4 |         return xs.iter().copied().filter(fn(x) => x % 2 == 1)
  |                ---------------------------------------------- this is `Filter[Copied[std.collections.SpanIter[i64], i64], fn(i64) -> bool]`, the type the `some` stands for
5 |     return xs.iter().copied()
  |            ^^^^^^^^^^^^^^^^^^ this is `Copied[std.collections.SpanIter[i64], i64]`
  |
  = help: a `some I` function returns one concrete type; to return different ones, return a `Box[dyn I]`
  = note: `some I` is the one type the body chooses, the same at every `return` [TYP-32]

error: could not compile due to 1 error(s)
exit=1
```

Both refusals give a line to write (E2261's names `Box[dyn I]`, which becomes `owned dyn I`; its
`i64` should be the program's `int`, AN-16). Verdict: **keep**: one meaning, plain English ("returns
some iterator"), on the type it describes; it says nothing about owning or changing, so the owner's
test has nothing to find missing.

### 5.3 The new words, today

Each agreed spelling is refused today, as expected (first error of each):

| file | line | today |
|---|---|---|
| `c1_copied.em` | `snapshot = copied items` | `error[E1010]: cannot find 'copied' in this scope` |
| `c2_changes.em` | `fn note(self, msg: String) changes log:` | `error[E0110]: 'note' is declared without a body` and `E0100 ... found 'changes'` |
| `c3_reads.em` | `fn load_assets(self) reads renderer -> Array[String]:` | `error[E0110]: 'load_assets' is declared without a body` and `E0100 ... found 'reads'` |
| `c4_with_copied.em` | `run(fn() with copied score: score += 1)` | `error[E0100]: expected '=>' or ':' after a lambda's parameters` |
| `c5_shared.em` | `other = shared items` | `error[E1010]: cannot find 'shared' in this scope` |
| `c6_owned_field.em` | `mesh: owned Mesh` | `error[E0100]: expected a name ... found 'owned'` |

### 5.4 `as?` and `cast`

`d1_cast_forms.em`, `a = s as? Circle` and `b = cast s if Circle`:

```text
error[E0100]: `as?` is not Ember: a downcast is written `cast x if D`
  --> d1_cast_forms.em:10:13
   |
10 |     a = s as? Circle
   |             ^ write `cast` before the value and `if` here
   |
   = help: `cast x as D` gives the `D` and panics if `x` is not one; `cast x if D` gives `Some(d)` or `None`

error: could not compile due to 1 error(s)
exit=1
```

The spec text (810, 945) still writes `as?`/`as!`; the compiler (ODR-280) writes `cast`. The readers
judged `as?`/`as!` from the spec; the vocabulary follows the compiler, whose message already gives the
line to write.


## 6. The checker's changes (06)

### 6.1 The changes

Every row went through four questions: (1) for a name the word binds, does the line say what the name
is (the owner's own thing, a copy, or handed over) and whether it may change it; (2) does the word mean
the same for every type; (3) does a word mean two things, or do two words mean one thing; (4) does it
read right where it sits, and name what it describes. A row with no **06:** mark passed all four as the
builder wrote it. The changes:

| id | rows | what changed | the test it failed, and why a programmer would care |
|---|---|---|---|
| C-1 | `mut`, `ref mut`, `ref`, `for`, `fn`, `MutSpan`, `iter_mut`, `mut self`; settlements 13, 14, 20 | The in-out parameter is written **`ref mut b: B`** (row 8's words, on the name), not `mut b: B`; the same for a lambda parameter (`fn(ref mut n: int)`), a callable type's parameter (`fn(ref mut int)`, grammar 617) and a loop name (`for ref mut v in xs:`). `mut b: B`, `fn(mut n)` and `x: ref mut T` on a parameter are refused with "to change the caller's `b` through `b`, write `ref mut b: B`". `mut` stays only in `mut self`, `ref mut`, `static mut`, `*mut T`. In a `for` header, `for v in xs.iter_mut():` gets "write `for ref mut v in xs:`" | (1) and (2). Row 8's own test: `mut path` "says may change but not the NPC's own list"; `fn bump(mut b: Counter)` says "may change" and nothing says "the caller's own `b`" (Rust reads it as a changeable copy, Python as a local). And today `mut` means two things by type: a struct parameter's fields change, a class parameter's handle is re-pointed with "no access on the object" (`[FN-9]` 1570), while `mut self` on a class changes the object (`k4`, accepted). `self` itself says what it is, so `mut self` passes and stays. One rule results: every name that changes someone else's value is written `ref mut name`, which also answers AN-20 (a class object changed through a parameter or a loop name) with no new word. Cost: 22 non-`self` `mut` parameters in the spec and 114 in `std/src` change spelling (grep counts, section 6.2) |
| C-2 | `owned`, `mem.drop`; settlement 17 | `owned` is **refused where the type is known to be `Copy`** (`owned n: int`, `with owned score` of an `int`, `match owned e` of a number), with "an `int` is copied, not handed over; remove `owned`" (a capture: "write `with copied score`"). `x: owned T` as a parameter's type with no mode on the name is refused ("to take it over write `owned x: T`; to read it write `x: T`"). `mem.drop` ends the name for every type as a rule of `mem.drop`, stated on it | (2) and (4). The owner's named failure is `owned` silently copying an `int`; the draft kept exactly that for parameters ("a `Copy` value is copied by its type's rule"), so `owned n: int` is accepted and the caller keeps `a` (`k1`). Refusing it where it can only copy removes the failure without breaking generic code (`fn push[T](mut self, owned v: T)` with `T = int`, where the callee's own copy and a hand-over cannot be told apart by anything the program computes). `fn f(m: owned Mesh)` and `fn f(owned m: Mesh)` differ by one word's place and mean "read it" and "take it": the first is refused. The draft's `mem.drop` rule contradicted its own signature `mem.drop(owned x: T)` (2261) once `owned` copies a `Copy` value; today `mem.drop(a)` then `println(a)` is accepted (`k3`) |
| C-3 | `Copy`, `class`, `owned` | `Copy` means **an independent copy**: numbers, `bool`, `ref T`, `Span`, structs of such. Class handles, `shared` values and `Weak` are not `Copy`. The `class` row gains the condition that `p2 = player` with no word may not silently share (it hands over, as row 9's `snapshot = items`, or is refused naming `copied` and `shared`); area C rules on AN-44 under it | (2) and (3). Today `Copy` covers class handles "whose copy is a share" (`[OWN-7]`, ODR-051): one word, two meanings, and the reason `owned h: Node` is a retained copy the caller keeps (`k2`, accepted; `[FN-9]` 1570), a share spelled as a hand-over. The brief allows a name with no word only where it "never aliases"; a silent share aliases |
| C-4 | `Sync`, `Sync(effect)`, `@sync`, `@nosync`, `SyncShared`; settlement 1 | Overturned: the **property** `Sync` -> **`ThreadShared`** and `@sync` -> **`@thread_shared`** (one family: the attribute makes a class's values `ThreadShared`); the **effect** keeps **`Sync`** and **`@nosync`** keeps its spelling. `@nosynchronise` and `Synchronise` are dropped | (3) and (4). The draft renamed `@sync` because "sync" reads as "synchronised", then kept the property spelled with the same word and meaning the same thing (a `@sync` class's handles are `Sync`, `[THR-9]` 3444-3448): it fails the same test, and beside `@thread_shared` it would be a second word for one meaning. The effect, by contrast, is what "sync" plainly says (the function synchronises: locks, atomics, channels, spawn and join, 3184-3186), and `@nosync` reads "no synchronisation", which is what it forbids. The draft's `Synchronise` would have left `Sync` and its own long form meaning two different things. This is reader 03's position; it renames the property (rarely written) and keeps the 7 `@nosync` uses |
| C-5 | `cstr` | Renamed, placeholder **`CStrView`** | (4), and the draft's own reason for `str` -> `StrView`: a borrowed view and the owned text (`CString`) told apart only by length and a capital. Renaming `str` alone would make the two text families read differently |
| C-6 | `upgrade` (placeholder `get`) | Kept as a rename, with a condition for area D: `get` elsewhere (`Array.get`, `SoA.get`) gives a reference that keeps nothing alive; if `w.get()` hands back a new owner, the spelling must say so | (1): in `p = w.get()`, nothing would say `p` keeps the object alive |
| C-7 | the agreed rows | Checked word by word against `inputs/JOB.md` rows 3-9: section 1 matches; no change made by 06 touches an agreed spelling or meaning. C-1 applies row 8's test to other names; it leaves row 8 exactly as agreed | none |
| C-8 | `let` | "It will mean" reworded as one rule: after `init`, nothing is changed through the field, for every type (a class-object field: neither re-pointed nor its object changed through it) | (2): the draft said "contents included" for a list field but only "never re-pointed" for a handle field, so a class-object field's object could still be changed: two meanings by type |
| C-9 | `dyn`, `shared` | The shared interface value is `shared dyn I`, already in the `shared` row; AN-66 needs no new word | (3): the draft asked 07 for a word the vocabulary already has |
| C-10 | `Cell` | Kept, with a condition for area B: a `Cell` field is set through a plain name today (`k10`, accepted), so either the plain-name default is stated with the `Cell` exception or setting a `Cell` field needs `ref mut` / `changes` | (1): the brief's default for a name with no word is "never changes anything" |

Rows checked and passed as written, among those the builder asked 06 to look at first: the
placeholders `StrView`, `RuntimeChecked`, `ReadGuard`, `WriteGuard`, `read`, `write`, `Foreign`,
`check_disjoint` (and `assert_disjoint` for the panicking form: an old call that used the `Result`
stops compiling, it does not change meaning silently), `@no_alias_checks` (it reads as the rest of
the `@no...` contract family: `@noalloc` forbids allocation, `@no_alias_checks` forbids run-time alias
checks, E4030), `@fp(fuse)`, the `from p` clause, `with owned x` (after C-2); the meaning changes of
`self` (row 3), `mut self`, `SoA.get`; and the merges and removals (`Box`, `Box[dyn I]`, `Shared`,
`.clone()`, `.copy()`, `Mut`, `as?`, `as!`, `owned fn`, `@no_runtime_checks`). `copied`, `shared`,
`changes`, `reads`, `String(...)` keep the agreed spellings and meanings.

### 6.2 Evidence (real output)

Every file is in `target/redesign3/06-checker/`; every output is from
`target/redesign3/bin/ember.exe check <file>`, saved in full in
`target/redesign3/06-checker/check_output.txt`.

`k1_owned_int.em` (`fn take(owned n: int)`, then `println(a + x)` after `take(a)`): `exit=0`, no
output. The caller's `a` lives on: `owned` copied it.

`k2_owned_handle.em` (`fn take(owned n: Node)`, then `println(a.v + x)` after `take(a)`): `exit=0`,
no output. The caller's handle lives on: `owned` shared it.

`k3_mem_drop_int.em` (`mem.drop(a)` of an `int`, then `println(a)`): `exit=0`, no output.

`k4_mut_param_two_meanings.em` (`fn set_struct(mut p: P): p.x = 5`; `fn repoint(mut h: Node): h =
Node(0)`; `fn set(mut self, n: int): self.v = n` on a class): `exit=0`, no output.

`k5_ref_mut_param_on_name.em` (`fn bump(ref mut x: int):`), first error and the count (the 8 others
follow from the same parse failure; all 9 are in `check_output.txt`):

```text
error[E0100]: expected a name
 --> k5_ref_mut_param_on_name.em:2:9
  |
2 | fn bump(ref mut x: int):
  |         ^^^ found `ref`
  |
...
error: could not compile due to 9 error(s)
exit=1
```

`k6_for_ref_mut.em` (`for ref mut v in xs: v += 1`): the form parses, and the name is read-only:

```text
error[E3021]: cannot write through a shared reference
 --> k6_for_ref_mut.em:5:9
  |
5 |         v += 1
  |         ^ this is `ref i64`, which only reads
  |
  = help: a reference is not pointed elsewhere: this writes through it, into what it refers to; to keep another value, keep a copy of it (`.clone()`), or its index
  = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]

error: could not compile due to 1 error(s)
exit=1
```

`k7_lambda_mut_param.em` (`apply(fn(mut n: int): n += 1, a)` with `f: fn(mut int)`): `exit=0`, no
output.

`k8_copied_span.em` (`ys = xs.as_span().to_array()` in two lines):

```text
error[E3063]: `Span[i64]` is a view, so it may not be passed to `to_array`, which stores it where only a `static` view may go
 --> k8_copied_span.em:5:10
  |
5 |     ys = s.to_array()
  |          ^^^^^^^^^^^^ stored by this call
  |
  = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
  = note: the view borrows `xs`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error[E3063]: `ref i64` is a view, so it may not be stored in an `Array` unless it is `static`
   --> C:\Users\ism19\Code\ember\std\src\core.em:988:22
    |
988 |             out.push(x)
    |                      ^ stored here
    |
    = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
    = note: the view borrows `__xs`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error: could not compile due to 2 error(s)
exit=1
```

`k9_iter_mut.em` (`for v in xs.iter_mut(): v += 1`): `exit=0`, no output.

`k10_cell_through_plain.em` (`fn tick(s: S): s.frame.set(s.frame.get() + 1)`): `exit=0`, no output.

Counts behind C-1's cost (Git Bash, from the repo root):
`grep -nE "[(,] *mut [a-z_][a-z_0-9]*\s*:" docs/spec-source/Ember_v0.9.10_Hardened_35.md | grep -v "mut self" | wc -l`
printed `22`; `grep -rhoE "[(,] *mut [a-z_][a-z_0-9]*\s*:" std/src | wc -l` printed `114`;
`grep -c "mut self"` on the spec printed `65` (unchanged by C-1).

### 6.3 Spec text that keeps an old meaning (for 15)

| spec lines | text | old meaning it keeps | follows |
|---|---|---|---|
| 553-554, 617, 716 (grammar) | `param := ["mut" \| "owned"] IDENT ":" type`; `fn_type_param := ["mut" \| "owned"] type`; `lambda_param := ["mut" \| "owned"] IDENT` | `mut` on a parameter's name | C-1: `"ref" "mut"` in those places |
| 1553-1563 (`[FN-1]`, `[FN-2]`) | `mut b: B` in-out; `owned c: C` "copied if `Copy`; a handle is copied without a retain"; "the caller passes `x.clone()`" | in-out as `mut`; `owned` copying; `.clone()` at the use | C-1, C-2, C-3; `copied x` |
| 1570-1578 (`[FN-9]`) | `owned` on a handle "a copy, retained ... while the caller's own lives on"; `mut` "lets the callee re-point the caller's handle, and opens no access on the object" | `owned` as a share; `mut` as re-point only | C-1, C-3 |
| 168, 2240, 2270 (`[OWN-7]`, ODR-051) | a `Copy` copy "retains each counted handle in it" | `Copy` covering handles | C-3 |
| 2260-2261 (`[OWN-6]`) | `mem.take(mut place: T)`, `mem.replace(mut place: T, ...)`, `mem.swap(mut a: T, mut b: T)`, `mem.drop(owned x: T)` | `mut` parameters; `mem.drop` ending the name only through `owned` | C-1, C-2 |
| 3444-3448 (`[THR-9]`), 3451-3475 (`[THR-7]`) | the property `Sync`; `@sync` | the property's old spelling | C-4 |
| 1749 (`[CLS-9a]`) | a `let` field's contents may change | `let` as "assignable only in `init`" | C-8 |
