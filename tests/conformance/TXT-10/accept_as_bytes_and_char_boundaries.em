#$ test: run-pass
#$ rules: TXT-10
#$ stdout:
#$ 6 104 195 false true true false
#$ 65
# `[TXT-10]` — `as_bytes()` is the text's `Span[u8]`, borrowing it as the
# text is; `is_char_boundary(i)` says whether byte `i` starts a character or
# ends the text (an `i` outside it is not a boundary).

s = "héllo"
b = s.as_bytes()
println(len(b), b[0], b[1], s.is_char_boundary(2), s.is_char_boundary(3), s.is_char_boundary(6), s.is_char_boundary(-1))
name: String = "Ada"
println(name.as_bytes()[0])
