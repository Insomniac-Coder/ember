#$ test: compile-fail
#$ rules: UNS-1, TIER-1
#$ profiles: debug
#$ error[E3100]: `increase(41)` (line 11) calls an `unsafe fn`, whose callers must make true what its `@safety` says

@safety("this test sets up what the operation requires")
unsafe fn increase(x: i32) -> i32:
    return x + 1

fn main():
    println(increase(41))
