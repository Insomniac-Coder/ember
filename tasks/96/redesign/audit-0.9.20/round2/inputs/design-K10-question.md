# K10-queued-drop-check-points (input for method 3; made by deep/v3/make_inputs.py from phase2.json, phase3.json, deep/phase1-groups.json, JOB.md and the part files)

## cluster
**cid:**
K10-queued-drop-check-points

**title:**
Where and when does a queued thread_shared value's drop run (inside a held lock, a walk, an idle worker, a C thread)?

### members
#### [1]
J7-01

#### [2]
J7-02

**primary:**
J7-01

**why_together:**
Both ask where the check points of RC-4 point 3 (a lead choice, not verdict 101) may empty the queue. J7-01 C and J7-02 B are the same idea ('never queued, all counts atomic'), so the owner's answer to one fixes the other.

**severity:**
high

**impact_rank:**
10

**notes:**
Present J7-01 C and J7-02 B as one option. J7-01 B must state its bound (see conflicts) to agree with J7-02's reason for rejecting C, 'leaves resources held for ever'.

## questions
### [1]
**id:**
J7-01

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
high

**confidence:**
0.75

#### rules
##### [1]
RC-4

##### [2]
WK-3

##### [3]
THR-20

##### [4]
THR-19

##### [5]
BRW-18

##### [6]
EFF-24

**reason:**
Nothing settles it. RC-4 point 3 puts check points at every loop's back edge and before every blocking call; RC-4 point 5 and WK-3 run a queued value's drop there. No rule counts those drops in the making thread's summary (EFF-24 counts what the code calls), none keeps check points out of a lock's `with` block, a walk or a live view, and THR-19 promises that every wait for a held lock is refused when compiling. THR-20 counts the drop only on the other thread. The check-point placement was a lead choice (decisions.md section 6, RC-4 point 3: 'the owner may overturn any'), not an owner verdict. Verdict 101 says only that the making thread deletes the value 'at its next check point'.

**title:**
Where may a queued thread_shared value's drop run: inside a held lock or a walk?

**problem:**
When another thread lets go of the last owner of a `thread_shared` object that travelled to it inside another object, the object goes back to the thread that made it, and that thread runs its `drop` at its next check point: a loop's back edge or a blocking call. That point can sit inside a `with lock:` block that the drop also takes, or inside a walk of a list that the drop pushes to. The specification does not say whether this compiles, and if it does, nothing stops the drop from hanging on the lock or changing the walked list.

**example:**
import std.thread
from std.sync import Lock

class Pool:                                   # a connection pool: a free list behind a lock (std::mutex + std::vector)
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:                                   # one connection
    id: int
    pool: thread_shared Pool
    fn drop():                                # gives its id back to the pool
        with this.pool.lock:
            this.pool.free.push(owns copy of this.id)

class Batch:                                  # a plain object, handed to the worker whole
    conns: Array[thread_shared Conn] = []

fn main():
    pool = thread_shared Pool()
    b = Batch(conns=[thread_shared Conn(id=1, pool=thread_shared pool)])   # the Conn is made on main
    h = thread.spawn(owns fn() owns b: b.conns.len())   # the Conn travels inside b; the worker ends its last owner
    for frame in 0..1000:                     # the server's main loop
        with pool.lock:                       # main holds the pool's lock
            for id in pool.free:              # this walk's back edge is a check point: Conn's drop may run HERE
                println(id)                   # ... and would wait for the lock main holds, and push to the walked list
    println(h.join())

**spec_today:**
RC-4 point 3 (p08-classes.md 169-172): "Check points. The compiler places them: at every loop's back edge, before every call that may block the thread (waiting for a lock, `join`, a channel's receive, a sleep) and when the thread ends." RC-4 point 5 (p08 179-181): "it is ended at its making thread's next check point." WK-3 (p08 365-367): "is ended at its making thread's next check point (`[RC-4]` point 5), and its `drop` runs there." THR-20 (p11-concurrency.md 296-301): "A `drop` runs where the last owner ends. ... a function that may run on another thread and holds such a count counts as possibly running that value's `drop`". THR-19 (p11 267-268): "A wait for a lock this thread holds is refused when compiling ... with no loop needed and nothing checked while the program runs". BRW-18 (p07-ownership.md 453-456): "While a `for` loop walks a collection ... nothing changes the collection, 
whatever the change".

