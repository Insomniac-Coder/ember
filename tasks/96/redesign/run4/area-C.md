# Area C under the run-4 rules: ownership and sharing, `owned` for `Box`, copies, hand-over, state machines (AN-34 to AN-52)

Run 4, phase 1, agent 04 (area C). Every answer of area C (`RUN3/combined-v4.md` lines 5656-7304),
brought to the rules of `RUN4/rules.md` and the words of `RUN4/vocabulary.md`. `RUN3` means
`tasks/96/redesign/run3/`, `RUN4` means `tasks/96/redesign/run4/`; "R1" .. "R18" are the run-4 rules;
"row N" is the owner's agreed row N in `inputs/JOB.md`; run 3's rules are written "run 3's C6" or
"run 3's R2". All paths are under `C:\Users\ism19\Code\ember\`.

Power: `PowerLineStatus` printed `Online` at the start of this work and again before writing this file.
No compiler change, no git command, no Ember program run; the only command run on Ember code was
`target/redesign4/bin/ember.exe check` on two probes (AN-40, AN-43; output pasted there, and in
`target/redesign4/04-area-C/probes/check_output.txt`).

How this file was made: the OLD part of every answer (its code and today's `ember check` output) is
carried over unchanged, since no OLD program changed; it was put in word for word from
`RUN3/combined-v4.md` by `target/redesign4/04-area-C/build_area_c.py`, so no OLD block was retyped.
Everything else was rewritten for the run-4 rules.

## Terms

The terms of `RUN4/rules.md` section 0 (owner, handed over, copy, `Copy` type, place, part, view, kept
view, into, link, position, reshape, `this`, first line, `shared` value, count, count pair, deferred-drop
region, the owner's test). Two more, used in the owner's-test tables below:

- **own local** (the default for a local name given a new value): the value made on the line belongs
  to the name (R1), so changing it changes nothing anyone else holds. It never aliases (a second name
  for the value needs `shared`, `sharedref mut` or a view word) and never copies silently. Python has
  no word for this either.
- **reads only** (the default for a plain parameter, a plain loop, pattern or opened name, and a name
  bound to a part): a read-only view; it never changes anything, never aliases a changeable name, never
  copies (a number is copied in, which no change can tell apart; R2, R4, R5).

## Run 3's area rules, and the run-4 rules that now say them

Area C's own rules C1 to C10 are gone as separate rules; every answer below cites the run-4 rule
instead.

| run 3 | run 4 | what is the same, and what changed |
|---|---|---|
| C1, one owner | R1, R2 | the same: `=` with a whole name hands it over; a name for a part is a read-only view |
| C2, `Copy` values | R1, R2 | the same: `=` copies a `Copy` value; a `Copy` struct copied with `=` is not then changed through the new name (E3023) |
| C3, words before a value | R3 | the same, plus `sharedref mut x` (owner, 2026-10-25) |
| C4, words in a type | R3 | the same; `owned` on a class type now has its code, E2114 |
| C5, parameters | R4 | the same modes; a call now writes `ref mut` where its parameter does (owner, 2026-10-25) |
| C6, changing a shared value | R8 (b), R9 | the same compile-time rule; `ref mut` is now alone everywhere (R8 (f)), and a changeable link several holders share is `sharedref mut` (R7) |
| C7, keeping an object alive | R10 | the same, with the deferred-drop region as one more cheap way |
| C8, `class` | R9, the `class` row of `RUN4/vocabulary.md` | the same 8-byte header; one owner unless a line says `shared` |
| C9, no run-time borrow checks | R9 | the same |
| C10, one thread | R9, R15 | the same |

What the owner's four decisions of 2026-10-25 change in this area: `self` becomes `this`, and a
method's first line no longer lists its object (AN-37, AN-38, AN-42, AN-48, AN-49, AN-50); a call to a
`ref mut` parameter says `ref mut` (AN-37's message, AN-48, AN-51); kept `ref mut` fields that several
holders share become `sharedref mut` (AN-48, AN-52). No answer of this area opened an `Option` or a
`weak` link with `if Some(...)`, so the new opening form appears in none of them.

## The answers

#### AN-34: `snapshot = items` hands the list over; using `items` after is E3040, whose help says "clone"

**Under run 4:** meaning unchanged. The hand-over form now uses row 9's own name, `snapshot = items`
(run 3 wrote `log = items`), so the row appears exactly as agreed.

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

   # the other two ways the line can be written, each meaning what its word says,
   # each a program of its own (they are alternatives, not one sequence):
   fn hand_over():
       items = [1, 2, 3]
       snapshot = items              # hands the list over: `items` ends here
       snapshot.push(4)

   fn two_owners():
       items = [1, 2, 3]
       other = shared items          # one list, two owners: a change through either is seen by both
       other.push(4)
       println(items.len())          # 4
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `items` (all three) | none (default, own local: the list made on this line is the name's own) | none (default, own local); in `two_owners`, from `other = shared items` on, one of two owners, and any owner may change the list (R9) |
   | `snapshot` in `main` | `copied`: its own new list | none (default, own local) |
   | `snapshot` in `hand_over` | none (default, row 9: `=` with a whole name hands the list over; `items` ends) | none (default, own local) |
   | `other` | `shared`: one of two owners of the same list | none (default: any owner may change it, R9) |

5. **Means exactly** (today's Ember; all three accepted, `exit=0`, no output, as run 3 pasted them):
   `an34_tr_copied.em` (`snapshot = items.clone()`), `an34_tr_handover.em` (`snapshot = items`,
   `items` not used again), `an34_tr_shared.em` (`items = Shared([1, 2, 3])`, `other = items`,
   `items.get_mut().push(4)`, `other.get().len()`; today's `Shared` checks each `get_mut` while running,
   which the NEW `shared` does not, R9). Files under `target/redesign3/10-area-C/`.
6. **Still refused** (R1, E3040; row 9's message, both ways out named):

   ```text
   error[E3040]: `items` was handed over to `snapshot` on line 3 and is used on line 4
     = note: `snapshot = items` hands the list over; `items` ends there
     = help: to keep both, write `snapshot = copied items` (a copy of your own) or `snapshot = shared items` (two owners of one list)
   ```

   With `other = shared items`, a `for` loop over `items` that pushes through `other` is refused (R8 (c),
   E3020; the message is AN-48's).
7. **Cost:** `copied`: one allocation and a copy of the elements, as C's `malloc` + `memcpy` and Rust's
   `.clone()`. `snapshot = items`: nothing (the list header moves, three words). `shared items`: one
   counted block for the list (count and list header together, as Rust's `Rc::new(vec)`), made where
   `items` is made because a line says `shared`; a count +1 per owner made, -1 per owner ended.
8. **Kind:** a readable spelling of Rust's guarantee (Rust's move, `.clone()` and `Rc`, each named by the
   word that says it).

#### AN-35: partial move (E3042) and move in a loop (E3041)

**Under run 4:** meaning unchanged. The loop name `_` is written `i` (`_` has no row in
`RUN4/vocabulary.md`; see "Words needed").

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
       n = p.name                    # a read-only view of p's name (R2): p stays whole
       println(n)
       keep(p)                       # after n's last use; a hand-over to an `owned` parameter has no word (R4)

   fn spawn_all(template: Player, n: int) -> Array[Player]:
       v: Array[Player] = []
       for i in 0..n:
           v.push(copied template)   # a new player each pass
       return v
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `p` | `owned` on the parameter: handed over to this function | none (default, own local: it is the function's own) |
   | `n` (first function) | none (default, reads only: a name for a part of `p`, R2, as row 8's plain `path`) | none (default, reads only) |
   | `template` | none (default, reads only: a plain parameter, R4) | none (default, reads only) |
   | `n` (parameter) | none (default: an `int` parameter, a number copied in, R4) | none (default: never changes the caller's) |
   | `v` | none (default, own local: the list made on this line) | none (default, own local) |
   | `i` | none (default, reads only: a number from the range) | none (default, reads only) |

5. **Means exactly:** `an35_tr_ref.em` (today's `n = ref p.name` then `keep(p)`: `exit=0`, no output);
   `an35_tr_loop.em` (`template: Player`, `v.push(template.clone())`: `exit=0`, no output).
6. **Still refused:**

   ```text
   error[E3060]: `n` views `p.name`, and `keep(p)` hands `p` over on line 11 while `n` is used on line 12
     = help: use `n` before line 11, or keep your own copy: `n = copied p.name`
   ```

   ```text
   error[E3013]: `v.push(template)` would hand over the caller's player, but `template` only reads it
     = note: `template: Player` is a plain parameter: a read-only view of the caller's player, which stays the caller's
     = help: to put a player of your own in the list write `v.push(copied template)`
   ```

   (R2 for the first: a view used after what it points into was handed over; R4 and the E3013 row of
   `RUN4/rules.md` section 3 for the second, whose help is that row's, word for word.)
7. **Cost:** `n = p.name`: nothing (a pointer). `copied template`: one copy per pass, as C's per-element
   copy and Rust's `.clone()` in the loop.
8. **Kind:** a readable spelling of Rust's guarantee.

#### AN-36: a write to a handed-over value is accepted and lost

**Under run 4:** meaning unchanged. The first form now declares `root`, so every name the block uses is
declared in it.

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
   root = Node(value=0)
   child = Node(value=1)
   child.value = 5
   root.children.push(child)            # hands the child over (R2: a value stored somewhere else)
   root.children[0].value = 6

   # or the child has two owners, the tree and this function (Python's meaning)
   struct Node:
       value: int
       children: Array[shared Node] = []

   root = Node(value=0)
   child = Node(value=1)
   root.children.push(shared child)
   child.value = 5                      # seen as root.children[0].value
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `root`, `child` | none (default, own local: the node made on this line) | none (default, own local) |
   | `value` | none (default: a field holds its value inside its node) | none (default: changed through a name that may change the node) |
   | `children` | first form: none (default: a list held by its node); second: `shared` in the element type, each child has several owners | as `value` |
   | `child` after `push(shared child)` | `shared` on that line: one of two owners | none (default: any owner may change it, R9) |

5. **Means exactly:** `an36_tr_before.em` (`exit=0`, no output); `an36_tr_shared.em` (a class `Node`,
   which is today's only spelling of "two owners and both change it"), as run 3 pasted it:

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

6. **Still refused** (R1, E3040; today accepts it, so `[OWN-3]` gains "assigning a field of" a
   handed-over name):

   ```text
   error[E3040]: `child` was handed over to `root.children` on line 8 and is changed on line 9, which would change nothing
     = note: after `root.children.push(child)` the node belongs to the list
     = help: set it before the push (`child.value = 5` above line 8), or change it in the list: `root.children[0].value = 5`; for two owners write `root.children.push(shared child)` with `children: Array[shared Node]`
   ```

   Warned, not refused: the second form's `children: Array[shared Node]` lets a `Node` own itself
   through its children, so it gets the cycle warning L3001 (R11), as today's class form above does;
   the warning's help in run 4's words is "if `children` can lead back to a node that holds it, link it
   with `weak Node` instead".
7. **Cost:** nothing: a compile-time refusal. The `shared` form pays one counted block per child and
   the counts, as Rust's `Rc<Node>`.
8. **Kind:** a readable spelling of Rust's guarantee (Rust refuses the same write, `E0382`, run 2's
   `rust_probes/a11_assign_moved_field.rs`).

#### AN-37: moving a field out from behind `mut self` (`match owned self.state` is E3013)

**Under run 4:** meaning unchanged; `self` becomes `this` (owner, 2026-10-25): the method no longer
lists its object, and its first line is `fn update() changes state:`, row 3's form without `self`. The
help for taking the state out writes `ref mut` at the call (`mem.replace(ref mut this.state, ...)`, R4).
The `_:` arm is written as its two variants (`_` has no row in `RUN4/vocabulary.md`).

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

       fn update() changes state:
           match this.state:
               State.Walking(ref mut path):
                   path.pop()
                   if path.is_empty():
                       this.state = State.Idle
               State.Idle:
                   pass
               State.Dead:
                   pass
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `this` | `this`: the NPC the method was called on (R5) | `changes state` on the first line: this method may change `state` only (row 3) |
   | `state` | none (default: a field holds its value inside the NPC) | `changes state` |
   | `path` | `ref mut`: a name for the NPC's own walk list, the only one while it is used (row 8, R8 (f)) | `mut` in `ref mut` |

5. **Means exactly:** `an37_tr_refmut_inline.em` (the same body with `fn update(mut self):`, today's
   spelling of `changes state`): `exit=0`, no output. Today already ends `path` at its last use
   (`path.is_empty()`), so `self.state = State.Idle` inside the arm is accepted.
6. **Still refused:**

   ```text
   error[E3013]: `match owned this.state` would take the state out of the NPC and leave it with none
     = note: `update` says `changes state`: it may change the NPC in place, but the NPC stays its caller's, so nothing can be taken out of it
     = help: to change the path where it is write `State.Walking(ref mut path):`; to take the state out and put another in write `match owned mem.replace(ref mut this.state, State.Idle):`
   ```

   And, while `path` is still used, `this.state = State.Idle` before `path.pop()` (R8 (b): changing an
   enum's variant is a reshape of the payload `path` names):

   ```text
   error[E3060]: `this.state = State.Idle` (line 13) replaces the walk list that `path` (line 12) still uses on line 14
     = help: use `path` before this line, or set the new state after the `match`
   ```

7. **Cost:** nothing: `path` is a pointer into the enum's payload; the pop and the store are C's.
8. **Kind:** a readable spelling of Rust's guarantee (Rust's `Walking(path) => ` on a `&mut self.state`,
   with the `&mut` written on the name it gives).

#### AN-38: `State.Walking(path):` then `path.pop()` is E3021 with a `.clone()` help; `mut path` is accepted read-only

**Under run 4:** meaning unchanged; `self` becomes `this`. The NEW is written out in full (run 3 wrote
"as in AN-37") for the OLD's own class and enum. Row 8's two agreed helps are word for word.

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
3. **NEW** (row 8, exactly as agreed):

   ```ember
   class Npc:
       state: State = State.Idle

       fn update() changes state:
           match this.state:
               State.Walking(ref mut path):
                   path.pop()
               State.Idle:
                   pass
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `state` | none (default: a field holds its value inside the NPC) | `changes state` |
   | `this` | `this`: the NPC the method was called on | `changes state` on the first line (row 3) |
   | `path` | `ref mut`: a name for the NPC's own walk list (row 8) | `mut` in `ref mut` |

