#$ test: compile-fail
#$ rules: R3
#$ help: remove `owns`: write `return xs`
## R3: `owns` is refused after `return` too, which already hands the value over (E0109;
## `V3/rules.md` section 4).

fn make() -> Array[int]:
    xs = [1, 2]
    return owns xs             #$ error[E0109]: `owns xs` is not needed: `return xs` already hands `xs` over

fn main():
    println(make().len())
