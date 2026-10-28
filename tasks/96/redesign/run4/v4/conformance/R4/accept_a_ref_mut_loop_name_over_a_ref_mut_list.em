#$ test: run-pass
#$ rules: R4, R5
#$ stdout: 11
## R4: the nearest allowed program: `heal_all` takes the list as `ref mut`, so its loop name may change
## the caller's players (completeness G6).

class Player:
    hp: int = 10

fn heal_all(ref mut players: Array[Player]):
    for ref mut p in players:
        p.hp += 1

fn main():
    team = [Player(), Player()]
    heal_all(ref mut team)
    println(team[0].hp)
