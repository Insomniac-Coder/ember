#$ test: compile-fail
#$ rules: HASH-1, HASH-2, OWN-3

from std.collections import DefaultHasher

fn main():
    hasher = DefaultHasher.new()
    _hash = hasher.finish()
    hasher.write_u8(1) #$ error[E3040]: `hasher` was handed over to `finish` on line 8 and is used on line 9; to keep both, hand over a copy of your own: `copy of hasher`
