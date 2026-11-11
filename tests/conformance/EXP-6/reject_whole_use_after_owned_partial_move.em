#$ test: compile-fail
#$ rules: EXP-6, OWN-3
# A sibling remains usable after a partial move, but the aggregate as a whole
# does not. This is shape O4 (`E3040` with `[OWN-11]`'s part text, `E3042` folded into it), not the ordinary whole-local O1 shape.

struct Part:
    pub n: i32

    fn drop():
        println(this.n)

struct Pair:
    pub first: Part
    pub second: Part

fn consume(owns pair: Pair):
    println(pair.second.n)

fn main():
    pair = Pair(Part(1), Part(2))
    _first = pair.first
    println(pair.second.n)
    consume(owns pair)              #$ error[E3040]: `pair.first` was handed over to `_first` on line 21; `pair` is used whole on line 23; give it a new value first, or keep a copy: `_first = copy of pair.first`
