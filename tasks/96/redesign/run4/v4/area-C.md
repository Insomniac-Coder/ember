# Area C, v4: ownership and sharing, `owns` replacing `Box`, copies, hand-over, state machines (AN-34 to AN-52)

Agent 04 (area-C), run 4, step 3 (v4). Every answer of area C brought to the v4 rules and words, which
apply all of the owner's verdicts on run 4's open questions, 1 to 82, where 27 replaces 25 and 26
replaces 1 (`inputs/JOB.md`, "Owner verdicts on run 4's open questions"): `V4/rules.md` (rules R1 to
R18, the codes), `V4/vocabulary.md` (the words) and `V4/changes.md` (what changed from v3, verdicts 53
to 82), with the lead's `V4/inputs/worklist.md`. `RUN3` means `tasks/96/redesign/run3/`, `RUN4` means
`tasks/96/redesign/run4/`, `V2` means `tasks/96/redesign/run4/v2/`, `V3` means
`tasks/96/redesign/run4/v3/`, `V4` means `tasks/96/redesign/run4/v4/`; "R1" .. "R18" are the v4 rules;
"row N" is the owner's agreed row N and "verdict N" the owner's verdict N in `inputs/JOB.md`; run 3's
area rules are written "run 3's C6". The probe files of runs 3 and 4 that the answers cite (the `an..`
files, `target/redesign3/...`, `target/redesign4/...`) are not re-run; their pasted outputs carry over
unchanged.

No compiler change, no git command, no Ember program compiled or run in this step. Every OLD part, with
its `ember check` output, and every fenced block of the "means exactly" parts is carried over unchanged
from `V3/area-C.md`; `target/redesign6/04/check_carried.py` checks it against the copy of the v3 file
taken at the start (`target/redesign6/04/area-C_start.md`), and its output is in
`target/redesign6/04/check_carried.txt`.

