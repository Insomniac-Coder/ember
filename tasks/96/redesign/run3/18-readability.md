# 18 readability attacker: report (run 3, phase 3)

Power: `PowerLineStatus` printed `Online` at the start and again before writing this report.
No compiler change, no git command, no Ember program run, no `ember check` needed (every finding is
about the NEW model text, which today's compiler cannot check; every claim about today's compiler
below points at output already pasted in `combined-v2.md`). Files written: this report, and under
`target/redesign3/18-readability/` the script `codes.sh` and its output `codes.txt` (every error code a
NEW refusal in `combined-v2.md` gives, with its answer and line).

## 0. Terms

- **the model**: `RUN3/combined-v2.md` (the combined memory model after agent 15's word fixes), with
  `RUN3/vocabulary-final.md`. Line numbers below are lines of `combined-v2.md` unless a file is named.
- **NEW block**: the proposed code of an answer (its point 3), and every line a refusal message tells
  the programmer to write.
- **the owner's test**: for every name in a line, the word in the line that says what it is (the
  owner's own thing, a copy, handed over) and the word that says whether it may change it; "none
  (default: ...)" only where the default never changes anything, never aliases, never copies silently.
- **the error-message rule**: every refusal names what was refused in the program's own names, says
  why in one line, and gives the exact line to write; a form that is accepted but does not do what its
  words mean elsewhere is refused with that advice.
- **plain name**: a name with no word on it (`x: T`, `for e in xs:`, `State.Walking(path):`).
- **view**: a name that points into something someone else owns (the model's own term, section 0).
- **one rule, two meanings**: a word or form that does one thing in one place and another elsewhere,
  so a reader cannot tell from the line which it is.
- **proven**: the failure is shown by two places of the model's own text that cannot both hold, or by
  a NEW line that the model's own rule refuses; **judgement**: the failure is my reading of the
  owner's test, given with the reason, for the owner to rule on.

## 1. What I did

1. Read the brief, `JOB.md`, my duty, agents 13, 14 and 15's reports, and all 10,718 lines of
   `combined-v2.md`: sections 0 to 9 (the rules R1 to R17, the clashes, the codes), every area's terms
   and rules, and all 84 answers, each NEW block line by line.
2. For every NEW block: applied the owner's test to every name (checking each answer's own test table
   against its rules and against the other answers); read every refusal against the error-message rule;
   compared the NEW code with its intent line and its "means exactly" translation; read it as a Python
   programmer would.
3. Looked up every word whose meaning I doubted in `vocabulary-final.md` (`mut`, `mut self`, `self`,
   `fn`, `once`, `Cell`, `Mutex`, `Atomic`, `MutSpan`, `.copied()`).
4. Listed every error code the NEW refusals give (`codes.sh` -> `codes.txt`, 64 refusals, 31 codes) and
   checked that one code means one thing.
5. Checked the agreed rows 3 to 9 word for word against `JOB.md` (section 3).

## 2. The failures

Most severe first. "Area" is the area whose answer the failure sits in (A to E), or "model" for the
combined rules.

| id | block | test it fails | kind | area |
|---|---|---|---|---|
| R18-01 | R5 against AN-11, AN-30, AN-20, AN-48 | "a plain name never changes anything" is false for some types; `ref mut` means two things | proven | model |
| R18-02 | AN-53 against AN-54 and D3 (agreed row 6) | the type `fn()` hides whether a call changes the function's own state; row 6's `with copied score` has no defined meaning inside AN-53's `Button` | proven | D |
| R18-03 | R1, R2, AN-28 | `=` does five things, chosen by what was on the right of an earlier line | proven | model |
| R18-04 | AN-17 against AN-76 rule 2 | `changes` means "this body and its calls" on a method, "this body only" on a free function | proven | B / E |
| R18-05 | AN-84 (E7016) against AN-74 and AN-84's own NEW | E7016's meaning names, as refused, the two loop forms the model accepts | proven | E |
| R18-06 | AN-59 `call_once(f: once fn())` | a type word acts as a hand-over with no mode on the name: the form 15 removed for `shared` | proven | D |
| R18-07 | AN-43 refusal | the message's line to write contradicts R4 after 15's change 1 | proven | C |
| R18-08 | AN-78 and AN-47 refusals for `owned fn` | two codes and three texts for one refusal; AN-78's names `score`, which the program does not have | proven | E / C |
| R18-09 | error codes (`codes.txt`) | one mistake, several codes; one code, several mistakes | proven | model |
| R18-10 | AN-77 (generators) | a `for` loop in a stored generator walks a list that may change, which the agreed limitation says a loop may not | proven | E |
| R18-11 | AN-70 after 15's `virtual` rule | the NEW `open class Door` declares a virtual no override may use to change the door | proven | D |
| R18-12 | AN-71, AN-34, AN-36, AN-39, AN-44 | a word on a later line turns an earlier plain local into a shared owner (and in AN-71 freezes it); the declaring line says "own, never aliases" | judgement | C / E |
| R18-13 | R1 against R2 (AN-62) | a `weak` link is copied when bound from a part and handed over when bound from a whole name | proven | model |
| R18-14 | `mut self`, `static mut` against `mut x` refused | `mut` alone still marks a changeable name in two places after it was refused on every other name for failing row 8's test | judgement | B |
| R18-15 | AN-15 `for ref mut pos, vel in q:` against AN-54's `with copied name, score` | one comma shape, two rules | proven | A / D |
| R18-16 | call sites: AN-03, AN-16, AN-27, AN-67 (clash 9, open point 1) | `bump(a)` changes `a` with no word in the line | judgement (open for the owner) | model |
| R18-17 | AN-05 NEW | the handlers run last-dead-first, and a position in `dead` goes stale if a handler removes another enemy | proven | A |
| R18-18 | AN-35 refusal of `v.push(template)` | the message's reason is the OLD signature's, not the NEW's | proven | C |
| R18-19 | AN-46, AN-81 refusals | messages name things that are not in the program (`keys`, `collide(xs[i], xs[j])`) | proven | C / E |
| R18-20 | a plain number parameter (AN-02 against R5) | whether `x += 1` on a plain parameter is accepted is stated two ways; no spelling is left for "a changeable copy of a parameter" | proven | model |
| R18-21 | AN-34 NEW block | as written, the block's third line uses a name its second line handed over | proven | C |
| R18-22 | `ref mut` on parameters against kept `ref mut` views (R4, R9; open point 4) | `ref mut` promises "no other name reaches it" in one place and not in the other | judgement (open for the owner) | model |

### R18-01. "A plain name never changes anything" is false for some types, and `ref mut` means two things

- R5 (line 137): "A plain name never changes anything: a plain parameter, a plain loop or pattern name,
  plain `self` without `changes`. That includes setting a `Cell` field ... and changing an object reached
  through a `shared` link." The brief allows a name with no word only on that promise.
- But a plain parameter changes the caller's data through four types:
  - `MutSpan`: AN-11's NEW `fn integrate(pos: MutSpan[float], ...)`: `pos[i] += vel[i] * dt` (line
    2505) writes the caller's elements through plain `pos`.
  - `Mutex`: AN-30's `fn top(scores: Mutex[...])` (line 4950): `with g = scores.lock():` gives `g`, which
    "may change: `lock()`: yes" (test row, line 4969); the OLD `clear(scores: Mutex[...])` with
    `g.clear()` stays accepted by the `Mutex` row of `vocabulary-final.md` ("a reference out of its guard"
    is the only refusal).
  - `Atomic`: AN-73's tasks call `total.fetch_add(n)` (line 9241); R9 says a `thread_shared` value is
    changed "only through `Atomic`, `Mutex`, ..." by owners that are otherwise never allowed to change it.
