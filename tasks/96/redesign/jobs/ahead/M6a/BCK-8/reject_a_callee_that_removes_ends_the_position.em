# Clause BCK-8.1: "which elements of a list it touches (a list parameter it only adds to (...)
# keeps a position of that list usable across the call)"; `[BRW-16]`: "a call that reads and
# removes included"
#$ test: compile-fail
#$ rules: BCK-8, BRW-16, EFF-24
# `cull` removes from the list, so `boss` is ended by the call and its later use is `E3060`. No
# text in the spec for this program: code only.

struct Enemy:
    hp: int

fn cull(ref mut es: Array[Enemy]):
    if es.len() > 1:
        es.remove(0)

fn main():
    enemies = [Enemy(hp=3), Enemy(hp=4)]
    boss = ref mut enemies[1]
    cull(ref mut enemies)
    boss.hp = 1    #$ error[E3060]
