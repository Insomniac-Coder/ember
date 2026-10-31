#$ test: compile-fail
#$ rules: LT-1a
#$ help: rename the parameter
# `[LT-1a]` — `from` names a parameter, or a field of `this` written bare; a name that is both could
# mean either, so it is refused until the parameter is renamed.

struct Parser:
    src: StrView

    fn next_word(src: StrView) -> StrView from src:    #$ error[E2118]: `src` is both a parameter and a field of `Parser`, so `from src` could mean either
        return src

fn main():
    p = Parser(src = "a b")
    println(p.next_word("c"))
