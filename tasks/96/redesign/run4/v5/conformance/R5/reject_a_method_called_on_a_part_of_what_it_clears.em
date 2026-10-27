#$ test: compile-fail
#$ rules: R5, R4, R8
#$ help: this.hp -= 1
## R5, R4, verdict 54: `this` counts like a view parameter: `hit` clears `ENEMIES` and then changes
## `this.hp`, so it cannot be called on `ENEMIES[0]`, which the clear frees under it (E3031).

static mut ENEMIES: Array[Enemy] = []

class Enemy:
    hp: int = 10

    fn hit():
        ENEMIES.clear()
        this.hp -= 1

fn main():
    ENEMIES.push(Enemy())
    ENEMIES[0].hit()    #$ error[E3031]: `hit` clears `ENEMIES` (line 13) and then changes `this.hp` (line 14), and `ENEMIES[0]` is part of `ENEMIES`; in `hit`, change `this.hp` before `ENEMIES.clear()`
    println(ENEMIES.len())
