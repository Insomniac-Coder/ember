#$ test: compile-fail
#$ rules: OWN-6, OWN-3

import std.mem

struct Owned:
    values: Array[i32]

fn main():
    value = Owned(Array[i32]())
    mem.drop(value)
    println(value.values.len()) #$ error[E3040]: `value` has been moved out of
