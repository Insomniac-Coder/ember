#$ test: compile-fail
#$ rules: BRW-11
#$ help: fn tick(ref mut g: Game):
## design rules: R4
## R4: an argument that is read and a changeable argument of the same call may not name one place;
## where both name `g` it is refused (AN-18, E3021).

class Game:
    players: Array[String]
    score: int = 0

fn tick(g: Game, ref mut other: Game):
    other.score = g.players.len()

fn main():
    g = Game(players=[String("orc")])
    tick(g, ref mut g)    #$ error[E3021]: `tick` would read `g` and change it through `other` in the same call; pass it once: write `fn tick(ref mut g: Game):` with the body `g.score = g.players.len()`, and call `tick(ref mut g)`
    println(g.score)
