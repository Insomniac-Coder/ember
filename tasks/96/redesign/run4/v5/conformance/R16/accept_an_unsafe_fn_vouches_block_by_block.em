#$ test: run-pass
#$ rules: TIER-1
#$ stdout: 1
## design rules: R16
## R16: "before a declaration it says "whoever uses this must vouch", with `@safety` saying what they
## must make true ...; before a block ... "this code vouches", with a `# SAFETY:` note saying why"; the
## `unsafe fn`'s own steps sit in an `unsafe:` block (AN-79's NEW).

@safety("p points to four readable bytes")
unsafe fn read_u32_le(p: *u8) -> u32:
    # SAFETY: the caller promised four readable bytes at `p` (@safety above); these are bytes 0 to 3
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
