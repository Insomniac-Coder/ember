#$ test: run-fail
#$ rules: LT-48, LT-1
#$ stdin: y
#$ panics: `s` changes while `r` looks at it: `p` is a `Yours`, whose `title` gives back the text it was handed
# `[LT-48]` (D-967, D-968) — given a `Yours`, whose `title` gives back the text it was handed, the
# program stops at `s.push_str("c")`, and the message says "text", not "list".

interface Titled:
    fn title(default: StrView) -> StrView

struct Mine implements Titled:
    name: String
    fn title(default: StrView) -> StrView:
        return this.name

struct Yours implements Titled:
    name: String
    fn title(default: StrView) -> StrView:
        return default

fn show(p: runtime dyn Titled):
    s = String("ab")
    r = p.title(s)
    s.push_str("c")
    println(r)

fn main():
    p: owns dyn Titled = owns Mine(name=String("mine")) if input() == "m" else owns Yours(name=String("yours"))
    show(p)
