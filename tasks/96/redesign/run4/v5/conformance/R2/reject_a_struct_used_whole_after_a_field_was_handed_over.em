#$ test: compile-fail
#$ rules: R2, R1
#$ help: give `p.name` a new value first, `p.name = String("")` above line 15, or keep a copy: write `names.push(copy of p.name)` in place of line 14
## R2, R1: after a field is handed over on its own, the owner is not whole, so using it whole is refused
## until the field gets a new value (E3040; verdict 78; completeness gap G11, whose code is now settled).

struct Player:
    name: String
    hp: int

fn main():
    p = Player(name=String("ann"), hp=10)
    names: Array[String] = []
    names.push(p.name)
    q = p                      #$ error[E3040]: `p.name` was handed over to `names` on line 14, so `p` is not whole on line 15
    println(q.hp)
