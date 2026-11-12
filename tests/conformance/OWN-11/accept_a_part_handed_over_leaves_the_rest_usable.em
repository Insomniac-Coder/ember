#$ test: run-pass
#$ rules: OWN-11
#$ stdout: 1 a
#$ stdout: b 1
# `[OWN-11]` — a part of a struct value handed over on its own (`names.push(owns p.name)`) leaves the rest
# usable (`p.hp`), and the part itself once it is given a new value (`p.name = String("b")`).

struct Player:
    name: String
    hp: int

fn main():
    p = Player(name = "a", hp = 1)
    names: Array[String] = []
    names.push(owns p.name)
    println(p.hp, names[0])
    p.name = String("b")
    println(p.name, p.hp)
