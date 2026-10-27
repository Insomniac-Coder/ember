#$ test: run-pass
#$ rules: BRW-16
#$ stdout: 14
## design rules: R8
## R8 (f): a `ref mut` name is alone only until its last use; after it, the list's own name reads and
## changes the element.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = ref mut enemies[0]
    boss.hp -= 1
    enemies[0].hp += 5
    println(enemies[0].hp)
