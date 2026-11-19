#$ test: run-fail
#$ rules: RT-7, WK-11, TST-31
#$ count_bits: 8
#$ panics: too many `weakref` links to a `Player` (255)
# `[RT-7]`, `[WK-11]` — before each +1 of a weak count it is compared with its maximum, and at the
# maximum the program stops; built with 8-bit counts, the limit is 255.

class Player:
    hp: int = 0

fn main():
    p = Player()
    links: Array[weakref Player] = []
    for _ in 0..300:
        links.push(owns (weakref p))
    println(links.len())
