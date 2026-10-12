#$ test: compile-fail
#$ rules: BRW-10
# `[BRW-10]` — a method that hands its receiver to anything but a method of
# the same kind borrows all of it.

struct Words:
    names: Array[String]
    count: int = 0

    fn bump(mut self):
        touch(self)

fn touch(mut w: Words):
    w.count += 1

fn main():
    w = Words(["a", "bb"])
    for name in w.names:
        w.bump() #$ error[E3020]: cannot mutate `w.names` while it is borrowed by this loop
        print(name)
