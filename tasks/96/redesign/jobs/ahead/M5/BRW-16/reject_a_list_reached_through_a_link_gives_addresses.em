# Clause BRW-16.5: "A list reached through a link (a local or field holding a `shares`, `shares mut`
# or `sharedref mut` link) gives addresses, `[BRW-17]`."
#$ test: compile-fail
#$ rules: BRW-16, BRW-17, BRW-1, BCK-1, RC-1
# The add a position survives ends an address: a read-only one at its later use (`E3060`), and a
# `ref mut` one is refused at the add itself (`E3022`, `[BRW-1]`). Codes only.

struct Enemy:
    hp: int

class Team:
    enemies: Array[Enemy] = []

fn read_only():
    team = shares mut Team()
    team.enemies.push(owns Enemy(hp=1))
    boss = team.enemies[0]
    team.enemies.push(owns Enemy(hp=2))
    println(boss.hp)    #$ error[E3060]

fn changing():
    team = shares mut Team()
    team.enemies.push(owns Enemy(hp=1))
    boss = ref mut team.enemies[0]
    team.enemies.push(owns Enemy(hp=2))    #$ error[E3022]
    boss.hp = 3

fn main():
    read_only()
    changing()
