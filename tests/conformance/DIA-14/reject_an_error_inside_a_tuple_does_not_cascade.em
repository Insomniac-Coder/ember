#$ test: compile-fail
#$ rules: DIA-14, DIA-2
# `[DIA-14]` (D-853) - a tuple holding a failed expression has no type to blame: `sort`, `max`
# and `<` on it report nothing more than the first error.

fn main():
    a = (undefined_name, 2)    #$ error[E1010]: cannot find `undefined_name` in this scope
    b = (1, 2)
    xs = [(missing, 1), (2, 3)]    #$ error[E1010]: cannot find `missing` in this scope
    xs.sort()
    println(max(a, b))
    println(a < b)
