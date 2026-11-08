# K38-parallel-reductions

Question: Must a @parallel loop declare its totals, and how is a max reduction written?
Severity: medium; impact rank 38
Members: J7-10, J7-09; primary J7-10
Why together: J7-09's `best = max(best, e)` is the one max spelling everywhere only if J7-10 A makes loops follow the task rule. Under J7-10 B, `reduce=[best: max]` stays the loop spelling and J7-09 covers tasks only.
Critic's notes: J7-10 A withdraws `@parallel(reduce=…)`, which SIMD-5 names as the permission for a float reduction in a vectorised loop (see conflicts).

## Finding J7-10
Rules: PAR-2, PAR-3, THR-11, THR-21; file tasks/spec-0.9.9/parts/p11-concurrency.md lines 455-458, 461-462, 480-481; kind contradiction; severity medium
Quote from Hardened_11:
`return`, `yield`, or a `continue` naming an outer loop, and what it uses from outside follows the / rules of
a scoped task (`[THR-11]`, `[THR-2]`). In a loop over a range, for each place `P` written in / through the
loop's `ref mut` name, and writes nothing else but declared reductions (`[PAR-3]`); its

The finder's question:
Which writes may a `@parallel` body make besides its indexed elements: an undeclared `total += x` (a reduction
under the scoped-task rules it follows), an integer one, an `Atomic`, a `guarded by` field under its lock?
Does this compile?

Program:
```
from std.sync import Atomic

fn total_mass(masses: Span[float], hits: Atomic[int]) -> float:
    total = 0.0
    count = 0
    @parallel
    for i in 0..masses.len():
        total += masses[i]
        count += 1
        hits.fetch_add(1)
    return total
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.65)
Reason:
PAR-2 (p11 455-456) says the body 'follows the rules of a scoped task' (THR-21: an undeclared `total += e` is
a reduction, decimals included, per verdict 92). Yet in a range loop every written place must be indexed `i +
k` and be a MutSpan or SoA column (clause c, p11 456-461), and a by-element loop 'writes nothing else but
declared reductions' (p11 461-462). PAR-3 (p11 480-481) says a float reduction 'must be declared', which
implies an integer one need not. E7010's message offers an `Atomic` as a fix (p11 469-470), but an Atomic is a
written place with no index, which clause (c) refuses. Each of `count += 1`, `total += x` and an Atomic gets
two answers. decisions.md's PAR-3 row covers only overflow.

options:
[{'letter': 'A', 'summary': 'One rule with scoped tasks: an undeclared total is a reduction in loops too',
'code': '# the program above, unchanged: `total` and `count` are reductions, combined in chunk order;\n# an
`Atomic` and a `guarded by` field under its lock may be changed in the body too;\n# `@parallel(reduce=...)` is
no longer needed and goes', 'outcome': 'Compiles and prints 3, then 6.5, the same on every machine. A float
total may differ from the serial sum in the last digit, as the owner accepted for tasks.'}, {'letter': 'B',
'summary': 'A loop declares every total, whole numbers too; tasks keep undeclared ones', 'code':
'@parallel(reduce=[total: +, count: +])\nfor i in 0..masses.len():\n    total += masses[i]\n    count += 1\n#
without the declaration: E7010, "... declare it: write `@parallel(reduce=[total: +, count: +])`\n#  in place
of line 4"; an `Atomic` and a guarded field under its lock are allowed', 'outcome': "The program as written
does not compile; with the declaration it prints 3, then 6.5. A scoped task's `total += x` stays
undeclared."}]

recommendation:
{'letter': 'A', 'reason': "The rule already says a parallel body follows a scoped task's rules, and the owner
accepted undeclared decimal totals for tasks. One rule for the same line in both places, with nothing said
twice.", 'checklist': "5 (one rule wherever the same thing happens; 'same rule for every loop'), 2 (accepts
the correct loop), 3 (nothing added while running), 11 (one spelling of a reduction, so `reduce=` goes)"}

## Finding J7-09
Rules: THR-21, E7007; file tasks/spec-0.9.9/parts/p11-concurrency.md lines 408-409, 424-425; kind contradiction; severity medium
Quote from Hardened_11:
`max`, `min`, `push`, `extend`, `add`, `m[k] op= e`) on a local / A / plain assignment from a task
(`a.target.hp = 5`) and a task that reads the current value / (`if stats.total < 10: stats.total += x`) need a
lock (`E7007`).

The finder's question:
How is a `max` reduction written in a task, and does `best = max(best, t.len())` compile? It is a plain
assignment that reads the current value, which the same rule sends to E7007.

Program:
```
import std.thread

fn longest(lines: Array[String]) -> int:
    best = 0
    with scope = thread.scope():
        for t in lines:
            scope.spawn(owns fn() ref t: best = max(best, t.len()))
    return best
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
THR-21 lists `max` and `min` among task reductions (p11 409), but Ember has no `max=` operator. The only
ordinary spelling, `best = max(best, x)`, is both shapes that the same rule refuses (p11 424-425): a plain
assignment from a task, and a task that reads the current value. PAR-3's `reduce=[best: max]` exists only for
loops. A scoped task has no place to declare one. So a listed reduction has no spelling that compiles.
Searched THR-11, THR-21, PAR-3, the E7007 row, the glossary's 'reduction', grep for 'max=' and 'max(', and
JOB.md verdicts 77 and 92.

options:
[{'letter': 'A', 'summary': '`best = max(best, e)` is the max reduction', 'code': "scope.spawn(owns fn() ref
t: best = max(best, t.len()))\n# a reduction when the place appears only as max's (or min's) first
argument;\n# any other read of `best` in a task stays E7007", 'outcome': 'Compiles and prints 7. `best =
max(best, t.len()) + 1`, or `if best < 3:` in a task, is refused (E7007).'}, {'letter': 'B', 'summary': 'Take
max and min off the task list: a task uses a lock', 'code': 'from std.sync import Mutex\n# best = Mutex(0)\n#
scope.spawn(owns fn() ref t: keep_max(best, t.len()))\n# fn keep_max(best: Mutex[int], n: int):\n#     with g
= best.lock():\n#         g = max(g, n)', 'outcome': '`best = max(best, t.len())` in a task is E7007, whose
message gives the lock. The rewrite compiles and prints 7, with one lock per task.'}]

recommendation:
{'letter': 'A', 'reason': 'It makes the reduction the rule already promises writable in the words people
already write, at the cost of one counted task total, with no lock. If the @parallel question goes one rule
for tasks and loops, it is the one `max` spelling everywhere.', 'checklist': '2 (a correct program compiles),
3 (no run-time cost beyond the reduction), 10 (spelled as people write it, `max(best, x)`), 12 (adds the
missing spelling instead of leaving `max` listed but unwritable)'}

