# 08 area A: lists, loops, temporary values (run 3, phase 2)

Area A answers AN-01 .. AN-16 of `RUN3/annoyances.md`, keeps agreed row 5 and the owner's "delete after
the loop" exactly, and repairs last run's report for this area (`tasks/96/redesign/phase3b-containers.md`
and `phase3b-containers-attack.md`). Words used: only those of `RUN3/vocabulary.md` and
`RUN3/planned-words.md`, plus one library method in the "new words" table (section 5).

Power: `PowerLineStatus` printed `Online` at the start, before the timing in AN-04, and at the end.

Every claim about today's Ember carries the real output of `target/redesign3/bin/ember.exe check`,
pasted here by `target/redesign3/08-area-A/render.py` from `old/check_output.txt` and
`means/check_output.txt` (written by `check.sh` in the same folder), so the pasted text is the
file's text. No Ember program was run. C programs include `harness.h`, were built with clang -O2,
run directly from Git Bash, and run once more under AddressSanitizer where they are a "means exactly".

## 0. Terms

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

## 1. The short version

| id | the answer in one line | kind | what runs while the program runs |
|---|---|---|---|
| AN-01 | `boss = ref mut enemies[0]` names position 0; adding to `enemies` while `boss` is in use is allowed (the buffer is read again); removing or reordering is refused at compile time | smarter compile time | one reload of the buffer pointer after an add (C reloads too) |
| AN-02 | a call whose summary only adds keeps `boss`; a call that may do more, or unseen code, is refused with the two lines that fix it | smarter compile time | nothing more |
| AN-03 | two `ref mut` parameters are two different places; the compiler proves the positions differ (`j in i + 1..`) or checks them once at the call | readable spelling of Rust's guarantee | one compare per call where not proven (Rust: the same, `get_disjoint_mut`) |
| AN-04 | kept: a list may not change while a loop walks it; the change goes after the loop (`retain`, `extend`), and the compiler fuses the loop and the `retain` into one pass | understandable limitation kept | two passes 1.71x of one-pass C as written; fused 1.00x (the same C) |
| AN-05 | kept: a loop may not call code that may change the list it walks; and a counting loop over `0..xs.len()` may not shrink `xs` (today accepted, it skips and then stops) | understandable limitation kept | a list of positions, then the calls after the loop |
| AN-06 | kept: an ECS system queues `destroy`, and now `create`, on `Commands`; applied after it returns | understandable limitation kept | one queued record per change, applied after the loop, as Bevy |
| AN-07 | `words = line.to_lower().split_whitespace().collect()` is an `Array[StrView]` that keeps the lowered text until `words`' last use | smarter compile time | nothing: views, no copy per word |
| AN-08 | row 5, as agreed: the lowered string lives until `word`'s last use | smarter compile time | nothing |
| AN-09 | row 5, as agreed: returning the view stays refused, the message names `word`, and `String(word)` exists | understandable limitation kept | one allocation, or none when the dying text's buffer is reused |
| AN-10 | a `mut self` method on a temporary is allowed when its result is used, refused when the change would be lost | smarter compile time | nothing |
| AN-11 | `pos: MutSpan[float]` writes its elements with no other word; re-pointing it is `ref mut pos` | readable spelling | nothing |
| AN-12 | `counts[w] = counts.get_or(w, 0) + 1`, compiled as one lookup | readable spelling | one hash and probe per word |
| AN-13 | `SoA.get(i)` names element `i` as `Array.get(i)` does; a copy is `copied ps[i]` | readable spelling | less than today (no gather) |
| AN-14 | `assert_disjoint(dst, src)` stops the program on overlap and marks both names non-overlapping; `check_disjoint` reports | readable spelling | two compares (as today) |
| AN-15 | `Query[(ref mut Position, Velocity)]` replaces `Mut[Position]` | readable spelling | nothing |
| AN-16 | every message of this area names the program's own names and types and gives the line to write | readable spelling | nothing |

