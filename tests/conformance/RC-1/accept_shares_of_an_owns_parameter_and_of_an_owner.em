#$ test: run-pass
#$ rules: RC-1, OWN-13
#$ profiles: debug, development, release
#$ stdout: 3
#$ 3
#$ 2
# `[RC-1]` — `shares x` is written before a new value, a local of this function that owns its value
# (an `owns` parameter, which the function owns from its first line), or a name that is already an
# owner: each makes one more owner of the same object.

class Player:
    hp: int

fn keep(owns p: Player) -> shares Player:
    return shares p

fn main():
    first = keep(owns Player(hp=3))
    second = shares first
    println(first.hp)
    println(second.hp)
    made = shares Player(hp=2)
    println(made.hp)
