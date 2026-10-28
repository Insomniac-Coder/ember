#$ test: compile-pass
#$ rules: UNS-8, UNS-7
#$ warning[W3013]: `read_u32_le` says what its callers must make true with `@safety`; a `# SAFETY:` note says why one block is sound and goes above each `unsafe:` block inside it: remove this line
# `[UNS-7]`, `[UNS-8]` — an `unsafe fn` says what its callers must make true with `@safety`, its only
# note; a `# SAFETY:` note says why one block is sound, so one on an `unsafe fn` line is `W3013`.

@safety("p points to four readable bytes")
# SAFETY: the caller guarantees `p` points to four readable bytes
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
    if Some(v) = parse(bytes.as_span()):
        println(v)
