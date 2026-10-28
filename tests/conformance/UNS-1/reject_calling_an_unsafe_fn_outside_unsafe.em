#$ test: compile-fail
#$ rules: UNS-1, UNS-12
# `[UNS-1]` — an `unsafe fn` says that whoever calls it must vouch; a call outside `unsafe` is
# `E3100`, whose message names the call and its line and gives the block to write.

@safety("p points to four readable bytes")
unsafe fn read_u32_le(p: *u8) -> u32:  # SAFETY: the callers keep the obligation `@safety` states
    # SAFETY: the caller promised four readable bytes at `p` (@safety above); these are bytes 0 to 3
    unsafe:
        return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4:
        return None
    return Some(read_u32_le(data.as_ptr()))    #$ error[E3100]: `read_u32_le(data.as_ptr())` (line 15) calls an `unsafe fn`, whose callers must make true what its `@safety` says; put line 15 in an `unsafe:` block with a `# SAFETY:` note saying why that holds here: above line 15 write `# SAFETY:` and the reason, then `unsafe:`, and indent line 15 under it

fn main():
    bytes: Array[u8] = [1, 0, 0, 0]
    if Some(v) = parse(bytes.as_span()):
        println(v)
