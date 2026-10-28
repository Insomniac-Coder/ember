#$ test: compile-fail
#$ rules: UNS-1, TIER-1
#$ profiles: debug
#$ error[E3100]: `increase(41)` (line 10) calls an `unsafe fn`, whose callers must make true what its `@safety` says

unsafe fn increase(x: i32) -> i32:  # SAFETY: this test sets up what the operation requires
    return x + 1

fn main():
    println(increase(41))
