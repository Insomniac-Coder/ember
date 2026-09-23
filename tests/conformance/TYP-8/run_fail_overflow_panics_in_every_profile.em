#$ test: run-fail
#$ rules: TYP-8, PRF-1
#$ profiles: debug, release, shipping
#$ panics: integer overflow in `+`
# 0.9.9: integer overflow panics in every profile; `release` and `shipping` no
# longer wrap. Wrapping is `@overflow(wrap)` or the `wrapping_*` methods.

fn main():
    x: i32 = 2147483647
    y = x + 1
    println(y)
