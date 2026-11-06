# K06-sharing-a-local (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K06-sharing-a-local

**title:**
What happens to a local made shared after it is made: do its views survive, and may the word be thread_shared?

### members
#### [1]
J5-06

#### [2]
J4-05

**primary:**
J5-06

**why_together:**
Both ask what RC-1's 'a local of this function (which becomes one of the owners)' does. J5-06 B's mechanism (the compiler places the block where the local is declared when it sees the sharing line) removes the cost J4-05 A gives as its whole reason. So one answer fixes the other.

**severity:**
high

**impact_rank:**
6

**notes:**
J5-06 is memory safety (a view of where the local used to be). See the recommendation change for J4-05. J7-05's lead fix is option A's edit and waits on this answer (rerouted). Verdict 23 already lets a local be shared (`other = shares mut items`), so for J4-05 the question is only whether thread_shared follows the other two sharing words.

## questions
### [1]
**id:**
J5-06

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
medium

**confidence:**
0.55

#### rules
##### [1]
RC-1

##### [2]
RC-7

##### [3]
HEAP-3

##### [4]
BCK-4

**reason:**
[RC-1] lets a local be shared and stay one of the owners, and [RC-7] keeps its rights. [HEAP-3] puts the value in a block at the line that shares it. [BCK-4] ends views only at a storage end, a hand-over, a replacement or an end, and sharing is none of these. No rule says whether a view taken before that line survives it (grep 'becomes one of the owners', 'where it is made shared'). Whether `println(g)` compiles is the owner's question.

**title:**
Does a view of a local still work after the local is made shared?

**problem:**
A local struct can be made shared halfway through a function (`shop = Shop(stock=shares mut inv)`), and `inv` stays one of its owners with the rights it had. The specification says the value is put in a block of its own at the line that shares it, which moves it out of the function's frame. It does not say what happens to a view of it taken before that line: in C++ terms, `int& g = inv.gold;` followed by `std::make_shared<Inventory>(std::move(inv))`. So `println(g)` is either refused or reads where `inv` used to be.

**example:**
struct Inventory:
    gold: int
    items: Array[String]

class Shop:
    stock: shares mut Inventory

fn main():
    inv = Inventory(gold=5, items=[String("sword")])   # a local struct, inline in main's frame
    g = ref inv.gold                                    # a view of its gold, where inv is now
    shop = Shop(stock=shares mut inv)                   # inv is made shared: put in a block of its own
    println(g)                                          # does g still see inv's gold?
    inv.gold += 1                                       # inv keeps what it could do
    println(shop.stock.gold)

**spec_today:**
p08-classes.md 106-108 [RC-1]: "`shares x`, `shares mut x` and `thread_shared x` are written before a new value, a local of this function (which becomes one of the owners) or a name that is already such an owner". p09-memory.md 61-62 [HEAP-3]: "any other value is put in a block where it is made shared (`count = shares mut Counter()`, `other = shares items`), one allocation". p08-classes.md 120-121 [RC-7]: "the name it was first shared from, which keeps what it could do". p18-implementation.md 185-186 [BCK-4]: "A place whose storage ends (scope exit, `StorageDead`, its owner handed over, replaced or ended) while a view of it is live is `E3060`". Sharing is none of these.

#### options
##### [1]
**letter:**
A

**summary:**
Sharing a local moves it into its block at that line and ends every view into it.

**code:**
struct Inventory:
    gold: int
    items: Array[String]

class Shop:
    stock: shares mut Inventory

fn main():
    inv = Inventory(gold=5, items=[String("sword")])
    shop = Shop(stock=shares mut inv)        # inv is put in a block of its own here
    g = ref inv.gold                         # the view is taken after the move
    println(g)
    inv.gold += 1
    println(shop.stock.gold)

**outcome:**
In the example, `println(g)` is E3060: "`shares mut inv` (line 11) puts `inv` in a block of its own, and `g` views it where it was (line 10) and is used on line 12; take `g = ref inv.gold` after line 11". This version compiles and prints 5, then 6.

