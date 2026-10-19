#$ test: run-pass
#$ rules: HEAP-9
#$ profiles: debug, development
#$ stdout: 16 0 0 0
# `[HEAP-9]` — a list's storage is aligned to its element type's alignment: a `u128` is aligned to
# 16 bytes, and so is every buffer an `Array[u128]` holds, from its first push through each growth
# and a clone.

fn misaligned(xs: Span[u128]) -> int:
    p: *u128 = xs.as_ptr()
    unsafe:  # SAFETY: only the address is read, as a number; nothing is accessed through it
        return ((p as usize) % 16) as int

fn main():
    xs: Array[u128] = Array[u128]()
    xs.push(1)
    first = misaligned(xs.as_span())
    for i in 0..1000:
        xs.push(i as u128)
    grown = misaligned(xs.as_span())
    copy = xs.clone()
    println(align_of[u128](), first, grown, misaligned(copy.as_span()))
