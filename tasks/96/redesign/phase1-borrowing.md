# Phase 1, borrowing: the catalogue (A1–A8, A11, A17, A18, and new A21–A25)

Written 2026-10-24 by the borrowing research agent. Every Rust error below is real output of
`rustc 1.98.1 (48a229cea 2026-09-01)` run as `rustc --edition 2021 --crate-type lib <file>`; every
Ember result is real output of `target/ci/ember.exe check <file>` (type and borrow checking only; no
Ember program was run). The programs and the full outputs are in
`target/redesign/phase1/borrowing/`:

- Rust programs `a*.rs`; outputs `rustc_output.txt`, `rustc_output2.txt`
- Ember programs `e*.em`; outputs `ember_output.txt`, `ember_output2.txt`
- the Python behaviour checks `py_checks.py` (run with `python -I`, Python 3.14 on this machine)

Error texts below are trimmed to their message, labels and help lines; nothing is quoted from memory.
Costs in the "eliminations" sections are reasoning from the operations involved (a compare, a load, a
count word), **not measurements**, unless a file is named. Frequency claims name their source; where
none was found the claim is marked UNVERIFIED.

## Summary

"Ember today" compares Ember with Rust on the same program: **same** = Ember refuses it too;
**partly** = Ember accepts some forms, or accepts it with a run-time check; **absent** = Ember accepts
the natural program; **worse** = Ember refuses something Rust accepts, or has a hazard Rust does not.

| Id | Annoyance | Severity (Python programmer) | Ember today | Evidence (Ember file → result) |
|---|---|---|---|---|
| A1 | Borrowing the whole object | high | partly | `e01*.em`: struct methods accepted ([BRW-10]); class method with `mut self` → E3020 |
| A2 | Container invalidation | high | same | `e02_container_invalidation.em` → E3022; Python spelling `boss = enemies[0]` → E3040 |
| A3 | Two mutable references into one container | medium | same | `e03_two_elements.em` → E3022 (help names `get_pair_mut`, which compiles) |
| A4 | Changing a collection while iterating it | high | same | `e04*.em` → E3020 (struct and class); ECS class pools via `cast` compile and leave it to a run-time check |
| A5 | Returning references; lifetimes; conditional borrows | medium | partly | no annotations, NLL case #3 accepted; ECS pool via `cast` → E3060 where Rust accepts |
| A6 | Structs holding references | medium | partly | `e06_struct_refs.em` accepted with no lifetime syntax; a view struct in an `Array` → E3063 |
| A7 | Self-referential structures | medium | same | `e07_self_ref.em` → E3063 (and [LT-25] E3024 behind it) |
| A8 | Graphs, parent pointers, cycles | high | partly | classes + `Weak` compile; strong cycles leak with a warning; `Option[ref Node]` **crashes the checker** |
| A11 | Moves, use after move, clone | high | same | `e11_moves.em` → E3040, E3042, E3041; `snapshot = items` moves; classes avoid it |
| A17 | Error messages that do not say what to do | medium | partly | Ember's helps are mostly concrete; four misleading ones found |
| A18 | Initialisation, two-phase init | low | partly | `e18*.em` → E2100 (field defaults soften it) |
| A21 (new) | Updating a map entry in place | low | partly | `counts[w] += 1` and `if w in counts` compile; `get(w).unwrap_or(0)` → E2020; `entry(..).or_default() += 1` → E2140 |
| A22 (new) | A view of a temporary kept past its statement | medium | same | `e22b_temporary.em` → E3060 |
| A23 (new) | Moving a field out from behind `mut self` (state machines) | medium | same | `e23_move_out_of_field.em` → E3013 |
| A24 (new) | struct, `Box` or class changes what compiles and what panics | high | worse | ODR-286 probes: class pools → E3060, `Box` pools compile; class loops → run-time check |
| A25 (new) | A collection of views (a list of words) refused even locally | high | worse | `e06e_local_array_of_views.em` → E3063; the Rust twin compiles |

Defects found in passing (for the lead, not fixed here; no compiler file was touched):

1. **Checker crash:** `struct Node: parent: Option[ref Node]` makes `ember.exe check` overflow its stack
   (`e08d_ref_self_type.em`, `e08_tree_struct.em`: `thread '<unknown>' has overflowed its stack`, exit 127).
2. `boss = enemies[0]` (move out of an `Array` element) reports E3040 "`enemies` is borrowed after it
   has been moved out of" (`e02b_container_python_spelling.em`); `[EXP-6]` says moving out of an
   `Array` element is `E3011`.
3. `counts.entry(k).or_default() += 1` is E2140 "assignment target is not a place" though
   `or_default` returns `ref mut V` (`e21_map_update.em`); binding it to a local first compiles
   (`e21c_map_update.em`).
4. `L3001` on a tree whose parent is already `Weak` tells the user to make `Node.children` weak, the
   owning edge (`e08b_tree_class.em`).
5. `[TYP-15a]`'s `BorrowList`/`ViewList` (the one sanctioned container of views) are not implemented
   (stated in `tests/conformance/LT-2/reject_mutating_either_source_of_a_view.em`; no `BorrowList`
   in `std/src/`).
