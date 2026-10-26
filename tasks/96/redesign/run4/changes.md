# What changed from run 3, and why (run 4, phase 1; agent 01, rule writer)

Every change from run 3's rules (`RUN3/combined-v4.md` sections 0 to 9) and words
(`RUN3/vocabulary-final.md`) to `RUN4/rules.md` and `RUN4/vocabulary.md`, each with its reason and the
problem it settles; every error-code change; what of run 3 still stands; and, at the end, what is left
for the owner. The rules and the vocabulary themselves carry no history: it is all here.

`RUN3` means `tasks/96/redesign/run3/`, `RUN4` means `tasks/96/redesign/run4/`. "R1" .. "R18" are the
run-4 rules; run 3's rules are written "run 3's R2". "G-01" .. "G-11" are the problems in
`RUN3/22-re-attacker.md`; "R18-01" .. "R18-22" those in `RUN3/18-readability.md`; "gap 1", "gap 2",
"gap 8" those in `RUN3/24-faithfulness.md`; "F-01" .. "F-12" those in `RUN3/20-re-attacker.md`. Terms
are defined in `RUN4/rules.md` section 0.

Power: `PowerLineStatus` printed `Online` at the start, before the `ember check` runs and before writing
this file. No compiler change, no git command, no Ember program run (only
`target/redesign4/bin/ember.exe check`). Scratch files: `target/redesign4/01-rule-writer/` (section 12).

## 1. How the rules were rewritten

Run 3's rules grew by patching: each round added a special case to a rule and opened new holes (48,
then 12, then 11 problems; `tasks/96/redesign/FINDINGS-run3.md` section 2). Here each rule a problem
touches was rewritten whole, as one plain statement, and checked against all the problems at once.
Four of the rewrites carry most of the weight:

1. **"Into an argument" means into everything it owns** (R6, section 0). Run 3 recorded some views as
   pointing "into an object" and others "into a part", and said a push to one of an object's lists does
   not end a view "into the object". One meaning, Rust's (a borrow of a whole argument), removes G-02 and
   the generator special case of F-09.
2. **Every `weak` link is counted, with no exception** (R11). Run 3 left the weak-count add out where it
   judged the copy's life covered; the judgement was wrong twice (F-01, then G-01). The one place a count
   was worth saving, a walk up `set by` parent links, now walks with views and needs neither a count nor
   a test (gap 1).
3. **One overlap rule for a call's arguments** (R4). Run 3 had two-place promises for `ref mut`, then a
   special case for a `MutSpan` of non-`Copy` elements with a run-time test that cannot see nested
   ownership (G-03). Now: arguments that may be changed may not overlap; an index compare where two
   paths differ only by an index; anything else possible is refused when compiling.
4. **One rule for what a call that may run a function value does** (R12). Run 3 listed three kinds of
   call that may run one, missed a fourth (G-04), and did not count what a function value reaches
   through its captured links (G-05). Now any call that may run a function value is taken to make every
   change those values make, through captured variables and captured links alike.

## 2. The owner's four decisions of 2026-10-25, and the spellings settled

### 2.1 A call that changes an argument says so: `bump(ref mut a)` (R18-16)

- **Rule:** R4. A call writes `ref mut` before an argument given to a `ref mut` parameter, with the same
  words as the parameter; a missing or an extra one is E3028 (new), and `ref mut` before something the
  function may not change is E3027 (today's code and meaning). The object a method is called on is not
  marked (`a.reset()`), as the owner ruled. Calls through function values follow the same rule
  (`h(ref mut this)` for `fn(ref mut Ui)`), and so do the library's own (`mem.swap(ref mut a, ref mut b)`).
- **Also:** an argument whose type is one of R5's "changed through any name" types (`MutSpan`, `Mutex`,
  `RwLock`, `Atomic`, a channel end) needs no word, since its type says it; a hand-over to an `owned`
  parameter has no word, as row 9's `=` has none.
- **Today:** `bump(ref mut a)` is already accepted as an argument with today's `mut c: Counter`
  parameter (`p2_call_ref_mut.em`, section 9: `exit=0`), so the call-site spelling is one today's
  parser knows.
- Replaces run 3's R4 "The call writes no word" and run 3's clash 9 and open points 1 and 6.

### 2.2 No `self`; the object is `this`; `ref mut this` (R18-14)

- **Rule:** R5. A method does not list its object; inside it the object is `this`. What the method does
  to `this` is said after the parameters, in one place:

  | first line | the method |
  |---|---|
  | `fn show():` | only reads `this` |
  | `fn load_assets() reads renderer -> Array[String]:` | reads exactly `renderer` (row 7) |
  | `fn note(msg: String) changes log:` | may change exactly `log` (row 3) |
  | `fn reset() ref mut this:` | may change any field, or the whole value |
  | `fn finish() owned this -> u64:` | takes the object over; the caller's name ends |

