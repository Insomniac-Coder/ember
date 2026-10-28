#$ test: run-pass
#$ rules: BRW-21
#$ stdout: 0
## design rules: R8, R4, R5
## R8 (b), R4, R5: the nearest allowed program to
## `reject_a_method_on_a_name_opened_from_a_link_that_clears_its_own_list`: `hit` uses `this` before
## it clears the list that owns it, and nothing uses `e` after the call.

class Enemy:
    hp: int = 10

    fn hit():
        this.hp -= 1
        ENEMIES.clear()

static mut ENEMIES: Array[Enemy] = []

fn main():
    ENEMIES.push(Enemy())
    w = weakref ENEMIES[0]
    if w exists as e:
        e.hit()
    println(ENEMIES.len())
