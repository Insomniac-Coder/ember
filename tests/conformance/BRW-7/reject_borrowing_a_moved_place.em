#$ test: compile-fail
#$ rules: BRW-7, OWN-3
# `[BRW-7]`, `[OWN-3]` (ODR-114) — borrowing a place a move may have reached is a use after the move:
# `E3040`, shape O1, labelled at the move and at the borrow.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    _b = a
    r: ref Array[i32] = ref a      #$ error[E3040]: `a` is borrowed after it has been moved out of
    println(r.len())
