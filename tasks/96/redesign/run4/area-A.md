# Area A under run 4's rules: lists, loops, temporary values (AN-01 to AN-16)

Run 4, phase 1, agent 02 (area A). Every answer of area A from `RUN3/combined-v4.md` lines 844-3700,
brought to the rules in `RUN4/rules.md`, the words in `RUN4/vocabulary.md` and the changes in
`RUN4/changes.md`. `RUN3` means `tasks/96/redesign/run3/`, `RUN4` means `tasks/96/redesign/run4/`; "R1" to
"R18" are the rules of `RUN4/rules.md`; "row N" is the owner's agreed row N in `inputs/JOB.md`.

Power: `PowerLineStatus` printed `Online` at the start of this work and again before writing this file.
No compiler change, no git command, no Ember program run. No new `ember check` was needed: every claim
here about what today's Ember accepts or refuses is run 3's pasted output, carried over with its
unchanged OLD code.

**How this file was made.** The OLD programs with their `ember check` output, the "means exactly"
programs with theirs, and the costs that did not change are copied verbatim from `RUN3/combined-v4.md`
by `target/redesign4/02-area-A/build_area.py` (each copied block is a line range of that file), so no
carried line can differ from run 3's by a retyping mistake. Everything else (intent, NEW, the owner's
test, the text around the code, what stays refused, kind) is written for run 4. File names inside the
carried blocks are relative to `target/redesign3/08-area-A/`, as in run 3.

## 0. Terms

The terms of `RUN4/rules.md` section 0 (owner, view, position, reshape, `this`, first line, summary,
function value, noalias, the owner's test, ...) are used as defined there. This area adds:

- **list**: `Array[T]`: a pointer to a block of memory holding the elements (the **buffer**), the length
  and the capacity. A full list that grows gets a new, larger buffer and frees the old one
  (**reallocates**), so an address into the old buffer would point into freed memory.
- **element name**: a name made from one element of a list, `boss = ref mut enemies[0]`. By R8 (a) it is
  a position: the element is found again at each use.
- **temporary**: a value made inside an expression and never given a name (`line.to_lower()` in
  `line.to_lower().trim()`).
- **"only adds" summary**: what the compiler records about a function that takes a list as `ref mut`
  when the function's only use of the list is adding at its end (`push`, `extend`); anything else (a
  removal, an insert, a reorder, `clear`, replacing the list, or reading or writing an element already
  there) is "may do more". Worked out from the body; across a package's edge it is recorded in the
  interface file (R17).
- **unseen code**: a call whose body the compiler cannot read here (a function value, an interface
  method, a foreign function); it is taken to do everything its type allows (R8 (b), R12).
