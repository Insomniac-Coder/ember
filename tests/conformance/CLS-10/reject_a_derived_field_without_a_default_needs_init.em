#$ test: compile-fail
#$ rules: CLS-10
# `[CLS-10]` — a derived class that adds a field with no default must declare
# an `init` (`E2101`), since its base's constructor cannot set that field.

open class Script:
    name: String

    fn init(mut self, owned name: String):
        self.name = name

class Bad(Script):
    width: int              #$ error[E2101]: class `Bad` adds a field `width` with no default, so it must declare `init`

fn main():
    pass