##### [2]
**letter:**
B

**summary:**
Views of a local survive its sharing. Where a view crosses the sharing line, the compiler gives the local its block earlier.

**code:**
struct Inventory:
    gold: int
    items: Array[String]

class Shop:
    stock: shares mut Inventory

fn main():
    inv = Inventory(gold=5, items=[String("sword")])
    g = ref inv.gold
    shop = Shop(stock=shares mut inv)
    println(g)
    inv.gold += 1
    println(shop.stock.gold)

**outcome:**
The example compiles and prints 5, then 6. Where a view of `inv` is used across the line that shares it, `inv` is put in its block where it is declared: the same one allocation, made earlier. Everywhere else the block is made at the sharing line, as it is now.

#### recommendation
**letter:**
B

**reason:**
Sharing does not change which inventory `g` views, so the program means what it says. The compiler does the placement unseen, and the cost is the same single allocation, made earlier only where a view needs it.

**checklist:**
1, 2 (accepts a correct program), 3 (no run-time cost added), 4 (nothing written), 13 (one model: a shared local is one value for its whole life)

### [2]
**id:**
J4-05

**verdict:**
confirmed

#### rules
##### [1]
RC-1

##### [2]
THR-1

##### [3]
OWN-13

##### [4]
OBJ-1

##### [5]
RC-4

**reason:**
A real contradiction. [RC-1] and E2122's row allow `thread_shared` before a local; [OWN-13], [THR-1] and E7017's row refuse it. [RC-1]'s sentence came in with 0.9.15_Hardened_2's write-up of verdict 103, and that verdict speaks only of `shares` from one-owner places. E7017 comes from the redesign's rule R3 (run 4), which is not an owner verdict. So no owner decision covers `thread_shared` of a local.

**route:**
owner_odr

**severity:**
medium

**confidence:**
0.7

**title:**
May a local object be shared between threads after it is made?

**problem:**
A program builds a settings object on the main thread, then shares it with a worker: `shared = thread_shared s`. The rule on owners says all three sharing words may be written before a local of the function. The rule on threads, with its own error, says `thread_shared` goes only before a new value or before one already shared between threads.

**example:**
class Settings:
    port: int = 0
    verbose: bool = false

fn main():
    s = Settings()
    s.port = 8080                       # filled in while `main` is its only owner
    shared = thread_shared s            # share the finished settings between threads: allowed, or refused?
    h = thread.spawn(owns fn() thread_shared shared: println(shared.port))
    h.join()                            # 8080

**spec_today:**
[RC-1] p08 106-108: "`shares x`, `shares mut x` and `thread_shared x` are written before a new value, a local of this function (which becomes one of the owners) or a name that is already such an owner." [THR-1] p11 70-72: "it is written before a new value (`st = thread_shared Stats()`) or on a value already `thread_shared` (`fn() thread_shared st:`), and on a value made otherwise it is `E7017`". [OWN-13] p07 139-142: "`thread_shared x`, one more owner on any thread, written only before a new value (`names = thread_shared Names()`) or a value already `thread_shared` (`E7017` otherwise ...)". E2122's row (p17a 91): "made of a place that has one owner and is not a local of this function". [OBJ-1] p08 63 and 67: "The header is 8 bytes"; "A `thread_shared` value's header is 32 bytes".

#### options
##### [1]
**letter:**
A

**summary:**
Refused, as the rule on threads says: a value shared between threads is made that way.

**code:**
s = thread_shared Settings(port=8080)       # made shared between threads where it is made
h = thread.spawn(owns fn() thread_shared s: println(s.port))
h.join()

**outcome:**
`shared = thread_shared s` is refused (E7017: "`s` is `main`'s own object, which it may change; to share it between threads, create it shared: `s = thread_shared Settings(port=8080)`"). Written as above, it compiles and prints 8080. `shares s` and `shares mut s` of a local stay allowed.

##### [2]
**letter:**
B

**summary:**
Allowed, as `shares s` of a local is: `s` becomes one of the owners.

