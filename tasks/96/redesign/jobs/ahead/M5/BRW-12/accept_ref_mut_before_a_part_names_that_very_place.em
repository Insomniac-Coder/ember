# Clause BRW-12.6: "A word before the part says otherwise: `ref mut` makes a name for that very place
# that may change it (`boss = ref mut enemies[0]`, `r = ref mut first_even(ref mut xs)`)"
#$ test: run-pass
#$ rules: BRW-12, LT-1, BCK-6
#$ stdout: 1
#$ stdout: 5
# `first_even` is `[BCK-6]`'s example.

struct Enemy:
    hp: int

fn first_even(ref mut xs: MutSpan[int]) -> ref mut int:
    for i in 0..xs.len():
        if xs[i] % 2 == 0:
            return ref mut xs[i]
    xs[0] = 0
    return ref mut xs[0]

fn main():
    enemies = [Enemy(hp=10)]
    boss = ref mut enemies[0]
    boss.hp = 1
    println(enemies[0].hp)
    xs = [1, 4, 7]
    r = ref mut first_even(ref mut xs)
    r += 1
    println(xs[1])
