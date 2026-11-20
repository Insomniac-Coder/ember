#$ test: run-pass
#$ rules: BRW-16
#$ stdout: 9 3
# `[BRW-16]` — a name made again from what an adding call gives holds
# what the call gives (`boss = ref mut grow_and_pick(ref mut enemies)`), not the position it held before.

struct Enemy:
    hp: int

fn grow_and_pick(ref mut es: Array[Enemy]) -> ref mut Enemy:
    es.push(owns Enemy(hp=9))
    last = es.len() - 1
    return ref mut es[last]

fn main():
    enemies = [Enemy(hp=1), Enemy(hp=2)]
    boss = ref mut enemies[0]
    boss.hp = 5
    boss = ref mut grow_and_pick(ref mut enemies)
    println(boss.hp, enemies.len())
