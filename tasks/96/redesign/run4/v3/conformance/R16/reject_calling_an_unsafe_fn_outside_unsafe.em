#$ test: compile-fail
#$ rules: R16
## R16: an `unsafe fn` says "whoever uses this must vouch"; calling it outside `unsafe` is E3100
## (today's message, AN-79).

@safety("p points to four readable bytes")
unsafe fn read_u32_le(p: *u8) -> u32:
    # SAFETY: the caller promised four readable bytes at `p` (@safety above); these are bytes 0 to 3
    unsafe:
        return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4:
        return None
    return Some(read_u32_le(data.as_ptr()))    #$ error[E3100]: calling an `unsafe fn` requires an `unsafe` block

fn main():
    bytes: Array[u8] = [1, 0, 0, 0]
    if parse(bytes.as_span()) exists as v:
        println(v)
