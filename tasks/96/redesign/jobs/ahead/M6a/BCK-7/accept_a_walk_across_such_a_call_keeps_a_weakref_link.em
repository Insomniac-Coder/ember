# Clause BCK-7.2: "so a walk across such a call keeps a `weakref` link and tests it at every step"
#$ test: run-pass
#$ rules: BCK-7, RC-5, WK-18
#$ stdout: 6
#$ stdout: 7
#$ stdout: gone
# The link, not a view, lives across the calls, and each step opens it again.

class Player:
    hp: int = 5

fn heal_all(ref mut ps: Array[Player]):
    for ref mut p in ps:
        p.hp += 1

fn main():
    players = [Player()]
    up = weakref players[0]
    for k in 0..3:
        heal_all(ref mut players)
        if k == 2:
            players.clear()
        if up exists as p:
            println(p.hp)
        else:
            println("gone")
