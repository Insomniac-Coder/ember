#$ test: run-pass
#$ rules: R8, R3
#$ stdout: 10 1
## R8 (g), verdict 5: a copy of your own (`copy of`) is restricted by nothing; the list's element may
## change while the copy is used, and the copy keeps its old value.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = copy of enemies[0]
    enemies[0].hp = 1
    println(boss.hp, enemies[0].hp)
