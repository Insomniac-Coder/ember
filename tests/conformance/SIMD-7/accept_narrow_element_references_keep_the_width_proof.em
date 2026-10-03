#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, OPT-2, TXT-10, TXT-11
#$ profiles: debug, release, shipping
#$ stdout: 4950 1189
#$ assert-c: contains("<= 2147483648ULL)")
#$ assert-c: contains("<= 4611686018427387903LL)")
# An i32 element read through the for loop's reference has the same width
# as an indexed element. The entry proof removes each addition's check.

fn main():
    xs: Array[i32] = []
    for i in 0..100:
        xs.push(i as i32)
    total = 0
    for x in xs:
        total += x as int
    text = String.from("hé🌶")
    bytes_total = 0
    for b in text.as_bytes():
        bytes_total += b as int
    println(total, bytes_total)
