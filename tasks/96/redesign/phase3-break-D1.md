# Phase 3, break D1: values and projections, attacked

Written 2026-10-24 by the phase-3 D1 breaker agent, against `tasks/96/redesign/phase3-design-D1.md`
(called "the design" below) and its C models in `target/redesign/bench/D1/`.

What was read: the whole design; `w1.c`..`w4.c` and `harness.h`; the design's four reference
programs and its Python check (`ref/ref1.c`..`ref4.c`, `py_d1.py` in the session scratchpad the
design names); spec rules `[OBJ-1]`..`[OBJ-5]`, `[RC-1]`..`[RC-3]`, `[OWN-1]`, `[OWN-2]`, `[CLO-2]`,
`[CLO-3]`, `[CLO-13]`, `[CORO-1]`..`[CORO-13]`, `[THR-1]`, `[THR-5]`, `[THR-11]`, `[THR-13]` in
`docs/spec-source/Ember_v0.9.10_Hardened_35.md`; ODR-286 in `tasks/95/odr-pending.md`; the probe
`target/analysis/ecs4/accept_change_during_loop.em`; `phase2-swift-mojo.md` (idea 3, lines 680-698);
`phase2-rc-elision.md` (lines 45, 299, 300) and `target/redesign/phase2-rc-elision/results.txt`.

What was run:

- The four D1 models as the design agent built them (`w1.exe`..`w4.exe`), once each, and the four
  reference programs once each. Only the checksums are used (section 3.1). The runs were single and
  unpinned, so the times they printed are not measurements and are not used anywhere.
- Two `python -I -c` one-liners (Python 3.14.2, no files written) for every Python behaviour this
  report quotes as "checked".
- One WebSearch and one WebFetch: EnTT's iteration rule and Java's substring change (sources at
  the end).

What was not done: nothing was built and no file was written except this report. No Ember program
was compiled or run. D1 has no implementation, so no D1 program below was checked by any tool:
"the rules accept it" always means "the design's text, quoted with its section, accepts it".

Default stance: a claim of the design without evidence is unproven.

---

## 0. The verdict

**D1 as written is not memory-safe.** Its own rule text admits use after free in single-threaded
safe code in at least six independent ways (B1, B2, B5, B6, B8, B9), a wrong-type read (B7), a data
race in its own A33 example (B3), and a data race with scoped tasks (B4). Every hole has a fix. Four
fixes are cheap: B1 costs nothing at run time, B5 and B7 one compare, B8 one allocation per frame.
The others cost something the design does not count: callers and callees
that fetch places again after calls (B6), deep copies or atomic counts for anything that crosses
threads through a lock or a `@sync` object (B3, B11), a new freeze rule for scoped tasks (B4), and a
second-class rule for generators and async frames that hold projections (B9), which is itself an
annoyance the design does not list.

**Annoyances not admitted.** The largest is that lists and dicts are values: three everyday Python
idioms lose updates silently and E-COPYLOST catches none of them (N1). A `for` variable writes
through to the list on plain assignment (N2). A23 is not removed, it moved to run time (N4). Threads
cannot write shared state through closures (N6). ASAP freeing runs a class's `drop` early, the Mojo
trap the design says cannot arise (N7). The A9 row's "one rare panic" undercounts by at least five
kinds (N10), and the A24 row is contradicted by the design's own examples (N11).

**The C models are faithful.** Each does every check, count update and copy the design's text says
the compiler emits, and exactly the workload's work; my one run of each reproduced the design's
checksums, equal to the reference programs'. The one large assumption is in W1: the view generator
is inlined into the loop, an optimisation section 1.5 does not list; without it D1-6 requires the
two pool projections to be fetched again on every resume (about 50 million times instead of 200
plus two per spawn) (M1). Smaller items: M2 to M6. None of the four workloads contains a program
shape that the safety fixes would make slower, so the models cannot show those costs (M5).

---

## 1. Breaking memory safety

Each entry: the program in D1 syntax, why the design's rules accept it (quoted), what happens, and
the fix with its cost.

### B1. E-EXCL's plain-data exemption lets a list and its own element both be changed (use after free)

```ember
fn grow(mut xs: Array[f32], mut x: f32):
    for i in 0..100:
        xs.push(0.0)          # the buffer grows: realloc moves it and frees the old block
    x += 1.0                  # x is still the address of the old xs[0]

fn main():
    v = [1.0]
    grow(v, v[0])
```

Why it is accepted. D1-5: a changed place "may not overlap another argument, except: the overlap is
plain data (numbers, and structs of numbers): two names for one plain value cannot free anything, so
they are allowed". The overlap of `v` and `v[0]` is `v[0]`, an `f32`. D1-7: "A `mut` or borrowed
parameter receives an address, which the callee cannot fetch again. For a local value, D1-5
guarantees nothing else reaches it during the call." `v` is a local, so nothing is pinned, and the
callee cannot fetch `x` again.

