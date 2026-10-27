#$ test: compile-fail
#$ rules: R3
#$ help: remove `owns`: `p: Player`
## R3: `owns T` is refused on a class type, which always has its own block (E2114; `V3/rules.md`
## section 3's message).

class Player:
    hp: int = 10

struct Team:
    p: owns Player             #$ error[E2114]: a `Player` object always lives in its own block

fn main():
    t = Team(p=Player())
    println(t.p.hp)
