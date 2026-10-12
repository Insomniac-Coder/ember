#$ test: compile-fail
#$ rules: BRW-10, CTL-2
# `[BRW-10]` — through the methods it calls on its receiver: `bump` hands
# `self` to `rename`, which pushes onto `names`.

struct Words:
    names: Array[String]
    count: int = 0

    fn rename(mut self):
        self.names.push("x")

    fn bump(mut self):
        self.count += 1
        self.rename()

fn main():
    w = Words(["a", "bb"])
    for name in w.names:
        w.bump() #$ error[E3020]: cannot mutate `w.names` while it is borrowed by this loop
        print(name)
