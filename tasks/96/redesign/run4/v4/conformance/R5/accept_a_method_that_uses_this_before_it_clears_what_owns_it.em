#$ test: run-pass
#$ rules: R5, R4
#$ stdout: 0
## R5, R4, verdict 54: the nearest allowed program, the order E3031's message gives: `hit` uses `this`
## only before `ENEMIES.clear()`, so `ENEMIES[0].hit()` is allowed.

static mut ENEMIES: Array[Enemy] = []

class Enemy:
    hp: int = 10

    fn hit() changes hp, ENEMIES:
        this.hp -= 1
        ENEMIES.clear()

fn main() changes ENEMIES:
    ENEMIES.push(Enemy())
    ENEMIES[0].hit()
    println(ENEMIES.len())
