#$ test: run-pass
#$ rules: BRW-16
#$ stdout: 3 x
## design rules: R8
## R8 (b): the way out E3060's message gives: `n` is a `String` of your own, so replacing
## `enemies[0].name` does not end it. AN-01, point 6.

struct Enemy:
    name: String
    hp: int

fn main():
    enemies = [Enemy(name=String(" orc "), hp=10)]
    boss = ref mut enemies[0]
    n = String(boss.name.trim())
    boss.name = String("x")
    println(n.len(), boss.name)
