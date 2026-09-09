#$ test: run-pass
#$ rules: CELL-1, OWN-5
# "**`set` and `replace` MUST store the new value before dropping the old
# one.** A drop can run arbitrary user code that re-enters the same `Cell` …
# a drop-then-store implementation would leave the `Cell` observably
# uninitialised across that window, which is a read of uninitialised memory."
#
# This is the one place in the compiler where that order is right: `[OWN-5]`
# makes an ordinary assignment do the exact opposite, dropping before it
# stores. `set` is therefore lowered itself rather than as an assignment, and
# ADR-020's last paragraph is the note not to generalise either rule.
#
# The order cannot be seen in the output. Ember has no way for a destructor to
# reach back into the cell being written — that needs a static holding a cell,
# or a raw back-pointer — so the two implementations print the same thing, and
# this is checked against the emitted C instead. `assert-c` alone cannot do it
# either: its needle is one line, and this is a question about two. The
# `assert-c-order` annotation exists for exactly this and for `[OWN-5]`.
#
# The output half still matters and is asserted too: the old bag is dropped
# once, by `set`, and the new one at scope end.

struct Bag:
    pub v: Array[i32]

    fn drop(mut self):
        println(self.v.len())

fn make(n: i32) -> Bag:
    xs: Array[i32] = Array[i32]()
    i = 0
    while i < n:
        xs.push(i)
        i = i + 1
    return Bag(xs)

fn main():
    c: Cell[Bag] = Cell(make(1))
    c.set(make(2))
    println(99)
#$ stdout: 1
#$ 99
#$ 2
#$ assert-c-order: ".value = " then "em_Bag_drop(&"

# The needles are chosen not to depend on MIR local numbering: `.value = ` is
# the store and occurs once, and `em_Bag_drop(&` is the *call* — the prototype
# at the top of the file reads `em_Bag_drop(em_Bag* _1)` and would otherwise
# match first, which is the trap in anchoring on a name alone.
