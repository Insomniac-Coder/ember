# Clause BRW-12.8: "`copy of` makes a copy of your own (`boss = copy of enemies[0]`)"
#$ test: run-pass
#$ rules: BRW-12, OWN-8
#$ stdout: 1 7

@derive(Clone)
struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = copy of enemies[0]
    boss.hp = 1
    enemies[0].hp = 7
    println(boss.hp, enemies[0].hp)
