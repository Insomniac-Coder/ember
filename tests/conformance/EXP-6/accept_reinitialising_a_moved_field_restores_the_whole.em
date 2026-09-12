#$ test: run-pass
#$ rules: EXP-6, OWN-3, OWN-5
# Reassigning the moved field fully reinitialises the aggregate. The overwrite
# drop before `pair.first = Part(3)` must be suppressed because that field is
# empty; moving the restored whole value is then legal.

struct Part:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Pair:
    pub first: Part
    pub second: Part

fn main():
    pair = Pair(Part(1), Part(2))
    old = pair.first
    pair.first = Part(3)
    restored = pair
#$ stdout: 2
#$ 3
#$ 1
