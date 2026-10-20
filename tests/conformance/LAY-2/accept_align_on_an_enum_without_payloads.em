#$ test: run-pass
#$ rules: LAY-2, HEAP-9
#$ profiles: debug, development, release
#$ stdout: 64 64 128 64 128
#$ stdout: Fast true 1 Slow
#$ stdout: 0 0 0 0
#$ stdout: 2 true
# `[LAY-2]` — `@align(N)` on an enum without payloads raises its alignment to `N` and makes its size
# a multiple of `N`, as on any other type, and every place holding it honours `N`: a local, a field,
# a tuple's element, an `Array`'s buffer (`[HEAP-9]`). It is still a value of its variants: it
# compares, prints, matches and converts to its discriminant.

@align(64)
enum Mode:
    Slow
    Fast

struct Holder:
    tag: u8
    mode: Mode

fn off(p: *Mode) -> int:
    unsafe:  # SAFETY: only the address is read, as a number; nothing is accessed through it
        return ((p as usize) % 64) as int

fn main():
    println(size_of[Mode](), align_of[Mode](), size_of[Holder](), offset_of[Holder](mode), size_of[(u8, Mode)]())
    m = Mode.Fast
    modes = [Mode.Slow, Mode.Fast, Mode.Slow]
    println(m, modes[1] == Mode.Fast, m as int, modes[2])
    h = Holder(tag = 1, mode = Mode.Slow)
    pair = (7 as u8, Mode.Fast)
    println(off(ref_to_ptr(ref m)), off(ref_to_ptr(ref h.mode)), off(ref_to_ptr(ref pair.1)), off(modes.as_span().as_ptr()))
    kept: Option[Mode] = Some(Mode.Fast)
    fast = 0
    for x in modes:
        match x:
            Mode.Fast:
                fast += 1
            Mode.Slow:
                pass
    if Some(Mode.Fast) = kept:
        fast += 1
    println(fast, kept == Some(Mode.Fast))
