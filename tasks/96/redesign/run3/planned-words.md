# 07 word planner: the words the five areas start from (run 3, phase 2)

Inputs: `RUN3/vocabulary.md` (06's approved table), `RUN3/annoyances.md` (AN-01 .. AN-84),
`inputs/JOB.md`. Before any design, this file fixes the new words the five areas will need, in one
place, so areas A-E start from one vocabulary. An area uses only the words of `vocabulary.md` and this
file; a word it still needs goes in its own "new words" table (duty 08-12).

Power: `PowerLineStatus` printed `Online` at the start and at the end of this work.
Probes: `target/redesign3/07-word-planner/p01..p12_*.em`; every output quoted below is from
`target/redesign3/bin/ember.exe check <file>`, saved in full in
`target/redesign3/07-word-planner/check_output.txt` (section 9 pastes it).

## 0. Terms

- **new word**: a spelling that is in neither `vocabulary.md` nor today's Ember.
- **new position**: a word already in `vocabulary.md`, written in one more place, with the same meaning.
- **value word**: a word written before a value, in a type, or in a lambda's capture list, that says
  what a name holds: its own copy, the one owner, one of several owners, or a link that does not keep
  its target alive.
- **capture list**: what follows `with` after a lambda's parameters (agreed row 6:
  `fn() with copied score:`).
- **the owner's test**: for every name in a line, the word that says what it is (the owner's own
  thing, a copy, or handed over) and the word that says whether it may change it (brief).
- **counted**: an object whose owners are counted while the program runs; it is freed when the count
  reaches zero. A **one-thread count** is changed with a plain add; an **atomic count** with a locked
  instruction, so that two threads may change it at once.
- **view**: a value that points into something else and may not outlive it (`ref T`, `Span[T]`,
  `StrView`).
- **placeholder**: a spelling 05 or 06 proposed so a row could be read, left to this file to fix.

## 1. The decision in short

**Four new words, plus one new argument spelling:**

| word | says | where written | annoyances |
|---|---|---|---|
| `thread_shared` | one of several owners, which may be on different threads | before a value, in a type, in a capture list | AN-39, AN-71, AN-74, AN-75, AN-80 |
| `weak` | a link that does not keep its target alive; reading it gives `Option` | before a value, in a type, in a capture list | AN-39, AN-60, AN-63, AN-64, AN-65 |
| `set by` | this link is kept by the language equal to the object whose named field holds this one; the program reads it and never assigns it | after a `weak` field's type | AN-63, AN-64 |
| `@cached` | the method's result is kept after the first call and cleared when a field its `reads` clause names changes | on a method, above `fn` | AN-21 |
| `@fp(allow_fma)` | final spelling of the placeholder `@fp(fuse)` (today `@fp(contract)`) | on a function | AN-83 |

**Four new positions of existing words** (section 4): every value word in the capture list
(`with shared log`, `with thread_shared log`, `with weak self`, and `with owned s` made final);
row 3's `changes` on a free function naming a module variable (`fn tick() changes FRAME:`); the `from`
clause after a field's type as well as after a result type; `owned T` on a foreign type, replacing
`ForeignBox[T]`.

**Placeholders fixed** (section 5): kept as spelled: `StrView`, `CStrView`, `RuntimeChecked`,
`ReadGuard`, `WriteGuard`, `read`, `write`, `check_disjoint`, `@thread_shared`, `ThreadShared`,
`@no_alias_checks`, `from`, `with owned x`. Changed: `Foreign[T]` becomes `owned T` (no new word);
`@fp(fuse)` becomes `@fp(allow_fma)`. Dropped: `get` for `upgrade` (reading a `weak` link needs no
method). Completed: `SyncShared[T]` becomes `thread_shared T`.

**Not planned** (section 6, each with its reason): `global`, `isinstance`, `copy(x)`, `backlink(...)`
as spelled last run (its mechanism is kept, respelled `set by`), `Foreign[T]`, `get` for `upgrade`,
`shared mut`, a second spelling of `unsafe`, a word for "delete after the loop", `get(k, default)`,
`no_share`, `nonlocal`, `lazy`.

The agreed rows 3-9 are untouched: no spelling or meaning in `inputs/JOB.md` changes. Row 6's capture
list gains more value words after `with`; row 3's `changes` gains one place (a free function); row 7's
`reads` is what `@cached` depends on.

## 2. One table for the value words

The family every area starts from. One word, one meaning, the same for every type; the position only
says what the word is put on.

