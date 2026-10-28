#$ test: compile-fail
#$ rules: UNS-12, UNS-1
# `[UNS-12]` — an `unsafe fn` body is no `unsafe` context: an unsafe step in it outside an `unsafe:`
# block is `E3100`, one for each line, which says the steps are the function's own and gives the
# block to write.

@safety("p points to four readable bytes")
unsafe fn read_u32_le(p: *u8) -> u32:
    return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)    #$ error[E3100]: `read_u32_le` is an `unsafe fn`, which says its callers must vouch; the reads through `p` on line 9 are this function's own unsafe steps, and nothing says why they are sound; put line 9 in an `unsafe:` block with a `# SAFETY:` note saying why the read is safe: above line 9 write `# SAFETY:` and the reason, then `unsafe:`, and indent line 9 under it

@safety("p points to a writable byte")
unsafe fn clear(p: *mut u8):
    p.write(0)    #$ error[E3100]: the write through `p` on line 13 is this function's own unsafe step, and nothing says why it is sound
    # A closure written in the body is the function's too.
    later = fn() => p.write(1)    #$ error[E3100]: `clear` is an `unsafe fn`, which says its callers must vouch; the write through `p` on line 15
    later()

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
