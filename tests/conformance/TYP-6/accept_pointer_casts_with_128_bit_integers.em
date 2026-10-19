#$ test: run-pass
#$ rules: TYP-6, TYP-7, UNS-1
#$ profiles: debug, development, release
#$ stdout: 42 42 true true
#$ 42 0
# `[TYP-6]` — a raw pointer converts to and from a 128-bit integer as to any other: a pointer
# zero-extends, and an integer keeps its low bits (here `1 << 100` above the address drops).

fn main():
    big: u128 = (1 as u128) << 100
    unsafe:  # SAFETY: the pointer is read only while the allocation lives
        p: *mut i64 = alloc[i64](1)
        write(p, 0, 42)
        wide = p as u128
        signed = p as i128
        back = wide as *mut i64
        again = signed as *mut i64
        println(read(back, 0), read(again, 0), wide == (p as usize) as u128, signed > 0)
        low = (big + wide) as *mut i64
        println(read(low, 0), (0 as *mut i64) as u128)
        free(p, 1)
