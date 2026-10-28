#$ test: compile-fail
#$ rules: R6, R7
#$ help: fn next_word(line: StrView) changes pos -> StrView from src:
## R6: `from` names one parameter or one field of `this`; a name that is both is refused, so the line
## never means two things (AN-22, E2118).

struct Parser:
    src: StrView
    pos: int = 0

    fn next_word(src: StrView) changes pos -> StrView from src:    #$ error[E2118]: `src` is both a parameter of `next_word` and a field of `Parser`, so `from src` could mean either; rename the parameter: for a view of the field write `fn next_word(line: StrView) changes pos -> StrView from src:`, for a view of the parameter `fn next_word(line: StrView) changes pos -> StrView from line:`
        rest = this.src[this.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        this.pos += end + 1
        return rest[..end]

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    println(p.next_word(text))