**code:**
shared = thread_shared s

**outcome:**
Compiles and prints 8080. From that line on, no owner changes the object directly (`s.port = 9` is refused). But every `Settings` object that may be shared this way must carry the 32-byte two-count header from the moment it is made, or be moved to a new block at that line, and no view or link of it may outlive the move.

#### recommendation
**letter:**
A

**reason:**
A value shared between threads keeps its two owner counts in a 32-byte header from the moment it is made; a one-thread object has 8 or 12 bytes. Turning an object that already exists into one would make every object of its class pay the bigger header, or move it and cut its links. That is the owner's own reason for refusing `shares` of a one-owner place: no cost spread to values that never need it. `shares` of a local keeps working, because every header already holds a count.

**checklist:**
3 (no object grows), 5 (one rule for struct and class values), 7, 9 (the error's line compiles), 13 (no hidden bigger header and no hidden move)

## group
**gid:**
P03

**principle:**
A value is made, once, in storage that fits everything the program later does with it (a counted block if any line shares it, the thread_shared header if any line shares it between threads, writable memory if links count it), so no later line moves it or changes its header while a view, a link or a C pointer points at it, and every sharing word may follow a local of the function alike.

**shared_cause:**
HEAP-3 puts a value in its counted block at the line that shares it, and CT-5 puts a static in read-only memory. Both decide storage from the line at hand, not from every use the program makes of the value. So a view taken before the sharing line points where the local used to be (J5-06). thread_shared of a local is refused (THR-1, OWN-13, E7017 against RC-1) only for the cost of a later header change that a whole-function placement avoids (J4-05, J7-05). And a weakref count must be written into read-only memory (J6-06).

### members
#### [1]
J5-06

#### [2]
J4-05

#### [3]
J7-05

#### [4]
J6-06

### clusters
#### [1]
K06-sharing-a-local

#### [2]
K13-weakref-to-static

### answered
### verdicts_involved
#### [1]
23: the first owner keeps what it could do after sharing; a local may be shared

#### [2]
81: adding to a list does not end a link to an object in its own block

#### [3]
101: the 32-byte thread_shared header

#### [4]
103: shares is not made from a one-owner place (its layout reason)

#### [5]
114: fix 3, an object C keeps never moves

#### [6]
165: the ThreadShared check stays at the thread_shared line

**other_places:**
weakref of a local (WK-11 lets any name make a link), into_raw of a local, a local's address given to C under retained (verdict 114 fix 3), shares of a static's object, an inline struct in a list that a link points into (verdict 81), relocation at a thread hand-over (verdict 114), relocation at a reload (P09), a local captured by a function that outlives its maker, and generator frames holding locals that views point into.

**severity:**
high

**rank:**
3

### depends_on
**notes:**
K06 asks J5-06 and J4-05 as one question. J7-05's lead-fix text is J4-05 option A's edit and is written only if the owner picks A (phase 3 reroute). Phase 3 changes J4-05's recommendation to B: the compiler sees the sharing line in the same function and places the block where the local is made, so A's cost reason does not hold. J6-06 A is the same mechanism for statics. E7017 rests on the redesign's rule R3, not a verdict; tell the owner what it rests on.

## conflicts_naming_a_member
### [1]
#### ids
##### [1]
J4-05

##### [2]
J5-06

**conflict:**
J4-05 A's whole reason is cost: 'Turning an object that already exists into one would make every object of its class pay the bigger header, or move it and cut its links'. J5-06 B, recommended in the same set, rests on the opposite fact: 'Where a view of `inv` is used across the line that shares it, `inv` is put in its block where it is declared: the same one allocation, made earlier'. A local shared later in the same function is visible to the compiler, so only that local pays.

**resolution:**
Decide K06 as one question. If J5-06 B, J4-05 B's outcome must say that the compiler gives `s` its 32-byte block where it is made. No other Settings object pays, and no link is cut. `s` keeps only what a thread_shared owner may do from that line (E7014 for a direct change). See the recommendation change for J4-05. If J5-06 A, J4-05 A's reason stands.

### [2]
#### ids
##### [1]
J4-05

##### [2]
J7-05

**conflict:**
The same contradiction (RC-1 allows `thread_shared x` before a local, while THR-1, OWN-13 and E7017 refuse it) is an owner question in J4-05 and a lead fix in J7-05. J4-05 found that 'E7017 comes from the redesign's rule R3 (run 4), which is not an owner verdict', and that verdict 103 'speaks only of `shares`'.

**resolution:**
Owner question (K06). J7-05's text is option A's edit and is written only if the owner picks A. See the reroutes.

## verdicts_involved
**23:**
JOB.md 212-215: 23. **The first owner keeps what it could do after sharing.** After `other = shares mut items` (or
    `shares items`), `items` may still change the list; both owners may. One thread runs one line at a
    time, the list rule still refuses a change during a loop through either name, and a `shares` value
    is never used from two threads (that is `thread_shared`, through a lock). Owner: "yes a".

**81:**
JOB.md 606-613: 81. **Adding to a list does not end a link to an object in its own block** (open question 18: R8 (b)'s
    "any reshape ends a view into what the list owns" against AN-52). With `players: Array[Player]` (class
    objects, each in its own block) and `w = Watcher(target=sharedref mut players[0])`,
    `players.push(Player())` moves at most the list's addresses, not the players, so `w.target.hp -= 1`
    after it is allowed. A link to a value stored inside the list's own block (an inline struct) is still
    ended by a push. Removing, clearing or replacing still ends links to what may be destroyed. The owner
    also noted that nothing on `players` says its players can be shared: a `sharedref mut` link owns
    nothing, so the word sits on the line that makes it, as `ref mut enemies[0]` does. Owner: "a".

**101:**
JOB.md 786-812: 101. **A `thread_shared` object's owners are counted in two numbers** (owner, 2026-10-27, after the
    measuring phase; `tasks/96/redesign/measure-0.9.15/brc/REPORT.md`).
    - **The two numbers.**
      - The thread that made the object counts its owners in one number, A, which only it ever reads or
        writes, with normal fast instructions.
      - Every other thread uses the second number, B, with locked instructions.
      - The object has A + B owners.
      - Only the making thread can read both, so when another thread takes the last owner it hands the
        object to the making thread. That thread deletes it at its next check point (placed by the
        compiler; always before the thread ends), or the other thread deletes it at once if the making
        thread has ended.
    - **The fixes.**
      - The maker's id is read from the object's memory chunk (fix 1).
      - A hand-over the compiler sees moves A into B at that moment (fix 2), so the delay remains only for
        an object that travels inside another object.
      - Rule Q: the drop that hands the object over marks it in the same atomic step, and only the making
        thread's check then deletes it.
      - A merged object's −1 is one locked add (the shortcut).
    - **Measured against one locked count**, 550 stress runs clean:
      - a thread's own objects, 25% faster;
      - the camera program, 89 ms against 544 ms;
      - several threads reading one thread's objects, up to 28% faster;
      - objects passed back and forth all the time, 9% to 13% slower;
      - each object 16 bytes bigger.
    - Owner, asked "Do you want the two-number design in the spec?" (with fix 2): "yes". The owner first
      kept the view's count on every read ("why the fuck would remove the counter for a read
      operation?"), and asked for a fix for the language, not for one benchmark.

