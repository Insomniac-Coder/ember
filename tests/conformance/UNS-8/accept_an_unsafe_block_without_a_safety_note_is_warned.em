#$ test: compile-pass
#$ rules: UNS-8
# `[UNS-8]` — before a block, `unsafe` says "this code vouches", with a `# SAFETY:` note saying why;
# a block without one is `W3012`.

@safety("p points to four readable bytes")
unsafe fn read_u32_le(p: *u8) -> u32:
    unsafe:    #$ warning[W3012]: this `unsafe` has no safety note
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
