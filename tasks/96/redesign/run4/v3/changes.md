# What changed from v2, and why (run 4, step 2, v3; agent 01, rule writer)

Every change from `V2/rules.md` and `V2/vocabulary.md` to `V3/rules.md` and `V3/vocabulary.md`, each with
the owner's verdict that caused it; every error-code change; what today's Ember says about the new
spellings; and, at the end, what the verdicts leave open for the owner. The rules and the vocabulary
carry no history: it is all here. What changed from run 4 to v2 (verdicts 1 to 20) is in `V2/changes.md`.

`V2` means `tasks/96/redesign/run4/v2/`, `V3` means `tasks/96/redesign/run4/v3/`, `RUN3` means
`tasks/96/redesign/run3/`. "Verdict N" is item N of `inputs/JOB.md`, section "Owner verdicts on run 4's
open questions"; "R1" .. "R18" are the v3 rules; "row N" is the owner's agreed row N. Terms are defined
in `V3/rules.md` section 0.

No compiler change, no git command, no Ember program run: the only command run on Ember code was
`target/redesign4/bin/ember check` on the probes of section 7. Scratch files:
`target/redesign4/v3-01-rule-writer/` (section 9).

## 1. Verdicts 21 to 34, and where each is applied

Verdicts 1 to 20 are in v2 and stay applied. Phase 0 (the lead's scripts, `V3/spelling-changes.txt`)
had already put `exists as` (26), `if X is T as n:` (27) and `.copy_each()` (33) into the code of the
copies; this step did the rest.

| verdict | what the owner decided | applied in |
|---|---|---|
| 21 | a library function handed a function accepts one that changes things, if it says so, and the call counts as making those changes; without the word it is refused | R12 (change lists), R4 (a function value counts in the overlap rule), R5, R8 (c); vocabulary `fn`, `changes`, the change-list row (new), the iterator row (the first lines of `for_each`, `inspect`, `retain`, `sort_by`); E2105, E3020, E3022 messages |
| 22 | a function that changes a variable it captured says so (`fn() changes score:`); plain `fn()` changes nothing at all; a list or field of such functions says `captured` (one) or `captured...` (any number); a two-variable function in a `captured` list is refused | R12 (rewritten first bullets), R3 (capture list), R5, R15 (scoped tasks); vocabulary section 1 (row 6), `fn`, `changes`, `captured` (new); E2105, E2113 messages |
| 23 | after `other = shares mut items`, `items` keeps what it could do; both owners may change it | R3, R9 (stated as settled, with why it is safe); vocabulary `shares` |
| 24 | the field says the kind of arena, the line that makes the value says which one (`using current`), checked when compiling | R3 (rewritten `using` point), section 2 (a cost row); vocabulary `using`, `Allocator`, `Array`; E2020, E3063 |
| 25 | `cast s when Circle` | **not applied**: replaced by 27 |
| 26 | `holds` is replaced by `exists as` | the code was done in phase 0; R13's prose (what `exists` and `as` each say); vocabulary `exists as`, `if`, `as`, the placeholders |
| 27 | `if s is Circle as c:` for "not sure"; `if s is Circle:` tests; `cast s as Circle` stays; `cast x if D` and `cast x when D` go | R13, R14 (rewritten), section 0 (kind test), section 2; vocabulary `is`, `as`, `if`, `match`, `cast`, `as?`, `as!`, the placeholders, section 6.1; E1010, E2036, E2106 |
| 28 | a walk whose visit function may delete nodes holds a `weakref` and checks it every step; a plain `ref` walk across such a call is refused; the cost is accepted, and compile-time proof is still sought | R11 (rewritten walk), R10 (closing paragraph), R8 (g) (a view whose last use is an argument), section 2 (a cost row); vocabulary `weakref`, `while`; E3021 |
| 29 | a function that changes something under a lock says so in its type, for `Mutex`, `RwLock` and `Atomic` too | R12, R5; vocabulary `Mutex`, `RwLock`, `Atomic`, `thread_shared`, `guarded by`, `fetch_add`; E2105, E3020 |
| 30 | `changes` lists everything a function causes to change, whenever; starting a thread counts | R5, R12, R15, R8 (c); vocabulary `thread.spawn`, `Scope`, `changes`, section 6.2 (`[THR-10]`, `[THR-11]`); E2105, E3020 |
| 31 | every C call counts as possibly running the callbacks C keeps, unless its declaration says `@ffi(no_callbacks)` | R12, R15, R17, R8 (b); vocabulary `@ffi`, section 6.2 (`[FFI-11]`, `[FFI-21]`); E3020, E5069 (new) |
| 32 | what a function may do with a parameter is set by the word in front of its name, for every type | R4 (first bullet rewritten); vocabulary `owns`, `shares`; E3023 |
| 33 | `.copied()` is renamed `.copy_each()` | vocabulary `.copied()` (renamed), `.copy_each()` (new), section 6.1; the E3060 row of rules section 3; E1010 |
| 34 | a `MutSpan` name or parameter that writes says `ref mut`; R5's exception by type is withdrawn | R2, R4, R5, R8 (f), (g), section 0; vocabulary `MutSpan`, `ref mut`, section 6; E3023, E3028 |

## 2. The rules that were rewritten, and how

### 2.1 Function types say everything they change (verdicts 21, 22, 29, 30, 31)

- **What a function is, itself** (verdict 22). v2 counted the real variables a lambda captures as part
  of the function itself, so row 6's handler was a plain `fn()`. Now only the function value's own
  state (what it captured with a word, and what that owns alone) is itself; a plain `fn()` changes
  nothing at all; a lambda names each captured variable it changes (`fn() changes score:`,
  `fn() changes score, lives, LOG:`), and a field, list or parameter that keeps such functions covers
  them with `captured` (exactly one variable) or `captured...` (any number), beside module variables and
  types (`Array[fn() changes captured..., LOG]`). A function that changes fewer than a type allows fits
  it; one that changes more is refused (E2113, "changes 2 captured variables, the list allows 1"). The
  check of captured variables in the function that writes the lambda (E3026, R4, R8) is v2's,
  unchanged; it is still what keeps them safe.
- **A function that runs such functions** names them too, in the word their type uses:
  `fn click() changes handlers, captured:` for `Array[fn() changes captured]` (R5, R12; verdict 35, the
  owner's answer to section 11's question 1). In the function that wrote the lambdas the word stands for
  its own locals and needs no clause.
- **Stored inside what it changes** (area D's note 1 in `V2/area-D.md`, now expressible): a function
  value that changes a captured variable counts, for R4's overlap rule, as an argument that changes
  that variable, so `b.on_click(fn() changes b: b.handlers.clear())` is refused (E3022), and so is a
  field assignment that stores a function in what it changes (`t.on_visit = fn(n: Node) changes t:
  ...`), as Rust refuses both. This closes the hole area D found (a walk's views of `t`'s nodes living
  across a call that clears them) and lets R12 say that inside a function that runs stored functions,
  `captured` never stands for `this`.
- **Change lists** (verdict 21). A generic parameter written after `changes` stands for what the
  function given to the call changes. The library writes `fn for_each[C, F: fn(Item) changes C](owns f:
  F) owns this changes C:`, so `scores.for_each(fn(s) changes LOG: LOG.push(s.name))` is a call that
  changes `LOG` (inside a loop over `LOG`, E3020), and a plain `fn(s)` makes it change nothing. The
  first lines of `for_each`, `inspect`, `retain` and `sort_by` are in the vocabulary's iterator row.
  The function is taken as a bound (`F: fn(Item) changes C`), as today's library takes it
  (`[F: fn(Item)]`, `std/src/core.em:885`), so each function given gets its own instance and the call
  through `f` is direct and may be inlined, as Rust's `F: FnMut`; the verdict's sketch
  `fn for_each[C](f: fn(T) changes C) changes C:` is the same contract written with a function value,
  which would call through two words on every item. Listed for the owner (question 4).
- **Iterator adapters** (`inspect`, `map`, `filter`) keep the function and run it at each `next`, so
  storing it there is not a change and driving the adapter is: `Inspect[Self, C, F]`'s
  `fn next() changes this, C -> Option[Item]`. An implementation of `Iterator.next` may make the
  changes its own type parameters carry, beyond `Iterator.next`'s line, because wherever the adapter's
  type is known they are known; a generic body's changes through its type parameters' methods are
  worked out per instance, as its body is compiled per instance. Such an adapter is not made a
  `dyn Iterator` unless its list is empty (E2113). Listed for the owner (question 3).
- **Changes under a lock** (verdict 29). R12's "changes through a `Mutex`, `RwLock`, `Atomic` or channel
  end need no word" is withdrawn for function types and first lines. A change is named by the type of
  what changes: `fn() with thread_shared log changes Log: log.push("a")` (a `Log` whose `guarded by`
  field `push` changes), `changes Mutex[Stats]`, `changes Atomic[int]`. Verdict 3's call-site rule is
  unchanged: these types take no word at a call, and a plain name may still use their own operations
  (R4, R5). The gain the owner named: `with log.lock: for line in log.lines: note()`, where `note` is a
  `fn() changes Log`, is refused when compiling (E3020) instead of waiting forever for the lock while
  running. A send on a channel end is named too, by the type of its end (`changes Sender[int]`;
  verdict 36, the owner's answer to section 11's question 2).
- **Starting a thread, and handing C a callback, count** (verdicts 30, 31). R5's "a thread that
  `thread.spawn` starts is not a call" is gone. The library writes
  `pub fn spawn[R, CHANGED](owns f: fn() changes CHANGED -> R) changes CHANGED -> JoinHandle[R]:`, and
  `scope.spawn` the same, so `main` says `fn main() changes FRAME, LOG, RESULTS:` and a `spawn` inside a
  loop over `RESULTS` is refused (E3020, "start the thread after the loop"); the thread rules (locks,
  `guarded by`, E7012) still keep threads apart. A C function's `callback(retained)` or `callback(once)`
  parameter counts the same way (`setup` and `main` say `changes Game` in AN-78's program);
  `callback(borrowed)` runs the function during the call, as `for_each` does. **Where the line falls**
  (the duty's question): a call that runs the function, itself or by handing it to a thread or to C,
  counts; a call that keeps it for its caller to run does not (`b.on_click(f)`, `setup` storing a
  button's handler, `xs.iter().inspect(f)`: G-06 stays), and the function that later runs it names the
  changes (`click`, an adapter's `next`).
- **Foreign calls run the callbacks C keeps** (verdict 31). Every call of a foreign function counts,
  for R8, as making every change the kept callbacks' types list (on the threads their contracts
  allow), unless the declaration says `@ffi(no_callbacks)`, an asserted fact listed by `ember tcb`
  (`[TCB-1]`). A missing mark refuses only a loop, a view or a `ref mut` name that such a change would
  reach across the call, and that message names the mark (E3020 for `for name in game.names:
  poll_events()`). The callbacks' changes go on the first lines of the functions that hand them to C,
  not on those of every function that makes a foreign call: this step reads "a missing mark only
  refuses a loop" that way (question 5). A package records the change lists of the callbacks it hands
  to C in its interface file (R17). `@ffi(no_callbacks)` on a function whose own contract takes a
  callback is refused (E5069, new).

### 2.2 A walk across a call that may delete nodes (verdict 28)

- **The rule** (R11, R10). With `on_visit: fn(Node) changes Node`, the walk holds a link:
  `up = weakref leaf`, `while up exists as n:`, `if n.parent exists as p:`, `up = weakref p`,
  `this.on_visit(p)`, `else: break`. The walk with views (`cur = ref n`, `cur = ref p`) across such a call
  is refused (E3021, R8 (g)), with a message that gives the link walk line by line. v2's R11 sentence
  that let a view walk go on across a call that may end nodes ("the reads after that call test") is
  gone: R8 (g) and R12 refuse a view across a call that may change its type (area D's note 2). With a
  visit function that changes no node, the view walk stays and pays nothing per step.
- **Two readings the walk needs, settled here** (area D's notes 3 and 6): a view whose last use is an
  argument of the call does not live across that call (R8 (g); `this.on_visit(p)` gives `p` and does not
  use it again), and making or ending a `weakref` link changes only the target's weak count, a header
  number no name reads, so it is not a change of the target (R11: `up = weakref p` from a read-only `p`;
  `child.parent = weakref this` in a method whose line names only `children`).
- **What it costs, honestly** (nothing was measured in this step): per step, one test of the node's
  header and one move of the link's weak count (a subtract on the node left, an add on its parent); the
  `n.parent` read between the test and the call needs no header test where R11's ownership condition
  holds. The nearest measured numbers: a header test on every step, 1.579x of plain C (`model_w2`,
  uncounted links), and safe Rust's walk with a count pair per step, 1.603x (`rust_w2`)
  (`RUN3/26-timer.md` lines 20 to 22). The weak-count moves were never measured.
- **Where compile-time proof removes part of it** (area D's note 4, made a rule): when the node's
  class has no `drop` method, the walk is placed in a deferred-drop region (R10 point 2); a link that
  lives only inside the region and is only given objects that live at that moment (`up = weakref p`
  from the view `p`) needs no weak count, because the region keeps every block it sets aside until it
  ends, after the link. Each step then pays only the header test (the first link of each walk,
  `up = weakref leaf`, keeps its count). It never changes what the program does (R17). Not measured;
  the next measuring phase should time it against `model_w2` and `rust_w2`.

### 2.3 The word on the name sets the rights, for every type (verdicts 32, 34)

- R4 now opens with the rule: the word in front of a parameter's name says what the function may do;
  the type after the colon says what the caller's value is. So `fn setup(game: shares mut Game)` only
  reads the game, and `with shares mut game` made from it is refused (E3023, with the lines to write:
  `fn setup(owns game: shares mut Game)`, called `setup(shares mut game)`, `with owns game`). The other
  reading area E listed (a plain `shares mut` parameter that changes and passes on) is not taken: the
  owner chose "A".
- A `MutSpan` name or parameter that writes says `ref mut` (R2, R4, R5): `d = ref mut
  tracks[dst_name].as_mut_span()`, `fn blend(ref mut dst: MutSpan[float], src: Span[float]):`, called
  `blend(ref mut d, s)`. R5's "a plain name may change what a `MutSpan` leads to" is withdrawn; a write
  through a plain one is E3023 with the line to write. A `ref mut dst: MutSpan[float]` parameter, like
  every `ref mut` parameter of a view, may also point the caller's span elsewhere; the caller's `d`
  holds a view as `rest` does, so `d = ref mut other.as_mut_span()` points it at another span (R2). v2's
  `ref mut s: MutSpan[T]` "to re-point the caller's view" is now the one writing parameter, which is
  what readability's F-14 asked for. Today's compiler already refuses a write through a plain `MutSpan`
  parameter (E3023, `r8_mutspan_plain.em`), with a help that says `mut dst`.
- What stays by type, and why: a plain name may still use a `Mutex`, `RwLock`, `Atomic` or channel end
  by its own operations, as verdict 3 requires (they take no word at a call). R5 now says why: each
  orders its own change (the lock's block, an atomic instruction), as Rust's `&self` methods of these
  types do; and by verdict 29 a function still names on its first line what it changes through one.

### 2.4 Kind tests (verdict 27)

R14 is rewritten: `if s is Circle as c:` runs its block when `s` is a `Circle` and names it with its
words (a value tested in its place stays a place: `if this.pools[k] is Pool[T] as ref mut pool:`);
`if s is Circle:` tests only; `elif`/`else` test other kinds, which is how a choice among kinds is
written now that no "maybe a `Circle`" value exists to `match` on. `cast s as Circle` stays. `is` before
a type name means "is of this kind", before a value "the same object"; `as` keeps its two uses. R13's
left side no longer lists `cast x if D`. Refused: today's `cast s if Circle`, `cast s when Circle`,
`match cast s if Circle:`, `as?` (E1010, with the line to write); a test that always passes (E2036); a
name already in use (E2106). The cost is v2's: one type-id compare.

### 2.5 Which arena (verdict 24)

R3's `using` point is rewritten: the place names the kind (`mesh: owns Mesh using LevelArena`), the
line that makes the value names the instance after the value (`owns Mesh(points=[0.0, 1.0]) using
current`, which calls `current.allocate(...)`; `Enemy(...) using current`; `[0.0, 1.0] using current`).
Checked when compiling: the instance is of that kind and the value given to the place was made in one
(E2020), and nothing made in an instance outlives it (R7's rule for values that hold views, E3063). The
verdict's example is an `owns` value; the same form is applied to every type `using` may follow, so
that a list field `names: Array[String] using LevelArena` has its making line too. **A cost this adds**,
stated in section 2: a value whose kind may have several instances keeps its instance's address so that
its growth and its end go back to it (one word, as Rust's `Box<T, &A>` keeps its `&A`); a kind with one
instance (`Global`, `System`) costs no word. Not measured.

### 2.6 The first owner of a shared value (verdict 23)

R3 and R9 state it as settled: after `other = shares mut items` (or `shares items`), `items` keeps what
it could do. R9 says why that is safe: one thread runs one line at a time, a list does not change while
a loop walks it through whichever of its names (R8 (c)), and a `shares` value never reaches another
thread (E7015).

### 2.7 Spellings (verdicts 26, 33)

Phase 0 changed the code; this step changed the prose, rows and messages. `exists as`: R13 says what
each part says (`exists` what the line checks, `as p` the name, as in `import x as y`); the vocabulary's
`exists as`, `if` and `as` rows cite verdict 26. Every remaining `holds` in the two files is English
("a value that holds views"), except the vocabulary's quotation of today's compiler on run 4's probe
`q1_holds.em`, which the lead restored. `.copy_each()`: the vocabulary's `.copied()` row says
"renamed", a new `.copy_each()` row carries the meaning, and the E3060 row of rules section 3 gives
AN-46's help with `.copy_each()`. The vocabulary now cites the owner's verdicts as "verdict N" (75
places that said "decision N"), as the brief and `inputs/JOB.md` do.

## 3. Every change from v2's rules

| v2 | v3 | what changed | why |
|---|---|---|---|
| title, opening | title, opening | step 2, v3; all 34 verdicts; `V3` paths; the "Power" line removed (this step ran in a cloud container; no power reading was taken) | — |
| section 0 | section 0 | view: names given by a kind test; read-only view: a `MutSpan` held by a `ref mut` name is the changeable one; first line: everything else it causes to change; new terms change list, kept callback, kind test; function value: "everything but its own state"; unknown call: a foreign function may run kept callbacks; memory manager: kind and instance | 21, 22, 24, 27, 31, 34 |
| R1 | R1 | unchanged | — |
| R2 | R2 | `ref mut` before a `MutSpan` a call gives says the name may write its elements, and a plain one only reads; a name holding a `MutSpan` is re-pointed as any view is; names given by a kind test keep their place | 27, 34 |
| R3 | R3 | the first owner keeps what it could do (stated); `using`: the place names the kind, the making line the instance, checked, one word per value of a several-instance kind; a lambda's changes follow its capture list, captured variables included | 22, 23, 24 |
| R4 | R4 | opens with "the word in front of the name, for every type"; `shares mut` parameter only reads (E3023 for making a changing owner from it); `MutSpan` parameter that writes says `ref mut`; `ref mut dst: MutSpan` may also re-point; a function value counts in the overlap rule | 21, 32, 34 |
| R5 | R5 | a change list may stand beside `changes this` and `owns this`; "everything else it causes to change, whenever"; `changes T` covers locks and atomics; `captured`; starting a thread and handing C a callback count; a change of a variable the function's own lambda captured needs no clause; an implementation may make its type parameters' changes; the plain-name rule names a local that holds a view and the elements of a `MutSpan`; the `MutSpan` exception withdrawn; the lock types kept, with why, and named on first lines | 21, 22, 29, 30, 31, 34 |
| R6, R7 | R6, R7 | unchanged | — |
| R8 | R8 | (b) foreign calls read the kept callbacks' types; (c) lists the calls that count; (f), (g) say how a `MutSpan` name fits; (g) a view whose last use is an argument does not live across the call | 21, 28, 30, 31, 34; area D's note 3 |
| R9 | R9 | the first owner's rights stated, with why they are safe | 23 |
| R10 | R10 | closing paragraph: across a call that may change objects of the view's type no read-only view lives; a walk keeps a link | 28 |
| R11 | R11 | making or ending a link is not a change of its target; the walk across a call that may delete nodes, its stop at the first ended parent, its cost and the region that removes the weak-count moves; v2's "reads after that call test" sentence removed; the view walk written `cur = ref n` ... `cur = ref p` | 28; area D's notes 2, 4, 6 |
| R12 | R12 | rewritten: a type says everything but the function's own state (captured variables by name, changes under a lock or through an `Atomic` by the type of what changes); `captured` / `captured...`; change lists; a function value in the overlap rule and the refusal of one stored where a variable it changes owns it; calls that run a function on their own; storing is not changing; foreign calls and kept callbacks, `@ffi(no_callbacks)`; types that keep a function carry its change list | 21, 22, 29, 30, 31; area D's note 1 |
| R13 | R13 | `cast x if D` removed from the left side; what `exists` and `as` each say; a kind test is `is` (R14) | 26, 27 |
| R14 | R14 | rewritten for kind tests | 27 |
| R15 | R15 | starting a thread counts; scoped tasks need no capture words (they name the captured variables they change); every foreign call may run kept callbacks unless marked | 22, 30, 31 |
| R16 | R16 | unchanged | — |
| R17 | R17 | the change lists of the callbacks a package hands to C are recorded in its interface file; a generic's changes through its type parameters are worked out per instance | 21, 31 |
| R18 | R18 | unchanged | — |
| section 2 | section 2 | rows added: the walk across a deleting call (cost, not measured); one word for a several-instance manager's address; nothing at run time for the compile-time checks of 21, 22, 29, 30, 31, 24; the `cast` row covers kind tests | 21, 22, 24, 27, 28, 29, 30, 31 |
| section 3 | section 3 | rows added: E3023 for a changing owner from a plain `shares mut` parameter (32) and for a write through a plain `MutSpan` name (34); E2105 for captured variables, a function given to `for_each`, a change under a lock, `captured` on a lambda, a thread started or a callback handed to C without naming its changes; E3020 for `for_each`, `note()` under a lock, `spawn`, `poll_events()` and an unmarked `sqrt`; E3021 for the view walk; E3022 for `LOG.retain(keep)` and a function stored in what it changes; E2113 for `captured` counts and a `dyn Iterator` of a changing adapter; E2020 and E3063 for arenas; E5069; E2036 and E2106 for kind tests; E3060 with AN-46's `.copy_each()` help; the removed-words row gains `cast x if D`, `cast x when D`, `as?`/`as!` respelled, `.copied()`; E3028's `integrate` message names `ref mut pos: MutSpan[float]` | 21, 22, 24, 27, 28, 29, 30, 31, 32, 33, 34 |
| section 4 | section 4 | section 6 below; "new in v2" marks replaced by bold for the model's own codes, with where each was checked unused | — |

## 4. Word changes

| rows of `V3/vocabulary.md` | change | caused by |
|---|---|---|
| title, opening, section 0 | step 2, v3; `V3` paths; the probes `r1` .. `r8`; the spec file named; "verdict N"; terms change list and kept callback; view and memory manager widened; the "Power" line removed | — |
| section 1 (agreed rows) | row 6 reads `b.on_click(fn() changes score: score += 1)`; the paragraph that said row 6 needs no `changes` word is replaced | 22 |
| section 2 | its history table (what changed from run 4, verdicts 1 to 20) moved out: it is `V2/changes.md` section 4; the old table is kept in the scratch file `vocab_v2_section2.md`; a pointer stands in its place | "write them clean" (the duty) |
| `as`, `is`, `if`, `match` | `is` before a type name, "is of this kind", and `if s is Circle as c:`; `as` keeps two uses; `if` lists the kind test apart from the opening; `match` on a maybe-downcast becomes `if`/`elif`/`else`; the opening cites verdict 26 | 26, 27 |
| `cast`, `as?`, `as!` | `cast x as D` stays; the "not sure" form is the kind test; `cast x if D` and `cast x when D` refused with the line to write; `as?` merges into the kind test | 27 |
| `fn`, `changes` | everything but the function's own state; captured variables named; `captured`; change lists; changes under a lock; threads and callbacks handed to C count | 21, 22, 29, 30, 31 |
| `captured`, `captured...` (new) | the word for the variables stored functions captured | 22 |
| a change list (new) | `[C]` ... `changes C` | 21, 30 |
| the iterator row; `Array`; `push`, ... | the first lines of `for_each`, `inspect`, `retain`, `sort_by` with a change list, and today's beside them | 21 |
| `thread.spawn`, `Scope` | their first lines with a change list; starting a thread counts | 30 |
| `@ffi` | a call giving a function to a `callback(retained)` or `callback(once)` parameter counts as making its changes; `no_callbacks` added; E5069 | 30, 31 |
| `Mutex`, `RwLock`, `Atomic`, `thread_shared`, `guarded by`, `fetch_add`/`store`/`load`, `send`/`channel` | a change made through them is named on first lines and in function types; a channel send is named too (`changes Sender[int]`, verdict 36); "R5's exception" reworded to R5's current text | 29 |
| `MutSpan`, `ref mut` | a name or parameter that writes through a `MutSpan` says `ref mut`; R5's exception by type gone; today's E3023 on `r8_mutspan_plain.em` cited | 34 |
| `using`, `Allocator`, `Array` | the instance on the making line; checked; one word per value of a several-instance kind | 24 |
| `shares`, `owns` | the first owner keeps what it could do; the word on the name sets the rights | 23, 32 |
| `weakref`, `while` | the walk across a call that may delete nodes; making a link is not a change | 28 |
| `exists as` | verdict 26 cited; v2's `holds` in "replaces"; what `exists` and `as` each say | 26 |
| `.copied()`, `.copy_each()` (new) | renamed | 33 |
| the note after table 3.6 | `captured` and `.copy_each()` added to the owner's spellings; `exists as` cited to verdict 26 | 22, 26, 33 |
| section 4 (placeholders) | rows for `exists as`, the arena instance, kind tests, `.copy_each()`, `captured`, change lists, `@ffi(no_callbacks)` | 21, 22, 24, 26, 27, 31, 33 |
| section 5 (words open) | v2's four open words are all answered (21, 22, 24, 27); four new ones from this step's readings (section 11) | 21, 22, 27, 29 |
| section 6 (the spec's uses) | `as?`/`as!` become the kind test and `cast x as D`; `.copied()` (lines 1451 2000 4367 4436 4440 6202); `MutSpan` parameters that write take `ref mut` (278 2520 3147 3618 3663 3704 6186; `[FN-1a]`; Annex A's `fill`); `[CLO-1]`/`[CLO-2]`'s function types; new rows for `[THR-10]`/`[THR-11]`, `[STD-19]`/`[STD-15]`/`[STD-16]`, `[FFI-11]`, `[ALC-1]`; `[FFI-21]` says registering counts | 21, 22, 24, 27, 30, 31, 33, 34 |

Every other row is v2's text, with "decision N" read as "verdict N".

## 5. The inputs' open items, and how each is settled

| item | settled |
|---|---|
| `V2/changes.md` section 11, question 1: library functions that run a function they are given | verdict 21: change lists (R12); the four first lines in the vocabulary's iterator row |
| question 2: what a function type names, and what "itself" includes | verdict 22: only its own state is itself; captured variables are named, and covered by `captured` in stored functions' types; a change through a link is still named by the target's type (`changes Ui`), as verdict 29's `changes Log` does |
| question 3: the first owner of a shared value | verdict 23: it keeps what it could do (R3, R9) |
| question 4: `using` with a manager that has several instances | verdict 24: the making line names the instance (R3) |
| question 5: the downcast opening's two `if`s | verdict 27: `if s is Circle as c:` (R14) |
| `V2/readability.md` F-12 (`.copied()`) | verdict 33: `.copy_each()` |
| F-14 (a plain local holding a `MutSpan` writes) | verdict 34: it says `ref mut` (R2, R4, R5) |
| C-1 (does a plain `fn()` cover a lambda that changes a captured variable?) | verdict 22: no |
| F-01 to F-11, F-13 (messages) | the lead's (phase 3); left as they are, since no rule this step changed rewrote those messages: F-06's E3023 row ("or make `game` with `shares mut`") is untouched, and verdict 32's message is a separate row |
| `V2/area-D.md` "For the owner" 1: the walk whose callback may remove nodes | verdict 28 (R10, R11) |
| `V2/area-D.md` notes for the rule writer | note 1 (a function value stored inside the variable it changes): refused (E3022, R12), now that the lambda names the variable; note 2 (R11's region walk against R8 (g)): settled by verdict 28; note 3 (a view as the argument of a call that may change it): R8 (g); note 4 (a link inside a region needs no weak count): R11, as the compile-time saving verdict 28 asks for; note 5 (a parent-link proof that fails silently): not settled here, a compiler note or `ember inspect` view is the lead's to decide; note 6 (`weakref this` in a method that names only a field): R11 |
| `V2/area-E.md` section 4.2, point 1 (a change under a lock) | verdict 29 (R12) |
| point 2 (what `spawn`, `scope.spawn` and a C callback parameter accept) | verdict 30: change lists in their first lines (R12) |
| point 3 (a foreign call may run a kept callback) | verdict 31 (R12, R15) |
| point 4 (a plain `shares mut` parameter cannot pass on a changing owner) | verdict 32 (R4) |
| `V3/spelling-changes.txt`: the lines left for the rule writer (`rules.md` 447, 455; `vocabulary.md` 295, 376, 378, 485, 514) | reworded: R13, R14; the `.copied()`, `as?`, `cast` rows; section 5's open word; section 6.1's `as?`/`as!` row |

## 6. Error-code changes

| code | change | reason |
|---|---|---|
| E5069 | **new**: `@ffi(no_callbacks)` on a foreign function whose own contract takes a callback; checked unused in the compiler, std, the development spec and `tasks/96/redesign/` (`free_codes.txt`) | 31 |
| E1010 | **widened** in its own meaning ("a name or form the program may not use here"): `cast x if D`, `cast x when D`, `.copied()` | 27, 33 |
| E2020 | **widened**: a value given to a `using A` place that was not made in an instance of `A` | 24 |
| E2036 | **widened**: a kind test that always passes | 27 |
| E2105 | **widened**: captured variables named on a lambda's first line (and `captured` written there instead); what library calls, threads started and callbacks handed to C change | 21, 22, 30, 31 |
| E2106 | **widened**: a kind test's name already in use | 27 |
| E2113 | **widened**: more captured variables than `captured` allows; a value whose type carries changes made a `dyn` value; an implementation may make its type parameters' changes | 21, 22 |
| E3020 | **widened**: a library call given a changing function, a `spawn` by what its thread changes, a foreign call by what the kept callbacks change | 21, 29, 30, 31 |
| E3021 | message added for the view walk across a call that may change nodes (its meaning is v2's) | 28 |
| E3022 | **widened**: a function value counts as an argument that changes what its type names; a function value stored in a place that a variable it changes owns | 21, 22 |
| E3023 | **widened**: a write through a plain name holding a `MutSpan`; a changing owner made from a plain parameter of a `shares mut` type (its meaning already covered "an owner that may change made from one that only reads") | 32, 34 |
| E3028 | reworded: a `MutSpan` parameter is a `ref mut` parameter now | 34 |
| E3060 | message added (AN-46's, with `.copy_each()`) | 33 |
| E3063 | **widened**: a value made in a memory manager's instance kept or returned where it would outlive that instance | 24 |

No code is retired in this step. E2116, E2117 and E2118 lose their "new in v2" marks, which were
history; they stay the model's own codes, in bold.

## 7. Today's Ember on the new spellings (`target/redesign4/bin/ember check`)

Files under `target/redesign4/v3-01-rule-writer/probes/`; output in `probes/check_output.txt`, pasted
whole. v2's probes `q1` .. `q8` still stand for `holds` (now `exists as`), `weakref`, `copy of`, `owns`,
`shares`, `using` in a type, `changes this` and a function type with `changes`.

```text
=== r1_captured.em
error[E0100]: expected `]`
 --> r1_captured.em:2:26
  |
2 |     handlers: Array[fn() changes captured] = []
  |                          ^^^^^^^ found `changes`
  |

error[E0100]: expected the end of the line
 --> r1_captured.em:2:26
  |
2 |     handlers: Array[fn() changes captured] = []
  |                          ^^^^^^^ found `changes`
  |

error[E2020]: field `handlers` of `Button` has no value
 --> r1_captured.em:5:9
  |
5 |     b = Button()
  |         ^^^^^^^^
  |

error: could not compile due to 3 error(s)
exit=1
=== r2_is_as.em
error[E1010]: cannot find `Circle` in this scope
 --> r2_is_as.em:8:13
  |
8 |     if s is Circle as c:
  |             ^^^^^^
  |

error[E2020]: `<error>` cannot be cast to `<error>` with `as`
 --> r2_is_as.em:8:13
  |
8 |     if s is Circle as c:
  |             ^^^^^^^^^^^
  |

error[E1010]: cannot find type `c` in this scope
 --> r2_is_as.em:8:23
  |
8 |     if s is Circle as c:
  |                       ^
  |

error[E1010]: cannot find `c` in this scope
 --> r2_is_as.em:9:16
  |
9 |         return c.r * c.r
  |                ^
  |
  = help: did you mean `s`?

error[E1010]: cannot find `c` in this scope
 --> r2_is_as.em:9:22
  |
9 |         return c.r * c.r
  |                      ^
  |
  = help: did you mean `s`?

error: could not compile due to 5 error(s)
exit=1
=== r3_copy_each.em
error[E2072]: `std.collections.MapKeys[String, i64, std.collections.DefaultHasher]` has no method named `copy_each`
 --> r3_copy_each.em:3:29
  |
3 |     names = sorted(m.keys().copy_each())
  |                             ^^^^^^^^^
  |

error: could not compile due to 1 error(s)
exit=1
=== r4_no_callbacks.em
error[E0104]: `no_callbacks` is not a contract word of `@ffi`
 --> r4_no_callbacks.em:2:5
  |
2 |     @ffi(no_callbacks)
  |     ^^^^^^^^^^^^^^^^^^
  |
  = help: `@ffi` on a foreign function takes param, result, effects, deterministic, link_name, variadic; the contract words are borrowed, owned, returns_owned, retained, one, count, nul_terminated, fixed, inout_count, nullable, from, exclusive, aliased, out, status, handle, callback, threads, destroys, unsafe, TODO
  = note: a hand declaration's contract is written `@ffi(param(name, word, …), result(word, …))` [FFI-11]

error: could not compile due to 1 error(s)
exit=1
=== r5_change_list.em
error[E0110]: `apply` is declared without a body
 --> r5_change_list.em:1:4
  |
1 | fn apply[C](f: fn(int) changes C) changes C:
  |    ^^^^^ a function here needs a body
  |
  = help: add ':' and a body

error[E0100]: expected `)`
 --> r5_change_list.em:1:24
  |
1 | fn apply[C](f: fn(int) changes C) changes C:
  |                        ^^^^^^^ found `changes`
  |

error[E0100]: expected the end of the line
 --> r5_change_list.em:1:24
  |
1 | fn apply[C](f: fn(int) changes C) changes C:
  |                        ^^^^^^^ found `changes`
  |

error[E0100]: expected an expression
 --> r5_change_list.em:2:5
  |
2 |     f(1)
  |     ^ found an indented block
  |

error[E1030]: `main` is already declared in this module
 --> r5_change_list.em:2:5
  |
2 |     f(1)
  |     ^
  |

error[E0100]: expected the end of the line
 --> r5_change_list.em:2:5
  |
2 |     f(1)
  |     ^ found `f`
  |

error[E2060]: cannot tell what `C` is here; write it out, as `apply[T](...)`
 --> r5_change_list.em:5:5
  |
5 |     apply(fn(x: int): println(x))
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |

error: could not compile due to 7 error(s)
exit=1
=== r6_using_current.em
error[E1010]: cannot find `owns` in this scope
 --> r6_using_current.em:6:9
  |
6 |     m = owns Mesh(points=[0.0, 1.0]) using current
  |         ^^^^
  |

error[E0100]: expected the end of the line
 --> r6_using_current.em:6:14
  |
6 |     m = owns Mesh(points=[0.0, 1.0]) using current
  |              ^^^^ found `Mesh`
  |

error: could not compile due to 2 error(s)
exit=1
=== r7_mutspan_ref_mut.em
error[E0100]: expected a name
 --> r7_mutspan_ref_mut.em:1:10
  |
1 | fn blend(ref mut dst: MutSpan[f64], src: Span[f64]):
  |          ^^^ found `ref`
  |

error[E0100]: expected `:`
 --> r7_mutspan_ref_mut.em:1:10
  |
1 | fn blend(ref mut dst: MutSpan[f64], src: Span[f64]):
  |          ^^^ found `ref`
  |

error[E1010]: cannot find type `dst` in this scope
 --> r7_mutspan_ref_mut.em:1:18
  |
1 | fn blend(ref mut dst: MutSpan[f64], src: Span[f64]):
  |                  ^^^
  |

error[E0100]: expected `)`
 --> r7_mutspan_ref_mut.em:1:21
  |
1 | fn blend(ref mut dst: MutSpan[f64], src: Span[f64]):
  |                     ^ found `:`
  |

error[E0100]: expected `=` after a list of assignment targets
 --> r7_mutspan_ref_mut.em:1:23
  |
1 | fn blend(ref mut dst: MutSpan[f64], src: Span[f64]):
  |                       ^^^^^^^^^^^^^^^^^
  |

error[E1010]: cannot find `src` in this scope
 --> r7_mutspan_ref_mut.em:1:37
  |
1 | fn blend(ref mut dst: MutSpan[f64], src: Span[f64]):
  |                                     ^^^
  |

error[E0100]: expected the end of the line
 --> r7_mutspan_ref_mut.em:1:40
  |
1 | fn blend(ref mut dst: MutSpan[f64], src: Span[f64]):
  |                                        ^ found `:`
  |

error[E0100]: expected an expression
 --> r7_mutspan_ref_mut.em:2:5
  |
2 |     dst[0] = src[0]
  |     ^ found an indented block
  |

error[E1030]: `main` is already declared in this module
 --> r7_mutspan_ref_mut.em:2:5
  |
2 |     dst[0] = src[0]
  |     ^
  |

error[E0100]: expected the end of the line
 --> r7_mutspan_ref_mut.em:2:5
  |
2 |     dst[0] = src[0]
  |     ^^^ found `dst`
  |

error[E0111]: only a place can be borrowed
 --> r7_mutspan_ref_mut.em:7:17
  |
7 |     d = ref mut a.as_mut_span()
  |                 ^^^^^^^^^^^^^^^ this is a value, not a place
  |
  = help: bind the value to a local first, then borrow the local

error[E2020]: `blend` takes 1 argument, found 2
 --> r7_mutspan_ref_mut.em:8:5
  |
1 | fn blend(ref mut dst: MutSpan[f64], src: Span[f64]):
  |    ----- `blend` is declared here
...
8 |     blend(ref mut d, b.as_span())
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |

error: could not compile due to 12 error(s)
exit=1
=== r8_mutspan_plain.em
error[E3023]: cannot mutate borrowed parameter `dst`
 --> r8_mutspan_plain.em:2:5
  |
1 | fn blend(dst: MutSpan[f64], src: Span[f64]):
  |          ----------------- `dst` is borrowed here
2 |     dst[0] = src[0]
  |     ^^^^^^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut dst: MutSpan[f64]`
  = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
  = help: if the value has identity, make it a `class`
  = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

error: could not compile due to 1 error(s)
exit=1
```

What it shows: none of `captured` (`r1`), the kind test `is T as` (`r2`: `is` takes a value, and `as c`
is read as a conversion), `.copy_each()` (`r3`, E2072), `@ffi(no_callbacks)` (`r4`, E0104, today's
contract words are listed in its help), a change-list parameter (`r5`), `using` on a making line (`r6`:
today stops at `owns`), or `ref mut` on a parameter's name (`r7`) is a word or form of today's Ember. A
write through a plain `MutSpan` parameter is refused today too (`r8`, E3023), with a help that gives
today's `mut dst: MutSpan[f64]` and offers `RefCell`; v3's message gives `ref mut dst` instead.

## 8. Answers the verdicts touch (for agents 02 to 06)

A pointer, not a ruling: each area updater decides its answers. Made by `touched_v3.py` from the text of
the five `V3/area-*.md` files (prose included, so a listed answer may only mention the construct);
output in `touched_v3.txt`.

| verdict | rule | answers |
|---|---|---|
| 21 library functions given a changing function | R12 | AN-04, AN-05, AN-16, AN-55 |
| 22 captured variables named; `captured` | R12, R3 | AN-05, AN-53 (row 6), AN-54, AN-57, AN-58, AN-60, AN-61, AN-71, AN-72, AN-76; and every scoped task that changes a captured variable (R15) |
| 23 the first owner keeps what it could do | R3, R9 | AN-34, AN-44, AN-45, AN-47, AN-48, AN-51, AN-52, AN-53, AN-57, AN-66, AN-78 |
| 24 the arena instance on the making line | R3 | AN-39, AN-40, AN-54, AN-71, AN-74, AN-76 |
| 27 kind tests replace `cast x if D` | R14 | AN-67, AN-68 |
| 28 a walk across a call that may delete nodes | R11 | AN-62 |
| 29 changes under a lock are named | R12 | AN-21, AN-30, AN-39, AN-71, AN-73, AN-74, AN-75, AN-76, AN-78, AN-80, AN-84 |
| 30 starting a thread counts | R5, R12, R15 | AN-02, AN-55, AN-71, AN-72, AN-73, AN-74, AN-75, AN-76, AN-80 |
| 31 foreign calls run kept callbacks; `@ffi(no_callbacks)` | R12 | AN-78, AN-84 |
| 32 the word on the name sets the rights | R4 | AN-39, AN-48, AN-49, AN-51, AN-52, AN-78 |
| 33 `.copy_each()` | vocabulary | AN-46, AN-59 |
| 34 a `MutSpan` name or parameter that writes says `ref mut` | R2, R4, R5 | AN-03, AN-11, AN-14, AN-16, AN-28, AN-71, AN-74, AN-84 |

Two consequences the area updaters will meet: AN-53's `Button.click` runs handlers of type
`fn() changes captured`, so its first line becomes `fn click() changes handlers, captured:` (R12,
verdict 35);
and AN-78's `setup` and `main` say `changes Game` (verdict 31's own example), while a function that
only calls `poll_events()` names nothing for the kept callback (R12).

## 9. Files

- `tasks/96/redesign/run4/v3/rules.md`, `vocabulary.md` (edited in place), `changes.md` (this file).
- Under `target/redesign4/v3-01-rule-writer/`: `probes/` (`r1` .. `r8` and `check_output.txt`, section
  7); `free_codes.txt` (E5069 unused, and today's meanings of the codes v3 widens, from
  `compiler/ember_diag/src/codes.rs`); `rows_v3_part1.py`, `rows_v3_part2.py` (the vocabulary row
  rewrites, each replacement asserted to occur exactly once; they ran once on the phase 0 copy);
  `vocab_v2_section2.md` (v2's vocabulary section 2, moved out); `touched_v3.py`, `touched_v3.txt`
  (section 8).

## 10. Run 4 step 1's questions for the owner, and where they went

All of `V2/changes.md` section 11 and of the "For the owner" sections of `V2/area-D.md`, `V2/area-E.md`
and `V2/readability.md` are answered by verdicts 21 to 34 (section 5 says where each landed). None is
carried over.

## 11. For the owner

What the verdicts leave open, one at a time, each with what this step does meanwhile.

1. **Answered (verdict 35, the owner: "a"): the word their type uses, `fn click() changes handlers,
   captured:`.** The question as it was put: **what a function that runs stored functions says about
   their captured variables** (verdicts 22 and 30). Verdict 22 settled the list's type (`handlers: Array[fn() changes captured]`); it did not
   say what `click`, which runs them, writes on its first line. This step reads verdict 30 ("`changes`
   only tells that a function will change something") as asking for it: `fn click() changes handlers,
   captured...:` (`captured...` since the list may hold several functions). Only the function that
   wrote the lambdas can reach those variables, and it checks them at every call that may run them, so
   safety does not need the word; the other choice is to name nothing for captured variables on first
   lines (`fn click() changes handlers:`). Which?
2. **Answered (verdict 36, the owner: "Yes go with a"): a send is named, `fn produce(tx: Sender[int])
   changes Sender[int]:`, as a change under a lock is.** The question as it was put: **how a change
   under a lock is named, and channel ends** (verdict 29). Meanwhile a change is named by
   the type of what changes: `changes Log` for a `Log` whose `guarded by` field changes,
   `changes Mutex[Stats]` for the value in a `Mutex[Stats]`, `changes Atomic[int]` for an atomic.
   Verdict 29 names `Mutex`, `RwLock` and `Atomic`, not channel ends, so a send (`tx.send(v)`) still
   names nothing: it changes only the channel's queue, which no loop walks and no view reaches. "One rule
   with no exception" would name it too (`changes Sender[int]`). Should a send be named?
3. **Iterator adapters that keep a function** (verdict 21 lists `inspect`, which runs its function
   later, at each `next`, not during the call). Meanwhile the adapter carries the function's change list
   as a type parameter (`Inspect[Self, C, F]`, `fn next() changes this, C -> Option[Item]`), driving it
   makes the changes, and an adapter whose list is not empty is not made a `dyn Iterator` (E2113). The
   alternative would give `Iterator` itself a change list that every generic consumer names (as Swift 6
   gives `AsyncIteratorProtocol` an associated `Failure` type), which costs every generic iterator
   function a word. Is the type-parameter form acceptable?
4. **The library's first lines take the function as a bound** (verdict 21's example takes a function
   value). Meanwhile `fn for_each[C, F: fn(Item) changes C](owns f: F) owns this changes C:`, as today's
   library writes `[F: fn(Item)]`: one instance per function given, so the call is direct and may be
   inlined, as Rust's `F: FnMut`. Your `fn for_each[C](f: fn(T) changes C) changes C:` means the same and
   reads shorter, but calls through the function value's two words on every item, slower than C and
   Rust unless the compiler specialises it. Keep the bound?
5. **A function that only makes a foreign call** (verdict 31: "a missing mark only refuses a loop").
   Meanwhile the kept callbacks' changes are named where they are handed to C (`setup` and `main` say
   `changes Game`), and a function that calls `poll_events()` names nothing for them, though R8 sees
   their changes at that call. The stricter reading puts `changes Game` on every function that makes an
   unmarked foreign call. Which?
6. **One word in every arena-made value** (verdict 24). Making the line name the instance means the
   value must find that instance again to grow or to end, so a value whose kind may have several
   instances keeps its address (one word, as Rust's `Box<T, &A>`); a kind with one instance costs
   nothing. The verdict's example is an `owns` field; this step applied the same making-line form to
   every type `using` follows (`[0.0, 1.0] using current` for a list). Not measured. Both as intended?
7. **The walk's compile-time saving** (verdict 28: "we will still try to optimise it"). Meanwhile R11
   places a walk across a deleting call in a deferred-drop region where the node class has no `drop`
   method, which removes the weak-count moves and leaves one header test per step. Nothing is measured;
   the next measuring phase should time it against run 3's `model_w2` (1.579x) and `rust_w2` (1.603x).
