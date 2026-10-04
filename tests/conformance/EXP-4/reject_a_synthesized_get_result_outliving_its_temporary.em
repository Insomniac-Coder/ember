#$ test: compile-fail
#$ rules: EXP-4, TXT-10, BRW-8
#$ profiles: debug, release, shipping
# D-488 (the other agent's case). The same source-statement limit applies to an
# Option view produced by get, independently of the split_once contract.

fn make() -> String:
    return "é:left"

fn main():
    part = make().get(0..2)  #$ error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
    println(part)
