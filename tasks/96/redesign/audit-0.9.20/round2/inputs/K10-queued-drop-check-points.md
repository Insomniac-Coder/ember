# K10-queued-drop-check-points

Question: Where and when does a queued thread_shared value's drop run (inside a held lock, a walk, an idle worker, a C thread)?
Severity: high; impact rank 10
Members: J7-01, J7-02; primary J7-01
Why together: Both ask where the check points of RC-4 point 3 (a lead choice, not verdict 101) may empty the queue. J7-01 C and J7-02 B are the same idea ('never queued, all counts atomic'), so the owner's answer to one fixes the other.
Critic's notes: Present J7-01 C and J7-02 B as one option. J7-01 B must state its bound (see conflicts) to agree with J7-02's reason for rejecting C, 'leaves resources held for ever'.

## Finding J7-01
Rules: THR-20, RC-4, WK-3, THR-19, BRW-18, EFF-6; file tasks/spec-0.9.9/parts/p11-concurrency.md lines 296-301 (and p08-classes.md 169-172, 179-181); kind contradiction; severity high
Quote from Hardened_11:
**A `drop` runs where the last owner ends.** The last owner of a `thread_shared` value may end on any thread
that holds a / 3. **Check points.** The compiler places them: at every loop's back edge, before every call
that may

The finder's question:
When a queued `thread_shared` value is ended at its making thread's check point (a loop's back edge), its
`drop` runs there. Does the program below compile, and if it does, what stops the drop from taking a lock this
thread holds, or from pushing to the list this loop walks?

Program:
```
import std.thread
from std.sync import Lock

class Pool:
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:
    id: int
    pool: thread_shared Pool
    fn drop():
        with this.pool.lock:
            this.pool.free.push(owns this.id)

class Batch:
    conns: Array[thread_shared Conn] = []

fn main():
    pool = thread_shared Pool()
    b = Batch(conns=[thread_shared Conn(id=1, pool=thread_shared pool)])
    h = thread.spawn(owns fn() owns b: b.conns.len())   # Conn travels inside b: no hand-over (RC-4 point 5)
    with pool.lock:
        for id in pool.free:          # back edge = check point; the queued Conn's drop runs here
            println(id)
    h.join()
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.75)
Reason:
Nothing settles it. RC-4 point 3 puts check points at every loop's back edge and before every blocking call;
RC-4 point 5 and WK-3 run a queued value's drop there. No rule counts those drops in the making thread's
summary (EFF-24 counts what the code calls), none keeps check points out of a lock's `with` block, a walk or a
live view, and THR-19 promises that every wait for a held lock is refused when compiling. THR-20 counts the
drop only on the other thread. The check-point placement was a lead choice (decisions.md section 6, RC-4 point
3: 'the owner may overturn any'), not an owner verdict. Verdict 101 says only that the making thread deletes
the value 'at its next check point'.

options:
[{'letter': 'A', 'summary': 'Refuse it: a check point counts as a call that may run the drop of every queued
type', 'code': '# the program above is refused at the inner `for` line (E7018):\n#   "this loop\'s back edge
may end a `Conn` another thread let go of, and `Conn`\'s drop takes the\n#    `Pool` lock this thread holds
(line 12); walk a copy after the lock: ..."\n# the programmer writes instead:\n    for frame in 0..1000:\n
with pool.lock:\n            ids = copy of pool.free\n        for id in ids:\n            println(id)',
'outcome': "The original does not compile. The rewrite compiles; Conn's drop runs at a back edge outside the
lock. Every loop under a lock, or walking a list, that a queued type's drop may touch is refused the same
way."}, {'letter': 'B', 'summary': 'Compile it: the compiler puts check points only where no queued drop can
clash', 'code': "# the program above, unchanged\n# the compiler empties main's queue only at check points
where this thread holds no lock, walks no list\n# and keeps no view that a queued type's drop may take, change
or end (worked out from the drops' code),\n# and always when the thread ends: here, the outer loop's back
edge, never the inner walk's", 'outcome': "Compiles. While the worker runs, frames print nothing (the free
list is empty). After the worker ends its last owner, the next outer back edge runs Conn's drop, which takes
the lock and pushes 1. Later frames print 1, and `h.join()` prints 1."}, {'letter': 'C', 'summary': 'Compile
it: a type whose drop reaches outside itself is never queued', 'code': "# the program above, unchanged\n# a
`thread_shared` value whose type's drop changes anything outside its own fields (here: the pool)\n# is counted
for every thread from the start (all its counts atomic), so the thread that ends its last\n# owner runs the
drop there, as THR-20 says, checked there", 'outcome': "Compiles. The worker runs Conn's drop when it ends
`b`, waits there for the pool's lock until main's `with` block ends, then pushes 1; later frames print 1.
Every count change of a `Conn` is an atomic one, on main too: the cost measured for objects passed between
threads (9% to 13% slower) for such types."}]

recommendation:
{'letter': 'B', 'reason': "It decides everything when compiling, keeps the measured speed of the two counts
for every type, and refuses no correct program. A's refusals would hit ordinary loops under a lock. C gives up
speed. The drop is delayed only until the thread's next point that is safe for it.", 'checklist': '2 (accepts
correct programs), 3 (decided when compiling, nothing added while running), 4 (the compiler places the points,
unseen), 7 (general: every queued type and every lock, walk and view), 13 (no special path: the queued drop is
checked like any drop)'}

## Finding J7-02
Rules: RC-4, THR-16, JOB-2, HR-3a; file tasks/spec-0.9.9/parts/p08-classes.md lines 169-172, 179-181 (and p11-concurrency.md 206-208, 495-497); kind undecided_case; severity medium
Quote from Hardened_11:
3. **Check points.** The compiler places them: at every loop's back edge, before every call that may / block
the thread (waiting for a lock, `join`, a channel's receive, a sleep) and when the thread / ends. A value
whose making thread has ended is ended at once by the thread that ends its last

The finder's question:
A value made on a job worker, or on a host thread that has gone back into C, is queued to that thread, which
runs no compiled Ember code and so reaches no check point. When does its `drop` run? Ever? And does the main
thread's end, a check point, run queued drops, when THR-16 says no drop runs once `main` returns?

Program:
```
import std.jobs

