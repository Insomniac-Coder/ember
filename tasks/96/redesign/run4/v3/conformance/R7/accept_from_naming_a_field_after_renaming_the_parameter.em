#$ test: run-pass
#$ rules: R7, R6
#$ stdout: let x
## R7, R6: with the parameter renamed, `from src` names only the field, so the two words point into the
## text and may be kept while `p.pos` changes. AN-22's NEW program, with the parameter of E2118's
## message.

struct Parser:
    src: StrView
    pos: int = 0

    fn next_word(line: StrView) changes pos -> StrView from src:
        rest = this.src[this.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        this.pos += end + 1
        return rest[..end]

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    a = p.next_word("")
    b = p.next_word("")
    println(a, b)
