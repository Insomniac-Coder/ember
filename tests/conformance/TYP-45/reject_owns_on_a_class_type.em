#$ test: compile-fail
#$ rules: TYP-45, OWN-10
# `[TYP-45]`, `[OWN-10]` — a class object always lives in its own block: `owns` before a class type
# is `E2114`.

class Player:
    hp: int

struct Holder:
    p: owns Player  #$ error[E2114]: a `Player` object always lives in its own block; remove `owns`: `p: Player`

fn main():
    println(1)
