# Clause BCK-8.1: "which elements of a list it touches (a list parameter it only adds to, `push` or
# `extend`, keeps a position of that list usable across the call)"
#$ test: run-pass
#$ rules: BCK-8, BRW-16, EFF-24
#$ stdout: 1 3
# `spawn_into` only adds to its list, so the position `boss` survives the call.

struct Enemy:
    hp: int

fn spawn_into(ref mut es: Array[Enemy]):
    es.push(owns Enemy(hp=5))
    es.extend(owns [Enemy(hp=6)])

fn main():
    enemies = [Enemy(hp=3)]
    boss = ref mut enemies[0]
    spawn_into(ref mut enemies)
    boss.hp = 1
    println(enemies[0].hp, enemies.len())