| word | before a value | in a type (field, element, result) | in a capture list | means |
|---|---|---|---|---|
| `copied` (row 6, 9) | `snapshot = copied items` | refused: a type does not copy | `fn() with copied score:` | a new, independent value equal to the old one |
| `owned` | refused (E0109: "`b = a` already hands `a` over") | `mesh: owned Mesh`; on a parameter's name, `owned x: T` | `fn() with owned s:` | handed over: one owner, the giver's name ends |
| `shared` (row 9) | `other = shared items` | `mesh: shared Mesh`, `shared dyn I` | `fn() with shared log:` | one of several owners, all on one thread; one-thread count |
| **`thread_shared`** (new) | `log = thread_shared Mutex(lines)` | `log: thread_shared Mutex[Array[String]]` | `fn() with thread_shared log:` | one of several owners, on any threads; atomic count |
| **`weak`** (new, replaces `Weak[T]`) | `cam.target = weak player` | `target: weak Player` | `fn() with weak self:` | a link that does not keep its target alive; reads as `Option` |
| `ref`, `ref mut` (row 8, C-1) | `r = ref mut xs[1]` | `-> ref T`, a pattern name, a parameter's name | not written: a capture not named is already the real variable (row 6) | a name for someone else's place; `mut` = may change it |

Two clauses complete it, each after a type: **`from`** ("these views point into the named value") and
**`set by`** ("the language keeps this link equal to the holder"). Swift writes the same capture
words in the same place (`{ [weak self] in ... }`), which is the closest precedent for the third column.

## 3. The new words

### 3.1 `thread_shared`

- **Spelling:** `thread_shared`, the same family letter for letter as `@thread_shared` (on a class)
  and `ThreadShared` (the property in a bound), both 06's (C-4).
- **Exact meaning:** the value has several owners, which may be on different threads; it lives until
  the last of them ends. Its count changes are atomic. Every owner may read it; no owner changes it
  directly: a change goes through what its type provides (`Mutex`, `RwLock`, `Atomic`, or such fields
  of a `@thread_shared` class). The same for every type.
- **Where written:** before a value (`log = thread_shared Mutex(lines)`), in a type
  (`log: thread_shared Mutex[Array[String]]`), in a capture list (`fn() with thread_shared log:`, the
  function becomes one more owner). The capture word means exactly what it means before the value.
- **Replaces:** `SyncShared[T]` (vocabulary: "07 spells the combination"); Rust's `Arc`.
- **Annoyances:** AN-75 (two threads append to one log, without the clone-before-move dance); AN-74
  (several tasks update one statistics object); AN-71 (a task that outlives `thread.scope()` and
  reads shared input); AN-39 (one of the eleven ownership mechanisms, now a word of the family);
  AN-80 (the `Sync` family has one spelling).
- **Why no existing word does:** `shared` is one thread, with a one-thread count. Whether a value may
  go to another thread changes what the program may do (a `shared` value captured into
  `thread.spawn` is refused) and what each new owner costs (a locked instruction instead of a plain
  add), so the line must say it. Letting the type decide does not work: one type (`Mutex[Array[String]]`,
  a value type, spec 3511) is held both ways, and a `shared T` parameter would then have to pay the
  atomic count everywhere. `@thread_shared` sits on a class and says what its objects may be; it does
  not say that this value is shared.
