#$ test: run-pass
#$ rules: FN-1
#$ profiles: debug, release
#$ stdout: written
#$ stdout: 5
# D-709 — a `mut` argument is the caller's place: what the callee writes there is what the caller
# reads after the call. A `bool` local known to be `false` before the call is not known after it.

fn set(mut b: bool, mut n: int):
    b = true
    n = 5

fn main():
    x = false
    k = 0
    set(x, k)
    if x:
        println("written")
    else:
        println("lost")
    println(k)
