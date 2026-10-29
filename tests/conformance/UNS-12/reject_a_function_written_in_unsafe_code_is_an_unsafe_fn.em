#$ test: compile-fail
#$ rules: UNS-12, UNS-7, FN-6
#$ warning[L3015]: this lambda is written in unsafe code, so it is an `unsafe fn(…)` value
# `[UNS-12]` (verdict 119) — a function written inside unsafe code is an `unsafe fn`: its body is
# unsafe code, its value is an `unsafe fn(…)`, which a safe callable type cannot hold, and it carries
# `@safety` (`L3015` without it). A lambda written there is an `unsafe fn(…)` value too.

fn make() -> fn(*mut u8):
    # SAFETY: nothing here touches memory
    unsafe:
        fn poke(q: *mut u8):    #$ warning[L3015]: `poke` is an `unsafe fn` that writes through the raw pointer `q`
            q.write(7)
        return poke    #$ error[E2020]: expected `fn(*mut u8) -> void`, found `unsafe fn(*mut u8) -> void`

fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn main():
    # SAFETY: the lambda touches no memory
    unsafe:
        println(apply(fn(x) => x + 1, 1))    #$ error[E2020]
