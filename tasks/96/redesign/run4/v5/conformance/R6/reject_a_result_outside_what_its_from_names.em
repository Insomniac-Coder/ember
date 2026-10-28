#$ test: compile-fail
#$ rules: LT-1a
#$ help: fn next_word(line: StrView) -> StrView from line:
## design rules: R6
## R6: with `from src` the body may give back only a view of what `src` names (AN-22, E3062).

struct Parser:
    src: StrView
    pos: int = 0

    fn next_word(line: StrView) -> StrView from src:
        end = line.find(" ").unwrap_or(line.len())
        this.pos += end + 1
        return line[..end]    #$ error[E3062]: `next_word` returns a view of `line`, but its first line says `from src`; return a view of `src`, or write `-> StrView from line`

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    println(p.next_word(text))
