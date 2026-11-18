#$ test: run-pass
#$ rules: EXP-5
#$ stdout: 2
#$ stdout: ann
#$ stdout: 5
# `[EXP-5]` — a call whose result is used stays allowed (`line.split(" ").next()`), and a method
# that changes its object is called on a place.

struct Enemy:
    hp: int

    fn heal() -> int:
        this.hp += 1
        return this.hp

fn make() -> Enemy:
    return Enemy(hp=1)

fn main():
    println(make().heal())
    line = "ann lee"
    word = line.split(" ").next()
    match word:
        Some(w):
            println(w)
        None:
            println("none")
    e = make()
    e.heal()
    e.hp = 5
    println(e.hp)
