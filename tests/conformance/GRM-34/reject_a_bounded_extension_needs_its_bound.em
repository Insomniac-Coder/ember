#$ test: compile-fail
#$ rules: GRM-34, TYP-17
# `[GRM-34]` — an extension under a bound gives its interface only to the
# instances whose arguments meet it: `Maybe[int]` is not `Describe`.

interface Describe:
    fn describe() -> StrView

enum Maybe[T]:
    Nothing
    Just(T)

extend[U: needs Describe] Maybe[U] implements Describe:
    fn describe() -> StrView:
        return "maybe"

[D: needs Describe]
fn show(d: D) -> StrView:
    return d.describe()

println(show(Maybe[int].Nothing))   #$ error[E2040]: `Maybe[i64]` does not implement `Describe`, which `D` requires
