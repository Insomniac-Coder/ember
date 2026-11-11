#$ test: compile-pass
#$ rules: LNT-3
#$ warning[L1001]: `unused` is never read
# `[LNT-3]` — `L1001` is reported by `ember build` and `ember check`, not only `ember lint`.

fn main():
    unused: i32 = 3
    println(1)
