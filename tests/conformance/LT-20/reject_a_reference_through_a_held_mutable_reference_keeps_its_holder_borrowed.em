#$ test: compile-fail
#$ rules: LT-20, BRW-1
# `[LT-20]`, `[BRW-1]` — a reference reached through a mutable reference that
# another place holds keeps that place borrowed: the mutable reference cannot
# be copied out of it, so a second `take` while the first element reference
# lives would make two mutable references to one element. (Through a shared
# reference, what it reaches outlives the holder's borrow: D-470.)

struct Slots:
    xs: ref mut Array[int]
    name: str

    fn take(mut self) -> (ref mut int, str):
        return (ref mut self.xs[0], self.name)

fn main():
    xs = [1, 2, 3]
    s = Slots(xs = ref mut xs, name = "n")
    (a, n1) = s.take()
    (b, n2) = s.take() #$ error[E3025]
    a = 7
    b = 8
    println(n1, n2)
