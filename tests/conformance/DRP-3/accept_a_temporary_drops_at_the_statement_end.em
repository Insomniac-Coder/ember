#$ test: run-pass
#$ rules: DRP-3, EXP-4
# "Temporaries drop at the end of the enclosing statement." The `R(1)` here is
# never bound, so its life is the statement, and the marker after it proves the
# destructor ran before the next line rather than at the end of the function.

struct R:
    pub n: i32

    fn drop(mut self):
        println(self.n)

fn take(r: R) -> i32:
    return r.n

fn main():
    println(take(R(1)))
    println(9)
#$ stdout: 1
#$ 1
#$ 9
