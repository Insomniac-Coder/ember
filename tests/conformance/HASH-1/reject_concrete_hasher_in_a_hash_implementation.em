#$ test: compile-fail
#$ rules: HASH-1, TYP-17, TYP-22

from std.collections import Hash, DefaultHasher

struct BadHash:
    value: i32

extend BadHash implements Hash: #$ error[E2040]: `BadHash.hash` does not match the signature required by `std.collections.Hash`
    fn hash(self, mut h: DefaultHasher):
        h.write_i32(self.value)
