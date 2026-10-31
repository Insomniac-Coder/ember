#$ test: run-pass
#$ rules: TXT-5, TXT-9, TXT-1, LEX-20
#$ stdout: abc ab
#$ stdout: 3
## `[TXT-5]` — `String(s)` makes a `String` of the program's own from the text `s`, a `StrView`
## (`[TXT-1]`): the new one may change, and the text it was made from stays as it was. A string
## literal is a `StrView` (`[LEX-20]`).

fn size(s: StrView) -> int:
    return s.len()

fn main():
    view: StrView = "ab"
    mine = String(view)
    mine.push_str("c")
    println(mine, view)
    println(size(mine.as_str()))
