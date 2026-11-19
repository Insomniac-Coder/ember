#$ test: run-fail
#$ rules: RT-7, RC-1, TST-31
#$ count_bits: 8
#$ panics: too many owners of a `Player` (255)
# `[RT-7]` — before each +1 of an owner count it is compared with its maximum, and at the maximum the
# program stops, naming the class and the limit; built with 8-bit counts, the limit is 255.

class Player:
    hp: int = 0

fn main():
    p = Player()
    owners: Array[shares Player] = []
    for _ in 0..300:
        owners.push(owns (shares p))
    println(owners.len())
