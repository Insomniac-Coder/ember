#$ test: compile-fail
#$ rules: DRP-6, BRW-1, OWN-3
# `get` is a real borrow of the Box owner, not a detached raw pointer.

fn consume(owns value: owns i32):
    println(value)

fn main():
    boxed: owns i32 = owns 9
    payload: ref i32 = boxed
    consume(owns boxed)
    println(payload)  #$ error[E3060]: `boxed` was handed over, and a view of it is still used here
