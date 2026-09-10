#$ test: compile-fail
#$ rules: DRP-5
# "[DRP-5]" through an `owned` call argument: passing `self.inner` to a
# function that takes ownership moves the field just as `x = self.inner`
# does, and the field still drops again after `drop` returns.

struct Inner:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Outer:
    pub inner: Inner

    fn drop(mut self):
        y = take(self.inner)         #$ error[E3010]: cannot move a field out of `drop`'s `mut self`
        println(y)

fn take(owned x: Inner) -> i32:
    return x.n

fn main():
    _o = Outer(Inner(1))
    println(200)
