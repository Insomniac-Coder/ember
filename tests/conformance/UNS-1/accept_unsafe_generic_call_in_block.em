#$ test: run-pass
#$ rules: UNS-1, TIER-1
#$ profiles: debug, release, shipping
#$ stdout: 42

unsafe fn same[T](x: T) -> T:
    return x

fn main():
    unsafe:
        println(same(42))
