#$ test: compile-fail
#$ rules: RC-1, OWN-13, FN-1
# `[RC-1]` — a plain parameter only reads the caller's value, which may have one owner, so no second
# owner is made of it (`E2122`, `fn keep(p: Player): LIST.push(shares p)`): the caller gives one.

class Player:
    hp: int = 100

fn keep(p: Player) -> shares Player:
    return shares p                   #$ error[E2122]: `p` is the caller's `Player`, which `keep` only reads, and no second owner can be made of it; let the caller give one: write `owns p: shares Player` for the parameter, and `shares` before the argument at each call (`keep(shares …)`)

fn main():
    p = Player()
    println(keep(p).hp)
