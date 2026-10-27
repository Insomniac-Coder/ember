#$ test: compile-fail
#$ rules: OWN-9
#$ help: remove `owns`: write `return xs`
## design rules: R3
## R3: `owns` is refused after `return` too, which already hands the value over (E0109;
## `V4/rules.md` section 4).

fn make() -> Array[int]:
    xs = [1, 2]
    return owns xs             #$ error[E0109]: `owns xs` is not needed: `return xs` already hands `xs` over

fn main():
    println(make().len())
