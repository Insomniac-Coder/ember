#$ test: run-pass
#$ rules: RT-7, RC-1, WK-11
#$ profiles: debug, development, release
#$ stdout: 300
#$ 300
#$ 7
# `[RT-7]` — every program but the two test programs of the stops keeps the 32-bit counts: 300 owners
# and 300 links of one player are far below the limit, and each compare before a +1 passes.

class Player:
    hp: int = 7

fn main():
    p = Player()
    owners: Array[shares Player] = []
    links: Array[weakref Player] = []
    for _ in 0..300:
        owners.push(owns (shares p))
        links.push(owns (weakref p))
    println(owners.len())
    println(links.len())
    if links[299] exists as q:
        println(q.hp)
