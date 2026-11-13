#$ test: compile-fail
#$ rules: UNS-10a
# `[UNS-10a]` — `UnsafeCell` suspends no rule globally: the checks of Part VIII apply to a class that
# holds one, so its `drop` storing `this` where it outlives the call is still `E3016` (`[OBJ-5]`).

from std.mem import UnsafeCell

class Counter:
    hits: UnsafeCell[int]

    fn init():
        this.hits = UnsafeCell(0)

    fn drop():
        kept: Array[Counter] = Array[Counter]()
        kept.push(owns this)    #$ error[E3016]: `drop` ends this `Counter`; an owner made from it would point at a freed object

fn main():
    _counter = Counter()
