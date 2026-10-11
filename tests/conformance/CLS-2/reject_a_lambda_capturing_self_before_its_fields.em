#$ test: compile-fail
#$ rules: CLS-2
# `[CLS-2]` — a closure capturing `self` uses it as a whole.

class Named:
    name: String
    size: int

    fn init(self, owned name: String):
        measure = fn() -> int => self.name.len() #$ error[E2100]: `self` is used before all class fields are initialized
        self.name = name
        self.size = measure()

fn main():
    n = Named("x")
    println(n.size)