6. `[BRW-10]` says a method visible outside its package borrows all of `self`; `e01b_whole_object_pub.em`
   (a `pub` method called from its own package) is accepted. Probably intended (the rule protects
   other packages' callers), but the spec text does not say so; the cross-package case was not tested.

What the measurements say overall: Ember already removed the worst of Rust's *syntax* (no lifetimes,
no `&`/`&mut` at call sites, field-precise private methods, problem case #3). What is left is Rust's
*rules*: invalidation (A2, A4), exclusivity (A3), moves (A11), views that cannot be stored (A25, A7),
plus one problem Rust does not have: two memory models whose rules differ (A24). The eliminations
that recur below are: storage that does not move (pages, as EnTT does), references that are paths
re-resolved on use, deferred structural changes during a loop, automatic temporary extension,
copy-on-write collections, arena-owned graphs, and using the fact that Ember panics abort (`[PAN-1]`)
so exception safety rules are not needed.

---

## A1. Borrowing the whole object

**(1) Rust.** A game tick that logs a death while iterating the players
(`a01_whole_object.rs`):

```rust
impl Game {
    fn note(&mut self, msg: String) { self.log.push(format!("[{}] {}", self.frame, msg)); }
    pub fn tick(&mut self) {
        for p in self.players.iter_mut() {
            p.hp -= 1;
            if p.hp <= 0 { self.note(format!("{} died", p.name)); }
        }
    }
}
```

```text
error[E0499]: cannot borrow `*self` as mutable more than once at a time
10 |         for p in self.players.iter_mut() {
   |                  ----------------------- first mutable borrow occurs here
13 |                 self.note(format!("{} died", p.name));
   |                 ^^^^ second mutable borrow occurs here
```

No help line. Workaround (`a01_whole_object_fix.rs`, compiles): make `note` a free function taking
`&mut self.log` and `self.frame` separately; or inline it; or split the struct.

**(2) Python.**

```python
def note(self, msg): self.log.append(f"[{self.frame}] {msg}")
def tick(self):
    for p in self.players:
        p.hp -= 1
        if p.hp <= 0: self.note(f"{p.name} died")
```

**(3) Ember today: partly.** `e01_whole_object.em` (struct, private `note(mut self)`) and
`e01b_whole_object_pub.em` (`pub`) are accepted: `[BRW-10]` computes the fields a method uses.
`e01c_whole_object_class.em` (class, plain `self`) is accepted: class field accesses are checked per
field at run time (`[EXC-19]`). But `e01d_whole_object_class_mut.em`, the same class with
`fn note(mut self, …)`, is refused:

```text
error[E3020]: cannot mutate `self.players` while it is borrowed by this loop
15 |         for p in self.players:
18 |                 self.note(f"{p.name} died")
```

and by `[BRW-10]`'s text a `pub` method called from another package borrows all of `self`
(UNVERIFIED in practice: needs two packages).

**(4) Frequency and severity.** The partial-borrowing request, rust-lang/rfcs issue #1215 "Partial
borrowing (for fun and profit)", has been open since 18 July 2015
(https://github.com/rust-lang/rfcs/issues/1215). Niko Matsakis's "After NLL: Interprocedural
conflicts" (2018) uses this exact shape (a loop over `self.widgets` calling `self.signal_event()`) and
lists inlining, factoring, free functions and view structs as the workarounds
(https://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/). The
LogLog Games post-mortem names it among the reasons the authors left Rust game development: being
told off for passing `self` around while also borrowing a field on `self`
(https://loglog.games/blog/leaving-rust-gamedev/). No count found. **Severity: high**: game and
GUI code is objects calling their own helpers.

**(5) The problem the rule prevents.** `note` takes all of `self`, so as far as the caller knows it
could `self.players.push(…)`, reallocating the buffer the loop is walking; the iterator would then read
freed memory.

- **(a) Pythonic spelling of the same guarantee: infer the field set, everywhere.** Ember already does
  it for private struct methods. Extend it to `pub` methods by writing the inferred field set into the
  package's interface file, and to class methods with `mut self` by giving `mut self` the same
  inferred field set instead of "every field".

  ```ember
  pub fn note(mut self, msg: String):      # interface records: writes {log}, reads {frame}
      self.log.push(f"[{self.frame}] {msg}")
  ```

  Cost: none at run time; one summary per method at compile time. Gives up: a body change that
  touches a new field changes the method's interface (a semver change); a tool must report it.
  Dynamic dispatch (`virtual`, `dyn`) must stay "all fields".
- **(b) Smarter: the loop cannot be invalidated.** Make `for p in self.players` iterate by index over a
  length re-read each step, and yield the element as a path (`self.players[i]`) rather than a pointer
  (see A2(a)). A `push` inside the loop is then harmless: nothing points into the old buffer. Cost:
  one bounds check and a length reload per iteration (a load and a compare), which can block
  vectorisation of that loop; loops that provably do not mutate keep the pointer form. Gives up: the
  compile-time warning that the loop changes what it iterates (Python has no such warning either).

## A2. Container invalidation

**(1) Rust.** A boss splits off a minion (`a02_container_invalidation.rs`):

```rust
pub fn split_boss(enemies: &mut Vec<Enemy>) {
    let boss = &mut enemies[0];
    enemies.push(Enemy { x: boss.x + 1.0, hp: boss.hp / 2 });
    boss.hp /= 2;
}
```

```text
error[E0499]: cannot borrow `*enemies` as mutable more than once at a time
5 |     let boss = &mut enemies[0];
  |                     ------- first mutable borrow occurs here
6 |     enemies.push(Enemy { x: boss.x + 1.0, hp: boss.hp / 2 });
  |     ^^^^^^^                 ------ first borrow later used here
```

Workaround (`a02_container_invalidation_fix.rs`): copy the fields out, push, index again. In the ECS:
`a05_ecs_misuse.rs`, holding `reg.get_mut::<i32>(0)` across `reg.emplace(…)`, is the same E0499.

**(2) Python.** `boss = enemies[0]; enemies.append(Enemy(boss.x + 1, boss.hp // 2)); boss.hp //= 2`.
`boss` is an alias of the object, so it stays right.

**(3) Ember today: same.** `e02_container_invalidation.em`:

```text
error[E3022]: `*enemies[0]` is already mutably borrowed
7 |     boss = ref mut enemies[0]
8 |     enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp - 10))
  = help: end the borrow before this: `boss` is what keeps it alive, so shorten its last use or put it in a block of its own
```

The Python spelling, `boss = enemies[0]`, is a move (`e02b_container_python_spelling.em`, E3040 with
the note "assigning a list, a map or a set moves it; in Python both names would point at one list").
With class elements the program compiles (`e02c_container_class.em`): `boss` is a counted handle,
at the cost of a heap block, a 24-byte header (`[OBJ-1]`) and counting per object.

**(4) Frequency and severity.** EnTT pages its pools so that component addresses survive insertion
(`tasks/95/ecs-entt-design.md` point 4); ODR-286 records the owner hitting this in the ECS
(`tasks/95/odr-pending.md`). C++ iterator invalidation is the hazard behind it. No count found for
Rust. **Severity: high.**

**(5) The problem.** `push` may reallocate the array; `boss` would then point into freed memory and
`boss.hp /= 2` would write into it (a use-after-free).

- **(a) Pythonic: a reference is a path, not a pointer.** `boss = enemies[0]` binds the *place*
  `enemies[0]`; every use re-evaluates it (bounds check against the current buffer).

  ```ember
  boss = enemies[0]                        # a place alias, not a move and not a pointer
  enemies.push(Enemy(x=boss.x + 1.0, hp=boss.hp - 10))
  boss.hp -= 10                            # re-indexes enemies[0] in the current buffer
  ```

  Cost: per use, a reload of the buffer pointer and a bounds check; the compiler caches the pointer
  between uses where nothing can change the array (the usual case), so straight-line code is C. Gives
  up: after `remove(0)` the alias names whatever is now at index 0, which is not Python's meaning
  (Python's alias follows the object). For `push` alone the meaning is Python's.
- **(b) Smarter: storage that never moves.** A paged array (fixed-size pages, a page table): `push`
  adds a page and never moves an element, so a pointer into it stays valid until that element is
  removed; removal is the only structural change left to police (with a generation check, as
  `Pool[T]`/`Handle[T]` do, `[HND-1]`). This is EnTT's design. Cost: random access goes through the
  page table (one more load); iteration runs page by page at array speed; memory wastes up to one
  partly filled page per container plus the table. Gives up: one contiguous `Span` over the whole
  container (a `Span` per page instead), so whole-array SIMD and `as_span()` change.

## A3. Two mutable references into one container

**(1) Rust.** Two bodies collide (`a03_two_elements.rs`):

```rust
pub fn step(bodies: &mut Vec<Body>, i: usize, j: usize) {
    collide(&mut bodies[i], &mut bodies[j]);
}
```

```text
error[E0499]: cannot borrow `*bodies` as mutable more than once at a time
8 |     collide(&mut bodies[i], &mut bodies[j]);
  = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
```

Workaround (`a03_two_elements_fix.rs`, compiles): `bodies.get_disjoint_mut([i, j])` or
`split_at_mut`.

**(2) Python.** `collide(bodies[i], bodies[j])`.

**(3) Ember today: same.** `e03_two_elements.em`:

```text
error[E3022]: `*bodies[…]` is already mutably borrowed
10 |     collide(bodies[i], bodies[j])
   = help: use `bodies.get_pair_mut(i, j)` for two elements at once (`None` when they are the same), or `bodies.as_mut_span().split_at(k)` …
```

`e03b_two_elements_pair.em` with `get_pair_mut` compiles. The help is better than Rust's; the natural
spelling is still refused.

**(4) Frequency and severity.** Rust 1.86 (2025-04-03) added `get_disjoint_mut` on slices and
`HashMap` "to safely support this pattern" (https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/).
**Severity: medium**: common in physics and graph code, and the fix is one call.

**(5) The problem.** With `i == j`, `a` and `b` are the same body: code written for two bodies
(`a.v += x; b.v -= x`) silently does the wrong thing, and an optimiser that assumed the two do not
alias (`noalias`) can produce wrong results.

- **(a) Pythonic: the compiler inserts the check.** Accept `collide(bodies[i], bodies[j])` and emit
  `if i == j: panic(…)` before the call, exactly what `get_pair_mut` checks.

  ```ember
  collide(bodies[i], bodies[j])     # compiled as: assert i != j; then two disjoint references
  ```

  Cost: one compare and a predicted branch per call. Gives up: a program that wants `i == j` handled
  must test for it itself (a panic otherwise, which the owner's rules allow).
- **(b) Smarter: aliasing plain data is not a memory-safety hazard.** Two writable references to the
  same plain-data element cannot free anything (the container is held, nothing can reallocate it), so
  memory safety holds without exclusivity; only the `noalias` optimisation and single-thread-ness
  depend on it. Ember already does this for `Copy` fields of class objects (`[EXC-17]`). Cost: no
  check; the backend loses `noalias` on such parameters, which can cost vectorisation in some loops.
  Gives up: the "two bodies are distinct" logic guarantee; a data-race rule is still needed across
  threads.

## A4. Changing a collection while iterating it

**(1) Rust.** Despawn dead enemies and spawn their splits in the update loop
(`a04_mutate_while_iterating.rs`):

```rust
for (i, e) in enemies.iter().enumerate() {
    if e.hp <= 0 {
        if e.splits { enemies.push(Enemy { hp: 5, splits: false }); }
        enemies.remove(i);
    }
}
```

```text
error[E0502]: cannot borrow `*enemies` as mutable because it is also borrowed as immutable
5 |     for (i, e) in enemies.iter().enumerate() {
8 |                 enemies.push(Enemy { hp: 5, splits: false });
error[E0502]: … (the same at `enemies.remove(i)`)
```

Workaround (`a04_mutate_while_iterating_fix.rs`): collect spawns in a second list, then
`retain(|e| e.hp > 0)` and `extend`.

**(2) Python.** Python allows it and gets it wrong: removing from a list while iterating skips the
next element (`py_checks.py`: `[0, 0, 5]` becomes `[0, 5]`), and a dict that grows during iteration
raises `RuntimeError: dictionary changed size during iteration` (same file). The Python tutorial
advises looping over a copy or building a new collection (https://docs.python.org/3/tutorial/controlflow.html,
section 4.2). So a Python programmer expects the syntax to work, not the semantics.

**(3) Ember today: same.** `e04_mutate_while_iterating.em` and the class version
`e04b_mutate_while_iterating_class.em` are both:

```text
error[E3020]: cannot mutate `*enemies` while it is borrowed by this loop
  = help: collect the indices or values first, then mutate; use `retain` or `drain` when the operation fits
```

The workaround `e04c_mutate_while_iterating_fix.em` compiles. But
`target/analysis/ecs4/accept_change_during_loop.em` (pools as class objects found by a `cast` of a map
lookup) **compiles**: the conflict is left to the run-time access check (`[EXC-18]`, `[EXC-1]`), which
by the spec panics when the loop pushes (UNVERIFIED: running it is not allowed here).

**(4) Frequency and severity.** EnTT allows removing the current entity's components during a view
and warns that adding may invalidate (`tasks/95/ecs-entt-design.md` point 3); LogLog Games reports
crashes from overlapping ECS queries at run time (https://loglog.games/blog/leaving-rust-gamedev/).
**Severity: high**: spawn/despawn in the frame loop is the core of a game.

**(5) The problem.** `push` can reallocate the buffer the loop walks (use-after-free); `remove` shifts
the elements so the loop skips one (the logic bug Python shows).

- **(a) Pythonic: structural changes to the iterated container are deferred to the loop's end.**
  Written exactly as Python; inside a `for` over `enemies`, `enemies.push` and `enemies.remove(e)`
  go to a queue applied when the loop exits (EnTT/Bevy "commands", built into the language).

  ```ember
  for e in enemies:
      if e.hp <= 0:
          if e.splits:
              enemies.push(Enemy(hp=5, splits=false))   # applied after the loop
          enemies.remove(e)                             # applied after the loop
  ```

  Cost: only loops that mutate their own container get the queue (the compiler sees it): one small
  buffer, allocated on first use; other loops are unchanged. Gives up: spawned elements are not
  visited in the same loop (Python would visit appended list items); reads of `enemies.len()` inside
  the loop see the old length (must be documented).
- **(b) Smarter: tombstones and stable pages.** With paged storage (A2(b)), iterate `0..len_at_start`;
  `remove` marks a slot dead (skipped by iteration) and compaction happens at a safe point (loop end,
  frame end); `push` appends to a page that never moves. Cost: a liveness test per element
  (a bit test), a compaction pass, a bit per slot. Gives up: dense order changes at compaction.

## A5. Returning references; lifetime annotations; conditionally returned borrows

**(1) Rust.** Three programs.

(i) `a05_return_refs.rs`, `fn longer(a: &str, b: &str) -> &str`:

```text
error[E0106]: missing lifetime specifier
2 | pub fn longer(a: &str, b: &str) -> &str {
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `a` or `b`
help: consider introducing a named lifetime parameter: pub fn longer<'a>(a: &'a str, b: &'a str) -> &'a str
```

(ii) NLL problem case #3, an asset cache (`a05_conditional_return.rs`):

```rust
pub fn get_or_load(cache: &mut HashMap<u32, String>, id: u32) -> &String {
    if let Some(s) = cache.get(&id) { return s; }
    cache.insert(id, format!("asset {}", id));
    cache.get(&id).unwrap()
}
```

```text
error[E0502]: cannot borrow `*cache` as mutable because it is also borrowed as immutable
5 |     if let Some(s) = cache.get(&id) {
6 |         return s;
  |                - returning this value requires that `*cache` is borrowed for `'1`
8 |     cache.insert(id, format!("asset {}", id));
```

Workarounds (`a05_conditional_return_fix.rs`): look up twice (`contains_key` then `get`), or the entry
API.

(iii) The ECS case of ODR-286 (`a05_ecs_get_mut.rs`): `get_mut<T>` downcasts a `Box<dyn Any>` found in
a `HashMap` and returns `&mut pool.dense[i]`. **Rust accepts it**; holding the result across an
`emplace` is refused (`a05_ecs_misuse.rs`, E0499).

**(2) Python.** `def longer(a, b): return a if len(a) >= len(b) else b`;
`def get_or_load(cache, id): return cache.setdefault(id, f"asset {id}")`;
`return self.pools[T].dense[i]` (a value, or an object alias).

**(3) Ember today: partly.** `e05a_longer.em` (no annotation, `[LT-1]` rule 3) and
`e05b_conditional_return.em` (problem case #3) are **accepted**. `e05c_class_returns_ref.em` (a class
method returning `ref mut self.dense[i]`) is accepted and its misuse `e05d…` refused (E3021). But the
ECS form with class pools (`target/analysis/ecs5/pos.em`) is refused where Rust accepts:

```text
error[E3060]: `pool.dense[…]` does not live long enough
20 |         pool = cast self.pools[7u128] as Pool[T]
   |         ---- `pool` is a local, so its storage ends with the frame
21 |         return ref mut pool.dense[i]
  = help: return an owned value, take the destination as a `mut` parameter, or borrow something the caller owns
```

and rule 1 ties a method's result to the whole receiver, so two words from one parser cannot be kept
(`e06b_two_words.em`, E3025) unless the method says `@borrows(self.src)` (`e06c…`, accepted). Rust's
elided form has the same over-borrowing (`a06c_two_words_elided.rs`, E0499) and the named `'a` form
does not (`a06b_two_words.rs`, accepted).

**(4) Frequency and severity.** Lifetimes were the hardest topic in the 2020 Rust survey: 61.4% rated
lifetime management tricky or very difficult (https://blog.rust-lang.org/2020/12/16/rust-survey-2020/).
Problem case #3 has been documented since RFC 2094 (https://rust-lang.github.io/rfcs/2094-nll.html)
and is still refused by rustc 1.98.1 (measured above); the 2026 Rust project goal says Polonius alpha
resolves it and is heading for stabilisation (https://rust-lang.github.io/goals/2026/polonius.html;
not stable on this machine's rustc). **Severity: medium** for Ember (most of it is already gone).

**(5) The problem.** A returned reference must not outlive what it points into, and the caller must
know which argument it borrows, so that it does not free or grow that argument while the reference
lives.

- **(a) Pythonic: infer what a result borrows from the body, and drop `@borrows`.** `[LT-22]` already
  does this for view types; do it for every function, record it in the interface, and treat the cast
  of a class-handle place as a projection that keeps the handle (the borrow of a class field already
  keeps its object alive, `[RC-5]`).

  ```ember
  fn next_word(mut self) -> str:         # inferred: borrows self.src, not self
      ...
  fn get_mut[T](mut self, i: int) -> ref mut T:
      return ref mut (cast self.pools[key] as Pool[T]).dense[i]   # a projection of self.pools
  ```

  Cost: compile time only. Gives up: a body change can change the interface (as A1(a)).
- **(b) Smarter: the referent cannot go away.** If pools live in stable storage owned by the registry
  for its whole life (a pool is never dropped or moved while the registry lives) and elements live in
  pages (A2(b)), a reference into a pool is valid until that element is removed; only removal needs a
  check (a generation compare, `[HND-1]`). The question "which argument does it borrow" then only
  matters for removal. Cost: as A2(b); pools are freed only with the registry. Gives up: dropping an
  empty pool early.

## A6. Structs holding references

**(1) Rust.** `a06_struct_refs.rs`, `pub struct Lexer { src: &str, pos: usize }`:

```text
error[E0106]: missing lifetime specifier
2 | pub struct Lexer { src: &str, pos: usize }
help: consider introducing a named lifetime parameter: pub struct Lexer<'a> { src: &'a str, pos: usize }
```

Workaround (`a06_struct_refs_fix.rs`): `'a` on `Lexer`, on `Parser` that holds it, on every `impl`,
and on the EnTT-style `Handle<'r> { reg: &'r mut Registry, entity: usize }`.

**(2) Python.** `class Lexer: def __init__(self, src): self.src = src; self.pos = 0`.

**(3) Ember today: partly.** `e06_struct_refs.em` (a `Lexer` with `src: str`, a `Parser` holding it,
and `Handle` with `reg: ref mut Registry`) is **accepted with no lifetime syntax**. But a view struct
cannot be put in a collection (`e06d_handles_in_array.em`):

```text
error[E3063]: `Handle` is a view, so it may not be stored in an `Array` unless it is `static`
  = help: store an owned copy … or store a `u32` index or a `Handle[T]` and name the container it indexes
```

(two `ref mut` to one registry would also be refused in Rust; the point is the rule, see A25).

**(4) Frequency and severity.** LogLog Games: lifetimes "become generics that poison every single
usage point" (https://loglog.games/blog/leaving-rust-gamedev/). **Severity: medium**: Ember already
removed the annotation; the storage rule remains.

**(5) The problem.** A struct holding a reference must not outlive the thing referred to.

- **(a) Pythonic: region-carrying containers, inferred.** Let an `Array[str]` local carry the regions
  of what it holds, as a view struct already does (`[LT-14]`); nothing is written.

  ```ember
  handles: Array[Handle] = []            # the array carries reg's region; inferred
  ```

  Cost: compile time. Gives up: such an array cannot be stored in a class field or a static (same as
  any view).
- **(b) Smarter: a stored reference keeps its target alive.** Where a view must outlive a frame, make
  it a counted reference (handle + offset), as class handles already are. Cost: a retain per store and
  a release per drop, 8 more bytes per view. Gives up: the target is pinned in memory while any view
  lives (a big string kept alive by one small word).

## A7. Self-referential structures

**(1) Rust.** A document that owns its text and keeps word slices (`a07_self_ref.rs`):

```rust
pub fn load<'a>(text: String) -> Document<'a> {
    let words = text.split_whitespace().collect();
    Document { text, words }
}
```

```text
error[E0515]: cannot return value referencing function parameter `text`
error[E0505]: cannot move out of `text` because it is borrowed
```

Workarounds: index ranges (`a07_self_ref_fix.rs`), or crates.

**(2) Python.** `self.text = text; self.words = text.split()`.

**(3) Ember today: same.** `e07_self_ref.em` stops even earlier, at storing views in an `Array`:

```text
error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
  = note: the view borrows `text`, and this place has no bounding region …
```

and a view struct borrowing a value moved into its own other field is `E3024`
(`tests/conformance/LT-25/reject_borrowing_an_owner_moved_into_the_same_view.em`).

**(4) Frequency and severity.** The two crates that exist to build self-referential structs: `ouroboros`
89,396,135 downloads and `self_cell` 81,296,771 downloads (crates.io API,
https://crates.io/api/v1/crates/ouroboros and …/self_cell, fetched 2026-10-24). **Severity: medium**:
rarer than A2/A4, but a parser/tokeniser hits it immediately.

**(5) The problem.** Moving the struct could move the data the views point at, and replacing `text`
frees the buffer the views point into.

- **(a) Pythonic: views into a field's heap buffer, with the field frozen.** A `String`'s characters
  live on the heap, so moving `Document` does not move them; the only hazard is changing `text`. The
  compiler can accept `Document(text=text, words=…)` when every view points into `text`'s heap buffer
  and then treat `text` as `let` and read-only for the struct's life.

  ```ember
  struct Document:
      text: String            # becomes read-only once `words` borrows it (inferred)
      words: Array[str]
  ```

  Cost: none at run time. Gives up: in-place edits of `text` (rebuild the document instead); inline
  data (a fixed array field) cannot be borrowed this way, since it moves with the struct.
- **(b) Smarter: shared immutable strings.** Make `String` contents immutable and counted (Python's
  `str` is immutable); a word is (buffer, offset, length) and keeps the buffer alive, so there is no
  self-reference problem at all. Cost: a count word per buffer, a retain per stored word, 8 more bytes
  per word; mutation needs a builder type. Gives up: in-place string mutation as the default.

## A8. Graphs, trees with parent pointers, doubly linked lists, cycles

**(1) Rust.** A scene tree whose nodes point at their parent (`a08_tree_parent.rs`):

```rust
let mut root = Node { name: "root".into(), parent: None, children: vec![] };
let leaf = Node { name: "leaf".into(), parent: Some(&root), children: vec![] };
root.children.push(leaf);
root
```

```text
error[E0502]: cannot borrow `root.children` as mutable because it is also borrowed as immutable
error[E0505]: cannot move out of `root` because it is borrowed
error[E0515]: cannot return value referencing local variable `root`
```

Workarounds (`a08_tree_parent_fix.rs`): `Rc<Node>` with `RefCell<Weak<Node>>` parents and
`RefCell<Vec<Rc<Node>>>` children (`borrow_mut`, `upgrade`, `Rc::downgrade`, `Rc::clone`), or an arena
of nodes linked by index.

**(2) Python.** `child.parent = self; self.children.append(child)`. Python's cycle collector frees the
cycle.

**(3) Ember today: partly.** Classes with a `Weak` parent compile (`e08b_tree_class.em`), with a
warning whose advice points at the wrong edge:

```text
warning[L3001]: potential reference cycle: Node.children -> Node
  = help: if `Node.children` is a back-reference, make it hold a `Weak` handle
```

A doubly linked list with strong `next` and `prev` compiles with the same warning and leaks at run
time (`e08c_dlist_class.em`; `[WK-1]`: a strong cycle "leaks. This is a documented property").
The plain-struct tree with `parent: Option[ref Node]` **crashes the checker**
(`e08_tree_struct.em`, minimised to `e08d_ref_self_type.em`: stack overflow).

**(4) Frequency and severity.** "Learning Rust With Entirely Too Many Linked Lists" exists because of
it and calls doubly-linked lists "kinda gnarly" (https://rust-unofficial.github.io/too-many-lists/);
LogLog Games observes that much of ECS's appeal in Rust is that entity ids are `Copy` and sidestep
the borrow checker (https://loglog.games/blog/leaving-rust-gamedev/). **Severity: high**: scene graphs,
UI trees and ASTs with parents are everyday code.

**(5) The problem.** A back-pointer must not dangle when its target is freed, and a cycle of owners
must not leak or be freed twice.

- **(a) Pythonic: the back edge is inferred weak, and read without ceremony.** When a field of type
  `Node` is assigned from `self` (`child.parent = self`) or the compiler's ownership graph (`[WK-5]`)
  shows it closes a cycle, it is stored weak; reading `child.parent` upgrades it and panics if the
  parent is gone (`Option[Node]` fields give `None` instead).

  ```ember
  class Node:
      name: String
      children: Array[Node] = []
      parent: Option[Node] = None      # inferred weak: it closes Node.children -> Node
  ```

  Cost: the weak count already in the header (`[OBJ-1]`), an upgrade (load, compare) per read.
  Gives up: an inference that guesses wrong silently changes lifetime; the rule must be exact and
  reported by `ember inspect`.
- **(b) Smarter: arena-owned graphs.** Nodes are allocated in a graph-owned region; edges between
  nodes of one region are plain pointers valid for the region's life; the region frees everything at
  once (`Arena`, `[ARN-1]`, but with references between its own allocations allowed). No counts, no
  cycles problem, C speed. Cost: memory of deleted nodes is reclaimed only with the region (or by a
  free list per type). Gives up: freeing single nodes eagerly; a node cannot outlive its graph.
  A second smarter option is a cycle collector for class objects (CPython's approach): costs a
  periodic scan and a word per object, and gives up deterministic destruction for objects on cycles.

## A11. Moves: use after move, clone everywhere, partial moves

**(1) Rust.** `a11_moves.rs`:

```rust
pub fn notify(m: Msg) { send(m); log(&m); }
pub fn rename(p: Player) { let n = p.name; println!("{}", n); keep(p); }
pub fn spawn_all(template: Player, n: usize) -> Vec<Player> {
    let mut v = Vec::new(); for _ in 0..n { v.push(template); } v
}
```

```text
error[E0382]: borrow of moved value: `m`        (note: if `Msg` implemented `Clone`, you could clone the value)
error[E0382]: use of partially moved value: `p`
error[E0382]: use of moved value: `template`    (value moved here, in previous iteration of loop)
```

Workaround: `.clone()`, `#[derive(Clone)]`, borrowing parameters.

**(2) Python.** All three just work: names are references. `snapshot = items; items.append(3)` makes
`snapshot` see the 3 (`py_checks.py`: `[1, 2, 3]`).

**(3) Ember today: same** for structs and collections (`e11_moves.em`):

```text
error[E3040]: `m` is borrowed after it has been moved out of
error[E3042]: `p` is partially moved
error[E3041]: `template` is moved in a loop
error[E3040]: `items` is borrowed after it has been moved out of
  = note: assigning a list, a map or a set moves it; in Python both names would point at one list (Appendix E)
```

With classes the same program compiles (`e11b_moves_class.em`): handles are `Copy`. Calls never write
a mode (`[FN-2a]`), so moves come only from `owned` parameters and assignment.

**(4) Frequency and severity.** The Rust project has run an "ergonomic ref-counting" goal since 2024H2
to reduce the clone noise of `Rc`/`Arc` (https://rust-lang.github.io/goals/2026/ergonomic-rc.html);
rustc's own notes above suggest `clone` for each case. **Severity: high**: `b = a` meaning a move is
the most un-Pythonic thing in the catalogue.

**(5) The problem.** Two owners of one heap buffer would both free it (double free); a moved-from
variable would read memory its new owner may have freed.

- **(a) Pythonic: copy-on-write collections.** `snapshot = items` shares the buffer and counts it;
  a write to a shared buffer copies first (Swift's arrays). No moves to get wrong, no double free.

  ```ember
  snapshot = items        # shares; count 2
  items.push(3)           # items copies its buffer first; snapshot keeps [1, 2]
  ```

  Cost: a count word per buffer; on every mutating call a uniqueness test (load, compare, predicted
  branch); a real copy only when shared. Gives up: Python's alias meaning (snapshot does not see the
  3), and the cost of a copy is hidden at the write.
- **(b) Smarter: a move at the last use, a copy elsewhere, reported.** The compiler moves a value at
  its last use and copies (clones) at an earlier use, so `send(m); log(m)` copies at `send` and moves
  nothing. Cost: hidden allocations wherever a copy is inserted; `ember inspect` must list each one,
  and `@noalloc` code refuses them. Gives up: the guarantee that a line with no `clone` never copies.

## A17. Borrow-error messages that do not say what to do

**(1) Rust.** Measured on the programs above: A1's E0499 has no help at all. Following rustc's own
advice fails twice:

- A23's E0507 suggests `ref mut path` and `ref other`; applied (`a17_suggested_fix.rs`) it gives two
  new errors, `E0308 expected Vec<(f32, f32)>, found &mut Vec<(f32, f32)>` and `match arms have
  incompatible types`.
- A8's E0505 suggests cloning `root`; applied (`a17_clone_suggestion.rs`) it gives
  `E0515: cannot return value referencing temporary value`.

The help that works is the mechanical one (E0106's `'a`, E0716's `let binding`, E0594's entry API).

**(2) Python.** No such errors: a run-time exception names the line.

**(3) Ember today: partly.** Most Ember helps name a real fix: E3022 in `e03` names `get_pair_mut`
(which compiles), E3020 names `retain`/collect-first (which compiles, `e04c`), E3060 in `e22b` says
"bind the value to a variable first", E3040 explains the Python difference. Misleading or missing
ones found:

- E3060 on `ecs5/pos.em` says "`pool` is a local, so its storage ends with the frame"; the storage is
  the pool object owned by the registry, and none of the three helps applies.
- L3001 on `e08b` tells the user to make `children` weak (the owning edge).
- `e21`: `counts.get(w).unwrap_or(0)` gives `E2020 expected ref i64, found an integer` with no help;
  `entry(…).or_default() += 1` gives `E2140 … not a place` though the result is `ref mut`.
- `e22`: `line.to_lower().split(" ").next()` gives `E2140: a mut self method needs a variable to write
  through` with no help.
- `e08d`: a crash instead of a message.

**(4) Frequency and severity.** Zhu et al., "Learning and Programming Challenges of Rust: A
Mixed-Methods Study" (ICSE 2022; 100 Stack Overflow questions and a survey of 101 programmers) studied
whether rustc's messages help (https://conf.researchr.org/details/icse-2022/icse-2022-papers/127/);
its numbers could not be retrieved (the PDF host refused the connection): UNVERIFIED. **Severity:
medium**: a good message shortens the fight; the fight is the problem.

**(5) The problem.** No safety rule of its own: this is the cost of every refusal above.

- **(a) Pythonic:** every refusal comes with the rewritten code as a fix-it, checked by re-running the
  checker on it before it is shown (rustc's failures above are suggestions never checked). Cost:
  compile time on the error path only.
- **(b) Smarter:** remove the refusals (the rest of this catalogue). For what stays, a run-time panic
  that names both accesses (`[EXC-6]` already does for class objects) can replace a compile-time
  refusal the programmer cannot decode, which is Python's model.

## A18. Initialisation: two-phase init, partially initialised values

**(1) Rust.** Build a game step by step, calling a helper before every field is set (`a18_init.rs`):

```rust
let mut g: Game;
g.renderer = Renderer { width };
g.frame = 0;
g.assets = g.load_assets();
```

```text
error[E0381]: partially assigned binding `g` isn't fully initialized
  = help: partial initialization isn't supported, fully initialize the binding with a default value and mutate it, or use `std::mem::MaybeUninit`
```

Workaround (`a18_init_fix.rs`): compute the parts as locals in dependency order, make the helper a
free function over the part it needs, build once.

**(2) Python.** In `__init__`: `self.renderer = Renderer(w); self.assets = self.load_assets()`; works
because `load_assets` only reads `renderer`.

**(3) Ember today: partly.** Field defaults and an `init` method exist (`[STR-6]`, `[CLS-2]`), but
`e18_init.em` (struct) and `e18b_init_class.em` (class) are both:

```text
error[E2100]: `self` is used before all struct fields are initialized
12 |         self.assets = self.load_assets()
```

with no help line.

**(4) Frequency and severity.** No count found (UNVERIFIED). **Severity: low**: defaults and
ordering cover most cases.

**(5) The problem.** A method reading a field that was never set reads garbage (in C, a wild pointer).

- **(a) Pythonic: check the callee's field reads, not "self as a whole".** `[BRW-10]` already knows
  which fields `load_assets` reads (`renderer`); accept the call when those are set.

  ```ember
  fn init(mut self, width: int):
      self.renderer = Renderer(width=width)
      self.assets = self.load_assets()     # reads only `renderer`: accepted
  ```

  Cost: compile time. Gives up: nothing for non-virtual methods; virtual calls stay conservative
  (`[CLS-11]`'s rule).
- **(b) Smarter: every field starts valid.** Zero- or default-initialise every field at allocation
  (Go, Java), so an early read sees a valid value, never garbage. Cost: a store per field per
  construction, mostly removed as dead stores. Gives up: types with no valid zero (a `ref`, a
  `NonZero`) need `Option`; reading a not-yet-set field silently gives zero.

## A21 (new). Updating a map entry in place

**(1) Rust.** Word count (`a21_map_update.rs`), `counts[w] += 1;`:

```text
error[E0594]: cannot assign to data in an index of `HashMap<String, u32>`
  = help: trait `IndexMut` is required to modify indexed content, but it is not implemented for `HashMap<String, u32>`
  = help: to modify a `HashMap<String, u32>`, use `.get_mut()`, `.insert()` or the entry API
```

Workaround (`a21_map_update_fix.rs`): `*counts.entry(w.to_string()).or_insert(0) += 1;`.

**(2) Python.** `counts[w] = counts.get(w, 0) + 1`, or `Counter`/`defaultdict`. `counts[w] += 1` on a
missing key raises `KeyError: 'w'` (`py_checks.py`).

**(3) Ember today: partly** (better than Rust). `counts[w] += 1` compiles (panics on a missing key,
as Python does, `std/src/collections.em` `index_mut`), and `if w in counts: … else: counts[w] = 1`
and `c = counts.entry(…).or_default(); c += 1` compile (`e21c_map_update.em`). Refused:
`counts.get(w).unwrap_or(0) + 1` (E2020, `Option[ref i64]` against an integer; `copied()` does not
exist, E2072, `e21b`) and `counts.entry(…).or_default() += 1` (E2140, defect 3).

**(4) Frequency and severity.** No count found (UNVERIFIED); the entry API is the standard Rust answer
(rustc's own help above). **Severity: low.**

**(5) The problem.** Rust's reason is that indexing cannot insert and a lookup-then-insert holds a
borrow of the map across the insertion (rehashing would move the values).

- **(a) Pythonic:** `get(k, default)` returning a value for `Copy` values; `Map(default=0)` so that
  `counts[w] += 1` inserts (defaultdict); `Option[ref T]` of a `Copy` `T` reads as `T` where a `T` is
  expected. Cost: one hash lookup, as `entry`. Gives up: nothing.
- **(b) Smarter:** none needed beyond (a): the operation is one lookup that returns a reference used
  within the statement, which the current rules already prove safe.

## A22 (new). A view of a temporary kept past its statement

**(1) Rust.** `a22_temporary.rs`:

```rust
let word = line.to_lowercase().split(' ').next().unwrap_or("");
word.len()
```

```text
error[E0716]: temporary value dropped while borrowed
  = help: consider using a `let` binding to create a longer lived value
```

Workaround (`a22_temporary_fix.rs`, rustc's own suggestion): `let lower = line.to_lowercase();`.

**(2) Python.** `word = line.lower().split(" ")[0]`.

**(3) Ember today: same.** `e22b_temporary.em`:

```text
error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
3 |     word = line.to_lower().trim()
  = help: bind the value to a variable first, so it lives as long as the borrow
```

The longer chain fails earlier, `E2140: a mut self method needs a variable to write through` at
`.split(" ").next()` (`e22_temporary.em`), and collecting the parts fails with E3063 (`e22c`, A25).

**(4) Frequency and severity.** Rust 2024 changed tail-expression temporary scopes because the old
rule "often comes as a surprise"
(https://doc.rust-lang.org/edition-guide/rust-2024/temporary-tail-expr-scope.html); Mara Bos's
`super let` proposal exists for the same problem (https://blog.m-ou.se/super-let/). Count UNVERIFIED.
**Severity: medium**: method chains on strings are everyday Python.

**(5) The problem.** The `String` made by `to_lower()` would be freed at the end of the statement while
`word` still points into it.

- **(a) Pythonic: the compiler does what its help says.** When a view of a temporary is bound to a
  local, the temporary lives until the view's last use (not the block's end, so memory is released as
  early as liveness allows).

  ```ember
  word = line.to_lower().trim()      # the temporary String lives until word's last use
  ```

  Cost: none in speed; the temporary's memory is held a little longer. Gives up: the exact drop point
  of a temporary with a `drop` of its own moves (must stay a fixed rule, stated in the spec).
- **(b) Smarter:** a view made from a temporary owns it (a counted buffer, as A7(b)). Cost: a count;
  gives up: nothing more than A7(b).

## A23 (new). Moving a field out from behind `mut self` (state machines)

**(1) Rust.** An NPC state machine (`a23_move_out_of_field.rs`):

```rust
self.state = match self.state {
    State::Walking(mut path) => { path.pop(); if path.is_empty() { State::Idle } else { State::Walking(path) } }
    other => other,
};
```

```text
error[E0507]: cannot move out of `self.state` as enum variant `Walking` which is behind a mutable reference
help: consider borrowing the pattern binding: State::Walking(ref mut path)   (wrong: see A17)
```

Workaround (`a23_move_out_of_field_fix.rs`): `std::mem::replace(&mut self.state, State::Idle)`.

**(2) Python.** `if isinstance(self.state, Walking): ...; self.state = Idle()`.

**(3) Ember today: same.** `e23_move_out_of_field.em`, `match owned self.state:`:

```text
error[E3013]: cannot move out of a reference
  = help: … move it out where its owner is held mutably, with `mem.take`, `mem.replace` or `swap` …
```

`e23b…` with `mem.replace(self.state, State.Idle)` compiles.

**(4) Frequency and severity.** The idiom has its own page in the Rust design-patterns book,
"`mem::{take(_), replace(_)}` to keep owned values in changed enums"
(https://rust-unofficial.github.io/patterns/idioms/mem-replace.html). Count UNVERIFIED. **Severity:
medium**: state machines are core game logic.

**(5) The problem.** In Rust, a panic between the move out and the reassignment unwinds to a caller
that still owns `*self`, which would then read or drop a moved-out field (double free).

- **(b) Smarter, and it already holds in Ember: there is no unwinding.** `[PAN-1]`: a panic calls
  `abort()`; "There is no unwinding in this version". No caller can observe the half-moved field, so
  the only rule needed is definite assignment: every path reassigns `self.state` before it is read or
  `self` is passed on (the analysis E2100 already does).

  ```ember
  fn update(mut self):
      match owned self.state:              # accepted: every path below reassigns self.state
          State.Walking(path):
              path.pop()
              self.state = State.Idle if path.is_empty() else State.Walking(path)
          other:
              self.state = other
  ```

  Cost: none (a drop flag only where a path might not reassign, which is then refused). Gives up:
  this would have to be revisited if unwinding is ever added.
- **(a) Pythonic:** the same code; (b) is what makes it the Pythonic spelling.

## A24 (new). struct, `Box` or class: the kind changes what compiles and what panics

**(1) Rust.** Not applicable: Rust has one model; `a05_ecs_get_mut.rs` (pools in `Box<dyn Any>`)
compiles and its misuse is refused at compile time (`a05_ecs_misuse.rs`).

**(2) Python.** One model: every value is an object.

**(3) Ember today: worse.** The same registry, measured on the ODR-286 probes:

| Pools are | `return ref mut pool.dense[i]` | adding during a view loop | probe |
|---|---|---|---|
| class objects via `cast` | E3060 (refused) | compiles; left to a run-time access check (panic by `[EXC-1]`, UNVERIFIED) | `ecs5/pos.em`, `ecs4/accept_change_during_loop.em` |
| `Box[Pool[int]]` | compiles; changing the pools while the reference lives is E3025 | not probed; by the static rules it is refused at compile time, as `e04` is (UNVERIFIED) | `ecs5/box1.em`, `ecs5/box_bad.em` |
| a class field loop seen directly | — | E3020 at compile time | `e04b_mutate_while_iterating_class.em` |

And A1/A2/A11 above: the class form of each program compiles where the struct form is refused,
paying a heap block, a 24-byte header (`[OBJ-1]`), counting, and a run-time access word per non-`Copy`
field (`[EXC-19]`).

**(4) Frequency and severity.** The owner hit it building the ECS (ODR-286,
`tasks/95/odr-pending.md`). **Severity: high**: the programmer must choose a kind before writing the
code, and the choice decides whether errors come at compile time, at run time, or not at all.

**(5) The problem.** Two models exist because a single static model (Rust's) refuses shared object
graphs, and a single dynamic model (Python's) costs counts and checks everywhere.

- **(a) Pythonic:** one declaration (`class`) whose representation the compiler chooses: a value when
  escape analysis proves a single owner, a counted object otherwise (`[OPT-1]` stack promotion is a
  start). Cost: none where the proof succeeds; counts and checks where it fails. Gives up: the
  predictability of the cost from the declaration alone (`[SEL-1]`'s "the compiler never picks");
  `ember inspect --alloc` must show the choice.
- **(b) Smarter:** make both kinds obey the same three rules (stable storage, deferred structural
  change, path references: A2, A4, A5), so the kind changes only the cost, never what compiles.

## A25 (new). A collection of views, refused even when it is local

**(1) Rust.** `a06d_local_vec_of_views.rs` compiles:

```rust
let lower = line.to_lowercase();
let words: Vec<&str> = lower.split_whitespace().collect();
words.len()
```

**(2) Python.** `words = line.lower().split()`.

**(3) Ember today: worse.** `e06e_local_array_of_views.em`, the same program:

```text
error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
6 |         words.push(w)
  = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index …
```

`[TYP-15]` makes every heap-collection element an unbounded place; `[TYP-15a]`'s `BorrowList` would
allow it and is not implemented (defect 5). The help's owned copy costs an allocation per word, the
opposite of the owner's C-performance goal.

**(4) Frequency and severity.** Measured here (Ember-specific). Splitting text into a list of words
is among the first things a Python programmer writes. **Severity: high.**

**(5) The problem.** A view stored in a heap collection must not outlive its source; Ember's rule
chooses the simplest sufficient condition (only `static` views), which refuses the bounded case.

- **(a) Pythonic:** `Array[str]` in a local carries the regions of its elements (A6(a), what Rust
  does with `Vec<&'a str>`), inferred; nothing written. Cost: compile time. Gives up: such an array
  still cannot go into a class field or static.
- **(b) Smarter:** shared immutable strings (A7(b)): a word keeps its buffer alive, so a list of words
  can go anywhere. Cost: a count per buffer and a retain per stored word.

---

## Sources

- rustc 1.98.1 and `ember.exe check` outputs: `target/redesign/phase1/borrowing/rustc_output*.txt`,
  `ember_output*.txt` (this agent's runs, 2026-10-24).
- Python behaviour: `target/redesign/phase1/borrowing/py_checks.py` (this agent's run, Python 3.14).
- 2020 Rust survey, lifetimes 61.4% "tricky or very difficult":
  https://blog.rust-lang.org/2020/12/16/rust-survey-2020/
- 2023 Rust survey (borrow checker named among the hardest areas, no percentage; 43% of those sharing
  worries fear Rust becoming too complex): https://blog.rust-lang.org/2024/02/19/2023-Rust-Annual-Survey-2023-results/
- RFC 2094 (NLL), problem case #3: https://rust-lang.github.io/rfcs/2094-nll.html
- Polonius 2026 goal: https://rust-lang.github.io/goals/2026/polonius.html
- Partial borrowing issue: https://github.com/rust-lang/rfcs/issues/1215
- Niko Matsakis, "After NLL: Interprocedural conflicts":
  https://smallcultfollowing.com/babysteps/blog/2018/11/01/after-nll-interprocedural-conflicts/
- LogLog Games, "Leaving Rust gamedev after 3 years": https://loglog.games/blog/leaving-rust-gamedev/
- Rust 1.86 `get_disjoint_mut`: https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/
- Too Many Linked Lists: https://rust-unofficial.github.io/too-many-lists/
- ouroboros / self_cell downloads: https://crates.io/api/v1/crates/ouroboros,
  https://crates.io/api/v1/crates/self_cell (fetched 2026-10-24)
- Ergonomic ref-counting goal: https://rust-lang.github.io/goals/2026/ergonomic-rc.html
- Rust 2024 temporary tail-expression scope:
  https://doc.rust-lang.org/edition-guide/rust-2024/temporary-tail-expr-scope.html
- Mara Bos, `super let`: https://blog.m-ou.se/super-let/
- mem::replace idiom: https://rust-unofficial.github.io/patterns/idioms/mem-replace.html
- Python tutorial on modifying a collection while iterating:
  https://docs.python.org/3/tutorial/controlflow.html
- Zhu et al., ICSE 2022 (numbers not retrieved, UNVERIFIED):
  https://conf.researchr.org/details/icse-2022/icse-2022-papers/127/
- Ember spec rules cited: `docs/spec-source/Ember_v0.9.10_Hardened_35.md` ([BRW-*], [LT-*], [EXC-*],
  [OBJ-1], [WK-1], [PAN-1], [TYP-15], [TYP-15a], [EXP-6], [HND-1], [ARN-1], [OPT-1], [SEL-1]).

Tools: WebSearch and WebFetch worked; one PDF (par.nsf.gov) refused the connection and a second
(arXiv 2301.02308) could not be rendered (`pdftoppm` is not installed), so their numbers are not used.
