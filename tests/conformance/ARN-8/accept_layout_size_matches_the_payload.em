#$ test: run-pass
#$ rules: ARN-8, TST-23
# Alignment equality is asserted directly by the compiler type-layout unit
# test; this executable half pins the target-visible size equality.

fn main():
    println(size_of[i64]())
    println(size_of[MaybeUninit[i64]]())
#$ stdout: 8
#$ 8
