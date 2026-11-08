# K02-thread-shared-not-threadshared

Question: May a type that is not safe to read from two threads be made thread_shared?
Severity: high; impact rank 2
Members: J7-03; primary J7-03
Why together: Single question.
Critic's notes: Data race on a Cell under an RwLock read guard. Under A, a wrapper of a thread-safe C object can become thread_shared only if K36 (J7-06) gives it a way to vouch ThreadShared. Mention that pairing in the draft.

Status: answered by the owner, verdict 165 (tasks/96/redesign/JOB.md).

## Finding J7-03
Rules: THR-1, THR-8, THR-9, THR-13; file tasks/spec-0.9.9/parts/p11-concurrency.md lines 49-51, 65-68, 80-87; kind undecided_case; severity high
Quote from Hardened_11:
`Send`; a `thread_shared T`, and a `weakref` link to one, when `T` is `Send` and `ThreadShared`; / `dyn I`
when `I` requires it (`interface I: needs Send:`). Of the function types, `thread_shared fn()` is / `Send`
(`[THR-1]`).

The finder's question:
Is `thread_shared X()` refused when `X` is `Send` but not `ThreadShared`? Does the program below compile (two
threads set one `Cell` under read guards)?

Program:
```
import std.thread
from std.sync import RwLock

class Cache:
    hits: RwLock[Cell[int]] = RwLock(Cell(0))

fn bump(c: thread_shared Cache):
    with g = c.hits.read():
        g.set(g.get() + 1)

fn main():
    c = thread_shared Cache()
    h = thread.spawn(owns fn() thread_shared c: bump(c))
    bump(c)
    h.join()
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.65)
Reason:
THR-1's checks at the line that makes a value `thread_shared` are E7014 (no direct change) and E7015
(everything reachable is `Send`). Neither asks for `ThreadShared`. THR-9 itself says `RwLock[Cell[int]]` is
not `ThreadShared`, and CELL-3 says `Cell` is `Send` but not `ThreadShared`, so a class holding one is `Send`
and passes E7015. THR-8 makes a `thread_shared T` owner `Send` only for a `ThreadShared` T, yet makes
`thread_shared fn()` `Send` with no condition, and THR-1 lets such a function hold any `thread_shared` owner.
E7014's list ('through ... an RwLock') does not clearly refuse `Cell.set` under a read guard. Searched
TYP-43's table (p04 577-578), E7014/E7015 rows (p17a 216-217), JOB.md verdicts 61, 94 and 101: none settles
it.

options:
[{'letter': 'A', 'summary': 'Refuse it where the value is made thread_shared', 'code': 'stats = thread_shared
FrameStats()\n# E7015: "`FrameStats.draws` is an `RwLock` of a `Cell`, and threads holding its read lock at
once\n#  would change the `Cell` together; write `draws: Atomic[int] = Atomic(0)` in `FrameStats`\n#  (with
`from std.sync import Atomic` at the top)"', 'outcome': 'Does not compile, at the `thread_shared FrameStats()`
line, naming the field. With `Atomic[int]` it compiles and prints 2 every time.'}, {'letter': 'B', 'summary':
'Allow making it; refuse it where it reaches another thread', 'code': 'stats = thread_shared FrameStats()
# accepted\nh = thread.spawn(owns fn() thread_shared stats: count_draw(stats))\n# E7015 here: "a
`thread_shared FrameStats` cannot go to another thread: `FrameStats.draws` ..."', 'outcome': 'Does not
compile, at the `thread.spawn` line. A `thread_shared FrameStats` that never leaves its thread compiles,
though `thread_shared` then buys nothing.'}]

recommendation:
{'letter': 'A', 'reason': "`thread_shared` means 'owners on several threads', so the check belongs where the
word is written, as `Bus.handlers` is refused at that line today. B accepts a word that can never do what it
says.", 'checklist': '5 (`thread_shared` keeps one meaning), 6 (the same line and code as the nearest rule,
E7015 for `Bus.handlers`), 8 (a forgotten `Atomic` is a refusal, not a race), 9 (the message gives a line that
compiles)'}

