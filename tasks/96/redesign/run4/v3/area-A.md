# Area A under the v3 rules: lists, loops, temporary values (AN-01 to AN-16)

Run 4, step 2, v3, agent 02 (area A). Every answer of area A, brought to the rules in `V3/rules.md`,
the words in `V3/vocabulary.md` and the changes in `V3/changes.md`, which apply the owner's verdicts 21
to 34 of 2026-10-25 (`inputs/JOB.md`; verdict 25 is replaced by 27 and not applied) on top of verdicts 1
to 20, which v2 applied. `RUN3` means `tasks/96/redesign/run3/`, `RUN4` means `tasks/96/redesign/run4/`,
`V2` means `tasks/96/redesign/run4/v2/`, `V3` means `tasks/96/redesign/run4/v3/`; "R1" to "R18" are the
rules of `V3/rules.md`; "row N" is the owner's agreed row N and "verdict N" the owner's verdict N in
`inputs/JOB.md` ("Owner verdicts on run 4's open questions").

No compiler change, no git command, no Ember program run, and no `ember check` run in this step: no new
claim about what today's Ember accepts or refuses is made here. This step ran in a cloud container, and
no power reading was taken. Every claim about today's Ember is a pasted output carried over with its
unchanged OLD code or "means exactly" program: run 3's (checked with `target/redesign3/bin/ember.exe`),
and the four "means exactly" files v2 checked with `target/redesign4/bin/ember.exe` (in the laptop's
`target/redesign4/v2-02-area-A/means/`, not in this checkout). Statements about the v3 rules carry no
output: they are not built.

**How this file was made.** v2's file was built by a script from line ranges of `RUN4/area-A.md`, so
that no carried line (an OLD program with its `ember check` output, a "means exactly" program with its
output, a C program, a measured cost) could differ from run 3's by a retyping mistake. This v3 file is
that file, copied by the lead's phase 0 (which changed no line of it: `V3/spelling-changes.txt` lists
eleven lines here that still say `holds`, every one English, and they stay), then edited in place for
verdicts 21 to 34. The carried blocks are unchanged; a copy of the phase 0 file and a diff of this step's
edits are in `target/redesign4/v3-02-area-A/`. File names inside the carried blocks are relative to
`target/redesign3/08-area-A/`, as in run 3. Each answer opens with a "**Meaning changed:**" line that
compares it with `V2/area-A.md` and names the verdict.

## 0. Terms

The terms of `V3/rules.md` section 0 (owner, view, read-only view, position, reshape, `this`, first line,
change list, summary, function value, noalias, the owner's test, ...) are used as defined there. This
area adds:

- **list**: `Array[T]`: a pointer to a block of memory holding the elements (the **buffer**), the length
  and the capacity. A full list that grows gets a new, larger buffer and frees the old one
  (**reallocates**), so an address into the old buffer would point into freed memory.
- **element name**: a name made from one element of a list, `boss = ref mut enemies[0]`. By R8 (a) it is
  a position: the element is found again at each use.
- **temporary**: a value made inside an expression and never given a name (`line.to_lower()` in
  `line.to_lower().trim()`).
- **"only adds" summary**: what the compiler records about a function that takes a list as `ref mut`
  when its only use of the list is adding at its end (`push`, `extend`) and it reads or writes no element
  already in the list; anything else is "may do more". Worked out from the body; across a package's edge
  it is recorded in the interface file (R17).
- **a call through a function value**: a call whose body the compiler cannot read here; it may do what
  the function value's type allows, no more (R12): `fn(ref mut Array[Enemy])` may change the list it is
  given in any way, and nothing else at all (a plain function type changes nothing beyond its `ref mut`
  parameters, the variables a lambda captured included, verdict 22).
- **refused**: a compile-time error, with its code from `V3/rules.md` section 4. **Stops**: a run-time
  stop with a message (Ember's panic). **Warned**: accepted, with a warning.
- **line numbers in messages** count the answer's program as a file whose first line is a comment, as
  run 3's files are; where a refused snippet replaces part of the NEW program, it is counted in that
  place.
- **PROVISIONAL**: a timing made while other agents were running (run 3's mark, kept on run 3's numbers).

## 1. Run 3's area rules A-1 to A-8, and where each is now

| run 3's area rule | now | where it stands in v3 |
|---|---|---|
| A-1: an element name is a position; plain `boss = enemies[0]` reads, `ref mut` changes, a copy is named, a number part is copied | R2, R8 (a) | as in v2; the copy is `copy of` (verdict 12): `boss = copy of enemies[0]` |
| A-2: while an element name is used, its list may grow but not shrink or move; writes to the element through another name are allowed | R8 (a), (f), (g) | as in v2: the last half is reversed for both kinds of name. While a `ref mut` element name is used, its element is neither read nor changed through another name (R8 (f), verdict 2); while a read-only one is used, its element is not changed through another name (R8 (g), verdict 5, E3021 with both ways out). Adding to the list stays allowed for both |
| A-3: a list may not change while a loop walks it; a counting loop may add, not remove | R8 (c), R12 | as in v2 (a generator paused in a list is such a walk too, verdict 16; a call through a function value changes what its type says, verdict 17), and **wider in v3**: a library call given a function counts as making that function's changes (verdict 21: `enemies.retain(keep_alive)` changes what `keep_alive` changes), and so does a call that hands a function to a thread or to C (verdicts 30, 31; none in this area) |
| A-4: two `ref mut` parameters are two different places; one compare where not proved | R4 | as in v2: every `ref mut` parameter is compiled with noalias, a `MutSpan` one included (R4) |
| A-5: a temporary lives until the last use of every view made from it | R8 (d) | as in v2 (row 5) |
| A-6: a method that changes its object, called on a temporary, is allowed when its result is used, refused when the change would be lost | R8 (d), E2140 | as in v2 (verdict 13), with both ways out in its message (AN-10) |
| A-7: a view's type says what it allows (`MutSpan` writes, `Span` and `StrView` read; an ECS query says `ref mut T`) | R2, R4, R5 | **v3 withdraws the exception by type** (verdict 34): a `MutSpan` is written only through a name or parameter that says `ref mut` (`fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):`, `d = ref mut tracks[dst_name].as_mut_span()`), and a plain one only reads. The call says `ref mut` before a `MutSpan` argument, as in v2 (verdict 3): `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`; two spans of one list given to one call, one changeable, are proved apart or refused (R4). `Span` and `StrView` read, and an ECS query says `ref mut T`, as before |
| A-8: messages in the program's names, why in one line, the exact line to write | R18 | as in v2 |

## 2. What changed from v2 in this area

| answer | what changed (verdict) | meaning changed? |
|---|---|---|
| AN-01, AN-02, AN-03 | none but "verdict N" for "decision N" in AN-01 and AN-02; no verdict of 21 to 34 reaches their programs (AN-03's `as_mut_span()` is in today's code, the "means exactly" program, which stays) | no |
| AN-04 | `retain`'s first line carries a change list (verdict 21), so the call makes what its test changes; this test changes nothing, so `update` names nothing; one refusal added: a test that changes a module variable makes `update` name it (E2105) | no |
| AN-05 | a plain function type changes nothing at all, captured variables included (verdict 22): a lambda that changes a variable it captured names it and does not fit `on_death`; an `update` that should accept any handler takes a change list (verdict 21) | **yes**, in what `on_death` accepts |
| AN-06 | `World.run` and `run_parallel` run the system they are given, so they count as making its changes (verdicts 21, 30); `reap` changes nothing beyond its query, so nothing is named | no |
| AN-07, AN-08, AN-09, AN-12 | the owner's-test rows of the plain `StrView` parameters `line` and `text` say that the plain parameter reads, where v2 said "read only, by its type" (verdict 32: the word in front of a parameter's name sets what the function may do, for every type) | no |
| AN-10 | "verdict N" for "decision N"; `line`'s owner's-test row as in AN-07 (verdict 32) | no |
| AN-11 | **every `MutSpan` parameter that writes says `ref mut`** (verdict 34): `fn integrate(ref mut pos: MutSpan[float], ...)`, `fn first_even(ref mut xs: MutSpan[int]) -> ref mut int:`; the spec's plain spelling is now refused (E3023); `pos` may now also point the caller's span elsewhere, so v2's refusal of `pos = ...` is gone, and a span of the callee's own list put there is refused (E3063) | **yes**: the parameters take a word, a plain one only reads, and a writing one may also re-point |
| AN-13 | "verdict N" for "decision N" only | no |
| AN-14 | `blend`'s parameter and the locals `d` say `ref mut` (verdict 34): `fn blend(ref mut dst: MutSpan[float], src: Span[float]):`, `d = ref mut tracks[dst_name].as_mut_span()`; a plain `d` given as `ref mut d` is refused (E3027); readability's F-14 is settled | **yes**: a plain name holding a `MutSpan` only reads |
| AN-15 | the world runs the system (verdict 21), as AN-06 | no |
| AN-16 | the lines the messages give, in the v3 spellings (`ref mut` on a writing `MutSpan` name, change lists); readability's F-08 fixed in AN-11's rewritten messages | no |

Verdicts 23, 24, 26, 27, 29, 31 and 33 do not reach this area's programs: no shared owner, arena,
`Option` or link opening, kind test, lock, C call or `.copied()` appears in a NEW block here (`.copied()`
was never used in area A, and phase 0 found no `holds` to replace in this file). Verdict 32 changes no
program here (no `shares mut` parameter), only the wording of the owner's-test rows of plain view
parameters (AN-07 to AN-12, AN-14). Area A has no `scope.spawn`, `thread.spawn` or `@parallel` loop
either: verdict 30 reaches it only through AN-06's `run_parallel`. Every answer keeps v2's spellings where no verdict changed them (`copy of`, `this`,
`changes this` and `changes <field>`, `ref mut` before a `MutSpan` argument).

## 3. The answers

### AN-01: Holding an element while adding to the list

**Meaning changed:** no. No verdict of 21 to 34 reaches this program.

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

fn main():
    es = [Enemy(x=0.0, hp=100)]
    split_boss(ref mut es)
    println(es.len(), es[0].hp)
```

   `boss` is a position (R8 (a)): element 0 of `enemies`, found again after the push. The push grows the
   list, which R8 (a) allows while `boss` is used and which R8 (f) does not count as a use of the
   element. The call says `ref mut es` because `split_boss`'s parameter says `ref mut` (R4).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `enemies` | `ref mut`: the caller's list | `ref mut`: yes |
| `boss` | `ref mut`: element 0 of `enemies`, someone else's (a position, R8 (a)) | `ref mut`: yes, and only `boss` touches that element until its last use (R8 (f)) |
| `es` | none (default: `main`'s own new list) | none (default: its own; `ref mut es` at the call says `split_boss` changes it) |
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
   - `boss = enemies[0]` (plain) then `boss.hp = boss.hp // 2`, whether `Enemy` is `Copy` or not (R2, R5:
     a plain name never changes anything), E3023: "`boss` reads `enemies[0]` and cannot change it; to
     change that enemy through `boss`, write `boss = ref mut enemies[0]`; to change a copy of your own,
     write `boss = copy of enemies[0]`".
   - a change of the element through another name while a plain `boss` is still used (R8 (g),
     verdict 5; run 4 let the plain name see the write):
     ```ember
     boss = enemies[0]
     enemies[0].hp = 1
     println(boss.hp)
     ```
     E3021: "`enemies[0].hp = 1` (line 8) changes the enemy `boss` reads (`boss = enemies[0]`, line 7),
     and `boss` is used on line 9; change it after `boss`'s last use, or write `boss = ref mut enemies[0]`
     and change it through `boss`: `boss.hp = 1`". A copy of your own, `boss = copy of enemies[0]`, is
     restricted by nothing.
   - a write or a read of the element through another name while a `ref mut` `boss` is still used
     (R8 (f), verdict 2):
     ```ember
     boss = ref mut enemies[0]
     enemies[0].hp = 1
     boss.hp -= 1
     ```
     E3022 (E3021 for a read): "`boss` is the only name for `enemies[0]` until line 9, and line 8 changes
     `enemies[0].hp`; change it through `boss`: `boss.hp = 1`".
   - a removal, insertion, reordering, `clear()` or `enemies = [...]` while `boss` is used after it
     (R8 (a)):
     ```ember
     boss = ref mut enemies[0]
     enemies.remove(0)
     boss.hp -= 1
     ```
     E3060: "`enemies.remove(0)` (line 8) may put another enemy in the place `boss` names (line 7), and
     `boss` is used on line 9; use `boss` before line 8, or name the enemy again after it:
     `boss = ref mut enemies[0]`" (a `ref mut` name may be given another ref, verdict 42).
   - a view into a part that a write frees, used after that write (with `name: String` in `Enemy`;
     R8 (b), (g): a change that may reshape what a view points into ends it):
     ```ember
     boss = ref mut enemies[0]
     n = boss.name.trim()
     boss.name = String("x")
     println(n.len())
     ```
     E3060: "`n` is a view of `enemies[0].name`, whose old text line 9 frees, and `n` is used on line 10;
     use `n` before line 9, or keep your own copy: write `n = String(boss.name.trim())` in place of
     line 8".
   - the call without its word (R4), `split_boss(es)`, E3028: "`split_boss` may change `es` (its first
     line says `ref mut enemies: Array[Enemy]`); say so at the call: `split_boss(ref mut es)`".

