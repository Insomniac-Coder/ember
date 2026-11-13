#$ test: run-pass
#$ rules: UNS-10a
#$ stdout: 3
# `[UNS-10a]` — unsafe code may break the aliasing rules inside an abstraction built on `UnsafeCell`:
# `Hits.bump` changes the count through a read-only `this`, writing through the cell's raw pointer
# inside `unsafe:`. It lets no view of the count escape into safe code: `bump` and `count` give and
# take numbers only, so safe code sees no conflicting view.

from std.mem import UnsafeCell

struct Hits:
    count: UnsafeCell[int]

    fn bump():
        # SAFETY(aliasing): `Hits` gives out no view of the count, so nothing else touches it here
        unsafe:
            p = this.count.get()
            write(p, 0, read(p, 0) + 1)

    fn count() -> int:
        # SAFETY(aliasing): as in `bump`, no view of the count exists outside these two methods
        unsafe:
            return read(this.count.get(), 0)

fn main():
    hits = Hits(count=UnsafeCell(0))
    hits.bump()
    hits.bump()
    hits.bump()
    println(hits.count())
