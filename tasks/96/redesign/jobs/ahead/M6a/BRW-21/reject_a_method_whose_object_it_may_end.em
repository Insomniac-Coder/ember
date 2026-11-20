# Clause BRW-21.5: "The object a method is called on counts the same way: with `fn hit():` whose
# body is `ENEMIES.clear()` then `this.hp -= 1`, `ENEMIES[0].hit()` is refused"
#$ test: compile-fail
#$ rules: BRW-21, EXC-15, BCK-8
# The spec's message, with this file's line numbers.

class Enemy:
    hp: int = 3

    fn hit():
        ENEMIES.clear()
        this.hp -= 1

static mut ENEMIES: Array[Enemy] = []

fn main():
    ENEMIES.push(owns Enemy())
    ENEMIES[0].hit()    #$ error[E3031]: `hit` clears `ENEMIES` (its line 11) and then changes `this.hp` (its line 12), and `ENEMIES[0]` is part of `ENEMIES`; in `hit`, change `this.hp` before `ENEMIES.clear()`
