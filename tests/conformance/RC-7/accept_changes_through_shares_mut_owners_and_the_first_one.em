#$ test: run-pass
#$ rules: RC-7, RC-1
#$ profiles: debug, development, release
#$ stdout: 7
#$ 9
# `[RC-7]` — a value made shared is changed by an owner made with `shares mut`, and by the name it
# was first shared from, which keeps what it could do; each sees the other's change, and nothing is
# checked while the program runs.

class Player:
    hp: int = 100

fn main():
    p = Player()
    other = shares mut p
    other.hp = 7
    println(p.hp)
    p.hp = 9
    println(other.hp)
