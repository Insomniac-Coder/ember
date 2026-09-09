#$ test: run-pass
#$ rules: DRP-2, OWN-2
# "struct fields after the struct's `drop`, in reverse declaration order". The
# order is what lets a destructor read its own fields: they are still there
# when it runs, and only afterwards do they go.

struct Part:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Whole:
    pub a: Part
    pub b: Part

    fn drop(mut self):
        println(0)

fn main():
    w = Whole(Part(1), Part(2))
    println(w.a.n + w.b.n)
#$ stdout: 3
#$ 0
#$ 2
#$ 1
