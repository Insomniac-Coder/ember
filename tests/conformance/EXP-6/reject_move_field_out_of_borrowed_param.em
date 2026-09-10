#$ test: compile-fail
#$ rules: EXP-6, FN-1
# `[FN-1]`: a borrowed parameter reads through a `ref` — "The callee cannot
# mutate or move `a`." It arrives as a bitwise copy with no loan behind it,
# so the move double-destroys across the call boundary: before the fix this
# compiled and the value dropped once with `x` and again with the caller.

struct Inner:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Outer:
    pub inner: Inner

fn steal(o: Outer):
    x = o.inner                  #$ error[E3013]: cannot move out of a borrowed parameter
    println(x.n)

fn main():
    o = Outer(Inner(7))
    steal(o)
    println(99)