#### options
##### [1]
**letter:**
A

**summary:**
Refuse it: a check point counts as a call that may run the drop of every queued type

**code:**
# the program above is refused at the inner `for` line (E7018):
#   "this loop's back edge may end a `Conn` another thread let go of, and `Conn`'s drop takes the
#    `Pool` lock this thread holds (line 12); walk a copy after the lock: ..."
# the programmer writes instead:
    for frame in 0..1000:
        with pool.lock:
            ids = copy of pool.free
        for id in ids:
            println(id)

**outcome:**
The original does not compile. The rewrite compiles; Conn's drop runs at a back edge outside the lock. Every loop under a lock, or walking a list, that a queued type's drop may touch is refused the same way.

##### [2]
**letter:**
B

**summary:**
Compile it: the compiler puts check points only where no queued drop can clash

**code:**
# the program above, unchanged
# the compiler empties main's queue only at check points where this thread holds no lock, walks no list
# and keeps no view that a queued type's drop may take, change or end (worked out from the drops' code),
# and always when the thread ends: here, the outer loop's back edge, never the inner walk's

**outcome:**
Compiles. While the worker runs, frames print nothing (the free list is empty). After the worker ends its last owner, the next outer back edge runs Conn's drop, which takes the lock and pushes 1. Later frames print 1, and `h.join()` prints 1.

##### [3]
**letter:**
C

**summary:**
Compile it: a type whose drop reaches outside itself is never queued

**code:**
# the program above, unchanged
# a `thread_shared` value whose type's drop changes anything outside its own fields (here: the pool)
# is counted for every thread from the start (all its counts atomic), so the thread that ends its last
# owner runs the drop there, as THR-20 says, checked there

**outcome:**
Compiles. The worker runs Conn's drop when it ends `b`, waits there for the pool's lock until main's `with` block ends, then pushes 1; later frames print 1. Every count change of a `Conn` is an atomic one, on main too: the cost measured for objects passed between threads (9% to 13% slower) for such types.

#### recommendation
**letter:**
B

**reason:**
It decides everything when compiling, keeps the measured speed of the two counts for every type, and refuses no correct program. A's refusals would hit ordinary loops under a lock. C gives up speed. The drop is delayed only until the thread's next point that is safe for it.

**checklist:**
2 (accepts correct programs), 3 (decided when compiling, nothing added while running), 4 (the compiler places the points, unseen), 7 (general: every queued type and every lock, walk and view), 13 (no special path: the queued drop is checked like any drop)

### [2]
**id:**
J7-02

**verdict:**
confirmed

**route:**
owner_odr

**severity:**
medium

**confidence:**
0.6

#### rules
##### [1]
RC-4

##### [2]
RC-3

##### [3]
THR-16

##### [4]
JOB-2

##### [5]
HR-3a

##### [6]
FFI-22

**reason:**
RC-4 point 3 places check points only in compiled Ember code, plus 'when the thread ends'. A job worker waiting in the runtime for work, and a host thread that has returned to C (attached by FFI-22, registered by HR-3a), run no Ember code, so a value queued to them waits with no bound. RC-3 says a value is ended 'never earlier' than its last owner, but sets no latest time. No rule makes the runtime run a check point when a worker goes idle or a host thread leaves Ember code. Verdict 101 says only 'always before the thread ends', and a pool worker ends only with the process, where THR-16 runs no drop.

**title:**
When does a value queued to an idle job worker or a C thread get its drop?

