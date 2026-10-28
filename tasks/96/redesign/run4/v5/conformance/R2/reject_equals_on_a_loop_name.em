#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: to change the elements of `xs` write `for ref mut x in xs:`
## design rules: R2
## R2: a loop name stands for what the loop gave and keeps it for the whole round: `=` on it is
## refused (E3023), unless it is `ref mut`.

fn main():
    xs = [1, 2]
    for x in xs:
        x = 0                  #$ error[E3023]: `x` is the element the loop gives and stays it for the whole round
    println(xs[0], xs[1])
