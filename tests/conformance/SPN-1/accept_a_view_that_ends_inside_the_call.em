#$ test: run-pass
#$ rules: SPN-1, BRW-2, LT-1
# The other half of the same rule: `first` takes a view and returns an `i32`,
# so nothing derived from the borrow outlives the call and `[BRW-2]` ends it at
# the closing bracket. A view that borrows correctly must still permit this, or
# passing an array to a function that reads it would be useless.

fn first(xs: Span[i32]) -> i32:
    return xs[0]

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    println(first(a))
    a.push(2)
    println(a[1])
#$ stdout: 1
#$ 2
