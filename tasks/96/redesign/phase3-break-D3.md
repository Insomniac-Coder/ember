# Phase 3, break D3

Written 2026-10-24 by the phase-3 adversary for design D3 (`tasks/96/redesign/phase3-design-D3.md`,
"prove it, or check it"). The task: break its memory safety, find the annoyances it does not admit,
and check its four C models (`target/redesign/bench/D3/w1.c` to `w4.c`).

How to read this:

- **No D3 compiler or checker exists.** Every break below is an argument from the design's own
  rules, quoted with their line numbers in `phase3-design-D3.md`. `ember check` tests today's rules,
  not D3's, so it was not used. A program here is "accepted by D3" when no rule the design states
  refuses it; where the design is silent, the report says so and gives the readings.
- One measurement was made: a count-only replay of W1's despawn schedule (inline Python, no file
  written, no model run). Its numbers are marked "(count replay)".
- No C model was built or run. No tool was missing.
- "UNVERIFIED" marks an estimate or a claim that could not be checked.

---

## Verdict

D3 is not memory-safe as written. Its two run-time mechanisms each cover only part of the hazard:

- **WATCH** covers references *stored* in fields and containers. It does not cover the references a
  program uses most: parameters, `self`, locals and interior aliases of an object that a call may
  destroy (break 1.1). The design's own C sketches treat those as raw pointers.
- **PIN** is specified for append and remove only. `pop()` that returns the value (1.2), element
  replacement, insert, sort and swap (1.8), and a settle that runs `drop` code (1.7) are not, and
  the obvious implementations of each leave a dangling pointer.

Four more holes are in the rules themselves: copying a value that holds an object's home makes two
homes (1.3); classes with and without a generation word share slab slots, so a stale watcher can
pass its check on an object of another class (1.4); scoped tasks and `@parallel` lose their race
freedom when `[BRW-1]` and `[BRW-11]` stop being errors (1.5); and hot reload can widen an effect
summary that callers were compiled against (1.6).

Every hole has a fix. The fixes cost what D3 set out to remove: more run-time panics (a generation
check on every non-home reference that lives across a destroying call), hidden copies (`pop` under a
pin), an object-level keep-alive counter (partial reference counting), or compile errors (threads).

The biggest annoyance D3 adds is not admitted: **an object's owner is the first place it is stored**,
so the order of two equal-looking lines decides when objects die. Swapping two adjacent lines of the
design's own W2 program turns it from "no check at all" into a panic in the first pass (2.1); the
standard Python parent/child idiom makes the child own the root (2.1). The claim that struct versus
class "never" changes what panics (A24) is false (2.2).

The C models do the specified work and the checks the design names. Their shortcuts are small in
time; the one that matters is that `w1.c` implements a pool-level pin and settle (one counter for two
arrays, sentinel tombstones, a settle that repairs `sparse`) that the design never defines and that
an Ember-written `Pool[T]` could not express with the stated rules (3.1).

---

## 1. Breaking memory safety

### 1.1 A non-home reference across a call that destroys its object (critical)

The rules (design lines 115-127): an object "is destroyed when its home lets it go"; "If the compiler
sees a watcher of it used later **in the same function**, destruction waits"; "A watcher whose object
may die first is checked. The object keeps a 32-bit generation in its header; a **stored** watcher
keeps the generation it saw." The W3 sketch (lines 838-844) checks `self->target` once and then uses
`t` as a raw pointer, and calls `self` "unchecked". Nothing says what a parameter, `self`, a local or
an interior alias carries.

```ember
class Ship:
    hp: i32
    log: Array[String] = []

class Turret:
    target: Option[Ship] = None          # a watcher

class World:
    ships: Array[Ship] = []              # the home of every ship

    fn kill(self, s: Ship):
        self.ships.remove(self.ships.index_of(s).unwrap())   # the home lets `s` go: destroyed now

fn hit(w: World, s: Ship):
    s.hp -= 10
    if s.hp <= 0:
        w.kill(s)
    s.hp = 0                             # Python: fine, the object is still referenced

fn fire(w: World, t: Turret):
    match t.target:
        Some(s):                         # one generation check here
            hit(w, s)                    # `s` came from a watcher: no container the caller could pin
        None:
            pass
```