What happens. `push` goes through `buf_reserve_slow`, which calls `realloc` (in all four models);
`x += 1.0` writes four bytes into the freed block. The justification "two names for one plain value
cannot free anything" is true only when both names are the plain value; here one name is the list
that owns the buffer.

Fix. Say "both arguments are plain values" (equivalently: neither place contains the other through
a buffer). `grow(v, v[0])` becomes E-EXCL, as in Rust. No run-time cost. The run-time form
`grow(vs[i], vs[j][0])` is already covered: the element type of `vs` owns heap memory, so D1-5
inserts `if i == j: panic`.

### B2. A captured local counts as an unreachable local (use after free of a closure box, of an element)

```ember
fn main():
    hs: Array[fn()] = []
    fn make() -> fn():
        n = 0
        return fn():
            hs.clear()        # hs is main's local, captured by reference
            n += 1            # n lives in make()'s box; only this closure held that box
    hs.push(make())
    hs[0]()
```

Why it is accepted. D1-7 calls a stored closure "through a copy of the closure value whose box is
retained for the call" only "for a place reached through an object handle or a module variable".
`hs` is a local, and for a local value "D1-5 guarantees nothing else reaches it during the call".
But section 1.6 changes `[CLO-2]` to "captures are by reference, Python-like", so the callee reaches
`hs`. The call has no arguments, so D1-5 has nothing to compare.

What happens. `hs.clear()` drops the only copy of the closure value, the count of `make()`'s box
reaches zero and the box is freed (exactly the path `hbox_release` takes in `w4.c`); `n += 1` writes
into it. Python prints `1` (checked).

The same hole with a lend instead of a call of a stored closure:

```ember
struct Squad:
    units: Array[int]

fn grow(mut s: Squad, f: fn()):
    f()                       # empties squads: the buffer holding s is freed
    s.units.push(3)           # s is the address of squads[0] in that buffer

fn main():
    squads = [Squad(units=[1, 2])]
    grow(squads[0], fn(): squads = [])
```

E-EXCL compares places; the design never says it looks at what a closure argument captures.

Fix. A variable that a closure captures by reference and writes already lives in a box when the
closure escapes (table 1.3); every rule that says "object field or module variable" (D1-6 fetch
again, D1-7 copy, retain and pin) must also say "captured variable". Cost: a count pair per call of a
closure stored in a captured list (about 0.84 ns per pair, `phase2-rc-elision.md` line 299) and pins
for lends of its elements. The other route, making E-EXCL treat a closure's captures as arguments, is
Rust's closure-borrow error: the A12 annoyance comes back.

### B3. The design's own A33 example races on a count

```ember
@sync class Board:
    scores: Mutex[Array[String]]

fn top(scores: Mutex[Array[String]]) -> String:   # the design's A33 function, unchanged
    with g = scores.lock():
        return g[0]                                # "a copy (one count increment)"

fn main():
    board = Board(scores=Mutex(["ann"]))
    with scope = thread.scope():
        scope.spawn(fn():
            for _ in 0..1_000_000:
                s = top(board.scores)              # thread A
        )
        for _ in 0..1_000_000:
            s = top(board.scores)                  # thread B
```

