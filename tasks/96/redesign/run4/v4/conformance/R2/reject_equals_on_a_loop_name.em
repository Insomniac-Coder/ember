#$ test: compile-fail
#$ rules: R2
#$ help: to change the elements of `xs` write `for ref mut x in xs:`
## R2: a loop name stands for what the loop gave and keeps it for the whole round: `=` on it is
## refused (E3023), unless it is `ref mut`.

fn main():
    xs = [1, 2]
    for x in xs:
        x = 0                  #$ error[E3023]: `x` is the element the loop gives and stays it for the whole round
    println(xs[0], xs[1])
