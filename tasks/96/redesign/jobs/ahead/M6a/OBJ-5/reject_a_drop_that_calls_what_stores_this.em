# Clause OBJ-5.1: "Inside a `drop`, and in everything it calls (by their summaries, `[EFF-1]`), no
# owner is made of the object being ended: (...) is `E3016`, as storing `this` anywhere that
# outlives the call is (`[CLS-7a]`)"
#$ test: compile-fail
#$ rules: OBJ-5, CLS-7a, EFF-1, DIA-22
# `drop` calls `retire`, and `retire` calls `archive`, which takes the object over and stores it in
# a module variable: the call in `drop` is refused. Where the refusal stands (the call in `drop`,
# `[DIA-22]`) is the reading this file takes; no text in the spec for this program: code only.

class Player:
    name: String

    fn init(owns name: String):
        this.name = name

    fn archive() owns this:
        ARCHIVE.push(owns this)

    fn retire() owns this:
        this.archive()

    fn drop():
        this.retire()    #$ error[E3016]

static mut ARCHIVE: Array[Player] = []

fn main():
    _p = Player(String("ann"))
