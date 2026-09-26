#$ test: compile-fail
#$ rules: TXT-2
#$ error[E2020]: expected `str`, found `Array[u8]`
#$ error[E2020]: expected `String`, found `Array[u8]`
# `[TXT-2]` — nothing becomes a `str` without validation: an `Array[u8]` is bytes, not text,
# and converts to neither `str` nor `String` implicitly (D-201).

fn main():
    bytes: Array[u8] = [104, 105]
    t: str = bytes
    s: String = bytes
