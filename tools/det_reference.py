#!/usr/bin/env python3
"""`[DET-4]` — writes `tests/conformance/DET-4/accept_each_function_is_within_its_ulp_bound.em`.

Each `std.math.det` function is within one unit in the last place of the exact result (`atan2`
within 1.3, `sqrt` correctly rounded). The reference values are computed here with mpmath at 400
bits, never with the platform's libm, and written into the test as exact decimal literals of
doubles: for each case the exact result `y` as `hi = RN(y)` and `lo = RN(y - hi)` (a double-double),
and `ulp`, the spacing of the doubles at `y`. The test then checks `|(got - hi) - lo| <= bound * ulp`
(`got - hi` is exact: both are within a few ulps of each other).

Run:  python tools/det_reference.py          (writes the test)
Needs: pip install mpmath
"""
import math
import pathlib
import random

from mpmath import mp, mpf

mp.prec = 400

OUT = pathlib.Path(__file__).resolve().parent.parent / "tests/conformance/DET-4/accept_each_function_is_within_its_ulp_bound.em"

UNARY = {
    "sin": (mp.sin, [0.1, 0.5, 1.0, 2.0, 3.0, 10.0, 100.0, 12345.678, 1e6, -0.75, 1e-5, 0.7853981633974483, 1.5707963267948966, 3.141592653589793]),
    "cos": (mp.cos, [0.1, 0.5, 1.0, 2.0, 3.0, 10.0, 100.0, 12345.678, 1e6, -0.75, 1e-5, 0.7853981633974483, 1.5707963267948966, 3.141592653589793]),
    "tan": (mp.tan, [0.1, 0.5, 1.0, 2.0, 3.0, 10.0, 100.0, 12345.678, 1e6, -0.75, 1e-5, 0.7853981633974483, 1.5707963267948966]),
    "asin": (mp.asin, [-0.99, -0.5, 0.1, 0.3, 0.5, 0.7, 0.9, 0.999999]),
    "acos": (mp.acos, [-0.99, -0.5, 0.1, 0.3, 0.5, 0.7, 0.9, 0.999999]),
    "atan": (mp.atan, [-100.0, -1.0, 0.1, 0.5, 1.0, 3.0, 50.0, 1e5, 1e-5]),
    "sinh": (mp.sinh, [-5.0, -1.0, 0.001, 0.5, 1.0, 2.5, 10.0, 50.0]),
    "cosh": (mp.cosh, [-5.0, -1.0, 0.001, 0.5, 1.0, 2.5, 10.0, 50.0]),
    "tanh": (mp.tanh, [-5.0, -1.0, 0.001, 0.5, 1.0, 2.5, 10.0, 20.0]),
    "exp": (mp.exp, [-700.0, -20.0, -1.0, -0.001, 0.5, 1.0, 5.0, 20.0, 100.0, 700.0]),
    "exp2": (lambda x: mp.power(2, x), [-1000.0, -10.0, -0.5, 0.3, 1.0, 7.5, 100.0, 1000.0]),
    "ln": (mp.log, [1e-300, 0.001, 0.5, 0.9999, 1.0001, 2.0, 10.0, 12345.6789, 1e300]),
    "log2": (lambda x: mp.log(x, 2), [1e-300, 0.001, 0.5, 0.9999, 1.0001, 3.0, 10.0, 12345.6789, 1e300]),
    "log10": (mp.log10, [1e-300, 0.001, 0.5, 0.9999, 1.0001, 2.0, 12345.6789, 1e300]),
    "cbrt": (lambda x: mp.sign(x) * mp.cbrt(abs(x)), [0.001, 0.5, 2.0, 3.0, 27.0, 1000.0, 1e10, -8.0, 123456.789]),
    "sqrt": (mp.sqrt, [2.0, 3.0, 0.5, 1e-10, 12345.6789, 1e300]),
}

