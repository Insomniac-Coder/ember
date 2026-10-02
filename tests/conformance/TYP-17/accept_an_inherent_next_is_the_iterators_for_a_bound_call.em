#$ test: run-pass
#$ rules: TYP-17, TYP-24, IFC-1
#$ profiles: debug, release
#$ stdout: [2, 1, 0]
#$ [2, 1, 0] [9, 2]
# D-473 — a type that implements `Iterator` with the `next` of its own
# methods (`[IFC-1]`'s `extend` adds none) gives that `next` to every call
# made through the bound: std's adapters (`skip`) reach it, and a program
# using adapters only on other types compiles too.

struct Countdown:
    left: int

    fn next(mut self) -> Option[int]:
        if self.left == 0:
            return None
        self.left -= 1
        return Some(self.left)

extend Countdown implements Iterator:
    type Item = int

fn main():
    seen: Array[int] = []
    for n in Countdown(left = 3):
        seen.push(n)
    println(seen)
    xs: Array[int] = [4, 9, 2]
    println(Countdown(left = 4).skip(1).to_array(), xs.copied().skip(1).to_array())
