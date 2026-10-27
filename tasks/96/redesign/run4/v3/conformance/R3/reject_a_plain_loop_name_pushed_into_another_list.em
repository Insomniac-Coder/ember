#$ test: compile-fail
#$ rules: R3, R2
#$ help: to take the elements out of `xs` write `for x in owns xs:`; to keep copies write `kept.push(copy of x)`
## R3, R2: without `owns` the loop name only reads each element, so storing it elsewhere would hand
## over what it does not own (E3013).

fn main():
    xs = [String("a"), String("b")]
    kept: Array[String] = []
    for x in xs:
        kept.push(x)           #$ error[E3013]: `kept.push(x)` would hand over an element of `xs`, but `x` only reads it
    println(kept.len())
