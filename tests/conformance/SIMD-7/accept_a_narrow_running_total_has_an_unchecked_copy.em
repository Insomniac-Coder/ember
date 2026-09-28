#$ test: run-pass
#$ rules: SIMD-7, OPT-2
#$ profiles: debug, release, shipping
#$ stdout: 4950
#$ assert-c: contains("<= 2147483648ULL)")
#$ assert-c: contains("<= 4611686018427387903LL)")
# A running total of `i32` elements widened into an `int`: at most 2^31
# iterations and a total starting within 2^62 - 1 of zero cannot overflow in
# any grouping, so a copy with no check runs when the entry test says so.

fn main():
    xs: Array[i32] = []
    for i in 0..100:
        xs.push(i as i32)
    total = 0
    for i in 0..100:
        total = total + (xs[i] as int)
    println(total)
