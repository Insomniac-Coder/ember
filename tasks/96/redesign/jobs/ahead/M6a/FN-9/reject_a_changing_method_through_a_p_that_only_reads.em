# Clause FN-9.4: "A method that may change its object is not called through a `p` that only reads
# it (`E3023`, `[EXC-15]`)."
#$ test: compile-fail
#$ rules: FN-9, EXC-15, BRW-15
# `heal` changes its object through its callee `add`: the fact is `heal`'s summary. No text in the
# spec for a parameter: code only.

class Player:
    hp: int = 0

    fn add(n: int):
        this.hp += n

    fn heal():
        this.add(10)

fn tend(p: Player):
    p.heal()    #$ error[E3023]

fn main():
    a = Player()
    tend(a)
