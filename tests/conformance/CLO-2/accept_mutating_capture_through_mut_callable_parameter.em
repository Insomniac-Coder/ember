#$ test: run-pass
#$ rules: CLO-2, CLO-3, FN-1, TST-19
#$ stdout: 1
#$ stdout: 2

# The closure's mutable-capture capability is carried by the existing outer
# parameter mode: `mut f` receives a mutable callable place and may invoke it.

fn invoke(mut f: fn() -> i32) -> i32:
    return f()

fn main():
    counter = 0
    increment = fn() -> i32:
        counter = counter + 1
        return counter
    println(invoke(increment))
    println(invoke(increment))
