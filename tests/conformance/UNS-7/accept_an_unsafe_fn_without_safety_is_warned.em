#$ test: compile-pass
#$ rules: UNS-7
# `[UNS-7]` — every `unsafe fn`, not only a `pub` one, carries `@safety`; without it, `L3015` names
# the first unsafe step its body takes, and what the attribute must say about it.

unsafe fn read_u32_le(p: *u8) -> u32:    #$ warning[L3015]: `read_u32_le` is an `unsafe fn` that reads through the raw pointer `p`, and nothing says what its callers must make true for that; say it above the function: `@safety("...")`, with what `p` must point to
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
