#$ test: run-pass
#$ rules: R6, R8
#$ stdout: let x let
## R6: `-> StrView from src` narrows a method's result to what one field of `this` views, so the
## result points into the text, not into `p`: `p.next_word()` may change `p.pos` while `a` is used,
## and `first_word` may return a word of a parser that ends (AN-22's NEW).

struct Parser:
    src: StrView
    pos: int = 0

    fn next_word() -> StrView from src:
        rest = this.src[this.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        this.pos += end + 1
        return rest[..end]

fn first_word(text: StrView) -> StrView:
    p = Parser(src=text)
    return p.next_word()

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    a = p.next_word()
    b = p.next_word()
    println(a, b, first_word(text))
