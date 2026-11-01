#$ test: compile-fail
#$ rules: LT-1a
# `[LT-1a]` — under `from src` the body may give only a view of `src`: a view of `this.buf` is
# `E3062`, whose message names the function and what the result points into, and whose help
# writes the `from` that names it (the spec's own example).
#$ help: return a view of `src`, or write `-> StrView from buf`

struct Parser:
    buf: StrView

    fn next_word(src: StrView) -> StrView from src:
        return this.buf    #$ error[E3062]: `next_word` returns a view of `this.buf`, but its first line says `from src`

fn main():
    p = Parser(buf="a b")
    println(p.next_word("c"))
