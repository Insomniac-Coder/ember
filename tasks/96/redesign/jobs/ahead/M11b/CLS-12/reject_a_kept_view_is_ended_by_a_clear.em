# Clause CLS-12.1: "any other as a `ref T` view into the kept value, which clearing it ends (a later
# use is `E3060`)"
#$ test: compile-fail
#$ rules: CLS-12, EFF-27, BRW-17
# `n` views the kept label, which the change of `name` clears. No text in the spec: code only.

class Tag:
    name: String

    fn init(owns name: String):
        this.name = name

    @cached
    fn label() -> String:
        return f"<{this.name}>"

fn main():
    t = Tag(String("a"))
    n = t.label()
    t.name = String("b")
    println(n)    #$ error[E3060]
