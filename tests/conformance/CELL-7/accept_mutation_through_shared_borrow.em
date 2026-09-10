#$ test: run-pass
#$ rules: CELL-7, CELL-9
# `[CELL-7]` through a container: `borrow_mut` hands out a `RefMut` that reads
# through to the `Array`, so `push`/`len` work through the guard. `[CELL-9]` —
# the counter check that makes the second borrow see the first's release is
# present in every profile (no `exclusivity` gate in the lowering; release
# probes run green).

fn main():
    c: RefCell[Array[i32]] = RefCell(Array())
    with list = c.borrow_mut():
        list.push(42)
    with g = c.borrow():
        println(g.len())
#$ stdout: 1
#$ assert-c: contains("panic_refcell")
