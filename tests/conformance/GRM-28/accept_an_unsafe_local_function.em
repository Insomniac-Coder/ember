#$ test: run-pass
#$ rules: GRM-28, UNS-1, UNS-12
#$ stdout: 513
#$ stdout: 515

## `[GRM-28]` — a local function may be `unsafe`, as any `fn_decl`: its body, as any `unsafe fn`'s,
## vouches for its own steps block by block (`[UNS-12]`), and each call needs `unsafe` (`[UNS-1]`),
## whether it captures (here `extra`) or not.

fn main():
    a: Array[u8] = [1, 2, 3]
    extra = 2
    @safety("the caller guarantees `p` points to two readable bytes")
    unsafe fn read_u16_le(p: *u8) -> u16:
        return (p.read() as u16) | ((p.add(1).read() as u16) << 8)
    @safety("the caller guarantees `p` points to two readable bytes")
    unsafe fn read_plus_extra(p: *u8) -> int:
        return ((p.read() as int) | ((p.add(1).read() as int) << 8)) + extra
    # SAFETY: `a` holds three bytes
    unsafe:
        println(read_u16_le(a.as_span().as_ptr()))
        println(read_plus_extra(a.as_span().as_ptr()))
