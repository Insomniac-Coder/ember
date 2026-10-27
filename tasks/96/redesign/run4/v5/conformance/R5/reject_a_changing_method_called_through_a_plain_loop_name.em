#$ test: compile-fail
#$ rules: R5
#$ help: for ref mut p in players:
## R5: a method that may change its object, worked out from its body (verdict 86), is refused on a name
## that only reads it (rules section 3, E3023).

class Player:
    hp: int = 10

    fn heal():
        this.hp = 100

fn heal_all(ref mut players: Array[Player]):
    for p in players:
        p.heal()    #$ error[E3023]: `heal` changes the player (`this.hp = 100`, line 11), and `p` only reads it; write `for ref mut p in players:` in place of line 14

fn main():
    team = [Player(), Player()]
    heal_all(ref mut team)
    println(team[1].hp)
