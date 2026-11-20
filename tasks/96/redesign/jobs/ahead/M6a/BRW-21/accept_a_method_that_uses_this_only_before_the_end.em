# Clause BRW-21.5: "and a `hit` that uses `this` only before the clear is allowed"
#$ test: run-pass
#$ rules: BRW-21, EXC-15, BCK-8
#$ stdout: 0
#$ stdout: 2
# `hit_first` uses `this` only before the clear; `hit`, which clears first, is called on an enemy
# that is not in `ENEMIES`.

class Enemy:
    hp: int = 3

    fn hit():
        ENEMIES.clear()
        this.hp -= 1

    fn hit_first():
        this.hp -= 1
        ENEMIES.clear()

static mut ENEMIES: Array[Enemy] = []

fn main():
    ENEMIES.push(owns Enemy())
    ENEMIES[0].hit_first()
    println(ENEMIES.len())
    mine = Enemy()
    mine.hit()
    println(mine.hp)
