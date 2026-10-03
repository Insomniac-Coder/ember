#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, OPT-2, TYP-8, CG-C-1
#$ profiles: debug, release, shipping
#$ stdout: 5 2 3 0
#$ assert-c: !contains("ember_sum_bytes(")
# The sum reads each freshly rewritten byte. Removing the writes or reducing
# the original stream produces a different total and different caller data.

@noinline
@overflow(wrap)
fn increment_and_sum(mut xs: Array[u8]) -> u64:
    total: u64 = 0
    for i in 0..xs.len():
        xs[i] = xs[i] + 1
        total += xs[i] as u64
    return total

fn main():
    xs: Array[u8] = [1, 2, 255]
    total = increment_and_sum(xs)
    println(total, xs[0], xs[1], xs[2])
