#$ test: compile-fail
#$ rules: IFC-4
# `[IFC-4]` (ODR-049) — a default meets the associated type's bounds as a
# stated value does: `Blob` takes `Key = Blob`, which is not `Hash`.

from std.collections import Hash

interface Keyed:
    type Key: Hash = Self
    fn key(self) -> Key

struct Blob:
    bytes: int

extend Blob implements Keyed: #$ error[E2040]: `Blob`'s `Key` is `Blob`, which does not implement `std.collections.Hash`
    fn key(self) -> Blob:
        return self

fn main():
    println(Blob(1).key().bytes)
