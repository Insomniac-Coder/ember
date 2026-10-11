#$ test: compile-fail
#$ rules: CLO-2, BRW-11

# ODR-102 — an owned callable value given to a callable parameter is lent
# mutably, so one value given to two is two mutable borrows (`E3022`).

fn counter() -> fn() -> int:
    n = 0
    return owned fn() -> int:
        n = n + 1
        return n

fn both(f: fn() -> int, g: fn() -> int) -> int:
    return f() * 10 + g()

fn main():
    c = counter()
    println(both(c, c))    #$ error[E3022]: `c` is already mutably borrowed
