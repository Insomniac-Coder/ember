#$ test: run-pass
#$ rules: R8
#$ stdout: 10 2
## R8 (a), (g): adding to the list that holds a read-only position is allowed; the element is found
## again at its next use.

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10)]
    boss = enemies[0]
    enemies.push(Enemy(hp=20))
    println(boss.hp, enemies.len())
