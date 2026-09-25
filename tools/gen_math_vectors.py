"""std.math's vector types (`[STD-28]`, ODR-043), written as Ember.

`Vec2/3/4`, `IVec2/3/4` and `UVec2/3/4` differ only in their size and their
component type, so this writes them: each struct with its constants and
methods, then three operator blocks (vector and vector, vector and scalar,
scalar and vector). The result is the part of `std/src/math.em` from the
line after `MARKER` to the matrices' heading.

    python tools/gen_math_vectors.py            # rewrite that part
    python tools/gen_math_vectors.py --check    # fail if it differs
"""

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MATH = ROOT / "std" / "src" / "math.em"
MARKER = "## (Written by `tools/gen_math_vectors.py`, to the matrices: edit the script, not this.)"
END = "## -- matrices ("

AXES = "xyzw"


def vec(name, elem, n, kind):
    """kind: 'float', 'int' (signed) or 'uint'."""
    comps = AXES[:n]
    zero = "0.0" if kind == "float" else "0"
    one = "1.0" if kind == "float" else "1"
    ctor = lambda parts: f"{name}({', '.join(parts)})"
    out = []
    what = {"float": "`f32`s", "int": "`i32`s", "uint": "`u32`s"}[kind]
    out.append(f"## {n} {what} (`[STD-28]`, ODR-043).")
    out.append("@derive(Copy)")
    out.append("@layout(c)")
    out.append(f"pub struct {name}:")
    for c in comps:
        out.append(f"    pub {c}: {elem}")
    out.append("")
    out.append(f"    pub const ZERO: {name} = {ctor([zero] * n)}")
    out.append(f"    pub const ONE: {name} = {ctor([one] * n)}")
    for i, c in enumerate(comps):
        parts = [one if j == i else zero for j in range(n)]
        out.append(f"    pub const {c.upper()}: {name} = {ctor(parts)}")
    out.append("")
    out.append("    ## Every component `v`.")
    out.append(f"    pub fn splat(v: {elem}) -> {name}:")
    out.append(f"        return {ctor(['v'] * n)}")
    out.append("")
    dot = " + ".join(f"self.{c} * o.{c}" for c in comps)
    out.append(f"    pub fn dot(self, o: {name}) -> {elem}:")
    out.append(f"        return {dot}")
    out.append("")
    out.append("    ## The smaller of each pair of components.")
    out.append(f"    pub fn min(self, o: {name}) -> {name}:")
    out.append(f"        return {ctor([f'min(self.{c}, o.{c})' for c in comps])}")
    out.append("")
    out.append("    ## The larger of each pair of components.")
    out.append(f"    pub fn max(self, o: {name}) -> {name}:")
    out.append(f"        return {ctor([f'max(self.{c}, o.{c})' for c in comps])}")
    out.append("")
    if kind != "uint":
        out.append(f"    pub fn abs(self) -> {name}:")
        out.append(f"        return {ctor([f'abs(self.{c})' for c in comps])}")
        out.append("")
    if kind == "float":
        out.append("    pub fn length_squared(self) -> f32:")
        out.append("        return self.dot(self)")
        out.append("")
        out.append("    pub fn length(self) -> f32:")
        out.append("        return self.length_squared().sqrt()")
        out.append("")
        out.append(f"    pub fn distance(self, o: {name}) -> f32:")
        out.append("        return (self - o).length()")
        out.append("")
        out.append(f"    pub fn distance_squared(self, o: {name}) -> f32:")
        out.append("        return (self - o).length_squared()")
        out.append("")
        out.append("    ## The same direction, of length one. A zero vector has no direction,")
        out.append("    ## and panics; `normalize_or_zero` gives it back.")
        out.append(f"    pub fn normalize(self) -> {name}:")
        out.append("        length = self.length()")
        out.append("        if length == 0.0:")
        out.append('            panic("normalize of a zero vector; use normalize_or_zero")')
        out.append("        return self / length")
        out.append("")
        out.append(f"    pub fn normalize_or_zero(self) -> {name}:")
        out.append("        length = self.length()")
        out.append("        if length == 0.0:")
        out.append(f"            return {name}.ZERO")
        out.append("        return self / length")
        out.append("")
        out.append("    ## `self * a + b` component by component, each rounded once, not")
        out.append("    ## twice (`[STD-3]`).")
        out.append(f"    pub fn mul_add(self, a: {name}, b: {name}) -> {name}:")
        out.append(f"        return {ctor([f'self.{c}.mul_add(a.{c}, b.{c})' for c in comps])}")
        out.append("")
        out.append("    ## The point a fraction `t` of the way from this one to `o`.")
        out.append(f"    pub fn lerp(self, o: {name}, t: f32) -> {name}:")
        out.append("        return self + (o - self) * t")
        out.append("")
        if n == 3:
            out.append("    ## The vector at right angles to both, by the right-hand rule.")
            out.append("    pub fn cross(self, o: Vec3) -> Vec3:")
            out.append("        return Vec3(self.y * o.z - self.z * o.y, self.z * o.x - self.x * o.z, self.x * o.y - self.y * o.x)")
            out.append("")
    prefix = {"float": "Vec", "int": "IVec", "uint": "UVec"}[kind]
    if n < 4:
        bigger = f"{prefix}{n + 1}"
        nxt = AXES[n]
        out.append(f"    ## This vector with a {nxt} component.")
        out.append(f"    pub fn extend(self, {nxt}: {elem}) -> {bigger}:")
        out.append(f"        return {bigger}({', '.join(f'self.{c}' for c in comps)}, {nxt})")
        out.append("")
    if n > 2:
        smaller = f"{prefix}{n - 1}"
        out.append("    ## This vector without its last component.")
        out.append(f"    pub fn truncate(self) -> {smaller}:")
        out.append(f"        return {smaller}({', '.join(f'self.{c}' for c in comps[:-1])})")
        out.append("")
    # Operators between two vectors.
    if kind == "float":
        ops = [("Add", "add", "+"), ("Sub", "sub", "-"), ("Mul", "mul", "*"), ("Div", "div", "/")]
    else:
        ops = [("Add", "add", "+"), ("Sub", "sub", "-"), ("Mul", "mul", "*"), ("FloorDiv", "floordiv", "//"), ("Rem", "rem", "%")]
    interfaces = [op[0] for op in ops] + (["Neg"] if kind != "uint" else [])
    out.append(f"extend {name} implements {', '.join(interfaces)}:")
    out.append(f"    type Output = {name}")
    for _, method, sym in ops:
        out.append("")
        out.append(f"    fn {method}(self, o: {name}) -> {name}:")
        out.append(f"        return {ctor([f'self.{c} {sym} o.{c}' for c in comps])}")
    if kind != "uint":
        out.append("")
        out.append(f"    fn neg(self) -> {name}:")
        out.append(f"        return {ctor([f'-self.{c}' for c in comps])}")
    out.append("")
    # A scalar on the right.
    out.append(f"extend {name} implements {', '.join(f'{op[0]}[{elem}]' for op in ops)}:")
    out.append(f"    type Output = {name}")
    for _, method, sym in ops:
        out.append("")
        out.append(f"    fn {method}(self, s: {elem}) -> {name}:")
        out.append(f"        return {ctor([f'self.{c} {sym} s' for c in comps])}")
    out.append("")
    # A scalar on the left.
    out.append(f"extend {elem} implements {', '.join(f'{op[0]}[{name}]' for op in ops)}:")
    out.append(f"    type Output = {name}")
    for _, method, sym in ops:
        out.append("")
        out.append(f"    fn {method}(self, v: {name}) -> {name}:")
        out.append(f"        return {ctor([f'self {sym} v.{c}' for c in comps])}")
    out.append("")
    return "\n".join(out)


def all_vectors():
    parts = []
    for n in (2, 3, 4):
        parts.append(vec(f"Vec{n}", "f32", n, "float"))
    for n in (2, 3, 4):
        parts.append(vec(f"IVec{n}", "i32", n, "int"))
    for n in (2, 3, 4):
        parts.append(vec(f"UVec{n}", "u32", n, "uint"))
    return "\n".join(parts)



def rendered(source):
    start = source.index(MARKER) + len(MARKER) + 1
    end = source.index(END)
    return source[:start] + "\n" + all_vectors() + "\n\n" + source[end:]


def main():
    source = MATH.read_text(encoding="utf-8")
    wanted = rendered(source)
    if "--check" in sys.argv[1:]:
        if wanted != source:
            print("std/src/math.em's vectors differ from tools/gen_math_vectors.py; run it")
            return 1
        print("std/src/math.em's vectors are as generated")
        return 0
    MATH.write_text(wanted, encoding="utf-8", newline="\n")
    print("wrote std/src/math.em's vectors")
    return 0


if __name__ == "__main__":
    sys.exit(main())
