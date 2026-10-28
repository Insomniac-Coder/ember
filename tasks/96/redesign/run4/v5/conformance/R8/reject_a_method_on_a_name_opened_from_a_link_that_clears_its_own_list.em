#$ test: compile-fail
#$ rules: BRW-21
#$ help: in `hit`, change `this.hp` before `ENEMIES.clear()`
## design rules: R8, R4, R5
## R8 (b), R4, R5 (verdict 54, hole 2 reached through a link): `e` is opened from a `weakref` and may be
## any `Enemy`, one of `ENEMIES` included; `hit` clears `ENEMIES` and then changes `this` (E3031).

class Enemy:
    hp: int = 10

    fn hit():
        ENEMIES.clear()
        this.hp -= 1

static mut ENEMIES: Array[Enemy] = []

fn main():
    ENEMIES.push(Enemy())
    w = weakref ENEMIES[0]
    if w exists as e:
        e.hit()    #$ error[E3031]: `hit` clears `ENEMIES` (its line 2) and then changes `this.hp` (its line 3), and `e`, opened from a link, may be one of `ENEMIES`; in `hit`, change `this.hp` before `ENEMIES.clear()`
