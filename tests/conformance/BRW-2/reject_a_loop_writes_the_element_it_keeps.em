#$ test: compile-fail
#$ rules: BRW-2, BRW-1
# The other side: the element `best` keeps is still borrowed when a later turn writes the array,
# so the write conflicts with that live borrow.

fn main():
    xs: Array[String] = ["a", "bb", "c"]
    best: str = ""
    for i in 0..xs.len():
        if xs[i].len() > best.len():
            best = xs[i]
        xs[0] = "z"            #$ error[E3021]: `xs[…]` cannot be written while it is borrowed
    println(best)
