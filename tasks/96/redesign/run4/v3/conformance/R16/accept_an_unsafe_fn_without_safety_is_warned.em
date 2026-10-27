#$ test: compile-pass
#$ rules: R16
## R16: "`@safety` saying what they must make true (L3015 when missing, on every `unsafe fn`)", not only
## a `pub` one (section 3's message, word for word; see the matrix, R16-2, on what it can know).

unsafe fn read_u32_le(p: *u8) -> u32:    #$ warning[L3015]: say what callers of `read_u32_le` must make true: `@safety("p points to four readable bytes")`
    # SAFETY: the caller promised four readable bytes at `p`; these are bytes 0 to 3
    unsafe:
        return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4:
        return None
    # SAFETY: the length check above guarantees four readable bytes
    unsafe:
        return Some(read_u32_le(data.as_ptr()))

fn main():
    bytes: Array[u8] = [1, 0, 0, 0]
    if parse(bytes.as_span()) exists as v:
        println(v)