Why the rules let it through: `kill` does not use `s` after the removal, so its own static "wait"
rule does not fire. In `hit`, the destruction happens inside a call, so the "same function" rule
cannot apply. `fire` reached the ship through a watcher, so there is no home container it could pin
(pins protect borrows *into a container*, lines 157-164). `s` is a parameter; the design gives a
parameter no generation and the sketches pass object parameters as raw pointers. `s.hp = 0` writes
into a freed slab slot. Slots are reused "only for objects of the same size class" (line 125), not
the same class, so the next object of any 40-byte class can be sitting there: a type confusion, not
only a stale write.

The same hole, other spellings:

```ember
pos = t.target.unwrap().pos      # an interior alias of a struct field of a watched object
w.kill(t.target.unwrap())
pos.x += 1.0                     # writes into the freed object

fn update(self):                 # an override called from a loop over a *different* list
    if self.hp <= 0:
        self.world.despawn(self) # removes `self` from its home
    self.log.push("died")        # `self` is a raw pointer in the sketches
```

The design does protect one case: a borrow of a **container field** of a destroyed object is safe,
because "Destroying the object that holds a pinned container ... frees its memory and its fields
only when the pin ends" (lines 182-185). Plain fields and the object itself have no such rule.

Fixes, and what they cost:

- **(a) Check it.** Every non-home object reference (parameter, `self`, local, interior alias) that
  lives across a call whose effect summary may destroy that class reads the object's generation when
  it is formed and compares it after each such call. The read is safe because slab memory keeps its
  generation word (line 125). Cost: 1 load when the reference is formed and 1 compare per use after
  such a call (UNVERIFIED: not measured), and a new panic in `hit` where Python prints nothing wrong.
- **(b) Keep it alive.** Give such objects a keep-alive counter, as pins do for containers: the
  caller increments it for a reference that lives across a destroying call, `destroy` bumps the
  generation at once but frees the memory when the counter returns to zero. This gives Python's
  meaning (the write to `s.hp` lands on the removed object). Cost: a 4-byte counter in the header of
  every class that can be destroyed through another path while referenced, 2 read-modify-writes per
  such reference, 1 test per destroy. This is counting, limited to borrows across destroying calls;
  the design's "No reference counts" (line 138) would no longer hold as stated.

### 1.2 `pop()` under a pin hands out a value the tombstone still holds (critical)

The rules: while pinned "a removal marks the slot as a tombstone and is applied later" (line 166-167);
for A2, "removing it while `boss` lives leaves a tombstone, and `boss` keeps the removed value until
its last use" (line 355). `xs.pop()` hands its value to the receiver (line 113). The design never says
what `pop()` (or a `remove(i)` that returns the element) does under a pin.

```ember
struct Item:
    name: String

class Bag:
    items: Array[Item] = []
    trash: Array[Item] = []

    fn discard_last(self):
        self.trash.push(self.items.pop())   # pop hands the value over; push moves it into trash

    fn empty_trash(self):
        self.trash.clear()                  # drops the Item: frees its name buffer

fn main():
    bag = Bag()
    bag.items.push(Item(name="sword"))
    it = bag.items[0]           # `it` names the element (a struct: an alias, rule 1, line 88)
    bag.discard_last()          # through a method: `bag.items` is pinned with a counter
    bag.empty_trash()
    println(it.name)            # Python: prints "sword"
```

