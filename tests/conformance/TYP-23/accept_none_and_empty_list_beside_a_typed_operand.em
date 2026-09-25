#$ test: run-pass
#$ rules: TYP-23, STR-5
#$ stdout: false true true true true
# `[TYP-23]` — in a comparison, `None` or `[]` beside a typed operand takes
# that operand's type, on either side.

fn main():
    o: Option[int] = Some(4)
    xs: Array[String] = []
    println(o == None, None != o, o == Some(4), xs == [], [] == xs)
