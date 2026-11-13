#$ test: run-pass
#$ rules: UNS-11, UNS-7, UNS-8, UNS-12
#$ stdout: 5
#$ stdout: 6
#$ stdout: 7
# `[UNS-11]` — `unsafe` is one word with one meaning by position. Before a declaration (`unsafe fn`,
# `unsafe interface`, an `unsafe fn(…)` type) it says that whoever uses it must vouch, with
# `@safety` saying what they must make true: `put` without one is `L3015`. Before a block or an
# implementation (`unsafe:`, `unsafe extend`, `unsafe extern`) it says that this code vouches, with
# `# SAFETY:` saying why: each without one is `W3012`. `safe:` inside unsafe code is checked as safe
# code.

@safety("`p` points to one writable `i32`")
unsafe fn set(p: *mut i32, v: i32):
    write(p, 0, v)

unsafe fn put(p: *mut i32, v: i32):    #$ warning[L3015]: `put` is an `unsafe fn`
    write(p, 0, v)

unsafe interface Raw:
    fn address() -> usize

struct Slot:
    at: usize

unsafe extend Slot implements Raw:    #$ warning[W3012]: no safety note
    fn address() -> usize:
        return this.at

unsafe extern "C":    #$ warning[W3012]: no safety note
    @safety("none: `abs` reads only its argument")
    unsafe fn abs(n: i32) -> i32

fn apply(f: unsafe fn(*mut i32, i32), p: *mut i32, v: i32):
    # SAFETY: `p` is the slot `main` allocated
    unsafe:
        f(p, v)

@safety("`p` points to one readable `i32`")
unsafe fn shown(p: *mut i32) -> i32:
    n: usize = 0
    safe:
        n = Slot(at=7).address()
    return read(p, 0) + (n - 7) as i32

fn main():
    unsafe:    #$ warning[W3012]: no safety note
        p = alloc[i32](1)
        set(p, 5)
        println(read(p, 0))
        put(p, abs(-6))
        println(read(p, 0))
        apply(set, p, 7)
        println(shown(p))
        free(p, 1)
