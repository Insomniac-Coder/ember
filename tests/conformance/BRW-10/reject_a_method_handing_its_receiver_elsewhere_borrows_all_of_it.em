#$ test: compile-fail
#$ rules: BRW-10
# `[BRW-10]` — a method that hands its receiver to anything but a method of
# the same kind borrows all of it.

struct Words:
    names: Array[String]
    count: int = 0

    fn bump():
        touch(ref mut this)

fn touch(ref mut w: Words):
    w.count += 1

fn main():
    w = Words(["a", "bb"])
    for name in w.names:
        w.bump() #$ error[E3020]: `w.bump()` (line 19) changes `w.names` (through `touch`, line 14) while this loop walks `w.names` (line 18); change it after the loop: move line 19, `w.bump()`, below the loop
        print(name)
