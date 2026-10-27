#$ test: compile-fail
#$ rules: FN-1
#$ help: fn show(c: Counter):
## design rules: R4
## R4: `ref` in a parameter's type is refused: the mode goes on the parameter's name, and a plain
## parameter already reads the caller's counter (AN-27, rules section 3's E0117 case `c: ref Counter`).

class Counter:
    n: int = 0

fn show(c: ref Counter):    #$ error[E0117]: the mode goes on the name: write `c: Counter`
    println(c.n)

fn main():
    counter = Counter()
    show(counter)
