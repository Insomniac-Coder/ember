# What changed from run 4, and why (run 4, step 1, v2; agent 01, rule writer)

Every change from run 4's rules (`RUN4/rules.md`) and words (`RUN4/vocabulary.md`) to `V2/rules.md` and
`V2/vocabulary.md`, each with the owner's decision or the area item that caused it; every error-code
change; what today's Ember says about the new spellings; and, at the end, what is left for the owner.
The rules and the vocabulary carry no history: it is all here.

`RUN3` means `tasks/96/redesign/run3/`, `RUN4` means `tasks/96/redesign/run4/`, `V2` means
`tasks/96/redesign/run4/v2/`. "Decision N" is the owner's decision N in `inputs/JOB.md`, section "Owner
verdicts on run 4's open questions" (2026-10-25); "R1" .. "R18" are the v2 rules; "row N" is the
owner's agreed row N. Terms are defined in `V2/rules.md` section 0.

Power: `PowerLineStatus` printed `Online` at the start, before the `ember check` runs and before writing
this file. No compiler change, no git command, no Ember program run (only
`target/redesign4/bin/ember.exe check`). Scratch files: `target/redesign4/v2-01-rule-writer/` (section 9).

## 1. The 20 decisions, and where each is applied

The three earlier verdict sections of `inputs/JOB.md` ("Readability verdicts on run 3's syntax", "Owner
verdicts on run 3's open word questions") were applied by run 4 and stay applied; where a run-4
spelling of them was replaced by a decision below (`ref mut this`, `if ref mut me = ui:`, `copied`,
`owned`, `shared`, `weak`), the decision wins.

| decision | what the owner decided | applied in |
|---|---|---|
| 1 | `holds` opens an `Option` or a `weak` link: `if cam.target holds p:`, `if ui holds ref mut me:`, `while ... holds ...:` | R13 (rewritten), R2, R10, R11, R14; vocabulary `holds` (new), `if`, `while`, `Some`, `cast`, `upgrade`, `Weak`, `weakref`; E0115, E2036, E2106 messages |
| 2 | `ref mut` keeps others from reading and changing; adding to the list stays allowed | R8 (f) (run 4's text, now settled, with "a link of the same type included"); vocabulary `ref mut`; E3021, E3022 |
| 3 | a `MutSpan` argument says `ref mut` at the call; `Mutex`, `RwLock`, `Atomic`, channel ends take no word | R4, R5, R8 (f); vocabulary `MutSpan`, `Mutex`, `RwLock`, `Atomic`, `ref mut`; E3028 widened |
| 4 | `shared mut` for an owner that may change the value; the family `ref`, `ref mut`, `sharedref mut`, `shared` / `shared mut` | R3, R4, R9, R10 (spelled `shares` / `shares mut` by decision 19); vocabulary `shares`, `Shared`, `mut`; E3023 widened (an owner made from a read-only one) |
| 5 | a read-only name promises that what it reads does not change while it is in use; `boss = copy of enemies[0]` is restricted by nothing | R8 (g) (new), R4, R6, R8 (a) (run 4's "a read-only position sees plain data written through another name" removed); vocabulary `ref`, `Span`, `for`; E3021 widened |
| 6 | `changes this` after the brackets replaces `ref mut this`; `changes log` the narrower form; neither: only reads | R5, R8 (f); vocabulary `this`, `changes`, `mut self`, `mut`, `override`, `virtual`, `init`, `Cell`; E2107, E2108, E0113, E1010, E2103, E3023, E2113 messages |
| 7 | `static fn` for a function in a class with no object | R5 (example `static fn starting_at(k: int) -> Counter:`); vocabulary `static`; E2115, E1010 messages |
| 8 | `static mut` stays; `static` alone never changes | R15; vocabulary `static`, `mut` |
| 9 | `weak` is renamed `weakref`; `Some(...)` not in `if`/`while` lines, kept to make a value and in a `match` branch | everywhere; R11, R13; vocabulary `weak` (renamed), `weakref` (new), `Weak`, `Some` |
| 10 | another link to the same target is `a = weakref self.keep`; `copied self.keep` refused with both ways out | R3, R11; vocabulary `weakref`, `copy of`; E2116 (new) |
| 11 | G-01 made safe: every weakref counted | R11 (run 4's rule, kept) |
| 12 | `copied` is renamed `copy of`; `of` a keyword only right after `copy`; rows 6 and 9 take the new spelling | everywhere; vocabulary `copy of` (new), `copied` (renamed), agreed rows 6 and 9 |
| 13 | a change silently lost on a temporary is refused, with both ways out; a call whose result is used stays allowed | R8 (d); E2140 (today's code, its meaning stated) |
| 14 | `set by` removed; a parent link is an ordinary field the program sets; make it fast by compile-time proof, or report the cost honestly | R11 (rewritten: the parent-link proof), R17; section 2's cost rows; vocabulary `set by` (removed), `weakref`; E2109 retired |
| 15 | a walk up a tree stops at the first deleted parent; the deleted part is held until the walk ends | R10 point 2 (set aside whole: everything it owns marked ended at once), R11 |
| 16 | one rule for every loop, generators included | R6, R8 (c), R15; vocabulary `for`, `yield`; E3020 widened, E3060 narrowed |
| 17 | functions say what they change, and so do lists of stored functions; plain `fn()` changes nothing outside itself | R12 (rewritten), R5, R8 (b), R10 point 1, R17; vocabulary `fn`, `changes`; E2113 and E2105 widened |
| 18 | `changes this` promises nothing about other names: no exclusivity, no noalias for `this`; `ref mut` names keep "only this name" | R4 (noalias), R5, R8 (f); vocabulary `this`, `ref mut` |
| 19 | `owned` -> `owns`, `shared` -> `shares`, everywhere; a memory manager named with `using` | everywhere; R3 (`using`); vocabulary `owns`, `shares`, `using` (new), `owned`, `shared` (renamed), `Box`, `Allocator`, `Array`; E2114 widened; E0109, E2085 messages |
| 20 | `thread_shared` and `sharedref mut` keep their spelling | unchanged |

## 2. The rules that were rewritten, and how

### 2.1 Parent links without `set by` (decisions 14, 15)

- **The rule** (R11). A parent link is an ordinary `weakref` field: `parent: weakref Node = None`,
  `child.parent = weakref this`, then `this.children.push(child)`. Like every `weakref` it holds a weak
  count, and a read tests the target's header, except where the compiler proves the target lives. It
  proves it from two conditions: (1) **the link follows ownership**: in the package, every line that
  gives the link a target is followed, before the link is read, by a line that puts the object into a
  place that target owns alone, every line that takes such an object out without ending it re-points or
  clears its link before the link is read, no owning list is handed over or exchanged whole, the objects
  have one owner and no `drop` method, and no other package can change the link or the owning places;
  (2) **nothing ends such an object** between the point where the object is known to live and the read
  (R10 point 1's test). Under (1) a live object's parent owns it, so the parent lives; under (2) that
  stays true along the walk. A walk `cur = ref leaf`, `while cur.parent holds p:`, `cur = p` is then
  compiled with no test per step.
- **Why it is sound.** (1) is an invariant of the program text: the target of every link owns the
  link's holder. An owner outlives what it owns, except during its own end, which (1) excludes by
  refusing the proof for a class with a `drop` method (a child's `drop` could read the link while its
  parent is ending). (2) excludes every line that could end an owner during the walk. The fact never
  changes what the program does: the test it removes would always pass (R17).
- **Where it cannot be proved**, every read tests, and the walk stops at the first ended parent
  (decision 15). An object ended by a call during the walk is set aside whole (R10 point 2 now marks it
  and everything it owns ended at once, so a node deleted "with everything under it" is ended at every
  level), or, under a count pair, kept as a memory block by the link's weak count; the next test finds it
  ended and the `while` stops. With decision 17, a walk that calls a plain `fn(Node)` callback has no
  line that may end a `Node` (the callback changes nothing outside itself), so (2) holds; only a
  callback whose type says `changes Node` (or another type that owns nodes) makes the walk test.
- **What it costs, honestly** (nothing was measured in this step):

  | walk | cost against plain C | source |
  |---|---|---|
  | proof holds: no header test per step | 1.176x | run 3's `diag_w2_hop_proved` (`RUN3/26-timer.md` line 22) |
  | proof fails: one header test per step | 1.579x | run 3's `model_w2` (line 21) |
  | safe Rust, the same walk | 1.603x | run 3's `rust_w2` (line 20) |

  Both model rows were measured with run 3's uncounted `set by` links. Parent links are now counted
  `weakref`s: one add to the parent's weak count when `child.parent = weakref this` runs and one
  subtract when the link changes or the child ends. W2 moves nodes between parents, so it would pay
  that on every move; it was not measured, and phase 3 (measure again) should run W2 with it.
- **What `set by` took with it**: its rule (run 4's R11 second half), its code E2109 (retired), its
  plain-load read row in section 2, its vocabulary row, and area D's open question on AN-62 (a node
  visited once more after its parent was removed), which the owner closed with decisions 14 and 15.

### 2.2 Function types say what they change (decision 17)

- **The rule** (R12). A function type says after its parameters what a function of that type may
  change outside itself: module variables (`fn() changes LOG`) and objects of a type that the function
  reaches through a link it holds (`fn() changes Ui`). A plain `fn()` changes nothing outside itself. A
  lambda says it on its first line (`fn() changes LOG: LOG.push(String("clicked"))`), checked against its
  body (E2105); a field or list of functions says it in its type (`handlers: Array[fn() changes LOG]`); a
  function that may change more than the type it is given to is refused (E2113). A call through a
  function value makes the changes its type says, and the function making the call names them on its
  first line (R5: `fn click() changes handlers, LOG:`).
- **What "itself" is**, settled here: the function value's own state (what it captured with a word and
  what that owns alone) and the real variables it captures (and what they own alone). The real
  variables need no word because the function value may not outlive them (E3026), so the function that
  writes the lambda sees every place it goes and checks them there (R12: a place holding the function
  value counts as reaching each captured real variable). This keeps agreed row 6 exactly as agreed:
  `b.on_click(fn(): score += 1)` with a plain `fn()` handler list. A change through a captured link
  (`weakref`, `shares mut`, `sharedref mut`) is outside itself, because other names may reach the same
  object; it is named by the object's type (`fn() with weakref ui changes Ui:`), the only name a type
  written in another class can use. Listed for the owner (section 11, question 2).
- **What it replaces**: run 4's R12 rule that a call that may run a function value is taken to make
  every change any function value of that type makes (the limitation G-07 named), and the summaries
  that recorded what function values change. Now G-04 (`run(log_click)` in a loop over `LOG`) is
  refused by `run`'s first line (`changes LOG`); G-05 (a handler reaching its Ui through `with weak ui`)
  needs `changes Ui` in its type, so it is refused where it is stored in an `Array[fn(Ui)]` (E2113) or,
  stored in an `Array[fn(Ui) changes Ui]`, the `for` walk over the list is refused (E3020); G-07's
  `b.click()` in a loop over `LOG` is accepted when `b`'s handlers are plain `fn()`.
- **A consequence**, listed for the owner (section 11, question 1): a library function that runs a
  function it is given (`for_each`, `retain`, `inspect`, `sort_by`) takes a plain `fn(T)` today, so a
  function that changes a module variable or a linked object cannot be given to it (E2113) until such a
  function can say "whatever the given function changes". Captured real variables are not affected:
  `xs.iter().for_each(fn(x): total += x)` stays accepted.

### 2.3 A read-only name promises no change (decision 5)

- **The rule** (R8 (g), new). While a read-only view is still used, what it points into is not changed
  through any other name (E3021 at the change), unless the compiler proves the two differ; a reshape
  ends it (E3060 at its later use, as before). Adding to the list that holds a read-only position stays
  allowed. A read-only parameter makes the same promise for the call (R4).
- **Removed**: run 4's R8 (a) sentence "A read-only position sees, at its next use, plain data written
  through another name" (readability F-04).
- **Consequences for answers**: AN-13 (`p = ps.get(0).unwrap()`, then `ps[0].life -= 0.5`, then
  `println(p.life)`) is now refused, with both ways out; AN-14 (two spans of one list given to one call,
  one changeable): run 4 compiled them without noalias; now they are proved apart with
  `assert_disjoint` (two compares, then noalias) or refused (R4); area B's AN-22 note 4 (a view living
  across a call that changes only a number in its object, wider than Rust) is now Rust's rule: the
  view points into the whole object (R6), so the change is refused unless `from` narrows it.

### 2.4 `changes this` (decisions 6, 18)

- R5: `changes this` after the brackets (any field, or the whole value), `changes a, b` (those fields),
  neither (only reads); `owns this` takes the object over. `changes this` stands without `reads` or a
  field, but a module variable or a type may stand beside it (E2107). It promises nothing about other
  names: `this` is never alone (R8 (f) no longer lists `ref mut this`) and never compiled with noalias
  (R4); a turret aimed at itself (`if this.target holds ref mut t: t.health -= 1`) is accepted with no
  check. R8 (b), (c), (g) keep views and walks safe without exclusivity, as they do for `sharedref mut`.
- `ref mut` parameters keep decision 2 and are compiled with noalias; run 4's exception "except where a
  link the function reads may reach the same object" is gone, because R8 (f) now refuses that link's use
  (a link of the same type is "another name that may name the same place") unless the line proves the
  two differ (`if t is not a:`). This settles run 4's question 4 (gap 8) the way decision 18 says.

### 2.5 Calls and `MutSpan` (decision 3)

R4: a call says `ref mut` before an argument given to a `ref mut` parameter or to a `MutSpan` parameter
(before a `MutSpan` the word names its elements, so it may stand before a span made in the call); a
`Mutex`, `RwLock`, `Atomic` or channel argument takes no word. E3028 covers both directions. Inside the
callee a `MutSpan` parameter is a `ref mut` name of its elements (R8 (f)). R5's "changed through any
name" exception stays for writes inside a body (`pos[i] = ...`).

### 2.6 One rule for every loop (decision 16)

R8 (c) covers a generator paused in a list as a walk of it, with the same code as a `for` loop (E3020);
R6's generator sentence points to it; R15's generator point says so. Run 4 refused the same program
with E3060 at the generator's later use; it is now E3020 at the change, the loop's own code and
message ("push after `g`'s last use").

### 2.7 A lost change on a temporary (decision 13)

R8 (d): a method that may change its object, called on a temporary that ends with the line, is refused
when the call's only effect is that change (it changes nothing else, through its first line or its
arguments, and nothing it gives back is used). E2140 is today's code for a change to a value that is not
a place ("cannot assign to a value expression"; today it refuses AN-10's program as "a `mut self` method
needs a variable to write through", pasted in `RUN4/area-A.md` AN-10); its message gives both ways out
of decision 13.

### 2.8 `shares` and `shares mut` (decisions 4, 19)

R3, R9: `shares x` makes an owner that reads, `shares mut x` one that may also change. The name the
value was first shared from stays an owner and keeps what it could do (settled here; section 11,
question 3); an owner made from another may do no more than it (E3023: `g2 = shares mut game` where
`game` only reads). A `shares` owner is an owner, not a view, so R8 (g)'s promise is not asked of it: it
sees changes made through owners that may change (R8 (g)'s last sentence). On a parameter:
`owns x: shares T` (one more reading owner), `owns x: shares mut T`, `ref mut x: shares mut T`.

### 2.9 `using` (decision 19)

R3: `using A` after a type whose value has heap memory of its own (`owns T`, `shares T`, `shares mut T`,
`thread_shared T`, a class object, `Array`, `Map`, `Set`); without it, the built-in manager (`Global`).
It replaces `[ALC-1]`'s allocator type parameter, so run 3's open point 2 (`Box[T, A]` had no spelling)
is closed. `using` after any other type is E2114 (widened, section 6). Where `A` has several instances, a
list is made in one as today (`Array.new_in(arena)`); a block of its own names a manager with one
instance, since there is no spelling yet for giving it one of several (section 11, question 4).

### 2.10 Spellings applied everywhere (decisions 1, 9, 10, 12, 19)

`holds` (R13), `weakref` (R11), `weakref this.keep` for another link, `copy of` (refused before a link,
E2116), `owns`, `shares`, `this`, `changes this`, `static fn`. `owns x` before a value is refused only
where the value's place already hands it over (E0109); it is the word for a `for` or `match` source that
hands its value over (`for x in owns xs:`), since a `for` loop's default is a view.

## 3. Every change from run 4's rules

| run 4 | v2 | what changed | why |
|---|---|---|---|
| section 0 | section 0 | every term in the new spellings; new terms: read-only view, parent link, memory manager; `Copy` lists `MutSpan` as not `Copy` and `StrView` as `Copy`; the deferred-drop region sets an object aside with everything it owns; a function value's type says what it changes; unknown call no longer includes calls whose type says what they change | decisions 5, 14, 15, 17, 19 |
| R1 | R1 | `b = a` where `a` is a view gives `b` a read-only view (`cur = ref p`) instead of reading as a hand-over | area D's note 1 |
| R2 | R2 | a call that makes a new value (a list of views included) hands it to the name, which owns it; only a call whose result is a view gives a read-only view; `copy of`; `cur = p` re-points | area A's note 6; decision 12 |
| R3 | R3 | `copy of`, `shares`, `shares mut`, `weakref` (with `weakref this.keep`), `owns`, `using`; an owner made from another may do no more than it (E3023); `copy of` refused before what it cannot copy (E2116); `owns` before a value refused only where its place already hands over; `for x in owns xs:` | decisions 4, 10, 12, 19 |
| R4 | R4 | read-only parameters promise no change; `ref mut` at the call for `MutSpan` arguments; no word for the lock types; the overlap rule counts spans of numbers too (proved apart or refused); every `ref mut` parameter noalias, `this` never | decisions 3, 5, 18 |
| R5 | R5 | `changes this` and `owns this` replace `ref mut this` and `owned this`; E2107 allows a module variable or a type beside them; `changes T` for function values' links; `changes this` promises nothing about other names; a virtual's line is checked one way; `static fn starting_at` | decisions 6, 7, 17, 18; area D's note 5 |
| R6 | R6 | `first(ref mut xs) += 1`; a view of an argument also stops other changes to it (R8 (g)); a generator walks what it views (E3020) | area B's note 3; decisions 5, 16 |
| R7 | R7 | a value holding views (a local list of views) is bounded in life, not made read-only; a class's view field without a usable `from` has a code (E2118) | area A's note 6; area B's note 1 |
| R8 | R8 | (a) loses the "sees plain data" sentence; (b) reads function values' changes from their types; (c) covers generators; (d) refuses a lost change on a temporary (E2140); (f) includes `MutSpan` parameters and links of the same type, drops `ref mut this`, and lets a call through a `ref mut` name of a stored function value give arguments that reach that name's place; (g) is new | decisions 2, 5, 13, 16, 17, 18; area D's note 2 |
| R9 | R9 | who may change a shared value: `shares mut` owners and the first owner; `shares` owners read | decision 4 |
| R10 | R10 | the protected links are `shares`, `shares mut`, `thread_shared`, `weakref`; point 1 counts calls whose function type may end an owner; point 2's region starts at any call that may end one and sets an object aside with everything it owns | decisions 15, 17 |
| R11 | R11 | `weakref`; `copy of` a link refused; `set by` removed; the parent-link proof; a walk stops at the first ended parent; L3001 on a type whose `shares` fields can form a cycle | decisions 9 to 11, 14, 15; area C's note 3 |
| R12 | R12 | rewritten: function types say what they change; "itself"; real variables checked in the writing function; run 4's rule that a call may make every change of every function value of its type removed | decision 17 |
| R13 | R13 | rewritten for `holds`; E2106 kept (the name after `holds` is new) | decision 1 |
| R14 | R14 | `if cast s if Circle holds c:`; a method no bound's interface declares is E2072 | decision 1; area D's note 4 |
| R15 | R15 | capture words respelled (`with owns`, `with copy of`, `with shares`, `with shares mut`, `with weakref`); `static` alone never changes; a generator paused in a list is a walk of it | decisions 8, 16, 19 |
| R16 | R16 | unchanged | — |
| R17 | R17 | inferred facts may also remove a test that would always pass; what function values change is now written (types, first lines), so it is no longer an inferred fact; the "only adds" summary also says the callee touches no element already in the list; the parent-link fact; a parent link proves itself only when no other package can change it | decisions 14, 17; area A's note 3 |
| R18 | R18 | "where it cannot see which function value runs, it says so" removed (function types say it) | decision 17 |
| section 2 | section 2 | the `set by` row replaced by the parent-link rows with the honest cost; the weak-count row includes parent links; the `assert_disjoint` row says when it is needed; a memory manager row | decisions 5, 14, 19 |
| section 3 | section 3 | every message in the new spellings; rows added for E3021 (read-only promise), E2140, E2116, E2117, E2118, E2113 (function values), E2020, E2072, E3023 (owner from a read-only owner), E3020 (generator), W2190; the `set by` row removed | the decisions; the areas' missing codes |
| section 4 | section 4 | section 6 below | — |

## 4. Word changes

`V2/vocabulary.md` section 2 lists them with their reasons; row by row:

| rows of `V2/vocabulary.md` | change | caused by |
|---|---|---|
| `holds` (new); `if`, `while`, `Some`, `cast`, `upgrade` | the opening with `holds`; `Some(...)` only to make a value and in a `match` branch | decisions 1, 9 |
| `ref mut` | keeps reads out too; written before every `MutSpan` argument; not on a method's object | decisions 2, 3, 18 |
| `MutSpan`, `Mutex`, `RwLock`, `Atomic` | the call's word: `ref mut` for a `MutSpan`, none for the others | decision 3 |
| `shares` (new), `shared` (now "renamed"), `Shared`, `class`, `dyn`, `mut`, `RefCell`, `Cell` | `shares` reads, `shares mut` may change; `mut` after `shares` | decisions 4, 19 |
| `ref`, `Span`, `for` | a read-only name promises no change while it is used | decision 5 |
| `this`, `changes`, `reads`, `self`, `mut self`, `override`, `virtual`, `init` | `changes this` after the brackets; no exclusivity for `this`; a virtual's line as its overrides' promise | decisions 6, 18; area D's note 5 |
| `static` | `static fn starting_at`; `static mut`, with `static` alone never changing | decisions 7, 8 |
| `weakref` (new), `weak` (now "renamed"), `Weak` | the new name; another link with `weakref this.keep`; counted always; parent links set by the program | decisions 9, 10, 11, 14 |
| `copy of` (new), `copied` (now "renamed"), `Copy`, `Clone`, `.copied()` | the new name; never a copy of a link (E2116) | decisions 10, 12 |
| `set by` | removed | decision 14 |
| `for`, `yield` | a generator paused in a list is a walk of it | decision 16 |
| `fn`, `changes`, `owned fn`, `with` | function types say what they change; capture words respelled | decisions 17, 19 |
| `owns` (new), `owned` (now "renamed"), `using` (new), `Box`, `Allocator`, `Array` | the verb; the memory manager after the type | decision 19 |
| `@cached`, `from` | their refusals have codes (E2117, E2118) | area B's note 1 |
| `get_mut` | its refusal is E3023 for a plain name; `ref mut` accepted | area D's notes 2, 3 |
| `assert_disjoint` | reads its arguments; only an unused result is warned | area A's note 4 |
| `@ffi` (`user_data`), the text-methods row (`find`), the iterator row (`count`), `Vec2`, `Vec3.ZERO`, the scalar row (`i16`), `_`, `*T`, `*mut T`, `read`, `add` | rows added or widened | the areas' "words needed" |
| `sharedref` | its class-field refusal names E2118 | area B's note 1 |
| every other row of section 3 | run 4's text, with the new spellings in its "will mean", "where", "refused" and "verdict" columns (`build_vocab_v2.py`) | decisions 1, 9, 12, 19 |
| section 1 (agreed rows) | rows 6 and 9 in `copy of` and `shares`; why row 6 needs no `changes` word | decisions 12, 17, 19 |
| sections 2, 4, 5 | rewritten for v2: what changed from run 4; placeholders settled by the owner; the four words still open | the decisions |
| section 6 (the spec's uses) | the "becomes" columns in the new spellings; rows added for the mode `owned` (86 spec lines, `spec_owned_lines.txt`) and the allocator parameter (2986, 4295, 4300); the `Node` example, `mut self`, `[FN-2a]`, `[FN-4]`, `[CLS-2]`, `[CLS-7]`, `[GRM-19]`, `[CLO-1]` rows rewritten | decisions 1, 3, 6, 14, 17, 19 |

## 5. The area files' items for the rule writer, and how each is settled

| item | settled |
|---|---|
| A 1 (for the owner): AN-10's lost change | decision 13; R8 (d), E2140 |
| A 2 (for the owner): `ref mut` alone in area A | decision 2; R8 (f) |
| A 3: the "only adds" summary needs one more fact | R17 and R8 (a), (f): the callee also reads and writes no element already in the list |
| A 4: `check_disjoint` and `assert_disjoint` take their arguments `owned` today | vocabulary `assert_disjoint`: they read their arguments; only an unused `check_disjoint` result is warned (W2190); "a view used after it took it" removed from the row (today's `std/src/mem.em:76, :85` are a library change for the lead) |
| A 5: three refusals with no code | `copy of` a view: E2116; `ref mut` on a loop name over a query item that only reads: E3027 (widened); AN-10's lost change: E2140 |
| A 6: is a local list of views read-only? | no: R2 says a call that makes a new list hands it to the name, which owns it (and may push to it); R7 bounds its life by what its views point into |
| A words: `Vec3.ZERO` | row added (with `Vec2`, which area C's AN-44 writes) |
| B 1: refusals with no code (`@cached` ×5; `from` ×3; a class's `StrView` field without `from`; `copied` of a value whose views point into its own `let` field) | E2117 (`@cached`); E2118 (`from` missing, naming a non-`let` field, a fixed array or inline field, or a name both a parameter and a field); E2116 (`copy of` such a value) |
| B 2: today's codes E2100, E2104, E2140 not listed | listed in section 4 of the rules with today's meanings and the cases the model gives them |
| B 3: R6's `first(xs) += 1` | written `first(ref mut xs) += 1` |
| B 4: AN-22 wider than Rust | decision 5 (R8 (g)) makes it Rust's rule; see 2.3 |
| B words: `find` | added to the text-methods row |
| C 1: E2200 for a view of the type itself | E2200's meaning names it: "a type that contains itself, or holds a view of its own type, with no block of its own between"; its message gives the parent link of R11 |
| C 2: E2200's "without `owned`" | reworded: "no block of its own between (an `owns` field, a list, a `shares` block)" |
| C 3: L3001 on the type or on a formed cycle | on the type, as today (the compiler cannot see every object a program forms); R11 and section 4 say so |
| C words: `_` | row added |
| D 1: `b = a` where `a` is a view name | R1: a read-only view of what `a` views; `ref` may say so (`cur = ref p`) |
| D 2: R8 (f) against R12 for AN-60's second way out | R8 (f): a call through a `ref mut` name of a stored function value copies the function out before its arguments are given, so the arguments may reach that name's place |
| D 3: the `get_mut` row's code | E3023 for a plain name; with `ref mut` the call is accepted (D 2) |
| D 4: refusals with no code | `set by` without `weak`: gone with `set by`; a lambda parameter's mode against the expected type: E2020 (today's "mismatched types"); a field read on an unopened `weakref` and an object stored into a `weakref` field without the word: E2020; a non-`pub` generic body calling a method no bound declares: E2072 (today's) |
| D 5: a virtual's first line and E2105 | R5: a virtual's line is its overrides' promise; E2105 checks a virtual's body one way only |
| D (for the owner): AN-62, a node visited after its parent was removed | decisions 14 and 15; R10 point 2, R11 |
| D words: `count` | added to the iterator row |
| E words: `count`, `read`, `add`, `*T`, `i16`, `user_data` | rows added (`count` in the iterator row, `i16` in the scalar row, `user_data` in the `@ffi` row, a row for raw pointers and their `read` and `add`) |
| E (for the owner): AN-77, a generator kept while its list changes | decision 16; R8 (c), E3020 |
| readability F-01 to F-04 | settled by decisions 1, 3, 4, 5 |
| readability F-05 to F-09 (messages) | the lead's; section 3's messages are only respelled here, except E3020's G-05 message and E3026's, whose lines would not compile under decision 17 without `fn(ref mut Ui)` and `changes Counter` |
| readability, found in passing: refusals with no code | all given codes above |

## 6. Error-code changes

| code | change | reason |
|---|---|---|
| E2116 | **new**: `copy of` before what it cannot copy as itself (a `weakref` link, a value of a view type, a type with no `Clone`, a value whose views point into its own `let` field) | decision 10; area A's note 5; area B's note 1 |
| E2117 | **new**: `@cached` on a method whose result it cannot keep safely | area B's note 1 |
| E2118 | **new**: a `from` missing where a view needs one, naming what a view cannot point into for its life, or naming two things | area B's note 1; R7 |
| E2109 | **retired** with `set by` | decision 14 |
| E2140 | **listed**, today's meaning ("cannot assign to a value expression"): a change to something that is not a place, which would be lost, now including decision 13's case | decision 13; area A's note 5; area B's note 2 |
| E3021 | **widened** in its own meaning ("a read and a change of one place overlap"): a change of what a read-only name reads while it is used | decision 5 |
| E3020 | **widened**: a list changed while a generator that walks it will still be used; a call through a function value counts by its type | decisions 16, 17 |
| E3060 | **narrowed**: the stored-generator case moved to E3020 | decision 16 |
| E2113 | **widened** in its own meaning ("a function that may change more than the one it stands in for promises"): a function value given where a function type allows less | decision 17 |
| E2105 | **widened**: a lambda's `changes`, and calls through function values by their types; a virtual's body checked one way | decision 17; area D's note 5 |
| E2114 | **widened** in its own meaning ("a type word that cannot apply to its type"): `using` after a type with no heap memory of its own, beside `owns` on a class type | decision 19 |
| E3027 | **widened** in its own meaning ("`ref mut` where the place may not be changed here"): a loop or pattern name over something it only reads | area A's note 5 |
| E3028 | **widened**: `MutSpan` arguments need `ref mut`; the lock types, `Atomic` and channel ends take none | decision 3 |
| E3023 | **widened**: an owner that may change made from one that only reads; messages with `changes this` | decisions 4, 6 |
| E2107, E2108 | respelled: `changes this` and `owns this`; E2107 allows a module variable or a type beside `changes this` | decision 6 |
| E0109 | respelled and narrowed: `owns` before a value its place already hands over, or before `fn`; a `for` or `match` source may say `owns` | decision 19 |
| E0113, E1010, E2103, E2085, E2173, E2200, E7004, E7010, E7015, L3001, E3026, E3013, E0115, E0116, E2036, E2106 | messages in the new spellings | decisions 1, 6, 9, 12, 19 |
| E2200 | meaning reworded: a view of its own type included; "no block of its own between" instead of "without `owned`" | area C's notes 1, 2 |
| E1030 | meaning corrected to today's ("two items with the same name in one scope", `compiler/ember_diag/src/codes.rs` line 188); run 4 gave only its second-`main` case | checked against today's registry |
| E2020, E2072, E2100, E2104, W2190 | **listed** with today's meanings, for the refusals the areas used them for | area B's note 2; area D's note 4; area A's note 4 |

The three new codes were checked unused in the compiler's sources, the spec and run 4's files; today's
meanings of every code listed as "today's" were read from the compiler's registry
(`target/redesign4/v2-01-rule-writer/free_codes.txt`).

## 7. Today's Ember on the new spellings (`target/redesign4/bin/ember.exe check`)

Files under `target/redesign4/v2-01-rule-writer/probes/`; output in `probes/check_output.txt`, pasted
whole. Run 4's probes `p1` .. `p6` (`RUN4/changes.md` section 9) still stand for `this`, `bump(ref mut
a)`, `sharedref`, the run-4 opening, `static fn` and `ref mut c: Counter`.

```text
=== q1_holds.em
error[E2035]: condition must be `bool`
 --> q1_holds.em:9:8
  |
9 |     if cam.target holds p:
  |        ^^^^^^^^^^ this is `Option[Player]`
  |
  = help: write `x is not None`

error[E0100]: expected `:`
 --> q1_holds.em:9:19
  |
9 |     if cam.target holds p:
  |                   ^^^^^ found `holds`
  |

error[E1010]: cannot find `holds` in this scope
 --> q1_holds.em:9:19
  |
9 |     if cam.target holds p:
  |                   ^^^^^
  |

error[E0100]: expected the end of the line
 --> q1_holds.em:9:25
  |
9 |     if cam.target holds p:
  |                         ^ found `p`
  |

error[E0100]: expected an expression
  --> q1_holds.em:10:9
   |
10 |         println(p.hp)
   |         ^ found an indented block
   |

error[E0100]: expected the end of the line
  --> q1_holds.em:10:9
   |
10 |         println(p.hp)
   |         ^^^^^^^ found `println`
   |

error: could not compile due to 6 error(s)
exit=1
=== q2_weakref.em
error[E1010]: cannot find type `weakref` in this scope
 --> q2_weakref.em:2:11
  |
2 |     keep: weakref Node
  |           ^^^^^^^
  |

error[E0100]: expected the end of the line
 --> q2_weakref.em:2:19
  |
2 |     keep: weakref Node
  |                   ^^^^ found `Node`
  |

error[E2020]: field `keep` of `Node` has no value
 --> q2_weakref.em:5:9
  |
5 |     n = Node()
  |         ^^^^^^
  |

error: could not compile due to 3 error(s)
exit=1
=== q3_copy_of.em
error[E1010]: cannot find `copy` in this scope
 --> q3_copy_of.em:3:16
  |
3 |     snapshot = copy of items
  |                ^^^^
  |

error[E0100]: expected the end of the line
 --> q3_copy_of.em:3:21
  |
3 |     snapshot = copy of items
  |                     ^^ found `of`
  |

error: could not compile due to 2 error(s)
exit=1
=== q4_owns.em
error[E1010]: cannot find type `owns` in this scope
 --> q4_owns.em:5:11
  |
5 |     mesh: owns Mesh
  |           ^^^^
  |

error[E0100]: expected the end of the line
 --> q4_owns.em:5:16
  |
5 |     mesh: owns Mesh
  |                ^^^^ found `Mesh`
  |

error[E0100]: expected `:`
 --> q4_owns.em:8:17
  |
8 |     fn add(owns child: Node):
  |                 ^^^^^ found `child`
  |

error[E1010]: cannot find type `child` in this scope
 --> q4_owns.em:8:17
  |
8 |     fn add(owns child: Node):
  |                 ^^^^^
  |

error[E0100]: expected `)`
 --> q4_owns.em:8:22
  |
8 |     fn add(owns child: Node):
  |                      ^ found `:`
  |

error[E1010]: cannot find `Node` in this scope
 --> q4_owns.em:8:24
  |
8 |     fn add(owns child: Node):
  |                        ^^^^
  |

error[E0100]: expected the end of the line
 --> q4_owns.em:8:28
  |
8 |     fn add(owns child: Node):
  |                            ^ found `)`
  |

error[E0100]: expected a field or method
 --> q4_owns.em:9:9
  |
9 |         pass
  |         ^ found an indented block
  |

error: could not compile due to 8 error(s)
exit=1
=== q5_shares.em
error[E1010]: cannot find `shares` in this scope
 --> q5_shares.em:3:13
  |
3 |     other = shares mut items
  |             ^^^^^^
  |

error[E0100]: expected the end of the line
 --> q5_shares.em:3:20
  |
3 |     other = shares mut items
  |                    ^^^ found `mut`
  |

error: could not compile due to 2 error(s)
exit=1
=== q6_using.em
error[E0100]: expected the end of the line
 --> q6_using.em:2:26
  |
2 |     names: Array[String] using LevelArena
  |                          ^^^^^ found `using`
  |

error: could not compile due to 1 error(s)
exit=1
=== q7_changes_this.em
error[E0110]: `reset` is declared without a body
 --> q7_changes_this.em:4:8
  |
4 |     fn reset() changes this:
  |        ^^^^^ a function here needs a body
  |
  = help: add ':' and a body

error[E0100]: expected the end of the line
 --> q7_changes_this.em:4:16
  |
4 |     fn reset() changes this:
  |                ^^^^^^^ found `changes`
  |

error[E0100]: expected a field or method
 --> q7_changes_this.em:5:9
  |
5 |         this.n = 0
  |         ^ found an indented block
  |

error[E2072]: `Counter` has no method named `reset`
 --> q7_changes_this.em:9:7
  |
9 |     c.reset()
  |       ^^^^^
  |

error: could not compile due to 4 error(s)
exit=1
=== q8_fn_changes.em
error[E1010]: `static mut` needs `unsafe`, which is not in this phase
 --> q8_fn_changes.em:1:12
  |
1 | static mut LOG: Array[String] = []
  |            ^^^
  |

error[E0100]: expected `]`
 --> q8_fn_changes.em:4:26
  |
4 |     handlers: Array[fn() changes LOG] = []
  |                          ^^^^^^^ found `changes`
  |

error[E0100]: expected the end of the line
 --> q8_fn_changes.em:4:26
  |
4 |     handlers: Array[fn() changes LOG] = []
  |                          ^^^^^^^ found `changes`
  |

error: could not compile due to 3 error(s)
exit=1
```

What it shows: none of `holds`, `weakref`, `copy of`, `owns`, `shares`, `using`, `changes this` or a
function type with `changes` is a word of today's Ember (each is E0100 or E1010); `copy` and `of` are not
reserved today (`copy` is looked up as a name), and today's `static mut` needs `unsafe` (q8), as
`[STA-1]` says.

## 8. Answers the decisions touch (for agents 02 to 06)

A pointer, not a ruling: each area updater decides its answers. Made by `touched_v2.py` from the text of
run 4's area files (prose included, so a listed answer may only mention the construct); output in
`touched_v2.txt`. Every answer takes the new spellings; the list names where meaning may change.

| change | rule | answers |
|---|---|---|
| `set by` removed: parent links set by the program, the walk's cost (decision 14) | R11 | AN-39, AN-41, AN-62, AN-63, AN-64, AN-70 |
| the opening with `holds` (decision 1) | R13 | AN-60, AN-62, AN-63, AN-65, AN-67, AN-68 |
| `changes this` for `ref mut this` (decision 6) | R5 | AN-10, AN-17 to AN-21, AN-23, AN-25, AN-32, AN-33, AN-53, AN-60 to AN-67, AN-70, AN-71, AN-77 |
| a `MutSpan` argument says `ref mut` (decision 3) | R4 | AN-03, AN-11, AN-14, AN-16, AN-28, AN-74, AN-84 |
| a read-only name promises no change (decision 5) | R8 (g) | AN-13 (now refused), AN-14 (overlapping spans: `assert_disjoint` or refused), AN-22 (area B's note 4), and every plain `boss = enemies[0]` beside a write |
| one rule for every loop (decision 16) | R8 (c) | AN-77, AN-84 |
| function types say what they change (decision 17) | R12 | AN-53 to AN-62, AN-70, AN-76 (G-04, G-05, G-07); AN-02, AN-05, AN-16, AN-29 mention handlers |
| a lost change on a temporary (decision 13) | R8 (d) | AN-10, AN-16 |
| `shares` / `shares mut` (decisions 4, 19): each `shared` owner that changes becomes `shares mut` | R3, R9 | AN-34, AN-36, AN-38, AN-39, AN-42 to AN-45, AN-47 to AN-49, AN-51, AN-52, AN-53 to AN-55, AN-57, AN-60, AN-63, AN-64, AN-66, AN-67, AN-71, AN-74 to AN-76, AN-78 |
| `using` for `Box[T, A]` (decision 19) | R3 | AN-40 |
| new codes for refusals that had none | rules section 4 | AN-07, AN-10, AN-15 (area A); AN-21, AN-22, AN-25, AN-26 (area B); AN-41 (area C); AN-56, AN-61, AN-63, AN-65, AN-69, AN-70 (area D) |
| words added | vocabulary 3.8 | AN-13 (`Vec3.ZERO`), AN-22 (`find`), AN-35, AN-37 (`_`), AN-55, AN-71, AN-73 (`count`), AN-78, AN-79, AN-84 (`*T`, `read`, `add`, `i16`, `user_data`) |

## 9. Files

- `C:\Users\ism19\Code\ember\tasks\96\redesign\run4\v2\rules.md`, `vocabulary.md`, `changes.md`.
- Under `C:\Users\ism19\Code\ember\target\redesign4\v2-01-rule-writer\`: `build_vocab_v2.py` (builds the
  vocabulary's table and spec sections from run 4's), `rows_v2.md` (its hand-written rows),
  `fix_spec_v2.py` (the hand fixes to section 6), `vocab_head.md`, `vocab_tail.md`,
  `assemble_vocab.py`, `leftovers.py` (finds old spellings left in new-meaning columns),
  `scan_words_v2.py` and `scan_words_v2.txt` (names in run 4's answers with no v2 row: only the
  programs' own names are left), `touched_v2.py` and `touched_v2.txt` (section 8),
  `spec_owned_lines.txt` (the spec lines that use `owned` and an allocator parameter), `free_codes.txt`
  (the new codes unused; today's meanings), `probes\` (`q1` .. `q8` and `check_output.txt`).

## 10. Run 4's questions for the owner, and where they went

| run 4 question (`RUN4/changes.md` section 13; `RUN4/readability.md` section 7) | answered by |
|---|---|
| G-01 made safe, not refused | decision 11 |
| the spellings settled in run 4 | decisions 1, 6, 7, 20 |
| `ref mut` alone; what "touch" means | decision 2 |
| what `ref mut` promises against links (gap 8) | decision 18 (2.4) |
| a word for what a function value changes | decision 17 (2.2) |
| `static mut` keeps a bare `mut` | decision 8 |
| `Box[T, A]` | decision 19 (`using`, 2.9) |
| the opening form | decision 1 |
| calls that change an argument through its type | decision 3 |
| changing through a `shared` owner | decision 4 |
| a plain name that follows later changes | decision 5 |

## 11. For the owner

Questions only the owner can decide, one at a time, each with what this step does meanwhile.

1. **Library functions that run a function they are given** (decision 17's consequence). `for_each`,
   `retain`, `inspect` and `sort_by` take a plain `fn(T)`, which "changes nothing outside itself", so
   `xs.iter().for_each(fn(x) changes LOG: LOG.push(x))` is refused (E2113) even though the call runs the
   function only while it lasts. Captured local variables are not affected (`for_each(fn(x): total +=
   x)` stays accepted, see question 2). The general answer is a library function that takes "whatever
   the given function changes" and says it changes the same, for example
   `fn for_each[C](f: fn(T) changes C) changes C:`. Meanwhile such a loop is written by hand
   (`for x in xs: LOG.push(x)`). Should the library get that form?
2. **What a function type names, and what "itself" includes** (decision 17, settled here for your
   verdict). A function changes nothing outside itself when it changes only its own copies and the
   local variables it uses from the function that wrote it (those cannot outlive that function, which
   checks them), so row 6's `b.on_click(fn(): score += 1)` stays as agreed with a plain `fn()`. A change
   made through a link the function holds (`with weakref ui`, `with shares mut count`) reaches an object
   other names may reach, so the type names the object's type: `fn() with weakref ui changes Ui:`,
   `handlers: Array[fn() changes Ui]`. Is that the reading you meant?
3. **The first owner of a shared value** (decision 4, settled here). After `other = shares mut items`,
   `items` stays an owner and may still change the list, as its own local could, and sees what `other`
   changes; the line names both. The stricter choice: sharing turns `items` into a reading owner too,
   so changing it needs its own `shares mut`. Which?
4. **`using` with a memory manager that has several instances** (decision 19). `mesh: owns Mesh using
   LevelArena` says where the mesh's block comes from; when `LevelArena` has more than one arena, a list
   is made in one as today (`Array.new_in(arena)`), but storing a plain `Mesh` into the `owns` field has
   no way to name the arena. Meanwhile a block of its own (`owns`, `shares`, a class object) names a
   manager with one instance after `using`, as `Global` and `System` are (R3). A way to name the
   instance (for example on the type that holds the field) is yours to choose.
5. **The downcast opening reads with two `if`s**: `if cast s if Circle holds c:` (today's `cast x if
   D`, opened with `holds`). A shorter line would need a new word (for example `if s holds Circle c:`).
   Keep it?
