#$ test: compile-fail
#$ rules: UNS-1, TIER-1
#$ profiles: debug
#$ error[E3100]: `same(42)` (line 11) calls an `unsafe fn`, whose callers must make true what its `@safety` says

@safety("this test sets up what the operation requires")
unsafe fn same[T](x: T) -> T:
    return x

fn main():
    println(same(42))