**103:**
JOB.md 819-831: 103. **`shares` cannot be made from a place that has one owner** (owner, 2026-10-27, gap map Q-B, (a)).
    - A list element (`shares mut players[1]` of an `Array[Player]`), a field (`shares this.items`) or a
      plain parameter (`fn keep(p: Player): LIST.push(shares p)`) is refused when compiling, with a code
      of its own (a sibling of E2119).
    - The help names the rewrite: the holder shares its values from the start, as the help of R7's
      `reject_removing_a_player_a_kept_link_points_at.em` writes it
      (`players = [shares mut Player(), shares mut Player()]`, then `shares mut players[1]`).
    - Why: a value in a one-owner place has no other owner, so its end frees at once with no check;
      only values written `shares` pay for counting. Allowing it for class objects would make every end
      of a one-owner place of that class test the count; allowing it for inline values would change the
      holder's layout everywhere in the program.
    - Owner, after the shorter explanation (one program, the table of who owns the second player on
      each line): "yes a".

**114:**
JOB.md 915-943: 114. **An object handed to another thread while a `weakref` on this thread points at it goes, and the
    links here then say "gone"** (owner, 2026-10-28, gap map Q-E, "aight approved with these fixes").
    Rust's run-time way (`Rc::try_unwrap`), which the lead checked with `rustc` 1.98.1: Rust's
    compile-time way refuses even `drop(w)` then `thread::spawn(move || names.items.len())` (E0277,
    "`Rc<Names>` cannot be sent between threads safely"), so it refuses fine programs as (a) did; with
    `Rc::try_unwrap(names)` the thread gets the object and `w.upgrade()` is `None` ("0 true").
    - At the hand-over the program reads the object's weak count, one compare. No link: the object
      goes as it is. A link: its contents move to a new place for the other thread, the old place is
      marked ended (its `drop` does not run; the new one's runs once, on the other thread) and stays
      here until its last link ends, so every `weakref` here says "gone". Nothing is refused and
      nothing stops. `names = Names()`, `w = weakref names`,
      `h = thread.spawn(fn() owns names: names.items.len())`, then `if w exists as p:` is skipped and
      `println(h.join())` prints 0 (the design test that refused this becomes a run-pass test).
    - Fix 1, objects inside it: the check also covers every object the handed object owns whose class
      a `weakref` is made to anywhere in the program (the compiler knows those classes);
      `cam.target = weakref level.enemies[0]` then `thread.spawn(fn() owns level: ...)` moves enemy 0
      and `cam.target` says "gone". One compare per such object (10,000 enemies, 10,000 compares).
      Today's text of the rule misses this case too.
    - Fix 2, a link open at that moment: a hand-over to another thread joins the lines after which a
      view opened from a link to an object of that type is not used (`if ROSTER[0] exists as p:`, the
      hand-over of a `Names`, then `p.items.len()` is refused when compiling); nothing runs for it.
    - Fix 3, an address C keeps: an object given to C under a `retained` contract never moves; for it
      a link at the hand-over stops the program (gap map Q-E's (b)). `unsafe` code gets the written
      obligation that an object handed to another thread may move.
    - Checked and fine: the other thread never touches this thread's counts (the old place stays here
      with the links; the new one starts with none); an object holding a `weakref` to a one-thread
      object cannot go to another thread at all; a lock in the object is one futex word, movable, and
      cannot be held across the hand-over.
    - Goes into the next hardening (`[THR-2]`, `[RC-5]`, `[WK-12]`, `[FFI-35a]`, `[UNS-4]`) with M9.

**165:**
JOB.md 1253-1253: 165. **`thread_shared` is refused where the value is made, when something in it is not safe for several threads to read at once** (`ThreadShared`): `stats = thread_shared FrameStats()` with `draws: RwLock[Cell[int]]` is `E7015` at that line, naming the field and the fix (`draws: Atomic[int]`). Shown as (a) refuse it where it is made, (b) refuse it only where it reaches another thread, recommendation (a). Owner (2026-11-02, gap-sweep group `K02-thread-shared-not-threadshared`): "a".

## allowed_rules
### [1]
BCK-4

### [2]
HEAP-3

### [3]
OBJ-1

### [4]
OWN-13

### [5]
RC-1

### [6]
RC-4

### [7]
RC-7

### [8]
THR-1

## rule_text
**BCK-4:**
p18-implementation.md lines 185-188
* `[BCK-4]` *(new in 0.9.9)* *(changed in 0.9.15)* **Storage end.** A place whose storage ends (scope
  exit, `StorageDead`, its owner handed over, replaced or ended) while a view of it is live is `E3060`
  at the view's later use; a view, or a value holding views, kept or returned where it would outlive
  what it points into, a value made in a memory manager's instance included, is `E3063`.

**HEAP-3:**
p09-memory.md lines 58-63
* `[HEAP-3]` *(changed in 0.9.15)* A value with several owners (`shares`, `shares mut`,
  `thread_shared`) lives in one heap block that holds its owner count, and, where `weakref` links may
  reach it, its weak count (`[OBJ-1]`). A class object already has its block, so another owner only
  adds one to its count; any other value is put in a block where it is made shared
  (`count = shares mut Counter()`, `other = shares items`), one allocation. Each added owner is first
  compared with the count's maximum (`[RC-1]`).

**OBJ-1:**
p08-classes.md lines 63-73
* `[OBJ-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* The header is 8 bytes: a 32-bit count and a
  32-bit type index. A value a `weakref` link may reach has a third 32-bit number there, its weak
  count, making 12 bytes: every object of a class family (the classes that share one root base class)
  of which some `weakref` link in the program may name an object, and every shared value of a type some
  `weakref` link may name. A `thread_shared` value's header is 32 bytes (verdict 101): its making
  thread's count of its owners, the type index, the weak count, the other threads' count with its
  marks, and its place in the making thread's queue (`[RC-4]`). Which thread made a value is read from
  the header of the memory chunk its block came from, written once when the chunk is made, never from
  the value. The header is part of the runtime ABI (`[VER-4]`). Foreign code never reads it; it calls
  `ember_rt`. An object carries nothing else for the memory model: no access words, and no flags but a
  `thread_shared` value's three marks.

**OWN-13:**
p07-ownership.md lines 135-156
* `[OWN-13]` *(new in 0.9.15)* **Words before a value.** Before a value, a word acts on the object the
  value names (for a link or an owner, on its target): `copy of x` (`[OWN-8]`); `owns x`
  (`[OWN-9]`); `shares x`, one more owner, on this thread, that reads it, and `shares mut x`, one more
  owner that may also change it, each written only before a new value, a local or an owner (`[RC-1]`,
  `E2122`); `thread_shared x`, one more owner on any thread, written only
  before a new value (`names = thread_shared Names()`) or a value already `thread_shared` (`E7017`
  otherwise: "`names` is `share_and_keep`'s own object, which it may change; to share it between
  threads, create it shared: `names = thread_shared Names()`"); `weakref x`, a link that does not keep it
  alive (`[WK-11]`); `sharedref mut x`, a changeable link other holders may share (`[LT-46]`); and
  `ref x` and `ref mut x`, views (`[BRW-12]`). An owner made from another may do no more than it:
  `shares mut x` needs `x` to be a name that may change the value (`E3023`: "`g2 = shares mut game`
  would change the `Game` through `game`, which only reads it (`game = shares Game()`, line 1), and an
  owner made from it only reads too; write `g2 = shares game`, or, for two owners that both change the
  game, write `game = shares mut Game()` instead of line 1"). A plain parameter of a `shares mut` type
  only reads the caller's value (`[FN-1]`), so an owner made from it only reads too; a function that
  hands on an owner that may change takes one: `fn setup(owns game: shares mut Game)`, called
  `setup(shares mut game)`. In a lambda's capture list each word means
  what it means before a value (`[CLO-2]`). In a list of names declared without types of their own (a
  capture list, a loop's names, a pattern's names, a lambda's parameters written without types) each
  word belongs to the one name after it, and a name with no word may not follow a name with a word, so
  that no word can be read as covering the next name (`E0116`: "`ref mut` here belongs to `pos` only;
  say what `vel` is: `for ref mut pos, ref vel in q:`, or `ref mut vel` to change it too").

**RC-1:**
p08-classes.md lines 101-118
* `[RC-1]` *(changed in 0.9.15)* **Several owners.** `shares x` makes one more owner of what `x`
  names, on this thread, that reads it; `shares mut x` one more that may also change it; and
  `thread_shared x` one more on any thread (`[OWN-13]`). A value made shared lives in a block of its own
  with the header of `[OBJ-1]` (a class object already does), and its owners are counted: making an
  owner adds one to the count, ending one subtracts one, and the subtraction that reaches zero ends the
  value. A value with one owner is not counted: its owner ends it. **Only a whole value is shared**
  (verdict 103): `shares x`, `shares mut x` and `thread_shared x` are written before a new value, a
  local of this function (which becomes one of the owners) or a name that is already such an owner. A
  place that has one owner and is not a local of this function, a list element (`shares mut players[1]`
  of an `Array[Player]`), a field (`shares this.items`) or a plain parameter (`fn keep(p: Player):
  LIST.push(shares p)`), is `E2122`: "`players[1]` has one owner, the list `players`, and no second
  owner can be made of one element; to let other names own the players, make the list share them from
  the start: write `players = [shares mut Player(), shares mut Player()]` in place of line 15". So a
  value in a one-owner place is ended by that place at once, with no count to test. **A count is checked
  before it rolls over:** before each +1 of an owner count it is compared with its maximum, and at the maximum the
  program stops, in every profile: "too many owners of a `Player` (4,294,967,295)" (`[RT-7]`), as
  Rust's `Rc` and `Arc` stop (a `thread_shared` count stops at the lower limit of `[RC-4]`). That is one
  compare per added owner and no extra memory.

**RC-4:**
p08-classes.md lines 155-190
* `[RC-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* The counts of a value that is not
  `thread_shared` are changed with plain adds and subtracts, and never cross threads (`E7015`).
  **A `thread_shared` value's owners are counted in two numbers** (verdict 101; biased counting, Choi,
  Shull and Torrellas, 2018):
  1. **Home and away.** The thread that made the value counts the owners it holds in the value's home
     count, with plain adds and subtracts; every other thread counts its owners in the away count, with
     atomic ones (`[RT-8]`). The value lives while the two add up to more than zero. Only the making
     thread reads both.
  2. **The hand-over to the making thread.** When another thread ends an owner the making thread
     counted, the away count goes below zero: that thread puts the value in the making thread's queue,
     marking it queued in the same atomic step as its subtract, and only the making thread then ends
     it. The making thread empties its queue at each of its check points: it adds each value's home
     count into its away count, and ends the value when the sum is zero. A queued value is ended by
     nothing else.
  3. **Check points.** The compiler places them: at every loop's back edge, before every call that may
     block the thread (waiting for a lock, `join`, a channel's receive, a sleep) and when the thread
     ends. A value whose making thread has ended is ended at once by the thread that ends its last
     owner.
  4. **Handed over at once.** A thread that stores an owner of a `thread_shared` value where other
     threads can reach it (into a lock-guarded `thread_shared` container, through a channel, into the
     capture of a thread it spawns or of a callback C keeps; every such place is known when compiling)
     first adds its own count of the value into the away count and marks the value handed over; from
     then on every thread, the making thread included, counts in the away count, and a later end on
     another thread never goes through the queue.
  5. **So a value's end is delayed only** when its last owner ends on a thread that did not make it
     and the value was not handed over at a place of point 4 (it travelled inside another object): it
     is ended at its making thread's next check point. Until then it is intact, and a `weakref` link
     opened on another thread may still find it (`[WK-3]`).

  The weak count of a `thread_shared` value is atomic. The away count, and the weak count, are raised
  with one atomic add and their limit is checked on the value the add returns: when the count before
  the add was 2,147,483,647 or more, the program stops ("too many owners of a `Player`
  (2,147,483,647)"; "too many `weakref` links to a `Player` (2,147,483,647)"). That limit is half the
  count's range, so threads racing past it each stop before the count can roll over, whatever their
  number. The home count, and a count that is not `thread_shared`, keep the exact compare of `[RC-1]`
  and `[WK-11]`.

**RC-7:**
p08-classes.md lines 119-133
* `[RC-7]` *(new in 0.9.15)* **Changing a shared value.** A value made shared is changed by its owners
  that may change it: an owner made with `shares mut`, and the name it was first shared from, which
  keeps what it could do. After `other = shares mut items` (or `other = shares items`), `items` may
  still change the list if it could before, and so may `other`; a `shares` owner only reads it
  (`E3023`, `[OWN-13]`). Any `ref mut` name of it and any holder of a `sharedref mut` link to it may
  change it too. That is safe as it stands, with nothing checked while the program runs: one thread
  runs one line at a time; a list does not change while a loop walks it, through whichever of its names
  (`[BRW-18]`); a view of it is checked against every name that may reach it (`[BRW-17]`); and a
  `shares` value never reaches another thread (`E7015`), which is what `thread_shared`, changed through a
  lock, is for. A `thread_shared` value is never changed directly by any owner, only through `Atomic`,
  `Mutex`, `RwLock`, a channel, or a `guarded by` field under its lock (`E7014`: "`names` is shared
  between threads (line 5), so no owner changes it directly, and line 6 pushes to its list; fill the
  list where the object is made: write `names = thread_shared Names(items=[copy of s])` instead of lines 5
  and 6"). How scoped tasks and threads may reach a shared value or a `sharedref mut` link is §XI.3's.
  An object or a shared value carries no access words and no flags for any of this.

**THR-1:**
p11-concurrency.md lines 69-98
* `[THR-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* **A value several threads own is made
  `thread_shared`.** `thread_shared x` makes one more owner of `x`, which may be on any thread; it is
  written before a new value (`st = thread_shared Stats()`) or on a value already `thread_shared`
  (`fn() thread_shared st:`), and on a value made otherwise it is `E7017` ("`names` is
  `share_and_keep`'s own object, which it may change; to share it between threads, create it shared:
  `names = thread_shared Names()`"). A `thread_shared` value lives until its last owner ends. Its
  owners are counted in two numbers, its making thread's and the other threads' (atomic), and its weak
  count is atomic (`[RC-4]`); each is compared with its limit before it grows (Part VIII). Three rules keep it safe:
  * **no owner changes a `thread_shared` value directly** (`E7014`: "`names` is shared between threads
    (line 5), so no owner changes it directly, and line 6 pushes to its list; fill the list where the
    object is made: write `names = thread_shared Names(items=[copy of s])` instead of lines 5 and 6");
    its state changes only through an `Atomic`, a `Mutex`, an `RwLock`, a channel end, or a field
    `guarded by` a lock, under that lock (`[THR-18]`);
  * everything reachable from it is `Send` (`[THR-8]`): a one-thread owner, a `weakref` link to a value
    that is not `thread_shared`, or a plain `fn()` value reachable from it is `E7015` at the line that
    makes it `thread_shared` ("`Bus.handlers` holds plain `fn()` values, and a `thread_shared Bus` may
    run them on any thread; write `handlers: Array[thread_shared fn()] = []` in `Bus`");
  * **`thread_shared fn()`** is a function any thread may run. It holds only `thread_shared` owners,
    values handed over to it and copies, never a one-thread owner or a `weakref` link to one (`E7015`:
    "`on_event` keeps functions any thread may run (`thread_shared fn()`), and this one holds `count`, a
    `shares mut` owner, which stays on one thread; share the counter between threads: write
    `count = thread_shared Counter()` in place of line 12, give `Counter` the field
    `n: Atomic[int] = Atomic(0)` (with `from std.sync import Atomic` at the top), and pass
    `fn() thread_shared count: count.n.fetch_add(1)`"). A list, field or parameter whose functions may
    run on any thread says so in its type: `handlers: Array[thread_shared fn()] guarded by lock = []`.

  As a parameter's type, `log: thread_shared Log` reads the caller's value with no count, and
  `owns log: thread_shared Log` takes one owner. A class needs no mark to be shared between threads:
  `@sync` is not Ember (`E0104`: "`@sync` is gone: a class needs no mark to be shared between threads,
  and `Stats`'s fields are `Atomic`, which threads may change as they are; remove `@sync` (line 1)").

## rules_not_found
