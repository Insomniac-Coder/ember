# Phase 3b: containers and loops (A2, A3, A4, A25, and the ECS view loop)

Written 2026-10-24 by the containers-and-loops agent, for the owner's job in [JOB.md](JOB.md):
speed on par with C and Rust, memory safety the same as Rust's or better, and Python-like code,
all at once, with the checking done when the program is compiled wherever that is possible.

Nothing in the language, the compiler or the standard library was changed. Every Ember program
below was checked with the real compiler (`target/ci/ember.exe check`, Ember 0.1.0, language
0.9.10); no Ember program was run. Every timing is mine, made on this laptop under the rules in
section 1.3, and every one is **PROVISIONAL**: other agents were running at the same time, so a
difference under 15% waits for the final measurer.

Files: Ember probes in `target/redesign2/containers/ember/` (their compiler output in
`ember_check_output.txt` there), the C and Rust programs in `target/redesign2/containers/bench/`,
raw timings in `target/redesign2/containers/bench/runs/`, the Python checks in
`target/redesign2/containers/py_checks.py` (output in `py_checks_output.txt`, Python 3.14.2), and
three `rustc` checks in `target/redesign2/containers/rust/` (output in `rustc_output.txt`).

---

## 0. The short version

**One idea carries A2, A4 and the ECS: a name for a list element is its position, not its
address.** Rust refuses these programs because a reference is an address, and adding to a list can
move the list's elements to a new block of memory (the old block is freed), so the address would
point into freed memory. If the compiler keeps a name such as `boss = enemies[0]` as "element 0 of
`enemies`", and holds the address only while nothing can change the list, then after a change it
simply looks the element up again. Looking it up again is what a careful C programmer and Rust's
own workaround do by hand, so it costs what they pay. All of this is decided when the program is
compiled.

| Annoyance | The answer | Kind | Cost measured (provisional) | What is left |
|---|---|---|---|---|
| A2: holding an element while adding to the list | names are positions; looked up again after a change | (b) smarter compile time | zero against Rust's workaround: the same instructions (section 2) | sorting the list while the name is still needed is a compile-time error |
| A3: two elements of one list in one call | two names may be one element; only a view into a part that a write can replace is checked | (b) | elements: 1.00x of C with `restrict`; spans: 1.07x with a check at the call, where safe Rust took 2.1x | holding a view into one element's text across a write that replaces it through the other name is a compile-time error |
| A4: changing a list while looping over it | the loop is compiled into a one-pass in-place filter | (b), plus a run-time fallback for code the compiler cannot see | 0.98x of the fastest plain C; safe Rust's idiom 1.62x | reordering the list inside its own loop is a compile-time error; a change made by unseen code that would reorder the list panics |
| A25: a local list of string views | local lists carry the lifetime of their text; a returned list takes its text along | (a) for local use, (b) for returning | 1.00x of C views (Rust 1.05x); owned words 2.34x, short-string words 1.46x | one list holding views of two texts with different lifetimes cannot be returned without copies |
| ECS: create and destroy entities inside a view loop | the library walks the pool backwards by position, components are names keyed by the entity, a cursor keeps "every entity once" | (b) | 1.03x to 1.12x of plain C in four runs; safe Rust 1.05x to 1.15x; last night's three designs 1.41x to 2.33x in the same session | nested view loops over one pool that destroy other entities keep "every entity once" only for the inner loop (memory safety is unaffected) |

