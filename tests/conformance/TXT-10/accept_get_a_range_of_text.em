#$ test: run-pass
#$ rules: TXT-10
#$ stdout:
#$ Some('h') Some('é') None Some('llo') Some('h') Some('h') Some('lo') None None
#$ None None
# `[TXT-10]` — `s.get(range)` is `Some` of the slice when the range is in
# order, within the text and on character boundaries, and `None` otherwise
# (where `s[range]` panics), for any of the prelude's ranges; it never
# panics, even for an end no text reaches.

s = "héllo"
println(s.get(0..1), s.get(1..3), s.get(1..2), s.get(3..), s.get(..1), s.get(0..=0), s.get(4..=5), s.get(5..9), s.get(3..2))
big: u64 = 18446744073709551615
println(s.get(0..big), s.get(0..=9223372036854775807))
