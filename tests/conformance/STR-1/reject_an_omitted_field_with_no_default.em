#$ test: compile-fail
#$ rules: STR-1, TYP-25
## `[STR-1]` - a field with a default may be omitted; one without may not (`E2021`, as at any call).

struct Model:
    name: String = "m"
    id: int
    scale: int = 2

fn main():
    k = Model(scale=3)    #$ error[E2021]: `Model(scale=3)` gives nothing for `id`, which has no default: `Model(name: String = …, id: int, scale: int = …)`
    println(k.id)
