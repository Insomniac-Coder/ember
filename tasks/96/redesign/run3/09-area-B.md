# 09 area B: references kept or returned, locks, objects still being built (run 3, phase 2)

Annoyances answered: AN-17 .. AN-33 (17). Agreed rows kept exactly: 3 (AN-17), 4 (AN-30), 7 (AN-31).
Last run's reports repaired: `tasks/96/redesign/phase3b-references.md` and
`phase3b-references-attack.md` (section 5 of this file says what was kept, changed and thrown away).

Power: `PowerLineStatus` printed `Online` at the start and before writing this file.

Files (all under `target/redesign3/09-area-B/`):
- `o*.em`: the OLD programs. `m*.em`: the "means exactly" translations into today's Ember.
  `p_let_struct_field.em`: one probe. Every output below is from
  `target/redesign3/bin/ember.exe check <file>`; all of it is saved in `check_output.txt`
  (`check_old.txt` + `check_means.txt`). `check.sh` is the script that ran them.
- `c24_views_list.c`, `c25_document.c`: the translations into C where today's Ember cannot say it.
  `c30_lock_ref_escape.c`: what today's Ember accepts for AN-30, run under AddressSanitizer.
  Built with `clang -O2 -I target/redesign/bench` (and `-O1 -g -fsanitize=address` for the checker
  runs); each includes `harness.h`, calls `h_start()` first and `h_report(...)` last.

No compiler change, no git write, no Ember program run.

## 0. Terms

- **field clause**: what a method's first line says after its parameters: `changes <fields>` (row 3:
  the fields it may change) and `reads <fields>` (row 7: the fields it reads). Both are checked
  against the body and at every call.
- **one-owner object**: a `class` object with one owner. After 06's C-3 a class object is not shared
  by `p2 = player`; a second owner is written `shared` (area C). So every second name for a one-owner
  object is a `ref`/`ref mut` name the compiler can see.
- **access word**: today's run-time check on class fields (`[EXC-19]`, spec 2712-2730): one word per
  non-`Copy` field, tested on each long-term access, set for a whole `mut self` call.
- **view**: a value that points into something else and may not outlive it (`ref T`, `Span[T]`,
  `StrView`, a struct holding one).
- **source** of a result: what a returned view points into.
- **guard**: what `m.lock()`, `rw.read()`, `rw.write()`, `c.read()`, `c.write()` return; it holds the
  lock (or the run-time check) until its `with` block ends.
- **own local**: a name made in this function by the value on its right (`total = 0`,
  `d = Door()`); only this function can change it and nothing else refers to it. In the owner's test
  it is written "none (default: own local)": it never aliases and never copies silently.
- **the owner's test count**: for the form, a name is counted "without a word" when at least one of
  the two questions (what it is; whether it may change) is answered by a stated default instead of a
  word in the line.

## 1. The model for area B, in plain words

1. **A method's first line says what it touches.** Plain `self` reads; `self ... changes a, b` may
   change exactly `a` and `b` (row 3); `mut self` may change any field (one meaning for every type,
   06 settlement 20); `self ... reads a, b` reads exactly `a` and `b` (row 7). `changes` and `mut self`
   are always written. A method with no `reads` reads, as far as a caller in another package knows,
   every field; inside the package the compiler uses what the body reads (today's `[BRW-10]` for
   reads only), and the error message gives the `reads` line when a caller needs it.
2. **These are checked when compiling, for every one-owner object, class or struct.** The class
   rule "`mut self` holds a write access on every field for the whole call" and the access words
   that check it while running go for one-owner objects: every other name for the object is a
   `ref`/`ref mut` name the borrow checker sees. Area C decides what a `shared` object keeps.
