#$ test: compile-fail
#$ rules: THR-3, BRW-1
#$ error[E3021]: cannot write through a shared reference
# `[THR-3]` — a read guard reads through to `ref T`: it cannot write.

from std.sync import RwLock

fn main():
    rw = RwLock(5)
    with r = rw.read():
        r += 1