Why the rules let it through: `it` pins `bag.items`, so `pop` cannot free the slot; but it must give
the receiver the value *now*. An unpinned `pop` moves the bits out. If the pinned `pop` does the
same, the tombstone keeps a second copy of the `String` header pointing at the buffer, `empty_trash`
frees that buffer, and `it.name` reads freed memory. The settle then drops the tombstone's copy too:
a double free. For a `class Item` the slot holds an owning pointer with no generation (owning slots
carry none: W3's `objects` elements are plain `Obj *`, line 838), so `it.name` reads a freed slab slot (break 1.1).

Fix: under a pin, a value-returning removal of a type that owns memory **copies** the value out (a
clone) and leaves the original in the tombstone, dropped at settle. Cost: one clone (an allocation per
owned buffer) per pinned `pop`; `it` and the trash item are then two values, where Python shares one
object. For objects, fix (a) or (b) of 1.1.

### 1.3 Copying a value that holds an object's home makes two homes (critical)

The rules: the home of an object is "where the object is first stored ... the container element,
field ..." (lines 111-113); "A move the compiler cannot follow becomes a copy" (line 97); a struct
pushed to two lists gives the second a copy (line 940). Nothing says what a copy does to an owning
object reference inside the copied value.

```ember
class Unit:
    hp: i32

struct Squad:
    leader: Unit                      # the Unit is first stored here: this field is its home

fn main():
    a: Array[Squad] = []
    b: Array[Squad] = []
    s = Squad(leader=Unit(hp=10))
    a.push(s)                         # s moves into a; s follows it
    b.push(s)                         # a COPY (reported): b[0].leader is ... ?
    a.clear()                         # a[0].leader lets the Unit go: destroyed
    b[0].leader.hp -= 1               # use after free
                                      # and at b's drop: a second destroy (double free)

    objects: Array[Unit] = [Unit(hp=1)]
    backup = copy(objects)            # Python's list(objects): a shallow copy sharing the units
    objects.clear()
    println(backup[0].hp)             # same problem
```

Why the rules let it through: owning versus watching is decided "per slot" (line 363), and every
`Squad.leader` slot was decided owning (no generation, no tag). A bitwise copy duplicates the owning
pointer: two homes, so a use after free and a double free, which corrupts the slab's free list.

Fix: a copy turns every owning reference inside the value into a watcher (or clones the object).
Cost: the copied slot needs a tag bit and a generation word, so either every `Squad` grows by 4 to 8
bytes per object field or `Array[Squad]` gets two layouts (owning elements, watching elements) and
generic code over it is instantiated twice; and a copy is no longer a `memcpy`. A deep clone instead
breaks Python's shallow-copy meaning.

### 1.4 A stale watcher passes its check on an object of another class (critical)

The rules: "Freed slots keep their generation word and are reused only for objects of the same size
class" (line 125); "if nothing else needs it the class has no generation word at all" and "A final
class with no checked watchers has no header at all" (lines 131, 136-137). So a 32-byte slot can hold
a watched class (generation at offset 0) and then an unwatched class (its first field at offset 0).

```ember
class Bullet:                     # final, watched: a 4-byte generation at offset 0
    dmg: i32                      # offset 4
    trail: Array[f32] = []        # offsets 8-31 (data, len, cap): 32 bytes

class Tally:                      # final, never watched, never cast: no header
    n: u32                        # offset 0: where a Bullet keeps its generation
    a: int                        # offset 8
    b: int                        # offset 16
    c: int                        # offset 24: 32 bytes, the same size class

class Gun:
    last: Option[Bullet] = None   # a watcher

fn main():
    bullets: Array[Bullet] = []
    g = Gun()
    bullets.push(Bullet(dmg=1))   # slot S; a fresh slot starts at generation 1 (w3.c line 61)
    g.last = Some(bullets[0])     # the watcher saves generation 1
    bullets.clear()               # S: generation 2, onto the free list
    t = Tally(n=1, a=4096, b=0, c=8)   # reuses S and writes 1 over the generation word
    match g.last:
        Some(b):                  # S's word (1) equals the saved 1: the check passes
            b.trail.push(1.0)     # trail.data is Tally.a: a write to address 4096
        None:
            pass
```

Why the rules let it through: the check compares one word; the class that now owns the slot does not
keep a generation there. Small generations and small first fields make a match likely, not rare.
This breaks "a stale watcher can never match a new object" (line 126) and gives an arbitrary write.

Fix: never share a slab slot between a class that keeps a generation and one that does not (separate
slabs, or a per-page generation array outside the slots). Cost: some memory fragmentation, or a
generation load from a second cache line per check if the array lives outside the slot (UNVERIFIED).

### 1.5 Threads: `[BRW-1]` and `[BRW-11]` were what made scoped tasks and `@parallel` race-free (critical)

The rules: `[BRW-1]` "aliasing xor mutation" is "replaced ... aliasing plain data is allowed"
(line 321); `[BRW-3]`, `[BRW-5]`, `[BRW-11]` become "analysis facts ... never errors" (line 322);
"scoped tasks ... may borrow the parent's data read-only" (lines 289-291). Today's `[THR-11]` gets its
safety from `[BRW-2]`'s borrows plus `[BRW-1]` (spec line 3565-3567), and `[PAR-2]` only checks index
forms per *place name* (spec lines 3583-3589), trusting that two parameters do not overlap.

```ember
fn main():
    xs: Array[int] = [1, 2, 3]
    with s = thread.scope():
        s.spawn(fn(): println(xs[0] + xs[1]))   # the task borrows xs read-only: allowed
        xs[0] = 10                              # the parent writes it meanwhile: a data race
        xs.push(4)                              # or grows it: the buffer moves under the task

fn shift_left(dst: MutSpan[float], src: Span[float]):
    @parallel(chunk=4096)
    for i in 0..dst.len() - 1:
        dst[i] = src[i + 1]                     # [PAR-2]: dst is accessed only through `i`: accepted

fn caller():
    xs: Array[float] = [1.0] * 1_000_000
    shift_left(xs.as_mut_span(), xs.as_span())  # today E3021 ([BRW-11]); D3: an analysis fact only
    # chunk c reads xs[4096c + 4096], which chunk c + 1 writes: a race at every chunk boundary
```

Why the rules let it through: D3's thread rule (lines 286-291) restricts what a task may do, not what
the parent does inside the scope, and nothing replaces `[BRW-11]` for spans handed to a parallel loop.
Pins are "single-thread facts" with plain counters (line 281), so they do not help.

Fix: keep aliasing-xor-mutation for every borrow a scoped task or a `@parallel` loop takes (the
parent may not write it until the join), a compile-time rule with no run-time cost; for `@parallel`,
alternatively an overlap test of the spans at loop entry (2 compares, as `[DSJ-1]` does) that falls
back to a serial loop. Cost: a refusal the user meets, which the design counts as removed for A15's
scoped case.

### 1.6 Hot reload widens an effect summary its callers relied on (major)

The rules: pins and watcher checks are elided where a whole-program effect summary proves nothing
changes (lines 157-160, 224-230); summaries "are recorded in the package interface". `[HR-1]` (spec
line 6239) reloads one changed function body without rebuilding its callers. The design discusses hot
reload only for W2's layout (line 493) and `E2050`.

```ember
# Built with the program:
fn log_hit(w: World, e: Enemy):
    w.hits += 1

fn update(w: World):
    for e in w.enemies:            # no pin: log_hit's summary does not change w.enemies
        log_hit(w, e)              # the loop keeps the buffer pointer and length in registers

# Reloaded while running ([HR-1]): the new body adds to the container the caller iterates
fn log_hit(w: World, e: Enemy):
    w.hits += 1
    if w.hits % 100 == 0:
        w.enemies.push(Enemy())    # unpinned: may reallocate; update's loop reads the freed buffer
```

Fix: a reload whose new summary is wider than the old one (or that changes an ownership fact a class
layout depends on) is refused, naming the callers; or reloadable builds compile every call into
reloadable code as "every effect its types allow" (pins and checks everywhere). Cost: refused reloads
(an annoyance) or slower development builds (UNVERIFIED how much).

### 1.7 A settle runs `drop` code that changes the container being settled (major)

The rules: a removal under a pin "is applied later"; at count zero "the container settles"
(lines 166-175); destruction "runs its `drop`" (line 116). The design does not say that the settle
keeps the container pinned while it destroys the removed elements.

```ember
class Obj:
    id: int
    dead: bool = False
    world: World                      # a watcher
    children: Array[int] = []         # ids destroyed together with this object

    fn drop(self):
        for c in self.children:
            self.world.destroy_by_id(c)   # removes from world.objects, the array being settled

class World:
    objects: Array[Obj] = []

    fn destroy_by_id(self, id: int):
        for i, o in enumerate(self.objects):
            if o.id == id:
                self.objects.remove(i)
                return

fn tick(w: World):
    for o in w.objects:               # pinned with a counter (destroy_by_id reaches it through w)
        if o.dead:
            w.destroy_by_id(o.id)     # a tombstone
    # unpin: count 0, settle; destroying a parent runs drop, which removes its children from
    # w.objects while the settle is in the middle of compacting it
```

Why it breaks: inside the settle the count is zero, so the drop's removal is real (or starts a nested
settle) and moves elements under the outer settle's indices: elements skipped, moved twice or
destroyed twice. Fix: the settle holds the pin while it destroys removed elements and repeats until
no tombstone is left. Cost: none on fast paths.

