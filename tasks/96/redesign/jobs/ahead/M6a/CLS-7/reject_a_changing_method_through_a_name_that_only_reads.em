# Clause CLS-7.1: "What a method does to it is worked out from its body, and `this` is never the
# only name of its object (`[EXC-15]`): a method whose body changes `this` may change it and is not
# called through a name that only reads it (`E3023`)"
#$ test: compile-fail
#$ rules: CLS-7, EXC-15, BRW-10, BRW-15
# `heal` changes `this` only through its callee `add`, so the fact comes from the summary. Four
# names that only read: a plain parameter, a part name, a `ref` name and a plain loop name.
# `[EXC-15]`'s text names a line of the method that changes `this` itself; here the change is a
# callee's, and what the message cites then is not in the spec: code only.

class Player:
    hp: int = 0

    fn add(n: int):
        this.hp += n

    fn heal():
        this.add(10)

fn tend(p: Player):
    p.heal()    #$ error[E3023]

fn main():
    players = [Player(), Player()]
    tend(players[0])
    first = players[0]
    first.heal()    #$ error[E3023]
    second = ref players[1]
    second.heal()    #$ error[E3023]
    for p in players:
        p.heal()    #$ error[E3023]
