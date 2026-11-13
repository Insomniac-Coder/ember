#$ test: compile-fail
#$ rules: EXP-6, FN-1
# The whole-value half of the borrowed-parameter shape: `x = o` moves the
# caller's value just as `x = o.inner` moves part of it, and the caller drops
# it again at its own scope end. Before the fix this compiled and printed the
# destructor line once per owner instead of once.

struct Inner:
    pub n: i32

    fn drop():
        println(this.n)

fn steal(o: Inner):
    x = owns o                   #$ error[E3013]: `x = owns o` would hand over the caller's inner, but `o` only reads it; for an inner of your own write `x = copy of o`
    println(x.n)

fn main():
    o = Inner(7)
    steal(o)
    println(99)
