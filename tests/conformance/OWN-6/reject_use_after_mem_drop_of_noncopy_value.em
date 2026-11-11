#$ test: compile-fail
#$ rules: OWN-6, OWN-3

import std.mem

struct Owned:
    values: Array[i32]

fn main():
    value = Owned(Array[i32]())
    mem.drop(value)
    println(value.values.len()) #$ error[E3040]: `value` was handed over on line 11 and is used on line 12; to keep both, hand over a copy of your own: `copy of value`
