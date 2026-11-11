#$ test: run-fail
#$ rules: LT-48, DIA-2
#$ stdin: y
#$ panics: `s` changes while `r` looks at it: `p` is a `Yours`, whose `title` gives back the text it was handed
# `[LT-48]` (D-968) — the stop message names what the method was handed by its own kind: a `String`
# is text, not a list (the compiler keeps a `String` as a list marked as text).

interface Titled:
    fn title(default: ref String) -> StrView

struct Mine implements Titled:
    name: String
    fn title(default: ref String) -> StrView:
        return this.name

struct Yours implements Titled:
    name: String
    fn title(default: ref String) -> StrView:
        return default

fn show(p: runtime dyn Titled):
    s = String("ab")
    r = p.title(s)
    s.push_str("c")
    println(r)

fn main():
    p: owns dyn Titled = owns Mine(name=String("mine")) if input() == "m" else owns Yours(name=String("yours"))
    show(p)
