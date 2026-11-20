# Clause BRW-1.6: "...; a position is ruled by `[BRW-16]`." `[BCK-6]`'s required acceptance: "a list
# added to while a position of it is still used (`boss = ref mut enemies[0]`, `enemies.push(owns
# Enemy(hp=5))`, then `boss.hp = 1`)".
#$ test: run-pass
#$ rules: BRW-1, BRW-16, BCK-6
#$ stdout: 1 5

struct Enemy:
    hp: int

fn main():
    enemies = [Enemy(hp=9)]
    boss = ref mut enemies[0]
    enemies.push(owns Enemy(hp=5))
    boss.hp = 1
    println(enemies[0].hp, enemies[1].hp)
