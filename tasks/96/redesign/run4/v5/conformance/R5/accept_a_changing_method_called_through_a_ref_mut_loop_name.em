#$ test: run-pass
#$ rules: R5
#$ stdout: 100 100
## R5: through a `ref mut` loop name a method that changes its object may be called; the call on the
## object carries no word (R4).

class Player:
    hp: int = 10

    fn heal():
        this.hp = 100

fn heal_all(ref mut players: Array[Player]):
    for ref mut p in players:
        p.heal()

fn main():
    team = [Player(), Player()]
    heal_all(ref mut team)
    println(team[0].hp, team[1].hp)
