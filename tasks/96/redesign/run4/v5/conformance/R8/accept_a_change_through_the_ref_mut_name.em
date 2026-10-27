#$ test: run-pass
#$ rules: BRW-16
#$ stdout: 1 1
## design rules: R8
## R8 (g): the second way out: name the element `ref mut` and change it through that name.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = ref mut enemies[0]
    boss.hp = 1
    println(boss.hp, enemies[0].hp)
