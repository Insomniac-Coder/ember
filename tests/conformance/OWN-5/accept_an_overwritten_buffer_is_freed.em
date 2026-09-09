#$ test: run-pass
#$ rules: OWN-5, OWN-2
# The same rule where what is overwritten owns a heap buffer.
#
# Written observably rather than as `assert-c: contains("ember_vec_free")`,
# which would have passed on the broken compiler too: it emitted exactly one
# free for two buffers, and `contains` cannot count. `Bag`'s drop prints the
# length of the buffer it is about to release, so the first buffer's release
# is a line of output and its absence is a red test.
#
# Expected: 1 (the overwritten bag, holding one element), then 0, then 2 (the
# surviving bag at scope end).

struct Bag:
    pub v: Array[i32]

    fn drop(mut self):
        println(self.v.len())

fn make(n: i32) -> Bag:
    xs: Array[i32] = Array[i32]()
    i = 0
    while i < n:
        xs.push(i)
        i = i + 1
    return Bag(xs)

fn main():
    b = make(1)
    b = make(2)
    println(0)
#$ stdout: 1
#$ 0
#$ 2
