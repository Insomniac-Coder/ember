# K41-some-after-conversions

Question: Does a value become Some after any other conversion, or only after four?
Severity: medium; impact rank 41
Members: J2-05; primary J2-05
Why together: Single question.
Critic's notes: 

## Finding J2-05
Rules: TYP-5; file tasks/spec-0.9.9/parts/p04-types.md lines 100-102; kind inconsistency; severity medium
Quote from Hardened_11:
Each rule composes with the others only where stated (widening after range erasure, and rule 11
  after rules 1–4).

The finder's question:
Does `return x` compile in a function returning `Option[int]` when `x` is a loop name, i.e. a read-only view
(rule 7, then rule 11)? Does `y: Option[u64] = n` compile after `n = 5` (rule 12, then 11)? Does `best:
Option[Entity] = player` compile for a subclass (rule 8 upcast, then 11)?

Program:
```
fn first_heavy(weights: Span[int], limit: int) -> Option[int]:
    for w in weights:            # `w` is a read-only view (CTL-1)
        if w > limit:
            return w             # rule 7 then rule 11: not a stated composition
    return None

fn main():
    n = 5
    cap: Option[u64] = n         # rule 12 then 11: refused, though `c: u64 = n` is accepted
    println(first_heavy([3, 9, 4], 5), cap)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
TYP-5 (p04 100-101) says the rules compose 'only where stated (widening after range erasure, and rule 11 after
rules 1–4)'. So the Option wrap (rule 11) does not follow rule 8 (a value to `owns dyn I`), rule 10 (a
function to a callable type), rule 12 (a known number widened) or rule 7 (reading through a view). `slot:
Option[owns dyn Weapon] = owns Sword(...)` and `on_hit: Option[Handler] = log_hit` are refused while the same
lines without Option compile. No spec example writes either (grep `Option[owns dyn`, `Option[fn`), and no
verdict covers composition.

options:
[{'letter': 'A', 'summary': 'Wrapping in `Some` follows any one of the other conversions: whatever a plain `T`
place accepts, an `Option[T]` place accepts as `Some` of it. Still one level only.', 'code': 's.weapon = owns
Sword(edge=4)\ns.on_hit = log_hit\ncap: Option[u64] = n', 'outcome': 'Compiles; prints `hit for 8`, then
`Some(5)`.'}, {'letter': 'B', 'summary': 'As today: `Some` follows only the first four conversions; the other
lines write `Some(...)`.', 'code': 's.weapon = Some(owns Sword(edge=4))\ns.on_hit = Some(log_hit)\ncap:
Option[u64] = Some(n)', 'outcome': 'The unwrapped lines are E2020; with `Some(...)` written the program
compiles and prints the same as A.'}]

recommendation:
{'letter': 'A', 'reason': '`x: T = v` and `x: Option[T] = v` then accept the same values, with no list of
which conversion may come first; the one-level limit, the part that prevents surprises, stays.', 'checklist':
"2 (correct programs accepted), 5 (one rule for T and Option[T] places), 6 (no exception the rule's wording
does not need), 7 (general over every conversion)"}