- **refused**: a compile-time error, with its code from `RUN4/rules.md` section 4. **Stops**: a run-time
  stop with a message (Ember's panic).
- **PROVISIONAL**: a timing made while other agents were running (run 3's mark, kept on run 3's numbers).

## 1. Run 3's area rules A-1 to A-8, and where each is now

Run 3's answers cited eight rules of this area. Run 3 merged most of them into its shared rules; this
table says which run-4 rule now decides each, so the answers below cite only run-4 rules.

| run 3's area rule | now | what changed for this area |
|---|---|---|
| A-1: an element name is a position; plain `boss = enemies[0]` reads, `ref mut` changes, `copied` copies, a number part is copied | R2, R8 (a) | nothing |
| A-2: while an element name is used, its list may grow but not shrink or move; writes to the element through another name are allowed | R8 (a), (b), **(f)** | **the last half is reversed**: while a `ref mut` element name is still used, its element is read or changed only through it (R8 (f), the owner's one meaning of `ref mut`, "only this name may touch it until it is done"); a write or read through `enemies[0]`, or through a helper given the list, is refused (AN-01, AN-02). A plain (read-only) element name still sees such writes (AN-13) |
| A-3: a list may not change while a loop walks it; a counting loop may add, not remove | R8 (c), R12 | a call that may run a function value counts as a change too (R12; AN-05) |
| A-4: two `ref mut` parameters are two different places; one compare where not proved | R4 | the call now says `ref mut` (owner, 2026-10-25): `collide(ref mut bodies[i], ref mut bodies[j])` (AN-03) |
| A-5: a temporary lives until the last use of every view made from it | R8 (d) | nothing (row 5) |
| A-6: a `mut self` method on a temporary is allowed when its result is used, refused when nothing it returns is used | **no run-4 rule** | `mut self` is `ref mut this` or `changes <field>` now (R5). Run 3 never merged A-6 into its shared rules, and `RUN4/rules.md` does not state it. AN-10 keeps the refusal of a lost change and lists the gap in section 5 |
| A-7: a view's type says what it allows (`MutSpan` writes, `Span` and `StrView` read; an ECS query says `ref mut T`) | R5's exception by type, R3, R4 | an argument whose type is `MutSpan` needs no word at the call (R4) |
| A-8: messages in the program's names, why in one line, the exact line to write | R18 | the line a message gives must compile in that program (R18) |

## 2. Answers whose meaning changed under run 4's rules

| answer | how |
|---|---|
| AN-01 | The NEW program means what it meant. Narrowed around it by R8 (f): while `boss = ref mut enemies[0]` is still used, `enemies[0].hp = 1` (or reading `enemies[0]`) through another name is refused (E3022, E3021); run 3's A-2 allowed it. This is the rule writer's question 3 to the owner (`RUN4/changes.md` section 13) |
| AN-02 | The NEW program means what it meant. Narrowed by R8 (f): a helper given `ref mut enemies` while `boss` is still used may only add to the list; one that also reads or writes an element already there (`heal_all(ref mut enemies)`) is refused (E3022); run 3 accepted it |
| AN-05 | The NEW program means what it meant. Narrowed by R12: a handler that changes its own state (a lambda that changes a `copied` capture) may not be given to the read-only parameter `on_death` (E3023: write `ref mut on_death` and pass it with `ref mut`) |
| AN-14 | `check_disjoint` and `assert_disjoint` no longer take the two views and hand them back: they read them (plain parameters, R4), and `check_disjoint`'s result holds views of what it was given (R6). Run 3's form would now be refused: handing over the plain parameter `dst` is E3013, and `owned` on a `Span` (a `Copy` type) is E2085. So using `dst` after `check_disjoint` is no longer refused, and a dropped result is the `@must_use` warning W2190. For spans of elements that own memory, an overlap is now refused when compiling at the call (R4, E3021) instead of tested while running, so `assert_disjoint` is only needed for spans of numbers |

Every other answer keeps its meaning; their spellings change where the owner's decisions of 2026-10-25
reach them (`ref mut` at calls: AN-02, AN-03, AN-05, AN-06; no `self`: AN-10).

## 3. The answers

### AN-01: Holding an element while adding to the list

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
     write `boss = copied enemies[0]`".
   - a write or a read of the element through another name while `boss` is still used (R8 (f)):
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
     `boss = ref mut enemies[0]`".
   - a view into a part that a write frees, used after that write (with `name: String` in `Enemy`;
     R8 (b)): `n = boss.name.trim()`, then `enemies[0].name = String("x")`, then `n.len()`, E3060: "`n` is
     a view of `enemies[0].name`, whose old text line 9 frees, and `n` is used on line 10; use `n` before
     line 9, or keep your own copy: `n = String(boss.name.trim())`".
   - the call without its word (R4), `split_boss(es)`, E3028: "`split_boss` may change `es` (its first
     line says `ref mut enemies: Array[Enemy]`); say so at the call: `split_boss(ref mut es)`".

7. **Cost:** after the push, one reload of `enemies`' buffer pointer, then an indexed store; no bounds
   check (adding never shortens the list). C must reload after a `realloc` too, and Rust's workaround
   (copy the fields out, push, index again) is the same instructions plus a bounds check
   (`tasks/96/redesign/phase3b-containers.md:233-239`, argued there from the emitted code).

8. **Kind:** a smarter compile-time solution (Rust's rule "no add while a reference lives" is not
   needed for adds; it stays for removals and reorders, and for other names of the element, R8 (f)).

### AN-02: A struct element's name used after a call that may change its list

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
   - unseen code given the list while `boss` is used after it (`on_hit(ref mut enemies)` with
     `on_hit: fn(ref mut Array[Enemy])`; R8 (a), R12), E3060: "`on_hit(ref mut enemies)` (line 9) may
     remove or reorder enemies (it is a function value, so what it does is not known here), which may put
     another enemy in the place `boss` names (line 7), and `boss` is used on line 10; use `boss` before
     line 9, or name the enemy again after it: `boss = ref mut enemies[0]`".
   - a helper the compiler can read that may remove (R8 (a)), E3060: "`kill_weak(ref mut enemies)`
     (line 9) may remove enemies (`kill_weak` line 3: `enemies.remove(i)`), which may put another enemy in
     the place `boss` names (line 7), and `boss` is used on line 10; use `boss` before line 9, or name the
     enemy again after it: `boss = ref mut enemies[0]`".
   - a helper that reads or writes an element already in the list (R8 (f); run 3 accepted this), with
     `fn heal_all(ref mut enemies: Array[Enemy]):` whose body is `for ref mut e in enemies: e.hp += 1`,
     E3022: "`boss` is the only name for `enemies[0]` until line 10, and `heal_all(ref mut enemies)`
     (line 9) changes the enemies in it (`heal_all` line 3: `e.hp += 1`); use `boss` before line 9, or
     name the enemy again after it: `boss = ref mut enemies[0]`".
   - the call without its word (R4), `spawn_minion(enemies, boss.x)`, E3028: "`spawn_minion` may change
     `enemies` (its first line says `ref mut enemies: Array[Enemy]`); say so at the call:
     `spawn_minion(ref mut enemies, boss.x)`".

7. **Cost:** nothing more than AN-01: one reload after the call (the C compiler reloads after any call
   that gets the list anyway).

8. **Kind:** a smarter compile-time solution for code the compiler can read; for unseen code the
   understandable limitation stays, with a message that gives the line.

### AN-03: Two elements of one list in one call

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
   - the same position twice, known when compiling (`collide(ref mut bodies[i], ref mut bodies[i])`;
     R4), E3022: "`collide` changes `a` and `b` as two different bodies, and both are `bodies[i]`; pass
     two different bodies, or write a one-body version of `collide`".
   - at run time, where not proved and `i == j` (a stop, not a refusal): "`collide` was given
     `bodies[3]` as both `a` and `b`; test `if i != j:` before the call to skip it".
   - the call without its words (R4), `collide(bodies[i], bodies[j])`, E3028: "`collide` may change both
     bodies (its first line says `ref mut a: Body, ref mut b: Body`); say so at the call:
     `collide(ref mut bodies[i], ref mut bodies[j])`".

7. **Cost:** where proved, nothing. Elsewhere one compare and a branch never taken. Run 2 measured the
   inserted compare at no cost (89.9 ms with it, 94.9 ms without it and without noalias; safe Rust's
   `get_disjoint_mut` 87.6 ms: `tasks/96/redesign/phase3b-containers.md:301-306`, from
   `target/redesign2/containers/bench/runs/a3.txt`). Noalias is what matters for speed: without it,
   two spans of one list took 1.53x (`phase3b-containers.md:351-352`, `runs/a3_span.txt`). R4 keeps
   noalias on every `ref mut` parameter that no link the function reads may reach.

8. **Kind:** a readable spelling of Rust's guarantee (two `&mut` never alias), with the compile-time
   proof that Rust's borrow checker does not make for computed positions.

### AN-04: Changing a list while looping over it

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
   function it is given, so it is a call that may run a function value (R12); that function changes
   nothing.

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
     loop walks (line 7); collect the new enemies in a list of your own, `spawned.push(Enemy(hp=5,
     splits=false))` here, and write `enemies.extend(spawned)` after the loop".
   - `enemies.remove(i)` in the loop (R8 (c)), E3020: "`enemies.remove(i)` shortens `enemies`, which this
     loop walks (line 7); remove the dead after the loop: `enemies.retain(fn(e) => e.hp > 0)`".
   - `for e in enemies:` then `e.hp -= 1` (R5: a plain loop name never changes anything), E3023: "`e`
     reads each enemy and cannot change it; to change the enemies through `e`, write
     `for ref mut e in enemies:`".
   - `for e in enemies.iter_mut():` then `e.hp -= 1`: the same (`e` is a plain loop name), E3023: "`e` is
     a plain loop name, which only reads; write `for ref mut e in enemies:`" (vocabulary, `iter_mut`: the
     one `for` spelling that changes elements is `for ref mut e in enemies:`).

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
     real variables it captures. A lambda that changes its own `copied` capture is refused at the call
     that gives it to `update` (section 6).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `enemies` | `ref mut`: the caller's list | `ref mut`: yes; `ref mut enemies` at the call says the handler may change it |
