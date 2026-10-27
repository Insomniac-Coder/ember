#$ test: compile-fail
#$ rules: R3
#$ help: hold the child in its own block: `left: Option[owns Tree]`
## R3: a type that contains itself with no block of its own between would have no end (E2200;
## AN-41: today the checker overflows its stack). The nearest program that is allowed is
## `run_a_value_stored_into_an_owns_place_gets_its_own_block`.

struct Tree:
    value: int
    left: Option[Tree] = None  #$ error[E2200]: `Tree` contains itself through `left`, so a `Tree` would have no end

fn main():
    t = Tree(value=1)
    println(t.value)