### 1.8 The pin protocol covers append and remove only (major)

The rules: "an add never moves an element ... a removal marks the slot as a tombstone" (lines
165-167); "Replacing a pinned container ... keeps the old buffer alive in the pin's side record"
(lines 179-181); a view into a child container is "protected by pinning that child" (line 356).
`insert(i, v)`, `sort`, `reverse`, `swap`, `drain`, assignment of a whole element, and EnTT's
`sort[T]` (planned in `tasks/95/ecs-entt-design.md` line 42) are not described.

```ember
struct Row:
    label: String
    score: int

class Board:
    rows: Array[Row] = []
    fn reorder(self):
        self.rows.sort(by=fn(r): r.score)     # or: self.rows.insert(0, Row(...))
    fn rename(self, i: int, s: String):
        self.rows[i].label = s                # replaces a child container

fn show(b: Board):
    l3 = b.rows[3].label.as_str()     # pins b.rows and rows[3].label
    l5 = b.rows[5].label.as_str()     # pins b.rows and rows[5].label
    b.reorder()                       # if sort moves rows 3 and 5, their label headers (and pin
                                      # counters) move with them
    println(l3)                       # last use: unpins the header now at rows[3] (the old row 5's)
    b.rename(3, "x")                  # that header's count is now 0: the old buffer is freed
    println(l5)                       # l5 views the freed buffer
```

