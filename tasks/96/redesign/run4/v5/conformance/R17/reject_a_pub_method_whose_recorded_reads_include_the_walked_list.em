#$ test: compile-fail
#$ rules: R17, R5, R8
#$ help: write `died: Array[String] = []` above line 13, `died.push(f"{p.name} died")` in place of line 16, and `for msg in died: g.note_any(msg)` below the loop
## R17: across a package's edge what a function reads is recorded in the package's interface file, as
## every fact worked out from a body is (verdict 86; no `reads` word): `ledger`'s `note_any` reads
## `this.players` (it counts them), which this loop changes through `p`, so the call inside the loop is
## refused (R8 (f); AN-17, E3021), the message naming the recorded fact. `ledger`'s `note`, which reads
## only `frame` and `log`, is allowed there (`accept_a_pub_methods_recorded_reads_cross_the_package_edge`).

from ledger import Game, Player

fn tick(ref mut g: Game):
    for ref mut p in g.players:
        p.hp -= 1
        if p.hp <= 0:
            g.note_any(f"{p.name} died")    #$ error[E3021]: `note_any` reads `g.players` (`ledger`'s interface file records it), which this loop changes through `p` (line 13); note after the loop

fn main():
    g = Game(players=[Player(name=String("orc"), hp=1)], log=[])
    tick(ref mut g)
    println(g.log[0])
