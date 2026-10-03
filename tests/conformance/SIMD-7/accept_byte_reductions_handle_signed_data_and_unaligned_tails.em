#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, OPT-2, CG-C-1
#$ profiles: debug, release, shipping
#$ stdout: byte sums ok
#$ assert-c: contains("ember_sum_bytes(")
#$ assert-c: contains(", true)")
#$ assert-c: contains(", false)")
# All signed/unsigned byte values, every tail length, nonzero starting totals,
# and unaligned subviews. The independent answer is a closed-form series.

@noinline
fn totals(xs: Span[u8], ys: Span[i8]) -> (int, int):
    a = 9
    b = -17
    for x in xs:
        a += x as int
    for y in ys:
        b -= y as int
    return (a, b)

fn main():
    xs: Array[u8] = []
    ys: Array[i8] = []
    for i in 0..256:
        xs.push(i as u8)
        ys.push((i - 128) as i8)
    for start in 0..16:
        for end in start..257:
            (a, b) = totals(xs[start..end], ys[start..end])
            series = (end * (end - 1) - start * (start - 1)) // 2
            assert(a == 9 + series)
            assert(b == -17 - series + 128 * (end - start))
    println("byte sums ok")
