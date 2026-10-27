#$ test: compile-fail
#$ rules: BRW-15
#$ help: for ref mut p in players:
## design rules: R5
## R5: a plain name never changes anything; a plain loop name only reads the element it names
## (AN-17, AN-20, E3023).

class Player:
    hp: int = 10

fn hurt_all(ref mut players: Array[Player]):
    for p in players:
        p.hp -= 1    #$ error[E3023]: `p.hp -= 1` changes a player, but `p` only reads it; to change the players through `p`, write `for ref mut p in players:`

fn main():
    team = [Player(), Player()]
    hurt_all(ref mut team)
    println(team[0].hp)
