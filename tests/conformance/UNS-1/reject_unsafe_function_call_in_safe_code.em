#$ test: compile-fail
#$ rules: UNS-1, TIER-1
#$ profiles: debug
#$ error[E3100]: calling an `unsafe fn` requires an `unsafe` block

unsafe fn increase(x: i32) -> i32:
    return x + 1

fn main():
    println(increase(41))