- **Where `ref mut this` goes, and how it sits with `changes`** (the duty's first open spelling):
  after the parameters, in the place `reads` and `changes` already hold, and alone. Reasons: (1) the
  owner ruled that methods do not list their object as a parameter, so `fn reset(ref mut this):` is
  out; (2) every other thing the method does to its object is already said in that place (row 3's
  `changes log`, row 7's `reads renderer`), so one place on the line says all of it, and a reader
  looking for "does this change its object?" looks in one place; (3) C++ puts the same kind of word
  there (`void reset() const`); (4) `ref mut this` already lets the method change every field, so beside
  `changes` it would say the same thing twice or contradict it: it stands alone (E2107), and `changes`
  is the narrower form for a method that changes only some fields. Rejected: `ref mut this fn reset():`
  (puts the object before `fn`, away from `reads` and `changes`), `fn ref mut this.reset():` (splits the
  name). The owner's test: in `fn reset() ref mut this:`, `this` says what the name is (the object the
  method was called on), `ref mut` that it may change it; in `fn note(msg: String) changes log:`,
  `changes log` says which part may change.
- **`owned this`** replaces today's `owned self` (`fn into_iter(owned self)`, spec 1318), in the same
  place.
- **`static fn`** (a consequence, for the owner to confirm): with no object in the parameter list,
  today's rule "a function in a type body with no receiver is an associated function" (`[FN-4]`, spec
  1582) has nothing left to look at. Deciding by whether the body uses `this` would be the compiler
  deciding what the code does not say (it changes how the function is called), so a word is needed:
  `static fn make(n: int) -> Pool:`, called `Pool.make(3)`. Java, C#, C++, Swift and TypeScript write
  `static` for exactly this, and the word keeps its Ember sense, "one for the whole program, not one per
  object" (`static` items, `static mut`). `static fn` outside a type body is E2115.
