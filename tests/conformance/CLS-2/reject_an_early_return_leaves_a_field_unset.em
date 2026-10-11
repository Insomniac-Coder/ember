#$ test: compile-fail
#$ rules: CLS-2
# D-565: a constructor's `return` hands the object to its caller, so every
# field without a default holds a value on that path.

class Named:
    name: String

    fn init(self, owned name: String, skip: bool):
        if skip:
            return #$ error[E2100]: field `name` is not definitely initialized by class `init`
        self.name = name

fn main():
    n = Named("x", true)
    println(n.name.len())
