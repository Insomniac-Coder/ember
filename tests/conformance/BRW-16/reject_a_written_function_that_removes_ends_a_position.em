#$ test: compile-fail
#$ rules: BRW-16, BRW-17, EFF-24
# `[BRW-16]`, `[EFF-24]` — what a written function may remove from the list it is given ends a position
# of it, as a built-in removal does: a later use is `E3060`, not `E3022` at the call; a library function
# that reads elements and removes (`retain`) the same.

struct Enemy:
    hp: int

fn wipe(ref mut es: Array[Enemy]):
    es.clear()

fn by_a_written_function():
    enemies = [Enemy(hp=1)]
    boss = enemies[0]
    wipe(ref mut enemies)
    println(boss.hp)    #$ error[E3060]

fn by_retain():
    enemies = [Enemy(hp=1), Enemy(hp=0)]
    boss = enemies[0]
    enemies.retain(owns fn(e) => e.hp > 0)
    println(boss.hp)    #$ error[E3060]

fn main():
    by_a_written_function()
    by_retain()
