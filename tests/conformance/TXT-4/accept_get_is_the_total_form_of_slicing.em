#$ test: run-pass
#$ rules: TXT-4
#$ stdout: Some('h') Some('é') None None None
#$ hé llo
## `[TXT-4]` - `s.get(a..b)` is the total form of `s[a..b]`: `None` where a bound is not on a character
## boundary or lies past the end, never a panic; a slice by byte offset is the text between.

fn main():
    s = "héllo"
    println(s.get(0..1), s.get(1..3), s.get(1..2), s.get(3..99), s.get(4..2))
    println(s[0..3], s[3..])
