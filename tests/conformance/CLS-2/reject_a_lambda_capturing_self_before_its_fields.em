#$ test: compile-fail
#$ rules: CLS-2
# `[CLS-2]` — a closure capturing `self` uses it as a whole.

class Named:
    name: String
    size: int

    fn init(owns name: String):
        measure = fn() -> int => this.name.len() #$ error[E2100]: `this` is used before all class fields are initialized
        this.name = name
        this.size = measure()

fn main():
    n = Named("x")
    println(n.size)
