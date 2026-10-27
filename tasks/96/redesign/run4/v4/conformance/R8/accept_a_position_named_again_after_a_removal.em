#$ test: run-pass
#$ rules: R8, R2
#$ stdout: 1 19
## R8 (a): after a removal the position is named again, the way out E3060's message gives; a `ref mut`
## name may be given another ref (R2, verdict 42).

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=10), Enemy(hp=20)]
    boss = ref mut enemies[0]
    boss.hp -= 1
    enemies.remove(0)
    boss = ref mut enemies[0]
    boss.hp -= 1
    println(enemies.len(), enemies[0].hp)
