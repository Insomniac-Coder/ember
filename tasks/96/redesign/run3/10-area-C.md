# 10 area C: ownership and sharing, `owned` replacing `Box`, copies, hand-over, state machines (run 3, phase 2)

Inputs read in full: `RUN3/inputs/brief.md`, `RUN3/inputs/JOB.md`, the duty "08-12 Area designers A-E"
(`RUN3/inputs/duties.md`), `RUN3/vocabulary.md` (sections 0-4 and 6), `RUN3/planned-words.md`,
`RUN3/annoyances.md` (area C: AN-34 .. AN-52), last run's `tasks/96/redesign/phase3b-ownership.md` and
`phase3b-ownership-attack.md`.

How every claim can be traced:

- **Today's Ember**: every OLD program and every "means exactly" translation is a file in
  `target/redesign3/10-area-C/` (`anNN_old_*.em`, `anNN_tr_*.em`), checked with
  `target/redesign3/bin/ember.exe check <file>` by `run_checks.sh` there; the full output of every file is
  `target/redesign3/10-area-C/check_output.txt`. Nothing was run with `ember run`, nothing emitted.
- **C and Rust**: `target/redesign3/10-area-C/bench/` (`share_c.c` in four models, last run's
  `share_cell.rs` and `share_index.rs` copied, `an46_sorted_views.c`), built with
  `clang -O2 -I target/redesign/bench` and `rustc -C opt-level=2 --edition 2021`, timed with
  `target/redesign2/bench/run.ps1` (P-cores, mains checked every round, no window). Raw runs:
  `bench/runs_share.txt`, `runs_share2.txt`, `runs_share3.txt`; medians from `bench/medians.py`:
  `runs_share*_summary.txt`. Every timing is **PROVISIONAL**: other agents were running (spreads up to
  35%); the Timer (26) measures the final model.
- **Power**: `PowerLineStatus` printed `Online` at the start and before each of the three timing batches.
- No compiler change, no git command, nothing written outside this file and `target/redesign3/10-area-C/`.

---

## 0. The short version

**The model in plain words.**

1. **Every value has one owner unless a line says otherwise.** `b = a` hands `a` over, for every type
   whose copy would not be independent: lists, strings, structs that own memory, class objects
   (row 9; vocabulary C-3). Numbers, `bool` and `Copy` structs are copied, as Python's numbers behave.
2. **Three words before a value say every other case:** `copied a` (a new, independent value: rows 6,
   9), `shared a` (one more owner of the same value; row 9), `weak a` (a link that does not keep it
   alive; area D). `owned a` is refused: `b = a` already hands over.
3. **The same words in a type say how a field holds what it holds:** plain `T` (held alone: a struct's
   value inside its owner, a class object in its own block), `owned T` (held alone in its own heap block,
   the one-owner field that replaces `Box`, owner's ruling), `shared T`, `weak T` (D),
   `thread_shared T` (E).
4. **Any owner of a `shared` value may change it**, written as any change is written (a field write, a
   method whose first line says `changes f` (row 3), a `ref mut` name). **Nothing is checked while the
   program runs.** What keeps it memory-safe is a compile-time rule that reads only first lines and the
   function itself: a view into a part of an object that owns memory (a list, a string, an enum's
   payload) ends at any change that may reshape that part of an object that may be the same one. This
   replaces `RefCell`, which goes (AN-49).
5. **`class` no longer means "shared".** A class object lives in its own block, has identity (`is`),
   may be subclassed, and carries an 8-byte header (a count and a type index) so that `shared` on it
   moves nothing. A second owner is written `shared`, as for every type.
6. **State machines** are row 8 exactly: `State.Walking(ref mut path):` changes the NPC's own list in
   place; the two old forms get the agreed message.

**What still runs while the program runs** (compared with C): a count +1/-1 when a `shared` owner is
made or ends, as Rust's `Rc` (no count for a view of a shared object when nothing in the view's life
can end an owner of its type, AN-51); one allocation where a line says `owned` or `shared` of a value
that was not in its own block, as Rust's `Box::new`/`Rc::new`; the copy where a line says `copied`. No
borrow flags, no access words, no panics from sharing.