Why it breaks: the pin counter of a child container lives in its header, inside the parent's
buffer. Any operation that moves or overwrites elements in place (sort, swap, an in-place insert
while there is capacity, assigning `rows[i] = Row(...)`) moves or wipes that header, and the unpin
then hits the wrong counter. For ECS pools the W1 model records tombstones by position (`w1.c`
lines 122-126); a `sort` during a pin would make the settle overwrite live components.

Fix: define each operation under a pin: inserts and permutations are deferred to the settle (a sort
that is not visible until the pin ends: a surprise), and in-place assignment keeps the pin fields of
every pinnable container inside the replaced value. Cost: slow paths for indexing while an insert is
pending; a pin test per nested pinnable field on every assignment and drop of such values (not in the
design's cost table).

### 1.9 Watchers into frozen objects shared across threads (minor, unspecified)

Frozen data "shared by unstructured threads is freed by an atomic count" (line 294); "a watcher
cannot cross a thread" (line 955). Not stated: whether a reference taken *inside* a frozen graph
(`self.cache = shared.child`) and stored by the receiving thread is counted or a watcher. If it is a
watcher, the thread that drops the last count bumps the generation and frees the slot while the
other thread reads the generation: a race on a plain word. Fix: references into frozen data are always
counted (atomic) when stored. Cost: an atomic increment per such store.

---

## 2. Annoyances D3 keeps or adds and does not admit

### 2.1 The owner is the first place an object is stored, so line order decides when objects die (major)

The rule (lines 111-113): the home is "where the object is first stored: a local, then (a name
follows its value) the container element, field ... it is stored into". Section 6 item 1 admits that
objects die with their home; it does not admit that the home depends on statement order.

**(a) The design's own W2, two lines swapped.** Python behaves the same in both orders.

```ember
    for i in 1..N:
        p = nodes[h_rand() as int % i]
        n = Node(value=(i % 1000) as i32, parent=Some(p))
        p.children.push(n)        # now the first store: n's home is p.children
        nodes.push(n)             # a watcher
```

Now `Node.children` is a home that removes, so every watcher is checked and `Node` gains a
generation (the design's own fallback estimate is "up to about +25%" on the walk, line 974,
UNVERIFIED). Worse, pass 0's first re-parent (`i = 100`) runs
`old.children.remove(old.children.index_of(node).unwrap())`: that removes node 100's home, destroying
node 100 and, through its `children` homes, its whole subtree. The static "wait" rule cannot fire
(the compiler cannot know which element `remove(idx)` destroys), so the next line,
`node.parent = Some(nodes[new_parent])`, uses a dead watcher typed `Node` and panics.

**(b) The standard parent-pointer idiom makes the child own the root.**

```ember
class Node:
    parent: Option[Node] = None
    children: Array[Node] = []

fn main():
    root = Node()
    c = Node()
    root.children.push(c)         # c's home: root.children
    c.parent = Some(root)         # root's first store after its local: root's home moves into c.parent
    root.children.clear()         # destroys c, whose parent field is root's home: root is destroyed
    println(len(root.children))   # `root` now names a place inside a destroyed object
```

