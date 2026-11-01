#$ test: run-pass
#$ rules: BRW-10, LT-1
#$ stdout: a 1
#$ b 1
#$ 2
# `[BRW-10]` with `[LT-1]` — `note` changes `this.log` and returns a view into its object: the call
# changes only `log`, so it runs while a loop walks `w.names`, and its result views `w` whole,
# read-only, which the loop's view of `w.names` does not conflict with.

struct World:
    names: Array[StrView]
    count: int
    log: Array[int]

    fn bump():
        this.count += 1

    fn note(n: int) -> ref int:
        this.log.push(owns n)
        return ref this.log[0]

fn main():
    w = World(names=["a", "b"], count=0, log=[])
    for n in w.names:
        w.bump()
        r = w.note(1)
        println(n, r)
    println(w.count)
