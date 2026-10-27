#$ test: run-pass
#$ rules: R3
#$ stdout: 10
## R3: the nearest program that is allowed: a plain `Player` field holds the object alone.

class Player:
    hp: int = 10

struct Team:
    p: Player

fn main():
    t = Team(p=Player())
    println(t.p.hp)
