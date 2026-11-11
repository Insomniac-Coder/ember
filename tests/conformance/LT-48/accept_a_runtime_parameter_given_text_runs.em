#$ test: run-pass
#$ rules: LT-48, LT-1
#$ stdin: m
#$ stdout: mine
# `[LT-48]` (D-967) — a `runtime` parameter's kind is checked while running whatever view type the
# call's result is: `title` takes a `StrView`, given a `String` (converted first), and a `Mine`'s
# `title` gives back its own name, so the change of `s` is safe and `show` prints `mine`. It was
# refused when compiling (`E3021`): the check looked for `s`'s loan among the call's arguments and
# missed it in the converted view.

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
    p: Box[dyn Titled] = Box(Mine(name=String("mine"))) if input() == "m" else Box(Yours(name=String("yours")))
    show(p)
