# Phase 3, design D2: generational references and stable stores

Written 2026-10-24 by the phase-3 D2 design agent, for the memory-model rework (owner's directive of
2026-10-23: C-like speed and memory, memory safety without Rust's annoying parts, Python-like
syntax; owner's guidance: remove an annoyance either by (a) a more Pythonic spelling of the same
guarantee or (b) a smarter mechanism that makes the hazard impossible or harmless).

What this report rests on, so every claim can be traced:

- The six earlier reports in this folder (`phase1-borrowing.md`, `phase1-sharing.md`,
  `phase2-vale-austral-pony.md`, `phase2-hylo-csharp.md`, `phase2-rc-elision.md`,
  `phase2-swift-mojo.md`), the spec `docs/spec-source/Ember_v0.9.10_Hardened_35.md` (Parts VII,
  VIII, IX, XI and the closure rules of VI.5), ODR-286 in `tasks/95/odr-pending.md`,
  `tasks/95/ecs-entt-design.md`, and the probes in `target/analysis/ecs4` and `ecs5`.
- Four C models I wrote, `target/redesign/bench/D2/w1.c` to `w4.c`, built with clang 22.1.8 `-O2`
  and run once each (section 5). Their checksums were cross-checked against naive reference
  programs (plain arrays, no D2 machinery) that I wrote in the session scratchpad; all four match.
- No web page was read for this phase; external facts (Vale, EnTT, Swift) come from the phase-2
  reports, which name their sources, or are marked UNVERIFIED.
- No Ember program was run. The proposed syntax below is new, so `ember check` cannot check it and
  was not used.
- Numbers: each names its source. "By construction" means the size or count follows from the C
  model's declarations (checked with `sizeof`). Expected slowdowns in section 6 are my estimates
  with their reasoning, not measurements, and are marked so.

---

## 0. The design in one paragraph

Every object (a class instance, a list, a dict, a string, a closure that keeps state) lives in a
**store** whose slots never move, and each slot carries a 32-bit **generation**. An object has
exactly **one owner**, and anything else may hold any number of **references** to it: a reference is
a slot number and the generation it expects, 8 bytes, the size of a C pointer. Using a reference
compares the two numbers; a destroyed object's generation has changed, so a stale reference reads
as `None` (if its type is optional) or panics (if not), deterministically. There is no borrow
checker, no lifetime, no aliasing rule, no reference count and no exclusivity check. The compiler
removes the generation check wherever it can prove the object alive (six proofs, section 1.6), which
in the four workloads leaves exactly one generation check that runs often: W3's "is my target still
alive", which is the program's own question. Containers whose elements are referenced keep them in stable
slots, and a `for` loop sees the container as it was when the loop began, so spawning and despawning
inside the loop just work.

---

## 1. The model in plain words

### 1.1 What the programmer sees: values, objects, references

