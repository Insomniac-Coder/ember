#$ test: run-pass
#$ rules: HEAP-1, DRP-6, TYP-23
# An explicit Box payload type is authoritative. Literal fallback typing must
# not replace the chosen instantiation.

fn main():
    boxed: Box[i64] = Box[i64](41)
    println(boxed.get())
#$ stdout: 41