- **`init`** sets fields through `this` with no word; `ref mut this` or `owned this` on it is E2108
  (run 3's `fn init(mut self)` refusal, AN-32, now with a code).
- **Rows 3 and 7** become `fn note(msg: String) changes log:` and
  `fn load_assets() reads renderer -> Array[String]:`, meaning unchanged, as the owner ruled.
- **`self`** is refused with E1010 and the line to write; `mut this` is E0113.
- **`mut self`** is gone, so no name keeps a bare `mut` (R18-14's first half settled). **`static mut`**
  keeps it: `static` says whose variable it is (the module's), so the line passes the owner's test; it is
  listed for the owner (section 13).

### 2.3 A changeable link others may share: `sharedref mut` (R18-22)

- **Rule:** R7, R8 (f), R9. `ref mut` now has the owner's one meaning everywhere, "only this name may
  touch it until it is done": a kept `ref mut` field too (while its holder is used, the place is not
  read or changed through any other name: E3021, E3022). A field that holds a changeable link other
  holders may also hold is `sharedref mut T`: on one thread any holder, and the owner, may change the
  target at any time (a number such as `hp`); a list in it may not change while a `for` loop walks it,
  whichever holder reaches it; on two threads, only under a lock or through `Atomic`. These are the
  owner's own three checks, as R7 and R9 state them.
- **Which spelling** (the duty's second open spelling): `sharedref mut`, of the owner's two. Both pass
  the "may change" half of the owner's test with `mut`. For "what it is", `sharedref` says both halves
  in words a reader already knows: "shared" (others hold it too) and "ref" (it does not own it, unlike
  `shared`, which owns and is counted). `sref` asks the reader to know that `s` stands for "shared": the
  brief's "a keyword says exactly what it does" fails on a one-letter abbreviation. So `sharedref mut`
  passes and `sref mut` does not; no third word was needed.
- `sharedref` is always followed by `mut` (E0114): a link that only reads is `ref T`, which any number
  of names may already hold, so `sharedref T` would be a second spelling of it.
- Written the same way before a value: `Watcher(target=sharedref mut players[r])`, as `shared x` and
  `weak x` are.
- Run 3's AN-52 (b) (watchers keeping `target: ref mut Player` that may name one player) becomes
  `target: sharedref mut Player`, with run 3's cost (1.00x to 1.06x of plain C, `RUN3/combined-v4.md`
  line 7298, PROVISIONAL): the same C.
- Replaces run 3's R9 sentence "Kept `ref mut` views ... may name one object", clash 3, and open points
  4 and 8.

### 2.4 A readable replacement for `if Some(ref mut me) = ui:` (R13)

- **Rule:** R13. `if ref mut me = ui:` runs its block when the link still points at something and calls
  that thing `me`, which may change it; `if p = cam.target:` is the read-only form; `while p = a:` walks;
  `else:` runs when there is nothing. The right side must be able to hold nothing (an `Option`, a `weak`
  link, `cast x if D`), otherwise the line is refused (E2036, today's code and meaning, "this pattern
  always matches"); the name must be new (E2106), so the `=` cannot hide a typo for `==`. `Some(...)` is
  written as a value and in `match` arms, not in an `if` or `while` header (E0115).
- **Why this form** (the duty's third open spelling). What made run 3's form unreadable is the
  `Some( ... )` wrapper around the name, with the mode words nested inside it. The new form keeps the two
  things a reader of Ember already knows: `=` names a value (every `x = ...` line), and the mode words sit
  on the name (`for ref mut p in players:`, `ref mut b: B`). Read aloud: "if there is a `me` that may
  change, from `ui`". It is Swift's `if let me = ui` without `let`, a form widely taken as readable.
  And it is what programmers write first: run 3's AN-68 is a programmer writing exactly
  `if c = cast s if Circle:` and today's compiler refusing it (`E2036`, pasted in
  `RUN3/combined-v4.md` lines 9032-9039); under R13 that line is the accepted one.
- **Rejected:** `if ui as ref mut me:` reads most like the owner's sentence ("if ui, as me"), but `as`
  already means "convert to" (`x as T`) and is part of `cast x as D`; with the cast that `if` needs most,
  it would read `if cast s if Circle as c:`, which is worse than run 3's form. Kotlin's smart cast
  (`if ui is not None: ui.clicks += 1`) leaves no word on the name that says it may change the Ui
  (fails the owner's test). `with ref mut me = ui:` does not say that the block is skipped when the Ui
  is gone (Python's `with` always runs).
- **The owner's test:** in `if ref mut me = ui:`, `ref mut` says what `me` is (a name for the Ui the link
  points at, someone else's) and that it may change it.
- **Today:** `if v = m:` with `m: Option[int]` is E2036 today (`p4_if_open.em`, section 9).

## 3. The five programs the rules wrongly accepted (G-01 to G-05)

Each with the rule that now decides it and the message. The programs are those of
`RUN3/22-re-attacker.md`, written with this run's words (`this`, the opening form, `ref mut` at calls).

| program | outcome | rule | message |
|---|---|---|---|
| G-01: a copy of a `weak` link inside a deferred-drop region, whose target ended before the loop | **accepted, now safe; not refused** | R11: every `weak` link holds one weak count, no exception | none: `a = copied this.keep` takes its count, so the block stays while `a` lives; `if n = a:` finds the node gone and the program returns 0, as it means |
| G-02: `pick(ref mut h.view, bag)` puts a view of `bag`'s list into `h.view`; `bag.items.push(k)`; `h.view[0]` | refused | R4 (a view put in through `ref mut` may point into the call's other arguments), R6 ("into" means into everything the argument owns), R8 (b) | E3060: "`bag.items.push(k)` (line 15) may move the list that `h.view` points into (`pick` may have put a view of `bag` there on line 13), and `h.view` is used on line 16; use `h.view` before line 15" |
| G-03: `refill(w1.lists.as_mut_span(), w2.lists[0].as_span())` with `w1`, `w2` two owners of one `Wrapper` | refused | R4: arguments that may be changed may not overlap; only an index-only difference is checked while running | E3021: "`refill` may replace the lists of `w1.lists` through `dst` while `src` shows numbers inside one of them, and `w1` and `w2` may be the same `Wrapper`; give `src` a list of its own: `first = copied w2.lists[0]`, then `refill(w1.lists.as_mut_span(), first.as_span())`" |
| G-04: `for entry in LOG: run(log_click)`, where `run` calls the function it is given and `log_click` pushes to `LOG` | refused | R12: a call whose callee's summary says it runs a function value it is given makes every change that value makes; R8 (c) | E3020: "this loop walks `LOG` (line 8), and `run(log_click)` (line 9) runs `log_click`, which pushes to `LOG` (line 7); call it after the loop, or collect what to add and push it after the loop" |
| G-05: `for ref mut h in this.handlers: h(this)`, with a handler that captures `weak ui` and pushes to `me.handlers` | refused | R12: a call that may run a function value makes every change its captured links reach; R8 (b)'s "may reach the same object"; R8 (c) | E3020: "this loop walks `this.handlers` (line 7), and `h(this)` may run the handler made on line 15, which adds to the `handlers` of the `Ui` it links to (`with weak ui`), and that may be this one; walk by position instead of the `for` line: `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](this)`, `this.current += 1`, with `fn fire() changes handlers, current:`" |

None of the five needed a special case: each is decided by a rule stated for every program (R4, R6,
R11, R12 apply to every call, every view, every link and every function value).

**G-01 is not refused, and on purpose.** The program is correct: it keeps a link to a node, the node
ends, and the walk asks the link whether the node still lives. Python's `weakref` and Rust's
`Weak::upgrade` both accept it and answer "gone". Run 3 accepted it with memory that could be freed under
the link (an uncounted copy); R11 makes the copy hold its count, so the answer is "gone" and nothing is
read from freed memory. Refusing it would need a rule against copying a `weak` link inside a loop, a
special case that protects nothing once the copy is counted. The cost is Rust's: one add and one subtract
of the weak count per copy. The brief says the five programs must be refused; this one is made safe
instead, and that is listed for the owner (section 13). Mode 2 of 22's proof
(`target/redesign3/22-re-attacker/c/g1_weak_copy_dead_target_output.txt`: `mode2 total=0`, no
AddressSanitizer report) is the C that R11 gives.

The other four are refused when compiling; none adds a check while the program runs (G-03's run-time
test is removed, not added).

## 4. The six readability problems (G-06 to G-11)

| problem | settled by | how |
|---|---|---|
| G-06: `changes LOG` required on a function that only writes a lambda pushing to `LOG` | R5 | the function that writes a lambda no longer names what the lambda changes (run 3's F-04 (a) is removed); `changes LOG` keeps one meaning, "a call of this function changes `LOG`". What function values change is stated in R5 as the one thing first lines do not show, and R12 checks it. Run 3's G-06 noted that (a) added nothing to safety |
| G-07: the refusal named a handler the button does not hold | R12, R18 | R12 states the breadth as a limitation ("the compiler does not follow which handlers each object holds"), and R18 asks a message to say only what the compiler knows: "`b.click()` (line 6) may run a handler stored in a `Button`, and the one made on line 4 pushes to `LOG`; the compiler does not follow which handlers each button holds; click after the loop, or collect what to add and push it after the loop" (E3020). Precision needs a word for what a function value changes, left for the owner (section 13) |
| G-08: `rest = tail.trim()` in a parse loop refused; `cur = Cursor(...)` accepted with the same meaning | R2 | a name declared with `=` stands for its own place, so `=` gives it a new value, a view included: `rest = tail.trim()` is accepted (as today, `RUN3/22-re-attacker.md` section 3 G-08 shows `exit=0`, and as Rust's `let mut rest`), checked by R8 against the new target, and `cur = Cursor(...)` follows the same sentence: one rule for a bare view and a struct holding one. A parameter, a loop, pattern or opened name keeps what it was given for its scope (run 3's R18-20 rule, widened), with a message that names a new local. R18-03's worry ("one shape, two meanings") is answered by the declaring line's word: a plain name stands for its own place; a `ref mut` name for the place it names, so `r = 20` writes `a` |
| G-09: AN-78's E7015 message led to a program E7014 refuses | R18; rules.md section 3 | R18: the line a message gives must compile under the rules in that program. AN-78's message now gives the field change too: "... write `with thread_shared game`, and, since `add_point` changes `score`, declare `score: Atomic[i32]` and write `this.score.fetch_add(1)` in `add_point`" |
| G-10: untyped lambda parameters `fn(ref mut ui, dx)` escaped the comma rule | R3 | the comma rule covers every list of names declared without types, untyped lambda parameters included (E0116), and `ref` may be written on any declared name, so the line to write exists: `fn(ref mut ui, ref dx)`, or types on both: `fn(ref mut ui: Ui, dx: float)` |
| G-11: AN-84's help, applied as given, left `i = 0` and `i += 1` | R18 | a help that replaces several lines names every one: "instead of `i = 0`, the `while` line and `i += 1`, write `for i in 0..idx.len():`" (E7016) |

## 5. The two contradictions in the model (`RUN3/24-faithfulness.md`)

| gap | settled in | how |
|---|---|---|
| gap 1: W2's walk tests the target's header on every hop (R11 as written), while AN-62 says no test is needed | R11 | one rule says when a read tests: reading a `set by` link is a plain load and a test for no link, with no header test, because its target lives while it holds the object (run 3's D7 rule 5); reading any other `weak` link tests the header; a copy of a `set by` link (`copied n.parent`) is an ordinary `weak` link. A walk up parent links therefore keeps views (`cur = p`, which R2 now allows) instead of copies, and pays no test and no count per step. That walk is the C of `diag_w2_hop_proved`: 1.176x of plain C, against 1.579x for the walk that tests every hop and 1.603x for safe Rust (`RUN3/26-timer.md` lines 20 to 22). Area D's AN-62 must be rewritten to walk with views (section 11) |
| gap 2: one box per captured variable (D1's text) against the handler's state block (23's C) | R12 | one rule places a captured real variable: at its address when the function value provably ends first; else in the function value's own state, when that function value is its only user from the moment it is made (no other function value captures it, the function that made it does not use it afterwards, and the function value is never copied); else in a box. The placement never changes what the program does. W4 with that placement: 1.091x of plain C, against 1.255x with a box per variable and safe Rust's 1.358x (`RUN3/26-timer.md` lines 29, 31, 32). "Never copied" was added because a copy of a function value would otherwise split a variable both copies must share |

Found while doing this, and settled too: **gap 8** (`RUN3/24-faithfulness.md`: noalias on `mut self`
while a `weak` link may reach the same object). R4 now compiles a `ref mut` parameter, `ref mut this`
included, with noalias only where no link the function reads may reach the same object; R8 (b) keeps
views safe either way. What `ref mut` promises against links is listed for the owner (section 13).

## 6. Every change from run 3's rules

| run 3 | run 4 | what changed | why |
|---|---|---|---|
| section 0 (terms) | section 0 | added: place, "into X", link, `this`, function value, summary; `Copy` lists `sharedref mut` and `ref mut` as not `Copy` | the rewritten rules use them |
| R1 | R1 | unchanged in meaning; written shorter | — |
| R2 | R2 | rewritten whole. Kept: a name for a part is a read-only view; a number part is copied; `ref mut`, `copied`, `ref` before the part; a stored part is handed over; a `ref`/`ref mut` call result is a place (F-05). Changed: `=` on a name declared with `=` gives it a new value, a view included (run 3's R18-03 and F-06 refused it); parameters, loop, pattern and opened names keep their value; the weak-count paragraph (F-01) moves to R11 with no exception. Removed: every "Was:" note | G-08, G-01; one statement of `=` |
| R3 | R3 | `sharedref mut x` added before a value and in a type; the comma rule covers untyped lambda parameters; `ref` may be written on any declared name; `owned` on a class type has a code (E2114), a bare interface type E2173 | owner's decision 3; G-10 |
| R4 | R4 | the call says `ref mut` (E3028, E3027); "two different places" rewritten as one overlap rule (an index compare, else refused when compiling) that covers a `MutSpan` of elements owning memory without a run-time test, and counts a method's object as an argument; noalias only where no link may reach the same object; a view put in through `ref mut` may point into the call's other arguments, all of them, and a local view may be given (run 3's F-02 E3023 refusal of a view name is gone) | owner's decision 1; G-03; G-02 (with R6); gap 8; R18-16 |
| R5 | R5 | `this` with no receiver parameter; `ref mut this` and `owned this` after the parameters, alone (E2107); `static fn` (E2115); `init` (E2108); the call rule (R18-04) kept; F-04 (a), the lambda writer's `changes`, removed; a change through `this` with no clause is E2105 (run 3's clash 8), not E3023; `virtual fn on_update(dt: float) ref mut this:`; an override that says more is E2113 | owner's decision 2; G-06 |
| R6 | R6 | "into an argument" means into everything it owns; a generator method's result views its object, however it walks; `from` narrows it; run 3's F-09 rule that a `for` generator must write `from log` (E3062) is removed | G-02 (both halves) |
| R7 | R7 | three kinds of kept view by word: `ref T`, `ref mut T` (alone), `sharedref mut T` (several holders); `sharedref` needs `mut` (E0114) | owner's decision 3; R18-22 |
| R8 | R8 | (b) covers every view a call gives and `sharedref mut` links; (c) covers calls that may run a function value; (d), (e) have codes (E3063); new (f): a `ref mut` name is alone (E3021, E3022). (f) narrows run 3's A-2 last bullet: a change or read of an element through another name while a `ref mut` name of it is still used is refused, unless the two are proved different | owner's decision 3 (one meaning of `ref mut`) |
| R9 | R9 | "kept `ref mut` views may name one object" replaced by `sharedref mut`; a `sharedref mut` target on another thread follows the `thread_shared` rule | owner's decision 3 |
| R10 | R10 | point 1 lists the places that own an object (a write to a `weak T` place no longer matters, since every link is counted); point 2: the region lasts as long as the view. Run 3 put it "around the loop", so a view opened before the loop and used after it could read an object the loop set aside and the loop's end freed (found while rewriting; no proof was run, the reading is from the text) | G-01; found here |
| R11 | R11 | every `weak` link counted, no exception; reading a `set by` link has no header test; a copy of one is an ordinary `weak` link; walks keep views; `set by` misuse has a code (E2109) | G-01; gap 1 |
| R12 | R12 | where a real variable lives (address, own state, box); D3's rule for a call that may run a function value rewritten to cover every such call (callee summaries included) and every change through captured links; F-07's "the holder is not an argument" kept, its use to restore AN-60's `for` form withdrawn where a handler can reach the Ui through a link; `once fn` and E3026 stated | gap 2; G-04; G-05 |
| — | R13 (new) | the opening form for an `Option` or a `weak` link | owner's decision 4 |
| R13 | R14 | the conditional cast is opened with R13's form | owner's decision 4 |
| R14 | R15 | the generator text of F-09 moves to R6; otherwise unchanged | G-02 |
| R15 | R16 | unchanged | — |
| R16 | R17 | "whether a function may run a function value it is given or reaches" added to the facts recorded in the interface file | G-04 |
| R17 | R18 | the line a message gives compiles in its program; a help that replaces several lines names them all; a message says only what the compiler knows | G-09, G-11, G-07 |
| 1.2 (what runs) | section 2 | the two-compare overlap test for a `MutSpan` of non-`Copy` elements is gone (compile time now); a `weak` link is always counted; the `set by` read has no header test (W2's 1.176x); the capture placement (W4's 1.091x); a `sharedref mut` link costs nothing | G-01, G-03, gaps 1 and 2 |
| 1.3 (what stays refused) | section 3 | every row has its code and its message; rows added for every new refusal | the duty's error-message rule |
| 8 (codes) | section 4 | section 8 below | — |

## 7. Word changes

`RUN4/vocabulary.md` section 2 lists them with their reasons. In short: `self` and `mut self` removed;
`this` new; `owned this`, `ref mut this` after the parameters; `static fn` (a new position of `static`);
`ref mut` at a call and with one meaning everywhere; `sharedref` new (`sharedref mut T`); `if`/`while`
opening form and `Some` kept out of their headers; `ref` on any declared name; `weak` always counted;
`set by` read with no header test; `MutSpan` overlap rule; `from` optional on a generator; section 3.8
added (the library names the rules and the 84 answers use: run 3's 26 from `combined-v4.md` section 1.4
and the rest found by `scan_words.py`). Every word kept from run 3 keeps its row, with run 3's history
marks taken out and run 3's rule numbers R13 to R17 renumbered to R14 to R18 (`build_vocab.py` does
both, so no carried row was retyped).

## 8. Error-code changes

| code | change | reason |
|---|---|---|
| E0114 | **new**: `sharedref` not followed by `mut` | R7 |
| E0115 | **new**: `Some(...)` in an `if` or `while` header | R13 |
| E0116 | **new**: the comma rule (run 3 gave this refusal no code) | R3, G-10 |
| E0117 | **new**: `ref` or `ref mut` in a parameter's type (run 3 gave this refusal no code) | R4 |
| E2106 | **new**: an opening that reuses a name already in use | R13 |
| E2107 | **new**: `ref mut this` or `owned this` beside `reads`, `changes` or each other | R5 |
| E2108 | **new**: `ref mut this` or `owned this` on `init` (run 3's `init(mut self)` refusal had no code) | R5 |
| E2109 | **new**: a `set by` link assigned, or its holder lent as a changeable list (run 3: no code) | R11 |
| E2112 | **new**: `virtual` in a class that cannot be subclassed (today the warning W2111; run 3: no code) | AN-70 |
| E2113 | **new**: an override whose first line says more than the virtual's (run 3: no code) | R5 |
| E2114 | **new**: `owned` on a class type (run 3: no code) | R3 |
| E2115 | **new**: `static fn` outside a type body | R5 |
| E3028 | **new**: a call's word does not match its parameter | R4, owner's decision 1 |
| E2036 | **listed**, today's code and meaning ("this pattern always matches"), now the refusal of an opening whose right side always holds a value | R13 |
| E3021 | **listed**, today's meaning ("a shared and a mutable borrow overlap"): an argument that may reach into a changeable one; a `ref mut` name's place read through another name. Run 3 had moved its new meaning to E3023 and left the code unused | R4, R8 (f) |
| E3022 | **listed**, today's meaning ("two mutable borrows of the same place"): two changeable arguments of one place (run 3's AN-03 refusal had no code); a `ref mut` name's place changed through another name | R4, R8 (f) |
| E3026 | **listed**, today's meaning: a function value that may outlive a place it uses (run 3's AN-55 and AN-57 refusals had no code) | R12 |
| E3027 | **listed**, today's meaning ("a `mut` argument is not a mutable place"): `ref mut` before an argument the function may not change | R4 |
| E0113 | meaning narrowed: `mut` alone on a name; run 3's exception for `mut self` is gone with `self`, and `mut this` is included | owner's decision 2 |
| E1010 | covers `self` (gone) and `this` where there is no object | owner's decision 2 |
| E2103 | `this` for `self` | owner's decision 2 |
| E2105 | run 3's F-04 (a) part (a lambda's change counted for the function that writes it) removed; a change through `this` with no clause is E2105 | G-06; clash 8 |
| E2173 | the meaning widened from today's ("an interface used as a type where its implementers are values") to every bare interface name used as a type, as run 3's AN-66 already refused it | AN-66 |
| E3013 | covers a plain `once fn()` parameter and a function value's capture given away where it may run twice (run 3: no code) | R12 |
| E3020 | covers a list changed through any call that may run a function value (run 3: three kinds of call) and a removal inside a counting loop | G-04, G-05 |
| E3023 | no longer refuses `=` on a local holding a view (run 3's F-06) or a view name given to a `ref mut` parameter (run 3's F-02); covers `=` on a parameter, loop, pattern or opened name, a new view given to a `ref mut` name, and a `ref mut this` method called through a read-only name | G-08; owner's decision 2 |
| E3060 | a stored generator after its list changed, however it walks (run 3 excepted a position walk) | G-02 |
| E3062 | back to today's meaning only (a result outside its `from`); run 3's F-09 meaning (a `for` generator without `from`) retired | G-02 |
| E3063 | now also the code of a view out of a lock's `with` block (row 4's refusal had no code in run 3) | R8 (e) |
| E4030 | the message writes the call with `ref mut` | owner's decision 1 |
| E7015 | AN-78's message gives the field change too | G-09 |
| E7016 | the message names every line it replaces | G-11 |
| retired | E7001, E7003, E7005 (with `@sync`) and run 3's proposed E3045, as in run 3 | — |

Every new code was checked unused in the compiler's sources and the spec
(`target/redesign4/01-rule-writer/free_codes.txt`).

## 9. Today's Ember on the new spellings (`target/redesign4/bin/ember.exe check`)

Files under `target/redesign4/01-rule-writer/probes/`; output in `probes/check_output.txt`, pasted whole.

```text
=== p1_this.em
error[E0110]: `bump` is declared without a body
 --> p1_this.em:4:8
  |
4 |     fn bump() ref mut this:
  |        ^^^^ a function here needs a body
  |
  = help: add ':' and a body

error[E0100]: expected the end of the line
 --> p1_this.em:4:15
  |
4 |     fn bump() ref mut this:
  |               ^^^ found `ref`
  |

error[E0100]: expected a field or method
 --> p1_this.em:5:9
  |
5 |         this.n += 1
  |         ^ found an indented block
  |

error[E2072]: `Counter` has no method named `bump`
 --> p1_this.em:9:7
  |
9 |     c.bump()
  |       ^^^^
  |

error: could not compile due to 4 error(s)
exit=1
=== p2_call_ref_mut.em
exit=0
=== p3_sharedref.em
error[E1010]: cannot find type `sharedref` in this scope
 --> p3_sharedref.em:5:13
  |
5 |     target: sharedref mut Player
  |             ^^^^^^^^^
  |

error[E0100]: expected the end of the line
 --> p3_sharedref.em:5:23
  |
5 |     target: sharedref mut Player
  |                       ^^^ found `mut`
  |

error[E1010]: cannot find `sharedref` in this scope
 --> p3_sharedref.em:9:24
  |
9 |     w = Watcher(target=sharedref mut p)
  |                        ^^^^^^^^^
  |

error[E0100]: expected `)`
 --> p3_sharedref.em:9:34
  |
9 |     w = Watcher(target=sharedref mut p)
  |                                  ^^^ found `mut`
  |

error[E0100]: expected the end of the line
 --> p3_sharedref.em:9:34
  |
9 |     w = Watcher(target=sharedref mut p)
  |                                  ^^^ found `mut`
  |

error: could not compile due to 5 error(s)
exit=1
=== p4_if_open.em
error[E2036]: this pattern always matches
 --> p4_if_open.em:3:8
  |
3 |     if v = m:
  |        ^^^^^
  |
  = help: did you mean `v == m`?
  = note: to declare `v`, write `v = m` on the line before

error: could not compile due to 1 error(s)
exit=1
=== p5_static_fn.em
error[E0100]: expected a field or method
 --> p5_static_fn.em:4:5
  |
4 |     static fn make(n: int) -> Pool:
  |     ^^^^^^ found `static`
  |

error[E0100]: expected a field or method
 --> p5_static_fn.em:5:9
  |
5 |         return Pool(size=n)
  |         ^ found an indented block
  |

error[E2020]: `Pool` has no associated function `make`
 --> p5_static_fn.em:8:9
  |
8 |     p = Pool.make(3)
  |         ^^^^^^^^^^^^
  |

error: could not compile due to 3 error(s)
exit=1
=== p6_ref_mut_param.em
error[E0100]: expected a name
 --> p6_ref_mut_param.em:4:9
  |
4 | fn bump(ref mut c: Counter):
  |         ^^^ found `ref`
  |

error[E0100]: expected `:`
 --> p6_ref_mut_param.em:4:9
  |
4 | fn bump(ref mut c: Counter):
  |         ^^^ found `ref`
  |

error[E1010]: cannot find type `c` in this scope
 --> p6_ref_mut_param.em:4:17
  |
4 | fn bump(ref mut c: Counter):
  |                 ^
  |

error[E0100]: expected `)`
 --> p6_ref_mut_param.em:4:18
  |
4 | fn bump(ref mut c: Counter):
  |                  ^ found `:`
  |

error[E1010]: cannot find `Counter` in this scope
 --> p6_ref_mut_param.em:4:20
  |
4 | fn bump(ref mut c: Counter):
  |                    ^^^^^^^
  |

error[E0100]: expected the end of the line
 --> p6_ref_mut_param.em:4:27
  |
4 | fn bump(ref mut c: Counter):
  |                           ^ found `)`
  |

error[E0100]: expected an expression
 --> p6_ref_mut_param.em:5:5
  |
5 |     c.n += 1
  |     ^ found an indented block
  |

error[E1030]: `main` is already declared in this module
 --> p6_ref_mut_param.em:5:5
  |
5 |     c.n += 1
  |     ^
  |

error[E0100]: expected the end of the line
 --> p6_ref_mut_param.em:5:5
  |
5 |     c.n += 1
  |     ^ found `c`
  |

error: could not compile due to 9 error(s)
exit=1
```

What it shows: `this`, `ref mut this`, `sharedref` and `static fn` are not words today (p1, p3, p5);
`bump(ref mut a)` is accepted today at a call to today's `mut c: Counter` parameter (p2), while the
parameter spelling `ref mut c: Counter` is not (p6, run 3's known C-1 change); `if v = m:` on an
`Option` is E2036 today (p4), the code R13 keeps for an opening whose right side always holds a value.

## 10. Run 3's rulings, clashes and open points that still stand

**Word rulings** (`RUN3/combined-v4.md` section 2): all stand. The four new words of the areas
(`Commands.create()`, `from static`, `guarded by`, `Lock`), 07's planned words as the areas used them
(`thread_shared`, `weak`, `set by`, `@cached`, `@fp(allow_fma)`, the capture words, `changes` on a free
function, `from` after a field's type, `owned T` on a foreign type), and the removals (`RuntimeChecked`
and its guards, `@thread_shared`, `get` for `upgrade`, `Foreign[T]`); `ThreadShared` and `Send` kept and
inferred; `Box[T, A]` still open.

**Clashes** (`RUN3/combined-v4.md` section 3):

| clash | status |
|---|---|
| 1 (what `x = <part>` binds) | stands: a part is a read-only view (R2); amended: a local declared with `=` is given a new value by `=`, a view included (G-08) |
| 2 (positions and addresses) | stands (R8 (a), (b)); a `ref mut` position is now alone (R8 (f)) |
| 3 (two changeable kept views of one object) | **replaced**: kept `ref mut` views are alone; several changeable holders write `sharedref mut` (owner's decision 3) |
| 4 (`get_mut[T]`'s first line, the downcast) | stands, written with `this` (`fn get_mut[T](i: int) changes pools -> ref mut T:`) and the opening form |
| 5 (`RuntimeChecked` removed) | stands |
| 6 (`@cached` across threads) | stands |
| 7 (a change through a plain name reaching a `shared` object) | stands (R5) |
| 8 (E2105 against E3023) | stands; a change through `this` with no clause is E2105 |
| 9 (no word at the call) | **reversed** by the owner (decision 1) |
| 10 (keeping an object alive) | stands (R10) |
| 11 (a map key written through a `StrView`) | stands |
| 12 (a `while` loop that changes its list) | stands (R8 (c)) |
| 13 (inferred facts at a package's edge) | stands (R17), with one more recorded fact |
| 14 (`@thread_shared` removed) | stands |
| 15 (`class` with one owner) | stands |
| 16 (`worker` says `changes RESULTS`) | stands |

**Open points** (`RUN3/combined-v4.md` section 9):

| point | status |
|---|---|
| 1, 6 (a word at the call) | settled by the owner: `bump(ref mut a)` |
| 2 (`Box[T, A]`) | stands, for the owner |
| 3 (`=` hands a whole name over and views a part) | stands: the agreed rows 8 and 9 do both |
| 4, 8 (`ref mut` on a parameter and on a kept view) | settled by the owner: `sharedref mut` |
| 5 (AN-60's `while` dispatch loop) | stands (R8 (c)) |
| 7 (`mut self`, `static mut`) | `mut self` settled by the owner (`ref mut this`); `static mut` kept, for the owner (section 13) |
| 9 (a word for what a function value changes) | stands, for the owner; G-07 adds to its reason |
| 10 (a `weak` link that `=` copies) | stands: a link is copied with `copied`, and every copy is counted |
| 11 (`=` never re-points a view) | settled the other way for locals (G-08): a local declared with `=` may be given another view; parameters, loop, pattern and opened names may not |
| 12 (a local view a call may point elsewhere) | settled: a local view may be given to a `ref mut` parameter (R4) |
| 13 (`r = first_even(...)`) | stands: `r = ref mut first_even(...)` |

## 11. Answers the new rules touch (for agents 02 to 06)

A pointer, not a ruling: each area updater decides its answers. Counts of the constructs in each
answer's code blocks (OLD and NEW together) are in `target/redesign4/01-rule-writer/touched.txt`
(`touched.py`).

| change | rule | answers |
|---|---|---|
| `self` becomes `this`; `mut self` becomes `ref mut this`; `owned self` becomes `owned this` | R5 | AN-10, 17, 18, 19, 21, 22, 24, 25, 26, 31, 32, 33, 37, 38, 42, 48, 49, 53, 60, 61, 62, 63, 67, 70, 74, 75, 77, 78, 80 |
| `ref mut` written at calls to `ref mut` parameters | R4 | every NEW call to a `ref mut` parameter: at least AN-01 to AN-06, 15, 18, 20, 23, 27, 48, 51, 54, 57, 60, 67, 74, and the R4 messages of AN-03, AN-18, AN-81 |
| the opening form for `if Some(...)` and `while Some(...)` | R13 | AN-60 (its `with weak ui` line), AN-62, AN-63, AN-65, AN-67, AN-68 (its NEW becomes the OLD's own `if c = cast s if Circle:`, and its refusal goes) |
| kept `ref mut` that names one object becomes `sharedref mut` | R7 | AN-52 (b) |
| a `ref mut` name is alone | R8 (f) | AN-01, AN-02 (a write or read through another name of the element while `boss` is used); AN-48's refused struct form gets "write `target: sharedref mut Player`" |
| the parent walk keeps views, not copies | R11 | AN-62 (`cur = n`, `while p = cur.parent:`, `cur = p`) |
| what a generator views | R6 | AN-77 (`from log` optional; no "walk by position" way out in the messages) |
| `=` gives a local another view | R2 | AN-11, AN-22, AN-28 (the E3023 refusals of re-pointing a local view go) |
| what a call that may run a function value changes | R12 | AN-53, AN-60 (the `for` form's "second way out" holds only where no handler can reach the Ui through a link), AN-76 (no `changes LOG` on a lambda's writer; G-07's message) |
| the capture placement | R12 | AN-53, AN-60 |
| a `MutSpan` of elements that own memory | R4 | AN-11, AN-14 (numbers: unchanged) |
| a change through `this` with no clause is E2105 | R5 | AN-19, AN-20 |
| `init`, `this =`, `virtual`, override | R5 | AN-32 (E2108), AN-50 (E2103), AN-70 (E2112, E2113, `ref mut this` on the virtual) |
| messages | R18 | AN-78 (G-09), AN-84 (G-11), AN-81 (E4030 with `ref mut`) |

## 12. Files

- `C:\Users\ism19\Code\ember\tasks\96\redesign\run4\rules.md`, `vocabulary.md`, `changes.md`.
- Under `C:\Users\ism19\Code\ember\target\redesign4\01-rule-writer\`: `build_vocab.py` (builds the
  vocabulary from run 3's), `vocab_head.md`, `vocab_tail.md` (its hand-written parts), `scan_words.py`
  and `scan_words.txt` (names in the answers' code with no vocabulary row), `touched.py` and
  `touched.txt` (section 11), `list_self_rows.py`, `spec_self_lines.txt` (the spec lines that use `self`),
  `free_codes.txt` (the new codes unused), `probes\` (the six `ember check` probes and
  `check_output.txt`).

## 13. For the owner

Questions only the owner can decide, one at a time, each with what this run did meanwhile.

1. **G-01 is made safe, not refused.** The brief asks that the five programs be refused. G-01 is a
   correct program (a link that outlives its node and is asked whether the node lives); R11 counts every
   `weak` link, so it now answers "gone" safely, as Python and Rust do. Refusing it would need a special
   rule that protects nothing more. Is "accepted and safe" the right outcome for it?
2. **The spellings settled here, for your verdict:** `ref mut this` after the parameters, alone
   (`fn reset() ref mut this:`; `changes log` for a method that changes only `log`); `sharedref mut`
   (chosen over `sref mut`, whose `s` is not a word); `if ref mut me = ui:` for opening a link or an
   `Option` (with `if p = cam.target:` and `while p = a:`); and `static fn` for a function in a type
   body that has no object, which your "methods do not list their object" makes necessary.
3. **`ref mut` is alone everywhere**, by your meaning "only this name may touch it until it is done".
   That narrows one thing run 3 allowed: while `boss = ref mut enemies[0]` is still used,
   `enemies[0].hp = 1` (or reading `enemies[0]`) through another name is refused, as in Rust; a plain
   `boss = enemies[0]` still sees such writes. Is that what you meant by "touch"?
4. **What `ref mut` promises against links** (`RUN3/24-faithfulness.md` gap 8). An object that `weak`,
   `shared` or `sharedref mut` links reach can also be reached through them: a turret whose `target`
   is itself, in a method with `ref mut this`, reaches itself through `this.target`. The rules keep
   every view safe and simply do not promise the C compiler "no overlap" there. A stricter rule is
   possible: refuse a change through a link that may reach the `ref mut` object unless the line proves
   otherwise (`if t is not this:`). Which do you want?
5. **What a function value changes** (run 3's open point 9, with G-07). Without a word for it, a call
   that may run a handler is taken to make every change any handler of that type makes, so a loop over
   `LOG` refuses `b.click()` even when `b` holds no handler that touches `LOG`; the message says so. A
   word such as `fn() changes LOG` would make it exact. Add one?
6. **`static mut` keeps a bare `mut`** (R18-14's second half). With `mut self` gone it is the one place
   `mut` stands without `ref` or `sharedref`; `static` says whose variable it is, so it passes the test.
   Keep it?
7. **`Box[T, A]` with an allocator** has no spelling (run 3's open point 2): no annoyance needs it.
