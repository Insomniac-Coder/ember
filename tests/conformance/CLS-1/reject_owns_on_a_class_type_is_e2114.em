#$ test: compile-fail
#$ rules: CLS-1, OWN-10
# `[CLS-1]` — a class object always lives in its own block, so `owns` on a class type says nothing
# new and is `E2114`.

class Player:
    name: String

class Team:
    captain: owns Player        #$ error[E2114]: a `Player` object always lives in its own block; remove `owns`: `captain: Player`

fn main():
    println("x")