Each answer has the eight parts: 1 intent; 2 OLD with today's output; 3 NEW; 4 the owner's test for
every declared name; 5 means exactly (today's Ember or C doing what NEW means); 6 still refused, each
with its code and message; 7 cost; 8 kind. In the owner's-test tables "none (default: ...)" means no
word in the line answers that question and the stated default does (the brief allows that only where the
default never changes anything, never aliases silently and never copies silently). What changed from v3
is said once per answer, in its "v4" line ("**Meaning changed:**" yes or no, with the verdict), and in
the table "Answers changed in v4"; what changed before is in each answer's "v3" line in `V3/area-C.md`
and "v2" line in `V2/area-C.md`. Every message in part 6 follows R18 and verdict 73: it names what was
refused in the program's own names, says why in one line, gives the whole line(s) to write, and the
lines it gives compile in that program.

## Terms

The terms of `V4/rules.md` section 0 (owner, handed over, copy, `Copy` type, place, part, view,
read-only view, kept view, into, link, parent link, position, reshape, ended, `this`, first line, change
list, `shares` value, count, weak count, count pair, deferred-drop region, function value, kept callback,
unknown call, kind test, summary, memory manager, noalias, the owner's test). Two more, used
in the owner's-test tables below:

- **own local** (the default for a local name given a new value): the value made on the line belongs
  to the name (R1), so changing it changes nothing anyone else holds. It never aliases (a second name
  for the value needs `shares`, `shares mut`, `sharedref mut` or a view word) and never copies
  silently. Python has no word for this either.
- **reads only** (the default for a plain parameter, a plain loop, pattern or opened name, and a name
  bound to a part): a read-only view; it never changes anything, never aliases a changeable name, never
  copies (a number is copied in, which no change can tell apart; R2, R4, R5), and promises that what it
  reads does not change while it is used (R8 (g), verdict 5).

## Run 3's area rules, and the v4 rules that now say them

Area C's own rules C1 to C10 are gone as separate rules; every answer below cites the v4 rule instead.

| run 3 | v4 | what is the same, and what changed |
|---|---|---|
| C1, one owner | R1, R2 | the same: `=` with a whole name hands it over; a name for a part is a read-only view, which now also promises that what it reads does not change while it is used (R8 (g)); a field of a struct value may be handed over on its own, as in Rust (verdict 78) |
| C2, `Copy` values | R1, R2 | the same: `=` copies a `Copy` value; a `Copy` struct copied with `=` is not then changed through the new name (E3023) |
| C3, words before a value | R3 | the same, spelled `copy of`, `shares`, `shares mut`, `weakref`, `sharedref mut x`; an owner made from another may do no more than it (E3023); `copy of` works on every `Copy` type (verdict 68) and copies the links a struct holds as links (verdict 79) |
| C4, words in a type | R3 | the same, spelled `owns T`, `shares T`, `shares mut T`, `weakref T`; `owns` on a class type is E2114; `using A` after the type names the kind of its memory manager, and the line that makes the value names the instance (verdicts 19, 24); the line that makes a value for an `owns` field says `owns` (verdict 74, E2123) |
| C5, parameters | R4 | the same modes, spelled `owns x: T`; a call writes `ref mut` where its parameter does; the word on a parameter's name sets what the function may do with it, for every type (verdict 32) |
| C6, changing a shared value | R5, R8 (b), R9 | the same compile-time rule; only a `shares mut` owner (and the first owner, which keeps what it could do: verdict 23) changes the value, a `shares` owner reads it (verdict 4); `ref mut` is alone everywhere (R8 (f)); a changeable link several holders share is `sharedref mut` (R7); a function that changes an object through a link says `changes T` on its first line, whatever holds the link (verdicts 63, 64) |
| C7, keeping an object alive | R10 | checked when compiling only: no line or call in the life of a view reached through a link may end an object of its type, and after one the view is not used (E3060, verdict 82); the deferred-drop region and the count pair are no longer used for it; only a `thread_shared` object reached through a `weakref` holds an atomic count for the view's life |
| C8, `class` | R9, the `class` row of `V4/vocabulary.md` | the same 8-byte header; one owner unless a line says `shares` or `shares mut`; a count is compared with its maximum before each +1 (verdict 59) |
| C9, no run-time borrow checks | R9 | the same |
| C10, one thread | R9, R15 | the same |

Verdicts 1 to 34 were applied in v2 and v3 (each answer's v2 and v3 lines in `V2/area-C.md` and
`V3/area-C.md` say where). What verdicts 53 to 82 change in this area:

- **63, 64: a change through a link is named `changes T`, whatever holds the link** (R5). Every NEW
  function that changes an object through a `shares mut` or `sharedref mut` link, held in a parameter,
  in its own local or in a field of its own local, now says so: AN-34's `two_owners` and
  `owner_that_reads` (`changes Array[int]`), AN-36's second form (`changes Node`), AN-39's `build`
  (`changes Mesh`), AN-44 (`changes Player`), AN-48's `main` in both forms and `transfer` (`changes
  Player`), AN-51's `step` (`changes Player`). The messages whose way out makes such a change (`write
  other = shares mut items`, `write p2 = shares mut player`, ...) now give that first line too
  (verdict 73), where the program's first line does not already say it.
- **55: a function that may end a class object of its own state names its type**: AN-47's `job`, which
  takes `report` over and gives it to `submit`, which ends it, is now `fn() with owns report changes
  Report: submit(report)`.
- **74: `owns` on the line that makes a value for an `owns` field**: AN-40's `Entity(mesh=owns
  Mesh(verts=[0.0]))` (E2123 without it), and the line AN-39's table gives for `owns T`.
- **82: R10 is checked when compiling only**: AN-51's part 6, which named the deferred-drop region and
  the count pair as R10's fallbacks, now gives E3060 for a line in `t`'s life that may end a `Player`.
- **81: a push does not end a link to an object in its own block**: AN-52 (b)'s "`players` may grow" is
  now the rules' own text (R8 (b)).
- **78: a field handed over on its own**: AN-35's part 6 gains the E3040 message for a part.
- **59: counts checked before they roll over**: one compare per +1, in the cost of every answer that
  counts (AN-34, AN-36, AN-39, AN-41, AN-44, AN-45, AN-48, AN-52).
- **68: `copy of` on every `Copy` type** settles AN-44's `mine = copy of p.pos` for a `Vec2` marked only
  `@derive(Copy)`, and AN-50's `copy of` an `int`.
- **73: every message checked** so that its way out compiles in its program and names every line it
  replaces (AN-35, AN-36, AN-39, AN-40, AN-43, AN-44, AN-45, AN-46, AN-47, AN-49, AN-50, AN-52).
- **53, 54: the call checked by the callee's body** (E3031): no answer of this area hands a function a
  part of what it changes; AN-48's `transfer` is the one call where it is asked, and is allowed (AN-48).
- **72: the loop rule covers maps**: AN-42's loop walks a map's values, refused at the change (E3020),
  as it was.

No answer of this area opens an `Option` or a `weakref` link, has a `drop`, uses `mem.forget`, runs a
function it is given on a thread or hands C a callback, so verdicts 56, 57, 58, 60, 61, 62, 65, 66,
67, 69, 70, 71, 75, 76, 77, 79 and 80 change no line here.

## The answers

#### AN-34: `snapshot = items` hands the list over; using `items` after is E3040, whose help says "clone"

**v4:** **Meaning changed:** no; two first lines change (verdicts 63, 64). `two_owners` changes the list
through `other`, a `shares mut` owner, and `owner_that_reads` through `items` after it has shared it,
when it no longer owns the list alone; both are changes through a link (R5; section 0: a link names an
object it does not own alone, `shares` and `shares mut` owners included), so both first lines say
`changes Array[int]`. `main` and `hand_over` change only a list that is the name's own, and say
nothing. Row 9's E3040 message gives, for its `shares` way out, the first line that way out then needs
(verdict 73); E3023's names its line; part 6 gains E2105 for `two_owners` without the word. The counts
are compared with their maximum
before each +1 (verdict 59). Row 9 is as agreed, in its spellings (`copy of`, `shares`; verdicts 12,
19).

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
       snapshot = copy of items      # a copy: the snapshot stays [1, 2, 3]
       items.push(4)
       println(snapshot.len())       # 3

   # the other ways the line can be written, each meaning what its word says,
   # each a program of its own (they are alternatives, not one sequence):
   fn hand_over():
       items = [1, 2, 3]
       snapshot = items              # hands the list over: `items` ends here
       snapshot.push(4)

   fn two_owners() changes Array[int]:    # it changes a list through a link (`other`)
       items = [1, 2, 3]
       other = shares mut items      # one list, two owners, both may change it: a change through either is seen by both
       other.push(4)
       println(items.len())          # 4

   fn owner_that_reads() changes Array[int]:    # `items` no longer owns the list alone when it pushes
       items = [1, 2, 3]
       other = shares items          # one list, two owners; `other` only reads it
       items.push(4)
       println(other.len())          # 4: a `shares` owner sees what the first owner changes
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `items` (all four) | none (default, own local: the list made on this line is the name's own) | none (default, own local); in `two_owners` and `owner_that_reads`, from the `shares` line on, the first of two owners, which keeps what it could do (R3, R9; verdict 23), and a change through it is named on the first line, `changes Array[int]` (R5; verdicts 63, 64) |
   | `snapshot` in `main` | `copy of`: its own new list | none (default, own local) |
   | `snapshot` in `hand_over` | none (default, row 9: `=` with a whole name hands the list over; `items` ends) | none (default, own local) |
   | `other` in `two_owners` | `shares`: one of two owners of the same list | `mut` in `shares mut`: it may change the list (R9); the change is named on the first line, `changes Array[int]` (R5) |
   | `other` in `owner_that_reads` | `shares`: one of two owners of the same list | none (default: a `shares` owner only reads, R9; the line says no `mut`) |

5. **Means exactly** (today's Ember; all three accepted, `exit=0`, no output, as run 3 pasted them):
   `an34_tr_copied.em` (`snapshot = items.clone()`), `an34_tr_handover.em` (`snapshot = items`,
   `items` not used again), `an34_tr_shared.em` (`items = Shared([1, 2, 3])`, `other = items`,
   `items.get_mut().push(4)`, `other.get().len()`; today's `Shared` checks each `get_mut` while running,
   which the NEW `shares` does not, R9). Files under `target/redesign3/10-area-C/`. No file of today's
   Ember was made for `owner_that_reads` (today's `Shared` has no read-only owner).
6. **Still refused** (R1, E3040; row 9's message, both ways out named, as `V4/rules.md` section 3 gives
   it; for the second way out the first line too, since `items.push(4)` is then a change through a link,
   R5 and verdict 73). The OLD program in v4's words, `fn main():` with lines 2 to 5 as OLD's:

   ```text
   error[E3040]: `items` was handed over to `snapshot` on line 3 and is used on line 4
     = note: `snapshot = items` hands the list over; `items` ends there
     = help: to keep both, write `snapshot = copy of items` (a copy of your own) in place of line 3, or `snapshot = shares items` (two owners of one list) in place of line 3 and `fn main() changes Array[int]:` in place of line 1
   ```

   A change through an owner that only reads (R9, E3023; the `shares` row of `V4/vocabulary.md`):
   `owner_that_reads` with `other.push(4)` in place of line 4:

   ```text
   error[E3023]: `other.push(4)` changes the list, but `other` only reads the list it shares
     = help: to change it through `other`, write `other = shares mut items` in place of line 3
   ```

   (The first line already says `changes Array[int]`, so the line given compiles.)

   `two_owners` without `changes Array[int]` (R5, E2105; the rules' row for a change through a link, in
   this program's names):

   ```text
   error[E2105]: `two_owners` changes the list that its own link `other` names (`other.push(4)`, line 4), and a change through a link is named by the type it reaches
     = help: write `fn two_owners() changes Array[int]:` in place of line 1
   ```

   With `other = shares mut items`, a `for` loop over `items` that pushes through `other` is refused
   (R8 (b), (c), E3020: `items` and `other` are one list; the message is AN-48's).
7. **Cost:** `copy of`: one allocation and a copy of the elements, as C's `malloc` + `memcpy` and Rust's
   `.clone()`. `snapshot = items`: nothing (the list header moves, three words). `shares items` and
   `shares mut items`: one counted block for the list (count and list header together, as Rust's
   `Rc::new(vec)`), made where `items` is made because a line says `shares`; a count +1 per owner made,
   each +1 after one compare with the count's maximum (R9, verdict 59; Rust's `Rc::clone` makes the
   same check), -1 per owner ended. `mut` and `changes Array[int]` cost nothing: they are checked when
   compiling.
8. **Kind:** a readable spelling of Rust's guarantee (Rust's move, `.clone()` and `Rc`, each named by the
   word that says it).

#### AN-35: partial move (E3042) and move in a loop (E3041)

**v4:** **Meaning changed:** no. No verdict of 53 to 82 changes a NEW line: `n = p.name` stays a
read-only view (R2). Verdict 78 lets a field of a struct value be handed over on its own, as in Rust,
which the OLD's partial move was; part 6 gains that case's message (E3040, the rules' text in this
program's names), and E3060's way out now names the line it moves (verdict 73). The NEW now shows
`struct Player`, with the `@derive(Clone)` that `copy of template` needs (E2116 without it); E3013's
help names that line too, since the OLD's `Player` has none (worklist item 2, verdict 73).

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
   @derive(Clone)                    # `copy of template` below makes a player with `Player`'s `Clone` (R3)
   struct Player:
       name: String
       inventory: Array[String]

   fn keep(owns p: Player):
       pass

   fn log_and_keep(owns p: Player):
       n = p.name                    # a read-only view of p's name (R2): p stays whole
       println(n)
       keep(p)                       # after n's last use; a hand-over to an `owns` parameter has no word (R4)

   fn spawn_all(template: Player, n: int) -> Array[Player]:
       v: Array[Player] = []
       for _ in 0..n:
           v.push(copy of template)  # a new player each pass
       return v
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `name`, `inventory` | none (default: a field holds its value inside the player) | none (default: changed through a name that may change the player) |
   | `p` (both functions) | `owns` on the parameter: handed over to this function | none (default, own local: it is the function's own) |
   | `n` (first function) | none (default, reads only: a name for a part of `p`, R2, as row 8's plain `path`) | none (default, reads only) |
   | `template` | none (default, reads only: a plain parameter, R4) | none (default, reads only) |
   | `n` (parameter) | none (default: an `int` parameter, a number copied in, R4) | none (default: never changes the caller's) |
   | `v` | none (default, own local: the list made on this line) | none (default, own local) |
   | `_` | declares no name (the `_` row of `V4/vocabulary.md`): nothing to ask | — |

5. **Means exactly:** `an35_tr_ref.em` (today's `n = ref p.name` then `keep(p)`: `exit=0`, no output);
   `an35_tr_loop.em` (`template: Player`, `v.push(template.clone())`: `exit=0`, no output).
6. **Still refused:**

   The refused program (the NEW's `log_and_keep` with the hand-over moved up):

   ```ember
   struct Player:
       name: String
       inventory: Array[String]

   fn keep(owns p: Player):
       pass

   fn log_and_keep(owns p: Player):
       n = p.name
       keep(p)
       println(n)
   ```

   ```text
   error[E3060]: `n` views `p.name` (line 9), and `keep(p)` hands `p` over on line 10 while `n` is used on line 11
     = help: use `n` before the hand-over: move `println(n)` (line 11) above line 10; or keep your own copy: write `n = copy of p.name` in place of line 9
   ```

   ```text
   error[E3013]: `v.push(template)` would hand over the caller's player, but `template` only reads it
     = note: `template: Player` is a plain parameter: a read-only view of the caller's player, which stays the caller's
     = help: to put a player of your own in the list write `v.push(copy of template)` in place of line 8, and `@derive(Clone)` above `struct Player`
   ```

   (R2 and the E3060 row for the first: a view used after what it points into was handed over; R4 and
   the E3013 row of `V4/rules.md` section 3 for the second, whose help is that row's, with the line it
   replaces and, since the OLD's `Player` has no `Clone`, the line `copy of` needs, E2116 otherwise:
   worklist item 2, verdict 73.)
   The OLD's `owned` is not a word of v4 (E1010: "`owned` is now `owns`", the `owned` row of
   `V4/vocabulary.md`); the two refusals above are for the OLD bodies in v4's spelling.

   A field handed over on its own, then the player used whole (R2, verdict 78; the E3040 row of
   `V4/rules.md` section 3, in this program's names). Handing `p.name` over is allowed, and so is
   reading `p.inventory` after it; `keep(p)` is not, since `p` has a hole where its name was:

   ```ember
   struct Player:
       name: String
       inventory: Array[String]

   fn keep(owns p: Player):
       pass

   fn file_and_keep(owns p: Player, ref mut names: Array[String]):
       names.push(p.name)
       println(p.inventory.len())
       keep(p)
   ```

   ```text
   error[E3040]: `p.name` was handed over to `names` on line 9, and `keep(p)` uses `p` whole on line 11
     = help: give it a new value first: write `p.name = String("")` above line 11; or keep a copy: write `names.push(copy of p.name)` in place of line 9
   ```
7. **Cost:** `n = p.name`: nothing (a pointer). `copy of template`: one copy per pass, as C's
   per-element copy and Rust's `.clone()` in the loop.
8. **Kind:** a readable spelling of Rust's guarantee.

#### AN-36: a write to a handed-over value is accepted and lost

**v4:** **Meaning changed:** no; the second form gains a first line (verdict 64). Its two writes,
`child.value = 5` through the node's first owner after it is shared and `root.children[0].value = 6`
through a `shares mut` element, are changes through a link, so the function holding them says
`changes Node` (R5); the two forms are now written as whole functions. E3040's ways out name the lines
they replace, and its third gives that first line (verdict 73). Counts are compared before each +1
(verdict 59). The cycle warning L3001 is R11's.

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
   fn main():
       root = Node(value=0)
       child = Node(value=1)
       child.value = 5
       root.children.push(child)            # hands the child over (R2: a value stored somewhere else)
       root.children[0].value = 6

   # or the child has two owners, the tree and this function, and both may change it (Python's meaning)
   struct Node:
       value: int
       children: Array[shares mut Node] = []

   fn main() changes Node:                  # it changes a node through a link (R5)
       root = Node(value=0)
       child = Node(value=1)
       root.children.push(shares mut child)
       child.value = 5                      # seen as root.children[0].value
       root.children[0].value = 6           # seen as child.value
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `root`, `child` | none (default, own local: the node made on this line) | none (default, own local) |
   | `value` | none (default: a field holds its value inside its node) | none (default: changed through a name that may change the node) |
   | `children` | first form: none (default: a list held by its node); second: `shares` in the element type, each child has several owners | first form as `value`; second: `mut` in `shares mut`, the list's owners may change each child |
   | `child` after `push(shares mut child)` | the first owner of the node (R3: the name a value is first shared from stays an owner) | none (default, own local: it keeps what it could do, R3, R9; verdict 23); the change through it is named on the first line, `changes Node` (R5; verdict 64) |
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
     = help: set it before the push: move `child.value = 5` (line 9) above line 8; or change it in the list: write `root.children[0].value = 5` in place of line 9; for two owners that both change it write `children: Array[shares mut Node] = []` in place of line 3, `root.children.push(shares mut child)` in place of line 8, and `fn main() changes Node:` in place of line 5
   ```

   In the second form, a list that only reads its children (`children: Array[shares Node]`) cannot be
   changed through: `root.children[0].value = 6` is refused (E3023: "`root.children[0]` only reads the
   node it shares; to change it through the list write `children: Array[shares mut Node] = []` in place
   of line 3 and `root.children.push(shares mut child)` in place of line 8"; the first line already says
   `changes Node`, for `child.value = 5`).

   Warned, not refused: the second form's `children: Array[shares mut Node]` lets a `Node` own itself
   through its children, so it gets the cycle warning L3001 on the type (R11), as today's class form
   above does; the list is the owning link and stays, so the warning names what would close the cycle:
   "`Node.children` holds owners of nodes, so a node put among its own children
   (`child.children.push(shares mut child)`) would own itself and never be freed; push each node under
   one parent only, as `root.children.push(shares mut child)` does; a link back up to a parent is one
   that does not keep it alive: `parent: weakref Node = None`".
7. **Cost:** nothing: a compile-time refusal. The `shares mut` form pays one counted block per child
   and the counts, each +1 after one compare with the maximum (R9, verdict 59), as Rust's `Rc<Node>`
   (with `RefCell` for the change, whose flag these rules do not pay).
8. **Kind:** a readable spelling of Rust's guarantee (Rust refuses the same write, `E0382`, run 2's
   `rust_probes/a11_assign_moved_field.rs`).

#### AN-37: moving a field out from behind `mut self` (`match owned self.state` is E3013)

**v4:** **Meaning changed:** no. No verdict of 53 to 82 changes a line: `path` is a `ref mut` name of
the NPC's own walk list, not a link, so `changes state` says all `update` changes (R5); `this` is the
caller's NPC, so nothing is taken out of it (verdict 78 keeps a field of a value one does not own from
being handed out; `mem.replace` is the way, as part 6 says). Replacing `this.state` while `path` is
still used stays E3022 (R8 (f), verdict 2).

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
                       this.state = State.Idle   # after `path`'s last use
               _:
                   pass
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `this` | `this`: the NPC the method was called on (R5) | `changes state` on the first line: this method may change `state` only (row 3) |
   | `state` | none (default: a field holds its value inside the NPC) | `changes state` |
   | `path` | `ref mut`: a name for the NPC's own walk list, the only one while it is used (row 8, R8 (f)) | `mut` in `ref mut` |
   | `_` | declares no name (the `_` row of `V4/vocabulary.md`): nothing to ask | — |

5. **Means exactly:** `an37_tr_refmut_inline.em` (the same body with `fn update(mut self):`, today's
   spelling of `changes state`): `exit=0`, no output. Today already ends `path` at its last use
   (`path.is_empty()`), so the store to the state inside the branch is accepted.
