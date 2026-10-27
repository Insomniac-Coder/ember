#$ test: compile-fail
#$ rules: R5
#$ help: for ref mut p in players:
## R5: a method that changes its object (its first line says `changes this`) is refused on a name that
## only reads it (rules section 3, E3023).

class Player:
    hp: int = 10

    fn heal() changes this:
        this.hp = 100

fn heal_all(ref mut players: Array[Player]):
    for p in players:
        p.heal()    #$ error[E3023]: `heal` changes the player (its first line says `changes this`), and `p` only reads it; write `for ref mut p in players:`

fn main():
    team = [Player(), Player()]
    heal_all(ref mut team)
    println(team[1].hp)
