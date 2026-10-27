#$ test: compile-fail
#$ rules: R6, R8
#$ help: fn next_word() -> StrView from src:
## R6: with no `from`, a method's result points into all of its object, so a change of any field of
## `p` while the result is used is refused (R8 (g); AN-22, E3021).

struct Parser:
    src: StrView
    pos: int = 0

    fn next_word() -> StrView:
        rest = this.src[this.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        this.pos += end + 1
        return rest[..end]

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    a = p.next_word()
    b = p.next_word()    #$ error[E3021]: `p.next_word()` (line 21) changes `p.pos`, and `a` (line 20) points into all of `p` (with no `from`, a method's result points into its object) and is used on line 22; if `next_word`'s result points only into the text, write `fn next_word() -> StrView from src:`; or use `a` before line 21: write `println(a)` above line 21 and `println(b)` in place of line 22
    println(a, b)
