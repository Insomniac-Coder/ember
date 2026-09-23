#$ test: compile-fail
#$ rules: ATT-1
#$ profiles: debug
#$ error[E0104]: `@derive` does not apply to a fn
#$ help: it applies to: struct, enum, class
# An attribute on a declaration it does not apply to is `E0104`, naming the
# declarations it does apply to.

@derive(Copy)
fn main():
    println(1)
