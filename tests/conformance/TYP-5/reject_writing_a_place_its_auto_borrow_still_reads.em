#$ test: compile-fail
#$ rules: TYP-5, BRW-1
# An auto-borrow is a borrow: the place cannot be written while the `ref`
# taken of it is still used.

fn main():
    y = 5
    r: ref int = y
    y = 6    #$ error[E3021]: `y = 6` (line 9) changes the number `r` reads (`r: ref int = y`, line 8)
    println(r)
