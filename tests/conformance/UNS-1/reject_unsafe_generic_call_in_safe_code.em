#$ test: compile-fail
#$ rules: UNS-1, TIER-1
#$ profiles: debug
#$ error[E3100]: calling an `unsafe fn` requires an `unsafe` block

unsafe fn same[T](x: T) -> T:
    return x

fn main():
    println(same(42))
