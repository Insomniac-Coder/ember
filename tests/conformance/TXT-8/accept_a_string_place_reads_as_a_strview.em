#$ test: run-pass
#$ rules: TXT-8
#$ stdout: 3 abc
#$ abcd
## `[TXT-8]` - a `String` place reads as a `StrView` wherever one is expected: an argument, an
## initialiser, a method's receiver; the view lives as long as the text it points into, and a reshape
## after its last use is fine.

fn show(v: StrView) -> int:
    return v.len()

fn main():
    s = String("abc")
    y: StrView = s
    println(show(s), y)
    s.push_str("d")
    println(s)
