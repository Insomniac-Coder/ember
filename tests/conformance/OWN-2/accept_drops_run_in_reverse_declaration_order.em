#$ test: run-pass
#$ rules: OWN-2, DRP-1
# "reverse declaration order" — `b` was declared second, so it goes first. The
# order is observable through the destructors, which is the only way a program
# can see it.

struct R:
    pub n: i32

    fn drop(mut self):
        println(self.n)

fn main():
    a = R(1)
    b = R(2)
    println(a.n + b.n)
#$ stdout: 3
#$ 2
#$ 1