- And two types that do the same job are refused through a plain name: `Cell` (AN-20, line 3846:
  "`tick` sets `s.frame`, but `s` only reads the caller's clock; write `fn tick(ref mut s: Clock):`") and
  a `shared` link (R5, 15's change 1, AN-48's `ref mut b: shared Player`, line 6497).
- So whether a plain name may change what it points at depends on its type: yes for `MutSpan`,
  `Mutex`, `Atomic`; no for `Cell`, `shared T`, a class object. `Cell` and `Mutex` are the one-thread and
  the threads way of the same thing (a value changed through a read-only path), and get opposite rules.
- `ref mut` on such a parameter then means two things. On a view type it means "may point the caller's
  view elsewhere" (AN-11, line 2560: "To point the caller's view elsewhere, write `ref mut pos:
  MutSpan[float]`"; the AN-16 table, line 3298, says the same). On a `shared` link it is what lets the
  callee change the object (AN-48's `b`, test row line 6518: "`ref mut`: yes", with no word on whether
  `transfer` may also re-point the caller's link). A reader of `ref mut b: shared Player` cannot tell
  whether `b = shared other` inside `transfer` changes the caller's link.
- Why a programmer cares: the one rule the brief rests every default on cannot be applied by reading the
  line; it needs the type's own rule, and two look-alike types have opposite ones.
- What would pass: R5 says the exception in one sentence ("a type whose name says `Mut`, `Mutex`,
  `Atomic` changes through any name; everything else needs `ref mut`"), and `Cell` joins one side; and
  `ref mut` on a link or view parameter says in one rule whether it covers the link, the target, or
  both, for `MutSpan` and `shared` alike. (A fixer may not add words, rule 19; this needs a rule, not a
  word.)

### R18-02. The type `fn()` hides whether a call changes the function's own state (agreed row 6 at stake)

- D3 (line 6926): "A function value whose body changes **its own** state is called through a name that
  may change it (the owner, or a `ref mut` parameter)". AN-54 refuses accordingly (line 7226): "`run(cb)`
  with `fn run(f: fn()):`: `cb` changes its own copy of `score` each time it runs, and `run` takes it
  read-only; write `fn run(ref mut f: fn()):`".
- AN-53's NEW (lines 7047-7055), which the model accepts, stores handlers as `handlers: Array[fn()]` and
  calls them from plain `self` through a plain loop name: `fn click(self): for h in self.handlers: h()`.
- Row 6's agreed spelling for a copy is `b.on_click(fn() with copied score: score += 1)`. Given to that
  `Button`, the handler changes its own state, and `click` calls it through plain `h` in plain `self`.
  The model says nowhere whether this is refused (D3 applied: then the agreed copy spelling cannot be
  used with the model's own `Button`), or accepted (then D3 is not kept, and `click(self)`, whose first
  line says it changes nothing, changes the handler's stored copy).
- Either way the line `fn click(self):` and the name `h` carry no word for the change: `fn()` as a type
  does not say "a call may change this function's own state", so the owner's test has nothing to point
  at. AN-62 has the same shape (`self.on_visit(p)` from plain `self`, line 7920).
- Agent 15 passed this on (`15-vocabulary-review.md` section 6 point 1); it is still open in the model.
- Row 6 itself is unchanged (section 3); the failure is that the model gives it no defined meaning in
  the program it is about.

### R18-03. `=` does five things, chosen by what was on the right of an earlier line

The model's own rules give `x = y` these meanings, with no word in the line:

| the line | what it does | rule |
|---|---|---|
| `b = a`, `a` a whole name of a list, string, class object | hands `a` over; `a` ends | R1, row 9 |
| `n2 = n`, a number; `v2 = v1`, a `Copy` struct | copies (the struct copy then may not be changed) | R1, C2 |
| `n = p.name`, a part | a read-only view of the part | R2 |
| `n = q.name` after `n = p.name` | points the view `n` at another part | R2, line 79: "A plain view name may be bound again with `=`" |
| `r = 20` after `r = ref mut a` | writes 20 into `a` | AN-28, lines 4781-4789 |

- The last two lines are the same shape (`name = value`, the name bound earlier) and do opposite
  things: one re-points the name, the other writes through it into someone else's place. Which one
  happens depends on the word on a line that may be far above. In Python `x = y` always re-points `x`.
- AN-28 refuses only `r = ref mut b` (line 4806); `r = b` or `r = 20` is accepted and writes into `a`.
  That is exactly "a form that is accepted but does not do what its words mean elsewhere" (the
  error-message rule): elsewhere (R2) re-binding a view name re-points it.
- Not stated at all: `n = p.name` then `n = String("x")` (a plain view name given a value that is not a
  part). R2 says the view "then names the new part"; a new value is not a part. Whether `n` becomes an
  owner, or the line is refused, is not in the model.
- The combiner listed half of this as open point 3 (line 499: hand-over against view). The re-point
  against write-through half is not listed anywhere.

### R18-04. `changes` means "this body and its calls" on a method, "this body only" on a free function

- AN-17 refuses (line 3527): "`tick` calls `note`, which changes `log`; add it to `tick`'s first line:
  `fn tick(self) changes players, log:`". So a method's `changes` must name what its callees change.
- AN-76 rule 2 (lines 9737-9741): "A function whose own body changes it says so on its first line:
  `changes FRAME` ... Callers write nothing; the compiler carries the change in the caller's effect
  summary". AN-76's own `main` calls `tick()` (which `changes FRAME, LOG`) and says nothing (line 9725).
- R5 (line 127) states one rule for both: "Every clause is checked against the body both ways ... and
  at every call."
- So one word, `changes`, has two scopes, and a reader of `fn main():` cannot see that it changes
  `FRAME`, while a reader of `fn tick(self) changes players, log:` can see everything `tick` changes.
  One of the two must give way (either callers of a `changes FRAME` function name it too, or a method's
  callers are carried silently as well, which would undo row 3's reason for existing).

### R18-05. E7016's meaning names, as refused, the two loop forms the model accepts

- AN-84's code table (line 10619): "`E7016` (new) | a `@parallel` loop is over something it cannot
  split (it counts over a range, or walks a list with `for ref mut p in xs`)".
- AN-74's NEW, accepted (lines 9387-9391): `@parallel` / `for ref mut p in players:` ("hands each chunk
  its own elements", line 9404).
- AN-84's own NEW, accepted (lines 10650-10653): `@parallel` / `for i in 0..idx.len():`, a loop that
  counts over a range.
- A programmer who looks E7016 up reads that both forms are refused. (The parenthesis appears to
  describe today's E7010 message, "a `@parallel` loop counts over a range `a..b`", line 9341; as written
  it says the opposite of what the model does.) Section 8 (line 488) gives no example either.

### R18-06. `f: once fn()` hands a function over with no mode on the name

- AN-59's NEW (line 7564): `fn call_once(f: once fn()): f()`. Calling `f` uses it up; the test row (line
  7585) says "`once`: a function the call uses up | it is used up by the call". The caller's function
  value ends: a hand-over.
- R4 (lines 98-108), after 15's change 1: "the mode is on the name and the type says what the value is";
  a plain `x: T` "reads the caller's value". 15 removed exactly this form for `shared` ("a hand-over or a
  new owner made by the type word with no mode on the name, which 06's C-2 refuses for `x: owned T`").
- `once` is a type word that makes a plain parameter take the value over. By the model's own rule the
  line is `fn call_once(owned f: once fn()):`, and `f: once fn()` is refused with that line. The `once`
  row of `vocabulary-final.md` (line 208) says only "before `fn` in a type".

### R18-07. AN-43's message gives a line that R4 now reads the other way

- AN-43 still refused (lines 6230-6231): "`take(shared a)` into `owned n: Node`: '`take` takes a `Node`
  held alone; to give it a second owner declare `n: shared Node`'".
- R4 after 15's change 1 (lines 103-105): "`x: shared T` reads the caller's `shared` value (a view: no
  count ...), and `owned x: shared T` makes the callee one more owner".
- So the message's line to write makes `take` a reader, not a second owner. It should read "declare
  `owned n: shared Node`". 15's change 1 updated R4, C5, AN-48 and AN-78 but not this message.

### R18-08. `owned fn` is refused with two codes and three texts; one names a variable the program lacks

| answer | code | text |
|---|---|---|
| AN-47 (line 6436) | E0109 | "`owned fn` does not say what the function takes" / "name what it takes: `fn() with owned report:`; `task` captures nothing, so write `fn() => println("done")`" |
| AN-54 (line 7218) | none given | "name what the function takes or copies on each name: `fn() with owned name, copied score:`; a name not listed is the real variable" |
| AN-78 (line 10136) | E0100 | "name what the function copies or takes: `fn() with copied score:`" |

- Section 8 (line 471) gives `owned fn` to E0109. AN-78 uses E0100, which today is the parser's
  "expected a name" (pasted at line 5888) and the `as?` refusal (line 8410).
- AN-78's program has `game`, not `score`: the line to write is not in the program's own names (the
  right one is `fn() with shared game:`, as AN-78's own E7004 message says, line 10130).

### R18-09. Error codes: one mistake, several codes; one code, several mistakes

From `target/redesign3/18-readability/codes.txt` (every refusal in a "Still refused" point):

- **A view used after a change that may move or free what it points into** (R8 (b)) is E3020 in AN-46
  (line 6382) and AN-52 (line 6842), E3025 in AN-37 (line 5728), E3060 in AN-50 (line 6678).
- **E3020** also means "a list changed while a loop walks it" (AN-42, AN-48, AN-76, AN-77: lines 6112,
  6527, 9811, 9980). **E3060** also means "a task uses a local of one loop round" (AN-73, line 9284).
- **E3021 and E3023** both mean "a change through a name that only reads" (section 8, lines 475-476),
  split by the kind of name; agent 15 passed this on (its section 6 point 2).
- **E0100** gets a third meaning (`mut path` in a pattern, AN-38, line 5780) besides today's two above.
- Section 8, the model's table of codes, lists none of E3020, E3025, E3060, E0100, so the reader cannot
  look them up there. AN-84's own rule is "one meaning per code"; the converse, one code per mistake,
  is what lets a programmer search for the fix.

### R18-10. A `for` loop in a stored generator walks a list that may change

- The agreed limitation (JOB; R8 (c), line 174): "A list may not change while a `for` loop walks it".
- AN-77's rule (lines 9922-9926): when the generator is consumed in place the limitation holds, but
  "When the generator is stored or passed, the walk stays memory-safe by position and sees the list as
  it is at each resume, as Python's list iterator does for `append` and `pop`".
- So `for entry in self.log: yield entry.len()` means "the list does not change during this loop" in
  one use of the generator and "walk by position over a list that may grow or shrink, skipping or
  repeating entries" in another, decided by how a caller elsewhere uses it. The line cannot tell the
  reader which. Memory safety is kept (positions are checked); the meaning is what changes, and the
  error-message rule asks for a refusal of a form that does not do what its words mean elsewhere.
- What would pass: refuse a change to the walked list while such a generator is alive (the agreed
  limitation, as for any loop), or have the generator's loop say it walks by position
  (`while i < len(self.log):`, AN-60's form), so the line says it.

### R18-11. AN-70's NEW declares a virtual no override may use to change the door

- 15 added to R5 (lines 129-136): an override's `changes` stays within the virtual's first line; "a base
  meant for overrides that change their object says so: `virtual fn on_update(mut self, dt: float):`".
- AN-70's NEW (lines 8719-8722) is `open class Door: angle: float = 0.0 / virtual fn on_update(self,
  dt: float):`, whose intent is "declare a method that subclasses may replace". With plain `self`, every
  override of `on_update` is refused if it changes `angle` or any field. The spec's own `Door` example,
  which 15 cites (spec 1704, 6160), changes `angle` in `on_update`. AN-70 was not updated with R5's
  new point, so its NEW now says less than its intent.

### R18-12. A word on a later line turns an earlier plain local into a shared owner

- AN-71 (line 9000), the owner's test row for `names` on line 19: "`thread_shared` (line 18's capture
  made the caller's name one of the owners, as row 9's `shared items` does to `items`) |
  `thread_shared` (read-only from now on)". `names = Names()` (line 17) is "a new local, the function's
  own object"; a capture list in a lambda on the next line makes it one of several owners on several
  threads and forbids changing it from then on (E7014).
- The same shape: AN-34 `other = shared items` (line 5423), AN-36 `root.children.push(shared child)` then
  `child.value = 5` "seen as `root.children[0].value`" (line 5600), AN-39 `mesh=shared m`, AN-44
  `p2 = shared player`.
- The test tables give the declaring name "none (default: the value made on this line belongs to the
  name)". After the later `shared`, the name aliases another owner, which the brief's default must
  never do. The word `shared x` does name `x`, so the owner may judge that it says it; but in AN-71 the
  word sits in a lambda's capture list and still changes what the outer `names` may do for the rest of
  the function, which nothing on `names`' own lines shows. Area E's own ruling (line 8866) says a value
  must be "created `thread_shared` from the start" when other owners exist; AN-71 does not do so.
- For the owner: is `shared x` on a later line enough, or must the declaring line say it
  (`names = thread_shared Names()`, as AN-80 does at line 10325)?

### R18-13. A `weak` link is copied from a part and handed over from a whole name

- R2 (lines 74-76): a part with no fields of its own, "a `weak` link", is copied when bound (`a =
  n.parent`, AN-62, lines 7918 and 7922).
- R1 and the `Copy` row (15's change 11): `weak` links are not `Copy`, so `w2 = w1` with a whole name
  hands `w1` over and ends it.
- So `a = n.parent` leaves `n.parent` working, and `b = a` on the next line ends `a`. Same type, same
  line shape, opposite effect. Agent 15 passed this on (its section 6 point 4). Either `weak` is `Copy`
  (it has no count to touch, so copying it is free and cannot surprise) or R2 does not copy it.

### R18-14. `mut` alone still marks a changeable name in two places

- Row 8 refused `mut path` because it says "may change" but not whose ("in Rust `mut path` is a changeable
  copy"). 06 extended the refusal to every parameter and lambda parameter (`vocabulary-final.md` line
  171; AN-27, line 4745; AN-20, line 3848).
- `mut self` stays (AN-18, line 3592; R5; 15's new `virtual fn on_update(mut self, ...)`), and so does
  `static mut` (AN-76). 06's reason for `mut self` (line 171): "the word `self` says what it is".
- That reason holds for the "what it is" column, but Rust's `fn f(mut self)` is also a changeable copy
  (by value), the exact reading the owner gave for failing `mut path`. A reader from Rust reads
  `mut self` as "my own changeable copy of the receiver"; the model means "the caller's receiver,
  any field". The spelling that passes row 8's test the same way as every other name is `ref mut self`.
- Judgement, for the owner: keep `mut self` (06's reason), or spell it as every other in-out name.

### R18-15. One comma shape, two rules

- AN-15's NEW (line 3199): `for ref mut pos, vel in q:`: `ref mut` applies to `pos` only; `vel` is plain
  (test row line 3214).
- AN-54's refusal (lines 7224-7225): `fn() with copied name, score:` is refused: "say what the function
  does with `score`: `with copied name, copied score`, or leave `score` out".
- The same visual list (a word, then two names separated by a comma) is accepted with the second name
  plain in a loop, and refused as unclear in a capture list. A reader cannot carry one rule from one to
  the other. Either the loop form is refused with the same advice (`for ref mut pos, vel` -> write the
  pattern as `(ref mut pos, vel)`), or the capture list accepts a plain second name.

### R18-16. A call changes an argument with no word in the line (clash 9, open point 1)

- R4 (lines 109-111): "The call writes no word." `collide(bodies[i], bodies[j])` (AN-03, line 1026)
  changes two bodies; `bump(a)` (AN-27, line 4715) changes `a`; `x.inspect(s)` (AN-67, line 8439)
  changes `s`; the AN-16 table (line 3296) gives `es` in `es[0].heal()` "none (default: its own; `heal`'s
  first line ... says it changes `es[0]`)", pointing at a word in another function.
- The owner's test asks, for every name in a line, for the word in the line. In `bump(a)` the name `a`
  is changed and no word in that line says so. The combiner's reason (an argument binds no name, line
  393) answers a different question; row 3 settled the method's first line, not its call.
- Already open for the owner (line 494). Listed because, read literally, it fails the test.

### R18-17. AN-05: handler order reversed, and a position can go stale

- Intent (line 1371): "the game's `on_death` handler (any function given in) is called for each dead
  enemy; the handler may remove it from the list."
- NEW (lines 1437-1438): `while dead.len() > 0: on_death(enemies, dead.pop().unwrap())`: the handlers
  run for the last dead enemy first, the reverse of the OLD program's order; a handler with any visible
  effect (a score, a log line, a sound) now runs in the other order. The intent does not say order, but
  the OLD program's order is part of what it did, and the NEW text does not mention the change.
- The note "each removal leaves the positions still to come where they were" (line 1441) holds only if
  the handler removes exactly the enemy at the position it is given. A handler "given in" may remove
  another enemy (a neighbour, the first one); then a later position in `dead` names a different enemy,
  or runs past the end (a run-time stop, not a memory error). The model gives no message or rule for it.

### R18-18. AN-35's refusal gives the OLD signature's reason

- NEW (line 5521): `fn spawn_all(template: Player, n: int)`: `template` is a plain parameter, a
  read-only view.
- Refusal (lines 5552-5553): "`v.push(template)` hands `template` over on the first pass, so the second
  pass has none". That is the reason for the OLD `owned template`. With the NEW signature the reason is
  that `template` only reads the caller's player and cannot be handed over at all ("`template` reads the
  caller's player; to put a player of your own in the list write `v.push(copied template)`").

### R18-19. Messages that name what is not in the program

- AN-46 (lines 6382-6383): "`m.insert(...)` changes `m`, whose keys `keys` (line 7) still views on line
  9 ... keep your own strings: `keys = sorted(m.keys().copied())`". The NEW code has no `keys`; it has
  `for k in sorted(m.keys()):` (line 6351).
- AN-81 (lines 10449-10451): "`total` is `@no_alias_checks`, and `collide(xs[i], xs[j])` (line N) compares
  `i` and `j`": `total` has no `collide`. The message shape is fine; its example is from AN-03.
- (AN-79's new warning for a `# SAFETY:` note on an `unsafe fn` line has no code, line 10253: "the code
  number is for the lead". Every other refusal carries one.)

