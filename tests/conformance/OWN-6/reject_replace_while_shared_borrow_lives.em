#$ test: compile-fail
#$ rules: OWN-6, BRW-1

import std.mem

struct Item:
    value: i32

fn main():
    value = Item(1)
    borrowed: ref Item = ref value
    old = mem.replace(value, Item(2)) #$ error[E3021]: `value` is borrowed here and mutably borrowed elsewhere
    println(borrowed.value)
    println(old.value)
