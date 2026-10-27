#$ test: compile-fail
#$ rules: BRW-19
#$ help: return String(word)
## design rules: R8
## R8 (d), row 5: returning a view of a temporary is refused with row 5's message. AN-09, point 6.

fn clean(line: StrView) -> StrView:
    word = line.to_lower().trim()
    return word    #$ error[E3063]: `word` is a view of the lowered copy that `line.to_lower()` made on line 7, which ends when `clean` returns; return `String(...)` to hand back your own copy: `return String(word)`, with the result type `String`

fn main():
    println(clean(" Hi "))