Two first versions were slow and were saved (section 1.4): the A4 loop (4.7x, a CPU store-buffer
stall, fixed by keeping the element in registers) and the ECS loop (1.34x, fixed by holding the
pools' arrays in locals between changes). Last night's three designs are timed beside the answer, in
the same session, in section 6 (the ECS); section 4 times the two-pass shape that two of them used for
loops.

---

## 1. How to read this report

### 1.1 Words used

- **List**: Ember's `Array[T]`, Python's `list`. Its header is three words: a pointer to a block of
  memory holding the elements (the **buffer**), the length, and the capacity
  (`runtime/ember_rt/include/ember_rt.h`, `ember_vec`, line 2022).
- **Reallocate**: when a full list grows, it gets a bigger buffer, copies the elements there and frees
  the old one. Any address into the old buffer then points into freed memory.
- **Reference** (Rust `&T`, Ember `ref T`): the address of a value. A **view** (`str`, `Span`) is an
  address plus a length.
- **Bounds check**: the test `i < len` before reading `xs[i]`; failing it is a panic (the program
  stops with a message, `[PAN-1]`), never a read of the wrong memory.
- **Panic**: a deterministic stop with a message. Ember's panics abort; there is no unwinding
  (`[PAN-1]`). Several answers below rely on that.
- **Effect summary**: what the compiler records about a function: which lists it may add to, remove
  from or reorder. Ember already computes the fields a private method touches (`[BRW-10]`); the
  answers below need that extended (section 7).
- **`restrict` / noalias**: a promise to the C compiler that two pointers never point at the same
  memory, which lets it reorder and vectorise. Rust gives it for every `&mut`.

The three designs from last night are named by what they did: the **copy-on-write values design**
(`phase3-design-D1.md`), the **checked-pointers design** (`phase3-design-D2.md`), and the
**prove-or-check design** (`phase3-design-D3.md`).

### 1.2 The answer kinds (from JOB.md)

(a) a Pythonic way of writing what Rust does, with the same compile-time guarantee; (b) a smarter
compile-time solution, so Rust's rule is not needed; (c) a new mechanism, as long as it is clever,
easy to read and write, on par with C and Rust in speed, and as safe as Rust.

### 1.3 How the timings were made

- Built with `"C:/Program Files/LLVM/bin/clang.exe" -O2 <file>.c -o <file>.exe` (clang 22.1.8) and
  `rustc -C opt-level=2 --edition 2021 <file>.rs -o <file>.exe` (rustc 1.98.1).
- C programs use the shared harness (a copy of `target/redesign/bench/harness.h`: no error dialogs,
  wall time, peak working set, checksum). The Rust programs include `bench/rharness.rs`, which makes
  the same calls and uses the same random generator.
- Run with `powershell -NoProfile -File bench/run.ps1 runs/<name>.txt <exe> ...`: 5 rounds, one
  program at a time, each pinned to the performance cores (`ProcessorAffinity = 0xC03C03`), order
  forward in odd rounds and reversed in even rounds; the script checks
  `(Get-CimInstance Win32_Battery).BatteryStatus` is 2 (mains) before every round. No round was
  aborted and no affinity warning was printed in any results file.
- Medians with `bench/summarize.sh runs/<name>.txt`. Every run of every program printed the same
  checksum as the plain C program it is compared with (`a25/gen_only.c`, which only generates the
  text, is not a variant and prints its own).
- "1.05x" means the median time divided by the plain C median from the same results file.
- Like every phase 3 model, the C models leave out Ember's integer overflow checks; plain C has none
  either.

### 1.4 Two answers that were slow at first, and what fixed them

The job says to try to save an idea before calling it a failure. Two of mine needed it.

**The A4 loop, 4.7x, then 0.98x.** My first C model of the in-place filter updated each enemy where
it lay (two 4-byte stores) and then copied it down with one 16-byte load and store. That took
444.6 ms against 93.9 ms (`runs/a4.txt`, `plain_naive.c`, `plain.c`). The assembly of
`plain_naive.c` shows why: `movss %xmm0, -4(%rax,%rdi)` and `decl -8(%rax,%rdi)` store x and hp, then
`movups (%rcx), %xmm0` loads the whole 16-byte element. A load that spans two smaller stores still
waiting in the CPU's store buffer cannot take its data from them and waits until they reach the
cache (a store-forwarding stall). Check: writing the copy field by field (four 4-byte loads, no
16-byte load) brought it to 135.1 ms (`plain_fieldcopy.c`, `runs/a4.txt`); loading the element once
into registers, updating it there and storing it once at its final place brought it to 93.9 ms
(`plain.c`). The compiler can emit the second form because nothing but the loop variable touches
the element during the iteration, which it can see. Rust's `retain_mut` has the same update-then-copy
shape and took 439.9 ms (`rust_retain_mut.rs`); that its cause is the same stall is inferred from
the shape and the time, not from its assembly (UNVERIFIED).

**The ECS loop, 1.34x, then 1.03x to 1.12x.** My first ECS model read each pool's buffer pointers
and lengths through the pool structs on every step and made two bounds checks per step (150.2 ms
against 111.9 ms, `runs/ecs_1.txt`, `ember_direct_naive.c`). The rare branch calls functions that
change the pools (`add` is not inlined in that model), so the C compiler had to assume any step
might see changed pools. The language knows more than C: on the common path nothing can change a
pool, so the compiler may hold those values in locals, read them again only after the rare branch,
and check the position once there instead of on every step. That is the same rule as for element
names, applied to the loop's own state (`ember_direct.c`, section 6).

---

## 2. A2: holding an element while adding to the list

### 2.1 What Rust's rule protects against

From phase 1 (`phase1-borrowing.md`, A2, checked there with `rustc 1.98.1`):

```rust
pub fn split_boss(enemies: &mut Vec<Enemy>) {
    let boss = &mut enemies[0];
    enemies.push(Enemy { x: boss.x + 1.0, hp: boss.hp / 2 });   // error[E0499]
    boss.hp /= 2;
}
```

`push` may reallocate: the buffer that `boss` points into is freed, and `boss.hp /= 2` would write
into freed memory. Removing an element instead (`enemies.remove(0)`) shifts the others down, so
`boss` would name a different enemy; and if the element owns memory (an enemy with a `name:
String`), a pointer to a removed element reads a string that was freed with it.

### 2.2 The Python program, and what Ember does today

| Python | Ember (`ember/a2_python.em`) |
|---|---|
| `boss = enemies[0]` | `boss = enemies[0]` |
| `enemies.append(Enemy(boss.x + 1.0, boss.hp // 2))` | `enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))` |
| `boss.hp //= 2` | `boss.hp = boss.hp // 2` |

Python: `boss` is the same object before and after the append (`py_checks_output.txt`: "A2 after
append: [E50, E50]"). Ember today:

```text
error[E3040]: `enemies` is borrowed after it has been moved out of
7 |     boss = enemies[0]
  |            ---------- moved here
  = note: assigning a list, a map or a set moves it; in Python both names would point at one list (Appendix E)
```

With today's explicit reference, `boss = ref mut enemies[0]` (`ember/a2_ref.em`):

```text
error[E3022]: `*enemies[0]` is already mutably borrowed
  = help: end the borrow before this: `boss` is what keeps it alive, so shorten its last use or put it in a block of its own
```

And the Python way of taking an item out of a list and keeping it (`ember/a2_take.em`:
`picked = items[i]; items.remove(i); bag.push(picked)`) gives `E3011 cannot move out of an element`
and `E3040`.

### 2.3 Answers

**(a) Pythonic spelling, same rule.** `boss = enemies[0]` names the element (no `ref mut`, and not a
move), and the compiler keeps Rust's rule: the `push` while `boss` is used afterwards is still
refused, with the rewrite as a fix-it. Cost: none. What it lacks: the annoyance itself stays; only
the spelling improves. Not chosen; its spelling is kept inside answer (b).

**(b) Names are positions, looked up again after a change (chosen).** The rules:

1. `boss = enemies[0]`, where the element is not a plain number, makes `boss` a name for the
   element at position 0 of `enemies`. The position is computed once (`boss = enemies[i]; i += 1`
   does not move `boss`). A number is copied, as Python's integers behave.
2. The compiler keeps the element's address while nothing can change `enemies`. After anything that
   may add to it (a `push` written there, or a call whose effect summary says it may add), it reads
   the buffer pointer again. No bounds check is needed then: adding never shortens the list.
3. After a removal or insertion written in the same function, the name follows its element: removing
   position `k` before it moves it down by one (`j -= (k < j)`, one compare and one subtract);
   inserting before it moves it up. If the name's own element is removed, the name keeps the removed
   value, which is Python's meaning (you still hold the object you removed): the value is moved into
   the name's own slot instead of being dropped, with a one-bit flag when the compiler cannot tell
   whether it was this element.
4. Sorting, reversing or swapping the list while the name is still needed afterwards is a
   compile-time error: "`boss` names enemies[0], which `sort` on line 9 may move; name it again after
   the sort". The same for `pop()` of the named element when the element owns memory (the value
   cannot have two owners); for plain data the name keeps a copy.
5. A change made by code the compiler cannot see into (a stored callback, a virtual method, a
   foreign function) is covered by the walkers count of section 4.3: while the name is needed across
   such a call, a reordering or removal inside it panics instead of silently changing which element
   `boss` names. Adding is always allowed.
6. Passing `boss` to a function receives an address. If that function may change `enemies` (its
   summary says so), a plain-data element is passed by copy and written back after the call
   (Swift's in-out), and an element that owns memory is a compile-time error with the fix-it "pass
   `enemies` and the position". Otherwise it is the address, as today.

| Python | Ember under (b) | What the compiler emits (C) |
|---|---|---|
| `boss = enemies[0]` | `boss = enemies[0]` | `size_t b = 0;` |
| `enemies.append(Enemy(boss.x + 1.0, boss.hp // 2))` | `enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))` | `Enemy t = {data[b].x + 1.0, data[b].hp / 2}; push(&enemies, t);` |
| `boss.hp //= 2` | `boss.hp = boss.hp // 2` | `data = enemies.data; data[b].hp /= 2;` |
| `picked = items[i]` | `picked = items[i]` | (position i) |
| `items.remove(picked)` | `items.remove(i)` | the element is moved into `picked`'s slot instead of being dropped |
| `bag.append(picked)` | `bag.push(picked)` | `push(&bag, picked_slot);` |

The same program written by hand with positions, `ember/a2_emitted.em`, is accepted by today's
compiler (exit 0), which shows that today's rules already prove it safe; the answer is that the
compiler writes it.

**Why it is memory-safe.** Every use of a name either goes through an address held only across code
that the compiler proved cannot change the list, or reads the buffer pointer again and (after
anything that may shorten the list) checks the position against the length. A position that is out
of range panics; it can never reach freed memory. The walkers count and the index adjustments only
keep the name on the right element; if they were wrong, the result would be the wrong element, not a
memory error. Attacks tried: push that reallocates (looked up again); `clear()` (a removal: the name
keeps its value); a callee that pushes (summary: looked up again); a callee that removes (summary:
compile-time rule 3 or 4; unseen: walkers count); a name into a nested list `grid[i][j]` whose outer
list grows (the whole path is looked up again: two loads); a name passed to a function that grows the
same list (rule 6).

**Cost.** Against Rust's workaround (`a02_container_invalidation_fix.rs` in phase 1: copy the fields
out, push, index again) the emitted code is the same operations: one reload of the buffer pointer
after the push and one indexed store; Rust's `enemies[0]` also bounds-checks, which the answer does
not need after a push. Against a correct C program the same holds: C must also index again after a
`realloc`. So zero, argued from the emitted code; nothing to time. Rule 3's compare-and-subtract runs
only after a removal while a name is live, next to the removal's own memory move. Rule 6's copies
are the copies Rust's workaround writes.

**(c) New mechanisms, from last night and from phase 2.**

- *Storage that never moves* (EnTT's paged pools; phase 2, `phase2-hylo-csharp.md` part 3): adding
  never moves an element, so an address stays valid. Measured there: random reads by position 1.30x,
  in-order sums 1.25x with 64 KiB pages and 1.03x with 1 MiB pages, appends 0.71x. Saved? The cost
  is in every random access to every paged list, and positions get the same safety for nothing, so
  it is discarded for lists. Good part kept: appending to pages is faster than doubling (0.71x), worth
  a separate `PagedArray` for append-heavy pools if the owner wants one.
- *Pin the list while a name lives* (the prove-or-check design): while pinned, adds go to overflow
  pages and removals leave tombstones. Its breaker found that `pop()` under a pin gives a value the
  tombstone still holds (a double free, `phase3-break-D3.md` 1.2) and that sort or insert under a pin
  moves the pin counters (1.8). Saved? Positions make the pin unnecessary for adding; for reordering,
  rule 4 refuses at compile time what the pin got wrong. Good part kept: "a name keeps the value of a
  removed element" (`phase3-design-D3.md` A2 row), now rule 3.
- *A shift version checked on every use* (the copy-on-write values design, D1-6: a counter that
  `remove` and `sort` bump, compared when a kept position is used): a run-time version of rules 3
  and 4, 4 more bytes per list header. Saved as compile-time rules 3 and 4 for changes the compiler
  sees, and as the walkers count for the rest.

### 2.4 Choice

(b). It meets all three goals: Python's program and meaning for adding and removing, Rust's safety
(bounds-checked positions; addresses only where proven), and Rust's workaround's cost. What it lacks:
reordering a list while a name into it is still needed is a compile-time error (Python would follow
the object), and popping an owning element that a name still holds is a compile-time error.

---

## 3. A3: two elements of one list in one call

### 3.1 What Rust's rule protects against

Phase 1 (A3): `collide(&mut bodies[i], &mut bodies[j])` is `E0499`; Rust's fix is
`bodies.get_disjoint_mut([i, j])`. With `i == j` the two references are one body. For plain numbers
that is not a memory hazard (writing a number frees nothing); it breaks two things: code that assumes
two bodies, and the optimiser's noalias promise. It becomes a memory hazard when the element owns
memory or holds an enum: a view into `a.name`'s characters is freed by `b.name = "x"`, and a
reference into `a.shape`'s `Circle` payload is reinterpreted when `b.shape = Square(...)`.

### 3.2 The Python program, and what Ember does today

| Python | Ember (`ember/a3_python.em`) |
|---|---|
| `def collide(a, b): a.vx, b.vx = b.vx, a.vx` | `fn collide(mut a: Body, mut b: Body):` then swap through `t` |
| `collide(bodies[i], bodies[j])` | `collide(bodies[i], bodies[j])` |

Python passes one body twice without complaint (`py_checks_output.txt`: "A3 same body twice: 1.0").
Ember today:

```text
error[E3022]: `*bodies[…]` is already mutably borrowed
   = help: use `bodies.get_pair_mut(i, j)` for two elements at once (`None` when they are the same), ...
   = note: computed indices may overlap; a structural split proves non-overlap (BRW-5)
```

The same error for two local names, `a = ref mut bodies[i]; b = ref mut bodies[j]`
(`ember/a3_locals.em`).

### 3.3 Answers

**(a) The compiler inserts the check `get_pair_mut` makes.** `collide(bodies[i], bodies[j])`
compiles to `if i == j: panic(...)`, then two non-aliasing addresses. Cost measured: 89.9 ms against
94.9 ms with no check and no `restrict` (`runs/a3.txt`, `c_check.c`, `c_alias.c`); safe Rust's
`get_disjoint_mut` 87.6 ms. The check costs nothing measurable; why the checked version came out 5%
faster was not investigated (UNVERIFIED; the spread in that file is under 2%). What it lacks: it
panics where Python works. Not chosen.

**(b) Two names may be one element; only a view into a replaceable part is checked (chosen).**

1. Two names into one list (`collide(bodies[i], bodies[j])`, or `a = bodies[i]; b = bodies[j]`) are
   allowed, `i == j` included, with Python's meaning: they are then one body.
2. Inside a function, two parameters (or names) of the same type are treated as possibly the same
   place, and so are a `T` and an element of an `Array[T]` (`fn update(mut self: Player, players:
   Array[Player])`). A **view into a part that a write can replace** (the characters of a `String` field, the
   elements of an `Array` field, the payload of an enum field) taken through one of them is
   invalidated by a write that may replace that part through the other. Using it afterwards is a
   compile-time error with fix-its:

   ```text
   error: `old` views a.name, which `b.name = "renamed"` may replace when a and b are the same Unit
     = help: copy it first: `old = a.name.copy()`, or take the view after the write
   ```

   (`ember/a3_heap_view.em` is that program; today's compiler accepts it only because today's
   `mut` parameters can never alias, `[BRW-11]`.)
3. Plain data written through both names needs nothing: no memory is freed by writing a number.

| Python | Ember under (b) |
|---|---|
| `collide(bodies[i], bodies[j])` | `collide(bodies[i], bodies[j])` |
| `a, b = bodies[i], bodies[j]; a.vx, b.vx = b.vx, a.vx` | `a = bodies[i]; b = bodies[j]; t = a.vx; a.vx = b.vx; b.vx = t` |

**Why it is memory-safe.** A write can free memory only when it replaces something that owns memory
or reinterprets an enum. Rule 2 forbids keeping a view into such a part across such a write through
any name that may be the same place; this is a compile-time check inside each function, because the
caller may pass the same element twice. Plain data cannot dangle. The list itself cannot move during
the call: a function that may change the list gets the element by copy (A2 rule 6). Attacks tried:
`i == j` with plain data (Python's result); a view into `a.name` across `b.name = ...` (rule 2); a
match on `a.shape` across `b.shape = Square(...)` (rule 2); `a.units.extend(b.units)` with `a` the
same as `b` (the loop inside `extend` sees `b.units` as possibly `a.units`, so it walks by position
and memory stays safe; Python's `extend` copies first to double the list, `py_checks_output.txt`
"extend self: [1, 2, 1, 2]", so the library writes `extend` with the length taken at the start).

**Cost.** Losing noalias, measured where it could matter:

| what (`bench/a3/`, `runs/a3.txt`, `runs/a3_span.txt`, `runs/a3_span_rust.txt`) | median ms | ratio |
|---|---|---|
| two body elements, `restrict` (what Rust's two `&mut` give LLVM) | 94.528 | 1.00 |
| two body elements, no `restrict` (the answer) | 94.864 | 1.00 |
| the same, function kept out of line, `restrict` / no `restrict` | 91.022 / 92.054 | 1.01 |
| two 16-float spans, length known at compile time: `restrict` | 88.206 | 1.00 |
| the same, no `restrict` | 135.342 | **1.53** |
| the same, answer saved: compare the two ranges at the call, call a `restrict` copy when they do not overlap | 94.276 | 1.07 |
| two spans, length unknown: `restrict` / no `restrict` / overlap check at the call | 90.270 / 101.715 / 96.714 | 1.00 / 1.13 / 1.07 |
| safe Rust, `split_at_mut` with an order test / `get_disjoint_mut` on two ranges | 191.236 / 192.938 | 2.10 / 2.12 (against 91.156) |

So for single elements, aliasing costs nothing measurable (the out-of-line `collide` has two more
instructions without `restrict`: 16 against 14, `c_alias_noinline.c` assembly). For two spans of one
list it can cost 1.53x, because without `restrict` LLVM unrolled the 16-step loop without vectors
(four `mulps`/`addps` pairs with `restrict`, sixteen scalar `mulss`/`addss` without, in the assembly
of `span_add.c`). Saved: when two span arguments of one element type cannot be proven apart, the
compiler emits one compare at the call (`|d - s| < n` as one unsigned compare) and calls a `restrict`
copy of the function when they do not overlap, the plain copy when they do (C's meaning, which is
also Python's for overlapping slices of one list). Measured 1.07x. My first version of that check
was two short-circuit compares and took 187 to 190 ms in unpinned single runs, because the first
compare's outcome depended on the random order of the two spans and mispredicted; one unsigned
compare fixed it. The safe Rust spellings took about 2.1x here; that their cost is the same kind of
misprediction is UNVERIFIED (not inspected).

**(c) Copy in, copy out (Swift's in-out).** Pass copies of both elements and write both back after
the call. With `i == j` the last write-back wins, which is neither Python's meaning nor an error.
Discarded for this; its good part is A2 rule 6 (passing an element to a function that may change its
list).

### 3.4 Choice

(b), with the call-site overlap check for spans. It meets the three goals: Python's program and
meaning, safety by a compile-time rule that targets exactly the memory hazard, and 1.00x (elements) to
1.07x (spans) of C with `restrict`, ahead of safe Rust's 2.1x on the span kernel. What it lacks:
keeping a view into one element's text or enum payload across a write that replaces it through a name
that may be the same element is a compile-time error (Python would keep the old string); and the
span check doubles the code of functions that take two spans of one element type.

---

## 4. A4: changing a list while looping over it

### 4.1 What Rust's rule protects against

Phase 1 (A4): a `push` in the loop body can reallocate the buffer the loop is walking (use after
free); a `remove` shifts the elements, so a loop that walks addresses reads past the end, into
elements already dropped. Rust refuses both (`E0502`) and the workaround is "collect the spawns in a
second list, then `retain` and `extend`" (phase 1 `a04_mutate_while_iterating_fix.rs`).

### 4.2 The Python program, Python's real behaviour, and what Ember does today

```python
for e in enemies:
    e.hp -= 1
    if e.hp <= 0:
        if e.splits:
            enemies.append(Enemy(5))
        enemies.remove(e)
```

Python runs it, but wrong: removing the current item makes the loop skip the next one. On
`[E0 (splits), E0, E3]` it visits E0, E3, E5 and leaves `[E0, E3, E5]`: the second dead enemy
survives. The same through a method (`self.kill(e)` removing from `self.enemies`) leaves
`[E0, E4]` with one kill missed. Appended items are visited ("A4 append visited: [1, 2, 3]"). All
from `py_checks_output.txt`. So the Python programmer expects the code to work, not Python's result.

Ember today, the closest spelling (`ember/a4_closest.em`, `iter_mut` to write, remove by position):

```text
error[E3020]: cannot mutate `*enemies` while it is borrowed by this loop
   = help: collect the indices or values first, then mutate; use `retain` or `drain` when the operation fits
   = note: a `for` loop borrows the iterable until the loop ends (CTL-2)
```

Through a method (`ember/a4_method_closest.em`): `E3020 cannot mutate *self.enemies while it is
borrowed by this loop` at `self.kill(i)`. The pure Python spelling (`ember/a4_python.em`) fails earlier:
`E3021 cannot write through a shared reference` at `e.hp -= 1` (a loop element is read-only unless
the loop says `iter_mut()`), and `E2020` at `enemies.remove(e)` (Ember's `remove` takes a position).

### 4.3 Answers

**(b) The loop is compiled into a one-pass in-place filter (chosen).**

| Python | Ember under (b) |
|---|---|
| `for e in enemies:` | `for e in enemies:` |
| `    e.hp -= 1` | `    e.hp -= 1` |
| `    if e.hp <= 0:` | `    if e.hp <= 0:` |
| `        enemies.append(Enemy(5))` | `        enemies.push(Enemy(hp=5, splits=false))` |
| `        enemies.remove(e)` | `        enemies.remove(e)` |

The rules:

1. A `for` loop whose body (or a function it calls that the compiler can see) may change the list it
   walks is compiled as a loop over positions with a read position `r` and a write position `w`,
   running to the live length. Elements appended during the loop are visited, as in Python.
2. `e` names the element at `r` (A2's rule). Writes through `e` are allowed with no `iter_mut()`.
3. `enemies.remove(e)` with `e` the loop's own element removes the current element: it is simply not
   stored at `w`. The next element is not skipped (Python's bug is not copied). `e` stays usable to
   the end of the iteration (Python's meaning). The list's logical length drops at once.
4. Removing another element (`del enemies[k]`) inside the body is allowed: positions before and after
   the gap are mapped (`k < w ? k : k + (r - w)`, a compare and an add) and it costs what Python's
   `remove` costs, a memory move. `len(enemies)` in the body is `w + (len - r)`. These mappings are
   emitted only in loops whose bodies use them.
5. Inserting, sorting or reversing the walked list inside its own loop is a compile-time error with
   the fix-it "do it after the loop" (Python's own behaviour there is to revisit or skip elements). A
   loop nested inside another loop over the same list may append to it but not remove from it:
   compile-time error with the fix-it "mark it and remove it in the outer loop".
6. `break` or `return` after removals closes the gap: one memory move of the rest of the list.
7. Removed elements are dropped when the loop ends (in one place, for every mode below).
8. **Through a method the compiler can see** (`self.kill(e)`, where `kill` does
   `self.enemies.remove(e)` on the same `self`): the compiler compiles a copy of `kill` for this call
   in which that removal becomes "the current element is not kept"; the rest of `kill` runs as written.
   `ember/a4_method_emitted.em` is that program written by hand, and today's compiler accepts it.
9. **Through code the compiler cannot see** (a stored callback, a virtual method, a function value,
   a foreign call, or a list parameter that may be the walked list): the walked list carries a
   **walkers count** in the top byte of its capacity word. The loop adds one at its start and takes one
   away at its end; while it is above zero, `remove`, `pop`, `insert`, `sort`, `reverse`, `swap`,
   `clear`, `truncate`, replacing the list and dropping it panic: "enemies changed while a loop over it was running (line
   12)". Adding stays allowed. A loop sets the count only when its body contains such a call; a frozen
   (read-only, shared) list never needs it.
10. Memory safety must not depend on the count, so after such a call the loop checks `r < len` and
    `w <= r` again (two compares). And when the elements own memory and such a call is in a loop that
    removes, the gap between `w` and `r` is filled with the removed elements themselves (swapped, never
    copied), so code that reads the list in the middle of the loop never meets a stale copy of an
    element that owns memory.

**Why it is memory-safe.** The loop holds positions, never an address across a change; `r < len` is
its own test and `w <= r` holds by construction, so the emitted indexing needs no further check
(rule 10 re-checks after unseen calls). An element moved from `r` to `w` leaves a stale bit copy at
`r` that nothing reads (the body reads only through the mappings of rule 4) and that is never dropped
(the list ends at `w`); rule 10 removes even the stale copy where unseen code could read it. No
unwinding (`[PAN-1]`) means no caller ever sees a half-filtered list after a panic. Attacks tried:
push that reallocates in the middle of the gap (the bits move with the buffer; nothing is dropped
twice); removing the current element twice (the second is a panic, "removed twice"; Python raises
`ValueError` or removes another equal item); removing another element ahead of and behind the cursor
(rule 4); a nested loop removing (rule 5); a callback that replaces `self.enemies = []` during the
loop (the drop of a walked list panics: last night's checked-pointers breaker found that this read
freed memory in that design, `phase3-break-D2.md` H4); a callback that sorts the list (panic); a list
parameter that is the walked list under another name (rule 9).

**Cost** (`runs/a4.txt`, `runs/a4_swapfill.txt`; workload in `bench/a4/common.h`: 1,000,000 enemies,
100 frames, about 1% die each frame and are replaced by one appended enemy; checksum 155569252 for
every program):

| program | what it is | median ms | ratio to plain |
|---|---|---|---|
| `plain.c` | plain C, fastest found: in-place filter, element kept in registers | 93.920 | 1.00 |
| `ember.c` | **the answer**: what the compiler emits for the loop above | 92.456 | **0.98** |
| `ember_swapfill.c` | the answer in rule 10's mode (owning elements, unseen calls) | 105.981 (against plain 91.135 in that file) | 1.16 |
| `rust_index.rs` | safe Rust, the same in-place filter with positions | 125.906 | 1.34 |
| `rust_idiom.rs` | safe Rust, the idiom: second list, `retain`, `extend` | 151.943 | 1.62 |
| `deferred.c` | removals marked, applied in a second pass at the loop's end | 143.756 | 1.53 |
| `plain_naive.c` / `ember_naive.c` | update in place, then copy down (section 1.4) | 444.559 / 446.476 | 4.73 / 4.75 |
| `rust_retain_mut.rs` | safe Rust's one-pass `retain_mut`, spawns in a second list | 439.934 | 4.68 |
| `rust_index_naive.rs` | safe Rust, positions, update in place then copy down | 468.238 | 4.99 |
| `plain_fieldcopy.c` | the naive form with a field-by-field copy (the mechanism check) | 135.060 | 1.44 |

Peak memory is the same for all (20,880 to 21,252 KB). Why `rust_index.rs` is 1.34x was not
investigated (UNVERIFIED; it has bounds checks on `v[w]` that the C model does not need). The walkers
count's only cost on a hot path is that `push` must mask the count off before comparing the length
with the capacity: 400,000,000 pushes took 199.756 ms without the mask and 202.463 ms with it
(`bench/count/push.c`, `runs/count.txt`, +1.4%, inside the spread of 196.5 to 203.3 ms). The count
costs no memory (it lives in the capacity word's top byte, so a list can hold at most 2^56 elements).

**(a) Rust's guarantee, written for you: the compiler applies the changes after the loop.** Phase 1's
A4(a): `push` and `remove` inside the loop are queued and applied when it ends. That is
`deferred.c` (1.53x) and the Rust idiom (1.62x): the second pass over the list is the cost, and the
meaning differs from Python's (a removed enemy is still in the list, and `len` unchanged, until the
loop ends). Saved? Doing it in one pass is answer (b). Discarded as a separate answer.

**(c) Last night's mechanisms, and the walkers count.**

- *Pins, tombstones and a settle at the end* (the prove-or-check design): holes found by its
  breaker (pop under a pin, a settle re-entered by `drop` code, sort under a pin;
  `phase3-break-D3.md` 1.2, 1.7, 1.8) and 1.63x to 1.90x in the ECS (section 6). Saved: positions
  remove the need to pin for adding; rule 5 refuses the reorderings; the one-pass filter replaces the
  settle. Good part kept: a loop visits each element once and the list closes up at the loop's end.
- *An iteration count tested by `remove`, tombstones, compaction at the end* (the checked-pointers
  design, L1 to L5): its breaker found a loop reading a freed buffer when the body replaced the list
  (H4) and an address held across a call that reallocates (H5). Saved: the walkers count is that
  iteration count, made to panic on every operation that reorders, removes, replaces or drops (H4's
  fix), with positions instead of addresses (H5's fix). Good part kept: the count itself.
- *The loop holds a path and panics after a shifting change* (the copy-on-write values design, D1-6
  and its A4 row): adding during the loop is seen, and a `remove` in a plain loop makes the next step
  panic. Python runs that program, so the panic fails goal 3. Saved: the one-pass filter gives the
  removal a meaning instead of a panic. Good part kept: the loop holds a path (a position), never an
  address, which is answer (b)'s base.
- *Iterate a copy-on-write snapshot* (Swift's loops; phase 2, `phase2-swift-mojo.md` idea 1 and
  `phase2-rc-elision.md` "Found in passing"): writes through `e` would go to the snapshot and be
  lost. Discarded for loops that write their elements.

### 4.4 Choice

(b), with rule 9's walkers count for changes made by code the compiler cannot see. On the workload it
is 0.98x of the fastest plain C and faster than safe Rust's idiom (1.62x); safety rests on positions
and bounds, not on the count. What it lacks: inserting, sorting or reversing a list inside its own
loop, and removing from it in a nested loop over the same list, are compile-time errors; a reordering
or removal made by unseen code during the loop is a run-time panic (deterministic, as Python's
`dictionary changed size during iteration`), where Rust refuses at compile time. That last one is a
check moved to run time; it is chosen over a compile-time refusal because the refusal would hit every
loop that calls a stored callback, most of which never touch the list.

---

## 5. A25: a local list of string views

### 5.1 What Rust's rule protects against

A view (`str`: address and length) into a string must not outlive the string's buffer; if the
string is freed or grows into a new buffer while the view lives, reading the view reads freed memory.
Rust accepts a local list of views of a local string (phase 1 `a06d_local_vec_of_views.rs`) and ties
it to the string's lifetime. It refuses the Python one-liner, because the lowered string is a
temporary freed at the end of the statement, and refuses returning views of a local
(`target/redesign2/containers/rust/rustc_output.txt`):

```text
a25_oneliner.rs  error[E0716]: temporary value dropped while borrowed
                 help: consider using a `let` binding to create a longer lived value
a25_return.rs    error[E0515]: cannot return value referencing local variable `lower`
a25_param.rs     (views of the parameter, returned) accepted
```

### 5.2 The Python program, and what Ember does today

| Python | Ember (`ember/a25_python.em`) |
|---|---|
| `words = line.lower().split()` | `words: Array[str] = line.to_lower().split_whitespace().collect()` |
| `return len(words)` | `return words.len()` |
| `def words_of(line): return line.lower().split()` | `fn words_of(line: str) -> Array[str]:` ... `return words` (`ember/a25_return.em`) |

Ember today refuses even the local list that Rust accepts (`a25_python.em`, `a25_named.em`,
`a25_return.em`):

```text
error[E3063]: `str` is a view, so it may not be passed to `collect`, which stores it where only a `static` view may go
  = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; ...
  = note: the view borrows `lower`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)
```

A function that collects views of its own parameter compiles (`a25_param.em`, `a25_local_param.em`),
but by `[TYP-15]` it passes the requirement to its callers: a caller must give it a `static` string,
so calling it with a string built at run time is refused at the call (`a25_param_misuse.em`:
`E3063 ... may not be passed to words_of, which stores it where only a static view may go`).

### 5.3 Answers

**(a) A local list carries the lifetime of what it holds (chosen for local use).** What Rust's
`Vec<&'a str>` does, with nothing written: the compiler infers that `words` borrows `lower`, as it
already does for a struct holding views (`[LT-14]`). With the temporary rule that another group owns
(A22: a temporary lives as long as the views made from it, `phase1-borrowing.md` A22(a)), the Python
one-liner compiles, which Rust refuses. Two refinements, both compile time: reassigning the text
while the words are still used keeps the old buffer alive until the words' last use (Python's
meaning: the words are separate from the name); changing the text in place while the words are used
is compiled the same way (a new buffer for the text, the old one kept for the words), which is
Python's meaning because Python strings never change in place, at the price of a copy where Rust
would refuse.

**(b) A returned list takes its text along (chosen for returning).** When a list of views escapes (is
returned or stored in a field) and every view points into one local string that is not used again,
the compiler moves that string into a hidden owner field of the returned list. The views point into
the string's heap buffer, which does not move when the list moves, and nobody can change it, because
the list owns it privately. Dropping the list frees the buffer.

| Python | Ember under (a) and (b) |
|---|---|
| `def words_of(line):` | `fn words_of(line: str) -> Array[str]:` |
| `    return line.lower().split()` | `    return line.to_lower().split_whitespace().collect()` |
| `ws = words_of("A b C")` | `ws = words_of("A b C")` |

This is the self-referential pair (a text and views into it) that Rust needs a crate for:
`ouroboros` and `self_cell` have 89,396,135 and 81,296,771 downloads (crates.io API, fetched by phase 1,
`phase1-borrowing.md` A7). Cost: nothing per word. A list whose elements are views (`Array[str]`,
`Array[Span[T]]`, lists of structs holding views) gets a 24-byte owner field in its header, so that
caller and callee agree on one layout; it is empty unless the list took its text along. Lists of
anything else are unchanged. What the compiler emits for `return words` is the list's header plus the
lowered string's header moved into that field, so the cost is argued from the emitted code; nothing
to time.

**Why (a) and (b) are memory-safe.** A view is used only while the buffer it points into is alive and
unchanged: in (a) the compiler proves it from lifetimes (Rust's rule, inferred), keeping a replaced
buffer alive instead of refusing; in (b) the buffer is owned by the same value as the views and is
never changed or freed before them. Attacks tried: the text reassigned while the words live (old
buffer kept); the text grown in place (copied first); the returned list copied or moved (the owner
moves with it; a copy depends on what another group decides `b = a` means for lists, section 7); a
view of another, shorter-lived string pushed into a returned list (refused by lifetime, fix-it: an
owned copy); one word taken out of a returned list and returned again as `str` (refused, fix-it:
return `String`, one allocation, as Python allocates a new `str`); the list sent to another thread
(moves with its owner, which nothing else reaches).

**Cost** (`runs/a25.txt`; workload `bench/a25/common.h`: 32 MiB of words, lowered, split, every word
hashed, 5 passes; checksum 12142372104838704574 for all; text generation alone 46.010 ms):

| program | words are | median ms | ratio | peak KB | ratio |
|---|---|---|---|---|---|
| `views.c` | views (address, length): the answer, and Rust's `Vec<&str>` | 713.429 | 1.00 | 201,128 | 1.00 |
| `rust_views.rs` | safe Rust `Vec<&str>` | 746.534 | 1.05 | 201,492 | 1.00 |
| `shared_count.c` | views that share a counted buffer (the copy-on-write values design) | 723.672 | 1.01 | 201,112 | 1.00 |
| `owned_sso.c` | owned strings, short-string layout (up to 23 bytes inline) | 1041.040 | 1.46 | 332,200 | 1.65 |
| `owned_malloc.c` | owned strings, one allocation each (Ember's fix-it today) | 1667.946 | 2.34 | 354,608 | 1.76 |
| `rust_owned.rs` | safe Rust `Vec<String>` | 1751.482 | 2.45 | 355,004 | 1.77 |

**(c) Other mechanisms.**

- *Words share a counted buffer* (the copy-on-write values design, and phase 1 A25(b)): 1.01x here,
  same memory, and a word can go anywhere. Saved? It is cheap in this workload (one counter, one
  cache line, one thread). It loses to (b) because the count must become atomic once words of one
  buffer are in two threads (phase 2 measured an atomic count pair at 5.8 ns against 0.84 ns,
  `FINDINGS.md` section 2), one live word keeps a whole text alive, and every string gets a count
  field. Good part: if the owner later wants strings that can be shared freely between objects (A9),
  this is the measured cheap form.
- *Owned short strings*: 1.46x time and 1.65x memory here, against 2.34x and 1.76x for one allocation
  per word. Good part: the cheapest way to make words owned when a program asks for owned copies (the
  fix-it of (b)'s residue); Ember's `String` has no such layout today (`ember_vec`).

### 5.4 Choice

(a) for local lists and (b) for returned or stored ones. They meet the three goals: Python's
one-liner and its return, Rust's guarantee (views never outlive their buffer, proven when compiled),
and 1.00x of C views. What they lack: a list holding views of two texts with different lifetimes
cannot escape both lifetimes without copies; a single word returned from a function that made its text
must be returned as `String` (one allocation).

---

## 6. The ECS: creating and destroying entities inside a view loop

### 6.1 What Rust's rule protects against

In the EnTT layout (one sparse set per component: a dense array of components, the entities in the
same order, and a sparse array from entity id to position, `tasks/95/ecs-entt-design.md`), destroying
an entity swaps the last component into its slot and shortens the array, and creating one appends,
which may reallocate. A loop that holds the addresses of `pos` and `vel` across either reads freed
memory or a swapped-in entity; a loop that walks forward visits the swapped-in entity twice. Rust
refuses the loop; Rust ECS libraries queue the changes (Bevy's `Commands`), which today's `std.ecs`
also has (`std/src/ecs.em`, `Commands`). EnTT allows destroying the current entity because it walks
backwards (`tasks/95/ecs-entt-design.md` item 3).

### 6.2 What a programmer writes, and what Ember does today

| Python (plain dicts, `py_checks.py`) | Ember (W1 of `phase3-design-D3.md`, unchanged) |
|---|---|
| `for e in list(vel):` | `for e, pos, vel in reg.view[(Position, Velocity)]():` |
| `    p, v = pos[e], vel[e]; p[0] += v[0]; p[1] += v[1]` | `    pos.x += vel.dx` and `pos.y += vel.dy` |
| `    if e % 100 == s:` | `    if e.id() % 100 == s:` |
| `        del pos[e]; del vel[e]` | `        reg.destroy(e)` |
| `        pos[n] = [0.0, 0.0]; vel[n] = [1.0, 1.0]; n += 1` | `        n = reg.create(); reg.emplace(n, Position(x=0.0, y=0.0)); reg.emplace(n, Velocity(dx=1.0, dy=1.0))` |

Python refuses to change a dict while walking it ("ECS dict walk: dictionary keys changed during
iteration", `py_checks_output.txt`), so the Python habit is to walk a copy of the keys, `list(vel)`.

Ember today: the view loop written with names (`ember/ecs_for_loop.em`) is
`E3020 cannot mutate *self.vel.ents while it is borrowed by this loop`; the same step written with
positions and no names held across the changes (`ember/ecs_index_loop.em`) is accepted.

### 6.3 Answers

**(b) Positions, names keyed by the entity, and a cursor (chosen).**

1. The view loop is library code (`std.ecs`). It walks the smaller pool backwards by position, as
   EnTT does: destroying the current entity moves an already visited entity into its slot, and
   created entities are appended past the loop, so they are not visited this step.
2. `pos` and `vel` are names keyed by the entity: "the Position of `e`". The compiler keeps their
   addresses while nothing can change the pools (the common path) and looks them up again through
   `sparse` after a call that may (A2's rule, with the entity as the key instead of a position). After
   `reg.destroy(e)` the lookup finds nothing and panics, or, when the use is in the same body, it is a
   compile-time error ("`pos` used after `reg.destroy(e)` on line 7"). A key keeps naming the same
   entity through any swap, so no index adjustment is needed.
3. The loop also holds the pools' buffer pointers and lengths in locals on the common path and reads
   them again after the rare branch: the fix of section 1.4.
4. Before a call that may destroy, the loop stores its position in the pool's `cursor` and reads it
   back after. `destroy` compares the removed position with the cursor: an entity the loop has not
   reached yet is replaced by the last unreached one, which is then replaced by the last visited one
   (two moves instead of one, and the cursor moves down by one). Every entity that exists when the
   loop starts and is not destroyed before its turn is visited exactly once.
5. A loop saves the previous cursor and restores it at its end, so loops nest.

**Why it is memory-safe.** Every array access is either the loop's own position on the common path,
which only decreases from a length checked at the start or after the rare branch, or a lookup guarded
by the view's own tests (an id below the sparse length, a position below the dense length), or a
bounds-checked index inside `destroy`/`emplace`. The cursor only decides which entity is visited; if
the cursor logic were wrong, an entity would be visited twice or skipped, never memory misread.
Attacks tried: destroying the current entity (EnTT's case); destroying entities already visited and
not yet reached, and creating entities, in random order (the self-check below); using `pos` after
`destroy(e)` (rule 2); `emplace` into the walked pool reallocating it (rule 3 reads it again);
`reg.clear()` inside the loop (by design `clear` sets the cursor to 0, so the loop ends; not in the
models); `reg.sort[T]()` inside a loop over that pool (by design a sort while a loop walks the pool
panics, using a walkers count beside the cursor as in section 4; not in the models); a listener on
`on_destroy` destroying more entities (rule 4 handles any order).

Self-check of rule 4: `bench/ecs/cursor_check.c` runs 200 random loops that destroy the current
entity, destroy other entities before and after the cursor, and create entities, and checks that every
entity is visited exactly once, none destroyed before its turn is visited, and none created is
visited. It prints `cursor-check ok`; with the rule removed (`if (i < p->cursor)` replaced by
`if (0)`), it fails at its exactly-once assertion (line 57). Built with clang -O2 and run once.

**Cost.** W1 from phase 3 (1,000,000 entities, half with a Velocity, 100 steps, 1% destroyed and
recreated each step; checksum 2111831520 for every program). Four pinned runs, medians in ms:

| program (`bench/ecs/`) | what it is | `ecs_1` | `ecs_2` | `ecs_3` | `ecs_4` |
|---|---|---|---|---|---|
| `plain_sparse.c` | plain C, EnTT layout, no checks (phase 3's `w1_sparse.c`) | 111.898 | 92.257 | 94.468 | 94.184 |
| `ember_direct.c` | **the answer**, same program structure | 115.472 (1.03x) | 103.299 (1.12x) | 101.398 (1.07x) | 99.189 (1.05x) |
| `plain_sparse_cached.c` | plain C holding the arrays in locals by hand | 110.586 (0.99x) | 99.255 (1.08x) | 95.319 (1.01x) | |
| `rust_sparse.rs` | safe Rust, same structure, positions | 117.584 (1.05x) | 104.699 (1.13x) | 108.346 (1.15x) | 108.150 (1.15x) |
| `plain_registry.c` | plain C with the library structure of the models below (pools in a table, destroy through a method table) | | | | 102.360 (1.09x) |
| `ember_registry.c` | the answer, library structure | | 110.244 (1.19x) | 105.297 (1.11x) | 104.350 (1.11x; 1.02x of `plain_registry`) |
| `lastnight_D3.c` | the prove-or-check design's model (pins, tombstones), library structure | 182.095 (1.63x) | 175.570 (1.90x) | 177.825 (1.88x) | 175.123 (1.86x) |
| `lastnight_D1.c` | the copy-on-write values design's model | 157.462 (1.41x) | | | |
| `lastnight_D2.c` | the checked-pointers design's model | 261.045 (2.33x) | | | |
| `ember_direct_naive.c` | the answer's first version (section 1.4) | 150.207 (1.34x) | 128.388 (1.39x) | | |
| `plain_archetype.c` | plain C, archetype layout (phase 3's `w1.c`) | 44.253 (0.40x) | | | |

(`ecs_1`'s row for the library-structure answer is left out: that run used its first version, now
`ember_registry_naive.c`, which took 137.326 ms in `ecs_2`.) Peak memory: the answer 39,164 to
39,204 KB against plain C's 39,164 to 39,208 KB.

The spread inside one run is wide (in `ecs_2` the answer ran between 92.5 and 107.6 ms, plain C
between 87.3 and 94.4), so the answer's 1.03x to 1.12x is within what this machine can tell apart
today; the final measurer decides. The emitted common path is the same instructions as plain C's:
in the assembly of `ember_direct.c` and `plain_sparse.c` (clang -O2 -S) the inner loop is, in both,
one load of the entity id, a compare, one load from `sparse`, a compare, two float load-add-store
pairs, and the `% 100` test; the differences are on the rare path (the cursor, the calls to `add`,
which the plain version inlines). Against the prove-or-check design's model, which has the same
library structure, the answer is about 1.7 times faster (105.297 against 177.825 ms in `ecs_3`,
104.350 against 175.123 in `ecs_4`).

Plain C in the EnTT layout is itself 2.1x to 2.5x the archetype layout's 44.253 ms (94.184 to
111.898 ms over the four runs): that gap is the layout, not the memory model, as
`phase3-measurements.md` found, and the choice belongs to the ECS design (the owner chose EnTT).

**(a) Queue the changes and apply them after the loop** (Bevy's `Commands`; today's `std.ecs`): what
Rust programmers write. Not measured here (UNMEASURED); its list analogue, `deferred.c`, took 1.53x in
section 4. Its meaning also differs: a destroyed entity stays visible to the rest of the loop. Not
chosen.

**(c) Last night's mechanisms.** Pins and tombstones: 1.63x to 1.90x above. Checked pointers with
stable stores: 2.33x. Pages that never move (EnTT's own reason for paging): needed only when C++ code
keeps addresses; with names that are keys, nothing keeps an address across a change, and pages cost
1.30x on random access (section 2.3). Good part kept from the checked-pointers design: "the loop
publishes its cursor before any call that may insert" (`phase3-design-D2.md` L2), which became rule 4,
with its breaker's note that nested loops need the outer cursor saved (`phase3-break-D2.md` N7),
which became rule 5.

### 6.4 Choice

(b). On W1 it is 1.03x to 1.12x of plain C in four provisional runs, at the same memory, against
safe Rust's 1.05x to 1.15x. What it lacks: in loops nested over the same pool, an inner loop that
destroys entities other than its own current one keeps "every entity once" for itself but not for the
outer loop (one cursor per pool; memory safety is unaffected; EnTT itself allows only the current
entity to be removed). A small stack of cursors per pool would fix it; not built.

---

## 7. How the answers fit together, and what they need from the other groups

**One rule set for every container.** A name for an element is its position (lists) or its key
(maps, ECS pools); an address is kept only across code the compiler proves cannot change that
container; after a change the name is looked up again; a loop that changes its own list is a one-pass
filter; changes the compiler cannot see are covered by the walkers count, which only keeps meaning,
never safety. A list of views carries its text's lifetime, or its text. Two names may be one element.
The ECS needs no language mechanism of its own: it is library code written with these rules
(`ember/ecs_index_loop.em` is accepted today).

**What these answers need from the other groups:**

1. **Returning places (the references group, A5).** `reg.get[Position](e)` and the view's yielded
   `pos` must reach the caller as "the Position of `e`" (a place the caller can look up again), not as
   a bare address: a function that returns a place, or a yield-once accessor (Hylo's, recommended in
   `phase2-hylo-csharp.md` part 4), with the key visible to the compiler.
2. **Temporaries (A22).** The temporary `line.to_lower()` must live as long as the views made from it
   (phase 1 A22(a)), or the A25 one-liner fails as it does in Rust.
3. **What `b = a` means for lists (the objects group, A11).** These rules work whether lists are values
   that move, values that copy, or shared objects. If lists become shared objects (Python's meaning),
   more changes reach a list under another name, so more loops need the walkers count, and a returned
   list of views (A25 (b)) is then one object with its owner inside. If lists copy, a copy of a list of
   views with an owner copies the owner or points into the original's.
4. **Effect summaries (whoever owns inference, A1).** Each function's summary must say which lists
   (by type and path from its parameters and globals) it may add to, remove from, reorder, replace or
   drop, and whether it removes the element it was given (for A4 rule 8). Ember computes field sets
   for private methods today (`[BRW-10]`); they must cover `pub` methods (written into the interface
   file) and these four kinds of change. Calls through virtual methods, function values and foreign
   functions are "may do anything their types allow", which is what selects the walkers count.
5. **Objects (A8 to A10).** For lists inside class objects: whether removing an object from a list
   destroys it (then A4's "removed elements are dropped when the loop ends" applies to objects too)
   and whether the containing object can be destroyed while one of its lists is walked (the walkers
   count then panics at the list's drop, which is safe; the objects group may prefer to delay the
   destruction instead, as the checked-pointers design's O5 does for running methods).
6. **Closures (A12).** A stored closure must not capture an element name (a position) of a list that
   outlives the closure's knowledge of it; the simplest rule is that capturing a name captures a copy
   of the element (plain data) or the container and the position. Last night's prove-or-check
   breaker showed both readings surprise (`phase3-break-D3.md` 2.5); that choice belongs to the
   closures group.
7. **Threads (A15).** Positions, the walkers count and the cursor are single-thread facts. They rely on
   the rule that a list is moved, frozen or locked when another thread can reach it; a frozen list
   never needs a walkers count.

---

## 8. Good parts of what was discarded

| Good part | From | Where it may help |
|---|---|---|
| Appending to pages is faster than doubling (0.71x, same peak memory) | phase 2 `phase2-hylo-csharp.md` part 3; EnTT | a `PagedArray` for append-heavy data, if wanted |
| A removed element stays usable through a name that held it | the prove-or-check design (A2 row) | A2 rule 3 |
| A shift version on the list | the copy-on-write values design (D1-6) | became A2 rules 3 and 4 at compile time |
| An iteration count on the list tested by changes | the checked-pointers design (L4), with its breaker's H4 fix | became the walkers count |
| The loop publishes its cursor before calls that may insert; nested loops save the outer cursor | the checked-pointers design (L2) and its breaker (N7) | ECS rules 4 and 5 |
| Words that share a counted buffer: 1.01x of views in A25's workload | the copy-on-write values design; phase 1 A25(b) | strings shared freely between objects, if the objects group wants that |
| The short-string layout: owned words at 1.46x instead of 2.34x | (this report, `owned_sso.c`) | the owned copies that remain as fix-its |
| An inserted `i != j` check costs nothing measurable | phase 1 A3(a); Rust's `get_disjoint_mut` | where a library wants Rust's "two distinct elements" guarantee |
| Copy in, copy out | Swift's in-out (phase 2) | A2 rule 6 |
| Removals applied at the loop's end | phase 1 A4(a); both last-night designs | A4 rule 7 (dropping removed elements at the loop's end) |

---

## 9. Files

- Ember probes, `target/redesign2/containers/ember/`: `a2_python.em`, `a2_ref.em`, `a2_emitted.em`,
  `a2_take.em`, `a3_python.em`, `a3_locals.em`, `a3_heap_view.em`, `a4_python.em`, `a4_closest.em`,
  `a4_emitted.em`, `a4_method.em`, `a4_method_closest.em`, `a4_method_emitted.em`, `a25_python.em`,
  `a25_named.em`, `a25_return.em`, `a25_param.em`, `a25_local_param.em`, `a25_param_misuse.em`,
  `a25_param_misuse2.em`, `ecs_for_loop.em`, `ecs_index_loop.em`; output of
  `target/ci/ember.exe check <file>` for each in `ember_check_output.txt`.
- C and Rust, `target/redesign2/containers/bench/`: `a4/` (plain, the answer, the swap-fill mode,
  deferred, three Rust versions, and the naive variants of section 1.4), `ecs/` (plain, the answer in
  two structures, Rust, copies of last night's three models and of phase 3's two plain baselines,
  `cursor_check.c`), `a3/` (`common.h` variants, `span_add.c` built six ways, three Rust versions),
  `a25/` (four C representations, two Rust versions, `gen_only.c`), `count/push.c`.
- Timing: `bench/run.ps1`, `bench/summarize.sh`, results `bench/runs/a4.txt`, `a4_swapfill.txt`,
  `ecs_1.txt` to `ecs_4.txt`, `a3.txt`, `a3_span.txt`, `a3_span_rust.txt`, `a25.txt`, `count.txt`.
- Python: `target/redesign2/containers/py_checks.py`, `py_checks_output.txt`
  (`C:/Python314/python.exe -I py_checks.py`, Python 3.14.2).
- Rust checks: `target/redesign2/containers/rust/a25_oneliner.rs`, `a25_return.rs`, `a25_param.rs`,
  `rustc_output.txt` (`rustc --edition 2021 --crate-type lib --emit=metadata <file>`).
