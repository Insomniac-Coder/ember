# Clause BRW-16.1: "A name for a list element made by indexing (`boss = enemies[0]`, `boss = ref mut
# enemies[0]`, `p = ps.get(0).unwrap()`), of a list reached through locals that own it, parameters,
# `this`, one-owner fields and module variables (`static mut`, `[LT-3]`), is a position: the index is
# read once, where the name is made, and kept with it (a later change of `i` moves no position made
# from `enemies[i]`)"
#$ test: run-pass
#$ rules: BRW-16, BCK-1, STD-15
#$ stdout: 9 2
#$ stdout: 2
#$ stdout: 9 0
# Each of the three spellings; `i` changes after each name is made, and the name stays on its
# element.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=1), Enemy(hp=2)]
    i = 0
    boss = ref mut enemies[i]
    i = 1
    boss.hp = 9
    println(enemies[0].hp, enemies[1].hp)
    b = enemies[i]
    i = 0
    println(b.hp)
    p = enemies.get(i).unwrap()
    i = 1
    println(p.hp, i - 1)