6. **Still refused:**

   ```text
   error[E3013]: `match owns this.state` would take the state out of the NPC and leave it with none
     = note: `update` says `changes state`: it may change the NPC in place, but the NPC stays its caller's, so nothing can be taken out of it
     = help: to take the state out and put one back, as this body does, write `match owns mem.replace(ref mut this.state, State.Idle):` in place of line 11; or change the path where it is: in place of lines 11 to 19 write `match this.state:` / `State.Walking(ref mut path):` / `path.pop()` / `if path.is_empty():` / `this.state = State.Idle` / `_:` / `pass`
   ```

   (The OLD's own spelling, `fn update(mut self):` with `match owned self.state:`, is refused first for
   its words: E0113 for `mut self` and E1010 for `owned` and `self`, each with the line to write, as
   `V4/rules.md` section 3 gives them; the message above is for the same body in v4's words, line for
   line, so its lines are the OLD's: the `match` on line 11, the `other:` branch ending on line 19. The
   second way out does not put back the path it changed: it changes it in place, and the old
   `State.Walking(path)` and `other` lines would hand over names that do not own their values, E3013.)

   And, while `path` is still used, `this.state = State.Idle` before `path.pop()` (R8 (f): `path` is the
   only name for the walk list in `this.state` until its last use, and replacing the state changes that
   place through another name):

   ```text
   error[E3022]: `path` is the only name for the walk list in `this.state` until its last use on line 14, and line 13 replaces `this.state`
     = help: set the new state after `path`'s last use: write `this.state = State.Idle` below `path.pop()` (line 14) instead of on line 13
   ```

7. **Cost:** nothing: `path` is a pointer into the enum's payload; the pop and the store are C's.
8. **Kind:** a readable spelling of Rust's guarantee (Rust's `Walking(path) => ` on a `&mut self.state`,
   with the `&mut` written on the name it gives).

#### AN-38: `State.Walking(path):` then `path.pop()` is E3021 with a `.clone()` help; `mut path` is accepted read-only

**v4:** **Meaning changed:** no. No verdict of 53 to 82 touches it; row 8's two agreed helps are word
for word.

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