**problem:**
A job worker makes a `thread_shared` object that reaches the main thread inside another object, and main lets go of its last owner, so the object goes back to the worker to be ended at the worker's next check point. The worker is now idle, waiting for work inside the runtime, where the compiler placed no check point. The same happens to an object made in a callback on a thread C owns, once that thread has gone back into C. The specification does not say when such an object's `drop` runs, or whether it ever does.

**example:**
import std.jobs

class Texture:                            # a GPU texture a worker loaded
    id: int
    fn drop():
        println("texture", this.id, "freed")   # in an engine: hand the GPU memory back

class Bundle:                             # a plain object, combined into main's list
    textures: Array[thread_shared Texture] = []

fn load_all(ids: Span[int]) -> Array[Bundle]:
    out: Array[Bundle] = []
    with s = jobs.scope():
        for id in ids:
            s.submit(owns fn() copy of id: out.push(owns Bundle(textures=[thread_shared Texture(id=id)])))   # each Texture is made on a worker
    return out

fn render_frame(n: int):
    println("frame", n)

fn main():
    bundles = load_all([1, 2])
    bundles.clear()                       # main ends the last owners: each Texture goes back to the worker that made it
    frame = 0
    while true:                           # the game runs on; the workers wait for work and run no Ember code
        render_frame(frame)
        frame += 1

**spec_today:**
RC-4 point 3 (p08-classes.md 169-172): "The compiler places them: at every loop's back edge, before every call that may block the thread ... and when the thread ends. A value whose making thread has ended is ended at once by the thread that ends its last owner." RC-4 point 2 (p08 166-167): "A queued value is ended by nothing else." RC-3 (p08 148-149): "A value is ended when its last owner ends as the source says ... and never earlier". XI.5 (p11-concurrency.md 495-497): "The job system is a pool of worker threads ... The runtime starts it before `main`". HR-3a (p20-annex-b.md 39-40): "Every thread that can run Ember code is registered: host threads on attach (`[FFI-22]`), `thread.spawn` threads, and job workers." THR-16 (p11 206-207): "When `main` returns ... no drop runs."

#### options
##### [1]
**letter:**
A

**summary:**
The runtime also empties the queue when a worker goes idle and when a C thread leaves Ember

**code:**
# the program above, unchanged
# a job worker empties its queue before it waits for work, and is woken to do so when a value is
# put in its queue while it waits; a host thread empties its queue when its outermost Ember call
# returns to C

**outcome:**
Compiles. Right after `bundles.clear()`, "texture 1 freed" and "texture 2 freed" are printed by the workers, then the frames. A host thread's queued objects are ended before each callback returns to C.

##### [2]
**letter:**
B

**summary:**
An object made on a worker or a C thread is never queued

**code:**
# the program above, unchanged
# a `thread_shared` value made on a job worker or a host thread is counted for every thread from the
# start (all its counts atomic), so whichever thread ends its last owner ends it at once

**outcome:**
Compiles. "texture 1 freed" and "texture 2 freed" are printed by main during `bundles.clear()`. Every count change of such an object is atomic: the measured cost of objects passed between threads, for everything a worker makes.

##### [3]
**letter:**
C

**summary:**
Keep today's text: it waits until that thread next runs Ember code

**code:**
# the program above, unchanged; the specification says the drop may wait until the worker runs
# another job (or never, if it runs none)

**outcome:**
Compiles. While the game loop runs and no job is submitted, the two textures are never freed and nothing is printed for them. A Texture's GPU memory leaks for the life of the process.

#### recommendation
**letter:**
A

**reason:**
Every queued object is ended within one job or one callback of its last owner ending. It costs one queue test at points where the thread is already stopping, and keeps the two-count speed for everything. C leaves resources held for ever, and B pays atomics for every object a worker makes.

**checklist:**
2 (correct programs free their resources), 3 (the cheapest run-time point: a test where the thread already waits), 7 (general: every kind of registered thread, no known ceiling), 13 (the same queue model for workers and C threads as for Ember threads)

## group
**gid:**
P05

