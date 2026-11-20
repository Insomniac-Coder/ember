# Clause EXC-15.1: "a call whose object is part of what the method may end before its last use of
# `this` is `E3031`"
#$ test: run-pass
#$ rules: EXC-15, BRW-21
#$ stdout: 3
#$ stdout: 0
# The same `hit`, called on an enemy that is not in `ENEMIES`: nothing it ends is its object.

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
    mine = Enemy()
    mine.hit()
    println(ENEMIES.len())
