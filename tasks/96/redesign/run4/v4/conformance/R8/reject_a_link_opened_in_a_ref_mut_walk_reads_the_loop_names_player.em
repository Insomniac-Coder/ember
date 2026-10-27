#$ test: compile-fail
#$ rules: R8, R13
#$ help: write `if q is not p:` above line 20
## R8 (f): `for ref mut p` makes `p` the only name for its player while it is used; a name opened from a
## link to a `Player` inside the walk may be that player, so reading through it is refused (E3021). Here
## nothing changes through the link, so `main` names no `changes Player`. Completeness check, G25.

class Player:
    hp: int = 10

fn main():
    players: Array[Player] = []
    first = Player(hp=10)
    w = weakref first
    players.push(first)
    players.push(Player(hp=20))
    for ref mut p in players:
        p.hp += 1
        if w exists as q:
            println(q.hp)    #$ error[E3021]: `p` is the only name for its player until its last use on line 21, and line 20 reads `q.hp`, where `q`, opened from `w` (line 19), may be that player; test first: write `if q is not p:` above line 20 and indent line 20 under it
        p.hp += 1
