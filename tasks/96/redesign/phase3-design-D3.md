# Phase 3, design D3: prove it, or check it

Static borrowing with an automatic fallback, one kind of object reference, and containers that hold
still while they are used.

Written 2026-10-24 by the phase-3 D3 design agent. The design was built from the six phase 1 and
phase 2 reports in this folder and from the spec (`docs/spec-source/Ember_v0.9.10_Hardened_35.md`,
Parts VII to IX, and the `[CLO-*]`, `[CTL-*]` and `[TYP-15]` rules it touches), ODR-286 in
`tasks/95/odr-pending.md`, `tasks/95/ecs-entt-design.md`, and the probes in `target/analysis/ecs4`
and `ecs5`.

How to read the claims:

- Nothing in this design is implemented. Every statement about what the D3 compiler does is a design
  claim, not a fact about a compiler. The analyses it relies on (whole-program effect summaries and
  ownership facts) are marked where a result depends on them.
- The four C models in `target/redesign/bench/D3/` (`w1.c` to `w4.c`) were each built once with
  clang -O2 and run once to check that they finish. Their checksums were compared with four
  independent reference programs written another way (`target/redesign/bench/D3/check/*_ref.c`,
  arrays indexed by id, no D3 machinery); all four agree (section 5). **No timing was taken or is
  reported here.** Every speed expectation in section 6 is an estimate from the instructions in the
  models and is marked UNVERIFIED; the measurer checks them.
- Numbers quoted from phase 2 name the report and the file that holds the raw output.
- No Ember program was run. No tool was missing for this task.

---

## 0. The design in one page

**What the programmer sees: two kinds of type and one rule about names.**

1. **Values**: numbers, `struct`, `enum`, tuples, and the collections (`Array`, `Map`, `Set`,
   `String`). Stored inline, as in C.
2. **Objects**: `class` instances. An object has identity, and any number of names may refer to it.
   This one kind replaces today's class handle, `Box[T]`, `Box[dyn I]`, `Shared[T]`, `Weak[T]` and
   `Pool[T]` with `Handle[T]`.
3. **Names bind places.** `boss = enemies[0]` makes `boss` another name for that element, as in
   Python; `b = a` makes `b` another name for `a`'s value. A value stored into a container or a field
   moves there, and its name follows it. The programmer never writes `ref`, `ref mut`, `mut`,
   `owned`, `@borrows`, `Box`, `Weak`, `Cell`, `RefCell` or a lifetime.

**What the compiler does: prove, or else insert one of five cheap fallbacks, never refuse.**

The borrow checker stays: regions are inferred as today, field by field, now also through local
containers and with every function's effects summarised (what it reads, writes, adds to, removes
from, destroys). Where it proves a program safe, the generated C is plain C. Where Ember today says
E3020, E3022, E3040, E3060, E3063 or E3026, D3 inserts a fallback and records it:

| fallback | when the proof fails that... | what runs | cost |
|---|---|---|---|
| **PIN** | a container keeps still while a borrow into it is live | a counter in the container; while it is above zero, an add never moves an element and a removal leaves a tombstone, applied when the count returns to zero | 2 counter updates per pinned borrow (not per element); 1 test per add-when-full and per remove; while pinned, a tombstone test per element in a loop and a slow path for indexing past the contiguous part |
| **WATCH** | an object outlives every stored reference to it | the reference carries the object's generation; each use compares it | +4 bytes per stored reference (packed beside other fields) or +8; +4 bytes per object; 1 load and 1 compare per use, on the object's own cache line |
| **COPY** | a moved value is not used again / a view does not outlive its source | the value is copied where the move or view was | the copy (an allocation for a collection) |
| **CELL** | a closure that is stored or returned dies before a variable it captures by reference, which its function still uses | the variable moves to a small heap cell, counted | 1 allocation, 1 indirection per access, count updates when the closure is copied or dropped |
| **CAST** | a downcast always succeeds | the object's type index is compared | 1 load and 1 compare |

The user sees no refusal for any of these. `ember inspect --checks` lists every inserted fallback
with the two lines that caused it and its cost; `@static` on a function (or a manifest setting) turns
chosen fallbacks back into compile errors for code that must be provably free.

**What goes from today's model:** reference counting of objects (`[RC-*]`), the 24-byte object
header (`[OBJ-1]`), the per-field run-time access words and their panics (`[EXC-*]`), `Weak`
(`[WK-*]`), `Box`/`Shared`/`Cell`/`RefCell`/`Handle` as things the user chooses, and "aliasing xor
mutation" (`[BRW-1]`) as a rule the user meets.

**What stays refused:** sending to another thread something the sender can still reach, watching an
object inside a `Mutex` from outside the lock, raw pointers outside `unsafe`, generic methods in a
`dyn` interface under hot reload, a virtual call in `init` before every field is set, and the
coherence rules for library packages (section 1.6).

**Cost in the four workloads (estimates, section 6):** W2 compiles to the same pointer chase as C
with no check at all; W1 and W3 add a few predictable compares per element on loops that are memory
bound; W4 pays a pin and an unpin around each of 10 million indirect calls, the largest relative
overhead of the four.

---

## 1. The model in plain words

### 1.1 Values, and names that bind places

A value lives where it is declared: in a local, inline in a field, inline in a container's buffer.
No header, no count. A collection is a small header (`data`, `len`, `cap`: 24 bytes) and a heap
buffer that it alone owns, as today.

Three rules give values Python's meaning without moves the user can get wrong:

1. **`x = place` makes `x` another name for the place** when the value is not a scalar (a struct,
   enum, tuple or collection). `boss = enemies[0]; boss.hp -= 10` changes `enemies[0]`, as in
   Python; today it is `E3040` (phase 1 borrowing, A2 (3)). `b = a; a.push(3)` makes `b` see the 3,
   Python's meaning; today it is a move and `E3040` (A11). Scalars are copied (Python's integers are
   immutable, so nothing can observe the difference). An explicit copy is `copy(e)` or `e.clone()`.
2. **A name follows its value.** `ps.push(p)` moves `p`'s value into the array, and afterwards `p`
   names that element. If the place a name refers to is reassigned while the name is still used,
   the name keeps the old value (the compiler moves the old value into the name's own slot at that
   point; this is a static rewrite, with no run-time cost).
3. **A move the compiler cannot follow becomes a copy** (fallback COPY): `send(m); log(m)` where
   `send` keeps `m` somewhere the caller cannot see passes a copy to `send`. The copy is listed by
   `ember inspect --checks`, and refused in `@static` code.

Parameters are borrowed by default (the callee works on the caller's place, Python's meaning for
objects and for mutable values); whether a parameter or `self` is written, kept or consumed is
inferred from the body and recorded in the package's interface file, so no `mut` or `owned` is
written (they remain allowed, as checked documentation).

### 1.2 Objects: a home, and watchers

An object is created by its constructor (`Ship(...)`) on the runtime's object slab for its size
class. Exactly one place **owns** it, its **home**; every other reference to it **watches** it.

- **The home** is where the object is first stored: a local, then (a name follows its value) the
  container element, field, module variable or closure capture it is stored into. Storing a
  reference that is already watching makes another watcher. `xs.pop()` and `xs.remove(i)` hand the
  home to the receiving place.
- **An object is destroyed when its home lets it go**: the element is removed, the field is
  overwritten, the local's scope ends. Destruction runs its `drop`, then frees the slot to the slab's
  free list for its size class. If the compiler sees a watcher of it used later in the same function,
  destruction waits for that watcher's last use (a static rule, no run-time cost).
- **A watcher whose object may die first is checked.** The object keeps a 32-bit generation in its
  header; a stored watcher keeps the generation it saw. Destruction adds one to the object's
  generation, so every older watcher no longer matches. Using a watcher compares the two: a watcher
  typed `Option[C]` then reads `None` (the program asks `match obj.target:`), and one typed `C`
  panics, naming the line where the object was destroyed. Freed slots keep their generation word and
  are reused only for objects of the same size class; a slot whose generation wraps around is retired
  (as `[HND-3]` does today), so a stale watcher can never match a new object. This is the counter
  generation of Vale's original design (phase 2 vale report, section 1.1), not the random one, so
  detection is deterministic.
