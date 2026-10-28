#$ test: compile-fail
#$ rules: BRW-19
#$ help: fn next_word() -> StrView from src:
## design rules: R6, R8
## R6: with no `from`, `p.next_word()` points into `p`, a local that ends when `first_word` returns, so
## returning it is refused (AN-22, E3063). The compiler works the source out from the body to write
## the line, and never applies it silently (R17).

struct Parser:
    src: StrView
    pos: int = 0

    fn next_word() -> StrView:
        rest = this.src[this.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        this.pos += end + 1
        return rest[..end]

fn first_word(text: StrView) -> StrView:
    p = Parser(src=text)
    return p.next_word()    #$ error[E3063]: `p.next_word()` points into `p` (with no `from`, a method's result points into its object), and `p` ends when `first_word` returns; if `next_word`'s result points only into the text, write `fn next_word() -> StrView from src:`

fn main():
    text: String = "let x = 1"
    println(first_word(text))
