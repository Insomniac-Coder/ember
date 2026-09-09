#$ test: run-pass
#$ rules: OWN-7
# `[OWN-7]` for a derived `Copy`: `@derive(Copy)` duplicates bitwise on use,
# so `q = p` copies and `p` stays readable. Without the attribute this is
# `E3040` (checked by removing it), so the case also pins `[STR-3]`'s
# never-implicit claim.

@derive(Copy)
struct Pt:
    pub x: i32
    pub y: i32

fn main():
    p = Pt(1, 2)
    q = p
    println(p.x + q.x)
#$ stdout: 2