- **A watcher the compiler proves safe is a plain pointer.** If every home of a class never lets an
  object go while watchers of it can still be used (an append-only array that lives longer than all
  the watchers, a local that outlives them by scope), the watchers carry no generation and are never
  checked, and if nothing else needs it the class has no generation word at all. W2 is such a
  program: its nodes live in an array that is only appended to, so `parent` and `children` compile to
  raw C pointers (section 3, W2).
- **The header is at most 8 bytes**: the generation (only for a class with checked watchers) and a
  32-bit type index (only for a class that is open, used through an interface, or cast). A final
  class with no checked watchers has no header at all. Today's header is 24 bytes plus a 4-byte
  access word per non-`Copy` field (`[OBJ-1]`, `[EXC-19]`).
- **No reference counts.** Copying, passing and storing a watcher touches no count. Ownership
  cycles cannot be built through watchers; the only way to leak is two objects each stored *first*
  into the other (each the other's home), which the debug build still reports at exit (today's
  `[WK-15]` report, kept).

A slot that receives an owning reference on one path and a watching one on another is rare; it
carries a one-bit tag in the pointer's low bit, tested when the slot is dropped.

This one kind covers today's five: a class handle (shared identity), `Box[T]` (one owner: the home),
`Box[dyn I]` (a reference typed by an interface), `Weak[T]` (a watcher), and `Handle[T]` into a
`Pool[T]` (a watcher into objects whose home is an array).

### 1.3 Containers that hold still: pins

The hazard behind A2, A3, A4, A9 and A32 is always the same: a pointer into a container's buffer
lives while something adds to the container (the buffer moves) or removes from it (the element is
freed or another one slides into its place). D3 makes the container hold still, but only while it
must:

- **Pinned or not.** The compiler knows each borrow into a container: a loop over it, a name bound
  to an element, a returned element, the running closure stored in it. If it can prove that nothing
  adds to or removes from the container while the borrow lives, nothing is inserted. Otherwise the
  container is **pinned** for the borrow: a counter in the container goes up when the borrow starts
  and down at its last use. When the change happens in the same function as the borrow the compiler
  knows it statically and needs no counter (W3's destroy loop); when it happens through another path
  (a method of another object, a closure, a type-erased table) the counter is real and the change
  tests it (W1, W4).
- **While pinned**, an add goes in place while there is capacity, else into an overflow page, so no
  element moves; a removal marks the slot as a tombstone and is applied later. `len(xs)` is always
  the true length. A loop over a pinned container visits the elements that existed when it began,
  once each, and skips tombstones: an element appended during the loop is not visited, an element
  removed before its turn is skipped. Indexing compares the index with the contiguous part's length
  (the same one compare as a bounds check); an index past it, or into a list with removals pending,
  takes a slow path.
- **When the count returns to zero the container settles**: overflow pages are merged into one
  buffer, tombstones are removed (in order for an `Array`; swap-and-pop for an ECS pool, EnTT's own
  order, which keeps settling proportional to the removals), and the buffer is contiguous again.
  Outside a pin, an `Array` is exactly today's contiguous buffer, so iteration, indexing, slicing,
  SIMD and FFI are unchanged. `Map` and `Set` follow the same protocol: while pinned, an insert that
  would rehash goes into a small overflow table searched after the main one (a flag tested on a
  miss), and a removal leaves a tombstone.
- **Replacing a pinned container** (`h.items = []` while a loop over `h.items` runs) keeps the old
  buffer alive in the pin's side record until the pin ends: the loop finishes over the old contents,
  as a Python loop does over the old list object.
- **Destroying the object that holds a pinned container** (its home lets it go during a loop over
  one of its fields) bumps its generation at once, so every watcher sees it gone, but frees its
  memory and its fields only when the pin ends. A destroy reads the pin counters of the class's
  pinnable fields: a few loads, only for classes that have such fields.
- **Layout.** Only container places that may be pinned through another path need the counter: their
  header grows from 24 to 40 bytes (the counter and a pointer to a side record allocated at the first
  deferred change), or to 56 for an `Array`, which also keeps its logical length and a fast index
  bound so that `len()` and indexing stay one load and one compare while removals are pending. The
  compiler decides this per field from the whole program; the extra fields come after `data`, `len`
  and `cap`, so read-only code works on both layouts and only code that adds or removes is
  instantiated twice. In W2 the children arrays are never pinned and keep the 24-byte header.

Why not pages everywhere (EnTT's layout, the phase 2 Hylo/C# proposal)? The Hylo/C# agent measured
random access by index through a page table at 1.30x a contiguous array (`target/redesign/
phase2-hylo-csharp/results.txt`, 16,777,216 random reads over 64 MiB), and in-order sums at 1.25x
with 64 KiB pages. Pinning pays nothing for arrays that are not pinned, and a pinned array only
leaves its contiguous buffer when it overflows during a pin. Why not copy-on-write (Swift, Roc)? The
rc-elision agent measured the is-unique test at +10% on random writes and a hidden full copy when a
buffer is shared (`target/redesign/phase2-rc-elision/results.txt`, `unique_set`, `cow_first_set`), and
a loop that writes through its own elements would write into a copy (the rc-elision report's own
caveat on snapshot iteration).

### 1.4 Closures

A closure captures by reference what it reads and writes, as Python does. It may be stored anywhere
(no `owned fn`, no `E3026`):

- **If the compiler proves the closure dies before what it captures**, the capture is a plain
  pointer. Local containers and objects carry the regions of what they hold, so a closure stored in a
  dispatcher that is a later local of the same function is proven (W4, and phase 1's A12 program).
- **A captured variable that only the closure uses afterwards moves into it** (W4's `counter`).
- **Otherwise the variable moves to a counted heap cell** shared by the function and the closure
  (fallback CELL; phase 1 sharing A12 (a)). This is the one reference count left in D3.

An owned callable value keeps today's representation: a function pointer and up to three words of
state inline (`[CLO-10]`).

### 1.5 Compile time, run time, and what each costs

Checked at compile time (no annotation anywhere):

- types and definite assignment, as today;
- every borrow's region, field by field and through local containers (today's `[LT-*]` inference,
  with `[TYP-15]` narrowed);
- each function's **effect summary**: which fields it reads and writes, which containers (by type and
  field path) it may add to or remove from, which classes' objects it may destroy, which parameters it
  writes, keeps or consumes. Summaries are recorded in the package interface; dynamic dispatch, a
  function value or a foreign call is assumed to have every effect its types allow (the cost of that
  pessimism is a pin, never an error);
- homes and watchers: which slots own and which watch, and whether each watcher can outlive its
  object;
- threads: what crosses a thread boundary is moved or frozen (section 1.7).

Checked at run time:

- bounds, as today (`[TYP-31]`, `[SPN-2]`);
- WATCH: a generation compare at each use of a watcher the compiler could not prove;
- CAST: a type-index compare for a downcast the compiler could not prove (`cast x as T` panics,
  `cast x if T` gives `None`, as ODR-280);
- PIN: bookkeeping that cannot fail (a counter, tombstones, overflow pages, a settle);
- integer overflow, as today (`[TYP-8]`; not part of the memory model).

The run-time failures left are deterministic panics: an index out of bounds, a non-`Option` watcher
whose object was destroyed, a failed `cast as`. None of them can read or write freed memory.

| | today (spec) | D3 |
|---|---|---|
| object header | 24 bytes (`[OBJ-1]`) | 0, 4 or 8 bytes, per class |
| per non-`Copy` field of a class | a 4-byte access word (`[EXC-19]`) | none |
| copying or dropping an object reference | retain / release (`[RC-1]`, `[RC-4]`) | nothing |
| stored back pointer | `Weak`: weak count, `upgrade` per use (`[WK-12]`) | a watcher: plain pointer if proven, else +4 bytes and a compare per use |
| a loop over a container that its body changes | `E3020` | the loop pins the container (counter only through another path) |
| an element held across a push | `E3022` | a pin |
| a long-term access to a class field | load, compare, maybe store, per access (`[EXC-19]`) | nothing (no exclusivity) |
| use after move | `E3040` | an alias, a followed move, or a reported copy |

For a sense of scale, the phase 2 measurements of the two run-time mechanisms D3 borrows (neither
measured in D3's exact layout): a generation-checked reference cost +0.57 ns per in-order use and
+4.61 ns per random use over a plain pointer, with 16-byte references (`target/redesign/
phase2-vale-austral-pony/results.txt`, the vale report's section 6); a non-atomic count increment and
decrement pair, which D3 removes, cost about 0.84 ns per element visit (`target/redesign/
phase2-rc-elision/results.txt`, the rc-elision report's section 2.3). D3's watcher is 12 bytes (an
8-byte pointer and a 4-byte generation packed into a neighbouring gap) where Vale's is 16, and its
generation sits on the object's first cache line, the one a use reads anyway.

### 1.6 What stays refused

1. **Threads**: sending a value or object graph that the sender can still reach (section 1.7).
2. **A watcher into a `Mutex`'s data from outside the lock** (A33): what leaves a lock is a copy or
   an object moved out.
3. **Raw pointers outside `unsafe`** (A20).
4. **Generic methods in a `dyn` interface under hot reload** (`E2050`, Annex B builds only; A13).
5. **A virtual call in `init` before every field is set** (A18, narrow).
6. **Coherence rules for library packages** (A16; not a memory-model rule).
7. **Opt-in**: a function marked `@static`, or a package whose manifest denies a fallback kind, gets a
   compile error, with the check report's explanation, wherever that fallback would be inserted.

### 1.7 Threads

Pins, generations and the aliasing of plain data are single-thread facts: their counters and
compares are plain memory operations. So D3 keeps one compile-time rule for threads, spelled with no
`Send` or `Sync` (phase 2 vale report 5.5, "move it or freeze it"; phase 2 Swift report idea 5,
region-based isolation):

- a value or object graph given to another thread must be **disconnected** (nothing the sender keeps
  can reach it, inferred) and is moved, or **frozen** (read-only for every thread from then on, a
  type change with no copy);
- scoped tasks (`thread.scope()`, `[THR-5]`) may borrow the parent's data read-only; `+=` on a
  captured number from scoped tasks is a per-task accumulator joined at the end (phase 1 sharing A15
  (a), today's `@parallel(reduce=…)` mechanism);
- shared mutable state across threads lives in `Mutex[T]` or `Atomic[T]`, and a `Mutex` owns its
  data: nothing inside it is watched from outside;
- frozen data shared by unstructured threads is freed by an atomic count (the only atomic counting
  in D3); frozen data in a scope needs none.

Where the disconnection cannot be proven, the send is refused with fix-its (copy, freeze, a
`Mutex`). That refusal is kept (A15).

### 1.8 The check report

```text
$ ember inspect --checks w1.em
w1.em:24  PIN    Pool[Position].dense/.entities, Pool[Velocity].dense/.entities  (counter: reached through reg.pools)
          why    the loop on line 24 holds `pos`, `vel` and its cursor while line 28 `reg.destroy(e)`
                 may remove from and line 30 `reg.emplace(...)` may add to these pools
          cost   per loop: 4 counter updates; per element: 1 tombstone test; per destroy/emplace:
                 1 counter test; at the loop's end: a settle proportional to the removals
w1.em:24  BOUNDS reg.view: sparse lookup, 1 compare per element (index read from a pool)
```

`ember inspect --checks --profile` builds an instrumented program that counts how often each check
ran. Nothing in the report changes what the program means.

### 1.9 What changes in the spec

| rules | today | D3 |
|---|---|---|
| `[OWN-1]`–`[OWN-6]` | one owner; a move empties the source; `E3040`, `E3041` | one owner per value stays; `x = place` aliases; a name follows its value; an unfollowable move is a COPY (reported); `E3040`/`E3041` go |
| `[OWN-7]`, `[OWN-8]` | copying a handle retains | copying an object reference does nothing |
| `[BRW-1]` | aliasing xor mutation | replaced: no use of a borrow into storage that a change may have moved or freed, guaranteed by proof, PIN or WATCH; aliasing plain data is allowed |
| `[BRW-3]`, `[BRW-5]`, `[BRW-11]` | two-phase borrows; indices not disjoint; call borrows live together | analysis facts for `noalias` proofs only, never errors |
| `[BRW-4]`, `[BRW-10]` | disjoint fields; private methods' field sets | kept; field sets inferred for every method, `pub` included, recorded in the interface |
| `[LT-1]`, `[LT-1a]`, `[LT-1b]` | signature elision, `@borrows` | result provenance inferred from every body (as `[LT-22]` does for views); `@borrows` optional |
| `[TYP-15]`, `[TYP-15a]` | views stored only in bounded places | local containers and objects are bounded by their own region; an unbounded store is a COPY (views of values) or a watcher (objects) |
| `[CTL-2]` | mutating the iterated place is `E3020` | the loop pins the place |
| `[EXC-1]`–`[EXC-19]` | per-field run-time exclusivity | removed; replaced by new `[PIN-*]` rules |
| `[RC-1]`–`[RC-6]` | counted handles and their elision | removed; replaced by new `[HOME-*]` and `[WATCH-*]` rules |
| `[OBJ-1]`–`[OBJ-5]` | 24-byte header, runtime allocator | 0–8-byte header per class; objects on size-class slabs whose slots keep their generation |
| `[WK-*]` | `Weak`, cycle warnings | `Weak` removed (watchers); the exit leak report kept for mutual homes |
| `[HEAP-1]`–`[HEAP-10]`, `[HND-*]`, `[CELL-*]` | `Box`, `Shared`, `SyncShared`, `Weak`, `Pool`/`Handle`, `Cell`, `RefCell` | removed from the user's surface; `SyncShared` becomes frozen objects; growable buffers gain the pinned overflow; `[HEAP-2]` doubling unchanged |
| `[SEL-1]` | eleven mechanisms | struct/enum/tuple, collections, class, `Arena` (bulk lifetime), `Mutex`/`RwLock`/`Atomic` (threads) |
| `[CLO-2]`, `[CLO-3]`, `[CLO-13]`, `[CLO-15]` | `owned fn`, copy-capture rules | capture by reference with a region proof; CELL fallback; `owned fn` optional |
| new | | `[PIN-*]`, `[HOME-*]`, `[WATCH-*]`, `[CHK-*]` (the report and `@static`) |

---

## 2. The annoyances, one by one

Verdicts:

- **removed**: no refusal and no annotation is left; any run-time cost is bookkeeping that cannot
  fail (a pin, a copy, a cell) or nothing. The kind says which of the owner's two ways removes it:
  **(a)** a more Pythonic spelling of the same guarantee, **(b)** a smarter mechanism that makes the
  hazard impossible or harmless.
- **moved to run time**: no refusal, but where the static proof fails a deterministic panic (or a
  `None`) can now happen at run time.
- **kept**: a refusal, annotation or ceremony is left in some case, named.

"Phase 1 (5)" names the elimination in that catalogue's part (5) that D3 uses.

| id | verdict | how in D3 | phase 1 (5) used | cost where the proof fails |
|---|---|---|---|---|
| A1 whole-object borrow | **removed, (a) + (b)** | every method's field set is inferred (`pub` and class methods included) and recorded in the interface, so `self.note(…)` inside `for p in self.players` touches only `log` and `frame`; there is no whole-object access any more (`mut self` no longer locks every field, `[EXC-15]` goes). Where field sets overlap (a virtual call is "all fields"), the loop pins `players` instead of refusing | borrowing A1 (a) field sets; A1 (b)'s aim (the loop cannot be invalidated), reached by pinning rather than by index paths, so unpinned loops keep pointer iteration | zero in the phase 1 program; a pin per loop otherwise |
| A2 container invalidation | **removed, (b), spelled (a)** | `boss = enemies[0]` names the element (no move, no `ref mut`); `enemies.push(…)` while `boss` lives goes in place or into an overflow page, so `boss` stays valid; removing it while `boss` lives leaves a tombstone, and `boss` keeps the removed value until its last use (Python's: you still hold the object you removed) | borrowing A2 (a) for the spelling; A2 (b) storage that does not move, applied only while pinned | 2 counter updates per borrow; 1 test per add-when-full or remove; none when the proof succeeds |
| A3 two mutable references into one container | **removed, (b)** | no exclusivity rule: `collide(bodies[i], bodies[j])` compiles, `i == j` included. Plain data aliasing cannot free memory once containers hold still; a view into a child container (`a.name`'s characters) across a write through a possible alias (`b.name = …`) is protected by pinning that child | borrowing A3 (b) | no check; `noalias` is given only with a proof, as `[LT-27]` already says today, so nothing is lost |
| A4 changing a collection while iterating | **removed, (b)** | `for e in enemies:` with `enemies.push(…)` and `enemies.remove(i)` in the body compiles: the loop pins `enemies`, visits the elements present when it began once each, skips removed ones, and the list settles at the loop's end; `remove(i)` of the loop's own index is O(1) | borrowing A4 (b) tombstones and stable storage, A4 (a)'s "applied after the loop" for removals | zero for loops that do not change their container; a tombstone test per element and a settle proportional to the removals (unordered) or to the tail after the first removal (ordered) |
| A5 returning references | **removed, (a) + (b)** | a function whose return expression is a place returns that place; provenance is inferred from every body and recorded in the interface (no `@borrows`, no `E3062`). ODR-286's `return pool.dense[i]` (today `E3060`) compiles: the pool is an object whose home (the registry's table) never lets it go while the registry lives, a whole-program fact, and its elements hold still under a pin while the caller's alias lives | borrowing A5 (a) inferred provenance; A5 (b) the referent cannot go away | a pin while the returned alias is live across a change; nothing otherwise |
| A6 structs holding references | **removed, (a)**; the unbounded case falls back | a struct holding borrows needs no lifetime (as today) and may now be put in a local container, which carries its region (`handles: Array[Handle] = []`); stored beyond any provable region, a borrow of an object becomes a watcher and a view of a value becomes an owned copy | borrowing A6 (a); A6 (b) with a generation instead of a count | WATCH (a compare per use) or COPY, reported |
| A7 self-referential structures | **removed, (a)** | `Document(text=…, words=text.split_whitespace())` compiles: the views point into `text`'s heap buffer, which does not move with the struct; if `text` is reassigned while `words` lives, the old buffer stays in the pin's side record until the views are gone | borrowing A7 (a) (the views' source field is frozen while they live, inferred) | zero; the memory of a replaced buffer is held while views of it live |
| A8 graphs, parent pointers, cycles | **moved to run time** where unprovable; free where provable | the container or field an object is first stored in is its home; `parent` and `children` elements are watchers. No ownership cycle forms through watchers, so nothing leaks and nothing needs `Weak`. If the homes never let an object go while watchers live (W2's append-only array), every watcher is a raw pointer and the class has no header; otherwise each use of a watcher compares a generation, and a dead one reads `None` or panics | borrowing A8 (b) (arena-owned graph, here inferred from the homes) and sharing A9 (b)'s second option (generational references), as the fallback | proven: zero (W2); else +4 bytes per object, +4 or 8 per watcher, a compare per use |
| A9 shared mutable state | **moved to run time** (object death); the borrow panics are **removed (b)** | two systems hold and write one object freely: no `RefCell`, no access words, no "already borrowed" panic. Walking `hud.target.inventory` while another reference pushes to it pins the list (the push lands in place or in an overflow page, the loop does not visit it). The only run-time failure left is using an object after its home destroyed it | sharing A9 (a) one spelling (`class`); A9 (b) a loop that cannot be invalidated, by pinning | PIN through another path (a real counter); WATCH for stored references |
| A10 choosing among ownership kinds | **removed, (a) + (b)** | two declarations, `struct` and `class`; `Box`, `Shared`, `SyncShared`, `Weak`, `Cell`, `RefCell`, `Pool`/`Handle` are not chosen by the user. Owning and watching are chosen per slot by the compiler; `Mutex`/`Atomic` remain for threads only | sharing A10 (a) two words; A10 (b) the compiler picks the representation, here only owning vs watching and the header's words, reported by `ember inspect --owners` | ≤ 8-byte header; no counts |
| A11 moves, clone everywhere | **removed, (a) + (b)** | `b = a` aliases (Python's meaning, not a move); `ps.push(p)` moves `p` there and `p` follows it; a move the compiler cannot follow becomes a copy at the earlier use, reported | borrowing A11 (b) move at last use, copy elsewhere, reported; A11 (a)'s copy-on-write was **not** taken (its +10% write cost, rc-elision `unique_set`) | a copy, only where a value is really used in two places; `@static` refuses it |
| A12 closures capturing mutable state | **removed, (a)** | `b.on_click(fn(): score += 1)` compiles and changes `score`; no `owned fn`, no `E3026`, no silent copy. Proven: a plain pointer capture; a variable used only by the escaping closure moves into it; otherwise a counted cell | sharing A12 (a) (the escaping variable moves to the heap), only where the proof fails | proven: zero; else 1 allocation and 1 indirection per access |
| A13 trait objects, downcasting | **removed, (a) + (b)** in whole-program builds; `E2050` **kept** under hot reload | `Box[dyn I]` is an object typed by an interface (a struct stored where an interface type is wanted is placed in an object of its own, today's `Box`, inferred); `cast x as T` works on any object or interface reference (the header's type index) and a cast of a place is a place (phase 2 Hylo H2). The ECS registry indexes its pools by the program's component type number, so the compiler knows each pool's type and emits no compare. A `dyn` interface with a generic method gets one table slot per instantiation the program uses | sharing A13 (a) and (b) | CAST: a compare where unproven |
| A14 async (`Pin`, `Send`, borrows across `await`) | **removed, (b)** for `Pin` and self-borrows; `Send` as A15 | a task's frame is allocated once and never moves, so a borrow of its own locals across `await` or `yield` is an ordinary borrow (today's `E2221`); a loop over a class field across a `yield` holds a pin while suspended (today's `E2229`) | sharing A14 (a) frames that never move | 1 allocation per spawned task; pins as anywhere |
| A15 threads (`Send`/`Sync`, sharing, scoped threads) | **kept** where disconnection cannot be proven; the annotations are **removed, (a)** | no `Send`/`Sync`/`@sync` written: moving or freezing is inferred; scoped tasks borrow; `+=` from tasks is a reduction; shared mutation is `Mutex`/`Atomic`. An unprovable send is refused with fix-its | sharing A15 (a); A15 (b) (moving an isolated graph) as the inference itself | zero; atomic counts only for frozen data outliving a scope |
| A16 generics friction | **kept** in part (not a memory-model rule) | D3 adopts sharing A16 (a): a leaf package may implement any interface for any type, and a private generic function's bounds are inferred. The orphan rule for libraries and the lack of specialisation stay | sharing A16 (a) | none |
| A17 error messages | **removed, (b)**, for the borrow errors | the refusals become fallbacks, so the messages that did not say what to do no longer occur; for each fallback the report names the two lines responsible; the few refusals left come with a fix-it re-checked by the compiler before it is shown | borrowing A17 (a) checked fix-its, (b) remove the refusals | compile time only |
| A18 initialisation | **removed, (a)** for direct calls; **kept** for a virtual call in `init` | `self.assets = self.load_assets()` before every field is set compiles when `load_assets` (non-virtual) reads only fields already set, using its inferred field set | borrowing A18 (a) | none |
| A19 global mutable state | **removed, (a)** on one thread; **kept** across threads | `var FRAME = 0` and `var LOG: Array[String] = []` at module level, touched freely by main-thread code (inferred effect); a borrow into `LOG` across a call that may push pins it; code reachable from a thread that touches a `var` is refused (use an `Atomic` or `Mutex` static) | sharing A19 (a) | pins where needed |
| A20 unsafe code and FFI | **removed, (b)** for callbacks; **kept** for raw pointers | the user-data contract is inferred from the header (sharing A20 (a)); a callback C holds is an entry in a runtime table reached through a watcher, so a call after unregistering finds nothing instead of freed memory (A20 (b)); a raw pointer turned into a reference stays `unsafe` | sharing A20 (a), (b) | 1 table load and 1 compare per callback |
| A21 updating a map entry in place | **removed, (a)** | `counts[w] += 1` on a `Map(default=0)`, `get_or(k, d)`, and `counts.entry(w).or_default() += 1` (the alias of a place returned by a call) | borrowing A21 (a) | none |
| A22 a view of a temporary kept past its statement | **removed, (a)** | `word = line.to_lower().trim()` keeps the temporary until `word`'s last use; if the view escapes the function, it becomes an owned copy | borrowing A22 (a); (b) not taken | COPY when it escapes |
| A23 moving a field out from behind `mut self` | **removed, (b)** | `match self.state:` binds `path` to the `Walking` payload; reassigning `self.state` in the arm leaves `path` holding the old value (a name keeps the old value, 1.1 rule 2), and since there is no unwinding (`[PAN-1]`) no caller can see the field half-moved: definite assignment is the only check | borrowing A23 (b) | none |
| A24 struct, `Box` or class changes what compiles | **removed, (b)** | one set of rules for every kind: pins for containers, homes and watchers for objects. The kind changes only the representation (inline value or object), never what compiles or what panics. Both ODR-286 probes compile and behave the same with pools as objects | borrowing A24 (b) | per representation |
| A25 a collection of views refused even locally | **removed, (a)** | `words = lower.split_whitespace().collect()` into a local `Array[str]` compiles: the array carries the region of `lower`; returned or stored beyond it, the words become owned copies, reported | borrowing A25 (a); (b) shared immutable strings not taken (a count per buffer) | COPY when it escapes |
| A31 caching inside a read-only method | **removed, (a)** | a method's writes to `self` are inferred, so `radius(self)` that fills `self.radius_cache` is a writing method with no `mut`; replacing a non-`Copy` cache while a reader views it pins the cache | sharing A31 (a) (generalised: mutation is inferred, not declared) | none |
| A32 callbacks that receive the object holding them | **removed, (b)** | `for h in self.handlers: h(self)` compiles; the call pins `handlers`, so a handler that adds or removes handlers (itself included) cannot move or free the running one; changes apply when the call returns | sharing A32 (b) (deferred changes), by pinning | per call: a pin, an unpin and a test (W4) |
| A33 lock ceremony and references out of a guard | **removed, (a)**, and today's soundness hole is closed; **kept**: watching inside a lock from outside | a borrow through a guard has the guard's region; `return g[0]` out of a `with g = scores.lock():` returns a copy (reported), an object leaves only by being moved out | sharing A33 (a) | a copy |
| A34 the clone-before-move dance | **removed, (a)** | object references copy freely with no count; to another thread, A15's move-or-freeze | sharing A34 (Ember already had it) | none |

Totals: removed 21 (A1–A7, A10–A14, A17, A21–A25, A31, A32, A34), moved to run time 2 (A8, A9),
kept in part 6 (A15, A16, A18, A19, A20, A33); A13 is removed in whole-program builds and keeps
`E2050` under hot reload.

---

## 3. The programs, written in D3

Ember's syntax stays where D3 does not change it (`fn`, `class`, `struct`, `Array`, `push`, `0..n`,
`match` with `Some(x):` arms, `cast … as`, no truthiness). What is gone: `ref`, `ref mut`, `mut`,
`owned`, `owned fn`, `@borrows`, `Box`, `Weak`, `Shared`, `Cell`, `RefCell`, `Handle`. `h_seed` and
`h_rand` stand for the harness's generator (`target/redesign/bench/harness.h`).

### W1. ECS

```ember
from std.ecs import Registry

struct Position:
    x: f32
    y: f32

struct Velocity:
    dx: f32
    dy: f32

fn main():
    reg = Registry(reuse_ids=false)          # ids 0, 1, 2, … in order, never given out twice
    for i in 0..1_000_000:
        e = reg.create()
        reg.emplace(e, Position(x=(i % 1000) as f32, y=(i // 1000) as f32))
        if i % 2 == 0:
            reg.emplace(e, Velocity(dx=(i % 7) as f32 * 0.5, dy=(i % 5) as f32 * 0.25))

    for s in 0..100:
        for e, pos, vel in reg.view[(Position, Velocity)]():
            pos.x += vel.dx
            pos.y += vel.dy
            if e.id() % 100 == s:
                reg.destroy(e)                # today: E3020/E3022; D3: the loop pins both pools
                n = reg.create()
                reg.emplace(n, Position(x=0.0, y=0.0))
                reg.emplace(n, Velocity(dx=1.0, dy=1.0))

    total: u64 = 0
    for e, pos in reg.view[(Position,)]():
        total += ((pos.x * 4.0) as i64 + (pos.y * 4.0) as i64) as u64
    total += len(reg.storage[Position]()) as u64
    println(total)
```

What D3 decides: the view's loop borrows both pools (its cursor, `pos`, `vel`) and its body calls
`destroy` and `emplace`, which may remove from and add to those pools; they reach the pools through
the registry's type-erased table, so the pin is a run-time counter. Destroying the current entity
after its movement leaves tombstones; the new entity is appended past the loop's starting length,
so it is not visited this step. `reuse_ids=false` is a `std.ecs` knob, not a memory-model matter:
the workload asks for "the next unused id", and EnTT's recycling would hand out an old index. With
ids never reused, a stale entity id always finds an empty sparse slot, so no version bits are needed.

### W2. A tree with parent pointers

```ember
from bench import h_seed, h_rand

class Node:
    value: i32
    parent: Option[Node] = None          # a watcher
    children: Array[Node] = []           # watchers

fn main():
    h_seed(42)
    N = 1_000_000
    nodes: Array[Node] = [Node(value=0)] # the home of every node
    for i in 1..N:
        p = nodes[h_rand() as int % i]
        n = Node(value=(i % 1000) as i32, parent=Some(p))
        nodes.push(n)                    # the home moves into `nodes`; `n` follows it
        p.children.push(n)               # a watcher

    depth_sum = 0
    for pass in 0..10:
        for n in nodes:
            d = n
            while d.parent is not None:
                depth_sum += 1
                d = d.parent.unwrap()
        for i in 1..N:
            if (i + pass) % 100 == 0:
                new_parent = h_rand() as int % i
                node = nodes[i]
                old = node.parent.unwrap()
                old.children.remove(old.children.index_of(node).unwrap())
                node.parent = Some(nodes[new_parent])
                nodes[new_parent].children.push(node)

    total = depth_sum
    for n in nodes:
        total += n.value as int * len(n.children)
    println(total as u64)
```

What D3 decides, with the whole program in view: every `Node` is first stored in `nodes`; `nodes` is
only ever appended to and lives until `main` ends; every other `Node` reference is in a `Node` or in
a local of `main` inside `nodes`' scope. No watcher can outlive its node, so `parent` and the
`children` elements are raw pointers with no generation and no check. `Node` is final and never cast,
so it has no header. Today this program needs `Weak[Node]` for `parent` (with an `upgrade` per step of
the walk) and counts every node (`[RC-1]`); the plain-struct version crashes today's checker (phase 1
borrowing, defect 1).

This result depends on the whole-program ownership analysis. Without it (separate compilation that
cannot see that `nodes` is append-only, or a hot-reload build), `Node` gets a 4-byte generation, each
watcher a 4-byte generation beside its pointer, and each step of the walk one compare (section 6).

### W3. Game objects that refer to each other

```ember
from bench import h_seed, h_rand

open class GameObject:
    id: int
    x: i32
    y: i32
    health: i32
    target: Option[GameObject] = None    # a watcher: reads None once its object is destroyed

    virtual fn update(self):
        pass

class Ship(GameObject):
    override fn update(self):
        match self.target:
            Some(t):
                self.x += sign(t.x - self.x)
                self.y += sign(t.y - self.y)
            None:
                pass

class Rock(GameObject):
    override fn update(self):
        self.x = (self.x + 1) % 1000

class Turret(GameObject):
    override fn update(self):
        match self.target:
            Some(t):
                if abs(t.x - self.x) + abs(t.y - self.y) < 300:
                    t.health -= 1
            None:
                pass

fn make(id: int, x: i32, y: i32, target: Option[GameObject]) -> GameObject:
    h = (100 + id % 50) as i32
    match id % 3:
        0: return Ship(id=id, x=x, y=y, health=h, target=target)
        1: return Rock(id=id, x=x, y=y, health=h, target=target)
        _: return Turret(id=id, x=x, y=y, health=h, target=target)

fn main():
    h_seed(7)
    M = 100_000
    objects: Array[GameObject] = []      # the home of every object
    for i in 0..M:
        x = (h_rand() % 1000) as i32
        y = (h_rand() % 1000) as i32
        target = None if i == 0 else Some(objects[h_rand() as int % i])
        objects.push(make(i, x, y, target))

    next_id = M
    destroyed_total = 0
    for frame in 0..1000:
        for o in objects:
            o.update()
        destroyed: Array[int] = []
        for i, o in enumerate(objects):
            if o.health <= 0:
                destroyed.push(o.id)
                objects.remove(i)        # the loop's own index: a tombstone, applied at the loop's end
        destroyed_total += len(destroyed)
        for _ in destroyed:              # in increasing destroyed id
            id = next_id
            next_id += 1
            x = (h_rand() % 1000) as i32
            y = (h_rand() % 1000) as i32
            target = Some(objects[h_rand() as int % len(objects)]) if not objects.is_empty() else None
            objects.push(make(id, x, y, target))

    total = 0
    for o in objects:
        total += o.x as int + o.y as int + o.health as int
    println((total + 1_000_000 * destroyed_total) as u64)
```

What D3 decides: `objects` is the home; `objects.remove(i)` destroys the object (it is not used
again), which bumps its generation; every `target` that watched it now reads `None`, with no code in
the program to clear it. `target` is checked at each use because `objects` does let objects go. The
update loop pins nothing: no `update` can add to or remove from `objects` (the effect summaries of
the three overrides). The removal loop's pin is static (the change is in the same function as the
loop), so it needs no counter. `enumerate` counts the elements it yields, so `i` is the element's
current index even after earlier removals in the same loop; the compiler sees that `i` is the loop's
own index and removes the element under the cursor, with no search. In Python, `objects.remove(i)`
inside `enumerate` would skip the next object; D3's deferral makes the written program correct.

### W4. Callbacks with captured mutable state

```ember
class Dispatcher:
    handlers: Array[fn(Dispatcher, int, int) -> int] = []
    current: int = 0

    fn register(self, h: fn(Dispatcher, int, int) -> int):
        self.handlers.push(h)            # `h` is kept: inferred, no `owned`

    fn unregister_current(self):
        self.handlers.remove(self.current)

    fn dispatch(self, k: int):
        self.current = k % len(self.handlers)
        self.handlers[self.current](self, k, k % 1000)

fn main():
    total = 0

    fn make_handler(m: int) -> fn(Dispatcher, int, int) -> int:
        counter = 0
        fn handler(d: Dispatcher, k: int, v: int) -> int:
            if v >= 0:                   # v < 0 asks the handler for its counter
                counter += v * m
                total += 1
                if k % 100_000 == 99_999:
                    d.unregister_current()                # itself, from inside its own call
                    d.register(make_handler(k % 17 + 1))
            return counter
        return handler

    d = Dispatcher()
    for i in 0..10_000:
        d.register(make_handler(i % 13 + 1))
    for k in 0..10_000_000:
        d.dispatch(k)

    s = 0
    for h in d.handlers:
        s += h(d, 0, -1)
    println((total + s) as u64)
```

What D3 decides: `counter` is used only by `handler` after it is made, so it moves into the closure;
`m` is copied; `total` is captured by reference as a plain pointer, because every closure ends up in
`d.handlers` and `d` is a later local of `main`, so it dies before `total` (proven; no cell). The
state is three words, inline in the array element (`[CLO-10]`). `dispatch` calls a closure stored in
`self.handlers` that may change `self.handlers`, so the call pins the array; `unregister_current`
and `register` run in methods also called unpinned, so the pin is a real counter. The running
handler's removal is a tombstone: its state (and `total`'s pointer in it) stays valid until it
returns. Today this program is `E3026` (the lambda stored), and its `for h in self.handlers: h(self)`
shape is `E3080` (phase 1 sharing, A12 and A32).

The `v >= 0` test exists because the checksum needs each remaining handler's counter, which only the
closure can read; it is one predictable compare per call and is in the C model.

### The EnTT registry's `get` and `view` (std.ecs in D3)

```ember
interface AnyPool:
    fn remove(self, e: Entity)

class Pool[T](AnyPool):              # dense, entities: Array fields the program pins, so 40-byte headers
    dense: Array[T] = []
    entities: Array[Entity] = []     # a slot holding Entity.NIL is a tombstone (the pin protocol)
    sparse: Array[u32] = []          # never pinned: a plain 24-byte header

    fn index_of(self, e: Entity) -> int:
        j = self.sparse[e.id()]                          # bounds-checked
        if j == NIL:
            panic(f"entity {e.id()} has no {type_name[T]()}")
        return j as int

    fn remove(self, e: Entity): ...  # pinned: tombstone; else swap-and-pop (EnTT)

class Registry:
    pools: Array[AnyPool]            # one slot per component type of the program, by type number
    next_id: u32 = 0

    fn storage[T](self) -> Pool[T]:
        return cast self.pools[type_number[T]()] as Pool[T]   # proven by the compiler: no compare

    fn get[T](self, e: Entity) -> T:
        pool = self.storage[T]()
        return pool.dense[pool.index_of(e)]                # a place: today E3060 (ODR-286)

    fn view[Q](self) -> View[Q]: ...                       # Q = (A, B): leads with the smaller pool
```

A user of `get`:

```ember
reg.get[Position](e).x += 1.0          # one statement: nothing to pin

p = reg.get[Position](e)               # p names the component
reg.emplace(other, Position(x=0.0, y=0.0))
p.x += 1.0                             # today E3025/E3060; D3 pins Pool[Position] while p lives
```

And of `view`: the W1 loop. ODR-286's three walls are gone: the cast of a type-erased pool is a
place, the returned element is a place whose pool is alive while the registry is, and adding to a
pool inside a view loop is what pinning is for. EnTT's paged storage is not needed for this (it
appears only as the overflow page of a pool that fills up while pinned).

### The three worst programs of phase 1

Phase 1 rated three annoyances **high** in severity and **worse** in Ember than in Rust: A12, A24
and A25 (the summary tables of `phase1-sharing.md` and `phase1-borrowing.md`).

**A12, a button handler that bumps a score** (`target/redesign/phase1/sharing/a12_closures.em`;
today `E3026`, and the `owned fn` the help suggests returns 0):

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
    return score                         # 1, as in Python
```

The lambda captures `score` by reference; `b` is a later local, so the closure dies first: a plain
pointer. If `b` were returned or stored in a module variable, `score` would move to a counted cell
and `return score` would read it, still 1. The click loop pins nothing: the only closure the program
stores in `Button.handlers` does not touch `b`.

**A24, the ODR-286 pool probes** (`target/analysis/ecs5/pos.em`, today `E3060`, and
`target/analysis/ecs4/accept_change_during_loop.em`, today accepted by the checker but, by
`[EXC-1]`, a run-time exclusivity panic):

```ember
from std.collections import Map

interface AnyPool:
    fn count(self) -> int

class Pool[T](AnyPool):
    dense: Array[T] = []
    entities: Array[int] = []
    fn count(self) -> int:
        return len(self.dense)

struct Pos:
    x: int

class Reg:
    pools: Map[u128, AnyPool] = {}

    fn emplace[T](self, e: int, value: T):
        if 7u128 not in self.pools:
            self.pools[7u128] = Pool[T]()
        pool = cast self.pools[7u128] as Pool[T]
        pool.dense.push(value)
        pool.entities.push(e)

    fn get_mut[T](self, i: int) -> T:
        pool = cast self.pools[7u128] as Pool[T]
        return pool.dense[i]

fn main():
    reg = Reg()
    reg.emplace(1, Pos(x=10))
    reg.emplace(2, Pos(x=20))
    p = reg.get_mut[Pos](0)
    total = 0
    for v in (cast reg.pools[7u128] as Pool[Pos]).dense:
        total += v.x
        reg.emplace(3, Pos(x=30))        # pinned: lands in place or in an overflow page, not visited
    p.x += 1
    println(total, reg.get_mut[Pos](0).x)    # 30 11
```

`p` and the loop are both live across `emplace`, which reaches the pool through a map lookup, so
`Pool[Pos].dense` is pinned with a counter from `p`'s creation to its last use; the two pushes land
in place (capacity 4), `p` stays valid, the loop sums the two elements it began with. Here the map's
key is a plain number, so each `cast` keeps its one type-index compare (the std.ecs registry, keyed
by type number, needs none). The same program with `Reg` a `struct`, or with the pools in a `Box`,
compiles to the same thing: there is no `Box` any more, and a struct field holding objects follows
the same home and pin rules.

**A25, a local list of words** (`target/redesign/phase1/borrowing/e06e_local_array_of_views.em`;
today `E3063`, and the Rust twin compiles):

```ember
fn count_words(line: str) -> int:
    lower = line.to_lower()
    words: Array[str] = []
    for w in lower.split_whitespace():
        words.push(w)
    return len(words)
```

`words` is a local array of views into `lower`; it carries `lower`'s region, so it compiles with no
copy. Returned or stored in an object, each word would become an owned `String` (COPY, reported).

---

## 4. What each compiles to (C sketches of the hot loops)

The full programs are the models in section 5; these are the inner loops, as the D3 compiler would
emit them. `CHECK` is a bounds or unwrap check that panics.

**W1, the view loop** (Velocity leads: it is the smaller pool):

```c
Pool_Velocity *lp = reg->pools[T_Velocity];        /* type number: no cast check */
Pool_Position *op = reg->pools[T_Position];
lp->pins++; op->pins++;                            /* PIN: the body may destroy and emplace */
int64_t n0 = lp->len;                              /* the slots that exist now */
Velocity *vd = lp->dense; uint32_t *ve = lp->ents; Position *pd = op->dense; /* pinned: cannot move */
for (int64_t i = 0; i < n0; i++) {
    uint32_t e = ve[i];
    if (e == NIL) continue;                        /* tombstone */
    if (e >= op->sparse.len) continue;             /* bounds; sparse is not pinned, so reloaded */
    uint32_t j = op->sparse.data[e];
    if (j == NIL) continue;                        /* no Position */
    Position *pos = j < op->len ? &pd[j] : Position_at_slow(op, j);
    Velocity *vel = &vd[i];
    pos->x += vel->dx; pos->y += vel->dy;
    if (e % 100 == s) {                            /* the rare path */
        reg_destroy(reg, e);                       /* per pool: pins ? tombstone : swap-and-pop */
        uint32_t n = reg->next_id++;
        Position_add(op, n, (Position){0, 0});     /* len < cap ? in place : pins ? overflow page : grow */
        Velocity_add(lp, n, (Velocity){1, 1});
    }
}
Velocity_unpin(lp); Position_unpin(op);            /* count 0: settle (fill tombstones swap-and-pop) */
```

**W2, the depth walk** (no check: every watcher is proven):

```c
for (int64_t i = 0; i < nodes.len; i++) {
    Node *d = nodes.data[i], *p;
    while ((p = d->parent) != NULL) { depth_sum++; d = p; }
}
```

**W3, the update loop, one update, and the destroy loop**:

```c
for (int64_t i = 0; i < n; i++) { Obj *o = od[i]; UPDATE[o->type](o); }   /* no pin; self unchecked */

static void turret_update(Obj *self) {
    Obj *t = self->target;
    if (t && t->gen == self->target_gen                                     /* WATCH: one compare */
        && abs(t->x - self->x) + abs(t->y - self->y) < 300) t->health -= 1;
}

for (int64_t i = 0; i < n; i++) {                  /* static pin: no counter */
    Obj *o = od[i];
    if (!o) continue;                              /* tombstone */
    if (o->health <= 0) {
        push(&destroyed, o->id);
        od[i] = NULL;                              /* remove(i): a tombstone */
        if (++o->gen != 0) { o->target = free_list; free_list = o; }   /* destroy: watchers now miss */
    }
}
/* the loop's end: compact from the first tombstone, keeping the order */
```

**W4, one dispatch**:

```c
HandlerList *L = &d->handlers;
d->current = k % L->count;
Handler *h = (uint64_t)d->current < (uint64_t)L->flen ? &L->data[d->current]   /* = bounds check */
                                                     : hl_at_slow(L, d->current);
L->pins++;                                         /* PIN: the closure may change d->handlers */
h->fn(&h->st, d, k, k % 1000);                     /* st = {counter, m, &total} */
if (--L->pins == 0 && (L->ntombs | L->ov_len)) hl_settle(L);
```

**EnTT `get`, held across an `emplace`**:

```c
Pool_Position *pool = reg->pools[T_Position];
CHECK(e < pool->sparse.len); uint32_t j = pool->sparse.data[e]; CHECK(j != NIL);
Position *p = j < pool->len ? &pool->dense[j] : Position_at_slow(pool, j);
pool->pins++;                                      /* only because p is live across the emplace */
Position_add(pool, other, (Position){0, 0});
p->x += 1.0f;
Position_unpin(pool);                              /* after p's last use */
```

**A12**: `int64_t score = 0; b->handlers` holds `{lambda_1, {&score}}`; the click loop is
`for (i < b->handlers.len) h->fn(&h->env);` and `lambda_1` is `(*env->score) += 1;`. No pin, no cell,
no count.

**A24**: as W1's pattern with one pool: `pool->pins++` around the loop over `pool->dense`; `emplace`
pushes in place while there is capacity, else into an overflow page; after the loop, the count
returns to 0 and the pool settles. `p` is a pointer into the pinned buffer for its whole life.

**A25**: `String lower = to_lower(line); Array_str words = {0}; for each word: push(&words,
(str){ptr, len}); n = words.len; free(words.data); free(lower.data);`: the C a programmer would write.

---

## 5. The C models

Files, all in `C:/Users/ism19/Code/ember/target/redesign/bench/D3/`:

| file | workload | what D3 puts in it |
|---|---|---|
| `w1.c` | W1 ECS | EnTT sparse sets; pools as objects with a 4-byte type index and pinnable `dense`/`ents` fields; a run-time pin per view loop on both pools; tombstones on removal while pinned, overflow pages on a full add while pinned, settle at unpin; bounds checks on sparse lookups and pool element access; no cast check (compile-time type numbers) |
| `w2.c` | W2 tree | nodes on a size-class slab, no header; `parent` and `children` as raw pointers (proven watchers); 24-byte array headers; bounds checks on the two unprovable indexings; the `unwrap` of `parent`; `remove(index_of(node))` as search and shift |
| `w3.c` | W3 game objects | 40-byte objects: 8-byte header (generation, type index), a 12-byte watcher packed into the padding; virtual dispatch through the type index; a generation compare on every target use; destruction bumps the generation and frees the slot to the slab's free list; the destroy loop's static pin (tombstones, ordered compaction) |
| `w4.c` | W4 callbacks | 32-byte inline closures (`fn`, `counter`, `m`, `&total`); a pinnable handler array with a logical count; a pin and an unpin around every call; tombstone removal while pinned, ordered settle |

Left out of every model, on purpose, because every memory model pays them alike: integer overflow
checks (`[TYP-8]`) and the floor-modulo sign fix-up where the operand is never negative here. The
pin's side fields (tombstones, overflow pages) are inline in `w1.c` and `w4.c`; the design keeps them
in a side record reached only on slow paths, so the fast paths are the same.

Each was built once with `"C:/Program Files/LLVM/bin/clang.exe" -O2 wN.c -o wN.exe` and run once; each
finished and printed its line. To check that the models do the specified work, four reference
programs written a different way (plain arrays indexed by id, no pins, no watchers, no objects) were
built and run once from `check/`:

| model | checksum printed | reference (`check/*_ref.c`) |
|---|---|---|
| `w1.exe` | 2111831520 | 2111831520 |
| `w2.exe` | 647034814 | 647034814 |
| `w3.exe` | 9288233970 | 9288233970 |
| `w4.exe` | 34946993000 | 34946993000 |

`w4.c` was edited twice after its first run (a settle condition that would have missed an empty
contiguous part with overflow pages, then keeping the logical length in one field as the standard
library would); each edited version was rebuilt and run once, with the same checksum. `w1.c` had a
comment-only edit after its run. The time and memory figures those runs printed are not used here.

---

## 6. Sharp edges, and what should be slower than C

### What would still surprise a Python programmer

1. **Objects die when their home lets them go, not when the last name goes.** `m = Mesh(); a =
   Entity(m); b = Entity(m)`: `a.mesh` is `m`'s home, `b.mesh` watches it. If `a` is destroyed first,
   `b.mesh` reads `None` (or panics, if typed `Mesh`). Python would keep the mesh alive. The compiler
   warns where it can see a watcher outliving a home, and `ember inspect --owners` shows every home.
   This is the price of having no counts and no cycles; Unity's and Godot's destroyed objects behave
   the same way.
2. **A struct is a value.** `ps.push(p); qs.push(p)`: `qs` gets a copy (reported); `p` follows its
   value into `ps`. Two lists sharing one thing need a `class`.
3. **Loops see the container as it was when they began.** An element appended inside `for x in xs`
   is not visited (Python visits it); an element removed before its turn is skipped, and every other
   one is visited once (Python skips the element after a removed one). A handler added during a
   dispatch runs from the next dispatch.
4. **Removals during a pin take effect for storage at the end of the pin.** `len(xs)` is right at
   once, but indexing a list that has removals pending inside the loop takes the slow path (a scan
   past tombstones): correct, and slow in that one situation.
5. **Hidden costs exist, and are listed, not shown in the code.** COPY and CELL allocate; PIN and
   WATCH add small costs. A body change can change a function's effect summary and so add a pin in a
   caller (never a compile error). `@static` and the report are the tools.
6. **Memory of watched classes is held at its high-water mark.** A slot with checked watchers keeps
   its generation word, so its memory goes back only to objects of the same size class, never to the
   system, while the program runs.
7. **Threads keep a rule.** A watcher cannot cross a thread; an object moves or freezes; shared
   mutation is a `Mutex` or an `Atomic`.
8. **A non-optional watcher can panic.** Declaring `target: GameObject` instead of
   `Option[GameObject]` asserts the target outlives the watcher; if it does not, the use panics,
   naming the line that destroyed it.
9. **Two objects stored first into each other leak** (each is the other's home); the debug build
   reports it at exit, as today.

### What should be slower than C, by how much, and why (estimates, UNVERIFIED)

The comparison is against a C program doing the same algorithm with the same data layout and no
checks. How the measurer's C is laid out will move these figures: a C program for W1 that indexes
flat arrays by entity id has no sparse-set indirection at all, and that difference belongs to the
EnTT registry design, which any memory model implementing EnTT pays.

| workload | extra work in D3, per hot iteration | expected time vs the same C | memory vs the same C | why |
|---|---|---|---|---|
| W1 | 4 predictable compares per visited entity (tombstone, sparse bounds, two contiguous-part tests); 4 counter updates and a settle per step; a counter test per destroy and emplace | within 10% | equal to EnTT's layout (8 B per component + 4 B per entity per pool + 4 B per id per pool's sparse list); +32 B per pool header | the loop is memory bound (a sparse lookup and a component fetch per entity), so compares on data already loaded are cheap |
| W2 | none in the depth walk; a bounds check per indexing in the re-parent pass (about 10,000 per pass) | 1.0x | equal (Node is 40 B: `int32`, pointer, 24-byte array header; children buffers as C's) | every watcher is proven, so the walk is the same pointer chase as C |
| W2, if the ownership analysis is not available | a generation compare per step of the walk (the generation shares the parent's cache line) | up to about +25%, by analogy with the phase 2 in-order genref figure (+23.4%), which also counted 16-byte references (`target/redesign/phase2-vale-austral-pony/results.txt`) | +4 B per node, +4 B per watcher where it does not pack (children elements become 12–16 B) | the fallback |
| W3 | a generation compare per target use (same cache line as `t.x`); a load of the target's generation per new object | 0.9x to 1.1x | +8 B per object (40 B against 32 B for the same fields in C: +25%, about 0.8 MB for 100,000 objects) | a C program must also clear the targets of destroyed objects (a scan, or back-reference lists); D3's generation bump is O(1) per destruction, so D3 may be at or below C there |
| W4 | a pin and an unpin (two read-modify-writes on one cache line), a settle test, the `flen` index compare (the bounds check), the user's `v >= 0` | +10% to +30% | equal (32 B per handler, as `{fn, counter, m, total*}` in C); +32 B on the one array's header (56 B, pinnable) | the work per call is only a multiply-add, an add and an indirect call, so a fixed few instructions per call are a visible fraction; this is the costliest place in D3: a one-call-long pinned borrow run 10 million times |

Where D3 should not be slower than C: loops over containers that do not change them (no pin, no
tombstone test), reads and writes of object fields (no access words, no counts), passing and storing
object references (no retain or release), and every watcher the analysis proves (raw pointers).

---

## Sources

- Phase 1: `tasks/96/redesign/phase1-borrowing.md`, `tasks/96/redesign/phase1-sharing.md` (ids,
  severities, the part (5) eliminations, the probe programs in `target/redesign/phase1/`).
- Phase 2: `tasks/96/redesign/phase2-rc-elision.md` (count pair cost, is-unique cost, boxed
  iteration; raw data `target/redesign/phase2-rc-elision/results.txt`),
  `tasks/96/redesign/phase2-vale-austral-pony.md` (generational references, counter vs random
  generations, Valen's path invalidation; raw data `target/redesign/phase2-vale-austral-pony/results.txt`),
  `tasks/96/redesign/phase2-hylo-csharp.md` (paged storage and handles measured; raw data
  `target/redesign/phase2-hylo-csharp/results.txt`), `tasks/96/redesign/phase2-swift-mojo.md`
  (Swift's dynamic exclusivity cost, cursor iteration, region-based isolation).
- Spec: `docs/spec-source/Ember_v0.9.10_Hardened_35.md`, rules `[OWN-*]`, `[BRW-*]`, `[LT-*]`,
  `[TYP-15]`, `[TYP-15a]`, `[TYP-8]`, `[TYP-31]`, `[CTL-0]`–`[CTL-3b]`, `[CLO-*]`, `[OBJ-*]`,
  `[RC-*]`, `[EXC-*]`, `[WK-*]`, `[HEAP-*]`, `[HND-*]`, `[CELL-*]`, `[SEL-1]`, `[PAN-1]`, Appendix E.
- ODR-286 and ODR-280: `tasks/95/odr-pending.md`; the EnTT mapping: `tasks/95/ecs-entt-design.md`;
  probes: `target/analysis/ecs4/*.em`, `target/analysis/ecs5/*.em`.
- Models and checks: `target/redesign/bench/D3/w1.c`–`w4.c`, `target/redesign/bench/D3/check/*_ref.c`,
  `target/redesign/bench/harness.h`.