Without the `clear`, the two are each other's home and leak at the end of `main` (section 6 item 9
admits mutual homes leak, as a rare case; this is the ordinary way to write a parent pointer).

**(c) Tree surgery must be written as Rust's take-then-put.** With children owning nodes,
`old.children.remove(i); new_parent.children.push(node)` destroys the subtree; only
`n = old.children.pop(i); new_parent.children.push(n)` hands the home over. Rust refuses the wrong
order at compile time; D3 runs it and panics (or, without the 1.1 fix, uses freed memory).

Fix: none that keeps both "no annotation" and "no counts". Options: an explicit one-word ownership
mark on the owning field or container (ceremony); refusing a class whose objects are stored in two
kinds of place until one is marked (a compile error); or counting (the cost D3 removes).

### 2.2 Struct or class does change what panics (contradicts A24) (major)

A24 (line 377): "The kind changes only the representation ..., never what compiles or what panics."

```ember
struct Mesh:                  # change to `class Mesh` and the last line panics
    verts: Array[f32]

fn main():
    a: Array[Mesh] = []
    b: Array[Mesh] = []
    m = Mesh(verts=[1.0])
    a.push(m)                 # struct: moves into a.   class: a[0] becomes the home
    b.push(m)                 # struct: b gets a copy.  class: b[0] watches it
    a.clear()                 # struct: b's copy lives. class: the Mesh is destroyed
    println(b[0].verts[0])    # struct: 1.0.            class: panic (b's element is typed Mesh)
```

Python prints `1.0` for both. Sharing one list between two holders also needs a `class` (section 6
item 2), which is A10's wrapper choice back in another form.

### 2.3 "A name keeps the old value" cannot hold across a call (minor)

Rule 2 (lines 93-96): if the place a name refers to is reassigned, "the name keeps the old value (the
compiler moves the old value into the name's own slot ...; this is a static rewrite)".

```ember
struct Weapon:
    name: String

class Player:
    weapon: Weapon
    fn upgrade(self):
        self.weapon = Weapon(name="axe")

fn main():
    p = Player(weapon=Weapon(name="sword"))
    w = p.weapon
    p.weapon = Weapon(name="bow")   # same function: rewritten, w keeps "sword"
    println(w.name)                 # sword (Python: sword)
    w2 = p.weapon
    p.upgrade()                     # the same change through a method: no static rewrite possible
    println(w2.name)                # axe (Python: bow), unless a hidden copy is made before every
                                    # call that may write the aliased place
```

Memory-safe (the place is still valid), but the meaning depends on whether the write is in the same
function. Fix: a COPY before such calls (a hidden clone) or documenting that a name is a place.

### 2.4 `enumerate`'s index under removal (minor)

Line 581-584: "`enumerate` counts the elements it yields, so `i` is the element's current index even
after earlier removals". The two halves disagree after a removal. If `i` is the current logical index,
it no longer lines up with a parallel list; if it is the count of yielded elements, `remove(i)` means
"remove the cursor" only through a compiler pattern-match, and `objects[i]` is a different element.

```ember
scores: Array[int] = [0] * len(enemies)
for i, e in enumerate(enemies):
    if e.hp <= 0:
        enemies.remove(i)         # deferred
    else:
        scores[i] += 1            # after the first removal: the wrong score (current-index reading)
        enemies[i].hp -= 1        # or the wrong enemy, and at the end out of bounds (count reading)
```

Silent wrong results either way. Fix: name the operation (`remove_current()`) and keep `i` Python's.

### 2.5 A stored closure that captures an element alias (major)

```ember
boss = enemies[0]                         # a struct element: an alias into enemies
button.on_click(fn(): boss.hp -= 10)      # the closure is stored in a long-lived button
```

The design's closure rules (lines 209-214) cover variables. `boss` is a place in a container: either
`enemies` stays pinned for the button's life (every later push goes to overflow pages, every removal
stays a tombstone, every index after a removal is a scan: 2.7), or `boss` is copied or moved into a
cell and clicks change the copy, not `enemies[0]` (Python changes the enemy). Unspecified; both
readings surprise.

### 2.6 Closures have no identity (minor)