**v4:** **Meaning changed:** no; `build`'s first line changes (verdicts 63, 64). `a.mesh.verts.push(1.0)`
changes the mesh through `a.mesh`, a `shares mut` link in `build`'s own entity, so `build` says
`changes Mesh` (R5: a change through a link is named, whatever holds the link). The table of kinds gains
the making line of an `owns` field (verdict 74) and a link row's naming of changes (verdicts 63, 64);
the E3040 message gives whole lines, the first line included (verdict 73); counts are compared before
each +1 (verdict 59).

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
       mesh: shares mut Mesh

   fn build() changes Mesh -> (Entity, Entity):    # it changes a mesh through a link (R5)
       m = Mesh(verts=[0.0, 0.0, 0.0])
       a = Entity(mesh=shares mut m)
       b = Entity(mesh=shares mut m)
       a.mesh.verts.push(1.0)           # b sees four vertices
       return (a, b)
   ```

   The choice is made from the intent, and the word is on the line:

   | the program means | written | replaces |
   |---|---|---|
   | a value that is part of its owner | plain `T` (no word) | `struct` field |
   | one owner, in its own heap block (a type containing itself, `dyn I`, one pointer) | `owns T`; the line that makes the value for an `owns` field says so, `Entity(mesh=owns Mesh(verts=[0.0]))` (R3, verdict 74) | `Box[T]`, `Box[dyn I]`, `ForeignBox[T]` |
   | its heap memory from a memory manager the program names | the kind in the type, `owns T using A`, `Array[T] using A`; the instance on the line that makes the value, `owns Mesh(verts=[0.0]) using current` (R3) | `Box[T, A]`, `Array[T, A]`, with the instance passed at construction |
   | several owners on one thread, which read it | `shares T`, `shares x` | `Shared[T]`, a class handle's copy |
   | several owners on one thread, which may change it | `shares mut T`, `shares mut x`; a function that changes the value through one says so on its first line, `changes Mesh` (R5) | `Shared[T]` with `get_mut` |
   | several owners on any threads | `thread_shared T` (area E) | `SyncShared[T]` |
   | a link that does not keep its target alive | `weakref T` (area D) | `Weak[T]` |
   | a link back to the object that owns this one | an ordinary `weakref T` field the program sets, `child.parent = weakref this` before `this.children.push(child)` (R11) | hand-kept `Weak` |
   | a name for someone else's place that reads | `ref T` (any number of names) | the same |
   | a name for someone else's place that changes it, the only one while used | `ref mut T` | the same |
   | a changeable link several holders share, not owning | `sharedref mut T` (R7); a change through it is named the same way, `changes Player` (R5) | two kept `ref mut` views of one object, which `ref mut` does not allow |
   | one value replaced whole through a read-only path | `Cell[T]` | the same |
   | a value threads change | `Mutex`, `RwLock`, `Atomic` (area E); a function that changes one says so on its first line, `changes Mutex[Stats]` (R12) | the same |
   | objects freed together, or found by position | `Arena`, `Pool` + `Handle[T]` (libraries) | the same |
   | a borrow checked while running | removed (AN-49) | `RefCell` |
   | an object with identity and subclasses | `class` (one owner unless `shares` or `shares mut`) | `class` |

   A parent link costs what R11 says: one add to the parent's weak count when it is set (after one
   compare with the count's maximum, verdict 59) and one subtract when it changes or the child ends; a
   read with no test of the parent's header where the compiler proves the parent lives, else one test per
   read (the costs are in AN-41).

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `verts` | none (default: a field holds its value inside the mesh) | none (default: changed through a name that may change the mesh) |
   | `mesh` | `shares` in the type: one of several owners of a mesh | `mut` in `shares mut`: it may change the mesh (R9); here through `a`, the function's own, and `build`'s first line says `changes Mesh` (R5) |
   | `m` | none (default, own local: the mesh made on this line; after `shares mut m` it is the first of its owners, R3) | none (default, own local: it keeps what it could do, R3, R9; verdict 23) |
   | `a`, `b` | none (default, own local: the entity made on this line) | none (default, own local) |

5. **Means exactly:** `an39_tr_shared_mesh.em` (`mesh: Shared[Mesh]`, `a.mesh.get_mut().verts.push(1.0)`):
   `exit=0`, no output. Today's `get_mut` is checked while running; the NEW change is checked when
   compiling (R8 (b), R9).
6. **Still refused** (R1, E3040). The OLD program in v4's words:

   ```ember
   struct Mesh:
       verts: Array[f32]

   struct Entity:
       mesh: owns Mesh

   fn build() -> (Entity, Entity):
       m = Mesh(verts=[0.0, 0.0, 0.0])
       a = Entity(mesh=m)
       b = Entity(mesh=m)
       a.mesh.verts.push(1.0)
       return (a, b)
   ```

   ```text
   error[E3040]: `m` was handed over to `a`'s mesh on line 9 and is used on line 10
     = help: for one mesh in both entities write `mesh: shares mut Mesh` in place of line 5, `a = Entity(mesh=shares mut m)` and `b = Entity(mesh=shares mut m)` in place of lines 9 and 10, and `fn build() changes Mesh -> (Entity, Entity):` in place of line 7; for two meshes write `@derive(Clone)` above line 1 and `a = Entity(mesh=copy of m)` in place of line 9
   ```

   (`Entity(mesh=m)` hands over a mesh made on line 8, so it takes no `owns`: R3's `owns` on a making
   line is for a value made on that line, as `Entity(mesh=owns Mesh(verts=[0.0]))` in AN-40. In the
   second way out `a` gets the copy and `b` the mesh itself, so nothing is used after its hand-over; the
   push then changes `a`'s own mesh, not through a link, and the first line says nothing.)

   With `mesh: shares Mesh` (owners that only read), `a.mesh.verts.push(1.0)` is refused (E3023:
   "`a.mesh` only reads the mesh it shares; to change it through `a.mesh` write `mesh: shares mut Mesh`
   in place of line 5 and `a = Entity(mesh=shares mut m)`, `b = Entity(mesh=shares mut m)` in place of
   lines 9 and 10"; the NEW's first line already says `changes Mesh`).

   `Box`, `Shared`, `SyncShared`, `Weak`, `RefCell`: each refused with the word that replaces it
   (E1010: "write `owns T`", "write `shares T` or `shares mut T`", and so on; AN-40, AN-49; the rows of
   `V4/vocabulary.md` section 3.4).
7. **Cost:** one counted block for the mesh and a count +1/-1 per owner made or ended, each +1 after one
   compare with the count's maximum (R9, verdict 59), as Rust's `Rc<Mesh>`; no borrow flag (Rust needs
   `Rc<RefCell<Mesh>>` to edit it, and pays a flag per borrow). `changes Mesh` costs nothing: it is
   checked when compiling.
8. **Kind:** a readable spelling of Rust's guarantee (one family of words, each saying its intent).

#### AN-40: `Box[T]` says nothing about one owner; `ForeignBox[T]` carries the same word

**v4:** **Meaning changed:** no; one making line changes (verdict 74). `e = Entity(mesh=Mesh(verts=[0.0]))`
becomes `e = Entity(mesh=owns Mesh(verts=[0.0]))`: the line that makes a value for an `owns` field says
`owns`, with or without an arena, and the line without it is refused (E2123, R3). This settles v3's note
4 (whether `owns` may stand before a value made on the line). Part 6 gains E2123's message; the E2020
example is written with `owns`, so that it has one fault; the `Box` message names the making line too
(verdict 73). The arena's address kept in each arena-made value (one word) is the owner's verdict 47,
"accepted for now".

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
3. **NEW** (the owner's ruling; `owns` on a foreign type; `using` for the memory manager):

   ```ember
   struct Entity:
       mesh: owns Mesh

   struct Registry:
       pools: Array[owns dyn Pool]

   @ffi(destroys(sqlite3_close))
   type SqliteDb

   struct App:
       db: owns SqliteDb

   struct Level:
       mesh: owns Mesh using LevelArena    # the block comes from a `LevelArena` (a type implementing `Allocator`, R3)

   e = Entity(mesh=owns Mesh(verts=[0.0]))    # `owns`: the mesh is put in a block of its own, held by `e.mesh`
   r = Registry(pools=[])
   r.pools.push(IntPool(items=[1, 2]))    # put in its own block at the push
   current = LevelArena()
   lvl = Level(mesh=owns Mesh(verts=[0.0]) using current)    # this mesh's block comes from the arena `current`
   println(e.mesh.verts.len() + r.pools[0].len() + lvl.mesh.verts.len())
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `mesh` (`Entity`) | `owns`: held alone, in its own block | none (default: changed through a name that may change the entity) |
   | `mesh` (`Level`) | `owns`: held alone, in its own block; `using LevelArena` says only which kind of arena the block's memory comes from, and the line that makes the level names which one (`using current`, R3) | none (default: changed through a name that may change the level) |
   | `pools` | `owns dyn Pool` elements: each pool held alone | as `mesh` |
   | `db` | `owns` on a foreign type: held alone, freed through `sqlite3_close` | as `mesh` |
   | `e`, `r` | none (default, own local: the value made on this line) | none (default, own local) |
   | `current` | none (default, own local: the arena made on this line) | none (default, own local: making a mesh in it, `using current`, changes the arena, which is this code's own) |
   | `lvl` | none (default, own local: the level made on this line; its mesh's block comes from `current`, so `lvl` lives no longer than `current`, R3) | none (default, own local) |
   | `verts`, `items` | none (default: a field holds its value) | as `mesh` |

5. **Means exactly:** `an40_old_box.em` itself (`Box[Mesh]`, `Box[dyn Pool]`): `exit=0`, no output; the
   foreign case is today's `ForeignBox[SqliteDb]` (`[FFI-36a]`, spec 4883-4887). The `using` field and
   its making line are, in the spec, `Box[Mesh, LevelArena]` with the instance passed at construction
   (`[ALC-1]`, spec 2986-2988: "an allocator instance is passed at construction (`Array.new_in(a)`)").
   Today's compiler has neither. A probe of this step
   (`target/redesign4/v3-04-area-C/probes/c3_box_in_arena.em`: a `LevelArena` implementing `Allocator`
   by `unsafe extend`, written as the conformance test
   `tests/conformance/ALC-2/accept_an_unsafe_extend_implements_allocator.em` writes it, a field
   `mesh: Box[Mesh, LevelArena]`, and `Box.new_in(Mesh(verts=[0.0]), current)`):

   ```text
   === c3_box_in_arena.em
   error[E2020]: `Box` takes one type argument
     --> c3_box_in_arena.em:21:11
      |
   21 |     mesh: Box[Mesh, LevelArena]
      |           ^^^^^^^^^^^^^^^^^^^^^
      |

   error[E1010]: cannot find `Box` in this scope
     --> c3_box_in_arena.em:25:22
      |
   25 |     lvl = Level(mesh=Box.new_in(Mesh(verts=[0.0]), current))
      |                      ^^^
      |

   error: could not compile due to 2 error(s)
   exit=1
   ```

   So the arena form is the spec's, not today's Ember's. In C it is an allocation from `current`'s own
   block, with the arena's address kept beside the mesh's pointer (part 7); no C file was made for it in
   this step.
6. **Still refused** (the `Box`, `ForeignBox` and `using` rows of `V4/vocabulary.md`):

   ```text
   error[E1010]: `Box` is not part of Ember
     = help: for a mesh held alone in its own block write `mesh: owns Mesh`, and make it with `owns` where it is made: `e = Entity(mesh=owns Mesh(verts=[0.0]))`; for `Box[dyn Pool]` write `owns dyn Pool`, and `r.pools.push(IntPool(items=[1, 2]))` for `Box(IntPool([1, 2]))`; for `Box[Mesh, LevelArena]` write `mesh: owns Mesh using LevelArena`, and on the line that makes the value name the arena: `lvl = Level(mesh=owns Mesh(verts=[0.0]) using current)`
   ```

   A mesh made for the `owns` field without the word (R3, verdict 74; the E2123 row of `V4/rules.md`
   section 3, in this program's names):

   ```text
   error[E2123]: `mesh` owns its mesh; say so where it is made
     = help: write `e = Entity(mesh=owns Mesh(verts=[0.0]))`
   ```

   `ForeignBox[SqliteDb]` (E1010): "write `owns SqliteDb`". `owns Player` on a class type (E2114, R3):
   "a `Player` object always lives in its own block; remove `owns`: `p: Player`". `count: int using
   LevelArena` (E2114, R3): "an `int` has no memory of its own to take from `LevelArena`; remove
   `using`". `e.mesh.get()` on an `owns` field (E2072, today's code for a missing method): "an `owns
   Mesh` reads as a `Mesh`, which has no `get`; remove `.get()`: `e.mesh.verts`". Today's E2072, on a
   probe of the same shape (`target/redesign4/04-area-C/probes/c2_no_method.em`, `m.get()` on a
   `struct Mesh`; run 4's output, carried over):

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

   The arena's two checks (R3; the E2020 and E3063 rows of `V4/rules.md` section 3, in this program's
   names). A level made without saying which arena, `lvl = Level(mesh=owns Mesh(verts=[0.0]))`:

   ```text
   error[E2020]: `mesh` takes its memory from a `LevelArena` (`mesh: owns Mesh using LevelArena`), and this `Mesh` comes from the built-in manager
     = help: say which arena: `lvl = Level(mesh=owns Mesh(verts=[0.0]) using current)`
   ```

   A function that makes the arena and returns the level made in it:

   ```ember
   fn load_level() -> Level:
       current = LevelArena()
       lvl = Level(mesh=owns Mesh(verts=[0.0]) using current)
       return lvl
   ```

   ```text
   error[E3063]: `lvl` holds a mesh made in `current` (line 3), and `current` ends when `load_level` returns, so `lvl` cannot be returned (line 4)
     = help: make it in an arena the caller keeps: `fn load_level(ref mut arena: LevelArena) -> Level:`, with `lvl = Level(mesh=owns Mesh(verts=[0.0]) using arena)`, and in the caller `current = LevelArena()` and `lvl = load_level(ref mut current)`
   ```

7. **Cost:** the block `Box` made: one allocation at the store, one pointer in the owner, freed with it;
   the same as C's `malloc` and Rust's `Box`. With `using current`, the allocation and the free are what
   the arena `current` does (a pointer bump, nothing freed one by one; not measured, `V4/rules.md`
   section 2), as Rust's `Box::new_in(mesh, &current)`. A `LevelArena` may have several instances
   (`current`, and any other made the same way), so `lvl.mesh` keeps the address of the arena it came
   from, one word beside the pointer, so that its end goes back to `current`, as Rust's
   `Box<Mesh, &LevelArena>` keeps its `&LevelArena` (R3; not measured); a manager with one instance
   (`Global`, `System`) costs no word. The word may be saved where the compiler, which sees the kind,
   knows the end needs no arena (a kind whose `deallocate` does nothing and a value that never grows,
   as this mesh), or where the arena's chunks are aligned so that the pointer finds its arena; neither
   is in the rules, and neither is measured (For the owner, below). The arena checks (the kind, and nothing made in `current`
   outliving it) are made when compiling and cost nothing while running. Ending an `App`'s `db` calls
   `sqlite3_close`, a C function, which counts as possibly running the callbacks C keeps (R12,
   verdict 31); this program hands C none, so it changes nothing here (note 7 below).
8. **Kind:** a readable spelling of Rust's guarantee.

#### AN-41: a type that contains itself crashes the checker

**v4:** **Meaning changed:** no. Making a `weakref` link stays no change of its target (R11), so `add`
needs no word beyond `changes children`; the weak count is now compared with its maximum before each +1
(verdict 59), which part 7 counts. A `Node` is ended the moment its last owner lets it go, before any
`drop` runs (verdict 56); `Node` has no `drop`, so nothing else changes. `t`'s making line keeps
`left=Some(Tree(value=2))`: `left` is an `Option[owns Tree]` field, and R3 requires `owns` on the making
line of an `owns` field, which the v4 rules do not say covers an `owns` inside an `Option` (For the
owner, below). The E2200 messages and the parent link's costs are those of `V4/rules.md` sections 2
and 3.

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
       left: Option[owns Tree] = None
       right: Option[owns Tree] = None

   t = Tree(value=1, left=Some(Tree(value=2)))
   ```

   A node with a link back to its parent, the form the second message below gives (the parent links
   are area D's, R11):

   ```ember
   class Node:
       value: int
       children: Array[Node] = []
       parent: weakref Node = None

       fn add(owns child: Node) changes children:
           child.parent = weakref this       # the program sets the link ...
           this.children.push(child)          # ... then puts the child in the place its parent owns
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `value` | none (default: a field holds its value) | none (default: changed through a name that may change the tree or node) |
   | `left`, `right` | `owns`: each child held alone in its own block | as `value` |
   | `t` | none (default, own local: the tree made on this line) | none (default, own local) |
   | `children` | none (default: a list held by its node; each `Node` in it is a class object, one owner, R3) | `changes children` on `add` |
   | `parent` | `weakref`: a link that does not keep its target alive, which may hold none (`= None`) | as `value` |
   | `this` (in `add`) | `this`: the node the method was called on | `changes children` on the first line; `child.parent = weakref this` changes `child`, the function's own, and only the weak count of `this`, which is not a change of the node (R11) |
   | `child` | `owns` on the parameter: handed over to `add` | none (default, own local: it is the function's own until it is pushed) |

5. **Means exactly:** `an41_tr_box.em` (`Option[Box[Tree]]`): `exit=0`, no output. No file of today's Ember
   was made for the `Node` class in this step (its parent link is area D's, which carries today's
   counterparts).
6. **Still refused** (the crash becomes these messages; E2200, R3; the second is `V4/rules.md`
   section 3's):

   ```text
   error[E2200]: `Tree` contains itself through `left`, so a `Tree` would have no end
     = help: hold the child in its own block: `left: Option[owns Tree]`
   ```

   ```text
   error[E2200]: `Node.parent` would be a view of the `Node` that holds this one, which no struct can keep
     = help: for a link back to the parent make `Node` a class with `children: Array[Node]` and `parent: weakref Node = None`, set with `child.parent = weakref this` before `this.children.push(child)`
   ```

7. **Cost:** one block per node, as `Box` and Rust's `Option<Box<Tree>>` (one pointer, `None` is null).
   The parent link of the `Node` class, honestly (R11; `V4/rules.md` section 2; nothing measured in this
   step): one add to the parent's weak count when `child.parent = weakref this` runs, after one compare
   with the count's maximum (R11, verdict 59), and one subtract when the link changes or the child ends
   (not measured); each read of the link is a load and a test
   for no link where the compiler proves the parent lives (`add` does its part of R11 condition 1: it
   sets the link and then puts the child in a place its parent owns alone; the proof also needs the
   rest of the package to keep the condition and nothing to end a `Node` before the read, R11), else
   one test of the parent's header per read. Run 3's parent walk W2 ran at 1.176x of plain C with no header test per step and at
   1.579x with one per step, against safe Rust's 1.603x, all measured with uncounted links
   (`RUN3/26-timer.md` lines 20 to 22); the weak count comes on top, unmeasured.
8. **Kind:** an understandable limitation kept (Rust needs `Box` too), now written in the owner's word
   and with a message instead of a crash; the parent link is Rust's `Weak`, read with no test where the
   compiler can prove its target lives (a smarter compile-time solution, R11).

#### AN-42: `struct`, `Box` or `class` decides what compiles and what panics

**v4:** **Meaning changed:** no. Verdict 72 says in so many words what the refusal relied on: the loop
rule covers maps as lists, and `reg.values[int]()` walks the map `pools`, so `reg.emplace(3, 30)` inside
the loop is refused at the change (E3020). The pools are `owns dyn AnyPool`, held alone, not links, so
`changes pools` says all the methods change (verdicts 63, 64 add nothing). `get_mut`'s
`cast this.pools[7u128] as Pool[T]` stays the "sure" cast (verdict 27), whose failure now stops the
program with the text of `V4/rules.md` section 2, in this program's names (for `get_mut[int]` on a map
holding a `Pool[float]` there: "`cast this.pools[7u128] as Pool[T]` on line 6: `this.pools[7u128]` is a
`Pool[float]`, not a `Pool[int]`"); a cast of a place is still a place, so `pool` is a `ref mut` name
of the pool in the map (R14).

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
   `owns` block and a class:

   ```ember
   struct Reg:
       pools: Map[u128, owns dyn AnyPool]

       [T]
       fn get_mut(i: int) changes pools -> ref mut T:
           pool = ref mut cast this.pools[7u128] as Pool[T]
           return ref mut pool.dense[i]

       [T]
       fn emplace(e: int, owns value: T) changes pools:
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
   | `pools` | `owns dyn AnyPool` elements: each pool held alone | `changes pools` on the methods that change it |
   | `this` | `this`: the registry the method was called on | `changes pools` on each method's first line |
   | `pool` | `ref mut`: a name for a pool in the map, the only one while used (R14: a cast of a place is a place) | `mut` in `ref mut` |
   | `i`, `e` | none (default: a number parameter, copied in) | none (default, reads only) |
   | `value` | `owns`: handed over to `emplace` (a `Copy` `T` such as `int` is copied, the `owns` row of `V4/vocabulary.md`) | none (default, own local) |
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
     = help: add after the loop: write `reg.emplace(3, 30)` below the loop instead of in it
   ```

   The `run-pass` test `ecs4/accept_change_during_loop.em` becomes a compile-fail test with this message.
7. **Cost:** nothing while running: today's class pools carry access words checked while running
   (`[EXC-19]`), which go (R9).
8. **Kind:** a smarter compile-time solution (one rule set for every kind; the kind changes the layout,
   never what compiles or what panics).

#### AN-43: `owned` on a parameter means three things by type; `mem.drop(owned x)` moves a handle

**v4:** **Meaning changed:** no. No verdict of 53 to 82 changes a NEW line: `take` ends the `Node` it is
handed, and the call shows it (the argument `a` is handed over), so neither `take` nor its caller needs a
word for it (R10 point 1 reads a call's arguments; verdict 55's word is for a function value's own
state, which no call shows). Two E3040 messages now name every line they replace (verdict 73; the
safety check's fix, `V4/safety/check-V1.md` and `check-V2.md`).

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

   fn take(owns n: Node) -> int:           # takes the object over
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
   | `n` (in `take`) | `owns` on the name: handed over to `take` (the word on the name sets it, for every type: R4, verdict 32) | none (default, own local) |
   | `v` | none (default: a field holds its number) | none (default: through a name that may change the node) |
   | `n` (in `add_one`) | none (default: a number parameter, copied in, R4) | none (default: never changes the caller's) |
   | `a`, `b` | none (default, own local: the value made on this line) | none (default, own local) |
   | `x`, `y` | none (default, own local: the number the call returns) | none (default, own local) |

5. **Means exactly:** `an43_tr_handover.em` (`take(a)` with `a` not used again, `take_int(n: int)`):
   `exit=0`, no output. Today's `owned n: Node` gives the callee a second owner; the hand-over is
   exactly today's code when the caller does not use `a` again, which the NEW refusal guarantees.
6. **Still refused** (the OLD files' `owned` is not a word of v4, E1010 "`owned` is now `owns`"; the
   messages below are for the OLD programs in v4's spelling):

   ```text
   error[E3040]: `a` was handed over to `take` on line 11 and `a.v` reads it on line 12
     = help: read it before the call: write `v = a.v` above line 11 and `println(v)` in place of line 12; or give `take` a copy of its own: write `@derive(Clone)` above `class Node` and `x = take(copy of a)` in place of line 11
   ```

   (The program: the NEW block with `println(a.v)` as line 12, right after `x = take(a)`.)

   ```text
   error[E2085]: `owns n: int`: an `int` is copied, not handed over
     = help: remove `owns`: write `fn take(n: int) -> int:`
   ```

   ```text
   error[E3040]: `a` ended at `mem.drop(a)` on line 6 and is used on line 7
     = help: use `a` before it ends: move line 7 above line 6; or remove line 6, and `a` ends at the end of its function
   ```

   (for every type, an `int` included: 06's `k3_mem_drop_int.em` is accepted today).
   `fn take(n: owns Node)` (E2114, R3): "a `Node` object always lives in its own block, so `owns` in
   its type says nothing; to take it over write `owns n: Node`; to read it write `n: Node`".
   `take(shares a)` into `owns n: Node` (E2020): "`take` takes a `Node` held alone, and `shares a` is a
   second owner; to let `take` be one more owner declare `owns n: shares Node` (R4)". Today's E2020 is a
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
8. **Kind:** a readable spelling of Rust's guarantee (`owns` is Rust's move, for every type).

#### AN-44: `b = a` has no word and does three things by type

**v4:** **Meaning changed:** no; the lines are now written as `main`'s body, whose first line says
`changes Player` (verdict 64): `p2.hp = 5` changes the player through `p2`, a `shares mut` owner, and a
change through a link is named whatever holds it (R5). Verdict 68 settles `mine = copy of p.pos` for a
`Vec2` marked only `@derive(Copy)` (every `Copy` type has `Clone`). The messages name every line they
replace, and the E3040 message's second way out, which ends the change through a link, gives the first
line without `changes Player` (verdict 73). The count is compared before each +1 (verdict 59).

1. **Intent:** `p2 = player` and `snapshot = items` in one program.
2. **OLD** (`an44_old_three.em`: `n2 = n` of an `int`, `snapshot = items`, `p2 = player` of a class,
   `p2.hp = 5`, `println(player.hp)`): `exit=0`, no output; it prints `5` (the class is shared). And the
   struct case (`an44_old_copy_struct.em`, last run's attack: `pos = p.pos; pos.x += 1.0` of a
   `@derive(Copy)` struct, Python prints `2.0`): `exit=0`, no output; it prints `1.0`, silently.
3. **NEW:**

   ```ember
   fn main() changes Player:      # it changes a player through a link, `p2` (R5)
       n = 1
       n2 = n                     # a number: copied (it has no parts to change)
       n2 += 1
       items = [1]
       snapshot = items           # handed over: `items` ends (row 9)
       player = Player()
       p2 = shares mut player     # one object, two owners, both may change it: a change through either is seen by both
       p2.hp = 5
       println(player.hp)         # 5
       p = P(pos=Vec2(x=1.0, y=2.0))
       pos = ref mut p.pos        # change p's position
       pos.x += 1.0
       mine = copy of p.pos       # your own copy to change (after pos's last use)
       mine.y = 0.0
   ```

   (`Player`, `P` and `Vec2` are the OLD files' types: a class `Player` with `hp`, a struct `P` with
   `pos: Vec2`, and `Vec2`, a struct marked only `@derive(Copy)`, with `x` and `y`. The lines are
   numbered from `fn main()`, line 1.)

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `n`, `n2` | none (default: a number; `=` copies it, R1) | none (default, own local) |
   | `items`, `player`, `p` | none (default, own local: the value made on this line) | none (default, own local) |
   | `snapshot` | none (default, row 9: `=` hands the list over) | none (default, own local) |
   | `p2` | `shares`: one of two owners of the player (`player` stays the first and keeps what it could do, R3, R9, verdict 23) | `mut` in `shares mut`: it may change the player (R9); `main`'s first line says `changes Player` (R5) |
   | `pos` | `ref mut`: a name for `p`'s position, the only one while used | `mut` in `ref mut` |
   | `mine` | `copy of`: its own new position (`Vec2` is `Copy`, so it has `Clone`: verdict 68) | none (default, own local) |
   | `hp`, `x`, `y` | none (default: a field holds its number) | through the name before them |

5. **Means exactly:** `an44_tr_three.em` (today's class `p2 = player` is exactly a second owner):
   `exit=0`, no output; `an44_tr_struct_refmut.em` (`pos = ref mut p.pos; pos.x += 1.0`, prints `2.0`):
   `exit=0`, no output; `copy of p.pos` is today's `p.pos` copy of a `Copy` struct.
6. **Still refused:**

   ```text
   error[E3040]: `player` was handed over to `p2` on line 8 and is used on line 10
     = note: `p2 = player` hands the object over, as for every value that is not a number
     = help: for one player with two names, both changing it, write `p2 = shares mut player` in place of line 8; for a second player write `@derive(Clone)` above `class Player`, `p2 = copy of player` in place of line 8, and `fn main():` in place of line 1 (the copy is `main`'s own, so nothing changes through a link)
   ```

   (The program: the NEW with `p2 = player` on line 8. Its first line then names a change the body no
   longer makes, E2105 too; by verdict 73 the message shown is E3040's, whose first way out makes the
   program compile as it stands.)

   ```text
   error[E3023]: `pos.x += 1.0` changes `p.pos` through `pos`, which only reads it
     = note: `pos = p.pos` names `p`'s position, a part of `p`, read-only
     = help: to change `p`'s position write `pos = ref mut p.pos` in place of line 12; to change your own copy write `pos = copy of p.pos` in place of line 12
   ```

   And reading `p.pos` while `pos` is still used (R8 (f), verdict 2): `mine = copy of p.pos` moved
   above `pos.x += 1.0`, as line 13, gives "`pos` is the only name for `p.pos` until its last use on
   line 14, and line 13 reads `p.pos`; read it through `pos`: write `mine = copy of pos` in place of line
   13; or move line 13 below line 14" (E3021).

   With `p2 = shares player` (an owner that only reads), `p2.hp = 5` is refused (E3023: "`p2` only
   reads the player it shares; to change it through `p2`, write `p2 = shares mut player` in place of
   line 8").
7. **Cost:** nothing; `shares mut player` is a count +1 after one compare with its maximum (R9, verdict
   59), and -1 when `p2` ends, as Rust's `Rc::clone`; `mut` and `changes Player` are checked when
   compiling and cost nothing.
8. **Kind:** a readable spelling of Rust's guarantee (Rust's move, `Copy`, `Rc::clone`), with the
   struct copy that Rust also makes silently now named.

#### AN-45: `.clone()` on a class handle shares the object instead of copying it

**v4:** **Meaning changed:** no. The lines are now `main`'s body, so that the messages can name whole
lines. Nothing changes a player through a link (`b` is a copy of `main`'s own, `c` only reads), so
`main`'s first line says nothing; the messages whose way out makes such a change (`b = shares mut a`,
`c = shares mut a`) give `fn main() changes Player:` too (verdicts 64, 73). E2116's message offers only
ways out that compile here (worklist item 2: a view of `a` cannot be changed). The count is compared
before each +1 (verdict 59).

1. **Intent:** make an independent copy of a player object and change it.
2. **OLD** (`an45_old_clone_class.em`: `b = a.clone()`, `b.health = 5`, `println(a.health)`): `exit=0`, no
   output; it prints `5` (spec 797, 1277: `clone` of a class is a second handle).
3. **NEW:**

   ```ember
   @derive(Clone)
   class Player:
       health: int

   fn main():
       a = Player(health=100)
       b = copy of a                     # a new player
       b.health = 5
       println(a.health)                 # 100
       c = shares a                      # the same player, a second owner that reads it
       println(c.health)                 # 100
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `health` | none (default: a field holds its number) | through the name before it |
   | `a` | none (default, own local: the object made on this line; after `shares a` the first of two owners, R3) | none (default, own local: it keeps what it could do, R3, R9; verdict 23) |
   | `b` | `copy of`: its own new object | none (default, own local) |
   | `c` | `shares`: a second owner of `a`'s object | none (default: a `shares` owner only reads, R9; the line says no `mut`) |

5. **Means exactly:** `an45_tr_copied.em` (`b = Player(a.health)`, what `Player`'s `Clone` runs):
   `exit=0`, no output.
6. **Still refused:**

   ```text
   error[E1010]: `a.clone()`: a copy is written `copy of a`
     = help: for a new player write `b = copy of a` in place of line 7; for a second owner of this same player, which changes it, write `b = shares mut a` in place of line 7 and `fn main() changes Player:` in place of line 5
   ```

   (The program: the NEW with `b = a.clone()` on line 7, the OLD's line.)

   `copy of a` of a class with no `Clone` (E2116, `V4/rules.md` section 3's message; its "or keep a view
   of it" is left out here, since `b.health = 5` cannot change a view): "`Player` has no `Clone`, so
   `copy of` cannot make one; write `@derive(Clone)` above `class Player`". `c.health = 5` through the
   reading owner (E3023): "`c` only reads the player it shares; to change it through `c`, write
   `c = shares mut a` in place of line 10 and `fn main() changes Player:` in place of line 5".
7. **Cost:** the copy where `copy of` is written (one block and the fields), as C's `malloc` + copy and
   Rust's `.clone()` of a `Box`; `shares a` is a count +1 after one compare with its maximum (R9,
   verdict 59), as Rust's `Rc::clone`.
8. **Kind:** a readable spelling of Rust's guarantee.

#### AN-46: silent copies no word names: `comptime(e)` of a heap value; `sorted(m.keys())`

**v4:** **Meaning changed:** no. No verdict of 53 to 82 changes a NEW line (no line here changes `m`
while a loop walks it; verdict 72 would refuse that at the change, E3020). E3060's first way out now
names the line it moves (verdict 73).

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
   own = copy of comptime(table())     # your own list, which may change
   own.push(4)

   m: Map[String, int] = Map()
   for k in sorted(m.keys()):          # views of m's own keys, sorted: no copy
       println(k)
   names = sorted(m.keys().copy_each())   # your own strings, kept past changes to m
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `total`, `m` | none (default, own local) | none (default, own local) |
   | `i` | none (default, reads only: a number from the range) | none (default, reads only) |
   | `t` | none (default, reads only: a value made when compiling is read-only data; the name is a view of it, never a copy) | none (default: it never changes) |
   | `own` | `copy of`: its own list | none (default, own local) |
   | `k` | none (default, reads only: a loop name; here a view of one of `m`'s keys) | none (default, reads only) |
   | `names` | `.copy_each()`: its own strings, each a copy of a key (the `.copy_each()` row of `V4/vocabulary.md`, verdict 33); `sorted` makes the list, which the name owns (R2) | none (default, own local) |

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
     = help: for your own list write `t = copy of comptime(table())`
   ```

   The sorted keys kept as a list and used after a change to `m`:
   `ks = sorted(m.keys())`, then `m.insert(String("z"), 1)`, then `println(ks[0])`:

   ```text
   error[E3060]: `m.insert(String("z"), 1)` (line 8) changes `m`, whose keys `ks` (line 7) still views on line 9
     = help: use `ks` before the change: move `println(ks[0])` (line 9) above line 8; or keep your own strings: write `ks = sorted(m.keys().copy_each())` in place of line 7
   ```

   Today's name of the adapter, `names = sorted(m.keys().copied())` (E1010; the `.copied()` row of
   `V4/vocabulary.md` and the removed-words row of `V4/rules.md` section 3):

   ```text
   error[E1010]: `.copied()` is now `.copy_each()`: it says what it copies, each item
     = help: write `names = sorted(m.keys().copy_each())`
   ```

7. **Cost:** less than today: no allocation per evaluation of the table (read-only data, as C's
   `static const`), and no string copy per key (an array of pointers, as the C above and Rust's
   `let mut ks: Vec<&String> = m.keys().collect(); ks.sort();`). Copies only where `copy of` or
   `.copy_each()` is written.
8. **Kind:** a smarter compile-time solution (the value is placed in read-only data and lent; the
   sorted keys are a local list of views, R7).

#### AN-47: `owned` in an expression: E0109 contradicts Annex A, and its help is wrong for `b = owned a`

**v4:** **Meaning changed:** no; `job`'s line changes (verdict 55). `job` takes `report` over and, when it
runs, gives it to `submit`, which ends it: a function that may end a class object of its own state names
the object's type, so that every call of it is seen to end a `Report` (R12, R10 point 1), and `job` is
now `fn() with owns report changes Report: submit(report)` (E2105 without the word). It is still a
`once fn()` that a call uses up (R12). `task` captures nothing and stays a plain `fn()`. The help
`job = fn(): submit(copy of report)` uses the real `report`, only reads it, and ends only the copy it
makes for `submit` on that line, which nothing else can point at, so it needs no word. The E0109 and
E3013 messages give whole lines (verdict 73).

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
   @derive(Clone)
   class Report:
       lines: Array[String] = []

   fn submit(owns r: Report):
       println(r.lines.len())

   fn main():
       a = [1, 2]
       b = a                                         # hands the list over
       task = fn() => println("done")                # captures nothing
       report = Report()
       job = fn() with owns report changes Report: submit(report)   # takes `report` over; when it runs, `submit` ends it
       job()                                         # once: `job` gave `report` away
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `lines` | none (default: a field holds its list) | through the name before it |
   | `r` | `owns` on the parameter: handed over to `submit` | none (default, own local) |
   | `a`, `report` (the local) | none (default, own local: the value made on this line) | none (default, own local) |
   | `b` | none (default, row 9: `=` hands over) | none (default, own local) |
   | `task` | none (default, own local: the function made on this line; it captures nothing) | none (default: a plain `fn()` changes nothing at all, R12, verdict 22) |
   | `job` | none (default, own local: the function made on this line); `owns report` in its capture list says it gives `report` away, so a call uses it up: a `once fn()` (R12) | `changes Report`: a call of it may end a `Report`, the one its own state holds (R12, verdict 55); nothing else |
   | `report` in the capture list | `owns`: handed over to the function (R3, R12) | none (default: the function's own) |

5. **Means exactly:** `an47_tr.em` (`b = a`; `task = fn() => println("done")`): `exit=0`, no output.
6. **Still refused** (E0109, R3; Annex A's `task = owned fn() => ...` is rewritten). The OLD lines as
   written use `owned`, which is not a word of v4 (E1010: "`owned` is now `owns`"); written with `owns`,
   they are refused as follows (`V4/rules.md` section 3's E0109 messages, in this program's names,
   `b = owns a` on line 10 and `task = owns fn() => println("done")` on line 11):

   ```text
   error[E0109]: `owns a` is not needed: `b = a` already hands `a` over
     = help: remove `owns`: write `b = a` in place of line 10; for a copy, `b = copy of a` there; for a second owner that reads it, `b = shares a` there
   ```

   ```text
   error[E0109]: `owns fn` does not say what the function takes or copies
     = help: `task` captures nothing, so write `task = fn() => println("done")` in place of line 11; a function that takes a name over says so on that name, as `job = fn() with owns report changes Report: submit(report)` does, and a name not listed is the real variable
   ```

   A second `job()` (R12, E3013; the rules' message in this program's names):

   ```text
   error[E3013]: `job` gives `report` away when it runs (`with owns report`), so it can run only once, and line 15 runs it again (a second `job()` below line 14)
     = help: to run it more than once, hand over a copy each time: write `job = fn(): submit(copy of report)` in place of line 13
   ```

   `job` without `changes Report` (R12, E2105; the rules' message for a function that ends class objects
   of its own state, in this program's names):

   ```text
   error[E2105]: this function ends the report it owns (`submit(report)` ends it); say so on its first line
     = help: write `job = fn() with owns report changes Report: submit(report)` in place of line 13
   ```

7. **Cost:** nothing: the capture moves `report`'s pointer into `job`'s state, as Rust's `move ||`
   closure; `once` and `changes Report` are checked when compiling.
8. **Kind:** a readable spelling of Rust's guarantee (a message that gives the line; Rust's `FnOnce`,
   said by the capture word).

#### AN-48: shared mutable state: a camera and a HUD both change one player

**v4:** **Meaning changed:** no; three first lines change (verdicts 63, 64). In (a), `main` changes the
player through `cam.target` and `hud.target`, `shares mut` links in its own objects, and `transfer`
changes it through `b`, a `ref mut` parameter that holds a link; in (b), `main` changes it through the
`sharedref mut` links of its own camera and HUD. A change through a link is named whatever holds the
link (R5), so the three lines say `changes Player`: `fn main() changes Player:` in both forms and
`fn transfer(a: shares Player, ref mut b: shares mut Player) changes Player:`, verdict 63's own shape.
`shake` changes the player through its own field `target`, which `changes target` already covers
("anything reached through" the field, R5). Verdicts 53, 54 (R4's check of a call against what the
callee's body does, E3031): `transfer(cam.target, ref mut hud.target)` is allowed, since `transfer`
changes only the player `b` leads to, and only inside `if a is not b:`, so its body never uses `a`
after a change of `a`'s player (For the owner, below, asks to confirm that the body's fact is this
precise). The messages give whole lines (verdict 73); counts are compared before each +1 (verdict 59).

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
   # (a) the camera and the HUD each keep the player alive, and both change it: owners, counted
   class Player:
       hp: int = 100
       inventory: Array[String] = []
       log: Array[String] = []

   class Camera:
       target: shares mut Player

       fn shake() changes target:
           this.target.hp -= 1

   class Hud:
       target: shares mut Player

   fn main() changes Player:                      # it changes the player through links (R5)
       p = Player()
       cam = Camera(target=shares mut p)
       hud = Hud(target=shares mut p)
       cam.shake()
       hud.target.inventory.push(String("key"))
       for item in hud.target.inventory:
           cam.target.hp -= item.len()            # a number: accepted
       for item in hud.target.inventory:
           cam.target.log.push(copy of item)      # another list of the player: accepted
       transfer(cam.target, ref mut hud.target)   # the call says `ref mut`, as the parameter does

   fn transfer(a: shares Player, ref mut b: shares mut Player) changes Player:
       if a is not b:
           for it in a.inventory:
               b.inventory.push(copy of it)       # accepted: the compiler knows they differ
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

   fn main() changes Player:                      # it changes the player through links (R5)
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
   | `target` (form (a), both classes) | `shares`: one of several owners of a player | `mut` in `shares mut`: the player may be changed through it (R9); `changes target` on `shake` |
   | `target` (form (b), both structs) | `sharedref`: a link to a player someone else owns, which other holders may also hold | `mut` in `sharedref mut` (R7) |
   | `this` (in `shake`) | `this`: the camera the method was called on | `changes target` on the first line |
   | `p` | none (default, own local: the player made on this line; in (a), after `shares mut p`, the first of its owners, R3) | none (default, own local: in (a) it keeps what it could do, R3, R9, verdict 23; in (b) the owner, which may change it beside the links, R7) |
   | `cam`, `hud` | none (default, own local: the object made on this line; in (b) a view of `p` by its field's word, R7) | none (default, own local); a change through their `target` links is named on `main`'s first line, `changes Player` (R5, verdict 64) |
   | `item`, `it` | none (default, reads only: a loop name for each element) | none (default, reads only) |
   | `a` | no word on the name: a view of the caller's player, with no count; `shares` in the type says only that the caller's player is shared (R4, verdict 32) | none (default, reads only: the word on the name, here none, sets what `transfer` may do, R4; it promises the player does not change while `a` is used, R8 (g), which `if a is not b:` keeps) |
   | `b` | `ref mut` on the name, `shares mut` in the type: a name for the caller's link `hud.target` and the player it leads to | `ref mut`: yes; by R4 it names the caller's place, so it covers both the player (`b.inventory.push(...)`, which `mut` in `shares mut` allows, named on the first line as `changes Player`, R5, verdict 63) and the caller's link (`b = shares mut q` would point `hud.target` at `q`) |

5. **Means exactly:** `an48_tr_class.em` (today's class form of the accepted lines of (a), with the
   refused loop rewritten as its message says: collect, then push after the loop): `exit=0`, no output.
   `an48_tr_is_not.em` (`transfer` with `if a is not b:`): `exit=0`, no output. Today both carry
   run-time access words on `inventory` and `log` (`[EXC-19]`); the NEW model has none (R9). Form (b)
   cannot be said in today's Ember (the OLD struct form is today's E3022); in C it is a plain pointer in
   each struct and no count, the shape of run 3's `bench/share_c.c` `-DMODEL=8`
   (`RUN3/combined-v4.md` line 7278).
6. **Still refused:**

   ```text
   error[E3020]: `cam.target.inventory.push(copy of item)` (line 24) may add to the list this loop walks (`hud.target.inventory`, line 23)
     = note: `cam.target` and `hud.target` may be the same `Player`, and adding can move the list's elements while the loop reads them
     = help: add after the loop: write `new_items: Array[String] = []` before the loop, `new_items.push(copy of item)` in place of line 24, and `cam.target.inventory.extend(new_items)` after the loop; or walk a list of your own: `for item in copy of hud.target.inventory:`
   ```

   The same for a call: a method whose first line says `changes target` (it may reach the player's
   inventory) called inside that loop: "`cam.shake()` says `changes target`, which may be the `Player`
   whose inventory this loop walks; write `cam.shake()` below the loop instead of in it" (E3020). `transfer` without the `is not`
   test (lines 28 and 29 its `for` and its push): "`a` and `b` may be the same player, and this loop
   walks `a.inventory` while adding to `b.inventory`; test first: write `if a is not b:` above line 28
   and indent lines 28 and 29 under it" (E3020). `transfer(cam.target, hud.target)` without the word:
   "`transfer` may change `b` (its first line says `ref mut b: shares mut Player`); say so at the call:
   write `transfer(cam.target, ref mut hud.target)` in place of line 25" (E3028). With `target: shares
   Player` (owners that only read), `this.target.hp -= 1` in `shake` is refused (E3023: "`this.target`
   only reads the player it shares; to change it through the camera write `target: shares mut Player`
   in place of line 7 and `cam = Camera(target=shares mut p)` in place of line 17"). A `shares` value
   sent to another thread: area E's E7015. (Lines are counted from `class Player`, line 1, as in the
   E3020 message above.)

   `main` without `changes Player` (R5, E2105; the rules' row for a change through a link, in this
   program's names): "`main` changes the `Player` that its camera's and HUD's links name
   (`hud.target.inventory.push(String("key"))`, line 20), and a change through a link is named by the
   type it reaches; write `fn main() changes Player:` in place of line 15". The same for `transfer`:
   "`transfer` changes the `Player` that `b` links to (`b.inventory.push(copy of it)`, line 30); write
   `fn transfer(a: shares Player, ref mut b: shares mut Player) changes Player:` in place of line 27".

   The OLD struct form, two kept `ref mut` fields of one player (R8 (f); the E3022 row of
   `V4/rules.md` section 3, in this program's lines):

   ```text
   error[E3022]: `p` is lent to `cam.target`, a `ref mut` field, on line 15 until its last use on line 17, and line 16 lends it to `hud.target` too
     = note: a `ref mut` name is the only name that may touch its place until it is done
     = help: if both may change the player, write `target: sharedref mut Player` in `Camera` and in `Hud`, `cam = Camera(target=sharedref mut p)` in place of line 15 and `hud = Hud(target=sharedref mut p)` in place of line 16, and `fn main() changes Player:` in place of `fn main():`, since line 17 then changes the player through a link
   ```

   In (b), a function that makes `p` and returns `cam`: "`cam` holds a link to `p`, which ends when
   this function returns, so `cam` may not outlive it (returned here); let the camera own a share that
   may change the player: write `target: shares mut Player` in `Camera`, and
   `cam = Camera(target=shares mut p)` in place of `cam = Camera(target=sharedref mut p)`" (E3063).
7. **Cost:** nothing checked while running. (a): counts +1/-1 per `shares mut` owner made or ended, each
   +1 after one compare with the count's maximum (R9, verdict 59); for the views the loops make
   (`hud.target.inventory`), no count (R10 point 1, checked when compiling: nothing in the loop may end a
   `Player`). `changes Player` costs nothing. (b): nothing at all, a pointer in each struct; in run 3's watcher workload this
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

**v4:** **Meaning changed:** no. No verdict of 53 to 82 changes a NEW line: `add` and `bump` change their
own object's fields, not through a link. The E1010 help's shared scene is changed through `shares mut`
links, so it now says that a function changing the scene through one names it, `changes Scene`
(verdicts 63, 64), and the E3060 message names the line it moves (verdict 73).

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

   A scene several objects own and change is `scene: shares mut Scene`, changed through any owner that
   says `shares mut` (R9, AN-48).
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
     = help: write the field plainly, `entities: Array[int]`, and the method's first line as `fn add(e: int) changes entities:`; for a scene several objects own and change, make it once, `s = Scene(entities=[])`, write the field that keeps it in each owner as `scene: shares mut Scene`, and give each owner its share where it is made, for example `Panel(scene=shares mut s)`; a function that changes the scene through such a field says so on its first line, `changes Scene` (a method changing it through its own field says `changes scene`)
   ```

   ```text
   error[E2105]: `bump` changes `this.n`, but its first line does not say so
     = help: write `fn bump() changes n:`
   ```

   The conflict `an49_old_refcell_conflict.em` panics on becomes a compile-time refusal (R8 (b)): a
   view of `s.entities` (`a = s.entities`) still used while `s.add(...)` changes it: "`s.add(4)`
   (line 7) changes `s.entities` (`add` says `changes entities`), which `a` views (line 6) and uses on
   line 8; use `a` before the change: move line 8 above line 7; or keep a copy: write `a = copy of
   s.entities` in place of line 6" (E3060).
7. **Cost:** less than today: no borrow flag (one word per `RefCell`, checked on every borrow in every
   profile, `[CELL-5]`, `[CELL-9]`); nothing runs.
8. **Kind:** a smarter compile-time solution (Rust's run-time-checked `RefCell` replaced by R8's
   compile-time rule).

#### AN-50: "handle" names a class object's counted pointer and also `Handle[T]`

**v4:** **Meaning changed:** no. Verdict 68 says in so many words that `copy of` works on every `Copy`
type, an `int` included, which `first = copy of o.bag.items[0]` relies on. The E3060 message names the
line it moves (verdict 73); the E2103 message is the E2103 row of `V4/rules.md` section 3, word for
word.

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
   the written words `JoinHandle` and `@ffi(handle)` (the `Handle` row of `V4/vocabulary.md`). A class
   object is "the `Player` object `p`", a link in a field "the link `o.bag`". The program the first
   message is about, as the new message makes it:

   ```ember
   first = copy of o.bag.items[0]      # or use `first` before replacing the bag
   o.bag = Bag()
   println(first)
   ```

4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `first` | `copy of`: its own number (for an `int`, plain `=` copies too, R2) | none (default, own local) |
   | `o` | none (default, own local: the object made on its line) | none (default, own local) |
   | `bag` | none (default: a field holds its `Bag` object alone, R3) | through the name before it |
   | `items` | none (default: a field holds its list) | through the name before it |

5. **Means exactly:** `an50_tr_use_before.em` (`first = o.bag.items[0]`, an `int` copied, then the bag
   replaced): `exit=0`, no output.
6. **Still refused**, with the new wording:

   ```text
   error[E3060]: `o.bag = Bag()` (line 11) replaces the `Bag` object that `first` (line 10) points into, and `first` is used on line 12
     = help: use `first` before the bag is replaced: move `println(first)` (line 12) above line 11; or keep your own copy: write `first = copy of o.bag.items[0]` in place of line 10
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

**v4:** **Meaning changed: yes, for part 6's other programs** (verdict 82); `step` itself means the same
and gains a word (verdict 63). `t.hp -= 1` and `t.inventory.push(frame)` change the player through the
`shares mut` link that `ws`, a `ref mut` parameter, reaches, so `step` says `changes Player` (R5:
a change through a link is named whatever holds it). R10 is now checked when compiling only: in v3 a
line in `t`'s life that could end a `Player` was allowed and paid for (a deferred-drop region, or a
count pair around `t`'s life); now such a line is allowed only if `t` is not used after it, and a use
after it is refused (E3060). `t = ref mut ws[i].target` is still R10's own example of a view made
through a `shares mut` field.

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
       target: shares mut Player

   fn step(ref mut ws: Array[Watcher], frame: i32) changes Player -> u64:    # called as `step(ref mut ws, frame)`
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
   plain name never changes anything, an object behind a `shares` link included), the link says
   `shares mut` (R9: only an owner that may change it changes the player), and the first line says
   `changes Player` (R5: a change through a link is named, even when the link came in through a `ref mut`
   parameter; verdict 63). The list and its links are not changed, and the C is the same.
4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `hp`, `x`, `inventory` | none (default: a field holds its value) | through the name before them |
   | `target` | `shares`: one of several owners of a player | `mut` in `shares mut`: the player may be changed through it, here through `t` |
   | `ws` | `ref mut`: the caller's list of watchers | `ref mut`: yes (the players its watchers link to are changed through it, which the first line names, `changes Player`) |
   | `frame` | none (default: a number parameter, copied in) | none (default, reads only) |
   | `acc` | none (default, own local: a number) | none (default, own local) |
   | `i`, `v` | none (default, reads only: a loop name for each value) | none (default, reads only) |
   | `t` | `ref mut`: a name for the link in `ws[i].target`, the only one while used (R8 (f)), through which the player is changed | `mut` in `ref mut` |

5. **Means exactly:** in C, run 3's `bench/share_c.c` `-DMODEL=3` (`Player *t = ws[i].target;` with no
   retain or release around the step; counts change only at re-targeting), whose checksum equals plain
   C's (`80363660820`, `RUN3/combined-v4.md` line 5754).
6. **Still refused:** nothing in `step` is refused. R10 point 1, checked when compiling: nothing in
   `t`'s life may end a `Player`, so nothing runs for `t`. A line in `t`'s life that could end one (a
   write to any place that owns a `Player`, such as `ws[j].target = shares mut q`; the end of a local
   owner of one; a call whose first line, summary, arguments or function type say it may do so) is
   allowed only where `t` is not used after it; a use after it is refused (E3060, with the message of
   the E3060 row of `V4/rules.md` section 3 for `reset()`: use `t` before that line, or name the player
   again after it, `t = ref mut ws[i].target`). This is R10 as verdict 82 rewrote it: the player is no
   longer kept alive across such a line by a deferred-drop region or a count pair. A write to
   `ws[i].target` itself while `t` is used is refused before that, by R8 (f) (E3022: `t` is the only
   name for that place until its last use). And while `t` is used, `ws[i].target` is not read through
   another name either (R8 (f), E3021): a line `println(ws[i].target.hp)` placed after `t.hp -= 1`
   gets "`t` is the only name for `ws[i].target` until its last use, and this line reads
   `ws[i].target.hp`; read it through `t`: write `println(t.hp)` in place of this line" (E3021).
7. **Cost:** 0 count operations per step where today makes 2 (a load-add-store each); the 13.5% to
   13.9% run 2 measured goes (`phase3b-ownership.md` section 4). `changes Player` costs nothing. Since
   verdict 82 no program pays a region or a count pair for a view like `t`: one that would have needed
   one is refused instead.
8. **Kind:** a smarter compile-time solution (`[RC-3]`'s removal, decided from first lines, `[RC-2b]`'s
   one-expression limit lifted).

#### AN-52: shared class objects re-pointed often ran at 1.16x to 1.17x plain C (last run's limit)

**v4:** **Meaning changed:** no. Verdict 81 is the rule (b) relied on: adding to a list does not move
what lives in its own block (a class object, or an `owns` element), so a `sharedref mut` link to a
player is not ended when `players` grows; a link to a value stored inline in the list would be (R8 (b),
section 0). The messages of part 6 now give whole lines that compile (verdict 73; `V4/safety/check-V3.md`
on AN-52's E3060): their way out of keeping the players alive takes `@view` off `Watcher` (a struct
whose field owns is no view, E2030 otherwise), declares `players` whole, makes each player with
`shares mut`, and names `changes Player` on the functions that change a player through a watcher
(verdict 64). Counts are compared before each +1 (verdict 59).

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
   # (a) the watcher keeps its player alive, and changes it: counted, as Rust's Rc
   struct Watcher:
       target: shares mut Player
   players: Array[shares mut Player] = ...        # the list is one of each player's owners
   ws[i].target = shares mut players[r]          # re-point: a count -1 on the old player, +1 on the new

   # (b) the watcher follows a player that `players` owns and that outlives the watchers;
   #     several watchers may follow, and change, one player: a kept link, no count
   @view
   struct Watcher:
       target: sharedref mut Player
   players: Array[owns Player] = ...              # `struct Player`, each in its own block (R3;
                                                  # a `class Player` list is plain `Array[Player]`)
   ws: Array[Watcher] = ...                       # local: lives while `players` does
   ws[i] = Watcher(target=sharedref mut players[r])   # re-point: one store

   # (c) a position in the owning list
   struct Watcher:
       target: int
   players[ws[i].target].hp -= 1
   ```

   (b)'s rules, all checked when compiling (R7, R8): `ws` may not outlive `players` (a list of views is
   a view, E3063); while `ws` is used, `players` may grow (its elements are `owns`, each in its own
   block, so growing moves only the list of pointers: R8 (b), verdict 81) but nothing may be removed
   from it or replace it (E3060); several watchers, and `players` itself, may change one player at any time (plain fields
   change freely; a view into a player's `inventory` ends at a reshape of that list through any of
   them, R8 (b)). `ref mut` parameters and names stay alone, so the C compiler is promised that two
   `ref mut` parameters do not overlap (R4); `sharedref mut` links get no such promise.
4. **The owner's test:**

   | name | what it is | may it change it |
   |---|---|---|
   | `target` (a) | `shares`: one of several owners | `mut` in `shares mut`: the player may be changed through it (R9) |
   | `target` (b) | `sharedref`: a link to a player `players` owns, which other watchers may also hold | `mut` in `sharedref mut` |
   | `target` (c) | none (default: a number, the player's position) | none (default: changed through the watcher's owner) |
   | `players` | (a) `shares mut` elements: the list is one of each player's owners, and may change it; (b) `owns` elements: each player held alone by the list | none (default, own local) |
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
     = help: remove it after the last use of `ws`: move line 29 below line 30; or let the watchers keep their players alive: write `struct Watcher:` without `@view`, with `target: shares mut Player`; declare the list `players: Array[shares mut Player] = []` and add each player as `players.push(shares mut Player(...))`; make each watcher `Watcher(target=shares mut players[r])`; and write `changes Player` on the first line of every function that changes a player through a watcher
   ```

   ```text
   error[E3063]: `ws` holds views of `players`, so it may not outlive it (returned here)
     = help: let the watchers keep their players alive: write `struct Watcher:` without `@view`, with `target: shares mut Player`; declare the list `players: Array[shares mut Player] = []` and add each player as `players.push(shares mut Player(...))`; make each watcher `Watcher(target=shares mut players[r])`; and write `changes Player` on the first line of every function that changes a player through a watcher; or keep positions: `target: int` in `Watcher`, `Watcher(target=r)` where a watcher is made, and return `players` beside `ws`
   ```

   (`Player(...)` stands for the program's own arguments, which the message copies from the line that
   makes each player.)

   Run 3's spelling of (b), `target: ref mut Player` with `Watcher(target=ref mut players[r])`
   (R8 (f)):

   ```text
   error[E3022]: `ws[i].target` is a `ref mut` field, the only name for `players[r]` while `ws` is used, and other watchers in `ws` may name the same player
     = help: if several watchers may change one player, write `target: sharedref mut Player` in `Watcher` and `Watcher(target=sharedref mut players[r])` where a watcher is made, and `changes Player` on the first line of every function that changes a player through a watcher
   ```

   `sharedref` without `mut` (E0114, R7): "a link that only reads may be held by any number of names:
   write `target: ref Player`; `sharedref` is for a link that may change its target:
   `target: sharedref mut Player`".
7. **Cost** (run 3's section 5, three batches, PROVISIONAL; `RUN3/combined-v4.md` lines 5756-5763):
   (a) 1.13x to 1.25x of plain C, against 1.48x to 1.57x for safe Rust's `Rc<Player>` with `Cell`;
   (b) 1.00x to 1.06x, the same memory as plain C (53,284 KB); (c) 0.80x to 0.85x and 8% less memory
   (49,188 KB), against 0.86x to 0.90x for safe Rust's `Vec` and positions. (b)'s C is unchanged by the
   new word, so its numbers stand (`V4/rules.md` section 2, the `sharedref mut` row). (a)'s `mut` costs
   nothing: it is checked when compiling. (a)'s counts now add one compare with the count's maximum
   before each +1 (R9, verdict 59), on the re-pointing path those numbers measured; not measured.
8. **Kind:** a new mechanism (`sharedref mut`: kept links several holders may change, checked by R8's
   compile-time rule; Rust allows only one `&mut`), with (a)'s counting kept as the understandable cost
   of "keeps alive", which Rust pays more for.

## Answers changed in v4

| answer | verdict | meaning changed | how |
|---|---|---|---|
| AN-51 | 82 (and 63) | **yes**, for part 6's other programs | R10 no longer keeps `t`'s player alive with a deferred-drop region or a count pair across a line that may end a `Player`: such a line is allowed only where `t` is not used after it, and a use after it is refused (E3060). `step` itself means the same, and says `changes Player` (verdict 63) |
| AN-34 | 63, 64, 73 | no | `two_owners` and `owner_that_reads` say `changes Array[int]`; E3040's second way out gives that first line, E3023's names its line |
| AN-35 | 78, 73 | no | part 6 gains E3040 for a field handed over on its own; `struct Player` shown with `@derive(Clone)`; E3060's and E3013's helps give whole lines |
| AN-36 | 64, 73 | no | written as whole functions; the second form's `main` says `changes Node`; E3040's and E3023's helps give whole lines, the first line included |
| AN-37 | 73 | no | E3013's help gives whole lines that compile |
| AN-39 | 63, 64, 74, 73 | no | `build` says `changes Mesh`; the table of kinds gains `owns` on a making line and the naming of a change through a link; E3040 is shown on its program, with whole lines |
| AN-40 | 74, 73 | no | `e = Entity(mesh=owns Mesh(verts=[0.0]))`; E2123's message; the E2020 example has one fault; the `Box` message names the making line |
| AN-41 | 59 | no | the weak count's compare in the cost |
| AN-42 | 72 | no | the loop over a map's values cited as verdict 72's; the failed cast's stop text |
| AN-43 | 73 | no | E3040's helps give whole lines (the safety check's fix) |
| AN-44 | 64, 68, 73 | no | written as `main`'s body, `fn main() changes Player:`; `copy of` on a `Copy`-only `Vec2` settled; E3040's second way out drops `changes Player`; whole lines |
| AN-45 | 64, 73 | no | written as `main`'s body; the ways out that share the player give `fn main() changes Player:`; E2116 without the view it cannot use |
| AN-46 | 73 | no | E3060's first way out names its line |
| AN-47 | 55, 73 | no | `job = fn() with owns report changes Report: submit(report)`; E2105's message for it; E0109's and E3013's helps give whole lines |
| AN-48 | 63, 64, 53, 73 | no | `main` (both forms) and `transfer` say `changes Player`; `transfer(cam.target, ref mut hud.target)` checked against E3031 and allowed; whole lines in every message |
| AN-49 | 64, 73 | no | E1010's help says how a shared scene's change is named; E3060 names its line |
| AN-50 | 68, 73 | no | `copy of` an `int` cited as verdict 68's; E3060 names its line |
| AN-52 | 81, 64, 73 | no | `players` may grow because of verdict 81's rule; E3060's, E3063's and E3022's helps give whole lines that compile (no `@view`, `players` declared, players made `shares mut`, `changes Player`) |
| AN-38 | — | no | a "v4" line only |
| every answer | — | — | a "v4" line in place of the "v3" line; `V4` rules, words and messages cited; counts compared before each +1 where an answer counts (verdict 59) |

## Words needed

None. Every word the NEW blocks and messages use has a row in `V4/vocabulary.md` (among them `owns` on a
making line, verdict 74; `changes T` for a change through a link, verdicts 63, 64; `changes Report` for a
function value that ends a class object of its own state, verdict 55). The names `Report`, `submit`,
`LevelArena`, `current`, `load_level`, `arena`, `Pool`, `AnyPool`, `IntPool`, `Reg`, `values`,
`emplace`, `get_mut` (on `Reg`), `Watcher`, `Camera`, `Hud`, `Player`, `Mesh`, `Entity`, `Registry`,
`App`, `Level`, `Tree`, `Node`, `Scene`, `Counter`, `Npc`, `State`, `Bag`, `P`, `table`, `two_owners`,
`owner_that_reads`, `hand_over` and `file_and_keep` are the programs' own (run 3's and run 4's; the
owner's `LevelArena`, verdict 19, and `current`, verdict 24's own example).

## The agreed rows

| row | where | as agreed |
|---|---|---|
| 3 | the methods' first lines (AN-37, AN-38, AN-41, AN-42, AN-48, AN-49) use row 3's form, `fn ...(params) changes field:`, without `self` | yes: the form is `fn note(msg: String) changes log:` as `inputs/JOB.md` writes it (verdict 6 keeps the field form beside `changes this`); row 3 itself is not reproduced here |
| 8 | AN-37, AN-38: `State.Walking(ref mut path):`, and the two messages ("to change that list through `path`, write `State.Walking(ref mut path):`", for plain `path` and for `mut path`) | yes, word for word |
| 9 | AN-34: `snapshot = copy of items` for a copy, `snapshot = items` hands the list over, the error names both ways out (`copy of`, `shares`); AN-44: `snapshot = items` | yes, in the new spelling (verdicts 12, 19); the message's `shares` way out now also gives the first line it needs, `fn main() changes Array[int]:` (verdicts 64, 73; For the owner, item 1) |

Rows 4, 5, 6 and 7 are not used in this area (row 6, `b.on_click(fn() changes score: score += 1)`, is
area D's AN-53).

## Notes for the lead and the rule writer (not for the owner)

1. **E3063's AN-52 row in `V4/rules.md` section 3** still gives v3's way out ("write `target: shares mut
   Player` in `Watcher`, `players: Array[shares mut Player]`, and `Watcher(target=shares mut
   players[r])`"). In AN-52's program that line runs into E2030 (`@view` on a struct that is no longer a
   view), leaves `players` without a whole declaration (`V4/safety/check-V3.md`), and leaves the
   functions that change a player through a watcher without `changes Player` (E2105, verdict 64). AN-52's
   message now gives the whole lines; the row could follow it.

2. **Row 9's E3040 row in `V4/rules.md` section 3** ("... or `snapshot = shares items` (two owners of one
   list)") gives no first line; in row 9's program `items.push(4)` then changes the list through a link,
   which verdict 64 names (`changes Array[int]`). AN-34's message adds it, under the reading of For the
   owner, item 1.

3. **E3023 for a change through a `shares` owner** (v3's note 2). The `shares` row of
   `V4/vocabulary.md` gives it E3023, and this file uses it in AN-34, AN-36, AN-39, AN-44, AN-45 and
   AN-48, but `V4/rules.md` section 3 still has no row for it, and section 4's E3023 meaning does not
   name it. The meaning fits E3023's ("a change the name does not allow"); the row could name it.

4. **E3022 for a change of the place a `ref mut` pattern name views** (AN-37, v3's note 3). R8 (f) now
   says that a removal, reorder or clear of a list ends a position and a later use is E3060, not E3022;
   AN-37's `path` is not a position but a pattern name of an enum's payload, so R8 (f)'s E3022 stands
   for `this.state = State.Idle` before `path.pop()`, as this file writes it.

5. **A second call of a `once fn()` local** (AN-47, v3's note 5). The E3013 row of `V4/rules.md` section
   3 gives the message for passing such a function where it may run twice; AN-47's message, for calling
   a local twice, is written in the same words.

6. **The end of a value whose type names a C destroy function** (verdict 31; v3's note 7). R12 still
   speaks of "a call of a foreign function", which a reader may not take to cover a call the compiler
   inserts at a value's end (AN-40's `App.db`, `@ffi(destroys(sqlite3_close))`). Nothing in area C
   depends on it: its programs hand C no callback.

v3's notes 1, 4 and 6 are settled: notes 1 and 6 by the lead's phase 3 of step 2, note 4 by verdict 74.
v3's question for the owner (one word in every arena-made value) is answered by verdict 47 ("accepted
for now"; AN-40's cost says it).

## For the owner

Questions the verdicts leave open for lines of this area; each answer is written meanwhile as it says.

1. **Is the first owner of a shared value a link once it has shared it?** Verdict 64 names a change
   through a link held in a function's own local (`fn main() changes Clock:` for `clock = shares mut
   Clock()`). In AN-34's `owner_that_reads` (`items = [1, 2, 3]`, `other = shares items`,
   `items.push(4)`) and AN-36's second form (`root.children.push(shares mut child)`, then `child.value =
   5`), the change goes through the name the value was first shared from. The rules' section 0 calls a
   link any name of an object it does not own alone, `shares` owners included, so this file reads it as
   a link and writes `fn owner_that_reads() changes Array[int]:` and `fn main() changes Node:`; and row
   9's message, whose `shares` way out then pushes through `items`, gives `fn main() changes
   Array[int]:` too. Is that the intended reach? (If not, `owner_that_reads` says nothing, and row 9's
   message stays as agreed.)
2. **A method that changes an object through a link in its own field: is `changes target` enough?**
   AN-48's `fn shake() changes target:` with `this.target.hp -= 1`, `target: shares mut Player`. R5 says
   `changes target` covers "anything reached through" the field, so this file keeps it; verdict 64's "a
   change through a link always says `changes T`, whatever holds the link" could be read to ask for
   `fn shake() changes target, Player:`. Which is meant?
3. **`owns` on a making line, where the `owns` is inside another type.** Verdict 74 requires `owns` on
   the line that makes a value for an `owns` field (`Entity(mesh=owns Mesh(verts=[0.0]))`, written so in
   AN-40). AN-41's `left: Option[owns Tree]`, made as `Tree(value=1, left=Some(Tree(value=2)))`, and
   AN-40's `pools: Array[owns dyn Pool]`, filled with `r.pools.push(IntPool(items=[1, 2]))`, are not
   `owns` fields themselves; the rules say a plain value stored into an `owns T` place is put in its
   block at that store, so this file leaves them without the word. Should they say it too
   (`left=Some(owns Tree(value=2))`, `r.pools.push(owns IntPool(items=[1, 2]))`)?
4. **How precise is the fact verdict 53 works out from a body?** AN-48's
   `fn transfer(a: shares Player, ref mut b: shares mut Player) changes Player:` changes only `b`'s
   player, and only inside `if a is not b:`, then reads `a` in the loop. This file reads the fact as
   "uses `a` after changing `a`'s player: never", so `transfer(cam.target, ref mut hud.target)`, where
   the two may be one player, is allowed. A fact kept by type alone ("changes a `Player`, then uses the
   `Player` parameter `a`") would refuse that call (E3031). Is the precise fact the one meant?
