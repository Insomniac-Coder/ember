#$ test: compile-fail
#$ rules: SPN-1
# The `as_str()` twin of `reject_as_span_still_borrows`: a `str` points into
# its `String` (D-037 gave the spelling its loan), so growing the buffer while
# `v` is live is `E3021`. Before the fix this compiled and `v` dangled across
# the reallocation — the same defect D-022 fixed for spans.

fn main():
    s: String = String()
    s.push_str("hi")
    v: str = s.as_str()
    s.push_str("!")   #$ error[E3021]: `s` is borrowed here and mutably borrowed elsewhere
    println(v)
