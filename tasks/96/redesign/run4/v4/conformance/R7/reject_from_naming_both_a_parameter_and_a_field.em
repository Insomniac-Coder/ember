#$ test: compile-fail
#$ rules: R7, R6
#$ help: fn next_word(line: StrView) changes pos -> StrView from src:
## R7, R6: a `from` that names two things (a parameter and a field of one name) is refused (E2118, the
## R7 row of `V4/rules.md` section 3). AN-22, point 6.

struct Parser:
    src: StrView
    pos: int = 0

    fn next_word(src: StrView) changes pos -> StrView from src:    #$ error[E2118]: `src` is both a parameter of `next_word` and a field of `Parser`, so `from src` could mean either; rename the parameter: for a view of the field write `fn next_word(line: StrView) changes pos -> StrView from src:`, for a view of the parameter `fn next_word(line: StrView) changes pos -> StrView from line:`
        rest = this.src[this.pos..]
        end = rest.find(" ").unwrap_or(rest.len())
        this.pos += end + 1
        return rest[..end]

fn main():
    p = Parser(src="let x")
    println(p.next_word("y"))
