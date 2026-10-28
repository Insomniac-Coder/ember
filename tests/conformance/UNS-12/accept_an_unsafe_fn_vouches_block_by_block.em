#$ test: run-pass
#$ rules: UNS-12, UNS-1, UNS-7
#$ stdout: 1
# `[UNS-12]` — an `unsafe fn` says that its callers must vouch, with `@safety` saying what they must
# make true; its own unsafe steps sit in an `unsafe:` block of their own, whose `# SAFETY:` note says
# why they are sound.

@safety("p points to four readable bytes")
unsafe fn read_u32_le(p: *u8) -> u32:  # SAFETY: the callers keep the obligation `@safety` states
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
