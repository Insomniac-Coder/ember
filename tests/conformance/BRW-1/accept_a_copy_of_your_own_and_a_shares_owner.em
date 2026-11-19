# Clause BRW-1.10: "A copy of your own (`boss = copy of enemies[0]`) is restricted by nothing. A
# `shares` owner is an owner, not a view: it sees the changes the value's other owners make
# (`[RC-1]`)."
#$ test: run-pass
#$ rules: BRW-1, OWN-8, RC-1, OWN-13
#$ stdout: 9 1
#$ stdout: 5

@derive(Clone)
struct Enemy:
    hp: int

class Player:
    hp: int = 10

fn main():
    enemies = [Enemy(hp=5)]
    boss = copy of enemies[0]
    enemies[0].hp = 1
    boss.hp = 9
    println(boss.hp, enemies[0].hp)
    a = shares mut Player()
    b = shares a
    a.hp = 5
    println(b.hp)