3. **A name that changes someone else's value says `ref mut` on the name** (06's C-1): a parameter
   (`ref mut c: Counter`), a loop name (`for ref mut p in players:`), a pattern name (row 8). A plain
   name never changes anything, including a `Cell` field (section 4 rules on 06's C-10).
4. **A returned view says where it points when the default is not enough.** Default (stated, as
   today's `[LT-1]`): a method's result points into the object; otherwise into the one parameter it
   could, or into all of them. `from <name>` after the result type narrows it (replaces `@borrows`).
   A `ref mut` parameter is always a source, so `smaller(ref mut a, ref mut b) -> ref mut int` needs
   nothing written.
5. **A call that returns `ref mut T` is a place**: `first(xs) += 1`.
6. **Views may be kept in lists and fields when their source is named.** A local list of views is a
   view itself (nothing written, as row 5's `word`); a field of views says `from <let field>` or
   `from static`. A `let` field never changes after `init`, so views into it stay valid for the
   object's whole life with no tracking.
7. **A view made through a guard points into the guard** (row 4), so it ends with the `with` block.
8. **`init` may set fields with plain `self`; a helper it calls says what it reads** (row 7).

What still runs while the program runs, in area B: Ember's bounds checks (as today, not this area's);
`@cached`'s flag test (one branch per call, the hand-written C cache); the one type-id compare of a
downcast (`cast ... as Pool[T]`, as today). Nothing else: every rule above is checked when compiling.

## 2. The answers

### AN-17: a method changes one field while a loop walks another (A1). Agreed row 3.

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

    fn note(self, msg: String) changes log:
        self.log.push(f"[{self.frame}] {msg}")

    fn tick(self) changes players, log:
        for ref mut p in self.players:
            p.hp -= 1
            if p.hp <= 0:
                self.note(f"{p.name} died")

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    g.tick()
    println(g.log)
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `name`, `hp` (fields of `Player`) | none (default: the player's own value, by its type) | none (default: a field changes only through a method whose first line names it, or `mut self`, or a `ref mut` name for the player) |
| `players`, `log`, `frame` (fields of `Game`) | none (default: the game's own list / number) | none (default as above): `changes players, log` on `tick`, `changes log` on `note` |
| `self` in `note` | `self`: the game `note` was called on | `changes log`: only `log` |
| `msg` | none (default: a read-only view of the caller's `String`) | none (default: a plain parameter never changes anything) |
| `self` in `tick` | `self` | `changes players, log` |
| `p` | `ref mut`: a name for the list's own element | `ref mut`: yes |
| `g` | none (default: own local, made by `Game(...)`) | none (default: own local) |

**5. Means exactly** (`m17_tick_note.em`): one owner, so today's struct rules give the same
compile-time check (the loop changes `players`, `note` touches only `log`, `[BRW-10]`):

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
- `note`'s body changing a field it does not name (`self.frame += 1`): "`note` changes `self.frame`,
  which its first line does not name; write `fn note(self, msg: String) changes log, frame:`".
- `tick` calling `note` without naming `log`: "`tick` calls `note`, which changes `log`; add it to
  `tick`'s first line: `fn tick(self) changes players, log:`".
- Inside the loop, a method that changes the list being walked (`self.add_player(...)` with
  `changes players`): "`tick` is walking `self.players` (line 14), and `add_player` changes
  `players`; collect the new players in a list and add them after the loop" (the understandable
  limitation, JOB: delete or add after the loop).
- From another package, a call to a method with no `reads` inside a loop that changes a field:
  "`note` may read any field of `Game`, including `players`, which this loop is changing; if it does
  not, say what it reads: `fn note(self, msg: String) reads frame changes log:`".

**7. Cost:** none; the check is when compiling. Today's class version, if it compiled, would pay
access-word checks while running (`[EXC-19]`, spec 2712-2730: a load and a compare per long-term
access, a check per field on entry to a `mut self` method).

**8. Kind:** a smarter compile-time solution. Rust refuses this program (E0499, a second `&mut self`
while the loop borrows `self.players`); the written field set lets it through, checked.

### AN-18: `mut self` on a class means "hold a write access on every field for the whole call"

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

    fn note(self, msg: String) changes log:
        self.log.push(copied msg)

    fn reset(mut self):
        self.players = []
        self.log = []
        self.score = 0

fn tick(ref mut g: Game):
    for p in g.players:
        g.note(f"{p} moved")
    g.reset()
```

`mut self` keeps one meaning for every type: "this method may change any field"; it is checked when
compiling, like `changes`, and nothing is marked while running. The OLD `tick(g, g)` cannot be
written for a one-owner object without saying it: `fn tick(g: Game, ref mut other: Game)` called as
`tick(g, g)` is a read and a change of the same object in one call (refused below).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `players`, `log`, `score` | none (default: the game's own values) | none (default: through `changes` or `mut self` only) |
| `self` in `note` | `self` | `changes log` |
| `msg` | none (default: read-only view of the caller's `String`) | none (default: never) |
| `self` in `reset` | `self` | `mut`: any field |
| `g` | `ref mut`: the caller's game | `ref mut`: yes |
| `p` | none (default: a read-only view of the list's element, as row 8's plain `path`) | none (default: never) |

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
- `g.reset()` inside the loop: "`tick` is walking `g.players` (line 2), and `reset` may change every
  field of `Game`; call it after the loop, or, if it only changes some fields, name them:
  `fn reset(self) changes score:`".
- `tick(g, g)` with `fn tick(g: Game, ref mut other: Game)`: "`tick` would read `g` and change it
  through `other` in the same call; pass it once: `fn tick(ref mut g: Game)`".
- `fn init(mut self)`: see AN-32.

**7. Cost:** none for one-owner objects: no access words, no check on entry (today: one word per
non-`Copy` field and a check per field on entry, `[EXC-19]`). A `shared` object keeps whatever area C
decides (AN-48); this answer does not remove a check there.

**8. Kind:** a smarter compile-time solution.

### AN-19: plain `self` in a class method changes fields with no word

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

    fn open(self) changes angle:
        self.angle = 90.0

fn main():
    d = Door()
    d.open()
    println(d.angle)
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `angle` | none (default: the door's own number) | none (default: through `changes angle` or `mut self`) |
| `self` | `self` | `changes angle` |
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

**6. Still refused:** OLD's `fn open(self): self.angle = 90.0`: "`open` changes `self.angle`; write its
first line as `fn open(self) changes angle:`" (the vocabulary's `self` row).

**7. Cost:** none.

**8. Kind:** a readable spelling of Rust's guarantee (`&mut self`), narrowed to the field.

### AN-20: a plain parameter, a `for` name or a `Cell` field changes a class object with no word

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
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `n`, `health` | none (default: the object's own number) | none (default: through a `ref mut` name, `changes` or `mut self`) |
| `frame` | `Cell`: a value replaced whole by `.set` | none (default as any field; section 4: `.set` needs a name that may change) |
| `c` | `ref mut`: the caller's counter | `ref mut`: yes |
| `players` | `ref mut`: the caller's list | `ref mut`: yes |
| `p` | `ref mut`: the list's own element | `ref mut`: yes |
| `s` in `tick` | `ref mut`: the caller's clock | `ref mut`: yes |
| `s` in `show` | none (default: a read-only view of the caller's clock) | none (default: never, `Cell` included) |

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
- `fn bump(c: Counter): c.n += 1`: "`bump` changes `c.n`, but `c` only reads the caller's counter; to
  change it write `fn bump(ref mut c: Counter):`".
- `for p in players: p.health = 100.0`: "`p` only reads each player; to change them write
  `for ref mut p in players:`" (and `players` must be `ref mut players` or the caller's own).
- `fn tick(s: Clock): s.frame.set(...)`: "`tick` sets `s.frame`, but `s` only reads the caller's
  clock; write `fn tick(ref mut s: Clock):`".
- `fn bump(mut c: Counter)`: "to change the caller's `c` through `c`, write `ref mut c: Counter`" (C-1).

**7. Cost:** none; the same instructions, only the spelling changes.

**8. Kind:** a readable spelling of Rust's guarantee (`&mut`), with the word on the name.

### AN-21: caching inside a read-only method (A31)

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

**3. NEW** (planned word `@cached`, row 7's `reads`):

```ember
struct Mesh:
    verts: Array[f64]

    @cached
    fn radius(self) reads verts -> f64:
        r = 0.0
        for v in self.verts:
            r = max(r, abs(v))
        return r

fn main():
    m = Mesh(verts=[1.0, -2.0])
    println(m.radius())      # computes 2.0 and keeps it
    println(m.radius())      # the kept 2.0, no loop
    m.verts.push(-5.0)       # a change to `verts` clears the kept result
    println(m.radius())      # computes 5.0
```

Area B's rulings on `@cached` (planned-words section 8): the result is a `Copy` value returned by
value, or, for any other type, a `ref T` view into the kept result (`-> ref Array[f64]`), which is a
view of the object and so keeps `verts` from changing while it is used (the default source rule,
section 1 item 4). A change to a `reads` field through any name clears the result: an assignment, a
`changes verts` or `mut self` call, or taking a `ref mut` view of `verts`.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `verts` | none (default: the mesh's own list) | none (default: field; any change clears the kept result) |
| `self` | `self`; `@cached` says the result is kept; `reads verts` says what it depends on | `self` without `changes`: no field changes |
| `r` | none (default: own local) | none (default: own local) |
| `v` | none (default: read-only view of each element) | none (default: never) |
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

**6. Still refused** (planned-words 3.4, kept): `@cached` without `reads` ("say what `radius`
depends on, so a change can clear it: `@cached fn radius(self) reads verts -> f64:`"); with other
parameters ("`@cached` keeps one result, and `radius` takes `scale`; keep a `Map` field of results by
hand"); reading a `shared`, `thread_shared`, `weak` or `Cell` field ("`radius` reads `mesh`, which its
other owners may change without `Mesh` seeing it, so a kept result could be stale; remove `@cached`,
or keep the result by hand"); with `changes` or `mut self` ("a cached method only reads; remove
`changes verts`"). New here: a `@cached` method on a value sent to another thread: "`Mesh` keeps
`radius`'s result in itself, and two threads could fill it at once; keep the mesh on one thread, or
remove `@cached`" (area E may replace this refusal with a once-flag).

**7. Cost:** one flag test per call; one store (clear the flag) at each place that may change `verts`;
the slot and flag in the object (8 bytes for the `f64` and one flag). The same as the cache a C
programmer writes by hand (the translation above is that program).

**8. Kind:** a new mechanism.

### AN-22: returning references (A5): a result borrows all of `self` unless `@borrows` is written above; the ECS pool reference is E3060

**1. Intent:** `next_word` returns the next word of the source text, and the caller keeps two words
at once; `get_mut[T]` returns a component inside its pool, to change it in place.

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

    fn next_word(self) changes pos -> StrView from src:
        rest = self.src[self.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        self.pos += end + 1
        return rest[..end]

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    a = p.next_word()
    b = p.next_word()        # a points into `text`, not into `p`
    println(a, b)
```

```ember
struct Registry:
    pools: Map[u128, owned AnyPool]

    fn get_mut[T](self, e: int) changes pools -> ref mut T:
        pool = ref mut cast self.pools[7u128] as Pool[T]
        return ref mut pool.dense[pool.sparse[e]]
```

The rules: `from <name>` after the result type names the source (a parameter, a field of `self`
written bare as in `changes`/`reads`, or `self`); a name that is both a parameter and a field is
refused ("`src` is both a parameter and a field; rename the parameter"). With no `from`, the stated
default applies (section 1 item 4). `from src` on a view field means "into what `src` views", so the
call's hold on `p` ends with the call. In `get_mut`, `pool` is spelled `ref mut`: a name for the slot
inside `self.pools`, not a local of its own, so the result points into `self` (the default) and E3060
cannot arise; a downcast of a place is a place (last run's 3.4, saved by its attack: one type-id
compare, never the method table's address).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `src` | `StrView`: a view of someone else's text | none (default: field) |
| `pos` | none (default: the parser's own number) | none (default: field; `changes pos`) |
| `self` in `next_word` | `self` | `changes pos` |
| `rest`, `a`, `b` | none (default: a read-only view, as row 5's `word`; `from src` on `next_word`'s line says where `a`, `b` point) | none (default: never) |
| `end` | none (default: own local) | none (default: own local) |
| `text`, `p` | none (default: own local) | none (default: own local) |
| `pools` | `owned`: one owner of each pool | none (default: field; `changes pools`) |
| `self` in `get_mut` | `self` | `changes pools` |
| `e` | none (default: read-only parameter) | none (default: never) |
| `pool` | `ref mut`: a name for the slot in `self.pools` | `ref mut`: yes |

**5. Means exactly:** `m22a_two_words_borrows.em` (today's `@borrows(self.src)`), and for one
component type `m22b_pool_place.em` (today's `Box` for `owned`, `ref mut` on the place):

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

The generic downcast cannot be said in today's Ember (`cast` needs class handles); its C is last
run's faithful model `target/redesign2/references/attack2/bench/get2.c -DCAST` (a type-id load and
compare, then the element's address).

**6. Still refused:**
- `return rest[..end]` under `from src` when `rest` came from somewhere else (E3062): "`next_word`
  returns a view of `line`, but its first line says `from src`; write `from src, line`, or return
  `String(...)`".
- At the call, when the default is too wide (today's E3025): "`a` points into `p` because
  `next_word`'s result points into the parser by default; if it points only into the text, write
  `fn next_word(self) changes pos -> StrView from src:`" (the compiler computes the set from the body
  to write this line; it never applies it silently).
- `pool = cast self.pools[k] as Pool[T]` (no `ref mut`): "this would take the pool out of
  `self.pools`; to work on it in place write `pool = ref mut cast self.pools[k] as Pool[T]`".
- Changing `self.pools` while the returned component is used (today's E3025, `box_bad.em`): "`p`
  points into `reg.pools`, which `clear` would empty; finish with `p` first".
- `@borrows(...)` on its own line: "write `from a` after the result type".

**7. Cost:** none for `from` (compile time only). The downcast: one type-id compare per get. Last
run's faithful model measured it at 1.223 to 1.244 of plain C, against safe Rust's `downcast_mut` at
1.571 to 1.635 (`tasks/96/redesign/phase3b-references-attack.md`, section 3.2 table, lines 282-287;
provisional, measured while other agents ran). Bounds checks are included in those rows.

**8. Kind:** a readable spelling of Rust's guarantee (a lifetime written as `from src` on the
signature).

### AN-23: a call that returns `ref mut` is not a place

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
    first(xs) += 1
    counts = Map[String, int]()
    counts.entry("a").or_default() += 1
    println(xs[0], counts["a"])
```

Rule: a call whose result type is `ref mut T` is a place for the statement it is in; `=`, `+=` and a
`mut self` method may be applied to it.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `xs` (parameter) | `ref mut`: the caller's list | `ref mut`: yes |
| `xs` (local) | none (default: own local) | none (default: own local; the call changes it through `first`'s `ref mut xs`) |
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

**6. Still refused:** a call whose result is a new value, `make_point().y = 1` or
`copy_of_first(xs) += 1`: "`copy_of_first(xs)` gives a new `int`, so adding to it would be lost;
keep it in a name first: `n = copy_of_first(xs)`, then `n += 1`".

**7. Cost:** none: the same address and store as the bound form.

**8. Kind:** a readable spelling of Rust's guarantee (Rust writes `*first(&mut xs) += 1`).

### AN-24: structs holding references (A6): a view struct cannot be stored in an `Array`

**1. Intent:** keep a list of handles that each read one entity of one registry, and sum their hit
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

**3. NEW:** the OLD program, unchanged, is accepted. Rule: a list (`Array`, `Map`, `Set`, `Option`)
whose element type holds a view is itself a view: it may live as long as every source of its
elements, no longer (one limit per view field of the element type, covering every element pushed, as
Rust's single `'a`; erasing to `owned dyn I` keeps the limit). Nothing is written for a local list;
a field holding such a list says its source with `from <let field>` (AN-25).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `hp`, `entity` | none (default: the struct's own values) | none (default: field) |
| `reg` (field) | `ref`: a read-only view of a registry | `ref`: no |
| `self` in `hp` | `self` | `self`: no |
| `reg` (local) | none (default: own local) | none (default: own local) |
| `vs` | none (default: own local list; its element type `View` holds a `ref`, so the list is a view of `reg`) | none (default: own local) |
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
- Two changeable views of one registry in one list (`o24a`, `Handle(reg=ref mut reg, ...)` twice), as
  in Rust: "`hs` would hold two changeable views of `reg` at once (lines 12 and 13), so one could
  change what the other reads; keep the entity numbers, `ids: Array[int] = [0, 1]`, and change
  `reg.hp[id]` through `reg`".
- The list kept longer than `reg` (returned, stored in a field with no `from`, sent to a thread):
  "`vs` holds views of `reg`, which ends when `main` returns; keep the entity numbers instead, or keep
  `reg` alive as long as `vs`".

**7. Cost:** none beyond plain C: the C above is the layout; the limit exists only when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (`Vec<View<'a>>` with nothing written).

### AN-25: self-referential data (A7): a value owning its text and the word views into it

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

    fn init(self, owned text: String):
        self.text = text
        self.words = self.text.split_whitespace().collect()

fn main():
    d = Document(String("the quick fox"))
    println(d.words.len())
```

The rules (area B's ruling on planned-words section 8, the heap-block condition):
1. `from text` names a `let` field of the same value whose contents live in their own heap block: a
   `String` or an `Array`. Today's runtime gives both the layout `{ptr, len, cap}` with the contents
   outside the header (`runtime/ember_rt/include/ember_rt.h:2022-2026`, `ember_vec`; `String` "is
   the same buffer"), and ADR-141's frame buffer is only for a list that never leaves its function,
   which a field's contents always do. So moving the value moves the header, never the characters.
   If `String` ever gains an inline small-string form, `from` on a `String` field must be refused.
2. `let` means `text` never changes after `init` (06's C-8), so the words stay valid for the value's
   whole life. Last run had to make the views "unusable" after every write to `text` and was attacked
   through an index write (`docs[i].text.push_str(...)` then `docs[j].words`, attack report section 5);
   with `let` there is no write to track, and that attack has nothing to hit.
3. `words` takes only views into `text` (or static text).
4. A view read out (`w = d.words[0]`) is a view of `d`: `d` may not be moved or ended while `w` lives
   (last run's attack fix 1).
5. Works for a `struct` too (`let` on a struct field is accepted today: `p_let_struct_field.em`,
   `exit=0`); a struct literal fills `text` before computing `words`.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `text` (field) | `String`: the document's own text | `let`: never after `init` |
| `words` | `StrView` elements, `from text`: views into `text` | none (default: field; changed through `changes words` or `mut self`) |
| `self` in `init` | `self` in `init`: the object being built | `init` sets its fields (vocabulary: "`init` may always set fields") |
| `text` (parameter) | `owned`: handed over by the caller | `owned`: it is now `init`'s, moved into the field |
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
- `from` naming a field that is not `let`: "`words` points into `text`, so `text` must never change
  after `init`; write `let text: String`".
- A change to `text` after `init`: AN-33's message.
- `self.words.push(other_word)` with a view of something else: "`words` holds views of `text` only,
  and `other_word` points into `line`; keep your own copy: change `words` to `Array[String]` and push
  `String(other_word)`".
- `copied d`: "`d.words` points into `d.text`, so a copy's words would point into the old text; write
  a `clone` for `Document` that splits the copy's text again" (re-pointing each view, last run's rule
  7, is listed as a good part, section 5).
- `from` on a fixed array or inline struct field: "`buf` is stored inside the value, so moving the
  value moves it; keep the text in a `String`".

**7. Cost:** none beyond plain C: the C above is the layout (pointer and length per word, no count, no
check). Last run measured this layout against Rust's index-range workaround (1.205 of plain C) and
today's one-`String`-per-word (1.451 time, 2.122 memory) (`phase3b-references.md`, section 5.4 table;
provisional; the attack report section 5 notes that the Ember program also pays bounds checks, about
1.046 of plain C, which every Ember program pays).

**8. Kind:** a smarter compile-time solution (Rust refuses self-reference; Ember accepts it because
the source is `let` and its contents never move).

### AN-26: `str` is a borrowed view while Python's `str` is the owned string

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

    fn init(self, name: StrView):
        self.name = String(name)
        self.kind = "human"
```

`str` is spelled `StrView` (06, 07). A `StrView` field of a class says what it points into:
`from <let field>` (AN-25) or `from static` (text written in the program). A function returning
`StrView` has the default source (section 1 item 4), or `from`.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `name` (field) | `String`: the player's own text | none (default: field) |
| `kind` | `StrView from static`: a view of text written in the program | none (default: field) |
| `self` | `self` in `init` | `init` sets fields |
| `name` (parameter) | `StrView`: a view of the caller's text | none (default: read-only parameter) |

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
- `str` anywhere: "`str` is now `StrView`, a view into someone else's text; for the player's own text
  write `String`".
- `name: StrView` in a class with no `from`: "a `Player` keeps `name` after the text it came from may
  be gone; to keep its own copy write `name: String` and `self.name = String(name)`".
- A non-static view into a `from static` field (`self.kind = name`): "`kind` holds only text written
  in the program, and `name` comes from the caller; write `self.kind = String(name)` with
  `kind: String`".
- `o26c`'s `push` while `n` lives: "`n` points into `names` (`first` returns a view of `xs`), and
  `push` may move what it points into; use `n` before the `push`, or keep `n = String(first(names))`".

**7. Cost:** none for the spelling. `String(name)` is one allocation and copy, the same as Python's
own `str` object and today's `to_string()`.

**8. Kind:** a readable spelling of Rust's guarantee (`&str` vs `String`, `&'static str`).

### AN-27: `ref T` / `ref mut T` as a parameter's type

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
    bump(a)
    println(a)
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `n` | `ref mut`: the caller's number | `ref mut`: yes |
| `a` | none (default: own local) | none (default: own local) |

**5. Means exactly** (`m27_bump.em`):

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

**6. Still refused:** `fn bump(n: ref mut int)`: "the word goes on the name: write
`fn bump(ref mut n: int):`"; `fn show(x: ref int)`: "a parameter already reads the caller's value
without copying it; write `fn show(x: int):`"; `fn bump(mut n: int)`: "to change the caller's `n`
through `n`, write `ref mut n: int`" (vocabulary C-1).

**7. Cost:** none (passed by address, as today's `mut n`).

**8. Kind:** a readable spelling of Rust's guarantee (`&mut i64`), on the name.

### AN-28: `r = ref mut b` on a `ref mut` local copies `b` into `r`'s first target

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
    rb = ref mut b
    rb = 30
    println(a, b)
```

A reference name always refers to the place it was made from (row 8: `path` is the NPC's list for
the whole arm), so `r = 20` writes into `a`; a second place gets a second name.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `a`, `b` | none (default: own local) | none (default: own local) |
| `r` | `ref mut`: a name for `a` | `ref mut`: yes |
| `rb` | `ref mut`: a name for `b` | `ref mut`: yes |

**5. Means exactly** (`m28_two_refs.em`, the same code):

```text
===== m28_two_refs.em
exit=0
```

**6. Still refused:** `r = ref mut b` (and `r = ref b`) on a reference name: "`r` always refers to
`a`; this copies `b`'s value into `a`; to refer to `b`, use a new name: `rb = ref mut b`" (vocabulary
`ref mut` row).

**7. Cost:** none.

**8. Kind:** an understandable limitation kept (a reference name is never re-pointed), with the one
misleading form refused.

### AN-29: `ref String` is not accepted where `str` is expected at a call

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

**3. NEW:** the OLD program, unchanged (with `str` spelled `StrView`), is accepted. Rule: a `String`
place or a `ref String` reads as a `StrView` of its text wherever a `StrView` is wanted, inside a
lambda as everywhere; the view points into that `String`.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `s` | `StrView`: a view of the caller's text | none (default: read-only parameter) |
| `f` | none (default: a read-only callable parameter) | none (default: never) |
| `names`, `total` | none (default: own local) | none (default: own local) |
| `n` | none (default: read-only view of each element) | none (default: never) |

**5. Means exactly** (`m29_str_in_lambda.em`, `shout(n.as_str())`):

```text
===== m29_str_in_lambda.em
exit=0
```

**6. Still refused:** a `ref mut String` where a `StrView` is kept past a change: "`v` points into
`n`'s text, which `push_str` may move; use `v` first" (the ordinary borrow rule).

**7. Cost:** none (pointer and length, as `as_str()`).

**8. Kind:** a smarter compile-time solution (one coercion rule, as Rust's `&String` to `&str`).

### AN-30: a reference out of a lock, `RefCell` or `RwLock` guard compiles and is unsafe (A33). Agreed row 4.

**1. Intent:** `top(scores)` returns the best name from a locked table.

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
        return copied g[0]

fn top_len(scores: Mutex[Array[String]]) -> int:
    with g = scores.lock():
        return g[0].len()
```

Rule: a view made through a guard (`g[0]`, `ref g[0]`, a slice of `g`) points into the guard, as if
the guard's reading method said `from g`; the guard ends with its `with` block, so such a view cannot
leave it. The same for `RwLock`'s `read()`/`write()` and `RuntimeChecked`'s `read()`/`write()`.

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `scores` | none (default: a read-only parameter; its type `Mutex` says the list inside changes only under the lock) | none (default: never directly) |
| `g` | `lock()`: the guard, the list while locked | `lock()`: yes, while the block lasts (vocabulary `Mutex` row) |

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

**6. Still refused:**
- `return ref g[0]` inside `with g = scores.lock():`: "this points into the locked list, which can
  change once the lock ends; return a copy, or do the work inside the `with` block" (row 4, exactly).
- The same under `scores.read()` / `scores.write()`: "this points into the read-locked list, which can
  change once the lock ends; return a copy, or do the work inside the `with` block".
- Under `RuntimeChecked`'s `c.read()`: "this points into the checked list, which can change once
  `read()` ends; return a copy, or do the work inside the `with` block".
- A view of `g` stored in an outer name (`best = ref g[0]` then used after the block): "`best` points
  into the locked list, which can change once the lock ends at line N; keep `best = copied g[0]`, or
  use it inside the `with` block".

**7. Cost:** `copied g[0]` is one allocation and copy of the name, as Rust's `.clone()` workaround and
Python's own behaviour of handing out an object; the `top_len` form costs nothing extra. The check
itself is when compiling.

**8. Kind:** a readable spelling of Rust's guarantee (a `MutexGuard` borrow ends with the guard).

### AN-31: initialisation (A18): a helper called in `init` before every field is set is E2100 with no help. Agreed row 7.

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
class Game:
    renderer: Renderer
    assets: Array[String]
    frame: int = 0

    fn init(self, width: int):
        self.renderer = Renderer(width=width)
        self.assets = self.load_assets()

    fn load_assets(self) reads renderer -> Array[String]:
        return [f"atlas@{self.renderer.width}px"]
```

Rule (row 7): a method with `reads` may be called in `init` once every field it names is set; its
body is checked to read only those fields (and to change none).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `renderer`, `assets`, `frame` | none (default: the game's own values) | none (default: field) |
| `self` in `init` | `self` in `init` | `init` sets fields |
| `width` | none (default: read-only parameter) | none (default: never) |
| `self` in `load_assets` | `self`, `reads renderer` | `self` without `changes`: no |

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
- The call before `self.renderer` is set: "`load_assets` reads `renderer`, which `init` has not set
  yet; set `self.renderer` before this line".
- The body reading a field not named: "`load_assets` reads `self.frame`, which its first line does not
  name; write `fn load_assets(self) reads renderer, frame -> Array[String]:`".
- A helper with no `reads` called before every field is set (today's E2100, new help): "`load_assets`
  may read any field of `Game`, and `assets` is not set yet; say what it reads:
  `fn load_assets(self) reads renderer -> Array[String]:`".

**7. Cost:** none.

**8. Kind:** a smarter compile-time solution (Rust cannot call a method on a half-built `self`).

### AN-32: `mut self` on `init` is accepted and means the same as `self`

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

    fn init(self):
        self.angle = 0.0

    fn open(self) changes angle:
        self.angle = 90.0
```

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `angle` | none (default: the door's own number) | none (default: field) |
| `self` in `init` | `self` in `init` | `init` sets fields |
| `self` in `open` | `self` | `changes angle` |

**5. Means exactly** (`m32_init_self.em`, the same with today's `fn open(self)`):

```text
===== m32_init_self.em
exit=0
```

**6. Still refused:** `fn init(mut self)`: "`init` may always set fields; write `fn init(self):`"
(vocabulary `mut` row).

**7. Cost:** none.

**8. Kind:** a readable spelling (one form for a constructor); the accepted form that says nothing is
refused with that advice.

### AN-33: a `let` field still changes through `push`; E2104 has no help

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

    fn init(self, name: StrView):
        self.items = []
        self.name = String(name)

    fn add(self, v: int) changes items:
        self.items.push(v)
```

`let` means nothing is changed through the field after `init`, for every type (06's C-8). A list that
grows is a list that changes, so it is not `let`; which methods change it is said on their first line
(`changes items`).

**4. The owner's test:**

| name | what it is | may it change |
|---|---|---|
| `items` | none (default: the log's own list) | none (default: field; `changes items` on `add`) |
| `name` (field) | `String`: the log's own text | `let`: never after `init` |
| `self` in `init` | `self` in `init` | `init` sets fields |
| `name` (parameter) | `StrView`: a view of the caller's text | none (default: read-only parameter) |
| `self` in `add` | `self` | `changes items` |
| `v` | none (default: read-only parameter) | none (default: never) |

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
- A change through a `let` field after `init` (`self.name.push_str("!")`, or OLD's `self.items.push(v)`
  on a `let items`): "`items` is `let`: it is set in `init` and never changed after; to add to it,
  remove `let` from `items`' declaration".
- E2104, new help: "cannot assign to `let` field `Log.items` outside `init`; set it in `init`, or
  remove `let` from its declaration".
- `add` changing `items` without saying so: AN-19's message.

**7. Cost:** none; `let` lets AN-25's views into the field go unchecked.

**8. Kind:** an understandable limitation kept ("set once" and "grows" are different promises; only
the first is `let`), with the messages that say what to write.

## 3. New words

No new word is used. One new position of two existing words, used by AN-26:

| spelling | meaning | where written | reason | example | used by |
|---|---|---|---|---|---|
| `static` (after `from`, as `from static`) | the view points only into text or data written in the program (the `static` region, spec `[LT-3]`) | after the type of a view field: `kind: StrView from static` | a class field holding a literal view is legal today (`o26a`, exit 0) but the declaration does not say so; without it the only spelling would be `kind: String`, an allocation per object. Both words already exist (`from`, planned-words 4; `static`, vocabulary 3.1) with these meanings | `kind: StrView from static` | AN-26 |

Two clause details, no new word: `from` may name a field of `self` bare (as `changes`/`reads` do),
a parameter, or `self`; a parameter and a field of the same name is refused (AN-22). `reads` and
`changes` take several names separated by commas (`reads renderer, frame`).

## 4. Rulings planned-words section 8 asked of area B

1. **`@cached`'s result type**: a `Copy` result is returned by value; any other result is returned as a
   `ref T` view into the kept value, which is a view of the object (the default source), so the fields
   it reads cannot change while the view is used. A `@cached` method's object is not sent to another
   thread unless area E gives the slot a once-flag (AN-21).
2. **`from` on a field, the heap-block condition**: the named field must be `let` and a `String` or
   `Array` (contents outside the header, `ember_vec {ptr, len, cap}`; ADR-141 frame buffers only for
   lists that never leave their function). A fixed array or inline struct field is refused (AN-25).
3. **`Cell` (06's C-10)**: the default "a plain name never changes anything" holds without an
   exception. Setting a `Cell` field needs a name that may change it (`ref mut s`, `mut self`,
   `changes frame`), or a value that is `shared` (area C decides whether one of several owners may
   change it, AN-48; a `Cell` is the one-thread way it would). Cost: none, `Cell.set` is a plain store.
   What `Cell` still gives: changing one field while other `ref` views of the same value are alive is
   still refused; it lets `shared` owners set a value whole, never lend a reference into it.

## 5. Last run's reports, repaired

| last run's answer | this run | why |
|---|---|---|
| A5: the body decides what a result borrows, nothing written (`phase3b-references.md` 3.3) | **changed**: the source is written (`from`) when the default is not enough; the compiler computes it only to print the line to write | JOB: "the compiler does not silently decide what the programmer did not say"; rows 3 and 7 write `changes`/`reads` where `[BRW-10]` inferred them; a written clause is also the public interface, which removes last run's attacks 1 and 2 (a body change widening a public summary; hot reload) |
| A5: a downcast of a place is a place; pool table keyed by the type (3.4, saved by the attack) | **kept**, the downcast as `ref mut cast ... as Pool[T]`; the table left as a library proposal | `ref mut` on `pool` is what makes it a place, in this run's words |
| A5: `return self.dense[i]` under `-> ref mut int` lends the place (3.5) | **thrown away** | a `return` with no word that lends a place hides meaning; `return ref mut self.dense[i]` says it. Good part: none beyond the observation that the result type already says it, which the error message uses ("write `return ref mut self.dense[i]`") |
| A6: a container of views is a view (4.3, saved) | **kept** (AN-24), with the attack's "erasure keeps the limit" and "one limit per view field" | |
| A6: an object holding a view carries the view's limit (4.4, saved) | **changed**: a class field of views says its source with `from` | with one-owner objects there are no other handles to carry the limit; a written `from` is readable and the attack's drop-check fix is not needed for `from static` or a `let` field of the same object |
| A7: views of a value's own heap contents, any write makes them unusable (5.3, saved) | **changed and simpler**: the source field is `let`, so nothing writes it; no unusable state, no index-write rule | the attack's index-write hole (section 5) cannot occur |
| A7: derived `clone` re-points each view (rule 7) | **thrown away for now**: `copied` of such a value is refused with a message | the minimal safe form; the re-pointing is a good part (one compare and one add per view inside a clone that already copies) to add if programs want it |
| A22 (temporaries), A23 (moving a field out) | not area B (area A and area C) | |
| Counted views (5.5) | **not used** | the static answer costs nothing; good parts already listed by last run (grouped retains, run-length release, 16-byte word) |

Good parts kept from what was thrown away: re-pointing views in a clone (A7 rule 7); computing the
result's source from the body (A5 3.3), now used to write the exact `from` line into the error
message; last run's E2140 help for the temporary (`p22e`), which is area A's.

## 6. Points for the combiner

1. **No word at a call site for a changed argument.** `first(xs) += 1` and `bump(a)` change the
   caller's `xs`/`a` with no word in the calling line; the word is on the callee's first line
   (`ref mut xs`), as `v.push(x)`'s change is said by the method. The vocabulary has no call-site
   word; if the owner's test is applied to arguments, every area needs one answer.
2. **`shared` objects**: AN-18's removal of run-time access words holds for one-owner objects only;
   area C's AN-48 decides what a `shared` object keeps.
3. **Reads across packages**: section 1 keeps `[BRW-10]`'s body-computed read set inside a package and
   uses `reads` (or "every field") across packages; area A's loops (AN-01..) use the same check.
4. **`@cached` and threads**: area E (once-flag or refusal).
5. **`from static`**: section 3.

## 7. Evidence

Every output above is copied from `target/redesign3/09-area-B/check_output.txt`. Summary:

| file | today |
|---|---|
| `o17a_class_note_in_loop.em` | E3020, exit 1 |
| `o17b_struct_pub_note.em` | exit 0 |
| `o18_class_mut_self_two_handles.em` | exit 0 |
| `o19_plain_self_writes.em`, `o20a`, `o20b`, `o20c` | exit 0 |
| `o21_lazy_cache.em` | E3023, exit 1 |
| `o22a_two_words.em`, `o22c_two_words_flat.em` | E3025, exit 1 |
| `o22b_ecs_pool.em` | E3060, exit 1 |
| `o23_call_result_place.em` | E2140 twice, exit 1 |
| `o24a_handles_in_array.em`, `o24b_views_in_local_array.em` | E3063 twice, exit 1 |
| `o25a_document.em` | E3063, exit 1 |
| `o25b_document_one_view.em` | E3024, E3060, exit 1 |
| `o26a_str_field_literal.em` | exit 0 |
| `o26b_str_field_string.em` | E3063, exit 1 |
| `o26c_str_return.em` | E3021, exit 1 |
| `o27a_ref_mut_type.em` | E3023, E2020, exit 1 |
| `o27b_help_form.em` | E2020, E3027, exit 1 |
| `o27c_ref_type.em`, `o28_ref_mut_rebind.em`, `o29_ref_string_to_str.em` | exit 0 |
| `o29b_ref_string_in_lambda.em` | E2020, exit 1 |
| `o30a`, `o30b`, `o30d` | exit 0 |
| `o30c_refcell.em` | L3011 warning, exit 0 |
| `o31a_init_struct.em`, `o31b_init_class.em` | E2100, exit 1 |
| `o32_init_mut_self.em`, `o33a_let_push.em` | exit 0 |
| `o33b_let_assign.em` | E2104, exit 1 |
| every `m*.em` (16 files, `m17`..`m33`) and `p_let_struct_field.em` | exit 0 |
| `c24.exe`, `c24_asan.exe` | checksum 30; no AddressSanitizer report |
| `c25.exe`, `c25_asan.exe` | checksum 204454; no AddressSanitizer report |
| `c30_asan.exe` | `ERROR: AddressSanitizer: heap-use-after-free` (AN-30's OLD) |
