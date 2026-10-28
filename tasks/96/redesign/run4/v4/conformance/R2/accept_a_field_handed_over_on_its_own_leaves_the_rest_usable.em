#$ test: run-pass
#$ rules: R2, R1
#$ stdout: 10
#$ ann bo
## R2, R1: a field of a struct value may be handed over on its own, as in Rust (verdict 78):
## `names.push(p.name)` moves `p`'s name into the list and `p.hp` stays usable; once `p.name` has a new
## value, `p` is whole again and may be handed over.

struct Player:
    name: String
    hp: int

fn main():
    p = Player(name=String("ann"), hp=10)
    names: Array[String] = []
    names.push(p.name)
    println(p.hp)
    p.name = String("bo")
    q = p
    println(names[0], q.name)
