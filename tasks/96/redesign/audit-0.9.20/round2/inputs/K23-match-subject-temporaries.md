# K23-match-subject-temporaries

Question: Does a lock taken in a match subject stay held while the arms run?
Severity: medium; impact rank 23
Members: J3-07; primary J3-07
Why together: Single question.
Critic's notes: Servers meet this. The example's `match owns` may lose `owns` depending on K22's answer. The question (when the temporary ends) is unchanged either way.

## Finding J3-07
Rules: EXP-4, DRP-3, GRM-15, THR-19; file tasks/spec-0.9.9/parts/p06-expressions.md lines 20-23; kind inconsistency; severity medium
Quote from Hardened_11:
  use), and a view of it that would outlive it is refused (`[BRW-19]`, `E3063`). A temporary created
  while evaluating the **condition** of an `if`, `elif` or `while` is dropped before the chosen block
  runs, unless the condition names what it holds (`exists as`, a kind test, a pattern) and the name

The finder's question:
Is a temporary made in a `match` subject (a lock guard) dropped before the arms run, as an `if` condition's
is, or held through every arm?

Program:
```
from std.sync import Mutex

static JOBS: Mutex[Array[Job]] = Mutex([])

fn run_next():
    match owns JOBS.lock().pop():
        Some(job):
            job.run()
        None:
            pass

# Job.run() may submit follow-up work, which takes JOBS.lock() again
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
EXP-4 (p06:16-25) gives exact lifetimes to the temporaries of an expression statement, an `if`/`elif`/`while`
condition, a `with` binding and a `for` iterable. It names no rule for a `match` subject. A `match` statement
is not an expression statement, so the only rule left is DRP-3 (p07:848), "Temporaries drop at the end of the
statement that created them". That keeps a lock guard from the subject alive through every arm. So `if
JOBS.lock().is_empty():` releases the lock before its block, while `match owns JOBS.lock().pop():` holds it
while an arm runs, and no reason is given for the difference. GRM-15 (p03:393-394) speaks of the subject's
ownership only, not of its temporaries.

options:
[{'letter': 'A', 'summary': "Like an `if` condition: a `match` subject's temporaries end before the chosen arm
runs, unless that arm's names point into them.", 'code': '    match owns JOBS.lock().pop():               #
the guard ends here; `job` is the popped value\n        Some(job):\n            println(job.id, pending())',
'outcome': "Compiles and prints `1 0`. The lock is released before the arm, so `pending()` takes it freely. An
arm whose names view the guard's contents keeps the guard to the end of that arm only."}, {'letter': 'B',
'summary': "As DRP-3 gives it today: a `match` subject's temporaries last until the whole `match` ends.",
'code': '    job = JOBS.lock().pop()                     # needed: take the job out on its own line first\n
match owns job:\n        Some(job):\n            println(job.id, pending())', 'outcome': 'The program as
written is refused when compiling, because `pending()` takes a lock this thread already holds (a lock taken
twice is refused, never left to hang). It compiles once the pop is moved to its own line, as above.'}]

recommendation:
{'letter': 'A', 'reason': 'One rule for every header that tests a value. A lock is held only as long as
something still looks into it, as `if`, `elif` and `while` already do.', 'checklist': "1 (the lock is held for
the pop, as the program means); 2 (accepts the correct program); 5 and 6 (the nearest rule, EXP-4's condition
rule, reused with no exception); 7 (general, for every temporary, not only locks)."}

