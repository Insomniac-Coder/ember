#$ test: compile-fail
#$ rules: UNS-10
# `[UNS-10]` — an `UnsafeCell` is never `Copy`, even of an `int`: a struct holding one cannot derive
# `Copy` (`E2080`), and assigning one hands it over (the other test of this folder, `E3040`).

from std.mem import UnsafeCell

@derive(Copy)
struct Hits:
    count: UnsafeCell[int]    #$ error[E2080]: field `count` has type `std.mem.UnsafeCell[i64]`, which is not Copy

fn main():
    _h = Hits(count=UnsafeCell(1))
