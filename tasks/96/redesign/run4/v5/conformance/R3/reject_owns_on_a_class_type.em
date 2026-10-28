#$ test: compile-fail
#$ rules: TYP-45, TYP-46, ALC-1
#$ help: remove `owns`: `p: Player`
## design rules: R3
## R3: `owns T` is refused on a class type, which always has its own block (E2114; `V4/rules.md`
## section 3's message).

class Player:
    hp: int = 10

struct Team:
    p: owns Player             #$ error[E2114]: a `Player` object always lives in its own block

fn main():
    t = Team(p=Player())
    println(t.p.hp)
