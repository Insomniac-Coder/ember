#$ test: compile-pass
#$ rules: UNS-8
## design rules: R16
## R16: "before a block ... it says "this code vouches", with a `# SAFETY:` note saying why (W3012 when
## missing)" (today's message).

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
    if parse(bytes.as_span()) exists as v:
        println(v)
