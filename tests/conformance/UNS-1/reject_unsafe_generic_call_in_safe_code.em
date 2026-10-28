#$ test: compile-fail
#$ rules: UNS-1, TIER-1
#$ profiles: debug
#$ error[E3100]: `same(42)` (line 10) calls an `unsafe fn`, whose callers must make true what its `@safety` says

unsafe fn same[T](x: T) -> T:  # SAFETY: this test sets up what the operation requires
    return x

fn main():
    println(same(42))
