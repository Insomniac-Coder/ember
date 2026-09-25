"""`std.math.det` gives the bits its model gives (`[DET-4]`, ADR-054).

Builds one program over a fixed set of arguments (random ones from a fixed
seed, and the hard cases: zeros, subnormals, overflow and underflow edges,
multiples of pi/2, huge arguments) and compares every result of every
function, `f64` and `f32`, bit for bit with `tools/det_model.py`.

    python tools/check_det.py            # needs a built compiler and a C compiler
"""

import math
import os
import random
import struct
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))
import det_model as m  # noqa: E402


def ember():
    candidates = [ROOT / "target" / p / n for p in ("release", "debug") for n in ("ember.exe", "ember")]
    built = [exe for exe in candidates if exe.exists()]
    if not built:
        sys.exit("build the compiler first: cargo build")
    return max(built, key=lambda exe: exe.stat().st_mtime)


def arguments():
    rng = random.Random(20260926)
    xs = []
    xs += [rng.uniform(-1, 1) for _ in range(200)]
    xs += [rng.uniform(-1e3, 1e3) for _ in range(200)]
    xs += [rng.uniform(-2e6, 2e6) for _ in range(100)]
    xs += [math.ldexp(rng.uniform(-1, 1), rng.randint(-1074, 1023)) for _ in range(300)]
    xs += [rng.uniform(-745, 710) for _ in range(100)]
    xs += [k * 1.5707963267948966 for k in range(-20, 21)]
    xs += [0.0, -0.0, 1.0, -1.0, 0.5, 2.0, 1e-310, 5e-324, 1e308, -1e308, 709.78, -745.13, 1e300,
           6381956970095103.0 * 2.0 ** 797]
    xs = [x for x in xs if math.isfinite(x)]
    ys = [rng.uniform(-5, 5) for _ in xs]
    return xs, ys


def f64_program(xs, ys):
    return "\n".join([
        "import math.det",
        "",
        "fn main():",
        "    xs: Array[f64] = [" + ", ".join(repr(x) for x in xs) + "]",
        "    ys: Array[f64] = [" + ", ".join(repr(y) for y in ys) + "]",
        "    for i in range(len(xs)):",
        "        x = xs[i]",
        "        y = ys[i]",
        "        println(det.sin(x), det.cos(x), det.tan(x), det.exp(x), det.log(abs(x)), det.atan2(x, y), "
        "det.pow(abs(x), y), det.pow(y, x.floor()), det.sqrt(abs(x)))",
        "        s: f32 = y as f32",
        "        println(det.sin(s), det.cos(s), det.tan(s), det.exp(s), det.log(abs(s)), det.atan2(s, 0.75), "
        "det.pow(abs(s), 1.25), det.sqrt(abs(s)))",
        "",
    ])


def same(a, b):
    if math.isnan(a) and math.isnan(b):
        return True
    return struct.pack("<d", a) == struct.pack("<d", b)


def main():
    xs, ys = arguments()
    with tempfile.TemporaryDirectory() as work:
        source = Path(work) / "det_check.em"
        source.write_text(f64_program(xs, ys), encoding="utf-8", newline="\n")
        result = subprocess.run([str(ember()), "run", str(source), "--out-dir", str(Path(work) / "out")],
                                capture_output=True, text=True, cwd=ROOT)
    if result.returncode != 0:
        print(result.stdout[-2000:] + result.stderr[-4000:])
        return 1
    lines = result.stdout.strip().split("\n")
    if len(lines) != 2 * len(xs):
        print(f"expected {2 * len(xs)} lines, found {len(lines)}")
        return 1
    bad = 0
    for i, (x, y) in enumerate(zip(xs, ys)):
        s = m.f32(y)
        wanted = [
            [m.sin64(x), m.cos64(x), m.tan64(x), m.exp64(x), m.log64(abs(x)), m.atan2_64(x, y),
             m.pow64(abs(x), y), m.pow64(y, math.floor(x)), math.sqrt(abs(x))],
            [m.f32(m.sin64(s)), m.f32(m.cos64(s)), m.f32(m.tan64(s)), m.f32(m.exp64(s)), m.f32(m.log64(abs(s))),
             m.f32(m.atan2_64(s, 0.75)), m.f32(m.pow64(abs(s), 1.25)), m.f32(math.sqrt(abs(s)))],
        ]
        for row, want in enumerate(wanted):
            have = [float(v) for v in lines[2 * i + row].split()]
            if row == 1:
                have = [m.f32(v) for v in have]
            for w, h in zip(want, have):
                if not same(w, h):
                    bad += 1
                    if bad <= 10:
                        print(f"x={x!r} y={y!r}: std.math.det gives {h!r}, the model {w!r}")
    total = len(xs) * 17
    if bad:
        print(f"{bad} of {total} results differ from tools/det_model.py")
        return 1
    print(f"std.math.det matches its model: {total} results, bit for bit")
    return 0


if __name__ == "__main__":
    sys.exit(main())
