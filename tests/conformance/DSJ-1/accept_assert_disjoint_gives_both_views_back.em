#$ test: run-pass
#$ rules: DSJ-1, DSJ-2, DSJ-3
#$ profiles: debug, release, shipping
#$ stdout: apart [1.0, 2.0, 3.0]
#$ overlap [1, 2] [2, 3]
#$ apart [1, 2] [3, 4]

# `[DSJ-1]` — `assert_disjoint` compares two views' address ranges and gives
# both back: in `Ok` when they are apart, unchanged in `Err` when they overlap,
# so the overlapping path can still use them. The spec's own `blend` (IX.8).

from std.mem import assert_disjoint

fn blend(mut dst: MutSpan[f32], src: Span[f32]) -> str:
    match assert_disjoint(dst, src):
        Ok((d, s)):
            for i in 0..d.len():
                d[i] = d[i] * 0.5 + s[i] * 0.5
            return "apart"
        Err((d, s)):
            for i in 0..d.len():
                d[i] = (d[i] + s[i]) * 0.5
            return "overlap"

fn main():
    a: Array[f32] = [2.0, 4.0, 6.0]
    b: Array[f32] = [0.0, 0.0, 0.0]
    println(blend(a, b), a)
    x: Array[int] = [1, 2, 3, 4]
    match assert_disjoint(x[0..2], x[1..3]):
        Ok((p, q)):
            println("apart", p, q)
        Err((p, q)):
            println("overlap", p, q)
    match assert_disjoint(x[0..2], x[2..4]):
        Ok((p, q)):
            println("apart", p, q)
        Err((p, q)):
            println("overlap", p, q)
