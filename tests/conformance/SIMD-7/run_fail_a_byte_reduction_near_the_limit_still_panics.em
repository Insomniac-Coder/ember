#$ test: run-fail
#$ rules: SIMD-5, SIMD-7, OPT-2
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
#$ assert-c: contains("ember_sum_bytes(")
#$ assert-c: contains("ember_ck_add_i64(")
# The vector sum belongs only to the proved copy. A starting total outside
# its safe margin takes the original checked loop and reports its first add.

@noinline
fn sum(xs: Span[u8], start: int) -> int:
    total = start
    for x in xs:
        total += x as int
    return total

fn main():
    xs: Array[u8] = [1, 0, 0]
    println(sum(xs[..], 9223372036854775807))