**What stays refused** (each with a message that names the line to write): using a name after it was
handed over (AN-34, AN-35, AN-36, AN-43, AN-44); changing a list, string or enum payload while a view
into it may still be used, through any owner (AN-48, the owner's "a list may not change while a loop
walks it"); a type that contains itself without `owned` (AN-41); `owned` where it can only copy, or
before a value (AN-43, AN-47); `.clone()`, `Box`, `RefCell`, `Shared`, `ForeignBox` (AN-40, AN-45,
AN-49).

| id | answer in one line | kind | what runs (vs C) |
|---|---|---|---|
| AN-34 | row 9: `copied items` / `items` / `shared items`; the error names both ways out | readable spelling | the copy, only where written |
| AN-35 | `n = ref p.name` or `copied p.name`; `v.push(copied template)`; messages | readable spelling | the copy, only where written |
| AN-36 | the lost write is refused, naming "set it before the push" and `shared` | readable spelling | nothing |
| AN-37 | row 8: `State.Walking(ref mut path):`, go idle in the same arm | readable spelling | nothing |
| AN-38 | row 8's two messages, as agreed | readable spelling | nothing |
| AN-39 | one family of words chosen by intent; eleven mechanisms become six words and two libraries | readable spelling | as each word |
| AN-40 | `owned T` replaces `Box[T]`, `Box[dyn I]` and `ForeignBox[T]` | readable spelling | the same block `Box` made |
| AN-41 | a self-containing type is an error naming `Option[owned Tree]` (or D's `weak ... set by`) | limitation kept | one block per node, as `Box` |
| AN-42 | one rule set for struct, `owned` and class: no access words, no panics, same refusals | smarter compile-time | nothing (today's access words go) |
| AN-43 | `owned` = hand-over for every non-`Copy` type, class included; refused where it can only copy | readable spelling | nothing |
| AN-44 | `b = a` hands over for every non-`Copy` type; a share is `shared a`; a struct copy changed later needs a word | readable spelling | nothing |
| AN-45 | `copied a` makes an independent object; `shared a` a second owner; `a.clone()` refused at a use | readable spelling | the copy, only where written |
| AN-46 | a `comptime` heap value is a read-only view of static data; `sorted(m.keys())` gives views; copies are written | smarter compile-time | less than today: no hidden copies |
| AN-47 | `owned` before a value is refused everywhere, E0109's help fixed, Annex A fixed | readable spelling | nothing |
| AN-48 | any owner changes a `shared` object; views checked at compile time by first lines | smarter compile-time | nothing beyond counts |
| AN-49 | `RefCell`, `Ref`, `RefMut`, `borrow`, `borrow_mut` removed; the beginner's E3023 names `changes` | smarter compile-time | less than today: no borrow flag |
| AN-50 | "handle" only for `Handle[T]`; a class object is "the `Player` object `p`" in every message | readable spelling | nothing |
| AN-51 | no count pair for a view of a shared object when no line in its life can end an owner of that type | smarter compile-time | 0 (today: a pair every step, 13.5% to 13.9%) |
| AN-52 | say the intent: `shared` (keeps alive: counts), kept `ref mut` views (C's speed), positions (faster) | new mechanism | 1.00x-1.06x (views), 1.13x-1.25x (`shared`), 0.80x-0.85x (positions) |

---

## 1. Terms

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

---

## 2. The rules of area C

- **C1, one owner.** Every value has one owner. `b = a` hands `a` over for every type that is not `Copy`
  (`Copy` as 06 narrowed it, C-3: numbers, `bool`, `ref T`, `Span`, structs of such; not class objects,
  not `shared` values, not `weak` links). Using a handed-over name is refused with the message pattern of
  row 9 (both ways out: `copied`, `shared`). A partial hand-over (`n = p.name`) leaves `p` unusable as a
  whole, as today, with a message naming `ref` and `copied`.
- **C2, `Copy` values.** `=` copies a `Copy` value. A number has no parts to change, so the copy cannot
  surprise (Python's `n2 = n; n2 += 1` is the same). A `Copy` **struct** copied by plain `=` from a place
  that lives on may not have a field changed through the new name: Python would change the original
  there (last run's attack 4.4, `copy_struct_name`), so the line must say which it means:
  `pos = copied p.pos` (your own copy) or `pos = ref mut p.pos` (change `p`'s).
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
- **C7, keeping an object alive under a view.** A view of an object reached through a `shared` link
  keeps it alive with no count when no line in the view's life may end an owner of that type (no write
  to a place of type `shared T`, no end of a local `shared T` owner, no call whose first line reaches
  one); otherwise with one count pair. This is AN-51.
- **C8, `class`.** A class object lives in its own heap block, has identity (`is`), may be subclassed
  (`open`, `virtual`), and has an 8-byte header (a 32-bit count, a 32-bit type index; last run's lean
  class, measured in section 5). The declaration no longer means "shared": a second owner is `shared`,
  for a class as for a list. One owner is the default for every type.
- **C9, no run-time borrow checks.** `RefCell`, `Ref`, `RefMut`, `borrow`, `borrow_mut` and the
  placeholder `RuntimeChecked[T]` are removed. `Cell[T]` stays (a whole-value replace, no reference, no
  check; 06's C-10 condition is area B's).
- **C10, one thread.** `shared` never crosses threads (area E's `thread_shared`).

Agreed rows used: 3 (`changes`), 6 (`copied` in a capture), 8 and 9 (this area's), unchanged.

---

## 3. The answers

### AN-34: `snapshot = items` hands the list over; using `items` after is E3040, whose help says "clone"

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

   # the other two ways the line can be written, each meaning what its word says:
   log = items                       # hands the list over: `items` ends here
   other = shared items              # one list, two owners: a change through either is seen by both
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `items` | none (default: the list made on this line belongs to the name it is given) | none (default: Python's locals: the function's own, which it may change) |
   | `snapshot` | `copied`: its own new list | none (default: Python's locals) |
   | `log` | none (default, row 9: `=` hands the list over; `items` ends) | none (default: Python's locals) |
   | `other` | `shared`: one of two owners of the same list | none (default: Python's locals; any owner may change it, C6) |

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

### AN-35: partial move (E3042) and move in a loop (E3041)

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
       n = ref p.name                # a view of p's name: p stays whole
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
   | `n` (first function) | `ref`: a view of `p`'s name | `ref` without `mut`: reads only |
   | `template` | none (default: a parameter reads the caller's value) | none (default: a parameter never changes the caller's value) |
   | `n` (parameter) | none (default: an `int` parameter, a copy of a number, C2) | none (default: never changes the caller's) |
   | `v` | none (default: the list made on this line belongs to the name) | none (default: Python's locals) |
   | `_` | none (default: a number from the range, never kept) | none |

5. **Means exactly:** `an35_tr_ref.em` (today's `n = ref p.name` then `keep(p)`: `exit=0`, no output);
   `an35_tr_loop.em` (`template: Player`, `v.push(template.clone())`: `exit=0`, no output).
6. **Still refused:**

   ```text
   error[E3042]: `keep(p)` needs the whole player, but `n = p.name` took the name out of `p` on line 9
     = note: `=` hands a field over, so `p` is no longer whole
     = help: to read the name write `n = ref p.name`; for your own copy write `n = copied p.name`
   ```

   ```text
   error[E3041]: `v.push(template)` hands `template` over on the first pass, so the second pass has none
     = help: for a new player each pass write `v.push(copied template)`; for one player in every slot write `v.push(shared template)` with `v: Array[shared Player]`
   ```

7. **Cost:** `ref p.name`: nothing (a pointer). `copied template`: one copy per pass, as C's per-element
   copy and Rust's `.clone()` in the loop.
8. **Kind:** a readable spelling of Rust's guarantee.

### AN-36: a write to a handed-over value is accepted and lost

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

### AN-37: moving a field out from behind `mut self` (`match owned self.state` is E3013)

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
   error[E3025]: `self.state = State.Idle` replaces the walk list that `path` (line 12) still uses on line 14
     = help: use `path` before this line, or set the new state after the `match`
   ```

7. **Cost:** nothing: `path` is a pointer into the enum's payload; the pop and the store are C's.
8. **Kind:** a readable spelling of Rust's guarantee (Rust's `Walking(path) => ` on a `&mut self.state`,
   with the `&mut` written on the name it gives).

### AN-38: `State.Walking(path):` then `path.pop()` is E3021 with a `.clone()` help; `mut path` is accepted read-only

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
   error[E3021]: `path.pop()` changes the walk list, but `path` only reads it
     = help: to change that list through `path`, write `State.Walking(ref mut path):`
   ```

   ```text
   error[E0100]: `mut path` in a pattern says "may change" but not whose list `path` is
     = help: to change that list through `path`, write `State.Walking(ref mut path):`
   ```

7. **Cost:** nothing.
8. **Kind:** a readable spelling of Rust's guarantee.

### AN-39: choosing among ownership kinds: eleven mechanisms

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

### AN-40: `Box[T]` says nothing about one owner; `ForeignBox[T]` carries the same word

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

### AN-41: a type that contains itself crashes the checker

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

### AN-42: `struct`, `Box` or `class` decides what compiles and what panics

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

       fn get_mut[T](self, i: int) -> ref mut T from self changes pools:
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

   `ref mut cast ... as Pool[T]` needs a `cast` of a place to be a place (area D's AN-67/68); the
   returned view's `from self` is area B's clause.
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

### AN-43: `owned` on a parameter means three things by type; `mem.drop(owned x)` moves a handle

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
   declare `n: shared Node`".
7. **Cost:** nothing: a hand-over passes the pointer or the value as today, and no count is made for a
   class (today's `[FN-9]` retains the caller's handle for the call).
8. **Kind:** a readable spelling of Rust's guarantee (`owned` is Rust's move, for every type).

### AN-44: `b = a` has no word and does three things by type

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
   error[E3045]: `pos.x += 1.0` changes a copy of `p.pos`, not `p`'s position
     = note: `pos = p.pos` copied the position (a `Copy` struct), so `p` does not see this change
     = help: to change `p`'s position write `pos = ref mut p.pos`; to change your own copy write `pos = copied p.pos`
   ```

7. **Cost:** nothing; `shared player` is a count +1 (and -1 when `p2` ends), as Rust's `Rc::clone`.
8. **Kind:** a readable spelling of Rust's guarantee (Rust's move, `Copy`, `Rc::clone`), with the
   struct copy that Rust also makes silently now named.

### AN-45: `.clone()` on a class handle shares the object instead of copying it

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
   error[E2020]: `a.clone()`: a copy is written `copied a`
     = help: for a new player write `b = copied a`; for a second owner of this one write `b = shared a`
   ```

   `copied a` of a class with no `Clone`: "`Player` does not say how to copy it; add `@derive(Clone)`
   above `class Player`, or write `b = shared a` for a second owner".
7. **Cost:** the copy where `copied` is written (one block and the fields), as C's `malloc` + copy and
   Rust's `.clone()` of a `Box`.
8. **Kind:** a readable spelling of Rust's guarantee.

### AN-46: silent copies no word names: `comptime(e)` of a heap value; `sorted(m.keys())`

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
   error[E3021]: `t.push(4)` changes `t`, a view of a table made when compiling, which never changes
     = help: for your own list write `t = copied comptime(table())`
   ```

   ```text
   error[E3020]: `m.insert(...)` changes `m`, whose keys `keys` (line 7) still views on line 9
     = help: use `keys` before this line, or keep your own strings: `keys = sorted(m.keys().copied())`
   ```

7. **Cost:** less than today: no allocation per evaluation of the table (read-only data, as C's
   `static const`), and no string copy per key (an array of pointers, as the C above and Rust's
   `let mut ks: Vec<&String> = m.keys().collect(); ks.sort();`). Copies only where `copied` is written.
8. **Kind:** a smarter compile-time solution (the value is placed in read-only data and lent; the
   sorted keys are a local list of views, area A's AN-07 rule).

### AN-47: `owned` in an expression: E0109 contradicts Annex A, and its help is wrong for `b = owned a`

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
   error[E0109]: `owned fn` does not say what the function takes
     = help: name what it takes: `fn() with owned report:`; `task` captures nothing, so write `fn() => println("done")`
   ```

7. **Cost:** nothing.
8. **Kind:** a readable spelling of Rust's guarantee (a message that gives the line).

### AN-48: shared mutable state: a camera and a HUD both change one player

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

   fn transfer(a: shared Player, b: shared Player):
       if a is not b:
           for it in a.inventory:
               b.inventory.push(copied it)        # accepted: the compiler knows they differ
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `hp`, `inventory`, `log` | none (default: a field holds its value in the player) | through the name before them |
   | `target` (both classes) | `shared`: one of several owners of a player | `changes target` on `shake`; through `cam`, `hud` (the function's own) in `main` |
   | `self` (in `shake`) | `self` | `changes target` |
   | `p`, `cam`, `hud` | none (default: the object made on this line belongs to the name) | none (Python's locals) |
   | `item`, `it` | none (default: a loop name reads each element, never changes, never copies) | none (reads only) |
   | `a`, `b` | `shared` in the parameter type: the function holds an owner of each | none (default: any owner may change it, C6) |

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

### AN-49: `RefCell` hides a run-time check that panics; E3023 offers it to a beginner who forgot `mut`

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
   error[E3023]: `bump` changes `self.n`, but its first line does not say so
     = help: write `fn bump(self) changes n:`
   ```

   The conflict `an49_old_refcell_conflict.em` panics on becomes a compile-time refusal: a view of
   `s.entities` (`a = ref s.entities`) still used while `s.entities.push(...)` runs: "`s.entities.push`
   changes the list `a` views (line 6), which it still uses on line 8; use `a` before, or keep a copy:
   `a = copied s.entities`".
7. **Cost:** less than today: no borrow flag (one word per `RefCell`, checked on every borrow in every
   profile, `[CELL-5]`, `[CELL-9]`); nothing runs.
8. **Kind:** a smarter compile-time solution (Rust's run-time-checked `RefCell` replaced by C6's
   compile-time rule).

### AN-50: "handle" names a class object's counted pointer and also `Handle[T]`

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

### AN-51: a count pair the spec allows to remove is emitted every loop step

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

   fn step(ws: Array[Watcher], frame: i32) -> u64:
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

   `ws: Array[Watcher]` is a read-only parameter, and `t = ref mut ws[i].target` changes the player
   through it: under C6, a view of a `shared` link may change the shared object (any owner may), while
   `ws` itself (its list, its links) does not change. The parameter is then written
   `ref mut ws: Array[Watcher]` if area B rules that changing an object reached through a link counts as
   changing the parameter (06's `let` rule C-8 says a class-object field's object changed through it is a
   change of the field); this answer works under either ruling.
4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `hp`, `x`, `inventory` | none (default: a field holds its value) | through the name before them |
   | `target` | `shared`: one of several owners of a player | through `t` |
   | `ws` | none (default: a parameter reads the caller's list) | none (default: the list itself is not changed) |
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

### AN-52: shared class objects re-pointed often ran at 1.16x to 1.17x plain C (last run's limit)

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
   error[E3020]: `players.remove(r)` would end a player that `ws` (line 14) still points at on line 30
     = help: remove players after the last use of `ws`, or let the watchers keep them alive: `target: shared Player`
   ```

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

---

## 4. Last run's reports, repaired

| last run's answer (`phase3b-ownership.md`, `-attack.md`) | this run | why | good parts kept |
|---|---|---|---|
| A11 "names follow values": `b = a` makes a second name, as Python | **replaced** by row 9 (`=` hands over; `shared`, `copied` written) | the owner agreed row 9; a second name with no word aliases, which the brief forbids | moving the old value into the name instead of dropping it (now: a re-point stores the new link before ending the old owner, C6); the register form of a compare where a write may replace a named place (attack 4.3, 1.00x of correct C); the error that offers both meanings instead of one fix-it (attack 4.5), kept in AN-35's and AN-44's messages |
| A10 `Box` implied by the compiler | **replaced** by `owned T` written (owner's ruling) | the compiler would decide what the code does not say | the error naming the fix instead of a crash (AN-41) |
| A10 two words, `struct` and `class`; `Shared`, `Cell`, `RefCell` gone | **kept in part**: `RefCell` and `Shared` go, `Cell` stays, `class` no longer means shared | the owner wants one owner or shared written in words | the lean class (8-byte header), measured again (section 5); the type index at 0.99x-1.03x of a type pointer (attack 5.4) |
| A10 one kind, representation chosen by the compiler / a "never shared" lint | **discarded** | layout would depend on distant code (last run's own reason) | the lint, reworded: "`class Particle` uses no identity or subclass; as a `struct` it is stored inline" |
| A9 field effects inferred from bodies, recorded in the interface | **kept as written first lines**: row 3's `changes`, row 7's `reads` | the owner agreed the written clause; nothing inferred across packages | the whole conflict rule (C6), by class and field; `drop` effects counted at a re-point; the attack's "unknown code has every effect" |
| A9 `if a is b: panic(...)` before a transfer | **replaced** by `if a is not b:` written by the program (AN-48) | a panic where the compiler can ask for one test | one compare, not a flag per borrow |
| A9 the lean class's speed "open" (1.16x-1.17x from a slab) | **saved in part** (AN-52): the intent is written, and two of three intents run at C's speed | counting cannot skip the touch of the new target | the measurements, and the attack's "uncounted stored handle" idea, built here as kept views (b) |
| A34 `owned fn` capturing a counted handle | **replaced** by `with thread_shared log` (07, area E) | `owned fn` copies an `int` silently (owner's named failure) | one count per capture, Rust's `Arc::clone` |
| A31 `@cached` | moved to area B (07's `@cached`) | | |
| A18, A21, A1 | moved to areas B and A by 05 | | |

---

## 5. Measurements (PROVISIONAL)

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

---

## 6. New words, clashes, and points for the combiner

**New words: none.** Every word used is in `vocabulary.md` or `planned-words.md`: `copied`, `shared`,
`owned` (value, type, parameter, capture positions as planned), `weak` and `set by` (area D's, only
named), `thread_shared` (area E's, only named), `ref`, `ref mut`, `changes` (row 3), `class`, `Cell`,
`Pool`, `Handle`, `mem.replace`, `mem.drop`, `.copied()`, `@derive(Clone)`, `@view`, `is not`, `cast`,
`from` (area B's clause), `String(...)` (row 5).

**Meanings this area sets** (no spelling changes):

- `class`: no longer "a shared, counted handle"; an object in its own block with identity and
  subclasses, one owner unless `shared` (C8). The vocabulary's `class` row allowed this (06's condition:
  `p2 = player` hands over or is refused).
- `owned` in a type is refused on a class type (it adds nothing); `owned m: owned Mesh` is accepted on a
  parameter (C5).
- `RuntimeChecked[T]`, `ReadGuard`, `WriteGuard` and the `RefCell` methods `read`/`write` are removed
  (the vocabulary allowed area C to remove the type); `read()`/`write()` on `RwLock` are untouched.
- `[OWN-3]`: assigning a field of a handed-over name is refused (AN-36).

**Clashes for 14 to settle:**

1. **Kept `ref mut` views that name one object** (AN-52 (b)): another area may assume every `ref mut` is
   the only name for its target. This answer keeps that for parameters (caller-checked) and drops it for
   views kept in a list or struct; it also needs area B's AN-24 (a local list of view structs).
2. **A change through a link reached from a read-only parameter** (AN-51's `ws`): whether
   `t = ref mut ws[i].target` needs `ref mut ws` depends on area B's reading of 06's `let` rule C-8.
3. **`cast` of a place is a place** (AN-42's `get_mut`): area D (AN-67, AN-68) or B.
4. **`Box[T, A]` with an allocator** (`[ALC-1]`) has no spelling once `Box` goes; no annoyance needs it.
5. **`shared` and threads**: area E rules what happens when a `shared` owner reaches `thread.spawn`
   (C10 assumes it is refused).

---

## 7. Files

All under `C:\Users\ism19\Code\ember\target\redesign3\10-area-C\`:

- `anNN_old_*.em` (every OLD program), `anNN_tr_*.em` (every "means exactly" in today's Ember),
  `run_checks.sh`, `check_output.txt` (the full output of `ember check` on all 63 files, as pasted).
- `bench/share_c.c` and its four builds `share_c_{0,3,8,9}.exe`; `bench/share_cell.rs`,
  `share_index.rs`, `harness.rs` (copied from `target/redesign2/ownership-attack2/bench/`) and
  `rs_share_cell.exe`, `rs_share_index.exe`; `bench/an46_sorted_views.c`; `bench/medians.py`;
  `bench/runs_share.txt`, `runs_share2.txt`, `runs_share3.txt` and their `_summary.txt`.
