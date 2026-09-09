#$ test: run-pass
#$ rules: DRP-1, OWN-2
# And the half that must keep working: exactly one run, at the end of the
# value's life, without the program asking for it.

struct R:
    pub n: i32

    fn drop(mut self):
        println(self.n)

fn main():
    r = R(5)
    println(r.n)
#$ stdout: 5
#$ 5
