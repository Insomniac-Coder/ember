# Phase 3, design D1: values and projections

Written 2026-10-24 by the phase-3 D1 design agent, for the memory-model rework (owner's directive of
2026-10-23: C-like speed and memory, memory safety without Rust's annoying parts, Python-like
syntax; and the owner's guidance: remove an annoyance by (a) a more Pythonic spelling of the same
guarantee, or (b) a smarter mechanism that makes the hazard impossible or harmless).

What was read: every phase 1 and phase 2 report in this folder; the spec's Part VII (ownership,
borrows, regions), Part VIII (classes and counting), IX.0 to IX.7, VI.2 and VI.5 (places, closures),
XI.1 to XI.3 (threads); `tasks/95/ecs-entt-design.md`, `tasks/95/odr-pending.md` (ODR-286), and the
probes in `target/analysis/ecs4` and `ecs5`.

What was made and run:

- The four C models `target/redesign/bench/D1/w1.c` .. `w4.c`, built once each with
  `"C:/Program Files/LLVM/bin/clang.exe" -O2 wN.c -o wN.exe` (clang 22.1.8, no warnings) and run once
  each. Their checksums were compared with four plain reference programs I wrote first, with a
  different data layout (arrays indexed by id or node number), in the session scratchpad
  (`C:/Users/ism19/AppData/Local/Temp/claude/C--Users-ism19-Code/ae3a2898-3f0f-4361-99e2-9bbbb0976123/scratchpad/ref/ref1.c`
  .. `ref4.c`, scratch files, not project files). All four agree (section 5).
- One Python 3.14.2 check script (`.../scratchpad/py_d1.py`, run with `python -I`) for the four
  Python behaviours this report leans on (section 6.1).
- Nothing was timed. The harness prints a time in its line; the check runs were single, unpinned
  runs, so those times are not measurements and are not used anywhere in this report.
- No Ember program was run; `ember check` was not needed, because D1 is a design that today's
  checker does not implement. No D1 program below was compiled by anything. No WebSearch or
  WebFetch was needed: every outside fact used comes from the phase 1 and 2 reports, which cite it.

Every number below names its source. A number marked **UNVERIFIED expectation** is my prediction
for the measurer to check, with the arithmetic shown.

---

## 0. The short version

D1 gives a programmer three things, and no references at all:

1. **Values**: numbers, `struct`, `enum`, tuples, fixed arrays, `Array`, `Map`, `Set`, `String`,
   closures and `dyn I` values. `b = a` gives `b` its own value. It is cheap: a collection is a
   pointer to a counted buffer, and the buffer is copied only when one of the two is changed while
   the other still exists (copy on write). There are no moves to get wrong, no `clone()`, no `Box`.
2. **Objects**: `class` instances, shared through counted handles exactly as Python names share
   objects. A `weak` field points at an object without keeping it alive and reads as `None` once
   the object is gone.
3. **Projections**: ways to work on a part of a value where it lives, for a while: a `mut`
   parameter, `for x in place:`, `with x = place:`, `match place:`, and functions that `yield` a
   place instead of returning a copy. A projection has no type, so it can never be stored, returned
   or captured by an escaping closure. That single fact is what removes lifetimes.

The safety of projections comes from one run-time rule instead of Rust's borrow rules: **the
compiler keeps an address only across code that cannot change what it points into; after code
that can, it fetches the address again from the projection's path (with a bounds check).** Where it
cannot fetch it again (the address was handed to a function), the list it points into is *pinned*
for the call, and changing that list's size panics.

What is left of Rust at compile time is two errors, both at one statement:

- **E-EXCL**: one call (or one `with` header) passes a place to be changed and also passes an
  overlapping place, where the overlap owns heap memory (`f(mut a, mut a)`, or `players[i].update(players)`).
- **E-COPYLOST**: a local was copied from a place and then changed in part, and the change can never
  be seen (`boss = enemies[0]; boss.hp //= 2`, with `Enemy` a struct).

Verdicts over the 29 ids (A1 to A25, A31 to A34; section 2): **removed 21** (16 by (b), 4 by (a),
1 by both; ten of them keep a residue the table names: A1, A2, A3, A6, A9, A12, A13, A14, A19,
A32),
**moved to run time 2** (A4 for removal during a plain loop, A8), **kept 5** (A15 for objects,
A16, A18 mostly, A20, A31 for structs), **absent before and after 1** (A34). The four workloads are written
as a Python programmer would write them and compile under D1 with no annotation beyond `mut`
parameters, `weak` on two fields, and the word `class`; their C models produce the reference
checksums.

---

## 1. The model in plain words

### 1.1 The rules

**D1-1 Values copy, cheaply.** A struct, enum, tuple, fixed array, collection, closure or `dyn I`
value is a value. Assigning it, passing it to a parameter that keeps it, storing it and returning
it give an independent value. A collection is one pointer to a heap buffer that carries a count;
copying it adds 1 to the count; the first change made through either copy while the count is above
1 copies the buffer first. At a value's last use the compiler moves it instead (no count change),
so most copies cost nothing. There is no `Copy`/`Clone` split, no `owned` mode, no use-after-move
error.

```ember
items = [1, 2]
snapshot = items          # the buffer is shared, count 2
items.push(3)             # items copies the buffer first; snapshot is still [1, 2]
send(items)               # last use of items: moved, no count change
```

