# K20-anyerror-send (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K20-anyerror-send

**title:**
Can a worker thread return Result[int], whose error is AnyError?

### members
#### [1]
J6-16

**primary:**
J6-16

**why_together:**
Single question.

**severity:**
high

**impact_rank:**
20

**notes:**
An ordinary program is refused today without saying so. A refuses `?` of a one-thread error into AnyError even in programs with one thread. Say that cost in the option.

## questions
### [1]
**id:**
J6-16

**verdict:**
confirmed

**reason:**
thread.spawn needs its result `R` to be Send (THR-10, p11 175-176). AnyError holds an `owns dyn Error` (ERR-8, p13 93-94; std/src/core.em:92-99). `dyn I` is Send only when `I` requires it (THR-8, p11 43ff, spec 5376), and Error requires only Debug and Display (ERR-3, p13 87-88; core.em:80). So by the rules AnyError is not Send and a worker returning `Result[int]` is refused, which the text never says; making it Send instead would let `From[E]` for every `E: needs Error` put a one-thread owner into a value sent to another thread. No verdict.

**route:**
owner_odr

**severity:**
high

**confidence:**
0.7

**title:**
Can a worker thread return Result[int], whose error is AnyError?

**problem:**
A program loads a file on a worker thread and returns `Result[int]`, whose error type is AnyError. A thread's result must be safe to send to another thread, and by the specification's rules AnyError is not, because it can hold any error, including one that holds a one-thread owner. The specification never says so, and this ordinary program is refused.

**example:**
import std.fs
import std.thread

fn load(path: StrView) -> Result[int]:            # fails with any error: AnyError
    text = fs.read_to_string(path)?               # io.Error -> AnyError
    return Ok(text.len())

fn main() -> Result[void]:
    h = thread.spawn(owns fn() => load("level1.cfg"))   # the result, Result[int, AnyError], must be Send
    n = h.join()?
    println(n)
    return Ok(())

**spec_today:**
p11 175-176 (THR-10): "`[R]` above `pub fn spawn(owns f: thread_shared fn() -> R) -> JoinHandle[R]:` ... and `R` MUST be `Send` (`E7015`, `E7008`)." Spec 5376 (THR-8): "`dyn I` when `I` requires it (`interface I: needs Send:`)". p13 87-88 (ERR-3): "`Error` is the interface of error types: `Display + Debug` with `fn source() -> Option[ref dyn Error]`". p13 93-95 (ERR-8): "a `dyn Error` held alone (`owns dyn Error`) ... It implements `From[E]` for every `E: needs Error`".

#### options
##### [1]
**letter:**
A

**summary:**
AnyError can go to another thread; it takes only errors that can

**code:**
h = thread.spawn(owns fn() => load("level1.cfg"))

**outcome:**
Compiles and prints the file's length. AnyError is Send and ThreadShared, and `?` converts only errors that are too; an error holding a one-thread owner (a `shares` field) is refused at the `?` with E2040 naming the field, help: hold a copy, or name the error type.

##### [2]
**letter:**
B

**summary:**
AnyError stays on its thread

**code:**
fn load(path: StrView) -> Result[int, io.Error]:

**outcome:**
The `Result[int]` program is refused at the spawn line (E7015: the result holds an `AnyError`, which may hold a one-thread owner); a worker names a concrete error type, as here, and converts it after `join`.

#### recommendation
**letter:**
A

