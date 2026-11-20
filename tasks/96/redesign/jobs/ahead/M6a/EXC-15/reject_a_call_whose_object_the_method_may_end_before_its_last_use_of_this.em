# Clause EXC-15.1: "a call whose object is part of what the method may end before its last use of
# `this` is `E3031` (`[BRW-21]`)"
#$ test: compile-fail
#$ rules: EXC-15, BRW-21, BCK-8
# The end and the later use both come through calls: `hit` calls `wipe`, which clears `ENEMIES`,
# and then `report`, which reads `this`. The object is `ENEMIES[0]`, written directly and through a
# part name. No text in the spec for this program: code only.

class Enemy:
    hp: int = 3

    fn report():
        println(this.hp)

    fn hit():
        wipe()
        this.report()

static mut ENEMIES: Array[Enemy] = []

fn wipe():
    ENEMIES.clear()

fn main():
    ENEMIES.push(owns Enemy())
    ENEMIES[0].hit()    #$ error[E3031]
    first = ENEMIES[0]
    first.hit()    #$ error[E3031]
