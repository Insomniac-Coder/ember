#$ test: compile-fail
#$ rules: UNS-1, UNS-12, FFI-54
## design rules: R16
## R16: "Inside an `unsafe fn` body every unsafe operation sits in its own `unsafe:` block (E3100)"
## (AN-79's OLD helper; section 3's message, in this program's lines).

@safety("p points to four readable bytes")
unsafe fn read_u32_le(p: *u8) -> u32:
    return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)    #$ error[E3100]: `read_u32_le` is an `unsafe fn`, which says its callers must vouch; the reads through `p` on line 8 are this function's own unsafe steps, and nothing says why they are sound; put line 8 in an `unsafe:` block with a `# SAFETY:` note saying why the read is safe: above line 8 write `# SAFETY:` and the reason, then `unsafe:`, and indent line 8 under it

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
