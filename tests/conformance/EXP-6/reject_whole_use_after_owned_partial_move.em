#$ test: compile-fail
#$ rules: EXP-6, OWN-3
# A sibling remains usable after a partial move, but the aggregate as a whole
# does not. This is O4/E3042, not the ordinary whole-local O1/E3040 shape.

struct Part:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Pair:
    pub first: Part
    pub second: Part

fn consume(owned pair: Pair):
    println(pair.second.n)

fn main():
    pair = Pair(Part(1), Part(2))
    _first = pair.first
    println(pair.second.n)
    consume(pair)              #$ error[E3042]: `pair` is partially moved
