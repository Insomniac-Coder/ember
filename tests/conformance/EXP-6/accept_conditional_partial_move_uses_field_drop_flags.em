#$ test: run-pass
#$ rules: EXP-6, OWN-2, OWN-3, DRP-2
# A conditional field move requires a flag for that field, not one flag for
# the entire struct. On the moved path `first` owns Part(1); on the other path
# `pair` still owns both fields. Part(2) remains owned by `pair` on both paths.

struct Part:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Pair:
    pub first: Part
    pub second: Part

fn exercise(move_first: bool):
    pair = Pair(Part(1), Part(2))
    if move_first:
        _first = pair.first
        println(10)

fn main():
    exercise(true)
    exercise(false)
#$ stdout: 10
#$ 1
#$ 2
#$ 2
#$ 1
