# K05-thread-shared-using

Question: May a thread_shared value take its memory from an arena or other manager?
Severity: high; impact rank 5
Members: J5-05; primary J5-05
Why together: Single question, but it must be answered before K34: its answer decides whether TYP-46's list may be restated as a property, which would bring thread_shared back in.
Critic's notes: Ask before K34 (J2-02/J5-09). If B, OBJ-1's spare header word gets a meaning (verdict 101, fix 1).

## Finding J5-05
Rules: OBJ-1, RC-4, ALC-1, TYP-46, LT-4; file tasks/spec-0.9.9/parts/p08-classes.md lines 69-71; kind undecided_case; severity high
Quote from Hardened_11:
Which thread made a value is read from
the header of the memory chunk its block came from, written once when the chunk is made, never from
the value.

The finder's question:
Does `thread_shared Stats() using level` compile? If so, where is its making thread read from, and on which
thread is its block given back to `level`?

Program:
```
from std.sync import Atomic

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    level = Arena.with_capacity(64 * 1024)
    stats = thread_shared Stats() using level        # [TYP-46]/[ALC-1]: `using` may follow thread_shared
    with scope = thread.scope():
        scope.spawn(fn() owns stats: stats.hits.fetch_add(1))   # handed over ([RC-4] point 4); its last owner ends on the task's thread
    println("done")
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.65)
Reason:
[TYP-46] (p04 433-434) and [ALC-1] allow `using` after `thread_shared`. [OBJ-1] (p08 69-71) reads the making
thread only from the header of the runtime's memory chunk (verdict 101, fix 1), and a manager's block has no
such header. After a hand-over ([RC-4] point 4), the last owner, and with it `drop` and the manager's
`deallocate`, may run on another thread. [LT-4]'s instance word is a kept `sharedref mut` link, which may be
changed on two threads only under a lock ([LT-46]). No rule refuses this or says how it runs (grep 'memory
chunk', 'making thread': OBJ-1 only). Whether it compiles is the owner's question.

options:
[{'letter': 'A', 'summary': '`using` may not follow `thread_shared`: such a value always takes its block from
the built-in manager.', 'code': 'from std.sync import Atomic\nimport std.thread\n\nclass Stats:\n    hits:
Atomic[int] = Atomic(0)\n\nfn main():\n    stats = thread_shared Stats()                         # its block
from the built-in manager\n    with scope = thread.scope():\n        scope.spawn(owns fn() owns stats:
stats.hits.fetch_add(1))\n    println("done")', 'outcome': 'The example\'s `thread_shared Stats() using level`
is E2114: "a `thread_shared` value takes its memory from the built-in manager, which every thread may give it
back to; remove `using level`". This version compiles and prints done.'}, {'letter': 'B', 'summary': 'Allowed
when the manager kind may be used from any thread and has one instance. The value then keeps its making thread
in its header.', 'code': 'from std.sync import Atomic\nimport std.thread\n\nclass Stats:\n    hits:
Atomic[int] = Atomic(0)\n\nfn main():\n    stats = thread_shared Stats() using System            # the
platform\'s manager: one instance, any thread may give memory back to it\n    with scope = thread.scope():\n
scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))\n    println("done")', 'outcome': "The example's
`using level` is refused, because an `Arena` is used by one thread (E7015, with that reason). `using System`,
or any manager kind with one instance that implements `ThreadShared`, compiles and prints done. Such a value
keeps its making thread in the 4 unused bytes of its header, since its block has no runtime chunk to read it
from."}]

recommendation:
{'letter': 'A', 'reason': "It keeps one way to find a shared value's making thread, the memory chunk that the
two-count design chose, and adds no second header path. A mistake becomes a refusal at the line, not a race at
run time. The program loses nothing it needs: its block can come from the built-in manager.", 'checklist': '3,
5 (one place for the making thread), 8 (a mistake is a refusal, not a race), 13 (no special path for values
made in a manager)'}