No new mechanism is needed in area A, and nothing is moved from compile time to run time: the only
run-time checks are bounds checks (as today), one compare in AN-03 where the compiler cannot prove two
positions differ (Rust's `get_disjoint_mut` makes the same compare), and two compares in
`assert_disjoint` (the spec's `[DSJ-1]`, as today).

## 2. The rules of area A

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

**A-4. Two `ref mut` parameters are two different places.** A function with two `ref mut` parameters
is compiled with noalias on both, as Rust's two `&mut`. A call that passes two elements of one list
must give two different positions: proved at compile time when it can be (constants that differ,
`j` from `i + 1..`, inside `if i != j:`), otherwise checked by one compare before the call, which stops
the program with a message naming both elements. The same holds for two `ref mut` element names in one
function.

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

## 3. The answers

### AN-01: Holding an element while adding to the list (A2)

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

### AN-02: A struct element's name used after a call that may change its list (last run's limit)

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
| `x` (parameter) | none (default: a number passed in, the function's own value) | none (default: read; changing it changes nothing the caller holds) |
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

### AN-03: Two elements of one list in one call (A3)

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

### AN-04: Changing a list while looping over it (A4)

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

### AN-05: A loop that removes from its own list while calling code the compiler cannot see (last run's limit)

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

### AN-07: A local list of string views (`words = text.split()`) is refused (A25)

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

### AN-08: A view of a temporary kept past its statement (A22)

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

### AN-09: Returning a view of a temporary: the message names `_5`, and the agreed advice `String(word)` is refused today

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

### AN-10: A `mut self` method on a temporary (`line.split(" ").next()`) is E2140 with no help; Rust accepts it

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

### AN-11: A `MutSpan` parameter cannot be written without also writing `mut`; the spec's own examples are refused; E3023's help offers `RefCell` and `class`

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

6. **Still refused:**
   - re-pointing the view with plain `pos` (`pos = other`): "`pos` is the view the caller gave you; you
     may write its elements (`pos[i] = ...`) but not point it elsewhere. To point the caller's view
     elsewhere, write `ref mut pos: MutSpan[float]`; for a view of your own, use a new name: `rest = other`."
   - `mut pos: MutSpan[float]`: "remove `mut`: `MutSpan` already lets `integrate` write `pos[i]`; to
     point the caller's view elsewhere, write `ref mut pos: MutSpan[float]`" (C-1).
   - writing through `vel: Span[float]`: "`vel` is a read-only view; to write its elements, take it as
     `vel: MutSpan[float]`." No `RefCell` or `class` help; the type as written (`float`, not `f64`).

7. **Cost:** none; the same code as today's `mut pos`.

8. **Kind:** a readable spelling (the type says it once).

### AN-12: Updating a map entry in place (A21): `counts.get(w).unwrap_or(0)` is E2020, there is no `get(k, default)`

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
   does (`an12c`, accepted); see section 7 for the combiner.

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
    for ref mut pos, vel in q:
        pos.x += vel.dx * dt
```

   `ref mut Position` in the query's type says the system changes positions (it sets the system's write
   set, `[ECS-4]`, as `Mut[Position]` does today); `ref mut pos` on the loop name says this name
   changes them (C-1). `std.ecs` maps `ref mut T` to the item `ref mut T` exactly as it maps `Mut[T]`
   today (`std/src/ecs.em:406`, the `QueryParam` interface).

4. **The owner's test:**

| name | what it is | may it change it |
|---|---|---|
| `q` | `ref mut`: the query the world made for this run | `ref mut`: yes |
| `pos` | `ref mut`: the `Position` of the current entity | `ref mut`: yes |
| `vel` | none (default: a plain loop name, a read-only view of the entity's `Velocity`) | none (default: read only) |
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
     `for ref mut pos, vel in q:`."
   - `for pos, ref mut vel in q:` when the query says `Velocity`: "the query reads `Velocity` only; to
     change it, ask for it: `Query[(ref mut Position, ref mut Velocity)]`."

7. **Cost:** none; the same code as `Mut[Position]`.

8. **Kind:** a readable spelling (one spelling for "changes it").

### AN-16: Messages use the compiler's names, not the program's, and many helps do not give the line to write (A17)

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

4. **The owner's test:** no names are bound in this answer (it is about message text); the names in the
   new messages are the program's own names, which is what M-1 checks.

5. **Means exactly:** the same programs are refused (or accepted) for the same reasons as in each
   answer's point 6; the codes stay; only the text changes. The translation is the table in point 2
   (today's text) against point 6 of each answer (the new text).

6. **Still refused:** the refusals of AN-01 to AN-15, each with its message there.

7. **Cost:** none while the program runs.

8. **Kind:** a readable spelling (of the refusals).

## 4. The agreed rows kept

| row | how area A uses it | unchanged? |
|---|---|---|
| 5 | AN-08 is row 5's program; AN-09 is row 5's refusal and message ("return `String(...)` to hand back your own copy", inside the new message); AN-07 applies row 5's rule to a list of views; `String(word)` is the constructor the message names | yes |
| 8 | `ref mut` on every name that changes someone else's value: `boss = ref mut enemies[0]`, `for ref mut e in enemies:`, `for ref mut pos, vel in q:`; a plain name reads (as row 8's plain `path`) | yes |
| 9 | `extend(spawned)` hands `spawned` over; `copied` makes a copy (`boss = copied enemies[0]`, `copied ps[0]`) | yes |
| "delete after the loop" (JOB) | A-3, AN-04, AN-05, AN-06: a list may not change while a loop walks it; `retain`, `extend` and `Commands` after the loop; no mechanism allows the change inside the loop (the fusion of AN-04 changes speed only) | yes |

## 5. New words

| word | meaning | where written | reason | example | answers |
|---|---|---|---|---|---|
| `Commands.create()` | a library method (not a keyword): a new entity id, given at once, whose entity exists once the system's queue is applied | a call on a system's `Commands` | the agreed "delete after the loop" needs its other half, "create after the loop", inside a system; today `Commands` has only `destroy`, `add`, `remove` (`std/src/ecs.em:1219-1237`) and `create` is E2072 (AN-06); the name is `World.create()`'s (`std/src/ecs.em:216`), so one operation keeps one name | `n = q.commands().create()` | AN-06 |

No keyword is added. Every other word used is in `vocabulary.md` or `planned-words.md`: `ref mut`
(including the parameter and loop positions of C-1), `copied`, `StrView`, `String(...)`, `MutSpan`,
`Span`, `retain`, `extend`, `get_or`, `check_disjoint`, `assert_disjoint`, `Commands`, `Query`,
`from` (in one message, the result clause).

## 6. Last run's report, repaired

Last run's answers for this area (`phase3b-containers.md`, saved in `phase3b-containers-attack.md`
section 9) were judged against this run's two new constraints: the owner's ruling that a list may not
change while a loop walks it ("no clever mechanism to allow the change inside the loop is wanted"), and
the owner's test (a plain name never changes anything, never aliases harmfully, never copies silently).

| last run's part | kept, changed or dropped | why |
|---|---|---|
| A2 rules 1, 2, 4 (a name is a position; read again after an add; reordering refused) | **kept** as A-1 and A-2 | compile time, the same instructions as C |
| A2 spelling `boss = enemies[0]` writing through the plain name | **changed**: writing needs `ref mut`, a copy needs `copied` | the owner's test: a plain name may not change the list's element (row 8) |
| A2 rule 3 (the name follows its element through removals; a removed element's value moved into the name, with one-bit flags) and rule 6 (a "place" passed into functions, moved by their removals) | **dropped**, replaced by A-2's compile-time refusal with the two lines that fix it | it made the compiler decide something the code did not say (which element `boss` names after a removal), and needed run-time flags; the refusal is simpler and equally fast |
| A2 rule 5 and A4 rules 9-10 (the walkers count for unseen code, run-time stops) | **dropped** | the owner's ruling settles it at compile time; it was a check moved to run time |
| A3 "two names may be one element" (no noalias; functions compiled twice; old buffers kept alive; slices copied) | **dropped**, replaced by A-4 (Rust's guarantee, proved or checked once) | it gave up noalias (1.53x on spans) and needed four repair rules after its attack found three memory holes (`phase3b-containers-attack.md` 4.2, 4.6); A-4 has none of those cases |
| A4 one-pass filter (removing inside the loop) | **dropped** as a meaning; its code is **kept** as the fusion of the loop and the `retain` after it | the owner's ruling; the fused code is the same fast C |
| A25 (a): a local list of views carries its text's life | **kept** as A-5 for lists (AN-07) | compile time, 1.00x |
| A25 (b): a returned list takes its text along in a hidden owner block | **dropped** | the list would own a text no word in the program names (the brief: "the compiler never decides what the code does not say"); returning views of a dying text is refused with two lines instead |
| ECS: walk backwards, a cursor in each pool, destroy inside the loop | **dropped** | the owner's ruling (`Commands`); and its attack found it held only if every leave operation applied the cursor (`phase3b-containers-attack.md` 7.5) |
| ECS: hold the pools' arrays in locals across the loop | **kept**, now always valid | nothing changes a pool while a loop walks it |

### Good parts of what was dropped

| good part | from | where it may help later |
|---|---|---|
| The index adjustment `j -= (k < j)` after a removal (one compare, one subtract) | last run's A2 rule 3 | if the owner ever wants Python's "the name follows the object" for removals written in the same function |
| A count of running loops in the capacity word's top byte: +1.4% on `push` (`phase3b-containers.md:504-509`) | last run's walkers count | a cheap run-time guard, if one is ever wanted for unseen code |
| One unsigned compare for "do two ranges overlap" (`\|d - s\| < n`), which fixed a mispredicted two-compare test (187-190 ms to 94 ms) | last run's A3 save (`phase3b-containers.md:361-366`) | the two compares of `assert_disjoint` and `check_disjoint` |
| Without noalias, two spans of one list cost 1.53x; with a call-site check 1.07x | last run's A3 measurements | the reason A-4 keeps noalias everywhere |
| Load the element once, update it in registers, store it once (a store-forwarding stall made the first version 4.7x) | last run's A4 (`phase3b-containers.md:102-114`) | the fused loop of AN-04 is emitted that way |
| A list that owns the text its views point into, at 1.00x of plain views | last run's A25 (b) | the mechanism behind area B's explicit `from` field (`words: Array[StrView] from text` in a type that owns `text`, AN-25), where the program says it |
| Walking a pool backwards, with a cursor that keeps "every entity once" under destroys (checked over 5,000 random loops) | last run's ECS answer and its attack | if in-loop destroy is ever wanted, as EnTT allows |
| Owned short strings at 1.46x instead of 2.34x for one allocation per word | last run's A25 (`owned_sso.c`) | the cost of `String(w)` in the refusals' advice, if `String` gains a short-string layout |

## 7. For the combiner and the other areas

- **A-1 and area C (AN-44).** A-1 says what plain `x = xs[i]` means for a list element (a read-only
  name, never a silent copy); area C rules on `b = a` for a whole variable. The two must read as one
  rule. Today's silent copy of a `Copy` struct element (`an01c`) is refused under A-1.
- **Summaries for `pub` functions (AN-02).** "only adds" is inferred, as `[BRW-10]` field sets are,
  and must be written into the package's interface file; the toolchain should report a change of a
  `pub` function's summary as an interface change. No word is proposed: the function's line already
  says `ref mut enemies`, and the summary changes only whether a caller compiles, never what it means.
- **Map keys (AN-12).** `counts[w] = v` with `w: StrView` and `String` keys copies `w` into a new key
  when it is new, with no word, as today (`an12c`). Requiring `counts[String(w)]` would say it, at an
  allocation per update. Area C (copies) may rule.
- **Area B, AN-23:** `counts.entry(w).or_default() += 1` needs a call returning `ref mut` to be a place.
- **Area D, AN-60:** a dispatch loop `for h in self.handlers:` that calls handlers which may change
  `self.handlers` falls under A-3 (refused, with "collect and act after the loop").
- **Area E:** `assert_disjoint`'s fact belongs to names in one function; a `Commands.create()` in a
  `run_parallel` system takes its id with an atomic add.
- **Run-time checks for phase 4 (W1, the ECS):** none new from area A. Bounds checks as today; the
  queued changes are applied after each system; the loop holds the pools' arrays in registers.

## 8. Files

All under `target/redesign3/08-area-A/`:
- `old/*.em` and `old/check_output.txt`: every OLD program and its `ember check` output.
- `means/*.em` and `means/check_output.txt`: every "means exactly" in today's Ember (all `exit=0`).
- `c/a4/`: `common.h`, `plain.c`, `two_pass.c`, `rust_idiom.rs` (with `harness.h` and `harness.rs`, copies
  of the shared harnesses), timing in `runs_an04.txt`.
- `c/an06/commands.c`, `c/an06/asan_output.txt`; `c/an07/views.c`, `c/an07/asan_output.txt`.
- `check.sh` (runs `ember check` on a folder), `render.py` and `report_template.txt` (build this report
  with every program and output pasted from the files).
