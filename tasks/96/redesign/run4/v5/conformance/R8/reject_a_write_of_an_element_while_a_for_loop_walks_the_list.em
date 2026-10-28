#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: for ref mut x in xs:
## design rules: R8
## R8 (c), verdict 95: while a `for` loop walks a list, nothing changes it through its own name, a
## write of one element included, as Rust refuses `xs[0] = 5` inside `for x in &xs`; the loop's own
## handle changes the element it is on. Rules section 3's E3020 message, in this program's lines.

fn main():
    xs = [1, 2, 3]
    for x in xs:
        xs[0] = 5    #$ error[E3020]: `xs[0] = 5` (line 11) writes an element of `xs`, which this loop walks (line 10); to change the element the loop is on, write `for ref mut x in xs:` in place of line 10 and `x = 5` in place of line 11
    println(xs)
