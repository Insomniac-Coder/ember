#$ test: compile-fail
#$ rules: DRP-5
# "[DRP-5]" through an `owned` call argument: passing `self.inner` to a
# function that takes ownership moves the field just as `x = self.inner`
# does, and the field still drops again after `drop` returns.

struct Inner:
    pub n: i32

    fn drop():
        println(this.n)

struct Outer:
    pub inner: Inner

    fn drop():
        y = take(owns this.inner)         #$ error[E3010]: cannot move a field out of `this` in `drop`
        println(y)

fn take(owns x: Inner) -> i32:
    return x.n

fn main():
    _o = Outer(Inner(1))
    println(200)
