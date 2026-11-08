# K73-det-ln-name

Question: Is the deterministic natural logarithm det.ln, like std.math's ln?
Severity: low; impact rank 73
Members: J6-10; primary J6-10
Why together: Single question.
Critic's notes: 

## Finding J6-10
Rules: DET-4, STD-21, STD-27; file tasks/spec-0.9.9/parts/p10-effects.md lines 266-270; kind inconsistency; severity low
Quote from Hardened_11:
* `[DET-4]` `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt` for

The finder's question:
Is the natural logarithm `det.log` or `det.ln`? Does `det.ln(x)` compile, and is `math.log(x)` anything?

Program:
```
import math.det
from std.math import ln

fn gain_db(p: f64) -> f64:
    return 10.0 * ln(p) / ln(10.0)

@deterministic
fn gain_db_det(p: f64) -> f64:
    return 10.0 * det.log(p) / det.log(10.0)   # same function, other name
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.75)
Reason:
DET-4 (p10 266) names det's natural logarithm `log`; STD-21 (p15 439) and STD-27 (p15 460, "`ln` is C's
`log`") name std.math's `ln`, and std.math has no `log`. std/src/math/det.em:109 has `pub fn log`,
std/src/math/mod.em:167 `pub fn ln`. Code moved into a @deterministic function must rename the call. No reason
is given and no verdict.

options:
[{'letter': 'A', 'summary': "det's names are std.math's: `det.ln`", 'code': 'return 10.0 * det.ln(power) /
det.ln(10.0)', 'outcome': 'Compiles; `det.log` is an error whose help is "write `det.ln`". Moving code between
the two modules changes only the module name.'}, {'letter': 'B', 'summary': 'Keep `det.log`', 'code': 'return
10.0 * det.log(power) / det.log(10.0)', 'outcome': 'Compiles as today; `det.ln` is E2072 (no such
function).'}]

recommendation:
{'letter': 'A', 'reason': "One function, one name in both modules, so det's functions are a subset of
std.math's by name.", 'checklist': "5, 11 (std.math's spelling reused), 16"}

