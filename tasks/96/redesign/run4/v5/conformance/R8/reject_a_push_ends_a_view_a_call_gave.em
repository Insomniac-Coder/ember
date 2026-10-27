#$ test: compile-fail
#$ rules: R8, R6
#$ help: n = String(first(names))
## R8 (b): every view a call gives is an address, which ends at a change that may reshape what it points
## into (here all of `names`, R6). AN-26, point 6.

fn first(xs: Array[String]) -> StrView:
    return xs[0]

fn main():
    names = [String("ann"), String("bob")]
    n = first(names)
    names.push(String("cy"))
    println(n)    #$ error[E3060]: `names.push(String("cy"))` (line 13) may move the strings `n` points into (`first` returns a view of `xs`, here `names`), and `n` is used on line 14; use `n` before line 13, or keep a copy of your own: write `n = String(first(names))` in place of line 12