- **Example (NEW):**

  ```ember
  log = thread_shared Mutex(lines)
  worker = thread.spawn(fn() with thread_shared log: append_line(log, "started"))
  ```

  The owner's test: `log` (line 1): `thread_shared` says it is one of several owners on any thread;
  `Mutex` (its type) says it changes only under the lock. `log` in the lambda: `thread_shared` in the
  capture list says the function holds its own owner, so the thread may outlive the caller's `log`.
  `worker`: none (default: the `JoinHandle` this call returns, the caller's own; `thread.spawn`'s
  result type says it, as row 5's `word` takes its meaning from its value).
- **Stays refused:**
  - a `shared` value sent to another thread: "`log` is shared on this thread only; to share it with
    other threads, write `log = thread_shared Mutex(lines)`";
  - a change through a `thread_shared` name: "`log` is shared between threads, so no owner changes it
    directly; share a `Mutex(lines)` and write inside `with g = log.lock():`";
  - `SyncShared[T]`: "write `thread_shared T`".
- **What runs:** an atomic add when a new owner is made and an atomic subtract when one ends, as
  Rust's `Arc`; reads are plain loads.
- **Today:** `p01_thread_shared.em`: `E1010: cannot find 'thread_shared' in this scope` (section 9).

### 3.2 `weak`

- **Spelling:** `weak`, the lowercase field word, beside `owned` and `shared`. Python's own module for
  the idea is `weakref`, so the word is Python's.
- **Exact meaning:** a link to an object that does not keep it alive. Reading it gives `Option`:
  `Some` while the object lives, `None` once it is gone. A name bound from that `Some` is a read-only
  view of the object, kept alive until the name's last use (row 5's rule for `word`); `Some(ref mut p)`
  binds a view that may change it (row 8's words). The target must be an object that lives on its own:
  a `class` object, or a value held `shared` or `thread_shared`.
- **Where written:** in a type (`target: weak Player`, `Array[weak City]`), before a value
  (`cam.target = weak player`: Python too writes the word at the assignment, `weakref.ref(target)`),
  in a capture list (`fn() with weak self:`, a callback that must not keep its owner alive).
- **Replaces:** the type `Weak[T]`, its constructors `Weak(x)` and `Weak.empty()`, and the method
  `upgrade` (Rust's word, AN-65). 06's placeholder `get` for `upgrade` is not needed (C-6's condition
  is met by having no method at all).
- **Annoyances:** AN-63 (parent pointers, graphs), AN-64 (L3001's fix names the right edge and word),
  AN-65 (`upgrade`), AN-60 (a handler that must not keep the UI alive: `with weak self`), AN-39.
- **Why no existing word does:** `Weak[T]` says the same thing but in a different form from the two
  words it sits beside: "does this link keep its target alive, and who owns it" is answered by
  `owned Mesh` and `shared Mesh` in a type, and should not switch to a generic wrapper for the third
  answer (JOB: "whether a link keeps its target alive" written in readable words). `Box` and `Shared`
  were merged into `owned` and `shared` for the same reason (vocabulary settlement 16).
- **Example (NEW):**

  ```ember
  class Camera:
      target: weak Player = None

  cam.target = weak player
  if Some(p) = cam.target:
      println(p.health)
  ```

  The owner's test: `target`: `weak` says it does not keep the player alive; it may be re-pointed
  through `changes target` or `mut self` like any field. `cam.target = weak player`: `weak` on the value
  says a link, not a hand-over (row 9 would hand `player` over without it); `player` stays the
  caller's. `p`: none (default: a read-only view of the player, kept alive until `p`'s last use, as
  row 8's plain `path` is a read-only view; it never changes anything and never copies).
- **Stays refused:**
  - `cam.target = player` into a `weak` field: "`target` does not keep the player alive, and `=` alone
    would hand `player` over; write `cam.target = weak player`";
  - `cam.target.health` with no test: "`cam.target` may be gone; write `if Some(p) = cam.target:`";
  - `weak pos` of a plain value (a `Vec3` inside `player`): "a weak link needs an object that lives on
    its own, and `pos` is part of `player`; link to the player: `weak player`";
  - `Weak[T]`, `Weak(x)`, `w.upgrade()`: "write `weak T`", "write `weak x`", "write `if Some(p) = w:`".
- **What runs:** one test of the target's header per read, as Rust's `Weak::upgrade`; last run kept
  this test on read because registering weak links instead cost 1.09x to 1.27x of plain C on churn,
  where Rust's `Weak`, a test on read, cost 0.92x and 0.94x (`tasks/96/redesign/phase3b-combined.md:124-129`). While `p` is
  alive the object is kept, by a count pair unless area D proves nothing in `p`'s life can free it.
- **Today:** `p03_weak_value.em`: `E1010: cannot find 'weak' in this scope`; `p02_weak_set_by.em`:
  `E1010: cannot find type 'weak'`. The spelling it replaces works: the spec's own example
  (`p09_weak_spec_example.em`, spec 2591-2606) is accepted, with L3001 telling the programmer to make
  the owning edge `children` weak (AN-64).

### 3.3 `set by`

- **Spelling:** `set by <field>`, two contextual words (they stay usable as names elsewhere, as
  `Cell.set(...)`), written after a `weak` field's type. It replaces last run's
  `parent: Option[Node] = backlink(children)`, whose mechanism is kept and whose spelling read as a
  default value computed by a call.
- **Exact meaning:** this link always points at the object whose field `<field>` holds this object,
  or is empty when none does. The language sets it when this object is put into that field and clears
  it when the object leaves; the program reads it and never assigns it. It does not keep its target
  alive (hence `weak`), and needs no test on read: the holder cannot end while it still holds this
  object without first clearing the link.
- **Where written:** after a `weak` field's type: `parent: weak Node set by children` (one holder);
  `into: Array[weak City] set by roads` (every object whose `roads` holds this one); `prev: weak Node
  set by next` (one to one). The named field belongs to the target's type and holds values of this
  type (`Array[Node]`, `Option[Node]`, `Array[weak City]`).
- **Annoyances:** AN-63 (a scene tree whose nodes know their parent; the Python spelling leaks today),
  AN-64 (L3001 tells the programmer to weaken the owning edge; with `set by`, the message names the
  back edge instead).
- **Why no existing word does:** `weak` alone says the link does not own, but leaves the program to
  set and clear it by hand and pays a test on every read; nothing else says "the language keeps this
  equal to the holder", which is what makes the read a plain load and makes `node.parent = p` an
  error. `from` means "points into" (a view of the named value's contents), which a parent link is not.
  `in` was tried and rejected: `parent: weak Node in children` reads as "the parent is in `children`",
  which is false (this node is in the parent's `children`), and `in` after a type already means a range
  type (`f32 in 0.0..=1.0`, spec 579).
- **Example (NEW):**

  ```ember
  class Node:
      value: int
      parent: weak Node set by children
      children: Array[Node] = []

  root.children.push(leaf)          # leaf.parent is now root
  ```

  The owner's test: `parent`: `weak` says it does not keep its node alive (reads as `Option[Node]`);
  `set by children` says only `children` changes it, so the program never assigns it. `children`:
  none (default: the node's own list of nodes it owns, changed through `changes children` or
  `mut self`, as any field). `leaf` in the push: none (default: handed over into the list, row 9).
- **Stays refused:**
  - `leaf.parent = root`: "`parent` is set by `children`; to give `leaf` a parent, write
    `root.children.push(leaf)`";
  - `parent: Node set by children` without `weak`: "`parent` points back at the node that owns this
    one, so it cannot own it too; write `parent: weak Node set by children`";
  - a single link `set by` a field whose elements are `shared`: "a node in `Array[shared Node]` can be
    in two `children` lists at once, so `parent` would have two answers; write
    `children: Array[Node]`, or make `parent` an `Array[weak Node] set by children`".
- **What runs:** reading is a plain load. Putting an object in or taking it out writes the link (one
  store); for a list-valued link, taking it out searches that list, as hand-written C does. Last run
  measured the mechanism on the tree workload at 0.964 and 1.024 of plain C in the same layout, where
  safe Rust's index arena took 1.519 and 1.357 (`tasks/96/redesign/phase3b-combined.md:102`, which marks
  both as provisional; phase 4's timer measures this run's model).
- **One change from last run, said here because it is a word question:** last run's push took the node
  out of its old parent's list first, a search the line did not say
  (`tasks/96/redesign/phase3b-graphs-callbacks.md:248-251`). Under this run's vocabulary that step is
  not needed: an element of `Array[Node]` has one owner, so the program moves a node by taking it out
  (`node = old.children.remove(i)`, the link is cleared) and pushing it in (the link is set), both
  written. Area D states the rest of last run's rules (the holder field cannot be lent to code that
  does not know the link, rule 6 there).
- **Today:** `p02_weak_set_by.em`: refused at `weak` (`E1010`) and `owned` (`E0100`) (section 9).

### 3.4 `@cached`

- **Spelling:** `@cached`, Python's word (`functools.cached_property`, `functools.cache`), as an
  attribute like `@inline`.
- **Exact meaning:** the method's result is computed at the first call and kept in a slot of the
  object; later calls return the kept result; the slot is cleared by every change to a field the
  method's `reads` clause names (row 7's word), through any name (taking a `ref mut` view of the field
  counts as a change). Unlike Python's, the kept value is never stale.
- **Where written:** on a method, above `fn`, whose receiver is plain `self` and which has a `reads`
  clause: `@cached fn radius(self) reads verts -> f64:`.
- **Annoyances:** AN-21 (`radius()` computes the mesh radius once and keeps it; today's struct form is
  E3023 with `RefCell`/`class` help).
- **Why no existing word does:** `Cell[Option[f64]]` in a field can hold the result, but the program
  then clears it by hand at every change, and a missed clear is a stale result; the field also has to
  be `Cell` for a read-only method to set it (06's C-10 condition). `Once` (spec 3516) runs a static's
  initialiser once and never clears. `reads` says what the result depends on but not that it is kept.
  `@cached` plus `reads` writes both on the method's first line: what is kept, and what clears it.
- **Example (NEW):**

  ```ember
  class Body:
      verts: Array[f64] = []

      @cached
      fn radius(self) reads verts -> f64:
          ...
  ```

  The owner's test: `self`: plain `self`, read-only (row 3); the slot is not a name the program writes.
  `verts` in `reads verts`: the field the result depends on, read only (row 7). `@cached` says the result
  is kept; `reads verts` says a change to `verts` clears it.
- **Stays refused:**
  - `@cached` without `reads`: "say what `radius` depends on, so a change can clear it:
    `@cached fn radius(self) reads verts -> f64:`";
  - `@cached` on a method with other parameters: "`@cached` keeps one result, and `radius` takes
    `scale`; keep a `Map` field of results by hand";
  - `@cached` reading a field that may change without this object seeing it (a `shared`,
    `thread_shared`, `weak` or `Cell` field): "`radius` reads `mesh`, which its other owners may change
    without `Body` seeing it, so a kept result could be stale; remove `@cached`, or keep the result by
    hand";
  - `@cached` with `changes` or `mut self`: "a cached method only reads; remove `changes verts`".
- **What runs:** the slot and a flag in the object, one branch per call, one store at each place that
  may change a named field: the lazy cache a C programmer writes by hand (last run, which measured the
  same mechanism: `tasks/96/redesign/phase3b-ownership.md:678-695`;
  `phase3b-ownership-attack.md:1023`, kept with the limit "only the object's own fields").
- **Open for areas B and E:** the result type (a `Copy` value, or a view of the kept value valid until
  a named field changes); an object shared between threads needs a once-flag for the slot, or
  `@cached` is refused on `@thread_shared` classes.
- **Today:** `p04_cached.em`: `E0104: '@cached' is not an attribute` (and `reads` is `E0100`, as 05
  found for row 7).

### 3.5 `@fp(allow_fma)`

- **Spelling:** `@fp(allow_fma)`; 05 proposed `@fp(fuse)` for today's `@fp(contract)`.
- **Exact meaning:** the same as today's `@fp(contract)` (`[TYP-9b]`, spec 1014): inside this function
  the compiler may fuse `a * b + c` into one fused multiply-add.
- **Annoyance:** AN-83 ("contract" means a checked promise everywhere else in Ember).
- **Why this spelling and not `fuse`:** "fuse" alone does not say what is fused. `fma` is already the
  standard library's name for the operation (`std/src/math/mod.em:212`, `pub fn fma[T: Number](a: T,
  b: T, c: T)`; spec 3380), so one operation keeps one name. `allow_` says the compiler may fuse, not
  that every `a * b + c` is fused, which matters to a program that wants the same bits everywhere.
- **Example (NEW):** `@fp(allow_fma)` above `fn dot(a: Vec3, b: Vec3) -> f64:`. No name is bound.
- **Stays refused:** `@fp(contract)`: "write `@fp(allow_fma)`".
- **What runs:** nothing more; one instruction where two were.
- **Today:** `p11_fp_contract_today.em` accepted; `p12_fp_allow_fma.em`: `E0104: '@fp' needs
  'contract': '@fp(contract)'`.

## 4. New positions of existing words

| word | the new position | meaning there (the same as everywhere) | annoyances | example (NEW) | today |
|---|---|---|---|---|---|
| `copied`, `owned`, `shared`, `thread_shared`, `weak` | the capture list, one word per name, separated by commas | what the function holds of that name, exactly as before a value; a name not in the list is the real variable (row 6) | AN-53..AN-55 (row 6), AN-60, AN-75, AN-78 | `fn() with copied score, thread_shared log:` | `p05_capture_shared.em`: `E0100: expected '=>' or ':' after a lambda's parameters` |
| `changes` (row 3) | on a free function, naming a module variable (`static mut`) | the places this function changes, other than through its `ref mut` parameters; checked against its body and at every call (a loop walking `LOG` may not call a function that `changes LOG`) | AN-76 | `fn tick() changes FRAME:` | `p06_changes_static.em`: `E0100` at `changes`; and `static mut` is refused (`E1010: 'static mut' needs 'unsafe'`) |
| `from` (06's clause, replaces `@borrows`) | after a field's type, naming a `let` field of the same object; as well as after a result type | these views point into the named value | AN-25, AN-24 (field); AN-22, AN-07 (result) | `words: Array[StrView] from text`; `fn first_word(text: StrView) -> StrView from text:` | `p08_from_field.em`, `p07_from_result.em`: `E0100` at `from` |
| `owned` | on a foreign type (`type T` with `@ffi(destroys(f))`) | one owner; freed through the contract's destructor when the owner ends | AN-40, AN-78 | `db: owned SqliteDb`; `std.ffi.adopt[T](p) -> owned T` | `owned` in a type is `E0100` today (05's `c6_owned_field.em`) |

Notes for the areas:
- **Capture list.** Each name carries its own word: `with copied a, copied b`. `with copied a, b` is
  refused ("say what the function does with `b`: `with copied a, copied b`, or leave `b` out to use the
  real `b`"). `ref` and `ref mut` are not written there: an unnamed capture is already the real
  variable (row 6), so `with ref mut x` is refused with "remove `ref mut x`: the function already uses
  the real `x`".
- **`changes` on a free function.** Python writes the same thing with `global FRAME` inside the
  function: a word in the function that says it changes the module variable. Spec `[STA-1]`
  (1803-1808) makes every `static` read-only and `static mut` foreign-only with `unsafe` at each use;
  area E rules whether an Ember `static mut` becomes safe when every function that changes it says
  `changes`, and what stops two threads reaching it. No `global` word is needed either way.
- **`from` on a field.** The named field must be `let` (06's C-8: nothing is changed through it after
  `init`), so the views into its contents stay valid as long as the object lives. Area B checks that
  the contents of the named field live in their own heap block (a `String` or `Array`): if `String`
  keeps short text inside its own header, moving the object would move that text, and the rule must say
  so. A local list of views needs no clause: it is inferred, as row 5's `word` is.
- **`owned` on a foreign type.** Spec `[FFI-36a]` (4883-4887): `adopt[T](h) -> ForeignBox[T]` takes
  ownership and frees through the contract's destructor; spec 6508 maps C++ `std::unique_ptr<T>` to it.
  That is the one-owner meaning of `owned` (vocabulary row `owned`: "the one-owner type that replaces
  `Box`"), with the type saying how it is freed. So `ForeignBox[T]` merges into `owned T` and 06's
  placeholder `Foreign[T]` (which says "foreign" and not "one owner") is not needed. `CppShared` and
  `CppWeak` keep their names: a C++ count is not Ember's (E5065).

## 5. The placeholders, final

| placeholder (vocabulary) | final | why |
|---|---|---|
| `StrView` (for `str`) | `StrView` | kept: names the pair with `String` (`Str...`), says "view"; C++'s `std::string_view` is the same idea |
| `CStrView` (for `cstr`) | `CStrView` | kept, the same pair with `CString` |
| `RuntimeChecked[T]` (for `RefCell`) | `RuntimeChecked[T]` | kept: says the cost the owner asks every answer to show (a check while running); area C may still remove the type |
| `ReadGuard`, `WriteGuard`, `read`, `write` | kept | `RwLock` already has `read()` and `write()` (spec 3515-3516), the same shape (`[CELL-8]`) |
| `Foreign[T]` (for `ForeignBox[T]`) | **`owned T`** on a foreign type | section 4: no new word; `ForeignBox` is one owner |
| `get` (for `Weak.upgrade`) | **dropped** | reading a `weak` link gives `Option` directly (3.2); no method, so C-6's condition cannot fail |
| `check_disjoint`; `assert_disjoint` for the panicking form | kept | an `assert` stops the program, as in Python |
| `@thread_shared`, `ThreadShared` | kept | the family `thread_shared` completes (3.1); area E may still remove the attribute |
| `@no_alias_checks` (for `@static_safe`) | kept | reads as the `@no...` contract family |
| `@fp(fuse)` | **`@fp(allow_fma)`** | 3.5 |
| `from p` (for `@borrows`) | `from` | kept; one more position, after a field's type (section 4) |
| `with owned x` | `with owned x` | kept; refused on a `Copy` capture ("write `with copied score`", C-2) |
| `SyncShared[T]` (merged into `shared`, "07 spells the combination") | **`thread_shared T`** | 3.1 |

## 6. Words considered and not planned

| word | proposed by | why not | the word that says it instead | good part kept |
|---|---|---|---|---|
| `global` | last run (A19) | Python's `global FRAME` is a statement inside the function that changes the module variable; `changes FRAME` on the function's first line says the same, checked, in row 3's form | `static mut` + `changes FRAME` (section 4) | the idea that the function, not only the variable, says it changes it |
| `isinstance` | last run (A13) | `cast x if D` and `cast x as D` already exist (ODR-280), and their messages give the line (vocabulary 5.4) | `cast` | narrowing after a test, for area D |
| `copy(x)` | last run (A11, A33) | one spelling for a copy (rows 6, 9) | `copied x` | — |
| `backlink(children)` | last run (A8) | spelled as a default value computed by a call: does not sit on the field's type and hides that the language sets it | `weak T set by f` (3.3) | the whole mechanism (plain load, measured on par with C) |
| `Foreign[T]` | 03, 04 | says "foreign", not "one owner" | `owned T` (section 4) | — |
| `get` for `upgrade` | 02 | not needed (3.2) | reading the `weak` link | — |
| `shared mut` | considered here | whether one of a shared value's owners may change it is said by the name or method that changes it (`ref mut p`, `changes target`), as for every other value; a `mut` inside the type would be a second place for the same fact. Area C may still ask for it if its mechanism needs the type to say it (AN-48) | `changes`, `ref mut`, `RuntimeChecked` | — |
| a second spelling of `unsafe` | 01, 04 (AN-79) | "an unsafe block" and "an unsafe function" both read right where they sit, with one meaning: the compiler cannot check this and a person vouches. The two directions become clear if every unsafe operation in an `unsafe fn` body must sit in an `unsafe` block, as Rust 2024 does; area E rules that | `unsafe` | — |
| a word for "delete after the loop" | — | the agreed limitation needs no word: `Array.retain` exists (`std/src/core.em:2318`) and `Commands` for the ECS; a list of additions is pushed after the loop | `retain`, `Commands`, `extend` | — |
| `get(k, default)` | annoyances AN-12 | `Map.get_or(k, default)` exists (`std/src/collections.em:811`) and compiles (`p10_existing_list_words.em`, accepted) | `get_or` | — |
| `no_share(...)` | the owner's example, JOB | the owner offered it "or a better word"; vocabulary already settled `owned` (`owned dyn Pool`) by his ruling | `owned` | — |
| `nonlocal` | Python | row 6 makes a capture the real variable unless named | — | — |
| `lazy`, `Once` in a field | considered here | `Once` runs a static's initialiser once and never clears (spec 3516); `lazy` would be a second word for `@cached` without saying what clears it | `@cached` | — |
| `Synchronise`, `@nosynchronise` | 05 | dropped by 06 (C-4) | `Sync` (effect), `@nosync` | — |

## 7. Every annoyance and the words its answer starts from

"Planned here" names what this file adds; "—" means the vocabulary already has every word the answer
needs, or the answer is not a word (a compiler fix, a message, a cost).

| id | words (from `vocabulary.md`) | planned here |
|---|---|---|
| AN-01 | `ref mut`, `copied` | — |
| AN-02 | `ref mut`, `copied` | — |
| AN-03 | `ref mut` (twice, with `i != j` proved or checked once) | — |
| AN-04, AN-05 | the understandable limitation; `retain`, `extend` after the loop | — |
| AN-06 | `Commands` | — |
| AN-07 | `StrView` (a local list is inferred); `from` to return one | `from` (result) |
| AN-08, AN-09 | row 5; `String(...)` | — |
| AN-10 | none (a temporary is a place) | — |
| AN-11 | `MutSpan`, `ref mut s: MutSpan[T]` | — |
| AN-12 | `get_or`, `entry`, `or_default` | — |
| AN-13 | `SoA.get` (a reference), `copied` | — |
| AN-14 | `check_disjoint`, `assert_disjoint` | final (section 5) |
| AN-15 | `ref mut T` in a `Query` | — |
| AN-16 | messages | — |
| AN-17, AN-18, AN-19 | row 3: `changes`, `mut self`, `self` | — |
| AN-20 | `ref mut` on the name (C-1); `Cell` (C-10) | — |
| AN-21 | `reads` (row 7) | **`@cached`** |
| AN-22 | `from` | `from` (result), final |
| AN-23 | `ref mut` result as a place | — |
| AN-24 | `@view`; local lists inferred | `from` (field) |
| AN-25 | `let` (C-8) | `from` (field) |
| AN-26 | `StrView`, `String` | final |
| AN-27 | `ref mut x: T` (C-1) | — |
| AN-28 | a refusal | — |
| AN-29 | `StrView` | — |
| AN-30 | row 4 | — |
| AN-31 | row 7: `reads` | — |
| AN-32 | `init`, `self` | — |
| AN-33 | `let` (C-8) | — |
| AN-34 | row 9: `copied`, `shared` | — |
| AN-35 | `copied` | — |
| AN-36 | a refusal (row 9) | — |
| AN-37, AN-38 | row 8: `ref mut path`; `mem.replace`, `mem.take` | — |
| AN-39 | `owned`, `shared`, `Pool`/`Handle`, `RuntimeChecked`, `Cell`, `Mutex`, `RwLock`, `Atomic` | **`thread_shared`**, **`weak`** |
| AN-40 | `owned T`, `owned dyn I` | `owned` on a foreign type |
| AN-41 | `Option[owned Tree]` | — |
| AN-42 | `owned`, `shared`, `class` | — |
| AN-43 | `owned` (C-2, C-3) | — |
| AN-44 | `copied`, `shared`, `class` (C-3) | — |
| AN-45, AN-46 | `copied` | — |
| AN-47 | a refusal (E0109's new help) | — |
| AN-48 | `shared`, `changes`, `ref mut`, `RuntimeChecked` | — (no `shared mut`, section 6) |
| AN-49 | `RuntimeChecked`, `read`, `write`, `ReadGuard`, `WriteGuard` | final |
| AN-50 | `Handle[T]` keeps the word; messages say "the `Player` object" and name the variable instead of "class handle" | — |
| AN-51, AN-52 | costs | — |
| AN-53, AN-54, AN-55 | row 6: `with copied`; `with owned` | capture list final (section 4) |
| AN-56 | `owned` on a lambda parameter's name | — |
| AN-57, AN-58 | messages, compiler | — |
| AN-59 | `once fn` | — |
| AN-60 | `changes`, `retain` after the loop | `with weak self` |
| AN-61 | a refusal | — |
| AN-62 | cost | — |
| AN-63 | — | **`weak`**, **`set by`** |
| AN-64 | the L3001 message | **`weak`**, **`set by`** |
| AN-65 | — | **`weak`** (no `upgrade`) |
| AN-66 | `shared dyn I`, `owned dyn I` | — |
| AN-67, AN-68 | `cast` | — |
| AN-69 | none | — |
| AN-70 | `virtual`, `open` | — |
| AN-71 | `thread.scope()`, `Atomic`, `@parallel(reduce=...)`, capture words | `thread_shared` (a task that outlives the scope) |
| AN-72, AN-73 | messages | — |
| AN-74 | `@thread_shared`, `ThreadShared`, `Atomic`, `Mutex` | **`thread_shared`** |
| AN-75 | — | **`thread_shared`**, `with thread_shared log` |
| AN-76 | `static mut`, `changes` | `changes` on a free function |
| AN-77 | `gen`, `yield` | — |
| AN-78 | `Retained`, the C contract words, `with owned` | `owned` on a foreign type |
| AN-79 | `unsafe` | — (section 6) |
| AN-80 | `ThreadShared`, `@thread_shared`, `Sync` (effect), `@nosync` | **`thread_shared`** |
| AN-81 | `@no_alias_checks` | final |
| AN-82 | removed | — |
| AN-83 | — | **`@fp(allow_fma)`** |
| AN-84 | error codes | — |

## 8. Points each area must rule on (words exist; the mechanism is the area's)

- **A:** none of the planned words is A's alone; AN-07 uses `from` only to return a list of views.
- **B:** `@cached`'s result type and its interplay with `Cell`; the `from` field form's heap-block
  condition (section 4).
- **C:** whether one owner of a `shared` value may change it, and how that is checked (AN-48); what
  `owned x: owned T` means on a parameter (vocabulary section 4).
- **D:** `weak`'s count while a bound name lives; `set by`'s full rules (last run's rule 6); a `weak`
  link to a `thread_shared` value (no new word planned: the target's header can carry whether its
  counts are atomic, as last run's count word did, `phase3b-combined.md:741`).
- **E:** whether `@thread_shared` stays once `thread_shared` exists; whether a `@thread_shared` class
  object held `shared` uses a one-thread count (this file's meaning: the count follows the word, for
  every type); safe `static mut` with `changes`; `@cached` across threads; the `unsafe fn` body rule.

## 9. Evidence (real output)

Every file is in `target/redesign3/07-word-planner/`; the full output is `check_output.txt` there.
Each NEW spelling is refused today, as expected; the existing words this file points to are accepted.

| file | what it tries | today |
|---|---|---|
| `p01_thread_shared.em` | `log = thread_shared Mutex(lines)` | `error[E1010]: cannot find 'thread_shared' in this scope`; exit 1 |
| `p02_weak_set_by.em` | `parent: weak Node set by children`, `children: Array[owned Node] = []` | `error[E1010]: cannot find type 'weak' in this scope`; `E0100 ... found 'owned'`; exit 1 |
| `p03_weak_value.em` | `cam.target = weak player` | `error[E1010]: cannot find 'weak' in this scope`; exit 1 |
| `p04_cached.em` | `@cached` over `fn radius(self) reads verts -> f64:` | `error[E0104]: '@cached' is not an attribute`; `E0100 ... found 'reads'`; exit 1 |
| `p05_capture_shared.em` | `run(fn() with shared log: println(log.len()))` | `error[E0100]: expected '=>' or ':' after a lambda's parameters`; exit 1 |
| `p06_changes_static.em` | `static mut FRAME: int = 0`, `fn tick() changes FRAME:` | `error[E1010]: 'static mut' needs 'unsafe', which is not in this phase`; `E0100 ... found 'changes'`; exit 1 |
| `p07_from_result.em` | `fn first_word(text: str) -> str from text:` | `E0110`, `E0100 ... found 'from'`; exit 1 |
| `p08_from_field.em` | `words: Array[str] from text = []` beside `let text: String` | `error[E0100]: expected the end of the line ... found 'from'`; exit 1 |
| `p09_weak_spec_example.em` | the spec's `Weak` example, 2591-2606 | accepted, exit 0, with `warning[L3001]: potential reference cycle: Node.children -> Node` and `help: if 'Node.children' is a back-reference, make it hold a 'Weak' handle` (the wrong edge, AN-64) |
| `p10_existing_list_words.em` | `counts.get_or("a", 0)`, `xs.retain(fn(x) => x % 2 == 0)` | accepted, exit 0, no output |
| `p11_fp_contract_today.em` | `@fp(contract)` | accepted, exit 0, no output |
| `p12_fp_allow_fma.em` | `@fp(allow_fma)` | `error[E0104]: '@fp' needs 'contract': '@fp(contract)'`; exit 1 |

`p09`, in full:

```text
warning[L3001]: potential reference cycle: Node.children -> Node
 --> p09_weak_spec_example.em:4:5
  |
4 |     children: Array[Node] = []
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ this strong field closes the shortest statically visible cycle
  |
  = help: if `Node.children` is a back-reference, make it hold a `Weak` handle
  = note: every edge in this statically visible cycle is strong; the cycle may leak at run time

exit=0
```

`p06`, first errors:

```text
error[E1010]: `static mut` needs `unsafe`, which is not in this phase
 --> p06_changes_static.em:2:12
  |
2 | static mut FRAME: int = 0
  |            ^^^^^
  |

error[E0110]: `tick` is declared without a body
 --> p06_changes_static.em:4:4
  |
4 | fn tick() changes FRAME:
  |    ^^^^ a function here needs a body
  |
  = help: add ':' and a body
```

Spellings checked against the standard library (Git Bash, `grep -rnw` over `std/src`): `weak`,
`thread_shared` and `cached` are used as no name in `std/src`; `by` appears only in comments.
