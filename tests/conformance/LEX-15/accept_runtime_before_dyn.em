#$ test: parse-pass
#$ rules: LEX-15, LT-48
## `[LEX-15]` — `runtime` is a keyword after a parameter's `:` and directly before `dyn`
## (`fn show(p: runtime dyn Picker):`, `[LT-48]`); a name everywhere else, a parameter's name too.

interface Picker:
    fn pick() -> int

fn show(p: runtime dyn Picker, runtime: int):
    n = runtime + p.pick()
    println(n)
