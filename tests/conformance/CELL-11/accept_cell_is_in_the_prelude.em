#$ test: run-pass
#$ rules: CELL-11
# "`Cell`, `RefCell`, `Ref`, `RefMut`, `Atomic`, `Mutex`, `RwLock` live in
# `std.cell` and `std.sync`; `Cell` and `RefCell` **are** in the prelude (owner
# decision `OQ-10`): `Shared[T]` is already there and is heavier on every axis
# this document measures, so taxing the zero-cost facility and not the
# expensive one inverted the gradient the earlier rule meant to create."
#
# For a compiler-known type, being in the prelude means the name resolves with
# no import, as `Array` and `Option` already do. This file imports nothing.
#
# `RefCell` is not built yet, so only half of the rule is covered here.

fn main():
    c: Cell[i32] = Cell(41)
    c.set(c.get() + 1)
    println(c.get())
#$ stdout: 42
