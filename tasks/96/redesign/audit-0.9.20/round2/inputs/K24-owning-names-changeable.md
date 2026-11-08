# K24-owning-names-changeable

Question: May a name that owns its value, like for job in owns jobs, be changed?
Severity: medium; impact rank 24
Members: J4-03; primary J4-03
Why together: Single question.
Critic's notes: Moves the same line as K25 and K26 (which names are read-only views). Edit BRW-12, BRW-14, BRW-15, OWN-3, CTL-1, GRM-13 and GRM-15 together.

## Finding J4-03
Rules: BRW-14, BRW-15, OWN-3, GRM-13, CTL-1; file tasks/spec-0.9.9/parts/p07-ownership.md lines 28-33, 72-74, 222-228, 235-237; kind inconsistency; severity medium
Quote from Hardened_11:
A plain parameter, a plain loop,
  pattern or opened name, and a plain local that holds a view never change what they name or what it
  leads to

The finder's question:
With `for job in owns jobs:`, does `job.retries += 1` compile? Under `match owns e:`, may a branch's names be
changed? Is `name = String("unnamed")` on an `owns name: String` parameter E3023?

Program:
```
struct Job:
    name: String
    retries: int

fn retry_all(owns jobs: Array[Job]) -> Array[Job]:
    again: Array[Job] = []
    for job in owns jobs:
        job.retries += 1       # `job` owns this Job ([CTL-1]); [BRW-15]/[CTL-1]: "a plain loop name changes nothing", E3023?
        again.push(owns job)
    return again

fn label(owns name: String) -> String:
    if name.len() == 0:
        name = String("unnamed")   # [BRW-14]: `=` on a parameter is E3023, though `name` is the callee's own
    return name
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.65)
Reason:
Not settled. [GRM-13] limits its read-only rule to names "in a place it does not take over", and it gives
`match owns e`'s names their parts. [CTL-1] gives each element to the name of `for x in owns e`. [BRW-18]'s
own fix hands such a name on (`for u in owns upper: counts.insert(owns u, owns 0)`). But [BRW-14], [BRW-15],
[CTL-1]'s last sentence, [OWN-3] and VII.1 do not leave these names out: they make every loop, pattern and
parameter name read-only and never given a new value. Changes through an `owns` parameter appear in the spec
(spec line 3937, `child.parent = weakref this`), but `=` on one is E3023 by [BRW-14]'s words. No verdict
covers this.

options:
[{'letter': 'A', 'summary': 'A name that took its value over is its owner, as a local is: it may be changed
and given a new value.', 'code': 'for job in owns jobs:\n    job.retries += 1\n    again.push(owns
job)\n...\nfn label(owns name: String) -> String:\n    if name.len() == 0:\n        name = String("unnamed")
# the old empty String ends here\n    return name', 'outcome': "Compiles; prints 1, then unnamed. `kept = job`
inside the loop would hand `job` over, as `=` does for any owner. The read-only rules cover only names that
look at someone else's value."}, {'letter': 'B', 'summary': 'Such a name may be changed through, but never
given a new value with `=`.', 'code': 'fn label(owns name: String) -> String:\n    return String("unnamed") if
name.len() == 0 else name', 'outcome': '`job.retries += 1` compiles. `name = String("unnamed")` is refused
(E3023: "`name` is the string the caller handed over and stays that string for the whole call"), so `label`
must be written as above.'}, {'letter': 'C', 'summary': 'As the read-only rules read today: a loop or branch
name is read-only whatever its source.', 'code': 'fn retry_all(owns jobs: Array[Job]) -> Array[Job]:\n    for
ref mut job in jobs:\n        job.retries += 1\n    return jobs', 'outcome': '`job.retries += 1` under `for
job in owns jobs:` is refused (E3023), and `name = ...` is refused as in B. A loop or branch that owns its
names may only hand them on, and the list is changed in place with `for ref mut job in jobs:`.'}]

recommendation:
{'letter': 'A', 'reason': "A name that took its value over owns it, exactly as a local does. The rules for
`match owns e:`, and the hand-on of `for u in owns upper:`'s `u`, already say so. The read-only rules exist
for names that look at a value someone else owns.", 'checklist': '2 (accepts the correct loop and function), 5
(`owns` means yours for parameters, loops and branches alike), 6 (nearest rule: a local owner may be changed
and given a new value), 13 (no second kind of owner)'}

