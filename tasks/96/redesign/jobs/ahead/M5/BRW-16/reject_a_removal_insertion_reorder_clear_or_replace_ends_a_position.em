# Clause BRW-16.3: "anything that may remove, insert, reorder, clear or replace the list ends the
# name, and a later use is `E3060` ("`enemies.remove(0)` (line 8) may put another enemy in the place
# `boss` names (line 6), and `boss` is used on line 9; use `boss` before line 8, or name the enemy
# again after it: `boss = ref mut enemies[0]`"), a call that reads elements and removes included."
#$ test: compile-fail
#$ rules: BRW-16, BCK-3, DIA-28, STD-15
# The first message is the spec's, with this file's line numbers (a position after a removal is
# `E3060`, not `E3022`: `[DIA-28]`); the other ways have no text in the spec: codes only.

struct Enemy:
    hp: int

fn removed():
    enemies = [Enemy(hp=1), Enemy(hp=2)]
    boss = ref mut enemies[0]
    boss.hp = 5
    enemies.remove(0)
    boss.hp = 1    #$ error[E3060]: `enemies.remove(0)` (line 17) may put another enemy in the place `boss` names (line 15), and `boss` is used on line 18; use `boss` before line 17, or name the enemy again after it: `boss = ref mut enemies[0]`

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
    boss.hp = 2    #$ error[E3060]

fn replaced():
    enemies = [Enemy(hp=1)]
    boss = enemies[0]
    enemies = [Enemy(hp=3)]
    println(boss.hp)    #$ error[E3060]
    println(enemies.len())

fn a_call_that_reads_and_removes():
    enemies = [Enemy(hp=1), Enemy(hp=0)]
    boss = enemies[0]
    enemies.retain(owns fn(e) => e.hp > 0)
    println(boss.hp)    #$ error[E3060]

fn main():
    removed()
    inserted()
    reordered()
    cleared()
    replaced()
    a_call_that_reads_and_removes()
