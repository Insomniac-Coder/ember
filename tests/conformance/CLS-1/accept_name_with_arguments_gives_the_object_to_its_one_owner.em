#$ test: run-pass
#$ rules: CLS-1
#$ stdout: 7 3
# `[CLS-1]` — `Name(args)` allocates, runs the constructor and gives the object to the place it is
# stored in, its one owner: a field here, a local there.

class Enemy:
    hp: int

class Level:
    boss: Enemy
    minions: Array[Enemy]

fn main():
    level = Level(boss=Enemy(hp=7), minions=[Enemy(hp=1), Enemy(hp=2)])
    println(level.boss.hp, level.minions[0].hp + level.minions[1].hp)
