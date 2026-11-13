# Clause BRW-17.5: "Adding to a list moves only what it stores inline: a view of a class object (a
# link, a kept view, or an address reached through a link), which lives in its own block, is not
# ended by a `push` or an `extend` of the list that owns it (with `players: Array[Player]` and `w =
# Watcher(target=sharedref mut players[0])`, `w.target.hp -= 1` after `players.push(owns Player())`
# is allowed)"
#$ test: run-pass
#$ rules: BRW-17, LT-46, BCK-3, BCK-7, GRM-44
#$ stdout: 9 2
#$ stdout: 8
# The spec's case, an `extend` too, and a name opened from a link used across a push.

class Player:
    hp: int = 10

struct Watcher:
    target: sharedref mut Player

fn main():
    players = [Player()]
    w = Watcher(target=sharedref mut players[0])
    players.push(owns Player())
    w.target.hp -= 1
    more = [Player(hp=2)]
    players.extend(owns more)
    println(players[0].hp, players[2].hp)
    link = weakref players[0]
    if link exists as p:
        players.push(owns Player())
        println(p.hp - 1)
