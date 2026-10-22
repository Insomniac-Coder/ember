#$ test: compile-fail
#$ rules: HR-14
#$ help: it applies to: field
# `@renamed_from` is the one attribute a payload field takes: another, written where the grammar
# allows attributes, is `E0104`, naming where it applies.

enum Shape:
    Dot
    Circle(@soa(flatten) radius: int)  #$ error[E0104]: `@soa` does not apply to a payload field

fn main():
    println(1)
