#$ test: compile-fail
#$ rules: OWN-3
#$ help: to keep `xs`, walk it without `owns` and keep copies: `for x in xs:` with `kept.push(copy of x)`
## design rules: R3, R1
## R3, R1: `for x in owns xs:` hands `xs` over to the loop, so a later use of `xs` is E3040.

fn main():
    xs = [String("a"), String("b")]
    kept: Array[String] = []
    for x in owns xs:
        kept.push(x)
    println(xs.len())          #$ error[E3040]: `xs` was handed over to the loop on line 9 (`for x in owns xs:`) and is used on line 11
