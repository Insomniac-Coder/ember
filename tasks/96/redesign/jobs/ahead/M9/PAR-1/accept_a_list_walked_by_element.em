# Clause PAR-1.1: "or a list walked by element (`for ref mut p in players:`) into chunks"
#$ test: run-pass
#$ rules: PAR-1, PAR-2, PAR-5
#$ stdout: 9 9
# §XI.4's `hurt_all`, and the `while` loop of the E7016 message written as its fix.

class Player:
    hp: int = 10

fn hurt_all(ref mut players: Array[Player]):
    @parallel
    for ref mut p in players:
        p.hp -= 1

fn place(ref mut out: Array[int], idx: Span[int]):
    @parallel
    for i in 0..idx.len():
        out[i] = idx[i]

fn main():
    players = [Player(), Player()]
    hurt_all(ref mut players)
    println(players[0].hp, players[1].hp)
    out = [0, 0]
    place(ref mut out, [1, 2])
