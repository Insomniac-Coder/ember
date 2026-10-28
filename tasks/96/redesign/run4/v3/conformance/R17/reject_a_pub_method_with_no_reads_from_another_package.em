#$ test: compile-fail
#$ rules: R17, R5, R8
#$ help: died: Array[String] = []
## R17: across a package's edge only the written word counts: `ledger`'s `pub fn note_any` says no
## `reads`, so callers here take it to read every field of `Game`, `players` included, which `p`
## changes while the loop runs (R8 (f); AN-17, E3021). The same program inside one package is
## accepted (accept_a_body_fact_is_used_inside_its_package).

from ledger import Game, Player

fn tick(ref mut g: Game):
    for ref mut p in g.players:
        p.hp -= 1
        if p.hp <= 0:
            g.note_any(f"{p.name} died")    #$ error[E3021]: `note_any` may read any field of `Game`, `players` included, which this loop changes through `p` (line 12); call `note_any` after the loop: write `died: Array[String] = []` before the loop, `died.push(f"{p.name} died")` in place of `g.note_any(f"{p.name} died")`, and `for msg in died: g.note_any(msg)` after the loop (in `ledger`, its first line could instead say what it reads: `pub fn note_any(msg: String) reads frame changes log:`)

fn main():
    g = Game(players=[Player(name=String("orc"), hp=1)], log=[])
    tick(ref mut g)
    println(g.log[0])
