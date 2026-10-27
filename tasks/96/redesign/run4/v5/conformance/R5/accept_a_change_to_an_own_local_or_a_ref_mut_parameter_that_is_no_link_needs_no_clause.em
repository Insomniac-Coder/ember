#$ test: run-pass
#$ rules: R5, R4
#$ stdout: 2
## R5: a change through a `ref mut` parameter or to an own local needs no word, as every change does
## (verdict 86): `clock` is `main`'s one-owner object, and `c` names it
## (`accept_a_change_through_a_link_needs_no_word_whatever_holds_it` is the same through a link).

class Clock:
    frame: int = 0

fn tick(ref mut c: Clock):
    c.frame += 1

fn main():
    clock = Clock()
    clock.frame += 1
    tick(ref mut clock)
    println(clock.frame)
