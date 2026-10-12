#$ test: run-pass
#$ rules: BRW-10
#$ stdout: a bb ccc 3
#$ stdout: 1 [2, 3]
# `[BRW-10]` — a call to a struct's method that is not visible outside its
# package borrows only the fields its body uses, through the methods of the
# same kind it calls: `w.bump()` may run while a loop iterates `w.names`, and
# a view `p.first_a()` returns keeps only `p.a` borrowed.

struct Words:
    names: Array[String]
    count: int = 0

    fn bump(mut self):
        self.add(1)

    fn add(mut self, n: int):
        self.count += n

struct Pair:
    a: Array[int]
    b: Array[int]

    fn first_a(self) -> ref int:
        return self.a[0]

fn main():
    w = Words(["a", "bb", "ccc"])
    for name in w.names:
        w.bump()
        print(name, "")
    println(w.count)
    p = Pair([1], [2])
    r = p.first_a()
    p.b.push(3)
    println(r, p.b)
