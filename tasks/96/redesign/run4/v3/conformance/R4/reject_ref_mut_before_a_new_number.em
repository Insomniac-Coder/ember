#$ test: compile-fail
#$ rules: R4
#$ help: b = a + 1
## R4: `ref mut` before something this function may not change is refused: `a + 1` is a new number,
## not a place (AN-27).

fn bump(ref mut n: int):
    n += 1

fn main():
    a = 1
    bump(ref mut a + 1)    #$ error[E3027]: `a + 1` is a new number, not a place `bump` can change; keep it in a name first: `b = a + 1`, then `bump(ref mut b)`
    println(a)