| `on_death` | its type `fn(ref mut Array[Enemy], int)`: a function given by the caller, read only (a plain parameter) | none (default: only called; its type's `ref mut` says the function may change the list it is given) |
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
   - `on_death(ref mut enemies, i)` inside `for ref mut e in enemies:` (R8 (c)), E3020: "`enemies` may not
     change while this loop walks it (line 6), and `on_death` may change it (its type takes
     `ref mut Array[Enemy]`); collect the positions and call it after the loop: `dead.push(i)` here, then
     `while dead.len() > 0: on_death(ref mut enemies, dead.pop().unwrap())`".
   - the same call inside `for i in 0..enemies.len():`, accepted today (R8 (c): a counting loop may add
     but not remove), E3020: "this loop counts over `enemies`' length with an end fixed when it started
     (line 6), and `on_death(ref mut enemies, i)` may remove from `enemies`, which would skip the next
     enemy and then run past the end; collect the positions and call it after the loop: `dead.push(i)`
     here, then `while dead.len() > 0: on_death(ref mut enemies, dead.pop().unwrap())`".
   - a handler that changes its own state, given to the read-only `on_death` (R12), for example
     `count = fn(ref mut list: Array[Enemy], i: int) with copied kills: kills += 1` then
     `update(ref mut es, count)`, E3023: "`count` changes its own copy of `kills` each time it runs, and
     `update` takes it read-only; write `ref mut on_death: fn(ref mut Array[Enemy], int)` in `update`'s
     first line and call `update(ref mut es, ref mut count)`".

7. **Cost:** one `push` per death into `dead`, and the handler calls after the loop. C calling a
   handler it does not know must do the same to stay correct (it cannot know what the handler
   removed).

8. **Kind:** an understandable limitation kept (agreed: the list may not change while a loop walks it);
   one run-time stop moved to compile time (the counting loop).

### AN-06: ECS: creating and destroying entities inside a view loop

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
   `changes cmds`), which the loop does not walk, so the loop may call it (R8 (c)), and it touches no
   `Health` that `h` names (R8 (f): a different field, of a different type). In the loop's names, plain
   `e` comes first and `ref mut h` after it, which R3's list rule allows (only a name with no word after a
   name with a word is refused).

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
   The list is a new value that `collect()` hands to `words` (R1); because it holds views, R7 counts it
   as a view for how long it may live: no longer than the lowered text.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `line` | `StrView`: a view of the caller's text | `StrView`: read only, by its type |
