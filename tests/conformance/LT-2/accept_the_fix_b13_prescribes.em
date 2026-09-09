#$ test: run-pass
#$ rules: LT-2, DIA-7a
# B13's help is "pass the two views as separate parameters rather than
# bundling them", and this is that program. Each argument gets its own region
# through `[LT-1]`'s elision, `first` returns an `i32` so neither borrow
# outlives the call, and `short` is the owner's again on the next line.
#
# A help that does not compile is worse than no help, so the fix a diagnostic
# names is a test beside the diagnostic.

fn first(a: Span[i32], b: Span[i32]) -> i32:
    return a[0]

fn main():
    long: Array[i32] = Array[i32]()
    long.push(1)
    short: Array[i32] = Array[i32]()
    short.push(2)
    println(first(long, short))
    short.push(3)
    println(short[1])
#$ stdout: 1
#$ 3
