#$ test: compile-fail
#$ rules: EXP-4, TXT-10, BRW-8
#$ profiles: debug, release, shipping
# D-488 (the other agent's case). Extending internal evaluation bindings must not
# extend a temporary source beyond the actual source statement.

fn make() -> String:
    return "left:right"

fn main():
    pair = make().split_once(":")  #$ error[E3060]: this temporary is dropped at the end of its statement while it is still borrowed
    println(pair)
