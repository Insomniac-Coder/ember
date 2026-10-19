#$ test: run-pass
#$ rules: UNS-1, TIER-1
#$ profiles: debug, development, release
#$ stdout: 42

unsafe fn same[T](x: T) -> T:  # SAFETY: this test sets up what the operation requires
    return x

fn main():
    unsafe:  # SAFETY: this test sets up what the operation requires
        println(same(42))
