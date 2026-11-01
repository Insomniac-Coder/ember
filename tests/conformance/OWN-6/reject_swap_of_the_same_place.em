#$ test: compile-fail
#$ rules: OWN-6, BRW-1

import std.mem

fn main():
    value = 1
    mem.swap(ref mut value, ref mut value) #$ error[E3022]: `value` is already mutably borrowed
