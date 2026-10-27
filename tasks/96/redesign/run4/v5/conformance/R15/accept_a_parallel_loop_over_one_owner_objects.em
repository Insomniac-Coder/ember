#$ test: run-pass
#$ rules: THR-1, OWN-1
#$ stdout: 9 1
## design rules: R15, R1
## R15: each element of `Array[Player]` has one owner (R1), so `for ref mut p in players` in a
## `@parallel` loop hands each chunk its own objects, with no run-time claim (AN-74's `hurt_all`).

class Player:
    hp: int = 10
    x: int = 0

fn hurt_all(ref mut players: Array[Player]):
    @parallel
    for ref mut p in players:
        p.hp -= 1
        p.x += p.hp & 1

fn main():
    players = [Player(), Player()]
    hurt_all(ref mut players)
    println(players[0].hp, players[0].x)