| `words` | none (default: the new list `collect()` hands over, of views into the lowered copy of `line`, kept until `words`' last use; its type `Array[StrView]` comes from its value, as row 5's `word`) | none (default: not changed here; its views only read) |

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
     ends when `words_of` returns; return copies of your own: make the result `Array[String]` and push
     `String(w)` for each word; or lower the text in the caller and return views of it:
     `fn words_of(text: StrView) -> Array[StrView] from text:`".
   - keeping a word where it outlives its text (`all_words.push(w)` with `all_words` made before a loop
     over lines; R8 (d)), E3063: "`w` points into the lowered copy of `line`, which ends with this loop
     step, and `all_words` lives longer; push your own copy: `all_words.push(String(w))`".
   - `copied w` of a view (vocabulary, `copied` row): "to make your own copy write `String(w)`".
     `RUN4/rules.md` section 4 gives this refusal no code (section 5 below).

7. **Cost:** no copy per word: a view is a pointer and a length. Last run measured views against the
   alternatives on 32 MiB of words: views 1.00, safe Rust `Vec<&str>` 1.05, one allocation per word
   (today's fix-it) 2.34x (`tasks/96/redesign/phase3b-containers.md:639-646`, `runs/a25.txt`).

8. **Kind:** a smarter compile-time solution (the lifetime and the temporary's life are inferred).

### AN-08: A view of a temporary kept past its statement

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
| `line` | `StrView`: a view of the caller's text | `StrView`: read only |
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
| `line` | `StrView`: a view of the caller's text | `StrView`: read only |
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

   `next` changes the temporary iterator (its first line is `fn next() ref mut this -> Option[StrView]:`,
   R5) and returns the first word, which is used; the iterator ends with the statement. No run-4 rule
   refuses a method that changes a temporary, so this is accepted, as in Rust.

   The second program, written so that the change reaches the enemy the program means:

```ember
@derive(Copy)
struct Enemy:
    hp: int

    fn heal() changes hp:
        this.hp += 10

fn main():
    es = [Enemy(hp=1)]
    es[0].heal()
    e = copied es[0]
    e.heal()
    println(es[0].hp, e.hp)
```

   `heal`'s first line says what it does to the enemy it is called on: `changes hp` (R5; there is no
   `self`, and the object is `this` inside the method). The calls carry no word: the object is before the
   dot (R4). `es[0].heal()` heals the enemy in the list; `e.heal()` heals a copy of your own.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `line` | `StrView`: a view of the caller's text | `StrView`: read only |
| `this` (in `heal`) | `this`: the enemy `heal` was called on | `changes hp`: it may change `hp` only |
| `es` | none (default: `main`'s own new list) | none (default: its own; `heal`'s first line says it changes `es[0].hp`) |
| `e` | `copied`: a copy of your own | none (default: its own copy; changing it changes no enemy in `es`) |
| `hp` (field) | none (default: a value stored in each `Enemy`) | none (default: through a name that may change the enemy, or `this` in a method that says `changes hp`) |

   The temporary iterator in `first_word` has no name; `next`'s own first line (`ref mut this`) says it
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

   The second program means today's `fn heal(mut self)` called on `es[0]` and on a copy held in a local;
   it is given in words only (today's Ember has no `this`, `changes` or `copied`, so no file is checked).

6. **Still refused:**
   - `enemy_at(es, 0).heal()`, where `enemy_at` returns an `Enemy` by value and nothing the call returns
     is used: "`heal` changes the enemy that `enemy_at(es, 0)` returns, which is a temporary value that
     ends on this line, so the change would be lost; to change the enemy in `es`, write `es[0].heal()`; to
     change a copy of your own, write `e = copied es[0]` and then `e.heal()`". This refusal is run 3's
     area rule A-6. **No rule in `RUN4/rules.md` states it, and section 4 there gives it no code** (today's
     compiler refuses it with E2140, pasted above). Without it the program compiles and heals a copy that
     ends on the same line, which is not what it means (the owner: a rule that makes code compile but do
     something the programmer did not mean is wrong). Listed for the owner in section 5.
   - today's spelling `fn heal(mut self):` (R5), E1010: "`self` is not a word of Ember: inside a method
     its object is `this`; write `fn heal() changes hp:` for a method that changes `hp`, and
     `this.hp += 10` in its body".
   - calling it through a name that only reads (R5), `for p in es: p.heal()`, E3023: "`heal` changes the
     enemy (its first line says `changes hp`), and `p` only reads it; write `for ref mut p in es:`".

7. **Cost:** none; the same code as the local.

8. **Kind:** a smarter compile-time solution.

### AN-11: A `MutSpan` parameter written without `mut`

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

3. **NEW** (the spec's examples, as the spec writes them; unchanged from run 3):

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

   R5's exception by type: a plain name may write what a `MutSpan` leads to, with the type's own
   operation at the use (`pos[i] = ...`). The caller writes no word: `integrate(p.as_mut_span(),
   v.as_span(), 0.5)`, since the argument's type says it may be changed (R4). The elements are numbers,
   so two spans of one list may be given to one call (`integrate(p.as_mut_span(), p.as_span(), 0.5)`);
   the call is then compiled without noalias for them (R4; AN-14 says how to buy it back).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `pos` | `MutSpan`: a view of the caller's elements | `MutSpan`: its elements may be written; `pos` itself keeps the view it was given (R2) |
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

   `r = first_even(a.as_mut_span())` then `r = 7` above is today's Ember, where a name bound to a
   `ref mut` result writes through it. Under R2 and R6 the NEW spelling of those two lines is
   `r = ref mut first_even(a.as_mut_span())`, `r = 7`, which writes `a[1]`: the call is a place, and
   `ref mut` on the line says `r` may change it. Plain `r = first_even(...)` copies the number, as
   `n = counts[0]` does (R2).

6. **Still refused:**
   - giving the parameter another view, `pos = other` (R2: a parameter keeps what the caller gave for
     the whole call), E3023: "`pos` is the view the caller passed and stays that view for the whole call;
     you may write its elements (`pos[i] = ...`); for a view of your own write `rest = other`; to point the
     caller's view elsewhere, write `ref mut pos: MutSpan[float]`, called as `integrate(ref mut ps, ...)`".
   - `mut pos: MutSpan[float]` (`mut` alone on a name), E0113: "remove `mut`: `MutSpan` already lets
     `integrate` write `pos[i]`; to point the caller's view elsewhere, write
     `ref mut pos: MutSpan[float]`".
   - writing through `vel: Span[float]` (R5), E3023: "`vel` is a read-only view; to write its elements,
     take it as `vel: MutSpan[float]`". No `RefCell` or `class` help; the type as written (`float`, not
     `f64`).
   - with `ref mut pos: MutSpan[float]`, pointing the caller's view at the callee's own list (R4), E3063:
     "`pos` is the caller's view and lives on after `integrate` returns, but `mine` ends with
     `integrate`; put a view of what the caller gave there, or return a list of your own".

7. **Cost:** none; the same code as today's `mut pos`.

8. **Kind:** a readable spelling (the type says it once).

### AN-12: Updating a map entry in place

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
| `text` | `StrView`: a view of the caller's text | `StrView`: read only |
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
    ps.push(Particle(pos=Vec3(0.0, 0.0, 0.0), life=2.0))
    p = ps.get(0).unwrap()       # names particle 0, as Array.get does; no copy
    ps[0].life -= 0.5
    println(p.life)              # 1.5
    snapshot = copied ps[0]      # a copy of your own, gathered from the columns
    ps[0].life -= 0.5
    println(snapshot.life, p.life)   # 1.5 1.0
```

   (Run 3 wrote `Vec3.ZERO`; `ZERO` has no row in `RUN4/vocabulary.md`, so this file writes the same
   value with the `Vec3` constructor, as `std/src/math/mod.em:363` defines `ZERO`. Section 4.)

   `SoA.get(i)` gives `Option[SoARef[T]]`, a name for element `i` (a position, R8 (a), which names this
   very line), as `Array.get(i)` gives `Option[ref T]`. `p` is read-only, so it sees, at its next use, the
   plain data written through `ps[0]` (R8 (a)); R8 (f) does not apply, since `p` is not `ref mut`.
   `load()` stays as the method `copied` runs on an `SoA` element, as `clone` is the method `copied` runs
   elsewhere (vocabulary, `SoA.get` and `.clone()` rows).

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
   - a change through `p` (R5: a plain name never changes anything), in
     `p = ps.get(0).unwrap()`, `p.life -= 0.5`, `println(p.life)`, E3023: "`p` reads particle 0 and cannot
     change it; to change it through `p`, write `p = ref mut ps[0]`".
   - with `p = ref mut ps[0]`, a write through `ps[0]` while `p` is still used (R8 (f)), E3022: "`p` is the
     only name for `ps[0]` until line 10, and line 9 changes `ps[0].life`; change it through `p`:
     `p.life -= 0.5`".
   - `ps.swap_remove(0)` while `p` is used after it (R8 (a)), E3060: "`ps.swap_remove(0)` (line N) may put
     another particle in the place `p` names, and `p` is used after it; use `p` before line N, or name the
     particle again after it: `p = ps.get(0).unwrap()`".
   - `ps[0].load()` written in a program (a word written instead, vocabulary `SoA.get` row), E1010: "write
     `copied ps[0]`".

7. **Cost:** less than today: `get` reads only the columns used (`p.life` reads one float), where
   today's `get` gathers every column into a new `Particle`.

8. **Kind:** a readable spelling (one meaning for `get`, and a named copy).

### AN-14: `assert_disjoint` returns a `Result` and never stops the program

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

3. **NEW** (the code is run 3's; what the two library functions take is changed, below):

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

   `assert_disjoint(a, b)` (vocabulary: today's `assert_disjoint_or_panic`) is a statement: it stops the
   program if the two views overlap, and afterwards the compiler treats `dst` and `src` as not overlapping
   (noalias); parameters keep the views they were given for the whole call (R2), so that holds to the end
   of `blend`. `check_disjoint` (today's `assert_disjoint`, renamed; vocabulary) reports whether they
   overlap and gives both back in its result.

   **What the two take, under run 4's rules.** Today both take their arguments `owned`
   (`std/src/mem.em:76` and `:85`) and hand them back. Under run 4 that would refuse `blend_any`: `dst` is
   a plain parameter, so handing its value over is E3013, and `owned` on a `Span` (a `Copy` type) is
   E2085. So both read their arguments (plain parameters, R4; a `MutSpan` and a `Span` need no word at the
   call), and `check_disjoint`'s result holds two views of what it was given (R6: a function's result
   points into its view arguments): `Ok((d, s))` names them. `d`, `s`, `dst` and `src` are views of the
   same elements; the elements are numbers, so writing through `d` frees nothing that another of them
   could still view.

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `dst` | `MutSpan`: a view of the caller's elements | `MutSpan`: its elements may be written |
| `src` | `Span`: a view of the caller's elements | `Span`: read only |
| `i` (each loop) | none (default: the loop's own number) | none (default: set by the loop) |
| `d`, `s` (each arm) | none (default: plain pattern names, views of `dst`'s and `src`'s elements that `check_disjoint` gives back, with the types `MutSpan[float]` and `Span[float]`) | by those types: `d`'s elements may be written, `s` reads |

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

   (Today's function takes the two views and gives them back; the NEW one reads them and gives back views
   of them. What `blend` does with the elements is the same.)

6. **Still refused, or warned:**
   - `check_disjoint(dst, src)` with its result dropped: warned, W2190 (`@must_use` on `check_disjoint`,
     vocabulary `@must_use` row): "`check_disjoint` only reports whether `dst` and `src` overlap and does
     not stop the program; to stop when they overlap, write `assert_disjoint(dst, src)`; to handle both
     cases, write `match check_disjoint(dst, src):`". Run 3 refused this, and refused using `dst` after
     `check_disjoint` took it; nothing is taken now, so the second is accepted, and a program that binds
     the result and ignores it (`_checked = check_disjoint(dst, src)`) is accepted and memory-safe: it
     simply does not act on an overlap.
   - two single-object references (`E3095`, `[DSJ-4]`): kept, today's.
   - two spans of one list of elements that own memory given to one call, one of them changeable
     (`blend_names(names.as_mut_span(), names.as_span())` with `MutSpan[String]` and `Span[String]`;
     R4), E3021: "`blend_names` may replace strings of `names` through `dst` while `src` shows them; give
     it two different lists, or a copy: `old = copied names`, then
     `blend_names(names.as_mut_span(), old.as_span())`".

7. **Cost:** two compares (`[DSJ-1]`), none when the compiler already knows the ranges apart
   (`[DSJ-5]`). It buys noalias: two spans of one list ran 1.53x of C without it and 1.00x to 1.07x
   with it (`tasks/96/redesign/phase3b-containers.md:346-355`). The two spans here are of `float`s, so
   an overlap frees nothing and R4 lets them overlap; `assert_disjoint` is how such a function buys
   noalias. With elements that own memory (`MutSpan[String]`), R4 refuses at the call any argument that
   may overlap the changeable span (E3021, above), so inside the function the two are apart already, the
   function gets noalias with no compare, and `assert_disjoint` adds nothing; run 3's two-compare test
   before such a call is gone (`RUN4/rules.md` section 2).

8. **Kind:** a readable spelling (an `assert` stops the program, as in Python).

### AN-15: The ECS query marker `Mut[T]` is a second spelling of `ref mut T`

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
   - `for ref mut pos, ref mut vel in q:` when the query says `Velocity`: "the query reads `Velocity`
     only; to change it, ask for it: `Query[(ref mut Position, ref mut Velocity)]`". `RUN4/rules.md`
     section 4 gives this refusal no code (section 5 below).

7. **Cost:** none; the same code as `Mut[Position]`.

8. **Kind:** a readable spelling (one spelling for "changes it").

### AN-16: Messages use the compiler's names, not the program's, and many helps do not give the line to write

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
     what list to make; write `words: Array[StrView] = ...`", not `C`).
   - **why:** one line, in the program's words ("`enemies` may not change while this loop walks it").
   - **the line to write:** always, in the program's names, once (`for ref mut e in enemies:`), and it
     compiles in that program under the rules (R18): a message that advises a call to a `ref mut`
     parameter writes the call with `ref mut` (`on_death(ref mut enemies, dead.pop().unwrap())`,
     `collide(ref mut bodies[i], ref mut bodies[j])`); where a line replaces several, the message names
     every line it replaces.
   - **one mistake, one message:** an error that the library would see as many errors is reported once,
     at the program's line, on the name that caused it; helps that do not apply are not printed. Where
     the compiler cannot see which function value runs, the message says so (AN-02's `on_hit`).
   - **a code from `RUN4/rules.md` section 4 on every refusal**, written next to each message in AN-01 to
     AN-15; three refusals of this area have none there yet (section 5).

4. **The owner's test.** AN-16's NEW is message text and binds no name itself; every name it brings into
   a program is in a line one of the messages tells the programmer to write. Each such line, and each name
   in it, with the same words as in the answer that refuses:

| line a message gives (answer) | name | what it is | may it change it |
|---|---|---|---|
| `boss = ref mut enemies[0]` (AN-01, AN-02) | `boss` | `ref mut`: element 0 of `enemies` (a position) | `ref mut`: yes, the only name for it until its last use |
| the same line | `enemies` | `ref mut` on the parameter (`ref mut enemies: Array[Enemy]`): the caller's list | `ref mut`: yes |
| `boss = copied enemies[0]` (AN-01) | `boss` | `copied`: a copy of your own | none (default: its own copy; changing it changes no enemy) |
| `boss.hp = 1` (AN-01) | none declared | — | — |
| `n = String(boss.name.trim())` (AN-01) | `n` | `String(...)`: your own text (row 5) | none (default: its own) |
| `split_boss(ref mut es)`, `spawn_minion(ref mut enemies, boss.x)` (AN-01, AN-02) | none declared | `ref mut` at the call says the argument is changed | — |
| `if i != j:` (AN-03) | `i`, `j` | none (default: numbers the function was given or counts, its own) | none (default: read) |
| `collide(ref mut bodies[i], ref mut bodies[j])` (AN-03) | none declared | `ref mut` at the call says both bodies are changed | — |
| `spawned.push(Enemy(hp=5, splits=false))`, `enemies.extend(spawned)` (AN-04) | `spawned` | none (default: the function's own new list) | none (default: its own; handed over by `extend`, row 9) |
| `enemies.retain(fn(e) => e.hp > 0)` (AN-04) | `e` | none (default: a lambda parameter with no word, a read-only view of each element) | none (default: read only) |
| `for ref mut e in enemies:` (AN-04) | `e` | `ref mut`: the current element of `enemies` | `ref mut`: yes |
| `dead.push(i)`, then `while dead.len() > 0: on_death(ref mut enemies, dead.pop().unwrap())` (AN-05) | `dead` | none (default: the function's own new list of positions) | none (default: its own) |
| `ref mut on_death: fn(ref mut Array[Enemy], int)` (AN-05) | `on_death` | `ref mut`: the caller's function value; its type: a function that takes the list `ref mut` | `ref mut`: the call may change the function value's own state |
| `update(ref mut es, ref mut count)` (AN-05) | none declared | `ref mut` at the call says `update` may change `es` and `count` | — |
| `Query[(Entity, ref mut Health)]` (AN-06) | none declared | the query's types: `ref mut Health` says the system changes health | — |
| `fn words_of(text: StrView) -> Array[StrView] from text:` (AN-07) | `text` | `StrView`: a view of the caller's text | `StrView`: read only |
| `all_words.push(String(w))` (AN-07) | `all_words` | none (default: the function's own list, made before the loop) | none (default: its own) |
| the same line | `w` | none (default: a plain loop name, a read-only view of one word) | none (default: read only) |
| `words: Array[StrView] = ...` (AN-07) | `words` | its type `Array[StrView]`: the function's own list of views | none (default: its own; its views only read) |
| `return String(word)` (AN-09) | `word` | none (default: a read-only view of the lowered copy, row 5) | none (default: read only) |
| `es[0].heal()` (AN-10) | `es` | none (default: `main`'s own list) | none (default: its own; `heal`'s first line, `changes hp`, says it changes `es[0].hp`) |
| `e = copied es[0]`, then `e.heal()` (AN-10) | `e` | `copied`: a copy of your own | none (default: its own copy) |
| `fn heal() changes hp:` (AN-10) | `this` (in its body) | `this`: the enemy `heal` was called on | `changes hp`: it may change `hp` only |
| `for ref mut p in es:` (AN-10) | `p` | `ref mut`: the current enemy of `es` | `ref mut`: yes |
| `ref mut pos: MutSpan[float]` (AN-11) | `pos` | `ref mut`: the caller's own view; `MutSpan`: of the caller's elements | `ref mut`: may point the caller's view elsewhere; `MutSpan`: may write its elements |
| `integrate(ref mut ps, ...)` (AN-11) | none declared | `ref mut` at the call says `integrate` may point `ps` elsewhere | — |
| `rest = other` (AN-11) | `rest` | none (default: a new view of your own, the value of the view `other`, R2) | none (default: by its type, as `other`) |
| `vel: MutSpan[float]` (AN-11) | `vel` | `MutSpan`: a view of the caller's elements | `MutSpan`: its elements may be written |
| `counts[w] = counts.get_or(w, 0) + 1` (AN-12) | `counts` | none (default: the function's own map) | none (default: its own) |
| the same line | `w` | none (default: a plain loop name, a read-only view of one word) | none (default: read only) |
| `p = ref mut ps[0]` (AN-13) | `p` | `ref mut`: particle 0 of `ps` (a position) | `ref mut`: yes, the only name for it until its last use |
| `p = ps.get(0).unwrap()` (AN-13) | `p` | none (default: what `get` gives, a read-only name of particle 0) | none (default: read only) |
| `snapshot = copied ps[0]` (AN-13) | `snapshot` | `copied`: a copy of your own | none (default: its own) |
| `assert_disjoint(dst, src)`, `match check_disjoint(dst, src):` (AN-14) | `dst` | `MutSpan` (its type): a view of the caller's elements | `MutSpan`: its elements may be written |
| the same lines | `src` | `Span` (its type): a view of the caller's elements | `Span`: read only |
| `old = copied names` (AN-14) | `old` | `copied`: a copy of your own | none (default: its own) |
| `for ref mut pos, ref vel in q:` (AN-15) | `pos` | `ref mut`: the current entity's `Position` | `ref mut`: yes |
| the same line | `vel` | `ref`: a read-only view of the entity's `Velocity` | `ref`: read only |
| `Query[(ref mut Position, ref mut Velocity)]` (AN-15) | none declared | `ref mut Velocity` says the system changes velocities | — |

   Every name has its words, or a stated default that never changes anything it does not own, never
   aliases silently and never copies silently.

5. **Means exactly:** the same programs are refused (or accepted) for the same reasons as in each
   answer's point 6, with the codes there; only the text changes. The translation is the table in point 2
   (today's text) against point 6 of each answer (the new text).

6. **Still refused:** the refusals of AN-01 to AN-15, each with its message there.

7. **Cost:** none while the program runs.

8. **Kind:** a readable spelling (of the refusals).

## 4. Words needed

Every word in the NEW code above has a row in `RUN4/vocabulary.md`. One name run 3's NEW used has none:

| word | why | what this file does |
|---|---|---|
| `Vec3.ZERO` | AN-13's run-3 NEW wrote `pos=Vec3.ZERO`; `ZERO` (a constant of `std.math`'s `Vec3`, `std/src/math/mod.em:363`) has no row, while `Vec3` has one (section 3.8) | not used: AN-13 writes `Vec3(0.0, 0.0, 0.0)`, the value `ZERO` is defined as. A row for `ZERO` (and `Vec3`'s other constants) would let AN-13 keep run 3's spelling; it says nothing about owning or changing, so it is a library-name row like those of section 3.8 |

## 5. For the owner, and found for the lead

For the owner (the lead raises these in chat):

1. **A method that changes a temporary whose change is then lost** (AN-10). `enemy_at(es, 0).heal()`,
   where `enemy_at` returns the enemy by value, heals a copy that ends on the same line, so `es[0]` never
   heals. Run 3 refused it by its area rule A-6 (allowed when the call's result is used, as in
   `line.split(" ").next()`; refused when nothing it returns is used). Neither run 3's shared rules nor
   `RUN4/rules.md` state that rule, and `RUN4/rules.md` section 4 has no code for it (today's compiler
   uses E2140). Without it the program compiles and does not do what it means. Should the rules keep
   A-6, with a code (today's E2140, "a method that changes its object was called on a temporary whose
   change nothing uses")? AN-10 keeps the refusal meanwhile.
2. **`ref mut` alone, in this area** (the rule writer's question 3, `RUN4/changes.md` section 13): AN-01
   and AN-02 are its examples. While `boss = ref mut enemies[0]` is still used, `enemies[0].hp = 1`, and a
   helper given `ref mut enemies` that writes or reads an element already in the list (`heal_all`), are
   now refused; run 3 accepted both. A helper that only adds (`spawn_minion`) stays accepted. Nothing
   else in area A changes with it.

Found for the lead (consistency of `RUN4/rules.md` and `RUN4/vocabulary.md` with this area; none needs
the owner):

3. **The "only adds" summary needs one more fact** (AN-02). R8 (f) lets a call that only grows the list
   hold a `ref mut` position's element; R17 records "area A's 'only adds' summary". For R8 (f) to be
   checked through a call, that summary must also say the callee reads and writes no element already in
   the list (section 0 of this file defines it so). R17's text names only "only adds".
4. **`check_disjoint` and `assert_disjoint`** (AN-14). Today's `std/src/mem.em:76, :85` take both views
   `owned`; under R4, R2 and E3013 that refuses a call with a plain parameter (`blend_any(dst, src)`),
   and `owned` on a `Span` is E2085. This file has them read their arguments and give back views (R6).
   The vocabulary's `assert_disjoint` row still lists "a view used after it took it" under "stays
   refused"; with nothing taken, that case is accepted, and only the dropped result is warned (W2190).
5. **Three refusals of this area have no code** in `RUN4/rules.md` section 4: `copied w` of a view (AN-07;
   the vocabulary `copied` row gives its message); `ref mut` on a loop name whose query item only reads
   (AN-15, `for ref mut pos, ref mut vel in q:` with `Velocity` in the query); and AN-10's lost change
   (point 1).
6. **Is a local list of views read-only?** (AN-07). R2 makes "a name bound to ... a call whose result is
   a view" a read-only view, and R7 says "a local list of views is itself a view", so read together
   `words = ....collect()` would be read-only (no `words.push(...)` afterwards). This file reads `words`
   as the list's owner (R1: `collect()` makes a new list and hands it over), bounded in life by R7. AN-07
   does not change `words`, so its meaning does not depend on the reading; a program that pushes to such a
   list does.
