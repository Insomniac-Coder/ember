#$ test: compile-fail
#$ rules: EXP-6, BRW-1
# `[EXP-6]`: "moving out of a `ref`/`ref mut` is `E3013`." The referent still
# owns the value and drops it, so the new owner drops it a second time —
# before the fix this compiled and printed `1` twice for one value
# (once with `x`, once with `o`).

struct Inner:
    pub n: i32

    fn drop(mut self):
        println(self.n)

struct Outer:
    pub inner: Inner

fn main():
    o = Outer(Inner(1))
    r: ref Outer = ref o
    x = r.inner                  #$ error[E3013]: cannot move out of a reference
    println(x.n)
