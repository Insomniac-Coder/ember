#$ test: run-pass
#$ rules: UNS-1
#$ stdout: 513
#$ stdout: 9 2
# `[UNS-1]` — reading and writing through a raw pointer, and offsetting one, as IX.4's example
# spells them (`p.read()`, `p.write(v)`, `p.add(n)`), each inside `unsafe` (D-748).

# SAFETY: the caller guarantees `p` points to two readable bytes
unsafe fn read_u16_le(p: *u8) -> u16:
    return (p.read() as u16) | ((p.add(1).read() as u16) << 8)

fn main():
    a: Array[u8] = [1, 2, 3]
    # SAFETY: `a` holds three bytes
    unsafe:
        println(read_u16_le(a.as_span().as_ptr()))
    b: Array[int] = [1, 2]
    p = b.as_mut_span().as_mut_ptr()
    # SAFETY: `b` holds two elements and is not otherwise borrowed while written
    unsafe:
        p.write(9)
    println(b[0], b[1])