Callables are values compared by function and state (`[CLO-10]`, spec line 2107). A handler that
unsubscribes by reference (`bus.unsubscribe(h)`, Python's habit) removes the first *equal* handler,
and a handler whose captured counter has changed no longer equals the caller's copy:

```ember
h1 = make_logger("ui")
h2 = make_logger("ui")
bus.subscribe(h1)
bus.subscribe(h2)
bus.unsubscribe(h2)       # removes h1 (equal function and state)
```

The design's W4 avoids this by unregistering through an index (`d.current`). Fix: a class wrapper
(wrapper noise) or identity for stored callables (an allocation each).

### 2.7 Indexing a pinned container with removals pending is a linear scan (major, cost)

Lines 170-171 and section 6 item 4 admit "a slow path ... slow in that one situation"; `w4.c`'s
`hl_at_slow` (lines 56-62) scans from the first slot. The cost is per index, so a pinned loop that
removes and indexes is quadratic:

```ember
for e in enemies:                       # pinned: kill() reaches enemies through world
    if e.hp <= 0:
        world.kill(e)                   # a tombstone
    t = enemies[e.target_index]         # after the first tombstone: a scan of the list, every time
```

And a pin can be long: a stored iterator (`self.cursor = iter(self.items)`), a generator suspended
inside a loop (line 367), or 2.5's closure. While it lasts, every removed element's memory is held.
Fix: O(1) for indexes below the first tombstone and a sorted tombstone list or a Fenwick tree above
it (O(log n)); cost: a few compares on the slow path only.

### 2.8 Distant edits change layouts and checks; whole-program analysis everywhere (major, cost)

W2's zero-check result holds only because no line anywhere in the program pops from `nodes`
(lines 485-495). One `nodes.pop()` in a debug command adds a header to every `Node`, a generation to
every watcher and a check per step of the walk. Section 6 item 5 admits that a body change can add a
pin in a caller; it does not admit that it can change a class's layout. The analyses (whole-program
effect summaries, home inference, closure flow) run on every build; their compile-time cost is not
estimated, and separate compilation and reloadable builds get the pessimistic versions (1.6).

### 2.9 Aliasing makes ordinary library loops pin (minor, cost)

With `[BRW-1]` gone, two parameters of one type may be the same place, so
`fn append_all(dst: Array[T], src: Array[T]): for x in src: dst.push(x)` must pin `src` with a real
counter (the change goes through a possibly aliasing parameter). Today `ref mut dst` and `ref src`
cannot alias and need nothing. Cost: 2 counter updates per such loop and a test per push when full.

### 2.10 The thread refusal comes back (minor)

The fix for 1.5 refuses `xs[0] = 10` inside a `with thread.scope()` while a task reads `xs`: the
borrow error, for threads only. The design lists A15 as kept only where "disconnection cannot be
proven".

### Admitted, but under-weighted

- Loops do not visit appended elements (section 6 item 3). This silently breaks Python's worklist
  idiom, `for n in queue: queue.extend(n.children)`, which visits only the first node.
- A struct or list shared by two holders is silently copied (section 6 item 2); the copy is reported
  only by `ember inspect --checks`.
- Mutual homes leak (section 6 item 9): see 2.1 (b), it is the default for parent pointers.

---

## 3. The C models

All four do the work the workload text specifies, and their checksums agree with the independent
references (`check/*_ref.c`, which I read and which follow the workload text as I read it). The
checks the design says the compiler emits are present: bounds and contiguous-part tests, tombstone
tests, pin and unpin counters, generation compares, the `unwrap` of `parent`, the user's `v >= 0`.

### 3.1 w1.c (ECS)

| shortcut | design says | effect |
|---|---|---|
| One pin counter, one `len`/`cap`, one tombstone list for `dense` and `ents` (`POOL` macro, lines 56-65) | `Pool[T]` has two `Array` fields, each a pinnable header with its own counter (lines 649-652) | per visited entity, one bounds test for the lead pool's two arrays instead of two; per add, one capacity test instead of two; per loop, 2 counter increments instead of 4 (and the same for decrements). Small |
| A pool-level tombstone (`NIL` in `ents`) and a settle that repairs `sparse` (lines 136-161) | "pinned: tombstone; else swap-and-pop" (line 660) inside Ember code; a generic `Array` pin keeps tombstones per array (a side record; `Position` has no spare bit pattern) and cannot repair a third array | the model is the best case of a mechanism the design never defines: Ember code would need an "is pinned" query and an on-settle hook. The cost of the real mechanism is UNVERIFIED |
| No cast check on pool lookup; `pools` a fixed C array with no bounds check (lines 173-185) | `cast self.pools[type_number[T]()] as Pool[T]  # proven by the compiler` (line 667) over `pools: Array[AnyPool]` | negligible time (per lookup, not per entity), but the "proof" that an array element always holds `Pool[T]` is an invariant over array contents no stated analysis gives (UNVERIFIED). If it is wrong, it is a type confusion; keeping the compare costs 1 load and 1 compare per lookup |
| The view loop is a hand-fused macro with hoisted base pointers | `view` is library code returning `View[Q]` (line 673, body not shown) | that the D3 compiler inlines the iterator into this shape is UNVERIFIED |
| No logical length or fast index bound in the pool header | a pinnable `Array` header is 56 bytes with both (lines 189-191) | 1-2 fewer stores per add and remove. Small |