1. **Values**: numbers, `bool`, `char`, enums, tuples, fixed arrays, and `struct`s. They live
   inline (in a variable, a field, a list's buffer), are copied on assignment as in C, and have no
   header, no identity and no check. A struct may hold references, and may own objects (a field
   that owns a `String`); section 1.3 says what copying such a struct does.
2. **Objects**: `class` instances, `list`, `dict`, `set`, `String` (immutable, as Python's `str`),
   closures that keep state, and `Store[T]`. Each lives in a store slot that never moves, carries a
   generation, and has one owner.
3. **References**: the only pointer a program sees. A class-typed or collection-typed local, field,
   parameter or element holds a reference unless it is the owner. `ref x` names a reference to a
   value that lives elsewhere (a field, a store element). References may be copied, stored in
   fields and collections, captured, returned and compared with `is`, with no restriction.

One word is added to the language: **`own`**, written on a field or a collection's element type
when that place owns the objects put in it (`nodes: list[own Node]`). Locals never need it (a
local owns what it creates), and a field initialised with a new object owns it without the word
(`children: list[Node] = []` owns its list; the list holds references). `Box`, `Shared`,
`SyncShared`, `Weak`, `Cell`, `RefCell`, `Pool`/`Handle` and `owned fn` are no longer needed.

### 1.2 Where objects live: stores

- Each class hierarchy (a root class and all its subclasses) has one **store** per thread: an
  address range reserved at start-up (`VirtualAlloc(MEM_RESERVE)` on Windows, `mmap` on Linux) and
  committed as it grows. Slots never move, so a slot number turns into an address with one multiply
  and add: `base + index * slot_size`, where `base` is loaded once per function. On targets without
  address reservation (32-bit, WASM) the store is pages plus a page table, one more load per use.
- A slot starts with its **generation**, `u32`: odd while an object lives in it, even while it is
  free. Allocating and destroying each add 1. A slot whose generation reaches 2^32 - 2 is retired and
  never reused (the rule `[HND-3]` already states for `Pool`). Slot 0 of every store is reserved with
  a dead generation, so `None` is `{0, 0}` and fails the same check a stale reference fails.
- An open hierarchy also stores a `u32` class id beside the generation (the dynamic type; virtual
  calls index a table with it). Its slots have the size of the largest class in the hierarchy.
- Free slots form a list threaded through the dead slots (no extra memory), reused last-in
  first-out. A store keeps its high-water mark: freed slots are reused by the same store, not given
  back to the operating system.
- A **`Store[T]`** is the same thing as a library container for values with identity (particles,
  ECS components): stable slots, generations, a free list. ECS pools are built on it (section 3.5).

### 1.3 Ownership

- **O1.** Every object has exactly one owner: a local, a field or element that owns (`own`, or a
  field initialised with a new object), a closure (what it captured), a `Store`, or a module.
- **O2.** A new object (a constructor call, a literal, `.copy()`, a call that returns an owner) is
  owned by the place it is stored into. A temporary is destroyed at the end of its statement, unless
  a local holds a reference or slice into it, in which case it lives until that local's scope ends
  (this removes A22).
- **O3.** Storing an owner into an owning place at the owner's last use moves ownership, and **the
  old name stays usable as a reference**: there is no moved-from state and no "use after move" error.
  Storing an object anywhere else stores a reference.
- **O4.** An object is destroyed when its owner is destroyed, overwritten, removed from its owning
  collection, or goes out of scope, or by `del x` on its owning local. Destroying runs `drop`,
  destroys what the object owns, changes the generation and frees the slot. Nothing is counted.
- **O5.** An object whose own method or closure body is running is not destroyed until that call
  returns; removing it from its container happens at once. (A handler that unregisters itself, a
  game object that removes itself from the world, keeps working until it returns.)
- **O6.** Copying a struct that owns objects copies references to them; the original keeps
  ownership, except at the original's last use, where the copy takes it (O3).
- **O7.** Compile-time errors left by this section, each with a fix-it: a new object stored only into
  reference places (nothing would own it: "make `target` own it, `target: own Node`, or store it in
  an owning place first"); an owning place given an object owned elsewhere ("this would take the
  Node owned by `nodes`: store a reference, or move it out with `pop`/`take`, or `.copy()` it").

### 1.4 References and the check

| what | layout | use |
|---|---|---|
| reference to an object | 8 bytes: `{u32 slot, u32 generation}` | address = `base + slot * size`; check: `slot.gen == ref.gen` |
| reference to a value inside a slot (a store element, a field) | 16 bytes: the slot's address and the occupant it expects (and the field's offset) | check the slot's generation word, then use the address |
| slice of a list or string (`Span`, `str`) | 16 bytes: `{object reference, u32 start, u32 length}` | check the object, then bounds |
| `None` | `{0, 0}` | fails the check like a stale reference |

- **G1.** Using a reference the compiler cannot prove live loads the slot's generation and compares
  it (one load from the same slot the program is about to read, a compare, a predicted branch).
- **G2.** A stale optional reference reads as `None`: `if t is not None:` is that one compare. A
  stale non-optional reference panics: `stale reference: the Node in slot 41 was destroyed (line 88,
  nodes.clear()); used at line 120`. The debug runtime records the destroying line per slot; other
  profiles name the type and the line of use.
- **G3.** A reference to a value in a contiguous `list` (not a `Store`) is a position: the list's
  reference and the index, re-resolved with a bounds check at each use. After `remove(0)` it names
  the element now at that index (section 6). A program that needs identity-stable plain values uses
  `Store[T]`.

### 1.5 Containers and loops

- `list[T]` is a contiguous buffer, doubling from four elements as today (`[HEAP-2]`): values inline,
  objects as 8-byte references. `dict` and `set` likewise. `Store[T]` keeps stable slots.
- **L1.** A `for` loop over a container visits the elements present when it began, in order, and
  skips those removed before it reaches them. Elements added during the loop are not visited.
- **L2.** Adding during such a loop: a list appends after the loop's start length; a store takes a
  free slot the loop has already passed (the loop publishes its cursor before any call that may
  insert), else appends.
- **L3.** Removing during such a loop: the element is removed (and destroyed if owned) at once; a
  list leaves a tombstone in its place and closes the gaps when the outermost loop over it ends.
- **L4.** L1-L3 hold whether the change is written in the loop body or happens in a function it
  calls: a list being iterated has an iteration count in its header that `remove` tests (one load and
  branch on `remove`, two stores per loop, none per iteration).
- **L5.** In a loop whose body cannot change the container (whole-program effect analysis), the loop
  is the plain pointer loop C writes. In a loop that may, the length and the buffer address are
  re-read each iteration and tombstones are skipped.

### 1.6 What is checked when, and what removes the checks

At **compile time**: types; one owner per object (O7); definite initialisation; the thread rule
(1.7); and the proofs below, which remove run-time checks. There are no borrow, lifetime, move or
exclusivity errors.

At **run time**: a generation check on each use of a reference that no proof covers; bounds checks;
the L2-L4 bookkeeping in loops that change their own container.

The proofs (each removed check is listed by `ember inspect --checks`, as `[EXC-3a]` does today):

- **P1 Owner access.** A reference read from its owning place (an owning local, field or collection
  element) and used before anything that could destroy objects of its class runs needs no check.
  This covers `for o in objects: o.update()` and calling a closure taken from its owning list.
- **P2 No-destroy regions.** Inside a loop or function that cannot destroy objects of class C (no
  reachable call removes, overwrites or ends an owner of C: whole-program effect analysis, which
  Ember already does for `[EXC-3]`), a C reference checked once stays live to the end of the region.
  A loop-invariant reference is checked once in the loop's preheader. Vale's `pure`, inferred.
- **P3 Grow-only owners (the arena case).** If every owner of C objects is an `own` collection that
  is only ever appended to, and outlives every use, no C reference needs a check while it lives.
- **P4 Program-lifetime owners.** Objects owned by `main`'s top-level locals that `main` never
  reassigns or deletes, and module variables, are live whenever code runs on the main thread.
- **P5 Fresh references.** A reference produced by an iteration step, a lookup or a constructor and
  used before any possible destroy needs no check (P2's local case).
- **P6 Self at entry.** A method's `self` or a closure's captured state is live on entry, because the
  caller checked or proved the reference it called through.

### 1.7 Closures, threads, C

- **Closures** capture values by copy and objects by reference. A captured local that the closure
  writes, when the closure escapes (is stored or returned), lives in a small cell object owned by
  the declaring scope; if the closure is that variable's only user after it is created, the closure
  owns it. A stored closure is an object (its captures are its fields, readable as `h.counter`, as
  Python's function attributes are), so it may remove itself from the list that holds it while it
  runs (O5). There is no `owned fn`, `once fn`, `FnMut` or `'static`.
- **Threads**: stores are per thread, so references do not cross threads. An object reaches another
  thread by being moved (with everything it owns; it is relocated into the receiver's stores and the
  sender's references go stale) or frozen (immutable, shared read-only, destroyed only after the
  threads that share it are joined). Shared mutable state between threads lives in a `Mutex` or
  `Atomic`, as in Python with locks; a reference derived inside a lock's `with` block cannot leave the
  block (this is the compile-time rule that fixes phase 1's A33 hole). `Send`/`Sync` are inferred
  from these rules, never written. This part is design only: no workload exercises it.
- **C**: C is handed references (slot and generation) for callbacks and user data, and the
  trampoline checks the generation, so a callback after its object died finds nothing (phase 1 A20(b)).
  Raw pointers into slots stay `unsafe`.

### 1.8 Costs, per object, per reference, per operation

| | plain C | D2 | Ember today |
|---|---|---|---|
| per object | 0 (8-byte vtable pointer if virtual) | 4-byte generation; +4-byte class id in an open hierarchy | 24-byte header (`[OBJ-1]`) + 4 bytes per non-`Copy` field (`[EXC-19]`) |
| reference to an object | 8-byte pointer | 8 bytes | 8-byte handle |
| stored reference to an element or field | 8-byte pointer | 16 bytes | not storable (a view) |
| copying or dropping a reference | nothing | nothing | retain / release (`[RC-1]`) |
| using a reference | one load | `base + slot * size` (one `lea` or `imul`), then the load | the load, plus an access-word check on non-`Copy` fields |
| check | none | where unproven: one load + compare + predicted branch | access word: load, compare, store when held |
| destroying | `free` | generation += 1, push the free list, `drop` | count to 0, `drop`, free |
| memory after a population shrinks | returned to `malloc` | kept by the store, reused by the same class | returned to the allocator |

Published and measured costs of the mechanism itself, from phase 2: a generation-checked reference
cost +0.57 ns per in-order use (+23%) and +4.61 ns per random use (+45%) against a plain pointer,
about half of it from 16-byte references (`phase2-vale-austral-pony.md`, section 6; D2's object
references are 8 bytes, which removes that half); a generation-checked handle with the generation
beside the value cost 1.37x a raw pointer on random access over 64 MiB, and 1.75x with the
generation in a separate array (`phase2-hylo-csharp.md`, Part 3; D2 keeps it beside the value);
Vale's own whole-program figure was +10.84% with no regions (one program, quoted in
`phase2-vale-austral-pony.md`, 1.3).

### 1.9 What changes in the spec

Removed: `[BRW-1]`-`[BRW-11]` (aliasing xor mutation, two-phase borrows, field and index
disjointness), `[LT-*]` (regions, elision, `@borrows`, region vectors, view-type storage rules
`[TYP-15]`), `[OWN-3]`/`[OWN-4]` (use after move, move in a loop), Part VIII's `[RC-*]`, `[EXC-*]`,
`[WK-*]` and the 24-byte `[OBJ-1]` header, `[HEAP-3]`-`[HEAP-10]` (`Shared`, `SyncShared`, `Weak`),
`Box` (an `own` field is the box), `[CELL-*]` (no exclusivity, so nothing needs interior mutability),
`[HND-*]` (`Pool`/`Handle` become `Store[T]` and references), `[CLO-2]`-`[CLO-6a]`, `[CLO-13]`,
`[CLO-15]` (capture modes, `owned fn`, `once fn`), `[CORO-6]` and `E2221`/`E2229` (frames are objects
and never move), `[SEL-1]`'s eleven-row table (three rows remain: value, object, `Store`).

Kept: `[OWN-1]` (one owner, now for objects), `[OWN-2]`/`[DRP-*]` (deterministic destruction),
`[PAN-1]` (abort, no unwinding: O5 and A23 rely on it), bounds checks, `[ARN-*]` arenas for plain
values, `[DSJ-*]` and `[SIMD-*]` (proven non-overlap for vector loops), `[THR-3]`-`[THR-15]` (locks,
atomics, channels, scoped tasks).

New: the store and generation rules (1.2, 1.4), O1-O7, L1-L5, P1-P6, and the `own` word.

---

## 2. Every annoyance, A1 to A25 and A31 to A34

Categories follow phase 2's: **removed** (the restriction is gone and no run-time check carries the
safety, because the hazard is impossible or harmless), **moved to run time** (the restriction is gone
and a generation check, a deterministic `None` or panic, carries the safety where no proof applies),
**kept**. Kind (a) is a more Pythonic spelling of the same guarantee; kind (b) a mechanism that makes
the hazard impossible or harmless. "Phase 1 (5)" names the elimination in phase 1's part (5) that
this verdict uses.

| id | verdict | kind | how | cost |
|---|---|---|---|---|
| A1 borrowing the whole object | **removed** | (b) | No aliasing rule. The hazard (a method reallocating the buffer a loop points into) cannot happen: a loop whose body may change the list re-reads its buffer and length each step (L5), and objects never move. Phase 1 (5)(b). | in such loops, one load of the buffer and length per iteration; loops proven not to change it are C's loop |
| A2 container invalidation | **removed** | (b) | Lists of objects hold references and objects never move, so `boss = enemies[0]; enemies.append(...); boss.hp //= 2` is Python's program and meaning. Values with identity live in `Store[T]` (stable slots). A reference to a value in a contiguous list is a position (G3). Phase 1 (5)(a) and (b). | none for growth; a bounds or generation check where a stored element reference is used unproven |
| A3 two mutable references into one container | **removed** | (b) | Two references may name the same element and both write: writing plain data frees nothing (phase 1 (5)(b), which Ember already applies to `Copy` class fields, `[EXC-17]`). `collide(bodies[i], bodies[j])` compiles; with `i == j` it aliases, as in Python and C. | no `noalias` on such parameters, as C without `restrict` (vector loops keep `[DSJ-*]`) |
| A4 changing a collection while iterating | **removed** | (b) | L1-L4: the loop sees the container as it began; appends are not visited, removals happen at once and the list closes its gaps at the loop's end, stores reuse only passed slots. W1 is the test. Phase 1 (5)(b) (tombstones and stable pages) and EnTT's paged pools. | in loops that change their container: a length and buffer reload per iteration, a tombstone test, one compaction pass; pools: one cursor store before a call that may insert |
| A5 returning references, conditional returns | **moved to run time** | (b) | A returned reference carries its own validity; no rule says which argument it borrows, so the ECS `get` through a map and a cast compiles (ODR-286's `E3060` disappears). A stale result panics or reads `None` at use. Phase 1 (5)(b). | a generation check at a use no proof covers |
| A6 structs holding references | **moved to run time** | (b) | References are ordinary 8- or 16-byte values; a struct may hold them, a list may hold such structs, with no region parameters and no view-type rules. Phase 1 (5)(b), without the count. | as A5 |
| A7 self-referential structures | **moved to run time** | (b) | Objects never move, so a struct may hold references or slices into an object it owns (`Document(text, words)`: the words are slices of the `text` string object). Phase 1 (5)(b), without the count. | as A5 |
| A8 graphs, parent pointers, cycles | **moved to run time** | (b) | References in any direction, any number; ownership is a tree of owners beside them. No `Weak`, no leaked count cycles. A parent that was destroyed reads `None`. W2 is the test, and P3 removes every check in it. Phase 1 (5)(b) (arena-owned graphs, with single nodes destroyable). | 4-byte generation per object; checks only where unproven |
| A9 shared mutable state | **moved to run time** | (b) | Any number of writable references to one object; no `RefCell`, no borrow panics. The remaining hazard (an object replaced or destroyed while referenced) is the generation check. Phase 1 (5)(b). | as A5 |
| A10 choosing among ownership kinds | **removed, with a residue kept** | (a) | Three kinds: value, object, reference. `Box`, `Rc`, `Arc`, `Cell`, `RefCell`, `Weak`, `Shared` are gone. **Kept**: the word `own` on owning fields and element types. Phase 1 (5)(a). | none |
| A11 moves, use after move, clone | **removed, with a residue kept** | (b) | Storing an owner moves ownership and leaves the name usable as a reference (O3); passing never moves; `snapshot = items` makes a second name for the same list, as Python. `spawn_all` that appends the same object n times builds n references to it, Python's meaning. **Kept**: an object dies with its owner even if references remain (they read `None` or panic); putting one object into an owning list twice needs `.copy()`. Phase 1 (5)(b), changed: no hidden copy. | none |
| A12 closures capturing mutable state | **moved to run time** | (a) and (b) | Escaping written captures become cells owned by the declaring scope (phase 1 sharing A12(a), with ownership instead of a count); stored closures are objects that may unregister themselves while running (O5). No `owned fn`, `FnMut`, `'static`. A closure that outlives the scope owning a cell it uses gets a stale-reference panic when it touches it. W4 is the test. | a cell per escaping written capture; a generation check on the cell where no proof covers it (P4 covers W4) |
| A13 trait objects, downcasting | **removed for downcasts; dyn compatibility kept** | (a) | `cast` of a `dyn` place gives a reference (phase 1 sharing A13(a)); the registry finds pools by a compile-time component index (A13(b)(1)), so the cast is checked once at the type level. Generic methods in `dyn` interfaces (`E2050`) are not a memory-model question: kept as is. | none at run time for the index form |
| A14 async: Pin, Send, borrows across await | **removed by design (not built)** | (b) | A suspended frame is an object in a store and never moves, so there is no `Pin` and a reference across `await` is an ordinary checked reference; the thread rule (1.7) replaces `Send`. Generators gain the same (`E2221` goes). `async` stays reserved. | one store slot per spawned task |
| A15 threads, Send/Sync, scoped threads | **kept, softened** | (a) | Move or freeze to cross threads (phase 2 Vale/Pony idea 5.5); `Mutex`/`Atomic` for shared mutation; `Send`/`Sync` inferred, never written; scoped tasks as today. Generations cannot stop data races, so the rule stays compile-time. | relocation when moving an object graph to a thread |
| A16 generics friction | **kept** | | Not part of the memory model; unchanged. | |
| A17 error messages that do not say what to do | **moved to run time** | (b) | Almost no compile-time ownership errors remain (O7's two, each with a fix-it). A stale use panics with the object, the destroying line (debug) and the use. Phase 1 (5)(b). | debug: one word per slot for the destroying line |
| A18 initialisation | **removed** | (b) | Store slots start zeroed: unset numbers are 0 and unset references are `None`, so an early read is defined, never garbage (phase 1 (5)(b)); a non-optional reference read before it is set panics as stale. `E2100` can be relaxed to a lint. | zeroing slots on reuse (dead stores removed where a constructor sets every field) |
| A19 global mutable state | **removed** | (a) | Module variables are objects owned by the module, confined to the main thread (phase 1 sharing A19(a)); references to them need no check (P4). | none |
| A20 unsafe code and FFI | **moved to run time** | (b) | C holds references (slot and generation), not addresses, for callbacks and user data; the trampoline checks the generation (phase 1 sharing A20(b)). Raw pointers stay `unsafe`. | one table load and compare per callback |
| A21 updating a map entry in place | **removed** | (a) | `counts[w] += 1` with `dict(default=0)`; a stored reference to a dict value is a position (dict, key), re-resolved by a lookup at use (G3). Phase 1 (5)(a). | none for the update; a hash lookup per use of a stored entry reference |
| A22 a view of a temporary kept past its statement | **removed** | (a) | O2: a temporary that a local refers to lives until that local's scope ends. Phase 1 (5)(a). | the temporary's memory is held longer |
| A23 moving a field out from behind `mut self` | **removed** | (b) | No moved-from state exists; `match self.state` binds references, and `self.state = Idle` destroys the old state object. `[PAN-1]`'s no-unwinding, as phase 1 (5)(b) says. | none |
| A24 struct, `Box` or class changes what compiles | **removed** | (b) | One rule set: values copy, objects are owned once and referenced freely; `Box` is gone. The ECS registry compiles the same whichever way pools are declared (section 3.5). Phase 1 (5)(b). | none |
| A25 a collection of views | **moved to run time** | (b) | A string slice is `{string reference, start, length}`, storable in a list or a field; strings are immutable objects, so a slice is valid while its string lives. Phase 1 (5)(b), with a generation instead of a count. | 16 bytes per slice; a check where unproven (none in the A25 program) |
| A31 caching inside a read-only method | **removed** | (b) | No read-only receivers: any method may write `self`'s fields; nothing can dangle, because replacing a field's object changes its generation. | none |
| A32 callbacks that receive the object that holds them | **removed** | (b) | `for h in self.handlers: h(self)` compiles; a handler may add or remove handlers (L1-L4), and one that removes itself finishes its call (O5). Handlers added during `fire` are not called by that `fire` (Python's would be). | the loop reloads the buffer and skips tombstones |
| A33 lock ceremony, references out of a guard | **kept** | (a) | A reference derived inside `with g = m.lock():` cannot leave the block (compile time); return a copy or move a value out. This fixes phase 1's soundness hole. Generations cannot replace it: the hazard is a race, not a free. | the copy |
| A34 clone-before-move for shared handles | **removed** | (b) | References are plain 8-byte values; nothing is counted, so nothing needs a clone before a capture. | none |

Summary: removed A1, A2, A3, A4, A10 (residue `own`), A11 (residue: dies with its owner), A13
(downcasts), A14 (by design), A18, A19, A21, A22, A23, A24, A31, A32, A34. Moved to run time A5,
A6, A7, A8, A9, A12, A17, A20, A25. Kept A15 (softened), A16, A33, and the residues named above.

---

## 3. The test programs in D2's syntax

Ember's syntax where it stays (`fn`, typed fields, `for i in 0..n` or `range`, `x as T`,
`Option[T]` with `is None`); D2 adds `own`, flow narrowing after `is not None` (as Python's type
checkers do), and nothing else. `h_seed`/`h_rand` stand for the harness's generator.

### 3.1 W1, ECS

```ember
from std.ecs import Registry

struct Position:
    x: f32
    y: f32

struct Velocity:
    dx: f32
    dy: f32

fn main():
    reg = Registry()
    for i in range(1_000_000):
        e = reg.create()                      # entity ids are serial and never reused: e.id == i
        reg.emplace(e, Position(x=(i % 1000) as f32, y=(i // 1000) as f32))
        if i % 2 == 0:
            reg.emplace(e, Velocity(dx=(i % 7) as f32 * 0.5, dy=(i % 5) as f32 * 0.25))

    for s in range(100):
        for e, p, v in reg.view[(Position, Velocity)]():
            p.x += v.dx
            p.y += v.dy
            if e.id % 100 == s:
                reg.destroy(e)                # now: e's components are gone, their references stale
                n = reg.create()              # not visited by this loop (L1)
                reg.emplace(n, Position(x=0.0, y=0.0))
                reg.emplace(n, Velocity(dx=1.0, dy=1.0))

    checksum: u64 = 0
    for e, p in reg.view[(Position,)]():
        checksum += (p.x * 4.0) as i64 as u64 + (p.y * 4.0) as i64 as u64
    checksum += reg.alive() as u64
    println(checksum)
```

What the user did not write: no `Commands` buffer, no collect-then-destroy, no `Mut[...]`, no
borrow of the registry. The despawn and spawn take effect at once (D2's choice); the loop does not
visit the spawned entity because the pool puts it in a slot the loop has passed (L2).

### 3.2 W2, a tree with parent pointers

```ember
class Node:
    value: i32
    parent: Option[Node] = None          # a reference
    children: list[Node] = []            # the node owns this list; its elements are references

fn main():
    h_seed(42)
    nodes: list[own Node] = [Node(value=0)]
    for i in range(1, 1_000_000):
        p = nodes[h_rand() % i]
        n = Node(value=(i % 1000) as i32, parent=p)
        nodes.append(n)                  # nodes owns it now; `n` is still usable as a reference
        p.children.append(n)

    depth_sum: u64 = 0
    for round in range(10):
        for n in nodes:
            d = 0
            m = n
            while m.parent is not None:
                m = m.parent
                d += 1
            depth_sum += d
        for i in range(1, 1_000_000):
            if (i + round) % 100 == 0:
                q = nodes[h_rand() % i]
                n = nodes[i]
                n.parent.children.remove(n)    # a `None` parent panics here, as Python's AttributeError
                n.parent = q
                q.children.append(n)

    checksum = depth_sum
    for n in nodes:
        checksum += (n.value as i64 * len(n.children)) as u64
    println(checksum)
```

No `Weak`, no `Rc<RefCell<…>>`, no arena of indices: the parent is a reference. P3 holds (every
`Node` is owned by `nodes`, which only grows), so this program carries no generation check at all.

### 3.3 W3, game objects that refer to each other

```ember
open class Object:
    id: int
    x: i32
    y: i32
    health: i32
    target: Option[Object] = None        # a reference: reads None once its object is destroyed

    virtual fn update(self):
        pass

class Ship(Object):
    override fn update(self):
        t = self.target
        if t is not None:
            self.x += sign(t.x - self.x)
            self.y += sign(t.y - self.y)

class Rock(Object):
    override fn update(self):
        self.x = (self.x + 1) % 1000

class Turret(Object):
    override fn update(self):
        t = self.target
        if t is not None and abs(t.x - self.x) + abs(t.y - self.y) < 300:
            t.health -= 1

fn make(id: int, x: i32, y: i32, target: Option[Object]) -> Object:
    h = (100 + id % 50) as i32
    match id % 3:
        0 => return Ship(id=id, x=x, y=y, health=h, target=target)
        1 => return Rock(id=id, x=x, y=y, health=h, target=target)
        _ => return Turret(id=id, x=x, y=y, health=h, target=target)

fn main():
    h_seed(7)
    objects: list[own Object] = []
    for i in range(100_000):
        x = (h_rand() % 1000) as i32
        y = (h_rand() % 1000) as i32
        target = None if i == 0 else objects[h_rand() % i]
        objects.append(make(i, x, y, target))

    next_id = 100_000
    destroyed = 0
    for frame in range(1000):
        for o in objects:
            o.update()
        dead: list[int] = []
        for o in objects:
            if o.health <= 0:
                dead.append(o.id)
                objects.remove(o)        # destroyed now; every target pointing at it reads None
        destroyed += len(dead)
        for _ in dead:
            x = (h_rand() % 1000) as i32
            y = (h_rand() % 1000) as i32
            target = objects[h_rand() % len(objects)] if len(objects) > 0 else None
            objects.append(make(next_id, x, y, target))
            next_id += 1

    total: i64 = 0
    for o in objects:
        total += o.x as i64 + o.y as i64 + o.health as i64
    println(total as u64 + 1_000_000 * destroyed as u64)
```

The program never clears a target: "the reference must never dangle" is the generation check
itself. I read the workload's "kind = id % 3 ... health = 100 + id % 50" in (c) as the new
object's id; the C model does the same (and the naive reference agrees).

### 3.4 W4, callbacks with captured mutable state

```ember
struct Event:
    k: int
    v: int

class Dispatcher:
    handlers: list[own fn(Event)] = []

    fn register(self, h: fn(Event)):         # stores into an `own` list: takes ownership (inferred)
        self.handlers.append(h)

    fn unregister(self, h: fn(Event)):
        self.handlers.remove(h)              # destroys it; if it is running, when it returns (O5)

    fn dispatch(self, k: int):
        h = self.handlers[k % len(self.handlers)]
        h(Event(k=k, v=k % 1000))

fn main():
    total = 0
    d = Dispatcher()

    fn make_handler(m: int) -> fn(Event):
        counter = 0
        fn handler(ev: Event):
            counter += ev.v * m
            total += 1                         # main's `total`: a cell owned by main
            if ev.k % 100_000 == 99_999:
                d.unregister(handler)          # itself
                d.register(make_handler((ev.k % 17) + 1))
        return handler

    for i in range(10_000):
        d.register(make_handler((i % 13) + 1))
    for k in range(10_000_000):
        d.dispatch(k)

    checksum = total as u64
    for h in d.handlers:
        checksum += h.counter as u64          # a stored closure's captures read as fields
    println(checksum)
```

Python's own program, without `nonlocal`. Unregistering first and then reading the captured `d`
and `total` works because O5 keeps the running handler alive until it returns.

### 3.5 The EnTT registry: `get` and `view`

The library (one pool per component type, found by a compile-time index, so ODR-286's `cast` is a
type-level fact):

```ember
class Pool[T](AnyPool):
    slots: Store[Slot[T]]          # stable slots: {owner entity id, value}; the owner is the generation
    sparse: list[u32] = []         # entity id -> slot + 1, 0 = absent

    fn get(self, e: Entity) -> ref T:
        s = self.sparse[e.id] if e.id < len(self.sparse) else 0
        if s == 0:
            panic(f"entity {e.id} has no {type_name[T]()}")
        return ref self.slots[s - 1].value      # {slot address, expected owner = e.id}

class Registry:
    pools: list[own AnyPool]

    fn get[T](self, e: Entity) -> ref T:
        return (self.pools[component_index[T]()] as Pool[T]).get(e)

    fn view[Q](self) -> View[Q]:               # Q a tuple of component types
        return View[Q](self)                   # iterates the smallest pool, probes the others
```

The owner's code, which ODR-286 found impossible to write today:

```ember
p = reg.get[Position](e)
reg.emplace(other, Position(x=1.0, y=1.0))   # the pool grows; its slots never move: p is fine
p.x += 1.0                                    # checked: slot.owner == e.id
for e2, q, v in reg.view[(Position, Velocity)]():
    q.x += v.dx
    if q.x > 100.0:
        reg.emplace(reg.create(), Position(x=0.0, y=0.0))   # inside the view: allowed (L2)
reg.destroy(e)
p.x += 1.0     # panic: stale reference: the Position of entity 17 was removed (destroy, line 12)
```

Two D2 changes to `tasks/95/ecs-entt-design.md`: an `Entity` is a 64-bit serial that is never
reused (so it needs no version, and the occupant id stored in each pool slot is that slot's
generation, at no memory cost beyond EnTT's packed entity array), and pools never swap-and-pop
(tombstones and a free list instead, so component references stay valid; EnTT offers such a policy
too, "in-place delete", from memory, UNVERIFIED). Items 3 and 4 of the design's "cannot copy"
list disappear.

### 3.6 The three worst programs from phase 1

Chosen as the high-severity ids that phase 1 rated "same" or "worse" in Ember and that W1-W4 do not
already cover (A2, A4 and A24 are W1 and 3.5, A8 is W2, A9 and A13 are W3, A12 is W4): A11 (moves),
A25 (a list of words) and A32 (callbacks that receive their holder).

**A11** (`target/redesign/phase1/borrowing/e11_moves.em`, four errors today):

```ember
struct Msg:
    to: String
    body: String

fn notify(m: Msg):
    send(m)
    log(m)                       # fine: passing never moves

fn rename(p: Player):
    n = p.name
    println(n)
    keep(p)                      # fine: `n` is a reference to p's name string

fn spawn_all(template: Player, n: int) -> list[Player]:
    v: list[Player] = []
    for _ in range(n):
        v.append(template)       # n references to one Player: Python's meaning
    return v

fn alias():
    items = [1, 2]
    snapshot = items             # a second name for the same list
    items.append(3)
    println(snapshot)            # [1, 2, 3], as Python
```

**A25** (`e06e_local_array_of_views.em`, `E3063` today):

```ember
fn count_words(line: str) -> int:
    lower = line.lower()          # a new immutable string, owned by `lower`
    words: list[str] = []
    for w in lower.split():
        words.append(w)           # a slice {lower, start, length}
    return len(words)
```

**A32** (`target/redesign/phase1/sharing/a32_reentrant_callback.em`, `E3080` today):

```ember
class Ui:
    handlers: list[own fn(Ui)] = []
    clicks: int = 0

    fn fire(self):
        for h in self.handlers:
            h(self)

fn count(ui: Ui):
    ui.clicks += 1

fn add_more(ui: Ui):
    ui.handlers.append(count)     # appended now; called from the next fire on (L1)
```

---

## 4. What each compiles to (C sketches of the hot loops)

Excerpts of the models in section 5, trimmed. `slot(i)` is `base + i * size`.

**W1, the view loop** (`w1.c`): the velocity pool leads; no generation check on `p` or `v` (P5).

```c
u32 n = vel.len;                                   /* L1: the slots present when the loop began */
for (u32 j = 0; j < n; j++) {
    VelSlot *v = (VelSlot *)vel.slots.base + j;
    u64 e = v->owner;
    if (e == DEAD_OWNER) continue;                 /* tombstone */
    u32 ps = e < pos.sparse_len ? ((u32 *)pos.sparse.base)[e] : 0;   /* probe, bounds-checked */
    if (ps == 0) continue;
    PosSlot *p = (PosSlot *)pos.slots.base + (ps - 1);
    p->v.x += v->v.dx;
    p->v.y += v->v.dy;
    if (e % 100 == s) {                            /* rare path */
        vel.iter_limit = j;                        /* L2: inserts may reuse only slots <= j */
        reg_destroy(&reg, e);                      /* each pool's remove through its interface table */
        u64 ne = reg_create(&reg);
        pos_emplace(&pos, ne, (Position){0, 0});
        vel_emplace(&vel, ne, (Velocity){1, 1});   /* takes slot j, just freed */
    }
}
vel.iter_limit = UINT32_MAX;
```

**W2, the parent walk** (`w2.c`; the listing clang emits is a `lea`, a load and a compare per hop,
one `lea` more than a C pointer chase, with both loads in the same slot):

```c
for (i64 i = 0; i < nodes.len; i++) {
    NodeSlot *m = node(nodes.ptr[i].idx);
    u64 d = 0;
    while (m->parent.gen != 0) { m = node(m->parent.idx); d++; }   /* P3: no check */
    depth_sum += d;
}
```

**W3, the update loop and a Ship** (`w3.c`): the target check is the only check, and it is the
program's meaning.

```c
for (i64 i = 0; i < objects.len; i++) {            /* P1: owners, no check */
    Ref r = objects.ptr[i];
    update_table[obj(r.idx)->cls](r);               /* the virtual call */
}
static void ship_update(Ref self) {
    ObjSlot *o = obj(self.idx);                     /* P6 */
    Ref t = o->target;
    ObjSlot *to = obj(t.idx);
    if (to->gen == t.gen) {                         /* `t is not None`: None and stale both fail */
        o->x += sign(to->x - o->x);
        o->y += sign(to->y - o->y);
    }
}
```

and the removal loop: `obj_destroy(r.idx); objects.ptr[i] = NONE;` per dead object, then one
compaction pass from the first removed index.

**W4, dispatch and the handler** (`w4.c`):

```c
for (i64 k = 0; k < E; k++) {
    DispatcherSlot *d = disp(dref.idx);             /* P1/P4 */
    i64 n = d->handlers.len;
    if (n == 0) panic("integer division by zero");
    Callable c = d->handlers.ptr[k % n];            /* in range by construction */
    c.fn(c.state, k, k % 1000);                     /* indirect call */
}
static void handler_fn(Ref st, i64 k, i64 v) {
    HandlerState *h = hstate(st.idx);               /* P6 */
    h->counter += v * h->m;
    cell(h->total.idx)->value += 1;                 /* P4: main's cell */
    if (k % 100000 == 99999) {                      /* rare path */
        h->pins += 1;                               /* O5: pinned before the call that may destroy it */
        dispatcher_unregister(disp(h->d.idx), st);  /* removed now, destroy deferred */
        dispatcher_register(disp(h->d.idx), (Callable){handler_fn,
                            make_handler(h->d, h->total, (k % 17) + 1)});
        h->pins -= 1;
        if (h->pins == DOOMED) hstate_free_now(st.idx);
    }
}
```

**Registry `get` and a later use**:

```c
PosRef r = { (PosSlot *)pool->slots.base + (sparse_get(pool, e) - 1), e };  /* after the absent check */
...                                                  /* emplace may run: slots never move */
if (r.slot->owner != r.e) panic_stale(...);          /* G1: the check at a use no proof covers */
r.slot->v.x += 1.0f;
```

**A11** compiles to pointer copies: `snapshot = items` copies an 8-byte reference to the list
object; `v.append(template)` appends an 8-byte reference; nothing is counted or copied deeply.

**A25**: `words.append(w)` stores `{lower_ref, start, len}`; the loop that reads the words needs no
check (P2: `lower` is an owning local of this function, not reassigned).

**A32**, `fire`:

```c
ui_list->iterating++;
i64 n0 = ui_list->len;                               /* L1 */
for (i64 i = 0; i < n0; i++) {
    Callable c = ui_list->ptr[i];                    /* buffer re-read: a handler may append (L5) */
    if (c.fn == NULL) continue;                      /* tombstone (L3) */
    c.fn(c.state, self);
}
if (--ui_list->iterating == 0 && ui_list->has_tombstones) compact(ui_list);
```

---

## 5. The C models for measurement

Files: `target/redesign/bench/D2/w1.c`, `w2.c`, `w3.c`, `w4.c`; each includes `../harness.h`,
calls `h_start()` first and `h_report("D2-wN", checksum)` last, and is self-contained. Each was
built once with `"C:/Program Files/LLVM/bin/clang.exe" -O2 wN.c -o wN.exe` (no warnings) and run
once on mains power (Win32_Battery status 2) to check that it finishes and prints its line. The
times those lines print are not reported here: one unpinned run each, and timing is the measurer's
job.

| model | checksum (single check run) | naive reference checksum | peak_kb printed by that run |
|---|---|---|---|
| D2-w1 | 2111831520 | 2111831520 | 42432 |
| D2-w2 | 647034814 | 647034814 | 77152 |
| D2-w3 | 9288233970 | 9288233970 | 9872 |
| D2-w4 | 34946993000 | 34946993000 | 4952 |

The naive references (plain C arrays and pointers; W1 keeps components in arrays indexed by id, W3
keeps targets as ids with a liveness array) were in the session scratchpad, not in the repo; they
exist only to check that each model does the specified work.

What each model contains, and what it leaves out:

- **Everything D2 needs**: store slots with generations, 8-byte references `{slot, generation}`,
  reserved address ranges committed in 1 MiB steps, free lists threaded through dead slots, slot
  retirement at generation exhaustion (W3, W4), the bounds checks no proof removes (W1 sparse
  lookups, W2 random `nodes[...]` indexing, W4 division by zero), the run-time generation checks no
  proof removes (W3's target checks), the L1-L3 loop bookkeeping (W1's cursor, W3's tombstones and
  compaction), O5's pin on W4's rare path, every `panic` the library would raise, and dynamic
  dispatch where the program has it (W1's `destroy` over each pool's interface table, W3's virtual
  `update`, W4's callable values).
- **Proofs applied**, each defined in 1.6: P1, P3, P4, P5, P6. Without P3, W2 would add one
  generation compare per parent hop; without P4, W4 would add one per event (on the cell, always in
  cache).
- **Left out, the same for every design**: Ember's integer overflow checks (`[TYP-8]`), which are
  not part of the memory model; the teardown of `main`'s locals, which runs after `h_report`.
- **Slot sizes, by construction** (`sizeof` checked): W1 `PosSlot`/`VelSlot` 16 bytes (8-byte owner
  id + 8-byte component); W2 `NodeSlot` 40 bytes; W3 `ObjSlot` 40 bytes; W4 handler state 40 bytes
  and a callable value 16 bytes; every object reference 8 bytes.

---

## 6. Sharp edges left, and what I expect to be slower than C

### 6.1 What would still surprise a Python programmer

1. **One owner.** An object dies when its owner lets go, even if other references remain: they read
   `None` or panic. Python keeps it alive. The trap that will bite most: replacing an owning list by a
   filtered copy, `objects = [o for o in objects if o.health > 0]`: the new list holds references,
   the old owning list is dropped, and every object dies. The compiler can recognise this pattern
   (an owning list replaced by a list of references to its own elements) and refuse it with the fix
   `objects.retain(...)`, but variations will slip through to a run-time panic.
2. **`own` must be written** on owning fields and element types other than locals and fields
   initialised with a new object.
3. **Loops do not visit what they add** (Python's list iterator does), and inside a loop that removed
   from a list, indexing that list sees the old positions (a removed position reads as a tombstone)
   until the loop ends.
4. **A reference to a plain value in a contiguous list is a position**, not an identity: after
   `remove(0)` it names the next element. Identity needs `Store[T]` or a class.
5. **A stale use panics at the use**, possibly far from the destroy that caused it; the message names
   both (the destroying line only in the debug profile).
6. **Memory stays with its store**: a class whose population spikes keeps that memory for its own
   later objects; an open hierarchy's slots are as large as its largest class.
7. **Ownership cycles** (`a.child = b` then `b.child = a`, both `own`) leak; the debug profile reports
   them at exit, as `[WK-15]` does for count cycles today.
8. **Strings become immutable** (a builder builds them), a change for Ember.
9. **Threads**: references do not cross; a moved object is relocated and the sender's references go
   stale.
10. **Destruction can be later than the removal** (O5): a `drop` with side effects runs when the
    removed object's running call returns.
11. **Address reservation** is needed for the one-instruction deref; 32-bit and WASM targets fall back
    to pages, one more load per use.
12. **Never-reused entity ids** cost each pool's sparse array 4 bytes per id ever issued, so a
    long-running game that spawns steadily grows them, unless sparse pages whose ids are all dead are
    released (a page with one long-lived entity stays).

### 6.2 Expected costs against plain C (estimates for the measurer to check, not measurements)

None of these was timed. They are reasoning from the generated code and the slot sizes above, and the
measurer's numbers replace them.

- **W1**: per visited entity D2 touches a 16-byte lead slot, a 4-byte sparse entry and a 16-byte
  position slot, about 36 bytes, plus a tombstone test and the sparse bounds test. An EnTT-style ECS
  with 32-bit entities touches about 24 (a 4-byte packed entity, two 8-byte components, a 4-byte
  sparse entry), and a C program that keeps 8-byte components in arrays indexed by id touches 16 per
  entity it updates but must also find which ids have both components. I expect D2 to be somewhat
  slower than both where the loop is memory-bound, by the extra bytes (the 8-byte owner words and the
  sparse probe): my estimate is up to about 1.5x against a tight C program. Memory, by construction
  from the sizes a scratch copy of the model printed (1,856,653 ids issued, 1,000,000 position and
  500,000 velocity slots): the slots take 24.0 MB, of which the owner words are 12.0 MB, and the two
  sparse arrays 14.9 MB (2 x 1,856,653 x 4 bytes), against the 42,432 KB peak the check run printed.
  The sparse arrays grow with every id ever issued, because ids are never reused (6.1, item 12).
- **W2**: the same pointer chase as C plus one `lea` per hop (seen in the assembly), and 40-byte
  nodes against roughly 32 in a C node with a parent pointer and a child array, so more cache lines
  on the random walk. I expect 0 to 20% slower. No generation check runs (P3).
- **W3**: a virtual call and one generation compare per Ship or Turret update; the compare reads the
  target's slot, whose `x` and `y` the update reads anyway, so it adds a compare, not a miss. A C
  program must also do something for "no target from then on" (a scan, back-references or ids); mine
  does nothing at destroy time. I expect W3 within 10% of C, possibly faster than a C version that
  clears targets on destroy.
- **W4**: per event, the same work as a C program with a function pointer and a context pointer,
  plus turning two slot numbers into addresses (the handler state and the `total` cell) and a 16-byte
  callable element instead of 8 or 16. The 64-bit division `k % n` dominates both. I expect 0 to 15%
  slower.
- **Where D2 would be slow, outside these workloads**: code that uses many references the proofs
  cannot cover, for example random walks over a graph whose nodes are destroyed during the walk;
  each use then pays the phase-2 measured cost of a generation check, +0.57 ns in order and +4.61 ns
  random per use against a plain pointer, of which D2's 8-byte references remove the part due to
  16-byte references (`phase2-vale-austral-pony.md`, section 6). Loops that change the list they
  iterate pay a reload per iteration and lose vectorisation (L5).
- **Against Ember today**: no count updates anywhere, no access words, and a 4-byte rather than
  24-byte header; phase 2 measured a non-atomic count pair at about 0.84 ns per element visit
  (`phase2-rc-elision.md`, section 2.3), which D2 never pays.

### 6.3 Open questions for the owner (not ruled here)

1. The `own` word: written on owning fields (this design), or inferred per field from how the
   program fills it (no word, but an inference that can surprise)?
2. Despawn and spawn inside a loop take effect at once (this design) rather than at the loop's end:
   confirm.
3. Entities as never-reused 64-bit serials (this design) rather than EnTT's index and version.
4. Immutable strings.

---

## Sources

- Phase reports in this folder: `phase1-borrowing.md`, `phase1-sharing.md`,
  `phase2-vale-austral-pony.md` (generational-reference measurement, section 6; Vale's +10.84%,
  section 1.3), `phase2-hylo-csharp.md` (paged storage and handle measurements, Part 3),
  `phase2-rc-elision.md` (counted-object and count-pair measurements, section 2.3),
  `phase2-swift-mojo.md`.
- Spec: `docs/spec-source/Ember_v0.9.10_Hardened_35.md`, rules cited by id.
- ODR-286: `tasks/95/odr-pending.md`; the EnTT design: `tasks/95/ecs-entt-design.md`; probes
  `target/analysis/ecs4/*.em`, `target/analysis/ecs5/*.em`; phase 1 probes
  `target/redesign/phase1/borrowing/e11_moves.em`, `e06e_local_array_of_views.em`,
  `target/redesign/phase1/sharing/a32_reentrant_callback.em`.
- The models: `target/redesign/bench/D2/w1.c`-`w4.c` and the shared `target/redesign/bench/harness.h`;
  clang 22.1.8 (`clang --version`).
- Tools: no tool was missing for this task. WebSearch and WebFetch were not needed: every external
  fact comes through a phase-2 report that names its source, or is marked UNVERIFIED.
