#$ test: run-pass
#$ rules: DRP-2, OWN-2
# "[DRP-2]: tuple elements in reverse" (of declaration order, like locals).

struct R:
    pub n: i32

    fn drop(mut self):
        println(self.n)

fn main():
    _t = (R(1), R(2))
#$ stdout: 2
#$ 1
