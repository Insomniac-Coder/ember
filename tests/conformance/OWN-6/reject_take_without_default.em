#$ test: compile-fail
#$ rules: OWN-6

import std.mem

struct NoDefault:
    values: Array[i32]

fn main():
    value = NoDefault(Array[i32]())
    old = mem.take(value) #$ error[E2040]: `NoDefault` does not implement `std.core.Default`, which `T` requires
    println(old.values.len())
