# What changed from v4, and why (run 4, step 4, v5; agent 01, rule writer)

Every change from the v4 rules and vocabulary (`V4/rules.md`, `V4/vocabulary.md`, copied into `V5` at the
start of this step) to `V5/rules.md` and `V5/vocabulary.md`, each with the owner's verdict that caused it;
every error-code change; the stop texts; the worklist's section 4 items marked **rules**, and where each
stands; the technical points this step settled itself, with reasons; and what is left for the owner. The
rules and the vocabulary carry no history: it is all here. What changed before (verdicts 1 to 82) is in
`V2/changes.md`, `V3/changes.md` and `V4/changes.md`.

`V4` means `tasks/96/redesign/run4/v4/`, `V5` means `tasks/96/redesign/run4/v5/`. "Verdict N" is item N
of `inputs/JOB.md`, section "Owner verdicts on run 4's open questions"; "R1" .. "R18" are the v5 rules;
"row N" is the owner's agreed row N; "worklist" is `V5/inputs/worklist.md`, and "the report" is
`V5/tools/step4_spell_report.md`. Terms are defined in `V5/rules.md` section 0.

No compiler change, no git write, no Ember program compiled or run. Commands run: the power check
(`PowerLineStatus` printed `Online`), a lookup of every section-4 code in
`compiler/ember_diag/src/codes.rs`, a search of the compiler and the repository for the new code E2037,
and two scripts of this step (`target/redesign7/01/vocab_edit.py`, which applies the vocabulary's row
changes and stops unless each edit matches exactly once; a one-off replacement of one message in both
files).

## 1. Verdicts 83 to 97, and the corrected records of 53 and 58: where each is applied

