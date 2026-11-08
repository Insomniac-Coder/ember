# K03-task-ends-shares-owners

Question: May a scoped task end shares owners, whose count has no atomics?
Severity: high; impact rank 3
Members: J7-04; primary J7-04
Why together: Single question.
Critic's notes: Corrupts a one-thread count (memory safety). A is THR-11's existing weakref sentence applied to owners.

Status: answered by the owner, verdict 166 (tasks/96/redesign/JOB.md).

## Finding J7-04
Rules: THR-11, THR-2, RC-1, OBJ-3, THR-13; file tasks/spec-0.9.9/parts/p11-concurrency.md lines 405-406 (and 99-104); kind inconsistency; severity high
Quote from Hardened_11:
A task makes, ends / or opens a `weakref` link only to a `thread_shared` target, and makes no `shares` owner
(`[THR-2]`).

The finder's question:
May a scoped task end a `shares` owner, or end an object that `weakref` links may reach? Both change the
object's header with plain arithmetic while the parent may be counting the same header. Does this compile?

Program:
```
import std.thread

class Enemy:
    hp: int = 10

class Level:
    enemies: Array[shares mut Enemy] = []

fn main():
    boss = shares mut Enemy()
    level = Level(enemies=[shares mut boss])
    with scope = thread.scope():
        scope.spawn(owns fn(): level.enemies.clear())   # task: plain -1 on boss's count
        hud_target = shares mut boss                    # parent: plain +1 on the same count, at once
        println(hud_target.hp)
    println(boss.hp)
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.6)
Reason:
THR-11 restricts a task's making, ending and opening of `weakref` links, but only its making of `shares`
owners. THR-2's list of routes (functions any thread may run, unscoped threads, jobs, thread_shared values)
does not name scoped tasks, and RC-7 and WK-11 defer to §XI.3. Ending a `shares` owner subtracts from a
one-thread count with a plain subtract (RC-4's first sentence). E7007 compares places, and no rule treats an
owner count as one. Searched THR-2, THR-11, THR-13's bullets, THR-21, RC-1, RC-4, RC-7 (p08 132), WK-11 (p08
338-339), OBJ-3, the E7007/E7015 rows, and JOB.md verdicts 77, 93, 94, 104 and 114.

options:
[{'letter': 'A', 'summary': 'A task neither makes nor ends a one-thread owner, as for weakref links', 'code':
'scope.spawn(owns fn(): level.enemies.clear())\n# E7015: "`level.enemies` holds `shares mut Enemy` owners,
whose count is kept without atomics, and\n#  the task on line 17 ends them on another thread; clear them after
the scope: move line 17 below\n#  the `with` block"\n# the same for a task that ends an object `weakref` links
on this thread point at', 'outcome': 'Does not compile. With `level.enemies.clear()` moved below the `with`
block it compiles and prints 10.'}, {'letter': 'B', 'summary': "Allow it unless the parent or another task
touches the same type's counts", 'code': '# a task that makes or ends an owner of an Enemy counts as changing
every Enemy it may reach;\n# E7007 when the parent or another task may make or end an Enemy owner before the
join:\nhud.target = Some(shares mut boss)\n# E7007: "the task on line 17 ends `Enemy` owners, and line 18
makes one before the join ..."', 'outcome': 'This program does not compile (line 18 clashes). Without line 18,
it compiles and prints 10.'}]

recommendation:
{'letter': 'A', 'reason': "It is the rule the same sentence already gives `weakref` links: touching a
one-thread header from a task is refused, and the fix moves one line. B accepts more programs, but needs a new
fact ('makes an owner of some T') followed through every function.", 'checklist': "3 (decided when compiling),
5 (one rule for every one-thread header: owners and links), 6 (consistent with the nearest rule, THR-11's
`weakref` sentence), 8 (a mistake is a refusal, not a corrupted count), 9 (the message's line compiles)"}

