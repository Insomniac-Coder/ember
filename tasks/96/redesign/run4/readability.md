# Readability check of run 4, phase 1 (agent 07)

Agent 07 (readability checker), run 4, phase 1. This file checks every NEW block of the five area files
against the owner's tests and verdicts. It fixes nothing. `RUN4` means `tasks/96/redesign/run4/`;
"R1" .. "R18" are the rules of `RUN4/rules.md`; "row N" is the owner's agreed row N in `inputs/JOB.md`.

Power: `PowerLineStatus` printed `Online` at the start of this work and again before writing this file.
No compiler change, no git command, no Ember program run, no `ember check` (this file makes no claim
about what today's Ember accepts).

## 0. Terms

- **NEW block**: code an area file proposes under the run-4 rules: part 3 ("NEW") of each answer, and
  the refused programs written out in part 6 ("Still refused"). OLD code and "means exactly" code are
  today's Ember, carried from run 3, and are not judged here.
- **message**: the text of a refusal (an error or a warning) that a NEW block or a rule gives.
- **the owner's test**: for every name a line declares, the word on the line that says what it is (the
  owner's own thing, a copy, or handed over) and the word that says whether it may change it. A name
  with no word passes only where the default is stated and cannot surprise a reader: it never changes
  anything, never aliases, never copies silently (brief, "The owner's test").
- **aliases**: two names reach one place, so a change through one is seen through the other.
- **view**: a name that points into something someone else owns (`RUN4/rules.md` section 0).
- **the error-message rule**: a message names what was refused in the program's own names, says why
  in one line, and gives the exact line to write, a line that compiles in that program (brief; R18).
- **the four verdicts** (owner, 2026-10-25): no `self` (the object is `this`); a call that changes an
  argument says `ref mut` (`bump(ref mut a)`); a new word for a changeable link other fields may share
  (`sharedref mut` or `sref mut`); a major rework of `if Some(ref mut me) = ui:`.
- **failure**: a place that does not meet one of these tests. **Concern**: something that meets the
  letter of a test but that the owner may judge differently; listed apart, not counted as a failure.

## 1. What was checked, and how

- Inputs read in full: `RUN4/rules.md`, `RUN4/vocabulary.md`, `RUN4/area-A.md` .. `area-E.md`,
  `inputs/JOB.md`, and sections 2, 4, 11 and 13 of `RUN4/changes.md` (to know what is already raised
  for the owner).
- **89 NEW code blocks** in 83 answers (AN-16's NEW is message text only; it was checked as messages),
  every owner's-test table, and every message in part 6 of the 84 answers and in `RUN4/rules.md`
  section 3.
- A script pulls every `ember` code block out of the five area files and sorts it OLD or NEW by the
  part of the answer it sits in, then searches the NEW blocks for the forms the verdicts remove
  (`self`, `if Some(` / `while Some(`, `mut` alone on a name, `sref`, `Box[`, `RefCell`, `Weak[`,
  `.upgrade(`, `.clone()`, `owned fn`), for names that break the comma rule (R3), and for calls to a
  function whose parameter says `ref mut` but whose argument does not, or the other way round:
  `target/redesign4/07-readability/extract.py`, output `blocks.txt` (every block, labelled) and
  `mech_checks.txt`. **Result: 0 hits in the 89 NEW blocks.** A second pass over the prose (inline
  code outside blocks) found these forms only where the text describes today's spelling or a refused
  program. Everything else was read by hand.

## 2. Results at a glance

| test | result |
|---|---|
| no `self` | **passes**: no NEW block and no message writes `self` except to refuse it (E1010) |
| `ref mut` at calls | every call to a `ref mut` parameter in the NEW blocks says `ref mut`; **fails** where R4 lets an argument of type `MutSpan`, `Mutex`, `RwLock`, `Atomic` or a channel end be changed by a call with no word (F-02) |
| the new link word | **passes**: `sharedref mut` (one of the owner's two spellings) in AN-24, AN-39, AN-48 (b), AN-52 (b) and the messages that lead to it; `sref` appears nowhere |
| no `if Some(...)` form | **passes by the letter** (no `if Some(` or `while Some(` in any NEW block); **fails** the owner's request for a major rework that says in plain words "if the link still points at something, call it `me`" (F-01) |
| agreed rows 3 to 9 | **all exactly as agreed** (section 4) |
| the owner's test on every declared name | two rule-level failures, each showing in many answers (F-03, F-04); every other name has its word or a default that never changes, aliases or copies |
| the error-message rule | messages whose line does not compile in its program (F-05, F-06); messages with no exact line (F-07, F-08); one name clash (F-09) |

## 3. Failures

Most severe first. Each with the blocks it is in, the test it fails, and why.

### F-01. The opening form is run 3's line without `Some(...)`, not the rework the owner asked for

- **Blocks:** R13 and its uses: AN-60 (`watch`: `if ref mut me = ui:`), AN-62 (`if n = leaf:`,
  `while p = cur.parent:`), AN-63 (`if p = n.parent:`), AN-65 (`if p = cam.target:`), AN-67
  (`if ref mut pool = cast this.pools[k] if Pool[T]:`), AN-68 (`if c = cast s if Circle:`).
- **Test:** verdict 4. The owner (JOB, "Readability verdicts"): "`if Some(ref mut me) = ui:` needs a
  major rework ... A replacement must say in plain words 'if the link still points at something, call
  it `me`' and pass the owner's test."
- **Why:** the new line is the old line with the wrapper `Some( ... )` taken off. It passes the owner's
  test for `me` (`ref mut` says what it is and that it may change it), but nothing on the line says
  "if the link still points at something": that meaning comes only from the type of the right side
  (an `Option` or a `weak` link), which the reader must know from elsewhere. Every NEW block has to add
  the plain words as a comment (`# the Ui still lives: call it me`, `# the leaf still lives: call it
  n`, `# the player still lives: call it p`), which shows the code does not say it. `=` in an `if`
  header also reads as an assignment, or as a typo for `==` (Python refuses `=` there); R13's two
  refusals (E2036 when the right side always holds a value, E2106 when the name is not new) stop the
  mistakes but do not make the line say what it does. With a downcast the line carries two `if`s, the
  second meaning "is a" (`if c = cast s if Circle:`). The rule writer lists the spelling for the
  owner's verdict (`RUN4/changes.md` section 13, question 2); this check says it does not yet meet the
  owner's own description of what the replacement must do.

### F-02. A call may change an argument with no word when the argument's type is `MutSpan`, `Mutex`, `RwLock`, `Atomic` or a channel end

- **Blocks:** R4 ("An argument whose type is made to be changed through any name ... needs no word:
  its type says it"); AN-11 (`integrate(p.as_mut_span(), v.as_span(), 0.5)`), AN-14
  (`blend(...)`, `assert_disjoint(dst, src)`), AN-30 (`top(scores)` with `scores: Mutex[...]`), AN-84
  (`place(a.as_mut_span(), order.as_span())`); `RUN4/changes.md` section 2.1 "Also".
- **Test:** verdict 1. The owner: "a call that changes an argument says so (`bump(ref mut a)`) ... A
  method called on an object (`a.reset()`) is not marked." That is the one exception the owner gave.
- **Why:** R4 adds a second exception the owner did not give. Where the span is made in the call
  (`p.as_mut_span()`) the word `mut` is visible, but with a span or a lock held in a name
  (`ps = p.as_mut_span()` then `integrate(ps, vs, dt)`; `top(scores)`) the call changes the caller's
  elements, or the locked list, and the line does not say so. This widens an owner answer without asking
  (the owner's rule: never widen an owner answer); it is not in `RUN4/changes.md` section 13.

### F-03. A `shared` owner may change the value with no word saying it may, while `sharedref` must be followed by `mut`

- **Blocks:** R3, R9 ("Any owner of a `shared` value ... may change it, written as any change is
  written"); AN-34 (`two_owners`: `other = shared items`, `other.push(4)`, `items.len()` is 4), AN-36
  (second form: `root.children.push(shared child)`, `child.value = 5` seen in the tree), AN-39
  (`a.mesh.verts.push(1.0)`, "b sees four vertices"), AN-44 (`p2 = shared player`, `p2.hp = 5`), AN-45
  (`c = shared a`), AN-48 (a) (`target: shared Player`), AN-51, AN-52 (a), AN-78 (`game.add_point()`
  through `with shared game`). In each owner's-test table the "may it change it" column for the
  `shared` name says "none (default: any owner may change it, R9)".
- **Test:** the owner's test. A name with no word passes only where its default never changes
  anything and never aliases; here the default is "may change, and every other owner sees it".
- **Why:** `shared` says what the name is (one of several owners); no word says it may change. The
  owner's rule for the non-owning link shows the expected shape: `sharedref` must be followed by `mut`
  (E0114), and the owner checked "both holders may change it" against the `mut`. The same change
  through a `shared` owner carries no `mut`. A second effect: after `other = shared items`, the first
  name `items`, declared as the function's own list with no word, becomes one of two owners and sees
  changes made through `other`; its own line never says so. Row 9 agrees the spelling
  `snapshot = shared items` for "two owners of one list"; it does not say whether those owners change
  the list.

### F-04. A plain (read-only) name for a part sees changes made through another name

- **Blocks:** R2 and R8 (a) ("A read-only position sees, at its next use, plain data written through
  another name"); AN-13 (`p = ps.get(0).unwrap()`, then `ps[0].life -= 0.5`, then `println(p.life)`
  prints 1.5, the new value), and every plain `boss = enemies[0]` the rules allow beside a write to
  `enemies[0]`.
- **Test:** the owner's test, "never aliases". `p` has no word; its stated default is "a read-only
  name of particle 0".
- **Why:** the default is not a snapshot: `p` and `ps[0]` are two names for one place, and a change
  through one shows through the other while `p` is used. Nothing on `p`'s line says it follows later
  changes, and Rust refuses the same program (a shared borrow stops the write). The rule writer's
  question 3 (`RUN4/changes.md` section 13) mentions that "a plain `boss = enemies[0]` still sees such
  writes", but asks only about `ref mut`; it does not ask whether a plain name may alias.

### F-05. Messages whose line does not compile, because it uses a name the program does not have

R18: "The line a message gives compiles under these rules in that program." Each of these gives a line
that needs a declaration the message does not give.

| where | message (shortened) | what is missing |
|---|---|---|
| `RUN4/rules.md` E3020, first message; AN-04 E3020 (push) | "collect the new enemies in `spawned` ... `enemies.extend(spawned)` after the loop" | `spawned: Array[Enemy] = []` |
| `RUN4/rules.md` E3020, second message; AN-53, AN-58 E3020 | "call it after the loop, or collect what to add and push it after the loop" | no line at all for the second way |
| AN-05, both E3020 messages | "`dead.push(i)` here, then `while dead.len() > 0: ...`" | `dead: Array[int] = []` |
| AN-17, E3020 | "collect the new players in `joined` and write `this.players.extend(joined)`" | `joined: Array[Player] = []` |
| AN-42, E3020 | "collect what to add in a list, then call `reg.emplace(...)` for each once the loop ends" | the list and the loop are not written |
| AN-48, E3020 | "collect the new items in a list, then `cam.target.inventory.extend(new_items)`" | `new_items: Array[String] = []` |
| `RUN4/rules.md` E7007; AN-74 E7007 | "guard it: `total: int guarded by lock`, and change it inside `with this.lock:`" | the field `lock: Lock = Lock()` |
| `RUN4/rules.md` E7010; AN-74 E7010 | "change it under its lock (`hp: int guarded by lock`)" | the field `lock: Lock = Lock()` in `Player` |
| `RUN4/rules.md` E7012; AN-76, AN-84 E7012 | "`static mut FRAME: int guarded by FRAME_LOCK` and change it inside `with FRAME_LOCK:`" | `static FRAME_LOCK: Lock = Lock()` |

AN-72's E7007 shows the form that passes: it gives `parts: Array[int] = []` "before the `with` block".

### F-06. E3026's advice names a type and a name the program does not have

- **Blocks:** `RUN4/rules.md` E3026; AN-53 part 6; AN-57 NEW (the message is AN-57's NEW).
- **Test:** the error-message rule (R18: the line compiles in that program).
- **Why:** "keep the total where the handler can own a share of it: `count = shared Counter()` in the
  caller, and `b.on_click(fn() with shared count: count.n += 1)`". AN-57's program has no `Counter`
  type, and `register` (which writes the lambda) has no `count`: following the advice also needs a
  class `Counter` with a field `n`, and `register`'s first line changed to take `count`
  (`fn register(ref mut b: Button, count: shared Counter):`). The message gives neither.

### F-07. Messages with no exact line to write

| where | message (shortened) | what is missing |
|---|---|---|
| AN-11 part 6, E3063 | "put a view of what the caller gave there, or return a list of your own" | `RUN4/rules.md`'s own row gives the lines (`best = other`, `-> Array[int]`); AN-11's does not |
| AN-21 part 6, three `@cached` refusals | "keep a `Map` field of results by hand"; "remove `@cached`, or keep the result by hand"; "keep the result in a field `guarded by lock`" | no line for the hand-kept form |
| AN-24 part 6, E3063 | "... or keep `reg` alive as long as `vs`" | no line |
| AN-31 part 6, E2100 | "set `this.renderer` before this line" | the line `this.renderer = Renderer(width=width)` |
| AN-16 NEW (the `collect` message) | "write `words: Array[StrView] = ...`" | `...` stands for the expression |
| AN-77 part 6, E2221 | "keep the value instead (`m = n`), or its position" | no line for "its position" |
| AN-56 part 6, E2020 | "`each_name` lends the name and keeps it; to keep your own ..." | the "why" contradicts itself ("lends" and "keeps"), so it is not one clear line |

### F-08. A message tells the programmer to edit another package

- **Block:** AN-17 part 6, E3021 (the call from another package, where `note` is `pub` with no `reads`).
- **Test:** the error-message rule (the exact line to write instead).
- **Why:** "if it does not, say what it reads: `fn note(msg: String) reads frame changes log:`". The
  programmer who gets this error is in the calling package and cannot edit `note`'s first line; the
  line is for `note`'s author. A line the caller can write (call `note` after the loop) is not given.

### F-09. A refused example names its struct `Handle`

- **Block:** AN-24 part 6 (`struct Handle:` holding `reg: ref mut Registry`; the message says "write
  `reg: sharedref mut Registry` in `Handle`").
- **Test:** keywords and names mean one thing everywhere (brief); the vocabulary's `Handle` row: the
  word "handle" is used only for `Handle[T]`, `JoinHandle` and `@ffi(handle)`, never for an object or
  a link, "in every message and in the model's own text".
- **Why:** the example and its message use "Handle" for a struct holding a link, the meaning AN-50
  removed. A reader of the message sees the library's `Handle[T]`. Any other name (`Pointer`, `Probe`)
  would do.

## 4. The agreed rows 3 to 9

| row | where | as agreed? |
|---|---|---|
| 3 | AN-17: `fn note(msg: String) changes log:`, checked against its body (E2105 messages) | yes; `self` dropped as the owner ruled, meaning unchanged |
| 4 | AN-30: `return ref g[0]` inside `with g = scores.lock():` refused, E3063, "this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block" | yes, word for word |
| 5 | AN-08: `word = line.to_lower().trim()` then `word.len()`; AN-09: returning the view refused with "return `String(...)` to hand back your own copy" inside the message | yes |
| 6 | AN-53: `b.on_click(fn(): score += 1)` changes `game`'s real `score`; `fn() with copied score:` makes the copy | yes |
| 7 | AN-31: `fn load_assets() reads renderer -> Array[String]:`, checked against its body and at the call in `init` | yes; `self` dropped as ruled |
| 8 | AN-37, AN-38: `State.Walking(ref mut path):`; plain `path` then `path.pop()` and `State.Walking(mut path):` both get "to change that list through `path`, write `State.Walking(ref mut path):`" | yes, word for word |
| 9 | AN-34: `snapshot = copied items` copies; `snapshot = items` hands over; E3040 names `copied` and `shared` | yes |

The vocabulary's table of agreed rows (`RUN4/vocabulary.md` section 1) matches as well.

## 5. Concerns (meet the tests' letter; for the owner's judgement, not counted as failures)

1. **`=` has four meanings by context.** `b = a` hands over (row 9); `rest = tail.trim()` and
   `cur = ref p` point a view elsewhere (R2, G-08); `r = 20` on a `ref mut` name writes the place it
   names (AN-28: `r = ref mut a` then `r = 20` writes `a`); `if p = x:` opens a link (R13). A reader of
   `r = 20` must look back to `r`'s declaring line to know `a` changes. The brief asks that a word mean
   the same everywhere; `=` is an operator, not a keyword, so this is listed as a concern.
2. **A function value that changes an object through a captured link has no word for it.** AN-60's
   `bump_ui = fn() with weak ui:` changes the Ui through `if ref mut me = ui:`; its type is `fn()`, and
   the call `h()` inside `Button.click` says nothing. This is G-05's shape, of which the owner said
   "ref mut would address the readability issue too". R12 states it as a limitation and the rule writer
   asks the owner for a word (`RUN4/changes.md` section 13, question 5).
3. **Calling a stored function value needs `changes` and `ref mut`.** R12 makes `fn click() changes
   handlers:` with `for ref mut h in this.handlers: h()` the only way to call handlers (AN-53, AN-58),
   and `fn depth_total() changes on_visit -> int:` for calling one stored callback (AN-62). The first
   line reads as "replaces `on_visit`" while the method only calls it. This is Rust's `FnMut` rule in
   words, so not a failure.
4. **`this` in `init` changes fields with no word.** The default ("`init` sets the fields of the object
   it builds") is stated and is how every constructor reads, so it passes; strictly it is a default that
   changes things.
5. **A virtual's first line says more than its body.** AN-70's `virtual fn on_update(dt: float) ref mut
   this: pass` says it changes the door while its body changes nothing (area D's note 5 asks whether
   E2105 applies to a virtual).

## 6. Found in passing (for the lead; not readability tests)

- **The areas treat names with no vocabulary row differently.** Area D rewrote AN-55 to avoid `.count()`
  (no row), and area C avoided `_`; area E kept `.count()` in AN-71 and AN-73 and lists it, with `read`,
  `add`, `*T`, `i16` and `user_data`, as "words needed". The duty says to use only words in
  `RUN4/vocabulary.md`.
- **Refusals with no code** are listed by areas A (3), B (8 kinds), D (4) and E; none is in
  `RUN4/rules.md` section 4.

## 7. For the owner

The lead raises these in chat, one at a time.

1. **The opening form** (F-01). You asked for a major rework of `if Some(ref mut me) = ui:` that says in
   plain words "if the link still points at something, call it `me`". The rules give
   `if ref mut me = ui:`: the same line without `Some(...)`. The test "does the link still point at
   something?" is not written; it comes from the type of `ui`, and the area files add the plain words as
   comments. Is this the rework you wanted, or should the line itself say it?
2. **Calls that change an argument through its type** (F-02). You allowed one call that changes an
   argument without `ref mut`: a method called on an object. The rules add a second: an argument of type
   `MutSpan`, `Mutex`, `RwLock`, `Atomic` or a channel end (`integrate(ps, vs, dt)` changes `ps`'s
   elements with no word). Should such calls say `ref mut` too?
3. **Changing through a `shared` owner** (F-03). `sharedref` must be followed by `mut` before a holder
   may change the target, but any `shared` owner changes the value with no `mut` (`other = shared
   items`, `other.push(4)`, and `items` sees it). Should a `shared` owner that changes the value say
   `mut` as well (for example `shared mut`)?
4. **A plain name that follows later changes** (F-04). `p = ps.get(0).unwrap()` with no word is a
   read-only name of particle 0, and it shows a change made through `ps[0]` afterwards (Rust refuses
   the write while `p` is used). Is a plain name allowed to follow such changes, or should it be a
   snapshot, or the write refused while it is used?

## 8. Files

- `C:\Users\ism19\Code\ember\tasks\96\redesign\run4\readability.md` (this file).
- `C:\Users\ism19\Code\ember\target\redesign4\07-readability\extract.py` (pulls and labels the code
  blocks, runs the searches), `blocks.txt` (every block with its label), `mech_checks.txt` (the
  searches' hits: none in NEW blocks), `notes.md` (working notes).