| verdict | what the owner decided | applied in |
|---|---|---|
| 53 (record corrected) | the call check reads every function the program can put in a function value or behind a `dyn` method; only a C function's code is unseen | R4 (the call check: "a call through a function value, or through an interface, `dyn` or virtual method, by every function the program can put there"; "Only a C function's code is unseen"), R11 (the walk's `this.on_visit(p)` compiles when no stored function uses its node after ending it; one that does is refused where it is stored), section 0 ("unknown call" became **unseen code**: a C function only), section 3 (E3031's storing-line message), section 4 (E3031) |
| 58 (record corrected) | an object C keeps is handed to a parameter declared `owned`, or given with `into_raw()`; `Retained` is only for callbacks | R1, R16 (new paragraph "Giving an object to C"), section 3 (the `mem.forget` message); vocabulary `Retained`, `mem.forget`, `ForeignBox`, new row `into_raw()` |
| 83 | `with` dropped from what a function keeps | R3 (the capture list: `fn() copy of a, owns b, ...:`; E0116's and E0120's lines), R4, R12, R15 (every capture), section 2 (`owns` row), section 3 (every message that wrote `with` before a capture; E0100's new message for `with` before a capture; L3001's), section 4 (E0100, E0120); vocabulary `with` (the capture use goes; the word keeps blocks and imports), `fn`, `owned fn`, `copy of`, `shares`, `thread_shared`, `Weak`, `SyncShared`, `thread.spawn`, `Scope`, section 1 (row 6), section 4, section 6 |
| 84 | `owns` hands over every type the same way, numbers included | section 0 (handed over), R1, R3 (`owns` before a value; the capture list), R4 (`owns x: T` for every type; `f(n)` ends `n`; `f(copy of n)` keeps it), section 3 (E3040's number message; E2085's row removed), section 4 (E2085 removed, E3040 widened); vocabulary `owns`, `mem.drop`, `fn`, section 6.2 (`[FN-1]`, `[OWN-6]`) |
| 85 | `owns` on the making line also inside an `Option` or a list | R3 (the `owns` bullet: `left=Some(owns Tree(value=2))`, `r.pools.push(owns IntPool(items=[1, 2]))`, `shapes=[owns Circle(r=1.0), owns Square(side=2.0)]`), section 3 (E2123's messages), section 4 (E2123); vocabulary `owns` |
| 86 | `changes` and `reads` are removed; the compiler works out what every function changes and reads from its code | section 2 below; every rule, row, message and example |
| 87 | the call check covers a `ref mut` parameter | R4 (`rename(ref mut LOG[0])`), section 3 (E3031's `rename` message); vocabulary `ref mut` |
| 88 | one whole variable under two names: the call is compiled against a copy of the function with one name | R4 (overlap rule, exception 1: `add_both(ref mut x, ref mut x)` prints 2; `rename(ref mut LOG)`), R4's noalias point, R8 (f), section 2 (the "nothing" row: one more copy of the function's code), section 3 (E3022's row title), section 4 (E3022); vocabulary `ref mut` |
| 89 | a thing whose changes go only through its lock may be reached through any number of names | R4 (overlap rule, exception 2: `dump(log, fn() thread_shared log: log.push("seen"))`), R15; vocabulary `thread_shared`, `guarded by` |
| 90 | (reversed by 95) | nothing kept |
| 91 | `into_raw()` only inside `unsafe`, built into the compiler; a C function declared `owned` takes an `owns T` by an ordinary hand-over | R1, R3 (`owns T` replaces `ForeignBox[T]`), R16, section 3 (E3100's `into_raw()` message; the removed words: `ForeignBox[T]`; `mem.forget`'s message), section 4 (E1010, E3100); vocabulary `ForeignBox`, `mem.forget`, `Retained`, `unsafe`, new row `into_raw()`, section 4, section 6.1 |
| 92 | a decimal reduction is allowed and may differ in the last digit | R15 (reductions: `[[0.1], [0.2, 0.3]]` gives `0.6`), section 2 (the reductions row); vocabulary `Scope` |
| 93 | a scoped task takes the loop's name `copy of t` or `ref t` | R3 (`ref` as a capture word), R12 (a name captured with `ref`), R15 (the loop's own variable), section 3 (E3026's two new messages, E7004's loop message), section 4 (E3026, E7004); vocabulary `fn`, `Scope`, `thread.spawn`, section 0 (view) |
| 94 | `thread.spawn` takes a `thread_shared fn()` | R12, R15 (`[R]` above `pub fn spawn(owns f: thread_shared fn() -> R) -> JoinHandle[R]:`; `scope.spawn` keeps `fn()`), section 3 (E7015's row), section 4 (E7015); vocabulary `thread.spawn`, `Scope`, `thread_shared`, section 6.2 (`[THR-10]`) |
| 95 | Rust's way for lists and maps: nothing changes a walked collection through its own name; the loop's handle changes the item it is on | R8 (c) (rewritten: "whatever the change"; `for ref mut x in xs: x = 5`, `for k, ref mut v in counts: v *= 2`; a counting loop does not walk), section 3 (E3020's two new messages), section 4 (E3020); vocabulary `for` |
| 96 | the count-limit tests use a real 8-bit counter | section 2 (a paragraph under the stop texts: `#$ count_bits: 8`, 255, the texts with `(255)`, and the one test that checks real programs keep 32 bits) |
| 97 | a value made later for an `owns` place says `owns`; E0109 keeps only `owns` before an existing name | R3, section 3 (E2123's `e.mesh = owns Mesh(...)` message; E0109's row), section 4 (E0109, E2123); vocabulary `owns` |

## 2. Verdict 86 in the rules: no `changes`, no `reads`

### 2.1 What went

- R5's clauses: `reads a, b`, `changes a, b`, `changes this`, `changes NAME`, `changes T`, `changes
  captured`, `changes captured...`, change lists (`[C, F: fn(Item) changes C]`, `CHANGED`), the named
  change list (`world: changelist = (Game, LOG)`), and every rule that checked a written clause against a
  body (E2105, E2107, E2113, E2122; section 5). v4's R5 bullets on what a first line says about `this`, on
  "everything else it causes to change", on a `drop`'s line, on the order, on checking both ways, on
  "needs no clause" and its link exception, on a callee's type-named change, and on a virtual's line as a
  promise for its overrides, are gone with them.
- R12's bullets "A function's type says everything it changes but its own state", "A field, a list or a
  parameter that holds functions says it in its type" (`captured`), "A function value goes only where its
  type allows" (E2113), "A change list", "A named change list", "A type that keeps a function carries its
  change list" (and the `dyn` type that says what it changes, verdict 46), the sentence "the function
  that makes the call names them on its first line", and the `captured` paragraph of "A call through a
  function value".
- Function types say nothing about changes: `fn()`, `fn(Node)`, `thread_shared fn()`, `owns dyn I`.
- Section 0's **change list**; **first line** lost its clauses; the vocabulary's **change list** term.
- R17's "a `pub` method with no `reads` is taken to read every field, and what a function or a function
  value changes is on its first line or in its type".
- Every message that asked for a clause (E2105's ten, E2113's five, E2107's, E2122's, the E0118 order
  that listed `reads` and `changes`, E1010's `self` message and E0113's `mut this` message that gave
  `changes this`, E0100's `global` message that gave `changes FRAME`, E3026's `Button` lines, the
  `@cached` message that gave `reads verts`, the E2118 and E3023 lines that wrote `changes pos` and
  `changes Game`).

### 2.2 What replaces it

R5 is rewritten as "A function's first line, and what the compiler works out from its body". Its first
line says the parameters with their words, `owns this`, the result type and `from`, in that order (E0118
keeps that order). Three bullets carry the worklist's paragraph:
- **The compiler works out what every function may change, read and end from its code**: its own lines,
  the functions it calls, every function the program can put in a function value or behind an interface,
  `dyn` or virtual method it calls (the whole program, libraries included), what the threads it starts and
  the callbacks it hands to C change, and the `drop` of every object it may end ("ending an object counts
  as a call of its `drop`", verdict 57, now also for a function value or `owns dyn I` value replaced or
  ended with what its state owns).
- **That fact decides every rule that needs it**: R8 (c) loops, R8 (g) read-only names, R4's call check
  (E3031), R10's ends, R12's stored functions, R15's locks and threads, `init` calling a method before
  the fields its body reads are set (E2100, row 7), `@cached` (E2117). Nothing is checked while the
  program runs. Across a package's edge the fact is kept in the interface file (R17); the message names
  the chain of calls; a message never asks for a word on a first line. C code cannot be read:
  `@ffi(no_callbacks)` stays.
- **What a method does to its object is worked out the same way** (rows 3 and 7, the turret of verdict
  18, `this` never noalias, `init`, E2108 for `owns this` on `init`).

The "plain name never changes anything" paragraph stays, without the `changes this` sentence.

### 2.3 Where every old clause's rule now gets its fact

| rule | v4 read it from | v5 reads it from |
|---|---|---|
| R4 overlap: a function value given to a call | its type's `changes` | every function the program can put there (R12) |
| R4 call check (E3031) | the body, with "a call whose body it cannot see counts as using every such parameter after every change its first line or type allows" | the body, through every callee, function value and `dyn` or virtual method read whole-program (verdict 53's corrected record) |
| R8 (b) what may reshape | callees' first lines and summaries, function values' types | the code (R5) |
| R8 (c) loops | a call's first line or type | the code (R5); the chain in the message |
| R8 (d) a change that would be lost | "it changes nothing else, through its first line" | its body |
| R10 point 1, R10's closing | first line, summary, arguments, function type (`changes Player`) | the code (R5); a write to a place of function type or `owns dyn I` whose state owns one (hole 3) |
| R11 the walk | `on_visit: fn(Node) changes Node` | every function the program stores in `on_visit` |
| R12 a call through a stored function value is a change of its holder | always | when the functions it may run change their own state |
| R12 kept callbacks and `tick` (verdict 39) | `fn tick() changes Game, LOG:` | `tick` changes what the kept callback changes, worked out |
| R15 locks (E7018) | the function value's type says `changes Log` | it may run a function that takes a `Log`'s lock |
| R15 threads (verdict 30) | `main` names what its threads change | starting a thread counts as making its changes, worked out |
| `init` (row 7) | `reads renderer` | the body reads only `renderer` |
| `@cached` | `reads verts` | the fields its body reads |

### 2.4 Rows 3, 6 and 7

Row 3: `fn note(msg: String):`; the loop rule still refuses `this.note(...)` inside a loop over
`this.log`. Row 6: `b.on_click(fn(): score += 1)`, and `fn() copy of score:` for a copy (verdict 83). Row
7: `fn load_assets() -> Array[String]:`, allowed in `init` once `renderer` is set because its body reads
only `renderer`. Each keeps its agreed meaning (verdict 86: "rows 3 and 7 lose their words").

## 3. Every change from v4's rules

| v4 | v5 | what changed | why |
|---|---|---|---|
| title, opening | title, opening | step 4, v5; verdicts 1 to 97; `V5` paths | — |
| section 0 | section 0 | handed over (every type, `fn() owns count`); view (a name captured with `ref`); type parameters' example; first line (no clauses); summary (what a function changes, reads and ends, the chain, the locks it takes); change list (removed); function value (its type says no changes); unknown call becomes **unseen code** (C only) | 53, 84, 86, 93 |
| R1 | R1 | `owns` hands over every type; an object C keeps goes to an `owned` parameter or by `into_raw()` in `unsafe:` (no more "through the `Retained` token") | 58, 84, 91 |
| R2 | R2 | a read-only `ref b` given to a `ref mut` name is refused (E3023), with the lead's message | worklist section 3 (U4) |
| R3 | R3 | a `sharedref mut` field copied as one more holder; a struct holding a `ref mut` field is a view type (E2116); new bullet "`owns` before a value hands it over, for every type": required inside an `Option` or a list and for a value made later; E0109 only before an existing name or `fn`; a function type says no changes; the instance address is a kept `sharedref mut` link; capture words with no `with`, `ref` among them, `owns` of a number; E0120's message | 79 (rule writer 4), 83, 84, 85, 86, 93, 97; T-W1 4 |
| R4 | R4 | `owns x: T` for every type, `f(n)` ends `n`; no `changes` in the `Mutex` and `guarded by` examples; overlap: `assert_disjoint`'s import, exception 1 (one whole variable, 88) and exception 2 (a thing changed only through its lock, 89); the call check rewritten (whole program, end or move, the turret, leaving a lock's block, precise facts, `ref mut` with the noalias widening, the storing line, only C unseen); noalias point (a copy with one name) | 53 (corrected), 84, 86, 87, 88, 89; worklist section 4 (wrong 3-6, area B 2, area C 4, T-W2 3, F-06) |
| R5 | R5 | rewritten (section 2) | 86 |
| R6 | R6 | the generator: E3020 for the list it walks, E3021 for another change of what it points into, unless `from` narrows it | T-W3 1 |
| R7 | R7 | `from <let field>`: a `String`, an `Array` or an `owns T`; E2118's inline-struct message; `sharedref mut` on two threads also by a reduction | T-W3 3; 77 (U10) |
| R8 | R8 | (a) a list reached through a link gives addresses; (b) "what may reshape" from the code, hole 3 part 1 (`rename()`), a view of a class object reached through a link survives a push; (c) rewritten: whatever the change, the loop's handle, a counting loop does not walk; (d) "by its body"; (f) one whole variable, the `ref mut` parameter's check at the call, a call that reads and removes is E3060, a reshape of a `ref mut` address is E3022; (g) "end or move", "the ways out that compile" | 81, 86, 88, 95; T-W3 4, 5, 8; T-W5 7; hole 3 |
| R9 | R9 | "otherwise than by a reduction" (E7007); "written as any change is written (R5)" goes | 77 (U10), 86 |
| R10 | R10 | point 1: the code, function values and `owns dyn I` whose state owns one (hole 3 part 2), the `remove_all` message's way out with `q` (F-01); the closing: "worked out from the functions it may run" | 86; F-01; hole 3 |
| R11 | R11 | `child.parent = weakref this` "in a method that otherwise changes only `this.children`"; the walk's `on_visit` by the stored functions; `this.on_visit(p)` compiles unless a stored function uses its node after ending it, refused where stored | 53 (corrected), 86 |
| R12 | R12 | rewritten: function types say no changes, the whole program is read; its own state (hole 3 parts 1 and 2: `rename`, `h.keep = fn(): pass`); `thread_shared fn()` with the class form (hole 6a, E7015 at the line that makes the object) and `thread.spawn`; template parameters for library functions (verdict 38, without the change list); a call through a function value; running a function on its own; C's kept callbacks (no `changes Game, LOG` on `tick`); real variables, `ref` captures; `owns` for a function that outlives its maker; the holder rule by what the functions may do; no `with` | 38, 53, 61, 80, 83, 86, 93, 94; hole 3; hole 6a |
| R13 | R13 | unchanged | — |
| R14 | R14 | E2037 for a kind test that can never pass; E3027 for `ref mut` through a plain cast name; E2040's one message for two operations | worklist section 3 (U5, U6); T-W6 10 |
| R15 | R15 | starting a thread counts, worked out; new bullet: `thread.spawn` takes `thread_shared fn()`, `scope.spawn` a plain `fn()`; reductions: decimal ones, two `sharedref mut` links, a plain assignment and a task that reads first need a lock, a `thread.spawn` thread that only adds may use an `Atomic`; new bullet on the loop's own variable (`copy of t`, `ref t`, E3026, E7004); capture words without `with`; a thing changed only through its lock; E7018 by what a function value may run, with a line to move; `static mut` without `changes NAME`; the trampoline's tag layout and the once-callback's stop | 52, 77, 83, 86, 89, 92, 93, 94; F-10; worklist section 3 (U8, U9) |
| R16 | R16 | "Giving an object to C": an `owned` parameter by an ordinary hand-over, `into_raw()` only in `unsafe:` (E3100), `Retained` gives no object away | 58 (corrected), 91 |
| R17 | R17 | every fact but a `pub` function's bounds is in the interface file: what each function changes, reads and ends with its chain, what a `ref mut` parameter's function reaches by other names, the locks it takes, the function values a package stores | 86 |
| R18 | R18 | the caller's variable at a call (F-13); a moved line is named with where it goes (F-12); a change a call makes names the chain; no message asks for a word on a first line; E3031 over E3021 where both fit | 86; F-12, F-13; T-W2 4 |
| section 2 | section 2 | the `@cached` row (fields its body reads); `owns` for a function's own state; reductions whole and decimal; the trampoline's once mark; the "nothing" row (worked out from code, one more copy for one whole variable); a stop text for a `callback(once)` called twice; the paragraph on `#$ count_bits: 8` | 86, 88, 92, 96; F-14 (U9) |
| section 3 | section 3 | removed: the six E2105 rows, E2107, E2122, E2085, E2113, E2105's half of the stored-function row; added: E3023 for a stored function through a read-only name (its own row), E2100 (row 7's refusal), E2037, E5013 (the lead's message); messages changed listed in section 7 below | 84, 86; worklist sections 3, 4 |
| section 4 | section 4 | note: the bold list (E2085, E2105, E2107, E2113, E2122 out; E2037 in) and E2037's check; meanings changed (section 5 below); "Removed" line | 84, 86 |

## 4. Word changes

| rows of `V5/vocabulary.md` | change | caused by |
|---|---|---|
| title, opening, section 0, section 2 | step 4, v5; verdicts 1 to 97; `V5` paths; view (a name captured with `ref`); **what a function does** replaces **change list** | 86, 93 |
| section 1 | rows 3 and 7 without `changes` and `reads`, row 6 without `changes score` and `with`; the paragraph under the table | 83, 86 |
| `dyn` | what its values' methods change is worked out from every value made into the type | 86 |
| `fn` | rewritten: function types say no changes; capture words after the brackets; `ref t`; `owns` of a number; refusals: E0100 for `with` and `changes`, E3026's loop message, E7015's class message; E2105, E2113 and E2085 gone | 83, 84, 86, 93, 94; hole 6a |
| `for` | rewritten: nothing changes the walked collection, the loop's handle (`for ref mut x`, `for k, ref mut v`), two new E3020 messages, a counting loop does not walk | 95 |
| `is` | E2037 | worklist section 3 (U5) |
| `mut` | E0113's messages: the caller's variable, `fn reset():` | 86; F-13 |
| `mut self` | `changes this` no longer takes its place | 86 |
| `override`, `virtual` | a call through a virtual counts every override's changes; E2113 gone | 86 |
| `owned fn` | `fn() copy of x:`, `fn() owns x:` | 83 |
| `ref mut` | a method's object; E3031 for a `ref mut` argument with the `rename` message; one whole variable | 86, 87, 88 |
| `self`, `this` | messages without `changes this`; `this` rewritten (worked out from the body; E2108 for `owns this` only; E2107 and E2122 gone) | 86 |
| `static` | `changes NAME` and its E2105 gone | 86 |
| `[...]`, `type` | `[F: fn(Item)]`; `type Handler = fn()`, `type DeathHandler = fn(ref mut Array[Enemy], int)` | 38, 48, 49, 86 |
| `unsafe` | `into_raw()` | 91 |
| `with` | the capture use goes; E0100 for `with` before captures | 83 |
| `from` | `String`, `Array` or `owns T`; E2118's inline-struct message; `next_word` without `changes pos` | 86; T-W3 3 |
| `@ffi` | a kept callback's changes count for every caller, not "named" | 86 |
| `Weak`, `SyncShared` | capture spellings | 83 |
| `RefCell`, `Cell` | the change is made through a name that may change; the `Scene` lines | 86 |
| `mem.drop` | ends the name for every type as every `owns` parameter does | 84 |
| `Mutex`, `RwLock`, `Atomic` | what a function changes in one is worked out; E7018 | 75 (void), 86 |
| `Retained` | keeps an object on Ember's side; gives none away | 58 (corrected), 91 |
| `init`, `read` | `init` by what the method's body reads, E2100's and E2108's messages; `read` without `reads` | 86 |
| `drop` | rewritten: no line about its changes; ends of function values | 69 (void spelling), 86; hole 3 |
| `ForeignBox`, `mem.forget`; new row `into_raw()` | `into_raw()` in `unsafe:`; the `mem.forget` message | 58 (corrected), 91 |
| `Scope`, `thread.spawn` | spawn's lines; `copy of t`, `ref t`; decimal reductions; E3026's and E7018's messages; no `changes` | 83, 86, 92, 93, 94 |
| `copy of`, `shares`, `sharedref` | capture spellings; `changes T` through a `shares mut` owner gone; "other than by a reduction" | 83, 86; U10 |
| `changes`, `captured`, a change list, `reads`, `changelist` | **removed** rows | 86 |
| `exists as` | E3060's way out with `q` | F-01 |
| `owns` | rewritten: every type, numbers included; `Option`, lists, a value made later; E0109 only before an existing name; E2085 gone | 84, 85, 97 |
| the paragraph after 3.6 | `captured` out of the owner's spellings; the removed words keep rows | 86 |
| `thread_shared` | rewritten: no `changes`; the class form of E7015; `thread.spawn`; verdict 89 | 61, 86, 89, 94 |
| `@cached`, `guarded by` | the fields its body reads; `add_line` without `changes Log`; verdict 89 | 86, 89 |
| the iterator row, `fetch_add`, `send` | template parameters with no change list; what a function changes through them is worked out | 38, 86 |
| `assert_disjoint` | W2190's line with its import | worklist section 4 (wrong 3-6) |
| section 4 (placeholders) | `with owned x`; the capture list's `with`; `ref mut this`, then `changes this`; `captured`; the library's first line; `ForeignBox.into_raw()` | 83, 86, 91 |
| section 5 | verdicts 83 to 97 leave no word open | — |
| section 6.1 (`Weak[T]`, `RefCell`, `ForeignBox[T]`, `owned fn`, `mut self`), 6.2 (`[FN-1]`, `[FN-4]`, `[CLS-7]`, `[CLS-2]`, `[DRP-1]`, `[OWN-6]`, `[CLO-1]`, `[BRW-10]`, `[STA-1]`, `[FFI-21]`, `[THR-10]`, `[STD-19]`) | what each spec line becomes, without clauses and without `with` | 83, 84, 86, 91, 94 |

Every other row is v4's text.

## 5. Error-code changes

| code | change | reason |
|---|---|---|
| **E2037** | **new**: a kind test that can never pass (the type tested for does not implement the interface or derive from the base class); message the lead's | worklist section 3 (U5); checked unused: `grep -rn "E2037" compiler/` finds nothing, and `grep -rlw E2037` over the repository outside `target/` and `.git/` finds only this step's two files (`target/redesign7/01/free_codes.txt`) |
| E2085 | **removed**: `owns` on a `Copy` type is allowed and hands over | 84 |
| E2105 | **removed**: every case was a written clause checked against a body | 86 |
| E2107 | **removed**: `changes this` beside other clauses | 86 |
| E2113 | **removed**: a function value, override or `dyn` value changing more than its type or the overridden method said; nothing says it now | 86 |
| E2122 | **removed**: `changes this` on a `drop` | 86 |
| E0100 | **widened**: a clause Ember does not have after a function's brackets (`changes FRAME`, `reads renderer`, `with` before captures), with the first line to write; `global`'s message without `changes FRAME` | 83, 86 |
| E0109 | **narrowed**: `owns` before a name that already exists, or before `fn`; a value made for an `owns` place is E2123's | 97 |
| E0118 | the order is parameters, `owns this`, `-> T`, `from` | 86 |
| E0120 | `fn() ref mut x:` (no `with`) | 83 |
| E1010 | **widened**: `ForeignBox`, `upgrade` named among the forms gone | 91; T-W4 4 |
| E2100 | `init` calling a method whose body reads a field not yet set (was "a `reads` method") | 86 |
| E2108 | **narrowed**: `owns this` on `init` only | 86 |
| E2116 | a struct holding a `ref mut` field named as a view type | worklist section 4 (rule writer 4) |
| E2117 | no `reads` case; "changes" and "reads a field" from the body | 86 |
| E2123 | **widened**: inside an `Option` or a list, and for a value made later | 85, 97 |
| E3020 | **widened**: whatever the change, an element written or a value under a key replaced included | 95 |
| E3022 | **narrowed**: one whole variable proved when compiling is allowed; **widened** in its text: a reshape of a `ref mut` address | 88; T-W3 4 |
| E3023 | **widened**: a read-only `ref` given to a `ref mut` name; a stored function that may change its own state called through a read-only name (was E2105 or E3023) | U4; 86 |
| E3026 | **widened**: a task or thread given a loop's name with no word, or with `ref` where it may outlive the element | 93; worklist section 3 |
| E3027 | **widened**: a place reached through a read-only name (`return ref mut pool.dense[...]`) | U6 |
| E3031 | **rewritten**: whole program read, only C unseen; "may end or move" (a number written is neither); for a `ref mut` parameter also a place the callee reads or changes by another name while it is used; at the line that stores a function into a function value | 53 (corrected), 87; T-W2 3; worklist section 1 (53's test); section 8 item 5 |
| E3040 | **widened**: a number handed over with `owns` | 84 |
| E3060 | **widened**: a view of what a function value's or an `owns dyn I` value's state owns after that value is replaced or ended | hole 3 part 2 |
| E3100 | **widened**: `into_raw()` outside `unsafe:` | 91 |
| E7004 | a loop's name included | 93 |
| E7007 | its meaning names `Atomic` beside a reduction and a lock; a plain assignment named; the message gives only the lock | recheck item 9; F-10; U10 |
| E7015 | **widened**: `thread.spawn`'s function; a plain `fn()` field of a class whose object is made `thread_shared` (at that line) | 94; hole 6a |
| E7018 | a call through a function value, or a `dyn` or virtual method, that may run a function taking the lock (was "whose type says it changes what the lock guards") | 86 |
| E5013 | gains a section-3 row with the lead's message | worklist section 3 |

**The note at the top of section 4**: the bold list is E0113 to E0118, E0120, E2037, E2106, E2108,
E2112, E2114 to E2119, E2123, E3028, E3031, E5013, E5069, E7007, E7008, E7012 to E7018 and W3013; every
section-4 code was looked up in `codes.rs` again (`target/redesign7/01/free_codes.txt`): the codes it
does not define are exactly these, and a script checked that every bold code is undefined there and every
plain one defined. The line under the table lists the removed codes (E2085, E2105, E2107, E2113, E2122
with this step; E2109, E7001, E7003, E7005 and E3045 from before).

## 6. Stop texts (rules section 2)

- New row: "C called the callback registered on line 29 a second time; it was registered
  `callback(once)`, so it may run only once" (the lead's text, worklist section 3; F-14, U9).
- Verdict 96: a paragraph under the table: the two count-limit tests are compiled with `#$ count_bits: 8`,
  stored and compared as the real count is; 255 owners or links reach the limit; their texts are "too
  many owners of a `Player` (255)" and "too many `weakref` links to a `Player` (255)"; real programs keep
  32 bits, which one more test checks; no test-only shortcut in the language. The rows for the 32-bit
  texts are unchanged.

## 7. The worklist's section 4 items marked **rules**, and where each stands

| item | done in |
|---|---|
| F-01 (E3060's "open the link again" reuses `p`) | R10 point 1, section 3 E3060 (`remove_all` and `reset()`: "write `if cam.target exists as q:` above line 13, indent line 13 under it and use `q` there"), vocabulary `exists as` |
| F-06 (53's corrected record) | R4, R11, section 0 |
| F-10 (E7007's atomic way out) | section 3 E7007 (only the lock, "which keeps the read and the add together"), R15 |
| F-12 (messages give the exact line) | R18 (a moved line is named with where it goes); section 3: the `b.click()` E3020 message ("move line 17, `b.click()`, below the loop"), the other "move ..." messages (E3020, E7018), the turret's E3021 ("write `if t is not a:` above line 20 and indent line 20 under it"), the read-lock variant of row 4 ("`return copy of g[0]`, with the result type `String`"; row 4's own words unchanged) |
| F-13 (E0113 names the parameter for the caller's variable) | R18, section 3 E0113 ("to change the caller's `a` through `n`, write `ref mut n: int`, and call it `bump(ref mut a)`"), vocabulary `mut` |
| F-14 (once-callback stop) | section 2 |
| wrong 3-6 (`assert_disjoint`'s import) | R4, section 3 E3021 and W2190, vocabulary `assert_disjoint` |
| U10 | R9, R7, R15, E7007 |
| hole 3, part 1 (a change in a function value's own state, read through a link) | R8 (b), R12, section 3 E3060's `rename()` message |
| hole 3, part 2 (replacing or ending a function value or `owns dyn I`) | R5, R10 point 1, R12, section 3 E3060's `h.keep` message, section 4 E3060 |
| hole 6a's form | R12, section 3 E7015, section 4 E7015 |
| rule writer 4 (`copy of` a struct with a `sharedref mut` field; a struct with a `ref mut` field) | R3, section 4 E2116 |
| area B 2 (leaving a lock's block) | R4 ("leaving a lock's `with` block ends nothing either") |
| area C 4 (precise facts) | R4 ("a change inside `if a is not b:` is not one for `a`") |
| T-W1 4 (the arena address a value keeps) | R3; section 8 item 7 |
| T-W2 3 (the turret) | R4, R5 |
| T-W2 4 (E3021 and E3031 on one line) | R18 |
| T-W2 5 (message fixes) | section 3 E1010 (`static fn`: "build the counter from `k`: `return Counter(n=k)`", the test's text), E0113 (with the call) |
| T-W3 1, 3, 4, 8 (technical) | section 8 items 8 to 11 |
| T-W3 5 (a list reached through a link) | R8 (a), (b) |
| T-W3 9 (whole lines) | section 3: E3022's `sharedref mut` message ("and `sharedref mut p` in place of `ref mut p` on lines 15 and 16"), E3063's AN-52 message (the list's line and the watcher's line in place of the old ones; the positions way out with its return line) |
| T-W4 4 (messages name the lines they replace) | section 3's removed words: the `upgrade` message, the test's text |
| T-W5 7 (R8 (g)'s ways out) | R8 (g) ("The refusal gives the ways out that compile in that program") |
| T-W6 10 (E2040 for two operations) | R14, section 4 E2040; section 8 item 12 |
| T-W6 11 (E7010's second way out) | section 3 E7010 (only the first, with whole lines) |

The items the worklist marks for answers or tests only, and those it calls void by verdict 86 (F-04,
F-05, F-07, F-08, U2, U3, wrong 1 and 7, T-W1 1-2, T-W2 1, T-W3 6-7, T-W4 5-6, T-W5 3, 5, 6, T-W6 9, area
B 1, area C 1-2, rule writer 3), need nothing in these two files; their clauses are gone with R5's.

## 8. Settled in step 4

Technical points no verdict settles in words, settled here, each safely and with its reason. None changes
how a program is written beyond what a verdict says.

1. **An argument for an `owns` parameter of a number ends the caller's name** (`f(n)` for `fn f(owns x:
   int)`; R4). Verdict 84 is written for a capture (`fn() owns count`); the same word on a parameter
   gets the same meaning, "`owns` hands over every type the same way, numbers included", and its worklist
   row says "after `owns count` the giver cannot use `count`". `f(copy of n)` keeps the caller's own,
   for every type. The generic case (`[T]` above `fn emplace(e: int, owns value: T)`) follows: the
   caller's name ends whatever `T` is.
2. **`owns` before an existing name is E0109 where its place already hands the value over**; for a
   number, which `=` copies, `b = owns n` hands it over and ends `n` (verdict 84's "the same way, numbers
   included"; the lead's correction after step 4's check, which had left this form without a test).
   Verdict 97 limits E0109 to `owns` before an existing name; the line without it compiles. `owns fn` keeps E0109: verdict 97's "only its own case" moves the making
   line to E2123, and `owns fn` was never a form of the language (AN-47's message stays).
3. **The capture word `ref`** (R3, R12). R3's rule that each capture word means what it means before a
   value gives `fn() ref t:` the meaning verdict 93 asks for: a read-only view of what `t` views at that
   moment, which for a loop's name is this round's element; the function value may not outlive it
   (E3026). Verdict 93 uses it for a scoped task; nothing else is added.
4. **What a function changes, for R12's "a call through a stored function value is a change of its
   holder"**: the holder is changed when the functions the program can put there may change their own
   state (worked out, verdict 86), not by the type, which says nothing now. A `for h in this.handlers:
   h()` over handlers that change no state of their own compiles; one that may is refused through a
   plain `h` (E3023, its own row now: v4 gave E2105 or E3023 for one mistake).
5. **The call check for a `ref mut` parameter covers what the callee reads or changes by another name
   while the parameter is used** (R4, R8 (f), E3031: `heal(ref mut PARTY[0])` for a `heal` that reads
   `PARTY[0]` and writes `p`). Every `ref mut` parameter is compiled with noalias (R4); verdicts 87 and 88
   accept `rename`'s body and decide at the call, and verdict 88 says other calls use the copy "where a
   `ref mut` parameter is the only way in". R8 (b)'s two-paths test does not count a parameter and a
   module variable as one place, so without this the C compiler would be promised `restrict` for two
   pointers to one player (undefined behaviour in C). Where the argument is the same whole variable,
   verdict 88's copy with one name compiles it. For a plain, `Span` or `this` parameter only an end or a
   move counts (the worklist's T-W2 3), so the turret stays accepted.
6. **E3031 at the line that stores a function into a function value**, where the fault is in that
   function (it ends what its parameter views and then uses the parameter): the worklist's row for
   verdict 53 asks for its test "refused at the line that stores it"; R4 and R11 now say so, and R18's
   "on the name that caused it" is the reason.
7. **The instance address a value made `using` an arena keeps is a kept `sharedref mut` link to the
   instance** (R3; T-W1 4, check-V1's U5). A `ref` could not let the value's end (`deallocate`) change the
   arena, and a `ref mut` would refuse reading the arena (`current.made`) while any value made in it
   lives; a `sharedref mut` link lets both happen on one thread, ends no later than the instance (E3063
   as before), and reaches another thread only as such a link does (R9). The two `run_` tests that read
   the arena's counts are right either way after check-V1's fix.
8. **A generator without `from` and a change of another field of its object** (R6; T-W3 1): the list it
   walks is R8 (c)'s (E3020); any other change of what its view points into is R8 (g)'s (E3021, at the
   change), as for every view a call gives; `from log` narrows it.
9. **`from` naming an inline struct field** (R7, E2118; T-W3 3): the message gives `let pair: owns Pair`.
   R7 asks for "a `let` field ... whose contents live in their own heap block", which an `owns T` field
   is, as a `String` or an `Array` is (area B named those two as examples); the line given compiles.
10. **A reshape through another name while a `ref mut` address (not a position) is still used** (R8 (f);
    T-W3 4): E3022 at the reshape. R8 (f) names the case and its fix (reshape after the name's last use,
    or through it) compiles; a position stays R8 (a)'s E3060 at the later use.
11. **A call that reads elements and removes, while a `ref mut` position of that list is still used** (R8
    (f); T-W3 8): E3060 at the later use, by R8 (a), the rule that names a position after a removal;
    its fix (use the name before the call, or name it again after) compiles.
12. **E2040 when two operations of a `pub` body need the missing bound** (R14; T-W6 10): one message,
    on the first line of the body that needs a bound the function does not write, giving every bound the
    body needs (R18: one mistake, one message).
13. **E7010's message** (T-W6 11): only "give each player one owner", written as whole lines; the guard
    way out would run a `shares mut` object into another thread (E7015).
14. **A list reached through a local that holds a link gives addresses, not positions** (R8 (a); T-W3
    5): R8 (a) said "reached through locals", R8 (b) "a view reached through a link"; a local holding a
    `shares`, `shares mut` or `sharedref mut` link is a link (section 0), so R8 (b) applies, and verdict
    81 decides a push: a class object survives it, an inline struct does not (E3060).
15. **A clause after a function's brackets that Ember no longer has** (`changes FRAME`, `reads ...`,
    `with` before captures): E0100, today's "unexpected token", with the first line to write. These words
    were the model's own; a program that still writes them gets a message that says what to write.
16. **E2100 has its own section-3 row** (row 7's refusal), with a message whose two ways out compile;
    v4 had it only inside an E2105 message.
17. **Reductions and E7007** (R15; F-10, recheck item 9): a task that reads the place first, and a plain
    assignment from a task, need a lock, which keeps the read and the change together (E7007's message
    gives only the lock); a `thread.spawn` thread that only adds may use an `Atomic`; E7007's meaning
    names `Atomic` beside reductions and locks, as its old message already offered.
18. **Verdict 89's reach** (R4, exception 2): "a thing whose changes go only through its lock" is written
    as a `thread_shared` object whose every field that may change is `guarded by` its lock, or is an
    `Atomic`, `Mutex` or `RwLock`.
19. **E5013's message** stands in section 3, as the lead gave it, with the reason it names only the
    promotion (R18: a message says only what the compiler knows).

## 9. What this step does not change

- Today's outputs, OLD code and "means exactly" programs are not in these two files.
- No number in section 2 was measured again; the new costs are written as not measured (one more copy of
  a function for one whole variable under two names; the trampoline's once mark).
- No answer and no test is edited: they belong to agents 02 to 08.

## 10. For agents 02 to 08 (a pointer, not a ruling)

- Every E2105, E2107, E2113 and E2122 test of the report's section 1 has no code left: the nearest test
  is the inferred check with its chain message (E3020, E3060, E3031, E7018, E7015), an `accept_`, or a
  `run_`.
- A stored function's fault in a walk (verdict 53) is refused at the line that stores it (E3031).
- A `ref mut` argument that the callee also reads or changes by another name is E3031 (section 8 item 5);
  the same whole variable compiles (verdict 88).
- `for h in this.handlers: h()` is refused through a plain `h` only when a stored handler may change its
  own state (section 8 item 4).
- E3060's "open the link again" way out names a new name (`q`).

## 11. Files

- `tasks/96/redesign/run4/v5/rules.md`, `vocabulary.md` (edited in place), `changes.md` (this file, new).
- Under `target/redesign7/01/`: `rules_v4.md`, `vocabulary_v4.md`, `changes_v4.md` (the files as this step
  found them); `rules_p1.md` .. `rules_p6.md` (the parts the new rules were joined from);
  `vocab_edit.py` (the vocabulary's edits), `vocab_try.md` (its output, copied to `V5`);
  `free_codes.txt` (E2037's check and every section-4 code looked up in `codes.rs`).

## 12. For the owner

None. Every point verdicts 83 to 97 leave open is technical and settled in section 8.

## After the workflow: the lead's changes (step 4)

- **Verdict 98** (the owner, after the verdict check): two changeable elements of one list whose positions
  are not proved different are refused when compiling (E3022), `if i != j:` the way out; R4's run-time
  index compare and its stop text are gone (R4, R8 (f), section 2, E3022, E4030 rows; AN-03, AN-14,
  AN-81; tests in R4, R8, R16).
- **Verdict 84 for `b = owns n`**: hands the number over (E3040 if `n` is used after); E0109 no longer
  refuses it (R3, E0109 rows; tests `R3/reject_a_number_used_after_it_was_handed_over_with_owns`,
  `R3/accept_a_number_handed_over_with_owns`).
- **A class object handed to another thread while a `weakref` link on this thread points at it**: E7015
  (step 4's safety re-check found it; R12, E7015 row; tests in R15).
- The verdict check's message failures F-03, F-04, F-05 (lines, not advice), F-02 (AN-47's table),
  F-06 (sanitizer claims of tests nothing ran, removed), F-07 (V5 citations); the re-check's wrong test
  (E2173's fix now gives the making line with `owns`) and its five refusals without a test.

