#$ test: run-pass
#$ rules: CLO-10, PHIL-2
#$ profiles: debug, release
#$ stdout: 340282366920938463463374607431768211455 -170141183460469231731687303715884105728
#$ assert-c: !contains("_Alignof(void*)")
# `[CLO-10]` (D-620) — up to three pointer-sized words of captured state are held inline, whatever
# their alignment: a captured `u128` is two words, so making the value allocates nothing and a
# `@noalloc` function may make one. The backend kept a state aligned above a pointer (a `u128` on
# clang and gcc) in a heap allocation, which carried no `Alloc`; the words are only ever copied,
# so they hold it.

@noalloc
fn keep(x: u128) -> fn() -> u128:
    return owned fn() => x

@noalloc
fn keep_signed(x: i128) -> fn() -> i128:
    return owned fn() => x

fn main():
    f = keep(u128.MAX)
    g = keep_signed(i128.MIN)
    println(f(), g())
