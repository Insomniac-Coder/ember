#$ test: compile-fail
#$ rules: DRP-5
# "[DRP-5]: a `drop` method's `mut self` may not move fields out (`[EXP-6]`)."
# The field drops again after `drop` returns, so the move double-destroys:
# the moved value drops with its new owner at the end of the `drop` body,
# then drops again as a field. Before the fix this compiled and printed `1`
# twice.

struct Inner:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Outer:
    pub inner: Inner

    fn drop(mut self):
        x = self.inner               #$ error[E3010]: cannot move a field out of `drop`'s `mut self`
        println(x.n)

fn main():
    _o = Outer(Inner(1))
    println(200)
