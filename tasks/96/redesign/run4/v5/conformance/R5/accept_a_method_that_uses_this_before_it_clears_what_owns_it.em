#$ test: run-pass
#$ rules: FN-4, FN-1
#$ stdout: 0
## design rules: R5, R4
## R5, R4, verdict 54: the nearest allowed program, the order E3031's message gives: `hit` uses `this`
## only before `ENEMIES.clear()`, so `ENEMIES[0].hit()` is allowed.

static mut ENEMIES: Array[Enemy] = []

class Enemy:
    hp: int = 10

    fn hit():
        this.hp -= 1
        ENEMIES.clear()

fn main():
    ENEMIES.push(Enemy())
    ENEMIES[0].hit()
    println(ENEMIES.len())
