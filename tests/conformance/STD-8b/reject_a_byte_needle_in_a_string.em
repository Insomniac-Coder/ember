#$ test: compile-fail
#$ rules: STD-8b
## `[STD-8b]` — a byte needle (`Span[u8] in s`) is `E2226`; byte search is on `s.as_bytes()`.

fn main():
    t = "héllo"
    needle: Array[u8] = [195]
    println(needle.as_span() in t)    #$ error[E2226]: `StrView` does not implement `Contains[Span[u8]]`
