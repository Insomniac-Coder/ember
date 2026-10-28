#$ test: compile-fail
#$ rules: BRW-1
#$ help: write `if q is not p:` above line 19
## design rules: R8, R13, R5
## R8 (b), (g): a name opened from a link inside a loop over the list that holds the link's target may
## be the loop name's player (two paths to values of one type, one through a link); a change through it
## while the loop name still reads is refused at the change (E3021). Completeness check, G25.

class Player:
    hp: int = 10

fn main():
    players: Array[Player] = []
    first = Player(hp=10)
    w = weakref first
    players.push(first)
    players.push(Player(hp=20))
    for p in players:
        if w exists as ref mut q:
            q.hp -= 1    #$ error[E3021]: `q.hp -= 1` (line 19) changes a player that `p` may read (`q`, opened from `w` on line 18, links to a `Player`, and `p` is a `Player` of `players`, line 17), and `p` is used on line 20; test first: write `if q is not p:` above line 19 and indent line 19 under it, or change it after the loop
        println(p.hp)
