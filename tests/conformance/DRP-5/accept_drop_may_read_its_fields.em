#$ test: run-pass
#$ rules: DRP-5
# "[DRP-5]" forbids moving fields out of `this` in `drop`; reading them
# stays allowed. Both reads below are `Copy` (`i32`), so no move happens and
# the destructor still sees a whole value before the fields drop after it.

struct Inner:
    pub n: i32

    fn drop():
        println(this.n)

struct Outer:
    pub inner: Inner
    pub tag: i32

    fn drop():
        println(this.tag)
        println(this.inner.n)

fn main():
    _o = Outer(Inner(1), 2)
    println(200)
#$ stdout: 200
#$ 2
#$ 1
#$ 1
