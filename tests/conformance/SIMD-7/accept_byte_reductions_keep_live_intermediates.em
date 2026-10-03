#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, OPT-2, TYP-8, CG-C-1
#$ profiles: debug, release, shipping
#$ stdout: 260 4 0 7
#$ assert-c: !contains("ember_sum_bytes(")
# The last byte survives the loop. A helper that only updates the total would
# leave its old value behind. An empty loop must preserve that old value too.

@noinline
@overflow(wrap)
fn sum_and_last(xs: Span[u8]) -> (u64, u8):
    total: u64 = 0
    last: u8 = 7
    for x in xs:
        last = x
        total += last as u64
    return (total, last)

fn main():
    xs: Array[u8] = [1, 255, 4]
    (total, last) = sum_and_last(xs[..])
    (empty_total, empty_last) = sum_and_last(xs[0..0])
    println(total, last, empty_total, empty_last)
