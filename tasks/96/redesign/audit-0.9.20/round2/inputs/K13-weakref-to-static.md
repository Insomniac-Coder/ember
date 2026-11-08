# K13-weakref-to-static

Question: May a weakref link point at a class object held in a static?
Severity: medium; impact rank 13
Members: J6-06; primary J6-06
Why together: Single question.
Critic's notes: As written today it writes into read-only memory, which crashes.

## Finding J6-06
Rules: CT-5, STA-1, WK-11, THR-9; file tasks/spec-0.9.9/parts/p14-comptime.md lines 66-68; kind undecided_case; severity high
Quote from Hardened_11:
* in a `static` (not `static mut`) whose type gives no name a way to change the value (no `Mutex`,
    `RwLock`, `Cell`, `Atomic` or `guarded by` field on the path to it), the value is materialised in
    read-only memory; statics are never dropped;

The finder's question:
May a program make a `weakref` link to a class object held in a `static`, from one thread or from several? If
it may, where is the weak count kept, given that CT-5 puts the object in read-only memory and WK-11 counts
every link in the target's header?

Program:
```
class Node:
    x: int = 0

static ORIGIN: Node = Node(x=0)

fn track():
    keep = weakref ORIGIN       # +1 on ORIGIN's weak count
    if keep exists as n:
        println(n.x)

fn main():
    h = thread.spawn(owns fn(): track())
    track()
    h.join()
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
Partly refuted: from another thread, a `weakref` to a static's object is already refused, since THR-2 (p11
107-110) allows making a link on a thread other than the target's owner only for a thread_shared target. The
one-thread case stays open: CT-5 (p14 66-68) puts a static whose type has no
Mutex/RwLock/Cell/Atomic/guarded-by field in read-only memory, while WK-11 (p08 331-338) lets any name make a
link and adds one to a count in the target's header ('making or ending a link is not a change of the target').
So making the link writes into read-only memory. No rule refuses it, places such an object elsewhere, or says
which thread owns a static's object. No verdict (103 covers `shares` only).

options:
[{'letter': 'A', 'summary': 'Allowed; such an object is kept in memory its count can be written in', 'code':
'cam.target = weakref ORIGIN', 'outcome': "Compiles and prints 0.0 0.0. A static's class object whose class a
`weakref` is made to anywhere in the program is placed in writable memory (the compiler knows those classes);
its links are counted as for any one-thread object, owned by the main thread, so making one from another
thread is refused as today (E7015). Nothing runs that would not run for any other link."}, {'letter': 'B',
'summary': 'Refused: a static never ends, so it needs no link', 'code': 'cam.target = weakref ORIGIN',
'outcome': 'A compile error: "`ORIGIN` is a static and never ends, so a `weakref` link to it is never needed;
read it by its name". The camera must follow an object made in `main` instead.'}]

recommendation:
{'letter': 'A', 'reason': "A `weakref` field that may hold either a live enemy or a fixed marker is a correct
program; A keeps one rule for every link's count, refuses only the other-thread case the language already
refuses, and costs nothing while running.", 'checklist': '1, 2, 3, 5, 7, 13 (no special case for statics in
the link rule)'}