5. **Means exactly:** `an38_tr_refmut.em` (reader 4's `t05`): `exit=0`, no output.
6. **Still refused** (the two agreed messages; the helps word for word as row 8):

   ```text
   error[E3023]: `path.pop()` changes the walk list in `this.state`, but `path` only reads it
     = help: to change that list through `path`, write `State.Walking(ref mut path):`
   ```

   ```text
   error[E0113]: `mut path` says `path` may change but not whose list it is
     = help: to change that list through `path`, write `State.Walking(ref mut path):`
   ```

   And the first line without its clause, `fn update():` with the same body: "`update` changes
   `this.state` through `path`; write its first line as `fn update() changes state:`" (E2105, R5).
7. **Cost:** nothing.
8. **Kind:** a readable spelling of Rust's guarantee.

#### AN-39: choosing among ownership kinds: eleven mechanisms

**Under run 4:** meaning unchanged. The table of kinds gains `sharedref mut T` (owner, 2026-10-25), and
its `ref mut` row now says "alone" (R8 (f)).

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
   | a name for someone else's place that reads | `ref T` (any number of names) | the same |
   | a name for someone else's place that changes it, the only one while used | `ref mut T` | the same |
   | a changeable link several holders share, not owning | `sharedref mut T` (R7) | two kept `ref mut` views of one object, which `ref mut` no longer allows |
   | one value replaced whole through a read-only path | `Cell[T]` | the same |
   | a value threads change | `Mutex`, `RwLock`, `Atomic` (area E) | the same |
   | objects freed together, or found by position | `Arena`, `Pool` + `Handle[T]` (libraries) | the same |
   | a borrow checked while running | removed (AN-49) | `RefCell` |
   | an object with identity and subclasses | `class` (one owner unless `shared`) | `class` |

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `verts` | none (default: a field holds its value inside the mesh) | none (default: changed through a name that may change the mesh) |
   | `mesh` | `shared` in the type: one of several owners of a mesh | none (default: any owner may change it, R9; here through `a`, the function's own) |
   | `m` | none (default, own local: the mesh made on this line; after `shared m` it is one of the owners) | none (default, own local) |
   | `a`, `b` | none (default, own local: the entity made on this line) | none (default, own local) |

5. **Means exactly:** `an39_tr_shared_mesh.em` (`mesh: Shared[Mesh]`, `a.mesh.get_mut().verts.push(1.0)`):
   `exit=0`, no output. Today's `get_mut` is checked while running; the NEW change is checked when
   compiling (R8 (b), R9).
6. **Still refused** (R1, E3040):

   ```text
   error[E3040]: `m` was handed over to `a`'s mesh on line 10 and is used on line 11
     = help: for one mesh in both entities write `mesh=shared m` in both (with `mesh: shared Mesh`); for two meshes write `mesh=copied m`
   ```

   `Box`, `Shared`, `SyncShared`, `Weak`, `RefCell`: each refused with the word that replaces it
   (E1010: "write `owned T`", "write `shared T`", and so on; AN-40, AN-49).
7. **Cost:** one counted block for the mesh and a count +1/-1 per owner made or ended, as Rust's
   `Rc<Mesh>`; no borrow flag (Rust needs `Rc<RefCell<Mesh>>` to edit it, and pays a flag per borrow).
8. **Kind:** a readable spelling of Rust's guarantee (one family of words, each saying its intent).

#### AN-40: `Box[T]` says nothing about one owner; `ForeignBox[T]` carries the same word

**Under run 4:** meaning unchanged. `owned` on a class type now has its code (E2114, R3), and the
`.get()` refusal has today's code for a missing method (E2072, probe below).

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
3. **NEW** (the owner's ruling; `owned` on a foreign type):

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
   | `mesh` | `owned`: held alone, in its own block | none (default: changed through a name that may change the entity) |
   | `pools` | `owned dyn Pool` elements: each pool held alone | as `mesh` |
   | `db` | `owned` on a foreign type: held alone, freed through `sqlite3_close` | as `mesh` |
   | `e`, `r` | none (default, own local: the value made on this line) | none (default, own local) |
   | `verts`, `items` | none (default: a field holds its value) | as `mesh` |

5. **Means exactly:** `an40_old_box.em` itself (`Box[Mesh]`, `Box[dyn Pool]`): `exit=0`, no output; the
   foreign case is today's `ForeignBox[SqliteDb]` (`[FFI-36a]`, spec 4883-4887).
6. **Still refused:**

   ```text
   error[E1010]: `Box` is not part of Ember
     = help: for a mesh held alone in its own block write `mesh: owned Mesh`; for `Box[dyn Pool]` write `owned dyn Pool`
   ```

   `ForeignBox[SqliteDb]` (E1010): "write `owned SqliteDb`". `owned Player` on a class type (E2114, R3):
   "a `Player` object always lives in its own block; remove `owned`: `p: Player`". `e.mesh.get()` on an
   `owned` field (E2072, today's code for a missing method): "an `owned Mesh` reads as a `Mesh`, which
   has no `get`; remove `.get()`: `e.mesh.verts`". Today's E2072, on a probe of the same shape
   (`target/redesign4/04-area-C/probes/c2_no_method.em`, `m.get()` on a `struct Mesh`):

   ```text
   === c2_no_method.em
   error[E2072]: `Mesh` has no method named `get`
    --> c2_no_method.em:6:15
     |
   6 |     println(m.get().verts.len())
     |               ^^^
     |

   error: could not compile due to 1 error(s)
   exit=1
   ```

7. **Cost:** the block `Box` made: one allocation at the store, one pointer in the owner, freed with it;
   the same as C's `malloc` and Rust's `Box`. Not covered: `Box[T, A]` with an allocator (`[ALC-1]`,
   spec 2986) has no spelling; no annoyance of this area needs it (`RUN4/changes.md` section 13, item 7).
8. **Kind:** a readable spelling of Rust's guarantee.

#### AN-41: a type that contains itself crashes the checker

**Under run 4:** meaning unchanged. The second message's help is fixed so that its line compiles (R18):
a `weak T set by f` link needs a target that lives on its own (R11), so the help now says to make
`Node` a `class`; run 3's help gave the link to a `struct Node`, whose children live inside a list
that may move them.

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

   A link back to the parent is area D's `parent: weak Node set by children`, in a `class Node`.
4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `value` | none (default: a field holds its value) | none (default: changed through a name that may change the tree) |
   | `left`, `right` | `owned`: each child held alone in its own block | as `value` |
   | `t` | none (default, own local: the tree made on this line) | none (default, own local) |

5. **Means exactly:** `an41_tr_box.em` (`Option[Box[Tree]]`): `exit=0`, no output.
6. **Still refused** (the crash becomes these messages; E2200, R3):

   ```text
   error[E2200]: `Tree` contains itself through `left`, so a `Tree` would have no end
     = help: hold the child in its own block: `left: Option[owned Tree]`
   ```

   ```text
   error[E2200]: `Node.parent` would be a view of the `Node` that holds this one, which no struct can keep
     = help: for a link back to the parent make `Node` a class and let its children hold the link: `class Node:` with `children: Array[Node]` and `parent: weak Node set by children`
   ```

7. **Cost:** one block per node, as `Box` and Rust's `Option<Box<Tree>>` (one pointer, `None` is null).
8. **Kind:** an understandable limitation kept (Rust needs `Box` too), now written in the owner's word
   and with a message instead of a crash.

#### AN-42: `struct`, `Box` or `class` decides what compiles and what panics

**Under run 4:** meaning unchanged; `self` becomes `this` and the methods no longer list it (clash 4's
first line, `RUN4/changes.md` section 10). `reg` is now declared in the block.

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
3. **NEW:** the same text whatever the pool type is; the rules (R1, R8) are the same for a struct, an
   `owned` block and a class:

   ```ember
   struct Reg:
       pools: Map[u128, owned dyn AnyPool]

       fn get_mut[T](i: int) changes pools -> ref mut T:
           pool = ref mut cast this.pools[7u128] as Pool[T]
           return ref mut pool.dense[i]

       fn emplace[T](e: int, owned value: T) changes pools:
           ...

   reg = Reg(pools=Map())
   total = 0
   for x in reg.values[int]():
       total += x
   reg.emplace(3, 30)                    # after the loop (the owner's "delete after the loop")
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `pools` | `owned dyn AnyPool` elements: each pool held alone | `changes pools` on the methods that change it |
   | `this` | `this`: the registry the method was called on | `changes pools` on each method's first line |
   | `pool` | `ref mut`: a name for a pool in the map, the only one while used (R14: a cast of a place is a place) | `mut` in `ref mut` |
   | `i`, `e` | none (default: a number parameter, copied in) | none (default, reads only) |
   | `value` | `owned`: handed over to `emplace` | none (default, own local) |
   | `reg`, `total` | none (default, own local) | none (default, own local) |
   | `x` | none (default, reads only: a loop name for each element) | none (default, reads only) |

   The first line follows R5's order (`changes` before `->`, as row 7's `reads`), and `from` is not
   written: a method's result points into its object by default (R6).
5. **Means exactly:** `an42_old_box_pools.em` (the `Box` form, `exit=0`, no output) is what both kinds
   compile to; the loop with the change after it is the `an42_old_box_loop.em` shape minus its two
   errors (the object taken with `mut self`, the `emplace` moved after the loop), as in
   `an49_tr_changes.em` (`exit=0`).
6. **Still refused**, for every kind of pool (R6: the loop's source views all of `reg`; R8 (c)):

   ```text
   error[E3020]: `reg.emplace(3, 30)` may add to the pool this loop walks (`reg.values[int]()`, line 19)
     = note: `emplace` says `changes pools`; adding can move the pool's elements while the loop reads them
     = help: add after the loop: collect what to add in a list, then call `reg.emplace(...)` for each once the loop ends
   ```

   The `run-pass` test `ecs4/accept_change_during_loop.em` becomes a compile-fail test with this message.
7. **Cost:** nothing while running: today's class pools carry access words checked while running
   (`[EXC-19]`), which go (R9).
8. **Kind:** a smarter compile-time solution (one rule set for every kind; the kind changes the layout,
   never what compiles or what panics).

#### AN-43: `owned` on a parameter means three things by type; `mem.drop(owned x)` moves a handle

**Under run 4:** meaning unchanged. Two messages are fixed so that the line they give compiles in this
program (R18): `take(copied a)` needs `Node`'s `Clone`, which `class Node` does not have, so the help
now says to add it; and `fn take(n: owned Node)` on a class now has its code, E2114.
The code for `take(shared a)` is today's type mismatch, E2020 (probe below).

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
   x = take(a)                              # `a` ends here; a hand-over has no word at the call (R4)
   b = 1
   y = add_one(b)
   mem.drop(x)                              # ends `x` now, for every type
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `n` (in `take`) | `owned`: handed over to `take` | none (default, own local) |
   | `v` | none (default: a field holds its number) | none (default: through a name that may change the node) |
   | `n` (in `add_one`) | none (default: a number parameter, copied in, R4) | none (default: never changes the caller's) |
   | `a`, `b` | none (default, own local: the value made on this line) | none (default, own local) |
   | `x`, `y` | none (default, own local: the number the call returns) | none (default, own local) |

5. **Means exactly:** `an43_tr_handover.em` (`take(a)` with `a` not used again, `take_int(n: int)`):
   `exit=0`, no output. Today's `owned n: Node` gives the callee a second owner; the hand-over is
   exactly today's code when the caller does not use `a` again, which the NEW refusal guarantees.
6. **Still refused:**

   ```text
   error[E3040]: `a` was handed over to `take` on line 11 and `a.v` reads it on line 12
     = help: read it before the call (`v = a.v` above line 11), or give `take` a copy of its own: add `@derive(Clone)` above `class Node` and write `take(copied a)`
   ```

   ```text
   error[E2085]: `owned n: int`: an `int` is copied, not handed over
     = help: remove `owned`: write `fn take(n: int) -> int:`
   ```

   ```text
   error[E3040]: `a` ended at `mem.drop(a)` on line 6 and is used on line 7
     = help: use `a` before line 6, or remove the `mem.drop`
   ```

   (for every type, an `int` included: 06's `k3_mem_drop_int.em` is accepted today).
   `fn take(n: owned Node)` (E2114, R3): "a `Node` object always lives in its own block, so `owned` in
   its type says nothing; to take it over write `owned n: Node`; to read it write `n: Node`".
   `take(shared a)` into `owned n: Node` (E2020): "`take` takes a `Node` held alone, and `shared a` is a
   second owner; to let `take` be one more owner declare `owned n: shared Node`". Today's E2020 is a
   type mismatch (`target/redesign4/04-area-C/probes/c1_type_mismatch.em`, a `Node` given to `n: int`):

   ```text
   === c1_type_mismatch.em
   error[E2020]: expected `i64`, found `Node`
    --> c1_type_mismatch.em:9:18
     |
   4 | fn take(n: int) -> int:
     |         ------ parameter `n` is declared `i64` here
   ...
   9 |     println(take(a))
     |                  ^ this is `Node`
     |

   error: could not compile due to 1 error(s)
   exit=1
   ```

7. **Cost:** nothing: a hand-over passes the pointer or the value as today, and no count is made for a
   class (today's `[FN-9]` retains the caller's handle for the call).
8. **Kind:** a readable spelling of Rust's guarantee (`owned` is Rust's move, for every type).

#### AN-44: `b = a` has no word and does three things by type

**Under run 4:** meaning unchanged. `pos = ref mut p.pos` is now the only name for `p.pos` while it is
used (R8 (f)); the NEW reads `p.pos` again only after `pos`'s last use, so it compiles as before.

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
   mine = copied p.pos        # your own copy to change (after pos's last use)
   mine.y = 0.0
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `n`, `n2` | none (default: a number; `=` copies it, R1) | none (default, own local) |
   | `items`, `player`, `p` | none (default, own local: the value made on this line) | none (default, own local) |
   | `snapshot` | none (default, row 9: `=` hands the list over) | none (default, own local) |
   | `p2` | `shared`: one of two owners of the player | none (default: any owner may change it, R9) |
   | `pos` | `ref mut`: a name for `p`'s position, the only one while used | `mut` in `ref mut` |
   | `mine` | `copied`: its own new position | none (default, own local) |
   | `hp`, `x`, `y` | none (default: a field holds its number) | through the name before them |

5. **Means exactly:** `an44_tr_three.em` (today's class `p2 = player` is exactly a second owner):
   `exit=0`, no output; `an44_tr_struct_refmut.em` (`pos = ref mut p.pos; pos.x += 1.0`, prints `2.0`):
   `exit=0`, no output; `copied p.pos` is today's `p.pos` copy of a `Copy` struct.
6. **Still refused:**

   ```text
   error[E3040]: `player` was handed over to `p2` on line 8 and is used on line 10
     = note: `p2 = player` hands the object over, as for every value that is not a number
     = help: for one player with two names write `p2 = shared player`; for a second player write `p2 = copied player`
   ```

   ```text
   error[E3023]: `pos.x += 1.0` changes `p.pos` through `pos`, which only reads it
     = note: `pos = p.pos` names `p`'s position, a part of `p`, read-only
     = help: to change `p`'s position write `pos = ref mut p.pos`; to change your own copy write `pos = copied p.pos`
   ```

   And reading `p.pos` while `pos` is still used (R8 (f)): `mine = copied p.pos` moved above
   `pos.x += 1.0` gives "`pos` is the only name for `p.pos` until line 13, and line 12 reads `p.pos`;
   read it through `pos` (`mine = copied pos`), or after line 13" (E3021).
7. **Cost:** nothing; `shared player` is a count +1 (and -1 when `p2` ends), as Rust's `Rc::clone`.
8. **Kind:** a readable spelling of Rust's guarantee (Rust's move, `Copy`, `Rc::clone`), with the
   struct copy that Rust also makes silently now named.

#### AN-45: `.clone()` on a class handle shares the object instead of copying it

**Under run 4:** meaning unchanged.

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

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `health` | none (default: a field holds its number) | through the name before it |
   | `a` | none (default, own local: the object made on this line; after `shared a` one of two owners) | none (default, own local) |
   | `b` | `copied`: its own new object | none (default, own local) |
   | `c` | `shared`: a second owner of `a`'s object | none (default: any owner may change it, R9) |

5. **Means exactly:** `an45_tr_copied.em` (`b = Player(a.health)`, what `Player`'s `Clone` runs):
   `exit=0`, no output.
6. **Still refused:**

   ```text
   error[E1010]: `a.clone()`: a copy is written `copied a`
     = help: for a new player write `b = copied a`; for a second owner of this one write `b = shared a`
   ```

   `copied a` of a class with no `Clone`: "`Player` does not say how to copy it; add `@derive(Clone)`
   above `class Player`, or write `b = shared a` for a second owner".
7. **Cost:** the copy where `copied` is written (one block and the fields), as C's `malloc` + copy and
   Rust's `.clone()` of a `Box`.
8. **Kind:** a readable spelling of Rust's guarantee.

#### AN-46: silent copies no word names: `comptime(e)` of a heap value; `sorted(m.keys())`

**Under run 4:** meaning unchanged. The block now declares `total` and `m`.

1. **Intent:** use a constant table; print a map's keys in order.
2. **OLD:** `an46_old_comptime.em` (`t = comptime(table())` in a loop, `table()` returns `Array[int]`):
   `exit=0`, no output; by spec 4038-4040 each run-time evaluation clones the table into a new
   allocation. `an46_old_sorted_keys.em` (`for k in sorted(m.keys()): println(k)`): `exit=0`, no output;
   by spec 4155 the array holds clones of the keys.
3. **NEW:**

   ```ember
   total = 0
   for i in 0..3:
       t = comptime(table())          # a view of a table made when compiling: no copy
       total += t[i]
   own = copied comptime(table())      # your own list, which may change
   own.push(4)

   m: Map[String, int] = Map()
   for k in sorted(m.keys()):          # views of m's own keys, sorted: no copy
       println(k)
   names = sorted(m.keys().copied())   # your own strings, kept past changes to m
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `total`, `m` | none (default, own local) | none (default, own local) |
   | `i` | none (default, reads only: a number from the range) | none (default, reads only) |
   | `t` | none (default, reads only: a value made when compiling is read-only data; the name is a view of it, never a copy) | none (default: it never changes) |
   | `own` | `copied`: its own list | none (default, own local) |
   | `k` | none (default, reads only: a loop name; here a view of one of `m`'s keys) | none (default, reads only) |
   | `names` | `.copied()`: its own strings | none (default, own local) |

5. **Means exactly:** `an46_tr_static.em` (`static TABLE: Array[int] = comptime(table())`,
   `t = ref TABLE`: no copy, as spec 4040 says): `exit=0`, no output. `an46_tr_copied.em`
   (`t = TABLE.clone()`, `t.push(4)`): `exit=0`, no output. `an46_tr_keys_copied.em` (today's
   `sorted(m.keys())`, which clones, kept past `m.insert`): `exit=0`, no output. The view form cannot be
   said in today's Ember (an `Array` cannot hold views, `[TYP-15]`); in C, run 3's
   `target/redesign3/10-area-C/bench/an46_sorted_views.c` (an array of pointers to the map's own keys,
   `qsort`, printed): built with clang `-O2`, run once: `a`, `b`,
   `an46-sorted-views ms=0.017 peak_kb=4060 checksum=3105` (`RUN3/combined-v4.md` line 6801).
6. **Still refused:**

   ```text
   error[E3023]: `t.push(4)` changes `t`, a view of a table made when compiling, which never changes
     = help: for your own list write `t = copied comptime(table())`
   ```

   The sorted keys kept as a list and used after a change to `m`:
   `ks = sorted(m.keys())`, then `m.insert(String("z"), 1)`, then `println(ks[0])`:

   ```text
   error[E3060]: `m.insert(...)` changes `m`, whose keys `ks` (line 7) still views on line 9
     = help: use `ks` before this line, or keep your own strings: `ks = sorted(m.keys().copied())`
   ```

7. **Cost:** less than today: no allocation per evaluation of the table (read-only data, as C's
   `static const`), and no string copy per key (an array of pointers, as the C above and Rust's
   `let mut ks: Vec<&String> = m.keys().collect(); ks.sort();`). Copies only where `copied` is written.
8. **Kind:** a smarter compile-time solution (the value is placed in read-only data and lent; the
   sorted keys are a local list of views, R7).

#### AN-47: `owned` in an expression: E0109 contradicts Annex A, and its help is wrong for `b = owned a`

**Under run 4:** meaning unchanged.

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
   report = Report()
   job = fn() with owned report: send(report)   # takes `report` over
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `a`, `report` (the local) | none (default, own local: the value made on this line) | none (default, own local) |
   | `b` | none (default, row 9: `=` hands over) | none (default, own local) |
   | `task`, `job` | none (default, own local: the function made on this line; `task` captures nothing) | none (default, own local) |
   | `report` in the capture list | `owned`: handed over to the function (R3, R12) | none (default: the function's own) |

5. **Means exactly:** `an47_tr.em` (`b = a`; `task = fn() => println("done")`): `exit=0`, no output.
6. **Still refused** (E0109, R3; Annex A's `task = owned fn() => ...` is rewritten):

   ```text
   error[E0109]: `owned a` is not needed: `b = a` already hands `a` over
     = help: remove `owned`: write `b = a`; for a copy write `b = copied a`; for a second owner write `b = shared a`
   ```

   ```text
   error[E0109]: `owned fn` does not say what the function takes or copies
     = help: name it on each captured name; `task` captures nothing, so write `task = fn() => println("done")`; a name not listed is the real variable
   ```

7. **Cost:** nothing.
8. **Kind:** a readable spelling of Rust's guarantee (a message that gives the line).

#### AN-48: shared mutable state: a camera and a HUD both change one player

**Under run 4: meaning changed.** `ref mut` now has one meaning everywhere, "only this name may touch
it until it is done" (owner, 2026-10-25), kept fields included (R7, R8 (f)). Run 3's rules let two kept
`ref mut` fields name one player (run 3's R9 sentence, AN-52 (b)), so the OLD struct form
(`target: ref mut Player` in both) was accepted by run 3's rules; it is now refused, with a message
that gives the owner's new word. The program's intent is kept by a new third form (b), with
`target: sharedref mut Player` in both: the camera and the HUD follow a player they do not own, with no
count. The `shared` form (a) is unchanged in meaning; `self` becomes `this`, and `transfer` is now
called with `ref mut` (R4).

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
   # (a) the camera and the HUD each keep the player alive: shared owners, counted
   class Player:
       hp: int = 100
       inventory: Array[String] = []
       log: Array[String] = []

   class Camera:
       target: shared Player

       fn shake() changes target:
           this.target.hp -= 1

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
       transfer(cam.target, ref mut hud.target)   # the call says `ref mut`, as the parameter does

   fn transfer(a: shared Player, ref mut b: shared Player):
       if a is not b:
           for it in a.inventory:
               b.inventory.push(copied it)        # accepted: the compiler knows they differ
   ```

   ```ember
   # (b) the camera and the HUD follow a player `main` owns, and both may change it: no count
   struct Player:
       hp: int
       inventory: Array[String]

   struct Camera:
       target: sharedref mut Player

       fn shake() changes target:
           this.target.hp -= 1

   struct Hud:
       target: sharedref mut Player

   fn main():
       p = Player(hp=100, inventory=[])
       cam = Camera(target=sharedref mut p)
       hud = Hud(target=sharedref mut p)
       cam.shake()
       hud.target.inventory.push(String("key"))
       for item in hud.target.inventory:
           cam.target.hp -= item.len()            # a number: accepted
       p.hp += 10                                 # the owner may change it too (R7)
   ```

   In (b), `Camera` and `Hud` hold a view, so they are views themselves (R7): `cam` and `hud` may not
   outlive `p` (E3063). A list in the player may not change while a `for` loop walks it, whichever
   holder reaches it (R7, R8 (c)), as in (a).
4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `hp`, `inventory`, `log` | none (default: a field holds its value in the player) | through the name before them |
   | `target` (form (a), both classes) | `shared`: one of several owners of a player | `changes target` on `shake`; through `cam`, `hud` (the function's own) in `main` |
   | `target` (form (b), both structs) | `sharedref`: a link to a player someone else owns, which other holders may also hold | `mut` in `sharedref mut` (R7) |
   | `this` (in `shake`) | `this`: the camera the method was called on | `changes target` on the first line |
   | `p` | none (default, own local: the player made on this line; in (a), after `shared p`, one of its owners) | none (default, own local; in (b) the owner, which may change it beside the links, R7) |
   | `cam`, `hud` | none (default, own local: the object made on this line; in (b) a view of `p` by its field's word, R7) | none (default, own local) |
   | `item`, `it` | none (default, reads only: a loop name for each element) | none (default, reads only) |
   | `a` | `shared` in the parameter type: a view of the caller's `shared` player (R4) | none (default, reads only) |
   | `b` | `ref mut` on the name, `shared` in the type: a name for the caller's `shared` player and its link | `ref mut`: yes; by R4 it names the caller's place, so it covers both the player (`b.inventory.push(...)`) and the caller's link (`b = shared q` would point `hud.target` at `q`) |

5. **Means exactly:** `an48_tr_class.em` (today's class form of the accepted lines of (a), with the
   refused loop rewritten as its message says: collect, then push after the loop): `exit=0`, no output.
   `an48_tr_is_not.em` (`transfer` with `if a is not b:`): `exit=0`, no output. Today both carry
   run-time access words on `inventory` and `log` (`[EXC-19]`); the NEW model has none (R9). Form (b)
   cannot be said in today's Ember (the OLD struct form is today's E3022); in C it is a plain pointer in
   each struct and no count, the shape of run 3's `bench/share_c.c` `-DMODEL=8`
   (`RUN3/combined-v4.md` line 7278).
6. **Still refused:**

   ```text
   error[E3020]: `cam.target.inventory.push(...)` may add to the list this loop walks (`hud.target.inventory`, line 23)
     = note: `cam.target` and `hud.target` may be the same `Player`, and adding can move the list's elements while the loop reads them
     = help: add after the loop: collect the new items in a list, then `cam.target.inventory.extend(new_items)` once the loop ends; or loop over `copied hud.target.inventory`
   ```

   The same for a call: a method whose first line says `changes target` (it may reach the player's
   inventory) called inside that loop: "`cam.shake()` says `changes target`, which may be the `Player`
   whose inventory this loop walks; call it after the loop" (E3020). `transfer` without the `is not`
   test: "`a` and `b` may be the same player, and this loop walks `a.inventory` while adding to
   `b.inventory`; test first: `if a is not b:`" (E3020). `transfer(cam.target, hud.target)` without
   the word: "`transfer` may change `b` (its first line says `ref mut b: shared Player`); say so at the
   call: `transfer(cam.target, ref mut hud.target)`" (E3028). A `shared` value sent to another thread:
   area E's E7015.

   The OLD struct form, two kept `ref mut` fields of one player (R8 (f); the E3022 row of
   `RUN4/rules.md` section 3, in this program's lines):

   ```text
   error[E3022]: `p` is lent to `cam.target`, a `ref mut` field, on line 15 until its last use on line 17, and line 16 lends it to `hud.target` too
     = note: a `ref mut` name is the only name that may touch its place until it is done
     = help: if both may change the player, write `target: sharedref mut Player` in `Camera` and in `Hud`, and `Camera(target=sharedref mut p)`, `Hud(target=sharedref mut p)`
   ```

   In (b), a function that makes `p` and returns `cam`: "`cam` holds a link to `p`, which ends when
   this function returns, so `cam` may not outlive it (returned here); let the camera own a share:
   `target: shared Player`" (E3063).
7. **Cost:** nothing checked while running. (a): counts +1/-1 per `shared` owner made or ended; for the
   views the loops make (`hud.target.inventory`), no count (R10 point 1: nothing in the loop may end a
   `Player` owner). (b): nothing at all, a pointer in each struct; in run 3's watcher workload this
   shape ran at 1.00x to 1.06x of plain C with the same memory (`RUN3/combined-v4.md` lines 5756-5763,
   `MODEL=8`, PROVISIONAL). Today's class pays 8 bytes of access words per non-`Copy` field and checks
   where not proved away (1.75x to 1.87x of plain C in run 2's workload, `phase3b-ownership.md`
   section 4); Rust pays `Rc<RefCell<Player>>`'s flag on each borrow (1.48x to 1.57x,
   `RUN3/combined-v4.md` lines 5756-5763).
8. **Kind:** (a) a smarter compile-time solution: Rust's guarantee (no view outlives a reshape of what
   it points into) checked from first lines instead of a run-time flag, with the owner's "a list may
   not change while a loop walks it" kept as the limitation. (b) a new mechanism (`sharedref mut`:
   several holders may change one object, checked by R8 when compiling; Rust allows only one `&mut`).

#### AN-49: `RefCell` hides a run-time check that panics; E3023 offers it to a beginner who forgot `mut`

**Under run 4:** meaning unchanged; `self` becomes `this`, and the methods no longer list it. The
refused conflict is written without `ref` (R2: a name for a part is a read-only view already).

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

       fn add(e: int) changes entities:
           this.entities.push(e)

   struct Counter:
       n: int

       fn bump() changes n:
           this.n += 1
   ```

   A scene several objects own is `scene: shared Scene`, changed through any owner (AN-48).
4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `entities`, `n` | none (default: a field holds its value) | `changes entities`, `changes n` on the methods |
   | `this` (both methods) | `this`: the scene, the counter the method was called on | `changes entities` / `changes n` on the first line |
   | `e` | none (default: a number parameter, copied in) | none (default, reads only) |

5. **Means exactly:** `an49_tr_changes.em` (`fn add(mut self, e: int)`, today's spelling of
   `changes entities`): `exit=0`, no output.
6. **Still refused** (`RefCell`, `Ref`, `RefMut`, `borrow`, `borrow_mut`, `RuntimeChecked` removed, R9):

   ```text
   error[E1010]: `RefCell` is not part of Ember
     = note: a change is said on the method that makes it, and checked when compiling
     = help: write the field plainly, `entities: Array[int]`, and the method's first line as `fn add(e: int) changes entities:`; for a scene several objects own write `shared Scene`
   ```

   ```text
   error[E2105]: `bump` changes `this.n`, but its first line does not say so
     = help: write `fn bump() changes n:`
   ```

   The conflict `an49_old_refcell_conflict.em` panics on becomes a compile-time refusal (R8 (b)): a
   view of `s.entities` (`a = s.entities`) still used while `s.add(...)` changes it: "`s.add(4)`
   (line 7) changes `s.entities` (`add` says `changes entities`), which `a` views (line 6) and uses on
   line 8; use `a` before line 7, or keep a copy: `a = copied s.entities`" (E3060).
7. **Cost:** less than today: no borrow flag (one word per `RefCell`, checked on every borrow in every
   profile, `[CELL-5]`, `[CELL-9]`); nothing runs.
8. **Kind:** a smarter compile-time solution (Rust's run-time-checked `RefCell` replaced by R8's
   compile-time rule).

#### AN-50: "handle" names a class object's counted pointer and also `Handle[T]`

**Under run 4:** meaning unchanged; `self` becomes `this` in the E2103 message (the E2103 row of
`RUN4/rules.md` section 3).

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
3. **NEW:** no new code; the word "handle" is used only for `Handle[T]`, a position in a `Pool`, and
   the written words `JoinHandle` and `@ffi(handle)` (the `Handle` row of `RUN4/vocabulary.md`). A class
   object is "the `Player` object `p`", a link in a field "the link `o.bag`". The program the first
   message is about, as the new message makes it:

   ```ember
   first = copied o.bag.items[0]       # or use `first` before replacing the bag
   o.bag = Bag()
   println(first)
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `first` | `copied`: its own number (for an `int`, plain `=` copies too, R2) | none (default, own local) |
   | `o` | none (default, own local: the object made on its line) | none (default, own local) |
   | `bag` | none (default: a field holds its `Bag` object alone, R3) | through the name before it |
   | `items` | none (default: a field holds its list) | through the name before it |

5. **Means exactly:** `an50_tr_use_before.em` (`first = o.bag.items[0]`, an `int` copied, then the bag
   replaced): `exit=0`, no output.
6. **Still refused**, with the new wording:

   ```text
   error[E3060]: `o.bag = Bag()` (line 11) replaces the `Bag` object that `first` (line 10) points into, and `first` is used on line 12
     = help: use `first` before line 11, or keep your own copy: `first = copied o.bag.items[0]`
   ```

   ```text
   error[E2103]: `this = Node()` cannot replace the `Node` object `reset` was called on
     = help: set its fields instead: `this.v = 0`, with `fn reset() changes v:`
   ```

   ```text
   panic: `pool[h]`: `h` refers to a value already removed from `pool`
     help: use `pool.get(h)`, which gives `None` for a removed value
   ```

7. **Cost:** nothing.
8. **Kind:** a readable spelling (messages in the program's own names).

#### AN-51: a count pair the spec allows to remove is emitted every loop step

**Under run 4:** meaning unchanged. A call to `step` now says `ref mut ws` (R4). What keeps `t`'s
player alive when a line could end it is R10's list, which now includes the deferred-drop region.

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

   fn step(ref mut ws: Array[Watcher], frame: i32) -> u64:    # called as `step(ref mut ws, frame)`
       acc: u64 = 0
       for i in range(ws.len()):
           t = ref mut ws[i].target      # a name for the watcher's link to its player: no count
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

   `t = ref mut ws[i].target` changes the player reached through `ws`, so `ws` says `ref mut` (R5: a
   plain name never changes anything, an object behind a `shared` link included). The list and its
   links are not changed, and the C is the same.
4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `hp`, `x`, `inventory` | none (default: a field holds its value) | through the name before them |
   | `target` | `shared`: one of several owners of a player | through `t` |
   | `ws` | `ref mut`: the caller's list of watchers | `ref mut`: yes (the players its watchers link to are changed through it) |
   | `frame` | none (default: a number parameter, copied in) | none (default, reads only) |
   | `acc` | none (default, own local: a number) | none (default, own local) |
   | `i`, `v` | none (default, reads only: a loop name for each value) | none (default, reads only) |
   | `t` | `ref mut`: a name for the link in `ws[i].target`, the only one while used (R8 (f)), through which the player is changed | `mut` in `ref mut` |

5. **Means exactly:** in C, run 3's `bench/share_c.c` `-DMODEL=3` (`Player *t = ws[i].target;` with no
   retain or release around the step; counts change only at re-targeting), whose checksum equals plain
   C's (`80363660820`, `RUN3/combined-v4.md` line 5754).
6. **Still refused / kept:** nothing new is refused. R10 keeps `t`'s player alive the cheapest way that
   is safe. Here, point 1: nothing in `t`'s life may end an owner of a `Player`, so no count. Where a
   line in `t`'s life could end one (a write to any place of type `shared Player`, such as
   `ws[j].target = shared q`; the end of a local `shared Player` owner; a call whose first line or
   summary may do so), it is point 2, a deferred-drop region, when `t`'s life contains a loop that makes
   an unknown call and `Player` has no `drop` of its own, else point 3, one count pair around `t`'s
   life. And while `t` is used, `ws[i].target` is not read or changed through another name (R8 (f),
   E3021 / E3022): a line `println(ws[i].target.hp)` placed after `t.hp -= 1` gets "`t` is the only
   name for `ws[i].target` until its last use, and this line reads `ws[i].target.hp`; read it through
   `t`: `println(t.hp)`" (E3021).
7. **Cost:** 0 count operations per step where today makes 2 (a load-add-store each); the 13.5% to
   13.9% run 2 measured goes (`phase3b-ownership.md` section 4).
8. **Kind:** a smarter compile-time solution (`[RC-3]`'s removal, decided from first lines, `[RC-2b]`'s
   one-expression limit lifted).

#### AN-52: shared class objects re-pointed often ran at 1.16x to 1.17x plain C (last run's limit)

**Under run 4: meaning changed in its spelling.** Form (b)'s watchers, several of which may follow and
change one player without owning it, were written `target: ref mut Player` under run 3's rule that kept
`ref mut` views may name one object. `ref mut` is now alone everywhere (owner, 2026-10-25; R8 (f)), so
(b) is written with the owner's new word, `target: sharedref mut Player`, and the run-3 spelling is
refused with a message that gives it. What (b) does, what it refuses and its C are unchanged
(`RUN4/changes.md` section 2.3). (a) and (c) are unchanged.

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
3. **NEW: the line says what the watcher means, and each meaning has its cost.** Run 2's class was tried
   first (run 3's section 5): the cost is the count touch on the new target, a random object; a count
   kept beside the objects did not help (1.16x), and no count-based form can skip that touch. So the
   answer is to stop counting where the program does not mean "keeps alive":

   ```ember
   # (a) the watcher keeps its player alive: counted, as Rust's Rc
   struct Watcher:
       target: shared Player
   ws[i].target = shared players[r]

   # (b) the watcher follows a player that `players` owns and that outlives the watchers;
   #     several watchers may follow, and change, one player: a kept link, no count
   @view
   struct Watcher:
       target: sharedref mut Player
   players: Array[owned Player] = ...             # `struct Player`, each in its own block (R3;
                                                  # a `class Player` list is plain `Array[Player]`)
   ws: Array[Watcher] = ...                       # local: lives while `players` does
   ws[i] = Watcher(target=sharedref mut players[r])   # re-point: one store

   # (c) a position in the owning list
   struct Watcher:
       target: int
   players[ws[i].target].hp -= 1
   ```

   (b)'s rules, all checked when compiling (R7, R8): `ws` may not outlive `players` (a list of views is
   a view, E3063); while `ws` is used, `players` may grow (its elements are `owned`, each in its own
   block, so growing moves only the list of pointers) but nothing may be removed from it or replace it
   (E3060); several watchers, and `players` itself, may change one player at any time (plain fields
   change freely; a view into a player's `inventory` ends at a reshape of that list through any of
   them, R8 (b)). `ref mut` parameters and names stay alone, so the C compiler is promised that two
   `ref mut` parameters do not overlap (R4); `sharedref mut` links get no such promise.
4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `target` (a) | `shared`: one of several owners | none (default: any owner may change it, R9; through the watcher's owner) |
   | `target` (b) | `sharedref`: a link to a player `players` owns, which other watchers may also hold | `mut` in `sharedref mut` |
   | `target` (c) | none (default: a number, the player's position) | none (default: changed through the watcher's owner) |
   | `players` | `owned` elements: each player held alone by the list | none (default, own local) |
   | `ws` | none (default, own local: the list made on this line; in (b) a list of views, kept until its last use, R7) | none (default, own local) |
   | `i`, `r` | none (default: numbers) | none (default, reads only) |
   | `hp` | none (default: a field holds its number) | through the name before it |

5. **Means exactly:** in C, run 3's `bench/share_c.c`: (a) `-DMODEL=3`; (b) `-DMODEL=8` (32-byte players,
   no header, no count, the bounds check on `players[r]`); (c) `-DMODEL=9` (players inline, a `u32`
   position, a bounds check on every read). Every build printed checksum `80363660820`, plain C's
   (`RUN3/combined-v4.md` line 5754). (c) in today's Ember: `an52_tr_positions.em`: `exit=0`, no output.
6. **Still refused** for (b):

   ```text
   error[E3060]: `players.remove(r)` (line 29) would end a player that `ws` (line 14) still points at on line 30
     = help: remove players after the last use of `ws`, or let the watchers keep them alive: `target: shared Player`
   ```

   ```text
   error[E3063]: `ws` holds views of `players`, so it may not outlive it (returned here)
     = help: return positions instead (`target: int`), or let the watchers own a share: `target: shared Player`
   ```

   Run 3's spelling of (b), `target: ref mut Player` with `Watcher(target=ref mut players[r])`
   (R8 (f)):

   ```text
   error[E3022]: `ws[i].target` is a `ref mut` field, the only name for `players[r]` while `ws` is used, and other watchers in `ws` may name the same player
     = help: if several watchers may change one player, write `target: sharedref mut Player` in `Watcher` and `Watcher(target=sharedref mut players[r])`
   ```

   `sharedref` without `mut` (E0114, R7): "a link that only reads may be held by any number of names:
   write `target: ref Player`; `sharedref` is for a link that may change its target:
   `target: sharedref mut Player`".
7. **Cost** (run 3's section 5, three batches, PROVISIONAL; `RUN3/combined-v4.md` lines 5756-5763):
   (a) 1.13x to 1.25x of plain C, against 1.48x to 1.57x for safe Rust's `Rc<Player>` with `Cell`;
   (b) 1.00x to 1.06x, the same memory as plain C (53,284 KB); (c) 0.80x to 0.85x and 8% less memory
   (49,188 KB), against 0.86x to 0.90x for safe Rust's `Vec` and positions. (b)'s C is unchanged by the
   new word, so its numbers stand (`RUN4/rules.md` section 2, the `sharedref mut` row).
8. **Kind:** a new mechanism (`sharedref mut`: kept links several holders may change, checked by R8's
   compile-time rule; Rust allows only one `&mut`), with (a)'s counting kept as the understandable cost
   of "keeps alive", which Rust pays more for.

## Answers whose meaning changed under the run-4 rules

| answer | how |
|---|---|
| AN-48 | two kept `ref mut` fields of one player, accepted by run 3's rules (kept `ref mut` views could name one object), are now refused (E3022, R8 (f)); the intent is kept by a new form (b) with `target: sharedref mut Player`, at no count. Form (a) is unchanged in meaning |
| AN-52 | form (b) is spelled `target: sharedref mut Player` and `Watcher(target=sharedref mut players[r])`; run 3's spelling `ref mut` now means "alone" and is refused there (E3022). What (b) does and its C and cost are unchanged |

Every other answer of the area means what it meant in run 3: the changes are `this` for `self` (AN-37,
AN-38, AN-42, AN-48, AN-49, AN-50), `ref mut` at calls (AN-37's help, AN-48, AN-51), codes and
messages brought to R18 (AN-40, AN-41, AN-43), `_` written out (AN-35, AN-37), and names now declared
in their block (AN-36, AN-42, AN-46, AN-47). AN-34's hand-over form now uses row 9's own name,
`snapshot`.

## Words needed

| word | why |
|---|---|
| `_` (a loop name or a `match` arm that names nothing) | run 3 wrote `for _ in 0..n:` (AN-35) and a `_:` arm (AN-37); `_` has no row in `RUN4/vocabulary.md`, so this file does not use it: AN-35 writes `for i in 0..n:` and AN-37 lists its arms `State.Idle:` and `State.Dead:`. Today's Ember accepts `_` in both places (`an35_tr_loop.em` and `an37_tr_refmut_inline.em` under `target/redesign3/10-area-C/`, both `exit=0` in that folder's `check_output.txt`). A row "`_`: a name for something not used; never declares a name" would let the answers use it again |

## The agreed rows

| row | where | as agreed |
|---|---|---|
| 3 | the methods' first lines (AN-37, AN-38, AN-42, AN-48, AN-49) use row 3's form, `fn ...(params) changes field:`, without `self` | yes: the form is `fn note(msg: String) changes log:` as the owner's decision of 2026-10-25 writes it; row 3 itself is not reproduced here |
| 8 | AN-37, AN-38: `State.Walking(ref mut path):`, and the two messages ("to change that list through `path`, write `State.Walking(ref mut path):`", for plain `path` and for `mut path`) | yes, word for word |
| 9 | AN-34: `snapshot = copied items` for a copy, `snapshot = items` hands the list over, the error names both ways out (`copied`, `shared`); AN-44: `snapshot = items` | yes |

## Notes for the lead (not for the owner)

1. **E2200 for a view of the type itself** (AN-41's second message). `RUN4/rules.md` gives E2200 the
   meaning "a type that contains itself without `owned`"; run 3 used the same code for
   `parent: Option[ref Node]`, which does not contain a `Node` but views one. This file keeps run 3's
   code, as the duty says ("keep run 3's number where the meaning stays"); the rule writer's E2200 row
   may want to name this case.
2. **E2200's wording "without `owned`"**: a `Node` that holds `Array[Node]` or `Array[shared Node]`
   (AN-36) contains itself only through a list's or a counted block's heap memory and is accepted; the
   row reads as if `owned` were the only way out. The meaning used here is run 3's AN-41: a type that
   contains itself directly, with no block of its own between.
3. **L3001 on every tree of `shared` children** (AN-36's second form): R11 gives the warning for "a
   cycle of `shared` owners"; whether it is given for a type that only could form one (today's
   "statically visible cycle") or only where the compiler sees one formed is not said. Today warns on
   the type (`an36_tr_shared.em` above).

## For the owner

Nothing from this area: every change here follows the owner's decisions of 2026-10-25 and the rule
writer's rules; the questions those raise are in `RUN4/changes.md` section 13.
