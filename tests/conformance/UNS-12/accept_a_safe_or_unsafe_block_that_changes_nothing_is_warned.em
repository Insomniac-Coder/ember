#$ test: compile-pass
#$ rules: UNS-12, UNS-3
# `[UNS-12]` — a `safe:` block outside unsafe code changes nothing (`L3020`); an `unsafe:` block
# directly inside unsafe code adds nothing (`L3010`, `[UNS-3]`). A name `safe` is still a name.

@safety("`p` points to a readable byte")
unsafe fn first(p: *u8) -> u8:
    # SAFETY: the caller promised a readable byte
    unsafe:    #$ warning[L3010]: this `unsafe:` block is in `first`, an `unsafe fn`, whose body is unsafe code, so it adds nothing
        return p.read()

fn main():
    safe:    #$ warning[L3020]: this `safe:` block is not inside unsafe code, so it changes nothing
        println(1)
    safe: int = 2
    println(safe)
