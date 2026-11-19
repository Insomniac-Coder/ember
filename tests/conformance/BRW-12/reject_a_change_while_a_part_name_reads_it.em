# Clause BRW-12.2: "is a read-only view of it: never a hand-over, and never a silent copy" -- the
# view reads the element itself, so changing it while `boss` is used is `[BRW-1]`'s refusal.
#$ test: compile-fail
#$ rules: BRW-12, BRW-1
# The message is `[BRW-1]`'s own example, with the file's line numbers.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = enemies[0]
    println(boss.hp)
    enemies[0].hp = 1    #$ error[E3021]: `enemies[0].hp = 1` (line 14) changes the enemy `boss` reads (`boss = enemies[0]`, line 12), and `boss` is used on line 16; change it after `boss`'s last use, or write `boss = ref mut enemies[0]` and change it through `boss`: `boss.hp = 1`
    println(enemies[0].hp)
    println(boss.hp)
