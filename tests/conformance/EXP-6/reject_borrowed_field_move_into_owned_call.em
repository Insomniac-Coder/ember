#$ test: compile-fail
#$ rules: EXP-6, FN-1
# The call-argument path through the same rule: passing `o.inner` to an
# `owned` parameter moves the field exactly as `x = o.inner` does, and the
# caller drops it again. The move sits in the call terminator rather than in
# an assignment, so this pins the terminator half of the check.

struct Inner:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Outer:
    pub inner: Inner

fn take(owned x: Inner) -> i32:
    return x.n

fn via_call(o: Outer) -> i32:
    return take(o.inner)         #$ error[E3013]: cannot move out of a borrowed parameter

fn main():
    o = Outer(Inner(7))
    println(via_call(o))
