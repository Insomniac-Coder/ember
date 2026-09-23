#$ test: run-fail
#$ rules: TYP-8
#$ profiles: debug, release, shipping
#$ panics: integer overflow
# D-192 — negating the minimum of a signed type overflows, and overflow panics
# in every profile (it was C's undefined `-INT64_MIN`).

fn negate(x: int) -> int:
    return -x

fn main():
    low = -9223372036854775807 - 1
    println(negate(low))
