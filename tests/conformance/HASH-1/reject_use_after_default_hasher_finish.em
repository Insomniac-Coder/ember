#$ test: compile-fail
#$ rules: HASH-1, HASH-2, OWN-3

from std.collections import DefaultHasher

fn main():
    hasher = DefaultHasher.new()
    _hash = hasher.finish()
    hasher.write_u8(1) #$ error[E3050]: `hasher` is borrowed after it has been moved out of
