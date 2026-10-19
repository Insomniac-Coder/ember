#$ test: run-pass
#$ rules: GRM-38, CTL-1
#$ profiles: debug, development
#$ stdout: 60 20 3 3
#$ [4, 9, 16]
# `[GRM-38]` — a parenthesised comprehension is a generator expression: a lazy iterator, usable as
# any value is, over anything a `for` loop iterates (`[CTL-1]`): a list, a text's `char`s, a list
# made for it, another iterator.

struct Player:
    hp: int

fn main():
    xs = [1, 2, 3, 4]
    evens = (x * 10 for x in xs if x % 2 == 0)
    total = 0
    for v in evens:
        total += v
    zipped = (a + b for (a, b) in xs.iter().zip(xs.iter()))
    z = 0
    for v in zipped:
        z += v
    letters = 0
    for _c in (c for c in "hey"):
        letters += 1
    players = [Player(3), Player(0)]
    alive = 0
    for hp in (p.hp for p in players if p.hp > 0):
        alive += hp
    println(total, z, letters, alive)
    squares = (y * y for y in [2, 3, 4])
    println([s for s in squares])
