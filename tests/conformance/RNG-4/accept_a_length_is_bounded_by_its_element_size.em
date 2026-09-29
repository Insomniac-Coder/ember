#$ test: run-pass
#$ rules: RNG-4, HEAP-8
#$ profiles: debug, release, shipping
#$ stdout: 48
#$ stdout: 32
#$ assert-c-count: contains("ember_ck_mul_i64(") == 1
# `[RNG-4]`, `[HEAP-8]` — no list holds more than `PTRDIFF_MAX` bytes, so a
# view of 8-byte numbers has at most `PTRDIFF_MAX / 8` of them and
# `len() * 8` cannot overflow: no check. A view of bytes can have
# `PTRDIFF_MAX` of them, and its `len() * 8` keeps its check.

fn bytes_of(xs: Span[int]) -> int:
    return xs.len() * 8

fn bits_of(bs: Span[u8]) -> int:
    return bs.len() * 8

fn main():
    xs: Array[int] = [1, 2, 3]
    bs: Array[u8] = [1, 2]
    println(bytes_of(xs.as_span()) + bytes_of(xs.as_span()))
    println(bits_of(bs.as_span()) + bits_of(bs.as_span()))
