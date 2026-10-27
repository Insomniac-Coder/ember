#$ test: compile-fail
#$ rules: R4, R5, R8
#$ help: fn heal_all(ref mut players: Array[Player]):
## R4: `ref mut` before something this function may not change is refused, a loop name over a list it
## only reads included: `players` is a plain parameter, so `p` would change the caller's players
## behind a read-only promise (AN-15, E3027; completeness G6).

class Player:
    hp: int = 10

fn heal_all(players: Array[Player]):
    for ref mut p in players:    #$ error[E3027]: `players` only reads the caller's list, so `p` cannot change its players; take it as `ref mut players: Array[Player]` on `heal_all`'s first line and call `heal_all(ref mut team)`
        p.hp += 1

fn main():
    team = [Player(), Player()]
    heal_all(team)
    println(team[0].hp)