### R18-20. A plain number parameter: may it be changed inside the function?

- AN-02's test row for `x: float` (line 918): "none (default: read; changing it changes nothing the
  caller holds)", which implies `x += 1` is accepted.
- R5 (line 137): "A plain name never changes anything: a plain parameter ...", and AN-43's row for
  `n: int` (line 6204): "never changes the caller's"; AN-03's `i`, `j` (line 1046): "none (default:
  read)".
- `mut n` on a parameter is refused (AN-27), and `ref mut n` changes the caller's. Python lets a
  function re-bind its own parameter (`def f(n): n += 1`). The model never says whether `n += 1` on a
  plain `n: int` is accepted, and, if it is refused, which line to write (`m = n`, then change `m`?).

### R18-21. AN-34's NEW block uses a name it has handed over

- Lines 5421-5423, written after `main` as "the other two ways the line can be written": `log = items`
  ("hands the list over: `items` ends here"), then `other = shared items`. Read as one block, the third
  line uses `items` after the second ended it, which AN-34's own refusal (line 5443) refuses. The two
  lines are alternatives, not a sequence; the block should show them in separate functions (as AN-36
  does with its two forms).

### R18-22. `ref mut` promises "no other name" on a parameter and not on a kept view

- R4 (lines 112-116): two `ref mut` arguments, or a `ref mut` and a view, are two different places;
  "Every `ref mut` parameter is compiled with noalias."
