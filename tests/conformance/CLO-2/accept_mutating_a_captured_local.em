#$ test: run-pass
#$ rules: CLO-1, CLO-2, BRW-1, TST-19
#$ stdout: 1
#$ stdout: 2

# Mutation captures the outer `counter` as `ref mut`, not as a fresh shadow
# local. The closure's direct call therefore receives its environment through
# the ordinary mutable-place path.

fn main():
    counter: i32 = 0
    fn increment_fn() -> i32:
        counter = counter + 1
        return counter
    increment = increment_fn
    println(increment())
    println(increment())
