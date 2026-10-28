#$ test: compile-fail
#$ rules: R8, R13, R5
#$ help: write `if q is not p:` above line 20
## R8 (b), (g): a name opened from a link inside a loop over the list that holds the link's target may
## be the loop name's player (two paths to values of one type, one through a link); a change through it
## while the loop name still reads is refused at the change (E3021). `main` changes a player through the
## link, so it says `changes Player` (R5, verdict 64). Completeness check, G25.

class Player:
    hp: int = 10

fn main() changes Player:
    players: Array[Player] = []
    first = Player(hp=10)
    w = weakref first
    players.push(first)
    players.push(Player(hp=20))
    for p in players:
        if w exists as ref mut q:
            q.hp -= 1    #$ error[E3021]: `q.hp -= 1` (line 20) changes a player that `p` may read (`q`, opened from `w` on line 19, links to a `Player`, and `p` is a `Player` of `players`, line 18), and `p` is used on line 21; test first: write `if q is not p:` above line 20 and indent line 20 under it, or change it after the loop
        println(p.hp)
