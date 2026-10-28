# The combined memory model (run 3, phase 3; agent 14, combiner)

One model and one vocabulary made from the five area reports of phase 2 (`RUN3/08-area-A.md` ..
`RUN3/12-area-E.md`), with `RUN3/vocabulary.md`, `RUN3/planned-words.md`, `RUN3/13-old-code-recheck.md`
and `inputs/JOB.md`. The vocabulary is `RUN3/vocabulary-combined.md`; the combiner's own report is
`RUN3/14-combiner.md`.

How to read this file:
- Sections 1 to 9 are the combiner's: the model in plain words, the rulings on every new word, every
  clash between areas and how it was settled, every rewrite made to an area's text, and the agreed rows.
- Sections 10 to 14 are the 84 answers, grouped by area, each in the eight-part form, copied from the
  area reports by `target/redesign3/14-combiner/build.py` (so every OLD program and every pasted
  `ember check` output is the area's own text, byte for byte). Each area's own rules come first in its
  section, because its answers cite them (A-2, C6, D5 ...). Wherever the combiner changed an area's
  text, the change is marked **14:** and listed in section 4.
- **This is `combined-v2.md`**: `combined.md` copied by agent 15 (vocabulary reviewer) with `cp`; every
  change 15 made is marked **15:** in place and listed, with its reason, in `RUN3/15-vocabulary-review.md`
  section 2. The vocabulary that goes with it is `RUN3/vocabulary-final.md`.
- **This is `combined-v3.md`**: `combined-v2.md` copied by agent 19 (fixer) with `cp`; every change 19
  made is marked **19:** in place, names the problem it fixes (R18-01 .. R18-22 from
  `RUN3/18-readability.md`; VOC from `RUN3/15-vocabulary-review.md` section 5), and is listed with the
  test it now passes in `RUN3/19-fixer.md`. No word was added or changed; the vocabulary rows that must
  follow these rule fixes are listed there for the lead (`vocabulary-final.md` is not edited).
- **This is `combined-v4.md`**: `combined-v3.md` copied by agent 21 (fixer, round 2) with `cp`; every
  change 21 made is marked **21:** in place, names the problem it fixes (F-01 .. F-12 from
  `RUN3/20-re-attacker.md`), and is listed with the test it now passes in `RUN3/21-fixer.md`. No word
  was added or changed; one error code was added (E7017, section 8).
- `RUN3` means `tasks/96/redesign/run3/`.

Power: `PowerLineStatus` printed `Online` at the start of this work and again before writing this file.
No compiler change, no git command, no Ember program run.

## 0. Terms

- **owner**: the one place (a local, a field, a list element) that ends a value when it ends.
- **handed over**: the giver's name ends; the receiver is now the owner (row 9: `snapshot = items`).
- **copy** (`copied`): a new value equal to the old one; nothing done to one is seen through the other.
- **part**: a field of a value (`p.name`), an element of a list (`enemies[0]`), or the payload of an enum
  variant (`State.Walking(path)`).
- **view**: a name or value that points into something someone else owns and may not outlive it
  (`ref T`, `ref mut T`, `Span[T]`, `MutSpan[T]`, `StrView`, a plain parameter, a plain loop or pattern
  name). A **kept view** is a view stored in a struct or a list.
- **position**: a view of a list element kept as "element `i` of this list"; the element is found again
  at each use (rule R8a).
- **reshape**: a change that can free or move memory: replacing a value that owns memory, `push`,
  `pop`, `insert`, `remove`, `clear`, `sort` on a list or map, changing an enum's variant, ending the last
  owner of an object. Writing a number or a `bool` is not a reshape.
- **first line** of a function: its signature with its clauses: `reads` (row 7), `changes` (row 3), the
  result type, `from`.
- **`shared` / `thread_shared` value**: a value with several owners, all on one thread / on any threads.
  It is freed when its last owner ends. Its owners are **counted** while the program runs: a number in
  the value's header, changed with a plain add (one thread) or an atomic add (any threads).
- **count pair**: one +1 and one -1 of a count, made around a use to keep an object alive.
- **deferred-drop region**: a stretch of code (a loop) during which an object whose count reaches zero
  is set aside whole and freed when the stretch ends (area D, AN-62).
- **unknown call**: a call through a function value, a `dyn` value, or code whose effects are not known,
  taken to do everything its type allows.
- **scope**: the block `with scope = thread.scope():`; its **tasks** may use the parent's variables
  because the block does not end until every task has finished (the **join**).
- **the owner's test** (brief): for every name in a line, the word that says what it is (the owner's
  own thing, a copy, or handed over) and the word that says whether it may change it; "none (default:
  ...)" only where the default never changes anything, never aliases silently and never copies silently.
- **noalias**: C's `restrict`: the promise to the C compiler that two pointers never point at the same
  memory, which lets it keep values in registers and use vector instructions.

## 1. The model in plain words

### 1.1 The rules

Each rule names the area rules it merges (A-1 .. A-8, B items 1-8, C1 .. C10, D1 .. D8, E items 1-9;
they are printed with their areas in sections 10 to 14). "14" marks what the combiner settled.

**R1. One owner; `=` with a whole name hands it over.** Every value has one owner unless a line says
otherwise. `b = a`, with a whole name on the right, hands `a` over (row 9) for every type that is not
`Copy`: lists, strings, structs that own memory, class objects. `Copy` (06's C-3) means an independent
copy: numbers, `bool`, `ref T`, `Span`, structs made only of such. A `Copy` struct copied with `=` from
a name that lives on may not then be changed through the new name: the line says `copied` (your own) or
`ref mut` (the original's) (C1, C2, C8).

**R2. A name for a part is a read-only view of it** (14, settling A-1 against C1). A name bound to a
part of a value someone keeps: a field (`n = p.name`), an element (`boss = enemies[0]`), a pattern name
(row 8's plain `State.Walking(path):`), a loop name, a view of a temporary (row 5's `word`), is a
read-only view of that part: never a hand-over, never a silent copy. A part with no fields of its own
(a number, a `bool`, a variant with no payload) is copied, as Python binds a number: nothing can be
changed through such a copy, so the copy cannot surprise. **19:** (R18-13) a `weak` link is not in that
list: it is not `Copy`, so it follows this rule and R1 like every other type that is not: `a = n.parent`
is a read-only view of the link, `a = copied n.parent` is a link of your own (AN-62 walks this way),
and `w2 = w1` hands `w1` over. A copied link is one pointer plus, as Rust's `Weak::clone`, one add to
the target's weak count (the count that keeps the target's block, not the object, until its last
link ends; `c/an63_links.c`'s `weak_store`). **21:** (F-01) The add is left out only where the copy's
whole life is covered: by R10's point 1 (no line in the copy's life can end the target's last owner),
or by a deferred-drop region that ends after the copy does (a local of the loop's body, as AN-62's
`a`; R10's point 2). A copy stored in a field, a list or a capture, handed to an `owned` parameter,
returned, or kept in a local that outlives the region takes its count when it is made, as Rust's
`Weak::clone` does: a region keeps the block of an object it set aside only until its loop ends, and
such a copy is read after that. (Was: "left out by R10's points 1 and 2 wherever nothing in the
copy's life can end the target's last owner or a deferred-drop region covers it", which let a copy
made inside the region and stored in a field take no count: `20-re-attacker/c/f1_weak_copy_escape.c`,
heap-use-after-free.) (Was, before 19: "a `weak` link" in the list above, which copied a link bound from a part while R1
hands a whole `weak` name over.) A part that is an owning link (a class-object field, a `shared` field) gives a view of the object it links to, as C's AN-51 says,
so re-pointing the field later does not move the view; R10 keeps that object alive. **19:** (R18-03) No
view name is pointed elsewhere: a plain view name and a `ref mut` name each refer, for their whole life,
to the place they were made from (AN-28's rule; row 8's `path` is the NPC's list for the whole arm;
AN-11's `pos`). So `=` on a name that already has a value means one thing, "put this value in the
place the name refers to": an own name's value is replaced (the old value ends); through a `ref mut`
name the value goes into the original (`r = 20` writes `a`, AN-28); a plain view name only reads, so
`n = q.name` or `n = String("x")` after `n = p.name` is a change through a plain name, refused (E3023):
"`n` reads `p.name` for its whole life; for `q`'s name use a new name: `m = q.name`". The names that
refer to a place are those bound to a part written as a place (`n = p.name`, `boss = enemies[0]`,
`r = ref mut a`), a parameter, and a loop or pattern name. (Was: "A plain view name may
be bound again with `=` (it then names the new part); a `ref mut` name is never re-pointed", which made
`n = q.name` re-point `n` while `r = 20` on a `ref mut` name wrote through it: one shape, two
opposite meanings, chosen by an earlier line.) **21:** (F-05, F-06) **A name whose value is a view
is given its value once, wherever the view came from.** A call whose result type is `ref mut T`
names a place (R6), and so does one whose result type is `ref T` (the same place, read-only), so a
name bound to such a call follows the rule for parts: plain `r = first_even(a.as_mut_span())` copies
the number, as `n = counts[0]` does (and is a read-only view when the result is not a number), while
`r = ref mut first_even(a.as_mut_span())` is a `ref mut` name of that element, so `r = 7` writes it,
as AN-28's `r = 20` writes `a`. A call whose result is a view (a `StrView`, a `Span` or a
`MutSpan`), as in `n = p.name.trim()`, `best = a.as_span()`, `a = p.next_word()` and row 5's `word`,
gives a name that views what that result points into for its whole life, as `n = p.name` views
`p.name`; so `=` on it later is refused the same way (E3023): "`n` reads `p.name` for its whole life;
for `q`'s name use a new name: `m = q.name.trim()`", and it is not given to a `ref mut` parameter,
which could point it elsewhere (R4). (What the view lets the name do is unchanged: a `MutSpan` name
still writes its elements, R5.) A name whose value is its own is replaced by `=`: a number, a
`String`, a list, a link or a copy made with `copied`, `shared` or `weak`, a hand-over, and a struct
or a generator a call makes even when it holds views (AN-22's `p = Parser(src=text)`, AN-77's
`g = d.replay()`: their view fields and what they walk follow R7, R8 and R14, and their own fields
change as any own value's; R7's "itself a view" says how long such a value may live, not that the
name only points). A name first
given its value inside a loop's body is given it once per round (`[GRM-4]`: it is declared in that
block). So whether a later `=` compiles is read from what the name holds, a view or a value of its
own (row 5 already asks the reader to know that `trim()` gives a view), never from whether its first
line ended in a call. (Was (19): "A name given the result of a call (`word = line.to_lower().trim()`,
`w = it.next()`, `a = p.next_word()`), a new value, `copied`/`shared` or a hand-over is an own name:
its value may itself be a view ..., and `=` replaces that value", which made `r = first_even(...)` an
own name though R6 calls the call a place, so `r = 7` no longer wrote the element (F-05), and
accepted `n = q.name.trim()` after `n = p.name.trim()` while refusing `n = q.name` after
`n = p.name` (F-06); the model's own owner's-test tables already called such names read-only views,
"as row 5's `word`" (AN-22's `rest`, `a`, `b`).) `ref mut` before the part makes a view that may
change it (row 8); `copied` makes your own copy (rows 6, 9); `ref` before the part says the default and
is accepted. A part **stored** somewhere else (`push`, `return`, a field, an argument to an `owned`
parameter) is handed over, which leaves its owner partly handed over (today's `[EXP-6]`).

**R3. Words before a value, in a type, in a capture list** (C3, C4, D2, D8, E items 2 and 4; 07's table).
- Before a value: `copied x` (runs the type's `Clone`), `shared x` (one more owner, this thread),
  `thread_shared x` (one more owner, any thread), `weak x` (a link that does not keep it alive).
  `owned x` is refused: `b = a` already hands over (E0109).
  **21:** (F-08) `thread_shared x` makes one more owner of a value that is already `thread_shared`
  (made so on the line that creates it, `names = thread_shared Names()`, or a parameter
  `x: thread_shared T`); before a new value (`thread_shared Names()`) it makes that value with owners
  on any thread. On a name whose value is not `thread_shared` (a plain local, a `shared` value, a plain
  parameter) it is refused, before a value and in a capture list alike, because it would freeze a
  value that its own line lets its owner change (R9): E7017, "`names` is `share_and_keep`'s own
  object, which it may change; to share it between threads, create it shared:
  `names = thread_shared Names()`". `shared x` on a plain local stays as row 9 has it
  (`snapshot = shared items`): any owner of a `shared` value may still change it, so the earlier line
  keeps its meaning. (Was: this line said "one more owner, any thread" for every value, while AN-71
  refused the capture of a plain local under E7015, whose meaning is a one-thread owner reaching
  another thread.)
- In a type: plain `T` (held alone: a struct's value inside its owner, a class object in its own block);
  `owned T` (held alone in its own heap block; replaces `Box[T]`, `Box[dyn I]` and `ForeignBox[T]`;
  refused on a class type, which is always in its own block); `shared T`; `thread_shared T`; `weak T`;
  `owned dyn I` and `shared dyn I` (a bare interface name used as a type is refused).
- In a lambda's capture list, after its parameters: `fn() with copied a, owned b, shared c,
  thread_shared d, weak e:`; each word means what it means before a value; a name not listed is the real
  variable (row 6).
- **19:** (R18-15) **One rule for a word before a list of names.** A word belongs to the one name
  after it. Where a word stands before the first of several names separated by commas, in a capture
  list or among a loop's names, every name in the list writes its own word, so the first word cannot
  be read as covering the list: `with copied name, copied score`; `for ref mut pos, ref vel in q:`
  (`ref` says a plain name's default, a read-only view, R2). `with copied name, score` and
  `for ref mut pos, vel in q:` are refused with the same advice: "`copied` here belongs to `name`
  only; say what the function does with `score`: `with copied name, copied score`, or leave `score`
  out to use the real `score`" and "`ref mut` here belongs to `pos` only; say what `vel` is:
  `for ref mut pos, ref vel in q:`, or `ref mut vel` to change it too". A list whose first name has no
  word (`for k, ref mut v in m:`) needs nothing more. (Was: AN-15's loop accepted a plain second name
  while AN-54 refused it in a capture list: one comma shape, two rules.)
  **21:** (F-12) The rule is for every list of names and for a word on any name in it: a capture
  list, a loop's names, and a pattern's names (`State.Moving(ref mut pos, vel):`,
  `if Some((ref mut a, b)) = pair:`). In any such list, a name with no word that follows a name with a
  word is refused with the same advice, since it can be read as covered by that word:
  `for e, ref mut pos, vel in q:` gets "`ref mut` here belongs to `pos` only; say what `vel` is:
  `for e, ref mut pos, ref vel in q:`, or `ref mut vel` to change it too", and
  `State.Moving(ref mut pos, vel):` gets "`ref mut` here belongs to `pos` only; say what `vel` is:
  `State.Moving(ref mut pos, ref vel):`, or `ref mut vel` to change it too". Names before the first
  word need none (`for k, ref mut v in m:`, `for e, ref mut h in q:`). A parameter list is not such a
  list: each parameter is written whole, with its own type (`fn collide(ref mut a: Body, b: Body):`).
  (Was: capture lists and loop names only, and a word on the first name only.)

**R4. Parameters and calls** (B item 3, C5, D AN-56, A-4; 14 settled the call site and widened A-4).
- `x: T` reads the caller's value (a read-only view; a number is copied). `ref mut x: T` may change the
  caller's value (06's C-1): the in-out parameter, written the same way on a lambda parameter
  (`fn(ref mut ui)`) and in a callable type (`fn(ref mut Ui)`). `owned x: T` takes it over (refused where
  `T` is known to be `Copy`). `owned m: owned Mesh` takes over a one-owner block and keeps the block.
  **15:** the mode is on the name and the type says what the value is, for `shared` and
  `thread_shared` as for every type: `x: shared T` reads the caller's `shared` value (a view: no count;
  R10 keeps the object alive), `ref mut x: shared T` may change it, and `owned x: shared T` makes the
  callee one more owner (the caller writes `shared y` for a new owner, or hands its own over); the same
  for `thread_shared T`. (Was: "`x: shared T` / `x: thread_shared T` makes the callee one more owner",
  a hand-over or a new owner made by the type word with no mode on the name, which 06's C-2 refuses for
  `x: owned T`, and which AN-78's own `setup(game)` followed by `game.score` contradicted.)
  **19:** (R18-20) A plain parameter is never changed in the function, a number included: "a number is
  copied" says how it is passed, not that the function may change it. `n += 1` on `n: int` is E3023, as
  today (`target/redesign3/19-fixer/o20_plain_int_param.em`: `error[E3023]: cannot mutate borrowed
  parameter \`n\``), now with the line to write: "`n` is the number the caller passed and is not
  changed here; for a number of your own write `count = n` and change `count`" (accepted,
  `m20_int_param_copy.em`, `exit=0`). A changeable copy of a parameter is always a new own name:
  `count = n` for a number (R1), `mine = copied p` for anything else.
  **19:** (R18-01) **What `ref mut` adds, one rule for every type:** `ref mut x` names the caller's
  place itself: every change through `x` reaches the caller's value, and `x = v` puts `v` in the
  caller's place (R2, AN-28). For a parameter whose value is a link or a view (`shared T`, `weak T`,
  `MutSpan[T]`, `Span[T]`, `StrView`), that place holds the link or the view, so `b = shared q` makes
  the caller's link point at `q` and `pos = other` points the caller's span at `other`. What the link
  or view leads to is changed by its own rule: an object behind a `shared` link only through a name
  that may change it (`ref mut`, an owner, R9), the elements behind a `MutSpan` through any name (R5's
  exception). So on `ref mut b: shared Player` the word gives both the re-point and the change of the
  player, and on `ref mut pos: MutSpan[float]` only the re-point, because plain `pos` may already
  write the elements; the word means the same in both. (Was unstated: AN-11 read `ref mut` on a
  `MutSpan` as "re-point the view", AN-48 read it on a `shared` link as "change the object".)
  **21:** (F-02) **What a view put in through `ref mut` may point into.** After a call, the caller's
  place given to a `ref mut` parameter whose value is a view (`Span[T]`, `MutSpan[T]`, `StrView`) may
  point into whatever the call's other view arguments point into, as well as into what it pointed into
  before: R6's default for a result ("into all of them"), applied to what the call may put there, so
  R8 checks every later use of that place against all of them (E3060 for a use after one of them
  ended). Inside the callee only such a view may be put there: another view parameter or a part of
  one, or text written in the program; a view of the callee's own locals or of a temporary is refused
  (E3063): "`best` is the caller's view and lives on after `pick_longer` returns, but `mine` ends with
  `pick_longer`; put a view of what the caller gave there (`best = other`), or return a list of your
  own: `-> Array[int]`". The place given is a field or a list element that holds a view
  (`pick_longer(c.best, b.as_span())`): a view name is given its value once (R2), so giving one to such
  a parameter is refused (E3023): "`pick_longer` may point `best` elsewhere (its first line says
  `ref mut best`), but `best` views `a` for its whole life; let the function return the view to keep:
  `fn longer(x: Span[int], y: Span[int]) -> Span[int]:`, then `best2 = longer(best, b.as_span())`".
  Links need none of this: a `shared` or `weak` link put in is counted (R10, R2). (Was unstated: after
  `pick_longer(best, b.as_span())` the checker still took `best` to view only `a`, so `b = [6]` then
  `best[0]` read freed memory: `20-re-attacker/c/f2_refmut_view_repoint.c`.)
- **The call writes no word.** `bump(a)`, `collide(bodies[i], bodies[j])`, `self.handlers[i](self)`: the
  callee's first line says what it changes and the call is checked against it, as row 3's
  `self.note(...)` carries no word and as today's `[FN-2a]` says.
- **Two `ref mut` arguments of one call, or a `ref mut` argument and any other view argument, are two
  different places**: proved when compiling (different constants, `j` from `i + 1..`, inside
  `if i != j:` or `if a is not b:`, different objects), or else checked by one compare before the call,
  which stops the program naming both. Every `ref mut` parameter is compiled with noalias. Two `ref mut`
  element names in one function follow the same rule.
  **21:** (F-03) A `MutSpan[T]` argument whose `T` is not `Copy` (R1: a `String`, a list, a class
  object, a link) counts here as a `ref mut` argument: it and any other view argument of the call are
  two different places, proved when compiling (two different lists) or checked by one overlap test
  before the call (two compares, as `assert_disjoint`). A write through it can end a string or an
  object that the other view shows, and the callee cannot see that the two share a list.
  `refill(names.as_mut_span(), names.as_span())` is refused when compiling: "`refill` may replace
  strings of `names` through `dst` while `src` shows them; give it two different lists, or a copy:
  `old = copied names`, then `refill(names.as_mut_span(), old.as_span())`". For a `Copy` `T`
  (numbers), an overlapping write frees nothing, so two spans of one list stay accepted with their
  meaning and cost (AN-11, AN-14). Inside one function the compiler sees that both views come from
  `names` (R6), and R8 (b) applies. (Was: R4's promise covered `ref mut` arguments only, so a plain
  `MutSpan[String]` overwrote a string that a `Span` argument of the same call still showed:
  `20-re-attacker/c/f3_mutspan_overlap.c`; today's Ember refuses that call, E3021.)

**R5. What a function's first line says** (rows 3, 7; B items 1, 8; C9 for `Cell`; E item 6; 14
settled the order and the code).
- Plain `self` reads. `self ... changes a, b` may change exactly those fields; `mut self` any field;
  `self ... reads a, b` reads exactly those. `init` sets fields through plain `self` and may call a
  `reads` helper once the fields it names are set. A free function that changes a module variable says
  `changes NAME`.
- Order on the first line: parameters, `reads ...`, `changes ...`, `-> T`, `from ...`, `:`
  (`fn get_mut[T](self, i: int) changes pools -> ref mut T:`; `fn next_word(self) changes pos -> StrView
  from src:`).
- Every clause is checked against the body both ways (error E2105: a change not named, or a name not
  changed) and at every call.
- **19:** (R18-04) **The body includes the calls it makes, for a method and a free function alike.** A
  function that calls one whose first line says `changes X` changes `X` too, and its own first line
  names it in the names it can write: a field of its own object through which it reaches what the
  callee changes (`tick` calls `self.note(...)`, which `changes log`, so `tick` says `changes players,
  log`, AN-17), or the module variable (`main` calls `tick()`, which `changes FRAME, LOG`, so `main`
  says `fn main() changes FRAME, LOG:`, AN-76). A change through a `ref mut` parameter or to an own
  local needs no clause (row 3's `g.tick()` in `main`, AN-17). A function that `thread.spawn` starts is
  not a call of the function that starts it: what it changes is checked by R14 (E7012); a scoped task's
  changes are the parent's (R14). So a reader of any first line sees everything a call of it may
  change. (Was: AN-76's rule 2, "callers write nothing; the compiler carries the change", against
  AN-17's refusal "`tick` calls `note`, which changes `log`; add it to `tick`'s first line": one word,
  two scopes.)
  **21:** (F-04) **Function values are the one exception to that sentence, said here.** A call
  through a function value changes what that value's body changes (R12), and the type `fn()` has no
  word for what that is (section 9, point 9). So: (a) a lambda's body is part of the function that
  writes it: that function's first line names a module variable the lambda changes
  (`fn main() changes LOG:` writes `b.on_click(fn(): LOG.push(...))`), or E2105: "the function made on
  line 2 changes `LOG`, and `setup` writes it; name it on `setup`'s first line:
  `fn setup(ref mut b: Button) changes LOG:`"; (b) a call that may run a function value (a call
  through one, an unknown call, or a call of a function whose first line lets it change a place that
  holds function values: `changes handlers`, `mut self`, a `ref mut` parameter holding them, R12) is
  taken to change every variable that a function value of the program changes: D3's rule, carried by
  the compiler (R16), not by first lines. So `b.click()` inside `for entry in LOG:` is refused when a
  handler written anywhere pushes to `LOG` (AN-76, rule 3). `click`'s own first line, `changes
  handlers`, tells its reader that a call runs the stored handlers; what each handler changes is
  written where it is made. (Was: "So a reader of any first line sees everything a call of it may
  change", with no exception, false for a handler that changes a `static mut`, while R16 left no
  summary that named it: `20-re-attacker/c/f4_fnvalue_static_loop.c`, heap-use-after-free.)
- **15:** A call through a `virtual` method or an interface method is checked against the first line
  of the method it names, so an `override` (or an implementation) says no more than that line: its
  `changes` names only fields the virtual's line lets it change, and only a virtual that says `mut self`
  lets overrides change any field of their own class. A base meant for overrides that change their
  object says so: `virtual fn on_update(mut self, dt: float):`. Refused: "`Door.on_update` changes
  `angle`, but it overrides `Script.on_update`, whose first line changes nothing, and callers of
  `Script` rely on that; write `virtual fn on_update(mut self, dt: float):` in `Script`". (Without this,
  `changes` on a virtual would not mean what it says at a call through the base.)
- A plain name never changes anything: a plain parameter, a plain loop or pattern name, plain `self`
  without `changes`. That includes setting a `Cell` field (B's ruling on 06's C-10) and changing an object
  reached through a `shared` link (14, for C's AN-51). A change through such a name is E3023, with the
  line to write (`ref mut x`, `changes f`).
  **19:** (R18-01) **The one exception, by type, said in one sentence:** a plain name may change what
  a `MutSpan`, `Mutex`, `RwLock`, `Atomic` or channel end leads to, and only with that type's own
  operation written at the use: a `MutSpan`'s elements (`pos[i] = ...`; the type's name says `Mut`);
  the value inside a `Mutex` or `RwLock` through the guard of `with g = m.lock():` or `rw.write()`; an
  `Atomic` through `fetch_add`, `store` and the like; a channel through `send`. These are the types
  made to be changed through read-only names: no owner may change a `thread_shared` value directly
  (R9), so on threads they are the only way, and a `MutSpan` is the changeable view itself. Every
  other type follows the default, `Cell` included: on one thread `ref mut`, `changes f`, `mut self` or a
  `shared` owner says the change at no cost (B's ruling on 06's C-10), so `Cell` needs no exception. A
  field `guarded by` a lock is a plain field of its object and follows its object's rule (`changes
  words`, AN-74). A function value's own state is not an exception either (R12). (Was: the default
  stated with no exception while AN-11's `pos`, AN-30's `scores` and AN-73's `total` changed things
  through plain names.)

**R6. Results that point into something** (B items 4, 5). A method's result points into its object by
default; a function's into the one parameter it could come from, or into all of them. `-> T from name`
narrows it (a parameter, a field of `self` written bare, or `self`; replaces `@borrows`). A call whose
result type is `ref mut T` is a place: `first(xs) += 1`, `counts.entry(k).or_default() += 1`.

**R7. Kept views** (B item 6, AN-24 to AN-26; A AN-07). A local list of views is itself a view and needs
nothing written. A struct that holds a view is itself a view (`[TYP-34]`): it lives no longer than what
its view fields point into, and its view fields' types already say so (`src: StrView`, `target: ref mut
Player`), so nothing more is written; `@view` may say it again and is checked (**15:** was "A struct
that holds a view is `@view`", which read as a required mark, while AN-22's `Parser` has none and
`[TYP-34]` made `@view` optional documentation). A class object is never a view, so a view field of a
class says what it points into: `from <let field>`, a `let` field of the same object whose contents
live in their own heap block (`String`, `Array`), or `from static` (text written in the program; on a
struct's field too, which then does not make the struct a view). `let` means nothing changes through
the field after `init` (06's C-8), so views into it stay valid for the object's whole life with no
tracking.

**R8. Views and changes, checked when compiling.**
- **(a) Positions** (A-1, A-2). A name for an element of a list reached through a local, a parameter or
  one-owner fields (`boss = ref mut enemies[0]`, `p = ps.get(0).unwrap()`) is a position: the list may
  grow while the name is used (its buffer is read again after the add); anything that may remove,
  insert, reorder, clear or replace the list ends the name: written here, in a callee whose summary says
  it may, or in unknown code given the list. Plain data written through another name is seen at the next
  use.
- **(b) Addresses** (C6). Every other view: kept in a struct or a list, reached through a `shared` or
  `weak` link, or into a value's own contents (a `String`'s text, an `Array` field's elements, an enum's
  payload), ends at any change that may reshape what it points into, through any name that may reach
  the same object. Two paths "may reach the same object" when they lead to values of one type and at
  least one goes through a `shared` or `weak` link or a kept view, unless the compiler knows they differ
  (different types, a fresh object, inside `if a is not b:`). "May reshape" is read from the function
  itself and from callees' first lines; unknown code may do everything its type allows. (14: (a) and (b)
  split the views between A's positions and C's addresses; see section 3, clash 2.)
- **(c) Loops** (A-3, the owner's understandable limitation; 14 for `while`). A list may not change
  while a `for` loop walks it, directly or through any call that may change it; the change goes after
  the loop (`retain`, `extend`, `Commands`). A counting loop `for i in 0..xs.len():` (its range fixed when
  it starts) may add to `xs` but not remove from it. A `while` loop that tests its position against the
  list's current length at every step may add and remove: it is not a walk, and each index is checked
  (area D's handler dispatch, AN-60).
- **(d) Temporaries** (row 5, A-5). A temporary lives until the last use of every view made from it,
  lists of views included. Returning such a view, or storing it where it would outlive the temporary,
  is refused with row 5's message.
- **(e) Guards** (row 4, B item 7, E item 3). A view made through a lock's guard (`g[0]`, `ref g[0]`, a
  slice of `g`) points into the guard and does not leave its `with` block; the same for `RwLock`'s
  `read()`/`write()` and for a `guarded by` field inside `with x.lock:`.

**R9. Several owners, nothing checked while running** (C6 to C10, E items 4 and 5; 14 lifted B's
refusal of two changeable kept views).
- Any owner of a `shared` value, and any `ref mut` name of it, may change it, written as any change is
  written (R5). A `thread_shared` value is never changed directly by any owner: only through `Atomic`,
  `Mutex`, `RwLock`, a channel, or a `guarded by` field under its lock.
- Kept `ref mut` views (in a struct or a list) may name one object, as `shared` owners do; R8(b) checks
  them. Only parameters keep the "different places" promise (R4).
- Nothing is checked while running: no access words, no borrow flags. `RefCell`, `Ref`, `RefMut`,
  `borrow`, `borrow_mut`, and 06's placeholders `RuntimeChecked[T]`, `ReadGuard`, `WriteGuard` are
  removed (C9, AN-49).
- `shared` counts are one-thread and never cross threads (E7015); `thread_shared` counts are atomic. A
  class object has an 8-byte header (a 32-bit count and a 32-bit type index).

**R10. Keeping an object alive under a view** (14, merging C7, D5 and D6, with one addition where D
meets E). A view of an object reached through a `shared`, `thread_shared` or `weak` link (a name bound
from one: `t = ref mut ws[i].target`, `if Some(p) = cam.target:`; **15:** and a parameter `x: shared T`
or `ref mut x: shared T`, whose life is the call, R4) must not outlive the object.
1. Nothing is paid when no line in the view's life can end an owner of the object's type: no write to
   a place of type `shared T`, `thread_shared T` or `weak T` of that type, no end of a local owner of it,
   no call whose first line reaches one. A one-owner object lent read-only cannot be ended during the
   loan (C7, D5).
2. Where the view's life contains a loop that makes an unknown call, and the class has no `drop` method
   of its own: a deferred-drop region around the loop: one add and one subtract of a per-thread counter
   per loop, nothing per step (D5, AN-62).
3. Otherwise a count pair around the view's life or around the call (C7, D5).
4. A view of a `thread_shared` object reached through a `weak` link always holds one atomic count for its
   life, as Rust's `Weak::upgrade` does: another thread may end the last owner at any moment, and a
   deferred-drop region belongs to one thread (14: D6 said only that the count is atomic).

**R11. Links** (D6, D7, AN-64). `weak T` reads as `Option` (`if Some(p) = cam.target:`), one test of the
target's header per read; its target is an object that lives on its own (a class object, or a value held
`shared` or `thread_shared`). `weak T set by f` is kept equal to the holder by the language: set when the
object is put into the holder's field `f`, cleared when it leaves, never assigned by the program, read as
a plain load; `f` is changed only through its own operations (`push`, `insert`, `remove`, `pop`, `clear`,
assigning the whole field). A list whose elements have one owner cannot form a cycle; the cycle warning
(L3001) is given only for a cycle of `shared` owners and names the link to make `weak`.

**R12. Function values** (row 6, D1 to D4, AN-56, AN-59). A capture not named is the real variable; its
place, an address when the function value provably ends first or a one-thread counted box otherwise, is
chosen when compiling and never changes what the program does. Calling a function value changes what
its body changes; one that changes its own state (`copied`, `owned` captures) is called through a name
that may change it. A stored function value is two words, `{code, state}`, copied out of its list or
field for the call; its body holds one count on its own state across a call that may remove it, when it
uses that state afterwards. `once fn()` is a function a call uses up. A lambda parameter's mode is what is
written on it.
**19:** (R18-02) **Where "called through a name that may change it" is checked.** The type `fn()` does
not say whether a value changes its own state, so the check is made where the value is known, and
where it is not, the call is taken to change it. A value passed to a parameter is known at the call:
`run(cb)` with `fn run(f: fn()):` is refused when `cb` changes its own copy (AN-54). A value stored in
a field or a list is not known to the code that later calls it, so **a call through a stored function
value is a change of the place that holds it**: the method says so on its first line (`changes
handlers`, or `mut self`) and calls through a `ref mut` name: `fn click(self) changes handlers:` with
`for ref mut h in self.handlers: h()` (AN-53), `fn depth_total(self) changes on_visit -> int:` (AN-62),
as Rust's `FnMut` is called through `&mut`. So row 6's copy spelling, `b.on_click(fn() with copied
score: score += 1)`, is accepted with AN-53's own `Button`, and the handler's own `score` changes only
through `click`, whose first line says so; a plain name never calls a function value that changes its
own state, which also keeps two tasks from running one such value at once (E7007). (Was unstated:
AN-53's `click(self)` called stored handlers through plain `h`, where D3 asks for a changeable name.)
**21:** (F-07) **What "a change of the place that holds it" means, and what it does not.** It decides
which name may make the call (one that may change the holder), because the call may change the
value's own state, which only the holder reaches. It does not make the list element or field an
argument of the call: the call copies the value's two words out first (D4), keeps no name for the
place they came from, and the state it changes lives in its own block. So R4 sees only the call's
arguments, and R8 (c) sees the list changed only if an argument may change it: the holder may be
given to the call, even as `ref mut` (AN-60's
`self.handlers[self.current](self)`, no compare), and the call may change or empty the list; and a
`for ref mut h in self.handlers:` walk may give each handler a read-only holder (`h(self)` with
`handlers: Array[fn(Ui)]`), since no handler can then change the list the loop walks (AN-60, point 6).
(Was unstated: 19's AN-60 note read `h(self)` as handing the handler "a second name for the list that
`h` is an element of, which R4 refuses", a reason that applied equally to the accepted
`self.handlers[self.current](self)`.)
**19:** (R18-06) A parameter that takes a `once fn()` is `owned f: once fn()`: the call uses the
function up, which only its holder can do, and the mode is on the name for every type (R4). A plain
`f: once fn()` is refused: "`call_once` uses `f` up, so it must hold it: write `owned f: once fn()`".

**R13. Interface values and downcasts** (D8, AN-67, AN-69, B's AN-22; 14 settled one rule).
`dyn I` as a parameter's type is a read-only view (`ref mut v: dyn I` may change it). **A `cast` of a
place is a place**: `cast p as D` names that place as a `D` and stops the program if it is not one;
`cast p if D` gives `Some` of it or `None`; the name bound from either is read-only unless written
`ref mut` (`pool = ref mut cast self.pools[k] as Pool[T]`; `if Some(ref mut pool) = cast self.pools[k]
if Pool[T]:`). One type-id compare. A generic method whose type parameter is only the type of a
`ref`/`ref mut` parameter gets one `dyn` entry. A non-`pub` function's bounds are worked out from its
body; a `pub` one writes them.

**R14. Threads** (E items 1 to 8).
- Scoped tasks need no words: every task's effects are checked against the other tasks and the parent
  until the join. Reductions (`total += e`, `-=`, `*=`, `|=`, `&=`, `^=`, `max`/`min`, `push`, `extend`,
  `add`, `m[k] op= e`) get one private accumulator per task, combined at the join in spawn order; nothing
  may read the place before the join. A local of one loop round is given to a task with `with copied n`.
- An unscoped thread, and a C callback that C keeps, say what they hold in the capture list:
  `with owned`, `with thread_shared`, `with copied`; `with shared` and `with weak` only where the function
  stays on this thread (a `threads(creator)` or `threads(main)` contract, checked by the trampoline).
- `guarded by lock` after a field's type (or a `static mut`'s) names the `Lock` that must be held for every
  read and change, on the thread that holds it, outside `init`.
- What may go to another thread is inferred from the words; `ThreadShared` and `Send` are inferred,
  written only in bounds, and `Send` is declared only by an `unsafe extend` author. `@sync` (06's
  `@thread_shared`) is removed.
- `static mut NAME: T = e` is a module variable that may change; a function that changes it says
  `changes NAME`; it is reached only by the main thread and its scoped tasks unless it is `guarded by` a
  `static` lock.
- Generators walk lists by position across `yield`; `async` stays reserved. C callbacks go through the
  checked trampoline (C holds a tagged `{slot, generation}` number, never an address).
  **19:** (R18-10) A generator whose `for` loop walks a list keeps the agreed limitation wherever the
  generator goes: the generator is a view of what it walks (R6: a method's result points into its
  object), so while it is still used the list may not change, in place or stored or passed (E3020,
  "`d.log.push(...)` changes `d.log`, which the generator `g` (line 3) still walks on line 5; change it
  after `g`'s last use, or walk by position in `replay`: `while i < len(self.log):`"). A generator
  meant to see the list as it is at each resume says so with that `while` loop (R8 (c)). (Was: a stored
  or passed generator walked a list that might change between resumes, so one `for` line meant "the
  list does not change" or "walk by position" by how a caller elsewhere used it.)
  **21:** (F-09) The first line says it, and a late use gets the code for a view. Such a generator writes R6's
  `from` with the list it walks, `gen fn replay(self) -> Generator[int] from log:`, so
  `g = d.replay()` is a view of `d.log` (R6) and R8 (b) ends it at a change of `d.log`; a `for` walk
  across `yield` of a list the first line does not name is refused (E3062): "`replay` walks `self.log`
  with `for`, so the generator it returns views `log`; write
  `gen fn replay(self) -> Generator[int] from log:`, or walk by position: `while i < len(self.log):`".
  The position walk's generator points into the object (R6's default), which a push to `log` does not
  end, so it sees the list as it is at each resume. A change while a `for` loop consumes the
  generator stays E3020 (a loop walks the list); a stored or passed generator used after a change is
  E3060, a view used after what it points into changed, as AN-46's `ks`: "`d.log.push(...)` changes
  `d.log`, which `g` (line 1) views (`replay`'s first line: `from log`) and uses on line 3; change it
  after `g`'s last use, or make `replay` walk by position: `while i < len(self.log):`". (Was: 19's
  E3020 text above, with the limitation read from `replay`'s body while R6's default made `g` a view
  of `d`, not of `d.log`.)

**R15. `unsafe`** (E item 9, AN-79, AN-81, AN-82). One word: before a declaration it says "whoever uses
this must vouch" (with `@safety`); before a block or an implementation, "this code vouches" (with
`# SAFETY:`). Inside an `unsafe fn` body every unsafe operation sits in its own `unsafe:` block (E3100).
**15:** so the two notes never sit on one thing: `@safety` (L3015, asked of every `unsafe fn`) goes on
the declaration, and a `# SAFETY:` note (W3012) on each block or implementation, not on an `unsafe fn`
line, where today's `[UNS-8]` asks for it and where it could only repeat `@safety` (AN-79's NEW had
both; today's W3012 on an `unsafe fn` becomes L3015's job).
`@no_alias_checks` (was `@static_safe`) forbids the run-time alias checks that are left: R4's compare and
`assert_disjoint`. No switch turns safety checks off.

**R16. Inferred facts at a package's edge** (14, settling A's AN-02 summaries against B's and D's rules).
Facts the compiler works out from a body (A's "only adds" summary of a `ref mut` list parameter, the read
set of a method with no `reads`, the effect summaries behind `static mut` and scoped tasks, a non-`pub`
function's bounds) never change what a line does; they only decide whether a line compiles. Inside a
package they are used freely. Across a package's edge, where a word exists to write the fact, only the
written word counts: a `pub` function's bounds are written (D, AN-69), and a `pub` method with no `reads`
is taken to read every field (B, AN-17). Where no word exists, the inferred fact is recorded in the
package's interface file, as today's effect sets are (`[EFF-4]`): A's "only adds" and E's effect
summaries; the toolchain reports a change to it as an interface change. **19:** (R18-04) what a call
changes in a `static mut` is now written on every first line that makes the change (R5), so of E's
summaries only those of scoped tasks stay inferred. **21:** (F-04) and one more: what the function
values a call may run change (D3, R5's exception for function values), since the type `fn()` has no
word for it; it is recorded in the interface file like the others. (Was: "only those of scoped tasks
stay inferred", which left a loop over `LOG` across `b.click()` with no summary that named `LOG`.)

**R17. Messages** (A-8, M-1 to M-4; D AN-57; E AN-72). Every refusal names what was refused in the
program's own names (`enemies[0]`, not `*enemies[0]`; `line.to_lower()`, not `_5`; `int`, not `i64`;
`fn()`, not `Callable0`), says why in one line, and gives the exact line to write. One mistake gives one
message, at the program's line, on the name that caused it.

### 1.2 What still runs while the program runs, compared with C

Every number is the area's, with the area's source; those marked PROVISIONAL were measured while other
agents ran and are for phase 4's timer to confirm.

| what runs | when | cost against C, and the source |
|---|---|---|
| bounds checks | indexing, as today | as today (not this rework's) |
| one compare of two positions or addresses (R4) | a call with two `ref mut` arguments (or a `ref mut` and a view; **21:** or a `MutSpan` of non-`Copy` elements and a view, one overlap test of two compares, F-03) not proved different | one compare and a branch never taken; last run measured 89.9 ms with it against safe Rust's `get_disjoint_mut` 87.6 ms (A, AN-03, citing `phase3b-containers.md:301-306`) |
| two compares (`assert_disjoint`) | where the program writes it | as today's `[DSJ-1]`; buys noalias (two spans of one list: 1.53x without, 1.00x to 1.07x with; A, AN-14) |
| the second pass of a loop then `retain` | only where AN-04's fusion conditions fail | 1.71x of one-pass C as written, 1.00x fused; safe Rust's idiom 1.77x (A, AN-04, `08-area-A/c/a4/runs_an04.txt`, PROVISIONAL) |
| one queued record per `Commands` change | ECS systems that destroy, create or add | applied after the system, as Bevy (A, AN-06) |
| a count +1 / -1 | a `shared` owner made or ended; atomic for `thread_shared` | Rust's `Rc` / `Arc`; re-pointed shared targets 1.13x to 1.25x of plain C, safe Rust's `Rc` 1.48x to 1.57x (C, AN-52 (a), PROVISIONAL) |
| nothing, or a deferred-drop region, or a count pair (R10) | a view of an object behind a `shared`/`weak` link | nothing where no line can end an owner (the 13.5% to 13.9% count pair of AN-51 is gone); region: 0.944 of plain C against 1.283 for a count pair per step (D, AN-62, PROVISIONAL, spread 29% to 37%) |
| one test of the target's header | each read of a `weak` link | Rust's `Weak` on churn 0.919 and 0.935 of plain C (D, AN-63) |
| **19:** one add and one subtract of the target's weak count | a `weak` link copied with `copied` (R2): **21:** (F-01) always for a copy stored (a field, a list, a capture, an `owned` argument), returned or outliving the region; for a local copy only where something in its life could end the target's last owner and no deferred-drop region outlives it | Rust's `Weak::clone` and its drop; none in AN-62's walk (the region covers it, `19-fixer/c/an62_defer_asan_output.txt`) |
| one store | putting an object into, or taking it out of, a `set by` holder | tree workload 1.025 and 1.149 of plain C in the same layout, safe Rust's index arena 1.519 and 1.357 (D, AN-63, last run's runs) |
| one flag test per call, one store per change of a `reads` field | a `@cached` method | the hand-written C cache (B, AN-21); an acquire load on objects several threads reach (E, section 5) |
| one allocation per captured variable instance | a capture whose function value may outlive the variable | 1.018 and 1.052 of plain C on last run's callback workload (D, AN-53) |
| two words copied out of the list | each call of a stored function value | dispatch 1.045 and 1.037 of plain C, safe Rust 1.490 and 1.472 (D, AN-60) |
| one count on a handler's own state | a handler that removes itself and then uses its state | rare path (D, AN-60) |
| one type-id compare | each `cast` | 1.223 to 1.244 of plain C, safe Rust's `downcast_mut` 1.571 to 1.635 (B, AN-22, PROVISIONAL) |
| a lock | where the program takes one (`with x.lock:`) | the lock C takes (E, AN-74) |
| nothing beyond C's per-thread counter | scoped tasks and reductions | 1.00 against an atomic add per word 10.22x (E, AN-71, last run's numbers, PROVISIONAL) |
| the checked trampoline | each C callback | 1.02x to 1.05x, today's 1.47x (E, AN-78) |
| the generator protocol | each `yield` | position re-read 1.18 of a plain C loop, the same as holding an address (E, AN-77) |
| a copy, an allocation | where a line says `copied`, or `owned`/`shared` puts a value in its own block | Rust's `.clone()`, `Box::new`, `Rc::new` |

Gone, compared with today's Ember: the access words checked on class fields (`[EXC-19]`), `RefCell`'s
borrow flag, the count pair per loop step on views of shared objects (AN-51), `@sync`'s frozen fields,
the 24-byte class header (now 8 bytes), and today's 1.47x trampoline.

### 1.3 What stays refused

Each with its message in the answer named; every message names the program's own names, the reason in
one line, and the line to write (R17).

| refused | message's line to write | answers |
|---|---|---|
| using a name after it was handed over | `snapshot = copied items` / `snapshot = shared items` (row 9) | AN-34, AN-36, AN-43, AN-44 |
| a change through a plain name (parameter, loop name, pattern name, plain `self`); **19:** `=` on a plain view name or a plain parameter (R2, R4); **21:** `=` on a name holding a view a call gave, or a view name given to a `ref mut` parameter (R2, R4) | `ref mut c: Counter`, `for ref mut p in players:`, `State.Walking(ref mut path):`, `fn open(self) changes angle:`; **19:** `m = q.name`, `count = n`; **21:** `m = q.name.trim()`, `r = ref mut first_even(...)`, `best2 = longer(best, b.as_span())` | AN-20, AN-19, AN-38, AN-01, AN-13; **19:** AN-11, AN-28; **21:** R2, R4 |
| a body that changes or reads what its first line does not name, or names what it does not change (**19:** its calls included, for methods and free functions alike, R5; **21:** a module variable a lambda changes, named by the function that writes it, R5) | the corrected first line | AN-17, AN-31, AN-76 |
| **19:** a call through a stored function value from a name that may not change it (R12) | `fn click(self) changes handlers:` with `for ref mut h in self.handlers:` | AN-53, AN-62 |
| a list changed while a `for` loop walks it (**19:** a generator's `for` loop included, R14; **21:** while a loop consumes the generator, and through a call that runs a stored handler which changes the list, R5, AN-76; a stored generator used after the change is the view row below, F-04, F-09) | `retain` / `extend` / `Commands` after the loop | AN-04, AN-05, AN-06, AN-17, AN-42, AN-48, AN-60, **19:** AN-77; **21:** AN-76 |
| a removal or reorder while an element name is still used | use the name before, or name the element again | AN-01, AN-02, AN-13 |
| a view used after a change that may reshape what it points into | use it before, or `copied` | AN-48, AN-49, AN-50, AN-52, AN-26; **21:** AN-77 (a stored generator, `from log`), R4 (a view put in through `ref mut`) |
| **21:** a generator whose `for` walks a list across `yield` without `from <list>` on its first line (E3062, F-09) | `gen fn replay(self) -> Generator[int] from log:`, or `while i < len(self.log):` | AN-77 |
| one place given twice to two `ref mut` parameters (known when compiling); **21:** one list given as a `MutSpan` of non-`Copy` elements and as another view of one call (R4, F-03) | two different elements, or a one-element version; **21:** `old = copied names`, then `refill(names.as_mut_span(), old.as_span())` | AN-03, AN-18; **21:** R4 |
| a view of a temporary returned or kept too long | `return String(word)` (row 5) | AN-07, AN-09 |
| **21:** a view of the callee's own locals put into the caller's view through `ref mut` (E3063, F-02) | `best = other`, or return a list of your own | R4 |
| a view leaving a lock's `with` block | return a copy, or do the work inside the block (row 4) | AN-30 |
| `mut` in a pattern; `mut x` on a parameter other than `self` (**21:** F-10: `mut self` stays, R5, open point 7); `x: ref mut T` | `State.Walking(ref mut path):` (row 8), `ref mut x: T` | AN-38, AN-27, AN-11 |
| a type that contains itself without `owned` | `left: Option[owned Tree]` | AN-41 |
| `owned` where it can only copy, before a value, or on a class type | remove `owned`; `with copied score` | AN-43, AN-47, AN-40 |
| a bare interface type | `Array[owned dyn Shape]` or `Array[shared dyn Shape]` | AN-66 |
| a function value keeping someone else's place it may outlive | `with shared count` / `with owned xs` / a scope | AN-55, AN-57 |
| a function that gives a capture away, where it may run twice | `owned f: once fn()` (**19:** was `f: once fn()`, R12) or `consume(copied s)` | AN-59 |
| assigning a `set by` link; lending its holder as a changeable list | `root.children.push(leaf)` | AN-63 |
| `virtual` in a class that cannot be subclassed | `open class Door:` | AN-70 |
| threads: a name an unscoped thread uses with no capture word (E7004); a one-thread owner reaching another thread (E7015); a `thread_shared` value changed directly (E7014); a guarded field used without its lock (E7013); two tasks changing one place (E7007); a thread-bound value sent away (E7008); a `static mut` reached from another thread (E7012); chunks of a parallel loop writing one place (E7010); **21:** `thread_shared` on a value not created `thread_shared` (E7017, F-08) | the capture word, `thread_shared`, `guarded by`, `with x.lock:`; **21:** `names = thread_shared Names()` | AN-71 to AN-76, AN-78 |
| an unsafe operation outside `unsafe:`, an `unsafe fn` body included (E3100) | an `unsafe:` block with `# SAFETY:` | AN-79 |
| the removed words | `owned T` for `Box`, `shared T` for `Shared`, `thread_shared T` for `SyncShared`, `weak T` for `Weak`, `if Some(p) = w:` for `upgrade`, `copied x` for `.clone()`, `StrView` for `str`, `CStrView` for `cstr`, `ref mut T` for `Mut[T]`, `from x` for `@borrows`, `@no_alias_checks` for `@static_safe`, `@fp(allow_fma)` for `@fp(contract)`, the capture list for `owned fn`, `cast` for `as?`/`as!`; `RefCell` and `@sync` with the line of AN-49 and AN-80; `@no_runtime_checks` with AN-82's | AN-40, AN-45, AN-47, AN-49, AN-54, AN-65, AN-80 to AN-83 |

### 1.4 Library names the NEW code uses (**19:**, VOC)

Agent 15 found 26 names in the NEW blocks with no row in the vocabulary (`15-vocabulary-review.md`
section 5). Each is a name the spec or the standard library already has, used with its spec meaning;
none says who owns or changes a value, so none is a word of the memory model, and none is added or
changed here. Each was checked against the spec (`docs/spec-source/Ember_v0.9.10_Hardened_35.md`, line
numbers below) so that no NEW block uses a name Ember does not have.

| name | what it is (its spec meaning, unchanged) | where the spec says it |
|---|---|---|
| `len`, `println`, `panic`, `sorted`, `max`, `abs` | prelude functions; `len(x)` is `x.len()`, `sorted(it)` returns a new sorted array | the prelude table, 1538; `[STD-26]` 4140 (`sorted`, 4154) |
| `iter` | `Iterable.iter()`, what a `for` loop calls | `[TYP-43]` 1260 (1313) |
| `collect`, `inspect` | iterator adapters (`collect[C]()`; `inspect` runs a function on each value); in AN-67 `inspect` is the program's own method | `[STD-19]` 4365 (4370, 4444); `[CLO-15]` 2129 (2135) |
| `split_whitespace`, `to_lower`, `trim` | text methods | `[TXT-10]` 4236 (4238-4241) |
| `get_or`, `entry`, `or_default` | `Map` methods; R6 names `counts.entry(k).or_default()` as a place | `[STD-16]` 4306 (4310-4313) |
| `fetch_add` | `Atomic[T]`'s add; a type R5's exception names | `[THR-14]` 3517 (3520) |
| `join` | `JoinHandle.join()`, waits for a thread | `[THR-10]` 3503 (3506) |
| `as_span`, `as_ptr` | a list as a `Span`; a raw pointer from a span | `[SPN-1]` 2533 (2536); `[SPN-8]` 2550 |
| `with_capacity` | a constructor that reserves room | `[SOA-3]` 3691 (3692); 2927 |
| `c_int` | `std.ffi`'s name for C's `int` | 4185, 4718 |
| `World`, `Component` | `std.ecs`'s world type and the interface `@derive(Component)` implements | the ECS example 3787; `[ECS-7]` 3841 |
| `Vec3` | `std.math`'s vector | `[STD-21]` 4500; 251 |
| `Neg`, `Add` | operator interfaces | 889; `[TYP-43]` 1260 (1198, 1330) |

## 2. Rulings on every new word

### 2.1 The four new words from the areas

| word | area | ruling | final | why |
|---|---|---|---|---|
| `Commands.create` | A | **accepted** | `Commands.create()` (a library method, not a keyword) | The agreed "delete after the loop" needs its other half inside an ECS system: today `Commands` has `destroy`, `add`, `remove` only (`std/src/ecs.em:1219-1237`) and `create` is E2072 (AN-06). It keeps `World.create()`'s name (`std/src/ecs.em:216`), so one operation has one name. The id is real at once, so `add(n, ...)` can name it; the entity exists when the queue is applied (Bevy's `Commands::spawn`). The owner's test: in `n = q.commands().create()`, `n` is an entity id, a 32-bit `Copy` value that changes no entity: "none (default)" is allowed. |
| `static` | B | **accepted** as a new position of two existing words | `from static`, after a view field's type: `kind: StrView from static` | Both words keep their meanings: `from` "points into" (07, B), `static` "lives for the whole program", the static region of `[LT-3]`. The field then says what it points into, as every field of views must (R7); without it the only spelling is `kind: String`, an allocation per object. One position, one meaning; no new spelling. |
| `guarded` | E | **accepted** | `guarded by <lock>`, after a field's or a `static mut`'s type | Which lock protects which field is what the program means, and a reader must see it on the field (the owner's "what the programmer means stays written"); last run inferred it. `changes` says what a method changes, not under which lock; `Mutex[T]` says it for one value but forces several fields into a separate struct (AN-74's translation). It reads as English, sits on the field it describes, means one thing for every type, and `by` keeps 07's meaning in `set by` ("by this"). Java's `@GuardedBy` and Clang's `GUARDED_BY` say the same thing in the same place. |
| `Lock` | E | **accepted** | `Lock` (a `std.sync` type), held with the bare `with x.lock:` | A lock that guards several fields has no value of its own; `Mutex[void]` would read `with self.guard.lock():`. `with x.lock:` is the bare `with` form of vocabulary settlement 10 ("hold this until the block ends"), so no new meaning of `with`. Python's `threading.Lock`. `Mutex[T]` stays for one value behind a lock (`with g = m.lock():`): two types, each read right where it sits. |

No answer had to be rewritten for a rejected word: none was rejected.

### 2.2 The words 07 planned, as the areas used them (confirmed)

| word | 07's meaning | used by | ruling |
|---|---|---|---|
| `thread_shared` | one of several owners on any threads; atomic count; never changed directly | E (AN-71, AN-74, AN-75, AN-80), C (named), D (D6) | confirmed; R3, R9, R14 |
| `weak` | a link that does not keep its target alive; reads as `Option`; replaces `Weak[T]`, `Weak(x)`, `upgrade` | D (AN-60 .. AN-65), C (named) | confirmed; R11; the read rule of R10 point 4 added |
| `set by` | the language keeps this `weak` link equal to the holder | D (AN-62, AN-63), C (AN-41's message) | confirmed with D7's rules; R11 |
| `@cached` | a method's result kept, cleared by a change to a field its `reads` names | B (AN-21), E (threads ruling) | confirmed; across threads E's once-flag replaces B's refusal (clash 6) |
| `@fp(allow_fma)` | today's `@fp(contract)` | E (AN-83) | confirmed |
| capture-list words, `changes` on a free function, `from` after a field's type, `owned T` on a foreign type | 07 section 4 | D, E, B, C | confirmed |

### 2.3 Words the areas removed or kept that the vocabulary had left open

| word | vocabulary.md said | ruling here | by |
|---|---|---|---|
| `RuntimeChecked[T]`, `ReadGuard`, `WriteGuard`, and their `read`/`write` | placeholders for `RefCell`, `Ref`, `RefMut`, `borrow`, `borrow_mut`; "area C may remove the type" | **removed** (`RwLock`'s `read()`/`write()` untouched) | C (AN-49, C9); B's AN-30 and E's AN-81 rewritten (clash 5) |
| `@thread_shared` | placeholder for `@sync`; "area E may remove the marker" | **removed**; the value says `thread_shared`, each field says `guarded by` or `Atomic` | E (AN-80) |
| `ThreadShared` | the property, renamed from `Sync` | **kept**, inferred, written only in bounds and messages | E (AN-80) |
| `Send` | "area E may stop programmers writing it" | **kept**, inferred; written only in `unsafe extend T implements Send:` | E (item 5) |
| `get` for `upgrade` | placeholder | **dropped** (a `weak` link is read directly) | 07, D (AN-65) |
| `Foreign[T]` for `ForeignBox[T]` | placeholder | **dropped**: `owned T` on a foreign type | 07, C (AN-40) |
| `Box[T, A]` (with an allocator) | "needs a place in the new form" | **open** for the owner: no annoyance needs it, and no word was invented for it (section 9) | C (AN-40) |

## 3. Clashes between areas, and how each was settled

| # | topic | the two (or three) answers | settled | why | rewritten |
|---|---|---|---|---|---|
| 1 | what `x = <part>` binds | A-1: `boss = enemies[0]` is a read-only view of the element (a number is copied); C1/C2: `n = p.name` hands the field over (a partial move), and a `Copy` struct part is copied (E3045 when changed) | **R2: a name bound to a part is a read-only view, for fields and elements alike; a part with no fields (a number, a `bool`, a `weak` link) is copied; a part that is an owning link views the object it links to; a plain view name may be bound again; a stored part is handed over** (the `weak` and owning-link cases keep D's AN-62 and C's AN-51 as written) | Row 8 already binds a pattern's part as a read-only view with no word (plain `path`), and row 5 binds `word` as a view of a temporary: A's rule is the agreed rows' rule, extended. It removes AN-35's annoyance (`n = p.name` then `keep(p)` now compiles when `n` is used first) instead of keeping it, and it never copies silently, where C2's struct copy did (row 6: "a copy is only made when named"). The advice in both areas' messages was the same (`ref mut` or `copied`), so only C's reason line changes. C2 stays for whole names (`b = a` of a `Copy` struct). | C1, C2 (notes); AN-35 (NEW, test, refusal); AN-44 (the E3045 refusal becomes A's E3021 message) |
| 2 | element names and the reshape rule | A-2: an element name survives an add to its list (the buffer is read again); C6: a view into a list ends at any reshape, `push` included | **R8 (a)/(b): element names of a list reached through locals, parameters and one-owner fields are positions (A); every other view is an address (C6)** | Both are compile-time and memory-safe; they describe different views. A position finds its element again from a path no one else can re-point, so an add frees nothing it uses; a kept view or a view through a `shared`/`weak` link holds an address, which an add may free. With the split, A-2's third bullet (a view into an element's own `String` or `Array`) is C6's rule, so the two read as one. | A-1, C6 (notes) |
| 3 | two changeable kept views of one object | C, AN-52 (b): watchers keep `ref mut` views that may name one player, checked by C6, 1.00x to 1.06x of C; B, AN-24: two changeable views of one registry in one list stay refused, "as in Rust" | **C's: kept `ref mut` views may name one object (R9); only parameters keep the "different places" promise (R4)** | In this model `shared` owners already change one object with no run-time check (C6, AN-48); a kept `ref mut` view is the uncounted form of the same thing and is checked by the same rule (R8 b). B's refusal was Rust's exclusivity, which the model no longer needs for memory safety, and C measured the accepted form at C's speed. Parameters keep noalias, which is where the speed comes from (A-4). | AN-24 (the first refusal) |
| 4 | the first line of `get_mut[T]` and the downcast | B, AN-22: `fn get_mut[T](self, e: int) changes pools -> ref mut T:`, `pool = ref mut cast ... as Pool[T]`, and `Map[u128, owned AnyPool]`; C, AN-42: `-> ref mut T from self changes pools:`; D, AN-67: `fn get_mut[T](mut self, k, i) -> ref mut T:` with `if Some(ref mut pool) = cast ... if Pool[T]:` | **R5's order (parameters, `reads`, `changes`, `->`, `from`); the method says `changes pools`; R13's one rule for `cast` of a place, of which both `cast` spellings are uses; `owned dyn AnyPool`** | Rows 3 and 7 put the clauses before `->` (`reads renderer -> Array[String]`), and B's `from` sits on the result it describes, after the type. `changes pools` says the narrower truth (row 3) where `mut self` says "any field". `from self` is the default for a method (R6), so it is not written. `AnyPool` is an interface, and a bare interface name as a type is refused (D8), so B's field type lacked `dyn`. `cast p as D` (stop if not) and `cast p if D` (test) are two uses of one rule, each fitting its intent. | AN-22 (field type, test row), AN-42 (first line, note), AN-67 (first line, test row, note) |
| 5 | `RuntimeChecked[T]` | C removes it with `RefCell` (C9, AN-49); B's AN-30 keeps a rule and a message for its `read()`; E's AN-81 message names it as the run-time alias check | **removed** | No annoyance needs a run-time-checked cell once C6 checks shared changes when compiling, and the vocabulary allowed area C to remove it. The run-time alias checks left in the model are R4's compare and `assert_disjoint`, so `@no_alias_checks` forbids those. | AN-30 (rule, refusal), AN-81 (refusal) |
| 6 | `@cached` on an object several threads reach | B, AN-21: refused ("keep the mesh on one thread"), "area E may replace this refusal with a once-flag"; E, section 5: a once-flag, and a refusal only where a `reads` field changes while other threads hold the object | **E's** | 07 asked area E; E's answer keeps the result correct (filled once under `Once`, an acquire load and a branch when filled) and refuses only the stale case. | AN-21 (refusal) |
| 7 | a change through a plain name that reaches a `shared` object | C, AN-51: `ws: Array[Watcher]` read-only, `t = ref mut ws[i].target` changes the player; "works under either ruling" | **B's rule: a plain name never changes anything, an object behind a `shared` link included; the parameter is `ref mut ws`** | The brief allows a name with no word only where it never changes anything; B's model (item 3) and 06's `let` rule (C-8) read it the same way. The cost is unchanged (the C is the same). | AN-51 (NEW, the paragraph after it, test row) |
| 8 | the error code for a first line that does not match its body | C: E3023 ("`bump` changes `self.n`, but its first line does not say so"); E: E2105 ("a placeholder for area B's check"); B: no code | **E2105 for every mismatch between a `changes`/`reads` clause (or its absence) and the body; E3023 for a change through a plain parameter, loop name or pattern name** | One check, one code (AN-84's rule). E3023 keeps today's meaning, "this name only reads", now with the line to write. | AN-49 (code) |
| 9 | a word at the call site for an argument given to `ref mut` | B (section 6 point 1) and D (section 5) asked | **none: the callee's first line says it, and the call is checked (R4)** | Row 3's own example calls `self.note(...)` with no word; today's `[FN-2a]` says the call site writes no mode; the owner's test asks about names bound in a line, and an argument binds none. Listed for the owner (section 9). | — |
| 10 | keeping an object alive under a view | C7: no count where no line in the view's life can end an owner, else a count pair; D5/D6: a deferred-drop region where a loop makes unknown calls, else a count pair; a `weak` link to a `thread_shared` target "uses the atomic count" | **R10, one rule in four points; point 4 added** | C7 and D5 are the same idea for `shared` and `weak` targets; D's region is the cheaper protection where C7 would pay a count pair per step. The region is per thread, so a view of a `thread_shared` object reached through `weak` must hold an atomic count, as Rust's `Weak::upgrade` (another thread may end it at any moment); D6's sentence left that open. | C7, D5 (notes) |
| 11 | a `Map[String, V]` written through a `StrView` key | A, AN-12 asked area C whether `counts[w] = ...` may make the key's `String` with no word | **no word: a new key is stored as the map's own `String`, made from the text** | Keys never change in a map, so no line can tell the stored copy from the view; requiring `counts[String(w)]` would cost an allocation per update, existing keys included, which is what A's point 7 holds against Rust's `entry(w.to_string())`. Python stores its own key too. | AN-12 (point 4 note) |
| 12 | changing a list during a dispatch loop | A-3: a list may not change while a loop walks it, and a counting loop may not remove; D, AN-60: `while self.current < len(self.handlers):` with handlers that add and remove | **R8 (c): A-3 governs `for` loops; a `while` loop that tests its position against the current length at every step may add and remove** | The owner's limitation is about a loop walking a list; D's loop is ordinary position code the program writes, with every index checked, and it is Python's own list-iterator behaviour. A's counting loop is refused for removal because its range is fixed when it starts (skips, then runs past the end), which a `while` loop testing the length does not do. | A-3 (note) |
| 13 | inferred facts across a package's edge | A, AN-02: "only adds" inferred and written into the interface file; B: a `pub` method with no `reads` reads every field; D, AN-69: inferred bounds stop at `pub`; E: effect summaries carried to callers | **R16** | Where a word exists, the written word is the interface (B's `reads`, D's bounds); where none exists, the inferred fact goes in the interface file as today's effect sets do (`[EFF-4]`), so no meaning is decided by the compiler and the cases differ only by whether the language has a word. | — |
| 14 | `@thread_shared` | 06 renamed `@sync` to it; 07 kept it as a placeholder; E removed it | **removed** (E) | The value says `thread_shared` where it is shared and each field says how it changes; a class mark would say it a second time. | — |
| 15 | `class` with one owner | B (item 2) and E (AN-74, AN-71) depended on area C ruling 06's condition on the `class` row | **consistent: C8 rules one owner unless `shared`** | B's removal of run-time access words and E's claim-free parallel loop both rest on it, and C confirms it. | — |
| 16 | `worker` in AN-76 changes `RESULTS` with no `changes RESULTS` (inside area E) | E's rule 2 for `static mut` ("a change not named is refused") against its own NEW | **`fn worker(x: int) changes RESULTS:`** | One rule for every module variable; the effect summary of callers still carries it. | AN-76 (NEW, test rows) |

**19:** two settlements are amended by 19's fixes, the rest stand: clash 1 (R2: a `weak` link is no longer
copied from a part, R18-13; a plain view name is no longer bound again, R18-03; **21:** nor is a name
holding a view a call gave, F-06) and clash 9 (unchanged,
but R18-16 shows it fails the owner's test read literally: section 9, point 6).

## 4. Every rewrite made to an area's text

Each is marked **14:** in place in sections 10 to 14.

| where | what was there | what it is now | clash |
|---|---|---|---|
| A, rule A-1 | element names only | note: extended to every part by R2; the element-name/position half of R8 | 1, 2 |
| A, rule A-3 | `for` loops | note: a `while` loop testing the current length may change the list (R8 c) | 12 |
| A, rule A-4 | two `ref mut` parameters | note: also a `ref mut` and any other view argument (R4) | 3 |
| A, AN-12 point 4 | "see section 7 for the combiner" | the ruling of clash 11 | 11 |
| A, AN-16 point 4 | "no names are bound in this answer" | the owner's test applied to every name in every line the messages give | section 6 |
| B, item 2 | "Area C decides what a `shared` object keeps" | note: C6, nothing checked while running | 15 |
| B, AN-21 point 6 | refusal for a value sent to another thread | E's once-flag ruling | 6 |
| B, AN-22 NEW and test | `pools: Map[u128, owned AnyPool]` | `owned dyn AnyPool` | 4 |
| B, AN-24 point 6 | two changeable views of one registry refused | accepted (R9), with what stays refused | 3 |
| B, AN-30 rule and point 6 | `RuntimeChecked`'s `read()`/`write()` | removed (AN-49) | 5 |
| C, rules C1, C2, C6, C7 | partial hand-over by `=`; struct part copied; reshape ends every view; count pair | notes pointing at R2, R8 (a)/(b), R10 | 1, 2, 10 |
| C, AN-35 | `n = ref p.name`; E3042 refusal | `n = p.name` (a view, R2); the refusal moves to using `n` after `keep(p)` | 1 |
| C, AN-42 | `fn get_mut[T](self, i: int) -> ref mut T from self changes pools:` | `fn get_mut[T](self, i: int) changes pools -> ref mut T:` | 4 |
| C, AN-44 point 6 | E3045 "changes a copy of `p.pos`" | E3021 "`pos` reads `p.pos` and cannot change it" (R2) | 1 |
| C, AN-49 point 6 | `E3023` for a first line that does not say the change | `E2105` | 8 |
| C, AN-51 | `fn step(ws: Array[Watcher], ...)` and "works under either ruling" | `fn step(ref mut ws: Array[Watcher], ...)` | 7 |
| D, rule D5 | protection by region or count pair | note: R10, with point 4 for `thread_shared` through `weak` | 10 |
| D, AN-67 | `fn get_mut[T](mut self, k: u128, i: int) -> ref mut T:` | `fn get_mut[T](self, k: u128, i: int) changes pools -> ref mut T:` | 4 |
| E, AN-76 | `fn worker(x: int):` changing `RESULTS` | `fn worker(x: int) changes RESULTS:` | 16 |
| E, AN-81 point 6 | E4030 naming `RuntimeChecked` | E4030 naming R4's compare | 5 |
| E, AN-84 points 3 and 4 | "no NEW names (error codes only)" | the programs the messages lead to, in NEW spelling, with the owner's test | section 6 |

No "means exactly" translation had to change: every rewrite either changes only a refusal's text, or
gives NEW code whose meaning the area's translation already shows (AN-35: `an35_tr_ref.em`; AN-42:
`an42_old_box_pools.em`; AN-51: `share_c.c -DMODEL=3`; AN-67: `m67_class_pools.em`; AN-76:
`c/an76_static_mut.c`, where `worker` already takes the lock and pushes). The only new NEW code is
AN-84's, whose translation is E's `means/an84_means.em` (pasted there, `exit=0`).

## 5. Mismatches found by the old-code re-checker

None. Agent 13 re-ran `target/redesign3/bin/ember.exe check` on all 245 OLD and "means exactly" files of
the five areas and found every output, error code, warning and exit code equal to what the reports paste
(`RUN3/13-old-code-recheck.md`; the only difference, the thread number in three stack-overflow lines of
AN-41, changes on every run). Nothing to correct.

## 6. Items the automatic check found incomplete

| answer | problem | completed |
|---|---|---|
| AN-16 (A) | the owner's test applied to no name | point 4 now applies it to every name in every line that the area's messages tell the programmer to write (section 10, AN-16) |
| AN-84 (E) | the owner's test applied to no name | points 3 and 4 now give the programs the new codes' messages lead to, in NEW spelling, and apply the test to every name in them (section 14, AN-84) |

## 7. The agreed rows, unchanged

| row | agreed (`inputs/JOB.md`) | where it is in this file | unchanged |
|---|---|---|---|
| 3 | `fn note(self, msg: String) changes log:`, checked against its body | AN-17 (B), section 11; R5 | yes |
| 4 | `return ref g[0]` inside `with g = scores.lock():` refused: "this points into the locked list, which can change once the lock ends; return a copy, or do the work inside the `with` block" | AN-30 (B), section 11; R8 (e) | yes |
| 5 | `word = line.to_lower().trim()` keeps the lowered string until `word`'s last use; returning such a view refused: "return `String(...)` to hand back your own copy" | AN-08, AN-09 (A), section 10; R8 (d) | yes |
| 6 | `b.on_click(fn(): score += 1)` changes `game`'s real `score`; a copy only when named: `fn() with copied score:` | AN-53, AN-54 (D), section 13; R3, R12 | yes |
| 7 | `fn load_assets(self) reads renderer -> Array[String]:`, checked against its body and at the call | AN-31 (B), section 11; R5 | yes |
| 8 | `State.Walking(ref mut path):` stays; plain `path` then `path.pop()` and `State.Walking(mut path):` both get "to change that list through `path`, write `State.Walking(ref mut path):`" | AN-37, AN-38 (C), section 12; R2 | yes |
| 9 | `snapshot = copied items` copies; `snapshot = items` hands the list over; the error names `copied` and `shared` | AN-34 (C), section 12; R1 | yes |

R2 (clash 1) is row 8's plain `path` rule applied to every part; it changes nothing in row 8 or row 5.
`target/redesign3/14-combiner/build.py` checks that each agreed spelling above appears in this file.

## 8. Error codes the combined model uses

New or changed codes from the areas, one meaning each (AN-84's rule); messages are in the answers.
**19:** (R18-09, R18-08) rewritten so that one mistake has one code and one code one meaning, and so
that every code a NEW refusal gives is listed here (the list was made from
`target/redesign3/18-readability/codes.txt`, every refusal in a "Still refused" point, and checked
again after 19's changes). Two codes are new and free: neither `E0113` nor `W3013` appears in the
compiler's sources (`compiler/**/*.rs`) or in the spec. **21:** E7017 is new and free too (the
compiler's sources and the spec use E7000 to E7006, E7010, E7011 and E7020 only; grep over both). "Today's" means the meaning today's compiler
gives the code.

| code | means | answers |
|---|---|---|
| E0005 | `async` is reserved (today's code; the text gives the line to write) | E (AN-77) |
| E0104 | an attribute that is gone, with the one to write (`@sync`, `@static_safe`, `@no_runtime_checks`, `@fp(contract)`) | E (AN-80 .. AN-83) |
| E0109 | `owned` before a value or before `fn` (`b = owned a`; `owned fn`): the message names what to write on each captured name | C (AN-47), D (AN-54), E (AN-78) |
| **E0113** (new, **19:**) | `mut` on a name other than `self` (a pattern name, a parameter, a lambda parameter): it says "may change" and no word says whose (row 8). **21:** (F-10) `mut self` (R5, AN-70) and `static mut` (a declaration, R14) keep `mut` and are not this code (open point 7) | C (AN-38), B (AN-20, AN-27), A (AN-11) |
| E1010 | a name the program may not use: gone from Ember (`Box`, `RefCell`, `Sync` as a property), or written with a word instead (`a.clone()` is `copied a`) | C (AN-40, AN-45, AN-49), E (AN-80) |
| E1030 | a second `main`: statements at file level in a file that has `fn main` (today's code; the text gives `static mut`) | E (AN-76) |
| E2085 | `owned` on a parameter whose type is known to be `Copy` | C (AN-43) |
| E2103 | `self` given another object | C (AN-50) |
| E2105 | a `changes`/`reads` clause, or its absence, does not match the body, the calls it makes included (R5); **21:** a lambda's change of a module variable counts for the function that writes it (R5, F-04) | B (AN-17, AN-31), C (AN-49), E (AN-76); clash 8 |
| E2200 | a type that contains itself without `owned` (today a checker crash) | C (AN-41) |
| E2221 | a view into a generator's own frame held across `yield` (today's meaning) | E (AN-77) |
| E3013 | a hand-over out of a name or place that does not own the value (`match owned self.state`; `v.push(template)` from a plain parameter) | C (AN-35, AN-37) |
| E3020 | a list changed while a `for` loop walks it, directly, through a generator the loop consumes, or through a call that runs a stored function value (R8 (c), R14, R5) (**21:** was also "or while a generator that walks it is still used", which is E3060's mistake, F-09) | A (AN-04, AN-05), C (AN-42, AN-48), E (AN-76, AN-77) |
| E3023 | a change through a name that only reads: a plain parameter, loop name, pattern name, part name, `p` from `get`, a value made when compiling, plain `self`; `=` on such a name (R2, R4); **21:** `=` on a name holding a view a call gave, and a view name given to a `ref mut` parameter (R2, R4, F-02, F-06) | A (AN-11), B (AN-20, **21:** AN-28), C (AN-38, AN-44, AN-46) |
| E3040 | a name used after it was handed over or ended | C (AN-34, AN-36, AN-39, AN-43, AN-44), E (AN-71, AN-75) |
| E3060 | a view used after what it points into was ended, handed over, replaced or reshaped (R8 (b)), or after the local it views ended; **21:** a stored or passed generator included (R14, F-09), and a view put in through `ref mut` (R4, F-02) | C (AN-35, AN-37, AN-46, AN-50, AN-52), E (AN-73, **21:** AN-77) |
| E3062 | (today's code, **21:** listed, F-09) a result points into something its first line does not name with `from`: a view returned from a source `from` leaves out, or a generator whose `for` walks a list across `yield` without `from <list>` (R6, R14) | B (AN-22), E (AN-77) |
| E3063 | a view, or a value holding views, kept or returned where it would outlive what it points into; **21:** a view of the callee's own locals put into the caller's view through `ref mut` (R4, F-02) | C (AN-52) |
| E3100 | an unsafe operation outside an `unsafe:` block, an `unsafe fn` body included | E (AN-79) |
| E4030 | a run-time alias check (R4's compare, `assert_disjoint`) inside `@no_alias_checks` | E (AN-81, after clash 5) |
| E5013 | a variadic further argument C would promote (was E5015) | E (AN-84) |
| E7004 | an unscoped thread or kept callback uses a name with no capture word | E |
| E7007 | two tasks of one scope change one place other than by a reduction or under a lock | E |
| E7008 | a thread-bound value (a guard, a `Scope`, a raw pointer, a `threads(creator)` handle) sent to another thread | E |
| E7010 | a parallel loop's chunks may write one place | E (AN-74, AN-84) |
| E7012 | a `static mut` not `guarded by` a lock reached from another thread | E (AN-76, AN-84) |
| E7013 | a `guarded by` field used without holding its lock | E |
| E7014 | a `thread_shared` value changed directly | E |
| E7015 | a one-thread owner (`shared`, a `weak` link of one) or a lent value reaches another thread | E |
| E7016 | a `@parallel` loop over something it cannot split into chunks: neither a range (`for i in 0..n:`) nor a list walked by element (`for ref mut p in xs:`), for example a `while` loop (**19:** was "it counts over a range, or walks a list with `for ref mut p in xs`", the two forms AN-74 and AN-84 accept, R18-05) | E (AN-84) |
| **E7017** (new, **21:**) | `thread_shared` on a name whose value was not created `thread_shared` (a plain local, a `shared` value, a plain parameter), before a value or in a capture list: it would freeze a value its own line lets its owner change (R3, R9; F-08) | E (AN-71, AN-75, AN-78) |
| L3015 | an `unsafe fn` with no `@safety` (every one, not only `pub`) | E (AN-79) |
| W3012 | an `unsafe:` block with no `# SAFETY:` note (today's) | E (AN-79) |
| **W3013** (new, **19:**) | a `# SAFETY:` note on an `unsafe fn` line, where `@safety` says it (R15) | E (AN-79) |

Retired: E7001, E7003, E7005 (with `@sync`), and C's proposed E3045 (clash 1: its case is E3023).
**19:** Moved, so each code keeps one meaning: E3021's new meaning (a change through a read-only view)
is E3023's, which already had it for parameters and pattern names (AN-38, AN-44, AN-46); a view used
after a reshape is E3060 everywhere (AN-37's E3025, AN-46's and AN-52's E3020, AN-35's E3040); E0100
keeps today's meaning only, the parser's (AN-38's `mut path` is E0113, AN-78's `owned fn` E0109);
AN-45's `.clone()` is E1010, not E2020 (today's type mismatch); AN-35's `v.push(template)` is E3013,
not E3041 (today's "moved in a loop"); AN-79's note on an `unsafe fn` line has a code, W3013.

## 9. Open points for the owner

1. **No word at the call site for a changed argument** (clash 9): `bump(a)` changes `a` with no word on
   the calling line; the callee's first line says `ref mut`. Settled by row 3's own example; the owner
   may want a call-site word (as C#'s `ref`), which would apply to every area at once.
2. **`Box[T, A]` with an allocator** (`[ALC-1]`, spec 2986) has no spelling once `Box` goes; no annoyance
   needs it, so no word was made.
3. **R2 makes `=` do two things by what is on its right**: a whole name is handed over (row 9), a part is
   viewed (row 8's rule). The agreed rows already do both; the owner may want to see the two side by
   side (C's AN-35 and AN-34).
4. **Kept `ref mut` views may name one object** (clash 3): Rust's one-`&mut` rule is dropped for views
   kept in structs and lists (not for parameters), on the strength of R8 (b).
5. **AN-60's dispatch loop changes the list it indexes** (clash 12): a `while` loop over positions, not a
   `for` walk; the owner's "no clever mechanism inside the loop" is kept for `for` loops.

**19:** the fixer may not add or change words, so these stay open, each with the problem that raised it
(`RUN3/18-readability.md`) and what the fix in this file does meanwhile:

6. **Point 1 read literally fails the owner's test** (R18-16): in `bump(a)` the name `a` is changed and
   no word in that line says so. Unchanged here; it needs a call-site word, which only the owner can add.
7. **`mut self` and `static mut` keep bare `mut`** (R18-14), after row 8's test refused `mut` on every
   other name; a reader from Rust takes `fn f(mut self)` for a changeable copy. The spelling that passes
   the test the way every other name does would be `ref mut self`: a change of a word, left to the owner.
8. **`ref mut` says "the only way in" on a parameter and "one of several ways in" on a kept view**
   (R18-22; point 4 above): a judgement for the owner.
9. **A word for "a function value that changes no state of its own"** (R18-02): with it, AN-53's
   `fn click(self):` could call its handlers through plain `h` again (Rust's `Fn` against `FnMut`).
   Meanwhile R12 takes every call through a stored function value as a change of its holder.
   **21:** (F-04) A type word of that kind could also say what a function value changes (as
   `fn() changes LOG`), so that first lines alone would show a `static mut` changed through a handler;
   meanwhile R5 states the exception and D3's inferred summary carries it (R16).
10. **A `weak` link that `=` copies** (R18-13): making `weak` `Copy` would let `a = n.parent` copy the
    link with no word, as numbers are copied. That changes the `Copy` row, so R2 now treats `weak` like
    every other non-`Copy` type instead (`copied n.parent`).
11. **`=` never re-points a view** (R18-03, R2): the Python habits `line = line.strip()` on a parameter
    and "best so far" (`best = e` inside a loop over views) are refused with a new name, a position or a
    copy as the line to write. A re-point that the line says (C#'s `r = ref b`) would need the owner to
    change AN-28's ruling and the `ref` row. **21:** (F-06) widened: a name holding a view a call gave
    is given its value once too (`n = p.name.trim()` then `n = q.name.trim()` is refused, with a new
    name as the line to write).
12. **21:** (F-02) **A local view that a call may point elsewhere** (Rust's `let mut rest = s;` then
    `advance(&mut rest)`): a view name is given its value once (R2), so a `ref mut` view parameter is
    given a field or a list element that holds a view; a local that may be re-pointed would need a word
    on its line. Narrowing what such a parameter may be given (R6's `from` written on a parameter, a
    new position of a word) is also the owner's; meanwhile the caller's place may view all the call's
    view arguments (R4).
13. **21:** (F-05) **20's alternative for `r = first_even(...)`**, a name bound to a `ref mut T` call
    with no word made a `ref mut` name, was not taken: no word on that line would say that `r` may
    change the element (the owner's test); R2 asks `r = ref mut first_even(...)`, as it asks
    `boss = ref mut enemies[0]`.

## 10. Area A: lists, loops, temporary values (AN-01 .. AN-16)

Copied from `RUN3/08-area-A.md` by `build.py`; the combiner's changes are marked **14:** and listed in section 4. File names in this area's text are relative to `target/redesign3/08-area-A/` unless they say otherwise; "section N" inside it means the area report's own section N.

### Terms of area A

- **list**: Ember's `Array[T]`: a pointer to a block of memory holding the elements (the **buffer**),
  the length and the capacity. **Reallocate**: a full list that grows gets a new, larger buffer and
  frees the old one, so an address into the old buffer would point into freed memory.
- **element name**: a name made from one element of a list, `boss = ref mut enemies[0]`.
- **position**: the element's index. Rule A-1 below makes an element name a position, not an address.
- **view**: a value that points into something else and may not outlive it (`ref T`, `Span[T]`,
  `MutSpan[T]`, `StrView`, an `SoA` element proxy).
- **temporary**: a value made inside an expression and never given a name (`line.to_lower()` in
  `line.to_lower().trim()`).
- **summary** (of a function): what the compiler records about what a function does to each list it
  takes as `ref mut`: **only adds** (`push`, `extend` at the end) or **may do more** (remove, insert,
  reorder, clear, replace). Inferred from the body, the way today's compiler already infers the
  fields a private method touches (`[BRW-10]`); for a `pub` function it is written into the package's
  interface file.
- **unseen code**: a call whose body the compiler cannot read here: a function value, an interface
  method, a foreign function. Its summary is "may do anything its type allows".
- **noalias**: the promise to the C compiler that two pointers never point at the same memory (C's
  `restrict`); Rust gives it for every `&mut`, and it lets the C compiler use vector instructions.
- **the owner's test**: for every name in a line, the word in the line that says what it is (the
  owner's own thing, a copy, or handed over) and the word that says whether it may change it (brief).
  "none (default: ...)" is written only where the brief allows it, with the default stated.
- **refused**: a compile-time error. **Stops**: a run-time stop with a message (Ember's panic: nothing
  is unwound, `[PAN-1]`).
- **PROVISIONAL**: a timing made while other agents were running; phase 4's timer decides.

### Rules of area A (A-1 .. A-8)

The answers in section 3 cite these.

**A-1. An element name is a position.** `boss = ref mut enemies[0]` makes `boss` a name for "element 0
of `enemies`", may change it. The position is computed once (`boss = enemies[i]; i += 1` does not move
`boss`). The compiler keeps the element's address in a register while nothing can change `enemies`,
and reads the buffer again after anything that may (A-2). The words, one meaning each:

| written | `boss` is | may change the element? |
|---|---|---|
| `boss = ref mut enemies[0]` | element 0 of `enemies` (row 8's words) | yes |
| `boss = enemies[0]`, the element not a number | element 0 of `enemies`, read only (as a plain `for` name and row 8's plain `path`) | no: a write through `boss` is refused (AN-01) |
| `n = counts[0]`, a number | the number, a value of its own (Python's meaning for numbers) | `n` is its own; `counts[0]` does not change |
| `boss = copied enemies[0]` | a copy of your own (rows 6, 9) | yes, the copy |

Today a plain `boss = enemies[0]` of a `Copy` struct copies it silently, and `boss.hp = ...` then
changes the copy, not the enemy (`an01c`, accepted): the owner's [CLO-15] failure in another place.
Under A-1 that line is refused with both ways out.

**14:** combined into R2 (a name bound to any part, a field or a pattern name as well as an element, is a
read-only view; area C's partial hand-over by `=` goes, clash 1) and R8 (a) (an element name of a list
reached through locals, parameters and one-owner fields is a position; every other view is an address and
follows area C's C6, clash 2).

**A-2. While an element name is still used, its list may grow but not shrink or move.**
- Adding (`push`, `extend`, or a call whose summary only adds) is allowed: the compiler reads the
  buffer pointer again after it. No bounds check is needed then: adding never shortens a list.
- Anything that may remove, insert, reorder, clear or replace the list (written here, or a call whose
  summary may do it, or unseen code given the list) while the name is used afterwards is refused, with
  "use `boss` before line N, or name the enemy again after it: `boss = ref mut enemies[0]`".
- A view into a part of the element that a write can free (the text of a `String` field, the elements
  of an `Array` field) may not be used after a write that replaces that part, through any name.
- Writes to the element through another name of the same list (`enemies[0].hp = 1`) are allowed: plain
  data frees nothing, and the next use of `boss` reads the element again.

**A-3. A list may not change while a loop walks it** (the owner's understandable limitation, kept as
agreed). In `for ref mut e in enemies:` (or a plain `for e in enemies:`), the body may not add to,
remove from, reorder, clear or replace `enemies`, directly or through a call that may. The change goes
after the loop: `retain` for removals, `extend` for additions. One addition: a loop that counts over a
list's length (`for i in 0..enemies.len():`) may add to the list, but may not remove from it, directly
or through a call (today it is accepted, and each removal skips the next element and the last steps
stop with an out-of-range index). The compiler may run the loop and the `retain` that follows it in
one pass (the fusion of AN-04); that changes the speed, never what any code can see.
**14:** A-3 governs `for` loops. A `while` loop that tests its position against the list's current length
at every step may add and remove (area D's AN-60; R8 (c), clash 12).

**A-4. Two `ref mut` parameters are two different places.** A function with two `ref mut` parameters
is compiled with noalias on both, as Rust's two `&mut`. A call that passes two elements of one list
must give two different positions: proved at compile time when it can be (constants that differ,
`j` from `i + 1..`, inside `if i != j:`), otherwise checked by one compare before the call, which stops
the program with a message naming both elements. The same holds for two `ref mut` element names in one
function.
**14:** widened by R4: a `ref mut` argument and any other view argument of one call are two different
places too (proved, or one compare), so area B's `tick(g, g)` refusal (AN-18) and this rule are one rule.
Views kept in structs or lists do not get this promise (R9, clash 3).

**A-5. A temporary lives until the last use of every view made from it** (agreed row 5), including
views kept in a local list. Returning such a view, or storing it where it would outlive the temporary,
stays refused (row 5), with the message naming the view and the expression that made the temporary.

**A-6. A `mut self` method on a temporary.** Allowed when what the call returns is used (the temporary
changes and then ends with its statement, as in Rust). Refused when nothing it returns is used, since
the change could only reach a temporary that ends on that line.

**A-7. A view's type says what it allows.** `MutSpan[T]` writes its elements with no other word;
`Span[T]` and `StrView` read; an `SoA` element is named, never silently copied; an ECS query says
`ref mut T` for what it changes.

**A-8. The message rule for this area**: the program's own names (`enemies[0]`, not `*enemies[0]`;
`line.to_lower()`, not `_5`; `float`, not `f64`); why in one line; the exact line to write; no help
that does not apply (no `RefCell` or `class` for a view parameter).

### The answers of area A

#### AN-01: Holding an element while adding to the list (A2)

1. **Intent:** `split_boss` halves the first enemy's hp and adds a minion with the other half.

2. **OLD.** With today's reference:

`target/redesign3/08-area-A/old/an01a_ref_mut_push.em`:

```ember
# AN-01: split_boss halves the first enemy's hp and adds a minion with the other half.
struct Enemy:
    x: float
    hp: int

fn split_boss(mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
    boss.hp = boss.hp // 2

fn main():
    es = [Enemy(x=0.0, hp=100)]
    split_boss(es)
    println(es.len(), es[0].hp)
```

`target/redesign3/bin/ember.exe check an01a_ref_mut_push.em`:

```text
error[E3022]: `*enemies[0]` is already mutably borrowed
 --> an01a_ref_mut_push.em:8:5
  |
7 |     boss = ref mut enemies[0]
  |                            - mutable borrow of `*enemies[0]` starts here
8 |     enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting access here
  |                                   --- borrow later used here
  |
  = help: end the borrow before this: `boss` is what keeps it alive, so shorten its last use or put it in a block of its own
  = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error: could not compile due to 1 error(s)
exit=1
```

The Python spelling (the message says "moved" where `[EXP-6]` says E3011, AN-16):

`target/redesign3/08-area-A/old/an01b_python_spelling.em`:

```ember
# AN-01, the Python spelling: boss = enemies[0].
struct Enemy:
    x: float
    hp: int

fn split_boss(mut enemies: Array[Enemy]):
    boss = enemies[0]
    enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
    boss.hp = boss.hp // 2

fn main():
    es = [Enemy(x=0.0, hp=100)]
    split_boss(es)
    println(es.len(), es[0].hp)
```

`target/redesign3/bin/ember.exe check an01b_python_spelling.em`:

```text
error[E3040]: `enemies` is borrowed after it has been moved out of
 --> an01b_python_spelling.em:8:5
  |
7 |     boss = enemies[0]
  |            ---------- moved here
8 |     enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ borrowed here after the move
  |
  = help: borrow before the move, or clone the value so the move takes the copy
  = note: the borrow would point at memory whose owner gave it away
  = note: assigning a list, a map or a set moves it; in Python both names would point at one list (Appendix E)

error: could not compile due to 1 error(s)
exit=1
```

And with a `Copy` struct the Python spelling is accepted and changes a private copy, so `es[0].hp`
stays 100 (the program is not run here; the meaning follows from `[OWN-7]`: assigning a `Copy` value
copies it):

`target/redesign3/08-area-A/old/an01c_copy_struct.em`:

```ember
# AN-01, a Copy struct: boss = enemies[0] then boss.hp changes. Which hp changes?
@derive(Copy)
struct Enemy:
    x: float
    hp: int

fn split_boss(mut enemies: Array[Enemy]):
    boss = enemies[0]
    enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
    boss.hp = boss.hp // 2

fn main():
    es = [Enemy(x=0.0, hp=100)]
    split_boss(es)
    println(es.len(), es[0].hp)
```

`target/redesign3/bin/ember.exe check an01c_copy_struct.em`:

```text
exit=0
```

3. **NEW:**

```ember
struct Enemy:
    x: float
    hp: int

fn split_boss(ref mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
    boss.hp = boss.hp // 2
```

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `enemies` | `ref mut`: the caller's list (C-1) | `ref mut`: yes |
| `boss` | `ref mut`: element 0 of `enemies` (A-1) | `ref mut`: yes |
| `x`, `hp` (fields) | none (default: a value stored in each `Enemy`) | none (default: changes only through a name that may change the enemy, here `boss`) |

5. **Means exactly:** `boss` is position 0, read again after the push. Today's Ember, accepted:

`target/redesign3/08-area-A/means/m01_positions.em`:

```ember
# AN-01 means exactly: `boss` is position 0 of `enemies`, read again after the push.
struct Enemy:
    x: float
    hp: int

fn split_boss(mut enemies: Array[Enemy]):
    b = 0
    enemies.push(Enemy(x=enemies[b].x + 1.0, hp=enemies[b].hp // 2))
    enemies[b].hp = enemies[b].hp // 2

fn main():
    es = [Enemy(x=0.0, hp=100)]
    split_boss(es)
    println(es.len(), es[0].hp, es[1].hp)
```

`target/redesign3/bin/ember.exe check m01_positions.em`:

```text
exit=0
```

6. **Still refused:**
   - `boss = enemies[0]` (plain) then `boss.hp = boss.hp // 2`, whether `Enemy` is `Copy` or not:
     "`boss` reads `enemies[0]` and cannot change it. To change that enemy through `boss`, write
     `boss = ref mut enemies[0]`; to change a copy of your own, write `boss = copied enemies[0]`."
   - a removal, insertion, reordering, `clear()` or `enemies = [...]` while `boss` is used after it:
     ```ember
     boss = ref mut enemies[0]
     enemies.remove(0)
     boss.hp -= 1
     ```
     "`boss` names `enemies[0]`, and `enemies.remove(0)` on line 8 may put another enemy in that place.
     Use `boss` before line 8, or name the enemy again after it: `boss = ref mut enemies[0]`."
   - a view into a part that a write frees, used after that write (with `name: String` in `Enemy`):
     `n = boss.name.trim()`, then `enemies[0].name = String("x")`, then `n.len()`: "`n` is a view of
     `enemies[0].name`, whose old text line 9 frees. Use `n` before line 9, or keep your own copy:
     `n = String(boss.name.trim())`."

7. **Cost:** after the push, one reload of `enemies`' buffer pointer, then an indexed store; no bounds
   check (adding never shortens the list). C must reload after a `realloc` too, and Rust's workaround
   (copy the fields out, push, index again) is the same instructions plus a bounds check
   (`tasks/96/redesign/phase3b-containers.md:233-239`, argued there from the emitted code).

8. **Kind:** a smarter compile-time solution (Rust's rule "no add while a reference lives" is not
   needed for adds; it stays for removals and reorders).

#### AN-02: A struct element's name used after a call that may change its list (last run's limit)

1. **Intent:** keep updating `enemies[0]` after calling a helper that adds enemies.

2. **OLD.** A helper the compiler can read:

`target/redesign3/08-area-A/old/an02a_helper_adds.em`:

```ember
# AN-02: keep updating enemies[0] after calling a helper that adds enemies.
struct Enemy:
    x: float
    hp: int

fn spawn_minion(mut enemies: Array[Enemy], x: float):
    enemies.push(Enemy(x=x, hp=10))

fn hit_boss(mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    boss.hp -= 10
    spawn_minion(enemies, boss.x)
    boss.hp -= 10

fn main():
    es = [Enemy(x=0.0, hp=100)]
    hit_boss(es)
    println(es.len(), es[0].hp)
```

`target/redesign3/bin/ember.exe check an02a_helper_adds.em`:

```text
error[E3022]: `*enemies[0]` is already mutably borrowed
  --> an02a_helper_adds.em:12:5
   |
10 |     boss = ref mut enemies[0]
   |                            - mutable borrow of `*enemies[0]` starts here
11 |     boss.hp -= 10
12 |     spawn_minion(enemies, boss.x)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting access here
   |                           ------ borrow later used here
   |
   = help: end the borrow before this: `boss` is what keeps it alive, so shorten its last use or put it in a block of its own
   = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error: could not compile due to 1 error(s)
exit=1
```

A helper it cannot read (a function value):

`target/redesign3/08-area-A/old/an02b_unseen_call.em`:

```ember
# AN-02: the helper is a function value the compiler cannot see into.
struct Enemy:
    x: float
    hp: int

fn hit_boss(mut enemies: Array[Enemy], on_hit: fn(mut Array[Enemy])):
    boss = ref mut enemies[0]
    boss.hp -= 10
    on_hit(enemies)
    boss.hp -= 10

fn spawn(mut enemies: Array[Enemy]):
    enemies.push(Enemy(x=1.0, hp=10))

fn main():
    es = [Enemy(x=0.0, hp=100)]
    hit_boss(es, spawn)
    println(es.len(), es[0].hp)
```

`target/redesign3/bin/ember.exe check an02b_unseen_call.em`:

```text
error[E3022]: `*enemies[0]` is already mutably borrowed
  --> an02b_unseen_call.em:9:5
   |
 7 |     boss = ref mut enemies[0]
   |                            - mutable borrow of `*enemies[0]` starts here
 8 |     boss.hp -= 10
 9 |     on_hit(enemies)
   |     ^^^^^^ conflicting access here
10 |     boss.hp -= 10
   |                -- borrow later used here
   |
   = help: end the borrow before this: `boss` is what keeps it alive, so shorten its last use or put it in a block of its own
   = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error: could not compile due to 1 error(s)
exit=1
```

3. **NEW:**

```ember
fn spawn_minion(ref mut enemies: Array[Enemy], x: float):
    enemies.push(Enemy(x=x, hp=10))

fn hit_boss(ref mut enemies: Array[Enemy]):
    boss = ref mut enemies[0]
    boss.hp -= 10
    spawn_minion(enemies, boss.x)
    boss.hp -= 10
```

   `spawn_minion`'s summary is "only adds to `enemies`", so `boss` stays valid across the call (A-2):
   the buffer is read again after it. The summary is what the body does, inferred; nothing in the line
   changes meaning with it: `boss` names element 0 either way, and the summary decides only whether
   the line compiles.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `enemies` (in `spawn_minion`) | `ref mut`: the caller's list | `ref mut`: yes |
| `x` (parameter) | none (default: a number passed in, the function's own value) | none (default: read; **19:** never changed in the function, R4: was "changing it changes nothing the caller holds", which implied `x += 1` was accepted, R18-20) |
| `enemies` (in `hit_boss`) | `ref mut`: the caller's list | `ref mut`: yes |
| `boss` | `ref mut`: element 0 of `enemies` | `ref mut`: yes |

5. **Means exactly:** position 0 kept across the call. Today's Ember, accepted:

`target/redesign3/08-area-A/means/m02_helper_adds.em`:

```ember
# AN-02 means exactly: position kept across a call that only adds.
struct Enemy:
    x: float
    hp: int

fn spawn_minion(mut enemies: Array[Enemy], x: float):
    enemies.push(Enemy(x=x, hp=10))

fn hit_boss(mut enemies: Array[Enemy]):
    b = 0
    enemies[b].hp -= 10
    spawn_minion(enemies, enemies[b].x)
    enemies[b].hp -= 10

fn main():
    es = [Enemy(x=0.0, hp=100)]
    hit_boss(es)
    println(es.len(), es[0].hp)
```

`target/redesign3/bin/ember.exe check m02_helper_adds.em`:

```text
exit=0
```

6. **Still refused:**
   - unseen code given the list while `boss` is used after it (`on_hit(enemies)` with
     `on_hit: fn(ref mut Array[Enemy])`): "`boss` names `enemies[0]`, and `on_hit(enemies)` on line 9
     may remove or reorder enemies (it is a function value, so what it does is not known here). Use
     `boss` before line 9, or name the enemy again after it: `boss = ref mut enemies[0]`."
   - a helper the compiler can read that may remove: "`boss` names `enemies[0]`, and
     `kill_weak(enemies)` on line 9 may remove enemies (`kill_weak` line 3: `enemies.remove(i)`). Use
     `boss` before line 9, or name the enemy again after it: `boss = ref mut enemies[0]`."

7. **Cost:** nothing more than AN-01: one reload after the call (the C compiler reloads after any call
   that gets the list anyway).

8. **Kind:** a smarter compile-time solution for code the compiler can read; for unseen code the
   understandable limitation stays, with a message that gives the line.

#### AN-03: Two elements of one list in one call (A3)

1. **Intent:** `collide(bodies[i], bodies[j])` swaps the speeds of two bodies.

2. **OLD:**

`target/redesign3/08-area-A/old/an03_two_elements.em`:

```ember
# AN-03: collide changes the speeds of two bodies of one list.
struct Body:
    pos: float
    vel: float

fn collide(mut a: Body, mut b: Body):
    t = a.vel
    a.vel = b.vel
    b.vel = t

fn step(mut bodies: Array[Body], i: int, j: int):
    collide(bodies[i], bodies[j])

fn main():
    bs = [Body(pos=0.0, vel=1.0), Body(pos=1.0, vel=-1.0)]
    step(bs, 0, 1)
    println(bs[0].vel)
```

`target/redesign3/bin/ember.exe check an03_two_elements.em`:

```text
error[E3022]: `*bodies[…]` is already mutably borrowed
  --> an03_two_elements.em:12:31
   |
12 |     collide(bodies[i], bodies[j])
   |                               ^ conflicting access here
   |                    - mutable borrow of `*bodies[…]` starts here
   |
   = help: use `bodies.get_pair_mut(i, j)` for two elements at once (`None` when they are the same), or `bodies.as_mut_span().split_at(k)` for two non-overlapping mutable spans; `swap`, `chunks_mut` and `iter_mut` cover other patterns
   = note: computed indices may overlap; a structural split proves non-overlap (BRW-5)

error: could not compile due to 1 error(s)
exit=1
```

3. **NEW:**

```ember
struct Body:
    pos: float
    vel: float

fn collide(ref mut a: Body, ref mut b: Body):
    t = a.vel
    a.vel = b.vel
    b.vel = t

fn step(ref mut bodies: Array[Body], i: int, j: int):
    collide(bodies[i], bodies[j])           # one compare: i != j

fn all_pairs(ref mut bodies: Array[Body]):
    for i in 0..bodies.len():
        for j in i + 1..bodies.len():
            collide(bodies[i], bodies[j])   # proved: j > i, nothing runs
```

   A-4: `collide` says "two bodies, both changed" with two `ref mut` names, so `a` and `b` are two
   different places, and the compiler emits noalias on both. In `all_pairs` the positions differ by
   construction; in `step` they are compared once before the call. The call itself carries no word,
   as row 3's `self.note(...)` carries none: the parameter's line says it, and the call is checked.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `a`, `b` | `ref mut`: a body of the caller's | `ref mut`: yes |
| `t` | none (default: a number, a value of its own) | none (default: its own; changes nothing else) |
| `bodies` (in `step`, in `all_pairs`) | `ref mut`: the caller's list | `ref mut`: yes |
| `i`, `j` (parameters of `step`) | none (default: numbers passed in) | none (default: read) |
| `i`, `j` (loop names in `all_pairs`) | none (default: the loop's own numbers) | none (default: set by the loop only) |
| `pos`, `vel` (fields) | none (default: values stored in each `Body`) | none (default: through a name that may change the body) |

5. **Means exactly:** today's `get_pair_mut` (the one compare) and `split_at` (the proof by
   construction). Accepted:

`target/redesign3/08-area-A/means/m03_pair.em`:

```ember
# AN-03 means exactly: two different bodies, checked once where not proven.
struct Body:
    pos: float
    vel: float

fn collide(mut a: Body, mut b: Body):
    t = a.vel
    a.vel = b.vel
    b.vel = t

fn step(mut bodies: Array[Body], i: int, j: int):
    match bodies.get_pair_mut(i, j):
        Some((a, b)):
            collide(a, b)
        None:
            panic(f"collide was given bodies[{i}] twice; test `if i != j:` before the call to skip it")

fn all_pairs(mut bodies: Array[Body]):
    n = bodies.len()
    for i in 0..n:
        for j in i + 1..n:
            parts = bodies.as_mut_span().split_at(j)
            collide(parts.0[i], parts.1[0])

fn main():
    bs = [Body(pos=0.0, vel=1.0), Body(pos=1.0, vel=-1.0)]
    step(bs, 0, 1)
    all_pairs(bs)
    println(bs[0].vel)
```

`target/redesign3/bin/ember.exe check m03_pair.em`:

```text
exit=0
```

6. **Still refused:**
   - the same position twice, known when compiling (`collide(bodies[i], bodies[i])`): "`collide`
     changes `a` and `b` as two different bodies, and both are `bodies[i]`. Pass two different bodies,
     or write a one-body version of `collide`."
   - at run time, where not proven and `i == j` (a stop, not a refusal): "`collide` was given `bodies[3]`
     as both `a` and `b`; they must be two different bodies. Test `if i != j:` before the call to skip
     it."

7. **Cost:** where proved, nothing. Elsewhere one compare and a branch never taken. Last run measured
   the inserted compare at no cost (89.9 ms with it, 94.9 ms without it and without noalias; safe Rust's
   `get_disjoint_mut` 87.6 ms: `tasks/96/redesign/phase3b-containers.md:301-306`, from
   `target/redesign2/containers/bench/runs/a3.txt`). Noalias is what matters for speed: without it,
   two spans of one list took 1.53x (`phase3b-containers.md:351-352`, `runs/a3_span.txt`). A-4 keeps
   noalias on every `ref mut` pair.

8. **Kind:** a readable spelling of Rust's guarantee (two `&mut` never alias), with the compile-time
   proof that Rust's borrow checker does not make for computed positions.

#### AN-04: Changing a list while looping over it (A4)

1. **Intent:** each frame, every enemy loses 1 hp; a dead enemy that splits makes a new enemy; the dead
   are removed. Settled by the owner: the removal and the addition happen after the loop.

2. **OLD:**

`target/redesign3/08-area-A/old/an04_change_in_loop.em`:

```ember
# AN-04: remove dead enemies and add their splits during the update loop.
struct Enemy:
    hp: int
    splits: bool

fn update(mut enemies: Array[Enemy]):
    i = 0
    for e in enemies.iter_mut():
        e.hp -= 1
        if e.hp <= 0:
            if e.splits:
                enemies.push(Enemy(hp=5, splits=false))
            enemies.remove(i)
        i += 1

fn main():
    es = [Enemy(hp=1, splits=true), Enemy(hp=3, splits=false)]
    update(es)
    println(es.len())
```

`target/redesign3/bin/ember.exe check an04_change_in_loop.em`:

```text
error[E3020]: cannot mutate `*enemies` while it is borrowed by this loop
  --> an04_change_in_loop.em:12:17
   |
 8 |     for e in enemies.iter_mut():
   |              ------- `*enemies` borrowed here for the whole loop
...
12 |                 enemies.push(Enemy(hp=5, splits=false))
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow here
   |
   = help: collect the indices or values first, then mutate; use `retain` or `drain` when the operation fits
   = help: or iterate over a fixed index range and mutate only after taking the current value
   = note: a `for` loop borrows the iterable until the loop ends (CTL-2)

error[E3020]: cannot mutate `*enemies` while it is borrowed by this loop
  --> an04_change_in_loop.em:13:13
   |
 8 |     for e in enemies.iter_mut():
   |              ------- `*enemies` borrowed here for the whole loop
...
13 |             enemies.remove(i)
   |             ^^^^^^^^^^^^^^^^^ mutable borrow here
   |
   = help: collect the indices or values first, then mutate; use `retain` or `drain` when the operation fits
   = help: or iterate over a fixed index range and mutate only after taking the current value
   = note: a `for` loop borrows the iterable until the loop ends (CTL-2)

error: could not compile due to 2 error(s)
exit=1
```

The vocabulary's loop spelling (C-1), accepted by the parser today and read-only:

`target/redesign3/08-area-A/old/an04b_for_ref_mut.em`:

```ember
# AN-04: the vocabulary's loop spelling `for ref mut e in enemies:` today.
struct Enemy:
    hp: int

fn update(mut enemies: Array[Enemy]):
    for ref mut e in enemies:
        e.hp -= 1

fn main():
    es = [Enemy(hp=1)]
    update(es)
    println(es[0].hp)
```

`target/redesign3/bin/ember.exe check an04b_for_ref_mut.em`:

```text
error[E3021]: cannot write through a shared reference
 --> an04b_for_ref_mut.em:7:9
  |
7 |         e.hp -= 1
  |         ^^^^ this is `ref Enemy`, which only reads
  |
  = help: take the borrow as `ref mut` if the callee must write, or assign to the place itself rather than through the reference
  = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]

error: could not compile due to 1 error(s)
exit=1
```

3. **NEW:**

```ember
struct Enemy:
    hp: int
    splits: bool

fn update(ref mut enemies: Array[Enemy]):
    spawned: Array[Enemy] = []
    for ref mut e in enemies:
        e.hp -= 1
        if e.hp <= 0 and e.splits:
            spawned.push(Enemy(hp=5, splits=false))
    enemies.retain(fn(e) => e.hp > 0)
    enemies.extend(spawned)
```

   A-3 as agreed. The new part is speed: the compiler runs the loop and the `retain` after it in one
   pass when (1) the `retain` follows the loop on the same list with nothing between that touches the
   list, (2) the loop has no `break` or `return`, (3) the `retain` test reads only its element and
   values the loop does not change, (4) the loop body does not read the list by position
   (`enemies[k]`), and (5) removing an element runs no code of its own that the program could see
   (freeing memory only). Each condition is decided when compiling; when one fails, the two passes run
   as written. Fusing changes no result any code can observe: the loop sees no removal, and the
   `retain` sees every element after the loop's write to it.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `enemies` | `ref mut`: the caller's list | `ref mut`: yes (after the loop) |
| `spawned` | none (default: the function's own new list) | none (default: its own; changing it changes nothing anyone else holds) |
| `e` (loop name) | `ref mut`: the current element of `enemies` (C-1) | `ref mut`: yes |
| `e` (the `retain` test's parameter) | none (default: each element, read only: a lambda parameter changes nothing unless written `ref mut`, C-1) | none (default: read only) |
| `hp`, `splits` (fields) | none (default: values stored in each `Enemy`) | none (default: through `e`) |

   `spawned` in `enemies.extend(spawned)` is handed over (row 9: `extend` keeps its elements, so
   `spawned` ends there).

5. **Means exactly:** in today's Ember (the loop, `retain`, `extend`), accepted:

`target/redesign3/08-area-A/means/m04_after_loop.em`:

```ember
# AN-04 means exactly: change after the loop: retain, then add the splits.
struct Enemy:
    hp: int
    splits: bool

fn update(mut enemies: Array[Enemy]):
    spawned: Array[Enemy] = []
    for e in enemies.iter_mut():
        e.hp -= 1
        if e.hp <= 0 and e.splits:
            spawned.push(Enemy(hp=5, splits=false))
    enemies.retain(fn(e: Enemy) => e.hp > 0)
    enemies.extend(spawned)

fn main():
    es = [Enemy(hp=1, splits=true), Enemy(hp=3, splits=false)]
    update(es)
    println(es.len())
```

`target/redesign3/bin/ember.exe check m04_after_loop.em`:

```text
exit=0
```

   And in C, as written (two passes) and fused (one pass; the same code a C programmer writes). Both
   print the same checksum (`c/a4/runs_an04.txt`):

`target/redesign3/08-area-A/c/a4/two_pass.c`:

```c
/* The NEW code as written, compiled without fusion: the loop updates in place and collects births;
   then `enemies.retain(fn(e) => e.hp > 0)` (a second pass); then `enemies.extend(spawned)`. */
#include "common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    List spawned = {NULL, 0, 0};
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        for (size_t r = 0; r < l.len; r++) {                 /* for ref mut e in enemies: */
            Enemy *e = &l.data[r];
            e->x += e->vx; e->hp -= 1;
            if (e->hp <= 0) list_push(&spawned, (Enemy){next++, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f});
        }
        size_t w = 0;                                        /* enemies.retain(fn(e) => e.hp > 0) */
        for (size_t r = 0; r < l.len; r++)
            if (l.data[r].hp > 0) l.data[w++] = l.data[r];
        l.len = w;
        list_extend(&l, &spawned);                           /* enemies.extend(spawned) */
    }
    h_report("an04-two-pass", checksum(&l, next));
    free(l.data); free(spawned.data);
    return 0;
}
```

`target/redesign3/08-area-A/c/a4/plain.c`:

```c
/* Plain C, fastest form found: one pass, element loaded once, updated in registers, stored once
   at its final place; births go to a side list appended after the pass. This is also what the
   compiler emits for the NEW code when it fuses the loop with the `retain` after it (AN-04). */
#include "common.h"
int main(void) {
    h_start();
    List l; fill(&l);
    List spawned = {NULL, 0, 0};
    uint32_t next = N;
    for (int f = 0; f < FRAMES; f++) {
        size_t w = 0, n = l.len;
        for (size_t r = 0; r < n; r++) {
            Enemy v = l.data[r];
            v.x += v.vx; v.hp -= 1;
            if (v.hp <= 0) { list_push(&spawned, (Enemy){next++, 2 + (int32_t)(h_rand() % 100), 0.0f, 1.0f}); continue; }
            l.data[w++] = v;
        }
        l.len = w;
        list_extend(&l, &spawned);
    }
    h_report("an04-plain-onepass", checksum(&l, next));
    free(l.data); free(spawned.data);
    return 0;
}
```

   (`c/a4/common.h` holds the workload; `c/a4/rust_idiom.rs` is safe Rust's idiom for the same meaning.)

6. **Still refused:**
   - `enemies.push(...)` in the loop: "`enemies` may not change while this loop walks it (line 7). Push
     the new enemy to a list of your own and add it after the loop: `spawned.push(Enemy(hp=5,
     splits=false))` here, and `enemies.extend(spawned)` after the loop."
   - `enemies.remove(i)` in the loop: "`enemies` may not change while this loop walks it (line 7).
     Remove the dead after the loop: `enemies.retain(fn(e) => e.hp > 0)`."
   - `for e in enemies:` then `e.hp -= 1`: "`e` reads each enemy and cannot change it; to change the
     enemies through `e`, write `for ref mut e in enemies:`."
   - `for e in enemies.iter_mut():`: "write `for ref mut e in enemies:`" (C-1).

7. **Cost.** 1,000,000 enemies, 100 frames, about 1% die per frame; five rounds with
   `target/redesign2/bench/run.ps1` (performance cores, mains checked each round), results in
   `target/redesign3/08-area-A/c/a4/runs_an04.txt`; every run printed checksum 153021199.
   PROVISIONAL: other agents were running (plain C ranged 114.2 to 137.9 ms).

| program | what it is | median of 5 (ms) | ratio |
|---|---|---|---|
| `plain.exe` | plain C, one pass; also the fused NEW | 119.120 | 1.00 |
| `two_pass.exe` | the NEW as written, not fused | 204.083 | 1.71 |
| `rust_idiom.exe` | safe Rust: `iter_mut`, a second `Vec`, `retain`, `extend` | 211.436 | 1.77 |

   So the limitation costs 1.71x where the compiler cannot fuse, and nothing where it can; Rust's idiom
   pays the second pass always. Peak memory: 20,864 to 21,264 KB for all three (same file).

8. **Kind:** an understandable limitation kept (agreed), made as fast as C by a compile-time fusion.

#### AN-05: A loop that removes from its own list while calling code the compiler cannot see (last run's limit)

1. **Intent:** each frame, every enemy loses 1 hp, and the game's `on_death` handler (any function
   given in) is called for each dead enemy; the handler may remove it from the list.

2. **OLD.** The loop over the list is refused (line 16); the counting loop in `update` (lines 6-10) is
   accepted, and there each removal skips the next enemy and, once the list is shorter than the range,
   `enemies[i]` stops the program with an out-of-range index:

`target/redesign3/08-area-A/old/an05_loop_calls_unseen.em`:

```ember
# AN-05: the loop calls a helper (a function value) that may also remove from the list.
struct Enemy:
    hp: int

fn update(mut enemies: Array[Enemy], on_death: fn(mut Array[Enemy], int)):
    for i in 0..enemies.len():
        enemies[i].hp -= 1
        if enemies[i].hp <= 0:
            on_death(enemies, i)

fn update_loop(mut enemies: Array[Enemy], on_death: fn(mut Array[Enemy], int)):
    i = 0
    for e in enemies.iter_mut():
        e.hp -= 1
        if e.hp <= 0:
            on_death(enemies, i)
        i += 1

fn drop_it(mut enemies: Array[Enemy], i: int):
    enemies.remove(i)

fn main():
    es = [Enemy(hp=1), Enemy(hp=3)]
    update_loop(es, drop_it)
    println(es.len())
```

`target/redesign3/bin/ember.exe check an05_loop_calls_unseen.em`:

```text
error[E3020]: cannot mutate `*enemies` while it is borrowed by this loop
  --> an05_loop_calls_unseen.em:16:13
   |
13 |     for e in enemies.iter_mut():
   |              ------- `*enemies` borrowed here for the whole loop
...
16 |             on_death(enemies, i)
   |             ^^^^^^^^ mutable borrow here
   |
   = help: collect the indices or values first, then mutate; use `retain` or `drain` when the operation fits
   = help: or iterate over a fixed index range and mutate only after taking the current value
   = note: a `for` loop borrows the iterable until the loop ends (CTL-2)

error: could not compile due to 1 error(s)
exit=1
```

3. **NEW:**

```ember
fn update(ref mut enemies: Array[Enemy], on_death: fn(ref mut Array[Enemy], int)):
    dead: Array[int] = []
    for i in 0..enemies.len():
        enemies[i].hp -= 1
        if enemies[i].hp <= 0:
            dead.push(i)
    while dead.len() > 0:
        on_death(enemies, dead.pop().unwrap())
```

   A-3. The handler runs after the loop, last position first, so each removal leaves the positions
   still to come where they were.

   **19:** (R18-17) What this does, stated, so the text says everything the program does:
   - **Order:** the handlers run for the last dead enemy first, the reverse of the OLD counting loop,
     which called them first-dead-first during the loop. Last-first is what keeps the positions right:
     a handler that removes the enemy it is given, or adds enemies at the end, moves no position still
     to come. A game whose handlers must run in the OLD order (a log, a score shown in order) walks
     `dead` from the front and does not let the handler remove: `for i in dead: on_death(enemies, i)`
     with `on_death: fn(Array[Enemy], int)` (read-only), then `enemies.retain(fn(e) => e.hp > 0)` once,
     the agreed "delete after the loop".
   - **A handler that removes another enemy** (a neighbour, the first one): the positions after the one
     it removed shift down, so a later position in `dead` names a different enemy, or is past the end.
     The second is the index check of every `enemies[i]` (a stop with the index and the length, never a
     memory error); the first the compiler cannot see, because the handler is a function given in. The
     handler's type allows it (`ref mut Array[Enemy]`), so this is the program's own contract: a handler
     given a position removes at most that enemy. A handler that must remove others queues them in a list
     and lets `update` remove them after the calls, as above.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `enemies` | `ref mut`: the caller's list | `ref mut`: yes |
| `on_death` | its type `fn(ref mut Array[Enemy], int)`: a function given by the caller | its type's `ref mut`: the function may change the list it is given; `on_death` itself is only called |
| `dead` | none (default: the function's own new list) | none (default: its own) |
| `i` | none (default: the loop's own number) | none (default: set by the loop only) |

5. **Means exactly:** today's Ember, accepted:

`target/redesign3/08-area-A/means/m05_after_loop_helper.em`:

```ember
# AN-05 means exactly: the loop collects the dead; the helper runs after the loop.
struct Enemy:
    hp: int

fn update(mut enemies: Array[Enemy], on_death: fn(mut Array[Enemy], int)):
    dead: Array[int] = []
    i = 0
    for e in enemies.iter_mut():
        e.hp -= 1
        if e.hp <= 0:
            dead.push(i)
        i += 1
    while dead.len() > 0:
        on_death(enemies, dead.pop().unwrap())

fn drop_it(mut enemies: Array[Enemy], i: int):
    _gone = enemies.remove(i)

fn main():
    es = [Enemy(hp=1), Enemy(hp=3), Enemy(hp=1)]
    update(es, drop_it)
    println(es.len())
```

`target/redesign3/bin/ember.exe check m05_after_loop_helper.em`:

```text
exit=0
```

6. **Still refused:**
   - `on_death(enemies, i)` inside `for ref mut e in enemies:`: "`enemies` may not change while this
     loop walks it (line 6), and `on_death` may change it (its type takes `ref mut Array[Enemy]`).
     Collect the positions and call it after the loop: `dead.push(i)` here, then
     `while dead.len() > 0: on_death(enemies, dead.pop().unwrap())`."
   - new (A-3): the same call inside `for i in 0..enemies.len():`, accepted today: "this loop counts
     over `enemies`' length (line 6), and `on_death(enemies, i)` may remove from `enemies`, which would
     skip the next enemy and then run past the end. Collect the positions and call it after the loop:
     `dead.push(i)` here, then `while dead.len() > 0: on_death(enemies, dead.pop().unwrap())`."

7. **Cost:** one `push` per death into `dead`, and the handler calls after the loop. C calling a
   handler it does not know must do the same to stay correct (it cannot know what the handler
   removed).

8. **Kind:** an understandable limitation kept (agreed: the list may not change while a loop walks it);
   one run-time stop moved to compile time (the counting loop).

#### AN-06: ECS: creating and destroying entities inside a view loop

1. **Intent:** a system lowers each entity's health; an entity at 0 is destroyed and a new one with 10
   health takes its place.

2. **OLD.** Inside a system there is no way to create (only `destroy`, `add`, `remove` are queued):

`target/redesign3/08-area-A/old/an06b_query_commands.em`:

```ember
# AN-06: inside a system over a Query: destroy through Commands (accepted), create (no way).
from std.ecs import World, Entity, Query, Mut, Component

@derive(Copy, Component)
struct Health:
    hp: int

fn reap(mut q: Query[(Entity, Mut[Health])], dt: float):
    cmds = q.commands()
    for e, h in q.iter_mut():
        h.hp -= 1
        if h.hp <= 0:
            cmds.destroy(e)
            n = cmds.create()
            cmds.add(n, Health(hp=10))

fn main():
    world = World()
    e = world.create()
    world.add(e, Health(hp=1))
    world.run(reap, 0.016)
    println(world.len())
```

`target/redesign3/bin/ember.exe check an06b_query_commands.em`:

```text
error[E2072]: `std.ecs.Commands` has no method named `create`
  --> an06b_query_commands.em:14:22
   |
14 |             n = cmds.create()
   |                      ^^^^^^
   |

error: could not compile due to 1 error(s)
exit=1
```

The same system with `destroy` only is accepted:

`target/redesign3/08-area-A/old/an06c_query_destroy_only.em`:

```ember
# AN-06: the same system with destroy only (what Commands offers today).
from std.ecs import World, Entity, Query, Mut, Component

@derive(Copy, Component)
struct Health:
    hp: int

fn reap(mut q: Query[(Entity, Mut[Health])], dt: float):
    cmds = q.commands()
    for e, h in q.iter_mut():
        h.hp -= 1
        if h.hp <= 0:
            cmds.destroy(e)

fn main():
    world = World()
    e = world.create()
    world.add(e, Health(hp=1))
    world.run(reap, 0.016)
    println(world.len())
```

`target/redesign3/bin/ember.exe check an06c_query_destroy_only.em`:

```text
exit=0
```

And outside a system, a loop over a copy of the ids changes the world freely (Python's habit
`for e in list(d)`; it stays accepted, the copy is a fresh list of ids):

`target/redesign3/08-area-A/old/an06a_world_in_loop.em`:

```ember
# AN-06: a system destroys dead entities and spawns new ones while iterating a view.
from std.ecs import World, Entity, Query, Mut, Component

@derive(Copy, Component)
struct Health:
    hp: int

fn reap(mut world: World):
    for e in world.entities_with[Health]():
        match world.storage[Health]():
            Some(_s):
                pass
            None:
                pass
        world.destroy(e)
        n = world.create()
        world.add(n, Health(hp=10))

fn main():
    world = World()
    e = world.create()
    world.add(e, Health(hp=0))
    reap(world)
    println(world.len())
```

`target/redesign3/bin/ember.exe check an06a_world_in_loop.em`:

```text
exit=0
```

3. **NEW:**

```ember
from std.ecs import World, Entity, Query, Component

@derive(Copy, Component)
struct Health:
    hp: int

fn reap(ref mut q: Query[(Entity, ref mut Health)], dt: float):
    for e, ref mut h in q:
        h.hp -= 1
        if h.hp <= 0:
            q.commands().destroy(e)
            n = q.commands().create()
            q.commands().add(n, Health(hp=10))
```

   As agreed: changes are queued and applied after the system returns (`[ECS-5]`, Bevy's
   `commands.entity(e).despawn()`). `Commands.create()` (section 5) hands out a real entity id at once,
   from the world's free indexes or past its highest index, so `add(n, ...)` can name it; the entity and
   its components exist once the queue is applied. The query parameter is `ref mut q` (C-1: the world
   builds the query and the system changes it by walking it and queueing on it); `World.run`'s system
   type becomes `fn(ref mut Query[Q], A)`.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `q` | `ref mut`: the query the world made for this run | `ref mut`: yes (walks it, queues on it) |
| `e` | none (default: the entity's id, a 32-bit value, `@derive(Copy)` in `std.ecs`) | none (default: an id; changing it changes no entity) |
| `h` | `ref mut`: the `Health` of `e` in the world (the query says `ref mut Health`) | `ref mut`: yes |
| `dt` | none (default: a number passed in) | none (default: read) |
| `n` | none (default: the new entity's id that `create()` returns; its meaning comes from its value, as row 5's `word`) | none (default: an id) |
| `hp` (field) | none (default: a value stored in each `Health`) | none (default: through `h`) |

5. **Means exactly:** today's Ember cannot say `create` in a system (above), so in C: what the system
   and `World.run` compile to. The pool is never changed inside the loop; destroy, create and add are
   records applied after it:

`target/redesign3/08-area-A/c/an06/commands.c`:

```c
/* AN-06 "means exactly": what the NEW system compiles to, in C.
     fn reap(ref mut q: Query[(Entity, ref mut Health)], dt: float):
         for e, ref mut h in q:
             h.hp -= 1
             if h.hp <= 0:
                 q.commands().destroy(e)
                 n = q.commands().create()
                 q.commands().add(n, Health(hp=10))
   One sparse set for Health (dense components, dense entities, sparse index), as std.ecs stores it.
   The loop never changes a pool: destroy, create and add are queued. `create` hands out a real entity
   id at once (reserved past the world's highest index), so `add(n, ...)` can name it; the queue is
   applied in order after the system returns (World.run's `self.apply(q.cmds)`).
   10,000 entities, 1,000 runs of the system; checksum = live count + sum of hp + sum of indexes. */
#include "harness.h"
#include <string.h>
#include <assert.h>
typedef struct { uint32_t index, gen; } Entity;
typedef struct { int32_t hp; } Health;
typedef struct {                       /* the world: generations, free indexes, the Health pool */
    uint32_t *gen; uint32_t n_index, cap_index;
    uint32_t *free_; uint32_t n_free, cap_free;
    Health *dense; Entity *ents; uint32_t n_dense, cap_dense;
    uint32_t *sparse;                  /* index -> dense slot + 1 (0 = none); cap_index long */
} World;
enum { CMD_DESTROY, CMD_CREATE, CMD_ADD };
typedef struct { int kind; Entity e; Health h; } Cmd;
typedef struct { Cmd *q; uint32_t n, cap; uint32_t taken, reserved; } Commands;
/* taken: freed indexes handed out (from the top of the free list); reserved: new indexes handed out */

static void *grow(void *p, uint32_t *cap, uint32_t need, size_t sz) {
    if (need <= *cap) return p;
    uint32_t c = *cap ? *cap : 16; while (c < need) c *= 2;
    p = realloc(p, (size_t)c * sz); *cap = c; return p;
}
static void w_reserve_index(World *w, uint32_t n) {       /* make indexes [0, n) exist */
    uint32_t old = w->cap_index;
    w->gen = grow(w->gen, &w->cap_index, n, sizeof(uint32_t));
    if (w->cap_index != old) {
        uint32_t c2 = old; w->sparse = grow(w->sparse, &c2, w->cap_index, sizeof(uint32_t));
        memset(w->sparse + old, 0, (size_t)(w->cap_index - old) * sizeof(uint32_t));
        memset(w->gen + old, 0, (size_t)(w->cap_index - old) * sizeof(uint32_t));
    }
    if (n > w->n_index) w->n_index = n;
}
static Entity w_create(World *w) {
    if (w->n_free) { uint32_t i = w->free_[--w->n_free]; return (Entity){i, w->gen[i]}; }
    uint32_t i = w->n_index; w_reserve_index(w, i + 1); return (Entity){i, 0};
}
static int w_alive(World *w, Entity e) { return e.index < w->n_index && w->gen[e.index] == e.gen; }
static void w_add(World *w, Entity e, Health h) {
    assert(w_alive(w, e));
    uint32_t s = w->sparse[e.index];
    if (s) { w->dense[s - 1] = h; return; }
    uint32_t old = w->cap_dense;
    w->dense = grow(w->dense, &w->cap_dense, w->n_dense + 1, sizeof(Health));
    if (w->cap_dense != old) w->ents = realloc(w->ents, (size_t)w->cap_dense * sizeof(Entity));
    w->dense[w->n_dense] = h; w->ents[w->n_dense] = e; w->n_dense++;
    w->sparse[e.index] = w->n_dense;
}
static void w_destroy(World *w, Entity e) {
    if (!w_alive(w, e)) return;
    uint32_t s = w->sparse[e.index];
    if (s) {                                              /* swap-remove from the pool */
        uint32_t last = w->n_dense - 1, at = s - 1;
        w->dense[at] = w->dense[last]; w->ents[at] = w->ents[last];
        w->sparse[w->ents[at].index] = at + 1; w->sparse[e.index] = 0; w->n_dense--;
    }
    w->gen[e.index] = (w->gen[e.index] + 1) & 4095u;
    w->free_ = grow(w->free_, &w->cap_free, w->n_free + 1, sizeof(uint32_t));
    w->free_[w->n_free++] = e.index;
}
static void c_push(Commands *c, Cmd x) { c->q = grow(c->q, &c->cap, c->n + 1, sizeof(Cmd)); c->q[c->n++] = x; }
static Entity c_create(Commands *c, World *w) {           /* a real id now: a freed index, else a new one */
    Entity e;
    if (c->taken < w->n_free) { uint32_t i = w->free_[w->n_free - 1 - c->taken++]; e = (Entity){i, w->gen[i]}; }
    else e = (Entity){ w->n_index + c->reserved++, 0 };
    c_push(c, (Cmd){CMD_CREATE, e, {0}}); return e;
}
static void w_apply(World *w, Commands *c) {              /* World.apply: in the order asked */
    w->n_free -= c->taken;                                 /* the free list is as it was when they were taken */
    w_reserve_index(w, w->n_index + c->reserved);
    for (uint32_t k = 0; k < c->n; k++) {
        Cmd x = c->q[k];
        if (x.kind == CMD_DESTROY) w_destroy(w, x.e);
        else if (x.kind == CMD_CREATE) { /* made live above */ }
        else w_add(w, x.e, x.h);
    }
    c->n = 0; c->taken = 0; c->reserved = 0;
}
static void reap(World *w, Commands *c) {                 /* the system: the pool never changes here */
    Health *dense = w->dense; Entity *ents = w->ents; uint32_t n = w->n_dense;
    for (uint32_t k = 0; k < n; k++) {
        Health *h = &dense[k];
        h->hp -= 1;
        if (h->hp <= 0) {
            c_push(c, (Cmd){CMD_DESTROY, ents[k], {0}});
            Entity m = c_create(c, w);
            c_push(c, (Cmd){CMD_ADD, m, {10}});
        }
    }
}
int main(void) {
    h_start(); h_seed(42);
    World w; memset(&w, 0, sizeof w); Commands c; memset(&c, 0, sizeof c);
    for (int i = 0; i < 10000; i++) { Entity e = w_create(&w); w_add(&w, e, (Health){1 + (int32_t)(h_rand() % 50)}); }
    for (int run = 0; run < 1000; run++) { reap(&w, &c); w_apply(&w, &c); }
    uint64_t s = w.n_dense;
    for (uint32_t k = 0; k < w.n_dense; k++) s += (uint64_t)w.dense[k].hp + w.ents[k].index;
    h_report("an06-commands", s);
    free(w.gen); free(w.free_); free(w.dense); free(w.ents); free(w.sparse); free(c.q);
    return 0;
}
```

   Built with clang -O2 and run, then with `-O1 -g -fsanitize=address` and run (AddressSanitizer: no
   report; the same checksum). The -O2 run printed:

```text
an06-commands ms=14.484 peak_kb=4472 checksum=55132960
```

   and the AddressSanitizer build:

`target/redesign3/08-area-A/c/an06/asan_output.txt`:

```text
an06-commands ms=22.934 peak_kb=9456 checksum=55132960
exit=0
```

6. **Still refused:**
   - `Mut[Health]` in the query: AN-15's message.
   - changing a pool while a loop walks it: there is no operation for it inside a system (the system
     holds only its query); last run's in-loop destroy with a cursor is not built (section 6).
   - not refused, and said in `Commands.create`'s documentation as Bevy says it: the query does not
     see `n` until the queue is applied. `create()` in a system that `run_parallel` runs is allowed,
     the id taken by one atomic add (the queues are already one per system, `std/src/ecs.em:347-376`).

7. **Cost:** one queued record per destroy, create and add (one push each into the system's queue), and
   the pool work after the loop (swap-remove, append), which C must do too. The loop holds the pool's
   arrays in registers for its whole length, since nothing can change a pool while it runs (last run
   measured that hold as the fix from 1.34x to 1.03x-1.12x of plain C:
   `tasks/96/redesign/phase3b-containers.md:116-123`). `create`: one add to a counter (an atomic add in
   a `run_parallel` system).

8. **Kind:** an understandable limitation kept (agreed), with the missing `create` added.

#### AN-07: A local list of string views (`words = text.split()`) is refused (A25)

1. **Intent:** split a line into words and count them, without copying each word.

2. **OLD.** Four spellings, all refused:

`target/redesign3/08-area-A/old/an07a_local_views.em`:

```ember
# AN-07: split a line into words and count them, without copying each word.
fn count_words(line: str) -> int:
    lower = line.to_lower()
    words: Array[str] = lower.split_whitespace().collect()
    return words.len()

fn main():
    println(count_words("A b C"))
```

`target/redesign3/bin/ember.exe check an07a_local_views.em`:

```text
error[E3063]: `str` is a view, so it may not be passed to `collect`, which stores it where only a `static` view may go
 --> an07a_local_views.em:4:25
  |
4 |     words: Array[str] = lower.split_whitespace().collect()
  |                         ^^^^^ stored by this call
  |
  = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
  = note: the view borrows `lower`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error: could not compile due to 1 error(s)
exit=1
```

`target/redesign3/08-area-A/old/an07b_local_views_push.em`:

```ember
# AN-07: the same, pushing each word (phase 1's e06e).
fn count_words(line: str) -> int:
    lower = line.to_lower()
    words: Array[str] = []
    for w in lower.split_whitespace():
        words.push(w)
    return words.len()

fn main():
    println(count_words("A b"))
```

`target/redesign3/bin/ember.exe check an07b_local_views_push.em`:

```text
error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
 --> an07b_local_views_push.em:6:20
  |
6 |         words.push(w)
  |                    ^ stored here
  |
  = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
  = note: the view borrows `lower`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error: could not compile due to 1 error(s)
exit=1
```

`target/redesign3/08-area-A/old/an07c_inferred.em`:

```ember
# AN-07: the Python spelling, words = line.split(), no type written.
fn count_words(line: str) -> int:
    words = line.split_whitespace().collect()
    return words.len()

fn main():
    println(count_words("A b C"))
```

`target/redesign3/bin/ember.exe check an07c_inferred.em`:

```text
error[E2060]: cannot tell what `C` is here; write it out, as `.collect[T](...)`
 --> an07c_inferred.em:3:13
  |
3 |     words = line.split_whitespace().collect()
  |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |

error: could not compile due to 1 error(s)
exit=1
```

And the list of views made from a parameter is accepted inside the function, but then every caller
must give a `static` text (a run-time string is refused at the call):

`target/redesign3/08-area-A/old/an07d_param_views.em`:

```ember
# AN-07 probe: a local list of views of a parameter (not of a local).
fn count_in(text: str) -> int:
    words: Array[str] = text.split_whitespace().collect()
    return words.len()

fn count_words(line: str) -> int:
    lower = line.to_lower()
    return count_in(lower)

fn main():
    println(count_words("A b C"))
```

`target/redesign3/bin/ember.exe check an07d_param_views.em`:

```text
error[E3063]: `str` is a view, so it may not be passed to `count_in`, which stores it where only a `static` view may go
 --> an07d_param_views.em:8:21
  |
8 |     return count_in(lower)
  |                     ^^^^^ stored by this call
  |
  = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
  = note: the view borrows `lower`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error: could not compile due to 1 error(s)
exit=1
```

3. **NEW:**

```ember
fn count_words(line: StrView) -> int:
    words = line.to_lower().split_whitespace().collect()
    return words.len()
```

   A-5 for a list: the lowered text is kept until the last use of `words`, whose elements are views
   into it (Rust's `Vec<&str>` with the lifetime inferred, plus row 5's rule for the temporary, which
   Rust refuses: `E0716`, `phase3b-containers.md:562-567`). And `collect()` with nothing that names the
   result type makes an `Array` of the items, here `Array[StrView]`, as Python's `split()` makes a list.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `line` | `StrView`: a view of the caller's text | `StrView`: read only, by its type |
| `words` | none (default: the function's own new list of views into the lowered copy of `line`, kept until `words`' last use; its type `Array[StrView]` comes from its value, as row 5's `word`) | none (default: its own list; changing it changes nothing else, and its views only read) |

5. **Means exactly:** today's Ember cannot keep the list (above), so in C: a list of (pointer, length)
   pairs into the lowered text, both freed after `words`' last use, the list first:

`target/redesign3/08-area-A/c/an07/views.c`:

```c
/* AN-07 "means exactly": what the NEW code compiles to, in C.
     fn count_words(line: StrView) -> int:
         words = line.to_lower().split_whitespace().collect()     # Array[StrView], views into the lowered text
         return words.len()
   The lowered text (a temporary) is kept until the last use of `words` (row 5's rule, for a list of
   views); `words` holds (pointer, length) pairs into it, no copy of any word; both are freed after
   that last use, the list first. Run on 200,000 lines; checksum = total words + sum of word lengths. */
#include "harness.h"
#include <string.h>
#include <ctype.h>
typedef struct { const char *p; size_t n; } StrView;
typedef struct { char *p; size_t n, cap; } String;
typedef struct { StrView *data; size_t len, cap; } Views;
static String to_lower(StrView s) {
    String o = { malloc(s.n + 1), s.n, s.n + 1 };
    for (size_t i = 0; i < s.n; i++) o.p[i] = (char)tolower((unsigned char)s.p[i]);
    return o;
}
static void views_push(Views *v, StrView w) {
    if (v->len == v->cap) { v->cap = v->cap ? v->cap * 2 : 8; v->data = realloc(v->data, v->cap * sizeof(StrView)); }
    v->data[v->len++] = w;
}
static uint64_t count_words(StrView line) {
    String tmp = to_lower(line);                  /* the temporary, kept to `words`' last use */
    Views words = {0};
    size_t i = 0;
    while (i < tmp.n) {                           /* split_whitespace().collect() */
        while (i < tmp.n && isspace((unsigned char)tmp.p[i])) i++;
        size_t s = i;
        while (i < tmp.n && !isspace((unsigned char)tmp.p[i])) i++;
        if (i > s) views_push(&words, (StrView){tmp.p + s, i - s});
    }
    uint64_t r = words.len, lens = 0;
    for (size_t k = 0; k < words.len; k++) lens += words.data[k].n;   /* a read of every view */
    free(words.data);                             /* last use of `words`: the list, then the text */
    free(tmp.p);
    return r * 1000003u + lens;
}
int main(void) {
    h_start(); h_seed(7);
    char buf[256]; uint64_t sum = 0;
    for (int l = 0; l < 200000; l++) {
        size_t n = 0; int words = 1 + (int)(h_rand() % 12);
        for (int w = 0; w < words; w++) {
            int len = 1 + (int)(h_rand() % 9);
            for (int c = 0; c < len; c++) buf[n++] = (char)((h_rand() % 2 ? 'A' : 'a') + h_rand() % 26);
            buf[n++] = ' ';
        }
        sum += count_words((StrView){buf, n});
    }
    h_report("an07-views", sum);
    return 0;
}
```

   The -O2 run printed:

```text
an07-views ms=70.077 peak_kb=4036 checksum=1298076383153
```

   and the AddressSanitizer build (no report, the same checksum):

`target/redesign3/08-area-A/c/an07/asan_output.txt`:

```text
an07-views ms=383.980 peak_kb=93192 checksum=1298076383153
exit=0
```

6. **Still refused:**
   - returning the list (`fn words_of(line: StrView) -> Array[StrView]:` ... `return words`): "`words`
     points into the lowered copy of `line`, which ends when `words_of` returns. Return copies of your
     own: make the result `Array[String]` and push `String(w)` for each word; or lower the text in the
     caller and return views of it: `fn words_of(text: StrView) -> Array[StrView] from text:`."
   - keeping a word where it outlives its text (`all_words.push(w)` with `all_words` made before a loop
     over lines): "`w` points into the lowered copy of `line`, which ends with this loop step, and
     `all_words` lives longer; push your own copy: `all_words.push(String(w))`."
   - `copied w` of a view: "to make your own copy write `String(w)`" (vocabulary, `copied`).

7. **Cost:** no copy per word: a view is a pointer and a length. Last run measured views against the
   alternatives on 32 MiB of words: views 1.00, safe Rust `Vec<&str>` 1.05, one allocation per word
   (today's fix-it) 2.34x (`tasks/96/redesign/phase3b-containers.md:639-646`, `runs/a25.txt`).

8. **Kind:** a smarter compile-time solution (the lifetime and the temporary's life are inferred).

#### AN-08: A view of a temporary kept past its statement (A22)

1. **Intent:** `word = line.to_lower().trim()` then use `word.len()`. Agreed row 5.

2. **OLD:**

`target/redesign3/08-area-A/old/an08_temp_view.em`:

```ember
# AN-08: word = line.to_lower().trim() then word.len().
fn first_word_len(line: str) -> int:
    word = line.to_lower().trim()
    return word.len()

fn main():
    println(first_word_len(" Hello "))
```

`target/redesign3/bin/ember.exe check an08_temp_view.em`:

```text
error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
 --> an08_temp_view.em:3:12
  |
3 |     word = line.to_lower().trim()
  |            ^^^^^^^^^^^^^^^ a temporary value, dropped at the end of this statement
4 |     return word.len()
  |            ---- borrow later used here
  |
  = help: bind the value to a variable first, so it lives as long as the borrow
  = note: a temporary lives to the end of the statement that makes it [EXP-4]

error: could not compile due to 1 error(s)
exit=1
```

3. **NEW** (row 5, exactly as agreed):

```ember
fn first_word_len(line: StrView) -> int:
    word = line.to_lower().trim()
    return word.len()
```

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `line` | `StrView`: a view of the caller's text | `StrView`: read only |
| `word` | none (default: a read-only view of the lowered copy of `line`, which is kept until `word`'s last use; row 5) | none (default: read only) |

5. **Means exactly:** the lowered string held in a local until `word`'s last use. Accepted:

`target/redesign3/08-area-A/means/m08_bound.em`:

```ember
# AN-08 means exactly: the lowered string kept in a local until word's last use.
fn first_word_len(line: str) -> int:
    lowered = line.to_lower()
    word = lowered.trim()
    return word.len()

fn main():
    println(first_word_len(" Hello "))
```

`target/redesign3/bin/ember.exe check m08_bound.em`:

```text
exit=0
```

6. **Still refused:** returning `word` (AN-09); storing it where it outlives the lowered text (AN-07's
   second message, with `word` for `w`).

7. **Cost:** none: the lowered string is freed after `word`'s last use instead of at the end of the
   statement.

8. **Kind:** a smarter compile-time solution (agreed row 5).

#### AN-09: Returning a view of a temporary: the message names `_5`, and the agreed advice `String(word)` is refused today

1. **Intent:** `clean(line)` returns the trimmed, lowered word. Agreed row 5.

2. **OLD.** Returning the view:

`target/redesign3/08-area-A/old/an09a_return_view.em`:

```ember
# AN-09: return the cleaned word from clean(line).
fn clean(line: str) -> str:
    word = line.to_lower().trim()
    return word

fn main():
    println(clean(" Hello "))
```

`target/redesign3/bin/ember.exe check an09a_return_view.em`:

```text
error[E3060]: `_5` does not live long enough
 --> an09a_return_view.em:4:12
  |
3 |     word = line.to_lower().trim()
  |            ---- `_5` is borrowed here
  |            --------------- `_5` is a local, so its storage ends with the frame
4 |     return word
  |            ^^^^ the borrow is still live when the function returns
  |
  = help: return an owned value, take the destination as a `mut` parameter, or borrow something the caller owns
  = note: a returned reference must derive from a parameter (LT-1)

error: could not compile due to 1 error(s)
exit=1
```

The agreed advice:

`target/redesign3/08-area-A/old/an09b_string_ctor.em`:

```ember
# AN-09: the agreed advice, String(word).
fn clean(line: str) -> String:
    word = line.to_lower().trim()
    return String(word)

fn main():
    println(clean(" Hello "))
```

`target/redesign3/bin/ember.exe check an09b_string_ctor.em`:

```text
error[E2020]: `String()` takes no arguments
 --> an09b_string_ctor.em:4:12
  |
4 |     return String(word)
  |            ^^^^^^^^^^^^
  |

error: could not compile due to 1 error(s)
exit=1
```

3. **NEW** (row 5: returning the view stays refused; `String(...)` is the constructor its message
   names):

```ember
fn clean(line: StrView) -> String:
    word = line.to_lower().trim()
    return String(word)
```

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `line` | `StrView`: a view of the caller's text | `StrView`: read only |
| `word` | none (default: a read-only view of the lowered copy of `line`; row 5) | none (default: read only) |

   The result: `String(...)` says "your own copy", handed to the caller (row 5's words).

5. **Means exactly:** `String(word)` is `String.from(word)`. Accepted:

`target/redesign3/08-area-A/means/m09_string_from.em`:

```ember
# AN-09 means exactly: String(word) is String.from(word).
fn clean(line: str) -> String:
    lowered = line.to_lower()
    word = lowered.trim()
    return String.from(word)

fn main():
    println(clean(" Hello "))
```

`target/redesign3/bin/ember.exe check m09_string_from.em`:

```text
exit=0
```

6. **Still refused:** `fn clean(line: StrView) -> StrView:` with `return word`: "`word` is a view of
   the lowered copy of `line` that `line.to_lower()` made on line 2, which ends when `clean` returns;
   return `String(...)` to hand back your own copy: `return String(word)`, with the result type
   `String`." (The agreed sentence is inside it; the message names `word` and `line.to_lower()`, never
   `_5`.)

7. **Cost:** one allocation for the copy, as Python's `strip()` makes a new string. The compiler can
   save it: when the view points into a local `String` that ends at this `return` (here the lowered
   copy), `String(word)` takes over that buffer and moves the trimmed text to its start (one memory
   move) instead of allocating, which is what a C programmer writes by hand.

8. **Kind:** an understandable limitation kept (agreed row 5), with its message and the constructor
   it names.

#### AN-10: A `mut self` method on a temporary (`line.split(" ").next()`) is E2140 with no help; Rust accepts it

1. **Intent:** take the first word of a line in one expression.

2. **OLD:**

`target/redesign3/08-area-A/old/an10_mut_self_on_temp.em`:

```ember
# AN-10: take the first word of a line in one expression.
fn first_word(line: str) -> int:
    return line.split(" ").next().unwrap_or("").len()

fn main():
    println(first_word("Hello World"))
```

`target/redesign3/bin/ember.exe check an10_mut_self_on_temp.em`:

```text
error[E2140]: a `mut self` method needs a variable to write through
 --> an10_mut_self_on_temp.em:3:12
  |
3 |     return line.split(" ").next().unwrap_or("").len()
  |            ^^^^^^^^^^^^^^^
  |

error: could not compile due to 1 error(s)
exit=1
```

And the case where the change would be lost, which must stay refused with a better message:

`target/redesign3/08-area-A/old/an10b_mut_self_discarded.em`:

```ember
# AN-10: a `mut self` method on a temporary copy whose change would be lost.
@derive(Copy)
struct Enemy:
    hp: int

    fn heal(mut self):
        self.hp += 10

fn enemy_at(xs: Array[Enemy], i: int) -> Enemy:
    return xs[i]

fn main():
    es = [Enemy(hp=1)]
    enemy_at(es, 0).heal()
    println(es[0].hp)
```

`target/redesign3/bin/ember.exe check an10b_mut_self_discarded.em`:

```text
error[E2140]: a `mut self` method needs a variable to write through
  --> an10b_mut_self_discarded.em:14:5
   |
14 |     enemy_at(es, 0).heal()
   |     ^^^^^^^^^^^^^^^
   |

error: could not compile due to 1 error(s)
exit=1
```

3. **NEW:**

```ember
fn first_word(line: StrView) -> int:
    return line.split(" ").next().unwrap_or("").len()
```

   A-6: `next` changes the temporary iterator and returns the first word, which is used; the iterator
   ends with the statement.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `line` | `StrView`: a view of the caller's text | `StrView`: read only |

   The temporary iterator has no name; `next`'s own first line (`fn next(mut self)`) says it changes
   it.

5. **Means exactly:** the iterator held in a local for the statement. Accepted:

`target/redesign3/08-area-A/means/m10_temp_iter.em`:

```ember
# AN-10 means exactly: the temporary iterator held in a local for the statement.
fn first_word(line: str) -> int:
    parts = line.split(" ")
    return parts.next().unwrap_or("").len()

fn main():
    println(first_word("Hello World"))
```

`target/redesign3/bin/ember.exe check m10_temp_iter.em`:

```text
exit=0
```

6. **Still refused:** `enemy_at(es, 0).heal()` (nothing it returns is used): "`heal` changes the enemy
   that `enemy_at(es, 0)` returns, which is a temporary value that ends on this line, so the change
   would be lost. To change the enemy in `es`, write `es[0].heal()`; to change a copy of your own,
   write `e = copied es[0]` and then `e.heal()`."

7. **Cost:** none; the same code as the local.

8. **Kind:** a smarter compile-time solution.

#### AN-11: A `MutSpan` parameter cannot be written without also writing `mut`; the spec's own examples are refused; E3023's help offers `RefCell` and `class`

1. **Intent:** `integrate(pos, vel, dt)` adds each velocity to its position in place.

2. **OLD.** The spec's own example (spec 3576), refused:

`target/redesign3/08-area-A/old/an11a_mutspan_no_mut.em`:

```ember
# AN-11: integrate adds each velocity to its position in place (the spec's own example, 3576).
fn integrate(pos: MutSpan[float], vel: Span[float], dt: float):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn main():
    p = [0.0, 1.0]
    v = [1.0, 1.0]
    integrate(p.as_mut_span(), v.as_span(), 0.5)
    println(p[0])
```

`target/redesign3/bin/ember.exe check an11a_mutspan_no_mut.em`:

```text
error[E3023]: cannot mutate borrowed parameter `pos`
 --> an11a_mutspan_no_mut.em:4:9
  |
2 | fn integrate(pos: MutSpan[float], vel: Span[float], dt: float):
  |              ------------------- `pos` is borrowed here
3 |     for i in 0..pos.len():
4 |         pos[i] += vel[i] * dt
  |         ^^^^^^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut pos: MutSpan[f64]`
  = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
  = help: if the value has identity, make it a `class`
  = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

error: could not compile due to 1 error(s)
exit=1
```

With `mut`, accepted:

`target/redesign3/08-area-A/old/an11b_mutspan_mut.em`:

```ember
# AN-11: the same with `mut pos` (accepted today).
fn integrate(mut pos: MutSpan[float], vel: Span[float], dt: float):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn main():
    p = [0.0, 1.0]
    v = [1.0, 1.0]
    integrate(p.as_mut_span(), v.as_span(), 0.5)
    println(p[0])
```

`target/redesign3/bin/ember.exe check an11b_mutspan_mut.em`:

```text
exit=0
```

The spec's `[BCK-6]` example (spec 6045-6052), which the spec says is accepted:

`target/redesign3/08-area-A/old/an11c_bck6.em`:

```ember
# AN-11: the spec's [BCK-6] example (6045-6052), which it says is accepted.
fn first_even(xs: MutSpan[int]) -> ref mut int:
    for i in 0..xs.len():
        if xs[i] % 2 == 0:
            return ref mut xs[i]
    xs[0] = 0
    return ref mut xs[0]

fn main():
    a = [1, 2, 3]
    r = first_even(a.as_mut_span())
    r = 7
    println(a[1])
```

`target/redesign3/bin/ember.exe check an11c_bck6.em`:

```text
error[E3023]: cannot mutate borrowed parameter `xs`
 --> an11c_bck6.em:5:28
  |
2 | fn first_even(xs: MutSpan[int]) -> ref mut int:
  |               ---------------- `xs` is borrowed here
...
5 |             return ref mut xs[i]
  |                            ^^^^^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut xs: MutSpan[i64]`
  = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
  = help: if the value has identity, make it a `class`
  = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

error[E3023]: cannot mutate borrowed parameter `xs`
 --> an11c_bck6.em:6:5
  |
2 | fn first_even(xs: MutSpan[int]) -> ref mut int:
  |               ---------------- `xs` is borrowed here
...
6 |     xs[0] = 0
  |     ^^^^^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut xs: MutSpan[i64]`
  = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
  = help: if the value has identity, make it a `class`
  = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

error[E3023]: cannot mutate borrowed parameter `xs`
 --> an11c_bck6.em:7:20
  |
2 | fn first_even(xs: MutSpan[int]) -> ref mut int:
  |               ---------------- `xs` is borrowed here
...
7 |     return ref mut xs[0]
  |                    ^^^^^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut xs: MutSpan[i64]`
  = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
  = help: if the value has identity, make it a `class`
  = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

error: could not compile due to 3 error(s)
exit=1
```

3. **NEW** (the spec's examples, as the spec writes them):

```ember
fn integrate(pos: MutSpan[float], vel: Span[float], dt: float):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn first_even(xs: MutSpan[int]) -> ref mut int:
    for i in 0..xs.len():
        if xs[i] % 2 == 0:
            return ref mut xs[i]
    xs[0] = 0
    return ref mut xs[0]
```

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `pos` | `MutSpan`: a view of the caller's elements | `MutSpan`: its elements may be written; the view itself is not re-pointed |
| `vel` | `Span`: a view of the caller's elements | `Span`: read only |
| `dt` | none (default: a number passed in) | none (default: read) |
| `i` (both functions) | none (default: the loop's own number) | none (default: set by the loop) |
| `xs` | `MutSpan`: a view of the caller's elements | `MutSpan`: its elements may be written |

5. **Means exactly:** today's `mut pos` / `mut xs`. Accepted:

`target/redesign3/08-area-A/means/m11_mutspan.em`:

```ember
# AN-11 means exactly: today's `mut pos` (the NEW plain `pos: MutSpan[float]`).
fn integrate(mut pos: MutSpan[float], vel: Span[float], dt: float):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn first_even(mut xs: MutSpan[int]) -> ref mut int:
    for i in 0..xs.len():
        if xs[i] % 2 == 0:
            return ref mut xs[i]
    xs[0] = 0
    return ref mut xs[0]

fn main():
    p = [0.0, 1.0]
    v = [1.0, 1.0]
    integrate(p.as_mut_span(), v.as_span(), 0.5)
    a = [1, 2, 3]
    r = first_even(a.as_mut_span())
    r = 7
    println(p[0], a[1])
```

`target/redesign3/bin/ember.exe check m11_mutspan.em`:

```text
exit=0
```

   **21:** (F-05) `r = first_even(a.as_mut_span())` then `r = 7` above is today's Ember, where a name
   bound to a `ref mut` result writes through it; the NEW spelling of those two lines is
   `r = ref mut first_even(a.as_mut_span())`, `r = 7`, which writes `a[1]` (R2: the call is a place,
   R6, and `ref mut` on the line says `r` may change it). Plain `r = first_even(...)` copies the number,
   as `n = counts[0]` does.

6. **Still refused:**
   - re-pointing the view with plain `pos` (`pos = other`): "`pos` is the view the caller gave you; you
     may write its elements (`pos[i] = ...`) but not point it elsewhere. To point the caller's view
     elsewhere, write `ref mut pos: MutSpan[float]`; for a view of your own, use a new name: `rest = other`."
   - `mut pos: MutSpan[float]`: "remove `mut`: `MutSpan` already lets `integrate` write `pos[i]`; to
     point the caller's view elsewhere, write `ref mut pos: MutSpan[float]`" (C-1).
   - writing through `vel: Span[float]`: "`vel` is a read-only view; to write its elements, take it as
     `vel: MutSpan[float]`." No `RefCell` or `class` help; the type as written (`float`, not `f64`).
   - **19:** codes (R18-09): the re-point through plain `pos` and the write through `vel` are E3023 (a
     change through a name that only reads, R2, R4); `mut pos` is E0113 (section 8). What `ref mut`
     adds on a `MutSpan` (the re-point) is R4's one rule for every type, the same word that gives both
     the re-point and the change on a `shared` link (R18-01).

7. **Cost:** none; the same code as today's `mut pos`.

8. **Kind:** a readable spelling (the type says it once).

#### AN-12: Updating a map entry in place (A21): `counts.get(w).unwrap_or(0)` is E2020, there is no `get(k, default)`

1. **Intent:** count the words of a text in a map.

2. **OLD.** Three spellings (the index form is accepted; the other two are refused):

`target/redesign3/08-area-A/old/an12a_map_update.em`:

```ember
# AN-12: count the words of a text in a map (phase 1's e21, three spellings).
from std.collections import Map

fn count_index(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts[w] += 1
    return counts

fn count_get(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts[w] = counts.get(w).unwrap_or(0) + 1
    return counts

fn count_entry(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts.entry(w.to_string()).or_default() += 1
    return counts

fn main():
    println(count_get("a b a").len(), count_entry("a b a").len(), count_index("a").len())
```

`target/redesign3/bin/ember.exe check an12a_map_update.em`:

```text
error[E2020]: expected `ref i64`, found `an integer`
  --> an12a_map_update.em:13:45
   |
13 |         counts[w] = counts.get(w).unwrap_or(0) + 1
   |                                             ^ this is `an integer`
   |

error[E2140]: assignment target is not a place
  --> an12a_map_update.em:19:9
   |
19 |         counts.entry(w.to_string()).or_default() += 1
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this is a value, not somewhere to store one
   |

error: could not compile due to 2 error(s)
exit=1
```

`get_or`, which exists (`std/src/collections.em:811`), is accepted:

`target/redesign3/08-area-A/old/an12b_get_or.em`:

```ember
# AN-12: the existing get_or spelling.
from std.collections import Map

fn count_words(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts[w.to_string()] = counts.get_or(w, 0) + 1
    return counts

fn main():
    println(count_words("a b a").len())
```

`target/redesign3/bin/ember.exe check an12b_get_or.em`:

```text
exit=0
```

The Python `in` spelling is accepted too, and shows that `counts[w] = 1` with a view `w` makes a `String`
key today:

`target/redesign3/08-area-A/old/an12c_python_in.em`:

```ember
# AN-12: the Python `if w in counts` spelling (phase 1's e21c).
from std.collections import Map

fn count_in(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        if w in counts:
            counts[w] += 1
        else:
            counts[w] = 1
    return counts

fn main():
    println(count_in("a b a").len())
```

`target/redesign3/bin/ember.exe check an12c_python_in.em`:

```text
exit=0
```

3. **NEW:**

```ember
from std.collections import Map

fn count_words(text: StrView) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts[w] = counts.get_or(w, 0) + 1
    return counts
```

   Python's `counts[w] = counts.get(w, 0) + 1`, with Ember's existing `get_or`. The compiler emits
   `m[k] = m.get_or(k, d) + e` as one lookup (find the slot, or insert `d` there, then add) when both
   sides name the same `m` and the same `k` and nothing between them changes `m`.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `text` | `StrView`: a view of the caller's text | `StrView`: read only |
| `counts` | none (default: the function's own new map; returned, so handed to the caller, row 9) | none (default: its own) |
| `w` | none (default: a plain `for` name, a read-only view of one word of `text`) | none (default: read only) |

   The key: `counts[w] = ...` stores a `String` key made from `w` when `w` is new, as today's `Map`
   does (`an12c`, accepted). **14:** settled (clash 11): no word is needed; a new key is stored as the
   map's own `String`, made from the text, and keys never change in a map, so no line can tell the copy
   from the view.

5. **Means exactly:** accepted today, as two lookups:

`target/redesign3/08-area-A/means/m12_get_or.em`:

```ember
# AN-12 means exactly: counts[w] = counts.get_or(w, 0) + 1.
from std.collections import Map

fn count_words(text: str) -> Map[String, int]:
    counts = Map[String, int]()
    for w in text.split_whitespace():
        counts[w] = counts.get_or(w, 0) + 1
    return counts

fn main():
    println(count_words("a b a").len())
```

`target/redesign3/bin/ember.exe check m12_get_or.em`:

```text
exit=0
```

6. **Still refused:**
   - `counts.get(w).unwrap_or(0)`: "`counts.get(w)` gives a reference to the count, not the count; to
     read it with a default, write `counts.get_or(w, 0)`." (Today: "expected `ref i64`, found `an
     integer`".)
   - `counts.entry(w.to_string()).or_default() += 1`: AN-23 (area B: a call that returns `ref mut` as a
     place).
   - `counts[w] += 1` on a missing key stops at run time, as Python's `KeyError` (spec 6769); its
     message: "`counts` has no key `"x"`; to start a missing count at 0, write
     `counts[w] = counts.get_or(w, 0) + 1`."

7. **Cost:** one hash and one probe per word when fused, as C's find-or-insert; a new word also copies
   its text once into the map's key (C and Python store the key too). Rust's `entry(w.to_string())`
   allocates a key for every word, new or not.

8. **Kind:** a readable spelling.

#### AN-13: `SoA.get(i)` silently copies, while `Array.get(i)` gives a reference

1. **Intent:** read particle `i` from column storage, and see its current life after it changes.

2. **OLD.** The `SoA` part is accepted (a silent copy: `p` does not see the write); the `Array` part is
   refused:

`target/redesign3/08-area-A/old/an13_soa_get.em`:

```ember
# AN-13: read particle i from column storage; SoA.get copies, Array.get borrows.
from std.math import Vec3

@derive(Copy)
struct Particle:
    pos: Vec3
    life: float

fn main():
    ps = SoA[Particle].with_capacity(4)
    ps.push(Particle(pos=Vec3.ZERO, life=2.0))
    p = ps.get(0).unwrap()
    ps[0].life -= 0.5
    println(p.life, ps[0].life)
    xs = [Particle(pos=Vec3.ZERO, life=2.0)]
    q = xs.get(0).unwrap()
    xs[0].life -= 0.5
    println(q.life)
```

`target/redesign3/bin/ember.exe check an13_soa_get.em`:

```text
error[E3021]: `xs` cannot be written while it is borrowed
  --> an13_soa_get.em:17:19
   |
16 |     q = xs.get(0).unwrap()
   |         -- borrow of `xs` starts here
17 |     xs[0].life -= 0.5
   |                   ^^^ conflicting access here
18 |     println(q.life)
   |             ------ borrow later used here
   |
   = help: end the borrow before this: `q` is what keeps it alive, so shorten its last use or put it in a block of its own
   = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error: could not compile due to 1 error(s)
exit=1
```

3. **NEW:**

```ember
from std.math import Vec3

@derive(Copy)
struct Particle:
    pos: Vec3
    life: float

fn main():
    ps = SoA[Particle].with_capacity(4)
    ps.push(Particle(pos=Vec3.ZERO, life=2.0))
    p = ps.get(0).unwrap()       # names particle 0, as Array.get does; no copy
    ps[0].life -= 0.5
    println(p.life)              # 1.5
    snapshot = copied ps[0]      # a copy of your own, gathered from the columns
    ps[0].life -= 0.5
    println(snapshot.life, p.life)   # 1.5 1.0
```

   A-1 and A-7: `SoA.get(i)` gives `Option[SoARef[T]]`, a name for element `i` (its position), as
   `Array.get(i)` gives `Option[ref T]`; both see writes made through `ps[0]` (A-2: plain data). `load()`
   stays as the method `copied` runs on an `SoA` element, as `clone` is the method `copied` runs
   elsewhere (vocabulary, `.clone()` row).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `ps` | none (default: `main`'s own new column storage) | none (default: its own) |
| `p` | none (default: what `get` gives, a read-only name of particle 0, the same for `Array` and `SoA`) | none (default: read only) |
| `snapshot` | `copied`: a copy of your own | none (default: its own copy) |
| `pos`, `life` (fields) | none (default: values stored in each `Particle`, one column each) | none (default: through a name that may change the particle) |

5. **Means exactly:** `p` read by position, the copy by today's `load()`. Accepted:

`target/redesign3/08-area-A/means/m13_soa_pos.em`:

```ember
# AN-13 means exactly: p names particle 0 (read by position); a copy is load().
from std.math import Vec3

@derive(Copy)
struct Particle:
    pos: Vec3
    life: float

fn main():
    ps = SoA[Particle].with_capacity(4)
    ps.push(Particle(pos=Vec3.ZERO, life=2.0))
    i = 0
    ps[0].life -= 0.5
    println(ps[i].life)
    snapshot = ps[0].load()
    ps[0].life -= 0.5
    println(snapshot.life, ps[i].life)
```

`target/redesign3/bin/ember.exe check m13_soa_pos.em`:

```text
exit=0
```

6. **Still refused:**
   - `p.life -= 0.5`: "`p` reads particle 0 and cannot change it; to change it through `p`, write
     `p = ref mut ps[0]`."
   - `ps.swap_remove(0)` while `p` is used after it: A-2's message ("`p` names `ps[0]`, and
     `ps.swap_remove(0)` on line N may put another particle in that place. Use `p` before line N, or
     name the particle again after it: `p = ps.get(0).unwrap()`").
   - `ps[0].load()` written in a program: "write `copied ps[0]`".

7. **Cost:** less than today: `get` reads only the columns used (`p.life` reads one float), where
   today's `get` gathers every column into a new `Particle`.

8. **Kind:** a readable spelling (one meaning for `get`, and a named copy).

#### AN-14: `assert_disjoint` returns a `Result` and never stops the program

1. **Intent:** check that two views do not overlap before writing one while reading the other.

2. **OLD.** The `Result` dropped (the program would go on after an overlap), and the views taken by the
   call; the help says "clone":

`target/redesign3/08-area-A/old/an14_assert_disjoint.em`:

```ember
# AN-14: check that two views do not overlap before writing both.
from std.mem import assert_disjoint

fn blend(mut dst: MutSpan[float], src: Span[float]):
    _checked = assert_disjoint(dst, src)
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn main():
    a = [1.0, 2.0]
    b = [3.0, 4.0]
    blend(a.as_mut_span(), b.as_span())
    println(a[0])
```

`target/redesign3/bin/ember.exe check an14_assert_disjoint.em`:

```text
error[E3040]: `dst` has been moved out of
 --> an14_assert_disjoint.em:6:17
  |
5 |     _checked = assert_disjoint(dst, src)
  |                                --- moved here
6 |     for i in 0..dst.len():
  |                 ^^^ used here after the move
  |
  = help: clone the value, or borrow it instead of moving it
  = note: a move gives the value away; the old owner cannot use it again

error: could not compile due to 1 error(s)
exit=1
```

3. **NEW:**

```ember
from std.mem import assert_disjoint, check_disjoint

fn blend(dst: MutSpan[float], src: Span[float]):
    assert_disjoint(dst, src)
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn blend_any(dst: MutSpan[float], src: Span[float]):
    match check_disjoint(dst, src):
        Ok((d, s)):
            for i in 0..d.len():
                d[i] = d[i] * 0.5 + s[i] * 0.5
        Err((d, s)):
            for i in 0..d.len():
                d[i] = (d[i] + s[i]) * 0.5
```

   `assert_disjoint(a, b)` (vocabulary: today's `assert_disjoint_or_panic`) is a statement: it stops
   the program if the two views overlap, and afterwards the compiler treats `dst` and `src` as not
   overlapping (noalias) for as long as neither name is pointed elsewhere; plain parameters never are
   (AN-11). `check_disjoint` is today's `assert_disjoint`, renamed (vocabulary): it reports and hands
   both views back.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `dst` | `MutSpan`: a view of the caller's elements | `MutSpan`: its elements may be written |
| `src` | `Span`: a view of the caller's elements | `Span`: read only |
| `i` (each loop) | none (default: the loop's own number) | none (default: set by the loop) |
| `d`, `s` (each arm) | none (default: `dst` and `src` handed back by `check_disjoint`, with their types, `MutSpan[float]` and `Span[float]`) | by those types: `d`'s elements may be written, `s` reads |

5. **Means exactly:** today's `assert_disjoint_or_panic`, which takes the views and gives them back.
   Accepted:

`target/redesign3/08-area-A/means/m14_assert.em`:

```ember
# AN-14 means exactly: assert_disjoint(dst, src) is today's assert_disjoint_or_panic.
from std.mem import assert_disjoint_or_panic

fn blend(mut dst: MutSpan[float], src: Span[float]):
    (d, s) = assert_disjoint_or_panic(dst, src)
    for i in 0..d.len():
        d[i] = d[i] * 0.5 + s[i] * 0.5

fn main():
    a = [1.0, 2.0]
    b = [3.0, 4.0]
    blend(a.as_mut_span(), b.as_span())
    println(a[0])
```

`target/redesign3/bin/ember.exe check m14_assert.em`:

```text
exit=0
```

6. **Still refused:**
   - `_checked = check_disjoint(dst, src)` then using `dst`, or the result unused: "`check_disjoint` only
     reports whether `dst` and `src` overlap; it does not stop the program, and it takes both views and
     hands them back in its result. To stop when they overlap, write `assert_disjoint(dst, src)`; to
     handle both cases, write `match check_disjoint(dst, src):`." (Today: E3040 with "clone the value".)
   - two single-object references (`E3095`, `[DSJ-4]`): kept.

7. **Cost:** two compares (`[DSJ-1]`), none when the compiler already knows the ranges apart
   (`[DSJ-5]`). It buys noalias: two spans of one list ran 1.53x of C without it and 1.00x to 1.07x
   with it (`tasks/96/redesign/phase3b-containers.md:346-355`).
   **21:** (F-03) The two spans here are of `float`s, so an overlap frees nothing and `blend_any` keeps
   two spans of one list (R4). With elements that own memory (`MutSpan[String]`), R4 treats a
   `MutSpan` argument and another view argument as two places: proved, or one overlap test before the
   call (two compares, the cost above).

8. **Kind:** a readable spelling (an `assert` stops the program, as in Python).

#### AN-15: The ECS query marker `Mut[T]` is a second spelling of `ref mut T`

1. **Intent:** a system asks for changeable positions and read-only velocities.

2. **OLD.** The marker, accepted:

`target/redesign3/08-area-A/old/an15_query_mut.em`:

```ember
# AN-15: a system asks for changeable positions and read-only velocities (spec XII.4 example).
from std.ecs import World, Query, Mut, Component

@derive(Copy, Component)
struct Position:
    x: float

@derive(Copy, Component)
struct Velocity:
    dx: float

fn integrate(mut q: Query[(Mut[Position], Velocity)], dt: float):
    for pos, vel in q.iter_mut():
        pos.x += vel.dx * dt

fn main():
    world = World()
    e = world.create()
    world.add(e, Position(x=0.0))
    world.add(e, Velocity(dx=1.0))
    world.run(integrate, 0.016)
```

`target/redesign3/bin/ember.exe check an15_query_mut.em`:

```text
exit=0
```

`ref mut Position` in the query, refused: 19 errors for one wrong type argument (6 at the program's
lines, 10 inside `std/src/ecs.em`, 3 with no line; AN-16):

`target/redesign3/08-area-A/old/an15b_query_ref_mut.em`:

```ember
# AN-15: the same query written with `ref mut Position` (what the marker means).
from std.ecs import World, Query, Component

@derive(Copy, Component)
struct Position:
    x: float

@derive(Copy, Component)
struct Velocity:
    dx: float

fn integrate(mut q: Query[(ref mut Position, Velocity)], dt: float):
    for pos, vel in q.iter_mut():
        pos.x += vel.dx * dt

fn main():
    world = World()
    e = world.create()
    world.add(e, Position(x=0.0))
    world.add(e, Velocity(dx=1.0))
    world.run(integrate, 0.016)
```

`target/redesign3/bin/ember.exe check an15b_query_ref_mut.em`:

```text
error[E2040]: `(ref mut Position, Velocity)` does not implement `std.ecs.QueryParam`, which `Query`'s `Q` requires
  --> an15b_query_ref_mut.em:12:21
   |
12 | fn integrate(mut q: Query[(ref mut Position, Velocity)], dt: float):
   |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |

error[E2020]: a tuple pattern cannot match `<error>`
  --> an15b_query_ref_mut.em:13:9
   |
13 |     for pos, vel in q.iter_mut():
   |         ^^^^^^^^
   |

error[E1010]: cannot find `pos` in this scope
  --> an15b_query_ref_mut.em:14:9
   |
14 |         pos.x += vel.dx * dt
   |         ^^^
   |

error[E1010]: cannot find `pos` in this scope
  --> an15b_query_ref_mut.em:14:9
   |
14 |         pos.x += vel.dx * dt
   |         ^^^
   |

error[E1010]: cannot find `vel` in this scope
  --> an15b_query_ref_mut.em:14:18
   |
14 |         pos.x += vel.dx * dt
   |                  ^^^
   |

error[E2040]: `(ref mut Position, Velocity)` does not implement `std.ecs.QueryParam`, which `Q` requires
  --> an15b_query_ref_mut.em:21:5
   |
21 |     world.run(integrate, 0.016)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |

error[E2040]: `(ref mut Position, Velocity)` does not implement `std.ecs.QueryParam`, which `QueryIter`'s `Q` requires
    --> C:\Users\ism19\Code\ember\std\src\ecs.em:1099:16
     |
1099 |         return QueryIter[Q](view=ref self.view, entities=self.entities, count=self.count, at=0)
     |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |

error[E2040]: `(ref mut Position, Velocity)` does not implement `std.ecs.QueryParam`, which `QueryIterMut`'s `Q` requires
    --> C:\Users\ism19\Code\ember\std\src\ecs.em:1102:16
     |
1102 |         return QueryIterMut[Q](view=ref mut self.view, entities=self.entities, count=self.count, at=0)
     |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |

error[E2020]: `<error>` has no associated function `get`
    --> C:\Users\ism19\Code\ember\std\src\ecs.em:1125:19
     |
1125 |             match Q.Ops.get(self.view, e):
     |                   ^^^^^^^^^^^^^^^^^^^^^^^
     |

error[E2040]: `(ref mut Position, Velocity)` has no associated type `Ops`
    --> C:\Users\ism19\Code\ember\std\src\ecs.em:1125:21
     |
1125 |             match Q.Ops.get(self.view, e):
     |                     ^^^
     |

error[E1010]: `Some` does not name a constructor of `<error>`
    --> C:\Users\ism19\Code\ember\std\src\ecs.em:1126:17
     |
1126 |                 Some(item):
     |                 ^^^^^^^^^^
     |

error[E1010]: cannot find `item` in this scope
    --> C:\Users\ism19\Code\ember\std\src\ecs.em:1127:33
     |
1127 |                     return Some(item)
     |                                 ^^^^
     |

error[E2020]: `<error>` has no associated function `get_mut`
    --> C:\Users\ism19\Code\ember\std\src\ecs.em:1148:19
     |
1148 |             match Q.Ops.get_mut(self.view, e):
     |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |

error[E2040]: `(ref mut Position, Velocity)` has no associated type `Ops`
    --> C:\Users\ism19\Code\ember\std\src\ecs.em:1148:21
     |
1148 |             match Q.Ops.get_mut(self.view, e):
     |                     ^^^
     |

error[E1010]: `Some` does not name a constructor of `<error>`
    --> C:\Users\ism19\Code\ember\std\src\ecs.em:1149:17
     |
1149 |                 Some(item):
     |                 ^^^^^^^^^^
     |

error[E1010]: cannot find `item` in this scope
    --> C:\Users\ism19\Code\ember\std\src\ecs.em:1150:33
     |
1150 |                     return Some(item)
     |                                 ^^^^
     |

error[E2040]: `(ref mut Position, Velocity)` does not implement `std.ecs.QueryParam`, which `QueryIter`'s `Q` requires

error[E2040]: `(ref mut Position, Velocity)` does not implement `std.ecs.QueryParam`, which `QueryIterMut`'s `Q` requires

error[E2040]: `(ref mut Position, Velocity)` does not implement `std.ecs.QueryParam`, which `Query`'s `Q` requires

error: could not compile due to 19 error(s)
exit=1
```

3. **NEW:**

```ember
from std.ecs import World, Query, Component

@derive(Copy, Component)
struct Position:
    x: float

@derive(Copy, Component)
struct Velocity:
    dx: float

fn integrate(ref mut q: Query[(ref mut Position, Velocity)], dt: float):
    for ref mut pos, ref vel in q:
        pos.x += vel.dx * dt
```

   **19:** (R18-15) `ref vel` was plain `vel`: a word before the first of several names belongs to that
   name only, so every name in the list writes its own (R3); `ref` says the default, a read-only view
   (R2). The query's type keeps `(ref mut Position, Velocity)`: inside the parentheses each element is
   a type of its own. The C is unchanged.

   `ref mut Position` in the query's type says the system changes positions (it sets the system's write
   set, `[ECS-4]`, as `Mut[Position]` does today); `ref mut pos` on the loop name says this name
   changes them (C-1). `std.ecs` maps `ref mut T` to the item `ref mut T` exactly as it maps `Mut[T]`
   today (`std/src/ecs.em:406`, the `QueryParam` interface).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `q` | `ref mut`: the query the world made for this run | `ref mut`: yes |
| `pos` | `ref mut`: the `Position` of the current entity | `ref mut`: yes |
| `vel` | **19:** `ref`: a read-only view of the entity's `Velocity` (was none (default: a plain loop name)) | `ref`: read only |
| `dt` | none (default: a number passed in) | none (default: read) |
| `x`, `dx` (fields) | none (default: values stored in each component) | none (default: through `pos`) |

5. **Means exactly:** today's `Mut[Position]`, the first program in point 2 (`m15_query_mut.em` is
   that file, copied), accepted:

```text
exit=0
```

6. **Still refused:**
   - `Mut[Position]`: "write `ref mut Position` in the query".
   - `for pos, vel in q:` then `pos.x += ...`: "`pos` reads each `Position`; to change it, write
     `for ref mut pos, ref vel in q:`." (**19:** was `for ref mut pos, vel in q:`, R3's list rule)
   - **19:** `for ref mut pos, vel in q:`: "`ref mut` here belongs to `pos` only; say what `vel` is:
     `for ref mut pos, ref vel in q:`, or `ref mut vel` to change it too" (R3, the same advice as
     AN-54's capture list).
   - `for pos, ref mut vel in q:` when the query says `Velocity`: "the query reads `Velocity` only; to
     change it, ask for it: `Query[(ref mut Position, ref mut Velocity)]`."

7. **Cost:** none; the same code as `Mut[Position]`.

8. **Kind:** a readable spelling (one spelling for "changes it").

#### AN-16: Messages use the compiler's names, not the program's, and many helps do not give the line to write (A17)

1. **Intent:** every refusal names what was refused in the program's own names, says why in one line,
   and gives the exact line to write.

2. **OLD.** The outputs pasted under AN-01 to AN-15 above, all real. The failures, by kind:

| message (answer) | today | what fails |
|---|---|---|
| E3022 (AN-01, AN-02) | "`*enemies[0]` is already mutably borrowed"; help "shorten its last use or put it in a block of its own" | `*enemies[0]` is not the program's spelling; the help gives no line |
| E3040 (AN-01) | "`enemies` is borrowed after it has been moved out of" for `boss = enemies[0]` | `enemies` was not moved; `[EXP-6]` says E3011 |
| E3022 (AN-03) | "`*bodies[…]` is already mutably borrowed" | not the program's names (`bodies[i]`, `bodies[j]`) |
| E3020 (AN-04, AN-05) | "cannot mutate `*enemies` while it is borrowed by this loop"; help "collect the indices or values first" | `*enemies`; no line to write |
| E3021 (AN-04) | "take the borrow as `ref mut` if the callee must write" for `for ref mut e in enemies:` | the program wrote `ref mut`; the loop must honour it (C-1) |
| E3063 (AN-07) | "store an owned copy — `String` for `str`, ... or store a `u32` index or a `Handle[T]`" | four options, none written as the line |
| E2060 (AN-07) | "cannot tell what `C` is here" | `C` is the library's type parameter, not the program's |
| E3060 (AN-08) | "bind the value to a variable first" | the right idea, no line |
| E3060 (AN-09) | "`_5` does not live long enough" | a compiler-internal name |
| E2020 (AN-09) | "`String()` takes no arguments" | refuses the agreed advice |
| E2140 (AN-10) | "a `mut self` method needs a variable to write through" | no help at all |
| E3023 (AN-11) | "declare this parameter `mut pos: MutSpan[f64]`"; helps `RefCell`, `class` | `f64` for `float`; two helps that do not apply |
| E2020 (AN-12) | "expected `ref i64`, found `an integer`" | `i64` for `int`; no line |
| E3040 (AN-14) | "clone the value, or borrow it instead of moving it" | the fix is a different function |
| E2040 (AN-15) | 19 errors for one wrong type argument, 10 inside `std/src/ecs.em` | one mistake, reported at the program's line once |
| (area D, `phase3b-graphs-callbacks.md:1147-1150`) | a lambda named `Callable0` | an internal type name |

3. **NEW:** the messages written in point 6 of AN-01 to AN-15, under rule A-8:
   - **M-1, names:** variables as written; an element as `enemies[0]`, `bodies[i]` and `bodies[j]` (never
     `*enemies[0]` or `*bodies[…]`); a temporary by its expression, `line.to_lower()` (never `_5`); types as
     the program wrote them, `float`, `int` (never `f64`, `i64`); a lambda by its text (never
     `Callable0`); a library's type parameter by what it stands for in the call (`collect` "cannot
     tell what list to make; write `words: Array[StrView] = ...`", not `C`).
   - **M-2, why:** one line, in the program's words ("`enemies` may not change while this loop walks
     it").
   - **M-3, the line to write:** always, in the program's names, once (`for ref mut e in enemies:`).
   - **M-4, one mistake, one message:** an error that the library would see as many errors is reported
     once, at the program's line; helps that do not apply are not printed.

4. **The owner's test** (**14:** completed; the automatic check found it applied to no name). AN-16's NEW
   is message text and binds no name itself; every name it brings into a program is in a line one of the
   messages tells the programmer to write. Each such line, and each name in it, with the same words as in
   the answer that refuses:

| line a message gives (answer) | name | what it is | may it change it |
|---|---|---|---|
| `boss = ref mut enemies[0]` (AN-01, AN-02) | `boss` | `ref mut`: element 0 of `enemies` (a position, A-1) | `ref mut`: yes |
| the same line | `enemies` | `ref mut` on the parameter (`ref mut enemies: Array[Enemy]`): the caller's list | `ref mut`: yes |
| `boss = copied enemies[0]` (AN-01) | `boss` | `copied`: a copy of your own | none (default: its own copy; changing it changes no enemy) |
| `n = String(boss.name.trim())` (AN-01) | `n` | `String(...)`: your own text (row 5) | none (default: its own) |
| `if i != j:` (AN-03) | `i`, `j` | none (default: numbers the function was given or counts, its own) | none (default: read) |
| `spawned.push(Enemy(hp=5, splits=false))`, `enemies.extend(spawned)` (AN-04) | `spawned` | none (default: the function's own new list) | none (default: its own; handed over by `extend`, row 9) |
| `enemies.retain(fn(e) => e.hp > 0)` (AN-04) | `e` | none (default: a lambda parameter, a read-only view of each element, C-1) | none (default: read only) |
| `for ref mut e in enemies:` (AN-04, M-3) | `e` | `ref mut`: the current element of `enemies` | `ref mut`: yes |
| `dead.push(i)`, then `while dead.len() > 0: on_death(enemies, dead.pop().unwrap())` (AN-05) | `dead` | none (default: the function's own new list of positions) | none (default: its own) |
| the same line | `on_death` | its type `fn(ref mut Array[Enemy], int)`: a function the caller gave | its type's `ref mut`: the function may change the list it is given; `on_death` itself is only called |
| `fn words_of(text: StrView) -> Array[StrView] from text:` (AN-07) | `text` | `StrView`: a view of the caller's text | `StrView`: read only |
| `all_words.push(String(w))` (AN-07) | `all_words` | none (default: the function's own list, made before the loop) | none (default: its own) |
| the same line | `w` | none (default: a plain loop name, a read-only view of one word) | none (default: read only) |
| `words: Array[StrView] = ...` (M-1, AN-07) | `words` | its type `Array[StrView]`: the function's own list of views | none (default: its own; its views only read) |
| `return String(word)` (AN-09) | `word` | none (default: a read-only view of the lowered copy, row 5) | none (default: read only) |
| `es[0].heal()` (AN-10) | `es` | none (default: `main`'s own list) | none (default: its own; `heal`'s first line, `mut self`, says it changes `es[0]`) |
| `e = copied es[0]`, then `e.heal()` (AN-10) | `e` | `copied`: a copy of your own | none (default: its own copy) |
| `ref mut pos: MutSpan[float]` (AN-11) | `pos` | `ref mut`: the caller's own view; `MutSpan`: of the caller's elements | `ref mut`: may point the caller's view elsewhere; `MutSpan`: may write its elements |
| `rest = other` (AN-11) | `rest` | none (default: a new view of your own, the value of the view `other`) | none (default: by its type, as `other`) |
| `vel: MutSpan[float]` (AN-11) | `vel` | `MutSpan`: a view of the caller's elements | `MutSpan`: its elements may be written |
| `counts[w] = counts.get_or(w, 0) + 1` (AN-12) | `counts` | none (default: the function's own map) | none (default: its own) |
| the same line | `w` | none (default: a plain loop name, a read-only view of one word) | none (default: read only) |
| `p = ref mut ps[0]` (AN-13) | `p` | `ref mut`: particle 0 of `ps` (a position) | `ref mut`: yes |
| `p = ps.get(0).unwrap()` (AN-13) | `p` | none (default: what `get` gives, a read-only name of particle 0) | none (default: read only) |
| `snapshot = copied ps[0]` (AN-13) | `snapshot` | `copied`: a copy of your own | none (default: its own) |
| `assert_disjoint(dst, src)`, `match check_disjoint(dst, src):` (AN-14) | `dst` | `MutSpan` (its type): a view of the caller's elements | `MutSpan`: its elements may be written |
| the same lines | `src` | `Span` (its type): a view of the caller's elements | `Span`: read only |
| `for ref mut pos, ref vel in q:` (AN-15; **19:** was `vel`) | `pos` | `ref mut`: the current entity's `Position` | `ref mut`: yes |
| the same line | `vel` | `ref`: a read-only view of the entity's `Velocity` | `ref`: read only |
| `fn integrate(ref mut q: Query[(ref mut Position, ref mut Velocity)], dt: float):` (AN-15) | `q` | `ref mut`: the query the world made for this run | `ref mut`: yes (walks it) |
| the same line | `dt` | none (default: a number passed in) | none (default: read) |

   31 rows, 25 distinct names. Every name has its words, or a stated default that never changes anything
   it does not own, never aliases silently and never copies silently.

5. **Means exactly:** the same programs are refused (or accepted) for the same reasons as in each
   answer's point 6; the codes stay; only the text changes. The translation is the table in point 2
   (today's text) against point 6 of each answer (the new text).

6. **Still refused:** the refusals of AN-01 to AN-15, each with its message there.

7. **Cost:** none while the program runs.

8. **Kind:** a readable spelling (of the refusals).

## 11. Area B: references kept or returned, locks, objects still being built (AN-17 .. AN-33)

Copied from `RUN3/09-area-B.md` by `build.py`; the combiner's changes are marked **14:** and listed in section 4. File names in this area's text are relative to `target/redesign3/09-area-B/` unless they say otherwise; "section N" inside it means the area report's own section N.

### Terms of area B

- **field clause**: what a method's first line says after its parameters: `changes <fields>` (row 3:
  the fields it may change) and `reads <fields>` (row 7: the fields it reads). Both are checked
  against the body and at every call.
- **one-owner object**: a `class` object with one owner. After 06's C-3 a class object is not shared
  by `p2 = player`; a second owner is written `shared` (area C). So every second name for a one-owner
  object is a `ref`/`ref mut` name the compiler can see.
- **access word**: today's run-time check on class fields (`[EXC-19]`, spec 2712-2730): one word per
  non-`Copy` field, tested on each long-term access, set for a whole `mut self` call.
- **view**: a value that points into something else and may not outlive it (`ref T`, `Span[T]`,
  `StrView`, a struct holding one).
- **source** of a result: what a returned view points into.
- **guard**: what `m.lock()`, `rw.read()`, `rw.write()`, `c.read()`, `c.write()` return; it holds the
  lock (or the run-time check) until its `with` block ends.
- **own local**: a name made in this function by the value on its right (`total = 0`,
  `d = Door()`); only this function can change it and nothing else refers to it. In the owner's test
  it is written "none (default: own local)": it never aliases and never copies silently.
- **the owner's test count**: for the form, a name is counted "without a word" when at least one of
  the two questions (what it is; whether it may change) is answered by a stated default instead of a
  word in the line.

### Rules of area B (items 1 .. 8)

1. **A method's first line says what it touches.** Plain `self` reads; `self ... changes a, b` may
   change exactly `a` and `b` (row 3); `mut self` may change any field (one meaning for every type,
   06 settlement 20); `self ... reads a, b` reads exactly `a` and `b` (row 7). `changes` and `mut self`
   are always written. A method with no `reads` reads, as far as a caller in another package knows,
   every field; inside the package the compiler uses what the body reads (today's `[BRW-10]` for
   reads only), and the error message gives the `reads` line when a caller needs it.
2. **These are checked when compiling, for every one-owner object, class or struct.** The class
   rule "`mut self` holds a write access on every field for the whole call" and the access words
   that check it while running go for one-owner objects: every other name for the object is a
   `ref`/`ref mut` name the borrow checker sees. Area C decides what a `shared` object keeps.
   **14:** area C decided: nothing is checked while running for `shared` objects either (C6; R9).
3. **A name that changes someone else's value says `ref mut` on the name** (06's C-1): a parameter
   (`ref mut c: Counter`), a loop name (`for ref mut p in players:`), a pattern name (row 8). A plain
   name never changes anything, including a `Cell` field (section 4 rules on 06's C-10).
4. **A returned view says where it points when the default is not enough.** Default (stated, as
   today's `[LT-1]`): a method's result points into the object; otherwise into the one parameter it
   could, or into all of them. `from <name>` after the result type narrows it (replaces `@borrows`).
   A `ref mut` parameter is always a source, so `smaller(ref mut a, ref mut b) -> ref mut int` needs
   nothing written.
5. **A call that returns `ref mut T` is a place**: `first(xs) += 1`.
6. **Views may be kept in lists and fields when their source is named.** A local list of views is a
   view itself (nothing written, as row 5's `word`); a field of views says `from <let field>` or
   `from static`. A `let` field never changes after `init`, so views into it stay valid for the
   object's whole life with no tracking.
7. **A view made through a guard points into the guard** (row 4), so it ends with the `with` block.
8. **`init` may set fields with plain `self`; a helper it calls says what it reads** (row 7).

What still runs while the program runs, in area B: Ember's bounds checks (as today, not this area's);
`@cached`'s flag test (one branch per call, the hand-written C cache); the one type-id compare of a
downcast (`cast ... as Pool[T]`, as today). Nothing else: every rule above is checked when compiling.

### Area B's rulings on `@cached`, `from` and `Cell` (its section 4)

1. **`@cached`'s result type**: a `Copy` result is returned by value; any other result is returned as a
   `ref T` view into the kept value, which is a view of the object (the default source), so the fields
   it reads cannot change while the view is used. A `@cached` method's object is not sent to another
   thread unless area E gives the slot a once-flag (AN-21).
2. **`from` on a field, the heap-block condition**: the named field must be `let` and a `String` or
   `Array` (contents outside the header, `ember_vec {ptr, len, cap}`; ADR-141 frame buffers only for
   lists that never leave their function). A fixed array or inline struct field is refused (AN-25).
3. **`Cell` (06's C-10)**: the default "a plain name never changes anything" holds without an
   exception. Setting a `Cell` field needs a name that may change it (`ref mut s`, `mut self`,
   `changes frame`), or a value that is `shared` (area C decides whether one of several owners may
   change it, AN-48; a `Cell` is the one-thread way it would). Cost: none, `Cell.set` is a plain store.
   What `Cell` still gives: changing one field while other `ref` views of the same value are alive is
   still refused; it lets `shared` owners set a value whole, never lend a reference into it.

### The answers of area B

#### AN-17: a method changes one field while a loop walks another (A1). Agreed row 3.

**1. Intent:** `tick` takes 1 hit point from every player and, when one dies, `note` appends a line to
the game's log; `note` only appends to `log`.

**2. OLD** (`o17a_class_note_in_loop.em`, phase 1's `e01d`):

```ember
class Player:
    name: String
    hp: int

class Game:
    players: Array[Player]
    log: Array[String]
    frame: int = 0

    fn note(mut self, msg: String):
        self.log.push(f"[{self.frame}] {msg}")

    fn tick(self):
        for p in self.players:
            p.hp -= 1
            if p.hp <= 0:
                self.note(f"{p.name} died")
```

```text
error[E3020]: cannot mutate `self.players` while it is borrowed by this loop
  --> o17a_class_note_in_loop.em:18:17
   |
15 |         for p in self.players:
   |                  ------------ `self.players` borrowed here for the whole loop
...
18 |                 self.note(f"{p.name} died")
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow here
   |
   = help: collect the indices or values first, then mutate; use `retain` or `drain` when the operation fits
   = help: or iterate over a fixed index range and mutate only after taking the current value
   = note: a `for` loop borrows the iterable until the loop ends (CTL-2)

error: could not compile due to 1 error(s)
exit=1
```

The message says `note` changes `self.players`; it does not, it changes `log`. (The same program as
a struct with a `pub` method compiles today inside one package, `o17b_struct_pub_note.em`, `exit=0`;
`[BRW-10]` says a `pub` method borrows all of `self` for callers in other packages.)

**3. NEW:**

```ember
class Player:
    name: String
    hp: int

class Game:
    players: Array[Player]
    log: Array[String]
    frame: int = 0

    fn note(self, msg: String) changes log:
        self.log.push(f"[{self.frame}] {msg}")

    fn tick(self) changes players, log:
        for ref mut p in self.players:
            p.hp -= 1
            if p.hp <= 0:
                self.note(f"{p.name} died")

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    g.tick()
    println(g.log)
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `name`, `hp` (fields of `Player`) | none (default: the player's own value, by its type) | none (default: a field changes only through a method whose first line names it, or `mut self`, or a `ref mut` name for the player) |
| `players`, `log`, `frame` (fields of `Game`) | none (default: the game's own list / number) | none (default as above): `changes players, log` on `tick`, `changes log` on `note` |
| `self` in `note` | `self`: the game `note` was called on | `changes log`: only `log` |
| `msg` | none (default: a read-only view of the caller's `String`) | none (default: a plain parameter never changes anything) |
| `self` in `tick` | `self` | `changes players, log` |
| `p` | `ref mut`: a name for the list's own element | `ref mut`: yes |
| `g` | none (default: own local, made by `Game(...)`) | none (default: own local) |

**5. Means exactly** (`m17_tick_note.em`): one owner, so today's struct rules give the same
compile-time check (the loop changes `players`, `note` touches only `log`, `[BRW-10]`):

```ember
struct Player:
    name: String
    hp: int

struct Game:
    players: Array[Player]
    log: Array[String]
    frame: int = 0

    fn note(mut self, msg: String):
        self.log.push(f"[{self.frame}] {msg}")

    fn tick(mut self):
        for p in self.players.iter_mut():
            p.hp -= 1
            if p.hp <= 0:
                self.note(f"{p.name} died")

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    g.tick()
    println(g.log)
```

```text
===== m17_tick_note.em
exit=0
```

**6. Still refused:**
- `note`'s body changing a field it does not name (`self.frame += 1`): "`note` changes `self.frame`,
  which its first line does not name; write `fn note(self, msg: String) changes log, frame:`".
- `tick` calling `note` without naming `log`: "`tick` calls `note`, which changes `log`; add it to
  `tick`'s first line: `fn tick(self) changes players, log:`".
- Inside the loop, a method that changes the list being walked (`self.add_player(...)` with
  `changes players`): "`tick` is walking `self.players` (line 14), and `add_player` changes
  `players`; collect the new players in a list and add them after the loop" (the understandable
  limitation, JOB: delete or add after the loop).
- From another package, a call to a method with no `reads` inside a loop that changes a field:
  "`note` may read any field of `Game`, including `players`, which this loop is changing; if it does
  not, say what it reads: `fn note(self, msg: String) reads frame changes log:`".

**7. Cost:** none; the check is when compiling. Today's class version, if it compiled, would pay
access-word checks while running (`[EXC-19]`, spec 2712-2730: a load and a compare per long-term
access, a check per field on entry to a `mut self` method).

**8. Kind:** a smarter compile-time solution. Rust refuses this program (E0499, a second `&mut self`
while the loop borrows `self.players`); the written field set lets it through, checked.

#### AN-18: `mut self` on a class means "hold a write access on every field for the whole call"

**1. Intent:** a class method says that it may change its object, and the program is checked when
compiling, not while running.

**2. OLD** (`o18_class_mut_self_two_handles.em`):

```ember
class Game:
    players: Array[String]
    log: Array[String]

    fn note(mut self, owned msg: String):
        self.log.push(msg)

fn tick(g: Game, other: Game):
    for p in g.players:
        other.note(p.clone())

fn main():
    g = Game(players=["orc"], log=[])
    tick(g, g)
    println(g.log)
```

```text
===== o18_class_mut_self_two_handles.em
exit=0
```

Accepted. By `[CLS-7]` (spec 1740-1742) and `[EXC-19]` (spec 2718-2721), `other.note(...)` "checks
every field word of the object ... at entry ... marks each as written": the loop holds a read access
on `players` of the same object, so by those rules the program would stop with "exclusivity
violation: write access to Game.players while a read access to Game.players is active" (`[EXC-1]`,
spec 2706) although `note` never touches `players` (read from the spec; the program was not run, as
the rules forbid). The conflict is found only while running, and it is a false one.

**3. NEW:**

```ember
class Game:
    players: Array[String]
    log: Array[String]
    score: int = 0

    fn note(self, msg: String) changes log:
        self.log.push(copied msg)

    fn reset(mut self):
        self.players = []
        self.log = []
        self.score = 0

fn tick(ref mut g: Game):
    for p in g.players:
        g.note(f"{p} moved")
    g.reset()
```

`mut self` keeps one meaning for every type: "this method may change any field"; it is checked when
compiling, like `changes`, and nothing is marked while running. The OLD `tick(g, g)` cannot be
written for a one-owner object without saying it: `fn tick(g: Game, ref mut other: Game)` called as
`tick(g, g)` is a read and a change of the same object in one call (refused below).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `players`, `log`, `score` | none (default: the game's own values) | none (default: through `changes` or `mut self` only) |
| `self` in `note` | `self` | `changes log` |
| `msg` | none (default: read-only view of the caller's `String`) | none (default: never) |
| `self` in `reset` | `self` | `mut`: any field |
| `g` | `ref mut`: the caller's game | `ref mut`: yes |
| `p` | none (default: a read-only view of the list's element, as row 8's plain `path`) | none (default: never) |

**5. Means exactly** (`m18_mut_self_static.em`), today's struct `mut self`, checked when compiling:

```ember
struct Game:
    players: Array[String]
    log: Array[String]
    score: int = 0

    fn note(mut self, msg: String):
        self.log.push(msg.clone())

    fn reset(mut self):
        self.players = []
        self.log = []
        self.score = 0

fn tick(mut g: Game):
    for p in g.players:
        g.note(f"{p} moved")
    g.reset()

fn main():
    g = Game(players=["orc"], log=[])
    tick(g)
    println(g.log)
```

```text
===== m18_mut_self_static.em
exit=0
```

**6. Still refused:**
- `g.reset()` inside the loop: "`tick` is walking `g.players` (line 2), and `reset` may change every
  field of `Game`; call it after the loop, or, if it only changes some fields, name them:
  `fn reset(self) changes score:`".
- `tick(g, g)` with `fn tick(g: Game, ref mut other: Game)`: "`tick` would read `g` and change it
  through `other` in the same call; pass it once: `fn tick(ref mut g: Game)`".
- `fn init(mut self)`: see AN-32.

**7. Cost:** none for one-owner objects: no access words, no check on entry (today: one word per
non-`Copy` field and a check per field on entry, `[EXC-19]`). A `shared` object keeps whatever area C
decides (AN-48); this answer does not remove a check there.

**8. Kind:** a smarter compile-time solution.

#### AN-19: plain `self` in a class method changes fields with no word

**1. Intent:** `open` sets the door's angle to 90.

**2. OLD** (`o19_plain_self_writes.em`):

```ember
class Door:
    angle: f64 = 0.0

    fn open(self):
        self.angle = 90.0

fn main():
    d = Door()
    d.open()
    println(d.angle)
```

```text
===== o19_plain_self_writes.em
exit=0
```

Accepted; nothing on `open`'s first line says it changes the door.

**3. NEW:**

```ember
class Door:
    angle: f64 = 0.0

    fn open(self) changes angle:
        self.angle = 90.0

fn main():
    d = Door()
    d.open()
    println(d.angle)
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `angle` | none (default: the door's own number) | none (default: through `changes angle` or `mut self`) |
| `self` | `self` | `changes angle` |
| `d` | none (default: own local) | none (default: own local) |

**5. Means exactly** (`m19_open.em`): today's class method, which does exactly this (a `Copy` field
write is never checked, `[EXC-17]`):

```ember
class Door:
    angle: f64 = 0.0

    fn open(self):
        self.angle = 90.0
```

```text
===== m19_open.em
exit=0
```

**6. Still refused:** OLD's `fn open(self): self.angle = 90.0`: "`open` changes `self.angle`; write its
first line as `fn open(self) changes angle:`" (the vocabulary's `self` row).

**7. Cost:** none.

**8. Kind:** a readable spelling of Rust's guarantee (`&mut self`), narrowed to the field.

#### AN-20: a plain parameter, a `for` name or a `Cell` field changes a class object with no word

**1. Intent:** `bump` adds 1 to the caller's counter; `heal_all` sets every player's health to 100;
`tick` advances a clock's frame number.

**2. OLD** (`o20a_param_changes_object.em`, `o20b_for_class.em`, `o20c_cell_plain.em`):

```ember
class Counter:
    n: int = 0

fn bump(c: Counter):
    c.n += 1
```

```ember
class Player:
    health: float = 0.0

fn heal_all(players: Array[Player]):
    for p in players:
        p.health = 100.0
```

```ember
struct S:
    frame: Cell[int]

fn tick(s: S):
    s.frame.set(s.frame.get() + 1)
```

```text
===== o20a_param_changes_object.em
exit=0
===== o20b_for_class.em
exit=0
===== o20c_cell_plain.em
exit=0
```

All three accepted; `[FN-1]` says the default parameter "cannot assign through it".

**3. NEW:**

```ember
class Counter:
    n: int = 0

class Player:
    health: f64 = 0.0

struct Clock:
    frame: Cell[int]

fn bump(ref mut c: Counter):
    c.n += 1

fn heal_all(ref mut players: Array[Player]):
    for ref mut p in players:
        p.health = 100.0

fn tick(ref mut s: Clock):
    s.frame.set(s.frame.get() + 1)

fn show(s: Clock):
    println(s.frame.get())
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `n`, `health` | none (default: the object's own number) | none (default: through a `ref mut` name, `changes` or `mut self`) |
| `frame` | `Cell`: a value replaced whole by `.set` | none (default as any field; section 4: `.set` needs a name that may change) |
| `c` | `ref mut`: the caller's counter | `ref mut`: yes |
| `players` | `ref mut`: the caller's list | `ref mut`: yes |
| `p` | `ref mut`: the list's own element | `ref mut`: yes |
| `s` in `tick` | `ref mut`: the caller's clock | `ref mut`: yes |
| `s` in `show` | none (default: a read-only view of the caller's clock) | none (default: never, `Cell` included) |

**5. Means exactly** (`m20_ref_mut_names.em`): today's spelling of the same run-time behaviour, with
no word on the names:

```ember
fn bump(c: Counter):
    c.n += 1

fn heal_all(players: Array[Player]):
    for p in players:
        p.health = 100.0

fn tick(s: Clock):
    s.frame.set(s.frame.get() + 1)

fn show(s: Clock):
    println(s.frame.get())
```

```text
===== m20_ref_mut_names.em
exit=0
```

**6. Still refused:**
- `fn bump(c: Counter): c.n += 1`: "`bump` changes `c.n`, but `c` only reads the caller's counter; to
  change it write `fn bump(ref mut c: Counter):`".
- `for p in players: p.health = 100.0`: "`p` only reads each player; to change them write
  `for ref mut p in players:`" (and `players` must be `ref mut players` or the caller's own).
- `fn tick(s: Clock): s.frame.set(...)`: "`tick` sets `s.frame`, but `s` only reads the caller's
  clock; write `fn tick(ref mut s: Clock):`".
- `fn bump(mut c: Counter)`: "to change the caller's `c` through `c`, write `ref mut c: Counter`" (C-1).

**7. Cost:** none; the same instructions, only the spelling changes.

**8. Kind:** a readable spelling of Rust's guarantee (`&mut`), with the word on the name.

#### AN-21: caching inside a read-only method (A31)

**1. Intent:** `radius()` computes the mesh radius once, keeps it, and computes it again only after
`verts` changes.

**2. OLD** (`o21_lazy_cache.em`, phase 1's `a31_lazy_cache.em`):

```ember
struct Mesh:
    verts: Array[f32]
    radius_cache: Option[f32]

    fn radius(self) -> f32:
        if self.radius_cache.is_none():
            r: f32 = 0.0
            for v in self.verts:
                r = max(r, abs(v))
            self.radius_cache = Some(r)
        return self.radius_cache.unwrap()
```

```text
error[E3023]: cannot mutate borrowed parameter `self`
  --> o21_lazy_cache.em:12:13
   |
 7 |     fn radius(self) -> f32:
   |               ---- `self` is borrowed here
...
12 |             self.radius_cache = Some(r)
   |             ^^^^^^^^^^^^^^^^^ this write needs mutable access
   |
   = help: restructure to a single owner and declare this parameter `mut self: Mesh`
   = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
   = help: if the value has identity, make it a `class`
   = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

error: could not compile due to 1 error(s)
exit=1
```

(The help's `mut self: Mesh` is not Ember's spelling either.) And a hand-kept cache goes stale when
someone changes `verts` and forgets to clear it.

**3. NEW** (planned word `@cached`, row 7's `reads`):

```ember
struct Mesh:
    verts: Array[f64]

    @cached
    fn radius(self) reads verts -> f64:
        r = 0.0
        for v in self.verts:
            r = max(r, abs(v))
        return r

fn main():
    m = Mesh(verts=[1.0, -2.0])
    println(m.radius())      # computes 2.0 and keeps it
    println(m.radius())      # the kept 2.0, no loop
    m.verts.push(-5.0)       # a change to `verts` clears the kept result
    println(m.radius())      # computes 5.0
```

Area B's rulings on `@cached` (planned-words section 8): the result is a `Copy` value returned by
value, or, for any other type, a `ref T` view into the kept result (`-> ref Array[f64]`), which is a
view of the object and so keeps `verts` from changing while it is used (the default source rule,
section 1 item 4). A change to a `reads` field through any name clears the result: an assignment, a
`changes verts` or `mut self` call, or taking a `ref mut` view of `verts`.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `verts` | none (default: the mesh's own list) | none (default: field; any change clears the kept result) |
| `self` | `self`; `@cached` says the result is kept; `reads verts` says what it depends on | `self` without `changes`: no field changes |
| `r` | none (default: own local) | none (default: own local) |
| `v` | none (default: read-only view of each element) | none (default: never) |
| `m` | none (default: own local) | none (default: own local; `push` changes it, which clears the kept result) |

**5. Means exactly** (`m21_cache_by_hand.em`): the slot the compiler adds, written by hand, and the
clear the compiler inserts after every change to `verts`:

```ember
struct Mesh:
    verts: Array[f64]
    radius_slot: Cell[Option[f64]] = Cell(None)

    fn radius(self) -> f64:
        kept = self.radius_slot.get()
        if kept.is_some():
            return kept.unwrap()
        r = 0.0
        for v in self.verts:
            r = max(r, abs(v))
        self.radius_slot.set(Some(r))
        return r

fn main():
    m = Mesh(verts=[1.0, -2.0])
    println(m.radius())
    println(m.radius())
    m.verts.push(-5.0)
    m.radius_slot.set(None)
    println(m.radius())
```

```text
===== m21_cache_by_hand.em
exit=0
```

**6. Still refused** (planned-words 3.4, kept): `@cached` without `reads` ("say what `radius`
depends on, so a change can clear it: `@cached fn radius(self) reads verts -> f64:`"); with other
parameters ("`@cached` keeps one result, and `radius` takes `scale`; keep a `Map` field of results by
hand"); reading a `shared`, `thread_shared`, `weak` or `Cell` field ("`radius` reads `mesh`, which its
other owners may change without `Mesh` seeing it, so a kept result could be stale; remove `@cached`,
or keep the result by hand"); with `changes` or `mut self` ("a cached method only reads; remove
`changes verts`"). **14:** (clash 6) on an object several threads reach, area E's ruling replaces the
refusal this answer proposed: the slot is filled once under a once-flag (`Once`: an acquire load and a
branch once filled); it is refused only where a `reads` field changes while other threads hold the
object: "`radius` reads `verts`, which other threads change under `lock`; a kept result could be stale;
remove `@cached`, or keep the result in a field `guarded by lock`".

**7. Cost:** one flag test per call; one store (clear the flag) at each place that may change `verts`;
the slot and flag in the object (8 bytes for the `f64` and one flag). The same as the cache a C
programmer writes by hand (the translation above is that program).

**8. Kind:** a new mechanism.

#### AN-22: returning references (A5): a result borrows all of `self` unless `@borrows` is written above; the ECS pool reference is E3060

**1. Intent:** `next_word` returns the next word of the source text, and the caller keeps two words
at once; `get_mut[T]` returns a component inside its pool, to change it in place.

**2. OLD** (`o22c_two_words_flat.em`; phase 1's nested form `o22a_two_words.em` gives the same E3025):

```ember
struct Parser:
    src: str
    pos: int = 0

    fn next_word(mut self) -> str:
        rest = self.src[self.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        self.pos += end + 1
        return rest[..end]

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    a = p.next_word()
    b = p.next_word()
    println(a, b)
```

```text
error[E3025]: cannot call a method on `p` while `p.src` is mutably borrowed
  --> o22c_two_words_flat.em:16:9
   |
15 |     a = p.next_word()
   |         ------------- mutable borrow of `p.src` starts here
16 |     b = p.next_word()
   |         ^^^^^^^^^^^^^ conflicting access here
17 |     println(a, b)
   |             - borrow later used here
   |
   = help: inline the field access, take the two fields as separate parameters, or split the method
   = note: a call borrows the fields of `self` its method uses (BRW-10); this method uses all of `self`, so the other fields do not stay disjoint across the call (BRW-4)

error: could not compile due to 1 error(s)
exit=1
```

The help does not name the fix that works (`@borrows(self.src)`, written on its own line above the
function). The ECS form (`o22b_ecs_pool.em`, `target/analysis/ecs5/pos.em`):

```ember
struct Reg:
    pools: Map[u128, AnyPool]

    fn get_mut[T](mut self, i: int) -> ref mut T:
        pool = cast self.pools[7u128] as Pool[T]
        return ref mut pool.dense[i]
```

```text
error[E3060]: `pool.dense[…]` does not live long enough
  --> o22b_ecs_pool.em:21:35
   |
20 |         pool = cast self.pools[7u128] as Pool[T]
   |         ---------------------------------------- `pool` is a local, so its storage ends with the frame
21 |         return ref mut pool.dense[i]
   |                                   ^ the borrow is still live when the function returns
   |                                   - `pool.dense[…]` is borrowed here
   |
   = help: return an owned value, take the destination as a `mut` parameter, or borrow something the caller owns
   = note: a returned reference must derive from a parameter (LT-1)

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW:**

```ember
struct Parser:
    src: StrView
    pos: int = 0

    fn next_word(self) changes pos -> StrView from src:
        rest = self.src[self.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        self.pos += end + 1
        return rest[..end]

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    a = p.next_word()
    b = p.next_word()        # a points into `text`, not into `p`
    println(a, b)
```

```ember
struct Registry:
    pools: Map[u128, owned dyn AnyPool]

    fn get_mut[T](self, e: int) changes pools -> ref mut T:
        pool = ref mut cast self.pools[7u128] as Pool[T]
        return ref mut pool.dense[pool.sparse[e]]
```

The rules: `from <name>` after the result type names the source (a parameter, a field of `self`
written bare as in `changes`/`reads`, or `self`); a name that is both a parameter and a field is
refused ("`src` is both a parameter and a field; rename the parameter"). With no `from`, the stated
default applies (section 1 item 4). `from src` on a view field means "into what `src` views", so the
call's hold on `p` ends with the call. In `get_mut`, `pool` is spelled `ref mut`: a name for the slot
inside `self.pools`, not a local of its own, so the result points into `self` (the default) and E3060
cannot arise; a downcast of a place is a place (last run's 3.4, saved by its attack: one type-id
compare, never the method table's address).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `src` | `StrView`: a view of someone else's text | none (default: field) |
| `pos` | none (default: the parser's own number) | none (default: field; `changes pos`) |
| `self` in `next_word` | `self` | `changes pos` |
| `rest`, `a`, `b` | none (default: a read-only view, as row 5's `word`; `from src` on `next_word`'s line says where `a`, `b` point) | none (default: never) |
| `end` | none (default: own local) | none (default: own local) |
| `text`, `p` | none (default: own local) | none (default: own local) |
| `pools` | `owned dyn` (**14:** `dyn` added, clash 4: `AnyPool` is an interface): one owner of each pool | none (default: field; `changes pools`) |
| `self` in `get_mut` | `self` | `changes pools` |
| `e` | none (default: read-only parameter) | none (default: never) |
| `pool` | `ref mut`: a name for the slot in `self.pools` | `ref mut`: yes |

**5. Means exactly:** `m22a_two_words_borrows.em` (today's `@borrows(self.src)`), and for one
component type `m22b_pool_place.em` (today's `Box` for `owned`, `ref mut` on the place):

```ember
struct Parser:
    src: str
    pos: int = 0

    @borrows(self.src)
    fn next_word(mut self) -> str:
        rest = self.src[self.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        self.pos += end + 1
        return rest[..end]

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    a = p.next_word()
    b = p.next_word()
    println(a, b)
```

```ember
struct Pool:
    dense: Array[int]
    sparse: Array[int]

struct Registry:
    pools: Array[Box[Pool]]

    fn get_mut(mut self, kind: int, e: int) -> ref mut int:
        pool = ref mut self.pools[kind]
        return ref mut pool.dense[pool.sparse[e]]

fn main():
    reg = Registry(pools=[Box(Pool(dense=[10, 20], sparse=[1, 0]))])
    p = reg.get_mut(0, 0)
    p += 1
    println(reg.pools[0].dense[1])
```

```text
===== m22a_two_words_borrows.em
exit=0
===== m22b_pool_place.em
exit=0
```

**21:** (F-05) `p = reg.get_mut(0, 0)` then `p += 1` in `m22b_pool_place.em` is today's spelling; in
NEW it is `p = ref mut reg.get_mut(0, 0)` (R2: a call whose result is `ref mut T` is a place, R6).

The generic downcast cannot be said in today's Ember (`cast` needs class handles); its C is last
run's faithful model `target/redesign2/references/attack2/bench/get2.c -DCAST` (a type-id load and
compare, then the element's address).

**6. Still refused:**
- `return rest[..end]` under `from src` when `rest` came from somewhere else (E3062): "`next_word`
  returns a view of `line`, but its first line says `from src`; write `from src, line`, or return
  `String(...)`".
- At the call, when the default is too wide (today's E3025): "`a` points into `p` because
  `next_word`'s result points into the parser by default; if it points only into the text, write
  `fn next_word(self) changes pos -> StrView from src:`" (the compiler computes the set from the body
  to write this line; it never applies it silently).
- `pool = cast self.pools[k] as Pool[T]` (no `ref mut`) followed by `return ref mut pool.dense[...]`:
  "`pool` reads the pool in `self.pools` and cannot change it; to return a changeable element, write
  `pool = ref mut cast self.pools[k] as Pool[T]`". (**15:** was "this would take the pool out of
  `self.pools`": by R2 and R13 a plain name bound from a `cast` of a place is a read-only view of it,
  never a hand-over.)
- Changing `self.pools` while the returned component is used (today's E3025, `box_bad.em`): "`p`
  points into `reg.pools`, which `clear` would empty; finish with `p` first".
- `@borrows(...)` on its own line: "write `from a` after the result type".

**7. Cost:** none for `from` (compile time only). The downcast: one type-id compare per get. Last
run's faithful model measured it at 1.223 to 1.244 of plain C, against safe Rust's `downcast_mut` at
1.571 to 1.635 (`tasks/96/redesign/phase3b-references-attack.md`, section 3.2 table, lines 282-287;
provisional, measured while other agents ran). Bounds checks are included in those rows.

**8. Kind:** a readable spelling of Rust's guarantee (a lifetime written as `from src` on the
signature).

#### AN-23: a call that returns `ref mut` is not a place

**1. Intent:** add 1 to the element a function hands back, and to a map's count for a key.

**2. OLD** (`o23_call_result_place.em`):

```ember
from std.collections import Map

fn first(mut xs: Array[int]) -> ref mut int:
    return ref mut xs[0]

fn main():
    xs = [1, 2, 3]
    first(xs) += 1
    counts = Map[String, int]()
    counts.entry("a").or_default() += 1
    println(xs[0])
```

```text
error[E2140]: assignment target is not a place
 --> o23_call_result_place.em:9:5
  |
9 |     first(xs) += 1
  |     ^^^^^^^^^ this is a value, not somewhere to store one
  |

error[E2140]: assignment target is not a place
  --> o23_call_result_place.em:11:5
   |
11 |     counts.entry("a").or_default() += 1
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this is a value, not somewhere to store one
   |

error: could not compile due to 2 error(s)
exit=1
```

The result type says `ref mut int`, a place; the message says it is a value, and gives no line.

**3. NEW:**

```ember
from std.collections import Map

fn first(ref mut xs: Array[int]) -> ref mut int:
    return ref mut xs[0]

fn main():
    xs = [1, 2, 3]
    first(xs) += 1
    counts = Map[String, int]()
    counts.entry("a").or_default() += 1
    println(xs[0], counts["a"])
```

Rule: a call whose result type is `ref mut T` is a place for the statement it is in; `=`, `+=` and a
`mut self` method may be applied to it.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `xs` (parameter) | `ref mut`: the caller's list | `ref mut`: yes |
| `xs` (local) | none (default: own local) | none (default: own local; the call changes it through `first`'s `ref mut xs`) |
| `counts` | none (default: own local) | none (default: own local) |

**5. Means exactly** (`m23_bound_first.em`):

```ember
from std.collections import Map

fn first(mut xs: Array[int]) -> ref mut int:
    return ref mut xs[0]

fn main():
    xs = [1, 2, 3]
    f = first(xs)
    f += 1
    counts = Map[String, int]()
    c = counts.entry("a").or_default()
    c += 1
    println(xs[0], counts["a"])
```

```text
===== m23_bound_first.em
exit=0
```

**21:** (F-05) `f = first(xs)`, `f += 1` and `c = counts.entry("a").or_default()`, `c += 1` are today's
spelling of NEW `f = ref mut first(xs)` and `c = ref mut counts.entry("a").or_default()` (R2); the
NEW block writes the call as the place itself.

**6. Still refused:** a call whose result is a new value, `make_point().y = 1` or
`copy_of_first(xs) += 1`: "`copy_of_first(xs)` gives a new `int`, so adding to it would be lost;
keep it in a name first: `n = copy_of_first(xs)`, then `n += 1`".

**7. Cost:** none: the same address and store as the bound form.

**8. Kind:** a readable spelling of Rust's guarantee (Rust writes `*first(&mut xs) += 1`).

#### AN-24: structs holding references (A6): a view struct cannot be stored in an `Array`

**1. Intent:** keep a list of views that each read one entity of one registry, and sum their hit
points. (**15:** "handles" became "views": the word "handle" is kept for `Handle[T]`, AN-50.)

**2. OLD** (`o24b_views_in_local_array.em`; phase 1's `e06d` with `ref mut` is `o24a`, same E3063):

```ember
struct Registry:
    hp: Array[int]

struct View:
    reg: ref Registry
    entity: int

    fn hp(self) -> int:
        return self.reg.hp[self.entity]

fn main():
    reg = Registry(hp=[10, 20])
    vs: Array[View] = []
    vs.push(View(reg=ref reg, entity=0))
    vs.push(View(reg=ref reg, entity=1))
    total = 0
    for v in vs:
        total += v.hp()
    println(total)
```

```text
error[E3063]: `View` is a view, so it may not be stored in an `Array` unless it is `static`
  --> o24b_views_in_local_array.em:15:13
   |
15 |     vs.push(View(reg=ref reg, entity=0))
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^ stored here
   |
   = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
   = note: the view borrows `reg`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error[E3063]: `View` is a view, so it may not be stored in an `Array` unless it is `static`
  --> o24b_views_in_local_array.em:16:13
   |
16 |     vs.push(View(reg=ref reg, entity=1))
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^ stored here
   |
   = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
   = note: the view borrows `reg`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error: could not compile due to 2 error(s)
exit=1
```

Rust compiles the twin (`Vec<View<'a>>`, last run's `r06a`).

**3. NEW:** the OLD program, unchanged, is accepted. Rule: a list (`Array`, `Map`, `Set`, `Option`)
whose element type holds a view is itself a view: it may live as long as every source of its
elements, no longer (one limit per view field of the element type, covering every element pushed, as
Rust's single `'a`; erasing to `owned dyn I` keeps the limit). Nothing is written for a local list;
a field holding such a list says its source with `from <let field>` (AN-25).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `hp`, `entity` | none (default: the struct's own values) | none (default: field) |
| `reg` (field) | `ref`: a read-only view of a registry | `ref`: no |
| `self` in `hp` | `self` | `self`: no |
| `reg` (local) | none (default: own local) | none (default: own local) |
| `vs` | none (default: own local list; its element type `View` holds a `ref`, so the list is a view of `reg`) | none (default: own local) |
| `total` | none (default: own local) | none (default: own local) |
| `v` | none (default: read-only view of each element) | none (default: never) |

**5. Means exactly:** today's Ember cannot store a view in an `Array`; in C (`c24_views_list.c`):

```c
typedef struct { int64_t *hp; size_t len; } Registry;
typedef struct { const Registry *reg; int64_t entity; } View;
typedef struct { View *ptr; size_t len, cap; } ViewList;
...
    Registry reg = { hp, 2 };                            /* main's own registry */
    ViewList vs = {0};                                   /* may not outlive reg: checked when compiling */
    push(&vs, (View){ &reg, 0 });
    push(&vs, (View){ &reg, 1 });
    for (size_t i = 0; i < vs.len; i++) total += view_hp(vs.ptr[i]);
    free(vs.ptr);                                        /* the list ends before reg */
```

```text
$ ./c24.exe
30
c24_views_list ms=0.018 peak_kb=4064 checksum=30
```

Under AddressSanitizer (`c24_asan.exe`): `30`, `c24_views_list ms=0.040 peak_kb=8148 checksum=30`, no
report.

**6. Still refused:**
- **14:** (clash 3) no longer refused: two changeable views of one registry in one list (`o24a`,
  `Handle(reg=ref mut reg, ...)` twice) are accepted, as area C's kept `ref mut` views of one player are
  (AN-52 (b), measured at 1.00x to 1.06x of plain C); R8 (b) checks them when compiling. What stays
  refused is a kept view into a part that another view reshapes, e.g. `h = ref mut reg.hp[0]` used after
  `hs[1].reg.hp.push(5)`: "`hs[1].reg.hp.push(5)` may move the list `h` points into (line N), and `h` is
  used on line M; use `h` first, or keep the entity number and read `reg.hp[0]` again".
- The list kept longer than `reg` (returned, stored in a field with no `from`, sent to a thread):
  "`vs` holds views of `reg`, which ends when `main` returns; keep the entity numbers instead, or keep
  `reg` alive as long as `vs`".

**7. Cost:** none beyond plain C: the C above is the layout; the limit exists only when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (`Vec<View<'a>>` with nothing written).

#### AN-25: self-referential data (A7): a value owning its text and the word views into it

**1. Intent:** a `Document` holds its text and the list of its words, without copying each word.

**2. OLD** (`o25a_document.em`, phase 1's `e07`; `o25b_document_one_view.em` shows the rule under it):

```ember
struct Document:
    text: String
    words: Array[str]

fn load(owned text: String) -> Document:
    words: Array[str] = []
    for w in text.split_whitespace():
        words.push(w)
    return Document(text=text, words=words)
```

```text
error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
 --> o25a_document.em:9:20
  |
9 |         words.push(w)
  |                    ^ stored here
  |
  = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
  = note: the view borrows `text`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error: could not compile due to 1 error(s)
exit=1
```

```text
error[E3024]: `text` would be moved into this value while another of its fields borrows it
 --> o25b_document_one_view.em:9:12
  |
9 |     return Document(text=text, first=w)
  |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `text` and a view of it would both be fields of this value
  |
  = help: store an index or a `Handle` into the owner instead of a view of it, or keep the owner outside the value
  = note: a field cannot borrow another field of the same value: moving the value would move what the view points into (TYP-15)
```

(`o25b` also prints an E3060 for the same line; the full output is in `check_output.txt`.)

**3. NEW:**

```ember
class Document:
    let text: String
    words: Array[StrView] from text

    fn init(self, owned text: String):
        self.text = text
        self.words = self.text.split_whitespace().collect()

fn main():
    d = Document(String("the quick fox"))
    println(d.words.len())
```

The rules (area B's ruling on planned-words section 8, the heap-block condition):
1. `from text` names a `let` field of the same value whose contents live in their own heap block: a
   `String` or an `Array`. Today's runtime gives both the layout `{ptr, len, cap}` with the contents
   outside the header (`runtime/ember_rt/include/ember_rt.h:2022-2026`, `ember_vec`; `String` "is
   the same buffer"), and ADR-141's frame buffer is only for a list that never leaves its function,
   which a field's contents always do. So moving the value moves the header, never the characters.
   If `String` ever gains an inline small-string form, `from` on a `String` field must be refused.
2. `let` means `text` never changes after `init` (06's C-8), so the words stay valid for the value's
   whole life. Last run had to make the views "unusable" after every write to `text` and was attacked
   through an index write (`docs[i].text.push_str(...)` then `docs[j].words`, attack report section 5);
   with `let` there is no write to track, and that attack has nothing to hit.
3. `words` takes only views into `text` (or static text).
4. A view read out (`w = d.words[0]`) is a view of `d`: `d` may not be moved or ended while `w` lives
   (last run's attack fix 1).
5. Works for a `struct` too (`let` on a struct field is accepted today: `p_let_struct_field.em`,
   `exit=0`); a struct literal fills `text` before computing `words`.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `text` (field) | `String`: the document's own text | `let`: never after `init` |
| `words` | `StrView` elements, `from text`: views into `text` | none (default: field; changed through `changes words` or `mut self`) |
| `self` in `init` | `self` in `init`: the object being built | `init` sets its fields (vocabulary: "`init` may always set fields") |
| `text` (parameter) | `owned`: handed over by the caller | `owned`: it is now `init`'s, moved into the field |
| `d` | none (default: own local) | none (default: own local) |

**5. Means exactly** (C, `c25_document.c`):

```c
typedef struct { char *ptr; size_t len, cap; } String;
typedef struct { const char *ptr; size_t len; } StrView;
typedef struct { StrView *ptr; size_t len, cap; } Words;
typedef struct { String text; Words words; } Document;     /* a class object: on the heap */

static Document *document_new(String owned_text) {        /* init(self, owned text: String) */
    Document *d = calloc(1, sizeof *d);
    d->text = owned_text;                                   /* handed over: the block does not move */
    ... split on spaces, push (ptr, len) views into d->text.ptr ...
}
static void document_drop(Document *d) { free(d->words.ptr); free(d->text.ptr); free(d); }
```

```text
$ ./c25.exe
3 words, first=the
c25_document ms=0.016 peak_kb=4056 checksum=204454
```

Under AddressSanitizer (`c25_asan.exe`): `3 words, first=the`, `c25_document ms=0.049 peak_kb=8172
checksum=204454`, no report.

**6. Still refused:**
- `from` naming a field that is not `let`: "`words` points into `text`, so `text` must never change
  after `init`; write `let text: String`".
- A change to `text` after `init`: AN-33's message.
- `self.words.push(other_word)` with a view of something else: "`words` holds views of `text` only,
  and `other_word` points into `line`; keep your own copy: change `words` to `Array[String]` and push
  `String(other_word)`".
- `copied d`: "`d.words` points into `d.text`, so a copy's words would point into the old text; write
  a `clone` for `Document` that splits the copy's text again" (re-pointing each view, last run's rule
  7, is listed as a good part, section 5).
- `from` on a fixed array or inline struct field: "`buf` is stored inside the value, so moving the
  value moves it; keep the text in a `String`".

**7. Cost:** none beyond plain C: the C above is the layout (pointer and length per word, no count, no
check). Last run measured this layout against Rust's index-range workaround (1.205 of plain C) and
today's one-`String`-per-word (1.451 time, 2.122 memory) (`phase3b-references.md`, section 5.4 table;
provisional; the attack report section 5 notes that the Ember program also pays bounds checks, about
1.046 of plain C, which every Ember program pays).

**8. Kind:** a smarter compile-time solution (Rust refuses self-reference; Ember accepts it because
the source is `let` and its contents never move).

#### AN-26: `str` is a borrowed view while Python's `str` is the owned string

**1. Intent:** keep a player's name in the player.

**2. OLD** (`o26a_str_field_literal.em`, `o26b_str_field_string.em`, `o26c_str_return.em`):

```ember
class Player:
    name: str

    fn init(self, name: str):
        self.name = name

fn make(s: String) -> Player:
    return Player(s)
```

```text
===== o26a_str_field_literal.em
exit=0
===== o26b_str_field_string.em
error[E3063]: `str` is a view, so it may not be passed to `init`, which stores it where only a `static` view may go
 --> o26b_str_field_string.em:8:19
  |
8 |     return Player(s)
  |                   ^ stored by this call
  |
  = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
  = note: the view borrows `s`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error: could not compile due to 1 error(s)
exit=1
```

The same class compiles when fed a literal (`o26a`) and is refused when fed a `String` (`o26b`): the
declaration `name: str` does not say which. And `fn first(xs: Array[String]) -> str:` (`o26c`) does
not say the result points into `xs`; the conflict shows only at a later `push` (E3021, full output in
`check_output.txt`).

**3. NEW:**

```ember
class Player:
    name: String
    kind: StrView from static

    fn init(self, name: StrView):
        self.name = String(name)
        self.kind = "human"
```

`str` is spelled `StrView` (06, 07). A `StrView` field of a class says what it points into:
`from <let field>` (AN-25) or `from static` (text written in the program). A function returning
`StrView` has the default source (section 1 item 4), or `from`.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `name` (field) | `String`: the player's own text | none (default: field) |
| `kind` | `StrView from static`: a view of text written in the program | none (default: field) |
| `self` | `self` in `init` | `init` sets fields |
| `name` (parameter) | `StrView`: a view of the caller's text | none (default: read-only parameter) |

**5. Means exactly** (`m26_name_string.em`):

```ember
class Player:
    name: String
    kind: str

    fn init(self, name: str):
        self.name = name.to_string()
        self.kind = "human"

fn make(s: String) -> Player:
    return Player(s)

fn main():
    p = make("ann".to_string())
    println(p.name, p.kind)
```

```text
===== m26_name_string.em
exit=0
```

**6. Still refused:**
- `str` anywhere: "`str` is now `StrView`, a view into someone else's text; for the player's own text
  write `String`".
- `name: StrView` in a class with no `from`: "a `Player` keeps `name` after the text it came from may
  be gone; to keep its own copy write `name: String` and `self.name = String(name)`".
- A non-static view into a `from static` field (`self.kind = name`): "`kind` holds only text written
  in the program, and `name` comes from the caller; write `self.kind = String(name)` with
  `kind: String`".
- `o26c`'s `push` while `n` lives: "`n` points into `names` (`first` returns a view of `xs`), and
  `push` may move what it points into; use `n` before the `push`, or keep `n = String(first(names))`".

**7. Cost:** none for the spelling. `String(name)` is one allocation and copy, the same as Python's
own `str` object and today's `to_string()`.

**8. Kind:** a readable spelling of Rust's guarantee (`&str` vs `String`, `&'static str`).

#### AN-27: `ref T` / `ref mut T` as a parameter's type

**1. Intent:** `bump(n)` adds 1 to the caller's number.

**2. OLD** (`o27a_ref_mut_type.em`, `o27b_help_form.em`, `o27c_ref_type.em`, reader 1's t3, t3c, t6):

```ember
fn bump_mode(mut x: int):
    x += 1

fn bump_ref(x: ref mut int):
    x += 1
```

```text
error[E3023]: cannot mutate borrowed parameter `x`
 --> o27a_ref_mut_type.em:6:5
  |
5 | fn bump_ref(x: ref mut int):
  |             -------------- `x` is borrowed here
6 |     x += 1
  |     ^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut x: ref mut i64`
  = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
  = help: if the value has identity, make it a `class`
  = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

error[E2020]: expected `ref mut i64`, found `i64`
  --> o27a_ref_mut_type.em:11:14
   |
 5 | fn bump_ref(x: ref mut int):
   |             -------------- parameter `x` is declared `ref mut i64` here
...
11 |     bump_ref(n)
   |              ^ this is `i64`
   |
   = help: borrow it explicitly: `ref mut` before the place (TYP-5 rule 7 borrows only as `ref`)

error: could not compile due to 2 error(s)
exit=1
```

The help's own form does not compile (`o27b`, `fn bump_ref(mut x: ref mut int)`):

```text
error[E2020]: `+=` is not defined on `ref mut i64`
 --> o27b_help_form.em:3:5
  |
3 |     x += 1
  |     ^^^^^^
  |
  = note: a type has `+=` by implementing `AddAssign`, or `Add` with its own type as `Output` [TYP-21]

error[E3027]: a `mut` argument is not a mutable place
 --> o27b_help_form.em:7:14
  |
7 |     bump_ref(ref mut n)
  |              ^^^^^^^^^ this value has no mutable storage for the callee to borrow
  |
  = help: bind the value to a local first, or pass the owner place directly (`f(obj.field)` rather than `f(obj.get_field())`)
  = note: the callee's declared `mut` mode forms the mutable borrow; the call site does not write a mode [FN-2a]

error: could not compile due to 2 error(s)
exit=1
```

And `fn show_ref(x: ref int)` means the same as `fn show_mode(x: int)` (`o27c`, `exit=0`).

**3. NEW:**

```ember
fn bump(ref mut n: int):
    n += 1

fn main():
    a = 1
    bump(a)
    println(a)
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `n` | `ref mut`: the caller's number | `ref mut`: yes |
| `a` | none (default: own local) | none (default: own local) |

**5. Means exactly** (`m27_bump.em`):

```ember
fn bump(mut n: int):
    n += 1

fn main():
    a = 1
    bump(a)
    println(a)
```

```text
===== m27_bump.em
exit=0
```

**6. Still refused:** `fn bump(n: ref mut int)`: "the word goes on the name: write
`fn bump(ref mut n: int):`"; `fn show(x: ref int)`: "a parameter already reads the caller's value
without copying it; write `fn show(x: int):`"; `fn bump(mut n: int)`: "to change the caller's `n`
through `n`, write `ref mut n: int`" (vocabulary C-1).

**7. Cost:** none (passed by address, as today's `mut n`).

**8. Kind:** a readable spelling of Rust's guarantee (`&mut i64`), on the name.

#### AN-28: `r = ref mut b` on a `ref mut` local copies `b` into `r`'s first target

**1. Intent:** keep a changeable reference to `a`, later work on `b` too.

**2. OLD** (`o28_ref_mut_rebind.em`, 05's `b2`):

```ember
fn main():
    a = 1
    b = 2
    r = ref mut a
    r = ref mut b
    println(a, b)
```

```text
===== o28_ref_mut_rebind.em
exit=0
```

Accepted; by `[BRW-1]` (spec 2286-2287) the second line writes `b`'s value into `a`, so it would
print `2 2` (read from the spec, not run), where the line reads as "`r` now refers to `b`".

**3. NEW:**

```ember
fn main():
    a = 1
    b = 2
    r = ref mut a
    r = 20
    rb = ref mut b
    rb = 30
    println(a, b)
```

A reference name always refers to the place it was made from (row 8: `path` is the NPC's list for
the whole arm), so `r = 20` writes into `a`; a second place gets a second name.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `a`, `b` | none (default: own local) | none (default: own local) |
| `r` | `ref mut`: a name for `a` | `ref mut`: yes |
| `rb` | `ref mut`: a name for `b` | `ref mut`: yes |

**5. Means exactly** (`m28_two_refs.em`, the same code):

```text
===== m28_two_refs.em
exit=0
```

**6. Still refused:** `r = ref mut b` (and `r = ref b`) on a reference name: "`r` always refers to
`a`; this copies `b`'s value into `a`; to refer to `b`, use a new name: `rb = ref mut b`" (vocabulary
`ref mut` row).
**19:** (R18-03) the same rule now holds for a plain view name (R2): `n = p.name` then `n = q.name` (or
`n = String("x")`) is E3023: "`n` reads `p.name` for its whole life and cannot be given another value;
for `q`'s name use a new name: `m = q.name`; for a string of your own, `s = String("x")`". So `=` on a
name that already has a value always puts the value where the name refers: into an own name, into the
original through `r` (line 4 above), and never into a plain view. (Was: R2 let `n = q.name` re-point
`n` while `r = 20` here wrote into `a`.) **21:** (F-05, F-06) the same for a name holding a view a
call gave (`n = p.name.trim()` then `n = q.name.trim()`: E3023, "use a new name:
`m = q.name.trim()`"); and a call whose result is `ref mut T` is a place, so
`r = ref mut first_even(...)` then `r = 7` writes the element, as `r = 20` writes `a` here (R2).

**7. Cost:** none.

**8. Kind:** an understandable limitation kept (a reference name is never re-pointed), with the one
misleading form refused.

#### AN-29: `ref String` is not accepted where `str` is expected at a call

**1. Intent:** pass a list's string element to a function that reads text, from inside a lambda.

**2. OLD** (`o29b_ref_string_in_lambda.em`):

```ember
fn shout(s: str) -> int:
    return s.len()

fn apply(f: fn() -> int) -> int:
    return f()

fn main():
    names: Array[String] = ["ann", "bob"]
    total = 0
    for n in names:
        total += apply(fn() => shout(n))
    println(total)
```

```text
error[E2020]: expected `str`, found `ref String`
  --> o29b_ref_string_in_lambda.em:12:38
   |
 2 | fn shout(s: str) -> int:
   |          ------ parameter `s` is declared `str` here
...
12 |         total += apply(fn() => shout(n))
   |                                      ^ this is `ref String`
   |

error: could not compile due to 1 error(s)
exit=1
```

The same call outside a lambda is accepted (`o29_ref_string_to_str.em`, `total += shout(n)`,
`exit=0`): one rule applied in one place and not the other.

**3. NEW:** the OLD program, unchanged (with `str` spelled `StrView`), is accepted. Rule: a `String`
place or a `ref String` reads as a `StrView` of its text wherever a `StrView` is wanted, inside a
lambda as everywhere; the view points into that `String`.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `s` | `StrView`: a view of the caller's text | none (default: read-only parameter) |
| `f` | none (default: a read-only callable parameter) | none (default: never) |
| `names`, `total` | none (default: own local) | none (default: own local) |
| `n` | none (default: read-only view of each element) | none (default: never) |

**5. Means exactly** (`m29_str_in_lambda.em`, `shout(n.as_str())`):

```text
===== m29_str_in_lambda.em
exit=0
```

**6. Still refused:** a `ref mut String` where a `StrView` is kept past a change: "`v` points into
`n`'s text, which `push_str` may move; use `v` first" (the ordinary borrow rule).

**7. Cost:** none (pointer and length, as `as_str()`).

**8. Kind:** a smarter compile-time solution (one coercion rule, as Rust's `&String` to `&str`).

#### AN-30: a reference out of a lock, `RefCell` or `RwLock` guard compiles and is unsafe (A33). Agreed row 4.

**1. Intent:** `top(scores)` returns the best name from a locked table.

**2. OLD** (`o30a_lock.em`, `o30b_lock_uaf.em`, `o30c_refcell.em`, `o30d_rwlock.em`):

```ember
from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> ref String:
    with g = scores.lock():
        return ref g[0]

fn clear(scores: Mutex[Array[String]]):
    with g = scores.lock():
        g.clear()

fn main():
    m: Mutex[Array[String]] = Mutex(["ann".to_string()])
    r = top(m)
    clear(m)
    println(r)
```

```text
===== o30a_lock.em
exit=0
===== o30b_lock_uaf.em
exit=0
===== o30c_refcell.em
warning[L3011]: a `RefCell` guard for `*c` is live across this call
 --> o30c_refcell.em:9:9
  |
8 |     with g = c.borrow_mut():
  |              -------------- guard borrowed here
9 |         g.clear()
  |         ^^^^^^^^^ call happens here
  |
  = help: drop the guard before the call, or put the call in a block of its own
  = note: a guard live across a call that re-enters the same cell panics at run time [CELL-7]

exit=0
===== o30d_rwlock.em
exit=0
```

All accepted. What `o30b` does, in C (`c30_lock_ref_escape.c`, built with AddressSanitizer):

```text
==116816==ERROR: AddressSanitizer: heap-use-after-free on address 0x126a7eaa0130 at pc 0x7ff900341f78 bp 0x00ae832feca0 sp 0x00ae832fece8
READ of size 2 at 0x126a7eaa0130 thread T0
    ...
    #8 0x7ff632242bf1 in main C:\Users\ism19\Code\ember\target\redesign3\09-area-B\c30_lock_ref_escape.c:27
0x126a7eaa0130 is located 0 bytes inside of 3-byte region [0x126a7eaa0130,0x126a7eaa0133)
freed by thread T0 here:
    #1 0x7ff632242433 in free S:\compiler-rt\lib\asan\asan_malloc_win_thunk.cpp:52
    #2 0x7ff632242b85 in main C:\Users\ism19\Code\ember\target\redesign3\09-area-B\c30_lock_ref_escape.c:26
previously allocated by thread T0 here:
    #1 0x7ff632242493 in malloc S:\compiler-rt\lib\asan\asan_malloc_win_thunk.cpp:64
    #2 0x7ff632242a9d in main C:\Users\ism19\Code\ember\target\redesign3\09-area-B\c30_lock_ref_escape.c:23
```

(Line 26 is `clear`'s `free`, inlined; line 27 is `println(r)`.)

**3. NEW:**

```ember
from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return copied g[0]

fn top_len(scores: Mutex[Array[String]]) -> int:
    with g = scores.lock():
        return g[0].len()
```

Rule: a view made through a guard (`g[0]`, `ref g[0]`, a slice of `g`) points into the guard, as if
the guard's reading method said `from g`; the guard ends with its `with` block, so such a view cannot
leave it. The same for `RwLock`'s `read()`/`write()`. **14:** `RuntimeChecked` is removed (AN-49, clash 5),
so its `read()`/`write()` are gone with it.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `scores` | none (default: a read-only parameter; its type `Mutex` says the list inside changes only under the lock) | none (default: never directly; **19:** `Mutex` in its type: the list inside may change through the guard, R5's one exception, R18-01) |
| `g` | `lock()`: the guard, the list while locked | `lock()`: yes, while the block lasts (vocabulary `Mutex` row) |

**5. Means exactly** (`m30_lock_copy.em`):

```ember
from std.sync import Mutex, RwLock

fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return g[0].clone()

fn top_read(scores: RwLock[Array[String]]) -> String:
    with g = scores.read():
        return g[0].clone()

fn top_len(scores: Mutex[Array[String]]) -> int:
    with g = scores.lock():
        return g[0].len()
```

```text
===== m30_lock_copy.em
exit=0
```

**6. Still refused:**
- `return ref g[0]` inside `with g = scores.lock():`: "this points into the locked list, which can
  change once the lock ends; return a copy, or do the work inside the `with` block" (row 4, exactly).
- The same under `scores.read()` / `scores.write()`: "this points into the read-locked list, which can
  change once the lock ends; return a copy, or do the work inside the `with` block".
- **14:** (the refusal for `RuntimeChecked`'s `c.read()` is removed with the type, clash 5.)
- A view of `g` stored in an outer name (`best = ref g[0]` then used after the block): "`best` points
  into the locked list, which can change once the lock ends at line N; keep `best = copied g[0]`, or
  use it inside the `with` block".

**7. Cost:** `copied g[0]` is one allocation and copy of the name, as Rust's `.clone()` workaround and
Python's own behaviour of handing out an object; the `top_len` form costs nothing extra. The check
itself is when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (a `MutexGuard` borrow ends with the guard).

#### AN-31: initialisation (A18): a helper called in `init` before every field is set is E2100 with no help. Agreed row 7.

**1. Intent:** `init` sets the renderer, then loads assets with a helper that only reads the renderer.

**2. OLD** (`o31b_init_class.em`, phase 1's `e18b`; the struct form `o31a` gives the same E2100):

```ember
class Renderer:
    width: int

class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(self, width: int):
        self.renderer = Renderer(width=width)
        self.assets = self.load_assets()

    fn load_assets(self) -> Array[String]:
        return [f"atlas@{self.renderer.width}px"]
```

```text
error[E2100]: `self` is used before all class fields are initialized
  --> o31b_init_class.em:12:23
   |
12 |         self.assets = self.load_assets()
   |                       ^^^^^^^^^^^^^^^^^^
   |

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW:**

```ember
class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(self, width: int):
        self.renderer = Renderer(width=width)
        self.assets = self.load_assets()

    fn load_assets(self) reads renderer -> Array[String]:
        return [f"atlas@{self.renderer.width}px"]
```

Rule (row 7): a method with `reads` may be called in `init` once every field it names is set; its
body is checked to read only those fields (and to change none).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `renderer`, `assets`, `frame` | none (default: the game's own values) | none (default: field) |
| `self` in `init` | `self` in `init` | `init` sets fields |
| `width` | none (default: read-only parameter) | none (default: never) |
| `self` in `load_assets` | `self`, `reads renderer` | `self` without `changes`: no |

**5. Means exactly** (`m31_init_helper.em`): the helper takes the one field it reads:

```ember
class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(self, width: int):
        self.renderer = Renderer(width=width)
        self.assets = load_assets(self.renderer)

fn load_assets(renderer: Renderer) -> Array[String]:
    return [f"atlas@{renderer.width}px"]
```

```text
===== m31_init_helper.em
exit=0
```

**6. Still refused:**
- The call before `self.renderer` is set: "`load_assets` reads `renderer`, which `init` has not set
  yet; set `self.renderer` before this line".
- The body reading a field not named: "`load_assets` reads `self.frame`, which its first line does not
  name; write `fn load_assets(self) reads renderer, frame -> Array[String]:`".
- A helper with no `reads` called before every field is set (today's E2100, new help): "`load_assets`
  may read any field of `Game`, and `assets` is not set yet; say what it reads:
  `fn load_assets(self) reads renderer -> Array[String]:`".

**7. Cost:** none.

**8. Kind:** a smarter compile-time solution (Rust cannot call a method on a half-built `self`).

#### AN-32: `mut self` on `init` is accepted and means the same as `self`

**1. Intent:** write a constructor.

**2. OLD** (`o32_init_mut_self.em`, reader 2's t4):

```ember
class Door:
    angle: f32

    fn init(mut self):
        self.angle = 0.0

    fn open(self):
        self.angle = 90.0
```

```text
===== o32_init_mut_self.em
exit=0
```

**3. NEW:**

```ember
class Door:
    angle: f32

    fn init(self):
        self.angle = 0.0

    fn open(self) changes angle:
        self.angle = 90.0
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `angle` | none (default: the door's own number) | none (default: field) |
| `self` in `init` | `self` in `init` | `init` sets fields |
| `self` in `open` | `self` | `changes angle` |

**5. Means exactly** (`m32_init_self.em`, the same with today's `fn open(self)`):

```text
===== m32_init_self.em
exit=0
```

**6. Still refused:** `fn init(mut self)`: "`init` may always set fields; write `fn init(self):`"
(vocabulary `mut` row).

**7. Cost:** none.

**8. Kind:** a readable spelling (one form for a constructor); the accepted form that says nothing is
refused with that advice.

#### AN-33: a `let` field still changes through `push`; E2104 has no help

**1. Intent:** a log whose list grows; its name is set once in `init` and never changes.

**2. OLD** (`o33a_let_push.em`, `o33b_let_assign.em`, reader 2's t2, t2b):

```ember
class Log:
    let items: Array[int]

    fn init(self):
        self.items = []

    fn add(self, v: int):
        self.items.push(v)

    fn clear(self):
        self.items = []
```

```text
===== o33a_let_push.em
exit=0
===== o33b_let_assign.em
error[E2104]: cannot assign to `let` field `Log.items` outside its `init`
 --> o33b_let_assign.em:8:9
  |
8 |         self.items = []
  |         ^^^^^^^^^^
  |

error: could not compile due to 1 error(s)
exit=1
```

`let` stops the assignment and lets the `push` through, and the refusal has no line to write.

**3. NEW:**

```ember
class Log:
    items: Array[int]
    let name: String

    fn init(self, name: StrView):
        self.items = []
        self.name = String(name)

    fn add(self, v: int) changes items:
        self.items.push(v)
```

`let` means nothing is changed through the field after `init`, for every type (06's C-8). A list that
grows is a list that changes, so it is not `let`; which methods change it is said on their first line
(`changes items`).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `items` | none (default: the log's own list) | none (default: field; `changes items` on `add`) |
| `name` (field) | `String`: the log's own text | `let`: never after `init` |
| `self` in `init` | `self` in `init` | `init` sets fields |
| `name` (parameter) | `StrView`: a view of the caller's text | none (default: read-only parameter) |
| `self` in `add` | `self` | `changes items` |
| `v` | none (default: read-only parameter) | none (default: never) |

**5. Means exactly** (`m33_log_changes.em`):

```ember
class Log:
    items: Array[int]
    let name: String

    fn init(self, name: str):
        self.items = []
        self.name = name.to_string()

    fn add(self, v: int):
        self.items.push(v)
```

```text
===== m33_log_changes.em
exit=0
```

**6. Still refused:**
- A change through a `let` field after `init` (`self.name.push_str("!")`, or OLD's `self.items.push(v)`
  on a `let items`): "`items` is `let`: it is set in `init` and never changed after; to add to it,
  remove `let` from `items`' declaration".
- E2104, new help: "cannot assign to `let` field `Log.items` outside `init`; set it in `init`, or
  remove `let` from its declaration".
- `add` changing `items` without saying so: AN-19's message.

**7. Cost:** none; `let` lets AN-25's views into the field go unchecked.

**8. Kind:** an understandable limitation kept ("set once" and "grows" are different promises; only
the first is `let`), with the messages that say what to write.

## 12. Area C: ownership and sharing, `owned` replacing `Box`, copies, hand-over, state machines (AN-34 .. AN-52)

Copied from `RUN3/10-area-C.md` by `build.py`; the combiner's changes are marked **14:** and listed in section 4. File names in this area's text are relative to `target/redesign3/10-area-C/` unless they say otherwise; "section N" inside it means the area report's own section N.

### Terms of area C

- **owner**: the one place (a local, a field, a list element) that ends a value when it ends.
- **handed over**: the giver's name ends; the receiver is the owner (vocabulary).
- **copy** (`copied`): a new value equal to the old one; nothing done to one is seen through the other.
- **`shared` value**: a value with several owners on one thread; it lives until the last owner ends. Its
  owners are counted while the program runs (a **count**: a number in the value's header).
- **count pair**: one +1 and one -1 of a count, made around a use to keep the value alive.
- **view**: a name or value that points into something someone else owns and may not outlive it
  (`ref`, `ref mut`, `Span`, `StrView`); a **kept view** is a view stored in a struct or list.
- **reshape**: a change that can free or move memory: replacing a value that owns memory, `push`,
  `pop`, `insert`, `remove`, `clear`, `sort` on a list or map, changing an enum's variant, ending the
  last owner of an object. Writing a number or a `bool` is not a reshape.
- **first line** of a function: its signature, with row 3's `changes` and row 7's `reads` clauses.
- **the owner's test**: for every name, the word that says what it is (its owner's own thing, a copy,
  handed over, or a view of someone else's) and the word that says whether it may change it. "none
  (default: ...)" is used only where the brief allows it.
- **Python's locals** default, used below: a local variable a function makes is the function's own,
  and the function may change it (Python has no word for this either, and it never aliases: a second
  name for the value needs `shared` or a view word).

### Rules of area C (C1 .. C10)

- **C1, one owner.** Every value has one owner. `b = a` hands `a` over for every type that is not `Copy`
  (`Copy` as 06 narrowed it, C-3: numbers, `bool`, `ref T`, `Span`, structs of such; not class objects,
  not `shared` values, not `weak` links). Using a handed-over name is refused with the message pattern of
  row 9 (both ways out: `copied`, `shared`). **14:** replaced by R2 (clash 1): `n = p.name` binds a read-only view of the
  field and `p` stays whole; a part stored elsewhere (`push`, `return`, a field, an `owned` argument) is
  still handed over and leaves `p` partly handed over (`[EXP-6]`).
- **C2, `Copy` values.** `=` copies a `Copy` value. A number has no parts to change, so the copy cannot
  surprise (Python's `n2 = n; n2 += 1` is the same). A `Copy` **struct** copied by plain `=` from a place
  that lives on may not have a field changed through the new name: Python would change the original
  there (last run's attack 4.4, `copy_struct_name`), so the line must say which it means:
  `pos = copied p.pos` (your own copy) or `pos = ref mut p.pos` (change `p`'s).
  **14:** for a part (`pos = p.pos`), R2 applies: `pos` is a read-only view of `p.pos`, not a copy, and the
  same two lines are the way out; C2 stays for whole names (`b = a` of a `Copy` struct) (clash 1).
- **C3, words before a value.** `copied x`: a new independent value; runs the type's `Clone`.
  `shared x`: one more owner; `x` stays an owner. `weak x`: area D. `owned x`: refused (AN-47).
- **C4, words in a type.** Plain `T`: held alone (a struct's value inside its owner; a class object in
  its own block). `owned T`: held alone, in its own heap block (replaces `Box`): needed for a type that
  contains itself (AN-41), for `dyn I` (`owned dyn I`), and where the programmer wants one pointer in the
  owner; refused on a class type ("a `Player` object always lives in its own block; remove `owned`").
  `shared T`, `weak T` (D), `thread_shared T` (E). A plain value stored into an `owned T` place is put
  in its block at that store; a `shared T` place gets a counted block.
- **C5, parameters.** `x: T` reads the caller's value (default). `ref mut x: T` may change it (06 C-1,
  area B). `owned x: T` takes it over (refused where `T` is known to be `Copy`, 06 C-2). `x: shared T`
  makes the callee one more owner (the caller writes `shared y` or hands an owner over). `x: owned T`
  with no mode on the name stays refused (06). `owned m: owned Mesh` takes over a one-owner block and
  keeps the block (two facts, two places: the mode on the name, the holding in the type); `owned m: Mesh`
  given an `owned Mesh` argument takes the mesh out of its block.
  **15:** by the same "two facts, two places", `x: shared T` reads the caller's `shared` value (a view)
  and `owned x: shared T` makes the callee one more owner (the caller writes `shared y` or hands an
  owner over); a change through the parameter is `ref mut x: shared T` (R4, R5).
- **C6, changing a shared value.** Every owner of a `shared` value, and every `ref mut` name, may change
  it, written as any change is written. Nothing is checked while running. **The compile-time rule:** a
  view into a part of an object that owns memory ends at any change that may reshape that part of an
  object that may be the same object. "May reshape" is read from the function itself and from callees'
  first lines only: a method that says `changes inventory` or `mut self` may reshape its receiver's
  fields; a method that says `changes target` may reshape anything reachable through `target`; a call
  through `dyn`, a function value or unseen code has what its type declares, or everything. "May be the
  same object": two paths to a value of the same type, at least one through a `shared` or `weak` link or
  a kept view, unless the compiler knows they differ (different types, a fresh object, or the branch of
  `if a is not b:`). Ending the last owner of an object runs its `drop`, so a re-point carries the
  `changes` of the target type's `drop`; a re-point stores the new link before ending the old owner
  (last run's attack, 3.2).
  **14:** this is R8 (b), for every view that is not an element position: an element name of a list reached
  through locals, parameters and one-owner fields is a position (A-1, A-2), which survives an add (R8 (a),
  clash 2).
- **C7, keeping an object alive under a view.** A view of an object reached through a `shared` link
  keeps it alive with no count when no line in the view's life may end an owner of that type (no write
  to a place of type `shared T`, no end of a local `shared T` owner, no call whose first line reaches
  one); otherwise with one count pair. This is AN-51.
  **14:** merged with D5 and D6 into R10 (clash 10): where a loop makes unknown calls and the class has no
  `drop`, a deferred-drop region replaces the count pair; a view of a `thread_shared` object reached
  through `weak` holds an atomic count.
- **C8, `class`.** A class object lives in its own heap block, has identity (`is`), may be subclassed
  (`open`, `virtual`), and has an 8-byte header (a 32-bit count, a 32-bit type index; last run's lean
  class, measured in section 5). The declaration no longer means "shared": a second owner is `shared`,
  for a class as for a list. One owner is the default for every type.
- **C9, no run-time borrow checks.** `RefCell`, `Ref`, `RefMut`, `borrow`, `borrow_mut` and the
  placeholder `RuntimeChecked[T]` are removed. `Cell[T]` stays (a whole-value replace, no reference, no
  check; 06's C-10 condition is area B's).
- **C10, one thread.** `shared` never crosses threads (area E's `thread_shared`).

Agreed rows used: 3 (`changes`), 6 (`copied` in a capture), 8 and 9 (this area's), unchanged.

### Area C's measurements (its section 5, cited by AN-52)

The shared-object workload of last run (262,144 players, 1,048,576 watchers, 32 frames, one re-target
in sixteen steps), players taken from one block as a slab gives them. `bench/share_c.c`, four models;
Rust from last run's files, unchanged. Built:
`"/c/Program Files/LLVM/bin/clang.exe" -O2 -I target/redesign/bench -DMODEL=<n> share_c.c`;
`rustc -C opt-level=2 --edition 2021 share_cell.rs` (and `share_index.rs`). Timed with
`powershell -NoProfile -ExecutionPolicy Bypass -File target/redesign2/bench/run.ps1 -Out <file> -Rounds 5 -Exes ...`.
Every run of every program printed checksum `80363660820` (the summaries list the set of checksums).

| program | batch 1 median ms (ratio) | batch 2 | batch 3 | peak KB |
|---|---|---|---|---|
| plain C, raw pointers (`MODEL=0`) | 273.870 (1.000) | 279.983 (1.000) | 297.922 (1.000) | 53,284 |
| (a) `shared Player`, counted at re-target (`MODEL=3`) | 341.216 (1.246) | 318.629 (1.138) | 336.947 (1.131) | 55,332 |
| (b) kept `ref mut` views (`MODEL=8`) | 283.354 (1.035) | 279.921 (1.000) | 316.427 (1.062) | 53,284 |
| (c) positions in one list (`MODEL=9`) | 225.420 (0.823) | 236.955 (0.846) | 239.540 (0.804) | 49,188 |
| safe Rust, `Rc<Player>` with `Cell`/`RefCell` | 428.531 (1.565) | 417.785 (1.492) | 441.206 (1.481) | 61,920 |
| safe Rust, `Vec` and positions | 246.380 (0.900) | 240.839 (0.860) | 259.148 (0.870) | 47,488 |

Sources: `bench/runs_share_summary.txt`, `runs_share2_summary.txt`, `runs_share3_summary.txt`. The
spread between a program's fastest and slowest run was over 10% in every batch for at least one program
(up to 34.7%, Rust positions, batch 2): the machine was shared, so these ratios are for the Timer to
confirm. What they show in every batch: (b) is within 6% of plain C, (a) is 13% to 25% over it and
below Rust's `Rc`, (c) is faster than plain C's pointers and than Rust's positions.

### The answers of area C

#### AN-34: `snapshot = items` hands the list over; using `items` after is E3040, whose help says "clone"

1. **Intent:** keep a snapshot of a list before changing it (row 9).
2. **OLD** (`an34_old_snapshot.em`):

   ```ember
   fn main():
       items = [1, 2, 3]
       snapshot = items
       items.push(4)
       println(snapshot.len())
   ```

   ```text
   error[E3040]: `items` is borrowed after it has been moved out of
    --> an34_old_snapshot.em:4:5
     |
   3 |     snapshot = items
     |                ----- moved here
   4 |     items.push(4)
     |     ^^^^^^^^^^^^^ borrowed here after the move
     |
     = help: borrow before the move, or clone the value so the move takes the copy
     = note: the borrow would point at memory whose owner gave it away
     = note: assigning a list, a map or a set moves it; in Python both names would point at one list (Appendix E)

   error: could not compile due to 1 error(s)
   exit=1
   ```

3. **NEW** (row 9, exactly as agreed):

   ```ember
   fn main():
       items = [1, 2, 3]
       snapshot = copied items       # a copy: the snapshot stays [1, 2, 3]
       items.push(4)
       println(snapshot.len())       # 3

   # the other two ways the line can be written, each meaning what its word says,
   # each a program of its own (19: they are alternatives, not one sequence):
   fn hand_over():
       items = [1, 2, 3]
       log = items                   # hands the list over: `items` ends here
       log.push(4)

   fn two_owners():
       items = [1, 2, 3]
       other = shared items          # one list, two owners: a change through either is seen by both
       other.push(4)
       println(items.len())          # 4
   ```

   **19:** (R18-21) was `log = items` then `other = shared items` written one after the other below
   `main`, so the third line used `items` after the second had handed it over, which this answer's own
   E3040 refuses. Each form now stands in its own function, as AN-36 shows its two forms.

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `items` | none (default: the list made on this line belongs to the name it is given) | none (default: Python's locals: the function's own, which it may change) |
   | `snapshot` | `copied`: its own new list | none (default: Python's locals) |
   | `log` | none (default, row 9: `=` hands the list over; `items` ends) | none (default: Python's locals) |
   | `other` | `shared`: one of two owners of the same list | none (default: Python's locals; any owner may change it, C6) |
   | `items` in `hand_over`, `two_owners` | as in `main` until the line that names it; in `two_owners`, from `other = shared items` on, `shared` on that line: one of the two owners (**19:**) | none (Python's locals; any owner may change it, C6) |

5. **Means exactly** (today's Ember; all three accepted, `exit=0`, no output):
   `an34_tr_copied.em` (`snapshot = items.clone()`), `an34_tr_handover.em` (`snapshot = items`,
   `items` not used again), `an34_tr_shared.em` (`items = Shared([1, 2, 3])`, `other = items`,
   `items.get_mut().push(4)`, `other.get().len()`; today's `Shared` checks each `get_mut` while running,
   which the NEW `shared` does not, C6).
6. **Still refused:**

   ```text
   error[E3040]: `items` was handed over to `snapshot` on line 3, so `items.push(4)` has no list to change
     = note: `snapshot = items` hands the list over; `items` ends there
     = help: for a separate copy write `snapshot = copied items`; for one list with two names write `snapshot = shared items`
   ```

   With `other = shared items`, a loop over `items` that pushes through `other` is refused (C6; the
   message is AN-48's).
7. **Cost:** `copied`: one allocation and a copy of the elements, as C's `malloc` + `memcpy` and Rust's
   `.clone()`. `log = items`: nothing (the list header moves, three words). `shared items`: one counted
   block for the list (count and list header together, as Rust's `Rc::new(vec)`), made where `items` is
   made because the line says `shared`; a count +1 per owner made, -1 per owner ended.
8. **Kind:** a readable spelling of Rust's guarantee (Rust's move, `.clone()` and `Rc`, each named by the
   word that says it).

#### AN-35: partial move (E3042) and move in a loop (E3041)

1. **Intent:** log a player's name and keep the player; fill a list with copies of a template.
2. **OLD** (`an35_old_partial.em`, `an35_old_loop.em`):

   ```ember
   struct Player:
       name: String
       inventory: Array[String]

   fn keep(owned p: Player):
       pass

   fn log_and_keep(owned p: Player):
       n = p.name
       println(n)
       keep(p)

   fn spawn_all(owned template: Player, n: int) -> Array[Player]:
       v: Array[Player] = []
       for _ in 0..n:
           v.push(template)
       return v
   ```

   ```text
   error[E3042]: `p` is partially moved
     --> an35_old_partial.em:11:10
      |
    8 | fn log_and_keep(owned p: Player):
      |                 --------------- `p` is declared here
   ...
   11 |     keep(p)
      |          ^ used here as a whole after one of its fields was moved
      |
      = help: reassign the moved field, or destructure the whole struct up front
      = note: moving a field out leaves the rest of the struct live, but the struct cannot be used as a whole [EXP-6]

   error: could not compile due to 1 error(s)
   exit=1
   ```

   ```text
   error[E3041]: `template` is moved in a loop
    --> an35_old_loop.em:8:16
     |
   8 |         v.push(template)
     |                ^^^^^^^^ moved here, and the loop comes back
     |
     = help: move the declaration inside the loop, clone it for each iteration, or use `mem.take` when replacing it on every pass
     = note: the first iteration moves it and the second finds it gone; a value moved in a loop must be put back before the next iteration [OWN-4]

   error: could not compile due to 1 error(s)
   exit=1
   ```

3. **NEW:**

   ```ember
   fn log_and_keep(owned p: Player):
       n = p.name                    # a read-only view of p's name (R2): p stays whole
       println(n)
       keep(p)                       # after n's last use

   fn spawn_all(template: Player, n: int) -> Array[Player]:
       v: Array[Player] = []
       for _ in 0..n:
           v.push(copied template)   # a new player each pass
       return v
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `p` | `owned` on the parameter: handed over to this function | none (default: Python's locals) |
   | `n` (first function) | none (default, R2: a read-only view of a part of `p`, as row 8's plain `path`; never changes, never copies) | none (default: reads only) |
   | `template` | none (default: a parameter reads the caller's value) | none (default: a parameter never changes the caller's value) |
   | `n` (parameter) | none (default: an `int` parameter, a copy of a number, C2) | none (default: never changes the caller's) |
   | `v` | none (default: the list made on this line belongs to the name) | none (default: Python's locals) |
   | `_` | none (default: a number from the range, never kept) | none |

5. **Means exactly:** `an35_tr_ref.em` (today's `n = ref p.name` then `keep(p)`: `exit=0`, no output);
   `an35_tr_loop.em` (`template: Player`, `v.push(template.clone())`: `exit=0`, no output).
6. **Still refused:**

   ```text
   error[E3060]: `n` views `p.name`, and `keep(p)` hands `p` over on line 11 while `n` is used on line 12
     = help: use `n` before line 11, or keep your own copy: `n = copied p.name`
   ```

   **19:** (R18-09) code E3060, a view used after what it points into was handed over (section 8); was
   E3040, which is for a name used after it was itself handed over.

   **14:** (clash 1) `n = p.name` no longer takes the name out of `p` (R2); what stays refused is using the
   view after `p` is handed over. `n = ref p.name` says the same as `n = p.name` and is accepted.

   ```text
   error[E3013]: `v.push(template)` would hand over the caller's player, but `template` only reads it
     = note: `template: Player` is a plain parameter: a read-only view of the caller's player, which stays the caller's
     = help: for a player of your own in each slot write `v.push(copied template)`
   ```

   **19:** (R18-18, R18-09) the reason is the NEW signature's: `template` is a plain parameter and can
   hand nothing over at all. Was "`v.push(template)` hands `template` over on the first pass, so the
   second pass has none" (E3041), the reason of the OLD `owned template`, with a help offering `shared
   template`, which a read-only view cannot make either. Code E3013, a hand-over out of a name that does
   not own the value (section 8).

7. **Cost:** `n = p.name`: nothing (a pointer). `copied template`: one copy per pass, as C's per-element
   copy and Rust's `.clone()` in the loop.
8. **Kind:** a readable spelling of Rust's guarantee.

#### AN-36: a write to a handed-over value is accepted and lost

1. **Intent:** add a child to a tree, then set the child's value.
2. **OLD** (`an36_old_store_then_write.em`):

   ```ember
   struct Node:
       value: int
       children: Array[Node] = []

   fn main():
       root = Node(value=0)
       child = Node(value=1)
       root.children.push(child)
       child.value = 5
       println(root.children[0].value)
   ```

   ```text
   exit=0
   ```

   Accepted with no message; last run read the emitted C: the write goes to the dead local, so the
   program prints `1` (`phase3b-ownership.md` section 7, item 1; `child.value = 5LL;` after the push in
   `target/redesign2/ownership/probes/a11_emit/c/debug/c/main.c`).
3. **NEW:** the OLD program is refused (below); the two ways to write what it means:

   ```ember
   # the child is the tree's: set it first, or change it in the list
   child = Node(value=1)
   child.value = 5
   root.children.push(child)
   root.children[0].value = 6

   # or the child has two owners, the tree and this function (Python's meaning)
   struct Node:
       value: int
       children: Array[shared Node] = []

   root.children.push(shared child)
   child.value = 5                   # seen as root.children[0].value
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `root`, `child` | none (default: the node made on this line belongs to the name) | none (default: Python's locals) |
   | `value` | none (default: a field holds its value inside its node) | none (default: changed through an owner that may change the node) |
   | `children` | first form: none (default: a list held by its node); second: `shared` on the element type, each child has several owners | as `value` |
   | `child` after `push(shared child)` | `shared`: one of two owners | none (default: Python's locals; any owner may change it, C6) |

5. **Means exactly:** `an36_tr_before.em` (`exit=0`, no output); `an36_tr_shared.em` (a class `Node`,
   which is today's only spelling of "two owners and both change it"):

   ```text
   warning[L3001]: potential reference cycle: Node.children -> Node
    --> an36_tr_shared.em:4:5
     |
   4 |     children: Array[Node] = []
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ this strong field closes the shortest statically visible cycle
     |
     = help: if `Node.children` is a back-reference, make it hold a `Weak` handle
     = note: every edge in this statically visible cycle is strong; the cycle may leak at run time

   exit=0
   ```

6. **Still refused** (new: today accepts it; `[OWN-3]` gains "assigning a field of" a handed-over name):

   ```text
   error[E3040]: `child` was handed over to `root.children` on line 8, so `child.value = 5` would change nothing
     = note: after `root.children.push(child)` the node belongs to the list
     = help: set it before the push (`child.value = 5` above line 8), or change it in the list: `root.children[0].value = 5`; for two owners write `root.children.push(shared child)` with `children: Array[shared Node]`
   ```

7. **Cost:** nothing: a compile-time refusal. The `shared` form pays one counted block per child and
   the counts, as Rust's `Rc<Node>`.
8. **Kind:** a readable spelling of Rust's guarantee (Rust refuses the same write, `E0382`, last run's
   `rust_probes/a11_assign_moved_field.rs`).

#### AN-37: moving a field out from behind `mut self` (`match owned self.state` is E3013)

1. **Intent:** `Npc.update` shortens the walk path and goes idle when it is empty.
2. **OLD** (`an37_old_match_owned.em`, phase 1's `e23`):

   ```ember
   enum State:
       Idle
       Walking(path: Array[int])
       Dead

   struct Npc:
       state: State

       fn update(mut self):
           match owned self.state:
               State.Walking(path):
                   path.pop()
                   if path.is_empty():
                       self.state = State.Idle
                   else:
                       self.state = State.Walking(path)
               other:
                   self.state = other
   ```

   ```text
   error[E3013]: cannot move out of a reference
     --> an37_old_match_owned.em:12:13
      |
   12 |             State.Walking(path):
      |             ^^^^^^^^^^^^^^^^^^^^ moved out here
      |
      = help: borrow it, `ref` (`ref mut` to change it where it is); move it out where its owner is held mutably, with `mem.take`, `mem.replace` or `swap`, or `pop`, `swap_remove`, `remove` or `drain` from a container; or copy it, `.clone()`, leaving the owner's value as it is
      = note: moving out of a `ref`/`ref mut` is forbidden [EXP-6]; the referent still owns the value and drops it

   error[E3013]: cannot move out of a reference
     --> an37_old_match_owned.em:18:13
      |
   18 |             other:
      |             ^^^^^^ moved out here
      |
      = help: borrow it, `ref` (`ref mut` to change it where it is); move it out where its owner is held mutably, with `mem.take`, `mem.replace` or `swap`, or `pop`, `swap_remove`, `remove` or `drain` from a container; or copy it, `.clone()`, leaving the owner's value as it is
      = note: moving out of a `ref`/`ref mut` is forbidden [EXP-6]; the referent still owns the value and drops it

   error: could not compile due to 2 error(s)
   exit=1
   ```

3. **NEW** (row 8: `ref mut path` stays; the method's first line in row 3's words):

   ```ember
   struct Npc:
       state: State

       fn update(self) changes state:
           match self.state:
               State.Walking(ref mut path):
                   path.pop()
                   if path.is_empty():
                       self.state = State.Idle
               _:
                   pass
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `self` | `self`: the NPC the method was called on | `changes state`: this method may change `state` only (row 3) |
   | `state` | none (default: a field holds its value inside the NPC) | `changes state` |
   | `path` | `ref mut`: a name for the NPC's own walk list (row 8) | `mut` in `ref mut` |

5. **Means exactly:** `an37_tr_refmut_inline.em` (the same body with `fn update(mut self):`, today's
   spelling of `changes state`): `exit=0`, no output. Today already ends `path` at its last use
   (`path.is_empty()`), so `self.state = State.Idle` inside the arm is accepted.
6. **Still refused:**

   ```text
   error[E3013]: `match owned self.state` would take the state out of the NPC and leave it with none
     = note: a method's `self` stays whole; it can be changed in place, not emptied
     = help: to change the path where it is write `State.Walking(ref mut path):`; to take the state out and put another in write `match owned mem.replace(self.state, State.Idle):`
   ```

   And, while `path` is still used, `self.state = State.Idle` before `path.pop()`:

   ```text
   error[E3060]: `self.state = State.Idle` replaces the walk list that `path` (line 12) still uses on line 14
     = help: use `path` before this line, or set the new state after the `match`
   ```

   **19:** (R18-09) code E3060, a view used after what it points into was replaced (section 8); was
   E3025.

7. **Cost:** nothing: `path` is a pointer into the enum's payload; the pop and the store are C's.
8. **Kind:** a readable spelling of Rust's guarantee (Rust's `Walking(path) => ` on a `&mut self.state`,
   with the `&mut` written on the name it gives).

#### AN-38: `State.Walking(path):` then `path.pop()` is E3021 with a `.clone()` help; `mut path` is accepted read-only

1. **Intent:** the same NPC update, written the obvious way.
2. **OLD** (`an38_old_plain.em`, reader 4's `t03`; `an38_old_mut.em`, `t04`, the same with `mut path`):

   ```ember
   class Npc:
       state: State = State.Idle
       fn update(mut self):
           match self.state:
               State.Walking(path):
                   path.pop()
               State.Idle:
                   pass
   ```

   ```text
   error[E3021]: cannot write through a shared reference
     --> an38_old_plain.em:10:17
      |
   10 |                 path.pop()
      |                 ^^^^^^^^^^ this is `ref Array[i64]`, which only reads
      |
      = help: a reference is not pointed elsewhere: this writes through it, into what it refers to; to keep another value, keep a copy of it (`.clone()`), or its index
      = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]

   error: could not compile due to 1 error(s)
   exit=1
   ```

   `an38_old_mut.em` (`State.Walking(mut path):`) gives the same `E3021` at 10:17, with the same help, and
   `exit=1` (the `mut` is accepted and binds read-only).
3. **NEW** (row 8, exactly as agreed): `State.Walking(ref mut path):` then `path.pop()`, as in AN-37.
4. **The owner's test:** `path`: `ref mut`, a name for the NPC's own list that may change it (row 8);
   `self`: as in AN-37. 2 names, every one with its words.
5. **Means exactly:** `an38_tr_refmut.em` (reader 4's `t05`): `exit=0`, no output.
6. **Still refused** (the two agreed messages, word for word):

   ```text
   error[E3023]: `path.pop()` changes the walk list, but `path` only reads it
     = help: to change that list through `path`, write `State.Walking(ref mut path):`
   ```

   ```text
   error[E0113]: `mut path` in a pattern says "may change" but not whose list `path` is
     = help: to change that list through `path`, write `State.Walking(ref mut path):`
   ```

   **19:** (R18-09) codes only; the two agreed helps are unchanged, word for word. E3023 is the one code
   for a change through a name that only reads (R5 already gave it to pattern names; was E3021); E0113
   is new and free, for `mut` on a name (was E0100, which today is the parser's "expected a name").

7. **Cost:** nothing.
8. **Kind:** a readable spelling of Rust's guarantee.

#### AN-39: choosing among ownership kinds: eleven mechanisms

1. **Intent:** two entities share one mesh and one of them edits it.
2. **OLD** (`an39_old_kinds.em`, phase 1's `a10_ownership_kinds.em`: the `Box`, `class` and `Shared`
   spellings in one file):

   ```ember
   struct Entity:
       mesh: Box[Mesh]

   fn build() -> (Entity, Entity):
       m = Box(Mesh([0.0, 0.0, 0.0]))
       a = Entity(m)
       b = Entity(m)
       return (a, b)
   ```

   ```text
   error[E3040]: `m` has been moved out of
     --> an39_old_kinds.em:12:9
      |
   11 |     a = Entity(m)
      |         --------- moved here
   12 |     b = Entity(m)
      |         ^^^^^^^^^ used here after the move
      |
      = help: clone the value, or borrow it instead of moving it
      = note: a move gives the value away; the old owner cannot use it again

   error: could not compile due to 1 error(s)
   exit=1
   ```

   (The `class MeshC` and `Shared[Mesh]` versions in the same file pass; the spec's `[SEL-1]` table lists
   eleven mechanisms: `struct`, `Box`, `class`, `Shared`, `SyncShared`, `Weak`, arenas, `Pool`+`Handle`,
   `Cell`/`RefCell`, `Mutex`/`RwLock`/`Atomic`.)
3. **NEW:**

   ```ember
   struct Mesh:
       verts: Array[f32]

   struct Entity:
       mesh: shared Mesh

   fn build() -> (Entity, Entity):
       m = Mesh(verts=[0.0, 0.0, 0.0])
       a = Entity(mesh=shared m)
       b = Entity(mesh=shared m)
       a.mesh.verts.push(1.0)           # b sees four vertices
       return (a, b)
   ```

   The choice is made from the intent, and the word is on the line:

   | the program means | written | replaces |
   |---|---|---|
   | a value that is part of its owner | plain `T` (no word) | `struct` field |
   | one owner, in its own heap block (a type containing itself, `dyn I`, one pointer) | `owned T` | `Box[T]`, `Box[dyn I]`, `ForeignBox[T]` |
   | several owners on one thread | `shared T`, `shared x` | `Shared[T]`, a class handle's copy |
   | several owners on any threads | `thread_shared T` (area E) | `SyncShared[T]` |
   | a link that does not keep its target alive | `weak T` (area D) | `Weak[T]` |
   | a back link the language keeps | `weak T set by f` (area D) | hand-kept `Weak` |
   | a name for someone else's place | `ref`, `ref mut` | the same |
   | one value replaced whole through a read-only path | `Cell[T]` | the same |
   | a value threads change | `Mutex`, `RwLock`, `Atomic` (area E) | the same |
   | objects freed together, or found by position | `Arena`, `Pool` + `Handle[T]` (libraries) | the same |
   | a borrow checked while running | removed (AN-49) | `RefCell` |
   | an object with identity and subclasses | `class` (one owner unless `shared`) | `class` |

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `verts` | none (default: a field holds its value inside the mesh) | none (default: changed through whoever may change the mesh) |
   | `mesh` | `shared` in the type: one of several owners of a mesh | none (default: any owner may change it, C6; here through `a`, the function's own) |
   | `m` | none (default: the mesh made on this line belongs to the name; after `shared m` it is one of the owners) | none (default: Python's locals) |
   | `a`, `b` | none (default: the entity made on this line belongs to the name) | none (default: Python's locals) |

5. **Means exactly:** `an39_tr_shared_mesh.em` (`mesh: Shared[Mesh]`, `a.mesh.get_mut().verts.push(1.0)`):
   `exit=0`, no output. Today's `get_mut` is checked while running; the NEW change is checked when
   compiling (C6).
6. **Still refused:**

   ```text
   error[E3040]: `m` was handed over to `a`'s mesh on line 10, so `Entity(mesh=m)` on line 11 has no mesh to give
     = help: for one mesh in both entities write `mesh=shared m` in both (with `mesh: shared Mesh`); for two meshes write `mesh=copied m`
   ```

   `Box`, `Shared`, `SyncShared`, `Weak`, `RefCell`: each refused with the word that replaces it
   ("write `owned T`", "write `shared T`", and so on; AN-40, AN-49).
7. **Cost:** one counted block for the mesh and a count +1/-1 per owner made or ended, as Rust's
   `Rc<Mesh>`; no borrow flag (Rust needs `Rc<RefCell<Mesh>>` to edit it, and pays a flag per borrow).
8. **Kind:** a readable spelling of Rust's guarantee (one family of words, each saying its intent).

#### AN-40: `Box[T]` says nothing about one owner; `ForeignBox[T]` carries the same word

1. **Intent:** an entity owns its mesh alone; a registry owns its pools alone.
2. **OLD** (`an40_old_box.em`: `mesh: Box[Mesh]`, `pools: Array[Box[dyn Pool]]`, `Box(IntPool([1, 2]))`):
   `exit=0`, no output. The owner's spelling today (`an40_old_owned.em`, `mesh: owned Mesh`):

   ```text
   error[E0100]: expected a name
    --> an40_old_owned.em:5:11
     |
   5 |     mesh: owned Mesh
     |           ^^^^^ found `owned`
     |

   error[E0100]: expected the end of the line
    --> an40_old_owned.em:5:11
     |
   5 |     mesh: owned Mesh
     |           ^^^^^ found `owned`
     |

   error[E1010]: cannot find type `<error>` in this scope
    --> an40_old_owned.em:5:11
     |
   5 |     mesh: owned Mesh
     |           ^^^^^
     |

   error: could not compile due to 3 error(s)
   exit=1
   ```

3. **NEW** (the owner's ruling; `owned` on a foreign type as 07 planned):

   ```ember
   struct Entity:
       mesh: owned Mesh

   struct Registry:
       pools: Array[owned dyn Pool]

   @ffi(destroys(sqlite3_close))
   type SqliteDb

   struct App:
       db: owned SqliteDb

   e = Entity(mesh=Mesh(verts=[0.0]))    # the mesh is put in its own block here
   r = Registry(pools=[])
   r.pools.push(IntPool(items=[1, 2]))    # put in its own block at the push
   println(e.mesh.verts.len() + r.pools[0].len())
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `mesh` | `owned`: held alone, in its own block | none (default: changed through whoever may change the entity) |
   | `pools` | `owned dyn Pool` elements: each pool held alone | as `mesh` |
   | `db` | `owned` on a foreign type: held alone, freed through `sqlite3_close` | as `mesh` |
   | `e`, `r` | none (default: the value made on this line belongs to the name) | none (default: Python's locals) |
   | `verts`, `items` | none (default: a field holds its value) | as `mesh` |

5. **Means exactly:** `an40_old_box.em` itself (`Box[Mesh]`, `Box[dyn Pool]`): `exit=0`, no output; the
   foreign case is today's `ForeignBox[SqliteDb]` (`[FFI-36a]`, spec 4883-4887).
6. **Still refused:**

   ```text
   error[E1010]: `Box` is not part of Ember
     = help: for a mesh held alone in its own block write `mesh: owned Mesh`; for `Box[dyn Pool]` write `owned dyn Pool`
   ```

   `ForeignBox[SqliteDb]`: "write `owned SqliteDb`". `owned Player` on a class type: "a `Player` object
   always lives in its own block, and a field holds it alone unless it says `shared`; remove `owned`".
   `b.get()` on an `owned` field: "an `owned Mesh` reads as a `Mesh`; remove `.get()`".
7. **Cost:** the block `Box` made: one allocation at the store, one pointer in the owner, freed with it;
   the same as C's `malloc` and Rust's `Box`. Not covered: `Box[T, A]` with an allocator (`[ALC-1]`,
   spec 2986) has no spelling in the vocabulary yet; no annoyance of this area needs it; left to the
   combiner (14), section 6.
8. **Kind:** a readable spelling of Rust's guarantee.

#### AN-41: a type that contains itself crashes the checker

1. **Intent:** a linked list or a binary tree of values.
2. **OLD** (`an41_old_self_struct.em`: `left: Option[Tree] = None`; `an41_old_node_next.em`:
   `next: Option[Node]`; `an41_old_ref_parent.em`: `parent: Option[ref Node] = None`):

   ```text
   ===== an41_old_node_next.em

   thread 'main' (101276) has overflowed its stack
   exit=127
   ===== an41_old_ref_parent.em

   thread '<unknown>' (26864) has overflowed its stack
   exit=127
   ===== an41_old_self_struct.em

   thread 'main' (68960) has overflowed its stack
   exit=127
   ```

3. **NEW:**

   ```ember
   struct Tree:
       value: int
       left: Option[owned Tree] = None
       right: Option[owned Tree] = None

   t = Tree(value=1, left=Some(Tree(value=2)))
   ```

   A parent link is area D's `parent: weak Node set by children`.
4. **The owner's test:** `value`: none (default: a field holds its value); `left`, `right`: `owned`,
   each child held alone in its own block; `t`: none (default: the tree made on this line belongs to the
   name; Python's locals). 4 names.
5. **Means exactly:** `an41_tr_box.em` (`Option[Box[Tree]]`): `exit=0`, no output.
6. **Still refused** (the crash becomes this message):

   ```text
   error[E2200]: `Tree` contains itself through `left`, so a `Tree` would have no end
     = help: hold the child in its own block: `left: Option[owned Tree]`
   ```

   ```text
   error[E2200]: `Node.parent` would be a view of the node that holds this one, which no struct can keep
     = help: for a link back to the parent write `parent: weak Node set by children`
   ```

7. **Cost:** one block per node, as `Box` and Rust's `Option<Box<Tree>>` (one pointer, `None` is null).
8. **Kind:** an understandable limitation kept (Rust needs `Box` too), now written in the owner's word
   and with a message instead of a crash.

#### AN-42: `struct`, `Box` or `class` decides what compiles and what panics

1. **Intent:** the same ECS registry, written with class pools or with boxed pools.
2. **OLD** (four files): `an42_old_box_pools.em` (ODR-286's `box1.em`, `Box[Pool[int]]`,
   `return ref mut pool.dense[i]`): `exit=0`, no output. `an42_old_class_pools.em` (`pos.em`, class
   pools through `cast`):

   ```text
   error[E3060]: `pool.dense[…]` does not live long enough
     --> an42_old_class_pools.em:21:35
      |
   20 |         pool = cast self.pools[7u128] as Pool[T]
      |         ---------------------------------------- `pool` is a local, so its storage ends with the frame
   21 |         return ref mut pool.dense[i]
      |                                   ^ the borrow is still live when the function returns
      |                                   - `pool.dense[…]` is borrowed here
      |
      = help: return an owned value, take the destination as a `mut` parameter, or borrow something the caller owns
      = note: a returned reference must derive from a parameter (LT-1)

   error: could not compile due to 1 error(s)
   exit=1
   ```

   `an42_old_class_loop.em` (`ecs4/accept_change_during_loop.em`, class pools, `reg.emplace(3, 30)` inside
   `for x in v.values():`, marked `run-pass` though `[EXC-18]` says it panics): `exit=0`, no output.
   `an42_old_box_loop.em` (the same loop with boxed pools):

   ```text
   error[E3021]: cannot write through a shared reference
     --> an42_old_box_loop.em:10:9
      |
   10 |         self.pools[7u128].dense.push(v)
      |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this is `ref Box[Pool[i64]]`, which only reads
      |
      = help: take the borrow as `ref mut` if the callee must write, or assign to the place itself rather than through the reference
      = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]

   error[E3020]: cannot mutate `reg.pools` while it is borrowed by this loop
     --> an42_old_box_loop.em:21:9
      |
   19 |     for x in reg.values():
      |              --- `reg.pools` borrowed here for the whole loop
   20 |         total += x
   21 |         reg.emplace(30)
      |         ^^^^^^^^^^^^^^^ mutable borrow here
      |
      = help: collect the indices or values first, then mutate; use `retain` or `drain` when the operation fits
      = help: or iterate over a fixed index range and mutate only after taking the current value
      = note: a `for` loop borrows the iterable until the loop ends (CTL-2)

   error: could not compile due to 2 error(s)
   exit=1
   ```

3. **NEW:** the same text whatever the pool type is; the rules (C1, C6) are the same for a struct, an
   `owned` block and a class:

   ```ember
   struct Reg:
       pools: Map[u128, owned dyn AnyPool]

       fn get_mut[T](self, i: int) changes pools -> ref mut T:
           pool = ref mut cast self.pools[7u128] as Pool[T]
           return ref mut pool.dense[i]

       fn emplace[T](self, e: int, owned value: T) changes pools:
           ...

   total = 0
   for x in reg.values[int]():
       total += x
   reg.emplace(3, 30)                    # after the loop (the owner's "delete after the loop")
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `pools` | `owned dyn AnyPool` elements: each pool held alone | `changes pools` on the methods that change it |
   | `self` | `self` | `changes pools` |
   | `pool` | `ref mut`: a name for a pool in the map | `mut` in `ref mut` |
   | `i`, `e` | none (default: a copy of a number, C2) | none |
   | `value` | `owned`: handed over to `emplace` | none (default: Python's locals) |
   | `total`, `x` | none (default: a number; `x` a loop name that reads each element) | `total`: Python's locals; `x`: reads only |

   **14:** (clash 4) the first line follows R5's order (`changes` before `->`, as row 7's `reads`), and
   `from self` is not written: a method's result points into its object by default (R6). A `cast` of a
   place is a place (R13), so `ref mut cast ... as Pool[T]` names the pool in the map.
5. **Means exactly:** `an42_old_box_pools.em` (the `Box` form, `exit=0`, no output) is what both kinds
   compile to; the loop with the change after it is the `an42_old_box_loop.em` shape minus its two
   errors (the receiver taken with `mut self`, the `emplace` moved after the loop), as in
   `an49_tr_changes.em` (`exit=0`).
6. **Still refused**, for every kind of pool:

   ```text
   error[E3020]: `reg.emplace(3, 30)` may add to the pool this loop walks (`reg.values[int]()`, line 19)
     = note: `emplace` says `changes pools`; adding can move the pool's elements while the loop reads them
     = help: add after the loop: collect what to add in a list, then `reg.emplace(...)` for each once the loop ends
   ```

   The `run-pass` test `ecs4/accept_change_during_loop.em` becomes a compile-fail test with this message.
7. **Cost:** nothing while running: today's class pools carry access words checked while running
   (`[EXC-19]`), which go (C6).
8. **Kind:** a smarter compile-time solution (one rule set for every kind; the kind changes the layout,
   never what compiles or what panics).

#### AN-43: `owned` on a parameter means three things by type; `mem.drop(owned x)` moves a handle

1. **Intent:** `take(x)` takes over what it is given.
2. **OLD:** `an43_old_owned_handle.em` (`fn take(owned n: Node)`, a class, then `println(a.v)`):

   ```text
   warning[L1001]: `x` is never read
    --> an43_old_owned_handle.em:9:5
     |
   9 |     x = take(a)
     |     ^^^^^^^^^^^ declared here and never used
     |
     = help: remove it, or rename it `_x` if that is deliberate

   exit=0
   ```

   `an43_old_owned_int.em` (`fn take(owned n: int)`, then `println(a)`): the same `L1001` at 6:5 and
   `exit=0`. `an43_old_owned_array.em` (`owned n: Array[int]`):

   ```text
   error[E3040]: `a` has been moved out of
    --> an43_old_owned_array.em:7:13
     |
   6 |     x = take(a)
     |              - moved here
   7 |     println(a.len())
     |             ^ used here after the move
     |
     = help: clone the value, or borrow it instead of moving it
     = note: a move gives the value away; the old owner cannot use it again
     = note: assigning a list, a map or a set moves it; in Python both names would point at one list (Appendix E)

   error: could not compile due to 1 error(s)
   exit=1
   ```

   (preceded by the same `L1001`). `an43_old_mem_drop_handle.em` (`mem.drop(a)` of a class, then `a.v`):

   ```text
   error[E3040]: `a` has been moved out of
    --> an43_old_mem_drop_handle.em:7:13
     |
   6 |     mem.drop(a)
     |     ----------- moved here
   7 |     println(a.v)
     |             ^^^ used here after the move
     |
     = help: clone the value, or borrow it instead of moving it
     = note: a move gives the value away; the old owner cannot use it again

   error: could not compile due to 1 error(s)
   exit=1
   ```

   So today `owned` moves a list, copies an `int`, and gives a class a second owner the caller keeps.
3. **NEW:**

   ```ember
   class Node:
       v: int

   fn take(owned n: Node) -> int:          # takes the object over
       return n.v

   fn add_one(n: int) -> int:               # a number is copied; no word
       return n + 1

   a = Node(v=1)
   x = take(a)                              # `a` ends here
   b = 1
   y = add_one(b)
   mem.drop(x)                              # ends `x` now, for every type
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `n` (in `take`) | `owned`: handed over to `take` | none (default: Python's locals) |
   | `v` | none (default: a field holds its number) | none (default: through whoever may change the node) |
   | `n` (in `add_one`) | none (default: a number parameter is a copy, C2) | none (default: never changes the caller's) |
   | `a`, `b` | none (default: the value made on this line belongs to the name) | none (Python's locals) |
   | `x`, `y` | none (default: the number the call returns, the caller's own) | none (Python's locals) |

5. **Means exactly:** `an43_tr_handover.em` (`take(a)` with `a` not used again, `take_int(n: int)`):
   `exit=0`, no output. Today's `owned n: Node` gives the callee a second owner; the hand-over is
   exactly today's code when the caller does not use `a` again, which the NEW refusal guarantees.
6. **Still refused:**

   ```text
   error[E3040]: `a` was handed over to `take` on line 11, so `a.v` has no object to read
     = help: read it before the call, or give `take` a copy: `take(copied a)`
   ```

   ```text
   error[E2085]: `owned n: int`: an `int` is copied, not handed over
     = help: remove `owned`: write `fn take(n: int) -> int:`
   ```

   ```text
   error[E3040]: `a` ended at `mem.drop(a)` on line 6
     = help: use `a` before line 6, or remove the `mem.drop`
   ```

   (for every type, an `int` included: 06's `k3_mem_drop_int.em` is accepted today).
   `fn take(n: owned Node)`: "to take it over write `owned n: Node`; to read it write `n: Node`" (06).
   `take(shared a)` into `owned n: Node`: "`take` takes a `Node` held alone; to give it a second owner
   declare `owned n: shared Node`" (**19:** (R18-07) was "declare `n: shared Node`", which after 15's R4
   change reads the caller's value instead of making `take` an owner).
7. **Cost:** nothing: a hand-over passes the pointer or the value as today, and no count is made for a
   class (today's `[FN-9]` retains the caller's handle for the call).
8. **Kind:** a readable spelling of Rust's guarantee (`owned` is Rust's move, for every type).

#### AN-44: `b = a` has no word and does three things by type

1. **Intent:** `p2 = player` and `snapshot = items` in one program.
2. **OLD** (`an44_old_three.em`: `n2 = n` of an `int`, `snapshot = items`, `p2 = player` of a class,
   `p2.hp = 5`, `println(player.hp)`): `exit=0`, no output; it prints `5` (the class is shared). And the
   struct case (`an44_old_copy_struct.em`, last run's attack: `pos = p.pos; pos.x += 1.0` of a
   `@derive(Copy)` struct, Python prints `2.0`): `exit=0`, no output; it prints `1.0`, silently.
3. **NEW:**

   ```ember
   n = 1
   n2 = n                     # a number: copied (it has no parts to change)
   n2 += 1
   items = [1]
   snapshot = items           # handed over: `items` ends (row 9)
   player = Player()
   p2 = shared player         # one object, two owners: a change through either is seen by both
   p2.hp = 5
   println(player.hp)         # 5
   p = P(pos=Vec2(x=1.0, y=2.0))
   pos = ref mut p.pos        # change p's position
   pos.x += 1.0
   mine = copied p.pos        # your own copy to change
   mine.y = 0.0
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `n`, `n2` | none (default: a number; `=` copies it, C2) | none (Python's locals) |
   | `items`, `player`, `p` | none (default: the value made on this line belongs to the name) | none (Python's locals) |
   | `snapshot` | none (default, row 9: `=` hands the list over) | none (Python's locals) |
   | `p2` | `shared`: one of two owners of the player | none (Python's locals; any owner may change it, C6) |
   | `pos` | `ref mut`: a name for `p`'s position | `mut` in `ref mut` |
   | `mine` | `copied`: its own new position | none (Python's locals) |
   | `hp`, `x`, `y` | none (default: a field holds its number) | through the name before them |

5. **Means exactly:** `an44_tr_three.em` (today's class `p2 = player` is exactly a second owner):
   `exit=0`, no output; `an44_tr_struct_refmut.em` (`pos = ref mut p.pos; pos.x += 1.0`, prints `2.0`):
   `exit=0`, no output; `copied p.pos` is today's `p.pos` copy of a `Copy` struct.
6. **Still refused:**

   ```text
   error[E3040]: `player` was handed over to `p2` on line 8, so `player.hp` has no object to read
     = note: `p2 = player` hands the object over, as for every value that is not a number
     = help: for one player with two names write `p2 = shared player`; for a second player write `p2 = copied player`
   ```

   ```text
   error[E3023]: `pos.x += 1.0` changes `p.pos` through `pos`, which only reads it
     = note: `pos = p.pos` names `p`'s position, a part of `p`, read-only
     = help: to change `p`'s position write `pos = ref mut p.pos`; to change your own copy write `pos = copied p.pos`
   ```

   **19:** (R18-09) code E3023, the one code for a change through a name that only reads (was E3021).

   **14:** (clash 1) under R2 `pos = p.pos` is a read-only view of the part, not a silent copy, so this is
   area A's E3021 message; C's proposed E3045 is not needed. The two lines to write are unchanged.

7. **Cost:** nothing; `shared player` is a count +1 (and -1 when `p2` ends), as Rust's `Rc::clone`.
8. **Kind:** a readable spelling of Rust's guarantee (Rust's move, `Copy`, `Rc::clone`), with the
   struct copy that Rust also makes silently now named.

#### AN-45: `.clone()` on a class handle shares the object instead of copying it

1. **Intent:** make an independent copy of a player object and change it.
2. **OLD** (`an45_old_clone_class.em`: `b = a.clone()`, `b.health = 5`, `println(a.health)`): `exit=0`, no
   output; it prints `5` (spec 797, 1277: `clone` of a class is a second handle).
3. **NEW:**

   ```ember
   @derive(Clone)
   class Player:
       health: int

   a = Player(health=100)
   b = copied a                      # a new player
   b.health = 5
   println(a.health)                 # 100
   c = shared a                      # the same player, a second owner
   ```

4. **The owner's test:** `health`: none (default: a field holds its number); `a`: none (default: the
   object made on this line belongs to the name; Python's locals); `b`: `copied`, its own new object;
   `c`: `shared`, a second owner of `a`'s object. 4 names; each may be changed as Python's locals.
5. **Means exactly:** `an45_tr_copied.em` (`b = Player(a.health)`, what `Player`'s `Clone` runs):
   `exit=0`, no output.
6. **Still refused:**

   ```text
   error[E1010]: `a.clone()`: a copy is written `copied a`
     = help: for a new player write `b = copied a`; for a second owner of this one write `b = shared a`
   ```

   **19:** (R18-09) code E1010, a name the program may not use, with the word to write (section 8); was
   E2020, which today is a type mismatch.

   `copied a` of a class with no `Clone`: "`Player` does not say how to copy it; add `@derive(Clone)`
   above `class Player`, or write `b = shared a` for a second owner".
7. **Cost:** the copy where `copied` is written (one block and the fields), as C's `malloc` + copy and
   Rust's `.clone()` of a `Box`.
8. **Kind:** a readable spelling of Rust's guarantee.

#### AN-46: silent copies no word names: `comptime(e)` of a heap value; `sorted(m.keys())`

1. **Intent:** use a constant table; print a map's keys in order.
2. **OLD:** `an46_old_comptime.em` (`t = comptime(table())` in a loop, `table()` returns `Array[int]`):
   `exit=0`, no output; by spec 4038-4040 each run-time evaluation clones the table into a new
   allocation. `an46_old_sorted_keys.em` (`for k in sorted(m.keys()): println(k)`): `exit=0`, no output;
   by spec 4155 the array holds clones of the keys.
3. **NEW:**

   ```ember
   for i in 0..3:
       t = comptime(table())          # a view of a table made when compiling: no copy
       total += t[i]
   own = copied comptime(table())      # your own list, which may change
   own.push(4)

   for k in sorted(m.keys()):          # views of m's own keys, sorted: no copy
       println(k)
   names = sorted(m.keys().copied())   # your own strings, kept past changes to m
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `i` | none (default: a number from the range) | none |
   | `t` | none (default: a value made when compiling is read-only data; the name is a view of it, never copies) | none (default: it never changes) |
   | `total` | none (default: a number, Python's locals) | none (Python's locals) |
   | `own` | `copied`: its own list | none (Python's locals) |
   | `k` | none (default: a loop name that reads each element; here a view of one of `m`'s keys) | none (reads only) |
   | `names` | `.copied()`: its own strings | none (Python's locals) |

5. **Means exactly:** `an46_tr_static.em` (`static TABLE: Array[int] = comptime(table())`,
   `t = ref TABLE`: no copy, as spec 4040 says): `exit=0`, no output. `an46_tr_copied.em`
   (`t = TABLE.clone()`, `t.push(4)`): `exit=0`, no output. `an46_tr_keys_copied.em` (today's
   `sorted(m.keys())`, which clones, kept past `m.insert`): `exit=0`, no output. The view form cannot be
   said in today's Ember (an `Array` cannot hold views, `[TYP-15]`); in C, `bench/an46_sorted_views.c`
   (an array of pointers to the map's own keys, `qsort`, printed): built with clang `-O2`, run once:
   `a`, `b`, `an46-sorted-views ms=0.017 peak_kb=4060 checksum=3105`.
6. **Still refused:**

   ```text
   error[E3023]: `t.push(4)` changes `t`, a view of a table made when compiling, which never changes
     = help: for your own list write `t = copied comptime(table())`
   ```

   The sorted keys kept as a list and used after a change to `m`:
   `ks = sorted(m.keys())`, then `m.insert(String("z"), 1)`, then `println(ks[0])`:

   ```text
   error[E3060]: `m.insert(...)` changes `m`, whose keys `ks` (line 7) still views on line 9
     = help: use `ks` before this line, or keep your own strings: `ks = sorted(m.keys().copied())`
   ```

   **19:** (R18-19, R18-09) the refused program is now written out, so the message names `ks`, a name
   the program has (was "`keys`", which no line of the NEW code binds: the NEW loop is `for k in
   sorted(m.keys()):`). Codes: E3023 for the change through a read-only view (was E3021), E3060 for a
   view used after a change to what it points into (was E3020, which is for a list changed while a loop
   walks it).

7. **Cost:** less than today: no allocation per evaluation of the table (read-only data, as C's
   `static const`), and no string copy per key (an array of pointers, as the C above and Rust's
   `let mut ks: Vec<&String> = m.keys().collect(); ks.sort();`). Copies only where `copied` is written.
8. **Kind:** a smarter compile-time solution (the value is placed in read-only data and lent; the
   sorted keys are a local list of views, area A's AN-07 rule).

#### AN-47: `owned` in an expression: E0109 contradicts Annex A, and its help is wrong for `b = owned a`

1. **Intent:** hand a value to a new name.
2. **OLD:** `an47_old_b_owned_a.em` (`b = owned a`):

   ```text
   error[E0109]: `owned` is not permitted in expression position
    --> an47_old_b_owned_a.em:3:9
     |
   3 |     b = owned a
     |         ^^^^^
     |
     = help: `owned` marks a parameter, a receiver, a closure or a consumed scrutinee; to move a value, pass it to an `owned` parameter

   error: could not compile due to 1 error(s)
   exit=1
   ```

   `an47_old_owned_fn_expr.em` (`task = owned fn() => println("done")`, Annex A's form): `exit=0`, no
   output.
3. **NEW:**

   ```ember
   a = [1, 2]
   b = a                               # hands the list over
   task = fn() => println("done")      # captures nothing
   job = fn() with owned report: send(report)   # takes `report` over (planned capture word)
   ```

4. **The owner's test:** `a`: none (default: the list made on this line belongs to the name); `b`: none
   (default, row 9: `=` hands over); `task`: none (default: the function made on this line belongs to
   the name; it captures nothing); `job`: none (as `task`); `report` in the capture list: `owned`, handed
   over to the function. 5 names; each may be changed as Python's locals (the captured `report` is the
   function's own).
5. **Means exactly:** `an47_tr.em` (`b = a`; `task = fn() => println("done")`): `exit=0`, no output.
6. **Still refused** (E0109's new text, the same for every place before a value; Annex A's
   `task = owned fn() => ...` is rewritten):

   ```text
   error[E0109]: `owned a` is not needed: `b = a` already hands `a` over
     = help: write `b = a`; for a copy write `b = copied a`; for a second owner write `b = shared a`
   ```

   ```text
   error[E0109]: `owned fn` does not say what the function takes or copies
     = help: name it on each captured name; `task` captures nothing, so write `task = fn() => println("done")`; a name not listed is the real variable
   ```

   **19:** (R18-08) one code, E0109, and one text for `owned fn` in every answer (AN-47, AN-54, AN-78):
   "`owned fn` does not say what the function takes or copies" / "name it on each captured name: ...;
   a name not listed is the real variable", with the program's own names in the line to write. Here the
   function captures nothing, so the line is the lambda without the word (was "name what it takes:
   `fn() with owned report:`", a name from another line of the NEW block).

7. **Cost:** nothing.
8. **Kind:** a readable spelling of Rust's guarantee (a message that gives the line).

#### AN-48: shared mutable state: a camera and a HUD both change one player

1. **Intent:** the camera and the HUD both change the player.
2. **OLD:** the class form, `an48_old_class.em` (phase 1's `a09_shared_state.em`): `exit=0`, no output.
   Its second loop grows the inventory through `cam.target` while it walks it through `hud.target`; it
   compiles and is left to run-time access words, which panic (`[EXC-1]`, last run's
   `a09_fields_emit`). The struct form, `an48_old_struct.em` (`target: ref mut Player` in both):

   ```text
   error[E3022]: `p` is already mutably borrowed
     --> an48_old_struct.em:16:15
      |
   15 |     cam = Camera(ref mut p)
      |                  --------- mutable borrow of `p` starts here
   16 |     hud = Hud(ref mut p)
      |               ^^^^^^^^^ conflicting access here
   17 |     cam.target.hp -= 1
      |                      - borrow later used here
      |
      = help: use one mutable access rather than borrowing the same place twice; split the owner's mutable view with `split_at` when the intended operands are disjoint parts of it
      = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

   error: could not compile due to 1 error(s)
   exit=1
   ```

3. **NEW:**

   ```ember
   class Player:
       hp: int = 100
       inventory: Array[String] = []
       log: Array[String] = []

   class Camera:
       target: shared Player

       fn shake(self) changes target:
           self.target.hp -= 1

   class Hud:
       target: shared Player

   fn main():
       p = Player()
       cam = Camera(target=shared p)
       hud = Hud(target=shared p)
       cam.shake()
       hud.target.inventory.push(String("key"))
       for item in hud.target.inventory:
           cam.target.hp -= item.len()            # a number: accepted
       for item in hud.target.inventory:
           cam.target.log.push(copied item)       # another list of the player: accepted

   fn transfer(a: shared Player, ref mut b: shared Player):
       if a is not b:
           for it in a.inventory:
               b.inventory.push(copied it)        # accepted: the compiler knows they differ
   ```

   **15:** `b` gains `ref mut`: a parameter's mode is on its name (R4), so a plain `b: shared Player`
   reads the caller's player, and a change through it is E3023 (R5). The C is unchanged: both
   parameters are pointers, and no count is made for either (R10 point 1: nothing in `transfer` ends a
   `Player` owner).

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `hp`, `inventory`, `log` | none (default: a field holds its value in the player) | through the name before them |
   | `target` (both classes) | `shared`: one of several owners of a player | `changes target` on `shake`; through `cam`, `hud` (the function's own) in `main` |
   | `self` (in `shake`) | `self` | `changes target` |
   | `p`, `cam`, `hud` | none (default: the object made on this line belongs to the name) | none (Python's locals) |
   | `item`, `it` | none (default: a loop name reads each element, never changes, never copies) | none (reads only) |
   | `a` | `shared` in the parameter type: a view of the caller's `shared` player (**15:** was "the function holds an owner of each") | none (default: a parameter reads) |
   | `b` | `ref mut` on the name, `shared` in the type: a name for the caller's `shared` player | `ref mut`: yes (**15:** was "none (default: any owner may change it, C6)"); **19:** (R18-01) by R4's one rule `ref mut` names the caller's place, so it covers both the player (`b.inventory.push(...)`) and the caller's link (`b = shared q` would point the caller's link at `q`) |

5. **Means exactly:** `an48_tr_class.em` (today's class form of the accepted lines, with the refused
   loop rewritten as its message says: collect, then push after the loop): `exit=0`, no output.
   `an48_tr_is_not.em` (`transfer` with `if a is not b:`): `exit=0`, no output. Today both carry run-time
   access words on `inventory` and `log` (`[EXC-19]`); the NEW model has none (C6).
6. **Still refused:**

   ```text
   error[E3020]: `cam.target.inventory.push(...)` may add to the list this loop walks (`hud.target.inventory`, line 23)
     = note: `cam.target` and `hud.target` may be the same `Player`, and adding can move the list's elements while the loop reads them
     = help: add after the loop: collect the new items in a list, then `cam.target.inventory.extend(new_items)` once the loop ends; or loop over `copied hud.target.inventory`
   ```

   The same for a call: a method whose first line says `changes target` (it may reach the player's
   inventory) called inside that loop: "`cam.shake()` says `changes target`, which may be the `Player`
   whose inventory this loop walks; call it after the loop". `transfer` without the `is not` test:
   "`a` and `b` may be the same player, and this loop walks `a.inventory` while adding to
   `b.inventory`; test first: `if a is not b:`". A `shared` value sent to another thread: area E's
   message.
7. **Cost:** nothing checked while running. Counts: +1/-1 per `shared` owner made or ended; for the
   two views the loops make (`hud.target.inventory`), no count (C7: nothing in the loop may end a
   `Player` owner). Today's class pays 8 bytes of access words per non-`Copy` field and checks where not
   proved away (1.75x to 1.87x of plain C in last run's workload, `phase3b-ownership.md` section 4); Rust
   pays `Rc<RefCell<Player>>`'s flag on each borrow (1.48x to 1.57x here, section 5).
8. **Kind:** a smarter compile-time solution: Rust's guarantee (no view outlives a reshape of what it
   points into) checked from first lines instead of a run-time flag, with the owner's "a list may not
   change while a loop walks it" kept as the limitation.

#### AN-49: `RefCell` hides a run-time check that panics; E3023 offers it to a beginner who forgot `mut`

1. **Intent:** change a value reached through a shared path (and: a counter's `bump` changes its count).
2. **OLD:** `an49_old_refcell.em` (spec IX.7's `Scene` with `entities: RefCell[Array[int]]`,
   `with list = s.entities.borrow_mut(): list.push(e)`): `exit=0`, no output.
   `an49_old_refcell_conflict.em` (`borrow()` then `borrow_mut()` of the same cell, nested): `exit=0`,
   no output: accepted, and it panics while running (`[CELL-5]`). `an49_old_forgot_mut.em`:

   ```text
   error[E3023]: cannot mutate borrowed parameter `self`
    --> an49_old_forgot_mut.em:5:9
     |
   4 |     fn bump(self):
     |             ---- `self` is borrowed here
   5 |         self.n += 1
     |         ^^^^^^ this write needs mutable access
     |
     = help: restructure to a single owner and declare this parameter `mut self: Counter`
     = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
     = help: if the value has identity, make it a `class`
     = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

   error: could not compile due to 1 error(s)
   exit=1
   ```

3. **NEW:**

   ```ember
   struct Scene:
       entities: Array[int]

       fn add(self, e: int) changes entities:
           self.entities.push(e)

   struct Counter:
       n: int

       fn bump(self) changes n:
           self.n += 1
   ```

   A scene several objects own is `scene: shared Scene`, changed through any owner (AN-48).
4. **The owner's test:** `entities`, `n`: none (default: a field holds its value); `self` (both):
   `self`, with `changes entities` / `changes n` saying what may change; `e`: none (default: a copy of a
   number). 5 names.
5. **Means exactly:** `an49_tr_changes.em` (`fn add(mut self, e: int)`, today's spelling of
   `changes entities`): `exit=0`, no output.
6. **Still refused** (`RefCell`, `Ref`, `RefMut`, `borrow`, `borrow_mut`, `RuntimeChecked` removed):

   ```text
   error[E1010]: `RefCell` is not part of Ember
     = note: a change is said on the method that makes it, and checked when compiling
     = help: write the field plainly, `entities: Array[int]`, and the method's first line as `fn add(self, e: int) changes entities:`; for a scene several objects own write `shared Scene`
   ```

   ```text
   error[E2105]: `bump` changes `self.n`, but its first line does not say so
     = help: write `fn bump(self) changes n:`
   ```

   **14:** code E2105, one code for every first line that does not match its body (clash 8).

   The conflict `an49_old_refcell_conflict.em` panics on becomes a compile-time refusal: a view of
   `s.entities` (`a = ref s.entities`) still used while `s.entities.push(...)` runs: "`s.entities.push`
   changes the list `a` views (line 6), which it still uses on line 8; use `a` before, or keep a copy:
   `a = copied s.entities`".
7. **Cost:** less than today: no borrow flag (one word per `RefCell`, checked on every borrow in every
   profile, `[CELL-5]`, `[CELL-9]`); nothing runs.
8. **Kind:** a smarter compile-time solution (Rust's run-time-checked `RefCell` replaced by C6's
   compile-time rule).

#### AN-50: "handle" names a class object's counted pointer and also `Handle[T]`

1. **Intent:** read a message about a stale handle and know which kind it means.
2. **OLD:** `an50_old_temp_handle.em` (`first = ref o.bag.items[0]; o.bag = Bag(); println(first)`):

   ```text
   error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
     --> an50_old_temp_handle.em:10:17
      |
   10 |     first = ref o.bag.items[0]
      |                 ^^^^^ a temporary value, dropped at the end of this statement
   11 |     o.bag = Bag()
   12 |     println(first)
      |             ----- borrow later used here
      |
      = help: bind the handle to a variable first (`c = p.child`), and borrow through that
      = note: a handle stored in an object can be replaced through another handle, which would free what the borrow points into; the borrow keeps its own copy only to the end of the statement [RC-5]

   error: could not compile due to 1 error(s)
   exit=1
   ```

   `an50_old_assign_self.em` (`self = Node()` in a class method):

   ```text
   error[E2103]: cannot assign to `self` in a class method
    --> an50_old_assign_self.em:5:9
     |
   5 |         self = Node()
     |         ^^^^ `self` names the object this method was called on
     |
     = help: to re-point a caller's handle, take it as a `mut` parameter [FN-9]
     = note: a `mut self` method holds every field of its object until it returns [EXC-15]

   error: could not compile due to 1 error(s)
   exit=1
   ```

   `an50_old_pool_handle.em` (`pool[h]` after `pool.remove(h)`): `exit=0`, no output; it panics with
   `stale handle; use .get(h) for an Option` (`std/src/collections.em:1477`).
3. **NEW:** no new code; the word "handle" is used only for `Handle[T]`, a position in a `Pool`
   (vocabulary settlement 11). A class object is "the `Player` object `p`", a link in a field "the link
   `o.bag`". The program the first message is about, as the new message makes it:

   ```ember
   first = copied o.bag.items[0]       # or use `first` before replacing the bag
   o.bag = Bag()
   println(first)
   ```

4. **The owner's test:** `first`: `copied`, its own number (for an `int`, plain `=` copies too, C2);
   `o`: none (default: the object made on its line belongs to the name; Python's locals); `bag`: none
   (default: a field holds its `Bag` object alone, C4); `items`: none (default: a field holds its list).
   4 names.
5. **Means exactly:** `an50_tr_use_before.em` (`first = o.bag.items[0]`, an `int` copied, then the bag
   replaced): `exit=0`, no output.
6. **Still refused**, with the new wording:

   ```text
   error[E3060]: `o.bag = Bag()` replaces the `Bag` object that `first` (line 10) points into, and `first` is used on line 12
     = help: use `first` before line 11, or keep your own copy: `first = copied o.bag.items[0]`
   ```

   ```text
   error[E2103]: `self = Node()` cannot replace the `Node` object `reset` was called on
     = help: set its fields instead: `self.v = 0`, with `fn reset(self) changes v:`
   ```

   ```text
   panic: `pool[h]`: `h` refers to a value already removed from `pool`
     help: use `pool.get(h)`, which gives `None` for a removed value
   ```

7. **Cost:** nothing.
8. **Kind:** a readable spelling (messages in the program's own names).

#### AN-51: a count pair the spec allows to remove is emitted every loop step

1. **Intent:** read each watcher's target in a loop.
2. **OLD** (`an51_old_watchers.em`, last run's `a09_emit/src/main.em`: `t = ws[i].target` then
   `t.hp -= 1`, `t.inventory.push(frame)` and a loop over `t.inventory`): `exit=0`, no output. Last run
   read the C today's compiler emits for it: `ember_retain_plain(...target)` before
   `t = ((em_7Watcher*)(*ws_17).ptr)[_29].target;` and `ember_release_plain((ember_obj_header*)t);` at
   the step's end (`target/redesign2/ownership/probes/a09_emit/c/release/c/main.c`, lines 310-312 and
   368, re-read now), measured at 13.5% and 13.9% of the workload (`phase3b-ownership.md` section 4).
3. **NEW:**

   ```ember
   class Player:
       hp: i32 = 1000
       x: i32 = 0
       inventory: Array[i32] = []

   struct Watcher:
       target: shared Player

   fn step(ref mut ws: Array[Watcher], frame: i32) -> u64:
       acc: u64 = 0
       for i in range(ws.len()):
           t = ref mut ws[i].target      # a view of the watcher's player: no count
           t.hp -= 1
           acc += (t.x as u64) * ((t.inventory.len() + 1) as u64)
           if (i & 7) == (frame & 7) as int:
               t.inventory.push(frame)
               if t.inventory.len() > 8:
                   for v in t.inventory:
                       acc += v as u64
                   t.inventory.clear()
       return acc
   ```

   **14:** (clash 7) `t = ref mut ws[i].target` changes the player reached through `ws`, so `ws` says
   `ref mut` (R5: a plain name never changes anything, an object behind a `shared` link included; 06's `let`
   rule C-8 reads it the same way). The list and its links are not changed, and the C is the same.
4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `hp`, `x`, `inventory` | none (default: a field holds its value) | through the name before them |
   | `target` | `shared`: one of several owners of a player | through `t` |
   | `ws` | `ref mut`: the caller's list of watchers | `ref mut`: yes (the players its watchers link to are changed through it) |
   | `frame` | none (default: a copy of a number) | none |
   | `acc` | none (default: a number, Python's locals) | none (Python's locals) |
   | `i`, `v` | none (default: a loop name reads each value) | none |
   | `t` | `ref mut`: a name for the player `ws[i].target` links to | `mut` in `ref mut` |

5. **Means exactly:** in C, `bench/share_c.c` `-DMODEL=3` (`Player *t = ws[i].target;` with no retain
   or release around the step; counts change only at re-targeting), whose checksum equals plain C's
   (`80363660820`, section 5).
6. **Still refused / kept:** nothing new is refused. The pair is kept (one +1 at `t`'s start, one -1 at
   its end) when a line in `t`'s life may end an owner of a `Player`: a write to any place of type
   `shared Player` (`ws[j].target = shared q`), the end of a local `shared Player` owner, or a call whose
   first line reaches one (`changes` naming a field that holds a `shared Player`, or an unseen call).
7. **Cost:** 0 count operations per step where today makes 2 (a load-add-store each); the 13.5% to
   13.9% last run measured goes.
8. **Kind:** a smarter compile-time solution (`[RC-3]`'s removal, decided from first lines, `[RC-2b]`'s
   one-expression limit lifted).

#### AN-52: shared class objects re-pointed often ran at 1.16x to 1.17x plain C (last run's limit)

1. **Intent:** game objects (watchers) keep re-pointing their targets.
2. **OLD:** `an51_old_watchers.em` (`retarget`: `ws[i].target = players[r]`): `exit=0`, no output.
   Last run measured today's emitted code at 1.71x to 1.78x of plain C and its proposed lean class at
   1.16x to 1.17x from a slab (`phase3b-ownership-attack.md` 5.3), with no fix found. The C-speed
   spelling in today's Ember, watchers keeping views into `players`, is refused
   (`an52_old_views.em`, `struct Watcher: target: ref mut Player`, `ws: Array[Watcher]`):

   ```text
   error[E3063]: `Watcher` is a view, so it may not be stored in an `Array` unless it is `static`
     --> an52_old_views.em:10:26
      |
   10 |     ws: Array[Watcher] = [Watcher(target=ref mut players[0])]
      |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ stored here
      |
      = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
      = note: the view borrows `players`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

   error[E3063]: `Watcher` is a view, so it may not be stored in an `Array` unless it is `static`
     --> an52_old_views.em:12:13
      |
   12 |     ws[0] = Watcher(target=ref mut players[1])
      |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ stored here
      |
      = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
      = note: the view borrows `players`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

   error: could not compile due to 2 error(s)
   exit=1
   ```

3. **NEW: the line says what the watcher means, and each meaning has its cost.** Tried to save last
   run's class first (section 5): the cost is the count touch on the new target, a random object; a count
   kept beside the objects did not help last run (1.16x), and no count-based form can skip that touch. So
   the answer is to stop counting where the program does not mean "keeps alive":

   ```ember
   # (a) the watcher keeps its player alive: counted, as Rust's Rc
   struct Watcher:
       target: shared Player
   ws[i].target = shared players[r]

   # (b) the watcher follows a player that `players` owns and that outlives the watchers:
   #     a kept view, no count (needs area B's AN-24: a local list of view structs)
   @view
   struct Watcher:
       target: ref mut Player
   players: Array[owned Player] = ...             # `struct Player`, each in its own block (C4;
                                                  # a `class Player` list is plain `Array[Player]`)
   ws: Array[Watcher] = ...                       # local: lives while `players` does
   ws[i] = Watcher(target=ref mut players[r])     # re-point: one store

   # (c) a position in the owning list
   struct Watcher:
       target: int
   players[ws[i].target].hp -= 1
   ```

   (b)'s rules, all checked when compiling: `ws` may not outlive `players` (row 5's rule for a view);
   while `ws` is used, `players` may grow (its elements are `owned`, each in its own block, so growing
   moves only the list of pointers) but nothing may be removed from it or replace it; several watchers'
   `ref mut` views may name one player (C6 applies to them as to `shared` owners: plain fields change
   freely, a view into `inventory` ends at a reshape of any player's `inventory`). `ref mut`
   **parameters** stay exclusive (checked by the caller, 06 C-1), so the C compiler may still assume two
   `ref mut` parameters do not overlap; views **kept** in a list or struct get no such assumption.
4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `target` (a) | `shared`: one of several owners | through the watcher's owner (C6) |
   | `target` (b) | `ref mut`: a name for a player `players` owns | `mut` in `ref mut` |
   | `target` (c) | none (default: a number, the player's position) | none (default: Python's locals for the watcher) |
   | `players` | `owned` elements: each player held alone by the list | none (Python's locals) |
   | `ws` | none (default: the list made on this line belongs to the name; in (b) a list of views, kept until its last use, row 5) | none (Python's locals) |
   | `i`, `r` | none (default: numbers) | none |
   | `hp` | none (default: a field holds its number) | through the name before it |

5. **Means exactly:** in C, `bench/share_c.c`: (a) `-DMODEL=3`; (b) `-DMODEL=8` (32-byte players, no
   header, no count, the bounds check on `players[r]`); (c) `-DMODEL=9` (players inline, a `u32`
   position, a bounds check on every read). Every build printed checksum `80363660820`, plain C's.
   (c) in today's Ember: `an52_tr_positions.em`: `exit=0`, no output.
6. **Still refused** for (b):

   ```text
   error[E3060]: `players.remove(r)` would end a player that `ws` (line 14) still points at on line 30
     = help: remove players after the last use of `ws`, or let the watchers keep them alive: `target: shared Player`
   ```

   **19:** (R18-09) code E3060, a view used after what it points into ended (section 8); was E3020,
   which is for a list changed while a loop walks it.

   ```text
   error[E3063]: `ws` holds views of `players`, so it may not outlive it (returned here)
     = help: return positions instead (`target: int`), or let the watchers own a share: `target: shared Player`
   ```

7. **Cost** (section 5, three batches, PROVISIONAL): (a) 1.13x to 1.25x of plain C, against 1.48x to
   1.57x for safe Rust's `Rc<Player>` with `Cell`; (b) 1.00x to 1.06x, the same memory as plain C
   (53,284 KB); (c) 0.80x to 0.85x and 8% less memory, against 0.86x to 0.90x for safe Rust's `Vec` and
   positions.
8. **Kind:** a new mechanism (kept `ref mut` views that may name one object, checked by C6's
   compile-time rule; Rust allows only one `&mut`), with (a)'s counting kept as the understandable cost
   of "keeps alive", which Rust pays more for.

## 13. Area D: linked objects, callbacks, interface types (AN-53 .. AN-70)

Copied from `RUN3/11-area-D.md` by `build.py`; the combiner's changes are marked **14:** and listed in section 4. File names in this area's text are relative to `target/redesign3/11-area-D/` unless they say otherwise; "section N" inside it means the area report's own section N.

### Terms of area D

- **function value**: a function or lambda kept in a variable, list or field (a callback, a handler):
  two words, its code and a pointer to its **state** (what it captured).
- **capture**: a name a lambda uses from the function that made it. **The real variable** (row 6): the
  capture is that very variable, not a copy; a write in the lambda is seen by the function, and back.
- **capture list**: what follows `with` after a lambda's parameters (`fn() with copied score:`).
- **box**: a small counted heap block holding one captured variable, shared by the function that owns
  the variable and every function value that captures it, freed with the last of them (Python's "cell").
- **count pair**: one increment and one decrement of an object's count around something.
- **view**: a name for something owned elsewhere that may not outlive it (`ref T`, a read-only
  parameter, a pattern name).
- **holder**: the object whose field holds another (a parent for its children).
- **deferred-drop region** (new mechanism, AN-62): a stretch of code during which an object whose count
  reaches zero is set aside whole (its fields and its memory untouched) and dropped when the stretch ends.
- **unknown call**: a call through a function value, a `dyn` value or another package's function whose
  effects are not known, which may therefore change anything a program may change through it.
- **AddressSanitizer** (ASan): a clang option that stops a program with a report the moment it reads or
  writes memory it must not.
- **today**: `target/redesign3/bin/ember.exe check <file>`. Every "today" claim has its output pasted.

### Area D's files (its section 2)

All under `C:\Users\ism19\Code\ember\target\redesign3\11-area-D\`:

- `old/*.em`: the OLD programs. 16 are last run's probes, copied unchanged from
  `target/redesign2/graphs-callbacks/probes/` (file names changed to the annoyance ids), 3 are the
  readers' and builder's (`04-reader-4/t13_o5.em`, `t10_w2111.em`, `05-builder/d1_cast_forms.em`), the
  rest are new. `old/check_output.txt` is made by `run_checks.sh old` (every file, `ember check`, exit
  code).
- `means/*.em`: every "means exactly" translation into today's Ember; `means/check_output.txt` the same
  way.
- `c/`: the C translations, each with its output file:
  `an53_captures.c` (`an53_captures_output.txt`), `an61_dispatch.c` (`an61_dispatch_output.txt`, three
  modes), `an62_defer_asan.c` (`an62_defer_asan_output.txt`, three modes), `an63_links.c`
  (`an63_links_output.txt`). ASan build: `clang -O1 -g -fsanitize=address -I target/redesign/bench`,
  run with the ASan DLL directory first on `PATH`, `ASAN_OPTIONS=abort_on_error=0`.
- `c/an62_walk_bench.c` (last run's `graphs-callbacks-attack2/bench/w2v_backlink.c` with one variant
  added, `-DDEFER`), `c/an62_walk_plain.c` (`w2v_plain.c`, unchanged), `c/ember_model.h` (unchanged);
  built `clang -O2 -DSLAB [-DARRAY24|-DRETAIN|-DDEFER]` into `c/bin/`; assembly `c/bin/walk_*.s`;
  timing `c/walk_runs.txt` by `target/redesign2/bench/run.ps1 -Rounds 5`; medians `c/walk_summary.txt`.

No Ember program was run. No compiler, spec or standard-library file was changed.

### Rules of area D (D1 .. D8)

The answers in section 4 refer to these by number.

**D1. A capture not named is the real variable** (row 6). `fn(): score += 1` reads and writes the
`score` of the function that wrote the lambda, wherever the function value goes. Its place is chosen
when compiling and never changes what the program does:
- **an address** when the compiler proves the function value ends before the variable does: it is kept
  only in locals, local lists, or objects that never leave the function, and every method it is handed
  to says what it changes (row 3's `changes handlers`, or `mut self`, on an object that itself stays in
  the function);
- **a box** otherwise (the function value is returned, stored in something that outlives the function,
  or handed to code whose first line does not limit where it goes).
`ember inspect --alloc` lists each box, as it lists every allocation today (`[CLO-10]`).

**D2. Named captures** (planned-words section 4): `with copied x` (the function's own copy), `with
owned x` (handed over; refused for a `Copy` type with "write `with copied x`"), `with shared x`, `with
thread_shared x`, `with weak x` (each with its meaning before a value). Each name carries its own word.

**D3. What a call changes.** Calling a function value changes what its body changes: the real variables
it captured (through their address or box) and its own named state (`copied`, `owned`). A function
value whose body changes **its own** state is called through a name that may change it (the owner, or a
`ref mut` parameter); one that changes only real variables may be called through any name. A variable
that a living function value changes is treated as changed by every call that may run that function
value: no view into it is kept across such a call (the attack on last run's pointer captures,
`phase3b-graphs-callbacks-attack.md` 5.2, closed at compile time).

**D4. A stored function value is an object** (last run's A32 answer, kept): its state is in its own
block, so the two words `{code, state}` may move with the list. A call through a list element or field
copies the two words out and keeps no pointer into the list. The body of a function value whose state
may be dropped by a call it makes (it removes itself), and which uses that state during or after the
call, holds one count on its state across that call (last run's two attack fixes, 6.2 and 6.3, kept).

**D5. Unknown calls and the objects a loop holds.** An object reached through a place (**15:** was "a
handle read from a place"; "handle" is kept for `Handle[T]`, AN-50) and used after an unknown
call is protected against that call ending its object only if the call could end it: if the object's
class is the target of a `weak` link, or is held `shared` or `thread_shared`, anywhere the call can
reach. A one-owner object lent read-only cannot be ended by anyone during the loan (the borrow rule).
Where protection is needed: a **deferred-drop region** around the loop (AN-62) when the class has no
`drop` method of its own; a count pair across the call otherwise.
**14:** merged with C7 into R10 (clash 10), with one point added where D6 meets area E: a view of a
`thread_shared` object reached through a `weak` link holds one atomic count for its life, because another
thread may end the last owner at any moment and a deferred-drop region belongs to one thread.

**D6. `weak`** (planned-words 3.2): a link that does not keep its target alive; reading it gives
`Option`. Reading tests the target's header (last run's combined choice, `phase3b-combined.md` 4.3:
Rust's mechanism; the registration alternative needed three fixes and costs 1.09x to 1.27x on churn).
A name bound from the `Some` is kept alive until its last use under D5's rule. `Some(ref mut p)` binds a
view that may change the target; the target then counts as "reachable by an unknown call" (D5) and
under area C's rule for objects with several ways in (AN-48). A `weak` link to a `thread_shared` value
uses the atomic count its header already says it has (no word).

**D7. `set by`** (planned-words 3.3): `parent: weak Node set by children`. Rules, from last run's back
links as saved by both attacks, rewritten for one-owner lists:
1. The named field (`children`) belongs to the target's type and holds this type with one owner
   (`Array[Node]`, `Option[Node]`) for a single link, or weak elements for a list link
   (`into: Array[weak City] set by roads`).
2. Putting an object into the holder field sets its link; taking it out clears it. One membership change,
   in this order: unlink the old value, link the new one, release the old one (attack 3.2's fix: a field
   assigned the value it already holds).
3. A holder that ends clears its elements' links before releasing them.
4. The link is never assigned by the program; the holder field is changed only through its own
   operations (`push`, `insert`, `remove`, `pop`, `clear`, assigning the whole field), never lent as a
   changeable list or span to a function that does not keep the links (last run's rule 6).
5. Reading the link is a plain load; no test: the holder cannot end while it holds this object without
   clearing the link first (rule 3).

**D8. Interface values say who owns them**: `owned dyn I` (one owner; replaces `Box[dyn I]`), `shared
dyn I` (several owners, counted), `dyn I` as a parameter's type (a read-only view; `ref mut v: dyn I`
for a changeable one), `some I` in a return type (as today), `[T: I]` for a generic. A bare `I` as a
type is refused.

### The answers of area D

#### AN-53: A closure that changes a variable of the function that made it (A12; agreed row 6)

**1. Intent:** the button adds 1 to `game`'s score.

**2. OLD.** The Python spelling (`old/an53_button_plain.em`, last run's `a12_button_python.em`):

```ember
class Button:
    handlers: Array[fn()] = []

    fn on_click(self, f: fn()):
        self.handlers.push(f)

    fn click(self):
        for h in self.handlers:
            h()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)
    b.click()
    return score
```

```text
error[E2020]: expected `fn() -> void`, found `Callable0`
 --> an53_button_plain.em:6:28
  |
6 |         self.handlers.push(f)
  |                            ^ this is `Callable0`
  |

error[E3026]: this lambda borrows what it captures, so it may not be stored as `fn() -> void`
 --> an53_button_plain.em:6:28
  |
6 |         self.handlers.push(f)
  |                            ^ stored here
  |
  = help: write it `owned fn`, which moves or copies each capture into the value
  = note: a `fn` type outside a parameter is an owned callable value; a lambda that borrows is a view, and this place has no bounding region (CLO-3, TYP-15)

error[E3021]: cannot write through a shared reference
  --> an53_button_plain.em:10:13
   |
10 |             h()
   |             ^^^ this is `ref fn() -> void`, which only reads
   |
   = help: a callable value may change what it captured, so it is called from a mutable place [CLO-2]: borrow it as `ref mut`, and iterate a collection of them with `.iter_mut()` (`for f in fs.iter_mut():`)
   = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]

error: could not compile due to 3 error(s)
exit=1
```

Following the help (`old/an53_button_owned_fn.em`: `owned f`, `.iter_mut()`, `b.on_click(owned fn():
score += 1)`) is accepted, and by `[CLO-2]` the handler adds 1 to its own copy, so `game()` returns 0:

```text
=== an53_button_owned_fn.em
exit=0
```

**3. NEW.**

```ember
class Button:
    handlers: Array[fn()] = []

    fn on_click(self, owned f: fn()) changes handlers:
        self.handlers.push(f)

    fn click(self) changes handlers:
        for ref mut h in self.handlers:
            h()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)       # row 6: game's real `score`
    b.click()
    return score                       # 1

fn game_copy() -> int:
    score = 0
    b = Button()
    b.on_click(fn() with copied score: score += 1)   # row 6's copy: the handler's own `score`
    b.click()
    return score                       # 0: only the handler's copy went up
```

**19:** (R18-02) `click` was `fn click(self):` with `for h in self.handlers: h()`. A stored `fn()` may
be a function that changes its own state, such as row 6's agreed copy spelling, and D3 asks for a name
that may change it; the type `fn()` cannot say which, so by R12 a call through a stored function value
is a change of the place that holds it: `click` says `changes handlers` and calls through `ref mut h`.
With that, `game_copy`, row 6's copy spelling given to this very `Button`, is defined: accepted, and the
handler's own copy changes through `click`, whose first line says so. The "means exactly" file was
already this form (`m53_button_box.em`: `for h in self.handlers.iter_mut():`, `exit=0`, pasted
below), and the C passes the same two words. `game_copy` means exactly point 2's
`old/an53_button_owned_fn.em` (`b.on_click(owned fn(): score += 1)` with `.iter_mut()`, `exit=0`):
today's `owned fn` copies the `int`, which is what `with copied score` names.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `handlers` (field) | none (default: the Button's own list; each element its own function value) | `changes handlers` on `on_click` |
| `self` in `on_click` | `self`: the Button the method was called on | `changes handlers` |
| `f` | `owned`: handed over to `on_click` | its new owner may; it is handed on to the list |
| `self` in `click` | `self` | `changes handlers` (**19:** was "no (plain `self`, read-only)"; R12) |
| `h` | `ref mut`: an element of `handlers` (**19:** was none (default: a read-only view)) | `ref mut`: yes; calling it changes what its body changes, its own state included (D3, R12) |
| `score` in `game` | none (default: `game`'s own variable, made by `=`) | its owner `game` may, by assignment |
| `score` in the lambda | none (agreed row 6: the real `score` of `game`) | yes, as row 6 says (`score += 1`) |
| `b` | none (default: `game`'s own Button, made by `=`) | through methods that say `changes` |
| `score` in `game_copy`'s lambda (**19:**) | `copied`: the handler's own copy (row 6) | yes, its own copy, through `click`'s `changes handlers` |

**5. Means exactly.** In today's Ember the real variable shared by `game` and the handler is a counted
object (`means/m53_button_box.em`):

```ember
class Score:
    n: int = 0
...
fn game() -> int:
    score = Score()
    b = Button()
    b.on_click(owned fn(): score.n += 1)
    b.click()
    return score.n
```

```text
=== m53_button_box.em
exit=0
```

That is D1's box. Here `b` never leaves `game` and `on_click` says it changes only `handlers`, so the
compiler proves the handler ends with `b`, before `score`, and passes `score`'s address instead: the C is
`c/an53_captures.c` case 1, `on_click(&b, (Fn0){bump, &score})`, run under ASan:

```text
game=1
counter=2
an53_captures ms=0.040 peak_kb=8176 checksum=0
```

(`counter=2` is case 2, a counter factory whose variable lives in a box because the function value is
returned: D1's other place.) No ASan report.

**6. Still refused.**
- A capture of someone else's place in a function value that may outlive it (AN-57's example):
  "the click handler changes `total`, which belongs to `register`'s caller, and `b` keeps the handler
  after `register` returns; keep the total where the handler can own a share of it:
  `count = shared Counter()` and `fn() with shared count: count.n += 1`".
- A capture used by a thread that may outlive it (AN-55's messages).
- A loop that walks a variable a living function value changes, around a call that may run it (D3):
  "`hs[0]()` may run the function made on line 3, which changes `xs`, while this loop walks `xs`;
  collect the changes and make them after the loop".

**7. Cost.** An address: none (the C passes `&score`). A box: one allocation per variable instance and
one load per use; last run measured the callback workload at 1.018 and 1.052 of plain C with the shared
total boxed, against 1.045 and 1.037 with it by address
(`target/redesign2/graphs-callbacks-attack2/bench/runs/w4_a.txt`, `w4_b.txt`, summarised in
`phase3b-graphs-callbacks-attack.md` 6.4), and a boxed variable written in a hot loop at 0.990 and 0.998
of a C local (`runs/micro_a.txt`, `micro_b.txt`, same report 5.3). Rust's spelling of the same program
is `Rc<Cell<i32>>`, the same box.

**8. Kind:** a smarter compile-time solution (Rust refuses the program; here the compiler proves the
cheaper place and otherwise uses the box Rust makes the programmer write).

---

#### AN-54: `owned fn` moves a `String` capture and copies an `int` one (agreed row 6)

**1. Intent:** a callback that uses a name and a score.

**2. OLD.** `old/an54b_owned_fn_int.em` (only the `int`):

```ember
fn run(mut f: fn()):
    f()

fn main():
    score = 0
    cb = owned fn(): score += 1
    run(cb)
    println(score)
```

```text
=== an54b_owned_fn_int.em
exit=0
```

Accepted, and it prints 0 by `[CLO-2]` (the `int` is copied). The same word with a `String` too
(`old/an54_owned_fn_two_types.em`, `cb = owned fn(): score += 1; println(name)`, then
`println(score)` and `println(name)`): the `String` was handed over, the `int` was not:

```text
error[E3040]: `name` is borrowed after it has been moved out of
  --> an54_owned_fn_two_types.em:13:13
   |
 8 |     cb = owned fn():
   |          ----------- moved here
...
13 |     println(name)
   |             ^^^^ borrowed here after the move
   |
   = help: borrow before the move, or clone the value so the move takes the copy
   = note: the borrow would point at memory whose owner gave it away
   = note: assigning a list, a map or a set moves it; in Python both names would point at one list (Appendix E)

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.** The usual callback names nothing (row 6, the real variables). A callback that must own the
name and keep its own score says so on each name:

```ember
fn run(ref mut f: fn()):
    f()

fn main():
    name = String("ann")
    score = 0
    cb = fn() with owned name, copied score:
        score += 1                    # its own copy
        println(name)                 # its own String
    run(cb)
    println(score)                    # 0, as `copied` says
```

**15:** `"ann".to_string()` became `String("ann")`, row 5's spelling for your own `String` made from
text (vocabulary rows `String(...)` and `.to_string()`); the meaning is the same.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` | `ref mut`: `main`'s function value, lent | yes (`ref mut`): `cb` changes its own copy when called (D3) |
| `name` in `main` | none (default: `main`'s own String, made by `=`) | its owner may |
| `score` in `main` | none (default: `main`'s own `int`) | its owner may |
| `cb` | none (default: `main`'s own function value) | its owner may (it is lent `ref mut` to `run`) |
| `name` in the lambda | `owned`: handed over to the function | its new owner may |
| `score` in the lambda | `copied`: the function's own copy | yes, its own copy (`score += 1`) |

**5. Means exactly.** Today's `owned fn` does exactly this (moves the `String`, copies the `int`); NEW
only writes it on each name (`means/m54_named_captures.em`):

```text
=== m54_named_captures.em
exit=0
```

**6. Still refused.**
- `owned fn(): ...`: E0109, "`owned fn` does not say what the function takes or copies; name it on each
  captured name: `fn() with owned name, copied score:`; a name not listed is the real variable"
  (**19:** (R18-08) the code and the opening sentence added, so the three answers give one code and one
  text).
- `fn() with owned score:` for an `int`: "an `int` is copied, not handed over; write `fn() with copied
  score:`" (vocabulary C-2).
- `println(name)` after `with owned name`: "`name` was handed to the function on line 7; to keep
  using it here, give the function a copy, `with copied name`, or share it, `with shared name`".
- `fn() with copied name, score:`: "`copied` here belongs to `name` only; say what the function does
  with `score`: `with copied name, copied score`, or leave `score` out to use the real `score`"
  (planned-words section 4; **19:** (R18-15) the first clause added: it is R3's one rule for a word
  before a list of names, which AN-15's loop now follows too).
- `run(cb)` with `fn run(f: fn()):`: "`cb` changes its own copy of `score` each time it runs, and
  `run` takes it read-only; write `fn run(ref mut f: fn()):`" (D3).

**7. Cost.** None beyond C: the `String`'s three words move into the function's state, the `int` is
copied, as a Rust `move` closure.

**8. Kind:** a readable spelling of Rust's guarantee (`move`, written per name).

---

#### AN-55: The compiler decides a capture is moved from which function receives the lambda (`[CLO-15]`)

**1. Intent:** count `xs` on another thread, and record the values an iterator sees into `seen`.

**2. OLD.** `old/an55_spawn_moves.em`:

```ember
fn main():
    xs = [1, 2, 3]
    seen: Array[int] = []
    _n = xs.iter().inspect(fn(v) => seen.push(v)).count()
    h = thread.spawn(fn() => work(xs))
    println(seen.len())
    println(xs.len())
    println(h.join())
```

```text
error[E3026]: the thread's function borrows `xs`, which it may outlive
  --> an55_spawn_moves.em:11:22
   |
11 |     h = thread.spawn(fn() => work(xs))
   |                      ^^^^^^^^^^^^^^^^ the thread may outlive what it views
   |
   = help: write it `owned fn`, which moves or copies each capture into it; or borrow it inside `with scope = thread.scope():`, which joins its threads before the block ends [THR-5]
   = note: a thread may outlive every local of the code that started it [THR-10]

error: could not compile due to 1 error(s)
exit=1
```

The same lambda text views `seen` at `inspect` and, by `[CLO-15]`, would be moved at a registry: the
callee decides, nothing in the line says it.

**3. NEW.**

```ember
fn main():
    xs = [1, 2, 3]
    seen: Array[int] = []
    _n = xs.iter().inspect(fn(v) => seen.push(v)).count()     # the real `seen`
    h = thread.spawn(fn() with owned xs: work(xs))           # `xs` handed to the thread
    println(seen.len())
    println(h.join())
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `xs` in `main` | none (default: `main`'s own list) | its owner may |
| `seen` in `main` | none (default: `main`'s own list) | its owner may |
| `v` | none (default: a read-only view of the element; an `int`, read) | no |
| `seen` in the lambda | none (row 6: the real `seen`) | yes (`push`), as row 6 says |
| `_n` | none (default: `main`'s own `int`, the result of `count()`) | its owner may |
| `h` | none (default: the `JoinHandle` `thread.spawn` returns, `main`'s own) | its owner may (`join`) |
| `xs` in the thread's function | `owned`: handed over to the thread | its new owner may |

**5. Means exactly.** `means/m55_spawn_owned.em` (`thread.spawn(owned fn() => work(xs))`, `xs` not
used after) and the scoped form `means/m55_scope.em` (the thread uses the real `xs` inside
`with scope = thread.scope():`):

```text
=== m55_scope.em
exit=0
=== m55_spawn_owned.em
exit=0
```

**6. Still refused.** `thread.spawn(fn() => work(xs))`: "the thread may still run after `main` ends,
and `xs` belongs to `main`; hand it to the thread, `thread.spawn(fn() with owned xs: work(xs))`, give it
a copy, `with copied xs`, or start the thread inside `with scope = thread.scope():`, which waits for it".
A box (D1) is never used across threads: its count is a one-thread count (area E's rule; `with
thread_shared x` is the shared form).

**7. Cost.** None beyond C: the view is an address, the hand-over moves three words.

**8. Kind:** a readable spelling of Rust's guarantee (`move` and scoped threads), with no rule that
depends on the callee.

---

#### AN-56: A lambda parameter without a mode takes `owned` from the expected callable type (`[TYP-23]`)

**1. Intent:** pass each name to a callback that keeps it in `names`.

**2. OLD.** `old/an56_lambda_param_owned.em`:

```ember
fn each_name(owned f: fn(owned String)):
    f("ann".to_string())

fn main():
    names: Array[String] = []
    each_name(fn(s) => names.push(s))
    println(names.len())
```

```text
=== an56_lambda_param_owned.em
exit=0
```

`s` has no word, yet it is handed over into `names` (`[TYP-23]`, ODR-025).

**3. NEW.**

```ember
fn each_name(owned f: fn(owned String)):
    f(String("ann"))

fn main():
    names: Array[String] = []
    each_name(fn(owned s) => names.push(s))       # takes `s` over, keeps it
    each_name(fn(s) => println(s.len()))          # only reads `s`
```

**15:** `"ann".to_string()` became `String("ann")` (row 5's spelling); the meaning is the same.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` | `owned`: handed over to `each_name` | its new owner may |
| `names` | none (default: `main`'s own list) | its owner may |
| `s` in the first lambda | `owned`: handed over to the lambda | its owner may (it hands it on to `names`) |
| `names` in the first lambda | none (row 6: the real `names`) | yes (`push`), as row 6 says |
| `s` in the second lambda | none (default for a parameter: a read-only view) | no |

**5. Means exactly.** `means/m56_lambda_owned_param.em` (`fn(owned s) => names.push(s)`) and
`means/m56_lambda_read_param.em` (`fn(s: String) => println(s.len())` where `fn(owned String)` is
expected):

```text
=== m56_lambda_owned_param.em
exit=0
=== m56_lambda_read_param.em
exit=0
```

A read-only lambda where a taking one is expected: the caller hands the `String` over, the lambda reads
it, and it ends after the call, as if the function had taken it and dropped it.

**6. Still refused.** `each_name(fn(s) => names.push(s))`: "`s` is only read here, so it cannot be kept
in `names`; to take it over write `fn(owned s) => names.push(s)`". `fn(owned s)` where the expected type
is `fn(String)` (a read-only parameter): "`each_name` lends the name and keeps it; to keep your own,
write `fn(s) => names.push(copied s)`".

**7. Cost.** None: the same code as today.

**8. Kind:** a readable spelling of Rust's guarantee (the parameter's mode is written, as on every other
parameter, vocabulary C-1).

---

#### AN-57: E3026 is reported inside the callee; a stored lambda gets E2020 naming `Callable0`

**1. Intent:** find which call wrote the lambda that cannot be kept.

**2. OLD.** `old/an57_button_owned_param.em` (last run's `a12_button_owned.em`, the lambda written at
line 16, `b.on_click(fn(): score += 1)`):

```text
error[E3026]: this lambda borrows what it captures, so it may not be stored as `fn() -> void`
 --> an57_button_owned_param.em:6:28
  |
6 |         self.handlers.push(f)
  |                            ^ stored here
  |
  = help: write it `owned fn`, which moves or copies each capture into the value
  = note: a `fn` type outside a parameter is an owned callable value; a lambda that borrows is a view, and this place has no bounding region (CLO-3, TYP-15)

error: could not compile due to 1 error(s)
exit=1
```

The error points at line 6 inside `Button.on_click`, not at line 16. Without `owned` on the parameter
the first error names a compiler type (`old/an53_button_plain.em`, pasted under AN-53):
`error[E2020]: expected \`fn() -> void\`, found \`Callable0\``.

**3. NEW.** The button program of AN-53 compiles (D1). What is still refused is reported at the lambda,
in the program's names:

```ember
fn register(ref mut b: Button, ref mut total: int):
    b.on_click(fn(): total += 1)
```

```text
error: the click handler changes `total`, which belongs to `register`'s caller
 --> app.em:2:16
  |
2 |     b.on_click(fn(): total += 1)
  |                ^^^^^^^^^^^^^^^^ written here, kept in `b`
  |
  = why: `b` keeps the handler after `register` returns, and `total` may have ended by then
  = help: keep the total where the handler can own a share of it:
          `count = shared Counter()` in the caller, and `b.on_click(fn() with shared count: count.n += 1)`
```

Callable types are printed as written: `fn()`, `fn(ref mut Ui)`, never `Callable0`.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `b` | `ref mut`: the caller's Button | yes (`ref mut`) |
| `total` | `ref mut`: the caller's `int` | yes (`ref mut`) |
| `total` in the lambda | none (row 6: the real `total`, here the caller's place) | yes, as row 6 says; refused because it may outlive it |

**5. Means exactly.** Nothing compiles for the refused program, which is the point. The accepted form in
the help is AN-53's box (`means/m53_button_box.em`, `exit=0`, pasted under AN-53); its `shared Counter`
is today's class `Score` there.

**6. Still refused.** The program above, with the message above. A lambda given to a callee that keeps
it while the lambda captures a view (a `ref`, `ref mut`, pattern or `for` name): the same message shape,
at the lambda's line, naming the captured view and the holder that keeps the lambda.

**7. Cost.** None (compile time only).

**8. Kind:** an understandable limitation kept (a function value may not outlive a place it does not
own; Rust's E0373), with the message at the right line and in the program's names.

---

#### AN-58: A closure stored in a local list that ends first is still E3026

**1. Intent:** collect a few local callbacks in a list and run them.

**2. OLD.** `old/an58_local_list.em` (last run's `a12_local_list.em`):

```ember
fn game() -> int:
    score = 0
    hs: Array[fn()] = []
    hs.push(fn(): score += 1)
    for h in hs.iter_mut():
        h()
    return score
```

```text
error[E3026]: this lambda borrows what it captures, so it may not be stored as `fn() -> void`
 --> an58_local_list.em:5:13
  |
5 |     hs.push(fn(): score += 1)
  |             ^^^^^^^^^^^^^^^^ stored here
  |
  = help: write it `owned fn`, which moves or copies each capture into the value
  = note: a `fn` type outside a parameter is an owned callable value; a lambda that borrows is a view, and this place has no bounding region (CLO-3, TYP-15)

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
fn game() -> int:
    score = 0
    hs: Array[fn()] = []
    hs.push(fn(): score += 1)
    for h in hs:
        h()
    return score                       # 1
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `score` | none (default: `game`'s own `int`) | its owner may |
| `hs` | none (default: `game`'s own list; its elements its own function values) | its owner may |
| `score` in the lambda | none (row 6: the real `score`) | yes, as row 6 says |
| `h` | none (default: a read-only view of an element; calling it changes what its body changes, D3) | no |

**5. Means exactly.** `hs` is declared after `score` and never leaves `game`, so it ends first and the
lambda holds `score`'s address (D1): the C of `c/an53_captures.c` case 1 (the same `{code, &score}` pair,
there kept in a local `Button`), clean under ASan, output pasted under AN-53 (`game=1`).

**6. Still refused.** The loop of D3: `for x in xs: hs[0]()` where `hs[0]` changes `xs`: "`hs[0]()` may
run the function made on line 3, which changes `xs`, while this loop walks `xs`; collect the changes and
make them after the loop". (Last run's `attacks2.exe 11` showed the use after free this refusal
prevents.)

**7. Cost.** None (the C passes an address).

**8. Kind:** a smarter compile-time solution (the list's end is proved; Rust needs the same proof through
lifetimes, and accepts this program).

---

#### AN-59: A closure that gives a capture away gets E3013 with ten options instead of "declare the parameter `once fn()`"

**1. Intent:** `call_twice` runs a callback that hands a string to `consume`.

**2. OLD.** `old/an59_call_twice.em` (reader 4's `t13_o5.em`):

```ember
fn call_twice(f: fn()):
    f()
    f()

fn consume(owned s: String):
    println(s)

fn main():
    s = "a".to_string()
    call_twice(fn() => consume(s))
```

```text
error[E3013]: cannot move out of a reference
  --> an59_call_twice.em:10:32
   |
10 |     call_twice(fn() => consume(s))
   |                                ^ moved out here
   |
   = help: borrow it, `ref` (`ref mut` to change it where it is); move it out where its owner is held mutably, with `mem.take`, `mem.replace` or `swap`, or `pop`, `swap_remove`, `remove` or `drain` from a container; or copy it, `.clone()`, leaving the owner's value as it is
   = note: moving out of a `ref`/`ref mut` is forbidden [EXP-6]; the referent still owns the value and drops it

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
fn call_once(owned f: once fn()):
    f()

fn call_twice(f: fn()):
    f()
    f()

fn main():
    s = String("a")
    call_once(fn() with owned s: consume(s))      # the function owns `s` and gives it away: once
    t = String("b")
    call_twice(fn(): consume(copied t))           # each run hands over a new copy
```

**15:** `"a".to_string()` and `"b".to_string()` became `String("a")` and `String("b")` (row 5's
spelling); the meaning is the same.

**19:** (R18-06) `call_once` takes `owned f: once fn()` (was `f: once fn()`): using the function up
is a hand-over, and the mode is on the name for every type (R4), the form 15 required of `shared`. In
today's Ember (`target/redesign3/19-fixer/m59_owned_once.em`, `fn call_once(owned f: once fn()):` with
`call_once(owned fn() => consume(s))`):

```text
===== m59_owned_once.em
exit=0
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `f` in `call_once` | `owned`: handed over to `call_once`; `once` in its type: a function the call uses up | it is used up by the call (**19:** `owned` added) |
| `f` in `call_twice` | none (default: a read-only function value; calling it changes only what its body changes, D3) | no |
| `s` in `main` | none (default: `main`'s own String) | its owner may |
| `s` in the first lambda | `owned`: handed over to the function | its owner may (it hands it to `consume`) |
| `t` in `main` | none (default: `main`'s own String) | its owner may |
| `t` in the second lambda | none (row 6: the real `t`, read); `copied` before it makes the value handed over | no (read only) |

**5. Means exactly.** `means/m59_once.em` (`call_once(f: once fn())` with `owned fn() => consume(s)`)
and `means/m59_copy_each_time.em` (`call_twice(fn() => consume(s.clone()))`):

```text
=== m59_copy_each_time.em
exit=0
=== m59_once.em
exit=0
```

**6. Still refused.**
- `call_twice(fn() with owned s: consume(s))`: "this function gives `s` away, so it can run only once,
  and `call_twice` may run it twice; if `call_twice` runs it once, declare the parameter
  `owned f: once fn()`; otherwise hand over a copy each time: `fn(): consume(copied s)`" (**19:** was
  `f: once fn()`).
- **19:** `fn call_once(f: once fn()):`: "`call_once` uses `f` up, so it must hold it: write
  `owned f: once fn()`" (R12).
- `call_twice(fn(): consume(s))` (no word): "`consume(s)` would hand `main`'s `s` over from inside a
  function that `main` keeps; give the function its own `s` to hand over, `fn() with owned s:
  consume(s)` (then it runs once), or hand over a copy, `consume(copied s)`".

**7. Cost.** None: the same code as today's `once fn` and `.clone()`.

**8. Kind:** a readable spelling of Rust's guarantee (`FnOnce`), with the message the annoyance asks for.

---

#### AN-60: A callback receiving the object that holds it, and a handler that removes itself (A32)

**1. Intent:** the UI calls each handler with the UI; a handler may add handlers (they run in the same
dispatch, as in Python) or remove itself.

**2. OLD.** The Python spelling (`old/an60_fire_for.em`, last run's `a32_fire_python.em`):

```ember
class Ui:
    handlers: Array[fn(Ui)] = []
    clicks: int = 0

    fn fire(self):
        for h in self.handlers:
            h(self)
```

```text
error[E3021]: cannot write through a shared reference
 --> an60_fire_for.em:8:13
  |
8 |             h(self)
  |             ^^^^^^^ this is `ref fn(Ui) -> void`, which only reads
  |
  = help: a callable value may change what it captured, so it is called from a mutable place [CLO-2]: borrow it as `ref mut`, and iterate a collection of them with `.iter_mut()` (`for f in fs.iter_mut():`)
  = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]

error: could not compile due to 1 error(s)
exit=1
```

Following the help (`old/an60_fire_itermut.em`):

```text
error[E3080]: `self.handlers` cannot be read while it is mutably borrowed
 --> an60_fire_itermut.em:8:15
  |
7 |         for h in self.handlers.iter_mut():
  |                  ------------- mutable borrow of `self.handlers` starts here
8 |             h(self)
  |               ^^^^ conflicting access here
  |
  = help: bind the borrow of `self.handlers` to a local and finish with it before this line, or copy the value out first
  = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
class Ui:
    handlers: Array[fn(ref mut Ui)] = []
    current: int = 0
    clicks: int = 0

    fn fire(mut self):
        self.current = 0
        while self.current < len(self.handlers):
            self.handlers[self.current](self)       # the handler may add or remove handlers
            self.current += 1

    fn remove_current(self) changes handlers, current:
        self.handlers.remove(self.current)
        self.current -= 1

fn count(ref mut ui: Ui):
    ui.clicks += 1

fn add_once(ref mut ui: Ui):                      # adds a handler, then removes itself
    ui.handlers.push(count)
    ui.remove_current()
```

Python's list iterator is this loop (a position checked against the length at every step), so a handler
added during the dispatch runs in it and a handler that removes itself makes the next one move into its
place, as in Python. A handler that must not keep the UI alive captures it weakly:
`ui.button.on_click(fn() with weak ui: if Some(ref mut me) = ui: me.clicks += 1)` (planned-words 3.2).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `handlers` | none (default: the Ui's own list of function values) | `changes handlers`, or `mut self` |
| element type `fn(ref mut Ui)` | `ref mut` on its parameter: a handler gets the caller's Ui | yes, the Ui (`ref mut`) |
| `current`, `clicks` | none (default: the Ui's own `int`s) | `changes current`, `mut self` |
| `self` in `fire` | `self` | yes (`mut self`) |
| `self` in `remove_current` | `self` | `changes handlers, current` |
| `ui` in `count` | `ref mut`: the caller's Ui | yes |
| `ui` in `add_once` | `ref mut`: the caller's Ui | yes |
| `count` given to `push` | none (default: a function with no state, copied like a number) | no |
| `ui` in the `with weak ui` lambda | `weak`: a link that does not keep the Ui alive | no (a link); `me` may (below) |
| `me` | `ref mut` on the pattern name: a view of the Ui that may change it | yes |

**5. Means exactly.** Today's Ember accepts only the unsound form (AN-61), so the translation is C:
`c/an61_dispatch.c` mode 2, the code D4 emits for `fire`, a handler that pushes 8 handlers (the list's
buffer moves) and then removes itself and reads its own state:

```c
for (u.current = 0; u.current < u.len && u.current < 12; u.current++) {
    Callable c = u.ptr[u.current];             /* two words copied out; no pointer into the list */
    c.fn(c.st, &u);
}
...
static void once_body(Box *st, Ui *u) {
    st->counter += st->m;
    for (int k = 0; k < 8; k++) push(u, (Callable){count_body, box_new(1)});   /* the buffer moves */
    if (g_mode == 2) st->rc++;                     /* hold: the state is used after a call that may drop it */
    remove_current(u);                             /* removes this very handler: drops the list's count */
    u->total += st->counter;                       /* uses its own state after the removal */
    if (g_mode == 2) box_release(st);
}
```

Under ASan (`c/an61_dispatch_output.txt`):

```text
--- mode 2
mode2 clicks=8 total=5
an61_dispatch ms=0.069 peak_kb=8232 checksum=2
```

The 8 handlers added during the dispatch ran in it (`clicks=8`), as in Python; no ASan report. Mode 3,
the same without the hold, proves the hold is needed:

```text
--- mode 3
=================================================================
==79044==ERROR: AddressSanitizer: heap-use-after-free on address 0x12032f7a0798 at pc 0x7ff7c24f36d6 bp 0x00c7c17ff550 sp 0x00c7c17ff598
READ of size 8 at 0x12032f7a0798 thread T0
    #0 0x7ff7c24f36d5 in once_body C:\Users\ism19\Code\ember\target\redesign3\11-area-D\c\an61_dispatch.c:93
```

(line 93 is `u->total += st->counter;`, the handler reading its own state after removing itself.)

**6. Still refused.**
- `for h in self.handlers: h(self)` with `fn(ref mut Ui)` handlers (the agreed limitation: a list may
  not change while a loop walks it): "a handler gets `self` and may change `self.handlers`, which this
  loop walks; walk by position, as Python's list does:
  `while self.current < len(self.handlers): self.handlers[self.current](self)`; if the handlers never
  change the list, give them a read-only Ui: `handlers: Array[fn(Ui)]`, with
  `fn fire(self) changes handlers:` and `for ref mut h in self.handlers: h(self)`" (**21:** (F-07) the
  second way out restored, spelled as R12 asks; **19:** had removed it: see below).
- Not refused, warned: a handler that captures the UI that holds it with `with shared ui` makes a cycle
  of owners that is never freed (a leak, not a memory error, as in Rust); AN-64's L3001 names it: "the
  handler is kept by `ui` and keeps `ui`; capture it weakly, `with weak ui`, or take the Ui as the
  handler's parameter".

**19:** (R18-02) Was: "With `handlers: Array[fn(Ui)]` (read-only Ui) on a Ui with one owner, the `for`
loop is accepted and nothing protects anything: no handler can reach the list to change it." Under
R12 a loop that calls stored handlers says `changes handlers` and walks them with `ref mut h` (a
stored `fn(Ui)` may change its own state), and `h(self)` would then hand the handler a second name for
the list that `h` is an element of, which R4 refuses. The position loop of point 3, which copies each
handler's two words out before the call (D4), is the one form for handlers that get the Ui, whether
they may change it or only read it.
**21:** (F-07) That reason is withdrawn. By R12 (21), a call through a stored value copies it out and
does not make its list element an argument, so `h(self)` gives the handler no second name that R4
compares, exactly as the accepted `self.handlers[self.current](self)` does not. With read-only
`fn(Ui)` handlers no handler can change the list, so the `for` walk meets the agreed limitation, and
each handler's own state changes through `ref mut h`, which `changes handlers` allows. Owner's test
on that form: `self` in `fire`: `self`, `changes handlers`; `h`: `ref mut`, an element of `handlers`,
may change it (its own state included); the handler's `ui`: none (default: a read-only view of the
Ui, `fn(Ui)`), never changes it.

**7. Cost.** A call copies two words out of the list (plain C copies the same `{fn, env}` pair). A
handler's body holds one count on its own state only across a call that may remove it and only when it
uses its state after it (a rare path). Last run measured this dispatch at 1.045 and 1.037 of plain C
doing exactly the same work with the same allocator, safe Rust at 1.490 and 1.472
(`target/redesign2/graphs-callbacks-attack2/bench/runs/w4_a.txt`, `w4_b.txt`;
`phase3b-graphs-callbacks-attack.md` 6.4); the measured program is this one (phase 3's W4).

**8. Kind:** a smarter compile-time solution (where each count is needed is decided when compiling;
the `for` refusal is the agreed limitation).

---

#### AN-61: Unsound today: a handler list dispatched by index is accepted and writes freed memory

**1. Intent:** dispatch handlers by position so each may change the list.

**2. OLD.** `old/an61_fire_index.em` (last run's `a32_fire_index.em`):

```ember
    fn fire(self):
        i = 0
        while i < self.handlers.len():
            h = self.handlers.get_mut(i).unwrap()
            h(self)
            i += 1
```

and `old/an61_remove_self.em` (a handler removes itself through `b.remove_current()`):

```text
=== an61_fire_index.em
exit=0
=== an61_remove_self.em
exit=0
```

Both accepted. The C that today's compiler emits for them keeps `h`, a pointer into the list's buffer,
across the call, and its trampoline copies the closure's state back to that address after the call
(last run's reading of the emitted C, `probes/a32_emit_uaf/c/release/c/main.c` lines 131-137, 233, 251;
`phase3b-graphs-callbacks.md` section 10; confirmed by the second attack, its section 9). The same shape,
written in C and run: `c/an61_dispatch.c` mode 1 (a handler pushes 8 handlers, so the buffer moves):

```text
--- mode 1
=================================================================
==43052==ERROR: AddressSanitizer: heap-use-after-free on address 0x11e85f1a0108 at pc 0x7ff90170b4dc bp 0x003b3fbeec50 sp 0x003b3fbeec98
WRITE of size 24 at 0x11e85f1a0108 thread T0
    #0 0x7ff90170b4db in _asan_memcpy+0x3db (C:\Program Files\LLVM\lib\clang\22\lib\windows\clang_rt.asan_dynamic-x86_64.dll+0x18004b4db)
    #1 0x7ff7c24f2bc6 in main C:\Users\ism19\Code\ember\target\redesign3\11-area-D\c\an61_dispatch.c:111
```

(the write is the trampoline's copy back, inlined into `main`; `c/an61_dispatch_output.txt` has the
whole report, with where the block was freed: the growth in `push1`).

**3. NEW.** The position call of AN-60:

```ember
    fn fire(mut self):
        i = 0
        while i < len(self.handlers):
            self.handlers[i](self)
            i += 1
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `self` | `self` | yes (`mut self`) |
| `i` | none (default: `fire`'s own `int`) | its owner may |
| `self.handlers[i]` | none (default: the element is read, two words, and called; nothing points into the list during the call, D4) | no |

**5. Means exactly.** `c/an61_dispatch.c` mode 2, pasted under AN-60: clean under ASan.

**6. Still refused.**
- `h = self.handlers.get_mut(i).unwrap()` then `h(self)`: "`h` points into `self.handlers`, and
  `h(self)` lets the handler change `self.handlers` while `h` points into it; call it by position:
  `self.handlers[i](self)`". (`get_mut` returns a `ref mut` view, the vocabulary's `get_mut` row; a
  `ref mut` view into a place may not live across a call given a changeable name that reaches the same
  place: Rust's rule, E0499.)

**7. Cost.** As AN-60. The refused form had no cheaper code: the copy back it needs is what made it
unsafe.

**8. Kind:** a smarter compile-time solution (the hole is closed when compiling; the form that works
costs what C costs).

---

#### AN-62: A loop calling a stored callback with unknown effects pays a count pair per step (last run's limit)

**1. Intent:** walk from a node up to the root and report every ancestor to a stored callback.

**2. OLD.** Today's spelling (`old/an62_walk_callback.em`):

```ember
class Node:
    value: int = 0
    children: Array[Node] = []
    parent: Weak[Node] = Weak.empty()

class Tree:
    nodes: Array[Node] = []
    on_visit: fn(Node) = fn(n: Node): pass

    fn depth_total(self) -> int:
        depth = 0
        for n in self.nodes:
            a = n.parent.upgrade()
            while a is not None:
                p = a.unwrap()
                self.on_visit(p)
                depth += 1
                a = p.parent.upgrade()
        return depth
```

```text
warning[L3001]: potential reference cycle: Node.children -> Node
 --> an62_walk_callback.em:4:5
  |
4 |     children: Array[Node] = []
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ this strong field closes the shortest statically visible cycle
  |
  = help: if `Node.children` is a back-reference, make it hold a `Weak` handle
  = note: every edge in this statically visible cycle is strong; the cycle may leak at run time

exit=0
```

Accepted; every `upgrade()` is a count pair (today's `Weak`, `[WK-*]`). In last run's model the same
walk paid a count pair per step because the callback's effects were unknown: 1.27x to 1.32x of the
same walk without it (`phase3b-graphs-callbacks-attack.md` 3.5, `runs/w2v_a.txt`, `w2v_b.txt`);
`FINDINGS-phase3b.md` section 5 item 6.

**3. NEW.**

```ember
class Node:
    value: int = 0
    parent: weak Node set by children
    children: Array[Node] = []

class Tree:
    root: Node
    leaves: Array[weak Node] = []
    on_visit: fn(Node)

    fn depth_total(self) changes on_visit -> int:
        depth = 0
        for leaf in self.leaves:
            if Some(n) = leaf:
                a = copied n.parent
                while Some(p) = a:
                    self.on_visit(p)          # unknown: may end `p` (Node is a `weak` target)
                    depth += 1
                    a = copied p.parent
        return depth
```

**19:** two changes, the C unchanged (`a` is one pointer either way, and the walk is inside the
deferred-drop region, so the copied link takes no weak count, R2, R10; **21:** (F-01) `a` is a local
of the loop's body, so its whole life is inside the region, as R2 now requires; a copy stored in a
field, such as `self.top = copied a`, would take its count: `21-fixer/c/f1_weak_copy_escape.c` mode 2): (R18-02) `depth_total` calls
the stored function value `on_visit`, which by R12 is a change of the field that holds it, so its first
line says `changes on_visit` (was `fn depth_total(self) -> int:`); (R18-13, R18-03) `a = copied
n.parent` and `a = copied p.parent` (were `a = n.parent`, `a = p.parent`): R2 no longer copies a
`weak` link bound from a part, and a view name is never pointed elsewhere, so the walk keeps a link of
its own in `a` and replaces it each step, which `=` does to an own name. The C is `c/an62_defer_asan.c` mode 2,
re-run by 19 under AddressSanitizer (`target/redesign3/19-fixer/c/an62_defer_asan_output.txt`): mode 1
(no protection) `heap-use-after-free` at line 89; mode 2 (the region, `a` a plain pointer, no weak
count) `mode2 depth=1 leaf.parent=None`, no report.

The program writes nothing for the protection. D5: `Node` is the target of `weak` links (`parent`,
`leaves`), so an unknown call may end a `Node`; `Node` has no `drop` method of its own, so the compiler
puts a **deferred-drop region** around the loop instead of a count pair per step: entering the loop adds
1 to a per-thread counter and leaving subtracts 1; a `Node` whose count reaches zero while the counter
is above zero is set aside whole (marked dead, so `weak` reads of it give `None`) and dropped when the
loop ends. `p` therefore stays readable after the call; if the callback took `p` out of the tree,
`p.parent` was cleared by `set by` and the walk stops there.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `value` | none (default: the node's own `int`) | `changes value`, `mut self` |
| `parent` | `weak` (does not keep its node alive), `set by children` (only `children` sets it) | no: the program never assigns it |
| `children` | none (default: the node's own list; each element has this one owner) | `changes children`, `mut self` |
| `root` | none (default: the tree's own Node) | `changes root`, `mut self` |
| `leaves` | `weak` on its elements: links that do not keep the leaves alive | `changes leaves`, `mut self` |
| `on_visit` | none (default: the tree's own function value) | `changes on_visit`, `mut self` |
| `self` | `self` | `changes on_visit` (**19:** was "no (read-only)"; R12: calling the stored function value may change it) |
| `depth` | none (default: the method's own `int`) | its owner may |
| `leaf` | none (default: a read-only view of an element of `leaves`, a `weak` link) | no |
| `n`, `p` | none (default: a read-only view of the node the link points at, kept alive until its last use, D6) | no |
| `a` | `copied`: the method's own copy of the link (**19:** was none (default: the method's own `Option` copy of a link, as `n.parent` gives it), R2) | its owner may: `=` replaces its value each step |

**5. Means exactly.** C, `c/an62_defer_asan.c`: a leaf, its parent `mid`, and a callback that takes
`mid` out of the tree and drops its last owner while the walk is on `mid`. Three modes, under ASan
(`c/an62_defer_asan_output.txt`):

```text
--- mode 1
=================================================================
==71988==ERROR: AddressSanitizer: heap-use-after-free on address 0x1220ea4a0918 at pc 0x7ff6579d3211 bp 0x004fe739fa00 sp 0x004fe739fa48
READ of size 8 at 0x1220ea4a0918 thread T0
    #0 0x7ff6579d3210 in main C:\Users\ism19\Code\ember\target\redesign3\11-area-D\c\an62_defer_asan.c:89
...
--- mode 2
mode2 depth=1 leaf.parent=None
an62_defer_asan ms=0.075 peak_kb=8268 checksum=1
--- mode 3
mode3 depth=1 leaf.parent=None
an62_defer_asan ms=0.060 peak_kb=8280 checksum=1
```

Mode 1 (no protection) reads `p.parent` from freed memory (line 89, `Node *next = a->parent;`); mode 2
(the region) and mode 3 (last run's count pair) are clean and give the same result.

The per-step code is the unprotected step's. The workload's walk (`c/an62_walk_bench.c`, 1,000,000
nodes, 10 passes, a call through a function pointer the C compiler cannot see on every step) built
`-DSLAB` and `-DSLAB -DDEFER`; the whole difference between the two assembly files (`c/bin/walk_noprot.s`,
`c/bin/walk_defer.s`), apart from the program's name string and the counter's declaration:

```text
205a206
> 	incq	g_defer(%rip)
233a235
> 	decq	g_defer(%rip)
```

one increment before each pass's loop and one decrement after it; nothing inside the loop.

**6. Still refused.** Nothing new is refused. What stays: a class with its own `drop` method keeps the
count pair (deferring it would move a visible action); a view into a `Node`'s contents (an element of
`p.children`) across the unknown call stays refused by the loop rule ("the callback may change
`p.children`, which this loop walks; ...", area A's message).

**7. Cost.** Timed with `target/redesign2/bench/run.ps1 -Rounds 5` (pinned to the fast cores, mains
checked each round; `c/walk_runs.txt`, medians in `c/walk_summary.txt`). Every run printed checksum
722575054.

| program | median ms | ratio to plain C, same layout | spread |
|---|---|---|---|
| plain C, same layout (`walk_plain.exe`, `-DSLAB -DARRAY24`) | 1905.5 | 1.000 | 30% |
| set by, no protection (`walk_noprot.exe`) | 2068.5 | 1.086 | 29% |
| set by, a count pair per step (`walk_retain.exe`, last run's rule) | 2444.0 | 1.283 | 34% |
| set by, deferred-drop region (`walk_defer.exe`) | 1797.9 | 0.944 | 37% |

**PROVISIONAL**: other agents of this phase were running; the spread (fastest to slowest) is 29% to 37%,
far over the 10% the timer accepts, so these numbers rank nothing finely. What they agree with is the
assembly above: the region's walk is the unprotected walk plus two instructions per pass, and the count
pair is again about 1.2x to 1.3x (last run: 1.27x to 1.32x). Peak memory: 84,544 KB for every `set by`
variant against 76,688 KB for plain C (the 8-byte header, `phase3b-combined.md` 4.4's choice; not this
answer's). A region holds the memory of the objects set aside until the loop ends: in this workload none.
Phase 4 measures the final form.

**8. Kind:** a new mechanism (clever: the protection moves from every step to the rare death; readable:
nothing is written; as fast as the unprotected walk inside the loop; as safe as the count pair, shown
under ASan).

---

#### AN-63: Trees with parent pointers, graphs, cycles (A8)

**1. Intent:** a scene tree whose nodes know their parent.

**2. OLD.** The Python spelling (`old/an63_tree_python.em`, last run's `a8_tree_python.em`):

```ember
class Node:
    name: String
    children: Array[Node] = []
    parent: Option[Node] = None

    fn add(self, child: Node):
        child.parent = Some(self)
        self.children.push(child)
```

```text
warning[L3001]: potential reference cycle: Node.children -> Node
 --> an63_tree_python.em:4:5
  |
4 |     children: Array[Node] = []
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ this strong field closes the shortest statically visible cycle
  |
  = help: if `Node.children` is a back-reference, make it hold a `Weak` handle
  = note: every edge in this statically visible cycle is strong; the cycle may leak at run time

exit=0
```

Accepted; every parent-child pair is a cycle of counted handles that leaks (`[WK-1]`). Today's working
spelling (`old/an63_tree_weak.em`: `parent: Weak[Node] = Weak.empty()`, `child.parent = Weak(self)`,
`match n.parent.upgrade():`) is accepted with the same warning, pasted in `old/check_output.txt`.

**3. NEW.**

```ember
class Node:
    name: String
    parent: weak Node set by children
    children: Array[Node] = []

fn path(n: Node) -> String:
    if Some(p) = n.parent:
        return f"{path(p)}/{n.name}"
    return copied n.name

fn main():
    root = Node(name="root")
    root.children.push(Node(name="a"))                   # a.parent is root
    root.children[0].children.push(Node(name="leaf"))    # leaf.parent is a
    println(path(root.children[0].children[0]))          # root/a/leaf
    moved = root.children[0].children.remove(0)          # leaf.parent is None
    root.children.push(moved)                            # leaf.parent is root
```

Moving a node is written: taken out of one list, pushed into another (planned-words 3.3). For links
that are not the reverse of an owning field (a camera's target, an observer list), `weak` alone:
`target: weak Player`, `observers: Array[weak Observer]`. For a graph, each city's roads and the roads
into it: `roads: Array[weak City]` and `into: Array[weak City] set by roads`, the cities owned by one
list (last run's rules 4.4, with D7).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `name` | none (default: the node's own String) | `changes name`, `mut self` |
| `parent` | `weak`: does not keep its node alive; `set by children`: only `children` sets it | no: never assigned by the program |
| `children` | none (default: the node's own list; one owner per element) | `changes children`, `mut self` |
| `n` | none (default: a read-only view, a parameter) | no |
| `p` | none (default: a read-only view of the parent, kept alive until its last use) | no |
| `root` | none (default: `main`'s own Node) | its owner may |
| `Node(name="a")` given to `push` | none (default: a new value handed over by `push`, row 9's hand-over) | its new owner may |
| `moved` | none (default: `main`'s own Node, handed over by `remove`) | its owner may |
| `moved` given to `push` | none (default: handed over, row 9) | its new owner may |

`copied n.name` makes the returned String (row 9's word).

**5. Means exactly.** C, `c/an63_links.c` part 1, the same steps plus a root that ends while a child is
held elsewhere; every link store is one store, every read a plain load; under ASan
(`c/an63_links_output.txt`):

```text
1 root/a/leaf
2 leaf
3 root/leaf
4 leaf parent=None
5 100
6 gone
an63_links ms=0.068 peak_kb=8264 checksum=0
```

No ASan report. (Lines 5 and 6 are part 2, AN-65.)

**6. Still refused.**
- `child.parent = Some(self)` (the Python spelling, a strong field): "`child.parent = Some(self)` would
  make `parent` a second owner of this node, and `self` is not yours to hand over; a link back to the
  node whose `children` holds this one is written `parent: weak Node set by children`, and `push` sets
  it".
- `leaf.parent = root` with `set by`: "`parent` is set by `children`; to give `leaf` a parent, write
  `root.children.push(leaf)`" (planned-words 3.3).
- `parent: Node set by children` without `weak`: planned-words' message.
- Lending a `set by` holder as a changeable list (`shuffle(ref mut xs: Array[Node])` called with
  `p.children`): "`children` keeps every child's `parent` in step, so it is changed only by its own
  methods; pass it read-only, or write the change with `push`, `insert`, `remove`".
- A cycle of `shared` owners (`a: shared Node` holding `b` holding `a`) is not refused (it is a leak,
  not a memory error, as in Rust); AN-64's warning names it.

**7. Cost.** `set by`: reading is a plain load; an insert or remove writes the link (one store). Last run
measured the same mechanism on the tree workload, the link's own work, at 1.025 and 1.149 of plain C laid
out the same way, and safe Rust's index arena at 1.519 and 1.357
(`graphs-callbacks-attack2/bench/runs/w2_a.txt`, `w2_b.txt`); with a 4-byte header 0.964 and 1.024
(`phase3b-combined.md` 4.4). A graph with edges kept both ways: 0.969 and 1.081 of plain C keeping the
same reverse lists (`runs/graph_a.txt`, `graph_b.txt`). `weak`: one test of the target's header per read,
as Rust's `Weak`, which on churn costs 0.919 and 0.935 of plain C (`runs/weak_g.txt`, `weak_h.txt`).
Last run's push also searched the node's old list; here the program writes the `remove`, so no hidden
search.

**8. Kind:** a new mechanism (`set by`), with `weak` as a readable spelling of Rust's `Weak`.

---

#### AN-64: L3001 tells the programmer to make the owning edge weak

**1. Intent:** follow the warning to fix a tree's cycle.

**2. OLD.** `old/an63_tree_python.em`, `old/an63_tree_weak.em`, `old/an62_walk_callback.em`: all three
get the same advice, pasted under AN-63 and AN-62: `help: if \`Node.children\` is a back-reference, make
it hold a \`Weak\` handle`. `children` is the owning edge; making it weak would free every child. In the
second and third files `parent` is already `Weak`, and the warning is still given.

**3. NEW.** The NEW tree of AN-63 gets no warning: a list whose elements have one owner cannot form a
cycle (handing a node into its own subtree would hand over a value while a view into it lives, which is
refused when compiling). The warning remains only for a cycle through `shared` fields, and names the
link to make `weak`:

```ember
class Person:
    name: String
    friends: Array[shared Person] = []
    best: Option[shared Person] = None
```

```text
warning[L3001]: `Person.best` and `Person.friends` can hold each other: a cycle of `shared` owners is never freed
 --> app.em:4:5
  |
4 |     best: Option[shared Person] = None
  |     ^^^^ this link points at a person another field already owns
  |
  = help: a link that should not keep its person alive is `weak`: `best: weak Person`
```

The field named is the single-valued one when the cycle has one (a back link), otherwise the field the
cycle closes on; a parent-child pair in the Python spelling is the error of AN-63, which names
`parent: weak Node set by children`.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `name` | none (default: the person's own String) | `changes name`, `mut self` |
| `friends` | `shared` on its elements: one of several owners of each person | `changes friends`, `mut self` |
| `best` | `shared`: one of several owners of that person | `changes best`, `mut self` |

**5. Means exactly.** The NEW tree is AN-63's C (`c/an63_links.c`): no ASan report, and its line
`4 leaf parent=None` (pasted under AN-63) shows the root's end clearing its child's link before
releasing it. (ASan on Windows has no leak checker, so "every node is freed" is read from the code's
drops, not from a leak report.) The warning itself is a message; the program it is given for is a leak,
not something to translate.

**6. Still refused.** Nothing new; the Python spelling's error is AN-63's.

**7. Cost.** None (compile time).

**8. Kind:** a smarter compile-time solution (one-owner links are proved acyclic, so the warning is only
given where a cycle can exist, and it names the right link).

---

#### AN-65: `upgrade` is Rust's word for "the object, if it is still alive"

**1. Intent:** read a weak link's target.

**2. OLD.** `old/an65_upgrade.em`:

```ember
class Camera:
    target: Weak[Player] = Weak.empty()

fn main():
    player = Player()
    cam = Camera()
    cam.target = Weak(player)
    match cam.target.upgrade():
        Some(p) => println(p.health)
        None => println("gone")
```

```text
=== an65_upgrade.em
exit=0
```

**3. NEW.**

```ember
class Camera:
    target: weak Player = None

fn main():
    player = Player()
    cam = Camera()
    cam.target = weak player
    if Some(p) = cam.target:
        println(p.health)
    else:
        println("gone")
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `target` | `weak`: a link that does not keep the player alive | `changes target`, `mut self` (re-pointing) |
| `player` | none (default: `main`'s own Player) | its owner may |
| `cam` | none (default: `main`'s own Camera) | its owner may |
| `weak player` | `weak`: a link, not a hand-over; `player` stays `main`'s | no |
| `p` | none (default: a read-only view of the player, kept alive until its last use) | no |

**5. Means exactly.** `means/m65_weak_read.em` (`if Some(p) = cam.target.upgrade():`):

```text
=== m65_weak_read.em
exit=0
```

In C, `c/an63_links.c` part 2: a read is `(w && !w->dead) ? w : NULL`; lines `5 100` and `6 gone` of
its output (pasted under AN-63) are the read before and after the player ends, clean under ASan.

**6. Still refused.** `cam.target.upgrade()`: "a `weak` link is read directly: `if Some(p) =
cam.target:`"; `Weak[Player]`, `Weak(player)`: "write `weak Player`", "write `weak player`";
`cam.target.health`: "`cam.target` may be gone; write `if Some(p) = cam.target:`"; `cam.target =
player`: planned-words 3.2's message.

**7. Cost.** One test of the target's header per read; a count pair only where D5 needs one (Rust's
`upgrade` always makes one).

**8. Kind:** a readable spelling of Rust's guarantee.

---

#### AN-66: A bare interface type is a counted, shared, dynamically dispatched handle with no word

**1. Intent:** store shapes of several kinds and draw them.

**2. OLD.** Over class objects, a bare interface type is accepted and its second name shares the object
silently (`old/an66_bare_interface_class.em`):

```ember
fn main():
    a: Shape = Circle()
    b = a
    b.grow()
    println(a.size())
```

```text
=== an66_bare_interface_class.em
exit=0
```

Over value types it is refused, and one owner needs `Box[dyn Shape]` (`old/an66_bare_interface.em`):

```text
error[E2173]: `Shape` is an interface, and the types implementing it are values, not classes
  --> an66_bare_interface.em:11:19
   |
11 |     shapes: Array[Shape]
   |                   ^^^^^
   |
   = help: hold such a value behind a pointer: `Box[dyn Shape]` in a field or a collection (`Box(v)` makes one), `ref dyn Shape` for a parameter; or take a type parameter, `[T: Shape]`, and pass the value itself
   = note: an interface's name as a type is a class handle typed by it [OBJ-2]; a value of a type implementing `Shape` has its own size, which `dyn Shape` keeps behind a pointer (ODR-182)

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
struct Scene:
    shapes: Array[owned dyn Shape]          # the list is each shape's one owner
    shown: Array[shared dyn Shape]          # shapes also held elsewhere

fn draw(s: dyn Shape):                      # a read-only view of any shape
    ...

fn main():
    scene = Scene(shapes=[Circle(r=1.0), Square(side=2.0)], shown=[])
    for s in scene.shapes:
        draw(s)
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `shapes` | `owned` on its elements: each shape has this one owner | `changes shapes`, `mut self` |
| `shown` | `shared` on its elements: one of several owners | `changes shown`, `mut self` |
| `s` in `draw` | none (default: a read-only view, a parameter) | no |
| `scene` | none (default: `main`'s own Scene) | its owner may |
| `Circle(...)`, `Square(...)` in the list | none (default: new values handed over to the list, row 9) | their owner may |
| `s` in the loop | none (default: a read-only view of an element) | no |

**5. Means exactly.** `owned dyn Shape` is today's `Box[dyn Shape]` (`means/m66_owned_dyn.em`):

```text
=== m66_owned_dyn.em
exit=0
```

`shared dyn Shape` over class objects is today's bare `Shape` handle (`old/an66_bare_interface_class.em`,
`exit=0` above, with the sharing written). Over value types today has no spelling at all
(`means/m66_shared_dyn.em`, `a: Shared[dyn Shape] = Shared(Circle(r=1.0))`):

```text
error[E2020]: expected `dyn Shape`, found `Circle`
  --> m66_shared_dyn.em:11:35
   |
11 |     a: Shared[dyn Shape] = Shared(Circle(r=1.0))
   |                                   ^^^^^^^^^^^^^ this is `Circle`
   |
```

(two more errors follow in the file); in C it is a block `{count, Circle}` and a pair `{pointer, method
table}`, Rust's `Rc<dyn Shape>`.

**6. Still refused.** `shapes: Array[Shape]`: "say who owns the shapes: `Array[owned dyn Shape]` (the
list is their only owner) or `Array[shared dyn Shape]` (other names own them too)". `b = a` for an
`owned dyn Shape`: hands it over (row 9); for a second owner, `b = shared a`. `Box[dyn Shape]`: "write
`owned dyn Shape`".

**7. Cost.** `owned dyn`: one heap block per value and a method-table pointer beside it (Rust's
`Box<dyn>`); `shared dyn`: the same plus a count (Rust's `Rc<dyn>`); a call is one indirect call.

**8. Kind:** a readable spelling of Rust's guarantee.

---

#### AN-67: Trait objects and downcasting (A13)

**1. Intent:** `get_mut[T]` finds the pool of `T` in the registry and returns element `i`.

**2. OLD.** `old/an67_box_cast.em` (last run's `a13_box_cast.em`, phase 1's registry):

```ember
struct Reg:
    pools: Map[u128, Box[dyn AnyPool]]

    fn get_mut[T](mut self, i: int) -> ref mut T:
        pool = cast self.pools[7u128] as Pool[T]
        return ref mut pool.dense[i]
```

```text
error[E2020]: `cast … as` and `cast … if` need related class handles, found `Box[dyn AnyPool]` and `Pool[T]`
  --> an67_box_cast.em:18:16
   |
18 |         pool = cast self.pools[7u128] as Pool[T]
   |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |

error: could not compile due to 1 error(s)
exit=1
```

A generic method through `dyn` (`old/an67_dyn_generic.em`, `fn inspect[V: Visitor](self, mut v: V)`):

```text
error[E2050]: interface `Inspect` is not `dyn`-compatible
  --> an67_dyn_generic.em:18:23
   |
18 | fn walk(xs: Array[Box[dyn Inspect]]) -> int:
   |                       ^^^^^^^^^^^ method `inspect` is generic
   |

error: could not compile due to 1 error(s)
exit=1
```

The spec's `as?` (`old/an67_cast_forms.em`, 05's probe):

```text
error[E0100]: `as?` is not Ember: a downcast is written `cast x if D`
  --> an67_cast_forms.em:10:13
   |
10 |     a = s as? Circle
   |             ^ write `cast` before the value and `if` here
   |
   = help: `cast x as D` gives the `D` and panics if `x` is not one; `cast x if D` gives `Some(d)` or `None`

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
struct Reg:
    pools: Map[u128, owned dyn AnyPool]

    fn get_mut[T](self, k: u128, i: int) changes pools -> ref mut T:
        if Some(ref mut pool) = cast self.pools[k] if Pool[T]:
            return ref mut pool.dense[i]
        panic("no pool of this type at this key")

interface Inspect:
    fn inspect[V: Visitor](self, ref mut v: V)

fn walk(xs: Array[owned dyn Inspect]) -> int:
    s = Sum(total=0)
    for x in xs:
        x.inspect(s)                # through `dyn`: the entry made with V = dyn Visitor
    return s.total
```

`cast <place> if D` on an `owned dyn` (or `shared dyn`) place tests the value's type and gives a view of
it as a `D`: `Some(ref mut pool)` binds a changeable view because the place is changeable (`changes pools`; **14:** was `mut self`, clash 4);
`Some(pool)` a read-only one. A generic method whose type parameter appears only as the type of a `ref`
or `ref mut` parameter, with a `dyn`-compatible bound, gets one method-table entry, the generic body
compiled with `V = dyn Visitor` (last run's answer, saved by its attack, 7.1: checked per
implementation). The spec's `as?`/`as!` text becomes `cast x if D` / `cast x as D` (the compiler's,
ODR-280).

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `pools` | `owned` on its values: each pool has the map as its one owner | `changes pools`, `mut self` |
| `self` in `get_mut` | `self` | `changes pools` |
| `k`, `i` | none (default: read-only numbers) | no |
| `pool` | `ref mut` on the pattern name: a view of the registry's pool that may change it (row 8's words) | yes |
| result `-> ref mut T` | `ref mut`: a view into the registry's element | yes |
| `v` | `ref mut`: the caller's visitor | yes |
| `xs` | `owned` on its elements (one owner, the list); the parameter itself none (a read-only view) | no |
| `s` | none (default: `walk`'s own Sum) | its owner may (lent `ref mut` to `inspect`) |
| `x` | none (default: a read-only view of an element) | no |

**5. Means exactly.** The cast, with the pools as class objects (today's `cast` needs class handles),
`means/m67_class_pools.em`:

```ember
class Reg:
    pools: Map[int, AnyPool] = Map[int, AnyPool]()

    fn bump(self, k: int, i: int):
        match cast self.pools[k] if Pool[int]:
            Some(p):
                p.dense[i] += 1
            None:
                pass
```

and the `dyn` entry, written by hand as Rust's workaround is, `means/m67_dyn_param.em`
(`fn inspect(self, v: ref mut dyn Visitor)`, `x.inspect(ref mut s)`):

```text
=== m67_class_pools.em
exit=0
=== m67_dyn_param.em
exit=0
```

**6. Still refused.**
- A generic method whose type parameter is returned or stored by value (`fn make[V: Visitor](self) ->
  V`) through `dyn`: "`Inspect` cannot be used as `dyn Inspect`: `make` returns its `V`, and a `dyn` value
  has one entry per method; take the visitor as `ref mut v: V`, or use a type parameter
  `[I: Inspect]` instead of `dyn Inspect`" (E2050 with the line).
- `cast x as D` / `cast x if D` between unrelated types (E2020), with "`Box[dyn AnyPool]` is now
  `owned dyn AnyPool`" where that is the cause.
- `as?` / `as!`: today's message (it already gives the line).

**7. Cost.** A `cast`: one load of the method table's type id and one compare (last run's references
attack fix, `phase3b-combined.md` 3.4); safe Rust's `downcast_mut` was measured slower (1.710x and 1.606x
against the pool table's 1.208x and 1.084x of plain C, `phase3b-references.md` 3.4 as quoted in
`phase3b-graphs-callbacks.md` 6.3). The `dyn` entry: the indirect calls Rust's hand-written workaround
makes.

**8. Kind:** a smarter compile-time solution.

---

#### AN-68: `if c = cast s if Circle:` gets E2036 with a help that is not what was meant

**1. Intent:** do something only when a shape is a circle.

**2. OLD.** `old/an68_if_cast.em`:

```ember
    for s in shapes:
        if c = cast s if Circle:
            t += c.r
```

```text
error[E2036]: this pattern always matches
  --> an68_if_cast.em:12:12
   |
12 |         if c = cast s if Circle:
   |            ^^^^^^^^^^^^^^^^^^^^
   |
   = help: did you mean `c == cast s if Circle`?
   = note: to declare `c`, write `c = cast s if Circle` on the line before

error[E2020]: `Option[Circle]` has no field `r`
  --> an68_if_cast.em:13:18
   |
13 |             t += c.r
   |                  ^^^
   |

error: could not compile due to 2 error(s)
exit=1
```

**3. NEW.**

```ember
    for s in shapes:
        if Some(c) = cast s if Circle:
            t += c.r
```

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `s` | none (default: a read-only view of an element) | no |
| `c` | none (default: a read-only view of the same object as a `Circle`) | no |
| `t` | none (default: the function's own `f64`) | its owner may |

**5. Means exactly.** This is today's Ember (`means/m68_if_some.em`):

```text
=== m68_if_some.em
exit=0
```

**6. Still refused.** `if c = cast s if Circle:`, with the message:
"`c = ...` names `c` but tests nothing; to run the block only when `s` is a `Circle`, write
`if Some(c) = cast s if Circle:`".

**7. Cost.** None (one type-id compare, as AN-67).

**8. Kind:** an understandable limitation kept (`=` in an `if` binds only through a pattern), with the
message giving the line that works.

---

#### AN-69: Generics friction (A16)

**1. Intent:** sum any addable values; print a list of vectors; give one type its own description.

**2. OLD.** `old/an69_bounds.em` (no bounds written):

```text
error[E2040]: `T`'s bounds do not provide a matching `Add` operator
 --> an69_bounds.em:5:15
  |
5 |         acc = acc + x
  |               ^^^^^^^
  |
  = help: add a `Add` bound with the required operand and output types
  = note: inside a generic body only the bounds' operations are available [TYP-17]

error: could not compile due to 1 error(s)
exit=1
```

`old/an69_orphan.em` (a one-file program implements `Neg` for `Array[Vec3]`):

```text
error[E2121]: `Array[Vec3]` and `Neg` are both another package's, so this package cannot implement one for the other
 --> an69_orphan.em:5:31
  |
5 | extend Array[Vec3] implements Neg:
  |                               ^^^
  |
  = help: implement it where `Neg` or `Array[Vec3]` is declared, or wrap the value in a type of this package
  = note: an implementation lives with its interface or its type, or with a type of its package among the interface's arguments [TYP-20]
```

(followed by `error[E2020]: \`-\` cannot be applied to \`Array[Vec3]\``.) `old/an69_overlap.em`:

```text
error[E2041]: `i32` already implements `Describe`
 --> an69_overlap.em:9:23
  |
5 | extend[T: Debug] T implements Describe:
  | --------------------------------------- every type implements it here
...
9 | extend i32 implements Describe:
  |                       ^^^^^^^^
  |
  = note: two implementations of one interface may not apply to one type [TYP-19]; the prelude converts every type from itself [ERR-7]

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW.**

```ember
fn sum[T](xs: Span[T]) -> T:                 # not `pub`: its bound is worked out from the body
    acc = copied xs[0]
    for x in xs[1..]:
        acc = acc + x
    return acc

pub fn total[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:    # `pub`: the bound is written
    ...

extend Array[Vec3] implements Neg:           # accepted in a program (a package nothing depends on)
    ...
```

Rules (last run's, with one change): a function that is not `pub` gets its bounds from its body (every
operation on `T` maps to the one interface in scope that provides it; recorded, printed by `ember inspect
--bounds`); a `pub` function writes its bounds, because they are its promise to other packages, and the
error gives the exact line; a package that nothing can depend on (a program or a test) may implement any
interface for any type (two implementations of one interface for one type anywhere in the program stay
refused). The change from last run: inferred bounds stop at `pub`, so nothing another package relies on
is decided by the compiler from a body it cannot see.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `xs` | none (default: a read-only view, a parameter) | no |
| `acc` | `copied`: the function's own copy of the first element | its owner may |
| `x` | none (default: a read-only view of an element) | no |

**5. Means exactly.** The bound the compiler infers for `sum`, written out (`means/m69_bounds_written.em`,
`fn sum[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:`):

```text
=== m69_bounds_written.em
exit=0
```

The orphan exemption is a permission, not code; the overlap stays refused, so it needs no translation.

**6. Still refused.**
- A `pub` generic with an operation its bounds lack: "`total` is `pub`, so its bound is written: write
  `pub fn total[T: Add[T, Output = T] + Copy](xs: Span[T]) -> T:`" (E2040 with the line; a non-`pub`
  function never gets it).
- E2121 in a library package: "`Array[Vec3]` and `Neg` both belong to other packages, and packages
  that depend on this one could see a second `Neg` for `Array[Vec3]`; wrap it: `struct Vecs(Array[Vec3])`
  and `extend Vecs implements Neg:`".
- E2041, overlapping implementations (the most specific one winning is unproven for Ember, last run's
  open item): "`i32` already gets `Describe` from `extend[T: Debug] T implements Describe` on line 5,
  and two implementations may not apply to one type; narrow the general one's bound so it leaves `i32`
  out, or give `i32` its own function: `fn describe_int(x: i32) -> String:`".
- A non-`pub` body calling a method no interface in scope declares: "`T` has no `weight`: no interface
  in scope declares `fn weight`; declare one (`interface Weighed: fn weight(self) -> int`) and the
  bound follows" (last run's duck-typing extension is not taken: it would let a typo surface at a call
  far away).

**7. Cost.** None at run time: every instance compiles to the code written bounds give.

**8. Kind:** a smarter compile-time solution (bounds and the orphan exemption); the overlap is an
understandable limitation kept, with its message.

---

#### AN-70: `virtual` in a class that cannot be subclassed is accepted with a warning (W2111)

**1. Intent:** declare a method that subclasses may replace.

**2. OLD.** `old/an70_virtual.em` (reader 4's `t10_w2111.em`):

```ember
class Door:
    angle: float = 0.0
    virtual fn on_update(self, dt: float):
        pass
```

```text
warning[W2111]: `virtual` has no effect in a final class
 --> an70_virtual.em:3:16
  |
3 |     virtual fn on_update(self, dt: float):
  |                ^^^^^^^^^
  |

exit=0
```

**3. NEW.**

```ember
open class Door:
    angle: float = 0.0
    virtual fn on_update(mut self, dt: float):
        pass

class SwingDoor(Door):
    override fn on_update(mut self, dt: float):
        self.angle = min(self.angle + 90.0 * dt, 90.0)
```

**19:** (R18-11) the virtual is `mut self` (was plain `self`). By 15's rule in R5 an override says no
more than the virtual's first line, so with plain `self` no subclass could change the door, which the
intent ("a method that subclasses may replace") and the spec's own `Door` (spec 1704: the override sets
`self.angle`) need. `SwingDoor` is added to show the override the first line now allows.

**4. The owner's test.**

| name | what it is | may it change it |
|---|---|---|
| `angle` | none (default: the door's own `float`) | `changes angle`, `mut self` |
| `self` (both methods) | `self` | `mut self`: any field of the door (**19:** was "no (read-only)") |
| `dt` | none (default: a read-only number) | no |

**5. Means exactly.** `means/m70_open_class.em`:

```text
=== m70_open_class.em
exit=0
```

**19:** and the override, `target/redesign3/19-fixer/m70_virtual_changes.em` (today a class method's
plain `self` may write fields, `[CLS-7]`, which is what the NEW `mut self` says; `SwingDoor(Door)` with
`override fn on_update(self, dt: float): self.angle = min(self.angle + 90.0 * dt, 90.0)`):

```text
===== m70_virtual_changes.em
exit=0
```

**6. Still refused.** `virtual` in a class that is not `open` or `abstract`, now an error: "`Door`
cannot be subclassed, so `virtual` on `on_update` changes nothing; to let subclasses replace it write
`open class Door:`, or remove `virtual`" (vocabulary row `virtual`).
**19:** and, by R5, an override that changes the door under a virtual with plain `self`:
"`SwingDoor.on_update` changes `angle`, but it overrides `Door.on_update`, whose first line changes
nothing, and callers of `Door` rely on that; write `virtual fn on_update(mut self, dt: float):` in
`Door`".

**7. Cost.** None.

**8. Kind:** an understandable limitation kept (only an `open` class has subclasses), now refused with
the line to write instead of accepted with a warning.

## 14. Area E: threads, locks across threads, `unsafe` (AN-71 .. AN-84)

Copied from `RUN3/12-area-E.md` by `build.py`; the combiner's changes are marked **14:** and listed in section 4. File names in this area's text are relative to `target/redesign3/12-area-E/` unless they say otherwise; "section N" inside it means the area report's own section N.

### Terms of area E

- **thread**: a sequence of instructions the operating system runs alongside others, sharing memory.
  **task**: a function handed to another thread. **scope**: the block `with scope = thread.scope():`;
  its tasks may use the parent's variables because the block does not end until every task has
  finished (the **join**). An **unscoped thread** (`thread.spawn(...)`) may outlive the function that
  started it.
- **data race**: two threads touch one place in memory at the same time, at least one writes, and
  nothing orders them. In C and Rust it is undefined behaviour; in practice it loses updates or frees
  memory still in use.
- **lock**: an object at most one thread holds at a time; `with self.lock:` waits until it is free,
  holds it for the block, and lets it go at the block's end.
- **atomic**: an operation two threads cannot interleave (on this processor a `lock`-prefixed
  instruction), several times dearer than a plain one.
- **count**: the number of owners of a value that has several (`shared`, `thread_shared`); the value
  is freed when it reaches zero. A **one-thread count** changes with a plain add, an **atomic count**
  with an atomic add.
- **effect summary**: what the compiler records about a function from its body: which places it reads,
  changes and **reshapes** (changes in a way that can free or move memory: replace, `push`, `clear`).
  Ember already computes effect sets (`[EFF-1]`) and stores them in package interfaces (`[EFF-4]`).
- **view**: a value that points into something else and may not outlive it (`ref T`, `Span[T]`,
  `StrView`).
- **reduction**: per-task results combined after the join.
- **trampoline**: the small C function Ember generates for a C library to call; it finds the Ember
  function and calls it. **user data**: the `void*` a C library hands back to its callback.
- **the owner's test** (brief): for every name in a line, the word that says what it is (the owner's
  own thing, a copy, or handed over) and the word that says whether it may change it. In the tables, a
  name counts as **without a word** when neither column has a word and both say "none (default: ...)".
- **kind**: the job's four outcomes: a readable spelling of Rust's guarantee; a smarter compile-time
  solution; a new mechanism; an understandable limitation kept.

### Rules of area E (items 1 .. 9)

What changes from last run, in one sentence: last run decided everything from effect summaries and
had programmers write no thread words at all; this run keeps every compile-time check last run and
its attack saved, and adds the words the owner asks for wherever the program's meaning would
otherwise be decided by the compiler: what a thread holds (the capture list, row 6), which lock guards
which field, and which module variable a function changes.

1. **Scoped tasks need no words** (last run's scope rule, kept with the attack's fixes). Inside
   `with scope = thread.scope():`, every task's effect summary is checked against every other task's
   and against the parent's lines until the join. A name a task uses is the real variable (row 6),
   which lives until the join; a view (`for t in texts`) is fine; a local of one loop round is not,
   and gets row 6's `with copied n`. Two tasks that change one place are refused, except:
   - **reductions**: `total += e`, `-=`, `*=`, `|=`, `&=`, `^=`, `max`/`min` on a number, `push`,
     `extend`, `add` on a list or set, and `m[k] op= e` on a map or array, written in the task's own
     body, compile to one private accumulator per task, combined at the join in spawn order (last run
     4.4; the attack's element reductions, 3.2). No task and no parent line may read the place before
     the join, so no line of the program can tell this from every task changing the real variable;
   - changes through `Atomic`, `Mutex`, `RwLock`, a channel, or a field `guarded by` a lock while
     holding it (3 below);
   - each task's own element: `for ref mut p in players:` hands out one element per task.
2. **What an unscoped thread or a kept C callback holds is written in its capture list** (row 6's
   words, 07's capture positions): `with owned names` (handed over; the caller's `names` ends),
   `with thread_shared log` (one more owner, on any thread; atomic count), `with copied n` (its own
   copy), `with shared game` / `with weak game` (only where the function stays on this thread, which a
   C contract `threads(creator)` or `threads(main)` guarantees). A name not in the list is the real
   variable, which an unscoped thread may outlive: refused, naming the words.
3. **A lock guards the fields its class names** (new words, section 7): `words: int guarded by lock`.
   Every read or change of `words` must sit inside `with x.lock:` on the same object, on the thread
   that holds it (the attack's "held is per thread": a task starts holding no lock). Fields not
   written after `init` need no lock. This replaces last run's inference of which lock guards which
   field, and the `@sync` rule that froze every field.
4. **`thread_shared` says a value has owners on several threads** (07's word). Its count is atomic
   because the word says so; no owner changes it directly, only through `Atomic`, `Mutex`, `RwLock`, a
   channel, or a `guarded by` field under its lock. `@sync` (06's `@thread_shared`) is removed: what it
   said on the class is now said on the value at the place it is shared. The property `ThreadShared`
   (06) stays for bounds and messages, inferred as Rust infers `Send` and `Sync`.
5. **What may go to another thread is inferred from the words**, not written as `Send`: a value
   handed over (`owned`) may go when nothing reachable from it is `shared` (a one-thread count), a
   `weak` link into or out of a one-thread value, or **thread-bound** (the attack's fact: a lock's
   guard, a `Scope`, a raw pointer, a C handle whose contract says `threads(creator)`). The `unsafe`
   author of a raw-pointer wrapper lifts it with today's `unsafe extend Window implements Send:`.
6. **Module variables**: `static mut FRAME: int = 0`, changed only by functions whose first line says
   `changes FRAME` (row 3's word on a free function, 07). Used only by the main thread and its scoped
   tasks; for other threads, `static mut RESULTS: Array[int] guarded by RESULTS_LOCK`.
7. **Generators walk by position** across `yield` (the attack's 6.1, 6.2); `async` stays reserved.
8. **C callbacks** go through the checked trampoline (last run 8.3 with the attack's fixes and tag).
9. **`unsafe`**: one word; on a declaration it says "callers must vouch" (with `@safety`), on a block
   "this code vouches" (with `# SAFETY:`); inside an `unsafe fn` body every unsafe operation sits in
   its own `unsafe:` block (Rust 2024's rule, made an error).

What runs while the program runs, compared with C: nothing for scoped tasks and reductions (plain C's
per-thread counter); a lock where the program takes one; an atomic count change where a
`thread_shared` owner is made or ends (Rust's `Arc`); netted counts where scoped tasks re-point
`shared` fields of parent objects (the attack, 1.05x); the checked trampoline per C callback (1.02x
to 1.05x). Each answer below gives its own numbers and their sources.

### Area E's safety argument (its section 3)

A data race needs two threads, one place, a write, and nothing ordering them. Every place two threads
can reach is one of these, and each is ordered or never written:
1. a place a scoped task reads: the parent and the other tasks may not change it until the join
   (scope rule; summaries over-approximate unknown code as "every effect");
2. a place tasks change by a reduction: each task has its own accumulator; the join orders them;
3. an `Atomic`, `Mutex`, `RwLock`, channel end, or a `guarded by` field: ordered by the atomic or the
   lock, held by the thread that touches it (a task starts holding none);
4. a value handed over (`owned`): the giver's name has ended, and nothing reachable from it is a
   one-thread owner, a `weak` link of a one-thread value, or thread-bound (E7015, E7008);
5. a `thread_shared` value: never changed directly by any owner (E7014), its count is atomic;
6. a `static mut`: reached only by the main thread and its scoped tasks (E7012), or `guarded by`.

Freed while in use: a scoped task's views die at the join, which is before the block ends; an
unscoped thread holds only what its capture list names (an owner, a copy, or a `weak` link that is
tested on read); one-thread counts never cross threads (E7015); where scoped tasks re-point `shared`
fields of parent objects, count changes are netted per task and decreases wait until every task has
finished its loop (the attack, 3.1, kept); a C callback holds a checked slot, not an address.

### Area E's rulings (its section 5)

- **Whether `@thread_shared` stays once `thread_shared` exists:** no; removed (AN-80).
- **Whether a class object held `shared` uses a one-thread count:** yes; the count follows the word, for
  every type. A `shared` value never reaches another thread (E7015); a value created `shared` cannot
  later be made `thread_shared` while other one-thread owners exist ("create it `thread_shared` from the
  start: `names = thread_shared Names()`").
- **Safe `static mut` with `changes`:** yes, under AN-76's rules.
- **`@cached` across threads:** on an object several threads reach (a `thread_shared` value, or a
  scope's tasks), the slot is filled under a once-flag (`[STA-3]`'s `Once`: an acquire load and a branch
  when filled). A `@cached` method whose `reads` clause names a field that changes while other threads
  hold the object (`guarded by` or `Atomic`) is refused: "`radius` reads `verts`, which other threads
  change under `lock`; a kept result could be stale; remove `@cached`, or keep the result in a field
  `guarded by lock`".
- **The `unsafe fn` body rule:** every unsafe operation in its own `unsafe:` block (AN-79).

### The answers of area E

#### AN-71: threads (A15): the loop variable, `total +=` from tasks, E7004 offering `@sync`

**1. Intent:** count the words of several texts on several threads, and let a thread read a list of
names.

**2. OLD** (`old/a15_threads.em`, phase 1's program, unchanged):

```ember
# A15: count words in several texts on several threads.
import std.thread

fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total += t.split_whitespace().count())
    return total

class Names:
    items: Array[String] = []

fn share_plain():
    names = Names()
    h = thread.spawn(owned fn() => names.items.len())
    println(h.join())

fn main():
    println(count_words(["a b".to_string(), "c".to_string()]))
    share_plain()
```

```text
error[E3021]: `t` cannot be written while it is borrowed
 --> a15_threads.em:7:9
  |
7 |         for t in texts:
  |         ^^^^^^^^^^^^^^^ conflicting access here
8 |             scope.spawn(fn(): total += t.split_whitespace().count())
  |                         ------------------------------------------- borrow of `t` starts here
  |             -------------------------------------------------------- borrow later used here
  |
  = help: end the borrow before this: `scope` is what keeps it alive, so shorten its last use or put it in a block of its own
  = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error[E3022]: `total` is already mutably borrowed
 --> a15_threads.em:8:25
  |
8 |             scope.spawn(fn(): total += t.split_whitespace().count())
  |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting access here
  |                         ------------------------------------------- mutable borrow of `total` starts here
  |
  = help: use one mutable access rather than borrowing the same place twice; split the owner's mutable view with `split_at` when the intended operands are disjoint parts of it
  = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error[E3060]: `t` does not live long enough
 --> a15_threads.em:8:25
  |
8 |             scope.spawn(fn(): total += t.split_whitespace().count())
  |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this borrow remains live after its source's scope ends
  |             -------------------------------------------------------- `t` ends its scope here
  |
  = help: declare the borrowed value in an enclosing scope, or drop its borrower before leaving this scope
  = note: borrowed storage must remain valid through every use, including a destructor [DRP-7]

error[E7004]: the thread's function captures `names`, a `Names`, which is not Send
  --> a15_threads.em:16:22
   |
16 |     h = thread.spawn(owned fn() => names.items.len())
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: declare it `@sync class Names`: its fields are all `Send` and `Sync` [THR-1]
   = note: what a thread's function captures moves to that thread [THR-10]

error: could not compile due to 4 error(s)
exit=1
```

**3. NEW** (`new/an71_new.em`):

```ember
fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total += t.split_whitespace().count())
    return total

class Names:
    items: Array[String] = []

fn share_plain():
    names = Names()
    h = thread.spawn(fn() with owned names: names.items.len())
    println(h.join())

fn share_and_keep():
    names = thread_shared Names()
    h = thread.spawn(fn() with thread_shared names: names.items.len())
    println(names.items.len(), h.join())
```

`count_words` is the OLD text, unchanged: it now compiles. `share_plain` hands the object to the
thread; `share_and_keep` shares it and keeps reading it.

**19:** (R18-12) `share_and_keep` makes `names` `thread_shared` on the line that creates it (was
`names = Names()`, then the capture `with thread_shared names` on the next line made it one of
several owners and froze it, E7014, though its own line said "a new local, the function's own
object"). This is area E's own ruling ("create it `thread_shared` from the start: `names =
thread_shared Names()`", section 5) and AN-80's spelling. The capture word now makes one more owner of
a value whose line already says it has owners on several threads; a capture `with thread_shared x` of
a plain local is refused: "`names` is `share_and_keep`'s own object, which it may change; to share it
between threads, create it shared: `names = thread_shared Names()`" (**21:** (F-08) E7017,
`thread_shared` on a value not created so, R3; was E7015, whose meaning is a one-thread owner reaching
another thread; the same line before a value, `keep = thread_shared names`, gets the same message). The translation (`means/an71_means_shared.em`, an object shared between
threads from its creation) is unchanged.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `texts` (1) | none (default: a parameter reads the caller's list; vocabulary `ref`/`mut` rows) | none (default: read-only) |
| `total` (2) | none (default: a new local, the function's own) | none (default: the function's own variable; `+=` is the change, written) |
| `scope` (3) | `with` (bound for this block, ended at its end) | none (default: read-only binding) |
| `t` (4) | none (default: a read-only view of each element of `texts`; vocabulary `for` row) | none (default: read-only) |
| `total` (5, in the task) | none (default, row 6: a name not in the capture list is the real variable) | none (default, row 6: the real `total` changes; `+=` written) |
| `t` (5, in the task) | none (default: the same view of the element of `texts`, which lives until the join) | none (default: read-only) |
| `items` (9) | none (default: the object's own list) | none (default: changed only through `changes items` or `mut self`, row 3) |
| `names` (12) | none (default: a new local, the function's own object) | none (default: its own) |
| `h` (13) | none (default: the `JoinHandle` the call returns, the caller's own; 07's same default) | none |
| `names` (13, capture) | `owned` (handed to the thread; the caller's `names` ends here) | `owned` (the thread's own now) |
| `names` (17) | `thread_shared`: a new object with owners on any thread (**19:** was none (default: a new local, the function's own object)) | `thread_shared`: no owner changes it directly (R9) |
| `h` (18) | none (default as line 13) | none |
| `names` (18, capture) | `thread_shared` (one more owner, on any thread) | `thread_shared` (no owner changes it directly) |
| `names` (19) | `thread_shared` (line 17) (**19:** was "line 18's capture made the caller's name one of the owners") | `thread_shared` (line 17: read-only) |

14 names; 10 without a word, each with its default stated. No default changes anything the line does
not show, aliases silently, or copies silently: the per-task accumulator behind line 5 is a
compile-time translation no line can observe (see 5).

**5. Means exactly** (`means/an71_means.em`, today's Ember):

```ember
import std.thread
from std.sync import Atomic

fn count_words(texts: Array[String]) -> int:
    total = Atomic(0)
    tr = ref total
    with scope = thread.scope():
        for t in texts:
            scope.spawn(owned fn() => tr.fetch_add(t.split_whitespace().count()))
    return total.load()

struct Names:
    items: Array[String]

fn share_plain():
    names = Names(items=[])
    h = thread.spawn(owned fn() => names.items.len())
    println(h.join())

fn main():
    println(count_words(["a b".to_string(), "c".to_string()]))
    share_plain()
```

```text
exit=0
```

and `means/an71_means_shared.em` for `share_and_keep` (an object shared between threads, atomic
count, never changed after): `@sync class Names` with `h = thread.spawn(owned fn() =>
names.items.len())` followed by `println(names.items.len(), h.join())`: `exit=0`.

Meaning: every task adds its part to the real total; the total is read after the join. The compiled
NEW code does not use an atomic: each task keeps its own count and the parent adds them at the join, in
spawn order (`reduce-0` below). Nothing in the program can tell the two apart, because a task or a
parent line that reads `total` before the join is refused (6). One difference, stated: with integers
that check overflow (`[TYP-8]`), a task's partial sum may overflow where some order of single adds
would not; that is a panic, never a wrong value (last run 4.4). `share_plain`'s one-owner object is
written as a struct value moved to the thread, which is what a hand-over of a one-owner object is.

**6. Still refused** (codes in section 8):

- A task reads `total` before the join: `scope.spawn(fn(): if total > 10: println("many"))`:
  `error[E7007]: tasks add their parts to \`total\`, which is known only after the scope ends at line
  7; read \`total\` after the \`with\` block`.
- A task changes `total` in a way the compiler cannot add up per task (AN-72's message).
- A float place one task adds to more than once: `error[E7007]: each task adds to \`sum\` many times;
  float sums depend on how the adds are grouped, so the result would not be any order of single adds;
  add up in the task first: \`mine = 0.0\`, \`mine += x\` for each, then \`sum += mine\` once`. (With
  one add per task the spawn-order result is one order of single adds.)
- Using `names` after `with owned names`: `error[E3040]: \`names\` was handed to the thread at line 13
  (\`with owned names\`); to keep using it here, create it shared, \`names = thread_shared Names()\`,
  and capture it \`with thread_shared names\`` (**21:** (F-08) was "write `with thread_shared names`"
  alone, which E7017 refuses on this plain `names`).
- An unscoped thread using a name with no capture word: `thread.spawn(fn() => names.items.len())`:
  `error[E7004]: this thread may outlive \`share_plain\`, so it cannot use \`share_plain\`'s
  \`names\`; say what the thread holds: \`fn() with owned names:\` (handed over), or create it shared,
  \`names = thread_shared Names()\`, and write \`fn() with thread_shared names:\` (one more owner)`. It
  never offers `@sync`. (**21:** (F-08) the second line to write now creates `names` shared first.)
- Handing over a value that holds a one-thread owner: if `Names` had `cache: shared Cache` and the
  caller another owner of it, `error[E7015]: \`names\` holds \`cache\`, which has other owners on this
  thread (line 9), whose counts change without atomics; share it between threads:
  \`cache: thread_shared Cache\``. The same for a class that is the target of a `weak` field anywhere
  in the build (its weak links stay on this thread): "share it instead: create it
  `names = thread_shared Names()` and capture it `with thread_shared names`" (**21:** F-08).
- Changing a `thread_shared` value directly: `names.items.push(s)` in `share_and_keep`: `error[E7014]:
  \`names\` is shared between threads (line 17), so no owner changes it directly; keep the list in
  a field \`guarded by\` a lock and change it inside \`with names.lock:\``.
- A thread-bound value sent to a task (a guard, a `Scope`, a raw pointer; the attack's `x05`, `x08`,
  `x09`): `error[E7008]: \`g\` is the guard of \`m\`'s lock, which belongs to the thread that took it;
  take the lock inside the task: \`with g = m.lock():\``.

**7. Cost.** Scoped tasks and reductions: nothing beyond plain C's per-thread counter. Last run,
`phase3b-threads.md:381-386` (64 MiB of text, 8 tasks; PROVISIONAL): `reduce-0`, plain C with a local
counter per thread summed after the join, 43.548 ms (1.00), which the reduction compiles to; an atomic
add per word (today's `Atomic` spelling) 10.22x; safe Rust's `AtomicU64` 10.35x, its join-sum 1.08x.
`with owned names`: a pointer handed over, no count. `with thread_shared names`: one atomic increment
when the thread's owner is made and one atomic decrement when it ends, Rust's `Arc` price. Because the
word says which values are thread-shared, the count kind is known when compiling: last run's run-time
"shared flag" (and its walk, 5.451 ms per 262,144 objects, `phase3b-threads.md:538`) is not needed.

**8. Kind:** a smarter compile-time solution (the scope rule and reductions), with a readable spelling
of Rust's `move` + `Arc` for unscoped threads (the capture words).

---

#### AN-72: errors land on the wrong variable in `a15_threads.em`

**1. Intent:** read the error and fix the right line.

**2. OLD.** In `a15_threads.em` (AN-71's OLD, output pasted there) the errors on `t` (E3021, E3060)
are caused by the write to `total`. Taking the write out leaves no error (`old/an72_read_only.em`,
the same loop with `scope.spawn(fn(): println(t.split_whitespace().count()))`):

```text
exit=0
```

Taking the read of `t` out leaves only the error on `total` (`old/an72_write_only.em`, the task is
`fn(): total += 1`):

```text
warning[L1001]: `t` is never read
 --> an72_write_only.em:7:13
  |
7 |         for t in texts:
  |             ^ declared here and never used
  |
  = help: remove it, or rename it `_t` if that is deliberate

error[E3022]: `total` is already mutably borrowed
 --> an72_write_only.em:8:25
  |
8 |             scope.spawn(fn(): total += 1)
  |                         ^^^^^^^^^^^^^^^^ conflicting access here
  |                         ---------------- mutable borrow of `total` starts here
  |
  = help: use one mutable access rather than borrowing the same place twice; split the owner's mutable view with `split_at` when the intended operands are disjoint parts of it
  = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW** (`new/an72_new.em`): a program that stays refused, and the one error it gets:

```ember
fn count_words(texts: Array[String]) -> int:
    total = 0
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): total = total * 2 + t.len())
    return total
```

```text
error[E7007]: two tasks change `total`, and `total = total * 2 + t.len()` uses its old value
  --> count.em:5:31
   |
 5 |             scope.spawn(fn(): total = total * 2 + t.len())
   |                               ^^^^^ every task of this scope runs this line at the same time as the others
   |
   = help: to add each task's part, write `total += t.len()`; to combine the parts another way, have each task keep its own (`parts.push(t.len())`) and combine `parts` after the `with` block
```

The rule: a task's conflict is reported once, on the captured name that causes it; errors on other
names of the same function that only follow from it are not shown, and no error ever names a name the
program uses correctly (`t` here).

**4. The owner's test:** `texts`, `total` (2), `t` (4), `total` (5), `t` (5): none (defaults as in
AN-71's table, the same lines); `scope`: `with`. 6 names, 5 without a word.

**5. Means exactly.** The program is refused; the line its help gives (`total += t.len()`) is AN-71's
program, whose translation is `means/an71_means.em` (`exit=0`, pasted in AN-71).

**6. Still refused:** the NEW program itself, with the message above.

**7. Cost:** none; a compile-time message.

**8. Kind:** a smarter compile-time solution (the error found at its cause).

---

#### AN-73: "cannot find `n`" (E1010) inside a lambda that calls an `Atomic` method

**1. Intent:** each task adds its text's word count to one `Atomic` total.

**2. OLD** (`old/an73_atomic_in_lambda.em`):

```ember
import std.thread
from std.sync import Atomic

fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(fn() => total.fetch_add(n))
    return total.load()
```

```text
error[E1010]: cannot find `n` in this scope
  --> an73_atomic_in_lambda.em:10:49
   |
10 |             scope.spawn(fn() => total.fetch_add(n))
   |                                                 ^
   |

error: could not compile due to 1 error(s)
exit=1
```

Two problems sit behind the one message: a name-lookup defect (the same lambda written `owned fn`
finds `n`: `means/an73_means.em`, below, `exit=0`), and a real one the message hides: `n` is a local
of one round of the loop, which ends while the task may still run. With the call moved into a helper
(`old/an73_helper_round_local.em`: `fn add(total: Atomic[int], n: int)` and
`scope.spawn(fn() => add(total, n))`) today's compiler finds `n` and then says exactly that, though its
help does not give the line to write:

```text
error[E3021]: `n` cannot be written while it is borrowed
  --> an73_helper_round_local.em:13:17
   |
13 |             n = t.split_whitespace().count()
   |                 ^ conflicting access here
14 |             scope.spawn(fn() => add(total, n))
   |                         --------------------- borrow of `n` starts here
   |             ---------------------------------- borrow later used here
   |
   = help: end the borrow before this: `scope` is what keeps it alive, so shorten its last use or put it in a block of its own
   = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error[E3060]: `n` does not live long enough
  --> an73_helper_round_local.em:14:25
   |
14 |             scope.spawn(fn() => add(total, n))
   |                         ^^^^^^^^^^^^^^^^^^^^^ this borrow remains live after its source's scope ends
   |             ---------------------------------- `n` ends its scope here
   |
   = help: declare the borrowed value in an enclosing scope, or drop its borrower before leaving this scope
   = note: borrowed storage must remain valid through every use, including a destructor [DRP-7]

error: could not compile due to 2 error(s)
exit=1
```

**3. NEW** (`new/an73_new.em`):

```ember
fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(fn() with copied n: total.fetch_add(n))
    return total.load()
```

The defect is fixed (a name used as an argument of a method call inside a lambda is looked up like any
other), and the task names what it copies, row 6's words. `t` needs no word: it is a view of an
element of `texts`, which lives until the join; `n` is the round's own number, which does not.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `texts` (1) | none (default: a parameter reads the caller's list) | none (default: read-only) |
| `total` (2) | none (default: a new local, the function's own) | none (default: an `Atomic` changes only through its own methods) |
| `scope` (3) | `with` | none (default: read-only binding) |
| `t` (4) | none (default: a read-only view of each element) | none |
| `n` (5) | none (default: a new local of this round, its own) | none (default: its own) |
| `n` (6, capture) | `copied` (the task's own copy, row 6) | `copied` (the task's own) |
| `total` (6, in the task) | none (default, row 6: the real `total`) | none (default: through `fetch_add`, the `Atomic`'s own method, written) |

7 names, 5 without a word.

**5. Means exactly** (`means/an73_means.em`):

```ember
fn count(texts: Array[String]) -> int:
    total = Atomic(0)
    tr = ref total
    with scope = thread.scope():
        for t in texts:
            n = t.split_whitespace().count()
            scope.spawn(owned fn() => tr.fetch_add(n))
    return total.load()
```

```text
exit=0
```

Today's `owned fn` copies `n` (an `int`) and copies the view `tr` of the real `total`: the task has its
own `n` and adds to the real total.

**6. Still refused:** the task without the word, once the lookup is fixed:
`error[E3060]: the task uses \`n\`, which ends at the end of this round of the loop (line 5), while
the task may still be running; give the task its own copy: \`fn() with copied n:\``.

**7. Cost:** one atomic add per task, as the program asks (an `Atomic` is the programmer's choice;
`total += n` on a plain `int` would be AN-71's reduction at plain C's cost). The copy of `n` is one
8-byte store into the task's state.

**8. Kind:** a smarter compile-time solution (a defect fixed, and the real cause named with row 6's
words).

---

#### AN-74: a `@sync` class freezes every field after `init`; objects written by several tasks need a lock or a claim

**1. Intent:** several tasks update one statistics object; and each task updates its own player.

**2. OLD** (`old/an74_sync_class.em`, reader 03's t5):

```ember
from std.sync import Atomic

@sync
class Stats:
    words: Atomic[int] = Atomic(0)
    total: int = 0

    fn bump(self):
        self.words.fetch_add(1)
        self.total = self.total + 1
```

```text
error[E7003]: cannot write field `total` of an `@sync` class after `init` or after using `self` as a whole
  --> an74_sync_class.em:11:9
   |
11 |         self.total = self.total + 1
   |         ^^^^^^^^^^
   |
   = help: wrap `total` in `Atomic[int]` for mutation after `init`

error: could not compile due to 1 error(s)
exit=1
```

and each task changing its own class object (`old/an74_parallel_class.em`):

```ember
class Player:
    hp: int = 10

fn hurt_all(players: Array[Player]):
    @parallel
    for p in players:
        p.hp -= 1
```

```text
error[E7010]: a `@parallel` loop counts over a range `a..b`
 --> an74_parallel_class.em:7:14
  |
7 |     for p in players:
  |              ^^^^^^^
  |
  = help: make the iterations independent: an `Atomic`, `chunks_mut`, or `@parallel(reduce=…)`
  = note: a `@parallel` loop's chunks run at once [PAR-2]

error: could not compile due to 1 error(s)
exit=1
```

Last run's answers: a lock field whose guarded fields the compiler infers (`phase3b-threads.md`
4.5), and for class objects in a parallel loop a per-object "claim" at 1.12x to 1.36x of plain C, 1.9x
to 4.0x when the objects are also held elsewhere (the attack, 3.5; `FINDINGS-phase3b.md:94`).

**3. NEW** (`new/an74_new.em`):

```ember
from std.sync import Lock

class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0
    tags: Array[String] guarded by lock = []

    fn add_all(self, text: StrView) changes words, tags:
        for w in text.split_whitespace():
            with self.lock:
                self.words += 1
                if w.starts_with("#"):
                    self.tags.push(String(w))

fn count(texts: Array[String]) -> int:
    stats = Stats()
    with scope = thread.scope():
        for t in texts:
            scope.spawn(fn(): stats.add_all(t))
    with stats.lock:
        return stats.words

class Player:
    hp: int = 10
    x: int = 0

fn hurt_all(ref mut players: Array[Player]):
    @parallel
    for ref mut p in players:
        p.hp -= 1
        p.x += p.hp & 1
```

The rules:
- **`guarded by lock`** (new, section 7) on a field: the field is read and changed only inside
  `with x.lock:` on the same object `x` (the same name, not re-pointed in the block; `self` in
  methods), on the thread that holds it, except in `init`. Several fields may name one lock; a class
  may have several locks. A helper whose body uses a guarded field without taking the lock is
  compiled as "needs `self.lock` held" (last run's attack, inferred and carried in function types) and
  may be called only with the lock held. A `with` on a lock this thread is known to hold calls the
  callee's entry that skips taking it again (the attack's static re-entry; no run-time owner). A view
  of a guarded field does not leave the `with` block (row 4's rule, area B).
- No class mark: `@sync` is gone (AN-80). The fields that are never written after `init` need nothing.
- **Each task its own object**: `for ref mut p in players` in a `@parallel` loop hands each chunk its
  own elements. An element of `Array[Player]` has one owner, the list, because a second owner of a
  class object needs a word (06's `class` row, C-3: `p2 = player` hands over or is refused; a list
  holding one object twice would be `Array[shared Player]`). So no two chunks can reach one object,
  and no run-time claim is needed. This depends on area C ruling the `class` row's condition as 06
  wrote it; if area C keeps silent sharing of class objects, the claim (the attack's 3.5) is the
  fallback (section 9).

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (4) | none (default: the object's own lock) | none (default: a `Lock` changes only by being held with `with`) |
| `words` (5) | none (default: the object's own number) | `guarded by lock` (only while holding `lock`) |
| `tags` (6) | none (default: the object's own list) | `guarded by lock` |
| `self` (8) | `self` (the value the method was called on) | `changes words, tags` (row 3) |
| `text` (8) | `StrView` (a view of someone else's text) | none (default: read-only) |
| `w` (9) | none (default: a read-only view of each word) | none |
| `texts` (15) | none (default: reads the caller's list) | none |
| `stats` (16) | none (default: a new local, the function's own object) | none (default: its own; changed through `add_all`, which says `changes`) |
| `scope` (17) | `with` | none |
| `t` (18) | none (default: a read-only view of each element) | none |
| `stats` (19, task) | none (default, row 6: the real `stats`) | none (default: through `add_all`'s `changes words, tags`, under the lock) |
| `t` (19, task) | none (default: the same view, alive until the join) | none |
| `players` (27) | `ref mut` (the caller's list) | `ref mut` (may change it; 06's C-1) |
| `p` (29) | `ref mut` (each element of the caller's list) | `ref mut` |

14 names, 7 without a word.

**5. Means exactly** (`means/an74_means_lock.em`, today's Ember): the guarded fields inside one
`Mutex`, the class `@sync`:

```ember
struct StatsData:
    words: int
    tags: Array[String]

@sync
class Stats:
    data: Mutex[StatsData] = Mutex(StatsData(words=0, tags=[]))

    fn add(self, w: str):
        with d = self.data.lock():
            d.words += 1
            if w.starts_with("#"):
                d.tags.push(w.to_string())
```

(with `count_into` and three scoped tasks in `main`, as in the file)

```text
exit=0
```

and `means/an74_means_each.em` for the players, each element changed by its own chunk with plain
writes:

```ember
struct Player:
    hp: int
    x: int

fn hurt_all(mut ps: MutSpan[Player]):
    @parallel
    for i in 0..ps.len():
        hp = ps[i].hp - 1
        ps[i] = Player(hp=hp, x=ps[i].x + (hp & 1))
```

```text
exit=0
```

(Today's `[PAR-2]` refuses `ps[i].hp -= 1`, a write to a field of the loop's own element:
`old/an74_parallel_field_write.em` gives `error[E7010]: the parallel loop writes through a place its
chunks share` at `ps[i].hp -= 1`, `exit=1`. The translation writes the whole element, the same stores.
Found in passing: section 10.)

**6. Still refused:**
- A guarded field used outside its lock: `stats.words += 1` in `count`: `error[E7013]: \`words\` is
  guarded by \`lock\`; change it inside \`with stats.lock:\``.
- The lock taken on another object: `with a.lock: b.words += 1`: `error[E7013]: \`b.words\` is
  guarded by \`b.lock\`, and this block holds \`a.lock\`; write \`with b.lock:\``.
- A lock held by the parent "protecting" tasks it spawns (the attack's `y14`): the task body holds no
  lock, so `stats.words += 1` in it is the first message.
- A field written after `init` by two tasks with no lock and no `guarded by`: `error[E7007]: two
  tasks change \`stats.total\` (line 19); guard it: \`total: int guarded by lock\`, and change it inside
  \`with self.lock:\`; or make it \`Atomic[int]\``.
- A list whose objects may have a second owner in a parallel loop: `for ref mut p in players` with
  `players: Array[shared Player]`: `error[E7010]: \`players\` holds \`shared Player\` objects, and one
  object may be in the list twice, so two chunks could change it at once; give each object one owner
  (\`Array[Player]\`), or change it under its lock (\`hp: int guarded by lock\`)`.

**7. Cost.** A lock where the program writes one, the lock C takes (last run, `phase3b-threads.md:741`,
`lock-0`, lock-use-unlock in C, 1479.479 ms against safe Rust's `Mutex` 2170.016 ms, PROVISIONAL);
no field is wrapped, so the object is its plain fields plus one lock word. One thread that uses such a
class pays an uncontended lock per `with`, as Python and C pay. The parallel loop over one-owner
elements: plain loads and stores, no count test and no claim: the per-object claim's 1.12x to 1.36x
(the attack, 3.5) is gone because the words prove what the claim checked while running.

**8. Kind:** a readable spelling of Rust's guarantee (`Mutex<Inner>` spelled as Python's lock field,
with the field saying which lock guards it), plus a smarter compile-time solution for the per-element
loop.

---

#### AN-75: clone-before-move for shared handles (A34)

**1. Intent:** two threads append to one log.

**2. OLD** (`old/a34_clone_dance.em`, phase 1's):

```ember
import std.thread
from std.sync import Mutex

@sync
class Log:
    lines: Mutex[Array[String]] = Mutex([])

fn push(log: Log, s: str):
    with g = log.lines.lock():
        g.push(s.to_string())

fn start(log: Log):
    a = thread.spawn(owned fn() => push(log, "a"))
    b = thread.spawn(owned fn() => push(log, "b"))
    a.join()
    b.join()

fn main():
    start(Log())
```

```text
exit=0
```

It compiles only because `owned fn` silently retains each class handle it captures. Once `owned fn`
goes (vocabulary, `owned fn` row: removed), the same text with plain `fn()`
(`old/an75_no_owned_fn.em`) gives:

```text
error[E3026]: the thread's function borrows `log`, which it may outlive
  --> an75_no_owned_fn.em:14:22
   |
14 |     a = thread.spawn(fn() => push(log, "a"))
   |                      ^^^^^^^^^^^^^^^^^^^^^^ the thread may outlive what it views
   |
   = help: write it `owned fn`, which moves or copies each capture into it; or borrow it inside `with scope = thread.scope():`, which joins its threads before the block ends [THR-5]
   = note: a thread may outlive every local of the code that started it [THR-10]

error[E3026]: the thread's function borrows `log`, which it may outlive
  --> an75_no_owned_fn.em:15:22
...
error: could not compile due to 2 error(s)
exit=1
```

and Rust's way out is the dance: `let log2 = Arc::clone(&log); thread::spawn(move || ...)`.

**3. NEW** (`new/an75_new.em`):

```ember
from std.sync import Lock

class Log:
    lock: Lock = Lock()
    lines: Array[String] guarded by lock = []

    fn push(self, s: StrView) changes lines:
        with self.lock:
            self.lines.push(String(s))

fn start(log: thread_shared Log):
    a = thread.spawn(fn() with thread_shared log: log.push("a"))
    b = thread.spawn(fn() with thread_shared log: log.push("b"))
    a.join()
    b.join()

fn main():
    start(thread_shared Log())
```

The capture word does what Rust's `Arc::clone` line does, on the name it is about: each thread gets
one more owner. No second name (`log2`) and no separate clone line.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (4) | none (default: the object's own lock) | none (default: changes only by being held) |
| `lines` (5) | none (default: the object's own list) | `guarded by lock` |
| `self` (7) | `self` | `changes lines` (row 3) |
| `s` (7) | `StrView` (a view of someone else's text) | none (default: read-only) |
| `log` (11) | `thread_shared` (in its type: a log shared between threads) | `thread_shared` (no owner changes it directly; `push` changes a guarded field) |
| `a` (12) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `log` (12, capture) | `thread_shared` (one more owner) | `thread_shared` |
| `b` (13) | none (default as `a`) | none |
| `log` (13, capture) | `thread_shared` | `thread_shared` |

9 names, 3 without a word.

**5. Means exactly** (`means/an75_means.em`, which is phase 1's `a34_clone_dance.em` itself): a
`@sync` class (atomic counts), its list inside a `Mutex`, and `owned fn` retaining the handle once per
thread. `exit=0`.

**6. Still refused:**
- One thread hands over and the other uses: `with owned log` in both: `error[E3040]: \`log\` was
  handed to the first thread at line 12 (\`with owned log\`); to give both threads the log, write
  \`with thread_shared log\` in both`.
- A one-thread owner sent to a thread: `with shared log`: `error[E7015]: \`shared\` owners stay on
  one thread, and this function runs on a new one; write \`with thread_shared log\``.
- No word: `thread.spawn(fn() => log.push("a"))`: AN-71's E7004 message.
- `with thread_shared log` where `log` is a plain parameter `log: Log` (a view of an object whose
  owner may be one-thread): `error[E7017]: \`log\` is lent to \`start\`, and its owner may not be shared
  between threads; take it as \`log: thread_shared Log\`` (**21:** (F-08) was E7015: this is
  `thread_shared` on a value not known to be `thread_shared`, R3).

**7. Cost:** one atomic increment per `with thread_shared` capture and one atomic decrement when the
thread's owner ends: exactly Rust's `Arc::clone` and drop, and what today's `owned fn` does for a
`@sync` handle. The lock is C's.

**8. Kind:** a readable spelling of Rust's guarantee (`Arc::clone` + `move`, written as one word on the
captured name).

---

#### AN-76: global mutable state (A19)

**1. Intent:** count frames and keep a log in module variables; worker threads add results to a
module list.

**2. OLD** (`old/a19_globals.em`, phase 1's):

```ember
static FRAME: int = 0
static LOG: Array[String] = []

fn tick():
    FRAME += 1
    LOG.push("tick".to_string())
```

```text
error[E1010]: cannot find `FRAME` in this scope
 --> a19_globals.em:7:5
  |
7 |     FRAME += 1
  |     ^^^^^
  |

error[E3021]: cannot write through a shared reference
 --> a19_globals.em:8:5
  |
8 |     LOG.push("tick".to_string())
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this is `ref Array[String]`, which only reads
  |
  = help: take the borrow as `ref mut` if the callee must write, or assign to the place itself rather than through the reference
  = note: while a shared borrow is live the owner may read and copy but not write [BRW-1]

error: could not compile due to 2 error(s)
exit=1
```

`static mut` (`old/an76_static_mut_today.em`):

```text
error[E1010]: `static mut` needs `unsafe`, which is not in this phase
 --> an76_static_mut_today.em:2:12
  |
2 | static mut FRAME: int = 0
  |            ^^^^^
  |

error[E1010]: `static mut` needs `unsafe`, which is not in this phase
 --> an76_static_mut_today.em:3:12
...
error[E1010]: cannot find `FRAME` in this scope
...
error[E1010]: cannot find `LOG` in this scope
...
error: could not compile due to 4 error(s)
exit=1
```

Python's spelling (`old/an76_module_assign.em`, `FRAME = 0` at module level):

```text
error[E1030]: `main` is already declared in this module
 --> an76_module_assign.em:2:1
  |
2 | FRAME = 0
  | ^^^^^^^^^
  |

error[E1010]: cannot find `FRAME` in this scope
...
error[E1010]: cannot find `LOG` in this scope
...
error: could not compile due to 3 error(s)
exit=1
```

**3. NEW** (`new/an76_new.em`):

```ember
static mut FRAME: int = 0
static mut LOG: Array[String] = []

fn tick() changes FRAME, LOG:
    FRAME += 1
    LOG.push(String("tick"))

static RESULTS_LOCK: Lock = Lock()
static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

fn worker(x: int) changes RESULTS:
    r = x * x
    with RESULTS_LOCK:
        RESULTS.push(r)

fn main() changes FRAME, LOG:
    tick()
    a = thread.spawn(fn(): worker(1))
    b = thread.spawn(fn(): worker(2))
    a.join()
    b.join()
```

The rules (07 asked area E to rule on a safe `static mut`):
1. `static mut NAME: T = e` declares a module variable that may change. It needs no `unsafe`; a
   foreign `static mut` (in an `extern` block) keeps `[STA-1]`'s `unsafe` at every use. The
   initialiser follows `[STA-3]`, as today's `static`s.
2. A function whose body changes it, itself or through a call, says so on its first line: `changes
   FRAME` (row 3's word, 07's position on a free function). Checked against the body both ways: a
   change not named is refused, a name not changed is refused. **19:** (R18-04) callers name it too,
   as a method names what the methods it calls change (AN-17; R5): `main` calls `tick()`, so `fn main()
   changes FRAME, LOG:`. Was: "Callers write nothing; the compiler carries the change in the caller's
   effect summary", which gave `changes` a second scope. A function `thread.spawn` starts is not a call
   of `main` (`worker` runs on its own thread; rule 4 and E7012 check it). Every check below sees the
   change through any chain of calls, each link of which now says it, and its messages name the chain.
   **21:** (F-04) A link that runs a function value is the one exception: its changes are D3's (R5),
   and rule 3 uses them.
3. One thread: a loop over `LOG`, or any view into it, across a call whose summary changes `LOG` is
   refused (the agreed limitation: change after the loop). A foreign call's summary includes the
   callbacks C may run on this thread during it (the attack's 5.1, `y16`), so a C call that runs an
   Ember callback which pushes to `LOG` counts as a change. **21:** (F-04) So does an Ember function
   value: a call that may run one (a call through it, an unknown call, or a call of a function whose
   first line lets it change a place holding function values, as `click`'s `changes handlers`, R12) is
   taken to change every module variable that a function value of the program changes (D3; R5, R16).
   (Was: rule 3 read with first lines only, which name no function value's change:
   `20-re-attacker/c/f4_fnvalue_static_loop.c`, heap-use-after-free.)
4. Threads: a `static mut` is used only by the main thread and the tasks of scopes it opens (where
   tasks' changes follow the scope rule and reductions, AN-71). A function reachable from
   `thread.spawn`, `jobs.submit`, a C callback whose contract is not `threads(main)`, or an
   `@export(threads=any|creator)` may not read or change it (E7012) unless it is
   `guarded by` a `static` lock, inside `with` that lock.
5. A `static` of a type that changes only through itself (`Lock`, `Atomic`, `Mutex`, `RwLock`, a
   channel end) is usable from any thread, as today.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `FRAME` (1) | `static` (the module's own, one per program) | `mut` |
| `LOG` (2) | `static` | `mut` |
| `FRAME` (5) | `static` (line 1) | `changes FRAME` (line 4) |
| `LOG` (6) | `static` (line 2) | `changes LOG` (line 4) |
| `RESULTS_LOCK` (8) | `static` | none (default: a `Lock` changes only by being held) |
| `RESULTS` (9) | `static` | `mut`, `guarded by RESULTS_LOCK` |
| `x` (11) | none (default: a parameter reads the caller's value) | none (default: read-only) |
| `r` (12) | none (default: a new local, the function's own) | none (default: its own) |
| `RESULTS` (14) | `static` (line 9) | `guarded by RESULTS_LOCK` (line 9), held by line 13's `with`; `changes RESULTS` on `worker` (line 11; **14:** added, clash 16: rule 2 below asks it of every function that changes a module variable) |
| `main` (17) | `fn` | `changes FRAME, LOG` (**19:** added: it calls `tick()`, R5) |
| `a` (18) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `b` (19) | none (default as `a`) | none |

12 names, 4 without a word (**19:** `main` added).

**5. Means exactly.** Today's Ember cannot say a changeable module variable without `unsafe` (the OLD
output above). In C (`c/an76_static_mut.c`): plain globals, `tick()` changing them, the guarded list
changed only under an SRW lock (the runtime's lock word) by four worker threads:

```c
static int64_t FRAME = 0;                 /* static mut FRAME: int = 0 */
static StrArray LOG = {0};                /* static mut LOG: Array[String] = [] */
static SRWLOCK RESULTS_LOCK = SRWLOCK_INIT;   /* static RESULTS_LOCK: Lock = Lock() */
static IntArray RESULTS = {0};            /* static mut RESULTS: Array[int] guarded by RESULTS_LOCK */

__declspec(noinline) static void tick(void) {   /* fn tick() changes FRAME, LOG: */
    FRAME += 1;
    str_push(&LOG, "tick");
}

static DWORD WINAPI worker(LPVOID p) {          /* fn worker(x: int): */
    int64_t x = (int64_t)(intptr_t)p;
    int64_t r = x * x;
    AcquireSRWLockExclusive(&RESULTS_LOCK);     /* with RESULTS_LOCK: */
    int_push(&RESULTS, r);
    ReleaseSRWLockExclusive(&RESULTS_LOCK);
    return 0;
}
```

Built with `clang -O2 -I target/redesign/bench`, run with `timeout 300 ./an76_static_mut.exe`
(`main` calls `tick()` 1,000,000 times and starts 4 workers with `x` = 0..3):

```text
an76_static_mut ms=26.241 peak_kb=27884 checksum=2000014
```

(checksum = `FRAME` 1,000,000 + `LOG.len()` 1,000,000 + the sum of `RESULTS` 0 + 1 + 4 + 9 = 14.)

**6. Still refused:**
- A change not named: `fn reset(): FRAME = 0`: `error[E2105]: \`reset\` changes \`FRAME\`; write its
  first line as \`fn reset() changes FRAME:\``.
- `changes LOG` on a function that does not change it: `error[E2105]: \`tick\` says \`changes LOG\`
  and never changes it; remove \`LOG\` from its first line`.
- A loop over `LOG` calling `tick()`: `error[E3020]: this loop walks \`LOG\`, and \`tick\` (line 15)
  changes \`LOG\` (\`fn tick() changes FRAME, LOG:\`, line 4); collect what to add in a local list and
  push it after the loop`.
- A worker thread using an unguarded one: `thread.spawn(fn(): tick())`: `error[E7012]: \`tick\`
  changes \`FRAME\`, a module variable of the main thread, and this thread runs beside it; guard it:
  \`static mut FRAME: int guarded by FRAME_LOCK\` and change it inside \`with FRAME_LOCK:\`, or make it
  \`Atomic[int]\``.
- `FRAME = 0` at module level (Python's spelling): `error[E1030]: \`FRAME = 0\` at module level is a
  statement, and this file has \`fn main\`; declare a module variable: \`static mut FRAME: int = 0\``.
- `global FRAME`: today's E0100 help, kept ("`global` is Python's"), pointing at `changes FRAME`.
- **21:** (F-04) A loop over `LOG` across a call that runs a stored handler which pushes to `LOG`
  (`20-re-attacker.md`, F-04's program: `b.on_click(fn(): LOG.push(String("clicked")))` on line 15,
  `for entry in LOG:` on line 16, `b.click()` on line 17): `error[E3020]: this loop walks \`LOG\`, and
  \`b.click()\` (line 17) runs the handlers in \`b.handlers\` (\`fn click(self) changes handlers:\`),
  one of which, made on line 15, pushes to \`LOG\`; click after the loop, or collect what to add and
  push it after the loop`. The program that message leads to is `21-fixer/c/f4_fnvalue_static_loop.c`
  mode 2: clean under AddressSanitizer.
- **21:** (F-04) A lambda's change not named by the function that writes it: `fn setup(ref mut b:
  Button):` with `b.on_click(fn(): LOG.push(String("clicked")))` as its line 2:
  `error[E2105]: the function made on line 2 changes \`LOG\`, and \`setup\` writes it; name it on
  \`setup\`'s first line: \`fn setup(ref mut b: Button) changes LOG:\``.

**7. Cost:** plain C globals: nothing counted, nothing checked while running. Last run measured the
same work (`phase3b-threads.md:854-863`, `tick()` called 100,000,000 times, PROVISIONAL): plain C
globals 192.219 ms (1.00); today's Ember workaround (`Atomic` + `Mutex`) 3.10x on one thread, 7.15x
once a second thread has started; safe Rust's `AtomicU64` + `Mutex` 7.44x; Rust's `thread_local!`
1.04x. The guarded list costs the lock C takes.

**8. Kind:** a smarter compile-time solution (Rust needs `unsafe` or atomics for this; here the
function's first line says what it changes and the compiler proves only one thread group reaches it).

---

#### AN-77: async and generators (A14)

**1. Intent:** a generator yields the lines of its own buffer; another yields the lengths of a
class object's log; an async server function.

**2. OLD** (`old/a14_generators.em`, phase 1's):

```ember
gen fn lines() -> Generator[String]:
    buf = ["a".to_string(), "b".to_string()]
    for s in buf:
        yield s.clone()

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay(self) -> Generator[int]:
        for entry in self.log:
            yield entry.len()
```

```text
error[E2221]: a borrow of `buf`, a local of this generator, is held across this `yield`
 --> a14_generators.em:7:15
  |
7 |         yield s.clone()
  |               ^ the generator suspends here
  |
  = help: iterate the collection `owned` (`for x in owned xs:`), or by index; borrows of the generator's parameters may be held
  = note: a suspended frame is a value that may move, which would leave the borrow pointing where the local was [CORO-6]

error[E2229]: a borrow of a field of a class object is held across this `yield`
  --> a14_generators.em:15:19
   |
15 |             yield entry.len()
   |                   ^^^^^ the generator suspends here
   |
   = help: copy what is needed out of the field before the `yield`, or iterate by index
   = note: a long-term access to an object's field may not span a suspension, when any other code may run [CORO-12]

error: could not compile due to 2 error(s)
exit=1
```

`old/a14_async_kw.em` (`async fn fetch() -> int:`):

```text
error[E0005]: `async` is reserved for v2
 --> a14_async_kw.em:3:1
  |
3 | async fn fetch() -> int:
  | ^^^^^
  |
  = help: rename the item, or write `r#async` to use the word as an identifier

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW** (`new/an77_new.em`):

```ember
gen fn lines() -> Generator[String]:
    buf = [String("a"), String("b")]
    for s in buf:
        yield copied s

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay(self) -> Generator[int] from log:
        for entry in self.log:
            yield entry.len()

fn main():
    for s in lines():
        println(s)
    d = Door()
    for n in d.replay():
        println(n)
```

The rule (the attack's 6.1 and 6.2, kept): a `for` loop over a list, string, map or set whose body
contains `yield` keeps a position in the generator's frame and, after each resume, reads the
collection's header again from where it lives now (the frame, or the object's field) and checks the
position against the current length. No address is held across the suspension, so the frame may move.
Between two resumes the ordinary loan rule applies. When the generator is consumed in place by a `for`
loop in the same function (`for n in d.replay():`), the consumer's body may not change `Door.log`: the
agreed limitation for any loop (a list does not change while a loop walks it). **19:** (R18-10) The
same when the generator is stored or passed: the generator is a view of the list it walks (R6, R14),
so while it is still used the list may not change, wherever the change is written (**21:** (F-09)
its first line says so, `-> Generator[int] from log` (R6, R14), and a stored generator used after a
change is E3060, below; was "(E3020, below)" with nothing on the first line). A
generator meant to see the list as it is at each resume, as Python's list iterator does for `append`
and `pop`, says so by walking by position: `while i < len(self.log):` (R8 (c)); its "means exactly"
is the file in point 5. Was: "When the generator is stored or passed, the walk stays memory-safe by
position and sees the list as it is at each resume", so the one line `for entry in self.log:` meant
"the list does not change" or "walk by position" by how a caller elsewhere used the generator. The
position kept in the frame stays (it is what lets the frame move); only what may change the list
during the walk is now the agreed limitation everywhere.

`async` stays reserved in this version (understandable limitation: no async runtime exists). Its
design is last run's (`phase3b-threads.md` 7.3 rule 3, the attack's 6.3): task groups, which are
scopes; frames that never move; the scope rule in place of `Send`.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `buf` (2) | none (default: a new local, the generator's own) | none (default: its own) |
| `s` (3) | none (default: a read-only view of each element) | none |
| `s` (4, yielded) | `copied` (row 9: a new, independent `String` is given out) | `copied` (the receiver's own) |
| `angle` (7) | none (default: the object's own field) | none (default: changed only through `changes angle` or `mut self`) |
| `log` (8) | none (default: the object's own list) | none (default as `angle`) |
| `self` (10) | `self` | none (default: read-only, row 3) |
| `log` (10, in `from log`) (**21:**) | `from`: the field the generator `replay` returns views (R6) | none (default: `replay` only reads it; while the generator is used, `d.log` may not change) |
| `entry` (11) | none (default: a read-only view of each element) | none |
| `s` (15) | none (default: the `String` the generator gives, the loop's own) | none (default: its own) |
| `d` (17) | none (default: a new local, the function's own object) | none |
| `n` (18) | none (default: the `int` the generator gives) | none |

11 names (**21:** `log` in `from log` added), 8 without a word.

**5. Means exactly** (`means/an77_means.em`, today's Ember): the walk by position written by hand,
which reads `len()` again after each resume:

```ember
gen fn lines() -> Generator[String]:
    buf = ["a".to_string(), "b".to_string()]
    i = 0
    while i < buf.len():
        yield buf[i].clone()
        i += 1

class Door:
    angle: float = 0.0
    log: Array[String] = []

    gen fn replay(self) -> Generator[int]:
        i = 0
        while i < self.log.len():
            yield self.log[i].len()
            i += 1
```

```text
exit=0
```

**6. Still refused:**
- A reference to the frame's own inline local across `yield` (`r = ref n`, then `yield`, then `r`):
  `error[E2221]: \`r\` points into this generator's own frame, which may move while it is suspended;
  keep the value instead (\`m = n\`), or its position`.
- The in-place consumer changing the walked field: `for n in d.replay(): d.log.clear()`:
  `error[E3020]: \`replay\` walks \`d.log\` while this loop runs; clear it after the loop`.
- **19:** (R18-10) a stored generator and a change to its list before its last use: `g = d.replay()`,
  `d.log.push(String("x"))`, `println(g.next())`: **21:** (F-09) `error[E3060]: \`d.log.push(...)\`
  changes \`d.log\`, which \`g\` (line 1) views (\`replay\`'s first line: \`from log\`) and uses on
  line 3; change it after \`g\`'s last use, or make \`replay\` walk by position:
  \`while i < len(self.log):\`` (was E3020 and "which the generator `g` (line 1) still walks": a view
  used after what it points into changed is E3060, as AN-46's `ks`).
- **21:** (F-09) the `for` walk without `from log` on the first line, `gen fn replay(self) ->
  Generator[int]:` with `for entry in self.log:` across `yield`: `error[E3062]: \`replay\` walks
  \`self.log\` with \`for\`, so the generator it returns views \`log\`; write \`gen fn replay(self) ->
  Generator[int] from log:\`, or walk by position: \`while i < len(self.log):\``.
- A walk with no position (a linked structure held by views): E2221 as today, with "walk it by an
  index or a `Handle`".
- `async`: `error[E0005]: \`async\` is planned for a later version; for work that waits, use a
  generator or threads (\`with scope = thread.scope():\`)`.

**7. Cost:** the measured model's own. The attack (`phase3b-threads-attack.md:766-775`, 104,857,600
values over a 1 MiB string, PROVISIONAL): plain C loop 69.943 ms (1.00); position re-read each resume,
`next()` inlined, 82.222 ms (1.18); an address held across `yield` (what the old rule allowed) 82.020
ms (1.17); a class object's field walked by position 71.308 ms (1.02). The gap to plain C is the
generator protocol, not the walk.

**8. Kind:** a smarter compile-time solution (generators); `async` is an understandable limitation kept.

---

#### AN-78: unsafe and FFI (A20): a C callback with user data

**1. Intent:** a C event library calls back into the game; each call adds a point to that game.

**2. OLD** (`old/a20_ffi.em`, phase 1's; `setup_raw` passes a capturing lambda where C wants a plain
function pointer, `setup` uses the contract and `owned fn`):

```ember
unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    fn register_raw(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)

fn setup_raw(game: Game):
    # SAFETY(ffi): test of the refusal
    unsafe:
        register_raw(fn(ud: *mut void): game.score += 1, null[*mut void]())

fn setup(game: Game):
    token = register_cb(owned fn(): game.score += 1)
```

```text
error[E5040]: expected `extern "C" fn(*mut void) -> void`, found a lambda that captures
  --> a20_ffi.em:22:22
   |
22 |         register_raw(fn(ud: *mut void): game.score += 1, null[*mut void]())
   |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this lambda captures
   |
   = help: pass the captured values as arguments, or give C a function that captures nothing
   = help: where C hands the function a user data pointer back, contract the parameter `callback(borrowed, user_data=p)`: it then takes this lambda
   = note: a C function pointer has no room for captured state: `extern "C" fn` holds capture-free functions only ([CLO-3], [FFI-21])

warning[L1001]: `token` is never read
  --> a20_ffi.em:26:5
   |
26 |     token = register_cb(owned fn(): game.score += 1)
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: remove it, or rename it `_token` if that is deliberate

error: could not compile due to 1 error(s)
exit=1
```

With the contract the call compiles, but through `owned fn` (removed) and today's trampoline, 1.47x
plain C per callback (`phase3b-threads.md:1133`, `ffi-1`).

**3. NEW** (`new/an78_new.em`):

```ember
class Game:
    score: i32 = 0

    fn add_point(self) changes score:
        self.score += 1

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)

fn setup(game: shared Game) -> Retained[ForeignCallback]:
    return register_cb(fn() with shared game: game.add_point())

fn main():
    game = shared Game()
    token = setup(game)
    println(game.score)
```

The rules:
- The capture list says what the callback C keeps holds (row 6, 07's capture words): here one more
  owner of the game. `with shared` is allowed because the contract says `threads(creator)`, which the
  trampoline checks; a `threads(any)` callback needs `with thread_shared`. A plain capture (the real
  variable) is allowed only for `callback(borrowed)`, which C calls only during the call.
- **Checked trampoline** (last run 8.3 with the attack's fixes, 7.1 and 7.2): C holds a tagged
  `{slot, generation}` number, never an Ember address; the top 16 bits are `0x4080`, not an address, so
  a C library that wrongly writes through it faults before writing anything. The trampoline checks the
  slot is inside a table that never moves, then the generation and the function's type in one compare
  (a call after `unregister` stops with the callback's name), then the calling thread for
  `threads(creator)`/`threads(main)`. A `threads(any)` callback that can be unregistered while it runs
  announces the running slot, and unregistering waits.
- The contract may be read from the header when its shape is a function pointer plus a `void*` it
  hands back (last run 8.4); it is listed by `ember tcb` as an asserted fact, like a written one.
- A foreign call's effect summary includes the callbacks C may run on this thread during it (AN-76
  rule 3).
- Whether `game.add_point()` may change the game through one of its `shared` owners is area C's
  ruling (AN-48); the callback is just one more owner on the same thread, and follows it.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `score` (2) | none (default: the object's own number) | none (default: changed only through `changes score`) |
| `self` (4) | `self` | `changes score` (row 3) |
| `cb` (8-10) | `extern "C" fn` (a C function pointer, its type) | none (default: a parameter only reads) |
| `ud` (8-10) | `*mut void` (C's raw pointer; the contract names it user data) | none (default: Ember never reads it here) |
| `game` (12) | `shared` in the type: a view of the caller's `shared` game, no count (**15:** R4: the mode is on the name; the capture on line 13 makes the owner C keeps) | none (default: a parameter reads) |
| `game` (13, capture) | `shared` (one more owner) | `shared`: an owner may change it (R9), here through `add_point`'s `changes score` (**15:** was "as line 12") |
| `game` (16) | `shared` (a new object with several owners) | as line 13 (**15:** was "as line 12") |
| `token` (17) | none (default: the `Retained` token the call returns, the caller's own; dropping it unregisters) | none |

8 names, 2 without a word.

**5. Means exactly** (`means/an78_means.em`, today's Ember): `owned fn` retains the class handle,
which is "one more owner":

```ember
fn setup(game: Game) -> Retained[ForeignCallback]:
    return register_cb(owned fn(): game.score += 1)

fn main():
    g = Game()
    token = setup(g)
    println(g.score)
```

```text
warning[L1001]: `token` is never read
  --> means/an78_means.em:18:5
   |
18 |     token = setup(g)
   |     ^^^^^^^^^^^^^^^^ declared here and never used
   |
   = help: remove it, or rename it `_token` if that is deliberate

exit=0
```

**6. Still refused:**
- No capture word for a kept callback: `register_cb(fn(): game.add_point())`: `error[E7004]: C keeps
  this function after \`setup\` returns (\`callback(retained)\`), so it cannot use \`setup\`'s
  \`game\`; say what it holds: \`fn() with shared game:\` (one more owner) or \`fn() with weak game:\`
  (a link that lets the game end)`.
- `with shared game` under `threads(any)`: `error[E7015]: C may call this function from any thread
  (\`threads(any)\`), and \`shared\` owners stay on one; create the game shared between threads,
  \`game = thread_shared Game()\`, take it as \`game: thread_shared Game\` in \`setup\`, and write
  \`with thread_shared game\`` (**21:** (F-08) was "write `with thread_shared game`" alone, which E7017
  refuses on a `shared` game, R3).
- A capturing lambda where C wants a bare function pointer: E5040, today's message (it already gives
  the line to write).
- `owned fn(...)`, as `register_cb(owned fn(): game.add_point())`: `error[E0109]: \`owned fn\` does
  not say what the function takes or copies; name it on each captured name: \`fn() with shared
  game:\`; a name not listed is the real variable` (vocabulary, `owned fn` row). **19:** (R18-08) was
  `E0100` and "`fn() with copied score:`": one code and one text for `owned fn` in every answer
  (section 8), and the line to write in this program's names (it has `game`, not `score`).

**7. Cost:** the checked trampoline, 1.05x plain C (the attack, `phase3b-threads-attack.md:902`,
164.575 against 156.179 ms; 1.02x in its second batch; the tag adds no instruction, 163.755 ms,
line 960), against today's trampoline 1.47x (`phase3b-threads.md:1133`). The announced
`threads(any)` version that may be unregistered while running costs 1.42x to 1.58x on an empty
callback (the attack, 7.1): above C's cost, and only on that path; Rust's workaround (0.97x,
`phase3b-threads.md:1137`) checks none of these facts.

**8. Kind:** a new mechanism (the checked trampoline), safer than Rust's: a stale, forged or
wrong-thread call is caught, where Rust's workaround is an unbounded `unsafe` reference.

---

#### AN-79: `unsafe` points two ways, and an `unsafe fn` body is unchecked with no mark per operation

**1. Intent:** write a small unsafe helper and call it.

**2. OLD** (`old/an79_unsafe_fn_body.em`, the spec's own example IX.4 with its notes): the helper's
body reads through a raw pointer four times with no `unsafe:` block and no note at any read.

```ember
@safety("p points to four readable bytes")
# SAFETY: the caller guarantees `p` points to four readable bytes
unsafe fn read_u32_le(p: *u8) -> u32:
    return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4:
        return None
    # SAFETY: the length check above guarantees four readable bytes
    unsafe:
        return Some(read_u32_le(data.as_ptr()))
```

```text
exit=0
```

**3. NEW** (`new/an79_new.em`):

```ember
@safety("p points to four readable bytes")
unsafe fn read_u32_le(p: *u8) -> u32:
    # SAFETY: the caller promised four readable bytes at `p` (@safety above); these are bytes 0 to 3
    unsafe:
        return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4:
        return None
    # SAFETY: the length check above guarantees four readable bytes
    unsafe:
        return Some(read_u32_le(data.as_ptr()))
```

The rule: one word, two places, each with its own sentence. `unsafe` before a declaration (`unsafe
fn`, `unsafe interface`, an `unsafe fn(...)` type) says "whoever uses this must vouch", and `@safety`
says what they must make true. `unsafe` before a block or an implementation (`unsafe:`, `unsafe
extend`, `unsafe extern`, `unsafe overlay`) says "this code vouches", and `# SAFETY:` says why. Inside
an `unsafe fn` body every operation of `[UNS-1]` must still sit in an `unsafe:` block, with its own
note (Rust 2024's `unsafe_op_in_unsafe_fn`, made an error, E3100). 07 planned no second spelling
(`planned-words.md` section 6) and the position already tells the two apart; this answer agrees.

**15:** the NEW text had kept the OLD line `# SAFETY: the caller guarantees ...` above `unsafe fn`:
a second note on the declaration, saying what `@safety` already says, while the rule above gives
`# SAFETY:` to blocks and implementations only. It is removed, so each note sits on the one thing it
describes (R15). Today's compiler asks for it (`[UNS-8]`): the NEW text without it gets W3012 on the
`unsafe fn` line (`target/redesign3/15-vocabulary-review/an79_no_fn_note.em`, `ember check`):

```text
warning[W3012]: this `unsafe` has no safety note
 --> an79_no_fn_note.em:3:1
  |
3 | unsafe fn read_u32_le(p: *u8) -> u32:
  | ^^^^^^ nothing here says why it is sound
  |
  = help: write `# SAFETY: <why it is sound>` on the line before it or at the end of this line; `SAFETY(ffi)`, `(layout)`, `(aliasing)`, `(intrinsic)`, `(performance)` or `(uninit)` names its category

exit=0
```

In the model W3012 is for `unsafe:` blocks and implementations; on an `unsafe fn` the check is
L3015's (`@safety`, asked of every `unsafe fn`). The translation below keeps the line only because
today's compiler asks for it; it changes nothing the program does.

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `p` (2) | `*u8` (a raw pointer to someone else's bytes) | `*u8` without `mut`: reads only |
| `data` (7) | none (default: a parameter reads the caller's bytes; `Span` is a view) | none (default: read-only) |

2 names, 1 without a word. (**15:** line numbers moved by one with the removed line.)

**5. Means exactly** (`means/an79_means.em`, which is the NEW text with today's extra note above the
`unsafe fn` line: today accepts it):

```text
exit=0
```

What changes is what is refused.

**6. Still refused:**
- An unsafe operation in an `unsafe fn` body outside a block: the OLD helper: `error[E3100]:
  \`read_u32_le\` is an \`unsafe fn\`, which says its callers must vouch; the read at line 4 is this
  function's own unsafe step: put it in an \`unsafe:\` block with a \`# SAFETY:\` note saying why it
  holds here`.
- `unsafe fn` with no `@safety` on any function, not only `pub` ones (today `L3015` for `pub`):
  `warning[L3015]: say what callers of \`read_u32_le\` must make true:
  \`@safety("p points to four readable bytes")\``.
- An `unsafe:` block with no note: `W3012`, today's message (it already gives the line).
- **15:** A `# SAFETY:` note on the `unsafe fn` line itself (the OLD's line 2), which the model no longer
  reads: `warning[W3013]`, "`read_u32_le` says what its callers must make true with `@safety`; a
  `# SAFETY:` note says why one block is sound and goes above each `unsafe:` block inside it: remove
  this line" (**19:** (R18-19) code W3013, new and free (section 8); was "the code number is for the
  lead").
- Calling an `unsafe fn` outside `unsafe`: E3100, today's.

**7. Cost:** none; compile-time only.

**8. Kind:** a readable spelling of Rust's guarantee (Rust 2024's rule, with the two directions told
apart by where `unsafe` sits and by their two notes).

---

#### AN-80: `Sync` names three things; `@sync` reads "synchronised"; `@nosync` reads "not `@sync`"

**1. Intent:** mark a class that threads may share; forbid locks in a hot function.

**2. OLD** (`old/an80_sync_nosync.em`): `@sync` read as "synchronised": two tasks call `note`, which
reads `longest` and then writes it in two separate atomic steps (the race `[THR-7]` warns about), and
`@nosync` on a hot function:

```ember
@sync
class Stats:
    words: Atomic[int] = Atomic(0)
    longest: Atomic[int] = Atomic(0)

    fn note(self, n: int):
        self.words.fetch_add(n)
        if n > self.longest.load():
            self.longest.store(n)

@nosync
fn sum(xs: Span[int]) -> int:
    s = 0
    for x in xs:
        s += x
    return s

fn main():
    st = Stats()
    with scope = thread.scope():
        scope.spawn(fn(): st.note(3))
        scope.spawn(fn(): st.note(5))
```

```text
exit=0
```

Accepted, and `longest` may end as 3: both tasks read 0, the one with 5 stores first, the one with 3
stores last. Writing the field plainly instead is E7003 (AN-74's OLD).

**3. NEW** (`new/an80_new.em`):

```ember
class Stats:
    lock: Lock = Lock()
    words: int guarded by lock = 0
    longest: int guarded by lock = 0

    fn note(self, n: int) changes words, longest:
        with self.lock:
            self.words += n
            if n > self.longest:
                self.longest = n

@nosync
fn sum(xs: Span[int]) -> int:
    s = 0
    for x in xs:
        s += x
    return s

fn main():
    st = thread_shared Stats()
    a = thread.spawn(fn() with thread_shared st: st.note(3))
    st.note(5)
    a.join()
```

The three meanings get three spellings, as 06 settled (C-4), and area E rules on the marker (07,
section 8):
- the **effect** keeps `Sync` and **`@nosync`** keeps its spelling: "no synchronisation", which is what
  it forbids (locks, atomics stronger than relaxed, channels, spawn and join, spec 3184-3186);
- the **property** "several threads may hold and read one value" is **`ThreadShared`** (06), written
  only in bounds (`fn keep[T: ThreadShared](x: T)`) and messages, inferred for every type as Rust
  infers `Sync`;
- the **marker** `@sync` (06's placeholder `@thread_shared`) is **removed**: what it said on the class
  (handles cross threads, atomic counts, fields frozen) is said on the value where it is shared,
  `thread_shared Stats()`, and which fields may change is said on each field (`guarded by`, `Atomic`).
  With the marker gone, `@nosync` can no longer be read as "not `@sync`".

**4. The owner's test:**

| name (line) | what it is | may it change it |
|---|---|---|
| `lock` (2) | none (default: the object's own lock) | none (default: changes only by being held) |
| `words` (3) | none (default: the object's own number) | `guarded by lock` |
| `longest` (4) | none (default: the object's own number) | `guarded by lock` |
| `self` (6) | `self` | `changes words, longest` (row 3) |
| `n` (6) | none (default: a parameter reads the caller's value) | none |
| `xs` (13) | none (default: reads the caller's numbers; `Span` is a view) | none |
| `s` (14) | none (default: a new local, the function's own) | none (default: its own) |
| `x` (15) | none (default: a read-only view of each number) | none |
| `st` (20) | `thread_shared` (one of several owners, on any thread) | `thread_shared` (changed only through `note`, under the lock) |
| `a` (21) | none (default: the `JoinHandle` the call returns, the caller's own) | none |
| `st` (21, capture) | `thread_shared` (one more owner) | `thread_shared` |

11 names, 6 without a word.

**5. Means exactly** (`means/an80_means.em`, today's Ember): the two counters inside one `Mutex`, so the
read and the write of `longest` are one step; `@sync` for the cross-thread object; `@nosync` as today:

```ember
struct StatsData:
    words: int
    longest: int

@sync
class Stats:
    data: Mutex[StatsData] = Mutex(StatsData(words=0, longest=0))

    fn note(self, n: int):
        with d = self.data.lock():
            d.words += n
            if n > d.longest:
                d.longest = n
```

(with `sum` and two scoped tasks in `main`, as in the file)

```text
exit=0
```

**6. Still refused:**
- `@sync class Stats`: `error[E0104]: \`@sync\` is gone: a class needs no mark to be shared between
  threads; share the object where it is shared, \`st = thread_shared Stats()\`, and say which lock
  guards each field that changes (\`words: int guarded by lock\`) or make it \`Atomic[int]\``.
- `T: Sync` in a bound: `error[E1010]: \`Sync\` is the effect \`@nosync\` forbids, not a property of
  a type; for "threads may share it" write \`T: ThreadShared\``.
- `st.longest = n` outside the lock: AN-74's E7013 message.
- `@nosync` reaching a lock: `E4002`, today's, which names the chain (`[EFF-6]`).

**7. Cost:** none from the words; the lock is C's (AN-74).

**8. Kind:** a readable spelling of Rust's guarantee (Rust's `Sync`, inferred, named `ThreadShared`;
the class marker replaced by words on the value and the fields).

---

#### AN-81: `@static_safe` forbids only aliasing checks

**1. Intent:** mark a hot function as having no run-time checks.

**2. OLD** (`old/an81_static_safe.em`):

```ember
@static_safe
fn total(xs: Span[int], n: int) -> int:
    s = 0
    for i in 0..n:
        s += xs[i]
    return s
```

```text
exit=0
```

Accepted, and `xs[i]` keeps its bounds check (`[EFF-12]`, spec 3260: `@static_safe` permits bounds,
arithmetic and stale checks).

**3. NEW** (`new/an81_new.em`):

```ember
@no_alias_checks
fn total(xs: Span[int], n: int) -> int:
    s = 0
    for i in 0..n:
        s += xs[i]
    return s
```

The same contract, spelled by what it forbids (vocabulary, 06, and 07 kept it): it reads as the rest of
the `@no...` family (`@noalloc` forbids allocation; `@no_alias_checks` forbids run-time alias checks,
E4030).

**4. The owner's test:** `xs` none (default: reads the caller's numbers), `n` none (default: reads the
caller's value), `s` none (default: the function's own local), `i` none (default: the loop's own
number). 4 names, 4 without a word; none changes anything of the caller's.

**5. Means exactly** (`means/an81_means.em`, the OLD text: `@static_safe` is today's spelling of the
same contract): `exit=0`.

**6. Still refused:**
- `@static_safe`: `error[E0104]: \`@static_safe\` is now \`@no_alias_checks\`: it forbids run-time
  alias checks only; bounds and overflow checks still run`.
- A run-time alias check inside a `@no_alias_checks` function, written out (**19:** (R18-19) was
  given against `total`, which calls no `collide`): AN-03's `step` marked `@no_alias_checks`, whose
  `collide(bodies[i], bodies[j])` (line 3) is compared once while running: `error[E4030]: \`step\` is
  \`@no_alias_checks\`, and \`collide(bodies[i], bodies[j])\` (line 3) compares \`i\` and \`j\` while
  running; prove them different (\`if i != j:\` around the call), or remove \`@no_alias_checks\``.
  (**21:** (F-11) `for j in i + 1..` was offered too, but `step`'s `j` is a parameter, not a loop name:
  a help that does not apply, A-8.) **14:** (clash 5) the run-time alias checks left in
  the model are R4's compare and `assert_disjoint`; `RuntimeChecked` is removed.

**7. Cost:** none; the contract is checked when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (a rename: the contract keeps its meaning and is
named for it).

---

#### AN-82: `@no_runtime_checks` is reserved, promising a switch that will never exist

**1. Intent:** turn off safety checks in a release build.

**2. OLD** (`old/an82_no_runtime_checks.em`):

```ember
@no_runtime_checks
fn total(xs: Span[int], n: int) -> int:
    s = 0
    for i in 0..n:
        s += xs[i]
    return s
```

```text
error[E0104]: `@no_runtime_checks` is reserved for a later version
 --> an82_no_runtime_checks.em:2:1
  |
2 | @no_runtime_checks
  | ^^^^^^^^^^^^^^^^^^
  |

error: could not compile due to 1 error(s)
exit=1
```

The message says the switch is coming; `[EXC-1]` (2708) and `[DSJ-8]` (3173) say it never will.

**3. NEW** (`new/an82_new.em`): no such word. The function the programmer can write:

```ember
fn total(xs: Span[int]) -> int:
    s = 0
    for x in xs:
        s += x
    return s
```

**4. The owner's test:** `xs` none (default: reads the caller's numbers), `s` none (default: its own
local), `x` none (default: a read-only view of each number). 3 names, 3 without a word.

**5. Means exactly** (`means/an82_means.em`, the NEW text): `exit=0`.

**6. Still refused:** `@no_runtime_checks`: `error[E0104]: there is no switch that turns safety
checks off, in any build (\`[PHIL-13]\`); to forbid run-time alias checks here write
\`@no_alias_checks\`; to skip one bounds check you have proved, write \`xs.get_unchecked(i)\` inside
\`unsafe:\` with a \`# SAFETY:\` note`.

**7. Cost:** the checks the program needs; a loop over the view itself (`for x in xs`) reads each
element once with no index to check.

**8. Kind:** an understandable limitation kept (safety is never switched off), with a message that says
what to write.

---

#### AN-83: `@fp(contract)` allows fused multiply-add while "contract" elsewhere means a checked promise

**1. Intent:** allow fused multiply-add in one function.

**2. OLD** (`old/an83_fp_contract.em`):

```ember
@fp(contract)
fn dot(ax: float, ay: float, bx: float, by: float) -> float:
    return ax * bx + ay * by
```

```text
exit=0
```

**3. NEW** (`new/an83_new.em`, 07's spelling):

```ember
@fp(allow_fma)
fn dot(ax: float, ay: float, bx: float, by: float) -> float:
    return ax * bx + ay * by
```

**4. The owner's test:** `ax`, `ay`, `bx`, `by`: none (default: parameters read the caller's
values). 4 names, 4 without a word.

**5. Means exactly** (`means/an83_means.em`, the OLD text, today's spelling): `exit=0`.

**6. Still refused:** `@fp(contract)`: `error[E0104]: \`@fp(contract)\` is now \`@fp(allow_fma)\`: it
lets the compiler fuse \`a * b + c\` into one fused multiply-add`. `@fp(allow_fma)` inside a
`@deterministic` function: E4072, as `@fastmath` today ("`dot` is `@deterministic`, and a fused
multiply-add rounds once where two operations round twice; remove `@fp(allow_fma)`").

**7. Cost:** none more; one instruction where two were.

**8. Kind:** a readable spelling of Rust's guarantee (a rename of an argument; Rust has no such
attribute, its `f64::mul_add` is explicit).

---

#### AN-84: two error codes mean two unrelated things each: E5015 and E7010

**1. Intent:** read an error code and look it up.

**2. OLD.** `old/an84_e5015_variadic.em`: an owning value in an exported signature, and a variadic
argument C would promote; both are E5015:

```text
error[E5015]: exported parameter `text` reaches an owning Ember value
 --> an84_e5015_variadic.em:4:29
  |
4 | pub extern "C" fn read_text(text: *String) -> i32:
  |                             ^^^^^^^^^^^^^
  |

error[E5015]: C promotes a `i16` passed as a further argument to `int`
  --> an84_e5015_variadic.em:10:31
   |
10 |         println(v.sum_wide(1, narrow))
   |                               ^^^^^^
   |
   = help: pass it as C reads it: `x as c_int` (or `c_uint`)
   = note: C reads a further argument with `va_arg` as the type it expects, after its default argument promotions [FFI-8]
   = note: declared in the header at variadic.h:8: `int64_t sum_wide(int count, ...)` [DIA-5]

error: could not compile due to 2 error(s)
exit=1
```

`old/an84_e7010_parallel.em` (a parallel loop writing through `xs[idx[i]]`):

```text
error[E7010]: the parallel loop writes `xs`, and reaches it here at an index that is not `i` plus a constant
 --> an84_e7010_parallel.em:5:9
  |
5 |         xs[idx[i]] = i
  |         ^^^^^^^^^^
  |
  = help: index it by the loop's own index, `i + k`, or make it a reduction
  = note: a `@parallel` loop's chunks run at once [PAR-2]

error: could not compile due to 1 error(s)
exit=1
```

The spec gives E7010 a second meaning (4966: an `any`-thread export reaching a static that is not
`Sync`); the compiler emits E7010 only for parallel loops (`compiler/ember_typeck/src/lib.rs`, lines
51927, 52238, 52274, 52292, 52308, and `coroutines.rs:1107`), but it also uses it for "a `@parallel`
loop counts over a range `a..b`" with the unrelated help "an `Atomic`, `chunks_mut`, or
`@parallel(reduce=…)`" (AN-74's OLD, `old/an74_parallel_class.em`). The code table (spec 5774, 5816)
lists one meaning each.

**3. NEW:** one meaning per code:

| code | means | was |
|---|---|---|
| `E5015` | this type may not cross into C as written (an owning Ember value in an exported signature; a type with no C form) | the same, plus the variadic case |
| **`E5013`** (new) | C would change this further argument of a variadic call (a default promotion, or an untyped literal); pass it as C reads it | E5015 |
| `E7010` | a parallel loop's chunks may write one place | the same, plus the export case (spec only) and the loop-shape case |
| **`E7012`** (new) | a module variable of the main thread (`static mut` not `guarded by` a lock) is reached by code that runs on another thread (an unscoped thread, an `any`/`creator` export, a C callback not `threads(main)`) | spec 4966's E7010 |
| **`E7016`** (new) | a `@parallel` loop is over something it cannot split into chunks: neither a range (`for i in 0..n:`) nor a list walked by element (`for ref mut p in xs:`), for example a `while` loop (**19:** (R18-05) was "(it counts over a range, or walks a list with `for ref mut p in xs`)", the two forms AN-74 and this answer's `place` accept; it described today's E7010 text, "a `@parallel` loop counts over a range `a..b`", whose rule is that a parallel loop must count over a range) | the compiler's E7010 at `for p in players` |

The other area E codes this report uses are in section 8.

Messages under the new codes:

```text
error[E5013]: C reads the further argument `narrow` as an `int`, and an `i16` would be promoted on the way
  --> an84_e5015_variadic.em:10:31
   = help: pass it as C reads it: `narrow as c_int`
```

```text
error[E7012]: `bump` may be called from any thread (`@export(threads=any)`), and it changes `HITS`, a module variable of the main thread
   = help: guard it: `static mut HITS: int guarded by HITS_LOCK`, and change it inside `with HITS_LOCK:`; or make it `Atomic[int]`
```

**19:** (R18-05) E7016's message. **21:** (F-11) given on the refused program written out, `place`
with its loop as a `while` loop (was given "on `@parallel` above `while i < n:` in `place`", whose
`place` has no `while`, no `n` and no `x`):

```ember
fn place(xs: MutSpan[int], idx: Span[int]):
    i = 0
    @parallel
    while i < idx.len():
        xs[i] = idx[i]
        i += 1
```

```text
error[E7016]: `@parallel` splits a range or a list into chunks, and `while i < idx.len():` (line 4) in `place` is neither
   = help: count over the range: `for i in 0..idx.len():`
```

The help gives one line: the body indexes both `xs` and `idx` by `i`, so a walk of `xs` by element
(the other form E7016 names) would not say it (A-8: no help that does not apply); the line it gives is
the accepted `place` below. The owner's test on the refused program: `xs`, `idx` as in the table
below; `i` (line 2): none (default: `place`'s own number), changed by `i += 1`, written.

**14:** completed (section 6 of this file): the programs the new codes' messages lead the programmer
to, in NEW spelling, so that the owner's test can be applied to every name a message asks for. Its
translation into today's Ember is area E's `means/an84_means.em` (point 5, `exit=0`) for `read_text`,
`place` and `main`; the guarded module variable that E7012's help asks for is the shape of AN-76's
`RESULTS`, whose C is `c/an76_static_mut.c` (pasted under AN-76).

```ember
import c "./variadic.c" with (overlay = "overlays/variadic.em") as v
from std.ffi import c_int
from std.sync import Lock

pub extern "C" fn read_text(text: *u8) -> i32:         # E5015's help: a C type in the export
    return 0

fn place(xs: MutSpan[int], idx: Span[int]):             # E7010's help: indexed by its own `i`
    @parallel
    for i in 0..idx.len():
        xs[i] = idx[i]

static HITS_LOCK: Lock = Lock()
static mut HITS: int guarded by HITS_LOCK = 0           # E7012's help

@export(threads=any)
fn bump() changes HITS:
    with HITS_LOCK:
        HITS += 1

fn main():
    narrow: i16 = 3
    unsafe:  # SAFETY(ffi): the further argument is passed as the `int` C reads
        println(v.sum_wide(1, narrow as c_int))         # E5013's help
    a = [0, 0, 0]
    order = [2, 0, 1]
    place(a.as_mut_span(), order.as_span())
    println(a)
```

**4. The owner's test** (**14:** completed; the automatic check found it applied to no name):

| name (line) | what it is | may it change it |
|---|---|---|
| `v` (1) | `import ... as v`: the C module's name | none (default: a module's name changes nothing) |
| `text` (5) | `*u8` (its type): a raw pointer to bytes C owns | `*u8` without `mut`: reads only |
| `xs` (8) | `MutSpan` (its type): a view of the caller's numbers (AN-11) | `MutSpan`: its elements may be written |
| `idx` (8) | `Span` (its type): a view of the caller's numbers | `Span`: read only |
| `i` (10) | none (default: the loop's own number) | none (default: set by the loop only) |
| `HITS_LOCK` (13) | `static`: the module's own lock | none (default: a `Lock` changes only by being held with `with`) |
| `HITS` (14) | `static`: the module's own number | `mut`, `guarded by HITS_LOCK` |
| `HITS` (19) | `static` (line 14) | `changes HITS` on `bump` (line 17), inside `with HITS_LOCK:` (line 18) |
| `narrow` (22) | none (default: `main`'s own number) | none (default: its own) |
| `a`, `order` (25, 26) | none (default: `main`'s own new lists) | none (default: its own; `place` writes `a`'s elements through its `MutSpan`) |

11 names, 4 without a word (`i`, `narrow`, `a`, `order`), each with its default stated; none of the defaults changes, aliases or copies
anything the line does not show.

**5. Means exactly** (`means/an84_means.em`): the programs after following each help (the refusals
stay; only the codes change): `narrow as c_int`, an export taking `*u8`, a parallel loop indexed by
its own `i`:

```text
exit=0
```

**6. Still refused:** everything refused today stays refused, under the codes above.

**7. Cost:** none.

**8. Kind:** a readable spelling of Rust's guarantee (the same refusals; each code now names one
thing, as Rust's codes do).

## 15. Files

- This file: built by `target/redesign3/14-combiner/build.py` from `target/redesign3/14-combiner/parts/`
  (`head.md`: sections 0 to 9; `an16_test.md`, `an84_test.md`: the two completed owner's tests;
  `tail.md`: this section) and the five area reports. The script applies every **14:** change as an exact
  replacement that must match once, and checks that the 84 answers are all present, in order, each with
  its eight parts, and that every agreed spelling of rows 3 to 9 appears.
- `RUN3/vocabulary-combined.md`: the one vocabulary, built by `target/redesign3/14-combiner/vocab.py`
  from `RUN3/vocabulary.md` with every row this model changes rewritten and the new words added.
- The areas' programs, outputs and C: `target/redesign3/08-area-A/` .. `12-area-E/`, unchanged; the
  re-check of every OLD and "means exactly" file: `target/redesign3/13-old-code-recheck/`.
- What each area dropped from last run, and the good parts it kept, are in the area reports (A section 6,
  B section 5, C section 4, D section 7, E section 4); none of them is changed by this combination.
