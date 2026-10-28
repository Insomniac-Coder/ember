#$ test: compile-fail
#$ rules: R6
#$ help: fn next_word(line: StrView) changes pos -> StrView from line:
## R6: with `from src` the body may give back only a view of what `src` names (AN-22, E3062).

struct Parser:
    src: StrView
    pos: int = 0

    fn next_word(line: StrView) changes pos -> StrView from src:
        end = line.find(" ").unwrap_or(line.len())
        this.pos += end + 1
        return line[..end]    #$ error[E3062]: `next_word` returns a view of `line`, but its first line says `from src`; return a view of `src`, or write `-> StrView from line`

fn main():
    text: String = "let x = 1"
    p = Parser(src=text)
    println(p.next_word(text))
