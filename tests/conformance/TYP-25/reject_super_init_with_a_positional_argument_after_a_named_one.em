#$ test: compile-fail
#$ rules: TYP-25, CLS-3
## `[TYP-25]` - `super.init(...)` is such a call as well (`[CLS-3]`).

open class Panel:
    name: StrView
    visible: bool = true

class Side(Panel):
    size: int

    fn init(size: int):
        this.size = size
        super.init(visible=false, "a")    #$ error[E2021]: `super.init(visible=false, "a")` gives `"a"` after the named `visible=false`, and positional arguments come first; write `super.init(visible=false, name="a")`

fn main():
    s = Side(7)
    println(s.size)
