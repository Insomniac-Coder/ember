# Clause BRW-15.2: "A method that may change its object, called through a name that only reads it,
# is `E3023`" -- one whose body changes nothing of `this` only reads it (`[EXC-15]`) and may be.
#$ test: run-pass
#$ rules: BRW-15, EXC-15
#$ stdout: 10
#$ stdout: 10

class Player:
    hp: int = 10
    fn health() -> int:
        return this.hp

fn show(p: Player):
    println(p.health())

fn main():
    players = [Player()]
    for p in players:
        println(p.health())
    show(players[0])
