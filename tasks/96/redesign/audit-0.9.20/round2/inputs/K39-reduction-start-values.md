# K39-reduction-start-values

Question: What does a task's private total start from, for -= on an unsigned field?
Severity: medium; impact rank 39
Members: J7-08; primary J7-08
Why together: Single question.
Critic's notes: Keeps verdict 105's promise. A also refuses mixed operators on one place (E7007).

## Finding J7-08
Rules: THR-21, PAR-3, TYP-8; file tasks/spec-0.9.9/parts/p11-concurrency.md lines 408-412, 415-417 (and 477-479); kind undecided_case; severity medium
Quote from Hardened_11:
`max`, `min`, `push`, `extend`, `add`, `m[k] op= e`) on a local, or on a field of an object the tasks / reach
(`scope.spawn(owns fn() copy of x: stats.total += x)`), a field reached through two `sharedref mut` / links to
one object included, is a reduction: each task adds into a private accumulator, and the

The finder's question:
What does a task's private accumulator (or a chunk's 'own copy' in PAR-3) start from, and how is it combined
for `-=` on an unsigned field, for `*=`, and for one task that mixes `+=` and `*=` on one place? What does the
program below print, or does it stop?

Program:
```
import std.thread

class Player:
    hp: u8 = 10

fn main():
    p = Player()
    with scope = thread.scope():
        scope.spawn(owns fn(): p.hp -= 1)
        scope.spawn(owns fn(): p.hp -= 2)
    println(p.hp)        # one by one: 7. An accumulator of type u8 starting at 0 stops at 0 - 1.
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.65)
Reason:
THR-21 (p11 411-412, 415-417) gives each task 'a private accumulator' with 'the place's type', combined at the
join in spawn order. It never says what the accumulator starts from, how `-=` and `*=` are combined, or what
happens when tasks (or one task) apply two different operators to one place. With a `u8` place and `-=`, an
accumulator that starts at 0 overflows on the first change, though the one-by-one result (7) fits. Verdict 105
('equal when no task's total overflows') does not say whether that total is -1 or 1. Tasks that add and
multiply one place give a result that depends on order, and no rule refuses that. PAR-3's 'each chunk has its
own copy' (p11 478) has the same hole. Searched THR-13, THR-21, PAR-3, PAR-5, the glossary's 'reduction', the
cost row (p10 338), and JOB.md verdicts 77, 92 and 105.

options:
[{'letter': 'A', 'summary': "Each total starts from the operator's neutral value; -= totals what is taken
away", 'code': "# the program above, unchanged\n# each task's total starts at 0 for + - | ^, 1 for *, all ones
for &, empty for push/extend/add;\n# for -= the task adds up what it takes away (1, 2), and the join takes
each from the place;\n# a place given two different operators by the tasks (or by one task) is E7007, which
names a lock", 'outcome': 'Compiles and prints 7 every time. A task whose own amount taken away passes 255
stops the program there, as the owner decided for task totals. `boss.hp += d` in one task and `boss.hp *= 2`
in another is refused (E7007).'}, {'letter': 'B', 'summary': "Each total starts at 0 and holds the signed
change, in the place's type", 'code': "# the program above, unchanged\n# each task's total is a u8 starting at
0; `-= 1` computes 0 - 1", 'outcome': 'Stops the program in the first task: "0 - 1 overflows a u8", though
adding one by one gives 7. The same program with `hp: int` prints 7.'}]

recommendation:
{'letter': 'A', 'reason': "It is the only reading that keeps THR-21's own promise (the one-by-one total
whenever no task's own total overflows) for every type and operator. Refusing mixed operators stops a result
that would depend on which task ran first.", 'checklist': '1 (prints what the program means, 7), 2 (refuses
only the order-dependent case), 5 (one rule for every operator and every integer type), 7 (general: tasks and
@parallel chunks alike)'}

