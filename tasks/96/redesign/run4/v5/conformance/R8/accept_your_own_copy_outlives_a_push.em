#$ test: run-pass
#$ rules: BRW-16, LT-1
#$ stdout: ann 3
## design rules: R8, R6
## R8 (b): the way out E3060's message gives: a `String` of your own is not a view, so the push does
## not end it. AN-26, point 6.

fn first(xs: Array[String]) -> StrView:
    return xs[0]

fn main():
    names = [String("ann"), String("bob")]
    n = String(first(names))
    names.push(String("cy"))
    println(n, names.len())
