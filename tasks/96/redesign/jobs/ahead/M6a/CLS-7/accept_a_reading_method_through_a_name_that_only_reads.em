# Clause CLS-7.1: "and one whose body neither changes nor ends anything of `this` only reads it"
#$ test: run-pass
#$ rules: CLS-7, EXC-15, BRW-10
#$ stdout: 0
#$ stdout: 0
#$ stdout: 0
#$ stdout: 0
#$ stdout: 0
#$ stdout: 10
# `show` reads `this` through its callee `get`: through each read-only name it is allowed. The
# changing method goes through `ref mut`.

class Player:
    hp: int = 0

    fn get() -> int:
        return this.hp

    fn show():
        println(this.get())

    fn add(n: int):
        this.hp += n

    fn heal():
        this.add(10)

fn look(p: Player):
    p.show()

fn main():
    players = [Player(), Player()]
    look(players[0])
    first = players[0]
    first.show()
    second = ref players[1]
    second.show()
    for p in players:
        p.show()
    for ref mut p in players:
        p.heal()
    println(players[1].hp)
