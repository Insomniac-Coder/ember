#$ test: compile-fail
#$ rules: GRM-34
# `[GRM-34]` — each parameter of an extension over a type parameter standing alone is the target or
# one an implemented interface names; another means nothing and is refused.

interface Tag:
    fn tag(self) -> int

extend[T, U] T implements Tag:    #$ error[E2020]: `U` is named by neither the type extended nor an interface it implements
    fn tag(self) -> int:
        return 1

fn main():
    pass