Not a shortcut, but the measurement does not cover it: the overflow-page path never runs in W1. The
largest number of despawns (each with one spawn) in one step is 18,298 and the total is 856,653
(count replay); the Velocity pool's capacity headroom is 524,288 - 500,000 = 24,288 and the Position
pool's 1,048,576 - 1,000,000 = 48,576 (capacities from `T##_add`'s doubling from 4). Also not a
shortcut: with `reuse_ids=false` each pool's `sparse` reaches 1,856,653 slots, a capacity of 2,097,152
(8 MiB) per pool (count replay plus the same doubling), against about 4 MiB if ids were recycled.

### 3.2 w2.c (tree)

Faithful to the design's W2. Only the proven case is modelled. The design names the fallback
(separate compilation, hot reload, or any `pop` of `nodes`: a generation per node and a check per step,
line 974); with no model of it, the measurer gets only the best case. `remove` keeps order (memmove)
where the workload allows any order: pessimistic for D3, not flattering.

### 3.3 w3.c (game objects)

Faithful. One small flattering shortcut: `destroyed` is one buffer reused across frames
(`destroyed.len = 0`, line 136), where the Ember program declares `destroyed: Array[int] = []` inside
the frame loop (design line 556): about 1000 allocations, frees and regrowths skipped. The bounds
check on `objects[h_rand() % len(objects)]` is dropped as "proven" (line 161); the design states no
range analysis that proves it, but it runs once per created object. One generation compare covers
three uses of `t` in Turret's update; there is no call in between, so that is sound.

### 3.4 w4.c (callbacks)

Faithful in the pin, unpin, settle test and index compare. One assumption: the handler's state is
three words inline (`counter`, `m`, `&total`; line 34). The Ember handler also calls `make_handler`,
itself a closure over `total` (design lines 606-614); fitting in three words needs the compiler to
merge that capture into the one `total` pointer, which the design does not state. Without it the state
exceeds `[CLO-10]`'s three words: one heap allocation per handler and one more dependent load per
call (UNVERIFIED cost).

### 3.5 Every model

Integer overflow checks (`[TYP-8]`) and zero-divisor checks (`[TYP-28]`, spec line 978; W4's
`k % len(self.handlers)` has a non-constant divisor) are left out of every model. The design says
every memory model pays them alike, which is true between designs, but against plain C they are real
work in the hot loops: W2's `depth_sum += 1` per walk step, W4's `counter += v * m` and `total += 1`
per call. Measured against C, every design's numbers are flattered by the same amount.

---

## Sources

- The design: `tasks/96/redesign/phase3-design-D3.md` (line numbers as cited).
- The models: `target/redesign/bench/D3/w1.c`, `w2.c`, `w3.c`, `w4.c`; references
  `target/redesign/bench/D3/check/w1_ref.c` to `w4_ref.c`; `target/redesign/bench/harness.h`.
- The spec: `docs/spec-source/Ember_v0.9.10_Hardened_35.md`: `[BRW-1]`-`[BRW-11]` (lines
  2283-2316), `[CLO-10]` (2103), `[THR-5]`, `[THR-11]` (3557-3569), `[PAR-2]` (3583-3595), `[DSJ-1]`
  (3157), `[TYP-28]` (969-980), `[STD-11]` (4295), `[HR-1]` (6239).
- ODR-286: `tasks/95/odr-pending.md` line 31; EnTT mapping: `tasks/95/ecs-entt-design.md` line 42.
- Count replay of W1's despawn schedule: inline Python run for this report (residue counting only).