BINARY = {
    "pow": (lambda a, b: mp.power(a, b), [(2.0, 0.5), (10.0, -3.7), (1.5, 100.0), (0.9, 50.0), (7.0, 3.3), (123.456, 0.123), (2.0, 10.0), (0.001, 2.5)]),
    "atan2": (mp.atan2, [(1.0, 1.0), (1.0, -1.0), (-2.0, -3.0), (0.001, 5.0), (5.0, 0.001), (3.0, 4.0), (1e-5, 1.0)]),
    "hypot": (lambda a, b: mp.sqrt(mpf(a) ** 2 + mpf(b) ** 2), [(3.0, 4.0), (1e-5, 1.0), (1e200, 1e200), (0.1, 0.2), (12345.6, 789.01)]),
}

BOUND = {"atan2": 1.3, "sqrt": 0.5}


def lit(x: float) -> str:
    """A double as a decimal literal Ember reads back as the same double."""
    text = f"{x:.17g}"
    if "e" in text:
        mantissa, exponent = text.split("e")
        if "." not in mantissa:
            mantissa += ".0"
        sign = "-" if exponent.startswith("-") else "+"
        text = f"{mantissa}e{sign}{int(exponent.lstrip('+-'))}"
    elif "." not in text:
        text += ".0"
    assert float(text) == x, (text, x)
    return text


def reference(exact):
    hi = float(exact)
    lo = float(exact - mpf(hi))
    # the spacing of the doubles at the exact result (its own binade, not hi's)
    ulp = math.ldexp(1.0, mp.floor(mp.log(abs(exact), 2)) .__int__() - 52)
    return hi, lo, ulp


def main():
    lines = []
    count = 0
    for name, (function, inputs) in UNARY.items():
        for x in inputs:
            hi, lo, ulp = reference(function(mpf(x)))
            lines.append(f'    bad += check("{name}({lit(x)})", det.{name}({lit(x)}), {lit(hi)}, {lit(lo)}, {lit(ulp)}, {BOUND.get(name, 1.0)})')
            count += 1
    for name, (function, inputs) in BINARY.items():
        for a, b in inputs:
            hi, lo, ulp = reference(function(mpf(a), mpf(b)))
            lines.append(f'    bad += check("{name}({lit(a)}, {lit(b)})", det.{name}({lit(a)}, {lit(b)}), {lit(hi)}, {lit(lo)}, {lit(ulp)}, {BOUND.get(name, 1.0)})')
            count += 1
    body = "\n".join(lines)
    OUT.write_text(f'''#$ test: run-pass
#$ rules: DET-4, DET-2
#$ profiles: debug, development, release
#$ stdout: {count} cases, 0 outside their bound
# `[DET-4]` — each function of `std.math.det` is within one unit in the last place of the exact result,
# `atan2` within 1.3, and `sqrt` is correctly rounded (within half an ulp). A docstring's
# measurement is not evidence: the reference values here were computed with mpmath at 400 bits by
# `tools/det_reference.py`, which wrote this file. Each case is the exact result `y` as the double
# `hi` nearest it and `lo`, the double nearest `y - hi`, and `ulp`, the spacing of the doubles at `y`;
# `got - hi` is exact, so the error in ulps is `((got - hi) - lo) / ulp`.

import math.det

fn check(label: StrView, got: f64, hi: f64, lo: f64, ulp: f64, bound: f64) -> int:
    error = ((got - hi) - lo) / ulp
    if error < 0.0:
        error = -error
    if error <= bound:
        return 0
    println(f"outside its bound: {{label}} gave {{got}}, the exact result is {{hi}} + {{lo}} ({{error}} ulps)")
    return 1

fn main():
    bad = 0
{body}
    println(f"{count} cases, {{bad}} outside their bound")
''', encoding="utf-8", newline="\n")
    print(f"wrote {OUT} ({count} cases)")


if __name__ == "__main__":
    main()