class Asset:
    file: File
    fn drop():
        this.file.close()

class Bundle:
    assets: Array[thread_shared Asset] = []

fn load_all(paths: Array[String]) -> Array[Bundle]:
    out: Array[Bundle] = []
    with s = jobs.scope():
        for p in paths:
            s.submit(owns fn() ref p: out.push(owns Bundle(assets=[thread_shared Asset(file=File.open(p))])))
    return out

fn main():
    bundles = load_all([String("a.bin"), String("b.bin")])
    bundles.clear()      # last owners end on main; each Asset is queued to the worker that made it
    # the worker sits idle in the runtime: is each file ever closed?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.6)
Reason:
RC-4 point 3 places check points only in compiled Ember code, plus 'when the thread ends'. A job worker
waiting in the runtime for work, and a host thread that has returned to C (attached by FFI-22, registered by
HR-3a), run no Ember code, so a value queued to them waits with no bound. RC-3 says a value is ended 'never
earlier' than its last owner, but sets no latest time. No rule makes the runtime run a check point when a
worker goes idle or a host thread leaves Ember code. Verdict 101 says only 'always before the thread ends',
and a pool worker ends only with the process, where THR-16 runs no drop.

options:
[{'letter': 'A', 'summary': 'The runtime also empties the queue when a worker goes idle and when a C thread
leaves Ember', 'code': '# the program above, unchanged\n# a job worker empties its queue before it waits for
work, and is woken to do so when a value is\n# put in its queue while it waits; a host thread empties its
queue when its outermost Ember call\n# returns to C', 'outcome': 'Compiles. Right after `bundles.clear()`,
"texture 1 freed" and "texture 2 freed" are printed by the workers, then the frames. A host thread\'s queued
objects are ended before each callback returns to C.'}, {'letter': 'B', 'summary': 'An object made on a worker
or a C thread is never queued', 'code': '# the program above, unchanged\n# a `thread_shared` value made on a
job worker or a host thread is counted for every thread from the\n# start (all its counts atomic), so
whichever thread ends its last owner ends it at once', 'outcome': 'Compiles. "texture 1 freed" and "texture 2
freed" are printed by main during `bundles.clear()`. Every count change of such an object is atomic: the
measured cost of objects passed between threads, for everything a worker makes.'}, {'letter': 'C', 'summary':
"Keep today's text: it waits until that thread next runs Ember code", 'code': '# the program above, unchanged;
the specification says the drop may wait until the worker runs\n# another job (or never, if it runs none)',
'outcome': "Compiles. While the game loop runs and no job is submitted, the two textures are never freed and
nothing is printed for them. A Texture's GPU memory leaks for the life of the process."}]

recommendation:
{'letter': 'A', 'reason': 'Every queued object is ended within one job or one callback of its last owner
ending. It costs one queue test at points where the thread is already stopping, and keeps the two-count speed
for everything. C leaves resources held for ever, and B pays atomics for every object a worker makes.',
'checklist': '2 (correct programs free their resources), 3 (the cheapest run-time point: a test where the
thread already waits), 7 (general: every kind of registered thread, no known ceiling), 13 (the same queue
model for workers and C threads as for Ember threads)'}