**principle:**
Every end the program does not write as a call (above all a thread_shared value's drop queued back to its making thread, and any drop the compiler places) counts as a call at each point where it may run, so the lock, walk and view rules see it, and it runs within a stated bound: before the thread waits idle, leaves Ember for C, or ends.

**shared_cause:**
RC-4 point 3, a lead choice and not verdict 101, puts the check points that run queued drops at loop back edges and blocking calls. No rule counts those drops as calls there: EFF-24 counts only what code calls, and THR-20 counts the drop only on the other thread. Nothing bounds when they run on a thread that is idle in the runtime or back in C. So the same hidden end can clash with a held lock or a walk (J7-01) or never run (J7-02).

### members
#### [1]
J7-01

#### [2]
J7-02

### clusters
#### [1]
K10-queued-drop-check-points

### answered
### verdicts_involved
#### [1]
43: a call that may take a lock this thread holds is refused when compiling

#### [2]
56: an object counts as ended before its drop runs

#### [3]
57: destroying an object counts as calling its drop

#### [4]
101: the making thread deletes at its next check point, always before the thread ends

#### [5]
114: the moved object's drop runs once, on the other thread

**other_places:**
Drops at a scope's end inside a with block, clear() and replacement inside a walk (verdict 57), a weakref target's end (WK-3), statement-end temporaries (DRP-3) under a lock, a reload's reclaim (HR-2a), program exit (THR-16: no drop after main returns), pool workers that end only with the process, callbacks C runs into Ember at arbitrary points, generators abandoned mid-iteration.

**severity:**
high

**rank:**
5

### depends_on
#### [1]
P01

**notes:**
Present J7-01 C and J7-02 B as one option ('never queued, all counts atomic'). J7-01 B must state its bound (phase 3 conflict): either refuse a loop with no clash-free check point (E7018, naming the lock), or also empty the queue when the clashing lock is released. With J7-02 A, every queued object is then ended within one job or one callback. J7-01 A is verdict 57's rule applied to queued drops.

## conflicts_naming_a_member
### [1]
#### ids
##### [1]
J7-01

##### [2]
J7-02

**conflict:**
J7-02 rejects C because 'C leaves resources held for ever'. J7-01 B ('the compiler empties main's queue only at check points where this thread holds no lock, walks no list and keeps no view that a queued type's drop may take') has the same hole. A thread whose loop runs entirely inside a `with lock:` block (or always walks a list) never reaches such a point, so the queued drop waits until the thread ends, or for ever for a pool worker or main (THR-16: no drop runs once main returns).

**resolution:**
J7-01 B must state its bound in the draft. Either (i) the compiler refuses a loop of a thread that may hold queued values when the loop has no clash-free check point (E7018, naming the lock), or (ii) the queue is also emptied when the clashing lock is released. Then J7-01 B and J7-02 A together keep 'every queued object is ended within one job or one callback'.

## verdicts_involved
**43:**
JOB.md 361-366: 43. **A call that may take a lock this thread already holds is refused when compiling** (area E's
    question 2). `flush` holds `log`'s lock (`with log.lock:`) and calls `note()`, a `fn() changes Log`
    whose `push` takes the same lock: in C++ `m.lock(); m.lock();`, the program freezes. Ember refuses the
    call (E7018: "call `note()` after the `with log.lock:` block"), with no loop needed; nothing is checked
    while running. A function that changes a different `Log` is refused too (the compiler cannot tell
    which). Owner: "B".

**56:**
JOB.md 450-457: 56. **An object counts as ended the moment its last owner lets it go, before its `drop` runs**
    (safety validation (b), hole 4, first part: C1's G14, W4's question 7). A `weakref` opened during its
    `drop`, or during the `drop` of anything it owns (a child's `drop` opening `this.parent` while the
    parent is being destroyed), finds it ended, so the `if ... exists as p:` block is skipped and the
    half-destroyed object is never read, nor counted again (the count pair that would end it a second
    time cannot arise). As C++'s `weak_ptr` and Rust's `Weak`. Ending the link itself
    (`this.parent = None`) stays allowed: it only subtracts the link's weak count in the target's header,
    which the weak count keeps in memory (section 0). Owner: "a".

**57:**
JOB.md 458-465: 57. **Destroying an object counts as calling its `drop`** (safety validation (b), hole 4, second part: C1's
    G12). Anything that may end an object (a list cleared, a value replaced, a local ending with its
    function) counts as a call of the object's `drop`, with what its `fn` line says it changes. So with
    `class Enemy:` whose `fn drop() changes LOG:` pushes to `LOG`, `enemies.clear()` inside
    `for line in LOG:` is refused (E3020: clear after the loop), as C++'s `~Enemy()` pushing to the vector
    a range-for walks breaks the loop; and, by verdict 30, a function that may destroy an `Enemy` says
    `changes LOG`. A `drop` may still change things outside its object (a log line, leaving a list).
    Owner: "a that makes sense", over (b), a `drop` changing nothing outside its own object.

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

## allowed_rules
### [1]
BRW-18

### [2]
EFF-24

### [3]
FFI-22

### [4]
HR-3a

### [5]
JOB-2

### [6]
RC-3

### [7]
RC-4

### [8]
THR-16

### [9]
THR-19

### [10]
THR-20

### [11]
WK-3

## rule_text
**BRW-18:**
p07-ownership.md lines 453-489
* `[BRW-18]` *(new in 0.9.15)* **Loops, generators included.** While a `for` loop walks a collection (a
  list, a map, a set or a string), or a generator that walks it (a `gen fn` paused in the middle of it,
  `[LT-1]`) will still be used, nothing changes the collection, whatever the change: nothing is added or
  removed, no element is written or replaced and no field of one changed (`xs[0] = 5`,
  `enemies[0].hp -= 1`), no value under a key replaced (`counts[k] *= 2`, `names[k] = String("bob")`),
  through the collection's own name or through any call that may change it (`[EFF-1]`: a call through
  a function value, a library call given a function that changes it, a `spawn` whose thread changes it
  other than under a lock the loop holds, a foreign call while a callback C keeps changes it, and the
  end of an object whose `drop` changes it, `[DRP-8]`). It is `E3020`, at the change:
  "`enemies.push(...)` adds to `enemies`, which this loop walks (line 3); collect the new enemies in a
  list of your own: write `spawned: Array[Enemy] = []` before the loop,
  `spawned.push(owns Enemy(hp=5, splits=false))` in place of this line, and `enemies.extend(spawned)` after
  the loop"; "`counts.insert(...)` (line 4) adds to `counts`, whose keys this loop walks (line 3);
  collect the new keys in a list of your own: write `upper: Array[String] = []` before the loop,
  `upper.push(owns k.to_upper())` in place of this line, and `for u in owns upper: counts.insert(owns u, owns 0)` after
  the loop"; "`b.click()` (line 17) adds to `LOG` (through `click`, which runs the handler stored on
  line 9, `log_click`, which pushes to `LOG`) while this loop walks `LOG` (line 16); click after the
  loop: move line 17, `b.click()`, below the loop"; "`d.log.push(owns "open")` (line 3) changes `d.log`,
  which `g` walks (`g = d.replay()`, line 1), and `g` is used again on line 4; push after `g`'s last use:
  move `d.log.push(owns "open")` below line 4". **The loop's own handle may change the item it is on**:
  `for ref mut x in xs: x = 5` gives `[5, 5, 5]`; `for k, ref mut v in counts: v *= 2`;
  `for k, ref mut v in names: v = String("bob")`; and the message gives that line: "`xs[0] = 5` (line
  3) writes an element of `xs`, which this loop walks (line 2); to change the element the loop is on,
  write `for ref mut x in xs:` in place of line 2 and `x = 5` in place of line 3"; "`counts[k] *= 2`
  (line 4) changes `counts`, whose keys this loop walks (line 3); change each value through the loop's
  own handle: write `for k, ref mut v in counts:` in place of line 3 and `v *= 2` in place of line 4".
  Anything else goes after the loop or after the generator's last use (`retain`, `extend`, `Commands`).
  A counting loop `for i in 0..xs.len():`, whose range is fixed when it starts, does not walk `xs`: it
  may write its elements and add to it, but not remove from it (`E3020`: "`enemies.remove(i)` (line 4)
  shortens `enemies`, which `for i in 0..enemies.len():` (line 2) counts over with an end fixed when it
  started; remove after the loop: instead of `if enemies[i].hp <= 0:` (line 3) and line 4, write
  `enemies.retain(owns fn(e) => e.hp > 0)` after the loop (and remove the `for` line if nothing is left in
  it)"). A `while` loop that tests its position against the list's current length at every step may
  add and remove: it is not a walk, and each index is checked. A call through a function value that may
  take a lock this thread holds, inside a walk under that lock, is `E7018`, not `E3020` (§XI.2):
  the message whose fix compiles is the one shown (`[DIA-28]`).
  For a call through a stored function that may change the walked list: E3020 ("`h(this)` (line 8) may clear `this.handlers` (through `close_all`, stored on line 20) while this loop walks `this.handlers` (line 7); give each handler the Ui to change, `handlers: Array[fn(ref mut Ui)]`, and walk by position instead of the `for` line: `this.current = 0`, `while this.current < len(this.handlers):`, `this.handlers[this.current](ref mut this)`, `this.current += 1`").

**EFF-24:**
p10-effects.md lines 90-105
* `[EFF-24]` *(new in 0.9.15)* **What a function does is worked out from its code.** Beside its effect
  set the compiler works out, for every function, its summary: what it may change, read and end, and
  through which chain of calls. It is computed over the same call graph as `[EFF-1]`, from the
  function's own lines; from the functions it calls; for a call through a function value, an interface,
  a `dyn` or a `virtual` method, from every function the program can put there (`[EFF-2]`); from what
  the threads, tasks and jobs it starts and the callbacks it hands to C change, now or later
  (`[EFF-26]`); and from the `drop` of every object it may end (`[DRP-8]`), a function value or an
  `owns dyn I` value replaced or ended counting the `drop`s of what its state owns. A `drop` changes its
  own object as a given. The summary also records which elements of a list the function touches;
  whether it only adds at the end of a `ref mut` list parameter (`push`, `extend`) and reads or writes no
  element already in it; whether it gives the link of an object it is given a target before reading it
  (Part VIII); which of its view parameters, `this` included, it uses after a line that may end or move
  what they view; what it reaches by other names while a `ref mut` parameter is used; and which locks it
  may take, or needs held (`[THR-18]`). No word on a function's first line says any of it: `changes`,
  `reads` or another clause after a function's brackets is `E0100` ("`changes FRAME` is not Ember: the
  compiler works out what `tick` changes from its body; write `fn tick():`").

**FFI-22:**
p16-ffi.md lines 435-450
* `[FFI-22]` *(changed in 0.9.9)* *(changed in 0.9.15)* Every exported function and trampoline first attaches the calling
  thread to the runtime (`ember_rt_thread_attach()`, a thread-local check after the first time). The
  compiler cannot see which thread C calls a callback from, so the callback's contract is the only
  evidence, and it is an asserted fact (`[TCB-1]`): `threads(main)` or `threads(creator)`, which the
  trampoline checks in every profile (`[FFI-53]`), or `threads(any)`, the default. A callback C may
  call from any thread, borrowed or kept, is a `thread_shared fn()` (Part VI): every value it holds is
  `Send` (`[THR-8]`): a one-thread owner (`shares`, `shares mut`, a `weakref` link of one) is `E7015`,
  a thread-bound value (a lock's guard, a raw pointer, a `threads(creator)` handle) `E7008`; and it
  changes the caller's real variables only by their own operations, as an `Atomic`, a lock or a
  `guarded by` field under its lock is changed (`E7015`: "C may call `visit` from any thread at once
  (`visit_each`'s `visit` has no `threads(...)` word), and it pushes to `seen`, a list of this thread;
  count with an atomic, or, if C calls it only on this thread, say so in the overlay:
  `visit: callback(borrowed, user_data=data) threads(creator)`"). The `E7015` message for a kept
  callback that holds a `shares` owner gives the lines that share the value between threads
  (`thread_shared`, with `Atomic` fields).
  Its message: E7015 ("C may call this function from any thread (`threads(any)`), and `shares` owners stay on one thread; share the game between threads: write `game = thread_shared Game(names=[String("ann"), String("bo")])` and `token = setup(game)` in `main`, `fn setup(game: thread_shared Game) -> Retained[ForeignCallback]:` and `return register_cb(fn() thread_shared game: game.add_point())` in `setup`, and, since `add_point` changes `score`, `from std.sync import Atomic` at the top, `score: Atomic[i32] = Atomic(0)`, `this.score.fetch_add(1)` as `add_point`'s body, and `println(game.score.load())`").

**HR-3a:**
p20-annex-b.md lines 39-41
* `[HR-3a]` Every thread that can run Ember code is registered: host threads on attach (`[FFI-22]`),
  `thread.spawn` threads, and job workers. `debug` builds assert on entry to Ember code from an
  unregistered thread.

**JOB-2:**
p11-concurrency.md lines 519-524
* `[JOB-2]` *(changed in 0.9.9)* *(changed in 0.9.15)* `jobs.scope()` follows `[THR-5]` and `[THR-11]`:
  `s.submit` on a `JobScope` takes a plain function (`owns f: fn()`), whose jobs may use the caller's own
  variables, and the block's exit joins them. `jobs.submit` outside a scope takes a function any thread
  may run, `pub fn submit(owns f: thread_shared fn()) -> JobHandle:`, as `thread.spawn` does
  (`[THR-10]`, `[THR-17]`): a function that holds a one-thread value is `E7015`, and a name used with no
  capture word `E7004`. `jobs.wait(h)` waits for it.

**RC-3:**
p08-classes.md lines 147-154
* `[RC-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* Further removals of count operations are allowed
  only when meaning is preserved (`[PHIL-5]`). A value is ended when its last owner ends as the source
  says (an owner's scope ending, its replacement, `mem.drop`) and never earlier: a removal takes away a
  +1 and a -1 only when it moves no `drop`, no outcome of an `exists as` opening and no foreign release
  across that point. A surviving owner in a field, a collection or a function value's state keeps the
  value alive. The last use of a view ends the view, not the owner (`[BRW-2]`) (ODR-063). `unsafe` code
  that keeps a raw pointer into an object keeps an owner of it alive for as long as it uses the pointer;
  `with h:` and `mem.keep_alive(h)` state that where no Safe use does.

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

**THR-16:**
p11-concurrency.md lines 206-208
* `[THR-16]` *(new in 0.9.9)* When `main` returns, or `process.exit` is called, the process ends: other
  threads stop without running destructors, no static is freed and no drop runs. The `debug` leak
  report (`[WK-15]`) runs only if no other thread is still running, and otherwise says how many were.

**THR-19:**
p11-concurrency.md lines 267-284
* `[THR-19]` *(new in 0.9.15)* **A wait for a lock this thread holds is refused when compiling.** Inside a
  lock's `with` block, these are `E7018`, with no loop needed and nothing checked while the program runs:
  a call through a function value, or through a `dyn` or `virtual` method, that may run a function taking
  a lock of the kind this thread holds (a `Log`'s lock, while it holds `log`'s: the compiler cannot tell
  which object's lock a stored function takes, so one that takes another `Log`'s lock is refused too, and
  the way out moves one line); a join of a
  thread, or the end of a `with scope` block, whose task changes what the held lock guards; and a direct
  call that may take the held lock and is not proved to enter past it (`[THR-18]`). The messages: "`note()`
  may run the function stored on line 12, which takes a `Log`'s lock (`log.push`, under `lock`), and this
  thread holds `log`'s lock, so taking it again would wait for ever; call `note()` after the
  `with log.lock:` block: move line 16 below the block"; "`t.join()` (line 16) waits for the thread
  started on line 15, whose function changes `RESULTS` under `RESULTS_LOCK`, and this thread holds that
  lock, so each would wait for the other for ever; join after the `with RESULTS_LOCK:` block: move line
  16 below the block". Inside a walk under that lock, such a call is `E7018`, not `E3020`, whose fix
  (after the loop, still inside the lock) would still wait for ever ("this loop walks `log.lines` (line
  9) under `log`'s lock, and `note()` (line 11) may run the function stored on line 4, which takes a
  `Log`'s lock, so it would wait for the lock this thread holds; call `note()` after the `with log.lock:`
  block: move line 11 below the block").

**THR-20:**
p11-concurrency.md lines 285-325
* `[THR-20]` *(new in 0.9.15)* **Module variables and threads.** A `static mut` (`[STA-1]`) is reached only
  by the main thread and the tasks of scopes it opens, unless it is `guarded by` a `static` `Lock` and
  used inside `with` that lock: a function that may run on another thread (reachable from
  `thread.spawn`, from a job submitted outside a scope, from a callback C may run on a thread other than
  the main one, or from an `@unsafe(export("name", threads=any))` or `@unsafe(export("name", threads=creator))`
  function) and reads or
  changes it is `E7012` ("`tick` changes `FRAME` and `LOG`, module variables of the main thread, and this
  thread runs beside it; guard both with one lock: write `from std.sync import Lock` at the top,
  `static TICK_LOCK: Lock = Lock()`, then `static mut FRAME: int guarded by TICK_LOCK = 0` instead of
  line 1 and `static mut LOG: Array[String] guarded by TICK_LOCK = []` instead of line 2, and put
  `tick`'s body inside `with TICK_LOCK:`"). A `static` of a type that changes only through itself
  (`Lock`, `Atomic`, `Mutex`, `RwLock`, a channel end) is usable from any thread. **A `drop` runs where
  the last owner ends.** The last owner of a `thread_shared` value may end on any thread that holds a
  count of it: an owner, or a view opened from a `weakref` link, which holds one for its life. So a
  function that may run on another thread and holds such a count counts as possibly running that
  value's `drop`, and the `drop` of everything it owns: what they change is changed on that thread, and
  a module variable of the main thread they reach is `E7012`.

  ```ember
  import std.thread
  from std.sync import Lock

  static RESULTS_LOCK: Lock = Lock()
  static mut RESULTS: Array[int] guarded by RESULTS_LOCK = []

  fn worker(x: int):
      r = x * x
      with RESULTS_LOCK:
          RESULTS.push(owns r)

  fn main():
      worker(1)
      handles: Array[thread.JoinHandle[void]] = []
      with RESULTS_LOCK:
          for r in RESULTS:
              handles.push(owns thread.spawn(owns fn() copy of r: worker(r)))
      for h in owns handles:
          h.join()
      with RESULTS_LOCK:
          println(RESULTS.len())
  ```

**WK-3:**
p08-classes.md lines 359-367
* `[WK-3]` *(changed in 0.9.15)* **An object is ended the moment its last owner lets it go, before its
  `drop` runs**, as C++'s `weak_ptr` and Rust's `Weak` see it: a `weakref` link opened during its
  `drop`, or during the `drop` of anything it owns (a child's `drop` opening `this.parent` while the
  parent is being ended), finds it ended, so the `if ... exists as p:` block is skipped and the
  half-ended object is never read, nor counted again. Ending the link itself (`this.parent = None`)
  stays allowed: it only subtracts the link's weak count in the target's header, which that count keeps
  in memory. A `thread_shared` value whose last owner ends on a thread that did not make it, and that
  was not handed over, is ended at its making thread's next check point (`[RC-4]` point 5), and its
  `drop` runs there.

## rules_not_found
