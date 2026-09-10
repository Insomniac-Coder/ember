#$ test: compile-fail
#$ rules: EXP-6, FN-1
# The return path through the same rule: `return o` moves the caller's value
# into the return slot exactly as `x = o` moves it into a local. `[EXP-6]`
# lists `return` among the positions a move happens in, so the check covers
# it alongside assignments and call arguments.

struct Inner:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Outer:
    pub inner: Inner

fn whole_to_return(o: Outer) -> Outer:
    return o                     #$ error[E3013]: cannot move out of a borrowed parameter

fn main():
    o = Outer(Inner(7))
    p = whole_to_return(o)
    println(p.inner.n)