**reason:**
Returning errors from worker threads is ordinary code; A makes it compile at no cost (as Rust's anyhow requires Send and Sync), and the rare error holding a one-thread owner is refused at the `?` with a line to write, never a race.

**checklist:**
1, 2, 3, 7, 8, 9

## group
**gid:**
P01

**principle:**
State made for one thread (a count without atomics, a link others change, a Cell, module state confined to one thread) is never reached from another thread by any route: each type's thread markers follow from everything it holds or can reach, a type-erased value by what it may hold, and every route to another thread (a channel, a spawn, a scoped task's work, a thread_shared owner, an export's calling thread) needs them.

**shared_cause:**
The specification lists the thread markers and the routes one case at a time instead of deriving them from one property. THR-8's Send list and THR-9's ThreadShared list leave out sharedref mut. THR-2's routes leave out scoped tasks. THR-11 limits a task's making of shares owners but says nothing of ending them. THR-1 checks only Send and direct changes at the thread_shared line. An erased error type (AnyError) has no stated markers. FFI-33/33c confine module statics differently under `main` and `creator` with no reason given. So each new holder, route or confinement falls between the lists.

### members
#### [1]
J2-06

#### [2]
J7-03

#### [3]
J7-04

#### [4]
J6-16

#### [5]
J8-10

### clusters
#### [1]
K01-sharedref-send

#### [2]
K02-thread-shared-not-threadshared

#### [3]
K03-task-ends-shares-owners

#### [4]
K20-anyerror-send

#### [5]
K57-export-threads-main-creator

### answered
#### [1]
J2-06: verdict 164 (a sharedref mut link never goes to another thread; a type holding one is neither Send nor ThreadShared; scoped tasks reach it through the caller's variables)

#### [2]
J7-03: verdict 165 (thread_shared is refused where the value is made when something in it is not ThreadShared, E7015)

#### [3]
J7-04: verdict 166 (a scoped task neither makes nor ends a shares owner, nor ends an object that weakref links on this thread point at)

### verdicts_involved
#### [1]
3: sharedref mut changed on two threads only with a lock

#### [2]
23: a shares value is never used from two threads

#### [3]
61: thread_shared fn() says any thread may run it

#### [4]
77: a reduction through two sharedref mut links from scoped tasks

#### [5]
93: a task may take copy of t or ref t

#### [6]
94: thread.spawn takes a thread_shared fn()

#### [7]
101: two counts, the making thread deletes

#### [8]
104: after join the parent may use what the task used

#### [9]
114: a handed-over object's links here say gone

#### [10]
164

#### [11]
165

#### [12]
166

**other_places:**
Every type-erased holder: dyn I without needs Send, owns dyn Error, function values (fn() against thread_shared fn()), iterators, generators and closures that capture shares owners or Cells. Channel ends (Sender/Receiver are Send only for a Send element: lead fix J7-07). JoinHandle results, jobs and job results, statics (STA-1 needs ThreadShared), the weak count in a one-thread object's header (written when a link is made or the object ends), callbacks C runs on threads C owns (FFI-22, HR-3a), export wrappers under each threads word, a one-thread memory manager reached from another thread (P02, J5-05), the queued drop of a thread_shared value (P05), hot-reload work done on another thread, and the vouching path for types the compiler cannot see into (P04, J7-06/J2-12).

**severity:**
high

**rank:**
1

### depends_on
**notes:**
Three of the five members are answered, and they fix the rule's shape: markers derived from contents (164), ThreadShared checked at the making line (165), and an operation on a one-thread count from a task counts as a route (166). Open: J6-16 (A, AnyError Send and ThreadShared with a refusal at `?` of an error that is not, is the rule's 'an erased type by what it may hold'; state its cost, a one-thread error refused at `?` even in a program with one thread) and J8-10 (which thread a module's static mut belongs to, one rule for main and creator). After verdict 165 a C wrapper reaches thread_shared or static only through P04's ThreadShared vouch (J7-06): present them together, as phase 3 notes for K02.

## conflicts_naming_a_member
## verdicts_involved
**3:**
JOB.md 1330-1331: 3. **If it cannot be saved, discard it**, and say why: the measurement or the counterexample that no
   fix removes.

**23:**
JOB.md 212-215: 23. **The first owner keeps what it could do after sharing.** After `other = shares mut items` (or
    `shares items`), `items` may still change the list; both owners may. One thread runs one line at a
    time, the list rule still refuses a change during a loop through either name, and a `shares` value
    is never used from two threads (that is `thread_shared`, through a lock). Owner: "yes a".

**61:**
JOB.md 485-495: 61. **A function type says whether any thread may run its functions: `thread_shared fn()`** (safety
    validation (b), hole 6a: C1's G26). A function that holds a one-thread value (`with shares mut count`)
    stored in an object threads share (`bus = thread_shared Bus()`) and run by another thread
    (`bus.fire()`) would let two threads change `count` at once. A list or parameter whose functions may
    run on any thread says so in its type: `handlers: Array[thread_shared fn() changes Counter] guarded by
    lock = []`, `fn on_event(owns f: thread_shared fn() changes Counter) changes handlers:`; such a
    function may hold `thread_shared` values or copies, never a `shares` or `shares mut` value, and a
    `thread_shared` object's fields that hold functions use such types. So `bus.on_event(fn() with shares
    mut count changes Counter: count.n += 1)` is refused (make the counter shareable:
    `count = thread_shared Counter()` with `n: Atomic[int]`). As Rust's `Send`. Owner: "b", over (a), the
    compiler finding it from the whole program with no word in `Bus`.

**77:**
JOB.md 574-586: 77. **Adding to an object's field from several scoped tasks is a reduction too** (open question 14: the
    reduction rule's `total += e` against E7007 naming `stats.total`). `scope.spawn(fn() with copy of x
    changes stats: stats.total += x)` gives each task a private sum, added together at the end of the
    scope (always the right total); nothing reads `stats.total` before then. A task that reads the
    current value (`if stats.total < 10: stats.total += x`), or a `thread.spawn` thread, still needs an
    `Atomic` or a lock (E7007). Owner: "okay so for reduction it should be fine" (after Rust's way, an
    atomic, a lock or Rayon's explicit `sum`, was shown). A whole-number total reached through two
    `sharedref mut` links to one object is one too (recheck U10): with `a` and `b` two `Watcher`s linking
    one `p = Player(hp=10)`, `scope.spawn(fn(): a.target.hp -= 1)` and `scope.spawn(fn(): b.target.hp -=
    2)` leave 7 every time; `a.target.hp = 5` from a task still needs a lock (E7007). Owner, asked on
    2026-10-27: "yes", over refusing it. It was decided on whole numbers
    (`xs = [1, 2, 3]`, "always 6"); a float total a task adds to more than once (recheck U7) was a
    separate question: verdict 92.

**93:**
JOB.md 715-724: 93. **A task may take its own copy of the loop's text or look at it where it is: `copy of t` or `ref t`**
    (area E's item 1, AN-71; verdict 65 gave only the copy). In `for t in texts:`, `scope.spawn(fn() copy
    of t: total += t.split_whitespace().count())` gives each task its own copy (one allocation and copy
    per text), and `scope.spawn(fn() ref t: ...)` lets the task look at the text `t` is on in that round,
    with no copy (as Rust's scoped threads borrow `&t`): safe because `texts` cannot change until every
    task has ended at the end of the `with scope` block. `ref t` is refused only where it would clash:
    the task could still look after the text is changed or gone (a `thread.spawn` thread may outlive the
    loop), the message giving `copy of t`. Owner: "both ways should be permissible right? like if you
    want to pass a copy that's fine and if you want it to look at the same variable that is also fine as
    long as there is no clash".

**94:**
JOB.md 725-730: 94. **`thread.spawn` takes a `thread_shared fn()`** (area E's item 4; verdict 61 applied to the library's
    own line). `pub fn spawn[R](owns f: thread_shared fn() -> R) -> JoinHandle[R]:`; calls are written as
    before (`h = thread.spawn(fn() copy of r: worker(r))`), and a function holding a one-thread owner is
    refused by verdict 61's general rule (E7015) instead of a rule of `spawn`'s own. `scope.spawn` keeps
    a plain `fn()`: its tasks may use the caller's own variables (`total`, `ref t`), since they all end
    with the `with scope` block. Owner: "yes".

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

**104:**
JOB.md 832-836: 104. **After `h.join()` the parent may use what the task used** (owner, 2026-10-27, gap map Q-C: "okay
    cool use it").
    - `[THR-11]` and `[THR-13]` check a task's claims until its join, not until the end of the
      `with scope:` block: `counts[0] += 1` after `h.join()`, inside the same block, is allowed.
    - Safe: after the join the task has finished.

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

**164:**
JOB.md 1251-1251: 164. **A `sharedref mut` link never goes to another thread**, as a `shares mut` owner does not: a type holding one is not `Send` and not `ThreadShared`, so `channel[Watcher]` for a `struct Watcher` holding `target: sharedref mut Player` is refused (as `thread.spawn` taking one already is, `E7015`); scoped tasks still reach the link where it is, through the caller's variables. Shown as (a) refuse it, (b) allow it with the race possible, recommendation (a). Owner (2026-11-02, gap-sweep group `K01-sharedref-send`): "yes a".

**165:**
JOB.md 1253-1253: 165. **`thread_shared` is refused where the value is made, when something in it is not safe for several threads to read at once** (`ThreadShared`): `stats = thread_shared FrameStats()` with `draws: RwLock[Cell[int]]` is `E7015` at that line, naming the field and the fix (`draws: Atomic[int]`). Shown as (a) refuse it where it is made, (b) refuse it only where it reaches another thread, recommendation (a). Owner (2026-11-02, gap-sweep group `K02-thread-shared-not-threadshared`): "a".

**166:**
JOB.md 1255-1255: 166. **A scoped task neither makes nor ends a `shares` owner** (whose count has no atomics), as it neither makes nor ends a `weakref` link to a one-thread value: `scope.spawn(owns fn(): level.enemies.clear())` with `enemies: Array[shares mut Enemy]` is `E7015`, the fix being to clear the list after the scope; the same for a task that ends an object `weakref` links on this thread point at. Shown as (a) this, (b) allowed unless the parent or another task touches the same type's counts, recommendation (a). Owner (2026-11-02, gap-sweep group `K03-task-ends-shares-owners`): "a, next question".

## allowed_rules
### [1]
ERR-3

### [2]
ERR-8

### [3]
THR-2

### [4]
THR-8

## rule_text
**ERR-3:**
p13-errors.md lines 87-92
* `[ERR-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* `Error` is the interface of error types:
  `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default `None`) (§IV.8).
  `@derive(Error)` on an enum or struct implements `Display` from `@error("…")` attributes — one per variant, whose `{name}` and
  `{0}` placeholders name the variant's fields — and implements `Debug` if it is not already derived.
  A variant marked `@from` whose single field is an error type gets `From` for that type and returns it
  from `source`.

**ERR-8:**
p13-errors.md lines 93-101
* `[ERR-8]` *(changed in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* **`AnyError`** is the
  prelude's "any error" type: a `dyn Error` held alone (`owns dyn Error`) plus an optional chain of
  context messages. It implements `From[E]` for every `E: needs Error`, so `?` converts any error into it,
  and `Display`, `Debug`, `source`, `[E]` above `fn downcast_ref() -> Option[ref E]`, and `[E]` above
  `fn is() -> bool`. **`AnyError` does not implement `Error`** — otherwise `[ERR-7]` and
  `[ERR-8]` would both supply `From[AnyError]` for `AnyError`. An `AnyError` passed through `?` into
  another `AnyError` is handed over (`[ERR-7]`). `AnyError.msg(owns text: String)`, a `static fn`,
  is an error whose text is `text`, so a function whose error type is `AnyError` fails with a message as `return
  Err(AnyError.msg(f"no such file: {p}"))`.

**THR-2:**
p11-concurrency.md lines 99-128
* `[THR-2]` *(changed in 0.9.9)* *(changed in 0.9.15)* **A one-thread value never reaches another
  thread.** A `shares` or `shares mut` owner counts its owners with plain adds, and every object that is
  not `thread_shared` counts its `weakref` links with plain adds. So no such owner, and no `weakref`
  link to such a value, reaches another thread by any route: captured or held by a function any thread
  may run (`thread_shared fn()`, the function of `thread.spawn` or `jobs.submit`, a callback C may run on
  another thread), handed to an unscoped thread or a job, or reachable from a `thread_shared` value
  (`[THR-1]`); and neither does a view of a place this thread owns, which an unscoped thread may outlive
  (`E7015`, whose message gives the words to write: the value made `thread_shared` from the start, or a
  copy). **Making, ending or opening a `weakref` link touches its target's header** (its weak count, and
  the test that it still lives): in a scoped task, a job, or any function that may run on another thread
  than its target's owner, it is allowed only for a `thread_shared` target, whose counts are kept for
  several threads and whose opened view holds a count (`[RC-4]`, `[RC-5]` point 2; `E7015`: "`players[0]` counts its links without atomics, and
  the task on line 9 makes one too; make the links after the scope, or make the player
  `thread_shared`"). **An object handed whole to another thread leaves its `weakref` links behind**
  (verdict 114; Rust's `Rc::try_unwrap`). At the hand-over the program reads the object's weak count,
  one compare. With no link, the object goes as it is. With a link, its contents move to a new block
  for the other thread, and the old block is marked ended (its `drop` does not run; the new block's
  runs once, on the other thread) and stays on this thread until its last link ends, so every link here
  finds it ended (`[WK-12]`): after `w = weakref names` and
  `h = thread.spawn(owns fn() owns names: names.items.len())`, `if w exists as p:` is skipped and
  `h.join()` gives 0. The same is done for every object the handed object owns whose class a `weakref`
  is made to anywhere in the program (the compiler knows those classes; one compare each), so after
  `cam.target = weakref level.enemies[0]`, handing `level` over leaves `cam.target` finding enemy 0
  ended. An object whose address C keeps (`[FFI-35a]`) never moves: a link to it at the hand-over
  stops the program ("`names` goes to another thread while `w`, a `weakref` link on this thread, points
  at it, and C keeps its address (`register(names)`, line 4), so it cannot move; end the link first:
  `w = None`"). A view opened from a link is not used after a hand-over that may move its object
  (`[RC-5]` point 1). The other thread never touches this thread's counts: the old block stays here
  with the links, and the new one starts with none. So an object that stays on one thread needs no
  atomic count.

**THR-8:**
p11-concurrency.md lines 43-58
* `[THR-8]` *(new in 0.9.9)* *(changed in 0.9.10)* *(changed in 0.9.15)* A type is **`Send`** when a value
  of it may be handed over to another thread: nothing reachable from it (its fields, its elements, what
  its `owns` parts hold, a function value's state) is a one-thread owner (`shares T`, `shares mut T`), a
  `weakref` link to a value that is not `thread_shared`, or bound to the thread that made it (a lock's
  guard, a `Scope` or `JobScope`, a raw pointer, a `threads(creator)` handle). Structs, enums, tuples,
  fixed arrays, collections and class objects are `Send` when every component is; an `owns T` when `T`
  is; `ref T` and `Span[T]` when `T` is `ThreadShared`, `ref mut T` and `MutSpan[T]` when `T` is
  `Send`; a `thread_shared T`, and a `weakref` link to one, when `T` is `Send` and `ThreadShared`;
  `dyn I` when `I` requires it (`interface I: needs Send:`). Of the function types, `thread_shared fn()` is
  `Send` (`[THR-1]`). A `Send` view still goes only where it cannot outlive what it views: to a scoped
  task or a job of a scope, never to an unscoped thread (`[THR-2]`). `Send` is inferred from the
  words and written only in bounds; the author of a type that holds a raw
  pointer declares it with `unsafe extend T implements Send:` and a `# SAFETY:` note (`E3100` without
  `unsafe`: "`Counter` holds a raw pointer, so the compiler cannot tell that another thread may have
  it; say that you vouch for it: write `unsafe extend Counter implements Send:` with a `# SAFETY:` note
  above it"). A thread-bound value sent to another thread is `E7008`.

## rules_not_found