**D1-2 Values never run code when they die.** Only classes have `drop`. A value's memory is freed
after its last use (Mojo's ASAP rule, phase2-swift-mojo idea 3), which can only lower peak memory.
What other languages express with a guard value (a lock guard, a borrow guard) is a projection
function in D1 (D1-4), so the early-free trap Mojo documents (a guard released too early) cannot
arise: there are no guard values.

**D1-3 Objects share.** `class C` declares objects reached through handles. `b = a` makes a second
name for one object. Handles are counted (non-atomic unless the class is `@sync`); the object's
`drop` runs when the count reaches zero. Any method may change the object's fields through `self`;
there is no `mut self` on classes. A field declared `weak` holds a handle that does not keep the
object alive; reading it gives `Some(obj)` while the object lives and `None` after.

```ember
class Node:
    value: i32
    weak parent: Option[Node] = None      # a back edge
    children: Array[Node] = []
```

**D1-4 Projections work on a part where it lives.** There are five forms, all second-class:

```ember
fn heal(mut p: Player):              # 1. a `mut` parameter changes the caller's place
    p.hp += 10
heal(players[3])                     #    no mark at the call (Ember's [FN-2a] stays)

for p in players:                    # 2. a loop binds each element in place
    p.hp -= 1

with boss = enemies[0]:              # 3. `with` names a place for a block
    boss.hp //= 2

match self.state:                    # 4. `match` binds the parts of a place in place
    Walking(path): path.pop()
    _: pass

fn get[T](mut self, e: Entity) -> T: # 5. a function that `yield`s a place is an accessor:
    with pool = self.pool[T]():      #    the caller works on the yielded place, then the
        yield pool.dense[pool.slot(e)]   #  function's frame finishes (Python's
                                     #    @contextmanager shape: one yield, code after it runs last)
```

A `gen fn` that yields places is an iterator over parts (the ECS view, section 3.5). Plain `=`
never makes a projection: it copies (D1-1). That is the line a Python programmer must learn:
**`for`, `with` and `match` work in place; `=` copies a struct; `=` shares an object.**

**D1-5 Exclusivity at one call site (E-EXCL), the only borrow rule left.** In one call, or one
`with` header, a place passed to be changed may not overlap another argument, except:

- the overlap is plain data (numbers, and structs of numbers): two names for one plain value cannot
  free anything, so they are allowed, as Python allows passing one object twice;
- the other argument is a part *inside* the changed place (`self.note(self.name)`): it is passed as
  a copy (a count increment for its buffers), so nothing conflicts.

Where the overlap depends on run-time indices (`collide(bodies[i], bodies[j])`) and the element type
owns heap memory, the compiler inserts `if i == j: panic(...)`; for plain elements it inserts
nothing.

**D1-6 No address survives a change.** A projection held by `for`, `with`, `match` or a suspended
accessor or generator is a *path* (the root, its fields, its indices or keys). The compiler keeps
the address in a register while nothing can change what it points into, and fetches it again from
the path after a call or statement that can, with a bounds check (or a key lookup for a `Map`). An
index that was computed once and kept (a literal or a local) is also checked against the buffer's
*shift version*, a counter that `remove`, `insert`, `swap_remove`, `sort`, `retain` and `clear`
increase and `push` does not: if the elements moved, using the projection panics instead of naming
another element. An index the path recomputes from the container's own state (`pool.slot(e)`) is
simply recomputed.

```ember
with boss = enemies[0]:
    enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))   # may move the buffer
    boss.hp //= 2          # boss is fetched again: enemies[0] in the new buffer
    enemies.remove(0)
    boss.hp -= 1           # panic: "boss names enemies[0], which remove() on line 4 moved"
```

**D1-7 Lending to a call.** A `mut` or borrowed parameter receives an address, which the callee
cannot fetch again. For a local value, D1-5 guarantees nothing else reaches it during the call. For
a place reached through an object handle or a module variable, when the callee may reach that object
(it is not *quiet*, Ember's own word for this in `[EXC-19]`):

- a borrowed argument is passed as a copy (struct bits, plus a count increment per buffer), so the
  callee reads a value nobody else can change;
- a stored closure is called through a copy of the closure value whose box is retained for the call,
  so a handler that removes itself from its list keeps running safely (W4);
- a `mut` argument that lies inside a list is passed by address and the list is **pinned** for the
  call: a bit in the list's count word. While pinned, changing the list's size, replacing it, or
  copying-on-write it panics ("players changed size while players[3] is lent to update(), line 12");
  writing elements through other names is allowed and seen by the callee, as in Python. The bit sits
  in the count word that every change already tests, so the fast paths pay nothing.

**D1-8 A lost change is an error (E-COPYLOST).** A local initialised by copying a place (not a
temporary), changed in part, and never used whole afterwards (not stored, passed, returned or copied)
is a compile-time error with two fix-its, because the change can never be seen:

```text
error[E-COPYLOST]: `boss` is a copy of `enemies[0]`, so this change is lost
 7 |     boss = enemies[0]
   |     ---- a copy is made here: `Enemy` is a struct, a value
 9 |     boss.hp //= 2
   |     ^^^^^^^^^^^^^ changes the copy, which is never used again
   = help: change the element where it is: `enemies[0].hp //= 2`
   = help: or name it in place: `with boss = enemies[0]:`
```

**D1-9 Threads.** A value goes to another thread by move (the sender loses it) or, when its buffers
are shared with values the sender keeps, as a deep copy, so counts on values are never touched by
two threads and never need atomics. A scoped task may read the caller's values through a read
projection; a copy it makes of such data is a deep copy. An object crosses threads only as a
`@sync class`, whose fields are immutable after `init` and change through `Atomic`/`Mutex`, as
today (`[THR-1]`). `m.lock()` is a projection function: `with g = m.lock():` and nothing can escape
the block.

### 1.2 What is checked at compile time

| Check | Error | Replaces |
|---|---|---|
| one call site passes overlapping places, one to be changed, the overlap owning heap memory | E-EXCL | `[BRW-1]`..`[BRW-11]`, `E3020`..`E3025` |
| a change made to a copy can never be seen | E-COPYLOST | (new: the price of D1-1) |
| a projection-yielding function yields exactly once on every path | as Python's `@contextmanager` | `[LT-1]`, `@borrows`, `E3060`..`E3065` |
| definite initialisation, types, `dyn` compatibility, `@sync` immutability, effects | as today | |

Nothing else: no regions, no lifetimes, no loans, no "borrowed by this loop", no "does not live long
enough". The compiler's analyses that make D1 fast (section 1.5) never refuse a program; they only
choose between a cheaper and a dearer translation.

### 1.3 What is checked at run time, and what each costs

| Mechanism | When it runs | Cost per use | Source of the figure |
|---|---|---|---|
| bounds check | an index the compiler cannot prove in range | a compare and a predicted branch | (as today; not measured here) |
| uniqueness test (copy on write) | before changing a buffer in place | a load, a compare, a predicted branch; hoisted out of a loop whose body cannot share the buffer | +10% on 16.8M random writes when **not** hoisted: `unique_set` 103.7 ms vs `inline_set` 94.5 ms, phase2-rc-elision.md, `target/redesign/phase2-rc-elision/results.txt` |
| hidden copy | changing a value while a copy of it still exists | an O(n) copy, reported by `ember inspect` | one write into a shared 64 MiB list: 17.8 ms and 2x peak memory (`cow_first_set`, same file) |
| count pair on a buffer or object | a copy the compiler cannot turn into a move or a borrow | one non-atomic increment and decrement | about 0.84 ns per pair (phase2-rc-elision.md, `boxed_iter_rc` vs `boxed_iter`) |
| weak read | each read of a `weak` field | a load of the target's strong count and a compare | the compare part of a generational check, 0.26 ns in order, 2.80 ns random (phase2-vale-austral-pony.md section 1.3) |
| fetch again (D1-6) | after a change while a projection is held | one to three loads and a bounds check | UNMEASURED |
| shift check (D1-6) | the same, for an index kept from before | a load and a compare | UNMEASURED |
| pin (D1-7) | lending a list element of an object to a call that may reach the object | set and clear a bit in the count word (two stores); fast paths unchanged | UNMEASURED |
| distinctness check (D1-5) | two `mut` elements chosen at run time, element owning heap memory | a compare | (trivial) |
| closure call from a list (D1-7) | each call of a stored closure that may change the list | one count pair on the closure's box | 0.84 ns per pair (above) |
| boxed captured variable | a closure that escapes and writes a captured variable | one allocation when the frame starts, one load per access | UNMEASURED |

Memory per item:

| Item | D1 | Ember today | Source |
|---|---|---|---|
| object header | 16 bytes: strong u32, weak u32, type pointer | 24 bytes plus a 4-byte access word per non-`Copy` field | `[OBJ-1]`, `[EXC-19]` |
| collection value | 8 bytes (one pointer; an empty collection points at a shared static buffer) | 24 bytes (pointer, length, capacity) | D1 design; Ember: `ember_vec` in `runtime/ember_rt/include/ember_rt.h` (line 2022) |
| collection buffer header | 24 bytes: count u32 (with the pin bits), shift version u32, length u64, capacity u64 | none (length and capacity live in the value) | D1 design; Ember: same file |
| closure value | 2 words (function, box); the box only when an escaping closure writes a capture | 1 to 4 words, `[CLO-10]` | |
| `dyn I` value | 2 words (type table, counted box) | `Box[dyn I]`: 2 words | |

### 1.4 What disappears from the language

`ref`, `ref mut`, `Span`/`MutSpan`/`str` as storable types (they stay as parameter types only, the
C-speed way to pass a slice: pointer and length), `@borrows`, regions, `owned` parameters,
`owned fn`, `Box`, `Shared`, `SyncShared` (a `@sync class` covers it), `Weak[T]` as a type (a `weak`
field modifier instead), `Cell`, `RefCell`, guard types, `mut self` on class methods, `.clone()`
(a copy is `=`; an eager deep copy is `copy(x)`), per-field access words and exclusivity panics.

What stays: `mut` on parameters and on struct methods' `self` (one word, at the declaration only),
`class`, `@sync`, `Atomic`, `Mutex`, `Pool`/`Handle` (as a library), arenas (as a library whose
allocations are handed out as projections), `unsafe` and raw pointers.

### 1.5 What it needs from the compiler

These analyses decide only how cheap the translation is, never whether a program compiles:

1. **Last use** (exists today for borrows, `[BRW-2]`): turns copies into moves.
2. **Uniqueness facts**: a buffer made, pushed to or unshared is unique until something copies it;
   used to hoist uniqueness tests out of loops and to skip them after a library call that leaves the
   buffer unique.
3. **Quiet calls** (exists today for `[EXC-19]`): which calls cannot reach a given class's fields;
   decides where D1-6 fetches again and where D1-7 copies or pins.
4. **Projection summaries**: which part of its `mut` parameters an accessor yields (inferred from the
   `yield` path, like `[BRW-10]`'s field sets), so two accessors on disjoint parts at one site pass
   D1-5 (the ECS view's two pools).
5. **Range facts for induction variables**: one range check before a loop, and again after a call
   that may change the array.
6. **Escape of closures**: whether a function keeps a closure it is given (a summary passed to
   callers, as `[TYP-15]`'s store rule is today); decides whether a variable the closure writes is
   boxed (D1-1's closures, A12).

Risk: (2) and (5) are standard; (3), (1) and (6) exist in some form in Ember's compiler; (4) is new. How much of
(2) Swift's optimiser does (its copy-on-write hoisting) is UNVERIFIED here.

### 1.6 What changes in the spec

- **Deleted**: `[BRW-1]`..`[BRW-11]`, every `[LT-*]` rule, `[OWN-3]`, `[OWN-4]`, `[EXP-6]`'s move
  errors, `[EXC-1]`..`[EXC-19]`, `[RC-5]`'s "a borrow keeps the handle" (replaced by D1-6/D1-7),
  `[TYP-14]`, `[TYP-15]`, `[TYP-15a]`, `[TYP-34]`, `[SPN-1]`'s borrow rule, `[CELL-*]`,
  `[HEAP-3]`..`[HEAP-6]`, `[HEAP-10]`, `[CORO-6]` (`E2221`), `[CORO-12]`'s `E2229`, `[CLO-15]`,
  `[SEL-1]`'s eleven-row table (three rows remain: value, object, weak field).
- **Changed**: `[OBJ-1]` (16-byte header, no access words), `[OWN-1]`/`[OWN-2]` (values are freed after
  their last use; only classes have `drop`), `[CLO-2]` (captures are by reference, Python-like; an
  escaping closure that writes a capture boxes it), `[CLO-13]` (an escaping closure that captures a
  projection, such as a `for` loop's element, captures a copy of it, since a projection cannot
  escape), `[CTL-1]`/`[CTL-2]` (a loop binds elements in
  place and holds a path; adding during a loop is allowed and seen; a shifting change during a
  plain loop panics at the next step), `[WK-*]` (a `weak` field modifier; reading clears a dead
  target).
- **Kept**: `[RC-1]`..`[RC-4]`, `[RC-6]`, `[WK-1]` (strong cycles leak, reported in debug),
  `[HND-*]`, `[THR-*]` for objects, `[PAN-1]` (no unwinding), `[FN-2a]` (no mode at a call).

---

## 2. The annoyances, one by one

Kinds: **(a)** a more Pythonic spelling of the same guarantee; **(b)** a smarter mechanism that
makes the hazard impossible or harmless. "Phase 1 (5)" names the elimination idea each verdict
builds on.

| Id | Verdict | Kind | How, in D1 | Phase 1 (5) used |
|---|---|---|---|---|
| A1 whole-object borrow | removed | (b) | a loop over `self.players` holds a path; calling `self.note()` inside it is legal and the loop fetches its list again after the call (D1-6). Residue: E-EXCL for `players[i].update(players)` on a local value (section 2.1) | A1(b) |
| A2 container invalidation | removed | (b) | no address survives `push` (D1-6); `with boss = enemies[0]:` then `enemies.push(...)` works; `boss = enemies[0]` copies, and a lost change is E-COPYLOST | A2(a) as a path, without A2(b)'s pages |
| A3 two elements at once | removed for plain data; moved to run time otherwise | (b) | two names for one plain value cannot free anything (D1-5); for elements owning heap memory, an inserted `i != j` check | A3(b), A3(a) |
| A4 change while iterating | removed for adding; moved to run time for removing in a plain loop | (b) | a loop sees elements added during it (Python's list); a shifting change makes the next step panic instead of silently skipping; the ECS view (a library generator) allows despawning in place (W1) | A4(b) without tombstones |
| A5 returning references | removed | (b) | an accessor `yield`s; its frame stays alive while the caller works, so "the local dies at return" (ODR-286's `E3060`) cannot happen; values are returned as copies (counts) | A5(b); phase2 Hylo H1 |
| A6 structs holding references | removed | (b) | a struct holds values (sharing buffers by count) or object handles; nothing to annotate. Residue: a struct cannot hold a *changeable* window into another value (an EnTT `handle` over a value registry); make the registry a class (section 6.1) | A6(b) |
| A7 self-referential structures | removed | (b) | a word of a `String` is a `String` sharing the buffer (buffer, start, length; count +1), so `Document(text, words)` has no self-reference | A7(b) |
| A8 graphs, parent pointers, cycles | moved to run time; partly kept | (b) | `weak parent: Option[Node]`, one word; each read is a count test; strong cycles still leak and are reported at exit in debug (`[WK-15]`) | A8(a) without inference |
| A9 shared mutable state | removed (one rare panic left) | (b) | an object is the shared cell; no `RefCell`, no access words, no exclusivity panics; the only run-time borrow panic left is a pinned list changing size during a lent call (D1-7) | A9(b) |
| A10 ownership kinds | removed | (a) | three kinds: value, object, `weak` field; `Box` is inferred for recursive enums and `dyn` values | A10(a) |
| A11 moves, clone | removed | (b) | copy on write makes two owners harmless; a last use moves invisibly (D1-1) | A11(a) |
| A12 closures | removed (cycles kept) | (a)+(b) | captures are by reference as in Python; an escaping closure that writes a capture boxes it (b); no `owned fn`, `FnMut`/`FnOnce` or `'static` (a). Kept: a closure stored in an object and capturing that object is a strong cycle | A12(a) |
| A13 trait objects, downcasting | removed (dyn compatibility kept) | (a) | `cast p as Pool[T]` on a place is a place (ODR-286's missing piece); `dyn I` is a value with no `Box`. Kept: a generic method makes an interface not `dyn`-compatible | A13(a) |
| A14 async | Pin removed; Send for objects kept | (b) | a suspended frame holds paths, not addresses (D1-6), so moving it is harmless; values need no `Send` (D1-9) | A14(b)(1) |
| A15 threads | partly removed; kept for objects | (a) | values move or deep-copy to a thread with no `Send`/`Sync` words; objects still need `@sync` and `Atomic`/`Mutex` | A15(a) partly |
| A16 generics friction | kept | | not part of the memory model | |
| A17 error messages | removed mostly | (b) | two compile-time memory errors remain, each with fix-its the checker re-checks before showing (A17(a)) | A17(b) |
| A18 initialisation | mostly kept, partly removed | (a) | as today, plus `init` may call a helper that reads only fields already set (`[BRW-10]`'s read sets) | A18(a) |
| A19 global mutable state | removed for one thread; kept across threads | (a) | `var LOG: Array[String] = []` at module level, main-thread only (effect-checked), treated like an object's field by D1-6/D1-7 | A19(a) |
| A20 unsafe, FFI | kept | | raw pointers in `unsafe`; a C callback's user data is a counted closure box, retained by registration (as `[FFI-21]`) | |
| A21 map entry update | removed | (a) | `counts.get(w, 0)` returns a value, no `Option[ref T]`; `counts[w] += 1` projects the entry | A21(a) |
| A22 view of a temporary | removed | (b) | `line.to_lower().trim()` returns a `String` sharing the temporary's buffer, which lives as long as the result | A22(b) |
| A23 moving a field out from behind `mut self` | removed | (b) | there are no moves; `match self.state:` binds in place and an assignment replaces the state | A23(b) |
| A24 struct, Box or class changes what compiles | removed | (b) | one rule set for both kinds; the kind decides copy or share and the cost, never whether it compiles | A24(b) |
| A25 a collection of views | removed | (b) | `words = line.lower().split()` is an `Array[String]` of buffer-sharing words; it can be stored anywhere | A25(b) |
| A31 caching in a read-only method | removed for classes; kept for structs | (b) for classes | an object's fields are writable through any handle; a struct method that writes needs `mut self` | A31(a) for classes |
| A32 callbacks that receive their holder | removed (removal in a plain loop moved to run time) | (b) | a stored closure is called through a retained copy (D1-7), so it may add or remove handlers, itself included; a plain `for` over the list panics at the next step after a removal | A32(a) without the put-back |
| A33 lock ceremony, references out of a guard | removed | (b) | `lock()` is an accessor; nothing escapes `with g = m.lock():`; returning `g[0]` returns a copy | A33(a)+(b) |
| A34 clone-before-move for handles | absent before and after | | handles copy with a count, as today | |

### 2.1 Notes with code, for the ids not shown in section 3

**A3**, plain and non-plain elements:

```ember
fn collide(mut a: Body, mut b: Body):      # Body: x, y, vx, vy (numbers only)
    a.vx, b.vx = b.vx, a.vx
collide(bodies[i], bodies[j])               # compiles; i == j swaps a body with itself, as Python would
fn merge(mut a: Squad, mut b: Squad):       # Squad holds an Array
    a.units.extend(b.units)
merge(squads[i], squads[j])                 # compiles to: if i == j: panic("squads[i] passed twice to be changed")
```

**A1's residue (E-EXCL)**, the one form still refused:

```ember
for p in players:                  # players is a local value
    p.update(players)              # E-EXCL: players[i] is changed by update while players is also passed
# fixes the help names: pass what update needs (p.update(enemies_near(players, p))), or copy(players)
```

**A5**, conditionally returned parts, no annotation:

```ember
fn longer(a: String, b: String) -> String:
    return a if a.len() >= b.len() else b          # a copy: one count increment

fn get_or_load(mut cache: Map[int, String], id: int) -> String:
    if id not in cache:
        cache[id] = f"asset {id}"
    yield cache[id]                                 # the caller works on the entry in place
```

**A12**, a callback that bumps a local, the program Rust and Ember today refuse (phase 1 A12):

```ember
fn game() -> int:
    score = 0                         # boxed: an escaping closure writes it
    b.on_click(fn(): score += 1)
    b.click()
    return score                      # 1, as in Python
```

**A13**, ODR-286's downcast:

```ember
fn pool[T](mut self) -> Pool[T]:
    match self.pools[type_index[T]()]:
        Some(p): yield cast p as Pool[T]       # a downcast of a place is a place: one compare
        None: unreachable()
```

**A14**, a generator walking its own list (Ember today `E2221`):

```ember
gen fn lines() -> Generator[String]:
    buf = ["a", "b"]
    for s in buf:            # the loop holds a path into the frame: moving the frame is harmless
        yield s              # a copy: one count increment
```

**A19**:

```ember
var FRAME: int = 0
var LOG: Array[String] = []
fn tick():
    FRAME += 1
    LOG.push("tick")         # main thread only; a thread function touching LOG is refused
```

**A21, A22, A23**:

```ember
counts: Map[String, int] = {}
for w in text.split():
    counts[w] = counts.get(w, 0) + 1          # A21: a value, no Option[ref int]
word = line.to_lower().trim()                  # A22: word shares the lowered text's buffer
match self.state:                              # A23: in place, nothing moved out
    Walking(path):
        path.pop()
        if path.is_empty():
            self.state = State.Idle
    _: pass
```

**A31** (struct kept, class removed):

```ember
class Mesh:
    verts: Array[f32]
    radius_cache: Option[f32] = None
    fn radius(self) -> f32:                    # writes through any handle: no mut, no Cell
        if self.radius_cache is None:
            self.radius_cache = max(abs(v) for v in self.verts)
        return self.radius_cache.unwrap()
```

**A33**:

```ember
fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():       # lock() yields the data; the unlock runs after the block
        return g[0]               # a copy (one count increment): Python's meaning
```

---

## 3. The test programs in D1 syntax

Ember's syntax stays where D1 does not change it. New: `yield` in a plain `fn` (an accessor),
`weak` on a field, `with name = place:` binding a place. `h_rand()`/`h_seed()` stand for the
harness's generator, called exactly where the workload says.

### 3.1 W1, the ECS

```ember
from std.ecs import Registry, Entity

struct Position:
    x: f32
    y: f32

struct Velocity:
    dx: f32
    dy: f32

fn main():
    reg = Registry()
    for i in 0..1_000_000:
        e = reg.create()
        reg.emplace(e, Position(x=(i % 1000) as f32, y=(i // 1000) as f32))
        if i % 2 == 0:
            reg.emplace(e, Velocity(dx=(i % 7) as f32 * 0.5, dy=(i % 5) as f32 * 0.25))

    for s in 0..100:
        for e, pos, vel in reg.view[Position, Velocity]():
            pos.x += vel.dx                      # pos and vel are the components, in place
            pos.y += vel.dy
            if e.id % 100 == s:
                reg.destroy(e)                   # legal inside the loop: it holds paths
                n = reg.create()
                reg.emplace(n, Position(x=0, y=0))
                reg.emplace(n, Velocity(dx=1, dy=1))

    checksum: u64 = 0
    for e, pos in reg.view[Position]():
        checksum += ((pos.x * 4.0) as i64 + (pos.y * 4.0) as i64) as u64
    checksum += reg.view[Position]().len() as u64
    println(checksum)
```

The despawn and the spawn take effect at once. The view walks the smaller pool from its end (as
EnTT does): `destroy` swaps the last element into the current slot, and the last element was either
visited already or spawned in this step, so every entity present at the start is visited exactly
once and no spawned one is (checked: the model's checksum equals the reference's, section 5).

### 3.2 W2, the tree with parent pointers

```ember
class Node:
    value: i32
    weak parent: Option[Node] = None
    children: Array[Node] = []

fn main():
    h_seed(42)
    nodes = [Node(value=0)]
    for i in 1..1_000_000:
        parent = nodes[h_rand() % i]
        child = Node(value=(i % 1000) as i32, parent=parent)
        parent.children.push(child)
        nodes.push(child)

    depth_sum = 0
    for p in 0..10:
        for node in nodes:
            n = node
            while Some(up) = n.parent:
                depth_sum += 1
                n = up
        for i in 1..1_000_000:
            if (i + p) % 100 == 0:
                new_parent = nodes[h_rand() % i]
                child = nodes[i]
                if Some(old) = child.parent:
                    old.children.swap_remove(old.children.index_of(child))
                child.parent = new_parent
                new_parent.children.push(child)

    total = depth_sum
    for node in nodes:
        total += node.value * node.children.len()
    println(total as u64)
```

### 3.3 W3, game objects that refer to each other

```ember
open class Thing:
    let id: int
    x: i32
    y: i32
    health: i32
    weak target: Option[Thing] = None      # never dangles: reads as None once the target is destroyed

    virtual fn update(self):
        pass

class Ship(Thing):
    override fn update(self):
        if Some(t) = self.target:
            self.x += sign(t.x - self.x)
            self.y += sign(t.y - self.y)

class Rock(Thing):
    override fn update(self):
        self.x = (self.x + 1) % 1000

class Turret(Thing):
    override fn update(self):
        if Some(t) = self.target:
            if abs(t.x - self.x) + abs(t.y - self.y) < 300:
                t.health -= 1

fn sign(v: i32) -> i32:
    return 1 if v > 0 else (-1 if v < 0 else 0)

fn make(id: int) -> Thing:
    x = (h_rand() % 1000) as i32
    y = (h_rand() % 1000) as i32
    health = (100 + id % 50) as i32
    return match id % 3:
        0 => Ship(id, x, y, health)
        1 => Rock(id, x, y, health)
        _ => Turret(id, x, y, health)

fn main():
    h_seed(7)
    live: Array[Thing] = []
    for i in 0..100_000:
        t = make(i)
        if i > 0:
            t.target = live[h_rand() % i]
        live.push(t)

    next_id = 100_000
    destroyed = 0
    for frame in 0..1000:
        for t in live:
            t.update()
        before = live.len()
        live.retain(fn(t) => t.health > 0)      # the dropped objects die here
        dead = before - live.len()
        destroyed += dead
        for _ in 0..dead:
            t = make(next_id)
            next_id += 1
            if live.len() > 0:
                t.target = live[h_rand() % live.len()]
            live.push(t)

    total = destroyed * 1_000_000
    for t in live:
        total += t.x + t.y + t.health
    println(total as u64)
```

### 3.4 W4, callbacks with captured mutable state

```ember
struct Handler:
    on_event: fn(Dispatcher, int, int)
    counter: fn() -> int

class Dispatcher:
    handlers: Array[Handler] = []
    current: int = 0

    fn register(self, h: Handler):
        self.handlers.push(h)

    fn unregister_current(self):
        self.handlers.remove(self.current)

    fn dispatch(self, k: int):
        self.current = k % self.handlers.len()
        self.handlers[self.current].on_event(self, k, k % 1000)

fn main():
    total = 0                                 # written by escaping closures: kept in a box

    fn make_handler(m: int) -> Handler:
        counter = 0
        fn on_event(d: Dispatcher, k: int, v: int):
            counter += v * m
            total += 1
            if k % 100_000 == 99_999:
                d.unregister_current()        # removes this very handler while it runs: safe
                d.register(make_handler(k % 17 + 1))
        return Handler(on_event=on_event, counter=fn() => counter)

    d = Dispatcher()
    for i in 0..10_000:
        d.register(make_handler(i % 13 + 1))
    for k in 0..10_000_000:
        d.dispatch(k)

    checksum = total
    for h in d.handlers:
        checksum += h.counter()
    println(checksum as u64)
```

The second closure, `fn() => counter`, shares `counter` with `on_event`, as two Python closures
over one variable do; it is how the program reads each handler's counter at the end.

### 3.5 The EnTT registry: `get` and `view` (the library side)

```ember
interface AnyPool:
    fn remove(mut self, e: Entity)

struct Pool[T]:
    sparse: Array[u32] = []          # entity id -> position in dense, NONE when absent
    packed: Array[Entity] = []
    dense: Array[T] = []

    fn contains(self, e: Entity) -> bool:
        return e.id < self.sparse.len() and self.sparse[e.id] != NONE

    fn slot(self, e: Entity) -> int:
        i = self.sparse[e.id]
        if i == NONE:
            panic(f"entity {e.id} has no {type_name[T]()}")
        return i as int

    fn remove(mut self, e: Entity):             # swap and pop, as EnTT
        if not self.contains(e):
            return
        i = self.slot(e)
        self.dense.swap_remove(i)
        self.packed.swap_remove(i)
        if i < self.packed.len():
            self.sparse[self.packed[i].id] = i as u32
        self.sparse[e.id] = NONE

extend[T] Pool[T] implements AnyPool

struct Registry:
    pools: Array[Option[dyn AnyPool]] = []     # slot type_index[T]() holds Pool[T]
    next: u32 = 0

    fn create(mut self) -> Entity:
        self.next += 1
        return Entity(id=self.next - 1)

    fn pool[T](mut self) -> Pool[T]:            # an accessor
        i = type_index[T]()                     # a compile-time constant per type
        while self.pools.len() <= i:
            self.pools.push(None)
        if self.pools[i] is None:
            self.pools[i] = Some(Pool[T]())
        match self.pools[i]:
            Some(p): yield cast p as Pool[T]
            None: unreachable()

    fn emplace[T](mut self, e: Entity, value: T):
        with p = self.pool[T]():
            if e.id >= p.sparse.len():
                p.sparse.resize(e.id + 1, NONE)
            if p.sparse[e.id] != NONE:
                panic("entity already has this component")
            p.sparse[e.id] = p.dense.len() as u32
            p.packed.push(e)
            p.dense.push(value)

    fn get[T](mut self, e: Entity) -> T:        # EnTT's get: the component, in place
        with p = self.pool[T]():
            yield p.dense[p.slot(e)]

    fn destroy(mut self, e: Entity):
        for slot in self.pools:
            match slot:
                Some(pool): pool.remove(e)       # a dyn call
                None: pass

    gen fn view[A, B](mut self) -> (Entity, A, B):
        with a = self.pool[A](), b = self.pool[B]():   # disjoint slots: passes D1-5
            k = b.packed.len() - 1
            while k >= 0:
                e = b.packed[k]
                if a.contains(e):
                    yield e, a.dense[a.slot(e)], b.dense[k]
                k -= 1
```

User side:

```ember
reg.get[Position](e).x += 1.0                 # one statement
with pos = reg.get[Position](e):              # a block
    pos.x += 1.0
    reg.emplace(other, Velocity(dx=1, dy=1))  # may grow other pools, even this one
    pos.y += 1.0                              # pos is fetched again: p.dense[p.slot(e)]
p = reg.get[Position](e)                      # a copy of the component: reading is fine;
p.x += 1.0                                    # E-COPYLOST if p is never used whole afterwards
```

This answers ODR-286 in full. The owner's first spelling, "`pool = cast self.pools[key] as Pool[T]`
then return a changeable element", is `with pool = cast ...: yield pool.dense[i]`. The hazard the
proof found ("a second handle to the same pool can grow the pool's array while the reference
lives") cannot happen: a pool held as a value has no second handle, and if the registry is made a
class, the caller's projection is fetched again after any call that may reach the registry (D1-6),
or the list is pinned while the projection is handed to a function (D1-7). EnTT's paged storage is
not needed, because nothing keeps an address across a change; phase2-hylo-csharp measured pages at
+30% on random access by index (its results.txt), a cost D1 does not pay. `view[A, B]` is written
for two types; with the EnTT design's tuple form (`view[(A, B)]`, ecs-entt-design.md point 1) it is
the same body over a tuple.

### 3.6 The three worst programs of phase 1

Selection rule: the phase-1 ids of severity "high" for which Ember today is "same" or "worse", and
which W1 to W4 and the registry do not already cover (A4 is W1, A8 is W2, A9 and A13 are W3, A12
and A32 are W4, A24 is section 3.5). That leaves A1, A2 with A11, and A25 (with A22 and A7).

**P1 (A1): a game tick that logs a death while iterating the players.** Rust: E0499; Ember today
accepts it only for private struct methods.

```ember
struct Player:
    name: String
    hp: int

struct Game:
    players: Array[Player]
    log: Array[String]
    frame: int

    fn note(mut self, msg: String):
        self.log.push(f"[{self.frame}] {msg}")

    pub fn tick(mut self):
        for p in self.players:
            p.hp -= 1
            if p.hp <= 0:
                self.note(f"{p.name} died")       # compiles, pub or not
```

**P2 (A2 and A11): a boss splits off a minion; a message is sent and logged; a list is snapshotted.**
Rust: E0499, E0382; Ember today: E3022, E3040.

```ember
struct Enemy:
    x: float
    hp: int

fn split_boss(mut enemies: Array[Enemy]):
    with boss = enemies[0]:
        enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
        boss.hp //= 2                     # enemies[0], fetched again after the push

fn split_boss_python_spelling(mut enemies: Array[Enemy]):
    boss = enemies[0]                     # a copy: Enemy is a struct
    enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp // 2))
    boss.hp //= 2                         # E-COPYLOST, with the two fix-its of D1-8
                                          # (with `class Enemy`, this version compiles and aliases, as in Python)

fn notify(m: Msg):
    send(m)                               # a copy (count +1) or, at a last use, a move
    log(m)

fn snapshot():
    items = [1, 2]
    snapshot = items
    items.push(3)                         # snapshot stays [1, 2]
```

**P3 (A25, A22, A7): the words of a line; a document that owns its text and its words.** Rust: E0716,
E0515/E0505; Ember today: E3063, E3060.

```ember
fn count_words(line: str) -> int:
    words = line.to_lower().split()       # Array[String]; every word shares the lowered buffer
    return words.len()

struct Document:
    text: String
    words: Array[String]

fn load(text: String) -> Document:
    return Document(text=text, words=text.split())   # no self-reference: the words count the buffer
```

---

## 4. What each compiles to (C sketches of the hot loops)

The full code is in the C models (section 5); these are the inner loops, in the same names.
`Buf` is the counted buffer (`count, shifts, len, cap`, then the elements); `EL(b, T)` its elements.

**W1**, the view inlined into the step loop:

```c
Pool *pa = reg_pool(&reg, 0, &VT_POS), *pb = reg_pool(&reg, 1, &VT_VEL); /* downcast compare, box test */
Buf *da = buf_mut(&pa->dense, 8, NULL);          /* pos is written: one uniqueness test per step */
Buf *sa = pa->sparse, *pk = pb->packed, *db = pb->dense;
int64_t k = (int64_t)pk->len - 1;
if (k >= 0 && (uint64_t)k >= db->len) d1_panic("index out of bounds");   /* hoisted range check */
for (; k >= 0; k--) {
    uint32_t e = EL(pk, uint32_t)[k];
    if (e >= sa->len) continue;                    /* a.contains(e) */
    uint32_t idx = EL(sa, uint32_t)[e];
    if (idx == NONE) continue;
    if (idx >= da->len) d1_panic("index out of bounds");   /* the one data-dependent check */
    Position *pos = (Position *)&EL(da, Comp8)[idx];
    const Velocity *vel = (const Velocity *)&EL(db, Comp8)[k];
    pos->x += vel->dx; pos->y += vel->dy;
    if (e % 100 == s) {
        reg_destroy(&reg, e); /* + create, two emplaces */
        pa = reg_pool(&reg, 0, &VT_POS); pb = reg_pool(&reg, 1, &VT_VEL);   /* fetch again */
        da = pa->dense; sa = pa->sparse; pk = pb->packed; db = pb->dense;
        if (k > 0 && ((uint64_t)(k - 1) >= pk->len || (uint64_t)(k - 1) >= db->len)) d1_panic("index out of bounds");
    }
}
```

**W2**, the depth walk:

```c
for (uint64_t i = 0; i < nb->len; i++) {           /* nodes does not change: hoisted */
    Node *n = EL(nb, Node *)[i];                     /* no count: nothing can release it */
    uint64_t d = 0;
    for (;;) {
        Node *up = n->parent;                        /* weak read */
        if (!up) break;
        if (up->strong == 0) { n->parent = NULL; weak_release(up); break; }   /* never taken here */
        d++; n = up;
    }
    depth_sum += d;
}
```

**W3**, the update loop and a turret:

```c
for (uint64_t i = 0; i < lb->len; i++) {            /* update() cannot reach `live`: no fetch again */
    Thing *t = EL(lb, Thing *)[i];
    t->type->update(t);                              /* virtual call */
}
static void turret_update(Thing *o) {
    Thing *t = o->target;                            /* weak read: test the strong count */
    if (t && t->strong == 0) { o->target = NULL; weak_release(t); t = NULL; }
    if (t && abs(t->x - o->x) + abs(t->y - o->y) < 300) t->health -= 1;
}
```

**W4**, one event:

```c
const Buf *hb = d->handlers;                         /* fetched every event: the call may change it */
uint64_t n = hb->len;
if (n == 0) d1_panic("division by zero");
uint64_t j = (uint64_t)k % n;                        /* < n: no bounds check */
d->current = (int64_t)j;
Handler h = EL(hb, Handler)[j];                      /* the closure value, copied out */
HBox *b = h.ev_box;
b->rc++;                                             /* the call may remove this handler */
h.on_event(b, d, k, k % 1000);                       /* body: b->counter += v * b->m; b->up->total += 1; ... */
hbox_release(b);
```

**Registry `get`** (`reg.get[Position](e).x += 1.0`):

```c
Pool *p = reg_pool(&reg, 0, &VT_POS);                /* bounds, downcast compare, box uniqueness */
Buf *sp = p->sparse;
if (e >= sp->len) d1_panic("index out of bounds");
uint32_t i = EL(sp, uint32_t)[e];
if (i == NONE) d1_panic("entity has no Position");
Buf *d = buf_mut(&p->dense, 8, NULL);
if (i >= d->len) d1_panic("index out of bounds");
((Position *)&EL(d, Comp8)[i])->x += 1.0f;
```

**P1** (`Game.tick`):

```c
Buf *pl = buf_mut(&self->players, sizeof(Player), retain_players);   /* hoisted */
uint32_t shifts0 = pl->shifts;
for (uint64_t i = 0; i < pl->len; i++) {                              /* Python's live length */
    Player *p = &EL(pl, Player)[i];
    if (--p->hp <= 0) {
        String msg = format_died(&p->name);          /* a new value: no overlap with self */
        Game_note(self, msg);                        /* may change self (a pub method) */
        pl = buf_mut(&self->players, sizeof(Player), retain_players);  /* fetch again, unique again */
        if (pl->shifts != shifts0) d1_panic("players changed by note() during the loop");
    }                                                /* p is not used after the call. note's summary */
}                                                    /* (writes {log}) removes all three lines */
```

**P2** (`split_boss`, the `with` form):

```c
Buf *b = *enemies;
if (b->len == 0) d1_panic("index out of bounds");               /* with boss = enemies[0] */
Enemy minion = { EL(b, Enemy)[0].x + 1.0, floordiv(EL(b, Enemy)[0].hp, 2) };
PUSH(enemies, Enemy, minion, NULL);                             /* may move the buffer */
b = *enemies;                                                   /* boss fetched again; push does not shift: no shift check */
EL(b, Enemy)[0].hp = floordiv(EL(b, Enemy)[0].hp, 2);           /* 0 < len: known after the push */
```

**P3** (`split`), each word shares the buffer:

```c
for (each word found at [start, end)) {
    src_buf->count++;                                           /* one count per word, no allocation */
    PUSH(&words, String, ((String){src_buf, start, end - start}), retain_strings);
}
```

---

## 5. The C models for measurement

| File | What it models | Check run: its one line (checksum) | Reference checksum (scratch `ref/refN.c`) |
|---|---|---|---|
| `target/redesign/bench/D1/w1.c` | section 3.1 with the registry of 3.5 inlined where the report says | `D1-w1 ... checksum=2111831520` | 2111831520 |
| `target/redesign/bench/D1/w2.c` | section 3.2 | `D1-w2 ... checksum=647034814` | 647034814 |
| `target/redesign/bench/D1/w3.c` | section 3.3 | `D1-w3 ... checksum=9288233970` | 9288233970 |
| `target/redesign/bench/D1/w4.c` | section 3.4 | `D1-w4 ... checksum=34946993000` | 34946993000 |

Each was built once (`clang -O2`, clang 22.1.8, no warnings) and run once; each finished and printed
its line, exit code 0. After the build, two comments in `w2.c` and `w3.c` were changed (the type
`Node?` written `Option[Node]`); no code changed, so the binaries stand. Facts from the reference runs used below: W1 hands out 1,856,653 ids in all
(856,653 spawns); W2's parent walks take 148,749,741 steps over the ten passes; W3 destroys 9,173
objects.

What every model contains, because D1 needs it:

- **Buffers**: an `Array` is one pointer to a buffer with a 24-byte header; every in-place change
  first tests `count == 1`; `push` tests count and capacity in one branch; growth doubles from 4
  (`[HEAP-2]`); `swap_remove`, `remove` and `retain` bump the shift version; an empty array points at a
  shared static buffer whose count never reads 1.
- **Hoisting the compiler is allowed to do** (section 1.5): a uniqueness test before a loop whose body
  cannot share the buffer, and not again after a library call that leaves the buffer unique; one
  range check of an induction variable before its loop and after a call that changes the array.
- **Bounds checks** on every index the compiler cannot prove: W1's `dense[idx]` from the sparse array
  and the library's checks inside `remove`; W2's `nodes[h_rand() % i]` and `nodes[i]`; W3's
  `live[h_rand() % i]` in the setup (the frame-time `live[h_rand() % live.len()]` is proven).
- **W1**: the pools are `dyn AnyPool` values (type table, counted box) in an `Array` indexed by a
  compile-time type index; every `pool[T]()` projection does a bounds check, a type-table compare and
  a box uniqueness test; `destroy` calls each pool's `remove` through the type table; after the
  destroy branch the generator's projections are fetched again.
- **W2, W3**: objects with a 16-byte header; weak reads test the strong count and clear a dead target;
  weak stores adjust weak counts; destruction releases the weak target and frees the block when the
  weak count reaches zero; counts on handles only where section 3's code copies a handle into a list
  or holds one across a release (W2's `old`); none in the loops ([RC-2b]/[RC-2e]-style elision).
- **W4**: one counted box per `make_handler` call (counter, m, link to main's box) and one for main
  (`total`); two closures per handler share the box; each dispatch copies the closure value and
  retains its box around the call; `remove` shifts the list and releases the removed closures.

What every model leaves out, and why:

- **The integer overflow checks of `[TYP-8]`.** They are the language's arithmetic rule, the same in
  every memory model, and would only blur the comparison. The measurer can add them to every model
  alike. (Float to integer `as` saturates in Ember, `[TYP-6]`; every value converted in these
  workloads is in range, so the plain C cast gives the same result.)
- **Teardown at the end of `main`.** In D1 values are memory only, and objects without a user `drop`
  run no code when they die, so freeing them just before the process ends cannot be observed; the
  compiler skips it. Peak memory is reached before the end either way.
- **Pins and stale-projection checks.** None of the four programs lends an object's list element to a
  call that may reach it, or uses a projection after a shifting change, so D1 emits neither.

---

## 6. The sharp edges left

### 6.1 What would still surprise a Python programmer

Python behaviours checked with Python 3.14.2 (`py_d1.py` in the scratchpad): a list loop visits
items appended during it (`[1, 2, 3, 4]`); removing during a loop skips an element silently
(`[0, 0, 5]` becomes `[0, 5]`); a `@contextmanager` generator that yields twice raises
`generator didn't stop`; `boss = ps[0]; ps.append(P()); boss.hp //= 2` changes `ps[0]` (prints 5).

1. **`=` copies a struct.** In Python every name aliases an object; in D1 `boss = enemies[0]` copies
   when `Enemy` is a struct, and aliases only when it is a class. E-COPYLOST catches the change that
   is lost outright; it cannot catch `p = players[0]; p.hp -= 1; print(p.hp)` followed by
   `players.push(p)` meant as an update. The programmer must know which of their types are values.
   This is the main cost of D1 in Python-likeness.
2. **Struct or class is a choice made at the declaration**, and it decides copy or share (A10's
   residue): Python has only objects.
3. **`mut` on a parameter that changes a struct argument**, one word at the declaration. A function
   that changes an object needs nothing.
4. **Removing from a list during a plain `for` panics at the next step** (Python skips an element
   silently). Adding is seen by the loop, as in Python, and a loop that appends forever never ends,
   as in Python.
5. **A pinned list cannot change size** while one of its elements is lent, through an object, to a
   method that can reach the object (`for p in self.players: p.update(self)` with `update` pushing
   to `self.players`). Python allows it. The panic names both lines; the fix is a class element or a
   deferred push.
6. **A `with` name follows its path**: after `enemies.push`, `boss` is `enemies[0]` again (right);
   after `enemies.remove(0)`, using `boss` panics (Python's `boss` would still be the removed object).
7. **`weak` must be written on back edges**, and strong cycles leak (closures stored in an object and
   capturing it included); the debug profile reports them at exit (`[WK-15]`).
8. **Hidden copies.** Changing a value while a copy of it lives copies it, O(n), invisibly;
   `ember inspect` must list every place this can happen.
9. **E-EXCL** refuses `p.update(players)` where `p` is an element of the local list `players`.
10. **Threads**: a copy made inside a scoped task of data shared with the parent is a deep copy; an
    object crosses threads only as a `@sync class`.
11. **Dead objects behind weak fields keep their block** (header and fields, not their heap parts)
    until the weak field is read or its holder dies (W3's rocks never read their target).

### 6.2 What I expect to be slower than C, by how much, and why

All of this is **UNVERIFIED expectation** for the measurer; the operation counts come from the
reference runs (section 5) and the per-operation costs from the phase-2 files named.

- **W1, the ECS.** Against C doing the same EnTT algorithm: per visited entity one extra compare and
  predicted branch (`idx < dense.len`), about 50 million in all (100 steps over a Velocity pool of
  about 500,000); per step two type-table compares and three uniqueness tests; per spawn
  (856,653 in all) two indirect `remove` calls, about ten uniqueness tests and one fetch-again. I
  expect within 5% of the same algorithm in C. Memory: the data itself (two sparse arrays of
  1,856,653 ids, packed and dense arrays) plus six 24-byte headers and two boxes; the check run's
  peak was 37,924 KB, a single unrepeated observation.
- **W2, the tree.** D1's tree is 1,000,000 heap objects of 40 bytes (16-byte header, value, weak
  parent, one-pointer children list) plus about one children buffer per inner node (24-byte header
  and at least 4 slots of 8 bytes). Against C that also allocates nodes and links them by pointer,
  the extra is the weak read per parent step: the parent's strong count sits at offset 0 of the
  40-byte object whose `parent` field (offset 24) the next step reads, so it is usually on a cache
  line already fetched; at phase 2's in-order compare cost (0.26 ns) the 148,749,741 steps add about
  39 ms, and at its random compare cost (2.80 ns) about 416 ms; I expect near the low end. Against C
  that stores the tree as index arrays (`parent: int32[N]`, 4 MB), D1 is several times slower and
  about twice the memory, because a walk chases 40-byte objects spread over tens of megabytes
  instead of a 4-byte array: phase 2 measured objects against inline data at 3.56x to 9.74x on
  iteration and 2.44x memory (`boxed_iter`, `boxed_iter_shuffled`, phase2-rc-elision results.txt).
  That gap is the class layout, which the Python programmer chose by writing `class Node`; a D1
  program can write the index form with values and pay neither.
- **W3, the game objects.** Against C with heap objects and function-pointer dispatch: one weak test
  per target read, 2/3 of 100,000 objects per frame for 1,000 frames, about 66.7 million; the target's
  line is fetched for `t.x` anyway, so I expect the in-order cost (0.26 ns each, about 17 ms in
  all), not the random one (2.80 ns, about 187 ms). Memory: 48 bytes per object with the header, and
  up to 9,173 dead 48-byte blocks kept by unread weak fields (at most about 0.4 MB).
- **W4, the callbacks.** One extra count pair per event around the stored-closure call: 10,000,000 x
  0.84 ns, about 8.4 ms (phase2-rc-elision). The box indirections (`b->m`, `b->up->total`) are what a C
  program with a context pointer does too.
- **Everywhere**: the uniqueness test is free when hoisted, and +10% on random writes where the
  compiler cannot hoist it (phase2-rc-elision `unique_set`); none of the four workloads has such a
  loop.

---

## 7. Open questions for the owner (not ruled here)

1. **Removing during a plain loop**: D1 panics at the next step; Python skips silently; EnTT-style
   backward iteration tolerates it. Which?
2. **`=` on a struct**: copy (D1, Swift, Hylo) or alias as in Python? Aliasing would make D1-6 bind
   every local initialised from a place as a path, which breaks when the name is later rebound
   (Python rebinding never writes through); D1 chose copy plus E-COPYLOST.
3. **Pins (D1-7) or Swift's per-field run-time exclusivity** for the rare lent-element case: pins
   panic only on a size change of the lent list and cost nothing on fast paths; Swift's check panics
   on any conflicting access and costs a check per access (phase2-swift-mojo: up to +840% on one
   microbenchmark, S3).
4. **Threads for values (D1-9)**: deep copies inside tasks keep every count thread-local; the
   alternative is Koka's thread-shared bit (phase2-rc-elision idea 2) or Swift's always-atomic counts.

---

## Sources

- Phase 1: `tasks/96/redesign/phase1-borrowing.md`, `phase1-sharing.md` (the ids A1..A25, A31..A34
  and their eliminations).
- Phase 2: `tasks/96/redesign/phase2-hylo-csharp.md` (projections, yield-once accessors, paged storage
  +30% random access, `target/redesign/phase2-hylo-csharp/results.txt`), `phase2-swift-mojo.md`
  (copy on write, dynamic exclusivity costs, ASAP destruction), `phase2-rc-elision.md` (count pair
  0.84 ns, uniqueness test +10%, hidden copy 17.8 ms, objects vs inline 3.56x/9.74x/2.44x;
  `target/redesign/phase2-rc-elision/results.txt`), `phase2-vale-austral-pony.md` (compare cost
  0.26 ns / 2.80 ns; `target/redesign/phase2-vale-austral-pony/results.txt`).
- Spec: `docs/spec-source/Ember_v0.9.10_Hardened_35.md`, rules named inline.
- ECS: `tasks/95/ecs-entt-design.md`; ODR-286 in `tasks/95/odr-pending.md`; probes
  `target/analysis/ecs4/*.em`, `target/analysis/ecs5/*.em`.
- C models: `target/redesign/bench/D1/w1.c` .. `w4.c`, harness `target/redesign/bench/harness.h`.
- Scratch (not project files): reference programs `.../scratchpad/ref/ref1.c` .. `ref4.c` and the
  Python checks `.../scratchpad/py_d1.py`, under
  `C:/Users/ism19/AppData/Local/Temp/claude/C--Users-ism19-Code/ae3a2898-3f0f-4361-99e2-9bbbb0976123/scratchpad/`.

Tools: clang 22.1.8 and Python 3.14.2 were present and used. WebSearch and WebFetch were not needed.
Nothing was downloaded or installed; no Ember program was run.
