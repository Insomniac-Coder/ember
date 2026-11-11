#$ test: run-pass
#$ rules: DSP-1, TXT-1
#$ stdout: mine
#$ ab
# `[DSP-1]` (D-969) — a method called through a `dyn` value may take a `StrView`. The C backend passes a
# view to a function it calls directly as two words and rebuilds it on entry; the method a table's
# adapter calls takes the view whole, as every function reached through a table does (a `Box[dyn I]`
# made of a value too).

interface Titled:
    fn title(default: StrView) -> StrView

struct Mine implements Titled:
    name: String
    fn title(default: StrView) -> StrView:
        return this.name

struct Echo implements Titled:
    fn title(default: StrView) -> StrView:
        return default

fn show(p: ref dyn Titled, s: StrView):
    println(p.title(s))

fn main():
    mine: Box[dyn Titled] = Box(Mine(name=String("mine")))
    echo: Box[dyn Titled] = Box(Echo())
    text = String("ab")
    show(mine, text)
    show(echo, text)
