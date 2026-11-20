# Clause BRW-21.3: "with the body's precise facts (a change inside `if a is not b:` is not one for
# `a`)"
#$ test: run-pass
#$ rules: BRW-21, BCK-8, EXC-3
#$ stdout: 3
#$ stdout: 3
#$ stdout: 1
# `strike` removes `ENEMIES[0]` only when `a` is not that enemy, so what it ends is never `a`,
# whichever enemy it is given; the other enemy lives in its own block and is not moved.

class Enemy:
    hp: int = 3

static mut ENEMIES: Array[Enemy] = []

fn strike(a: Enemy):
    if a is not ENEMIES[0]:
        ENEMIES.remove(0)
    println(a.hp)

fn main():
    ENEMIES.push(owns Enemy())
    ENEMIES.push(owns Enemy())
    strike(ENEMIES[0])
    strike(ENEMIES[1])
    println(ENEMIES.len())