- R9 (lines 192-193): "Kept `ref mut` views (in a struct or a list) may name one object, as `shared`
  owners do." AN-52 (b), line 6804: `target: ref mut Player` in many watchers naming one player.
- The same two words mean "this name is the only way in for this call" on a parameter and "one of
  possibly many ways in" in a field. A reader from Rust reads both as the first. The combiner listed
  this as open point 4 (line 502). Judgement, for the owner.

## 3. The agreed rows 3 to 9: unchanged

| row | agreed (`JOB.md`) | in the model | exactly as agreed? |
|---|---|---|---|
| 3 | `fn note(self, msg: String) changes log:`, checked against its body | AN-17 NEW line 3464; refusals 3525-3528 | yes (but see R18-04: what `changes` means for callers differs from AN-76) |
| 4 | `return ref g[0]` in `with g = scores.lock():` refused: "this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block" | AN-30 line 4995-4996, word for word | yes |
| 5 | `word = line.to_lower().trim()` kept to `word`'s last use; returning it refused, "return `String(...)` to hand back your own copy" | AN-08 line 2093; AN-09 refusal lines 2237-2240 contain the sentence | yes |
| 6 | `b.on_click(fn(): score += 1)` changes `game`'s real `score`; a copy only when named, `fn() with copied score:` | AN-53 line 7060; R3 line 93; AN-54 | yes as words; see R18-02 for the copy spelling inside AN-53's `Button` |
| 7 | `fn load_assets(self) reads renderer -> Array[String]:`, checked against its body and at the call | AN-31 line 5057; refusals 5094-5101 | yes |
| 8 | `ref mut path` stays; plain `path` + `path.pop()` and `mut path` both get "to change that list through `path`, write `State.Walking(ref mut path):`" | AN-37 line 5698; AN-38 refusals 5774-5782, the help word for word in both | yes (the `mut path` refusal uses E0100, R18-09) |
| 9 | `snapshot = copied items` copies; `snapshot = items` hands over; the error names `copied` and `shared` | AN-34 lines 5417, 5422, refusal 5443-5445 | yes (R18-21 is about the block's layout, not the row) |

## 4. Looked at and passed (not failures)

So the fixer does not spend time on them:
- AN-12's map key: `counts[w] = ...` stores the map's own `String` made from the view `w` with no word.
  It is a copy no word names, but the map's type `Map[String, int]` says its keys are its own, and keys
  never change, so no line can tell the copy from the view (clash 11). Passes.
- Implicit boxing into an `owned` field (`e = Entity(mesh=Mesh(...))`, AN-40, line 5929;
  `Array[owned dyn Shape]`, AN-66): the field's type says it. Passes.
- R2's plain part view (`boss = enemies[0]`) aliases the element (A-2: a write through `enemies[0]` is
  seen at `boss`'s next use). The brief's default must "never alias"; the owner accepted plain `path` as
  a read-only view in row 8, so read-only aliasing is taken as agreed. For the owner to confirm.
- `@cached` writing its slot through plain `self` (AN-21): `@cached` is the word. Passes.
- `copied w` of a `StrView` refused with "write `String(w)`" (AN-07, line 2045) while `copied ps[0]`
  of an `SoA` element makes a value (AN-13): both are "your own copy"; for text the copy's type differs
  from the view's, so the refusal gives the line. Passes the error-message rule.
- `.copied()` on an iterator (AN-46) beside the keyword `copied`: the vocabulary keeps it with the same
  meaning (line 340 of `vocabulary-final.md`). Passes.
- `Pool` naming an interface (AN-40), a program's generic struct (AN-22, AN-42) and the library's
  `Pool[T]` (AN-39's table): program names, not words; confusing only across examples (15's note 3).
- Heavy but readable lines: `fn integrate(ref mut q: Query[(ref mut Position, Velocity)], dt: float):`
  with `for ref mut pos, vel in q:` (three `ref mut`, each about a different thing); `pool = ref mut cast
  self.pools[7u128] as Pool[T]`; `owned m: owned Mesh`; `fn() with weak ui: if Some(ref mut me) = ui:
  me.clicks += 1` (two colons on one line, AN-60, line 7694). Each word says one true thing; they pass
  the test, and are listed only as the places where the NEW code reads least like Python.
- Every other answer's owner's-test table matches its rules (beyond the rows above), and every other
  "means exactly" translation does what its intent line says, including AN-71's reduction (its one
  difference, overflow in a partial sum, is stated) and AN-62's deferred-drop region (no visible drop
  moves: only classes with no `drop` method).

## 5. Files

- `C:\Users\ism19\Code\ember\tasks\96\redesign\run3\18-readability.md`: this report.
- `C:\Users\ism19\Code\ember\target\redesign3\18-readability\codes.sh`: lists every error code a NEW
  refusal gives, with answer and line; `codes.txt`: its output (64 refusals, 31 codes), the evidence
  for R18-08 and R18-09.
