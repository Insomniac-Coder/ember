#$ test: run-pass
#$ rules: UNS-3, UNS-12
#$ stdout: 7
#$ stdout: 9
# `[UNS-3]` — `L3010` reports an `unsafe:` block written directly inside unsafe code, which adds
# nothing: one in an `unsafe fn`'s body, whose body is unsafe code (`[UNS-12]`), and one inside
# another `unsafe:` block. One inside a `safe:` block of unsafe code is unsafe again, and needed.

@safety("`p` points to one writable `i32`")
unsafe fn put(p: *mut i32, v: i32):
    # SAFETY: the caller gives a writable slot
    unsafe:    #$ warning[L3010]: this `unsafe:` block is in `put`, an `unsafe fn`, whose body is unsafe code, so it adds nothing
        write(p, 0, v)

@safety("`p` points to one readable `i32`")
unsafe fn show(p: *mut i32):
    safe:
        n = 2
        # SAFETY: the caller gives a readable slot
        unsafe:
            println(read(p, 0) + n)

fn main():
    # SAFETY: one slot, written before it is read and freed once
    unsafe:
        p = alloc[i32](1)
        # SAFETY: as above
        unsafe:    #$ warning[L3010]: this `unsafe:` block is inside another `unsafe:` block, so it adds nothing
            put(p, 7)
        println(read(p, 0))
        show(p)
        free(p, 1)
