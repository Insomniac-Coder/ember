# K22-opened-name-owns-call-result

Question: Does a name opened from pop()'s result own the popped value?
Severity: medium; impact rank 22
Members: J1-09; primary J1-09
Why together: Single question.
Critic's notes: B makes `exists as` over a temporary take it over with no word, while GRM-15 still needs `match owns` over the same temporary (see conflicts). The draft should show `match jobs.pop():` beside the `exists as` line, so that one rule covers for, match and exists as. Answer before K23 (J3-07), whose example spells `match owns JOBS.lock().pop()`.

## Finding J1-09
Rules: GRM-44, GRM-13, CTL-1, CTL-11, BRW-14; file tasks/spec-0.9.9/parts/p03-grammar.md lines 219-220, 271-273; kind undecided_case; severity medium
Quote from Hardened_11:
`import x as y`. The name is declared there with its words, as a pattern name is (`[GRM-13]`):
  plain `p` is a read-only view and never changes (`E3023`); `if ui exists as ref mut me:` may change

The finder's question:
When `exists as` opens a temporary `Option` (the result of `pop()` or `remove()`), may the opened name be
handed over (`done.push(owns job)`)? How long does the temporary live? `for` over a temporary hands each
element to the loop's name (CTL-1), but an opened name is only a read-only view and `opened_name` has no
`owns` word.

Program:
```
fn drain(ref mut jobs: Array[Job], ref mut done: Array[Job]):
    while jobs.pop() exists as job:
        job.run()
        done.push(owns job)     # does `job` own the popped Job, or view a temporary?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
How long the temporary lives is settled, so that part is refuted: EXP-4 (p06 20-23) says a temporary in a
`while` condition that names what it holds 'lives to the end of that block only'. Ownership is not settled.
GRM-44 (p03 271-272) and BRW-14 (p07 222-225) make a plain opened name a read-only view that keeps its place.
BRW-12 (p07 200-202) says 'A call whose result is a new value the callee made ... hands that value to the
name, which owns it'. CTL-1 (p06 132-133) hands a temporary given to `for` over with no word. `match` needs
`owns` before its scrutinee to take it (GRM-15 p03 392-394). GRM-15 lists no `owns` before an `exists as`
value, `opened_name`'s words (p03 220) have no `owns`, and `while Some(job) = jobs.pop():` is E0115 (GRM-19).
No rule or example opens a temporary (grep 'exists as': only links and map lookups). So whether
`done.push(owns job)` compiles is undecided, and the nearest rules disagree.

options:
[{'letter': 'a', 'summary': 'Write `owns` before the value, as `match owns` does', 'code': 'while owns
jobs.pop() exists as job:\n    job.run()\n    done.push(owns job)', 'outcome': 'Compiles and prints 2: `job`
owns each popped Job. Without `owns`, `job` is a read-only view and `done.push(owns job)` is refused with this
line as its help.'}, {'letter': 'b', 'summary': 'A value a call made is handed to the opened name with no
word, as with `=` and `for`', 'code': 'while jobs.pop() exists as job:\n    job.run()\n    done.push(owns
job)', 'outcome': 'Compiles as written and prints 2: `job` owns what the Option held. A place that is opened
(`if this.target exists as t:`, a map lookup, a link) stays a read-only view, as today.'}, {'letter': 'c',
'summary': 'An opened name is always a view; to keep the value, copy it or use match', 'code': 'while true:\n
match owns jobs.pop():\n        Some(job):\n            job.run()\n            done.push(owns job)\n
None:\n            break', 'outcome': '`while jobs.pop() exists as job:` with `done.push(owns job)` is refused
(the name only looks at the popped Job), with this `match owns` form, or `done.push(owns copy of job)`, as the
help.'}]

recommendation:
{'letter': 'b', 'reason': 'The language already gives a name the value a call made, with no word, at `=` (`j =
jobs.pop()`) and at `for` over a temporary. The opened name is the same thing, a name for a value nothing else
holds, so one rule covers all three.', 'checklist': "5, 6 (nearest rules: a name given a call's new value owns
it; `for` over a temporary), 2 (the queue drain compiles), 13 (no second spelling just for headers)"}

