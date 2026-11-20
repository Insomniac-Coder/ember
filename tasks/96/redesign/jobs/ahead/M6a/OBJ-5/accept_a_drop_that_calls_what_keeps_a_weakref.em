# Clause OBJ-5.2: "`weakref this` and `weakref thread_shared this` stay allowed (`[OWN-13]`): the weak
# count keeps the block, and opening the link finds the object ended (`[WK-3]`). Nothing is checked
# for it while the program runs."
#$ test: run-pass
#$ rules: OBJ-5, WK-3, OWN-13
#$ stdout: ended
# `drop` calls `remember`, which keeps a `weakref` link to the object being ended; opened after the
# end, the link finds it ended.

class Player:
    hp: int = 0

    fn remember():
        GHOSTS.push(weakref this)

    fn drop():
        this.remember()

static mut GHOSTS: Array[weakref Player] = []

fn make_and_end():
    _p = Player()

fn main():
    make_and_end()
    if GHOSTS[0] exists as g:
        println(g.hp)
    else:
        println("ended")
