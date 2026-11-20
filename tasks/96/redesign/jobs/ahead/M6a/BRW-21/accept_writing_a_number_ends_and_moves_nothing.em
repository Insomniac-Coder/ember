# Clause BRW-21.4: "Writing a number or a `bool` ends and moves nothing, so a method that writes
# `this.health` through a link that may lead to its own object (a turret aimed at itself) is
# allowed"
#$ test: run-pass
#$ rules: BRW-21, EXC-15, EXC-17
#$ stdout: 9
#$ stdout: 2
# The turret aimed at itself (`[EXC-15]`'s example), and a function that writes a number of the
# enemy its parameter reads: neither ends nor moves what `this` or `e` views (that `e` sees the write
# is `[FN-1]`'s "shows through", M5's Q9). The clause's last case, leaving a lock's `with` block,
# needs `Lock` (M9) and is not here.

class Turret:
    health: int = 10
    target: weakref Turret = None

    fn fire():
        if this.target exists as ref mut t:
            t.health -= 1
        println(this.health)

class Enemy:
    hp: int = 3

static mut ENEMIES: Array[Enemy] = []

fn hurt_all(e: Enemy):
    for ref mut x in ENEMIES:
        x.hp -= 1
    println(e.hp)

fn main():
    a = Turret()
    a.target = weakref a
    a.fire()
    ENEMIES.push(owns Enemy())
    hurt_all(ENEMIES[0])
