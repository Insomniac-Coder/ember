#$ test: run-pass
#$ rules: UNS-1, TIER-1
#$ profiles: debug, development, release
#$ stdout: 42

@safety("this test sets up what the operation requires")
unsafe fn same[T](x: T) -> T:
    return x

fn main():
    unsafe:  # SAFETY: this test sets up what the operation requires
        println(same(42))
