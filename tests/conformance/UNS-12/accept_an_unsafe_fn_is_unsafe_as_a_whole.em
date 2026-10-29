#$ test: run-pass
#$ rules: UNS-12, UNS-1, UNS-7
#$ stdout: 513
#$ stdout: 6
#$ stdout: 3
#$ stdout: 7
#$ stdout: 8
# `[UNS-12]` (verdict 118) — an `unsafe fn` is unsafe as a whole: its body reads, writes and offsets
# raw pointers with no `unsafe:` block, and its callers vouch, each in a block of its own. A `safe:`
# block in it is checked as safe code. (Verdict 119) A function written in unsafe code is an
# `unsafe fn`, with `@safety`, and a lambda written there an `unsafe fn(…)` value, with its own
# `@safety` written just before it (verdict 126).

@safety("`p` points to two readable bytes")
unsafe fn read_u16_le(p: *u8) -> u16:
    return (p.read() as u16) | ((p.add(1).read() as u16) << 8)

@safety("`p` points to a writable `int`")
unsafe fn store(p: *mut int, v: int):
    p.write(v)
    safe:
        doubled = v * 2
        println(doubled)

fn main():
    a: Array[u8] = [1, 2, 3]
    # SAFETY: `a` holds three bytes
    unsafe:
        println(read_u16_le(a.as_span().as_ptr()))
    b: Array[int] = [1, 2]
    p = b.as_mut_span().as_mut_ptr()
    # SAFETY: `b` holds two elements, and nothing else reaches them while they are written
    unsafe:
        store(p, 3)
    println(b[0])
    # SAFETY: `p` still points to `b[0]`, which `poke` writes
    unsafe:
        @safety("`q` points to a writable `int`")
        fn poke(q: *mut int):
            q.write(7)
        poke(p)
    println(b[0])
    # SAFETY: the lambda writes through `p`, which still points to `b[0]`
    unsafe:
        set = @safety("`p` points to a writable `int`") fn(v: int) => p.write(v)
        set(8)
    println(b[0])