7. **Cost:** after the push, one reload of `enemies`' buffer pointer, then an indexed store; no bounds
   check (adding never shortens the list). C must reload after a `realloc` too, and Rust's workaround
   (copy the fields out, push, index again) is the same instructions plus a bounds check
   (`tasks/96/redesign/phase3b-containers.md:233-239`, argued there from the emitted code).

8. **Kind:** a smarter compile-time solution (Rust's rule "no add while a reference lives" is not
   needed for adds; it stays for removals and reorders, and for other names of the element, whether the
   element name changes it (R8 (f)) or only reads it (R8 (g))).

### AN-02: A struct element's name used after a call that may change its list

**Meaning changed:** no. `on_hit: fn(ref mut Array[Enemy])` already said what it may change; under
verdict 22 a plain function type changes nothing beyond its `ref mut` parameter, as v2 read it for this
program.

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
    spawn_minion(ref mut enemies, boss.x)
    boss.hp -= 10
```

   `spawn_minion`'s summary is "only adds to `enemies`": it pushes at the end and reads or writes no
   element already there. So the call is growth of the list that holds `boss`'s position: `boss` stays
   valid across it (R8 (a); the buffer is read again after the call), and it is not a use of element 0
   through another name (R8 (f)). The summary is what the body does, worked out; it decides only whether
   the line compiles, never what it does: `boss` names element 0 either way. `boss.x` is a number, copied
   into the parameter `x` before the call (R4), so the call's two arguments do not overlap (R4).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `enemies` (in `spawn_minion`) | `ref mut`: the caller's list | `ref mut`: yes |
| `x` (parameter) | none (default: a number passed in, copied into the parameter, R4) | none (default: read; `x = ...` in the function is refused, R2) |
| `enemies` (in `hit_boss`) | `ref mut`: the caller's list | `ref mut`: yes; `ref mut enemies` at the call says `spawn_minion` changes it |
| `boss` | `ref mut`: element 0 of `enemies` (a position) | `ref mut`: yes, and the only name that touches it until its last use (R8 (f)) |

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
   - a call through a function value given the list, while `boss` is used after it (`on_hit(ref mut
     enemies)` with `on_hit: fn(ref mut Array[Enemy])`; R8 (a), R12), E3060: "`on_hit(ref mut enemies)`
     (line 9) may remove or reorder enemies (its type, `fn(ref mut Array[Enemy])`, lets it change the
     list in any way), which may put another enemy in the place `boss` names (line 7), and `boss` is used
     on line 10; use `boss` before line 9, or name the enemy again after it:
     `boss = ref mut enemies[0]`".
   - a helper the compiler can read that may remove (R8 (a)), E3060: "`kill_weak(ref mut enemies)`
     (line 9) may remove enemies (`kill_weak` line 3: `enemies.remove(i)`), which may put another enemy in
     the place `boss` names (line 7), and `boss` is used on line 10; use `boss` before line 9, or name the enemy again after it:
     `boss = ref mut enemies[0]`".
   - a helper that reads or writes an element already in the list (R8 (f), verdict 2), with
     `fn heal_all(ref mut enemies: Array[Enemy]):` whose body is `for ref mut e in enemies: e.hp += 1`,
     E3022: "`boss` is the only name for `enemies[0]` until line 10, and `heal_all(ref mut enemies)`
     (line 9) changes the enemies in it (`heal_all` line 3: `e.hp += 1`); use `boss` before line 9, or name the enemy again after it:
     `boss = ref mut enemies[0]`".
   - the call without its word (R4), `spawn_minion(enemies, boss.x)`, E3028: "`spawn_minion` may change
     `enemies` (its first line says `ref mut enemies: Array[Enemy]`); say so at the call:
     `spawn_minion(ref mut enemies, boss.x)`".

7. **Cost:** nothing more than AN-01: one reload after the call (the C compiler reloads after any call
   that gets the list anyway).

8. **Kind:** a smarter compile-time solution for code the compiler can read; for a function value whose
   type lets it remove from the list, the understandable limitation stays, with a message that gives the
   line.

### AN-03: Two elements of one list in one call

**Meaning changed:** no. Verdict 34 reaches only the "means exactly" program's `as_mut_span()`, which is
today's Ember and stays.

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
    collide(ref mut bodies[i], ref mut bodies[j])           # one compare: i != j

fn all_pairs(ref mut bodies: Array[Body]):
    for i in 0..bodies.len():
        for j in i + 1..bodies.len():
            collide(ref mut bodies[i], ref mut bodies[j])   # proved: j > i, nothing runs
```

   R4: `collide` says "two bodies, both changed" with two `ref mut` names, and the call says the same
   words before each argument (the owner, 2026-10-25: "during a function call saying bump(ref mut a)
   seems reasonable and okay it tell you that the value is going to be touched"). Two arguments that may
   be changed may not name one place: in `all_pairs` the positions differ by construction (`j` from
   `i + 1..`); in `step` they differ only by an index, so one compare before the call stops the program
   if `i == j`. `Body` holds no link, so both parameters are compiled with noalias (R4).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `a`, `b` | `ref mut`: a body of the caller's | `ref mut`: yes |
| `t` | none (default: a number part, copied into a value of its own, R2) | none (default: its own; changes nothing else) |
| `bodies` (in `step`, in `all_pairs`) | `ref mut`: the caller's list | `ref mut`: yes; `ref mut bodies[i]` at the call says `collide` changes that body |
| `i`, `j` (parameters of `step`) | none (default: numbers passed in, copied) | none (default: read) |
| `i`, `j` (loop names in `all_pairs`) | none (default: the loop's own numbers) | none (default: only the loop gives them their values) |
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
   - the same position twice, known when compiling (`collide(ref mut bodies[i], ref mut bodies[i])`;
     R4), E3022: "`collide` changes `a` and `b` as two different bodies, and both are `bodies[i]`; pass
     two different bodies: `collide(ref mut bodies[i], ref mut bodies[j])`".
   - at run time, where not proved and `i == j` (a stop, not a refusal): "`collide` was given
     `bodies[3]` as both `a` and `b`; test `if i != j:` before the call to skip it".
   - the call without its words (R4), `collide(bodies[i], bodies[j])`, E3028: "`collide` may change both
     bodies (its first line says `ref mut a: Body, ref mut b: Body`); say so at the call:
     `collide(ref mut bodies[i], ref mut bodies[j])`".

7. **Cost:** where proved, nothing. Elsewhere one compare and a branch never taken. Run 2 measured the
   inserted compare at no cost (89.9 ms with it, 94.9 ms without it and without noalias; safe Rust's
   `get_disjoint_mut` 87.6 ms: `tasks/96/redesign/phase3b-containers.md:301-306`, from
   `target/redesign2/containers/bench/runs/a3.txt`). Noalias is what matters for speed: without it,
   two spans of one list took 1.53x (`phase3b-containers.md:351-352`, `runs/a3_span.txt`). R4 compiles
   every `ref mut` parameter with noalias.

8. **Kind:** a readable spelling of Rust's guarantee (two `&mut` never alias), with the compile-time
   proof that Rust's borrow checker does not make for computed positions.

### AN-04: Changing a list while looping over it

**Meaning changed:** no. `retain`'s first line now carries a change list (verdict 21), so the call makes
what its test changes; this program's test changes nothing. One refusal is added in point 6.

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

3. **NEW** (unchanged from run 3):

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

   R8 (c), as agreed: the list does not change while the loop walks it; the removal and the addition go
   after it. `retain` and `extend` are methods called on `enemies`, so they carry no word (R4: the object
   a method is called on is not marked; their first lines say they change the list). `retain` runs the
   function it is given, now, on every element, so the call makes what that function changes (verdict
   21, R12): the library writes `pub fn retain[C, F: fn(T) changes C -> bool](owns keep: F) changes
   this, C:` (vocabulary, the iterator row), and `C` is filled from the function given. `fn(e) =>
   e.hp > 0` changes nothing, so `C` is empty, the call changes only `enemies`, and `update`'s first line
   names nothing beyond its `ref mut enemies` (point 6 shows a test that changes something).

   The speed part: the compiler runs the loop and the `retain` after it in one pass when (1) the `retain`
   follows the loop on the same list with nothing between that touches the list, (2) the loop has no
   `break` or `return`, (3) the `retain` test reads only its element and values the loop does not
   change, (4) the loop body does not read the list by position (`enemies[k]`), and (5) removing an
   element runs no code of its own that the program could see (freeing memory only). Each condition is
   decided when compiling; when one fails, the two passes run as written. Fusing changes no result any
   code can observe: the loop sees no removal, and the `retain` sees every element after the loop's
   write to it.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `enemies` | `ref mut`: the caller's list | `ref mut`: yes (after the loop) |
| `spawned` | none (default: the function's own new list) | none (default: its own; changing it changes nothing anyone else holds) |
| `e` (loop name) | `ref mut`: the current element of `enemies` | `ref mut`: yes, the only name for it during its round (R8 (f)) |
| `e` (the `retain` test's parameter) | none (default: a lambda parameter with no word reads each element, R4) | none (default: read only) |
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
   - `enemies.push(...)` in the loop (R8 (c)), E3020: "`enemies.push(...)` adds to `enemies`, which this
     loop walks (line 7); collect the new enemies in a list of your own: write `spawned: Array[Enemy] = []`
     before the loop, `spawned.push(Enemy(hp=5, splits=false))` in place of this line, and
     `enemies.extend(spawned)` after the loop".
   - `enemies.remove(i)` in the loop (R8 (c)), E3020: "`enemies.remove(i)` shortens `enemies`, which this
     loop walks (line 7); remove the dead after the loop: `enemies.retain(fn(e) => e.hp > 0)`".
   - `for e in enemies:` then `e.hp -= 1` (R5: a plain loop name never changes anything), E3023: "`e`
     reads each enemy and cannot change it; to change the enemies through `e`, write
     `for ref mut e in enemies:`".
   - `for e in enemies.iter_mut():` then `e.hp -= 1`: the same (`e` is a plain loop name), E3023: "`e` is
     a plain loop name, which only reads; write `for ref mut e in enemies:`" (vocabulary, `iter_mut`: the
     one `for` spelling that changes elements is `for ref mut e in enemies:`).
   - a `retain` test that also changes a module variable, while `update`'s first line does not name it
     (verdict 21; R5, R12). With `static mut LOG: Array[String] = []` and this test:
     ```ember
     fn keep_alive(e: Enemy) changes LOG -> bool:
         if e.hp <= 0:
             LOG.push(String("an enemy died"))
         return e.hp > 0
     ```
     `enemies.retain(keep_alive)` in place of the NEW's `retain` line is a call that changes `LOG` as well
     as `enemies`, so `update` names it. Without the word, E2105: "`update` calls
     `enemies.retain(keep_alive)`, which runs `keep_alive` on every enemy, and `keep_alive` changes `LOG`
     (its first line says `changes LOG`); add it to `update`'s first line: `fn update(ref mut enemies:
     Array[Enemy]) changes LOG:`". With the word, a call of `update` inside a loop over `LOG` is refused
     (E3020), as for any function that changes `LOG`. A lambda test that pushes to `LOG` without saying so
     is refused on its own first line (E2105, as the rules' `for_each` message). In the test, `e` is a
     plain parameter (default: it reads the enemy `retain` gives it) and `LOG` is `static mut` (a module
     variable that may change, R15).

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

### AN-05: A loop that removes from its own list while calling code the compiler cannot see

**Meaning changed:** yes, in what `on_death` accepts (verdict 22). The NEW program does what it did.
v2 let `on_death: fn(ref mut Array[Enemy], int)` hold a lambda that changes a variable it captured
(that variable counted as part of the function itself). In v3 a plain function type changes nothing at
all beyond its `ref mut` parameters, so such a lambda names the variable on its first line and is refused
where it is given to `on_death` (E2113, point 6). An `update` meant to take any handler says so with a change list (verdict 21), shown in
point 3.

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
        on_death(ref mut enemies, dead.pop().unwrap())
```

   R8 (c): the handler runs after the loop, last position first, so each removal leaves the positions
   still to come where they were. The call through the function value says `ref mut enemies`, because
   the callable type says `ref mut` for that parameter (R4: "a callable type writes the same words").

   What this does, stated, so the text says everything the program does:
   - **Order:** the handlers run for the last dead enemy first, the reverse of the OLD counting loop,
     which called them first-dead-first during the loop. Last-first is what keeps the positions right:
     a handler that removes the enemy it is given, or adds enemies at the end, moves no position still
     to come. A game whose handlers must run in the OLD order (a log, a score shown in order) walks
     `dead` from the front and does not let the handler remove: `for i in dead: on_death(enemies, i)`
     with `on_death: fn(Array[Enemy], int)` (read-only, so the call has no word), then
     `enemies.retain(fn(e) => e.hp > 0)` once, the agreed "delete after the loop".
   - **A handler that removes another enemy** (a neighbour, the first one): the positions after the one
     it removed shift down, so a later position in `dead` names a different enemy, or is past the end.
     The second is the index check of every `enemies[i]` (a stop with the index and the length, never a
     memory error); the first the compiler cannot see, because the handler is a function given in. The
     handler's type allows it (`ref mut Array[Enemy]`), so this is the program's own contract: a handler
     given a position removes at most that enemy. A handler that must remove others queues them in a list
     and lets `update` remove them after the calls, as above.
   - **The handler itself is only called**, through a read-only parameter, so it must be a function that
     changes no state of its own (R12): a plain function such as `drop_it`, or a lambda that changes only
     the list it is given. A lambda that changes its own copy (`with copy of kills`) is refused at the
     call that gives it to `update` (point 6).
   - **What the handler may change**: its type, `fn(ref mut Array[Enemy], int)`, says it changes the list
     it is given and nothing else at all (a change through a `ref mut` parameter needs no word, and a plain
     function type changes nothing beyond it, the variables a lambda captured included: verdict 22). So
     `update`'s first line names nothing beyond its `ref mut enemies`. A handler that changes anything
     else says so on its first line, and so does not fit this type (E2113): a function that changes a
     module variable (`fn drop_and_log(ref mut list: Array[Enemy], i: int) changes LOG:`), or a lambda
     that changes a variable it captured (`fn(ref mut list: Array[Enemy], i: int) changes kills: kills
     += 1`, point 6). An `update` meant to take any handler says so with a change list (verdict 21,
     R12), as the library's `for_each` does, and keeps the same body:
     ```ember
     fn update[C](ref mut enemies: Array[Enemy], on_death: fn(ref mut Array[Enemy], int) changes C) changes C:
     ```
     `C` is filled from the handler each call is given: `update(ref mut es, drop_and_log)` changes `LOG`,
     and its caller names `LOG` on its own first line (R5); `update(ref mut es, drop_it)` changes nothing
     but `es`. A call of the first inside a loop over `LOG` is refused (E3020). This is the owner's sketch
     of verdict 21 (`f: fn(T) changes C`), a function value called through its two words, as `on_death`
     already is; the library writes the same contract as a bound, one instance per function
     (`V3/changes.md` section 11, question 4).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `enemies` | `ref mut`: the caller's list | `ref mut`: yes; `ref mut enemies` at the call says the handler may change it |
| `on_death` | its type `fn(ref mut Array[Enemy], int)`: a function given by the caller, read only (a plain parameter) | none (default: only called; its type's `ref mut` says the function may change the list it is given, and nothing else at all, verdict 22) |
| `dead` | none (default: the function's own new list) | none (default: its own) |
| `i` | none (default: the loop's own number) | none (default: only the loop gives it its values) |

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
   - `on_death(ref mut enemies, i)` inside `for ref mut e in enemies:` (R8 (c)), E3020: "`enemies` may not
     change while this loop walks it (line 6), and `on_death` may change it (its type takes
     `ref mut Array[Enemy]`); collect the positions and call it after the loop: write
     `dead: Array[int] = []` before the loop, `dead.push(i)` in place of this line, and after the loop
     `while dead.len() > 0: on_death(ref mut enemies, dead.pop().unwrap())`".
   - the same call inside `for i in 0..enemies.len():`, accepted today (R8 (c): a counting loop may add
     but not remove), E3020: "this loop counts over `enemies`' length with an end fixed when it started
     (line 6), and `on_death(ref mut enemies, i)` may remove from `enemies`, which would skip the next
     enemy and then run past the end; collect the positions and call it after the loop: write
     `dead: Array[int] = []` before the loop, `dead.push(i)` in place of this line, and after the loop
     `while dead.len() > 0: on_death(ref mut enemies, dead.pop().unwrap())`".
   - a handler that changes its own state, given to the read-only `on_death` (R12), for example
     `count = fn(ref mut list: Array[Enemy], i: int) with copy of kills: kills += 1` then
     `update(ref mut es, count)`, E3023: "`count` changes its own copy of `kills` each time it runs, and
     `update` takes it read-only; write `ref mut on_death: fn(ref mut Array[Enemy], int)` in `update`'s
     first line and call `update(ref mut es, ref mut count)`".
   - a lambda that changes a variable it captured, given to `on_death` (R12, verdict 22), in `main`:
     ```ember
     kills = 0
     update(ref mut es, fn(ref mut list: Array[Enemy], i: int) changes kills: kills += 1)
     ```
     E2113: "this function changes `kills`, a variable it uses from `main` (`fn(ref mut list:
     Array[Enemy], i: int) changes kills`), and `update`'s `on_death` takes `fn(ref mut Array[Enemy],
     int)`, a function that changes nothing but the list it is given; to let `update` take handlers that
     change something, write its first line as `fn update[C](ref mut enemies: Array[Enemy], on_death:
     fn(ref mut Array[Enemy], int) changes C) changes C:`". Here `kills` is `main`'s own number (no word:
     a local of its own), `list` is `ref mut` (the list `update` gives) and `i` a plain parameter (a
     number passed in). The same lambda without `changes kills` is refused on its own first line (E2105:
     "this function changes `kills`, a variable it uses from `main`; say so on its first line:
     `fn(ref mut list: Array[Enemy], i: int) changes kills: kills += 1`").

7. **Cost:** one `push` per death into `dead`, and the handler calls after the loop. C calling a
   handler it does not know must do the same to stay correct (it cannot know what the handler
   removed).

8. **Kind:** an understandable limitation kept (agreed: the list may not change while a loop walks it);
   one run-time stop moved to compile time (the counting loop).

### AN-06: ECS: creating and destroying entities inside a view loop

**Meaning changed:** no. `World.run` and `run_parallel` run the system they are given, so under verdicts
21 and 30 they count as making what the system's type says it changes; `reap` changes nothing beyond the
query it is given, so nothing is named (point 3).

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

3. **NEW** (unchanged from run 3):

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
   `commands.entity(e).despawn()`). `Commands.create()` hands out a real entity id at once, from the
   world's free indexes or past its highest index, so `add(n, ...)` can name it; the entity and its
   components exist once the queue is applied. The query parameter is `ref mut q`: the world builds the
   query and the system changes it by walking it and queueing on it. `World.run`'s system type becomes
   `fn(ref mut Query[Q], A)`, and `World.run` calls the system with `ref mut` before the query, as every
   call to a `ref mut` parameter says it (R4).

   Under R8: `q.commands()` changes only the query's queue of changes (its first line names that field,
   `changes cmds`), which the loop does not walk, so the loop may call it (R8 (c)); and it neither reads
   nor changes a `Health` value: the query's fields hold a view of the world's pools, and reading that
   view is not reading what it points at (`V3/rules.md` section 0, "into X": what `q` owns, and `q` does
   not own the pools its view points at). So `h`, the only name for its `Health` while it is used
   (R8 (f), verdict 2: reads kept out too), is not reached. In the loop's names, plain `e` comes first
   and `ref mut h` after it, which R3's list rule allows (only a name with no word after a name with a
   word is refused).

   `World.run` runs the system it is given, so the call counts as making what the system's type says it
   changes (verdict 21, R12); `run_parallel` hands systems to the job system's threads, which counts the
   same way (verdict 30: "starting a thread counts", R12, R15). `reap`'s type is `fn(ref mut
   Query[(Entity, ref mut Health)], float)`: it changes the query it is given and nothing else, and the
   query's changes are applied by the world after the system returns, so a program that calls
   `world.run(reap, 0.016)` names nothing on its first line for it. A system that also changed a module
   variable would say `changes LOG` on its first line, and the function that runs it would name `LOG`
   too.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `q` | `ref mut`: the query the world made for this run | `ref mut`: yes (walks it, queues on it) |
| `e` | none (default: a plain loop name, a read-only view of the current entity's id) | none (default: read only; an id, so changing it would change no entity anyway) |
| `h` | `ref mut`: the `Health` of `e` in the world (the query says `ref mut Health`) | `ref mut`: yes |
| `dt` | none (default: a number passed in) | none (default: read) |
| `n` | none (default: the new entity's id that `create()` returns, a value of its own) | none (default: its own id) |
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

   (The C file's comment names `World.run`'s `self.apply(q.cmds)`: that is today's `std/src/ecs.em`
   spelling; under R5 it is `this.apply(q.cmds)`. The C is unchanged.)

6. **Still refused:**
   - `Mut[Health]` in the query (a removed word), E1010: "`Mut[Health]` is now `ref mut Health`: write
     `Query[(Entity, ref mut Health)]`".
   - changing a pool while a loop walks it: there is no operation for it inside a system (the system
     holds only its query); run 2's in-loop destroy with a cursor is not built.
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

### AN-07: A local list of string views (`words = text.split()`)

**Meaning changed:** no. Only the owner's-test row of the plain parameter `line` is reworded (verdict
32: a plain parameter reads, whatever its type).

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

3. **NEW** (unchanged from run 3):

```ember
fn count_words(line: StrView) -> int:
    words = line.to_lower().split_whitespace().collect()
    return words.len()
```

   R8 (d) for a list: the lowered text is kept until the last use of `words`, whose elements are views
   into it (Rust's `Vec<&str>` with the lifetime inferred, plus row 5's rule for the temporary, which
   Rust refuses: `E0716`, `phase3b-containers.md:562-567`). `collect()` with nothing that names the
   result type makes an `Array` of the items, here `Array[StrView]`, as Python's `split()` makes a list.
   The list is a new value that `collect()` hands to `words`, which owns it (R2: a call that makes a new
   value, a list of views included, hands it to the name); because it holds views, R7 bounds its life:
   no longer than the lowered text. `words` may therefore be changed like any list of its own
   (`words.push(...)` of a view of text that lives as long), and nothing outside it is.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `line` | `StrView`: a view of the caller's text (a plain parameter) | none (default: a plain parameter reads, R4, verdict 32) |
| `words` | none (default: its own new list, which `collect()` hands to it (R2), of views into the lowered copy of `line`; the list lives no longer than that copy (R7, R8 (d)); its type `Array[StrView]` comes from its value, as row 5's `word`) | none (default: its own list; its views only read, so changing it changes nothing anyone else holds) |

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
   - returning the list (`fn words_of(line: StrView) -> Array[StrView]:` ... `return words`; R8 (d)),
     E3063: "`words` points into the lowered copy of `line` that `line.to_lower()` made on line 2, which
     ends when `words_of` returns; return copies of your own: write the first line as
     `fn words_of(line: StrView) -> Array[String]:` and, in place of line 2, `words: Array[String] = []`
     then `for w in line.to_lower().split_whitespace(): words.push(String(w))`; or lower the text in the
     caller and return views of it:
     `fn words_of(text: StrView) -> Array[StrView] from text:`".
   - keeping a word where it outlives its text (`all_words.push(w)` with `all_words` made before a loop
     over lines; R8 (d)), E3063: "`w` points into the lowered copy of `line`, which ends with this loop
     step, and `all_words` lives longer; push your own copy: `all_words.push(String(w))`".
   - `copy of w` of a view (R3), E2116: "`w` is a view of `line`'s text; to make your own copy write
     `String(w)`".

7. **Cost:** no copy per word: a view is a pointer and a length. Last run measured views against the
   alternatives on 32 MiB of words: views 1.00, safe Rust `Vec<&str>` 1.05, one allocation per word
   (today's fix-it) 2.34x (`tasks/96/redesign/phase3b-containers.md:639-646`, `runs/a25.txt`).

8. **Kind:** a smarter compile-time solution (the lifetime and the temporary's life are inferred).

### AN-08: A view of a temporary kept past its statement

**Meaning changed:** no (row 5, exactly as agreed). Only the owner's-test row of the plain parameter
`line` is reworded (verdict 32).

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

3. **NEW** (row 5, exactly as agreed; unchanged from run 3):

```ember
fn first_word_len(line: StrView) -> int:
    word = line.to_lower().trim()
    return word.len()
```

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `line` | `StrView`: a view of the caller's text (a plain parameter) | none (default: a plain parameter reads, R4, verdict 32) |
| `word` | none (default: a name bound to a call that gives a view is a read-only view, R2: here of the lowered copy of `line`, which is kept until `word`'s last use, R8 (d), row 5) | none (default: read only) |

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

6. **Still refused:** returning `word` (AN-09, E3063); storing it where it outlives the lowered text
   (AN-07's second message, with `word` for `w`, E3063).

7. **Cost:** none: the lowered string is freed after `word`'s last use instead of at the end of the
   statement.

8. **Kind:** a smarter compile-time solution (agreed row 5).

### AN-09: Returning a view of a temporary

**Meaning changed:** no (row 5, exactly as agreed). Only the owner's-test row of the plain parameter
`line` is reworded (verdict 32).

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
   names; unchanged from run 3):

```ember
fn clean(line: StrView) -> String:
    word = line.to_lower().trim()
    return String(word)
```

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `line` | `StrView`: a view of the caller's text (a plain parameter) | none (default: a plain parameter reads, R4, verdict 32) |
| `word` | none (default: a read-only view of the lowered copy of `line`, R2, row 5) | none (default: read only) |

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

6. **Still refused:** `fn clean(line: StrView) -> StrView:` with `return word` (R8 (d)), E3063: "`word`
   is a view of the lowered copy that `line.to_lower()` made on line 2, which ends when `clean` returns;
   return `String(...)` to hand back your own copy: `return String(word)`, with the result type
   `String`". (The agreed sentence is inside it; the message names `word` and `line.to_lower()`, never
   `_5`.)

7. **Cost:** one allocation for the copy, as Python's `strip()` makes a new string. The compiler can
   save it: when the view points into a local `String` that ends at this `return` (here the lowered
   copy), `String(word)` takes over that buffer and moves the trimmed text to its start (one memory
   move) instead of allocating, which is what a C programmer writes by hand.

8. **Kind:** an understandable limitation kept (agreed row 5), with its message and the constructor
   it names.

### AN-10: A method that changes a temporary (`line.split(" ").next()`)

**Meaning changed:** no. Only the owner's-test row of the plain parameter `line` is reworded (verdict
32).

1. **Intent:** take the first word of a line in one expression; and, in the second program, heal the
   first enemy of a list.

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

   `next` changes the temporary iterator (its first line is `fn next() changes this -> Option[StrView]:`,
   R5) and returns the first word, which is used; the iterator ends with the statement. R8 (d) refuses a
   change to a temporary only when the call's only effect is that change and nothing it gives back is
   used; here the word is used, so this is accepted, as in Rust (verdict 13: "a call whose result is used
   (`line.split(" ").next()`) stays allowed").

   The second program heals the first enemy of a list, written so that the change reaches the enemy the
   program means, in each of the two ways verdict 13 names, and heals a copy of your own beside them:

```ember
@derive(Copy)
struct Enemy:
    hp: int

    fn heal() changes hp:
        this.hp += 10

fn enemy_at(ref mut xs: Array[Enemy], i: int) -> ref mut Enemy:
    return ref mut xs[i]

fn main():
    es = [Enemy(hp=1)]
    es[0].heal()                      # heals the enemy in es
    enemy_at(ref mut es, 0).heal()    # the same enemy: enemy_at gives back es[0] itself
    e = copy of es[0]
    e.heal()                          # heals a copy of your own
    println(es[0].hp, e.hp)           # 21 31
```

   `heal`'s first line says what it does to the enemy it is called on: `changes hp` (R5; there is no
   `self`, and the object is `this` inside the method). A method call carries no word: the object is
   before the dot (R4). `es[0].heal()` heals the enemy in the list. `enemy_at` gives back the element
   itself: its result type `ref mut Enemy` makes the call a place (R6), and the call says `ref mut es`
   because the parameter says `ref mut` (R4); so `enemy_at(ref mut es, 0).heal()` heals `es[0]` too.
   `e = copy of es[0]` makes a copy of your own, which `e.heal()` heals.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `line` | `StrView`: a view of the caller's text (a plain parameter) | none (default: a plain parameter reads, R4, verdict 32) |
| `this` (in `heal`) | `this`: the enemy `heal` was called on | `changes hp`: it may change `hp` only |
| `xs` (in `enemy_at`) | `ref mut`: the caller's list | `ref mut`: yes; the result `ref mut Enemy` gives back one of its elements, which the caller may change |
| `i` (in `enemy_at`) | none (default: a number passed in, copied) | none (default: read) |
| `es` | none (default: `main`'s own new list) | none (default: its own; `heal`'s first line says it changes `es[0].hp`, and `ref mut es` at the call says `enemy_at` may change it) |
| `e` | `copy of`: a copy of your own | none (default: its own copy; changing it changes no enemy in `es`) |
| `hp` (field) | none (default: a value stored in each `Enemy`) | none (default: through a name that may change the enemy, or `this` in a method that says `changes hp`) |

   The temporary iterator in `first_word` has no name; `next`'s own first line (`changes this`) says it
   changes it.

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

   The second program in today's spelling (`fn heal(mut self)`, a `mut` parameter, `ref mut` result; a
   plain `e = es[0]` of a `Copy` struct copies today):

`target/redesign4/v2-02-area-A/means/m10v2_heal.em`:

```ember
# AN-10 v2 means exactly: heal es[0] in place, through a function that gives back es[0] itself, and a copy held in a local.
@derive(Copy)
struct Enemy:
    hp: int

    fn heal(mut self):
        self.hp += 10

fn enemy_at(mut xs: Array[Enemy], i: int) -> ref mut Enemy:
    return ref mut xs[i]

fn main():
    es = [Enemy(hp=1)]
    es[0].heal()
    enemy_at(es, 0).heal()
    e = es[0]
    e.heal()
    println(es[0].hp, e.hp)
```

`target/redesign4/bin/ember.exe check m10v2_heal.em`:

```text
exit=0
```

6. **Still refused:**
   - a change that would be lost (R8 (d), verdict 13): `enemy_at(es, 0).heal()`, where `enemy_at` is
     `fn enemy_at(xs: Array[Enemy], i: int) -> Enemy:` with `return xs[i]` (a copy) and nothing the call
     gives back is used, E2140: "`heal` changes the enemy that `enemy_at(es, 0)` returns, a copy that ends
     on this line, so the change would be lost; to heal the enemy in `es` write `es[0].heal()`, or let
     `enemy_at` give back the enemy itself: `fn enemy_at(ref mut xs: Array[Enemy], i: int) -> ref mut
     Enemy:` with `return ref mut xs[i]`, called `enemy_at(ref mut es, 0).heal()`". (Today's compiler
     refuses it with E2140 and no help, pasted in point 2.)
   - today's spelling `fn heal(mut self):` (R5), E1010: "`self` is not a word of Ember: inside a method
     its object is `this`; write `fn heal() changes hp:` for a method that changes `hp`, and
     `this.hp += 10` in its body".
   - calling it through a name that only reads (R5), `for p in es: p.heal()`, E3023: "`heal` changes the
     enemy (its first line says `changes hp`), and `p` only reads it; write `for ref mut p in es:`".
   - `enemy_at(es, 0).heal()` with the `ref mut` version of `enemy_at` (R4), E3028: "`enemy_at` may change
     `es` (its first line says `ref mut xs: Array[Enemy]`); say so at the call:
     `enemy_at(ref mut es, 0).heal()`".

7. **Cost:** none; the same code as the local, and as today's `ref mut` result.

8. **Kind:** a smarter compile-time solution: a call on a temporary is accepted when what it gives back
   is used, and a change that would be lost is refused with both ways out (verdict 13).

### AN-11: A `MutSpan` parameter written without `mut`

**Meaning changed:** yes (verdict 34). v2 kept the spec's plain `pos: MutSpan[float]` and let it write
the elements (R5's exception by type). In v3 the exception is withdrawn: a `MutSpan` parameter that
writes says `ref mut` on its name, `fn integrate(ref mut pos: MutSpan[float], vel: Span[float],
dt: float):`, and a plain one only reads (the spec's spelling is refused, E3023). A `ref mut` parameter
names the caller's place, so `pos` may now also point the caller's span elsewhere (R2, R4): v2's refusal
of `pos = ...` is gone, and putting a span of the callee's own list there is refused instead (E3063).
The calls are v2's.

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

3. **NEW** (the spec's two examples, with `ref mut` on each `MutSpan` parameter that writes, verdict 34,
   and their callers):

```ember
fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn first_even(ref mut xs: MutSpan[int]) -> ref mut int:
    for i in 0..xs.len():
        if xs[i] % 2 == 0:
            return ref mut xs[i]
    xs[0] = 0
    return ref mut xs[0]

fn main():
    p = [0.0, 1.0]
    v = [1.0, 1.0]
    integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)
    a = [1, 2, 3]
    r = ref mut first_even(ref mut a.as_mut_span())
    r = 7
    println(p[0], a[1])
```

   **A `MutSpan` parameter that writes says `ref mut`** (verdict 34: "a `MutSpan` name says `ref mut` in
   front, like every other name that may change something it does not own"; R4, R5). The word on the
   name sets what the function may do with the parameter, for every type (R4, verdict 32); the type
   after the colon, `MutSpan[float]`, says what the caller's value is, a view of elements someone else
   owns. So `ref mut pos` may write those elements (`pos[i] += ...`), and so may `ref mut xs`, which also
   gives one of them back as a place (`return ref mut xs[i]`). As every `ref mut` parameter, it names the
   caller's place, here the caller's span: `pos` may also point that span elsewhere (R2, R4), and where
   the span is made in the call, as here, that changes only the span made for the call, which ends with
   the line (R4). **The call says `ref mut` before a `MutSpan` argument** (R4, verdict 3): before a
   `MutSpan` the word names its elements, so it stands before the span made in the call,
   `ref mut p.as_mut_span()`. `r = ref mut first_even(ref mut a.as_mut_span())` names the element the
   call gives back (a call whose result type is `ref mut int` is a place, R6), and `r = 7` writes it:
   `a[1]` becomes 7.

   Inside `integrate`, `pos` is the only name for its elements while it is used (R8 (f)), and `pos` and
   `vel` are compiled with noalias (R4): every call proves its arguments apart. `p` and `v` are two
   different lists, so the compiler knows it with nothing to run. Two spans of one list given to one call
   (`integrate(ref mut p.as_mut_span(), p.as_span(), 0.5)`) are refused (point 6); run 4 accepted that
   call for numbers and compiled it without noalias.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `pos` | `ref mut`: the caller's place, a `MutSpan` (a view of elements the caller owns, here the elements of `p`) | `ref mut`: yes; it writes those elements and is their only name while it is used (R8 (f)), and it may point the caller's span elsewhere (R2, R4) |
| `vel` | `Span`: a view of the caller's elements (a plain parameter) | none (default: a plain parameter reads; they do not change while `vel` is used, R8 (g)) |
| `dt` | none (default: a number passed in) | none (default: read) |
| `i` (both functions) | none (default: the loop's own number) | none (default: only the loop gives it its values) |
| `xs` | `ref mut`: the caller's place, a `MutSpan` (a view of the elements of `a`) | `ref mut`: yes; it writes those elements and gives one back as a place (`-> ref mut int`), and may point the caller's span elsewhere |
| `p`, `v`, `a` | none (default: `main`'s own new lists) | none (default: their own; `ref mut` at a call says the callee writes their elements) |
| `r` | `ref mut`: the element `first_even` gives back (`a[1]` here) | `ref mut`: yes; `r = 7` writes that element (R2) |

   In `ref mut pos: MutSpan[float]` the line says "mut" twice, as the owner noted for verdict 34: `ref
   mut` on the name says what `integrate` may do, `MutSpan` in the type says what kind of view the caller
   gives.

5. **Means exactly:** today's `mut pos` / `mut xs`, which is the NEW `ref mut pos` / `ref mut xs`
   (today's `mut` on a parameter is the in-out parameter that v3 spells `ref mut`, vocabulary `mut x: T`
   row). Accepted:

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

   Today's `r = first_even(a.as_mut_span())` then `r = 7` writes through `r`, because today a name bound
   to a `ref mut` result writes through it. Under R2 and R6 the NEW line says so:
   `r = ref mut first_even(ref mut a.as_mut_span())`, and plain `r = first_even(...)` copies the number,
   as `n = counts[0]` does (R2).

6. **Still refused** (line numbers count the NEW program as a file whose first line is a comment, so
   `fn integrate` is line 2 and `fn first_even` line 6):
   - the spec's own spelling, a plain `MutSpan` parameter that writes (verdict 34; R4, R5: a plain name
     never changes anything), `fn integrate(pos: MutSpan[float], vel: Span[float], dt: float):` with
     `pos[i] += vel[i] * dt`, E3023: "`pos[i] += vel[i] * dt` (line 4) writes an element of the caller's
     span, but `pos` only reads it; write `fn integrate(ref mut pos: MutSpan[float], vel: Span[float],
     dt: float):`, and the call stays `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`". And
     `fn first_even(xs: MutSpan[int]) -> ref mut int:`, E3023 at its first such line: "`return ref mut
     xs[i]` (line 9) gives back an element of the caller's span to be changed, but `xs` only reads it;
     write `fn first_even(ref mut xs: MutSpan[int]) -> ref mut int:`, and the call stays
     `first_even(ref mut a.as_mut_span())`". One message for each function, at the program's line: the
     later writes in the same body (`xs[0] = 0`, `return ref mut xs[0]`) are not reported again. Today's
     compiler refuses the same lines with E3023 too (point 2, three messages for `first_even`), with
     helps that give `mut xs: MutSpan[i64]`, `RefCell` and `class`; this message gives the line, with the
     type as written (`int`, not `i64`).
   - the call without its word (R4, verdict 3), `integrate(p.as_mut_span(), v.as_span(), 0.5)`, E3028:
     "`integrate` changes the elements of `p` (its first line says `ref mut pos: MutSpan[float]`); say so
     at the call: `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`".
   - two spans of one list given to one call, one of them changeable (R4), `integrate(ref mut
     p.as_mut_span(), p.as_span(), 0.5)`, E3021: "`integrate` changes the elements of `p` through `pos`
     while `vel` shows the same elements; give `vel` a copy of its own: instead of this line write
     `old = copy of p` and `integrate(ref mut p.as_mut_span(), old.as_span(), 0.5)`". (Run 4 accepted
     it; R4 refuses any overlap it cannot prove away, numbers included. Two spans that may or may not
     overlap are proved apart with `assert_disjoint`, AN-14.)
   - `mut pos: MutSpan[float]` (`mut` alone on a name; today's spelling, point 2), E0113: "`mut pos` says
     `pos` may change but not whose elements; to write the caller's elements through `pos`, write
     `fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):`".
   - writing through `vel: Span[float]` (R4, R5), E3023: "`vel[i] = 0.0` writes an element the caller
     gave as a `Span`, which only reads, and `vel` is a plain parameter; to write the caller's velocities,
     write `fn integrate(ref mut pos: MutSpan[float], ref mut vel: MutSpan[float], dt: float):` and call
     it `integrate(ref mut p.as_mut_span(), ref mut v.as_mut_span(), 0.5)`". No `RefCell` or `class`
     help; the type as written (`float`, not `f64`).
   - pointing the caller's span at the callee's own list (R2, R4), with these two lines at the top of
     `integrate`'s body (lines 3 and 4):
     ```ember
     mine = [0.0, 0.0]
     pos = ref mut mine.as_mut_span()
     ```
     E3063: "`pos = ref mut mine.as_mut_span()` (line 4) would point the caller's span at `mine`, which
     ends when `integrate` returns, while the caller's span lives on; to write `mine`'s numbers into the
     caller's elements, write `for i in 0..pos.len(): pos[i] = mine[i]` instead of this line". `pos` may
     be pointed only at a span the caller gave, or a part of one (R4).

7. **Cost:** none beyond today's `mut pos`, and the loop runs with noalias: `pos` and `vel` are proved
   apart at every call (R4), here with nothing to run (two different lists). Noalias is what made two
   spans of one list run at 1.00x to 1.07x of C against 1.53x without it
   (`tasks/96/redesign/phase3b-containers.md:346-355`); run 4 compiled an overlapping call without it.
   One possible cost is new since v2, not measured: a `ref mut` parameter may point the caller's span
   elsewhere, so the span may have to be passed by its address, where v2's plain `pos` could be passed as
   its two words. Inside a package the compiler sees whether the body ever gives `pos` another span
   (`integrate` and `first_even` never do) and can pass the two words as before; with noalias the loop
   reads the span's pointer and length once before it starts either way. Across a package's edge no
   word or recorded fact says it (found for the lead, section 5).

8. **Kind:** a readable spelling: the name says `ref mut`, as every name that may change something it
   does not own (verdict 34), the type says what the view is, and the call says `ref mut` as every call
   that changes an argument does (verdict 3).

### AN-12: Updating a map entry in place

**Meaning changed:** no. Only the owner's-test row of the plain parameter `text` is reworded (verdict
32).

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

3. **NEW** (unchanged from run 3):

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
   sides name the same `m` and the same `k` and nothing between them changes `m`. The loop walks the
   words of `text`, not `counts`, so changing `counts` in it is allowed (R8 (c)).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `text` | `StrView`: a view of the caller's text (a plain parameter) | none (default: a plain parameter reads, R4, verdict 32) |
| `counts` | none (default: the function's own new map; returned, so handed to the caller, row 9) | none (default: its own) |
| `w` | none (default: a plain `for` name, a read-only view of one word of `text`) | none (default: read only) |

   The key: `counts[w] = ...` stores a `String` key made from `w` when `w` is new, as today's `Map` does
   (`an12c`, accepted). No word is needed (vocabulary, `Map` row; run 3's clash 11): a new key is stored
   as the map's own `String`, made from the text, and keys never change in a map, so no line can tell the
   copy from the view.

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
   - `counts.get(w).unwrap_or(0)` (a type mismatch, not a memory rule), E2020: "`counts.get(w)` gives a
     reference to the count, not the count; to read it with a default, write `counts.get_or(w, 0)`".
     (Today: "expected `ref i64`, found `an integer`".)
   - `counts[w] += 1` on a missing key stops at run time, as Python's `KeyError` (spec 6769); its
     message: "`counts` has no key `"x"`; to start a missing count at 0, write
     `counts[w] = counts.get_or(w, 0) + 1`".
   - Not refused: `counts.entry(w.to_string()).or_default() += 1`. A call whose result type is `ref mut T`
     is a place (R6, which names this very line; area B's AN-23). Run 3 listed it here under "still
     refused" while pointing at AN-23, which accepts it; under R6 it is accepted.

7. **Cost:** one hash and one probe per word when fused, as C's find-or-insert; a new word also copies
   its text once into the map's key (C and Python store the key too). Rust's `entry(w.to_string())`
   allocates a key for every word, new or not.

8. **Kind:** a readable spelling.

### AN-13: `SoA.get(i)` silently copies, while `Array.get(i)` gives a reference

**Meaning changed:** no, since v2 (v2 changed it under verdict 5; point 3 says how). `ref mut ps[0]`
gives an `SoAMut[T]`, not a `MutSpan`, so verdict 34 does not reach it, and it already says `ref mut`.

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
    snapshot = copy of ps[0]          # a copy of your own, gathered from the columns
    p = ref mut ps[0]                 # particle 0 itself, which p may change; no copy
    p.life -= 0.5
    println(p.life)                   # 1.5
    q = ps.get(0).unwrap()            # a read-only name of particle 0, as Array.get gives one
    println(snapshot.life, q.life)    # 2.0 1.5
```

   `SoA.get(i)` gives `Option[SoARef[T]]`, a read-only name for element `i` (a position, R8 (a)), as
   `Array.get(i)` gives `Option[ref T]`: neither copies. `ps[0]` names the element by position too, and
   `ref mut ps[0]` names it as a place `p` may change (an `SoAMut[T]`). A copy is made only where the line
   says `copy of` (rows 6, 9); `load()` stays the method `copy of` runs on an `SoA` element, as `clone` is
   the method it runs elsewhere (vocabulary, `SoA.get` row).

   **What changed since run 4** (verdict 5): run 4's NEW took `p = ps.get(0).unwrap()`, changed the
   particle through `ps[0]` and then read `p.life`, letting the read-only `p` see the write. A read-only
   name now promises that what it reads does not change while it is used (R8 (g)), so that program is
   refused, with both ways out (section 6); it is the same refusal today's Ember gives the same lines on an
   `Array` (point 2: `q = xs.get(0).unwrap()`, `xs[0].life -= 0.5`, `q.life`, E3021). The intent, "see
   its current life after it changes", is met in the two ways that message gives: change it through a
   name that may change it (`p`) and read it there, or name it again after the change (`q`).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `ps` | none (default: `main`'s own new column storage) | none (default: its own) |
| `snapshot` | `copy of`: a copy of your own | none (default: its own copy; changing it changes no particle in `ps`) |
| `p` | `ref mut`: particle 0 of `ps` (a position) | `ref mut`: yes, and the only name that reads or changes it until its last use (R8 (f)) |
| `q` | none (default: what `get` gives, a read-only name of particle 0, the same for `Array` and `SoA`) | none (default: read only; particle 0 does not change while `q` is used, R8 (g)) |
| `pos`, `life` (fields) | none (default: values stored in each `Particle`, one column each) | none (default: through a name that may change the particle, here `p`) |

5. **Means exactly:** the copy by today's `load()`, particle 0 changed and read by position. Accepted:

`target/redesign4/v2-02-area-A/means/m13v2_soa.em`:

```ember
# AN-13 v2 means exactly: a copy by load(); particle 0 changed in place, then read again by position.
from std.math import Vec3

@derive(Copy)
struct Particle:
    pos: Vec3
    life: float

fn main():
    ps = SoA[Particle].with_capacity(4)
    ps.push(Particle(pos=Vec3.ZERO, life=2.0))
    snapshot = ps[0].load()
    i = 0
    ps[i].life -= 0.5
    println(ps[i].life)
    println(snapshot.life, ps[i].life)
```

`target/redesign4/bin/ember.exe check m13v2_soa.em`:

```text
exit=0
```

6. **Still refused:**
   - run 4's NEW, a change of what a read-only name reads while it is still used (R8 (g), verdict 5):
     ```ember
     p = ps.get(0).unwrap()
     ps[0].life -= 0.5
     println(p.life)
     ```
     E3021: "`ps[0].life -= 0.5` (line 13) changes the particle `p` reads (`p = ps.get(0).unwrap()`,
     line 12), and `p` is used on line 14; change it after `p`'s last use, or write `p = ref mut ps[0]`
     and change it through `p`: `p.life -= 0.5`".
   - a change through `p` when it only reads (R5: a plain name never changes anything),
     `p = ps.get(0).unwrap()`, `p.life -= 0.5`, E3023: "`p` reads particle 0 and cannot change it; to
     change it through `p`, write `p = ref mut ps[0]`".
   - with `p = ref mut ps[0]`, a write through `ps[0]` while `p` is still used (R8 (f)), `ps[0].life -=
     0.5` on line 13 and `println(p.life)` on line 14, E3022: "`p` is the only name for `ps[0]` until line
     14, and line 13 changes `ps[0].life`; change it through `p`: `p.life -= 0.5`".
   - `ps.swap_remove(0)` while `p` is used after it (R8 (a), (g): a reshape ends the name), with
     `p = ps.get(0).unwrap()` on line 12, `ps.swap_remove(0)` on line 13 and `println(p.life)` on line 14,
     E3060: "`ps.swap_remove(0)` (line 13) may put another particle in the place `p` names (line 12), and
     `p` is used on line 14; use `p` before line 13, or name the particle again after it:
     `p = ps.get(0).unwrap()`".
   - `snapshot = ps[0].load()` written in a program (a word written instead, vocabulary `SoA.get` row),
     E1010: "`ps[0].load()` copies particle 0 with a method of its own, and a copy in Ember is always
     written `copy of`; write `snapshot = copy of ps[0]`".

7. **Cost:** less than today: `get` and `ref mut ps[0]` read and write only the columns used (`p.life`
   is one float), where today's `get` gathers every column into a new `Particle`. `copy of ps[0]` gathers
   every column once, as today's `load()` does.

8. **Kind:** a readable spelling (one meaning for `get`, a named copy), with Rust's rule kept for a
   read-only name (verdict 5).

### AN-14: `assert_disjoint` returns a `Result` and never stops the program

**Meaning changed:** yes (verdict 34). v2's `blend(dst: MutSpan[float], ...)` and its locals
`d = tracks[dst_name].as_mut_span()` wrote elements with no word on the name (R5's exception by type).
In v3 a plain name holding a `MutSpan` only reads: `blend` says `ref mut dst`, and `mix` and `mix_any`
say `d = ref mut tracks[dst_name].as_mut_span()` (readability's F-14 is settled by it). Where the check
is written, and what `check_disjoint` gives back, are v2's.

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
from std.collections import Map

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    assert_disjoint(d, s)          # stops the program if the two names are one track
    blend(ref mut d, s)

fn mix_any(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    match check_disjoint(d, s):
        Ok(_):
            blend(ref mut d, s)
        Err(_):
            pass                   # one track: blending it with itself leaves it as it is
```

   **Where the check is written now.** By R4 every call proves its arguments apart where one of them may
   be changed: a `MutSpan` argument is marked `ref mut` at the call (verdict 3), and the call is accepted
   only where the compiler knows the other arguments do not reach its elements (two different lists, a
   split, different constants, `j` from `i + 1..`, inside `if i != j:`, after `assert_disjoint`), with one
   index compare where two elements of one list differ only by an index. So inside `blend`, `dst` and
   `src` never overlap, both are compiled with noalias, and a check there on its own parameters would
   always pass (run 4's `blend` wrote one). The check belongs where two spans are made from something the
   compiler cannot tell apart: here two tracks of one map, which are one list when `dst_name` equals
   `src_name`. `assert_disjoint(d, s)` compares the two address ranges, stops the program if they overlap,
   and afterwards the compiler treats `d` and `s` as apart, so the call is accepted (R4).
   `check_disjoint(d, s)` only reports the answer and stops nothing; in its `Ok` branch the two are known
   apart, as inside `if i != j:`; in its `Err` branch they overlap, and here that means one track.

   **What the two library functions take** (vocabulary `assert_disjoint` row): both read their arguments
   (plain parameters, R4), take nothing and give nothing back but the answer, so `d` and `s` stay usable
   after them. Today's `std/src/mem.em:76, :85` take both `owned` and hand them back (a library change for
   the lead, `V2/changes.md` section 5, A 4).

   **The names that write say `ref mut`** (verdict 34; R2, R4, R5). `blend` writes the elements of the
   span it is given, so its parameter is `ref mut dst: MutSpan[float]`, called `blend(ref mut d, s)`;
   `src` only reads, so it has no word. `d` is how `mix` hands those elements to `blend` to be written,
   so it says `ref mut` too: `d = ref mut tracks[dst_name].as_mut_span()` names the elements of
   `tracks[dst_name]` through the span `as_mut_span()` gives, and may let them be written. A plain
   `d = tracks[dst_name].as_mut_span()` would only read them, and `blend(ref mut d, s)` would then be
   refused (point 6).

   While `d` is used it is the only name for the elements of `tracks[dst_name]` (R8 (f)). `s` is made
   while `d` is in use, from `tracks[src_name]`, which is the same list when the two names are equal. This
   file reads R8 (f) as refusing a read or a change of `d`'s elements through `s`, not the making of `s`
   or the reading of its address range: `as_span()` reads the list's buffer address and length, and
   `assert_disjoint` and `check_disjoint` compare the two address ranges, and none of them reads an
   element. The first use of `s` that reads elements is inside `blend`, after the two are proved apart.
   That reading is what lets `assert_disjoint(d, s)` exist at all (R4 names it as a proof), and it is
   left for the lead to state in R8 (f) (section 5).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `dst` | `ref mut`: the caller's place, a `MutSpan` (a view of elements the caller owns) | `ref mut`: yes; it writes those elements, is their only name while it is used (R8 (f)), and may point the caller's span elsewhere (R4) |
| `src` | `Span`: a view of the caller's elements (a plain parameter) | none (default: a plain parameter reads; the elements do not change while `src` is used, R8 (g)) |
| `i` | none (default: the loop's own number) | none (default: only the loop gives it its values) |
| `tracks` (in `mix`, `mix_any`) | `ref mut`: the caller's map | `ref mut`: yes |
| `dst_name`, `src_name` | `StrView`: views of the caller's text (plain parameters) | none (default: a plain parameter reads) |
| `d` | `ref mut`: the elements of `tracks[dst_name]`, someone else's, through the `MutSpan` that `as_mut_span()` gives | `ref mut`: yes; the only name for them while it is used (R8 (f)); `ref mut d` at the call says `blend` writes them |
| `s` | none (default: a name bound to a call that gives a view is a read-only view, R2: here a `Span` of the elements of `tracks[src_name]`) | none (default: read only; the elements do not change while `s` is used, R8 (g)) |

   `_` in `Ok(_)` and `Err(_)` declares no name.

5. **Means exactly:** `blend` is today's `mut dst` loop over two lists the call proves apart. Accepted:

`target/redesign4/v2-02-area-A/means/m14v2b_blend.em`:

```ember
# AN-14 v2 means exactly: blend of two different lists, nothing checked (today's `mut dst`).
fn blend(mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn main():
    a = [1.0, 2.0]
    b = [3.0, 4.0]
    blend(a.as_mut_span(), b.as_span())
    println(a[0])
```

`target/redesign4/bin/ember.exe check m14v2b_blend.em`:

```text
exit=0
```

   `mix` is today's stopping form, `assert_disjoint_or_panic` (two compares of the two address ranges, a
   stop if they overlap), which takes the two views and gives them back:

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

   Today's Ember cannot take the two spans of one map that `mix` takes (it refuses the second borrow of
   `tracks` before any check can run), so `mix` itself is given in words: two compares, a stop on overlap,
   then the call to `blend` with noalias:

`target/redesign4/v2-02-area-A/means/m14v2_mix.em`:

```ember
# AN-14 v2 means exactly: two spans of two tracks of one map, checked apart once, then blended.
from std.mem import assert_disjoint_or_panic
from std.collections import Map

fn blend(mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn mix(mut tracks: Map[String, Array[float]], dst_name: str, src_name: str):
    (d, s) = assert_disjoint_or_panic(tracks[dst_name].as_mut_span(), tracks[src_name].as_span())
    blend(d, s)

fn main():
    tracks = Map[String, Array[float]]()
    tracks[String.from("a")] = [1.0, 2.0]
    tracks[String.from("b")] = [3.0, 4.0]
    mix(tracks, "a", "b")
    println(tracks["a"][0])
```

`target/redesign4/bin/ember.exe check m14v2_mix.em`:

```text
error[E3021]: `*tracks` is borrowed here and mutably borrowed elsewhere
  --> m14v2_mix.em:10:71
   |
10 |     (d, s) = assert_disjoint_or_panic(tracks[dst_name].as_mut_span(), tracks[src_name].as_span())
   |                                                                       ^^^^^^ conflicting access here
   |                                       ---------------- mutable borrow of `*tracks` starts here
11 |     blend(d, s)
   |              - borrow later used here
   |
   = help: end the borrow before this: `d` is what keeps it alive, so shorten its last use or put it in a block of its own
   = note: a borrow lasts until its last use, not to the end of the scope (BRW-2)

error: could not compile due to 1 error(s)
exit=1
```

6. **Still refused, or warned:**
   - `mix` without the check (R4), lines 10 to 12 being `d = ...`, `s = ...` and `blend(ref mut d, s)`,
     E3021: "`blend` changes the elements of `d` (`tracks[dst_name]`, line 10) while `s` shows
     `tracks[src_name]` (line 11), and the two are one list when `dst_name` equals `src_name`; prove them
     apart first, `assert_disjoint(d, s)` before line 12, or blend from a copy: instead of lines 10 to 12
     write `old = copy of tracks[src_name]` and `blend(ref mut tracks[dst_name].as_mut_span(),
     old.as_span())`".
   - one list given as both (R4), `blend(ref mut a.as_mut_span(), a.as_span())`, E3021: "`blend` changes
     the elements of `a` through `dst` while `src` shows the same elements; give `src` a copy of its own:
     instead of this line write `old = copy of a` and `blend(ref mut a.as_mut_span(), old.as_span())`".
   - `check_disjoint(d, s)` with its result dropped: warned, W2190 (`@must_use` on `check_disjoint`,
     vocabulary `assert_disjoint` row): "`check_disjoint` only reports whether `d` and `s` overlap and
     does not stop the program; to stop when they overlap, write `assert_disjoint(d, s)`; to handle both
     cases, write `match check_disjoint(d, s):`". A program that binds the result and ignores it
     (`_checked = check_disjoint(d, s)`) is accepted, and its call `blend(ref mut d, s)` is refused as in
     the first bullet: ignoring the answer proves nothing.
   - two single-object references given to `assert_disjoint` (`E3095`, `[DSJ-4]`): kept, today's code
     (section 5).
   - the call without its word (R4), `blend(d, s)`, E3028: "`blend` changes the elements of `d` (its
     first line says `ref mut dst: MutSpan[float]`); say so at the call: `blend(ref mut d, s)`".
   - a plain `blend` parameter that writes (verdict 34; R4, R5), `fn blend(dst: MutSpan[float], src:
     Span[float]):`, E3023: "`dst[i] = dst[i] * 0.5 + src[i] * 0.5` (line 7) writes an element of the
     caller's span, but `dst` only reads it; write `fn blend(ref mut dst: MutSpan[float], src:
     Span[float]):`, and the calls stay `blend(ref mut d, s)`".
   - a plain local holding the span, given to be written (verdict 34; R2, R4), `d =
     tracks[dst_name].as_mut_span()` on line 10, then `blend(ref mut d, s)` on line 13, E3027: "`d` only
     reads the elements of `tracks[dst_name]` (line 10 gives it no `ref mut`), so it cannot be given to
     `blend` as `ref mut`; write line 10 as `d = ref mut tracks[dst_name].as_mut_span()`". A write
     through it, `d[0] = 0.0`, is E3023 with the same line to write (the rules' E3023 row).

7. **Cost:** in `mix`, two compares at `assert_disjoint` (`[DSJ-1]`), none where the compiler already
   knows the ranges apart (`[DSJ-5]`); in `blend`, nothing: its parameters are proved apart at every call
   and compiled with noalias. Noalias is what the compares buy: two spans of one list ran 1.53x of C
   without it and 1.00x to 1.07x with it (`tasks/96/redesign/phase3b-containers.md:346-355`). Run 3's
   two-compare test inside a callee is gone (`V3/rules.md` section 2).

8. **Kind:** a readable spelling (an `assert` stops the program, as in Python), placed where the
   compiler cannot prove the two apart; everywhere else the proof is the compiler's (R4).

### AN-15: The ECS query marker `Mut[T]` is a second spelling of `ref mut T`

**Meaning changed:** no. As in AN-06, `world.run(integrate, 0.016)` counts as making what `integrate`'s
type says it changes (verdict 21), which is nothing beyond the query it is given.

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

3. **NEW** (unchanged from run 3):

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

   `ref mut Position` in the query's type says the system changes positions (it sets the system's write
   set, `[ECS-4]`, as `Mut[Position]` does today); `ref mut pos` on the loop name says this name changes
   them. In the loop's list of names each word belongs to the one name after it, and a name with no word
   may not follow one with a word (R3), so `vel` writes `ref`, the default spelled out: a read-only view.
   Inside the query's parentheses each element is a type of its own, so `Velocity` needs no word there.
   `std.ecs` maps `ref mut T` to the item `ref mut T` exactly as it maps `Mut[T]` today
   (`std/src/ecs.em:406`, the `QueryParam` interface). The world runs it as `world.run(integrate,
   0.016)` and calls it with `ref mut` before the query (AN-06).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `q` | `ref mut`: the query the world made for this run | `ref mut`: yes |
| `pos` | `ref mut`: the `Position` of the current entity | `ref mut`: yes |
| `vel` | `ref`: a read-only view of the entity's `Velocity` | `ref`: read only |
| `dt` | none (default: a number passed in) | none (default: read) |
| `x`, `dx` (fields) | none (default: values stored in each component) | none (default: through `pos`) |

5. **Means exactly:** today's `Mut[Position]`, the first program in point 2 (`m15_query_mut.em` is
   that file, copied), accepted:

```text
exit=0
```

6. **Still refused:**
   - `Mut[Position]` (a removed word), E1010: "`Mut[Position]` is now `ref mut Position`: write
     `Query[(ref mut Position, Velocity)]`".
   - `for pos, ref vel in q:` then `pos.x += ...` (R5: a plain loop name never changes anything), E3023:
     "`pos` reads each `Position`; to change it, write `for ref mut pos, ref vel in q:`".
   - `for ref mut pos, vel in q:` (R3), E0116: "`ref mut` here belongs to `pos` only; say what `vel` is:
     `for ref mut pos, ref vel in q:`, or `ref mut vel` to change it too".
   - `for ref mut pos, ref mut vel in q:` when the query says `Velocity` (a loop name over something it
     only reads, `V3/rules.md` section 4), E3027: "the query reads `Velocity` only; to change it, ask for
     it: `Query[(ref mut Position, ref mut Velocity)]`".

7. **Cost:** none; the same code as `Mut[Position]`.

8. **Kind:** a readable spelling (one spelling for "changes it").

### AN-16: Messages use the compiler's names, not the program's, and many helps do not give the line to write

**Meaning changed:** no. The refusals are those of AN-01 to AN-15; the lines their messages give take
the v3 spellings (`ref mut` on a `MutSpan` name or parameter that writes, verdict 34; a change list,
verdict 21; a lambda naming the captured variable it changes, verdict 22).

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

3. **NEW:** the messages written in point 6 of AN-01 to AN-15, under R18:
   - **names:** variables as written; an element as `enemies[0]`, `bodies[i]` and `bodies[j]` (never
     `*enemies[0]` or `*bodies[…]`); a temporary by its expression, `line.to_lower()` (never `_5`); types as
     the program wrote them, `float`, `int` (never `f64`, `i64`); a lambda by its text (never
     `Callable0`); a library's type parameter by what it stands for in the call (`collect` "cannot tell
     what list `collect()` makes here; write the list's type on the name:
     `words: Array[StrView] = line.split_whitespace().collect()`", not `C`).
   - **why:** one line, in the program's words ("`enemies` may not change while this loop walks it").
   - **the line to write:** always, in the program's names, once (`for ref mut e in enemies:`), and it
     compiles in that program under the rules (R18): a message that advises a call to a `ref mut`
     parameter, a `MutSpan` one included, writes the call with `ref mut` (`on_death(ref mut enemies,
     dead.pop().unwrap())`, `collide(ref mut bodies[i], ref mut bodies[j])`,
     `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`); a message about a write through a
     `MutSpan` gives the name or parameter with `ref mut` (`fn integrate(ref mut pos: MutSpan[float],
     vel: Span[float], dt: float):`, `d = ref mut tracks[dst_name].as_mut_span()`; verdict 34); a message
     about what a function given to a call changes gives the whole first line that names it
     (`fn update(ref mut enemies: Array[Enemy]) changes LOG:` in AN-04, the change-list line in AN-05;
     verdicts 21, 22); where a line replaces several, the message names every line it replaces (AN-11's
     and AN-14's copies).
   - **both ways out** where there are two: a change of what a read-only name reads gives "change it
     after its last use" and the `ref mut` line (AN-01, AN-13; verdict 5); a lost change gives the
     place and the function that gives back the element itself (AN-10; verdict 13).
   - **one mistake, one message:** an error that the library would see as many errors is reported once,
     at the program's line, on the name that caused it; helps that do not apply are not printed. Where a
     call goes through a function value, the message names what its type allows (AN-02's `on_hit`).
   - **a code from `V3/rules.md` section 4 on every refusal**, written next to each message in AN-01 to
     AN-15; every refusal of this area has one.

4. **The owner's test.** AN-16's NEW is message text and binds no name itself; every name it brings into
   a program is in a line one of the messages tells the programmer to write. Each such line, and each name
   in it, with the same words as in the answer that refuses:

| line a message gives (answer) | name | what it is | may it change it |
|---|---|---|---|
| `boss = ref mut enemies[0]` (AN-01, AN-02; after the removal or the call it gives `boss` another ref, verdict 42) | `boss` | `ref mut`: element 0 of `enemies` (a position) | `ref mut`: yes, the only name for it until its last use |
| the same line | `enemies` | `ref mut` on the parameter (`ref mut enemies: Array[Enemy]`): the caller's list | `ref mut`: yes |
| `boss = copy of enemies[0]` (AN-01) | `boss` | `copy of`: a copy of your own | none (default: its own copy; changing it changes no enemy) |
| `boss.hp = 1` (AN-01) | none declared | — | — |
| `n = String(boss.name.trim())` (AN-01) | `n` | `String(...)`: your own text (row 5) | none (default: its own) |
| `split_boss(ref mut es)`, `spawn_minion(ref mut enemies, boss.x)` (AN-01, AN-02) | none declared | `ref mut` at the call says the argument is changed | — |
| `if i != j:` (AN-03) | `i`, `j` | none (default: numbers the function was given or counts, its own) | none (default: read) |
| `collide(ref mut bodies[i], ref mut bodies[j])` (AN-03) | none declared | `ref mut` at the call says both bodies are changed | — |
| `spawned.push(Enemy(hp=5, splits=false))`, `enemies.extend(spawned)` (AN-04) | `spawned` | none (default: the function's own new list) | none (default: its own; handed over by `extend`, row 9) |
| `enemies.retain(fn(e) => e.hp > 0)` (AN-04) | `e` | none (default: a lambda parameter with no word, a read-only view of each element) | none (default: read only) |
| `for ref mut e in enemies:` (AN-04) | `e` | `ref mut`: the current element of `enemies` | `ref mut`: yes |
| `fn update(ref mut enemies: Array[Enemy]) changes LOG:` (AN-04) | `enemies` | `ref mut`: the caller's list | `ref mut`: yes; `changes LOG` says the call also changes the module variable `LOG` (through `keep_alive`, which `retain` runs) |
| `dead.push(i)`, then `while dead.len() > 0: on_death(ref mut enemies, dead.pop().unwrap())` (AN-05) | `dead` | none (default: the function's own new list of positions) | none (default: its own) |
| `ref mut on_death: fn(ref mut Array[Enemy], int)` (AN-05) | `on_death` | `ref mut`: the caller's function value; its type: a function that takes the list `ref mut` and changes nothing else at all (verdict 22) | `ref mut`: the call may change the function value's own state |
| `update(ref mut es, ref mut count)` (AN-05) | none declared | `ref mut` at the call says `update` may change `es` and `count` | — |
| `fn update[C](ref mut enemies: Array[Enemy], on_death: fn(ref mut Array[Enemy], int) changes C) changes C:` (AN-05) | `enemies` | `ref mut`: the caller's list | `ref mut`: yes |
| the same line | `on_death` | its type `fn(ref mut Array[Enemy], int) changes C`: a function given by the caller (a plain parameter) | none (default: only called); `changes C` says it may change what the function given to the call changes, and `update`'s own `changes C` says the call makes those changes |
| the same line | `C` | a change list: it stands for what the function given to the call changes (R12); it names no owner and no change of its own, so the test has nothing to ask of it (vocabulary, change-list row) | — |
| `fn(ref mut list: Array[Enemy], i: int) changes kills: kills += 1` (AN-05) | `list` | `ref mut`: the list `update` gives the handler | `ref mut`: yes |
| the same line | `i` | none (default: a number passed in, copied) | none (default: read) |
| the same line | `kills` (captured, not declared here) | none (default: the real variable of `main`, row 6) | `changes kills`: yes, the real one (verdict 22) |
| `Query[(Entity, ref mut Health)]` (AN-06) | none declared | the query's types: `ref mut Health` says the system changes health | — |
| `fn words_of(text: StrView) -> Array[StrView] from text:` (AN-07) | `text` | `StrView`: a view of the caller's text (a plain parameter) | none (default: a plain parameter reads) |
| `fn words_of(line: StrView) -> Array[String]:` (AN-07) | `line` | `StrView`: a view of the caller's text (a plain parameter) | none (default: a plain parameter reads) |
| `words: Array[String] = []` (AN-07) | `words` | its type `Array[String]`: the function's own list of strings of its own | none (default: its own; handed back by `return words`) |
| `for w in line.to_lower().split_whitespace(): words.push(String(w))` (AN-07) | `w` | none (default: a plain loop name, a read-only view of one word of the lowered copy, which lives to the loop's end, R8 (d)) | none (default: read only; `String(w)` makes the copy pushed) |
| `all_words.push(String(w))` (AN-07) | `all_words` | none (default: the function's own list, made before the loop) | none (default: its own) |
| the same line | `w` | none (default: a plain loop name, a read-only view of one word) | none (default: read only) |
| `String(w)` (AN-07) | none declared | `String(...)`: your own text (row 5) | — |
| `words: Array[StrView] = line.split_whitespace().collect()` (AN-16's `collect` message, AN-07's OLD) | `words` | its type `Array[StrView]`: the function's own list of views into `line` | none (default: its own; its views only read) |
| `return String(word)` (AN-09) | `word` | none (default: a read-only view of the lowered copy, row 5) | none (default: read only) |
| `es[0].heal()` (AN-10) | `es` | none (default: `main`'s own list) | none (default: its own; `heal`'s first line, `changes hp`, says it changes `es[0].hp`) |
| `fn enemy_at(ref mut xs: Array[Enemy], i: int) -> ref mut Enemy:` with `return ref mut xs[i]` (AN-10) | `xs` | `ref mut`: the caller's list | `ref mut`: yes; the result gives back one of its elements as a place |
| the same line | `i` | none (default: a number passed in, copied) | none (default: read) |
| `enemy_at(ref mut es, 0).heal()` (AN-10) | none declared | `ref mut` at the call says `enemy_at` may change `es`; `heal` changes the enemy it gives back | — |
| `fn heal() changes hp:` (AN-10) | `this` (in its body) | `this`: the enemy `heal` was called on | `changes hp`: it may change `hp` only |
| `for ref mut p in es:` (AN-10) | `p` | `ref mut`: the current enemy of `es` | `ref mut`: yes |
| `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)` (AN-11) | none declared | `ref mut` at the call says `integrate` writes the elements of `p` | — |
| `old = copy of p` (AN-11) | `old` | `copy of`: a copy of your own | none (default: its own) |
| `fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):` (AN-11) | `pos` | `ref mut`: the caller's place, a `MutSpan` (a view of the caller's elements) | `ref mut`: yes; it writes those elements and may point the caller's span elsewhere |
| the same line | `vel` | `Span`: a view of the caller's elements (a plain parameter) | none (default: a plain parameter reads) |
| the same line | `dt` | none (default: a number passed in) | none (default: read) |
| `fn first_even(ref mut xs: MutSpan[int]) -> ref mut int:` (AN-11) | `xs` | `ref mut`: the caller's place, a `MutSpan` | `ref mut`: yes; it writes those elements and gives one back as a place |
| `first_even(ref mut a.as_mut_span())` (AN-11) | none declared | `ref mut` at the call says `first_even` writes the elements of `a` | — |
| `fn integrate(ref mut pos: MutSpan[float], ref mut vel: MutSpan[float], dt: float):` (AN-11) | `vel` | `ref mut`: the caller's place, a `MutSpan` | `ref mut`: yes; it writes those elements |
| `integrate(ref mut p.as_mut_span(), ref mut v.as_mut_span(), 0.5)` (AN-11) | none declared | `ref mut` before each span says `integrate` writes the elements of `p` and of `v` | — |
| `for i in 0..pos.len(): pos[i] = mine[i]` (AN-11) | `i` | none (default: the loop's own number) | none (default: only the loop gives it its values) |
| `counts[w] = counts.get_or(w, 0) + 1` (AN-12) | `counts` | none (default: the function's own map) | none (default: its own) |
| the same line | `w` | none (default: a plain loop name, a read-only view of one word) | none (default: read only) |
| `p = ref mut ps[0]`, `p.life -= 0.5` (AN-13) | `p` | `ref mut`: particle 0 of `ps` (a position) | `ref mut`: yes, the only name for it until its last use |
| `p = ps.get(0).unwrap()` (AN-13) | `p` | none (default: what `get` gives, a read-only name of particle 0) | none (default: read only) |
| `snapshot = copy of ps[0]` (AN-13) | `snapshot` | `copy of`: a particle of your own, gathered from the columns | none (default: its own copy) |
| `assert_disjoint(d, s)`, `match check_disjoint(d, s):` (AN-14) | none declared | they read `d` and `s` (plain parameters of the library functions) | — |
| `blend(ref mut d, s)` (AN-14) | none declared | `ref mut` at the call says `blend` writes the elements of `d` | — |
| `fn blend(ref mut dst: MutSpan[float], src: Span[float]):` (AN-14) | `dst` | `ref mut`: the caller's place, a `MutSpan` | `ref mut`: yes; it writes those elements |
| the same line | `src` | `Span`: a view of the caller's elements (a plain parameter) | none (default: a plain parameter reads) |
| `d = ref mut tracks[dst_name].as_mut_span()` (AN-14) | `d` | `ref mut`: the elements of `tracks[dst_name]`, through the `MutSpan` `as_mut_span()` gives | `ref mut`: yes; the only name for them while it is used |
| `old = copy of tracks[src_name]`, `old = copy of a` (AN-14) | `old` | `copy of`: a copy of your own | none (default: its own) |
| `for ref mut pos, ref vel in q:` (AN-15) | `pos` | `ref mut`: the current entity's `Position` | `ref mut`: yes |
| the same line | `vel` | `ref`: a read-only view of the entity's `Velocity` | `ref`: read only |
| `Query[(ref mut Position, ref mut Velocity)]` (AN-15) | none declared | `ref mut Velocity` says the system changes velocities | — |

   Every name has its words, or a stated default that never changes anything it does not own, never
   aliases silently and never copies silently.

5. **Means exactly:** the same programs are refused (or accepted, or warned) for the same reasons as in
   each answer's point 6, with the codes there; only the text changes. The translation is the table in
   point 2 (today's text) against point 6 of each answer (the new text).

6. **Still refused:** the refusals of AN-01 to AN-15, each with its message there.

7. **Cost:** none while the program runs.

8. **Kind:** a readable spelling (of the refusals).

## 4. Words needed

None. Every word in the NEW code above, and in every line a message gives, has a row in
`V3/vocabulary.md`: the words this step added to the area (`ref mut` on a `MutSpan` name or parameter,
the `MutSpan` and `ref mut` rows; a change list, AN-05's `update[C]` with `changes C`, its own row; a lambda's
`changes kills`, the `fn` and `changes` rows; `static mut`, `retain`, `push`, `String(...)`) and v2's
(`copy of`, `this`, `changes this`, `assert_disjoint`, `check_disjoint`, `Vec3.ZERO`, ...). `World.run`
and `run_parallel` appear only in prose, as today's `std/src/ecs.em` names, as in v2.

## 5. For the owner, and found for the lead

For the owner (one item; the lead raises it):

1. **Answered (verdict 42, the owner: "yes, every ref mut name"): plain `ref mut` may be given another
   ref (a `ref mut dst: MutSpan[float]` parameter may point the caller's `d` elsewhere); `const ref mut
   dst: MutSpan[float]` writes the elements and is never given another ref, so the caller's `d` is the
   same span after the call.** The question as it was put: **one word, two rights, on a `MutSpan`
   parameter** (verdict 34; AN-11, AN-14). `ref mut dst:
   MutSpan[float]` now both writes the caller's elements and may point the caller's span elsewhere,
   because every `ref mut` parameter of a view names the caller's place (R4; `V3/changes.md` section
   2.3). A reader of `fn blend(ref mut dst: MutSpan[float], src: Span[float]):` cannot tell from the
   first line whether `blend` only writes the elements (as it does) or also re-points `d`. Safety is
   kept either way. What it costs: the caller must take `d` to view, after the call, what it viewed or
   any other argument (R4), so a later change to `s`'s list ends `d` unless the compiler has seen
   `blend`'s body; and the span may have to be passed by its address (AN-11 point 7; not measured).
   Meanwhile this file writes verdict 34's spelling and relies on the compiler's knowledge of the body
   inside a package (for the lead, item 8 below). The other choice would keep re-pointing for a
   separate spelling (v2 had `ref mut s: MutSpan[T]` for it, when a plain `MutSpan` wrote). Is one
   spelling for both what you meant?

The rule writer's open questions that reach this area: question 4 (the library takes a function as a
bound; AN-05's `update[C]` writes the function-value form of your sketch, since `on_death` is already a
function value) and question 3 (iterator adapters; no adapter that keeps a changing function is used
here). Run 4's questions for this area stay answered (the lost change on a temporary, verdict 13;
`ref mut` alone, verdict 2).

Found for the lead (consistency of `V3/rules.md` and `V3/vocabulary.md` with this area; none needs the
owner beyond item 1 above):

1. **`check_disjoint`'s `Ok` branch as a proof** (AN-14), still open in v3. R4 lists what proves two
   arguments apart ("after `assert_disjoint(a, b)`", "inside `if i != j:`"), but not the `Ok` branch of
   `match check_disjoint(a, b):`, which the W2190 message offers as the way "to handle both cases". This
   file reads it as a proof, as `if i != j:` is; R4's list should name it, or the W2190 message should not
   offer it.
2. **Making a view while a `ref mut` name of the same elements may be in use** (AN-14, new in v3). With
   `d = ref mut tracks[dst_name].as_mut_span()` (verdict 34), R8 (f) says the elements are "read or
   changed only through" `d` while it is used. `mix` then makes `s = tracks[src_name].as_span()` and
   calls `assert_disjoint(d, s)` before anything reads an element through `s`. This file reads R8 (f) as
   refusing a read or a change of the elements through another name, not the making of a view or the
   comparing of two address ranges (point 3 of AN-14); without that reading `assert_disjoint`, which R4
   names as a proof, could never be reached with a `ref mut` span. R8 (f) should say so. (v2's note 2, a
   local holding a `MutSpan` promising nothing, is settled by verdict 34: R8 (f) and (g) now name it.)
3. **The E3021 message for two spans** (rules section 3, the `blend` example, still there in v3): "prove
   them apart first: `assert_disjoint(dst, src)`" names the callee's parameters. Under R4 the check
   belongs in the caller, before the call, on the caller's names (AN-14: `assert_disjoint(d, s)`); inside
   the callee it would always pass.
4. **Two keys of one map** (AN-14). R4's one compare covers two elements of one list that differ only by
   an index; two values of one map under two keys (`tracks[dst_name]`, `tracks[src_name]`) are the same
   case (a map's keys are unique), and could be covered the same way: one compare of the keys before the
   call, a stop if equal. Then `mix` would need no `assert_disjoint`; AN-14 keeps it because R4 does not
   say so. A smarter compile-time rule, the lead's to decide.
5. **E3095** (AN-14, two single-object references given to `assert_disjoint`, `[DSJ-4]`) is today's
   code and is not in `V3/rules.md` section 4.
6. **`std/src/mem.em:76, :85`** still take both arguments `owned` and hand them back; the vocabulary's
   `assert_disjoint` row (both read their arguments, take nothing, give only the answer) is a library
   change (already listed in `V2/changes.md` section 5, A 4).
7. **The spec's two plain `MutSpan` parameters that write** (AN-11). The vocabulary's section 6.1 row
   for `mut x: T` lists the spec lines whose `MutSpan` parameter takes `ref mut` (278 2520 3147 3618 3663
   3704 6186), which are the lines written with `mut`. The spec's examples written with no word at all,
   which verdict 34 now refuses, are not listed: `fn integrate(pos: MutSpan[float], vel: Span[float],
   dt: float):` (line 3576 of `docs/spec-source/Ember_v0.9.10_Hardened_35.md`) and `[BCK-6]`'s
   `fn first_even(xs: MutSpan[int]) -> ref mut int:` (line 6046). Both are AN-11's NEW, with `ref mut`.
8. **Whether a `ref mut` view parameter is ever pointed elsewhere** (AN-11, AN-14; item 1 for the
   owner). R17 records the "only adds" summary of a `ref mut` list parameter in the interface file, but
   no fact says that a `ref mut` view parameter (`ref mut dst: MutSpan[float]`) is never given another
   view. Without it, across a package's edge or through a function value, a caller's `d` is taken to
   view any argument of the call afterwards (R4), and the span may have to be passed by address. The
   same kind of recorded fact as the "only adds" summary would keep both as v2 had them.
9. **A change list on a program's own function** (AN-05). R12 states change lists for any generic
   parameter after `changes`, but the vocabulary's change-list row says it is written "in a library
   function's first line, and in the function type or bound of its parameter". AN-05's `fn update[C](ref
   mut enemies: Array[Enemy], on_death: fn(ref mut Array[Enemy], int) changes C) changes C:` is a
   program's own function; the row could say "a function's first line".
10. **Messages in this area that still break the message rule** (the lead's, phase 3; left as they are,
    since no verdict of this step rewrote them): `V2/readability.md` F-01 (AN-01's E3060 and AN-02's
    three messages, "name the enemy again after it: `boss = ref mut enemies[0]`", which R2 refuses on the
    existing `boss`), F-05 (AN-04's E3020 with `spawned`, AN-05's two E3020 with `dead`, lists the
    messages never declare), F-06 (AN-07's first E3063, AN-13's E1010 for `load()`, AN-16's `collect`
    message: ways out in words). Two more found here: AN-03's E3022 ("pass two different bodies, or
    write a one-body version of `collide`", the same as the rules' E3022 row) gives no line to write;
    and AN-01's last E3060 ("keep your own copy: `n = String(boss.name.trim())`") does not say that the
    line replaces line 8, `n = boss.name.trim()`, and on an existing `n`, a view, it would not compile.
    F-06's AN-11 E3063 and F-08 (AN-11's E3023 with `ps` and `vs`) were rewritten here, since verdict 34
    changed those messages; F-14 is settled by verdict 34.
    **Phase 3 (the lead):** all of these are fixed in this file and in the rules' rows; each fix is listed
    in `V3/phase3-messages.md`.
