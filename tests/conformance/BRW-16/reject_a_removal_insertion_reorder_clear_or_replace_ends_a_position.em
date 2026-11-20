#$ test: compile-fail
#$ rules: BRW-16, DIA-28, STD-15
# `[BRW-16]` — anything that may remove, insert, reorder, clear or replace the list ends a position, and a
# later use is `E3060` (not `E3022` at the change: `[DIA-28]`), with the spec's message for a removal.

struct Enemy:
    hp: int

fn removed():
    enemies = [Enemy(hp=1), Enemy(hp=2)]
    boss = ref mut enemies[0]
    boss.hp = 5
    enemies.remove(0)
    boss.hp = 1    #$ error[E3060]: `enemies.remove(0)` (line 13) may put another enemy in the place `boss` names (line 11), and `boss` is used on line 14; use `boss` before line 13, or name the enemy again after it: `boss = ref mut enemies[0]`

fn inserted():
    enemies = [Enemy(hp=1)]
    boss = enemies[0]
    enemies.insert(0, owns Enemy(hp=2))
    println(boss.hp)    #$ error[E3060]

fn reordered():
    enemies = [Enemy(hp=2), Enemy(hp=1)]
    boss = enemies[0]
    enemies.reverse()
    println(boss.hp)    #$ error[E3060]

fn cleared():
    enemies = [Enemy(hp=1)]
    boss = ref mut enemies[0]
    enemies.clear()
    boss.hp = 2    #$ error[E3060]: `enemies.clear()` (line 31) may end the enemy `boss` names (line 30), and `boss` is used on line 32; use `boss` before line 31, or name the enemy again after it: `boss = ref mut enemies[0]`

fn replaced():
    enemies = [Enemy(hp=1)]
    boss = enemies[0]
    enemies = [Enemy(hp=3)]
    println(boss.hp)    #$ error[E3060]
    println(enemies.len())

fn main():
    removed()
    inserted()
    reordered()
    cleared()
    replaced()
