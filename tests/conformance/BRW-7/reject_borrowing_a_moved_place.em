#$ test: compile-fail
#$ rules: BRW-7, OWN-3
# `[BRW-7]`, `[OWN-3]` (ODR-114) — borrowing a place a move may have reached is a use after the move:
# `E3040`, shape O1, labelled at the move and at the borrow.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(owns 1)
    _b = a
    r: ref Array[i32] = ref a      #$ error[E3040]: `a` was handed over to `_b` on line 9 and is used on line 10; to keep both, write `_b = copy of a` (a copy of your own) or `_b = shares a` (two owners of one value)
    println(r.len())
