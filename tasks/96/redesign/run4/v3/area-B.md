# Area B, v3: references kept or returned, locks, objects still being built (AN-17 .. AN-33)

Agent 03 (area-B), run 4, step 2 (v3). Every answer of area B brought to the v3 rules and words, which
apply all of the owner's verdicts 1 to 34 (`inputs/JOB.md`, "Owner verdicts on run 4's open questions";
27 replaces 25): `V3/rules.md` (rules R1 to R18, the codes), `V3/vocabulary.md` (the words) and
`V3/changes.md` (what changed from v2). `RUN3` means `tasks/96/redesign/run3/`, `RUN4` means
`tasks/96/redesign/run4/`, `V2` means `tasks/96/redesign/run4/v2/`, `V3` means
`tasks/96/redesign/run4/v3/`; "R1" .. "R18" are the v3 rules; "row N" is the owner's agreed row N and
"verdict N" the owner's verdict N in `inputs/JOB.md`.

No compiler change, no git command, no Ember program run, and no `ember check` was needed: this file
makes no new claim about what today's Ember accepts or refuses. Every OLD block and every "means
exactly" block, with its `ember check` output, is carried over unchanged from `V2/area-B.md` (which
carried them from `RUN4/area-B.md` and `RUN3/combined-v4.md` section 11): this file is phase 0's copy
of `V2/area-B.md` (phase 0 changed no line of it: `V3/spelling-changes.txt` lists none, and its 12
lines with "holds" in area B are all English), with only the prose, the NEW blocks, the owner's-test
tables, the refusals and the costs edited (`target/redesign4/v3-03-area-B/check_carried.py` checks
that every OLD and means-exactly block is byte for byte v2's). What changed from v2 is said once per
answer, in its "v3" line, and in the table of section 3; what changed before that is in `V2/area-B.md`.

Each answer has the eight parts: 1 intent; 2 OLD with today's output; 3 NEW; 4 the owner's test for
every declared name; 5 means exactly (today's Ember or C doing what NEW means); 6 still refused, each
with its code and message; 7 cost; 8 kind. In the owner's-test tables "none (default: ...)" means no
word in the line answers that question and the stated default does (the brief allows that only where the
default never changes anything, never aliases silently and never copies silently).

## 0. Terms used in this area

- **field clause**: what a method's first line says after its parameters about its object, `this`:
  `changes <fields>` (row 3: the fields it may change), `reads <fields>` (row 7: the fields it reads),
  `changes this` (any field, or the whole value), `owns this` (it takes the object over). Checked
  against the body and at every call (R5). `changes this` and `changes <fields>` say only that the
  method may change its own object; they promise nothing about other names (verdict 18).
- **one-owner object**: an object with one owner. A second owner is written `shares` or `shares mut`
  (R1, R3), so every other name for a one-owner object is a `ref` or `ref mut` name the compiler can see.
- **read-only view**: every view but `ref mut`, `sharedref mut` and a `MutSpan` held by a `ref mut` name
  (a plain name or parameter holding a `MutSpan` only reads its elements, R5, verdict 34); it promises
  that what it reads does not change while it is used (R8 (g), verdict 5).
- **kind test**: `if s is Circle as c:` (runs its block when `s` is a `Circle`, and names it `c`) and
  `if s is Circle:` (tests only); the "not sure" downcast. `cast s as Circle` is the "sure" one: it
  stops the program if `s` is not a `Circle` (R14, verdict 27).
- **a change under a lock**: a change made through a `Mutex`, `RwLock` or `Atomic` (`g.clear()` inside
  `with g = scores.lock():`). A plain name may make it, by the type's own operations, and the call takes
  no word (verdict 3); the function's first line names it by the type of what changes
  (`changes Mutex[Array[String]]`), as every change is named (R5, R12, verdict 29).
- **access word**: today's run-time check on class fields (`[EXC-19]`, spec 2712-2730): one word per
  non-`Copy` field, tested on each long-term access, set for a whole `mut self` call. Gone in the new
  rules.
- **view**: a value that points into something someone else owns and may not outlive it (`ref T`,
  `Span[T]`, `StrView`, a struct holding one; `V3/rules.md` section 0).
- **source** of a result: what a returned view points into (R6).
- **guard**: what `m.lock()`, `rw.read()`, `rw.write()` return; it holds the lock until its `with`
  block ends.
- **own local**: a name made in this function by the value on its right (`total = 0`, `d = Door()`);
  only this function can change it and nothing else refers to it.
- **reshape**: a change that can free or move memory (`push`, `clear`, replacing a value that owns
  memory; `V3/rules.md` section 0). Writing a number is not a reshape.

## 1. Area B's rules, as the v3 rules state them

1. **A method's first line says what it does to `this`** (R5): nothing (it only reads), `reads a, b`,
   `changes a, b`, `changes this`, or `owns this`, after the parameters. A method does not list its
   object among its parameters; inside it the object is `this`. A method with no `reads` is taken by
   callers in other packages to read every field (R17). `changes this` promises nothing about other
   names: `this` is never the only name of its object and is never compiled with noalias (R4, R5,
   verdict 18).
2. **All of it is checked when compiling, for every object, class or struct** (R5, R8, R9): no access
   words, nothing checked while running.
3. **A name that changes someone else's value says `ref mut` on the name** (R4, R5): a parameter
   (`ref mut c: Counter`), a loop name (`for ref mut p in players:`), a pattern name (row 8), and a name
   or parameter that writes the elements of a `MutSpan` (`fn first_even(ref mut ps: MutSpan[int]) ->
   ref mut int:`; verdict 34); **and a call that changes an argument says it too**: `bump(ref mut a)`, a
   `MutSpan` argument included (R4; verdict 3). What a function may do with a parameter is set by the
   word in front of the parameter's name, for every type (R4, verdict 32). A `Mutex`, `RwLock` or
   `Atomic` argument takes no word at the call (verdict 3). A plain name never changes anything, a
   `Cell` field included (R5); only a `Mutex`, `RwLock`, `Atomic` or channel end is changed through a
   plain name, by its own operations (R5).
4. **A read-only name promises that what it reads does not change while it is used** (R8 (g), verdict
   5): a change of it through another name is refused (E3021), and a reshape ends it (E3060 at its later
   use). A copy of your own is `copy of x`.
5. **A returned view points into its object (a method) or into its view arguments (a function), all of
   it**; `from <name>` after the result type narrows it to one parameter or one field of `this` (R6;
   replaces `@borrows`). Since a view of the whole object stops other changes of the object while it is
   used (R8 (g)), `from` is what lets a caller change the rest of the object meanwhile (AN-22).
6. **A call whose result type is `ref mut T` is a place**: `first(ref mut xs) += 1` (R6).
7. **Views may be kept in lists and fields** (R7): a local list of views lives no longer than what its
   views point into, with nothing written; a view field of a class says `from <let field>` or
   `from static` (E2118 otherwise). A `let` field never changes after `init`, so views into it stay
   valid for the object's whole life with no tracking.
8. **A view made through a guard points into the guard** (R8 (e), row 4), so it ends with the `with`
   block.
9. **`init` sets the fields of the object it builds through `this` with no word, and may call a `reads`
   method once the fields it names are set** (R5, row 7).
10. **A function that changes something under a lock says so on its first line** (R5, R12, verdict 29),
    named by the type of what changes: `fn clear(scores: Mutex[Array[String]]) changes
    Mutex[Array[String]]:` for `g.clear()` inside `with g = scores.lock():`. The call still takes no word
    (`clear(m)`, verdict 3). A loop that walks the locked list while such a call runs is refused when
    compiling (E3020), instead of waiting for the lock this thread already holds.
11. **A downcast says whether it is sure** (R14, verdict 27): `cast x as T` when sure (it stops the
    program if `x` is not a `T`), `if x is T as name:` when not; a cast or a kind test of a place stays a
    place, so `pool = ref mut cast this.pools[k] as Pool[T]` and `if this.pools[k] is Pool[T] as
    ref mut pool:` both name the registry's own pool. `cast x if T` is gone.

What still runs while the program runs, in area B: Ember's bounds checks (as today, not this area's);
`@cached`'s flag test (one branch per call, the hand-written C cache); the one type-id compare of a
downcast or a kind test (`cast ... as Pool[T]`, as today; `if ... is Pool[T] as ...:`); the lock a
program takes (`with g = scores.lock():`), as C takes it. Nothing else (`V3/rules.md` section 2).

## 2. Area B's rulings on `@cached`, `from` and `Cell`, in the v3 words

1. **`@cached`'s result**: a `Copy` result is returned by value; any other result is returned as a
   `ref T` view into the kept value. A change to a field its `reads` names, through any name, clears the
   kept value; clearing it replaces a value that owns memory, a reshape, so it ends every view of the
   kept value (R8 (b), E3060 at a later use). On an object several threads reach, the slot is filled
   once under a once-flag (area E's ruling, run 3's clash 6). Filling the slot is not named on the
   method's first line, though it is made under that flag: the slot is not a field of the program, no
   name but the method reads it, and filling it changes nothing a view or a loop reaches (it held no
   value before); the change that clears it is the change of a `reads` field, which is named where it
   is made. Verdict 29's "a change under a lock is named" therefore adds nothing to a `@cached` method
   (section 6, note 4). Its refusals are E2117.
2. **`from` on a field, the heap-block condition**: the named field must be `let` and a `String` or an
   `Array` (contents outside the header, `ember_vec {ptr, len, cap}`; ADR-141 frame buffers only for
   lists that never leave their function). A fixed array or an inline struct field is refused (AN-25,
   E2118).
3. **`Cell`**: "a plain name never changes anything" holds with no exception (R5). Setting a `Cell`
   field needs a name that may change it (`ref mut s`, `changes this`, `changes frame`), or an owner of
   a `shares mut` value (R9): a name made with `shares mut`, or a parameter `owns s: shares mut Clock`,
   not a plain parameter `s: shares mut Clock`, which only reads (R4, verdict 32). Cost: none,
   `Cell.set` is a plain store. What `Cell` still gives: it lets
   the `shares mut` owners of a value set it whole, and never lends a reference into it.

## 3. What v3 changed in each answer

"Meaning changed" compares with `V2/area-B.md`. Every answer now cites the owner's verdicts as
"verdict N" (v2 wrote "decision N") and the v3 files. The verdicts that land in area B are 29 (locks,
AN-30), 27 (downcasts, AN-22), 34 (`MutSpan`, AN-28), 22 (what a lambda's type covers, AN-29) and 32
(the word on the name, AN-20); 21 appears in AN-29's refusal. Verdicts 23, 24, 26, 28, 30, 31 and 33
touch no code of area B (no `shares` hand-on, arena, `holds`, tree walk, thread, C call or `.copied()`
in its NEW blocks).

| answer | what changed | meaning changed? |
|---|---|---|
| AN-17 (row 3) | the v3 line only | no |
| AN-18 | the v3 line only | no |
| AN-19 | the v3 line only | no |
| AN-20 | a refusal added: a plain parameter of a `shares mut` type that sets a `Cell` (E3023, verdict 32); section 2's `Cell` ruling says the same | no |
| AN-21 | why verdict 29 adds nothing to a `@cached` method's once-flag (section 2, ruling 1) | no |
| AN-22 | the sure downcast kept (verdict 27); the kind test `if this.pools[k] is Pool[T] as ref mut pool:` shown in a new method `put`, with its owner's-test rows; three refusals added: a plain name from the kind test that changes the pool (E3023), today's `cast ... if Pool[T]` and the removed `cast ... when Pool[T]` (E1010), `put` without `changes pools` (E2105) | no |
| AN-23 | the v3 line only | no |
| AN-24 | the v3 line only | no |
| AN-25 | the v3 line only | no |
| AN-26 | the v3 line only | no |
| AN-27 | the v3 line only | no |
| AN-28 | `first_even` takes `ref mut ps: MutSpan[int]` (verdict 34); how a name holding a `MutSpan` is pointed at another span; the plain parameter refused (E3023) | **yes**, for one line: v2's `fn first_even(ps: MutSpan[int]) -> ref mut int:` is refused; the program does what it did |
| AN-29 | the lambda's plain `fn() -> int` read under verdict 22 (captured variables are not part of the function); two refusals added: a lambda that changes a captured variable without naming it (E2105), and one that names it given to `apply` (E2113, with the change-list line of verdict 21) | no |
| AN-30 (row 4) | OLD's `clear` added to NEW with `changes Mutex[Array[String]]` (verdict 29); `main` keeps a copy, clears, prints; owner's-test rows for `clear`, `best`, `n`; two refusals added: `clear` without its clause (E2105) and `clear(m)` in a loop over the locked list (E3020) | no, for v2's lines (`clear` is new to NEW) |
| AN-31 (row 7) | the v3 line only | no |
| AN-32 | the v3 line only | no |
| AN-33 | the v3 line only | no |

## 4. The answers

#### AN-17: a method changes one field while a loop walks another (A1). Agreed row 3.

**v3:** as in v2; verdicts 21 to 34 change nothing here (no function value, lock, downcast or
`MutSpan`). Row 3's line stays `fn note(msg: String) changes log:`, as the owner agreed it without
`self` (verdict 2). R8 (g) (verdict 5) refuses nothing here: the loop reads the player only through `p`
itself. Meaning unchanged.

**1. Intent:** `tick` takes 1 hit point from every player and, when one dies, `note` appends a line to
the game's log; `note` only appends to `log`.

**2. OLD** (`o17a_class_note_in_loop.em`, phase 1's `e01d`):

```ember
class Player:
    name: String
    hp: int

class Game:
    players: Array[Player]
    log: Array[String]
    frame: int = 0

    fn note(mut self, msg: String):
        self.log.push(f"[{self.frame}] {msg}")

    fn tick(self):
        for p in self.players:
            p.hp -= 1
            if p.hp <= 0:
                self.note(f"{p.name} died")
```

```text
error[E3020]: cannot mutate `self.players` while it is borrowed by this loop
  --> o17a_class_note_in_loop.em:18:17
   |
15 |         for p in self.players:
   |                  ------------ `self.players` borrowed here for the whole loop
...
18 |                 self.note(f"{p.name} died")
   |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow here
   |
   = help: collect the indices or values first, then mutate; use `retain` or `drain` when the operation fits
   = help: or iterate over a fixed index range and mutate only after taking the current value
   = note: a `for` loop borrows the iterable until the loop ends (CTL-2)

error: could not compile due to 1 error(s)
exit=1
```

The message says `note` changes `self.players`; it does not, it changes `log`. (The same program as
a struct with a `pub` method compiles today inside one package, `o17b_struct_pub_note.em`, `exit=0`;
`[BRW-10]` says a `pub` method borrows all of `self` for callers in other packages.)

**3. NEW:**

```ember
class Player:
    name: String
    hp: int

class Game:
    players: Array[Player]
    log: Array[String]
    frame: int = 0

    fn note(msg: String) changes log:
        this.log.push(f"[{this.frame}] {msg}")

    fn tick() changes players, log:
        for ref mut p in this.players:
            p.hp -= 1
            if p.hp <= 0:
                this.note(f"{p.name} died")

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    g.tick()
    println(g.log)
```

`this.note(...)` is a call on `this` while `p` is the only name for one player (R8 (f)); it is accepted
because `note`'s line says it changes only `log`, and its body (seen inside the package, R17) reads only
`frame` and `log`, so it cannot touch the player `p` names. The calls `this.note(...)` and `g.tick()`
are on an object and carry no word (R4): the methods' first lines say what they change.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `name`, `hp` (fields of `Player`) | none (default: the player's own value, by its type) | none (default: a field changes only through a name that may change its object: a `ref mut` name, or a method whose line says `changes <field>` or `changes this`) |
| `players`, `log`, `frame` (fields of `Game`) | none (default: the game's own list / number) | none (default as above): `changes players, log` on `tick`, `changes log` on `note` |
| `msg` | none (default: a plain parameter reads the caller's `String`, R4) | none (default: a plain parameter never changes anything, R5) |
| `this` in `note` | `this`: the game `note` was called on (R5) | `changes log`: only `log` |
| `this` in `tick` | `this` | `changes players, log` |
| `p` | `ref mut`: a name for the list's own element | `ref mut`: yes, and the only name for it while it is used (R8 (f)) |
| `g` | none (default: own local, made by `Game(...)`) | none (default: own local) |

**5. Means exactly** (`m17_tick_note.em`): one owner, so today's struct rules give the same
compile-time check (the loop changes `players`, `note` touches only `log`, `[BRW-10]`); today writes
`mut self` and `iter_mut()`:

```ember
struct Player:
    name: String
    hp: int

struct Game:
    players: Array[Player]
    log: Array[String]
    frame: int = 0

    fn note(mut self, msg: String):
        self.log.push(f"[{self.frame}] {msg}")

    fn tick(mut self):
        for p in self.players.iter_mut():
            p.hp -= 1
            if p.hp <= 0:
                self.note(f"{p.name} died")

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    g.tick()
    println(g.log)
```

```text
===== m17_tick_note.em
exit=0
```

**6. Still refused:**
- `note`'s body changing a field it does not name (`this.frame += 1`), E2105: "`note` changes
  `this.frame`, which its first line does not name; write `fn note(msg: String) changes log, frame:`".
- `tick` calling `note` without naming `log`, E2105: "`tick` calls `note`, which changes `log`; add it
  to `tick`'s first line: `fn tick() changes players, log:`".
- Inside the loop, a method that changes the list being walked (`this.add_player(Player(name="imp",
  hp=3))` on line 17, with `fn add_player(owns p: Player) changes players: this.players.push(p)`), E3020:
  "`this.add_player(...)` (line 17) changes `this.players`, which this loop walks (line 14); collect the
  new players in a list of your own: write `joined: Array[Player] = []` before the loop,
  `joined.push(Player(name="imp", hp=3))` in place of line 17, and `this.players.extend(joined)` after
  the loop" (the understandable limitation: add or delete after the loop).
- From another package, where `note` is `pub` and says no `reads` (R17), E3021: "`note` may read any
  field of `Game`, `players` included, which this loop changes through `p` (line 14); call `note` after
  the loop: write `died: Array[String] = []` before the loop, `died.push(f"{p.name} died")` in place of
  `this.note(f"{p.name} died")`, and `for msg in died: this.note(msg)` after the loop (in `note`'s own
  package, its first line can instead say what it reads: `fn note(msg: String) reads frame changes
  log:`)".
- A plain loop name that changes the player (`for p in this.players:` then `p.hp -= 1`), E3023:
  "`p.hp -= 1` changes a player, but `p` only reads it; to change the players through `p`, write
  `for ref mut p in this.players:`".

**7. Cost:** none; the check is when compiling. Today's class version, if it compiled, would pay
access-word checks while running (`[EXC-19]`, spec 2712-2730: a load and a compare per long-term
access, a check per field on entry to a `mut self` method).

**8. Kind:** a smarter compile-time solution. Rust refuses this program (E0499, a second `&mut self`
while the loop borrows `self.players`); the written field set lets it through, checked.

#### AN-18: `mut self` on a class means "hold a write access on every field for the whole call"

**v3:** as in v2 (`fn reset() changes this:`, verdict 6; `copy of msg`, verdict 12; `this` compiled
without noalias, verdict 18); verdicts 21 to 34 change nothing here. Verdict 32 (the word in front of a
parameter's name sets what the function may do with it) is what `tick(ref mut g: Game)` already
writes. Meaning unchanged.

**1. Intent:** a class method says that it may change its object, and the program is checked when
compiling, not while running.

**2. OLD** (`o18_class_mut_self_two_handles.em`):

```ember
class Game:
    players: Array[String]
    log: Array[String]

    fn note(mut self, owned msg: String):
        self.log.push(msg)

fn tick(g: Game, other: Game):
    for p in g.players:
        other.note(p.clone())

fn main():
    g = Game(players=["orc"], log=[])
    tick(g, g)
    println(g.log)
```

```text
===== o18_class_mut_self_two_handles.em
exit=0
```

Accepted. By `[CLS-7]` (spec 1740-1742) and `[EXC-19]` (spec 2718-2721), `other.note(...)` "checks
every field word of the object ... at entry ... marks each as written": the loop holds a read access
on `players` of the same object, so by those rules the program would stop with "exclusivity
violation: write access to Game.players while a read access to Game.players is active" (`[EXC-1]`,
spec 2706) although `note` never touches `players` (read from the spec; the program was not run, as
the rules forbid). The conflict is found only while running, and it is a false one.

**3. NEW:**

```ember
class Game:
    players: Array[String]
    log: Array[String]
    score: int = 0

    fn note(msg: String) changes log:
        this.log.push(copy of msg)

    fn reset() changes this:
        this.players = []
        this.log = []
        this.score = 0

fn tick(ref mut g: Game):
    for p in g.players:
        g.note(f"{p} moved")
    g.reset()

fn main():
    g = Game(players=["orc"], log=[])
    tick(ref mut g)
    println(g.log)
```

`changes this` has one meaning for every type: this method may change any field of its object, or the
whole value (R5); it is checked when compiling, like `changes log`, and nothing is marked while running.
It says nothing about other names: the game could also be reached through a link while `reset` runs
(R5, verdict 18), and R8 (b), (c) and (g) keep every view and walk safe without `this` being alone. In
`tick`, the loop name `p` reads an element of `g.players` while `g.note(...)` runs; `note`'s line says
it changes only `log`, so `p`'s promise (R8 (g)) holds. The OLD `tick(g, g)` cannot be written for a one-owner object without saying it: with
`fn tick(g: Game, ref mut other: Game)` the call must be `tick(g, ref mut g)`, a read and a change of
one object in one call (refused below).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `players`, `log`, `score` | none (default: the game's own values) | none (default: through `changes <field>` or `changes this` only) |
| `this` in `note` | `this`: the game `note` was called on | `changes log`: only `log` |
| `msg` | none (default: a plain parameter reads the caller's `String`) | none (default: never); `copy of msg` makes the copy that is pushed |
| `this` in `reset` | `this` | `changes this`: any field |
| `g` (parameter of `tick`) | `ref mut`: the caller's game | `ref mut`: yes |
| `p` | none (default: a read-only view of the list's element, as row 8's plain `path`) | none (default: never) |
| `g` (in `main`) | none (default: own local) | none (default: own local); `tick(ref mut g)` says the call changes it |

**5. Means exactly** (`m18_mut_self_static.em`), today's struct `mut self`, checked when compiling:

```ember
struct Game:
    players: Array[String]
    log: Array[String]
    score: int = 0

    fn note(mut self, msg: String):
        self.log.push(msg.clone())

    fn reset(mut self):
        self.players = []
        self.log = []
        self.score = 0

fn tick(mut g: Game):
    for p in g.players:
        g.note(f"{p} moved")
    g.reset()

fn main():
    g = Game(players=["orc"], log=[])
    tick(g)
    println(g.log)
```

```text
===== m18_mut_self_static.em
exit=0
```

**6. Still refused:**
- `g.reset()` inside the loop, E3020: "`reset` may change every field of `Game`, `players` included,
  and this loop walks `g.players` (line 15); call `g.reset()` after the loop".
- `tick(g, ref mut g)` with `fn tick(g: Game, ref mut other: Game)`, E3021: "`tick` would read `g` and
  change it through `other` in the same call; pass it once: write `fn tick(ref mut g: Game):` and call
  `tick(ref mut g)`".
- `tick(g)` for `fn tick(ref mut g: Game)`, E3028: "`tick` may change `g` (its first line says
  `ref mut g: Game`); say so at the call: `tick(ref mut g)`".
- `g.reset()` where `tick`'s `g` is a plain parameter, E3023: "`reset` changes the game (its first line
  says `changes this`), and `g` only reads it; write `fn tick(ref mut g: Game):` and call
  `tick(ref mut g)`".
- `this.log.push(msg)` in `note`, E3013: "`this.log.push(msg)` would hand over the caller's `String`,
  but `msg` only reads it; to push a `String` of your own write `this.log.push(copy of msg)`".
- `changes this` on `init`: see AN-32 (E2108).

**7. Cost:** none for one-owner objects: no access words, no check on entry (today: one word per
non-`Copy` field and a check per field on entry, `[EXC-19]`). A `shares` object is not checked while
running either (R9). `this` in `reset` is compiled without noalias (R4, verdict 18), as a C function
compiles a plain `Game *` parameter; `tick`'s `ref mut g` is compiled with noalias (C's `restrict`).
Nothing was measured for this answer.

**8. Kind:** a smarter compile-time solution.

#### AN-19: plain `self` in a class method changes fields with no word

**v3:** as in v2; verdicts 21 to 34 change nothing here. Meaning unchanged.

**1. Intent:** `open` sets the door's angle to 90.

**2. OLD** (`o19_plain_self_writes.em`):

```ember
class Door:
    angle: f64 = 0.0

    fn open(self):
        self.angle = 90.0

fn main():
    d = Door()
    d.open()
    println(d.angle)
```

```text
===== o19_plain_self_writes.em
exit=0
```

Accepted; nothing on `open`'s first line says it changes the door.

**3. NEW:**

```ember
class Door:
    angle: f64 = 0.0

    fn open() changes angle:
        this.angle = 90.0

fn main():
    d = Door()
    d.open()
    println(d.angle)
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `angle` | none (default: the door's own number) | none (default: through `changes angle` or `changes this`) |
| `this` | `this`: the door `open` was called on | `changes angle` |
| `d` | none (default: own local) | none (default: own local) |

**5. Means exactly** (`m19_open.em`): today's class method, which does exactly this (a `Copy` field
write is never checked, `[EXC-17]`):

```ember
class Door:
    angle: f64 = 0.0

    fn open(self):
        self.angle = 90.0
```

```text
===== m19_open.em
exit=0
```

**6. Still refused:**
- The OLD form with `this` (`fn open():` then `this.angle = 90.0`), E2105: "`open` changes
  `this.angle`; write its first line as `fn open() changes angle:`".
- The OLD form itself (`fn open(self):`), E1010: "`self` is not a word of Ember: inside a method its
  object is `this`; write `fn open() changes angle:` and `this.angle = 90.0` in its body".

**7. Cost:** none.

**8. Kind:** a readable spelling of what Rust's `&mut self` says (this method may change its object),
narrowed to the field. Unlike `&mut self` it does not promise that no other name reaches the door
(verdict 18); R8 keeps every view safe without that promise.

#### AN-20: a plain parameter, a `for` name or a `Cell` field changes a class object with no word

**v3:** as in v2. Verdict 32 is what this answer already writes: the word in front of each
parameter's name (`ref mut c`, `ref mut players`, `ref mut s`, plain `s` in `show`) sets what the
function may do, for a class, a list and a struct alike. The `Cell` ruling (section 2) now says that a
plain parameter of a `shares mut` type does not set a `Cell` (verdict 32). Meaning unchanged.

**1. Intent:** `bump` adds 1 to the caller's counter; `heal_all` sets every player's health to 100;
`tick` advances a clock's frame number.

**2. OLD** (`o20a_param_changes_object.em`, `o20b_for_class.em`, `o20c_cell_plain.em`):

```ember
class Counter:
    n: int = 0

fn bump(c: Counter):
    c.n += 1
```

```ember
class Player:
    health: float = 0.0

fn heal_all(players: Array[Player]):
    for p in players:
        p.health = 100.0
```

```ember
struct S:
    frame: Cell[int]

fn tick(s: S):
    s.frame.set(s.frame.get() + 1)
```

```text
===== o20a_param_changes_object.em
exit=0
===== o20b_for_class.em
exit=0
===== o20c_cell_plain.em
exit=0
```

All three accepted; `[FN-1]` says the default parameter "cannot assign through it".

**3. NEW:**

```ember
class Counter:
    n: int = 0

class Player:
    health: f64 = 0.0

struct Clock:
    frame: Cell[int]

fn bump(ref mut c: Counter):
    c.n += 1

fn heal_all(ref mut players: Array[Player]):
    for ref mut p in players:
        p.health = 100.0

fn tick(ref mut s: Clock):
    s.frame.set(s.frame.get() + 1)

fn show(s: Clock):
    println(s.frame.get())

fn main():
    counter = Counter()
    bump(ref mut counter)
    team = [Player(), Player()]
    heal_all(ref mut team)
    clock = Clock(frame=Cell(0))
    tick(ref mut clock)
    show(clock)
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `n`, `health` | none (default: the object's own number) | none (default: through a `ref mut` name, `changes <field>` or `changes this`) |
| `frame` | `Cell`: a value replaced whole by `.set` | none (default as any field; section 2: `.set` needs a name that may change) |
| `c` | `ref mut`: the caller's counter | `ref mut`: yes |
| `players` | `ref mut`: the caller's list | `ref mut`: yes |
| `p` | `ref mut`: the list's own element | `ref mut`: yes |
| `s` in `tick` | `ref mut`: the caller's clock | `ref mut`: yes |
| `s` in `show` | none (default: a read-only view of the caller's clock) | none (default: never, `Cell` included) |
| `counter`, `team`, `clock` | none (default: own local) | none (default: own local); `ref mut` at each call says the call changes it |

**5. Means exactly** (`m20_ref_mut_names.em`): today's spelling of the same run-time behaviour, with
no word on the names:

```ember
fn bump(c: Counter):
    c.n += 1

fn heal_all(players: Array[Player]):
    for p in players:
        p.health = 100.0

fn tick(s: Clock):
    s.frame.set(s.frame.get() + 1)

fn show(s: Clock):
    println(s.frame.get())
```

```text
===== m20_ref_mut_names.em
exit=0
```

**6. Still refused:**
- `fn bump(c: Counter): c.n += 1`, E3023: "`bump` changes `c.n`, but `c` only reads the caller's
  counter; to change it write `fn bump(ref mut c: Counter):` and call `bump(ref mut counter)`".
- `for p in players: p.health = 100.0`, E3023: "`p.health = 100.0` changes a player, but `p` only reads
  it; write `for ref mut p in players:`, with `fn heal_all(ref mut players: Array[Player]):`".
- `fn tick(s: Clock): s.frame.set(...)`, E3023: "`tick` sets `s.frame`, but `s` only reads the caller's
  clock; write `fn tick(ref mut s: Clock):` and call `tick(ref mut clock)`".
- A plain parameter of a shared clock that sets its `Cell`, `fn tick(s: shares mut Clock):` with
  `s.frame.set(s.frame.get() + 1)`, E3023 (R4, verdict 32): "`tick` sets `s.frame`, but `s` only reads
  the caller's clock (`shares mut Clock` says what the caller's clock is, not what `tick` may do with
  it); write `fn tick(ref mut s: shares mut Clock):` and call `tick(ref mut clock)`".
- `fn bump(mut c: Counter):`, E0113: "`mut c` on a parameter: to change the caller's `c` through `c`,
  write `ref mut c: Counter`".
- `bump(counter)`, E3028: "`bump` may change `counter` (its first line says `ref mut c: Counter`); say
  so at the call: `bump(ref mut counter)`"; `show(ref mut clock)`, E3028: "`show` only reads its
  argument; remove `ref mut`: `show(clock)`".

**7. Cost:** none; the same instructions, only the spelling changes.

**8. Kind:** a readable spelling of Rust's guarantee (`&mut`), with the word on the name and at the
call.

#### AN-21: caching inside a read-only method (A31)

**v3:** as in v2; the five E2117 refusals keep v2's messages, which are `V3/rules.md` section 3's.
Verdict 29 (a change under a lock is named on the first line) adds nothing to `radius`: filling the
kept result on an object several threads reach is done under a once-flag, but the slot is not a field
of the program and nothing but `radius` reads it (section 2, ruling 1). Meaning unchanged.

**1. Intent:** `radius()` computes the mesh radius once, keeps it, and computes it again only after
`verts` changes.

**2. OLD** (`o21_lazy_cache.em`, phase 1's `a31_lazy_cache.em`):

```ember
struct Mesh:
    verts: Array[f32]
    radius_cache: Option[f32]

    fn radius(self) -> f32:
        if self.radius_cache.is_none():
            r: f32 = 0.0
            for v in self.verts:
                r = max(r, abs(v))
            self.radius_cache = Some(r)
        return self.radius_cache.unwrap()
```

```text
error[E3023]: cannot mutate borrowed parameter `self`
  --> o21_lazy_cache.em:12:13
   |
 7 |     fn radius(self) -> f32:
   |               ---- `self` is borrowed here
...
12 |             self.radius_cache = Some(r)
   |             ^^^^^^^^^^^^^^^^^ this write needs mutable access
   |
   = help: restructure to a single owner and declare this parameter `mut self: Mesh`
   = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
   = help: if the value has identity, make it a `class`
   = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

error: could not compile due to 1 error(s)
exit=1
```

(The help's `mut self: Mesh` is not Ember's spelling either.) And a hand-kept cache goes stale when
someone changes `verts` and forgets to clear it.

**3. NEW** (`@cached`, row 7's `reads`):

```ember
struct Mesh:
    verts: Array[f64]

    @cached
    fn radius() reads verts -> f64:
        r = 0.0
        for v in this.verts:
            r = max(r, abs(v))
        return r

fn main():
    m = Mesh(verts=[1.0, -2.0])
    println(m.radius())      # computes 2.0 and keeps it
    println(m.radius())      # the kept 2.0, no loop
    m.verts.push(-5.0)       # a change to `verts` clears the kept result
    println(m.radius())      # computes 5.0
```

The result is a `Copy` value returned by value, or, for any other type, a `ref T` view into the kept
result (`-> ref Array[f64]`). A change to a `reads` field through any name clears the kept result: an
assignment, a call of a method whose line says `changes verts` or `changes this`, a `ref mut` view of
`verts`; clearing it ends every view of the kept result (section 2).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `verts` | none (default: the mesh's own list) | none (default: field; any change clears the kept result) |
| `this` | `this`; `@cached` says the result is kept; `reads verts` says what it depends on | no `changes`: no field changes (R5) |
| `r` | none (default: own local) | none (default: own local) |
| `v` | none (default: reads each number in turn) | none (default: a plain loop name never changes anything) |
| `m` | none (default: own local) | none (default: own local; `push` changes it, which clears the kept result) |

**5. Means exactly** (`m21_cache_by_hand.em`): the slot the compiler adds, written by hand, and the
clear the compiler inserts after every change to `verts`:

```ember
struct Mesh:
    verts: Array[f64]
    radius_slot: Cell[Option[f64]] = Cell(None)

    fn radius(self) -> f64:
        kept = self.radius_slot.get()
        if kept.is_some():
            return kept.unwrap()
        r = 0.0
        for v in self.verts:
            r = max(r, abs(v))
        self.radius_slot.set(Some(r))
        return r

fn main():
    m = Mesh(verts=[1.0, -2.0])
    println(m.radius())
    println(m.radius())
    m.verts.push(-5.0)
    m.radius_slot.set(None)
    println(m.radius())
```

```text
===== m21_cache_by_hand.em
exit=0
```

**6. Still refused:**
- `@cached` without `reads`, E2117: "say what `radius` depends on, so a change can clear it: write
  `fn radius() reads verts -> f64:` under `@cached`".
- With other parameters, E2117: "`@cached` keeps one result, and `radius` takes `scale`; remove
  `@cached`, or keep the results in a field: `radii: Map[f64, f64]`".
- Reading a `shares`, `shares mut`, `thread_shared`, `weakref`, `sharedref mut` or `Cell` field, E2117:
  "`radius` reads `mesh`, which its other owners may change without `Mesh` seeing it, so a kept result
  could be stale; remove `@cached`".
- With `changes` or `changes this`, E2117: "a cached method only reads; remove `changes verts`".
- On an object several threads reach, the slot is filled once under a once-flag (`Once`: an acquire
  load and a branch once filled, area E's ruling); refused only where a `reads` field changes while other
  threads hold the object, E2117: "`radius` reads `verts`, which other threads change under `lock`; a
  kept result could be stale; remove `@cached`".
- A view of a kept result used after a change to `verts`, with
  `@cached fn sorted_verts() reads verts -> ref Array[f64]:` and the lines `s = m.sorted_verts()` (line
  1), `m.verts.push(1.0)` (line 2), `println(s[0])` (line 3), E3060: "`m.verts.push(1.0)` (line 2)
  clears the kept result `s` views (`m.sorted_verts()`, line 1), and `s` is used on line 3; use `s`
  before line 2, or name the result again after it: `s = m.sorted_verts()`".

**7. Cost:** one flag test per call; one store (clear the flag) at each place that may change `verts`;
the slot and flag in the object (8 bytes for the `f64` and one flag). The same as the cache a C
programmer writes by hand (the translation above is that program).

**8. Kind:** a new mechanism.

#### AN-22: returning references (A5): a result borrows all of `self` unless `@borrows` is written above; the ECS pool reference is E3060

**v3:** NEW as in v2. `get_mut`'s downcast is the "sure" one, `cast this.pools[7u128] as Pool[T]`,
which verdict 27 keeps (the pool under that key is always a `Pool[T]`; if it is not, the program
stops); a cast of a place is a place, so `pool = ref mut cast ...` names the registry's own pool
(R14). The prose now gives the "not sure" form, the kind test `if this.pools[k] is Pool[T] as ref mut
pool:`, and part 6 refuses today's `cast ... if Pool[T]` (E1010), a plain name from a kind test that
then changes the pool (E3023), and a kind test that changes the pool in a method whose first line does
not say `changes pools` (E2105). Meaning unchanged. (v2's change, a two-word program without `from src`
refused by R8 (g), stands; it is explained in part 3.)

**1. Intent:** `next_word` returns the next word of the source text, and the caller keeps two words at
once; `get_mut[T]` returns a component inside its pool, to change it in place; `put[T]` (part 3's
prose) stores a component only if the pool under its key holds that kind.

**2. OLD** (`o22c_two_words_flat.em`; phase 1's nested form `o22a_two_words.em` gives the same E3025):

```ember
struct Parser:
    src: str
    pos: int = 0

    fn next_word(mut self) -> str:
        rest = self.src[self.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        self.pos += end + 1
        return rest[..end]

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    a = p.next_word()
    b = p.next_word()
    println(a, b)
```

```text
error[E3025]: cannot call a method on `p` while `p.src` is mutably borrowed
  --> o22c_two_words_flat.em:16:9
   |
15 |     a = p.next_word()
   |         ------------- mutable borrow of `p.src` starts here
16 |     b = p.next_word()
   |         ^^^^^^^^^^^^^ conflicting access here
17 |     println(a, b)
   |             - borrow later used here
   |
   = help: inline the field access, take the two fields as separate parameters, or split the method
   = note: a call borrows the fields of `self` its method uses (BRW-10); this method uses all of `self`, so the other fields do not stay disjoint across the call (BRW-4)

error: could not compile due to 1 error(s)
exit=1
```

The help does not name the fix that works (`@borrows(self.src)`, written on its own line above the
function). The ECS form (`o22b_ecs_pool.em`, `target/analysis/ecs5/pos.em`):

```ember
struct Reg:
    pools: Map[u128, AnyPool]

    fn get_mut[T](mut self, i: int) -> ref mut T:
        pool = cast self.pools[7u128] as Pool[T]
        return ref mut pool.dense[i]
```

```text
error[E3060]: `pool.dense[…]` does not live long enough
  --> o22b_ecs_pool.em:21:35
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

**3. NEW:**

```ember
struct Parser:
    src: StrView
    pos: int = 0

    fn next_word() changes pos -> StrView from src:
        rest = this.src[this.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        this.pos += end + 1
        return rest[..end]

fn first_word(text: StrView) -> StrView:
    p = Parser(src=text)
    return p.next_word()     # points into `text` (`from src`), so it may outlive `p`

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    a = p.next_word()
    b = p.next_word()        # `a` points into `text`, not into `p`
    println(a, b, first_word(text))
```

```ember
struct Registry:
    pools: Map[u128, owns dyn AnyPool]

    [T]
    fn get_mut(e: int) changes pools -> ref mut T:
        pool = ref mut cast this.pools[7u128] as Pool[T]
        return ref mut pool.dense[pool.sparse[e]]
```

The rules: `from <name>` after the result type names the source: one parameter, or one field of `this`
written bare, as in `changes` and `reads` (R6). With no `from`, a method's result points into its object
and a function's into every view argument it could come from. `from src` on a view field means "into
what `src` views", so the result does not point into `p`, and `p.next_word()` may change `p.pos` while
`a` is used. Inside `next_word`, `rest` views the text that `this.src` views, not `this`, so
`this.pos += end + 1` while `rest` is still used changes nothing `rest` reads (R8 (g)). In `get_mut`,
`pool` is `ref mut`: a name for the slot inside `this.pools`, not a local of its own, so the result
points into `this` (the default) and the old E3060 cannot arise; a cast of a place is a place (R14): one
type-id compare, never the method table's address. `cast ... as` is the sure downcast: it stops the
program if the pool under that key is not a `Pool[T]`, a bug in the registry. Where the kind is not
sure, the test is written in an `if` and the block runs only when it passes (R14, verdict 27); this
method of `Registry` puts entity `e`'s component `c` in the pool under `k` when that pool holds `T`s,
and does nothing otherwise:

```ember
    [T]
    fn put(k: u128, e: int, owns c: T) changes pools:
        if this.pools[k] is Pool[T] as ref mut pool:
            pool.dense[pool.sparse[e]] = c
```

`is Pool[T]` asks whether the pool under `k` is a `Pool[T]`; `as ref mut pool` names it, and since a
value tested in its place stays a place, `pool` is the registry's own pool and may change it; with no
word, `pool` would only read it. `owns c` takes the component over, so it can be stored.

Without `from src` the two-word program is refused (part 6, E3021): with no `from`, a method's result
points into all of its object (R6), and a read-only view promises that what it points into does not
change while it is used (R8 (g), verdict 5), so `p.next_word()` may not change `p.pos` while `a` is
used, as run 3, today's Ember (E3025, OLD above) and Rust refuse it. `from src` is what lets the caller
keep two words, and what lets `first_word` return one.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `src` | `StrView`: a view of someone else's text | none (default: field) |
| `pos` | none (default: the parser's own number) | none (default: field; `changes pos`) |
| `this` in `next_word` | `this` | `changes pos` |
| `rest` | none (default: a read-only view of the text, as row 5's `word`) | none (default: never) |
| `end` | none (default: own local; `unwrap_or` gives a new number) | none (default: own local) |
| `text` (parameter of `first_word`) | `StrView`: a view of the caller's text | none (default: read-only parameter) |
| `p` (in `first_word` and in `main`) | none (default: own local; a `Parser` holds a view of `text`, so it lives no longer than `text`, R7) | none (default: own local) |
| `text` (in `main`) | `String`: own local | none (default: own local) |
| `a`, `b` | none (default: read-only views; `from src` on `next_word`'s line says they point into `text`) | none (default: never) |
| `pools` | `owns dyn`: one owner of each pool, each in its own block | none (default: field; `changes pools`) |
| `this` in `get_mut` | `this` | `changes pools` |
| `e` | none (default: read-only parameter; a number is copied in) | none (default: never) |
| `pool` (in `get_mut`) | `ref mut`: a name for the slot in `this.pools` | `ref mut`: yes |
| `this` in `put` | `this` | `changes pools` |
| `k`, `e` (parameters of `put`) | none (default: read-only parameters; numbers are copied in) | none (default: never) |
| `c` | `owns`: handed over by the caller | `owns`: it is `put`'s, handed on into the pool |
| `pool` (in `put`) | `is Pool[T]`: the pool under `k`, seen as a `Pool[T]`; a place tested in its place stays that place (R14) | `ref mut`: yes, inside the `if` block only |

**5. Means exactly:** `m22a_two_words_borrows.em` (today's `@borrows(self.src)` for `from src`), and for
one component type `m22b_pool_place.em` (today's `Box` for `owns`, `ref mut` on the place):

```ember
struct Parser:
    src: str
    pos: int = 0

    @borrows(self.src)
    fn next_word(mut self) -> str:
        rest = self.src[self.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        self.pos += end + 1
        return rest[..end]

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    a = p.next_word()
    b = p.next_word()
    println(a, b)
```

```ember
struct Pool:
    dense: Array[int]
    sparse: Array[int]

struct Registry:
    pools: Array[Box[Pool]]

    fn get_mut(mut self, kind: int, e: int) -> ref mut int:
        pool = ref mut self.pools[kind]
        return ref mut pool.dense[pool.sparse[e]]

fn main():
    reg = Registry(pools=[Box(Pool(dense=[10, 20], sparse=[1, 0]))])
    p = reg.get_mut(0, 0)
    p += 1
    println(reg.pools[0].dense[1])
```

```text
===== m22a_two_words_borrows.em
exit=0
===== m22b_pool_place.em
exit=0
```

`p = reg.get_mut(0, 0)` then `p += 1` in `m22b_pool_place.em` is today's spelling; in NEW it is
`p = ref mut reg.get_mut[int](0)` (R2: a name bound to a call whose result is `ref mut T` says
`ref mut`). The generic downcast cannot be said in today's Ember (`cast` needs class handles); its C is
last run's faithful model `target/redesign2/references/attack2/bench/get2.c -DCAST` (a type-id load and
compare, then the element's address). `put`'s kind test is the same load and compare used as the test
of an `if` (the store runs when it passes) instead of a stop; no separate program was written for it.

**6. Still refused:**
- `first_word` when `next_word`'s line has no `from` (`fn next_word() changes pos -> StrView:`), E3063:
  "`p.next_word()` points into `p` (with no `from`, a method's result points into its object), and `p`
  ends when `first_word` returns; if `next_word`'s result points only into the text, write
  `fn next_word() changes pos -> StrView from src:`". The compiler works the source out from the body
  to write this line; it never applies it silently (R17).
- A body that returns a view of something its `from` does not name
  (`fn next_word(line: StrView) changes pos -> StrView from src:` returning a view of `line`), E3062:
  "`next_word` returns a view of `line`, but its first line says `from src`; return a view of `src`, or
  write `-> StrView from line`".
- Two words kept from a `next_word` whose line has no `from` (`fn next_word() changes pos -> StrView:`),
  in `main`'s lines `a = p.next_word()` (line 18), `b = p.next_word()` (line 19), `println(a, b)` (line
  20), E3021: "`p.next_word()` (line 19) changes `p.pos`, and `a` (line 18) points into all of `p` (with
  no `from`, a method's result points into its object) and is used on line 20; if `next_word`'s result
  points only into the text, write `fn next_word() changes pos -> StrView from src:`; or use `a` before
  line 19".
- A parameter and a field of one name (`next_word(src: StrView)` with `from src`), E2118: "`src` is
  both a parameter of `next_word` and a field of `Parser`, so `from src` could mean either; rename the
  parameter: for a view of the field write `fn next_word(line: StrView) changes pos -> StrView from src:`,
  for a view of the parameter `fn next_word(line: StrView) changes pos -> StrView from line:`".
- `pool = cast this.pools[k] as Pool[T]` (no `ref mut`) followed by `return ref mut pool.dense[...]`,
  E3023: "`pool` reads the pool in `this.pools` and cannot change it; to return a changeable element,
  write `pool = ref mut cast this.pools[k] as Pool[T]`" (R14: a plain name bound from a cast of a place
  is a read-only view of it, never a hand-over).
- The same in `put`'s kind test, `if this.pools[k] is Pool[T] as pool:` followed by
  `pool.dense[pool.sparse[e]] = c`, E3023 (R14): "`pool` reads the pool in `this.pools` and cannot
  change it; to store the component through it, write `if this.pools[k] is Pool[T] as ref mut pool:`".
- Today's "not sure" downcast, `if Some(pool) = cast this.pools[k] if Pool[T]:`, E1010 (R14, verdict
  27): "`cast this.pools[k] if Pool[T]` is not Ember: to run a block when the pool is a `Pool[T]`,
  write `if this.pools[k] is Pool[T] as ref mut pool:`; to stop the program when it is not one, write
  `pool = ref mut cast this.pools[k] as Pool[T]`". `cast this.pools[k] when Pool[T]`, the spelling
  verdict 27 removed, gets the same message with its own words in the first quote.
- A kind test in a method whose first line does not say it changes `pools`, `fn put[T](k: u128, e:
  int, owns c: T):` with the same body, E2105: "`put` changes `this.pools` through `pool`; write its
  first line as `[T]` above `fn put(k: u128, e: int, owns c: T) changes pools:`".
- A change of `reg.pools` while the returned component is used, in the lines
  `p = ref mut reg.get_mut[int](3)` (line 1), `reg.pools.clear()` (line 2), `p += 1` (line 3), E3022:
  "`p` is the only name for a component in `reg.pools` until its last use on line 3, and line 2 empties
  `reg.pools`; finish with `p` first: move `reg.pools.clear()` after line 3".
- `@borrows(...)` on its own line, E0104: "`@borrows` is gone: say it after the result type:
  `fn next_word() changes pos -> StrView from src:`".

**7. Cost:** none for `from` (compile time only). The downcast: one type-id compare per get, and the
same one compare per kind test in `put`. Last
run's faithful model measured it at 1.223 to 1.244 of plain C, against safe Rust's `downcast_mut` at
1.571 to 1.635 (`tasks/96/redesign/phase3b-references-attack.md`, section 3.2 table, lines 282-287;
provisional, measured while other agents ran). Bounds checks are included in those rows.

**8. Kind:** a readable spelling of Rust's guarantee (a lifetime written as `from src` on the
signature, where Rust writes `fn next_word(&mut self) -> &'a str` with `src: &'a str`); the kind test
is a readable spelling of Rust's `if let Some(pool) = any.downcast_mut::<Pool<T>>()`.

#### AN-23: a call that returns `ref mut` is not a place

**v3:** as in v2; verdicts 21 to 34 change nothing here. The E2140 message in part 6 is left as v2
wrote it: `V2/readability.md` F-02 found that its way out loses the change again, and the message
fixes are the lead's (phase 3; listed in section 6). Meaning unchanged.

**1. Intent:** add 1 to the element a function hands back, and to a map's count for a key.

**2. OLD** (`o23_call_result_place.em`):

```ember
from std.collections import Map

fn first(mut xs: Array[int]) -> ref mut int:
    return ref mut xs[0]

fn main():
    xs = [1, 2, 3]
    first(xs) += 1
    counts = Map[String, int]()
    counts.entry("a").or_default() += 1
    println(xs[0])
```

```text
error[E2140]: assignment target is not a place
 --> o23_call_result_place.em:9:5
  |
9 |     first(xs) += 1
  |     ^^^^^^^^^ this is a value, not somewhere to store one
  |

error[E2140]: assignment target is not a place
  --> o23_call_result_place.em:11:5
   |
11 |     counts.entry("a").or_default() += 1
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this is a value, not somewhere to store one
   |

error: could not compile due to 2 error(s)
exit=1
```

The result type says `ref mut int`, a place; the message says it is a value, and gives no line.

**3. NEW:**

```ember
from std.collections import Map

fn first(ref mut xs: Array[int]) -> ref mut int:
    return ref mut xs[0]

fn main():
    xs = [1, 2, 3]
    first(ref mut xs) += 1
    counts = Map[String, int]()
    counts.entry("a").or_default() += 1
    println(xs[0], counts["a"])
```

Rule (R6): a call whose result type is `ref mut T` is a place for the statement it is in; `=`, `+=` and
a method whose first line says `changes <field>` or `changes this` may be applied to it. Bound to a
name it is written `f = ref mut first(ref mut xs)` (R2). `counts.entry("a")` is a call on an object and
carries no word (R4).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `xs` (parameter) | `ref mut`: the caller's list | `ref mut`: yes |
| `xs` (local) | none (default: own local) | none (default: own local); `ref mut xs` at the call says `first` changes it |
| `counts` | none (default: own local) | none (default: own local) |

**5. Means exactly** (`m23_bound_first.em`):

```ember
from std.collections import Map

fn first(mut xs: Array[int]) -> ref mut int:
    return ref mut xs[0]

fn main():
    xs = [1, 2, 3]
    f = first(xs)
    f += 1
    counts = Map[String, int]()
    c = counts.entry("a").or_default()
    c += 1
    println(xs[0], counts["a"])
```

```text
===== m23_bound_first.em
exit=0
```

`f = first(xs)`, `f += 1` and `c = counts.entry("a").or_default()`, `c += 1` are today's spelling of
NEW's `f = ref mut first(ref mut xs)` and `c = ref mut counts.entry("a").or_default()` (R2); the NEW
block writes the call as the place itself.

**6. Still refused:**
- A call whose result is a new value, `make_point().y = 1` or `first_value(xs) += 1` (with
  `fn first_value(xs: Array[int]) -> int:`), E2140 (today's code, "assignment target is not a place", in
  the OLD output above; R8 (d)): "`first_value(xs)` gives back a new `int`, a copy of the element, so
  adding to it would be lost; to add 1 to the element in `xs` write `xs[0] += 1`, or let the function
  give back the element itself: `fn first(ref mut xs: Array[int]) -> ref mut int:` with
  `return ref mut xs[0]`, called `first(ref mut xs) += 1`".
- `first(xs) += 1` with no word at the call, E3028: "`first` may change `xs` (its first line says
  `ref mut xs: Array[int]`); say so at the call: `first(ref mut xs) += 1`".
- The element written through another name while a `ref mut` name of it is used, in the lines
  `f = ref mut first(ref mut xs)` (line 1), `xs[0] = 5` (line 2), `f += 1` (line 3), E3022 (R8 (f)):
  "`f` is the only name for `xs[0]` until its last use on line 3, and line 2 writes `xs[0]`; write it
  through `f`: `f = 5`".

**7. Cost:** none: the same address and store as the bound form.

**8. Kind:** a readable spelling of Rust's guarantee (Rust writes `*first(&mut xs) += 1`).

#### AN-24: structs holding references (A6): a view struct cannot be stored in an `Array`

**v3:** as in v2; verdicts 21 to 34 change nothing here. The list `vs` is an own local list that lives
no longer than `reg` (R2, R7); each kept `ref` view promises that the registry does not change while it
is used (R8 (g), verdict 5); holders that may all change one registry write `sharedref mut Registry`
(R7; verdict 20 keeps the spelling). Meaning unchanged.

**1. Intent:** keep a list of views that each read one entity of one registry, and sum their hit
points.

**2. OLD** (`o24b_views_in_local_array.em`; phase 1's `e06d` with `ref mut` is `o24a`, same E3063):

```ember
struct Registry:
    hp: Array[int]

struct View:
    reg: ref Registry
    entity: int

    fn hp(self) -> int:
        return self.reg.hp[self.entity]

fn main():
    reg = Registry(hp=[10, 20])
    vs: Array[View] = []
    vs.push(View(reg=ref reg, entity=0))
    vs.push(View(reg=ref reg, entity=1))
    total = 0
    for v in vs:
        total += v.hp()
    println(total)
```

```text
error[E3063]: `View` is a view, so it may not be stored in an `Array` unless it is `static`
  --> o24b_views_in_local_array.em:15:13
   |
15 |     vs.push(View(reg=ref reg, entity=0))
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^ stored here
   |
   = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
   = note: the view borrows `reg`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error[E3063]: `View` is a view, so it may not be stored in an `Array` unless it is `static`
  --> o24b_views_in_local_array.em:16:13
   |
16 |     vs.push(View(reg=ref reg, entity=1))
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^ stored here
   |
   = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
   = note: the view borrows `reg`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error: could not compile due to 2 error(s)
exit=1
```

Rust compiles the twin (`Vec<View<'a>>`, last run's `r06a`).

**3. NEW:**

```ember
struct Registry:
    hp: Array[int]

struct View:
    reg: ref Registry
    entity: int

    fn hp() -> int:
        return this.reg.hp[this.entity]

fn main():
    reg = Registry(hp=[10, 20])
    vs: Array[View] = []
    vs.push(View(reg=ref reg, entity=0))
    vs.push(View(reg=ref reg, entity=1))
    total = 0
    for v in vs:
        total += v.hp()
    println(total)
```

Rule (R7): a value that holds views, a local list of views included, lives no longer than what they
point into, whoever owns it (one limit per view field of the element type, covering every element
pushed, as Rust's single `'a`; erasing to `owns dyn I` keeps the limit). Nothing is written for a local
list; a field of a class holding such a list says its source with `from <let field>` (AN-25). The list
itself is `main`'s own (R2) and may be pushed to; what its views read does not change while they are
used (R8 (g)).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `hp`, `entity` | none (default: the struct's own values) | none (default: field) |
| `reg` (field) | `ref`: a read-only view of a registry | `ref`: no |
| `this` in `hp` | `this`: the view `hp` was called on | no `changes`: no (R5) |
| `reg` (local) | none (default: own local) | none (default: own local) |
| `vs` | none (default: own local list; its element type `View` holds a `ref`, so the list lives no longer than `reg`, R7) | none (default: own local) |
| `total` | none (default: own local) | none (default: own local) |
| `v` | none (default: read-only view of each element) | none (default: never) |

**5. Means exactly:** today's Ember cannot store a view in an `Array`; in C (`c24_views_list.c`):

```c
typedef struct { int64_t *hp; size_t len; } Registry;
typedef struct { const Registry *reg; int64_t entity; } View;
typedef struct { View *ptr; size_t len, cap; } ViewList;
...
    Registry reg = { hp, 2 };                            /* main's own registry */
    ViewList vs = {0};                                   /* may not outlive reg: checked when compiling */
    push(&vs, (View){ &reg, 0 });
    push(&vs, (View){ &reg, 1 });
    for (size_t i = 0; i < vs.len; i++) total += view_hp(vs.ptr[i]);
    free(vs.ptr);                                        /* the list ends before reg */
```

```text
$ ./c24.exe
30
c24_views_list ms=0.018 peak_kb=4064 checksum=30
```

Under AddressSanitizer (`c24_asan.exe`): `30`, `c24_views_list ms=0.040 peak_kb=8148 checksum=30`, no
report.

**6. Still refused:**
- Two kept `ref mut` views of one registry, with `struct Editor:` holding `reg: ref mut Registry` and
  `entity: int`, a list `eds: Array[Editor] = []`, and the lines
  `eds.push(Editor(reg=ref mut reg, entity=0))` (line 1), `eds.push(Editor(reg=ref mut reg, entity=1))`
  (line 2), E3022: "`reg` is lent to `eds[0].reg`, a `ref mut` field, for as long as `eds` is used, and
  line 2 lends it to `eds[1].reg` too; if both may change the registry, write
  `reg: sharedref mut Registry` in `Editor`, and `Editor(reg=sharedref mut reg, entity=0)` and
  `Editor(reg=sharedref mut reg, entity=1)` on lines 1 and 2".
- With `reg: sharedref mut Registry`, a view into a part used after another holder reshapes it, in the
  lines `s = eds[0].reg.hp.as_span()` (line 1), `eds[1].reg.hp.push(5)` (line 2), `println(s[0])` (line
  3), E3060: "`eds[1].reg.hp.push(5)` (line 2) may move the list `s` points into (`eds[0].reg` and
  `eds[1].reg` may be the same registry), and `s` is used on line 3; use `s` before line 2, or read the
  number again after it: write `println(eds[0].reg.hp[0])` in place of line 3".
- A change of the registry while the views that read it are still used, in the lines
  `vs.push(View(reg=ref reg, entity=1))` (line 1), `reg.hp[0] = 5` (line 2), `for v in vs:` (line 3),
  E3021 (R8 (g)): "`reg.hp[0] = 5` (line 2) changes the registry that `vs[1].reg` reads
  (`View(reg=ref reg, entity=1)`, line 1), and `vs` is used on line 3; change it after `vs`'s last use,
  or, if the registry must change while the views are kept, write `reg: sharedref mut Registry` in
  `View` and `View(reg=sharedref mut reg, entity=1)`".
- The list kept longer than `reg` (returned from `fn views() -> Array[View]:`, which makes
  `reg = Registry(hp=[10, 20])` and `vs` as above and ends with `return vs`; or stored in a field with no
  `from`, or sent to a thread), E3063: "`vs` holds views of `reg`, which ends when `views` returns, so it
  may not be returned; make the registry in the caller and give it to `views`: write
  `fn views(reg: Registry) -> Array[View] from reg:` without the line that makes `reg`, and call it
  `reg = Registry(hp=[10, 20])`, then `vs = views(reg)`".

**7. Cost:** none beyond plain C: the C above is the layout; the limit exists only when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (`Vec<View<'a>>` with nothing written).

#### AN-25: self-referential data (A7): a value owning its text and the word views into it

**v3:** as in v2 (`fn init(owns text: String):`, verdict 19; `copy of d` refused with E2116, verdict
12); verdicts 21 to 34 change nothing here. Meaning unchanged.

**1. Intent:** a `Document` holds its text and the list of its words, without copying each word.

**2. OLD** (`o25a_document.em`, phase 1's `e07`; `o25b_document_one_view.em` shows the rule under it):

```ember
struct Document:
    text: String
    words: Array[str]

fn load(owned text: String) -> Document:
    words: Array[str] = []
    for w in text.split_whitespace():
        words.push(w)
    return Document(text=text, words=words)
```

```text
error[E3063]: `str` is a view, so it may not be stored in an `Array` unless it is `static`
 --> o25a_document.em:9:20
  |
9 |         words.push(w)
  |                    ^ stored here
  |
  = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
  = note: the view borrows `text`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error: could not compile due to 1 error(s)
exit=1
```

```text
error[E3024]: `text` would be moved into this value while another of its fields borrows it
 --> o25b_document_one_view.em:9:12
  |
9 |     return Document(text=text, first=w)
  |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `text` and a view of it would both be fields of this value
  |
  = help: store an index or a `Handle` into the owner instead of a view of it, or keep the owner outside the value
  = note: a field cannot borrow another field of the same value: moving the value would move what the view points into (TYP-15)
```

(`o25b` also prints an E3060 for the same line; the full output is in `check_output.txt`.)

**3. NEW:**

```ember
class Document:
    let text: String
    words: Array[StrView] from text

    fn init(owns text: String):
        this.text = text
        this.words = this.text.split_whitespace().collect()

fn main():
    d = Document(String("the quick fox"))
    println(d.words.len())
```

The rules (section 2, ruling 2; R7):
1. `from text` names a `let` field of the same value whose contents live in their own heap block: a
   `String` or an `Array`. Today's runtime gives both the layout `{ptr, len, cap}` with the contents
   outside the header (`runtime/ember_rt/include/ember_rt.h:2022-2026`, `ember_vec`; `String` "is the
   same buffer"), and ADR-141's frame buffer is only for a list that never leaves its function, which a
   field's contents always do. So moving the value moves the header, never the characters. If `String`
   ever gains an inline small-string form, `from` on a `String` field must be refused.
2. `let` means nothing is changed through `text` after `init` (the vocabulary's `let` row), so the
   words stay valid for the value's whole life. Last run had to make the views unusable after every
   write to `text` and was attacked through an index write (`docs[i].text.push_str(...)` then
   `docs[j].words`, attack report section 5); with `let` there is no write to track, and that attack has
   nothing to hit.
3. `words` takes only views into `text` (or text written in the program).
4. A view read out (`w = d.words[0]`) is a view of `d`: `d` may not be handed over or ended while `w`
   lives (last run's attack fix 1), nor changed through another name (R8 (g)).
5. It works for a `struct` too (`let` on a struct field is accepted today: run 3's
   `p_let_struct_field.em`, `exit=0`); a struct literal fills `text` before computing `words`.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `text` (field) | `String`: the document's own text | `let`: never after `init` |
| `words` | `StrView` elements, `from text`: views into `text` | none (default: field; changed through `changes words` or `changes this`) |
| `this` in `init` | `this` in `init`: the object being built | `init` sets its fields with no word (R5) |
| `text` (parameter) | `owns`: handed over by the caller | `owns`: it is now `init`'s, handed on into the field |
| `d` | none (default: own local) | none (default: own local) |

**5. Means exactly** (C, `c25_document.c`):

```c
typedef struct { char *ptr; size_t len, cap; } String;
typedef struct { const char *ptr; size_t len; } StrView;
typedef struct { StrView *ptr; size_t len, cap; } Words;
typedef struct { String text; Words words; } Document;     /* a class object: on the heap */

static Document *document_new(String owned_text) {        /* init(self, owned text: String) */
    Document *d = calloc(1, sizeof *d);
    d->text = owned_text;                                   /* handed over: the block does not move */
    ... split on spaces, push (ptr, len) views into d->text.ptr ...
}
static void document_drop(Document *d) { free(d->words.ptr); free(d->text.ptr); free(d); }
```

```text
$ ./c25.exe
3 words, first=the
c25_document ms=0.016 peak_kb=4056 checksum=204454
```

Under AddressSanitizer (`c25_asan.exe`): `3 words, first=the`, `c25_document ms=0.049 peak_kb=8172
checksum=204454`, no report.

**6. Still refused:**
- `from` naming a field that is not `let`, E2118: "`words` points into
  `text`, so `text` must never change after `init`; write `let text: String`".
- A change to `text` after `init` (`this.text += "!"` in a method), E2104: "`text` is `let`, because
  `words` points into it: it is set in `init` and never changed after; for other text, make a new
  document: `d2 = Document(String("the slow fox"))`".
- `this.words.push(other_word)` with a view of something else, E3063: "`words` holds views of `text`
  only, and `other_word` points into `line`; keep your own copies: write the field as
  `words: Array[String] = []`, in `init` write `for w in this.text.split_whitespace():
  this.words.push(String(w))` in place of `this.words = this.text.split_whitespace().collect()`, and
  write `this.words.push(String(other_word))` in place of this line".
- `d2 = copy of d`, E2116: "`d.words` points into `d.text`, so a copy's words would point into `d`'s
  text; make the copy from its text: `d2 = Document(String(d.text))`" (re-pointing each view, last run's
  rule 7, stays a good part noted in run 3).
- `from` on a fixed array or an inline struct field, E2118: "`buf` is
  stored inside the value, so moving the value moves it; keep the text in a `String`, which has a block
  of its own: write the field as `buf: String`".

**7. Cost:** none beyond plain C: the C above is the layout (pointer and length per word, no count, no
check). Last run measured this layout against Rust's index-range workaround (1.205 of plain C) and
today's one-`String`-per-word (1.451 time, 2.122 memory) (`phase3b-references.md`, section 5.4 table;
provisional; the attack report section 5 notes that the Ember program also pays bounds checks, about
1.046 of plain C, which every Ember program pays).

**8. Kind:** a smarter compile-time solution (Rust refuses self-reference; Ember accepts it because the
source is `let` and its contents never move).

#### AN-26: `str` is a borrowed view while Python's `str` is the owned string

**v3:** as in v2; verdicts 21 to 34 change nothing here. Meaning unchanged.

**1. Intent:** keep a player's name in the player.

**2. OLD** (`o26a_str_field_literal.em`, `o26b_str_field_string.em`, `o26c_str_return.em`):

```ember
class Player:
    name: str

    fn init(self, name: str):
        self.name = name

fn make(s: String) -> Player:
    return Player(s)
```

```text
===== o26a_str_field_literal.em
exit=0
===== o26b_str_field_string.em
error[E3063]: `str` is a view, so it may not be passed to `init`, which stores it where only a `static` view may go
 --> o26b_str_field_string.em:8:19
  |
8 |     return Player(s)
  |                   ^ stored by this call
  |
  = help: store an owned copy — `String` for `str`, `Array[T]` for `Span[T]` — and note that costs one allocation per element; or store a `u32` index or a `Handle[T]` and name the container it indexes
  = note: the view borrows `s`, and this place has no bounding region: only a view with the `static` region, such as a string literal, may be stored in it (TYP-15, LT-3)

error: could not compile due to 1 error(s)
exit=1
```

The same class compiles when fed a literal (`o26a`) and is refused when fed a `String` (`o26b`): the
declaration `name: str` does not say which. And `fn first(xs: Array[String]) -> str:` (`o26c`) does
not say the result points into `xs`; the conflict shows only at a later `push` (E3021, full output in
`check_output.txt`).

**3. NEW:**

```ember
class Player:
    name: String
    kind: StrView from static

    fn init(name: StrView):
        this.name = String(name)
        this.kind = "human"

fn make(s: String) -> Player:
    return Player(s)
```

`str` is spelled `StrView`. A `StrView` field of a class says what it points into: `from <let field>`
(AN-25) or `from static` (text written in the program) (R7). A function returning `StrView` has the
default source (R6), or `from`. In `make`, `s`, a `String`, is read as a `StrView` of its text where
`init` wants one (AN-29), and `init` keeps its own copy.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `name` (field) | `String`: the player's own text | none (default: field) |
| `kind` | `StrView from static`: a view of text written in the program | none (default: field) |
| `this` in `init` | `this` in `init`: the object being built | `init` sets its fields with no word (R5) |
| `name` (parameter) | `StrView`: a view of the caller's text | none (default: read-only parameter) |
| `s` | none (default: read-only parameter) | none (default: never) |

**5. Means exactly** (`m26_name_string.em`):

```ember
class Player:
    name: String
    kind: str

    fn init(self, name: str):
        self.name = name.to_string()
        self.kind = "human"

fn make(s: String) -> Player:
    return Player(s)

fn main():
    p = make("ann".to_string())
    println(p.name, p.kind)
```

```text
===== m26_name_string.em
exit=0
```

**6. Still refused:**
- `str` anywhere, E1010: "`str` is now `StrView`, a view into someone else's text; for the player's own
  text write `String`".
- `name: StrView` in a class with no `from`, E2118: "a `Player` keeps
  `name` after the text it came from may be gone; to keep its own copy write `name: String` and
  `this.name = String(name)`".
- A view of something else stored in a `from static` field (`this.kind = name`), E3063: "`kind` holds
  only text written in the program, and `name` comes from the caller; write `this.kind = String(name)`
  with `kind: String`".
- `o26c`'s `push` while `n` lives, E3060: "`names.push(...)` may move the strings `n` points into
  (`first` returns a view of `xs`, here `names`), and `n` is used after it; use `n` before the `push`,
  or keep a copy of your own: `n = String(first(names))`".

**7. Cost:** none for the spelling. `String(name)` is one allocation and copy, the same as Python's
own `str` object and today's `to_string()`.

**8. Kind:** a readable spelling of Rust's guarantee (`&str` vs `String`, `&'static str`).

#### AN-27: `ref T` / `ref mut T` as a parameter's type

**v3:** as in v2: the call says `bump(ref mut a)` (R4). Verdict 32 is the rule this answer already
follows: `ref mut n` in front of the name, not `ref mut` in the type, sets what `bump` may do. Meaning
unchanged.

**1. Intent:** `bump(n)` adds 1 to the caller's number.

**2. OLD** (`o27a_ref_mut_type.em`, `o27b_help_form.em`, `o27c_ref_type.em`, reader 1's t3, t3c, t6):

```ember
fn bump_mode(mut x: int):
    x += 1

fn bump_ref(x: ref mut int):
    x += 1
```

```text
error[E3023]: cannot mutate borrowed parameter `x`
 --> o27a_ref_mut_type.em:6:5
  |
5 | fn bump_ref(x: ref mut int):
  |             -------------- `x` is borrowed here
6 |     x += 1
  |     ^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut x: ref mut i64`
  = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
  = help: if the value has identity, make it a `class`
  = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

error[E2020]: expected `ref mut i64`, found `i64`
  --> o27a_ref_mut_type.em:11:14
   |
 5 | fn bump_ref(x: ref mut int):
   |             -------------- parameter `x` is declared `ref mut i64` here
...
11 |     bump_ref(n)
   |              ^ this is `i64`
   |
   = help: borrow it explicitly: `ref mut` before the place (TYP-5 rule 7 borrows only as `ref`)

error: could not compile due to 2 error(s)
exit=1
```

The help's own form does not compile (`o27b`, `fn bump_ref(mut x: ref mut int)`):

```text
error[E2020]: `+=` is not defined on `ref mut i64`
 --> o27b_help_form.em:3:5
  |
3 |     x += 1
  |     ^^^^^^
  |
  = note: a type has `+=` by implementing `AddAssign`, or `Add` with its own type as `Output` [TYP-21]

error[E3027]: a `mut` argument is not a mutable place
 --> o27b_help_form.em:7:14
  |
7 |     bump_ref(ref mut n)
  |              ^^^^^^^^^ this value has no mutable storage for the callee to borrow
  |
  = help: bind the value to a local first, or pass the owner place directly (`f(obj.field)` rather than `f(obj.get_field())`)
  = note: the callee's declared `mut` mode forms the mutable borrow; the call site does not write a mode [FN-2a]

error: could not compile due to 2 error(s)
exit=1
```

And `fn show_ref(x: ref int)` means the same as `fn show_mode(x: int)` (`o27c`, `exit=0`).

**3. NEW:**

```ember
fn bump(ref mut n: int):
    n += 1

fn main():
    a = 1
    bump(ref mut a)
    println(a)
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `n` | `ref mut`: the caller's number | `ref mut`: yes |
| `a` | none (default: own local) | none (default: own local); `ref mut a` at the call says `bump` changes it |

**5. Means exactly** (`m27_bump.em`; today's call writes no word):

```ember
fn bump(mut n: int):
    n += 1

fn main():
    a = 1
    bump(a)
    println(a)
```

```text
===== m27_bump.em
exit=0
```

**6. Still refused:**
- `fn bump(n: ref mut int)`, E0117: "the mode goes on the name: write `ref mut n: int`".
- `fn show(x: ref int)`, E0117: "the mode goes on the name, and a plain parameter already reads the
  caller's value without copying it: write `x: int`".
- `fn bump(mut n: int)`, E0113: "`mut n` on a parameter: to change the caller's `n` through `n`, write
  `ref mut n: int`".
- `bump(a)`, E3028: "`bump` may change `a` (its first line says `ref mut n: int`); say so at the call:
  `bump(ref mut a)`".
- `bump(ref mut a + 1)`, E3027: "`a + 1` is a new number, not a place `bump` can change; keep it in a
  name first: `b = a + 1`, then `bump(ref mut b)`".

**7. Cost:** none (passed by address, as today's `mut n`).

**8. Kind:** a readable spelling of Rust's guarantee (`&mut i64`), on the name and at the call.

#### AN-28: `r = ref mut b` on a `ref mut` local copies `b` into `r`'s first target

**v3:** NEW as in v2. The prose's `first_even` takes its span as `ref mut ps: MutSpan[int]` (verdict
34: a `MutSpan` parameter that gives write access to its elements says `ref mut`); the prose now says
how a name holding a `MutSpan` is pointed at another span (R2), and part 6 refuses the plain parameter.
**Meaning changed:** yes, by verdict 42 (the owner: "yes, every ref mut name"): the OLD's own line,
`r = ref mut b`, is accepted and points `r` at `b`, which is what the line reads as; v3 had refused it.
`const ref mut` is the spelling for a name that is never given another ref. And by verdict 34: v2's
`fn first_even(ps: MutSpan[int]) -> ref mut int:` is refused (E3023); `fn first_even(ref mut ps:
MutSpan[int]) -> ref mut int:` means what it meant.

**1. Intent:** keep a changeable reference to `a`, later work on `b` too.

**2. OLD** (`o28_ref_mut_rebind.em`, 05's `b2`):

```ember
fn main():
    a = 1
    b = 2
    r = ref mut a
    r = ref mut b
    println(a, b)
```

```text
===== o28_ref_mut_rebind.em
exit=0
```

Accepted; by `[BRW-1]` (spec 2286-2287) the second line writes `b`'s value into `a`, so it would
print `2 2` (read from the spec, not run), where the line reads as "`r` now refers to `b`".

**3. NEW:**

```ember
fn main():
    a = 1
    b = 2
    r = ref mut a
    r = 20
    r = ref mut b
    r = 30
    println(a, b)
```

A `ref mut` name stands for the place it was made from until it is given another ref (R2, verdict 42):
`r = 20` writes into `a`; `r = ref mut b` points `r` at `b`, the line saying `ref mut` again; `r = 30`
then writes into `b`. It prints `20 30`. `a` is free of `r` from the line that gives `r` another ref, and
`println(a, b)` reads both after `r`'s last use, as R8 (f) asks. A name that must keep its place for
its whole scope says so: `r = const ref mut a`, after which `r = ref mut b` is refused (part 6). The same holds for a call whose result is
`ref mut T`: with `fn first_even(ref mut ps: MutSpan[int]) -> ref mut int:`,
`r = ref mut first_even(ref mut xs.as_mut_span())` then `r = 7` writes the element (R2; the parameter
says `ref mut`, since what it gives back may write its elements, and the call says `ref mut` before
the `MutSpan` argument, R4, verdict 34).

A name that holds a `MutSpan` follows the same rule: `d = ref mut xs.as_mut_span()` then
`d = ref mut ys.as_mut_span()` points `d` at the second span; nothing is written into `xs`, and
`ref mut` on each line says that `d` may write the elements of the span it then holds (R2, verdicts 34
and 42).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `a`, `b` | none (default: own local) | none (default: own local) |
| `r` | `ref mut`: a name for `a`, then, after `r = ref mut b`, for `b` (the second `ref mut` says so) | `ref mut`: yes |

**5. Means exactly** (`m28_two_refs.em`, today's spelling with a second name, `rb = ref mut b`, since
today's `r = ref mut b` writes `b`'s value into `a`):

```text
===== m28_two_refs.em
exit=0
```

**6. Still refused:**
- `r = ref mut b` on a `const ref mut` name (`r = const ref mut a`), E3023: "`r` is `const ref mut a`,
  so it always refers to `a`; to refer to `b` use a new name, `rb = ref mut b`, or make `r` with plain
  `ref mut`: `r = ref mut a`". And `r = ref b` on a `ref mut` name, E3023: "`r` may change what it
  refers to, and `ref b` only reads `b`; to point `r` at `b` write `r = ref mut b`, or for a read-only
  name `rb = ref b`".
- `a` read through its own name while `r` is still used, with `println(a)` as line 5 between
  `r = ref mut a` (line 4) and `r = 20` (line 6), E3021 (R8 (f)): "`r` is the only name for `a` until its
  last use on line 6, and line 5 reads `a`; read it through `r`, `println(r)`, or read `a` after line 6".
- A plain `MutSpan` parameter whose element is given back to be written, `fn first_even(ps:
  MutSpan[int]) -> ref mut int:`, E3023 (R4, R5, verdict 34): "`first_even` gives back an element of
  the caller's span to be changed, but `ps` only reads it; write `fn first_even(ref mut ps:
  MutSpan[int]) -> ref mut int:` and call it `first_even(ref mut xs.as_mut_span())`".

Accepted: `n = p.name` then `n = q.name` makes `n` view `q.name`, checked by R8 against its new target
(R2, G-08); `d = ref mut xs.as_mut_span()` then `d = ref mut ys.as_mut_span()` points `d` at the second
span (part 3).

**7. Cost:** none.

**8. Kind:** an understandable limitation kept (a `ref mut` name of a place is never re-pointed), with
the one misleading form refused.

#### AN-29: `ref String` is not accepted where `str` is expected at a call

**v3:** NEW as in v2. The reason it is accepted is now verdict 22's: a plain `fn() -> int` changes
nothing at all, captured variables included, and this lambda only reads the real `n` it captured
(v2 counted the real `n` as part of the function itself; it is not: only what a lambda captures with a
word is its own state, R12). Part 6 adds the refusal of a lambda that changes a variable it captured,
given to `apply`. Meaning unchanged.

**1. Intent:** pass a list's string element to a function that reads text, from inside a lambda.

**2. OLD** (`o29b_ref_string_in_lambda.em`):

```ember
fn shout(s: str) -> int:
    return s.len()

fn apply(f: fn() -> int) -> int:
    return f()

fn main():
    names: Array[String] = ["ann", "bob"]
    total = 0
    for n in names:
        total += apply(fn() => shout(n))
    println(total)
```

```text
error[E2020]: expected `str`, found `ref String`
  --> o29b_ref_string_in_lambda.em:12:38
   |
 2 | fn shout(s: str) -> int:
   |          ------ parameter `s` is declared `str` here
...
12 |         total += apply(fn() => shout(n))
   |                                      ^ this is `ref String`
   |

error: could not compile due to 1 error(s)
exit=1
```

The same call outside a lambda is accepted (`o29_ref_string_to_str.em`, `total += shout(n)`,
`exit=0`): one rule applied in one place and not the other.

**3. NEW:**

```ember
fn shout(s: StrView) -> int:
    return s.len()

fn apply(f: fn() -> int) -> int:
    return f()

fn main():
    names: Array[String] = ["ann", "bob"]
    total = 0
    for n in names:
        total += apply(fn() => shout(n))
    println(total)
```

Rule: a `String` place or a `ref String` reads as a `StrView` of its text wherever a `StrView` is wanted,
inside a lambda as everywhere; the view points into that `String`. The lambda uses the real `n` (a
capture not named is the real variable, row 6, R12), only reads it, and ends before `n` does, so it uses
`n` at its own address (R12). Its type is a plain `fn() -> int`: it changes nothing at all, the
variables it captured included; reading the real `n` is not a change (R12, verdict 22). Had it changed
a variable it captured, its first line would name it (`fn() changes seen -> int:`), and a plain
`fn() -> int` parameter would not take it (part 6). `apply`'s `f` only reads, which is allowed because
this function value changes no state of its own (R12).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `s` | `StrView`: a view of the caller's text | none (default: read-only parameter) |
| `f` | none (default: a read-only function-value parameter) | none (default: never; its type `fn() -> int` changes nothing at all, so R12 refuses giving it a function value that changes anything, its own state or a captured variable) |
| `names`, `total` | none (default: own local) | none (default: own local) |
| `n` (loop name) | none (default: read-only view of each element) | none (default: never) |
| `n` (captured by the lambda) | none (default: the real `n`, not a copy; row 6, R12) | none (default: a lambda changes only what its first line names after `changes`, and this one names nothing, verdict 22) |

**5. Means exactly** (`m29_str_in_lambda.em`, `shout(n.as_str())`):

```text
===== m29_str_in_lambda.em
exit=0
```

**6. Still refused:**
- A `StrView` of a list's string kept past a change to the list, in the lines
  `head: StrView = names[0]` (line 1), `names.clear()` (line 2), `println(shout(head))` (line 3), E3060:
  "`names.clear()` (line 2) ends the text `head` points into (`names[0]`, line 1), and `head` is used on
  line 3; use `head` before line 2, or keep a copy of your own: write line 1 as
  `head = String(names[0])`".
- A lambda that also counts the names it shouts in `seen`, a variable of `main` (`seen = 0` before the
  loop), with no word on its first line, E2105 (R12, verdict 22): "this function changes `seen`, a
  variable it uses from `main`; say so on its first line: `fn() changes seen -> int:`".
- The same lambda with its first line `fn() changes seen -> int:`, given to `apply`, E2113 (R12): "this
  function changes `seen`, and `apply` takes `f: fn() -> int`, functions that change nothing; to let
  `apply` run functions that change things, write `fn apply[C, F: fn() changes C -> int](owns f: F)
  changes C -> int:`" (a change list, as the library's `for_each` takes one, verdict 21; the call
  `apply(...)` then counts as changing `seen`).

**7. Cost:** none (pointer and length, as `as_str()`).

**8. Kind:** a smarter compile-time solution (one coercion rule, as Rust's `&String` to `&str`); the
function type is a readable spelling of Rust's `Fn` against `FnMut`, with what changes named.

#### AN-30: a reference out of a lock, `RefCell` or `RwLock` guard compiles and is unsafe (A33). Agreed row 4.

**v3:** NEW adds OLD's `clear`, which changes the list under the lock, so its first line says so:
`fn clear(scores: Mutex[Array[String]]) changes Mutex[Array[String]]:` (verdict 29, R5, R12); the call
still takes no word, `clear(m)` (verdict 3). `main` now does what OLD's `main` meant (keep the best
name, clear the table, print the name), with a copy of the name. Part 6 adds the refusals verdict 29
brings: `clear` without its `changes`, and `clear(m)` inside a loop over the locked list (E3020, at
compile time instead of waiting forever for the lock). The owner's-test row for `scores` no longer
calls the lock types "R5's one exception": a plain name still uses them by their own operations, and
the first line names the change. Row 4's message is exactly as agreed. Meaning unchanged for v2's
lines; `clear` is new to NEW, and v2's rules would have taken it with no `changes`.

**1. Intent:** `top(scores)` returns the best name from a locked table; `clear(scores)` empties the
table; `main` keeps the best name, clears the table, then prints the name it kept.

**2. OLD** (`o30a_lock.em`, `o30b_lock_uaf.em`, `o30c_refcell.em`, `o30d_rwlock.em`):

```ember
from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> ref String:
    with g = scores.lock():
        return ref g[0]

fn clear(scores: Mutex[Array[String]]):
    with g = scores.lock():
        g.clear()

fn main():
    m: Mutex[Array[String]] = Mutex(["ann".to_string()])
    r = top(m)
    clear(m)
    println(r)
```

```text
===== o30a_lock.em
exit=0
===== o30b_lock_uaf.em
exit=0
===== o30c_refcell.em
warning[L3011]: a `RefCell` guard for `*c` is live across this call
 --> o30c_refcell.em:9:9
  |
8 |     with g = c.borrow_mut():
  |              -------------- guard borrowed here
9 |         g.clear()
  |         ^^^^^^^^^ call happens here
  |
  = help: drop the guard before the call, or put the call in a block of its own
  = note: a guard live across a call that re-enters the same cell panics at run time [CELL-7]

exit=0
===== o30d_rwlock.em
exit=0
```

All accepted. What `o30b` does, in C (`c30_lock_ref_escape.c`, built with AddressSanitizer):

```text
==116816==ERROR: AddressSanitizer: heap-use-after-free on address 0x126a7eaa0130 at pc 0x7ff900341f78 bp 0x00ae832feca0 sp 0x00ae832fece8
READ of size 2 at 0x126a7eaa0130 thread T0
    ...
    #8 0x7ff632242bf1 in main C:\Users\ism19\Code\ember\target\redesign3\09-area-B\c30_lock_ref_escape.c:27
0x126a7eaa0130 is located 0 bytes inside of 3-byte region [0x126a7eaa0130,0x126a7eaa0133)
freed by thread T0 here:
    #1 0x7ff632242433 in free S:\compiler-rt\lib\asan\asan_malloc_win_thunk.cpp:52
    #2 0x7ff632242b85 in main C:\Users\ism19\Code\ember\target\redesign3\09-area-B\c30_lock_ref_escape.c:26
previously allocated by thread T0 here:
    #1 0x7ff632242493 in malloc S:\compiler-rt\lib\asan\asan_malloc_win_thunk.cpp:64
    #2 0x7ff632242a9d in main C:\Users\ism19\Code\ember\target\redesign3\09-area-B\c30_lock_ref_escape.c:23
```

(Line 26 is `clear`'s `free`, inlined; line 27 is `println(r)`.)

**3. NEW:**

```ember
from std.sync import Mutex

fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return copy of g[0]

fn top_len(scores: Mutex[Array[String]]) -> int:
    with g = scores.lock():
        return g[0].len()

fn clear(scores: Mutex[Array[String]]) changes Mutex[Array[String]]:
    with g = scores.lock():
        g.clear()

fn main():
    m: Mutex[Array[String]] = Mutex([String("ann")])
    best = top(m)
    n = top_len(m)
    clear(m)
    println(best, n)
```

Rule (R8 (e)): a view made through a guard (`g[0]`, `ref g[0]`, a slice of `g`) points into the guard,
and the guard ends with its `with` block, so such a view cannot leave it. The same for `RwLock`'s
`read()` and `write()`. `best` is a `String` of `main`'s own, so `clear(m)` cannot end it; OLD's
`println(r)` after `clear(m)` read freed text. `top(m)` and `clear(m)` carry no word: a `Mutex` exists
to be shared, and its own "only me" is the lock's `with` block (R4, R5, verdict 3).

`clear` changes the list inside the `Mutex` under its lock, and its first line says so by the type of
what changes, `changes Mutex[Array[String]]`, as every change is said (R5, R12, verdict 29); `top` and
`top_len` only read under the lock and name nothing. `main` names nothing for `clear(m)`: `m` is its
own local, and a change to an own local needs no clause (R5). What the clause buys is at the call: a
`clear(...)` while a loop walks a list in a `Mutex[Array[String]]`, under that lock, is refused when
compiling (part 6), where it would otherwise wait for the lock this thread holds, forever.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `scores` (in `top`, `top_len`) | none (default: a read-only parameter) | none (default: never directly); its type, `Mutex`, is used through any name by its own operation, `lock()` (R5), and these two only read under it: their first lines name no change |
| `g` (in `top`, `top_len`) | `lock()`: the guard, the list while locked | `lock()`: yes, while the `with` block lasts; these two only read through it |
| `scores` (in `clear`) | none (default: a read-only parameter) | `changes Mutex[Array[String]]` on `clear`'s first line: it changes the list in the `Mutex` under its lock (verdict 29); the change is made through `lock()` (R5) |
| `g` (in `clear`) | `lock()`: the guard, the list while locked | `lock()`: yes, while the `with` block lasts; `g.clear()` is the change the first line names |
| `m` | none (default: own local, made by `Mutex(...)`) | none (default: own local; its type, `Mutex`, is changed through `lock()`) |
| `best` | none (default: own local; `top` hands back a new `String`, `copy of g[0]`, R2) | none (default: own local) |
| `n` | none (default: own local; a number) | none (default: own local) |

**5. Means exactly** (`m30_lock_copy.em`):

```ember
from std.sync import Mutex, RwLock

fn top(scores: Mutex[Array[String]]) -> String:
    with g = scores.lock():
        return g[0].clone()

fn top_read(scores: RwLock[Array[String]]) -> String:
    with g = scores.read():
        return g[0].clone()

fn top_len(scores: Mutex[Array[String]]) -> int:
    with g = scores.lock():
        return g[0].len()
```

```text
===== m30_lock_copy.em
exit=0
```

`clear` means exactly OLD's `clear` (today's Ember writes no clause, `o30a_lock.em`, `exit=0` above;
the `changes` clause exists only when compiling), and `main` is OLD's `main` with `top` giving a copy,
as `m30_lock_copy.em`'s `top` does. No new probe was checked for the whole program: this makes no new
claim about today's Ember.

**6. Still refused:**
- `return ref g[0]` inside `with g = scores.lock():`, E3063: "this points into the locked list, which
  can change once the lock ends; return a copy, or do the work inside the `with` block" (row 4,
  exactly).
- The same under `scores.read()` or `scores.write()` of an `RwLock`, E3063: "this points into the
  read-locked list, which can change once the lock ends; return a copy, or do the work inside the
  `with` block".
- A view of `g` stored in an outer name (`best = ref g[0]`, then `best` used after the block), E3063:
  "`best` points into the locked list, which can change once the lock ends with the `with` block; keep
  `best = copy of g[0]`, or use it inside the `with` block".
- `top(ref mut m)`, E3028: "`top` takes the lock of `scores` itself; a `Mutex` takes no word at the
  call: `top(m)`".
- `clear` without its clause, `fn clear(scores: Mutex[Array[String]]):` with the same body, E2105 (R5,
  verdict 29): "`clear` changes the list in `scores` under its lock (`g.clear()`, line 13); say so on
  its first line: `fn clear(scores: Mutex[Array[String]]) changes Mutex[Array[String]]:`".
- `clear(m)` while a loop walks the locked list, in the lines `with g = m.lock():` (line 17),
  `for s in g:` (line 18), `clear(m)` (line 19), E3020 (R8 (c), R12, verdict 29): "this loop walks the
  list in `m` (line 18) under `m`'s lock, and `clear(m)` (line 19) may change any
  `Mutex[Array[String]]`, this one included (its first line says `changes Mutex[Array[String]]`), so
  it would change the list during the walk and wait for the lock this thread holds; call `clear(m)`
  after the `with g = m.lock():` block".

**7. Cost:** `copy of g[0]` is one allocation and copy of the name, as Rust's `.clone()` workaround and
Python's own behaviour of handing out an object; the `top_len` form costs nothing extra. The checks
themselves, and `clear`'s `changes` clause, are when compiling; the lock is the one C takes.

**8. Kind:** a readable spelling of Rust's guarantee (a `MutexGuard` borrow ends with the guard).

#### AN-31: initialisation (A18): a helper called in `init` before every field is set is E2100 with no help. Agreed row 7.

**v3:** as in v2; verdicts 21 to 34 change nothing here. Row 7's line stays
`fn load_assets() reads renderer -> Array[String]:`, as the owner agreed it without `self` (verdict 2).
Meaning unchanged.

**1. Intent:** `init` sets the renderer, then loads assets with a helper that only reads the renderer.

**2. OLD** (`o31b_init_class.em`, phase 1's `e18b`; the struct form `o31a` gives the same E2100):

```ember
class Renderer:
    width: int

class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(self, width: int):
        self.renderer = Renderer(width=width)
        self.assets = self.load_assets()

    fn load_assets(self) -> Array[String]:
        return [f"atlas@{self.renderer.width}px"]
```

```text
error[E2100]: `self` is used before all class fields are initialized
  --> o31b_init_class.em:12:23
   |
12 |         self.assets = self.load_assets()
   |                       ^^^^^^^^^^^^^^^^^^
   |

error: could not compile due to 1 error(s)
exit=1
```

**3. NEW:**

```ember
class Renderer:
    width: int

class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(width: int):
        this.renderer = Renderer(width=width)
        this.assets = this.load_assets()

    fn load_assets() reads renderer -> Array[String]:
        return [f"atlas@{this.renderer.width}px"]
```

Rule (row 7, R5): a method with `reads` may be called in `init` once every field it names is set; its
body is checked to read only those fields and to change none.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `width` (field of `Renderer`) | none (default: the renderer's own number) | none (default: field) |
| `renderer`, `assets`, `frame` | none (default: the game's own values) | none (default: field) |
| `this` in `init` | `this` in `init`: the object being built | `init` sets its fields with no word (R5) |
| `width` (parameter) | none (default: read-only parameter; a number is copied in) | none (default: never) |
| `this` in `load_assets` | `this`, `reads renderer` | no `changes`: no |

**5. Means exactly** (`m31_init_helper.em`): the helper takes the one field it reads:

```ember
class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(self, width: int):
        self.renderer = Renderer(width=width)
        self.assets = load_assets(self.renderer)

fn load_assets(renderer: Renderer) -> Array[String]:
    return [f"atlas@{renderer.width}px"]
```

```text
===== m31_init_helper.em
exit=0
```

**6. Still refused:**
- The call before `this.renderer` is set, E2100 (today's code, "used before all class fields are
  initialized", in the OLD output above, with this help): "`load_assets` reads `renderer`, which `init`
  has not set yet; write `this.renderer = Renderer(width=width)` before this line".
- The body reading a field not named, E2105: "`load_assets` reads `this.frame`, which its first line
  does not name; write `fn load_assets() reads renderer, frame -> Array[String]:`".
- A helper with no `reads` called before every field is set, E2100 with a new help: "`load_assets` may
  read any field of `Game`, and `assets` is not set yet; say what it reads:
  `fn load_assets() reads renderer -> Array[String]:`".
- A helper that reads `assets` while `init` calls it before `assets` is set, E2105: "`load_assets` reads
  `assets`, which its first line does not name, and `init` calls it on line 11 before `assets` is set;
  set `assets` first, or write `fn load_assets() reads renderer -> Array[String]:` and stop reading
  `assets`".

**7. Cost:** none.

**8. Kind:** a smarter compile-time solution (Rust cannot call a method on a half-built `self`).

#### AN-32: `mut self` on `init` is accepted and means the same as `self`

**v3:** as in v2 (the refused form `fn init() changes this:`, E2108, verdict 6); verdicts 21 to 34
change nothing here. Meaning unchanged.

**1. Intent:** write a constructor.

**2. OLD** (`o32_init_mut_self.em`, reader 2's t4):

```ember
class Door:
    angle: f32

    fn init(mut self):
        self.angle = 0.0

    fn open(self):
        self.angle = 90.0
```

```text
===== o32_init_mut_self.em
exit=0
```

**3. NEW:**

```ember
class Door:
    angle: f32

    fn init():
        this.angle = 0.0

    fn open() changes angle:
        this.angle = 90.0
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `angle` | none (default: the door's own number) | none (default: field) |
| `this` in `init` | `this` in `init`: the object being built | `init` sets its fields with no word (R5) |
| `this` in `open` | `this`: the door `open` was called on | `changes angle` |

**5. Means exactly** (`m32_init_self.em`, the same with today's `fn init(self)` and `fn open(self)`):

```text
===== m32_init_self.em
exit=0
```

**6. Still refused:**
- `fn init() changes this:`, E2108: "`init` may always set the fields of the object it builds; write
  `fn init():`".
- `fn init(mut self):`, E1010: "`self` is not a word of Ember: inside a method its object is `this`, and
  `init` may always set its fields; write `fn init():` and `this.angle = 0.0` in its body".
- `fn open():` with `this.angle = 90.0`, E2105 (AN-19's message).

**7. Cost:** none.

**8. Kind:** a readable spelling (one form for a constructor); the accepted form that says nothing is
refused with that advice.

#### AN-33: a `let` field still changes through `push`; E2104 has no help

**v3:** as in v2: E2104 keeps today's code with the vocabulary's `let` meaning (any change through a
`let` field after `init`, `V3/rules.md` section 4); verdicts 21 to 34 change nothing here. Meaning
unchanged.

**1. Intent:** a log whose list grows; its name is set once in `init` and never changes.

**2. OLD** (`o33a_let_push.em`, `o33b_let_assign.em`, reader 2's t2, t2b):

```ember
class Log:
    let items: Array[int]

    fn init(self):
        self.items = []

    fn add(self, v: int):
        self.items.push(v)

    fn clear(self):
        self.items = []
```

```text
===== o33a_let_push.em
exit=0
===== o33b_let_assign.em
error[E2104]: cannot assign to `let` field `Log.items` outside its `init`
 --> o33b_let_assign.em:8:9
  |
8 |         self.items = []
  |         ^^^^^^^^^^
  |

error: could not compile due to 1 error(s)
exit=1
```

`let` stops the assignment and lets the `push` through, and the refusal has no line to write.

**3. NEW:**

```ember
class Log:
    items: Array[int]
    let name: String

    fn init(name: StrView):
        this.items = []
        this.name = String(name)

    fn add(v: int) changes items:
        this.items.push(v)
```

`let` means nothing is changed through the field after `init`, for every type. A list that grows is a
list that changes, so it is not `let`; which methods change it is said on their first line
(`changes items`).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `items` | none (default: the log's own list) | none (default: field; `changes items` on `add`) |
| `name` (field) | `String`: the log's own text | `let`: never after `init` |
| `this` in `init` | `this` in `init`: the object being built | `init` sets its fields with no word (R5) |
| `name` (parameter) | `StrView`: a view of the caller's text | none (default: read-only parameter) |
| `this` in `add` | `this`: the log `add` was called on | `changes items` |
| `v` | none (default: read-only parameter; a number is copied in) | none (default: never) |

**5. Means exactly** (`m33_log_changes.em`):

```ember
class Log:
    items: Array[int]
    let name: String

    fn init(self, name: str):
        self.items = []
        self.name = name.to_string()

    fn add(self, v: int):
        self.items.push(v)
```

```text
===== m33_log_changes.em
exit=0
```

**6. Still refused:**
- OLD's `this.items.push(v)` on a `let items`, E2104: "`items` is `let`: it is set in `init` and never
  changed after; to add to it, write the field without `let`: `items: Array[int]`".
- A change through the `let` field `name` (`this.name += "!"`), E2104: "`name` is `let`: it is set in
  `init` and never changed after; to change it, write the field without `let`: `name: String`".
- OLD's `this.items = []` in `clear` on a `let items`, E2104: "`clear` sets `items`, which is `let`:
  set once in `init`; to empty it after `init`, write the field without `let`, `items: Array[int]`,
  and the method's first line as `fn clear() changes items:`".
- `add` changing `items` without saying so, E2105: "`add` changes `this.items`; write its first line as
  `fn add(v: int) changes items:`".

**7. Cost:** none; `let` lets AN-25's views into the field go unchecked.

**8. Kind:** an understandable limitation kept ("set once" and "grows" are different promises; only
the first is `let`), with the messages that say what to write.

## 5. Words needed

None. Every word the NEW blocks, the owner's-test tables and the messages use has a row in
`V3/vocabulary.md`: the kind test (`is`, `as`, `if`, `elif`), `cast`, `Mutex` with its `changes
Mutex[...]` on first lines, `MutSpan` with `ref mut`, `owns`, a change list (`changes C`), `changes`,
`fn`. The program's own names (`Parser`, `AnyPool`, `Pool`, `put`, `Renderer`, `first_value`,
`first_even`, `seen`) are not words. `push_str` still has no row; the refusals of AN-25 and AN-33
write `+=` instead (`[TXT-11]`: `s += t` appends), so it is not needed.

## 6. For the lead and the rule writer (not owner questions)

1. **`this` without noalias, not measured** (verdict 18, R4). A `changes this` or `changes <field>`
   method is compiled without noalias on `this`, as C compiles a plain `Game *` parameter, where Rust's
   `&mut self` is noalias. For a method that loops over one field of `this` while writing another
   (AN-17's `tick`), the C compiler may then load a field again after each write it cannot prove apart.
   No area-B answer measured this; the phase that measures again should time one such method (for
   example AN-17's `tick` over a long `players` list) against Rust's `&mut self`.
2. **AN-22's two-word program is refused without `from src`** (R8 (g), verdict 5, E3021): a method's
   result with no `from` points into all of its object, so the object may not change while the result
   is used. Its message is written in AN-22 (part 6); `V3/rules.md` section 3's E3021 row still has no
   example of this case.
3. **Carried blocks keep today's spellings by design**: the OLD and means-exactly blocks (and the C
   comment `/* init(self, owned text: String) */` in AN-25's `c25_document.c`) are today's Ember and
   run 3's C, copied byte for byte (`check_carried.py`); only NEW blocks, tables and messages use the
   v3 words.
4. **A `@cached` method's once-flag and verdict 29** (AN-21). This file reads verdict 29 ("a function
   that changes something under a lock says so") as adding nothing to a `@cached` method: filling its
   kept result under the once-flag changes a slot that is not a field of the program, that nothing but
   the method reads, and that held no value before; the change that clears it is the change of a
   `reads` field, named where it is made. If the rule writer reads it otherwise, R12 and the `@cached`
   row would need a clause the method's first line cannot write (the slot has no name).
5. **A by-type change through a call on an own local** (AN-30). `clear` says `changes
   Mutex[Array[String]]`; `main`, which calls `clear(m)` on its own local `m`, names nothing, by R5's "a
   change ... to an own local needs no clause". `changes T` is a claim about every object of a type, so
   R5 could be read either way; this file takes the reading R5 states. R5 could say it in one sentence
   ("a call whose first line says `changes T` needs no clause in a caller where every `T` it reaches is
   its own local or reached through its own `ref mut` parameters").
6. **A message in `V3/rules.md` that breaks verdict 22** (not area B's, found while checking the words):
   section 3's E2020 row gives "`fn(s) => names.push(copy of s)`", a lambda that changes the captured
   `names` with no `changes names` on its first line, which R12 now refuses (E2105). The line to write
   would be `fn(s) changes names => names.push(copy of s)` if a lambda written with `=>` takes a clause;
   neither file shows a `changes` clause on a `=>` lambda, so the spelling is the rule writer's to give
   (AN-56 is area D's).
7. **A user function given a function that changes things** (AN-29, part 6). R12 states the change list
   for the library (`for_each`); this file applies the same form to a user function, `fn apply[C, F:
   fn() changes C -> int](owns f: F) changes C -> int:`, which R12's wording ("a generic parameter
   written after `changes` stands for what the function given to the call changes") already covers.
8. **Messages that still break the message rule** (the lead's, phase 3; left as v2 wrote them, since no
   rule this step changed rewrote them):
   - AN-17 part 6, E3020: `this.players.extend(joined)` uses a list `joined` the program does not
     declare, and no line declares it (`V2/readability.md` F-05).
   - AN-17 part 6, E3021 from another package: asks the caller to rewrite `note`'s first line in a
     package it cannot change (F-09).
   - AN-23 part 6, E2140: "keep it in a name first: `n = first_value(xs)`, then `n += 1`" compiles and
     still loses the change; the two ways out (`xs[0] += 1`, or `first(ref mut xs) += 1` with
     `-> ref mut int`) are not given (F-02).
   - AN-24 part 6, E3063: "or keep `reg` alive as long as `vs`" is a way out in words (F-06).
   - AN-24 part 6, E3060: "keep the entity number and read `hs[0].reg.hp[0]` again" is a way out in
     words; the line would be `println(hs[0].reg.hp[0])` in place of line 3 (found in this step).
   - AN-24 part 6, E3022: the example's view struct is named `Handle`, a word the vocabulary keeps for
     tokens that name something elsewhere (F-10).
   - AN-25 part 6, E3063: "change `words` to `Array[String]`" does not give the field's line,
     `words: Array[String]` (F-06).
   - AN-31 part 6, E2100: "set `this.renderer` before this line" does not give the line,
     `this.renderer = Renderer(width=width)` (F-06).
   - AN-22 part 6, E2118 (and the rules' E2118 row): "rename the parameter" gives no line; it could give
     `fn next_word(line: StrView) changes pos -> StrView from src:` (found in this step).
   - AN-33 part 6, E2104 (minor): "remove `let` from `items`' declaration" could give the line,
     `items: Array[int]` (found in this step).
   **Phase 3 (the lead):** all of these are fixed in this file (AN-24's view struct is now `Editor`, its
   list `eds`), with the rules' E2118 row and, for item 6, the rules' E2020 row (the `:` form,
   `fn(s) changes names: names.push(copy of s)`, as AN-56 writes it); each fix is listed in
   `V3/phase3-messages.md`.

## 7. For the owner

1. **Answered (verdict 42, the owner: "yes, every ref mut name"): every `ref mut` name may be given
   another ref, `r = ref mut b` included; `const ref mut` is never given one.** The question as it was
   put: **two lines that look alike, one refused and one accepted** (AN-28; verdict 34 and R2). After
   `r = ref mut a`, the line `r = ref mut b` is refused (it would write `b`'s value into `a`; write
   `rb = ref mut b`). After `d = ref mut xs.as_mut_span()`, the line `d = ref mut ys.as_mut_span()` is
   accepted and points `d` at the other span (R2: a name that holds a `MutSpan` holds a view of its own,
   as `rest = tail.trim()` does). The rule is consistent underneath (`ref mut a` names a place,
   `ref mut xs.as_mut_span()` a span that a call gives), but a reader sees `x = ref mut ...` twice with
   two outcomes. Meanwhile this file follows R2. The other choice is one rule for every name made with
   `ref mut`: it is never pointed elsewhere, and `d = ref mut ys.as_mut_span()` is refused with "use a
   new name: `d2 = ref mut ys.as_mut_span()`"; that costs nothing while running, and loses only the
   reuse of one name for a second span (a loop that walks several spans through one name would name
   each round's span inside the loop instead). Which?

## 8. Files

- `/home/user/ember/tasks/96/redesign/run4/v3/area-B.md` (this file), phase 0's copy of `V2/area-B.md`
  with only the prose, NEW blocks, tables, refusals and costs edited.
- Under `/home/user/ember/target/redesign4/v3-03-area-B/`: `v2-area-B.md` (the copy it started from),
  `sec3.md` and `sec5to8.md` (the new sections), `check_carried.py` (checks that every OLD and
  means-exactly block is byte for byte v2's, and lists old spellings left outside them) and its output
  `check_carried.txt`.
