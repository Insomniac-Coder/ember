#$ test: run-pass
#$ rules: R2, R3
#$ stdout: 7 1
## R2, R3: `boss = copy of enemies[0]` makes a copy of your own (rows 6, 9); changing it leaves the
## list's enemy as it was, and nothing restricts it (R8 (g)).

@derive(Clone)
struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10), Enemy(hp=20)]
    boss = copy of enemies[0]
    boss.hp = 1
    enemies[0].hp = 7
    println(enemies[0].hp, boss.hp)
