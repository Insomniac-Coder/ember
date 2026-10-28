#$ test: run-pass
#$ rules: R8
#$ stdout:
#$ 10
#$ 1
## R8 (g): the first way out: the change comes after `boss`'s last use.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = enemies[0]
    println(boss.hp)
    enemies[0].hp = 1
    println(enemies[0].hp)
