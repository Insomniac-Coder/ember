# Clause OBJ-5.1: "an owner made from `this` (`thread_shared this`; `shares this` and `shares mut
# this` are refused in every method, `[RC-1]`) is `E3016`"
#$ test: compile-fail
#$ rules: OBJ-5, CLS-7a, RC-1, EFF-1
# Alone because `thread_shared` is M9's. `drop` calls `publish`, whose body makes an owner of
# `this`; `publish` called anywhere else is allowed. No text in the spec for this program: code
# only.

class Player:
    hp: int = 0

    fn publish():
        BOARD.push(thread_shared this)

    fn drop():
        this.publish()    #$ error[E3016]

static mut BOARD: Array[thread_shared Player] = []

fn main():
    _p = Player()