Why it is accepted. D1-9 keeps every count on values non-atomic ("counts on values are never touched
by two threads and never need atomics") on the strength of move-or-deep-copy at a send. But section
2.1 says `return g[0]` is "a copy (one count increment)". The increment runs under the lock; the
matching decrement runs when `s` dies, after the lock is released.

What happens. The string buffer has one holder (the array) and count 1. Thread A: `top()` raises it
to 2 under the lock; A's `s` dies after the unlock and A loads 2. Thread B, under the lock, loads 2
and stores 3. A stores 1. There are two holders (the array and B's `s`) and the count says 1; when
B's `s` dies the buffer is freed while the array still points at it, and the next `top()` reads
freed memory. The same race needs no lock at all for a value in an immutable field of a `@sync`
object: two threads that copy `board.names` each increment the same count without synchronisation.

Fix, either (a) a copy that leaves a lock, or is taken from a `@sync` object, is a deep copy: A33's
"one count increment" becomes an allocation and a copy of the string; or (b) Koka's thread-shared
bit on buffers reachable from a `Mutex` or a `@sync` object, with atomic count operations when the
bit is set (`phase2-rc-elision.md` idea 2, line 45). An atomic pair cost about 5.8 ns against 0.84 ns
for a plain pair on phase 2's microbenchmark (`phase2-rc-elision.md` line 300). D1-9's "never need
atomics" is false as written.

### B4. A scoped task reads a value its parent is changing

```ember
fn main():
    xs = [1, 2, 3]
    with scope = thread.scope():
        scope.spawn(fn() => print(sum(xs)))    # the task reads xs through a read projection (D1-9)
        for i in 0..1_000_000:
            xs.push(i)                         # the parent grows xs at the same time
```

Why it is accepted. Today `[THR-11]` keeps each borrow a task makes "live until the end of the block,
so the ordinary borrow rules (`[BRW-2]`) reject ... one that writes a place another reads". D1
deletes `[BRW-1]`..`[BRW-11]` (section 1.6), and D1-9 only says "A scoped task may read the caller's
values through a read projection". D1-5 checks `xs.push(i)` alone and finds no overlap; D1-6 fetches
again only for projections held by the same thread.

What happens. A data race on `xs`'s length and buffer pointer; `push`'s `realloc` frees the buffer
the task is summing.

Fix, either (a) lending a value to a scoped task pins it for the whole scope with a pin that also
forbids element writes, so the parent's first change panics (only the parent writes the pin, so it
needs no atomics; cost a pin per lent value per scope), or (b) a compile-time rule that a value a
task captured is read-only in the scope block. (b) is a borrow rule, small and lexical, but a borrow
rule; the design must pick one.

### B5. Nested lends clear each other's pin (use after free)

```ember
struct Player:
    hp: int
    bag: Array[int]

    fn update(mut self, team: Team):
        team.heal_first()          # lends team.players[0]: sets the pin bit, then CLEARS it
        team.players.clear()       # the bit is clear, so allowed: every element is dropped
        self.bag.push(1)           # self is still the address of team.players[i]; its bag was freed

fn heal(mut p: Player, t: Team):
    p.hp += 1

class Team:
    players: Array[Player] = []

    fn heal_first(self):
        heal(self.players[0], self)     # heal may reach the team: players is pinned for the call

    fn tick(self):
        for p in self.players:
            p.update(self)              # players is pinned for this call
```

Why it is accepted. D1-7: "the list is pinned for the call: a bit in the list's count word"; table
1.3: "set and clear a bit in the count word (two stores)". One bit cannot count two lends.

What happens. After `heal_first` returns, the outer lend is unprotected; `clear()` drops the
elements and frees `self.bag`'s buffer; `push` writes into it.

Fix. A pin counter (add and subtract instead of set and clear), the same cost. The counter takes
bits from the 32-bit count word, so the count then needs an overflow check on each increment, as
`[RC-1]` has for objects: D1 shares one buffer among all the words of a string (A7, A25), one
increment per word, and a count that wraps frees the buffer while words still use it. Neither the
design nor the models check a buffer count for overflow.

### B6. What a lend does not protect (use after free in the callee)

D1-7 protects a `mut` argument only when it "lies inside a list", and then only against a change of
the list's size. Four ways around it:

**(a) The lent element is replaced through another name**, which D1-7 allows on purpose ("writing
elements through other names is allowed and seen by the callee"):

```ember
struct Player:
    hp: int
    bag: Array[int]

    fn fill(mut self, team: Team):
        for item in self.bag:        # a projection into self.bag's buffer
            team.respawn(0)          # team.players[0] = Player(...): allowed while pinned
            item += 1                # if self is team.players[0], that buffer was just freed

class Team:
    players: Array[Player] = []

    fn respawn(self, i: int):
        self.players[i] = Player(hp=100, bag=[])

    fn tick(self):
        self.players[0].fill(self)
```

Inside `fill`, whether `item` is fetched again after `team.respawn(0)` depends on whether that call
"can change" `self.bag`. D1-7's reason for passing an address says no ("For a local value, D1-5
guarantees nothing else reaches it during the call"); "seen by the callee" says yes. The design never
states the callee's rule, and the callee cannot know which kind of argument it received. If it keeps
the address (the cheap reading), use after free. If it fetches again (the safe reading), every
function with a `mut` struct parameter must treat every call that is not quiet (any `dyn` call,
virtual call, closure call, or call that may reach an object) as a change to its parameters, and
fetch again and repeat uniqueness tests after it. That cost is not in table 1.3 and not in the models.

**(b) A `mut` argument that is a struct field of an object, or a `Map` entry, has no protection at
all:**

```ember
class Scores:
    by_name: Map[String, Stats] = {}

fn add_win(mut s: Stats, all: Scores):
    all.by_name["newcomer"] = Stats()    # may grow and rehash: the old table is freed
    s.wins += 1                          # s is the address of "ann"'s entry in the old table

fn main():
    sc = Scores()
    sc.by_name["ann"] = Stats()
    add_win(sc.by_name["ann"], sc)
```

It is not E-EXCL: for objects the design accepts an element passed together with its owner
(`for p in self.players: p.update(self)`, section 6.1 item 5). Of D1-7's three cases, none applies:
the argument is not borrowed, not a stored closure, and a `Map` is not a list. So it is passed by
address and nothing is pinned. The same holds for `h.stats.total(h)` where `stats` is a struct field
of class `Hero` and the callee runs `h.stats = Stats(...)`.

**(c) The owner of a pinned list dies, or an outer list changes:** `update(g.team.players[0], g)`
where `update` runs `g.team = Team()`. The old `Team` dies and drops its pinned `players` buffer. The
design does not say what releasing a pinned buffer does. If the release tests the whole count word
(pin bit set, so not zero), the buffer is not freed and, since the unpin only clears a bit, never
freed: a leak. If the release masks the pin bit, the buffer is freed under the callee: use after
free. The same for `team.squads[1].members[3]` lent while the callee removes `team.squads[1]` (only
`members` is pinned).

**(d) A release runs user code inside a library method.** `REGISTRY.remove(0)` on a module
variable, where the removed element holds the last handle to an object whose `drop` pushes to
`REGISTRY`: the release inside `remove` grows the very list `remove` is working on. D1-2's "Values
never run code when they die" is false for any value that holds a handle: the release runs the
class's `drop`.

Fix. One rule stated for the callee: a `mut` parameter is a path whose root may be reached by any
call or release that is not quiet; after one, the callee fetches its parts again (with B7's checks).
Plus: pin every collection on the lent path, `Map` included; retain every object on the path for the
call (a count pair per object hop, about 0.84 ns each, `phase2-rc-elision.md` line 299); and a
pinned buffer's release defers the free to the unpin. None of it is measured.

### B7. Fetching again through a `match` arm, a `cast` or a weak read (wrong type)

```ember
enum Slot:
    Empty
    Full(items: Array[int])

fn main():
    state = Slot.Full([1, 2, 3])
    match state:
        Full(items):               # items is the path state.Full.items (D1-4 form 4: in place)
            state = Slot.Empty     # changes what items points into
            items.push(4)          # items is fetched again from its path
```

Why it is accepted. D1-6 fetches a projection again "from the path ... with a bounds check (or a key
lookup for a `Map`)". A path through a variant needs a tag check, a path through `cast p as Pool[T]`
a type check (the W1 model does this one: `if (d->vt != vt) d1_panic(...)`), and a path through
`Some(t) = self.target` on a `weak` field a liveness check. The rule names none of them.

What happens. Without the check, `items` is read from `Empty`'s payload bytes and `push` writes
through whatever is there. With the check, a panic (see N4).

Fix. One compare per variant, cast or weak step fetched again. Cheap.

### B8. An escaping closure that reads a variable written later

```ember
fn make() -> fn() -> int:
    n = 1
    f = fn() => n          # escapes (returned) and only reads n
    n = 2
    return f               # Python: make()() == 2 (checked)
```

Why it is accepted. D1 boxes a capture only when "an escaping closure ... writes" it (section 1.6,
`[CLO-2]`; table 1.3, "boxed captured variable: a closure that escapes and writes a captured
variable"), and "captures are by reference". So `n` stays in `make`'s frame and `f` reads it after
the frame is gone. Today's `[CLO-13]` copies such a capture only when it "is not assigned after the
closure is created" and refuses the rest; D1 replaced that rule.

Fix. Box every variable that an escaping closure captures and that anything writes after the
capture: one allocation per frame, the box the design already has.

### B9. A generator (or async) frame holding a projection outlives its root

```ember
gen fn evens(xs: Array[int]) -> Generator[int]:   # xs: a borrowed parameter, an address (D1-7)
    for x in xs:
        if x % 2 == 0:
            yield x

fn make() -> Generator[int]:
    local = [2, 4, 6]
    return evens(local)        # the frame keeps a path rooted at local; local dies here

fn main():
    g = make()
    print(next(g))
```

Why it is accepted. A generator's frame is a value (`[CORO-1]`; `[CORO-5]`: "an ordinary move-only
value that may live in a local, a field, an `Array`"; D1-1 makes every value copyable). Today
`[CORO-6]` makes a frame holding a parameter's reference "a view type bound by their regions"; D1
deletes regions, and its replacement ("a projection has no type, so it can never be stored,
returned") speaks of projections, not of a frame that contains one. ASAP freeing (D1-2) ends `local`
at its last use. The A14 row ("a suspended frame holds paths, not addresses, so moving it is
harmless") covers paths into the frame's own locals, not into the caller's.

What happens. `next(g)` fetches `xs` from a root that died with `make`'s frame.

Fix. A generator or async frame whose parameters are projections is second-class: only the `for` or
`await` that creates it may consume it; it is never stored, returned or copied. No run-time cost, but
`it = reg.view[A, B]()` and returning a generator over a parameter are refused, an A5/A14 annoyance
the design does not list (Python allows both). Passing a generator's borrowed parameters as copies
(a count increment) would keep `evens` storable, but cannot work for `view[A, B](mut self)`: a `mut`
parameter is a projection.

### B10 (minor). A stale projection can name another element

D1-6 promises: "if the elements moved, using the projection panics instead of naming another
element". It does not hold:

- The shift version is per buffer, and the models' copy-on-write starts the copy at 0 (`nb->shifts =
  0` in `buf_reserve_slow`, all four models). A projection recorded at version 1, then a
  copy-on-write, then one `remove`, reads version 1 again: no panic.
- `pop`, `truncate`, a shrinking `resize` and element assignment are not in D1-6's list of shifting
  operations: `with b = xs[2]: xs.pop(); xs.push(e)` leaves `b` naming `e`, silently.
- The version is a `u32`; it wraps after 2^32 shifting operations.

Bounds checks keep this memory-safe. Fix: carry the version across copy-on-write (or compare the
buffer's address too) and list every operation that moves or replaces elements.

### B11 (cost). Moving a value to another thread needs a walk of all of it

D1-9 deep-copies "when its buffers are shared with values the sender keeps". A top-level count of 1
says nothing about the buffers inside: an `Array[String]` whose strings share buffers with strings the
sender keeps. A send that checks only the top count races exactly as B3 does. A correct send walks
every nested buffer, O(size of the value) per send (Koka's "one-time marking walk when an object
graph is sent", `phase2-rc-elision.md` line 45). And a list of words split from one text cannot be
moved at all: every word's count is above 1, so the send deep-copies each word separately. Not
counted in the design.

### B12 (residue, unsafe code). ASAP freeing and raw pointers

`p = buf.as_ptr(); c_write(fd, p, n)`: the last Ember use of `buf` is `as_ptr()`, so D1-2 frees it
before C reads it. Today `[RC-3]` says unsafe code that keeps a raw pointer "keeps a handle to it alive
for as long as it uses the pointer", with `mem.keep_alive`; D1 must carry this over to values. It is
inside `unsafe` (A20), so it is not a safe-code break.

---

## 2. Annoyances the design keeps or adds and does not admit

### N1. Lists and dicts are values: Python aliasing idioms lose updates silently (A11, A2)

```ember
matrix = []
row = []
matrix.append(row)        # a copy: count + 1
row.append(1)             # row copies the buffer first; matrix stays [[]]

d: Map[String, Array[int]] = {}
lst = d.get("a")
if lst is None:
    lst = []
    d["a"] = lst          # a copy
lst.append(1)             # lost: d["a"] stays []

inv.items().append("x")   # items() returns a copy; the append is lost
```

Python gives `[[1]]`, `{'a': [1, 2]}` for two appends, and `['x']` (all checked). E-COPYLOST fires for
none of the three: it needs "a local initialised by copying a place (not a temporary)", and here the
local was a literal or a function result, or there is no local. Section 6.1 item 1 names only struct
copies ("`=` copies a struct"). A partial fix: E-COPYLOST also for "a local stored or passed whole
and then changed in part, with the change never seen", and for any change made to a temporary
(`inv.items().append("x")` can never be seen). Python's meaning cannot be had without aliasing.

### N2. A `for` variable writes through on plain assignment

```ember
for line in lines:
    line = line.strip()       # D1: replaces lines[i]; the loop binds each element in place
    if line == "":
        continue
for x in counts:
    x += 1                    # D1: changes counts
```

Python leaves both lists unchanged (checked: `['  a ', 'b ']` and `[1, 2, 3]`). In D1 both write into
the list, or are refused when the list is a borrowed parameter. The design knows rebinding is the
issue (open question 2: "Python rebinding never writes through") but binds every loop variable in
place (D1-4 form 2). Fix: plain assignment to a loop variable rebinds it (a local copy from then on);
only writes to its parts (`p.hp -= 1`) go in place. No run-time cost.

### N3. E-COPYLOST refuses a correct program, and its fix-its are wrong for it (A17)

```ember
fn half_hp(enemies: Array[Enemy]) -> int:
    boss = enemies[0]         # a scratch copy, on purpose
    boss.hp //= 2
    return boss.hp            # the change is seen, but boss is "never used whole afterwards"
```

D1-8 makes "never used whole afterwards (not stored, passed, returned or copied)" an error. Its first
fix-it, `enemies[0].hp //= 2`, does not compile (`enemies` is borrowed) and would change the
caller's data. Fix: a read of a changed part counts as a use.

### N4. A23 is not removed; it moved to run time

```ember
match self.state:
    Walking(path):
        self.state = State.Idle       # stop walking
        for step in path:             # finish the steps already planned
            self.log_step(step)
```

Python does this, and Ember today does it with `mem.take`. In D1, `path` is the path
`self.state.Walking.path`; after the assignment it is fetched again: with B7's fix it panics, without
it the read is garbage. The A23 row says "removed".

### N5. During a lend, a copy of the list is not its own value, and writes to it panic

During a D1-7 lend, `snapshot = team.players` is allowed (a count increment). The callee's writes
through its address go into the shared buffer, so `snapshot` changes after it was taken (D1-1:
"`b = a` gives `b` its own value"). And any write to any copy of the list during the lend needs a
copy-on-write, which "panics" while pinned. Section 6.1 item 5 admits only size changes.

### N6. Threads: closures cannot share state, and in-place parallel work is gone (A15, A12)

```ember
results: Array[int] = []
with scope = thread.scope():
    for i in 0..4:
        scope.spawn(fn(): results.push(i * i))
print(results)               # Python threads: [0, 1, 4, 9] (checked)
```

D1-9 lets a task only read the caller's values; a closure sent to a thread is a value whose box is
shared with the sender, so by D1-9 it is deep-copied (the pushes are lost silently) or refused; the
design says which in neither case. Two tasks that each change one half of an array in place (today's
`[THR-11]` mutable borrows) have no D1 spelling: move the halves out and back (two copies) or use a
`Mutex`. The A15 row says "partly removed".

### N7. ASAP freeing runs a `drop` early (the Mojo trap)

```ember
class FileLock:
    fn drop(self):
        unlock(LOCK_PATH)

struct Job:
    lock: FileLock
    name: String

fn run():
    job = Job(lock=FileLock(), name="nightly")
    print(job.name)           # last use of job: D1-2 frees it here, and the handle's release runs drop
    write_report()            # runs without the lock
```

D1-2: "A value's memory is freed after its last use", and "the early-free trap Mojo documents ...
cannot arise: there are no guard values". A struct that holds the last handle to a class with `drop`
is a guard value. `[RC-3]`, which D1 keeps, says an object is deinitialised "never earlier" than its
last owner's scope end. Phase 2 idea 3 (`phase2-swift-mojo.md` lines 680-698) limited ASAP to values
"whose drop only frees memory" for this reason; D1 dropped the limit. Fix: ASAP only for values that
transitively hold no handle to a class that has or may have a `drop` (`dyn` and open classes "may").
It gives back some of the peak-memory gain.

### N8. Memory held longer than C would hold it (cost)

- A word shares its whole text's buffer (A7, A22, A25): one kept five-letter word keeps the text
  alive. Java's `substring` worked this way and was changed to copy in 7u6 because of exactly this
  leak (Java Specialists issue 230; mjtsai.com 2015-07-20).
- One box per call frame for all captured variables (`w4.c`: "one counted box per call frame"): a
  small closure that outlives its frame keeps every captured variable of that frame alive, including
  large arrays only other closures used.
- A `String` that shares a buffer is (buffer, start, length), three words (the A7 row), not the
  "8 bytes (one pointer)" table 1.3 gives for a collection value.

### N9. An escaping closure over a `for` element changes a private copy (A12)

```ember
for p in players:                     # Player is a struct
    on_hit.push(fn(): p.hp -= 1)      # [CLO-13] as changed by D1: captures a copy of p
```

Every callback changes its own copy; no error is given (E-COPYLOST covers locals only). Python,
where `p` is an object, changes the player.

### N10. More run-time memory-model panics than "one rare panic" (A9)

The A9 row says "the only run-time borrow panic left is a pinned list changing size during a lent
call". By the design's own text there are at least: two `mut` elements at run-time indices that are
equal (D1-5); a stale projection after a shifting change (D1-6); removal during a plain loop (A4);
a pinned list's size change and a pinned list's copy-on-write (D1-7); `view[T, T]`, whose two `with`
projections of one pool hit D1-5's `i == j` check; and, with B7's fix, a variant that changed under a
projection.

### N11. The kind still decides whether a program compiles (A24 not removed)

The A24 row: "the kind decides copy or share and the cost, never whether it compiles". But
`for p in players: p.update(players)` is E-EXCL for a local list of structs (section 2.1) and compiles
when the list belongs to an object (section 6.1 item 5); `boss = enemies[0]; boss.hp //= 2` is
E-COPYLOST for a struct and compiles for a class (section 3.6, P2).

### N12. The ECS view allows despawning only the current entity

EnTT's documentation: "For all the other entities, destroying them or removing their iterated
components is not allowed and can result in undefined behavior" (`docs/md/entity.md`, "What is allowed
and what is not"). D1's view (section 3.5) walks backward with swap-and-pop: destroying an entity not
yet visited moves the last element (already visited, or spawned in this step) into its slot, and it
is visited a second time. Memory-safe, silently wrong; the design says only that the view "allows
despawning in place". Fix: a deferred-destroy list or tombstones in the library (a cost on destroy).

### N13. Borrowed arguments through objects are snapshots, `mut` arguments are live (cost)

D1-7 passes a borrowed argument reached through an object as "a copy (struct bits, plus a count
increment per buffer)". So `fn show(st: Stats, g: Game): g.reset(); print(st.hp)` prints the old
`hp`, while a `mut` parameter "sees" the change. Python sees it in both. It also costs a copy of the
whole struct plus a count pair per buffer on every such call, where C passes a pointer. Not measured.

---

## 3. The C models against the design and the workloads

### 3.1 Checksums (my runs, one each)

| Program | Checksum printed | Design's table (section 5) |
|---|---|---|
| `D1/w1.exe` | 2111831520 | 2111831520 |
| `D1/w2.exe` | 647034814 | 647034814 |
| `D1/w3.exe` | 9288233970 | 9288233970 |
| `D1/w4.exe` | 34946993000 | 34946993000 |
| `ref1.exe` .. `ref4.exe` | the same four values | |

The reference runs also printed `next=1856653` (W1: 856,653 spawns), `depth=148749741` (W2) and
`destroyed=9173` (W3), matching the design's section 5. I read the four reference programs against
the workload text and found them faithful (visit sets, despawn timing, order of `h_rand()` calls,
checksums), so a matching checksum is real evidence that each model does the workload's work. I did
not rebuild the binaries (writing files outside this report is not allowed), so that the `.exe`
files match the `.c` files (the design says only comments changed after the build) is the design's
claim, not mine.

### 3.2 Check by check

| Model | What the design says is emitted | In the model |
|---|---|---|
| W1 | uniqueness tests (pools list, each box, dense before writing, every library change); bounds checks on `dense[slot]`, inside `remove`, `sparse[moved]`; type-table compare per `pool[T]()`; `remove` through the type table; shift bumps; fetch again after the destroy branch | all present; `dense[k]`'s range checked once per step and again after each destroy (allowed by section 1.5 item 5) |
| W2 | 16-byte header; weak read tests the strong count and clears a dead target; weak stores adjust weak counts; retain on copy into `children`; `old` retained across the release; bounds checks on `nodes[h_rand() % i]`, `nodes[i]`; `index_of` scan; swap-remove bumps the shift version | all present |
| W3 | virtual call; weak read per target read; `retain` tests uniqueness, compacts, releases, bumps the version; dropped objects release their weak target and weak unit; bounds check in the setup | all present |
| W4 | one box per `make_handler` call and one for `main`; two closures per box; copy the closure out and retain its box around each call; `remove` checks the index, shifts, bumps, releases both closures; division-by-zero check | all present |

Things that look like shortcuts and are not:

- W3's `live.retain(fn(t) => t.health > 0)` is inlined with no indirect call per element. `[CLO-3]`,
  which D1 keeps, makes a `fn(...)` parameter a bound: "each argument monomorphises the callee, and a
  call through it is a direct call".
- W2 does not retain `child` across `node_release(removed)` while it retains `old`. `child` came from
  `nodes`, which nothing in that code can write, so `[RC-3]` allows the elision; `old` came from a
  weak read, which proves nothing.
- W4 retains only `on_event`'s box for a dispatch: only that closure is called.
- W1's checksum loop reads the Position pool without uniqueness tests; the design defines no
  one-type view, and this runs once.

### 3.3 Shortcuts that would flatter the numbers

**M1 (major). W1 assumes the view generator is inlined into the user's loop.** The model fetches the
two pool projections again only after the destroy branch: twice per step and twice per spawn. That is
right only when the compiler sees the loop body and knows nothing else touched `reg`. Section 1.5's
six analyses do not include inlining a generator, and `[CORO-4]` compiles a generator to a state
machine. Without inlining, D1-6 requires the generator to fetch `a` and `b` again on every resume,
because the caller may have changed `reg` while it was suspended: two pool-accessor runs (each a
uniqueness test on `pools`, a range walk, a type-table compare and a box uniqueness test, the model's
`reg_pool`) per visited entity. W1 visits exactly 500,000 entities per step (the 500,000 even ids
start with both components, and each despawn of a visited entity is matched by a spawn with both),
so 50,000,000 resumes over 100 steps, against the model's 200 plus two per spawn (1,713,306 for
856,653 spawns, my `ref1` run). UNMEASURED. Fix: list generator inlining as a required analysis, and
give the measurer a second W1 model with an out-of-line generator.

**M2 (minor). W1 skips uniqueness tests after the destroy branch** ("the calls left what they changed
unique"). That needs an interprocedural uniqueness summary of `destroy` and `emplace` (section 1.5
item 2, unbuilt). It touches only the 856,653 spawns.

**M3 (minor). No teardown at the end of `main`.** Section 5 says "the compiler skips it" because no
object has a user `drop`. Section 1.5 does not list that analysis. It saves freeing 1,000,000 nodes
in W2 and 100,000 objects in W3 (counts from the workload text); peak memory is not affected. The
reference programs skip teardown too, so D1 against them is like with like.

**M4 (minor). Integer overflow checks are left out** of every model (stated). Fair between memory
models and against plain C, but the D1 numbers are not what an Ember build would run: `[TYP-8]`
checks stay in the language.

**M5 (minor for these four workloads). The models carry none of the costs the safety fixes add.** No
workload has a function with a `mut` struct parameter that calls through `dyn`, a virtual method or
a closure, a closure stored in a captured list, a lend through an object, a `Map` entry lent, a
generator stored in a variable, or a thread. Under B6's fix, W1's `Registry.destroy(mut self)` would
fetch `self.pools` again after each of its two `dyn` `remove` calls (the model keeps `pb`), 856,653
times; nothing else in W1 to W4 changes. The cost of the fixes needs a workload that has those
shapes, for example a physics step that calls a stored callback for each body it changes through a
`mut` parameter.

**M6 (minor). W1 uses one `pool_remove` for both pool types**, so the type-table call in `destroy`
always goes to the same target. A real build has two functions with identical code, which a linker
may or may not fold. 1,713,306 calls in all (two per spawn), so negligible.

---

## 4. What D1 has to change, with the cost of each change

| Change | Fixes | Cost |
|---|---|---|
| E-EXCL's exemption only when both arguments are plain values | B1 | none |
| captured variables count as object fields in D1-6 and D1-7 | B2 | a count pair per call of a closure stored in a captured list; pins for its elements |
| a copy out of a lock or a `@sync` object is deep, or shared buffers use atomic counts | B3, B11 | a full copy per copy-out, or 5.8 ns against 0.84 ns per count pair on shared buffers (phase2-rc-elision.md line 300) |
| a scoped task's captures are frozen in the parent until the join (pin or lexical rule) | B4 | a pin per lent value, or a small borrow rule |
| the pin is a counter; buffer counts check overflow | B5 | one compare per copy |
| callee rule: `mut` parameters are fetched again after calls and releases that are not quiet; every collection on a lent path pinned, every object retained; pinned buffers free at unpin | B6 | refetches in every function with `mut` parameters that calls through `dyn`, virtual methods or closures; a count pair per object on a lent path; unmeasured |
| tag, type and liveness checks when a path is fetched again | B7 | one compare per step |
| box any variable an escaping closure captures and anything writes later | B8 | one allocation per frame |
| generators and async frames with projection parameters are second-class | B9 | none at run time; refuses storing a view or returning a generator over a parameter |
| keep the shift version across copy-on-write; list every element-moving operation | B10 | none |
| ASAP only for values with no handle to a class that may have a `drop` | N7 | some peak memory |
| assignment to a loop variable rebinds it | N2 | none |
| E-COPYLOST counts a read of a changed part as a use, and covers stored-then-changed locals and changes to temporaries | N1 (partly), N3 | none |

---

## Sources

- The design: `tasks/96/redesign/phase3-design-D1.md` (rules D1-1..D1-9, tables 1.3, sections 1.6, 2,
  2.1, 3.1..3.6, 5, 6.1).
- C models: `target/redesign/bench/D1/w1.c`..`w4.c` and their `.exe` files; `target/redesign/bench/harness.h`.
- Reference programs and Python check named by the design:
  `C:/Users/ism19/AppData/Local/Temp/claude/C--Users-ism19-Code/ae3a2898-3f0f-4361-99e2-9bbbb0976123/scratchpad/ref/ref1.c`..`ref4.c`
  and `.../scratchpad/py_d1.py`.
- Spec: `docs/spec-source/Ember_v0.9.10_Hardened_35.md`: `[OBJ-1]`..`[OBJ-5]` and `[RC-1]`..`[RC-3]`
  (section VIII.1, VIII.2), `[OWN-1]`, `[OWN-2]`, `[CLO-2]`, `[CLO-3]`, `[CLO-13]`, `[CORO-1]`..`[CORO-13]`,
  `[THR-1]`, `[THR-5]`, `[THR-11]`, `[THR-13]`.
- `tasks/96/redesign/phase2-rc-elision.md` lines 45 (Koka's thread-shared bit and marking walk), 299
  (0.84 ns per plain count pair), 300 (5.8 ns per atomic pair); `target/redesign/phase2-rc-elision/results.txt`.
- `tasks/96/redesign/phase2-swift-mojo.md` lines 63, 466-470 (Mojo's ASAP trap), 680-698 (idea 3:
  ASAP only for memory-only values).
- ODR-286: `tasks/95/odr-pending.md`; probe `target/analysis/ecs4/accept_change_during_loop.em`.
- EnTT, "What is allowed and what is not":
  [docs/md/entity.md](https://raw.githubusercontent.com/skypjack/entt/master/docs/md/entity.md).
- Java's substring change in 7u6: [Java Specialists, issue 230](https://www.javaspecialists.eu/archive/Issue230.html);
  [mjtsai.com, "Java Strings No Longer Share Storage"](https://mjtsai.com/blog/2015/07/20/java-strings-no-longer-share-storage).
- Python 3.14.2 checks (run with `python -I -c`, no files): loop-variable rebinding, late binding of
  a closure, list aliasing (`row`, the bucket idiom, a getter), a closure that clears its own list,
  threads appending to a shared list.

Tools: clang was not needed (nothing built). Python 3.14.2, WebSearch and WebFetch were used. No
tool was missing. Nothing was downloaded or installed; no Ember program was compiled or run.
