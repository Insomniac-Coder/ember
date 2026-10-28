#$ test: run-pass
#$ rules: RC-5, WK-11, RC-1
#$ stdout: 3
## design rules: R10, R11, R9
## R10 point 2: "A view of a `thread_shared` object reached through a `weakref` link holds one atomic
## count for its life, as Rust's `Weak::upgrade` does: another thread may end the last owner at any
## moment, which no line of this thread can be checked against". Always: even here, where no line of
## this thread can end a `Config` (section 2, "one atomic count for the view's life"). Checked in the
## emitted C: one atomic +1 at the opening and one atomic -1 after `v`'s last use.

class Config:
    level: int = 3

fn main():
    c = thread_shared Config()
    w = weakref c
    if w exists as v:
        println(v.level)
