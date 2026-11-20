#$ test: run-pass
#$ rules: BRW-16, EFF-24
#$ stdout: 7 101
#$ stdout: 2
# `[BRW-16]`, `[EFF-24]` — a function whose summary only adds to the list it is given (`spawn`, a hundred
# pushes) keeps a position of it, as a built-in add does: the name finds its element again after the call.

struct Enemy:
    hp: int

fn spawn(ref mut es: Array[Enemy], n: int):
    for k in 0..n:
        es.push(owns Enemy(hp=k))

fn main():
    enemies = [Enemy(hp=1)]
    boss = ref mut enemies[0]
    spawn(ref mut enemies, 100)
    boss.hp = 7
    println(enemies[0].hp, enemies.len())
    first = enemies[1]
    spawn(ref mut enemies, 3)
    println(first.hp + 2)
