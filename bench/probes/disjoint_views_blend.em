# D-557 probe: `[DSJ-3]`'s `blend` over views `assert_disjoint` proved apart, whose loop runs in
# a `restrict` function for every C compiler. Input: `<length> <rounds>`.
from std.mem import assert_disjoint

@noinline
fn blend(mut dst: MutSpan[f32], src: Span[f32]) -> int:
    match assert_disjoint(dst, src):
        Ok((d, s)):
            for i in 0..d.len():
                d[i] = d[i] * 0.5 + s[i] * 0.5
            return 1
        Err((d, s)):
            for i in 0..d.len():
                d[i] = (d[i] + s[i]) * 0.5
            return 0

fn main():
    line = input()
    parts = line.split(" ")
    n: int = parts.next().unwrap().parse[int]().unwrap()
    rounds: int = parts.next().unwrap().parse[int]().unwrap()
    a: Array[f32] = []
    b: Array[f32] = []
    for i in 0..n:
        a.push((i % 7) as f32)
        b.push((i % 5) as f32)
    apart = 0
    for _ in 0..rounds:
        apart += blend(a, b)
    total: f64 = 0.0
    for x in a:
        total += x as f64
    println(apart, total)
