#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, OPT-2
#$ profiles: debug, release, shipping
#$ stdout: 1 3 6
#$ assert-c: !contains("ember_sum_bytes(")
# Reading the running total each turn is observable, so replacing the whole
# loop with a horizontal reduction would lose required prefix results.

fn main():
    xs: Array[u8] = [1, 2, 3]
    out: Array[int] = []
    total = 0
    for x in xs:
        total += x as int
        out.push(total)
    println(out[0], out[1], out[2])
