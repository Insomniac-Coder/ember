#$ test: compile-fail
#$ rules: OWN-3
#$ help: give it a new value first, `p.name = String("")` above line 16, or keep a copy: write `names.push(copy of p.name)` in place of line 15
## design rules: R2, R1
## R2, R1: a field handed over on its own (verdict 78) is refused until it gets a new value (E3040, the
## message of `V4/rules.md` section 3); the rest of `p` stays usable
## (`accept_a_field_handed_over_on_its_own_leaves_the_rest_usable`).

struct Player:
    name: String
    hp: int

fn main():
    p = Player(name=String("ann"), hp=10)
    names: Array[String] = []
    names.push(p.name)
    println(p.name)            #$ error[E3040]: `p.name` was handed over to `names` on line 15 and is used on line 16
