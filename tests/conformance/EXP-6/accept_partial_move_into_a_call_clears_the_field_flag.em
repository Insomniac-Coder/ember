#$ test: run-pass
#$ rules: EXP-6, OWN-2, OWN-3
# Call arguments are MIR terminators, so their moves need the same per-field
# flag update as assignment rvalues. Otherwise the source field is dropped
# again after ownership has already transferred to `consume`.

struct Part:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Pair:
    pub first: Part
    pub second: Part

fn consume(owned part: Part):
    println(10)

fn exercise(move_first: bool):
    pair = Pair(Part(1), Part(2))
    if move_first:
        consume(pair.first)

fn main():
    exercise(true)
    exercise(false)
#$ stdout: 10
#$ 1
#$ 2
#$ 2
#$ 1
