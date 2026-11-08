# K54-nondet-contested-lock

Question: Which line makes a function nondeterministic when threads race for a lock?
Severity: medium; impact rank 54
Members: J6-07; primary J6-07
Why together: Single question.
Critic's notes: B must leave THR-21's reductions as they are.

## Finding J6-07
Rules: DET-2, DET-1, THR-21; file tasks/spec-0.9.9/parts/p10-effects.md lines 253-261; kind undecided_case; severity medium
Quote from Hardened_11:
monotonic clock, the system random source, thread and job completion order; reading padding bytes,

The finder's question:
Which operation carries `Nondet` for 'thread and job completion order'? In the example, which line, if any, is
E4070?

Program:
```
static ORDER_LOCK: Lock = Lock()
static mut ORDER: Array[int] guarded by ORDER_LOCK = []

fn push_one():
    with ORDER_LOCK:
        ORDER.push(owns 1)

fn push_two():
    with ORDER_LOCK:
        ORDER.push(owns 2)

@deterministic
fn race() -> Array[int]:
    a = thread.spawn(push_one)
    b = thread.spawn(push_two)
    a.join()
    b.join()
    with ORDER_LOCK:
        return copy of ORDER       # [1, 2] or [2, 1]
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.65)
Reason:
DET-2 (p10 253-262) lists Nondet's sources 'exactly' and includes 'thread and job completion order', which is
not an operation: it does not say whether spawn, join, taking a lock, a channel receive, try_lock or an atomic
read-modify-write carries it. It cannot be spawn or join in general, since THR-21's reduction of scoped tasks
is not Nondet (p10 260-261). Nothing elsewhere assigns it (every Nondet mention checked). Which line of a
@deterministic program is E4070 is undecided. No verdict (92 and 105 cover reductions only).

options:
[{'letter': 'A', 'summary': "Named operations carry it: a contested lock, a shared channel's receive,
try_lock, a used atomic result", 'code': 'with INPUT_LOCK:\n    INPUTS.push(owns player)', 'outcome': '`with
INPUT_LOCK:` in `receive` is Nondet because another thread may take the same lock; `gather_turn` is E4070 at
line 13, naming `receive` and the lock. Also Nondet: `recv`/`try_recv` on a channel more than one thread sends
to, `try_lock`, and an atomic read-modify-write whose result is used. Starting and joining a thread are
not.'}, {'letter': 'B', 'summary': 'Threads touching one place carry it, worked out from what each function
changes', 'code': 'b = thread.spawn(owns fn(): receive(2))', 'outcome': 'E4070 at line 13: "`receive` (started
on lines 12 and 13) adds to `INPUTS` from two threads at once, so the list\'s order depends on which runs
first". The rule: starting a thread or job whose function changes a place (under a lock, an Atomic, a channel)
that another running thread, or the starter before its join, also reads or changes is Nondet; a lock only one
thread takes is not, and reductions of scoped tasks stay as they are.'}]

recommendation:
{'letter': 'B', 'reason': 'B names the real cause (two threads touching one place) with one rule for locks,
atomics and channels, decided when compiling from what the compiler already works out about every function,
the way it already refuses two tasks changing one place; A also refuses a lock that only one thread ever
takes.', 'checklist': '1, 2, 3, 4, 5, 6 (the two-tasks-change-one-place rule), 7'}

