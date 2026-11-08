# K55-nondet-math-functions

Question: Which math functions count as nondeterministic in @deterministic code?
Severity: medium; impact rank 55
Members: J6-08; primary J6-08
Why together: Single question (J6-09 is already merged into it).
Critic's notes: 

## Finding J6-08
Rules: DET-2, DET-4, STD-27, STD-21, STD-28; file tasks/spec-0.9.9/parts/p15-stdlib.md lines 438-446, 459-462; kind undecided_case; severity medium
Quote from Hardened_11:
`nearbyint`, `round_half_away` its `round`, `mul_add` its `fma`); the transcendental ones are
  `[DET-2]`'s platform functions.

The finder's question:
Which of std.math's functions and methods are 'transcendental', and so Nondet? Does `@deterministic` code that
calls `v.normalize()` (sqrt), `math.hypot`, `math.cbrt`, `math.rsqrt` or `Quat.slerp` compile?

Program:
```
from std.math import Vec3

@deterministic
fn steer(vel: Vec3, target: Vec3) -> Vec3:
    return (target - vel).normalize() * 4.0   # sqrt inside: Nondet or not?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
DET-2 (p10 253-255) makes 'a transcendental function from the platform library' Nondet but never lists them,
and STD-27 (p15 459-462) repeats the word. Mathematically sqrt, cbrt, hypot and rsqrt are not transcendental,
yet C libraries round cbrt and hypot differently, and sqrt is exact on every machine. The SIMD `_approx`
operations (SIMD-4, spec 5953-5955) differ between CPU makers but are in neither of DET-2's float entries. The
std.math vector and quaternion methods (STD-28) call these with no stated effect. J6-09 is the same question
(which float operations are Nondet) and is merged here. No verdict.

options:
[{'letter': 'A', 'summary': 'By the result: whatever may differ between machines is Nondet', 'code': 'boost =
cbrt(speed)\nv.rsqrt_approx()', 'outcome': '`steer` is E4070 at the `cbrt` line and `inv_lengths` at
`rsqrt_approx`; `normalize` is allowed. The list in DET-2: every std.math function except those exact on every
machine (`sqrt`, `abs`, `floor`, `ceil`, `trunc`, `fract`, `round`, `round_half_away`, `copysign`,
`mul_add`/`fma`, `rsqrt` as `1.0 / sqrt(x)`, and `lerp`, `smoothstep`, which are `+` and `*`), and every
`_approx` operation. So vector `length`, `normalize`, `cross`, matrix products and inverses are allowed;
`slerp`, `from_axis_angle`, `perspective_rh` (sin, cos, tan) are not, unless built on std.math.det.'},
{'letter': 'B', 'summary': 'By the word: only the transcendental functions are Nondet', 'code': 'boost =
cbrt(speed)\nv.rsqrt_approx()', 'outcome': 'Both functions compile. Only `sin` through `pow` (the
trigonometric, hyperbolic, exponential and logarithm functions) are Nondet; `cbrt`, `hypot` and the `_approx`
operations are not, though their results may differ between machines.'}]

recommendation:
{'letter': 'A', 'reason': '`@deterministic` promises the same bits on every machine; A keeps that promise with
one test (does the result differ between machines?) and still allows the exact operations games use most.',
'checklist': "1, 2, 5, 6 (Nondet's own definition in the effect table), 7, 8 (a forgotten case is refused, not
a desync)"}

