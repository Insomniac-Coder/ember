#$ test: run-pass
#$ rules: EXP-6, OWN-2, OWN-3
# Move paths are recursive. Taking `outer.left.first` must not invalidate its
# sibling inside `left` or the wholly disjoint `right` field.

struct Part:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Pair:
    pub first: Part
    pub second: Part

struct Outer:
    pub left: Pair
    pub right: Part

fn main():
    outer = Outer(Pair(Part(1), Part(2)), Part(3))
    _first = outer.left.first
    println(outer.left.second.n + outer.right.n)
#$ stdout: 5
#$ 1
#$ 3
#$ 2
