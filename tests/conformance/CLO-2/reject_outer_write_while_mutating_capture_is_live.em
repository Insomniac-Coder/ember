#$ test: compile-fail
#$ rules: CLO-2, BRW-1, BRW-2, TST-19

# The environment owns the mutable borrow until the closure's last use; the
# outer place cannot be written in the intervening region.

fn main():
    counter: i32 = 0
    increment = fn() -> i32:
        counter = counter + 1
        return counter
    counter = 99 #$ error[E3021]: `counter` cannot be written while it is borrowed
    println(increment())
