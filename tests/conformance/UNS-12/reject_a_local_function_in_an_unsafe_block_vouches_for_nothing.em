#$ test: compile-fail
#$ rules: UNS-12, UNS-11, GRM-28
# `[UNS-12]`, `[UNS-11]` — a local function runs wherever it is called from, so its body starts outside
# `unsafe` even when it is written inside an `unsafe:` block: the block vouches for its own statements,
# not for a function that can leave it. (A closure is the code around it, and keeps the block's `unsafe`.)

fn make() -> fn(*mut u8):
    # SAFETY: nothing here touches memory
    unsafe:
        fn poke(q: *mut u8):
            q.write(7)    #$ error[E3100]: `write` through a raw pointer needs an `unsafe` block
        return poke

fn main():
    a: Array[u8] = [1, 2]
    poke = make()
    poke(a.as_mut_span().as_mut_ptr())
