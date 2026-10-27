#$ test: compile-pass
#$ rules: R16
#$ warning[W3013]: `read_u32_le` says what its callers must make true with `@safety`; a `# SAFETY:` note says why one block is sound and goes above each `unsafe:` block inside it: remove this line
## R16: "W3013 for a `# SAFETY:` note on an `unsafe fn` line" (AN-79's OLD note, kept above the
## `unsafe fn` line; the warning is on that note's line, so it is written in the header).

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
    if parse(bytes.as_span()) exists as v:
        println(v)
